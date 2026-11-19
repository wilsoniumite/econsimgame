//! Ironmaking: the charcoal blast furnace and the coke blast furnace.

use glam::Vec3;

use super::kit::{Ctx, Face, Fill};
use super::State;

/// How tall a furnace stands in the works' present state.
fn stands(c: &Ctx, h: f32) -> f32 {
    match c.w.state {
        State::GoingUp => h * 0.5,
        State::Derelict => h * 0.9,
        _ => h,
    }
}

/// An earth bank with a stone face toward the yard: `h0` high at the back
/// (z0) and `h1` at its face (z1).
fn bank(c: &mut Ctx, x0: f32, x1: f32, z0: f32, z1: f32, h0: f32, h1: f32) {
    let grass = c.tone(*c.pal.ground.furnace_bank);
    let stone = c.tone(*c.pal.stone.yard_wall);
    let site = c.site;
    let p = move |x: f32, z: f32, h: f32| site.at(x, z) + Vec3::Y * h;
    // earth lies back at its natural slope on three sides; the face to the yard is walled
    let (b0, b1) = (((h0 + 0.01) * 0.5).min((z1 - z0) * 0.4), ((h1 + 0.01) * 0.5).min((x1 - x0) * 0.2));
    let t = [p(x0 + b0, z0 + b0, h0), p(x1 - b0, z0 + b0, h0), p(x1 - b1, z1, h1), p(x0 + b1, z1, h1)];
    let f = [p(x0, z0, -0.01), p(x1, z0, -0.01), p(x1, z1, -0.01), p(x0, z1, -0.01)];
    c.solid.quad(t[3], t[2], t[1], t[0], grass);
    c.solid.quad(f[3], f[2], t[2], t[3], stone);
    c.solid.quad(f[2], f[1], t[1], t[2], crate::colour::shade(grass, 0.9));
    c.solid.quad(f[1], f[0], t[0], t[1], crate::colour::shade(grass, 0.8));
    c.solid.quad(f[0], f[3], t[3], t[0], crate::colour::shade(grass, 0.92));
    if c.far() {
        return;
    }
    // buttresses against the weight of the bank, two bays apart
    let n = (((x1 - x0) / (2.0 * super::kit::BAY)).round() as usize).max(1);
    for i in 0..=n {
        let x = x0 + (x1 - x0) * i as f32 / n as f32;
        let (w, out) = (0.0034, 0.0028);
        let (a, b, top) = (p(x - w * 0.5, z1 + out, -0.01), p(x + w * 0.5, z1 + out, -0.01), h1 * 0.9);
        let (ta, tb) = (p(x - w * 0.5, z1, top), p(x + w * 0.5, z1, top));
        let pier = crate::colour::shade(stone, 0.94);
        c.solid.quad(a, b, tb, ta, pier);
        c.solid.quad(p(x - w * 0.5, z1, -0.01), a, ta, ta, crate::colour::shade(pier, 0.9));
        c.solid.quad(b, p(x + w * 0.5, z1, -0.01), tb, tb, crate::colour::shade(pier, 0.84));
    }
}

/// A charcoal blast furnace: a square tapered stone stack against a bank,
/// charged over a bridge from the bank top; a casting house before it, the
/// bellows house at its side (with its wheel where there is water), the
/// charcoal barn, ore and limestone in heaps and pig iron by the gate. A
/// big one is two or three stacks side by side.
pub fn charcoal_furnace(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_earth, false);
    c.wall(c.yard_wall(), 0.007, 0.022 * g);
    let stone = c.tone(*pal.stone.works);
    let roof = c.roofing();
    let (side, fh, cast) = (0.034 * g, 0.075 * g, 0.03 * g);
    let (bw, bd) = (0.024 * g, 0.022 * g);
    let pitch = side + 0.02 * g;
    let n = (if size < 8.0 { 1 } else if size < 40.0 { 2 } else { 3 }).min(c.fit(2.0 * hw - 0.02 - bw, pitch, 3));
    // as near the back of the yard as leaves a bank behind it; in a small yard, as near as the casting house allows
    let zf = (hd - 0.012 - cast + 0.005 * g - side * 0.5).min(-hd + 0.006 + 0.06 * g + side * 0.5);
    let x0 = -hw + 0.008 + bw + 0.003;
    let row = n as f32 * pitch - 0.02 * g;

    // the bank the furnace is charged from
    let (b0, b1, bank_h) = (-hd + 0.006, zf - side * 0.5 - 0.005 * g, fh * 0.56);
    let banked = b1 - b0 > 0.004;
    if banked {
        bank(c, x0 - 0.003 * g, x0 + row + 0.003 * g, b0, b1, bank_h * 0.4, bank_h);
    }
    for i in 0..n {
        let x = x0 + side * 0.5 + i as f32 * pitch;
        let h = stands(c, fh);
        let top = c.furnace_stack(x, zf, side, side * std::f32::consts::FRAC_1_SQRT_2, h, stone);
        if !c.going_up() {
            // the tunnel head, where the flame stands
            c.solid.frustum_turned(top, 0.0085 * g, 0.0065 * g, 0.012 * g, 6, 0.0, 0.0, c.tone(*pal.brick.dark), Some(c.throat()));
            c.plume(top + Vec3::Y * 0.012 * g, 0.8);
            if banked && !c.derelict() {
                // the charging bridge
                c.barrow_way((x, b1, bank_h - 0.003 * g), (x, zf - side * 0.3, h * 0.86), 0.009 * g, *pal.timber.headframe);
            }
        }
        // the casting house, its arch open on the fire
        let s = c.shed(x, zf + side * 0.5 - 0.005 * g + cast * 0.5, cast, 0.03 * g, 0.024 * g, true, stone, roof);
        if !c.going_up() {
            c.glow(&s, Face::EndB, -0.008 * g, 0.008 * g, 0.0, 0.014 * g, true);
        }
    }
    // the bellows house
    let bx = -hw + 0.008 + bw * 0.5;
    let b = c.shed(bx, zf, bw, bd, 0.022 * g, false, stone, roof);
    if c.water_power() {
        let r = 0.013 * g;
        let wz = zf + bd * 0.5 + 0.004 * g;
        c.race((-hw + 0.003, wz), (bx + r, wz), 0.006 * g);
        c.wheel(bx, wz, r, 0.005 * g, false, c.tone(*pal.timber.waterwheel));
    } else if c.w.founded >= 1775 {
        // a blowing engine in place of the wheel
        c.stack(bx, zf - bd * 0.5 - 0.005 * g, 0.022 * g + b.rise + 0.03 * g, 0.008 * g, c.tone(*pal.brick.chimney), 0.4);
    }
    // the charcoal barns down the side, and the stock before them
    let xr = x0 + row + 0.006;
    let room = hw - 0.008 - xr;
    if room >= 0.024 * g {
        let dep = (0.03 * g).min(room);
        // one barn to a small furnace; a great one has long barns in ranks, never more than
        // three ranks of two, with a cart's width between them
        let lane = if size < 3.0 { 0.008 } else { 0.014 * g };
        let deep = 2.0 * hd - 0.02 - 0.03 * g;
        let (cols, barns, len) = if size < 3.0 {
            (c.fit(room + lane, dep + lane, 4), 1, (0.045 * g).min(zf + side * 0.5 + hd - 0.008))
        } else {
            let barns = c.fit(deep + lane, 0.045 * g + lane, 2);
            (c.fit(room + lane, dep + lane, 3), barns, ((deep - lane * (barns as f32 - 1.0)) / barns as f32).min(0.07 * g))
        };
        for i in 0..cols {
            for j in 0..barns {
                let barn = c.shed(xr + dep * 0.5 + i as f32 * (dep + lane), -hd + 0.008 + len * 0.5 + j as f32 * (len + lane), len, dep, 0.024 * g, true, c.tone(*pal.timber.boards), roof);
                c.fill(&barn, Fill::Posts);
                if !c.going_up() {
                    c.doors(&barn, Face::EndB, -dep * 0.25, dep * 0.25, 0.0, 0.017 * g, false, *pal.timber.door);
                }
            }
        }
        let front = -hd + 0.008 + barns as f32 * (len + lane);
        let depth = hd - 0.012 - front;
        if c.stocked() && depth > 0.012 {
            let r = (room * 0.24).min(depth * 0.3).min(0.03 * g);
            c.heap(xr + room * 0.27, front + depth * 0.3, r, r * 0.8, *pal.ground.spoil_ironstone);
            c.heap(xr + room * 0.75, front + depth * 0.32, r * 0.9, r * 0.7, *pal.stone.limestone);
            c.stacks(xr + room * 0.5, front + depth * 0.78, room * 0.8, depth * 0.3, c.fit(room * 0.8, 0.014 * g, 12), c.fit(depth * 0.3, 0.012 * g, 4), 0.004 * g, *pal.iron.plate);
        }
    } else if c.stocked() {
        c.heap(hw - 0.008 - room * 0.5, -hd * 0.3, room * 0.4, room * 0.32, *pal.goods.coal);
    }
    // in a deep yard, pig iron in rows before the casting houses, and charcoal to hand
    let z0 = zf + side * 0.5 - 0.005 * g + cast + 0.008;
    let d = hd - 0.012 - z0;
    let w = xr - 0.006 + hw - 0.01;
    if d > 0.014 && w > 0.03 {
        c.stacks(-hw + 0.01 + w * 0.6, z0 + d * 0.5, w * 0.7, d * 0.8, c.fit(w * 0.7, 0.016 * g, 16), c.fit(d * 0.8, 0.012 * g, 8), 0.004 * g, *pal.iron.plate);
        if c.stocked() {
            let r = (w * 0.12).min(d * 0.4).min(0.03 * g);
            c.heap(-hw + 0.012 + r, z0 + d * 0.5, r, r * 0.75, *pal.goods.coal);
        }
    }
}

/// A coke blast furnace: tall furnaces in a row (square and of stone when
/// early, round and cased in iron later), a lift and a gantry to charge
/// them, hot-blast stoves behind, a casting house before each, the blowing
/// engine's house and its tall chimney, a bank of coke ovens, and the slag
/// heap, which grows.
pub fn coke_furnace(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_cinders, false);
    c.wall(c.yard_wall(), 0.008, 0.024 * g);
    let roof = c.roofing();
    let square = c.early();
    let (r, pitch) = (0.02 * g, 0.047 * g);
    let fh = if square { 0.095 * g } else { 0.105 * g };
    let (lift, eng) = (0.014 * g, if size >= 0.7 { 0.024 * g } else { 0.0 });
    let stoves = size >= 1.0 && c.w.founded >= 1830 && !square;
    let closed = c.w.founded >= 1850 && !square;
    let fixed = lift + 0.004 * g + eng + 0.006 * g + 0.03 * g;
    let wanted = (0.99 * size.powf(0.4)).ceil().max(1.0) as usize;
    let n = c.fit(2.0 * hw - 0.016 - fixed, pitch, 6).min(wanted);
    let spare = (2.0 * hw - 0.016 - fixed - n as f32 * pitch).max(0.0);
    let xl = -hw + 0.008 + spare * 0.45;
    let zf = -hd + 0.008 + r + if stoves { 0.022 * g } else { 0.0 };
    let cast = (0.034 * g).min(hd - 0.012 - (zf + r - 0.004 * g));
    let shell = c.tone(if square { *pal.stone.works } else if c.late() { *pal.iron.plate } else { *pal.brick.dark });
    let plate = c.tone(*pal.iron.plate);
    let fx = |i: usize| xl + lift + 0.004 * g + pitch * (i as f32 + 0.5);
    let h = stands(c, fh);

    for i in 0..n {
        let x = fx(i);
        let p = c.at(x, zf);
        if square {
            c.furnace_stack(x, zf, 0.0255 * g * std::f32::consts::SQRT_2, 0.0185 * g * std::f32::consts::SQRT_2, h, shell);
        } else {
            c.furnace_shell(x, zf, r, h, shell);
            c.charging_gallery(x, zf, r * 0.98, h * 0.9);
        }
        if !c.going_up() && !c.derelict() {
            if closed {
                // a closed top: the bell, and the downcomer that takes the gas to the stoves
                c.solid.frustum_turned(p + Vec3::Y * h, r * 0.5, 0.0, 0.01 * g, 6, 0.0, 0.0, plate, None);
                let foot = c.at(x + r * 0.7, zf - r - 0.004 * g);
                c.solid.beam(p + Vec3::Y * (h * 0.97) + (foot - p).normalize_or_zero() * r * 0.5, foot, 0.005 * g, plate);
            } else {
                c.solid.frustum_turned(p + Vec3::Y * h, 0.009 * g, 0.007 * g, 0.01 * g, 6, 0.0, 0.0, c.tone(*pal.brick.dark), Some(c.throat()));
                c.plume(p + Vec3::Y * (h + 0.01 * g), 1.0);
            }
        }
        if stoves {
            let zs = zf - r - 0.011 * g;
            if c.w.founded >= 1860 {
                for dx in [-0.0095 * g, 0.0095 * g] {
                    let top = c.flue(x + dx, zs, 0.068 * g, 0.008 * g, 0.008 * g, plate, plate, 0.0);
                    c.hoops(x + dx, zs, top.y - c.at(x + dx, zs).y, 0.008 * g, 0.008 * g, &[0.2, 0.45, 0.7], crate::colour::shade(plate, 1.45));
                    if !c.going_up() && !c.derelict() {
                        c.solid.frustum_turned(top, 0.008 * g, 0.0, 0.007 * g, 6, 0.0, 0.0, plate, None);
                    }
                }
                c.plume(c.at(x, zs) + Vec3::Y * 0.075 * g, 0.6);
            } else {
                let top = c.flue(x, zs, 0.045 * g, 0.0095 * g, 0.0085 * g, c.tone(*pal.brick.dark), *pal.smoke.soot, if closed { 0.6 } else { 0.0 });
                c.hoops(x, zs, top.y - c.at(x, zs).y, 0.0095 * g, 0.0085 * g, &[0.3, 0.6], crate::colour::shade(plate, 0.9));
            }
        }
        // the casting house
        if cast > 0.012 {
            let s = c.shed(x, zf + r - 0.004 * g + cast * 0.5, cast, 0.03 * g, 0.022 * g, true, c.masonry(true), roof);
            if !c.going_up() {
                c.glow(&s, Face::EndB, -0.008 * g, 0.008 * g, 0.0, 0.013 * g, true);
            }
        }
    }
    // the way up: an incline when early, a lift after, and the gantry along the furnace tops
    if !c.going_up() && !c.derelict() {
        let lx = xl + lift * 0.5;
        if square {
            c.barrow_way((xl, zf + r * 0.4, 0.0), (fx(0) - r * 0.5, zf + r * 0.4, h * 0.96), 0.008 * g, c.tone(*pal.timber.headframe));
        } else {
            c.lift(lx, zf, lift, h + 0.008 * g);
            c.barrow_way((lx, zf, h + 0.003 * g), (fx(n - 1), zf, h + 0.003 * g), 0.006 * g, plate);
        }
    }
    // the blowing engine's house and its chimney
    let xr = fx(n - 1) + r + 0.006 * g;
    if eng > 0.0 {
        let s = c.tower(xr + eng * 0.5, zf, eng, 0.05 * g, c.masonry(false), roof);
        if !c.going_up() {
            c.light(&s, Face::Front, -eng * 0.16, eng * 0.16, 0.012 * g, 0.034 * g, true);
        }
    }
    let tall = fh * 1.3 * size.max(1.0).powf(0.04);
    c.chimney(xr + eng + 0.016 * g, (zf - 0.008 * g).max(-hd + 0.008 + 0.016 * g), tall, 0.008 * g, 1.2);
    // what the rest of the yard holds
    let front = zf + r - 0.004 * g + cast;
    let deep = hd - 0.012 - front;
    let slag_col = c.tone(*pal.ground.slag);
    if deep >= 0.04 * g {
        // a great works. Before the casting houses, the pig beds; then across the yard the slag
        // tip on one side and, on the other, the stock in heaps and the batteries of coke ovens
        let (x0, x1) = (-hw + 0.01, hw - 0.01);
        let pig = 0.02 * g;
        c.stacks((fx(0) + fx(n - 1)) * 0.5, front + 0.006 + pig * 0.5, n as f32 * pitch, pig, n * 3, 2, 0.004 * g, plate);
        let (z0, z1) = (front + 0.012 + pig, hd - 0.012);
        let d = z1 - z0;
        let tip = (x1 - x0) * 0.3;
        if !c.going_up() {
            let rr = (tip * 0.5).min(d * 0.5);
            c.heap(x0 + rr, z1 - rr, rr, rr * 0.62, slag_col);
            if d > rr * 3.0 {
                c.heap(x0 + rr * 0.75, z1 - rr * 2.5, rr * 0.7, rr * 0.4, slag_col);
            }
        }
        let (ox0, olen) = (x0 + tip + 0.012, x1 - x0 - tip - 0.012);
        let stock = (0.034 * g).min(d * 0.3);
        if c.stocked() {
            let heaps = c.fit(olen, stock * 1.3, 12);
            let cols = [*pal.goods.coal, *pal.ground.spoil_ironstone, *pal.stone.limestone];
            for i in 0..heaps {
                c.heap(ox0 + olen * (i as f32 + 0.5) / heaps as f32, z0 + stock * 0.5, stock * 0.5, stock * 0.4, cols[i * 3 / heaps]);
            }
        }
        let room = d - stock - 0.006;
        let batteries = if room >= 0.018 * g { c.fit(room, 0.034 * g, 4) } else { 0 };
        for j in 0..batteries {
            // (in a planned works, whose stock lies in yards of its own, the ovens stand up by the pig beds)
            let z = if c.stock_apart { z0 + 0.012 * g + j as f32 * 0.034 * g } else { z1 - 0.008 * g - j as f32 * 0.034 * g };
            coke_ovens(c, ox0, olen, z);
        }
    } else {
        // a small works: slag in the spare corner, the ovens along the front beside the casting houses
        let slag = (spare * 0.45 * 0.5).min(hd * 0.62);
        if slag > 0.004 && !c.going_up() {
            c.heap(-hw + 0.008 + slag, hd - 0.012 - slag, slag, slag * 0.75, slag_col);
        }
        let ox0 = fx(n - 1) + 0.015 * g + 0.006 * g;
        let olen = hw - 0.01 - ox0;
        let oz = hd - 0.012 - 0.008 * g;
        if olen >= 0.02 {
            coke_ovens(c, ox0, olen, oz);
            // pig iron between the engine house and the ovens
            let (z0, z1) = (zf + eng.max(r) * 0.5 + 0.01 * g, oz - 0.008 * g - 0.006);
            let (px0, px1) = (xr.max(ox0), hw - 0.012);
            if z1 - z0 > 0.01 && px1 - px0 > 0.015 {
                let (w, d) = (px1 - px0, z1 - z0);
                c.stacks((px0 + px1) * 0.5, z0 + d * 0.72, w * 0.9, d * 0.5, c.fit(w * 0.9, 0.02 * g, 12), c.fit(d * 0.5, 0.01 * g, 6), 0.004 * g, plate);
            }
        }
    }
}

/// A battery of coke ovens: a long low bank, a fire at every door and a
/// little smoke along its top. It runs `len` from `x0` along the front.
fn coke_ovens(c: &mut Ctx, x0: f32, len: f32, z: f32) {
    let (g, pal) = (c.g, c.pal);
    let s = c.bank(x0 + len * 0.5, z, len, 0.016 * g, 0.011 * g, false, c.tone(*pal.brick.dark), c.tone(*pal.roofs.coke_ovens));
    if c.going_up() {
        return;
    }
    let doors = c.fit(len, 0.011 * g, 60);
    for i in 0..doors {
        let cx = -len * 0.5 + len * (i as f32 + 0.5) / doors as f32;
        c.glow(&s, Face::Front, cx - 0.0028 * g, cx + 0.0028 * g, 0.001, 0.007 * g, false);
    }
    let wisps = (len / 0.09).ceil();
    for i in 0..wisps as usize {
        c.plume(c.at(x0 + len * (i as f32 + 0.5) / wisps, z) + Vec3::Y * 0.011 * g, 0.4);
    }
}
