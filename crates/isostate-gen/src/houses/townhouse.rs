use crate::colour::{pick, shade};
use crate::hash::Seed;
use crate::houses::cottage::back_door;
use crate::houses::kit::{Detail, End, Era, Kit, Open, Opening, Range, RoofSpec, Side, FOOT};
use crate::houses::roof::Roof;
use crate::houses::{door, door_at, hearth_smoke, timber_frame, window, window_at, Fabric, BACK_ROOM};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::houses::kit::STOREY;
use crate::site::Site;

/// A house in a continuous frontage.
#[derive(Clone, Copy, Debug)]
pub struct Townhouse {
    pub plot_width: f32,
    pub plot_depth: f32,
    /// two to four
    pub storeys: u8,
    /// what its plot's house was roofed with, and how weathered that is now
    /// (a townhouse is a rebuilding, so it is never thatched)
    pub roof: Roof,
    /// when it was built, in the steps its style takes
    pub era: Era,
    /// it stands in the hills: stone
    pub upland: bool,
    /// its ground floor is a shop
    pub shop: bool,
    /// a house stands against it on its left (as seen from the street) and on its right
    pub neighbours: (bool, bool),
    /// how much of it to draw
    pub detail: Detail,
    pub seed: Seed,
}

impl Townhouse {
    /// A townhouse of the middle years in the lowland, no shop, no neighbours, drawn in full.
    pub fn new(plot_width: f32, plot_depth: f32, storeys: u8, roof: Roof, seed: Seed) -> Townhouse {
        Townhouse { plot_width, plot_depth, storeys, roof, era: Era::Georgian, upland: false, shop: false, neighbours: (false, false), detail: Detail::Near, seed }
    }
}

/// A townhouse has two storeys, and a third once its ground is dense enough.
pub fn storeys(density: f32, seed: Seed) -> u8 {
    2 + (density >= 0.62 + 0.35 * seed.u) as u8
}

/// How the roof of a townhouse is shaped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Form {
    /// the ridge along the street, the eaves over the front
    Eaves,
    /// the gable to the street, as the oldest houses stand
    FrontGable,
    /// the front wall carried up to hide a low roof
    Parapet,
    /// two ridges over a deep house, a valley between
    Double,
}

/// A townhouse: the full width of its plot, hard on the street, two to four
/// storeys, its own colour and roof, so a high street reads as a row of
/// separate houses. The oldest are timber-framed and jettied, many with
/// their gables to the street; those of the eighteenth century have sashes
/// in two or three bays, a hood or a fanlight over the door, and some a
/// parapet; the tallest may have two roofs side by side. Its stacks stand on
/// the party walls. Half have a wing behind, and every one a walled yard.
/// Where the town is busy its ground floor is a shop.
///
/// From far off it is its walls and its roof. At the distance the game is
/// played from it has its openings, its stacks, its dormers and its wing.
/// Close to, the windows sit in the wall, the shop has its board and
/// perhaps an awning, and the yard has its wall and its privy.
pub fn townhouse(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, t: &Townhouse, pal: &Palette) {
    let Seed { u, v } = t.seed;
    let r = |n: i32| t.seed.pick(n);
    let mut k = Kit::new(solid, site, pal, t.detail, 0.0);
    let (pw, pd) = (t.plot_width, t.plot_depth);
    let (w, d) = (pw * 0.99, pd * (0.74 + 0.1 * u));
    let (hw, zf) = (w * 0.5, pd * 0.5 - 0.001);
    let zb = zf - d;
    let n = t.storeys.clamp(2, 4) as usize;
    let h = STOREY * n as f32;
    let fabric = Fabric::of(pal, t.era, t.upland, 0.25, 0.6, t.seed);
    let wall = fabric.wall;
    let colour = t.roof.rebuilt().colour(pal, t.seed);
    let lie = t.roof.rebuilt().cover.lie();
    let body = Range::rect(-hw, hw, zb, zf);
    let y0 = -k.drop(&body);
    let form = if t.era == Era::Old && r(3) < 0.55 {
        Form::FrontGable
    } else if t.era >= Era::Georgian && r(4) < if n >= 3 { 0.55 } else { 0.18 } {
        Form::Parapet
    } else if n >= 3 && r(5) < 0.4 {
        Form::Double
    } else {
        Form::Eaves
    };
    let jetty = fabric.timber && !k.far();
    const JETTY: f32 = 0.0026;

    // the wing behind, a storey lower, on one side
    let wing = r(12) < 0.55 && !k.far();
    let (wing_w, wing_d, wing_left) = (w * 0.46, 0.012, r(13) < 0.5);
    let (wx0, wx1) = if wing_left { (-hw, -hw + wing_w) } else { (hw - wing_w, hw) };
    let behind_wing = |x: f32| wing && x > wx0 && x < wx1;

    // openings by rule: two or three bays, a window in each on every floor, the door in an end bay
    let bays = if r(8) < 0.55 { 3 } else { 2 };
    let door_bay = if r(9) < 0.5 { 0 } else { bays - 1 };
    let glazing = window(t.era);
    let ww = if bays == 3 { 0.0056 } else { 0.0066 };
    let mut front = Vec::new();
    let mut back = Vec::new();
    if !k.far() {
        if t.shop {
            let paints = [*pal.walls.shop_front, pick(&pal.roofs.awnings, r(10)), pick(&pal.walls.door_paints, r(10))];
            let paint = paints[(r(20) * 3.0) as usize % 3];
            let awning = (r(11) < 0.4).then(|| pick(&pal.roofs.awnings, r(21)));
            front.push(Opening { u: w * 0.5, y: FOOT, w: w * 0.8, h: 0.0136, kind: Open::Shop { paint, awning } });
        }
        for b in 0..bays {
            let at = w * (b as f32 + 0.5) / bays as f32;
            if !t.shop {
                if b == door_bay {
                    front.push(door_at(at, door(pal, t.era, t.seed)));
                } else {
                    front.push(window_at(at, 0, 0.0082, ww, if t.era == Era::Old { 0.0074 } else { 0.0092 }, glazing));
                }
            }
            let x = hw - at;
            for s in 1..n {
                // the first floor is the tallest, the top one the lowest, as the eighteenth century had it
                let tall = if t.era == Era::Old { 0.0074 } else if s == n - 1 && n >= 3 { 0.0078 } else { 0.0098 };
                front.push(window_at(at, s, 0.0068, ww, tall, glazing));
                if !(behind_wing(x) && s < n - 1) {
                    back.push(window_at(at, s, 0.0068, ww, tall.min(0.0086), glazing));
                }
            }
            if !behind_wing(x) {
                if b == door_bay {
                    back.push(door_at(at, back_door(pal)));
                } else {
                    back.push(window_at(at, 0, 0.0085, ww, 0.0082, glazing));
                }
            }
        }
    }
    let sides = (Side::PLAIN, Side::PLAIN);
    if jetty {
        let ground = Range::rect(-hw, hw, zb, zf - JETTY);
        let (lower, upper): (Vec<Opening>, Vec<Opening>) = front.iter().partition(|o| o.y < STOREY);
        k.walls(&ground, y0, STOREY, &wall, [Side::with(&lower), sides.0, Side::NONE, sides.1], true, false);
        k.walls(&body, STOREY, h, &wall, [Side::with(&upper), sides.0, Side::NONE, sides.1], false, true);
        k.walls(&body, y0, h, &wall, [Side::NONE, Side::NONE, Side::with(&back), Side::NONE], true, true);
        if k.near() {
            k.strip(&body, 0, 0.0, w, STOREY - 0.0014, STOREY + 0.0008, 0.0008, *pal.timber.house_frame);
        }
    } else {
        k.walls(&body, y0, h, &wall, [Side::with(&front), sides.0, Side::with(&back), sides.1], true, form != Form::Parapet);
    }
    if fabric.timber {
        let posts: Vec<f32> = (1..bays).map(|b| w * b as f32 / bays as f32).collect();
        if jetty {
            timber_frame(&mut k, &Range::rect(-hw, hw, zb, zf - JETTY), 0, FOOT, STOREY - 0.0014, &posts);
        }
        timber_frame(&mut k, &body, 0, STOREY + 0.0008, h, &posts);
        timber_frame(&mut k, &body, 2, FOOT, h, &posts);
    }

    // the roof, and where a stack stands on a party wall
    let verge = |near: bool| if near { 0.0008 } else { lie.verge };
    let (vr, vl) = (verge(t.neighbours.1), verge(t.neighbours.0));
    let zm = (zf + zb) * 0.5;
    let (stack_z, stack_from, stack_top);
    match form {
        Form::FrontGable => {
            // the ridge from back to front: a gable to the street
            let rise = w * 0.4;
            let rg = Range { cx: 0.0, cz: zm, len: d, dep: w, turn: 3 };
            k.roof(&rg, &RoofSpec { h, rise, eaves: 0.0008, verge: 0.0026, ends: [End::Gable, End::Gable], col: colour, gable: wall.col, thick: lie.thick });
            if !k.far() {
                // a window to the garret
                let g = *pal.glass.window;
                if k.near() {
                    k.strip(&body, 0, hw - 0.0036, hw + 0.0036, h + rise * 0.14, h + rise * 0.14 + 0.0078, 0.0006, wall.frame);
                }
                k.strip(&body, 0, hw - 0.003, hw + 0.003, h + rise * 0.14 + 0.0006, h + rise * 0.14 + 0.0072, 0.0012, g);
                if fabric.timber {
                    let oak = *pal.timber.house_frame;
                    if k.near() {
                        k.strip(&body, 0, 0.0012, w - 0.0012, h - 0.0008, h + 0.0008, 0.0007, oak);
                    }
                }
            }
            (stack_z, stack_from, stack_top) = (zm, h - 0.002, h + rise * 0.55 + 0.007);
        }
        Form::Parapet => {
            // the front wall goes on up, with a coping, and the roof is low behind it
            let (thick, up) = (0.0018, 0.0046);
            if !k.far() {
                k.boxy(-hw, hw, zf - thick, zf, h, h + up, wall.col, Some(wall.sill));
                if k.near() {
                    k.strip(&body, 0, 0.0, w, h - 0.0012, h, 0.0007, wall.sill);
                }
            }
            let rg = Range::rect(-hw, hw, zb, zf - thick);
            let rise = rg.dep * 0.34;
            k.roof(&rg, &RoofSpec { h, rise, eaves: 0.0012, verge: vr.min(vl), ends: [End::Gable, End::Gable], col: colour, gable: wall.col, thick: lie.thick });
            (stack_z, stack_from, stack_top) = (rg.cz, h + rise * 0.5, h + rise + 0.007);
        }
        Form::Double => {
            let rise = d * 0.5 * 0.56;
            for rg in [Range::rect(-hw, hw, zm, zf), Range::rect(-hw, hw, zb, zm)] {
                k.roof(&rg, &RoofSpec { h, rise, eaves: lie.eaves * 0.8, verge: vr.min(vl), ends: [End::Gable, End::Gable], col: colour, gable: wall.col, thick: lie.thick });
            }
            (stack_z, stack_from, stack_top) = (zm, h, h + rise + 0.006);
        }
        Form::Eaves => {
            let rise = d * (0.42 + 0.16 * u);
            // an end with no house against it may be hipped
            let end = |near: bool, pickn: i32| if !near && t.era >= Era::Georgian && r(pickn) < 0.4 { End::Hip } else { End::Gable };
            let ends = [end(t.neighbours.1, 6), end(t.neighbours.0, 22)];
            let spec = RoofSpec { h, rise, eaves: lie.eaves, verge: vr.min(vl), ends, col: colour, gable: wall.col, thick: lie.thick };
            k.roof(&body, &spec);
            if r(7) < 0.3 && n <= 3 {
                // rooms in the roof
                for b in 0..bays {
                    if bays == 3 && b == 1 {
                        continue;
                    }
                    k.dormer(&body, &spec, -hw + w * (b as f32 + 0.5) / bays as f32, 0.0062, 1.0, 0.2, &wall);
                }
            } else if t.era == Era::Late && r(7) < 0.55 {
                k.gablet(&body, &spec, -hw + w * (door_bay as f32 + 0.5) / bays as f32, w / bays as f32 * 0.9, 1.0, &wall, true);
            }
            let hip = ends[0] == End::Hip;
            (stack_z, stack_from, stack_top) = (zm, h + rise * if hip { 0.2 } else { 0.55 }, h + rise + 0.006);
        }
    }

    if wing {
        let rg = Range::rect(wx0, wx1, zb - wing_d, zb);
        let wn = (n - 1).max(1);
        let wh = STOREY * wn as f32;
        // the side that looks into the yard runs into the angle of the house
        let (right, left) = if wing_left { (Side::PLAIN.dark(true, false), Side::PLAIN) } else { (Side::PLAIN, Side::PLAIN.dark(false, true)) };
        let mut ops = vec![door_at(wing_w * 0.3, back_door(pal))];
        for s in 0..wn {
            ops.push(window_at(wing_w * 0.68, s, 0.0082, 0.005, 0.008, glazing));
        }
        k.walls(&rg, -k.drop(&rg), wh, &wall, [Side::NONE, right, Side::with(&ops), left], true, true);
        let roof = Range::rect_across(wx0, wx1, zb - wing_d, zb);
        k.roof(&roof, &RoofSpec { h: wh, rise: wing_w * 0.5 * 0.62, eaves: 0.0018, verge: 0.0016, ends: [End::Gable, End::Party], col: colour, gable: wall.col, thick: lie.thick });
    }

    if !k.far() {
        // stacks on the party walls: on the right, and on the wider houses on the left too
        let (sw, sd) = (0.0056, 0.0068);
        let pots = 2 + (r(14) > 0.5) as usize;
        let top = k.stack(hw - 0.0042, stack_z, sw, sd, stack_from, stack_top, fabric.stack, pots);
        if bays == 3 && r(15) < 0.6 {
            k.stack(-hw + 0.0042, stack_z + (r(23) - 0.5) * 0.004, sw, sd, stack_from, stack_top, fabric.stack, 2);
        }
        if r(16) < 0.16 {
            hearth_smoke(smoke, top, t.seed, pal);
        }
        // the yard behind: flagged, walled at the back and down the left, a privy in a corner
        let rear = -pd * 0.5 - BACK_ROOM + 0.001;
        k.patch(-hw, hw, rear, zb, 0.0034, *pal.ground.back_yard);
        if k.near() {
            k.low_wall((-hw + 0.0008, rear), (hw, rear), 0.006, 0.0014, fabric.yard_wall);
            k.low_wall((-hw + 0.0008, rear), (-hw + 0.0008, zb - if wing && wing_left { wing_d } else { 0.0 }), 0.006, 0.0014, fabric.yard_wall);
            let (px0, px1, fall) = if wing_left { (hw - 0.0068, hw - 0.0012, -1.0) } else { (-hw + 0.002, -hw + 0.0076, 1.0) };
            k.shed(px0, px1, rear + 0.001, rear + 0.0068, 0.008, fall, shade(fabric.yard_wall, 1.08), colour, true);
            if !behind_wing(px0) && v < 0.5 {
                k.butt(if wing_left { hw - 0.003 } else { -hw + 0.003 }, zb - 0.0024);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::houses::PLOT;
    use crate::site::Model;
    use glam::Vec3;

    fn built(t: &Townhouse) -> Model {
        let mut m = Model::default();
        townhouse(&mut m.solid, &mut m.smoke, &Site::level(Vec3::ZERO, 0.0), t, &Palette::standard());
        assert!(m.is_sound());
        m
    }

    #[test]
    fn it_fills_the_width_of_its_plot_and_keeps_to_its_ground() {
        let pal = Palette::standard();
        for n in 0..60 {
            let seed = Seed::of(n);
            let roof = Roof::of(1750, 1850, false, false, seed);
            for era in Era::ALL {
                for detail in Detail::ALL {
                    let m = built(&Townhouse { era, detail, shop: n % 3 == 0, neighbours: (n % 2 == 0, n % 4 < 2), upland: n % 7 == 0, ..Townhouse::new(PLOT.0, PLOT.1, 2 + (n % 3) as u8, roof, seed) });
                    // whatever its plot's cottage was roofed with, a townhouse is not thatched
                    assert!(!m.solid.col.contains(&*pal.thatch.cottage) && !m.solid.col.contains(&*pal.thatch.weathered));
                    let (lo, hi) = m.solid.bounds().unwrap();
                    // wall to wall is the plot, and the eaves reach a little over the neighbours'
                    assert!(hi.x - lo.x >= PLOT.0 * 0.99 - 1e-5 && hi.x - lo.x < PLOT.0 + 0.008, "seed {n} {era:?} {detail:?}: {}", hi.x - lo.x);
                    assert!(lo.z > -PLOT.1 * 0.5 - BACK_ROOM - 0.001 && hi.z < PLOT.1 * 0.5 + 0.007, "seed {n} {era:?} {detail:?}: from {} to {}", lo.z, hi.z);
                }
            }
        }
    }

    #[test]
    fn two_storeys_then_three_and_each_is_taller() {
        let s = Seed::new(0.2, 0.5);
        assert_eq!(storeys(0.3, s), 2);
        assert_eq!(storeys(0.7, s), 3);
        for n in 0..20 {
            let seed = Seed::of(n);
            let top = |storeys: u8| built(&Townhouse { detail: Detail::Far, ..Townhouse::new(PLOT.0, PLOT.1, storeys, Roof::of(1820, 1860, false, false, seed), seed) }).solid.bounds().unwrap().1.y;
            // (a taller house may have a lower roof: a parapet, or two ridges)
            assert!(top(2) + 0.012 < top(3) && top(3) + 0.012 < top(4), "seed {n}");
        }
    }

    #[test]
    fn a_shop_has_its_board_and_no_house_door() {
        let pal = Palette::standard();
        let seed = Seed::of(4);
        let roof = Roof::of(1820, 1860, false, false, seed);
        let count = |m: &Model, c: crate::colour::Rgba| m.solid.col.iter().filter(|x| **x == c).count();
        for detail in [Detail::Play, Detail::Near] {
            let (house, shop) = (built(&Townhouse { detail, ..Townhouse::new(PLOT.0, PLOT.1, 3, roof, seed) }), built(&Townhouse { detail, shop: true, ..Townhouse::new(PLOT.0, PLOT.1, 3, roof, seed) }));
            assert_ne!(house, shop);
            // the upper floors are the same house
            assert_eq!(house.solid.bounds().unwrap().1.y, shop.solid.bounds().unwrap().1.y);
            // no house door, and at this level or closer a name board in the shop's paint
            let doors = |m: &Model| pal.walls.door_paints.iter().map(|p| count(m, **p)).sum::<usize>();
            assert!(doors(&house) >= 2 && doors(&shop) < doors(&house), "{detail:?}");
            assert!(shop.solid.triangles() != house.solid.triangles());
        }
        // far off there is no telling them apart
        let far = |shop: bool| built(&Townhouse { detail: Detail::Far, shop, ..Townhouse::new(PLOT.0, PLOT.1, 3, roof, seed) });
        assert_eq!(far(false), far(true));
    }

    #[test]
    fn each_level_draws_more_than_the_last() {
        for n in 0..60 {
            let seed = Seed::of(n);
            let at = |detail: Detail| built(&Townhouse { detail, era: Era::of(1700 + 3 * n as i32), ..Townhouse::new(PLOT.0, PLOT.1, 3, Roof::of(1750, 1850, false, false, seed), seed) });
            let (far, play, near) = (at(Detail::Far), at(Detail::Play), at(Detail::Near));
            // its walls and its roof, which may be two ridges or hipped at the ends
            assert!(far.solid.triangles() <= 4 * 2 + 12, "seed {n}: {} far off", far.solid.triangles());
            assert!(far.smoke.is_empty());
            assert!(play.solid.triangles() > far.solid.triangles() + 20 && play.solid.triangles() < 130, "seed {n}: {} at play", play.solid.triangles());
            assert!(near.solid.triangles() > play.solid.triangles() * 2, "seed {n}");
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_house() {
        let roof = Roof::of(1820, 1860, false, false, Seed::of(3));
        let build = |n: u32| built(&Townhouse::new(PLOT.0, PLOT.1, 3, roof, Seed::of(n)));
        assert_eq!(build(3), build(3));
        assert_ne!(build(3), build(4));
    }
}
