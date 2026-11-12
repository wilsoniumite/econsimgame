//! Steel: the crucible shop and the converter.

use super::kit::{Ctx, Face, Shed};
use crate::colour::Rgba;

/// Crucible steel: a melting shop with a comb of short square stacks along
/// its wall, one to each range of melting holes. More shops as it grows,
/// first along the back and then down the sides of a court, their stacks
/// on the outer walls; an office range with a carriage arch across the
/// front; a forge and a tall chimney in the court.
pub fn crucible(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_cinders, false);
    c.wall(c.yard_wall(), 0.008, 0.02 * g);
    let (wall, roof) = (c.masonry(true), c.roofing());
    let (ul, dep) = ((0.045 * g).min(2.0 * hw - 0.016), 0.03 * g);
    let wanted = size.sqrt().ceil().max(1.0) as usize;
    let n_back = c.fit(2.0 * hw - 0.016, ul, 8).min(wanted);
    let zb = -hd + 0.008 + 0.004 * g + dep * 0.5;
    melting_shops(c, 0.0, zb, n_back, ul, dep, false, Face::Back, wall, roof);
    let behind = zb + dep * 0.5;
    let range = size >= 6.0;
    let fr = 0.024 * g;
    let court_front = hd - 0.008 - if range { fr + 0.006 } else { 0.004 };
    // down the sides
    let per_side = ((court_front - behind - 0.004) / ul).floor().max(0.0) as usize;
    let mut left = wanted - n_back;
    for (side, face) in [(-1.0f32, Face::Back), (1.0, Face::Front)] {
        let n = left.min(per_side).min((wanted - n_back).div_ceil(2));
        if n == 0 {
            continue;
        }
        left -= n;
        melting_shops(c, side * (hw - 0.008 - 0.004 * g - dep * 0.5), behind + 0.004 + n as f32 * ul * 0.5, n, ul, dep, true, face, wall, roof);
    }
    let winged = wanted > n_back && per_side > 0;
    // the range across the front, with the way in under its arch
    if range {
        let len = 2.0 * hw - 0.02;
        let s = c.shed(0.0, hd - 0.008 - fr * 0.5, len, fr, c.storeys(2), false, c.masonry(false), roof);
        c.windows(&s, 0, 2, false);
        if !c.going_up() {
            for face in [Face::Front, Face::Back] {
                c.mouth(&s, face, -0.007 * g, 0.007 * g, 0.0, 0.02 * g, true);
            }
        }
    }
    // the court: a forge to draw the steel down, the engine's chimney, bar in the stack
    let cw = 2.0 * hw - 0.02 - if winged { 2.0 * (dep + 0.008 * g) } else { 0.0 };
    let cd = court_front - behind - 0.012;
    if cd > 0.012 && cw > 0.02 {
        let mut z0 = behind + 0.008;
        if size >= 15.0 {
            // forges to draw the steel down: one, then rows of them
            let (len, d2) = ((0.09 * g).min(cw * 0.6), (0.034 * g).min(cd * 0.4));
            let cols = c.fit(cw * 0.7 + 0.012, len + 0.012, 2);
            let rows = c.fit(cd * 0.6, d2 + 0.012, 3);
            for j in 0..rows {
                for i in 0..cols {
                    let s = c.shed(-cw * 0.36 + len * 0.5 + i as f32 * (len + 0.012), z0 + 0.006 + d2 * 0.5 + j as f32 * (d2 + 0.012), len, d2, 0.026 * g, false, wall, roof);
                    c.ridge_vent(&s, 0.7, false);
                }
            }
            z0 += 0.012 + rows as f32 * (d2 + 0.012);
        }
        if size >= 10.0 {
            c.chimney(cw * 0.5 - 0.016 * g, behind + 0.02 * g, 0.13 * g, 0.0065 * g, 1.0);
        }
        let d = court_front - 0.006 - z0;
        if d > 0.008 {
            c.stacks(-cw * 0.1, z0 + d * 0.5, cw * 0.6, d * 0.8, c.fit(cw * 0.6, 0.02 * g, 10), c.fit(d * 0.8, 0.008 * g, 6), 0.0035 * g, *pal.iron.plate);
            if c.stocked() {
                let r = (0.009 * g).min(d * 0.4).min(cw * 0.12);
                c.heap(cw * 0.36, z0 + d * 0.55, r, r * 0.8, *pal.goods.coal);
            }
        }
    }
}

/// A row of `n` melting shops as one building, with the stacks of its
/// melting holes in a comb along the wall that `outer` names.
#[allow(clippy::too_many_arguments)]
fn melting_shops(c: &mut Ctx, x: f32, z: f32, n: usize, ul: f32, dep: f32, across: bool, outer: Face, wall: Rgba, roof: Rgba) -> Shed {
    let (g, pal) = (c.g, c.pal);
    let h = c.storeys(2);
    let s = c.shed(x, z, n as f32 * ul, dep, h, across, wall, roof);
    c.windows(&s, 0, 2, false);
    let out = if outer == Face::Front { 1.0 } else { -1.0 };
    let brick = c.tone(*pal.brick.chimney);
    for i in 0..n {
        let cx = -(n as f32) * ul * 0.5 + ul * (i as f32 + 0.5);
        c.slab_stack(&s, cx, out * dep * 0.5, (0.03 * g).min(ul * 0.7), 0.008 * g, h + s.rise + 0.024 * g, brick, 0.6);
    }
    s
}

/// A converter shop: a tall house with a great hood through its roof over
/// each vessel and a bay open on the fire; the cupolas behind, the blowing
/// engine's house and its chimney at the end. A longer house with more
/// vessels as it grows, then a second house.
pub fn converter(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_cinders, false);
    c.wall(c.yard_wall(), 0.008, 0.024 * g);
    let (wall, roof) = (c.masonry(true), c.roofing());
    let plate = c.tone(*pal.iron.plate);
    let (dep, h, eng, cup) = (0.045 * g, 0.055 * g, 0.026 * g, 0.016 * g);
    let room = 2.0 * hw - 0.016 - eng - 0.004;
    let ul = (0.07 * g).min(room);
    let wanted = (size.sqrt() / 1.5).ceil().max(1.0) as usize;
    let per = c.fit(room, ul, 5);
    let pitch = dep + cup + 0.03 * g;
    let rows = c.fit(2.0 * hd - 0.016 - 0.4 * hd, pitch, 3).min(wanted.div_ceil(per));
    let mut left = wanted.min(rows * per);
    let zrow = |j: usize| -hd + 0.008 + cup + dep * 0.5 + j as f32 * pitch;
    for j in 0..rows {
        let n = left.min(per);
        left -= n;
        let (len, z) = (n as f32 * ul, zrow(j));
        let x = -hw + 0.008 + len * 0.5;
        let s = c.shed(x, z, len, dep, h, false, wall, roof);
        // the cupolas that melt the iron, under a lean-to at the back
        c.lean_to(x, z - dep * 0.5 - cup * 0.5, len * 0.7, cup, 0.034 * g, 0.02 * g, false, wall, roof);
        for i in 0..n {
            let cx = x - len * 0.5 + ul * (i as f32 + 0.5);
            for dx in [-ul * 0.22, ul * 0.22] {
                c.flue(cx + dx, z, h + s.rise + 0.03 * g, 0.0115 * g, 0.008 * g, c.tone(*pal.iron.converter), c.throat(), 0.9);
            }
            c.flue(cx, z - dep * 0.5 - cup * 0.5, 0.075 * g, 0.006 * g, 0.005 * g, plate, *pal.smoke.soot, 0.5);
            if !c.going_up() {
                let u = cx - x;
                c.mouth(&s, Face::Front, u - ul * 0.32, u - ul * 0.1, 0.0, h * 0.62, false);
                c.glow(&s, Face::Front, u - ul * 0.1, u + ul * 0.1, 0.0, h * 0.62, false);
                c.mouth(&s, Face::Front, u + ul * 0.1, u + ul * 0.32, 0.0, h * 0.62, false);
            }
        }
    }
    // the blowing engine and its chimney
    let front_z = zrow(rows - 1);
    let ex = -hw + 0.008 + (wanted.min(rows * per).min(per)) as f32 * ul + 0.004 + eng * 0.5;
    let e = c.tower(ex, front_z, eng, 0.06 * g, c.masonry(false), roof);
    if !c.going_up() {
        c.light(&e, Face::Front, -eng * 0.16, eng * 0.16, 0.014 * g, 0.042 * g, true);
    }
    c.chimney(ex, (front_z - eng * 0.5 - 0.016 * g).max(-hd + 0.008 + 0.014 * g), 0.17 * g, 0.008 * g, 1.2);
    // ingots cooling in the yard, and the crane that lifts them
    let z0 = front_z + dep * 0.5 + 0.008;
    let d = hd - 0.012 - z0;
    if d > 0.008 {
        let w = hw * 1.2;
        c.stacks(-hw * 0.25, z0 + d * 0.5, w, d * 0.8, c.fit(w, 0.012 * g, 16), c.fit(d * 0.8, 0.012 * g, 6), 0.008 * g, plate);
        if size >= 3.0 && d > 0.03 {
            c.sheer_legs(hw * 0.55, z0 + d * 0.5, 0.05 * g);
        }
        if c.stocked() {
            let r = (0.01 * g).min(d * 0.4);
            c.heap(hw - 0.012 - r, z0 + d * 0.5, r, r * 0.8, *pal.goods.coal);
        }
    }
}
