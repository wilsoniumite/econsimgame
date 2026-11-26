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
//! The rules, each with its reason where it is named:
//!
//! 1. Twelve people to a drawn house (`world::PEOPLE_TO_A_HOUSE`).
//! 2. A parcel of town ground is cut three times more, into eight closes
//!    (some eighty to a cell), and a close is built up or open.
//! 3. The order a cell's closes are built up in is fixed the year the cell
//!    joins (`world::town_order`): by the way to the market place along the
//!    roads of that year, ground off the road counting three times. A
//!    town's first cell is built up in rings round its market place
//!    (`world::OFF_THE_ROAD_AT_MARKET`).
//! 4. A cell has two counts of closes (`world::closes_built`): those painted
//!    as built-up ground, at `world::BUILT_GROUND` a house, and the wider
//!    count its houses' plots may stand on, at `world::HOUSE_GROUND` a house
//!    (`TownCell::built`, `TownCell::reach`). Between the two the houses
//!    stand among the closes of the town's edge. The works' share of the
//!    cell is not counted in: a works has a yard of its own where
//!    `scatter.rs` stands it, and the houses keep out of the yards as drawn.
//! 5. Houses outlast a fall of a quarter in their cell's residents
//!    (`world::HOUSES_KEEP`).
//! 6. The streets and plots are laid over all of a cell's town ground,
//!    built up or not. A plot's turn is its close's place in the cell's
//!    order plus up to five closes of its own (`world::EDGE_CLOSES`); it may
//!    be built on once its turn is under the count of closes the houses
//!    reach; and a row's turn is its last plot's. So the order never changes
//!    as the ground grows (`lay_out`).
//! 7. A lane behind a way stops where another way is nearer
//!    (`Laying::gives_way`), so the lanes of ways that meet make rings round
//!    a market place and do not run through one another's plots. The lanes
//!    of a town's old middle are built house by house, not a row at a time
//!    (`OLD_MIDDLE`).
//! 8. A house's year is the year the record first shows its cell with that
//!    many houses; ground older than the record is dated back by the rule
//!    the simulation fills it by (`Story::year_of`).
//! 9. Cottages of one storey, then two; townhouses on roads and market
//!    places only where a neighbour stands; a third storey only between two
//!    neighbours; terraces on rows from 1815; villas on lanes of ground with
//!    little works the year they went up (`what_stands`). What a house grows
//!    into is read from the most houses its cell has had (`Story::most`), so
//!    a house raised or rebuilt stays so when its cell loses people.
//! 10. For the houses to stay where they are built: the plots round a market
//!     place stand where it comes to at its widest (`MARKET_RING`); a plot
//!     fronts a street that runs whether or not the ground the street is on
//!     is yet town ground (`Laying::runs`); and a way's plots are the same
//!     while it is a lane and once it is a road, clear of the widest a road
//!     comes to be (`WIDEST_WAY`).
//! 11. The ground shader paints the open closes as paddocks, orchards,
//!     garden ground and greens within hedges (`town_edge` in terrain.wesl),
//!     and from across the island gives a town the tone of its roofs by how
//!     full each cell is (`town_roofs_far`).
//! 12. The one thing a town remembers is the works' yards of earlier years
//!     (`Yards`): ground a yard gave up takes its turn after all that was
//!     built by then, so no standing house is pulled down for it. The back
//!     of a plot is asked as its middle is, so that no garden runs into a
//!     yard, a churchyard, the railway or a lane (down to the water's edge it
//!     need only be dry).
//! 13. Each house is told its street (`family_of`, from the street's own
//!     number), the age a cottage was raised in and the age a townhouse was
//!     given a new front in; from these the generators make a street's
//!     family of walls and roofs and its habits (`houses::Street`). A house
//!     told none of them is the house it was before they were told.
//! 14. After 1900, where the record says how densely each square is built
//!     and what its land is worth, a lot first built is built as a form of
//!     its year, and a square that comes to hold more than its buildings do
//!     is rebuilt in part (`lots.rs`, where the rule is). A square's houses
//!     then take as many plots as its residents over its `dens` ask
//!     (`Replay::footprint`), and what a house built by 1900 grows into is
//!     read from the houses its cell had by 1900 (`Story::most_to`).
//!
//! `ISOSTATE_TOWNS=1` prints a year's count of houses against residents,
//! `=towns` the same town by town, `=cells` the cells that are short of
//! plots, and `=years` lays out every year in turn and says how many houses
//! of the year before are gone and why; `ISOSTATE_TOWNS_CELL=X,Y` says what
//! became of every plot of one cell.

use super::*;

pub(crate) mod estates;
pub(crate) mod lots;

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

/// The works' yards of the years before this one, so that the layout knows
/// when ground that a yard held came free. It is the one thing a town must
/// remember for its houses to stay: a works' yard is the only thing that
/// keeps plots out and then gives them back (a church, a station, the
/// railway and a market place only ever take ground). Without it, a plot
/// that a yard gave up took its old place in its cell's order, before
/// houses built since, and the last of those was pulled down for it though
/// nobody had left: a third of all the houses that went in a century and a
/// half went so. With it, such a plot comes into the order when the yard is
/// given up, after everything built up by then (`lay_out`).
///
/// Nothing is kept from year to year: the yards of a cell are worked out
/// from the record when a plot of the cell first asks, the same whichever
/// year is laid out and however it was come by. A yard is taken as
/// `scatter::build_year_inner` lays it: the works' footprint for its kind
/// and size that year, turned as it stands, and a little room.
pub(in crate::scatter) struct Yards<'a> {
    r: &'a Replay,
    slots: &'a Slots,
    k: usize,
    /// the firms of each cell
    firms: HashMap<u32, Vec<u32>>,
    /// for each cell asked about, its yards: where each stands, and the spans
    /// of years (first, last, half its width, half its depth as it lies on
    /// the map) it was each size
    known: std::cell::RefCell<HashMap<u32, std::rc::Rc<Vec<Yard>>>>,
}

type Yard = (f32, f32, Vec<(u32, u32, f32, f32)>);

impl<'a> Yards<'a> {
    pub(in crate::scatter) fn new(r: &'a Replay, slots: &'a Slots, k: usize) -> Yards<'a> {
        let mut firms: HashMap<u32, Vec<u32>> = HashMap::default();
        for (f, firm) in r.header.firms.iter().enumerate() {
            if firm.born <= r.y0 + k as i32 && f < slots.firm_pos.len() {
                firms.entry(firm.cell as u32).or_default().push(f as u32);
            }
        }
        Yards { r, slots, k, firms, known: Default::default() }
    }

    fn of_cell(&self, c: u32) -> std::rc::Rc<Vec<Yard>> {
        if let Some(known) = self.known.borrow().get(&c) {
            return known.clone();
        }
        let mut yards: Vec<Yard> = Vec::new();
        for &f in self.firms.get(&c).map_or(&[][..], |v| &v[..]) {
            let (x, z, yaw) = self.slots.firm_pos[f as usize];
            let (sin, cos) = yaw.sin_cos();
            let mut spans: Vec<(u32, u32, f32, f32)> = Vec::new();
            let first = (self.r.header.firms[f as usize].born - self.r.y0).max(0) as usize;
            for y in first..=self.k {
                let Some(w) = crate::scatter::firm_works(self.r, self.slots, f as usize, y) else { continue };
                let (fw, fd) = isostate_gen::works::footprint(w.kind, w.size);
                let (hx, hz) = ((fw * cos.abs() + fd * sin.abs()) * 0.5 + 0.02, (fw * sin.abs() + fd * cos.abs()) * 0.5 + 0.02);
                match spans.last_mut() {
                    Some(last) if last.1 + 1 == y as u32 && last.2 == hx && last.3 == hz => last.1 = y as u32,
                    _ => spans.push((y as u32, y as u32, hx, hz)),
                }
            }
            if !spans.is_empty() {
                yards.push((x, z, spans));
            }
        }
        let yards = std::rc::Rc::new(yards);
        self.known.borrow_mut().insert(c, yards.clone());
        yards
    }

    /// The first year from `from` on (counted from the start) in which no
    /// works' yard held a spot: next year, if one has held it from then to now.
    fn first_clear(&self, p: Vec2, from: u32) -> u32 {
        let c = ((p.y.floor().max(0.0) as usize).min(self.r.h - 1) * self.r.w + (p.x.floor().max(0.0) as usize).min(self.r.w - 1)) as u32;
        if !self.firms.contains_key(&c) {
            return from;
        }
        let yards = self.of_cell(c);
        let mut y = from;
        while let Some(span) = yards.iter().filter(|(x, z, _)| (x - p.x).abs() < 0.5 && (z - p.y).abs() < 0.5).flat_map(|(x, z, spans)| spans.iter().map(move |s| (x, z, s))).find(|(x, z, s)| s.0 <= y && y <= s.1 && (*x - p.x).abs() < s.2 && (*z - p.y).abs() < s.3) {
            y = span.2 .1 + 1;
        }
        y
    }

    /// The year (counted from the start) from which a spot has been clear of
    /// the works' yards: the year after the last in which one held it, or 0
    /// if none ever did; next year, if one holds it now.
    fn clear_since(&self, p: Vec2) -> u32 {
        let c = ((p.y.floor().max(0.0) as usize).min(self.r.h - 1) * self.r.w + (p.x.floor().max(0.0) as usize).min(self.r.w - 1)) as u32;
        if !self.firms.contains_key(&c) {
            return 0;
        }
        let mut since = 0;
        for (x, z, spans) in self.of_cell(c).iter() {
            let (dx, dz) = ((x - p.x).abs(), (z - p.y).abs());
            if let Some(span) = spans.iter().rev().find(|s| dx < s.2 && dz < s.3) {
                since = since.max(span.1 + 1);
            }
        }
        since
    }
}

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
    /// after 1900 (`lots.rs`): groups of plots built as forms of their year,
    /// and of them those that stand where houses of 1900 or before stood
    pub forms: usize,
    pub rebuilt: usize,
    /// filled squares, and the closes kept as their parks
    pub filled: usize,
    pub park: usize,
    /// closes of filled squares built over by themselves (`lots::infill`)
    pub infill: usize,
    /// squares laid out as estates, with streets of their own, and the plots
    /// (and whole blocks) standing on them (`estates.rs`)
    pub estates: usize,
    pub estate_lots: usize,
    /// of the closes built over by themselves, those on estates
    pub infill_estates: usize,
}

/// A street that may be laid on kept town ground.
struct Way {
    a: Vec2,
    b: Vec2,
    width: f32,
    id: i32,
    /// a road, which is drawn already
    road: bool,
    /// 0 the way between two cells, 1 to 3 the lanes behind it, 4 a cross
    /// street, 5 a station road; on an estate (`estates.rs`) 6 a street along
    /// the way it is laid from, 7 one out from it (a cross street, a link road,
    /// a cul-de-sac or its stub). (A plot of kind 8 is an estate's whole block.)
    kind: u8,
    /// for a lane or a cross street, the way it belongs to
    main: i32,
    /// the year (counted from the start) its way was first there: the first
    /// year its road ran, or the year the cell it leads out to joined its town;
    /// an estate's street, the year its estate was laid out
    since: u32,
    /// an estate's street: its estate and the way it is laid from
    estate: Option<estates::Laid>,
}

/// Where a street came from (`docs/designs/ways.md`, part 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Source {
    /// a road through, older than the town's ground
    Road,
    /// a lane: the way to the square a square grew from, or the lanes and
    /// cross streets behind a way, which stand for the lanes and hedge lines of
    /// the closes the town was built over
    Lane,
    /// laid out by one owner at one time: an estate's streets, a station road
    Owner,
}

impl Source {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Source::Road => "road through",
            Source::Lane => "field lane",
            Source::Owner => "laid out by one owner",
        }
    }
}

impl Way {
    fn source(&self) -> Source {
        match self.kind {
            0 if self.road => Source::Road,
            0..=4 => Source::Lane,
            _ => Source::Owner,
        }
    }
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
    /// a town's first cell, with its market place
    node: bool,
    /// for each close of the cell's order, how many houses the cell must
    /// have for its houses to reach the close (`world::closes_built`)
    reached_at: Vec<u16>,
}

impl Story {
    fn of(r: &Replay, k: usize, c: usize, order: &[(u32, u8, f32)]) -> Story {
        let dry = order.iter().take_while(|o| o.2 > 0.0).count();
        let mut reached_at: Vec<u16> = Vec::with_capacity(dry);
        let mut n = 0;
        while reached_at.len() < dry && n < u16::MAX {
            n += 1;
            reached_at.resize(world::closes_built(order, n as usize).1.max(reached_at.len()), n);
        }
        let mut since = k;
        while since > 0 && r.built[(since - 1) * r.n + c] > 0 {
            since -= 1;
        }
        let mut steps: Vec<(usize, usize)> = Vec::new();
        for y in since..=k {
            let n = (r.footprint(y, c) / world::PEOPLE_TO_A_HOUSE).round() as usize;
            if steps.last().map_or(true, |s| n > s.0) {
                steps.push((n, y));
            }
        }
        Story {
            houses: world::town_houses(r, k, c),
            steps,
            since,
            joined: r.joined(k, c).unwrap_or(r.y0),
            node: r.header.towns.iter().any(|t| t.node == c),
            reached_at,
        }
    }

    /// The most houses the cell has had since it joined. What a house grows
    /// into is read from this and not from the houses of the year, so that a
    /// house raised a storey or rebuilt as a townhouse stays so when its
    /// cell loses people.
    fn most(&self) -> usize {
        self.steps.last().map_or(self.houses, |s| s.0.max(self.houses))
    }

    /// The most houses the cell had by year `last` (counted from the start),
    /// in a layout of year `k`: after 1900 a house grows by the forms'
    /// rule (`lots.rs`), and by its own only as far as its cell had grown by
    /// 1900.
    fn most_to(&self, k: usize, last: usize) -> usize {
        if last >= k {
            return self.most();
        }
        self.steps.iter().filter(|s| s.1 <= last).map(|s| s.0).max().unwrap_or(0)
    }

    /// How many houses the cell had when the turn of a plot came: when its
    /// houses first reached the close of that turn. It is the plot's own
    /// number, whatever becomes of the plots about it, so the year a house
    /// went up, its style and what it grows into never change. (A house's
    /// place among the houses standing on its cell does: a yard given up or
    /// a road laid frees or takes plots whose turn comes earlier, and with
    /// the year read from that place some two houses in a hundred counted as
    /// built in another year every year, and a row of cottages could turn
    /// into a brick terrace, or a terrace back into cottages.)
    fn houses_at(&self, turn: f32) -> usize {
        self.reached_at.get(turn.max(0.0) as usize).or(self.reached_at.last()).map_or(1, |&n| n as usize)
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
    /// the two numbers its street's houses share (`family_of`): what the houses are told of their street
    family: Seed,
    /// a plot of an estate (`estates.rs`): built the year its estate was laid out
    estate: Option<EstateLot>,
    /// it stands on an estate's ground, an estate's own plot or not: a way's
    /// frontage there is never rebuilt as a whole block back to the lane
    /// behind (`lots::block_ground`), for the estate's own streets run there
    on_estate: bool,
}

/// What a plot of an estate knows of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EstateLot {
    /// the calendar year its estate was laid out, in which it is built
    pub year: i32,
    /// a whole block between four streets (`estates::BlockGround`), not a plot along one
    pub block: bool,
    /// its street's pattern
    pub pattern: estates::Pattern,
}

/// The two numbers the houses of a street share, from the street's own
/// number (a market place has one too). They are the street's for good: a
/// street's number is that of the link between two cells it runs along, so
/// it does not change with the year, with the road's level or with what is
/// built on it. The houses' generators make of them a family of walls and
/// roofs and the street's habits of building (`houses::Street`).
fn family_of(street: i32) -> Seed {
    Seed::new(world::hash2(street, 9, 31), world::hash2(street, 10, 31))
}

/// How near an older street a later lane may be laid: clear of the older
/// street's plots.
const OLDER_STREET: f32 = 0.055;

/// The widest a road comes to be (`ways`, a road of the third level).
const WIDEST_WAY: f32 = 0.034;

/// How far from the middle of a market place the houses round it stand: clear
/// of the widest a market place comes to be (`squares`).
const MARKET_RING: f32 = 0.09 + 0.03;

/// How many plots stand round a market place: one to every 0.05 of the ring.
const RING_PLOTS: i32 = (std::f32::consts::TAU * MARKET_RING / 0.05) as i32;

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
    /// how many houses its cell had when its plot's turn came
    /// (`Story::houses_at`, less this one), the calendar year that was, and
    /// its roof
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
/// plots that were there before it (`lay_out`). Then the streets of the
/// year's estates (`estates.rs`), laid from the ways, and the ground of their
/// whole blocks.
#[allow(clippy::too_many_arguments)]
fn ways(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, town: &world::TownGround, sq: &[Square], stations: &[Option<Vec2>], estates: &HashMap<u32, estates::Estate>) -> (Vec<Way>, Vec<estates::BlockGround>) {
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
        out.push(Way { a, b, width: if lv == 3 { 0.034 } else if lv == 2 { 0.022 } else { 0.016 }, id: sid, road: lv >= 2, kind: 0, main: sid, since, estate: None });
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
                out.push(Way { a: a + u * (short_a + jit.max(-0.03)) + o, b: b - u * (short_b - jit.min(0.03)) + o, width: 0.014, id, road: false, kind: 1 + ring as u8, main: sid, since, estate: None });
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
                out.push(Way { a: base + w * from, b: base + w * reach, width: 0.012, id, road: false, kind: 4, main: sid, since, estate: None });
            }
        }
    }
    for (s, st) in sq.iter().zip(stations) {
        let node = r.header.towns[s.town].node;
        if let Some(st) = *st {
            let d = (st - s.at).normalize_or_zero();
            // from the year the railway came
            let since = (0..=k).find(|&y| r.trans[y * r.n + node] & 4 != 0).unwrap_or(k) as u32;
            out.push(Way { a: s.at + d * s.radius, b: st - d * 0.03, width: 0.02, id: -1 - s.town as i32, road: false, kind: 5, main: -1 - s.town as i32, since, estate: None });
        }
    }
    let mains: Vec<(Vec2, Vec2, i32, u32)> = out.iter().filter(|w| w.kind == 0).map(|w| (w.a, w.b, w.id, w.since)).collect();
    let (laid, blocks) = estates::streets(r, estates, &mains);
    out.extend(laid);
    out.sort_by_key(|w| (w.since, w.main, w.id));
    (out, blocks)
}

/// The year (counted from the start) cell `c` joined its town: 0 for ground older than the record.
fn joined_in(r: &Replay, k: usize, c: usize) -> u32 {
    r.joined(k, c).map_or(k as u32, |y| ((y - r.y0).max(0) as u32).min(k as u32))
}

//// What was laid out for a year: the market places, the lanes to draw, the
/// houses that stand and the story of each cell that has any.
pub(super) struct Layout {
    /// the lengths of lane to draw, each a line of points and a width
    lanes: Vec<(Vec<Vec2>, f32)>,
    /// for each of them, the street it is a length of (`LaneOf`)
    lane_of: Vec<LaneOf>,
    /// the lengths of the roads through that run over a town's built-up
    /// ground straight, once tarred, to draw again as its main streets
    roads: Vec<(Vec<Vec2>, f32, LaneOf)>,
    /// the year's estates, by their cells (`estates.rs`)
    estates: HashMap<u32, estates::Estate>,
    /// the length of each road through town ground laid this year on each
    /// cell, by the year it was laid: for `ISOSTATE_LOTS`
    laid: Vec<(u32, Source, i32, f32)>,
    /// the houses, street by street and in order along each
    standing: Vec<Stood>,
    /// what stands on the plot of each of them (`what_stands`)
    whats: Vec<What>,
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

/// What a length of lane to draw is a length of: where its street came
/// from and the year it was laid (for `ISOSTATE_LOTS`), its estate's pattern
/// and year (if it is an estate's street), and whether it is a cul-de-sac's
/// turning circle.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LaneOf {
    pub source: Source,
    pub year: i32,
    pub estate: Option<(estates::Pattern, i32)>,
    pub circle: bool,
    /// its rank (`estates::Rank`), and whether it lies in its town's middle
    /// (`lots::in_middle`): what its surface follows
    pub rank: estates::Rank,
    pub middle: bool,
}

/// A street that may be laid, and what has been asked of it so far.
struct Street {
    way: Way,
    len: f32,
    /// it is walked in this many steps of about 60 m
    steps: usize,
    /// at each step: 0 not asked yet, 1 not laid there, 2 laid, 3 laid on
    /// built-up ground, 4 not laid only because the ground is not town
    /// ground, 5 not laid only because a works' yard lies across it, 6 (a
    /// lane behind a way) not laid only because the ground is an estate's:
    /// it runs there for the plots of ground built by 1900 that front it,
    /// which stand as they stood, and for nothing else
    state: Vec<std::cell::Cell<u8>>,
    /// at each step, the first year since its way came in which no works'
    /// yard lay across it (`Yards::first_clear`), once asked; `u32::MAX` until then
    clear: Vec<std::cell::Cell<u32>>,
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
    /// the works' yards of this year and of those before, where the run has works that are drawn
    yards: Option<&'a Yards<'a>>,
    /// the year, counted from the start
    k: u32,
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
    /// the ways that come within `MAINS_REACH` of each cell of the map
    mains_by_cell: HashMap<(i32, i32), Vec<u32>>,
    /// the year's estates (`estates.rs`), by their cells
    estates: &'a HashMap<u32, estates::Estate>,
    /// 1900 (`lots::FROM`), counted from the start: a plot built by then is of
    /// ground the estates leave alone
    old: u32,
}

/// How near equally far from two ways a back street between their ground is
/// laid (`Laying::on_seam`): it runs on while the line where they meet bends
/// no more than this off its own.
const SEAM: f32 = 0.06;

/// How far from a cell's edge the ways are looked up by it
/// (`Laying::mains_by_cell`): far enough to find the nearest way to every
/// spot of an estate (`estates.rs`). (A lane, never further than `FAR_LANE`
/// and a little from its own way, finds the same nearer ways as it did with
/// that as the reach.)
const MAINS_REACH: f32 = 0.8;

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

    /// The year from which a spot has been clear of the works' yards.
    fn clear_since(&self, p: Vec2) -> u32 {
        self.yards.map_or(0, |y| y.clear_since(p))
    }

    /// The year street `i` first ran at one of its steps, as far as the
    /// works' yards go: the first year since its way came in which none lay
    /// across it there.
    fn step_first_ran(&self, i: usize, step: usize) -> u32 {
        let s = &self.streets[i];
        match s.clear[step].get() {
            u32::MAX => {
                let first = self.yards.map_or(s.way.since, |y| y.first_clear(s.at(step), s.way.since));
                s.clear[step].set(first);
                first
            }
            first => first,
        }
    }

    /// The year from which street `i` has been clear of the works' yards at
    /// share `t` of its length: at the steps on either side.
    fn clear_since_at(&self, i: usize, t: f32) -> u32 {
        let s = &self.streets[i];
        let x = t.clamp(0.0, 1.0) * s.steps as f32;
        self.clear_since(s.at(x.floor() as usize)).max(self.clear_since(s.at(x.ceil() as usize)))
    }

    /// Whether a lane behind a way, or a cross street off it, has come at `p`
    /// to ground that lies nearer another way: there it stops, and the other
    /// way's lanes take the ground. So where ways meet, at a market place
    /// above all, the lanes behind them make rings of streets round it and do
    /// not run across one another. (When every way
    /// laid its lanes the whole of its length, those of ways that met crossed
    /// in the angle between them, and of the plots there whose turn came six
    /// in ten were lost to a street that ran through them: the middle of a
    /// town was the thinnest part of it.) A way that came later takes no
    /// ground from the lanes of one that was there before it, so a new road
    /// moves no house.
    fn gives_way(&self, s: &Street, p: Vec2) -> bool {
        self.nearer_way(s.way.main, s.way.since, p)
    }

    /// Whether a spot lies on the line where the ground of ways `a` and `b`
    /// (there by year `since`) meets: the two are within `SEAM` of equally
    /// far, and no other way that stood then is nearer than both.
    fn on_seam(&self, a: i32, b: i32, since: u32, p: Vec2) -> bool {
        let (Some(&ia), Some(&ib)) = (self.main_at.get(&a), self.main_at.get(&b)) else { return false };
        let off = |i: u32| {
            let (a, b, _, _) = self.mains[i as usize];
            off_line(p, a, b)
        };
        let (da, db) = (off(ia), off(ib));
        if (da - db).abs() > SEAM {
            return false;
        }
        let both = da.min(db);
        !self.mains_by_cell.get(&(p.x.floor() as i32, p.y.floor() as i32)).is_some_and(|near| near.iter().any(|&j| j != ia && j != ib && self.mains[j as usize].3 <= since && off(j) < both - 1e-3))
    }

    /// Whether a way other than way `main`, there by year `since`, is nearer a spot.
    fn nearer_way(&self, main: i32, since: u32, p: Vec2) -> bool {
        let Some(&own) = self.main_at.get(&main) else { return false };
        let (a, b, _, _) = self.mains[own as usize];
        let off = off_line(p, a, b);
        self.mains_by_cell.get(&(p.x.floor() as i32, p.y.floor() as i32)).is_some_and(|near| {
            near.iter().any(|&j| {
                let (a, b, _, at) = self.mains[j as usize];
                j != own && at <= since && off_line(p, a, b) < off - 1e-3
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
        matches!(self.asked(i, step), 2..=4 | 6)
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
        if let Some(e) = s.way.estate {
            let state = self.estate_step(s, &e, p, at);
            s.state[step].set(state);
            return state;
        }
        let dry = at.is_some() || world::height_at(self.g, p.x, p.y) >= 0.05;
        // what never changes: the ground is dry, and the lane does not give way to another
        let lies = dry && (s.way.road || !(s.lane() && (self.gives_way(s, p) || at.is_some_and(|a| s.way.since > self.joined[&a.0]) && self.in_street(p, OLDER_STREET, None, |o| o.way.since < s.way.since))));
        // what may: a works' yard, a churchyard, the railway, a market place or a station across it
        let held = lies && !s.way.road && ((self.near_plot)(p.x, p.y) || self.in_square(p));
        let state = match (lies, held, at) {
            (false, ..) => 1,
            // (5: only a works' yard keeps it out, and a yard may be given up)
            (_, true, _) => if self.clear_since(p) > self.k { 5 } else { 1 },
            (_, _, None) => 4,
            // (6: a lane behind a way on an estate's ground, which its own streets take)
            (_, _, Some(a)) if s.lane() && self.estates.contains_key(&a.0) => 6,
            (_, _, Some(a)) => 2 + a.2 as u8,
        };
        s.state[step].set(state);
        state
    }

    /// What an estate's street is at a spot (`Street::state`): laid (3) only
    /// on its own square's built-up ground (not its park), dry and clear of
    /// the water's edge, on ground nearer the way it is laid from than any
    /// other way that stood in its year, and clear of the works' yards, the
    /// churchyards, the railway and the market places (5 where only a yard
    /// keeps it out); else not laid (1).
    fn estate_step(&self, s: &Street, e: &estates::Laid, p: Vec2, at: Option<(u32, u16, bool)>) -> u8 {
        let Some((cell, _, built)) = at else { return 1 };
        if cell != e.cell || !built || world::water_dist(self.g, p.x, p.y) < 0.02 {
            return 1;
        }
        // a back street between the ground of two ways, where the two are near enough equally
        // far and no other way is nearer than either; any other street where its own way is nearest
        let nearest = match e.between {
            Some(other) => self.on_seam(s.way.main, other, s.way.since, p),
            None => !self.gives_way(s, p),
        };
        if !nearest {
            return 1;
        }
        if (self.near_plot)(p.x, p.y) || self.in_square(p) {
            return if self.clear_since(p) > self.k { 5 } else { 1 };
        }
        3
    }

    /// Whether a spot is within `clear` of the edge of a laid street, among
    /// those that `counts` passes. With `age`, the year a plot was first
    /// there to build on, a length of lane counts if it first ran by then,
    /// whether or not a works' yard lies across it this year: a lane first
    /// laid later, when a yard was given up, was laid round what stood; and
    /// a lane that a yard has since closed still keeps out the plots it
    /// always kept out, or they would be built on the day the yard came and
    /// the last houses of the cell pulled down for them.
    fn in_street(&self, p: Vec2, clear: f32, age: Option<u32>, counts: impl Fn(&Street) -> bool) -> bool {
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
            let step = (t * s.steps as f32).round() as usize;
            p.distance(s.way.a + along * t) < half + clear
                && match age {
                    None => self.runs(j as usize, step),
                    // (a lane an estate took counts only for a plot of ground built by 1900)
                    Some(age) => {
                        let state = self.asked(j as usize, step);
                        state >= 2 && (state != 6 || age <= self.old) && (!s.lane() || self.step_first_ran(j as usize, step) <= age)
                    }
                }
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
#[allow(clippy::too_many_arguments)]
pub(super) fn lay_out(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, parishes: &[Parish], near_plot: &dyn Fn(f32, f32) -> bool, yards: Option<&Yards>, land: &world::YearLand) -> Layout {
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
    // the squares laid out as estates, with streets of their own (`estates.rs`)
    let estates = estates::of_year(r, k, town);
    let mut lay = Laying {
        g,
        town,
        near_plot,
        yards,
        k: k as u32,
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
        estates: &estates,
        old: (lots::FROM - r.y0).max(0) as u32,
    };
    let (ways, block_grounds) = ways(r, g, cen, k, town, &sq, &stations, &estates);
    for way in ways {
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
            // every cell of the map the way comes within its furthest lane of, or the furthest
            // spot of an estate laid from it
            let reach = MAINS_REACH;
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
        lay.streets.push(Street { way, len, steps, state: vec![std::cell::Cell::new(0); steps + 1], clear: vec![std::cell::Cell::new(u32::MAX); steps + 1] });
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
    let mut side_lots: Vec<(Lot, f32, u32)> = Vec::new();
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
        // whatever the road's width: were a lane's plots wider, set further back or
        // later in their turns, the year the road came, or was widened, the whole of
        // its frontage would be laid out again and its houses move or go)
        let spacing = match s.kind {
            _ if s.estate.is_some() => s.estate.map_or(0.05, |e| e.pattern.plot()),
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
        // most streets keep their houses' second number near one of their own. (It was the
        // street's family of colours before the houses were told of their street, which now
        // does that better, `family_of`. It is kept because the second number also settles
        // how wide a cottage is and how soon a house is raised or rebuilt as a townhouse: so
        // a street's houses are of a size and rise together, and every house is the size, and
        // is raised in the year, it was before.)
        let family = (world::hash2(s.id, 7, 31) < 0.7).then(|| 0.16 + 0.68 * world::hash2(s.id, 8, 31));
        let told = family_of(s.id);
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
                let (pu, pv) = (world::hash2(id, i, 24), world::hash2(id, i, 25));
                // an estate's plot: on its own square's built-up ground, its turn its close's
                // place, in the row its estate's pattern makes; built the year the estate was
                // laid out, whatever the count of its square's houses
                if let Some(e) = s.estate {
                    if c != e.cell || rank as f32 >= built_of(c) {
                        tell((id, j), i, 1);
                        note(c, s.kind, 1);
                        continue;
                    }
                    let freed = lay.clear_since(p).max(lay.clear_since(p + n * side * (depth * 0.5))).max(lay.clear_since_at(si, t_of(i)));
                    let v = family.map_or(pv, |f| (f + 0.3 * (pv - 0.5)).clamp(0.0, 0.999));
                    let pl = Plot { p, yaw: yaw_along + if side > 0.0 { std::f32::consts::PI } else { 0.0 }, run: (id, j), row: Some(e.row(s.kind, p, i)), high: false, index: i, width: spacing, depth, need: 0.0, u: pu, v };
                    let lot = Lot { pl, cell: c, turn: rank as f32 + 0.5 * pu, kind: s.kind, road: false, age: s.since.max(freed), street: (0, 0.0), family: told, estate: Some(EstateLot { year: e.year, block: false, pattern: e.pattern }), on_estate: true };
                    side_lots.push((lot, t_of(i), freed));
                    continue;
                }
                // the lanes behind a way are not laid on an estate's ground, which its own streets take
                if st.lane() && lay.estates.contains_key(&c) {
                    tell((id, j), i, 7);
                    note(c, s.kind, 7);
                    continue;
                }
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
                // (the year from which a works' yard has held neither the plot nor its length of
                // lane: the plot at its middle and at its back, the two places a house is kept out by)
                let freed = lay.clear_since(p).max(lay.clear_since(p + n * side * (depth * 0.5))).max(lay.clear_since_at(si, t_of(i)));
                let v = family.map_or(pv, |f| (f + 0.3 * (pv - 0.5)).clamp(0.0, 0.999));
                let pl = Plot { p, yaw: yaw_along + if side > 0.0 { std::f32::consts::PI } else { 0.0 }, run: (id, j), row: in_row.then_some(row), high: s.kind == 0, index: i, width: spacing, depth, need: 0.0, u: pu, v };
                side_lots.push((Lot { pl, cell: c, turn, kind: s.kind, road: s.road, age: s.since.max(joined).max(freed), street: (0, 0.0), family: told, estate: None, on_estate: lay.estates.contains_key(&c) }, t_of(i), freed));
            }
            // a row goes up together: its turn is that of the last of its plots in the cell
            // (a row that runs over a cell's edge is two, one to each cell)
            for a in 0..side_lots.len() {
                if let Some(ri) = side_lots[a].0.pl.row {
                    let cell = side_lots[a].0.cell;
                    // (a row is six plots at most, and its plots follow one another)
                    let near = &side_lots[a.saturating_sub(6)..(a + 7).min(side_lots.len())];
                    let last = near.iter().filter(|(l, ..)| l.pl.row == Some(ri) && l.cell == cell).map(|(l, ..)| l.turn).fold(f32::MIN, f32::max);
                    side_lots[a].0.turn = side_lots[a].0.turn.max(last);
                }
            }
            // ground that a works' yard held, or a plot whose length of lane a yard lay
            // across: it comes into the order the year the yard is given up, after all the
            // ground built up by then, so that no house standing then is pulled down to make
            // way for a plot that came free behind it (`Yards`). It comes in by itself: the
            // rest of its row keep their turn, and the houses that stand on them.
            // (an estate's plots go up with their estate, whatever came free when)
            for (lot, _, freed) in side_lots.iter_mut().filter(|l| l.0.estate.is_none()) {
                let (c, freed) = (lot.cell, *freed);
                // (a yard that holds it this year keeps it out this year, as it always did)
                if freed > lay.joined[&c] && freed <= k as u32 {
                    let was = *built_before.entry((c, freed)).or_insert_with(|| {
                        town.cells.get(&c).map_or(0.0, |cell| world::closes_built(&cell.order, world::town_houses(r, freed as usize - 1, c as usize)).1 as f32)
                    });
                    if lot.turn < was {
                        lot.turn = was + lot.turn * 1e-3;
                    }
                }
            }
            for (lot, t, _) in side_lots.drain(..) {
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
        let count = RING_PLOTS;
        for i in 0..count {
            let a = i as f32 / count as f32 * std::f32::consts::TAU;
            let dirv = Vec2::new(a.cos(), a.sin());
            let p = s.at + dirv * MARKET_RING;
            let id = -1000 - s.town as i32;
            let pu = world::hash2(id, i, 26);
            let Some((c, _, _)) = lay.town_at(p) else { continue };
            // the market place is the first of a town to be built round (but for ground a
            // works' yard held: that comes into the order when the yard is given up)
            let (joined, freed) = (lay.joined[&c], lay.clear_since(p).max(lay.clear_since(p + dirv * (houses::PLOT.1 * 0.5))));
            let mut turn = -2.0 + pu;
            if freed > joined && freed <= k as u32 {
                let was = *built_before.entry((c, freed)).or_insert_with(|| {
                    town.cells.get(&c).map_or(0.0, |cell| world::closes_built(&cell.order, world::town_houses(r, freed as usize - 1, c as usize)).1 as f32)
                });
                turn = was + pu * 1e-3;
            }
            cands.push(Lot {
                pl: Plot { p, yaw: (dirv.y).atan2(-dirv.x) + std::f32::consts::FRAC_PI_2, run: (id, 0), row: None, high: true, index: i, width: 0.05, depth: houses::PLOT.1, need: 0.0, u: pu, v: world::hash2(id, i, 27) },
                cell: c,
                turn,
                kind: 9,
                road: true,
                age: joined.max(freed),
                street: (0, 0.0),
                family: family_of(id),
                estate: None,
                on_estate: lay.estates.contains_key(&c),
            });
        }
    }
    // the whole blocks of the estates on blocks: one plot each, the first of its square to go
    // up, where all of the block's ground is its square's built-up ground, nearer its way than
    // any other, dry and clear of what keeps a house out
    for b in &block_grounds {
        let side = Vec2::new(b.front.y, -b.front.x);
        let fits = [(-0.5f32, -0.5f32), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5), (0.0, 0.0), (0.0, 0.5), (0.0, -0.5)].iter().all(|&(x, y)| {
            let q = b.at + side * (x * b.width) + b.front * (y * b.depth);
            matches!(lay.town_at(q), Some((c, rank, true)) if c == b.cell && (rank as f32) < built_of(c))
                && world::water_dist(g, q.x, q.y) >= 0.03
                && !near_plot(q.x, q.y)
                && !lay.in_square(q)
                && !lay.nearer_way(b.main, b.since, q)
        });
        if !fits {
            continue;
        }
        let pl = Plot { p: b.at, yaw: b.front.x.atan2(b.front.y), run: (b.id, 0), row: Some(0), high: false, index: 0, width: b.width, depth: b.depth, need: 0.0, u: world::hash2(b.id, 0, 24), v: world::hash2(b.id, 0, 25) };
        cands.push(Lot { pl, cell: b.cell, turn: -10.0, kind: 8, road: false, age: b.since, street: (0, 0.0), family: family_of(b.id), estate: Some(EstateLot { year: b.year, block: true, pattern: estates::Pattern::Blocks }), on_estate: true });
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
    // the ground of the estates' whole blocks that stand
    let mut placed: Vec<lots::Footprint> = Vec::new();
    for (_, _, _, i) in order {
        let Some(lot) = cands[i as usize].take() else { continue };
        let p = lot.pl.p;
        // has its cell houses left to build? (asked first: it is the cheapest to answer)
        let (story, built) = stories.entry(lot.cell).or_insert_with(|| (Story::of(r, k, lot.cell as usize, town.cells.get(&lot.cell).map_or(&[][..], |cell| &cell.order[..])), 0));
        // (an estate's plots go up with their estate, whatever its square's count of houses)
        let room = *built < story.houses || lot.estate.is_some();
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
        if lot.kind == 8 {
            // a whole block: its ground was asked when it was found
            let front = Vec2::new(lot.pl.yaw.sin(), lot.pl.yaw.cos());
            placed.push(lots::Footprint { at: p, along: Vec2::new(front.y, -front.x), front, half: (lot.pl.width * 0.5, lot.pl.depth * 0.5) });
            taken.entry(bucket(p)).or_default().push(p);
            let year = lot.estate.map_or(r.y0 + k as i32, |e| e.year);
            let roof = roof_in(r, k, lot.cell as usize, year, Seed::new(lot.pl.u, lot.pl.v));
            note(lot.cell, lot.kind, 0);
            standing.push(Stood { lot, before: story.houses_at(-10.0) - 1, year, roof, neighbours: (false, false) });
            continue;
        }
        if lot.kind != 9 && (!lay.runs_at(lot.street.0 as usize, lot.street.1) || lay.town_at(p).is_none()) {
            tell(lot.pl.run, lot.pl.index, 7);
            note(lot.cell, lot.kind, 7);
            continue;
        }
        // (its yard or garden runs back from the house to `back`, as far again as the plot
        // is deep. What keeps a house out (`near_plot`) keeps it some way off what it guards:
        // 0.02 off a works' yard, 0.03 off a churchyard, 0.016 off the railway's banks. So
        // the back of the plot (`edge`) is asked as its middle is, and the garden behind it
        // then ends clear of the yard, the churchyard and the banks as they are drawn. Asking
        // the end of the garden itself, with that room again, emptied the old middles about
        // the works and the churches, where a garden may well end at a yard's wall. A garden
        // may come down to the water's edge, where a house may not stand: there the end of
        // it need only be dry.)
        let dir = Vec2::new(lot.pl.yaw.sin(), lot.pl.yaw.cos());
        let (back, edge) = (p - dir * (lot.pl.depth * 0.5 + houses::BACK_ROOM), p - dir * (lot.pl.depth * 0.5));
        let behind = if world::water_dist(g, edge.x, edge.y) < 0.03 { world::height_at(g, back.x, back.y) < 0.05 || lay.clear_since(edge) > k as u32 } else { near_plot(edge.x, edge.y) };
        if near_plot(p.x, p.y) || behind {
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
        if lay.in_street(p, if lot.kind == 9 { 0.03 } else { 0.02 }, Some(lot.age), |o| !o.lane() || o.way.since <= lot.age) || lay.in_street(back, 0.0, Some(lot.age), |o| !o.lane() || o.way.since <= lot.age) {
            tell(lot.pl.run, lot.pl.index, 5);
            note(lot.cell, lot.kind, 5);
            continue;
        }
        let c = bucket(p);
        // (on an estate's whole block, or too near a house)
        if placed.iter().any(|f| f.holds(p) || f.holds(back)) || (-1..=1).any(|dy| (-1..=1).any(|dx| taken.get(&(c.0 + dx, c.1 + dy)).is_some_and(|v| v.iter().any(|q| q.distance(p) < 0.042)))) {
            tell(lot.pl.run, lot.pl.index, 6);
            note(lot.cell, lot.kind, 6);
            continue;
        }
        if let Some(key) = row {
            row_in.insert(key, true);
        }
        taken.entry(c).or_default().push(p);
        let seed = Seed::new(lot.pl.u, lot.pl.v);
        // (an estate's houses are not among those its square's people ask for)
        *built += lot.estate.is_none() as usize;
        // the houses its cell had when its turn came, and the year that was: a row's
        // plots share a turn, and so the year; an estate's, its estate's year
        let before = story.houses_at(lot.turn) - 1;
        let own = lot.pl.row.map_or(lot.pl.u, |ri| world::hash2(lot.pl.run.0, ri, 29));
        let year = lot.estate.map_or_else(|| story.year_of(r, k, before + 1, own), |e| e.year);
        let roof = roof_in(r, k, lot.cell as usize, year, seed);
        note(lot.cell, lot.kind, 0);
        standing.push(Stood { lot, before, year, roof, neighbours: (false, false) });
    }
    find_neighbours(&mut standing, (r.builds.kept() && r.y0 + k as i32 > lots::FROM).then_some(lots::FROM));

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
    let mut lane_of: Vec<LaneOf> = Vec::new();
    let mut roads: Vec<(Vec<Vec2>, f32, LaneOf)> = Vec::new();
    let now = r.y0 + k as i32;
    for (i, st) in lay.streets.iter().enumerate() {
        let s = &st.way;
        let mid = (s.a + s.b) * 0.5;
        let middle = lots::in_middle(r, k, (mid.y.floor().clamp(0.0, (r.h - 1) as f32) as usize) * r.w + mid.x.floor().clamp(0.0, (r.w - 1) as f32) as usize);
        let rank = estates::Rank::of(s.kind, s.width, s.estate.is_some(), s.estate.is_some_and(|e| e.circle));
        let of = LaneOf { source: s.source(), year: r.y0 + s.since as i32, estate: s.estate.map(|e| (e.pattern, e.year)), circle: false, rank, middle };
        if s.road {
            // a road through is drawn by the ways between squares (`network`); on a town's
            // ground, from the year main streets are tarred, it is drawn again over that
            // as a town's main street, where it runs straight
            if estates::surface(rank, of.year, s.width, None, middle, true, now).is_some() && crate::network::route(g, s.a, s.b, r.round()).straight() {
                let mut got = Vec::new();
                lay.lengths(i, 0.0, 1.0, false, &mut got);
                roads.extend(got.into_iter().map(|(line, w)| (line, w, of)));
            }
            continue;
        }
        let half = 0.03 / st.len;
        let first = lanes.len();
        match s.kind {
            6 | 7 => {
                // an estate's street is laid whole where it runs, a little past its last steps
                // so that the streets of two squares beside one way meet
                lay.lengths(i, 0.0, 1.0, false, &mut lanes);
                let reach = 0.5 * st.len / st.steps as f32;
                for (line, _) in &mut lanes[first..] {
                    let n = line.len();
                    let (d0, d1) = ((line[0] - line[1]).normalize_or_zero(), (line[n - 1] - line[n - 2]).normalize_or_zero());
                    line[0] += d0 * reach;
                    line[n - 1] += d1 * reach;
                }
                lane_of.extend((first..lanes.len()).map(|_| of));
                // a cul-de-sac laid to its end turns in a circle there
                if s.estate.is_some_and(|e| e.circle) && lay.laid(i, st.steps) && lay.laid(i, st.steps.saturating_sub(1)) {
                    let ring: Vec<Vec2> = (0..=16).map(|j| s.b + Vec2::from_angle(j as f32 / 16.0 * std::f32::consts::TAU) * 0.0105).collect();
                    lanes.push((ring, 0.021));
                    lane_of.push(LaneOf { circle: true, ..of });
                }
                continue;
            }
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
        lane_of.extend((first..lanes.len()).map(|_| of));
    }
    // (for ISOSTATE_LOTS: the length of every road laid through town ground, by cell, which the
    // roads' own ribbons draw; the lanes are measured as drawn)
    let mut laid: Vec<(u32, Source, i32, f32)> = Vec::new();
    if std::env::var_os("ISOSTATE_LOTS").is_some() {
        for (i, st) in lay.streets.iter().enumerate().filter(|(_, st)| st.way.road) {
            let step = st.len / st.steps as f32;
            for j in 0..=st.steps {
                if lay.laid(i, j) {
                    let p = st.at(j);
                    let c = ((p.y.floor().max(0.0) as usize).min(r.h - 1) * r.w + (p.x.floor().max(0.0) as usize).min(r.w - 1)) as u32;
                    laid.push((c, st.way.source(), r.y0 + st.way.since as i32, step));
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
    let stories: HashMap<u32, Story> = stories.into_iter().map(|(c, (story, _))| (c, story)).collect();
    let whats = what_stands(r, k, &sq, &standing, &stories);
    Layout { lanes, lane_of, roads, estates, laid, standing, whats, stories, ms: [ms(t0, t1), ms(t1, t2), ms(t2, t3)], why }
}

/// Put the houses in order, street by street and along each, and say of each
/// whether a house stands on the next plot to its left and to its right.
/// With `old` (the forms' year, `lots::FROM`, in a layout after it), a house
/// built by then counts only the houses that were built by then too: what a
/// house of 1900 is drawn as does not change when a lot beside it is built.
fn find_neighbours(standing: &mut [Stood], old: Option<i32>) {
    standing.sort_by_key(|s| (s.lot.pl.run, s.lot.pl.index));
    for i in 0..standing.len() {
        let counts = |j: usize| old.is_none_or(|y| standing[i].year > y || standing[j].year <= y);
        let next_to = |j: usize| standing[j].lot.pl.run == standing[i].lot.pl.run && (standing[j].lot.pl.index - standing[i].lot.pl.index).abs() == 1 && counts(j);
        let (before, after) = (i > 0 && next_to(i - 1), i + 1 < standing.len() && next_to(i + 1));
        standing[i].neighbours = if standing[i].lot.pl.run.1 == 1 { (after, before) } else { (before, after) };
    }
    // round a market place the last plot's neighbour is the first: the ring closes on itself
    let ring = RING_PLOTS - 1;
    for i in 0..standing.len() {
        if standing[i].lot.kind == 9 && standing[i].lot.pl.index == 0 {
            let run = standing[i].lot.pl.run;
            if let Some(j) = (i + 1..standing.len()).take_while(|&j| standing[j].lot.pl.run == run).find(|&j| standing[j].lot.pl.index == ring && old.is_none_or(|y| (standing[i].year <= y) == (standing[j].year <= y))) {
                standing[i].neighbours.0 = true;
                standing[j].neighbours.1 = true;
            }
        }
    }
}

/// What stands on a plot: its kind, how tall it is, and what else its
/// generator is told that the plot alone does not say.
#[derive(Clone, Copy, Debug, PartialEq)]
enum What {
    /// a cottage, and the age its second storey went on in
    Cottage { storeys: u8, raised: Option<Era> },
    /// a townhouse in the manner of an age, perhaps a shop, perhaps given a new front in a later age
    Townhouse { storeys: u8, era: Era, shop: bool, fronted: Option<Era> },
    Villa,
    /// one of a brick row: `row` is how many houses the row has, for the first of them, and 0 for the rest
    Terraced { storeys: u8, row: u8 },
}

impl What {
    /// 0 a cottage, 1 a townhouse, 2 a villa, 3 one of a row.
    fn kind(&self) -> u8 {
        match self {
            What::Cottage { .. } => 0,
            What::Townhouse { .. } => 1,
            What::Villa => 2,
            What::Terraced { .. } => 3,
        }
    }

    fn storeys(&self) -> u8 {
        match *self {
            What::Cottage { storeys, .. } | What::Townhouse { storeys, .. } | What::Terraced { storeys, .. } => storeys,
            What::Villa => 2,
        }
    }
}

/// What stands on each plot of `standing` (rule 9). Everything a house grows
/// into is read from the most houses its cell has had (`Story::most`) and
/// from the year it was built, so that it holds from year to year: a house
/// raised stays raised, a townhouse is not a cottage again, and a villa is a
/// villa by the works its cell had the year it went up, not this year's.
fn what_stands(r: &Replay, k: usize, sq: &[Square], standing: &[Stood], stories: &HashMap<u32, Story>) -> Vec<What> {
    let mut whats = Vec::with_capacity(standing.len());
    // (after 1900, where the forms build the town, a house grows as far as its cell had grown by 1900)
    let grown_to = if r.builds.kept() { ((lots::FROM - r.y0).max(0) as usize).min(k) } else { k };
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
            let storeys = 2 + (story.most_to(k, grown_to) >= first.before + 96 + (32.0 * first.lot.pl.u) as usize) as u8;
            whats.push(What::Terraced { storeys, row: (j - i) as u8 });
            whats.extend((i + 1..j).map(|_| What::Terraced { storeys, row: 0 }));
        } else {
            let s = first;
            let pl = &s.lot.pl;
            let seed = Seed::new(pl.u, pl.v);
            let most = story.most_to(k, grown_to);
            let (joined, both) = (s.neighbours.0 || s.neighbours.1, s.neighbours.0 && s.neighbours.1);
            // villas: on the lanes and the ways out, on ground that had little works on it
            // the year the house went up, and never in a row
            let villa = pl.row.is_none() && s.lot.kind != 9 && !story.node && houses::villa_plot(s.year, seed) && r.works_share((s.year - r.y0).clamp(0, k as i32) as usize, s.lot.cell as usize) < 0.15;
            // rebuilt as a townhouse, the full width of its plot, once the cell has this many houses:
            // soon on a road through the town or round its market place, later on the old back
            // streets (those of the brick years are terraces from the first)
            let rebuilt = s.before + if s.lot.road { 10 + (38.0 * pl.v) as usize } else { 28 + (56.0 * pl.v) as usize };
            whats.push(if villa {
                What::Villa
            } else if joined && most >= rebuilt && (s.lot.road || s.year < TERRACE_ERA) {
                // a third storey on a road where the frontage is whole and the cell has grown as much
                // again; a fourth only round the market place of a great town
                let great = s.lot.kind == 9 && sq.iter().any(|q| q.big && q.at.distance(pl.p) < 0.2);
                let raised = rebuilt + 36 + (56.0 * pl.u) as usize;
                let third = s.lot.road && both && most >= raised;
                let fourth = third && great && most >= rebuilt + 120;
                // one in two of the houses given a third storey were given a new front with it, in
                // the manner of the year the cell had the houses for it (the generator shows it only
                // where that is a later age than the house's own)
                let fronted = (third && seed.pick(50) < 0.5).then(|| Era::of(story.year_of(r, k, raised, pl.u).max(s.year)));
                What::Townhouse {
                    storeys: 2 + third as u8 + fourth as u8,
                    era: Era::of(story.year_of(r, k, rebuilt, pl.u).max(s.year)),
                    shop: s.lot.road && houses::shop_at((most.saturating_sub(s.before) as f32 / 84.0).min(1.0), seed),
                    fronted,
                }
            } else {
                // a cottage: one storey, and a second once the cell has grown well past it,
                // raised in the year the cell first had the houses for it: the later work shows
                // above the old where that is a later age than the cottage's own
                let second = s.before + 36 + (88.0 * pl.v) as usize;
                let storeys = 1 + (most >= second) as u8;
                What::Cottage { storeys, raised: (storeys == 2).then(|| Era::of(story.year_of(r, k, second, pl.u).max(s.year))) }
            });
        }
        i = j;
    }
    whats
}

/// The houses of the kept towns for year `k`, built into `into`.
#[allow(clippy::too_many_arguments)]
pub(super) fn houses_kept(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, lk: &Look, parishes: &[Parish], near_plot: &dyn Fn(f32, f32) -> bool, yards: Option<&Yards>, land: &world::YearLand, into: &mut Into) -> Census {
    let town = &land.town;
    let layout = lay_out(r, g, cen, k, parishes, near_plot, yards, land);
    steadiness(r, k, &layout);
    let Layout { lanes, lane_of, roads, estates, laid, standing, whats, ms: laying, .. } = layout;
    let t3 = std::time::Instant::now();
    // after 1900: which plots are built as forms of their year, and which whole blocks stand (`lots.rs`)
    let lots::Lots { drawn, mut blocks } = lots::drawn_on(r, g, k, &standing, near_plot);
    // and the estates' whole blocks
    for (s, d) in standing.iter().zip(&drawn) {
        if s.lot.kind == 8 && matches!(d, lots::Drawn::Form(..)) {
            let front = Vec2::new(s.lot.pl.yaw.sin(), s.lot.pl.yaw.cos());
            blocks.push((lots::Footprint { at: s.lot.pl.p, along: Vec2::new(front.y, -front.x), front, half: (s.lot.pl.width * 0.5, s.lot.pl.depth * 0.5) }, 1));
        }
    }
    let colour = *lk.ground.street_earth;
    let mut ribbon = |line: &[Vec2], width: f32, colour: isostate_gen::colour::Rgba, lift: f32| {
        // (a lane runs on up to a whole block and on from it, not through it)
        let mut run: Vec<Vec3> = Vec::new();
        for (i, p) in line.iter().enumerate() {
            let inside = |q: Vec2| blocks.iter().any(|(f, _)| f.holds(q));
            let cut = inside(*p) || (i > 0 && inside((line[i - 1] + *p) * 0.5));
            if cut {
                if run.len() >= 2 {
                    into.houses.ribbon(&run, width, colour);
                }
                run.clear();
                if inside(*p) {
                    continue;
                }
            }
            run.push(Vec3::new(p.x, world::height_at(g, p.x, p.y).max(0.0) + lift, p.y));
        }
        if run.len() >= 2 {
            into.houses.ribbon(&run, width, colour);
        }
    };
    let now = r.y0 + k as i32;
    // (for ISOSTATE_LOTS: each street drawn after 1900 and its surface)
    let mut surfaced: Vec<(&[Vec2], estates::Surface)> = Vec::new();
    for ((line, width), of) in lanes.iter().zip(&lane_of) {
        // a street made up in the way of its age and rank (`estates::surface`), with its edges
        if let Some(s) = estates::surface(of.rank, of.year, *width, of.estate.map(|e| e.0), of.middle, false, now) {
            paint(&mut ribbon, lk, line, &s, 0.0);
            surfaced.push((line, s));
            continue;
        }
        match of.estate {
            None => ribbon(line, *width, colour, 0.006),
            Some((pattern, year)) => {
                // an estate's street in the surface of its era, with its edges: flagged footways
                // and setts in the by-law years, grass verges in the garden suburb, footways and
                // tarmac after
                let (surface, edge, wide) = street_surface(lk, pattern, year);
                ribbon(line, *width + 2.0 * wide, edge, 0.0056);
                ribbon(line, *width, surface, 0.0063);
            }
        }
    }
    // the roads through, tarred, as the town's main streets, over the ways' own ribbon
    for (line, width, of) in &roads {
        if let Some(s) = estates::surface(of.rank, of.year, *width, None, of.middle, true, now) {
            paint(&mut ribbon, lk, line, &s, crate::network::ROAD_LIFT - 0.0056 + 0.0006);
            surfaced.push((line, s));
        }
    }

    // the houses, street by street
    let mut census = Census::default();
    let mut on_cell: HashMap<u32, usize> = HashMap::default();
    for s in &standing {
        *on_cell.entry(s.lot.cell).or_default() += 1;
    }
    for (&c, cell) in &town.cells {
        census.cells += 1;
        census.filled += cell.filled as usize;
        census.park += cell.park as usize;
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
    // a filled square's ground that no street's plots reach is built over close by close (`lots::infill`)
    let infill = lots::infill(r, g, k, town, &standing, &blocks, &lanes, near_plot, lk, into);
    census.infill = infill.len();
    census.infill_estates = infill.iter().filter(|f| estates.contains_key(&(((f.at.y.floor().max(0.0) as usize).min(r.h - 1) * r.w + (f.at.x.floor().max(0.0) as usize).min(r.w - 1)) as u32))).count();
    census.estates = estates.len();
    census.estate_lots = standing.iter().filter(|s| s.lot.estate.is_some()).count();
    // (for ISOSTATE_LOTS: the ground every building drawn stands on)
    let telling = std::env::var_os("ISOSTATE_LOTS").is_some();
    let mut rects: Vec<lots::Footprint> = Vec::new();
    // (and every building's middle, its storeys, and whether it fronts a main street)
    let mut heights: Vec<(Vec2, u8, bool)> = Vec::new();
    let mut others: Vec<(Vec2, f32, u8, u32)> = Vec::new();
    if telling {
        lots::tell(r, k, &standing, &whats.iter().map(What::storeys).collect::<Vec<u8>>(), &drawn);
        // the forms of five storeys or more on the lots, each where it stands and facing as its plot does
        let mut tall: Vec<(Vec2, f32, u8, u32)> = Vec::new();
        let mut shapes = Vec::new();
        for (i, d) in drawn.iter().enumerate() {
            if let lots::Drawn::Form(b, plots) = *d {
                let (a, z) = (&standing[i].lot.pl, &standing[i + plots - 1].lot.pl);
                tall.push(((a.p + z.p) * 0.5, a.yaw, b.storeys, standing[i].lot.cell));
                if blocks.iter().any(|(f, _)| f.holds((a.p + z.p) * 0.5 + Vec2::new(a.yaw.sin(), a.yaw.cos()) * (a.depth * 0.5 - lots::BLOCK_DEPTH * 0.5))) {
                    shapes.push(b.shape);
                }
            }
        }
        others = lots::tell_square(r, k, &tall, &lanes, &blocks, &shapes);
    }
    for (i, (s, what)) in standing.iter().zip(&whats).enumerate() {
        let pl = &s.lot.pl;
        let seed = Seed::new(pl.u, pl.v);
        let upland = r.terrain[s.lot.cell as usize] >= 2;
        let street = houses::Street::of(s.lot.family);
        let storeys = what.storeys();
        match drawn[i] {
            lots::Drawn::House => {}
            lots::Drawn::Covered => continue,
            lots::Drawn::Form(b, plots) => {
                let drawn = lots::draw(g, &standing, i, plots, &b, lk, into);
                if telling {
                    heights.extend(drawn.iter().map(|f| (f.at, b.storeys, s.lot.road || s.lot.kind == 9)));
                    rects.extend(drawn);
                }
                census.forms += 1;
                census.rebuilt += (s.year <= lots::FROM) as usize;
                let tall = (b.storeys as usize).clamp(1, 3) - 1;
                census.storeys[tall] += plots;
                continue;
            }
        }
        if telling {
            let front = Vec2::new(pl.yaw.sin(), pl.yaw.cos());
            let length = match *what {
                What::Terraced { row: 0, .. } => 0.0,
                What::Terraced { row, .. } => pl.p.distance(standing[i + row as usize - 1].lot.pl.p) + pl.width,
                _ => pl.width,
            };
            let mid = match *what {
                What::Terraced { row, .. } if row > 0 => (pl.p + standing[i + row as usize - 1].lot.pl.p) * 0.5,
                _ => pl.p,
            };
            if length > 0.0 {
                rects.push(lots::Footprint { at: mid, along: Vec2::new(front.y, -front.x), front, half: (length * 0.48, pl.depth * 0.5) });
                heights.push((mid, what.storeys(), s.lot.road || s.lot.kind == 9));
            }
        }
        match *what {
            What::Terraced { row: 0, .. } => continue,
            What::Terraced { row, .. } => {
                let (a, b) = (pl.p, standing[i + row as usize - 1].lot.pl.p);
                let mid = (a + b) * 0.5;
                let t = Terrace { houses: row as usize, length: a.distance(b) + pl.width * 0.96, depth: pl.depth, storeys, roof: s.roof, era: Era::of(s.year), upland, street, detail: into.lod.at(mid.x, mid.y), seed };
                houses::terrace(into.houses, into.smoke, &Site::on(ground(g, mid.x, mid.y), pl.yaw, &height), &t, lk);
                census.terraced += row as usize;
                census.storeys[(storeys as usize - 1).min(2)] += row as usize;
            }
            What::Villa => {
                let site = Site::on(ground(g, pl.p.x, pl.p.y), pl.yaw, &height);
                let v = Villa { plot_width: pl.width, plot_depth: pl.depth, roof: s.roof, era: Era::of(s.year), upland, street, detail: into.lod.at(pl.p.x, pl.p.y), seed };
                houses::villa(into.houses, into.smoke, &mut into.trees[leaf_group(pl.v)], &site, &v, lk);
                census.villas += 1;
                census.storeys[1] += 1;
            }
            What::Townhouse { era, shop, fronted, .. } => {
                let site = Site::on(ground(g, pl.p.x, pl.p.y), pl.yaw, &height);
                let t = Townhouse { plot_width: pl.width, plot_depth: pl.depth, storeys, roof: s.roof, era, upland, shop, neighbours: s.neighbours, street, fronted, detail: into.lod.at(pl.p.x, pl.p.y), seed };
                houses::townhouse(into.houses, into.smoke, &site, &t, lk);
                census.townhouses += 1;
                census.storeys[(storeys as usize - 1).min(2)] += 1;
                census.tall_alone += (storeys >= 3 && !(s.neighbours.0 && s.neighbours.1)) as usize;
            }
            What::Cottage { raised, .. } => {
                let site = Site::on(ground(g, pl.p.x, pl.p.y), pl.yaw, &height);
                let c = Cottage { plot_width: pl.width, plot_depth: pl.depth, storeys, roof: s.roof, era: Era::of(s.year), upland, neighbours: s.neighbours, street, raised, detail: into.lod.at(pl.p.x, pl.p.y), seed };
                houses::cottage(into.houses, into.smoke, &mut into.trees[leaf_group(pl.v)], &site, &c, lk);
                census.cottages += 1;
                census.storeys[storeys as usize - 1] += 1;
            }
        }
    }
    census.drawn = standing.len();
    // ISOSTATE_OLD_PLOTS=DIR: write every plot of ground built by 1900 standing this year, with
    // its year and how it is drawn, to DIR/plots-YEAR.txt, for setting the old streets beside the
    // new (`ISOSTATE_STREETS=old`): nothing built to 1900 may change
    if let Some(dir) = std::env::var_os("ISOSTATE_OLD_PLOTS") {
        // (`ISOSTATE_OLD_PLOTS_ALL=1`: every plot, of whatever ground)
        let all = std::env::var_os("ISOSTATE_OLD_PLOTS_ALL").is_some();
        let mut lines: Vec<String> = standing
            .iter()
            .zip(&whats)
            .zip(&drawn)
            .filter(|((s, _), _)| all || r.joined(k, s.lot.cell as usize).is_some_and(|j| j <= lots::FROM))
            .map(|((s, w), d)| format!("{},{} {} {} {} {} {:.5} {:.5} {:?} {:?} joined {:?}", s.lot.cell as usize % r.w, s.lot.cell as usize / r.w, s.lot.pl.run.0, s.lot.pl.run.1, s.lot.pl.index, s.year, s.lot.pl.p.x, s.lot.pl.p.y, w, d, r.joined(k, s.lot.cell as usize)))
            .collect();
        lines.sort();
        let path = std::path::Path::new(&dir).join(format!("plots-{}.txt", r.y0 + k as i32));
        if let Err(e) = std::fs::write(&path, lines.join("\n")) {
            eprintln!("could not write {}: {e}", path.display());
        }
    }
    if telling {
        // (the offices and the closes built over by themselves, noted as they were drawn)
        let lots_end = heights.len();
        heights.extend(others.iter().map(|o| (o.0, o.2, false)));
        estates::tell(r, g, k, land, &estates, &laid, &lanes, &lane_of, &standing, &drawn, &rects, &infill, near_plot);
        estates::tell_heights(r, k, &heights, lots_end, &surfaced, &estates);
    }
    if std::env::var_os("ISOSTATE_TIMING").is_some() {
        eprintln!("timing kept towns: streets {:.0} ms, plots {:.0} ms, turns {:.0} ms, houses {:.0} ms", laying[0], laying[1], laying[2], t3.elapsed().as_secs_f64() * 1000.0);
    }
    census
}

/// Draw a street's line in its surface (`estates::surface`): its edges
/// under it, its carriageway, and down a main street's middle a broken white
/// line, dashes of 0.02 every 0.045. `raise` lifts it all (over a road's own
/// ribbon).
fn paint(ribbon: &mut dyn FnMut(&[Vec2], f32, isostate_gen::colour::Rgba, f32), lk: &Look, line: &[Vec2], s: &estates::Surface, raise: f32) {
    let gr = &lk.ground;
    let made = match s.made {
        estates::Made::Earth => *gr.street_earth,
        estates::Made::Setts => *gr.yard_setts,
        estates::Made::Macadam => *gr.road_gravel,
        estates::Made::Worn => *gr.tarmac_worn,
        estates::Made::Tarmac => *gr.tarmac,
    };
    match s.edge {
        estates::Edge::None => {}
        estates::Edge::Footway => ribbon(line, s.width + 2.0 * s.edge_width, *gr.shop_pavement, 0.0056 + raise),
        estates::Edge::Verge => ribbon(line, s.width + 2.0 * s.edge_width, *gr.churchyard_grass, 0.0056 + raise),
    }
    ribbon(line, s.width, made, if s.made == estates::Made::Earth && s.edge == estates::Edge::None { 0.006 } else { 0.0063 } + raise);
    if s.line {
        // the dashes, measured along the whole line so that they run on over its bends
        let (dash, every) = (0.02, 0.045);
        let mut at = 0.0f32;
        for w in line.windows(2) {
            let len = w[0].distance(w[1]);
            if len < 1e-5 {
                continue;
            }
            let mut t = (at / every).ceil() * every - at;
            while t < len {
                let end = (t + dash).min(len);
                ribbon(&[w[0].lerp(w[1], t / len), w[0].lerp(w[1], end / len)], 0.0012, *gr.road_line, 0.0067 + raise);
                t += every;
            }
            at += len;
        }
    }
}

/// The surface of an estate's street of a pattern laid out in a year, the
/// colour of its edges, and how wide each edge is: setts between flagged
/// footways in the by-law years (and on blocks to 1918), worn tarmac between
/// grass verges in the garden suburb, tarmac between footways after.
fn street_surface(lk: &Look, pattern: estates::Pattern, year: i32) -> (isostate_gen::colour::Rgba, isostate_gen::colour::Rgba, f32) {
    let gr = &lk.ground;
    match pattern {
        estates::Pattern::ByLaw => (*gr.yard_setts, *gr.shop_pavement, 0.004),
        estates::Pattern::Blocks if year <= 1918 => (*gr.yard_setts, *gr.shop_pavement, 0.005),
        estates::Pattern::Blocks => (*gr.tarmac, *gr.shop_pavement, 0.005),
        estates::Pattern::Garden => (*gr.tarmac_worn, *gr.churchyard_grass, 0.006),
        estates::Pattern::Loop => (*gr.tarmac, *gr.shop_pavement, 0.004),
        estates::Pattern::Closes => (*gr.tarmac, *gr.shop_pavement, 0.003),
    }
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
    // by plot: the cell, the year the house on it went up, its kind and its storeys
    type Plots = std::collections::HashMap<((i32, u8), i32), (u32, i32, u8, u8)>;
    static LAST: std::sync::Mutex<Option<(usize, Plots, HashMap<u32, usize>)>> = std::sync::Mutex::new(None);
    if std::env::var_os("ISOSTATE_TOWNS").is_none() {
        return;
    }
    let now: Plots = layout.standing.iter().zip(&layout.whats).map(|(s, w)| ((s.lot.pl.run, s.lot.pl.index), (s.lot.cell, s.year, w.kind(), w.storeys()))).collect();
    let houses: HashMap<u32, usize> = layout.stories.iter().map(|(&c, s)| (c, s.houses)).collect();
    let mut last = LAST.lock().unwrap();
    if let Some((_, before, had)) = last.as_ref().filter(|l| l.0 + 1 == k) {
        let gone: Vec<u32> = before.iter().filter(|(plot, _)| !now.contains_key(plot)).map(|(_, &(c, ..))| c).collect();
        // the houses that stand where they stood and are another kind of house, those of
        // them that were cottages (a cottage rebuilt as a townhouse is the town growing;
        // any other change is a house that did not stay as it was built), and those that
        // have lost a storey
        let both: Vec<((u8, u8), (u8, u8))> = before.iter().filter_map(|(plot, was)| now.get(plot).map(|n| ((was.2, was.3), (n.2, n.3)))).collect();
        let (rekinded, rebuilt, lower) = (both.iter().filter(|(a, b)| a.0 != b.0).count(), both.iter().filter(|(a, b)| a.0 == 0 && b.0 == 1).count(), both.iter().filter(|(a, b)| b.1 < a.1).count());
        eprintln!("towns {}: {} houses that stand where they stood are another kind of house, {} of them cottages rebuilt as townhouses; {} have lost a storey", r.y0 + k as i32, rekinded, rebuilt, lower);
        if rekinded > rebuilt || lower > 0 {
            let names = ["cottage", "townhouse", "villa", "terraced"];
            let mut pairs: std::collections::BTreeMap<(u8, u8), usize> = Default::default();
            for (a, b) in both.iter().filter(|(a, b)| a.0 != b.0 && !(a.0 == 0 && b.0 == 1)) {
                *pairs.entry((a.0, b.0)).or_default() += 1;
            }
            let pairs: Vec<String> = pairs.iter().map(|(&(a, b), n)| format!("{n} {} to {}", names[a as usize], names[b as usize])).collect();
            let same_kind_lower = both.iter().filter(|(a, b)| a.0 == b.0 && b.1 < a.1).count();
            eprintln!("towns {}: not as built: {}; {} lost a storey and kept their kind", r.y0 + k as i32, pairs.join(", "), same_kind_lower);
        }
        let unasked = gone.iter().filter(|c| houses.get(c).copied().unwrap_or(0) >= had.get(c).copied().unwrap_or(0)).count();
        // the houses that stand where they stood but count as built in another year (a
        // house's year is its place in its cell's count), and those of them whose year
        // has crossed from the rows of cottages to the brick terraces or back
        let redated: Vec<(i32, i32)> = before.iter().filter_map(|(plot, was)| now.get(plot).filter(|n| n.1 != was.1).map(|n| (was.1, n.1))).collect();
        let restyled = redated.iter().filter(|(a, b)| (*a >= TERRACE_ERA) != (*b >= TERRACE_ERA)).count();
        eprintln!("towns {}: {} houses that stand where they stood count as built in another year, {} of them across {TERRACE_ERA}", r.y0 + k as i32, redated.len(), restyled);
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
        let layout = lay_out(r, g, &cen, k, &[], &|_, _| false, None, &land);
        let mut houses: Vec<House> = layout.standing.iter().map(|s| ((s.lot.pl.run, s.lot.pl.index), s.lot.cell, [s.lot.pl.p.x.to_bits(), s.lot.pl.p.y.to_bits()], s.year)).collect();
        houses.sort();
        (houses, land)
    }

    /// The houses a cell's residents ask for at `PEOPLE_TO_A_HOUSE` each, over the town.
    fn asked(r: &Replay, k: usize) -> usize {
        (0..r.n).map(|c| (r.residents(k, c) / world::PEOPLE_TO_A_HOUSE).round() as usize).sum()
    }

    /// The made-up town moved on to 1898, so that its seven years run 1898
    /// to 1904, with the two layers a record past 1900 keeps of its squares
    /// (`lots.rs`): in year `k` every square of town ground is built up
    /// `dens(k)` times and its land is worth `value(k)` times its town's of 1900.
    fn past_1900(dens: impl Fn(usize) -> f64, value: impl Fn(usize) -> f64) -> Replay {
        let mut r = made_up(true);
        r.y0 = 1898;
        r.header.meta.start_year = 1898;
        let n = r.n;
        r.builds.dens = vec![0; r.ny * n];
        r.builds.value = vec![0; r.ny * n];
        for k in 0..r.ny {
            for c in (0..n).filter(|&c| r.built[k * n + c] > 0) {
                r.builds.dens[k * n + c] = isostate_core::record::dens_byte(dens(k));
                r.builds.value[k * n + c] = isostate_core::record::value_byte(value(k));
            }
        }
        r
    }

    /// Every house of a year's layout by its plot: the year it went up, what
    /// stands (as `What` says it, in words) and how it is drawn.
    fn drawn_in(r: &Replay, g: &Ground, k: usize) -> std::collections::BTreeMap<((i32, u8), i32), (i32, String, lots::Drawn)> {
        let cen = crate::network::centres(r);
        let land = world::year_land(r, g, k);
        let layout = lay_out(r, g, &cen, k, &[], &|_, _| false, None, &land);
        let drawn = lots::drawn(r, k, &layout.standing);
        layout.standing.iter().zip(&layout.whats).zip(drawn).map(|((s, w), d)| ((s.lot.pl.run, s.lot.pl.index), (s.year, format!("{w:?}"), d))).collect()
    }

    /// A town whose ground never grew dear keeps in 1903 every house it had
    /// in 1900, as it was: none is rebuilt, raised or given a new front; and
    /// what is built on the square it takes in 1902 is built in the forms of
    /// its year. (Its old squares hold the people they held in 1900. Where
    /// they grow, a house the old rule dates before 1900 can come to stand
    /// beside a house of 1900 that stood alone, which then joins it as a
    /// townhouse: see the notes of `look-follows`.)
    #[test]
    fn a_town_whose_ground_never_grew_dear_keeps_its_houses_of_1900() {
        let mut r = past_1900(|_| 1.0, |_| 0.8);
        let n = r.n;
        for c in (0..n).filter(|&c| r.built[2 * n + c] > 0) {
            for k in 3..r.ny {
                r.res[k * n + c] = r.res[2 * n + c];
            }
        }
        let g = world::build_ground(&r);
        let (then, now) = (drawn_in(&r, &g, 2), drawn_in(&r, &g, 5));
        assert!(then.len() > 80, "{} houses in 1900", then.len());
        for (plot, was) in &then {
            assert_eq!(was.2, lots::Drawn::House, "{plot:?} in 1900");
            assert_eq!(now.get(plot), Some(was), "{plot:?}: in 1900 {was:?}");
        }
        let new: Vec<_> = now.iter().filter(|(p, _)| !then.contains_key(p)).collect();
        assert!(!new.is_empty() && new.iter().all(|(_, d)| d.0 > lots::FROM && d.2 != lots::Drawn::House), "{new:?}");
        // the same record draws the same town, however often it is drawn
        assert_eq!(drawn_in(&r, &g, 5), now);
    }

    /// No paddock or field is drawn inside a filled square. Past 1900, in a
    /// record that says how much of each square is built and which is park
    /// (the core's `fill` and `park`), every dry close of a filled square is
    /// built up or is its park, never the open ground of a town's edge; the
    /// park is the last of its order and about the share the record keeps;
    /// and the ground shader is told which parcels lie in a filled square. A
    /// square at the edge, built on a fifth, keeps its open closes; and 1900,
    /// whose fill the record keeps too, is laid out as with no fill at all.
    #[test]
    fn no_field_or_paddock_is_drawn_inside_a_filled_square() {
        let mut r = past_1900(|_| 1.2, |_| 2.0);
        let n = r.n;
        let last = r.ny - 1;
        // the newest square is the edge
        let edge = (0..n).filter(|&c| r.built[last * n + c] > 0).max_by_key(|&c| (r.built[last * n + c], c)).unwrap();
        r.builds.fill = vec![0; r.ny * n];
        r.builds.park = vec![0; r.ny * n];
        for k in 0..r.ny {
            for c in (0..n).filter(|&c| r.built[k * n + c] > 0) {
                let (fill, park) = if c == edge { (52, 0) } else { (225, 25) };
                r.builds.fill[k * n + c] = fill;
                r.builds.park[k * n + c] = park;
            }
        }
        let g = world::build_ground(&r);
        let land = world::year_land(&r, &g, last);
        let mut filled = 0;
        for (&c, cell) in &land.town.cells {
            let dry = &cell.order[..cell.closes as usize];
            if c as usize == edge {
                assert!(!cell.filled && cell.built < cell.closes, "the edge square: {} of {} closes built", cell.built, cell.closes);
                continue;
            }
            assert!(cell.filled, "cell {c}");
            filled += 1;
            let (mut park, mut all) = (0.0, 0.0);
            for (i, &(parcel, sub, area)) in dry.iter().enumerate() {
                let (built, kept) = (land.town.built_up(parcel, sub), land.town.park_at(parcel, sub));
                assert!(built != kept, "cell {c}, close {i}: built {built}, park {kept}");
                assert_eq!(kept, i >= cell.built as usize, "cell {c}, close {i}: the park is the last of the order");
                park += if kept { area } else { 0.0 };
                all += area;
            }
            assert!(cell.park > 0 && park / all > 0.04 && park / all < 0.2, "cell {c}: park {} of {all}", park);
        }
        assert!(filled >= 3, "{filled} filled squares");
        // the shader is told which parcels lie in a filled square
        let px = crate::terrain::year_parcels(&r, &g, last, &land);
        for (i, &cls) in land.class.iter().enumerate().filter(|&(_, c)| *c == world::URBAN) {
            let inside = land.town.in_filled(g.land.wcell[i]);
            assert_eq!(px[i * 4 + 1] & crate::terrain::FILLED_BIT != 0, inside, "parcel {i} of class {cls}");
        }
        // 1900 is as it was
        let mut none = r.clone();
        none.builds.fill.clear();
        none.builds.park.clear();
        assert_eq!(year(&r, &g, 2).0, year(&none, &g, 2).0);
    }

    /// A town whose ground grows dear after 1900 rebuilds some of its old
    /// lots in the forms of the years it grew in, and every house of 1900 not
    /// rebuilt stands as it was.
    #[test]
    fn a_town_whose_ground_grows_dear_rebuilds_some_and_keeps_the_rest() {
        let r = past_1900(|k| [1.0, 1.0, 1.0, 1.3, 1.6, 1.9, 1.9][k], |k| if k <= 2 { 1.0 } else { 6.0 });
        let g = world::build_ground(&r);
        let (then, now) = (drawn_in(&r, &g, 2), drawn_in(&r, &g, 5));
        let rebuilt = then.keys().filter(|p| matches!(now.get(p).map(|d| d.2), Some(lots::Drawn::Form(..)) | Some(lots::Drawn::Covered))).count();
        assert!(rebuilt > 0 && rebuilt * 2 < then.len(), "{rebuilt} of {} rebuilt", then.len());
        for (plot, was) in &then {
            match now.get(plot) {
                Some((_, _, lots::Drawn::Form(b, _))) => assert!(b.year > lots::FROM && b.storeys >= 4, "{plot:?}: {b:?}"),
                Some((_, _, lots::Drawn::Covered)) => {}
                other => assert_eq!(other, Some(was), "{plot:?}"),
            }
        }
    }

    /// The made-up town moved on past 1900 as `past_1900` moves it, in the
    /// second layout of the town bytes that a record past 1900 keeps, with its
    /// newest square (north of the market place, town ground from the fifth
    /// year, 1902) joined in 1901 and filled whole with a park from then: an
    /// estate (`estates.rs`). The rest keep the ground they had by 1900.
    fn with_an_estate() -> (Replay, u32) {
        let mut r = past_1900(|_| 1.0, |k| if k < 4 { 1.0 } else { 2.0 });
        let n = r.n;
        let node = r.header.towns[0].node;
        let estate = node - r.w;
        r.header.meta.town_bytes = 2;
        r.builds.fill = vec![0; r.ny * n];
        r.builds.park = vec![0; r.ny * n];
        for k in 0..r.ny {
            for c in 0..n {
                let o = k * n + c;
                if r.built[o] > 0 {
                    // (the same year to a year, in steps of two, and the same people in sixteens)
                    let year = replay::BUILT_YEAR0 + r.built[o] as i32;
                    r.built[o] = ((year - replay::BUILT_YEAR0) / 2) as u8;
                    r.res[o] = (r.res[o] as f32 / 2.0).round() as u8;
                }
            }
            if r.built[k * n + estate] > 0 {
                r.built[k * n + estate] = ((1901 - replay::BUILT_YEAR0) / 2) as u8;
                (r.builds.fill[k * n + estate], r.builds.park[k * n + estate]) = (225, 25);
            }
        }
        assert_eq!(r.joined(r.ny - 1, estate), Some(1901));
        (r, estate as u32)
    }

    /// A layout of the estate's record with the estates (`streets`) or without them.
    fn with_streets(r: &Replay, g: &Ground, k: usize, streets: bool) -> Layout {
        estates::OLD_IN_TEST.with(|o| o.set(Some(!streets)));
        let l = laid(r, g, k, &|_, _| false);
        estates::OLD_IN_TEST.with(|o| o.set(None));
        l
    }

    /// Nothing built by 1900 changes when the estates are laid out: every
    /// house standing on ground that was town ground by 1900 stands where it
    /// stood, of the year it was, and is the same house, in 1900 and after,
    /// with the estates' streets or without them; and 1900 itself is laid
    /// out the same to the last house.
    #[test]
    fn the_ground_built_by_1900_is_the_same_with_the_estates_or_without() {
        let (r, estate) = with_an_estate();
        let g = world::build_ground(&r);
        let old = |l: &Layout| -> std::collections::BTreeMap<((i32, u8), i32), (u32, [u32; 2], i32, What)> {
            l.standing.iter().zip(&l.whats).filter(|(s, _)| s.lot.cell != estate && s.year <= lots::FROM).map(|(s, w)| ((s.lot.pl.run, s.lot.pl.index), (s.lot.cell, [s.lot.pl.p.x.to_bits(), s.lot.pl.p.y.to_bits()], s.year, *w))).collect()
        };
        for k in [2, 4, 6] {
            let (with, without) = (with_streets(&r, &g, k, true), with_streets(&r, &g, k, false));
            let (a, b) = (old(&with), old(&without));
            assert!(a.len() > 80, "year {k}: {} houses of 1900", a.len());
            assert_eq!(a, b, "year {k}");
            assert_eq!(with.estates.is_empty(), k < 4, "year {k}: {:?}", with.estates);
        }
        let (with, without) = (with_streets(&r, &g, 2, true), with_streets(&r, &g, 2, false));
        let all = |l: &Layout| l.standing.iter().map(|s| ((s.lot.pl.run, s.lot.pl.index), s.year)).collect::<Vec<_>>();
        assert_eq!(all(&with), all(&without));
        assert_eq!(with.lanes, without.lanes);
    }

    /// An estate has streets of its own, of the pattern of its year, laid on
    /// its own square's built-up ground and never over its park; its plots
    /// front them and are all built the year it was laid out; and once laid,
    /// its streets and houses stand as they were in every later year, however
    /// the years are laid out.
    #[test]
    fn an_estate_has_streets_of_its_own_and_keeps_them() {
        let (r, estate) = with_an_estate();
        let g = world::build_ground(&r);
        let first = with_streets(&r, &g, 4, true);
        let e = first.estates[&estate];
        assert_eq!((e.year, e.pattern, e.walls), (1902, estates::Pattern::ByLaw, true), "{e:?}");
        // its streets: drawn, on its square's ground (its parcels, which keep the field
        // system's lines and not the map's) but for the half step they reach past their last,
        // and none over its park
        let land = world::year_land(&r, &g, 4);
        let mut closes = world::CloseLookup::new(&g);
        let (mut points, mut outside) = (0, 0);
        for ((line, _), of) in first.lanes.iter().zip(&first.lane_of).filter(|(_, of)| of.estate.is_some()) {
            assert_eq!(of.estate, Some((estates::Pattern::ByLaw, 1902)));
            for (i, &p) in line.iter().enumerate() {
                let (parcel, sub) = closes.at(p.x, p.y);
                let end = i == 0 || i + 1 == line.len();
                points += 1;
                outside += (g.land.wcell[parcel as usize] != estate && !end) as usize;
                assert!(!land.town.park_at(parcel, sub) || end, "a street over the park at {p}");
            }
        }
        assert!(points > 60 && outside == 0, "{points} points of street, {outside} off the square's ground");
        // its plots: many, all of 1902, each fronting an estate street
        let plots: Vec<&Stood> = first.standing.iter().filter(|s| s.lot.estate.is_some()).collect();
        assert!(plots.len() > 40, "{} plots on the estate", plots.len());
        assert!(plots.iter().all(|s| s.year == 1902 && s.lot.cell == estate && matches!(s.lot.kind, 6 | 7)));
        // the same in the years after, laid out in any order
        let key = |l: &Layout| -> (Vec<(Vec<[u32; 2]>, u32)>, Vec<(((i32, u8), i32), i32)>) {
            let lanes = l.lanes.iter().zip(&l.lane_of).filter(|(_, of)| of.estate.is_some()).map(|((line, w), _)| (line.iter().map(|p| [p.x.to_bits(), p.y.to_bits()]).collect(), w.to_bits())).collect();
            let mut plots: Vec<_> = l.standing.iter().filter(|s| s.lot.estate.is_some()).map(|s| ((s.lot.pl.run, s.lot.pl.index), s.year)).collect();
            plots.sort();
            (lanes, plots)
        };
        let was = key(&first);
        for k in [6, 5, 4] {
            assert_eq!(key(&with_streets(&r, &g, k, true)), was, "year {k}");
        }
        let g = world::build_ground(&r);
        assert_eq!(key(&with_streets(&r, &g, 6, true)), was, "year 6 on new ground");
    }

    /// The patterns by year and land value, and the streets of each between
    /// two parallel ways: laid from each way in its own frame, along it and
    /// out from it, with their turning circles where they are culs-de-sac.
    #[test]
    fn each_era_lays_its_own_pattern_from_each_way() {
        use estates::Pattern;
        assert_eq!(Pattern::of(1905, 0.6), Pattern::ByLaw);
        assert_eq!(Pattern::of(1930, 1.3), Pattern::Garden);
        assert_eq!(Pattern::of(1960, 2.4), Pattern::Loop);
        assert_eq!(Pattern::of(1980, 1.5), Pattern::Closes);
        assert_eq!(Pattern::of(1925, lots::BLOCK_VALUE), Pattern::Blocks);
        let r = made_up(true);
        let c = (5 * r.w + 8) as u32;
        // two ways along the map's rows, one through the square's middle and one beside it
        let mains = [(Vec2::new(7.5, 5.5), Vec2::new(9.5, 5.5), 100, 0), (Vec2::new(7.5, 6.3), Vec2::new(9.5, 6.3), 200, 0)];
        for (year, value) in [(1905, 0.6), (1930, 1.3), (1960, 2.4), (1980, 1.5), (1925, 3.5)] {
            let pattern = Pattern::of(year, value);
            let mut one: HashMap<u32, estates::Estate> = HashMap::default();
            one.insert(c, estates::Estate { since: 1, year, pattern, value, walls: value >= estates::WALLS });
            let (streets, blocks) = estates::streets(&r, &one, &mains);
            for main in [100, 200] {
                let mine: Vec<&Way> = streets.iter().filter(|w| w.main == main).collect();
                assert!(!mine.is_empty(), "{pattern:?}: no street from way {main}");
                for w in &mine {
                    let e = w.estate.unwrap();
                    assert_eq!((e.cell, e.pattern, w.since), (c, pattern, 1));
                    // along the way or out from it, square to it
                    let d = (w.b - w.a).normalize();
                    assert!(w.kind == 6 && (d.y.abs() < 0.35 || e.between.is_some()) || w.kind == 7 && (d.x.abs() < 1e-3 || e.circle), "{pattern:?}: {:?} {d}", w.kind);
                }
                let circles = mine.iter().filter(|w| w.estate.unwrap().circle).count();
                assert_eq!(circles > 0, matches!(pattern, Pattern::Loop | Pattern::Closes), "{pattern:?}");
            }
            assert_eq!(!blocks.is_empty(), pattern == Pattern::Blocks, "{pattern:?}");
            // every street's number its own
            let mut ids: Vec<i32> = streets.iter().map(|w| w.id).chain(blocks.iter().map(|b| b.id)).collect();
            ids.sort();
            let all = ids.len();
            ids.dedup();
            assert_eq!(ids.len(), all, "{pattern:?}");
        }
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
    fn a_house_keeps_its_year_whatever_becomes_of_the_plots_about_it() {
        let r = made_up(true);
        let g = world::build_ground(&r);
        let cen = crate::network::centres(&r);
        let land = world::year_land(&r, &g, 5);
        // the town as it stands, and with a works' yard on a strip of its middle
        let node = Vec2::new(cen[5 * r.w + 5].0, cen[5 * r.w + 5].1);
        let free = lay_out(&r, &g, &cen, 5, &[], &|_, _| false, None, &land);
        let yard = lay_out(&r, &g, &cen, 5, &[], &|x, z| (x - node.x - 0.2).abs() < 0.12 && (z - node.y).abs() < 0.3, None, &land);
        let years = |l: &Layout| -> std::collections::BTreeMap<((i32, u8), i32), (usize, i32)> { l.standing.iter().map(|s| ((s.lot.pl.run, s.lot.pl.index), (s.before, s.year))).collect() };
        let (free, yard) = (years(&free), years(&yard));
        // the yard takes houses, and others go up elsewhere in their place
        let taken = free.keys().filter(|p| !yard.contains_key(p)).count();
        let placed = yard.keys().filter(|p| !free.contains_key(p)).count();
        assert!(taken >= 10 && placed >= 10, "{taken} houses taken by the yard, {placed} built elsewhere");
        // every house that stands in both went up in the same year, with as many before it
        for (plot, was) in &free {
            if let Some(now) = yard.get(plot) {
                assert_eq!(now, was, "the house on plot {plot:?}");
            }
        }
    }

    /// The made-up town laid out for year `k`, with `near_plot` for its works' yards and its water.
    fn laid(r: &Replay, g: &Ground, k: usize, near_plot: &dyn Fn(f32, f32) -> bool) -> Layout {
        lay_out(r, g, &crate::network::centres(r), k, &[], near_plot, None, &world::year_land(r, g, k))
    }

    #[test]
    fn a_house_s_kind_and_height_hold_from_year_to_year_once_built() {
        let r = made_up(true);
        let g = world::build_ground(&r);
        let mut before: Option<std::collections::BTreeMap<((i32, u8), i32), What>> = None;
        let (mut rebuilt, mut raised) = (0, 0);
        // the people grow every year to the sixth, and in the seventh a fifth of them are gone
        for k in 0..r.ny {
            let l = laid(&r, &g, k, &|_, _| false);
            let now: std::collections::BTreeMap<_, _> = l.standing.iter().zip(&l.whats).map(|(s, w)| ((s.lot.pl.run, s.lot.pl.index), *w)).collect();
            assert_eq!(now.len(), l.standing.len());
            if let Some(was) = &before {
                for (plot, a) in was {
                    let b = now.get(plot).unwrap_or_else(|| panic!("year {k}: the house on plot {plot:?} is gone"));
                    // a cottage may be rebuilt as a townhouse: nothing else changes a house's kind
                    assert!(a.kind() == b.kind() || (a.kind() == 0 && b.kind() == 1), "year {k}, plot {plot:?}: {a:?}, then {b:?}");
                    assert!(b.storeys() >= a.storeys(), "year {k}, plot {plot:?}: {a:?}, then {b:?}");
                    rebuilt += (a.kind() != b.kind()) as usize;
                    raised += (b.storeys() > a.storeys()) as usize;
                    // what was told of it then is told of it now: the age it was raised in, and its front's
                    match (a, b) {
                        (What::Cottage { storeys: 2, raised: then }, What::Cottage { raised: now, .. }) => assert_eq!(then, now, "year {k}, plot {plot:?}"),
                        (What::Townhouse { era: then, fronted: Some(front), .. }, What::Townhouse { era: now, fronted, .. }) => assert_eq!((then, Some(*front)), (now, *fronted), "year {k}, plot {plot:?}"),
                        _ => {}
                    }
                }
            }
            before = Some(now);
        }
        assert!(rebuilt > 10 && raised > 10, "{rebuilt} cottages rebuilt as townhouses and {raised} houses raised in six years");
    }

    /// The same when a cell loses more people than its houses outlast
    /// (`world::HOUSES_KEEP`): the last-built houses go, and those left stand as
    /// they were, not as the smaller count would have built them.
    #[test]
    fn the_houses_left_when_a_cell_loses_people_keep_their_kind_and_height() {
        let mut r = made_up(true);
        // in the last year every cell of the town loses two in five of its people
        let (n, last) = (r.n, r.ny - 1);
        for c in 0..n {
            let (was, now) = ((last - 1) * n + c, last * n + c);
            r.res[now] = (r.res[was] as f32 * 0.6).round() as u8;
        }
        let g = world::build_ground(&r);
        let at = |k: usize| -> std::collections::BTreeMap<((i32, u8), i32), What> {
            let l = laid(&r, &g, k, &|_, _| false);
            l.standing.iter().zip(&l.whats).map(|(s, w)| ((s.lot.pl.run, s.lot.pl.index), *w)).collect()
        };
        let (was, now) = (at(last - 1), at(last));
        let (mut kept, mut tall, mut lone) = (0, 0, 0);
        for (plot, b) in &now {
            let Some(a) = was.get(plot) else { continue };
            kept += 1;
            tall += (a.kind() == 1 || a.storeys() >= 2) as usize;
            // (the one change left: a townhouse is a townhouse only where a neighbour stands,
            // and that is read from this year's houses, so one whose neighbours were the
            // last-built and went is a cottage again. Two of the made-up town's do so; reading
            // the cell's count from this year, as before the second pass, made 92 houses change)
            if matches!((a, b), (What::Townhouse { .. }, What::Cottage { .. })) {
                lone += 1;
                continue;
            }
            assert!(a == b, "plot {plot:?}: {a:?}, then {b:?} when its cell lost people");
        }
        // the fall took houses, and left many, the grown ones among them
        assert!(now.len() < was.len() * 9 / 10 && kept > 60 && tall > 20, "{} houses, then {}: {kept} kept, {tall} of them grown", was.len(), now.len());
        assert!(lone <= 2, "{lone} townhouses that lost their neighbours are cottages again");
    }

    #[test]
    fn no_back_yard_runs_into_a_works_yard_a_lane_or_the_water() {
        let r = made_up(true);
        let g = world::build_ground(&r);
        let cen = crate::network::centres(&r);
        let node = Vec2::new(cen[5 * r.w + 5].0, cen[5 * r.w + 5].1);
        // a works' yard across a strip of the town's middle, and a pond beside it, as they
        // are drawn; the layout is told of them with the room scatter.rs keeps about a yard
        // (0.02), as it is told of the water
        let held = |x: f32, z: f32, room: f32| (x - node.x - 0.2).abs() < 0.12 + room && (z - node.y).abs() < 0.3 + room || Vec2::new(x, z).distance(node + Vec2::new(-0.25, 0.2)) < 0.1 + room;
        let yard = |x: f32, z: f32| held(x, z, 0.0);
        for k in [0, 3, 5] {
            let l = laid(&r, &g, k, &|x, z| held(x, z, 0.02));
            assert!(l.standing.len() > 60, "year {k}: {} houses", l.standing.len());
            for s in &l.standing {
                let pl = &s.lot.pl;
                // from the middle of its plot back to the end of its yard or garden
                let back = pl.p - Vec2::new(pl.yaw.sin(), pl.yaw.cos()) * (pl.depth * 0.5 + houses::BACK_ROOM);
                assert!(!yard(pl.p.x, pl.p.y) && !yard(back.x, back.y), "year {k}: the house on plot {:?} at {}, or the end of its garden at {back}, is in a works' yard or the water", (pl.run, pl.index), pl.p);
                // and neither is under water (the made-up island's ground is dry where its town is)
                assert!(world::height_at(&g, pl.p.x, pl.p.y) >= 0.05 && world::height_at(&g, back.x, back.y) >= 0.05, "year {k}: plot {:?}", (pl.run, pl.index));
                // nor does a lane that is drawn run through the house or across the end of its
                // garden, unless the lane is younger than the house and was laid round it
                for (line, width) in &l.lanes {
                    for w in line.windows(2) {
                        assert!(off_line(pl.p, w[0], w[1]) >= width * 0.5 + 0.012, "year {k}: a lane runs through the house on plot {:?}", (pl.run, pl.index));
                        assert!(off_line(back, w[0], w[1]) >= width * 0.5, "year {k}: a lane runs across the end of the garden of plot {:?}", (pl.run, pl.index));
                    }
                }
            }
        }
    }

    #[test]
    fn the_houses_round_a_market_place_know_their_neighbour_across_the_ring_s_seam() {
        let r = made_up(true);
        let g = world::build_ground(&r);
        let mut seams = 0;
        for k in 0..r.ny {
            let mut standing = laid(&r, &g, k, &|_, _| false).standing;
            // the plots round the market place are numbered from the east, where the made-up
            // town's road comes in and no house stands: so that the seam is tried, turn the
            // numbers round the ring by every count in turn (which houses stand side by side
            // is the same however the ring is numbered)
            for turn in 0..RING_PLOTS {
                for s in standing.iter_mut().filter(|s| s.lot.kind == 9) {
                    s.lot.pl.index = (s.lot.pl.index + 1).rem_euclid(RING_PLOTS);
                    s.neighbours = (false, false);
                }
                find_neighbours(&mut standing, None);
                let ring: Vec<&Stood> = standing.iter().filter(|s| s.lot.kind == 9).collect();
                let stands = |i: i32| ring.iter().any(|s| s.lot.pl.index == i.rem_euclid(RING_PLOTS));
                assert!(ring.len() >= 4, "year {k}: {} houses round the market place", ring.len());
                for s in &ring {
                    // the plot before it round the ring and the plot after, the last plot's being the first
                    let i = s.lot.pl.index;
                    assert_eq!(s.neighbours, (stands(i - 1), stands(i + 1)), "year {k}, turned {turn}: the house on plot {i} of {RING_PLOTS} round the market place");
                    seams += ((i == 0 && stands(-1)) || (i == RING_PLOTS - 1 && stands(0))) as usize;
                }
            }
        }
        assert!(seams >= 2 * r.ny, "{seams} houses stood beside the ring's seam");
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
            let census = houses_kept(&r, &g, &cen, k, &lk, &[], &|_, _| false, None, &land, &mut Into { houses: &mut houses, smoke: &mut smoke, trees: &mut trees, lod: &lod });
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
