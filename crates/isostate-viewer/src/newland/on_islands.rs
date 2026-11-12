//! The ground of the new land, held to the core's on three islands: 2 (which
//! has tablelands by the sea), 3 (two towns at a lake's outlet) and 7 (the
//! island a game is played on unless another is asked for). Each island is
//! made once by the core's generator, with no economy on it: its cells carry
//! a made-up mix of arable, pasture, wood and heath, and its first towns
//! stand where the core put them.

use std::sync::{Arc, OnceLock};

use bevy::math::Vec2;

use super::ground::{metres, units};
use super::middles;
use super::NewLand;
use crate::replay::Replay;
use crate::world::{self, Ground, FARM, HEATH, PASTURE, SEA, VRES, WOOD};

struct Island {
    seed: u64,
    r: Replay,
    g: Ground,
    cen: Vec<(f32, f32)>,
}

impl Island {
    fn land(&self) -> &NewLand {
        self.r.land.as_deref().unwrap()
    }

    /// The ground's own height at a spot, lake beds and all.
    fn bare(&self, x: f32, z: f32) -> f32 {
        world::grid_at(&self.g.hgt, self.g.gw, self.g.gh, x, z)
    }
}

/// The three islands, made the first time a test asks for them.
fn islands() -> &'static [Island] {
    static MADE: OnceLock<Vec<Island>> = OnceLock::new();
    MADE.get_or_init(|| {
        use rayon::prelude::*;
        [2u64, 3, 7].into_par_iter().map(make).collect()
    })
}

fn make(seed: u64) -> Island {
    let (w, h) = (120usize, 84usize);
    let world = isostate_core::landcells::generate_land(seed, w, h, 6, &isostate_core::landform::LandParams::game());
    let lf = world.landform.as_deref().expect("the generator keeps its land");
    let n = w * h;
    let towns: Vec<String> = world.town_sites.iter().enumerate().map(|(t, c)| format!(r#"{{"name":"Town {t}","node":{c},"founded":1700}}"#)).collect();
    let header: crate::replay::Header = serde_json::from_str(&format!(
        r#"{{"meta":{{"W":{w},"H":{h},"start_year":1750,"n_years":1,"seed":{seed}}},"goods":[],"methods":[],"towns":[{}],"firms":[],"events":[],"uses":[],"arrays":[]}}"#,
        towns.join(",")
    ))
    .unwrap();
    let nt = towns.len();
    let mut r = Replay {
        header,
        w,
        h,
        n,
        ny: 1,
        y0: 1750,
        n_towns: nt,
        elev: isostate_core::record::elev_bytes(&world),
        terrain: world.terrain.clone(),
        river: isostate_core::record::river_bytes(&world),
        down: world.down.iter().map(|&d| d as i32).collect(),
        use_: vec![0; n],
        inten: vec![0; n],
        trans: vec![0; n],
        catch: vec![0; n],
        rent: Vec::new(),
        dist: Vec::new(),
        mix_a: vec![SEA; n],
        mix_b: vec![SEA; n],
        mix_s: vec![255; n],
        town_active: vec![1; nt],
        town_pop: vec![2000.0; nt],
        town_urban: vec![800.0; nt],
        town_wage: Vec::new(),
        town_price: Vec::new(),
        firm_first: Vec::new(),
        firm_len: Vec::new(),
        firm_off: Vec::new(),
        firm_k: Vec::new(),
        series_names: Vec::new(),
        series: Vec::new(),
        series_len: 0,
        filled: 1,
        owner: Vec::new(),
        built: Vec::new(),
        res: Vec::new(),
        works: Vec::new(),
        workings: Vec::new(),
        land: Some(Arc::new(NewLand::from_core(lf, w, h))),
    };
    // open land of all four kinds, two to a cell as the record keeps them: corn and grass or
    // corn and wood in the lowland, grass with wood or heath in the hills, wood and heath above
    for c in (0..n).filter(|&c| world.terrain[c] > 0) {
        let odd = (c % w + c / w) % 2 == 1;
        (r.mix_a[c], r.mix_b[c], r.mix_s[c]) = match (world.terrain[c], odd) {
            (1, false) => (FARM, PASTURE, 153),
            (1, true) => (FARM, WOOD, 179),
            (2, false) => (PASTURE, WOOD, 128),
            (2, true) => (PASTURE, HEATH, 153),
            (_, false) => (WOOD, HEATH, 102),
            (_, true) => (PASTURE, HEATH, 77),
        };
    }
    let g = world::build_ground(&r);
    let cen = crate::network::centres(&r);
    Island { seed, r, g, cen }
}

#[test]
fn each_land_cell_s_mean_height_as_drawn_is_the_core_s() {
    // The core's height of a cell is the mean of its dry nodes, kept as a byte of 5.9 m (the
    // byte stops at 1,500 m, which no island reaches). As drawn it is measured two ways, each
    // read back to metres through the stretch: at the middles of those nodes, where the
    // ground is the core's unless a river's bed has lowered it; and at every vertex that lies
    // in those nodes, where the vertices on a node's edge lean half on the node beside it,
    // which may be in the next cell, under the sea or in a lake.
    for isl in islands() {
        let (l, g) = (isl.land(), &isl.g);
        let per = VRES / l.fine;
        let step = 1500.0 / 255.0;
        // [at the nodes' middles, at every vertex]: cells within one step, within two, the sum of the differences, the worst
        let (mut cells, mut one, mut two, mut sum, mut worst) = (0usize, [0usize; 2], [0usize; 2], [0.0f64; 2], [0.0f32; 2]);
        for c in (0..isl.r.n).filter(|&c| isl.r.terrain[c] > 0) {
            let (cx, cy) = (c % l.w, c / l.w);
            let (mut drawn, mut count) = ([0.0f64; 2], [0usize; 2]);
            for nj in cy * l.fine..(cy + 1) * l.fine {
                for ni in cx * l.fine..(cx + 1) * l.fine {
                    let n = nj * l.fw + ni;
                    if l.lake[n] > 0 || l.height[n] <= 0.0 {
                        continue;
                    }
                    for vj in nj * per..(nj + 1) * per {
                        for vi in ni * per..(ni + 1) * per {
                            let m = metres(g.hgt[vj * g.gw + vi]).max(0.0) as f64;
                            if (vi % per, vj % per) == (per / 2, per / 2) {
                                drawn[0] += m;
                                count[0] += 1;
                            }
                            drawn[1] += m;
                            count[1] += 1;
                        }
                    }
                }
            }
            if count[0] == 0 {
                continue;
            }
            cells += 1;
            let core = isl.r.elev[c] as f32 * step;
            for way in 0..2 {
                let off = ((drawn[way] / count[way] as f64) as f32 - core).abs();
                one[way] += (off <= step) as usize;
                two[way] += (off <= 2.0 * step) as usize;
                sum[way] += off as f64;
                worst[way] = worst[way].max(off);
            }
        }
        let share = |k: usize| k as f32 / cells as f32;
        for (way, name) in ["at the middles of its dry nodes", "at every vertex in its dry nodes"].iter().enumerate() {
            eprintln!(
                "island {}: of {cells} land cells the mean height as drawn {name} is within a step of the byte (5.9 m) of the core's in {:.1}%, within two in {:.1}%; {:.1} m out on average, {:.0} m at the worst",
                isl.seed,
                100.0 * share(one[way]),
                100.0 * share(two[way]),
                sum[way] / cells as f64,
                worst[way]
            );
        }
        assert!(cells > 5000);
        // at the nodes: the core's, but for the rivers' beds
        assert!(share(one[0]) > 0.95 && share(two[0]) > 0.98, "island {}: at the nodes only {:.1}% of land cells are drawn within a step of the core's height and {:.1}% within two", isl.seed, 100.0 * share(one[0]), 100.0 * share(two[0]));
        assert!(sum[0] / (cells as f64) < 2.5, "island {}: at the nodes {:.1} m out on average", isl.seed, sum[0] / cells as f64);
        // at every vertex: within two steps in nine cells of ten
        assert!(share(one[1]) > 0.78 && share(two[1]) > 0.90, "island {}: only {:.1}% of land cells are drawn within a step of the core's height and {:.1}% within two", isl.seed, 100.0 * share(one[1]), 100.0 * share(two[1]));
        assert!(sum[1] / (cells as f64) < 4.5, "island {}: {:.1} m out on average", isl.seed, sum[1] / cells as f64);
    }
}

#[test]
fn the_shore_runs_between_a_dry_node_and_a_wet_one_and_nowhere_else() {
    // a vertex lies between one, two or four nodes: where all of them are dry it is above
    // the sea, and where all of them are under it so is the vertex, with no overshoot at all
    for isl in islands() {
        let (l, g) = (isl.land(), &isl.g);
        let per = VRES / l.fine;
        assert_eq!(per, 2, "two vertices to a node: one at its middle and one on its edge");
        let (mut dry, mut wet, mut between) = (0usize, 0usize, 0usize);
        for vj in 0..g.gh {
            for vi in 0..g.gw {
                // the nodes about the vertex along one axis: at a node's middle that node alone,
                // on a node's edge the two that meet there
                let about = |v: usize, most: usize| if v % 2 == 1 { (v / 2, v / 2) } else { ((v / 2).saturating_sub(1).min(most - 1), (v / 2).min(most - 1)) };
                let ((i0, i1), (j0, j1)) = (about(vi, l.fw), about(vj, l.fh));
                let heights: Vec<f32> = [(i0, j0), (i1, j0), (i0, j1), (i1, j1)].iter().map(|&(i, j)| l.height[j * l.fw + i]).collect();
                let k = vj * g.gw + vi;
                if heights.iter().all(|&h| h > 0.0) {
                    dry += 1;
                    assert!(g.hgt[k] > 0.0, "island {}: the vertex {vi},{vj} among dry nodes ({heights:?} m) is drawn at {}", isl.seed, g.hgt[k]);
                } else if heights.iter().all(|&h| h <= 0.0) {
                    wet += 1;
                    assert!(g.hgt[k] <= 0.0, "island {}: the vertex {vi},{vj} among nodes under the sea ({heights:?} m) is drawn at {}", isl.seed, g.hgt[k]);
                } else {
                    between += 1;
                }
            }
        }
        assert!(dry > 300_000 && wet > 200_000 && between > 3_000, "island {}: {dry} dry, {wet} wet, {between} between", isl.seed);
        // and a vertex at a node's middle stands at the node's own height, where no river's bed has lowered it
        let mut same = 0;
        for n in 0..l.fw * l.fh {
            let k = ((n / l.fw) * per + per / 2) * g.gw + (n % l.fw) * per + per / 2;
            assert!(g.hgt[k] <= units(l.height[n]) + 1e-6);
            same += ((g.hgt[k] - units(l.height[n])).abs() < 1e-6) as usize;
            assert_eq!(g.laid.as_ref().unwrap().metres[k], l.height[n]);
        }
        assert!(same * 100 > l.fw * l.fh * 97, "island {}: only {same} of {} nodes are drawn at their height", isl.seed, l.fw * l.fh);
    }
}

#[test]
fn every_river_point_s_ground_is_at_or_under_its_water() {
    for isl in islands() {
        let (mut points, mut km) = (0usize, 0.0f32);
        assert!(isl.g.rivers.len() > 40, "island {} has {} rivers drawn", isl.seed, isl.g.rivers.len());
        for (ri, rv) in isl.g.rivers.iter().enumerate() {
            for i in 0..rv.pts.len() {
                let p = rv.pts[i];
                let ground = isl.bare(p.x, p.y);
                // the bed is 0.003 under nothing: the water is drawn 0.009 above the level
                assert!(ground <= rv.level[i] + 0.0031, "island {}: river {ri} at {:.2},{:.2}: the ground stands at {ground}, the water's level is {}", isl.seed, p.x, p.y, rv.level[i]);
                // and under the banks' edge too, as near as the mesh's squares go
                if i > 0 {
                    assert!(rv.level[i] <= rv.level[i - 1], "island {}: river {ri} rises downstream at point {i}", isl.seed);
                    km += rv.pts[i - 1].distance(p) * 2.0;
                }
            }
            points += rv.pts.len();
        }
        eprintln!("island {}: {} rivers drawn, {points} points, {km:.0} km, every point's ground under its water", isl.seed, isl.g.rivers.len());
    }
}

#[test]
fn no_cell_s_middle_stands_on_water_or_marsh_where_its_cell_has_dry_ground() {
    for isl in islands() {
        let l = isl.land();
        let rivers = middles::Rivers::of(l);
        let (mut land, mut moved, mut poor, mut missed, mut under) = (0usize, 0usize, 0usize, 0usize, 0usize);
        for c in (0..isl.r.n).filter(|&c| isl.r.terrain[c] > 0) {
            land += 1;
            let (x, y) = isl.cen[c];
            // in its own cell, and not hard against its edge
            let (cx, cy) = ((c % l.w) as f32, (c / l.w) as f32);
            assert!(x > cx + 0.1 && x < cx + 0.9 && y > cy + 0.1 && y < cy + 0.9, "island {}: cell {c}'s middle is at {x},{y}", isl.seed);
            moved += ((x, y) != middles::jittered(c, l.w)) as usize;
            let here = middles::footing(l, &rivers, x, y);
            if here.tier() == 0 {
                // as drawn it is out of the water and above the beach, and all but a handful above the
                // line nothing is built under (the ground is read between vertices 250 m apart, and one
                // of them may be the shore's)
                let drawn = world::height_at(&isl.g, x, y);
                assert!(drawn > 0.022, "island {}: cell {c}'s middle at {x},{y} is drawn at {drawn}", isl.seed);
                assert!(world::water_dist(&isl.g, x, y) > 0.0, "island {}: cell {c}'s middle at {x},{y} is in the water as drawn", isl.seed);
                under += (drawn < 0.05) as usize;
                continue;
            }
            // on poorer ground only where the cell has none better: of the spots it was chosen from
            // (thirteen each way) none is of a better kind
            poor += 1;
            let best = |per: usize| {
                let mut best = (u8::MAX, (0.0, 0.0));
                for j in 0..per {
                    for i in 0..per {
                        let (px, py) = (cx + 0.14 + 0.72 * i as f32 / (per - 1) as f32, cy + 0.14 + 0.72 * j as f32 / (per - 1) as f32);
                        let there = middles::footing(l, &rivers, px, py);
                        if there.tier() < best.0 {
                            best = (there.tier(), (px, py));
                        }
                    }
                }
                best
            };
            let (tier, at) = best(13);
            assert!(tier >= here.tier(), "island {}: cell {c}'s middle at {x:.2},{y:.2} stands on {}, and {at:?} in the same cell is better", isl.seed, here.words());
            // and a grid twice as fine seldom finds firm, dry ground that the first did not: a patch
            // of it under 150 m across
            missed += (here.tier() >= 2 && best(26).0 == 0) as usize;
        }
        assert!(missed * 500 <= land, "island {}: {missed} cells have a patch of dry ground that their middle on marsh or water missed", isl.seed);
        assert!(under * 500 <= land, "island {}: {under} middles on dry, firm ground are drawn under the line nothing is built below", isl.seed);
        eprintln!("island {}: {under} middles on dry, firm ground are drawn under 0.05; {missed} cells have dry ground a finer grid finds and their middle missed", isl.seed);
        // the first towns, which the core puts on the water: none has its market place in it or on marsh
        for (t, town) in isl.r.header.towns.iter().enumerate() {
            let (x, y) = isl.cen[town.node];
            let here = middles::footing(l, &rivers, x, y);
            assert!(!here.water, "island {}: town {t}'s market place at {x:.2},{y:.2} is on water", isl.seed);
            eprintln!("island {}: town {t} in cell {},{}: its market place at {x:.2},{y:.2} stands on {} falling 1 in {:.0}", isl.seed, town.node % l.w, town.node / l.w, here.words(), 1.0 / here.slope.max(1e-4));
        }
        eprintln!("island {}: of {land} land cells' middles {moved} were moved and {poor} still stand on poorer ground than dry and firm, their cells having no better", isl.seed);
        assert!(moved > 500 && moved < land * 6 / 10 && poor * 50 < land, "island {}: {moved} of {land} moved, {poor} on poor ground", isl.seed);
    }
}

#[test]
fn on_the_old_island_the_middles_are_where_they_were() {
    // a run with no land: the jittered middle of every cell, to the last bit
    let r = crate::replay::tiny(1750);
    assert!(r.land.is_none());
    let cen = crate::network::centres(&r);
    for c in 0..r.n {
        let (x, y) = ((c % r.w) as f32, (c / r.w) as f32);
        let was = (x + 0.5 + 0.36 * (world::hash2(c as i32, 11, 1) - 0.5), y + 0.5 + 0.36 * (world::hash2(c as i32, 13, 2) - 0.5));
        assert_eq!(cen[c], was);
        assert_eq!(middles::jittered(c, r.w), was, "the new land starts from the same spot");
    }
}

#[test]
fn a_lake_is_water_to_whoever_asks() {
    for isl in islands() {
        let (l, g) = (isl.land(), &isl.g);
        let laid = g.laid.as_ref().unwrap();
        let mut nodes = 0;
        for n in (0..l.fw * l.fh).filter(|&n| l.lake[n] > 0) {
            let p = l.node_xy(n);
            // the ground itself is the lake's bed, under its sheet; to whoever asks it is water
            assert!(isl.bare(p.x, p.y) < laid.surface[l.lake[n] as usize - 1], "island {}: the bed at {p} stands above its lake's sheet", isl.seed);
            assert!(laid.in_lake(isl.bare(p.x, p.y), p.x, p.y));
            assert!(world::height_at(g, p.x, p.y) < -0.004, "island {}: the lake at {p} is {} high to whoever asks", isl.seed, world::height_at(g, p.x, p.y));
            assert!(world::water_dist(g, p.x, p.y) < 0.0, "island {}: the lake at {p} is {} from the water", isl.seed, world::water_dist(g, p.x, p.y));
            nodes += 1;
        }
        assert!(nodes > 40, "island {} has {nodes} nodes of lake", isl.seed);
        // no parcel has dry land where the spots of it that were looked at are all under a lake
        let mut drowned = 0;
        for i in 0..g.land.at.len() {
            let p = g.land.at[i];
            if p.x < 0.2 || p.y < 0.2 || p.x > l.w as f32 - 0.2 || p.y > l.h as f32 - 0.2 {
                continue;
            }
            let spots = [(0.0, 0.0), (-0.08, -0.08), (0.08, -0.08), (0.08, 0.08), (-0.08, 0.08)];
            if spots.iter().all(|&(dx, dy)| laid.in_lake(isl.bare(p.x + dx, p.y + dy), p.x + dx, p.y + dy)) {
                drowned += 1;
                assert_eq!(g.land.area[i], 0.0, "island {}: the parcel at {p} lies in a lake and has {} of dry land", isl.seed, g.land.area[i]);
            }
        }
        assert!(drowned > 10, "island {}: {drowned} parcels lie in its lakes", isl.seed);
        // well away from every lake the distance to one says so, and the ground is told as it is
        let far = (0..g.gw * g.gh).filter(|&k| laid.lake_dist[k] > 2.0).count();
        assert!(far * 2 > g.gw * g.gh);
        eprintln!("island {}: {nodes} nodes of lake and {drowned} parcels wholly in a lake, all water to whoever asks", isl.seed);
    }
}

#[test]
fn no_scenery_slot_lies_in_a_lake_or_on_the_tide() {
    for isl in islands() {
        let g = &isl.g;
        let laid = g.laid.as_ref().unwrap();
        let slots = crate::scatter::make_slots(&isl.r, g);
        let mut by_a_lake = 0;
        let mut on_shore = 0;
        for (what, list) in [("tree", &slots.trees), ("house", &slots.houses), ("farm", &slots.farms), ("pit", &slots.mines), ("hedgerow tree", &slots.hedge)] {
            assert!(!list.is_empty(), "island {} has no {what} slots", isl.seed);
            for s in list.iter() {
                let h = isl.bare(s.x, s.z);
                assert!(!laid.in_lake(h, s.x, s.z), "island {}: a {what} slot at {:.2},{:.2} lies in a lake", isl.seed, s.x, s.z);
                assert!(h >= 0.05, "island {}: a {what} slot at {:.2},{:.2} stands at {h}, under the line nothing is built or grown below", isl.seed, s.x, s.z);
                by_a_lake += (laid.lake_dist_at(s.x, s.z) < 0.25) as usize;
                let c = (s.z as usize).min(isl.r.h - 1) * isl.r.w + (s.x as usize).min(isl.r.w - 1);
                on_shore += (isl.r.terrain[c] == 0) as usize;
            }
        }
        // the lakes' shores are not bare for it: things stand within 500 m of the water, and on the dry
        // ground of cells the record calls water
        assert!(by_a_lake > 200, "island {}: only {by_a_lake} slots within 500 m of a lake", isl.seed);
        assert!(on_shore > 200, "island {}: only {on_shore} slots on the dry shore of cells of water", isl.seed);
        eprintln!("island {}: {} tree slots, {} house, {} farm, {} hedge; {by_a_lake} within 500 m of a lake, {on_shore} on the dry shore of cells the record calls water, none in a lake", isl.seed, slots.trees.len(), slots.houses.len(), slots.farms.len(), slots.hedge.len());
    }
}

#[test]
fn the_dry_shore_of_a_cell_of_water_takes_the_land_s_classes() {
    for isl in islands() {
        let (r, g) = (&isl.r, &isl.g);
        let land = world::year_land(r, g, 0);
        // every parcel with dry ground belongs to a land cell, unless no land cell is within two cells of it
        let (mut shore, mut bare, mut bare_area, mut dry_area) = (0usize, 0usize, 0.0f32, 0.0f32);
        for i in 0..g.land.at.len() {
            if g.land.area[i] <= 0.0 {
                continue;
            }
            dry_area += g.land.area[i];
            let p = g.land.at[i];
            let own = (p.y.floor().clamp(0.0, r.h as f32 - 1.0) as usize) * r.w + p.x.floor().clamp(0.0, r.w as f32 - 1.0) as usize;
            shore += (r.terrain[own] == 0 && r.terrain[g.land.wcell[i] as usize] != 0) as usize;
            if land.class[i] == SEA {
                bare += 1;
                bare_area += g.land.area[i];
                assert_eq!(r.terrain[g.land.wcell[i] as usize], 0);
            } else {
                assert!((FARM..=HEATH).contains(&land.class[i]), "island {}: parcel {i} is of class {}", isl.seed, land.class[i]);
            }
        }
        eprintln!("island {}: {shore} parcels of dry shore in cells of water take a land cell's classes; {bare} dry parcels ({:.2}% of the dry land) have no land cell within two cells and are left as the sea's", isl.seed, 100.0 * bare_area / dry_area);
        assert!(shore > 300, "island {}: {shore} parcels of dry shore", isl.seed);
        assert!(bare_area < 0.002 * dry_area, "island {}: {:.2}% of the dry land is painted as the sea's", isl.seed, 100.0 * bare_area / dry_area);
    }
}

#[test]
fn the_drawn_land_keeps_the_record_s_shares_on_the_new_land() {
    for isl in islands() {
        let (r, g) = (&isl.r, &isl.g);
        let land = world::year_land(r, g, 0);
        let (mut rec, mut drawn) = ([0.0f64; 4], [0.0f64; 4]);
        for c in (0..r.n).filter(|&c| r.terrain[c] > 0) {
            for (t, s) in world::open_shares(r, 0, c).iter().enumerate() {
                rec[t] += *s as f64;
            }
        }
        for i in 0..g.land.at.len() {
            if (FARM..=HEATH).contains(&land.class[i]) {
                drawn[(land.class[i] - FARM) as usize] += g.land.area[i] as f64;
            }
        }
        let share = |v: [f64; 4]| v.map(|x| 100.0 * x / v.iter().sum::<f64>());
        let (rec, drawn) = (share(rec), share(drawn));
        eprintln!(
            "island {}: open land, % of it (the record / as drawn): arable {:.1} / {:.1}, pasture {:.1} / {:.1}, wood {:.1} / {:.1}, heath {:.1} / {:.1}",
            isl.seed, rec[0], drawn[0], rec[1], drawn[1], rec[2], drawn[2], rec[3], drawn[3]
        );
        for (t, name) in ["arable", "pasture", "wood", "heath"].iter().enumerate() {
            assert!((rec[t] - drawn[t]).abs() < 2.5, "island {}: {name} is {:.1}% of the open land in the record and {:.1}% as drawn", isl.seed, rec[t], drawn[t]);
        }
    }
}

#[test]
fn the_lakes_sheets_lie_over_their_lakes_and_stop_short_of_lower_ground() {
    for isl in islands() {
        let (l, g) = (isl.land(), &isl.g);
        let laid = g.laid.as_ref().unwrap();
        let sw = g.gw - 1;
        let per = VRES / l.fine;
        // every square of a lake's node has the lake's sheet over it
        for n in (0..l.fw * l.fh).filter(|&n| l.lake[n] > 0) {
            for vj in (n / l.fw) * per..(n / l.fw + 1) * per {
                for vi in (n % l.fw) * per..(n % l.fw + 1) * per {
                    assert_eq!(laid.sheet[vj * sw + vi], l.lake[n]);
                }
            }
        }
        // a sheet lies at its lake's level, and beyond its lake no further than the middle of the nodes beside it
        let mut squares = 0;
        for (k, &id) in laid.sheet.iter().enumerate() {
            if id == 0 {
                continue;
            }
            squares += 1;
            let mid = Vec2::new(((k % sw) as f32 + 0.5) / VRES as f32, ((k / sw) as f32 + 0.5) / VRES as f32);
            let n = l.node_at(mid.x, mid.y);
            if l.lake[n] == 0 {
                let level = l.lakes[id as usize - 1].level;
                assert!(l.height[n] >= level - 0.5, "island {}: a sheet lies over the node at {mid}, which stands {} m under its lake's water", isl.seed, level - l.height[n]);
                assert!(l.lake_beside(n).is_some());
            }
            assert!((laid.surface[id as usize - 1] - units(l.lakes[id as usize - 1].level)).abs() < 0.001);
        }
        assert!(squares >= l.lake.iter().filter(|&&k| k > 0).count() * per * per);
    }
}
