//! Courses: which ways the carts wore, where each passes through its cells
//! and the course it runs between them, kept by the simulation (not in the
//! Python; `docs/rules/courses.md`). It only watches: nothing in the economy
//! reads it. The squares (`squares.rs`) turn to it, and the record keeps it.
//!
//! - **Which ways exist.** The viewer has drawn a road between two cells only
//!   where carts wore one since 26 November 2026 (its `network::drawn_links`).
//!   The same rule runs here, on the same numbers at the same moment of the
//!   year: a link between two squares of made road is worn when it has carried
//!   `WORN_WEIGHT` or more by cart in `WORN_YEARS` of the last `WORN_WINDOW`
//!   years (a year of `WORN_HEAVY` counting `HEAVY_COUNTS`; in the record's
//!   first years, in every year so far), goes back to grass after
//!   `WORN_WINDOW` years in which it never did, and a trunk road's or a
//!   bypass's link stands while `road_segments`' rule joins it. The weight is
//!   read as the record's byte keeps it (`traffic::goods_byte`), so that the
//!   two agree to the link.
//! - **A cell's place.** The ways through a cell pass at one place, chosen the
//!   year the first of them reaches it and never moved after (`place`). A
//!   town's cell keeps its meeting place, where its market and its streets are
//!   drawn. Elsewhere the ground chooses: a spot of the cell that is dry, not
//!   on a river's band, not steep, where the ways that meet there that year run
//!   cheapest to the places at their other ends, leaning a little toward the
//!   meeting place.
//! - **A link's course.** Found once, the year the link is first drawn, over
//!   the ground between its two places (`seek`): the cheapest way over squares
//!   of 125 m by the prices of its kind that year (the viewer's lane, turnpike
//!   and trunk road), pulled taut. Kept for ever: a road that rises a level
//!   keeps the course it was found on, as the game draws it today, and a road
//!   that went back to grass and is worn again comes back on it.
//!
//! The island's roads are older than the record: as the run opens (`open`)
//! the made roads that stand are given their places and courses, so that the
//! squares of 1750 line them; the years after go by the carts.
//!
//! What it writes for the record and a live game is a log of each year's
//! changes (`tell`, `encode_year`): the places chosen, the courses found and
//! the links drawn and no longer drawn.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::crossings::{self, Fine, OnLand};
use crate::params::Params;
use crate::state::State;
use crate::traffic;

/// The switch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CourseParams {
    /// The simulation keeps which ways the carts wore, each cell's place and
    /// each way's course, and the squares face them.
    pub enabled: bool,
}

impl Default for CourseParams {
    /// Off: what the port runs with.
    fn default() -> CourseParams {
        CourseParams { enabled: false }
    }
}

impl CourseParams {
    /// The game's courses.
    pub fn game() -> CourseParams {
        CourseParams { enabled: true }
    }
}

// ---------------------------------------------------------------- the viewer's numbers

/// What a link must carry by cart in a year for that year to wear it (the
/// viewer's `network::WORN_WEIGHT`).
pub const WORN_WEIGHT: f32 = 1500.0;
/// In how many years of the last `WORN_WINDOW` (`network::WORN_YEARS`, `WORN_WINDOW`).
pub const WORN_YEARS: u32 = 10;
pub const WORN_WINDOW: u32 = 20;
/// A worn link goes back to grass once fewer than this many of the last
/// `WORN_WINDOW` years wore it (`network::WORN_FADE`).
pub const WORN_FADE: u32 = 1;
/// A year of this much counts `HEAVY_COUNTS` (`network::WORN_HEAVY`, `HEAVY_COUNTS`).
pub const WORN_HEAVY: f32 = 40_000.0;
pub const HEAVY_COUNTS: u32 = 5;

/// A point's unit: 1/256 of a cell (7.8 m). Places and courses are kept in it,
/// so the state, the record and a live game hold the same numbers.
pub const UNIT: f32 = 256.0;
/// Squares to a cell's side, as the squares and the viewer's search have them.
const FINE: usize = 16;

/// The kinds of road a course is found for (the viewer's `road_ground::Kind`).
pub const LANE: u8 = 0;
pub const TURNPIKE: u8 = 1;
pub const TRUNK: u8 = 2;
pub const KINDS: [&str; 3] = ["lane", "turnpike", "trunk"];

/// What a road of one kind pays besides its length (a cell of length is 1).
#[derive(Clone, Copy, Debug)]
struct Price {
    /// metres of level road a metre climbed or fallen is worth
    rise: f32,
    /// the steepest its carts take, as 1 in this
    ruling: f32,
    /// metres of level road a metre beyond that is worth
    steep: f32,
    /// for each cell of length, what each cell off the straight line costs
    off_line: f32,
    /// how much of the price of wet ground it pays
    wet: f32,
}

/// The viewer's prices (`road_ground::LANE`, `TURNPIKE`, `TRUNK`).
fn price(kind: u8) -> Price {
    match kind {
        TRUNK => Price { rise: 10.0, ruling: 25.0, steep: 30.0, off_line: 3.0, wet: 0.5 },
        TURNPIKE => Price { rise: 20.0, ruling: 20.0, steep: 60.0, off_line: 1.0, wet: 1.0 },
        _ => Price { rise: 20.0, ruling: 12.0, steep: 60.0, off_line: 0.5, wet: 1.0 },
    }
}

/// Standing water, ground too low to build on, marsh and a valley's floor,
/// for each cell of length (the railways' prices, `newland::passing`).
const WATER: f32 = 40.0;
const LOW: f32 = 3.0;
const MARSH: f32 = 1.5;
const FLOOR: f32 = 0.3;
/// A river's band, for each cell of length: crossing it squarely costs some
/// cell of way round, running along it far more.
const RIVER: f32 = 16.0;
/// A town's ground that is neither of the link's own cells, for each cell of length.
const TOWN: f32 = 6.0;
/// How far round the straight line between its places a way may stray, in cells.
const PAD: f32 = 0.4;
/// How much dearer than the way over the squares a straight line may be and still be taken.
const SLACK: f32 = 0.03;
/// Metres to a cell's side.
const CELL_M: f32 = 2000.0;
/// Below this a spot is too low to build on (`crossings::DRY_M`).
const DRY_M: f32 = 1.8;

/// The sixteen steps a way may take from a square (the viewer's `road_ground::STEPS`).
const STEPS: [(isize, isize); 16] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1), (2, 1), (2, -1), (-2, 1), (-2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2)];

// ---------------------------------------------------------------- the book

/// One link's course: from the place of the link's own cell (`c` of `c * 4 + slot`)
/// to the place of its neighbour, in `UNIT`s.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Course {
    /// The year it was found (the year before the record's first for a road of the opening).
    pub year: i16,
    /// The kind it was found for (`LANE`, `TURNPIKE`, `TRUNK`), the link's that year.
    pub kind: u8,
    /// Its points, the two places first and last, `[x, y]` in `UNIT`s.
    pub pts: Vec<[u16; 2]>,
}

/// One cell's ground as a way finds it, square by square.
#[derive(Clone, Debug)]
struct Patch {
    /// metres above the sea at the square's middle
    metres: [f32; FINE * FINE],
    /// what lying there costs besides its length, for each cell of length
    dear: [f32; FINE * FINE],
    /// a river's water within reach
    river: [bool; FINE * FINE],
}

/// Everything the courses keep. Empty while the switch is off.
#[derive(Clone, Debug, Default)]
pub struct CourseBook {
    /// By link (`c * 4 + slot`): the last 32 years, worn and worn heavily, the latest lowest.
    bits: Vec<u32>,
    heavy: Vec<u32>,
    /// By link: worn now.
    pub worn: Vec<bool>,
    /// By link: drawn this year (worn, or a state's road `road_segments` joins).
    pub drawn: Vec<bool>,
    /// By link: the year it was first drawn, 0 never.
    pub first: Vec<i16>,
    /// Years gone through, from the record's first.
    pub through: usize,
    /// By cell: its place in `UNIT`s, and the year it was chosen (0: none yet).
    pub place: Vec<[u16; 2]>,
    pub placed: Vec<i16>,
    /// By link: its course, once found.
    pub courses: BTreeMap<u32, Course>,
    /// By cell: the straight lengths of the kept courses that come within half
    /// a cell of it, in cells (what the squares face).
    pub near: Vec<Vec<[f32; 4]>>,
    /// By cell: the ground, read the first time a way needs it.
    patches: Vec<Option<Box<Patch>>>,
    /// By cell: the river lengths within reach of it, with their half widths, in cells.
    rivers: Vec<Vec<[f32; 5]>>,
    /// What the log has told: each link drawn, the places and courses told.
    told_drawn: Vec<bool>,
    new_places: Vec<u32>,
    new_courses: Vec<u32>,
    /// The log of the last year's end (`encode_year`).
    pub log: Vec<u8>,
    /// Year's ends logged so far.
    pub told: u64,
    /// Seconds spent finding places and courses, and how many were found (for the measure).
    pub found: (usize, usize),
}

impl CourseBook {
    /// Whether the book is kept in this run.
    pub fn kept(&self) -> bool {
        !self.place.is_empty()
    }

    /// A place or a course's point in cells.
    pub fn at(p: [u16; 2]) -> (f32, f32) {
        (p[0] as f32 / UNIT, p[1] as f32 / UNIT)
    }

    /// The place of cell c in cells, if it has one.
    pub fn place_of(&self, c: usize) -> Option<(f32, f32)> {
        (self.placed.get(c).copied().unwrap_or(0) != 0).then(|| CourseBook::at(self.place[c]))
    }
}

fn start_book(st: &mut State) {
    let (N, W, H) = (st.N, st.world.W, st.world.H);
    let mut rivers: Vec<Vec<[f32; 5]>> = vec![Vec::new(); N];
    if let Some(lf) = st.world.landform.as_deref() {
        for r in &lf.rivers {
            for i in 1..r.x.len() {
                // drawn from a cubic metre a second, and above the tide (below it the tideway is water)
                if r.flow[i] < crossings::MIN_FLOW || r.level[i] <= 0.0 {
                    continue;
                }
                let half = 0.5 * r.width[i] / CELL_M + 0.05;
                let s = [r.x[i - 1], r.y[i - 1], r.x[i], r.y[i], half];
                let (x0, x1) = ((s[0].min(s[2]) - half).floor() as isize, (s[0].max(s[2]) + half).floor() as isize);
                let (y0, y1) = ((s[1].min(s[3]) - half).floor() as isize, (s[1].max(s[3]) + half).floor() as isize);
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        if x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H {
                            rivers[y as usize * W + x as usize].push(s);
                        }
                    }
                }
            }
        }
    }
    st.wc = CourseBook {
        bits: vec![0; N * 4],
        heavy: vec![0; N * 4],
        worn: vec![false; N * 4],
        drawn: vec![false; N * 4],
        first: vec![0; N * 4],
        place: vec![[0, 0]; N],
        placed: vec![0; N],
        near: vec![Vec::new(); N],
        patches: vec![None; N],
        rivers,
        told_drawn: vec![false; N * 4],
        ..CourseBook::default()
    };
}

/// Whether the rule acts in this run: it is on, the run keeps its traffic, and
/// the island is the new land (whose heights and rivers the courses read).
pub fn active(st: &State, P: &Params) -> bool {
    P.courses.enabled && P.traffic.enabled && st.world.landform.is_some()
}

// ---------------------------------------------------------------- the ground

fn seg_dist(px: f32, py: f32, s: &[f32]) -> f32 {
    let (dx, dy) = (s[2] - s[0], s[3] - s[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((px - s[0]) * dx + (py - s[1]) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    (px - s[0] - t * dx).hypot(py - s[1] - t * dy)
}

fn patch(st: &State, rivers: &[[f32; 5]], c: usize) -> Patch {
    let w = st.world.W;
    let lf = st.world.landform.as_deref().expect("the new land");
    let l = OnLand { lf, w, h: st.world.H };
    let (cx, cy) = ((c % w) as f32, (c / w) as f32);
    let mut p = Patch { metres: [0.0; FINE * FINE], dear: [0.0; FINE * FINE], river: [false; FINE * FINE] };
    for k in 0..FINE * FINE {
        let (x, y) = (cx + ((k % FINE) as f32 + 0.5) / FINE as f32, cy + ((k / FINE) as f32 + 0.5) / FINE as f32);
        let m = crossings::metres_at(&l, x, y);
        let n = crossings::node_at(&l, x, y);
        let mut e = if m <= 0.0 || l.lake(n) > 0 {
            WATER
        } else if m < DRY_M {
            LOW
        } else {
            0.0
        };
        e += match l.wet(n) {
            0 => 0.0,
            1 => FLOOR,
            _ => MARSH,
        };
        p.metres[k] = m;
        p.dear[k] = e;
        p.river[k] = rivers.iter().any(|s| seg_dist(x, y, s) <= s[4]);
    }
    p
}

/// The ground of every cell a box of the map touches, read if it has not been.
fn know(st: &mut State, lo: (f32, f32), hi: (f32, f32)) {
    let (w, h) = (st.world.W, st.world.H);
    let (x0, x1) = (lo.0.floor().max(0.0) as usize, (hi.0.floor().max(0.0) as usize).min(w - 1));
    let (y0, y1) = (lo.1.floor().max(0.0) as usize, (hi.1.floor().max(0.0) as usize).min(h - 1));
    for y in y0..=y1 {
        for x in x0..=x1 {
            let c = y * w + x;
            if st.wc.patches[c].is_none() {
                let p = patch(st, &st.wc.rivers[c], c);
                st.wc.patches[c] = Some(Box::new(p));
            }
        }
    }
}

/// The ground at a square of the whole map (`FINE` to a cell's side): its
/// metres, its price for wet ground, whether a river is there, and its cell.
/// The cell's ground must be known (`know`).
fn square_at(b: &CourseBook, w: usize, sx: usize, sy: usize) -> (f32, f32, bool, usize) {
    let c = (sy / FINE) * w + sx / FINE;
    let k = (sy % FINE) * FINE + sx % FINE;
    let p = b.patches[c].as_deref().expect("the ground is known");
    (p.metres[k], p.dear[k], p.river[k], c)
}

#[derive(PartialEq)]
struct Reach(f32, u32);
impl Eq for Reach {}
impl Ord for Reach {
    fn cmp(&self, other: &Reach) -> std::cmp::Ordering {
        other.0.total_cmp(&self.0).then(other.1.cmp(&self.1))
    }
}
impl PartialOrd for Reach {
    fn partial_cmp(&self, other: &Reach) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// The cheapest way from `a` to `b` for a road of `kind`, over the squares in
/// a box `PAD` round them, pulled taut: its points between `a` and `b` (not
/// those two). `own` are the link's own cells, `town` tells a town's cell.
/// Empty where the two are in one square or no way is found.
fn seek(st: &State, a: (f32, f32), b: (f32, f32), kind: u8, own: [usize; 2], town: &[bool]) -> Vec<(f32, f32)> {
    let pr = price(kind);
    let (w, h) = (st.world.W, st.world.H);
    let f = FINE as f32;
    let (fw, fh) = ((w * FINE) as isize, (h * FINE) as isize);
    let (x0, y0) = ((((a.0.min(b.0) - PAD) * f).floor() as isize).max(0), (((a.1.min(b.1) - PAD) * f).floor() as isize).max(0));
    let (x1, y1) = ((((a.0.max(b.0) + PAD) * f).floor() as isize).min(fw - 1), (((a.1.max(b.1) + PAD) * f).floor() as isize).min(fh - 1));
    let (nx, ny) = ((x1 - x0 + 1) as usize, (y1 - y0 + 1) as usize);
    let square = |p: (f32, f32)| -> Option<usize> {
        let (sx, sy) = ((p.0 * f).floor() as isize, (p.1 * f).floor() as isize);
        (sx >= x0 && sx <= x1 && sy >= y0 && sy <= y1).then(|| (sy - y0) as usize * nx + (sx - x0) as usize)
    };
    let (Some(start), Some(goal)) = (square(a), square(b)) else { return Vec::new() };
    if start == goal {
        return Vec::new();
    }
    let n = nx * ny;
    let (mut ground, mut dear, mut river) = (vec![0.0f32; n], vec![0.0f32; n], vec![false; n]);
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len_ab = dx.hypot(dy).max(1e-6);
    for i in 0..n {
        let (sx, sy) = ((x0 + (i % nx) as isize) as usize, (y0 + (i / nx) as isize) as usize);
        let (m, e, r, c) = square_at(&st.wc, w, sx, sy);
        ground[i] = m;
        let mut d = e * pr.wet;
        if !own.contains(&c) && town[c] {
            d += TOWN;
        }
        // off the straight line between the places
        let (px, py) = ((sx as f32 + 0.5) / f, (sy as f32 + 0.5) / f);
        d += pr.off_line * ((px - a.0) * dy - (py - a.1) * dx).abs() / len_ab;
        if r {
            d += RIVER;
        }
        dear[i] = d;
        river[i] = r;
    }
    let (rise, steep, ruling) = (pr.rise / CELL_M, pr.steep / CELL_M, CELL_M / pr.ruling);
    let mut best = vec![f32::INFINITY; n];
    let mut came = vec![u32::MAX; n];
    let mut heap = std::collections::BinaryHeap::new();
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
        for (sx, sy) in STEPS {
            let (vx, vy) = (ux + sx, uy + sy);
            if vx < 0 || vy < 0 || vx >= nx as isize || vy >= ny as isize {
                continue;
            }
            let v = vy as usize * nx + vx as usize;
            // a knight's move pays for the river between too
            let mid = (vy - sy / 2) as usize * nx + (vx - sx / 2) as usize;
            let len = ((sx * sx + sy * sy) as f32).sqrt() / f;
            let dh = (ground[v] - ground[u]).abs();
            let over = if mid != u && mid != v && river[mid] { RIVER * len } else { 0.0 };
            let step = len * (1.0 + 0.5 * (dear[u] + dear[v])) + over + rise * dh + steep * (dh - ruling * len).max(0.0);
            if cost + step < best[v] {
                best[v] = cost + step;
                came[v] = u as u32;
                heap.push(Reach(cost + step, v as u32));
            }
        }
    }
    if !best[goal].is_finite() {
        return Vec::new();
    }
    let mut path = vec![goal];
    while *path.last().unwrap() != start {
        path.push(came[*path.last().unwrap()] as usize);
    }
    path.reverse();
    let spot = |i: usize| ((x0 + (i % nx) as isize) as f32 + 0.5, (y0 + (i / nx) as isize) as f32 + 0.5);
    let mut pts: Vec<(f32, f32)> = path.iter().map(|&i| (spot(i).0 / f, spot(i).1 / f)).collect();
    let last = pts.len() - 1;
    pts[0] = a;
    pts[last] = b;
    // what the straight line between two points costs, read every half square
    let line = |p: (f32, f32), q: (f32, f32)| -> Option<f32> {
        let dist = (q.0 - p.0).hypot(q.1 - p.1);
        let k = ((dist * f * 2.0).ceil() as usize).max(1);
        let len = dist / k as f32;
        let (mut prev, mut run, mut total) = (square(p)?, 0.0f32, 0.0f32);
        for s in 1..=k {
            let t = s as f32 / k as f32;
            let v = square((p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t))?;
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
    // pulled taut: from each point on, straight to the furthest point that the straight
    // line reaches for no more than the way over the squares costs between them
    let mut taut = Vec::new();
    let mut i = 0;
    while i < last {
        let mut j = last;
        while j > i + 1 && !line(pts[i], pts[j]).is_some_and(|c| c <= (best[path[j]] - best[path[i]]) * (1.0 + SLACK) + 1e-4) {
            j -= 1;
        }
        if j < last {
            taut.push(pts[j]);
        }
        i = j;
    }
    taut
}

/// What the straight way between two spots costs a lane, read every square:
/// how a cell's place is chosen (`place`). The ground must be known.
fn straight_cost(st: &State, p: (f32, f32), q: (f32, f32)) -> f32 {
    let pr = price(LANE);
    let w = st.world.W;
    let (fw, fh) = (st.world.W * FINE, st.world.H * FINE);
    let f = FINE as f32;
    let dist = (q.0 - p.0).hypot(q.1 - p.1);
    let k = ((dist * f).ceil() as usize).max(1);
    let len = dist / k as f32;
    let at = |t: f32| {
        let (x, y) = (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t);
        square_at(&st.wc, w, ((x * f) as usize).min(fw - 1), ((y * f) as usize).min(fh - 1))
    };
    let mut prev = at(0.0);
    let mut total = 0.0;
    for s in 1..=k {
        let here = at(s as f32 / k as f32);
        let dh = (here.0 - prev.0).abs();
        let wet = 0.5 * (prev.1 + here.1) + if here.2 { RIVER } else { 0.0 };
        total += len * (1.0 + wet) + pr.rise / CELL_M * dh + pr.steep / CELL_M * (dh - CELL_M / pr.ruling * len).max(0.0);
        prev = here;
    }
    total
}

/// Spots tried along each side of a cell for its place, and how far from its edge they keep.
const TRY: usize = 9;
const INSET: f32 = 0.15;
/// What each cell of distance from the meeting place costs a place, against
/// the ways' costs: it keeps a dead end's place from sliding to its cell's edge.
const LEAN: f32 = 0.5;

/// Where the ways through cell c pass (`CourseBook::place`): a town's meeting
/// place; elsewhere the best spot of the cell for the ways that meet there,
/// `others` the places at their far ends.
fn choose_place(st: &mut State, c: usize, others: &[(f32, f32)]) -> (f32, f32) {
    let w = st.world.W;
    let meet = meeting(st, c);
    if st.urban.get(c).copied().unwrap_or(false) || others.is_empty() {
        return meet;
    }
    let (cx, cy) = ((c % w) as f32, (c / w) as f32);
    let (mut lo, mut hi) = (meet, meet);
    for o in others {
        (lo, hi) = ((lo.0.min(o.0), lo.1.min(o.1)), (hi.0.max(o.0), hi.1.max(o.1)));
    }
    know(st, (lo.0.min(cx), lo.1.min(cy)), (hi.0.max(cx + 1.0), hi.1.max(cy + 1.0)));
    let mut spots = vec![meet];
    for j in 0..TRY {
        for i in 0..TRY {
            let at = |k: usize| INSET + (1.0 - 2.0 * INSET) * k as f32 / (TRY - 1) as f32;
            spots.push((cx + at(i), cy + at(j)));
        }
    }
    let (fw, fh) = (st.world.W * FINE, st.world.H * FINE);
    let f = FINE as f32;
    let mut best = (f32::INFINITY, meet);
    for &p in &spots {
        let (m, e, r, _) = square_at(&st.wc, w, ((p.0 * f) as usize).min(fw - 1), ((p.1 * f) as usize).min(fh - 1));
        // not in water, nor on a river's band, nor on ground too low to build on
        if r || e >= LOW || m < DRY_M {
            continue;
        }
        // nor on a steep slope: above 1 in 12, priced as the squares price ground to build on
        let lf = st.world.landform.as_deref().unwrap();
        let l = OnLand { lf, w, h: st.world.H };
        let d = 0.5 / f;
        let grade = (crossings::metres_at(&l, p.0 + d, p.1) - crossings::metres_at(&l, p.0 - d, p.1)).hypot(crossings::metres_at(&l, p.0, p.1 + d) - crossings::metres_at(&l, p.0, p.1 - d)) / 125.0;
        if grade > 1.0 / 8.0 {
            continue;
        }
        let ways: f32 = others.iter().map(|&o| straight_cost(st, p, o)).sum();
        let cost = ways + e + (grade / 0.083).powi(2).min(4.0) * 0.1 + LEAN * (p.0 - meet.0).hypot(p.1 - meet.1);
        if cost < best.0 {
            best = (cost, p);
        }
    }
    best.1
}

fn meeting(st: &State, c: usize) -> (f32, f32) {
    let w = st.world.W;
    match st.xs.table.middles.get(c) {
        Some(&m) => m,
        None => ((c % w) as f32 + 0.5, (c / w) as f32 + 0.5),
    }
}

fn quant(p: (f32, f32)) -> [u16; 2] {
    [(p.0 * UNIT).round().clamp(0.0, u16::MAX as f32) as u16, (p.1 * UNIT).round().clamp(0.0, u16::MAX as f32) as u16]
}

// ---------------------------------------------------------------- the year

/// The link of square c in slot s, its other end, and the level of road on both.
fn ends(st: &State, c: usize, s: usize) -> Option<(usize, u8)> {
    let m = traffic::other_end(st.world.W, st.world.H, c, s)?;
    Some((m, st.road[c].min(3).min(st.road[m].min(3))))
}

/// The kind of road on link (c, m) this year.
fn kind_of(st: &State, c: usize, m: usize, lv: u8) -> u8 {
    let trunk = |x: usize| st.hw.trunk.get(x).is_some_and(|&t| t != crate::highways::NOT);
    if trunk(c) && trunk(m) {
        TRUNK
    } else if lv >= 3 {
        TURNPIKE
    } else {
        LANE
    }
}

/// The links `road_segments` joins (every two neighbouring squares of made
/// road, a diagonal left out where a pair of straight links joins the two).
fn joined(st: &State, c: usize, s: usize) -> bool {
    let (w, h) = (st.world.W as isize, st.world.H as isize);
    let made = |x: isize, y: isize| x >= 0 && y >= 0 && x < w && y < h && st.road[(y * w + x) as usize] >= 2;
    let (x, y) = ((c % st.world.W) as isize, (c / st.world.W) as isize);
    let (dx, dy) = traffic::SLOTS[s];
    !(dx != 0 && dy != 0 && (made(x + dx, y) || made(x, y + dy)))
}

/// As the run opens: the made roads that stand, as `road_segments` joins them,
/// are given their places and courses (dated the year before the first), and
/// the squares of the opening face them.
pub fn open(st: &mut State, C: &Content, P: &Params, year: i64) {
    if !active(st, P) {
        return;
    }
    let _ = C;
    start_book(st);
    let N = st.N;
    let mut links = Vec::new();
    for c in 0..N {
        for s in 0..4 {
            if let Some((_, lv)) = ends(st, c, s) {
                if lv >= 2 && joined(st, c, s) {
                    links.push(c * 4 + s);
                }
            }
        }
    }
    let y16 = (year - 1).clamp(i16::MIN as i64, i16::MAX as i64) as i16;
    find(st, &links, y16);
    for &l in &links {
        st.wc.drawn[l] = true;
    }
    lay_near(st);
}

/// The year's end, after the year's carting and before the squares: which
/// links are drawn, the places and courses of those drawn for the first time,
/// and what the squares face.
pub fn close_year(st: &mut State, C: &Content, P: &Params, year: i64) {
    if !active(st, P) {
        return;
    }
    if !st.wc.kept() {
        start_book(st);
    }
    let _ = C;
    wear(st);
    let N = st.N;
    let mut new = Vec::new();
    for c in 0..N {
        for s in 0..4 {
            let l = c * 4 + s;
            let Some((m, lv)) = ends(st, c, s) else {
                st.wc.drawn[l] = false;
                continue;
            };
            let state = {
                let on = |v: &[i32], x: usize| v.get(x).is_some_and(|&t| t != crate::highways::NOT);
                (on(&st.hw.trunk, c) && on(&st.hw.trunk, m)) || (on(&st.hw.bypass, c) && on(&st.hw.bypass, m))
            };
            let drawn = lv >= 2 && ((state && joined(st, c, s)) || st.wc.worn[l]);
            st.wc.drawn[l] = drawn;
            if drawn {
                if st.wc.first[l] == 0 {
                    st.wc.first[l] = year.clamp(i16::MIN as i64, i16::MAX as i64) as i16;
                }
                if !st.wc.courses.contains_key(&(l as u32)) {
                    new.push(l);
                }
            }
        }
    }
    find(st, &new, year.clamp(i16::MIN as i64, i16::MAX as i64) as i16);
    lay_near(st);
    tell(st);
}

/// The year's wear (the viewer's `network::wear_through`, for one more year).
fn wear(st: &mut State) {
    let n4 = st.N * 4;
    let y = st.wc.through;
    let need = WORN_YEARS.min(y as u32 + 1);
    let full = y + 1 >= WORN_WINDOW as usize;
    let mask = u32::MAX >> (32 - WORN_WINDOW);
    let kept = st.tf.goods.len() == n4;
    for l in 0..n4 {
        // the weight by cart, as the record's byte keeps it
        let w = if kept && st.tf.mode[l] & 3 == traffic::CART { traffic::byte_goods(traffic::goods_byte(st.tf.goods[l] as f64)) } else { 0.0 };
        let (wore, heavy) = (w >= WORN_WEIGHT, w >= WORN_HEAVY);
        let b = &mut st.wc;
        b.bits[l] = (b.bits[l] << 1) | wore as u32;
        b.heavy[l] = (b.heavy[l] << 1) | heavy as u32;
        let count = (b.bits[l] & mask).count_ones() + (HEAVY_COUNTS - 1) * (b.heavy[l] & mask).count_ones();
        if !b.worn[l] && count >= need {
            b.worn[l] = true;
        } else if b.worn[l] && full && count < WORN_FADE {
            b.worn[l] = false;
        }
    }
    st.wc.through += 1;
}

/// Places for the cells of `links` that have none, then a course for each link.
fn find(st: &mut State, links: &[usize], year: i16) {
    if links.is_empty() {
        return;
    }
    let N = st.N;
    // the ways that meet at each cell to be placed: the far ends of these links and of
    // the drawn links already there, at their places, or their meeting places if they have none yet
    let mut meet: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    let (w, h) = (st.world.W, st.world.H);
    let other = move |l: usize| traffic::other_end(w, h, l / 4, l % 4).unwrap();
    for &l in links {
        let (c, m) = (l / 4, other(l));
        for (x, y) in [(c, m), (m, c)] {
            if st.wc.placed[x] == 0 {
                meet.entry(x).or_default().push(y);
            }
        }
    }
    let placed_before: Vec<bool> = (0..N).map(|c| st.wc.placed[c] != 0).collect();
    for (&c, far) in &meet {
        let others: Vec<(f32, f32)> = far.iter().map(|&o| if placed_before[o] { CourseBook::at(st.wc.place[o]) } else { meeting(st, o) }).collect();
        let p = choose_place(st, c, &others);
        st.wc.place[c] = quant(p);
        st.wc.placed[c] = year;
        st.wc.new_places.push(c as u32);
        st.wc.found.0 += 1;
    }
    let town: Vec<bool> = st.urban.clone();
    for &l in links {
        let (c, m) = (l / 4, other(l));
        let (a, b) = (CourseBook::at(st.wc.place[c]), CourseBook::at(st.wc.place[m]));
        let lv = st.road[c].min(3).min(st.road[m].min(3));
        let kind = kind_of(st, c, m, lv);
        know(st, (a.0.min(b.0) - PAD, a.1.min(b.1) - PAD), (a.0.max(b.0) + PAD, a.1.max(b.1) + PAD));
        let mid = seek(st, a, b, kind, [c, m], &town);
        let mut pts = vec![st.wc.place[c]];
        pts.extend(mid.into_iter().map(quant));
        pts.push(st.wc.place[m]);
        pts.dedup();
        st.wc.courses.insert(l as u32, Course { year, kind, pts });
        st.wc.new_courses.push(l as u32);
        st.wc.found.1 += 1;
    }
}

/// What the squares face this year: each cell's straight lengths of the kept
/// courses that come within half a cell of it. A course whose link the carts
/// no longer wear is faced as well: a town draws such a road as its own street.
fn lay_near(st: &mut State) {
    let (w, h) = (st.world.W, st.world.H);
    let b = &mut st.wc;
    b.near.iter_mut().for_each(|v| v.clear());
    for course in b.courses.values() {
        for pair in course.pts.windows(2) {
            let (p, q) = (CourseBook::at(pair[0]), CourseBook::at(pair[1]));
            let (x0, x1) = ((p.0.min(q.0) - 0.5).floor().max(0.0) as usize, ((p.0.max(q.0) + 0.5).floor().max(0.0) as usize).min(w - 1));
            let (y0, y1) = ((p.1.min(q.1) - 0.5).floor().max(0.0) as usize, ((p.1.max(q.1) + 0.5).floor().max(0.0) as usize).min(h - 1));
            for y in y0..=y1 {
                for x in x0..=x1 {
                    b.near[y * w + x].push([p.0, p.1, q.0, q.1]);
                }
            }
        }
    }
}

// ---------------------------------------------------------------- the log

/// The log of the year's end: the places chosen and the courses found since
/// the last, and every link whose drawing changed.
fn tell(st: &mut State) {
    let b = &mut st.wc;
    b.new_places.sort_unstable();
    b.new_places.dedup();
    let places: Vec<(u32, [u16; 2], i16)> = b.new_places.iter().map(|&c| (c, b.place[c as usize], b.placed[c as usize])).collect();
    let mut links: BTreeMap<u32, (bool, Option<&Course>)> = BTreeMap::new();
    for &l in &b.new_courses {
        links.insert(l, (b.drawn[l as usize], b.courses.get(&l)));
    }
    for l in 0..b.drawn.len() {
        if b.drawn[l] != b.told_drawn[l] {
            links.entry(l as u32).or_insert((b.drawn[l], None));
        }
    }
    let year_links: Vec<(u32, bool, Option<Course>)> = links.into_iter().map(|(l, (d, c))| (l, d, c.cloned())).collect();
    b.log = encode(&places, &year_links);
    b.told_drawn.clone_from(&b.drawn);
    b.new_places.clear();
    b.new_courses.clear();
    b.told += 1;
}

/// Everything the book holds, as one year's log from nothing: for a live
/// game that has missed a year's end.
pub fn whole(st: &State) -> Vec<u8> {
    let b = &st.wc;
    let places: Vec<(u32, [u16; 2], i16)> = (0..b.placed.len()).filter(|&c| b.placed[c] != 0).map(|c| (c as u32, b.place[c], b.placed[c])).collect();
    let mut links: BTreeMap<u32, (bool, Option<Course>)> = BTreeMap::new();
    for (&l, c) in &b.courses {
        links.insert(l, (b.told_drawn[l as usize], Some(c.clone())));
    }
    for l in 0..b.told_drawn.len() {
        if b.told_drawn[l] {
            links.entry(l as u32).or_insert((true, None));
        }
    }
    let list: Vec<(u32, bool, Option<Course>)> = links.into_iter().map(|(l, (d, c))| (l, d, c)).collect();
    encode(&places, &list)
}

fn put_varint(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push((v as u8) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

fn get_varint(bytes: &[u8], i: &mut usize) -> Result<u64, String> {
    let mut v = 0u64;
    for shift in (0..64).step_by(7) {
        let Some(&b) = bytes.get(*i) else { return Err("the ways' log ends in the middle of a number".into()) };
        *i += 1;
        v |= ((b & 0x7f) as u64) << shift;
        if b < 0x80 {
            return Ok(v);
        }
    }
    Err("the ways' log has a number too long to be one".into())
}

fn zig(d: i64) -> u64 {
    if d >= 0 { 2 * d as u64 } else { (-2 * d - 1) as u64 }
}

fn zag(z: u64) -> i64 {
    if z % 2 == 0 { (z / 2) as i64 } else { -((z / 2) as i64) - 1 }
}

/// What the record's header says of the ways' log; a later layout gets a higher number.
pub const LOG_LAYOUT: u32 = 1;

/// A year's log in bytes, as `encode_year` lays it out.
fn encode(places: &[(u32, [u16; 2], i16)], links: &[(u32, bool, Option<Course>)]) -> Vec<u8> {
    let mut out = Vec::new();
    put_varint(&mut out, places.len() as u64);
    let mut prev = 0u32;
    for &(c, p, y) in places {
        put_varint(&mut out, (c - prev) as u64);
        prev = c;
        out.extend_from_slice(&p[0].to_le_bytes());
        out.extend_from_slice(&p[1].to_le_bytes());
        put_varint(&mut out, zig(y as i64));
    }
    put_varint(&mut out, links.len() as u64);
    let mut prev = 0u32;
    for (l, drawn, course) in links {
        put_varint(&mut out, (l - prev) as u64);
        prev = *l;
        out.push(*drawn as u8 | (course.is_some() as u8) << 1);
        if let Some(c) = course {
            put_varint(&mut out, zig(c.year as i64));
            out.push(c.kind);
            put_varint(&mut out, c.pts.len() as u64);
            let mut at = [0i64, 0i64];
            for p in &c.pts {
                put_varint(&mut out, zig(p[0] as i64 - at[0]));
                put_varint(&mut out, zig(p[1] as i64 - at[1]));
                at = [p[0] as i64, p[1] as i64];
            }
        }
    }
    out
}

/// One year of the ways' log, read back.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WaysYear {
    /// The cells placed: (cell, its place in `UNIT`s, the year it was chosen).
    pub places: Vec<(u32, [u16; 2], i16)>,
    /// The links whose drawing changed or whose course was found: (link
    /// `c * 4 + slot`, drawn from this year, its course where it was found).
    pub links: Vec<(u32, bool, Option<Course>)>,
}

/// A year's bytes of the ways' log (`way_log`): the number of cells placed;
/// for each, the cell less the one before (the first less 0), its place as two
/// little-endian `u16`s in `UNIT`s (1/256 of a cell), the year it was chosen,
/// zigzagged; then the number of links; for each, the link (`c * 4 + slot`,
/// the core's `traffic::SLOTS`) less the one before, a byte (1: drawn from this
/// year on; 2: its course follows), and where its course follows: the year it
/// was found, zigzagged, its kind (`KINDS`), its number of points, and each
/// point as the zigzagged step from the one before in `UNIT`s (the first from
/// nought). A course's first point is the place of the link's own cell `c`,
/// and its last that of its neighbour. Every number but the place's halves and
/// the two single bytes is a varint, seven bits a byte, low first.
pub fn encode_year(year: &WaysYear) -> Vec<u8> {
    encode(&year.places, &year.links)
}

/// A year of the ways' log read back (`encode_year`), or what is wrong with the bytes.
pub fn decode_year(bytes: &[u8]) -> Result<WaysYear, String> {
    let mut out = WaysYear::default();
    if bytes.is_empty() {
        return Ok(out);
    }
    let mut i = 0usize;
    let n = get_varint(bytes, &mut i)?;
    let mut c = 0u64;
    for _ in 0..n {
        c += get_varint(bytes, &mut i)?;
        let Some(b) = bytes.get(i..i + 4) else { return Err("the ways' log ends in the middle of a place".into()) };
        i += 4;
        let p = [u16::from_le_bytes([b[0], b[1]]), u16::from_le_bytes([b[2], b[3]])];
        let y = zag(get_varint(bytes, &mut i)?) as i16;
        out.places.push((c as u32, p, y));
    }
    let n = get_varint(bytes, &mut i)?;
    let mut l = 0u64;
    for _ in 0..n {
        l += get_varint(bytes, &mut i)?;
        let Some(&flags) = bytes.get(i) else { return Err("the ways' log ends in the middle of a link".into()) };
        i += 1;
        let course = if flags & 2 != 0 {
            let year = zag(get_varint(bytes, &mut i)?) as i16;
            let Some(&kind) = bytes.get(i) else { return Err("the ways' log ends in the middle of a course".into()) };
            i += 1;
            let k = get_varint(bytes, &mut i)?;
            let mut at = [0i64, 0i64];
            let mut pts = Vec::with_capacity(k as usize);
            for _ in 0..k {
                at[0] += zag(get_varint(bytes, &mut i)?);
                at[1] += zag(get_varint(bytes, &mut i)?);
                pts.push([at[0].clamp(0, u16::MAX as i64) as u16, at[1].clamp(0, u16::MAX as i64) as u16]);
            }
            Some(Course { year, kind, pts })
        } else {
            None
        };
        out.links.push((l as u32, flags & 1 != 0, course));
    }
    Ok(out)
}

// ---------------------------------------------------------------- measures

/// The share of the drawn courses' length within `deg` degrees of the eight
/// directions of the map (the measure of the lattice the roads' pieces used).
pub fn share_on_eight(st: &State, deg: f32) -> (f64, f64) {
    let b = &st.wc;
    let (mut on, mut all) = (0.0f64, 0.0f64);
    for (&l, c) in &b.courses {
        if !b.drawn[l as usize] {
            continue;
        }
        for pair in c.pts.windows(2) {
            let (p, q) = (CourseBook::at(pair[0]), CourseBook::at(pair[1]));
            let len = (q.0 - p.0).hypot(q.1 - p.1) as f64;
            if len < 1e-6 {
                continue;
            }
            let a = (q.1 - p.1).atan2(q.0 - p.0).to_degrees().rem_euclid(45.0);
            if a.min(45.0 - a) <= deg {
                on += len;
            }
            all += len;
        }
    }
    (on, all)
}

/// Distance in squares (125 m) from a spot in cells to the nearest kept
/// course about its cell, if any comes within half a cell of it.
pub fn to_course(st: &State, x: f32, y: f32) -> Option<f32> {
    let w = st.world.W;
    let c = (y.floor().max(0.0) as usize).min(st.world.H - 1) * w + (x.floor().max(0.0) as usize).min(w - 1);
    st.wc.near.get(c)?.iter().map(|s| seg_dist(x, y, s) * FINE as f32).reduce(f32::min)
}
