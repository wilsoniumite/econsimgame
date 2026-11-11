//! Small scenes for the lab: several generator calls placed together the way
//! the island places them, because how things sit side by side matters more
//! than how one stands alone.
//!
//! These follow the viewer's own rules in a small way (the plots along a
//! street, the slots of a wood, the houses round a market place) with the
//! same spacings, set-backs, facings and thresholds, so a scene shows what
//! the game would. They are not what the game runs: the island's layout is
//! in the viewer's `scatter.rs` and `scatter/towns.rs`, and a change there
//! is not seen here until these are changed to match.

use glam::{Vec2, Vec3};

use crate::hash::{hash2, Seed};
use crate::houses::{self, storeys, Cottage, Terrace, Townhouse, BACK_PLOT_WIDTH, PLOT};
use crate::palette::Palette;
use crate::site::{Model, Site};
use crate::town;
use crate::trees::{bush, tree, Species, Tree};
use crate::works::{works, yard_shed, Kind, Works, YardShed};

/// A spot scattered as the island's slots are: its place, its two fixed
/// numbers and a turn.
#[derive(Clone, Copy, Debug)]
pub struct Spot {
    pub x: f32,
    pub z: f32,
    pub seed: Seed,
    pub yaw: f32,
}

/// A block of `cols` by `rows` spots round the origin, spaced `n` to a map
/// cell each way, each moved off its grid point by up to `jitter` of the
/// spacing: the way the viewer lays out the places where a tree or a shed
/// may stand (trees six to a cell each way, houses and sheds seven).
pub fn spots(cols: i32, rows: i32, n: usize, seed: i32, jitter: f32) -> Vec<Spot> {
    let (half_x, half_z) = (cols as f32 * 0.5 / n as f32, rows as f32 * 0.5 / n as f32);
    let mut out = Vec::new();
    for j in 0..rows {
        for i in 0..cols {
            let id = j * 1009 + i;
            out.push(Spot {
                x: -half_x + (i as f32 + 0.5 + jitter * (hash2(id, 1, seed) - 0.5)) / n as f32,
                z: -half_z + (j as f32 + 0.5 + jitter * (hash2(id, 2, seed) - 0.5)) / n as f32,
                seed: Seed::new(hash2(id, 3, seed), hash2(id, 4, seed)),
                yaw: hash2(id, 5, seed) * std::f32::consts::TAU,
            });
        }
    }
    out
}

// ------------------------------------------------------------------ a street

/// A high street with a back lane behind each frontage.
#[derive(Clone, Copy, Debug)]
pub struct Street {
    pub length: f32,
    /// how built up the ground is in the middle of the street and at its ends, 0 to 1
    pub middle: f32,
    pub ends: f32,
    /// the back lanes were built from 1815 on, so their rows are brick terraces
    pub terraces: bool,
    /// a number of the street's own, which fixes every plot along it
    pub id: i32,
}

/// The back lane runs this far behind the high street.
const BACK_LANE: f32 = 0.13;

/// A high street along the site's x: a main road, a frontage of cottages and
/// townhouses on both sides that fills from the middle, and a back lane
/// behind each with houses a row at a time.
pub fn street(m: &mut Model, site: &Site, s: &Street, pal: &Palette) {
    let half = s.length * 0.5;
    let density = |x: f32| s.ends + (s.middle - s.ends) * (1.0 - (x.abs() / half).min(1.0));
    let lift = |p: Vec3, by: f32| p + Vec3::Y * by;
    // the road itself, in gravel
    m.flat.ribbon(&[lift(site.at(-half, 0.0), 0.01), lift(site.at(half, 0.0), 0.01)], 0.034, *pal.ground.road_gravel);
    // the towns hand a plot its facing so: along the street, turned about on the far side
    let facing = |side: f32| site.yaw + if side < 0.0 { std::f32::consts::PI } else { 0.0 };

    // the frontage: a plot every 0.05 on both sides, each built once its ground is dense enough
    let set_back = 0.034 * 0.5 + PLOT.1 * 0.5 + 0.005;
    let count = (s.length / PLOT.0).floor() as i32;
    for (j, side) in [(0, -1.0f32), (1, 1.0)] {
        let id = s.id * 4 + j;
        for i in 0..count {
            let x = -half + (i as f32 + 0.5) * PLOT.0;
            let seed = Seed::new(hash2(id, i, 24), hash2(id, i, 25));
            let (need, d) = (seed.u * 0.75, density(x));
            if d < need {
                continue;
            }
            let plot = site.moved(site.at(x, side * set_back), facing(side));
            // a house is rebuilt as a townhouse of the full plot once its ground is dense enough
            if d >= need + 0.12 + 0.25 * seed.v {
                let t = Townhouse { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: storeys::townhouse(d, seed), seed };
                houses::townhouse(&mut m.solid, &mut m.smoke, &plot, &t, pal);
            } else {
                let c = Cottage { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: storeys::cottage(d, seed), seed };
                houses::cottage(&mut m.solid, &mut m.smoke, &plot, &c, pal);
            }
        }
    }

    // the back lanes, laid where the ground is built up enough, with rows of three to six plots
    for (j, lane_side) in [(0, -1.0f32), (1, 1.0)] {
        let lane_id = s.id * 64 + 2 + j;
        let lane_need = 0.3 + 0.08 * hash2(lane_id, 2, 22);
        if s.middle <= lane_need {
            continue;
        }
        // the stretch of the lane whose ground is dense enough
        let reach = if s.middle > s.ends { half * ((s.middle - lane_need) / (s.middle - s.ends)).min(1.0) } else { half };
        let reach = reach.min(half - 0.1);
        if reach < 0.03 {
            continue;
        }
        let z = lane_side * BACK_LANE;
        m.flat.ribbon(&[lift(site.at(-reach, z), 0.006), lift(site.at(reach, z), 0.006)], 0.014, *pal.ground.street_earth);
        let lane_back = 0.014 * 0.5 + PLOT.1 * 0.5 + 0.005;
        let count = (2.0 * half / BACK_PLOT_WIDTH).floor() as i32;
        let x_of = |i: i32| -half + (i as f32 + 0.5) * BACK_PLOT_WIDTH;
        for (k, side) in [(0, -1.0f32), (1, 1.0)] {
            let id = lane_id * 4 + k;
            let mut i = 0;
            let mut row = 0;
            while i < count {
                let len = (3 + (hash2(id, row, 28) * 4.0) as i32).min(count - i);
                let need = lane_need + (1.0 - lane_need) * hash2(id, row, 29) * 0.95;
                // a row comes in only where its whole length is laid and built up
                let stands = (i..i + len).all(|q| x_of(q).abs() <= reach && density(x_of(q)) >= need);
                if stands {
                    let first = Seed::new(hash2(id, i, 24), hash2(id, i, 25));
                    let mean = (i..i + len).map(|q| density(x_of(q))).sum::<f32>() / len as f32;
                    if s.terraces && len >= 2 {
                        let mid = 0.5 * (x_of(i) + x_of(i + len - 1));
                        let t = Terrace { houses: len as usize, length: Terrace::length_of(len as usize, BACK_PLOT_WIDTH), depth: PLOT.1, storeys: storeys::terrace(mean, first), seed: first };
                        houses::terrace(&mut m.solid, &mut m.smoke, &site.moved(site.at(mid, z + side * lane_back), facing(side)), &t, pal);
                    } else {
                        for q in i..i + len {
                            let seed = Seed::new(hash2(id, q, 24), hash2(id, q, 25));
                            let c = Cottage { plot_width: BACK_PLOT_WIDTH, plot_depth: PLOT.1, storeys: storeys::cottage(density(x_of(q)), seed), seed };
                            houses::cottage(&mut m.solid, &mut m.smoke, &site.moved(site.at(x_of(q), z + side * lane_back), facing(side)), &c, pal);
                        }
                    }
                }
                i += len;
                row += 1;
            }
        }
    }
}

// ------------------------------------------------------------------ a market place

/// A town's market place.
#[derive(Clone, Copy, Debug)]
pub struct Market {
    /// a big town: a wider place, a market hall for the cross, a larger church
    pub big: bool,
    /// how built up the ground is, 0 to 1
    pub density: f32,
    pub id: i32,
}

/// A market place: its paving, the cross or the hall in the middle, the
/// church on its east side, and houses round it facing in.
pub fn market(m: &mut Model, site: &Site, mk: &Market, pal: &Palette) {
    let radius = if mk.big { 0.09 } else { 0.06 };
    let mid = site.origin + Vec3::Y * 0.009;
    // the paving: an uneven ring
    let n = 9;
    let ring: Vec<Vec3> = (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * std::f32::consts::TAU + hash2(mk.id, i, 6) * 0.3;
            let rr = radius * (0.85 + 0.3 * hash2(mk.id, i, 7));
            site.at(a.cos() * rr, a.sin() * rr) + Vec3::Y * 0.009
        })
        .collect();
    for i in 0..n as usize {
        m.flat.tri(mid, ring[(i + 1) % n as usize], ring[i], *pal.ground.market_paving);
    }
    if mk.big {
        town::market_hall(&mut m.solid, site, pal);
    } else {
        town::market_cross(&mut m.solid, site, pal);
    }
    let church_at = Vec2::new(radius + 0.05, 0.0);
    town::church(&mut m.solid, &site.moved(site.at(church_at.x, church_at.y), site.yaw + std::f32::consts::FRAC_PI_2), mk.big, pal);
    // houses round the market place, facing in
    let count = ((std::f32::consts::TAU * (radius + 0.03)) / PLOT.0).floor() as i32;
    let id = -1000 - mk.id;
    for i in 0..count {
        let a = i as f32 / count as f32 * std::f32::consts::TAU;
        let dir = Vec2::new(a.cos(), a.sin());
        let p = dir * (radius + 0.03);
        if p.distance(church_at) < 0.07 {
            continue;
        }
        let seed = Seed::new(hash2(id, i, 26), hash2(id, i, 27));
        let need = 0.55 * seed.u;
        if mk.density < need {
            continue;
        }
        let plot = site.moved(site.at(p.x, p.y), site.yaw + dir.y.atan2(-dir.x) + std::f32::consts::FRAC_PI_2);
        if mk.density >= need + 0.12 + 0.25 * seed.v {
            let t = Townhouse { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: storeys::townhouse(mk.density, seed), seed };
            houses::townhouse(&mut m.solid, &mut m.smoke, &plot, &t, pal);
        } else {
            let c = Cottage { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: storeys::cottage(mk.density, seed), seed };
            houses::cottage(&mut m.solid, &mut m.smoke, &plot, &c, pal);
        }
    }
}

// ------------------------------------------------------------------ a works in its yard

/// A works with the sheds of its yard about it.
#[derive(Clone, Copy, Debug)]
pub struct WorksYard {
    pub kind: Kind,
    pub tier: u8,
    pub id: i32,
}

/// A works on its plot, and round it the sheds and warehouses that stand on
/// works ground: half the house slots of the yard (seven by seven to a
/// cell; the yard here is five slots by three, 0.71 by 0.43 of a cell),
/// square to the map, keeping clear of the works' plot.
pub fn works_yard(m: &mut Model, site: &Site, y: &WorksYard, pal: &Palette) {
    works(&mut m.solid, &mut m.smoke, site, &Works { kind: y.kind, tier: y.tier, seed: y.id as f32 * 0.37 }, pal);
    for s in spots(5, 3, 7, 12 + y.id, 0.7) {
        if s.seed.u >= 0.5 || (s.x.abs() < 0.085 && s.z.abs() < 0.085) {
            continue;
        }
        let yaw = (s.yaw / std::f32::consts::FRAC_PI_2).round() * std::f32::consts::FRAC_PI_2;
        yard_shed(&mut m.solid, &mut m.smoke, &site.moved(site.at(s.x, s.z), site.yaw + yaw), &YardShed { seed: s.seed, smoke_seed: s.yaw }, pal);
    }
}

// ------------------------------------------------------------------ woods and heath

/// What grows on a patch of ground.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cover {
    /// a wood in the lowland: broadleaves, and one tree in eight a conifer
    LowlandWood,
    /// a wood in the hills: two trees in three conifers
    UplandWood,
    /// heath: gorse and broom, and the odd tree
    Heath,
}

/// A patch of wood or heath, five tree slots by three (the slots are six by
/// six to a cell, so the patch is 0.83 by 0.5 of one), with a tree at every
/// slot as the island's woods have.
pub fn cover(m: &mut Model, site: &Site, what: Cover, id: i32, pal: &Palette) {
    for s in spots(5, 3, 6, 11 + id, 0.9) {
        let Seed { u, v } = s.seed;
        let here = site.moved(site.at(s.x, s.z), 0.0);
        let (grow, size) = match what {
            Cover::LowlandWood | Cover::UplandWood => (true, 0.85 + 0.4 * v),
            Cover::Heath => {
                if u < 0.16 {
                    bush(&mut m.green, &here, v, s.yaw, pal);
                }
                (u < 0.035, 0.6 + 0.3 * v)
            }
        };
        if !grow {
            continue;
        }
        let conifer = what == Cover::UplandWood && v > 0.35 || v > 0.88;
        let species = if conifer { Species::Conifer } else { Species::Broadleaf };
        tree(&mut m.green, &here, &Tree { species, size, tint: u, turn: s.yaw }, pal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level() -> Site<'static> {
        Site::level(Vec3::ZERO, 0.0)
    }

    #[test]
    fn spots_fill_their_rectangle_and_are_the_same_each_time() {
        let a = spots(5, 3, 6, 11, 0.9);
        assert_eq!(a.len(), 15);
        // each stays within its own square of the grid
        assert!(a.iter().all(|s| s.x.abs() <= 2.5 / 6.0 && s.z.abs() <= 1.5 / 6.0));
        let b = spots(5, 3, 6, 11, 0.9);
        assert!(a.iter().zip(&b).all(|(p, q)| p.x == q.x && p.z == q.z && p.seed == q.seed));
    }

    #[test]
    fn a_street_fills_from_the_middle() {
        let pal = Palette::standard();
        let build = |middle: f32, ends: f32, terraces: bool| {
            let mut m = Model::default();
            street(&mut m, &level(), &Street { length: 1.0, middle, ends, terraces, id: 3 }, &pal);
            assert!(m.is_sound());
            m
        };
        let (thin, thick) = (build(0.3, 0.05, false), build(0.95, 0.3, false));
        assert!(thick.solid.triangles() > thin.solid.triangles() * 2);
        // a thinly built street has no back lanes: only the road lies flat
        assert_eq!(thin.flat.triangles(), 2);
        assert_eq!(thick.flat.triangles(), 6);
        // the houses stand along the road, within the lanes' reach of it
        let (lo, hi) = thick.solid.bounds().unwrap();
        assert!(lo.x >= -0.53 && hi.x <= 0.56 && lo.z > -0.2 && hi.z < 0.2, "{lo} {hi}");
        // terraces put fewer, longer roofs on the back lanes than separate cottages do
        assert_ne!(build(0.95, 0.3, true).solid.triangles(), thick.solid.triangles());
    }

    #[test]
    fn a_market_place_has_its_church_and_fills_with_houses() {
        let pal = Palette::standard();
        let build = |big: bool, density: f32| {
            let mut m = Model::default();
            market(&mut m, &level(), &Market { big, density, id: 2 }, &pal);
            assert!(m.is_sound());
            m
        };
        let (empty, full) = (build(false, 0.0), build(false, 1.0));
        // the paving is nine wedges; even with no houses the church and the cross stand
        assert_eq!(empty.flat.triangles(), 9);
        assert!(empty.solid.bounds().unwrap().1.y > 0.14);
        assert!(full.solid.triangles() > empty.solid.triangles() + 8 * 22);
        // a big town's place is wider
        let width = |m: &Model| m.flat.bounds().map(|(lo, hi)| hi.x - lo.x).unwrap();
        assert!(width(&build(true, 1.0)) > width(&full));
    }

    #[test]
    fn a_works_stands_in_its_yard_with_sheds_clear_of_it() {
        let pal = Palette::standard();
        let mut alone = Model::default();
        works(&mut alone.solid, &mut alone.smoke, &level(), &Works { kind: Kind::TextileMill, tier: 3, seed: 0.37 }, &pal);
        let mut m = Model::default();
        works_yard(&mut m, &level(), &WorksYard { kind: Kind::TextileMill, tier: 3, id: 1 }, &pal);
        assert!(m.is_sound());
        assert!(m.solid.triangles() >= alone.solid.triangles() + 3 * 14, "fewer than three sheds");
    }

    #[test]
    fn woods_are_trees_and_heath_is_mostly_bushes() {
        let pal = Palette::standard();
        let build = |what| {
            let mut m = Model::default();
            cover(&mut m, &level(), what, 0, &pal);
            assert!(m.is_sound() && m.solid.is_empty());
            m.green
        };
        let (low, high, heath) = (build(Cover::LowlandWood), build(Cover::UplandWood), build(Cover::Heath));
        let top = |b: &crate::mesh::MeshBuf| b.bounds().unwrap().1.y;
        assert!(top(&low) > 0.09 && top(&high) > 0.09);
        assert!(heath.triangles() < low.triangles() / 3);
        // the hills have more conifers, which are two cones where a broadleaf is a ball of more faces
        assert!(high.triangles() < low.triangles());
    }
}
