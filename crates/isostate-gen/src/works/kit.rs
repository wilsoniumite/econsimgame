//! The parts a works is put together from, and the yard they stand in.
//!
//! Every kind of works is a few lines that place these parts in its yard:
//! sheds of one sort and another, chimneys and stacks, heaps and stacks of
//! goods, a waterwheel, the wall and the gate. The parts know the works'
//! state (a roof is not on yet while it is going up, and half off once it is
//! derelict; nothing smokes unless it is working) and its level of detail,
//! so a kind says what it is and the state and the level follow by
//! themselves.
//!
//! The walls, roofs and openings are the houses' (`crate::kit`): a shed's
//! walls are built last (`Ctx::finish`), when everything the kind hung on
//! them is known, so that close to each opening is let into its wall as a
//! house's is. A stretch of wall the kind left bare is given what such a
//! wall has (`Fill`): windows at the module and the storey, or the posts of
//! a boarded frame.
//!
//! Places are in the works' own frame: x along its front, z out toward the
//! gate, the origin in the middle of the yard, which is `2 * hw` wide and
//! `2 * hd` deep. Heights are above the ground under each part.

use glam::{Quat, Vec3};

use super::{State, Works};
use crate::colour::{lerp, shade, Rgba};
use crate::hash::hash2;
use crate::houses::roof::{Cover, Lie};
use crate::kit::{Detail, End, Kit, Open, Opening, Range, RoofSpec, Side, Wall, FOOT};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// The height of one storey and the width of one bay: the scale's.
pub(super) use crate::scale::{BAY, STOREY};

/// How far what stands in a yard reaches below the ground, so that a slope shows no gap.
pub(super) const SINK: f32 = 0.02;
/// How far a yard's ground lies above the land, and what lies on it above that.
pub(super) const LIFT: f32 = 0.0045;
/// The yard's ground follows the land in squares no bigger than this.
pub(super) const GROUND_STEP: f32 = 0.045;
/// A works' window: how wide, how tall, and how high its sill stands above its floor.
const WINDOW: (f32, f32) = (0.0066, 0.0086);
const SILL: f32 = FOOT + 0.0058;
/// The wind: every plume leans this way, as the pennants fly.
pub const WIND: Vec3 = Vec3::new(0.894, 0.0, 0.447);

/// A face of a shed: its two long walls and its two gable ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Face {
    /// the long wall on the shed's own +z
    Front,
    Back,
    /// the gable end on the shed's own +x
    EndA,
    EndB,
}

/// What a stretch of wall that a kind hung nothing on is given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Fill {
    /// windows of small panes, a bay apart and a storey apart
    Windows,
    /// the posts of a boarded frame, a bay apart
    Posts,
    /// nothing: a bank, a wall that something else stands against
    Nothing,
}

/// A building that has been placed, for hanging doors, windows and vents on.
/// Its own frame: x along the ridge, z across, the origin in the middle of
/// its floor.
#[derive(Clone, Copy, Debug)]
pub(super) struct Shed {
    /// where its middle stands in the yard
    pub x: f32,
    pub z: f32,
    pub len: f32,
    pub dep: f32,
    /// the height of its walls, and of its ridge above them
    pub h: f32,
    pub rise: f32,
    /// its ridge runs across the yard (along z) rather than along the front
    pub across: bool,
    /// which of the works' walled buildings it is (none, for a roof on posts)
    id: usize,
    /// how far its ridge dips in the middle: old roofs sag
    dip: f32,
    o: Vec3,
    q: Quat,
    yaw: f32,
}

impl Shed {
    /// A point in the shed's own frame, on the map.
    pub fn pt(&self, x: f32, y: f32, z: f32) -> Vec3 {
        self.o + self.q * Vec3::new(x, y, z)
    }

    /// Where a point of the shed's own frame is in the yard.
    pub fn local(&self, sx: f32, sz: f32) -> (f32, f32) {
        if self.across { (self.x + sz, self.z - sx) } else { (self.x + sx, self.z + sz) }
    }

    /// The shed as the kit has it: a range about the shed's own middle, its
    /// length along the ridge. Its sides in the kit's order are the front,
    /// end A, the back and end B.
    fn range(&self) -> Range {
        Range { cx: 0.0, cz: 0.0, len: self.len, dep: self.dep, turn: self.across as u8 }
    }
}

/// The walls of a building, waiting for everything that is hung on them.
struct Pending {
    s: Shed,
    wall: Wall,
    /// the height of the walls
    top: f32,
    /// the openings of each side, in the kit's order
    ops: [Vec<Opening>; 4],
    fill: Fill,
    /// the walls end under the eaves of a roof
    eaves: bool,
    /// a lean-to: the heights of its low and its high side, and its roof
    lean: Option<(f32, f32, Rgba)>,
    /// a lean-to that stands by itself, with a back wall of its own
    free: bool,
}

/// What a kind of works builds with: the buffers, the site, the works itself
/// and its yard.
pub(super) struct Ctx<'a, 's> {
    pub solid: &'a mut MeshBuf,
    pub smoke: &'a mut MeshBuf,
    pub flat: &'a mut MeshBuf,
    pub site: &'a Site<'s>,
    pub pal: &'a Palette,
    pub w: Works,
    /// half the yard's width and half its depth
    pub hw: f32,
    pub hd: f32,
    /// how much bigger than a works of the standard size the buildings are drawn
    pub g: f32,
    /// the top of the highest gable so far
    pub peak: Option<Vec3>,
    /// the main buildings, in the order they were built
    pub sheds: Vec<Shed>,
    /// its windows are lit from within: a mill at work
    pub lit: bool,
    /// it is a part of a planned works (`site`): the plot's ground and the wall
    /// round it are the plan's, so `ground` and `wall` only say what was asked for
    pub planned: bool,
    /// how weathered it is by its age, in steps from 0 (new) to 2 (two generations old)
    pub worn: f32,
    /// the ground and the wall a kind asked for while it was a part of a plan
    pub asked_ground: Option<Rgba>,
    pub asked_wall: Option<(Rgba, f32)>,
    pending: Vec<Pending>,
    count: i32,
}

impl<'a, 's> Ctx<'a, 's> {
    pub fn new(solid: &'a mut MeshBuf, smoke: &'a mut MeshBuf, flat: &'a mut MeshBuf, site: &'a Site<'s>, w: &Works, pal: &'a Palette) -> Self {
        let (fw, fd) = super::footprint(w.kind, w.size);
        Ctx { solid, smoke, flat, site, pal, w: *w, hw: fw * 0.5, hd: fd * 0.5, g: super::grain(w.size), peak: None, sheds: Vec::new(), lit: false, planned: false, worn: 0.0, asked_ground: None, asked_wall: None, pending: Vec::new(), count: 0 }
    }

    /// A part of a planned works (`site`): it stands on the ground the plan
    /// gave it, `2 * hw` by `2 * hd` about the site's origin, and has the
    /// year it was built in `w.founded`, its own state and its own seed.
    #[allow(clippy::too_many_arguments)]
    pub fn part(solid: &'a mut MeshBuf, smoke: &'a mut MeshBuf, flat: &'a mut MeshBuf, site: &'a Site<'s>, w: &Works, pal: &'a Palette, hw: f32, hd: f32, worn: f32) -> Self {
        Ctx { hw, hd, planned: true, worn, ..Ctx::new(solid, smoke, flat, site, w, pal) }
    }

    // -------------------------------------------------------------- what the works is

    /// Its fires are lit: chimneys smoke, furnace mouths glow, goods stand in the yard.
    pub fn busy(&self) -> bool {
        matches!(self.w.state, State::Working | State::Extending)
    }

    /// There is stock in the yard: it is working, or stands idle with its heaps about it.
    pub fn stocked(&self) -> bool {
        self.busy() || self.w.state == State::Idle
    }

    pub fn going_up(&self) -> bool {
        self.w.state == State::GoingUp
    }

    pub fn derelict(&self) -> bool {
        self.w.state == State::Derelict
    }

    /// Seen from across the island: walls, roofs and the tall chimneys.
    pub fn far(&self) -> bool {
        self.w.detail == Detail::Far
    }

    /// Seen close to: everything.
    pub fn near(&self) -> bool {
        self.w.detail == Detail::Near
    }

    /// Built before brick: stone, timber and stone slates, and water for power where there is any.
    pub fn early(&self) -> bool {
        self.w.founded < 1790
    }

    /// Built in the age of iron and glass.
    pub fn late(&self) -> bool {
        self.w.founded >= 1840
    }

    /// A wheel turns it: an early works by water.
    pub fn water_power(&self) -> bool {
        self.early() && self.w.water
    }

    /// The next of the works' own fixed numbers, from 0 to 1. (A part takes
    /// as many of these at every level of detail, so that the works is the
    /// same works at each.)
    pub fn rnd(&mut self) -> f32 {
        self.count += 1;
        hash2(self.count, (self.w.seed.u * 9973.0) as i32, 31 + (self.w.seed.v * 977.0) as i32)
    }

    /// How many of its own numbers the works has taken.
    pub fn taken(&self) -> i32 {
        self.count
    }

    /// How many of a thing `unit` long go into `room`, from one to `most`.
    pub fn fit(&self, room: f32, unit: f32, most: usize) -> usize {
        ((room / unit.max(1e-4)).floor() as usize).clamp(1, most.max(1))
    }

    /// How to fill `room` with things in a line, `gap` apart and none longer
    /// than `longest`: how many, and how long each (they share the room evenly).
    pub fn spread(&self, room: f32, longest: f32, gap: f32, most: usize) -> (usize, f32) {
        let n = (((room + gap) / (longest + gap)).ceil() as usize).clamp(1, most.max(1));
        (n, ((room - gap * (n as f32 - 1.0)) / n as f32).min(longest).max(0.02))
    }

    /// The height of walls of so many storeys.
    pub fn storeys(&self, n: usize) -> f32 {
        n as f32 * STOREY
    }

    /// A colour as it weathers: dulled and greened once the works is derelict.
    pub fn tone(&self, c: Rgba) -> Rgba {
        // an old part is a little sooted, as a roof is a little mossed
        let c = if self.worn > 0.0 { lerp(c, *self.pal.smoke.soot, 0.07 * self.worn) } else { c };
        if self.derelict() { shade(lerp(c, *self.pal.ground.yard_weeds, 0.22), 0.88) } else { c }
    }

    /// The walls of the period: stone early, then brick, sooty for the trades that blacken it.
    pub fn masonry(&self, sooty: bool) -> Rgba {
        self.tone(if self.early() { *self.pal.stone.works } else if sooty { *self.pal.brick.dark } else { *self.pal.brick.red })
    }

    /// The roof of the period: stone slates early, then Welsh slate.
    pub fn roofing(&self) -> Rgba {
        self.tone(if self.early() { *self.pal.roofs.stone_slate } else { *self.pal.slate.works })
    }

    /// How the period's covering lies: its oversail at the eaves and the verge, and the thickness of its edge.
    fn lie(&self) -> Lie {
        if self.early() { Cover::StoneSlate.lie() } else { Cover::Slate.lie() }
    }

    /// The point on the ground at a place in the yard.
    pub fn at(&self, x: f32, z: f32) -> Vec3 {
        self.site.at(x, z)
    }

    /// The wall round a yard, in the stone or the brick of the period.
    pub fn yard_wall(&self) -> Rgba {
        if self.early() { *self.pal.stone.yard_wall } else { lerp(*self.pal.brick.dark, *self.pal.smoke.soot, 0.3) }
    }

    /// A point that may be the works' highest: the family's flag flies from the highest.
    pub fn note_peak(&mut self, p: Vec3) {
        if self.peak.map_or(true, |q| p.y > q.y) {
            self.peak = Some(p);
        }
    }

    // -------------------------------------------------------------- buildings

    #[allow(clippy::too_many_arguments)]
    fn frame(&self, x: f32, z: f32, len: f32, dep: f32, h: f32, rise: f32, across: bool) -> Shed {
        let yaw = self.site.yaw + if across { std::f32::consts::FRAC_PI_2 } else { 0.0 };
        Shed { x, z, len, dep, h, rise, across, id: usize::MAX, dip: 0.0, o: self.site.at(x, z), q: Quat::from_rotation_y(yaw), yaw }
    }

    /// Build with the houses' kit on a shed: the kit stands at the shed's
    /// middle, square to the yard, and is handed the shed as a range.
    fn with_kit<R>(&mut self, s: &Shed, build: impl FnOnce(&mut Kit, &Range) -> R) -> R {
        let site = self.site.moved(s.o, self.site.yaw);
        let mut k = Kit::new(&mut *self.solid, &site, self.pal, self.w.detail, 0.0);
        if s.dip > 0.0 {
            k.crooked((0.0, 0.0), s.dip, s.len * 0.5 + 0.002, s.h, s.h + s.rise);
        }
        build(&mut k, &s.range())
    }

    /// What a wall of this colour is made of: a plinth a shade darker, painted frames, stone sills.
    fn fabric(&self, col: Rgba) -> Wall {
        Wall { col, plinth: Some(shade(col, 0.84)), frame: self.tone(*self.pal.walls.window_frame), sill: self.tone(*self.pal.stone.sill) }
    }

    /// Stand a building's walls, to be built when all that hangs on them is known.
    fn raise(&mut self, s: &mut Shed, wall: Rgba, top: f32, eaves: bool, lean: Option<(f32, f32, Rgba)>) {
        s.id = self.pending.len();
        self.pending.push(Pending { s: *s, wall: self.fabric(wall), top, ops: Default::default(), fill: Fill::Windows, eaves, lean, free: false });
    }

    /// Say what a shed's bare walls are given: a boarded shed has the posts
    /// of its frame, and no plinth.
    pub fn fill(&mut self, s: &Shed, fill: Fill) {
        if let Some(p) = self.pending.get_mut(s.id) {
            p.fill = fill;
            if fill == Fill::Posts {
                p.wall.plinth = None;
            }
        }
    }

    /// One slope of a pitched roof on a derelict shed, from `x0` to `x1` along the ridge.
    fn slope(k: &mut Kit, rg: &Range, s: &Shed, lie: &Lie, x0: f32, x1: f32, front: bool, col: Rgba) {
        let (hd, pitch) = (s.dep * 0.5, s.rise / (s.dep * 0.5));
        let (e, ce, top) = (s.h - lie.eaves * pitch + 0.0005, hd + lie.eaves, s.h + s.rise + 0.0005);
        let side = if front { 1.0 } else { -1.0 };
        let p = [k.rp(rg, x0, e, side * ce), k.rp(rg, x1, e, side * ce), k.rp(rg, x1, top, 0.0), k.rp(rg, x0, top, 0.0)];
        k.quad(p, if front { col } else { shade(col, 0.82) }, Vec3::Y);
    }

    /// A pitched roof: whole on a works that stands, none on one going up,
    /// and with slates gone in patches on a derelict one.
    fn pitched(&mut self, s: &Shed, roof: Rgba, gable: Rgba) {
        if self.going_up() {
            return;
        }
        let lie = self.lie();
        if !self.derelict() {
            let spec = RoofSpec { h: s.h, rise: s.rise, eaves: lie.eaves, verge: lie.verge, ends: [End::Gable, End::Gable], col: roof, gable, thick: lie.thick };
            self.with_kit(s, |k, rg| k.roof(rg, &spec));
            return;
        }
        let n = ((s.len / (2.0 * BAY)).ceil() as usize).clamp(2, 10);
        let gone: Vec<bool> = (0..2 * n).map(|_| self.rnd() < 0.42).collect();
        let s = *s;
        self.with_kit(&s, |k, rg| {
            let (hl, hd, top) = (s.len * 0.5, s.dep * 0.5, s.h + s.rise - 0.0006);
            let along = (k.rp(rg, 1.0, 0.0, 0.0) - k.rp(rg, 0.0, 0.0, 0.0)).normalize_or_zero();
            for (sign, shade_k) in [(1.0f32, 0.93), (-1.0, 0.95)] {
                let p = [k.rp(rg, sign * hl, s.h, hd), k.rp(rg, sign * hl, s.h, -hd), k.rp(rg, sign * hl, top, 0.0)];
                k.solid.tri_toward(p, [shade(gable, shade_k); 3], along * sign);
            }
            for (i, &g) in gone.iter().enumerate() {
                if g {
                    continue;
                }
                let (front, i) = (i < n, i % n);
                let (x0, x1) = (-hl - lie.verge + (s.len + 2.0 * lie.verge) * i as f32 / n as f32, -hl - lie.verge + (s.len + 2.0 * lie.verge) * (i + 1) as f32 / n as f32);
                Self::slope(k, rg, &s, &lie, x0, x1, front, roof);
            }
        });
    }

    /// A shed under a pitched roof, its ridge along the front or (`across`)
    /// from front to back: `len` along the ridge, `dep` across it, walls `h` high.
    #[allow(clippy::too_many_arguments)]
    pub fn shed(&mut self, x: f32, z: f32, len: f32, dep: f32, h: f32, across: bool, wall: Rgba, roof: Rgba) -> Shed {
        self.shed_pitched(x, z, len, dep, h, 0.32, across, wall, roof)
    }

    /// A shed whose roof rises `pitch` of its depth: 0.32 is a works' roof, 0.55 a cottage's.
    #[allow(clippy::too_many_arguments)]
    pub fn shed_pitched(&mut self, x: f32, z: f32, len: f32, dep: f32, h: f32, pitch: f32, across: bool, wall: Rgba, roof: Rgba) -> Shed {
        let mut s = self.frame(x, z, len, dep, h, dep * pitch, across);
        // an old roof sags a little between its gables
        let sag = self.rnd();
        if self.early() && !across && !self.going_up() && !self.derelict() {
            s.dip = (0.0006 + 0.0012 * sag) * (len / 0.08).min(1.0);
        }
        self.raise(&mut s, wall, h, true, None);
        self.pitched(&s, roof, wall);
        if !self.going_up() {
            // the gable end nearer the front, or the one to the right
            self.note_peak(s.pt(if across { -len * 0.5 } else { len * 0.5 }, s.h + s.rise, 0.0));
        }
        self.sheds.push(s);
        s
    }

    /// A lean-to: a single slope down from `h_back` on the shed's own -z to
    /// `h_front` on its +z. Openings may be hung on its front.
    #[allow(clippy::too_many_arguments)]
    pub fn lean_to(&mut self, x: f32, z: f32, len: f32, dep: f32, h_front: f32, h_back: f32, across: bool, wall: Rgba, roof: Rgba) -> Shed {
        let mut s = self.frame(x, z, len, dep, h_front, 0.0, across);
        self.raise(&mut s, wall, h_front, true, Some((h_front, h_back, roof)));
        s
    }

    /// A lean-to that stands by itself and not against a taller building: it has a back wall of its own.
    pub fn stands_free(&mut self, s: &Shed) {
        if let Some(p) = self.pending.get_mut(s.id) {
            p.free = true;
        }
    }

    /// A roof on posts with no walls: a drying shed, a sawing shed.
    #[allow(clippy::too_many_arguments)]
    pub fn open_shed(&mut self, x: f32, z: f32, len: f32, dep: f32, h: f32, across: bool, post: Rgba, roof: Rgba) -> Shed {
        let s = self.frame(x, z, len, dep, h, dep * 0.3, across);
        if !self.far() {
            let n = ((len / (2.0 * BAY)).ceil() as usize).clamp(1, 6);
            let t = if self.near() { 0.0018 } else { 0.0028 };
            for side in [-1.0f32, 1.0] {
                for i in 0..=n {
                    let px = -len * 0.5 + 0.002 + (len - 0.004) * i as f32 / n as f32;
                    let foot = s.pt(px, 0.0, side * (dep * 0.5 - 0.002));
                    self.solid.beam(foot - Vec3::Y * SINK * 0.5, foot + Vec3::Y * h, t, post);
                    if self.near() && i < n {
                        // the plate the roof sits on
                        let next = s.pt(-len * 0.5 + 0.002 + (len - 0.004) * (i + 1) as f32 / n as f32, h - 0.001, side * (dep * 0.5 - 0.002));
                        self.solid.beam(foot + Vec3::Y * (h - 0.001), next, 0.0014, shade(post, 0.9));
                    }
                }
            }
        }
        self.pitched(&s, roof, post);
        s
    }

    /// A saw-tooth roof on four walls: `teeth` north lights side by side
    /// along the shed's length, each a glazed upright face and a slated
    /// slope, their ridges running from front to back.
    #[allow(clippy::too_many_arguments)]
    pub fn sawtooth(&mut self, x: f32, z: f32, len: f32, dep: f32, h: f32, teeth: usize, across: bool, wall: Rgba, roof: Rgba) -> Shed {
        let pitch = len / teeth.max(1) as f32;
        let rise = pitch * 0.55;
        let mut s = self.frame(x, z, len, dep, h, rise, across);
        self.raise(&mut s, wall, h, false, None);
        self.sheds.push(s);
        if self.going_up() {
            return s;
        }
        let glass = self.tone(*self.pal.glass.roof_light);
        let frame = self.tone(*self.pal.walls.window_frame);
        let hd = dep * 0.5;
        let near = self.near();
        let before = self.solid.triangles();
        for i in 0..teeth {
            if self.derelict() && self.rnd() < 0.4 {
                continue;
            }
            let (x0, x1) = (-len * 0.5 + pitch * i as f32, -len * 0.5 + pitch * (i + 1) as f32);
            let top = h + rise;
            // the glass looks along the shed's own +x, the slates slope up to it from behind
            self.solid.quad(s.pt(x1, h, hd), s.pt(x1, h, -hd), s.pt(x1, top, -hd), s.pt(x1, top, hd), glass);
            self.solid.quad(s.pt(x0, h, hd), s.pt(x1, top, hd), s.pt(x1, top, -hd), s.pt(x0, h, -hd), roof);
            self.solid.tri(s.pt(x0, h, hd), s.pt(x1, h, hd), s.pt(x1, top, hd), wall);
            self.solid.tri(s.pt(x1, h, -hd), s.pt(x0, h, -hd), s.pt(x1, top, -hd), shade(wall, 0.86));
            if near {
                // the bars of the lights, a bay apart, and the rail they stand on
                let bars = ((dep / BAY).round() as usize).max(2);
                let (o, t) = (0.0005, 0.0004);
                for b in 1..bars {
                    let z = -hd + dep * b as f32 / bars as f32;
                    self.solid.quad(s.pt(x1 + o, h, z + t), s.pt(x1 + o, h, z - t), s.pt(x1 + o, top, z - t), s.pt(x1 + o, top, z + t), frame);
                }
                let mid = h + rise * 0.5;
                self.solid.quad(s.pt(x1 + o, mid - t, hd), s.pt(x1 + o, mid - t, -hd), s.pt(x1 + o, mid + t, -hd), s.pt(x1 + o, mid + t, hd), frame);
            }
        }
        crate::scale::tally_roof(len * dep * 1.3, self.solid.triangles() - before);
        self.note_peak(s.pt(len * 0.5, h + rise, hd));
        s
    }

    /// A flat-topped block: a bank of coke ovens, a platform. Openings may be hung on it.
    #[allow(clippy::too_many_arguments)]
    pub fn bank(&mut self, x: f32, z: f32, len: f32, dep: f32, h: f32, across: bool, wall: Rgba, top: Rgba) -> Shed {
        let mut s = self.frame(x, z, len, dep, h, 0.0, across);
        let h = if self.going_up() { h * 0.55 } else { h };
        self.raise(&mut s, wall, h, false, None);
        self.fill(&s, Fill::Nothing);
        let (hl, hd) = (len * 0.5, dep * 0.5);
        self.solid.quad(s.pt(-hl, h, hd), s.pt(hl, h, hd), s.pt(hl, h, -hd), s.pt(-hl, h, -hd), top);
        s
    }

    /// A square tower under a pyramid roof: a mill's stair tower, an engine house.
    pub fn tower(&mut self, x: f32, z: f32, side: f32, h: f32, wall: Rgba, roof: Rgba) -> Shed {
        let mut s = self.frame(x, z, side, side, h, side * 0.45, false);
        self.raise(&mut s, wall, h, true, None);
        if !self.going_up() && !self.derelict() {
            let lie = self.lie();
            let turn = std::f32::consts::FRAC_PI_4 - s.yaw;
            let pitch = s.rise / (side * 0.5);
            self.solid.frustum_turned(s.o + Vec3::Y * (h - lie.eaves * pitch + 0.0005), (side * 0.5 + lie.eaves) * std::f32::consts::SQRT_2, 0.0, s.rise + lie.eaves * pitch, 4, turn, 0.0, roof, None);
            crate::scale::tally_roof(side * side * (1.0 + pitch * pitch).sqrt(), 4);
            self.note_peak(s.pt(0.0, h + s.rise, 0.0));
        }
        self.sheds.push(s);
        s
    }

    /// A vent along the ridge of a shed, over `part` of its length: louvred
    /// (dark) to let the heat out, or a glazed lantern to let light in.
    pub fn ridge_vent(&mut self, s: &Shed, part: f32, glazed: bool) {
        if self.going_up() || self.derelict() || self.far() {
            return;
        }
        let before = self.solid.triangles();
        let (w, d) = (s.len * part, (s.dep * 0.2).max(0.005));
        let side = if glazed { *self.pal.glass.roof_light } else { *self.pal.smoke.soot };
        let base = s.pt(0.0, s.h + s.rise - d * 0.42, 0.0);
        let up = d * 0.42 + 0.0035;
        self.solid.block(base, w, d, up, s.yaw, side, None, 0.0);
        let top = if glazed { side } else { self.roofing() };
        self.solid.gable(base, w, d, up, d * 0.3, s.yaw, 0.001, top, side);
        if self.near() {
            // the slats of the louvres, or the bars of the lantern, on both sides
            let (col, y0) = (if glazed { self.tone(*self.pal.walls.window_frame) } else { shade(top, 0.8) }, s.h + s.rise + 0.0008);
            let n = ((w / BAY).round() as usize).max(1);
            for sz in [1.0f32, -1.0] {
                let z = sz * (d * 0.5 + 0.0004);
                let run = |c: &mut Self, x0: f32, x1: f32, ya: f32, yb: f32| {
                    let p = [s.pt(x0, ya, z), s.pt(x1, ya, z), s.pt(x1, yb, z), s.pt(x0, yb, z)];
                    if sz > 0.0 { c.solid.quad(p[0], p[1], p[2], p[3], col) } else { c.solid.quad(p[1], p[0], p[3], p[2], shade(col, 0.86)) }
                };
                if glazed {
                    for i in 0..=n {
                        let x = -w * 0.5 + w * i as f32 / n as f32;
                        run(self, x - 0.0004, x + 0.0004, y0, y0 + 0.0028);
                    }
                } else {
                    run(self, -w * 0.5, w * 0.5, y0 + 0.0004, y0 + 0.0011);
                    run(self, -w * 0.5, w * 0.5, y0 + 0.0017, y0 + 0.0024);
                }
            }
        }
        crate::scale::tally_roof(0.0, self.solid.triangles() - before);
    }

    /// Panes let into both slopes of a shed's roof, `n` along each.
    pub fn roof_lights(&mut self, s: &Shed, n: usize) {
        if self.going_up() || self.derelict() || self.far() {
            return;
        }
        let glass = *self.pal.glass.roof_light;
        let frame = self.tone(*self.pal.walls.window_frame);
        let at = |f: f32, side: f32, lift: f32| (s.h + s.rise * f + lift, side * s.dep * 0.5 * (1.0 - f));
        let before = self.solid.triangles();
        for i in 0..n {
            let cx = -s.len * 0.5 + s.len * (i as f32 + 0.5) / n as f32;
            let hw = (s.len / n as f32 * 0.3).min(BAY * 0.45);
            for side in [1.0f32, -1.0] {
                let ((y0, z0), (y1, z1)) = (at(0.25, side, 0.0012), at(0.75, side, 0.0012));
                let p = [s.pt(cx - hw, y0, z0), s.pt(cx + hw, y0, z0), s.pt(cx + hw, y1, z1), s.pt(cx - hw, y1, z1)];
                let col = if side > 0.0 { glass } else { shade(glass, 0.85) };
                if side > 0.0 { self.solid.quad(p[0], p[1], p[2], p[3], col) } else { self.solid.quad(p[1], p[0], p[3], p[2], col) }
                if self.near() {
                    // the bar down the middle of the light
                    let ((y0, z0), (y1, z1)) = (at(0.25, side, 0.0016), at(0.75, side, 0.0016));
                    let t = 0.0004;
                    let p = [s.pt(cx - t, y0, z0), s.pt(cx + t, y0, z0), s.pt(cx + t, y1, z1), s.pt(cx - t, y1, z1)];
                    if side > 0.0 { self.solid.quad(p[0], p[1], p[2], p[3], frame) } else { self.solid.quad(p[1], p[0], p[3], p[2], frame) }
                }
            }
        }
        crate::scale::tally_roof(0.0, self.solid.triangles() - before);
    }

    // -------------------------------------------------------------- openings

    /// An opening on a face of a shed, from `u0` to `u1` along the face (the
    /// shed's own x on its long walls, its own z on its ends) and `y0` to
    /// `y1` up it. It takes the place of any opening hung there before it.
    #[allow(clippy::too_many_arguments)]
    fn open(&mut self, s: &Shed, face: Face, u0: f32, u1: f32, y0: f32, y1: f32, kind: Open) {
        if self.going_up() || u1 - u0 < 1e-4 || y1 - y0 < 1e-4 {
            return;
        }
        let Some(p) = self.pending.get_mut(s.id) else { return };
        let mid = (u0 + u1) * 0.5;
        let (i, u, len) = match face {
            Face::Front => (0, mid + s.len * 0.5, s.len),
            Face::EndA => (1, s.dep * 0.5 - mid, s.dep),
            Face::Back => (2, s.len * 0.5 - mid, s.len),
            Face::EndB => (3, mid + s.dep * 0.5, s.dep),
        };
        // within the wall, clear of its corners and of its top
        let w = (u1 - u0).min(len - 0.002);
        let u = u.clamp(w * 0.5 + 0.001, (len - w * 0.5 - 0.001).max(w * 0.5 + 0.001));
        let top = if p.lean.is_some() { p.top - 0.0036 } else { p.top - 0.0034 };
        let (y0, y1) = (y0.max(0.0), y1.min(top));
        if y1 - y0 < 0.002 {
            return;
        }
        // an opening that would leave less than a pier between itself and its neighbour stands hard against it
        let (mut a, mut b) = (u - w * 0.5, u + w * 0.5);
        for e in &p.ops[i] {
            if e.y + e.h <= y0 || e.y >= y1 {
                continue;
            }
            let (ea, eb) = (e.u - e.w * 0.5, e.u + e.w * 0.5);
            if a > eb && a - eb < 0.0013 {
                a = eb;
            }
            if b < ea && ea - b < 0.0013 {
                b = ea;
            }
        }
        let o = Opening { u: (a + b) * 0.5, y: y0, w: b - a, h: y1 - y0, kind };
        let m = 1e-5;
        p.ops[i].retain(|e| e.u + e.w * 0.5 <= o.u - o.w * 0.5 + m || e.u - e.w * 0.5 >= o.u + o.w * 0.5 - m || e.y + e.h <= o.y + m || e.y >= o.y + o.h - m);
        p.ops[i].push(o);
    }

    /// An opening with nothing in it: an open bay, an arcade, a way through.
    #[allow(clippy::too_many_arguments)]
    pub fn mouth(&mut self, s: &Shed, face: Face, u0: f32, u1: f32, y0: f32, y1: f32, arch: bool) {
        let within = *self.pal.smoke.soot;
        self.open(s, face, u0, u1, y0, y1, Open::Mouth { within, round: arch });
    }

    /// Fire seen in an opening on a shed's face: lit only while the works is working.
    #[allow(clippy::too_many_arguments)]
    pub fn glow(&mut self, s: &Shed, face: Face, u0: f32, u1: f32, y0: f32, y1: f32, arch: bool) {
        let within = self.throat();
        self.open(s, face, u0, u1, y0, y1, Open::Mouth { within, round: arch });
    }

    /// A pair of boarded doors wide enough for a cart.
    #[allow(clippy::too_many_arguments)]
    pub fn doors(&mut self, s: &Shed, face: Face, u0: f32, u1: f32, y0: f32, y1: f32, arch: bool, paint: Rgba) {
        let paint = self.tone(paint);
        self.open(s, face, u0, u1, y0, y1, Open::Leaves { paint, round: arch });
    }

    /// A door for a man, boarded and plain.
    pub fn door(&mut self, s: &Shed, face: Face, u: f32) {
        let paint = self.tone(*self.pal.timber.door);
        self.open(s, face, u - 0.0026, u + 0.0026, 0.0, FOOT + 0.0124, Open::Door { paint, over: crate::kit::Over::Nothing });
    }

    /// One window of small panes, of any size; with `arch`, under a round head.
    #[allow(clippy::too_many_arguments)]
    pub fn light(&mut self, s: &Shed, face: Face, u0: f32, u1: f32, y0: f32, y1: f32, arch: bool) {
        let kind = self.paned(u1 - u0, y1 - y0, arch);
        self.open(s, face, u0, u1, y0, y1, kind);
    }

    /// A shop's front, as a house's shop has it: the window over its stall
    /// riser, the door at one end and the name board above, painted.
    pub fn shop_front(&mut self, s: &Shed, face: Face, u0: f32, u1: f32, paint: Rgba) {
        let paint = self.tone(paint);
        self.open(s, face, u0, u1, 0.0, FOOT + 0.0112, Open::Shop { paint, awning: None });
    }

    /// Slats in a wall, to let the heat out. (No kind uses it yet: it is here for the site planner.)
    #[allow(clippy::too_many_arguments, dead_code)]
    pub fn louvre(&mut self, s: &Shed, face: Face, u0: f32, u1: f32, y0: f32, y1: f32) {
        self.open(s, face, u0, u1, y0, y1, Open::Louvre);
    }

    /// A window of small panes for a hole of a size: its panes are about 0.003 square.
    fn paned(&self, w: f32, h: f32, round: bool) -> Open {
        Open::Paned { cols: ((w / 0.0032).round() as u8).clamp(1, 12), rows: ((h / 0.003).round() as u8).clamp(1, 12), round, lit: self.lit }
    }

    /// How many bays a wall of a length has: none if it is too short for a window.
    fn bays(len: f32) -> usize {
        if len < 0.8 * BAY { 0 } else { ((len / (1.25 * BAY)).round() as usize).max(1) }
    }

    /// The windows of storey `st` in the bay about `u` of a face, on a wall
    /// `top` high built in storeys; or, on a hall's wall, tall windows in tiers.
    #[allow(clippy::too_many_arguments)]
    fn bay_windows(&mut self, s: &Shed, face: Face, u: f32, top: f32, storeyed: bool, st: usize) {
        let (hw, wh) = (WINDOW.0 * 0.5, WINDOW.1);
        if storeyed {
            let y = st as f32 * STOREY + SILL;
            self.light(s, face, u - hw, u + hw, y, y + wh, false);
            return;
        }
        // a hall: one tall space, its windows as tall as the wall allows, in tiers where it is
        // very tall; a low wall's sit lower
        let sill = SILL.min(top * 0.36).max(FOOT + 0.0015);
        let room = top - sill - 0.0048;
        if room < 0.0035 {
            return;
        }
        let tiers = (room / (2.0 * STOREY)).ceil().max(1.0);
        let each = room / tiers;
        for t in 0..tiers as usize {
            let y = sill + each * t as f32;
            self.light(s, face, u - hw, u + hw, y, y + each - if t + 1 < tiers as usize { 0.004 } else { 0.0 }, false);
        }
    }

    /// Whether a wall is built in storeys (its height a whole number of them) and how many.
    fn storeyed(top: f32) -> (bool, usize) {
        let n = (top / STOREY).round().max(1.0);
        ((top - n * STOREY).abs() < 0.03 * STOREY, n as usize)
    }

    /// Rows of windows on both long walls of a shed, and on its gable ends
    /// when `ends` is set, a bay apart: on the storeys from `from` up to (not
    /// counting) `upto` of walls built in storeys, or, on a hall's walls
    /// (`from` 0, `upto` 1), tall windows.
    pub fn windows(&mut self, s: &Shed, from: usize, upto: usize, ends: bool) {
        let Some(top) = self.pending.get(s.id).map(|p| p.top) else { return };
        let (storeyed, n) = Self::storeyed(top);
        let faces: &[(Face, f32)] = if ends { &[(Face::Front, s.len), (Face::Back, s.len), (Face::EndA, s.dep), (Face::EndB, s.dep)] } else { &[(Face::Front, s.len), (Face::Back, s.len)] };
        for &(face, len) in faces {
            let cols = Self::bays(len);
            for c in 0..cols {
                let u = -len * 0.5 + len * (c as f32 + 0.5) / cols as f32;
                if storeyed {
                    for st in from..upto.min(n) {
                        self.bay_windows(s, face, u, top, true, st);
                    }
                } else if from == 0 {
                    self.bay_windows(s, face, u, top, false, 0);
                }
            }
        }
    }

    /// One long band of glass along both long walls of a shed, at `y`: a weaver's window.
    pub fn window_band(&mut self, s: &Shed, y: f32, tall: f32) {
        for face in [Face::Front, Face::Back] {
            self.light(s, face, -s.len * 0.44, s.len * 0.44, y, y + tall, false);
        }
    }

    /// A painted board on a face of a shed (a name board, a sign), from `u0` to `u1` and `y0` to `y1`.
    /// (No kind uses it yet: it is here for the site planner.)
    #[allow(clippy::too_many_arguments, dead_code)]
    pub fn board(&mut self, s: &Shed, face: Face, u0: f32, u1: f32, y0: f32, y1: f32, col: Rgba) {
        if self.going_up() || self.far() {
            return;
        }
        let (i, a, b) = match face {
            Face::Front => (0, u0 + s.len * 0.5, u1 + s.len * 0.5),
            Face::EndA => (1, s.dep * 0.5 - u1, s.dep * 0.5 - u0),
            Face::Back => (2, s.len * 0.5 - u1, s.len * 0.5 - u0),
            Face::EndB => (3, u0 + s.dep * 0.5, u1 + s.dep * 0.5),
        };
        let s = *s;
        self.with_kit(&s, |k, rg| k.strip(rg, i, a, b, y0, y1, 0.0009, col));
    }

    /// A hoist: loading doors one above another up a face of a shed at `u`,
    /// from storey `from` to the top storey, under a beam that juts from
    /// the wall beneath a little hood. A warehouse's.
    pub fn hoist(&mut self, s: &Shed, face: Face, u: f32, from: usize) {
        let Some(top) = self.pending.get(s.id).map(|p| p.top) else { return };
        let (storeyed, n) = Self::storeyed(top);
        if !storeyed || self.going_up() {
            return;
        }
        let paint = *self.pal.timber.door;
        for st in from..n {
            let y = st as f32 * STOREY + if st == 0 { 0.0 } else { FOOT };
            self.doors(s, face, u - 0.0042, u + 0.0042, y, st as f32 * STOREY + 0.0156, false, paint);
        }
        if self.far() || self.derelict() {
            return;
        }
        // the beam and its hood, above the top door
        let (out, sign) = match face {
            Face::Front => (s.dep * 0.5, 1.0),
            Face::Back => (s.dep * 0.5, -1.0),
            Face::EndA => (s.len * 0.5, 1.0),
            Face::EndB => (s.len * 0.5, -1.0),
        };
        let at = |along: f32, y: f32, off: f32| if matches!(face, Face::Front | Face::Back) { s.pt(along, y, sign * (out + off)) } else { s.pt(sign * (out + off), y, along) };
        let y = top - 0.0026;
        let iron = self.tone(*self.pal.iron.plate);
        self.solid.beam(at(u, y, -0.001), at(u, y, 0.0062), if self.near() { 0.0012 } else { 0.0016 }, iron);
        if self.near() {
            // the hood over the beam, and the rope down from it
            let roof = self.roofing();
            let (a, b, c2, d) = (at(u - 0.004, y + 0.003, 0.0), at(u + 0.004, y + 0.003, 0.0), at(u + 0.004, y + 0.0012, 0.0066), at(u - 0.004, y + 0.0012, 0.0066));
            self.solid.quad_toward([a, b, c2, d], [roof; 4], Vec3::Y);
            self.solid.beam(at(u, y, 0.0052), at(u, y - 0.009, 0.0052), 0.0005, shade(iron, 0.8));
        }
    }

    // -------------------------------------------------------------- chimneys and smoke

    /// A tall round chimney on a square foot, with its smoke; `puff` is how
    /// much smoke it makes (1 for a mill's). Close to it has its plinth
    /// and cornice, iron bands round its shaft and a corbelled head.
    pub fn chimney(&mut self, x: f32, z: f32, h: f32, r: f32, puff: f32) {
        let p = self.at(x, z);
        let brick = self.tone(*self.pal.brick.chimney);
        let (h, whole) = match self.w.state {
            State::GoingUp => (h * 0.4, false),
            State::Derelict => (h * 0.72, false),
            _ => (h, true),
        };
        let soot = *self.pal.smoke.soot;
        let turn = std::f32::consts::FRAC_PI_4 - self.site.yaw;
        if self.far() {
            self.solid.frustum_turned(p, r * 1.1, r * 0.62, h, 5, 0.0, SINK, brick, Some(soot));
        } else {
            let (foot, sides) = (h * 0.1, if self.near() { 8 } else { 6 });
            self.solid.frustum_turned(p, r * 1.75, r * 1.75, foot, 4, turn, SINK, shade(brick, 0.94), Some(shade(brick, 0.8)));
            self.solid.frustum_turned(p + Vec3::Y * foot, r, r * 0.62, h * 0.9, sides, 0.0, 0.0, brick, Some(soot));
            // how wide the shaft is at a share of its height
            let at = |t: f32| r * (1.0 - 0.38 * t);
            if self.near() {
                // the cornice of the plinth
                self.solid.frustum_turned(p + Vec3::Y * (foot - 0.0012), r * 1.9, r * 1.9, 0.0012, 4, turn, 0.0, shade(brick, 1.04), Some(shade(brick, 0.86)));
                // iron bands to hold the shaft together
                let iron = self.tone(*self.pal.iron.plate);
                for t in [0.3, 0.55, 0.78] {
                    self.solid.frustum_turned(p + Vec3::Y * (foot + h * 0.9 * t), at(t) + 0.0005, at(t + 0.01) + 0.0005, 0.0011, sides, 0.0, 0.0, iron, None);
                }
            }
            if whole {
                // the oversailing courses at the top: one ring from the play distance, two close to
                self.solid.frustum_turned(p + Vec3::Y * h * 0.95, r * 0.8, r * 0.8, h * 0.035, sides, 0.0, 0.0, shade(brick, 0.86), None);
                if self.near() {
                    self.solid.frustum_turned(p + Vec3::Y * (h * 0.985 - 0.0012), r * 0.9, r * 0.9, 0.0012, sides, 0.0, 0.0, shade(brick, 0.94), Some(soot));
                }
            }
        }
        self.plume(p + Vec3::Y * h, puff);
    }

    /// A short square stack, as a furnace or a hearth has, standing square
    /// to the works: `side` across, with a wisp of smoke if `puff` is more than nothing.
    pub fn stack(&mut self, x: f32, z: f32, h: f32, side: f32, col: Rgba, puff: f32) {
        let p = self.at(x, z);
        let h = match self.w.state {
            State::GoingUp => h * 0.45,
            State::Derelict => h * (0.6 + 0.3 * self.rnd()),
            _ => h,
        };
        let turn = std::f32::consts::FRAC_PI_4 - self.site.yaw;
        let soot = *self.pal.smoke.soot;
        if self.near() && self.w.state != State::GoingUp {
            // its head: a course that oversails, as a house's stack has
            let neck = h - 0.002;
            self.solid.frustum_turned(p, side * 0.707, side * 0.61, neck, 4, turn, SINK, col, None);
            self.solid.frustum_turned(p + Vec3::Y * neck, side * 0.6 + 0.001, side * 0.6 + 0.001, 0.002, 4, turn, 0.0, shade(col, 0.9), Some(soot));
        } else {
            self.solid.frustum_turned(p, side * 0.707, side * 0.6, h, 4, turn, SINK, col, Some(soot));
        }
        if puff > 0.0 {
            self.plume(p + Vec3::Y * h, puff);
        }
    }

    /// A broad slab of a stack standing against a shed, at a place in the
    /// shed's own frame: `w` along the shed and `d` through, as the flues of
    /// a row of melting holes are gathered into one.
    #[allow(clippy::too_many_arguments)]
    pub fn slab_stack(&mut self, s: &Shed, sx: f32, sz: f32, w: f32, d: f32, h: f32, col: Rgba, puff: f32) {
        let h = match self.w.state {
            State::GoingUp => h * 0.45,
            State::Derelict => h * (0.65 + 0.25 * self.rnd()),
            _ => h,
        };
        let p = s.pt(sx, 0.0, sz);
        let soot = *self.pal.smoke.soot;
        if self.near() && self.w.state != State::GoingUp {
            let neck = h - 0.002;
            self.solid.block(p, w, d, neck, s.yaw, col, None, SINK);
            self.solid.block(p + Vec3::Y * neck, w + 0.0014, d + 0.0014, 0.002, s.yaw, shade(col, 0.9), Some(soot), 0.0);
            if !self.derelict() {
                // the iron straps that hold the brickwork against the heat, above the shed's roof
                let iron = shade(*self.pal.iron.plate, 0.9);
                for k in [0.62, 0.84] {
                    self.solid.block(p + Vec3::Y * (h * k), w + 0.0008, d + 0.0008, 0.0016, s.yaw, iron, None, 0.0);
                }
            }
        } else {
            self.solid.block(p, w, d, h, s.yaw, col, Some(soot), SINK);
        }
        if puff > 0.0 {
            self.plume(p + Vec3::Y * h, puff);
        }
    }

    /// A plain round flue standing on the ground (a cupola, a stove, a
    /// furnace's shaft): `r0` at the foot, `r1` at the top, capped in `cap`.
    /// It gives back the middle of its top.
    #[allow(clippy::too_many_arguments)]
    pub fn flue(&mut self, x: f32, z: f32, h: f32, r0: f32, r1: f32, col: Rgba, cap: Rgba, puff: f32) -> Vec3 {
        let p = self.at(x, z);
        let h = match self.w.state {
            State::GoingUp => h * 0.45,
            State::Derelict => h * 0.8,
            _ => h,
        };
        let sides = if self.near() { 8 } else if self.far() { 5 } else { 6 };
        self.solid.frustum_turned(p, r0, r1, h, sides, 0.0, SINK, col, Some(cap));
        if self.near() && self.w.state != State::GoingUp {
            // a band at its head
            self.solid.frustum_turned(p + Vec3::Y * (h - 0.0022), r1 + 0.0006, r1 + 0.0006, 0.0014, sides, 0.0, 0.0, shade(col, 0.82), None);
        }
        if puff > 0.0 {
            self.plume(p + Vec3::Y * h, puff);
        }
        p + Vec3::Y * h
    }

    /// Smoke from a flue whose top is at `top`: a soft plume that rises, leans
    /// downwind, widens and thins to nothing. Only a works that is working smokes.
    pub fn plume(&mut self, top: Vec3, scale: f32) {
        if !self.busy() || scale <= 0.0 {
            return;
        }
        let phase = self.rnd() * std::f32::consts::TAU;
        plume(self.smoke, top, scale, phase, *self.pal.smoke.puff);
    }

    /// The colour of a furnace's open throat: fire while it is in blast, soot otherwise.
    pub fn throat(&self) -> Rgba {
        if self.busy() { shade(*self.pal.iron.glow, 1.6) } else { *self.pal.smoke.soot }
    }

    // -------------------------------------------------------------- what stands in the yard

    /// A heap of something loose (coal, ore, slag), `r` across at the foot.
    /// From far off only the great heaps are drawn; close to a heap has a shoulder.
    pub fn heap(&mut self, x: f32, z: f32, r: f32, h: f32, col: Rgba) {
        let turn = self.rnd() * 3.0;
        if self.far() && r < 0.02 {
            return;
        }
        let sides = if self.near() { 7 } else { 5 };
        // (a small heap comes to a point: the flat of its top would be finer than the level draws)
        let top = if self.near() || r * 0.28 >= 0.004 { r * 0.28 } else { 0.0 };
        self.solid.frustum_turned(self.at(x, z), r, top, h, sides, turn, SINK * 0.5, col, Some(shade(col, 1.08)));
        if self.near() && r > 0.006 {
            let (dx, dz) = (turn.cos() * r * 0.55, turn.sin() * r * 0.55);
            self.solid.frustum_turned(self.at(x + dx, z + dz), r * 0.5, 0.0, h * 0.55, 5, turn + 1.0, SINK * 0.5, shade(col, 0.94), None);
        }
    }

    /// A square stack of something made (bricks, bales, planks, pig iron):
    /// `len` along the front (or from front to back, with `across`).
    /// Not drawn from far off; close to it is laid in two courses.
    #[allow(clippy::too_many_arguments)]
    pub fn pile(&mut self, x: f32, z: f32, len: f32, dep: f32, h: f32, across: bool, col: Rgba) {
        if self.far() {
            return;
        }
        let yaw = self.site.yaw + if across { std::f32::consts::FRAC_PI_2 } else { 0.0 };
        let p = self.at(x, z);
        if self.near() && h > 0.0036 && len.min(dep) > 0.004 {
            let low = h * 0.55;
            self.solid.block(p, len, dep, low, yaw, shade(col, 0.94), Some(shade(col, 0.8)), SINK * 0.5);
            self.solid.block(p + Vec3::Y * low, len - 0.0012, dep - 0.0012, h - low, yaw, col, Some(shade(col, 1.08)), 0.0);
        } else {
            self.solid.block(p, len, dep, h, yaw, col, Some(shade(col, 1.08)), SINK * 0.5);
        }
    }

    /// Goods stacked in rows over a patch of the yard, `cols` by `rows`, only
    /// while the works is working (and fewer while it is being enlarged).
    #[allow(clippy::too_many_arguments)]
    pub fn stacks(&mut self, x: f32, z: f32, w: f32, d: f32, cols: usize, rows: usize, h: f32, col: Rgba) {
        if !self.busy() {
            return;
        }
        let (pw, pd) = (w / cols as f32, d / rows as f32);
        for j in 0..rows {
            for i in 0..cols {
                // a stack or two is always away at the customer's
                if self.rnd() < 0.18 {
                    continue;
                }
                let (px, pz) = (x - w * 0.5 + pw * (i as f32 + 0.5), z - d * 0.5 + pd * (j as f32 + 0.5));
                let hh = h * (0.6 + 0.4 * self.rnd());
                self.pile(px, pz, pw * 0.78, pd * 0.72, hh, false, col);
            }
        }
    }

    /// A waterwheel standing at a place in the yard, `r` in radius, seen
    /// face on from the front (or from the side, with `across`); in iron for
    /// a flywheel. Not drawn from far off.
    pub fn wheel(&mut self, x: f32, z: f32, r: f32, width: f32, across: bool, col: Rgba) {
        if self.far() {
            return;
        }
        let s = self.frame(x, z, 0.0, 0.0, 0.0, 0.0, across);
        let cy = r * 0.82;
        let n: usize = if self.near() { 12 } else { 8 };
        let rim = |i: usize, z: f32| {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            s.pt(a.cos() * r, cy + a.sin() * r, z)
        };
        let hz = width * 0.5;
        for i in 0..n {
            let j = (i + 1) % n;
            let k = 0.8 + 0.2 * (i as f32 / n as f32 * std::f32::consts::TAU).sin();
            self.solid.quad(rim(i, hz), rim(i, -hz), rim(j, -hz), rim(j, hz), shade(col, k));
            self.solid.tri(s.pt(0.0, cy, hz), rim(i, hz), rim(j, hz), shade(col, if i % 2 == 0 { 1.0 } else { 0.86 }));
            self.solid.tri(s.pt(0.0, cy, -hz), rim(j, -hz), rim(i, -hz), shade(col, if i % 2 == 0 { 0.8 } else { 0.7 }));
        }
        if self.near() {
            // its axle
            self.solid.beam(s.pt(0.0, cy, -hz - 0.0016), s.pt(0.0, cy, hz + 0.0016), (r * 0.16).max(0.0012), shade(col, 0.6));
        }
    }

    /// Water led across the yard: a mill race from the edge to the wheel.
    pub fn race(&mut self, from: (f32, f32), to: (f32, f32), width: f32) {
        let n = (((from.0 - to.0).hypot(from.1 - to.1) / GROUND_STEP).ceil() as usize).max(1);
        let pts: Vec<Vec3> = (0..=n)
            .map(|i| {
                let t = i as f32 / n as f32;
                self.at(from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t) + Vec3::Y * (LIFT + 0.002)
            })
            .collect();
        self.flat.ribbon(&pts, width, *self.pal.water.river);
    }

    /// Three legs and a jib: a crane in the yard.
    pub fn sheer_legs(&mut self, x: f32, z: f32, h: f32) {
        if self.far() {
            return;
        }
        let col = self.tone(*self.pal.timber.headframe);
        let top = self.at(x, z) + Vec3::Y * h;
        for (dx, dz) in [(-0.3, -0.25), (0.3, -0.25), (0.0, 0.35)] {
            self.solid.beam(self.at(x + dx * h, z + dz * h), top, (h * 0.07).min(if self.near() { 0.0022 } else { 0.004 }), col);
        }
        if self.near() {
            // the rope, and what hangs on it
            self.solid.beam(top, top - Vec3::Y * h * 0.55, 0.0005, shade(col, 0.6));
        }
    }

    // -------------------------------------------------------------- the yard

    /// The yard's own ground over the whole footprint (or, with `round`,
    /// over the ellipse within it), following the land.
    pub fn ground(&mut self, col: Rgba, round: bool) {
        if self.planned {
            self.asked_ground = Some(col);
            return;
        }
        let col = if self.derelict() { lerp(col, *self.pal.ground.yard_weeds, 0.65) } else { col };
        let up = Vec3::Y * LIFT;
        if self.site.is_level() {
            if round {
                let mid = self.at(0.0, 0.0) + up;
                const N: usize = 12;
                let (site, hw, hd) = (self.site, self.hw, self.hd);
                let rim = |i: usize| {
                    let a = i as f32 / N as f32 * std::f32::consts::TAU;
                    site.at(a.cos() * hw, a.sin() * hd) + up
                };
                for i in 0..N {
                    self.flat.tri(mid, rim(i + 1), rim(i), col);
                }
            } else {
                self.flat.quad(self.at(-self.hw, self.hd) + up, self.at(self.hw, self.hd) + up, self.at(self.hw, -self.hd) + up, self.at(-self.hw, -self.hd) + up, col);
            }
            return;
        }
        let (nx, nz) = (((2.0 * self.hw / GROUND_STEP).ceil() as usize).max(1), ((2.0 * self.hd / GROUND_STEP).ceil() as usize).max(1));
        let (sx, sz) = (2.0 * self.hw / nx as f32, 2.0 * self.hd / nz as f32);
        for j in 0..nz {
            for i in 0..nx {
                let (x0, z0) = (-self.hw + sx * i as f32, -self.hd + sz * j as f32);
                let (mx, mz) = ((x0 + sx * 0.5) / self.hw, (z0 + sz * 0.5) / self.hd);
                if round && nx > 2 && nz > 2 && mx * mx + mz * mz > 1.0 {
                    continue;
                }
                self.flat.quad(self.at(x0, z0 + sz) + up, self.at(x0 + sx, z0 + sz) + up, self.at(x0 + sx, z0) + up, self.at(x0, z0) + up, col);
            }
        }
    }

    /// A patch of other ground within the yard (a clay pit, a hearth),
    /// `n`-sided; a patch of a higher `layer` lies over one of a lower.
    #[allow(clippy::too_many_arguments)]
    pub fn patch(&mut self, x: f32, z: f32, rx: f32, rz: f32, n: usize, layer: u8, col: Rgba) {
        let up = Vec3::Y * (LIFT + 0.0012 * (layer as f32 + 1.0));
        let turn = self.rnd() * 2.0;
        let mid = self.at(x, z) + up;
        let site = self.site;
        let rim = |i: usize| {
            let a = i as f32 / n as f32 * std::f32::consts::TAU + turn;
            site.at(x + a.cos() * rx, z + a.sin() * rz) + up
        };
        for i in 0..n {
            self.flat.tri(mid, rim(i + 1), rim(i), col);
        }
    }

    /// A run of wall or fence from one place in the yard to another,
    /// following the ground; close to, with a coping that oversails it.
    pub fn wall_run(&mut self, a: (f32, f32), b: (f32, f32), h: f32, t: f32, col: Rgba, coping: bool) {
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        if len < 1e-4 {
            return;
        }
        let n = if self.site.is_level() { 1 } else { ((len / 0.06).ceil() as usize).max(1) };
        let site = self.site;
        let at = |i: usize| {
            let f = i as f32 / n as f32;
            site.at(a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f)
        };
        let (p0, p1) = (at(0), at(n));
        let dir = Vec3::new(p1.x - p0.x, 0.0, p1.z - p0.z).normalize_or_zero();
        let o = dir.cross(Vec3::Y) * (t * 0.5);
        let (up, down) = (Vec3::Y * h, Vec3::Y * (SINK * 0.5));
        for i in 0..n {
            let gone = self.derelict() && self.rnd() < 0.3;
            if gone || self.far() {
                continue;
            }
            let (p, q) = (at(i), at(i + 1));
            // darker at its foot, as a house's wall is
            let foot = shade(col, 0.8);
            self.solid.quad_toward([p + o - down, q + o - down, q + o + up, p + o + up], [foot, foot, col, col], o);
            let back = shade(col, 0.84);
            self.solid.quad_toward([q - o - down, p - o - down, p - o + up, q - o + up], [shade(back, 0.8), shade(back, 0.8), back, back], -o);
            if coping && self.near() {
                let (c, lip) = (o * 1.35, Vec3::Y * 0.0009);
                let stone = shade(col, 1.12);
                self.solid.quad(p + c + up, q + c + up, q + c + up + lip, p + c + up + lip, shade(stone, 0.9));
                self.solid.quad(q - c + up, p - c + up, p - c + up + lip, q - c + up + lip, shade(stone, 0.8));
                self.solid.quad(p + c + up + lip, q + c + up + lip, q - c + up + lip, p - c + up + lip, stone);
            } else {
                self.solid.quad(p + o + up, q + o + up, q - o + up, p - o + up, shade(col, 1.03));
            }
        }
    }

    /// A pier of a wall or a gate at a place in the yard: square, and capped close to.
    pub fn pier(&mut self, x: f32, z: f32, side: f32, h: f32, col: Rgba) {
        let p = self.at(x, z);
        self.solid.block(p, side, side, h, self.site.yaw, shade(col, 0.95), Some(shade(col, 1.1)), SINK * 0.5);
        if self.near() {
            let turn = std::f32::consts::FRAC_PI_4 - self.site.yaw;
            self.solid.frustum_turned(p + Vec3::Y * h, (side * 0.5 + 0.0006) * std::f32::consts::SQRT_2, 0.0, side * 0.42, 4, turn, 0.0, shade(col, 1.14), None);
        }
    }

    /// The wall (or fence) round the yard, with a gate in the middle of the
    /// front `gate` wide: piers either side of it, the gate standing open
    /// while the works is at work and shut across it when it stands idle.
    /// Close to the wall has its coping and a pier at each corner. Nothing
    /// while the works is going up, and nothing from far off, where the
    /// yard's ground says where the works is.
    pub fn wall(&mut self, col: Rgba, h: f32, gate: f32) {
        if self.planned {
            self.asked_wall = Some((col, h));
            return;
        }
        if self.going_up() {
            return;
        }
        let col = self.tone(col);
        let masonry = h >= 0.0065;
        let t = if masonry { 0.0034 } else { 0.0016 };
        // its outer face stands at the edge of the yard's ground: a strip of ground left
        // outside it is a pixel wide from the nearest camera, and shows as a dotted line
        let (hw, hd) = (self.hw - 0.0002 - t * 0.5, self.hd - 0.0002 - t * 0.5);
        let gate = gate.min(hw * 1.2);
        self.wall_run((hw, -hd), (-hw, -hd), h, t, col, masonry);
        self.wall_run((-hw, -hd), (-hw, hd), h, t, col, masonry);
        self.wall_run((hw, hd), (hw, -hd), h, t, col, masonry);
        self.wall_run((-hw, hd), (-gate * 0.5, hd), h, t, col, masonry);
        self.wall_run((gate * 0.5, hd), (hw, hd), h, t, col, masonry);
        if self.far() {
            return;
        }
        if self.w.size >= 1.0 && !self.derelict() {
            for side in [-1.0f32, 1.0] {
                self.pier(side * gate * 0.5, hd, t * 1.9, h * 1.45, col);
            }
        }
        if masonry && self.near() && !self.derelict() {
            for (x, z) in [(-hw, -hd), (hw, -hd), (-hw, hd), (hw, hd)] {
                self.pier(x, z, t * 1.5, h * 1.15, col);
            }
        }
        let wood = self.tone(*self.pal.timber.door);
        if self.w.state == State::Idle {
            self.wall_run((-gate * 0.5, hd), (gate * 0.5, hd), h * 0.9, t * 0.5, wood, false);
        } else if self.near() && !self.derelict() {
            // the two leaves of the gate, swung back into the yard
            for side in [-1.0f32, 1.0] {
                let x = side * (gate * 0.5 - t);
                self.wall_run((x, hd - t), (x + side * gate * 0.08, hd - gate * 0.44), h * 0.9, 0.0007, wood, false);
            }
        }
    }

    /// Poles and boards round a building that is going up or being enlarged.
    pub fn scaffold(&mut self, s: &Shed) {
        if self.far() {
            return;
        }
        let (hw, hd) = (s.len * 0.5 + 0.004, s.dep * 0.5 + 0.004);
        let h = if self.going_up() { s.h * 0.55 + 0.012 } else { s.h + s.rise * 0.6 };
        let pole = *self.pal.timber.scaffold_pole;
        let board = *self.pal.timber.scaffold_board;
        let corners = [s.pt(-hw, 0.0, -hd), s.pt(hw, 0.0, -hd), s.pt(hw, 0.0, hd), s.pt(-hw, 0.0, hd)];
        let t = if self.near() { 0.0016 } else { (0.003 * self.g).min(0.0045) };
        let mut poles: Vec<Vec3> = corners.to_vec();
        // a pole every couple of bays along the long sides
        let between = if self.near() { ((s.len / (2.0 * BAY)).round() as usize).clamp(1, 8) } else if s.len > 0.05 { 2 } else { 1 };
        for i in 1..between {
            let x = -hw + 2.0 * hw * i as f32 / between as f32;
            poles.push(s.pt(x, 0.0, -hd));
            poles.push(s.pt(x, 0.0, hd));
        }
        for c in poles {
            self.solid.beam(c - Vec3::Y * 0.01, c + Vec3::Y * h, t, pole);
        }
        let lifts: &[f32] = if self.near() && h > 2.0 * STOREY { &[0.3, 0.6, 0.9] } else { &[0.45, 0.9] };
        for lift in lifts {
            let y = Vec3::Y * (h * lift);
            for i in 0..4 {
                self.solid.beam(corners[i] + y, corners[(i + 1) % 4] + y, t * 1.2, board);
            }
        }
        self.solid.beam(corners[3] + Vec3::Y * 0.004, corners[2] + Vec3::Y * (h * 0.9), t * 0.8, pole);
    }

    /// What lies about a works that is being built: a pile of timber and a heap of bricks.
    pub fn builders_yard(&mut self) {
        // (a part of a planned works has no yard of its own to lay timber about in)
        if self.planned {
            return;
        }
        let (x, z) = (self.hw * 0.5, self.hd * 0.72);
        let board = *self.pal.timber.scaffold_board;
        let len = (0.03 * self.g).min(self.hw * 0.7);
        self.pile(x, z, len, len * 0.4, len * 0.24, false, board);
        let brick = if self.early() { *self.pal.stone.works } else { *self.pal.goods.bricks };
        self.heap(x - len * 1.1, z, len * 0.36, len * 0.36, brick);
    }

    // -------------------------------------------------------------- the walls, last

    /// Give a stretch of wall that nothing was hung on what such a wall has.
    fn fill_bare(&mut self, id: usize) {
        let (s, top, fill, lean) = {
            let p = &self.pending[id];
            (p.s, p.top, p.fill, p.lean.is_some())
        };
        if fill != Fill::Windows || top < 0.6 * STOREY {
            return;
        }
        let (storeyed, n) = Self::storeyed(top);
        let faces: &[(usize, Face, f32)] = if lean { &[(0, Face::Front, s.len)] } else { &[(0, Face::Front, s.len), (1, Face::EndA, s.dep), (2, Face::Back, s.len), (3, Face::EndB, s.dep)] };
        for &(i, face, len) in faces {
            let cols = Self::bays(len);
            let cw = len / cols.max(1) as f32;
            for c in 0..cols {
                // the bay as the kit measures along the face, and as the shed does
                let (k0, k1) = (cw * (c as f32 + 0.2), cw * (c as f32 + 0.8));
                let u = match face {
                    Face::Front | Face::EndB => -len * 0.5 + cw * (c as f32 + 0.5),
                    Face::Back | Face::EndA => len * 0.5 - cw * (c as f32 + 0.5),
                };
                let tiers = if storeyed { n } else { 1 };
                for st in 0..tiers {
                    let (ya, yb) = if storeyed { (st as f32 * STOREY + FOOT, (st + 1) as f32 * STOREY - 0.003) } else { (FOOT, top) };
                    // how high what is already in the bay reaches
                    let taken = self.pending[id].ops[i].iter().filter(|o| o.u + o.w * 0.5 > k0 && o.u - o.w * 0.5 < k1 && o.y + o.h > ya && o.y < yb).map(|o| o.y + o.h).fold(0.0f32, f32::max);
                    if taken == 0.0 {
                        self.bay_windows(&s, face, u, top, storeyed, st);
                    } else if !storeyed && top - 0.0048 - taken >= 0.6 * STOREY {
                        // a hall's wall above its doors and its open bays: a light high up
                        self.light(&s, face, u - WINDOW.0 * 0.5, u + WINDOW.0 * 0.5, taken + 0.0045, top - 0.0048, false);
                    }
                }
            }
        }
    }

    /// Build the walls of every building, now that all that hangs on them
    /// is known: to their full height, or part of the way while the works
    /// is going up; open to the sky then and when it is derelict, so the
    /// inside of the far wall is drawn too.
    pub fn finish(&mut self) {
        for id in 0..self.pending.len() {
            if !self.going_up() && !self.far() {
                self.fill_bare(id);
            }
        }
        let pending = std::mem::take(&mut self.pending);
        let (going_up, derelict, far, near) = (self.going_up(), self.derelict(), self.far(), self.near());
        let soot = *self.pal.smoke.soot;
        let lie = self.lie();
        for mut p in pending {
            let s = p.s;
            if derelict {
                // the glass is gone from its windows
                for o in p.ops.iter_mut().flatten() {
                    if let Open::Paned { round, .. } = o.kind {
                        o.kind = Open::Mouth { within: soot, round };
                    }
                }
            }
            let k_up = if going_up { 0.55 } else { 1.0 };
            let top = p.top * k_up;
            let open_top = going_up || derelict;
            let posts = p.fill == Fill::Posts && !going_up && !far;
            self.with_kit(&s, |k, rg| {
                let y0 = -k.drop(rg);
                let side = |i: usize| Side::free(&p.ops[i]);
                match p.lean {
                    Some((low, high, roof)) if !open_top => {
                        k.lean_to(rg, low, high, &p.wall, roof, lie.eaves, side(0), [true, true]);
                        if p.free {
                            k.walls(rg, y0, high, &p.wall, [Side::NONE, Side::NONE, Side::PLAIN, Side::NONE], true, false);
                        }
                    }
                    Some(_) => k.walls(rg, y0, top, &p.wall, [side(0), Side::PLAIN, Side::PLAIN, Side::PLAIN], true, false),
                    None => k.walls(rg, y0, top, &p.wall, [side(0), side(1), side(2), side(3)], true, p.eaves && !open_top),
                }
                if posts {
                    // the posts of a boarded frame, a bay apart, and close to the joints of its boards
                    let (post, joint) = (shade(p.wall.col, 0.8), shade(p.wall.col, 0.9));
                    let t = if near { 0.0008 } else { 0.0013 };
                    let sides: &[(usize, f32)] = match p.lean {
                        Some((_, high, _)) if p.free => &[(0, top), (2, high)],
                        Some(_) => &[(0, top)],
                        None => &[(0, top), (1, top), (2, top), (3, top)],
                    };
                    for &(i, up) in sides {
                        let len = if i % 2 == 0 { s.len } else { s.dep };
                        let cols = Self::bays(len).max(1);
                        let clear = |u: f32, t: f32| p.ops[i].iter().all(|o| (o.u - u).abs() > o.w * 0.5 + t);
                        for c in 0..=cols {
                            let u = (len * c as f32 / cols as f32).clamp(t, len - t);
                            if clear(u, t) {
                                k.strip(rg, i, u - t, u + t, 0.0, up - 0.0014, 0.0006, post);
                            }
                            if near && c < cols {
                                for j in 1..5 {
                                    let u = len * (c as f32 + j as f32 * 0.2) / cols as f32;
                                    if clear(u, 0.0004) {
                                        k.strip(rg, i, u - 0.0002, u + 0.0002, FOOT, up - 0.0014, 0.0003, joint);
                                    }
                                }
                            }
                        }
                    }
                }
            });
            if open_top && p.lean.is_none() {
                let (hl, hd) = (s.len * 0.5 - 0.0015, s.dep * 0.5 - 0.0015);
                let inside = shade(p.wall.col, 0.62);
                let c = [(-hl, -hd), (hl, -hd), (hl, hd), (-hl, hd)];
                for i in 0..4 {
                    let (a, b) = (c[i], c[(i + 1) % 4]);
                    self.solid.quad(s.pt(a.0, 0.0, a.1), s.pt(b.0, 0.0, b.1), s.pt(b.0, top, b.1), s.pt(a.0, top, a.1), inside);
                }
            }
        }
    }
}

/// A plume of smoke from `top`: five soft rings along a path that rises and
/// leans downwind, each wider and thinner than the last, closed to a point
/// at both ends. Its corners are shared and their normals lean upward, so
/// it takes the light as one soft shape with no dark underside.
pub fn plume(smoke: &mut MeshBuf, top: Vec3, scale: f32, phase: f32, col: Rgba) {
    const N: usize = 5;
    const SEG: usize = 5;
    let (rise, drift) = (0.05 * scale, 0.085 * scale);
    let side = Vec3::new(-WIND.z, 0.0, WIND.x);
    let base = smoke.pos.len() as u32;
    let mut push = |p: Vec3, n: Vec3, a: f32, k: f32| {
        smoke.pos.push(p.to_array());
        smoke.nrm.push(n.to_array());
        smoke.col.push([col[0] * k, col[1] * k, col[2] * k, a]);
    };
    push(top, Vec3::Y, 0.85, 0.9);
    for i in 0..N {
        let t = (i as f32 + 0.6) / N as f32;
        let sway = side * (0.007 * scale * (phase + t * 5.0).sin() * t);
        let c = top + Vec3::Y * (rise * t.powf(0.7)) + WIND * (drift * t.powf(1.5)) + sway;
        let r = scale * (0.0032 + 0.0125 * t) * (1.0 + 0.2 * (phase * 1.7 + t * 8.0).sin());
        let dir = (Vec3::Y * (0.7 * rise * t.powf(-0.3)) + WIND * (1.5 * drift * t.powf(0.5))).normalize_or_zero();
        let u = dir.cross(side).normalize_or_zero();
        let alpha = 0.78 * (1.0 - t).powf(0.7) + 0.04;
        for k in 0..SEG {
            let a = k as f32 / SEG as f32 * std::f32::consts::TAU + phase;
            let off = u * a.cos() + side * a.sin();
            push(c + off * r, (off * 0.45 + Vec3::Y * 0.85).normalize_or_zero(), alpha, 0.92 + 0.08 * t);
        }
    }
    push(top + Vec3::Y * rise + WIND * drift, Vec3::Y, 0.0, 1.0);
    let ring = |i: usize, k: usize| base + 1 + (i * SEG + k % SEG) as u32;
    let last = base + 1 + (N * SEG) as u32;
    for k in 0..SEG {
        smoke.idx.extend_from_slice(&[base, ring(0, k), ring(0, k + 1)]);
        for i in 0..N - 1 {
            smoke.idx.extend_from_slice(&[ring(i, k), ring(i + 1, k), ring(i + 1, k + 1), ring(i, k), ring(i + 1, k + 1), ring(i, k + 1)]);
        }
        smoke.idx.extend_from_slice(&[ring(N - 1, k), last, ring(N - 1, k + 1)]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::works::Kind;

    #[test]
    fn a_plume_is_one_soft_shape_that_rises_leans_downwind_and_thins_to_nothing() {
        let top = Vec3::new(1.0, 0.5, 2.0);
        let build = |scale: f32| {
            let mut b = MeshBuf::default();
            plume(&mut b, top, scale, 0.3, [0.9, 0.9, 0.9, 1.0]);
            assert!(b.is_sound());
            b
        };
        let b = build(1.0);
        // its corners are shared, so it is shaded as one shape: far fewer than three to a triangle
        assert!(b.pos.len() < b.triangles());
        let (lo, hi) = b.bounds().unwrap();
        assert!(lo.y >= top.y - 1e-6 && hi.y > top.y + 0.03, "it rises from the flue");
        assert!(hi.x - top.x > 0.05 && lo.x - top.x > -0.02 && hi.z > top.z, "it leans with the wind");
        // solid at the flue, nothing at the tip, and thinner all the way
        let alpha: Vec<f32> = b.col.iter().map(|c| c[3]).collect();
        assert!(alpha[0] > 0.8 && *alpha.last().unwrap() == 0.0);
        assert!(alpha.windows(2).all(|w| w[1] <= w[0]));
        // every corner looks upward: there is no dark underside
        assert!(b.nrm.iter().all(|n| n[1] > 0.3));
        // more smoke is a bigger plume, not more triangles
        let big = build(2.0);
        assert_eq!(big.triangles(), b.triangles());
        assert!(big.bounds().unwrap().1.y - top.y > 1.8 * (hi.y - top.y));
    }

    #[test]
    fn things_in_a_line_share_the_room() {
        let pal = Palette::standard();
        let (mut a, mut b, mut f) = (MeshBuf::default(), MeshBuf::default(), MeshBuf::default());
        let site = Site::level(Vec3::ZERO, 0.0);
        let c = Ctx::new(&mut a, &mut b, &mut f, &site, &Works::new(Kind::Other, 1.0), &pal);
        // one thing no longer than the room
        assert_eq!(c.spread(0.05, 0.1, 0.01, 4), (1, 0.05));
        // two things and the gap between them fill the room exactly
        let (n, len) = c.spread(0.25, 0.2, 0.01, 4);
        assert_eq!(n, 2);
        assert!((2.0 * len + 0.01 - 0.25).abs() < 1e-6);
        // never more than the most allowed, nor longer than the longest
        assert_eq!(c.spread(2.0, 0.2, 0.01, 3), (3, 0.2));
        assert_eq!(c.fit(0.1, 0.03, 8), 3);
        assert_eq!(c.fit(0.01, 0.03, 8), 1);
    }

    #[test]
    fn a_yard_follows_sloping_ground() {
        let pal = Palette::standard();
        let slope = |x: f32, _z: f32| x * 0.2;
        let site = Site::on(Vec3::ZERO, 0.0, &slope);
        let (mut solid, mut smoke, mut flat) = (MeshBuf::default(), MeshBuf::default(), MeshBuf::default());
        crate::works::works(&mut solid, &mut smoke, &mut flat, &site, &Works::new(Kind::TextileMill, 30.0), &pal);
        assert!(solid.is_sound() && flat.is_sound());
        // the yard's ground lies just above the land everywhere, in squares small enough to follow it
        assert!(flat.triangles() > 20);
        for p in &flat.pos {
            let above = p[1] - slope(p[0], p[2]);
            assert!(above > 0.003 && above < 0.012, "the yard is {above} above the land at {p:?}");
        }
    }
}
