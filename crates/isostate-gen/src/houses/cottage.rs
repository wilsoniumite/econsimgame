use crate::colour::shade;
use crate::hash::Seed;
use crate::houses::kit::{Detail, End, Era, Kit, Open, Opening, Over, Range, RoofSpec, Side, FOOT};
use crate::houses::roof::{Cover, Roof};
use crate::houses::{door, door_at, garden_tree, hearth_smoke, timber_frame, window, window_at, Fabric, BACK_ROOM, SET_BACK};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::houses::kit::STOREY;
use crate::site::Site;
use crate::trees::Greenery;

/// A house standing on its own plot.
#[derive(Clone, Copy, Debug)]
pub struct Cottage {
    /// the plot it stands on; the house is a little narrower, so the gaps show
    pub plot_width: f32,
    pub plot_depth: f32,
    /// one to three
    pub storeys: u8,
    /// what it was roofed with when it went up, and how weathered that is now
    pub roof: Roof,
    /// when its plot was first built, in the steps its style takes
    pub era: Era,
    /// it stands in the hills: stone, to the ground
    pub upland: bool,
    /// a house stands close on its left (as seen from the street) and on its right
    pub neighbours: (bool, bool),
    /// how much of it to draw
    pub detail: Detail,
    pub seed: Seed,
}

impl Cottage {
    /// A cottage of the middle years in the lowland with no near neighbours, drawn in full.
    pub fn new(plot_width: f32, plot_depth: f32, storeys: u8, roof: Roof, seed: Seed) -> Cottage {
        Cottage { plot_width, plot_depth, storeys, roof, era: Era::Georgian, upland: false, neighbours: (false, false), detail: Detail::Near, seed }
    }
}

/// Each house adds a storey at its own density, so a street grows a house at a time.
pub fn storeys(density: f32, seed: Seed) -> u8 {
    1 + (density >= 0.35 + 0.35 * seed.v) as u8 + (density >= 0.8 + 0.15 * seed.u) as u8
}

/// How far the upper storeys of a timber house oversail its ground storey.
const JETTY: f32 = 0.0026;

/// A cottage: washed walls (stone in the hills, brick once brick is to be
/// had) under a steep roof, a little askew on its plot, toward the front of
/// it, with its garden behind. The roof is what the house was given when it
/// went up; thatch, which is steeper and deeper at the eaves, stays only on
/// a cottage of one storey, as one that has been raised was roofed again in
/// tile. The oldest are timber-framed, lean, and sag at the ridge; one of
/// two storeys or more is jettied. About half have an outshut at the back
/// under a catslide or a lean-to. Every one has a chimney at one end, built
/// against the end wall on the older ones, and a few of them smoke.
///
/// From far off it is four walls and a roof. At the distance the game is
/// played from it has its openings, its chimney, its outshut, the green of
/// its garden and perhaps a tree. Close to, the windows sit in the wall and
/// have frames, the door has a step and a path, the thatch has thickness,
/// and the garden has its beds, its hedge (a wall in the hills, a paling
/// later), a woodpile and a water butt.
pub fn cottage(solid: &mut MeshBuf, smoke: &mut MeshBuf, green: &mut Greenery, site: &Site, c: &Cottage, pal: &Palette) {
    let Seed { u, v } = c.seed;
    let r = |n: i32| c.seed.pick(n);
    let age: f32 = match c.era {
        Era::Old => 1.0,
        Era::Georgian => 0.5,
        _ => 0.12,
    };
    let mut k = Kit::new(solid, site, pal, c.detail, (v - 0.5) * 0.1 * age.max(0.3));
    let (pw, pd) = (c.plot_width, c.plot_depth);
    let (w, d) = (pw * (0.66 + 0.2 * v), pd * (0.74 + 0.12 * u));
    let (hw, zf) = (w * 0.5, pd * 0.5 - 0.0035);
    let zb = zf - d;
    let n = c.storeys.max(1) as usize;
    let roof = if n > 1 { c.roof.rebuilt() } else { c.roof };
    let (lie, thatch) = (roof.cover.lie(), roof.cover == Cover::Thatch);
    let h = STOREY * n as f32 * if thatch { 0.92 } else { 1.0 };
    let rise = d * lie.rise;
    let fabric = Fabric::of(pal, c.era, c.upland, 0.0, 0.6, c.seed);
    let wall = fabric.wall;
    let body = Range::rect(-hw, hw, zb, zf);
    let y0 = -k.drop(&body);
    let jetty = fabric.timber && n >= 2 && !k.far();
    if age > 0.2 {
        let dip = 0.0028 * age * (0.5 + roof.weathering as f32 / 6.0) * if thatch { 1.3 } else { 1.0 };
        k.crooked(((r(9) - 0.5) * 0.07 * age, (r(10) - 0.5) * 0.04 * age), dip, hw + lie.verge, h, h + rise);
    }

    // the outshut at the back: across the whole of it, or over one end
    let outshut = n <= 2 && r(6) < 0.55 && !k.far();
    let (ox0, ox1) = if r(7) < 0.35 { (-hw, hw) } else if r(7) < 0.68 { (-hw, -hw + w * 0.58) } else { (hw - w * 0.58, hw) };
    let under = |x: f32| outshut && x > ox0 && x < ox1;

    // openings by rule: two or three bays, the door in one of them, a window over each
    let bays = if w >= 0.04 { 3 } else { 2 };
    let door_bay = if bays == 3 { 1 } else { (r(3) < 0.5) as usize };
    let glazing = window(c.era);
    let (ww, wh) = if c.era == Era::Old { (0.0062, 0.0068) } else { (0.0056, 0.0082) };
    let mut front = Vec::new();
    let mut back = Vec::new();
    if !k.far() {
        for b in 0..bays {
            let at = w * (b as f32 + 0.5) / bays as f32;
            if b == door_bay {
                front.push(door_at(at, door(pal, c.era, c.seed)));
            } else {
                front.push(window_at(at, 0, 0.0088, ww, wh, glazing));
            }
            for s in 1..n {
                front.push(window_at(at, s, 0.0078, ww, wh * 0.94, glazing));
                back.push(window_at(at, s, 0.0078, ww, wh * 0.94, glazing));
            }
            // the back is seen from its own left: its first bay is the house's right
            let x = hw - at;
            if !under(x) {
                if b == door_bay && !outshut {
                    back.push(door_at(at, back_door(pal)));
                } else if b != door_bay {
                    back.push(window_at(at, 0, 0.0088, ww, wh, glazing));
                }
            }
        }
    }
    let dim = |near: bool| if near { 0.84 } else { 1.0 };
    let (right, left) = (Side::PLAIN.dim(dim(c.neighbours.1)), Side::PLAIN.dim(dim(c.neighbours.0)));
    if jetty {
        // the ground storey stands back under the storeys above
        let ground = Range::rect(-hw, hw, zb, zf - JETTY);
        let (lower, upper): (Vec<Opening>, Vec<Opening>) = front.iter().partition(|o| o.y < STOREY);
        k.walls(&ground, y0, STOREY, &wall, [Side::with(&lower), right, Side::NONE, left], true, false);
        k.walls(&body, STOREY, h, &wall, [Side::with(&upper), right, Side::NONE, left], false, true);
        k.walls(&body, y0, h, &wall, [Side::NONE, Side::NONE, Side::with(&back), Side::NONE], true, true);
        if k.near() {
            k.strip(&body, 0, 0.0, w, STOREY - 0.0014, STOREY + 0.0008, 0.0008, *pal.timber.house_frame);
        }
    } else {
        k.walls(&body, y0, h, &wall, [Side::with(&front), right, Side::with(&back), left], true, true);
    }
    if fabric.timber {
        let posts: Vec<f32> = (1..bays).map(|b| w * b as f32 / bays as f32).collect();
        if jetty {
            timber_frame(&mut k, &Range::rect(-hw, hw, zb, zf - JETTY), 0, FOOT, STOREY - 0.0014, &posts);
            timber_frame(&mut k, &body, 0, STOREY + 0.0008, h, &posts);
        } else {
            timber_frame(&mut k, &body, 0, FOOT, h, &posts);
        }
        timber_frame(&mut k, &body, 2, FOOT, h, &posts);
        timber_frame(&mut k, &body, 1, FOOT, h, &[d * 0.5]);
        timber_frame(&mut k, &body, 3, FOOT, h, &[d * 0.5]);
    }

    let colour = roof.colour(pal, c.seed);
    let hipped = thatch && r(8) < 0.5 && !k.far();
    let end = if hipped { End::HalfHip } else { End::Gable };
    let spec = RoofSpec { h, rise, eaves: lie.eaves, verge: lie.verge, ends: [end, end], col: colour, gable: wall.col, thick: lie.thick };
    k.roof(&body, &spec);

    if outshut {
        // a range whose low side is the back
        let depth = 0.0085;
        let rg = Range { cx: (ox0 + ox1) * 0.5, cz: zb - depth * 0.5, len: ox1 - ox0, dep: depth, turn: 2 };
        let (high, low) = if n == 1 { (h, (h - depth * spec.pitch(&body) * 0.8).max(0.0105)) } else { (STOREY + 0.0045, 0.0125) };
        let ops = [Opening { u: (ox1 - ox0) * 0.3, y: FOOT, w: 0.0046, h: (low - FOOT - 0.0036).min(0.012), kind: back_door(pal) }];
        k.lean_to(&rg, low, high, &wall, shade(colour, 0.82), lie.eaves * 0.8, Side::with(&ops), [true, true]);
    }

    // the chimney at one end: built against the end wall on the older houses where the plot has room
    if !k.far() {
        let side = if u < 0.5 { 1.0 } else { -1.0 };
        let zm = (zf + zb) * 0.5 + (r(19) - 0.5) * 0.004;
        let top = h + rise + 0.0055;
        let outside = age >= 0.5 && !hipped && hw + 0.0042 <= pw * 0.5 - 0.0003;
        let at = if outside {
            let (x0, x1) = (side * (hw - 0.0004), side * (hw + 0.0038));
            let (x0, x1) = (x0.min(x1), x0.max(x1));
            k.boxy(x0, x1, zm - 0.0044, zm + 0.0044, y0, h * 0.86, fabric.stack, Some(shade(fabric.stack, 1.06)));
            k.stack((x0 + x1) * 0.5, zm, 0.0042, 0.0054, h * 0.86 - 0.001, top, fabric.stack, 1)
        } else {
            k.stack(side * (hw - 0.0046), zm, 0.0056, 0.0052, h + rise * 0.45, top, fabric.stack, if bays == 3 { 2 } else { 1 })
        };
        if r(11) < 0.22 {
            hearth_smoke(smoke, at, c.seed, pal);
        }

        // the garden: the plot and the ground behind it
        let rear = -pd * 0.5 - BACK_ROOM + 0.001;
        let (gx0, gx1) = (-pw * 0.5 + 0.0008, pw * 0.5 - 0.0008);
        k.patch(gx0, gx1, rear, zf + 0.001, 0.0034, *pal.ground.garden);
        let tree = r(15) < 0.28;
        if tree {
            garden_tree(green, &k, site, -side * (pw * 0.5 - 0.009), rear + 0.007, 0.24 + 0.08 * r(16), c.seed, pal);
        }
        if k.near() {
            let door_x = -hw + w * (door_bay as f32 + 0.5) / bays as f32;
            k.patch(door_x - 0.002, door_x + 0.002, zf + 0.002, pd * 0.5 + SET_BACK - 0.0005, 0.0042, *pal.ground.garden_path);
            // beds dug across the back of it, a row of something green down each
            let back_of_house = zb - if outshut { 0.0085 + lie.eaves } else { 0.0 };
            let room = back_of_house - rear - 0.004;
            let beds = ((room / 0.0042).floor() as usize).min(3);
            let (bx0, bx1) = if tree { if side > 0.0 { (gx0 + 0.016, gx1 - 0.003) } else { (gx0 + 0.003, gx1 - 0.016) } } else { (gx0 + 0.003, gx1 - 0.003) };
            for i in 0..beds {
                let z = rear + 0.003 + 0.0042 * i as f32;
                k.patch(bx0, bx1, z, z + 0.003, 0.0038, *pal.ground.garden_bed);
                k.patch(bx0 + 0.001, bx1 - 0.001, z + 0.0009, z + 0.0021, 0.0052, *pal.foliage.vegetables);
            }
            // its bounds at the back and down the left: a hedge, a wall in the hills, a paling later
            let (bound, bh, bt) = if c.upland { (*pal.stone.garden_wall, 0.0042, 0.0018) } else if age >= 0.5 { (*pal.foliage.hedge, 0.005, 0.0026) } else { (*pal.timber.fence, 0.0045, 0.0008) };
            k.low_wall((gx0 + 0.001, rear), (gx1, rear), bh, bt, bound);
            k.low_wall((gx0 + 0.001, rear), (gx0 + 0.001, zb), bh, bt, bound);
            // wood cut for the winter against the end wall away from the chimney, and a butt under the eaves
            if r(13) < 0.6 && pw * 0.5 - hw > 0.0042 {
                let x = -side * (hw + 0.0002);
                k.woodpile(x.min(x - side * 0.003), zb + 0.004, 0.003, d * 0.4);
            }
            let bx = side * (hw - 0.003);
            if r(14) < 0.5 && !under(bx) {
                k.butt(bx, zb - 0.0026);
            }
        }
    }
}

/// A back door: plain boards.
pub(super) fn back_door(pal: &Palette) -> Open {
    Open::Door { paint: *pal.timber.door, over: Over::Nothing }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::houses::PLOT;
    use crate::site::Model;
    use glam::Vec3;

    fn built(c: &Cottage) -> Model {
        let mut m = Model::default();
        cottage(&mut m.solid, &mut m.smoke, &mut m.green, &Site::level(Vec3::ZERO, 0.0), c, &Palette::standard());
        assert!(m.is_sound());
        m
    }

    fn roofed(storeys: u8, roof: Roof, seed: Seed) -> Model {
        built(&Cottage::new(PLOT.0, PLOT.1, storeys, roof, seed))
    }

    /// A cottage standing when the game opens, as it is a century on: some are thatched.
    fn build(storeys: u8, seed: Seed) -> Model {
        roofed(storeys, Roof::of(1750, 1850, false, false, seed), seed)
    }

    #[test]
    fn thatch_is_steeper_and_stays_only_on_one_storey() {
        let pal = Palette::standard();
        let seed = Seed::of(2);
        let (thatch, tile) = (Roof { cover: Cover::Thatch, weathering: 0 }, Roof { cover: Cover::Tile, weathering: 0 });
        let far = |roof: Roof| built(&Cottage { detail: Detail::Far, era: Era::Late, ..Cottage::new(PLOT.0, PLOT.1, 1, roof, seed) });
        let (a, b) = (far(thatch), far(tile));
        assert!(a.solid.col.contains(&*pal.thatch.cottage) && !b.solid.col.contains(&*pal.thatch.cottage));
        // the ridge is higher above the eaves, though the walls are a little lower
        let top = |m: &Model| m.solid.bounds().unwrap().1.y;
        assert!(top(&a) - STOREY * 0.92 > top(&b) - STOREY + 0.003);
        // raised a storey, it was roofed again in tile
        assert!(!roofed(2, thatch, seed).solid.col.contains(&*pal.thatch.cottage));
    }

    #[test]
    fn a_storey_is_added_at_a_density_of_the_house_s_own() {
        let s = Seed::new(0.5, 0.5);
        assert_eq!(storeys(0.0, s), 1);
        assert_eq!(storeys(0.52, s), 1);
        assert_eq!(storeys(0.53, s), 2);
        assert_eq!(storeys(0.88, s), 3);
        // two houses on the same ground need not stand at the same step
        assert_ne!(storeys(0.5, Seed::new(0.5, 0.1)), storeys(0.5, Seed::new(0.5, 0.9)));
    }

    #[test]
    fn it_stays_on_its_plot_and_grows_with_its_storeys() {
        for n in 0..40 {
            let seed = Seed::of(n);
            for era in Era::ALL {
                for detail in Detail::ALL {
                    let mut last = 0.0;
                    for storeys in 1..=3 {
                        let m = built(&Cottage { era, detail, upland: n % 5 == 0, ..Cottage::new(PLOT.0, PLOT.1, storeys, Roof::of(1750, 1850, false, false, seed), seed) });
                        let (lo, hi) = m.solid.bounds().unwrap();
                        // the eaves of the widest, turned a little, overhang the plot by a hair, no more
                        assert!(lo.x > -PLOT.0 * 0.5 - 0.004 && hi.x < PLOT.0 * 0.5 + 0.004, "seed {n} {era:?}: from {} to {}", lo.x, hi.x);
                        // behind, it keeps to its plot and the room behind that; before, to its path
                        assert!(lo.z > -PLOT.1 * 0.5 - BACK_ROOM - 0.003 && hi.z < PLOT.1 * 0.5 + SET_BACK + 0.003, "seed {n} {era:?}: from {} to {}", lo.z, hi.z);
                        assert!(hi.y > last, "seed {n}: no taller at {storeys} storeys");
                        last = hi.y;
                    }
                }
            }
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_house() {
        assert_eq!(build(2, Seed::of(3)), build(2, Seed::of(3)));
        assert_ne!(build(2, Seed::of(3)), build(2, Seed::of(4)));
    }

    #[test]
    fn every_cottage_has_a_chimney_and_a_few_of_them_smoke() {
        let pal = Palette::standard();
        let mut smoking = 0;
        for n in 0..400 {
            let m = build(1, Seed::of(n));
            assert!(m.solid.col.contains(&*pal.smoke.stack_top), "seed {n} has no chimney");
            smoking += !m.smoke.is_empty() as usize;
        }
        assert!((50..130).contains(&smoking), "{smoking} of 400 smoke");
    }

    #[test]
    fn each_level_draws_more_than_the_last_and_far_is_a_shell_and_a_roof() {
        let pal = Palette::standard();
        for n in 0..40 {
            let seed = Seed::of(n);
            let at = |detail: Detail| built(&Cottage { detail, era: Era::of(1700 + 5 * n as i32), ..Cottage::new(PLOT.0, PLOT.1, 2, Roof::of(1750, 1850, false, false, seed), seed) });
            let (far, play, near) = (at(Detail::Far), at(Detail::Play), at(Detail::Near));
            // four walls, two slopes and two gables
            assert_eq!(far.solid.triangles(), 4 * 2 + 2 * 2 + 2, "seed {n}");
            assert!(far.smoke.is_empty() && far.green.is_empty());
            assert!(!far.solid.col.contains(&*pal.glass.window));
            assert!(play.solid.triangles() > far.solid.triangles() * 2 && play.solid.triangles() < 90, "seed {n}: {} at play", play.solid.triangles());
            assert!(play.solid.col.contains(&*pal.glass.window) && play.solid.col.contains(&*pal.ground.garden));
            assert!(near.solid.triangles() > play.solid.triangles() * 2, "seed {n}");
            // the same house at every level: but for its chimney, as tall
            let (f, p) = (far.solid.bounds().unwrap(), near.solid.bounds().unwrap());
            assert!(p.1.y > f.1.y && p.1.y - f.1.y < 0.012);
        }
    }

    #[test]
    fn the_oldest_sag_at_the_ridge_and_the_newest_are_level() {
        let pal = Palette::standard();
        let seed = Seed::of(7);
        let roof = Roof { cover: Cover::Tile, weathering: 3 };
        let ridge = |era: Era| {
            let m = built(&Cottage { era, ..Cottage::new(PLOT.0, PLOT.1, 1, roof, seed) });
            // the corners of the front slope that lie along the ridge: how far the lowest is below the highest
            let ys: Vec<f32> = m.solid.pos.iter().zip(&m.solid.col).filter(|(_, c)| **c == roof.colour(&pal, seed)).map(|(p, _)| p[1]).collect();
            let top = ys.iter().cloned().fold(f32::MIN, f32::max);
            top - ys.iter().cloned().filter(|y| *y > top - 0.004).fold(f32::MAX, f32::min)
        };
        assert!(ridge(Era::Late) < 1e-5, "a new ridge is level");
        let old = ridge(Era::Old);
        assert!(old > 0.001 && old < 0.004, "an old one dips by {old}");
    }

    #[test]
    fn it_sits_into_a_slope() {
        let slope = |x: f32, z: f32| 0.1 + x * 0.15 + z * 0.1;
        let pal = Palette::standard();
        let seed = Seed::of(5);
        let mut m = Model::default();
        let c = Cottage::new(PLOT.0, PLOT.1, 2, Roof::of(1750, 1850, false, false, seed), seed);
        cottage(&mut m.solid, &mut m.smoke, &mut m.green, &Site::on(Vec3::new(0.0, 0.1, 0.0), 0.0, &slope), &c, &pal);
        assert!(m.is_sound());
        // walls reach the ground at the lowest corner, and nothing is buried deeper than the fall across the plot
        let low = m.solid.pos.iter().map(|p| p[1] - slope(p[0], p[2])).fold(f32::MAX, f32::min);
        assert!(low < 0.0 && low > -0.02, "{low}");
    }
}
