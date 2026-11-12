//! Shared spatial logic: the ground's heights, its rivers, and the rules that
//! turn each cell's land mix into fields, woods and towns. The class of every
//! parcel of the field system is decided here, once a year (`year_land`), and
//! handed to the ground shader (terrain.wesl) in a texture with a texel for
//! each parcel, so the two cannot disagree about a parcel. Where wood and
//! heath lie is finer than a parcel: the year's land also has two fields
//! that vary smoothly across the country, how wooded and how much heath
//! (`YearLand::field`), handed over in a texture of their own, and a spot is
//! wood or heath where its field stands above a half. The shader repeats
//! only what is finer still: `hash2`, `vnoise`, the field system of
//! fields.rs, the warp and the fraying of the two fields' edges (`edge_warp`,
//! `edge_fray`, `place_at`), the summer snow line and the glades (`glade`).
//! The scattered trees and houses and the road ribbons ask the functions
//! here. Change one side of what is repeated and you must change the other.

use bevy::math::Vec2;

use crate::fields;
use crate::replay::{self, Replay};
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
#[derive(Default)]
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
    /// how many times it is cut into fields, with the towns that stand when
    /// the record begins; a town founded later cuts the fields round it
    /// further from its year on (`YearLand::depth`)
    pub depth: Vec<u8>,
    /// what `depth` is the whole number of, before the small closes round a
    /// town are counted in (`town_closes`)
    pub size: Vec<f32>,
    /// the year its open fields are enclosed (0 where they never were open)
    pub open_until: Vec<i32>,
    /// how much of its arable is under grass and green crops, and the tone of
    /// its soil, both in 0..255
    pub ley: Vec<u8>,
    pub soil: Vec<u8>,
    /// what has been worked out about the closes of town ground, which does
    /// not change and is asked for again every year (`town_ground`)
    pub closes: TownClosesKnown,
    /// the grid the year's wood and heath are laid on (`YearLand::field`),
    /// `FIELD_RES` texels to a cell: its size, and the parcel under the
    /// middle of each texel (u32::MAX in the sea)
    pub tw: usize,
    pub th: usize,
    pub tex: Vec<u32>,
    /// for each texel, a slow swell that is added to the year's field of wood
    /// and another to its field of heath before the districts' areas are
    /// kept: where it stands high the woods run together in masses some
    /// kilometres across, where low they break up into copses and the
    /// farmland lies open, so that woods and clearings come in every size
    pub swell: Vec<[f32; 2]>,
}

/// Texels of the year's wood and heath fields (`YearLand::field`) to a cell,
/// along each axis.
pub const FIELD_RES: usize = 8;

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
    // the towns that stand when the record begins: the open fields are theirs, and the small
    // closes round them are there from the first year. A town founded later was never the
    // middle of an open-field parish; it gets its closes in its year (`year_land`).
    let towns: Vec<(Vec2, i32)> = r.header.towns.iter().enumerate().filter(|(_, t)| t.founded <= r.y0).map(|(ti, t)| {
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
        size: f32,
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
                let mut one = One { at, area: 0.0, wcell: wcell as u32, sub: uniform(quant, sub_raw(x, y)) * 255.0, blend, raw: [0.0; 3], depth: 3, size: 3.0, open_until: 0, ley: 77, soil: 128 };
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
                one.size = 4.0 + 2.2 * (old - 0.45) * (1.0 - hill) + margin - 1.3 * hill + fields::hash3(id, 7, 311);
                one.depth = field_depth(one.size, near);
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
    // the parcel under each texel of the grid the year's wood and heath are laid on
    let (tw, th) = (w * FIELD_RES, h * FIELD_RES);
    let tex: Vec<u32> = (0..tw * th)
        .into_par_iter()
        .map(|t| {
            let (x, y) = (((t % tw) as f32 + 0.5) / FIELD_RES as f32, ((t / tw) as f32 + 0.5) / FIELD_RES as f32);
            if height(x, y) <= 0.02 {
                return u32::MAX;
            }
            let (bl, pc, _) = fields::parcel(Vec2::new(x, y));
            let b = (bl.j - bj0).clamp(0, bh as i32 - 1) as usize * bw + (bl.i - bi0).clamp(0, bw as i32 - 1) as usize;
            (b * 8 + (pc.id as usize).saturating_sub(8).min(7)) as u32
        })
        .collect();
    let swell: Vec<[f32; 2]> = (0..tw * th)
        .into_par_iter()
        .map(|t| {
            let (x, y) = (((t % tw) as f32 + 0.5) / FIELD_RES as f32, ((t / tw) as f32 + 0.5) / FIELD_RES as f32);
            let slow = |s: i32, wide: f32, close: f32| wide * vnoise(x * 0.3 + 2.9, y * 0.3 + 7.3, s) + close * vnoise(x * 1.07 + 5.1, y * 1.07 + 0.7, s + 1);
            // the woods' is strong enough to outweigh a parcel's own rank: a wood of a few
            // parcels and a forest some kilometres across in the same district
            [slow(477, 1.04, 0.48), slow(481, 0.26, 0.12)]
        })
        .collect();
    Land {
        tw,
        th,
        tex,
        swell,
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
        size: ones.iter().map(|o| o.size).collect(),
        open_until: ones.iter().map(|o| o.open_until).collect(),
        ley: ones.iter().map(|o| o.ley).collect(),
        soil: ones.iter().map(|o| o.soil).collect(),
        closes: Default::default(),
    }
}

/// How far from a town its small closes reach, in cells.
const CLOSES_REACH: f32 = 2.6;

/// How many times a parcel of `size` (`Land::size`) is cut into fields when
/// the nearest town is `near` cells off: fields are small round a town.
fn field_depth(size: f32, near: f32) -> u8 {
    (size + 1.3 * (1.0 - fields::smoothstep(0.8, CLOSES_REACH, near))).floor().clamp(fields::PARCEL_LEVEL as f32, fields::MAX_DEPTH as f32) as u8
}

/// The year's land: the class of every parcel, and where within the parcels
/// the woods and the heaths lie.
pub struct YearLand {
    /// the display class of each parcel
    pub class: Vec<u8>,
    /// which of the town's ground is built up this year (`town_ground`)
    pub town: TownGround,
    /// How wooded the land is, and how much heath, as two fields that vary
    /// smoothly across the country: two bytes to a texel of the land's grid
    /// (`Land::tw` by `Land::th`), read between texels. A spot of pasture,
    /// wood or heath is wood where the first stands above a half, heath
    /// where the second does, and grass where neither (`YearLand::at`), so
    /// the outline of a wood or a heath owes nothing to the parcels' sides.
    pub field: Vec<u8>,
    /// the width of `field` in texels
    pub tw: usize,
    /// How many times each parcel is cut into fields this year: `Land::depth`,
    /// and more round a town founded since the record began. A cut only ever
    /// divides a field that is there, so the hedges that stand keep their
    /// places and new ones are planted between them.
    pub depth: Vec<u8>,
    /// what each parcel of open land is where neither wood nor heath stands
    /// on it: arable or pasture, with its own fields and hedges. For a parcel
    /// of arable or pasture that is its class; for one of wood or heath it is
    /// what the ground the wood or the heath does not reach is farmed as.
    pub under: Vec<u8>,
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
    let mut under = vec![SEA; n];
    // how far each parcel's ranks stand inside what makes a wood of it, and a
    // heath (negative: outside), in units of SURE, -1 to 1
    let mut lean = vec![[-1.0f32; 2]; n];
    // how much grassier each parcel of open land ranks than the share of
    // arable in its mix asks: pasture at or above 0, arable below
    let mut tilt = vec![f32::NAN; n];
    // How far inside a rank's threshold a parcel is surely wood (or heath), and how far
    // outside surely not. Between, it leans one way by degrees and the country round it
    // decides (`land_field`): were every parcel sure, the woods would all be of one grain,
    // that of the ranks, in belts a kilometre wide from one end of the island to the other.
    const SURE: f32 = 0.45;
    for i in 0..n {
        let c = l.wcell[i] as usize;
        if r.terrain[c] == 0 {
            continue;
        }
        let first = pick_class(r, k, c, l.sub[i]);
        under[i] = first;
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
            under[i] = if first == FARM { FARM } else { PASTURE };
            continue;
        }
        // farmed, it would be pasture or arable by how grassy it ranks
        tilt[i] = l.lea[i] - f[0] / improved.max(1e-6);
        under[i] = if tilt[i] >= 0.0 { PASTURE } else { FARM };
        let tame = improved / (improved + wilds);
        {
            let woods = if wilds > 0.0 { f[2] / wilds } else { 0.0 };
            // wild enough; wood rather than heath; heath rather than wood
            let a = if tame <= 0.02 && l.wild[i] >= tame { 1.0 } else { l.wild[i] - tame };
            let b = if f[2] <= 0.0 { -1.0 } else if woods >= 0.98 && l.moor[i] < woods { 1.0 } else { woods - l.moor[i] };
            let c = if f[3] <= 0.0 { -1.0 } else if woods <= 0.02 && l.moor[i] >= woods { 1.0 } else { l.moor[i] - woods };
            lean[i] = [(a.min(b) / SURE).clamp(-1.0, 1.0), (a.min(c) / SURE).clamp(-1.0, 1.0)];
        }
        class[i] = if l.wild[i] >= tame && wilds > 0.0 {
            if l.moor[i] >= f[2] / wilds { HEATH } else { WOOD }
        } else {
            under[i]
        };
    }
    keep_farmed(l, &shares, r.w, &tilt, &mut class, &mut under);
    let field = timed("year's wood and heath", || land_field(l, &class, &lean, &shares, r.w));
    // the small closes round the towns founded since the record began
    let mut depth = l.depth.clone();
    let year = r.y0 + k as i32;
    let late: Vec<usize> = r.header.towns.iter().filter(|t| t.founded > r.y0 && t.founded <= year && t.node < r.n).map(|t| t.node).collect();
    if !late.is_empty() {
        let cen = crate::network::centres(r);
        let blocks = (CLOSES_REACH / fields::BLOCK).ceil() as i32 + 2;
        for node in late {
            let town = Vec2::new(cen[node].0, cen[node].1);
            let q = fields::to_block(town);
            let (bi, bj) = (q.x.floor() as i32, q.y.floor() as i32);
            for j in (bj - blocks).max(l.bj0)..=(bj + blocks).min(l.bj0 + l.bh as i32 - 1) {
                for i in (bi - blocks).max(l.bi0)..=(bi + blocks).min(l.bi0 + l.bw as i32 - 1) {
                    for id in 8..16 {
                        let p = l.index(i, j, id);
                        depth[p] = depth[p].max(field_depth(l.size[p], l.at[p].distance(town)));
                    }
                }
            }
        }
    }
    let town = timed("town ground", || town_ground(r, g, k, &class));
    YearLand { class, town, field, tw: l.tw, under, depth }
}

/// Arable against pasture, district by district. A parcel of open land is
/// farmed as pasture where it ranks grassier than the share of arable in its
/// mix asks (`tilt`), and the mix is blended from the cells round it, so where
/// the record changes sharply from one district to the next the parcels can
/// be many points off it. In a region of `REGION` by `REGION` districts of
/// `KEEP` squares whose farmed parcels are further than `KEEP_WITHIN` from the
/// record's share of arable in its farmed land, the line between arable and
/// pasture is moved in each of its districts, through the parcels' own ranks,
/// by what is over; the grassiest still go to grass first. Every other region
/// is left exactly as it was.
fn keep_farmed(l: &Land, shares: &[[f32; 4]], w: usize, tilt: &[f32], class: &mut [u8], under: &mut [u8]) {
    let h = shares.len() / w.max(1);
    if w == 0 || h == 0 {
        return;
    }
    let (dw, dh) = (w.div_ceil(KEEP), h.div_ceil(KEEP));
    let district = |p: Vec2| ((p.y.max(0.0) as usize).min(h - 1) / KEEP) * dw + (p.x.max(0.0) as usize).min(w - 1) / KEEP;
    let rw = dw.div_ceil(REGION);
    let region = |d: usize| (d / dw / REGION) * rw + (d % dw) / REGION;
    let mut farmed: Vec<Vec<usize>> = vec![Vec::new(); dw * dh];
    for i in 0..tilt.len() {
        if tilt[i].is_finite() && l.area[i] > 0.0 && (class[i] == FARM || class[i] == PASTURE) {
            farmed[district(l.at[i])].push(i);
        }
    }
    // each district's farmed area and its arable as the parcels stand; each region's
    // the same, and the record's arable and pasture in it
    let sums: Vec<[f32; 2]> = farmed.iter().map(|own| [own.iter().map(|&i| l.area[i]).sum(), own.iter().filter(|&&i| tilt[i] < 0.0).map(|&i| l.area[i]).sum()]).collect();
    let mut regions = vec![[0.0f32; 4]; rw * dh.div_ceil(REGION)];
    for (d, s) in sums.iter().enumerate() {
        let g = &mut regions[region(d)];
        (g[0], g[1]) = (g[0] + s[0], g[1] + s[1]);
    }
    for (c, s) in shares.iter().enumerate() {
        let g = &mut regions[region((c / w / KEEP) * dw + (c % w) / KEEP)];
        (g[2], g[3]) = (g[2] + s[0], g[3] + s[1]);
    }
    // where each district draws its line: 0 for the rule as it stands
    let mut line = vec![0.0f32; dw * dh];
    for (d, own) in farmed.iter_mut().enumerate() {
        let ([total, arable], g) = (sums[d], regions[region(d)]);
        if total <= 1e-6 || g[0] <= 1e-6 || g[2] + g[3] <= 1e-6 {
            continue;
        }
        let over = beyond(g[2] / (g[2] + g[3]) - g[1] / g[0]);
        if over == 0.0 {
            continue;
        }
        let goal = (arable / total + over).clamp(0.0, 1.0);
        // the least grassy first, as many as make up the arable wanted
        own.sort_by(|&a, &b| tilt[a].total_cmp(&tilt[b]));
        let (mut sum, mut k) = (0.0, 0);
        while k < own.len() && sum + 0.5 * l.area[own[k]] < goal * total {
            sum += l.area[own[k]];
            k += 1;
        }
        line[d] = match k {
            0 => tilt[own[0]] - 1e-3,
            k if k == own.len() => tilt[own[k - 1]] + 1e-3,
            k => 0.5 * (tilt[own[k - 1]] + tilt[own[k]]),
        };
    }
    for i in 0..tilt.len() {
        let s = if tilt[i].is_finite() { line[district(l.at[i])] } else { 0.0 };
        if s == 0.0 {
            continue;
        }
        under[i] = if tilt[i] >= s { PASTURE } else { FARM };
        if class[i] == FARM || class[i] == PASTURE {
            class[i] = under[i];
        }
    }
}

// Where the woods and the heaths lie. Built ground is what it is, to its
// parcel's side, and arable meets pasture along a hedge. But wood and heath
// meet the farmed land, and each other, along lines of their own. Each
// parcel's ranks say how far it stands inside what makes a wood of it this
// year, or outside (`lean`, likewise for heath): those are laid on a grid
// and smoothed over the open parcels round about, so that a wood's edge runs
// between the middles of a wooded parcel and a farmed one, nearer the one
// that is less surely what it is, round their corners and not along their
// sides. A slow swell is added (`Land::swell`), so that the woods run
// together in masses in one stretch of country and break up into copses in
// the next. Smoothing loses the small woods and fills the small clearings,
// and the swell adds and takes, so each district's fields are then shifted
// until as much of it stands above the line as its parcels of wood (and of
// heath) come to, or, where the parcels of a region of 12 km stand more than
// two points from the record, as brings the region within two points of it
// (`REGION`, `KEEP_WITHIN`): the areas are the simulation's, district by
// district, and only the outlines are the viewer's. What a wood or a heath does not reach
// of its own parcels is farmed (`YearLand::under`), as pasture or arable in
// the district's own proportion, which makes up for what they take of the
// parcels beside them.

/// The side of the districts the areas of wood and heath are kept over, in cells.
const KEEP: usize = 3;

/// How many times the districts' lines are drawn (`land_field`).
const KEEP_ROUNDS: usize = 4;

/// The districts are judged against the record in regions of this many by
/// this many (12 km, as `plan::maybe_areas` measures them): small enough to
/// catch the parcels smearing a change in the record a cell or two across,
/// large enough that the record's own lumpiness, cell by cell, is not
/// pressed onto a picture whose mixes are blended for smoothness.
const REGION: usize = 2;

/// How far a region's parcels may stand from the record's share of its open
/// land in each use before it is brought nearer, as a share of that land:
/// two points. Within it the parcels' own areas are kept, so a region
/// already near the record is drawn exactly as it was.
const KEEP_WITHIN: f32 = 0.02;

/// How far an error stands beyond `KEEP_WITHIN`, with its sign: 0 within it.
fn beyond(err: f32) -> f32 {
    err.signum() * (err.abs() - KEEP_WITHIN).max(0.0)
}

/// How much steeper than they were smoothed the fields of wood and of heath
/// are handed on, about the line that makes a wood or a heath: it keeps the
/// fraying of an edge (`edge_fray`) and the scrub outside it narrow.
const STEEPEN: (f32, f32) = (1.8, 1.5);

fn land_field(l: &Land, class: &[u8], lean: &[[f32; 2]], shares: &[[f32; 4]], w: usize) -> Vec<u8> {
    use rayon::prelude::*;
    let (tw, th) = (l.tw, l.th);
    let n = tw * th;
    let free = |t: usize| l.tex[t] != u32::MAX && (FARM..=HEATH).contains(&class[l.tex[t] as usize]);
    // what the parcel under each texel says, by its weight; then smoothed
    let mut layers = [vec![0f32; n], vec![0f32; n], vec![0f32; n]];
    for t in 0..n {
        if l.tex[t] == u32::MAX {
            continue;
        }
        let i = l.tex[t] as usize;
        let w = if (FARM..=HEATH).contains(&class[i]) { 1.0 } else { 0.0 };
        layers[0][t] = w;
        layers[1][t] = w * lean[i][0];
        layers[2][t] = w * lean[i][1];
    }
    layers.par_iter_mut().for_each(|a| {
        box_blur(a, tw, th, 2);
        box_blur(a, tw, th, 1);
    });
    let [wt, mut fw, mut fh] = layers;
    for t in 0..n {
        if wt[t] > 1e-4 {
            fw[t] = fw[t] / wt[t] + l.swell[t][0];
            fh[t] = fh[t] / wt[t] + l.swell[t][1];
        } else {
            (fw[t], fh[t]) = (-1.0, -1.0);
        }
    }
    // Each district's open texels, and how many of them lie in its parcels of
    // wood and of heath: so many must stand above the wood's line, and of the
    // rest so many above the heath's. But where the record changes sharply
    // from one country to the next the parcels can be many points off it, as
    // their mixes are blended across the cells round them; so in a region
    // whose parcels are further from the record than `KEEP_WITHIN`, each of
    // its districts gives or takes its share of what is over.
    let side = KEEP * FIELD_RES;
    let (dw, dh) = (tw.div_ceil(side), th.div_ceil(side));
    let rows = shares.len() / w.max(1);
    let mut districts: Vec<(Vec<u32>, usize, usize)> = (0..dw * dh)
        .into_par_iter()
        .map(|d| {
            let (x0, y0) = ((d % dw) * side, (d / dw) * side);
            let mut own: Vec<u32> = Vec::with_capacity(side * side);
            for y in y0..(y0 + side).min(th) {
                own.extend((x0..(x0 + side).min(tw)).map(|x| y * tw + x).filter(|&t| free(t)).map(|t| t as u32));
            }
            let count = |c: u8| own.iter().filter(|&&t| class[l.tex[t as usize] as usize] == c).count();
            let (woods, heaths) = (count(WOOD), count(HEATH));
            (own, woods, heaths)
        })
        .collect();
    // each region's open texels, its parcels' wood and heath, and the record's four shares
    let rw = dw.div_ceil(REGION);
    let region = |d: usize| (d / dw / REGION) * rw + (d % dw) / REGION;
    let mut regions = vec![([0.0f32; 3], [0.0f32; 4]); rw * dh.div_ceil(REGION)];
    for (d, (own, woods, heaths)) in districts.iter().enumerate() {
        let g = &mut regions[region(d)].0;
        (g[0], g[1], g[2]) = (g[0] + own.len() as f32, g[1] + *woods as f32, g[2] + *heaths as f32);
    }
    for (c, s) in shares.iter().enumerate() {
        let d = ((c / w.max(1)) / KEEP) * dw + (c % w.max(1)) / KEEP;
        if d < dw * dh && c / w.max(1) < rows {
            for (r, v) in regions[region(d)].1.iter_mut().zip(s) {
                *r += v;
            }
        }
    }
    for (d, (own, woods, heaths)) in districts.iter_mut().enumerate() {
        let (g, rec) = &regions[region(d)];
        let open: f32 = rec.iter().sum();
        if g[0] < 1.0 || open <= 1e-6 {
            continue;
        }
        let n = own.len() as f32;
        let keep = |parcels: &mut usize, now: f32, s: f32| *parcels = (*parcels as f32 + n * beyond(s / open - now / g[0])).clamp(0.0, n).round() as usize;
        keep(woods, g[1], rec[2]);
        keep(heaths, g[2], rec[3]);
    }
    // the lines at a texel, read between the districts' middles so that no edge steps where
    // two districts meet
    let at = |lines: &[(f32, f32)], t: usize| -> (f32, f32) {
        let (u, v) = (((t % tw) as f32 + 0.5) / side as f32 - 0.5, ((t / tw) as f32 + 0.5) / side as f32 - 0.5);
        let (i, j) = (u.floor().clamp(0.0, dw as f32 - 1.0), v.floor().clamp(0.0, dh as f32 - 1.0));
        let (a, b) = ((u - i).clamp(0.0, 1.0), (v - j).clamp(0.0, 1.0));
        let (i, j) = (i as usize, j as usize);
        let (i1, j1) = ((i + 1).min(dw - 1), (j + 1).min(dh - 1));
        let of = |x: usize, y: usize| lines[y * dw + x];
        let mix = |p: (f32, f32), q: (f32, f32), k: f32| (p.0 + (q.0 - p.0) * k, p.1 + (q.1 - p.1) * k);
        mix(mix(of(i, j), of(i1, j), a), mix(of(i, j1), of(i1, j1), a), b)
    };
    // how far a line must rise to leave `want` of the values above it; with none or all
    // wanted, no further than clears the values smoothing or the swell carried across it
    let rise = |v: &mut [f32], want: usize| -> f32 {
        if want == 0 {
            (v.iter().copied().fold(f32::MIN, f32::max) + 0.01).max(0.0)
        } else if want >= v.len() {
            (v.iter().copied().fold(f32::MAX, f32::min) - 0.01).min(0.0)
        } else {
            let (above, nth, _) = v.select_nth_unstable_by(want, |a, b| b.total_cmp(a));
            0.5 * (above.iter().copied().fold(f32::MAX, f32::min) + *nth)
        }
    };
    // Reading the lines between the districts' middles lets a district's neighbours move
    // its edges, so the lines are drawn a few times over, each district making up what the
    // round before left it short of or over.
    let mut lines = vec![(0.0f32, 0.0f32); dw * dh];
    for _ in 0..KEEP_ROUNDS {
        let drawn: Vec<Option<(f32, f32)>> = districts
            .par_iter()
            .enumerate()
            .map(|(d, (own, woods, heaths))| {
                if own.len() < 8 {
                    return None;
                }
                let left: Vec<(f32, f32)> = own
                    .iter()
                    .map(|&t| {
                        let line = at(&lines, t as usize);
                        (fw[t as usize] - line.0, fh[t as usize] - line.1)
                    })
                    .collect();
                let mut v: Vec<f32> = left.iter().map(|x| x.0).collect();
                let wood = rise(&mut v, *woods);
                let mut v: Vec<f32> = left.iter().filter(|x| x.0 <= wood).map(|x| x.1).collect();
                let want = (*heaths).min(v.len());
                let heath = if v.is_empty() { 0.0 } else { rise(&mut v, want) };
                Some((lines[d].0 + wood, lines[d].1 + heath))
            })
            .collect();
        // a district with no such land takes the lines of those round it, or the parcels' own
        lines = (0..dw * dh)
            .map(|d| {
                if let Some(v) = drawn[d] {
                    return v;
                }
                let (dx, dy) = (d % dw, d / dw);
                let (mut sum, mut k) = ((0.0, 0.0), 0.0);
                for y in dy.saturating_sub(1)..=(dy + 1).min(dh - 1) {
                    for x in dx.saturating_sub(1)..=(dx + 1).min(dw - 1) {
                        if let Some(v) = drawn[y * dw + x] {
                            sum = (sum.0 + v.0, sum.1 + v.1);
                            k += 1.0;
                        }
                    }
                }
                if k > 0.0 { (sum.0 / k, sum.1 / k) } else { (0.0, 0.0) }
            })
            .collect();
    }
    // the fields less their lines
    (0..n)
        .into_par_iter()
        .flat_map_iter(|t| {
            let line = at(&lines, t);
            let byte = |v: f32| (127.5 + 127.5 * v.clamp(-1.0, 1.0)).round() as u8;
            [byte((fw[t] - line.0) * STEEPEN.0), byte((fh[t] - line.1) * STEEPEN.1)]
        })
        .collect()
}

// ------------------------------------------------------------------ town ground (task A: towns from kept ground)
//
// Under the game's rules a town keeps the ground it builds on, and the record
// says of every cell of it the year it joined, its residents and how much of
// it is under works (docs/rules/towns.md). A cell is 2 km across and holds
// 300 people the year it joins and some 1,400 at its fullest: most of a town
// cell is not houses. So the parcels of town ground are cut three times more,
// into eight closes each (some eighty to a cell), and a close is either built
// up (streets, houses, yards, paving) or open (the gardens, orchards, paddocks
// and small closes of a town's edge, which the ground shader draws).
//
// Which closes are built up follows an order that is fixed for the cell, so
// that built-up ground never moves and only grows: along the roads that ran
// by the cell in the year it joined its town, nearest the town's market
// place first, and then outward from those roads (`town_order`). So a town
// is built up along its roads out, the newer ground carrying on where the
// older left off, and what lies between the roads stays open longest. A
// cell no road ran near fills from its side nearest the market place, and a
// town's first cell in rings round its market place. As many closes are
// built up as the cell's houses need ground for (`closes_built`; the works
// stand in yards of their own, which the houses keep out of).
//
// A run under the port's rules has none of this: all its town ground is
// built up, and is drawn as it always was.

/// People to one drawn house: twelve. The number is the map's, not the
/// period's: everything that stands on the map is drawn about five times life
/// size, a town plot is 100 m wide, and the streets of a cell that is built
/// up whole have plots for about 120 houses. At twelve people to a house
/// the fullest ground of 1900 (1,430 people) is 119 houses, which with their
/// yards and streets take the whole of the cell, and the newest ground (300
/// people) is 25 houses along the way out of town. (Eight, the unit the
/// record keeps residents in, asks 179 houses of the fullest ground, and
/// four fifths of the houses asked for were drawn; at ten it was nine tenths.)
pub const PEOPLE_TO_A_HOUSE: f32 = 12.0;
/// The ground a drawn house takes with its yard, the streets and the room
/// left about it, in square cells, where the plots are as few as a cell's
/// streets leave them: what the closes a cell's houses reach are counted out by.
pub const HOUSE_GROUND: f32 = 0.0084;
/// The same where houses stand close: what the closes that are built up, and
/// drawn as the paving, yards and gardens of a town, are counted out by.
/// Houses beyond them stand in the closes of the town's edge.
pub const BUILT_GROUND: f32 = 0.0055;
/// Houses outlast a fall in their cell's residents of this share: a town that
/// has lost a quarter of its people still stands whole.
pub const HOUSES_KEEP: f32 = 0.25;
/// How many closes the edge of a cell's built-up ground thins out over: a
/// house plot's turn comes up to this many closes after its close's place in
/// the cell's order (`scatter/towns/kept.rs`), and a plot is built on only
/// once its turn is under the count of built-up closes. So the order the
/// plots of a cell are built in never changes as its ground grows, and half
/// of this many closes are built up beyond what the houses need, for the
/// plots at the edge whose turn has not come.
pub const EDGE_CLOSES: f32 = 5.0;
/// The cuts that make a close of town ground: three more than a parcel.
pub const CLOSE_LEVEL: u32 = fields::PARCEL_LEVEL + 3;

/// One of the eight closes a parcel is cut into: its middle on the map, and
/// its area in square cells (nothing for one that lies in the water).
#[derive(Clone, Copy, Debug, Default)]
pub struct Close {
    pub at: Vec2,
    pub area: f32,
}

/// The closes of a cell's town ground in the order they are built up: the
/// parcel and the close's number in it, and its area.
pub type Order = std::sync::Arc<Vec<(u32, u8, f32)>>;

/// How finely the closes are kept for looking up (`CloseLookup`): this many
/// spots along each side of a cell, 42 m apart.
const CLOSE_GRID: usize = 48;

#[derive(Default)]
pub struct TownClosesKnown {
    /// the eight closes of a parcel
    closes: std::sync::Mutex<std::collections::HashMap<u32, [Close; 8]>>,
    /// the order of a cell's closes, by the cell, the year it joined and how many parcels it has
    orders: std::sync::Mutex<std::collections::HashMap<(u32, u32, u32), Order>>,
    /// the parcel and the close at each of `CLOSE_GRID` by `CLOSE_GRID` spots of a cell of the map, by the cell
    grids: std::sync::Mutex<std::collections::HashMap<u32, std::sync::Arc<Vec<(u32, u8)>>>>,
    /// how many years of the record's roads have been gone through, and the first year each road ran (`road_years`)
    roads: std::sync::Mutex<(usize, std::collections::HashMap<(u32, u32), u32>)>,
}

/// The closes of the map's cells for looking up, as far as they have been
/// asked for: taken out once for a year's work, so that a look-up is a step
/// into a table (`close_at` works it out from the field system each time).
pub struct CloseLookup<'a> {
    g: &'a Ground,
    cells: std::collections::HashMap<u32, std::sync::Arc<Vec<(u32, u8)>>>,
    /// the cell asked for last, which is most often the cell asked for next
    last: Option<(u32, std::sync::Arc<Vec<(u32, u8)>>)>,
}

impl<'a> CloseLookup<'a> {
    pub fn new(g: &'a Ground) -> CloseLookup<'a> {
        CloseLookup { g, cells: Default::default(), last: None }
    }

    /// The parcel at a spot and which of its eight closes the spot lies in, to the nearest 42 m.
    pub fn at(&mut self, x: f32, z: f32) -> (u32, u8) {
        let g = self.g;
        let (cx, cy) = ((x.max(0.0) as usize).min(g.w - 1), (z.max(0.0) as usize).min(g.h - 1));
        let c = (cy * g.w + cx) as u32;
        if self.last.as_ref().map(|l| l.0) != Some(c) {
            self.last = Some((c, self.cells.entry(c).or_insert_with(|| close_grid(g, c)).clone()));
        }
        let grid = &self.last.as_ref().unwrap().1;
        let n = CLOSE_GRID as f32;
        let (i, j) = ((((x - cx as f32) * n).max(0.0) as usize).min(CLOSE_GRID - 1), (((z - cy as f32) * n).max(0.0) as usize).min(CLOSE_GRID - 1));
        grid[j * CLOSE_GRID + i]
    }
}

/// The closes at the spots of one cell of the map.
fn close_grid(g: &Ground, c: u32) -> std::sync::Arc<Vec<(u32, u8)>> {
    if let Some(known) = g.land.closes.grids.lock().unwrap().get(&c) {
        return known.clone();
    }
    let (cx, cy) = ((c as usize % g.w) as f32, (c as usize / g.w) as f32);
    let n = CLOSE_GRID as f32;
    let grid: Vec<(u32, u8)> = (0..CLOSE_GRID * CLOSE_GRID).map(|i| close_at(g, cx + ((i % CLOSE_GRID) as f32 + 0.5) / n, cy + ((i / CLOSE_GRID) as f32 + 0.5) / n)).collect();
    let grid = std::sync::Arc::new(grid);
    g.land.closes.grids.lock().unwrap().insert(c, grid.clone());
    grid
}

/// A cell of town ground this year, in a run that keeps it.
#[derive(Clone, Debug, Default)]
pub struct TownCell {
    /// its closes in the order they are built up (`town_order`)
    pub order: Order,
    /// how many closes it has dry ground for; how many of them (the first of
    /// its order) are built up; and how many its houses reach, the last of
    /// them standing among gardens and paddocks (`closes_built`)
    pub closes: u16,
    pub built: u16,
    pub reach: u16,
    /// the houses that stand on it (`town_houses`)
    pub houses: u16,
    /// the cell its ground is built up from, if it is not a town's first
    pub parent: Option<u32>,
}

/// The town ground of a year.
#[derive(Default)]
pub struct TownGround {
    /// per parcel: which of its eight closes are built up, a bit for each;
    /// every bit on town ground in a run that does not keep it
    pub built: Vec<u8>,
    /// per close (eight to a parcel): its place in its cell's order, on kept town ground
    pub rank: Vec<u16>,
    /// the cells of kept town ground
    pub cells: std::collections::HashMap<u32, TownCell>,
}

impl TownGround {
    /// Whether close `sub` of a parcel is built up.
    pub fn built_up(&self, parcel: u32, sub: u8) -> bool {
        self.built.get(parcel as usize).is_some_and(|m| m >> sub & 1 != 0)
    }

    /// The place of a close in its cell's order, if it is kept town ground.
    pub fn rank_of(&self, parcel: u32, sub: u8) -> Option<u16> {
        self.rank.get(parcel as usize * 8 + sub as usize).copied().filter(|&r| r != u16::MAX)
    }
}

/// The houses that stand on a cell of kept town ground in year `k`: one to
/// every `PEOPLE_TO_A_HOUSE` residents, and those the cell had at its fullest
/// since it joined the town for as long as it has not lost more than
/// `HOUSES_KEEP` of the people they were built for.
pub fn town_houses(r: &Replay, k: usize, c: usize) -> usize {
    if r.joined(k, c).is_none() {
        return 0;
    }
    let of = |y: usize| r.residents(y, c) / PEOPLE_TO_A_HOUSE;
    let now = of(k);
    let mut peak = now;
    for y in (0..k).rev() {
        if r.built[y * r.n + c] == 0 {
            break;
        }
        peak = peak.max(of(y));
    }
    now.max(peak.min(now / (1.0 - HOUSES_KEEP))).round() as usize
}

/// The most of cell `c` that has been under works in any year since it joined
/// its town, up to year `k`: ground that has been a yard stays built-up ground.
pub fn town_works(r: &Replay, k: usize, c: usize) -> f32 {
    let mut most = r.works_share(k, c);
    for y in (0..k).rev() {
        if r.built[y * r.n + c] == 0 {
            break;
        }
        most = most.max(r.works_share(y, c));
    }
    most
}

/// Whether a road ran between two neighbouring cells in year `k` (`network::road_segments`).
pub fn road_between(r: &Replay, k: usize, a: usize, b: usize) -> bool {
    let tr = &r.trans[k * r.n..(k + 1) * r.n];
    if (tr[a] & 3).min(tr[b] & 3) < 2 {
        return false;
    }
    let (ax, ay, bx, by) = (a % r.w, a / r.w, b % r.w, b / r.w);
    // a diagonal is left out where a pair of straight links already joins the two
    ax == bx || ay == by || !(tr[ay * r.w + bx] & 2 != 0 || tr[by * r.w + ax] & 2 != 0)
}

/// The cell a cell of kept town ground is built up from: of its neighbours
/// that were town ground before it, one a road ran to in the year it joined,
/// or else the oldest, a neighbour across a side before one across a corner.
/// A town's first cell has none; nor has one with no older neighbour, unless
/// one joined in the same year.
pub fn town_parent(r: &Replay, k: usize, c: usize) -> Option<usize> {
    let own = r.built[k * r.n + c];
    if own == 0 || r.header.towns.iter().any(|t| t.node == c) {
        return None;
    }
    let (x, y) = ((c % r.w) as isize, (c / r.w) as isize);
    let joined = ((replay::BUILT_YEAR0 + own as i32 - r.y0).max(0) as usize).min(k);
    let mut best: Option<((bool, bool, u8, bool, usize), usize)> = None;
    for (dx, dy) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
        let (nx, ny) = (x + dx, y + dy);
        if nx < 0 || ny < 0 || nx as usize >= r.w || ny as usize >= r.h {
            continue;
        }
        let m = ny as usize * r.w + nx as usize;
        let theirs = r.built[k * r.n + m];
        // of two that joined in the same year, the one with the lower number came first
        if theirs == 0 || theirs > own || (theirs == own && m > c) {
            continue;
        }
        let key = (theirs == own, !road_between(r, joined, c, m), theirs, dx != 0 && dy != 0, m);
        if best.map_or(true, |(b, _)| key < b) {
            best = Some((key, m));
        }
    }
    best.map(|(_, m)| m)
}

/// The piece of a block reached by taking, cut after cut, the half that
/// `path` names (its highest bit first; `fields::parcels_of` does the same for a parcel).
fn piece(bl: &fields::Block, path: u32, levels: u32) -> fields::Cut {
    let mut b = *bl;
    let mut c = fields::Cut { lo: Vec2::ZERO, hi: Vec2::ONE, id: 1 };
    for lvl in 0..levels {
        let upper = (path >> (levels - 1 - lvl)) & 1 == 1;
        b.uv = if upper { c.hi - Vec2::splat(1e-5) } else { c.lo };
        c = fields::cut(&b, c, lvl, lvl + 1);
    }
    c
}

/// The eight closes of a parcel.
pub fn closes_of(g: &Ground, parcel: u32) -> [Close; 8] {
    if let Some(&known) = g.land.closes.closes.lock().unwrap().get(&parcel) {
        return known;
    }
    let l = &g.land;
    let b = parcel as usize / 8;
    let bl = fields::block(l.bi0 + (b % l.bw) as i32, l.bj0 + (b / l.bw) as i32);
    let out: [Close; 8] = std::array::from_fn(|sub| {
        let c = piece(&bl, ((parcel % 8) << 3) | sub as u32, CLOSE_LEVEL);
        let corner = |u: f32, v: f32| fields::bilinear(bl.a, bl.b, bl.c, bl.d, Vec2::new(u, v));
        let q = [corner(c.lo.x, c.lo.y), corner(c.hi.x, c.lo.y), corner(c.hi.x, c.hi.y), corner(c.lo.x, c.hi.y)];
        let area = 0.5 * (0..4).map(|i| q[i].x * q[(i + 1) % 4].y - q[i].y * q[(i + 1) % 4].x).sum::<f32>().abs() * fields::BLOCK * fields::BLOCK;
        let at = fields::from_block(fields::bilinear(bl.a, bl.b, bl.c, bl.d, (c.lo + c.hi) * 0.5));
        let dry = height_at(g, at.x, at.y) >= 0.05 && water_dist(g, at.x, at.y) >= 0.03;
        Close { at, area: if dry { area } else { 0.0 } }
    });
    g.land.closes.closes.lock().unwrap().insert(parcel, out);
    out
}

/// The parcel at a spot and which of its eight closes the spot lies in.
pub fn close_at(g: &Ground, x: f32, z: f32) -> (u32, u8) {
    let (bl, pc, _) = fields::parcel(Vec2::new(x, z));
    let close = fields::cut(&bl, pc, fields::PARCEL_LEVEL, CLOSE_LEVEL);
    (g.land.index(bl.i, bl.j, pc.id) as u32, (close.id & 7) as u8)
}

/// How much further a spot counts from the market place for each cell it lies off the road.
const OFF_THE_ROAD: f32 = 3.0;
/// The same in the cell of a town's market place: there the way across the
/// ground counts for hardly more than the way along a road, so the closes
/// come in rings round the market place and the old middle of a town is a
/// knot of streets, built up behind its roads as far as along them. (At
/// 1.6 the town of 1750 was houses strung along its roads to the edge of
/// the cell with nothing behind them.)
const OFF_THE_ROAD_AT_MARKET: f32 = 1.08;

/// The order in which the closes of cell `c`'s town ground (its `parcels`)
/// are built up. Each close has a way to the market place of the town the
/// cell joined: across the ground to the nearest road that ran by in the
/// year the cell joined, and along it; ground off the road counts
/// `OFF_THE_ROAD` times. The closes with the shortest way come first: those
/// along the roads, from the town's end of them outward. With no road by,
/// it is the distance to the market place alone.
fn town_order(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, c: usize, parcels: &[u32]) -> Order {
    let joined = ((replay::BUILT_YEAR0 + r.built[k * r.n + c] as i32 - r.y0).max(0) as usize).min(k);
    let key = (c as u32, joined as u32, parcels.len() as u32);
    if let Some(known) = g.land.closes.orders.lock().unwrap().get(&key) {
        return known.clone();
    }
    let own = Vec2::new(cen[c].0, cen[c].1);
    // the market place the cell's town grew from: the nearest that stood when the cell joined
    let stood = |t: usize| r.town_active.get(joined * r.n_towns + t).is_some_and(|&a| a > 0);
    let nearest = |all: bool| {
        r.header.towns.iter().enumerate().filter(|&(t, _)| all || stood(t)).map(|(_, t)| Vec2::new(cen[t.node].0, cen[t.node].1)).min_by(|a, b| a.distance_squared(own).total_cmp(&b.distance_squared(own)))
    };
    let market = nearest(false).or_else(|| nearest(true)).unwrap_or(own);
    // the roads of that year within two cells
    let (x, y) = ((c % r.w) as isize, (c / r.w) as isize);
    let mut roads: Vec<(Vec2, Vec2)> = Vec::new();
    for cy in (y - 2).max(0)..=(y + 2).min(r.h as isize - 1) {
        for cx in (x - 2).max(0)..=(x + 2).min(r.w as isize - 1) {
            let a = cy as usize * r.w + cx as usize;
            for (dx, dy) in [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)] {
                let (nx, ny) = (cx + dx, cy + dy);
                if nx < 0 || nx >= r.w as isize || ny >= r.h as isize {
                    continue;
                }
                let b = ny as usize * r.w + nx as usize;
                if road_between(r, joined, a, b) {
                    roads.push((Vec2::new(cen[a].0, cen[a].1), Vec2::new(cen[b].0, cen[b].1)));
                }
            }
        }
    }
    // a town's first cell is built up closer round its market place than the ground beyond
    // is along its roads: the old middle is a knot of streets, not a ribbon
    let off = if r.header.towns.iter().any(|t| t.node == c) { OFF_THE_ROAD_AT_MARKET } else { OFF_THE_ROAD };
    let score = |q: Vec2| {
        let by_road = roads.iter().map(|&(a, b)| {
            let t = ((q - a).dot(b - a) / a.distance_squared(b).max(1e-9)).clamp(0.0, 1.0);
            let foot = a + (b - a) * t;
            off * q.distance(foot) + foot.distance(market)
        });
        by_road.fold(off * q.distance(market), f32::min)
    };
    let mut all: Vec<(f32, u32, u8, f32)> = Vec::with_capacity(parcels.len() * 8);
    for &i in parcels {
        for (sub, close) in closes_of(g, i).iter().enumerate() {
            // a close in the water is never built up: it comes last
            all.push((if close.area > 0.0 { score(close.at) } else { f32::MAX }, i, sub as u8, close.area));
        }
    }
    all.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let order: Order = std::sync::Arc::new(all.into_iter().map(|(_, i, sub, area)| (i, sub, area)).collect());
    g.land.closes.orders.lock().unwrap().insert(key, order.clone());
    order
}

/// How many of a cell's closes, the first of its `order`, `houses` houses
/// take: the closes that are built up, at `BUILT_GROUND` a house, and the
/// closes the houses reach, at `HOUSE_GROUND` a house with the edge where
/// the plots' turns come one by one (`EDGE_CLOSES`). Between the two a house
/// stands among the gardens and paddocks of the town's edge. Closes in the
/// water are never counted.
///
/// The works are not counted for. The record says how much of a cell is
/// under works; but a works is drawn in a yard of its own size where
/// `scatter.rs` stands it and the houses keep out of the yards as drawn,
/// and with that share of the cell's closes built up as well the middle of
/// a town, which the record has wholly under works, was a pale plain with a
/// few yards on it.
pub fn closes_built(order: &[(u32, u8, f32)], houses: usize) -> (usize, usize) {
    let count = |ground: f32, edge: usize| {
        let want = houses as f32 * ground;
        let (mut have, mut built) = (0.0, 0usize);
        while built < order.len() && order[built].2 > 0.0 && (have < want || (built == 0 && houses > 0)) {
            have += order[built].2;
            built += 1;
        }
        let mut edge = if houses > 0 { edge } else { 0 };
        while edge > 0 && built < order.len() && order[built].2 > 0.0 {
            built += 1;
            edge -= 1;
        }
        built
    };
    (count(BUILT_GROUND, 0), count(HOUSE_GROUND, (EDGE_CLOSES * 0.5).round() as usize))
}

/// The first year (counted from the start) each pair of neighbouring cells
/// had a road between them, among the years up to `k`: by the lower cell and
/// the higher. A road the map draws no more (a diagonal is left out once two
/// straight roads join its ends) is still a street of the town it ran through.
pub fn road_years(r: &Replay, g: &Ground, k: usize) -> std::collections::HashMap<(u32, u32), u32> {
    let mut known = g.land.closes.roads.lock().unwrap();
    while known.0 <= k {
        let y = known.0;
        for (a, b, _) in crate::network::road_segments(r, y) {
            known.1.entry((a.min(b) as u32, a.max(b) as u32)).or_insert(y as u32);
        }
        known.0 += 1;
    }
    known.1.iter().filter(|&(_, &y)| y as usize <= k).map(|(&l, &y)| (l, y)).collect()
}

/// Which of the year's town ground is built up. `class` is the year's class of every parcel.
pub fn town_ground(r: &Replay, g: &Ground, k: usize, class: &[u8]) -> TownGround {
    let l = &g.land;
    let mut town = TownGround { built: vec![0; class.len()], ..Default::default() };
    if !r.kept() {
        for (i, &cls) in class.iter().enumerate() {
            if cls == URBAN {
                town.built[i] = 255;
            }
        }
        return town;
    }
    // the parcels of each cell of town ground
    let mut by_cell: std::collections::BTreeMap<u32, Vec<u32>> = Default::default();
    for (i, &cls) in class.iter().enumerate() {
        if cls == URBAN {
            if r.built[k * r.n + l.wcell[i] as usize] > 0 {
                by_cell.entry(l.wcell[i]).or_default().push(i as u32);
            } else {
                // a patch of town in a cell the record does not call town ground: as before
                town.built[i] = 255;
            }
        }
    }
    let cen = crate::network::centres(r);
    town.rank = vec![u16::MAX; class.len() * 8];
    for (&c, parcels) in &by_cell {
        let c = c as usize;
        let parent = town_parent(r, k, c);
        let order = town_order(r, g, &cen, k, c, parcels);
        let houses = town_houses(r, k, c);
        let (built, reach) = closes_built(&order, houses);
        for (n, &(i, sub, _)) in order.iter().enumerate() {
            town.rank[i as usize * 8 + sub as usize] = n as u16;
            if n < built {
                town.built[i as usize] |= 1 << sub;
            }
        }
        let closes = order.iter().filter(|o| o.2 > 0.0).count();
        town.cells.insert(c as u32, TownCell { order: order.clone(), closes: closes as u16, built: built as u16, reach: reach as u16, houses: houses.min(u16::MAX as usize) as u16, parent: parent.map(|p| p as u32) });
    }
    town
}

/// The number of the parcel at a map position (`Land::index`).
pub fn parcel_index(g: &Ground, fx: f32, fy: f32) -> usize {
    let (bl, pc, _) = fields::parcel(Vec2::new(fx, fy));
    g.land.index(bl.i, bl.j, pc.id)
}

// Wood and heath meet the farmed land along lines of their own
// (`land_field`). At a spot the two fields are read between the texels round
// it, at a place a little way off (`edge_warp`: lobes and bays a kilometre or
// so across) and with a fraying added (`edge_fray`), so the outline of a wood
// is an organic line at every zoom. A spot of open land is wood where the
// wood's field stands above a half; failing that, heath where the heath's
// does; failing both, it is farmed as its parcel is (`YearLand::under`), and
// just outside a wood or a heath it is rough with scrub. Built ground is
// what it is. Above the snow that lies all summer nothing is enclosed: all
// open land there is moor. terrain.wesl does the same per pixel.

/// How much faster than its field a wood deepens inward from its edge
/// (`YearLand::wooded`; terrain.wesl has the same number).
pub const WOOD_DEPTH: f32 = 1.8;

/// How high a field must stand, short of the half that makes a wood or a
/// heath, for the ground to be rough with scrub: the ragged edge outside one.
pub const SCRUB: f32 = 0.44;

/// Where the fields of wood and heath are read for a spot: a little way off,
/// by a slow noise, so that their edges wander in lobes and bays.
pub fn edge_warp(x: f32, z: f32) -> (f32, f32) {
    (x + 0.14 * vnoise(x * 1.5 + 3.7, z * 1.5 + 8.2, 461), z + 0.14 * vnoise(x * 1.5 + 6.4, z * 1.5 + 1.1, 463))
}

/// What is added to both fields at a spot, to fray their edges.
pub fn edge_fray(x: f32, z: f32) -> f32 {
    0.09 * vnoise(x * 3.7, z * 3.7, 465) + 0.055 * vnoise(x * 8.9, z * 8.9, 467) + 0.028 * vnoise(x * 21.0, z * 21.0, 469)
}

/// The height at which the snow that lies all summer begins at a spot: the
/// snow line of terrain.wesl (`sp.snow` in summer), with the same noise.
pub fn summer_snow_line(x: f32, z: f32) -> f32 {
    1.75 - 0.12 * vnoise(x * 1.7, z * 1.7, 31)
}

/// Where a spot lies in the land, for asking what it is in any year: its
/// parcel; where in the grid of the year's wood and heath its fields are read
/// (the texel up and left of the warped spot, and how far the spot lies from
/// it toward the next, 0 to 1 each way) and what the fraying adds; and
/// whether it lies above the summer snow.
#[derive(Clone, Copy, Debug, Default)]
pub struct Place {
    pub parcel: u32,
    pub tex: u32,
    pub tx: f32,
    pub ty: f32,
    pub fray: f32,
    pub snow: bool,
}

pub fn place_at(g: &Ground, fx: f32, fy: f32) -> Place {
    let (bl, pc, _) = fields::parcel(Vec2::new(fx, fy));
    let l = &g.land;
    let (qx, qz) = edge_warp(fx, fy);
    let (u, v) = (qx * FIELD_RES as f32 - 0.5, qz * FIELD_RES as f32 - 0.5);
    let (i, j) = (u.floor().clamp(0.0, l.tw as f32 - 2.0), v.floor().clamp(0.0, l.th as f32 - 2.0));
    Place {
        parcel: l.index(bl.i, bl.j, pc.id) as u32,
        tex: (j as usize * l.tw + i as usize) as u32,
        tx: (u - i).clamp(0.0, 1.0),
        ty: (v - j).clamp(0.0, 1.0),
        fray: edge_fray(fx, fy),
        snow: height_at(g, fx, fy) > summer_snow_line(fx, fy),
    }
}

impl YearLand {
    /// How wooded a place is and how much heath, each about 0 to 1: wood
    /// where the first is above a half, heath where the second is.
    pub fn fields(&self, pl: Place) -> (f32, f32) {
        let (t, row) = (pl.tex as usize * 2, self.tw * 2);
        let read = |ch: usize| {
            let at = |o: usize| self.field[t + o + ch] as f32 / 255.0;
            let (a, b, c, d) = (at(0), at(2), at(row), at(row + 2));
            (a + (b - a) * pl.tx) * (1.0 - pl.ty) + (c + (d - c) * pl.tx) * pl.ty + pl.fray
        };
        (read(0), read(1))
    }

    /// The class drawn at a place, and whether the place is the rough,
    /// scrubby ground just outside a wood or a heath.
    pub fn at(&self, pl: Place) -> (u8, bool) {
        let own = self.class[pl.parcel as usize];
        if !(FARM..=HEATH).contains(&own) {
            return (own, false);
        }
        if pl.snow {
            return (HEATH, false);
        }
        let (wood, heath) = self.fields(pl);
        if wood > 0.5 {
            (WOOD, false)
        } else if heath > 0.5 {
            (HEATH, false)
        } else {
            (self.under[pl.parcel as usize], wood > SCRUB || heath > SCRUB)
        }
    }

    /// How wooded a place is, 0 to 1: a half at a wood's edge, more inside
    /// it, less out in the country it frays into. Inside, the field rises
    /// slowly from the edge (it is smoothed), so it is steepened: a wood is
    /// at its full depth a few hundred metres in.
    pub fn wooded(&self, pl: Place) -> f32 {
        wood_depth(self.fields(pl).0)
    }
}

/// How wooded a spot is, 0 to 1, from the wood's field there (`YearLand::wooded`;
/// `wood_depth` in terrain.wesl).
pub fn wood_depth(w: f32) -> f32 {
    (if w > 0.5 { 0.5 + WOOD_DEPTH * (w - 0.5) } else { w }).clamp(0.0, 1.0)
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

/// How far above the tree line a spot of height `h` is, 0 to 1. Over the last
/// climb to the snow that lies on the mountain all summer the woods thin out
/// and their trees dwindle, and nothing grows on the snow itself. The line is
/// the summer snow line of terrain.wesl (`sp.snow`), with the same noise, so
/// the trees stop where the white begins.
pub fn above_tree_line(x: f32, z: f32, h: f32) -> f32 {
    // nearly all the island lies below the lowest the line can come, and needs no noise
    if h < 1.75 - 0.12 - 0.3 {
        return 0.0;
    }
    fields::smoothstep(-0.3, 0.08, h - summer_snow_line(x, z))
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
            built: Vec::new(),
            res: Vec::new(),
            works: Vec::new(),
            workings: Vec::new(),
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

    /// What is painted at `per` spots a cell each way over the made-up island:
    /// for each spot its parcel's class, the class drawn, and whether it is scrub.
    fn painted(g: &Ground, land: &YearLand, per: usize) -> Vec<(u8, u8, bool)> {
        let mut out = Vec::new();
        for j in 0..10 * per {
            for i in 0..12 * per {
                let (x, z) = (1.0 + (i as f32 + 0.5) / per as f32, 1.0 + (j as f32 + 0.5) / per as f32);
                if height_at(g, x, z) > 0.02 {
                    let pl = place_at(g, x, z);
                    let (cls, margin) = land.at(pl);
                    out.push((land.class[pl.parcel as usize], cls, margin));
                }
            }
        }
        out
    }

    #[test]
    fn the_land_keeps_its_areas_as_painted() {
        let r = island();
        let g = build_ground(&r);
        let land = year_land(&r, &g, 0);
        let spots = painted(&g, &land, 12);
        let open: Vec<_> = spots.iter().filter(|s| (FARM..=HEATH).contains(&s.0)).collect();
        // wood and heath are held to their parcels' areas district by district; arable and
        // pasture share what is left as the parcels under the woods and heaths are farmed
        for (name, class, within) in [("arable", FARM, 0.04), ("pasture", PASTURE, 0.04), ("wood", WOOD, 0.02), ("heath", HEATH, 0.02)] {
            let parcels = open.iter().filter(|s| s.0 == class).count() as f32 / open.len() as f32;
            let drawn = open.iter().filter(|s| s.1 == class).count() as f32 / open.len() as f32;
            assert!((drawn - parcels).abs() < within, "{name} is painted over {drawn:.3} of the open land, and its parcels are {parcels:.3} of it");
        }
    }

    #[test]
    fn each_district_keeps_its_wood_and_heath() {
        // the year's two fields, before their edges are warped and frayed: in every region of
        // REGION by REGION districts as many texels stand above the wood's line as lie in its
        // parcels of wood, and of the rest as many above the heath's as lie in its parcels of
        // heath; or, where the parcels are further than KEEP_WITHIN from the record, that far
        // from it
        let r = island();
        let g = build_ground(&r);
        let land = year_land(&r, &g, 0);
        let l = &g.land;
        let side = REGION * KEEP * FIELD_RES;
        let (mut regions, mut worst, mut sum, mut held) = (0, 0.0f32, 0.0f32, 0);
        for d in 0..l.tw.div_ceil(side) * l.th.div_ceil(side) {
            let (x0, y0) = ((d % l.tw.div_ceil(side)) * side, (d / l.tw.div_ceil(side)) * side);
            let mut own = Vec::new();
            for y in y0..(y0 + side).min(l.th) {
                own.extend((x0..(x0 + side).min(l.tw)).map(|x| y * l.tw + x).filter(|&t| l.tex[t] != u32::MAX && (FARM..=HEATH).contains(&land.class[l.tex[t] as usize])));
            }
            if own.len() < 600 {
                continue;
            }
            regions += 1;
            let mut rec = [0.0f32; 4];
            for c in (0..r.n).filter(|&c| (c / r.w) / (REGION * KEEP) == y0 / side && (c % r.w) / (REGION * KEEP) == x0 / side) {
                for (s, v) in rec.iter_mut().zip(open_shares(&r, 0, c)) {
                    *s += v;
                }
            }
            let n = own.len() as f32;
            let open: f32 = rec.iter().sum();
            let wood = own.iter().filter(|&&t| land.field[2 * t] >= 128).count() as f32;
            let heath = own.iter().filter(|&&t| land.field[2 * t] < 128 && land.field[2 * t + 1] >= 128).count() as f32;
            for (name, class, got) in [("wood", WOOD, wood), ("heath", HEATH, heath)] {
                let parcels = own.iter().filter(|&&t| land.class[l.tex[t] as usize] == class).count() as f32 / n;
                let record = rec[(class - FARM) as usize] / open;
                // what the region is held to: its parcels, or KEEP_WITHIN of the record
                let want = parcels.clamp(record - KEEP_WITHIN, record + KEEP_WITHIN);
                held += (want != parcels) as usize;
                let out = (got / n - want).abs();
                (worst, sum) = (worst.max(out), sum + out);
                assert!(out < 0.03, "region {d}: {name} stands on {got} texels of {n}, its parcels on {:.0} and the record has {:.0}", parcels * n, record * n);
            }
        }
        assert!(regions >= 3, "only {regions} regions with land enough to judge");
        assert!(held >= 1, "no region of the test island is held to the record, so this does not test that it is");
        let mean = sum / (2 * regions) as f32;
        assert!(mean < 0.01, "the regions are out by {:.2}% of their open land on average, {:.2}% at worst", 100.0 * mean, 100.0 * worst);
    }

    #[test]
    fn each_district_keeps_its_arable_and_pasture() {
        // in every region of REGION by REGION districts the farmed parcels' arable is within
        // KEEP_WITHIN of the record's share of arable in its farmed land, to the size of a
        // parcel in each of its districts
        let r = island();
        let g = build_ground(&r);
        let land = year_land(&r, &g, 0);
        let l = &g.land;
        let span = REGION * KEEP;
        let rw = r.w.div_ceil(span);
        let mut sums = vec![([0.0f32; 2], [0.0f32; 2], 0.0f32); rw * r.h.div_ceil(span)];
        for c in 0..r.n {
            let s = open_shares(&r, 0, c);
            let d = &mut sums[(c / r.w) / span * rw + (c % r.w) / span];
            (d.0[0], d.0[1]) = (d.0[0] + s[0], d.0[1] + s[1]);
        }
        for i in (0..l.at.len()).filter(|&i| l.area[i] > 0.0 && (land.class[i] == FARM || land.class[i] == PASTURE)) {
            let d = &mut sums[(l.at[i].y.max(0.0) as usize).min(r.h - 1) / span * rw + (l.at[i].x.max(0.0) as usize).min(r.w - 1) / span];
            d.1[(land.class[i] - FARM) as usize] += l.area[i];
            d.2 = d.2.max(l.area[i]);
        }
        let mut judged = 0;
        for (d, (rec, got, biggest)) in sums.iter().enumerate() {
            let farmed = got[0] + got[1];
            if farmed < 1.0 || rec[0] + rec[1] < 1e-3 {
                continue;
            }
            judged += 1;
            let (want, has) = (rec[0] / (rec[0] + rec[1]), got[0] / farmed);
            let slack = KEEP_WITHIN + (REGION * REGION) as f32 * 0.5 * biggest / farmed;
            assert!((has - want).abs() <= slack, "region {d}: arable is {has:.3} of the farmed parcels, and the record has {want:.3}");
        }
        assert!(judged >= 3, "only {judged} regions farmed enough to judge");
    }

    #[test]
    fn the_outline_of_wood_and_heath_is_free_of_the_parcels() {
        let r = island();
        let g = build_ground(&r);
        let land = year_land(&r, &g, 0);
        let spots = painted(&g, &land, 12);
        let n = |f: &dyn Fn(&(u8, u8, bool)) -> bool| spots.iter().filter(|s| f(s)).count();
        // each has grown over some of the farmed parcels beside it, and left as much of its own to be farmed
        for (name, class) in [("wood", WOOD), ("heath", HEATH)] {
            let (over, left) = (n(&|s| s.0 < WOOD && s.1 == class), n(&|s| s.0 == class && s.1 < WOOD));
            assert!(over > 40 && left > 40, "{name} over {over} spots of farmed parcels, and {left} spots of its own left to be farmed");
            assert!((over as f32 / left as f32 - 1.0).abs() < 0.5, "{name} takes {over} spots and gives {left}");
        }
        // the scrub is farmed land, just outside a wood or a heath
        assert!(n(&|s| s.2) > 40);
        assert_eq!(n(&|s| s.2 && s.1 >= WOOD), 0);
    }

    #[test]
    fn a_place_reads_the_fields_between_their_texels() {
        // four texels in a row and two rows: wood rising to the right, heath to the bottom
        let land = YearLand {
            class: vec![PASTURE, WOOD, HEATH, FARM, URBAN],
            town: Default::default(),
            field: vec![0, 0, 85, 0, 170, 0, 255, 0, 0, 255, 85, 255, 170, 255, 255, 255],
            tw: 4,
            under: vec![PASTURE, FARM, PASTURE, FARM, URBAN],
            depth: vec![3; 5],
        };
        let at = |parcel: u32, tex: u32, tx: f32, ty: f32| land.at(Place { parcel, tex, tx, ty, fray: 0.0, snow: false });
        let (w, h) = land.fields(Place { parcel: 0, tex: 1, tx: 0.5, ty: 0.25, fray: 0.0, snow: false });
        assert!((w - 0.5).abs() < 1e-5 && (h - 0.25).abs() < 1e-5, "wood {w}, heath {h}");
        // open land is wood where the wood's field is above a half, heath where the heath's is
        assert_eq!(at(0, 1, 0.6, 0.0), (WOOD, false));
        assert_eq!(at(3, 1, 0.6, 0.0), (WOOD, false));
        assert_eq!(at(0, 0, 0.5, 0.9), (HEATH, false));
        assert_eq!(at(2, 2, 0.5, 0.9), (WOOD, false));
        // and where neither is, it is farmed as its parcel is, a wood's or a heath's parcel too
        assert_eq!(at(0, 0, 0.5, 0.2), (PASTURE, false));
        assert_eq!(at(1, 0, 0.5, 0.2), (FARM, false));
        assert_eq!(at(2, 0, 0.5, 0.2), (PASTURE, false));
        // rough with scrub just outside the wood
        assert_eq!(at(3, 1, 0.4, 0.0), (FARM, true));
        // the fraying moves the line
        assert_eq!(land.at(Place { parcel: 0, tex: 1, tx: 0.4, ty: 0.0, fray: 0.1, snow: false }), (WOOD, false));
        // built ground is what it is
        assert_eq!(at(4, 2, 0.5, 0.9), (URBAN, false));
        // above the summer snow all open land is moor
        assert_eq!(land.at(Place { parcel: 3, tex: 0, tx: 0.0, ty: 0.0, fray: 0.0, snow: true }), (HEATH, false));
        assert_eq!(land.wooded(Place { parcel: 3, tex: 2, tx: 1.0, ty: 0.0, fray: 0.2, snow: false }), 1.0);
        // a wood deepens inward faster than its field, and what lies outside it is as the field says
        let deep = land.wooded(Place { parcel: 0, tex: 1, tx: 0.8, ty: 0.0, fray: 0.0, snow: false });
        assert!((deep - (0.5 + WOOD_DEPTH * 0.1)).abs() < 1e-4, "{deep}");
        assert!((land.wooded(Place { parcel: 0, tex: 0, tx: 0.9, ty: 0.0, fray: 0.0, snow: false }) - 0.3).abs() < 1e-4);
    }

    #[test]
    fn the_trees_stop_at_the_summer_snow() {
        for i in 0..200 {
            let (x, z) = (100.0 * hash2(i, 5, 11), 80.0 * hash2(i, 6, 11));
            // the shader's summer snow begins at this height (terrain.wesl, `sp.snow`)
            let snow = 1.75 - 0.12 * vnoise(x * 1.7, z * 1.7, 31);
            assert_eq!(above_tree_line(x, z, 1.3), 0.0, "the hillsides keep their woods");
            assert_eq!(above_tree_line(x, z, snow + 0.08), 1.0, "nothing grows on the snow");
            let (low, high) = (above_tree_line(x, z, snow - 0.2), above_tree_line(x, z, snow - 0.05));
            assert!(0.0 < low && low < high && high < 1.0, "the woods thin out on the way up: {low} then {high}");
        }
    }

    #[test]
    fn a_town_founded_later_gets_its_closes_in_its_year() {
        // two years of the made-up island, with a town from before the record and one founded in 1751
        let mut r = island();
        (r.ny, r.filled) = (2, 2);
        for a in [&mut r.mix_a, &mut r.mix_b, &mut r.mix_s] {
            let year = a.clone();
            a.extend(year);
        }
        let (old, new) = (4 * r.w + 3, 7 * r.w + 9);
        r.header.towns = serde_json::from_str(&format!(r#"[{{"name":"Oldham","node":{old},"founded":1700}},{{"name":"Newton","node":{new},"founded":1751}}]"#)).unwrap();
        let g = build_ground(&r);
        let cen = crate::network::centres(&r);
        let at = |node: usize| Vec2::new(cen[node].0, cen[node].1);
        let (before, after) = (year_land(&r, &g, 0), year_land(&r, &g, 1));
        // the first year is the land as built: closes round the old town, none yet round the new
        assert_eq!(before.depth, g.land.depth);
        let mean = |depth: &[u8], town: Vec2, within: f32| {
            let near: Vec<f32> = (0..depth.len()).filter(|&i| g.land.area[i] > 0.0 && g.land.at[i].distance(town) < within).map(|i| depth[i] as f32).collect();
            near.iter().sum::<f32>() / near.len() as f32
        };
        assert!(mean(&before.depth, at(old), 0.9) > mean(&before.depth, at(new), 0.9) + 0.5, "the old town has no closes of its own");
        // in its year the new town's fields are cut further, and no field anywhere is cut less
        assert!(mean(&after.depth, at(new), 0.9) > mean(&before.depth, at(new), 0.9) + 0.8, "the new town got no closes");
        for i in 0..after.depth.len() {
            assert!(after.depth[i] >= before.depth[i], "parcel {i} lost a hedge");
            if g.land.at[i].distance(at(new)) > CLOSES_REACH {
                assert_eq!(after.depth[i], before.depth[i], "parcel {i}, far from the new town, was cut again");
            }
        }
        // a town founded later was never the middle of an open-field parish
        for i in 0..g.land.at.len() {
            assert!(g.land.open_until[i] == 0 || g.land.at[i].distance(at(old)) < g.land.at[i].distance(at(new)) + 2.0);
        }
    }

    #[test]
    fn nothing_is_enclosed_above_the_summer_snow() {
        let r = island();
        let mut g = build_ground(&r);
        // the line wanders within 0.12 of 1.75, and the made-up island's ridges stand far
        // below it: no place on it is above the line
        for i in 0..500 {
            let (x, z) = (1.0 + 12.0 * hash2(i, 3, 9), 1.0 + 10.0 * hash2(i, 4, 9));
            assert!(!place_at(&g, x, z).snow);
            assert!(summer_snow_line(x, z) >= 1.63 && summer_snow_line(x, z) <= 1.87);
        }
        // raise the ground above the line and every place is under the snow, where all open
        // land is moor whatever its parcel and the year's fields say
        let land = year_land(&r, &g, 0);
        g.hgt.iter_mut().for_each(|h| *h += 2.0);
        let mut farmed = 0;
        for i in 0..500 {
            let (x, z) = (1.0 + 12.0 * hash2(i, 3, 9), 1.0 + 10.0 * hash2(i, 4, 9));
            let pl = place_at(&g, x, z);
            assert!(pl.snow);
            let own = land.class[pl.parcel as usize];
            if (FARM..=HEATH).contains(&own) {
                farmed += (own < WOOD) as usize;
                assert_eq!(land.at(pl), (HEATH, false));
            }
        }
        assert!(farmed > 50, "only {farmed} spots of farmed parcels to judge by");
    }
}
