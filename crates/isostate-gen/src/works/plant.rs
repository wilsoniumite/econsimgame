//! The plant of the fire trades: what stands in a yard that is not a
//! building. A blast furnace's stack, built of stone or cased in brick or
//! iron; the gallery it is charged from; hoops and bands; a lift's tower; a
//! bottle kiln. And the plant of the twentieth century, which more than one
//! kind stands in its yard: a cooling tower, a round tank, a column with its
//! platforms and ladder, a rack of pipes.
//!
//! These are parts of the works' kit like the sheds and chimneys in `kit`
//! (methods of `Ctx`; each knows the state and the level), and they are
//! built to the same scale: from far off the plain shape, from the play
//! distance its openings as flat dark panels and one band, close to its
//! courses, hoops, arches with their rings of voussoirs, rails and braces,
//! none finer than a house's glazing bar.

use glam::{Quat, Vec3};

use super::kit::Ctx;
use crate::colour::{shade, Rgba};
use crate::mesh::MeshBuf;

use std::f32::consts::{PI, TAU};

/// How far the foot of anything here reaches below the ground, so that a slope shows no gap.
const SINK: f32 = 0.02;

/// A column that tapers, as `MeshBuf::frustum_turned` builds it, for
/// putting things on its faces and round it.
#[derive(Clone, Copy)]
struct Column {
    /// the middle of its foot
    p: Vec3,
    /// to its corners, at its foot and at its head
    r0: f32,
    r1: f32,
    h: f32,
    /// how far its foot reaches below the ground, where it is `r0` out
    sink: f32,
    sides: usize,
    turn: f32,
}

impl Column {
    /// How far out its corners are at a height above the ground.
    fn r(&self, y: f32) -> f32 {
        self.r0 + (self.r1 - self.r0) * ((y + self.sink) / (self.h + self.sink).max(1e-6))
    }

    /// The way face `i` looks.
    fn facing(&self, i: usize) -> Vec3 {
        let a = (i as f32 + 0.5) / self.sides as f32 * TAU + self.turn;
        Vec3::new(a.cos(), 0.0, a.sin())
    }

    /// The face that looks most nearly the way `d` points.
    fn face_toward(&self, d: Vec3) -> usize {
        (0..self.sides).max_by(|&a, &b| self.facing(a).dot(d).total_cmp(&self.facing(b).dot(d))).unwrap_or(0)
    }

    /// Half the width of a face at a height.
    fn half(&self, y: f32) -> f32 {
        self.r(y) * (PI / self.sides as f32).sin()
    }

    /// A point on face `i`: `w` along it from its middle, `y` up it, `out` proud of it.
    fn on(&self, i: usize, w: f32, y: f32, out: f32) -> Vec3 {
        let o = self.facing(i);
        let s = Vec3::new(-o.z, 0.0, o.x);
        self.p + o * (self.r(y) * (PI / self.sides as f32).cos() + out) + s * w + Vec3::Y * y
    }

    /// The column itself, capped or open.
    fn build(&self, solid: &mut MeshBuf, col: Rgba, cap: Option<Rgba>) {
        solid.frustum_turned(self.p, self.r0, self.r1, self.h, self.sides, self.turn, self.sink, col, cap);
    }

    /// A band round it from `y` up `tall`, `proud` of it. A course of
    /// masonry stands square, with a top of its own; a hoop of iron lies on
    /// the taper.
    fn band(&self, solid: &mut MeshBuf, y: f32, tall: f32, proud: f32, square: bool, col: Rgba) {
        let k = 1.0 / (PI / self.sides as f32).cos();
        let low = self.r(y) + proud * k;
        if square {
            solid.frustum_turned(self.p + Vec3::Y * y, low, low, tall, self.sides, self.turn, 0.0, col, Some(shade(col, 1.06)));
        } else {
            solid.frustum_turned(self.p + Vec3::Y * y, low, self.r(y + tall) + proud * k, tall, self.sides, self.turn, 0.0, col, None);
        }
    }

    /// An arched opening on face `i`, seen as a dark panel: `w` wide, its
    /// foot at `y0` and its crown `tall` above that, its head in `segs` flat
    /// lengths; with a ring of voussoirs `ring` round the head, if any.
    #[allow(clippy::too_many_arguments)]
    fn arch(&self, solid: &mut MeshBuf, i: usize, w: f32, y0: f32, tall: f32, out: f32, segs: usize, within: Rgba, ring: Option<Rgba>) {
        let hw = (w * 0.5).min(self.half(y0 + tall) - 0.0008).max(0.0008);
        let rise = hw.min(tall * 0.6);
        let spring = y0 + tall - rise;
        let toward = self.facing(i);
        let arc = |k: usize, r: f32| {
            let a = k as f32 / segs as f32 * PI;
            (a.cos() * (hw + r), spring + a.sin() * (rise + r))
        };
        let foot = self.on(i, 0.0, y0, out);
        let mut last = self.on(i, hw, y0, out);
        for k in 0..=segs {
            let (x, y) = arc(k, 0.0);
            let next = self.on(i, x, y, out);
            solid.tri_toward([foot, last, next], [within; 3], toward);
            last = next;
        }
        solid.tri_toward([foot, last, self.on(i, -hw, y0, out)], [within; 3], toward);
        if let Some(stone) = ring {
            let t = 0.0009;
            for k in 0..segs {
                let ((x0, y0), (x1, y1)) = (arc(k, 0.0), arc(k + 1, 0.0));
                let ((x2, y2), (x3, y3)) = (arc(k + 1, t), arc(k, t));
                let p = [self.on(i, x0, y0, out + 0.0003), self.on(i, x1, y1, out + 0.0003), self.on(i, x2, y2, out + 0.0003), self.on(i, x3, y3, out + 0.0003)];
                solid.quad_toward(p, [stone; 4], toward);
            }
        }
    }
}

impl Ctx<'_, '_> {
    /// The way the works' own +x runs on the map.
    fn along_front(&self) -> Vec3 {
        Quat::from_rotation_y(self.site.yaw) * Vec3::X
    }

    /// A blast furnace's stack built of masonry: square and battered,
    /// `foot` across at the ground and `head` across at its top, `h` high
    /// (the caller says how high it stands in the works' state), standing
    /// square to the yard. From the play distance it shows the arches in
    /// its two sides (the blowing arches: the casting arch is under the
    /// casting house before it) and a string course; close to it has its
    /// plinth, two string courses and an oversailing head, the arches'
    /// rings, and the iron plates of the ties that bind it. It gives back
    /// the middle of its top.
    pub fn furnace_stack(&mut self, x: f32, z: f32, foot: f32, head: f32, h: f32, col: Rgba) -> Vec3 {
        let p = self.at(x, z);
        let k = std::f32::consts::FRAC_1_SQRT_2;
        let c = Column { p, r0: foot * k, r1: head * k, h, sink: SINK, sides: 4, turn: std::f32::consts::FRAC_PI_4 - self.site.yaw };
        let soot = *self.pal.smoke.soot;
        c.build(self.solid, col, Some(soot));
        if self.far() || self.going_up() {
            return p + Vec3::Y * h;
        }
        let near = self.near();
        let along = self.along_front();
        let sides = [c.face_toward(along), c.face_toward(-along)];
        let (aw, ah) = (foot * 0.34, (foot * 0.36).min(h * 0.3));
        for i in sides {
            c.arch(self.solid, i, aw, 0.0, ah, 0.0006, if near { 4 } else { 2 }, soot, near.then(|| shade(col, 1.1)));
        }
        if !near {
            c.band(self.solid, h * 0.62, 0.0016, 0.0006, false, shade(col, 1.08));
            return p + Vec3::Y * h;
        }
        c.band(self.solid, 0.0, 0.0034, 0.0009, true, shade(col, 0.9));
        for t in [0.4, 0.7] {
            c.band(self.solid, h * t, 0.0018, 0.0007, true, shade(col, 1.08));
        }
        // (a hair below the top, so that its upper face is not in the plane of the stack's own)
        c.band(self.solid, h - 0.0023, 0.002, 0.001, true, shade(col, 0.94));
        // the plates of the iron ties, under each string course
        let iron = self.tone(*self.pal.iron.plate);
        for i in sides {
            for t in [0.4, 0.7] {
                let y = h * t - 0.0032;
                for s in [-1.0f32, 1.0] {
                    let u = s * (c.half(y) - 0.0036);
                    let q = [c.on(i, u - 0.0009, y - 0.0009, 0.0005), c.on(i, u + 0.0009, y - 0.0009, 0.0005), c.on(i, u + 0.0009, y + 0.0009, 0.0005), c.on(i, u - 0.0009, y + 0.0009, 0.0005)];
                    self.solid.quad_toward(q, [iron; 4], c.facing(i));
                }
            }
        }
        p + Vec3::Y * h
    }

    /// A blast furnace cased in brick or in iron plate: round, `r` at the
    /// ground and narrowing to its throat, `h` high. From the play distance
    /// it stands on the brick drum of its hearth; close to that has the
    /// arches of its tuyeres all round, the blast main rings it above them,
    /// and hoops bind its casing. It gives back the middle of its top.
    pub fn furnace_shell(&mut self, x: f32, z: f32, r: f32, h: f32, shell: Rgba) -> Vec3 {
        let p = self.at(x, z);
        let c = Column { p, r0: r, r1: r * 0.72, h, sink: SINK, sides: 8, turn: 0.0 };
        c.build(self.solid, shell, Some(*self.pal.smoke.soot));
        if self.far() || self.going_up() {
            return p + Vec3::Y * h;
        }
        let brick = self.tone(*self.pal.brick.dark);
        let hearth = h * 0.15;
        let drum = Column { r0: r + 0.0016, r1: r + 0.0016, h: hearth, ..c };
        drum.build(self.solid, brick, Some(shade(brick, 1.08)));
        if !self.near() {
            return p + Vec3::Y * h;
        }
        let soot = *self.pal.smoke.soot;
        for i in 0..drum.sides {
            drum.arch(self.solid, i, drum.half(0.0) * 0.8, 0.0, hearth * 0.72, 0.0005, 3, soot, None);
        }
        let iron = self.tone(*self.pal.iron.plate);
        // the blast main, and the hoops: paler than an iron casing, darker than a brick one
        c.band(self.solid, hearth + 0.0018, 0.0024, 0.0016, true, shade(iron, 0.8));
        let hoop = if shell == iron { shade(shell, 1.45) } else { shade(iron, 0.9) };
        for t in [0.3, 0.44, 0.58, 0.72] {
            c.band(self.solid, h * t, 0.0018, 0.0005, false, hoop);
        }
        p + Vec3::Y * h
    }

    /// The gallery a furnace is charged from, round its throat: an iron
    /// platform `r` out from the middle at `y` above the ground at a place,
    /// and close to its posts and its rail.
    pub fn charging_gallery(&mut self, x: f32, z: f32, r: f32, y: f32) {
        if self.going_up() || self.derelict() {
            return;
        }
        let p = self.at(x, z);
        let plate = self.tone(*self.pal.iron.plate);
        let tall = 0.006 * self.g;
        self.solid.frustum_turned(p + Vec3::Y * y, r, r, tall, 8, 0.0, 0.0, plate, Some(plate));
        if !self.near() {
            return;
        }
        let (top, rail) = (y + tall, 0.0042);
        let dark = shade(plate, 0.7);
        for i in 0..8 {
            let a = i as f32 / 8.0 * TAU;
            let foot = p + Vec3::new(a.cos() * (r - 0.0005), top, a.sin() * (r - 0.0005));
            self.solid.beam(foot, foot + Vec3::Y * rail, 0.0009, dark);
        }
        self.solid.frustum_turned(p + Vec3::Y * (top + rail - 0.0009), r - 0.0002, r - 0.0002, 0.0009, 8, 0.0, 0.0, dark, None);
    }

    /// Hoops round a plain flue that stands at a place (`Ctx::flue`), at
    /// shares of its height: close to only.
    #[allow(clippy::too_many_arguments)]
    pub fn hoops(&mut self, x: f32, z: f32, h: f32, r0: f32, r1: f32, at: &[f32], col: Rgba) {
        if !self.near() || self.going_up() {
            return;
        }
        let c = Column { p: self.at(x, z), r0, r1, h, sink: SINK, sides: 8, turn: 0.0 };
        for &t in at {
            c.band(self.solid, h * t, 0.0018, 0.0005, false, col);
        }
    }

    /// The tower of a furnace's lift: four iron legs `side` apart round a
    /// place, `h` high, under the platform the barrows are run off at; close
    /// to, braced from leg to leg.
    pub fn lift(&mut self, x: f32, z: f32, side: f32, h: f32) {
        let plate = self.tone(*self.pal.iron.plate);
        let g = self.g;
        let k = side * 0.42;
        let feet = [(-k, -k), (k, -k), (k, k), (-k, k)].map(|(dx, dz)| self.at(x + dx, z + dz));
        for foot in feet {
            self.solid.beam(foot, foot + Vec3::Y * h, 0.003 * g, plate);
        }
        self.solid.block(self.at(x, z) + Vec3::Y * h, side, side, 0.004 * g, self.site.yaw, plate, Some(plate), 0.0);
        if !self.near() {
            return;
        }
        // a brace across every side in each lift of the tower, leaning one way and then the other
        let lifts = ((h / (2.0 * k * 1.6)).round() as usize).clamp(2, 6);
        let dark = shade(plate, 0.82);
        for i in 0..4 {
            let (a, b) = (feet[i], feet[(i + 1) % 4]);
            for j in 0..lifts {
                let (y0, y1) = (h * j as f32 / lifts as f32, h * (j + 1) as f32 / lifts as f32);
                let (from, to) = if j % 2 == 0 { (a, b) } else { (b, a) };
                self.solid.beam(from + Vec3::Y * y0, to + Vec3::Y * y1, 0.0015 * g, dark);
                if j > 0 {
                    self.solid.beam(a + Vec3::Y * y0, b + Vec3::Y * y0, 0.0015 * g, dark);
                }
            }
        }
    }

    /// A way for barrows from one place to another, each a height above the
    /// ground there: a gantry along the furnace tops, a bridge from the
    /// bank, an incline. A beam `wide`; close to it has a rail either side.
    pub fn barrow_way(&mut self, from: (f32, f32, f32), to: (f32, f32, f32), wide: f32, col: Rgba) {
        let (a, b) = (self.at(from.0, from.1) + Vec3::Y * from.2, self.at(to.0, to.1) + Vec3::Y * to.2);
        self.solid.beam(a, b, wide, col);
        if !self.near() {
            return;
        }
        let dir = (b - a).normalize_or_zero();
        let side = Vec3::new(-dir.z, 0.0, dir.x).normalize_or_zero() * (wide * 0.5 - 0.0004);
        let (up, dark) = (Vec3::Y * (wide * 0.5 + 0.0036), shade(col, 0.72));
        let posts = (((b - a).length() / 0.012).round() as usize).clamp(1, 12);
        for s in [-1.0f32, 1.0] {
            self.solid.beam(a + side * s + up, b + side * s + up, 0.0009, dark);
            for i in 0..=posts {
                let foot = a + (b - a) * (i as f32 / posts as f32) + side * s;
                self.solid.beam(foot, foot + up, 0.0009, dark);
            }
        }
    }

    /// A bottle kiln: a round brick chamber `r` across at the ground, under
    /// the hovel that narrows to its neck (or, with `whole` unset, the stump
    /// of one). It turns its wicket to the front of the yard. From the play
    /// distance the wicket and the band at its shoulder; close to its
    /// firing holes all round, the iron bands that hold the hovel, and the
    /// rim of its neck. It gives back the middle of its top.
    pub fn bottle_kiln(&mut self, x: f32, z: f32, r: f32, whole: bool, brick: Rgba) -> Vec3 {
        let (g, p) = (self.g, self.at(x, z));
        let front = Quat::from_rotation_y(self.site.yaw) * Vec3::Z;
        const SIDES: usize = 7;
        // a face square to the front of the yard, for the wicket
        let turn = front.z.atan2(front.x) - PI / SIDES as f32;
        let (shoulder, wide) = (0.018 * g, r * 1.15);
        let chamber = Column { p, r0: r, r1: wide, h: shoulder, sink: SINK, sides: SIDES, turn };
        chamber.build(self.solid, brick, None);
        if self.going_up() {
            return p + Vec3::Y * shoulder;
        }
        let (tall, neck) = (if whole { 0.034 * g } else { 0.02 * g }, if whole { r * 0.38 } else { r * 0.7 });
        let soot = *self.pal.smoke.soot;
        let hovel = Column { p: p + Vec3::Y * shoulder, r0: wide, r1: neck, h: tall, sink: 0.0, sides: SIDES, turn };
        hovel.build(self.solid, brick, Some(soot));
        let top = p + Vec3::Y * (shoulder + tall);
        if self.far() {
            return top;
        }
        let near = self.near();
        let wicket = chamber.face_toward(front);
        let iron = shade(brick, 0.74);
        chamber.arch(self.solid, wicket, r * 0.42, 0.0, shoulder * 0.74, 0.0006, if near { 4 } else { 2 }, soot, near.then(|| shade(brick, 1.14)));
        chamber.band(self.solid, shoulder - 0.0016, 0.0016, 0.0006, false, iron);
        if !near {
            return top;
        }
        for i in (0..SIDES).filter(|&i| i != wicket) {
            chamber.arch(self.solid, i, 0.0038, 0.0, 0.0048, 0.0005, 3, soot, Some(shade(brick, 1.14)));
        }
        if whole {
            for t in [0.3, 0.62] {
                hovel.band(self.solid, tall * t, 0.0018, 0.0005, false, iron);
            }
            hovel.band(self.solid, tall - 0.0019, 0.0016, 0.0008, true, shade(brick, 0.9));
        }
        top
    }

    // ------------------------------------------------------------------ the plant of the twentieth century

    /// A cooling tower: a shell of concrete `r` across at its foot and `h`
    /// high, that narrows to a waist two thirds of the way up and flares a
    /// little to its rim, standing on a ring of raking legs. From far off the
    /// shell in two lengths, on the ground; from the play distance in four,
    /// lifted on the dark of the open ring under it, with a lip at its rim;
    /// close to in six, with the legs themselves, two to each side, crossing
    /// as they rise. Going up, the legs and the first lifts of the shell, the
    /// same at every level. The caller tones `col`. It gives back the middle
    /// of its rim, where the steam rises.
    pub fn cooling_tower(&mut self, x: f32, z: f32, r: f32, h: f32, col: Rgba) -> Vec3 {
        let p = self.at(x, z);
        let inside = shade(col, 0.42);
        // the shell's profile: shares of its height, and of its radius at the foot
        const SHAPE: [(f32, f32); 6] = [(0.0, 1.0), (0.2, 0.86), (0.45, 0.72), (0.7, 0.64), (0.88, 0.645), (1.0, 0.68)];
        if self.going_up() {
            let c = Column { p, r0: r, r1: r * 0.86, h: h * 0.2, sink: SINK, sides: 8, turn: 0.0 };
            c.build(self.solid, col, Some(inside));
            return p + Vec3::Y * h * 0.2;
        }
        let (sides, steps): (usize, &[usize]) = if self.far() { (8, &[0, 3, 5]) } else if self.near() { (14, &[0, 1, 2, 3, 4, 5]) } else { (10, &[0, 2, 3, 5]) };
        // (from the play distance the shell stands on its legs: the open ring under it is dark)
        let foot = if self.far() { 0.0 } else { 0.07 * h };
        let at = |k: usize| (foot + (h - foot) * SHAPE[k].0, r * SHAPE[k].1);
        for w in steps.windows(2) {
            let ((y0, r0), (y1, r1)) = (at(w[0]), at(w[1]));
            let last = w[1] == 5;
            let sink = if y0 == 0.0 { SINK } else { 0.0 };
            self.solid.frustum_turned(p + Vec3::Y * y0, r0, r1, y1 - y0, sides, 0.0, sink, col, last.then_some(inside));
        }
        if self.far() {
            return p + Vec3::Y * h;
        }
        // the open ring under the shell, in shadow, and the lip at its rim
        self.solid.frustum_turned(p, r * 0.9, r * 0.9, foot, sides, 0.0, SINK, *self.pal.smoke.soot, None);
        let rim = r * SHAPE[5].1;
        self.solid.frustum_turned(p + Vec3::Y * (h - 0.0016), rim + 0.0007, rim + 0.0007, 0.0016, sides, 0.0, 0.0, shade(col, 1.06), None);
        if self.near() {
            // the raking legs, two to each side of the shell, crossing as they rise
            let leg = shade(col, 0.86);
            for i in 0..sides {
                let a = |k: f32| (k / sides as f32) * TAU;
                let ground = |k: f32| p + Vec3::new(a(k).cos() * r * 0.97, -0.002, a(k).sin() * r * 0.97);
                let under = |k: f32| p + Vec3::new(a(k).cos() * r * 0.97, foot, a(k).sin() * r * 0.97);
                let i = i as f32;
                self.solid.beam(ground(i), under(i + 0.5), 0.0012, leg);
                self.solid.beam(ground(i + 0.5), under(i), 0.0012, leg);
            }
        }
        p + Vec3::Y * h
    }

    /// A round tank of iron plate `r` across and `h` to its eaves, under a
    /// shallow cone: from the play distance with a band at its eaves and the
    /// dark of its foot; close to with its hoops, the ladder up its side
    /// toward the front of the yard, and the rail round its roof. Going up,
    /// half its plating, open, the same at every level. The caller tones
    /// `col`. It gives back the top of its roof.
    pub fn tank(&mut self, x: f32, z: f32, r: f32, h: f32, col: Rgba) -> Vec3 {
        let p = self.at(x, z);
        if self.going_up() {
            Column { p, r0: r, r1: r, h: h * 0.5, sink: SINK, sides: 8, turn: 0.0 }.build(self.solid, col, Some(shade(col, 0.5)));
            return p + Vec3::Y * h * 0.5;
        }
        let sides = if self.far() { 8 } else if self.near() { 14 } else { 10 };
        let shell = Column { p, r0: r, r1: r, h, sink: SINK, sides, turn: 0.0 };
        shell.build(self.solid, col, None);
        let rise = r * 0.2;
        self.solid.frustum_turned(p + Vec3::Y * h, r + 0.0006, 0.0, rise, sides, 0.0, 0.0, shade(col, 1.12), None);
        let top = p + Vec3::Y * (h + rise);
        if self.far() {
            return top;
        }
        let dark = shade(col, 0.7);
        shell.band(self.solid, h - 0.0018, 0.0016, 0.0005, true, shade(col, 0.9));
        shell.band(self.solid, 0.0, 0.0014, 0.0006, true, dark);
        if !self.near() {
            return top;
        }
        for t in [0.36, 0.68] {
            shell.band(self.solid, h * t, 0.0008, 0.0003, false, shade(col, 0.92));
        }
        // the ladder, on the side toward the front of the yard
        let front = Quat::from_rotation_y(self.site.yaw) * Vec3::Z;
        self.ladder(p, front, r, h, dark);
        // the rail round the roof
        for i in 0..8 {
            let a = i as f32 / 8.0 * TAU;
            let foot = p + Vec3::new(a.cos() * (r - 0.0006), h, a.sin() * (r - 0.0006));
            self.solid.beam(foot, foot + Vec3::Y * 0.0036, 0.0006, dark);
        }
        self.solid.frustum_turned(p + Vec3::Y * (h + 0.003), r - 0.0003, r - 0.0003, 0.0006, sides, 0.0, 0.0, dark, None);
        top
    }

    /// A ladder up the outside of something round, `r` out from `p` on the
    /// side toward `dir`, from the ground to `h`: two stiles and their rungs. Close to only.
    fn ladder(&mut self, p: Vec3, dir: Vec3, r: f32, h: f32, col: Rgba) {
        let out = Vec3::new(dir.x, 0.0, dir.z).normalize_or_zero();
        let side = Vec3::new(-out.z, 0.0, out.x) * 0.0012;
        let base = p + out * (r + 0.0009);
        for s in [-1.0f32, 1.0] {
            self.solid.beam(base + side * s - Vec3::Y * 0.002, base + side * s + Vec3::Y * (h + 0.003), 0.0005, col);
        }
        let rungs = ((h / 0.003).floor() as usize).clamp(2, 40);
        for k in 1..rungs {
            let y = Vec3::Y * (h * k as f32 / rungs as f32);
            self.solid.beam(base - side + y, base + side + y, 0.00045, col);
        }
    }

    /// A column of a chemical works: a slim tower of iron plate `r` across
    /// and `h` high under a domed head. From the play distance the platforms
    /// that ring it, one to each lift of its height; close to their posts and
    /// rails, the ladder that climbs it toward the front of the yard, and the
    /// seams of its plates. Going up, its lower half, open, the same at every
    /// level. The caller tones `col`. It gives back the top of its head.
    pub fn column(&mut self, x: f32, z: f32, r: f32, h: f32, col: Rgba) -> Vec3 {
        let p = self.at(x, z);
        if self.going_up() {
            Column { p, r0: r, r1: r, h: h * 0.5, sink: SINK, sides: 6, turn: 0.0 }.build(self.solid, col, Some(shade(col, 0.5)));
            return p + Vec3::Y * h * 0.5;
        }
        let sides = if self.far() { 6 } else if self.near() { 10 } else { 8 };
        let shell = Column { p, r0: r, r1: r, h, sink: SINK, sides, turn: 0.0 };
        shell.build(self.solid, col, None);
        let head = r * 0.7;
        self.solid.frustum_turned(p + Vec3::Y * h, r, r * 0.55, head * 0.6, sides, 0.0, 0.0, shade(col, 1.1), Some(shade(col, 1.16)));
        let top = p + Vec3::Y * (h + head * 0.6);
        if self.far() {
            return top;
        }
        let deck = shade(col, 0.78);
        let lifts = ((h / 0.032).round() as usize).clamp(1, 4);
        let reach = r + 0.0035;
        for k in 1..=lifts {
            let y = h * k as f32 / (lifts as f32 + 0.6);
            self.solid.frustum_turned(p + Vec3::Y * y, reach, reach, 0.0014, 8, 0.0, 0.0, deck, Some(deck));
            if self.near() {
                for i in 0..8 {
                    let a = (i as f32 + 0.5) / 8.0 * TAU;
                    let foot = p + Vec3::new(a.cos() * (reach - 0.0004), y + 0.0014, a.sin() * (reach - 0.0004));
                    self.solid.beam(foot, foot + Vec3::Y * 0.0034, 0.0005, deck);
                }
                self.solid.frustum_turned(p + Vec3::Y * (y + 0.0042), reach - 0.0002, reach - 0.0002, 0.0005, 8, 0.0, 0.0, deck, None);
            }
        }
        if self.near() {
            for t in [0.2, 0.5, 0.8] {
                shell.band(self.solid, h * t, 0.0007, 0.0003, false, shade(col, 0.9));
            }
            let front = Quat::from_rotation_y(self.site.yaw + 0.6) * Vec3::Z;
            self.ladder(p, front, r, h, deck);
        }
        top
    }

    /// A rack of pipes from one place in the yard to another, `h` above the
    /// ground: frames of two legs and a beam, about a bay and a half apart,
    /// carrying `pipes` pipes along their tops. Not drawn from far off; close
    /// to, thinner and braced. Going up, the frames alone; derelict, the frames
    /// and the lowest pipe, the others taken down for scrap.
    pub fn pipe_rack(&mut self, a: (f32, f32), b: (f32, f32), h: f32, pipes: usize) {
        if self.far() {
            return;
        }
        let (pa, pb) = (self.at(a.0, a.1), self.at(b.0, b.1));
        let run = Vec3::new(pb.x - pa.x, 0.0, pb.z - pa.z);
        let len = run.length();
        if len < 0.004 {
            return;
        }
        let dir = run / len;
        let across = Vec3::new(-dir.z, 0.0, dir.x);
        let pipes = pipes.clamp(1, 4);
        let wide = 0.0012 + 0.0026 * pipes as f32;
        let near = self.near();
        let (t, pipe) = if near { (0.0009, 0.0014) } else { (0.0013, 0.0017) };
        let iron = self.tone(*self.pal.iron.plate);
        let frames = ((len / 0.021).ceil() as usize).max(1);
        for i in 0..=frames {
            let f = i as f32 / frames as f32;
            let foot = pa + (pb - pa) * f;
            let (l, r) = (foot - across * wide * 0.5, foot + across * wide * 0.5);
            self.solid.beam(l - Vec3::Y * 0.002, l + Vec3::Y * h, t, iron);
            self.solid.beam(r - Vec3::Y * 0.002, r + Vec3::Y * h, t, iron);
            self.solid.beam(l + Vec3::Y * h, r + Vec3::Y * h, t, iron);
            if near {
                // the knee braces under the beam
                self.solid.beam(l + Vec3::Y * (h - 0.003), l + across * 0.002 + Vec3::Y * h, 0.0006, shade(iron, 0.85));
                self.solid.beam(r + Vec3::Y * (h - 0.003), r - across * 0.002 + Vec3::Y * h, 0.0006, shade(iron, 0.85));
            }
        }
        if self.going_up() {
            return;
        }
        let shown = if self.derelict() { 1 } else { pipes };
        for k in 0..shown {
            let off = across * (-wide * 0.5 + 0.0006 + 0.0026 * (k as f32 + 0.5));
            let y = Vec3::Y * (h + t * 0.5 + pipe * 0.5);
            self.solid.beam(pa + off + y, pb + off + y, pipe, shade(iron, 1.2 - 0.12 * k as f32));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::Detail;
    use crate::palette::Palette;
    use crate::site::Site;
    use crate::works::{Kind, State, Works};

    /// Build one part in a yard at a level and in a state, and hand back what it is made of.
    fn part(detail: Detail, state: State, yaw: f32, build: impl Fn(&mut Ctx)) -> MeshBuf {
        let pal = Palette::standard();
        let (mut solid, mut smoke, mut flat) = (MeshBuf::default(), MeshBuf::default(), MeshBuf::default());
        let site = Site::level(Vec3::ZERO, yaw);
        let mut c = Ctx::new(&mut solid, &mut smoke, &mut flat, &site, &Works { detail, state, founded: 1850, ..Works::new(Kind::CokeFurnace, 3.0) }, &pal);
        build(&mut c);
        assert_eq!(c.taken(), 0, "a part of the plant takes none of the works' own numbers, so it is the same at every level");
        assert!(solid.is_sound());
        solid
    }

    /// How far a mesh reaches from the upright through the origin.
    fn reach(b: &MeshBuf) -> f32 {
        b.pos.iter().map(|p| p[0].hypot(p[2])).fold(0.0, f32::max)
    }

    #[test]
    fn each_part_of_the_plant_draws_more_the_nearer_it_is_seen_and_keeps_its_shape() {
        let pal = Palette::standard();
        let col = *pal.stone.works;
        let parts: [(&str, Box<dyn Fn(&mut Ctx)>, f32); 4] = [
            ("a stone furnace", Box::new(move |c: &mut Ctx| { c.furnace_stack(0.0, 0.0, 0.034, 0.024, 0.075, col); }), 0.034 * std::f32::consts::FRAC_1_SQRT_2),
            ("a cased furnace", Box::new(move |c: &mut Ctx| { c.furnace_shell(0.0, 0.0, 0.02, 0.1, col); }), 0.02),
            ("a bottle kiln", Box::new(move |c: &mut Ctx| { c.bottle_kiln(0.0, 0.0, 0.015, true, col); }), 0.015 * 1.15),
            ("a lift", Box::new(move |c: &mut Ctx| c.lift(0.0, 0.0, 0.014, 0.1)), 0.014 * 0.42 * std::f32::consts::SQRT_2),
        ];
        for (name, build, body) in &parts {
            for yaw in [0.0, 0.7] {
                let [far, play, near] = Detail::ALL.map(|d| part(d, State::Working, yaw, build));
                assert!(far.triangles() <= play.triangles() && play.triangles() < near.triangles(), "{name}: {} {} {}", far.triangles(), play.triangles(), near.triangles());
                // what is added stands on the shape and hardly beyond it: the yard set aside holds it at every level
                for (b, level) in [(&far, "far"), (&play, "play"), (&near, "near")] {
                    assert!(reach(b) < body + 0.004, "{name} reaches {} at the {level} level, its body {body}", reach(b));
                }
                let height = |b: &MeshBuf| b.bounds().unwrap().1.y;
                assert!((height(&far) - height(&near)).abs() < 1e-5 && (height(&far) - height(&play)).abs() < 1e-5, "{name} is as tall at every level");
                // going up, it is the plain shape at every level
                let rising = Detail::ALL.map(|d| part(d, State::GoingUp, yaw, build).triangles());
                assert!(rising[0] == rising[1] && rising[1] == rising[2] || *name == "a lift", "{name} going up: {rising:?}");
            }
        }
    }

    #[test]
    fn a_stone_furnace_shows_its_arches_to_the_sides_of_the_yard() {
        let pal = Palette::standard();
        let (col, soot) = (*pal.stone.works, *pal.smoke.soot);
        for yaw in [0.0f32, 0.9, -2.1] {
            let b = part(Detail::Play, State::Working, yaw, |c| {
                c.furnace_stack(0.0, 0.0, 0.034, 0.024, 0.075, col);
            });
            // the dark of the arches low on the stack looks along the yard's front, one each way
            let along = Quat::from_rotation_y(yaw) * Vec3::X;
            let mut looks = [0, 0];
            for (i, c) in b.col.iter().enumerate() {
                if *c == soot && b.pos[i][1] < 0.02 {
                    let n = Vec3::from_array(b.nrm[i]);
                    assert!(n.dot(along).abs() > 0.9, "an arch looks {n:?} in a yard turned {yaw}");
                    looks[(n.dot(along) > 0.0) as usize] += 1;
                }
            }
            assert!(looks[0] > 0 && looks[0] == looks[1], "{looks:?}");
        }
    }

    #[test]
    fn a_kiln_turns_its_wicket_to_the_front_of_the_yard() {
        let pal = Palette::standard();
        let (brick, soot) = (*pal.brick.red, *pal.smoke.soot);
        for yaw in [0.0f32, 0.9, -2.1] {
            let b = part(Detail::Play, State::Working, yaw, |c| {
                c.bottle_kiln(0.0, 0.0, 0.015, true, brick);
            });
            let front = Quat::from_rotation_y(yaw) * Vec3::Z;
            let mut dark = 0;
            for (i, c) in b.col.iter().enumerate() {
                if *c == soot && b.pos[i][1] < 0.015 {
                    // (the chamber's wall leans out a little as it rises, so the wicket looks a little down)
                    assert!(Vec3::from_array(b.nrm[i]).dot(front) > 0.9);
                    dark += 1;
                }
            }
            assert!(dark > 0);
        }
    }
}
