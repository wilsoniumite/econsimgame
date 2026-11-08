//! Procedural map: terrain, coast, rivers, resources and the first towns. Python: `world.py`.
//!
//! All fields are row-major (H, W) grids flattened to length N = H * W; cell index
//! c = y * W + x.

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::f64::consts::PI;

use crate::ndimage::{self, Mode};
use crate::rng::{Rng, integers, random_n, uniform};

pub const NEIGH8: [(isize, isize); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

pub const SEA: u8 = 0;
pub const LOWLAND: u8 = 1;
pub const HILLS: u8 = 2;
pub const MOUNTAINS: u8 = 3;

#[derive(Clone, Debug, Default)]
pub struct World {
    pub W: usize,
    pub H: usize,
    pub land: Vec<bool>,
    /// Height above sea level, 0..1.
    pub h: Vec<f64>,
    pub terrain: Vec<u8>,
    /// Downstream neighbour of each land cell (-1 for outlets and sea).
    pub down: Vec<i64>,
    /// Flow accumulation: number of cells draining through each cell.
    pub acc: Vec<f64>,
    pub river: Vec<bool>,
    pub navigable: Vec<bool>,
    pub coast: Vec<bool>,
    pub fertility: Vec<f64>,
    pub pasture: Vec<f64>,
    pub forest_pot: Vec<f64>,
    pub rock: Vec<f64>,
    pub clay: Vec<f64>,
    pub ore: Vec<f64>,
    pub coal: Vec<f64>,
    pub jitter: Vec<f64>,
    pub town_sites: Vec<usize>,
    pub forest0: Vec<f64>,
}

impl World {
    /// A resource field by its content name (`sector.field`); "forest" is not a
    /// world field, the caller uses the state's tree cover for it.
    pub fn field(&self, name: &str) -> &[f64] {
        match name {
            "fertility" => &self.fertility,
            "pasture" => &self.pasture,
            "forest_pot" => &self.forest_pot,
            "rock" => &self.rock,
            "clay" => &self.clay,
            "ore" => &self.ore,
            "coal" => &self.coal,
            _ => panic!("unknown world field {name:?}"),
        }
    }
}

/// A total order on f64 as u64 keys, for heaps (negative values sort below positive).
pub fn ord_key(v: f64) -> u64 {
    let b = v.to_bits();
    if b >> 63 == 1 { !b } else { b | (1 << 63) }
}

/// Python's `max(H) - min(H)`-normalised fractal value noise in [0, 1]: random
/// grids upsampled with cubic splines.
pub fn smooth_noise(rng: &mut Rng, H: usize, W: usize, scale: f64, octaves: usize, persistence: f64) -> Vec<f64> {
    let mut out = vec![0.0; H * W];
    let (mut amp, mut total) = (1.0, 0.0);
    for o in 0..octaves {
        // Python's round() rounds half to even
        let step = 2usize.max((scale / 2f64.powi(o as i32)).round_ties_even() as usize);
        let (gh, gw) = (H / step + 4, W / step + 4);
        let grid = random_n(rng, gh * gw);
        let (z, _, zw) = ndimage::zoom_cubic(&grid, gh, gw, step);
        let oy = integers(rng, step);
        let ox = integers(rng, step);
        for y in 0..H {
            for x in 0..W {
                out[y * W + x] += amp * z[(oy + y) * zw + ox + x];
            }
        }
        total += amp;
        amp *= persistence;
    }
    for v in out.iter_mut() {
        *v /= total;
    }
    let lo = out.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = out.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    out.iter().map(|v| (v - lo) / (hi - lo + 1e-12)).collect()
}

pub fn blob(H: usize, W: usize, cy: f64, cx: f64, sy: f64, sx: f64, angle: f64) -> Vec<f64> {
    let (ca, sa) = (angle.cos(), angle.sin());
    let mut out = vec![0.0; H * W];
    for y in 0..H {
        for x in 0..W {
            let (dy, dx) = (y as f64 - cy, x as f64 - cx);
            let u = (dx * ca + dy * sa) / sx;
            let v = (-dx * sa + dy * ca) / sy;
            out[y * W + x] = (-(u * u + v * v)).exp();
        }
    }
    out
}

/// Distance (in cells) from every cell to a quadratic Bezier curve, sampled at 300 points.
pub fn bezier_distance(H: usize, W: usize, p0: (f64, f64), p1: (f64, f64), p2: (f64, f64)) -> Vec<f64> {
    let n = 300;
    let step = 1.0 / (n - 1) as f64;
    let mut d = vec![f64::INFINITY; H * W];
    for k in 0..n {
        let t = if k == n - 1 { 1.0 } else { k as f64 * step };
        let (a, b, c) = ((1.0 - t) * (1.0 - t), 2.0 * (1.0 - t) * t, t * t);
        let py = a * p0.0 + b * p1.0 + c * p2.0;
        let px = a * p0.1 + b * p1.1 + c * p2.1;
        for y in 0..H {
            for x in 0..W {
                let v = (y as f64 - py).hypot(x as f64 - px);
                let e = &mut d[y * W + x];
                if v < *e {
                    *e = v;
                }
            }
        }
    }
    d
}

/// Priority-flood so every land cell drains to the sea.
pub fn fill_pits(elev: &[f64], land: &[bool], H: usize, W: usize) -> Vec<f64> {
    let mut filled = elev.to_vec();
    let mut done: Vec<bool> = land.iter().map(|&l| !l).collect();
    // heapq on (elevation, y, x): ties break on y, then x
    let mut heap: BinaryHeap<Reverse<(u64, usize, usize)>> = BinaryHeap::new();
    for y in 0..H {
        for x in 0..W {
            if land[y * W + x] {
                let mut edge = y == 0 || y == H - 1 || x == 0 || x == W - 1;
                if !edge {
                    for (dy, dx) in NEIGH8 {
                        let (ny, nx) = ((y as isize + dy) as usize, (x as isize + dx) as usize);
                        if !land[ny * W + nx] {
                            edge = true;
                            break;
                        }
                    }
                }
                if edge {
                    heap.push(Reverse((ord_key(filled[y * W + x]), y, x)));
                    done[y * W + x] = true;
                }
            }
        }
    }
    while let Some(Reverse((_, y, x))) = heap.pop() {
        let e = filled[y * W + x];
        for (dy, dx) in NEIGH8 {
            let (ny, nx) = (y as isize + dy, x as isize + dx);
            if ny >= 0 && ny < H as isize && nx >= 0 && nx < W as isize {
                let nc = ny as usize * W + nx as usize;
                if !done[nc] {
                    done[nc] = true;
                    filled[nc] = filled[nc].max(e + 1e-5);
                    heap.push(Reverse((ord_key(filled[nc]), ny as usize, nx as usize)));
                }
            }
        }
    }
    filled
}

/// Downstream neighbour index for each land cell (-1 for outlets and sea).
pub fn flow_directions(filled: &[f64], land: &[bool], H: usize, W: usize) -> Vec<i64> {
    let mut down = vec![-1i64; H * W];
    for y in 0..H {
        for x in 0..W {
            if !land[y * W + x] {
                continue;
            }
            let (mut best, mut bi) = (filled[y * W + x], -1i64);
            for (dy, dx) in NEIGH8 {
                let (ny, nx) = (y as isize + dy, x as isize + dx);
                if ny >= 0 && ny < H as isize && nx >= 0 && nx < W as isize {
                    let nc = ny as usize * W + nx as usize;
                    let e = if land[nc] { filled[nc] } else { -1.0 };
                    if e < best {
                        best = e;
                        bi = nc as i64;
                    }
                }
            }
            down[y * W + x] = bi;
        }
    }
    down
}

pub fn flow_accumulation(filled: &[f64], land: &[bool], down: &[i64]) -> Vec<f64> {
    let mut acc: Vec<f64> = land.iter().map(|&l| if l { 1.0 } else { 0.0 }).collect();
    // highest first; every cell drains to a strictly lower one, so ties do not matter
    let mut order: Vec<usize> = (0..land.len()).filter(|&c| land[c]).collect();
    order.sort_by(|&a, &b| filled[b].total_cmp(&filled[a]));
    for c in order {
        let d = down[c];
        if d >= 0 {
            acc[d as usize] += acc[c];
        }
    }
    acc
}

/// `np.quantile(a, q)` with the default linear method.
fn quantile(a: &[f64], q: f64) -> f64 {
    let mut s = a.to_vec();
    s.sort_by(|x, y| x.total_cmp(y));
    let n = s.len();
    let virt = q * (n - 1) as f64;
    let prev = virt.floor();
    let t = virt - prev;
    let i = prev as usize;
    let (lo, hi) = (s[i], s[(i + 1).min(n - 1)]);
    let diff = hi - lo;
    if t >= 0.5 { hi - diff * (1.0 - t) } else { lo + diff * t }
}

fn hypot_grid(H: usize, W: usize, ty: usize, tx: usize) -> Vec<f64> {
    let mut out = vec![0.0; H * W];
    for y in 0..H {
        for x in 0..W {
            out[y * W + x] = (y as f64 - ty as f64).hypot(x as f64 - tx as f64);
        }
    }
    out
}

pub fn generate_world(rng: &mut Rng, W: usize, H: usize, n_towns: usize, land_fraction: f64) -> World {
    let N = H * W;
    let base = smooth_noise(rng, H, W, 26.0, 5, 0.55);
    let detail = smooth_noise(rng, H, W, 8.0, 3, 0.5);
    // Domain-warped falloff gives bays and peninsulas instead of a round blob.
    let wx: Vec<f64> = smooth_noise(rng, H, W, 22.0, 3, 0.5).iter().map(|v| v - 0.5).collect();
    let wy: Vec<f64> = smooth_noise(rng, H, W, 22.0, 3, 0.5).iter().map(|v| v - 0.5).collect();
    let mut island = vec![0.0; N];
    for y in 0..H {
        for x in 0..W {
            let c = y * W + x;
            let (fx, fy) = (x as f64 / (W - 1) as f64, y as f64 / (H - 1) as f64);
            let a = (fx + 0.22 * wx[c] - 0.5) / 0.5;
            let b = (fy + 0.22 * wy[c] - 0.52) / 0.47;
            let falloff = (a * a + b * b).sqrt();
            island[c] = 1.0 - falloff.clamp(0.0, 1.4).powf(2.3);
        }
    }

    // A curved spine of hills, like a Pennine chain, with randomised control points.
    let (Hf, Wf) = (H as f64, W as f64);
    let p0 = (Hf * uniform(rng, 0.12, 0.25), Wf * uniform(rng, 0.25, 0.4));
    let p1 = (Hf * uniform(rng, 0.4, 0.6), Wf * uniform(rng, 0.45, 0.7));
    let p2 = (Hf * uniform(rng, 0.75, 0.9), Wf * uniform(rng, 0.35, 0.6));
    let ridge_d = bezier_distance(H, W, p0, p1, p2);
    let ridge: Vec<f64> = (0..N)
        .map(|c| {
            let r = ridge_d[c] / (0.07 * Wf);
            (-(r * r)).exp() * (0.6 + 0.6 * base[c])
        })
        .collect();

    let mut elev: Vec<f64> = (0..N).map(|c| 0.5 * base[c] + 0.14 * detail[c] + 0.4 * ridge[c] + 0.55 * island[c]).collect();
    let emin = elev.iter().cloned().fold(f64::INFINITY, f64::min);
    for y in [0, 1, H - 2, H - 1] {
        for x in 0..W {
            elev[y * W + x] = emin;
        }
    }
    for y in 0..H {
        for x in [0, 1, W - 2, W - 1] {
            elev[y * W + x] = emin;
        }
    }
    let sea_level = quantile(&elev, 1.0 - land_fraction);
    let mut land: Vec<bool> = elev.iter().map(|&e| e > sea_level).collect();
    // drop tiny islands and lakes for a cleaner map
    let (lab, n) = ndimage::label(&land, H, W);
    if n > 1 {
        let mut sizes = vec![0usize; n + 1];
        for &l in &lab {
            if l > 0 {
                sizes[l as usize] += 1;
            }
        }
        let mut keep = 1;
        for l in 2..=n {
            if sizes[l] > sizes[keep] {
                keep = l;
            }
        }
        land = lab.iter().map(|&l| l as usize == keep).collect();
    }
    let not_land: Vec<bool> = land.iter().map(|&l| !l).collect();
    let (sea_lab, n) = ndimage::label(&not_land, H, W);
    if n > 1 {
        let mut border = vec![false; n + 1];
        for x in 0..W {
            border[sea_lab[x] as usize] = true;
            border[sea_lab[(H - 1) * W + x] as usize] = true;
        }
        for y in 0..H {
            border[sea_lab[y * W] as usize] = true;
            border[sea_lab[y * W + W - 1] as usize] = true;
        }
        for c in 0..N {
            if !land[c] && !border[sea_lab[c] as usize] {
                land[c] = true; // a lake
            }
        }
    }

    let emax = (0..N).filter(|&c| land[c]).map(|c| elev[c]).fold(f64::NEG_INFINITY, f64::max);
    let h: Vec<f64> = (0..N)
        .map(|c| {
            let v = if land[c] { (elev[c] - sea_level) / (emax - sea_level) } else { 0.0 };
            v.clamp(0.0, 1.0).powf(1.35)
        })
        .collect();

    let terrain: Vec<u8> = (0..N)
        .map(|c| {
            if !land[c] {
                SEA
            } else if h[c] < 0.24 {
                LOWLAND
            } else if h[c] < 0.5 {
                HILLS
            } else {
                MOUNTAINS
            }
        })
        .collect();

    let filled = fill_pits(&h, &land, H, W);
    let down = flow_directions(&filled, &land, H, W);
    let acc = flow_accumulation(&filled, &land, &down);
    let river: Vec<bool> = (0..N).map(|c| land[c] && acc[c] >= 45.0).collect();
    let navigable: Vec<bool> = (0..N).map(|c| land[c] && acc[c] >= 140.0).collect();

    let not_land: Vec<bool> = land.iter().map(|&l| !l).collect();
    let near_sea = ndimage::binary_dilation(&not_land, H, W, true);
    let coast: Vec<bool> = (0..N).map(|c| land[c] && near_sea[c]).collect();
    let not_river: Vec<bool> = river.iter().map(|&r| !r).collect();
    let near_river = ndimage::distance_transform_edt(&not_river, H, W);

    let n1 = smooth_noise(rng, H, W, 14.0, 4, 0.5);
    let n2 = smooth_noise(rng, H, W, 10.0, 4, 0.5);
    let n3 = smooth_noise(rng, H, W, 12.0, 4, 0.5);
    let by_terrain = |c: usize, lo: f64, hi: f64, mo: f64| match terrain[c] {
        LOWLAND => lo,
        HILLS => hi,
        MOUNTAINS => mo,
        _ => 0.0,
    };
    let landf = |c: usize| if land[c] { 1.0 } else { 0.0 };

    let fertility: Vec<f64> = (0..N)
        .map(|c| {
            let f = by_terrain(c, 0.78, 0.38, 0.06) * (0.6 + 0.55 * n1[c]) + 0.18 * (-near_river[c] / 2.0).exp() * landf(c);
            f.clamp(0.0, 1.0) * landf(c)
        })
        .collect();
    let pasture: Vec<f64> = (0..N).map(|c| (by_terrain(c, 0.5, 0.75, 0.35) * (0.7 + 0.45 * n2[c])).clamp(0.0, 1.0) * landf(c)).collect();
    let forest_pot: Vec<f64> =
        (0..N).map(|c| (by_terrain(c, 0.72, 0.9, 0.45) * (0.65 + 0.5 * n3[c])).clamp(0.0, 1.0) * landf(c)).collect();
    let rock: Vec<f64> = (0..N).map(|c| (by_terrain(c, 0.08, 0.5, 0.85) * (0.6 + 0.6 * n2[c])).clamp(0.0, 1.0) * landf(c)).collect();
    let clay: Vec<f64> = (0..N)
        .map(|c| {
            let base = if terrain[c] == LOWLAND { 0.25 + 0.55 * (-near_river[c] / 2.5).exp() } else { 0.08 };
            (base * (0.6 + 0.6 * n3[c])).clamp(0.0, 1.0) * landf(c)
        })
        .collect();

    // Ore bodies in the hills, coalfields on the flanks of the spine.
    let mut ore = vec![0.0f64; N];
    let hill_cells: Vec<usize> = (0..N).filter(|&c| terrain[c] >= HILLS && land[c]).collect();
    assert!(!hill_cells.is_empty(), "world has no hills for ore bodies");
    for _ in 0..7 {
        let c = hill_cells[integers(rng, hill_cells.len())];
        let (cy, cx) = ((c / W) as f64, (c % W) as f64);
        let k = uniform(rng, 0.55, 1.0);
        let (sy, sx, ang) = (uniform(rng, 1.5, 3.0), uniform(rng, 1.5, 3.5), uniform(rng, 0.0, PI));
        let b = blob(H, W, cy, cx, sy, sx, ang);
        for i in 0..N {
            ore[i] = ore[i].max(k * b[i]);
        }
    }
    for i in 0..N {
        ore[i] = (if ore[i] > 0.15 { ore[i] } else { 0.0 }) * landf(i);
    }

    let mut coal = vec![0.0f64; N];
    let flank: Vec<usize> =
        (0..N).filter(|&c| land[c] && ridge_d[c] > 0.06 * Wf && ridge_d[c] < 0.16 * Wf && terrain[c] <= HILLS).collect();
    assert!(!flank.is_empty(), "world has no ridge flanks for coalfields");
    for _ in 0..4 {
        let c = flank[integers(rng, flank.len())];
        let (cy, cx) = ((c / W) as f64, (c % W) as f64);
        let k = uniform(rng, 0.7, 1.0);
        let (sy, sx, ang) = (uniform(rng, 3.0, 4.5), uniform(rng, 6.0, 9.0), uniform(rng, 0.0, PI));
        let b = blob(H, W, cy, cx, sy, sx, ang);
        for i in 0..N {
            coal[i] = coal[i].max(k * b[i]);
        }
    }
    for i in 0..N {
        coal[i] = (if coal[i] > 0.18 { coal[i] } else { 0.0 }) * landf(i);
        ore[i] = ore[i].max(0.4 * coal[i]); // ironstone bands in the coal measures
    }

    let jitter = smooth_noise(rng, H, W, 4.0, 2, 0.5);
    let mut world = World {
        W,
        H,
        land,
        h,
        terrain,
        down,
        acc,
        river,
        navigable,
        coast,
        fertility,
        pasture,
        forest_pot,
        rock,
        clay,
        ore,
        coal,
        jitter,
        town_sites: Vec::new(),
        forest0: Vec::new(),
    };
    world.town_sites = pick_town_sites(&world, rng, n_towns);
    // Woods are already thinned around old settlements in 1750.
    let mut cleared = vec![0.0f64; N];
    for &c in &world.town_sites {
        let d = hypot_grid(H, W, c / W, c % W);
        for i in 0..N {
            cleared[i] = cleared[i].max((-d[i] / 9.0).exp());
        }
    }
    world.forest0 = (0..N).map(|i| world.forest_pot[i] * (1.0 - 0.85 * cleared[i])).collect();
    world
}

pub fn pick_town_sites(world: &World, rng: &mut Rng, n: usize) -> Vec<usize> {
    let (H, W) = (world.H, world.W);
    let N = H * W;
    let fert_sum = ndimage::uniform_filter(&world.fertility, H, W, 11, Mode::Reflect);
    let near_nav = ndimage::binary_dilation(&world.navigable, H, W, false);
    let noise = random_n(rng, N);
    let mut score: Vec<f64> = (0..N)
        .map(|c| {
            fert_sum[c]
                + 0.35 * (near_nav[c] as u8 as f64)
                + 0.25 * (world.coast[c] as u8 as f64)
                + 0.08 * noise[c]
        })
        .collect();
    for c in 0..N {
        let (y, x) = (c / W, c % W);
        if !world.land[c] || world.terrain[c] == MOUNTAINS || y < 3 || y >= H - 3 || x < 3 || x >= W - 3 {
            score[c] = f64::NEG_INFINITY;
        }
    }
    let mut sites = Vec::new();
    for _ in 0..n {
        let mut c = 0;
        for i in 1..N {
            if score[i] > score[c] {
                c = i;
            }
        }
        if !score[c].is_finite() {
            break;
        }
        sites.push(c);
        let d = hypot_grid(H, W, c / W, c % W);
        for i in 0..N {
            if d[i] < 17.0 {
                score[i] = f64::NEG_INFINITY;
            }
        }
    }
    sites
}

pub fn site_context(world: &World, c: usize) -> &'static str {
    let W = world.W;
    let (y, x) = (c / W, c % W);
    if world.coast[c] {
        return "coast";
    }
    if world.coal[c] > 0.2 || world.ore[c] > 0.2 {
        return "mine";
    }
    for yy in y.saturating_sub(1)..(y + 2).min(world.H) {
        for xx in x.saturating_sub(1)..(x + 2).min(W) {
            if world.river[yy * W + xx] {
                return "river";
            }
        }
    }
    if world.terrain[c] >= HILLS {
        return "hills";
    }
    "plain"
}
