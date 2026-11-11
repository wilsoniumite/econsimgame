//! Land (not in the Python; see docs/rules/land.md): an island made by water.
//!
//! The port's island is a sum of smooth noise. This one is built the way land is:
//! rock of different hardness is lifted at different rates in different places (a
//! mountain belt and its foothills, a tableland or two of hard rock over soft,
//! rolling lowland, a basin, a coastal plain), and rivers cut it down. The work is
//! done on a grid finer than the cells (`LandParams::fine` nodes along a cell's
//! side), in metres.
//!
//! This file holds the numbers (`LandParams`), the plan of an island drawn from
//! the seed (`Design`), and the erosion (`Eroder`). `landwater.rs` puts the water
//! on the finished land (rain, lakes, discharge, river courses) and `landcells.rs`
//! reduces it all to the `World` the economy runs on.
//!
//! Nothing here is reached unless `P.land.enabled`; the port's `generate_world`
//! is not touched, and every random draw comes from stream 11 (`rng::land_stream`).

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::ndimage;
use crate::rng::{Rng, random, uniform};
use crate::world::ord_key;

/// A cell is 2 km square, as everywhere else.
pub const CELL_M: f64 = 2000.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LandParams {
    /// The island is made by uplift and erosion, with its water, instead of from noise.
    pub enabled: bool,
    /// Fine nodes along a cell's side (4 is 500 m, 8 is 250 m). Even, 2 to 8.
    pub fine: usize,
    /// Share of the map that is land, as the port's 0.58.
    pub land_fraction: f64,
    /// Height the mountain belt is lifted towards, metres.
    pub relief: f64,
    /// Erosion steps on a grid of half the fineness, where the big valleys are cut...
    pub coarse_steps: usize,
    /// ...and on the fine grid, where the small ones are.
    pub fine_steps: usize,
    /// How fast rivers cut: the stream-power law's K, for rock of hardness 1, a
    /// drainage area of 1 square km and a fall over 500 m, per step.
    pub erodibility: f64,
    /// Share of the load from upstream a river drops again (fans, flood plains).
    pub deposition: f64,
    /// Soil creep: how far a hillside node moves towards its neighbours' mean each step.
    pub creep: f64,
    /// Steepest slope rock holds before it slides.
    pub max_slope: f64,
    /// The most upland lakes an island looks for: each seed looks for 2 or
    /// more, up to this many (0 for none).
    pub upland_lakes: usize,
    /// Chance that an island has a large lowland lake.
    pub lowland_lake: f64,
    /// Mean rain at sea level on the windward coast, mm a year.
    pub rain: f64,
    /// Mean flow, cubic metres a second, from which a cell counts as having a river...
    pub river_flow: f64,
    /// ...and from which boats use it, where it falls less than `nav_slope`.
    pub nav_flow: f64,
    pub nav_slope: f64,
    /// Smallest stream kept as a line for drawing, cubic metres a second.
    pub stream_flow: f64,
    /// Steps of meander migration given to the lowland reaches.
    pub meander_steps: usize,
    /// Height in metres that counts as 1 in the world's `h`.
    pub h_ref: f64,
}

impl Default for LandParams {
    /// Off: what the port runs with.
    fn default() -> LandParams {
        LandParams {
            enabled: false,
            fine: 4,
            land_fraction: 0.58,
            relief: 1000.0,
            coarse_steps: 110,
            fine_steps: 36,
            erodibility: 0.25,
            deposition: 0.6,
            creep: 0.06,
            max_slope: 0.55,
            upland_lakes: 5,
            lowland_lake: 0.35,
            rain: 900.0,
            river_flow: 5.0,
            nav_flow: 12.0,
            nav_slope: 0.0015,
            stream_flow: 0.3,
            meander_steps: 230,
            h_ref: 1500.0,
        }
    }
}

impl LandParams {
    /// The game's land.
    pub fn game() -> LandParams {
        LandParams { enabled: true, ..LandParams::default() }
    }
}

// ---------------------------------------------------------------- small tools

pub fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn hash(ix: i64, iy: i64, seed: u64) -> u64 {
    let mut h = seed ^ (ix as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (iy as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 32;
    h = h.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    h ^= h >> 32;
    h = h.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    h ^ (h >> 32)
}

/// A number in [0, 1) that belongs to a lattice point.
pub fn hash01(ix: i64, iy: i64, seed: u64) -> f64 {
    (hash(ix, iy, seed) >> 11) as f64 / (1u64 << 53) as f64
}

/// Gradient noise, about -1 to 1, smooth, with a lattice of spacing 1.
pub fn perlin(x: f64, y: f64, seed: u64) -> f64 {
    const G: [(f64, f64); 8] = [
        (1.0, 0.0),
        (0.7071067811865476, 0.7071067811865476),
        (0.0, 1.0),
        (-0.7071067811865476, 0.7071067811865476),
        (-1.0, 0.0),
        (-0.7071067811865476, -0.7071067811865476),
        (0.0, -1.0),
        (0.7071067811865476, -0.7071067811865476),
    ];
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (ix, iy) = (x0 as i64, y0 as i64);
    let corner = |dx: i64, dy: i64| {
        let g = G[(hash(ix + dx, iy + dy, seed) >> 20) as usize & 7];
        g.0 * (fx - dx as f64) + g.1 * (fy - dy as f64)
    };
    let u = fx * fx * fx * (fx * (fx * 6.0 - 15.0) + 10.0);
    let v = fy * fy * fy * (fy * (fy * 6.0 - 15.0) + 10.0);
    let a = mix(corner(0, 0), corner(1, 0), u);
    let b = mix(corner(0, 1), corner(1, 1), u);
    1.5 * mix(a, b, v)
}

/// Octaves of `perlin`, each of half the wavelength and `pers` the weight; 0 to 1.
pub fn fbm01(x: f64, y: f64, seed: u64, octaves: u32, pers: f64) -> f64 {
    let (mut amp, mut tot, mut f, mut s) = (1.0, 0.0, 1.0, 0.0);
    for o in 0..octaves {
        s += amp * perlin(x * f, y * f, seed.wrapping_add(o as u64 * 7919));
        tot += amp;
        amp *= pers;
        f *= 2.0;
    }
    (0.5 + 0.5 * s / tot).clamp(0.0, 1.0)
}

/// A grid of nodes over the map, `k` to a cell's side. Node (i, j) has its
/// centre at ((i + 0.5) / k, (j + 0.5) / k) in cell units, so cell (x, y) holds
/// the nodes with i in x*k .. x*k + k and j likewise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    pub w: usize,
    pub h: usize,
    pub k: usize,
}

/// The eight neighbours as (dx, dy, distance in node spacings).
pub const N8: [(isize, isize, f64); 8] = [
    (-1, -1, std::f64::consts::SQRT_2),
    (0, -1, 1.0),
    (1, -1, std::f64::consts::SQRT_2),
    (-1, 0, 1.0),
    (1, 0, 1.0),
    (-1, 1, std::f64::consts::SQRT_2),
    (0, 1, 1.0),
    (1, 1, std::f64::consts::SQRT_2),
];

impl Grid {
    pub fn new(W: usize, H: usize, k: usize) -> Grid {
        Grid { w: W * k, h: H * k, k }
    }
    pub fn n(&self) -> usize {
        self.w * self.h
    }
    /// Node spacing in metres.
    pub fn dx(&self) -> f64 {
        CELL_M / self.k as f64
    }
    /// A node's centre in cell units.
    pub fn xy(&self, n: usize) -> (f64, f64) {
        (((n % self.w) as f64 + 0.5) / self.k as f64, ((n / self.w) as f64 + 0.5) / self.k as f64)
    }
    /// Index offsets of the eight neighbours, in `N8` order.
    pub fn offsets(&self) -> [isize; 8] {
        let w = self.w as isize;
        [-w - 1, -w, -w + 1, -1, 1, w - 1, w, w + 1]
    }
    /// The cell a node lies in.
    pub fn cell(&self, n: usize) -> usize {
        (n / self.w / self.k) * (self.w / self.k) + (n % self.w) / self.k
    }
}

/// Distance in node spacings from every node where `from` is false to the nearest
/// where it is true (0 there): a two-pass chamfer, good to a few per cent.
pub fn chamfer(from: &[bool], w: usize, h: usize) -> Vec<f32> {
    let big = 1e9f32;
    let mut d: Vec<f32> = from.iter().map(|&f| if f { 0.0 } else { big }).collect();
    let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
    for y in 0..h {
        for x in 0..w {
            let n = y * w + x;
            let mut v = d[n];
            if x > 0 {
                v = v.min(d[n - 1] + a);
            }
            if y > 0 {
                v = v.min(d[n - w] + a);
                if x > 0 {
                    v = v.min(d[n - w - 1] + b);
                }
                if x + 1 < w {
                    v = v.min(d[n - w + 1] + b);
                }
            }
            d[n] = v;
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let n = y * w + x;
            let mut v = d[n];
            if x + 1 < w {
                v = v.min(d[n + 1] + a);
            }
            if y + 1 < h {
                v = v.min(d[n + w] + a);
                if x + 1 < w {
                    v = v.min(d[n + w + 1] + b);
                }
                if x > 0 {
                    v = v.min(d[n + w - 1] + b);
                }
            }
            d[n] = v;
        }
    }
    d
}

/// Bilinear sample of a node field at a point in cell units.
pub fn sample(field: &[f64], g: Grid, x: f64, y: f64) -> f64 {
    let fx = (x * g.k as f64 - 0.5).clamp(0.0, (g.w - 1) as f64);
    let fy = (y * g.k as f64 - 0.5).clamp(0.0, (g.h - 1) as f64);
    let (i, j) = ((fx as usize).min(g.w - 2), (fy as usize).min(g.h - 2));
    let (u, v) = (fx - i as f64, fy - j as f64);
    let n = j * g.w + i;
    mix(mix(field[n], field[n + 1], u), mix(field[n + g.w], field[n + g.w + 1], u), v)
}

// ---------------------------------------------------------------- the plan

/// A Gaussian patch: an ore body, a coalfield, a basin.
#[derive(Clone, Debug, PartialEq)]
pub struct Blob {
    pub x: f64,
    pub y: f64,
    /// Half-widths along and across `ang`, in cells.
    pub sx: f64,
    pub sy: f64,
    pub ang: f64,
    /// Richness at the centre.
    pub k: f64,
}

impl Blob {
    /// 1 at the centre, falling as a Gaussian.
    pub fn at(&self, x: f64, y: f64) -> f64 {
        let (ca, sa) = (self.ang.cos(), self.ang.sin());
        let (dx, dy) = (x - self.x, y - self.y);
        let u = (dx * ca + dy * sa) / self.sx;
        let v = (-dx * sa + dy * ca) / self.sy;
        (-(u * u + v * v)).exp()
    }
}

/// A tableland: a block of hard rock lying on soft, lifted whole.
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    pub blob: Blob,
    /// Height of its top, metres.
    pub height: f64,
    /// The top tilts: its height changes by this share from the centre to the edge, towards `dip_ang`.
    pub dip: f64,
    pub dip_ang: f64,
    /// Thickness of the hard cap, metres.
    pub cap: f64,
}

/// What the seed decides about an island before any water runs.
#[derive(Clone, Debug, PartialEq)]
pub struct Design {
    pub W: usize,
    pub H: usize,
    pub seeds: [u64; 12],
    /// The way the prevailing wind blows, a unit vector in (x, y).
    pub wind: (f64, f64),
    /// The mountain belt's line: three control points (x, y) of a quadratic curve, in cells.
    pub spine: [(f64, f64); 3],
    /// The belt's half-width, cells.
    pub belt_w: f64,
    /// Where along the line (0 to 1) the belt is broken by a pass.
    pub gaps: Vec<f64>,
    pub tables: Vec<Table>,
    /// Lowland basins of soft rock; the first may hold a lake.
    pub basins: Vec<Blob>,
    /// The first basin is scooped out into a large lake.
    pub lowland_lake: bool,
    /// How many upland lakes to look for.
    pub upland_lakes: usize,
    pub coal: Vec<Blob>,
    pub ore: Vec<Blob>,
    /// The lowland's strata run in bands across this direction, this far apart (cells).
    pub band_ang: f64,
    pub band_len: f64,
    /// The island's outline is where `shape` crosses this.
    pub shore: f64,
}

/// Rock at the surface, as `Relief::rock` names it.
pub const ROCK_BASEMENT: u8 = 0;
pub const ROCK_CAP: u8 = 1;
pub const ROCK_SOFT: u8 = 2;
pub const ROCK_BAND: u8 = 3;
pub const ROCK_VALE: u8 = 4;
pub const ROCK_COAL: u8 = 5;
pub const ROCK_ALLUVIUM: u8 = 6;
pub const ROCK_NAMES: [&str; 7] = ["basement", "cap", "soft", "band", "vale", "coal measures", "alluvium"];

/// Hardness against rivers (1 is ordinary rock; smaller is harder).
const K_CAP: f64 = 0.03;
const K_SOFT: f64 = 2.2;
const K_ALLUVIUM: f64 = 2.6;
/// Soft rock under a tableland's cap, metres.
const SOFT_THICK: f64 = 220.0;

impl Design {
    fn spine_at(&self, t: f64) -> (f64, f64) {
        let [p0, p1, p2] = self.spine;
        let (a, b, c) = ((1.0 - t) * (1.0 - t), 2.0 * (1.0 - t) * t, t * t);
        (a * p0.0 + b * p1.0 + c * p2.0, a * p0.1 + b * p1.1 + c * p2.1)
    }

    /// The unit normal to the belt's line at t (to its left going from start to end).
    fn spine_normal(&self, t: f64) -> (f64, f64) {
        let [p0, p1, p2] = self.spine;
        let tx = 2.0 * (1.0 - t) * (p1.0 - p0.0) + 2.0 * t * (p2.0 - p1.0);
        let ty = 2.0 * (1.0 - t) * (p1.1 - p0.1) + 2.0 * t * (p2.1 - p1.1);
        let l = tx.hypot(ty).max(1e-9);
        (-ty / l, tx / l)
    }

    /// A point `off` cells to one side of the belt's line at t, kept on the map.
    fn beside(&self, t: f64, off: f64) -> (f64, f64) {
        let (p, n) = (self.spine_at(t), self.spine_normal(t));
        ((p.0 + off * n.0).clamp(12.0, self.W as f64 - 12.0), (p.1 + off * n.1).clamp(10.0, self.H as f64 - 10.0))
    }

    /// The island's shape before the sea is let in: high in the middle, falling to
    /// the edges, with the fall bent by noise into bays and headlands.
    pub fn shape(&self, x: f64, y: f64) -> f64 {
        let (W, H) = (self.W as f64, self.H as f64);
        let wx = fbm01(x / 22.0, y / 22.0, self.seeds[0], 3, 0.5) - 0.5;
        let wy = fbm01(x / 22.0, y / 22.0, self.seeds[1], 3, 0.5) - 0.5;
        let a = (x / W + 0.3 * wx - 0.5) / 0.5;
        let b = (y / H + 0.3 * wy - 0.52) / 0.47;
        let fall = (a * a + b * b).sqrt();
        // the belt and the tablelands stand out to sea a little; the basins draw it in
        1.0 - fall.clamp(0.0, 1.4).powf(2.3) + 0.25 * (fbm01(x / 9.0, y / 9.0, self.seeds[2], 4, 0.55) - 0.5)
    }
}

fn seed_of(rng: &mut Rng) -> u64 {
    (random(rng) * (1u64 << 53) as f64) as u64
}

/// Draw an island's plan. All of the generator's draws that do not depend on the
/// finished land are made here, in a fixed order.
pub fn draw_design(rng: &mut Rng, W: usize, H: usize, P: &LandParams) -> Design {
    let (Wf, Hf) = (W as f64, H as f64);
    let mut seeds = [0u64; 12];
    for s in seeds.iter_mut() {
        *s = seed_of(rng);
    }
    // the wind is westerly, give or take: from the south-west round to the north-west
    let wa = uniform(rng, -0.7, 0.7);
    let wind = (wa.cos(), wa.sin());
    // the belt: the port's curved spine, which may lie the other way round and lean
    let mut p0 = (Wf * uniform(rng, 0.25, 0.4), Hf * uniform(rng, 0.12, 0.25));
    let mut p1 = (Wf * uniform(rng, 0.45, 0.7), Hf * uniform(rng, 0.4, 0.6));
    let mut p2 = (Wf * uniform(rng, 0.35, 0.6), Hf * uniform(rng, 0.75, 0.9));
    let mirror = random(rng) < 0.5;
    let lean = uniform(rng, -0.45, 0.45);
    for p in [&mut p0, &mut p1, &mut p2] {
        if mirror {
            p.0 = Wf - p.0;
        }
        // lean the line about the middle of the map, a cell being as tall as it is wide
        let (dx, dy) = (p.0 - 0.5 * Wf, p.1 - 0.5 * Hf);
        p.0 = (0.5 * Wf + dx * lean.cos() - dy * lean.sin()).clamp(0.15 * Wf, 0.85 * Wf);
        p.1 = (0.5 * Hf + dx * lean.sin() + dy * lean.cos()).clamp(0.1 * Hf, 0.9 * Hf);
    }
    let belt_w = 0.07 * Wf * uniform(rng, 0.8, 1.15);
    let mut d = Design {
        W,
        H,
        seeds,
        wind,
        spine: [p0, p1, p2],
        belt_w,
        gaps: Vec::new(),
        tables: Vec::new(),
        basins: Vec::new(),
        lowland_lake: false,
        upland_lakes: 1,
        coal: Vec::new(),
        ore: Vec::new(),
        band_ang: 0.0,
        band_len: 16.0,
        shore: 0.0,
    };
    let n_gaps = 1 + (random(rng) < 0.5) as usize;
    for g in 0..n_gaps {
        d.gaps.push(if g == 0 { uniform(rng, 0.3, 0.5) } else { uniform(rng, 0.6, 0.78) });
    }
    // tablelands and basins take turns on the two sides of the belt, apart along it
    let side = if random(rng) < 0.5 { 1.0 } else { -1.0 };
    let n_tables = 1 + (random(rng) < 0.5) as usize;
    for k in 0..n_tables {
        let t = if k == 0 { uniform(rng, 0.2, 0.45) } else { uniform(rng, 0.6, 0.85) };
        let s = if k == 0 { side } else { -side };
        let (x, y) = d.beside(t, s * Wf * uniform(rng, 0.17, 0.28));
        d.tables.push(Table {
            blob: Blob { x, y, sx: uniform(rng, 7.0, 12.0), sy: uniform(rng, 5.0, 9.0), ang: uniform(rng, 0.0, PI), k: 1.0 },
            height: uniform(rng, 180.0, 320.0),
            dip: uniform(rng, 0.2, 0.75),
            dip_ang: uniform(rng, 0.0, 2.0 * PI),
            cap: uniform(rng, 90.0, 150.0),
        });
    }
    let n_basins = 1 + (random(rng) < 0.6) as usize;
    for k in 0..n_basins {
        let t = if k == 0 { uniform(rng, 0.55, 0.85) } else { uniform(rng, 0.15, 0.4) };
        let s = if k == 0 { side } else { -side };
        let (x, y) = d.beside(t, s * Wf * uniform(rng, 0.15, 0.27));
        d.basins.push(Blob { x, y, sx: uniform(rng, 7.0, 11.0), sy: uniform(rng, 5.0, 8.0), ang: uniform(rng, 0.0, PI), k: 1.0 });
    }
    d.lowland_lake = random(rng) < P.lowland_lake;
    // (the draw is made whatever the number, so the stream stays in step)
    let more = P.upland_lakes.saturating_sub(1).max(1);
    d.upland_lakes = (2 + ((random(rng) * more as f64) as usize).min(more - 1)).min(P.upland_lakes);
    // coalfields on the belt's flanks, lying along it; ore in the belt itself
    for _ in 0..4 {
        let t = uniform(rng, 0.12, 0.88);
        let s = if random(rng) < 0.5 { 1.0 } else { -1.0 };
        let (x, y) = d.beside(t, s * Wf * uniform(rng, 0.075, 0.15));
        let n = d.spine_normal(t);
        let ang = n.0.atan2(-n.1) + uniform(rng, -0.4, 0.4);
        d.coal.push(Blob { x, y, sx: uniform(rng, 6.0, 9.0), sy: uniform(rng, 3.0, 4.5), ang, k: uniform(rng, 0.7, 1.0) });
    }
    for _ in 0..7 {
        let t = uniform(rng, 0.08, 0.92);
        let (x, y) = d.beside(t, Wf * uniform(rng, -0.075, 0.075));
        d.ore.push(Blob { x, y, sx: uniform(rng, 1.5, 3.5), sy: uniform(rng, 1.5, 3.0), ang: uniform(rng, 0.0, PI), k: uniform(rng, 0.55, 1.0) });
    }
    d.band_ang = uniform(rng, 0.0, PI);
    d.band_len = uniform(rng, 12.0, 22.0);
    // the shore: where the shape leaves `land_fraction` of the map (and a little
    // more, which the sea takes back at the end) above it. The outer two cells are sea.
    let mut v: Vec<f64> = Vec::with_capacity(W * H * 4);
    for j in 0..H * 2 {
        for i in 0..W * 2 {
            let (x, y) = ((i as f64 + 0.5) / 2.0, (j as f64 + 0.5) / 2.0);
            let edge = x < 2.0 || y < 2.0 || x > Wf - 2.0 || y > Hf - 2.0;
            v.push(if edge { -9.0 } else { d.shape(x, y) });
        }
    }
    v.sort_by(|a, b| a.total_cmp(b));
    let q = (1.0 - (P.land_fraction + DROWN)).clamp(0.0, 1.0);
    d.shore = v[((v.len() - 1) as f64 * q) as usize];
    d
}

/// Share of the map the sea takes back when it rises at the end, drowning the
/// lower valleys into estuaries.
const DROWN: f64 = 0.035;

/// The plan worked out at every node of one grid.
#[derive(Clone, Debug)]
pub struct Fields {
    pub g: Grid,
    /// Sea, fixed while the rivers work.
    pub sea: Vec<bool>,
    /// The surface the rivers start from, metres (below 0 in the sea).
    pub z0: Vec<f64>,
    /// Uplift over the whole run, metres.
    pub uplift: Vec<f64>,
    /// Hardness of the rock below any layers.
    pub kbase: Vec<f32>,
    /// Thickness of a hard cap at the top and of soft rock under it, metres.
    pub cap: Vec<f32>,
    pub soft: Vec<f32>,
    /// What the base rock is called (`ROCK_*`).
    pub base_rock: Vec<u8>,
    /// How far into a coalfield the node is (0 outside, to 1).
    pub coal: Vec<f32>,
    /// Distance to the sea, km (0 in it).
    pub coast_km: Vec<f32>,
}

impl Design {
    /// The plan at every node of a grid.
    pub fn fields(&self, g: Grid, P: &LandParams) -> Fields {
        let n = g.n();
        let (Wf, Hf) = (self.W as f64, self.H as f64);
        // land where the shape stands above the shore; one island, no holes
        let mut land: Vec<bool> = (0..n)
            .map(|i| {
                let (x, y) = g.xy(i);
                !(x < 2.0 || y < 2.0 || x > Wf - 2.0 || y > Hf - 2.0) && self.shape(x, y) > self.shore
            })
            .collect();
        let (lab, nl) = ndimage::label(&land, g.h, g.w);
        if nl > 1 {
            let mut sizes = vec![0usize; nl + 1];
            for &l in &lab {
                sizes[l as usize] += 1;
            }
            let keep = (1..=nl).max_by_key(|&l| (sizes[l], Reverse(l))).unwrap();
            land = lab.iter().map(|&l| l as usize == keep).collect();
        }
        let not: Vec<bool> = land.iter().map(|&l| !l).collect();
        let (lab, _) = ndimage::label(&not, g.h, g.w);
        let open = lab[0]; // the corner is open sea
        for i in 0..n {
            if !land[i] && lab[i] != open {
                land[i] = true;
            }
        }
        let sea: Vec<bool> = land.iter().map(|&l| !l).collect();
        let km = g.dx() / 1000.0;
        let coast_km: Vec<f32> = chamfer(&sea, g.w, g.h).iter().map(|&d| d * km as f32).collect();
        let off_km = chamfer(&land, g.w, g.h);

        // the belt's line, sampled, for each node's distance to it and place along it
        const NS: usize = 160;
        let line: Vec<(f64, f64)> = (0..NS).map(|k| self.spine_at(k as f64 / (NS - 1) as f64)).collect();
        let band = (self.band_ang.cos(), self.band_ang.sin());
        let (cx, cy) = (0.5 * Wf, 0.5 * Hf);

        let mut f = Fields {
            g,
            sea,
            z0: vec![0.0; n],
            uplift: vec![0.0; n],
            kbase: vec![1.0; n],
            cap: vec![0.0; n],
            soft: vec![0.0; n],
            base_rock: vec![ROCK_VALE; n],
            coal: vec![0.0; n],
            coast_km,
            };
        for i in 0..n {
            let (x, y) = g.xy(i);
            if f.sea[i] {
                // the sea floor falls away from the shore
                f.z0[i] = -(2.0 + 9.0 * off_km[i] as f64 * km).min(160.0);
                continue;
            }
            let (mut d2, mut tk) = (f64::INFINITY, 0usize);
            for (k, p) in line.iter().enumerate() {
                let v = (x - p.0) * (x - p.0) + (y - p.1) * (y - p.1);
                if v < d2 {
                    d2 = v;
                    tk = k;
                }
            }
            let (ds, t) = (d2.sqrt(), tk as f64 / (NS - 1) as f64);
            // massifs and passes along the belt
            let mut massif = 0.6 + 0.4 * (0.5 + 0.5 * perlin(t * 5.0, 0.37, self.seeds[3]));
            for &tg in &self.gaps {
                massif *= 1.0 - 0.6 * (-((t - tg) / 0.05) * ((t - tg) / 0.05)).exp();
            }
            let r = ds / self.belt_w;
            let belt = (-r * r).exp() * massif;
            let foot = (-(r / 2.3) * (r / 2.3)).exp();
            let mtn = belt * (0.7 + 0.6 * fbm01(x / 9.0, y / 9.0, self.seeds[4], 3, 0.5));
            let hills = 0.3 * foot * (0.3 + 0.7 * fbm01(x / 7.0, y / 7.0, self.seeds[5], 3, 0.5));
            let low = 0.03 + 0.13 * fbm01(x / 11.0, y / 11.0, self.seeds[6], 3, 0.5).powf(1.5);
            let mut basin = 0.0f64;
            for b in &self.basins {
                basin = basin.max(b.at(x, y));
            }
            // a plain along the coast, wide in the lee of the island and narrow or
            // missing where the wind comes ashore
            let (ox, oy) = (x - cx, (y - cy) * Wf / Hf);
            let lee = 0.5 + 0.5 * (ox * self.wind.0 + oy * self.wind.1) / ox.hypot(oy).max(1.0);
            let plain_km = 1.5 + 16.0 * lee.powf(1.5) * fbm01(x / 13.0, y / 13.0, self.seeds[7], 2, 0.5);
            let shore = 0.05 + 0.95 * smoothstep(0.0, plain_km, f.coast_km[i] as f64);
            let T = (0.8 * mtn + hills + low) * (1.0 - 0.9 * basin) * shore;

            // tablelands
            let (mut tm, mut th, mut cap) = (0.0f64, 0.0f64, 0.0f64);
            for tb in &self.tables {
                let b = &tb.blob;
                let (ca, sa) = (b.ang.cos(), b.ang.sin());
                let (u, v) = (((x - b.x) * ca + (y - b.y) * sa) / b.sx, (-(x - b.x) * sa + (y - b.y) * ca) / b.sy);
                // an outline of bays and spurs, not an oval
                let rr = u.hypot(v) + 0.9 * (fbm01(x / 9.0, y / 9.0, self.seeds[8], 4, 0.55) - 0.5);
                let m = 1.0 - smoothstep(0.9, 1.06, rr);
                if m > tm {
                    tm = m;
                    // the beds dip: a high scarp on one side, a long back slope on the other
                    let tilt = 1.0 + tb.dip * (u * tb.dip_ang.cos() + v * tb.dip_ang.sin()).clamp(-1.0, 1.0);
                    // the top rolls a little
                    th = tb.height * tilt * (0.94 + 0.12 * fbm01(x / 3.0, y / 3.0, self.seeds[8] ^ 0x7AB, 3, 0.5));
                    cap = tb.cap;
                }
            }
            // a tableland ends at the mountains' foot and does not reach the beach
            let tm = tm * (1.0 - smoothstep(0.25, 0.6, belt)) * smoothstep(0.5, 3.0, f.coast_km[i] as f64);

            let rough = 5.0 * fbm01(x / 1.3, y / 1.3, self.seeds[9], 2, 0.5) + 1.5 * hash01(i as i64, 0, self.seeds[9]);
            // the starting surface is uneven at every scale, so that the first
            // streams gather into trees and do not run side by side down a ramp
            let grain = 0.55 + 0.9 * fbm01(x / 2.5, y / 2.5, self.seeds[9] ^ 0x51, 4, 0.6);
            f.z0[i] = 1.0 + 0.45 * P.relief * T * grain + th * tm + rough;
            f.uplift[i] = 1.15 * P.relief * T * (0.8 + 0.4 * fbm01(x / 4.0, y / 4.0, self.seeds[5] ^ 0x77, 3, 0.5)) + 0.12 * th * tm;

            // rock: hard basement under the belt; in the lowland, strata in bands,
            // hard ridge and soft vale by turns; soft in the basins; the coal measures
            let s = (2.0 * PI * (x * band.0 + y * band.1) / self.band_len + 5.0 * fbm01(x / 14.0, y / 14.0, self.seeds[10], 2, 0.5)).sin();
            let ridge = smoothstep(0.35, 0.6, s);
            let k_low = mix(1.9, 0.55, ridge);
            let k_mtn = 0.42 * (0.75 + 0.5 * fbm01(x / 5.0, y / 5.0, self.seeds[11], 2, 0.5));
            let wm = smoothstep(0.12, 0.5, belt + 0.45 * foot);
            let mut k = mix(k_low, k_mtn, wm);
            let mut rock = if wm > 0.5 {
                ROCK_BASEMENT
            } else if ridge > 0.5 {
                ROCK_BAND
            } else {
                ROCK_VALE
            };
            if basin > 0.3 {
                k = mix(k, 2.1, smoothstep(0.3, 0.7, basin));
                rock = ROCK_VALE;
            }
            let mut cw = 0.0f64;
            for b in &self.coal {
                cw = cw.max(b.k * b.at(x, y));
            }
            if cw > 0.18 {
                k = mix(k, 1.3, ((cw - 0.18) * 4.0).min(1.0));
                rock = ROCK_COAL;
                f.coal[i] = cw as f32;
            }
            f.kbase[i] = k as f32;
            f.base_rock[i] = rock;
            if tm > 0.5 {
                f.cap[i] = cap as f32;
            }
            f.soft[i] = (SOFT_THICK * tm) as f32;
        }
        f
    }
}

// ---------------------------------------------------------------- erosion

/// Water running off a surface: where each node drains, in what order, and how
/// much land drains through it.
#[derive(Clone, Debug, Default)]
pub struct Routing {
    /// The surface with its hollows filled to their spill points, plus a hair of
    /// slope so that every node has somewhere lower to drain to.
    pub fill: Vec<f64>,
    /// The node each land node drains to.
    pub rcv: Vec<u32>,
    /// Land nodes, each after the node it drains to (the sea's edge first).
    pub order: Vec<u32>,
    /// Nodes draining through each node, itself included.
    pub area: Vec<f64>,
    done: Vec<bool>,
    heap: BinaryHeap<Reverse<(u64, u32)>>,
}

/// Fall given to a filled hollow per node spacing, metres: enough to order the
/// nodes, too little to see.
const EPS: f64 = 2e-4;

impl Routing {
    /// Route water over `z`. `sea` nodes are the base level; `shore` lists the
    /// sea nodes that touch land. A priority flood from the shore inwards (Barnes
    /// and others, 2014), then steepest descent on the filled surface.
    pub fn route(&mut self, g: Grid, z: &[f64], sea: &[bool], shore: &[u32]) {
        let n = g.n();
        if self.fill.len() != n {
            self.fill = vec![0.0; n];
            self.rcv = vec![0; n];
            self.area = vec![0.0; n];
            self.done = vec![false; n];
        }
        self.order.clear();
        self.heap.clear();
        self.done.copy_from_slice(sea);
        let off = g.offsets();
        for &s in shore {
            self.fill[s as usize] = z[s as usize].min(0.0);
            self.heap.push(Reverse((ord_key(self.fill[s as usize]), s)));
        }
        while let Some(Reverse((_, c))) = self.heap.pop() {
            let c = c as usize;
            if !sea[c] {
                self.order.push(c as u32);
            }
            let fc = self.fill[c].max(0.0);
            for k in 0..8 {
                let nb = (c as isize + off[k]) as usize;
                if !self.done[nb] {
                    self.done[nb] = true;
                    let v = z[nb].max(fc + EPS * N8[k].2);
                    self.fill[nb] = v;
                    self.heap.push(Reverse((ord_key(v), nb as u32)));
                }
            }
        }
        for &c in &self.order {
            let c = c as usize;
            let fc = self.fill[c];
            let (mut best, mut br) = (0.0, c);
            for k in 0..8 {
                let nb = (c as isize + off[k]) as usize;
                let fnb = if sea[nb] { 0.0f64.min(fc - EPS) } else { self.fill[nb] };
                let s = (fc - fnb) / N8[k].2;
                if s > best {
                    best = s;
                    br = nb;
                }
            }
            self.rcv[c] = br as u32;
            self.area[c] = 1.0;
        }
        for &c in self.order.iter().rev() {
            let r = self.rcv[c as usize] as usize;
            if !sea[r] {
                self.area[r] += self.area[c as usize];
            }
        }
    }
}

/// The sea nodes that touch land (the nodes a flood starts from).
pub fn shore_nodes(g: Grid, sea: &[bool]) -> Vec<u32> {
    let off = g.offsets();
    let mut out = Vec::new();
    for j in 1..g.h - 1 {
        for i in 1..g.w - 1 {
            let c = j * g.w + i;
            if sea[c] && off.iter().any(|&o| !sea[(c as isize + o) as usize]) {
                out.push(c as u32);
            }
        }
    }
    out
}

/// The land being worn down: the surface, the rock under it, and what the
/// rivers have laid on it.
pub struct Eroder {
    pub g: Grid,
    pub f: Fields,
    /// The surface, metres above the sea.
    pub z: Vec<f64>,
    /// Where the starting surface would stand now had nothing been worn off it.
    pub zr: Vec<f64>,
    /// Thickness of river-laid sediment at the surface, metres.
    pub sed: Vec<f64>,
    pub route: Routing,
    shore: Vec<u32>,
    /// What each node lost in the last step (metres; negative where it gained).
    ero: Vec<f64>,
    qs: Vec<f64>,
    tmp: Vec<f64>,
    /// Uplift per step.
    up: Vec<f64>,
}

impl Eroder {
    pub fn new(f: Fields, steps_total: usize) -> Eroder {
        let g = f.g;
        let n = g.n();
        let shore = shore_nodes(g, &f.sea);
        let up = f.uplift.iter().map(|u| u / steps_total.max(1) as f64).collect();
        Eroder {
            g,
            z: f.z0.clone(),
            zr: f.z0.clone(),
            sed: vec![0.0; n],
            route: Routing::default(),
            shore,
            ero: vec![0.0; n],
            qs: vec![0.0; n],
            tmp: vec![0.0; n],
            up,
            f,
        }
    }

    /// How easily a river cuts the rock now at the surface of node c, and its name.
    pub fn rock(&self, c: usize) -> (f64, u8) {
        if self.sed[c] > 1.0 {
            return (K_ALLUVIUM, ROCK_ALLUVIUM);
        }
        let worn = self.zr[c] - (self.z[c] - self.sed[c]);
        let cap = self.f.cap[c] as f64;
        if worn < cap {
            (K_CAP, ROCK_CAP)
        } else if worn < cap + self.f.soft[c] as f64 {
            (K_SOFT, ROCK_SOFT)
        } else {
            (self.f.kbase[c] as f64, self.f.base_rock[c])
        }
    }

    /// One step: lift the land, route the water, let every river cut down towards
    /// the node below it and drop part of its load, let steep ground slide and
    /// soil creep.
    ///
    /// The cutting is the stream-power law, dz/dt = U - K sqrt(A) S, solved
    /// implicitly from the sea upwards along the drainage (Braun and Willett,
    /// 2013), so a step may be long. Deposition is the upstream load's term of
    /// Yuan and others (2019), taken from the step before.
    pub fn step(&mut self, P: &LandParams) {
        let g = self.g;
        self.route.route(g, &self.z, &self.f.sea, &self.shore);
        let dx = g.dx();
        let node_km2 = dx * dx / 1e6;
        let w = g.w;
        // the load: what was worn off upstream in the last step
        for &c in &self.route.order {
            self.qs[c as usize] = self.ero[c as usize];
        }
        for &c in self.route.order.iter().rev() {
            let r = self.route.rcv[c as usize] as usize;
            if !self.f.sea[r] {
                self.qs[r] += self.qs[c as usize];
            }
        }
        for idx in 0..self.route.order.len() {
            let c = self.route.order[idx] as usize;
            let r = self.route.rcv[c] as usize;
            let before = self.z[c];
            let up = self.up[c];
            // a hollow fills with what the water brings
            let lifted = before.max(self.route.fill[c]) + up;
            let base = if self.f.sea[r] { 0.0 } else { self.z[r] };
            let diag = (c % w != r % w) && (c / w != r / w);
            let dist = if diag { dx * std::f64::consts::SQRT_2 } else { dx };
            let area = self.route.area[c];
            let (k, _) = self.rock(c);
            let fcut = P.erodibility * k * (area * node_km2).sqrt() * (500.0 / dist);
            let mut zn = (lifted + fcut * base) / (1.0 + fcut);
            // a river drops load only where it runs slack, and builds its bed no
            // steeper than it can carry the load over: a fan at the mountain
            // front, a flood plain below. Big rivers carry on gentler slopes.
            let drop = P.deposition * self.qs[c].max(0.0) / area;
            let carry = base + (0.02 / (area * node_km2).sqrt()).clamp(0.0004, 0.03) * dist;
            if drop > 0.0 && zn < carry {
                zn = (zn + drop / (1.0 + fcut)).min(carry);
            }
            let steepest = base + P.max_slope * dist;
            if zn > steepest {
                zn = steepest;
            }
            self.z[c] = zn;
            self.zr[c] += up;
            let gain = zn - (before + up);
            self.ero[c] = -gain;
            self.sed[c] = (self.sed[c] + gain).max(0.0);
        }
        // creep on the hillsides (not in the channels, which it would fill). Soil
        // creeps; a cap of hard rock hardly does, which is what keeps a scarp sharp.
        if P.creep > 0.0 {
            self.tmp.copy_from_slice(&self.z);
            for idx in 0..self.route.order.len() {
                let c = self.route.order[idx] as usize;
                if self.route.area[c] <= 3.0 {
                    let side = |nb: usize| if self.f.sea[nb] { 0.0 } else { self.tmp[nb] };
                    let mean = 0.25 * (side(c - 1) + side(c + 1) + side(c - w) + side(c + w));
                    let soft = self.rock(c).0.min(1.0);
                    self.z[c] += P.creep * soft * (mean - self.tmp[c]);
                }
            }
        }
    }

    /// The same land on a finer grid: heights, rock and sediment carried over,
    /// with a little roughness for the small valleys to start from.
    pub fn refine(&self, f: Fields, steps_total: usize, seed: u64) -> Eroder {
        let g = f.g;
        let land_z: Vec<f64> = (0..self.g.n()).map(|c| self.z[c].max(0.0)).collect();
        let mut e = Eroder::new(f, steps_total);
        for c in 0..g.n() {
            if e.f.sea[c] {
                continue;
            }
            let (x, y) = g.xy(c);
            let rough = 2.5 * (hash01(c as i64, 1, seed) - 0.5);
            e.z[c] = (sample(&land_z, self.g, x, y) + rough).max(0.3);
            e.zr[c] = sample(&self.zr, self.g, x, y).max(e.z[c]);
            e.sed[c] = sample(&self.sed, self.g, x, y);
        }
        e
    }
}

/// The finished land, before its water: `landwater` takes it from here.
#[derive(Clone, Debug, Default)]
pub struct Relief {
    pub k: usize,
    pub w: usize,
    pub h: usize,
    /// Height in metres above the sea (below 0 under it).
    pub z: Vec<f64>,
    /// River-laid sediment at the surface, metres.
    pub sed: Vec<f64>,
    /// How much rock has been worn off each node, metres.
    pub worn: Vec<f64>,
    /// Rock at the surface (`ROCK_*`) and its hardness against rivers.
    pub rock: Vec<u8>,
    pub khard: Vec<f32>,
    /// How far into a coalfield each node is.
    pub coal: Vec<f32>,
}

/// Lift and wear down an island from its plan: first on a grid of half the
/// fineness, where the trunk valleys and the watersheds are settled, then on the
/// fine grid.
pub fn build_relief(d: &Design, P: &LandParams) -> Relief {
    let (W, H) = (d.W, d.H);
    let k = P.fine.clamp(2, 8) & !1;
    let total = (P.coarse_steps + P.fine_steps).max(1);
    let coarse = Grid::new(W, H, (k / 2).max(2).min(k));
    let mut e = Eroder::new(d.fields(coarse, P), total);
    for _ in 0..P.coarse_steps {
        e.step(P);
    }
    if coarse.k != k {
        let fine = Grid::new(W, H, k);
        e = e.refine(d.fields(fine, P), total, d.seeds[9]);
    }
    for _ in 0..P.fine_steps {
        e.step(P);
    }
    let n = e.g.n();
    let mut out = Relief { k, w: e.g.w, h: e.g.h, z: Vec::new(), sed: Vec::new(), worn: vec![0.0; n], rock: vec![0; n], khard: vec![1.0; n], coal: Vec::new() };
    for c in 0..n {
        if !e.f.sea[c] {
            let (kk, r) = e.rock(c);
            out.rock[c] = r;
            out.khard[c] = kk as f32;
            out.worn[c] = e.zr[c] - (e.z[c] - e.sed[c]);
        }
    }
    out.coal = std::mem::take(&mut e.f.coal);
    out.z = e.z;
    out.sed = e.sed;
    out
}
