//! Railways as engineering. The simulation only knows which cells have track,
//! and makes it dearer on slopes and over water. Here the railway is made in
//! three steps, each of which a canal could take in its own way:
//!
//! 1. **A course through the cells** (`course.rs`). The line passes each cell
//!    a little to one side of its middle (where the roads meet and a town has
//!    its market place), running the way the line through that cell ran when
//!    it was first laid; every length that meets there, a branch too, leaves
//!    along that one line, so a junction is a pair of points and not a
//!    crossroads. Between two cells the line bows toward the ground that
//!    suits its level and, where it must climb more than its length allows,
//!    winds to find more length.
//! 2. **A level under rules** (`profile.rs`). Every cell has one level, settled
//!    the year its track is laid and kept ever after, so lines meet at one
//!    level and a branch added later moves nothing. No length climbs more
//!    than the ruling gradient; the line lies on the ground or above it, and
//!    under it only in a tunnel; ground too steep for that is climbed by a
//!    winding length, and at the last by an incline (worked by a stationary
//!    engine at its head before 1850, by a banking engine from a shed at its
//!    foot after).
//! 3. **Structures from the difference between level and ground**: a bank
//!    where the ground falls away, a viaduct of arches where it falls far, a
//!    girder bridge over a river, a tunnel with a portal at each end (roofed
//!    over until the hill is high enough to hide a train) where it rises.
//!
//! The drawn track, the earthworks and the trains all read the one level.
//! What is settled is kept for the run (`Net`), so a year only strings its
//! lines together from lengths already made.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

use bevy::math::{Quat, Vec2, Vec3};

use crate::course::{self, Swing};
use crate::look::Look;
use crate::meshkit::{shade, Builder};
use crate::profile::{self, Rules};
use crate::replay::Replay;
use crate::trains::TrainPath;
use crate::world::{self, Ground};

/// How steeply a line may climb, in height per unit of length.
pub const RULING: f32 = 0.085;
/// The rules the levels are settled by.
pub const RULES: Rules = Rules { ruling: RULING, ease: 0.7, steep: 0.45, wind: 1.6, wind_steps: 4, high: 0.1, tunnel: 0.1 };
/// Height of the rails above their bed.
const LIFT: f32 = 0.016;
/// How far apart the points of a line are.
const STEP: f32 = 0.02;
/// How far to the side of a cell's middle the line passes.
const BESIDE: f32 = 0.2;
/// Half the width of the bed.
const HALF: f32 = 0.026;
/// A bank higher than this is a viaduct.
const VIADUCT: f32 = 0.07;
/// The tallest a viaduct is built before the line dips to keep it lower.
const TALLEST: f32 = 0.22;
/// How deep under the ground a train is hidden.
const HIDDEN: f32 = 0.07;
/// The first year a steep length is worked by a banking engine and not a rope.
const BANKING_FROM: i32 = 1850;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Grade,
    Embankment,
    Viaduct,
    /// over a river
    Bridge,
    Tunnel,
}

/// How a length of line is worked.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Worked {
    Plain,
    /// a steep bank, with a banking engine's shed at its foot
    Bank,
    /// an inclined plane, with a winding engine at its head
    Rope,
}

pub struct Line {
    /// along the line: the map position and the level of the rail bed
    pub pts: Vec<Vec3>,
    /// the ground under each point
    pub ground: Vec<f32>,
    pub kind: Vec<Kind>,
    pub worked: Vec<Worked>,
    /// the cells it runs through, end to end
    pub cells: Vec<usize>,
}

/// The line at a cell: where it passes, the way it runs there, its level,
/// and the ground.
#[derive(Clone, Copy, Debug)]
pub struct Stop {
    pub at: Vec2,
    pub dir: Vec2,
    pub level: f32,
    pub ground: f32,
    /// how many lengths of line meet here
    pub ways: usize,
}

pub struct RailYear {
    pub lines: Vec<Line>,
    /// track being laid: dashed, straight between cell centres
    pub building: Vec<(Vec2, Vec2)>,
    /// the line at each cell that has track
    pub stops: HashMap<usize, Stop>,
    /// the solid works, made once, the first time they are asked for
    works: OnceLock<Builder>,
}

// ------------------------------------------------------------------ what is settled and kept

#[derive(Clone, Copy)]
struct Cell {
    at: Vec2,
    /// the way the line runs here (either way along it)
    dir: Vec2,
    ground: f32,
    level: f32,
    /// the slope of the line here, going the way `dir` points
    slope: f32,
    town: bool,
}

/// A length of line between two neighbouring cells, from `a` to `b`.
struct Length {
    a: usize,
    stage: u8,
    /// the year (from the start) it was laid
    opened: usize,
    pts: Vec<Vec2>,
    level: Vec<f32>,
    ground: Vec<f32>,
    /// over a river
    wet: Vec<bool>,
    /// under the ground
    under: Vec<bool>,
}

/// The railway as far as it has been worked out: each cell's place and level
/// and each length's course, as settled the year it was laid.
#[derive(Default)]
struct Net {
    /// which island and run this is of
    ident: u64,
    /// the track laid in each year worked out so far
    keys: Vec<u64>,
    cells: HashMap<usize, Cell>,
    lengths: HashMap<(usize, usize), Arc<Length>>,
    /// cells left under the ground or high over it because nothing more could give
    given_up: usize,
}

static NET: Mutex<Option<Net>> = Mutex::new(None);

fn v2(p: (f32, f32)) -> Vec2 {
    Vec2::new(p.0, p.1)
}

/// A number for the track laid in year k (not the track being laid).
fn laid_key(r: &Replay, k: usize) -> u64 {
    r.trans[k * r.n..(k + 1) * r.n].iter().fold(1469598103934665603u64, |h, &b| (h ^ (b & 4) as u64).wrapping_mul(1099511628211))
}

fn ident(r: &Replay, g: &Ground) -> u64 {
    let mut h = 1469598103934665603u64;
    let mut eat = |v: u64| h = (h ^ v).wrapping_mul(1099511628211);
    eat(r.w as u64);
    eat(r.h as u64);
    eat(r.y0 as u64);
    for i in (0..g.hgt.len()).step_by((g.hgt.len() / 97).max(1)) {
        eat(g.hgt[i].to_bits() as u64);
    }
    h
}

/// The ground a line would lie on at a point: the highest under the width of its bed.
fn bed_ground(g: &Ground, p: Vec2, side: Vec2) -> f32 {
    let h = |q: Vec2| world::height_at(g, q.x, q.y);
    h(p).max(h(p + side * 0.022)).max(h(p - side * 0.022)).max(0.0)
}

impl Net {
    /// Work out every year up to `k` that has not been yet.
    fn extend(&mut self, r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) {
        let me = ident(r, g);
        if self.ident != me || self.keys.get(k).is_some_and(|&key| key != laid_key(r, k)) {
            *self = Net { ident: me, ..Net::default() };
        }
        for y in self.keys.len()..=k {
            self.add_year(r, g, cen, y);
        }
    }

    fn add_year(&mut self, r: &Replay, g: &Ground, cen: &[(f32, f32)], y: usize) {
        let key = laid_key(r, y);
        let same = self.keys.last() == Some(&key);
        self.keys.push(key);
        if same {
            return;
        }
        let adj = course::graph(r, y, |v| v & 4 != 0);
        let tr = &r.trans[y * r.n..(y + 1) * r.n];
        let fresh: Vec<usize> = (0..r.n).filter(|&c| tr[c] & 4 != 0 && !self.cells.contains_key(&c)).collect();
        let mut new_links: Vec<(usize, usize)> = Vec::new();
        for c in 0..r.n {
            for &m in &adj[c] {
                if c < m && !self.lengths.contains_key(&(c, m)) {
                    new_links.push((c, m));
                }
            }
        }
        if fresh.is_empty() && new_links.is_empty() {
            return;
        }
        let towns: HashSet<usize> = r.header.towns.iter().map(|t| t.node).collect();
        // 1. where the line passes each new cell, and the way it runs there: outward from the
        // lines that are there already, so that each cell can take its neighbour's side
        let fresh_set: HashSet<usize> = fresh.iter().copied().collect();
        let mut placed: HashSet<usize> = HashSet::new();
        let mut queue: VecDeque<usize> = fresh.iter().copied().filter(|&c| adj[c].iter().any(|m| self.cells.contains_key(m))).collect();
        loop {
            while let Some(c) = queue.pop_front() {
                if !placed.insert(c) {
                    continue;
                }
                let cell = self.place(g, cen, &adj, c, towns.contains(&c));
                self.cells.insert(c, cell);
                for &m in &adj[c] {
                    if fresh_set.contains(&m) && !placed.contains(&m) {
                        queue.push_back(m);
                    }
                }
            }
            match fresh.iter().find(|c| !placed.contains(c)) {
                Some(&c) => queue.push_back(c),
                None => break,
            }
        }
        // 2. the levels of the new cells, and how each new length climbs: among the new cells and
        // the settled ones they touch
        let mut region: Vec<usize> = fresh.clone();
        for &c in &fresh {
            region.extend(adj[c].iter().copied());
        }
        for &(a, b) in &new_links {
            region.extend([a, b]);
        }
        region.sort_unstable();
        region.dedup();
        let index: HashMap<usize, usize> = region.iter().enumerate().map(|(i, &c)| (c, i)).collect();
        let nodes: Vec<profile::Node> = region
            .iter()
            .map(|&c| {
                let cell = &self.cells[&c];
                let settled = !fresh_set.contains(&c);
                profile::Node {
                    ground: cell.ground,
                    // a station stands on the ground
                    fixed: if settled { Some(cell.level) } else if cell.town { Some(cell.ground) } else { None },
                    tunnel_ok: !cell.town && adj[c].len() == 2,
                }
            })
            .collect();
        let mut edges: Vec<profile::Edge> = Vec::new();
        let mut edge_cells: Vec<(usize, usize)> = Vec::new();
        for &c in &region {
            for &m in &adj[c] {
                if c < m && index.contains_key(&m) {
                    let old = self.lengths.get(&(c, m));
                    let len = self.cells[&c].at.distance(self.cells[&m].at) * 1.02;
                    edges.push(profile::Edge { a: index[&c], b: index[&m], len, stage: old.map_or(0, |l| l.stage), new: old.is_none() });
                    edge_cells.push((c, m));
                }
            }
        }
        let year = r.y0 + y as i32;
        let longest_tunnel = if year < 1840 { 1 } else if year < 1870 { 2 } else { 3 };
        let settled = profile::settle(&nodes, &mut edges, &RULES, longest_tunnel);
        self.given_up += settled.given_up;
        for (i, &c) in region.iter().enumerate() {
            if fresh_set.contains(&c) {
                self.cells.get_mut(&c).unwrap().level = settled.level[i];
            }
        }
        let stage: HashMap<(usize, usize), u8> = edge_cells.iter().zip(&edges).map(|(&cm, e)| (cm, e.stage)).collect();
        let stage_of = |a: usize, b: usize| -> u8 { stage.get(&(a.min(b), a.max(b))).copied().or(self.lengths.get(&(a.min(b), a.max(b))).map(|l| l.stage)).unwrap_or(0) };
        // 3. the slope of the line at each new cell: level at a station, at the end of a line
        // and where an incline begins, and elsewhere what its two neighbours make it
        let mut slopes: Vec<(usize, f32)> = Vec::new();
        for &c in &fresh {
            let cell = self.cells[&c];
            let mut sides: [Option<(f32, f32, f32)>; 2] = [None, None];
            let mut flat = cell.town;
            for &m in &adj[c] {
                let other = self.cells[&m];
                let v = other.at - cell.at;
                let align = cell.dir.dot(v.normalize_or_zero());
                let st = stage_of(c, m);
                flat |= st >= RULES.incline();
                let side = &mut sides[(align < 0.0) as usize];
                if side.map_or(true, |s| align.abs() > s.0) {
                    *side = Some((align.abs(), other.level - cell.level, v.length() * 1.02 * RULES.wound(st)));
                }
            }
            let lim = RULES.ease * RULES.ruling;
            let slope = match sides {
                [Some(fore), Some(back)] if !flat => ((fore.1 - back.1) / (fore.2 + back.2)).clamp(-lim, lim),
                _ => 0.0,
            };
            slopes.push((c, slope));
        }
        for (c, slope) in slopes {
            self.cells.get_mut(&c).unwrap().slope = slope;
        }
        // 4. the course and the profile of each new length
        let squares: Vec<Vec2> = towns.iter().map(|&c| v2(cen[c])).collect();
        let staged: Vec<(usize, usize, u8)> = new_links.iter().map(|&(a, b)| (a, b, stage_of(a, b))).collect();
        for (a, b, stage) in staged {
            let (ca, cb) = (self.cells[&a], self.cells[&b]);
            let near: Vec<Vec2> = squares.iter().copied().filter(|s| s.distance(ca.at) < 2.0).collect();
            let length = lay(g, a, &ca, &cb, stage, y, &near);
            self.lengths.insert((a, b), Arc::new(length));
        }
    }

    /// Where the line passes a cell and the way it runs there, from the track
    /// about it the year it is laid.
    fn place(&self, g: &Ground, cen: &[(f32, f32)], adj: &[Vec<usize>], c: usize, town: bool) -> Cell {
        let mid = v2(cen[c]);
        let dirs: Vec<Vec2> = adj[c].iter().map(|&m| (v2(cen[m]) - mid).normalize_or(Vec2::X)).collect();
        let dir = match dirs.len() {
            0 => Vec2::X,
            1 => -dirs[0],
            _ => {
                // the straightest way through
                let mut best = (f32::MAX, dirs[0]);
                for i in 0..dirs.len() {
                    for j in i + 1..dirs.len() {
                        let d = dirs[i].dot(dirs[j]);
                        if d < best.0 {
                            best = (d, (dirs[i] - dirs[j]).normalize_or(dirs[i]));
                        }
                    }
                }
                best.1
            }
        };
        let perp = Vec2::new(-dir.y, dir.x);
        let wet = |p: Vec2| world::height_at(g, p.x, p.y) < 0.04 || world::water_dist(g, p.x, p.y) < 0.05;
        // to one side of the middle: the side its neighbours took, on dry ground
        let first = if world::hash2(c as i32, 3, 71) < 0.5 { 1.0 } else { -1.0 };
        let mut best: Option<(f32, Vec2)> = None;
        for (rank, reach) in [BESIDE, BESIDE * 0.5, 0.0].into_iter().enumerate() {
            for side in [first, -first] {
                let off = perp * side * reach;
                let mut score = rank as f32 * 3.0 + if wet(mid + off) { 10.0 } else { 0.0 } + if side == first { 0.0 } else { 0.5 };
                for &m in &adj[c] {
                    if let Some(o) = self.cells.get(&m) {
                        if (o.at - v2(cen[m])).dot(off) < 0.0 {
                            score += 1.0;
                        }
                    }
                }
                if best.map_or(true, |b| score < b.0) {
                    best = Some((score, mid + off));
                }
            }
        }
        let at = best.map_or(mid, |b| b.1);
        Cell { at, dir, ground: bed_ground(g, at, perp).max(0.03), level: 0.0, slope: 0.0, town }
    }
}

/// Lay a length of line from cell `a` to cell `b`: its course, then its level
/// along that course.
fn lay(g: &Ground, a_cell: usize, a: &Cell, b: &Cell, stage: u8, opened: usize, squares: &[Vec2]) -> Length {
    let chord = a.at.distance(b.at).max(1e-3);
    // each end leaves its cell along that cell's line, toward the other
    let sa = if a.dir.dot(b.at - a.at) >= 0.0 { 1.0 } else { -1.0 };
    let sb = if b.dir.dot(a.at - b.at) >= 0.0 { 1.0 } else { -1.0 };
    let (da, db) = (a.dir * sa, -b.dir * sb);
    let incline = stage >= RULES.incline();
    let want = chord * 1.02 * RULES.wound(stage);
    let height = |p: Vec2| world::height_at(g, p.x, p.y).max(0.0);
    // the swing that keeps nearest the ground at the line's level, at the length wanted
    let scale = chord.min(1.2);
    let bows: &[f32] = &[-0.24, -0.12, 0.0, 0.12, 0.24];
    let esses: &[f32] = if stage == 0 || incline { &[-0.1, 0.0, 0.1] } else { &[-0.3, -0.2, -0.1, 0.0, 0.1, 0.2, 0.3] };
    let waves: &[f32] = if stage >= 2 && !incline { &[-0.2, -0.1, 0.0, 0.1, 0.2] } else { &[0.0] };
    let mut best = (f32::MAX, Swing::default());
    for &bow in bows {
        for &ess in esses {
            for &wave in waves {
                let swing = Swing { bow: bow * scale, ess: ess * scale, wave: wave * scale };
                let pts = course::curve(a.at, da, b.at, db, swing, 24);
                let s = course::along(&pts);
                let len = s[24];
                let mut cost = 0.0;
                for (i, p) in pts.iter().enumerate() {
                    let level = a.level + (b.level - a.level) * s[i] / len;
                    let dev = height(*p) - level;
                    // a cut costs more than a fill: the ground cannot be dug
                    cost += if dev > 0.0 { 3.0 * dev.min(0.3) } else { (-dev).min(0.3) };
                    if i % 2 == 0 && world::water_dist(g, p.x, p.y) < 0.02 {
                        cost += 0.06;
                    }
                    if squares.iter().any(|sq| sq.distance(*p) < 0.13) {
                        cost += 0.6;
                    }
                }
                cost = cost / 25.0 + 2.0 * (want - len).max(0.0) / chord + 0.15 * (len - want).max(0.0) / chord + 0.02 * (bow.abs() + ess.abs() + wave.abs());
                if cost < best.0 {
                    best = (cost, swing);
                }
            }
        }
    }
    let rough = course::length(&course::curve(a.at, da, b.at, db, best.1, 24));
    let n = ((rough / STEP).ceil() as usize).max(2);
    let pts = course::curve(a.at, da, b.at, db, best.1, n);
    let s = course::along(&pts);
    let len = s[n].max(1e-3);
    // the smooth curve between the two levels, no steeper than the ruling gradient
    let (h0, h1) = (a.level, b.level);
    let (mut m0, mut m1) = (sa * a.slope * len, -sb * b.slope * len);
    if !incline {
        for _ in 0..4 {
            let steepest = (0..=20).map(|i| profile::hermite_slope(h0, h1, m0, m1, i as f32 / 20.0).abs()).fold(0.0, f32::max) / len;
            if steepest <= RULING {
                break;
            }
            // nearer the straight line from one level to the other
            m0 += ((h1 - h0) - m0) * 0.5;
            m1 += ((h1 - h0) - m1) * 0.5;
        }
    }
    let smooth: Vec<f32> = s.iter().map(|&x| profile::hermite(h0, h1, m0, m1, x / len)).collect();
    let grad = if incline { RULING.max(1.6 * (h1 - h0).abs() / len) } else { RULING };
    // the ground under the bed, and clear of a river where it crosses one
    let side = |i: usize| -> Vec2 {
        let d = (pts[(i + 1).min(n)] - pts[i.saturating_sub(1)]).normalize_or(da);
        Vec2::new(-d.y, d.x)
    };
    let ground: Vec<f32> = pts.iter().map(|p| height(*p)).collect();
    let water: Vec<f32> = pts.iter().map(|p| world::water_dist(g, p.x, p.y)).collect();
    let floor: Vec<f32> = (0..=n).map(|i| bed_ground(g, pts[i], side(i)) + if water[i] < 0.03 { 0.05 } else { 0.0 }).collect();
    // a tunnel where the hill stands high enough over the smooth curve to hide a train, with a
    // short approach at each mouth
    let mut under = vec![false; n + 1];
    let mut i = 0;
    while i <= n {
        if floor[i] - smooth[i] <= HIDDEN {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < n && floor[j + 1] - smooth[j + 1] > HIDDEN {
            j += 1;
        }
        if s[j] - s[i] >= 0.1 || i == 0 || j == n {
            let (mut lo, mut hi) = (i, j);
            while lo > 0 && s[i] - s[lo - 1] <= 0.12 && floor[lo - 1] - smooth[lo - 1] > 0.01 {
                lo -= 1;
            }
            while hi < n && s[hi + 1] - s[j] <= 0.12 && floor[hi + 1] - smooth[hi + 1] > 0.01 {
                hi += 1;
            }
            for u in &mut under[lo..=hi] {
                *u = true;
            }
        }
        i = j + 1;
    }
    // elsewhere the line lies on the ground or over it, and no higher over it than a viaduct is built
    let eased: Vec<f32> = (0..=n).map(|i| {
        let (lo, hi) = (i.saturating_sub(5), (i + 5).min(n));
        floor[lo..=hi].iter().sum::<f32>() / (hi - lo + 1) as f32
    }).collect();
    let mut least: Vec<f32> = (0..=n).map(|i| if under[i] { smooth[i] } else { floor[i].max(smooth[i].min(eased[i] + TALLEST)) }).collect();
    for i in 0..=n {
        // the two ends are where their cells have them, and the line can only climb so fast from there
        let reach = (h0 + grad * s[i]).min(h1 + grad * (len - s[i]));
        if least[i] > reach {
            least[i] = reach;
            // ground that the line cannot get over is roofed
            if floor[i] - reach > 0.006 {
                under[i] = true;
            }
        }
    }
    least[0] = h0;
    least[n] = h1;
    let level = profile::raise(&s, &least, grad);
    let wet = water.iter().map(|&w| w < 0.012).collect();
    Length { a: a_cell, stage, opened, pts, level, ground, wet, under }
}

/// What the line stands on at each point, with short runs merged into their
/// neighbours so that a viaduct is never a stub.
fn classify(pts: &[Vec3], ground: &[f32], wet: &[bool], under: &[bool]) -> Vec<Kind> {
    let n = pts.len();
    let mut kind: Vec<Kind> = (0..n)
        .map(|i| {
            let d = pts[i].y - ground[i];
            if under[i] {
                Kind::Tunnel
            } else if wet[i] {
                Kind::Bridge
            } else if d > VIADUCT {
                Kind::Viaduct
            } else if d > 0.006 {
                Kind::Embankment
            } else {
                Kind::Grade
            }
        })
        .collect();
    let runs = |kind: &[Kind]| -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < n {
            let mut j = i;
            while j + 1 < n && kind[j + 1] == kind[i] {
                j += 1;
            }
            out.push((i, j));
            i = j + 1;
        }
        out
    };
    // a viaduct shorter than this is a bank
    for (i, j) in runs(&kind) {
        if kind[i] == Kind::Viaduct && pts[i].distance(pts[j]) < 0.14 {
            let by_water = (i > 0 && kind[i - 1] == Kind::Bridge) || (j + 1 < n && kind[j + 1] == Kind::Bridge);
            if !by_water {
                kind[i..=j].fill(Kind::Embankment);
            }
        }
    }
    // a viaduct that comes to a river carries on over it
    for (i, j) in runs(&kind) {
        if kind[i] == Kind::Bridge && ((i > 0 && kind[i - 1] == Kind::Viaduct) || (j + 1 < n && kind[j + 1] == Kind::Viaduct)) {
            kind[i..=j].fill(Kind::Viaduct);
        }
    }
    kind
}

fn build_year(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> RailYear {
    let adj = course::graph(r, k, |v| v & 4 != 0);
    let mut guard = NET.lock().unwrap();
    let net = guard.get_or_insert_with(Net::default);
    net.extend(r, g, cen, k);
    let mut lines = Vec::new();
    for cells in course::chains(&adj) {
        if cells.len() < 2 {
            continue;
        }
        let (mut pts, mut ground, mut wet, mut under, mut worked) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for w in cells.windows(2) {
            let Some(len) = net.lengths.get(&(w[0].min(w[1]), w[0].max(w[1]))) else { continue };
            let how = if len.stage < RULES.incline() {
                Worked::Plain
            } else if r.y0 + (len.opened as i32) < BANKING_FROM {
                Worked::Rope
            } else {
                Worked::Bank
            };
            let m = len.pts.len();
            let forward = len.a == w[0];
            for q in 0..m {
                // the end of one length is the start of the next
                if q == 0 && !pts.is_empty() {
                    continue;
                }
                let i = if forward { q } else { m - 1 - q };
                pts.push(Vec3::new(len.pts[i].x, len.level[i], len.pts[i].y));
                ground.push(len.ground[i]);
                wet.push(len.wet[i]);
                under.push(len.under[i]);
                worked.push(how);
            }
        }
        if pts.len() < 2 {
            continue;
        }
        let kind = classify(&pts, &ground, &wet, &under);
        lines.push(Line { pts, ground, kind, worked, cells });
    }
    let stops = (0..r.n)
        .filter(|&c| r.trans[k * r.n + c] & 4 != 0)
        .filter_map(|c| net.cells.get(&c).map(|cell| (c, Stop { at: cell.at, dir: cell.dir, level: cell.level, ground: cell.ground, ways: adj[c].len() })))
        .collect();
    drop(guard);
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
    RailYear { lines, building, stops, works: OnceLock::new() }
}

static CACHE: Mutex<Option<(u64, Arc<RailYear>)>> = Mutex::new(None);

/// A number for the railways of year k that is the same in two years with the
/// same network: the track laid and being laid, cell by cell (so a live
/// game's new year is noticed too).
pub fn key(r: &Replay, k: usize) -> u64 {
    r.trans[k * r.n..(k + 1) * r.n].iter().fold(1469598103934665603u64, |h, &b| (h ^ (b & 12) as u64).wrapping_mul(1099511628211))
}

/// The railways of year k. They are strung together once and kept for as
/// long as the network stays the same, which it does for most years.
pub fn year(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> Arc<RailYear> {
    let key = key(r, k) ^ ident(r, g);
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
/// level everywhere it can be seen; a rope between the rails of an inclined
/// plane; dashed earthworks where track is being laid.
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
            // a step into the tunnel mouth, so that the track does not stop short of it
            let (from, to) = (i.saturating_sub(1), (j + 1).min(n - 1));
            if to > from {
                let bed: Vec<Vec3> = (from..=to).map(|q| line.pts[q] + Vec3::Y * LIFT).collect();
                b.ribbon(&bed, 0.04, ballast);
                let top: Vec<Vec3> = bed.iter().map(|p| *p + Vec3::Y * 0.004).collect();
                b.ribbon(&top, 0.016, iron);
            }
            i = j + 1;
        }
        // the rope of an inclined plane, on its rollers between the rails
        let mut i = 0;
        while i < n {
            let mut j = i;
            while j + 1 < n && line.worked[j + 1] == line.worked[i] {
                j += 1;
            }
            if line.worked[i] == Worked::Rope && j > i {
                let rope: Vec<Vec3> = (i..=j).filter(|&q| line.kind[q] != Kind::Tunnel).map(|q| line.pts[q] + Vec3::Y * (LIFT + 0.0055)).collect();
                b.ribbon(&rope, 0.005, *lk.rail.rope);
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

/// Paths for the trains, along the track at its own level (through the
/// tunnels too, where the hill hides them).
pub fn train_paths(ry: &RailYear) -> Vec<TrainPath> {
    let mut out = Vec::new();
    for line in &ry.lines {
        let pts: Vec<Vec3> = line.pts.iter().map(|p| *p + Vec3::Y * (LIFT + 0.004)).collect();
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

/// The solid works: banks, viaducts and bridges, and the portals of the
/// tunnels. They are kept with the lines, in the look they were first asked
/// for in: the look does not change in a run.
pub fn structures(ry: &RailYear, g: &Ground, lk: &Look) -> Builder {
    ry.works.get_or_init(|| build_structures(ry, g, lk)).clone()
}

/// The runs of points along a line that `pick` holds for.
fn runs_of(n: usize, pick: impl Fn(usize) -> bool) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        if !pick(i) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j + 1 < n && pick(j + 1) {
            j += 1;
        }
        out.push((i, j));
        i = j + 1;
    }
    out
}

fn build_structures(ry: &RailYear, g: &Ground, lk: &Look) -> Builder {
    let mut b = Builder::default();
    let grass = *lk.rail.bank_grass;
    let grass_dark = *lk.rail.bank_grass_shade;
    let stone = *lk.rail.stone;
    let brick = *lk.rail.viaduct_brick;
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    for (li, line) in ry.lines.iter().enumerate() {
        let n = line.pts.len();
        if n < 2 {
            continue;
        }
        let dir_at = |i: usize| -> Vec3 { side_of(line.pts[i.saturating_sub(1)], line.pts[(i + 1).min(n - 1)]) };
        // banks: earth under the bed wherever the ground falls away from it, on either side,
        // sloping down to the ground. Each side of each point: the foot of the slope, and how
        // far the ground there lies under the bed.
        let feet: Vec<[(Vec3, f32); 2]> = (0..n)
            .map(|i| {
                let (p, s) = (line.pts[i], dir_at(i));
                [1.0f32, -1.0].map(|side| {
                    let near = p + s * ((HALF + 0.01) * side);
                    let mut drop = (p.y - height(near.x, near.z)).max(0.0);
                    let mut foot = near;
                    // on a hillside the foot is further down than the edge of the bed is
                    for _ in 0..2 {
                        foot = p + s * ((HALF + 1.5 * drop + 0.01) * side);
                        drop = (p.y - height(foot.x, foot.z)).max(0.0);
                    }
                    (Vec3::new(foot.x, height(foot.x, foot.z) - 0.006, foot.z), drop)
                })
            })
            .collect();
        for i in 0..n - 1 {
            let (k0, k1) = (line.kind[i], line.kind[i + 1]);
            let open = |k: Kind| k == Kind::Grade || k == Kind::Embankment;
            if !(open(k0) || open(k1)) || k0 == Kind::Tunnel || k1 == Kind::Tunnel || k0 == Kind::Bridge || k1 == Kind::Bridge {
                continue;
            }
            let (p0, p1) = (line.pts[i], line.pts[i + 1]);
            let (s0, s1) = (dir_at(i), dir_at(i + 1));
            let mut any = false;
            for (j, side) in [1.0f32, -1.0].into_iter().enumerate() {
                let ((g0, d0), (g1, d1)) = (feet[i][j], feet[i + 1][j]);
                if d0 <= 0.006 && d1 <= 0.006 {
                    continue;
                }
                any = true;
                let t0 = p0 + s0 * (HALF * side);
                let t1 = p1 + s1 * (HALF * side);
                // by the sea the bank is faced with stone
                let sea_wall = g0.y < 0.03;
                let col = if sea_wall { if side > 0.0 { stone } else { shade(stone, 0.85) } } else if (side > 0.0) == ((s0.x - s0.z) > 0.0) { grass } else { grass_dark };
                if side > 0.0 {
                    b.quad(t0, g0, g1, t1, col);
                } else {
                    b.quad(t1, g1, g0, t0, col);
                }
            }
            if any {
                // the top of the bank, under the ballast
                let (a0, a1) = (p0 + s0 * HALF, p1 + s1 * HALF);
                let (c0, c1) = (p0 - s0 * HALF, p1 - s1 * HALF);
                b.quad(a0, a1, c1, c0, *lk.rail.bank_top);
            }
        }
        // viaducts: piers and arches of stone or brick, a parapet along the top, an abutment at each end
        for (i, j) in runs_of(n, |q| line.kind[q] == Kind::Viaduct) {
            viaduct(&mut b, line, i, j, g, if li % 3 == 0 { brick } else { stone });
        }
        // an iron girder bridge over each river, from bank to bank
        for (i, j) in runs_of(n, |q| line.kind[q] == Kind::Bridge) {
            let (a, c) = (line.pts[i.saturating_sub(1)], line.pts[(j + 1).min(n - 1)]);
            girder_bridge(&mut b, lk, a, c, g);
        }
        // tunnels: a portal where the line goes into the hill, and a roof over the line from
        // there until the hill is high enough to hide a train
        for (i, j) in runs_of(n, |q| line.kind[q] == Kind::Tunnel) {
            for (inside, outside) in [(i, i.checked_sub(1)), (j, (j + 1 < n).then_some(j + 1))] {
                let Some(outside) = outside else { continue };
                let (p, q) = (line.pts[outside], line.pts[inside]);
                let out_dir = Vec3::new(p.x - q.x, 0.0, p.z - q.z).normalize_or(Vec3::X);
                portal(&mut b, q, out_dir, stone, *lk.rail.tunnel_mouth);
            }
            let shallow = |q: usize| line.ground[q] - line.pts[q].y < HIDDEN + 0.01;
            for q in i..j {
                if !(shallow(q) || shallow(q + 1)) {
                    continue;
                }
                let (p0, p1) = (line.pts[q], line.pts[q + 1]);
                let (s0, s1) = (dir_at(q), dir_at(q + 1));
                let (hw, wall, top) = (0.034, 0.05, 0.064);
                for side in [1.0f32, -1.0] {
                    let (f0, f1) = (p0 + s0 * (hw * side) - Vec3::Y * 0.02, p1 + s1 * (hw * side) - Vec3::Y * 0.02);
                    let (e0, e1) = (p0 + s0 * (hw * side) + Vec3::Y * wall, p1 + s1 * (hw * side) + Vec3::Y * wall);
                    let (r0, r1) = (p0 + Vec3::Y * top, p1 + Vec3::Y * top);
                    let turf = if (side > 0.0) == ((s0.x - s0.z) > 0.0) { grass } else { grass_dark };
                    if side > 0.0 {
                        b.quad(f0, f1, e1, e0, stone);
                        b.quad(e0, e1, r1, r0, turf);
                    } else {
                        b.quad(f1, f0, e0, e1, shade(stone, 0.85));
                        b.quad(e1, e0, r0, r1, turf);
                    }
                }
            }
        }
    }
    b
}

/// A run of arches carrying the line from point i to point j, with an
/// abutment at each end where it meets the bank or the ground.
fn viaduct(b: &mut Builder, line: &Line, i: usize, j: usize, g: &Ground, col: [f32; 4]) {
    let dark = shade(col, 0.7);
    let span = 0.055;
    // spread the piers evenly along the run
    let mut s = vec![0f32];
    for q in i..j {
        s.push(s[s.len() - 1] + line.pts[q].distance(line.pts[q + 1]));
    }
    let total = *s.last().unwrap();
    if total < 1e-4 {
        return;
    }
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
        if p == 0 || p == count {
            // an abutment: a solid block as wide as the bank's top, into which the bank runs
            b.block(Vec3::new(pt.x, ground, pt.z), 0.03, width + 0.014, h + 0.006, yaw, col, Some(shade(col, 1.06)), 0.0);
            continue;
        }
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
        let (a0, a1, c0, c1) = (a + sa * hw, a - sa * hw, c + sc * hw, c - sc * hw);
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
                    b.quad(u0, l0, l1, u1, c_side);
                } else {
                    b.quad(u1, l1, l0, u0, c_side);
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
    b.block(at(0.0, -0.02, 0.0), 0.016, 0.085, 0.095, yaw, stone, Some(shade(stone, 1.05)), 0.0);
    // the dark mouth of the tunnel on the outer face
    let (w, h) = (0.022, 0.056);
    let face = 0.0081;
    let segs = 6;
    for k in 0..segs {
        let (a0, a1) = (k as f32 / segs as f32 * std::f32::consts::PI, (k + 1) as f32 / segs as f32 * std::f32::consts::PI);
        let pt = |a: f32| at(face, h - w + w * a.sin(), w * a.cos());
        b.tri(at(face, 0.0, 0.0), pt(a1), pt(a0), dark);
    }
    b.quad(at(face, 0.0, w), at(face, 0.0, -w), at(face, h - w, -w), at(face, h - w, w), dark);
    // a coping along the top and wing walls either side
    b.block(at(0.0, 0.075, 0.0), 0.024, 0.095, 0.008, yaw, shade(stone, 0.9), Some(shade(stone, 1.1)), 0.0);
    for side in [-1.0f32, 1.0] {
        b.block(at(0.012, -0.02, 0.05 * side), 0.03, 0.012, 0.062, yaw + 0.5 * side, stone, Some(stone), 0.0);
    }
}

/// An iron girder bridge carrying the line over a river from `a` to `c`, on
/// stone abutments that stand on the banks, with a pier in the water where
/// the river is wide.
fn girder_bridge(b: &mut Builder, lk: &Look, a: Vec3, c: Vec3, g: &Ground) {
    let along = Vec3::new(c.x - a.x, 0.0, c.z - a.z);
    let span = along.length() + 0.012;
    let along = along.normalize_or(Vec3::X);
    let centre = (a + c) * 0.5 + Vec3::Y * 0.006;
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
    // abutments down to the banks, and a pier in the middle of a long span
    let mut stands = vec![-1.0f32, 1.0];
    if span > 0.14 {
        stands.push(0.0);
    }
    for end in stands {
        let e = at(end * half, 0.0, 0.0);
        let ground = world::height_at(g, e.x, e.z) - 0.02;
        let h = (centre.y - 0.008 - ground).max(0.01);
        b.block(Vec3::new(e.x, ground, e.z), if end == 0.0 { 0.014 } else { 0.024 }, 0.062, h, yaw, stone, Some(stone), 0.0);
    }
}

// ------------------------------------------------------------------ the check

/// What the check found in one year's railway.
#[derive(Default, Debug)]
pub struct Checked {
    pub lines: usize,
    pub km: f32,
    /// the largest change of level from one point to the next beyond what the gradient allows
    pub step: (f32, f32, f32),
    /// the largest difference of level between lines where they meet
    pub meet: (f32, f32, f32),
    /// the steepest ordinary track, and the steepest incline or bank
    pub steepest: (f32, f32, f32),
    pub steepest_incline: (f32, f32, f32),
    /// the highest the line stands over the ground
    pub tallest: (f32, f32, f32),
    /// the deepest the line lies under the ground outside a tunnel
    pub buried: (f32, f32, f32),
    /// the furthest a train's path lies from the level of the drawn track
    pub train_off: (f32, f32, f32),
    /// how far over the ground the highest dead end of a line is
    pub end_off: (f32, f32, f32),
    pub viaducts: usize,
    pub viaduct_cells: f32,
    pub tunnels: usize,
    pub tunnel_cells: f32,
    pub longest_tunnel: (f32, f32, f32),
    /// tunnel roofed over because the hill does not hide it
    pub roofed_cells: f32,
    pub inclines: usize,
    pub incline_cells: f32,
}

/// Walk every line of a year and measure how well it holds together.
pub fn check(ry: &RailYear) -> Checked {
    let mut c = Checked { lines: ry.lines.len(), ..Checked::default() };
    let worst = |slot: &mut (f32, f32, f32), v: f32, p: Vec3| {
        if v > slot.0 {
            *slot = (v, p.x, p.z);
        }
    };
    let trains: HashMap<(i32, i32), f32> = train_paths(ry).iter().flat_map(|t| t.pts.iter().map(|p| (((p.x * 1000.0).round() as i32, (p.z * 1000.0).round() as i32), p.y))).collect();
    let mut ends: HashMap<(i32, i32), Vec<(f32, Vec3, f32)>> = HashMap::new();
    for line in &ry.lines {
        let n = line.pts.len();
        for i in 0..n {
            let p = line.pts[i];
            if line.kind[i] != Kind::Tunnel {
                worst(&mut c.tallest, p.y - line.ground[i], p);
                worst(&mut c.buried, line.ground[i] - p.y, p);
            }
            if let Some(&y) = trains.get(&((p.x * 1000.0).round() as i32, (p.z * 1000.0).round() as i32)) {
                worst(&mut c.train_off, (y - (p.y + LIFT + 0.004)).abs(), p);
            }
            if i == 0 {
                continue;
            }
            let q = line.pts[i - 1];
            let ds = Vec2::new(p.x - q.x, p.z - q.z).length().max(1e-6);
            c.km += ds * 2.0;
            let plain = line.worked[i] == Worked::Plain && line.worked[i - 1] == Worked::Plain;
            let grad = (p.y - q.y).abs() / ds;
            if plain {
                worst(&mut c.steepest, grad, p);
                worst(&mut c.step, (p.y - q.y).abs() - RULING * ds, p);
            } else {
                worst(&mut c.steepest_incline, grad, p);
                worst(&mut c.step, (p.y - q.y).abs() - 1.6 * RULES.steep * ds, p);
                c.incline_cells += ds;
            }
            match line.kind[i] {
                Kind::Viaduct => c.viaduct_cells += ds,
                Kind::Tunnel => {
                    c.tunnel_cells += ds;
                    if line.ground[i] - p.y < HIDDEN + 0.01 {
                        c.roofed_cells += ds;
                    }
                }
                _ => {}
            }
        }
        c.viaducts += runs_of(n, |q| line.kind[q] == Kind::Viaduct).len();
        for (i, j) in runs_of(n, |q| line.kind[q] == Kind::Tunnel) {
            c.tunnels += 1;
            let len: f32 = (i..j).map(|q| line.pts[q].distance(line.pts[q + 1])).sum();
            worst(&mut c.longest_tunnel, len, line.pts[i]);
        }
        c.inclines += runs_of(n, |q| line.worked[q] != Worked::Plain).len();
        for i in [0, n - 1] {
            let p = line.pts[i];
            ends.entry(((p.x * 1000.0).round() as i32, (p.z * 1000.0).round() as i32)).or_default().push((p.y, p, line.ground[i]));
        }
    }
    for v in ends.values() {
        let (lo, hi) = v.iter().fold((f32::MAX, f32::MIN), |(lo, hi), e| (lo.min(e.0), hi.max(e.0)));
        worst(&mut c.meet, hi - lo, v[0].1);
        if v.len() == 1 {
            worst(&mut c.end_off, (v[0].0 - v[0].2).abs(), v[0].1);
        }
    }
    c
}

/// With ISOSTATE_RAIL_CHECK set, walk every line of every tenth year (and the
/// last) and say how well the railway holds together (`check`). Then stop.
/// ISOSTATE_RAIL_CHECK=list also names the inclines, the viaducts and the
/// tunnels of the last year, for pointing a camera at them.
pub fn maybe_check(r: &Replay, g: &Ground) {
    let Some(how) = std::env::var_os("ISOSTATE_RAIL_CHECK") else { return };
    let cen = crate::network::centres(r);
    let last = r.filled.clamp(1, r.ny) - 1;
    let mut years: Vec<usize> = (0..=last).filter(|k| (r.y0 + *k as i32) % 10 == 0).collect();
    if years.last() != Some(&last) {
        years.push(last);
    }
    let t0 = std::time::Instant::now();
    for &k in &years {
        let ry = year(r, g, &cen, k);
        if ry.lines.is_empty() {
            continue;
        }
        let c = check(&ry);
        let at = |s: (f32, f32, f32)| format!("{:.3} at {:.1},{:.1}", s.0, s.1, s.2);
        eprintln!(
            "rail check {}: {} lines, {:.0} km; step along a line {}; step where lines meet {}; steepest {}; steepest incline {}; highest over the ground {}; deepest under it outside a tunnel {}; trains off the track's level {}; a dead end off the ground {}; {} viaducts ({:.1} cells), {} tunnels ({:.1} cells, {:.1} of them roofed, the longest {}), {} inclines and banks ({:.1} cells)",
            r.y0 + k as i32, c.lines, c.km, at(c.step), at(c.meet), at(c.steepest), at(c.steepest_incline), at(c.tallest), at(c.buried), at(c.train_off), at(c.end_off), c.viaducts, c.viaduct_cells, c.tunnels, c.tunnel_cells, c.roofed_cells, at(c.longest_tunnel), c.inclines, c.incline_cells
        );
    }
    if let Some(net) = NET.lock().unwrap().as_ref() {
        let wound = net.lengths.values().filter(|l| l.stage > 0 && l.stage < RULES.incline()).count();
        let inclines = net.lengths.values().filter(|l| l.stage >= RULES.incline()).count();
        eprintln!("rail check: {} cells and {} lengths settled in {:.0} ms; {} lengths wound to find length, {} inclines, {} cells where nothing more could give", net.cells.len(), net.lengths.len(), t0.elapsed().as_secs_f64() * 1000.0, wound, inclines, net.given_up);
    }
    if how == "list" {
        let ry = year(r, g, &cen, last);
        for (li, line) in ry.lines.iter().enumerate() {
            let n = line.pts.len();
            for (what, runs) in [
                ("incline", runs_of(n, |q| line.worked[q] != Worked::Plain)),
                ("viaduct", runs_of(n, |q| line.kind[q] == Kind::Viaduct)),
                ("tunnel", runs_of(n, |q| line.kind[q] == Kind::Tunnel)),
                ("bridge", runs_of(n, |q| line.kind[q] == Kind::Bridge)),
            ] {
                for (i, j) in runs {
                    let (a, b) = (line.pts[i], line.pts[j]);
                    let len: f32 = (i..j).map(|q| line.pts[q].distance(line.pts[q + 1])).sum();
                    let over = (i..=j).map(|q| line.pts[q].y - line.ground[q]).fold(f32::MIN, f32::max);
                    eprintln!("line {li}: {what} from {:.2},{:.2} to {:.2},{:.2}, {:.2} long, levels {:.3} to {:.3}, at most {:.3} over the ground", a.x, a.z, b.x, b.z, len, a.y, b.y, over);
                }
            }
        }
    }
    std::process::exit(0);
}
