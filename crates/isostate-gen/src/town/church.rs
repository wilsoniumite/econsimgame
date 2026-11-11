use glam::{Vec2, Vec3};

use crate::colour::{shade, Rgba};
use crate::hash::{hash2, Seed};
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::site::Site;

/// A town's parish church.
#[derive(Clone, Copy, Debug)]
pub struct Church {
    /// a big town's church: a quarter larger, with an aisle either side of the nave
    pub big: bool,
    /// a spire on the tower, as a big town's has and as the Victorians gave a
    /// small town's when they restored it; without one the tower has a low
    /// cap and a pinnacle at each corner
    pub spire: bool,
    /// the town's own numbers, which place the yews and the gravestones
    pub seed: Seed,
}

impl Church {
    fn scale(&self) -> f32 {
        if self.big { 1.25 } else { 1.0 }
    }

    /// How wide the church is across its nave (and its aisles).
    fn body_width(&self) -> f32 {
        self.scale() * if self.big { NAVE_WIDTH + 2.0 * AISLE_WIDTH } else { NAVE_WIDTH }
    }

    /// The churchyard: how wide it is along the site's x and how deep along
    /// its z. The town keeps this ground clear of houses.
    pub fn yard(&self) -> (f32, f32) {
        (self.body_width() + 2.0 * YARD_SIDE, self.scale() * (TOWER + NAVE_LENGTH + CHANCEL_LENGTH) + YARD_FRONT + YARD_BACK)
    }

    /// How high the tower's walls stand.
    pub fn tower_height(&self) -> f32 {
        self.scale() * TOWER_HEIGHT
    }

    /// Where the church stands off a market place whose middle is at
    /// `square`, out along `bearing` (a unit vector on the map): the middle of
    /// its churchyard, and the yaw that turns its front to the market place.
    pub fn beside_square(&self, square: Vec2, bearing: Vec2) -> (Vec2, f32) {
        (square + bearing * (GATE_FROM_SQUARE + self.yard().1 * 0.5), (-bearing.x).atan2(-bearing.y))
    }

    /// Whether a map point is in the churchyard or within `margin` of it,
    /// when the yard's middle is at `at` and the church is turned by `yaw`.
    pub fn yard_holds(&self, at: Vec2, yaw: f32, p: Vec2, margin: f32) -> bool {
        let d = p - at;
        let (sin, cos) = yaw.sin_cos();
        let (x, z) = (d.x * cos - d.y * sin, d.x * sin + d.y * cos);
        let (w, depth) = self.yard();
        x.abs() < w * 0.5 + margin && z.abs() < depth * 0.5 + margin
    }

    /// Whether a house plot with its middle at `p` would crowd the church:
    /// stand in the churchyard, hard against its sides or its back, or in
    /// the way from its gate to the market place. Houses may back onto the
    /// yard's front wall either side of the gate.
    pub fn crowded_by(&self, at: Vec2, yaw: f32, p: Vec2) -> bool {
        let d = p - at;
        let (sin, cos) = yaw.sin_cos();
        let (x, z) = (d.x * cos - d.y * sin, d.x * sin + d.y * cos);
        let (w, depth) = self.yard();
        let in_yard = x.abs() < w * 0.5 + 0.026 && z > -depth * 0.5 - 0.026 && z < depth * 0.5 + 0.004;
        let in_the_way = x.abs() < 0.03 && z >= depth * 0.5 && z < depth * 0.5 + 0.07;
        in_yard || in_the_way
    }
}

/// How far the churchyard's gate is from the middle of the market place:
/// just beyond the widest a market place gets, so that a church never has to
/// move as its town grows.
pub const GATE_FROM_SQUARE: f32 = 0.102;

// the sizes of a small town's church, in map units (a house plot is 0.05 by 0.036 and a storey 0.024)
// (the nave's ridge is as high as a three-storey house's, and the tower twice that)
const TOWER: f32 = 0.042;
const TOWER_HEIGHT: f32 = 0.165;
const SPIRE_HEIGHT: f32 = 0.12;
const NAVE_WIDTH: f32 = 0.048;
const NAVE_LENGTH: f32 = 0.07;
const NAVE_WALL: f32 = 0.054;
const NAVE_RISE: f32 = 0.034;
const AISLE_WIDTH: f32 = 0.014;
const CHANCEL_LENGTH: f32 = 0.026;
// the churchyard's ground beyond the church: before the west door, behind the chancel, and down each side
const YARD_FRONT: f32 = 0.012;
const YARD_BACK: f32 = 0.010;
const YARD_SIDE: f32 = 0.022;

/// A parish church in its churchyard: a west tower (with a spire, or a low
/// cap and pinnacles), a nave under lead with tall windows, a lower chancel
/// at the east end, and round it a walled yard of grass with a gate, a few
/// yews and gravestones.
///
/// The site is the middle of the churchyard. The church runs along the
/// site's z with the tower and its door at the front (+z), where the gate
/// is: a town stands it with its front to the market place.
pub fn church(solid: &mut MeshBuf, site: &Site, c: &Church, pal: &Palette) {
    let k = c.scale();
    let yaw = site.yaw;
    let along = yaw + std::f32::consts::FRAC_PI_2;
    let (stone, lead, glass) = (*pal.stone.church, *pal.roofs.church_lead, *pal.glass.window);
    let (yard_w, yard_d) = c.yard();
    // the church stands on a level floor at the height of the site's middle
    let floor = |dx: f32, dz: f32| {
        let p = site.at(dx, dz);
        Vec3::new(p.x, site.origin.y, p.z)
    };

    // ---- the churchyard: grass draped on the ground, a wall round it, a gate at the front
    let grass = *pal.ground.churchyard_grass;
    let lift = Vec3::Y * 0.004;
    let (nx, nz) = (3, 5);
    let corner = |i: usize, j: usize| site.at(-yard_w * 0.5 + yard_w * i as f32 / nx as f32, -yard_d * 0.5 + yard_d * j as f32 / nz as f32);
    for j in 0..nz {
        for i in 0..nx {
            solid.quad(corner(i, j) + lift, corner(i, j + 1) + lift, corner(i + 1, j + 1) + lift, corner(i + 1, j) + lift, grass);
        }
    }
    let wall_stone = *pal.stone.churchyard_wall;
    let gate = 0.009;
    let (hw, hd) = (yard_w * 0.5, yard_d * 0.5);
    let mut run = |from: (f32, f32), to: (f32, f32), parts: usize| {
        for i in 0..parts {
            let at = |t: f32| site.at(from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            wall(solid, at(i as f32 / parts as f32), at((i + 1) as f32 / parts as f32), 0.008, 0.0045, wall_stone);
        }
    };
    run((gate, hd), (hw, hd), 1);
    run((hw, hd), (hw, -hd), nz);
    run((hw, -hd), (-hw, -hd), nx);
    run((-hw, -hd), (-hw, hd), nz);
    run((-hw, hd), (-gate, hd), 1);
    for side in [-1.0f32, 1.0] {
        solid.block(site.at(side * gate, hd), 0.006, 0.006, 0.014, yaw, wall_stone, Some(shade(wall_stone, 1.06)), 0.012);
    }
    // the path from the gate to the west door
    let path = *pal.ground.market_paving;
    let lift = Vec3::Y * 0.005;
    solid.quad(site.at(-0.006, hd - YARD_FRONT - 0.001) + lift, site.at(-0.006, hd) + lift, site.at(0.006, hd) + lift, site.at(0.006, hd - YARD_FRONT - 0.001) + lift, path);

    // ---- the church: tower, nave and chancel in a line from the front to the back
    let tower_z = hd - YARD_FRONT - TOWER * k * 0.5;
    let nave_z = tower_z - (TOWER + NAVE_LENGTH) * k * 0.5;
    let chancel_z = nave_z - (NAVE_LENGTH + CHANCEL_LENGTH) * k * 0.5;
    let (nave_w, nave_len, nave_h, nave_rise) = (NAVE_WIDTH * k, NAVE_LENGTH * k, NAVE_WALL * k, NAVE_RISE * k);

    // the nave, its ridge along the site's z
    solid.block(floor(0.0, nave_z), nave_len, nave_w, nave_h, along, stone, None, 0.03);
    solid.gable(floor(0.0, nave_z), nave_len, nave_w, nave_h, nave_rise, along, 0.003, lead, stone);
    // the chancel: narrower and lower
    let (ch_w, ch_len, ch_h) = (nave_w * 0.72, CHANCEL_LENGTH * k, nave_h * 0.78);
    solid.block(floor(0.0, chancel_z), ch_len, ch_w, ch_h, along, stone, None, 0.03);
    solid.gable(floor(0.0, chancel_z), ch_len, ch_w, ch_h, nave_rise * 0.72, along, 0.003, lead, stone);
    // the east window: three lancets, the middle one the tallest
    let east = chancel_z - ch_len * 0.5 - 0.0006;
    for (x, top) in [(-0.29f32, 0.8), (0.0, 0.95), (0.29, 0.8)] {
        pane(solid, site, ((x + 0.09) * ch_w, east), ((x - 0.09) * ch_w, east), ch_h * 0.34, ch_h * top, glass);
    }

    // a big town's church has a lean-to aisle either side, and the nave's windows are in its clerestory
    let aisle = AISLE_WIDTH * k;
    let outer = if c.big { nave_w * 0.5 + aisle } else { nave_w * 0.5 };
    let wall_h = if c.big { nave_h * 0.5 } else { nave_h };
    if c.big {
        for side in [-1.0f32, 1.0] {
            let (x0, x1) = (side * nave_w * 0.5, side * outer);
            let (z0, z1) = (nave_z - nave_len * 0.5, nave_z + nave_len * 0.5);
            let at = |x: f32, y: f32, z: f32| floor(x, z) + Vec3::Y * y;
            let (low, high) = (wall_h, nave_h * 0.74);
            // the outer wall, the two ends and the roof up to the nave wall
            face(solid, at(x1, -0.03, z0), at(x1, -0.03, z1), at(x1, low, z1), at(x1, low, z0), side, shade(stone, 0.96));
            face(solid, at(x1, low, z0), at(x1, low, z1), at(x0, high, z1), at(x0, high, z0), side, lead);
            for (z, turn) in [(z1, -side), (z0, side)] {
                face(solid, at(x0, -0.03, z), at(x1, -0.03, z), at(x1, low, z), at(x0, high, z), turn, stone);
            }
        }
    }
    // tall windows down both sides, and a porch in place of one on the south side
    let bays = if c.big { 5 } else { 4 };
    let bay_z = |i: usize| nave_z - nave_len * 0.5 + nave_len * (i as f32 + 0.5) / bays as f32;
    for side in [-1.0f32, 1.0] {
        for i in 0..bays {
            let z = bay_z(i);
            let x = side * (outer + 0.0006);
            if side > 0.0 && i == bays - 2 {
                continue;
            }
            let (a, b) = if side > 0.0 { ((x, z + 0.003 * k), (x, z - 0.003 * k)) } else { ((x, z - 0.003 * k), (x, z + 0.003 * k)) };
            pane(solid, site, a, b, wall_h * 0.3, wall_h * 0.88, glass);
            if c.big {
                // the clerestory, above the aisle roof
                let x = side * (nave_w * 0.5 + 0.0006);
                let (a, b) = if side > 0.0 { ((x, z + 0.003 * k), (x, z - 0.003 * k)) } else { ((x, z - 0.003 * k), (x, z + 0.003 * k)) };
                pane(solid, site, a, b, nave_h * 0.8, nave_h * 0.96, glass);
            }
        }
    }
    let porch = floor(outer + 0.006 * k, bay_z(bays - 2));
    solid.block(porch, 0.012 * k, 0.014 * k, 0.018 * k, yaw, stone, None, 0.03);
    solid.gable(porch, 0.012 * k, 0.014 * k, 0.018 * k, 0.009 * k, yaw, 0.002, lead, stone);

    // the tower, with its west door and window and a louvred opening for the bells on every side
    let (tw, th) = (TOWER * k, c.tower_height());
    let tower = floor(0.0, tower_z);
    solid.block(tower, tw, tw, th, yaw, stone, Some(lead), 0.03);
    let front = tower_z + tw * 0.5 + 0.0006;
    pane(solid, site, (-0.005 * k, front), (0.005 * k, front), 0.0, 0.02 * k, *pal.timber.door);
    pane(solid, site, (-0.004 * k, front), (0.004 * k, front), 0.034 * k, 0.062 * k, glass);
    let r = tw * 0.5 + 0.0006;
    for (a, b) in [((-1.0f32, 1.0f32), (1.0f32, 1.0f32)), ((1.0, 1.0), (1.0, -1.0)), ((1.0, -1.0), (-1.0, -1.0)), ((-1.0, -1.0), (-1.0, 1.0))] {
        // a pair of slim openings in the middle of each face
        let (ax, az, bx, bz) = (a.0 * r, a.1 * r, b.0 * r, b.1 * r);
        let along_face = |t: f32| (ax + (bx - ax) * t, tower_z + az + (bz - az) * t);
        for t in [0.3, 0.56] {
            pane(solid, site, along_face(t), along_face(t + 0.14), th - 0.032 * k, th - 0.01 * k, *pal.timber.belfry_louvre);
        }
    }
    // a parapet round the top
    let top = tower + Vec3::Y * th;
    let parapet = 0.006 * k;
    for (dx, dz, w, d) in [(0.0, 1.0, tw, 0.004), (0.0, -1.0, tw, 0.004), (1.0, 0.0, 0.004, tw), (-1.0, 0.0, 0.004, tw)] {
        let o = site.turn() * Vec3::new(dx * (tw * 0.5 - 0.002), 0.0, dz * (tw * 0.5 - 0.002));
        solid.block(top + o, w, d, parapet, yaw, stone, Some(shade(stone, 1.04)), 0.0);
    }
    if c.spire {
        // an eight-sided spire, its flats square to the tower
        cone(solid, top, tw * 0.46, SPIRE_HEIGHT * k, 8, yaw + std::f32::consts::FRAC_PI_8, *pal.stone.church_spire);
    } else {
        // a low cap, and a pinnacle at each corner
        cone(solid, top, tw * 0.6, 0.016 * k, 4, yaw + std::f32::consts::FRAC_PI_4, lead);
        for (dx, dz) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let o = site.turn() * Vec3::new(dx * (tw * 0.5 - 0.003), 0.0, dz * (tw * 0.5 - 0.003));
            cone(solid, top + o, 0.0042 * k, 0.02 * k, 4, yaw + std::f32::consts::FRAC_PI_4, stone);
        }
    }

    // ---- yews in the corners of the yard, and gravestones down the sides
    let salt = (c.seed.u * 65535.0) as i32;
    let yew = *pal.foliage.yew;
    let spots = [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)];
    let skip = if c.big { 4 } else { (c.seed.v * 4.0) as usize % 4 };
    for (i, (sx, sz)) in spots.into_iter().enumerate() {
        if i == skip {
            continue;
        }
        let s = 0.85 + 0.4 * hash2(i as i32, 1, salt);
        let p = site.at(sx * (hw - 0.014), sz * (hd - 0.015));
        solid.frustum(p, 0.0025 * s, 0.002 * s, 0.008 * s, 3, *pal.foliage.trunk, None);
        solid.ball(p + Vec3::Y * 0.02 * s, 0.0095 * s, 0.016 * s, 5, 3, yew, hash2(i as i32, 2, salt) * 6.0);
    }
    let stones = if c.big { 9 } else { 6 };
    let grave = *pal.stone.gravestone;
    for i in 0..stones {
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        // down the nave's two sides: east of the porch on the south, further west on the north
        let x = side * (outer + 0.006 + (hw - outer - 0.012) * hash2(i, 3, salt));
        let from = if side > 0.0 { -0.5 } else { -0.25 };
        let z = nave_z + nave_len * (from + 0.5 * hash2(i, 4, salt));
        let lean = (hash2(i, 5, salt) - 0.5) * 0.5;
        solid.block(site.at(x, z), 0.0045, 0.0016, 0.006 + 0.003 * hash2(i, 6, salt), yaw + lean, grave, Some(grave), 0.006);
    }
}

/// A low wall along the ground from `p0` to `p1`: two faces and a top.
fn wall(b: &mut MeshBuf, p0: Vec3, p1: Vec3, h: f32, t: f32, col: Rgba) {
    let d = Vec3::new(p1.x - p0.x, 0.0, p1.z - p0.z).normalize_or_zero();
    let n = Vec3::new(-d.z, 0.0, d.x) * (t * 0.5);
    let (up, down) = (Vec3::Y * h, Vec3::Y * 0.012);
    b.quad(p0 + n - down, p1 + n - down, p1 + n + up, p0 + n + up, col);
    b.quad(p1 - n - down, p0 - n - down, p0 - n + up, p1 - n + up, shade(col, 0.88));
    b.quad(p0 + n + up, p1 + n + up, p1 - n + up, p0 - n + up, shade(col, 1.06));
}

/// A flat four-sided face a-b-c-d, wound one way or the other by `side` so
/// that the two sides of a church mirror each other.
fn face(b: &mut MeshBuf, p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, side: f32, col: Rgba) {
    if side > 0.0 {
        b.quad(p1, p0, p3, p2, col);
    } else {
        b.quad(p0, p1, p2, p3, col);
    }
}

/// An upright pane (a window, a door, a louvre) on a wall: from `a` to `b`
/// in the site's own frame (x, z), left to right as seen from outside,
/// between two heights above the floor.
fn pane(b: &mut MeshBuf, site: &Site, a: (f32, f32), c: (f32, f32), y0: f32, y1: f32, col: Rgba) {
    let at = |p: (f32, f32), y: f32| {
        let q = site.at(p.0, p.1);
        Vec3::new(q.x, site.origin.y + y, q.z)
    };
    b.quad(at(a, y0), at(c, y0), at(c, y1), at(a, y1), col);
}

/// A pyramid or spire of `seg` sides standing on `base`, turned by `turn`
/// (a frustum's sides are fixed to the map; a tower's must follow the tower).
fn cone(b: &mut MeshBuf, base: Vec3, r: f32, h: f32, seg: usize, turn: f32, col: Rgba) {
    // the same turn as a block's: a positive yaw swings +x toward -z
    let ring = |i: usize| {
        let a = i as f32 / seg as f32 * std::f32::consts::TAU - turn;
        base + Vec3::new(a.cos() * r, 0.0, a.sin() * r)
    };
    let tip = base + Vec3::Y * h;
    for i in 0..seg {
        // lit from the south-west, as a frustum's sides are
        let k = 0.86 + 0.14 * (((i as f32 + 0.5) / seg as f32 * std::f32::consts::TAU - turn + 0.8).cos() * 0.5 + 0.5);
        b.tri(ring(i + 1), ring(i), tip, shade(col, k));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::houses::PLOT;
    use crate::parts::STOREY;

    fn build(big: bool, spire: bool, n: u32) -> MeshBuf {
        let mut b = MeshBuf::default();
        church(&mut b, &Site::level(Vec3::ZERO, 0.0), &Church { big, spire, seed: Seed::of(n) }, &Palette::standard());
        assert!(b.is_sound());
        b
    }

    #[test]
    fn the_tower_stands_over_the_town() {
        // a three-storey townhouse with its roof and chimney comes to about 0.1, a full-grown tree to 0.14
        let house = STOREY * 3.0 + PLOT.1 * 0.6;
        let plain = build(false, false, 1).bounds().unwrap().1.y;
        assert!(plain > house * 1.4 && plain > 0.14, "{plain}");
        // a spire is taller, and a big town's church taller again
        let spired = build(false, true, 1).bounds().unwrap().1.y;
        assert!(spired > plain + 0.07);
        assert!(build(true, true, 1).bounds().unwrap().1.y > spired * 1.2);
        assert!(build(true, false, 1).bounds().unwrap().1.y > plain * 1.2);
    }

    #[test]
    fn the_churchyard_holds_the_church() {
        for (big, spire) in [(false, false), (false, true), (true, false), (true, true)] {
            for n in 0..12 {
                let c = Church { big, spire, seed: Seed::of(n) };
                let (w, d) = c.yard();
                let (lo, hi) = build(big, spire, n).bounds().unwrap();
                // the wall is the outermost thing, half its thickness beyond the yard's edge
                let over = 0.0045 * 0.5 + 0.001;
                assert!(lo.x >= -w * 0.5 - over && hi.x <= w * 0.5 + over, "{big} {spire} seed {n}: x from {} to {}", lo.x, hi.x);
                assert!(lo.z >= -d * 0.5 - over && hi.z <= d * 0.5 + over, "{big} {spire} seed {n}: z from {} to {}", lo.z, hi.z);
            }
        }
        // a small town's churchyard is about two house plots wide and three long
        let (w, d) = Church { big: false, spire: false, seed: Seed::of(0) }.yard();
        assert!(w < PLOT.0 * 2.0 && d < PLOT.0 * 3.5, "{w} by {d}");
    }

    #[test]
    fn it_stands_off_the_market_place_with_its_front_to_it() {
        let c = Church { big: false, spire: false, seed: Seed::of(0) };
        let square = Vec2::new(3.0, 4.0);
        for bearing in [Vec2::X, Vec2::Y, Vec2::new(-0.6, 0.8), Vec2::new(0.6, -0.8)] {
            let (at, yaw) = c.beside_square(square, bearing);
            // the front of the site (+z) looks back at the market place
            let front = Site::level(Vec3::ZERO, yaw).at(0.0, 1.0);
            assert!((Vec2::new(front.x, front.z) + bearing).length() < 1e-5, "{bearing}: {front}");
            // the gate is where the yard meets the way out from the market place, and the yard runs on from it
            let (w, d) = c.yard();
            let gate = square + bearing * GATE_FROM_SQUARE;
            assert!(c.yard_holds(at, yaw, gate, 0.001) && !c.yard_holds(at, yaw, gate - bearing * 0.002, 0.001));
            assert!(c.yard_holds(at, yaw, gate + bearing * (d - 0.001), 0.0) && !c.yard_holds(at, yaw, gate + bearing * (d + 0.001), 0.0));
            let across = Vec2::new(-bearing.y, bearing.x);
            assert!(c.yard_holds(at, yaw, at + across * (w * 0.5 - 0.001), 0.0) && !c.yard_holds(at, yaw, at + across * (w * 0.5 + 0.001), 0.0));
            assert!(c.yard_holds(at, yaw, at + across * (w * 0.5 + 0.02), 0.03));
            // a house keeps out of the yard and out of the way to the gate, but may back onto the front wall beside it
            assert!(c.crowded_by(at, yaw, at) && c.crowded_by(at, yaw, at + across * (w * 0.5 + 0.02)));
            assert!(c.crowded_by(at, yaw, gate - bearing * 0.02));
            assert!(!c.crowded_by(at, yaw, gate - bearing * 0.02 + across * 0.04));
            assert!(!c.crowded_by(at, yaw, at + across * (w * 0.5 + 0.03)));
        }
    }

    #[test]
    fn the_tower_is_at_the_front_over_the_gate() {
        let b = build(false, true, 3);
        // the highest point is the tip of the spire, toward +z
        let tip = b.pos.iter().max_by(|p, q| p[1].total_cmp(&q[1])).unwrap();
        assert!(tip[2] > 0.03 && tip[0].abs() < 1e-5, "{tip:?}");
    }

    #[test]
    fn the_same_town_has_the_same_church() {
        assert_eq!(build(false, false, 5), build(false, false, 5));
        // another town's yews and gravestones stand elsewhere
        assert_ne!(build(false, false, 5), build(false, false, 6));
    }

    #[test]
    fn faces_look_outward() {
        // every wall, roof and window of the church itself turns away from its middle line
        let b = build(true, true, 2);
        let pal = Palette::standard();
        let glass = *pal.glass.window;
        for t in b.idx.chunks(3) {
            if b.col[t[0] as usize] != glass {
                continue;
            }
            let (p, n) = (Vec3::from_array(b.pos[t[0] as usize]), Vec3::from_array(b.nrm[t[0] as usize]));
            // a window on a side wall looks away from the church's axis, one on an end wall along it
            assert!(n.x * p.x > 1e-6 || n.x.abs() < 1e-4, "a window looks inward at {p}: {n}");
        }
    }
}
