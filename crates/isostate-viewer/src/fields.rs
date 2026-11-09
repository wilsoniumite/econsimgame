//! The field system: how the land is divided into fields, and which field
//! decides the land class at a spot.
//!
//! Blocks of land are the quads of a jittered grid, turned so they do not line
//! up with the 2 km cells and gently warped so that their edges bend a little
//! (more in some districts, the old enclosures, than in others, the planned
//! ones). Each block is cut again and again, always across its longer side at a
//! point near the middle, into fields: so there are long hedge lines with
//! T-junctions, fields of four sides, and blocks of near-parallel fields, as in
//! enclosed country. Some blocks are laid out in long strips.
//!
//! The first `PARCEL_LEVEL` cuts make parcels, and a whole parcel takes one land
//! class: the class rule in world.rs looks up the patch noise and the warped
//! cell at the parcel's centre, so every boundary between farm, pasture, wood
//! and town runs along a hedge line. How much further a parcel is cut depends on
//! the ground (`Ground::field_depth`): small closes round the towns, larger
//! fields in the lowland and large enclosures on the hills.
//!
//! terrain.wesl repeats all of this line for line, in the same order of
//! operations, so the ground it paints agrees with the trees and houses placed
//! here. Change one side and you must change the other.

use bevy::math::Vec2;

use crate::world::{hash2, vnoise};

/// Size of a block, in cells.
pub const BLOCK: f32 = 0.9;
/// The block grid is turned by 0.33 radians against the cells.
const TURN_C: f32 = 0.946_042_3;
const TURN_S: f32 = 0.324_043_6;
/// Cuts that make a parcel, the unit that takes one land class.
pub const PARCEL_LEVEL: u32 = 3;
/// The most cuts a field can have.
pub const MAX_DEPTH: u32 = 5;

pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// An integer hash of three numbers, 0..1, in the same wrapping u32 arithmetic
/// as the shader.
pub fn hash3(a: u32, b: u32, c: u32) -> f32 {
    let mut h = a.wrapping_mul(374_761_393) ^ b.wrapping_mul(668_265_263) ^ c.wrapping_mul(1_442_695_041);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    h = (h ^ (h >> 15)).wrapping_mul(2_246_822_519);
    h ^= h >> 13;
    h as f32 / 4_294_967_295.0
}

/// Map position (cells) to block space.
pub fn to_block(p: Vec2) -> Vec2 {
    let q = Vec2::new(TURN_C * p.x + TURN_S * p.y, -TURN_S * p.x + TURN_C * p.y) / BLOCK;
    // old enclosures bend, planned ones run straight
    let a = 0.13 * smoothstep(-0.3, 0.5, vnoise(q.x * 0.21 + 3.7, q.y * 0.21 + 1.3, 141));
    let wx = vnoise(q.x * 1.1, q.y * 1.1, 143);
    let wy = vnoise(q.x * 1.1 + 7.1, q.y * 1.1 + 2.9, 147);
    Vec2::new(q.x + a * wx, q.y + a * wy)
}

/// Back from block space to the map, leaving out the warp: a fixed point near
/// the true one, good for looking things up about a field.
pub fn from_block(q: Vec2) -> Vec2 {
    let s = q * BLOCK;
    Vec2::new(TURN_C * s.x - TURN_S * s.y, TURN_S * s.x + TURN_C * s.y)
}

/// Corner (i, j) of the jittered block grid.
pub fn corner(i: i32, j: i32) -> Vec2 {
    Vec2::new(i as f32 + 0.5 * (hash2(i, j, 151) - 0.5), j as f32 + 0.5 * (hash2(i, j, 153) - 0.5))
}

fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

/// Where p lies in the quad a b c d (a to b is u, a to d is v), after Inigo
/// Quilez's inverse bilinear interpolation.
pub fn inv_bilinear(p: Vec2, a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Vec2 {
    let e = b - a;
    let f = d - a;
    let g = a - b + c - d;
    let h = p - a;
    let k2 = cross(g, f);
    let k1 = cross(e, f) + cross(h, g);
    let k0 = cross(h, e);
    if k2.abs() < 1e-4 {
        let v = -k0 / k1;
        let u = (h.x - f.x * v) / (e.x + g.x * v);
        return Vec2::new(u, v);
    }
    let w = (k1 * k1 - 4.0 * k0 * k2).max(0.0).sqrt();
    let ik2 = 0.5 / k2;
    let mut v = (-k1 - w) * ik2;
    let mut u = (h.x - f.x * v) / (e.x + g.x * v);
    if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
        v = (-k1 + w) * ik2;
        u = (h.x - f.x * v) / (e.x + g.x * v);
    }
    Vec2::new(u, v)
}

pub fn bilinear(a: Vec2, b: Vec2, c: Vec2, d: Vec2, uv: Vec2) -> Vec2 {
    a + (b - a) * uv.x + (d - a) * uv.y + (a - b + c - d) * (uv.x * uv.y)
}

/// The block a point of block space lies in, with the point's (u, v) there.
#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub i: i32,
    pub j: i32,
    pub uv: Vec2,
    pub a: Vec2,
    pub b: Vec2,
    pub c: Vec2,
    pub d: Vec2,
}

pub fn locate(q: Vec2) -> Block {
    let mut i = q.x.floor() as i32;
    let mut j = q.y.floor() as i32;
    let mut out = Block { i, j, uv: Vec2::ZERO, a: Vec2::ZERO, b: Vec2::ZERO, c: Vec2::ZERO, d: Vec2::ZERO };
    for _ in 0..4 {
        let (a, b, c, d) = (corner(i, j), corner(i + 1, j), corner(i + 1, j + 1), corner(i, j + 1));
        let uv = inv_bilinear(q, a, b, c, d);
        out = Block { i, j, uv, a, b, c, d };
        let di = if uv.x < 0.0 { -1 } else if uv.x >= 1.0 { 1 } else { 0 };
        let dj = if uv.y < 0.0 { -1 } else if uv.y >= 1.0 { 1 } else { 0 };
        if di == 0 && dj == 0 {
            break;
        }
        i += di;
        j += dj;
    }
    out.uv = out.uv.clamp(Vec2::ZERO, Vec2::splat(0.99999));
    out
}

/// A field (or a parcel): its box in the block's (u, v), and its number within
/// the block (1 for the whole block, then 2n and 2n + 1 for the two halves of n).
#[derive(Clone, Copy, Debug)]
pub struct Cut {
    pub lo: Vec2,
    pub hi: Vec2,
    pub id: u32,
}

/// Cut the block down from `from` (at level `level`) to `depth` cuts, keeping
/// the piece that holds the block's point.
pub fn cut(bl: &Block, from: Cut, level: u32, depth: u32) -> Cut {
    let lu = 0.5 * ((bl.b - bl.a).length() + (bl.c - bl.d).length());
    let lv = 0.5 * ((bl.d - bl.a).length() + (bl.c - bl.b).length());
    let (bi, bj) = (bl.i as u32, bl.j as u32);
    // long strips in some blocks: below the parcels, always across the strips
    let strips = hash3(bi, bj, 157) < 0.25;
    let strip_u = hash3(bi, bj, 158) < 0.5;
    let mut c = from;
    let mut lvl = level;
    while lvl < depth {
        let ex = (c.hi.x - c.lo.x) * lu;
        let ey = (c.hi.y - c.lo.y) * lv;
        let k = hash3(bi, bj.wrapping_add(c.id.wrapping_mul(7919)), 160 + lvl);
        let mut along_u = ex >= ey;
        if strips && lvl >= PARCEL_LEVEL {
            along_u = strip_u;
        } else if k < 0.12 {
            along_u = !along_u;
        }
        let t = 0.36 + 0.28 * hash3(bi.wrapping_add(c.id.wrapping_mul(104_729)), bj, 170 + lvl);
        if along_u {
            let s = c.lo.x + (c.hi.x - c.lo.x) * t;
            if bl.uv.x < s {
                c.hi.x = s;
                c.id *= 2;
            } else {
                c.lo.x = s;
                c.id = c.id * 2 + 1;
            }
        } else {
            let s = c.lo.y + (c.hi.y - c.lo.y) * t;
            if bl.uv.y < s {
                c.hi.y = s;
                c.id *= 2;
            } else {
                c.lo.y = s;
                c.id = c.id * 2 + 1;
            }
        }
        lvl += 1;
    }
    c
}

/// The parcel at a map position, and the fixed point that stands for it.
pub fn parcel(p: Vec2) -> (Block, Cut, Vec2) {
    let bl = locate(to_block(p));
    let pc = cut(&bl, Cut { lo: Vec2::ZERO, hi: Vec2::ONE, id: 1 }, 0, PARCEL_LEVEL);
    let centre = from_block(bilinear(bl.a, bl.b, bl.c, bl.d, (pc.lo + pc.hi) * 0.5));
    (bl, pc, centre)
}

/// A field, with how far the spot is from its nearest edge (in cells) and which
/// way that edge lies, on the map.
#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub block: Block,
    pub cut: Cut,
    pub edge: f32,
    pub toward: Vec2,
}

/// A stable number for a field, for its crop and the like.
pub fn field_key(f: &Field) -> u32 {
    (f.block.i as u32).wrapping_mul(73_856_093) ^ (f.block.j as u32).wrapping_mul(19_349_663) ^ f.cut.id.wrapping_mul(83_492_791)
}

/// The field at a map position, given the parcel there and its depth.
pub fn field(bl: &Block, pc: Cut, depth: u32) -> Field {
    let c = cut(bl, pc, PARCEL_LEVEL, depth.max(PARCEL_LEVEL));
    // distance to the nearest side, through the Jacobian of the quad
    let g = bl.a - bl.b + bl.c - bl.d;
    let ju = (bl.b - bl.a) + g * bl.uv.y;
    let jv = (bl.d - bl.a) + g * bl.uv.x;
    let det = cross(ju, jv).abs().max(1e-6);
    let (du0, du1) = (bl.uv.x - c.lo.x, c.hi.x - bl.uv.x);
    let (dv0, dv1) = (bl.uv.y - c.lo.y, c.hi.y - bl.uv.y);
    let su = det / jv.length().max(1e-6);
    let sv = det / ju.length().max(1e-6);
    let eu = du0.min(du1) * su;
    let ev = dv0.min(dv1) * sv;
    // gradients of u and v in block space
    let gu = Vec2::new(jv.y, -jv.x) / cross(ju, jv);
    let gv = Vec2::new(-ju.y, ju.x) / cross(ju, jv);
    let (edge, dir) = if eu < ev {
        (eu, if du0 < du1 { -gu } else { gu })
    } else {
        (ev, if dv0 < dv1 { -gv } else { gv })
    };
    let dir = dir.normalize_or_zero();
    // back to the map: undo the turn (the warp is small)
    let toward = Vec2::new(TURN_C * dir.x - TURN_S * dir.y, TURN_S * dir.x + TURN_C * dir.y);
    Field { block: *bl, cut: c, edge: edge * BLOCK, toward }
}
