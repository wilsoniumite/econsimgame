//! The course of each road as it is drawn, and never moving once drawn.
//!
//! **Read from the simulation.** Since 27 November 2026 the simulation keeps
//! where each way runs (the core's courses.rs, docs/rules/courses.md): which
//! links the carts wore, each cell's place where its ways meet, and each
//! link's course between two places, found once over the ground by the
//! prices of its kind of road and pulled taut. A run that keeps them (the
//! game's rules on the new land, and every record and game made since: a
//! saved game is played again, so it keeps them too) has every road drawn on
//! its course, from one place to the next (`network::Worn::course`), eased
//! through the course's taut corners on the railways' curve (`eased`), and
//! the squares the simulation hands out face the same line. The viewer finds
//! no way of its own. What the simulation does not keep is still the
//! viewer's: a road's width and surface, its bridges, fords and causeways
//! where its line crosses water (`network::kept_way`), and what lines it.
//!
//! **Laid by the viewer.** A run that keeps no courses (the port's rules, the
//! old island, a record made before 27 November 2026) has each link drawn
//! from one cell's meeting place to the next, on a curve:
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
//! Until 27 November 2026 a road on the new land also sought its own way over
//! the ground between its two ends (`road_ground.rs`, now gone): the
//! simulation's courses took its place.
//!
//! The years are gone through in order, as the roads the carts wore are
//! (`network::Worn`), and what is fixed is kept in `Ground::courses`.
//! `ISOSTATE_CURVES=off` draws every road straight as before, or, where the
//! run keeps its courses, on the course's taut corners with no easing;
//! `ISOSTATE_COURSES=off` draws a run that keeps them as one that does not,
//! to set the two side by side.

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
/// How far apart the points of a kept course that its curve is drawn
/// through are, along each of its straight lengths, in cells (300 m).
const SPACING: f32 = 0.15;
/// How near either side of a course's corner its curve is drawn through a
/// point, in cells (80 m): the curve turns within this of the corner.
const CORNER: f32 = 0.04;

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

/// The courses fixed so far (`Ground::courses`), and the ways drawn for them.
#[derive(Default)]
pub struct Courses {
    known: std::sync::Mutex<Known>,
    /// each link's way, and whether it is the simulation's course
    ways: std::sync::Mutex<HashMap<(u32, u32), (Way, bool)>>,
    /// what the carters' rounds keep from one year to the next (`trains::year_rounds`), which
    /// is read from these courses and so forgotten with them
    pub(crate) carts: std::sync::Mutex<Option<crate::trains::RoadMemo>>,
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
    /// In a test: draw a run that keeps its courses as one that does not.
    pub static NO_COURSES_IN_TEST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the roads of run `r` are drawn on the courses the simulation
/// keeps: wherever it keeps them, unless `ISOSTATE_COURSES=off` asks for
/// the viewer's own curves, to set the two side by side.
pub fn kept(r: &Replay) -> bool {
    static OFF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    network::logged(r) && !NO_COURSES_IN_TEST.with(|a| a.get()) && !*OFF.get_or_init(|| std::env::var("ISOSTATE_COURSES").is_ok_and(|v| v == "off"))
}

impl Courses {
    /// An edit taken back (back.rs): what was fixed from year `from` on is
    /// forgotten; as with the wear, everything is gone through again.
    pub fn forget_from(&self, from: usize) {
        if let Ok(mut known) = self.known.lock() {
            if let Ok(mut ways) = self.ways.lock() {
                ways.clear();
            }
            if from < known.through {
                *known = Known::default();
            }
            if let Ok(mut carts) = self.carts.lock() {
                *carts = None;
            }
        }
    }

    /// Draw the ways of every link drawn in year `k` that has none kept yet,
    /// side by side on all the processor's threads, and keep them. A year's
    /// scenery asks for this first, off the main thread (`year::build_scenery`),
    /// so that whatever reads a course after (the roads, the towns' streets,
    /// the carters' rounds, the Traffic view's ribbons) finds it kept. Each
    /// way is the one `way_found` would draw alone. Returns how many were drawn.
    pub fn find_year(&self, r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> usize {
        use rayon::prelude::*;
        if r.filled == 0 {
            return 0;
        }
        let from_sim = kept(r);
        if !from_sim && !on() {
            return 0;
        }
        // (the ends of every link up to year k, fixed in order, as one walk through the years;
        // or the simulation's log read to year k)
        if from_sim {
            g.worn.place(r, k.min(r.filled.min(r.ny).saturating_sub(1)), 0);
        } else {
            self.fixed(r, g, cen, 0, 0, k);
        }
        let todo: Vec<(usize, usize)> = {
            let known = self.known.lock().unwrap();
            let ways = self.ways.lock().unwrap();
            network::drawn_links(r, &g.worn, k)
                .into_iter()
                .map(|(c, m, _)| (c.min(m), c.max(m)))
                .filter(|&(c, m)| !ways.contains_key(&key(c, m)) && (from_sim || known.ends.get(&key(c, m)).is_some_and(|f| f.bend.is_some())))
                .collect()
        };
        todo.par_iter().for_each(|&(c, m)| {
            self.way_found(r, g, cen, c, m, k);
        });
        todo.len()
    }

    /// How many links' ways are kept (for a test that nothing is drawn where it should not be).
    #[cfg(test)]
    pub fn kept_ways(&self) -> usize {
        self.ways.lock().map_or(0, |w| w.len())
    }

    /// How the link between `c` and `m` is fixed by the viewer, going through
    /// the record up to year `k` first; none where it has not been drawn by
    /// then, or where the run keeps its courses (the viewer fixes nothing).
    pub fn fixed(&self, r: &Replay, g: &Ground, cen: &[(f32, f32)], c: usize, m: usize, k: usize) -> Option<Fixed> {
        if kept(r) {
            return None;
        }
        let mut known = self.known.lock().unwrap();
        go_through(r, g, cen, &mut known, k);
        known.ends.get(&key(c, m)).copied()
    }

    /// The way the link between `c` and `m` is drawn in year `k`.
    pub fn way(&self, r: &Replay, g: &Ground, cen: &[(f32, f32)], c: usize, m: usize, k: usize) -> Way {
        self.way_found(r, g, cen, c, m, k).0
    }

    /// The way the link between `c` and `m` is drawn in year `k`, and whether
    /// it is the simulation's course. A link drawn in a run that keeps its
    /// courses always has one (the simulation finds it the year it first
    /// draws the link); one asked for past what the record holds, or never
    /// drawn, is drawn straight between the two places, as before.
    pub fn way_found(&self, r: &Replay, g: &Ground, cen: &[(f32, f32)], c: usize, m: usize, k: usize) -> (Way, bool) {
        let (lo, hi) = (c.min(m), c.max(m));
        let kept_way = || self.ways.lock().ok().and_then(|w| w.get(&key(lo, hi)).cloned());
        let keep = |found: (Way, bool)| match self.ways.lock() {
            // (the first drawn is the one kept; drawn twice at once, the two are the same)
            Ok(mut w) => w.entry(key(lo, hi)).or_insert(found).clone(),
            Err(_) => found,
        };
        let (way, found) = if kept(r) {
            let last = r.filled.min(r.ny).saturating_sub(1);
            match kept_way() {
                Some(w) => w,
                None => match g.worn.course(r, k.min(last), lo, hi) {
                    Some(pts) => keep((network::kept_way(g, eased(&pts)), true)),
                    None => (network::route(g, network::ways_meet(r, g, cen, k.min(last), lo), network::ways_meet(r, g, cen, k.min(last), hi), r.round()), false),
                },
            }
        } else {
            let (a, b) = (Vec2::new(cen[lo].0, cen[lo].1), Vec2::new(cen[hi].0, cen[hi].1));
            let fixed = if on() { self.fixed(r, g, cen, lo, hi, k) } else { None };
            match fixed.and_then(|f| f.bend) {
                None => (network::route(g, a, b, r.round()), false),
                Some((da, db)) => kept_way().unwrap_or_else(|| keep((network::bent_route(g, a, b, da, db, r.round()), false))),
            }
        };
        (if c <= m { way } else { reversed(way) }, found)
    }
}

/// A course the simulation keeps (its places first and last and its taut
/// corners between), as it is drawn: with points put in along each straight
/// length longer than twice `SPACING`, every `SPACING` or closer, and a chain
/// of the railways' curves (`course::Ends`) through every point, running
/// there from the one before toward the one after (Catmull-Rom's way), and
/// leaving and reaching each place along the course's own first and last
/// lengths. So it keeps to the long straight lengths, flows through the
/// corners of the short ones, and passes through every point the simulation
/// keeps; every 0.04 of a cell or closer. With `ISOSTATE_CURVES=off`, the
/// course's own points, taut.
pub fn eased(pts: &[Vec2]) -> Vec<Vec2> {
    let n = pts.len();
    if n <= 2 || !on() {
        return pts.to_vec();
    }
    let all = along(pts);
    let last = all.len() - 1;
    let (a, b) = (all[0], all[last]);
    let da = (pts[1] - pts[0]).normalize_or(Vec2::X);
    let db = (pts[n - 1] - pts[n - 2]).normalize_or(Vec2::X);
    let mut stops: Vec<(Vec2, Vec2)> = vec![(a, da)];
    for i in 1..last {
        stops.push((all[i], (all[i + 1] - all[i - 1]).normalize_or(da)));
    }
    stops.push((b, db));
    let mut out = vec![a];
    for w in stops.windows(2) {
        let ends = crate::course::Ends { a: w[0].0, da: w[0].1, b: w[1].0, db: w[1].1 };
        let k = ((w[0].0.distance(w[1].0) / 0.04).ceil() as usize).max(2);
        out.extend((1..=k).map(|i| ends.point(i as f32 / k as f32, 0.0)));
    }
    // (the last point is the place itself, to the bit)
    if let Some(p) = out.last_mut() {
        *p = b;
    }
    out
}

/// A line of points, with points put in along each straight length of it
/// longer than twice `SPACING`, every `SPACING` or closer, and `CORNER` either
/// side of each corner where the length has room, its corners kept: so the
/// curve through them turns within `CORNER` of each corner and runs straight
/// between.
fn along(taut: &[Vec2]) -> Vec<Vec2> {
    let n = taut.len();
    let mut out = vec![taut[0]];
    for (i, w) in taut.windows(2).enumerate() {
        let len = w[0].distance(w[1]);
        let k = if len > 2.0 * SPACING { (len / SPACING).ceil() as usize } else { 1 };
        let mut at: Vec<f32> = (1..=k).map(|j| len * j as f32 / k as f32).collect();
        // (a corner at either end of the length: a point CORNER from it, where there is room)
        if len > 3.0 * CORNER {
            if i > 0 {
                at.push(CORNER);
            }
            if i + 2 < n {
                at.push(len - CORNER);
            }
        }
        at.sort_by(f32::total_cmp);
        at.dedup_by(|x, y| (*x - *y).abs() < 0.5 * CORNER);
        // (the end itself is last, to the bit)
        if let Some(t) = at.last_mut() {
            *t = len;
        }
        out.extend(at.into_iter().map(|t| if t >= len { w[1] } else { w[0].lerp(w[1], t / len) }));
    }
    out
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

/// The slope of the ground along a line of points, as the land itself has
/// it (in metres for each metre along): the mean over its length and the
/// share of its length steeper than 1 in 12, read every 0.02 of a cell
/// (40 m) and over a lake at its water. Returned as the sums (slope times
/// length, length steeper, length).
pub fn slope_along(g: &Ground, pts: &[Vec2]) -> (f32, f32, f32) {
    let relief = if g.laid.is_some() { crate::newland::RELIEF } else { 3.0 };
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
    /// the links drawn on a curve of the viewer's (the rest as before,
    /// straight or round water, or on the simulation's course), and the
    /// links held straight for a square of town ground
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
    /// the same for the ways on the simulation's courses, the links held
    /// straight on town ground and the rest, each with its share of the length
    pub on_eight_by: [(f32, f32); 3],
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
    /// where the first few links on a curve cross a river (for pictures)
    pub bent_crossings: Vec<Vec2>,
    /// the links drawn on the simulation's course (the core's courses.rs)
    pub kept: usize,
    /// the slope of the ground along the drawn ways, as the land has it: the
    /// mean over their length (metres for each metre along), and the share of
    /// their length steeper than 1 in 12 (`slope_along`)
    pub slope: f32,
    pub steep: f32,
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
    let found: Vec<(Way, bool)> = links.iter().map(|&(c, m, _)| g.courses.way_found(r, g, cen, c, m, k)).collect();
    out.kept = found.iter().filter(|f| f.1).count();
    let from_sim: Vec<bool> = found.iter().map(|f| f.1).collect();
    let ways: Vec<Way> = found.into_iter().map(|f| f.0).collect();
    let mut by = [(0.0f32, 0.0f32); 3];
    let (mut slope, mut steep, mut along) = (0.0f32, 0.0f32, 0.0f32);
    for way in &ways {
        let (s, st, l) = slope_along(g, &way.pts);
        (slope, steep, along) = (slope + s, steep + st, along + l);
    }
    (out.slope, out.steep) = (slope / along.max(1e-9), steep / along.max(1e-9));
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
        let group = if from_sim[i] { 0 } else if fixed.is_some_and(|f| f.held) { 1 } else { 2 };
        by[group].1 += len;
        for w in p.windows(2) {
            let d = w[1] - w[0];
            let deg = d.y.atan2(d.x).to_degrees().rem_euclid(45.0);
            if deg.min(45.0 - deg) < 5.0 {
                out.on_eight += d.length();
                by[group].0 += d.length();
            }
        }
        let tight = crate::course::tightest(p, 0.08);
        out.tightest = out.tightest.min(tight);
        out.tight_links += (tight < 0.25) as usize;
        let cuts = |pts: &[Vec2]| pts.windows(2).map(|w| crate::rivers::crossing_points(&g.rivers, &g.water, w[0], w[1]).len()).sum::<usize>();
        let drawn = cuts(p);
        out.crossings += drawn;
        let (a, b) = (p[0], p[n - 1]);
        out.extra_crossings += (drawn > cuts(&network::route(g, a, b, r.round()).pts)) as usize;
        for &(at, river, _, _, along) in &way.crossings {
            square += along.perp_dot(river).abs();
            crossed += 1;
            if bent && out.bent_crossings.len() < 12 {
                out.bent_crossings.push(at);
            }
        }
    }
    out.on_eight /= out.length.max(1e-9);
    out.on_eight_by = by.map(|(on, all)| (on / all.max(1e-9), all / out.length.max(1e-9)));
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

/// The distance from `p` to the nearest point of a line of points.
#[cfg(test)]
fn off_line(line: &[Vec2], p: Vec2) -> f32 {
    line.windows(2)
        .map(|w| {
            let ab = w[1] - w[0];
            let t = ((p - w[0]).dot(ab) / ab.length_squared().max(1e-12)).clamp(0.0, 1.0);
            p.distance(w[0] + ab * t)
        })
        .fold(f32::INFINITY, f32::min)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Island 7 played forty years and read back from its record.
    fn island_7() -> (isostate_core::save::Game, Replay) {
        let mut game = isostate_core::save::Game::new(7, 6, 0);
        for _ in 0..40 {
            game.year().unwrap();
            game.turn();
        }
        let s = &game.sim;
        let bytes = isostate_core::record::pack(&s.st, &s.content, &s.params, &s.events, &s.rec, "test", false);
        let r = crate::replay::parse("test.bin", &bytes).expect("the game's record reads");
        (game, r)
    }

    /// The pass of 27 November 2026: on a new record, no point of a drawn
    /// way comes from the viewer's own finding. Island 7 under the game's
    /// rules, with the traffic and the courses kept by name, played forty
    /// years. Every road drawn in every year runs on the simulation's course
    /// for its link: it begins at the place the simulation keeps for its one
    /// cell and ends at the other's, passes through every point the course
    /// keeps, and keeps within a fiftieth of a cell (40 m) of the course's
    /// line all along (its easing through the course's corners); with the
    /// curves off it is the course's points and nothing else. No point of any
    /// road moves from the year it is first drawn to the last, every link
    /// drawn has its course, and the simulation's places of most town cells
    /// are the meeting places the viewer draws their markets at (a town that
    /// grew where a way had passed before keeps the way's place).
    #[test]
    fn every_road_drawn_runs_on_the_simulations_course() {
        let (game, r) = island_7();
        let s = &game.sim;
        assert!(s.params.traffic.enabled && s.params.courses.enabled, "the case is played with the traffic and the courses kept");
        assert!(kept(&r), "the record keeps its ways");
        let cen = network::centres(&r);
        let last = (game.last_year() - s.content.start_year) as usize;
        let g = crate::world::build_ground(&r);
        let (mut links, mut furthest, mut places, mut towns) = (0usize, 0.0f32, std::collections::HashSet::new(), 0usize);
        let mut at_market = 0usize;
        for k in [0, 10, 25, last] {
            for (c, m, _) in network::drawn_links(&r, &g.worn, k) {
                let course = g.worn.course(&r, k, c, m).unwrap_or_else(|| panic!("year {k}: the link {c}-{m} is drawn with no course"));
                let (way, from_sim) = g.courses.way_found(&r, &g, &cen, c, m, k);
                assert!(from_sim, "year {k}: the link {c}-{m} is not drawn on its course");
                let (pa, pb) = (g.worn.place(&r, k, c).unwrap(), g.worn.place(&r, k, m).unwrap());
                let n = way.pts.len();
                assert!(way.pts[0] == pa && way.pts[n - 1] == pb && course[0] == pa && course[course.len() - 1] == pb, "year {k}: the link {c}-{m} does not run from place to place");
                for &q in course.iter() {
                    assert!(off_line(&way.pts, q) < 1e-4, "year {k}: the link {c}-{m} misses its course's point {q}");
                }
                let off = way.pts.iter().map(|&p| off_line(&course, p)).fold(0.0, f32::max);
                assert!(off < 0.02, "year {k}: the link {c}-{m} runs {off} of a cell off its course");
                furthest = furthest.max(off);
                links += 1;
                for x in [c, m] {
                    if places.insert(x) && network::town_ground(&r, k, x) && r.header.towns.iter().any(|t| t.node == x) {
                        towns += 1;
                        let p = g.worn.place(&r, k, x).unwrap();
                        if p.distance(Vec2::new(cen[x].0, cen[x].1)) < 0.01 {
                            at_market += 1;
                        } else {
                            eprintln!("year {k}: the town of cell {x} has its market at {:?} and its ways meet at {p}", cen[x]);
                        }
                    }
                }
            }
        }
        assert_eq!(moved(&r, &g, &cen, 0..=last), (0, 0.0), "a road's line moved");
        // with the curves off, the course's own points
        STRAIGHT_IN_TEST.with(|a| a.set(true));
        let g0 = crate::world::build_ground(&r);
        for (c, m, _) in network::drawn_links(&r, &g0.worn, last) {
            assert_eq!(&g0.courses.way(&r, &g0, &cen, c, m, last).pts[..], &g0.worn.course(&r, last, c, m).unwrap()[..]);
        }
        STRAIGHT_IN_TEST.with(|a| a.set(false));
        eprintln!("{links} links drawn in four years, at most {furthest} of a cell off their courses; of {towns} towns' markets, {at_market} where their ways meet");
        assert!(links > 1000 && towns > 3 && at_market * 4 >= towns * 3, "{links} links; of {towns} towns' markets, {at_market} where their ways meet");
    }

    /// A run that keeps no courses (here the same record, read as one that
    /// does not): the viewer's curves through the cells' meeting places, as
    /// before. No point of any road drawn moves from the year it is first
    /// drawn to the last; most roads are drawn on curves, and where a road
    /// passes through a meeting place the line turns there less than 5
    /// degrees at most of them, where straight it did at under a quarter; no
    /// link's way crosses a river more often than the way drawn before; and
    /// the straight runs are shorter than the straight lines' were.
    #[test]
    fn without_courses_roads_bend_through_their_meeting_places_and_never_move() {
        let (game, r) = island_7();
        let s = &game.sim;
        NO_COURSES_IN_TEST.with(|a| a.set(true));
        let cen = network::centres(&r);
        let last = (game.last_year() - s.content.start_year) as usize;
        let g = crate::world::build_ground(&r);
        let curved = shape(&r, &g, &cen, last);
        assert_eq!(moved(&r, &g, &cen, 0..=last), (0, 0.0), "a road's line moved");
        STRAIGHT_IN_TEST.with(|a| a.set(true));
        let g0 = crate::world::build_ground(&r);
        let straight = shape(&r, &g0, &cen, last);
        STRAIGHT_IN_TEST.with(|a| a.set(false));
        NO_COURSES_IN_TEST.with(|a| a.set(false));
        eprintln!("straight {straight:?}\ncurves {curved:?}");
        assert_eq!((straight.bent, curved.kept), (0, 0));
        assert!(curved.links > 300 && curved.bent * 2 > curved.links, "{curved:?}");
        let through: usize = curved.turns.iter().sum();
        assert!((curved.turns[0] + curved.turns[1]) * 2 > through, "{:?}", curved.turns);
        assert!((straight.turns[0] + straight.turns[1]) * 4 < through, "{:?}", straight.turns);
        assert_eq!(curved.extra_crossings, 0, "{curved:?}");
        assert!(curved.run_mean_by_length < straight.run_mean_by_length, "{curved:?}");
    }

    /// A course with corners is drawn through every one of its points, from
    /// its first along its first length to its last along its last.
    #[test]
    fn a_kept_course_is_eased_through_its_points() {
        let pts = [Vec2::new(0.0, 0.0), Vec2::new(0.5, 0.0), Vec2::new(0.7, 0.3), Vec2::new(1.0, 0.3)];
        let drawn = eased(&pts);
        let n = drawn.len();
        assert_eq!((drawn[0], drawn[n - 1]), (pts[0], pts[3]));
        assert!((drawn[1] - drawn[0]).normalize().dot(Vec2::X) > 0.999 && (drawn[n - 1] - drawn[n - 2]).normalize().dot(Vec2::X) > 0.999);
        for p in pts {
            assert!(off_line(&drawn, p) < 1e-5, "{p:?}");
        }
        for w in drawn.windows(2) {
            assert!(w[0].distance(w[1]) <= 0.04 + 1e-6, "{w:?}");
        }
        // and a course with no corner is its straight line
        assert_eq!(eased(&pts[..2]), pts[..2].to_vec());
    }

    /// What drawing a year's roads costs side by side (`Courses::find_year`)
    /// against one after another, and that they are the same ways: for the
    /// first run named in `ISOSTATE_CURVE_RUNS`, the year `ISOSTATE_CURVE_YEAR`
    /// (its last if not named), every link's way drawn on a fresh ground each
    /// way, by the clock on the wall. Run by hand, with
    /// `cargo test -p isostate-viewer time_a_year_found_side_by_side -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn time_a_year_found_side_by_side() {
        let Ok(paths) = std::env::var("ISOSTATE_CURVE_RUNS") else { return };
        let path = paths.split(';').next().unwrap();
        let bytes = std::fs::read(path).unwrap();
        let r = crate::replay::parse(path, &bytes).unwrap();
        let cen = network::centres(&r);
        let last = r.filled.min(r.ny) - 1;
        let k = std::env::var("ISOSTATE_CURVE_YEAR").ok().and_then(|v| v.parse::<i32>().ok()).map_or(last, |y| ((y - r.y0) as usize).min(last));
        for _ in 0..2 {
            let g = crate::world::build_ground(&r);
            let t0 = std::time::Instant::now();
            g.courses.fixed(&r, &g, &cen, 0, 0, k);
            g.worn.place(&r, k, 0);
            let walk = t0.elapsed().as_secs_f64() * 1000.0;
            let links = network::drawn_links(&r, &g.worn, k);
            let t1 = std::time::Instant::now();
            let n = g.courses.find_year(&r, &g, &cen, k);
            let side = t1.elapsed().as_secs_f64() * 1000.0;
            let g1 = crate::world::build_ground(&r);
            g1.courses.fixed(&r, &g1, &cen, 0, 0, k);
            g1.worn.place(&r, k, 0);
            let t2 = std::time::Instant::now();
            for &(c, m, _) in &links {
                g1.courses.way_found(&r, &g1, &cen, c, m, k);
            }
            let alone = t2.elapsed().as_secs_f64() * 1000.0;
            let same = links.iter().all(|&(c, m, _)| g.courses.way(&r, &g, &cen, c, m, k).pts == g1.courses.way(&r, &g1, &cen, c, m, k).pts);
            println!("{}: the years walked to it in {walk:.0} ms; {} links drawn, {n} ways drawn side by side in {side:.0} ms; one after another in {alone:.0} ms; the same ways: {same}", r.y0 + k as i32, links.len());
            assert!(same);
        }
    }

    /// Where the simulation's courses bend most, to aim pictures at: for the
    /// first run named in `ISOSTATE_CURVE_RUNS`, in the year
    /// `ISOSTATE_CURVE_YEAR` (2050 if not named), the twelve links drawn whose
    /// course lies furthest from the straight line between its two places,
    /// with where the link's middle is, how far, and the slope along each.
    /// Run by hand.
    #[test]
    #[ignore]
    fn where_the_courses_bend_most() {
        let Ok(paths) = std::env::var("ISOSTATE_CURVE_RUNS") else { return };
        let path = paths.split(';').next().unwrap();
        let bytes = std::fs::read(path).unwrap();
        let r = crate::replay::parse(path, &bytes).unwrap();
        let cen = network::centres(&r);
        let year: i32 = std::env::var("ISOSTATE_CURVE_YEAR").ok().and_then(|v| v.parse().ok()).unwrap_or(2050);
        let k = ((year - r.y0) as usize).min(r.filled.min(r.ny) - 1);
        let g = crate::world::build_ground(&r);
        let mut rows: Vec<(f32, String)> = Vec::new();
        for (c, m, lv) in network::drawn_links(&r, &g.worn, k) {
            let way = g.courses.way(&r, &g, &cen, c, m, k);
            let n = way.pts.len();
            let chord = [way.pts[0], way.pts[n - 1]];
            let off = way.pts.iter().map(|&p| off_line(&chord, p)).fold(0.0, f32::max);
            let (s0, _, l0) = slope_along(&g, &chord);
            let (s1, _, l1) = slope_along(&g, &way.pts);
            let mid = (chord[0] + chord[1]) * 0.5;
            rows.push((off, format!("link {c}-{m} level {lv}: middle {:.2},{:.2}, lies {off:.3} off its straight line; slope {:.4} straight, {:.4} on its course; length {l0:.3} and {l1:.3}", mid.x, mid.y, s0 / l0.max(1e-9), s1 / l1.max(1e-9))));
        }
        rows.sort_by(|x, y| y.0.total_cmp(&x.0));
        println!("{} links drawn in {year}", rows.len());
        for (_, row) in rows.iter().take(12) {
            println!("{row}");
        }
    }

    /// The roads' shape measured on recorded runs (`shape`), straight as
    /// before, on the viewer's curves and on the simulation's courses, for each run named in `ISOSTATE_CURVE_RUNS`
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
            for how in ["straight", "curves", "kept"] {
                STRAIGHT_IN_TEST.with(|a| a.set(how == "straight"));
                NO_COURSES_IN_TEST.with(|a| a.set(how != "kept"));
                let g = crate::world::build_ground(&r);
                for year in [1850, 1900, 2050] {
                    let k = (year - r.y0) as usize;
                    if k >= r.ny {
                        continue;
                    }
                    let t0 = std::time::Instant::now();
                    let s = shape(&r, &g, &cen, k);
                    println!("{path} {how} {year}: {s:?} ({:.0} ms)", t0.elapsed().as_secs_f64() * 1000.0);
                }
                let t0 = std::time::Instant::now();
                let (n, most) = moved(&r, &g, &cen, 0..r.filled.min(r.ny));
                println!("{path} {how}: over every year, {n} links drawn again with a point moved, the furthest by {most} ({:.0} ms)", t0.elapsed().as_secs_f64() * 1000.0);
                STRAIGHT_IN_TEST.with(|a| a.set(false));
                NO_COURSES_IN_TEST.with(|a| a.set(false));
            }
        }
    }
}
