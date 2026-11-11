//! Shared spatial logic: the ground's heights, its rivers, and the patchwork
//! rule that turns each cell's land mix into fields, woods and towns. The
//! scattered trees and houses and the road ribbons ask these functions, and the
//! ground shader (terrain.wesl) repeats `hash2`, `vnoise`, `warp_raw`, `sub_raw`,
//! the field system of fields.rs and `pick_class` exactly on the GPU, so
//! everything agrees about what is where. Change one side and you must change
//! the other.

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
    /// per cell: how many times its parcels are cut into fields
    pub field_depth: Vec<u8>,
    /// per cell: the year its open fields are enclosed (0 where they never were open)
    pub open_until: Vec<i32>,
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
    // fields: large on the hills, smaller in the lowland, small round the towns;
    // and round each town, open fields in strips until the parish is enclosed,
    // in a year of its own between 1755 and 1835
    let base_depth = |t: u8| -> u8 { if t >= 2 { 3 } else { 4 } };
    let mut field_depth: Vec<u8> = r.terrain.iter().map(|&t| base_depth(t)).collect();
    let mut open_until = vec![0i32; w * h];
    for (ti, t) in r.header.towns.iter().enumerate() {
        let enclosed = 1755 + (hash2(ti as i32, 7, 301) * 80.0) as i32;
        let (tx, ty) = cen[t.node.min(r.n - 1)];
        let (cx, cy) = (tx.floor() as isize, ty.floor() as isize);
        for dy in -3..=3isize {
            for dx in -3..=3isize {
                let (x, y) = (cx + dx, cy + dy);
                if x < 0 || y < 0 || x as usize >= w || y as usize >= h {
                    continue;
                }
                let c = y as usize * w + x as usize;
                let d = ((x as f32 + 0.5 - tx).powi(2) + (y as f32 + 0.5 - ty).powi(2)).sqrt();
                let bonus = if d < 2.2 { 1 } else { 0 };
                field_depth[c] = field_depth[c].max((base_depth(r.terrain[c]) + bonus).min(fields::MAX_DEPTH as u8));
                if d < 1.7 && r.terrain[c] == 1 {
                    open_until[c] = open_until[c].max(enclosed);
                }
            }
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
    Ground { w, h, gw, gh, hgt, sea_dist, quant, field_depth, open_until, rivers, water, floodplain }
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

/// How much a spot lies on a river's valley floor, 0..1.
pub fn flood_at(g: &Ground, x: f32, z: f32) -> f32 {
    let gx = (x * VRES as f32).clamp(0.0, (g.gw - 1) as f32 - 0.001);
    let gy = (z * VRES as f32).clamp(0.0, (g.gh - 1) as f32 - 0.001);
    let (i, j) = (gx.floor() as usize, gy.floor() as usize);
    let (tx, ty) = (gx - i as f32, gy - j as f32);
    let k = j * g.gw + i;
    let f = &g.floodplain;
    (f[k] * (1.0 - tx) + f[k + 1] * tx) * (1.0 - ty) + (f[k + g.gw] * (1.0 - tx) + f[k + g.gw + 1] * tx) * ty
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

fn uniform(g: &Ground, v: f32) -> f32 {
    let q = &g.quant;
    let idx = q.partition_point(|&x| x < v);
    (idx as f32 / (q.len() - 1) as f32).min(0.999)
}

/// The patch noise of the parcel at a position, mapped to an even spread over [0, 1).
pub fn sub_uniform(g: &Ground, fx: f32, fy: f32) -> f32 {
    let (cx, cy) = class_point(fx, fy);
    uniform(g, sub_raw(cx, cy))
}

/// The warped cell and the patch value (0..255) that decide the class at a spot.
pub fn class_key(g: &Ground, fx: f32, fy: f32) -> (usize, f32) {
    let (px, py) = class_point(fx, fy);
    let (wx, wy) = warp_raw(px, py);
    let c = (wy.floor().clamp(0.0, g.h as f32 - 1.0) as usize) * g.w + wx.floor().clamp(0.0, g.w as f32 - 1.0) as usize;
    (c, uniform(g, sub_raw(px, py)) * 255.0)
}

/// The cell of the parcel's own point at a spot, which says how its fields are
/// laid out (`field_depth`, `open_until`).
pub fn own_cell(g: &Ground, fx: f32, fy: f32) -> usize {
    let (cx, cy) = class_point(fx, fy);
    (cy.floor().clamp(0.0, g.h as f32 - 1.0) as usize) * g.w + cx.floor().clamp(0.0, g.w as f32 - 1.0) as usize
}

/// The field at a map position, cut as finely as the ground there asks.
pub fn field_at(g: &Ground, fx: f32, fy: f32) -> fields::Field {
    let (bl, pc, centre) = fields::parcel(Vec2::new(fx, fy));
    let c = ((centre.y.floor().clamp(0.0, g.h as f32 - 1.0)) as usize) * g.w + centre.x.floor().clamp(0.0, g.w as f32 - 1.0) as usize;
    fields::field(&bl, pc, g.field_depth[c] as u32)
}

/// Which display class is drawn at a position in year index k, and in which cell.
/// The lower class number always takes the low noise values, so patches run on
/// across cell edges instead of stopping at them.
pub fn class_at(r: &Replay, g: &Ground, k: usize, fx: f32, fy: f32) -> (u8, usize) {
    let (px, py) = class_point(fx, fy);
    let (wx, wy) = warp_raw(px, py);
    let cx = (wx.floor() as isize).clamp(0, r.w as isize - 1) as usize;
    let cy = (wy.floor() as isize).clamp(0, r.h as isize - 1) as usize;
    let c = cy * r.w + cx;
    if r.terrain[c] == 0 {
        return (SEA, c);
    }
    let u = uniform(g, sub_raw(px, py)) * 255.0;
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
