use crate::colour::shade;
use crate::hash::Seed;
use crate::kit::{Detail, End, Era, Kit, Open, Opening, Over, Range, RoofSpec, Side, Wall, FOOT};
use crate::houses::roof::{Cover, Roof};
use crate::houses::{door, door_at, garden_tree, hearth_smoke, timber_frame, window, window_at, Fabric, Street, BACK_ROOM, SET_BACK};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::kit::STOREY;
use crate::site::Site;
use crate::trees::Greenery;

/// A house standing on its own plot.
#[derive(Clone, Copy, Debug)]
pub struct Cottage {
    /// the plot it stands on; the house is a little narrower, so the gaps
    /// show. In a street of a family a plot of `WIDE_PLOT` or more has
    /// room for a cross-wing
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
    /// the family its street's walls and tiles run in, and its street's
    /// habits of building (`Street::ALONE` for a house told nothing of it)
    pub street: Street,
    /// it was built lower and raised to the storeys it has now in this age.
    /// Where that is a later age than its own, the later work shows above
    /// the old: brick over wash once brick was to be had, sashes over
    /// casements, and no frame or jetty above a framed ground storey. None
    /// for a house built as it stands
    pub raised: Option<Era>,
    /// how much of it to draw
    pub detail: Detail,
    pub seed: Seed,
}

/// A plot at least this wide has room for a cottage with a cross-wing, or
/// a long one of four bays: the widest plots of a town's ways out, and those
/// of its furthest lanes.
pub const WIDE_PLOT: f32 = 0.056;

/// How a cottage stands on its plot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// its ridge along the street
    Along,
    /// its gable to the street, its ridge running back
    EndOn,
    /// a range along the street with a cross-wing standing forward at one end, gable to the street
    Wing,
}

impl Cottage {
    /// A cottage of the middle years in the lowland with no near neighbours, told nothing of its street, drawn in full.
    pub fn new(plot_width: f32, plot_depth: f32, storeys: u8, roof: Roof, seed: Seed) -> Cottage {
        Cottage { plot_width, plot_depth, storeys, roof, era: Era::Georgian, upland: false, neighbours: (false, false), street: Street::ALONE, raised: None, detail: Detail::Near, seed }
    }

    /// How it stands on its plot. A cottage told nothing of its street
    /// stands with its ridge along it, as every cottage once did. In a
    /// street of a family the street's habit settles it, and a number of
    /// the house's own: on a plot of `WIDE_PLOT` or more four in ten have
    /// a cross-wing, taking the streets together; and of the cottages built
    /// before the brick years on the narrower plots one in seven stands
    /// end-on, its gable to the street, as the old lanes of a town have
    /// them. Either is common in one street and not seen in the next.
    pub fn form(&self) -> Form {
        let own = self.seed.pick(45);
        if self.plot_width >= WIDE_PLOT {
            if own < self.street.habit(0.4, 7) { Form::Wing } else { Form::Along }
        } else if self.era <= Era::Georgian && own > 1.0 - self.street.habit(0.14, 8) {
            Form::EndOn
        } else {
            Form::Along
        }
    }
}

/// Each house adds a storey at its own density, so a street grows a house at a time.
pub fn storeys(density: f32, seed: Seed) -> u8 {
    1 + (density >= 0.35 + 0.35 * seed.v) as u8 + (density >= 0.8 + 0.15 * seed.u) as u8
}

/// How far the upper storeys of a timber house oversail its ground storey.
const JETTY: f32 = 0.0026;
/// How much higher than a storey the eaves of a cottage with rooms in its roof stand.
const HALF_STOREY: f32 = 0.4;

/// How the walls of a range are put up.
#[derive(Clone, Copy, PartialEq)]
enum Build {
    /// in one build, from the ground to the eaves
    Whole,
    /// the storeys above oversail the ground storey at the front: where the back and the front of the range are
    Jettied(f32, f32),
    /// the ground storey of one age, what is above it of a later one
    Raised,
}

/// The four sides of a range: the openings of each, whether it is drawn,
/// how dim it is, and whether it runs into a corner at its beginning or its end.
#[derive(Default)]
struct Faces {
    ops: [Vec<Opening>; 4],
    hide: [bool; 4],
    dim: [f32; 4],
    dark: [(bool, bool); 4],
}

impl Faces {
    fn new() -> Faces {
        Faces { dim: [1.0; 4], ..Default::default() }
    }

    fn side<'o>(&self, i: usize, ops: &'o [Opening]) -> Side<'o> {
        if self.hide[i] { Side::NONE } else { Side::with(ops).dim(self.dim[i]).dark(self.dark[i].0, self.dark[i].1) }
    }
}

/// The walls of a range from `y0` to `h`: `lower` is what the house was
/// built of, `upper` the later work of one that was raised.
#[allow(clippy::too_many_arguments)]
fn put_up(k: &mut Kit, rg: &Range, y0: f32, h: f32, lower: &Wall, upper: &Wall, how: Build, f: &Faces) {
    if how == Build::Whole {
        k.walls(rg, y0, h, lower, [f.side(0, &f.ops[0]), f.side(1, &f.ops[1]), f.side(2, &f.ops[2]), f.side(3, &f.ops[3])], true, true);
        return;
    }
    let parts: [(Vec<Opening>, Vec<Opening>); 4] = std::array::from_fn(|i| f.ops[i].iter().partition(|o| o.y < STOREY));
    if how == Build::Raised {
        k.walls(rg, y0, STOREY, lower, [f.side(0, &parts[0].0), f.side(1, &parts[1].0), f.side(2, &parts[2].0), f.side(3, &parts[3].0)], true, false);
        k.walls(rg, STOREY, h, upper, [f.side(0, &parts[0].1), f.side(1, &parts[1].1), f.side(2, &parts[2].1), f.side(3, &parts[3].1)], false, true);
        return;
    }
    // the ground storey stands back under the storeys above, at the front only
    let Build::Jettied(back, front) = how else { return };
    let ground = Range::rect(rg.cx - rg.len * 0.5, rg.cx + rg.len * 0.5, back, front - JETTY);
    k.walls(&ground, y0, STOREY, lower, [f.side(0, &parts[0].0), f.side(1, &parts[1].0), Side::NONE, f.side(3, &parts[3].0)], true, false);
    k.walls(rg, STOREY, h, lower, [f.side(0, &parts[0].1), f.side(1, &parts[1].1), Side::NONE, f.side(3, &parts[3].1)], false, true);
    k.walls(rg, y0, h, lower, [Side::NONE, Side::NONE, f.side(2, &f.ops[2]), Side::NONE], true, true);
    if k.near() {
        k.strip(rg, 0, 0.0, rg.len, STOREY - 0.0014, STOREY + 0.0008, 0.0008, *k.pal.timber.house_frame);
    }
}

/// A cottage: washed walls (stone in the hills, brick once brick is to be
/// had) under a steep roof, a little askew on its plot, toward the front of
/// it, with its garden behind. The roof is what the house was given when it
/// went up; thatch, which is steeper and deeper at the eaves, stays only on
/// a cottage of one storey, as one that has been raised was roofed again in
/// tile. The oldest are timber-framed, lean, and sag at the ridge; one of
/// two storeys or more is jettied. About half have an outshut at the back
/// under a catslide or a lean-to. Every one has a chimney, and a few of
/// them smoke.
///
/// A cottage told nothing of its street is as it always was: its ridge
/// along the street, gabled, one chimney at an end. In a street of a family
/// (`Street`) its walls and its tiles run in the family, and the street's
/// habits and a number of its own settle the rest: how it stands
/// (`Cottage::form`: along the street, end-on with its gable to it and
/// running back down its plot, or with a cross-wing on a wide plot, where
/// the longest have four bays); a roof hipped at both ends on those built
/// from the eighteenth century on; rooms in the roof of a cottage of one
/// storey, their windows in small gables standing on the eaves; and where
/// its chimney stands: at one end, on the ridge in the middle, as the oldest
/// plans had it, or at both ends. One that was raised in a later age shows
/// it (`Cottage::raised`).
///
/// From far off it is its walls and its roof. At the distance the game is
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
    let form = c.form();
    let st = &c.street;
    let n = c.storeys.max(1) as usize;
    let roof = if n > 1 { c.roof.rebuilt() } else { c.roof };
    let (lie, thatch) = (roof.cover.lie(), roof.cover == Cover::Thatch);
    let fabric = Fabric::of(pal, c.era, c.upland, 0.0, 0.6, c.seed, &c.street);
    let wall = fabric.wall;
    // the later work of a house that was raised: of brick where the later age built this house in brick
    let later = c.raised.filter(|e| n >= 2 && *e > c.era);
    let above = later.map(|e| Fabric::of(pal, e, c.upland, 0.0, 0.6, c.seed, &c.street)).filter(|f| f.brick && !fabric.brick).map_or(wall, |f| f.wall);
    let glazing = window(c.era);
    let glazing_above = later.map_or(glazing, window);
    let how = if later.is_some() && (above.col != wall.col || fabric.timber || glazing_above != glazing) {
        Build::Raised
    } else if fabric.timber && n >= 2 && later.is_none() && form != Form::Wing && !k.far() {
        Build::Jettied(0.0, 0.0)
    } else {
        Build::Whole
    };
    // rooms in the roof: the eaves stand at a storey and a half
    let half = n == 1 && !thatch && form != Form::Wing && r(46) < st.habit(0.2, 9);
    let h = STOREY * n as f32 * if thatch { 0.92 } else { 1.0 } + if half { STOREY * HALF_STOREY } else { 0.0 };

    // how much of its plot it takes, by how it stands
    let (w, d, wing_d) = match form {
        Form::Along => (pw * (0.66 + 0.2 * v), pd * (0.74 + 0.12 * u), 0.0),
        // (end-on it runs back down its plot, into the ground an outshut would take)
        Form::EndOn => (pw * (0.5 + 0.08 * v), pd * (0.96 + 0.14 * u), 0.0),
        Form::Wing => (pw * (0.72 + 0.14 * v), pd * (0.62 + 0.1 * u), 0.0058 + 0.003 * r(49)),
    };
    // the front of the house, and of its range where a wing stands forward of it
    let (hw, zf) = (w * 0.5, pd * 0.5 - 0.0035);
    let zfm = zf - wing_d;
    let zb = zfm - d;
    let how = if let Build::Jettied(..) = how { Build::Jettied(zb, zfm) } else { how };
    let body = Range::rect(-hw, hw, zb, zfm);
    // the range its roof is fitted to: for a house end-on, the same ground with its length running to the front
    let roofed = if form == Form::EndOn { Range { cx: 0.0, cz: (zfm + zb) * 0.5, len: d, dep: w, turn: 3 } } else { body };
    let rise = if form == Form::EndOn { w * lie.rise * 0.9 } else { d * lie.rise };
    let y0 = -k.drop(&body);
    if age > 0.2 {
        let dip = 0.0028 * age * (0.5 + roof.weathering as f32 / 6.0) * if thatch { 1.3 } else { 1.0 };
        k.crooked(((r(9) - 0.5) * 0.07 * age, (r(10) - 0.5) * 0.04 * age), if form == Form::EndOn { dip * 0.4 } else { dip }, hw + lie.verge, h, h + rise);
    }

    // the cross-wing, at one end
    let wing_left = r(48) < 0.5;
    let wing_w = (w * 0.4).clamp(0.017, 0.024);
    let (wx0, wx1) = if wing_left { (-hw, -hw + wing_w) } else { (hw - wing_w, hw) };
    let before_wing = |x: f32| form == Form::Wing && x > wx0 && x < wx1;

    // the outshut at the back: across the whole of it, or over one end
    let rear = -pd * 0.5 - BACK_ROOM + 0.001;
    let out_depth = if form == Form::Along { 0.0085 } else { (zb - rear - 0.002).min(0.0085) };
    let outshut = n <= 2 && r(6) < 0.55 && out_depth >= 0.006 && !k.far();
    let (ox0, ox1) = if r(7) < 0.35 || form == Form::EndOn { (-hw, hw) } else if r(7) < 0.68 { (-hw, -hw + w * 0.58) } else { (hw - w * 0.58, hw) };
    let under = |x: f32| outshut && x > ox0 && x < ox1;

    // openings by rule: two to four bays, the door in one of them, a window over each
    let bays = match form {
        Form::EndOn => 2,
        _ if w >= 0.052 && st.family.is_some() => 4,
        _ if w >= 0.04 => 3,
        _ => 2,
    };
    let bay_at = |b: usize| w * (b as f32 + 0.5) / bays as f32;
    let door_bay = match (form, bays) {
        // beside the wing
        (Form::Wing, _) => (0..bays).filter(|&b| !before_wing(-hw + bay_at(b))).min_by_key(|&b| if wing_left { b } else { bays - b }).unwrap_or(0),
        (_, 3) => 1,
        (_, 4) => 1 + (r(3) < 0.5) as usize,
        _ => (r(3) < 0.5) as usize,
    };
    let (ww, wh) = if c.era == Era::Old { (0.0062, 0.0068) } else { (0.0056, 0.0082) };
    let mut faces = Faces::new();
    let mut wing_front = Vec::new();
    if !k.far() {
        for b in 0..bays {
            let at = bay_at(b);
            if !before_wing(-hw + at) {
                if b == door_bay {
                    faces.ops[0].push(door_at(at, door(pal, c.era, c.seed)));
                } else {
                    faces.ops[0].push(window_at(at, 0, 0.0088, ww, wh, glazing));
                }
                for s in 1..n {
                    faces.ops[0].push(window_at(at, s, 0.0078, ww, wh * 0.94, glazing_above));
                }
            }
            for s in 1..n {
                faces.ops[2].push(window_at(at, s, 0.0078, ww, wh * 0.94, glazing_above));
            }
            // the back is seen from its own left: its first bay is the house's right
            let x = hw - at;
            if !under(x) {
                if b == door_bay && !outshut {
                    faces.ops[2].push(door_at(at, back_door(pal)));
                } else if b != door_bay {
                    faces.ops[2].push(window_at(at, 0, 0.0088, ww, wh, glazing));
                }
            }
        }
        if form == Form::Wing {
            wing_front.push(window_at(wing_w * 0.5, 0, 0.0088, ww, wh, glazing));
            for s in 1..n {
                wing_front.push(window_at(wing_w * 0.5, s, 0.0078, ww, wh * 0.94, glazing_above));
            }
        }
    }
    let dim = |near: bool| if near { 0.84 } else { 1.0 };
    (faces.dim[1], faces.dim[3]) = (dim(c.neighbours.1), dim(c.neighbours.0));
    put_up(&mut k, &body, y0, h, &wall, &above, how, &faces);
    let gable_wall = if how == Build::Raised { above.col } else { wall.col };
    if fabric.timber {
        // the frame shows on what is left of the house as it was built
        let top = if how == Build::Raised { STOREY - 0.0014 } else { h };
        let posts: Vec<f32> = (1..bays).map(|b| w * b as f32 / bays as f32).collect();
        if let Build::Jettied(..) = how {
            timber_frame(&mut k, &Range::rect(-hw, hw, zb, zfm - JETTY), 0, FOOT, STOREY - 0.0014, &posts);
            timber_frame(&mut k, &body, 0, STOREY + 0.0008, h, &posts);
        } else {
            timber_frame(&mut k, &body, 0, FOOT, top, &posts);
        }
        timber_frame(&mut k, &body, 2, FOOT, top, &posts);
        timber_frame(&mut k, &body, 1, FOOT, top, &[d * 0.5]);
        timber_frame(&mut k, &body, 3, FOOT, top, &[d * 0.5]);
    }

    let colour = roof.colour_in(pal, c.seed, &c.street);
    let half_hipped = thatch && r(8) < 0.5 && form == Form::Along && !k.far();
    let hipped = !thatch && c.era >= Era::Georgian && form == Form::Along && r(40) < st.habit(0.26, 10);
    let end = if half_hipped { End::HalfHip } else if hipped { End::Hip } else { End::Gable };
    let verge = if form == Form::EndOn { lie.verge.max(0.0024) } else { lie.verge };
    let spec = RoofSpec { h, rise, eaves: lie.eaves, verge, ends: [end, end], col: colour, gable: gable_wall, thick: lie.thick };
    k.roof(&roofed, &spec);
    if form == Form::Wing {
        // the wing: its gable to the street, its ridge running back to meet the range's front slope
        let rg = Range::rect(wx0, wx1, zfm, zf);
        let mut wf = Faces::new();
        wf.ops[0] = wing_front;
        wf.hide[2] = true;
        // its inner side shows where it stands forward of the range, and runs into the angle
        (wf.dim[1], wf.dim[3]) = if wing_left { (1.0, faces.dim[3]) } else { (faces.dim[1], 1.0) };
        if wing_left {
            wf.dark[1] = (false, true);
        } else {
            wf.dark[3] = (true, false);
        }
        let wy0 = -k.drop(&rg);
        put_up(&mut k, &rg, wy0, h, &wall, &above, if how == Build::Raised { how } else { Build::Whole }, &wf);
        let wing_rise = (wing_w * 0.5 * spec.pitch(&body)).min(rise * 0.9);
        let run = d * 0.5 * wing_rise / rise;
        let over = Range { cx: (wx0 + wx1) * 0.5, cz: zfm + wing_d * 0.5, len: wing_d, dep: wing_w, turn: 3 };
        k.roof(&over, &RoofSpec { h, rise: wing_rise, eaves: lie.eaves, verge: lie.verge.max(0.0024), ends: [End::Gable, End::Valley(run)], col: colour, gable: gable_wall, thick: lie.thick });
    }
    if !k.far() {
        if half && form != Form::EndOn {
            // the windows of the rooms in the roof, each in a small gable standing on the
            // eaves (a house end-on has its garret window in its gable)
            for b in (0..bays).filter(|&b| bays == 2 || b != door_bay) {
                k.gablet(&body, &spec, -hw + bay_at(b), w / bays as f32 * 0.66, 1.0, &wall, true);
            }
        }
        if form == Form::EndOn {
            // a window to the garret, in the gable
            let y = h + rise * 0.14;
            if k.near() {
                k.strip(&body, 0, hw - 0.0034, hw + 0.0034, y, y + 0.0072, 0.0006, wall.frame);
            }
            k.strip(&body, 0, hw - 0.0028, hw + 0.0028, y + 0.0006, y + 0.0066, 0.0012, *pal.glass.window);
        }
    }

    if outshut {
        // a range whose low side is the back
        let rg = Range { cx: (ox0 + ox1) * 0.5, cz: zb - out_depth * 0.5, len: ox1 - ox0, dep: out_depth, turn: 2 };
        let pitch = if form == Form::EndOn { 0.62 } else { spec.pitch(&body) };
        let (high, low) = if n == 1 && !half { (h, (h - out_depth * pitch * 0.8).max(0.0105)) } else { (STOREY + 0.0045, 0.0125) };
        let ops = [Opening { u: (ox1 - ox0) * 0.3, y: FOOT, w: 0.0046, h: (low - FOOT - 0.0036).min(0.012), kind: back_door(pal) }];
        k.lean_to(&rg, low, high, &wall, shade(colour, 0.82), lie.eaves * 0.8, Side::with(&ops), [true, true]);
    }

    // which end the chimney is at, where the garden ends behind, and whether a tree stands in it.
    // The tree is drawn at every level, so that the island's trees are the same wherever the camera is
    let side = if u < 0.5 { 1.0 } else { -1.0 };
    let tree = r(15) < 0.28;
    if tree {
        garden_tree(green, &k, site, -side * (pw * 0.5 - 0.009), rear + 0.007, 0.24 + 0.08 * r(16), c.seed, pal);
    }

    if !k.far() {
        let zm = (zfm + zb) * 0.5 + (r(19) - 0.5) * 0.004;
        let top = h + rise + 0.0055;
        let at = match form {
            Form::EndOn => {
                // built against a side wall where the plot has room, or astride the ridge at the back
                if age >= 0.5 && hw + 0.0042 <= pw * 0.5 - 0.0003 {
                    let (x0, x1) = (side * (hw - 0.0004), side * (hw + 0.0038));
                    let (x0, x1) = (x0.min(x1), x0.max(x1));
                    let zs = zb + d * 0.36;
                    k.boxy(x0, x1, zs - 0.0044, zs + 0.0044, y0, h * 0.86, fabric.stack, Some(shade(fabric.stack, 1.06)));
                    k.stack((x0 + x1) * 0.5, zs, 0.0042, 0.0054, h * 0.86 - 0.001, h + rise * 0.6 + 0.0055, fabric.stack, 1)
                } else {
                    k.stack(0.0, zb + 0.0046, 0.0052, 0.0056, h + rise * 0.45, top, fabric.stack, 1)
                }
            }
            Form::Wing => {
                // where the wing's ridge meets the range's, and one at the range's far end on the bigger
                let x = if wing_left { wx1 + 0.0036 } else { wx0 - 0.0036 };
                let at = k.stack(x, (zfm + zb) * 0.5, 0.0058, 0.0054, h + rise * 0.6, top + 0.001, fabric.stack, 2);
                if n >= 2 || bays == 4 {
                    let far_end = if wing_left { hw - 0.0046 } else { -hw + 0.0046 };
                    k.stack(far_end, zm, 0.0056, 0.0052, h + rise * 0.45, top, fabric.stack, 1);
                }
                at
            }
            Form::Along => {
                // on the ridge in the middle, between two rooms, as the oldest plans have it and
                // every long house; at both ends of the wider houses of the later ages; or at one end
                let from = h + rise * if hipped { 0.12 } else { 0.45 };
                let middle = bays == 4 || r(47) < st.habit(if c.era == Era::Old { 0.5 } else if c.era == Era::Georgian { 0.18 } else { 0.0 }, 11);
                let outside = age >= 0.5 && !half_hipped && !hipped && hw + 0.0042 <= pw * 0.5 - 0.0003;
                if middle {
                    let x = if bays == 3 { side * w / 6.0 } else { 0.0 };
                    k.stack(x, (zfm + zb) * 0.5, 0.0062, 0.0056, h + rise * 0.6, top + 0.001, fabric.stack, 2)
                } else if outside {
                    let (x0, x1) = (side * (hw - 0.0004), side * (hw + 0.0038));
                    let (x0, x1) = (x0.min(x1), x0.max(x1));
                    k.boxy(x0, x1, zm - 0.0044, zm + 0.0044, y0, h * 0.86, fabric.stack, Some(shade(fabric.stack, 1.06)));
                    k.stack((x0 + x1) * 0.5, zm, 0.0042, 0.0054, h * 0.86 - 0.001, top, fabric.stack, 1)
                } else {
                    let at = k.stack(side * (hw - 0.0046), zm, 0.0056, 0.0052, from, top, fabric.stack, if bays >= 3 { 2 } else { 1 });
                    if bays >= 3 && c.era >= Era::Georgian && r(47) > 1.0 - st.habit(0.38, 12) {
                        k.stack(-side * (hw - 0.0046), zm, 0.0056, 0.0052, from, top, fabric.stack, 1);
                    }
                    at
                }
            }
        };
        if r(11) < 0.22 {
            hearth_smoke(smoke, at, c.seed, pal);
        }

        // the garden: the plot and the ground behind it
        let (gx0, gx1) = (-pw * 0.5 + 0.0008, pw * 0.5 - 0.0008);
        k.patch(gx0, gx1, rear, zf + 0.001, 0.0034, *pal.ground.garden);
        if k.near() {
            let door_x = -hw + bay_at(door_bay);
            k.patch(door_x - 0.002, door_x + 0.002, zfm + 0.002, pd * 0.5 + SET_BACK - 0.0005, 0.0042, *pal.ground.garden_path);
            // beds dug across the back of it, a row of something green down each
            let back_of_house = zb - if outshut { out_depth + lie.eaves } else { 0.0 };
            let room = back_of_house - rear - 0.004;
            let beds = ((room / 0.0042).floor().max(0.0) as usize).min(3);
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

    /// A street of a family: the `n`th of them.
    fn street(n: u32) -> Street {
        Street::of(Seed::of(700 + n))
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

    /// The widths of plot a cottage is tried on: a back lane's, the usual, and the wide ones of the ways and the furthest lanes.
    const WIDTHS: [f32; 4] = [crate::houses::BACK_PLOT_WIDTH, PLOT.0, 0.06, 0.066];

    #[test]
    fn it_stays_on_its_plot_and_grows_with_its_storeys() {
        for n in 0..40 {
            let seed = Seed::of(n);
            for era in Era::ALL {
                for detail in Detail::ALL {
                    for width in WIDTHS {
                        // told nothing of its street, and in three streets
                        for st in [Street::ALONE, street(n), street(n + 1), street(n + 2)] {
                            let mut last = 0.0;
                            for storeys in 1..=3 {
                                let raised = (storeys > 1 && n % 3 > 0).then_some(Era::Late);
                                let m = built(&Cottage { era, detail, upland: n % 5 == 0, street: st, raised, ..Cottage::new(width, PLOT.1, storeys, Roof::of(1750, 1850, false, false, seed), seed) });
                                let (lo, hi) = m.solid.bounds().unwrap();
                                // the eaves of the widest, turned a little, overhang the plot by a hair, no more
                                assert!(lo.x > -width * 0.5 - 0.004 && hi.x < width * 0.5 + 0.004, "seed {n} {era:?} on {width}: from {} to {}", lo.x, hi.x);
                                // behind, it keeps to its plot and the room behind that; before, to its path
                                assert!(lo.z > -PLOT.1 * 0.5 - BACK_ROOM - 0.003 && hi.z < PLOT.1 * 0.5 + SET_BACK + 0.003, "seed {n} {era:?} on {width}: from {} to {}", lo.z, hi.z);
                                assert!(hi.y > last, "seed {n}: no taller at {storeys} storeys");
                                last = hi.y;
                            }
                        }
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
            for (width, st) in [(PLOT.0, Street::ALONE), (crate::houses::BACK_PLOT_WIDTH, street(n / 20)), (PLOT.0, street(n / 20)), (0.062, street(n / 20))] {
                let seed = Seed::of(n);
                let m = built(&Cottage { street: st, era: Era::ALL[n as usize % 4], ..Cottage::new(width, PLOT.1, 1, Roof::of(1750, 1850, false, false, seed), seed) });
                assert!(m.solid.col.contains(&*pal.smoke.stack_top), "seed {n} on {width} has no chimney");
                smoking += (st == Street::ALONE && !m.smoke.is_empty()) as usize;
            }
        }
        assert!((50..130).contains(&smoking), "{smoking} of 400 smoke");
    }

    #[test]
    fn each_level_draws_more_than_the_last_and_far_is_a_shell_and_a_roof() {
        let pal = Palette::standard();
        for n in 0..60 {
            let seed = Seed::of(n);
            for (width, st) in [(PLOT.0, Street::ALONE), (crate::houses::BACK_PLOT_WIDTH, street(n / 6)), (PLOT.0, street(n / 6)), (0.062, street(n / 6))] {
                let c = Cottage { street: st, era: Era::of(1700 + 5 * n as i32), ..Cottage::new(width, PLOT.1, 2, Roof::of(1750, 1850, false, false, seed), seed) };
                let at = |detail: Detail| built(&Cottage { detail, ..c });
                let (far, play, near) = (at(Detail::Far), at(Detail::Play), at(Detail::Near));
                // four walls, two slopes and two gables or hips; with a cross-wing three walls
                // more, its two slopes, their valleys and its gable
                let shell = 4 * 2 + 2 * 2 + 2;
                assert_eq!(far.solid.triangles(), if c.form() == Form::Wing { shell + 3 * 2 + 2 * 2 + 2 + 1 } else { shell }, "seed {n} on {width}");
                assert!(far.smoke.is_empty());
                // the tree in its garden, if it has one, is the same tree at every level
                assert!(far.green == play.green && play.green == near.green, "seed {n}");
                assert!(!far.solid.col.contains(&*pal.glass.window));
                assert!(play.solid.triangles() > far.solid.triangles() * 2 && play.solid.triangles() < 110, "seed {n} on {width}: {} at play", play.solid.triangles());
                assert!(play.solid.col.contains(&*pal.glass.window) && play.solid.col.contains(&*pal.ground.garden));
                assert!(near.solid.triangles() > play.solid.triangles() * 2, "seed {n}");
                // the same house at every level: but for its chimney, as tall
                let (f, p) = (far.solid.bounds().unwrap(), near.solid.bounds().unwrap());
                assert!(p.1.y > f.1.y && p.1.y - f.1.y < 0.012);
            }
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
        for (width, st) in [(PLOT.0, Street::ALONE), (crate::houses::BACK_PLOT_WIDTH, street(3)), (0.062, street(4))] {
            let mut m = Model::default();
            let c = Cottage { street: st, ..Cottage::new(width, PLOT.1, 2, Roof::of(1750, 1850, false, false, seed), seed) };
            cottage(&mut m.solid, &mut m.smoke, &mut m.green, &Site::on(Vec3::new(0.0, 0.1, 0.0), 0.0, &slope), &c, &pal);
            assert!(m.is_sound());
            // walls reach the ground at the lowest corner, and nothing is buried deeper than the fall across the plot
            let low = m.solid.pos.iter().map(|p| p[1] - slope(p[0], p[2])).fold(f32::MAX, f32::min);
            assert!(low < 0.0 && low > -0.02, "{low}");
        }
    }

    #[test]
    fn the_street_and_the_plot_settle_how_it_stands() {
        let roof = Roof { cover: Cover::Tile, weathering: 1 };
        let on = |width: f32, n: u32, era: Era, st: Street| Cottage { era, street: st, ..Cottage::new(width, PLOT.1, 1, roof, Seed::of(n)) };
        let count = |width: f32, era: Era, st: Street, form: Form| (0..200).filter(|&n| on(width, n, era, st).form() == form).count();
        // told nothing of its street, every cottage stands along it, whatever its plot
        for width in WIDTHS {
            assert_eq!(count(width, Era::Old, Street::ALONE, Form::Along), 200);
        }
        // over many streets: a wing on four in ten of the wide plots and on none of the others;
        // end-on, one in seven of those built before the brick years on the narrower plots, and
        // none built after; and either is one street's habit, hardly seen in the next
        let (mut wings, mut end_on, mut streets_with, mut streets_of) = (0, 0, 0, 0);
        for s in 0..60 {
            let st = street(s);
            let (w, e) = (count(0.06, Era::Brick, st, Form::Wing), count(PLOT.0, Era::Georgian, st, Form::EndOn));
            wings += w;
            end_on += e;
            streets_with += (e > 0) as usize;
            streets_of += (e > 60) as usize;
            assert_eq!(count(PLOT.0, Era::Georgian, st, Form::Wing) + count(0.06, Era::Old, st, Form::EndOn) + count(PLOT.0, Era::Brick, st, Form::EndOn), 0);
        }
        assert!((60 * 60..60 * 100).contains(&wings), "{wings} of 12000 with a wing");
        assert!((60 * 20..60 * 50).contains(&end_on), "{end_on} of 12000 end-on");
        assert!((15..45).contains(&streets_with) && streets_of >= 3, "{streets_with} streets of 60 have a cottage end-on, {streets_of} have many");
        for n in 0..200 {
            let size = |c: &Cottage| {
                let (lo, hi) = built(&Cottage { detail: Detail::Far, ..*c }).solid.bounds().unwrap();
                (hi.x - lo.x, hi.z - lo.z)
            };
            // end-on it is narrower than it is deep, verges and all, by an eighth at the least,
            // and along the street the other way about
            let st = street(n / 10);
            let c = on(PLOT.0, n, Era::Georgian, st);
            let (w, d) = size(&c);
            match c.form() {
                Form::EndOn => assert!(w * 1.125 < d, "seed {n}: {w} wide and {d} deep end-on"),
                _ => assert!(w > d - 0.003, "seed {n}: {w} wide and {d} deep along the street"),
            }
            // a wide plot's cottage is longer than the usual plot's
            if c.form() == Form::Along {
                assert!(size(&on(0.066, n, Era::Late, st)).0 > w + 0.006, "seed {n}");
            }
        }
    }

    #[test]
    fn a_street_s_habits_show_in_its_roofs() {
        // over many streets a quarter of the tiled cottages of the later ages are hipped and a
        // fifth of those of one storey have rooms in the roof; in a street it is most or none
        let roof = Roof { cover: Cover::Tile, weathering: 1 };
        let shell = 4 * 2 + 2 * 2 + 2;
        let (mut taller, mut streets_none, mut streets_most) = (0, 0, 0);
        for s in 0..60 {
            let mut here = 0;
            for n in 0..40 {
                let c = Cottage { era: Era::Late, detail: Detail::Far, ..Cottage::new(PLOT.0, PLOT.1, 1, roof, Seed::of(n)) };
                let (alone, told) = (built(&c), built(&Cottage { street: street(s), ..c }));
                // whatever its roof, from far off it is a shell: as many triangles
                assert_eq!(told.solid.triangles(), shell);
                // with rooms in its roof it stands higher at the eaves, and so at the ridge
                here += (told.solid.bounds().unwrap().1.y > alone.solid.bounds().unwrap().1.y + STOREY * 0.3) as usize;
            }
            taller += here;
            streets_none += (here == 0) as usize;
            streets_most += (here > 20) as usize;
        }
        assert!((60 * 4..60 * 12).contains(&taller), "{taller} of 2400 have rooms in the roof");
        assert!(streets_none >= 12 && streets_most >= 3, "{streets_none} streets of 60 with none, {streets_most} with most");
    }

    #[test]
    fn a_house_raised_in_a_later_age_shows_the_later_work_above_the_old() {
        let pal = Palette::standard();
        let bricks = [*pal.brick.red, *pal.brick.terrace_brown, *pal.brick.terrace_orange, *pal.brick.terrace_sooty];
        let of_brick = |m: &Model| m.solid.col.iter().any(|c| bricks.contains(c));
        let mut two_tone = 0;
        for n in 0..200 {
            let seed = Seed::of(n);
            let c = Cottage { era: Era::Georgian, detail: Detail::Play, ..Cottage::new(PLOT.0, PLOT.1, 2, Roof::of(1760, 1880, false, false, seed), seed) };
            let (as_built, raised_then, raised_later) = (built(&c), built(&Cottage { raised: Some(Era::Georgian), ..c }), built(&Cottage { raised: Some(Era::Late), ..c }));
            // raised in its own age it is one build; a washed house of the eighteenth century has no brick in it
            assert_eq!(as_built, raised_then, "seed {n}");
            assert!(!of_brick(&as_built));
            two_tone += of_brick(&raised_later) as usize;
            // it is as tall either way
            assert_eq!(as_built.solid.bounds().unwrap().1.y, raised_later.solid.bounds().unwrap().1.y);
        }
        // six in ten of the later age's cottages are of brick, and so are the storeys it added
        assert!((90..150).contains(&two_tone), "{two_tone} of 200 raised in brick");
        // a framed house raised later keeps its frame below and has none above, and no jetty
        let old = (0..400).map(Seed::of).find(|&s| Fabric::of(&pal, Era::Old, false, 0.0, 0.6, s, &Street::ALONE).timber).unwrap();
        let c = Cottage { era: Era::Old, ..Cottage::new(PLOT.0, PLOT.1, 2, Roof::of(1700, 1880, false, false, old), old) };
        let oak = *pal.timber.house_frame;
        let frame_top = |m: &Model| m.solid.pos.iter().zip(&m.solid.col).filter(|(_, col)| col[0] <= oak[0] + 1e-4 && (col[0] / oak[0] - col[1] / oak[1]).abs() < 1e-3 && (col[0] / oak[0] - col[2] / oak[2]).abs() < 1e-3).map(|(p, _)| p[1]).fold(0.0f32, f32::max);
        let (framed, raised) = (built(&c), built(&Cottage { raised: Some(Era::Brick), ..c }));
        assert!(frame_top(&framed) > STOREY * 1.8, "{}", frame_top(&framed));
        assert!(frame_top(&raised) > STOREY * 0.8 && frame_top(&raised) < STOREY, "{}", frame_top(&raised));
    }
}
