//! The coal power station.

use super::kit::{Ctx, Face, BAY};
use crate::kit::FOOT;
use crate::colour::shade;

/// From this year a station built at `GREAT` times the standard size or more
/// is of a class of its own: a hall of concrete and cooling towers. One
/// founded before it keeps the brick hall at any size (and a works on its
/// plot is drawn stage by stage, each in the style of its own year, so a
/// brick station of 1900 enlarged in 1950 has its towers beside the old hall).
/// The size is the whole station's when the stage was built (`Ctx::whole`),
/// not the size its stage's buildings are drawn at, which is much less.
pub(super) const GREAT_FROM: i32 = 1930;
pub(super) const GREAT: f32 = 5.0;

/// A coal power station. To 1930, and after it below the great size: a brick
/// engine hall with tall arched windows and a lantern along its roof, the
/// boiler house beside it with louvres along its ridge, the tall round
/// chimneys behind (more of them as it is bigger), and the coal in heaps
/// down the side of the yard. A great station built from 1930: a turbine hall
/// of concrete, the tall boiler house, one or two very tall chimneys, and the
/// cooling towers, their steam rising white.
pub fn power_station(c: &mut Ctx) {
    let (g, pal) = (c.g, c.pal);
    c.ground(*pal.ground.yard_cinders, false);
    c.wall(c.yard_wall(), 0.008, 0.026 * g);
    if c.w.founded >= GREAT_FROM && c.whole >= GREAT {
        concrete(c);
    } else {
        brick(c);
    }
}

/// One set of a station's buildings, from the back of its row to the front:
/// the chimneys, the boiler house, the hall. `x` is its middle along the
/// front and `len` its length; `z0` the back of its row; `k` how much its
/// depths are drawn in to fit the yard.
struct Set {
    x: f32,
    len: f32,
    z0: f32,
    k: f32,
}

/// The sizes of a set's parts: the chimney's radius and how tall it is, and
/// the boiler house's and the hall's depth and height.
struct Parts {
    cr: f32,
    tall: f32,
    chimneys: usize,
    bd: f32,
    bh: f32,
    ed: f32,
    eh: f32,
}

impl Parts {
    /// The depth of a set's row, from the back of its chimneys to the front of its hall.
    fn depth(&self) -> f32 {
        2.0 * self.cr * 1.75 + 0.004 + self.bd + 0.004 + self.ed
    }
}

/// Lay out `wanted` sets of a station's buildings in rows from the back of
/// the yard, as many as its yard holds, leaving `aside` of the yard's width
/// on the right for what stands beside them; and give back where the front
/// of the last row is. A row of fewer sets than it has room for has them
/// longer, up to a quarter more than `longest`.
fn sets(c: &mut Ctx, p: &Parts, wanted: usize, longest: f32, aside: f32, concrete: bool) -> f32 {
    let (hw, hd, g) = (c.hw, c.hd, c.g);
    let room = (2.0 * hw - 0.016 - aside).max(0.03);
    let (per, _) = c.spread(room, longest, 0.008 * g, 8);
    // (and room before the gate for a cart to turn)
    let depth = 2.0 * hd - 0.016 - 0.018 * g;
    let k = (depth / p.depth()).min(1.0);
    let rows = c.fit(depth, p.depth() * k + 0.014 * g, 4).min(wanted.div_ceil(per));
    let mut left = wanted.min(per * rows).max(1);
    let mut front = -hd + 0.008;
    for j in 0..rows {
        let n = left.min(per);
        left -= n;
        let len = ((room - 0.008 * g * (n as f32 - 1.0)) / n as f32).min(longest * 1.25);
        let z0 = -hd + 0.008 + j as f32 * (p.depth() * k + 0.014 * g);
        for i in 0..n {
            let x = -hw + 0.008 + len * 0.5 + i as f32 * (len + 0.008 * g);
            set(c, p, &Set { x, len, z0, k }, concrete);
        }
        front = z0 + p.depth() * k;
    }
    front
}

/// One set: the chimneys at the back, the boiler house, and the hall before it.
fn set(c: &mut Ctx, p: &Parts, s: &Set, concrete: bool) {
    let (g, pal) = (c.g, c.pal);
    let roof = c.roofing();
    let (cr, bd, ed) = (p.cr * s.k.max(0.7), p.bd * s.k, p.ed * s.k);
    let strip = 2.0 * cr * 1.75 + 0.004;
    // the chimneys, standing a little apart along the back of the boiler house
    let zc = s.z0 + strip * 0.5;
    for i in 0..p.chimneys {
        let x = s.x - s.len * 0.5 + s.len * (i as f32 + 0.5) / p.chimneys as f32;
        c.chimney(x, zc, p.tall, cr, if concrete { 1.3 } else { 1.0 });
    }
    // the boiler house: sooty brick, a louvre along its ridge to let the heat out, a fire seen at its end
    let zb = s.z0 + strip + bd * 0.5;
    let b = c.shed_pitched(s.x, zb, s.len, bd, p.bh, if concrete { 0.18 } else { 0.32 }, false, c.masonry(true), roof);
    c.ridge_vent(&b, 0.7, false);
    if !c.going_up() {
        c.glow(&b, Face::EndB, -0.005 * g, 0.005 * g, 0.0, 0.011 * g, true);
    }
    // the hall where the engines turn the dynamos: tall windows a bay and a half apart down its
    // front (arched in brick, straight and glazed to the eaves in concrete), a great window in
    // its gable, doors at one end wide enough for a boiler, and a lantern along its ridge
    let ze = s.z0 + strip + bd + 0.004 + ed * 0.5;
    let wall = if concrete { c.tone(shade(*pal.stone.works, 1.12)) } else { c.masonry(false) };
    let e = c.shed_pitched(s.x, ze, s.len, ed, p.eh, if concrete { 0.14 } else { 0.3 }, false, wall, roof);
    if c.going_up() {
        return;
    }
    let pitch = 1.4 * BAY;
    let n = ((s.len - 0.03 * g) / pitch).floor().max(1.0) as usize;
    let (sill, head) = (FOOT + if concrete { 0.003 } else { 0.006 }, p.eh - if concrete { 0.004 } else { 0.007 });
    for i in 0..n {
        let u = -s.len * 0.5 + 0.015 * g + pitch * (i as f32 + 0.5) + ((s.len - 0.03 * g) - pitch * n as f32) * 0.5;
        c.light(&e, Face::Front, u - 0.0034, u + 0.0034, sill, head, !concrete);
    }
    c.doors(&e, Face::Front, s.len * 0.5 - 0.004 - 0.016 * g, s.len * 0.5 - 0.004, 0.0, (0.022 * g).min(p.eh - 0.006), !concrete, *pal.timber.door);
    let half = (ed * 0.22).min(0.012);
    c.light(&e, Face::EndB, -half, half, FOOT + 0.006, p.eh - 0.004, !concrete);
    // (the lantern on all but the shortest hall: on that one it would be most of its roof)
    if s.len >= 0.07 {
        c.ridge_vent(&e, 0.6, true);
    }
}

/// A station of the brick age, of any size.
fn brick(c: &mut Ctx) {
    let (hw, hd, g, size) = (c.hw, c.hd, c.g, c.w.size);
    let parts = Parts {
        cr: 0.0072 * g,
        tall: 0.16 * g * size.max(1.0).powf(0.03),
        chimneys: if size >= 2.5 { 2 } else { 1 },
        bd: 0.028 * g,
        bh: 0.04 * g,
        ed: 0.034 * g,
        eh: 0.034 * g,
    };
    // (the coal lies down the right of the yard, which a station on its plot leaves to the plan)
    let aside = if c.stock_apart { 0.0 } else { 0.034 * g };
    // a set of buildings to so much capacity, as the ground grows: one to the standard size, three at
    // four times it, seven at twenty, sixteen at a hundred
    let wanted = size.max(0.4).powf(0.6).ceil().max(1.0) as usize;
    let front = sets(c, &parts, wanted, 0.12 * g, aside, false);
    coal(c, hw - 0.008 - aside * 0.5, aside * 0.5);
    switch_house(c, front, hd);
}

/// A great station built from 1930: the cooling towers in a column or two at
/// the right of the yard, a set of concrete halls to the left of them.
fn concrete(c: &mut Ctx) {
    let (hw, hd, g, size, pal) = (c.hw, c.hd, c.g, c.w.size, c.pal);
    let col = c.tone(shade(*pal.stone.works, 1.12));
    // the towers: one at five times the standard size, two at twenty, five at a hundred
    let (tr, th) = (0.036 * g, 0.14 * g);
    let wanted = ((size / 5.0).sqrt().ceil() as usize).clamp(1, 8);
    let pitch = 2.0 * tr + 0.006 * g;
    let per = c.fit(2.0 * hd - 0.016, pitch, 4);
    let cols = wanted.div_ceil(per).min(c.fit(hw * 0.9, pitch, 4));
    let towers = wanted.min(per * cols);
    for k in 0..towers {
        let (i, j) = (k / per, k % per);
        let (x, z) = (hw - 0.008 - tr - i as f32 * pitch, -hd + 0.008 + tr + j as f32 * pitch);
        let top = c.cooling_tower(x, z, tr, th, col);
        c.steam(top, 1.4 * g);
    }
    let aside = cols as f32 * pitch + 0.004;
    let parts = Parts {
        cr: 0.0085 * g,
        tall: 0.2 * g,
        chimneys: if size >= 40.0 { 2 } else { 1 },
        bd: 0.04 * g,
        bh: 0.07 * g,
        ed: 0.042 * g,
        eh: 0.046 * g,
    };
    // a set to nine times the standard size: one at five, two at twenty, four at a hundred
    let wanted = (size / 9.0).sqrt().ceil().max(1.0) as usize;
    let front = sets(c, &parts, wanted, 0.17 * g, aside, true);
    // the coal in a row of great heaps before the halls
    let room = hd - 0.012 - front;
    let r = (room * 0.5 - 0.002).min(0.024 * g);
    if r >= 0.006 {
        let w = 2.0 * hw - 0.016 - aside;
        let n = c.fit(w, 2.0 * r + 0.004, 10);
        for i in 0..n {
            let x = -hw + 0.008 + w * (i as f32 + 0.5) / n as f32;
            if c.stocked() || i == 0 {
                c.heap(x, front + 0.004 + r, r * 0.95, r * 0.6, *pal.goods.coal);
            }
        }
    }
}

/// The coal in heaps down a strip of the yard about `x`, `r` across each:
/// as much of the strip as the works' stock fills. (A station on its plot has
/// its coal laid out by the plan.)
fn coal(c: &mut Ctx, x: f32, r: f32) {
    if r < 0.006 {
        return;
    }
    let (hd, pal) = (c.hd, c.pal);
    let r = r.min(0.02 * c.g);
    let n = c.fit(2.0 * hd - 0.02, 2.0 * r + 0.003, 8);
    let coal = *pal.goods.coal;
    for i in 0..n {
        let z = -hd + 0.01 + r + i as f32 * (2.0 * r + 0.003);
        if c.stocked() || i == 0 {
            c.heap(x, z, r * 0.95, r * 0.7, coal);
        }
    }
}

/// The switch house and offices before the halls, in a station with ground
/// for them: two storeys of windows, a door to the yard.
fn switch_house(c: &mut Ctx, front: f32, hd: f32) {
    let g = c.g;
    let room = hd - 0.012 - front;
    let (len, dep) = (0.05 * g, 0.022 * g);
    if room < dep + 0.008 || c.w.size < 1.5 {
        return;
    }
    let x = -c.hw + 0.008 + len * 0.5;
    let s = c.shed_pitched(x, front + 0.006 + dep * 0.5, len, dep, c.storeys(2), 0.45, false, c.masonry(false), c.roofing());
    c.windows(&s, 0, 2, true);
    c.door(&s, Face::Front, len * 0.2);
}
