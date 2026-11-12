//! Towns on the ground they keep (task A). Under the game's rules a town
//! keeps the ground it builds on, and the record says of every cell of it
//! when it joined, how many live on it and how much of it is works
//! (docs/rules/towns.md). The towns of such a run are laid out from that:
//!
//! - **How many houses.** One to every twelve residents of the cell
//!   (`world::PEOPLE_TO_A_HOUSE`, where the reason is), so the houses on the
//!   map follow the people from year to year and from town to town.
//! - **Where.** Only on the closes the cell's houses reach
//!   (`world::town_ground`, `TownCell::reach`), which gather from the side of
//!   the town's older ground and along the way out of it; the first of them
//!   are painted as built-up ground and the last are the gardens and paddocks
//!   of the town's edge, with houses among them. Streets are laid there and
//!   nowhere else: the roads, a lane
//!   where no road yet runs between a cell and the older cell it grew from,
//!   and back lanes and cross streets behind them. Plots stand at fixed
//!   places along the streets, and each has a turn (`Plot::turn`): the place
//!   of its close in the cell's order, and a number of its own. The cell's
//!   houses stand on the plots whose turn has come, so a house, once built,
//!   stays where it is.
//! - **When.** A plot is first built in the year its cell first had the
//!   houses for its turn to come, which the record's history gives
//!   (`Story::year_of`). A cell that was town ground when the record opens
//!   is dated back by the rule the simulation fills it by: three hundred
//!   people the year it joined, and the rest as it aged.
//! - **What kind and how tall.** A plot's house is a cottage of one storey,
//!   long and low on the wider plots of the lanes, and of two once its cell
//!   has grown well past it. On the roads through a town and round its
//!   market place a house that has a neighbour is rebuilt as a townhouse,
//!   the full width of its plot, once the cell has some dozens more houses
//!   than when it was built (in the style of that year), and gets a third
//!   storey only where houses stand on both sides of it and the cell has
//!   grown as much again. The back streets go up a row at a time, brick
//!   terraces from 1815. Villas stand on the lanes of ground that has little
//!   works on it. So frontages join before they rise, and nothing tall
//!   stands alone.
//!
//! A run under the port's rules has no such record and is laid out as before
//! (`houses_port`).
//!
//! **Taken over.** The first helper on this task was cut off and left no
//! account of its decisions. Read back from its code, they were these, and
//! the second helper kept every one of them unless a line below says not:
//!
//! 1. Eight people to a drawn house (`world::PEOPLE_TO_A_HOUSE`): the unit
//!    the record keeps residents in, and as many houses as the fullest
//!    ground of 1900 has room for at the size houses are drawn.
//! 2. A parcel of town ground is cut three times more, into eight closes
//!    (some eighty to a cell), and a close is built up or open.
//! 3. The order a cell's closes are built up in is fixed the year the cell
//!    joins (`world::town_order`): by the way to the market place along the
//!    roads of that year, ground off the road counting three times.
//! 4. As many closes are built up as the cell's houses need at
//!    `world::HOUSE_GROUND` each, and its works for their share of the cell.
//! 5. Houses outlast a fall of a quarter in their cell's residents
//!    (`world::HOUSES_KEEP`).
//! 6. A plot's turn is its close's place in that order plus up to five
//!    closes of its own (`world::EDGE_CLOSES`), and the cell's houses stand
//!    on the plots whose turn comes first. CHANGED: as left, the plots were
//!    those of the streets laid on this year's built-up ground, so each
//!    close that was built up could bring plots whose turn came before
//!    houses already standing, which then lost their place. Now the streets and plots
//!    are laid over all of a cell's town ground, a plot may be built on only
//!    once its turn is under the count of built-up closes, and a row's turn
//!    is its last plot's, so the order never changes (`lay_out`).
//! 7. A house's year is the year the record first shows its cell with that
//!    many houses; ground older than the record is dated back by the rule
//!    the simulation fills it by.
//! 8. Cottages of one storey, then two; townhouses on roads and market
//!    places only where a neighbour stands; a third storey only between two
//!    neighbours; terraces on rows from 1815; villas on lanes of ground with
//!    little works.
//! 9. The open closes are painted by the ground shader as paddocks,
//!    orchards, garden ground and greens within hedges (`town_edge` in
//!    terrain.wesl).
//!
//! **Taken over again.** The second helper was cut off in its turn, with
//! three changes in its tree that no commit message tells of. A third helper
//! read them back from the code and the pictures and kept all three:
//!
//! 10. Twelve people to a drawn house, not eight (item 1; the reason is at
//!     `world::PEOPLE_TO_A_HOUSE`): the streets of a cell built up whole
//!     have plots for about 120 houses, and the fullest ground of 1900 holds
//!     1,430 people.
//! 11. The works' share of a cell is no longer counted into its built-up
//!     closes (item 4): a works is drawn in a yard of its own size where
//!     `scatter.rs` stands it and the houses keep out of the yards as drawn,
//!     so counting the record's share as well left the middle of a town a
//!     pale plain (`world::closes_built`).
//! 12. Two counts of closes to a cell where there was one: those painted as
//!     built-up ground, at `world::BUILT_GROUND` a house, and the wider
//!     count the houses' plots may stand on, at `world::HOUSE_GROUND` a
//!     house (`TownCell::built`, `TownCell::reach`). Between the two the
//!     houses stand among the closes of the town's edge.
//!
//! The rule that frontages join before they rise (item 8) stands as the
//! second helper left it. What the third changed, each where it says why:
//!
//! 13. A lane behind a way stops where another way is nearer
//!     (`Laying::gives_way`), so the lanes of ways that meet make rings round
//!     a market place and no longer run through one another's plots.
//! 14. The closes of a town's first cell come in rings round its market
//!     place, not along its roads (item 3; `world::OFF_THE_ROAD_AT_MARKET`),
//!     and the lanes of that old middle are built house by house
//!     (`OLD_MIDDLE`). Every other cell is built up along its roads, as before.
//! 15. For the houses to stay where they are built: the plots round a market
//!     place stand where it comes to at its widest (`MARKET_RING`); a plot
//!     fronts a street that runs whether or not the ground the street is on
//!     is yet town ground (`Laying::runs`); and a way's plots are the same
//!     while it is a lane and once it is a road, clear of the widest a road
//!     comes to be (`WIDEST_WAY`).
//! 16. From across the island the built-up closes take the colour of their
//!     roofs (terrain.wesl, `town_roofs_far`).
//!
//! `ISOSTATE_TOWNS=1` prints a year's count of houses against residents,
//! `=towns` the same town by town, `=cells` the cells that are short of
//! plots, and `=years` lays out every year in turn and says how many houses
//! of the year before are gone and why; `ISOSTATE_TOWNS_CELL=X,Y` says what
//! became of every plot of one cell.

use super::*;

/// A quick hash for the small whole-number keys the layout looks things up
/// by, many thousands of times a year (the standard one is made to stand up
/// to hostile keys, and is several times slower). It is also the same from
/// run to run.
#[derive(Default)]
struct Quick(u64);

impl std::hash::Hasher for Quick {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_u64(b as u64);
        }
    }

    fn write_u64(&mut self, v: u64) {
        self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x517c_c1b7_2722_0a95);
    }

    fn write_u8(&mut self, v: u8) {
        self.write_u64(v as u64);
    }

    fn write_u32(&mut self, v: u32) {
        self.write_u64(v as u64);
    }

    fn write_i32(&mut self, v: i32) {
        self.write_u64(v as u32 as u64);
    }

    fn write_usize(&mut self, v: usize) {
        self.write_u64(v as u64);
    }
}

/// The maps of this file use it.
type HashMap<K, V> = std::collections::HashMap<K, V, std::hash::BuildHasherDefault<Quick>>;

/// A third ring of lanes, for cells that fill.
const FAR_LANE: f32 = 0.39;

/// What a year's layout of the kept towns came to, for measuring it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Census {
    /// cells of kept town ground, and the people the record says live on them
    pub cells: usize,
    pub residents: f32,
    /// the houses those people ask for (with those that outlast a fall), and the houses drawn
    pub wanted: usize,
    pub drawn: usize,
    pub cottages: usize,
    pub townhouses: usize,
    pub terraced: usize,
    pub villas: usize,
    /// houses of one, two and three storeys or more
    pub storeys: [usize; 3],
    /// three-storey houses with no neighbour on one side (there should be none)
    pub tall_alone: usize,
    /// houses wanted on cells that have not the plots for them: on cells with
    /// open closes left, and on cells that are built up whole
    pub short: usize,
    pub short_full: usize,
}

/// A street that may be laid on kept town ground.
struct Way {
    a: Vec2,
    b: Vec2,
    width: f32,
    id: i32,
    /// a road, which is drawn already
    road: bool,
    /// 0 the way between two cells, 1 to 3 the lanes behind it, 4 a cross street, 5 a station road
    kind: u8,
    /// for a lane or a cross street, the way it belongs to
    main: i32,
    /// the year (counted from the start) its way was first there: the first
    /// year its road ran, or the year the cell it leads out to joined its town
    since: u32,
}

/// The story of a cell's houses up to this year.
struct Story {
    /// the houses that stand this year
    houses: usize,
    /// each time the cell had more houses than ever before since it joined: how many, and the year (counted from the start)
    steps: Vec<(usize, usize)>,
    /// the year (counted from the start) the cell joined, or 0 if it was town ground when the record opens
    since: usize,
    /// the calendar year it joined
    joined: i32,
    /// the share of it under works
    works: f32,
    /// a town's first cell, with its market place
    node: bool,
}

impl Story {
    fn of(r: &Replay, k: usize, c: usize) -> Story {
        let mut since = k;
        while since > 0 && r.built[(since - 1) * r.n + c] > 0 {
            since -= 1;
        }
        let mut steps: Vec<(usize, usize)> = Vec::new();
        for y in since..=k {
            let n = (r.residents(y, c) / world::PEOPLE_TO_A_HOUSE).round() as usize;
            if steps.last().map_or(true, |s| n > s.0) {
                steps.push((n, y));
            }
        }
        Story {
            houses: world::town_houses(r, k, c),
            steps,
            since,
            joined: r.joined(k, c).unwrap_or(r.y0),
            works: r.works_share(k, c),
            node: r.header.towns.iter().any(|t| t.node == c),
        }
    }

    /// The calendar year the cell first had `n` houses, as far as this year
    /// knows. `own` is a number of the plot's own, 0 to 1. For a house that
    /// stood when the record opens the year is reckoned back as the
    /// simulation fills old ground (`isostate_core::towns::room`): three
    /// hundred people the year the cell joined, the rest as it aged, most of
    /// them early.
    fn year_of(&self, r: &Replay, k: usize, n: usize, own: f32) -> i32 {
        let Some(&(_, y)) = self.steps.iter().find(|s| s.0 >= n) else { return r.y0 + k as i32 };
        if y > self.since || self.since > 0 {
            return r.y0 + y as i32;
        }
        let age = (r.y0 - self.joined).max(0) as f32;
        let (people, first) = (n as f32 * world::PEOPLE_TO_A_HOUSE, self.steps[0].0 as f32 * world::PEOPLE_TO_A_HOUSE);
        if people <= 300.0 || first <= 300.0 || age <= 0.0 {
            // the cottages that were there when the town took the ground, or went up as it did
            return self.joined - (own * 14.0) as i32;
        }
        let share = ((people - 300.0) / (first - 300.0)).clamp(0.0, 1.0);
        let after = -30.0 * (1.0 - share * (1.0 - (-age / 30.0).exp())).max(1e-6).ln();
        self.joined + after.min(age) as i32
    }
}

/// A house plot on kept ground: a `Plot`, and its turn.
struct Lot {
    pl: Plot,
    /// the cell whose houses it is one of
    cell: u32,
    /// when its turn comes among the plots of its cell: the place of its close in the cell's order, and a little of its own
    turn: f32,
    /// what kind of street it fronts (`Way::kind`; 9 round a market place)
    kind: u8,
    /// its street is a road, or a market place
    road: bool,
    /// the year (counted from the start) it was first there to build on: the
    /// first year of its way, or the year its cell joined if that was later
    age: u32,
    /// its street among the year's streets, and how far along it the plot is (not for a market place)
    street: (u32, f32),
}

/// How near an older street a later lane may be laid: clear of the older
/// street's plots.
const OLDER_STREET: f32 = 0.055;

/// The widest a road comes to be (`ways`, a road of the third level).
const WIDEST_WAY: f32 = 0.034;

/// How far from the middle of a market place the houses round it stand: clear
/// of the widest a market place comes to be (`squares`).
const MARKET_RING: f32 = 0.09 + 0.03;

/// The old middle of a town: this many closes, the first of its first cell's
/// order (about 500 m round the market place). Its lanes were built up house
/// by house, not a row at a time, so a plot there takes its own turn and the
/// lanes behind the market place fill as soon as the roads out of it. (With
/// rows everywhere a row waited for the last of its plots, three to six
/// plots along the lane, and the town of 1750 was houses strung along its
/// roads with nothing behind them.)
const OLD_MIDDLE: u16 = 20;

/// A house that stands this year.
struct Stood {
    lot: Lot,
    /// how many houses its cell had before it, the calendar year it went up, and its roof
    before: usize,
    year: i32,
    roof: Roof,
    neighbours: (bool, bool),
}

/// The roof of a house built in a calendar year on the ground of cell `wc`, as it is in year `k`.
fn roof_in(r: &Replay, k: usize, wc: usize, year: i32, seed: Seed) -> Roof {
    let now = r.y0 + k as i32;
    let laid = (Roof::laid(year, now, seed) - r.y0).clamp(0, k as i32) as usize;
    Roof::of(year, now, r.terrain[wc] >= 2, r.trans[laid * r.n + wc] & 4 != 0, seed)
}

/// The link between two neighbouring cells as the roads number them: the
/// lower cell, and which way the other lies (east, south, south-east, south-west).
fn link(r: &Replay, a: usize, b: usize) -> Option<(usize, usize)> {
    let (lo, hi) = (a.min(b), a.max(b));
    let dir = match hi - lo {
        1 => 0,
        d if d == r.w => 1,
        d if d == r.w + 1 => 2,
        d if d + 1 == r.w => 3,
        _ => return None,
    };
    Some((lo, dir))
}

/// Every street that could be laid this year on kept town ground: the roads
/// that touch it, and those that ever did (the map leaves a diagonal road
/// out once two straight ones join its ends, but the street it was stays a
/// lane of the town); a lane from each cell to the older cell it grew from
/// where no road has run between them (when the road comes it takes the
/// lane's place, and its plots); and lanes and cross streets behind each.
/// The oldest ways first: a later way's lanes give way to the streets and
/// plots that were there before it (`lay_out`).
fn ways(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, town: &world::TownGround, sq: &[Square], stations: &[Option<Vec2>]) -> Vec<Way> {
    let on_town = |c: usize| r.built[k * r.n + c] > 0;
    // (lower cell, direction) -> the road's level, or 0 for a lane, and the year the way was first there
    let mut links: std::collections::BTreeMap<(usize, usize), (u8, u32)> = Default::default();
    let first = world::road_years(r, g, k);
    // a road is a way of the town from the year it ran, or from the year the first of its
    // two cells joined the town if that was later
    let town_since = |c: usize, m: usize, road: u32| road.max([c, m].into_iter().filter(|&c| on_town(c)).map(|c| joined_in(r, k, c)).min().unwrap_or(k as u32));
    for (c, m, lv) in crate::network::road_segments(r, k) {
        if on_town(c) || on_town(m) {
            if let Some(key) = link(r, c, m) {
                links.insert(key, (lv, town_since(c, m, first.get(&(c.min(m) as u32, c.max(m) as u32)).copied().unwrap_or(k as u32))));
            }
        }
    }
    for (&(c, m), &year) in &first {
        if on_town(c as usize) || on_town(m as usize) {
            if let Some(key) = link(r, c as usize, m as usize) {
                links.entry(key).or_insert((0, town_since(c as usize, m as usize, year)));
            }
        }
    }
    let mut cells: Vec<(&u32, &world::TownCell)> = town.cells.iter().collect();
    cells.sort_by_key(|c| *c.0);
    for (&c, cell) in cells {
        if let Some(key) = cell.parent.and_then(|p| link(r, c as usize, p as usize)) {
            links.entry(key).or_insert((0, joined_in(r, k, c as usize)));
        }
    }
    // a town with no road at all still has a street through its market place
    for s in sq {
        let node = r.header.towns[s.town].node;
        let touched = links.keys().any(|&(lo, dir)| lo == node || lo + [1, r.w, r.w + 1, r.w - 1][dir] == node);
        if !touched && on_town(node) && node % r.w + 1 < r.w {
            links.insert((node, 0), (0, joined_in(r, k, node)));
        }
    }
    let mut out = Vec::new();
    let node = |c: usize| r.header.towns.iter().any(|t| t.node == c);
    for (&(c, dir), &(lv, since)) in &links {
        let m = c + [1, r.w, r.w + 1, r.w - 1][dir];
        // the lanes behind a way stop short of a market place, and run nearly to the bend where the way only passes through
        let (short_a, short_b) = (if node(c) { 0.1 } else { 0.035 }, if node(m) { 0.1 } else { 0.035 });
        let (a, b) = (v2(cen[c]), v2(cen[m]));
        let sid = ((c * 4 + dir) * 64) as i32;
        out.push(Way { a, b, width: if lv == 3 { 0.034 } else if lv == 2 { 0.022 } else { 0.016 }, id: sid, road: lv >= 2, kind: 0, main: sid, since });
        let along = b - a;
        let len = along.length().max(1e-4);
        let u = along / len;
        let n = Vec2::new(-u.y, u.x);
        for (j, side) in [(0, -1.0f32), (1, 1.0)] {
            for (ring, off) in [(0, BACK_LANE), (1, OUTER_LANE), (2, FAR_LANE)] {
                let o = n * side * off;
                let id = sid + 2 + ring * 2 + j;
                let jit = 0.06 * (world::hash2(id, 1, 22) - 0.5);
                let jit = jit * if short_a < 0.1 && short_b < 0.1 { 0.4 } else { 1.0 };
                out.push(Way { a: a + u * (short_a + jit.max(-0.03)) + o, b: b - u * (short_b - jit.min(0.03)) + o, width: 0.014, id, road: false, kind: 1 + ring as u8, main: sid, since });
            }
        }
        let count = ((len - 0.2) / CROSS_EVERY).floor().max(0.0) as i32;
        for i in 0..count {
            let t = (0.1 + CROSS_EVERY * (i as f32 + 0.5)) / len + 0.04 * (world::hash2(sid, i, 23) - 0.5);
            let base = a + along * t.clamp(0.08, 0.92);
            for (j, side) in [(0, -1.0f32), (1, 1.0)] {
                let id = sid + 8 + i * 2 + j;
                let h = world::hash2(id, 3, 23);
                let reach = if h < 0.4 { BACK_LANE } else if h < 0.8 { OUTER_LANE } else { FAR_LANE };
                let w = n * side;
                // every other one or so starts behind the plots of the way, which it leaves
                // unbroken: a high street's frontage runs on past the backs of its side lanes
                let from = if world::hash2(id, 5, 23) < 0.55 { 0.062 } else { 0.02 };
                out.push(Way { a: base + w * from, b: base + w * reach, width: 0.012, id, road: false, kind: 4, main: sid, since });
            }
        }
    }
    for (s, st) in sq.iter().zip(stations) {
        let node = r.header.towns[s.town].node;
        if let Some(st) = *st {
            let d = (st - s.at).normalize_or_zero();
            // from the year the railway came
            let since = (0..=k).find(|&y| r.trans[y * r.n + node] & 4 != 0).unwrap_or(k) as u32;
            out.push(Way { a: s.at + d * s.radius, b: st - d * 0.03, width: 0.02, id: -1 - s.town as i32, road: false, kind: 5, main: -1 - s.town as i32, since });
        }
    }
    out.sort_by_key(|w| (w.since, w.main, w.id));
    out
}

/// The year (counted from the start) cell `c` joined its town: 0 for ground older than the record.
fn joined_in(r: &Replay, k: usize, c: usize) -> u32 {
    r.joined(k, c).map_or(k as u32, |y| ((y - r.y0).max(0) as u32).min(k as u32))
}

//// What was laid out for a year: the market places, the lanes to draw, the
/// houses that stand and the story of each cell that has any.
pub(super) struct Layout {
    sq: Vec<Square>,
    /// the lengths of lane to draw, each a line of points and a width
    lanes: Vec<(Vec<Vec2>, f32)>,
    /// the houses, street by street and in order along each
    standing: Vec<Stood>,
    stories: HashMap<u32, Story>,
    /// how long the streets, the plots and the turns took, in milliseconds
    ms: [f64; 3],
    /// with ISOSTATE_TOWNS set, why each plot with no house has none: 1 its
    /// turn has not come, 2 its cell has all its houses, 3 a works' yard, a
    /// churchyard or water, 4 the market place, the station or the way to
    /// the church, 5 a street, 6 a house already standing too near, 7 its
    /// street is not laid there
    why: HashMap<((i32, u8), i32), u8>,
}

impl Layout {
    /// The plots that carry a house: the street and side, the place along it, and the cell.
    pub(super) fn plots(&self) -> impl Iterator<Item = ((i32, u8), i32, u32)> + '_ {
        self.standing.iter().map(|s| (s.lot.pl.run, s.lot.pl.index, s.lot.cell))
    }
}

/// A street that may be laid, and what has been asked of it so far.
struct Street {
    way: Way,
    len: f32,
    /// it is walked in this many steps of about 60 m
    steps: usize,
    /// at each step: 0 not asked yet, 1 not laid there, 2 laid, 3 laid on
    /// built-up ground, 4 not laid only because the ground is not town ground
    state: Vec<std::cell::Cell<u8>>,
}

impl Street {
    fn at(&self, step: usize) -> Vec2 {
        self.way.a.lerp(self.way.b, step as f32 / self.steps as f32)
    }

    /// a lane behind a way, or a cross street: not a way itself, nor a station road
    fn lane(&self) -> bool {
        self.way.kind != 0 && self.way.kind != 5
    }
}

/// The streets of a year, laid only where and when something asks: most of a
/// town's ground is not built on, and a street is walked only by the plots
/// whose turn has come and by the lanes that are drawn.
struct Laying<'a> {
    g: &'a Ground,
    town: &'a world::TownGround,
    near_plot: &'a dyn Fn(f32, f32) -> bool,
    sq: &'a [Square],
    /// the stations of the towns the railway has come to (`station_at`)
    stations: Vec<Vec2>,
    parishes: &'a [Parish],
    /// the cells of the map with a market place, a station or a church in them or next to them
    by_square: std::collections::HashSet<(i32, i32)>,
    closes: std::cell::RefCell<world::CloseLookup<'a>>,
    /// the year (counted from the start) each cell of town ground joined its town
    joined: HashMap<u32, u32>,
    streets: Vec<Street>,
    /// the streets that run through or by each square of the map, `NEAR` squares to a cell's side
    by_cell: HashMap<(i32, i32), Vec<u32>>,
    /// the ways (the roads and the lanes between cells, not the lanes behind
    /// them): each one's ends, its number and the year it was first there
    mains: Vec<(Vec2, Vec2, i32, u32)>,
    /// a way among `mains`, by its number
    main_at: HashMap<i32, u32>,
    /// the ways that come within `FAR_LANE` and a little of each cell of the map
    mains_by_cell: HashMap<(i32, i32), Vec<u32>>,
}

/// How far a spot is from the line between two others.
fn off_line(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let t = ((p - a).dot(b - a) / a.distance_squared(b).max(1e-9)).clamp(0.0, 1.0);
    p.distance(a + (b - a) * t)
}

impl Laying<'_> {
    /// The close a spot lies in, if it is ground of a kept town cell: the cell, the close's
    /// place in the cell's order, and whether it is built up this year.
    fn close_at(&self, p: Vec2) -> Option<(u32, u16, bool)> {
        let (parcel, sub) = self.closes.borrow_mut().at(p.x, p.y);
        let rank = self.town.rank_of(parcel, sub)?;
        Some((self.g.land.wcell[parcel as usize], rank, self.town.built_up(parcel, sub)))
    }

    /// The same, if the spot is dry land.
    fn town_at(&self, p: Vec2) -> Option<(u32, u16, bool)> {
        self.close_at(p).filter(|_| world::height_at(self.g, p.x, p.y) >= 0.05)
    }

    /// By a market place or a station, or crowding a church or the way to its gate.
    fn in_square(&self, p: Vec2) -> bool {
        self.by_square.contains(&(p.x.floor() as i32, p.y.floor() as i32))
            && (self.sq.iter().any(|s| p.distance(s.at) < s.radius + 0.012) || self.stations.iter().any(|st| p.distance(*st) < 0.08) || self.parishes.iter().any(|c| c.crowded_by(p)))
    }

    /// Whether a lane behind a way, or a cross street off it, has come at `p`
    /// to ground that lies nearer another way: there it stops, and the other
    /// way's lanes take the ground. So where ways meet, at a market place
    /// above all, the lanes behind them make rings of streets round it and do
    /// not run across one another. (As the second helper left it every way
    /// laid its lanes the whole of its length, those of ways that met crossed
    /// in the angle between them, and of the plots there whose turn came six
    /// in ten were lost to a street that ran through them: the middle of a
    /// town was the thinnest part of it.) A way that came later takes no
    /// ground from the lanes of one that was there before it, so a new road
    /// moves no house.
    fn gives_way(&self, s: &Street, p: Vec2) -> bool {
        let Some(&own) = self.main_at.get(&s.way.main) else { return false };
        let (a, b, _, _) = self.mains[own as usize];
        let off = off_line(p, a, b);
        self.mains_by_cell.get(&(p.x.floor() as i32, p.y.floor() as i32)).is_some_and(|near| {
            near.iter().any(|&j| {
                let (a, b, _, since) = self.mains[j as usize];
                j != own && since <= s.way.since && off_line(p, a, b) < off - 1e-3
            })
        })
    }

    /// Whether street `i` is laid at one of its steps: over dry town ground,
    /// and, unless it is a road, clear of the works' yards, the churchyards,
    /// the water and the market places. A lane that came to ground already
    /// town ground also gives way to the streets that were there before it,
    /// and to their plots; and any lane gives way where another way is nearer
    /// (`gives_way`).
    fn laid(&self, i: usize, step: usize) -> bool {
        matches!(self.asked(i, step), 2 | 3)
    }

    /// Whether street `i` runs at one of its steps: it is laid there, or
    /// would be if the ground were town ground. A plot fronts a street that
    /// runs, and keeps out of one: where a street comes to the edge of its
    /// town the ground beyond is not town ground until the town takes it, and
    /// plots that waited for that came into their cell's order late, before
    /// houses already standing, which then counted as built years after they
    /// were (and the last of them went).
    fn runs(&self, i: usize, step: usize) -> bool {
        self.asked(i, step) >= 2
    }

    /// What street `i` is at one of its steps (`Street::state`).
    fn asked(&self, i: usize, step: usize) -> u8 {
        let s = &self.streets[i];
        match s.state[step].get() {
            0 => {}
            asked => return asked,
        }
        let p = s.at(step);
        let at = self.town_at(p);
        let dry = at.is_some() || world::height_at(self.g, p.x, p.y) >= 0.05;
        let ok = dry
            && (s.way.road
                || (!(self.near_plot)(p.x, p.y)
                    && !self.in_square(p)
                    && !(s.lane() && (self.gives_way(s, p) || at.is_some_and(|a| s.way.since > self.joined[&a.0]) && self.in_street(p, OLDER_STREET, |o| o.way.since < s.way.since)))));
        let state = match (ok, at) {
            (false, _) => 1,
            (true, None) => 4,
            (true, Some(a)) => 2 + a.2 as u8,
        };
        s.state[step].set(state);
        state
    }

    /// Whether a spot is within `clear` of the edge of a laid street, among those that `counts` passes.
    fn in_street(&self, p: Vec2, clear: f32, counts: impl Fn(&Street) -> bool) -> bool {
        let Some(near) = self.by_cell.get(&((p.x * NEAR).floor() as i32, (p.y * NEAR).floor() as i32)) else { return false };
        near.iter().any(|&j| {
            let s = &self.streets[j as usize];
            if !counts(s) {
                return false;
            }
            let along = s.way.b - s.way.a;
            let t = ((p - s.way.a).dot(along) / (s.len * s.len)).clamp(0.0, 1.0);
            // (a way counts at the widest a road comes to be, so that one widened takes no plot)
            let half = if s.way.kind == 0 { WIDEST_WAY * 0.5 } else { s.way.width * 0.5 };
            p.distance(s.way.a + along * t) < half + clear && self.runs(j as usize, (t * s.steps as f32).round() as usize)
        })
    }

    /// Whether street `i` runs at share `t` of its length: at the steps on either side.
    fn runs_at(&self, i: usize, t: f32) -> bool {
        let x = t.clamp(0.0, 1.0) * self.streets[i].steps as f32;
        self.runs(i, x.floor() as usize) && self.runs(i, x.ceil() as usize)
    }

    /// The lengths of street `i` within a span of it that are laid (and, if
    /// asked, on built-up ground), as lines of points.
    fn lengths(&self, i: usize, t0: f32, t1: f32, built: bool, into: &mut Vec<(Vec<Vec2>, f32)>) {
        let s = &self.streets[i];
        let (first, last) = ((t0.clamp(0.0, 1.0) * s.steps as f32).floor() as usize, (t1.clamp(0.0, 1.0) * s.steps as f32).ceil() as usize);
        let mut run: Vec<Vec2> = Vec::new();
        for step in first..=last.min(s.steps) {
            if self.laid(i, step) && (!built || s.state[step].get() == 3) {
                run.push(s.at(step));
            } else if run.len() > 1 {
                into.push((std::mem::take(&mut run), s.way.width));
            } else {
                run.clear();
            }
        }
        if run.len() > 1 {
            into.push((run, s.way.width));
        }
    }
}

/// The streets are looked up by squares this many to a cell's side (`Laying::by_cell`).
const NEAR: f32 = 4.0;

/// Houses on one lane further apart than this have each their own length of it.
const LANE_GAP: f32 = 0.2;

/// The streets and the houses of the kept towns for year `k`.
///
/// What is fixed, whatever the year: the streets that may be laid over a
/// cell's town ground, built up or not, the plots along them, and each
/// plot's turn. What the year decides is how many closes of the cell are
/// built up and how many houses it has. A plot may be built on once its turn
/// is under the count of built-up closes, and the plots that may be are
/// built on in the order of their turns, so the houses of a year are those
/// of the year before and more: a house, once built, stays where it is.
/// (What can still move one: a works that takes its ground or gives it up, a
/// road laid where none ran, the market place widening as its town grows,
/// and a fall of more than a quarter in its cell's residents.)
pub(super) fn lay_out(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, parishes: &[Parish], near_plot: &dyn Fn(f32, f32) -> bool, land: &world::YearLand) -> Layout {
    let town = &land.town;
    let sq = squares(r, cen, k);
    let t0 = std::time::Instant::now();
    let mut by_square: std::collections::HashSet<(i32, i32)> = Default::default();
    // (the station stands beside the line where it passes the town: `station_at`)
    let stations: Vec<Option<Vec2>> = sq.iter().map(|s| station_at(r, g, cen, k, s).map(|(at, _, _)| at)).collect();
    for p in sq.iter().map(|s| s.at).chain(stations.iter().flatten().copied()).chain(parishes.iter().map(|c| c.at)) {
        for (dx, dy) in (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dx, dy))) {
            by_square.insert((p.x.floor() as i32 + dx, p.y.floor() as i32 + dy));
        }
    }
    let mut lay = Laying {
        g,
        town,
        near_plot,
        sq: &sq,
        stations: stations.iter().flatten().copied().collect(),
        parishes,
        by_square,
        closes: std::cell::RefCell::new(world::CloseLookup::new(g)),
        joined: town.cells.keys().map(|&c| (c, joined_in(r, k, c as usize))).collect(),
        streets: Vec::new(),
        by_cell: HashMap::default(),
        mains: Vec::new(),
        main_at: HashMap::default(),
        mains_by_cell: HashMap::default(),
    };
    for way in ways(r, g, cen, k, town, &sq, &stations) {
        let len = way.a.distance(way.b).max(1e-4);
        let steps = ((len / 0.03).ceil() as usize).max(1);
        // every quarter of a cell of the map the street comes within a tenth of a cell of
        let (lo, hi) = ((way.a.min(way.b) - Vec2::splat(0.1)) * NEAR, (way.a.max(way.b) + Vec2::splat(0.1)) * NEAR);
        for cy in lo.y.floor() as i32..=hi.y.floor() as i32 {
            for cx in lo.x.floor() as i32..=hi.x.floor() as i32 {
                let mid = Vec2::new(cx as f32 + 0.5, cy as f32 + 0.5) / NEAR;
                let t = ((mid - way.a).dot(way.b - way.a) / (len * len)).clamp(0.0, 1.0);
                if mid.distance(way.a.lerp(way.b, t)) < 0.1 + 0.71 / NEAR {
                    lay.by_cell.entry((cx, cy)).or_default().push(lay.streets.len() as u32);
                }
            }
        }
        if way.kind == 0 {
            // every cell of the map the way comes within its furthest lane of
            let reach = FAR_LANE + 0.05;
            let (lo, hi) = (way.a.min(way.b) - Vec2::splat(reach), way.a.max(way.b) + Vec2::splat(reach));
            for cy in lo.y.floor() as i32..=hi.y.floor() as i32 {
                for cx in lo.x.floor() as i32..=hi.x.floor() as i32 {
                    if off_line(Vec2::new(cx as f32 + 0.5, cy as f32 + 0.5), way.a, way.b) < reach + 0.71 {
                        lay.mains_by_cell.entry((cx, cy)).or_default().push(lay.mains.len() as u32);
                    }
                }
            }
            lay.main_at.insert(way.id, lay.mains.len() as u32);
            lay.mains.push((way.a, way.b, way.id, way.since));
        }
        lay.streets.push(Street { way, len, steps, state: vec![std::cell::Cell::new(0); steps + 1] });
    }
    let lay = lay;
    // (the closes its houses reach: they are built up closely on the first of them, and stand
    // among the closes of the town's edge on the last)
    let built: HashMap<u32, f32> = town.cells.iter().map(|(&c, cell)| (c, cell.reach as f32)).collect();
    let built_of = |c: u32| built.get(&c).copied().unwrap_or(0.0);
    // the towns' first cells, each with its market place
    let nodes: std::collections::HashSet<u32> = r.header.towns.iter().map(|t| t.node as u32).collect();
    // the rows of one side of a street, each on one cell, with a plot on a close that is not built up
    let mut dead: Vec<(i32, u32)> = Vec::new();
    // (for ISOSTATE_TOWNS: why each plot that carries no house does not)
    let telling = std::env::var_os("ISOSTATE_TOWNS").is_some();
    let mut why: HashMap<((i32, u8), i32), u8> = HashMap::default();
    // (ISOSTATE_TOWNS_CELL=X,Y: what became of every plot of one cell, by the kind of its street)
    let watched: Option<u32> = std::env::var("ISOSTATE_TOWNS_CELL").ok().and_then(|v| {
        let (x, y) = v.split_once(',')?;
        Some((y.trim().parse::<usize>().ok()? * r.w + x.trim().parse::<usize>().ok()?) as u32)
    });
    let fates = std::cell::RefCell::new([[0usize; 8]; 10]);
    let note = |cell: u32, kind: u8, fate: u8| {
        if watched == Some(cell) {
            fates.borrow_mut()[kind as usize][fate as usize] += 1;
        }
    };
    let mut tell = |run: (i32, u8), index: i32, reason: u8| {
        if telling {
            why.insert((run, index), reason);
        }
    };

    let t1 = std::time::Instant::now();
    // plots along both sides of every street, at fixed places along it, then round each market place
    let mut cands: Vec<Lot> = Vec::new();
    // how many closes of a cell were built up the year before a way came to it
    let mut built_before: HashMap<(u32, u32), f32> = HashMap::default();
    // one side of a street: its plots on town ground, each with the row it is in and its turn
    let mut side_lots: Vec<(Lot, f32)> = Vec::new();
    for (si, st) in lay.streets.iter().enumerate() {
        let s = &st.way;
        if s.kind == 5 {
            continue;
        }
        let along = s.b - s.a;
        let len = st.len;
        let u = along / len;
        let n = Vec2::new(-u.y, u.x);
        // streets differ: the width of their plots and how far their houses stand back are the street's own
        let (h1, h2) = (world::hash2(s.id, 5, 31), world::hash2(s.id, 6, 31));
        // (a way's plots are the same whether it is yet a road or still a lane, and
        // whatever the road's width: as taken over a lane had wider plots, set further
        // back, with later turns, and the year the road came, or was widened, the whole
        // of its frontage was laid out again and its houses moved or went)
        let spacing = match s.kind {
            0 => 0.05 + 0.012 * h1,
            1 => houses::BACK_PLOT_WIDTH,
            3 => 0.056 + 0.01 * h1,
            _ => houses::BACK_PLOT_WIDTH + 0.006 * h1,
        };
        let depth = houses::PLOT.1;
        let back = if s.kind == 0 { 0.003 * h2 } else if s.kind == 3 { 0.005 * h2 } else { 0.0 };
        // a way's frontage stands clear of the widest road it may come to be
        let half = if s.kind == 0 { WIDEST_WAY * 0.5 } else { s.width * 0.5 };
        let set_back = half + depth * 0.5 + houses::SET_BACK + back;
        let yaw_along = (-u.y).atan2(u.x);
        let count = (len / spacing).floor() as i32;
        let t_of = |i: i32| (i as f32 + 0.5) * spacing / len;
        // a family of colours to a street: most streets keep their houses' second number near one of their own
        let family = (world::hash2(s.id, 7, 31) < 0.7).then(|| 0.16 + 0.68 * world::hash2(s.id, 8, 31));
        for (j, side) in [(0u8, -1.0f32), (1, 1.0)] {
            let id = s.id * 4 + j as i32;
            side_lots.clear();
            dead.clear();
            // the back streets are built a row at a time: rows of three to six plots, fixed per street
            let (mut row, mut row_end) = (-1, 0);
            for i in 0..count {
                if s.kind != 0 && i >= row_end {
                    row += 1;
                    row_end = i + 3 + (world::hash2(id, row, 28) * 4.0) as i32;
                }
                let p = s.a + along * t_of(i) + n * side * set_back;
                // (its close is looked up where the plot would stand on a road of middling
                // width, so that a road widened does not move its plots to another close)
                let Some((c, rank, _)) = lay.close_at(p + n * side * (0.011 - half)) else { continue };
                // a plot on a lane is one of a row, but for those of a town's old middle
                let in_row = s.kind != 0 && !(rank < OLD_MIDDLE && nodes.contains(&c));
                if rank as f32 >= built_of(c) {
                    // its turn cannot have come, nor that of the row it is in
                    if in_row {
                        dead.push((row, c));
                    }
                    tell((id, j), i, 1);
                    note(c, s.kind, 1);
                    continue;
                }
                let (pu, pv) = (world::hash2(id, i, 24), world::hash2(id, i, 25));
                // the frontage of the way itself fills before the lanes behind it: a high
                // street is built up whole from its town's end
                let own = if s.kind == 0 { 0.3 * pu } else { 0.25 + 0.75 * pu };
                let mut turn = rank as f32 + world::EDGE_CLOSES * own;
                // a way that came to the cell after it was town ground: its plots take their
                // turn after all the ground that was built up before it came
                let joined = lay.joined[&c];
                if s.since > joined {
                    let was = *built_before.entry((c, s.since)).or_insert_with(|| {
                        let y = s.since as usize - 1;
                        town.cells.get(&c).map_or(0.0, |cell| world::closes_built(&cell.order, world::town_houses(r, y, c as usize)).1 as f32)
                    });
                    if turn < was {
                        turn = was + turn * 1e-3;
                    }
                }
                let v = family.map_or(pv, |f| (f + 0.3 * (pv - 0.5)).clamp(0.0, 0.999));
                let pl = Plot { p, yaw: yaw_along + if side > 0.0 { std::f32::consts::PI } else { 0.0 }, run: (id, j), row: in_row.then_some(row), high: s.kind == 0, index: i, width: spacing, depth, need: 0.0, u: pu, v };
                side_lots.push((Lot { pl, cell: c, turn, kind: s.kind, road: s.road, age: s.since.max(joined), street: (0, 0.0) }, t_of(i)));
            }
            // a row goes up together: its turn is that of the last of its plots in the cell
            // (a row that runs over a cell's edge is two, one to each cell)
            for a in 0..side_lots.len() {
                if let Some(ri) = side_lots[a].0.pl.row {
                    let cell = side_lots[a].0.cell;
                    // (a row is six plots at most, and its plots follow one another)
                    let near = &side_lots[a.saturating_sub(6)..(a + 7).min(side_lots.len())];
                    let last = near.iter().filter(|(l, _)| l.pl.row == Some(ri) && l.cell == cell).map(|(l, _)| l.turn).fold(f32::MIN, f32::max);
                    side_lots[a].0.turn = side_lots[a].0.turn.max(last);
                }
            }
            for (lot, t) in side_lots.drain(..) {
                // a plot may be built on once its turn is under the count of its cell's built-up closes
                if lot.turn >= built_of(lot.cell) || lot.pl.row.is_some_and(|ri| dead.contains(&(ri, lot.cell))) {
                    tell(lot.pl.run, lot.pl.index, 1);
                    note(lot.cell, lot.kind, 1);
                } else {
                    // (whether its street is laid there is asked when its turn comes, if its
                    // cell still has houses to build: most plots are never asked)
                    cands.push(Lot { street: (si as u32, t), ..lot });
                }
            }
        }
    }
    for s in &sq {
        // (the plots stand where the market place comes to at its widest, whatever its
        // width this year: it widens as its town grows, and at the year's own width the
        // houses round it shuffled outward every year the town grew)
        let count = ((std::f32::consts::TAU * MARKET_RING) / 0.05).floor() as i32;
        for i in 0..count {
            let a = i as f32 / count as f32 * std::f32::consts::TAU;
            let dirv = Vec2::new(a.cos(), a.sin());
            let p = s.at + dirv * MARKET_RING;
            let id = -1000 - s.town as i32;
            let pu = world::hash2(id, i, 26);
            let Some((c, _, _)) = lay.town_at(p) else { continue };
            // the market place is the first of a town to be built round
            cands.push(Lot {
                pl: Plot { p, yaw: (dirv.y).atan2(-dirv.x) + std::f32::consts::FRAC_PI_2, run: (id, 0), row: None, high: true, index: i, width: 0.05, depth: houses::PLOT.1, need: 0.0, u: pu, v: world::hash2(id, i, 27) },
                cell: c,
                turn: -2.0 + pu,
                kind: 9,
                road: true,
                age: lay.joined[&c],
                street: (0, 0.0),
            });
        }
    }

    let t2 = std::time::Instant::now();
    if std::env::var_os("ISOSTATE_TIMING").is_some() {
        eprintln!("timing kept towns: {} streets, {} plots whose turn has come", lay.streets.len(), cands.len());
    }
    // the older ground first, and on each cell in the order the turns come: the plots that
    // are built keep their ground against later ones
    // (plots whose turns come together, as a row's do, keep the order they were found in:
    // street by street, the oldest ways first, and along each street)
    let joined = |c: u32| r.built[k * r.n + c as usize];
    let mut order: Vec<(u8, u32, f32, u32)> = cands.iter().enumerate().map(|(i, l)| (joined(l.cell), l.cell, l.turn, i as u32)).collect();
    order.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.total_cmp(&b.2)));
    let mut cands: Vec<Option<Lot>> = cands.into_iter().map(Some).collect();
    let bucket = |p: Vec2| ((p.x / 0.05).floor() as i32, (p.y / 0.05).floor() as i32);
    let mut taken: HashMap<(i32, i32), Vec<Vec2>> = HashMap::default();
    let mut stories: HashMap<u32, (Story, usize)> = HashMap::default();
    let mut standing: Vec<Stood> = Vec::new();
    // whether a row came in: the whole of it does, or none
    let mut row_in: HashMap<(i32, i32, u32), bool> = HashMap::default();
    for (_, _, _, i) in order {
        let Some(lot) = cands[i as usize].take() else { continue };
        let p = lot.pl.p;
        // has its cell houses left to build? (asked first: it is the cheapest to answer)
        let (story, built) = stories.entry(lot.cell).or_insert_with(|| (Story::of(r, k, lot.cell as usize), 0));
        let room = *built < story.houses;
        let row = lot.pl.row.map(|ri| (lot.pl.run.0, ri, lot.cell));
        match row.map(|key| row_in.get(&key).copied()) {
            Some(Some(false)) => {
                tell(lot.pl.run, lot.pl.index, 2);
                note(lot.cell, lot.kind, 2);
                continue;
            }
            Some(Some(true)) => {}
            _ if !room => {
                if let Some(key) = row {
                    row_in.insert(key, false);
                }
                tell(lot.pl.run, lot.pl.index, 2);
                note(lot.cell, lot.kind, 2);
                continue;
            }
            _ => {}
        }
        if lot.kind != 9 && (!lay.runs_at(lot.street.0 as usize, lot.street.1) || lay.town_at(p).is_none()) {
            tell(lot.pl.run, lot.pl.index, 7);
            note(lot.cell, lot.kind, 7);
            continue;
        }
        if near_plot(p.x, p.y) {
            tell(lot.pl.run, lot.pl.index, 3);
            note(lot.cell, lot.kind, 3);
            continue;
        }
        if lay.in_square(p) {
            tell(lot.pl.run, lot.pl.index, 4);
            note(lot.cell, lot.kind, 4);
            continue;
        }
        // in a way, or in a lane that is no younger than the plot (a later lane runs round the plots it finds)
        if lay.in_street(p, if lot.kind == 9 { 0.03 } else { 0.02 }, |o| !o.lane() || o.way.since <= lot.age) {
            tell(lot.pl.run, lot.pl.index, 5);
            note(lot.cell, lot.kind, 5);
            continue;
        }
        let c = bucket(p);
        if (-1..=1).any(|dy| (-1..=1).any(|dx| taken.get(&(c.0 + dx, c.1 + dy)).is_some_and(|v| v.iter().any(|q| q.distance(p) < 0.042)))) {
            tell(lot.pl.run, lot.pl.index, 6);
            note(lot.cell, lot.kind, 6);
            continue;
        }
        if let Some(key) = row {
            row_in.insert(key, true);
        }
        taken.entry(c).or_default().push(p);
        let seed = Seed::new(lot.pl.u, lot.pl.v);
        let before = *built;
        *built += 1;
        let year = story.year_of(r, k, before + 1, lot.pl.u);
        let roof = roof_in(r, k, lot.cell as usize, year, seed);
        note(lot.cell, lot.kind, 0);
        standing.push(Stood { lot, before, year, roof, neighbours: (false, false) });
    }
    // a row shares the year and the place in the count of its first house
    let mut row_first: HashMap<(i32, i32, u32), (usize, i32)> = HashMap::default();
    for s in &standing {
        if let Some(ri) = s.lot.pl.row {
            let e = row_first.entry((s.lot.pl.run.0, ri, s.lot.cell)).or_insert((s.before, s.year));
            if s.before < e.0 {
                *e = (s.before, s.year);
            }
        }
    }
    for s in &mut standing {
        if let Some(ri) = s.lot.pl.row {
            (s.before, s.year) = row_first[&(s.lot.pl.run.0, ri, s.lot.cell)];
        }
    }
    // street by street, and which houses have a neighbour on the next plot along
    standing.sort_by_key(|s| (s.lot.pl.run, s.lot.pl.index));
    for i in 0..standing.len() {
        let next_to = |j: usize| standing[j].lot.pl.run == standing[i].lot.pl.run && (standing[j].lot.pl.index - standing[i].lot.pl.index).abs() == 1;
        let (before, after) = (i > 0 && next_to(i - 1), i + 1 < standing.len() && next_to(i + 1));
        standing[i].neighbours = if standing[i].lot.pl.run.1 == 1 { (after, before) } else { (before, after) };
    }

    // the lanes, where houses front them: a lane is drawn along each group of houses it has,
    // a cross street from where it starts to its last house (or to the lane it reaches, if
    // that has houses there), the way out of an older cell over its built-up ground and
    // along its houses, and the station road whole
    let way_at: HashMap<i32, usize> = lay.streets.iter().enumerate().map(|(i, s)| (s.way.id, i)).collect();
    let mut fronts: HashMap<usize, Vec<f32>> = HashMap::default();
    for s in standing.iter().filter(|s| s.lot.kind != 9) {
        if let Some(&i) = way_at.get(&s.lot.pl.run.0.div_euclid(4)) {
            let w = &lay.streets[i].way;
            fronts.entry(i).or_default().push((s.lot.pl.p - w.a).dot(w.b - w.a) / w.a.distance_squared(w.b).max(1e-9));
        }
    }
    // the groups of houses along a street: from the first of each to its last
    let groups = |i: usize| -> Vec<(f32, f32)> {
        let mut ts = fronts.get(&i).cloned().unwrap_or_default();
        ts.sort_by(f32::total_cmp);
        let gap = LANE_GAP / lay.streets[i].len;
        let mut out: Vec<(f32, f32)> = Vec::new();
        for t in ts {
            match out.last_mut() {
                Some(last) if t - last.1 <= gap => last.1 = t,
                _ => out.push((t, t)),
            }
        }
        out
    };
    let mut lanes: Vec<(Vec<Vec2>, f32)> = Vec::new();
    for (i, st) in lay.streets.iter().enumerate() {
        let s = &st.way;
        if s.road {
            continue;
        }
        let half = 0.03 / st.len;
        match s.kind {
            0 => {
                lay.lengths(i, 0.0, 1.0, true, &mut lanes);
                for (a, b) in groups(i) {
                    lay.lengths(i, a - half, b + half, false, &mut lanes);
                }
            }
            5 => lay.lengths(i, 0.0, 1.0, false, &mut lanes),
            4 => {
                // the lane it runs out to, and whether houses stand on that lane where it arrives
                let ring = if st.len > 0.29 { 2 } else if st.len > 0.15 { 1 } else { 0 };
                let lane = s.main + 2 + ring * 2 + (s.id - s.main - 8).rem_euclid(2);
                let arrives = way_at.get(&lane).is_some_and(|&l| {
                    let w = &lay.streets[l];
                    let t = (s.b - w.way.a).dot(w.way.b - w.way.a) / (w.len * w.len);
                    let slack = 0.1 / w.len;
                    groups(l).iter().any(|&(a, b)| t >= a - slack && t <= b + slack)
                });
                match (arrives, groups(i).last()) {
                    (true, _) => lay.lengths(i, 0.0, 1.0, false, &mut lanes),
                    (false, Some(&(_, last))) => lay.lengths(i, 0.0, last + half, false, &mut lanes),
                    _ => {}
                }
            }
            _ => {
                for (a, b) in groups(i) {
                    lay.lengths(i, a - half, b + half, false, &mut lanes);
                }
            }
        }
    }
    if let Some(c) = watched {
        let cell = town.cells.get(&c);
        eprintln!(
            "towns {}: cell {},{}: {} houses, {} closes of {} built up and {} reached. Plots by street (way, back lane, outer lane, far lane, cross street, -, -, -, -, market place) as stands / turn not come / cell has its houses / yard or water / market place / a street / a house too near / street not laid:",
            r.y0 + k as i32, c as usize % r.w, c as usize / r.w, cell.map_or(0, |c| c.houses), cell.map_or(0, |c| c.built), cell.map_or(0, |c| c.closes), cell.map_or(0, |c| c.reach)
        );
        for (kind, f) in fates.borrow().iter().enumerate().filter(|(_, f)| f.iter().any(|&n| n > 0)) {
            eprintln!("towns   kind {kind}: {} / {} / {} / {} / {} / {} / {} / {}", f[0], f[1], f[2], f[3], f[4], f[5], f[6], f[7]);
        }
    }
    let t3 = std::time::Instant::now();
    let ms = |a: std::time::Instant, b: std::time::Instant| (b - a).as_secs_f64() * 1000.0;
    drop(lay);
    Layout { sq, lanes, standing, stories: stories.into_iter().map(|(c, (story, _))| (c, story)).collect(), ms: [ms(t0, t1), ms(t1, t2), ms(t2, t3)], why }
}

/// The houses of the kept towns for year `k`, built into `into`.
#[allow(clippy::too_many_arguments)]
pub(super) fn houses_kept(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, lk: &Look, parishes: &[Parish], near_plot: &dyn Fn(f32, f32) -> bool, land: &world::YearLand, into: &mut Into) -> Census {
    let town = &land.town;
    let layout = lay_out(r, g, cen, k, parishes, near_plot, land);
    steadiness(r, k, &layout);
    let Layout { sq, lanes, standing, stories, ms: laying, .. } = layout;
    let t3 = std::time::Instant::now();
    let colour = *lk.ground.street_earth;
    for (line, width) in &lanes {
        let pts: Vec<Vec3> = line.iter().map(|p| Vec3::new(p.x, world::height_at(g, p.x, p.y).max(0.0) + 0.006, p.y)).collect();
        into.houses.ribbon(&pts, *width, colour);
    }

    // the houses, street by street
    let mut census = Census::default();
    let mut on_cell: HashMap<u32, usize> = HashMap::default();
    for s in &standing {
        *on_cell.entry(s.lot.cell).or_default() += 1;
    }
    for (&c, cell) in &town.cells {
        census.cells += 1;
        census.residents += r.residents(k, c as usize);
        census.wanted += cell.houses as usize;
        // the houses a cell has not the plots for: on ground that is all built up, or not
        let short = (cell.houses as usize).saturating_sub(on_cell.get(&c).copied().unwrap_or(0));
        if cell.reach >= cell.closes {
            census.short_full += short;
        } else {
            census.short += short;
        }
        // ISOSTATE_TOWNS=cells names the cells that are five houses short or more
        if short >= 5 && std::env::var("ISOSTATE_TOWNS").as_deref() == Ok("cells") {
            eprintln!(
                "towns {}: cell {},{} joined {:?}: {:.0} residents, {} houses wanted, {} short; {} of {} dry closes built up, {:.2} under works",
                r.y0 + k as i32, c as usize % r.w, c as usize / r.w, r.joined(k, c as usize), r.residents(k, c as usize), cell.houses, short, cell.reach, cell.closes, world::town_works(r, k, c as usize)
            );
        }
    }
    // ISOSTATE_TOWNS=towns says, town by town, how the houses drawn follow the people
    if std::env::var("ISOSTATE_TOWNS").as_deref() == Ok("towns") {
        for line in by_town(r, k, town, &on_cell) {
            eprintln!("towns {}: {line}", r.y0 + k as i32);
        }
    }
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let mut i = 0;
    while i < standing.len() {
        let first = &standing[i];
        let story = &stories[&first.lot.cell];
        let mut j = i + 1;
        if first.lot.pl.row.is_some() && first.year >= TERRACE_ERA {
            while j < standing.len() && standing[j].lot.pl.run == first.lot.pl.run && standing[j].lot.pl.row == first.lot.pl.row && standing[j].lot.cell == first.lot.cell && standing[j].lot.pl.index == standing[j - 1].lot.pl.index + 1 {
                j += 1;
            }
        }
        if j - i >= 2 {
            // a brick terrace: two storeys, and a third only in a cell that has filled long since
            let (a, b) = (first.lot.pl.p, standing[j - 1].lot.pl.p);
            let mid = (a + b) * 0.5;
            let seed = Seed::new(first.lot.pl.u, first.lot.pl.v);
            let storeys = 2 + (story.houses >= first.before + 80 + (27.0 * seed.u) as usize) as u8;
            let t = Terrace {
                houses: j - i,
                length: a.distance(b) + first.lot.pl.width * 0.96,
                depth: first.lot.pl.depth,
                storeys,
                roof: first.roof,
                era: Era::of(first.year),
                upland: r.terrain[first.lot.cell as usize] >= 2,
                detail: into.lod.at(mid.x, mid.y),
                seed,
            };
            houses::terrace(into.houses, into.smoke, &Site::on(ground(g, mid.x, mid.y), first.lot.pl.yaw, &height), &t, lk);
            census.terraced += j - i;
            census.storeys[(storeys as usize - 1).min(2)] += j - i;
        } else {
            for s in &standing[i..j] {
                one_house(into, r, k, lk, g, &height, s, &stories[&s.lot.cell], sq.iter().any(|q| q.big && s.lot.kind == 9 && q.at.distance(s.lot.pl.p) < 0.2), &mut census);
            }
        }
        i = j;
    }
    census.drawn = standing.len();
    if std::env::var_os("ISOSTATE_TIMING").is_some() {
        eprintln!("timing kept towns: streets {:.0} ms, plots {:.0} ms, turns {:.0} ms, houses {:.0} ms", laying[0], laying[1], laying[2], t3.elapsed().as_secs_f64() * 1000.0);
    }
    census
}

/// A line for each town with kept ground in year `k`: where its market place
/// is, its squares, its residents, and the houses they ask for and those
/// drawn (`on_cell`, by cell). A square counts to the town it trades in.
fn by_town(r: &Replay, k: usize, town: &world::TownGround, on_cell: &HashMap<u32, usize>) -> Vec<String> {
    // per town: squares, residents, houses wanted, houses drawn
    let mut sums: std::collections::BTreeMap<usize, (usize, f32, usize, usize)> = Default::default();
    for (&c, cell) in &town.cells {
        let t = (r.catch[k * r.n + c as usize] as usize).wrapping_sub(1);
        let e = sums.entry(t).or_default();
        e.0 += 1;
        e.1 += r.residents(k, c as usize);
        e.2 += cell.houses as usize;
        e.3 += on_cell.get(&c).copied().unwrap_or(0);
    }
    sums.into_iter()
        .map(|(t, (squares, residents, wanted, drawn))| {
            let (name, at, hill) = r.header.towns.get(t).map_or(("no town".to_string(), String::new(), false), |town| {
                (town.name.clone(), format!(" at {},{}", town.node % r.w, town.node / r.w), r.terrain[town.node] >= 2)
            });
            format!(
                "{name}{at}{}: {squares} squares, {residents:.0} residents, {wanted} houses wanted, {drawn} drawn ({:.1} people to a house)",
                if hill { " (hills)" } else { "" },
                residents / drawn.max(1) as f32
            )
        })
        .collect()
}

/// With ISOSTATE_TOWNS set, say how steady the houses are from one year to
/// the next: when two years in a row are laid out one after the other, how
/// many of the first year's houses are gone in the second, and how many of
/// those stood on cells that lost no houses (which a works taking their
/// ground, a new road or a widened market place may do; nothing else should).
fn steadiness(r: &Replay, k: usize, layout: &Layout) {
    type Plots = std::collections::HashMap<((i32, u8), i32), u32>;
    static LAST: std::sync::Mutex<Option<(usize, Plots, HashMap<u32, usize>)>> = std::sync::Mutex::new(None);
    if std::env::var_os("ISOSTATE_TOWNS").is_none() {
        return;
    }
    let now: Plots = layout.plots().map(|(run, index, cell)| ((run, index), cell)).collect();
    let houses: HashMap<u32, usize> = layout.stories.iter().map(|(&c, s)| (c, s.houses)).collect();
    let mut last = LAST.lock().unwrap();
    if let Some((_, before, had)) = last.as_ref().filter(|l| l.0 + 1 == k) {
        let gone: Vec<u32> = before.iter().filter(|(plot, _)| !now.contains_key(plot)).map(|(_, &c)| c).collect();
        let unasked = gone.iter().filter(|c| houses.get(c).copied().unwrap_or(0) >= had.get(c).copied().unwrap_or(0)).count();
        // why: no plot there any more, then the reasons of `Layout::why`
        let mut reasons = [0usize; 8];
        for (plot, _) in before.iter().filter(|(plot, _)| !now.contains_key(plot)) {
            reasons[layout.why.get(plot).copied().unwrap_or(0) as usize] += 1;
        }
        eprintln!(
            "towns {}: of {} houses the year before {} are gone, {} of them on cells that lost none; {} are new. Gone because: no plot there now {}, turn not come {}, cell has its houses {}, a yard or water {}, market place {}, a street {}, a house too near {}, its street not laid there {}",
            r.y0 + k as i32, before.len(), gone.len(), unasked, now.keys().filter(|p| !before.contains_key(p)).count(), reasons[0], reasons[1], reasons[2], reasons[3], reasons[4], reasons[5], reasons[6], reasons[7]
        );
    }
    *last = Some((k, now, houses));
}

/// A house on its own plot: a cottage, a villa, or on a road through the town
/// or its market place, once the cell has grown past it, a townhouse.
#[allow(clippy::too_many_arguments)]
fn one_house(into: &mut Into, r: &Replay, k: usize, lk: &Look, g: &Ground, height: &dyn Fn(f32, f32) -> f32, s: &Stood, story: &Story, great: bool, census: &mut Census) {
    let pl = &s.lot.pl;
    let seed = Seed::new(pl.u, pl.v);
    let upland = r.terrain[s.lot.cell as usize] >= 2;
    let detail = into.lod.at(pl.p.x, pl.p.y);
    let site = Site::on(ground(g, pl.p.x, pl.p.y), pl.yaw, height);
    let (joined, both) = (s.neighbours.0 || s.neighbours.1, s.neighbours.0 && s.neighbours.1);
    // villas: on the lanes and the ways out, on ground with little works on it, and never in a row
    let villa = pl.row.is_none() && s.lot.kind != 9 && !story.node && story.works < 0.15 && houses::villa_plot(s.year, seed);
    // rebuilt as a townhouse, the full width of its plot, once the cell has this many houses:
    // soon on a road through the town or round its market place, later on the old back
    // streets (those of the brick years are terraces from the first)
    let rebuilt = s.before + if s.lot.road { 7 + (27.0 * pl.v) as usize } else { 20 + (40.0 * pl.v) as usize };
    if villa {
        let v = Villa { plot_width: pl.width, plot_depth: pl.depth, roof: s.roof, era: Era::of(s.year), upland, detail, seed };
        houses::villa(into.houses, into.smoke, &mut into.trees[leaf_group(pl.v)], &site, &v, lk);
        census.villas += 1;
        census.storeys[1] += 1;
    } else if joined && story.houses >= rebuilt && (s.lot.road || s.year < TERRACE_ERA) {
        // a third storey on a road where the frontage is whole and the cell has grown as much
        // again; a fourth only round the market place of a great town
        let third = s.lot.road && both && story.houses >= rebuilt + 30 + (47.0 * pl.u) as usize;
        let fourth = third && great && story.houses >= rebuilt + 100;
        let storeys = 2 + third as u8 + fourth as u8;
        let t = Townhouse {
            plot_width: pl.width,
            plot_depth: pl.depth,
            storeys,
            roof: s.roof,
            era: Era::of(story.year_of(r, k, rebuilt, pl.u).max(s.year)),
            upland,
            shop: s.lot.road && houses::shop_at(((story.houses - s.before) as f32 / 60.0).min(1.0), seed),
            neighbours: s.neighbours,
            detail,
            seed,
        };
        houses::townhouse(into.houses, into.smoke, &site, &t, lk);
        census.townhouses += 1;
        census.storeys[(storeys as usize - 1).min(2)] += 1;
        census.tall_alone += (storeys >= 3 && !both) as usize;
    } else {
        // a cottage: one storey, and a second once the cell has grown well past it
        let storeys = 1 + (story.houses >= s.before + 30 + (73.0 * pl.v) as usize) as u8;
        let c = Cottage { plot_width: pl.width, plot_depth: pl.depth, storeys, roof: s.roof, era: Era::of(s.year), upland, neighbours: s.neighbours, detail, seed };
        houses::cottage(into.houses, into.smoke, &mut into.trees[leaf_group(pl.v)], &site, &c, lk);
        census.cottages += 1;
        census.storeys[storeys as usize - 1] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::made_up;
    use super::*;

    /// A house of a year's layout: its plot (the street and side, and the
    /// place along it), its cell, where it stands, and the year it went up.
    type House = (((i32, u8), i32), u32, [u32; 2], i32);

    /// The made-up town's houses in year `k`, in the order of their plots, and its land.
    fn year(r: &Replay, g: &Ground, k: usize) -> (Vec<House>, world::YearLand) {
        let cen = crate::network::centres(r);
        let land = world::year_land(r, g, k);
        let layout = lay_out(r, g, &cen, k, &[], &|_, _| false, &land);
        let mut houses: Vec<House> = layout.standing.iter().map(|s| ((s.lot.pl.run, s.lot.pl.index), s.lot.cell, [s.lot.pl.p.x.to_bits(), s.lot.pl.p.y.to_bits()], s.year)).collect();
        houses.sort();
        (houses, land)
    }

    /// The houses a cell's residents ask for at `PEOPLE_TO_A_HOUSE` each, over the town.
    fn asked(r: &Replay, k: usize) -> usize {
        (0..r.n).map(|c| (r.residents(k, c) / world::PEOPLE_TO_A_HOUSE).round() as usize).sum()
    }

    #[test]
    fn the_layout_is_the_same_however_it_is_come_by() {
        let r = made_up(true);
        assert!(r.kept());
        let g = world::build_ground(&r);
        let forward: Vec<Vec<House>> = (0..r.ny).map(|k| year(&r, &g, k).0).collect();
        assert!(forward[0].len() > 80, "{} houses in the first year", forward[0].len());
        // again on the same ground, which remembers the closes and their order
        for (k, was) in forward.iter().enumerate() {
            assert_eq!(&year(&r, &g, k).0, was, "year {k} laid out a second time");
        }
        // and on ground made anew, the last year first: a year's towns do not hang on the
        // years laid out before it
        let g = world::build_ground(&r);
        for k in (0..r.ny).rev() {
            assert_eq!(year(&r, &g, k).0, forward[k], "year {k} laid out on new ground, the years backwards");
        }
    }

    #[test]
    fn built_up_ground_and_houses_only_grow_while_the_people_do() {
        let r = made_up(true);
        let g = world::build_ground(&r);
        let mut before: Option<(Vec<House>, world::YearLand)> = None;
        // the people grow every year to the sixth
        for k in 0..6 {
            let now = year(&r, &g, k);
            if let Some(was) = &before {
                for (i, (&a, &b)) in was.1.town.built.iter().zip(&now.1.town.built).enumerate() {
                    assert_eq!(a & !b, 0, "year {k}: a close of parcel {i} was built up and is not");
                }
                for (i, (&a, &b)) in was.1.town.rank.iter().zip(&now.1.town.rank).enumerate() {
                    assert!(a == u16::MAX || a == b, "year {k}: close {i} has changed its place in its cell's order");
                }
                // every house stands where it stood, and is as old as it was
                for h in &was.0 {
                    assert!(now.0.binary_search(h).is_ok(), "year {k}: the house {h:?} is gone, or moved, or built again: {:?}", now.0.iter().find(|n| n.0 == h.0));
                }
                assert!(now.0.len() > was.0.len(), "year {k}: {} houses, {} the year before", now.0.len(), was.0.len());
                let built = |l: &world::YearLand| l.town.built.iter().map(|m| m.count_ones()).sum::<u32>();
                assert!(built(&now.1) > built(&was.1), "year {k}: no more ground built up");
            }
            before = Some(now);
        }
        // in the seventh a fifth of the people are gone: the houses outlast it, every one
        let (was, last) = (before.unwrap(), year(&r, &g, 6));
        assert_eq!(last.0, was.0);
        assert_eq!(last.1.town.built, was.1.town.built);
    }

    #[test]
    fn the_houses_drawn_follow_the_people() {
        let r = made_up(true);
        let g = world::build_ground(&r);
        for k in 0..6 {
            let (houses, land) = year(&r, &g, k);
            let (asked, people) = (asked(&r, k), (0..r.n).map(|c| r.residents(k, c)).sum::<f32>());
            assert_eq!(land.town.cells.values().map(|c| c.houses as usize).sum::<usize>(), asked);
            // on level dry ground with no works on it, nineteen in twenty of the houses asked
            // for find a plot, and the town is a twelfth out at the most
            assert!(houses.len() * 20 >= asked * 19, "year {k}: {} houses drawn for the {asked} that {people} people ask", houses.len());
            let to_a_house = people / houses.len() as f32;
            assert!((11.0..=13.0).contains(&to_a_house), "year {k}: {to_a_house} people to a drawn house");
            // cell by cell too: four in five at the least, and over what its people ask
            // for only by the end of a row, which goes up whole (six houses at the most)
            for (&c, cell) in &land.town.cells {
                let drawn = houses.iter().filter(|h| h.1 == c).count();
                assert!(drawn <= cell.houses as usize + 5 && drawn * 10 >= cell.houses as usize * 8, "year {k}, cell {c}: {drawn} houses drawn of {} asked", cell.houses);
            }
        }
    }

    #[test]
    fn frontages_join_before_they_rise_and_the_edge_is_cottages() {
        let r = made_up(true);
        let g = world::build_ground(&r);
        let cen = crate::network::centres(&r);
        let lk = Look::standard();
        let lod = Lod::all(houses::Detail::Play);
        let build = |k: usize| {
            let land = world::year_land(&r, &g, k);
            let (mut houses, mut smoke, mut trees) = (Builder::default(), Builder::default(), <[Greenery; LEAF_GROUPS]>::default());
            let census = houses_kept(&r, &g, &cen, k, &lk, &[], &|_, _| false, &land, &mut Into { houses: &mut houses, smoke: &mut smoke, trees: &mut trees, lod: &lod });
            (census, houses.pos.len(), crate::mesh_sum(&houses))
        };
        let (first, _, sum) = build(0);
        // the same year built twice is the same to the last vertex
        assert_eq!(build(0).2, sum);
        // a small town of 1750: cottages of one storey for the most part, nothing of three
        assert_eq!(first.drawn, first.cottages + first.townhouses + first.terraced + first.villas);
        assert!(first.storeys[0] * 2 > first.drawn, "{first:?}");
        assert_eq!((first.storeys[2], first.terraced, first.villas), (0, 0, 0), "{first:?}");
        // six years on it has three times the people: more houses, more of them joined into
        // townhouses, and no house of three storeys without a neighbour on both sides
        let (last, vertices, _) = build(5);
        assert!(last.drawn > first.drawn * 2 && last.townhouses > first.townhouses, "{first:?} then {last:?}");
        assert_eq!(last.tall_alone, 0, "{last:?}");
        assert!(last.storeys[2] > 0 && last.storeys[2] * 4 < last.drawn, "{last:?}");
        assert!(vertices > 0);
        // the newest square, a year in the town: all cottages of one storey
        assert!(last.storeys[0] >= 25, "{last:?}");
    }
}
