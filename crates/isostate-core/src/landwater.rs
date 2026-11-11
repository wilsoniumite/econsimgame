//! Land (see `landform.rs` and docs/rules/land.md): the water on the finished land.
//!
//! In order: lakes are scooped where ice would have left them (ribbon lakes in
//! upland valleys, now and then a broad one in a lowland basin); the sea rises a
//! little and drowns the lower valleys into estuaries; the big rivers plane their
//! valley floors flat; rain falls by height and by the side the wind comes from;
//! a monthly water balance turns rain into runoff; runoff gathers down the
//! drainage into a flow for every reach; hollows fill to their spill points and
//! the big ones stay as lakes; and the rivers are traced as lines, the lowland
//! reaches wandering by a model of bank erosion until they loop and cut off.

use std::f64::consts::PI;

use crate::landform::{CELL_M, Design, Grid, LandParams, Relief, Routing, fbm01, hash01, sample, shore_nodes};
use crate::rng::{Rng, integers, uniform};

/// Seconds in a year, for turning metres of runoff a year into cubic metres a second.
const YEAR_S: f64 = 3.156e7;

/// What a river runs into.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum RiverEnd {
    #[default]
    Sea,
    /// A lake, by its place in `lakes`.
    Lake(usize),
    /// A bigger river, by its place in `rivers`.
    River(usize),
}

/// A river's course: a line of points from the source down, in cell units (cell
/// (x, y) covers x..x+1 by y..y+1), with what the river is like at each.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize)]
pub struct River {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    /// Width of the water, metres.
    pub width: Vec<f32>,
    /// Height of the water above the sea, metres; it never rises downstream.
    pub level: Vec<f32>,
    /// Mean flow, cubic metres a second.
    pub flow: Vec<f32>,
    /// The river runs in several threads over gravel here.
    pub braided: Vec<bool>,
    pub end: RiverEnd,
}

/// A cut-off loop of an old course, left as a crescent of still water.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize)]
pub struct Oxbow {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub width: f32,
    pub level: f32,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize)]
pub struct Lake {
    /// Height of the water above the sea, metres.
    pub level: f32,
    pub area_km2: f32,
    /// Greatest depth, metres.
    pub depth: f32,
    /// Its middle, cell units.
    pub x: f32,
    pub y: f32,
    /// Where the water leaves it, cell units.
    pub outlet: (f32, f32),
    /// Mean flow out of it, cubic metres a second.
    pub outflow: f32,
}

/// A fine node's ground by the water: dry, flood plain, or marsh and fen.
pub const WET_DRY: u8 = 0;
pub const WET_PLAIN: u8 = 1;
pub const WET_MARSH: u8 = 2;

/// The land with its water, at the fine grid's scale.
#[derive(Clone, Debug, Default)]
pub struct Water {
    pub k: usize,
    pub w: usize,
    pub h: usize,
    /// Height in metres above the sea (below 0 under it); a lake's bed under a lake.
    pub z: Vec<f64>,
    /// Open sea.
    pub sea: Vec<bool>,
    pub rt: Routing,
    /// 0, or the lake a node lies under plus 1.
    pub lake_id: Vec<u16>,
    pub lakes: Vec<Lake>,
    /// Mean flow through each node, cubic metres a second.
    pub flow: Vec<f64>,
    /// Height of each node above the water it drains to, metres.
    pub hand: Vec<f32>,
    /// `WET_*`.
    pub wet: Vec<u8>,
    /// The river line through each node, or -1.
    pub owner: Vec<i32>,
    pub rivers: Vec<River>,
    pub oxbows: Vec<Oxbow>,
    /// By cell: rain, mm a year, and runoff, mm in each month (`runoff[m * N + c]`).
    pub rain: Vec<f64>,
    pub runoff: Vec<f32>,
    /// How far the sea rose at the end, metres.
    pub drowned: f64,
}

impl Water {
    pub fn grid(&self) -> Grid {
        Grid { w: self.w, h: self.h, k: self.k }
    }
}

/// Open sea: everything at or below sea level that the corner of the map reaches
/// (by all eight neighbours, as water runs, so a river's last nodes below the
/// tide are its estuary and not a string of ponds).
fn open_sea(g: Grid, z: &[f64]) -> Vec<bool> {
    let mut sea = vec![false; g.n()];
    let mut stack = vec![0usize];
    sea[0] = true;
    while let Some(c) = stack.pop() {
        let (x, y) = (c % g.w, c / g.w);
        let mut go = |nb: usize| {
            if !sea[nb] && z[nb] <= 0.0 {
                sea[nb] = true;
                stack.push(nb);
            }
        };
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                let (nx, ny) = (x as isize + dx, y as isize + dy);
                if nx >= 0 && ny >= 0 && (nx as usize) < g.w && (ny as usize) < g.h {
                    go(ny as usize * g.w + nx as usize);
                }
            }
        }
    }
    sea
}

/// Scoop the lakes. An upland lake is a stretch of valley deepened along the
/// stream, as a glacier leaves it: the water stands behind the lip at its lower
/// end. A lowland lake is a broad hollow in the first basin.
fn carve_lakes(d: &Design, g: Grid, z: &mut [f64], rt: &Routing, sea: &[bool], rng: &mut Rng) {
    let dx = g.dx();
    let node_km2 = dx * dx / 1e6;
    let z0 = z.to_vec();
    let mut cand: Vec<usize> = Vec::new();
    for &c in &rt.order {
        let c = c as usize;
        let a = rt.area[c] * node_km2;
        let r = rt.rcv[c] as usize;
        if (12.0..=150.0).contains(&a) && (100.0..=520.0).contains(&z0[c]) && !sea[r] && (z0[c] - z0[r]) / dx < 0.02 {
            cand.push(c);
        }
    }
    cand.sort_unstable();
    let mut chosen: Vec<(f64, f64)> = Vec::new();
    for _ in 0..200 {
        // the draws are made whether or not a site is found, so the stream stays in step
        let pick = integers(rng, cand.len().max(1));
        let (len_km, depth, rad_km) = (uniform(rng, 4.0, 11.0), uniform(rng, 15.0, 45.0), uniform(rng, 0.7, 1.3));
        if chosen.len() >= d.upland_lakes || cand.is_empty() {
            continue;
        }
        let c = cand[pick];
        let (x, y) = g.xy(c);
        if chosen.iter().any(|&(px, py)| (px - x).hypot(py - y) < 14.0) {
            continue;
        }
        chosen.push((x, y));
        let n_path = (len_km * 1000.0 / dx) as usize;
        let mut path = vec![c];
        let mut cur = c;
        // the trough runs down the valley while the valley falls gently: ice
        // deepens a reach, it does not level a staircase
        while path.len() < n_path {
            let r = rt.rcv[cur] as usize;
            if sea[r] || z0[r] < 60.0 || z0[c] - z0[r] > 60.0 {
                break;
            }
            path.push(r);
            cur = r;
        }
        if path.len() < 4 {
            chosen.pop();
            continue;
        }
        // its floor lies below the lip at its lower end, so the water stands the whole length
        let lip = z0[*path.last().unwrap()];
        for (s, &p) in path.iter().enumerate() {
            let prof = (PI * (s as f64 + 0.5) / path.len() as f64).sin();
            let deep = z0[p] - lip + depth * prof.powf(0.7);
            let rad = (0.45 + 0.75 * prof) * rad_km * 1000.0 / dx;
            let (pi, pj) = ((p % g.w) as isize, (p / g.w) as isize);
            let rr = rad.ceil() as isize;
            for dj in -rr..=rr {
                for di in -rr..=rr {
                    let dist = ((di * di + dj * dj) as f64).sqrt() / rad;
                    if dist < 1.0 {
                        let nb = ((pj + dj) * g.w as isize + pi + di) as usize;
                        if !sea[nb] {
                            z[nb] = z[nb].min(z0[p] - deep * (1.0 - dist * dist));
                        }
                    }
                }
            }
        }
    }
    let deep = uniform(rng, 10.0, 22.0);
    if d.lowland_lake {
        let b = &d.basins[0];
        // the lake sits on the basin's main river
        let (mut best, mut cn) = (0.0, usize::MAX);
        for &c in &rt.order {
            let (x, y) = g.xy(c as usize);
            if (x - b.x).abs() < 3.0 && (y - b.y).abs() < 3.0 && rt.area[c as usize] > best {
                best = rt.area[c as usize];
                cn = c as usize;
            }
        }
        if cn != usize::MAX {
            let zc = z0[cn];
            let (cx, cy) = g.xy(cn);
            let (ca, sa) = (b.ang.cos(), b.ang.sin());
            for c in 0..g.n() {
                if sea[c] {
                    continue;
                }
                let (x, y) = g.xy(c);
                let (u, v) = (((x - cx) * ca + (y - cy) * sa) / (0.5 * b.sx), (-(x - cx) * sa + (y - cy) * ca) / (0.5 * b.sy));
                let rr = u.hypot(v) + 0.7 * (fbm01(x / 5.0, y / 5.0, d.seeds[8] ^ 0x1A4E, 3, 0.55) - 0.5);
                if rr < 1.0 {
                    z[c] = z[c].min(zc - deep * (1.0 - rr * rr));
                }
            }
        }
    }
}

/// Let the sea rise until `P.land_fraction` of the cells are land (a cell is land
/// when at least half its nodes are dry), and return how far it rose.
fn drown(g: Grid, z: &mut [f64], lake: &[bool], P: &LandParams) -> f64 {
    let k = g.k;
    let (W, H) = (g.w / k, g.h / k);
    let target = (P.land_fraction * (W * H) as f64).round() as usize;
    let count = |s: f64| -> usize {
        let mut dry = vec![0u16; W * H];
        for c in 0..g.n() {
            if z[c] > s || lake[c] {
                dry[g.cell(c)] += 1;
            }
        }
        dry.iter().filter(|&&d| 2 * d as usize >= k * k).count()
    };
    let (mut lo, mut hi) = (0.0, 60.0);
    if count(lo) <= target {
        return 0.0;
    }
    for _ in 0..24 {
        let mid = 0.5 * (lo + hi);
        if count(mid) > target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    for v in z.iter_mut() {
        *v -= hi;
    }
    hi
}

/// Rain by cell, mm a year. Air comes in off the sea with the wind, drops more
/// as it is pushed up and over high ground, and is drier beyond it: a wet side
/// and a rain shadow. Worked a column at a time from the windward edge (the
/// wind is always westerly, so each column only needs the two before it).
fn rainfall(d: &Design, P: &LandParams, hc: &[f64], landc: &[bool]) -> Vec<f64> {
    let (W, H) = (d.W, d.H);
    // the air feels the hills smoothed over a few cells
    let mut hs = vec![0.0; W * H];
    for y in 0..H {
        for x in 0..W {
            let (mut s, mut n) = (0.0, 0.0);
            for yy in y.saturating_sub(2)..(y + 3).min(H) {
                for xx in x.saturating_sub(2)..(x + 3).min(W) {
                    s += hc[yy * W + xx];
                    n += 1.0;
                }
            }
            hs[y * W + x] = s / n;
        }
    }
    let at = |f: &[f64], x: f64, y: f64| -> f64 {
        let (fx, fy) = (x.clamp(0.0, (W - 1) as f64), y.clamp(0.0, (H - 1) as f64));
        let (i, j) = ((fx as usize).min(W - 2), (fy as usize).min(H - 2));
        let (u, v) = (fx - i as f64, fy - j as f64);
        let n = j * W + i;
        (f[n] * (1.0 - u) + f[n + 1] * u) * (1.0 - v) + (f[n + W] * (1.0 - u) + f[n + W + 1] * u) * v
    };
    let mut wet = vec![1.0; W * H];
    let mut rain = vec![0.0; W * H];
    let (wx, wy) = d.wind;
    for x in 0..W {
        for y in 0..H {
            let c = y * W + x;
            let (ux, uy) = (x as f64 - 1.5 * wx, y as f64 - 1.5 * wy);
            let m = if x < 2 { 1.0 } else { at(&wet, ux, uy) };
            let rise = (hs[c] - at(&hs, ux, uy)) / 1.5;
            let lift = 0.014 * rise.max(0.0) + 1.1 * hs[c] / 1000.0;
            // air coming down the lee side dries
            let lee = 1.0 - 0.3 * (-rise / 60.0).clamp(0.0, 1.0);
            rain[c] = P.rain * m * (1.0 + lift) * lee;
            // rain takes the water out of the air; the sea puts some back
            let mut m2 = m * (1.0 - 0.0025 - 0.02 * lift);
            if !landc[c] {
                m2 += 0.05 * (1.0 - m2);
            }
            wet[c] = m2;
        }
    }
    rain
}

/// Runoff by cell and month, mm, from a year of rain: winter is wetter, summer
/// takes more back, the soil holds some and lets the rest go at once or slowly.
/// Returns `runoff[m * N + c]`.
pub fn runoff_months(rain: &[f64], hc: &[f64]) -> Vec<f32> {
    let n = rain.len();
    let mut out = vec![0.0f32; 12 * n];
    let pet_w: Vec<f64> = (0..12).map(|m| (1.0 + 1.1 * (2.0 * PI * (m as f64 - 6.0) / 12.0).cos()).max(0.0)).collect();
    let pet_sum: f64 = pet_w.iter().sum();
    for c in 0..n {
        let pet = (540.0 - 0.3 * hc[c]).max(260.0);
        let (mut soil, mut ground) = (100.0, 60.0);
        for year in 0..4 {
            for m in 0..12 {
                let r = rain[c] * (1.0 + 0.28 * (2.0 * PI * (m as f64 + 0.5) / 12.0).cos()) / 12.0;
                let have = soil + r;
                let et = (pet * pet_w[m] / pet_sum).min(have);
                soil = have - et;
                let spill = (soil - 100.0).max(0.0);
                soil -= spill;
                ground += 0.5 * spill;
                let base = 0.3 * ground;
                ground -= base;
                if year == 3 {
                    out[m * n + c] = (0.5 * spill + base) as f32;
                }
            }
        }
    }
    out
}

/// The points of one river as it is traced and moved: metres, with each point's
/// place along the first tracing so that its level and flow can be looked up.
struct Course {
    p: Vec<(f64, f64)>,
    u: Vec<f64>,
    lev: Vec<f64>,
    q: Vec<f64>,
    sl: Vec<f64>,
    sed: Vec<f64>,
}

fn lerp_at(a: &[f64], u: f64) -> f64 {
    let i = (u as usize).min(a.len() - 1);
    let j = (i + 1).min(a.len() - 1);
    a[i] + (a[j] - a[i]) * (u - i as f64).clamp(0.0, 1.0)
}

impl Course {
    /// Points every `ds` metres along the line, the two ends kept.
    fn resample(&mut self, ds: f64) {
        let n = self.p.len();
        if n < 3 {
            return;
        }
        let (mut p, mut u) = (vec![self.p[0]], vec![self.u[0]]);
        let mut need = ds;
        for i in 1..n {
            let (a, b) = (self.p[i - 1], self.p[i]);
            let seg = (b.0 - a.0).hypot(b.1 - a.1);
            let mut at = 0.0;
            while seg - at >= need {
                at += need;
                let t = at / seg;
                p.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
                u.push(self.u[i - 1] + (self.u[i] - self.u[i - 1]) * t);
                need = ds;
            }
            need -= seg - at;
        }
        let last = *p.last().unwrap();
        if (last.0 - self.p[n - 1].0).hypot(last.1 - self.p[n - 1].1) < 0.4 * ds && p.len() > 1 {
            p.pop();
            u.pop();
        }
        p.push(self.p[n - 1]);
        u.push(self.u[n - 1]);
        self.p = p;
        self.u = u;
    }

    fn smooth(&mut self, passes: usize) {
        for _ in 0..passes {
            let old = self.p.clone();
            for i in 1..old.len().saturating_sub(1) {
                self.p[i] = (0.25 * old[i - 1].0 + 0.5 * old[i].0 + 0.25 * old[i + 1].0, 0.25 * old[i - 1].1 + 0.5 * old[i].1 + 0.25 * old[i + 1].1);
            }
        }
    }
}

/// Width of a river's water in metres from its mean flow. Wider than the
/// textbook's hydraulic geometry (about 4.5 sqrt Q) by a factor of two and a
/// half, so that a river and its bends can be seen on a map of 2 km cells.
pub fn river_width(q: f64) -> f64 {
    11.0 * q.max(0.0).sqrt()
}

/// Meander migration on one course (Howard and Knutson, 1984): each point's bank
/// erodes at a rate set by the curvature there and, more strongly, by the
/// curvature upstream of it, so bends grow, travel downstream, and at last touch
/// and cut themselves off. Steep reaches and small streams are left alone, and a
/// bend cannot climb the valley side: the ground must lie near the water's level.
fn meander(c: &mut Course, g: Grid, z: &[f64], steps: usize, ds: f64, oxbows: &mut Vec<Oxbow>) {
    let mobile = |q: f64, sl: f64| -> f64 { if q < 1.0 { 0.0 } else { (1.0 - sl / 0.004).clamp(0.0, 1.0) } };
    let (mut r0, mut w, mut mob, mut fade) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    // the stretch of the course that can move, found again whenever the points are re-laid
    let (mut lo, mut hi, mut fresh) = (0usize, 0usize, true);
    for step in 0..steps {
        let n = c.p.len();
        if n < 8 {
            break;
        }
        if fresh {
            w.clear();
            mob.clear();
            fade.clear();
            (lo, hi) = (n, 0);
            for i in 0..n {
                let (q, sl) = (lerp_at(&c.q, c.u[i]), lerp_at(&c.sl, c.u[i]));
                let wi = river_width(q);
                let m = mobile(q, sl) * ((i.min(n - 1 - i)) as f64 / 4.0).min(1.0);
                if m > 0.0 {
                    lo = lo.min(i);
                    hi = i + 1;
                }
                w.push(wi);
                mob.push(m);
                // the pull of a bend upstream fades over a width and a half
                fade.push((-ds / (1.5 * wi.max(1.0))).exp());
            }
            if hi == 0 {
                return;
            }
            lo = lo.saturating_sub(7).max(1);
            hi = hi.min(n - 1);
            fresh = false;
        }
        r0.clear();
        r0.resize(n, 0.0);
        for i in lo..hi {
            let (a, b) = ((c.p[i].0 - c.p[i - 1].0, c.p[i].1 - c.p[i - 1].1), (c.p[i + 1].0 - c.p[i].0, c.p[i + 1].1 - c.p[i].1));
            let (la, lb) = (a.0.hypot(a.1).max(1.0), b.0.hypot(b.1).max(1.0));
            // curvature times width: how tight the bend is for a river of this size
            r0[i] = (a.0 * b.1 - a.1 * b.0) / (la * lb) / (0.5 * (la + lb)) * w[i];
        }
        let old = c.p.clone();
        for i in lo..hi {
            if mob[i] <= 0.0 {
                continue;
            }
            let (mut acc, mut tot, mut gk) = (0.0, 0.0, 1.0);
            let mut j = i;
            while j > 1 && i - j < 6 {
                j -= 1;
                gk *= fade[i];
                acc += r0[j] * gk;
                tot += gk;
            }
            let r1 = -r0[i] + if tot > 0.0 { 2.5 * acc / tot } else { 0.0 };
            let (tx, ty) = (old[i + 1].0 - old[i - 1].0, old[i + 1].1 - old[i - 1].1);
            let tl = tx.hypot(ty).max(1e-6);
            // a left-hand bend eats into its right bank
            let rate = (0.3 * w[i] * r1 * mob[i]).clamp(-0.3 * ds, 0.3 * ds);
            let cand = (old[i].0 + ty / tl * rate, old[i].1 - tx / tl * rate);
            let bank = sample(z, g, cand.0 / CELL_M, cand.1 / CELL_M) - lerp_at(&c.lev, c.u[i]);
            let room = 1.0 - ((bank - 3.0) / 5.0).clamp(0.0, 1.0);
            c.p[i] = (old[i].0 + (cand.0 - old[i].0) * room, old[i].1 + (cand.1 - old[i].1) * room);
        }
        if step % 3 == 2 {
            // necks: two parts of the course that have met
            let mut i = lo.saturating_sub(1);
            while i + 8 < c.p.len() && i < hi {
                let reach = (1.4 * w[i.min(w.len() - 1)]).max(0.9 * ds);
                let mut cut = None;
                for j in (i + 6..(i + 130).min(c.p.len())).rev() {
                    if (c.p[i].0 - c.p[j].0).hypot(c.p[i].1 - c.p[j].1) < reach {
                        cut = Some(j);
                        break;
                    }
                }
                if let Some(j) = cut {
                    // the loops cut off lately are still water; the older ones have silted up
                    if j - i >= 8 && step * 10 >= steps * 6 {
                        oxbows.push(Oxbow {
                            x: c.p[i + 2..j - 1].iter().map(|p| (p.0 / CELL_M) as f32).collect(),
                            y: c.p[i + 2..j - 1].iter().map(|p| (p.1 / CELL_M) as f32).collect(),
                            width: (0.8 * river_width(lerp_at(&c.q, c.u[i]))) as f32,
                            level: lerp_at(&c.lev, c.u[i]) as f32,
                        });
                    }
                    c.p.drain(i + 1..j);
                    c.u.drain(i + 1..j);
                    hi = hi.saturating_sub(j - i - 1);
                }
                i += 1;
            }
            if step % 6 == 5 {
                c.resample(ds);
            }
            fresh = true;
        }
    }
    c.smooth(1);
}

/// Put the water on a finished relief. `rng` is the land's stream, after the
/// plan's draws.
pub fn make_water(d: &Design, P: &LandParams, relief: &Relief, rng: &mut Rng) -> Water {
    let g = Grid { w: relief.w, h: relief.h, k: relief.k };
    let n = g.n();
    let (W, H) = (d.W, d.H);
    let N = W * H;
    let dx = g.dx();
    let node_km2 = dx * dx / 1e6;
    let mut z = relief.z.clone();
    let mut rt = Routing::default();

    // lakes scooped, then the sea let in
    let sea0: Vec<bool> = z.iter().map(|&v| v <= 0.0).collect();
    rt.route(g, &z, &sea0, &shore_nodes(g, &sea0));
    carve_lakes(d, g, &mut z, &rt, &sea0, rng);
    rt.route(g, &z, &sea0, &shore_nodes(g, &sea0));
    let lake0: Vec<bool> = (0..n).map(|c| !sea0[c] && rt.fill[c] - z[c] > 0.25).collect();
    let drowned = drown(g, &mut z, &lake0, P);
    let sea = open_sea(g, &z);
    let shore = shore_nodes(g, &sea);
    rt.route(g, &z, &sea, &shore);

    // the big rivers plane their valley floors: wide in soft rock, hardly at all in hard
    let mut floor = vec![false; n];
    {
        let zp = z.clone();
        for &c in &rt.order {
            let c = c as usize;
            let r = rt.rcv[c] as usize;
            let a = rt.area[c] * node_km2;
            if a < 25.0 || rt.fill[c] - zp[c] > 0.25 || (rt.fill[c] - if sea[r] { 0.0 } else { rt.fill[r] }) / dx > 0.006 {
                continue;
            }
            let half = (250.0 * (a / 50.0).powf(0.4) * (relief.khard[c] as f64).sqrt()).min(2500.0);
            let rr = (half / dx).floor() as isize;
            let (ci, cj) = ((c % g.w) as isize, (c / g.w) as isize);
            for dj in -rr..=rr {
                for di in -rr..=rr {
                    let dist = ((di * di + dj * dj) as f64).sqrt() * dx;
                    if dist <= half {
                        let nb = ((cj + dj) * g.w as isize + ci + di) as usize;
                        if !sea[nb] {
                            let level = rt.fill[c] + 0.6 + 0.004 * dist;
                            if z[nb] > level {
                                z[nb] = level;
                            }
                            floor[nb] = true;
                        }
                    }
                }
            }
        }
    }
    rt.route(g, &z, &sea, &shore);

    // lakes: hollows big enough to keep; the small ones silt up
    let mut lake_id = vec![0u16; n];
    let mut lakes: Vec<Lake> = Vec::new();
    let mut outlet_node: Vec<usize> = Vec::new();
    {
        let min_nodes = ((0.75 / node_km2).ceil() as usize).max(3);
        let off = g.offsets();
        let mut seen = vec![false; n];
        let mut stack = Vec::new();
        for start in 0..n {
            if seen[start] || sea[start] || rt.fill[start] - z[start] <= 0.25 {
                continue;
            }
            let mut nodes = vec![start];
            seen[start] = true;
            stack.push(start);
            while let Some(c) = stack.pop() {
                for &o in &off {
                    let nb = (c as isize + o) as usize;
                    if !seen[nb] && !sea[nb] && rt.fill[nb] - z[nb] > 0.25 {
                        seen[nb] = true;
                        nodes.push(nb);
                        stack.push(nb);
                    }
                }
            }
            if nodes.len() < min_nodes || lakes.len() >= 250 {
                for &c in &nodes {
                    z[c] = rt.fill[c];
                }
                continue;
            }
            let id = lakes.len() as u16 + 1;
            let (mut level, mut low, mut deep, mut sx, mut sy) = (f64::INFINITY, nodes[0], 0.0f64, 0.0, 0.0);
            for &c in &nodes {
                lake_id[c] = id;
                if rt.fill[c] < level {
                    level = rt.fill[c];
                    low = c;
                }
                deep = deep.max(rt.fill[c] - z[c]);
                let (x, y) = g.xy(c);
                sx += x;
                sy += y;
            }
            let out = rt.rcv[low] as usize;
            let (ox, oy) = g.xy(out);
            outlet_node.push(out);
            lakes.push(Lake {
                level: level as f32,
                area_km2: (nodes.len() as f64 * node_km2) as f32,
                depth: deep as f32,
                x: (sx / nodes.len() as f64) as f32,
                y: (sy / nodes.len() as f64) as f32,
                outlet: (ox as f32, oy as f32),
                outflow: 0.0,
            });
        }
    }

    // rain, runoff and the flow of every reach
    let mut hc = vec![0.0; N];
    let mut dry = vec![0usize; N];
    for c in 0..n {
        if !sea[c] {
            hc[g.cell(c)] += z[c].max(0.0);
            dry[g.cell(c)] += 1;
        }
    }
    let landc: Vec<bool> = (0..N).map(|c| 2 * dry[c] >= g.k * g.k).collect();
    for c in 0..N {
        hc[c] /= dry[c].max(1) as f64;
    }
    let rain = rainfall(d, P, &hc, &landc);
    let runoff = runoff_months(&rain, &hc);
    let year_mm: Vec<f64> = (0..N).map(|c| (0..12).map(|m| runoff[m * N + c] as f64).sum()).collect();
    let mut flow = vec![0.0; n];
    for &c in &rt.order {
        let c = c as usize;
        flow[c] = year_mm[g.cell(c)] / 1000.0 * dx * dx / YEAR_S;
    }
    for &c in rt.order.iter().rev() {
        let r = rt.rcv[c as usize] as usize;
        if !sea[r] {
            flow[r] += flow[c as usize];
        }
    }
    for (l, &o) in lakes.iter_mut().zip(&outlet_node) {
        l.outflow = if sea[o] { 0.0 } else { flow[o] as f32 };
    }

    // height above the water each node drains to; flood plain and marsh
    let mut hand = vec![0.0f32; n];
    let mut wet = vec![WET_DRY; n];
    {
        let mut level = vec![0.0f64; n];
        let mut big = vec![0.0f64; n];
        for &c in &rt.order {
            let c = c as usize;
            let r = rt.rcv[c] as usize;
            if lake_id[c] > 0 || flow[c] >= 1.5 {
                level[c] = rt.fill[c];
                big[c] = if lake_id[c] > 0 { 99.0 } else { flow[c] };
            } else if sea[r] {
                level[c] = 0.0;
                big[c] = 99.0;
            } else {
                level[c] = level[r];
                big[c] = big[r];
            }
            hand[c] = (z[c] - level[c]).max(0.0) as f32;
            if lake_id[c] > 0 {
                continue;
            }
            let slope = (rt.fill[c] - if sea[r] { 0.0 } else { rt.fill[r] }) / dx;
            if hand[c] <= 0.9 && slope <= 0.002 && (z[c] < 6.0 || big[c] >= 99.0 || (floor[c] && slope < 0.0004)) {
                wet[c] = WET_MARSH;
            } else if floor[c] || (hand[c] <= 3.0 && big[c] < 99.0) {
                wet[c] = WET_PLAIN;
            }
        }
    }

    // the rivers as lines
    let is_river = |c: usize| !sea[c] && lake_id[c] == 0 && flow[c] >= P.stream_flow;
    let mut main_up = vec![u32::MAX; n];
    for &c in &rt.order {
        let c = c as usize;
        let r = rt.rcv[c] as usize;
        if is_river(c) && is_river(r) && (main_up[r] == u32::MAX || flow[c] > flow[main_up[r] as usize]) {
            main_up[r] = c as u32;
        }
    }
    let mut owner = vec![-1i32; n];
    let mut traced: Vec<(Vec<usize>, RiverEnd, usize)> = Vec::new(); // nodes, end, the node it ends on
    for &c in rt.order.iter().rev() {
        let c = c as usize;
        if !is_river(c) || main_up[c] != u32::MAX {
            continue;
        }
        let id = traced.len() as i32;
        let mut nodes = vec![c];
        let mut cur = c;
        owner[c] = id;
        let (end, on) = loop {
            let r = rt.rcv[cur] as usize;
            if sea[r] {
                break (RiverEnd::Sea, r);
            }
            if lake_id[r] > 0 {
                break (RiverEnd::Lake(lake_id[r] as usize - 1), r);
            }
            if main_up[r] as usize != cur {
                break (RiverEnd::River(usize::MAX), r);
            }
            nodes.push(r);
            owner[r] = id;
            cur = r;
        };
        traced.push((nodes, end, on));
    }
    for t in traced.iter_mut() {
        if let RiverEnd::River(_) = t.1 {
            t.1 = RiverEnd::River(owner[t.2] as usize);
        }
    }
    // the biggest first, so that a tributary can be led to where its river now runs
    let mut by_size: Vec<usize> = (0..traced.len()).collect();
    by_size.sort_by(|&a, &b| {
        let (qa, qb) = (flow[*traced[a].0.last().unwrap()], flow[*traced[b].0.last().unwrap()]);
        qb.total_cmp(&qa).then(a.cmp(&b))
    });
    let mut rivers: Vec<River> = vec![River::default(); traced.len()];
    let mut done: Vec<Vec<(f64, f64)>> = vec![Vec::new(); traced.len()];
    let mut oxbows: Vec<Oxbow> = Vec::new();
    let metres = |c: usize| {
        let (x, y) = g.xy(c);
        (x * CELL_M, y * CELL_M)
    };
    for &id in &by_size {
        let (nodes, end, on) = &traced[id];
        let m = nodes.len();
        let mut c = Course { p: Vec::new(), u: Vec::new(), lev: Vec::new(), q: Vec::new(), sl: Vec::new(), sed: Vec::new() };
        for (k, &nd) in nodes.iter().enumerate() {
            c.p.push(metres(nd));
            c.u.push(k as f64);
            c.lev.push(rt.fill[nd]);
            c.q.push(flow[nd]);
            c.sed.push(relief.sed[nd]);
        }
        // the last point: the sea or lake it enters, or the bigger river where that now runs
        let mut last = metres(*on);
        let mut last_lev = match end {
            RiverEnd::Sea => 0.0,
            _ => rt.fill[*on].min(c.lev[m - 1]),
        };
        if let RiverEnd::River(main) = end {
            let line = &done[*main];
            if let Some(best) = line.iter().min_by(|a, b| (a.0 - last.0).hypot(a.1 - last.1).total_cmp(&(b.0 - last.0).hypot(b.1 - last.1))) {
                last = *best;
            }
        }
        last_lev = last_lev.min(c.lev[m - 1]);
        c.p.push(last);
        c.u.push(m as f64);
        c.lev.push(last_lev);
        c.q.push(flow[nodes[m - 1]]);
        c.sed.push(relief.sed[nodes[m - 1]]);
        // the fall of each reach, taken over a few nodes
        for k in 0..=m {
            let (a, b) = (k.saturating_sub(2), (k + 2).min(m));
            let mut len = 0.0;
            for j in a..b {
                len += (c.p[j + 1].0 - c.p[j].0).hypot(c.p[j + 1].1 - c.p[j].1);
            }
            c.sl.push(if len > 0.0 { (c.lev[a] - c.lev[b]) / len } else { 0.0 });
        }
        let mut widths: Vec<f64> = c.q.iter().map(|&q| river_width(q)).collect();
        widths.sort_by(|a, b| a.total_cmp(b));
        let ds = (1.25 * widths[widths.len() / 2]).clamp(40.0, 200.0);
        c.smooth(3);
        c.resample(ds);
        // no bank is quite straight: a wobble of a fraction of the width for the bends to grow from
        for i in 1..c.p.len().saturating_sub(1) {
            let (tx, ty) = (c.p[i + 1].0 - c.p[i - 1].0, c.p[i + 1].1 - c.p[i - 1].1);
            let tl = tx.hypot(ty).max(1e-6);
            let wob = 0.3 * river_width(lerp_at(&c.q, c.u[i])) * (hash01(i as i64, id as i64, d.seeds[1] ^ 0x3EA) - 0.5);
            c.p[i] = (c.p[i].0 - ty / tl * wob, c.p[i].1 + tx / tl * wob);
        }
        meander(&mut c, g, &z, P.meander_steps, ds, &mut oxbows);
        let r = &mut rivers[id];
        r.end = *end;
        for i in 0..c.p.len() {
            let (q, sl, lev) = (lerp_at(&c.q, c.u[i]), lerp_at(&c.sl, c.u[i]), lerp_at(&c.lev, c.u[i]));
            // gravel, a slope a river of this size cannot hold to one channel on (Leopold and Wolman)
            let braided = q >= 1.5 && lerp_at(&c.sed, c.u[i]) > 1.5 && sl > 0.0044 * q.powf(-0.44);
            r.x.push((c.p[i].0 / CELL_M) as f32);
            r.y.push((c.p[i].1 / CELL_M) as f32);
            r.width.push((river_width(q) * if braided { 2.2 } else { 1.0 }) as f32);
            r.level.push(lev as f32);
            r.flow.push(q as f32);
            r.braided.push(braided);
        }
        done[id] = c.p;
    }
    // the ground under a river's new course is cut down to its water
    for r in &rivers {
        for i in 0..r.x.len() {
            let (fi, fj) = ((r.x[i] as f64 * g.k as f64) as usize, (r.y[i] as f64 * g.k as f64) as usize);
            if fi < g.w && fj < g.h {
                let c = fj * g.w + fi;
                if !sea[c] && lake_id[c] == 0 && z[c] > r.level[i] as f64 {
                    z[c] = r.level[i] as f64;
                }
            }
        }
    }

    Water { k: g.k, w: g.w, h: g.h, z, sea, rt, lake_id, lakes, flow, hand, wet, owner, rivers, oxbows, rain, runoff, drowned }
}
