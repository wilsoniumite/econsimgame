//! The trades of the town: handloom weavers, smithies, millwrights, and the
//! shops and trades the simulation calls services.

use super::kit::{Ctx, Face, Shed, STOREY};
use crate::colour::{pick, shade, Rgba};

/// Handloom weavers: a cottage with the long window of the loomshop under
/// its eaves; then a row of them; then a fold, rows round three sides of a
/// court with the cloth out on tenter frames; and at last a loomshop of
/// several floors in the middle of it.
pub fn hand_loom(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_setts, false);
    c.wall(c.yard_wall(), 0.006, 0.016 * g);
    let washed = c.early() && c.w.seed.u < 0.6;
    let wall = c.tone(if washed { *pal.walls.weaver_cottage } else if c.early() { *pal.stone.works } else { pal.brick.terrace(c.w.seed.v) });
    let roof = c.roofing();
    let (uw, dep) = (0.03 * g, 0.026 * g);
    let storeys = if size < 2.5 { 2 } else { 3 };

    // the row along the back, and from a middling size a wing down each side
    let n_back = c.fit(2.0 * hw - 0.012, uw, 12);
    let zb = -hd + 0.006 + dep * 0.5;
    cottages(c, 0.0, zb, n_back, uw, dep, storeys, false, false, wall, roof);
    let front = zb + dep * 0.5;
    let room = hd - 0.008 - front - 0.004;
    let n_side = if size >= 6.0 { (room / uw).floor() as usize } else { 0 };
    if n_side > 0 {
        for side in [-1.0f32, 1.0] {
            let z = front + 0.004 + n_side as f32 * uw * 0.5;
            cottages(c, side * (hw - 0.006 - dep * 0.5), z, n_side, uw, dep, storeys, true, side > 0.0, wall, roof);
        }
    }
    // the court between them
    let court = if n_side > 0 { hw - 0.012 - dep } else { hw - 0.01 };
    let mut z0 = front + 0.008;
    if size >= 20.0 {
        // a loomshop: every floor a long window
        let floors = if size >= 60.0 { 4 } else { 3 };
        let (len, d2) = ((0.1 * g).min(court * 1.7), 0.03 * g);
        let s = c.shed(0.0, z0 + 0.004 + d2 * 0.5, len, d2, floors as f32 * STOREY, false, wall, roof);
        for f in 0..floors {
            c.window_band(&s, f as f32 * STOREY + 0.007, STOREY * 0.45, *pal.glass.window);
        }
        z0 += 0.012 + d2;
    }
    // cloth stretched on the tenter frames
    let depth = hd - 0.012 - z0;
    if c.busy() && depth >= 0.008 {
        let rows = c.fit(depth, 0.011, 6);
        for i in 0..rows {
            let z = z0 + depth * (i as f32 + 0.5) / rows as f32;
            c.pile(-court * 0.1, z, court * 1.3, 0.0012, 0.007, false, *pal.goods.cloth);
        }
    }
}

/// A row of `n` weavers' cottages as one building: windows below, the long
/// loomshop window on the top floor, a door and a stack to each.
#[allow(clippy::too_many_arguments)]
fn cottages(c: &mut Ctx, x: f32, z: f32, n: usize, uw: f32, dep: f32, storeys: usize, across: bool, flip: bool, wall: Rgba, roof: Rgba) -> Shed {
    let pal = c.pal;
    let h = storeys as f32 * STOREY;
    let s = c.shed_pitched(x, z, n as f32 * uw, dep, h, 0.5, across, wall, roof);
    c.windows(&s, storeys, 0, storeys - 1, uw * 0.5, false, *pal.glass.window);
    c.window_band(&s, h - STOREY * 0.74, STOREY * 0.46, *pal.glass.window);
    for i in 0..n {
        let cx = -(n as f32) * uw * 0.5 + uw * (i as f32 + 0.5);
        if !c.going_up() {
            c.panel(&s, if flip { Face::Back } else { Face::Front }, cx - 0.003, cx + 0.003, 0.0, 0.013, false, *pal.timber.door);
        }
        let (sx, sz) = s.local(cx + uw * 0.5 - 0.005, 0.0);
        c.stack(sx, sz, h + s.rise + 0.007, 0.0065, c.tone(*pal.brick.chimney), if i % 3 == 0 { 0.3 } else { 0.0 });
    }
    s
}

/// A smithy: a low forge open to the yard, a hearth to each bay with its
/// broad stack behind; a bigger one is a longer rank of hearths, then two
/// and three ranks.
pub fn smithy(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_setts, false);
    c.wall(c.yard_wall(), 0.007, 0.02 * g);
    let (wall, roof) = (c.masonry(false), c.roofing());
    let (bay, dep, h) = ((0.036 * g).min(2.0 * hw - 0.016), 0.03 * g, 0.02 * g);
    let wanted = size.sqrt().ceil().max(1.0) as usize;
    let per_row = c.fit(2.0 * hw - 0.016, bay, 8);
    let pitch = dep + 0.016 * g;
    let rows = c.fit(2.0 * hd - 0.02 - 0.012 * g, pitch, 4).min(wanted.div_ceil(per_row));
    let mut left = wanted.min(rows * per_row);
    let mut front = -hd;
    for j in 0..rows {
        let n = left.min(per_row);
        left -= n;
        let z = -hd + 0.01 + dep * 0.5 + j as f32 * pitch;
        let s = c.shed(0.0, z, n as f32 * bay, dep, h, false, wall, roof);
        for i in 0..n {
            let cx = -(n as f32) * bay * 0.5 + bay * (i as f32 + 0.5);
            // the open front, and the fire on the hearth
            if !c.going_up() {
                c.panel(&s, Face::Front, cx - bay * 0.36, cx + bay * 0.08, 0.0, h * 0.78, false, *pal.smoke.soot);
                c.glow(&s, Face::Front, cx + bay * 0.08, cx + bay * 0.36, 0.0, h * 0.78, false);
            }
            c.stack(cx, z - dep * 0.5 + 0.004 * g, h + s.rise + 0.022 * g, 0.012 * g, wall, 0.45);
        }
        front = z + dep * 0.5;
    }
    // coal for the hearths and bar iron to work
    if c.stocked() {
        let z = (front + hd) * 0.5;
        c.heap(-hw * 0.55, z, 0.007 * g, 0.006 * g, *pal.goods.coal);
        let bars = c.fit(hw, 0.03 * g, 5);
        for i in 0..bars {
            c.pile(hw * 0.5 - i as f32 * 0.006 * g, z, 0.004 * g, 0.02 * g, 0.003 * g, false, *pal.iron.plate);
        }
    }
}

/// A millwright: a boarded workshop and an open sawing shed, timber in the
/// stack, a wheel half made in the yard; more shops, more timber and a
/// crane as it grows, and its own waterwheel on the wall where there is water.
pub fn millwright(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.yard_earth, false);
    c.wall(*pal.timber.fence, 0.006, 0.022 * g);
    let boards = c.tone(*pal.timber.boards);
    let roof = c.roofing();
    let (shop, open, dep, h) = (0.044 * g, 0.03 * g, 0.03 * g, 0.03 * g);
    let room = 2.0 * hw - 0.016;
    let both = room >= shop + open + 0.006;
    let unit = if both { shop + open + 0.006 } else { shop.min(room) };
    let cols = c.fit(room, unit + 0.01, 3);
    let pitch = dep + 0.014 * g;
    let rows = c.fit(2.0 * hd * 0.6, pitch, 3);
    let mut front = -hd;
    let mut first: Option<Shed> = None;
    for j in 0..rows {
        for i in 0..cols {
            let x0 = (i as f32 - (cols as f32 - 1.0) * 0.5) * (unit + 0.01) - unit * 0.5;
            let z = -hd + 0.009 + dep * 0.5 + j as f32 * pitch;
            let len = if both { shop } else { unit };
            let s = c.shed(x0 + len * 0.5, z, len, dep, h, false, boards, roof);
            if !c.going_up() {
                c.panel(&s, Face::Front, -len * 0.2, len * 0.2, 0.0, h * 0.7, false, *pal.timber.door);
            }
            first.get_or_insert(s);
            if both {
                c.open_shed(x0 + shop + 0.006 + open * 0.5, z, open, dep * 0.9, h * 0.62, false, *pal.timber.headframe, roof);
                // a baulk on the trestles over the sawpit
                if c.stocked() {
                    c.pile(x0 + shop + 0.006 + open * 0.5, z, open * 0.9, 0.004 * g, h * 0.3, false, *pal.timber.logs);
                }
            }
            front = z + dep * 0.5;
        }
    }
    let s = first.unwrap();
    // the smith's hearth in the corner of the first shop
    c.stack(s.x - s.len * 0.5 + 0.006 * g, s.z - dep * 0.5 + 0.005 * g, h + s.rise + 0.016 * g, 0.008 * g, c.tone(*pal.brick.chimney), 0.4);
    if c.water_power() {
        let r = 0.016 * g;
        let (wx, wz) = (s.x - s.len * 0.5 + r + 0.002, s.z + dep * 0.5 + 0.004 * g);
        c.race((-hw + 0.004, wz), (wx + r, wz), 0.006 * g);
        c.wheel(wx, wz, r, 0.005 * g, false, c.tone(*pal.timber.waterwheel));
    }
    // sawn timber in the stack, logs waiting, and a wheel being built
    let depth = hd - 0.01 - front - 0.008;
    if depth > 0.008 {
        let z = front + 0.008 + depth * 0.5;
        c.stacks(-hw * 0.4, z, hw * 0.9, depth, c.fit(hw * 0.9, 0.03 * g, 6), c.fit(depth, 0.011 * g, 5), 0.007 * g, *pal.timber.planks);
        if c.stocked() {
            c.pile(hw * 0.72, z, 0.007 * g, (0.04 * g).min(depth), 0.007 * g, false, *pal.timber.logs);
            if size >= 0.8 {
                c.wheel(hw * 0.4, z, (0.013 * g).min(depth * 0.5), 0.004 * g, false, *pal.timber.planks);
            }
        }
        if size >= 8.0 {
            c.sheer_legs(hw * 0.12, z, 0.045 * g);
        }
    }
}

/// Shops and trades: a row of shop fronts under their awnings, facing a
/// paved forecourt; a longer and taller row, then wings either side, and
/// in a great concern a covered market in the court (glazed, once there is
/// iron and glass to roof it with).
pub fn shops(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    c.ground(*pal.ground.shop_pavement, false);
    let wall = c.tone(if c.early() { pal.walls.town_house(c.w.seed.v) } else { pal.brick.terrace(c.w.seed.v) });
    let roof = c.tone(if c.early() { *pal.roofs.tile_red } else { *pal.slate.town });
    let (sw, dep) = ((0.022 * g).min(2.0 * hw - 0.014), 0.026 * g);
    let storeys = if size < 3.0 { 2 } else { 3 };
    let wanted = (1.5 * size.sqrt()).ceil().max(1.0) as usize;
    let n_back = c.fit(2.0 * hw - 0.014, sw, 14).min(wanted);
    let zb = -hd + 0.007 + dep * 0.5;
    shop_row(c, 0.0, zb, n_back, sw, dep, storeys, false, Face::Front, wall, roof);
    let front = zb + dep * 0.5;
    let per_wing = (((hd - 0.007 - front - 0.003) / sw).floor() as usize).min((wanted - n_back).div_ceil(2));
    if per_wing > 0 {
        for side in [-1.0f32, 1.0] {
            let z = front + 0.003 + per_wing as f32 * sw * 0.5;
            shop_row(c, side * (hw - 0.007 - dep * 0.5), z, per_wing, sw, dep, storeys, true, if side < 0.0 { Face::Front } else { Face::Back }, wall, roof);
        }
    }
    // the covered market
    let (cw, cd) = (2.0 * (hw - 0.004 - dep) - 0.03 * g, hd - 0.004 - front - 0.03 * g);
    if wanted >= 9 && per_wing > 0 && cw > 0.03 && cd > 0.02 {
        let (len, d2, h) = (cw * 0.8, cd * 0.7, 0.024 * g);
        let glazed = c.late();
        let top = if glazed { c.tone(*pal.glass.roof_light) } else { roof };
        let s = c.shed(0.0, front + 0.015 * g + cd * 0.5, len, d2, h, false, c.tone(*pal.walls.market_hall), top);
        c.ridge_vent(&s, 0.7, glazed);
        if !c.going_up() {
            let n = c.fit(len, 0.016 * g, 20);
            for i in 0..n {
                let cx = -len * 0.5 + len * (i as f32 + 0.5) / n as f32;
                c.panel(&s, Face::Front, cx - len / n as f32 * 0.32, cx + len / n as f32 * 0.32, 0.0, h * 0.6, true, *pal.smoke.soot);
            }
        }
    }
}

/// A row of `n` shops as one building: to each a window, a painted fascia
/// and an awning on the face that looks on the forecourt, windows above,
/// and a stack to every other.
#[allow(clippy::too_many_arguments)]
fn shop_row(c: &mut Ctx, x: f32, z: f32, n: usize, sw: f32, dep: f32, storeys: usize, across: bool, face: Face, wall: Rgba, roof: Rgba) -> Shed {
    let pal = c.pal;
    let h = storeys as f32 * STOREY;
    let s = c.shed_pitched(x, z, n as f32 * sw, dep, h, 0.45, across, wall, roof);
    c.windows(&s, storeys, 1, storeys, sw * 0.5, false, *pal.glass.window);
    let out = if face == Face::Front { 1.0 } else { -1.0 };
    for i in 0..n {
        let cx = -(n as f32) * sw * 0.5 + sw * (i as f32 + 0.5);
        if !c.going_up() {
            let lit = if c.derelict() { *pal.smoke.soot } else { *pal.glass.window };
            c.panel(&s, face, cx - sw * 0.38, cx + sw * 0.38, 0.002, STOREY * 0.56, false, lit);
            c.panel(&s, face, cx - sw * 0.42, cx + sw * 0.42, STOREY * 0.56, STOREY * 0.74, false, c.tone(*pal.walls.shop_front));
        }
        if c.stocked() {
            // the awning, out over the pavement
            let col = pick(&pal.roofs.awnings, c.rnd());
            let (y0, y1, z0, z1) = (STOREY * 0.74, STOREY * 0.5, out * (dep * 0.5 + 0.0008), out * (dep * 0.5 + 0.0075));
            let (a, b, d, e) = (s.pt(cx - sw * 0.4, y0, z0), s.pt(cx + sw * 0.4, y0, z0), s.pt(cx + sw * 0.4, y1, z1), s.pt(cx - sw * 0.4, y1, z1));
            if out > 0.0 {
                c.solid.quad(e, d, b, a, col);
            } else {
                c.solid.quad(a, b, d, e, shade(col, 0.9));
            }
        }
        if i % 2 == 0 {
            let (sx, sz) = s.local(cx + sw * 0.5 - 0.005, 0.0);
            c.stack(sx, sz, h + s.rise + 0.007, 0.0065, c.tone(*pal.brick.chimney), if i % 4 == 0 { 0.28 } else { 0.0 });
        }
    }
    s
}
