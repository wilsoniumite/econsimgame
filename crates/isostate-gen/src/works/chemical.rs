//! The chemical plant: the nitrogen works, oil made from coal, the ammonia-soda works.

use super::kit::{Ctx, Face};
use crate::colour::shade;

/// A chemical plant, laid out in trains from the back of the yard: each
/// train a house at its left where the compressors run (or, in a works that
/// heats coal in retorts, its firing floor), with louvres along its ridge; a
/// row of iron columns beside it with their platforms and ladders; a rack of
/// pipes before them, which a branch joins from the house; and round tanks
/// before the rack. As many trains as the works' size wants and its yard
/// holds, and in each as many columns and tanks as its width holds. A tall
/// slim stack in the far corner, and coal in a heap by the gate. (A plant on
/// its plot has its coal laid out by the plan, where its process takes coal.)
pub fn chemical_plant(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_cinders, false);
    c.wall(c.yard_wall(), 0.008, 0.024 * g);
    let iron = c.tone(*pal.iron.plate);
    // the stack, tall and slim, in the far right corner
    let sr = 0.0052 * g;
    let (sx, sz) = (hw - 0.008 - sr * 1.8, -hd + 0.008 + sr * 1.8);
    c.chimney(sx, sz, 0.15 * g * size.max(1.0).powf(0.04), sr, 0.8);
    // the depths of a train's parts: the columns (their platforms reach out round them), the
    // rack, the tanks; and the room kept before the gate for the coal and the carts
    let cr = 0.0058 * g;
    let reach = cr + 0.0035 + 0.0016;
    let gap = 0.006 * g;
    let tr = 0.012 * g;
    let train = 2.0 * reach + gap + 2.0 * tr + gap;
    let front_room = 0.034 * g;
    let heap = (0.016 * g).min(hw * 0.3);
    // a train to so much capacity: one to twice the standard size, two at four times it, four at twenty
    let wanted = (size / 2.0).max(0.4).powf(0.5).ceil().max(1.0) as usize;
    let room = 2.0 * hd - 0.016 - front_room;
    let trains = if room >= train { c.fit(room, train + gap, 6).min(wanted) } else { 1 };
    // (a small yard draws its one train in to fit)
    let k = if room >= train { 1.0 } else { ((2.0 * hd - 0.016 - 0.012) / train).clamp(0.4, 1.0) };
    let mut z = -hd + 0.008;
    for t in 0..trains {
        z = one_train(c, z, k, t, (sx, sr), iron);
        z += gap;
    }
    // the coal, by the gate
    let r = heap.min((hd - 0.01 - z) * 0.5);
    if r >= 0.005 {
        c.heap(-hw + 0.008 + r, hd - 0.01 - r, r, r * 0.7, *pal.goods.coal);
    }
}

/// One train of a chemical plant, from `z` toward the gate, drawn in by `k`
/// where the yard is small; `t` is which train it is, and `stack` is where the
/// stack stands and how wide it is, which the first keeps clear of. It gives
/// back the front of the train.
fn one_train(c: &mut Ctx, z: f32, k: f32, t: usize, stack: (f32, f32), iron: crate::colour::Rgba) -> f32 {
    let (hw, g, size, pal) = (c.hw, c.g, c.w.size, c.pal);
    let roof = c.roofing();
    let cr = 0.0058 * g * k.max(0.7);
    let reach = cr + 0.0035 + 0.0016;
    let gap = 0.006 * g;
    // the house, at the left of the columns
    let (hl, hdp, hh) = ((0.05 * g).min(2.0 * hw * 0.34), (2.0 * reach).max(0.02), 0.03 * g);
    let (hx, hz) = (-hw + 0.008 + hl * 0.5, z + hdp * 0.5);
    let house = c.shed(hx, hz, hl, hdp, hh, false, c.masonry(true), roof);
    // (a louvre along the ridge of all but the smallest house: on that one it would be most of its roof)
    if hl >= 0.04 {
        c.ridge_vent(&house, 0.6, false);
    }
    if !c.going_up() {
        c.doors(&house, Face::Front, -hl * 0.5 + 0.004, -hl * 0.5 + 0.004 + 0.013 * g, 0.0, (0.016 * g).min(hh - 0.005), false, *pal.timber.door);
    }
    // the columns, from the house to the stack (or, behind the first train, to the wall); not all of a height
    let xa = hx + hl * 0.5 + 0.008;
    let xb = if t == 0 { stack.0 - stack.1 * 1.8 - 0.006 } else { hw - 0.008 };
    let per = c.fit(xb - xa, 2.0 * reach + gap, 12);
    let tall = 0.08 * g * size.max(1.0).powf(0.03);
    let zc = z + reach;
    for i in 0..per {
        let x = xa + (xb - xa) * (i as f32 + 0.5) / per as f32;
        let top = c.column(x, zc, cr, tall * [1.0, 0.78, 0.9, 0.84][(i + t) % 4], iron);
        if i == 0 {
            c.steam(top, 0.35 * g);
        }
    }
    // the rack before them, and the branch from the house
    let zr = (z + 2.0 * reach).max(hz + hdp * 0.5) + 0.004 * g;
    let rh = 0.016 * g;
    c.pipe_rack((hx, zr), (hw - 0.012, zr), rh, if size >= 4.0 { 3 } else { 2 });
    c.pipe_rack((hx + hl * 0.5 - 0.004, hz + hdp * 0.5), (hx + hl * 0.5 - 0.004, zr), rh, 1);
    // the tanks before the rack
    let tr = 0.012 * g * k;
    let z0 = zr + 0.005 * g;
    let (ta, tb) = (-hw + 0.008, hw - 0.008);
    let n = c.fit(tb - ta, 2.0 * tr + gap, 10);
    for i in 0..n {
        let x = ta + (tb - ta) * (i as f32 + 0.5) / n as f32;
        // (painted paler than the columns, against the sun)
        c.tank(x, z0 + tr, tr, (0.018 * g).max(tr), shade(iron, 1.5));
    }
    z0 + 2.0 * tr
}
