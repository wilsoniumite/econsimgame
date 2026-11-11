//! Railways as engineering. The simulation only knows which cells have track,
//! and makes it dearer on slopes and over water. Here each line between
//! junctions is a smooth curve through the cell centres, and it is graded: its
//! level follows the ground only loosely and never climbs more steeply than a
//! locomotive could. Where the ground falls away the line runs on an embankment,
//! and on a viaduct of arches where the valley is deep or a river runs below;
//! where the ground rises it runs in a cutting, and through a tunnel, with a
//! portal at each end, where the hill is too high to cut. At the towns and the
//! junctions the line comes back to the ground.
//!
//! Everything for a year is worked out once and kept, since the track, the
//! structures and the trains all ask for it.

use std::sync::{Arc, Mutex, OnceLock};

use bevy::math::{Quat, Vec2, Vec3};

use crate::look::Look;
use crate::meshkit::{shade, Builder};
use crate::replay::Replay;
use crate::rivers;
use crate::trains::TrainPath;
use crate::world::{self, Ground};

/// How steeply a line may climb, in height per unit of length.
const RULING: f32 = 0.085;
/// Height of the rails above their bed.
const LIFT: f32 = 0.016;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Grade,
    Embankment,
    Cutting,
    Viaduct,
    Tunnel,
}

pub struct Line {
    /// along the line: the map position and the level of the rail bed
    pub pts: Vec<Vec3>,
    /// the ground under each point
    pub ground: Vec<f32>,
    pub kind: Vec<Kind>,
}

pub struct RailYear {
    pub lines: Vec<Line>,
    /// track being laid: dashed, straight between cell centres
    pub building: Vec<(Vec2, Vec2)>,
    /// the solid works, made once, the first time they are asked for
    works: OnceLock<Builder>,
}

/// The cells whose transport byte has track, linked to their neighbours (a
/// diagonal only where no pair of straight links joins the two).
fn graph(r: &Replay, k: usize, on: impl Fn(u8) -> bool) -> Vec<Vec<usize>> {
    let (w, h, n) = (r.w, r.h, r.n);
    let tr = &r.trans[k * n..(k + 1) * n];
    let has = |x: isize, y: isize| x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && on(tr[y as usize * w + x as usize]);
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for c in 0..n {
        if !on(tr[c]) {
            continue;
        }
        let (x, y) = ((c % w) as isize, (c / w) as isize);
        for (dx, dy) in [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)] {
            let (nx, ny) = (x + dx, y + dy);
            if !has(nx, ny) {
                continue;
            }
            if dx != 0 && dy != 0 && (has(x + dx, y) || has(x, y + dy)) {
                continue;
            }
            let m = ny as usize * w + nx as usize;
            adj[c].push(m);
            adj[m].push(c);
        }
    }
    adj
}

/// The lines as chains of cells between junctions and ends (and closed loops).
fn chains(adj: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = adj.len();
    let mut used = std::collections::HashSet::new();
    let mut out = Vec::new();
    let walk = |start: usize, first: usize, used: &mut std::collections::HashSet<(usize, usize)>| -> Vec<usize> {
        let mut cells = vec![start];
        let (mut prev, mut cur) = (start, first);
        loop {
            used.insert((prev.min(cur), prev.max(cur)));
            cells.push(cur);
            if adj[cur].len() != 2 {
                break;
            }
            let next = if adj[cur][0] == prev { adj[cur][1] } else { adj[cur][0] };
            if used.contains(&(cur.min(next), cur.max(next))) {
                break;
            }
            prev = cur;
            cur = next;
        }
        cells
    };
    for start in 0..n {
        if adj[start].is_empty() || adj[start].len() == 2 {
            continue;
        }
        for &first in &adj[start] {
            if !used.contains(&(start.min(first), start.max(first))) {
                out.push(walk(start, first, &mut used));
            }
        }
    }
    // closed loops, which have no ends
    for start in 0..n {
        for &first in &adj[start] {
            if !used.contains(&(start.min(first), start.max(first))) {
                out.push(walk(start, first, &mut used));
            }
        }
    }
    out
}

/// Grade a line: its level follows a smoothed ground, never climbs faster than
/// the ruling gradient, keeps clear of the rivers it crosses, and comes back to
/// the ground at both ends and wherever it passes a town.
fn grade(pts: &[Vec2], ground: &[f32], towns: &[Vec2], g: &Ground) -> Vec<f32> {
    let n = pts.len();
    let mut s = vec![0f32; n];
    for i in 1..n {
        s[i] = s[i - 1] + pts[i].distance(pts[i - 1]);
    }
    let total = s[n - 1];
    // smooth the ground over half a cell or so
    let sigma = 0.3;
    let mut lev: Vec<f32> = (0..n)
        .map(|i| {
            let (mut sum, mut wsum) = (0.0f32, 0.0f32);
            let mut j = i;
            while j > 0 && s[i] - s[j] < 2.5 * sigma {
                j -= 1;
            }
            while j < n && s[j] - s[i] < 2.5 * sigma {
                let d = (s[j] - s[i]) / sigma;
                let w = (-0.5 * d * d).exp();
                sum += w * ground[j];
                wsum += w;
                j += 1;
            }
            sum / wsum
        })
        .collect();
    // back to the ground at the ends and at the towns
    for i in 0..n {
        let mut pin = 1.0 - crate::fields::smoothstep(0.0, 0.5, s[i].min(total - s[i]));
        for t in towns {
            let d = t.distance(pts[i]);
            pin = pin.max(1.0 - crate::fields::smoothstep(0.15, 0.55, d));
        }
        lev[i] = lev[i] + (ground[i] - lev[i]) * pin;
    }
    // clear of the rivers
    for i in 0..n {
        if world::water_dist(g, pts[i].x, pts[i].y) < 0.03 {
            lev[i] = lev[i].max(ground[i] + 0.05);
        }
    }
    // the ruling gradient, both ways, a few times over
    for _ in 0..3 {
        for i in 1..n {
            let ds = s[i] - s[i - 1];
            lev[i] = lev[i].clamp(lev[i - 1] - RULING * ds, lev[i - 1] + RULING * ds);
        }
        for i in (0..n - 1).rev() {
            let ds = s[i + 1] - s[i];
            lev[i] = lev[i].clamp(lev[i + 1] - RULING * ds, lev[i + 1] + RULING * ds);
        }
    }
    lev
}

/// What the line stands on at each point, with short runs merged into their
/// neighbours so that a viaduct or a tunnel is never a stub.
fn classify(pts: &[Vec2], lev: &[f32], ground: &[f32]) -> Vec<Kind> {
    let n = pts.len();
    let raw: Vec<Kind> = (0..n)
        .map(|i| {
            let d = lev[i] - ground[i];
            if d > 0.075 {
                Kind::Viaduct
            } else if d > 0.008 {
                Kind::Embankment
            } else if d < -0.11 {
                Kind::Tunnel
            } else if d < -0.008 {
                Kind::Cutting
            } else {
                Kind::Grade
            }
        })
        .collect();
    let mut kind = raw.clone();
    // a tunnel or viaduct shorter than this becomes a cutting or embankment
    let min_run = 0.12;
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && raw[j + 1] == raw[i] {
            j += 1;
        }
        let len: f32 = (i..j).map(|q| pts[q].distance(pts[q + 1])).sum();
        if len < min_run {
            let to = match raw[i] {
                Kind::Tunnel => Some(Kind::Cutting),
                Kind::Viaduct => Some(Kind::Embankment),
                _ => None,
            };
            if let Some(t) = to {
                for q in i..=j {
                    kind[q] = t;
                }
            }
        }
        i = j + 1;
    }
    kind
}

fn build_year(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> RailYear {
    let adj = graph(r, k, |v| v & 4 != 0);
    // every town, not only this year's, so the lines depend on the network alone
    let towns: Vec<Vec2> = r.header.towns.iter().map(|t| {
        let c = cen[t.node.min(r.n - 1)];
        Vec2::new(c.0, c.1)
    }).collect();
    let mut lines = Vec::new();
    for cells in chains(&adj) {
        if cells.len() < 2 {
            continue;
        }
        let ctrl: Vec<Vec2> = cells.iter().map(|&c| Vec2::new(cen[c].0, cen[c].1)).collect();
        let pts = rivers::spline(&ctrl, 0.02);
        let ground: Vec<f32> = pts.iter().map(|p| world::height_at(g, p.x, p.y).max(0.0)).collect();
        let lev = grade(&pts, &ground, &towns, g);
        let kind = classify(&pts, &lev, &ground);
        let pts3 = pts.iter().zip(&lev).map(|(p, &l)| Vec3::new(p.x, l, p.y)).collect();
        lines.push(Line { pts: pts3, ground, kind });
    }
    // track being laid
    let (w, h, n) = (r.w, r.h, r.n);
    let tr = &r.trans[k * n..(k + 1) * n];
    let bit = |x: isize, y: isize, m: u8| x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && tr[y as usize * w + x as usize] & m != 0;
    let mut building = Vec::new();
    for c in 0..n {
        let v = tr[c];
        if v & 12 == 0 {
            continue;
        }
        let (x, y) = ((c % w) as isize, (c / w) as isize);
        for (dx, dy) in [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)] {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || nx >= w as isize || ny >= h as isize {
                continue;
            }
            let m = ny as usize * w + nx as usize;
            let u = tr[m];
            let diag = dx != 0 && dy != 0;
            let skip = diag && (bit(x + dx, y, 12) || bit(x, y + dy, 12));
            let laid = v & 4 != 0 && u & 4 != 0;
            if !laid && ((v & 8 != 0 && u & 12 != 0) || (u & 8 != 0 && v & 12 != 0)) && !skip {
                building.push((Vec2::new(cen[c].0, cen[c].1), Vec2::new(cen[m].0, cen[m].1)));
            }
        }
    }
    RailYear { lines, building, works: OnceLock::new() }
}

static CACHE: Mutex<Option<(u64, Arc<RailYear>)>> = Mutex::new(None);

/// The railways of year k. They are worked out once and kept for as long as
/// the network stays the same, which it does for most years, so stepping
/// through the years does not grade the lines again.
pub fn year(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> Arc<RailYear> {
    // the track laid and being laid, cell by cell (a live game's new year is noticed too)
    let key = r.trans[k * r.n..(k + 1) * r.n].iter().fold(1469598103934665603u64, |h, &b| (h ^ (b & 12) as u64).wrapping_mul(1099511628211));
    let mut c = CACHE.lock().unwrap();
    if let Some((ckey, ry)) = c.as_ref() {
        if *ckey == key {
            return ry.clone();
        }
    }
    let ry = Arc::new(world::timed("year rails", || build_year(r, g, cen, k)));
    *c = Some((key, ry.clone()));
    ry
}

/// The track: a bed of ballast and a pair of dark rails, on the line's own
/// level; nothing inside a tunnel; dashed earthworks where track is being laid.
pub fn track(ry: &RailYear, g: &Ground, lk: &Look) -> Builder {
    let mut b = Builder::default();
    let (ballast, iron, build) = (*lk.rail.ballast, *lk.rail.rails, *lk.rail.earthworks);
    for line in &ry.lines {
        let n = line.pts.len();
        let mut i = 0;
        while i < n {
            if line.kind[i] == Kind::Tunnel {
                i += 1;
                continue;
            }
            let mut j = i;
            while j + 1 < n && line.kind[j + 1] != Kind::Tunnel {
                j += 1;
            }
            if j > i {
                // in a cutting the bed is shown at the ground, between its banks
                let bed: Vec<Vec3> = (i..=j).map(|q| Vec3::new(line.pts[q].x, bed_level(line, q) + LIFT, line.pts[q].z)).collect();
                b.ribbon(&bed, 0.04, ballast);
                let top: Vec<Vec3> = bed.iter().map(|p| *p + Vec3::Y * 0.004).collect();
                b.ribbon(&top, 0.016, iron);
            }
            i = j + 1;
        }
    }
    for &(a, c) in &ry.building {
        let len = a.distance(c);
        let m = ((len / 0.08).ceil() as usize).max(1);
        for q in 0..m {
            if q % 2 == 1 {
                continue;
            }
            let (t0, t1) = (q as f32 / m as f32, (q as f32 + 1.0) / m as f32);
            let (p0, p1) = (a + (c - a) * t0, a + (c - a) * t1);
            let pts = [p0, p1].map(|p| Vec3::new(p.x, world::height_at(g, p.x, p.y).max(0.0) + 0.014, p.y));
            b.ribbon(&pts, 0.03, build);
        }
    }
    b
}

/// The level the track is drawn at: its graded level, except in a cutting,
/// where the ground is not dug away and the track runs on the ground between
/// painted banks.
fn bed_level(line: &Line, i: usize) -> f32 {
    match line.kind[i] {
        Kind::Cutting => line.ground[i].max(line.pts[i].y),
        _ => line.pts[i].y,
    }
}

/// Paths for the trains, along the drawn track (through the tunnels too, where
/// the hill hides them).
pub fn train_paths(ry: &RailYear) -> Vec<TrainPath> {
    let mut out = Vec::new();
    for line in &ry.lines {
        let pts: Vec<Vec3> = (0..line.pts.len()).map(|i| Vec3::new(line.pts[i].x, bed_level(line, i) + LIFT + 0.004, line.pts[i].z)).collect();
        let mut cum = vec![0.0];
        for i in 1..pts.len() {
            cum.push(cum[i - 1] + pts[i].distance(pts[i - 1]));
        }
        let len = *cum.last().unwrap_or(&0.0);
        if len > 2.0 {
            out.push(TrainPath { pts, cum, len });
        }
    }
    out
}

fn side_of(a: Vec3, b: Vec3) -> Vec3 {
    let d = Vec3::new(b.x - a.x, 0.0, b.z - a.z).normalize_or(Vec3::X);
    Vec3::new(-d.z, 0.0, d.x)
}

/// The solid works: embankments, viaducts and bridges, the banks of the
/// cuttings, and the portals of the tunnels. They are kept with the lines, in
/// the look they were first asked for in: the look does not change in a run.
pub fn structures(ry: &RailYear, g: &Ground, lk: &Look) -> Builder {
    ry.works.get_or_init(|| build_structures(ry, g, lk)).clone()
}

fn build_structures(ry: &RailYear, g: &Ground, lk: &Look) -> Builder {
    let mut b = Builder::default();
    let grass = *lk.rail.bank_grass;
    let grass_dark = *lk.rail.bank_grass_shade;
    let stone = *lk.rail.stone;
    let brick = *lk.rail.viaduct_brick;
    for (li, line) in ry.lines.iter().enumerate() {
        let n = line.pts.len();
        if n < 2 {
            continue;
        }
        let dir_at = |i: usize| -> Vec3 {
            let (a, c) = (line.pts[i.saturating_sub(1)], line.pts[(i + 1).min(n - 1)]);
            side_of(a, c)
        };
        // embankments: a bank of earth under the track, sloping down to the ground
        for i in 0..n - 1 {
            let (k0, k1) = (line.kind[i], line.kind[i + 1]);
            let bank = |k: Kind| k == Kind::Embankment || k == Kind::Viaduct;
            if !(bank(k0) && bank(k1)) || (k0 == Kind::Viaduct && k1 == Kind::Viaduct) {
                continue;
            }
            let (p0, p1) = (line.pts[i], line.pts[i + 1]);
            let (s0, s1) = (dir_at(i), dir_at(i + 1));
            let half = 0.026;
            let h0 = (p0.y - line.ground[i]).max(0.0);
            let h1 = (p1.y - line.ground[i + 1]).max(0.0);
            for side in [-1.0f32, 1.0] {
                let t0 = p0 + s0 * (half * side);
                let t1 = p1 + s1 * (half * side);
                let f0 = p0 + s0 * ((half + 1.5 * h0 + 0.01) * side);
                let f1 = p1 + s1 * ((half + 1.5 * h1 + 0.01) * side);
                let g0 = Vec3::new(f0.x, world::height_at(g, f0.x, f0.z) - 0.006, f0.z);
                let g1 = Vec3::new(f1.x, world::height_at(g, f1.x, f1.z) - 0.006, f1.z);
                // by the sea the bank is faced with stone
                let sea_wall = world::height_at(g, f0.x, f0.z) < 0.03;
                let col = if sea_wall { if side > 0.0 { stone } else { shade(stone, 0.85) } } else if (side > 0.0) == ((s0.x - s0.z) > 0.0) { grass } else { grass_dark };
                if side > 0.0 {
                    b.quad(t0, g0, g1, t1, col);
                } else {
                    b.quad(t1, g1, g0, t0, col);
                }
            }
            // the top of the bank, under the ballast
            let (a0, a1) = (p0 + s0 * half, p1 + s1 * half);
            let (c0, c1) = (p0 - s0 * half, p1 - s1 * half);
            b.quad(a0, a1, c1, c0, *lk.rail.bank_top);
        }
        // viaducts: piers and arches of stone or brick, a parapet along the top
        let mut i = 0;
        while i < n {
            if line.kind[i] != Kind::Viaduct {
                i += 1;
                continue;
            }
            let mut j = i;
            while j + 1 < n && line.kind[j + 1] == Kind::Viaduct {
                j += 1;
            }
            viaduct(&mut b, line, i, j, g, if li % 3 == 0 { brick } else { stone });
            i = j + 1;
        }
        // cuttings: the ground is not dug away, so the cutting is shown by its
        // banks, a raised bank of earth along each side of the track whose
        // inner face is lit or in shade by the sun, and turns to rock where the
        // cut is deep. Seen from above at a slant the track lies in a trench.
        for i in 0..n - 1 {
            if line.kind[i] != Kind::Cutting || line.kind[i + 1] != Kind::Cutting {
                continue;
            }
            let (p0, p1) = (line.pts[i], line.pts[i + 1]);
            let (s0, s1) = (dir_at(i), dir_at(i + 1));
            let d0 = (line.ground[i] - p0.y).max(0.0);
            let d1 = (line.ground[i + 1] - p1.y).max(0.0);
            // fade the banks in and out over the ends of the cutting
            let ends = |q: usize| -> f32 {
                let mut m = 1.0f32;
                for (dq, k) in [(1usize, 0.5f32), (2, 0.8)] {
                    if q < dq || q + dq >= n || line.kind[q - dq] != Kind::Cutting || line.kind[q + dq] != Kind::Cutting {
                        m = m.min(k);
                    }
                }
                m
            };
            let (e0, e1) = (ends(i), ends(i + 1));
            let height = |d: f32, e: f32| (0.008 + 0.22 * d).min(0.034) * e;
            let (h0, h1) = (height(d0, e0), height(d1, e1));
            let deep = ((d0 + d1) * 5.0).min(1.0);
            for side in [-1.0f32, 1.0] {
                let at = |p: Vec3, s: Vec3, off: f32, up: f32| {
                    let q = p + s * (off * side);
                    Vec3::new(q.x, world::height_at(g, q.x, q.z).max(0.0) + up, q.z)
                };
                let (inner0, inner1) = (at(p0, s0, 0.024, 0.004), at(p1, s1, 0.024, 0.004));
                let (crest0, crest1) = (at(p0, s0, 0.03 + h0, h0), at(p1, s1, 0.03 + h1, h1));
                let (outer0, outer1) = (at(p0, s0, 0.04 + 2.6 * h0, -0.004), at(p1, s1, 0.04 + 2.6 * h1, -0.004));
                let facing = Vec2::new(-s0.x * side, -s0.z * side);
                let lit = facing.dot(Vec2::new(-0.682, 0.731)) > 0.0;
                let face = crate::meshkit::lerp(
                    if lit { *lk.rail.cutting_grass_lit } else { *lk.rail.cutting_grass_shade },
                    if lit { *lk.rail.cutting_rock_lit } else { *lk.rail.cutting_rock_shade },
                    deep * 0.8,
                );
                let back = *lk.rail.cutting_back;
                if side > 0.0 {
                    b.quad(inner0, crest0, crest1, inner1, face);
                    b.quad(crest0, outer0, outer1, crest1, back);
                } else {
                    b.quad(inner1, crest1, crest0, inner0, face);
                    b.quad(crest1, outer1, outer0, crest0, back);
                }
            }
        }
        // tunnel portals where the line goes into the hill
        for i in 1..n {
            let (a, c) = (line.kind[i - 1], line.kind[i]);
            if (a == Kind::Tunnel) == (c == Kind::Tunnel) {
                continue;
            }
            // the portal stands at the open end, facing out along the line
            let (inside, outside) = if c == Kind::Tunnel { (i, i - 1) } else { (i - 1, i) };
            let p = line.pts[outside];
            let q = line.pts[inside];
            let out_dir = Vec3::new(p.x - q.x, 0.0, p.z - q.z).normalize_or(Vec3::X);
            portal(&mut b, Vec3::new(p.x, bed_level(line, outside), p.z), out_dir, stone, *lk.rail.tunnel_mouth);
        }
    }
    // road bridges are made in scatter.rs; rail bridges over rivers here
    for line in &ry.lines {
        for i in 0..line.pts.len().saturating_sub(1) {
            let (p0, p1) = (line.pts[i], line.pts[i + 1]);
            if world::water_dist(g, p0.x, p0.z) > 0.1 {
                continue;
            }
            for (at, _, half, _) in rivers::crossings(&g.rivers, &g.water, Vec2::new(p0.x, p0.z), Vec2::new(p1.x, p1.z)) {
                if line.kind[i] == Kind::Viaduct {
                    continue; // the viaduct spans it
                }
                let along = Vec3::new(p1.x - p0.x, 0.0, p1.z - p0.z).normalize_or(Vec3::X);
                let deck = p0.y.max(p1.y) + 0.006;
                girder_bridge(&mut b, lk, Vec3::new(at.x, deck, at.y), along, half * 2.0 + 0.05, g);
            }
        }
    }
    b
}

/// A run of arches carrying the line from point i to point j.
fn viaduct(b: &mut Builder, line: &Line, i: usize, j: usize, g: &Ground, col: [f32; 4]) {
    let dark = shade(col, 0.7);
    let span = 0.055;
    // spread the piers evenly along the run
    let mut s = vec![0f32];
    for q in i..j {
        s.push(s[s.len() - 1] + line.pts[q].distance(line.pts[q + 1]));
    }
    let total = *s.last().unwrap();
    let count = ((total / span).round() as usize).max(1);
    let at = |t: f32| -> (Vec3, Vec3) {
        let q = s.partition_point(|&x| x < t).clamp(1, s.len() - 1);
        let f = ((t - s[q - 1]) / (s[q] - s[q - 1]).max(1e-6)).clamp(0.0, 1.0);
        let (a, c) = (line.pts[i + q - 1], line.pts[i + q]);
        (a + (c - a) * f, Vec3::new(c.x - a.x, 0.0, c.z - a.z).normalize_or(Vec3::X))
    };
    let width = 0.05;
    for p in 0..=count {
        let (pt, dir) = at(total * p as f32 / count as f32);
        let ground = world::height_at(g, pt.x, pt.z).min(pt.y) - 0.02;
        let yaw = (-dir.z).atan2(dir.x);
        let h = pt.y - ground;
        // a pier, a little wider at the foot
        b.block(Vec3::new(pt.x, ground, pt.z), 0.014, width + 0.006, h - 0.012, yaw, col, None, 0.0);
        b.block(Vec3::new(pt.x, ground, pt.z), 0.02, width + 0.012, (0.25 * h).min(0.03), yaw, dark, Some(dark), 0.0);
    }
    // the deck, its parapets, and an arch between each pair of piers
    for p in 0..count {
        let (a, da) = at(total * p as f32 / count as f32);
        let (c, dc) = at(total * (p + 1) as f32 / count as f32);
        let (sa, sc) = (Vec3::new(-da.z, 0.0, da.x), Vec3::new(-dc.z, 0.0, dc.x));
        let hw = width * 0.5;
        // deck slab
        let top = Vec3::Y * 0.0;
        let (a0, a1, c0, c1) = (a + sa * hw + top, a - sa * hw + top, c + sc * hw + top, c - sc * hw + top);
        b.quad(a0, c0, c1, a1, col);
        // the face of the arch on each side: the spandrel over a round opening
        let rise = 0.6 * (a.distance(c) * 0.5).min((a.y - world::height_at(g, a.x, a.z)).max(0.01));
        let segs = 6;
        for (side, s_a, s_c) in [(1.0f32, sa, sc), (-1.0, -sa, -sc)] {
            let fa = a + s_a * hw;
            let fc = c + s_c * hw;
            let spring_y = fa.y.min(fc.y) - 0.012 - rise;
            for k in 0..segs {
                let (t0, t1) = (k as f32 / segs as f32, (k + 1) as f32 / segs as f32);
                let arch = |t: f32| -> Vec3 {
                    let x = fa + (fc - fa) * t;
                    let curve = (std::f32::consts::PI * t).sin();
                    Vec3::new(x.x, spring_y + rise * curve, x.z)
                };
                let (u0, u1) = (fa + (fc - fa) * t0, fa + (fc - fa) * t1);
                let (l0, l1) = (arch(t0), arch(t1));
                let c_side = if side > 0.0 { col } else { shade(col, 0.88) };
                if side > 0.0 {
                    b.quad(l0, l1, u1 - Vec3::Y * 0.0, u0, c_side);
                } else {
                    b.quad(l1, l0, u0, u1, c_side);
                }
            }
            // the parapet
            if side > 0.0 {
                b.quad(fa, fc, fc + Vec3::Y * 0.01, fa + Vec3::Y * 0.01, shade(col, 1.06));
            } else {
                b.quad(fc, fa, fa + Vec3::Y * 0.01, fc + Vec3::Y * 0.01, shade(col, 1.06));
            }
        }
        // the soffit, the underside of the arch
        for k in 0..segs {
            let (t0, t1) = (k as f32 / segs as f32, (k + 1) as f32 / segs as f32);
            let point = |t: f32, s: f32| -> Vec3 {
                let fa = a + sa * (hw * s);
                let fc = c + sc * (hw * s);
                let x = fa + (fc - fa) * t;
                let spring_y = (a.y.min(c.y)) - 0.012 - rise;
                Vec3::new(x.x, spring_y + rise * (std::f32::consts::PI * t).sin(), x.z)
            };
            b.quad(point(t0, -1.0), point(t1, -1.0), point(t1, 1.0), point(t0, 1.0), dark);
        }
    }
}

/// A tunnel portal: a stone face across the line with a dark arch, and wing
/// walls stepping back into the hillside.
fn portal(b: &mut Builder, p: Vec3, out: Vec3, stone: [f32; 4], dark: [f32; 4]) {
    let yaw = (-out.z).atan2(out.x);
    let q = Quat::from_rotation_y(yaw);
    let at = |x: f32, y: f32, z: f32| p + q * Vec3::new(x, y, z);
    // the face, a slab standing across the line
    b.block(at(0.0, -0.02, 0.0), 0.016, 0.085, 0.075, yaw, stone, Some(shade(stone, 1.05)), 0.0);
    // the dark mouth of the tunnel on the outer face
    let (w, h) = (0.022, 0.04);
    let face = 0.0081;
    let segs = 6;
    for k in 0..segs {
        let (a0, a1) = (k as f32 / segs as f32 * std::f32::consts::PI, (k + 1) as f32 / segs as f32 * std::f32::consts::PI);
        let pt = |a: f32| at(face, h - w + w * a.sin(), w * a.cos());
        b.tri(at(face, 0.0, 0.0), pt(a1), pt(a0), dark);
    }
    b.quad(at(face, 0.0, w), at(face, 0.0, -w), at(face, h - w, -w), at(face, h - w, w), dark);
    // a coping along the top and wing walls either side
    b.block(at(0.0, 0.055, 0.0), 0.024, 0.095, 0.008, yaw, shade(stone, 0.9), Some(shade(stone, 1.1)), 0.0);
    for side in [-1.0f32, 1.0] {
        b.block(at(0.012, -0.02, 0.05 * side), 0.03, 0.012, 0.05, yaw + 0.5 * side, stone, Some(stone), 0.0);
    }
}

/// An iron girder bridge carrying the line over a river, on stone abutments.
fn girder_bridge(b: &mut Builder, lk: &Look, centre: Vec3, along: Vec3, span: f32, g: &Ground) {
    let yaw = (-along.z).atan2(along.x);
    let q = Quat::from_rotation_y(yaw);
    let at = |x: f32, y: f32, z: f32| centre + q * Vec3::new(x, y, z);
    let iron = *lk.rail.girder;
    let stone = *lk.rail.stone;
    let half = span * 0.5;
    // the deck and two plate girders either side
    b.block(at(0.0, -0.008, 0.0), span, 0.048, 0.008, yaw, iron, Some(shade(iron, 1.2)), 0.0);
    for side in [-1.0f32, 1.0] {
        b.block(at(0.0, -0.008, 0.026 * side), span, 0.005, 0.022, yaw, iron, Some(iron), 0.0);
    }
    // abutments down to the banks
    for end in [-1.0f32, 1.0] {
        let e = at(end * half, 0.0, 0.0);
        let ground = world::height_at(g, e.x, e.z) - 0.02;
        let h = (centre.y - 0.008 - ground).max(0.01);
        b.block(Vec3::new(e.x, ground, e.z), 0.02, 0.06, h, yaw, stone, Some(stone), 0.0);
    }
}
