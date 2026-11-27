//! Squares of 125 metres where something is built (not in the Python;
//! `docs/rules/squares.md`). The shadow: kept from 1750, read by no rule.
//!
//! The books say how much of a cell is works, pits and town ground, and not
//! where in the cell each stands; the game's scenery invents that. With
//! `squares.enabled` the simulation keeps the place too. Every cell that has
//! something built gets a table of sixteen by sixteen squares, each 125 m
//! across, and a square has one tenant and the year it was taken:
//!
//! - a works (which one): as many squares as its ground comes to, in one
//!   piece, grown from the square it first stood on;
//! - a pit's surface (which working);
//! - on a town's ground, house blocks (with their shops), a park, and the
//!   town's open ground, which is the rest; with `works.in_blocks` a house
//!   block says how much of its floor is let to shops, offices, care and the
//!   builders (`Table::floor`), and those works hold no square of their own
//!   on a town's cells;
//! - derelict ground, where a works closed and the books call the cell derelict.
//!
//! A way takes no square. A cell with nothing built keeps no table.
//!
//! The squares are allotted once a year, at the year's end, after every rule
//! that changes the books has run (`close_year`, from `sim::end_year`), and once
//! as the run opens (`open`, from `sim::start_from`). Allotting is a reckoning
//! of differences, not a laying out afresh: a tenant keeps its squares and
//! their year until the books take its ground away; a tenant that holds more
//! than its ground gives back the squares it took last, and one that holds
//! less takes squares beside its own. Where each new square goes is a rule of
//! the ground (beside the way, toward or away from the town's middle, by water
//! for a mill, off steep and wet ground) with a hash of the square's place to
//! break ties, so no draw comes from the run's streams of chance.
//!
//! Each year's end also writes down what it changed (`tell`, into
//! `SquareBook::changes`), which the record keeps as a log in bytes
//! (`encode_year`) and a live game hands over: the same list for both.
//!
//! Nothing here is read by any rule: the economy, the books and the news are
//! the same to the bit with the switch on or off. Switched off, `open` and
//! `close_year` return at once and the book stays empty.

use std::collections::BTreeMap;
use std::f32::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::crossings::{self, Fine, OnLand};
use crate::params::Params;
use crate::state::State;

/// Squares to a cell's side: 16 of 125 m to a cell of 2 km.
pub const SIDE: usize = 16;
/// Squares to a cell.
pub const PER_CELL: usize = SIDE * SIDE;
/// Metres to a square's side.
pub const METRES: f64 = 125.0;

/// The switch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SquaresParams {
    /// The simulation keeps squares of 125 m where something is built.
    pub enabled: bool,
    /// With the ground by method (`works.by_method`): a works keeps the squares
    /// it holds while its ground in the books is at this share of them or more
    /// (`keep_squares`): its buildings stand while some of its hands leave.
    pub keep: f64,
}

impl Default for SquaresParams {
    /// Off: what the port runs with.
    fn default() -> SquaresParams {
        SquaresParams { enabled: false, keep: 0.7 }
    }
}

impl SquaresParams {
    /// The game's squares.
    pub fn game() -> SquaresParams {
        SquaresParams { enabled: true, ..SquaresParams::default() }
    }
}

// ---------------------------------------------------------------- tenants

/// What a square is given to. Held in a table as one `u32`: the kind in the
/// top four bits, the number below.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tenant {
    /// Nothing built: the open country.
    None,
    /// Works f.
    Works(u32),
    /// House blocks of town i, with their shops.
    Houses(u32),
    /// A park of town i.
    Park(u32),
    /// The surface of the working of resource row r on this cell (`mine_sectors[r]`).
    Pit(u32),
    /// Derelict ground, left by works f.
    Derelict(u32),
    /// Town i's ground not yet built on: yards, gardens, paddocks, lanes.
    Open(u32),
}

const KIND_SHIFT: u32 = 28;
const ID_MASK: u32 = (1 << KIND_SHIFT) - 1;

/// The kinds of tenant by the number a code holds in its top four bits
/// (`Tenant::kind` names them), as the record's header lists them.
pub const KINDS: [&str; 7] = ["none", "works", "houses", "park", "pit", "derelict", "open"];

/// The uses of floor let in a house block (`Table::floor`'s high four bits),
/// as the record's header lists them: shops (the shops and trades and the
/// emporium), offices, care, and the builders' yards and offices.
pub const FLOOR_USES: [&str; 5] = ["none", "shops", "offices", "care", "trades"];

/// The use of floor a works of method m lets (`FLOOR_USES`): by its good,
/// and for the good services by its ground a hand (an office's desk at 12 m²
/// or less, a shop's counter above it).
pub fn floor_use(C: &Content, m: usize) -> u8 {
    let history = match C.m_ground_history.get(m) {
        Some(&h) if h > 0.0 => h,
        _ => C.m_ground.get(m).copied().unwrap_or(0.0),
    };
    match C.good_name[C.method_out[m]].as_str() {
        "care" => 3,
        "building" => 4,
        _ if history <= 12.0 => 2,
        _ => 1,
    }
}

/// The most eighths of a square's floor shops take while there are house
/// blocks of the cell left to take: the street floor of a block of four.
const SHOP_EIGHTHS: u8 = 2;
impl Tenant {
    /// The tenant as a table holds it.
    pub fn code(self) -> u32 {
        let (k, id) = match self {
            Tenant::None => (0, 0),
            Tenant::Works(f) => (1, f),
            Tenant::Houses(i) => (2, i),
            Tenant::Park(i) => (3, i),
            Tenant::Pit(r) => (4, r),
            Tenant::Derelict(f) => (5, f),
            Tenant::Open(i) => (6, i),
        };
        (k << KIND_SHIFT) | (id & ID_MASK)
    }

    /// The tenant a table's `u32` stands for.
    pub fn of(code: u32) -> Tenant {
        let id = code & ID_MASK;
        match code >> KIND_SHIFT {
            1 => Tenant::Works(id),
            2 => Tenant::Houses(id),
            3 => Tenant::Park(id),
            4 => Tenant::Pit(id),
            5 => Tenant::Derelict(id),
            6 => Tenant::Open(id),
            _ => Tenant::None,
        }
    }

    /// A short name for the kind, as the census and the pictures use it.
    pub fn kind(self) -> &'static str {
        KINDS[(self.code() >> KIND_SHIFT) as usize]
    }

    /// Town ground: house blocks, a park or open ground.
    fn town(self) -> Option<u32> {
        match self {
            Tenant::Houses(i) | Tenant::Park(i) | Tenant::Open(i) => Some(i),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------- the book

/// One cell's squares, row by row from its north-west corner: square
/// `k = qy * 16 + qx`, its middle at (`cell_x + (qx + 0.5) / 16`, `cell_y + (qy + 0.5) / 16`)
/// in the map's cells.
#[derive(Clone, Debug)]
pub struct Table {
    pub cell: usize,
    /// `Tenant::code` of each square.
    pub tenant: [u32; PER_CELL],
    /// The year each square was taken by its tenant (0 where it has none).
    pub year: [i16; PER_CELL],
    /// Which side the way lies on, from the square's middle toward the
    /// nearest point of the cell's ways when the square was taken: a bearing in
    /// 256ths of a turn, 0 toward the east (the map's x) and 64 toward the south (its y).
    pub face: [u8; PER_CELL],
    /// On a square of house blocks, the floor in it let to shops, offices,
    /// care and the builders (works.in_blocks, docs/rules/works.md part J):
    /// the use in the high four bits (`FLOOR_USES`), the eighths of the
    /// square's floor in that use in the low four (0 to 8). 0 on every other square.
    pub floor: [u8; PER_CELL],
    /// The square lies under the sea or a lake.
    pub water: [bool; PER_CELL],
    /// How bad the ground is to build on: steep and wet (`ground_cost`).
    bad: [f32; PER_CELL],
    /// How wet: 0 dry, 1 flood plain, 2 marsh.
    wet: [u8; PER_CELL],
    /// Squares from the square's middle to the nearest water (river, lake or sea), at most 16.
    to_water: [f32; PER_CELL],
    /// The town's house blocks and park in the order they were taken.
    houses: Vec<u8>,
    park: Vec<u8>,
    /// Each square as the log last told it (`Change::pack`): what the
    /// record and a live game hold of it.
    told: [u64; PER_CELL],
}

/// One change to a square, as the record and a live game carry it: the
/// square, and what it is from the year the change is logged in. A year's
/// changes are those that make the squares of its end out of the squares of
/// the year before's (out of none, for the run's first year, whose changes
/// carry the opening too).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Change {
    /// The cell times 256 plus the square (`k = qy * 16 + qx`).
    pub at: u32,
    /// The new tenant's `Tenant::code`: 0 where the square was given up.
    pub tenant: u32,
    /// The year its tenant took it, which is the year of the log for most;
    /// the squares of the opening are dated as the books date what stands. 0
    /// where the square was given up.
    pub year: i16,
    /// `Table::face`: which side its way lies on (0 where it was given up).
    pub face: u8,
    /// `Table::floor`: the floor of a house block let to works, its use and
    /// eighths (0 on every other square).
    pub floor: u8,
}

impl Change {
    pub fn cell(&self) -> usize {
        self.at as usize / PER_CELL
    }

    pub fn square(&self) -> usize {
        self.at as usize % PER_CELL
    }

    /// Tenant, year, face and floor in one number, 0 for a square nothing holds.
    pub fn pack(tenant: u32, year: i16, face: u8, floor: u8) -> u64 {
        if tenant == 0 { 0 } else { tenant as u64 | (year as u16 as u64) << 32 | (face as u64) << 48 | (floor as u64) << 56 }
    }

    /// The change that makes square `at` what `packed` says.
    pub fn unpack(at: u32, packed: u64) -> Change {
        Change { at, tenant: packed as u32, year: (packed >> 32) as u16 as i16, face: (packed >> 48) as u8, floor: (packed >> 56) as u8 }
    }
}

/// A works' or a pit's squares, in the order taken (the first is where it first stood).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Holding {
    pub cell: usize,
    pub squares: Vec<u8>,
    /// `Table::face` of its first square: which side its way lies on.
    pub face: u8,
    /// The way's own heading nearest its first square, in 256ths of a half turn
    /// (a way runs both ways): what a yard is turned square to.
    pub along: u8,
    /// The year it first took ground.
    pub year: i16,
    /// Squares it took apart from its own because other works and pits closed
    /// it in on every side: only such a holding may be in more than one piece.
    pub annexes: u16,
}

/// Everything the squares keep. Empty while the switch is off.
#[derive(Clone, Debug, Default)]
pub struct SquareBook {
    /// By cell: its table's place in `tables` plus one, or 0 where it has none.
    pub index: Vec<u32>,
    pub tables: Vec<Table>,
    /// By cell with no table now: the place of the table it last had, plus one,
    /// or 0. A table given up stays in `tables`, empty, with its ground worked
    /// out, for when its cell is built on again.
    spare: Vec<u32>,
    /// By works: its squares.
    pub works: Vec<Holding>,
    /// By (resource row, cell): a working's squares.
    pub pits: BTreeMap<(usize, usize), Holding>,
    /// By cell: the river's stretches that touch it, in cells.
    rivers: Vec<Vec<[f32; 4]>>,
    /// The last year allotted (0: never).
    pub year: i64,
    /// Times a works or pit took a square not beside its own, for want of one.
    pub split: u64,
    /// Squares a tenant should have had and could not, for want of room in its cell.
    pub short: u64,
    /// The log: the squares the last year's end changed, by `Change::at`
    /// (the record and a live game carry them; docs/rules/squares.md, "In the record").
    pub changes: Vec<Change>,
    /// Year's ends logged so far: the first's `changes` are from no squares at all.
    pub told: u64,
    /// What each works and working took this year, in the order taken (the log's holdings).
    pub took: Vec<Took>,
    /// The holdings' log of the last year's end (`encode_holds`).
    pub holds: Vec<u8>,
}

/// What one works or working took in a year's end, as the record's
/// holdings' log (`sq_hold`) and a live game carry it: its squares in the
/// order taken, and where it took its first, the heading of the way there.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Took {
    pub cell: u32,
    /// A working's pit (its resource row as `id`), else a works (its number).
    pub pit: bool,
    pub id: u32,
    /// It held none before: the first of `squares` is where it first stands,
    /// and `along` (`Holding::along`) is the heading of the way there.
    pub fresh: bool,
    pub along: u8,
    pub squares: Vec<u8>,
}

impl SquareBook {
    /// Whether the book is kept in this run.
    pub fn kept(&self) -> bool {
        !self.index.is_empty()
    }

    /// The table of cell c, if it has one.
    pub fn table(&self, c: usize) -> Option<&Table> {
        match self.index.get(c) {
            Some(&k) if k > 0 => Some(&self.tables[k as usize - 1]),
            _ => None,
        }
    }

    /// The tenant and year of square k of cell c.
    pub fn at(&self, c: usize, k: usize) -> (Tenant, i16) {
        self.table(c).map_or((Tenant::None, 0), |t| (Tenant::of(t.tenant[k]), t.year[k]))
    }

    /// Every square something holds, as the log has told them so far, by
    /// `Change::at`: what a reader that has every year's changes holds after
    /// the last of them, and what a live game sends whole to a viewer that has
    /// missed a year.
    pub fn whole(&self) -> Vec<Change> {
        let mut all: Vec<Change> = Vec::new();
        for t in &self.tables {
            for k in 0..PER_CELL {
                if t.told[k] != 0 {
                    all.push(Change::unpack((t.cell * PER_CELL + k) as u32, t.told[k]));
                }
            }
        }
        all.sort_unstable_by_key(|c| c.at);
        all
    }
}

// ---------------------------------------------------------------- the ground under a square

/// A square's middle in the map's cells.
fn middle(c: usize, w: usize, k: usize) -> (f32, f32) {
    ((c % w) as f32 + ((k % SIDE) as f32 + 0.5) / SIDE as f32, (c / w) as f32 + ((k / SIDE) as f32 + 0.5) / SIDE as f32)
}

/// What a square's ground costs a builder: steepness (1 at a grade of 1 in
/// 12, rising as its square, at most 25) and wetness (0.6 on the flood plain,
/// 2.5 in the marsh).
fn ground_cost(grade: f32, wet: u8) -> f32 {
    let steep = (grade / 0.083).powi(2).min(25.0);
    steep + match wet {
        0 => 0.0,
        1 => 0.6,
        _ => 2.5,
    }
}

fn seg_dist(px: f32, py: f32, s: &[f32; 4]) -> (f32, f32, f32) {
    let (ax, ay, bx, by) = (s[0], s[1], s[2], s[3]);
    let (dx, dy) = (bx - ax, by - ay);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((px - ax) * dx + (py - ay) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let (qx, qy) = (ax + t * dx, ay + t * dy);
    ((px - qx).hypot(py - qy), qx, qy)
}

fn new_table(st: &State, book: &SquareBook, c: usize) -> Table {
    let w = st.world.W;
    let mut t = Table {
        cell: c,
        tenant: [0; PER_CELL],
        year: [0; PER_CELL],
        face: [0; PER_CELL],
        floor: [0; PER_CELL],
        water: [false; PER_CELL],
        bad: [0.0; PER_CELL],
        wet: [0; PER_CELL],
        to_water: [16.0; PER_CELL],
        houses: Vec::new(),
        park: Vec::new(),
        told: [0; PER_CELL],
    };
    let Some(lf) = st.world.landform.as_deref() else {
        // the old island: no ground finer than the cell, and water only where the cell is
        if !st.world.land[c] {
            t.water = [true; PER_CELL];
        }
        return t;
    };
    let l = OnLand { lf, w, h: st.world.H };
    let (fine, fw, fh) = l.grid();
    let h = 0.5 / SIDE as f32;
    let (cx, cy) = ((c % w) as f32, (c / w) as f32);
    // the water nodes about the cell, in cells (the node's middle)
    let mut water_nodes: Vec<(f32, f32)> = Vec::new();
    let (n0x, n0y) = ((cx as isize - 1) * fine as isize, (cy as isize - 1) * fine as isize);
    for j in n0y..n0y + 3 * fine as isize {
        for i in n0x..n0x + 3 * fine as isize {
            if i < 0 || j < 0 || i as usize >= fw || j as usize >= fh {
                continue;
            }
            let n = j as usize * fw + i as usize;
            if lf.lake_id[n] > 0 || lf.height[n] <= 0.0 {
                water_nodes.push(((i as f32 + 0.5) / fine as f32, (j as f32 + 0.5) / fine as f32));
            }
        }
    }
    for k in 0..PER_CELL {
        let (x, y) = middle(c, w, k);
        let here = crossings::metres_at(&l, x, y);
        let n = crossings::node_at(&l, x, y);
        t.water[k] = here <= 0.0 || lf.lake_id[n] > 0;
        let gx = (crossings::metres_at(&l, x + h, y) - crossings::metres_at(&l, x - h, y)) / METRES as f32;
        let gy = (crossings::metres_at(&l, x, y + h) - crossings::metres_at(&l, x, y - h)) / METRES as f32;
        t.wet[k] = lf.wet[n].min(2);
        t.bad[k] = ground_cost(gx.hypot(gy), t.wet[k]);
        let mut d = 16.0f32;
        for s in &book.rivers[c] {
            d = d.min(seg_dist(x, y, s).0 * SIDE as f32);
        }
        for &(wx, wy) in &water_nodes {
            d = d.min((wx - x).hypot(wy - y) * SIDE as f32);
        }
        t.to_water[k] = d;
    }
    t
}

// ---------------------------------------------------------------- where in the cell

/// What the squares of one cell are placed by this year: its ways and the
/// middle of its market town, in squares from the cell's north-west corner.
struct Frame {
    ways: Vec<[f32; 4]>,
    meet: (f32, f32),
    mid: (f32, f32),
    c: usize,
    w: usize,
    /// The ways are the kept courses (courses.rs): house blocks and shops take the frontage first.
    courses: bool,
}

fn meeting(st: &State, c: usize) -> (f32, f32) {
    let w = st.world.W;
    match st.xs.table.middles.get(c) {
        Some(&m) => m,
        None => ((c % w) as f32 + 0.5, (c / w) as f32 + 0.5),
    }
}

fn frame(st: &State, c: usize) -> Frame {
    let (w, h) = (st.world.W, st.world.H);
    let (cx, cy) = ((c % w) as f32, (c / w) as f32);
    let local = |p: (f32, f32)| ((p.0 - cx) * SIDE as f32, (p.1 - cy) * SIDE as f32);
    let m = meeting(st, c);
    let mut meet = local(m);
    let mut ways = Vec::new();
    // courses (docs/rules/courses.md): the year's drawn courses about the cell,
    // and its place where it has one; no way yet, the place or the meeting place
    // is where one would come
    let courses = st.wc.kept();
    if courses {
        if let Some(p) = st.wc.place_of(c) {
            meet = local(p);
        }
        for s in &st.wc.near[c] {
            let (a, b) = (local((s[0], s[1])), local((s[2], s[3])));
            ways.push([a.0, a.1, b.0, b.1]);
        }
    }
    // the ways as the viewer joined them before: from the meeting place halfway to
    // each neighbour's, where both cells have a road
    if !courses && st.road.get(c).is_some_and(|&r| r > 0) {
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                let (nx, ny) = ((c % w) as isize + dx, (c / w) as isize + dy);
                if (dx, dy) == (0, 0) || nx < 0 || ny < 0 || nx as usize >= w || ny as usize >= h {
                    continue;
                }
                let n = ny as usize * w + nx as usize;
                if st.road[n] > 0 {
                    let o = meeting(st, n);
                    let half = local(((m.0 + o.0) / 2.0, (m.1 + o.1) / 2.0));
                    ways.push([meet.0, meet.1, half.0, half.1]);
                }
            }
        }
    }
    if ways.is_empty() {
        // no way yet: the meeting place is where one would come
        ways.push([meet.0, meet.1, meet.0, meet.1]);
    }
    let t = st.catch.get(c).copied().unwrap_or(-1);
    let mid = if t >= 0 { local(meeting(st, st.town_node[t as usize])) } else { meet };
    Frame { ways, meet, mid, c, w, courses }
}

impl Frame {
    fn xy(k: usize) -> (f32, f32) {
        ((k % SIDE) as f32 + 0.5, (k / SIDE) as f32 + 0.5)
    }

    /// Squares to the nearest way, and that point.
    fn to_way(&self, k: usize) -> (f32, f32, f32, usize) {
        let (x, y) = Frame::xy(k);
        let mut best = (f32::INFINITY, x, y, 0);
        for (j, s) in self.ways.iter().enumerate() {
            let (d, qx, qy) = seg_dist(x, y, s);
            if d < best.0 {
                best = (d, qx, qy, j);
            }
        }
        best
    }

    fn to_mid(&self, k: usize) -> f32 {
        let (x, y) = Frame::xy(k);
        (x - self.mid.0).hypot(y - self.mid.1)
    }

    /// A number from 0 to 1 for square k, the same in every run.
    fn hash(&self, k: usize, s: i32) -> f32 {
        let gx = (self.c % self.w * SIDE + k % SIDE) as i32;
        let gy = (self.c / self.w * SIDE + k / SIDE) as i32;
        crossings::hash2(gx, gy, 9100 + s)
    }

    /// Which side the way lies on from square k, and the way's heading there.
    fn face(&self, k: usize) -> (u8, u8) {
        let (x, y) = Frame::xy(k);
        let (d, qx, qy, j) = self.to_way(k);
        let s = &self.ways[j];
        let (dx, dy) = if d > 1e-3 { (qx - x, qy - y) } else { (self.meet.0 - x, self.meet.1 - y) };
        let face = if dx.hypot(dy) > 1e-6 { ((dy.atan2(dx) / TAU).rem_euclid(1.0) * 256.0).round() as u32 % 256 } else { 0 };
        let (ax, ay) = (s[2] - s[0], s[3] - s[1]);
        let along = if ax.hypot(ay) > 1e-6 { ((ay.atan2(ax) / (TAU / 2.0)).rem_euclid(1.0) * 256.0).round() as u32 % 256 } else { 0 };
        (face as u8, along as u8)
    }
}

/// What a tenant looks for in a square (higher is better).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Want {
    /// A shop or an office: beside the way, toward the town's middle.
    Shop,
    /// A works: beside the way, away from the town's middle.
    Works,
    /// A works that needs water (a mill): by the water too.
    Mill,
    /// A pit: dry ground, near enough the way, away from the town.
    Pit,
    /// House blocks: beside the way, toward the town's middle.
    Houses,
    /// A park: the wet, the steep and the waterside.
    Park,
}

/// What a square on a kept course's frontage is worth to a house block or a
/// shop, over its other terms (courses.rs).
const FRONTAGE: f32 = 1.0;
/// How far from a kept course a works' yard still counts as on its way, in squares (courses.rs).
const YARD_REACH: f32 = 3.0;

fn score(t: &Table, fr: &Frame, k: usize, want: Want) -> f32 {
    if t.water[k] && want != Want::Park {
        // under water: only when there is nothing else
        return -1000.0 - fr.to_way(k).0;
    }
    let way = fr.to_way(k).0;
    let mid = fr.to_mid(k).min(24.0);
    let bad = t.bad[k];
    let wat = t.to_water[k].min(16.0);
    let h = fr.hash(k, want as i32);
    // courses: a house block or a shop on the way's frontage, within a square of it,
    // before any square behind (FRONTAGE is about eight squares nearer the town's middle)
    let front = if fr.courses && way <= 1.0 && matches!(want, Want::Houses | Want::Shop) { FRONTAGE } else { 0.0 };
    // courses: a works wants its way within reach (YARD_REACH squares), not on its
    // frontage, so that the works of a cell do not crowd along its one course and hem
    // each other in (with the eight half-links every square was near a way)
    let yard_way = if fr.courses { way.min(YARD_REACH) } else { way };
    front + match want {
        Want::Shop => -0.3 * way - 0.15 * mid - bad + 0.2 * h,
        Want::Works => -0.2 * yard_way + 0.06 * mid - bad + 0.2 * h,
        Want::Mill => -0.2 * yard_way + 0.06 * mid - 0.35 * wat - bad + 0.2 * h,
        Want::Pit => -0.05 * way + 0.06 * mid - 0.5 * bad + 0.2 * h,
        Want::Houses => -0.25 * way - 0.12 * mid - 1.5 * bad + 0.3 * h,
        Want::Park => {
            if t.water[k] {
                -1000.0
            } else {
                0.8 * (t.wet[k] as f32) + 0.3 * bad.min(6.0) - 0.15 * wat + 0.2 * h
            }
        }
    }
}

/// `score`, kept in `memo` once worked out.
fn scored(memo: &mut [f32; PER_CELL], t: &Table, fr: &Frame, k: usize, want: Want) -> f32 {
    if memo[k].is_nan() {
        memo[k] = score(t, fr, k, want);
    }
    memo[k]
}

/// What a works of method m looks for.
fn works_want(C: &Content, m: usize) -> Want {
    if C.good_name[C.method_out[m]] == "services" {
        Want::Shop
    } else if C.method_name[m].contains("mill") {
        Want::Mill
    } else {
        Want::Works
    }
}

/// What it costs a works or a pit to take a square from its tenant, or None
/// where it may not: house blocks and parks give way to works, at a cost that
/// sends a works round them where it can.
fn displace(code: u32) -> Option<f32> {
    match Tenant::of(code) {
        Tenant::None | Tenant::Derelict(_) | Tenant::Open(_) => Some(0.0),
        Tenant::Park(_) => Some(1.0),
        Tenant::Houses(_) => Some(3.0),
        Tenant::Works(_) | Tenant::Pit(_) => None,
    }
}

// ---------------------------------------------------------------- the year

/// As the run opens, after the warm-up and the families: the squares of what
/// stands, each dated as the books date it (a works by its founding, town
/// ground by the year it was built, a working by its opening).
pub fn open(st: &mut State, C: &Content, P: &Params, year: i64) {
    if !P.squares.enabled {
        return;
    }
    start_book(st);
    allot(st, C, P, year, true);
}

/// The year's end, after every rule that changes the books has run: the
/// squares follow what the books now say.
pub fn close_year(st: &mut State, C: &Content, P: &Params, year: i64) {
    if !P.squares.enabled {
        return;
    }
    if !st.sq.kept() {
        start_book(st);
    }
    allot(st, C, P, year, false);
    tell(&mut st.sq);
}

/// The log of the year's end: every square whose tenant, year or face is not
/// what the log last told, in order of `Change::at`. Every table is looked at,
/// those given up too (their squares went back to nothing as they were).
fn tell(b: &mut SquareBook) {
    b.changes.clear();
    for t in &mut b.tables {
        for k in 0..PER_CELL {
            let now = Change::pack(t.tenant[k], t.year[k], t.face[k], t.floor[k]);
            if now != t.told[k] {
                t.told[k] = now;
                b.changes.push(Change::unpack((t.cell * PER_CELL + k) as u32, now));
            }
        }
    }
    b.changes.sort_unstable_by_key(|c| c.at);
    // (the first year's log is made out of nothing, its squares' and its holdings' alike:
    // every holding's squares in the order taken, the opening's among them)
    b.holds = if b.told == 0 { whole_holds(b) } else { encode_holds(&b.took) };
    b.took.clear();
    b.told += 1;
}

/// Every works' and working's squares in the order taken, as one year's
/// holdings' log from nothing: for a live game that has missed a year's end.
pub fn whole_holds(b: &SquareBook) -> Vec<u8> {
    let mut all: Vec<Took> = Vec::new();
    for (f, h) in b.works.iter().enumerate() {
        if !h.squares.is_empty() {
            all.push(Took { cell: h.cell as u32, pit: false, id: f as u32, fresh: true, along: h.along, squares: h.squares.clone() });
        }
    }
    for (&(r, c), h) in &b.pits {
        if !h.squares.is_empty() {
            all.push(Took { cell: c as u32, pit: true, id: r as u32, fresh: true, along: h.along, squares: h.squares.clone() });
        }
    }
    encode_holds(&all)
}

/// The holdings' log of a year in bytes (`sq_hold`; layout 3): the takings
/// in order of cell, then pits after works, then number. For each: the cell
/// less the one before it (the first less 0); a head, `(id << 1 | pit) << 1
/// | fresh`; where `fresh`, a byte, the way's heading there (`Holding::along`,
/// in 256ths of a half turn); the number of squares taken; and each square's
/// place in the cell (`k`), in the order taken. A holding's squares are a
/// stack: the first it ever took is where it first stood, and what it gives
/// back is always what it took last (the squares' log shows which went).
/// Every number but the heading and the squares is a varint, seven bits a
/// byte, low first.
pub fn encode_holds(took: &[Took]) -> Vec<u8> {
    let mut sorted: Vec<&Took> = took.iter().collect();
    sorted.sort_by_key(|t| (t.cell, t.pit, t.id));
    let mut out = Vec::new();
    let mut prev = 0u32;
    for t in sorted {
        put_varint(&mut out, (t.cell - prev) as u64);
        prev = t.cell;
        put_varint(&mut out, ((t.id as u64) << 1 | t.pit as u64) << 1 | t.fresh as u64);
        if t.fresh {
            out.push(t.along);
        }
        put_varint(&mut out, t.squares.len() as u64);
        out.extend_from_slice(&t.squares);
    }
    out
}

/// A year of the holdings' log read back (`encode_holds`), or what is wrong with the bytes.
pub fn decode_holds(bytes: &[u8]) -> Result<Vec<Took>, String> {
    let mut out = Vec::new();
    let (mut i, mut cell) = (0usize, 0u64);
    while i < bytes.len() {
        cell += get_varint(bytes, &mut i)?;
        let head = get_varint(bytes, &mut i)?;
        let (fresh, pit, id) = (head & 1 == 1, head >> 1 & 1 == 1, head >> 2);
        let along = if fresh {
            let Some(&b) = bytes.get(i) else { return Err("the holdings' log ends in the middle of a heading".into()) };
            i += 1;
            b
        } else {
            0
        };
        let n = get_varint(bytes, &mut i)? as usize;
        let Some(sq) = bytes.get(i..i + n) else { return Err("the holdings' log ends in the middle of a holding".into()) };
        i += n;
        out.push(Took { cell: cell as u32, pit, id: id as u32, fresh, along, squares: sq.to_vec() });
    }
    Ok(out)
}

fn start_book(st: &mut State) {
    let N = st.N;
    let w = st.world.W;
    let mut book = SquareBook { index: vec![0; N], spare: vec![0; N], rivers: vec![Vec::new(); N], ..SquareBook::default() };
    if let Some(lf) = st.world.landform.as_deref() {
        for r in &lf.rivers {
            for i in 1..r.x.len() {
                let s = [r.x[i - 1], r.y[i - 1], r.x[i], r.y[i]];
                // the stretch is put in every cell its box touches
                let (x0, x1) = (s[0].min(s[2]).floor() as isize, s[0].max(s[2]).floor() as isize);
                let (y0, y1) = (s[1].min(s[3]).floor() as isize, s[1].max(s[3]).floor() as isize);
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < st.world.H {
                            book.rivers[y as usize * w + x as usize].push(s);
                        }
                    }
                }
            }
        }
    }
    st.sq = book;
}

fn table_of(st: &mut State, c: usize) -> usize {
    let k = st.sq.index[c];
    if k > 0 {
        return k as usize - 1;
    }
    // a cell that had a table before has it again, with its ground worked out
    // already (a table given up is left empty in its place, never handed to another cell)
    let k = match st.sq.spare[c] {
        0 => {
            let t = new_table(st, &st.sq, c);
            st.sq.tables.push(t);
            st.sq.tables.len() - 1
        }
        k => k as usize - 1,
    };
    st.sq.index[c] = k as u32 + 1;
    k
}

/// Shares a cell's whole squares among those holding `x` squares each (all
/// on the cell), the sum rounded to the nearest square: each gets its whole
/// squares, and the squares left over go to the largest remainders, those
/// that already hold the square first, so that no tenant gives up a square
/// another takes. `held` is what each holds now.
fn share_out(x: &[f64], held: &[usize], total: usize) -> Vec<usize> {
    let mut n: Vec<usize> = x.iter().map(|v| v.max(0.0).floor() as usize).collect();
    let base: usize = n.iter().sum();
    let mut left = total.saturating_sub(base);
    let mut order: Vec<usize> = (0..x.len()).filter(|&j| x[j] - x[j].floor() > 0.0).collect();
    order.sort_by(|&a, &b| {
        let keep = |j: usize| held[j] > n[j];
        keep(b).cmp(&keep(a)).then((x[b] - x[b].floor()).total_cmp(&(x[a] - x[a].floor()))).then(a.cmp(&b))
    });
    for j in order {
        if left == 0 {
            break;
        }
        n[j] += 1;
        left -= 1;
    }
    n
}

/// Whether works f stands in the blocks of its town (works.in_blocks,
/// docs/rules/works.md part J): a works of the blocks on a town's cell, which
/// holds no square of its own but floor in the house blocks (`Table::floor`).
pub fn floored(st: &State, C: &Content, P: &Params, f: usize) -> bool {
    st.urban.get(st.f_cell[f]).copied().unwrap_or(false) && crate::works::in_blocks(C, P, st.f_method[f])
}

/// Squares each works should hold this year, by works, from the books. A
/// works in the blocks (`floored`) holds none.
pub fn works_targets(st: &State, C: &Content, P: &Params) -> Vec<usize> {
    let F = st.F_count;
    let mut by_cell: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for f in 0..F {
        if st.f_alive[f] && !floored(st, C, P, f) {
            by_cell.entry(st.f_cell[f]).or_default().push(f);
        }
    }
    let mut want = vec![0usize; F];
    for (_, fs) in by_cell {
        let x: Vec<f64> = fs.iter().map(|&f| crate::works::land_held(st, C, P, f) * PER_CELL as f64).collect();
        let sum: f64 = x.iter().sum();
        let scale = if sum > PER_CELL as f64 { PER_CELL as f64 / sum } else { 1.0 };
        let x: Vec<f64> = x.iter().map(|v| v * scale).collect();
        let total = (sum * scale).round() as usize;
        let held: Vec<usize> = fs.iter().map(|&f| st.sq.works.get(f).map_or(0, |h| h.squares.len())).collect();
        let mut n = share_out(&x, &held, total);
        if P.works.enabled && P.works.by_method {
            keep_squares(&x, &held, &mut n, P.squares.keep);
        }
        for (j, n) in n.into_iter().enumerate() {
            want[fs[j]] = n;
        }
    }
    want
}

/// Works ground by method (`works.by_method`, docs/rules/works.md, part I): a
/// works keeps the squares it holds while its ground in the books stays at
/// `keep` of them or more, as a building stands while some of its hands are
/// laid off; below that it gives back down to its ground. Where the cell has
/// not room for what is kept, the works that keep most over their ground give
/// it up first. `x` is each works' ground in squares, `held` what it holds,
/// `n` the squares `share_out` gave it.
pub fn keep_squares(x: &[f64], held: &[usize], n: &mut [usize], keep: f64) {
    for j in 0..n.len() {
        if held[j] > n[j] && x[j] >= keep * held[j] as f64 {
            n[j] = held[j];
        }
    }
    let mut over = n.iter().sum::<usize>().saturating_sub(PER_CELL);
    if over == 0 {
        return;
    }
    let mut order: Vec<usize> = (0..n.len()).filter(|&j| n[j] as f64 > x[j].round()).collect();
    order.sort_by(|&a, &b| (n[b] as f64 - x[b]).total_cmp(&(n[a] as f64 - x[a])).then(a.cmp(&b)));
    for j in order {
        let floor = (x[j].round() as usize).min(n[j]);
        let back = (n[j] - floor).min(over);
        n[j] -= back;
        over -= back;
        if over == 0 {
            break;
        }
    }
}

/// Squares each working should hold this year, from the books, within what the works leave.
pub fn pit_targets(st: &State, works_on: &[usize]) -> BTreeMap<(usize, usize), usize> {
    let mut want = BTreeMap::new();
    let R = st.mine_sectors.len();
    if st.mine_surface.rows < R || R == 0 {
        return want;
    }
    let N = st.N;
    for c in 0..N {
        let rs: Vec<usize> = (0..R).filter(|&r| st.mine_surface[(r, c)] > 0.0).collect();
        if rs.is_empty() {
            continue;
        }
        let x: Vec<f64> = rs.iter().map(|&r| st.mine_surface[(r, c)] * PER_CELL as f64).collect();
        let room = PER_CELL.saturating_sub(works_on[c]);
        let sum: f64 = x.iter().sum();
        let scale = if sum > room as f64 { room as f64 / sum } else { 1.0 };
        let x: Vec<f64> = x.iter().map(|v| v * scale).collect();
        let total = ((sum * scale).round() as usize).min(room);
        let held: Vec<usize> = rs.iter().map(|&r| st.sq.pits.get(&(r, c)).map_or(0, |h| h.squares.len())).collect();
        for (j, n) in share_out(&x, &held, total).into_iter().enumerate() {
            want.insert((rs[j], c), n);
        }
    }
    want
}

/// The squares of a town cell's ground that are park and house blocks, where
/// `ground` squares are the town's (the cell less its works and pits) and
/// `dry` of them are not under water. The books count a cell as 4 km² of land
/// whatever the sea takes of it; the houses and the park are reckoned on the
/// dry ground, so that a town on the shore does not build in the sea. There
/// are never fewer house blocks than `least`, the blocks the floor let to
/// works fills (works.in_blocks), while the ground holds them.
pub fn town_split(fill: f64, park: f64, dry: usize, ground: usize, least: usize) -> (usize, usize) {
    let n_park = ((park * dry as f64).round() as usize).min(ground);
    let n_houses = ((fill * dry as f64).round() as usize).max(least).min(ground - n_park);
    (n_park, n_houses)
}

/// The eighths of a cell's house blocks' floor each use should have this year
/// (`FLOOR_USES`), from the books' ground of the works in its blocks
/// (`floored`), `want`: each works' ground in squares, times eight. With the
/// ground by method a use keeps the floor it holds while its ground stays at
/// `squares.keep` of it, as a works keeps its squares (`keep_squares`). `held`
/// is what each use holds now.
pub fn floor_targets(want: &[f64; 5], held: &[usize; 5], P: &Params) -> [usize; 5] {
    let mut n = [0usize; 5];
    for u in 1..5 {
        n[u] = want[u].max(0.0).round() as usize;
        if P.works.by_method && held[u] > n[u] && want[u] >= P.squares.keep * held[u] as f64 {
            n[u] = held[u];
        }
    }
    n
}

/// The eighths of floor each use holds in a table's house blocks.
fn floor_held(t: &Table) -> [usize; 5] {
    let mut held = [0usize; 5];
    for k in 0..PER_CELL {
        if matches!(Tenant::of(t.tenant[k]), Tenant::Houses(_)) {
            held[(t.floor[k] >> 4) as usize % 5] += (t.floor[k] & 0x0f) as usize;
        }
    }
    held
}

/// Lets the floor of one table's house blocks, `floor` eighths to each use: a
/// use that holds more gives back from its worst squares first; one that
/// holds less takes the best house blocks for a shop (beside the way, toward
/// the town's middle), shops `SHOP_EIGHTHS` of a block's floor while there
/// are blocks left and the other uses whole blocks. What does not fit is
/// counted in `short`.
fn let_floor(t: &mut Table, fr: &Frame, floor: &[usize; 5], short: &mut u64) {
    let mut memo = [f32::NAN; PER_CELL];
    let house = |t: &Table, k: usize| matches!(Tenant::of(t.tenant[k]), Tenant::Houses(_));
    for k in 0..PER_CELL {
        if !house(t, k) {
            t.floor[k] = 0;
        }
    }
    let mut held = floor_held(t);
    for u in 1..5u8 {
        let mut over = held[u as usize].saturating_sub(floor[u as usize]);
        if over == 0 {
            continue;
        }
        held[u as usize] -= over;
        let mut mine: Vec<(f32, usize)> = (0..PER_CELL).filter(|&k| t.floor[k] >> 4 == u && t.floor[k] & 0x0f > 0).map(|k| (scored(&mut memo, t, fr, k, Want::Shop), k)).collect();
        mine.sort_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
        for &(_, k) in &mine {
            let e = t.floor[k] & 0x0f;
            let back = (e as usize).min(over);
            t.floor[k] = if e as usize == back { 0 } else { u << 4 | (e - back as u8) };
            over -= back;
            if over == 0 {
                break;
            }
        }
    }
    for u in [2u8, 3, 4, 1] {
        let mut left = floor[u as usize].saturating_sub(held[u as usize]);
        for cap in [if u == 1 { SHOP_EIGHTHS } else { 8 }, 8] {
            if left == 0 {
                break;
            }
            // free blocks and its own first; a use other than the shops may take
            // a block whose street floor alone is shops', and the shops let
            // theirs again in their turn, which is last
            let shops_street = |t: &Table, k: usize| u != 1 && t.floor[k] >> 4 == 1 && t.floor[k] & 0x0f <= SHOP_EIGHTHS;
            // (its own blocks in part let first, so that a use fills its blocks)
            let mut open: Vec<(bool, bool, f32, usize)> = (0..PER_CELL)
                .filter(|&k| house(t, k) && (t.floor[k] == 0 || t.floor[k] >> 4 == u && t.floor[k] & 0x0f < cap || shops_street(t, k)))
                .map(|k| (shops_street(t, k), t.floor[k] >> 4 != u, scored(&mut memo, t, fr, k, Want::Shop), k))
                .collect();
            open.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(b.2.total_cmp(&a.2)).then(a.3.cmp(&b.3)));
            for &(street, _, _, k) in &open {
                if street {
                    held[1] -= (t.floor[k] & 0x0f) as usize;
                    t.floor[k] = 0;
                }
                let e = t.floor[k] & 0x0f;
                let add = ((cap - e) as usize).min(left);
                t.floor[k] = u << 4 | (e + add as u8);
                left -= add;
                if left == 0 {
                    break;
                }
            }
        }
        *short += left as u64;
    }
}

/// The house blocks the floor of `floor` eighths, by use, fills: each use's
/// in whole blocks, since a block's floor is let to one use.
pub fn floor_blocks(floor: &[usize; 5]) -> usize {
    floor.iter().skip(1).map(|e| e.div_ceil(8)).sum()
}

fn derelict_ok(st: &State, c: usize) -> bool {
    st.derelict_timer.get(c).is_some_and(|&t| t > 0) && !st.urban[c]
}

/// Takes `need` squares of cell c for a works or a pit, `tenant`, grown from
/// what it holds (`h.squares`), or from the best square for it where it holds
/// none. What it cannot have for want of room is counted in `SquareBook::short`.
fn grow(st: &mut State, h: &mut Holding, tenant: Tenant, want: Want, need: usize, year: i16) {
    let c = h.cell;
    let ti = table_of(st, c);
    let fr = frame(st, c);
    let code = tenant.code();
    let held = h.squares.len();
    let SquareBook { tables, split, short, took, .. } = &mut st.sq;
    let t = &mut tables[ti];
    // each square's score, worked out when first asked for (it reads the ground and the ways, not the tenants)
    let mut memo = [f32::NAN; PER_CELL];
    let mut mine = [false; PER_CELL];
    for &k in &h.squares {
        mine[k as usize] = true;
    }
    // the free squares beside what it holds, and what each is worth to it
    let mut near_mine = [false; PER_CELL];
    let mut cs = [0.0f32; PER_CELL];
    for &q in &h.squares {
        for j in beside(q as usize) {
            if !mine[j] && displace(t.tenant[j]).is_some() {
                near_mine[j] = true;
            }
        }
    }
    for k in 0..PER_CELL {
        if near_mine[k] {
            cs[k] = beside_score(&mut memo, t, &fr, &mine, k, want);
        }
    }
    for _ in 0..need {
        let pick = if h.squares.is_empty() {
            // where it first stands. The free ground of the cell falls into pieces
            // between the works and pits already there; it takes the smallest piece
            // that holds what it needs (so that the large pieces are kept for large
            // works), at its best square there, set against the works and pits
            // beside it rather than out in the open
            let mut piece = [0usize; PER_CELL];
            let mut size = vec![0usize];
            for k0 in 0..PER_CELL {
                if piece[k0] > 0 || displace(t.tenant[k0]).is_none() {
                    continue;
                }
                let id = size.len();
                size.push(0);
                let mut stack = vec![k0];
                piece[k0] = id;
                while let Some(k) = stack.pop() {
                    size[id] += 1;
                    let (x, y) = ((k % SIDE) as isize, (k / SIDE) as isize);
                    for (dx, dy) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                        let (a, b) = (x + dx, y + dy);
                        if a < 0 || b < 0 || a >= SIDE as isize || b >= SIDE as isize {
                            continue;
                        }
                        let j = b as usize * SIDE + a as usize;
                        if piece[j] == 0 && displace(t.tenant[j]).is_some() {
                            piece[j] = id;
                            stack.push(j);
                        }
                    }
                }
            }
            let largest = size.iter().copied().max().unwrap_or(0);
            let fits = need.min(largest);
            // squares (in steps) from each free square to the nearest works or pit
            let mut apart = [u8::MAX; PER_CELL];
            let mut front: Vec<usize> = (0..PER_CELL).filter(|&k| displace(t.tenant[k]).is_none()).collect();
            front.iter().for_each(|&k| apart[k] = 0);
            let mut step = 0u8;
            while !front.is_empty() && step < 8 {
                step += 1;
                let mut next = Vec::new();
                for &k in &front {
                    let (x, y) = ((k % SIDE) as isize, (k / SIDE) as isize);
                    for (dx, dy) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                        let (a, b) = (x + dx, y + dy);
                        if a < 0 || b < 0 || a >= SIDE as isize || b >= SIDE as isize {
                            continue;
                        }
                        let j = b as usize * SIDE + a as usize;
                        if apart[j] == u8::MAX {
                            apart[j] = step;
                            next.push(j);
                        }
                    }
                }
                front = next;
            }
            let reach = ((need as f32).sqrt().ceil() + 1.0).min(8.0);
            let mut best: Option<(f32, usize)> = None;
            for k in 0..PER_CELL {
                let Some(cost) = displace(t.tenant[k]) else { continue };
                let n = size[piece[k]];
                if n < fits {
                    continue;
                }
                let s = scored(&mut memo, t, &fr, k, want) - cost + 0.6 * (apart[k] as f32).min(reach) - 0.02 * (n - fits).min(150) as f32;
                if best.is_none_or(|(b, j)| s > b || (s == b && k < j)) {
                    best = Some((s, k));
                }
            }
            best.map(|(_, k)| k)
        } else {
            // beside its own: the best free neighbour, the more of its own beside it the better
            let mut best: Option<(f32, usize)> = None;
            for k in 0..PER_CELL {
                if near_mine[k] && best.is_none_or(|(b, _)| cs[k] > b) {
                    best = Some((cs[k], k));
                }
            }
            match best {
                Some((_, k)) => Some(k),
                None => {
                    // hemmed in by other works and pits on every side: an annex, on the free
                    // square nearest what it holds (across a neighbour's yard, as a works
                    // that cannot grow where it stands takes ground over the way)
                    let near = |k: usize| {
                        let (x, y) = ((k % SIDE) as f32, (k / SIDE) as f32);
                        h.squares.iter().map(|&q| ((q as usize % SIDE) as f32 - x).hypot((q as usize / SIDE) as f32 - y)).fold(f32::INFINITY, f32::min)
                    };
                    let k = (0..PER_CELL)
                        .filter(|&k| !mine[k] && displace(t.tenant[k]).is_some())
                        .map(|k| (scored(&mut memo, t, &fr, k, want) - displace(t.tenant[k]).unwrap() - 2.0 * near(k), k))
                        .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)))
                        .map(|(_, k)| k);
                    if k.is_some() {
                        *split += 1;
                        h.annexes += 1;
                    }
                    k
                }
            }
        };
        let Some(k) = pick else {
            *short += 1;
            continue;
        };
        if h.squares.is_empty() {
            let (face, along) = fr.face(k);
            h.face = face;
            h.along = along;
            h.year = year;
            h.annexes = 0;
        }
        t.tenant[k] = code;
        t.year[k] = year;
        t.face[k] = fr.face(k).0;
        mine[k] = true;
        near_mine[k] = false;
        h.squares.push(k as u8);
        // the squares beside it are open to it now, and those it was beside have one more of its own
        for j in beside(k) {
            if !mine[j] && displace(t.tenant[j]).is_some() {
                near_mine[j] = true;
                cs[j] = beside_score(&mut memo, t, &fr, &mine, j, want);
            }
        }
    }
    // the holdings' log: what it took, in order
    if h.squares.len() > held {
        let (pit, id) = match tenant {
            Tenant::Pit(r) => (true, r),
            Tenant::Works(f) => (false, f),
            _ => (false, 0),
        };
        took.push(Took { cell: c as u32, pit, id, fresh: held == 0, along: h.along, squares: h.squares[held..].to_vec() });
    }
}

/// The squares that share a side with square k.
fn beside(k: usize) -> impl Iterator<Item = usize> {
    let (x, y) = ((k % SIDE) as isize, (k / SIDE) as isize);
    [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)].into_iter().filter_map(move |(dx, dy)| {
        let (a, b) = (x + dx, y + dy);
        (a >= 0 && b >= 0 && a < SIDE as isize && b < SIDE as isize).then(|| b as usize * SIDE + a as usize)
    })
}

/// What free square k is worth to a holding (`mine`) growing beside it: its
/// score, less what it costs to take, more for each side its own squares hold
/// and less for each another works or a pit holds.
fn beside_score(memo: &mut [f32; PER_CELL], t: &Table, fr: &Frame, mine: &[bool; PER_CELL], k: usize, want: Want) -> f32 {
    let (mut own, mut others) = (0.0f32, 0.0f32);
    for j in beside(k) {
        if mine[j] {
            own += 1.0;
        } else if displace(t.tenant[j]).is_none() {
            others += 1.0;
        }
    }
    scored(memo, t, fr, k, want) - displace(t.tenant[k]).unwrap_or(0.0) + 1.5 * own - 0.8 * others
}

fn allot(st: &mut State, C: &Content, P: &Params, year: i64, opening: bool) {
    let N = st.N;
    let y16 = year.clamp(i16::MIN as i64, i16::MAX as i64) as i16;
    let F = st.F_count;
    if st.sq.works.len() < F {
        st.sq.works.resize(F, Holding::default());
    }

    // what the books say each works and each working holds
    let want = works_targets(st, C, P);
    let mut works_on = vec![0usize; N];
    for f in 0..F {
        if st.f_alive[f] {
            works_on[st.f_cell[f]] += want[f];
        }
    }
    let pit_want = pit_targets(st, &works_on);

    // 1. what is given back: the squares taken last go first
    for f in 0..F {
        let target = if st.f_alive[f] { want[f] } else { 0 };
        if st.sq.works[f].squares.len() <= target {
            continue;
        }
        let c = st.sq.works[f].cell;
        let left = if st.f_alive[f] { Tenant::None } else if derelict_ok(st, c) { Tenant::Derelict(f as u32) } else { Tenant::None };
        let ti = st.sq.index[c] as usize - 1;
        while st.sq.works[f].squares.len() > target {
            let k = st.sq.works[f].squares.pop().unwrap() as usize;
            let t = &mut st.sq.tables[ti];
            t.tenant[k] = left.code();
            t.year[k] = if left == Tenant::None { 0 } else { y16 };
        }
    }
    let pit_keys: Vec<(usize, usize)> = st.sq.pits.keys().copied().collect();
    for key in pit_keys {
        let target = pit_want.get(&key).copied().unwrap_or(0);
        let h = st.sq.pits.get_mut(&key).unwrap();
        if h.squares.len() <= target {
            continue;
        }
        let ti = st.sq.index[key.1] as usize - 1;
        while h.squares.len() > target {
            let k = h.squares.pop().unwrap() as usize;
            st.sq.tables[ti].tenant[k] = 0;
            st.sq.tables[ti].year[k] = 0;
        }
        if h.squares.is_empty() {
            st.sq.pits.remove(&key);
        }
    }

    // 2. what is taken: the works, those that take most first (so that a large
    // works finds its ground in one piece before the small ones scatter over
    // it), then the workings
    let mut growing: Vec<usize> = (0..F).filter(|&f| st.f_alive[f] && want[f] > st.sq.works[f].squares.len()).collect();
    growing.sort_by_key(|&f| (std::cmp::Reverse(want[f] - st.sq.works[f].squares.len()), f));
    for f in growing {
        let held = st.sq.works[f].squares.len();
        let mut h = std::mem::take(&mut st.sq.works[f]);
        h.cell = st.f_cell[f];
        let taken = if opening { st.f_born[f].clamp(i16::MIN as i64, i16::MAX as i64) as i16 } else { y16 };
        grow(st, &mut h, Tenant::Works(f as u32), works_want(C, st.f_method[f]), want[f] - held, taken);
        st.sq.works[f] = h;
    }
    for (&(r, c), &n) in &pit_want {
        let held = st.sq.pits.get(&(r, c)).map_or(0, |h| h.squares.len());
        if n <= held {
            continue;
        }
        let mut h = st.sq.pits.remove(&(r, c)).unwrap_or(Holding { cell: c, ..Holding::default() });
        let opened = st.mine_opened.get(r * N + c).copied().unwrap_or(0) as i64;
        let taken = if opening && opened != 0 { opened.clamp(i16::MIN as i64, i16::MAX as i64) as i16 } else { y16 };
        grow(st, &mut h, Tenant::Pit(r as u32), Want::Pit, n - held, taken);
        if !h.squares.is_empty() {
            st.sq.pits.insert((r, c), h);
        }
    }

    // 3. town ground: every town cell is the town's, less its works and pits.
    // The works in its blocks (works.in_blocks) let floor in its house blocks:
    // what each use wants of it, in eighths of a square
    let mut floor_want: BTreeMap<usize, [f64; 5]> = BTreeMap::new();
    for f in 0..F {
        if st.f_alive[f] && floored(st, C, P, f) {
            let w = floor_want.entry(st.f_cell[f]).or_insert([0.0; 5]);
            w[floor_use(C, st.f_method[f]) as usize] += crate::works::land_held(st, C, P, f) * (PER_CELL * 8) as f64;
        }
    }
    let mut town_of = vec![-1i64; N];
    for (i, list) in st.town_ground.iter().enumerate() {
        for &c in list {
            if st.urban.get(c).copied().unwrap_or(false) {
                town_of[c] = i as i64;
            }
        }
    }
    for c in 0..N {
        if town_of[c] >= 0 {
            table_of(st, c);
        }
    }
    for ti in 0..st.sq.tables.len() {
        let c = st.sq.tables[ti].cell;
        if st.sq.index[c] as usize != ti + 1 {
            continue;
        }
        let i = town_of[c];
        if i < 0 {
            // no town's: its town ground goes back to the country, and derelict ground
            // the books no longer call derelict goes with it
            let keep_derelict = derelict_ok(st, c);
            let t = &mut st.sq.tables[ti];
            for k in 0..PER_CELL {
                let tn = Tenant::of(t.tenant[k]);
                if tn.town().is_some() || (matches!(tn, Tenant::Derelict(_)) && !keep_derelict) {
                    t.tenant[k] = 0;
                    t.year[k] = 0;
                }
            }
            t.houses.clear();
            t.park.clear();
            t.floor = [0; PER_CELL];
            continue;
        }
        let i = i as u32;
        let built = st.town_built.get(c).copied().unwrap_or(year);
        let taken = if opening { built.clamp(i16::MIN as i64, i16::MAX as i64) as i16 } else { y16 };
        let fill = crate::ground::built_share(st, C, P, year, c);
        let park = crate::ground::park_of(st, P, c);
        let fr = frame(st, c);
        let t = &mut st.sq.tables[ti];
        // the town's ground: what no works or pit holds
        let mut ground = 0usize;
        // (and how much of it is dry: the houses and the park are reckoned on that)
        let dry = (0..PER_CELL).filter(|&k| !t.water[k] && !matches!(Tenant::of(t.tenant[k]), Tenant::Works(_) | Tenant::Pit(_))).count();
        for k in 0..PER_CELL {
            match Tenant::of(t.tenant[k]) {
                Tenant::Works(_) | Tenant::Pit(_) => {}
                Tenant::Houses(_) => {
                    t.tenant[k] = Tenant::Houses(i).code();
                    ground += 1;
                }
                Tenant::Park(_) => {
                    t.tenant[k] = Tenant::Park(i).code();
                    ground += 1;
                }
                Tenant::Open(_) => {
                    t.tenant[k] = Tenant::Open(i).code();
                    ground += 1;
                }
                Tenant::None | Tenant::Derelict(_) => {
                    t.tenant[k] = Tenant::Open(i).code();
                    t.year[k] = taken;
                    ground += 1;
                }
            }
        }
        // the house blocks and park the works have taken are gone from their lists
        let still = |t: &Table, k: u8, kind: fn(u32) -> Tenant| t.tenant[k as usize] == kind(i).code();
        let houses: Vec<u8> = t.houses.iter().copied().filter(|&k| still(t, k, Tenant::Houses)).collect();
        let parks: Vec<u8> = t.park.iter().copied().filter(|&k| still(t, k, Tenant::Park)).collect();
        t.houses = houses;
        t.park = parks;
        // the floor each use should have, and the house blocks that takes;
        // where the ground holds fewer blocks than the floor fills, each use is cut in proportion
        let mut floor = floor_targets(&floor_want.get(&c).copied().unwrap_or([0.0; 5]), &floor_held(t), P);
        let all: usize = floor.iter().sum();
        let (n_park, n_houses) = town_split(fill, park, dry, ground, floor_blocks(&floor));
        if floor_blocks(&floor) > n_houses {
            // (leaving each use a block it may fill in part)
            let uses = floor.iter().skip(1).filter(|&&e| e > 0).count();
            let room = 8 * n_houses.saturating_sub(uses);
            for u in 1..5 {
                floor[u] = floor[u] * room / all;
            }
            st.sq.short += (all - floor.iter().sum::<usize>()) as u64;
        }
        for (list, n) in [(0, n_park), (1, n_houses)] {
            let lst = if list == 0 { &mut t.park } else { &mut t.houses };
            while lst.len() > n {
                let k = lst.pop().unwrap() as usize;
                t.tenant[k] = Tenant::Open(i).code();
                t.year[k] = y16;
            }
        }
        for (want, n) in [(Want::Park, n_park), (Want::Houses, n_houses)] {
            let have = if want == Want::Park { t.park.len() } else { t.houses.len() };
            if n <= have {
                continue;
            }
            let mut open: Vec<(f32, usize)> =
                (0..PER_CELL).filter(|&k| t.tenant[k] == Tenant::Open(i).code()).map(|k| (score(t, &fr, k, want), k)).collect();
            open.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
            let code = if want == Want::Park { Tenant::Park(i).code() } else { Tenant::Houses(i).code() };
            for &(_, k) in open.iter().take(n - have) {
                t.tenant[k] = code;
                t.year[k] = taken;
                t.face[k] = fr.face(k).0;
                if want == Want::Park { t.park.push(k as u8) } else { t.houses.push(k as u8) }
            }
        }
        let_floor(t, &fr, &floor, &mut st.sq.short);
    }

    // 4. a cell with nothing built keeps no table
    for ti in 0..st.sq.tables.len() {
        let c = st.sq.tables[ti].cell;
        if st.sq.index[c] as usize == ti + 1 && st.sq.tables[ti].tenant.iter().all(|&x| x == 0) {
            st.sq.index[c] = 0;
            st.sq.spare[c] = ti as u32 + 1;
            let t = &mut st.sq.tables[ti];
            t.houses.clear();
            t.park.clear();
            t.year = [0; PER_CELL];
        }
    }
    st.sq.year = year;
}

// ---------------------------------------------------------------- the log in the record

/// What the record's header says of its squares (`record::pack`), so that a
/// reader knows how to read `sq_log`; a later layout gets a higher number.
/// Layout 2 (works.in_blocks) carries each square's floor let to works.
/// Layout 3 (courses.rs) has the same `sq_log` and beside it the holdings'
/// log, `sq_hold` (`encode_holds`), so a reader of layout 2 reads its `sq_log`.
pub const LOG_LAYOUT: u32 = 3;
/// The layouts whose `sq_log` a reader of this one reads as its own.
pub const LOG_READS: [u32; 2] = [2, 3];

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
        let Some(&b) = bytes.get(*i) else { return Err("the squares' log ends in the middle of a number".into()) };
        *i += 1;
        v |= ((b & 0x7f) as u64) << shift;
        if b < 0x80 {
            return Ok(v);
        }
    }
    Err("the squares' log has a number too long to be one".into())
}

/// One year's changes as the record writes them, in bytes (`sq_log`; the
/// changes in any order, each square once). Cell by cell in rising order,
/// each cell as: the cell less the one before it in the year (the first less
/// 0), then the number of runs, then each run of changes that give its
/// squares to one tenant with one year and one floor: a head,
/// `((id << 3 | kind) << 1 | dated) << 1 | floored` with `id` and `kind` as
/// in `Tenant::code` (0 for squares given up), then the year less the log's
/// year, zigzagged (`2y` for y >= 0, `-2y - 1` below), where `dated` is 1,
/// then the floor (`Table::floor`) as one byte where `floored` is 1, then the
/// number of squares, then each square: its place in the cell (`k`), and its
/// face but where it was given up. Every number but the squares, faces and
/// floor, which are single bytes, is a varint of seven bits a byte, low
/// first. A year's ordinary change, a square taken for a tenant that takes
/// others in its cell that year, is two bytes.
pub fn encode_year(changes: &[Change], year: i64) -> Vec<u8> {
    // a square given up has no year and no floor
    let key = |c: &Change| (c.cell(), c.tenant, if c.tenant == 0 { 0 } else { c.year }, if c.tenant == 0 { 0 } else { c.floor }, c.at);
    let mut sorted: Vec<Change> = changes.to_vec();
    sorted.sort_unstable_by_key(key);
    let mut out = Vec::with_capacity(sorted.len() * 2 + 16);
    let mut prev = 0usize;
    let mut i = 0;
    while i < sorted.len() {
        let cell = sorted[i].cell();
        let end = i + sorted[i..].iter().take_while(|c| c.cell() == cell).count();
        let run_of = |c: &Change| if c.tenant == 0 { (0, 0, 0) } else { (c.tenant, c.year, c.floor) };
        let runs = 1 + (i + 1..end).filter(|&j| run_of(&sorted[j]) != run_of(&sorted[j - 1])).count();
        put_varint(&mut out, (cell - prev) as u64);
        prev = cell;
        put_varint(&mut out, runs as u64);
        while i < end {
            let (tenant, y, floor) = run_of(&sorted[i]);
            let n = sorted[i..end].iter().take_while(|c| run_of(c) == (tenant, y, floor)).count();
            let dated = tenant != 0 && y as i64 != year;
            let floored = floor != 0;
            let head = ((((((tenant >> KIND_SHIFT) as u64) | ((tenant & ID_MASK) as u64) << 3) << 1) | dated as u64) << 1) | floored as u64;
            put_varint(&mut out, head);
            if dated {
                let d = y as i64 - year;
                put_varint(&mut out, if d >= 0 { 2 * d as u64 } else { (-2 * d - 1) as u64 });
            }
            if floored {
                out.push(floor);
            }
            put_varint(&mut out, n as u64);
            for c in &sorted[i..i + n] {
                out.push(c.square() as u8);
                if tenant != 0 {
                    out.push(c.face);
                }
            }
            i += n;
        }
    }
    out
}

/// The changes of one year of the log (`encode_year`), by `Change::at`, or
/// what is wrong with the bytes.
pub fn decode_year(bytes: &[u8], year: i64) -> Result<Vec<Change>, String> {
    let mut out = Vec::new();
    let (mut i, mut cell) = (0usize, 0u64);
    while i < bytes.len() {
        cell += get_varint(bytes, &mut i)?;
        let runs = get_varint(bytes, &mut i)?;
        for _ in 0..runs {
            let head = get_varint(bytes, &mut i)?;
            let (floored, head) = (head & 1 == 1, head >> 1);
            let (kind, id) = (((head >> 1) & 7) as u32, head >> 4);
            if kind > 6 || id > ID_MASK as u64 {
                return Err(format!("the squares' log names a tenant of kind {kind} and number {id}, which is none"));
            }
            let tenant = if kind == 0 { 0 } else { (kind << KIND_SHIFT) | id as u32 };
            let y = if head & 1 == 1 {
                let z = get_varint(bytes, &mut i)?;
                year + if z % 2 == 0 { (z / 2) as i64 } else { -((z / 2) as i64) - 1 }
            } else {
                year
            };
            let floor = if floored {
                let Some(&b) = bytes.get(i) else { return Err("the squares' log ends in the middle of a run's floor".into()) };
                i += 1;
                b
            } else {
                0
            };
            let n = get_varint(bytes, &mut i)?;
            for _ in 0..n {
                let step = if tenant == 0 { 1 } else { 2 };
                let Some(b) = bytes.get(i..i + step) else { return Err("the squares' log ends in the middle of a run".into()) };
                i += step;
                let at = cell.checked_mul(PER_CELL as u64).map(|a| a + b[0] as u64).filter(|&a| a <= u32::MAX as u64);
                let Some(at) = at else { return Err(format!("the squares' log names cell {cell}, past any map")) };
                out.push(if tenant == 0 {
                    Change { at: at as u32, tenant: 0, year: 0, face: 0, floor: 0 }
                } else {
                    Change { at: at as u32, tenant, year: y.clamp(i16::MIN as i64, i16::MAX as i64) as i16, face: b[1], floor }
                });
            }
        }
    }
    out.sort_unstable_by_key(|c| c.at);
    Ok(out)
}

// ---------------------------------------------------------------- reading the book

/// Squares held, by kind, over the island.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Census {
    pub tables: usize,
    pub works: usize,
    pub houses: usize,
    pub park: usize,
    pub pit: usize,
    pub derelict: usize,
    pub open: usize,
    /// Squares under the sea or a lake that something holds, and those of them
    /// held by works, house blocks or pits (not open ground).
    pub under_water: usize,
    pub built_under_water: usize,
    /// Works alive, and those that hold no square of their own (under half a square).
    pub works_alive: usize,
    pub works_without: usize,
    /// Works in more than one piece.
    pub works_split: usize,
    pub split: u64,
    pub short: u64,
    /// House blocks with floor let to works (works.in_blocks), and the eighths
    /// of a square's floor let, by use (`FLOOR_USES`).
    pub floored: usize,
    pub floor: [usize; 5],
}

pub fn census(st: &State) -> Census {
    let b = &st.sq;
    let mut n = Census { tables: b.index.iter().filter(|&&k| k > 0).count(), split: b.split, short: b.short, ..Census::default() };
    for (c, &k) in b.index.iter().enumerate() {
        if k == 0 {
            continue;
        }
        let t = &b.tables[k as usize - 1];
        debug_assert_eq!(t.cell, c);
        for q in 0..PER_CELL {
            let tn = Tenant::of(t.tenant[q]);
            match tn {
                Tenant::None => continue,
                Tenant::Works(_) => n.works += 1,
                Tenant::Houses(_) => {
                    n.houses += 1;
                    if t.floor[q] & 0x0f > 0 {
                        n.floored += 1;
                        n.floor[(t.floor[q] >> 4) as usize % 5] += (t.floor[q] & 0x0f) as usize;
                    }
                }
                Tenant::Park(_) => n.park += 1,
                Tenant::Pit(_) => n.pit += 1,
                Tenant::Derelict(_) => n.derelict += 1,
                Tenant::Open(_) => n.open += 1,
            }
            if t.water[q] {
                n.under_water += 1;
                if matches!(tn, Tenant::Works(_) | Tenant::Houses(_) | Tenant::Pit(_)) {
                    n.built_under_water += 1;
                }
            }
        }
    }
    for f in 0..st.F_count {
        if st.f_alive[f] {
            n.works_alive += 1;
            let h = b.works.get(f);
            if h.is_none_or(|h| h.squares.is_empty()) {
                n.works_without += 1;
            } else if pieces(&h.unwrap().squares) > 1 {
                n.works_split += 1;
            }
        }
    }
    n
}

/// How many pieces a set of squares of one cell is in (squares that share a side are one piece).
pub fn pieces(squares: &[u8]) -> usize {
    let mut mine = [false; PER_CELL];
    for &k in squares {
        mine[k as usize] = true;
    }
    let mut seen = [false; PER_CELL];
    let mut n = 0;
    for &k0 in squares {
        if seen[k0 as usize] {
            continue;
        }
        n += 1;
        let mut stack = vec![k0 as usize];
        seen[k0 as usize] = true;
        while let Some(k) = stack.pop() {
            let (x, y) = ((k % SIDE) as isize, (k / SIDE) as isize);
            for (dx, dy) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
                let (a, b) = (x + dx, y + dy);
                if a < 0 || b < 0 || a >= SIDE as isize || b >= SIDE as isize {
                    continue;
                }
                let j = b as usize * SIDE + a as usize;
                if mine[j] && !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
    }
    n
}

/// Every rule of the book, checked against the books as they stand at the
/// year's end: what a test asks of every year. Returns what is wrong, if anything.
pub fn check(st: &State, C: &Content, P: &Params, year: i64) -> Result<(), String> {
    let b = &st.sq;
    if !b.kept() {
        return Err("no book is kept".into());
    }
    let N = st.N;
    // the tables say what the holdings say, and no square has two tenants
    let mut claimed = vec![0u8; 0];
    claimed.resize(b.tables.len() * PER_CELL, 0);
    let mut works_on = vec![0usize; N];
    // each cell's works share as the books have it
    let mut ks = vec![0.0f64; N];
    // and the floor its house blocks let to works in them (works.in_blocks), by use
    let mut floor_want: BTreeMap<usize, [f64; 5]> = BTreeMap::new();
    for f in 0..st.F_count {
        if st.f_alive[f] && floored(st, C, P, f) {
            let w = floor_want.entry(st.f_cell[f]).or_insert([0.0; 5]);
            w[floor_use(C, st.f_method[f]) as usize] += crate::works::land_held(st, C, P, f) * (PER_CELL * 8) as f64;
        } else if st.f_alive[f] {
            ks[st.f_cell[f]] += crate::works::land_held(st, C, P, f);
        }
    }
    for f in 0..st.F_count.min(b.works.len()) {
        let h = &b.works[f];
        if h.squares.is_empty() {
            continue;
        }
        if !st.f_alive[f] {
            return Err(format!("{year}: works {f} is closed and holds {} squares", h.squares.len()));
        }
        if floored(st, C, P, f) {
            return Err(format!("{year}: works {f} stands in the blocks of its town and holds {} squares", h.squares.len()));
        }
        if h.cell != st.f_cell[f] {
            return Err(format!("{year}: works {f} holds squares of cell {} and stands on {}", h.cell, st.f_cell[f]));
        }
        let Some(t) = b.table(h.cell) else { return Err(format!("{year}: works {f} holds squares of cell {}, which has no table", h.cell)) };
        let ti = b.index[h.cell] as usize - 1;
        for &k in &h.squares {
            if Tenant::of(t.tenant[k as usize]) != Tenant::Works(f as u32) {
                return Err(format!("{year}: square {k} of cell {} is {:?}, not works {f}'s", h.cell, Tenant::of(t.tenant[k as usize])));
            }
            claimed[ti * PER_CELL + k as usize] += 1;
        }
        works_on[h.cell] += h.squares.len();
        if pieces(&h.squares) > 1 + h.annexes as usize {
            return Err(format!("{year}: works {f}'s {} squares on cell {} are in {} pieces", h.squares.len(), h.cell, pieces(&h.squares)));
        }
        // a works holds its share of the cell to within a square
        let x = crate::works::land_held(st, C, P, f) * PER_CELL as f64;
        let n = h.squares.len() as f64;
        // (a cell over-full is scaled down), or keeps more while its ground in the
        // books is at `squares.keep` of what it holds (works.by_method)
        let kept = P.works.enabled && P.works.by_method && n > x && x >= P.squares.keep * n - 1e-9;
        if (n - x).abs() >= 1.0 + 1e-9 && !kept && ks[h.cell] <= 1.0 && b.short == 0 {
            return Err(format!("{year}: works {f} holds {n} squares for {x:.2} of ground"));
        }
    }
    for (&(r, c), h) in &b.pits {
        let Some(t) = b.table(c) else { return Err(format!("{year}: the working {r} of cell {c} has no table")) };
        let ti = b.index[c] as usize - 1;
        for &k in &h.squares {
            if Tenant::of(t.tenant[k as usize]) != Tenant::Pit(r as u32) {
                return Err(format!("{year}: square {k} of cell {c} is not the working {r}'s"));
            }
            claimed[ti * PER_CELL + k as usize] += 1;
        }
        if pieces(&h.squares) > 1 + h.annexes as usize {
            return Err(format!("{year}: the working {r} of cell {c} is in {} pieces", pieces(&h.squares)));
        }
    }
    for (ti, t) in b.tables.iter().enumerate() {
        if b.index[t.cell] as usize != ti + 1 {
            continue;
        }
        let c = t.cell;
        let mut town = 0usize;
        let mut kinds = [0usize; 7];
        for k in 0..PER_CELL {
            let tn = Tenant::of(t.tenant[k]);
            if t.floor[k] != 0 && (!matches!(tn, Tenant::Houses(_)) || t.floor[k] & 0x0f > 8 || t.floor[k] >> 4 > 4 || t.floor[k] & 0x0f == 0) {
                return Err(format!("{year}: square {k} of cell {c} is {tn:?} with floor {:#04x}", t.floor[k]));
            }
            let held = matches!(tn, Tenant::Works(_) | Tenant::Pit(_));
            if held != (claimed[ti * PER_CELL + k] == 1) || claimed[ti * PER_CELL + k] > 1 {
                return Err(format!("{year}: square {k} of cell {c} is {tn:?} and held by {} holdings", claimed[ti * PER_CELL + k]));
            }
            kinds[(t.tenant[k] >> KIND_SHIFT) as usize] += 1;
            if let Some(i) = tn.town() {
                town += 1;
                if !st.urban[c] || st.catch[c] != i as i64 && !st.town_ground[i as usize].contains(&c) {
                    return Err(format!("{year}: square {k} of cell {c} is town {i}'s ground and the cell is not"));
                }
            }
            if matches!(tn, Tenant::Derelict(_)) && !derelict_ok(st, c) {
                return Err(format!("{year}: square {k} of cell {c} is derelict and the cell is not"));
            }
        }
        if kinds.iter().skip(1).sum::<usize>() == 0 {
            return Err(format!("{year}: cell {c} keeps a table with nothing on it"));
        }
        // the works' squares come to the cell's works share within a square, or
        // as far over it as works keep squares while their ground ebbs
        // (works.by_method, squares.keep)
        let share = ks[c].min(1.0);
        let over = if P.works.enabled && P.works.by_method { (share * PER_CELL as f64 / P.squares.keep).min(PER_CELL as f64) } else { share * PER_CELL as f64 };
        let n = works_on[c] as f64;
        if (n < share * PER_CELL as f64 - 1.0 || n > over + 1.0) && b.short == 0 {
            return Err(format!("{year}: cell {c} has {} squares of works for {share:.4} of the cell", works_on[c]));
        }
        // a pit's squares come to its surface within a square
        let ms: f64 = (0..st.mine_sectors.len()).filter(|_| st.mine_surface.rows > 0).map(|r| st.mine_surface[(r, c)]).sum();
        let pits = kinds[4];
        if ms * PER_CELL as f64 <= (PER_CELL - works_on[c]) as f64 && (pits as f64 - ms * PER_CELL as f64).abs() > 1.0 && b.short == 0 {
            return Err(format!("{year}: cell {c} has {pits} squares of pits for {ms:.3} of the cell"));
        }
        if st.urban[c] {
            // a town takes the cell whole, less its works and pits
            if town + kinds[1] + kinds[4] != PER_CELL {
                return Err(format!("{year}: town cell {c} has {town} squares of town ground beside {} of works and {} of pits", kinds[1], kinds[4]));
            }
            let fill = crate::ground::built_share(st, C, P, year, c);
            let park = crate::ground::park_of(st, P, c);
            let dry = (0..PER_CELL).filter(|&k| !t.water[k] && Tenant::of(t.tenant[k]).town().is_some()).count();
            // the floor let to the works in its blocks: each use its ground in
            // eighths of a square, within an eighth, or kept within `squares.keep`
            let held = floor_held(t);
            let want = floor_want.get(&c).copied().unwrap_or([0.0; 5]);
            for u in 1..5 {
                let (h, w) = (held[u] as f64, want[u]);
                let kept = P.works.by_method && h > w && w >= P.squares.keep * h - 1e-9;
                if (h - w).abs() >= 1.0 + 1e-9 && !kept && b.short == 0 {
                    return Err(format!("{year}: town cell {c} lets {h} eighths of floor to {} for {w:.2}", FLOOR_USES[u]));
                }
            }
            let let_out: usize = held.iter().sum();
            let (n_park, n_houses) = town_split(fill, park, dry, town, floor_blocks(&held));
            let fills = (fill * dry as f64).round() as usize >= n_houses;
            // (where the floor wanted more blocks than the ground holds, every
            // block of the ground is built and the floor cut to fit, to the eighth)
            let cut = kinds[2] == town - n_park && kinds[2] > n_houses;
            // (the blocks are counted from the floor wanted, which may be an
            // eighth or two more than the floor let where the shops' street
            // floor is spread thin: a block more)
            let spread = let_out > 0 && kinds[2] == n_houses + 1;
            if kinds[2] != n_houses && !cut && !spread || fills && (fill * dry as f64 - n_houses as f64).abs() > 1.0 && n_houses + n_park < town {
                return Err(format!("{year}: town cell {c} has {} squares of houses for {fill:.3} of {dry} dry ({n_houses} by the floor of {let_out} eighths let, {town} of town ground, {n_park} park)", kinds[2]));
            }
            if kinds[3] != n_park || (park * dry as f64 - n_park as f64).abs() > 1.0 {
                return Err(format!("{year}: town cell {c} has {} squares of park for {park:.3} of {dry} dry", kinds[3]));
            }
        } else if town > 0 {
            return Err(format!("{year}: cell {c} is no town's and has {town} squares of town ground"));
        }
    }
    // a cell with no table has nothing the books count in whole squares
    for c in 0..N {
        if b.index[c] == 0 && (ks[c].min(1.0) * PER_CELL as f64 > 1.0 || st.urban[c] || floor_want.contains_key(&c)) {
            return Err(format!("{year}: cell {c} keeps no table and holds {:.4} of works{}", ks[c], if st.urban[c] { " and is a town's" } else { "" }));
        }
    }
    // every works holds its squares. What a works keeps while its ground ebbs
    // (works.by_method) hangs on what it held before the year's allotting, so
    // the targets reckoned again from what it holds after can differ where a
    // cell is full; with that rule the check of each works against its ground,
    // above, is the one that holds
    if P.works.enabled && P.works.by_method {
        return Ok(());
    }
    let want = works_targets(st, C, P);
    for f in 0..st.F_count {
        if st.f_alive[f] {
            let n = b.works.get(f).map_or(0, |h| h.squares.len());
            if n != want[f] && b.short == 0 {
                return Err(format!("{year}: works {f} holds {n} squares and its ground comes to {}", want[f]));
            }
        }
    }
    Ok(())
}
