//! The course of each road as it is drawn: one smooth line from one meeting
//! place to the next, bending gently through the places between, and never
//! moving once drawn.
//!
//! The simulation says only which neighbouring cells a road joins
//! (`network::drawn_links`, the roads the carts wore). Before this, each
//! link was drawn straight from one cell's meeting place to the next and
//! turned sharply at each: a road read as ruler lines in eight directions
//! that kinked every two kilometres. Now:
//!
//! - **A meeting place has a line through it**, fixed in the year it first
//!   has a road: the line from the one neighbour to the other of the
//!   straightest pair of its roads that year (as a Catmull-Rom spline takes
//!   it), or the one road's own line where it has only one. A road through
//!   the place leaves it along that line, one each way, so two roads that
//!   meet there run on into each other without a kink, and the place is a
//!   point on one smooth curve.
//! - **A link's two ends are fixed in the year it is first drawn.** Each end
//!   leaves its place along the place's line if that way is free and lies
//!   within `MOST_OFF` of the link's own straight line, and along its own
//!   straight line if not: a later road joins what is there, meeting it at
//!   its own angle as a side road does at a T. Between its ends the link
//!   runs on the railways' curve (`course::Ends`). A road that fades and is
//!   worn again comes back on the line it was first drawn on.
//! - **A link drawn on town ground stays straight.** Where either of its
//!   squares is town ground in the year it is first drawn, it is drawn as
//!   it always was (`network::route`), since the town's streets and houses
//!   are laid along the straight line (`scatter::towns`).
//! - **Rivers.** The curve crosses a river only where the straight line
//!   does, through the same point and square to the water
//!   (`network::bent_route`); where it cannot, the link keeps the way it was
//!   drawn before.
//!
//! The years are gone through in order, as the roads the carts wore are
//! (`network::Worn`), and what is fixed is kept in `Ground::courses`.
//! `ISOSTATE_CURVES=off` draws every road straight, as before, to set the
//! two side by side.

use std::collections::HashMap;

use bevy::math::Vec2;

use crate::network::{self, Way};
use crate::replay::Replay;
use crate::world::Ground;

/// How far from a link's straight line its end may leave a meeting place to
/// run on along the place's line (60 degrees, as a cosine): a road turning
/// 90 degrees at a place leaves each of its two links 45 degrees off, and
/// sweeps round; one doubling back sharper than 120 degrees keeps its corner.
const MOST_OFF: f32 = 0.0;
/// An end within this of its straight line (about 1.8 degrees, as a sine)
/// counts as straight: a link both of whose ends are is drawn as before.
const STRAIGHT: f32 = 0.03;

/// How a link leaves its two meeting places, fixed in the year it was first
/// drawn: the direction it leaves its lower-numbered cell's place in and the
/// direction it arrives at the other's (each of length one), or none where
/// it is drawn as before (`network::route`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fixed {
    pub year: usize,
    pub bend: Option<(Vec2, Vec2)>,
    /// whether it is held straight, having a square of town ground the year it was first drawn
    pub held: bool,
    /// at its lower cell's place and at the other's, which way along the
    /// place's line it leaves (0 along it, 1 against it), if it does
    pub along: [Option<u8>; 2],
}

#[derive(Default)]
struct Known {
    /// how many years have been gone through
    through: usize,
    /// for each cell, its meeting place's line, fixed the year it first had a road
    line: Vec<Option<Vec2>>,
    /// for each link (its two cells, the lower first), its ends
    ends: HashMap<(u32, u32), Fixed>,
}

/// The courses fixed so far (`Ground::courses`), and the ways found for them.
#[derive(Default)]
pub struct Courses {
    known: std::sync::Mutex<Known>,
    ways: std::sync::Mutex<HashMap<(u32, u32), Way>>,
}

/// Whether the roads are drawn on curves: unless `ISOSTATE_CURVES=off`
/// asks for the picture before, to set the two side by side.
pub fn on() -> bool {
    static OFF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    !STRAIGHT_IN_TEST.with(|a| a.get()) && !*OFF.get_or_init(|| std::env::var("ISOSTATE_CURVES").is_ok_and(|v| v == "off"))
}

thread_local! {
    /// In a test: draw every road straight, as before the curves.
    pub static STRAIGHT_IN_TEST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

impl Courses {
    /// An edit taken back (back.rs): what was fixed from year `from` on is
    /// forgotten; as with the wear, everything is gone through again.
    pub fn forget_from(&self, from: usize) {
        if let Ok(mut known) = self.known.lock() {
            if from < known.through {
                *known = Known::default();
                if let Ok(mut ways) = self.ways.lock() {
                    ways.clear();
                }
            }
        }
    }

    /// How the link between `c` and `m` is fixed, going through the record
    /// up to year `k` first; none where it has not been drawn by then.
    pub fn fixed(&self, r: &Replay, g: &Ground, cen: &[(f32, f32)], c: usize, m: usize, k: usize) -> Option<Fixed> {
        let mut known = self.known.lock().unwrap();
        go_through(r, g, cen, &mut known, k);
        known.ends.get(&key(c, m)).copied()
    }

    /// The way the link between `c` and `m` is drawn in year `k`.
    pub fn way(&self, r: &Replay, g: &Ground, cen: &[(f32, f32)], c: usize, m: usize, k: usize) -> Way {
        let (lo, hi) = (c.min(m), c.max(m));
        let (a, b) = (Vec2::new(cen[lo].0, cen[lo].1), Vec2::new(cen[hi].0, cen[hi].1));
        let bend = if on() { self.fixed(r, g, cen, lo, hi, k).and_then(|f| f.bend) } else { None };
        let Some((da, db)) = bend else {
            let way = network::route(g, a, b, r.round());
            return if c <= m { way } else { reversed(way) };
        };
        let kept = self.ways.lock().ok().and_then(|w| w.get(&key(lo, hi)).cloned());
        let way = kept.unwrap_or_else(|| {
            let found = network::bent_route(g, a, b, da, db, r.round());
            if let Ok(mut w) = self.ways.lock() {
                w.insert(key(lo, hi), found.clone());
            }
            found
        });
        if c <= m { way } else { reversed(way) }
    }
}

fn key(c: usize, m: usize) -> (u32, u32) {
    (c.min(m) as u32, c.max(m) as u32)
}

fn reversed(mut way: Way) -> Way {
    way.pts.reverse();
    way
}

/// Go through the record's years up to `k` (no further than it holds), fixing each
/// meeting place's line and each link's ends in the year they are first drawn.
fn go_through(r: &Replay, g: &Ground, cen: &[(f32, f32)], known: &mut Known, k: usize) {
    if known.line.len() != r.n {
        *known = Known { line: vec![None; r.n], ..Default::default() };
    }
    let last = k.min(r.filled.min(r.ny).saturating_sub(1));
    let at = |c: usize| Vec2::new(cen[c].0, cen[c].1);
    while known.through <= last && known.through < r.filled.min(r.ny) {
        let y = known.through;
        let drawn: Vec<(usize, usize)> = network::drawn_links(r, &g.worn, y).into_iter().map(|(c, m, _)| (c.min(m), c.max(m))).collect();
        // the ways along the places' lines the roads drawn this year already leave by (a road
        // gone back to grass leaves its way free; worn again, it comes back on its old line)
        let mut taken: HashMap<(usize, u8), bool> = HashMap::new();
        for &(c, m) in &drawn {
            if let Some(f) = known.ends.get(&key(c, m)) {
                for (p, s) in [(c, f.along[0]), (m, f.along[1])] {
                    if let Some(s) = s {
                        taken.insert((p, s), true);
                    }
                }
            }
        }
        let new: Vec<(usize, usize)> = drawn.into_iter().filter(|&(c, m)| !known.ends.contains_key(&key(c, m))).collect();
        // the places that have their first road this year: their line, from the straightest pair
        let mut at_place: HashMap<usize, Vec<usize>> = HashMap::new();
        for &(c, m) in &new {
            at_place.entry(c).or_default().push(m);
            at_place.entry(m).or_default().push(c);
        }
        let mut places: Vec<usize> = at_place.keys().copied().filter(|&p| known.line[p].is_none()).collect();
        places.sort_unstable();
        for p in places {
            let others = &at_place[&p];
            let out = |q: usize| (at(q) - at(p)).normalize_or(Vec2::X);
            let mut line = out(others[0]);
            let mut best = f32::MAX;
            for i in 0..others.len() {
                for j in i + 1..others.len() {
                    // the pair that runs most nearly straight on through the place
                    let turn = out(others[i]).dot(out(others[j]));
                    if turn < best {
                        best = turn;
                        line = (at(others[j]) - at(others[i])).normalize_or(line);
                    }
                }
            }
            known.line[p] = Some(line);
        }
        // each new link's ends: along its place's line where that way is free and near enough
        // its own, the nearest first; else along its own straight line
        let mut claims: Vec<(f32, usize, usize, usize)> = Vec::new();
        for (i, &(c, m)) in new.iter().enumerate() {
            let held = town(r, y, c) || town(r, y, m);
            for (p, q) in [(c, m), (m, c)] {
                let (Some(line), false) = (known.line[p], held) else { continue };
                let own = (at(q) - at(p)).normalize_or(Vec2::X);
                let cos = line.dot(own).abs();
                if cos >= MOST_OFF {
                    claims.push((-cos, i, p, q));
                }
            }
        }
        claims.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)).then(x.2.cmp(&y.2)));
        let mut leave: HashMap<(usize, usize), (Vec2, u8)> = HashMap::new();
        for (_, _, p, q) in claims {
            let line = known.line[p].unwrap();
            let own = at(q) - at(p);
            let (slot, dir) = if line.dot(own) >= 0.0 { (0u8, line) } else { (1, -line) };
            if !taken.contains_key(&(p, slot)) {
                taken.insert((p, slot), true);
                leave.insert((p, q), (dir, slot));
            }
        }
        for &(c, m) in &new {
            let own = (at(m) - at(c)).normalize_or(Vec2::X);
            let da = leave.get(&(c, m)).map_or(own, |l| l.0);
            let db = leave.get(&(m, c)).map_or(own, |l| -l.0);
            let straight = da.perp_dot(own).abs() < STRAIGHT && db.perp_dot(own).abs() < STRAIGHT;
            let along = [leave.get(&(c, m)).map(|l| l.1), leave.get(&(m, c)).map(|l| l.1)];
            known.ends.insert(key(c, m), Fixed { year: y, bend: (!straight).then_some((da, db)), held: town(r, y, c) || town(r, y, m), along });
        }
        known.through += 1;
    }
}

/// Whether cell `c` is town ground in year `k`, whose streets are laid along the straight line.
fn town(r: &Replay, k: usize, c: usize) -> bool {
    network::town_ground(r, k, c)
}

/// How many degrees a drawn line may turn and still count as straight on.
pub const STRAIGHT_ON: f32 = 5.0;
/// The bins of a turn, in degrees: up to 1, 5, 15, 30, 60, 100, and more.
pub const TURN_BINS: [f32; 6] = [1.0, 5.0, 15.0, 30.0, 60.0, 100.0];

/// The roads of a year as they are drawn (`shape`): how straight and how
/// much in the map's eight directions they run, and how they turn where
/// they meet. Lengths are in cells (2 km); a link along a row is one.
#[derive(Clone, Debug, Default)]
pub struct Shape {
    pub year: i32,
    pub links: usize,
    /// the links drawn on a curve (the rest as before, straight or round water),
    /// and the links held straight for a square of town ground
    pub bent: usize,
    pub held: usize,
    pub length: f32,
    /// the study's measure, of the links alone: lines of links in one of the
    /// eight directions, running on through any meeting place, in links
    pub lattice_lines: usize,
    pub lattice_mean: f32,
    pub lattice_longest: usize,
    /// straight runs of the drawn line, running on through a meeting place
    /// where it turns less than `STRAIGHT_ON`, and ending where it has turned
    /// that far from where the run began: how many, their mean length, the
    /// mean length of the run a point of road lies on, and the longest
    pub runs: usize,
    pub run_mean: f32,
    pub run_mean_by_length: f32,
    pub run_longest: f32,
    /// at each meeting place a road passes through (two roads), how far the
    /// drawn line turns there, in the bins of `TURN_BINS`; and at a junction
    /// (three roads or more) the same for its straightest pair
    pub turns: [usize; 7],
    pub junction_turns: [usize; 7],
    /// the share of the drawn length that runs within 5 degrees of one of
    /// the eight directions
    pub on_eight: f32,
    /// the tightest curve of any link (its radius, over 0.08 of a cell), and
    /// how many links curve tighter than 0.25 of a cell (500 m)
    pub tightest: f32,
    pub tight_links: usize,
    /// the river crossings of the drawn ways, and the links whose drawn way
    /// crosses a river more often than the way drawn before (`network::route`)
    pub crossings: usize,
    pub extra_crossings: usize,
    /// the bridges and fords: the mean of how square each crosses (1 square, 0 along the water)
    pub square: f32,
    /// the links on a curve with a square of town ground of 1900 (or of the
    /// record's last year, if earlier), whose streets and houses are laid
    /// along the straight line
    pub bent_on_1900_town: usize,
    /// the meeting places a road passes through turning more than
    /// `STRAIGHT_ON`, by why: a link held straight on town ground, a link
    /// that keeps its old way at a river, the two links' straight lines
    /// turning more than 120 degrees there (a corner kept), and two links
    /// drawn in different years that could not join
    pub kinks: [usize; 4],
}

/// Measure the roads drawn in year `k` (`Shape`).
pub fn shape(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> Shape {
    use isostate_core::traffic as tf;
    let links = network::drawn_links(r, &g.worn, k);
    let mut out = Shape { year: r.y0 + k as i32, links: links.len(), tightest: f32::INFINITY, ..Default::default() };
    // the lattice lines, as the study counted them
    let set: std::collections::HashSet<(usize, usize)> = links.iter().filter_map(|&(c, m, _)| tf::link(r.w, c, m)).collect();
    let mut lines: Vec<usize> = Vec::new();
    for &(c, s) in &set {
        let (dx, dy) = tf::SLOTS[s];
        let (x, y) = ((c % r.w) as isize - dx, (c / r.w) as isize - dy);
        if x >= 0 && y >= 0 && (x as usize) < r.w && (y as usize) < r.h && set.contains(&(y as usize * r.w + x as usize, s)) {
            continue;
        }
        let mut n = 0;
        let mut at = Some(c);
        while let Some(cc) = at.filter(|&cc| set.contains(&(cc, s))) {
            n += 1;
            at = tf::other_end(r.w, r.h, cc, s);
        }
        lines.push(n);
    }
    out.lattice_lines = lines.len();
    out.lattice_mean = lines.iter().sum::<usize>() as f32 / lines.len().max(1) as f32;
    out.lattice_longest = lines.iter().copied().max().unwrap_or(0);
    // each link's way, from its lower cell, and how it leaves each end
    let ways: Vec<Way> = links.iter().map(|&(c, m, _)| network::course(r, g, cen, c, m, k)).collect();
    let mut ends: HashMap<usize, Vec<(usize, bool, Vec2)>> = HashMap::new();
    let (mut square, mut crossed) = (0.0f32, 0usize);
    for (i, (&(c, m, _), way)) in links.iter().zip(&ways).enumerate() {
        let p = &way.pts;
        let n = p.len();
        ends.entry(c).or_default().push((i, false, (p[1] - p[0]).normalize_or(Vec2::X)));
        ends.entry(m).or_default().push((i, true, (p[n - 2] - p[n - 1]).normalize_or(Vec2::X)));
        let fixed = g.courses.fixed(r, g, cen, c, m, k);
        let bent = on() && fixed.is_some_and(|f| f.bend.is_some()) && n > 2;
        out.bent += bent as usize;
        out.held += fixed.is_some_and(|f| f.held) as usize;
        let k1900 = ((1900 - r.y0).max(0) as usize).min(r.filled.min(r.ny).saturating_sub(1));
        out.bent_on_1900_town += (bent && (town(r, k1900, c) || town(r, k1900, m))) as usize;
        let len = crate::course::length(p);
        out.length += len;
        for w in p.windows(2) {
            let d = w[1] - w[0];
            let deg = d.y.atan2(d.x).to_degrees().rem_euclid(45.0);
            if deg.min(45.0 - deg) < 5.0 {
                out.on_eight += d.length();
            }
        }
        let tight = crate::course::tightest(p, 0.08);
        out.tightest = out.tightest.min(tight);
        out.tight_links += (tight < 0.25) as usize;
        let cuts = |pts: &[Vec2]| pts.windows(2).map(|w| crate::rivers::crossing_points(&g.rivers, &g.water, w[0], w[1]).len()).sum::<usize>();
        let drawn = cuts(p);
        out.crossings += drawn;
        let (a, b) = (Vec2::new(cen[c].0, cen[c].1), Vec2::new(cen[m].0, cen[m].1));
        out.extra_crossings += (drawn > cuts(&network::route(g, a, b, r.round()).pts)) as usize;
        for &(_, river, _, _, along) in &way.crossings {
            square += along.perp_dot(river).abs();
            crossed += 1;
        }
    }
    out.on_eight /= out.length.max(1e-9);
    out.square = square / crossed.max(1) as f32;
    let bin = |deg: f32| TURN_BINS.iter().position(|&b| deg <= b).unwrap_or(TURN_BINS.len());
    // the turn between two links at a place: how far the one's way in turns from the other's way out
    let turn = |u: Vec2, v: Vec2| (-u).angle_to(v).abs().to_degrees();
    // a line runs on through a place between the pair that turns least, where they turn less than STRAIGHT_ON
    let mut next: HashMap<(usize, bool), (usize, bool)> = HashMap::new();
    let mut places: Vec<usize> = ends.keys().copied().collect();
    places.sort_unstable();
    for p in places {
        let e = &ends[&p];
        let mut pairs: Vec<(f32, usize, usize)> = Vec::new();
        for i in 0..e.len() {
            for j in i + 1..e.len() {
                pairs.push((turn(e[i].2, e[j].2), i, j));
            }
        }
        pairs.sort_by(|x, y| x.0.total_cmp(&y.0));
        if let Some(&(t, _, _)) = pairs.first() {
            if e.len() == 2 { out.turns[bin(t)] += 1 } else { out.junction_turns[bin(t)] += 1 }
            if e.len() == 2 && t > STRAIGHT_ON && on() {
                // why it turns there
                let fixed: Vec<Option<Fixed>> = e.iter().map(|x| g.courses.fixed(r, g, cen, links[x.0].0, links[x.0].1, k)).collect();
                let kept_old = e.iter().zip(&fixed).any(|(x, f)| {
                    let Some((da, db)) = f.and_then(|f| f.bend) else { return false };
                    let want = if x.1 { -db } else { da };
                    x.2.dot(want) < 0.996
                });
                let chords = |x: &(usize, bool, Vec2)| {
                    let (c, m, _) = links[x.0];
                    let d = Vec2::new(cen[m].0 - cen[c].0, cen[m].1 - cen[c].1).normalize_or(Vec2::X);
                    if x.1 { -d } else { d }
                };
                let why = if fixed.iter().any(|f| f.is_some_and(|f| f.held)) {
                    0
                } else if kept_old {
                    1
                } else if chords(&e[0]).dot(chords(&e[1])) > 0.5 {
                    2
                } else {
                    3
                };
                out.kinks[why] += 1;
            }
        }
        let mut used = vec![false; e.len()];
        for (t, i, j) in pairs {
            if t < STRAIGHT_ON && !used[i] && !used[j] {
                used[i] = true;
                used[j] = true;
                next.insert((e[i].0, e[i].1), (e[j].0, e[j].1));
                next.insert((e[j].0, e[j].1), (e[i].0, e[i].1));
            }
        }
    }
    // the drawn lines, link after link through the places they run straight on through, and the
    // straight runs along them
    let mut seen = vec![false; links.len()];
    let mut runs: Vec<f32> = Vec::new();
    let pts_of = |i: usize, from_end: bool| -> Vec<Vec2> {
        let mut p = ways[i].pts.clone();
        if from_end {
            p.reverse();
        }
        p
    };
    for start in 0..links.len() {
        if seen[start] {
            continue;
        }
        // back to the line's beginning (or round a loop to where it began)
        let (mut i, mut at_end) = (start, false);
        while let Some(&(j, j_end)) = next.get(&(i, at_end)) {
            if j == start {
                break;
            }
            i = j;
            at_end = !j_end;
        }
        // then forward: the link i, entered at its end `at_end` and left by the other
        let mut line: Vec<Vec2> = Vec::new();
        let (mut i, mut from_end) = (i, at_end);
        loop {
            if seen[i] {
                break;
            }
            seen[i] = true;
            let p = pts_of(i, from_end);
            line.extend(p.into_iter().skip(if line.is_empty() { 0 } else { 1 }));
            match next.get(&(i, !from_end)) {
                Some(&(j, j_end)) => {
                    i = j;
                    from_end = j_end;
                }
                None => break,
            }
        }
        // along it, every 0.02 of a cell: a run ends where the line has turned STRAIGHT_ON from its beginning
        let mut run = 0.0f32;
        let mut first: Option<Vec2> = None;
        for w in line.windows(2) {
            let d = w[1] - w[0];
            let len = d.length();
            if len < 1e-6 {
                continue;
            }
            let dir = d / len;
            match first {
                Some(f) if f.angle_to(dir).abs().to_degrees() < STRAIGHT_ON => run += len,
                _ => {
                    if run > 0.0 {
                        runs.push(run);
                    }
                    first = Some(dir);
                    run = len;
                }
            }
        }
        if run > 0.0 {
            runs.push(run);
        }
    }
    out.runs = runs.len();
    out.run_mean = runs.iter().sum::<f32>() / runs.len().max(1) as f32;
    out.run_mean_by_length = runs.iter().map(|r| r * r).sum::<f32>() / runs.iter().sum::<f32>().max(1e-9);
    out.run_longest = runs.iter().copied().fold(0.0, f32::max);
    out
}

/// Every point of every road drawn in the years `ks` (in order), against
/// the same road's points in the first of those years it was drawn: how
/// many links were drawn again with a point moved, and the furthest a
/// point moved (none and nought, if no road's line ever moves).
pub fn moved(r: &Replay, g: &Ground, cen: &[(f32, f32)], ks: impl IntoIterator<Item = usize>) -> (usize, f32) {
    let mut first: HashMap<(u32, u32), Vec<Vec2>> = HashMap::new();
    let (mut moved, mut most) = (0, 0.0f32);
    for k in ks {
        for (c, m, _) in network::drawn_links(r, &g.worn, k) {
            let pts = network::course(r, g, cen, c, m, k).pts;
            match first.get(&key(c, m)) {
                None => {
                    first.insert(key(c, m), pts);
                }
                Some(was) => {
                    let by = if was.len() != pts.len() { f32::INFINITY } else { was.iter().zip(&pts).map(|(p, q)| p.distance(*q)).fold(0.0, f32::max) };
                    if by > 0.0 {
                        moved += 1;
                        most = most.max(by);
                    }
                }
            }
        }
    }
    (moved, most)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Island 7 under the game's rules, which keep the traffic, played forty
    /// years and read back from its record. It leans on no history but the
    /// traffic being kept (the roads the carts wore, `network::drawn_links`,
    /// with the curves on): no point of any road drawn moves from the year it
    /// is first drawn to the last; most roads are drawn on curves, and where
    /// a road passes through a meeting place the line turns there less than a
    /// degree at most of them, where straight it turned at every one; no
    /// link's way crosses a river more often than the way drawn before; and
    /// the straight runs are shorter than the straight lines' were.
    #[test]
    fn roads_bend_through_their_meeting_places_and_never_move() {
        let mut game = isostate_core::save::Game::new(7, 6, 0);
        assert!(game.sim.params.traffic.enabled, "the case is played with the traffic kept");
        for _ in 0..40 {
            game.year().unwrap();
            game.turn();
        }
        let s = &game.sim;
        let bytes = isostate_core::record::pack(&s.st, &s.content, &s.params, &s.events, &s.rec, "test", false);
        let r = crate::replay::parse("test.bin", &bytes).expect("the game's record reads");
        let cen = network::centres(&r);
        let last = r.filled.min(r.ny) - 1;
        let g = crate::world::build_ground(&r);
        let curved = shape(&r, &g, &cen, last);
        assert_eq!(moved(&r, &g, &cen, 0..=last), (0, 0.0), "a road's line moved");
        STRAIGHT_IN_TEST.with(|a| a.set(true));
        let g0 = crate::world::build_ground(&r);
        let straight = shape(&r, &g0, &cen, last);
        STRAIGHT_IN_TEST.with(|a| a.set(false));
        eprintln!("straight {straight:?}\ncurves {curved:?}");
        assert_eq!(straight.bent, 0);
        assert!(curved.links > 300 && curved.bent * 2 > curved.links, "{curved:?}");
        let through: usize = curved.turns.iter().sum();
        assert!(curved.turns[0] * 4 > through * 3, "{:?}", curved.turns);
        assert!(straight.turns[0] * 4 < through, "{:?}", straight.turns);
        assert_eq!(curved.extra_crossings, 0, "{curved:?}");
        assert!(curved.run_mean_by_length < straight.run_mean_by_length, "{curved:?}");
    }

    /// The roads' shape measured on recorded runs (`shape`), straight as
    /// before and on curves, for each run named in `ISOSTATE_CURVE_RUNS`
    /// (paths to `run.bin`, apart by `;`), in 1850, 1900 and 2050, with how
    /// far any road's line moves from one year to the next over the whole
    /// run. Run by hand, with
    /// `cargo test -p isostate-viewer measure_the_curves -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn measure_the_curves() {
        let Ok(paths) = std::env::var("ISOSTATE_CURVE_RUNS") else { return };
        for path in paths.split(';') {
            let bytes = std::fs::read(path).unwrap();
            let r = crate::replay::parse(path, &bytes).unwrap();
            let cen = network::centres(&r);
            for straight in [true, false] {
                STRAIGHT_IN_TEST.with(|a| a.set(straight));
                let g = crate::world::build_ground(&r);
                for year in [1850, 1900, 2050] {
                    let k = (year - r.y0) as usize;
                    if k >= r.ny {
                        continue;
                    }
                    let t0 = std::time::Instant::now();
                    let s = shape(&r, &g, &cen, k);
                    println!("{path} {} {year}: {s:?} ({:.0} ms)", if straight { "straight" } else { "curves" }, t0.elapsed().as_secs_f64() * 1000.0);
                }
                let t0 = std::time::Instant::now();
                let (n, most) = moved(&r, &g, &cen, 0..r.filled.min(r.ny));
                println!("{path} {}: over every year, {n} links drawn again with a point moved, the furthest by {most} ({:.0} ms)", if straight { "straight" } else { "curves" }, t0.elapsed().as_secs_f64() * 1000.0);
                STRAIGHT_IN_TEST.with(|a| a.set(false));
            }
        }
    }
}
