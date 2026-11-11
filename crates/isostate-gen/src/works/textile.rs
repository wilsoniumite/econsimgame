//! The powered mill.

use super::kit::{Ctx, Face};

/// A powered mill: a block of many storeys with windows on every side and a
/// stair tower on its front. Early and by water it has a wheel on its wall
/// and a bell on its ridge; otherwise an engine house and a tall round
/// chimney. It gains storeys and length as it grows, then a weaving shed
/// under north lights across the front of the yard, then more mills, side
/// by side and row behind row.
pub fn mill(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_setts, false);
    c.wall(c.yard_wall(), 0.008, 0.024 * g);
    let (wall, roof) = (c.masonry(false), c.roofing());
    let lit = if c.busy() { *pal.glass.mill_lit } else { *pal.glass.window };
    let storeys = (2.6 + 1.6 * size.max(0.3).log10()).round().clamp(2.0, 6.0) as usize;
    let h = storeys as f32 * 0.02 * g.sqrt();
    let (dep, ts) = (0.03 * g, 0.016 * g);
    let wheel = c.water_power();
    let tail = if wheel { 0.004 } else { 0.034 * g };
    let room = 2.0 * hw - 0.016 - tail;
    let wanted = (size.sqrt() * 0.5).ceil().max(1.0) as usize;
    let (cols, len) = c.spread(room, 0.16 * g, 0.012, 3);
    let weaving = size >= 6.0 && c.w.founded >= 1790;
    let shed_d = if weaving { (2.0 * hd - 0.02) * 0.28 } else { 0.0 };
    let pitch = dep + 0.026 * g;
    let rows = c.fit(2.0 * hd - 0.02 - shed_d - 0.012 * g, pitch, 4).min(wanted.div_ceil(cols));
    let mut left = wanted.min(rows * cols);
    let zrow = |j: usize| -hd + 0.008 + dep * 0.5 + j as f32 * pitch;
    for j in 0..rows {
        for i in 0..cols {
            if left == 0 {
                break;
            }
            left -= 1;
            let (x, z) = (-hw + 0.008 + len * 0.5 + i as f32 * (len + 0.012), zrow(j));
            let m = c.shed_pitched(x, z, len, dep, h, 0.22, false, wall, roof);
            c.windows(&m, storeys, 0, storeys, 0.011, size >= 1.0, lit);
            c.tower(x - len * 0.2, z + dep * 0.5 + ts * 0.5 - 0.001, ts, h + 0.014 * g, wall, roof);
            if wheel && i == 0 {
                let r = (0.02 * g).min(h * 0.55);
                let (wx, wz) = (x + len * 0.5 - r - 0.003, z + dep * 0.5 + 0.004 * g);
                c.race((hw - 0.003, wz), (wx - r, wz), 0.007 * g);
                c.wheel(wx, wz, r, 0.006 * g, false, c.tone(*pal.timber.waterwheel));
                if j == 0 {
                    // the bell that calls the hands
                    c.tower(x + len * 0.25, z, 0.008 * g, h + m.rise + 0.004 * g, wall, roof);
                }
            }
        }
        if !wheel && j % 2 == 0 {
            // the chimney at the end of the row, and the engine house before it
            let (r, ew, ed) = (0.0075 * g, 0.026 * g, 0.024 * g);
            let (ex, zc) = (-hw + 0.008 + cols as f32 * (len + 0.012) - 0.012 + 0.003 + ew * 0.5, zrow(j) - dep * 0.5 + r * 1.75);
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
    }
    // the weaving shed, under its north lights, across the front of the yard
    let front = zrow(rows - 1) + dep * 0.5 + ts;
    let strip = 0.014 * g;
    let mut z0 = front + 0.008;
    if weaving && hd - 0.012 - strip - z0 > 0.03 {
        let (wl, wd) = (2.0 * hw - 0.024, hd - 0.012 - strip - z0);
        let teeth = ((wl / (0.018 * g)).floor() as usize).max(2);
        c.sawtooth(0.0, z0 + wd * 0.5, wl, wd, 0.016 * g, teeth, false, wall, roof);
        z0 += wd + 0.004;
    }
    // bales by the gate
    let d = hd - 0.012 - z0;
    if d > 0.006 {
        let w = hw * 0.8;
        c.stacks(hw * 0.45, z0 + d * 0.5, w, d.min(0.03 * g), c.fit(w, 0.011 * g, 16), c.fit(d.min(0.03 * g), 0.011 * g, 3), 0.01 * g, *pal.goods.bales);
    }
}
