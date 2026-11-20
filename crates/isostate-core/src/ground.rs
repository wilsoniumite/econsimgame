//! GROUND: house rent that clears the room there is, and how far a town may
//! reach, from the end of 1900 (not in the Python; see `docs/rules/ground.md`).
//!
//! Under the towns' rule (`towns.rs`) a town's house rent is a fixed share of
//! the wage a head (`urban_rent_rate`), and a town takes its best free
//! neighbour whenever its people outgrow its room, while the room of every
//! cell rises with the calendar. So room is never scarce and rent cannot rise
//! as a share of what people earn. From the end of `from` (1900) two things
//! change, each with a switch of its own:
//!
//! - **Room** (`room`). A town's room is counted once, by what is built: each
//!   cell's room with the calendar's ceiling held at its value in `from`,
//!   times the cell's `dens`, which starts at 1 and moves `dens_rate` a year
//!   toward the square root of the cell's rent over the town's rent in `from`
//!   (both over the long run's anchor), kept between 1 and `floors`, or up to
//!   `floors_frame` where the town knows the steel-frame builders. The
//!   townsfolk, one kind of household, each want at least `room_min` of a
//!   unit of room and spend `beta` of what is left of their budget after the
//!   subsistence bundle on more, so the rent of a unit of room has a closed
//!   form, which the month's rent moves a third of the way to. What is
//!   charged moves evenly from the old rule to this one over `blend_years`.
//!   Each head pays the rent of the room it takes, never more than half its
//!   budget.
//! - **Reach** (`reach`). A cell may join a town only within `reach_walk`
//!   cells of its centre in a straight line, or `reach_tram` where the
//!   tramway is known and the town has `tram_people` townsfolk; `belt`, when
//!   set, is the most cells a town may add in a year.
//!
//! The tramway is found here, on the island as a whole, as office computing
//! is found (`docs/rules/computing.md`): not before `tram_after`, and then in
//! the first year the island has made `tram_floor` electrical goods a head
//! since 1750.
//!
//! Nothing acts before the year after `from`: the yearly step of `from` only
//! takes the town's rent and room as they stand, and the months of the next
//! year are the first to pay the new rule's rent. So the played century is
//! untouched by construction. Switched off, every function returns at once
//! and `State::ground` stays empty. The facts read the books and the record,
//! so they are given with the rule off too.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::facts::Kind;
use crate::mat::Mat;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;

/// The numbers of room and reach.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GroundParams {
    /// Room that clears and reach, from the end of `from`.
    pub enabled: bool,
    /// The last year of the old rule: the rule takes over at its end.
    pub from: i64,
    /// Years over which what is charged moves from the old rule to the new.
    pub blend_years: i64,
    /// The room that clears: room counted by what is built, and its rent.
    pub room: bool,
    /// How far a town may reach from its centre.
    pub reach: bool,
    /// The least room a head wants, in units of room (a unit is the room of
    /// one person at the ceiling of `from`).
    pub room_min: f64,
    /// The share of a budget, after the subsistence bundle and the least room,
    /// spent on more room.
    pub beta: f64,
    /// How fast `dens` moves toward its aim: this share of itself a year.
    pub dens_rate: f64,
    /// The most `dens` may reach...
    pub floors: f64,
    /// ...and where the whole town knows the steel-frame builders.
    pub floors_frame: f64,
    /// The walking reach, in cells from the centre in a straight line.
    pub reach_walk: u32,
    /// The tram's reach.
    pub reach_tram: u32,
    /// Townsfolk a town needs before it runs trams.
    pub tram_people: f64,
    /// The most cells a town may add in a year (unset: no limit).
    pub belt: Option<u32>,
    /// The tramway is not found before this year...
    pub tram_after: i64,
    /// ...nor before the island has made this many electrical goods a head.
    pub tram_floor: f64,
}

impl Default for GroundParams {
    /// Off: what the port runs with. The numbers are the game's.
    fn default() -> GroundParams {
        GroundParams {
            enabled: false,
            from: 1900,
            blend_years: 20,
            room: true,
            reach: true,
            room_min: 0.3,
            beta: 0.09,
            dens_rate: 0.02,
            floors: 1.5,
            floors_frame: 3.0,
            reach_walk: 5,
            reach_tram: 8,
            tram_people: 30_000.0,
            belt: None,
            tram_after: 1901,
            // the office machines' floor (to2050-7b): the electric drive's
            // threshold, 15,000 made, over island 7's people in 1893
            tram_floor: 0.02,
        }
    }
}

impl GroundParams {
    /// The game's ground: room and reach, with no belt.
    pub fn game() -> GroundParams {
        GroundParams { enabled: true, ..GroundParams::default() }
    }
}

/// One year of the rule, for the facts and the tests.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct GroundYear {
    pub year: i64,
    /// House rent charged over the townsfolk's budget: the island's, and the
    /// largest town's (by townsfolk), over the year.
    pub burden: f64,
    pub burden_largest: f64,
    /// The highest of any town in any month of the year.
    pub burden_worst: f64,
    pub largest: usize,
    /// `dens` in the largest town at the year's end: the mean of its cells and the highest.
    pub dens_largest: f64,
    pub dens_largest_max: f64,
    /// The largest town's rent a unit of room over its rent in `from`, both over the anchor.
    pub rent_largest: f64,
    /// Cells of town ground on the island at the year's end, and how many joined a town in the year.
    pub town_cells: usize,
    pub joined: usize,
    /// The most cells any one town added in the year, and the farthest a cell
    /// that joined lies from its town's centre (in cells; 0 if none joined).
    pub joined_most: usize,
    pub joined_farthest: f64,
}

/// `State::ground`: empty while the rule is off and until the end of `from`.
#[derive(Clone, Debug, Default)]
pub struct GroundState {
    /// The rule has taken over (set at the end of `from`).
    pub on: bool,
    /// The year whose months are running.
    pub year_now: i64,
    /// Each cell's `dens` (1 where nothing has been built up).
    pub dens: Vec<f64>,
    /// Each town's room, in units of room: what its months' rent clears.
    pub room: Vec<f64>,
    /// Each town's rent a unit of room, as the month ended.
    pub rent: Vec<f64>,
    /// Each town's rent a unit of room in `from` (or the year it was first
    /// seen, for a town founded after), and the anchor then.
    pub rent0: Vec<f64>,
    pub anchor0: Vec<f64>,
    /// The year the tramway was found.
    pub tram_year: Option<i64>,
    /// Every complete year from the year after `from`.
    pub years: Vec<GroundYear>,
    // the year so far, by town: house rent charged, the townsfolk's budget
    paid: Vec<f64>,
    budget: Vec<f64>,
    worst: f64,
    // this year's joining, by town: cells added, the farthest
    added: Vec<usize>,
    farthest: f64,
}

/// Whether the room that clears acts in the months now running.
fn room_acts(st: &State, P: &Params) -> bool {
    P.ground.enabled && P.ground.room && st.ground.on
}

/// Whether the yearly step of `year` builds by the rule (counts room by what
/// is built, holds a town to its reach): only after `from`.
pub fn acts_in(st: &State, P: &Params, year: i64) -> bool {
    P.ground.enabled && st.ground.on && year > P.ground.from
}

/// The year the calendar's ceiling is read at for the room of a cell in the
/// yearly step of `year`: `from` once the room rule builds, else `year`.
pub fn ceiling_year(st: &State, P: &Params, year: i64) -> i64 {
    if acts_in(st, P, year) && P.ground.room { P.ground.from } else { year }
}

/// What a cell's room is multiplied by: its `dens` once the room rule builds.
pub fn dens_of(st: &State, P: &Params, year: i64, c: usize) -> f64 {
    if acts_in(st, P, year) && P.ground.room { st.ground.dens.get(c).copied().unwrap_or(1.0) } else { 1.0 }
}

/// A cell's distance from town `i`'s centre in cells, in a straight line
/// (as the designers' count of reach has it: across water too).
pub fn from_centre(st: &State, i: usize, c: usize) -> f64 {
    let W = st.world.W;
    let n = st.town_node[i];
    let (dy, dx) = ((c / W) as f64 - (n / W) as f64, (c % W) as f64 - (n % W) as f64);
    dy.hypot(dx)
}

/// The reach of town `i` in the yearly step of `year`, in cells; None where
/// reach does not act.
pub fn reach_of(st: &State, P: &Params, year: i64, i: usize) -> Option<f64> {
    if !(acts_in(st, P, year) && P.ground.reach) {
        return None;
    }
    let gp = &P.ground;
    let tram = st.ground.tram_year.is_some_and(|y| y <= year) && st.urban_pop[i] >= gp.tram_people;
    Some(if tram { gp.reach_tram } else { gp.reach_walk } as f64)
}

/// The most cells a town may add in the yearly step of `year`.
pub fn belt(st: &State, P: &Params, year: i64) -> Option<usize> {
    if acts_in(st, P, year) && P.ground.reach { P.ground.belt.map(|b| b as usize) } else { None }
}

/// Town `i` took `added` cells in the yearly step, the farthest `farthest`
/// cells from its centre (for the year's record).
pub fn joined(st: &mut State, P: &Params, year: i64, i: usize, added: usize, farthest: f64) {
    if !acts_in(st, P, year) || added == 0 {
        return;
    }
    let T = st.T_max;
    let g = &mut st.ground;
    if g.added.len() != T {
        g.added = vec![0; T];
    }
    g.added[i] += added;
    g.farthest = g.farthest.max(farthest);
}

/// The steel-frame builders' place among the methods, if the content has them.
fn frame_method(C: &Content) -> Option<usize> {
    C.method_name.iter().position(|n| n == "steel_frame")
}

// ---------------------------------------------------------------- the month

/// `econ::step_month`, before the households spend: the house rent each town's
/// households pay this month, given their budgets and the price of a unit of
/// each need (`unit`). None while the room rule does not act, when the port's
/// charge (the towns' bill, never more than half the budget) stands.
pub fn month(st: &mut State, C: &Content, P: &Params, budget: &[f64], unit: &Mat) -> Option<Vec<f64>> {
    if !room_acts(st, P) {
        return None;
    }
    let gp = &P.ground;
    let T = st.T_max;
    let w = ((st.ground.year_now - gp.from) as f64 / gp.blend_years.max(1) as f64).clamp(0.0, 1.0);
    let g = &mut st.ground;
    if g.paid.len() != T {
        g.paid = vec![0.0; T];
        g.budget = vec![0.0; T];
    }
    let mut pay = vec![0.0; T];
    for t in 0..T {
        let old = st.urban_rent[t].min(0.5 * budget[t]);
        let n = st.urban_pop[t].max(0.0);
        let S = g.room.get(t).copied().unwrap_or(0.0);
        if !st.town_active[t] || st.pop[t] <= 0.0 || n <= 0.0 || S <= 0.0 || !g.rent[t].is_finite() {
            pay[t] = old;
            continue;
        }
        // a head's budget, and the cost of its subsistence bundle at the town's prices
        let b = budget[t] / st.pop[t];
        let sub: f64 = (0..C.subsistence.len()).map(|s| C.subsistence[s] * unit[(t, s)]).sum();
        let spare = (b - sub).max(0.0);
        let cap = 0.5 * n * b;
        let denom = S - gp.room_min * n * (1.0 - gp.beta);
        // the rent a unit of room that clears the room there is; where the
        // townsfolk's least room fills it all, the rent at which they pay half
        // their budget
        let aim = if denom > 0.0 {
            n * gp.beta * spare / denom
        } else {
            ((0.5 * b - gp.beta * spare) / (gp.room_min * (1.0 - gp.beta)).max(1e-12)).max(0.0)
        };
        g.rent[t] += (aim - g.rent[t]) / 3.0;
        // each head takes room_min (1 - beta) + beta spare / rent, and pays the rent of it
        let bill = (n * (g.rent[t] * gp.room_min * (1.0 - gp.beta) + gp.beta * spare)).min(cap);
        let charged = ((1.0 - w) * old + w * bill).min(cap).min(0.5 * budget[t]);
        pay[t] = charged;
        g.paid[t] += charged;
        g.budget[t] += n * b;
        if n * b > 0.0 {
            g.worst = g.worst.max(charged / (n * b));
        }
    }
    Some(pay)
}

// ---------------------------------------------------------------- the year

/// `towns::update_urban`, before the towns grow: once the rule has taken
/// over, the tramway may be found, and each town cell's `dens` moves toward
/// its aim on the rent of the year that has just run.
pub fn year_start(st: &mut State, C: &Content, P: &Params, year: i64) {
    if !acts_in(st, P, year) {
        return;
    }
    let gp = &P.ground;
    let (N, T) = (st.N, st.T_max);
    // the tramway: island-wide, from its year, once enough electrical goods have been made
    if st.ground.tram_year.is_none() && year >= gp.tram_after {
        if let Some(&e) = C.good_index.get("electricals") {
            let pop: f64 = (0..T).filter(|&t| st.town_active[t]).map(|t| st.pop[t]).sum();
            if pop > 0.0 && st.cumprod[e] / pop >= gp.tram_floor {
                st.ground.tram_year = Some(year);
            }
        }
    }
    if !gp.room {
        return;
    }
    if st.ground.dens.len() != N {
        st.ground.dens = vec![1.0; N];
    }
    let anchor = crate::longrun::anchor(st, P);
    let frame = frame_method(C);
    for i in 0..T {
        if !st.town_active[i] || st.town_ground[i].is_empty() {
            continue;
        }
        let r0 = st.ground.rent0[i] / st.ground.anchor0[i];
        if !(r0 > 0.0) {
            continue;
        }
        let list = &st.town_ground[i];
        // the cell's rent a unit of room: the town's, shared out by 1 - distance / fringe
        let dmax = list.iter().map(|&c| st.dist[(i, c)]).fold(0.0, f64::max);
        let fringe = dmax + 1.0;
        let fall = |c: usize| (1.0 - st.dist[(i, c)] / fringe).max(0.0);
        let room: f64 = list.iter().map(|&c| st.town_room[c]).sum();
        let mean_fall = list.iter().map(|&c| st.town_room[c] * fall(c)).sum::<f64>() / room.max(1e-9);
        if !(mean_fall > 0.0) {
            continue;
        }
        let know = frame.map_or(0.0, |m| st.know[(i, m)]);
        let top = gp.floors + (gp.floors_frame - gp.floors).max(0.0) * know;
        let r = st.ground.rent[i] / anchor;
        for &c in list {
            let cell = r * fall(c) / mean_fall;
            let aim = (cell / r0).max(0.0).sqrt().clamp(1.0, top.max(1.0));
            let d = st.ground.dens[c];
            let step = gp.dens_rate * d;
            st.ground.dens[c] = if aim > d { (d + step).min(aim) } else { (d - step).max(aim) };
        }
    }
}

/// Put away the year whose months have just run.
fn close_year(st: &mut State, P: &Params, year: i64) {
    let anchor = crate::longrun::anchor(st, P);
    let T = st.T_max;
    let g = &st.ground;
    if g.paid.len() != T {
        return;
    }
    let largest = (0..T).filter(|&t| st.town_active[t]).max_by(|&a, &b| st.urban_pop[a].total_cmp(&st.urban_pop[b])).unwrap_or(0);
    let ratio = |a: f64, b: f64| if b > 0.0 { a / b } else { f64::NAN };
    let cells = &st.town_ground[largest];
    let dens: Vec<f64> = cells.iter().map(|&c| g.dens.get(c).copied().unwrap_or(1.0)).collect();
    let y = GroundYear {
        year,
        burden: ratio(g.paid.iter().sum(), g.budget.iter().sum()),
        burden_largest: ratio(g.paid[largest], g.budget[largest]),
        burden_worst: g.worst,
        largest,
        dens_largest: ratio(dens.iter().sum(), dens.len() as f64),
        dens_largest_max: dens.iter().cloned().fold(f64::NAN, f64::max),
        rent_largest: ratio(g.rent[largest] / anchor, g.rent0[largest] / g.anchor0[largest]),
        town_cells: (0..T).map(|t| st.town_ground[t].len()).sum(),
        joined: g.added.iter().sum(),
        joined_most: g.added.iter().copied().max().unwrap_or(0),
        joined_farthest: g.farthest,
    };
    let g = &mut st.ground;
    g.years.push(y);
    g.paid.iter_mut().for_each(|v| *v = 0.0);
    g.budget.iter_mut().for_each(|v| *v = 0.0);
    g.worst = 0.0;
    g.added.iter_mut().for_each(|v| *v = 0);
    g.farthest = 0.0;
}

/// `towns::update_urban`, once the towns have grown and their bills are set:
/// at the end of `from` the rule takes over, from the town's rent and room as
/// they stand; after it, the year that has just run is put away, each town's
/// room is what it has built, and a town
/// first seen takes its rent now as its own of `from`. A cell no town holds
/// loses what was built up on it.
pub fn year_end(st: &mut State, P: &Params, year: i64) {
    if !P.ground.enabled || year < P.ground.from {
        return;
    }
    let (N, T) = (st.N, st.T_max);
    let anchor = crate::longrun::anchor(st, P);
    if !st.ground.on {
        st.ground = GroundState {
            on: true,
            dens: vec![1.0; N],
            room: vec![0.0; T],
            rent: vec![f64::NAN; T],
            rent0: vec![f64::NAN; T],
            anchor0: vec![f64::NAN; T],
            paid: vec![0.0; T],
            budget: vec![0.0; T],
            added: vec![0; T],
            ..GroundState::default()
        };
    }
    st.ground.year_now = year + 1;
    if acts_in(st, P, year) {
        close_year(st, P, year);
    }
    for c in 0..N {
        if !st.urban[c] {
            st.ground.dens[c] = 1.0;
        }
    }
    for i in 0..T {
        let room: f64 = st.town_ground[i].iter().map(|&c| st.town_room[c]).sum();
        st.ground.room[i] = if st.town_active[i] { room } else { 0.0 };
        if st.town_active[i] && room > 0.0 && !st.ground.rent0[i].is_finite() {
            let r = st.urban_rent[i] / room;
            st.ground.rent0[i] = r;
            st.ground.anchor0[i] = anchor;
            st.ground.rent[i] = r;
        }
    }
}

// ---------------------------------------------------------------- the facts

/// The years the facts are given at: 1880 and 1890, then the marks from 1900.
fn fact_years(y0: i64, y1: i64) -> Vec<i64> {
    let mut v: Vec<i64> = [1880, 1890].into_iter().filter(|&y| y >= y0 && y <= y1).collect();
    v.extend(crate::facts::marks(y0, y1).into_iter().filter(|&y| y >= crate::facts::DECADES_FROM));
    v
}

/// The facts (`docs/rules/ground.md`, "Facts"). The burden, the town cells,
/// the reach of 1900's ground and the fork's room are read from the books,
/// the record and the gauges, so a run with the rule off has them too;
/// `dens` and the tramway are the rule's own.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C, P, rec) = (&r.state, &r.content, &r.params, &r.rec);
    let mut out = Vec::new();
    if !P.towns.enabled || rec.town_layers.is_empty() {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + rec.n_years as i64 - 1;
    let years = fact_years(y0, y1);
    let (N, T) = (rec.N, rec.T);
    let tcol = |name: &str| crate::accounts::TOWN_FIELDS.iter().position(|&n| n == name).expect("a town column");
    let ratio = |a: f64, b: f64| if b > 0.0 { a / b } else { f64::NAN };
    // the burden: house rent over the townsfolk's budget (the households' spending
    // and rent, times the townsfolk's share of the town's people)
    if st.acct.on {
        let (c_rent, c_spend) = (tcol("rent_town"), tcol("spend_households"));
        for &y in &years {
            let k = (y - y0) as usize;
            let (mut rent, mut budget) = (0.0, 0.0);
            let mut largest: Option<(f64, f64, f64)> = None;
            if let Some(b) = st.acct.year((y - st.acct.start_year).max(0) as usize).filter(|_| y >= st.acct.start_year) {
                for t in 0..T {
                    if rec.town_active[k * T + t] == 0 {
                        continue;
                    }
                    let (u, p) = (rec.town_urban[k * T + t] as f64, rec.town_pop[k * T + t] as f64);
                    let rt = b.towns[t * crate::accounts::KT + c_rent] as f64;
                    let bt = (b.towns[t * crate::accounts::KT + c_spend] as f64 + rt) * ratio(u, p).min(1.0);
                    if !bt.is_finite() {
                        continue;
                    }
                    rent += rt;
                    budget += bt;
                    if largest.is_none_or(|l| u > l.0) {
                        largest = Some((u, rt, bt));
                    }
                }
            }
            out.push((format!("rent_burden_{y}"), Kind::Share, ratio(rent, budget)));
            out.push((format!("rent_burden_largest_{y}"), Kind::Share, largest.map_or(f64::NAN, |l| ratio(l.1, l.2))));
        }
    }
    // town ground as the record keeps it (built, in its market's town)
    let Some(built) = rec.town_layer("built") else { return out };
    let catch = rec.layer("catch");
    let W = st.world.W;
    let town_cells = |k: usize| (0..N).filter(|&c| built[k * N + c] > 0 && catch[k * N + c] > 0).count();
    for &y in &years {
        out.push((format!("town_cells_{y}"), Kind::Count, town_cells((y - y0) as usize) as f64));
    }
    // 1900's town ground beyond the walking reach and the tram's, counted as the
    // designers counted it: in a straight line from the town's meeting place
    let turn = P.ground.from;
    if turn >= y0 && turn <= y1 {
        let k = (turn - y0) as usize;
        let (mut all, mut walk, mut tram) = (0usize, 0usize, 0usize);
        for c in 0..N {
            let m = catch[k * N + c] as usize;
            if built[k * N + c] == 0 || m == 0 || m > T {
                continue;
            }
            let n = st.town_node[m - 1];
            let d = ((c / W) as f64 - (n / W) as f64).hypot((c % W) as f64 - (n % W) as f64);
            all += 1;
            walk += (d > P.ground.reach_walk as f64) as usize;
            tram += (d > P.ground.reach_tram as f64) as usize;
        }
        out.push((format!("cells_beyond_reach_{turn}"), Kind::Count, walk as f64));
        out.push((format!("cells_beyond_reach_share_{turn}"), Kind::Share, ratio(walk as f64, all as f64)));
        out.push((format!("cells_beyond_tram_{turn}"), Kind::Count, tram as f64));
    }
    // the fork: what a wage buys of all goods and of room (1750 = 1)
    if st.gauges.on {
        for &y in &years {
            let g = st.gauges.years.iter().find(|g| g.year == y);
            out.push((format!("wage_in_goods_{y}"), Kind::Ratio, g.map_or(f64::NAN, |g| g.fork_all)));
            out.push((format!("wage_in_room_{y}"), Kind::Ratio, crate::gauges::fork_room_of(st, y)));
        }
    }
    // the rule's own
    if P.ground.enabled {
        for &y in &years {
            let g = st.ground.years.iter().find(|g| g.year == y);
            out.push((format!("dens_largest_{y}"), Kind::Ratio, g.map_or(f64::NAN, |g| g.dens_largest)));
            out.push((format!("dens_largest_max_{y}"), Kind::Ratio, g.map_or(f64::NAN, |g| g.dens_largest_max)));
            out.push((format!("rent_room_largest_{y}"), Kind::Ratio, g.map_or(f64::NAN, |g| g.rent_largest)));
        }
        out.push(("found_year_tramway".into(), Kind::Year, st.ground.tram_year.map_or(f64::NAN, |y| y as f64)));
        out.push(("ground_burden_worst".into(), Kind::Share, st.ground.years.iter().map(|g| g.burden_worst).fold(0.0, f64::max)));
    }
    out
}
