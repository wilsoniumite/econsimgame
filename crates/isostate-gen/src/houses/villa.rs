use crate::colour::shade;
use crate::hash::Seed;
use crate::houses::cottage::back_door;
use crate::houses::kit::{Detail, End, Era, Kit, Open, Over, Range, RoofSpec, Side, FOOT};
use crate::houses::roof::Roof;
use crate::houses::{door_at, garden_tree, hearth_smoke, window_at, Fabric, BACK_ROOM, SET_BACK};
use crate::colour::pick;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::parts::STOREY;
use crate::site::Site;
use crate::trees::Greenery;

/// A villa: the house of someone who has done well, in its own garden.
#[derive(Clone, Copy, Debug)]
pub struct Villa {
    /// the plot it stands on, which it shares with its garden
    pub plot_width: f32,
    pub plot_depth: f32,
    /// what it is roofed with, and how weathered that is now
    pub roof: Roof,
    /// when it was built, in the steps its style takes
    pub era: Era,
    /// it stands in the hills: stone
    pub upland: bool,
    /// how much of it to draw
    pub detail: Detail,
    pub seed: Seed,
}

impl Villa {
    /// A villa of the railway years in the lowland, drawn in full.
    pub fn new(plot_width: f32, plot_depth: f32, roof: Roof, seed: Seed) -> Villa {
        Villa { plot_width, plot_depth, roof, era: Era::Brick, upland: false, detail: Detail::Near, seed }
    }
}

/// A villa: two storeys, set back in a garden with a wall and a gate before
/// it, a hedge round it and a tree or two. The earlier are a square box in
/// stucco under a low hipped roof with broad eaves, three bays, the door in
/// the middle under a porch on two columns, a stack toward each end. The
/// later are of brick with stone dressings and an L on plan: a wing with
/// its gable and a bay window to the road, the range beside it running into
/// the wing's roof in valleys, and a porch in the angle.
///
/// From far off it is its walls and roofs. At the distance the game is
/// played from it has its openings, stacks, porch, lawn and trees. Close
/// to, its windows sit in the wall and the garden has its wall, hedge,
/// path and beds.
pub fn villa(solid: &mut MeshBuf, smoke: &mut MeshBuf, green: &mut Greenery, site: &Site, t: &Villa, pal: &Palette) {
    let Seed { v, .. } = t.seed;
    let r = |n: i32| t.seed.pick(n);
    let mut k = Kit::new(solid, site, pal, t.detail, 0.0);
    let (pw, pd) = (t.plot_width, t.plot_depth);
    let n = 2usize;
    let h = STOREY * n as f32;
    let mut fabric = Fabric::of(pal, t.era, t.upland, 0.0, 0.5, t.seed);
    if !fabric.brick && !t.upland {
        // stucco, lined out as stone
        fabric.wall.col = if v < 0.5 { *pal.walls.cream } else { *pal.walls.limewash };
        fabric.wall.plinth = Some(*pal.stone.sill);
    }
    let wall = fabric.wall;
    let roof = t.roof.rebuilt();
    let (colour, lie) = (roof.colour(pal, t.seed), roof.cover.lie());
    let glazing = Open::Sash;
    let paint = pick(&pal.walls.door_paints, r(4));
    let front_door = Open::Door { paint, over: Over::Fanlight };
    let boxy = t.era <= Era::Brick && !fabric.brick || r(1) < 0.3;
    let zb = -pd * 0.5 + 0.0045;
    // where the front door is, for the path
    let door_x;
    let tops: [glam::Vec3; 2];

    if boxy {
        let (w, d) = (pw * 0.84, pd * 0.68);
        let (hw, zf) = (w * 0.5, zb + d);
        let body = Range::rect(-hw, hw, zb, zf);
        let y0 = -k.drop(&body);
        let mut front = Vec::new();
        let mut back = Vec::new();
        if !k.far() {
            for b in 0..3 {
                let at = w * (b as f32 + 0.5) / 3.0;
                if b == 1 {
                    front.push(door_at(at, front_door));
                    back.push(door_at(at, back_door(pal)));
                } else {
                    front.push(window_at(at, 0, 0.0068, 0.0058, 0.0108, glazing));
                    back.push(window_at(at, 0, 0.0082, 0.0056, 0.0088, glazing));
                }
                front.push(window_at(at, 1, 0.0066, 0.0058, 0.0098, glazing));
                back.push(window_at(at, 1, 0.0072, 0.0056, 0.0086, glazing));
            }
        }
        let ends = [window_at(d * 0.5, 0, 0.0075, 0.0056, 0.0098, glazing), window_at(d * 0.5, 1, 0.0068, 0.0056, 0.0094, glazing)];
        let end = if k.far() { Side::PLAIN } else { Side::with(&ends) };
        k.walls(&body, y0, h, &wall, [Side::with(&front), end, Side::with(&back), end], true, true);
        let rise = d * 0.36;
        k.roof(&body, &RoofSpec { h, rise, eaves: 0.0038, verge: 0.0, ends: [End::Hip, End::Hip], col: colour, gable: wall.col, thick: lie.thick.max(0.0012) });
        door_x = 0.0;
        let ridge = hw - d * 0.5;
        let st = |k: &mut Kit, side: f32| k.stack(side * ridge.max(0.004), (zb + zf) * 0.5, 0.0062, 0.0056, h + rise * 0.55, h + rise + 0.006, fabric.stack, 3);
        tops = [st(&mut k, 1.0), st(&mut k, -1.0)];
        if !k.far() {
            // the porch: a flat top on two columns
            let (px, pz, ph) = (0.0046, 0.0042, FOOT + 0.0158);
            k.boxy(-px, px, zf, zf + pz, ph, ph + 0.0014, wall.sill, Some(shade(wall.sill, 1.05)));
            if k.near() {
                for side in [-1.0f32, 1.0] {
                    let x = side * (px - 0.0008);
                    k.boxy(x - 0.0006, x + 0.0006, zf + pz - 0.0014, zf + pz - 0.0002, 0.0, ph, wall.sill, None);
                }
                k.strip(&body, 0, 0.0, w, STOREY - 0.0005, STOREY + 0.0005, 0.0004, wall.sill);
            }
        }
    } else {
        // an L: a wing with its gable to the road, and a range beside it
        let left = r(3) < 0.5;
        let (ww, wm) = (pw * 0.42, pw * 0.44);
        let total = ww + wm;
        let (wx0, wx1, mx0, mx1) = if left { (-total * 0.5, -total * 0.5 + ww, -total * 0.5 + ww, total * 0.5) } else { (total * 0.5 - ww, total * 0.5, -total * 0.5, total * 0.5 - ww) };
        let zfw = pd * 0.5 - 0.0052;
        let zfm = zfw - 0.0056;
        let (wing, main) = (Range::rect(wx0, wx1, zb, zfw), Range::rect(mx0, mx1, zb, zfm));
        let y0 = -k.drop(&wing).max(k.drop(&main));
        let (wing_rise, dm) = (ww * 0.5 * 0.96, zfm - zb);
        let main_rise = (dm * 0.5 * 0.9).min(wing_rise);
        let mut wing_front = Vec::new();
        let mut main_front = Vec::new();
        let mut main_back = Vec::new();
        let mut wing_back = Vec::new();
        if !k.far() {
            wing_front.push(window_at(ww * 0.5, 1, 0.0068, 0.0062, 0.0096, glazing));
            wing_back.push(window_at(ww * 0.5, 0, 0.0082, 0.0056, 0.0088, glazing));
            wing_back.push(window_at(ww * 0.5, 1, 0.0072, 0.0056, 0.0086, glazing));
            for b in 0..2 {
                let at = wm * (b as f32 + 0.5) / 2.0;
                main_front.push(window_at(at, 1, 0.0068, 0.0056, 0.0092, glazing));
                main_back.push(window_at(at, 1, 0.0072, 0.0056, 0.0086, glazing));
                main_back.push(if b == 0 { door_at(at, back_door(pal)) } else { window_at(at, 0, 0.0082, 0.0056, 0.0088, glazing) });
            }
            // the ground floor window of the range, away from the porch in the angle
            main_front.push(window_at(if left { wm * 0.72 } else { wm * 0.28 }, 0, 0.0075, 0.006, 0.01, glazing));
        }
        // the wing's inner side shows where it stands forward of the range; the range's front runs into the angle
        let (wing_right, wing_left) = (Side::PLAIN, Side::PLAIN);
        k.walls(&wing, y0, h, &wall, [Side::with(&wing_front), wing_right, Side::with(&wing_back), wing_left], true, true);
        let (main_right, main_left) = if left { (Side::PLAIN, Side::NONE) } else { (Side::NONE, Side::PLAIN) };
        k.walls(&main, y0, h, &wall, [Side::with(&main_front).dark(left, !left), main_right, Side::with(&main_back), main_left], true, true);
        // the wing's roof, its ridge from back to front; the range's runs into it
        let wing_roof = Range { cx: (wx0 + wx1) * 0.5, cz: (zb + zfw) * 0.5, len: zfw - zb, dep: ww, turn: 3 };
        k.roof(&wing_roof, &RoofSpec { h, rise: wing_rise, eaves: 0.003, verge: 0.003, ends: [End::Gable, End::Gable], col: colour, gable: wall.col, thick: 0.0014 });
        let run = ww * 0.5 * main_rise / wing_rise;
        let ends = if left { [End::Gable, End::Valley(run)] } else { [End::Valley(run), End::Gable] };
        k.roof(&main, &RoofSpec { h, rise: main_rise, eaves: 0.003, verge: 0.0024, ends, col: colour, gable: wall.col, thick: lie.thick.max(0.0012) });
        if !k.far() {
            // a window in the wing's gable, a bay below it, and a porch in the angle
            let g = *pal.glass.window;
            let mid = ww * 0.5;
            if k.near() {
                k.strip(&wing, 0, mid - 0.003, mid + 0.003, h + wing_rise * 0.2, h + wing_rise * 0.2 + 0.0062, 0.0004, wall.frame);
                k.strip(&wing, 0, 0.0, ww, STOREY - 0.0005, STOREY + 0.0005, 0.0004, wall.sill);
            }
            k.strip(&wing, 0, mid - 0.0024, mid + 0.0024, h + wing_rise * 0.2 + 0.0006, h + wing_rise * 0.2 + 0.0056, 0.0007, g);
            k.bay(&wing, 0, mid, ww * 0.56, 0.0028, y0, FOOT + 0.0172, &wall, shade(colour, 0.9));
            let (px0, px1) = if left { (mx0, mx0 + 0.0086) } else { (mx1 - 0.0086, mx1) };
            let porch = Range::rect(px0, px1, zfm, zfm + 0.0046);
            let door = [door_at((px1 - px0) * 0.5, front_door)];
            k.lean_to(&porch, FOOT + 0.0162, FOOT + 0.0202, &wall, shade(colour, 0.94), 0.0012, Side::with(&door), [left, !left]);
        }
        door_x = if left { mx0 + 0.0043 } else { mx1 - 0.0043 };
        // a stack on the wing's outer wall, and one at the range's gable
        let outer = if left { wx0 + 0.004 } else { wx1 - 0.004 };
        let gable = if left { mx1 - 0.004 } else { mx0 + 0.004 };
        tops = [
            k.stack(outer, (zb + zfw) * 0.5, 0.0056, 0.0068, h, h + wing_rise * 0.5 + 0.009, fabric.stack, 3),
            k.stack(gable, (zb + zfm) * 0.5, 0.0056, 0.0062, h + main_rise * 0.5, h + main_rise + 0.006, fabric.stack, 2),
        ];
    }
    if k.far() {
        return;
    }
    if r(11) < 0.3 {
        hearth_smoke(smoke, tops[(r(12) * 2.0) as usize % 2], t.seed, pal);
    }

    // the garden: lawn over the whole plot and the ground behind, a tree or two
    let rear = -pd * 0.5 - BACK_ROOM + 0.001;
    let (gx0, gx1, gate) = (-pw * 0.5 + 0.0008, pw * 0.5 - 0.0008, pd * 0.5 + SET_BACK - 0.0012);
    k.patch(gx0, gx1, rear, gate, 0.0034, *pal.ground.garden);
    garden_tree(green, &k, site, gx0 + 0.007, rear + 0.007, 0.42 + 0.12 * r(16), t.seed, pal);
    if r(15) < 0.5 {
        garden_tree(green, &k, site, gx1 - 0.007, rear + 0.009, 0.36 + 0.1 * r(19), Seed::new(t.seed.v, t.seed.u), pal);
    }
    if !k.near() {
        return;
    }
    // a path from the gate to the door, a low wall along the road with the gate in it, a hedge round the rest
    k.patch(door_x - 0.0022, door_x + 0.0022, zb + 0.02, gate + 0.001, 0.0042, *pal.ground.garden_path);
    let front_wall = if t.upland { *pal.stone.garden_wall } else if fabric.brick { *pal.brick.dark } else { *pal.stone.sill };
    k.low_wall((gx0, gate), (door_x - 0.0032, gate), 0.0036, 0.0014, front_wall);
    k.low_wall((door_x + 0.0032, gate), (gx1, gate), 0.0036, 0.0014, front_wall);
    for x in [door_x - 0.0032, door_x + 0.0032] {
        k.boxy_on(x - 0.0011, x + 0.0011, gate - 0.0011, gate + 0.0011, 0.0052, front_wall, Some(shade(front_wall, 1.08)));
    }
    let hedge = if t.upland { *pal.stone.garden_wall } else { *pal.foliage.hedge };
    k.low_wall((gx0 + 0.001, rear), (gx1, rear), 0.0052, 0.0026, hedge);
    k.low_wall((gx0 + 0.001, rear), (gx0 + 0.001, gate - 0.001), 0.0052, 0.0026, hedge);
    // a bed of flowers under the front wall, either side of the path
    for (a, b) in [(gx0 + 0.004, door_x - 0.005), (door_x + 0.005, gx1 - 0.002)] {
        if b - a > 0.004 {
            k.patch(a, b, gate - 0.0042, gate - 0.0016, 0.0039, *pal.ground.garden_bed);
            k.patch(a + 0.0008, b - 0.0008, gate - 0.0036, gate - 0.0022, 0.0052, pick(&pal.roofs.awnings, r(20)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::houses::PLOT;
    use crate::site::Model;
    use glam::Vec3;

    fn built(t: &Villa) -> Model {
        let mut m = Model::default();
        villa(&mut m.solid, &mut m.smoke, &mut m.green, &Site::level(Vec3::ZERO, 0.0), t, &Palette::standard());
        assert!(m.is_sound());
        m
    }

    #[test]
    fn it_keeps_to_its_plot_at_every_level_and_in_every_era() {
        for n in 0..40 {
            let seed = Seed::of(n);
            for era in [Era::Georgian, Era::Brick, Era::Late] {
                let mut last = 0;
                for detail in Detail::ALL {
                    let m = built(&Villa { era, detail, upland: n % 6 == 0, ..Villa::new(PLOT.0, PLOT.1, Roof::of(1850, 1880, false, true, seed), seed) });
                    let (lo, hi) = m.solid.bounds().unwrap();
                    assert!(lo.x > -PLOT.0 * 0.5 - 0.002 && hi.x < PLOT.0 * 0.5 + 0.002, "seed {n} {era:?}: from {} to {}", lo.x, hi.x);
                    assert!(lo.z > -PLOT.1 * 0.5 - BACK_ROOM - 0.002 && hi.z < PLOT.1 * 0.5 + SET_BACK + 0.001, "seed {n} {era:?} {detail:?}: from {} to {}", lo.z, hi.z);
                    assert!(m.solid.triangles() > last, "seed {n} {era:?}: no more at {detail:?}");
                    last = m.solid.triangles();
                    // it has a garden with a tree in it from the distance the game is played at
                    assert_eq!(m.green.is_empty(), detail == Detail::Far);
                }
            }
        }
    }

    #[test]
    fn the_early_are_hipped_boxes_and_the_late_have_a_wing() {
        let far = |era: Era, n: u32| built(&Villa { era, detail: Detail::Far, ..Villa::new(PLOT.0, PLOT.1, Roof::of(1850, 1880, false, true, Seed::of(n)), Seed::of(n)) });
        // a box: four walls, and a roof of two slopes and two hips
        assert!((0..20).all(|n| far(Era::Georgian, n).solid.triangles() == 4 * 2 + 2 * 2 + 2));
        // an L: seven walls and two roofs, one running into the other
        let wings = (0..40).filter(|&n| far(Era::Late, n).solid.triangles() > 20).count();
        assert!(wings > 20, "{wings} of 40 late villas have a wing");
    }

    #[test]
    fn the_same_seed_gives_the_same_villa() {
        let build = |n: u32| built(&Villa::new(PLOT.0, PLOT.1, Roof::of(1850, 1880, false, true, Seed::of(3)), Seed::of(n)));
        assert_eq!(build(3), build(3));
        assert_ne!(build(3), build(5));
    }
}
