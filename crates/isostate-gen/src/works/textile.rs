//! The powered mill.

use super::kit::{Ctx, Face};

/// A powered mill: a block of many storeys with windows on every side and a
/// stair tower on its front. Early and by water it has a wheel on its wall
/// and a bell on its ridge; otherwise an engine house and a tall round
/// chimney. It gains storeys and length as it grows, then weaving sheds
/// under north lights, then a second and a third mill round the yard.
pub fn mill(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_setts, false);
    c.wall(c.yard_wall(), 0.008, 0.024 * g);
    let (wall, roof) = (c.masonry(false), c.roofing());
    let lit = if c.busy() { *pal.glass.mill_lit } else { *pal.glass.window };
    let storeys = (2.6 + 1.6 * size.max(0.3).log10()).round().clamp(2.0, 6.0) as usize;
    let h = storeys as f32 * 0.02 * g.sqrt();
    let dep = 0.03 * g;
    let wheel = c.water_power();
    let tail = if wheel { 0.004 } else { 0.034 * g };
    let room = 2.0 * hw - 0.016 - tail;
    let len = room.min(0.13 * g);
    let (x, z) = (-hw + 0.008 + len * 0.5, -hd + 0.008 + dep * 0.5);
    let m = c.shed_pitched(x, z, len, dep, h, 0.22, false, wall, roof);
    c.windows(&m, storeys, 0, storeys, 0.011, size >= 1.0, lit);
    let ts = 0.016 * g;
    c.tower(x - len * 0.2, z + dep * 0.5 + ts * 0.5 - 0.001, ts, h + 0.014 * g, wall, roof);
    let front = z + dep * 0.5;

    if wheel {
        let r = (0.02 * g).min(h * 0.55);
        let (wx, wz) = (x + len * 0.5 - r - 0.003, front + 0.004 * g);
        c.race((hw - 0.003, wz), (wx - r, wz), 0.007 * g);
        c.wheel(wx, wz, r, 0.006 * g, false, c.tone(*pal.timber.waterwheel));
        // the bell that calls the hands
        c.tower(x + len * 0.25, z, 0.008 * g, h + m.rise + 0.004 * g, wall, roof);
    } else {
        // the chimney at the back corner, and the engine house before it
        let (r, ew, ed) = (0.0075 * g, 0.026 * g, 0.024 * g);
        let (ex, zc) = (x + len * 0.5 + 0.003 + ew * 0.5, -hd + 0.008 + r * 1.75);
        let tall = (0.11 + 0.014 * storeys as f32) * g.powf(0.6);
        c.chimney(ex, zc, tall, r, 1.0 + 0.1 * storeys as f32);
        let e = c.shed(ex, zc + r * 1.75 + 0.002 + ed * 0.5, ew, ed, (0.04 * g).min(h), false, wall, roof);
        if !c.going_up() {
            c.panel(&e, Face::Front, -ew * 0.16, ew * 0.16, 0.008 * g, 0.028 * g.min(1.3), true, lit);
        }
        if c.stocked() && size >= 3.0 {
            c.heap(ex, zc + r * 1.75 + ed + 0.014 * g, 0.008 * g, 0.006 * g, *pal.goods.coal);
        }
    }

    // more mills down the sides of the yard
    let side_len = (len * 0.8).min(hd - 0.014 - front - 0.006);
    let second = size >= 20.0 && side_len > dep * 1.5;
    let third = size >= 70.0 && second;
    for (on, sx) in [(second, -hw + 0.008 + dep * 0.5), (third, x + len * 0.5 - dep * 0.5)] {
        if on {
            let s = c.shed_pitched(sx, front + 0.006 + side_len * 0.5, side_len, dep, h, 0.22, true, wall, roof);
            c.windows(&s, storeys, 0, storeys, 0.011, true, lit);
        }
    }
    // the weaving shed, under its north lights
    let (x0, x1) = (-hw + 0.008 + if second { dep + 0.008 } else { 0.0 }, if third { x + len * 0.5 - dep - 0.008 } else { hw - 0.012 });
    let (z0, z1) = (front + ts + 0.008, hd - 0.018);
    let mut bales_z = (front + ts + hd - 0.012) * 0.5;
    if size >= 6.0 && c.w.founded >= 1820 && x1 - x0 > 0.04 && z1 - z0 > 0.03 {
        let (wl, wd) = ((x1 - x0) * 0.9, (z1 - z0) * 0.72);
        let teeth = ((wl / (0.014 * g)).floor() as usize).max(2);
        c.sawtooth(x0 + wl * 0.5, z0 + wd * 0.5, wl, wd, 0.016 * g, teeth, false, wall, roof);
        bales_z = (z0 + wd + hd - 0.012) * 0.5;
    }
    // bales by the gate
    let d = (hd - 0.012 - bales_z) * 2.0;
    if d > 0.006 {
        let w = (x1 - x0) * 0.5;
        c.stacks(x0 + w * 0.5 + (x1 - x0) * 0.45, bales_z, w, d.min(0.03 * g), c.fit(w, 0.012 * g, 10), c.fit(d.min(0.03 * g), 0.011 * g, 3), 0.007 * g, *pal.goods.bales);
    }
}
