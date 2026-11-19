//! Crossings (not in the Python; see docs/rules/crossings.md): one bank, one
//! bridge.
//!
//! Under the port a river that is not navigable costs a road nothing to cross,
//! so the cheapest way wanders over it as often as it likes, and the viewer
//! puts a bridge wherever its drawn road meets its drawn river. With this rule
//! on, and on the new land only (`land.enabled`, whose fine rivers say where
//! the water is), crossing water is a thing that costs, that is built once by
//! somebody, and that everybody then uses:
//!
//! - **A table of crossings.** Every two neighbouring land cells are joined by
//!   a way drawn straight between their meeting places (the spot where a
//!   cell's roads meet, which the viewer draws its roads from: `meeting_places`,
//!   the same code, so the two agree to the last bit). Wherever that line
//!   crosses the line of a river carrying a cubic metre a second or more
//!   (every river the viewer draws), the way crosses water there, as wide as
//!   the river is at that point. A way between two cells is a *link*; a link
//!   may cross no water, one river, or the same river two or three times
//!   where its straight line cuts a meander.
//! - **A ford or a ferry costs.** A link over water with no bridge costs
//!   `ford` more to pass in the cost graph for each metre of water forded,
//!   and a river wider than `ford_width` cannot be forded at all: it is
//!   crossed by a ferry, at `ferry`. So the cheapest way bends to keep to one
//!   bank and to use what bridges there are. The charge is carting time, paid
//!   to the carters as every other unit of the graph is.
//! - **A bridge is built once by the parish.** Each year, after the roads have
//!   measured the year's traffic, each link over water that is not bridged is
//!   priced: `bridge_stone` a metre of water, bought at the market of the town
//!   whose district holds it, with `wages` on every shilling of stone paid to
//!   that town's men (the road menders of `roads.rs`, hands taken from its
//!   farms and works). Where the fords' or the ferry's charge on last year's
//!   traffic over the link would pay `hurdle` a year on that, the parish puts
//!   it on its list, best first, as far as a highway rate of `rate` of its
//!   landlords' cash goes (a quarter of that of its households' where the
//!   landlords are in debt). The work is a road works (`ways::ROAD`, serving
//!   `BRIDGE`); when its stone is all bought the bridge stands, for good, and
//!   the link costs nothing more to cross. The year it was built is kept.
//! - **The roads of 1750 are inherited with their bridges.** Where a made road
//!   or a turnpike crosses water in the first recorded year, its bridge was
//!   built by earlier centuries and stands; a track fords.
//!
//! Nothing else about the roads changes: traffic wears them as before
//! (`sim::update_roads`). On the old island, which has no fine rivers, the
//! rule does nothing at all: its rivers are the viewer's invention, and the
//! core cannot say where a drawn road would meet them.
//!
//! Every flow of money is between purses `econ::money_total` counts: the
//! parish's rate goes into the works' cash, and the works pays the market and
//! the town's households as every other works does.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::landcells::Landform;
use crate::params::Params;
use crate::state::{Event, Project, State};
use crate::transport as tr;
use crate::world::NEIGH8;

/// `Project::serves` of a bridge works (`roads.rs` uses 2, 3 and 9).
pub const BRIDGE: u8 = 4;

/// A cell's side in metres.
pub const CELL_M: f32 = 2000.0;

/// The smallest stream that is a crossing, in cubic metres a second of mean
/// flow: the smallest the viewer draws (its `newland::ground::MIN_FLOW`).
pub const MIN_FLOW: f32 = 1.0;

/// No bridge stands on a link.
pub const NOT_BUILT: i32 = i32::MAX;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CrossingParams {
    /// Crossing water costs, and bridges are built (on the new land only).
    pub enabled: bool,
    /// What a metre of water forded adds to a link's cost in the cost graph
    /// (a lowland cell with no road is 1, a made road 0.45). Widths are the
    /// textbook's, `roads::WIDTH_PER_ROOT_FLOW` times the root of the flow.
    pub ford: f64,
    /// What a ford adds to a link's cost whatever its width: the banks to go
    /// down and up, the wet and the wait.
    pub ford_base: f64,
    /// The widest water that is forded, metres. A wider river is crossed by
    /// a ferry.
    pub ford_width: f64,
    /// What crossing by a ferry adds to a link's cost.
    pub ferry: f64,
    /// Stone a metre of water for a bridge (the prototype's `roads.bridge_stone`).
    pub bridge_stone: f64,
    /// Wages paid on every shilling of stone: bridge building is mostly labour.
    pub wages: f64,
    /// Months a bridge takes if the market fills every order.
    pub months: f64,
    /// A parish may lay out this share of its landlords' cash in a year on its
    /// bridges (a quarter of it of its households' where the landlords are in debt).
    pub rate: f64,
    /// What a bridge must save a year in fords' and ferries' charges, on last
    /// year's traffic, over what it costs.
    pub hurdle: f64,
}

impl Default for CrossingParams {
    /// Off: what the port runs with.
    fn default() -> CrossingParams {
        CrossingParams { enabled: false, ford: 0.03, ford_base: 0.0, ford_width: 25.0, ferry: 1.5, bridge_stone: 8.0, wages: 2.0, months: 12.0, rate: 0.02, hurdle: 0.05 }
    }
}

impl CrossingParams {
    /// The game's crossings.
    pub fn game() -> CrossingParams {
        CrossingParams { enabled: true, ..CrossingParams::default() }
    }
}

/// One place where a link's straight line crosses a river's line.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Crossing {
    /// Where, in cells.
    pub x: f32,
    pub y: f32,
    /// Which river (its place in `Landform::rivers`) and which of its lengths
    /// (from point `seg` to the next).
    pub river: u32,
    pub seg: u32,
    /// How far along the link it lies, 0 at `a` and 1 at `b`.
    pub t: f32,
    /// The water's width there as the land keeps it (and the viewer draws it),
    /// metres, and the textbook's width from its flow, which costs are reckoned in.
    pub drawn: f32,
    pub width: f32,
}

/// A link between two neighbouring land cells whose way crosses water.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Link {
    /// The two cells, `a` the lower.
    pub a: u32,
    pub b: u32,
    /// Its crossings: `Table::crossings[first..first + n]`, in order along it.
    pub first: u32,
    pub n: u32,
    /// Metres of water crossed in all (textbook widths).
    pub span: f32,
    /// A river on it is too wide to ford, and it is crossed by a ferry.
    pub ferry: bool,
    /// What crossing it costs in the cost graph while it has no bridge.
    pub charge: f32,
    /// How much further the way goes, in cells, going round the meanders its
    /// straight line cuts; it costs that much more to pass, bridge or none.
    pub detour: f32,
}

/// The crossings of an island: static once made.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    /// Where each cell's roads meet, in cells.
    pub middles: Vec<(f32, f32)>,
    pub links: Vec<Link>,
    pub crossings: Vec<Crossing>,
    /// `at[c * 8 + k]`: the link from cell c to its neighbour in direction
    /// `NEIGH8[k]`, or `u32::MAX` where that way crosses no water.
    pub at: Vec<u32>,
}

/// A bridge, as it was built.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Built {
    pub link: u32,
    pub year: i64,
    /// The parish (town) that paid, and whether out of its landlords' cash
    /// (else its households'); none for one inherited in 1750.
    pub town: usize,
    pub lords: bool,
    /// What it cost, money, and the metres of water it spans.
    pub cost: f64,
    pub span: f64,
    /// It stood before the first recorded year.
    pub inherited: bool,
}

/// Everything the crossings keep. Empty while the rule is off or on the old island.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CrossingState {
    pub table: Table,
    /// By link: the year its bridge was built, or `NOT_BUILT`.
    pub built: Vec<i32>,
    /// By link: the weight carted over it last year, both ways.
    pub traffic: Vec<f64>,
    /// Bridge works under way: (town, its links).
    pub works: Vec<(usize, Vec<u32>)>,
    /// Every bridge in the order it was built.
    pub bridges: Vec<Built>,
    /// The first recorded year has begun (the roads of 1750 are inherited).
    pub begun: bool,
}

/// Whether the rule acts in this run: it is on and the island has fine rivers.
pub fn active(st: &State, P: &Params) -> bool {
    P.crossings.enabled && st.world.landform.is_some()
}

// ------------------------------------------------------------------ the fine land

/// What the meeting places and the crossings read of the land: the core's
/// `Landform`, or the viewer's copy of it, which must give the same numbers.
pub trait Fine {
    /// The map's size in cells.
    fn cells(&self) -> (usize, usize);
    /// Nodes to a cell's side, and the fine grid's size in nodes.
    fn grid(&self) -> (usize, usize, usize);
    /// Per node: metres above the sea.
    fn height(&self) -> &[f32];
    /// Per node: 0, or the lake it lies under plus 1, as a byte.
    fn lake(&self, n: usize) -> u8;
    /// Per node: `landwater::WET_*`.
    fn wet(&self, n: usize) -> u8;
    /// The level of lake `id` (its number plus 1), metres.
    fn lake_level(&self, id: u8) -> f32;
    /// The river lines: how many, how many points each, each point in cells,
    /// and the width (metres), level (metres) and flow at each.
    fn rivers(&self) -> usize;
    fn points(&self, r: usize) -> usize;
    fn point(&self, r: usize, i: usize) -> (f32, f32);
    fn width(&self, r: usize) -> &[f32];
    fn level(&self, r: usize) -> &[f32];
    fn flow(&self, r: usize) -> &[f32];
}

/// The core's land, with the map's size beside it.
pub struct OnLand<'a> {
    pub lf: &'a Landform,
    pub w: usize,
    pub h: usize,
}

impl Fine for OnLand<'_> {
    fn cells(&self) -> (usize, usize) {
        (self.w, self.h)
    }
    fn grid(&self) -> (usize, usize, usize) {
        (self.lf.fine, self.lf.fw, self.lf.fh)
    }
    fn height(&self) -> &[f32] {
        &self.lf.height
    }
    fn lake(&self, n: usize) -> u8 {
        // (a byte, as the record keeps it and the viewer reads it)
        self.lf.lake_id[n].min(255) as u8
    }
    fn wet(&self, n: usize) -> u8 {
        self.lf.wet[n]
    }
    fn lake_level(&self, id: u8) -> f32 {
        self.lf.lakes.get(id as usize - 1).map_or(0.0, |l| l.level)
    }
    fn rivers(&self) -> usize {
        self.lf.rivers.len()
    }
    fn points(&self, r: usize) -> usize {
        self.lf.rivers[r].x.len()
    }
    fn point(&self, r: usize, i: usize) -> (f32, f32) {
        (self.lf.rivers[r].x[i], self.lf.rivers[r].y[i])
    }
    fn width(&self, r: usize) -> &[f32] {
        &self.lf.rivers[r].width
    }
    fn level(&self, r: usize) -> &[f32] {
        &self.lf.rivers[r].level
    }
    fn flow(&self, r: usize) -> &[f32] {
        &self.lf.rivers[r].flow
    }
}

/// A number from 0 to 1 for a pair of integers and a stream (the viewer's and
/// the generators' `hash2`, the same to the bit).
pub fn hash2(x: i32, y: i32, s: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263) ^ (s as u32).wrapping_mul(1_442_695_041);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    h as f32 / 4_294_967_295.0
}

/// Catmull-Rom weights for the four nodes about a spot `t` of the way from the second to the third.
fn weights(t: f32) -> [f32; 4] {
    let (t2, t3) = (t * t, t * t * t);
    [-0.5 * t3 + t2 - 0.5 * t, 1.5 * t3 - 2.5 * t2 + 1.0, -1.5 * t3 + 2.0 * t2 + 0.5 * t, 0.5 * t3 - 0.5 * t2]
}

/// Metres above the sea at a spot in cells: a smooth surface through the
/// nodes, never lower than the lowest of the nodes the spot lies between, and
/// under the sea where all of those are (the viewer's `NewLand::metres_at`).
pub fn metres_at<L: Fine + ?Sized>(l: &L, x: f32, y: f32) -> f32 {
    let (fine, fw, fh) = l.grid();
    let height = l.height();
    let (fx, fy) = (x * fine as f32 - 0.5, y * fine as f32 - 0.5);
    let (i, j) = (fx.floor(), fy.floor());
    let (tx, ty) = (fx - i, fy - j);
    let (wx, wy) = (weights(tx), weights(ty));
    let (i, j) = (i as isize, j as isize);
    let node = |a: isize, b: isize| height[(j + b).clamp(0, fh as isize - 1) as usize * fw + (i + a).clamp(0, fw as isize - 1) as usize];
    let mut sum = 0.0;
    for (b, wyb) in wy.iter().enumerate() {
        for (a, wxa) in wx.iter().enumerate() {
            sum += wxa * wyb * node(a as isize - 1, b as isize - 1);
        }
    }
    let (ax, by) = (if tx > 1e-4 { 1 } else { 0 }, if ty > 1e-4 { 1 } else { 0 });
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for b in 0..=by {
        for a in 0..=ax {
            let h = node(a, b);
            lo = lo.min(h);
            hi = hi.max(h);
        }
    }
    let sum = sum.max(lo);
    if hi <= 0.0 { sum.min(hi) } else { sum }
}

/// The node a spot in cells lies in.
pub fn node_at<L: Fine + ?Sized>(l: &L, x: f32, y: f32) -> usize {
    let (fine, fw, fh) = l.grid();
    let i = ((x * fine as f32).max(0.0) as usize).min(fw - 1);
    let j = ((y * fine as f32).max(0.0) as usize).min(fh - 1);
    j * fw + i
}

/// The level of a lake that has a node beside node `n` (on any of its eight
/// sides), in metres: the highest, if there are two.
pub fn lake_beside<L: Fine + ?Sized>(l: &L, n: usize) -> Option<f32> {
    let (_, fw, fh) = l.grid();
    let (i, j) = ((n % fw) as isize, (n / fw) as isize);
    let mut level: Option<f32> = None;
    for dj in -1..=1 {
        for di in -1..=1 {
            let (mi, mj) = (i + di, j + dj);
            if (di, dj) == (0, 0) || mi < 0 || mj < 0 || mi as usize >= fw || mj as usize >= fh {
                continue;
            }
            let id = l.lake(mj as usize * fw + mi as usize);
            if id > 0 {
                level = Some(level.map_or(l.lake_level(id), |v| v.max(l.lake_level(id))));
            }
        }
    }
    level
}

// ------------------------------------------------------------------ where a cell's roads meet
//
// The viewer's newland/middles.rs, moved here so that the simulation and the
// picture agree on where every way runs. Its words, briefly: a land cell's
// meeting place is its middle with a little jitter, moved where that is bad
// onto the nearest dry and level spot of its own cell: off tidewater, lakes
// and marsh, back from a river, and off the steepest ground where the cell has
// gentler.

/// Spots tried along each side of a cell.
const TRY: usize = 13;
/// How far from the cell's edge the spots tried keep, in cells.
const INSET: f32 = 0.14;
/// How high a place must stand to be built on, metres.
const DRY_M: f32 = 1.8;
/// The height the tide's beach reaches, metres.
const BEACH_M: f32 = 0.4;
/// How far round a place the ground must be dry and firm, in cells.
const ROUND: f32 = 0.07;
/// How far back from a river's water a place keeps, in cells (some 160 m).
const BACK: f32 = 0.08;
/// A slope that is level enough for anybody (1 in 20).
const LEVEL: f32 = 0.05;
/// Cells of the grid the rivers are marked on, to a map cell's side.
const MARK: usize = 16;
/// How far from a river's line its bed can lower the ground, in cells.
const BED: f32 = 0.27;
/// `landwater::WET_MARSH`.
const WET_MARSH: u8 = 2;

/// The old island's meeting place for cell `c`: its middle with a little jitter.
pub fn jittered(c: usize, w: usize) -> (f32, f32) {
    let (x, y) = ((c % w) as f32, (c / w) as f32);
    (x + 0.5 + 0.36 * (hash2(c as i32, 11, 1) - 0.5), y + 0.5 + 0.36 * (hash2(c as i32, 13, 2) - 0.5))
}

/// What is wrong with a spot as a meeting place, the worst first, and how steep it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Footing {
    /// the spot is under tidewater or a lake
    pub water: bool,
    /// the spot is dry and stands under `DRY_M`, where nothing is built
    pub low: bool,
    /// water close round the spot (within `ROUND`)
    pub shore: bool,
    /// marsh at the spot or close round it, or ground too low to build on close round it
    pub marsh: bool,
    /// a river's water within `BACK`
    pub river: bool,
    /// the ground's slope there, as the core's heights have it
    pub slope: f32,
}

impl Footing {
    /// How bad, in kind: 0 is dry, firm ground back from the river.
    pub fn tier(&self) -> u8 {
        if self.water {
            5
        } else if self.low {
            4
        } else if self.shore {
            3
        } else if self.marsh {
            2
        } else if self.river {
            1
        } else {
            0
        }
    }

    /// The same in a word or two, for a report.
    pub fn words(&self) -> &'static str {
        ["dry ground", "a river's bank", "marsh", "the water's edge", "ground too low to build on", "water"][self.tier() as usize]
    }
}

/// The spots near a river's water, on a grid of `MARK` to a cell, and the
/// bank each cell's meeting place keeps to.
pub struct Marks {
    w: usize,
    h: usize,
    /// within `BACK` of the water
    near: Vec<bool>,
    /// within `BED` of a reach whose water stands under `DRY_M`
    low: Vec<bool>,
    /// By map cell: the biggest river through it and the bank of it the
    /// cell's roads meet on (`Bank`), or none.
    bank: Vec<Option<Bank>>,
}

/// Which bank of a river a cell's meeting place keeps to. A cell that a
/// river runs through has its roads meet on the side where most of the
/// cell lies; where the river runs through its middle, on the side the cell
/// before it down the same river took. So the meeting places of a valley's
/// cells keep to one bank down a reach, and a road from one to the next
/// does not cross the river and back again for want of a place to meet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bank {
    /// the river, and the points of it that lie in the cell (and one either side)
    pub river: u32,
    pub i0: u32,
    pub i1: u32,
    /// the left bank, looking downstream
    pub left: bool,
}

/// How much of a cell must lie on one side of its river for that side to be
/// its bank whatever the cell up the river took.
const BANK_SHARE: f32 = 0.6;

/// Whether a spot lies on the left of a river (looking downstream), by the
/// nearest of the river's lengths `i0..i1`.
fn left_of<L: Fine + ?Sized>(l: &L, river: usize, i0: usize, i1: usize, x: f32, y: f32) -> bool {
    let (mut best, mut left) = (f32::MAX, true);
    for i in i0..i1.min(l.points(river) - 1) {
        let (p, q) = (l.point(river, i), l.point(river, i + 1));
        let (ex, ey) = (q.0 - p.0, q.1 - p.1);
        let len2 = (ex * ex + ey * ey).max(1e-12);
        let u = (((x - p.0) * ex + (y - p.1) * ey) / len2).clamp(0.0, 1.0);
        let (nx, ny) = (p.0 + ex * u - x, p.1 + ey * u - y);
        let d = nx * nx + ny * ny;
        if d < best {
            best = d;
            // (y grows down the map: a spot on the left of the way the river runs has a negative cross)
            left = ex * (y - p.1) - ey * (x - p.0) < 0.0;
        }
    }
    left
}

/// The bank each cell's meeting place keeps to (`Bank`).
fn find_banks<L: Fine + ?Sized>(l: &L) -> Vec<Option<Bank>> {
    let (cw, ch) = l.cells();
    let mut bank: Vec<Option<Bank>> = vec![None; cw * ch];
    let mut biggest = vec![0.0f32; cw * ch];
    let cell_of = |p: (f32, f32)| (p.1.floor().clamp(0.0, ch as f32 - 1.0) as usize) * cw + p.0.floor().clamp(0.0, cw as f32 - 1.0) as usize;
    for r in 0..l.rivers() {
        let Some(from) = drawn_from(l, r) else { continue };
        let (n, flow, level) = (l.points(r), l.flow(r), l.level(r));
        let mut was: Option<bool> = None;
        let mut i = from;
        while i < n {
            let c = cell_of(l.point(r, i));
            let mut j = i + 1;
            while j < n && cell_of(l.point(r, j)) == c {
                j += 1;
            }
            if level[i] >= 0.0 {
                let (i0, i1) = (i.saturating_sub(1).max(from), j.min(n - 1));
                // the share of the cell on the left, over a grid of spots
                let (cx, cy) = ((c % cw) as f32, (c / cw) as f32);
                let mut on_left = 0;
                for b in 0..9 {
                    for a in 0..9 {
                        on_left += left_of(l, r, i0, i1, cx + (a as f32 + 0.5) / 9.0, cy + (b as f32 + 0.5) / 9.0) as usize;
                    }
                }
                let share = on_left as f32 / 81.0;
                let left = if share >= BANK_SHARE {
                    true
                } else if share <= 1.0 - BANK_SHARE {
                    false
                } else {
                    was.unwrap_or(share >= 0.5)
                };
                was = Some(left);
                let q = flow[i..j].iter().copied().fold(0.0, f32::max);
                if q > biggest[c] {
                    biggest[c] = q;
                    bank[c] = Some(Bank { river: r as u32, i0: i0 as u32, i1: i1 as u32, left });
                }
            }
            i = j;
        }
    }
    bank
}

impl Marks {
    /// Mark every spot near the water of a river the viewer draws, and,
    /// with `banks`, the bank each cell's meeting place keeps to (without,
    /// the meeting places are those the viewer drew before the crossings).
    pub fn of<L: Fine + ?Sized>(l: &L, banks: bool) -> Marks {
        let (cw, ch) = l.cells();
        let (w, h) = (cw * MARK, ch * MARK);
        let (mut near, mut low) = (vec![false; w * h], vec![false; w * h]);
        let m = MARK as f32;
        let mark = |marks: &mut [bool], p: (f32, f32), reach: f32| {
            let (x0, x1) = (((p.0 - reach) * m).floor().max(0.0) as usize, (((p.0 + reach) * m).floor().max(0.0) as usize).min(w - 1));
            let (y0, y1) = (((p.1 - reach) * m).floor().max(0.0) as usize, (((p.1 + reach) * m).floor().max(0.0) as usize).min(h - 1));
            for y in y0..=y1 {
                for x in x0..=x1 {
                    // the middle of the grid's cell, within reach and half the cell's own size
                    let q = ((x as f32 + 0.5) / m, (y as f32 + 0.5) / m);
                    let (dx, dy) = (q.0 - p.0, q.1 - p.1);
                    if (dx * dx + dy * dy).sqrt() <= reach + 0.5 / m {
                        marks[y * w + x] = true;
                    }
                }
            }
        };
        for r in 0..l.rivers() {
            let (width, level, flow) = (l.width(r), l.level(r), l.flow(r));
            for i in 0..l.points(r) {
                if flow[i] < MIN_FLOW {
                    continue;
                }
                let p = l.point(r, i);
                let half = 0.5 * width[i] / CELL_M;
                mark(&mut near, p, half + BACK);
                if level[i] < DRY_M {
                    mark(&mut low, p, half + BED);
                }
            }
        }
        Marks { w, h, near, low, bank: if banks { find_banks(l) } else { Vec::new() } }
    }

    /// The bank cell `c`'s meeting place keeps to, if a river runs through it.
    pub fn bank(&self, c: usize) -> Option<Bank> {
        self.bank.get(c).copied().flatten()
    }

    fn at(&self, x: f32, y: f32) -> usize {
        ((y * MARK as f32).max(0.0) as usize).min(self.h - 1) * self.w + ((x * MARK as f32).max(0.0) as usize).min(self.w - 1)
    }
}

/// What a spot is like to stand a market place on.
pub fn footing<L: Fine + ?Sized>(l: &L, marks: &Marks, x: f32, y: f32) -> Footing {
    // the spot and eight spots round it
    let mut round = vec![(x, y)];
    for k in 0..8 {
        let a = k as f32 * std::f32::consts::FRAC_PI_4;
        round.push((x + ROUND * a.cos(), y + ROUND * a.sin()));
    }
    let (mut water, mut low, mut shore, mut marsh) = (false, false, false, false);
    for (i, &(px, py)) in round.iter().enumerate() {
        let n = node_at(l, px, py);
        let m = metres_at(l, px, py);
        // under the tide (or on the beach it leaves), in a lake, or under the water of a lake beside it
        let wet = l.lake(n) > 0 || m < BEACH_M || lake_beside(l, n).is_some_and(|level| m < level + 0.5);
        // too low to build on: by its own height, or brought down by the bed of a river near the tide
        let sunk = m < DRY_M || marks.low[marks.at(px, py)];
        if i == 0 {
            (water, low) = (wet, !wet && sunk);
        } else {
            shore |= wet;
            marsh |= sunk;
        }
        marsh |= l.wet(n) == WET_MARSH;
    }
    let e = 0.06;
    let fall = (metres_at(l, x + e, y) - metres_at(l, x - e, y)).hypot(metres_at(l, x, y + e) - metres_at(l, x, y - e)) / (2.0 * e * CELL_M);
    Footing { water, low, shore, marsh, river: marks.near[marks.at(x, y)], slope: fall }
}

/// The meeting place of every cell: for a cell of water (`terrain` 0) the old
/// island's, and for a land cell the nearest spot to the old island's that is
/// as good as the cell has.
pub fn meeting_places<L: Fine + ?Sized>(l: &L, terrain: &[u8], banks: bool) -> Vec<(f32, f32)> {
    let marks = Marks::of(l, banks);
    let (w, h) = l.cells();
    (0..w * h).map(|c| meeting_place(l, &marks, terrain, c)).collect()
}

/// The meeting place of cell `c` (`meeting_places`), given the river marks.
pub fn meeting_place<L: Fine + ?Sized>(l: &L, marks: &Marks, terrain: &[u8], c: usize) -> (f32, f32) {
    let (w, _) = l.cells();
    let first = jittered(c, w);
    if terrain.get(c).is_none_or(|&t| t == 0) {
        return first;
    }
    // which bank of its river the cell's roads meet on, if a river runs through it
    let bank = marks.bank(c);
    let on_bank = |p: (f32, f32)| bank.is_none_or(|b| left_of(l, b.river as usize, b.i0 as usize, b.i1 as usize, p.0, p.1) == b.left);
    // most of the lowland: where the old island would put it is dry, firm and level, and it stays
    let there = footing(l, marks, first.0, first.1);
    if there.tier() == 0 && there.slope <= LEVEL && on_bank(first) {
        return first;
    }
    // else that spot and a grid of spots over the cell
    let (cx, cy) = ((c % w) as f32, (c / w) as f32);
    let mut spots = vec![first];
    for j in 0..TRY {
        for i in 0..TRY {
            let at = |k: usize| INSET + (1.0 - 2.0 * INSET) * k as f32 / (TRY - 1) as f32;
            spots.push((cx + at(i), cy + at(j)));
        }
    }
    let found: Vec<Footing> = spots.iter().map(|&(x, y)| footing(l, marks, x, y)).collect();
    // the spots on the cell's bank, where it has one and its ground there is fit to
    // build on: as good in kind as the cell's best, or dry ground by the river where the
    // best is dry ground back from it
    let best_all = found.iter().map(|f| f.tier()).min().unwrap_or(0);
    let banked: Vec<bool> = spots.iter().map(|&p| on_bank(p)).collect();
    let best_bank = found.iter().zip(&banked).filter(|(_, b)| **b).map(|(f, _)| f.tier()).min();
    let keep_bank = best_bank.is_some_and(|t| t <= best_all.max(1));
    let may = |i: usize| !keep_bank || banked[i];
    // the best kind of ground the cell has, and among the spots of that kind the gentlest
    let best = found.iter().enumerate().filter(|&(i, _)| may(i)).map(|(_, f)| f.tier()).min().unwrap_or(0);
    let gentlest = found.iter().enumerate().filter(|&(i, f)| may(i) && f.tier() == best).map(|(_, f)| f.slope).fold(f32::MAX, f32::min);
    // level enough: level, or not much steeper than the gentlest the cell has
    let level = LEVEL.max(1.5 * gentlest + 0.01);
    let d2 = |p: (f32, f32)| (p.0 - first.0).powi(2) + (p.1 - first.1).powi(2);
    let mut chosen = 0;
    let mut least = f32::MAX;
    for (i, f) in found.iter().enumerate() {
        if may(i) && f.tier() == best && f.slope <= level && d2(spots[i]) < least {
            (chosen, least) = (i, d2(spots[i]));
        }
    }
    spots[chosen]
}

// ------------------------------------------------------------------ the table

/// The four ways from a cell to a neighbour that each link is found from once
/// (as the viewer's `network::road_segments` joins road cells), as (dx, dy).
pub const FORWARD: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (-1, 1)];

/// The direction `k` of `NEIGH8` from a cell to a neighbour (dx, dy).
pub fn dir(dx: isize, dy: isize) -> usize {
    NEIGH8.iter().position(|&(ny, nx)| (ny, nx) == (dy, dx)).expect("a neighbour")
}

/// One river's lengths that are drawn: from where it first carries
/// `MIN_FLOW`, and only where its water stands at or above the sea.
fn drawn_from<L: Fine + ?Sized>(l: &L, r: usize) -> Option<usize> {
    let from = l.flow(r).iter().position(|&q| q >= MIN_FLOW)?;
    if l.points(r) - from < 2 {
        return None;
    }
    Some(from)
}

/// Where the straight line from `a` to `b` crosses the drawn rivers: for
/// each crossing, how far along (0 to 1), the river, the length of it, and
/// how far along that length. The arithmetic is the viewer's
/// `rivers::crossings`, in the same order, so the two find the same points.
/// `index` holds each drawn length of river by the cells its box touches.
fn cross_line<L: Fine + ?Sized>(l: &L, index: &[Vec<(u32, u32)>], a: (f32, f32), b: (f32, f32)) -> Vec<(f32, u32, u32, f32)> {
    let (w, h) = l.cells();
    let (lo, hi) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));
    let (x0, y0) = (lo.0.floor().max(0.0) as usize, lo.1.floor().max(0.0) as usize);
    let (x1, y1) = ((hi.0.floor().max(0.0) as usize).min(w - 1), (hi.1.floor().max(0.0) as usize).min(h - 1));
    let d = (b.0 - a.0, b.1 - a.1);
    let mut seen: Vec<(u32, u32)> = Vec::new();
    let mut out = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            for &(r, i) in &index[y * w + x] {
                if seen.contains(&(r, i)) {
                    continue;
                }
                seen.push((r, i));
                let (p, q) = (l.point(r as usize, i as usize), l.point(r as usize, i as usize + 1));
                let e = (q.0 - p.0, q.1 - p.1);
                let den = d.0 * e.1 - d.1 * e.0;
                if den.abs() < 1e-9 {
                    continue;
                }
                let wv = (p.0 - a.0, p.1 - a.1);
                let t = (wv.0 * e.1 - wv.1 * e.0) / den;
                let u = (wv.0 * d.1 - wv.1 * d.0) / den;
                if (0.0..=1.0).contains(&t) && (0.0..1.0).contains(&u) {
                    out.push((t, r, i, u));
                }
            }
        }
    }
    out.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)).then(x.2.cmp(&y.2)));
    out
}

/// Each drawn length of river (river, point it starts at) by every cell its
/// box touches.
fn river_index<L: Fine + ?Sized>(l: &L) -> Vec<Vec<(u32, u32)>> {
    let (w, h) = l.cells();
    let mut index: Vec<Vec<(u32, u32)>> = vec![Vec::new(); w * h];
    for r in 0..l.rivers() {
        let Some(from) = drawn_from(l, r) else { continue };
        let level = l.level(r);
        // (the levels never rise downstream; the least so far is held as the viewer holds it)
        let mut lv = level[from];
        for i in from..l.points(r) - 1 {
            lv = lv.min(level[i]);
            if lv < 0.0 {
                continue; // out in the tide
            }
            let (p, q) = (l.point(r, i), l.point(r, i + 1));
            let (x0, x1) = (p.0.min(q.0).floor().max(0.0) as usize, (p.0.max(q.0).floor().max(0.0) as usize).min(w - 1));
            let (y0, y1) = (p.1.min(q.1).floor().max(0.0) as usize, (p.1.max(q.1).floor().max(0.0) as usize).min(h - 1));
            for y in y0..=y1 {
                for x in x0..=x1 {
                    index[y * w + x].push((r as u32, i as u32));
                }
            }
        }
    }
    index
}

/// The table of crossings for a land: every link between two neighbouring
/// land cells whose way, drawn straight between their meeting places, crosses
/// a drawn river, with each crossing and the river's width there.
pub fn table<L: Fine + ?Sized>(l: &L, land: &[bool], terrain: &[u8], C: &CrossingParams) -> Table {
    // (the meeting places keep to their banks while the rule is on; the probe
    // measures the rule off on the meeting places as the viewer drew them before)
    let middles = meeting_places(l, terrain, C.enabled);
    links_for(l, land, middles, C, C.enabled)
}

/// How far it is along river `r` from `s0` to `s1`, each a point of it and
/// how far on to the next (`i + u`), in cells.
fn arc<L: Fine + ?Sized>(l: &L, r: usize, s0: f32, s1: f32) -> f32 {
    let (lo, hi) = (s0.min(s1), s0.max(s1));
    let at = |s: f32| {
        let i = (s.floor() as usize).min(l.points(r) - 2);
        let u = s - i as f32;
        let (p, q) = (l.point(r, i), l.point(r, i + 1));
        (p.0 + (q.0 - p.0) * u, p.1 + (q.1 - p.1) * u)
    };
    let mut pts = vec![at(lo)];
    for i in (lo.floor() as usize + 1)..=(hi.floor() as usize) {
        if (i as f32) < hi {
            pts.push(l.point(r, i));
        }
    }
    pts.push(at(hi));
    pts.windows(2).map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt()).sum()
}

/// `table`, with the meeting places given. With `round`, a way whose straight
/// line crosses the same river an even number of times keeps to its bank and
/// goes round the meander instead, and one that crosses it an odd number of
/// times crosses it once, at the middle of those points, going round the rest
/// (the picture draws it so: the viewer's `network::route`); without, every
/// point where the straight line meets a river is a crossing, which is what
/// the picture drew before the rule.
pub fn links_for<L: Fine + ?Sized>(l: &L, land: &[bool], middles: Vec<(f32, f32)>, C: &CrossingParams, round: bool) -> Table {
    let (w, h) = l.cells();
    let index = river_index(l);
    let mut links: Vec<Link> = Vec::new();
    let mut crossings: Vec<Crossing> = Vec::new();
    let mut at = vec![u32::MAX; w * h * 8];
    for a in 0..w * h {
        if !land[a] {
            continue;
        }
        let (x, y) = ((a % w) as isize, (a / w) as isize);
        for &(dx, dy) in &FORWARD {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || nx >= w as isize || ny >= h as isize {
                continue;
            }
            let b = ny as usize * w + nx as usize;
            if !land[b] {
                continue;
            }
            let mut found = cross_line(l, &index, middles[a], middles[b]);
            if found.is_empty() {
                continue;
            }
            // going round: by river, the points in order along the way; an even number
            // is a meander skirted, an odd number one crossing at the middle point
            let mut detour = 0.0f32;
            let mut rivers: Vec<u32> = found.iter().map(|f| f.1).collect();
            rivers.sort_unstable();
            rivers.dedup();
            // (where one river's points along the way run among another's, the way is drawn
            // straight and crosses at every one, as `network::route` draws it)
            let span = |r: u32| {
                let ts: Vec<f32> = found.iter().filter(|f| f.1 == r).map(|f| f.0).collect();
                (ts[0], ts[ts.len() - 1])
            };
            let tangled = rivers.iter().enumerate().any(|(x, &rx)| rivers[x + 1..].iter().any(|&ry| span(rx).0 <= span(ry).1 && span(ry).0 <= span(rx).1));
            let some_twice = rivers.len() < found.len();
            if round && some_twice && !tangled {
                let mut kept = Vec::new();
                for r in rivers {
                    let of: Vec<(f32, u32, u32, f32)> = found.iter().copied().filter(|f| f.1 == r).collect();
                    if of.len() >= 2 {
                        let (s0, s1) = (of[0].2 as f32 + of[0].3, of[of.len() - 1].2 as f32 + of[of.len() - 1].3);
                        let chord = (of[of.len() - 1].0 - of[0].0) * ((middles[b].0 - middles[a].0).powi(2) + (middles[b].1 - middles[a].1).powi(2)).sqrt();
                        detour += (arc(l, r as usize, s0, s1) - chord).max(0.0);
                    }
                    if of.len() % 2 == 1 {
                        kept.push(of[of.len() / 2]);
                    }
                }
                kept.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
                found = kept;
            }
            let first = crossings.len() as u32;
            let (mut span, mut ferry, mut charge) = (0.0f64, false, 0.0f64);
            for &(t, r, i, u) in &found {
                let (r_, i_) = (r as usize, i as usize);
                let (width, flow) = (l.width(r_), l.flow(r_));
                let q = (flow[i_] + (flow[i_ + 1] - flow[i_]) * u).max(0.0) as f64;
                let true_w = crate::roads::WIDTH_PER_ROOT_FLOW * q.sqrt();
                span += true_w;
                if true_w > C.ford_width {
                    ferry = true;
                    charge += C.ferry;
                } else {
                    charge += C.ford_base + C.ford * true_w;
                }
                crossings.push(Crossing {
                    x: middles[a].0 + (middles[b].0 - middles[a].0) * t,
                    y: middles[a].1 + (middles[b].1 - middles[a].1) * t,
                    river: r,
                    seg: i,
                    t,
                    drawn: width[i_] + (width[i_ + 1] - width[i_]) * u,
                    width: true_w as f32,
                });
            }
            let li = links.len() as u32;
            links.push(Link { a: a as u32, b: b as u32, first, n: found.len() as u32, span: span as f32, ferry, charge: charge as f32, detour });
            at[a * 8 + dir(dx, dy)] = li;
            at[b * 8 + dir(-dx, -dy)] = li;
        }
    }
    Table { middles, links, crossings, at }
}

impl Table {
    /// The link between two neighbouring cells, if its way crosses water.
    pub fn between(&self, w: usize, p: usize, n: usize) -> Option<usize> {
        let (dx, dy) = ((n % w) as isize - (p % w) as isize, (n / w) as isize - (p / w) as isize);
        if dx.abs() > 1 || dy.abs() > 1 || (dx, dy) == (0, 0) {
            return None;
        }
        let li = self.at[p * 8 + dir(dx, dy)];
        (li != u32::MAX).then_some(li as usize)
    }

    /// A link's crossings.
    pub fn of(&self, li: usize) -> &[Crossing] {
        let l = &self.links[li];
        &self.crossings[l.first as usize..(l.first + l.n) as usize]
    }
}

/// The table for the state's island, made once.
fn ensure(st: &mut State, P: &Params) {
    if !st.xs.table.links.is_empty() || !st.xs.table.middles.is_empty() {
        return;
    }
    let Some(lf) = st.world.landform.as_deref() else { return };
    let on = OnLand { lf, w: st.world.W, h: st.world.H };
    let table = table(&on, &st.world.land, &st.world.terrain, &P.crossings);
    st.xs.built = vec![NOT_BUILT; table.links.len()];
    st.xs.traffic = vec![0.0; table.links.len()];
    st.xs.table = table;
}

// ------------------------------------------------------------------ the rule

/// The fords and ferries in the cost graph: a link over water with no bridge
/// costs its charge more to pass, either way, except by rail (a railway
/// bridges what it crosses as part of its own cost) and by boat along a
/// navigable river.
pub fn charges(st: &mut State, P: &Params, g: &mut tr::Graph) {
    if !active(st, P) {
        return;
    }
    ensure(st, P);
    charge_graph(&st.xs.table, &st.xs.built, &st.world.navigable, st.world.W, g);
}

/// `charges` on a graph, given the table, the bridges built (`NOT_BUILT` for
/// none) and which cells are navigable.
pub fn charge_graph(table: &Table, built: &[i32], navigable: &[bool], W: usize, g: &mut tr::Graph) {
    let mut any = false;
    for (li, link) in table.links.iter().enumerate() {
        // (going round a meander costs the way's length at a cell of open lowland, 1)
        let add = if built[li] != NOT_BUILT { 0.0 } else { link.charge as f64 } + link.detour as f64;
        if add <= 0.0 {
            continue;
        }
        let (a, b) = (link.a as usize, link.b as usize);
        if navigable[a] && navigable[b] {
            continue;
        }
        let (dx, dy) = ((b % W) as isize - (a % W) as isize, (b / W) as isize - (a / W) as isize);
        for e in [a * 8 + dir(dx, dy), b * 8 + dir(-dx, -dy)] {
            if !g.railed[e] && g.w[e].is_finite() {
                g.w[e] += add;
                any = true;
            }
        }
    }
    if any {
        let finite = g.w.iter().copied().filter(|x| x.is_finite());
        g.min_w = finite.clone().fold(f64::INFINITY, f64::min);
        g.max_w = finite.fold(0.0, f64::max);
    }
}

/// The weight carted over each link last year, both ways: the same trees and
/// the same flows `sim::update_roads` wore the roads with, added up on the
/// links over water instead of on the cells.
fn link_traffic(st: &State) -> Vec<f64> {
    let (N, W) = (st.N, st.world.W);
    let tab = &st.xs.table;
    let mut out = vec![0.0; tab.links.len()];
    if st.cell_flow_year.len() != N {
        return out;
    }
    let mut acc = vec![0.0; N];
    for i in 0..st.T_max {
        if !st.town_active[i] || st.pred[i].len() != N {
            continue;
        }
        let mut any = false;
        for c in 0..N {
            acc[c] = if st.catch[c] == i as i64 { st.cell_flow_year[c] } else { 0.0 };
            any |= acc[c] > 0.0;
        }
        if !any {
            continue;
        }
        for &n in st.settle[i].iter().rev() {
            let n = n as usize;
            let p = st.pred[i][n];
            if p >= 0 {
                let p = p as usize;
                acc[p] += acc[n];
                if let Some(li) = tab.between(W, p, n) {
                    out[li] += acc[n];
                }
            }
        }
    }
    // the trade between towns along its paths
    let T = st.T_max;
    if st.trade_F_year.rows == T && st.town_cost.rows == T {
        for i in 0..T {
            if !st.town_active[i] || st.pred[i].len() != N {
                continue;
            }
            let root = st.town_node[i] as i64;
            for j in 0..T {
                let f = st.trade_F_year[(i, j)];
                if f <= 0.0 || i == j || !st.town_cost[(i, j)].is_finite() {
                    continue;
                }
                let mut n = st.town_node[j] as i64;
                let mut steps = 0;
                while n != root && n >= 0 && steps < 100_000 {
                    let p = st.pred[i][n as usize];
                    if p >= 0 {
                        if let Some(li) = tab.between(W, p as usize, n as usize) {
                            out[li] += f;
                        }
                    }
                    n = p;
                    steps += 1;
                }
            }
        }
    }
    out
}

/// The year's crossings, after `sim::update_roads` has measured the year's
/// traffic: the traffic over each link over water is counted; in the first
/// recorded year the roads of 1750 are given their bridges; and each parish
/// puts the bridges that would pay for themselves in hand.
pub fn yearly(st: &mut State, C: &Content, P: &Params, year: i64, warm: bool) {
    if !active(st, P) {
        return;
    }
    ensure(st, P);
    st.xs.traffic = link_traffic(st);
    if warm {
        return;
    }
    let X = &P.crossings;
    let gs = C.gi("stone");
    let freight = crate::longrun::freight(st, P);
    if !st.xs.begun {
        st.xs.begun = true;
        // the made roads and turnpikes of 1750 stand with the bridges earlier
        // centuries built on them: those their traffic pays for by the parishes'
        // own test (below), whatever a year's rate would have run to
        for li in 0..st.xs.table.links.len() {
            let l = &st.xs.table.links[li];
            let (a, b) = (l.a as usize, l.b as usize);
            let t = if st.catch[a] >= 0 { st.catch[a] } else { st.catch[b] };
            if t < 0 || l.n == 0 || st.road[a] < 2 || st.road[b] < 2 || st.xs.built[li] != NOT_BUILT {
                continue;
            }
            let p = st.pbar[(t as usize, gs)] * (1.0 + X.wages);
            if freight * st.xs.traffic[li] * l.charge as f64 >= X.hurdle * X.bridge_stone * l.span as f64 * p {
                st.xs.built[li] = year as i32;
                st.xs.bridges.push(Built { link: li as u32, year, town: t as usize, lords: true, cost: 0.0, span: l.span as f64, inherited: true });
            }
        }
    }
    let T = st.T_max;
    let mut busy = vec![false; T];
    let mut working = vec![false; st.xs.table.links.len()];
    for (t, links) in &st.xs.works {
        busy[*t] = true;
        for &li in links {
            working[li as usize] = true;
        }
    }
    // (link, stone, what it saves over what it costs) by parish
    let mut cand: Vec<Vec<(u32, f64, f64)>> = vec![Vec::new(); T];
    for (li, l) in st.xs.table.links.iter().enumerate() {
        if l.n == 0 || st.xs.built[li] != NOT_BUILT || working[li] || st.xs.traffic[li] <= 0.0 {
            continue;
        }
        let (a, b) = (l.a as usize, l.b as usize);
        let t = if st.catch[a] >= 0 { st.catch[a] } else { st.catch[b] };
        if t < 0 || !st.town_active[t as usize] || busy[t as usize] {
            continue;
        }
        let t = t as usize;
        let p = st.pbar[(t, gs)] * (1.0 + X.wages);
        if !(p > 0.0) {
            continue;
        }
        let stone = X.bridge_stone * l.span as f64;
        let saves = freight * st.xs.traffic[li] * l.charge as f64;
        if saves >= X.hurdle * stone * p {
            cand[t].push((li as u32, stone, saves / (stone * p)));
        }
    }
    let G = C.G;
    let mut new_works = Vec::new();
    for t in 0..T {
        if cand[t].is_empty() {
            continue;
        }
        let list = &mut cand[t];
        list.sort_by(|x, y| y.2.total_cmp(&x.2).then(x.0.cmp(&y.0)));
        let p = st.pbar[(t, gs)] * (1.0 + X.wages);
        let lords = st.lord_cash[t] > 0.0;
        let budget = if lords { X.rate * st.lord_cash[t] } else { 0.25 * X.rate * st.hh_cash[t].max(0.0) };
        let (mut stone, mut links) = (0.0, Vec::new());
        for &(li, s, _) in list.iter() {
            if (stone + s) * p <= budget {
                stone += s;
                links.push(li);
            }
        }
        if links.is_empty() {
            continue;
        }
        let cash = stone * p;
        if lords {
            st.lord_cash[t] -= cash;
        } else {
            st.hh_cash[t] -= cash;
        }
        let cells: Vec<usize> = links.iter().map(|&li| st.xs.table.links[li as usize].a as usize).collect();
        let mut need = vec![0.0; G];
        need[gs] = stone;
        new_works.push(Project {
            a: t,
            b: t,
            target: cells[0],
            town: t,
            length_km: 0.0,
            cells: cells.clone(),
            new: Vec::new(),
            order: need.clone(),
            need,
            cash,
            start: year,
            cost: cash,
            way: crate::ways::ROAD,
            months: X.months,
            serves: BRIDGE,
            paid_in: cash,
            subs: vec![(lords, t, cash)],
            wages: X.wages,
            ..Project::default()
        });
        st.xs.works.push((t, links));
    }
    st.projects.extend(new_works);
}

/// The month's end for the bridges (after `econ::step_month`): a works short
/// of cash calls on its parish for more, and a works whose stone is all
/// bought opens its bridges.
pub fn month(st: &mut State, C: &Content, P: &Params, year_float: f64, events: &mut Vec<Event>) {
    if !active(st, P) || st.xs.works.is_empty() {
        return;
    }
    let gs = C.gi("stone");
    let projects = std::mem::take(&mut st.projects);
    let mut keep = Vec::with_capacity(projects.len());
    for mut pr in projects {
        if pr.way != crate::ways::ROAD || pr.serves != BRIDGE {
            keep.push(pr);
            continue;
        }
        let t = pr.town;
        let rq = pr.order[gs].min(pr.need[gs] / pr.months.max(1.0));
        let short = rq * st.price[(t, gs)] * (1.0 + pr.wages) - pr.cash;
        if short > 0.0 {
            let (lords, frac) = if st.lord_cash[t] > 0.0 { (true, 0.1) } else { (false, 0.025) };
            let cash = if lords { &mut st.lord_cash } else { &mut st.hh_cash };
            let call = (cash[t].max(0.0) * frac).min(short);
            if call > 0.0 {
                cash[t] -= call;
                pr.cash += call;
                pr.paid_in += call;
                match pr.subs.iter_mut().find(|s| s.0 == lords && s.1 == t) {
                    Some(s) => s.2 += call,
                    None => pr.subs.push((lords, t, call)),
                }
            }
        }
        if pr.order.iter().sum::<f64>() > 0.01 * pr.need.iter().sum::<f64>() {
            keep.push(pr);
            continue;
        }
        // done: the bridges stand
        let Some(w) = st.xs.works.iter().position(|(town, _)| *town == t) else {
            keep.push(pr);
            continue;
        };
        let (_, links) = st.xs.works.remove(w);
        let spent = pr.paid_in - pr.cash;
        let year = year_float.floor() as i64;
        let span: f64 = links.iter().map(|&li| st.xs.table.links[li as usize].span as f64).sum();
        let lords = pr.subs.iter().map(|s| if s.0 { s.2 } else { 0.0 }).sum::<f64>() >= 0.5 * pr.paid_in;
        let mut ferried = None;
        for &li in &links {
            let l = &st.xs.table.links[li as usize];
            st.xs.built[li as usize] = year as i32;
            let share = if span > 0.0 { spent * l.span as f64 / span } else { 0.0 };
            st.xs.bridges.push(Built { link: li, year, town: t, lords, cost: share, span: l.span as f64, inherited: false });
            if l.ferry && ferried.is_none() {
                ferried = Some((l.a as usize, l.span));
            }
        }
        let subs = std::mem::take(&mut pr.subs);
        crate::ways::repay(st, &subs, pr.cash);
        pr.cash = 0.0;
        // a bridge where there was a ferry is news in its town
        if let Some((cell, span)) = ferried {
            let mut ev = Event::new(
                year_float,
                "bridge",
                0.2,
                format!("{} bridges its river", st.town_name[t]),
                format!(
                    "The parish of {} has built a bridge of stone over {:.0} m of river, where carts waited for the ferry. It was paid for out of the highway rate.",
                    st.town_name[t], span
                ),
            );
            ev.town = Some(t);
            ev.cell = Some(cell);
            events.push(ev);
        }
    }
    st.projects = keep;
}

/// Every link over water and how it is crossed, for the viewer and the record.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct LinkOut {
    pub a: usize,
    pub b: usize,
    /// The year its bridge was built (the first recorded year for one that
    /// stood before it), or none: it is forded, or ferried where `ferry`.
    pub bridged: Option<i64>,
    pub ferry: bool,
}

/// The crossings as they stand (empty while the rule does not act).
pub fn links_out(st: &State) -> Vec<LinkOut> {
    let xs = &st.xs;
    xs.table
        .links
        .iter()
        .enumerate()
        // (a way that only goes round a meander crosses no water)
        .filter(|(_, l)| l.n > 0)
        .map(|(li, l)| LinkOut { a: l.a as usize, b: l.b as usize, bridged: (xs.built[li] != NOT_BUILT).then_some(xs.built[li] as i64), ferry: l.ferry })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A made-up valley for the table alone: a flat land of `w` by `h` cells
    /// with one river line.
    pub struct Valley {
        pub w: usize,
        pub h: usize,
        pub height: Vec<f32>,
        pub line: Vec<(f32, f32)>,
        pub width: Vec<f32>,
        pub level: Vec<f32>,
        pub flow: Vec<f32>,
    }

    impl Fine for Valley {
        fn cells(&self) -> (usize, usize) {
            (self.w, self.h)
        }
        fn grid(&self) -> (usize, usize, usize) {
            (4, 4 * self.w, 4 * self.h)
        }
        fn height(&self) -> &[f32] {
            &self.height
        }
        fn lake(&self, _: usize) -> u8 {
            0
        }
        fn wet(&self, _: usize) -> u8 {
            0
        }
        fn lake_level(&self, _: u8) -> f32 {
            0.0
        }
        fn rivers(&self) -> usize {
            1
        }
        fn points(&self, _: usize) -> usize {
            self.line.len()
        }
        fn point(&self, _: usize, i: usize) -> (f32, f32) {
            self.line[i]
        }
        fn width(&self, _: usize) -> &[f32] {
            &self.width
        }
        fn level(&self, _: usize) -> &[f32] {
            &self.level
        }
        fn flow(&self, _: usize) -> &[f32] {
            &self.flow
        }
    }

    #[test]
    fn a_link_across_a_river_crosses_it_once_and_one_beside_it_not_at_all() {
        // a river down the land at x = 2.2, 9 cubic m a second, on a land of 5 by 5 cells
        let line: Vec<(f32, f32)> = (0..=50).map(|i| (2.2, i as f32 / 10.0)).collect();
        let n = line.len();
        let v = Valley { w: 5, h: 5, height: vec![20.0; 400], line, width: vec![33.0; n], level: vec![10.0; n], flow: vec![9.0; n] };
        let middles: Vec<(f32, f32)> = (0..25).map(|c| ((c % 5) as f32 + 0.5, (c / 5) as f32 + 0.5)).collect();
        let t = links_for(&v, &[true; 25], middles, &CrossingParams::game(), true);
        // across: from column 1 to column 2, straight or aslant, and nothing else
        for l in &t.links {
            let (xa, xb) = (l.a as usize % 5, l.b as usize % 5);
            assert_eq!((xa.min(xb), xa.max(xb)), (1, 2), "{l:?}");
            assert_eq!(l.n, 1);
        }
        // 5 straight links and 8 aslant
        assert_eq!(t.links.len(), 13);
        // 4.4 times the root of 9 is 13.2 m, forded at 0.03 a metre
        let l = &t.links[t.between(5, 2 * 5 + 1, 2 * 5 + 2).unwrap()];
        assert!((l.span - 13.2).abs() < 1e-3 && !l.ferry && (l.charge - 0.396).abs() < 1e-4, "{l:?}");
        assert!((t.of(t.between(5, 11, 12).unwrap())[0].x - 2.2).abs() < 1e-6);
        // a link beside the river, or down the valley, crosses nothing
        assert!(t.between(5, 12, 13).is_none() && t.between(5, 11, 16).is_none() && t.between(5, 12, 17).is_none());
    }

    /// The crossings of the cheapest way from cell `a` to cell `b` over a flat
    /// lowland valley, with the table's fords charged or not.
    fn crossings_on_way(v: &Valley, t: &Table, charged: bool, a: usize, b: usize) -> (u32, f64) {
        let (W, H) = (v.w, v.h);
        let N = W * H;
        let world = crate::world::World {
            W,
            H,
            land: vec![true; N],
            h: vec![0.1; N],
            terrain: vec![crate::world::LOWLAND; N],
            navigable: vec![false; N],
            ..crate::world::World::default()
        };
        let none = vec![0u8; N];
        let mut g = tr::build_graph(&world, &none, &none, &vec![false; N], &vec![0.0; N]);
        if charged {
            charge_graph(t, &vec![NOT_BUILT; t.links.len()], &world.navigable, W, &mut g);
        }
        let p = tr::dijkstra(&g, &[(a, 0.0)], Some(b));
        let path = tr::path_cells(&p.pred, a, b);
        let n = path.windows(2).map(|s| t.between(W, s[0], s[1]).map_or(0, |li| t.links[li].n)).sum();
        (n, p.dist[b])
    }

    #[test]
    fn a_way_that_crossed_its_river_three_times_crosses_it_once_where_a_ford_costs() {
        // a valley of 7 by 3 cells of open lowland, and a river from its west end to its east
        // that weaves across the middle row: north of the row's meeting places, south of
        // them, north again and south again, so that the row's straight way crosses it three
        // times, and the first cell of the row and the last are on opposite banks
        let line: Vec<(f32, f32)> = vec![(0.0, 1.2), (1.8, 1.2), (2.0, 1.8), (3.8, 1.8), (4.0, 1.2), (5.8, 1.2), (6.0, 1.8), (7.0, 1.8)];
        let n = line.len();
        // (25 cubic m a second, 22 m of water: two fords of it cost more than the way round, which
        // is two diagonal steps for two straight ones, 0.83 of a cell more; a stream of 13 m, at
        // 0.4 a ford, is still forded twice rather than gone round so far)
        let v = Valley { w: 7, h: 3, height: vec![20.0; 28 * 12], line, width: vec![55.0; n], level: vec![10.0; n], flow: vec![25.0; n] };
        let middles: Vec<(f32, f32)> = (0..21).map(|c| ((c % 7) as f32 + 0.5, (c / 7) as f32 + 0.5)).collect();
        let (a, b) = (7, 13);
        // without the rule a river costs nothing: the cheapest way is the straight row, over the water three times
        let raw = links_for(&v, &[true; 21], middles.clone(), &CrossingParams::default(), false);
        let (n, cost) = crossings_on_way(&v, &raw, false, a, b);
        assert_eq!(n, 3, "the row's way crosses the river three times");
        assert!((cost - 6.0).abs() < 1e-9);
        // with it each ford costs 0.66 (22 m of water at 0.03 a metre): the way keeps to its
        // bank down the valley's side and crosses once, at its end
        let ruled = links_for(&v, &[true; 21], middles, &CrossingParams::game(), true);
        let (n, cost) = crossings_on_way(&v, &ruled, true, a, b);
        assert_eq!(n, 1, "the way crosses the river once");
        assert!((cost - (4.0 + 2.0 * 2f64.sqrt() + 0.66)).abs() < 1e-3, "{cost}");
    }
}
