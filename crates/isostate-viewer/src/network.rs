//! Rivers, roads and railways as ribbons over the terrain. The rivers are
//! made in rivers.rs and the railways in rails.rs; the roads are draped here.
//!
//! A road between two cells is drawn between the two cells' middles
//! (`centres`), straight in plan, or round the meanders its straight line
//! cuts and, on the new land, round a lake or a tideway (`route`); and
//! everything that goes with it is read from here: the ribbon on the ground
//! (`way_line`), the bridges where it crosses a river (`road_bridges`), the
//! causeway where it runs over drowned ground (`causeways`), and the surface
//! a cart rides along it, which is whichever of those is on top (`driven`).

use bevy::math::{Vec2, Vec3};
use isostate_gen::bridge::{RoadBridge, RoadFord};
use isostate_gen::houses::Detail;

use crate::look::Look;
use crate::meshkit::Builder;
use crate::replay::{Crossed, Replay};
use crate::world::{self, Ground};

/// Cell centres with a little jitter, shared by every line on the map: where
/// a cell's roads meet, its town has its market square and its railway
/// passes. On the new land a land cell's is moved onto the nearest dry and
/// level spot of the cell (newland/middles.rs): the water there is where the
/// core put it, and the old island's middle is as often as not in it. On the
/// old island they are where they always were.
pub fn centres(r: &Replay) -> Vec<(f32, f32)> {
    if let Some(land) = &r.land {
        return land.middles(&r.terrain).to_vec();
    }
    (0..r.n)
        .map(|c| {
            let (x, y) = ((c % r.w) as f32, (c / r.w) as f32);
            (
                x + 0.5 + 0.36 * (world::hash2(c as i32, 11, 1) - 0.5),
                y + 0.5 + 0.36 * (world::hash2(c as i32, 13, 2) - 0.5),
            )
        })
        .collect()
}

fn drape(g: &Ground, pts2: &[(f32, f32)], lift: f32) -> Vec<Vec3> {
    // resample so the ribbon follows the ground closely
    let mut out = Vec::new();
    for w in pts2.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let n = ((len / 0.08).ceil() as usize).max(1);
        for i in 0..n {
            let t = i as f32 / n as f32;
            let (x, z) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            out.push(Vec3::new(x, world::height_at(g, x, z).max(0.0) + lift, z));
        }
    }
    if let Some(&(x, z)) = pts2.last() {
        out.push(Vec3::new(x, world::height_at(g, x, z).max(0.0) + lift, z));
    }
    out
}

/// How far above the ground a road's ribbon lies (never below the sea's surface).
pub const ROAD_LIFT: f32 = 0.01;
/// The top of a causeway, over drowned ground.
pub const CAUSEWAY_TOP: f32 = 0.016;

/// How wide a road's ribbon is: a main road (level 3) or a lesser one.
pub fn road_width(lv: u8) -> f32 {
    if lv == 3 { 0.034 } else { 0.022 }
}

/// Whether the picture before the ways' second stage left the road between
/// two cells' middles out: on the new land, where a tenth of a cell of its
/// straight line or more (200 m) lay under standing water (a lake or the
/// tide), looked at every fortieth of a cell. Every road is in the picture
/// now (`route` takes it round the water, or over it on a causeway); this is
/// kept for the roads' check (`ISOSTATE_ROADS`), to count what was left out.
pub fn was_left_out(g: &Ground, a: Vec2, b: Vec2) -> bool {
    if g.laid.is_none() {
        return false;
    }
    let n = ((a.distance(b) / 0.025).ceil() as usize).max(1);
    let under = (0..=n).filter(|&i| {
        let p = a.lerp(b, i as f32 / n as f32);
        world::height_at(g, p.x, p.y) < 0.0
    });
    under.count() >= 4
}

/// The middle line of the ribbon of the road from `a` to `b` (two cells'
/// middles), as it is drawn: straight in plan, and at the ground's height
/// plus `ROAD_LIFT` every 0.08 cells or less, straight from one point to the next.
pub fn road_line(g: &Ground, a: (f32, f32), b: (f32, f32)) -> Vec<Vec3> {
    drape(g, &[a, b], ROAD_LIFT)
}

/// How far from a river's water a road going round a meander keeps, in
/// cells (some 20 m), beyond half its own width.
const BANK_GAP: f32 = 0.01;

/// A road's way between two cells' middles, as it is drawn (`route`).
#[derive(Clone, Debug)]
pub struct Way {
    /// the way in plan, from the one middle to the other
    pub pts: Vec<Vec2>,
    /// where it crosses water: the point on the river's middle line, the
    /// river's direction there, its half width, its water level, and the way
    /// the road runs across it
    pub crossings: Vec<(Vec2, Vec2, f32, f32, Vec2)>,
    /// what it does about standing water (a lake or the tide) on its line
    pub water: OnWater,
}

impl Way {
    /// Whether the way is the straight line between the two middles.
    pub fn straight(&self) -> bool {
        self.pts.len() == 2
    }
}

/// What a road's way does about standing water, a lake or the tide, on the
/// line it would otherwise take (the new land only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OnWater {
    /// there is none on its line
    #[default]
    None,
    /// it goes round on dry ground, along the shore
    Round,
    /// it cannot get round inside its cells without crossing water the run
    /// keeps no crossing for, and is carried over on a causeway: the
    /// simulation carries a road over water no rule of its own has priced
    /// (docs/rules/crossings.md, "Lakes and tideways")
    Causeway,
}

/// The ways found round standing water, kept by the two middles (as bits)
/// and whether the run keeps its crossings (`Ground::detours`).
#[derive(Default)]
#[allow(clippy::type_complexity)]
pub struct Detours(std::sync::Mutex<std::collections::HashMap<(u32, u32, u32, u32, bool), Way>>);

impl Detours {
    /// Forget every way found, so that the next are found afresh (for timing them).
    pub fn forget(&self) {
        if let Ok(mut m) = self.0.lock() {
            m.clear();
        }
    }
}

/// The way of the road from `a` to `b` (two cells' middles). Straight, as
/// every road was drawn before the crossings; and so on a run that keeps no
/// crossings (`round` false: the old island, or the rule off). With `round`,
/// where its straight line cuts the same river an even number of times, it
/// keeps to its bank and goes round the meander, along the water's edge;
/// where it cuts it an odd number of times, it goes round to the middle of
/// those points, crosses there once, straight over, and goes round on the
/// far bank. This is the core's way (`crossings::links_for`, "going round"),
/// which finds the same points: a drawn road crosses a drawn river only where
/// the simulation has a bridge or a ford.
///
/// On the new land the way then keeps off standing water, a lake or the tide
/// (`round_water`): where its line runs over some, it goes round on dry
/// ground inside its two cells (or the cells beside them), along the shore,
/// and where it cannot, it crosses on a causeway. What it finds is kept in
/// `Ground::detours`, since finding it takes a few milliseconds.
pub fn route(g: &Ground, a: Vec2, b: Vec2, round: bool) -> Way {
    let way = route_rivers(g, a, b, round);
    if g.laid.is_none() || !standing_on(g, &way.pts) {
        return way;
    }
    let key = (a.x.to_bits(), a.y.to_bits(), b.x.to_bits(), b.y.to_bits(), round);
    if let Some(kept) = g.detours.0.lock().ok().and_then(|m| m.get(&key).cloned()) {
        return kept;
    }
    let found = round_water(g, a, b, way);
    if let Ok(mut m) = g.detours.0.lock() {
        m.insert(key, found.clone());
    }
    found
}

/// How finely a way is looked along for standing water, in cells (as the
/// picture before looked along a link to leave it out).
const LOOK: f32 = 0.025;

/// Whether any of a line of points stands in a lake or the tide, looked at every `LOOK`.
fn standing_on(g: &Ground, pts: &[Vec2]) -> bool {
    standing_every(g, pts, LOOK)
}

/// Whether any of a line of points stands in a lake or the tide, looked at every `step`.
fn standing_every(g: &Ground, pts: &[Vec2], step: f32) -> bool {
    pts.windows(2).any(|w| {
        let n = ((w[0].distance(w[1]) / step).ceil() as usize).max(1);
        (0..=n).any(|i| {
            let p = w[0].lerp(w[1], i as f32 / n as f32);
            world::height_at(g, p.x, p.y) < 0.0
        })
    })
}

/// How high ground must stand for a way to go over it with no causeway
/// under it (the causeway's own test, `causeway_runs`).
const DRY: f32 = 0.004;
/// The squares of the grid a way round water is looked for on, a cell's side
/// divided (some 60 m).
const GRID: usize = 32;
/// What a step over water costs a way going round, against one over dry
/// ground: it goes round wherever the cells give it ground to, and over the
/// water only where they do not.
const OVER_WATER: f32 = 30.0;
/// What a step in a cell beside the two the road joins costs, against one in
/// them: a third cell is used where the shore asks for it.
const THIRD_CELL: f32 = 1.25;
/// What a step onto a point beside the water (a grid's square from it) or on
/// the beach (below `BEACH`) costs, against one further back: the way keeps
/// a field's breadth back from the shore where it can, and runs along the
/// water's edge where it must.
const SHORE: f32 = 4.0;
/// The top of the beach the ground is painted with, in the viewer's height.
const BEACH: f32 = 0.022;
/// How much dry ground a way drawn straight between two of its points keeps
/// on either hand of its middle line, in cells (40 m: a main road's ribbon
/// is 0.034 wide).
const CLEAR: f32 = 0.02;

/// `route` on the new land, for a way whose line runs over standing water:
/// each stretch of it over the water is replaced by the cheapest way between
/// the dry ground before and after it (`shore_way`), over a grid of the
/// cells the road joins and, for two cells side by side, the cells beside
/// them. The way never crosses a river there that its line did not cross,
/// so a drawn road still crosses a river only where the simulation has a
/// crossing; where nothing dry is left, it goes over the water by the
/// shortest crossing it can find, on a causeway.
fn round_water(g: &Ground, a: Vec2, b: Vec2, way: Way) -> Way {
    // the way, looked along every LOOK: each point, and whether it is one of the way's own
    let mut look: Vec<(Vec2, bool, bool)> = Vec::new();
    for w in way.pts.windows(2) {
        let n = ((w[0].distance(w[1]) / LOOK).ceil() as usize).max(1);
        for i in 0..n {
            let p = w[0].lerp(w[1], i as f32 / n as f32);
            look.push((p, i == 0, world::height_at(g, p.x, p.y) < 0.0));
        }
    }
    let last = *way.pts.last().unwrap();
    look.push((last, true, world::height_at(g, last.x, last.y) < 0.0));
    // the stretches over water, as the points of dry ground either side (two closer than a tenth of a cell are one)
    let mut stretches: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < look.len() {
        if !look[i].2 {
            i += 1;
            continue;
        }
        let mut j = i;
        while j + 1 < look.len() && look[j + 1].2 {
            j += 1;
        }
        let (from, to) = (i.saturating_sub(1), (j + 1).min(look.len() - 1));
        match stretches.last_mut() {
            Some(s) if from <= s.1 + 4 => s.1 = to,
            _ => stretches.push((from, to)),
        }
        i = j + 1;
    }
    let area = Area::of(g, a, b);
    let mut pts = Vec::new();
    let mut wet = false;
    let mut next = 0;
    for (from, to) in stretches {
        pts.extend(look[next..from].iter().filter(|l| l.1).map(|l| l.0));
        let (p, q) = (look[from].0, look[to].0);
        pts.push(p);
        match area.shore_way(g, p, q) {
            Some((inner, over)) => {
                pts.extend(inner);
                wet |= over;
            }
            None => wet = true,
        }
        pts.push(q);
        next = to + 1;
    }
    pts.extend(look[next..].iter().filter(|l| l.1).map(|l| l.0));
    pts.dedup_by(|x, y| x.distance(*y) < 1e-6);
    // where the way turns back on itself (where it leaves its line for the grid's nearest
    // point), it goes straight on, so long as that keeps out of standing water and crosses no river
    let mut i = 1;
    while i + 1 < pts.len() {
        let (u, v, w) = (pts[i - 1], pts[i], pts[i + 1]);
        let back = (v - u).normalize_or(Vec2::X).dot((w - v).normalize_or(Vec2::X)) < 0.0;
        if back && !standing_every(g, &[u, w], 0.01) && crate::rivers::crossing_points(&g.rivers, &g.water, u, w).is_empty() {
            pts.remove(i);
            i = i.saturating_sub(1).max(1);
        } else {
            i += 1;
        }
    }
    // (a point found again after going round water is still the way's end)
    if pts.len() < 2 {
        pts = vec![a, b];
    }
    // it crosses water on a causeway if any of it still stands in water, looked at as closely as the check looks
    let wet = wet && standing_every(g, &pts, 0.01);
    Way { pts, crossings: way.crossings, water: if wet { OnWater::Causeway } else { OnWater::Round } }
}

/// The grid a way round water is looked for on: the cells the road joins and,
/// for two cells side by side, the cells beside them on either hand.
struct Area {
    /// the corner of the grid, and its size in points
    x0: f32,
    y0: f32,
    nx: usize,
    ny: usize,
    /// the two cells the road joins (and, on a diagonal, the two its line passes between), as the box of them
    own: (usize, usize, usize, usize),
}

impl Area {
    fn of(g: &Ground, a: Vec2, b: Vec2) -> Area {
        let (ax, ay, bx, by) = (a.x.floor() as isize, a.y.floor() as isize, b.x.floor() as isize, b.y.floor() as isize);
        let (mut x0, mut x1, mut y0, mut y1) = (ax.min(bx), ax.max(bx), ay.min(by), ay.max(by));
        let own = (x0.max(0) as usize, x1.max(0) as usize, y0.max(0) as usize, y1.max(0) as usize);
        if ay == by {
            (y0, y1) = (y0 - 1, y1 + 1);
        }
        if ax == bx {
            (x0, x1) = (x0 - 1, x1 + 1);
        }
        let (x0, x1) = (x0.max(0), x1.min(g.w as isize - 1));
        let (y0, y1) = (y0.max(0), y1.min(g.h as isize - 1));
        Area { x0: x0 as f32, y0: y0 as f32, nx: (x1 - x0 + 1) as usize * GRID + 1, ny: (y1 - y0 + 1) as usize * GRID + 1, own }
    }

    fn at(&self, i: usize) -> Vec2 {
        Vec2::new(self.x0 + (i % self.nx) as f32 / GRID as f32, self.y0 + (i / self.nx) as f32 / GRID as f32)
    }

    fn nearest(&self, p: Vec2) -> usize {
        let i = (((p.x - self.x0) * GRID as f32).round().max(0.0) as usize).min(self.nx - 1);
        let j = (((p.y - self.y0) * GRID as f32).round().max(0.0) as usize).min(self.ny - 1);
        j * self.nx + i
    }

    /// The way from `p` to `q`, two points of dry ground either side of
    /// water: the points between, and whether it still goes over water. The
    /// cheapest way over the grid's points, a step costing its length, times
    /// `OVER_WATER` where it ends in water and `THIRD_CELL` outside the two
    /// cells, and never across a river's line; then drawn straight from
    /// point to point wherever the straight line keeps as dry, so that it
    /// follows the shore and no grid. None if no way is found at all.
    fn shore_way(&self, g: &Ground, p: Vec2, q: Vec2) -> Option<(Vec<Vec2>, bool)> {
        use std::cmp::Reverse;
        let n = self.nx * self.ny;
        let step = 1.0 / GRID as f32;
        let hi = Vec2::new(self.x0 + (self.nx - 1) as f32 * step, self.y0 + (self.ny - 1) as f32 * step);
        let rivers = crate::rivers::lengths_in(&g.rivers, &g.water, Vec2::new(self.x0, self.y0), hi);
        let heights: Vec<f32> = (0..n).map(|i| {
            let s = self.at(i);
            world::height_at(g, s.x, s.y)
        }).collect();
        let wet: Vec<bool> = heights.iter().map(|&h| h < DRY).collect();
        // the dry points on the beach or beside the water
        let shore: Vec<bool> = (0..n).map(|i| {
            let (x, y) = ((i % self.nx) as isize, (i / self.nx) as isize);
            !wet[i] && (heights[i] < BEACH || (-1..=1isize).any(|dy| (-1..=1isize).any(|dx| {
                let (u, v) = (x + dx, y + dy);
                u >= 0 && v >= 0 && (u as usize) < self.nx && (v as usize) < self.ny && wet[v as usize * self.nx + u as usize]
            })))
        }).collect();
        // the points near a river's line, whose steps are looked at for crossing it
        let mut near = vec![false; n];
        for &(r0, r1, half) in &rivers {
            let reach = half + 1.5 * step;
            let (lo, top) = (r0.min(r1) - Vec2::splat(reach), r0.max(r1) + Vec2::splat(reach));
            let (i0, j0) = (((lo.x - self.x0) * GRID as f32).floor().max(0.0) as usize, ((lo.y - self.y0) * GRID as f32).floor().max(0.0) as usize);
            let (i1, j1) = ((((top.x - self.x0) * GRID as f32).ceil().max(0.0) as usize).min(self.nx - 1), (((top.y - self.y0) * GRID as f32).ceil().max(0.0) as usize).min(self.ny - 1));
            for j in j0..=j1 {
                for i in i0..=i1 {
                    near[j * self.nx + i] = true;
                }
            }
        }
        let crosses_river = |u: Vec2, v: Vec2| rivers.iter().any(|&(r0, r1, _)| meets(u, v, r0, r1));
        let third = |s: Vec2| {
            let (cx, cy) = (s.x.floor().max(0.0) as usize, s.y.floor().max(0.0) as usize);
            !(self.own.0..=self.own.1).contains(&cx) || !(self.own.2..=self.own.3).contains(&cy)
        };
        let (start, goal) = (self.nearest(p), self.nearest(q));
        let goal_at = self.at(goal);
        let mut cost = vec![f32::MAX; n];
        let mut came = vec![usize::MAX; n];
        let mut heap = std::collections::BinaryHeap::new();
        cost[start] = 0.0;
        heap.push((Reverse(self.at(start).distance(goal_at).to_bits()), start));
        while let Some((_, u)) = heap.pop() {
            if u == goal {
                break;
            }
            let (ui, uj) = ((u % self.nx) as isize, (u / self.nx) as isize);
            let at_u = self.at(u);
            for (di, dj) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let (vi, vj) = (ui + di, uj + dj);
                if vi < 0 || vj < 0 || vi as usize >= self.nx || vj as usize >= self.ny {
                    continue;
                }
                let v = vj as usize * self.nx + vi as usize;
                let at_v = self.at(v);
                if (near[u] || near[v]) && crosses_river(at_u, at_v) {
                    continue;
                }
                let len = if di != 0 && dj != 0 { std::f32::consts::SQRT_2 * step } else { step };
                let c = cost[u] + len * if wet[v] { OVER_WATER } else if shore[v] { SHORE } else { 1.0 } * if third(at_v) { THIRD_CELL } else { 1.0 };
                if c < cost[v] {
                    cost[v] = c;
                    came[v] = u;
                    // (costs are never below nought, so their bits order as they do)
                    heap.push((Reverse((c + at_v.distance(goal_at)).to_bits()), v));
                }
            }
        }
        if cost[goal] == f32::MAX {
            return None;
        }
        let mut path = vec![goal];
        while *path.last().unwrap() != start {
            path.push(came[*path.last().unwrap()]);
        }
        path.reverse();
        // the points, from p to q, and whether each is in water
        let mut way: Vec<(Vec2, bool)> = vec![(p, false)];
        way.extend(path.iter().map(|&i| (self.at(i), wet[i])));
        way.push((q, false));
        // straight wherever it keeps dry, with `CLEAR` of dry ground either hand, and off the
        // beach unless it is on the beach at either end already, and crosses no river; over water
        // straight from shore to shore
        let height = |t: Vec2| world::height_at(g, t.x, t.y);
        let dry_line = |u: Vec2, v: Vec2| {
            let k = ((u.distance(v) / 0.01).ceil() as usize).max(1);
            let side = (v - u).normalize_or(Vec2::X).perp() * CLEAR;
            let least = BEACH.min(height(u)).min(height(v));
            (0..=k).all(|i| {
                let s = u.lerp(v, i as f32 / k as f32);
                height(s) >= least && height(s + side) >= DRY && height(s - side) >= DRY
            }) && !crosses_river(u, v)
        };
        let mut out: Vec<Vec2> = Vec::new();
        let mut over = false;
        let mut i = 0;
        while i + 1 < way.len() {
            if way[i + 1].1 {
                // into the water: on to the first dry point after it
                let mut j = i + 1;
                while j + 1 < way.len() && way[j].1 {
                    j += 1;
                }
                over = true;
                if crosses_river(way[i].0, way[j].0) {
                    out.extend(way[i + 1..=j].iter().map(|w| w.0));
                } else {
                    out.push(way[j].0);
                }
                i = j;
                continue;
            }
            let mut j = i + 1;
            while j + 1 < way.len() && !way[j + 1].1 && dry_line(way[i].0, way[j + 1].0) {
                j += 1;
            }
            out.push(way[j].0);
            i = j;
        }
        // (the last is q, which the caller puts in)
        out.pop();
        Some((out, over))
    }
}

/// Whether the lengths from `a` to `b` and from `p` to `q` meet.
fn meets(a: Vec2, b: Vec2, p: Vec2, q: Vec2) -> bool {
    let (d, e) = (b - a, q - p);
    let den = d.perp_dot(e);
    if den.abs() < 1e-12 {
        return false;
    }
    let w = p - a;
    let (t, u) = (w.perp_dot(e) / den, w.perp_dot(d) / den);
    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
}

/// `route` before the water: round the meanders, as the crossings' stage drew it.
fn route_rivers(g: &Ground, a: Vec2, b: Vec2, round: bool) -> Way {
    use crate::rivers::River;
    let straight = || {
        let along = (b - a).normalize_or(Vec2::X);
        let crossings = crate::rivers::crossings(&g.rivers, &g.water, a, b).into_iter().map(|(at, river, half, level)| (at, river, half, level, along)).collect();
        Way { pts: vec![a, b], crossings, water: OnWater::None }
    };
    if !round {
        return straight();
    }
    let found = crate::rivers::crossing_points(&g.rivers, &g.water, a, b);
    // by river, in order along the road
    let mut groups: Vec<(u32, Vec<(f32, u32, f32)>)> = Vec::new();
    for &(t, ri, i, u) in &found {
        match groups.iter_mut().find(|gr| gr.0 == ri) {
            Some(gr) => gr.1.push((t, i, u)),
            None => groups.push((ri, vec![(t, i, u)])),
        }
    }
    if groups.iter().all(|gr| gr.1.len() < 2) {
        return straight();
    }
    // a river's points along the road must not run among another's (then it is drawn straight)
    for (x, gx) in groups.iter().enumerate() {
        for gy in &groups[x + 1..] {
            let (x0, x1, y0, y1) = (gx.1[0].0, gx.1[gx.1.len() - 1].0, gy.1[0].0, gy.1[gy.1.len() - 1].0);
            if x0 <= y1 && y0 <= x1 {
                return straight();
            }
        }
    }
    let at = |rv: &River, s: f32| {
        let i = (s.floor() as usize).min(rv.pts.len() - 2);
        let u = s - i as f32;
        (rv.pts[i].lerp(rv.pts[i + 1], u), (rv.pts[i + 1] - rv.pts[i]).normalize_or(Vec2::X), rv.half[i] + (rv.half[i + 1] - rv.half[i]) * u, rv.level[i] + (rv.level[i + 1] - rv.level[i]) * u)
    };
    // the water's edge on one side (`side` 1 is the left of the way the river runs, -1 the right), and a little back
    let edge = |rv: &River, s: f32, side: f32| {
        let (p, dir, half, _) = at(rv, s);
        p + dir.perp() * side * (half + BANK_GAP)
    };
    // along the edge from s0 to s1, past every point of the river between
    let bank = |rv: &River, s0: f32, s1: f32, side: f32, out: &mut Vec<Vec2>| {
        out.push(edge(rv, s0, side));
        let (lo, hi) = (s0.min(s1), s0.max(s1));
        let mut inner: Vec<usize> = ((lo.floor() as usize + 1)..=(hi.ceil() as usize).min(rv.pts.len() - 1)).filter(|&i| (i as f32) > lo && (i as f32) < hi).collect();
        if s1 < s0 {
            inner.reverse();
        }
        for i in inner {
            let dir = (rv.pts[(i + 1).min(rv.pts.len() - 1)] - rv.pts[i.saturating_sub(1)]).normalize_or(Vec2::X);
            out.push(rv.pts[i] + dir.perp() * side * (rv.half[i] + BANK_GAP));
        }
        out.push(edge(rv, s1, side));
    };
    groups.sort_by(|x, y| x.1[0].0.total_cmp(&y.1[0].0));
    let mut pts = vec![a];
    let mut crossings = Vec::new();
    let line_along = (b - a).normalize_or(Vec2::X);
    for (ri, list) in &groups {
        let rv = &g.rivers[*ri as usize];
        let s: Vec<f32> = list.iter().map(|&(_, i, u)| i as f32 + u).collect();
        if list.len() == 1 {
            let (p, dir, half, level) = at(rv, s[0]);
            crossings.push((p, dir, half, level, line_along));
            continue;
        }
        // the bank the road comes from
        let (p, dir, _, _) = at(rv, s[0]);
        let from = *pts.last().unwrap();
        let near = if dir.perp_dot(from - p) >= 0.0 { 1.0 } else { -1.0 };
        let k = list.len();
        if k % 2 == 0 {
            bank(rv, s[0], s[k - 1], near, &mut pts);
        } else {
            let m = k / 2;
            bank(rv, s[0], s[m], near, &mut pts);
            let (p, dir, half, level) = at(rv, s[m]);
            let (here, there) = (edge(rv, s[m], near), edge(rv, s[m], -near));
            crossings.push((p, dir, half, level, (there - here).normalize_or(line_along)));
            bank(rv, s[m], s[k - 1], -near, &mut pts);
        }
    }
    pts.push(b);
    Way { pts, crossings, water: OnWater::None }
}

/// How far a road's curve may leave its straight line where it crosses a
/// river to cross it square: at most this angle from the line (60 degrees),
/// so that a river running nearly along the road does not twist it into an S.
const SQUARE_MOST: f32 = 0.5;
/// How far past a bridge's ramp (`isostate_gen::bridge::APPROACH`, 0.02)
/// a road's curve runs straight on, so that a cart is square on the bridge
/// before it climbs.
const APPROACH_PAST: f32 = 0.03;

/// The way of the road from `a` to `b` (two cells' middles) as a smooth
/// curve (`road_course`): it leaves `a` along `da` and arrives at `b` along
/// `db` (both of length one), on the railways' curve (`course::Ends`).
/// Where the straight line crosses a river, the curve passes through the
/// same point, square to the water (or as near square as `SQUARE_MOST`
/// lets it), and it must cross every river exactly as often as the way
/// `route` draws does: a drawn road crosses a drawn river only where the
/// simulation has a bridge or a ford, and no more often. Where it would
/// not, or where `route` takes the road round a meander along the bank, the
/// way is `route`'s. On the new land it then keeps off standing water as
/// `route` does (`round_water`).
///
/// `seek` is asked for the way of each stretch between two of those points
/// (from a point and its direction to the next): its points after the first,
/// or none for the curve. A road on the new land seeks its way over the
/// ground there (`road_ground`); where the way it finds crosses a river more
/// often than the straight line, or stands in water, the curve is drawn
/// instead, as it is with no seeking at all.
pub fn bent_route(g: &Ground, a: Vec2, b: Vec2, da: Vec2, db: Vec2, round: bool, seek: &mut dyn FnMut(Vec2, Vec2, Vec2, Vec2) -> Option<Vec<Vec2>>) -> Way {
    let straight = route_rivers(g, a, b, round);
    // (a way that goes round a meander, a lake or the tide keeps the way it had)
    if !straight.straight() || route(g, a, b, round).water != OnWater::None {
        return route(g, a, b, round);
    }
    let chord = (b - a).normalize_or(Vec2::X);
    // the points the curve passes through, in order along the road, with its direction at each
    let mut marks: Vec<(f32, Vec2, Vec2, f32)> = straight
        .crossings
        .iter()
        .map(|&(at, river, half, _, _)| {
            let mut square = river.perp();
            if square.dot(chord) < 0.0 {
                square = -square;
            }
            if square.dot(chord) < SQUARE_MOST {
                square = (square + chord).normalize_or(chord);
            }
            // straight across the bridge or the ford and its ramps, from bank to bank and a little past
            let reach = half / square.perp_dot(river).abs().max(0.34) + 0.0175 + APPROACH_PAST;
            ((at - a).dot(chord), at, square, reach)
        })
        .collect();
    marks.sort_by(|x, y| x.0.total_cmp(&y.0));
    let mut stops: Vec<(Vec2, Vec2)> = vec![(a, da)];
    for &(_, at, along, reach) in &marks {
        stops.push((at - along * reach, along));
        stops.push((at + along * reach, along));
    }
    stops.push((b, db));
    // (the straight lengths must leave room for the curve between them)
    if stops.windows(2).any(|w| (w[1].0 - w[0].0).dot(chord) < 0.03) {
        return route(g, a, b, round);
    }
    let curve = |stops: &[(Vec2, Vec2)], seek: &mut dyn FnMut(Vec2, Vec2, Vec2, Vec2) -> Option<Vec<Vec2>>| {
        let mut pts: Vec<Vec2> = vec![a];
        for (i, w) in stops.windows(2).enumerate() {
            // (across a bridge or a ford and its ramps, every second stretch, the way runs
            // straight and seeks nothing)
            match (i % 2 == 0).then(|| seek(w[0].0, w[0].1, w[1].0, w[1].1)).flatten() {
                Some(found) => pts.extend(found),
                None => {
                    let ends = crate::course::Ends { a: w[0].0, da: w[0].1, b: w[1].0, db: w[1].1 };
                    let n = ((w[0].0.distance(w[1].0) / 0.04).ceil() as usize).max(4);
                    pts.extend((1..=n).map(|i| ends.point(i as f32 / n as f32, 0.0)));
                }
            }
        }
        pts
    };
    let mut pts = curve(&stops, seek);
    // it crosses each river as often as the straight line it is drawn for does, and only there
    let count = |pts: &[Vec2]| {
        let mut by: Vec<(u32, usize)> = Vec::new();
        for w in pts.windows(2) {
            for (_, ri, _, _) in crate::rivers::crossing_points(&g.rivers, &g.water, w[0], w[1]) {
                match by.iter_mut().find(|x| x.0 == ri) {
                    Some(x) => x.1 += 1,
                    None => by.push((ri, 1)),
                }
            }
        }
        by.sort_unstable();
        by
    };
    let want = count(&[a, b]);
    // (a way sought over the ground that crosses a river more often, or stands in a lake or
    // the tide, gives way to the curve)
    if count(&pts) != want || (g.laid.is_some() && standing_on(g, &pts)) {
        pts = curve(&stops, &mut |_, _, _, _| None);
    }
    if count(&pts) != want {
        return route(g, a, b, round);
    }
    // each crossing as the straight line has it, crossed the way the curve runs there
    let crossings = straight.crossings.iter().map(|&(at, river, half, level, _)| (at, river, half, level, marks.iter().find(|m| m.1 == at).map_or(chord, |m| m.2))).collect();
    let way = Way { pts, crossings, water: OnWater::None };
    if g.laid.is_none() || !standing_on(g, &way.pts) {
        return way;
    }
    round_water(g, a, b, way)
}

/// The road from a link's way as a cart drives it (`driven`), for any way.
pub fn driven_on(g: &Ground, way: &Way, main: bool, crossed: Crossed, round: bool) -> Vec<Vec3> {
    if way.straight() {
        let (a, b) = (way.pts[0], way.pts[1]);
        return driven(g, (a.x, a.y), (b.x, b.y), main, crossed, round);
    }
    let bridges = if crossed == Crossed::Bridge { bridges_on(way, main, |_| Detail::Play) } else { Vec::new() };
    let fords = if crossed != Crossed::Bridge { fords_on(way, main, crossed) } else { Vec::new() };
    driven_round(g, way, &bridges, &fords)
}

/// The bridges on a link's way (`course`): one wherever it crosses a
/// river's middle line, each built at the level of detail `detail` gives
/// for its place. None where the run says the road crosses its water by a
/// ford or a ferry (`crossed`, from `Replay::crossed`: the core's
/// docs/rules/crossings.md).
pub fn bridges_of(way: &Way, main: bool, crossed: Crossed, detail: impl Fn(Vec2) -> Detail) -> Vec<RoadBridge> {
    if crossed != Crossed::Bridge {
        return Vec::new();
    }
    bridges_on(way, main, detail)
}

/// The fords (or a ferry's landings) on a link's way, where the run says it
/// crosses its water with no bridge: one wherever the way crosses a river's
/// middle line, reaching along the road as far as the water does.
pub fn fords_of(way: &Way, main: bool, crossed: Crossed) -> Vec<RoadFord> {
    if crossed == Crossed::Bridge {
        return Vec::new();
    }
    fords_on(way, main, crossed)
}

/// The way the road between cells `c` and `m` is drawn in year `k`: its
/// course as `road_course` fixed it the year it was first drawn.
pub fn course(r: &Replay, g: &Ground, cen: &[(f32, f32)], c: usize, m: usize, k: usize) -> Way {
    g.courses.way(r, g, cen, c, m, k)
}

/// The middle line of the ribbon of the road along `way`, as `road_line` drapes a straight one.
pub fn way_line(g: &Ground, way: &Way) -> Vec<Vec3> {
    let pts: Vec<(f32, f32)> = way.pts.iter().map(|p| (p.x, p.y)).collect();
    drape(g, &pts, ROAD_LIFT)
}

/// The bridges where a way crosses water.
fn bridges_on(way: &Way, main: bool, detail: impl Fn(Vec2) -> Detail) -> Vec<RoadBridge> {
    way.crossings
        .iter()
        .map(|&(at, river, half, level, along)| {
            // a road that crosses aslant has further to go from bank to bank (up to three times as far)
            let half = half / along.perp_dot(river).abs().max(0.34);
            RoadBridge { at, along, half, level, main, detail: detail(at) }
        })
        .collect()
}

/// The fords or ferries where a way crosses water.
fn fords_on(way: &Way, main: bool, crossed: Crossed) -> Vec<RoadFord> {
    way.crossings
        .iter()
        .map(|&(at, river, half, level, along)| {
            // a road that crosses aslant has further to go from bank to bank (up to three times as far)
            let reach = half / along.perp_dot(river).abs().max(0.34) + 0.004;
            RoadFord { at, along, reach, surface: level.max(0.0) + 0.009, main, ferry: crossed == Crossed::Ferry }
        })
        .collect()
}

/// Where the straight road from `a` to `c` runs over drowned ground (the
/// mouth of a river, a creek) and a causeway carries it at `CAUSEWAY_TOP`:
/// each stretch as how far along the road it starts and ends (0 at `a`, 1 at
/// `c`), from one dry end to the other and a little past each, looked at
/// every `step` along the road.
fn causeway_runs_every(g: &Ground, a: Vec2, c: Vec2, step: f32) -> Vec<(f32, f32)> {
    let len = a.distance(c);
    let n = ((len / step).ceil() as usize).max(2);
    let wet = |t: f32| {
        let p = a + (c - a) * t;
        world::height_at(g, p.x, p.y) < 0.004
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        if !wet(i as f32 / n as f32) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < n && wet(j as f32 / n as f32) {
            j += 1;
        }
        out.push((((i as f32 - 1.0) / n as f32).max(0.0), ((j as f32 + 0.5) / n as f32).min(1.0)));
        i = j;
    }
    out
}

/// The causeways along a road's way: on each of its straight lengths, where
/// it runs over drowned ground (`causeway_runs`), from where to where, and how
/// high its top stands: `CAUSEWAY_TOP` over the tide, and as much over a
/// lake's water where the way crosses a lake (`OnWater::Causeway`).
pub fn causeways(g: &Ground, way: &Way) -> Vec<(Vec2, Vec2, f32)> {
    let mut out = Vec::new();
    // (a way that meets standing water is looked along twice as closely, so that no narrow
    // reach of water slips between two looks)
    let step = if way.water == OnWater::None { 0.02 } else { 0.01 };
    for w in way.pts.windows(2) {
        for (t0, t1) in causeway_runs_every(g, w[0], w[1], step) {
            let (p, q) = (w[0].lerp(w[1], t0), w[0].lerp(w[1], t1));
            out.push((p, q, water_top(g, p.lerp(q, 0.5)) + CAUSEWAY_TOP));
        }
    }
    out
}

/// The height of the water's surface at a spot under standing water: a
/// lake's sheet where one lies over it, else the sea's, nought.
fn water_top(g: &Ground, p: Vec2) -> f32 {
    let Some(laid) = &g.laid else { return 0.0 };
    let h = world::grid_at(&g.hgt, g.gw, g.gh, p.x, p.y);
    laid.lake_over(h, p.x, p.y).unwrap_or(0.0)
}

/// The road from `a` to `b` as a cart drives it: its ribbon's middle line,
/// at the height of whatever is on top there, the ribbon itself, a bridge's
/// roadway (up its ramp, over the hump and down again) or a causeway. Each
/// is straight from one of its points to the next, so the surface is too,
/// and the points returned are every place where one of them bends or one
/// rises above another. The bridges are read at the play distance's detail,
/// whose roadway is the same as the near one's; from far off a bridge's
/// roadway is drawn in fewer lengths, and the cart rides a shade above it.
pub fn driven(g: &Ground, a: (f32, f32), b: (f32, f32), main: bool, crossed: Crossed, round: bool) -> Vec<Vec3> {
    let (pa, pb) = (Vec2::new(a.0, a.1), Vec2::new(b.0, b.1));
    let way = route(g, pa, pb, round);
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let bridges = if crossed == Crossed::Bridge { bridges_on(&way, main, |_| Detail::Play) } else { Vec::new() };
    let fords = if crossed != Crossed::Bridge { fords_on(&way, main, crossed) } else { Vec::new() };
    if !way.straight() {
        return driven_round(g, &way, &bridges, &fords);
    }
    let ribbon = road_line(g, a, b);
    let len = pa.distance(pb);
    if len < 1e-6 {
        return ribbon;
    }
    let dir = (pb - pa) / len;
    let t_of = |p: Vec2| (p - pa).dot(dir) / len;
    let mut layers: Vec<Vec<(f32, f32)>> = vec![ribbon.iter().map(|p| (t_of(Vec2::new(p.x, p.z)), p.y)).collect()];
    for br in &bridges {
        layers.push(br.roadway(&height).into_iter().map(|(t, h)| (t_of(br.point(t)), h)).collect());
    }
    // (a cart wades a ford a hair over the water, on its bar of gravel; a ferry carries it at the same height)
    for f in &fords {
        layers.push(vec![(t_of(f.point(0.0)), f.wade()), (t_of(f.point(1.0)), f.wade())]);
    }
    for (p, q, top) in causeways(g, &way) {
        layers.push(vec![(t_of(p), top), (t_of(q), top)]);
    }
    envelope(&layers).into_iter().map(|(t, h)| {
        let p = pa + dir * (len * t);
        Vec3::new(p.x, h, p.y)
    }).collect()
}

/// `driven` for a way that goes round a meander, or round a lake's shore or
/// a tideway's: the same layers, read along the way by how far along it they
/// lie.
fn driven_round(g: &Ground, way: &Way, bridges: &[RoadBridge], fords: &[RoadFord]) -> Vec<Vec3> {
    let pts = &way.pts;
    let mut cum = vec![0.0f32];
    for w in pts.windows(2) {
        cum.push(cum[cum.len() - 1] + w[0].distance(w[1]));
    }
    let total = cum[cum.len() - 1].max(1e-6);
    // how far along the way a point of the map lies, by the nearest of its lengths
    let t_of = |p: Vec2| {
        let mut best = (f32::MAX, 0.0f32);
        for i in 0..pts.len() - 1 {
            let (q, r) = (pts[i], pts[i + 1]);
            let len2 = q.distance_squared(r).max(1e-12);
            let u = ((p - q).dot(r - q) / len2).clamp(0.0, 1.0);
            let d = p.distance_squared(q.lerp(r, u));
            if d < best.0 {
                best = (d, cum[i] + u * (cum[i + 1] - cum[i]));
            }
        }
        best.1 / total
    };
    let at = |t: f32| {
        let s = t * total;
        let i = cum.partition_point(|&c| c < s).clamp(1, pts.len() - 1);
        let u = ((s - cum[i - 1]) / (cum[i] - cum[i - 1]).max(1e-9)).clamp(0.0, 1.0);
        pts[i - 1].lerp(pts[i], u)
    };
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let ribbon = way_line(g, way);
    let mut layers: Vec<Vec<(f32, f32)>> = vec![ribbon.iter().map(|p| (t_of(Vec2::new(p.x, p.z)), p.y)).collect()];
    for br in bridges {
        layers.push(br.roadway(&height).into_iter().map(|(t, h)| (t_of(br.point(t)), h)).collect());
    }
    for f in fords {
        let (t0, t1) = (t_of(f.point(0.0)), t_of(f.point(1.0)));
        layers.push(vec![(t0.min(t1), f.wade()), (t0.max(t1), f.wade())]);
    }
    for (p, q, top) in causeways(g, way) {
        let (t0, t1) = (t_of(p), t_of(q));
        layers.push(vec![(t0.min(t1), top), (t0.max(t1), top)]);
    }
    for l in layers.iter_mut() {
        l.sort_by(|x, y| x.0.total_cmp(&y.0));
    }
    envelope(&layers).into_iter().map(|(t, h)| {
        let p = at(t);
        Vec3::new(p.x, h, p.y)
    }).collect()
}

/// The top of several surfaces along a road, each given as heights at points
/// along it (0 to 1) and straight between them, and nothing outside its first
/// and last: the height of the highest at every point where one of them
/// bends, begins or ends, or two of them cross, from 0 to 1. Where one begins
/// or ends standing above the rest (the foot of a bridge's ramp where the
/// river's level stands above the bank), the top steps there, within `STEP`
/// of the way along, rather than sloping across what lies below.
fn envelope(layers: &[Vec<(f32, f32)>]) -> Vec<(f32, f32)> {
    const EPS: f32 = 1e-6;
    const STEP: f32 = 1e-4;
    let at = |l: &[(f32, f32)], t: f32| -> Option<f32> {
        let (first, last) = (l.first()?, l.last()?);
        if t < first.0 - EPS || t > last.0 + EPS {
            return None;
        }
        if l.len() == 1 {
            return Some(first.1);
        }
        let i = l.partition_point(|p| p.0 < t).clamp(1, l.len() - 1);
        let ((t0, h0), (t1, h1)) = (l[i - 1], l[i]);
        Some(if t1 - t0 > EPS { h0 + (h1 - h0) * ((t - t0) / (t1 - t0)).clamp(0.0, 1.0) } else { h1 })
    };
    // (and just outside where each begins and ends, for the step)
    let edges = layers.iter().filter_map(|l| Some([l.first()?.0 - STEP, l.last()?.0 + STEP])).flatten();
    let mut ts: Vec<f32> = layers.iter().flatten().map(|p| p.0).chain(edges).filter(|t| (0.0..=1.0).contains(t)).collect();
    ts.extend([0.0, 1.0]);
    ts.sort_by(f32::total_cmp);
    ts.dedup_by(|x, y| (*x - *y).abs() < EPS);
    // where two surfaces cross between two of those points, the top bends there too
    let mut more = Vec::new();
    for w in ts.windows(2) {
        let (t0, t1) = (w[0], w[1]);
        let ends: Vec<(f32, f32)> = layers.iter().filter_map(|l| Some((at(l, t0)?, at(l, t1)?))).collect();
        for i in 0..ends.len() {
            for j in i + 1..ends.len() {
                let (d0, d1) = (ends[i].0 - ends[j].0, ends[i].1 - ends[j].1);
                if d0 * d1 < 0.0 {
                    more.push(t0 + (t1 - t0) * d0 / (d0 - d1));
                }
            }
        }
    }
    ts.extend(more);
    ts.sort_by(f32::total_cmp);
    ts.dedup_by(|x, y| (*x - *y).abs() < EPS);
    ts.into_iter().filter_map(|t| Some((t, layers.iter().filter_map(|l| at(l, t)).reduce(f32::max)?))).collect()
}

/// The height of a line of points at `t` of the way from its first to its
/// last, measured in plan along the straight line between them (as for a
/// road's line from `driven`).
pub fn height_along(pts: &[Vec3], t: f32) -> f32 {
    let (a, b) = (Vec2::new(pts[0].x, pts[0].z), Vec2::new(pts[pts.len() - 1].x, pts[pts.len() - 1].z));
    let (dir, len) = ((b - a).normalize_or(Vec2::X), a.distance(b).max(1e-6));
    let ts: Vec<f32> = pts.iter().map(|p| (Vec2::new(p.x, p.z) - a).dot(dir) / len).collect();
    let i = ts.partition_point(|&s| s < t).clamp(1, pts.len() - 1);
    let (t0, t1) = (ts[i - 1], ts[i]);
    let u = if t1 - t0 > 1e-6 { ((t - t0) / (t1 - t0)).clamp(0.0, 1.0) } else { 1.0 };
    pts[i - 1].y + (pts[i].y - pts[i - 1].y) * u
}

/// The water that shows on the map, for asking where a road meets it: the
/// sea, and the rivers' water as `rivers::ribbons` draws it, bucketed by
/// cell (each length of a river whose two ends both lie above the sea, with
/// its half width and its surface at either end).
pub struct Waters {
    w: usize,
    h: usize,
    cells: std::collections::HashMap<usize, Vec<(Vec2, Vec2, f32, f32, f32, f32)>>,
}

impl Waters {
    pub fn new(r: &Replay, g: &Ground) -> Waters {
        let mut cells: std::collections::HashMap<usize, Vec<(Vec2, Vec2, f32, f32, f32, f32)>> = Default::default();
        for rv in &g.rivers {
            for i in 1..rv.pts.len() {
                // drawn where the level stands above the sea, its surface 0.009 over the level
                if rv.level[i - 1] <= 0.004 || rv.level[i] <= 0.004 {
                    continue;
                }
                let seg = (rv.pts[i - 1], rv.pts[i], rv.half[i - 1], rv.half[i], rv.level[i - 1].max(0.0) + 0.009, rv.level[i].max(0.0) + 0.009);
                let reach = rv.half[i - 1].max(rv.half[i]) + 0.02;
                let (lo, hi) = (rv.pts[i - 1].min(rv.pts[i]) - Vec2::splat(reach), rv.pts[i - 1].max(rv.pts[i]) + Vec2::splat(reach));
                for y in (lo.y.floor().max(0.0) as usize)..=(hi.y.floor().max(0.0) as usize).min(r.h - 1) {
                    for x in (lo.x.floor().max(0.0) as usize)..=(hi.x.floor().max(0.0) as usize).min(r.w - 1) {
                        cells.entry(y * r.w + x).or_default().push(seg);
                    }
                }
            }
        }
        Waters { w: r.w, h: r.h, cells }
    }

    /// The water that shows at a spot, if any: its surface's height, and
    /// whether it is the sea. A river's water is drawn flat across its width,
    /// and where the bank stands higher than it the ground hides it, so it
    /// counts only where it lies above the ground.
    pub fn at(&self, g: &Ground, p: Vec2) -> Option<(f32, bool)> {
        let ground = world::height_at(g, p.x, p.y);
        if ground < 0.0 {
            return Some((0.0, true));
        }
        let (x, y) = (p.x.floor().clamp(0.0, self.w as f32 - 1.0) as usize, p.y.floor().clamp(0.0, self.h as f32 - 1.0) as usize);
        let mut top: Option<f32> = None;
        for &(a, b, ha, hb, la, lb) in self.cells.get(&(y * self.w + x)).into_iter().flatten() {
            let ab = b - a;
            let u = ((p - a).dot(ab) / ab.length_squared().max(1e-9)).clamp(0.0, 1.0);
            if p.distance(a + ab * u) < ha + (hb - ha) * u {
                let level = la + (lb - la) * u;
                top = Some(top.map_or(level, |t: f32| t.max(level)));
            }
        }
        top.filter(|&t| t > ground).map(|t| (t, false))
    }

    /// Whether any river's water lies near the straight road from `a` to `b`
    /// (in the cells under it: those its two ends lie in and, on a diagonal,
    /// the two beside them). Only a river can drown a road, as the ribbon lies
    /// at least `ROAD_LIFT` above the sea, and most roads have none near.
    pub fn near(&self, a: Vec2, b: Vec2) -> bool {
        let (lo, hi) = (a.min(b), b.max(a));
        (lo.y.floor().max(0.0) as usize..=(hi.y.floor().max(0.0) as usize).min(self.h - 1))
            .any(|y| (lo.x.floor().max(0.0) as usize..=(hi.x.floor().max(0.0) as usize).min(self.w - 1)).any(|x| self.cells.contains_key(&(y * self.w + x))))
    }

    /// Whether a road's line from `driven` goes under the water anywhere:
    /// where it runs along a river's channel inside its water, with no bridge
    /// or causeway to carry it, the ribbon lies on the channel's bed. Looked
    /// at every 0.01 cells, on the middle line and where a cart keeps to
    /// either side of it (`carts::SIDE`): the edge of a river's water can
    /// stand over the road beside the middle line and not on it (island 7
    /// with the shelves going with the district, docs/rules/shelves.md, has
    /// a cart 0.0009 under the water at 94.05,28.53 in 1850 otherwise).
    pub fn drowns(&self, g: &Ground, pts: &[Vec3]) -> bool {
        let (a, b) = (Vec2::new(pts[0].x, pts[0].z), Vec2::new(pts[pts.len() - 1].x, pts[pts.len() - 1].z));
        // crossings: a way that goes round a meander is read along its own lengths
        let off = |p: &Vec3| (Vec2::new(p.x, p.z) - a).perp_dot((b - a).normalize_or(Vec2::X)).abs();
        if pts.iter().any(|p| off(p) > 1e-3) {
            return pts.windows(2).any(|w| {
                let (p, q) = (Vec2::new(w[0].x, w[0].z), Vec2::new(w[1].x, w[1].z));
                let across = (q - p).normalize_or(Vec2::X).perp() * crate::carts::SIDE;
                let n = ((p.distance(q) / 0.01).ceil() as usize).max(1);
                (0..=n).any(|i| {
                    let t = i as f32 / n as f32;
                    let (s, h) = (p.lerp(q, t), w[0].y + (w[1].y - w[0].y) * t);
                    [s, s + across, s - across].into_iter().any(|x| self.at(g, x).is_some_and(|(top, _)| h < top))
                })
            });
        }
        if !self.near(a, b) {
            return false;
        }
        let across = (b - a).normalize_or(Vec2::X).perp() * crate::carts::SIDE;
        let n = ((a.distance(b) / 0.01).ceil() as usize).max(1);
        (0..=n).any(|i| {
            let t = i as f32 / n as f32;
            let p = a + (b - a) * t;
            [p, p + across, p - across].into_iter().any(|q| self.at(g, q).is_some_and(|(top, _)| height_along(pts, t) < top))
        })
    }
}

/// The rivers: water and wet banks, lying on each river's own level (see
/// rivers.rs). On the new land the lakes are with them, each a sheet of the
/// rivers' water at its level (newland/ground.rs), so that a lake goes to
/// steel in the depth of winter with the rivers that feed it.
pub fn rivers(g: &Ground, lk: &Look) -> Builder {
    let mut b = crate::rivers::ribbons(&g.rivers, lk);
    if let Some(laid) = &g.laid {
        laid.sheets(&mut b, *lk.water.river);
    }
    b
}

pub struct Nets {
    pub roads: Builder,
    pub rails: Builder,
}

/// The roads drawn in year `k`: (cell, neighbouring cell, level 2 or 3). Each
/// link appears once; a diagonal is left out where a pair of straight links
/// already joins the two cells.
pub fn road_segments(r: &Replay, k: usize) -> Vec<(usize, usize, u8)> {
    let (w, h, n) = (r.w, r.h, r.n);
    let tr = &r.trans[k * n..(k + 1) * n];
    let bit = |x: isize, y: isize| -> bool { x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && tr[y as usize * w + x as usize] & 2 != 0 };
    let mut out = Vec::new();
    for c in 0..n {
        let v = tr[c];
        if v & 3 < 2 {
            continue;
        }
        let (x, y) = ((c % w) as isize, (c / w) as isize);
        for (dx, dy) in [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)] {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || nx >= w as isize || ny >= h as isize {
                continue;
            }
            let m = ny as usize * w + nx as usize;
            let lv = (v & 3).min(tr[m] & 3);
            let diag = dx != 0 && dy != 0;
            if lv >= 2 && !(diag && (bit(x + dx, y) || bit(x, y + dy))) {
                out.push((c, m, lv));
            }
        }
    }
    out
}

/// What a link must carry by cart in a year for that year to wear it, in
/// the goods' units of weight: the simulation's own first threshold, at
/// which a square's traffic makes it a track (`road_thresholds`, the core's
/// params.rs), here read on the link (the core's docs/rules/traffic.md).
pub const WORN_WEIGHT: f32 = 1500.0;
/// In how many years of the last `WORN_WINDOW` a link must carry
/// `WORN_WEIGHT` by cart for the carts to have worn a road along it.
pub const WORN_YEARS: u16 = 10;
pub const WORN_WINDOW: u16 = 20;
/// A worn road goes back to grass in the picture once fewer than this many
/// of the last `WORN_WINDOW` years wore it: after twenty years in which it
/// never carried `WORN_WEIGHT`. The simulation's road stays (its squares
/// keep their level); what goes is the link the picture drew it on. The
/// simulation's trees of cheapest paths move from one link to a parallel one
/// over the years, and a road kept on every link it ever wore piles three
/// centuries of them into a net (on island 7 in 2050, 7,170 links, as many
/// as every road between road squares; `drawn_links`).
pub const WORN_FADE: u16 = 1;
/// A year a link carries this much by cart, a turnpike's traffic (the
/// core's third threshold), wears it as `HEAVY_COUNTS` years do: a new way
/// that takes a heavy flow at once (to a new pit, a new works) is worn in
/// two such years.
pub const WORN_HEAVY: f32 = 40_000.0;
pub const HEAVY_COUNTS: u16 = 5;

/// A rule of wear: a link is worn once it has carried `weight` or more by
/// cart in `years` of the last `window` years (every year so far where
/// `window` is 0), a year of `WORN_HEAVY` counting `HEAVY_COUNTS`; and a
/// worn link goes back to grass once fewer than `fade` of the last `window`
/// years wore it (never where `fade` is 0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WornRule {
    pub weight: f32,
    pub years: u16,
    pub window: u16,
    pub fade: u16,
}

/// The roads the carts wore, as far as the record has been gone through:
/// for each link (`c * 4 + slot`, the core's `traffic::SLOTS`), whether it
/// was worn in each year. Kept in `Ground::worn`.
pub struct Worn {
    known: std::sync::Mutex<WornKnown>,
    rule: WornRule,
}

impl Default for Worn {
    fn default() -> Worn {
        Worn::by(WornRule { weight: WORN_WEIGHT, years: WORN_YEARS, window: WORN_WINDOW, fade: WORN_FADE })
    }
}

#[derive(Default)]
struct WornKnown {
    /// how many years have been gone through
    through: usize,
    /// for each link, the years of wear so far (with no window), or the last
    /// 32 years as bits, the latest lowest: worn, and worn heavily
    count: Vec<u16>,
    bits: Vec<u32>,
    heavy: Vec<u32>,
    /// for each link, whether it is worn now
    on: Vec<bool>,
    /// for each year, the links worn, as bits
    years: Vec<Vec<u64>>,
}

impl WornKnown {
    fn worn(&self, k: usize, l: usize) -> bool {
        self.years.get(k).is_some_and(|b| b[l / 64] >> (l % 64) & 1 != 0)
    }
}

impl Worn {
    /// Wear by another rule than the game's (for the measure, `tests::measure_a_recorded_run`).
    pub fn by(rule: WornRule) -> Worn {
        Worn { known: Default::default(), rule }
    }

    /// An edit taken back (back.rs): the years from `from` (counted from the
    /// start) are gone through again when next asked for. The counts cannot
    /// be unwound a year at a time, so everything is gone through again.
    pub fn forget_from(&self, from: usize) {
        if let Ok(mut known) = self.known.lock() {
            if from < known.through {
                *known = WornKnown::default();
            }
        }
    }
}

/// Whether cell `c` is a town's ground in year `k`, as the town's streets
/// are laid on it (`scatter::towns`).
pub(crate) fn town_ground(r: &Replay, k: usize, c: usize) -> bool {
    let off = k * r.n + c;
    if r.kept() {
        r.built[off] > 0
    } else {
        r.use_[off] == 10 || r.mix_a[off] == world::URBAN || r.mix_b[off] == world::URBAN
    }
}

thread_local! {
    /// In a test: draw every road by `road_segments`' rule, as before the
    /// roads the carts wore (as `ISOSTATE_WORN=off` does for a run).
    pub static ALL_IN_TEST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the roads are drawn by wear (`drawn_links`): unless
/// `ISOSTATE_WORN=off` asks for the picture before, to set the two side by side.
pub fn by_wear() -> bool {
    static OFF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    !ALL_IN_TEST.with(|a| a.get()) && !*OFF.get_or_init(|| std::env::var("ISOSTATE_WORN").is_ok_and(|v| v == "off"))
}

/// The roads drawn in year `k`, as (cell, neighbouring cell, level 2 or 3),
/// each link once and in the order `road_segments` gives them. A link
/// between two squares of made road or turnpike is drawn where the carts
/// wore it (`Worn`): where it has carried `WORN_WEIGHT` or more by cart in
/// `WORN_YEARS` of the last `WORN_WINDOW` years (a year of `WORN_HEAVY` or
/// more counting `HEAVY_COUNTS`; in the record's first years, in every year
/// so far), a diagonal as a diagonal; and once worn it stays drawn while
/// both its squares are made roads, until twenty years go by in which it
/// never carries `WORN_WEIGHT` (`WORN_FADE`). A trunk road's or a bypass's
/// link is drawn where `road_segments`' rule draws it, as always, so that
/// the state's roads keep their lines, and as a diagonal too where the carts
/// wore one. A run that kept no traffic (the port's rules, or a record made
/// before 25 November 2026) has every road drawn by `road_segments`, and so
/// does a year past what the record holds.
///
/// A town's streets are laid along every road through its ground, worn or
/// not (`scatter::towns`), so that no street or house moves; where the
/// carts did not wear one, the town draws it as its own street, along its
/// houses and its built-up ground (`kept.rs`), and the country beyond has no
/// road there.
///
/// Which links are drawn is decided here alone; how a link is drawn (its
/// course, ribbon, bridges) is `route` and `year_nets`.
pub fn drawn_links(r: &Replay, worn: &Worn, k: usize) -> Vec<(usize, usize, u8)> {
    let today = road_segments(r, k);
    if !r.traffic.kept() || k >= r.filled.min(r.ny) || !by_wear() {
        return today;
    }
    let mut known = worn.known.lock().unwrap();
    wear_through(r, worn, &mut known, k);
    let (w, h, n) = (r.w, r.h, r.n);
    let tr = &r.trans[k * n..(k + 1) * n];
    let state = |c: usize, m: usize| (tr[c] & 64 != 0 && tr[m] & 64 != 0) || (tr[c] & 128 != 0 && tr[m] & 128 != 0);
    let mut today = today.into_iter().peekable();
    let mut out = Vec::new();
    for c in 0..n {
        if tr[c] & 3 < 2 {
            continue;
        }
        let (x, y) = ((c % w) as isize, (c / w) as isize);
        for (s, (dx, dy)) in isostate_core::traffic::SLOTS.into_iter().enumerate() {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || nx >= w as isize || ny >= h as isize {
                continue;
            }
            let m = ny as usize * w + nx as usize;
            let lv = (tr[c] & 3).min(tr[m] & 3);
            if lv < 2 {
                continue;
            }
            // (road_segments gives the same links in the same order, less some diagonals)
            let drawn_today = today.next_if(|&(a, b, _)| a == c && b == m).is_some();
            let keep = (state(c, m) && drawn_today) || known.worn(k, c * 4 + s);
            if keep {
                out.push((c, m, lv));
            }
        }
    }
    out
}

/// Go through the record's years up to `k` for `Worn`. In the record's
/// first `WORN_YEARS` years a link is worn by carrying `WORN_WEIGHT` in
/// every year so far: the island's roads are older than the record, and
/// what the carts use from its first year is all there is to go by.
fn wear_through(r: &Replay, worn: &Worn, known: &mut WornKnown, k: usize) {
    let (n, rule) = (r.n, worn.rule);
    if known.on.len() != n * 4 {
        *known = WornKnown { through: 0, count: vec![0; n * 4], bits: vec![0; n * 4], heavy: vec![0; n * 4], on: vec![false; n * 4], years: Vec::new() };
    }
    let window = rule.window.min(32);
    let mask = if window == 0 { 0 } else { u32::MAX >> (32 - window) };
    while known.through <= k {
        let y = known.through;
        // (in the record's first years, every year so far)
        let need = rule.years.min(y as u16 + 1);
        let full = window == 0 || y + 1 >= window as usize;
        let mut year = vec![0u64; (n * 4).div_ceil(64)];
        for c in 0..n {
            for s in 0..4 {
                let l = c * 4 + s;
                let w = if r.traffic.mode(n, y, c, s) == isostate_core::traffic::CART { r.traffic.weight(n, y, c, s) } else { 0.0 };
                let (wore, heavy) = (w >= rule.weight, w >= WORN_HEAVY.max(rule.weight));
                let count = if window == 0 {
                    known.count[l] = known.count[l].saturating_add(if heavy { HEAVY_COUNTS } else { wore as u16 });
                    known.count[l]
                } else {
                    known.bits[l] = (known.bits[l] << 1) | wore as u32;
                    known.heavy[l] = (known.heavy[l] << 1) | heavy as u32;
                    ((known.bits[l] & mask).count_ones() + (HEAVY_COUNTS as u32 - 1) * (known.heavy[l] & mask).count_ones()) as u16
                };
                if !known.on[l] && count >= need {
                    known.on[l] = true;
                } else if known.on[l] && rule.fade > 0 && full && count < rule.fade {
                    known.on[l] = false;
                }
                if known.on[l] {
                    year[l / 64] |= 1 << (l % 64);
                }
            }
        }
        known.years.push(year);
        known.through += 1;
    }
}

/// How many squares of four cells the links close: blocks of four cells
/// whose four sides (the links between them east and south) are all drawn,
/// a square 2 km on a side. The measure of the lattice in the picture.
pub fn closed_squares(w: usize, h: usize, links: &[(usize, usize, u8)]) -> usize {
    let mut side = vec![[false; 2]; w * h];
    for &(c, m, _) in links {
        if m == c + 1 {
            side[c][0] = true;
        } else if m == c + w {
            side[c][1] = true;
        }
    }
    (0..w * h).filter(|&c| c % w + 1 < w && c / w + 1 < h && side[c][0] && side[c][1] && side[c + 1][1] && side[c + w][0]).count()
}

/// The roads of a year measured (`measure`): how many links today's rule
/// (`road_segments`) draws and how many squares they close, the same for
/// the links the carts wore (`drawn_links`), and where the year's carting
/// went: the weight carried by cart times the length of the link, on the
/// links drawn, on made roads not drawn, and on tracks and open ground.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RoadMeasure {
    pub year: i32,
    pub today: usize,
    pub today_closed: usize,
    pub drawn: usize,
    pub closed: usize,
    /// the closed squares with a cell of town ground among their four
    pub closed_town: usize,
    /// diagonals drawn that today's rule leaves out
    pub diagonals: usize,
    pub carted_drawn: f64,
    pub carted_road_not_drawn: f64,
    pub carted_track: f64,
}

/// Measure the roads of year `k` (`RoadMeasure`), by the rule `worn`.
pub fn measure(r: &Replay, worn: &Worn, k: usize) -> RoadMeasure {
    let today = road_segments(r, k);
    let drawn = drawn_links(r, worn, k);
    let (w, n) = (r.w, r.n);
    let tr = &r.trans[k * n..(k + 1) * n];
    let on_town = |c: usize| town_ground(r, k, c);
    let mut side = vec![[false; 2]; n];
    for &(c, m, _) in &drawn {
        if m == c + 1 {
            side[c][0] = true;
        } else if m == c + w {
            side[c][1] = true;
        }
    }
    let closed: Vec<usize> = (0..n).filter(|&c| c % w + 1 < w && c / w + 1 < r.h && side[c][0] && side[c][1] && side[c + 1][1] && side[c + w][0]).collect();
    let set: std::collections::HashSet<(usize, usize)> = drawn.iter().map(|&(c, m, _)| (c, m)).collect();
    let was: std::collections::HashSet<(usize, usize)> = today.iter().map(|&(c, m, _)| (c, m)).collect();
    let mut out = RoadMeasure {
        year: r.y0 + k as i32,
        today: today.len(),
        today_closed: closed_squares(w, r.h, &today),
        drawn: drawn.len(),
        closed: closed.len(),
        closed_town: closed.iter().filter(|&&c| [c, c + 1, c + w, c + w + 1].into_iter().any(on_town)).count(),
        diagonals: drawn.iter().filter(|&&(c, m, _)| m != c + 1 && m != c + w && !was.contains(&(c, m))).count(),
        ..Default::default()
    };
    if !r.traffic.kept() {
        return out;
    }
    for c in 0..n {
        for s in 0..4 {
            let Some(m) = isostate_core::traffic::other_end(w, r.h, c, s) else { continue };
            if r.traffic.mode(n, k, c, s) != isostate_core::traffic::CART {
                continue;
            }
            let carted = r.traffic.weight(n, k, c, s) as f64 * isostate_core::traffic::slot_len(s);
            if set.contains(&(c, m)) {
                out.carted_drawn += carted;
            } else if (tr[c] & 3).min(tr[m] & 3) >= 2 {
                out.carted_road_not_drawn += carted;
            } else {
                out.carted_track += carted;
            }
        }
    }
    out
}

/// Where the railway has sidings in year `k`: two at a big town's station,
/// one at a middling town's, and one in a cell where three works or more
/// stand by the line.
fn sidings(r: &Replay, cen: &[(f32, f32)], k: usize) -> Vec<crate::rails::Sidings> {
    let railed = |c: usize| r.trans[k * r.n + c] & 4 != 0;
    let middle = |c: usize| bevy::math::Vec2::new(cen[c].0, cen[c].1);
    let mut out: Vec<crate::rails::Sidings> = Vec::new();
    for t in 0..r.n_towns.min(r.header.towns.len()) {
        let node = r.header.towns[t].node;
        let urban = r.town_urban[k * r.n_towns + t];
        if r.town_active[k * r.n_towns + t] != 0 && railed(node) && urban > 3000.0 {
            out.push((node, if urban > 12000.0 { 2 } else { 1 }, middle(node)));
        }
    }
    let mut works: std::collections::HashMap<usize, usize> = Default::default();
    for (f, firm) in r.header.firms.iter().enumerate() {
        let (first, len, off) = (r.firm_first[f].max(0) as usize, r.firm_len[f].max(0) as usize, r.firm_off[f].max(0) as usize);
        if railed(firm.cell) && k >= first && k < first + len && r.firm_k[off + k - first] > 0.0 {
            *works.entry(firm.cell).or_default() += 1;
        }
    }
    let mut districts: Vec<usize> = works.into_iter().filter(|&(c, n)| n >= 3 && !out.iter().any(|s| s.0 == c)).map(|(c, _)| c).collect();
    districts.sort_unstable();
    out.extend(districts.into_iter().map(|c| (c, 1, middle(c))));
    out
}

/// What the roads of year `k` do about standing water (`ISOSTATE_ROADS`,
/// from `carts::maybe_check`), as lines to print: how many links the picture
/// before this stage left out, how many now go round and how many cross on a
/// causeway (with the longest), how those links cross rivers, and how many
/// lie on standing water with nothing under them (none, if all is well);
/// and how long it takes to find every link's way that year with none of the
/// ways round water kept from before.
pub fn water_check(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> Vec<String> {
    g.detours.forget();
    let t0 = std::time::Instant::now();
    let segs = drawn_links(r, &g.worn, k);
    for &(c, m, _) in &segs {
        route(g, Vec2::new(cen[c].0, cen[c].1), Vec2::new(cen[m].0, cen[m].1), r.round());
    }
    let cold = t0.elapsed().as_secs_f64() * 1000.0;
    let t0 = std::time::Instant::now();
    for &(c, m, _) in &segs {
        route(g, Vec2::new(cen[c].0, cen[c].1), Vec2::new(cen[m].0, cen[m].1), r.round());
    }
    let warm = t0.elapsed().as_secs_f64() * 1000.0;
    let (mut links, mut main) = (0, 0);
    let (mut left, mut left_main) = (0, 0);
    let (mut wet, mut wet_main, mut roundw, mut round_main, mut over, mut over_main) = (0, 0, 0, 0, 0, 0);
    let (mut bare, mut bare_main, mut bare_len) = (0, 0, 0.0f32);
    let mut kinds = [0usize; 3];
    let mut extra = 0.0f32;
    let mut faults: Vec<(f32, Vec2, Vec2, bool, bool)> = Vec::new();
    let mut over_lake = 0;
    let mut rounds: Vec<(f32, Vec2, bool)> = Vec::new();
    let mut bares: Vec<(f32, Vec2, OnWater)> = Vec::new();
    for (c, m, lv) in segs {
        let (a, b) = (Vec2::new(cen[c].0, cen[c].1), Vec2::new(cen[m].0, cen[m].1));
        links += 1;
        main += (lv == 3) as usize;
        if was_left_out(g, a, b) {
            left += 1;
            left_main += (lv == 3) as usize;
        }
        let way = route(g, a, b, r.round());
        if way.water == OnWater::None {
            continue;
        }
        wet += 1;
        wet_main += (lv == 3) as usize;
        let len = |pts: &[Vec2]| pts.windows(2).map(|w| w[0].distance(w[1])).sum::<f32>();
        let further = len(&way.pts) - len(&route_rivers(g, a, b, r.round()).pts);
        if !way.crossings.is_empty() {
            kinds[r.crossed(c, m, k) as usize] += 1;
        }
        let runs = causeways(g, &way);
        match way.water {
            OnWater::Round => {
                roundw += 1;
                round_main += (lv == 3) as usize;
                extra += further;
                rounds.push((further, a.lerp(b, 0.5), lv == 3));
            }
            _ => {
                over += 1;
                over_main += (lv == 3) as usize;
                let water: f32 = runs.iter().map(|(p, q, _)| p.distance(*q)).sum();
                // (over a lake where a causeway's top stands above the sea's)
                let lake = runs.iter().any(|r| r.2 > CAUSEWAY_TOP + 1e-4);
                over_lake += lake as usize;
                faults.push((water, runs.first().map_or(a, |x| x.0), a.lerp(b, 0.5), lv == 3, lake));
            }
        }
        // on standing water with no causeway under it, looked at every 0.01 of a cell
        let mut here = 0.0f32;
        for w in way.pts.windows(2) {
            let n = ((w[0].distance(w[1]) / 0.01).ceil() as usize).max(1);
            for i in 0..=n {
                let p = w[0].lerp(w[1], i as f32 / n as f32);
                let carried = runs.iter().any(|&(s, e, _)| {
                    let se = e - s;
                    let u = (p - s).dot(se) / se.length_squared().max(1e-12);
                    (-1e-3..=1.0 + 1e-3).contains(&u) && p.distance(s + se * u.clamp(0.0, 1.0)) < 1e-3
                });
                if world::height_at(g, p.x, p.y) < 0.0 && !carried {
                    here += w[0].distance(w[1]) / n as f32;
                }
            }
        }
        if here > 0.0 {
            bares.push((here, a.lerp(b, 0.5), way.water));
            bare += 1;
            bare_main += (lv == 3) as usize;
            bare_len += here;
        }
    }
    faults.sort_by(|x, y| y.0.total_cmp(&x.0));
    let at = |p: Vec2| format!("{:.2},{:.2}", p.x, p.y);
    let year = r.y0 + k as i32;
    let mut out = vec![
        format!("water {year}: {links} links ({main} main); the picture before left out {left} ({left_main} main); standing water on the way of {wet} ({wet_main} main): {roundw} ({round_main} main) go round on dry ground, {:.1} cells further in all, and {over} ({over_main} main) cross on a causeway ({over_lake} of them over a lake), {:.2} cells of it; of those links, {} also cross a river on a bridge, {} by a ford, {} by a ferry; on standing water with nothing under them: {bare} links ({bare_main} main), {bare_len:.3} cells; every link's way found in {cold:.1} ms with no way round water kept, {warm:.1} ms with them kept",
            extra, faults.iter().map(|f| f.0).sum::<f32>(), kinds[Crossed::Bridge as usize], kinds[Crossed::Ford as usize], kinds[Crossed::Ferry as usize]),
    ];
    for (water, p, mid, is_main, lake) in &faults {
        out.push(format!("  causeway at {} for {:.3} cells over {}, on the link through {}{}", at(*p), water, if *lake { "a lake" } else { "the tide" }, at(*mid), if *is_main { ", a main road" } else { "" }));
    }
    for (len, mid, water) in &bares {
        out.push(format!("  on standing water with nothing under it for {:.3} cells, the link through {} ({water:?})", len, at(*mid)));
    }
    rounds.sort_by(|x, y| y.0.total_cmp(&x.0));
    for (further, mid, is_main) in &rounds {
        out.push(format!("  round the water on the link through {}, {:.3} cells further{}", at(*mid), further, if *is_main { ", a main road" } else { "" }));
    }
    out
}

pub fn year_nets(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, lk: &Look) -> Nets {
    world::timed("year nets", || year_nets_inner(r, g, cen, k, lk))
}

fn year_nets_inner(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, lk: &Look) -> Nets {
    let mut roads = Builder::default();
    // the railways are graded curves with their own works (rails.rs)
    let rails = crate::rails::track(&crate::rails::year(r, g, cen, k), g, lk, &sidings(r, cen, k));
    let (lane, main) = (*lk.ground.lane_dust, *lk.ground.road_gravel);
    // highways (the core's docs/rules/highways.md): the record marks a trunk road's
    // square 64 and a bypass's 128; a link between two such squares is drawn in
    // tarmac, a bypass's paler, until their look is drawn (a later piece)
    let tr = &r.trans[k * r.n..(k + 1) * r.n];
    for (c, m, lv) in drawn_links(r, &g.worn, k) {
        let trunk = tr[c] & 64 != 0 && tr[m] & 64 != 0;
        let colour = match (trunk, tr[c] & 128 != 0 || tr[m] & 128 != 0) {
            (true, true) => *lk.ground.tarmac_worn,
            (true, false) => *lk.ground.tarmac,
            _ if lv == 3 => main,
            _ => lane,
        };
        // (its course, fixed the year it was first drawn: a curve through the meeting places,
        // or round the meanders its straight line cuts; on the new land round a lake or a
        // tideway, or over it on a causeway)
        roads.ribbon(&way_line(g, &course(r, g, cen, c, m, k)), road_width(lv), colour);
    }
    Nets { roads, rails }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The roads the carts wore, measured on recorded runs: for each run
    /// named in `ISOSTATE_WORN_RUNS` (paths to `run.bin`, apart by `;`) and
    /// each rule in `ISOSTATE_WORN_RULES` (`weight:years`, apart by `,`; the
    /// game's if none), a line a year of 1800, 1850, 1900, 1950, 2000 and
    /// 2050 with the measure (`RoadMeasure`). Run by hand, with
    /// `cargo test -p isostate-viewer measure_a_recorded_run -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn measure_a_recorded_run() {
        let Ok(paths) = std::env::var("ISOSTATE_WORN_RUNS") else { return };
        let rules: Vec<WornRule> = std::env::var("ISOSTATE_WORN_RULES").map_or(vec![Worn::default().rule], |v| {
            v.split(',').map(|r| {
                let p: Vec<&str> = r.split(':').collect();
                WornRule { weight: p[0].parse().unwrap(), years: p[1].parse().unwrap(), window: p.get(2).map_or(0, |x| x.parse().unwrap()), fade: p.get(3).map_or(0, |x| x.parse().unwrap()) }
            }).collect()
        });
        for path in paths.split(';') {
            let bytes = std::fs::read(path).unwrap();
            let r = crate::replay::parse(path, &bytes).unwrap();
            for &rule in &rules {
                let worn = Worn::by(rule);
                let WornRule { weight, years, window, fade } = rule;
                for year in [1800, 1850, 1900, 1950, 2000, 2050] {
                    let k = (year - r.y0) as usize;
                    if k >= r.ny {
                        continue;
                    }
                    let m = measure(&r, &worn, k);
                    if std::env::var_os("ISOSTATE_WORN_HEAVY").is_some() {
                        // the heaviest carting on made roads not drawn: where, how much, and its years of wear
                        let drawn: std::collections::HashSet<(usize, usize)> = drawn_links(&r, &worn, k).into_iter().map(|(c, m, _)| (c, m)).collect();
                        let known = worn.known.lock().unwrap();
                        let mut heavy: Vec<(f32, usize, usize, u16, u8, u8)> = Vec::new();
                        for c in 0..r.n {
                            for s in 0..4 {
                                let Some(d) = isostate_core::traffic::other_end(r.w, r.h, c, s) else { continue };
                                let (tc, td) = (r.trans[k * r.n + c], r.trans[k * r.n + d]);
                                if r.traffic.mode(r.n, k, c, s) == isostate_core::traffic::CART && (tc & 3).min(td & 3) >= 2 && !drawn.contains(&(c, d)) {
                                    heavy.push((r.traffic.weight(r.n, k, c, s), c, d, known.count[c * 4 + s].max(known.bits[c * 4 + s].count_ones() as u16), tc, td));
                                }
                            }
                        }
                        heavy.sort_by(|a, b| b.0.total_cmp(&a.0));
                        for (wt, c, d, yrs, tc, td) in heavy.iter().take(12) {
                            println!("  {year}: {wt:.0} carted between {},{} and {},{}, {yrs} years of wear, trans {tc} and {td}", c % r.w, c / r.w, d % r.w, d / r.w);
                        }
                    }
                    let all = m.carted_drawn + m.carted_road_not_drawn + m.carted_track;
                    println!(
                        "{path} rule {weight} in {years} of {window} years, fading under {fade}, {year}: today {} links closing {}; drawn {} ({:.2} of today's) closing {} ({} with a cell of town ground), {} diagonals today's rule leaves out; of the year's carting (weight by length) {:.3} on drawn links, {:.3} on made roads not drawn, {:.3} on tracks",
                        m.today, m.today_closed, m.drawn, m.drawn as f64 / m.today.max(1) as f64, m.closed, m.closed_town, m.diagonals,
                        m.carted_drawn / all.max(1e-9), m.carted_road_not_drawn / all.max(1e-9), m.carted_track / all.max(1e-9)
                    );
                }
            }
        }
    }

    #[test]
    fn the_top_of_a_road_steps_where_a_ramp_ends_above_it_and_bends_where_two_cross() {
        // a road falling from 0.1 to 0 along its length; a ramp from 0.2 to 0.4 that begins 0.01
        // above it and rises to 0.12; a deck from 0.4 to 0.6 at 0.05, under the road at first
        let road = vec![(0.0, 0.1), (1.0, 0.0)];
        let ramp = vec![(0.2, 0.09), (0.4, 0.12)];
        let deck = vec![(0.4, 0.05), (0.6, 0.05)];
        let top = envelope(&[road, ramp, deck]);
        let at = |t: f32| {
            let i = top.partition_point(|p| p.0 < t).clamp(1, top.len() - 1);
            let ((t0, h0), (t1, h1)) = (top[i - 1], top[i]);
            h0 + (h1 - h0) * ((t - t0) / (t1 - t0)).clamp(0.0, 1.0)
        };
        // the road, and up onto the ramp at once where it begins, not sloping up to it
        assert!((at(0.1) - 0.09).abs() < 1e-5 && (at(0.199) - 0.0801).abs() < 1e-4, "{top:?}");
        assert!((at(0.2) - 0.09).abs() < 1e-5 && (at(0.3) - 0.105).abs() < 1e-5, "{top:?}");
        // down to the road at once where the ramp ends, the deck being under the road there
        assert!((at(0.401) - 0.0599).abs() < 1e-4 && (at(0.45) - 0.055).abs() < 1e-5, "{top:?}");
        // the road goes under the deck at 0.5, and the top bends there
        assert!(top.iter().any(|p| (p.0 - 0.5).abs() < 1e-5), "{top:?}");
        assert!((at(0.55) - 0.05).abs() < 1e-5 && (at(0.7) - 0.03).abs() < 1e-5, "{top:?}");
    }
}
