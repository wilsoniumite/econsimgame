//! The parts a works is put together from, and the yard they stand in.
//!
//! Every kind of works is a few lines that place these parts in its yard:
//! sheds of one sort and another, chimneys and stacks, heaps and stacks of
//! goods, a waterwheel, the wall and the gate. The parts know the works'
//! state (a roof is not on yet while it is going up, and half off once it is
//! derelict; nothing smokes unless it is working), so a kind says what it is
//! and the state follows by itself.
//!
//! Places are in the works' own frame: x along its front, z out toward the
//! gate, the origin in the middle of the yard, which is `2 * hw` wide and
//! `2 * hd` deep. Heights are above the ground under each part.

use glam::{Quat, Vec3};

use super::{State, Works};
use crate::colour::{lerp, shade, Rgba};
use crate::hash::hash2;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// The height of one storey of a works.
pub(super) const STOREY: f32 = 0.022;
/// How far walls reach below the ground, so that a slope shows no gap.
const SINK: f32 = 0.02;
/// How far a roof overhangs its walls.
const OVER: f32 = 0.003;
/// How far a yard's ground lies above the land, and what lies on it above that.
const LIFT: f32 = 0.0045;
/// The yard's ground follows the land in squares no bigger than this.
const GROUND_STEP: f32 = 0.045;
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

/// A building that has been drawn, for hanging doors, windows and vents on.
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

    fn face_pt(&self, face: Face, u: f32, y: f32) -> Vec3 {
        const PROUD: f32 = 0.0007;
        match face {
            Face::Front => self.pt(u, y, self.dep * 0.5 + PROUD),
            Face::Back => self.pt(u, y, -self.dep * 0.5 - PROUD),
            Face::EndA => self.pt(self.len * 0.5 + PROUD, y, u),
            Face::EndB => self.pt(-self.len * 0.5 - PROUD, y, u),
        }
    }
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
    count: i32,
}

impl<'a, 's> Ctx<'a, 's> {
    pub fn new(solid: &'a mut MeshBuf, smoke: &'a mut MeshBuf, flat: &'a mut MeshBuf, site: &'a Site<'s>, w: &Works, pal: &'a Palette) -> Self {
        let (fw, fd) = super::footprint(w.kind, w.size);
        Ctx { solid, smoke, flat, site, pal, w: *w, hw: fw * 0.5, hd: fd * 0.5, g: super::grain(w.size), peak: None, sheds: Vec::new(), count: 0 }
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

    /// The next of the works' own fixed numbers, from 0 to 1.
    pub fn rnd(&mut self) -> f32 {
        self.count += 1;
        hash2(self.count, (self.w.seed.u * 9973.0) as i32, 31 + (self.w.seed.v * 977.0) as i32)
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

    /// A colour as it weathers: dulled and greened once the works is derelict.
    pub fn tone(&self, c: Rgba) -> Rgba {
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

    /// The point on the ground at a place in the yard.
    pub fn at(&self, x: f32, z: f32) -> Vec3 {
        self.site.at(x, z)
    }

    /// The wall round a yard, in the stone or the brick of the period.
    pub fn yard_wall(&self) -> Rgba {
        if self.early() { *self.pal.stone.yard_wall } else { shade(*self.pal.brick.dark, 0.92) }
    }

    /// A point that may be the works' highest: the family's flag flies from the highest.
    pub fn note_peak(&mut self, p: Vec3) {
        if self.peak.map_or(true, |q| p.y > q.y) {
            self.peak = Some(p);
        }
    }

    // -------------------------------------------------------------- buildings

    fn frame(&self, x: f32, z: f32, len: f32, dep: f32, h: f32, rise: f32, across: bool) -> Shed {
        let yaw = self.site.yaw + if across { std::f32::consts::FRAC_PI_2 } else { 0.0 };
        Shed { x, z, len, dep, h, rise, across, o: self.site.at(x, z), q: Quat::from_rotation_y(yaw), yaw }
    }

    /// Four walls: to their full height, or part of the way while the works
    /// is going up; open to the sky then and when it is derelict, so the
    /// inside of the far wall is drawn too.
    fn walls(&mut self, s: &Shed, wall: Rgba) {
        let h = if self.going_up() { s.h * 0.55 } else { s.h };
        self.solid.block(s.o, s.len, s.dep, h, s.yaw, wall, None, SINK);
        if self.going_up() || self.derelict() {
            let (hw, hd) = (s.len * 0.5 - 0.0015, s.dep * 0.5 - 0.0015);
            let inside = shade(wall, 0.62);
            let c = [(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)];
            for i in 0..4 {
                let (a, b) = (c[i], c[(i + 1) % 4]);
                self.solid.quad(s.pt(a.0, 0.0, a.1), s.pt(b.0, 0.0, b.1), s.pt(b.0, h, b.1), s.pt(a.0, h, a.1), inside);
            }
        }
    }

    /// One slope of a pitched roof, from `x0` to `x1` along the ridge.
    fn slope(&mut self, s: &Shed, x0: f32, x1: f32, front: bool, col: Rgba) {
        let (e, hd) = (s.h - OVER * 0.6, s.dep * 0.5 + OVER);
        let top = s.h + s.rise;
        if front {
            self.solid.quad(s.pt(x0, e, hd), s.pt(x1, e, hd), s.pt(x1, top, 0.0), s.pt(x0, top, 0.0), col);
        } else {
            self.solid.quad(s.pt(x1, e, -hd), s.pt(x0, e, -hd), s.pt(x0, top, 0.0), s.pt(x1, top, 0.0), shade(col, 0.82));
        }
    }

    /// A pitched roof: whole on a works that stands, none on one going up,
    /// and with slates gone in patches on a derelict one.
    fn pitched(&mut self, s: &Shed, roof: Rgba, gable: Rgba) {
        if self.going_up() {
            return;
        }
        if !self.derelict() {
            self.solid.gable(s.o, s.len, s.dep, s.h, s.rise, s.yaw, OVER, roof, gable);
            return;
        }
        let (ew, ed, top) = (s.len * 0.5, s.dep * 0.5, s.h + s.rise - OVER * 0.2);
        self.solid.tri(s.pt(ew, s.h, ed), s.pt(ew, s.h, -ed), s.pt(ew, top, 0.0), gable);
        self.solid.tri(s.pt(-ew, s.h, -ed), s.pt(-ew, s.h, ed), s.pt(-ew, top, 0.0), shade(gable, 0.9));
        let n = ((s.len / 0.02).ceil() as usize).clamp(2, 8);
        for front in [true, false] {
            for i in 0..n {
                if self.rnd() < 0.42 {
                    continue;
                }
                let (x0, x1) = (-ew - OVER + (s.len + 2.0 * OVER) * i as f32 / n as f32, -ew - OVER + (s.len + 2.0 * OVER) * (i + 1) as f32 / n as f32);
                self.slope(s, x0, x1, front, roof);
            }
        }
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
        let s = self.frame(x, z, len, dep, h, dep * pitch, across);
        self.walls(&s, wall);
        self.pitched(&s, roof, wall);
        if !self.going_up() {
            // the gable end nearer the front, or the one to the right
            self.note_peak(s.pt(if across { -len * 0.5 } else { len * 0.5 }, s.h + s.rise, 0.0));
        }
        self.sheds.push(s);
        s
    }

    /// A lean-to: a single slope down from `h_back` on the shed's own -z to
    /// `h_front` on its +z.
    #[allow(clippy::too_many_arguments)]
    pub fn lean_to(&mut self, x: f32, z: f32, len: f32, dep: f32, h_front: f32, h_back: f32, across: bool, wall: Rgba, roof: Rgba) -> Shed {
        let s = self.frame(x, z, len, dep, h_front, 0.0, across);
        let k = if self.going_up() { 0.55 } else { 1.0 };
        let (hf, hb, hw, hd) = (h_front * k, h_back * k, len * 0.5, dep * 0.5);
        let c = [s.pt(-hw, -SINK, -hd), s.pt(hw, -SINK, -hd), s.pt(hw, -SINK, hd), s.pt(-hw, -SINK, hd)];
        let t = [s.pt(-hw, hb, -hd), s.pt(hw, hb, -hd), s.pt(hw, hf, hd), s.pt(-hw, hf, hd)];
        self.solid.quad(c[3], c[2], t[2], t[3], wall);
        self.solid.quad(c[2], c[1], t[1], t[2], shade(wall, 0.93));
        self.solid.quad(c[1], c[0], t[0], t[1], shade(wall, 0.86));
        self.solid.quad(c[0], c[3], t[3], t[0], shade(wall, 0.95));
        if !self.going_up() && !self.derelict() {
            let o = OVER;
            self.solid.quad(s.pt(-hw - o, hf - o * 0.4, hd + o), s.pt(hw + o, hf - o * 0.4, hd + o), s.pt(hw + o, hb, -hd), s.pt(-hw - o, hb, -hd), roof);
        }
        s
    }

    /// A roof on posts with no walls: a drying shed, a sawing shed.
    #[allow(clippy::too_many_arguments)]
    pub fn open_shed(&mut self, x: f32, z: f32, len: f32, dep: f32, h: f32, across: bool, post: Rgba, roof: Rgba) -> Shed {
        let s = self.frame(x, z, len, dep, h, dep * 0.3, across);
        let n = ((len / 0.035).ceil() as usize).clamp(1, 3);
        for side in [-1.0f32, 1.0] {
            for i in 0..=n {
                let px = -len * 0.5 + 0.002 + (len - 0.004) * i as f32 / n as f32;
                let foot = s.pt(px, 0.0, side * (dep * 0.5 - 0.002));
                self.solid.beam(foot - Vec3::Y * SINK * 0.5, foot + Vec3::Y * h, 0.0028, post);
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
        let s = self.frame(x, z, len, dep, h, rise, across);
        self.walls(&s, wall);
        self.sheds.push(s);
        if self.going_up() {
            return s;
        }
        let glass = self.tone(*self.pal.glass.roof_light);
        let hd = dep * 0.5;
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
        }
        self.note_peak(s.pt(len * 0.5, h + rise, hd));
        s
    }

    /// A flat-topped block: a bank of coke ovens, a platform.
    #[allow(clippy::too_many_arguments)]
    pub fn bank(&mut self, x: f32, z: f32, len: f32, dep: f32, h: f32, across: bool, wall: Rgba, top: Rgba) -> Shed {
        let s = self.frame(x, z, len, dep, h, 0.0, across);
        let h = if self.going_up() { h * 0.55 } else { h };
        self.solid.block(s.o, len, dep, h, s.yaw, wall, Some(top), SINK);
        s
    }

    /// A square tower under a pyramid roof: a mill's stair tower, an engine house.
    pub fn tower(&mut self, x: f32, z: f32, side: f32, h: f32, wall: Rgba, roof: Rgba) -> Shed {
        let s = self.frame(x, z, side, side, h, side * 0.45, false);
        self.walls(&s, wall);
        if !self.going_up() && !self.derelict() {
            let turn = std::f32::consts::FRAC_PI_4 - s.yaw;
            self.solid.frustum_turned(s.o + Vec3::Y * (h - OVER * 0.5), (side * 0.5 + OVER) * std::f32::consts::SQRT_2, 0.0, s.rise, 4, turn, 0.0, roof, None);
            self.note_peak(s.pt(0.0, h + s.rise, 0.0));
        }
        self.sheds.push(s);
        s
    }

    /// A vent along the ridge of a shed, over `part` of its length: louvred
    /// (dark) to let the heat out, or a glazed lantern to let light in.
    pub fn ridge_vent(&mut self, s: &Shed, part: f32, glazed: bool) {
        if self.going_up() || self.derelict() {
            return;
        }
        let (w, d) = (s.len * part, (s.dep * 0.2).max(0.005));
        let side = if glazed { *self.pal.glass.roof_light } else { *self.pal.smoke.soot };
        let base = s.pt(0.0, s.h + s.rise - d * 0.42, 0.0);
        let up = d * 0.42 + 0.0035;
        self.solid.block(base, w, d, up, s.yaw, side, None, 0.0);
        self.solid.gable(base, w, d, up, d * 0.3, s.yaw, 0.001, if glazed { side } else { self.roofing() }, side);
    }

    /// Panes let into both slopes of a shed's roof, `n` along each.
    pub fn roof_lights(&mut self, s: &Shed, n: usize) {
        if self.going_up() || self.derelict() {
            return;
        }
        let glass = *self.pal.glass.roof_light;
        let at = |f: f32, side: f32| (s.h + s.rise * f + 0.0012, side * s.dep * 0.5 * (1.0 - f));
        for i in 0..n {
            let cx = -s.len * 0.5 + s.len * (i as f32 + 0.5) / n as f32;
            let hw = s.len / n as f32 * 0.3;
            let ((y0, z0), (y1, z1)) = (at(0.25, 1.0), at(0.75, 1.0));
            self.solid.quad(s.pt(cx - hw, y0, z0), s.pt(cx + hw, y0, z0), s.pt(cx + hw, y1, z1), s.pt(cx - hw, y1, z1), glass);
            let ((y0, z0), (y1, z1)) = (at(0.25, -1.0), at(0.75, -1.0));
            self.solid.quad(s.pt(cx + hw, y0, z0), s.pt(cx - hw, y0, z0), s.pt(cx - hw, y1, z1), s.pt(cx + hw, y1, z1), shade(glass, 0.85));
        }
    }

    /// A flat panel on a face of a shed (a window, a door, an opening), from
    /// `u0` to `u1` along the face and `y0` to `y1` up it; with `arch`, a
    /// rounded head above `y1`.
    #[allow(clippy::too_many_arguments)]
    pub fn panel(&mut self, s: &Shed, face: Face, u0: f32, u1: f32, y0: f32, y1: f32, arch: bool, col: Rgba) {
        let turn = matches!(face, Face::Back | Face::EndA);
        let mut put = |p: [(f32, f32); 4]| {
            let v = p.map(|(u, y)| s.face_pt(face, u, y));
            if turn {
                self.solid.quad(v[1], v[0], v[3], v[2], col);
            } else {
                self.solid.quad(v[0], v[1], v[2], v[3], col);
            }
        };
        put([(u0, y0), (u1, y0), (u1, y1), (u0, y1)]);
        if arch {
            let w = u1 - u0;
            put([(u0, y1), (u1, y1), (u1 - w * 0.27, y1 + w * 0.3), (u0 + w * 0.27, y1 + w * 0.3)]);
        }
    }

    /// Rows of windows on both long walls of a shed, and a pair to a storey
    /// on its gable ends when `ends` is set: the shed has `storeys` storeys,
    /// those from `from` up to (not counting) `upto` are given windows, about `pitch` apart.
    #[allow(clippy::too_many_arguments)]
    pub fn windows(&mut self, s: &Shed, storeys: usize, from: usize, upto: usize, pitch: f32, ends: bool, glass: Rgba) {
        if self.going_up() {
            return;
        }
        let glass = if self.derelict() { *self.pal.smoke.soot } else { glass };
        let sh = s.h / storeys.max(1) as f32;
        let cols = ((s.len / pitch).floor() as usize).max(1);
        let ww = (s.len / cols as f32 * 0.5).min(0.007);
        for face in [Face::Front, Face::Back] {
            for st in from..upto {
                let y = sh * (st as f32 + 0.32);
                for c in 0..cols {
                    let cx = -s.len * 0.5 + s.len * (c as f32 + 0.5) / cols as f32;
                    self.panel(s, face, cx - ww * 0.5, cx + ww * 0.5, y, y + sh * 0.46, false, glass);
                }
            }
        }
        if ends {
            for face in [Face::EndA, Face::EndB] {
                for st in from..upto {
                    let y = sh * (st as f32 + 0.32);
                    for cz in [-s.dep * 0.22, s.dep * 0.22] {
                        self.panel(s, face, cz - ww * 0.5, cz + ww * 0.5, y, y + sh * 0.46, false, glass);
                    }
                }
            }
        }
    }

    /// One long band of glass along both long walls of a shed, at `y`: a weaver's window.
    pub fn window_band(&mut self, s: &Shed, y: f32, tall: f32, glass: Rgba) {
        if self.going_up() {
            return;
        }
        let glass = if self.derelict() { *self.pal.smoke.soot } else { glass };
        for face in [Face::Front, Face::Back] {
            self.panel(s, face, -s.len * 0.44, s.len * 0.44, y, y + tall, false, glass);
        }
    }

    // -------------------------------------------------------------- chimneys and smoke

    /// A tall round chimney on a square foot, with its smoke; `puff` is how
    /// much smoke it makes (1 for a mill's).
    pub fn chimney(&mut self, x: f32, z: f32, h: f32, r: f32, puff: f32) {
        let p = self.at(x, z);
        let brick = self.tone(*self.pal.brick.chimney);
        let (h, whole) = match self.w.state {
            State::GoingUp => (h * 0.4, false),
            State::Derelict => (h * 0.72, false),
            _ => (h, true),
        };
        let turn = std::f32::consts::FRAC_PI_4 - self.site.yaw;
        self.solid.frustum_turned(p, r * 1.75, r * 1.75, h * 0.1, 4, turn, SINK, shade(brick, 0.94), Some(shade(brick, 0.8)));
        self.solid.frustum_turned(p + Vec3::Y * h * 0.1, r, r * 0.62, h * 0.9, 6, 0.0, 0.0, brick, Some(*self.pal.smoke.soot));
        if whole {
            // the oversailing courses at the top
            self.solid.frustum_turned(p + Vec3::Y * h * 0.95, r * 0.8, r * 0.8, h * 0.035, 6, 0.0, 0.0, shade(brick, 0.86), None);
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
        self.solid.frustum_turned(p, side * 0.707, side * 0.6, h, 4, turn, SINK, col, Some(*self.pal.smoke.soot));
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
        self.solid.block(p, w, d, h, s.yaw, col, Some(*self.pal.smoke.soot), SINK);
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
        self.solid.frustum_turned(p, r0, r1, h, 6, 0.0, SINK, col, Some(cap));
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

    /// Fire seen in an opening on a shed's face: lit only while the works is working.
    #[allow(clippy::too_many_arguments)]
    pub fn glow(&mut self, s: &Shed, face: Face, u0: f32, u1: f32, y0: f32, y1: f32, arch: bool) {
        let col = if self.busy() { shade(*self.pal.iron.glow, 1.6) } else { *self.pal.smoke.soot };
        self.panel(s, face, u0, u1, y0, y1, arch, col);
    }

    /// The colour of a furnace's open throat: fire while it is in blast, soot otherwise.
    pub fn throat(&self) -> Rgba {
        if self.busy() { shade(*self.pal.iron.glow, 1.6) } else { *self.pal.smoke.soot }
    }

    // -------------------------------------------------------------- what stands in the yard

    /// A heap of something loose (coal, ore, slag), `r` across at the foot.
    pub fn heap(&mut self, x: f32, z: f32, r: f32, h: f32, col: Rgba) {
        let turn = self.rnd() * 3.0;
        self.solid.frustum_turned(self.at(x, z), r, r * 0.28, h, 5, turn, SINK * 0.5, col, Some(shade(col, 1.08)));
    }

    /// A square stack of something made (bricks, bales, planks, pig iron):
    /// `len` along the front (or from front to back, with `across`).
    #[allow(clippy::too_many_arguments)]
    pub fn pile(&mut self, x: f32, z: f32, len: f32, dep: f32, h: f32, across: bool, col: Rgba) {
        let yaw = self.site.yaw + if across { std::f32::consts::FRAC_PI_2 } else { 0.0 };
        self.solid.block(self.at(x, z), len, dep, h, yaw, col, Some(shade(col, 1.08)), SINK * 0.5);
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
    /// a flywheel.
    pub fn wheel(&mut self, x: f32, z: f32, r: f32, width: f32, across: bool, col: Rgba) {
        let s = self.frame(x, z, 0.0, 0.0, 0.0, 0.0, across);
        let cy = r * 0.82;
        const N: usize = 8;
        let rim = |i: usize, z: f32| {
            let a = i as f32 / N as f32 * std::f32::consts::TAU;
            s.pt(a.cos() * r, cy + a.sin() * r, z)
        };
        let hz = width * 0.5;
        for i in 0..N {
            let j = (i + 1) % N;
            let k = 0.8 + 0.2 * (i as f32 / N as f32 * std::f32::consts::TAU).sin();
            self.solid.quad(rim(i, hz), rim(i, -hz), rim(j, -hz), rim(j, hz), shade(col, k));
            self.solid.tri(s.pt(0.0, cy, hz), rim(i, hz), rim(j, hz), shade(col, if i % 2 == 0 { 1.0 } else { 0.86 }));
            self.solid.tri(s.pt(0.0, cy, -hz), rim(j, -hz), rim(i, -hz), shade(col, if i % 2 == 0 { 0.8 } else { 0.7 }));
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
        let col = self.tone(*self.pal.timber.headframe);
        let top = self.at(x, z) + Vec3::Y * h;
        for (dx, dz) in [(-0.3, -0.25), (0.3, -0.25), (0.0, 0.35)] {
            self.solid.beam(self.at(x + dx * h, z + dz * h), top, h * 0.07, col);
        }
    }

    // -------------------------------------------------------------- the yard

    /// The yard's own ground over the whole footprint (or, with `round`,
    /// over the ellipse within it), following the land.
    pub fn ground(&mut self, col: Rgba, round: bool) {
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
    /// following the ground.
    fn wall_run(&mut self, a: (f32, f32), b: (f32, f32), h: f32, t: f32, col: Rgba) {
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
            if self.derelict() && self.rnd() < 0.3 {
                continue;
            }
            let (p, q) = (at(i), at(i + 1));
            self.solid.quad(p + o - down, q + o - down, q + o + up, p + o + up, col);
            self.solid.quad(q - o - down, p - o - down, p - o + up, q - o + up, shade(col, 0.84));
            self.solid.quad(p + o + up, q + o + up, q - o + up, p - o + up, shade(col, 1.03));
        }
    }

    /// The wall (or fence) round the yard, with a gate in the middle of the
    /// front `gate` wide: posts either side of it, and the gate shut across
    /// it when the works stands idle. Nothing while the works is going up.
    pub fn wall(&mut self, col: Rgba, h: f32, gate: f32) {
        if self.going_up() {
            return;
        }
        let col = self.tone(col);
        let (hw, hd, t) = (self.hw - 0.0025, self.hd - 0.0025, 0.0046);
        let gate = gate.min(hw * 1.2);
        self.wall_run((hw, -hd), (-hw, -hd), h, t, col);
        self.wall_run((-hw, -hd), (-hw, hd), h, t, col);
        self.wall_run((hw, hd), (hw, -hd), h, t, col);
        self.wall_run((-hw, hd), (-gate * 0.5, hd), h, t, col);
        self.wall_run((gate * 0.5, hd), (hw, hd), h, t, col);
        if self.w.size >= 1.0 && !self.derelict() {
            for side in [-1.0f32, 1.0] {
                let p = self.at(side * gate * 0.5, hd);
                self.solid.block(p, t * 2.0, t * 2.0, h * 1.5, self.site.yaw, shade(col, 0.95), Some(shade(col, 1.1)), SINK * 0.5);
            }
        }
        if self.w.state == State::Idle {
            let wood = *self.pal.timber.door;
            self.wall_run((-gate * 0.5, hd), (gate * 0.5, hd), h * 0.9, t * 0.5, wood);
        }
    }

    /// Poles and boards round a building that is going up or being enlarged.
    pub fn scaffold(&mut self, s: &Shed) {
        let (hw, hd) = (s.len * 0.5 + 0.004, s.dep * 0.5 + 0.004);
        let h = if self.going_up() { s.h * 0.55 + 0.012 } else { s.h + s.rise * 0.6 };
        let pole = *self.pal.timber.scaffold_pole;
        let board = *self.pal.timber.scaffold_board;
        let corners = [s.pt(-hw, 0.0, -hd), s.pt(hw, 0.0, -hd), s.pt(hw, 0.0, hd), s.pt(-hw, 0.0, hd)];
        let t = (0.003 * self.g).min(0.0045);
        for c in corners {
            self.solid.beam(c - Vec3::Y * 0.01, c + Vec3::Y * h, t, pole);
        }
        if s.len > 0.05 {
            for c in [s.pt(0.0, 0.0, -hd), s.pt(0.0, 0.0, hd)] {
                self.solid.beam(c - Vec3::Y * 0.01, c + Vec3::Y * h, t, pole);
            }
        }
        for lift in [0.45, 0.9] {
            let y = Vec3::Y * (h * lift);
            for i in 0..4 {
                self.solid.beam(corners[i] + y, corners[(i + 1) % 4] + y, t * 1.2, board);
            }
        }
        self.solid.beam(corners[3] + Vec3::Y * 0.004, corners[2] + Vec3::Y * (h * 0.9), t * 0.8, pole);
    }

    /// What lies about a works that is being built: a pile of timber and a heap of bricks.
    pub fn builders_yard(&mut self) {
        let (x, z) = (self.hw * 0.5, self.hd * 0.72);
        let board = *self.pal.timber.scaffold_board;
        let len = (0.03 * self.g).min(self.hw * 0.7);
        self.pile(x, z, len, len * 0.4, len * 0.24, false, board);
        let brick = if self.early() { *self.pal.stone.works } else { *self.pal.goods.bricks };
        self.heap(x - len * 1.1, z, len * 0.36, len * 0.36, brick);
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
