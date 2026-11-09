//! Shared spatial logic: the terrain heights, distances to the coast, and the
//! patchwork rule that turns each cell's land mix into fields, woods and towns.
//! The scattered trees and houses and the road ribbons ask these functions, and
//! the ground shader (terrain.wesl) repeats `hash2`, `vnoise`, `warp`,
//! `sub_raw` and `pick_class` exactly on the GPU, so everything agrees about
//! what is where. Change one side and you must change the other.

use crate::replay::Replay;

/// Terrain vertices per map cell along each axis.
pub const VRES: usize = 4;

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

pub fn hash2(x: i32, y: i32, s: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ (s as u32).wrapping_mul(1_442_695_041);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    h as f32 / 4_294_967_295.0
}

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

/// Map position (in cells) to the warped position whose cell decides the class.
/// The warp turns square cells into organic shapes.
pub fn warp(fx: f32, fy: f32) -> (f32, f32) {
    let wx = fx + 0.40 * vnoise(fx * 0.7, fy * 0.7, 5) + 0.15 * vnoise(fx * 2.3, fy * 2.3, 9);
    let wy = fy + 0.40 * vnoise(fx * 0.7 + 31.7, fy * 0.7 + 17.3, 7) + 0.15 * vnoise(fx * 2.3 + 5.1, fy * 2.3 + 9.7, 13);
    (wx, wy)
}

fn sub_raw(fx: f32, fy: f32) -> f32 {
    vnoise(fx * 1.7, fy * 1.7, 31) + 0.55 * vnoise(fx * 3.9, fy * 3.9, 37) + 0.25 * vnoise(fx * 8.3, fy * 8.3, 43)
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
    let mut hgt = vec![0f32; gw * gh];
    for j in 0..gh {
        let gy = (j as f32 / VRES as f32 - 0.5).clamp(0.0, h as f32 - 1.001);
        let (y0, ty) = (gy.floor() as usize, gy - gy.floor());
        for i in 0..gw {
            let gx = (i as f32 / VRES as f32 - 0.5).clamp(0.0, w as f32 - 1.001);
            let (x0, tx) = (gx.floor() as usize, gx - gx.floor());
            let c = y0 * w + x0;
            let v = (cell_h[c] * (1.0 - tx) + cell_h[c + 1] * tx) * (1.0 - ty)
                + (cell_h[c + w] * (1.0 - tx) + cell_h[c + w + 1] * tx) * ty;
            hgt[j * gw + i] = v;
        }
    }
    box_blur(&mut hgt, gw, gh, 2);
    box_blur(&mut hgt, gw, gh, 1);
    // a little texture in the relief, more in the hills
    for j in 0..gh {
        for i in 0..gw {
            let (fx, fy) = (i as f32 / VRES as f32, j as f32 / VRES as f32);
            let v = hgt[j * gw + i];
            let amp = if v > 0.4 { 0.05 } else if v > 0.0 { 0.012 } else { 0.02 };
            hgt[j * gw + i] = v + amp * (vnoise(fx * 1.3, fy * 1.3, 21) + 0.5 * vnoise(fx * 3.1, fy * 3.1, 22));
        }
    }
    // quantiles of the patch noise
    let mut samples: Vec<f32> = Vec::with_capacity(512 * 512);
    for j in 0..512 {
        for i in 0..512 {
            samples.push(sub_raw(i as f32 / 512.0 * w as f32, j as f32 / 512.0 * h as f32));
        }
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let quant: Vec<f32> = (0..=256).map(|q| samples[(q * (samples.len() - 1)) / 256]).collect();
    Ground { w, h, gw, gh, hgt, sea_dist, quant }
}

/// Terrain height at a map position (in cells).
pub fn height_at(g: &Ground, x: f32, z: f32) -> f32 {
    let gx = (x * VRES as f32).clamp(0.0, (g.gw - 1) as f32 - 0.001);
    let gy = (z * VRES as f32).clamp(0.0, (g.gh - 1) as f32 - 0.001);
    let (i, j) = (gx.floor() as usize, gy.floor() as usize);
    let (tx, ty) = (gx - i as f32, gy - j as f32);
    let k = j * g.gw + i;
    (g.hgt[k] * (1.0 - tx) + g.hgt[k + 1] * tx) * (1.0 - ty) + (g.hgt[k + g.gw] * (1.0 - tx) + g.hgt[k + g.gw + 1] * tx) * ty
}

/// The patch noise at a position, mapped to an even spread over [0, 1).
pub fn sub_uniform(g: &Ground, fx: f32, fy: f32) -> f32 {
    let v = sub_raw(fx, fy);
    let q = &g.quant;
    let idx = q.partition_point(|&x| x < v);
    (idx as f32 / (q.len() - 1) as f32).min(0.999)
}

/// Which display class is drawn at a position in year index k, and in which cell.
/// The lower class number always takes the low noise values, so patches run on
/// across cell edges instead of stopping at them.
pub fn class_at(r: &Replay, g: &Ground, k: usize, fx: f32, fy: f32) -> (u8, usize) {
    let (wx, wy) = warp(fx, fy);
    let cx = (wx.floor() as isize).clamp(0, r.w as isize - 1) as usize;
    let cy = (wy.floor() as isize).clamp(0, r.h as isize - 1) as usize;
    let c = cy * r.w + cx;
    if r.terrain[c] == 0 {
        return (SEA, c);
    }
    let u = sub_uniform(g, fx, fy) * 255.0;
    (pick_class(r, k, c, u), c)
}

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
