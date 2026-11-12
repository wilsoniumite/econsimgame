//! The parts a house is put together from.
//!
//! A house is a few ranges (a main range, perhaps a rear wing or an outshut)
//! with walls, a roof fitted to each, chimney stacks, and what stands behind
//! it. Every kind of house is a few lines that place these parts on its plot;
//! the parts know how much of themselves to draw (`Detail`), so a kind says
//! what the house is and the level follows by itself.
//!
//! Places are in the house's own frame: x along its front, z out toward the
//! street, the origin where the site says, heights above the ground there.
//! Every corner goes through `Kit::pt`, which is where an old house leans and
//! its ridge sags, so walls, windows and roof go crooked together.

use glam::{Quat, Vec3};

use crate::colour::{shade, Rgba};
use crate::mesh::MeshBuf;
use crate::palette::Palette;

use crate::site::Site;

/// How much of a house is drawn. The viewer chooses by how far off the
/// camera is: from across the island a house is its shell and its roof; at
/// the distance the game is played from it has its openings, its chimneys,
/// its back wing and the colour of its garden; close to, everything.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Detail {
    Far,
    Play,
    #[default]
    Near,
}

impl Detail {
    pub const ALL: [Detail; 3] = [Detail::Far, Detail::Play, Detail::Near];

    pub fn label(self) -> &'static str {
        match self {
            Detail::Far => "far",
            Detail::Play => "play",
            Detail::Near => "near",
        }
    }
}

/// When a house was built, in the steps its style takes. The year a plot was
/// first built on is fixed, so a house keeps its era for good.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Era {
    /// before 1720: timber frames and wash, casements, crooked with age
    Old,
    /// to 1815: washed or stone walls, sash windows, a hood or a fanlight over the door
    Georgian,
    /// to 1860: brick and blue slate
    Brick,
    /// after: brick with stone dressings, bay windows, bargeboards
    Late,
}

impl Era {
    pub const ALL: [Era; 4] = [Era::Old, Era::Georgian, Era::Brick, Era::Late];

    /// The era of a house built in a year.
    pub fn of(built: i32) -> Era {
        match built {
            i32::MIN..=1719 => Era::Old,
            1720..=1814 => Era::Georgian,
            1815..=1859 => Era::Brick,
            _ => Era::Late,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Era::Old => "before 1720",
            Era::Georgian => "1720 to 1815",
            Era::Brick => "1815 to 1860",
            Era::Late => "after 1860",
        }
    }
}

/// The height of one storey of a house. (A works' storey is a little more, and the towns' other
/// buildings keep `parts::STOREY`: houses are drawn low, so that a street of three storeys is
/// roofs and not towers.)
pub(super) const STOREY: f32 = 0.0205;
/// How high the band at the foot of a wall is: a plinth, and the floor's height above the ground.
pub(super) const FOOT: f32 = 0.0026;
/// How deep the band of shadow under the eaves is.
const EAVE_BAND: f32 = 0.003;
/// How far a window sits back in its wall, and a doorway.
const REVEAL: f32 = 0.0013;
/// How far one flat thing must stand off another it lies on to be drawn cleanly from the game's distances.
const LAYER: f32 = 0.0006;
/// How far a flat window or door stands off the wall, where it is not let into it.
const PROUD: f32 = 0.0006;
/// The soft light faked on the four sides of a building, as `MeshBuf::block` has it: front, right, back, left.
const SIDE_SHADE: [f32; 4] = [1.0, 0.93, 0.86, 0.95];

/// A rectangle of the plan with a way round: `a` runs along it (along its
/// ridge, if it has a roof) and `c` across. Turned by quarter turns: at 0
/// `a` is the house's x and `c` its z (the +c side is the front); at 1 `a`
/// runs to the back and +c is to the right; at 2 `a` runs to the left and
/// +c is the back; at 3 `a` runs to the front and +c is to the left.
#[derive(Clone, Copy, Debug)]
pub(super) struct Range {
    pub cx: f32,
    pub cz: f32,
    pub len: f32,
    pub dep: f32,
    pub turn: u8,
}

/// One side of a range, seen from outside: where it begins, the way it runs, and how long it is.
#[derive(Clone, Copy, Debug)]
pub(super) struct Face {
    ox: f32,
    oz: f32,
    ux: f32,
    uz: f32,
    pub len: f32,
}

impl Range {
    /// The rectangle from x0 to x1 and z0 to z1, its length along x.
    pub fn rect(x0: f32, x1: f32, z0: f32, z1: f32) -> Range {
        Range { cx: (x0 + x1) * 0.5, cz: (z0 + z1) * 0.5, len: x1 - x0, dep: z1 - z0, turn: 0 }
    }

    /// The same rectangle, its length running from front to back (`a` toward the back).
    pub fn rect_across(x0: f32, x1: f32, z0: f32, z1: f32) -> Range {
        Range { cx: (x0 + x1) * 0.5, cz: (z0 + z1) * 0.5, len: z1 - z0, dep: x1 - x0, turn: 1 }
    }

    #[cfg(test)]
    pub fn turned(self, turn: u8) -> Range {
        Range { turn, ..self }
    }

    /// The ways `a` and `c` run in the house's frame, as (x, z).
    fn axes(&self) -> ((f32, f32), (f32, f32)) {
        match self.turn % 4 {
            0 => ((1.0, 0.0), (0.0, 1.0)),
            1 => ((0.0, -1.0), (1.0, 0.0)),
            2 => ((-1.0, 0.0), (0.0, -1.0)),
            _ => ((0.0, 1.0), (-1.0, 0.0)),
        }
    }

    /// Where a point of the range is in the house's frame.
    pub fn at(&self, a: f32, c: f32) -> (f32, f32) {
        let (da, dc) = self.axes();
        (self.cx + da.0 * a + dc.0 * c, self.cz + da.1 * a + dc.1 * c)
    }

    /// A side of the range: 0 is the +c side, 1 the +a end, 2 the -c side, 3 the -a end.
    pub fn face(&self, i: usize) -> Face {
        let (hl, hd) = (self.len * 0.5, self.dep * 0.5);
        let (da, dc) = self.axes();
        let (o, u, len) = match i % 4 {
            0 => ((-hl, hd), da, self.len),
            1 => ((hl, hd), (-dc.0, -dc.1), self.dep),
            2 => ((hl, -hd), (-da.0, -da.1), self.len),
            _ => ((-hl, -hd), dc, self.dep),
        };
        let (ox, oz) = self.at(o.0, o.1);
        Face { ox, oz, ux: u.0, uz: u.1, len }
    }
}

/// What a wall is made of.
#[derive(Clone, Copy, Debug)]
pub(super) struct Wall {
    pub col: Rgba,
    /// the band at its foot, where it is not the wall's own
    pub plinth: Option<Rgba>,
    /// its window frames
    pub frame: Rgba,
    /// its sills, lintels and steps
    pub sill: Rgba,
}

/// What is over a door.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Over {
    Nothing,
    /// a little pent roof on brackets
    Hood,
    /// a light over the door, within the doorway
    Fanlight,
}

/// What an opening in a wall is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Open {
    /// a window of two sashes, one over the other
    Sash,
    /// a window of two lights side by side
    Casement,
    Door { paint: Rgba, over: Over },
    /// a shop's window over its stall riser, the door at one end and the name board above
    Shop { paint: Rgba, awning: Option<Rgba> },
    /// a way through for a cart
    Arch,
}

/// An opening in a wall: its middle along the wall, its foot, how wide and how tall.
#[derive(Clone, Copy, Debug)]
pub(super) struct Opening {
    pub u: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub kind: Open,
}

/// How one side of a range is drawn.
#[derive(Clone, Copy)]
pub(super) struct Side<'o> {
    pub show: bool,
    /// a wall that little light reaches (close against a neighbour) is drawn this much darker
    pub k: f32,
    pub ops: &'o [Opening],
    /// the wall runs into a corner at its beginning or its end, and is darker there
    pub dark: (bool, bool),
}

impl<'o> Side<'o> {
    /// A side not drawn: a party wall, or one another range stands against.
    pub const NONE: Side<'static> = Side { show: false, k: 1.0, ops: &[], dark: (false, false) };
    /// A blank wall.
    pub const PLAIN: Side<'static> = Side { show: true, k: 1.0, ops: &[], dark: (false, false) };

    pub fn with(ops: &'o [Opening]) -> Side<'o> {
        Side { show: true, k: 1.0, ops, dark: (false, false) }
    }

    pub fn dim(self, k: f32) -> Side<'o> {
        Side { k, ..self }
    }

    pub fn dark(self, begin: bool, end: bool) -> Side<'o> {
        Side { dark: (begin, end), ..self }
    }
}

/// How a roof ends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum End {
    /// the wall runs up to the ridge and the roof oversails it
    Gable,
    /// the roof slopes back from the end as it does from the sides
    Hip,
    /// a gable part of the way up, hipped above
    HalfHip,
    /// it stops flush against a neighbour or a taller wall
    Party,
    /// it runs on into another roof at the same eaves: the ridge goes this
    /// much further, and the slopes meet the other roof in valleys
    Valley(f32),
}

/// A pitched roof on a range.
#[derive(Clone, Copy, Debug)]
pub(super) struct RoofSpec {
    /// the height of the eaves (the top of the walls), and of the ridge above them
    pub h: f32,
    pub rise: f32,
    /// how far it oversails the walls at the eaves and at the gable ends
    pub eaves: f32,
    pub verge: f32,
    /// how it ends toward +a and toward -a
    pub ends: [End; 2],
    pub col: Rgba,
    /// the wall in its gables
    pub gable: Rgba,
    /// how thick its edge is drawn close to: thatch is deep, slate thin
    pub thick: f32,
}

impl RoofSpec {
    /// The slope: how much the roof rises for each unit across.
    pub fn pitch(&self, rg: &Range) -> f32 {
        self.rise / (rg.dep * 0.5)
    }
}

/// What a house builds with: the buffer, the site, the level of detail, and
/// the frame its corners are worked out in.
pub(super) struct Kit<'a, 's> {
    pub solid: &'a mut MeshBuf,
    pub pal: &'a Palette,
    pub detail: Detail,
    site: &'a Site<'s>,
    o: Vec3,
    q: Quat,
    yaw: f32,
    /// how far out of plumb the walls go for each unit of height: along the front, and toward it
    lean: (f32, f32),
    /// how far the ridge dips in the middle, the half length it dips over, and the heights of the eaves and the ridge
    sag: (f32, f32, f32, f32),
}

impl<'a, 's> Kit<'a, 's> {
    /// A kit for a house on `site`, turned `askew` from the way the site faces.
    pub fn new(solid: &'a mut MeshBuf, site: &'a Site<'s>, pal: &'a Palette, detail: Detail, askew: f32) -> Self {
        let yaw = site.yaw + askew;
        Kit { solid, pal, detail, site, o: site.origin, q: Quat::from_rotation_y(yaw), yaw, lean: (0.0, 0.0), sag: (0.0, 1.0, 0.0, 1.0) }
    }

    /// An old house: its walls lean, and its ridge dips by `dip` in the
    /// middle of a roof `half` long each way from the house's middle, whose
    /// eaves are at `eaves` and ridge at `ridge`.
    pub fn crooked(&mut self, lean: (f32, f32), dip: f32, half: f32, eaves: f32, ridge: f32) {
        self.lean = lean;
        self.sag = (dip, half.max(1e-4), eaves, ridge.max(eaves + 1e-4));
    }

    pub fn near(&self) -> bool {
        self.detail == Detail::Near
    }

    pub fn far(&self) -> bool {
        self.detail == Detail::Far
    }

    /// A point of the house's frame, on the map.
    pub fn pt(&self, x: f32, y: f32, z: f32) -> Vec3 {
        let (dip, half, eaves, ridge) = self.sag;
        let up = y.max(0.0);
        let mut y = y;
        if dip > 0.0 && y > eaves {
            let mid = (1.0 - (x / half) * (x / half)).max(0.0);
            y -= dip * mid * ((y - eaves) / (ridge - eaves)).min(1.0);
        }
        self.o + self.q * Vec3::new(x + self.lean.0 * up, y, z + self.lean.1 * up)
    }

    /// A way in the house's frame, on the map.
    pub fn dir(&self, x: f32, y: f32, z: f32) -> Vec3 {
        self.q * Vec3::new(x, y, z)
    }

    /// How high the land at a place in the house's frame lies above the land at its origin.
    pub fn ground(&self, x: f32, z: f32) -> f32 {
        if self.site.is_level() {
            return 0.0;
        }
        let p = self.o + self.q * Vec3::new(x, 0.0, z);
        self.site.height(p.x, p.z) - self.o.y
    }

    /// How far walls round a range must reach below the origin's height to
    /// meet the ground at its lowest corner (a plinth that takes up the slope).
    pub fn drop(&self, rg: &Range) -> f32 {
        const MARGIN: f32 = 0.003;
        if self.site.is_level() {
            return MARGIN;
        }
        let (hl, hd) = (rg.len * 0.5, rg.dep * 0.5);
        let low = [(-hl, -hd), (hl, -hd), (hl, hd), (-hl, hd)].iter().map(|&(a, c)| rg.at(a, c)).map(|(x, z)| self.ground(x, z)).fold(0.0f32, f32::min);
        MARGIN - low
    }

    /// A point of a range, on the map.
    pub fn rp(&self, rg: &Range, a: f32, y: f32, c: f32) -> Vec3 {
        let (x, z) = rg.at(a, c);
        self.pt(x, y, z)
    }

    fn fp(&self, f: &Face, u: f32, y: f32, out: f32) -> Vec3 {
        // the way out of a wall is a quarter turn from the way along it
        self.pt(f.ox + f.ux * u - f.uz * out, y, f.oz + f.uz * u + f.ux * out)
    }

    fn face_out(&self, f: &Face) -> Vec3 {
        self.dir(-f.uz, 0.0, f.ux)
    }

    fn face_along(&self, f: &Face) -> Vec3 {
        self.dir(f.ux, 0.0, f.uz)
    }

    /// A quad of one colour facing `toward`.
    pub fn quad(&mut self, p: [Vec3; 4], col: Rgba, toward: Vec3) {
        self.solid.quad_toward(p, [col; 4], toward);
    }

    /// A panel on a face, `out` off it: from u0 to u1 along and y0 to y1 up, in colours for its foot and its top.
    #[allow(clippy::too_many_arguments)]
    fn panel(&mut self, f: &Face, u0: f32, u1: f32, y0: f32, y1: f32, out: f32, foot: Rgba, top: Rgba) {
        let p = [self.fp(f, u0, y0, out), self.fp(f, u1, y0, out), self.fp(f, u1, y1, out), self.fp(f, u0, y1, out)];
        self.solid.quad_toward(p, [foot, foot, top, top], self.face_out(f));
    }

    /// A panel of one colour on a side of a range (a beam of a timber frame, a band of stone).
    #[allow(clippy::too_many_arguments)]
    pub fn strip(&mut self, rg: &Range, side: usize, u0: f32, u1: f32, y0: f32, y1: f32, out: f32, col: Rgba) {
        let f = rg.face(side);
        let col = shade(col, SIDE_SHADE[(side + rg.turn as usize) % 4]);
        self.panel(&f, u0, u1, y0, y1, out, col, col);
    }

    // -------------------------------------------------------------- walls

    /// The walls of a range from `y0` up to `y1`. With `foot` they stand on
    /// the ground: darker there, and close to with a plinth; with `eaves`
    /// they end under a roof that shades their top.
    #[allow(clippy::too_many_arguments)]
    pub fn walls(&mut self, rg: &Range, y0: f32, y1: f32, w: &Wall, sides: [Side; 4], foot: bool, eaves: bool) {
        for (i, s) in sides.iter().enumerate() {
            if !s.show {
                continue;
            }
            let f = rg.face(i);
            let k = SIDE_SHADE[(i + rg.turn as usize) % 4] * s.k;
            let col = shade(w.col, k);
            // darker toward a corner it runs into
            let reach = f.len.min(0.012);
            let ends = |u: f32| (if s.dark.0 { 0.74 + 0.26 * (u / reach).min(1.0) } else { 1.0 }) * (if s.dark.1 { 0.74 + 0.26 * ((f.len - u) / reach).min(1.0) } else { 1.0 });
            let out = self.face_out(&f);
            let band = |kit: &mut Self, u0: f32, u1: f32, ya: f32, yb: f32, off: f32, c: Rgba, ka: f32, kb: f32| {
                let p = [kit.fp(&f, u0, ya, off), kit.fp(&f, u1, ya, off), kit.fp(&f, u1, yb, off), kit.fp(&f, u0, yb, off)];
                let (e0, e1) = (ends(u0), ends(u1));
                kit.solid.quad_toward(p, [shade(c, ka * e0), shade(c, ka * e1), shade(c, kb * e1), shade(c, kb * e0)], out);
            };
            if !self.near() {
                band(self, 0.0, f.len, y0, y1, 0.0, col, if foot { 0.78 } else { 1.0 }, 1.0);
                if self.detail == Detail::Play {
                    for o in s.ops {
                        self.flat_opening(&f, o);
                    }
                }
                continue;
            }
            let mut lo = y0;
            if foot {
                match w.plinth {
                    Some(p) => band(self, 0.0, f.len, y0, FOOT, 0.0004, shade(p, k), 0.7, 0.96),
                    None => band(self, 0.0, f.len, y0, FOOT, 0.0, col, 0.7, 0.96),
                }
                lo = FOOT;
            }
            let mut hi = y1;
            if eaves {
                hi = y1 - EAVE_BAND;
                band(self, 0.0, f.len, hi, y1, 0.0, col, 1.0, 0.72);
            }
            if s.ops.is_empty() {
                band(self, 0.0, f.len, lo, hi, 0.0, col, 1.0, 1.0);
                continue;
            }
            // the wall between its openings, a storey at a time
            let rows = ((hi / STOREY - 1e-3).ceil() as i32).max(1);
            let row_of = |o: &Opening| (((o.y + o.h * 0.5) / STOREY).floor() as i32).clamp(0, rows - 1);
            for row in 0..rows {
                let (ra, rb) = (if row == 0 { lo } else { row as f32 * STOREY }, if row == rows - 1 { hi } else { (row + 1) as f32 * STOREY });
                let mut here: Vec<&Opening> = s.ops.iter().filter(|o| row_of(o) == row).collect();
                here.sort_by(|p, q| p.u.total_cmp(&q.u));
                let mut at = 0.0;
                for o in here {
                    let (u0, u1) = ((o.u - o.w * 0.5).max(at), (o.u + o.w * 0.5).min(f.len));
                    let (oy0, oy1) = (o.y.max(ra), (o.y + o.h).min(rb));
                    if u1 - u0 < 1e-4 || oy1 - oy0 < 1e-4 {
                        continue;
                    }
                    if u0 - at > 1e-5 {
                        band(self, at, u0, ra, rb, 0.0, col, 1.0, 1.0);
                    }
                    if oy0 - ra > 1e-5 {
                        band(self, u0, u1, ra, oy0, 0.0, col, 1.0, 1.0);
                    }
                    if rb - oy1 > 1e-5 {
                        band(self, u0, u1, oy1, rb, 0.0, col, 1.0, 1.0);
                    }
                    self.opening(&f, u0, u1, oy0, oy1, o, w, k);
                    at = u1;
                }
                if f.len - at > 1e-5 {
                    band(self, at, f.len, ra, rb, 0.0, col, 1.0, 1.0);
                }
            }
        }
    }

    /// An opening at the distance the game is played from: a flat panel on the wall.
    fn flat_opening(&mut self, f: &Face, o: &Opening) {
        let (u0, u1, y0, y1) = (o.u - o.w * 0.5, o.u + o.w * 0.5, o.y, o.y + o.h);
        let glass = *self.pal.glass.window;
        match o.kind {
            Open::Sash | Open::Casement => self.panel(f, u0, u1, y0, y1, PROUD, glass, glass),
            Open::Door { paint, .. } => self.panel(f, u0, u1, y0, y1, PROUD, paint, paint),
            Open::Shop { paint, .. } => {
                self.panel(f, u0, u1, y0, y1, PROUD, glass, glass);
                self.panel(f, u0 - 0.001, u1 + 0.001, y1, y1 + 0.0032, PROUD, paint, paint);
            }
            Open::Arch => {
                let dark = *self.pal.smoke.soot;
                self.panel(f, u0, u1, y0, y1, PROUD, dark, dark);
            }
        }
    }

    /// An opening seen close to, let into the hole the wall has left for it.
    #[allow(clippy::too_many_arguments)]
    fn opening(&mut self, f: &Face, u0: f32, u1: f32, y0: f32, y1: f32, o: &Opening, w: &Wall, k: f32) {
        let (out, along) = (self.face_out(f), self.face_along(f));
        let glass = *self.pal.glass.window;
        // the sides of the hole, and what stands at the back of it
        let jambs = |kit: &mut Self, depth: f32, col: Rgba| {
            kit.quad([kit.fp(f, u0, y0, 0.0), kit.fp(f, u0, y0, -depth), kit.fp(f, u0, y1, -depth), kit.fp(f, u0, y1, 0.0)], shade(col, 0.9), along);
            kit.quad([kit.fp(f, u1, y0, 0.0), kit.fp(f, u1, y0, -depth), kit.fp(f, u1, y1, -depth), kit.fp(f, u1, y1, 0.0)], shade(col, 0.74), -along);
        };
        match o.kind {
            Open::Sash | Open::Casement => {
                // a window is a few pixels across at the nearest: the glass fills the hole, the
                // painted lining of the reveal and the sill are its frame, and one bar of paint
                // across the glass says sash (where the two meet) or casement (the mullion)
                let frame = shade(w.frame, k);
                jambs(self, REVEAL, frame);
                self.quad([self.fp(f, u0 - 0.0003, y0, 0.0005), self.fp(f, u1 + 0.0003, y0, 0.0005), self.fp(f, u1 + 0.0003, y0, -REVEAL), self.fp(f, u0 - 0.0003, y0, -REVEAL)], shade(w.sill, 1.04), Vec3::Y);
                self.panel(f, u0, u1, y0, y1, -REVEAL, glass, shade(glass, 0.86));
                let (d, t) = (-REVEAL + LAYER, 0.0004);
                if o.kind == Open::Sash {
                    let mid = (y0 + y1) * 0.5;
                    self.panel(f, u0, u1, mid - t, mid + t, d, frame, frame);
                } else {
                    let mid = (u0 + u1) * 0.5;
                    self.panel(f, mid - t, mid + t, y0, y1, d, frame, frame);
                }
            }
            Open::Door { paint, over } => {
                jambs(self, REVEAL, shade(w.col, k * 0.86));
                let top = if over == Over::Fanlight { y1 - (y1 - y0) * 0.2 } else { y1 };
                self.panel(f, u0, u1, y0, top, -REVEAL, shade(paint, 0.85), paint);
                if over == Over::Fanlight {
                    let frame = shade(w.frame, k);
                    self.panel(f, u0, u1, top, y1, -REVEAL, frame, frame);
                    self.panel(f, u0 + 0.0005, u1 - 0.0005, top + 0.0004, y1 - 0.0004, -REVEAL + LAYER, glass, glass);
                }
                // the step up to it
                let (s0, s1, so) = (u0 - 0.0007, u1 + 0.0007, 0.0024);
                let stone = w.sill;
                let foot = y0 - FOOT - 0.002;
                self.quad([self.fp(f, s0, y0, 0.0), self.fp(f, s1, y0, 0.0), self.fp(f, s1, y0, so), self.fp(f, s0, y0, so)], shade(stone, 1.05), Vec3::Y);
                self.panel(f, s0, s1, foot, y0, so, shade(stone, 0.8), shade(stone, 0.92));
                self.quad([self.fp(f, s0, foot, 0.0), self.fp(f, s0, foot, so), self.fp(f, s0, y0, so), self.fp(f, s0, y0, 0.0)], shade(stone, 0.86), -along);
                self.quad([self.fp(f, s1, foot, 0.0), self.fp(f, s1, foot, so), self.fp(f, s1, y0, so), self.fp(f, s1, y0, 0.0)], shade(stone, 0.8), along);
                if over == Over::Hood {
                    let (h0, h1, ho) = (u0 - 0.0012, u1 + 0.0012, 0.003);
                    let (ya, yb) = (y1 + 0.0026, y1 + 0.0012);
                    self.quad([self.fp(f, h0, ya, 0.0), self.fp(f, h1, ya, 0.0), self.fp(f, h1, yb, ho), self.fp(f, h0, yb, ho)], stone, Vec3::Y);
                    self.panel(f, h0, h1, yb - 0.0007, yb, ho, shade(stone, 0.78), shade(stone, 0.78));
                }
            }
            Open::Shop { paint, awning } => {
                let depth = REVEAL * 1.3;
                jambs(self, depth, paint);
                let frame = shade(w.frame, k);
                let riser = y0 + (y1 - y0) * 0.24;
                // the door at the left end, the window over its stall riser beside it
                let door = u0 + (u1 - u0) * 0.26;
                self.panel(f, u0, door, y0, y1, -depth, shade(paint, 0.7), shade(paint, 0.8));
                self.panel(f, door, u1, y0, riser, -depth, shade(paint, 0.85), paint);
                self.quad([self.fp(f, door, riser, 0.0), self.fp(f, u1, riser, 0.0), self.fp(f, u1, riser, -depth), self.fp(f, door, riser, -depth)], shade(w.sill, 1.04), Vec3::Y);
                self.panel(f, door, u1, riser, y1, -depth, frame, frame);
                let panes = (((u1 - door) / 0.006).round() as usize).clamp(1, 4);
                let pw = (u1 - door) / panes as f32;
                for i in 0..panes {
                    let a = door + pw * i as f32;
                    self.panel(f, a + 0.0005, a + pw - 0.0005, riser + 0.0005, y1 - 0.0005, -depth + LAYER, glass, shade(glass, 1.15));
                }
                // the name board over it, and its cornice
                let (b0, b1, top) = (u0 - 0.001, u1 + 0.001, y1 + 0.0034);
                self.panel(f, b0, b1, y1, top, 0.0008, shade(paint, 0.92), paint);
                self.quad([self.fp(f, b0, top, 0.0), self.fp(f, b1, top, 0.0), self.fp(f, b1, top, 0.0014), self.fp(f, b0, top, 0.0014)], shade(w.sill, 1.0), Vec3::Y);
                if let Some(cloth) = awning {
                    let (reach, fall) = (0.0055, 0.0032);
                    self.quad([self.fp(f, door, y1, 0.0009), self.fp(f, u1, y1, 0.0009), self.fp(f, u1, y1 - fall, reach), self.fp(f, door, y1 - fall, reach)], cloth, Vec3::Y);
                    self.panel(f, door, u1, y1 - fall - 0.0009, y1 - fall, reach, shade(cloth, 0.8), shade(cloth, 0.8));
                }
            }
            Open::Arch => {
                let depth = 0.005;
                let dark = *self.pal.smoke.soot;
                jambs(self, depth, shade(w.col, k * 0.6));
                self.panel(f, u0, u1, y0, y1, -depth, shade(dark, 1.5), dark);
                // the way under it lies in shadow: seen from above, it is most of what shows of the arch
                self.quad([self.fp(f, u0, y0, 0.0), self.fp(f, u1, y0, 0.0), self.fp(f, u1, y0, -depth), self.fp(f, u0, y0, -depth)], shade(dark, 1.7), Vec3::Y);
                // the haunches of the arch
                let hw = (u1 - u0) * 0.22;
                let col = shade(w.col, k);
                for (corner, toward) in [(u0, 1.0f32), (u1, -1.0)] {
                    let p = [self.fp(f, corner, y1, 0.0), self.fp(f, corner + toward * hw, y1, 0.0), self.fp(f, corner, y1 - hw * 0.9, 0.0)];
                    self.solid.tri_toward(p, [col; 3], out);
                }
                self.panel(f, u0 - 0.0008, u1 + 0.0008, y1, y1 + 0.0012, LAYER, shade(w.sill, 0.95), w.sill);
            }
        }
    }

    // -------------------------------------------------------------- roofs

    /// A flat shape of three to six corners, facing `toward`.
    fn poly(&mut self, p: &[Vec3], col: Rgba, toward: Vec3) {
        match p.len() {
            3 => self.solid.tri_toward([p[0], p[1], p[2]], [col; 3], toward),
            4 => self.quad([p[0], p[1], p[2], p[3]], col, toward),
            5 => {
                self.quad([p[0], p[1], p[2], p[3]], col, toward);
                self.solid.tri_toward([p[0], p[3], p[4]], [col; 3], toward);
            }
            _ => {
                self.quad([p[0], p[1], p[2], p[3]], col, toward);
                self.quad([p[0], p[3], p[4], p[5]], col, toward);
            }
        }
    }

    /// A pitched roof on a range, its ridge along the range's `a`: gabled,
    /// hipped or half-hipped at each end, stopped against a neighbour, or
    /// run on into another roof with valleys.
    pub fn roof(&mut self, rg: &Range, s: &RoofSpec) {
        /// how far up a half-hipped gable its hip begins
        const HALF: f32 = 0.56;
        let (hl, hd) = (rg.len * 0.5, rg.dep * 0.5);
        /// how far the roof lies above the line of the wall's top, so that the two never share a place
        const LIFT: f32 = 0.0005;
        let pitch = s.rise / hd;
        let ce = hd + s.eaves;
        let sagging = self.sag.0 > 0.0 && !self.far() && rg.turn % 2 == 0 && rg.cx.abs() < 1e-4;
        // a ridge that dips flattens the slope under it, which would let the top of the
        // wall through the roof at the eaves: the roof of a crooked house is lifted by as much
        let lift = LIFT + self.sag.0 * s.eaves / ce;
        let (ye, yr) = (s.h - s.eaves * pitch + lift, s.h + s.rise + lift);
        let (cf, yf) = (hd * (1.0 - HALF), s.h + s.rise * HALF + lift);
        let near = self.near();
        // where the eaves and the ridge stop, toward +a and toward -a
        let stop = |e: End, sign: f32| match e {
            End::Gable => (sign * (hl + s.verge), sign * (hl + s.verge)),
            End::Party | End::Valley(_) => (sign * hl, sign * hl),
            End::Hip => (sign * (hl + s.eaves), sign * (hl - hd).max(0.0)),
            End::HalfHip => (sign * (hl + s.verge), sign * (hl - hd * (1.0 - HALF)).max(0.0)),
        };
        let ((ep, rp), (en, rn)) = (stop(s.ends[0], 1.0), stop(s.ends[1], -1.0));
        let (da, dc) = rg.axes();
        let (along, across) = (self.dir(da.0, 0.0, da.1), self.dir(dc.0, 0.0, dc.1));
        let edge = shade(s.col, 0.66);
        for (side, col) in [(1.0f32, s.col), (-1.0, shade(s.col, 0.82))] {
            // a slope is one flat shape, or two where the ridge sags between its ends
            let halves: &[(bool, bool)] = if sagging { &[(false, true), (true, false)] } else { &[(true, true)] };
            for &(to_p, to_n) in halves {
                let mut p = [Vec3::ZERO; 6];
                let mut n = 0;
                let mut push = |v: Vec3| {
                    p[n] = v;
                    n += 1;
                };
                push(self.rp(rg, if to_n { en } else { 0.0 }, ye, side * ce));
                push(self.rp(rg, if to_p { ep } else { 0.0 }, ye, side * ce));
                if to_p && s.ends[0] == End::HalfHip {
                    push(self.rp(rg, ep, yf, side * cf));
                }
                push(self.rp(rg, if to_p { rp } else { 0.0 }, yr, 0.0));
                push(self.rp(rg, if to_n { rn } else { 0.0 }, yr, 0.0));
                if to_n && s.ends[1] == End::HalfHip {
                    push(self.rp(rg, en, yf, side * cf));
                }
                self.poly(&p[..n], col, Vec3::Y);
            }
            for (e, sign) in [(s.ends[0], 1.0f32), (s.ends[1], -1.0)] {
                if let End::Valley(run) = e {
                    let v = [self.rp(rg, sign * hl, s.h + lift, side * hd), self.rp(rg, sign * hl, yr, 0.0), self.rp(rg, sign * (hl + run), yr, 0.0)];
                    self.solid.tri_toward(v, [col; 3], Vec3::Y);
                }
            }
            if near && s.thick > 0.0 {
                let v = [self.rp(rg, en, ye, side * ce), self.rp(rg, ep, ye, side * ce), self.rp(rg, ep, ye - s.thick, side * ce), self.rp(rg, en, ye - s.thick, side * ce)];
                self.quad(v, edge, across * side);
            }
        }
        for (e, sign, at) in [(s.ends[0], 1.0f32, ep), (s.ends[1], -1.0, en)] {
            let out = along * sign;
            let k = if sign > 0.0 { SIDE_SHADE[(1 + rg.turn as usize) % 4] } else { SIDE_SHADE[(3 + rg.turn as usize) % 4] };
            let wall = shade(s.gable, k);
            let peak = shade(wall, if near { 0.8 } else { 1.0 });
            match e {
                End::Gable => {
                    let v = [self.rp(rg, sign * hl, s.h, hd), self.rp(rg, sign * hl, s.h, -hd), self.rp(rg, sign * hl, yr - 0.0006, 0.0)];
                    self.solid.tri_toward(v, [wall, wall, peak], out);
                    if near && s.thick > 0.0 {
                        for side in [1.0f32, -1.0] {
                            let v = [self.rp(rg, at, ye, side * ce), self.rp(rg, at, yr, 0.0), self.rp(rg, at, yr - s.thick, 0.0), self.rp(rg, at, ye - s.thick, side * ce)];
                            self.quad(v, edge, out);
                        }
                    }
                }
                End::HalfHip => {
                    let v = [self.rp(rg, sign * hl, s.h, hd), self.rp(rg, sign * hl, s.h, -hd), self.rp(rg, sign * hl, yf, -cf), self.rp(rg, sign * hl, yf, cf)];
                    self.solid.quad_toward(v, [wall, wall, peak, peak], out);
                    let ridge = if sign > 0.0 { rp } else { rn };
                    let v = [self.rp(rg, at, yf, cf), self.rp(rg, at, yf, -cf), self.rp(rg, ridge, yr, 0.0)];
                    self.solid.tri_toward(v, [shade(s.col, 0.9); 3], Vec3::Y);
                    if near && s.thick > 0.0 {
                        for side in [1.0f32, -1.0] {
                            let v = [self.rp(rg, at, ye, side * ce), self.rp(rg, at, yf, side * cf), self.rp(rg, at, yf - s.thick, side * cf), self.rp(rg, at, ye - s.thick, side * ce)];
                            self.quad(v, edge, out);
                        }
                        let v = [self.rp(rg, at, yf, cf), self.rp(rg, at, yf, -cf), self.rp(rg, at, yf - s.thick, -cf), self.rp(rg, at, yf - s.thick, cf)];
                        self.quad(v, edge, out);
                    }
                }
                End::Hip => {
                    let ridge = if sign > 0.0 { rp } else { rn };
                    let v = [self.rp(rg, at, ye, ce), self.rp(rg, at, ye, -ce), self.rp(rg, ridge, yr, 0.0)];
                    self.solid.tri_toward(v, [shade(s.col, if sign > 0.0 { 0.92 } else { 0.88 }); 3], Vec3::Y);
                    if near && s.thick > 0.0 {
                        let v = [self.rp(rg, at, ye, ce), self.rp(rg, at, ye, -ce), self.rp(rg, at, ye - s.thick, -ce), self.rp(rg, at, ye - s.thick, ce)];
                        self.quad(v, edge, out);
                    }
                }
                End::Party | End::Valley(_) => {}
            }
        }
    }

    /// A small gable standing on the wall plate of a roof's +c (`side` 1) or
    /// -c (-1) side, `w` wide about `a0` along the ridge, its own little roof
    /// running back into the slope. With `window`, a window in it.
    #[allow(clippy::too_many_arguments)]
    pub fn gablet(&mut self, rg: &Range, s: &RoofSpec, a0: f32, w: f32, side: f32, wall: &Wall, window: bool) {
        let hd = rg.dep * 0.5;
        // as steep as the roof it stands on, and never as high
        let gr = (w * 0.5 * s.pitch(rg).max(0.9)).min(s.rise * 0.8);
        let (front, back) = (side * (hd + s.eaves * 0.6), side * hd * (1.0 - gr / s.rise));
        let top = s.h + gr;
        let (da, dc) = rg.axes();
        let out = self.dir(dc.0, 0.0, dc.1) * side;
        let along = self.dir(da.0, 0.0, da.1);
        for sign in [-1.0f32, 1.0] {
            let a = a0 + sign * w * 0.5;
            let v = [self.rp(rg, a, s.h, front), self.rp(rg, a0, top, front), self.rp(rg, a0, top, back), self.rp(rg, a, s.h, side * hd)];
            self.quad(v, shade(s.col, if sign > 0.0 { 0.94 } else { 0.86 }), Vec3::Y + along * sign);
        }
        let col = shade(wall.col, if side > 0.0 { SIDE_SHADE[rg.turn as usize % 4] } else { SIDE_SHADE[(2 + rg.turn as usize) % 4] });
        let v = [self.rp(rg, a0 - w * 0.5, s.h, side * hd), self.rp(rg, a0 + w * 0.5, s.h, side * hd), self.rp(rg, a0, top - 0.0003, side * hd)];
        self.solid.tri_toward(v, [col, col, shade(col, if self.near() { 0.82 } else { 1.0 })], out);
        if window && !self.far() {
            let (ww, wh) = (w * 0.26, gr * 0.42);
            let glass = *self.pal.glass.window;
            let c = side * (hd + PROUD);
            if self.near() {
                let v = [self.rp(rg, a0 - ww * 0.5 - 0.0004, s.h + gr * 0.1, c), self.rp(rg, a0 + ww * 0.5 + 0.0004, s.h + gr * 0.1, c), self.rp(rg, a0 + ww * 0.5 + 0.0004, s.h + gr * 0.14 + wh, c), self.rp(rg, a0 - ww * 0.5 - 0.0004, s.h + gr * 0.14 + wh, c)];
                self.quad(v, wall.frame, out);
            }
            let c = side * (hd + PROUD + LAYER);
            let v = [self.rp(rg, a0 - ww * 0.5, s.h + gr * 0.12, c), self.rp(rg, a0 + ww * 0.5, s.h + gr * 0.12, c), self.rp(rg, a0 + ww * 0.5, s.h + gr * 0.12 + wh, c), self.rp(rg, a0 - ww * 0.5, s.h + gr * 0.12 + wh, c)];
            self.quad(v, glass, out);
        }
    }

    /// A dormer window standing in a roof's +c (`side` 1) or -c (-1) slope,
    /// `up` of the way from the eaves to the ridge, `w` wide about `a0`.
    #[allow(clippy::too_many_arguments)]
    pub fn dormer(&mut self, rg: &Range, s: &RoofSpec, a0: f32, w: f32, side: f32, up: f32, wall: &Wall) {
        if self.far() {
            return;
        }
        let hd = rg.dep * 0.5;
        let (y0, tall) = (s.h + s.rise * up, w * 0.9);
        let (front, top) = (side * hd * (1.0 - up), y0 + tall);
        let back = side * hd * (1.0 - (top - s.h) / s.rise).max(0.0);
        let (da, dc) = rg.axes();
        let out = self.dir(dc.0, 0.0, dc.1) * side;
        let along = self.dir(da.0, 0.0, da.1);
        let (a, b) = (a0 - w * 0.5, a0 + w * 0.5);
        let cheek = shade(*self.pal.roofs.church_lead, 0.8);
        self.quad([self.rp(rg, a, y0, front), self.rp(rg, b, y0, front), self.rp(rg, b, top, front), self.rp(rg, a, top, front)], wall.frame, out);
        let m = w * 0.14;
        let c = front + side * LAYER;
        self.quad([self.rp(rg, a + m, y0 + m, c), self.rp(rg, b - m, y0 + m, c), self.rp(rg, b - m, top - m, c), self.rp(rg, a + m, top - m, c)], *self.pal.glass.window, out);
        for (at, way) in [(a, -1.0f32), (b, 1.0)] {
            let v = [self.rp(rg, at, y0, front), self.rp(rg, at, top, front), self.rp(rg, at, top, back)];
            self.solid.tri_toward(v, [cheek; 3], along * way);
        }
        let e = 0.0006;
        let lip = front + side * e;
        self.quad([self.rp(rg, a - e, top, lip), self.rp(rg, b + e, top, lip), self.rp(rg, b + e, top + 0.0006, back), self.rp(rg, a - e, top + 0.0006, back)], shade(s.col, 0.9), Vec3::Y);
    }

    /// A lean-to: a range whose roof is one slope, from `high` on its -c
    /// side (where it stands against something taller) down to `low` on its
    /// +c side. `outer` is how the low wall is drawn, and `ends` which of
    /// its end walls (toward +a and -a) are.
    #[allow(clippy::too_many_arguments)]
    pub fn lean_to(&mut self, rg: &Range, low: f32, high: f32, w: &Wall, roof: Rgba, over: f32, outer: Side, ends: [bool; 2]) {
        let (hl, hd) = (rg.len * 0.5, rg.dep * 0.5);
        let y0 = -self.drop(rg);
        self.walls(rg, y0, low, w, [outer, Side::NONE, Side::NONE, Side::NONE], true, true);
        for (i, show) in [(1usize, ends[0]), (3, ends[1])] {
            if !show {
                continue;
            }
            let f = rg.face(i);
            let col = shade(w.col, SIDE_SHADE[(i + rg.turn as usize) % 4]);
            // the +a end runs from the low side to the high, the -a end from the high to the low
            let (ya, yb) = if i == 1 { (low, high) } else { (high, low) };
            let p = [self.fp(&f, 0.0, y0, 0.0), self.fp(&f, f.len, y0, 0.0), self.fp(&f, f.len, yb, 0.0), self.fp(&f, 0.0, ya, 0.0)];
            let foot = shade(col, 0.78);
            self.solid.quad_toward(p, [foot, foot, col, col], self.face_out(&f));
        }
        if self.far() {
            // from far off the slope alone, with no oversail
            let v = [self.rp(rg, -hl, high, -hd), self.rp(rg, hl, high, -hd), self.rp(rg, hl, low, hd), self.rp(rg, -hl, low, hd)];
            self.quad(v, roof, Vec3::Y);
            return;
        }
        let pitch = (high - low) / rg.dep;
        let (ce, ye) = (hd + over, low - over * pitch);
        let (a0, a1) = (-hl - if ends[1] { over * 0.6 } else { 0.0 }, hl + if ends[0] { over * 0.6 } else { 0.0 });
        let v = [self.rp(rg, a0, high, -hd), self.rp(rg, a1, high, -hd), self.rp(rg, a1, ye, ce), self.rp(rg, a0, ye, ce)];
        self.quad(v, roof, Vec3::Y);
        if self.near() {
            let (_, dc) = rg.axes();
            let v = [self.rp(rg, a0, ye, ce), self.rp(rg, a1, ye, ce), self.rp(rg, a1, ye - 0.0012, ce), self.rp(rg, a0, ye - 0.0012, ce)];
            self.quad(v, shade(roof, 0.66), self.dir(dc.0, 0.0, dc.1));
        }
    }

    /// A bay window standing `out` from a side of a range, `w` wide about
    /// `u` along it, from the ground to `top`: glazed in front and at its
    /// cheeks, under a little roof of its own.
    #[allow(clippy::too_many_arguments)]
    pub fn bay(&mut self, rg: &Range, side: usize, u: f32, w: f32, out: f32, y0: f32, top: f32, wall: &Wall, roof: Rgba) {
        if self.far() {
            return;
        }
        let f = rg.face(side);
        let k = SIDE_SHADE[(side + rg.turn as usize) % 4];
        let (u0, u1) = (u - w * 0.5, u + w * 0.5);
        let (col, along) = (shade(wall.col, k), self.face_along(&f));
        self.panel(&f, u0, u1, y0, top, out, shade(col, 0.8), col);
        for (at, way) in [(u0, -1.0f32), (u1, 1.0)] {
            let p = [self.fp(&f, at, y0, 0.0), self.fp(&f, at, y0, out), self.fp(&f, at, top, out), self.fp(&f, at, top, 0.0)];
            self.quad(p, shade(col, if way > 0.0 { 0.9 } else { 0.95 }), along * way);
        }
        let e = 0.0006;
        let p = [self.fp(&f, u0 - e, top + out * 0.7, 0.0), self.fp(&f, u1 + e, top + out * 0.7, 0.0), self.fp(&f, u1 + e, top, out + e), self.fp(&f, u0 - e, top, out + e)];
        self.quad(p, roof, Vec3::Y);
        let glass = *self.pal.glass.window;
        let (sill, head) = (FOOT + 0.006, top - 0.0012);
        if !self.near() {
            self.panel(&f, u0 + 0.0008, u1 - 0.0008, sill, head, out + PROUD, glass, glass);
            return;
        }
        let frame = shade(wall.frame, k);
        self.panel(&f, u0 + 0.0005, u1 - 0.0005, sill - 0.0004, head + 0.0004, out + LAYER, frame, frame);
        let lights = 3;
        let lw = (w - 0.0016) / lights as f32;
        for i in 0..lights {
            let a = u0 + 0.0008 + lw * i as f32;
            self.panel(&f, a + 0.0003, a + lw - 0.0003, sill, head, out + 2.0 * LAYER, glass, shade(glass, 1.12));
        }
        for (at, way) in [(u0, -1.0f32), (u1, 1.0)] {
            let o = way * LAYER;
            let p = [self.fp(&f, at + o, sill, out * 0.2), self.fp(&f, at + o, sill, out * 0.85), self.fp(&f, at + o, head, out * 0.85), self.fp(&f, at + o, head, out * 0.2)];
            self.quad(p, shade(glass, 0.9), along * way);
        }
    }

    // -------------------------------------------------------------- stacks and what stands about

    /// A box from x0 to x1 and z0 to z1, y0 to y1 above the origin's height, with a top if it is given one.
    #[allow(clippy::too_many_arguments)]
    pub fn boxy(&mut self, x0: f32, x1: f32, z0: f32, z1: f32, y0: f32, y1: f32, col: Rgba, top: Option<Rgba>) {
        let rg = Range::rect(x0, x1, z0, z1);
        for i in 0..4 {
            let f = rg.face(i);
            let c = shade(col, SIDE_SHADE[i]);
            self.panel(&f, 0.0, f.len, y0, y1, 0.0, c, c);
        }
        if let Some(t) = top {
            self.quad([self.pt(x0, y1, z1), self.pt(x1, y1, z1), self.pt(x1, y1, z0), self.pt(x0, y1, z0)], t, Vec3::Y);
        }
    }

    /// A box standing on the ground at a place, `h` tall.
    #[allow(clippy::too_many_arguments)]
    pub fn boxy_on(&mut self, x0: f32, x1: f32, z0: f32, z1: f32, h: f32, col: Rgba, top: Option<Rgba>) {
        let g = self.ground((x0 + x1) * 0.5, (z0 + z1) * 0.5);
        self.boxy(x0, x1, z0, z1, g - 0.003, g + h, col, top);
    }

    /// A chimney stack `w` along the front and `d` through, from `y0` (within
    /// the roof, or the ground for one built against an end wall) to `y1`,
    /// with its pots. It gives back the middle of its top, where its smoke rises.
    #[allow(clippy::too_many_arguments)]
    pub fn stack(&mut self, x: f32, z: f32, w: f32, d: f32, y0: f32, y1: f32, col: Rgba, pots: usize) -> Vec3 {
        let top = self.pt(x, y1, z);
        if self.far() {
            return top;
        }
        let soot = *self.pal.smoke.stack_top;
        let (hw, hd) = (w * 0.5, d * 0.5);
        if !self.near() {
            self.boxy(x - hw, x + hw, z - hd, z + hd, y0, y1, col, Some(soot));
            return top;
        }
        // the oversailing courses at its head, and the pots
        let (neck, e) = (y1 - 0.002, 0.0007);
        self.boxy(x - hw, x + hw, z - hd, z + hd, y0, neck, col, None);
        self.boxy(x - hw - e, x + hw + e, z - hd - e, z + hd + e, neck, y1, shade(col, 0.9), Some(soot));
        let pot = *self.pal.roofs.chimney_pot;
        let along_x = w >= d;
        let room = if along_x { w } else { d };
        for i in 0..pots {
            let t = (i as f32 + 0.5) / pots as f32 - 0.5;
            let p = if along_x { self.pt(x + t * room * 0.8, y1, z) } else { self.pt(x, y1, z + t * room * 0.8) };
            self.solid.frustum_turned(p, 0.0011, 0.0008, 0.003, 5, 0.3 - self.yaw, 0.0, pot, Some(*self.pal.smoke.soot));
        }
        top + Vec3::Y * 0.003
    }

    /// A patch of ground from x0 to x1 and z0 to z1, lying `lift` above the land.
    #[allow(clippy::too_many_arguments)]
    pub fn patch(&mut self, x0: f32, x1: f32, z0: f32, z1: f32, lift: f32, col: Rgba) {
        let at = |kit: &Self, x: f32, z: f32| kit.pt(x, kit.ground(x, z) + lift, z);
        let p = [at(self, x0, z1), at(self, x1, z1), at(self, x1, z0), at(self, x0, z0)];
        self.quad(p, col, Vec3::Y);
    }

    /// A run of wall, hedge or fence from one place to another, `h` high and `t` thick, following the ground.
    pub fn low_wall(&mut self, a: (f32, f32), b: (f32, f32), h: f32, t: f32, col: Rgba) {
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        if len < 1e-4 {
            return;
        }
        let (dx, dz) = ((b.0 - a.0) / len, (b.1 - a.1) / len);
        // a quarter turn from the way it runs
        let (nx, nz) = (-dz * t * 0.5, dx * t * 0.5);
        let (ga, gb) = (self.ground(a.0, a.1), self.ground(b.0, b.1));
        let side = self.dir(-dz, 0.0, dx);
        for (s, k) in [(1.0f32, 1.0), (-1.0, 0.84)] {
            let p = [
                self.pt(a.0 + nx * s, ga - 0.002, a.1 + nz * s),
                self.pt(b.0 + nx * s, gb - 0.002, b.1 + nz * s),
                self.pt(b.0 + nx * s, gb + h, b.1 + nz * s),
                self.pt(a.0 + nx * s, ga + h, a.1 + nz * s),
            ];
            let c = shade(col, k);
            self.solid.quad_toward(p, [shade(c, 0.8), shade(c, 0.8), c, c], side * s);
        }
        let p = [self.pt(a.0 + nx, ga + h, a.1 + nz), self.pt(b.0 + nx, gb + h, b.1 + nz), self.pt(b.0 - nx, gb + h, b.1 - nz), self.pt(a.0 - nx, ga + h, a.1 - nz)];
        self.quad(p, shade(col, 1.06), Vec3::Y);
        // its two ends
        let along = self.dir(dx, 0.0, dz);
        for (e, g, way) in [(a, ga, -1.0f32), (b, gb, 1.0)] {
            let p = [self.pt(e.0 + nx, g - 0.002, e.1 + nz), self.pt(e.0 - nx, g - 0.002, e.1 - nz), self.pt(e.0 - nx, g + h, e.1 - nz), self.pt(e.0 + nx, g + h, e.1 + nz)];
            self.quad(p, shade(col, 0.9), along * way);
        }
    }

    /// A privy or a wash-house: a small shed standing on the ground under a
    /// single slope that falls toward +x (`fall` 1) or -x (-1), a door in its front.
    #[allow(clippy::too_many_arguments)]
    pub fn shed(&mut self, x0: f32, x1: f32, z0: f32, z1: f32, h: f32, fall: f32, wall: Rgba, roof: Rgba, door_front: bool) {
        let g = self.ground((x0 + x1) * 0.5, (z0 + z1) * 0.5);
        let (lo, hi) = (g + h, g + h + (x1 - x0) * 0.35);
        let (yl, yr) = if fall > 0.0 { (hi, lo) } else { (lo, hi) };
        let rg = Range::rect(x0, x1, z0, z1);
        let y0 = g - 0.003;
        for i in 0..4 {
            let f = rg.face(i);
            let c = shade(wall, SIDE_SHADE[i]);
            // the heights of the wall's top at its beginning and its end
            let (ta, tb) = match i {
                0 => (yl, yr),
                1 => (yr, yr),
                2 => (yr, yl),
                _ => (yl, yl),
            };
            let p = [self.fp(&f, 0.0, y0, 0.0), self.fp(&f, f.len, y0, 0.0), self.fp(&f, f.len, tb, 0.0), self.fp(&f, 0.0, ta, 0.0)];
            self.solid.quad_toward(p, [shade(c, 0.8), shade(c, 0.8), c, c], self.face_out(&f));
        }
        let e = 0.0007;
        let p = [self.pt(x0 - e, yl, z1 + e), self.pt(x1 + e, yr, z1 + e), self.pt(x1 + e, yr, z0 - e), self.pt(x0 - e, yl, z0 - e)];
        self.quad(p, roof, Vec3::Y);
        let door = *self.pal.timber.door;
        let f = rg.face(if door_front { 0 } else { 2 });
        let m = f.len * 0.5;
        self.panel(&f, m - 0.0016, m + 0.0016, g, g + h * 0.78, LAYER, shade(door, 0.85), door);
    }

    /// A water butt standing on the ground: a tub of staves.
    pub fn butt(&mut self, x: f32, z: f32) {
        let g = self.ground(x, z);
        let wood = *self.pal.timber.logs;
        self.solid.frustum_turned(self.pt(x, g, z), 0.0019, 0.0017, 0.0048, 6, 0.4 - self.yaw, 0.002, wood, Some(shade(*self.pal.water.river, 0.8)));
    }

    /// A pile of cut wood, `len` long from x0, standing against something at `z`.
    pub fn woodpile(&mut self, x0: f32, z: f32, len: f32, deep: f32) {
        let wood = *self.pal.timber.logs;
        let (za, zb) = if deep > 0.0 { (z, z + deep) } else { (z + deep, z) };
        self.boxy_on(x0, x0 + len, za, zb, 0.0042, shade(wood, 1.25), Some(wood));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WALL: Wall = Wall { col: [0.8, 0.7, 0.6, 1.0], plinth: None, frame: [1.0; 4], sill: [0.9, 0.9, 0.9, 1.0] };

    fn kit_on<'a>(b: &'a mut MeshBuf, site: &'a Site<'static>, pal: &'a Palette, detail: Detail) -> Kit<'a, 'static> {
        Kit::new(b, site, pal, detail, 0.0)
    }

    /// Every face of what was built looks away from the point `mid`.
    fn looks_out(b: &MeshBuf, mid: Vec3) -> bool {
        b.idx.chunks(3).all(|t| {
            let p = [0, 1, 2].map(|i| Vec3::from_array(b.pos[t[i] as usize]));
            let n = Vec3::from_array(b.nrm[t[0] as usize]);
            n.dot((p[0] + p[1] + p[2]) / 3.0 - mid) > -1e-5
        })
    }

    #[test]
    fn eras_follow_the_year_a_house_was_built() {
        assert_eq!(Era::of(1690), Era::Old);
        assert_eq!(Era::of(1720), Era::Georgian);
        assert_eq!(Era::of(1814), Era::Georgian);
        assert_eq!(Era::of(1815), Era::Brick);
        assert_eq!(Era::of(1860), Era::Late);
        assert!(Era::ALL.windows(2).all(|w| w[0] < w[1]) && Detail::ALL.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn a_range_turns_by_quarters_and_its_sides_look_outward() {
        let pal = Palette::standard();
        let site = Site::level(Vec3::ZERO, 0.0);
        for turn in 0..4 {
            let rg = Range::rect(-0.02, 0.02, -0.01, 0.01).turned(turn);
            // its four sides, in order, are a closed way round it
            for i in 0..4 {
                let (f, g) = (rg.face(i), rg.face(i + 1));
                assert!((f.ox + f.ux * f.len - g.ox).abs() < 1e-6 && (f.oz + f.uz * f.len - g.oz).abs() < 1e-6, "turn {turn} side {i}");
            }
            for detail in Detail::ALL {
                let mut b = MeshBuf::default();
                let mut k = kit_on(&mut b, &site, &pal, detail);
                k.walls(&rg, -0.003, 0.048, &WALL, [Side::PLAIN; 4], true, true);
                assert!(b.is_sound() && looks_out(&b, Vec3::new(0.0, 0.02, 0.0)), "turn {turn} {detail:?}");
            }
        }
    }

    #[test]
    fn a_wall_close_to_leaves_a_hole_for_each_opening_and_the_rest_is_whole() {
        let pal = Palette::standard();
        let site = Site::level(Vec3::ZERO, 0.0);
        let rg = Range::rect(-0.02, 0.02, -0.01, 0.01);
        let ops = [
            Opening { u: 0.01, y: FOOT, w: 0.005, h: 0.014, kind: Open::Door { paint: [0.2, 0.3, 0.2, 1.0], over: Over::Hood } },
            Opening { u: 0.03, y: 0.009, w: 0.006, h: 0.008, kind: Open::Sash },
            Opening { u: 0.01, y: STOREY + 0.008, w: 0.006, h: 0.008, kind: Open::Casement },
            Opening { u: 0.03, y: STOREY + 0.008, w: 0.006, h: 0.008, kind: Open::Sash },
        ];
        let mut b = MeshBuf::default();
        let mut k = kit_on(&mut b, &site, &pal, Detail::Near);
        k.walls(&rg, -0.003, 2.0 * STOREY, &WALL, [Side::with(&ops), Side::NONE, Side::NONE, Side::NONE], true, true);
        assert!(b.is_sound());
        // the wall's own faces lie in its plane and are of its colour; together with the holes they make up the whole wall
        let in_plane = |t: &[u32]| t.iter().all(|&i| (b.pos[i as usize][2] - 0.01).abs() < 1e-6);
        let area = |t: &[u32]| {
            let p = [0, 1, 2].map(|i| Vec3::from_array(b.pos[t[i] as usize]));
            (p[1] - p[0]).cross(p[2] - p[0]).length() * 0.5
        };
        let wall: f32 = b.idx.chunks(3).filter(|t| in_plane(t)).map(area).sum();
        let holes: f32 = ops.iter().map(|o| o.w * o.h).sum();
        let whole = 0.04 * (2.0 * STOREY + 0.003);
        assert!((wall + holes - whole).abs() < 1e-6, "wall {wall} and holes {holes} of {whole}");
        // the glass sits back in the wall
        let glass = *pal.glass.window;
        assert!(b.pos.iter().zip(&b.col).filter(|(_, c)| **c == glass).all(|(p, _)| p[2] < 0.01 - 0.0005));
        // from the play distance the same openings are panels on a whole wall
        let mut flat = MeshBuf::default();
        let mut k = kit_on(&mut flat, &site, &pal, Detail::Play);
        k.walls(&rg, -0.003, 2.0 * STOREY, &WALL, [Side::with(&ops), Side::NONE, Side::NONE, Side::NONE], true, true);
        assert_eq!(flat.triangles(), 2 * (1 + ops.len()));
        assert!(flat.triangles() * 5 < b.triangles());
    }

    #[test]
    fn roofs_of_every_end_look_up_and_keep_their_heights() {
        let pal = Palette::standard();
        let site = Site::level(Vec3::ZERO, 0.0);
        let ends = [End::Gable, End::Hip, End::HalfHip, End::Party, End::Valley(0.004)];
        for turn in 0..4 {
            for (i, &e0) in ends.iter().enumerate() {
                let e1 = ends[(i + 2) % ends.len()];
                let rg = Range::rect(-0.03, 0.03, -0.012, 0.012).turned(turn);
                let spec = RoofSpec { h: 0.024, rise: 0.012, eaves: 0.003, verge: 0.002, ends: [e0, e1], col: [0.6, 0.3, 0.2, 1.0], gable: WALL.col, thick: 0.002 };
                let mut b = MeshBuf::default();
                let mut k = kit_on(&mut b, &site, &pal, Detail::Near);
                k.roof(&rg, &spec);
                assert!(b.is_sound() && !b.is_empty());
                assert!(looks_out(&b, Vec3::new(0.0, 0.02, 0.0)), "turn {turn}, {e0:?} and {e1:?}");
                let (lo, hi) = b.bounds().unwrap();
                assert!((hi.y - 0.0365).abs() < 1e-5 && lo.y > 0.024 - 0.006, "{lo} {hi}");
                // far off it is slopes and gables only
                let mut far = MeshBuf::default();
                let mut k = kit_on(&mut far, &site, &pal, Detail::Far);
                k.roof(&rg, &spec);
                assert!(far.triangles() <= b.triangles() && far.triangles() <= 12);
            }
        }
    }

    #[test]
    fn a_valley_meets_the_roof_it_runs_into() {
        // a wing behind a main range, their eaves level: the wing's ridge runs on to the main slope
        let pal = Palette::standard();
        let site = Site::level(Vec3::ZERO, 0.0);
        // (the main range is 0.024 deep about the origin)
        let (h, rise, wing_rise) = (0.024, 0.012, 0.007);
        let wing = Range::rect_across(-0.007, 0.007, -0.03, -0.012);
        let run = 0.012 * wing_rise / rise;
        let mut b = MeshBuf::default();
        let mut k = kit_on(&mut b, &site, &pal, Detail::Play);
        k.roof(&wing, &RoofSpec { h, rise: wing_rise, eaves: 0.002, verge: 0.002, ends: [End::Gable, End::Valley(run)], col: [0.5; 4], gable: WALL.col, thick: 0.0 });
        // the furthest the wing's ridge reaches toward the front is on the main roof's back slope
        let reach = b.pos.iter().filter(|p| (p[1] - (h + wing_rise + 0.0005)).abs() < 1e-6).map(|p| p[2]).fold(f32::MIN, f32::max);
        let slope_at = |z: f32| h + rise * (z + 0.012) / 0.012;
        assert!((slope_at(reach) - (h + wing_rise)).abs() < 1e-5, "the ridge ends at {reach}");
    }

    #[test]
    fn an_old_house_goes_crooked_all_together_and_a_new_one_not_at_all() {
        let pal = Palette::standard();
        let site = Site::level(Vec3::ZERO, 0.0);
        let mut b = MeshBuf::default();
        let mut k = kit_on(&mut b, &site, &pal, Detail::Near);
        assert_eq!(k.pt(0.01, 0.03, 0.02), Vec3::new(0.01, 0.03, 0.02));
        k.crooked((0.05, -0.02), 0.003, 0.02, 0.024, 0.04);
        // the foot of a wall stays where it is, its top leans, and the middle of the ridge dips
        assert_eq!(k.pt(0.01, 0.0, 0.02), Vec3::new(0.01, 0.0, 0.02));
        let top = k.pt(0.01, 0.024, 0.02);
        assert!((top.x - 0.0112).abs() < 1e-6 && (top.z - 0.01952).abs() < 1e-6 && top.y == 0.024);
        assert!((k.pt(0.0, 0.04, 0.0).y - 0.037).abs() < 1e-6);
        assert!((k.pt(0.02, 0.04, 0.0).y - 0.04).abs() < 1e-6);
    }

    #[test]
    fn a_sagging_roof_still_clears_the_top_of_its_walls() {
        // the roof over the line of each long wall, wherever along it, is above the wall's top
        let pal = Palette::standard();
        let site = Site::level(Vec3::ZERO, 0.0);
        let rg = Range::rect(-0.02, 0.02, -0.013, 0.013);
        let (h, rise) = (0.0205, 0.016);
        let spec = RoofSpec { h, rise, eaves: 0.003, verge: 0.002, ends: [End::Gable, End::Gable], col: [0.6, 0.3, 0.2, 1.0], gable: WALL.col, thick: 0.0 };
        let mut b = MeshBuf::default();
        let mut k = kit_on(&mut b, &site, &pal, Detail::Near);
        k.crooked((0.0, 0.0), 0.0036, 0.022, h, h + rise);
        k.roof(&rg, &spec);
        let over = |x: f32, z: f32| {
            b.idx.chunks(3).filter(|t| b.nrm[t[0] as usize][1] > 0.3).find_map(|t| {
                let p = [0, 1, 2].map(|i| Vec3::from_array(b.pos[t[i] as usize]));
                let (e1, e2) = (p[1] - p[0], p[2] - p[0]);
                let det = e1.x * e2.z - e2.x * e1.z;
                let (dx, dz) = (x - p[0].x, z - p[0].z);
                let (s, t) = ((dx * e2.z - e2.x * dz) / det, (e1.x * dz - dx * e1.z) / det);
                (s >= 0.0 && t >= 0.0 && s + t <= 1.0).then(|| p[0].y + s * e1.y + t * e2.y)
            })
        };
        for i in 0..=40 {
            let x = -0.02 + 0.001 * i as f32;
            for z in [-0.013, 0.013] {
                let roof = over(x, z).unwrap_or_else(|| panic!("no roof over {x}, {z}"));
                assert!(roof > h + 0.0003, "the roof is {} above the wall's top at {x}, {z}", roof - h);
            }
        }
    }

    #[test]
    fn the_way_under_an_arch_has_a_floor_in_shadow() {
        let pal = Palette::standard();
        let site = Site::level(Vec3::ZERO, 0.0);
        let rg = Range::rect(-0.02, 0.02, -0.01, 0.01);
        let ops = [Opening { u: 0.02, y: FOOT, w: 0.02, h: 0.0138, kind: Open::Arch }];
        let mut b = MeshBuf::default();
        let mut k = kit_on(&mut b, &site, &pal, Detail::Near);
        k.walls(&rg, -0.003, 2.0 * STOREY, &WALL, [Side::with(&ops), Side::NONE, Side::NONE, Side::NONE], true, true);
        assert!(b.is_sound());
        // a face that looks up, at the foot of the opening, behind the wall's face, and dark
        let floor = b.idx.chunks(3).any(|t| {
            let up = b.nrm[t[0] as usize][1] > 0.99;
            let within = t.iter().all(|&i| (b.pos[i as usize][1] - FOOT).abs() < 1e-6 && b.pos[i as usize][2] <= 0.01 + 1e-6);
            let dark = t.iter().all(|&i| b.col[i as usize][..3].iter().all(|c| *c < 0.2));
            up && within && dark
        });
        assert!(floor);
    }

    #[test]
    fn walls_reach_down_a_slope_and_yards_follow_it() {
        let pal = Palette::standard();
        let slope = |x: f32, _z: f32| x * 0.2;
        let site = Site::on(Vec3::ZERO, 0.0, &slope);
        let mut b = MeshBuf::default();
        let mut k = Kit::new(&mut b, &site, &pal, Detail::Near, 0.0);
        let rg = Range::rect(-0.02, 0.02, -0.01, 0.01);
        assert!((k.drop(&rg) - 0.007).abs() < 1e-6);
        assert!((k.ground(0.01, 0.0) - 0.002).abs() < 1e-7);
        k.patch(-0.02, 0.02, -0.03, -0.01, 0.004, [0.5; 4]);
        k.low_wall((-0.02, -0.03), (0.02, -0.03), 0.005, 0.0015, [0.5; 4]);
        assert!(b.is_sound());
        for p in &b.pos {
            let above = p[1] - slope(p[0], p[2]);
            assert!(above > -0.0021 && above < 0.0051, "{above} above the land at {p:?}");
        }
    }
}
