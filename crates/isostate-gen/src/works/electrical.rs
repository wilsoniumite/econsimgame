//! The electrical works.

use super::kit::{Ctx, Face};

/// A works making electrical goods: long workshops of brick and glass under
/// north lights at the back of the yard, more and longer rows of them as it
/// grows; a narrow range of offices along the road, of two storeys (three in
/// a great works), either side of the gate once it is big enough; the boiler
/// house in the far corner with its modest square stack; and crates and
/// drums waiting in the yard between.
pub fn electrical_works(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_setts, false);
    let gate = 0.026 * g;
    c.wall(c.yard_wall(), 0.008, gate);
    let gate = gate.min(hw * 1.2);
    let (wall, roof) = (c.masonry(false), c.roofing());
    // the offices along the road, to the left of the gate and, in a bigger works, to its right
    let (od, storeys) = (0.017 * g, if size >= 10.0 { 3 } else { 2 });
    let oz = hd - 0.008 - od * 0.5 - 0.002;
    let ol = hw - 0.008 - gate * 0.5 - 0.006;
    if ol >= 0.024 {
        for side in [-1.0f32, 1.0] {
            if side > 0.0 && size < 2.0 {
                continue;
            }
            let s = c.shed_pitched(side * (gate * 0.5 + 0.006 + ol * 0.5), oz, ol, od, c.storeys(storeys), 0.45, false, wall, roof);
            c.windows(&s, 0, storeys, true);
            c.door(&s, Face::Back, side * ol * 0.25);
            c.door(&s, Face::Front, -side * ol * 0.3);
        }
    }
    let office = if ol >= 0.024 { od + 0.004 } else { 0.0 };
    // the boiler house in the far right corner, and its stack
    let (bw, bd, bh) = (0.026 * g, (0.022 * g).min(2.0 * hd - 0.02), 0.022 * g);
    let (bx, bz) = (hw - 0.008 - bw * 0.5, -hd + 0.008 + bd * 0.5);
    let b = c.shed(bx, bz, bw, bd, bh, false, c.masonry(true), roof);
    if !c.going_up() {
        c.glow(&b, Face::Front, -0.004 * g, 0.004 * g, 0.0, 0.01 * g, true);
    }
    c.stack(bx + bw * 0.5 - 0.005 * g, bz - bd * 0.5 + 0.005 * g, bh + 0.05 * g, 0.0075 * g, c.tone(*pal.brick.chimney), 0.5);
    // the workshops: rows from the back, as long as the yard is wide less the boiler house; so
    // much length of them as the works' size wants (about a tenth of a unit at the standard size)
    let room = 2.0 * hw - 0.016 - bw - 0.006;
    let (dep, h) = ((0.05 * g).min(2.0 * hd - 0.02 - office), 0.026 * g);
    let wanted = 0.1 * g * size.max(0.4).powf(0.55);
    let gap = 0.012 * g;
    let depth = 2.0 * hd - 0.016 - office - 0.012;
    let rows = c.fit(depth, dep + gap, 5).min((wanted / room).ceil().max(1.0) as usize);
    let mut left = wanted;
    let mut front = -hd + 0.008;
    let tooth = 0.017 * g;
    for j in 0..rows {
        let len = left.min(room).max((0.05 * g).min(room));
        left -= len;
        let (x, z) = (-hw + 0.008 + len * 0.5, -hd + 0.008 + dep * 0.5 + j as f32 * (dep + gap));
        let s = if c.late() {
            c.sawtooth(x, z, len, dep, h, ((len / tooth).round() as usize).max(2), false, wall, roof)
        } else {
            let s = c.shed(x, z, len, dep, h, false, wall, roof);
            c.roof_lights(&s, ((len / (0.03 * g)).round() as usize).max(1));
            s
        };
        if !c.going_up() {
            // a loading door at the end of each shop nearer the gate, and the way in from the yard
            c.doors(&s, Face::Front, len * 0.5 - 0.004 - 0.014 * g, len * 0.5 - 0.004, 0.0, (0.017 * g).min(h - 0.005), false, *pal.timber.door);
            c.door(&s, Face::Front, -len * 0.5 + 0.012);
        }
        front = z + dep * 0.5;
    }
    // what waits in the yard: crates of goods going out, and drums of oil and varnish
    let (z0, z1) = (front + 0.008, hd - 0.008 - office - 0.006);
    let d = z1 - z0;
    if d > 0.01 {
        let crates = (hw - 0.012).min(0.06 * g);
        c.stacks(-hw + 0.01 + crates * 0.5, z0 + d * 0.5, crates, d, c.fit(crates, 0.012 * g, 8), c.fit(d, 0.01 * g, 4), 0.006 * g, *pal.timber.planks);
        let w = (hw * 0.5).min(0.05 * g);
        c.drums(hw - 0.01 - w * 0.5, z0 + d * 0.5, w, d * 0.8, *pal.iron.plate);
    }
}
