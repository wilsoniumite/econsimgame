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
pub const RULES: Rules = Rules { ruling: RULING, ease: 0.9, steep: 0.45, wind: 1.6, wind_steps: 4, high: 0.2, bank: 0.07, tunnel: 0.1 };
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
/// How deep under the ground a train is hidden.
const HIDDEN: f32 = 0.07;
/// How many points between the two cells of a length have a level settled for them.
const SUBS: usize = 3;
/// The first year a steep length is worked by a banking engine and not a rope.
const BANKING_FROM: i32 = 1850;
/// A curve of less than this radius is tighter than a train can take: the
/// trains are 0.16 long.
const TIGHT: f32 = 0.05;
/// The length of line a curve's radius is measured over.
const CURVE_OVER: f32 = 0.08;
/// The lie of the land bends a line no tighter than this radius.
const EASY: f32 = 0.07;
/// Sidings lie beside a line that curves no tighter than this.
const SIDING_CURVE: f32 = 0.3;
/// How far the ground under an engine house or a shed may rise over the
/// rails beside it (its walls stand that deep in the hillside), and how far
/// it may fall below them (it stands on a stone footing that high).
const STAND_SUNK: f32 = 0.012;
const STAND_FOOTING: f32 = 0.045;

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
    /// whether each point lies in the hills, where they build in stone
    pub upland: Vec<bool>,
}

/// The line at a cell: where it passes, the way it runs there, and its level.
#[derive(Clone, Copy, Debug)]
pub struct Stop {
    pub at: Vec2,
    pub dir: Vec2,
    pub level: f32,
    /// how many lengths of line meet here
    pub ways: usize,
    /// a town's cell (whether or not the town is there yet)
    pub town: bool,
}

/// Something that stands by the line: where (the level of the rails beside
/// it for its height), and the way it faces.
#[derive(Clone, Copy, Debug)]
pub struct Stand {
    pub what: Standing,
    pub at: Vec3,
    pub yaw: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Standing {
    WindingHouse,
    BankerShed,
    BufferStop,
}

pub struct RailYear {
    pub lines: Vec<Line>,
    /// track being laid: dashed, straight between cell centres
    pub building: Vec<(Vec2, Vec2)>,
    /// the line at each cell that has track
    pub stops: HashMap<usize, Stop>,
    /// what stands by the line: winding engine houses, banking engines' sheds, buffer stops
    pub stands: Vec<Stand>,
    /// the solid works and their smoke, made once, the first time they are asked for
    works: OnceLock<(Builder, Builder)>,
    /// the ground the railway takes
    room: OnceLock<Room>,
}

/// The ground the railway takes: the points of the lines above ground, the
/// stations and what stands by the line, each with the room it takes, in
/// squares a tenth of a cell across; and the cells of the map that hold any
/// of it, so that most of the island is answered at once.
struct Room {
    cells: HashSet<(i32, i32)>,
    spots: HashMap<(i32, i32), Vec<(f32, f32, f32)>>,
}

/// The furthest a spot of the railway's ground reaches.
const ROOM_REACH: f32 = 0.2;

impl RailYear {
    /// Whether a spot is on the railway or its banks, or within `extra` of them.
    pub fn near(&self, x: f32, z: f32, extra: f32) -> bool {
        let room = self.room.get_or_init(|| {
            let mut room = Room { cells: HashSet::new(), spots: HashMap::new() };
            let mut take = |x: f32, z: f32, reach: f32| {
                for (dx, dz) in [(-ROOM_REACH, -ROOM_REACH), (ROOM_REACH, -ROOM_REACH), (-ROOM_REACH, ROOM_REACH), (ROOM_REACH, ROOM_REACH)] {
                    room.cells.insert(((x + dx).floor() as i32, (z + dz).floor() as i32));
                }
                room.spots.entry(((x * 10.0).floor() as i32, (z * 10.0).floor() as i32)).or_default().push((x, z, reach));
            };
            for line in &self.lines {
                for (i, p) in line.pts.iter().enumerate() {
                    if line.kind[i] != Kind::Tunnel {
                        take(p.x, p.z, HALF + 0.01 + (1.2 * (p.y - line.ground[i])).clamp(0.0, 0.12));
                    }
                }
            }
            // a station and its sidings
            for stop in self.stops.values().filter(|stop| stop.town) {
                take(stop.at.x, stop.at.y, 0.1);
            }
            for st in self.stands.iter().filter(|st| st.what != Standing::BufferStop) {
                take(st.at.x, st.at.z, 0.075);
            }
            room
        });
        debug_assert!(extra < ROOM_REACH - 0.16);
        if !room.cells.contains(&(x.floor() as i32, z.floor() as i32)) {
            return false;
        }
        let (bx, bz) = ((x * 10.0).floor() as i32, (z * 10.0).floor() as i32);
        for dz in -2..=2 {
            for dx in -2..=2 {
                if let Some(v) = room.spots.get(&(bx + dx, bz + dz)) {
                    if v.iter().any(|&(px, pz, reach)| (px - x).hypot(pz - z) < reach + extra) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// What a works' yard must keep clear of besides the track itself: the
    /// middle of each town's station ground and of each engine house and shed,
    /// and how far to keep from it.
    pub fn grounds(&self) -> Vec<(Vec2, f32)> {
        let stations = self.stops.values().filter(|stop| stop.town).map(|stop| (stop.at, 0.14));
        let stands = self.stands.iter().filter(|st| st.what != Standing::BufferStop).map(|st| (Vec2::new(st.at.x, st.at.z), 0.085));
        stations.chain(stands).collect()
    }
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
    /// which way along each cell's line it leaves that cell (1 or -1)
    signs: (f32, f32),
    /// the year (from the start) it was laid
    opened: usize,
    len: f32,
    /// the levels settled for the points between its ends
    subs: [f32; SUBS],
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
        // a line that ends at a new cell comes straight in from where it passes the cell before
        for &c in fresh.iter().filter(|&&c| adj[c].len() == 1) {
            let from = self.cells[&adj[c][0]].at;
            let cell = self.cells.get_mut(&c).unwrap();
            cell.dir = through(&[(from - cell.at).normalize_or(-cell.dir)]);
        }
        // 2. a first course for each new length, by the lie of the land between its two cells
        // as they stand: straight where its length is enough for the climb, wound where it is
        // not, an incline where winding is not enough
        let squares: Vec<Vec2> = towns.iter().map(|&c| v2(cen[c])).collect();
        let first_level = |c: usize| if fresh_set.contains(&c) { self.cells[&c].ground } else { self.cells[&c].level };
        // each end of a new length leaves its cell along that cell's line, toward the other
        // cell. But a line runs through a cell and does not double back on itself there: where
        // a length would leave the way another length already does and nothing leaves the
        // other way, it takes the free way and comes round, if the cell has only the two (a
        // line that ended here and is carried on, or one laid again by another way) or if it
        // is going off to the side (a branch from the end of a line).
        let mut out: HashMap<usize, [bool; 2]> = HashMap::new();
        let mut leave = |c: usize, toward: usize| -> f32 {
            let cell = &self.cells[&c];
            let along = cell.dir.dot((self.cells[&toward].at - cell.at).normalize_or_zero());
            let taken = out.entry(c).or_insert_with(|| {
                let mut taken = [false; 2];
                for &m in &adj[c] {
                    if let Some(l) = self.lengths.get(&(c.min(m), c.max(m))) {
                        taken[((if l.a == c { l.signs.0 } else { l.signs.1 }) < 0.0) as usize] = true;
                    }
                }
                taken
            });
            let mut sign = if along >= 0.0 { 1.0 } else { -1.0 };
            if taken[(sign < 0.0) as usize] && !taken[(sign > 0.0) as usize] && (adj[c].len() == 2 || along.abs() < 0.5) {
                sign = -sign;
            }
            taken[(sign < 0.0) as usize] = true;
            sign
        };
        let signs: Vec<(f32, f32)> = new_links.iter().map(|&(a, b)| (leave(a, b), leave(b, a))).collect();
        let mut drafts: Vec<Draft> = new_links
            .iter()
            .zip(&signs)
            .map(|(&(a, b), &signs)| {
                let (ca, cb) = (self.cells[&a], self.cells[&b]);
                draft(g, a, b, &ca, &cb, signs, first_level(a), first_level(b), 0, &squares)
            })
            .collect();
        // 3. the levels of the new cells and of the points along the new lengths, among the
        // new and the settled cells they touch. Where the ground cannot be kept to, a length
        // is wound further or made an incline, laid again, and the levels settled again.
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
        let year = r.y0 + y as i32;
        let longest_tunnel = (SUBS + 1) * if year < 1840 { 1 } else if year < 1870 { 2 } else { 3 };
        let mut settled;
        let mut round = 0;
        // the lengths that winding did not bring down to the ground, which are inclines instead
        let mut steep = vec![false; drafts.len()];
        loop {
            let last = round >= 6;
            let mut nodes: Vec<profile::Node> = region
                .iter()
                .map(|&c| {
                    let cell = &self.cells[&c];
                    let old = !fresh_set.contains(&c);
                    // the line must clear the ground along the first of each new length too
                    let ground = drafts.iter().fold(cell.ground, |g, d| if d.a == c { g.max(d.end_ground[0]) } else if d.b == c { g.max(d.end_ground[1]) } else { g });
                    // a station stands on the ground, and a line ends on it
                    let grounded = cell.town || adj[c].len() < 2;
                    profile::Node {
                        ground: if old || grounded { cell.ground } else { ground },
                        fixed: if old { Some(cell.level) } else if grounded { Some(cell.ground) } else { None },
                        tunnel_ok: !cell.town && adj[c].len() == 2,
                    }
                })
                .collect();
            let mut edges: Vec<profile::Edge> = Vec::new();
            for &c in &region {
                for &m in &adj[c] {
                    if let (true, Some(&im), Some(old)) = (c < m, index.get(&m), self.lengths.get(&(c, m))) {
                        let len = self.cells[&c].at.distance(self.cells[&m].at) * 1.02;
                        edges.push(profile::Edge { a: index[&c], b: im, len, stage: old.stage, new: false, group: usize::MAX - edges.len(), top: old.stage });
                    }
                }
            }
            for (di, d) in drafts.iter().enumerate() {
                let first = nodes.len();
                for k in 0..SUBS {
                    nodes.push(profile::Node { ground: d.sub_ground[k], fixed: None, tunnel_ok: true });
                }
                let whole = self.cells[&d.a].at.distance(self.cells[&d.b].at) * 1.02;
                let len = whole / (SUBS + 1) as f32;
                // an incline only where the ground between the two cells is steeper than winding climbs
                let top = if steep[di] || (first_level(d.a) - first_level(d.b)).abs() > RULES.cap(whole, RULES.wind_steps) { RULES.incline() } else { RULES.wind_steps };
                for k in 0..=SUBS {
                    let a = if k == 0 { index[&d.a] } else { first + k - 1 };
                    let b = if k == SUBS { index[&d.b] } else { first + k };
                    edges.push(profile::Edge { a, b, len, stage: d.stage, new: true, group: di, top });
                }
            }
            settled = profile::settle(&nodes, &mut edges, &RULES, longest_tunnel, !last);
            // the lengths that were made to give are laid again as they now are
            let mut changed = false;
            let mut e = edges.iter().filter(|e| e.new);
            for (di, d) in drafts.iter_mut().enumerate() {
                let mut stage = e.next().map_or(d.stage, |e| e.stage);
                for _ in 0..SUBS {
                    e.next();
                }
                // a wound length that still stands in the air for most of its way is no winding
                // line on a viaduct: it goes straight up the hill, as an incline
                // and so does one that could not find the length it was to wind to
                let first = region.len() + di * SUBS;
                let over = (0..SUBS).filter(|&k| settled.level[first + k] - d.sub_ground[k] > VIADUCT).count();
                let short = stage == d.stage && d.len < 0.92 * self.cells[&d.a].at.distance(self.cells[&d.b].at) * 1.02 * RULES.wound(stage);
                if !last && stage > 0 && stage < RULES.incline() && (over * 2 > SUBS || short) {
                    steep[di] = true;
                    stage = RULES.incline();
                }
                if stage != d.stage {
                    changed = true;
                    let (ca, cb) = (self.cells[&d.a], self.cells[&d.b]);
                    let level = |c: usize| settled.level[index[&c]];
                    *d = draft(g, d.a, d.b, &ca, &cb, d.signs, level(d.a), level(d.b), stage, &squares);
                }
            }
            if !changed || last {
                break;
            }
            round += 1;
        }
        self.given_up += settled.given_up;
        // a length that was wound and then did not need the length it found (what stood in its
        // way could not be helped by winding) runs straight after all
        for (di, d) in drafts.iter_mut().enumerate() {
            if d.stage == 0 || d.stage >= RULES.incline() {
                continue;
            }
            let at = region.len() + di * SUBS;
            let mut levels = vec![settled.level[index[&d.a]]];
            levels.extend((0..SUBS).map(|k| settled.level[at + k]));
            levels.push(settled.level[index[&d.b]]);
            let climb = levels.windows(2).map(|w| (w[0] - w[1]).abs()).fold(0.0, f32::max);
            let part = self.cells[&d.a].at.distance(self.cells[&d.b].at) * 1.02 / (SUBS + 1) as f32;
            let stage = (0..d.stage).find(|&s| RULES.cap(part, s) >= climb - 1e-5).unwrap_or(d.stage);
            if stage != d.stage {
                let (ca, cb) = (self.cells[&d.a], self.cells[&d.b]);
                *d = draft(g, d.a, d.b, &ca, &cb, d.signs, levels[0], levels[SUBS + 1], stage, &squares);
            }
        }
        let mut under: HashSet<usize> = HashSet::new();
        for (i, &c) in region.iter().enumerate() {
            if fresh_set.contains(&c) {
                self.cells.get_mut(&c).unwrap().level = settled.level[i];
                if settled.under[i] {
                    under.insert(c);
                }
            }
        }
        // 4. the slope of the line at each new cell: level at a station, at the end of a line
        // and where an incline begins, and elsewhere what the points either side make it
        let mut slopes: Vec<(usize, f32)> = Vec::new();
        for &c in &fresh {
            let cell = self.cells[&c];
            let mut sides: [Option<(f32, f32, f32)>; 2] = [None, None];
            let mut flat = cell.town;
            for &m in &adj[c] {
                let other = self.cells[&m];
                let align = cell.dir.dot((other.at - cell.at).normalize_or_zero());
                // the nearest point along the length to that cell: its level, and how far off it is
                // and which way along the cell's line the length leaves
                let (level, dist, stage, sign) = match drafts.iter().position(|d| (d.a, d.b) == (c.min(m), c.max(m))) {
                    Some(di) => {
                        let d = &drafts[di];
                        let k = if d.a == c { 0 } else { SUBS - 1 };
                        (settled.level[region.len() + di * SUBS + k], d.len / (SUBS + 1) as f32, d.stage, if d.a == c { d.signs.0 } else { d.signs.1 })
                    }
                    None => match self.lengths.get(&(c.min(m), c.max(m))) {
                        Some(l) => (l.subs[if l.a == c { 0 } else { SUBS - 1 }], l.len / (SUBS + 1) as f32, l.stage, if l.a == c { l.signs.0 } else { l.signs.1 }),
                        None => continue,
                    },
                };
                flat |= stage >= RULES.incline();
                let side = &mut sides[(sign < 0.0) as usize];
                if side.map_or(true, |s| align.abs() > s.0) {
                    *side = Some((align.abs(), level - cell.level, dist));
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
        // 5. the profile of each new length along its course
        for (di, d) in drafts.into_iter().enumerate() {
            let (ca, cb) = (self.cells[&d.a], self.cells[&d.b]);
            let at = region.len() + di * SUBS;
            let subs: [f32; SUBS] = std::array::from_fn(|k| settled.level[at + k]);
            let mut holes: [bool; SUBS + 2] = [false; SUBS + 2];
            holes[0] = under.contains(&d.a);
            holes[SUBS + 1] = under.contains(&d.b);
            for k in 0..SUBS {
                holes[k + 1] = settled.under[at + k];
            }
            let key = (d.a, d.b);
            self.lengths.insert(key, Arc::new(finish(d, &ca, &cb, subs, holes, y)));
        }
    }

    /// Where the line passes a cell and the way it runs there, from the track
    /// about it the year it is laid.
    fn place(&self, g: &Ground, cen: &[(f32, f32)], adj: &[Vec<usize>], c: usize, town: bool) -> Cell {
        let mid = v2(cen[c]);
        let dirs: Vec<Vec2> = adj[c].iter().map(|&m| (v2(cen[m]) - mid).normalize_or(Vec2::X)).collect();
        let dir = through(&dirs);
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
        // the line clears the highest ground close about the spot, so that whichever way a
        // length leaves it, now or in years to come, it does not start under the ground
        let about = (0..8).map(|i| Vec2::from_angle(i as f32 * std::f32::consts::FRAC_PI_4) * 0.07).map(|o| world::height_at(g, at.x + o.x, at.y + o.y)).fold(0.0, f32::max);
        Cell { at, dir, ground: bed_ground(g, at, perp).max(about).max(0.03), level: 0.0, slope: 0.0, town }
    }
}

/// The way a line runs through a cell, from the ways to the cells it is
/// linked to: on from the one before where it ends, and elsewhere the
/// straightest way through that any two of them make.
fn through(dirs: &[Vec2]) -> Vec2 {
    match dirs {
        [] => Vec2::X,
        [only] => -*only,
        _ => {
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
    }
}

/// A new length of line as far as its course: where it runs, and the ground
/// along it.
struct Draft {
    a: usize,
    b: usize,
    stage: u8,
    /// which way along each cell's line it leaves that cell (1 or -1)
    signs: (f32, f32),
    pts: Vec<Vec2>,
    /// the distance along it to each point, and its whole length
    s: Vec<f32>,
    len: f32,
    /// the ground under each point, at its middle and under the width of the bed; the least the
    /// line may lie at there (the bed's ground, and clear of a river where it crosses one); and
    /// how near a river is
    ground: Vec<f32>,
    bed: Vec<f32>,
    floor: Vec<f32>,
    water: Vec<f32>,
    /// the highest ground about each of the points between its ends whose level is settled, and by each end
    sub_ground: [f32; SUBS],
    end_ground: [f32; 2],
}

/// Choose the course of a length of line from cell `a` to cell `b`, whose
/// levels are `la` and `lb` for now. `signs` is which way along each cell's
/// line the length leaves it.
#[allow(clippy::too_many_arguments)]
fn draft(g: &Ground, a_cell: usize, b_cell: usize, a: &Cell, b: &Cell, signs: (f32, f32), la: f32, lb: f32, stage: u8, squares: &[Vec2]) -> Draft {
    let chord = a.at.distance(b.at).max(1e-3);
    let (sa, sb) = signs;
    let ends = course::Ends { a: a.at, da: a.dir * sa, b: b.at, db: -b.dir * sb };
    let incline = stage >= RULES.incline();
    let near: Vec<Vec2> = squares.iter().copied().filter(|s| s.distance(a.at) < 2.0).collect();
    // what a point of the course costs: how far the ground there is from the line's level (a cut
    // more than a fill: the ground cannot be dug), a river under it, a market place in its way
    let cost = |p: Vec2, t: f32| -> f32 {
        let dev = world::height_at(g, p.x, p.y).max(0.0) - (la + (lb - la) * t);
        // (and once the line is up on a viaduct, a little less height is not worth a bend)
        let mut c = if dev > 0.0 { 3.0 * dev.min(0.3) } else { (-dev).min(0.05) + 0.2 * (-dev - 0.05).clamp(0.0, 0.25) };
        if world::water_dist(g, p.x, p.y) < 0.02 {
            c += 0.06;
        }
        if near.iter().any(|sq| sq.distance(p) < 0.13) {
            c += 0.6;
        }
        c
    };
    // a length that must find length swings from side to side as far as it needs to
    let want = chord * 1.02 * RULES.wound(stage);
    let scale = chord.min(1.2);
    let mut swing = Swing::default();
    if stage > 0 && !incline {
        let mut best = f32::MAX;
        for ess in [-0.4f32, -0.3, -0.2, -0.1, 0.0, 0.1, 0.2, 0.3, 0.4] {
            for wave in [-0.2f32, -0.1, 0.0, 0.1, 0.2] {
                let sw = Swing { bow: 0.0, ess: ess * scale, wave: wave * scale };
                let pts = course::curve(&ends, sw, &[], 48);
                let len = course::length(&pts);
                let fit: f32 = pts.iter().enumerate().map(|(i, p)| cost(*p, i as f32 / 48.0)).sum::<f32>() / 49.0;
                // a winding tighter than a train can take is no way to find length
                let hooked = 4.0 * (1.0 - course::tightest(&pts, CURVE_OVER) / TIGHT).max(0.0);
                let c = fit + hooked + 2.0 * (want - len).max(0.0) / chord + 0.15 * (len - want).max(0.0) / chord + 0.02 * (ess.abs() + wave.abs());
                if c < best {
                    best = c;
                    swing = sw;
                }
            }
        }
    }
    // and every length lies to one side or the other of that as the land lies
    let mut reach = if swing == Swing::default() { 0.3 * scale } else { 0.12 * scale };
    let mut lie = course::seek(&ends, swing, 5, reach, &cost);
    // but not so as to bend it tighter than a train takes easily (or than its winding does):
    // a shift to the inside of a curve tightens it, and far enough makes a hook of it
    let easy = EASY.min(course::tightest(&course::curve(&ends, swing, &[], 48), CURVE_OVER));
    while course::tightest(&course::curve(&ends, swing, &lie, 48), CURVE_OVER) < easy - 1e-4 {
        reach *= 0.5;
        lie = course::seek(&ends, swing, 5, if reach < 0.03 { 0.0 } else { reach }, &cost);
    }
    let rough = course::length(&course::curve(&ends, swing, &lie, 24));
    let n = ((rough / STEP).ceil() as usize).max(2 * (SUBS + 1));
    let pts = course::curve(&ends, swing, &lie, n);
    let s = course::along(&pts);
    let len = s[n].max(1e-3);
    let side = |i: usize| -> Vec2 {
        let d = (pts[(i + 1).min(n)] - pts[i.saturating_sub(1)]).normalize_or(ends.da);
        Vec2::new(-d.y, d.x)
    };
    let ground: Vec<f32> = pts.iter().map(|p| world::height_at(g, p.x, p.y).max(0.0)).collect();
    let water: Vec<f32> = pts.iter().map(|p| world::water_dist(g, p.x, p.y)).collect();
    // the ground under the bed, and clear of a river where it crosses one
    let bed: Vec<f32> = (0..=n).map(|i| bed_ground(g, pts[i], side(i))).collect();
    let floor: Vec<f32> = (0..=n).map(|i| bed[i] + if water[i] < 0.03 { 0.05 } else { 0.0 }).collect();
    let highest = |from: f32, to: f32| -> f32 { (0..=n).filter(|&i| s[i] >= from * len - 1e-4 && s[i] <= to * len + 1e-4).map(|i| floor[i]).fold(0.0, f32::max) };
    let part = 1.0 / (SUBS + 1) as f32;
    let sub_ground = std::array::from_fn(|k| highest((k as f32 + 0.5) * part, (k as f32 + 1.5) * part));
    let end_ground = [highest(0.0, 0.5 * part), highest(1.0 - 0.5 * part, 1.0)];
    Draft { a: a_cell, b: b_cell, stage, signs: (sa, sb), pts, s, len, ground, bed, floor, water, sub_ground, end_ground }
}

/// Give a drafted length its level: a smooth curve through the levels
/// settled for its two cells and the points between, lifted over what ground
/// stands above it, and under the ground only where `holes` says a tunnel is
/// (at each of the settled points, the two cells among them).
fn finish(d: Draft, a: &Cell, b: &Cell, subs: [f32; SUBS], holes: [bool; SUBS + 2], opened: usize) -> Length {
    let Draft { pts, s, len, ground, bed, floor, water, .. } = d;
    let n = pts.len() - 1;
    let incline = d.stage >= RULES.incline();
    // the settled points: how far along each is, its level, and the slope there
    let m = SUBS + 1;
    let at: Vec<f32> = (0..=m).map(|k| len * k as f32 / m as f32).collect();
    let mut h = vec![a.level];
    h.extend(subs);
    h.push(b.level);
    let lim = if incline { f32::MAX } else { 0.9 * RULING };
    let slope: Vec<f32> = (0..=m)
        .map(|k| if k == 0 { d.signs.0 * a.slope } else if k == m { -d.signs.1 * b.slope } else { ((h[k + 1] - h[k - 1]) / (at[k + 1] - at[k - 1])).clamp(-lim, lim) })
        .collect();
    let curve = |x: f32, straight: bool| -> f32 {
        let k = ((x / len * m as f32).floor() as usize).min(m - 1);
        let (span, u) = (at[k + 1] - at[k], ((x - at[k]) / (at[k + 1] - at[k])).clamp(0.0, 1.0));
        if straight { h[k] + (h[k + 1] - h[k]) * u } else { profile::hermite(h[k], h[k + 1], slope[k] * span, slope[k + 1] * span, u) }
    };
    let mut smooth: Vec<f32> = s.iter().map(|&x| curve(x, false)).collect();
    let steepest = (1..=n).map(|i| (smooth[i] - smooth[i - 1]).abs() / (s[i] - s[i - 1]).max(1e-6)).fold(0.0, f32::max);
    if !incline && steepest > RULING {
        // straight from one settled level to the next, which is within the gradient
        smooth = s.iter().map(|&x| curve(x, true)).collect();
    }
    let grad = if incline { (0..m).map(|k| 1.6 * (h[k + 1] - h[k]).abs() / (at[k + 1] - at[k])).fold(RULING, f32::max) } else { RULING };
    // a tunnel about each settled point that is to be in one: from where the hill is deepest
    // over the line there, out to where the line comes to the surface. (The hill is the ground
    // itself: the room a river asks for over its water is no hill, and is never bored.)
    let mut under = vec![false; n + 1];
    for k in (0..=m).filter(|&k| holes[k]) {
        let (lo, hi) = (at[k] - 0.5 * len / m as f32, at[k] + 0.5 * len / m as f32);
        let Some(deep) = (0..=n).filter(|&i| s[i] >= lo && s[i] <= hi).max_by(|&i, &j| (bed[i] - smooth[i]).total_cmp(&(bed[j] - smooth[j]))) else { continue };
        if bed[deep] - smooth[deep] <= HIDDEN {
            continue;
        }
        let (mut i, mut j) = (deep, deep);
        while i > 0 && bed[i - 1] - smooth[i - 1] > 0.012 {
            i -= 1;
        }
        while j < n && bed[j + 1] - smooth[j + 1] > 0.012 {
            j += 1;
        }
        under[i..=j].fill(true);
    }
    // elsewhere the line lies on the ground or over it
    let mut least: Vec<f32> = (0..=n).map(|i| if under[i] { smooth[i] } else { floor[i].max(smooth[i]) }).collect();
    for i in 0..=n {
        // the two ends are where their cells have them, and the line can only climb so fast from there
        let reach = (a.level + grad * s[i]).min(b.level + grad * (len - s[i]));
        if least[i] > reach {
            least[i] = reach;
            // ground that the line cannot get over is roofed (a river it cannot get so far
            // over is crossed lower)
            if bed[i] - reach > 0.006 {
                under[i] = true;
            }
        }
    }
    least[0] = a.level;
    least[n] = b.level;
    // a point or two just under the grass is not worth an arch
    let mut i = 0;
    while i <= n {
        let mut j = i;
        while j < n && under[j + 1] == under[i] {
            j += 1;
        }
        if under[i] && j - i < 2 && (i..=j).all(|q| bed[q] - least[q] < LIFT) {
            under[i..=j].fill(false);
        }
        i = j + 1;
    }
    let level = profile::raise(&s, &least, grad);
    let wet = water.iter().map(|&w| w < 0.012).collect();
    Length { a: d.a, stage: d.stage, signs: d.signs, opened, len, subs, pts, level, ground, wet, under }
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
        let upland = pts.iter().map(|p| r.terrain[(p.z.floor().max(0.0) as usize).min(r.h - 1) * r.w + (p.x.floor().max(0.0) as usize).min(r.w - 1)] >= 2).collect();
        lines.push(Line { pts, ground, kind, worked, upland });
    }
    let stops = (0..r.n)
        .filter(|&c| r.trans[k * r.n + c] & 4 != 0)
        .filter_map(|c| net.cells.get(&c).map(|cell| (c, Stop { at: cell.at, dir: cell.dir, level: cell.level, ways: adj[c].len(), town: cell.town })))
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
    let stands = stands(&lines, &stops, g);
    RailYear { lines, building, stops, stands, works: OnceLock::new(), room: OnceLock::new() }
}

/// What stands by the lines: a winding engine house by the head of each
/// inclined plane, a banking engine's shed by the foot of each steep bank,
/// and a buffer stop at each dead end. A house or a shed stands on ground of
/// its own beside the line: clear of every line, dry, where the line runs on
/// the ground or a low bank and the ground lies near the level of the rails
/// (a little under it is made up with a stone footing). It goes to the
/// first such place, working along the incline from that end, on whichever
/// side the ground lies nearer; an incline with no such place has none.
fn stands(lines: &[Line], stops: &HashMap<usize, Stop>, g: &Ground) -> Vec<Stand> {
    use isostate_gen::rail;
    // the track above ground, in squares a tenth of a cell across
    let square = |p: Vec2| ((p.x * 10.0).floor() as i32, (p.y * 10.0).floor() as i32);
    let mut track: HashMap<(i32, i32), Vec<Vec2>> = HashMap::new();
    for line in lines {
        for (p, _) in line.pts.iter().zip(&line.kind).filter(|(_, k)| **k != Kind::Tunnel) {
            let p = Vec2::new(p.x, p.z);
            track.entry(square(p)).or_default().push(p);
        }
    }
    let fits = |st: &Stand| -> bool {
        let (bx, bz) = square(Vec2::new(st.at.x, st.at.z));
        let on_track = (-1..=1).any(|dz| (-1..=1).any(|dx| track.get(&(bx + dx, bz + dz)).is_some_and(|v| v.iter().any(|&p| stand_holds(st, p, 0.8 * HALF)))));
        let (lo, hi) = stand_ground(g, st);
        !on_track && lo > 0.03 && hi - st.at.y <= STAND_SUNK && st.at.y - lo <= STAND_FOOTING && world::water_dist(g, st.at.x, st.at.z) > 0.06
    };
    let mut out = Vec::new();
    for line in lines {
        let n = line.pts.len();
        for (i, j) in runs_of(n, |q| line.worked[q] != Worked::Plain) {
            if j - i < 12 {
                continue;
            }
            let rope = line.worked[i] == Worked::Rope;
            let what = if rope { Standing::WindingHouse } else { Standing::BankerShed };
            let back = stand_size(what).1 * 0.5 + rail::TRACK_CLEAR;
            // from a little way in from the head of an incline, or from the foot of a bank
            let up = line.pts[j].y > line.pts[i].y;
            let from_end = rope == up;
            let found = (6..(j - i) / 2 + 6).map(|d| if from_end { j - d } else { i + d }).filter(|&q| q > i && q < j).find_map(|q| {
                if !matches!(line.kind[q], Kind::Grade | Kind::Embankment) {
                    return None;
                }
                let p = line.pts[q];
                let across = side_of(line.pts[q - 1], line.pts[q + 1]);
                // its front to the track
                let stand = |side: f32| Stand { what, at: p + across * (back * side), yaw: (-across.x * side).atan2(-across.z * side) };
                let lie = |st: &Stand| (world::height_at(g, st.at.x, st.at.z) - p.y).abs();
                let (a, b) = (stand(1.0), stand(-1.0));
                let (first, second) = if lie(&a) <= lie(&b) { (a, b) } else { (b, a) };
                [first, second].into_iter().find(|st| fits(st))
            });
            out.extend(found);
        }
        for end in [0, n - 1] {
            let p = line.pts[end];
            let dead = stops.values().any(|s| s.ways == 1 && s.at.distance(Vec2::new(p.x, p.z)) < 1e-3);
            if dead && n > 2 && line.kind[end] != Kind::Tunnel {
                let inner = line.pts[if end == 0 { 1 } else { n - 2 }];
                let dir = Vec3::new(p.x - inner.x, 0.0, p.z - inner.z).normalize_or(Vec3::X);
                out.push(Stand { what: Standing::BufferStop, at: p + Vec3::Y * LIFT, yaw: (-dir.z).atan2(dir.x) });
            }
        }
    }
    out
}

/// The ground an engine house or a shed keeps: along the track, and back from it.
fn stand_size(what: Standing) -> (f32, f32) {
    use isostate_gen::rail;
    match what {
        Standing::WindingHouse => rail::WINDING_HOUSE,
        Standing::BankerShed => rail::BANKER_SHED,
        Standing::BufferStop => (0.0, 0.0),
    }
}

/// Whether a spot lies on the ground of something that stands by the line, or within `extra` of it.
fn stand_holds(st: &Stand, p: Vec2, extra: f32) -> bool {
    let (w, d) = stand_size(st.what);
    let rel = Quat::from_rotation_y(-st.yaw) * Vec3::new(p.x - st.at.x, 0.0, p.y - st.at.z);
    rel.x.abs() < 0.5 * w + extra && rel.z.abs() < 0.5 * d + extra
}

/// The lowest and the highest ground under the four corners and the middle of what stands by the line.
fn stand_ground(g: &Ground, st: &Stand) -> (f32, f32) {
    let (w, d) = stand_size(st.what);
    let q = Quat::from_rotation_y(st.yaw);
    [(0.0, 0.0), (0.5, 0.5), (0.5, -0.5), (-0.5, 0.5), (-0.5, -0.5)].into_iter().fold((f32::MAX, f32::MIN), |(lo, hi), (fx, fz)| {
        let p = st.at + q * Vec3::new(fx * w, 0.0, fz * d);
        let h = world::height_at(g, p.x, p.z);
        (lo.min(h), hi.max(h))
    })
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

/// A place for sidings: the cell, how many, and the middle of the cell, which
/// they lie on the far side of the line from (a station stands on the near side).
pub type Sidings = (usize, usize, Vec2);

/// The track: a bed of ballast and a pair of dark rails, on the line's own
/// level everywhere it can be seen; a rope between the rails of an inclined
/// plane; a siding or two beside the line where `sidings` asks, each leaving
/// the line and coming back to it; dashed earthworks where track is being laid.
pub fn track(ry: &RailYear, g: &Ground, lk: &Look, sidings: &[Sidings]) -> Builder {
    let mut b = Builder::default();
    let (ballast, iron, build) = (*lk.rail.ballast, *lk.rail.rails, *lk.rail.earthworks);
    for &(cell, count, middle) in sidings {
        let Some(stop) = ry.stops.get(&cell) else { continue };
        // the line through the cell, and a length of it either side of where it passes
        let Some((line, i)) = ry.lines.iter().find_map(|l| l.pts.iter().position(|p| Vec2::new(p.x, p.z).distance(stop.at) < 1e-3).map(|i| (l, i))) else { continue };
        let n = line.pts.len();
        // only where the line is on the ground the whole way and runs straight enough for a
        // loop beside it: there, or a little way along from there
        let stretch = [0isize, -5, 5, -10, 10].into_iter().find_map(|shift| {
            let at = (i as isize + shift).clamp(0, n as isize - 1) as usize;
            let (lo, hi) = (at.saturating_sub(9), (at + 9).min(n - 1));
            let on_the_ground = (lo..=hi).all(|q| matches!(line.kind[q], Kind::Grade | Kind::Embankment) && line.pts[q].y - line.ground[q] <= 0.012);
            let flat: Vec<Vec2> = line.pts[lo..=hi].iter().map(|p| Vec2::new(p.x, p.z)).collect();
            (hi - lo >= 9 && on_the_ground && course::tightest(&flat, 2.0 * CURVE_OVER) >= SIDING_CURVE).then_some((lo, hi))
        });
        let Some((lo, hi)) = stretch else { continue };
        let across = |q: usize| side_of(line.pts[q.saturating_sub(1)], line.pts[(q + 1).min(n - 1)]);
        let a = across(i);
        let side = if Vec2::new(a.x, a.z).dot(middle - stop.at) > 0.0 { -1.0 } else { 1.0 };
        for j in 1..=count {
            let bed: Vec<Vec3> = (lo..=hi)
                .map(|q| {
                    let u = (q - lo) as f32 / (hi - lo) as f32;
                    let ease = crate::fields::smoothstep(0.0, 0.3, u) * crate::fields::smoothstep(0.0, 0.3, 1.0 - u);
                    line.pts[q] + across(q) * (side * 0.034 * j as f32 * ease) + Vec3::Y * LIFT
                })
                .collect();
            b.ribbon(&bed, 0.04, ballast);
            let top: Vec<Vec3> = bed.iter().map(|p| *p + Vec3::Y * 0.004).collect();
            b.ribbon(&top, 0.016, iron);
        }
    }
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

/// The solid works: banks, viaducts and bridges, the portals of the tunnels
/// and what stands by the line; and the smoke of the winding engines. They
/// are kept with the lines, in the look they were first asked for in: the
/// look does not change in a run.
pub fn structures(ry: &RailYear, g: &Ground, lk: &Look) -> (Builder, Builder) {
    ry.works.get_or_init(|| world::timed("year rail works", || build_structures(ry, g, lk))).clone()
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

fn build_structures(ry: &RailYear, g: &Ground, lk: &Look) -> (Builder, Builder) {
    let mut b = Builder::default();
    let mut smoke = Builder::default();
    let grass = *lk.rail.bank_grass;
    let grass_dark = *lk.rail.bank_grass_shade;
    let stone = *lk.rail.stone;
    let brick = *lk.rail.viaduct_brick;
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    for line in &ry.lines {
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
                        foot = p + s * ((HALF + (1.2 * drop).min(0.12) + 0.01) * side);
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
                // and on the high fells it is of broken rock, where no grass grows
                let high = ((p0.y - 1.2) / 0.5).clamp(0.0, 1.0);
                let lit = (side > 0.0) == ((s0.x - s0.z) > 0.0);
                let col = if sea_wall {
                    if side > 0.0 { stone } else { shade(stone, 0.85) }
                } else {
                    crate::meshkit::lerp(if lit { grass } else { grass_dark }, if lit { *lk.rail.bank_scree } else { shade(*lk.rail.bank_scree, 0.86) }, high)
                };
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
        // viaducts: piers and arches, a parapet along the top, an abutment at each end; of stone
        // in the hills and of brick in the low country, as the houses there are
        for (i, j) in runs_of(n, |q| line.kind[q] == Kind::Viaduct) {
            viaduct(&mut b, line, i, j, g, if line.upland[(i + j) / 2] { stone } else { brick });
        }
        // an iron girder bridge over each river, from bank to bank
        for (i, j) in runs_of(n, |q| line.kind[q] == Kind::Bridge) {
            let (a, c) = (line.pts[i.saturating_sub(1)], line.pts[(j + 1).min(n - 1)]);
            girder_bridge(&mut b, lk, a, c, g);
        }
        // tunnels: a portal where the line goes into the hill, and a roof over the line from
        // there until the hill is high enough to hide a train. A rise the line only just goes
        // under is a short stone arch over it, with no more of a portal than its two faces.
        for (i, j) in runs_of(n, |q| line.kind[q] == Kind::Tunnel) {
            let short = (i..j).map(|q| line.pts[q].distance(line.pts[q + 1])).sum::<f32>() < 0.15;
            let (i, j) = if short { (i.saturating_sub(1), (j + 1).min(n - 1)) } else { (i, j) };
            for (inside, outside) in [(i, i.checked_sub(1)), (j, (j + 1 < n).then_some(j + 1))] {
                let Some(outside) = outside else { continue };
                let (p, q) = (line.pts[outside], line.pts[inside]);
                let out_dir = Vec3::new(p.x - q.x, 0.0, p.z - q.z).normalize_or(Vec3::X);
                portal(&mut b, q, out_dir, stone, *lk.rail.tunnel_mouth, !short);
            }
            let shallow = |q: usize| short || line.ground[q] - line.pts[q].y < HIDDEN + 0.01;
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
                    let turf = if short { shade(stone, 1.05) } else if (side > 0.0) == ((s0.x - s0.z) > 0.0) { grass } else { grass_dark };
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
    // what stands by the line
    for st in &ry.stands {
        use isostate_gen::{rail, Site};
        // an engine house or a shed on ground that falls away stands on a stone footing
        if st.what != Standing::BufferStop {
            let (w, d) = stand_size(st.what);
            let low = stand_ground(g, st).0;
            if st.at.y - low > 0.004 {
                b.block(Vec3::new(st.at.x, low - 0.01, st.at.z), w + 0.006, d + 0.006, st.at.y - low + 0.01, st.yaw, shade(stone, 0.92), Some(*lk.rail.bank_top), 0.0);
            }
        }
        match st.what {
            Standing::WindingHouse => rail::winding_house(&mut b, &mut smoke, &Site::level(st.at, st.yaw), lk),
            Standing::BankerShed => rail::banker_shed(&mut b, &Site::level(st.at, st.yaw), lk),
            Standing::BufferStop => rail::buffer_stop(&mut b, &Site::level(st.at, st.yaw), lk),
        }
    }
    (b, smoke)
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

/// A tunnel portal: a stone face across the line with a dark arch, and (with
/// `wings`) wing walls stepping back into the hillside.
fn portal(b: &mut Builder, p: Vec3, out: Vec3, stone: [f32; 4], dark: [f32; 4], wings: bool) {
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
    for side in [-1.0f32, 1.0].into_iter().filter(|_| wings) {
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
    /// the tightest curve of any line, as one over its radius, and how many curves are
    /// tighter than a train can take (`TIGHT`)
    pub curve: (f32, f32, f32),
    pub hooks: usize,
    /// the furthest the track stands over the ground where it is drawn as a tunnel
    pub tunnel_in_air: (f32, f32, f32),
    /// what stands by the line (engine houses and sheds): how many, the furthest the ground
    /// under one lies from the rails beside it, and how many stand on a line
    pub stands: usize,
    pub stand_off: (f32, f32, f32),
    pub stands_on_track: usize,
}

/// Walk every line of a year and measure how well it holds together.
pub fn check(ry: &RailYear, g: &Ground) -> Checked {
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
            } else {
                worst(&mut c.tunnel_in_air, p.y - line.ground[i], p);
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
        let flat: Vec<Vec2> = line.pts.iter().map(|p| Vec2::new(p.x, p.z)).collect();
        let radii = course::radii(&flat, CURVE_OVER);
        for (i, &r) in radii.iter().enumerate() {
            worst(&mut c.curve, 1.0 / r, line.pts[i]);
        }
        c.hooks += runs_of(n, |q| radii[q] < TIGHT).len();
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
    for st in ry.stands.iter().filter(|st| st.what != Standing::BufferStop) {
        c.stands += 1;
        let (lo, hi) = stand_ground(g, st);
        worst(&mut c.stand_off, (st.at.y - lo).max(hi - st.at.y), st.at);
        if ry.lines.iter().any(|line| line.pts.iter().zip(&line.kind).any(|(p, k)| *k != Kind::Tunnel && stand_holds(st, Vec2::new(p.x, p.z), 0.8 * HALF))) {
            c.stands_on_track += 1;
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
        let c = check(&ry, g);
        let at = |s: (f32, f32, f32)| format!("{:.3} at {:.1},{:.1}", s.0, s.1, s.2);
        eprintln!(
            "rail check {}: {} lines, {:.0} km; step along a line {}; step where lines meet {}; steepest {}; steepest incline {}; highest over the ground {}; deepest under it outside a tunnel {}; trains off the track's level {}; a dead end off the ground {}; {} viaducts ({:.1} cells), {} tunnels ({:.1} cells, {:.1} of them roofed, the longest {}; in a tunnel yet over the ground {}), {} inclines and banks ({:.1} cells); the tightest curve has a radius of {:.3} at {:.1},{:.1}, {} curves tighter than {TIGHT}; {} engine houses and sheds, {} of them on a line, the ground furthest from the rails {}",
            r.y0 + k as i32, c.lines, c.km, at(c.step), at(c.meet), at(c.steepest), at(c.steepest_incline), at(c.tallest), at(c.buried), at(c.train_off), at(c.end_off), c.viaducts, c.viaduct_cells, c.tunnels, c.tunnel_cells, c.roofed_cells, at(c.longest_tunnel), at(c.tunnel_in_air), c.inclines, c.incline_cells, 1.0 / c.curve.0.max(1e-6), c.curve.1, c.curve.2, c.hooks, c.stands, c.stands_on_track, at(c.stand_off)
        );
    }
    if let Some(net) = NET.lock().unwrap().as_ref() {
        let wound = net.lengths.values().filter(|l| l.stage > 0 && l.stage < RULES.incline()).count();
        let inclines = net.lengths.values().filter(|l| l.stage >= RULES.incline()).count();
        eprintln!("rail check: {} cells and {} lengths settled in {:.0} ms; {} lengths wound to find length, {} inclines, {} cells where nothing more could give", net.cells.len(), net.lengths.len(), t0.elapsed().as_secs_f64() * 1000.0, wound, inclines, net.given_up);
    }
    if how == "list" {
        let ry = year(r, g, &cen, last);
        for st in ry.stands.iter().filter(|st| st.what != Standing::BufferStop) {
            eprintln!("{:?} at {:.2},{:.2}, the ground {:.3} to {:.3} under its floor", st.what, st.at.x, st.at.z, st.at.y - stand_ground(g, st).1, st.at.y - stand_ground(g, st).0);
        }
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
            let flat: Vec<Vec2> = line.pts.iter().map(|p| Vec2::new(p.x, p.z)).collect();
            let radii = course::radii(&flat, CURVE_OVER);
            for (i, j) in runs_of(n, |q| radii[q] < TIGHT) {
                let q = (i..=j).min_by(|&a, &b| radii[a].total_cmp(&radii[b])).unwrap_or(i);
                eprintln!("line {li}: tight curve at {:.2},{:.2}, radius {:.3}", line.pts[q].x, line.pts[q].z, radii[q]);
            }
        }
    }
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A straight line of points 0.02 apart at these heights over ground at nought.
    fn kinds(over: &[f32], wet: &[usize], under: &[usize]) -> Vec<Kind> {
        let pts: Vec<Vec3> = over.iter().enumerate().map(|(i, &h)| Vec3::new(i as f32 * 0.02, h, 0.0)).collect();
        let flag = |at: &[usize]| (0..over.len()).map(|i| at.contains(&i)).collect::<Vec<bool>>();
        classify(&pts, &vec![0.0; over.len()], &flag(wet), &flag(under))
    }

    #[test]
    fn what_a_line_stands_on_follows_its_height_over_the_ground() {
        // on the ground, then a bank, then high enough for arches for long enough
        let mut over = vec![0.0; 4];
        over.extend([0.03; 4]);
        over.extend([0.1; 12]);
        let k = kinds(&over, &[], &[]);
        assert!(k[..4].iter().all(|&x| x == Kind::Grade) && k[4..8].iter().all(|&x| x == Kind::Embankment) && k[8..].iter().all(|&x| x == Kind::Viaduct));
        // a few points of height are a bank, not a stub of a viaduct
        let mut over = vec![0.03; 6];
        over.extend([0.1; 3]);
        over.extend([0.03; 6]);
        assert!(kinds(&over, &[], &[]).iter().all(|&x| x == Kind::Embankment));
    }

    #[test]
    fn a_river_is_bridged_and_a_viaduct_carries_on_over_one() {
        let k = kinds(&[0.05; 12], &[5, 6], &[]);
        assert_eq!(&k[4..8], &[Kind::Embankment, Kind::Bridge, Kind::Bridge, Kind::Embankment]);
        let k = kinds(&[0.1; 20], &[9, 10], &[]);
        assert!(k.iter().all(|&x| x == Kind::Viaduct));
        // and what is under the ground is a tunnel, whatever else
        let k = kinds(&[-0.1; 6], &[2], &[1, 2, 3]);
        assert_eq!(&k[1..4], &[Kind::Tunnel; 3]);
    }

    /// A year's railway of one straight line along x at these heights over ground at nought.
    fn one_line(over: &[f32]) -> RailYear {
        let pts: Vec<Vec3> = over.iter().enumerate().map(|(i, &h)| Vec3::new(3.0 + i as f32 * 0.02, h, 5.5)).collect();
        let n = pts.len();
        let kind = classify(&pts, &vec![0.0; n], &vec![false; n], &vec![false; n]);
        let line = Line { pts, ground: vec![0.0; n], kind, worked: vec![Worked::Plain; n], upland: vec![false; n] };
        RailYear { lines: vec![line], building: Vec::new(), stops: HashMap::new(), stands: Vec::new(), works: OnceLock::new(), room: OnceLock::new() }
    }

    #[test]
    fn the_railway_takes_the_ground_it_runs_on_and_its_banks() {
        // on the ground: the bed and a little beside it
        let ry = one_line(&[0.0; 40]);
        assert!(ry.near(3.4, 5.5, 0.0) && ry.near(3.4, 5.5 + HALF, 0.0));
        assert!(!ry.near(3.4, 5.56, 0.0) && ry.near(3.4, 5.56, 0.03));
        // nowhere else on the island, in the same cell or far from it
        assert!(!ry.near(3.4, 5.9, 0.016) && !ry.near(40.2, 17.5, 0.016) && !ry.near(3.4, 4.5, 0.016));
        // on a bank it takes the bank's slopes too
        let ry = one_line(&[0.06; 40]);
        assert!(ry.near(3.4, 5.58, 0.0) && !ry.near(3.4, 5.7, 0.0));
        // just across the edge of a cell from the line is still by the line
        let ry = one_line(&[0.0; 40]);
        let mut edge = one_line(&[0.0; 40]);
        edge.lines[0].pts.iter_mut().for_each(|p| p.z = 6.01);
        assert!(edge.near(3.4, 5.99, 0.0) && !ry.near(3.4, 5.99, 0.0));
    }

    #[test]
    fn what_stands_by_the_line_holds_its_own_ground_and_no_more() {
        use isostate_gen::rail;
        // a shed whose front faces +z, its middle at the origin
        let st = Stand { what: Standing::BankerShed, at: Vec3::new(10.0, 0.3, 20.0), yaw: 0.0 };
        let (w, d) = rail::BANKER_SHED;
        assert!(stand_holds(&st, Vec2::new(10.0 + 0.49 * w, 20.0 - 0.49 * d), 0.0));
        assert!(!stand_holds(&st, Vec2::new(10.0 + 0.51 * w, 20.0), 0.0) && !stand_holds(&st, Vec2::new(10.0, 20.0 + 0.51 * d), 0.0));
        // the track it faces runs clear of it, with the room a bed takes
        let track = Vec2::new(10.0, 20.0 + 0.5 * d + rail::TRACK_CLEAR);
        assert!(!stand_holds(&st, track, 0.8 * HALF));
        // turned a quarter, its length lies along z
        let turned = Stand { yaw: std::f32::consts::FRAC_PI_2, ..st };
        assert!(stand_holds(&turned, Vec2::new(10.0, 20.0 + 0.45 * w), 0.0) && !stand_holds(&turned, Vec2::new(10.0 + 0.45 * w, 20.0), 0.0));
        // and what the yards are told to keep off covers it
        let mut ry = one_line(&[0.0; 4]);
        ry.stands.push(st);
        let grounds = ry.grounds();
        assert_eq!(grounds.len(), 1);
        assert!(grounds[0].1 >= 0.5 * w.hypot(d));
    }

    #[test]
    fn the_rules_allow_what_the_track_is_drawn_to() {
        // two neighbouring points of a length may differ by no more than the ruling gradient
        // allows over a quarter of a straight cell, so the straight line between them holds it
        let part = 1.02 / (SUBS + 1) as f32;
        assert!(RULES.cap(part, 0) / part <= RULING);
        assert!(RULES.cap(part, RULES.wind_steps) / (part * RULES.wind) <= RULING);
        assert!(RULES.cap(part, RULES.incline()) / part > 4.0 * RULING);
        assert_eq!(runs_of(6, |i| i == 1 || i == 2 || i == 5), vec![(1, 2), (5, 5)]);
    }
}
