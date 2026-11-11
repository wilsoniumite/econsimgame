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
use crate::houses::roof::{self, Roof};
use crate::houses::{self, storeys, Cottage, Terrace, Townhouse, BACK_PLOT_WIDTH, PLOT};
use crate::palette::Palette;
use crate::site::{Model, Site};
use crate::town;
use crate::trees::{bush, tree, Species, Tree};

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
    /// the year it is seen in. The street began before 1750 and has grown
    /// steadily since, so a plot that needs little was built early; a back
    /// lane's row built from 1815 on is a brick terrace
    pub year: i32,
    /// a number of the street's own, which fixes every plot along it
    pub id: i32,
}

/// The back lane runs this far behind the high street.
const BACK_LANE: f32 = 0.13;
/// The year the lab's towns are reckoned from, as the island's are.
const START: i32 = 1750;
/// Rows on the back streets built from this year on are terraces.
const TERRACE_ERA: i32 = 1815;

/// The year a plot was built, and its roof, in a town that has grown
/// steadily from 1750 to `year`: a plot that `need`s little of the density
/// its ground has now was built early, and the first quarter stood before
/// the start. The railway is taken to have come in 1845.
fn roof_of(need: f32, density: f32, year: i32, seed: Seed) -> (i32, Roof) {
    let t = (need / density.max(0.01)).min(1.0);
    let built = if t < 0.25 { roof::built_before(START, seed) } else { START + ((year - START) as f32 * (t - 0.25) / 0.75) as i32 };
    (built, Roof::of(built, year, false, Roof::laid(built, year, seed) >= 1845, seed))
}

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
            let (_, roof) = roof_of(need, d, s.year, seed);
            // a house is rebuilt as a townhouse of the full plot once its ground is dense enough
            if d >= need + 0.12 + 0.25 * seed.v {
                let t = Townhouse { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: storeys::townhouse(d, seed), roof, seed };
                houses::townhouse(&mut m.solid, &mut m.smoke, &plot, &t, pal);
            } else {
                let c = Cottage { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: storeys::cottage(d, seed), roof, seed };
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
                    // the row went up together, and its year settles its style
                    let (built, roof) = roof_of(need, mean, s.year, first);
                    if built >= TERRACE_ERA && len >= 2 {
                        let mid = 0.5 * (x_of(i) + x_of(i + len - 1));
                        let t = Terrace { houses: len as usize, length: Terrace::length_of(len as usize, BACK_PLOT_WIDTH), depth: PLOT.1, storeys: storeys::terrace(mean, first), roof, seed: first };
                        houses::terrace(&mut m.solid, &mut m.smoke, &site.moved(site.at(mid, z + side * lane_back), facing(side)), &t, pal);
                    } else {
                        for q in i..i + len {
                            let seed = Seed::new(hash2(id, q, 24), hash2(id, q, 25));
                            let roof = Roof::of(built, s.year, false, Roof::laid(built, s.year, seed) >= 1845, seed);
                            let c = Cottage { plot_width: BACK_PLOT_WIDTH, plot_depth: PLOT.1, storeys: storeys::cottage(density(x_of(q)), seed), roof, seed };
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
    /// the year it is seen in, which with each plot's need settles how old its house is
    pub year: i32,
    pub id: i32,
}

/// A market place: its paving, the cross or the hall in the middle, the
/// church in its yard off the east side (a big town's has a spire), and
/// houses round it facing in.
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
    // the church stands off the east side, its tower and gate to the market place
    let church = town::Church { big: mk.big, spire: mk.big, seed: Seed::of(mk.id as u32) };
    let (church_at, church_yaw) = church.beside_square(Vec2::ZERO, Vec2::X);
    town::church(&mut m.solid, &site.moved(site.at(church_at.x, church_at.y), site.yaw + church_yaw), &church, pal);
    // houses round the market place, facing in
    let count = ((std::f32::consts::TAU * (radius + 0.03)) / PLOT.0).floor() as i32;
    let id = -1000 - mk.id;
    for i in 0..count {
        let a = i as f32 / count as f32 * std::f32::consts::TAU;
        let dir = Vec2::new(a.cos(), a.sin());
        let p = dir * (radius + 0.03);
        if church.crowded_by(church_at, church_yaw, p) {
            continue;
        }
        let seed = Seed::new(hash2(id, i, 26), hash2(id, i, 27));
        let need = 0.55 * seed.u;
        if mk.density < need {
            continue;
        }
        let plot = site.moved(site.at(p.x, p.y), site.yaw + dir.y.atan2(-dir.x) + std::f32::consts::FRAC_PI_2);
        let (_, roof) = roof_of(need, mk.density, mk.year, seed);
        if mk.density >= need + 0.12 + 0.25 * seed.v {
            let t = Townhouse { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: storeys::townhouse(mk.density, seed), roof, seed };
            houses::townhouse(&mut m.solid, &mut m.smoke, &plot, &t, pal);
        } else {
            let c = Cottage { plot_width: PLOT.0, plot_depth: PLOT.1, storeys: storeys::cottage(mk.density, seed), roof, seed };
            houses::cottage(&mut m.solid, &mut m.smoke, &plot, &c, pal);
        }
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
        let build = |middle: f32, ends: f32, year: i32| {
            let mut m = Model::default();
            street(&mut m, &level(), &Street { length: 1.0, middle, ends, year, id: 3 }, &pal);
            assert!(m.is_sound());
            m
        };
        let (thin, thick) = (build(0.3, 0.05, 1800), build(0.95, 0.3, 1800));
        assert!(thick.solid.triangles() > thin.solid.triangles() * 2);
        // a thinly built street has no back lanes: only the road lies flat
        assert_eq!(thin.flat.triangles(), 2);
        assert_eq!(thick.flat.triangles(), 6);
        // the houses stand along the road, within the lanes' reach of it
        let (lo, hi) = thick.solid.bounds().unwrap();
        assert!(lo.x >= -0.53 && hi.x <= 0.56 && lo.z > -0.2 && hi.z < 0.2, "{lo} {hi}");
        // terraces put fewer, longer roofs on the back lanes than separate cottages do
        let later = build(0.95, 0.3, 1880);
        assert_ne!(later.solid.triangles(), thick.solid.triangles());
        // and by then some roofs are blue slate, which none was in 1800
        let slate = |m: &Model| m.solid.col.iter().filter(|c| **c == *pal.slate.house || **c == *pal.slate.town).count();
        assert!(slate(&thick) == 0 && slate(&later) > 0);
    }

    #[test]
    fn a_market_place_has_its_church_and_fills_with_houses() {
        let pal = Palette::standard();
        let build = |big: bool, density: f32| {
            let mut m = Model::default();
            market(&mut m, &level(), &Market { big, density, year: 1850, id: 2 }, &pal);
            assert!(m.is_sound());
            m
        };
        let (empty, full) = (build(false, 0.0), build(false, 1.0));
        // the paving is nine wedges; even with no houses the church and the cross stand
        assert_eq!(empty.flat.triangles(), 9);
        assert!(empty.solid.bounds().unwrap().1.y > 0.14);
        // a big town's church has a spire
        assert!(build(true, 0.0).solid.bounds().unwrap().1.y > 0.25);
        assert!(full.solid.triangles() > empty.solid.triangles() + 8 * 22);
        // a big town's place is wider
        let width = |m: &Model| m.flat.bounds().map(|(lo, hi)| hi.x - lo.x).unwrap();
        assert!(width(&build(true, 1.0)) > width(&full));
    }

    #[test]
    fn woods_are_trees_and_heath_is_mostly_bushes() {
        let pal = Palette::standard();
        let build = |what| {
            let mut m = Model::default();
            cover(&mut m, &level(), what, 0, &pal);
            assert!(m.is_sound() && m.solid.is_empty());
            // as it stands in summer: trunks, conifers and bushes, and the crowns in leaf
            let mut summer = m.green.evergreen;
            summer.merge(&m.green.leaves);
            summer
        };
        let (low, high, heath) = (build(Cover::LowlandWood), build(Cover::UplandWood), build(Cover::Heath));
        let top = |b: &crate::mesh::MeshBuf| b.bounds().unwrap().1.y;
        assert!(top(&low) > 0.09 && top(&high) > 0.09);
        assert!(heath.triangles() < low.triangles() / 3);
        // the hills have more conifers, which are two cones where a broadleaf is a ball of more faces
        assert!(high.triangles() < low.triangles());
    }
}
