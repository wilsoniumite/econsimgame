//! The iron mills: the forge that makes wrought rails, the rolling mill and
//! the steel rail mill.

use glam::Vec3;

use super::kit::{Ctx, Face};

/// A forge: a long low shed open at the sides, a louvred vent along its
/// ridge, and a rank of short stacks down each side, one to a puddling
/// furnace; the hammer house at its end (turned by a wheel where there is
/// water). More sheds side by side as it grows, and rails stacked by the gate.
pub fn forge(c: &mut Ctx) {
    let (hw, hd, g, pal) = (c.hw, c.hd, c.g, c.pal);
    c.ground(*pal.ground.yard_cinders, false);
    c.wall(c.yard_wall(), 0.008, 0.024 * g);
    let (wall, roof) = (c.masonry(true), c.roofing());
    let brick = c.tone(*pal.brick.chimney);
    let (dep, h, ham) = (0.036 * g, 0.024 * g, 0.03 * g);
    let room = 2.0 * hw - 0.016 - ham - 0.004;
    let (cols, len) = c.spread(room, 0.16 * g, 0.02 * g, 3);
    let pitch = dep + 0.016 * g;
    let rows = c.fit(2.0 * hd - 0.016 - 0.01 * g - 0.4 * hd, pitch, 4);
    let zrow = |j: usize| -hd + 0.008 + 0.009 * g + dep * 0.5 + j as f32 * pitch;
    for j in 0..rows {
        for i in 0..cols {
            let (x, z) = (-hw + 0.008 + len * 0.5 + i as f32 * (len + 0.02 * g), zrow(j));
            let s = c.shed(x, z, len, dep, h, false, wall, roof);
            c.ridge_vent(&s, 0.8, false);
            // the wide doors in its end, for the carts of pig iron and coal
            c.doors(&s, Face::EndB, -dep * 0.24, dep * 0.24, 0.0, h * 0.74, false, *pal.timber.door);
            let ns = ((len / (0.03 * g)).floor() as usize).max(1);
            let sh = h + s.rise + 0.02 * g;
            for k in 0..ns {
                let sx = x - len * 0.5 + len * (k as f32 + 0.5) / ns as f32;
                c.stack(sx, z - dep * 0.5 - 0.004 * g, sh, 0.008 * g, brick, if k % 2 == 0 { 0.4 } else { 0.0 });
                if j == rows - 1 {
                    c.stack(sx, z + dep * 0.5 + 0.004 * g, sh, 0.008 * g, brick, if k % 2 == 1 { 0.4 } else { 0.0 });
                }
            }
            // the open arcade, and the furnace doors glowing within
            if !c.going_up() {
                let arches = ns * 2;
                for a in 0..arches {
                    let cx = -len * 0.5 + len * (a as f32 + 0.5) / arches as f32;
                    let w = len / arches as f32 * 0.3;
                    if a % 3 == 1 {
                        c.glow(&s, Face::Front, cx - w, cx + w, 0.0, h * 0.5, true);
                    } else {
                        c.mouth(&s, Face::Front, cx - w, cx + w, 0.0, h * 0.5, true);
                    }
                }
            }
        }
    }
    // the hammer house
    let front_z = zrow(rows - 1);
    let hx = -hw + 0.008 + cols as f32 * (len + 0.02 * g) - 0.02 * g + 0.004 + ham * 0.5;
    let hl = (0.04 * g).min(dep * 1.2);
    c.shed(hx, front_z, hl, ham, 0.034 * g, true, wall, roof);
    if c.water_power() {
        let r = 0.014 * g;
        let wz = front_z + hl * 0.5 + 0.004 * g;
        c.race((hw - 0.003, wz), (hx - r, wz), 0.006 * g);
        c.wheel(hx, wz, r, 0.005 * g, false, c.tone(*pal.timber.waterwheel));
    } else {
        c.chimney(hx, (front_z - 0.03 * g).max(-hd + 0.008 + 0.011 * g), 0.1 * g, 0.006 * g, 0.8);
    }
    // rails in the stack, pig iron and coal
    let z0 = front_z + dep * 0.5 + 0.009 * g + 0.004;
    let d = hd - 0.012 - z0;
    if d > 0.008 {
        let w = hw * 1.1;
        c.stacks(-hw * 0.3, z0 + d * 0.5, w, d, c.fit(w, 0.05 * g, 8), c.fit(d, 0.006 * g, 10), 0.0035 * g, *pal.rail.rails);
        if c.stocked() {
            let r = (0.01 * g).min(d * 0.45);
            c.heap(hw * 0.55, z0 + d * 0.5, r, r * 0.8, *pal.goods.coal);
        }
    }
}

/// A rolling mill: a wide shed under a louvred ridge, the engine house
/// standing across its end with the flywheel beside it and one tall chimney
/// behind, and a few short stacks along the back for the reheating
/// furnaces. More sheds as it grows, and plate and bar stacked in the yard.
pub fn rolling_mill(c: &mut Ctx) {
    let (hw, hd, g, pal) = (c.hw, c.hd, c.g, c.pal);
    c.ground(*pal.ground.yard_cinders, false);
    c.wall(c.yard_wall(), 0.008, 0.024 * g);
    let (wall, roof) = (c.masonry(true), c.roofing());
    let brick = c.tone(*pal.brick.chimney);
    let plate = c.tone(*pal.iron.plate);
    let (dep, h, eh) = (0.046 * g, 0.03 * g, 0.032 * g);
    let room = 2.0 * hw - 0.016 - eh - 0.004;
    let (cols, len) = c.spread(room, 0.15 * g, 0.016 * g, 3);
    let pitch = dep + 0.014 * g;
    let rows = c.fit(2.0 * hd - 0.016 - 0.01 * g - 0.4 * hd, pitch, 3);
    let zrow = |j: usize| -hd + 0.008 + 0.009 * g + dep * 0.5 + j as f32 * pitch;
    for j in 0..rows {
        for i in 0..cols {
            let (x, z) = (-hw + 0.008 + len * 0.5 + i as f32 * (len + 0.016 * g), zrow(j));
            let s = c.shed(x, z, len, dep, h, false, wall, roof);
            c.ridge_vent(&s, 0.7, false);
            if c.late() {
                c.roof_lights(&s, ((len / (0.03 * g)) as usize).max(1));
            }
            let ns = ((len / (0.045 * g)).floor() as usize).max(1);
            for k in 0..ns {
                let sx = x - len * 0.5 + len * (k as f32 + 0.5) / ns as f32;
                c.stack(sx, z - dep * 0.5 - 0.004 * g, h + s.rise + 0.018 * g, 0.008 * g, brick, if (k + j) % 2 == 0 { 0.45 } else { 0.0 });
            }
            if !c.going_up() {
                c.mouth(&s, Face::Front, -len * 0.42, -len * 0.42 + 0.014 * g, 0.0, h * 0.7, true);
            }
        }
    }
    // the engine house across the end of the front shed, its flywheel, and the chimney
    let front_z = zrow(rows - 1);
    let right = -hw + 0.008 + cols as f32 * (len + 0.016 * g) - 0.016 * g;
    let ex = right + 0.004 + eh * 0.5;
    let el = (0.05 * g).min(dep * 1.1);
    let e = c.shed(ex, front_z, el, eh, 0.046 * g, true, c.masonry(false), roof);
    if !c.going_up() {
        for u in [-eh * 0.22, eh * 0.22] {
            c.light(&e, Face::EndB, u - 0.0035 * g, u + 0.0035 * g, 0.012 * g, 0.03 * g, true);
        }
    }
    let r = (0.02 * g).min(len * 0.3);
    c.wheel(right - r - 0.003 * g, front_z + dep * 0.5 + 0.0045 * g, r, 0.005 * g, false, plate);
    c.chimney(ex, (front_z - el * 0.5 - 0.014 * g).max(-hd + 0.008 + 0.014 * g), 0.15 * g, 0.0075 * g, 1.0);
    // plate and bar in the stack, and coal for the furnaces
    let z0 = front_z + dep * 0.5 + 0.012 * g;
    let d = hd - 0.012 - z0;
    if d > 0.008 {
        let w = (right - r * 2.0 - 0.01 * g + hw - 0.01).max(0.01);
        c.stacks(-hw + 0.01 + w * 0.5, z0 + d * 0.5, w, d, c.fit(w, 0.024 * g, 12), c.fit(d, 0.014 * g, 6), 0.003 * g, plate);
        if c.stocked() {
            let rr = (0.01 * g).min(d * 0.45).min((hw - 0.01 - right) * 0.45);
            if rr > 0.003 {
                c.heap(hw - 0.01 - rr, z0 + d * 0.5, rr, rr * 0.8, *pal.goods.coal);
            }
        }
    }
}

/// A steel rail mill: the longest of the mills, its bays in pairs under
/// twin ridges with a lantern along each; tall slim chimneys at one end and
/// the engine house at the other; rails by the hundred in the bank before
/// it, under a gantry.
pub fn steel_rail_mill(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_cinders, false);
    c.wall(c.yard_wall(), 0.008, 0.024 * g);
    let (wall, roof) = (c.masonry(true), c.roofing());
    let plate = c.tone(*pal.iron.plate);
    let (dep, h, lead, tail) = (0.03 * g, 0.03 * g, 0.022 * g, 0.03 * g);
    let room = 2.0 * hw - 0.016 - lead - tail;
    let (cols, len) = c.spread(room, 0.22 * g, 0.02 * g, 2);
    let pitch = 2.0 * dep + 0.014 * g;
    let rows = c.fit(2.0 * hd - 0.016 - 0.5 * hd + 0.014 * g, pitch, 3);
    let z0 = |j: usize| -hd + 0.008 + j as f32 * pitch;
    let glazed = c.late();
    for j in 0..rows {
        for i in 0..cols {
            let x = -hw + 0.008 + lead + len * 0.5 + i as f32 * (len + 0.02 * g);
            for bay in 0..2 {
                let s = c.shed(x, z0(j) + dep * (0.5 + bay as f32), len, dep, h, false, wall, roof);
                c.ridge_vent(&s, 0.86, glazed);
                if bay == 1 && !c.going_up() {
                    let doors = c.fit(len, 0.05 * g, 12);
                    for k in 0..doors {
                        let cx = -len * 0.5 + len * (k as f32 + 0.5) / doors as f32;
                        c.mouth(&s, Face::Front, cx - 0.006 * g, cx + 0.006 * g, 0.0, h * 0.66, true);
                    }
                }
            }
        }
        // the reheating furnaces' chimneys, at the head of the pair
        let stacks = if size >= 10.0 { 3 } else { 2 };
        for k in 0..stacks {
            c.chimney(-hw + 0.008 + lead * 0.5, z0(j) + 2.0 * dep * (k as f32 + 0.5) / stacks as f32, 0.13 * g, 0.006 * g, 0.8);
        }
    }
    // the engine house at the tail of the front pair
    let right = -hw + 0.008 + lead + cols as f32 * (len + 0.02 * g) - 0.02 * g;
    let front = z0(rows - 1);
    let e = c.shed(right + 0.004 + 0.013 * g, front + dep, (0.05 * g).min(2.0 * dep), 0.026 * g, 0.04 * g, true, c.masonry(false), roof);
    if !c.going_up() {
        c.light(&e, Face::EndB, -0.004 * g, 0.004 * g, 0.01 * g, 0.028 * g, true);
    }
    // the rail bank and its gantry
    let zb = front + 2.0 * dep + 0.006;
    let d = hd - 0.012 - zb;
    if d > 0.007 {
        let w = (right + hw - 0.008 - lead) * 0.92;
        let (cx, cz) = (-hw + 0.008 + lead + w * 0.5, zb + d * 0.5);
        c.stacks(cx, cz, w, d * 0.85, c.fit(w, 0.06 * g, 10), c.fit(d * 0.85, 0.006 * g, 12), 0.0035 * g, *pal.rail.rails);
        if size >= 1.0 && d > 0.014 && !c.going_up() && !c.derelict() {
            let top = Vec3::Y * 0.028 * g;
            let (gx, gz) = (w * 0.42, d * 0.42);
            let feet = [c.at(cx - gx, cz - gz), c.at(cx - gx, cz + gz), c.at(cx + gx, cz - gz), c.at(cx + gx, cz + gz)];
            for f in feet {
                c.solid.beam(f, f + top, 0.003 * g, plate);
            }
            c.solid.beam(feet[0] + top, feet[1] + top, 0.004 * g, plate);
            c.solid.beam(feet[2] + top, feet[3] + top, 0.004 * g, plate);
            c.solid.beam((feet[0] + feet[1]) * 0.5 + top, (feet[2] + feet[3]) * 0.5 + top, 0.005 * g, plate);
        }
    }
}
