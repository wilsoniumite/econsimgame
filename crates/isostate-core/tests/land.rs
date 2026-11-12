//! Land (not in the Python; docs/rules/land.md): an island made by uplift and
//! erosion, with its water. The island is the same for the same seed and
//! different for another; water runs downhill all the way to the sea; the
//! economy's `World` is whole; a run on it conserves money and is deterministic;
//! and with the switch off nothing is made (the golden test covers the rest).
#![allow(non_snake_case)]

use isostate_core::landcells::{Landform, generate_land};
use isostate_core::landform::LandParams;
use isostate_core::landwater::RiverEnd;
use isostate_core::world::{MOUNTAINS, SEA, World};
use isostate_core::*;

fn island(seed: u64) -> World {
    generate_land(seed, 120, 84, 6, &LandParams::game())
}

fn form(w: &World) -> &Landform {
    w.landform.as_ref().expect("the land mechanism's island keeps its landform")
}

#[test]
fn off_by_default_and_the_port_s_island_has_no_landform() {
    let P = default_params();
    assert!(!P.land.enabled);
    assert!(!serde_json::to_string(&P).unwrap().contains("\"land\""), "a port run's params.json does not mention the land");
    let (st, _) = init_state(&P, &default_content());
    assert!(st.world.landform.is_none());
    // and under the port's rules with the switch on, the island is the mechanism's
    let mut P = default_params();
    P.land = LandParams::game();
    let (st, _) = init_state(&P, &default_content());
    assert!(st.world.landform.is_some());
}

#[test]
fn the_same_seed_makes_the_same_island_and_another_seed_another() {
    let (a, b, c) = (island(3), island(3), island(4));
    let (fa, fb, fc) = (form(&a), form(&b), form(&c));
    assert!(fa.height.iter().zip(&fb.height).all(|(x, y)| x.to_bits() == y.to_bits()), "the fine heights differ between two makings");
    assert_eq!(fa.rivers, fb.rivers);
    assert_eq!(fa.lakes, fb.lakes);
    assert_eq!(a.town_sites, b.town_sites);
    assert!(a.fertility.iter().zip(&b.fertility).all(|(x, y)| x.to_bits() == y.to_bits()));
    assert_eq!(a.down, b.down);
    assert_ne!(a.land, c.land, "two seeds gave the same coast");
    assert_ne!(fa.rivers.len(), 0);
    assert!(fa.height != fc.height);
}

#[test]
fn the_world_is_whole() {
    for seed in [1, 2, 5] {
        let w = island(seed);
        let lf = form(&w);
        let N = w.W * w.H;
        let land = w.land.iter().filter(|&&l| l).count() as f64;
        assert!((land / N as f64 - 0.58).abs() < 0.015, "seed {seed}: {land} land cells of {N}");
        for c in 0..N {
            if w.land[c] {
                assert!(w.terrain[c] != SEA && !lf.lake[c], "seed {seed}: land cell {c} is water");
                assert!((0.0..=1.0).contains(&w.h[c]));
            } else {
                assert_eq!(w.terrain[c], SEA);
                assert!(!w.river[c] && !w.navigable[c] && !w.coast[c] && w.down[c] == -1);
                for f in [&w.fertility, &w.pasture, &w.forest_pot, &w.rock, &w.clay, &w.ore, &w.coal] {
                    assert_eq!(f[c], 0.0, "seed {seed}: a resource under water at {c}");
                }
            }
            for f in [&w.fertility, &w.pasture, &w.forest_pot, &w.rock, &w.clay, &w.ore, &w.coal, &w.forest0, &w.jitter] {
                assert!((0.0..=1.0).contains(&f[c]), "seed {seed}: a field out of range at {c}");
            }
        }
        // the first towns: six, apart, on land that is not mountain
        assert_eq!(w.town_sites.len(), 6);
        for (i, &a) in w.town_sites.iter().enumerate() {
            assert!(w.land[a] && w.terrain[a] != MOUNTAINS);
            for &b in &w.town_sites[..i] {
                let d = ((a / w.W) as f64 - (b / w.W) as f64).hypot((a % w.W) as f64 - (b % w.W) as f64);
                assert!(d >= 17.0, "seed {seed}: towns {d} cells apart");
            }
        }
        // there is corn land, coal and ore to run an economy on
        let sum = |f: &[f64]| f.iter().sum::<f64>();
        assert!(sum(&w.fertility) > 2000.0 && sum(&w.coal) > 40.0 && sum(&w.ore) > 40.0, "seed {seed}: a poor island");
        assert!(w.navigable.iter().filter(|&&n| n).count() > 30, "seed {seed}: no navigable water");
        // no wood above the tree line, 760 m
        let tops: Vec<usize> = (0..N).filter(|&c| w.land[c] && w.h[c] * lf.h_ref >= 760.0).collect();
        assert!(!tops.is_empty(), "seed {seed}: nothing 760 m up to look at");
        for &c in &tops {
            assert_eq!(w.forest_pot[c], 0.0, "seed {seed}: wood at {:.0} m in cell {c}", w.h[c] * lf.h_ref);
        }
    }
}

#[test]
fn water_runs_downhill_to_the_sea() {
    for seed in [1, 6] {
        let w = island(seed);
        let lf = form(&w);
        let N = w.W * w.H;
        // every land cell's water reaches the sea, a lake or an outlet without going round in a ring
        for start in 0..N {
            if !w.land[start] {
                continue;
            }
            let (mut c, mut steps) = (start, 0);
            while w.down[c] >= 0 && w.land[w.down[c] as usize] {
                c = w.down[c] as usize;
                steps += 1;
                assert!(steps <= N, "seed {seed}: the water from cell {start} goes round in a ring");
            }
        }
        // a cell's stream is smaller than the one it runs into
        for c in 0..N {
            let d = w.down[c];
            if w.land[c] && d >= 0 && w.land[d as usize] {
                assert!(lf.discharge[d as usize] >= lf.discharge[c], "seed {seed}: the flow falls from cell {c} to {d}");
            }
        }
        // a river's water never rises on its way down, and it ends where it says
        for (k, r) in lf.rivers.iter().enumerate() {
            assert!(r.x.len() >= 2 && r.x.len() == r.level.len() && r.x.len() == r.width.len() && r.x.len() == r.flow.len());
            for i in 1..r.level.len() {
                assert!(r.level[i] <= r.level[i - 1] + 1e-3, "seed {seed}: river {k} rises at point {i}");
            }
            match r.end {
                RiverEnd::Sea => assert!(*r.level.last().unwrap() <= 0.01, "seed {seed}: river {k} meets the sea above it"),
                RiverEnd::Lake(l) => assert!(l < lf.lakes.len()),
                RiverEnd::River(m) => assert!(m < lf.rivers.len() && m != k),
            }
        }
        // lakes lie above the sea, water runs out of each, and every lake node is under a listed lake
        assert!(lf.lakes.iter().all(|l| l.level > 0.0 && l.area_km2 > 0.0 && l.depth > 0.0 && l.outflow > 0.0));
        assert!(lf.lake_id.iter().all(|&l| l as usize <= lf.lakes.len()));
        // rain: the wet side is wetter than the rain shadow
        let rain: Vec<f32> = (0..N).filter(|&c| w.land[c]).map(|c| lf.rain[c]).collect();
        let (lo, hi) = (rain.iter().cloned().fold(f32::INFINITY, f32::min), rain.iter().cloned().fold(0.0, f32::max));
        assert!(hi > 2.0 * lo && lo > 200.0, "seed {seed}: rain from {lo} to {hi} mm");
        // an average year is an average year, and a dry one lowers every river
        let ones = vec![1.0; N];
        let (avg, dry) = (lf.discharge_with(&w, &ones), lf.discharge_with(&w, &vec![0.6; N]));
        for c in 0..N {
            if w.land[c] {
                assert!((avg[c] - lf.discharge[c] as f64).abs() <= 1e-6 * avg[c].max(1.0));
                assert!((dry[c] - 0.6 * avg[c]).abs() <= 1e-6 * avg[c].max(1.0));
            }
        }
        // winter rivers run higher than summer ones
        let (jan, jul) = (lf.discharge_month(&w, 0), lf.discharge_month(&w, 6));
        let big = (0..N).filter(|&c| w.navigable[c]).max_by(|&a, &b| lf.discharge[a].total_cmp(&lf.discharge[b])).unwrap();
        assert!(jan[big] > 1.3 * jul[big], "seed {seed}: January {} against July {}", jan[big], jul[big]);
    }
}

#[test]
fn rivers_reach_the_sea_by_their_estuaries() {
    // the drowned valleys are inlets narrower than a cell: the cells along one
    // are land with a tideway through them, and a boat on the river above goes
    // down it to the coast
    for seed in [1, 6, 10] {
        let w = island(seed);
        let lf = form(&w);
        let (W, H) = (w.W, w.H);
        let N = W * H;
        let near = |c: usize| -> Vec<usize> {
            let mut out = Vec::new();
            for (dy, dx) in world::NEIGH8 {
                let (y, x) = ((c / W) as isize + dy, (c % W) as isize + dx);
                if y >= 0 && y < H as isize && x >= 0 && x < W as isize {
                    out.push(y as usize * W + x as usize);
                }
            }
            out
        };
        // every land cell's water has somewhere to go, and gets to the sea or a lake
        for c in 0..N {
            if w.land[c] {
                assert!(w.down[c] >= 0, "seed {seed}: land cell {c} runs nowhere");
            }
        }
        // a tideway is land, is a river, and its water gets out to the open sea
        let tideways: Vec<usize> = (0..N).filter(|&c| lf.estuary[c]).collect();
        assert!(tideways.len() >= 10, "seed {seed}: {} cells of tideway", tideways.len());
        for &start in &tideways {
            assert!(w.land[start] && w.river[start]);
            let mut c = start;
            while w.land[c] {
                c = w.down[c] as usize;
            }
            assert!(!lf.lake[c], "seed {seed}: the tideway from {start} ends in a lake");
        }
        // most navigable water is joined to the open sea by navigable water
        // (what is not lies above a reach too steep for boats, or on an upland lake)
        let open = |c: usize| !w.land[c] && !lf.lake[c];
        let mut seen = vec![false; N];
        let mut stack: Vec<usize> = (0..N).filter(|&c| w.navigable[c] && near(c).into_iter().any(open)).collect();
        for &c in &stack {
            seen[c] = true;
        }
        while let Some(c) = stack.pop() {
            for nc in near(c) {
                if w.navigable[nc] && !seen[nc] {
                    seen[nc] = true;
                    stack.push(nc);
                }
            }
        }
        let (nav, joined) = (w.navigable.iter().filter(|&&n| n).count(), seen.iter().filter(|&&s| s).count());
        assert!(joined * 5 >= nav * 4, "seed {seed}: {joined} of {nav} navigable cells reach the sea");
        // some river's estuary has a head, where a town would stand, and ships
        // come up a tideway: it is coast, and one of the first towns is a port
        assert!(lf.site_kind.iter().any(|&k| k == landcells::SITE_ESTUARY));
        assert!(tideways.iter().any(|&c| w.coast[c] && !near(c).into_iter().any(open)), "seed {seed}: no tideway is coast");
        assert!(w.town_sites.iter().any(|&c| w.coast[c]), "seed {seed}: no first town is a port");
    }
}

#[test]
fn a_lake_is_not_the_sea_and_a_tideway_s_mouth_is_a_harbour() {
    use isostate_core::transport::{self as tr, BEACH_COST, PORT_COST, SEA_COST};
    let mut w = island(8);
    let (W, H) = (w.W, w.H);
    let N = W * H;
    let no = vec![0u8; N];
    let (ports, cover) = (vec![false; N], vec![0.0; N]);
    let edge = |g: &tr::Graph, a: usize, b: usize| (0..8).find(|&k| tr::neighbour(H, W, a, k) == Some(b)).map(|k| g.w[a * 8 + k]).unwrap();
    // with no town's harbour anywhere, the mouth of a navigable tideway still lets boats out
    let g = tr::build_graph(&w, &no, &no, &ports, &cover);
    let lf = form(&w);
    let mut mouths = 0;
    for a in 0..N {
        for k in 0..8 {
            let Some(b) = tr::neighbour(H, W, a, k) else { continue };
            if w.land[a] && !w.land[b] && !lf.lake[b] {
                let harbour = lf.estuary[a] && w.navigable[a];
                assert_eq!(g.w[a * 8 + k], if harbour { PORT_COST } else { BEACH_COST });
                assert_eq!(g.w[a * 8 + k], edge(&g, b, a));
                mouths += harbour as usize;
            }
        }
    }
    assert!(mouths > 0, "no tideway meets the open sea");
    // put a lake cell beside the sea: the two are not one water
    let lake = (0..N).find(|&c| lf.lake[c]).expect("seed 8 has a lowland lake");
    let sea = lake + 1;
    let lf = w.landform.as_mut().unwrap();
    lf.lake[sea] = false;
    w.land[sea] = false;
    let g = tr::build_graph(&w, &no, &no, &ports, &cover);
    assert_eq!(edge(&g, lake, sea), BEACH_COST);
    assert_eq!(edge(&g, sea, lake), BEACH_COST);
    // while two cells of the lake are crossed at the sea's rate
    let lf = form(&w);
    if let Some(other) = (0..8).filter_map(|k| tr::neighbour(H, W, lake, k)).find(|&c| lf.lake[c]) {
        let step = if (other / W != lake / W) && (other % W != lake % W) { std::f64::consts::SQRT_2 } else { 1.0 };
        assert_eq!(edge(&g, lake, other), SEA_COST * step);
    }
}

#[test]
fn a_run_on_the_new_land_conserves_money_and_is_deterministic() {
    // thirty years under the game's rules, with six families, on an island made by water
    let mut P = game_params();
    P.seed = 3;
    P.n_families = 6;
    P.end_year = Some(1779);
    P.land = LandParams::game();
    let run = |P: &Params| sim::run_with(P, game_content(), false, true).expect("money is conserved every month");
    let (a, b) = (run(&P), run(&P));
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    for (k, name) in st.series.names.iter().enumerate() {
        let (x, y) = (&st.series.data[k], &b.state.series.data[k]);
        assert!(x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()), "series {name} differs between identical runs");
    }
    // the economy stands on it: people, towns and works
    assert!(st.pop.iter().sum::<f64>() > 135_000.0);
    assert!(st.f_alive.iter().filter(|&&a| a).count() > 100);
    // and the run's file carries the land
    let blob = record::pack(st, &a.content, &a.params, &a.events, &a.rec, "land", false);
    let hlen = u32::from_le_bytes(blob[..4].try_into().unwrap()) as usize;
    let header: serde_json::Value = serde_json::from_slice(&blob[4..4 + hlen]).unwrap();
    let names: Vec<&str> = header["arrays"].as_array().unwrap().iter().map(|a| a["name"].as_str().unwrap()).collect();
    for want in ["land_height", "river_xy", "river_level", "land_lake", "land_basin", "land_power", "land_floodplain", "land_wetness", "land_runoff"] {
        assert!(names.contains(&want), "run.bin has no {want}");
    }
    assert!(header["land"]["fine"].as_u64().unwrap() >= 2);
    // a port run's file has none of it
    let mut P = default_params();
    P.seed = 3;
    P.end_year = Some(1751);
    let r = sim::run_with(&P, default_content(), false, true).unwrap();
    let blob = record::pack(&r.state, &r.content, &r.params, &r.events, &r.rec, "port", false);
    let hlen = u32::from_le_bytes(blob[..4].try_into().unwrap()) as usize;
    let header: serde_json::Value = serde_json::from_slice(&blob[4..4 + hlen]).unwrap();
    assert!(header.get("land").is_none());
    assert!(header["arrays"].as_array().unwrap().iter().all(|a| !a["name"].as_str().unwrap().starts_with("land_")));
}
