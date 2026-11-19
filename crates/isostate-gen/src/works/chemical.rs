//! The chemical plant: the nitrogen works, oil made from coal, the ammonia-soda works.

use super::kit::{Ctx, Face};

/// A chemical plant: the house at the back of the yard where the compressors
/// run (or, in a works that heats coal in retorts, its firing floor), with
/// louvres along its ridge; a row of iron columns beside it with their
/// platforms and ladders, more of them, and a second row, as the works
/// grows; a rack of pipes before them, which a branch joins from the house;
/// round tanks before the rack; a tall slim stack in the far corner; and coal
/// in a heap by the gate. (A plant on its plot has its coal laid out by the
/// plan, where its process takes coal.)
pub fn chemical_plant(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_cinders, false);
    c.wall(c.yard_wall(), 0.008, 0.024 * g);
    let iron = c.tone(*pal.iron.plate);
    let roof = c.roofing();
    // the house
    let (hl, hdp, hh) = ((0.05 * g).min(2.0 * hw * 0.34), (0.028 * g).min(hd), 0.03 * g);
    let (hx, hz) = (-hw + 0.008 + hl * 0.5, -hd + 0.008 + hdp * 0.5);
    let house = c.shed(hx, hz, hl, hdp, hh, false, c.masonry(true), roof);
    // (a louvre along the ridge of all but the smallest house: on that one it would be most of its roof)
    if hl >= 0.04 {
        c.ridge_vent(&house, 0.6, false);
    }
    if !c.going_up() {
        c.doors(&house, Face::Front, -hl * 0.5 + 0.004, -hl * 0.5 + 0.004 + 0.013 * g, 0.0, (0.016 * g).min(hh - 0.005), false, *pal.timber.door);
    }
    // the stack, tall and slim, in the far right corner
    let sr = 0.0052 * g;
    let (sx, sz) = (hw - 0.008 - sr * 1.8, -hd + 0.008 + sr * 1.8);
    c.chimney(sx, sz, 0.15 * g * size.max(1.0).powf(0.04), sr, 0.8);
    // the columns, in a row (or two) along the back between the house and the stack
    let cr = 0.0058 * g;
    let reach = cr + 0.0035 + 0.0016;
    let (xa, xb) = (hx + hl * 0.5 + 0.008, sx - sr * 1.8 - 0.006);
    let pitch = 2.0 * reach + 0.006 * g;
    let per = c.fit(xb - xa, pitch, 12);
    let wanted = (2.0 * size.max(0.4).powf(0.4)).ceil() as usize;
    let rows = wanted.div_ceil(per).clamp(1, 2);
    let tall = 0.08 * g * size.max(1.0).powf(0.03);
    let mut back_of_rack = -hd + 0.008;
    for k in 0..wanted.min(per * rows) {
        let (i, j) = (k % per, k / per);
        let x = xa + (xb - xa) * (i as f32 + 0.5) / per as f32;
        let z = -hd + 0.008 + reach + j as f32 * pitch;
        // (they are not all of a height: the tallest does the most separating)
        let h = tall * [1.0, 0.78, 0.9][k % 3];
        let top = c.column(x, z, cr, h, iron);
        if k == 0 {
            c.steam(top, 0.35 * g);
        }
        back_of_rack = back_of_rack.max(z + reach);
    }
    // the rack of pipes before them, and the branch from the house
    let zr = back_of_rack.max(hz + hdp * 0.5) + 0.005 * g;
    let rh = 0.016 * g;
    let pipes = if size >= 4.0 { 3 } else { 2 };
    c.pipe_rack((hx, zr), (xb, zr), rh, pipes);
    c.pipe_rack((hx + hl * 0.5 - 0.004, hz + hdp * 0.5), (hx + hl * 0.5 - 0.004, zr), rh, 1);
    // the tanks before the rack
    let z0 = zr + 0.007 * g;
    let room = hd - 0.008 - z0;
    let tr = (0.012 * g).min(room * 0.5 - 0.002);
    let heap = (0.016 * g).min(hw * 0.3);
    if tr >= 0.005 {
        let (ta, tb) = (-hw + 0.008 + if c.stock_apart { 0.0 } else { 2.0 * heap + 0.006 }, hw - 0.008);
        let wanted = (1.5 * size.max(0.4).powf(0.35)).ceil() as usize;
        let n = c.fit(tb - ta, 2.0 * tr + 0.006 * g, 10).min(wanted);
        for i in 0..n {
            let x = ta + (tb - ta) * (i as f32 + 0.5) / n as f32;
            c.tank(x, z0 + tr, tr, (0.018 * g).max(tr), iron);
        }
    }
    // the coal, by the gate
    let r = heap.min((hd - 0.008 - z0) * 0.5);
    if r >= 0.005 {
        c.heap(-hw + 0.008 + r, hd - 0.01 - r, r, r * 0.7, *pal.goods.coal);
    }
}
