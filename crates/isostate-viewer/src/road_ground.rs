//! A road's way over the ground: between its two ends, fixed where
//! `road_course` fixed them, a road on the new land finds the cheapest way
//! for its kind, along a valley's side and round a hill rather than over it.
//!
//! The piece before this drew each link on one smooth curve from its one
//! meeting place to the next, leaving each along a fixed direction; the curve
//! paid no heed to the ground, so a road ran straight over a hill a carter
//! would have gone round, and kept to a valley only by chance. Now the ground
//! of the link's two cells and the land round them is read every 125 m (the
//! railways' `newland::passing::Lie`, which reads it for the lines), and the
//! way from one end to the other is the one that costs least, stepping from
//! square to square as the railways' search does. What a way costs is its
//! length, and then, by the road's kind (`Kind`):
//!
//! - **every metre it climbs or falls**, worth so many metres of level road
//!   (a horse lifts its load up every rise, and holds it back down every
//!   fall), and far more for each metre beyond the steepest its carts can
//!   take, its ruling gradient;
//! - **wet ground**: standing water (so much that it goes round a lake),
//!   ground too low to build on, the water's edge, marsh and a valley's
//!   floor, as the railways price them;
//! - **a town's ground** it has no business in: a square of a town that is
//!   neither of its own two cells;
//! - **straying from the curve** of the piece before this, for a turnpike and
//!   more for a trunk road: a lane may wind, a turnpike straightens, and a
//!   trunk road straightens more, cutting and banking through the ground (its
//!   climbing is priced lower, as its earthworks take some of it).
//!
//! A way never steps onto a river: the road crosses a river only where the
//! simulation has its bridge or ford, as the piece before this placed it,
//! straight across and square to the water (`network::bent_route`, which
//! hands each stretch between two such crossings to `seek`). The way over
//! the squares is then pulled taut: from each of its points it runs straight
//! to the furthest point the straight line reaches for no more than the
//! squares' way costs, so that it runs in any direction and not in the
//! sixteen of the search's steps, and turns only where the ground turns it.
//! It is drawn as a chain of the railways' curves (`course::Ends`) through
//! points along those straight lengths, leaving each end along the direction
//! fixed there. The way is kept only if it crosses every river as often as
//! the link's straight line, stands in no lake or tide, and costs less by
//! the road's own prices than the curve of the piece before (`cost`); else
//! the link keeps that curve.
//!
//! A course is found once, for the kind of road the link was the year it was
//! first drawn, and kept: a road bettered later keeps its line. The old
//! island has heights by the cell only (its relief under a cell is the
//! viewer's own), so there the roads keep the curves of the piece before.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use bevy::math::Vec2;

use crate::newland::passing::{Lie, FINE};
use crate::newland::RELIEF;
use crate::world::Ground;

/// What a road of one kind pays for the ground it crosses, besides its
/// length (one for each cell of length, 2 km).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kind {
    /// how many metres of level road a metre climbed or fallen is worth
    pub rise_worth: f32,
    /// the steepest its carts take, as 1 in this
    pub ruling_1_in: f32,
    /// how many metres of level road a metre climbed or fallen beyond that is worth
    pub steep_worth: f32,
    /// for each cell of length, what each cell it lies off the curve of the piece before costs
    pub off_line: f32,
    /// how much of the railways' price for wet ground and the water's edge it pays
    pub wet: f32,
}

/// A lane, the simulation's made road (level 2): it winds as the ground asks.
/// A rise is worth twenty times its height in length (a cart's horse on a
/// rise of 1 in 20 does about twice the work of the level, so going round by
/// as much again is no dearer), and a lane takes 1 in 12 before it must zigzag.
pub const LANE: Kind = Kind { rise_worth: 20.0, ruling_1_in: 12.0, steep_worth: 60.0, off_line: 0.5, wet: 1.0 };
/// A turnpike (level 3): engineered to 1 in 20 (the turnpike trusts' and
/// Telford's aim was 1 in 30 to 1 in 20), and it straightens.
pub const TURNPIKE: Kind = Kind { rise_worth: 20.0, ruling_1_in: 20.0, steep_worth: 60.0, off_line: 1.0, wet: 1.0 };
/// A trunk road (both squares marked so in the record): 1 in 25, cut and
/// banked through the ground, so its rises are priced at half, and it
/// straightens most.
pub const TRUNK: Kind = Kind { rise_worth: 10.0, ruling_1_in: 25.0, steep_worth: 30.0, off_line: 3.0, wet: 0.5 };

/// The kind of road the link between `c` and `m` is in year `k`: 0 a lane, 1 a turnpike, 2 a trunk road.
pub fn kind_in(r: &crate::replay::Replay, k: usize, c: usize, m: usize) -> u8 {
    let tr = &r.trans[k * r.n..(k + 1) * r.n];
    if tr[c] & 64 != 0 && tr[m] & 64 != 0 {
        2
    } else if (tr[c] & 3).min(tr[m] & 3) >= 3 {
        1
    } else {
        0
    }
}

/// The prices of a kind (`kind_in`).
pub fn prices(kind: u8) -> Kind {
    match kind {
        2 => TRUNK,
        1 => TURNPIKE,
        _ => LANE,
    }
}

/// What a square of a town's ground costs a road with no business there, for each cell of length.
const TOWN: f32 = 6.0;
/// How far a road leaves each of its ends along the direction fixed there
/// before it seeks its way, in cells (200 m), so that two roads meeting at a
/// place still run on into each other there.
const LEAD: f32 = 0.1;
/// How far round the straight line between its ends a way may stray, in cells (800 m).
const PAD: f32 = 0.4;
/// How far apart the points of the way that the curve is drawn through are, in cells (300 m).
const SPACING: f32 = 0.15;
/// How much dearer than the way over the squares a straight line may be and
/// still be taken: a little, so that the way is not held to the lattice of
/// the squares' middles where the ground hardly cares.
const SLACK: f32 = 0.03;

/// The sixteen steps a way may take from a square, as the railways' search takes them.
const STEPS: [(isize, isize); 16] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1), (2, 1), (2, -1), (-2, 1), (-2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2)];

#[derive(PartialEq)]
struct Reach(f32, u32);

impl Eq for Reach {}

impl Ord for Reach {
    fn cmp(&self, other: &Reach) -> Ordering {
        other.0.total_cmp(&self.0).then(other.1.cmp(&self.1))
    }
}

impl PartialOrd for Reach {
    fn partial_cmp(&self, other: &Reach) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// What a road asks of its search: its prices, the cells it joins, and which
/// cells were a town's ground the year it was first drawn.
pub struct Asked<'a> {
    pub kind: Kind,
    pub cells: [usize; 2],
    pub town: &'a dyn Fn(usize) -> bool,
}

/// The cheapest way for the road from `a` (leaving along `da`) to `b`
/// (arriving along `db`), drawn as a smooth line: its points after `a` up to
/// and including `b`. None where the ground gives it nothing to seek (the
/// two ends too near, or no way that keeps off the rivers).
pub fn seek(lie: &mut Lie, g: &Ground, asked: &Asked, a: Vec2, da: Vec2, b: Vec2, db: Vec2) -> Option<Vec<Vec2>> {
    let chord = a.distance(b);
    if chord < 4.0 * LEAD {
        return None;
    }
    let (a1, b1) = (a + da * LEAD, b - db * LEAD);
    for c in asked.cells {
        lie.know(g, c);
    }
    let f = FINE as f32;
    let (fw, fh) = ((g.w * FINE) as isize, (g.h * FINE) as isize);
    let lo = a.min(b) - Vec2::splat(PAD);
    let hi = a.max(b) + Vec2::splat(PAD);
    let (x0, y0) = (((lo.x * f).floor() as isize).max(0), ((lo.y * f).floor() as isize).max(0));
    let (x1, y1) = (((hi.x * f).ceil() as isize).min(fw - 1), ((hi.y * f).ceil() as isize).min(fh - 1));
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let (nx, ny) = ((x1 - x0 + 1) as usize, (y1 - y0 + 1) as usize);
    let spot = |i: usize| Vec2::new((x0 + (i % nx) as isize) as f32 + 0.5, (y0 + (i / nx) as isize) as f32 + 0.5) / f;
    let square = |p: Vec2| -> Option<usize> {
        let (sx, sy) = ((p.x * f).floor() as isize, (p.y * f).floor() as isize);
        (sx >= x0 && sx <= x1 && sy >= y0 && sy <= y1).then(|| (sy - y0) as usize * nx + (sx - x0) as usize)
    };
    let (start, goal) = (square(a1)?, square(b1)?);
    // what each square of the box is: the ground a way lies on, what lying there costs for each
    // cell of length, and whether a river runs there
    let base = old_curve(a, da, b, db);
    let n = nx * ny;
    let mut ground = vec![0.0f32; n];
    let mut dear = vec![0.0f32; n];
    let mut river = vec![false; n];
    for i in 0..n {
        let (sx, sy) = ((x0 + (i % nx) as isize) as usize, (y0 + (i / nx) as isize) as usize);
        let (h, e) = lie.at(g, sx, sy);
        ground[i] = h;
        let p = spot(i);
        let cell = sy / FINE * g.w + sx / FINE;
        let mut d = e * asked.kind.wet;
        if !asked.cells.contains(&cell) && (asked.town)(cell) {
            d += TOWN;
        }
        if asked.kind.off_line > 0.0 {
            d += asked.kind.off_line * crate::newland::passing::off_way(&base, p);
        }
        dear[i] = d;
        river[i] = lie.river_at(sx, sy).is_some();
    }
    let (rise, steep, ruling) = (asked.kind.rise_worth / RELIEF, asked.kind.steep_worth / RELIEF, RELIEF / asked.kind.ruling_1_in);
    let mut best = vec![f32::INFINITY; n];
    let mut came = vec![u32::MAX; n];
    let mut heap = BinaryHeap::new();
    best[start] = 0.0;
    heap.push(Reach(0.0, start as u32));
    while let Some(Reach(cost, u)) = heap.pop() {
        let u = u as usize;
        if cost > best[u] {
            continue;
        }
        if u == goal {
            break;
        }
        let (ux, uy) = ((u % nx) as isize, (u / nx) as isize);
        for (dx, dy) in STEPS {
            let (vx, vy) = (ux + dx, uy + dy);
            if vx < 0 || vy < 0 || vx >= nx as isize || vy >= ny as isize {
                continue;
            }
            let v = vy as usize * nx + vx as usize;
            // never onto a river, nor over one between two squares (a knight's move's middle)
            let mid = (vy - dy / 2) as usize * nx + (vx - dx / 2) as usize;
            if (river[v] && v != goal) || (river[mid] && mid != start && mid != goal) {
                continue;
            }
            let len = ((dx * dx + dy * dy) as f32).sqrt() / f;
            let dh = (ground[v] - ground[u]).abs();
            let step = len * (1.0 + 0.5 * (dear[u] + dear[v])) + rise * dh + steep * (dh - ruling * len).max(0.0);
            if cost + step < best[v] {
                best[v] = cost + step;
                came[v] = u as u32;
                heap.push(Reach(cost + step, v as u32));
            }
        }
    }
    if !best[goal].is_finite() {
        return None;
    }
    let mut path = vec![goal];
    while *path.last().unwrap() != start {
        path.push(came[*path.last().unwrap()] as usize);
    }
    path.reverse();
    let mut pts: Vec<Vec2> = path.iter().map(|&i| spot(i)).collect();
    pts[0] = a1;
    let last = pts.len() - 1;
    pts[last] = b1;
    // what the straight line between two points costs, read every half square (and its climb
    // where it passes from one square to the next); none if it meets a river or leaves the box
    let line = |p: Vec2, q: Vec2| -> Option<f32> {
        let k = ((p.distance(q) * f * 2.0).ceil() as usize).max(1);
        let len = p.distance(q) / k as f32;
        let (mut prev, mut run, mut total) = (square(p)?, 0.0f32, 0.0f32);
        for s in 1..=k {
            let v = square(p.lerp(q, s as f32 / k as f32))?;
            if river[v] && v != start && v != goal {
                return None;
            }
            run += len;
            total += len * (1.0 + 0.5 * (dear[prev] + dear[v]));
            if v != prev {
                let dh = (ground[v] - ground[prev]).abs();
                total += rise * dh + steep * (dh - ruling * run).max(0.0);
                (prev, run) = (v, 0.0);
            }
        }
        Some(total)
    };
    // the way pulled taut: from each point on, straight to the furthest point of it that the
    // straight line reaches for no more than the way over the squares costs (and `SLACK`). A search over
    // squares runs in the sixteen directions of its steps; pulled taut, a way runs in any
    // direction, and turns only where the ground turns it
    let mut taut = vec![pts[0]];
    let mut i = 0;
    while i < last {
        let mut j = last;
        while j > i + 1 && !line(pts[i], pts[j]).is_some_and(|c| c <= (best[path[j]] - best[path[i]]) * (1.0 + SLACK) + 1e-4) {
            j -= 1;
        }
        taut.push(pts[j]);
        i = j;
    }
    Some(draw(a, da, b, db, &along(&taut)))
}

/// A way pulled taut, with points put in along each straight length of it
/// longer than twice `SPACING`, every `SPACING` or closer, so that the curve
/// drawn through them keeps to the long straight lengths, and flows through
/// the corners of the short ones.
fn along(taut: &[Vec2]) -> Vec<Vec2> {
    let mut out = vec![taut[0]];
    for w in taut.windows(2) {
        let len = w[0].distance(w[1]);
        let k = if len > 2.0 * SPACING { (len / SPACING).ceil() as usize } else { 1 };
        out.extend((1..=k).map(|i| w[0].lerp(w[1], i as f32 / k as f32)));
    }
    out
}

/// The curve of the piece before this between two ends, every 0.04 of a cell or closer.
fn old_curve(a: Vec2, da: Vec2, b: Vec2, db: Vec2) -> Vec<Vec2> {
    let ends = crate::course::Ends { a, da, b, db };
    let n = ((a.distance(b) / 0.04).ceil() as usize).max(4);
    (0..=n).map(|i| ends.point(i as f32 / n as f32, 0.0)).collect()
}

/// The road from `a` to `b` through the points `through` (whose first lies
/// `LEAD` on from `a` along `da`, and whose last `LEAD` short of `b` along
/// `db`): straight out along `da`, then a chain of the railways' curves
/// through each point, running there from the one before it toward the one
/// after (Catmull-Rom's way), and straight in along `db`. Its points after
/// `a`, every 0.04 of a cell or closer.
fn draw(a: Vec2, da: Vec2, b: Vec2, db: Vec2, through: &[Vec2]) -> Vec<Vec2> {
    let n = through.len();
    let mut stops: Vec<(Vec2, Vec2)> = vec![(a, da)];
    for i in 0..n {
        let dir = if i == 0 {
            da
        } else if i == n - 1 {
            db
        } else {
            (through[i + 1] - through[i - 1]).normalize_or(da)
        };
        stops.push((through[i], dir));
    }
    stops.push((b, db));
    let mut out = Vec::new();
    for w in stops.windows(2) {
        let ends = crate::course::Ends { a: w[0].0, da: w[0].1, b: w[1].0, db: w[1].1 };
        let k = ((w[0].0.distance(w[1].0) / 0.04).ceil() as usize).max(2);
        out.extend((1..=k).map(|i| ends.point(i as f32 / k as f32, 0.0)));
    }
    out
}

/// What a line of points costs a road of this kind by the prices `seek`
/// uses, read every 1/32 of a cell (but for its leaving the curve of the
/// piece before, which only the search prices): to set a way found against
/// the curve it would replace.
pub fn cost(lie: &Lie, g: &Ground, asked: &Asked, pts: &[Vec2]) -> f32 {
    let f = FINE as f32;
    let (fw, fh) = (g.w * FINE, g.h * FINE);
    let (rise, steep, ruling) = (asked.kind.rise_worth / RELIEF, asked.kind.steep_worth / RELIEF, RELIEF / asked.kind.ruling_1_in);
    let read = |p: Vec2| -> (f32, f32) {
        let (sx, sy) = (((p.x * f).max(0.0) as usize).min(fw - 1), ((p.y * f).max(0.0) as usize).min(fh - 1));
        let (_, e) = lie.at(g, sx, sy);
        let cell = sy / FINE * g.w + sx / FINE;
        let town = if !asked.cells.contains(&cell) && (asked.town)(cell) { TOWN } else { 0.0 };
        (crate::rails::lie(g, p.x, p.y), e * asked.kind.wet + town)
    };
    let mut total = 0.0;
    for w in pts.windows(2) {
        let k = ((w[0].distance(w[1]) * 32.0).ceil() as usize).max(1);
        let mut prev = read(w[0]);
        for i in 1..=k {
            let p = w[0].lerp(w[1], i as f32 / k as f32);
            let here = read(p);
            let len = w[0].distance(w[1]) / k as f32;
            let dh = (here.0 - prev.0).abs();
            total += len * (1.0 + 0.5 * (prev.1 + here.1)) + rise * dh + steep * (dh - ruling * len).max(0.0);
            prev = here;
        }
    }
    total
}

/// The slope of the ground along a line of points, as the land itself has
/// it (in metres for each metre along): the mean over its length and the
/// share of its length steeper than 1 in 12, read every 0.02 of a cell
/// (40 m) and over a lake at its water. Returned as the sums (slope times
/// length, length steeper, length).
pub fn slope_along(g: &Ground, pts: &[Vec2]) -> (f32, f32, f32) {
    let relief = if g.laid.is_some() { RELIEF } else { 3.0 };
    let (mut s, mut steep, mut len) = (0.0, 0.0, 0.0);
    for w in pts.windows(2) {
        let k = ((w[0].distance(w[1]) / 0.02).ceil() as usize).max(1);
        let l = w[0].distance(w[1]) / k as f32;
        if l < 1e-9 {
            continue;
        }
        let mut prev = crate::rails::lie(g, w[0].x, w[0].y);
        for i in 1..=k {
            let p = w[0].lerp(w[1], i as f32 / k as f32);
            let h = crate::rails::lie(g, p.x, p.y);
            let grade = (h - prev).abs() / relief / l;
            s += grade * l;
            if grade > 1.0 / 12.0 {
                steep += l;
            }
            len += l;
            prev = h;
        }
    }
    (s, steep, len)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The curve through the points leaves `a` along `da` and arrives at `b` along `db`, and passes through each point.
    #[test]
    fn the_drawn_way_leaves_and_arrives_along_its_ends_and_passes_its_points() {
        let (a, b) = (Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0));
        let (da, db) = (Vec2::new(1.0, 1.0).normalize(), Vec2::new(1.0, -1.0).normalize());
        let through = [a + da * LEAD, Vec2::new(0.5, 0.3), b - db * LEAD];
        let pts = draw(a, da, b, db, &through);
        assert!((pts[0] - a).normalize().dot(da) > 0.999, "{:?}", pts[0]);
        let n = pts.len();
        assert!((pts[n - 1] - pts[n - 2]).normalize().dot(db) > 0.999);
        assert!(pts[n - 1].distance(b) < 1e-6);
        for t in through {
            assert!(pts.iter().any(|p| p.distance(t) < 1e-5), "{t:?}");
        }
    }

    /// Points are put in along each long straight length, its corners kept.
    #[test]
    fn a_taut_way_has_points_along_its_lengths_and_keeps_its_corners() {
        let taut = [Vec2::new(0.0, 0.0), Vec2::new(0.5, 0.0), Vec2::new(0.5, 0.25)];
        let p = along(&taut);
        for t in taut {
            assert!(p.contains(&t), "{t:?}");
        }
        // the long length has points every SPACING or closer, the short one none
        assert_eq!(p.len(), 6, "{p:?}");
        for w in p.windows(2) {
            assert!(w[0].distance(w[1]) <= 2.0 * SPACING + 1e-6, "{w:?}");
        }
    }
}
