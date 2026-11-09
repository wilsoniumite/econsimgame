//! Rivers: meandering centre lines made from the simulation's river cells, the
//! valleys they cut into the heightfield, the ribbons of water, and the spots
//! where a line crosses them.
//!
//! The simulation only knows which 2 km cells a river runs through and how big
//! it is there. Here the river runs beside the cell centres (where the roads
//! meet and the towns have their market squares) on a smooth curve, and then
//! meanders about that curve as a sine-generated curve, the shape real meanders
//! take: wide loops in flat lowland, a nearly straight course in steep upland.
//! Its meander length grows with its width, and its width grows downstream. The
//! water level falls steadily from source to mouth, and the river cuts its valley
//! down to that level: a flat floodplain about the meander belt in the lowland,
//! a narrow V in the hills, and a drowned funnel, an estuary, at the coast.

use bevy::math::{Vec2, Vec3};

use crate::fields::smoothstep;
use crate::meshkit::{hex, Builder};
use crate::replay::Replay;
use crate::world::{hash2, vnoise};

#[derive(Clone, Debug, Default)]
pub struct River {
    /// centre line, upstream first, in cells
    pub pts: Vec<Vec2>,
    /// half the width of the water at each point
    pub half: Vec<f32>,
    /// water level at each point
    pub level: Vec<f32>,
    /// how far each point lies to the side of the river's smooth course
    pub swing: Vec<f32>,
    /// the river ends in the sea (an estuary), not in another river
    pub mouth: bool,
    /// for a river that reaches the sea, how far along it the coast is
    pub coast: f32,
}

/// Width of the water for the simulation's river size (110 small, 200 great).
fn half_width(strength: f32) -> f32 {
    let s = ((strength - 105.0) / 100.0).clamp(0.0, 1.0);
    0.5 * (0.026 + 0.075 * s.powf(1.15))
}

/// A heightfield on the vertex grid, for sampling while the ground is built.
pub struct HeightGrid<'a> {
    pub hgt: &'a [f32],
    pub gw: usize,
    pub gh: usize,
    pub res: f32,
}

impl HeightGrid<'_> {
    pub fn at(&self, p: Vec2) -> f32 {
        let gx = (p.x * self.res).clamp(0.0, (self.gw - 1) as f32 - 0.001);
        let gy = (p.y * self.res).clamp(0.0, (self.gh - 1) as f32 - 0.001);
        let (i, j) = (gx.floor() as usize, gy.floor() as usize);
        let (tx, ty) = (gx - i as f32, gy - j as f32);
        let k = j * self.gw + i;
        let h = self.hgt;
        (h[k] * (1.0 - tx) + h[k + 1] * tx) * (1.0 - ty) + (h[k + self.gw] * (1.0 - tx) + h[k + self.gw + 1] * tx) * ty
    }
}

fn cell_xy(r: &Replay, c: usize) -> Vec2 {
    Vec2::new((c % r.w) as f32, (c / r.w) as f32)
}

/// Centripetal Catmull-Rom through `pts`, sampled about every `step`.
pub fn spline(pts: &[Vec2], step: f32) -> Vec<Vec2> {
    if pts.len() < 2 {
        return pts.to_vec();
    }
    let n = pts.len();
    let get = |i: isize| -> Vec2 {
        if i < 0 {
            pts[0] * 2.0 - pts[1]
        } else if i as usize >= n {
            pts[n - 1] * 2.0 - pts[n - 2]
        } else {
            pts[i as usize]
        }
    };
    let mut out = vec![pts[0]];
    for s in 0..n - 1 {
        let (p0, p1, p2, p3) = (get(s as isize - 1), get(s as isize), get(s as isize + 1), get(s as isize + 2));
        let knot = |a: Vec2, b: Vec2| a.distance(b).max(1e-4).sqrt();
        let t1 = knot(p0, p1);
        let t2 = t1 + knot(p1, p2);
        let t3 = t2 + knot(p2, p3);
        let m = ((p2 - p1).length() / step).ceil().max(1.0) as usize;
        for k in 1..=m {
            let t = t1 + (t2 - t1) * k as f32 / m as f32;
            let a1 = p0 * ((t1 - t) / t1) + p1 * (t / t1);
            let a2 = p1 * ((t2 - t) / (t2 - t1)) + p2 * ((t - t1) / (t2 - t1));
            let a3 = p2 * ((t3 - t) / (t3 - t2)) + p3 * ((t - t2) / (t3 - t2));
            let b1 = a1 * ((t2 - t) / t2) + a2 * (t / t2);
            let b2 = a2 * ((t3 - t) / (t3 - t1)) + a3 * ((t - t1) / (t3 - t1));
            out.push(b1 * ((t2 - t) / (t2 - t1)) + b2 * ((t - t1) / (t2 - t1)));
        }
    }
    out
}

fn arclen(pts: &[Vec2]) -> Vec<f32> {
    let mut s = vec![0.0];
    for i in 1..pts.len() {
        s.push(s[i - 1] + pts[i].distance(pts[i - 1]));
    }
    s
}

/// Value of a sampled curve at arc length `t` (linear between samples).
fn sample<T: Copy + std::ops::Mul<f32, Output = T> + std::ops::Add<Output = T>>(s: &[f32], v: &[T], t: f32) -> T {
    let i = s.partition_point(|&x| x < t).clamp(1, s.len() - 1);
    let seg = (s[i] - s[i - 1]).max(1e-6);
    let f = ((t - s[i - 1]) / seg).clamp(0.0, 1.0);
    v[i - 1] * (1.0 - f) + v[i] * f
}

fn nearest_on(pts: &[Vec2], p: Vec2) -> (usize, Vec2) {
    let mut best = (f32::MAX, 0, p);
    for i in 0..pts.len().saturating_sub(1) {
        let (a, b) = (pts[i], pts[i + 1]);
        let ab = b - a;
        let t = ((p - a).dot(ab) / ab.length_squared().max(1e-9)).clamp(0.0, 1.0);
        let q = a + ab * t;
        let d = q.distance_squared(p);
        if d < best.0 {
            best = (d, if t > 0.5 { i + 1 } else { i }, q);
        }
    }
    (best.1, best.2)
}

/// Build the rivers over the heightfield `hg` (before any valley is cut).
pub fn build(r: &Replay, cen: &[(f32, f32)], hg: &HeightGrid) -> Vec<River> {
    let n = r.n;
    let land = |c: usize| r.terrain[c] > 0;
    let is_river = |c: usize| land(c) && r.river[c] > 0;
    let down = |c: usize| -> Option<usize> {
        let d = r.down[c];
        if d < 0 || d as usize == c || d as usize >= n { None } else { Some(d as usize) }
    };
    // the main stream into each cell is the biggest river flowing into it
    let mut upm = vec![usize::MAX; n];
    let mut best = vec![0u8; n];
    let mut has_up = vec![false; n];
    for c in 0..n {
        if !is_river(c) {
            continue;
        }
        if let Some(d) = down(c) {
            has_up[d] = true;
            if r.river[c] > best[d] || (r.river[c] == best[d] && c < upm[d]) {
                best[d] = r.river[c];
                upm[d] = c;
            }
        }
    }
    // chains from each source down to the sea or to the river they join
    struct Chain {
        cells: Vec<usize>,
        /// the cell where it joins a bigger river, or the sea cell it runs into
        end: Option<usize>,
        mouth: bool,
    }
    let mut chains: Vec<Chain> = Vec::new();
    for s in 0..n {
        if !is_river(s) || (has_up[s] && upm[s] != usize::MAX && is_river(upm[s])) {
            continue;
        }
        let mut cells = vec![s];
        let mut c = s;
        let (mut end, mut mouth) = (None, false);
        while let Some(d) = down(c) {
            if !land(d) {
                end = Some(d);
                mouth = true;
                break;
            }
            if !is_river(d) || cells.len() > 400 {
                break;
            }
            if upm[d] != c {
                end = Some(d);
                break;
            }
            cells.push(d);
            c = d;
        }
        chains.push(Chain { cells, end, mouth });
    }
    // which chain runs through each cell
    let mut owner = vec![usize::MAX; n];
    for (i, ch) in chains.iter().enumerate() {
        for &c in &ch.cells {
            owner[c] = i;
        }
    }
    // build rivers that others join before the ones that join them
    let mut order: Vec<usize> = Vec::new();
    let mut done = vec![false; chains.len()];
    while order.len() < chains.len() {
        let before = order.len();
        for i in 0..chains.len() {
            if done[i] {
                continue;
            }
            let ready = match (chains[i].end, chains[i].mouth) {
                (Some(e), false) => owner[e] == usize::MAX || done[owner[e]],
                _ => true,
            };
            if ready {
                done[i] = true;
                order.push(i);
            }
        }
        if order.len() == before {
            for (i, d) in done.iter_mut().enumerate() {
                if !*d {
                    *d = true;
                    order.push(i);
                }
            }
        }
    }

    let mut rivers: Vec<River> = vec![River::default(); chains.len()];
    for &ci in &order {
        let ch = &chains[ci];
        // flow direction at each cell, to set the river beside the centre
        let pos = |c: usize| Vec2::new(cen[c].0, cen[c].1);
        let mut ctrl: Vec<Vec2> = Vec::new();
        let mut strength: Vec<f32> = Vec::new();
        let m = ch.cells.len();
        for (k, &c) in ch.cells.iter().enumerate() {
            let prev = if k > 0 { pos(ch.cells[k - 1]) } else { pos(c) };
            let next = if k + 1 < m { pos(ch.cells[k + 1]) } else { ch.end.map(pos).unwrap_or(pos(c)) };
            let dir = (next - prev).normalize_or(Vec2::X);
            let side = Vec2::new(-dir.y, dir.x);
            let xy = cell_xy(r, c);
            // a slowly changing offset, so the river keeps clear of the
            // crossroads and market squares at the cell centres
            let mut off = 0.32 * vnoise(xy.x * 0.37 + 0.5, xy.y * 0.37 + 0.5, 201);
            off += if off >= 0.0 { 0.12 } else { -0.12 };
            ctrl.push(pos(c) + side * off);
            strength.push(r.river[c] as f32);
        }
        let mut join_main: Option<usize> = None;
        if let Some(e) = ch.end {
            if ch.mouth {
                // out into the sea: past the coast to the sea cell's centre
                ctrl.push(pos(e));
                strength.push(*strength.last().unwrap_or(&120.0));
            } else if owner[e] != usize::MAX && !rivers[owner[e]].pts.is_empty() {
                // meet the bigger river where it passes the joining cell
                let main = &rivers[owner[e]];
                let target = pos(e);
                let (_, q) = nearest_on(&main.pts, target);
                ctrl.push(q);
                strength.push(*strength.last().unwrap_or(&120.0));
                join_main = Some(owner[e]);
            } else {
                ctrl.push(pos(e));
                strength.push(*strength.last().unwrap_or(&120.0));
            }
        }
        if ctrl.len() < 2 {
            continue;
        }
        // smooth the corners of the cell path a little, keeping the ends
        for _ in 0..2 {
            let prev = ctrl.clone();
            for k in 1..ctrl.len() - 1 {
                ctrl[k] = prev[k] * 0.5 + (prev[k - 1] + prev[k + 1]) * 0.25;
            }
        }
        // the river's smooth course
        let base = spline(&ctrl, 0.02);
        let bs = arclen(&base);
        let blen = *bs.last().unwrap();
        if blen < 0.05 {
            continue;
        }
        // control index along the course, for the width
        let cs = arclen(&ctrl);
        let cfrac = |t: f32| -> f32 {
            let f = t / blen * cs.last().unwrap();
            let i = cs.partition_point(|&x| x < f).clamp(1, cs.len() - 1);
            let seg = (cs[i] - cs[i - 1]).max(1e-6);
            let w = ((f - cs[i - 1]) / seg).clamp(0.0, 1.0);
            strength[i - 1] * (1.0 - w) + strength[i] * w
        };
        // steepness of the course, smoothed over half a cell
        let bh: Vec<f32> = base.iter().map(|&p| hg.at(p)).collect();
        let slope_at = |t: f32| -> f32 {
            let a = sample(&bs, &bh, (t - 0.4).max(0.0));
            let b = sample(&bs, &bh, (t + 0.4).min(blen));
            ((a - b) / 0.8).max(0.0)
        };
        let normal_at = |t: f32| -> Vec2 {
            let a = sample(&bs, &base, (t - 0.03).max(0.0));
            let b = sample(&bs, &base, (t + 0.03).min(blen));
            let d = (b - a).normalize_or(Vec2::X);
            Vec2::new(-d.y, d.x)
        };
        // meanders: a sine-generated curve about the course
        let ds = 0.008;
        let seed = ci as i32;
        let mut phase = hash2(seed, 1, 211) * std::f32::consts::TAU;
        let (mut sig, mut y) = (0.0f32, 0.0f32);
        let mut pts = Vec::new();
        let mut half = Vec::new();
        let mut swing = Vec::new();
        let mut steps = 0;
        while sig < blen && steps < 200_000 {
            steps += 1;
            let st = cfrac(sig);
            let hw = half_width(st);
            // loops on the scale of the map, like the houses and the hedges
            let lam = (60.0 * hw).clamp(0.75, 2.4) * (1.0 + 0.4 * vnoise(sig * 0.45, seed as f32, 213));
            let steep = smoothstep(0.012, 0.09, slope_at(sig));
            let mut omega = 1.55 * (1.0 - steep) + 0.2 * steep;
            // calm near the ends that join other rivers or start at a source
            omega *= smoothstep(0.0, 0.5, sig);
            if join_main.is_some() {
                omega *= smoothstep(0.0, 0.8, blen - sig);
            }
            // irregular loops: their size wanders, and they lean up or down the
            // valley for a stretch (the skew of a Kinoshita curve)
            omega *= 0.4 + 0.8 * (0.5 + 0.5 * vnoise(sig * 0.7, seed as f32 * 3.1, 215));
            let skew = 0.3 * vnoise(sig * 0.2, seed as f32 * 1.7, 217);
            let phi = omega * phase.sin() + skew * omega * (3.0 * phase).cos() - 1.1 * y / lam;
            sig += ds * phi.cos();
            y += ds * phi.sin();
            phase += std::f32::consts::TAU * ds / lam;
            let taper = if join_main.is_some() { smoothstep(0.0, 0.6, blen - sig) } else { 1.0 };
            let t = sig.min(blen);
            let p = sample(&bs, &base, t) + normal_at(t) * (y * taper);
            if pts.last().map_or(true, |l: &Vec2| l.distance(p) >= 0.012) {
                pts.push(p);
                half.push(hw);
                swing.push(y * taper);
            }
        }
        let end = *base.last().unwrap();
        if pts.last().map_or(true, |l: &Vec2| l.distance(end) > 1e-4) {
            pts.push(end);
            half.push(*half.last().unwrap_or(&0.015));
            swing.push(0.0);
        }
        // water level: the ground under the river, smoothed, never rising downstream
        let raw: Vec<f32> = pts.iter().map(|&p| hg.at(p)).collect();
        let s = arclen(&pts);
        let mut level: Vec<f32> = (0..pts.len())
            .map(|i| {
                let (mut sum, mut wsum) = (0.0, 0.0);
                let mut j = i;
                while j > 0 && s[i] - s[j] < 0.25 {
                    j -= 1;
                }
                while j < pts.len() && s[j] - s[i] < 0.25 {
                    sum += raw[j];
                    wsum += 1.0;
                    j += 1;
                }
                (sum / wsum).min(raw[i])
            })
            .collect();
        for i in 1..level.len() {
            level[i] = level[i].min(level[i - 1] - 0.00005);
        }
        if let Some(mi) = join_main {
            let main = &rivers[mi];
            let (k, _) = nearest_on(&main.pts, *pts.last().unwrap());
            let lj = main.level[k.min(main.level.len() - 1)];
            for l in level.iter_mut() {
                *l = l.max(lj);
            }
            if let Some(l) = level.last_mut() {
                *l = lj;
            }
        }
        let mut coast = f32::MAX;
        if ch.mouth {
            // into the sea: the last stretch before the coast is drowned, and
            // the river opens out into an estuary
            let ic = raw.iter().rposition(|&h| h > 0.0).unwrap_or(0);
            coast = s[ic];
            for i in 0..level.len() {
                let k = smoothstep(coast - 1.1, coast - 0.1, s[i]);
                level[i] = level[i] * (1.0 - k) - 0.03 * k;
                // wider where it is drowned, so the drawn river does not flare on the shore
                half[i] *= 1.0 + 4.0 * smoothstep(coast - 0.5, coast + 0.3, s[i]);
            }
        }
        rivers[ci] = River { pts, half, level, swing, mouth: ch.mouth, coast };
    }
    rivers.into_iter().filter(|rv| rv.pts.len() >= 2).collect()
}

/// Cut the valleys into the heightfield, and say for every vertex how much it
/// lies on a valley floor (1) or not (0).
pub fn carve(rivers: &[River], hgt: &mut [f32], gw: usize, gh: usize, res: f32) -> Vec<f32> {
    let mut floor = vec![0f32; gw * gh];
    let orig = hgt.to_vec();
    for rv in rivers {
        let n = rv.pts.len();
        let s = arclen(&rv.pts);
        // the valley follows the meander belt: its floor is wide where the
        // river swings wide, and widens into a funnel at the mouth
        let belt: Vec<f32> = (0..n)
            .map(|i| {
                let mut m: f32 = 0.0;
                let mut j = i;
                while j > 0 && s[i] - s[j] < 0.5 {
                    j -= 1;
                }
                while j < n && s[j] - s[i] < 0.5 {
                    m = m.max(rv.swing[j].abs());
                    j += 1;
                }
                m
            })
            .collect();
        let mut i = 0;
        while i + 1 < n {
            // coarser steps for the valley, every few points
            let j = (i + 3).min(n - 1);
            let (a, b) = (rv.pts[i], rv.pts[j]);
            let lev = rv.level[i].min(rv.level[j]);
            let hw = rv.half[i].max(rv.half[j]);
            let mut fl = 0.05 + 2.0 * hw + 0.8 * belt[i];
            let mut side = 0.45;
            if rv.mouth {
                fl += 0.6 * smoothstep(rv.coast - 1.4, rv.coast + 0.2, s[i]);
            }
            // in steep country the floor narrows and the sides steepen
            let ground = orig[((a.y * res) as usize).min(gh - 1) * gw + ((a.x * res) as usize).min(gw - 1)];
            let steep = smoothstep(0.35, 0.9, ground - lev + 0.3 * (ground - 0.3).max(0.0));
            fl *= 1.0 - 0.45 * steep;
            side += 1.1 * steep;
            let reach = fl + 0.9;
            let (x0, x1) = (((a.x.min(b.x) - reach) * res).floor().max(0.0) as usize, ((a.x.max(b.x) + reach) * res).ceil().min(gw as f32 - 1.0) as usize);
            let (y0, y1) = (((a.y.min(b.y) - reach) * res).floor().max(0.0) as usize, ((a.y.max(b.y) + reach) * res).ceil().min(gh as f32 - 1.0) as usize);
            let ab = b - a;
            let l2 = ab.length_squared().max(1e-9);
            for gy in y0..=y1 {
                for gx in x0..=x1 {
                    let p = Vec2::new(gx as f32 / res, gy as f32 / res);
                    let t = ((p - a).dot(ab) / l2).clamp(0.0, 1.0);
                    let d = p.distance(a + ab * t);
                    if d > reach {
                        continue;
                    }
                    let lv = rv.level[i] + (rv.level[j] - rv.level[i]) * t;
                    let target = lv + 0.004 + (d - fl).max(0.0) * side + 0.02 * smoothstep(fl * 0.6, fl, d);
                    let k = gy * gw + gx;
                    if target < hgt[k] {
                        hgt[k] = target;
                    }
                    let w = 1.0 - smoothstep(fl * 0.85, fl + 0.1, d);
                    if w > floor[k] && lev > -0.01 {
                        floor[k] = w;
                    }
                }
            }
            i = j;
        }
    }
    floor
}

/// River segments in buckets, for asking how far a spot is from the water and
/// where a line crosses a river.
pub struct WaterIndex {
    cell: f32,
    w: usize,
    h: usize,
    /// per bucket: the segment's ends, its half width, and which river and point it starts at
    buckets: Vec<Vec<(Vec2, Vec2, f32, u32, u32)>>,
}

impl WaterIndex {
    pub fn new(rivers: &[River], map_w: usize, map_h: usize) -> WaterIndex {
        let cell = 0.25;
        let (w, h) = ((map_w as f32 / cell).ceil() as usize + 1, (map_h as f32 / cell).ceil() as usize + 1);
        let mut buckets = vec![Vec::new(); w * h];
        for (ri, rv) in rivers.iter().enumerate() {
            for i in 0..rv.pts.len().saturating_sub(1) {
                if rv.level[i] < -0.005 && rv.level[i + 1] < -0.005 {
                    continue; // out in the sea
                }
                let (a, b) = (rv.pts[i], rv.pts[i + 1]);
                let hw = rv.half[i].max(rv.half[i + 1]);
                let lo = a.min(b) - Vec2::splat(hw);
                let hi = a.max(b) + Vec2::splat(hw);
                let (bx0, by0) = ((lo.x / cell).floor().max(0.0) as usize, (lo.y / cell).floor().max(0.0) as usize);
                let (bx1, by1) = (((hi.x / cell).floor() as usize).min(w - 1), ((hi.y / cell).floor() as usize).min(h - 1));
                for by in by0..=by1 {
                    for bx in bx0..=bx1 {
                        buckets[by * w + bx].push((a, b, hw, ri as u32, i as u32));
                    }
                }
            }
        }
        WaterIndex { cell, w, h, buckets }
    }

    /// Distance from the water's edge (negative in the water), up to about 0.25.
    pub fn dist(&self, p: Vec2) -> f32 {
        let (bx, by) = ((p.x / self.cell).floor() as isize, (p.y / self.cell).floor() as isize);
        let mut best = f32::MAX;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (x, y) = (bx + dx, by + dy);
                if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
                    continue;
                }
                for &(a, b, hw, _, _) in &self.buckets[y as usize * self.w + x as usize] {
                    let ab = b - a;
                    let t = ((p - a).dot(ab) / ab.length_squared().max(1e-9)).clamp(0.0, 1.0);
                    best = best.min(p.distance(a + ab * t) - hw);
                }
            }
        }
        best
    }
}

/// Where the straight line from a to b crosses a river: the point, the river's
/// direction there, its half width and its water level.
pub fn crossings(rivers: &[River], index: &WaterIndex, a: Vec2, b: Vec2) -> Vec<(Vec2, Vec2, f32, f32)> {
    let mut out: Vec<(Vec2, Vec2, f32, f32)> = Vec::new();
    let (lo, hi) = (a.min(b), a.max(b));
    let cell = index.cell;
    let (bx0, by0) = ((lo.x / cell).floor().max(0.0) as usize, (lo.y / cell).floor().max(0.0) as usize);
    let (bx1, by1) = (((hi.x / cell).floor().max(0.0) as usize).min(index.w - 1), ((hi.y / cell).floor().max(0.0) as usize).min(index.h - 1));
    let d = b - a;
    let mut seen: Vec<(u32, u32)> = Vec::new();
    for by in by0..=by1 {
        for bx in bx0..=bx1 {
            for &(p, q, _, ri, i) in &index.buckets[by * index.w + bx] {
                if seen.contains(&(ri, i)) {
                    continue;
                }
                seen.push((ri, i));
                let rv = &rivers[ri as usize];
                let i = i as usize;
                if rv.level[i] < 0.0 {
                    continue;
                }
                let e = q - p;
                let den = d.x * e.y - d.y * e.x;
                if den.abs() < 1e-9 {
                    continue;
                }
                let w = p - a;
                let t = (w.x * e.y - w.y * e.x) / den;
                let u = (w.x * d.y - w.y * d.x) / den;
                if (0.0..=1.0).contains(&t) && (0.0..1.0).contains(&u) {
                    out.push((a + d * t, e.normalize_or(Vec2::X), rv.half[i], rv.level[i] + (rv.level[i + 1] - rv.level[i]) * u));
                }
            }
        }
    }
    out
}

/// The water, and a strip of wet bank on either side, as ribbons lying on the
/// river's own level.
pub fn ribbons(rivers: &[River]) -> Builder {
    let mut b = Builder::default();
    let water = hex(0x5d9fc4);
    let bank = hex(0x7d8a55);
    for rv in rivers {
        // only the part above the sea
        let mut start = None;
        let n = rv.pts.len();
        let mut runs = Vec::new();
        for i in 0..n {
            let wet = rv.level[i] > 0.004;
            match (wet, start) {
                (true, None) => start = Some(i),
                (false, Some(s)) => {
                    runs.push((s, (i + 1).min(n)));
                    start = None;
                }
                _ => {}
            }
        }
        if let Some(s) = start {
            runs.push((s, n));
        }
        for (s, e) in runs {
            if e - s < 2 {
                continue;
            }
            let at = |i: usize, lift: f32| Vec3::new(rv.pts[i].x, rv.level[i].max(0.0) + lift, rv.pts[i].y);
            let banks: Vec<Vec3> = (s..e).map(|i| at(i, 0.006)).collect();
            let wet: Vec<Vec3> = (s..e).map(|i| at(i, 0.009)).collect();
            let wb: Vec<f32> = (s..e).map(|i| 2.0 * rv.half[i] + 0.022).collect();
            let ww: Vec<f32> = (s..e).map(|i| 2.0 * rv.half[i]).collect();
            b.ribbon_var(&banks, &wb, bank);
            b.ribbon_var(&wet, &ww, water);
        }
    }
    b
}
