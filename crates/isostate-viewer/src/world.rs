//! Shared spatial logic: the ground's heights, its rivers, and the rules that
//! turn each cell's land mix into fields, woods and towns. The class of every
//! parcel of the field system is decided here, once a year (`year_land`), and
//! handed to the ground shader (terrain.wesl) in a texture with a texel for
//! each parcel, so the two cannot disagree about a parcel. The shader repeats
//! only what is finer than a parcel: `hash2`, `vnoise`, the field system of
//! fields.rs, the edges of wood and heath that lean across a parcel's side
//! (`place_at`, `fringe`) and the glades (`glade`). The scattered trees and
//! houses and the road ribbons ask the functions here. Change one side of
//! what is repeated and you must change the other.

use bevy::math::Vec2;

use crate::fields;
use crate::replay::Replay;
use crate::rivers::{self, HeightGrid, River, WaterIndex};

/// Terrain vertices per map cell along each axis.
pub const VRES: usize = 8;

// display classes, as in record.DISP
pub const SEA: u8 = 0;
pub const FARM: u8 = 1;
pub const PASTURE: u8 = 2;
pub const WOOD: u8 = 3;
pub const HEATH: u8 = 4;
pub const QUARRY: u8 = 5;
pub const CLAYPIT: u8 = 6;
pub const IRONMINE: u8 = 7;
pub const COALMINE: u8 = 8;
pub const WORKS: u8 = 9;
pub const URBAN: u8 = 10;
#[allow(dead_code)] // only the ground shader draws it
pub const DERELICT: u8 = 11;

/// Run `f`, and when the ISOSTATE_TIMING variable is set, say how long it took,
/// so the cost of preparing the island and of each year's rebuild can be measured.
pub fn timed<T>(what: &str, f: impl FnOnce() -> T) -> T {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ON.get_or_init(|| std::env::var_os("ISOSTATE_TIMING").is_some()) {
        return f();
    }
    let t0 = std::time::Instant::now();
    let out = f();
    eprintln!("timing {what}: {:.1} ms", t0.elapsed().as_secs_f64() * 1000.0);
    out
}

/// The hash every fixed choice hangs on. It lives with the generators, which
/// have no Bevy in them; the ground shader repeats it.
pub use isostate_gen::hash2;

pub fn vnoise(x: f32, y: f32, s: i32) -> f32 {
    let (ix, iy) = (x.floor(), y.floor());
    let (fx, fy) = (x - ix, y - iy);
    let (ix, iy) = (ix as i32, iy as i32);
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);
    let a = hash2(ix, iy, s);
    let b = hash2(ix + 1, iy, s);
    let c = hash2(ix, iy + 1, s);
    let d = hash2(ix + 1, iy + 1, s);
    (a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy) * 2.0 - 1.0
}

/// The warp that turns square cells into organic shapes: the class at a spot is
/// decided by the cell its parcel's point falls in after this warp.
pub fn warp_raw(fx: f32, fy: f32) -> (f32, f32) {
    let wx = fx + 0.40 * vnoise(fx * 0.7, fy * 0.7, 5) + 0.15 * vnoise(fx * 2.3, fy * 2.3, 9);
    let wy = fy + 0.40 * vnoise(fx * 0.7 + 31.7, fy * 0.7 + 17.3, 7) + 0.15 * vnoise(fx * 2.3 + 5.1, fy * 2.3 + 9.7, 13);
    (wx, wy)
}

/// The patch noise. Each parcel takes it at its own point, so only the coarser
/// octaves are kept: neighbouring parcels mostly agree, and land uses come in
/// runs of fields rather than salt and pepper.
fn sub_raw(fx: f32, fy: f32) -> f32 {
    vnoise(fx * 1.7, fy * 1.7, 31) + 0.45 * vnoise(fx * 3.9, fy * 3.9, 37)
}

pub struct Ground {
    pub w: usize,
    pub h: usize,
    /// vertex grid size
    pub gw: usize,
    pub gh: usize,
    /// vertex heights, row-major, gh rows of gw
    pub hgt: Vec<f32>,
    /// distance from each sea cell to land, in cells (0 on land)
    pub sea_dist: Vec<f32>,
    /// quantiles of the patch noise, for turning it into a uniform value
    pub quant: Vec<f32>,
    /// the parcels of the field system, and what does not change about each
    pub land: Land,
    /// the rivers, with their meanders, widths and water levels
    pub rivers: Vec<River>,
    /// the rivers' segments in buckets, for keeping things out of the water
    pub water: WaterIndex,
    /// per vertex: 1 on a river's valley floor, falling to 0 off it
    pub floodplain: Vec<f32>,
}

/// Land height in world units (one unit = one 2 km cell) from the stored elevation byte.
/// Relief is exaggerated about three times, as on a model.
pub fn land_height(e: u8) -> f32 {
    let h = e as f32 / 255.0;
    0.06 + 2.5 * h.powf(1.35)
}

fn chamfer(w: usize, h: usize, seed: &[bool]) -> Vec<f32> {
    let big = 1e9f32;
    let mut d: Vec<f32> = seed.iter().map(|&s| if s { 0.0 } else { big }).collect();
    let r2 = std::f32::consts::SQRT_2;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let mut m = d[i];
            if x > 0 { m = m.min(d[i - 1] + 1.0); }
            if y > 0 {
                m = m.min(d[i - w] + 1.0);
                if x > 0 { m = m.min(d[i - w - 1] + r2); }
                if x + 1 < w { m = m.min(d[i - w + 1] + r2); }
            }
            d[i] = m;
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            let mut m = d[i];
            if x + 1 < w { m = m.min(d[i + 1] + 1.0); }
            if y + 1 < h {
                m = m.min(d[i + w] + 1.0);
                if x + 1 < w { m = m.min(d[i + w + 1] + r2); }
                if x > 0 { m = m.min(d[i + w - 1] + r2); }
            }
            d[i] = m;
        }
    }
    d
}

fn box_blur(a: &mut [f32], w: usize, h: usize, r: usize) {
    let n = (2 * r + 1) as f32;
    let mut tmp = vec![0f32; a.len()];
    for y in 0..h {
        let row = y * w;
        let mut s = 0f32;
        for i in 0..=2 * r {
            s += a[row + (i as isize - r as isize).clamp(0, w as isize - 1) as usize];
        }
        for x in 0..w {
            tmp[row + x] = s / n;
            let add = (x + r + 1).min(w - 1);
            let sub = (x as isize - r as isize).max(0) as usize;
            s += a[row + add] - a[row + sub];
        }
    }
    for x in 0..w {
        let mut s = 0f32;
        for i in 0..=2 * r {
            s += tmp[(i as isize - r as isize).clamp(0, h as isize - 1) as usize * w + x];
        }
        for y in 0..h {
            a[y * w + x] = s / n;
            let add = (y + r + 1).min(h - 1);
            let sub = (y as isize - r as isize).max(0) as usize;
            s += tmp[add * w + x] - tmp[sub * w + x];
        }
    }
}

/// Largest value within r steps (a square window), for the height behind a shore.
fn max_filter(a: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    let mut tmp = vec![0f32; a.len()];
    for y in 0..h {
        for x in 0..w {
            let (x0, x1) = (x.saturating_sub(r), (x + r).min(w - 1));
            tmp[y * w + x] = a[y * w + x0..=y * w + x1].iter().copied().fold(f32::MIN, f32::max);
        }
    }
    let mut out = vec![0f32; a.len()];
    for y in 0..h {
        let (y0, y1) = (y.saturating_sub(r), (y + r).min(h - 1));
        for x in 0..w {
            out[y * w + x] = (y0..=y1).map(|yy| tmp[yy * w + x]).fold(f32::MIN, f32::max);
        }
    }
    out
}

/// Bays and headlands, and beaches or cliffs depending on the ground behind
/// the shore: low land meets the sea on a beach, high land breaks off in a cliff
/// with deep water at its foot.
fn shape_coast(hgt: &mut [f32], gw: usize, gh: usize, res: f32) {
    // a ragged coastline
    for j in 0..gh {
        for i in 0..gw {
            let k = j * gw + i;
            let (fx, fy) = (i as f32 / res, j as f32 / res);
            let near = 1.0 - fields::smoothstep(0.0, 0.16, hgt[k].abs());
            if near > 0.0 {
                let n = vnoise(fx * 2.1, fy * 2.1, 51) + 0.5 * vnoise(fx * 5.3, fy * 5.3, 53);
                hgt[k] += 0.045 * n * near;
            }
        }
    }
    let land: Vec<bool> = hgt.iter().map(|&v| v > 0.0).collect();
    let sea: Vec<bool> = land.iter().map(|&l| !l).collect();
    let from_sea = chamfer(gw, gh, &sea);
    let from_land = chamfer(gw, gh, &land);
    let top = max_filter(hgt, gw, gh, (0.6 * res) as usize);
    // how much of a cliff each stretch of shore is, from the highest ground near it
    for k in 0..gw * gh {
        let cliff = fields::smoothstep(0.17, 0.34, top[k]);
        if land[k] {
            let d = (from_sea[k] - 0.5) / res;
            if d > 0.7 {
                continue;
            }
            // a beach climbs out of the water within a hundred metres or so
            let beach = hgt[k].max((0.42 * d).min(0.05));
            // a cliff stands nearly at full height right at the water
            let bluff = hgt[k].max(top[k] * 0.85 * fields::smoothstep(-0.02, 0.09, d));
            hgt[k] = beach + (bluff - beach) * cliff;
        } else {
            let d = (from_land[k] - 0.5) / res;
            if d > 1.5 {
                continue;
            }
            // shallow water off a beach, a drop off a cliff
            let shelf = hgt[k].min(-0.008 - 0.06 * d);
            let deep = hgt[k].min(-0.06 - 0.35 * d);
            hgt[k] = shelf + (deep - shelf) * cliff;
        }
    }
}

/// Mean of sqrt|vnoise| and of (1 - |vnoise|)^2, so the relief adds as much as
/// it takes away and the cells keep their heights on average.
fn relief_means() -> (f32, f32) {
    let (mut a, mut b, mut n) = (0f64, 0f64, 0f64);
    for j in 0..96 {
        for i in 0..96 {
            let v = vnoise(i as f32 * 0.377 + 0.13, j as f32 * 0.411 + 0.71, 25);
            a += v.abs().sqrt() as f64;
            b += ((1.0 - v.abs()) * (1.0 - v.abs())) as f64;
            n += 1.0;
        }
    }
    ((a / n) as f32, (b / n) as f32)
}

/// Landform below the scale of a cell: soft swells in the lowland, rounded
/// spurs between V-shaped valleys in the hills, sharp crests in the mountains.
fn add_relief(hgt: &mut [f32], gw: usize, gh: usize, res: f32) {
    let (mv, mr) = relief_means();
    for j in 0..gh {
        for i in 0..gw {
            let k = j * gw + i;
            let hb = hgt[k];
            if hb <= 0.0 {
                continue;
            }
            let (fx, fy) = (i as f32 / res, j as f32 / res);
            let hill = fields::smoothstep(0.26, 0.5, hb);
            let mount = fields::smoothstep(0.95, 1.45, hb);
            let calm = fields::smoothstep(0.02, 0.12, hb);
            let low = 0.016 * (vnoise(fx * 1.3, fy * 1.3, 21) + 0.5 * vnoise(fx * 3.1, fy * 3.1, 22));
            let wx = fx + 0.4 * vnoise(fx * 0.55, fy * 0.55, 23);
            let wy = fy + 0.4 * vnoise(fx * 0.55 + 5.2, fy * 0.55 + 1.7, 24);
            let v1 = vnoise(wx * 0.8, wy * 0.8, 25).abs().sqrt() - mv;
            let v2 = vnoise(wx * 1.9 + 3.1, wy * 1.9, 25).abs().sqrt() - mv;
            let v3 = vnoise(wx * 4.3, wy * 4.3 + 7.7, 25).abs().sqrt() - mv;
            let hills = 0.16 * v1 + 0.07 * v2 + 0.025 * v3;
            let r1 = 1.0 - vnoise(wx * 0.6 + 9.0, wy * 0.6, 25).abs();
            let mountains = 0.34 * (r1 * r1 - mr) + 0.12 * v2 + 0.04 * v3;
            let d = low * (1.0 - hill) + hills * hill * (1.0 - mount) + mountains * mount;
            hgt[k] = hb + d * calm;
        }
    }
}

pub fn build_ground(r: &Replay) -> Ground {
    timed("ground", || build_ground_inner(r))
}

fn build_ground_inner(r: &Replay) -> Ground {
    let (w, h) = (r.w, r.h);
    let land: Vec<bool> = r.terrain.iter().map(|&t| t > 0).collect();
    let sea_dist = chamfer(w, h, &land);
    // cell heights: land from elevation, sea floor deepening away from the coast
    let cell_h: Vec<f32> = (0..w * h)
        .map(|c| if land[c] { land_height(r.elev[c]) } else { -(0.12 + 0.09 * sea_dist[c]).min(1.1) })
        .collect();
    let (gw, gh) = (w * VRES + 1, h * VRES + 1);
    let res = VRES as f32;
    let mut hgt = vec![0f32; gw * gh];
    for j in 0..gh {
        let gy = (j as f32 / res - 0.5).clamp(0.0, h as f32 - 1.001);
        let (y0, ty) = (gy.floor() as usize, gy - gy.floor());
        for i in 0..gw {
            let gx = (i as f32 / res - 0.5).clamp(0.0, w as f32 - 1.001);
            let (x0, tx) = (gx.floor() as usize, gx - gx.floor());
            let c = y0 * w + x0;
            let v = (cell_h[c] * (1.0 - tx) + cell_h[c + 1] * tx) * (1.0 - ty)
                + (cell_h[c + w] * (1.0 - tx) + cell_h[c + w + 1] * tx) * ty;
            hgt[j * gw + i] = v;
        }
    }
    box_blur(&mut hgt, gw, gh, VRES / 2);
    box_blur(&mut hgt, gw, gh, VRES / 4);
    shape_coast(&mut hgt, gw, gh, res);
    add_relief(&mut hgt, gw, gh, res);
    // rivers cut their valleys into the finished landform
    let cen = crate::network::centres(r);
    let rivers = rivers::build(r, &cen, &HeightGrid { hgt: &hgt, gw, gh, res });
    let floodplain = rivers::carve(&rivers, &mut hgt, gw, gh, res);
    let water = WaterIndex::new(&rivers, w, h);
    // quantiles of the patch noise
    let mut samples: Vec<f32> = Vec::with_capacity(512 * 512);
    for j in 0..512 {
        for i in 0..512 {
            samples.push(sub_raw(i as f32 / 512.0 * w as f32, j as f32 / 512.0 * h as f32));
        }
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let quant: Vec<f32> = (0..=256).map(|q| samples[(q * (samples.len() - 1)) / 256]).collect();
    let land = timed("land", || build_land(r, &Fields { hgt: &hgt, flood: &floodplain, gw, gh, water: &water }, &quant, &cen));
    Ground { w, h, gw, gh, hgt, sea_dist, quant, land, rivers, water, floodplain }
}

/// A value of a vertex grid at a map position (in cells), between its vertices.
fn grid_at(a: &[f32], gw: usize, gh: usize, x: f32, z: f32) -> f32 {
    let gx = (x * VRES as f32).clamp(0.0, (gw - 1) as f32 - 0.001);
    let gy = (z * VRES as f32).clamp(0.0, (gh - 1) as f32 - 0.001);
    let (i, j) = (gx.floor() as usize, gy.floor() as usize);
    let (tx, ty) = (gx - i as f32, gy - j as f32);
    let k = j * gw + i;
    (a[k] * (1.0 - tx) + a[k + 1] * tx) * (1.0 - ty) + (a[k + gw] * (1.0 - tx) + a[k + gw + 1] * tx) * ty
}

/// Terrain height at a map position (in cells).
pub fn height_at(g: &Ground, x: f32, z: f32) -> f32 {
    grid_at(&g.hgt, g.gw, g.gh, x, z)
}

/// How much a spot lies on a river's valley floor, 0..1.
pub fn flood_at(g: &Ground, x: f32, z: f32) -> f32 {
    grid_at(&g.floodplain, g.gw, g.gh, x, z)
}

// ------------------------------------------------------------------ the lie of the land
//
// The simulation says how much of each 2 km square is arable, pasture, wood and
// heath; where within a district each lies is decided here, once, from the
// shape of the ground. Every parcel of the field system gets three ranks, each
// spread evenly over 0..1 among the parcels of the country round it (a couple
// of squares each way):
//
//   wild   how unfit for the plough: steep, high, exposed, away from the
//          valley floor. The wildest parcels of a district are its wood and heath.
//   moor   among wild land, how much a top: high and exposed is heath, steep
//          valley sides and stream banks are wood.
//   lea    among improved land, how much grass: the floodplain's meadows, the
//          steeper and higher closes are pasture, the flat and dry is arable.
//
// Each rank is coherent noise at several scales (`country`) added to what the
// ground says, so where the ground is expressive it decides, and where it is
// flat the noise does, and the uses meet along lines that wander. Because a
// rank is even over a district, "the wildest 30%" is 30% of the district's
// area whatever its shape: the shares the simulation gives are kept
// (`year_land`). The rules read the height, slope, floodplain and distance to
// water of the finished ground and nothing of how those were made.

/// The fields of the finished ground that the lie of the land is read from.
struct Fields<'a> {
    hgt: &'a [f32],
    flood: &'a [f32],
    gw: usize,
    gh: usize,
    water: &'a WaterIndex,
}

/// The parcels of the field system (fields.rs), eight to a block, and what does
/// not change about each. A parcel's number is its block's place in the grid
/// of blocks that covers the map times eight, plus its number in the block.
pub struct Land {
    /// the first block along each axis, and how many there are
    pub bi0: i32,
    pub bj0: i32,
    pub bw: usize,
    pub bh: usize,
    /// the point that stands for the parcel
    pub at: Vec<Vec2>,
    /// its dry land, in square cells (0 for a parcel in the sea)
    pub area: Vec<f32>,
    /// the cell of its warped point, and its patch noise in 0..255: what
    /// `pick_class` reads
    pub wcell: Vec<u32>,
    pub sub: Vec<f32>,
    /// the four cells round its warped point and their weights, for the
    /// district's mix of open land there
    pub blend: Vec<[(u32, f32); 4]>,
    /// the ranks, each even over 0..1 across a district
    pub wild: Vec<f32>,
    pub moor: Vec<f32>,
    pub lea: Vec<f32>,
    /// how many times it is cut into fields
    pub depth: Vec<u8>,
    /// the year its open fields are enclosed (0 where they never were open)
    pub open_until: Vec<i32>,
    /// how much of its arable is under grass and green crops, and the tone of
    /// its soil, both in 0..255
    pub ley: Vec<u8>,
    pub soil: Vec<u8>,
}

impl Land {
    /// The number of parcel `id` (8 to 15) of block (bi, bj).
    pub fn index(&self, bi: i32, bj: i32, id: u32) -> usize {
        let i = (bi - self.bi0).clamp(0, self.bw as i32 - 1) as usize;
        let j = (bj - self.bj0).clamp(0, self.bh as i32 - 1) as usize;
        (j * self.bw + i) * 8 + (id as usize).saturating_sub(8).min(7)
    }
}

/// Coherent noise at several scales about a spot, the wide scales weighing
/// most, on a warped footing so that nothing in it runs straight, with a
/// ridged layer for the bands and belts that woods and commons lie in.
fn country(x: f32, y: f32, s: i32) -> f32 {
    let wx = x + 0.9 * vnoise(x * 0.31 + 7.3, y * 0.31 + 2.1, s + 1);
    let wy = y + 0.9 * vnoise(x * 0.31 + 1.9, y * 0.31 + 8.4, s + 2);
    let fbm = vnoise(wx * 0.23, wy * 0.23, s) + 0.55 * vnoise(wx * 0.57, wy * 0.57, s + 3) + 0.24 * vnoise(wx * 1.3, wy * 1.3, s + 4) + 0.09 * vnoise(wx * 2.9, wy * 2.9, s + 5);
    let ridge = 1.0 - 2.0 * vnoise(wx * 0.41 + 13.0, wy * 0.41 + 5.0, s + 6).abs();
    fbm + 0.3 * ridge
}

fn build_land(r: &Replay, f: &Fields, quant: &[f32], cen: &[(f32, f32)]) -> Land {
    use rayon::prelude::*;
    let (w, h) = (r.w, r.h);
    // the blocks that cover the map, with room for their jitter and warp
    let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    for p in [Vec2::ZERO, Vec2::new(w as f32, 0.0), Vec2::new(0.0, h as f32), Vec2::new(w as f32, h as f32)] {
        let q = fields::to_block(p);
        lo = lo.min(q);
        hi = hi.max(q);
    }
    let (bi0, bj0) = (lo.x.floor() as i32 - 2, lo.y.floor() as i32 - 2);
    let (bw, bh) = ((hi.x.ceil() as i32 + 2 - bi0) as usize, (hi.y.ceil() as i32 + 2 - bj0) as usize);
    let n = bw * bh * 8;
    let height = |x: f32, z: f32| grid_at(f.hgt, f.gw, f.gh, x, z);
    // heights smoothed over a square and a half each way, to tell tops from hollows
    let mut smooth = f.hgt.to_vec();
    box_blur(&mut smooth, f.gw, f.gh, VRES * 3 / 2);
    let towns: Vec<(Vec2, i32)> = r.header.towns.iter().enumerate().map(|(ti, t)| {
        let (tx, ty) = cen[t.node.min(r.n - 1)];
        (Vec2::new(tx, ty), 1755 + (hash2(ti as i32, 7, 301) * 80.0) as i32)
    }).collect();
    let cell_of = |x: f32, y: f32| (y.floor().clamp(0.0, h as f32 - 1.0) as usize) * w + x.floor().clamp(0.0, w as f32 - 1.0) as usize;

    struct One {
        at: Vec2,
        area: f32,
        wcell: u32,
        sub: f32,
        blend: [(u32, f32); 4],
        raw: [f32; 3],
        depth: u8,
        open_until: i32,
        ley: u8,
        soil: u8,
    }
    let ones: Vec<One> = (0..bw * bh)
        .into_par_iter()
        .flat_map_iter(|b| {
            let bl = fields::block(bi0 + (b % bw) as i32, bj0 + (b / bw) as i32);
            fields::parcels_of(&bl).into_iter().enumerate().map(|(pn, (_, at, quad))| {
                let (x, y) = (at.x, at.y);
                let (wx, wy) = warp_raw(x, y);
                let wcell = cell_of(wx, wy);
                let (bx, by) = (wx - 0.5, wy - 0.5);
                let (tx, ty) = (bx - bx.floor(), by - by.floor());
                let blend = [
                    (cell_of(bx, by) as u32, (1.0 - tx) * (1.0 - ty)),
                    (cell_of(bx + 1.0, by) as u32, tx * (1.0 - ty)),
                    (cell_of(bx, by + 1.0) as u32, (1.0 - tx) * ty),
                    (cell_of(bx + 1.0, by + 1.0) as u32, tx * ty),
                ];
                let mut one = One { at, area: 0.0, wcell: wcell as u32, sub: uniform(quant, sub_raw(x, y)) * 255.0, blend, raw: [0.0; 3], depth: 3, open_until: 0, ley: 77, soil: 128 };
                if x < -0.5 || y < -0.5 || x > w as f32 + 0.5 || y > h as f32 + 0.5 {
                    return one;
                }
                // the ground at the parcel: its height, how much of it is dry, its slope,
                // whether it is a top or a hollow, the valley floor and the nearest water
                let e = 0.08;
                let hs = [height(x, y), height(x - e, y - e), height(x + e, y - e), height(x + e, y + e), height(x - e, y + e)];
                let dry = hs.iter().filter(|&&v| v > 0.02).count() as f32 / 5.0;
                one.area = quad * dry;
                let hm = hs.iter().sum::<f32>() / 5.0;
                let steep = ((height(x + e, y) - height(x - e, y)).hypot(height(x, y + e) - height(x, y - e)) / (2.0 * e)).min(1.6);
                let top = ((hs[0] - grid_at(&smooth, f.gw, f.gh, x, y)) / 0.12).clamp(-1.5, 1.5);
                let flood = grid_at(f.flood, f.gw, f.gh, x, y);
                let stream = (1.0 - fields::smoothstep(0.02, 0.14, f.water.dist(at))) * (1.0 - flood);
                one.raw = [
                    1.5 * steep + 0.8 * hm + 0.25 * top - 0.7 * flood + 0.35 * stream + 0.42 * country(x, y, 201),
                    1.3 * hm + 0.55 * top - 0.8 * steep - 0.5 * stream - 0.6 * flood + 0.4 * country(x, y, 221),
                    1.4 * flood + 0.9 * steep + 0.5 * hm - 0.25 * top + 0.42 * country(x, y, 241),
                ];
                // fields: small crooked closes in old-enclosed country, on the hill margins and
                // round the towns; large straight ones on the late-enclosed plains and the hills
                let id = (b * 8 + pn) as u32;
                let old = fields::old_country(at);
                let hill = fields::smoothstep(0.30, 0.55, hm);
                let margin = fields::smoothstep(0.22, 0.34, hm) * (1.0 - fields::smoothstep(0.42, 0.6, hm));
                let (near, enclosed) = towns.iter().map(|&(t, y)| (t.distance(at), y)).fold((f32::MAX, 0), |a, b| if b.0 < a.0 { b } else { a });
                let size = 4.0 + 2.2 * (old - 0.45) * (1.0 - hill) + margin - 1.3 * hill + 1.3 * (1.0 - fields::smoothstep(0.8, 2.6, near));
                one.depth = (size + fields::hash3(id, 7, 311)).floor().clamp(fields::PARCEL_LEVEL as f32, fields::MAX_DEPTH as f32) as u8;
                // round each town the lowland lies in open fields, in strips, until the
                // parish is enclosed in a year of its own between 1755 and 1835
                if r.terrain[cell_of(x, y)] == 1 && near + 0.3 * vnoise(x * 1.3, y * 1.3, 317) < 1.7 {
                    one.open_until = enclosed;
                }
                // grass country and corn country, and the tone of the soil
                let wet = flood.max(fields::smoothstep(0.0, 1.2, -top) * (1.0 - hill));
                let ley = 0.14 + 0.3 * wet + 0.2 * hill + 0.3 * (0.5 + 0.5 * vnoise(x * 0.19 + 4.4, y * 0.19 + 9.1, 331)) + 0.08 * vnoise(x * 0.7, y * 0.7, 333);
                one.ley = (ley.clamp(0.08, 0.8) * 255.0) as u8;
                let soil = 0.5 + 0.5 * (0.7 * vnoise(x * 0.16 + 2.2, y * 0.16 + 6.3, 335) + 0.3 * vnoise(x * 0.55, y * 0.55, 337));
                one.soil = (soil.clamp(0.0, 1.0) * 255.0) as u8;
                one
            }).collect::<Vec<One>>()
        })
        .collect();
    debug_assert_eq!(ones.len(), n);

    // The ranks: each parcel's place among the dry parcels within two squares and a
    // half of it, by area. First each measure has its trend across that country
    // taken out (a plane fitted to it there): a rank against one's neighbours is
    // only even where the measure has no drift, and on the flank of a mountain,
    // where everything to one side is higher, every parcel would otherwise rank
    // in the middle and the shares would not be kept. So the ground decides
    // within a district, and the districts keep their shares against each other.
    // `moor` and `lea` are ranked among the neighbours about as wild as the
    // parcel itself, so that they say nothing more about how wild it is (if they
    // did, the split of the wild land into wood and heath, and of the rest into
    // arable and pasture, would come out uneven).
    const REACH: f32 = 2.6;
    let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); w * h];
    for (i, o) in ones.iter().enumerate() {
        if o.area > 0.0 {
            buckets[cell_of(o.at.x, o.at.y)].push(i as u32);
        }
    }
    let near = |i: usize, each: &mut dyn FnMut(usize)| {
        let p = ones[i].at;
        let (cx, cy) = (p.x.floor() as isize, p.y.floor() as isize);
        for y in (cy - 3).max(0)..=(cy + 3).min(h as isize - 1) {
            for x in (cx - 3).max(0)..=(cx + 3).min(w as isize - 1) {
                for &j in &buckets[y as usize * w + x as usize] {
                    if ones[j as usize].at.distance_squared(p) < REACH * REACH {
                        each(j as usize);
                    }
                }
            }
        }
    };
    // each measure less the plane that fits it best over the country round the parcel
    let level: Vec<[f32; 3]> = (0..n)
        .into_par_iter()
        .map(|i| {
            if ones[i].area <= 0.0 {
                return [0.0; 3];
            }
            let p = ones[i].at;
            // sums of 1, x, y, xx, xy, yy, and for each measure of v, vx, vy, all by area
            let (mut s, mut t) = ([0.0f64; 6], [[0.0f64; 3]; 3]);
            near(i, &mut |j| {
                let (a, d) = (ones[j].area as f64, ones[j].at - p);
                let (x, y) = (d.x as f64, d.y as f64);
                for (k, v) in [1.0, x, y, x * x, x * y, y * y].into_iter().enumerate() {
                    s[k] += a * v;
                }
                for m in 0..3 {
                    let v = ones[j].raw[m] as f64;
                    t[m][0] += a * v;
                    t[m][1] += a * v * x;
                    t[m][2] += a * v * y;
                }
            });
            // the plane's height at the parcel itself, by Cramer's rule
            let det = s[0] * (s[3] * s[5] - s[4] * s[4]) - s[1] * (s[1] * s[5] - s[4] * s[2]) + s[2] * (s[1] * s[4] - s[3] * s[2]);
            std::array::from_fn(|m| {
                let [v, vx, vy] = t[m];
                let here = if det.abs() > 1e-6 * s[0].powi(3) {
                    (v * (s[3] * s[5] - s[4] * s[4]) - s[1] * (vx * s[5] - s[4] * vy) + s[2] * (vx * s[4] - s[3] * vy)) / det
                } else {
                    v / s[0].max(1e-9)
                };
                ones[i].raw[m] - here as f32
            })
        })
        .collect();
    let wild: Vec<f32> = (0..n)
        .into_par_iter()
        .map(|i| {
            if ones[i].area <= 0.0 {
                return 0.5;
            }
            let (mut below, mut all) = (0.0f32, 0.0f32);
            near(i, &mut |j| {
                all += ones[j].area;
                if level[j][0] < level[i][0] {
                    below += ones[j].area;
                }
            });
            ((below + 0.5 * ones[i].area) / all.max(1e-9)).min(0.9999)
        })
        .collect();
    let within: Vec<(f32, f32)> = (0..n)
        .into_par_iter()
        .map(|i| {
            if ones[i].area <= 0.0 {
                return (0.5, 0.5);
            }
            let (mut below, mut all) = ([0.0f32; 2], 0.0f32);
            near(i, &mut |j| {
                if (wild[j] - wild[i]).abs() <= 0.1 {
                    all += ones[j].area;
                    for t in 0..2 {
                        if level[j][t + 1] < level[i][t + 1] {
                            below[t] += ones[j].area;
                        }
                    }
                }
            });
            let rank = |b: f32| ((b + 0.5 * ones[i].area) / all.max(1e-9)).min(0.9999);
            (rank(below[0]), rank(below[1]))
        })
        .collect();
    Land {
        bi0,
        bj0,
        bw,
        bh,
        at: ones.iter().map(|o| o.at).collect(),
        area: ones.iter().map(|o| o.area).collect(),
        wcell: ones.iter().map(|o| o.wcell).collect(),
        sub: ones.iter().map(|o| o.sub).collect(),
        blend: ones.iter().map(|o| o.blend).collect(),
        wild,
        moor: within.iter().map(|v| v.0).collect(),
        lea: within.iter().map(|v| v.1).collect(),
        depth: ones.iter().map(|o| o.depth).collect(),
        open_until: ones.iter().map(|o| o.open_until).collect(),
        ley: ones.iter().map(|o| o.ley).collect(),
        soil: ones.iter().map(|o| o.soil).collect(),
    }
}

/// The year's land: the class of every parcel, and how wooded each is.
pub struct YearLand {
    /// the display class of each parcel
    pub class: Vec<u8>,
    /// for a wood, 128 at its edge to 255 deep inside; for other open land,
    /// 0 to 127 as it comes nearer to being wood (the country a wood's edge
    /// frays into)
    pub wood: Vec<u8>,
}

/// The shares of arable, pasture, wood and heath in a cell's mix for year k.
pub fn open_shares(r: &Replay, k: usize, c: usize) -> [f32; 4] {
    let off = k * r.n + c;
    let (a, b, s) = (r.mix_a[off], r.mix_b[off], r.mix_s[off] as f32 / 255.0);
    let mut out = [0.0; 4];
    if (FARM..=HEATH).contains(&a) {
        out[(a - FARM) as usize] += s;
    }
    if b != a && (FARM..=HEATH).contains(&b) {
        out[(b - FARM) as usize] += 1.0 - s;
    }
    out
}

/// Decide every parcel's class for year k. Towns, works, diggings and derelict
/// ground lie where the patchwork rule of `pick_class` puts them, cell by cell.
/// The open land that rule leaves is shared out again: the mix of arable,
/// pasture, wood and heath at a parcel is that of the four cells round it,
/// blended, so that it changes smoothly across the country and never at a
/// cell's edge; and the parcel's ranks say which of them it is. The wildest
/// are wood and heath, the tops among those heath; the grassiest of the rest
/// pasture.
pub fn year_land(r: &Replay, g: &Ground, k: usize) -> YearLand {
    let l = &g.land;
    let n = l.at.len();
    let shares: Vec<[f32; 4]> = (0..r.n).map(|c| open_shares(r, k, c)).collect();
    let mut class = vec![SEA; n];
    let mut wood = vec![0u8; n];
    // how far inside a rank's threshold counts as deep in the wood, or far from it
    const FRAY: f32 = 0.22;
    for i in 0..n {
        let c = l.wcell[i] as usize;
        if r.terrain[c] == 0 {
            continue;
        }
        let first = pick_class(r, k, c, l.sub[i]);
        if !(FARM..=HEATH).contains(&first) {
            class[i] = first;
            continue;
        }
        let mut f = [0.0f32; 4];
        for &(cell, wgt) in &l.blend[i] {
            for (t, s) in shares[cell as usize].iter().enumerate() {
                f[t] += wgt * s;
            }
        }
        let (improved, wilds) = (f[0] + f[1], f[2] + f[3]);
        if improved + wilds <= 1e-6 {
            class[i] = first;
            continue;
        }
        let tame = improved / (improved + wilds);
        if l.wild[i] >= tame && wilds > 0.0 {
            let woods = f[2] / wilds;
            if l.moor[i] >= woods {
                class[i] = HEATH;
                wood[i] = if f[2] > 0.0 { (127.0 * (1.0 - (l.moor[i] - woods) / FRAY).clamp(0.0, 1.0)) as u8 } else { 0 };
            } else {
                class[i] = WOOD;
                let from_fields = if tame > 0.02 { (l.wild[i] - tame) / FRAY } else { 1.0 };
                let from_heath = if woods < 0.98 { (woods - l.moor[i]) / FRAY } else { 1.0 };
                wood[i] = 128 + (127.0 * from_fields.min(from_heath).clamp(0.0, 1.0)) as u8;
            }
        } else {
            class[i] = if l.lea[i] >= f[0] / improved.max(1e-6) { PASTURE } else { FARM };
            if f[2] > 0.0 {
                wood[i] = (127.0 * (1.0 - (tame - l.wild[i]) / FRAY).clamp(0.0, 1.0) * (f[2] / wilds)) as u8;
            }
        }
    }
    YearLand { class, wood }
}

/// The number of the parcel at a map position (`Land::index`).
pub fn parcel_index(g: &Ground, fx: f32, fy: f32) -> usize {
    let (bl, pc, _) = fields::parcel(Vec2::new(fx, fy));
    g.land.index(bl.i, bl.j, pc.id)
}

// Wood and heath do not stop at a parcel's straight side. Where one meets
// other open land, the line between them leans to and fro about the side by
// a coherent noise (`fringe`): the wilder of the two (wood before heath,
// heath before the fields) takes the neighbour's ground where the lean is
// its way, and gives up as much of its own where it is not, so neither gains
// area. What a wood or a heath gives up to a field is its margin: rough
// grass and scrub, still the wood's parcel. terrain.wesl does the same per
// pixel.

/// Which way the line between wild land and its neighbour leans at a spot, -1 to 1.
pub fn fringe(x: f32, z: f32) -> f32 {
    0.65 * vnoise(x * 4.3, z * 4.3, 421) + 0.35 * vnoise(x * 10.7, z * 10.7, 423)
}

/// Where a spot lies in the field system, for asking what it is in any year:
/// its parcel, the parcel across the nearest side, and whether the spot is
/// near enough to that side for a wilder neighbour to take it (`lean` 1) or
/// for its own wild parcel to give it up (-1).
#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub parcel: u32,
    pub beyond: u32,
    pub lean: i8,
}

pub fn place_at(g: &Ground, fx: f32, fy: f32) -> Place {
    let (bl, pc, _) = fields::parcel(Vec2::new(fx, fy));
    let parcel = g.land.index(bl.i, bl.j, pc.id) as u32;
    let (side, nb, npc) = fields::across(&bl, pc);
    if side >= fields::FRINGE {
        return Place { parcel, beyond: parcel, lean: 0 };
    }
    let lean = fringe(fx, fy);
    Place { parcel, beyond: g.land.index(nb.i, nb.j, npc.id) as u32, lean: if side < fields::FRINGE * lean { 1 } else if side < -fields::FRINGE * lean { -1 } else { 0 } }
}

impl YearLand {
    /// The class drawn at a place, and whether the place is the margin a
    /// wood or heath leaves to the fields beside it.
    pub fn at(&self, pl: Place) -> (u8, bool) {
        let own = self.class[pl.parcel as usize];
        let open = |c: u8| (FARM..=HEATH).contains(&c);
        if pl.lean == 0 || !open(own) {
            return (own, false);
        }
        let other = self.class[pl.beyond as usize];
        if other == own || !open(other) || (own < WOOD && other < WOOD) {
            return (own, false);
        }
        let wildness = |c: u8| if c == WOOD { 2 } else if c == HEATH { 1 } else { 0 };
        if wildness(other) > wildness(own) {
            (if pl.lean > 0 { other } else { own }, false)
        } else if pl.lean < 0 {
            if other >= WOOD { (other, false) } else { (own, true) }
        } else {
            (own, false)
        }
    }

    /// How wooded a place is, 0 to 1: `wood` of its parcel, or a wood's edge
    /// where a wood has leant across into it.
    pub fn wooded(&self, pl: Place) -> f32 {
        if self.class[pl.parcel as usize] != WOOD && self.at(pl).0 == WOOD { 0.5 } else { self.wood[pl.parcel as usize] as f32 / 255.0 }
    }
}

/// How open a wood is at a spot, 0 under the trees to 1 in a glade or a ride,
/// for a wood of depth `deep` (0 at its edge to 1 far inside). The ground
/// shader paints the glades by the same measure (`glade` in terrain.wesl).
pub fn glade(x: f32, z: f32, deep: f32) -> f32 {
    let qx = x + 0.12 * vnoise(x * 2.1 + 3.3, z * 2.1 + 7.1, 431);
    let qz = z + 0.12 * vnoise(x * 2.1 + 9.2, z * 2.1 + 1.4, 433);
    let n = 0.5 + 0.5 * (0.7 * vnoise(qx * 3.7, qz * 3.7, 435) + 0.3 * vnoise(qx * 9.1, qz * 9.1, 437));
    let open = fields::smoothstep(0.0, 0.07, n - (0.66 + (0.8 - 0.66) * deep));
    let ride = 1.0 - fields::smoothstep(0.012, 0.03, vnoise(qx * 1.3 + 5.5, qz * 1.3 + 2.5, 439).abs());
    open.max(ride * deep)
}

/// How far a spot is from the water of a river (negative in it); about 0.25 or
/// more means nowhere near.
pub fn water_dist(g: &Ground, x: f32, z: f32) -> f32 {
    g.water.dist(Vec2::new(x, z))
}

/// The point that stands for the parcel at (fx, fy): its warped cell and its
/// patch noise decide the land class of the whole parcel.
pub fn class_point(fx: f32, fy: f32) -> (f32, f32) {
    // The same spots are asked about year after year (the house plots along
    // the roads, for one), and `warp` and `sub_uniform` are usually asked about
    // the same spot one after the other, so the answers are kept.
    thread_local! {
        static SEEN: std::cell::RefCell<std::collections::HashMap<u64, (f32, f32)>> = std::cell::RefCell::new(std::collections::HashMap::new());
    }
    let key = ((fx.to_bits() as u64) << 32) | fy.to_bits() as u64;
    if let Some(hit) = SEEN.with(|m| m.borrow().get(&key).copied()) {
        return hit;
    }
    let (_, _, c) = fields::parcel(Vec2::new(fx, fy));
    SEEN.with(|m| {
        let mut m = m.borrow_mut();
        if m.len() > 400_000 {
            m.clear();
        }
        m.insert(key, (c.x, c.y));
    });
    (c.x, c.y)
}

/// The warped position whose cell decides the class at (fx, fy). The whole
/// parcel there shares it, so land classes change only at hedge lines.
pub fn warp(fx: f32, fy: f32) -> (f32, f32) {
    let (cx, cy) = class_point(fx, fy);
    warp_raw(cx, cy)
}

fn uniform(q: &[f32], v: f32) -> f32 {
    let idx = q.partition_point(|&x| x < v);
    (idx as f32 / (q.len() - 1) as f32).min(0.999)
}

/// The patch noise of the parcel at a position, mapped to an even spread over [0, 1).
pub fn sub_uniform(g: &Ground, fx: f32, fy: f32) -> f32 {
    let (cx, cy) = class_point(fx, fy);
    uniform(&g.quant, sub_raw(cx, cy))
}

/// The warped cell and the patch value (0..255) that `pick_class` reads at a spot.
pub fn class_key(g: &Ground, fx: f32, fy: f32) -> (usize, f32) {
    let (px, py) = class_point(fx, fy);
    let (wx, wy) = warp_raw(px, py);
    let c = (wy.floor().clamp(0.0, g.h as f32 - 1.0) as usize) * g.w + wx.floor().clamp(0.0, g.w as f32 - 1.0) as usize;
    (c, uniform(&g.quant, sub_raw(px, py)) * 255.0)
}

/// The cell of the parcel's own point at a spot.
pub fn own_cell(g: &Ground, fx: f32, fy: f32) -> usize {
    let (cx, cy) = class_point(fx, fy);
    (cy.floor().clamp(0.0, g.h as f32 - 1.0) as usize) * g.w + cx.floor().clamp(0.0, g.w as f32 - 1.0) as usize
}

/// The field at a map position, cut as finely as its parcel asks.
pub fn field_at(g: &Ground, fx: f32, fy: f32) -> fields::Field {
    let (bl, pc, _) = fields::parcel(Vec2::new(fx, fy));
    fields::field(&bl, pc, g.land.depth[g.land.index(bl.i, bl.j, pc.id)] as u32)
}

/// Which display class is drawn at a position, given the year's land.
pub fn class_at(g: &Ground, land: &YearLand, fx: f32, fy: f32) -> u8 {
    land.at(place_at(g, fx, fy)).0
}

/// The patchwork rule cell by cell: the class a cell's own mix gives a parcel
/// whose warped point lies in it, the lower class number taking the low values
/// of the patch noise. It is the whole truth for towns, works, diggings and
/// derelict ground. For open land (arable, pasture, wood, heath) it says only
/// that the parcel is open: which of the four it is, `year_land` decides.
pub fn pick_class(r: &Replay, k: usize, c: usize, u255: f32) -> u8 {
    let off = k * r.n + c;
    let (ca, cb, s) = (r.mix_a[off], r.mix_b[off], r.mix_s[off] as f32);
    if ca == cb {
        ca
    } else if ca < cb {
        if u255 < s { ca } else { cb }
    } else if u255 < 255.0 - s {
        cb
    } else {
        ca
    }
}

/// Share of a class in a cell's mix for year k (0..1).
pub fn class_share(r: &Replay, k: usize, c: usize, class: u8) -> f32 {
    let off = k * r.n + c;
    let s = r.mix_s[off] as f32 / 255.0;
    let mut v = 0.0;
    if r.mix_a[off] == class { v += s; }
    if r.mix_b[off] == class && r.mix_b[off] != r.mix_a[off] { v += 1.0 - s; }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A made-up island of 14 squares by 12: sea round the rim, the land in
    /// ridges that run north and south, four squares from crest to crest,
    /// arable and wood in the west half (six to four) and pasture and heath in
    /// the east (four to six).
    fn island() -> Replay {
        let (w, h) = (14usize, 12usize);
        let n = w * h;
        let header: crate::replay::Header =
            serde_json::from_str(r#"{"meta":{"W":14,"H":12,"start_year":1750,"n_years":1},"goods":[],"methods":[],"towns":[],"firms":[],"events":[],"uses":[],"arrays":[]}"#).unwrap();
        let land = |c: usize| {
            let (x, y) = (c % w, c / w);
            x >= 1 && y >= 1 && x < w - 1 && y < h - 1
        };
        let mut r = Replay {
            header,
            w,
            h,
            n,
            ny: 1,
            y0: 1750,
            n_towns: 0,
            elev: (0..n).map(|c| if land(c) { 30 + 20 * [0, 1, 2, 1][(c % w) % 4] } else { 0 }).collect(),
            terrain: (0..n).map(|c| if !land(c) { 0 } else if c % w > 9 { 2 } else { 1 }).collect(),
            river: vec![0; n],
            down: vec![-1; n],
            use_: vec![0; n],
            inten: vec![0; n],
            trans: vec![0; n],
            catch: vec![0; n],
            rent: Vec::new(),
            dist: Vec::new(),
            mix_a: vec![SEA; n],
            mix_b: vec![SEA; n],
            mix_s: vec![255; n],
            town_active: Vec::new(),
            town_pop: Vec::new(),
            town_urban: Vec::new(),
            town_wage: Vec::new(),
            town_price: Vec::new(),
            firm_first: Vec::new(),
            firm_len: Vec::new(),
            firm_off: Vec::new(),
            firm_k: Vec::new(),
            series_names: Vec::new(),
            series: Vec::new(),
            series_len: 0,
            filled: 1,
            owner: Vec::new(),
        };
        for c in (0..n).filter(|&c| land(c)) {
            (r.mix_a[c], r.mix_b[c], r.mix_s[c]) = if c % w < 7 { (FARM, WOOD, 153) } else { (PASTURE, HEATH, 102) };
        }
        r
    }

    /// The dry area of the parcels of each open class, as shares of them all.
    fn drawn(g: &Ground, land: &YearLand) -> [f32; 4] {
        let mut a = [0.0f32; 4];
        for i in 0..g.land.at.len() {
            if (FARM..=HEATH).contains(&land.class[i]) {
                a[(land.class[i] - FARM) as usize] += g.land.area[i];
            }
        }
        let all: f32 = a.iter().sum();
        a.map(|v| v / all)
    }

    #[test]
    fn a_parcel_s_number_finds_it_again() {
        let r = island();
        let g = build_ground(&r);
        for i in 0..400 {
            let (x, y) = (1.0 + 12.0 * hash2(i, 1, 7), 1.0 + 10.0 * hash2(i, 2, 7));
            let at = g.land.at[parcel_index(&g, x, y)];
            assert_eq!((at.x, at.y), class_point(x, y), "the parcel at {x}, {y}");
        }
    }

    #[test]
    fn the_ranks_are_even_over_the_land() {
        let r = island();
        let g = build_ground(&r);
        for (name, rank) in [("wild", &g.land.wild), ("moor", &g.land.moor), ("lea", &g.land.lea)] {
            let mut tenths = [0.0f32; 10];
            for i in 0..rank.len() {
                assert!((0.0..1.0).contains(&rank[i]), "{name} of parcel {i} is {}", rank[i]);
                tenths[(rank[i] * 10.0) as usize] += g.land.area[i];
            }
            let all: f32 = tenths.iter().sum();
            for (t, a) in tenths.iter().enumerate() {
                assert!((a / all - 0.1).abs() < 0.035, "{:.3} of the land has a {name} in tenth {t}", a / all);
            }
        }
    }

    #[test]
    fn the_drawn_land_keeps_the_record_s_shares() {
        let r = island();
        let g = build_ground(&r);
        let got = drawn(&g, &year_land(&r, &g, 0));
        // sixty squares of six arable to four wood, sixty of four pasture to six heath
        for (name, got, want) in [("arable", got[0], 0.3), ("pasture", got[1], 0.2), ("wood", got[2], 0.2), ("heath", got[3], 0.3)] {
            assert!((got - want).abs() < 0.04, "{name} is {got:.3} of the open land, the record says {want}");
        }
    }

    #[test]
    fn the_mix_changes_smoothly_and_not_at_a_cell_s_edge() {
        let r = island();
        let g = build_ground(&r);
        let land = year_land(&r, &g, 0);
        // the two halves meet at x = 7: within a square of the line both mixes are drawn
        let mut seen = [[false; 4]; 2];
        for i in 0..g.land.at.len() {
            let p = g.land.at[i];
            if g.land.area[i] > 0.0 && (p.x - 7.0).abs() < 1.0 && (FARM..=HEATH).contains(&land.class[i]) {
                seen[(p.x >= 7.0) as usize][(land.class[i] - FARM) as usize] = true;
            }
        }
        assert!(seen[0][1] || seen[0][3], "no pasture or heath reaches west of the line");
        assert!(seen[1][0] || seen[1][2], "no arable or wood reaches east of the line");
    }

    #[test]
    fn the_ground_decides_where_each_use_lies() {
        let r = island();
        let g = build_ground(&r);
        let land = year_land(&r, &g, 0);
        let mean_height = |class: u8| {
            let (mut sum, mut area) = (0.0, 0.0);
            for i in 0..g.land.at.len() {
                if land.class[i] == class && g.land.area[i] > 0.0 {
                    sum += height_at(&g, g.land.at[i].x, g.land.at[i].y) * g.land.area[i];
                    area += g.land.area[i];
                }
            }
            sum / area
        };
        // on the ridges, wood lies higher than the arable and heath than the pasture
        assert!(mean_height(WOOD) > mean_height(FARM) + 0.01, "wood at {} and arable at {}", mean_height(WOOD), mean_height(FARM));
        assert!(mean_height(HEATH) > mean_height(PASTURE) + 0.01, "heath at {} and pasture at {}", mean_height(HEATH), mean_height(PASTURE));
    }

    #[test]
    fn towns_stay_where_the_cell_rule_puts_them() {
        let mut r = island();
        let c = 5 * r.w + 4;
        (r.mix_a[c], r.mix_b[c], r.mix_s[c]) = (URBAN, FARM, 140);
        let g = build_ground(&r);
        let land = year_land(&r, &g, 0);
        let mut towns = 0;
        for i in 0..g.land.at.len() {
            let cell = g.land.wcell[i] as usize;
            let first = if r.terrain[cell] == 0 { SEA } else { pick_class(&r, 0, cell, g.land.sub[i]) };
            assert_eq!(land.class[i] == URBAN, first == URBAN, "parcel {i}");
            towns += (first == URBAN) as usize;
        }
        assert!(towns > 0, "the town has no parcel");
    }

    #[test]
    fn a_leaning_edge_gives_as_it_takes() {
        // parcels: 0 a wood, 1 arable, 2 heath, 3 pasture
        let land = YearLand { class: vec![WOOD, FARM, HEATH, PASTURE], wood: vec![200, 60, 0, 0] };
        let at = |parcel: u32, beyond: u32, lean: i8| land.at(Place { parcel, beyond, lean });
        // a wood takes the field's edge where the lean is its way, and leaves a margin of its own where it is not
        assert_eq!(at(1, 0, 1), (WOOD, false));
        assert_eq!(at(1, 0, -1), (FARM, false));
        assert_eq!(at(0, 1, -1), (WOOD, true));
        assert_eq!(at(0, 1, 1), (WOOD, false));
        // wood against heath: the heath takes what the wood gives up
        assert_eq!(at(2, 0, 1), (WOOD, false));
        assert_eq!(at(0, 2, -1), (HEATH, false));
        // heath against pasture, as wood against arable
        assert_eq!(at(3, 2, 1), (HEATH, false));
        assert_eq!(at(2, 3, -1), (HEATH, true));
        // and the hedge between two fields stays where it is
        assert_eq!(at(1, 3, 1), (FARM, false));
        assert_eq!(at(3, 1, -1), (PASTURE, false));
        // a wood leant into a field is a wood's edge there
        assert_eq!(land.wooded(Place { parcel: 1, beyond: 0, lean: 1 }), 0.5);
    }

    #[test]
    fn a_wood_leans_over_its_sides_both_ways() {
        // over the island, the spots a wilder neighbour may take and those a wild
        // parcel may give up are as many, within a tenth
        let r = island();
        let g = build_ground(&r);
        let (mut take, mut give) = (0, 0);
        for i in 0..20000 {
            let pl = place_at(&g, 1.0 + 12.0 * hash2(i, 3, 9), 1.0 + 10.0 * hash2(i, 4, 9));
            assert!(pl.lean == 0 || pl.beyond != pl.parcel);
            take += (pl.lean > 0) as i32;
            give += (pl.lean < 0) as i32;
        }
        assert!(take > 1000 && (take - give).abs() * 10 < take, "{take} to take and {give} to give");
    }
}
