use glam::Vec3;

use crate::colour::shade;
use crate::hash::Seed;
use crate::houses::cottage::back_door;
use crate::houses::kit::{Detail, End, Era, Kit, Open, Opening, Over, Range, RoofSpec, Side, FOOT};
use crate::houses::roof::Roof;
use crate::houses::{door_at, hearth_smoke, window_at, Fabric, BACK_ROOM};
use crate::colour::pick;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::houses::kit::STOREY;
use crate::site::Site;

/// A terraced row. It stands with its middle on the site, along the site's x.
#[derive(Clone, Copy, Debug)]
pub struct Terrace {
    /// how many houses are in the row
    pub houses: usize,
    /// from end wall to end wall
    pub length: f32,
    /// the depth of its plots
    pub depth: f32,
    /// two or three
    pub storeys: u8,
    /// what the row was roofed with when it went up, and how weathered that is now
    pub roof: Roof,
    /// when the row went up, in the steps its style takes
    pub era: Era,
    /// it stands in the hills: stone
    pub upland: bool,
    /// how much of it to draw
    pub detail: Detail,
    /// the row's own numbers: its first plot's
    pub seed: Seed,
}

impl Terrace {
    /// The length of a row of `houses` plots, each `unit` wide, as the towns lay them out.
    pub fn length_of(houses: usize, unit: f32) -> f32 {
        (houses as f32 - 1.0) * unit + unit * 0.96
    }

    /// A brick row of the railway years in the lowland, drawn in full.
    pub fn new(houses: usize, length: f32, depth: f32, storeys: u8, roof: Roof, seed: Seed) -> Terrace {
        Terrace { houses, length, depth, storeys, roof, era: Era::Brick, upland: false, detail: Detail::Near, seed }
    }
}

/// A row is raised a storey when its own ground is dense enough.
pub fn storeys(density: f32, seed: Seed) -> u8 {
    if density >= 0.78 + 0.2 * seed.v { 3 } else { 2 }
}

/// A row steps where the ground under the next house lies this much higher or lower than the floor of the last.
const STEP: f32 = 0.0035;
/// How far a back extension reaches behind the row, and half its width (two houses share one).
const EXT: (f32, f32) = (0.011, 0.0105);

/// A terraced row: brick (stone in the hills) under one long roof, two-up
/// two-down, doors in pairs, a chimney stack on every party wall. The row
/// follows the ground: where it falls or rises along the street the houses
/// step, the higher showing its gable over the lower. Behind, each pair of
/// houses shares a back extension, and each has a walled yard with a privy
/// at the foot of it and a gate to the back lane. A long row may have a
/// way through for a cart, and a row of the later years has a bay window
/// to each front room.
///
/// From far off it is its walls and its roof. At the distance the game is
/// played from it has its openings, its stacks, its back extensions and
/// the flags of its yards. Close to, the windows sit in the wall and the
/// yards have their walls, privies and gates.
pub fn terrace(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, t: &Terrace, pal: &Palette) {
    let houses = t.houses.max(1);
    let unit = t.length / houses as f32;
    let (zf, d) = (t.depth * 0.5 - 0.002, t.depth * 0.68);
    let zm = zf - d * 0.5;
    // the floor of each house: the ground under it, unless that is near enough the floor of the last
    let mut floors: Vec<f32> = Vec::with_capacity(houses);
    for i in 0..houses {
        let g = site.at(-t.length * 0.5 + unit * (i as f32 + 0.5), zm).y - site.origin.y;
        let last = floors.last().copied().unwrap_or(g);
        floors.push(if (g - last).abs() > STEP { g } else { last });
    }
    // each run of houses on one floor is built as one
    let mut from = 0;
    while from < houses {
        let mut to = from + 1;
        while to < houses && floors[to] == floors[from] {
            to += 1;
        }
        let mid = -t.length * 0.5 + unit * (from + to) as f32 * 0.5;
        let at = site.at(mid, 0.0);
        let sub = site.moved(Vec3::new(at.x, site.origin.y + floors[from], at.z), site.yaw);
        let step = |i: usize, j: usize| floors[i] - floors[j];
        let run = Run {
            from,
            to,
            // what is beyond each end: nothing (the row's own end), or a run that stands lower, or higher
            left: if from == 0 { Beyond::Open } else if step(from, from - 1) > 0.0 { Beyond::Lower } else { Beyond::Higher },
            right: if to == houses { Beyond::Open } else if step(to - 1, to) > 0.0 { Beyond::Lower } else { Beyond::Higher },
        };
        build_run(solid, smoke, &sub, t, &run, unit, pal);
        from = to;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Beyond {
    Open,
    Lower,
    Higher,
}

/// The houses `from..to` of a row, which stand on one floor.
struct Run {
    from: usize,
    to: usize,
    left: Beyond,
    right: Beyond,
}

fn build_run(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, t: &Terrace, run: &Run, unit: f32, pal: &Palette) {
    let r = |n: i32| t.seed.pick(n);
    let mut k = Kit::new(solid, site, pal, t.detail, 0.0);
    let count = run.to - run.from;
    let len = unit * count as f32;
    let (hl, zf, d) = (len * 0.5, t.depth * 0.5 - 0.002, t.depth * 0.68);
    let zb = zf - d;
    let n = t.storeys.clamp(2, 4) as usize;
    let h = STOREY * n as f32;
    let fabric = Fabric::of(pal, t.era, t.upland, 1.0, 1.0, Seed::new(t.seed.v, t.seed.u));
    let wall = fabric.wall;
    let roof = t.roof.rebuilt();
    let (colour, lie) = (roof.colour(pal, t.seed), roof.cover.lie());
    let body = Range::rect(-hl, hl, zb, zf);
    let y0 = -k.drop(&body);
    let rise = d * lie.rise;
    let glazing = if t.era == Era::Old { Open::Casement } else { Open::Sash };
    let two_up = r(1) < 0.5;
    let arch_at = (t.houses >= 5 && r(2) < 0.4).then_some(t.houses / 2);
    let paints = &pal.walls.door_paints;

    // every house has its door and its windows; alternate houses are handed the other way, so doors stand in pairs
    let mut front = Vec::new();
    let mut back = Vec::new();
    // the rows of the later years have a bay window to each front room
    let mut bays: Vec<f32> = Vec::new();
    if !k.far() {
        for i in 0..count {
            let j = run.from + i;
            let x0 = unit * i as f32;
            let handed = j % 2 == 1;
            let at = |f: f32| x0 + unit * if handed { 1.0 - f } else { f };
            let over = if t.era >= Era::Brick { Over::Fanlight } else { Over::Nothing };
            let paint = pick(paints, crate::hash::hash2(j as i32, (t.seed.u * 9973.0) as i32, 61));
            if arch_at == Some(j) {
                front.push(Opening { u: at(0.62), y: FOOT, w: unit * 0.5, h: 0.0138, kind: Open::Arch });
                front.push(door_at(at(0.17), Open::Door { paint, over }));
            } else if t.era == Era::Late {
                front.push(door_at(at(0.2), Open::Door { paint, over }));
                bays.push(at(0.66));
            } else {
                front.push(door_at(at(0.24), Open::Door { paint, over }));
                front.push(window_at(at(0.68), 0, 0.0084, 0.0066, 0.009, glazing));
            }
            for s in 1..n {
                if two_up {
                    front.push(window_at(at(0.27), s, 0.0074, 0.0054, 0.0088, glazing));
                    front.push(window_at(at(0.73), s, 0.0074, 0.0054, 0.0088, glazing));
                } else {
                    front.push(window_at(at(0.62), s, 0.0074, 0.0064, 0.0088, glazing));
                }
            }
            // the back, seen from its own left: the half away from the extension has the windows
            let bx = len - x0 - unit * if handed { 0.72 } else { 0.28 };
            for s in 1..n {
                back.push(window_at(bx, s, 0.0074, 0.0054, 0.0082, glazing));
            }
            back.push(window_at(bx, 0, 0.0084, 0.0054, 0.0082, glazing));
        }
    }
    // an end wall is drawn at the row's own end and where it stands above the next run
    let end = |b: Beyond| if b == Beyond::Higher { Side::NONE } else { Side::PLAIN };
    k.walls(&body, y0, h, &wall, [Side::with(&front), end(run.right), Side::with(&back), end(run.left)], true, true);
    let roof_end = |b: Beyond| if b == Beyond::Higher { End::Party } else { End::Gable };
    let verge = if run.left == Beyond::Open || run.right == Beyond::Open { 0.0016 } else { 0.0007 };
    k.roof(&body, &RoofSpec { h, rise, eaves: lie.eaves, verge, ends: [roof_end(run.right), roof_end(run.left)], col: colour, gable: wall.col, thick: lie.thick });
    if k.far() {
        return;
    }
    for &u in &bays {
        k.bay(&body, 0, u, unit * 0.36, 0.0018, y0, FOOT + 0.0148, &wall, shade(colour, 0.9));
    }

    // a stack on every party wall and at each end, astride the ridge
    for b in run.from..=run.to {
        // a wall between two runs carries its stack on the higher of them
        if (b == run.from && run.left == Beyond::Higher) || (b == run.to && run.right == Beyond::Higher) {
            continue;
        }
        let inner = b > 0 && b < t.houses;
        let x = (-hl + unit * (b - run.from) as f32).clamp(-hl + 0.0042, hl - 0.0042);
        let top = k.stack(x, zf - d * 0.5, 0.0076, 0.0058, h + rise * 0.5, h + rise + 0.0055, *pal.brick.dark, if inner { 4 } else { 2 });
        if b == t.houses / 2 && t.seed.u < 0.25 {
            hearth_smoke(smoke, top, t.seed, pal);
        }
    }

    // behind: a back extension to each pair of houses, and the yards
    let rear = -t.depth * 0.5 - BACK_ROOM + 0.001;
    let ext_h = STOREY * (n - 1) as f32 + 0.005;
    let ext_rise = EXT.1 * 0.5;
    let mut ext_at: Vec<(f32, f32)> = Vec::new();
    for i in 0..count {
        let j = run.from + i;
        let (xa, xb) = (-hl + unit * i as f32, -hl + unit * (i + 1) as f32);
        let partner_in_run = if j % 2 == 0 { j + 1 < run.to } else { j > run.from };
        let (x0, x1) = match (j % 2 == 0, partner_in_run) {
            (true, true) => (xb - EXT.1, xb + EXT.1),
            (true, false) => (xb - EXT.1, xb),
            (false, true) => continue,
            (false, false) => (xa, xa + EXT.1),
        };
        ext_at.push((x0, x1));
        let rg = Range::rect(x0, x1, zb - EXT.0, zb);
        let ops: Vec<Opening> = (0..n - 1).map(|s| window_at((x1 - x0) * 0.5, s, 0.0086, 0.005, 0.0078, glazing)).collect();
        let doors = [door_at(EXT.0 * 0.5, back_door(pal))];
        // a door into each yard it stands in; its sides run into the angle with the row
        let yard = |has: bool| if has { Side::with(&doors) } else { Side::PLAIN };
        let (right, left) = (yard(j % 2 == 1 || partner_in_run), yard(j % 2 == 0));
        k.walls(&rg, -k.drop(&rg), ext_h, &wall, [Side::NONE, right.dark(true, false), Side::with(&ops), left.dark(false, true)], true, true);
        let across = Range::rect_across(x0, x1, zb - EXT.0, zb);
        k.roof(&across, &RoofSpec { h: ext_h, rise: ext_rise * (x1 - x0) / (2.0 * EXT.1), eaves: 0.0014, verge: 0.0012, ends: [End::Gable, End::Party], col: colour, gable: wall.col, thick: lie.thick });
    }
    k.patch(-hl, hl, rear, zb, 0.0034, *pal.ground.back_yard);
    if !k.near() {
        return;
    }
    let brick = fabric.yard_wall;
    let (wh, wt) = (0.0062, 0.0013);
    k.low_wall((-hl + 0.0007, rear), (hl - 0.0007, rear), wh, wt, brick);
    for b in 0..=count {
        let x = (-hl + unit * b as f32).clamp(-hl + 0.0007, hl - 0.0007);
        // a wall between two yards begins where the extension on it ends
        let behind_ext = ext_at.iter().any(|&(x0, x1)| x > x0 + 1e-4 && x < x1 - 1e-4);
        k.low_wall((x, rear), (x, zb - if behind_ext { EXT.0 } else { 0.0 }), wh, wt, brick);
    }
    for i in 0..count {
        let j = run.from + i;
        let (xa, xb) = (-hl + unit * i as f32, -hl + unit * (i + 1) as f32);
        // the privy in the corner away from the extension, the gate beside it
        let (px0, px1, fall, gate) = if j % 2 == 0 { (xa + 0.0016, xa + 0.007, 1.0, xa + unit * 0.55) } else { (xb - 0.007, xb - 0.0016, -1.0, xb - unit * 0.55) };
        k.shed(px0, px1, rear + 0.0012, rear + 0.007, 0.0078, fall, shade(brick, 1.1), colour, true);
        let g = k.ground(gate, rear);
        let wood = *pal.timber.door;
        k.quad(
            [k.pt(gate - 0.002, g, rear - wt * 0.5 - 0.0003), k.pt(gate + 0.002, g, rear - wt * 0.5 - 0.0003), k.pt(gate + 0.002, g + wh * 0.92, rear - wt * 0.5 - 0.0003), k.pt(gate - 0.002, g + wh * 0.92, rear - wt * 0.5 - 0.0003)],
            wood,
            k.dir(0.0, 0.0, -1.0),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::houses::{BACK_PLOT_WIDTH, PLOT};
    use crate::site::Model;

    fn row(houses: usize, storeys: u8, seed: Seed, detail: Detail) -> Terrace {
        Terrace { detail, ..Terrace::new(houses, Terrace::length_of(houses, BACK_PLOT_WIDTH), PLOT.1, storeys, Roof::of(1840, 1870, false, false, seed), seed) }
    }

    fn build(houses: usize, storeys: u8, seed: Seed, detail: Detail) -> Model {
        let mut m = Model::default();
        terrace(&mut m.solid, &mut m.smoke, &Site::level(Vec3::ZERO, 0.0), &row(houses, storeys, seed, detail), &Palette::standard());
        assert!(m.is_sound());
        m
    }

    #[test]
    fn a_row_is_as_long_as_its_houses_and_keeps_to_its_ground() {
        for houses in 2..=6 {
            for detail in Detail::ALL {
                let m = build(houses, 2, Seed::of(1), detail);
                let (lo, hi) = m.solid.bounds().unwrap();
                let want = Terrace::length_of(houses, BACK_PLOT_WIDTH);
                assert!(hi.x - lo.x >= want - 1e-5 && hi.x - lo.x < want + 0.005, "{houses} houses: {} long, wanted {want}", hi.x - lo.x);
                assert!(lo.z > -PLOT.1 * 0.5 - BACK_ROOM - 0.001 && hi.z < PLOT.1 * 0.5 + 0.004, "{houses} houses {detail:?}: from {} to {}", lo.z, hi.z);
            }
        }
    }

    #[test]
    fn every_house_has_a_door_and_every_party_wall_a_stack() {
        let pal = Palette::standard();
        // a seed whose row has no smoking chimney
        let seed = Seed::new(0.5, 0.5);
        for houses in 2..=6 {
            let m = build(houses, 2, seed, Detail::Play);
            // at this level a door is one panel, in one of the paints, and a stack one box with a sooty top
            let doors = m.solid.col.iter().filter(|c| pal.walls.door_paints.iter().any(|p| **p == **c)).count() / 4;
            assert_eq!(doors, houses);
            let stack_tops = m.solid.col.iter().filter(|c| **c == *pal.smoke.stack_top).count() / 4;
            assert_eq!(stack_tops, houses + 1);
        }
    }

    #[test]
    fn two_storeys_then_three() {
        let s = Seed::new(0.5, 0.5);
        assert_eq!(storeys(0.8, s), 2);
        assert_eq!(storeys(0.9, s), 3);
        for detail in Detail::ALL {
            assert!(build(4, 3, s, detail).solid.bounds().unwrap().1.y > build(4, 2, s, detail).solid.bounds().unwrap().1.y + 0.018);
        }
    }

    #[test]
    fn each_level_draws_more_than_the_last_and_far_is_one_shell_and_one_roof() {
        for n in 0..20 {
            let seed = Seed::of(n);
            let (far, play, near) = (build(5, 2, seed, Detail::Far), build(5, 2, seed, Detail::Play), build(5, 2, seed, Detail::Near));
            assert_eq!(far.solid.triangles(), 4 * 2 + 2 * 2 + 2);
            assert!(far.smoke.is_empty());
            assert!(play.solid.triangles() > 5 * 20 && play.solid.triangles() < 5 * 70, "seed {n}: {} at play", play.solid.triangles());
            assert!(near.solid.triangles() > play.solid.triangles() * 2, "seed {n}");
        }
    }

    #[test]
    fn a_row_steps_with_the_ground_and_stays_whole_on_the_level() {
        let pal = Palette::standard();
        let seed = Seed::of(3);
        let t = row(6, 2, seed, Detail::Far);
        let on = |slope: &dyn Fn(f32, f32) -> f32| {
            let mut m = Model::default();
            terrace(&mut m.solid, &mut m.smoke, &Site::on(Vec3::new(0.0, slope(0.0, 0.0), 0.0), 0.0, slope), &t, &pal);
            assert!(m.is_sound());
            m
        };
        // ground that hardly falls: one run, as on the level
        let gentle = on(&|x, _| 1.0 + x * 0.01);
        assert_eq!(gentle.solid.triangles(), 14);
        // a hill street: the row breaks into runs, each on its own floor, and its ridge follows the hill
        let steep = on(&|x, _| 1.0 + x * 0.12);
        assert!(steep.solid.triangles() >= 3 * 12, "{} triangles", steep.solid.triangles());
        let ridge = |m: &Model, x0: f32, x1: f32| m.solid.pos.iter().filter(|p| p[0] >= x0 && p[0] <= x1).map(|p| p[1]).fold(f32::MIN, f32::max);
        let (low, high) = (ridge(&steep, -0.14, -0.1), ridge(&steep, 0.1, 0.14));
        assert!(high - low > 0.02 && high - low < 0.04, "the ridge rises {} along the row", high - low);
        // no house floats: the walls reach below the ground, and not far below it
        let lowest = steep.solid.pos.iter().map(|p| p[1] - (1.0 + p[0] * 0.12)).fold(f32::MAX, f32::min);
        assert!(lowest < 0.0 && lowest > -0.012, "the walls end {lowest} above the ground");
    }

    #[test]
    fn a_long_row_may_have_a_way_through() {
        let pal = Palette::standard();
        let dark = *pal.smoke.soot;
        let with = (0..60).filter(|&n| build(6, 2, Seed::of(n), Detail::Play).solid.col.contains(&dark)).count();
        assert!((10..40).contains(&with), "{with} of 60 long rows have an arch");
        assert!((0..60).all(|n| !build(4, 2, Seed::of(n), Detail::Play).solid.col.contains(&dark)));
    }
}
