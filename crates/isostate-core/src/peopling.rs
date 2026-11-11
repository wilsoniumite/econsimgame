//! The game's rules for people (`P.people`; docs/rules/people.md; not in the Python):
//! a population that answers to how people live, towns founded all century, and
//! households that go with their cells when a market's area changes.
//!
//! The port's rules are in `people.rs` and stay as they are. With the switch off
//! nothing here runs. With it on:
//!
//! - `growth` sets each town's yearly rate in place of the capped rule;
//! - `limit_arrivals` keeps a town from taking in more migrants than it can house;
//! - `found_towns` replaces the port's founding rule, and is called once the year's
//!   roads are known, so that the new town's area can be counted exactly;
//! - `move_with_cells` replaces `people::resettle_new_towns`, every year, and what
//!   the district was used to (its wage, the prices of its produce) goes with it.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::land;
use crate::mat::Mat;
use crate::names::town_name;
use crate::ndimage::{self, Mode};
use crate::params::Params;
use crate::rng;
use crate::state::{Event, State};
use crate::transport as tr;
use crate::util::{argmax, argsort, bool_f, fmt_thousands_f0, pairwise_sum};
use crate::world::site_context;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PeopleParams {
    /// The game's population rule, founding rule and resettling are in force.
    pub enabled: bool,

    // ---- natural increase
    /// Births less deaths a year where income is high and nobody is crowded: the
    /// ceiling the rate approaches and never reaches.
    pub growth_top: f64,
    /// Income over the cost of subsistence at which births just balance deaths.
    pub y_still: f64,
    /// How quickly the rate nears its ceiling as income rises: the gap to the
    /// ceiling shrinks as `(y_still / y)^ease`.
    pub ease: f64,
    /// Extra deaths a year per head of those living on town ground in a great town.
    pub town_deaths: f64,
    /// Townsfolk at which a town is half way to being a great one.
    pub town_size: f64,
    /// Dearth begins when the worst month's food falls below this share of need...
    pub dearth_from: f64,
    /// ...and is complete at this share.
    pub dearth_full: f64,
    /// Deaths a year, per head, in a complete dearth.
    pub dearth: f64,
    /// Share of a town's people who leave the island in a complete dearth, if a
    /// port can be reached. Their cash is spent at that port on the passage.
    pub emigration: f64,
    /// Births less deaths never fall below this (those who sail are counted besides).
    pub growth_floor: f64,

    // ---- migration
    /// Migrants a town can take in a year, as a share of its own people.
    pub arrivals_max: f64,

    // ---- new towns
    /// A new market is founded where the carting it would save in a year is at
    /// least this share of the year's wages earned in the district it would serve.
    pub found_saving: f64,
    /// The district must hold at least this many cells...
    pub found_cells: usize,
    /// ...and this many people.
    pub found_people: f64,
    /// Least distance in cells from the new town to any other.
    pub found_spacing: f64,
    /// No town gives up more than this share of its people to a founding.
    pub donor_max: f64,
    /// Years a town that gave people to a founding is left alone.
    pub donor_rest: i64,
}

impl Default for PeopleParams {
    /// Off: what the port runs with.
    fn default() -> PeopleParams {
        PeopleParams {
            enabled: false,
            growth_top: 0.0165,
            y_still: 1.5,
            ease: 2.1,
            town_deaths: 0.012,
            town_size: 10_000.0,
            dearth_from: 1.0,
            dearth_full: 0.6,
            dearth: 0.03,
            emigration: 0.02,
            growth_floor: -0.06,
            arrivals_max: 0.1,
            found_saving: 0.08,
            found_cells: 40,
            found_people: 3000.0,
            found_spacing: 8.0,
            donor_max: 0.2,
            donor_rest: 8,
        }
    }
}

impl PeopleParams {
    /// The game's people.
    pub fn game() -> PeopleParams {
        PeopleParams { enabled: true, ..PeopleParams::default() }
    }
}

/// A town founded under these rules, and why.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Founding {
    pub town: usize,
    pub year: i64,
    /// What most of the district's hands do: "coal", "iron", "stone", "works" or "farm".
    pub kind: String,
    /// The site is on the coast, so the town is a port.
    pub port: bool,
    /// A railway ran within a cell of the site when the town was founded.
    pub rail: bool,
    /// Cells and people of the district on the day.
    pub cells: usize,
    pub people: f64,
    /// Carting saved in a year over the year's wages of the district.
    pub saving: f64,
}

// ---------------------------------------------------------------- natural increase

/// Births less deaths from income alone: zero at `y_still`, rising toward `growth_top`.
pub fn income_term(pp: &PeopleParams, y: f64) -> f64 {
    pp.growth_top * (1.0 - (pp.y_still / y.max(1e-3)).powf(pp.ease))
}

/// What the town itself costs in lives: the share of people on town ground, times
/// how great the town is.
pub fn crowding_term(pp: &PeopleParams, townsfolk: f64, people: f64) -> f64 {
    if people <= 0.0 || townsfolk <= 0.0 {
        return 0.0;
    }
    pp.town_deaths * (townsfolk / people).min(1.0) * townsfolk / (townsfolk + pp.town_size)
}

/// How complete the dearth was, 0 to 1, from the worst month's food over need.
pub fn dearth_depth(pp: &PeopleParams, food_min: f64) -> f64 {
    ((pp.dearth_from - food_min) / (pp.dearth_from - pp.dearth_full).max(1e-9)).clamp(0.0, 1.0)
}

/// The least a town's rate of increase can be in one year, deaths and sailings
/// together, however the numbers are set. The game's numbers stop at -0.08.
const WORST_YEAR: f64 = -0.5;

/// The year's rate of increase for every town, in place of the port's capped rule.
/// Emigrants' cash is moved here; the caller applies the rate to `st.pop`.
pub fn growth(st: &mut State, P: &Params, y: &[f64], year: i64, events: &mut Vec<Event>) -> Vec<f64> {
    let pp = &P.people;
    let T = st.T_max;
    let mut g = vec![0.0; T];
    // the port a town's emigrants sail from: the nearest town on the coast
    let ports: Vec<usize> = (0..T).filter(|&t| st.town_active[t] && st.world.coast[st.town_node[t]]).collect();
    st.pp_income_term = vec![0.0; T];
    // who sailed this year: from each town, and through each port
    let (mut left, mut sailed) = (vec![0.0; T], vec![0.0; T]);
    for t in 0..T {
        if !st.town_active[t] {
            continue;
        }
        let inc = income_term(pp, y[t]);
        let depth = dearth_depth(pp, st.food_min_acc[t]);
        // whatever the numbers are set to, no town loses more than half its people in a
        // year (a town with nobody in it is not something the markets can price)
        let mut rate = (inc - crowding_term(pp, st.urban_pop[t], st.pop[t]) - pp.dearth * depth).max(pp.growth_floor).max(WORST_YEAR);
        if depth > 0.0 && pp.emigration > 0.0 {
            let port = ports.iter().copied().filter(|&j| st.town_cost[(t, j)].is_finite()).min_by(|&a, &b| st.town_cost[(t, a)].total_cmp(&st.town_cost[(t, b)]));
            if let Some(j) = port {
                let share = (pp.emigration * depth).min(rate - WORST_YEAR);
                // the passage: what they had goes to the port's merchants
                let fare = st.hh_cash[t].max(0.0) * share;
                st.hh_cash[t] -= fare;
                st.merch_cash[j] += fare;
                st.pp_emigrants += st.pop[t] * share;
                left[t] = st.pop[t] * share;
                sailed[j] += st.pop[t] * share;
                rate -= share;
            }
        }
        g[t] = rate;
        st.pp_income_term[t] = inc;
    }
    st.pp_growth = g.clone();
    // news when a thousandth of the island takes ship in one year
    let gone = pairwise_sum(&left);
    if gone > 0.001 * pairwise_sum(&st.pop) {
        let (from, port) = (argmax(&left), argmax(&sailed));
        let others = left.iter().filter(|&&v| v > 0.0).count() - 1;
        let whence = match others {
            0 => format!("from {}", st.town_name[from]),
            1 => format!("from {} and one other town", st.town_name[from]),
            k => format!("from {} and {k} other towns", st.town_name[from]),
        };
        let quay = match (others, from == port) {
            (0, true) => "from its own quay".to_string(),
            (0, false) => format!("at {}", st.town_name[port]),
            _ => format!("most of them at {}", st.town_name[port]),
        };
        let mut ev = Event::new(
            year as f64 + 0.6,
            "famine",
            0.5,
            format!("{} people leave the island", fmt_thousands_f0(gone)),
            format!("After a hungry year {} people {whence} take ship, {quay}. What they had goes on the passage.", fmt_thousands_f0(gone)),
        );
        ev.town = Some(port);
        ev.cell = Some(st.town_node[port]);
        events.push(ev);
    }
    g
}

// ---------------------------------------------------------------- migration

/// Scale the flows into each town so that the year's arrivals are at most
/// `arrivals_max` of its people. `flow[(i, j)]` is from `act[i]` to `act[j]`.
pub fn limit_arrivals(st: &State, P: &Params, act: &[usize], flow: &mut Mat) {
    let n = act.len();
    for j in 0..n {
        let mut arrivals = 0.0;
        for i in 0..n {
            arrivals += flow[(i, j)];
        }
        let room = P.people.arrivals_max * st.pop[act[j]];
        if arrivals > room {
            let k = room / arrivals;
            for i in 0..n {
                flow[(i, j)] *= k;
            }
        }
    }
}

// ---------------------------------------------------------------- new towns

/// Labour at work on each cell last month: land units and works together.
/// (Between the yearly land-use update and the next month the land units are new
/// and have no labour yet; nothing here is called then.)
fn work_by_cell(st: &State) -> Vec<f64> {
    let mut work = vec![0.0; st.N];
    for u in 0..st.ex_cell.len().min(st.ex_labor.len()) {
        work[st.ex_cell[u]] += st.ex_labor[u];
    }
    for f in 0..st.f_alive.len() {
        if st.f_alive[f] {
            work[st.f_cell[f]] += st.f_labor[f];
        }
    }
    work
}

/// Each town's total of a per-cell quantity over the cells of `catch`.
fn by_town(catch: &[i64], v: &[f64], T: usize) -> Vec<f64> {
    let mut tot = vec![0.0; T];
    for (c, &i) in catch.iter().enumerate() {
        if i >= 0 {
            tot[i as usize] += v[c];
        }
    }
    tot
}

/// A site that might become a town, and what the district round it looks like.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub cell: usize,
    /// Cells that would trade through a market here.
    pub cells: usize,
    pub people: f64,
    /// Carting saved in a year, in money.
    pub saved: f64,
    /// The year's wages of the district.
    pub wages: f64,
    /// The largest share any one town would give up of its people.
    pub donor_share: f64,
    /// Towns that would give up people.
    pub donors: Vec<usize>,
    pub kind: &'static str,
}

/// What the district is known for: the trade that takes a larger share of its
/// hands than it does on the island as a whole ("coal", "iron", "stone", "works"),
/// or "farm" where none stands out. Services are left out: they follow people.
fn district_kind(st: &State, C: &Content, cells: &[bool]) -> &'static str {
    const NAMES: [&str; 4] = ["coal", "iron", "stone", "works"];
    // [land, coal, iron, stone, works], in the district and on the island
    let (mut here, mut all) = ([0.0; 5], [0.0; 5]);
    for u in 0..st.ex_cell.len().min(st.ex_labor.len()) {
        let k = match C.sector_name[st.ex_sector[u]].as_str() {
            "coalmine" => 1,
            "ironmine" => 2,
            "quarry" | "claypit" => 3,
            _ => 0,
        };
        all[k] += st.ex_labor[u];
        if cells[st.ex_cell[u]] {
            here[k] += st.ex_labor[u];
        }
    }
    let services = C.gi("services");
    for f in 0..st.f_alive.len() {
        if st.f_alive[f] && C.method_out[st.f_method[f]] != services {
            all[4] += st.f_labor[f];
            if cells[st.f_cell[f]] {
                here[4] += st.f_labor[f];
            }
        }
    }
    let (h, a): (f64, f64) = (here.iter().sum(), all.iter().sum());
    let mut best = ("farm", 1.25);
    for k in 1..5 {
        let (share, usual) = (here[k] / h.max(1e-9), all[k] / a.max(1e-9));
        if share >= 0.1 && share / usual.max(1e-9) > best.1 {
            best = (NAMES[k - 1], share / usual.max(1e-9));
        }
    }
    best.0
}

/// The sites worth a look this year, best first: where the most carting would be
/// saved by a market, counted on last year's roads. Nothing is changed.
pub fn candidates(st: &State, C: &Content, P: &Params) -> Vec<Candidate> {
    let pp = &P.people;
    let (H, W, N, T) = (st.world.H, st.world.W, st.N, st.T_max);
    let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t]).collect();
    let work = work_by_cell(st);
    let work_town = by_town(&st.catch, &work, T);
    // what each cell pays to cart its year's tonnage to market
    let bill: Vec<f64> = (0..N).map(|c| if st.catch[c] >= 0 && st.dcatch[c].is_finite() { st.cell_flow_year[c] * st.dcatch[c] } else { 0.0 }).collect();
    let near: Vec<f64> = ndimage::uniform_filter(&bill, H, W, 7, Mode::Constant).iter().map(|v| v * 49.0).collect();
    let mut ok: Vec<bool> = (0..N).map(|c| st.world.land[c] && st.world.terrain[c] < 3 && st.catch[c] >= 0 && st.dcatch[c].is_finite() && st.cell_K_share[c] < 0.5).collect();
    for &i in &act {
        let (ty, tx) = ((st.town_node[i] / W) as f64, (st.town_node[i] % W) as f64);
        for c in 0..N {
            ok[c] = ok[c] && ((c / W) as f64 - ty).hypot((c % W) as f64 - tx) > pp.found_spacing;
        }
    }
    // the busiest spots, a few cells apart, each settled on the best ground nearby
    let score: Vec<f64> = (0..N).map(|c| if ok[c] { -near[c] } else { 0.0 }).collect();
    let mut peaks: Vec<usize> = Vec::new();
    for &c in argsort(&score).iter() {
        if !ok[c] || near[c] <= 0.0 || peaks.len() >= 6 {
            break;
        }
        let (y, x) = ((c / W) as f64, (c % W) as f64);
        if peaks.iter().all(|&p| ((p / W) as f64 - y).hypot((p % W) as f64 - x) > 5.0) {
            peaks.push(c);
        }
    }
    let mut out: Vec<Candidate> = Vec::new();
    for &c in &peaks {
        let (cy, cx) = ((c / W) as isize, (c % W) as isize);
        let (mut best, mut bc) = (f64::NEG_INFINITY, c);
        for dy in -2isize..=2 {
            for dx in -2isize..=2 {
                let (ny, nx) = (cy + dy, cx + dx);
                if ny >= 0 && ny < H as isize && nx >= 0 && nx < W as isize {
                    let nc = ny as usize * W + nx as usize;
                    if ok[nc] {
                        let s = near[nc] / near[c].max(1e-9) + 0.3 * bool_f(st.world.river[nc]) + 0.3 * bool_f(st.world.coast[nc])
                            - 0.2 * (st.world.terrain[nc] as f64 - 1.0);
                        if s > best {
                            best = s;
                            bc = nc;
                        }
                    }
                }
            }
        }
        if out.iter().any(|o| o.cell == bc) {
            continue;
        }
        // the district: every cell nearer to this spot than to its market
        let d = tr::distances(&st.graph, &[(bc, 0.0)]);
        let mine: Vec<bool> = (0..N).map(|k| st.catch[k] >= 0 && d[k] < st.dcatch[k]).collect();
        let (mut cells, mut saved, mut wages) = (0usize, 0.0, 0.0);
        let mut gone = vec![0.0; T];
        for k in 0..N {
            if mine[k] {
                let i = st.catch[k] as usize;
                cells += 1;
                saved += P.freight * st.cell_flow_year[k] * (st.dcatch[k] - d[k]);
                wages += 12.0 * work[k] * st.wbar[i];
                gone[i] += work[k];
            }
        }
        let share: Vec<f64> = (0..T).map(|i| if work_town[i] > 0.0 { gone[i] / work_town[i] } else { 0.0 }).collect();
        out.push(Candidate {
            cell: bc,
            cells,
            people: pairwise_sum(&(0..T).map(|i| share[i] * st.pop[i]).collect::<Vec<_>>()),
            saved,
            wages,
            donor_share: share.iter().cloned().fold(0.0, f64::max),
            donors: (0..T).filter(|&i| gone[i] > 0.0).collect(),
            kind: district_kind(st, C, &mine),
        });
    }
    out.sort_by(|a, b| b.saved.total_cmp(&a.saved));
    out
}

/// Whether a district is enough to keep a market: the safeguards every founding
/// has to pass, on the estimate and again on the real catchment.
pub fn district_ok(pp: &PeopleParams, cells: usize, people: f64, donor_share: f64) -> bool {
    cells >= pp.found_cells && people >= pp.found_people && donor_share <= pp.donor_max
}

/// What the cells trading through town `n` under `st.catch` hold, counted against
/// the areas the towns had before (`old`): cells, people, the largest share of
/// its people any one town gives up, and the towns that give people.
pub fn district_of(st: &State, old: &[i64], n: usize) -> (usize, f64, f64, Vec<usize>) {
    let T = st.T_max;
    let work = work_by_cell(st);
    let work_town = by_town(old, &work, T);
    let mut gone = vec![0.0; T];
    let mut cells = 0;
    for c in 0..st.N {
        if st.catch[c] == n as i64 {
            cells += 1;
            if old[c] >= 0 {
                gone[old[c] as usize] += work[c];
            }
        }
    }
    let share: Vec<f64> = (0..T).map(|i| if i != n && work_town[i] > 0.0 { gone[i] / work_town[i] } else { 0.0 }).collect();
    let people = pairwise_sum(&(0..T).map(|i| share[i] * st.pop[i]).collect::<Vec<_>>());
    (cells, people, share.iter().cloned().fold(0.0, f64::max), (0..T).filter(|&i| share[i] > 0.0).collect())
}

/// Make `cell` the node of a new town in the next free slot, with its donor's
/// prices, expected prices, wage and knowledge, and give it a name. No people
/// yet: they come with their cells in `move_with_cells`, which also sets the
/// prices of the district's own produce and its wage. Returns the town's index.
pub fn open_town(st: &mut State, P: &Params, cell: usize, donor: usize, year: i64) -> usize {
    let n = (0..st.T_max).filter(|&t| st.town_active[t]).count();
    let ctx = site_context(&st.world, cell);
    if st.pp_rng.is_none() {
        st.pp_rng = Some(rng::people_stream(P.seed));
    }
    let name = town_name(st.pp_rng.as_mut().unwrap(), ctx, &mut st.used_names);
    st.town_active[n] = true;
    st.town_node[n] = cell;
    st.town_name.push(name);
    st.town_founded[n] = year;
    st.town_new[n] = true;
    for g in 0..st.price.cols {
        st.price[(n, g)] = st.price[(donor, g)];
        // the port leaves the slot's expected prices as they were set in 1750, and for
        // some months a new town's land is worked for prices nobody is paying
        st.pexp[(n, g)] = st.pexp[(donor, g)];
        st.fill[(n, g)] = st.fill[(donor, g)];
    }
    st.wage[n] = st.wage[donor];
    for t in 0..st.know.cols {
        st.know[(n, t)] = st.know[(donor, t)];
    }
    st.tools_mult[n] = st.tools_mult[donor];
    st.imp_pc[n] = st.imp_pc[donor];
    n
}

/// At most one new town a year: the site where a market would save the most
/// carting, if that is worth `found_saving` of the district's wages and the
/// district passes the safeguards. Call after the year's roads are settled and
/// before `land::update_transport`.
///
/// The district is first reckoned on last year's paths. The best site that passes
/// is then tried for real: the town's node is switched on, the catchments are
/// worked out as `update_transport` will work them out, and the town stands only
/// if its real area still passes. So no town can begin with a handful of cells.
pub fn found_towns(st: &mut State, C: &Content, P: &Params, year: i64, events: &mut Vec<Event>) -> Vec<usize> {
    let pp = &P.people;
    let T = st.T_max;
    let n = (0..T).filter(|&t| st.town_active[t]).count();
    if n >= T {
        return Vec::new();
    }
    if st.pp_gave.len() != T {
        st.pp_gave = vec![-10_000; T];
    }
    // tests and scenarios: a town ordered at a given cell is founded whatever its district
    if let Some(cell) = st.pp_found_next.take() {
        if st.catch[cell] >= 0 {
            let t = open_town(st, P, cell, st.catch[cell] as usize, year);
            st.pp_gave[t] = year;
            return vec![t];
        }
    }
    let mut tried = 0;
    for cand in candidates(st, C, P) {
        if cand.saved < pp.found_saving * cand.wages
            || !district_ok(pp, cand.cells, cand.people, cand.donor_share)
            || !cand.donors.iter().all(|&i| year - st.pp_gave[i] >= pp.donor_rest)
        {
            continue;
        }
        // try it for real (at most twice a year: each try is a full set of paths)
        tried += 1;
        if tried > 2 {
            break;
        }
        let (old, old_d) = (st.catch.clone(), st.dcatch.clone());
        let donor = old[cand.cell] as usize;
        let node_before = st.town_node[n];
        st.town_active[n] = true;
        st.town_node[n] = cand.cell;
        land::update_transport(st, C, P);
        let (cells, people, donor_share, givers) = district_of(st, &old, n);
        // the caller keeps last year's areas and carting to compare against, and works the paths out again
        st.catch = old;
        st.dcatch = old_d;
        st.town_active[n] = false;
        st.town_node[n] = node_before;
        // the real area may draw on a town the estimate did not: it is owed its rest too
        if !district_ok(pp, cells, people, donor_share) || !givers.iter().all(|&i| year - st.pp_gave[i] >= pp.donor_rest) {
            continue;
        }
        let rail = {
            let (W, H) = (st.world.W as isize, st.world.H as isize);
            let (cy, cx) = ((cand.cell as isize) / W, (cand.cell as isize) % W);
            (-1..=1).any(|dy| (-1..=1).any(|dx| {
                let (y, x) = (cy + dy, cx + dx);
                y >= 0 && y < H && x >= 0 && x < W && st.rail[(y * W + x) as usize] > 0
            }))
        };
        let t = open_town(st, P, cand.cell, donor, year);
        for &i in &givers {
            st.pp_gave[i] = year;
        }
        st.pp_gave[t] = year;
        let name = st.town_name[t].clone();
        let port = st.world.coast[cand.cell];
        let what = match (cand.kind, port) {
            ("coal", _) => "a colliery town",
            ("iron", _) => "a mining town",
            ("stone", _) => "a quarry town",
            ("works", _) => "a town of works",
            (_, true) => "a harbour town",
            _ => "a market town",
        };
        let mut text = format!(
            "The carts from this district to {} cost a good part of what its {} people earn. {name} grows up among them as {what}, with a market of its own.",
            st.town_name[donor],
            fmt_thousands_f0(people)
        );
        if port && cand.kind != "farm" {
            text.push_str(" It stands on the coast, and ships call.");
        }
        if rail {
            text.push_str(" The railway passes its door.");
        }
        let mut ev = Event::new(year as f64 + 0.9, "town", 0.6, format!("{name} founded"), text);
        ev.town = Some(t);
        ev.cell = Some(cand.cell);
        events.push(ev);
        st.pp_foundings.push(Founding { town: t, year, kind: cand.kind.to_string(), port, rail, cells, people, saving: cand.saved / cand.wages.max(1e-9) });
        return vec![t];
    }
    Vec::new()
}

// ---------------------------------------------------------------- people go with their cells

/// After the catchments are worked out again: the households of every cell that
/// now trades through another town go to that town, with their cash and their
/// landlords' cash. A town founded this year gets its people this way, and with
/// them a share of the old market's stocks and its merchants' cash.
///
/// The share of a town's people on a group of cells is the share of its labour at
/// work there, so a town keeps the people whose cells it keeps.
///
/// What the district was used to goes with it: its prices (`prices_go_with_trade`),
/// the wage its people were paid and what they earned last month.
pub fn move_with_cells(st: &mut State, C: &Content, P: &Params) {
    let (N, T) = (st.N, st.T_max);
    let work = work_by_cell(st);
    let work_town = by_town(&st.prev_catch, &work, T);
    // moved[(i, j)]: labour on cells that left town i for town j
    let mut moved = Mat::zeros(T, T);
    let mut any = false;
    for c in 0..N {
        let (i, j) = (st.prev_catch[c], st.catch[c]);
        if i >= 0 && j >= 0 && i != j && work[c] > 0.0 {
            moved[(i as usize, j as usize)] += work[c];
            any = true;
        }
    }
    if any {
        // share[(i, j)]: the part of town i's people who go to town j
        let mut share = Mat::zeros(T, T);
        for i in 0..T {
            if work_town[i] > 0.0 {
                // a town always keeps its own cell, and with it some of its people
                let leaving = pairwise_sum(moved.row(i)) / work_town[i];
                let keep = if leaving > 0.95 { 0.95 / leaving } else { 1.0 };
                for j in 0..T {
                    share[(i, j)] = keep * moved[(i, j)] / work_town[i];
                }
            }
        }
        prices_go_with_trade(st, C, P, &share);
        let (pop, hh, lord, wage) = (st.pop.clone(), st.hh_cash.clone(), st.lord_cash.clone(), st.wage.clone());
        let (merch, inv, outflow) = (st.merch_cash.clone(), st.inv.clone(), st.outflow.clone());
        let (hh_earned, lord_earned) = (st.hh_income_prev.clone(), st.lord_income_prev.clone());
        // by the town they come to: the newcomers, and the month's wages they were earning
        let (mut came, mut earned) = (vec![0.0; T], vec![0.0; T]);
        for i in 0..T {
            for j in 0..T {
                let frac = share[(i, j)];
                if frac <= 0.0 {
                    continue;
                }
                let people = pop[i] * frac;
                st.pop[i] -= people;
                st.pop[j] += people;
                came[j] += people;
                earned[j] += people * wage[i];
                for (v, was) in [(&mut st.hh_cash, &hh), (&mut st.lord_cash, &lord)] {
                    let cash = was[i].max(0.0) * frac;
                    v[i] -= cash;
                    v[j] += cash;
                }
                // and what they earned last month, which is what they spend out of this
                // month (not a purse: without it a new town buys next to no bread in its
                // first month, and the year goes down as a famine)
                for (v, was) in [(&mut st.hh_income_prev, &hh_earned), (&mut st.lord_income_prev, &lord_earned)] {
                    let income = was[i] * frac;
                    v[i] -= income;
                    v[j] += income;
                }
                if st.town_new[j] {
                    // a new market starts with its share of the old one's trade
                    let cash = merch[i].max(0.0) * frac;
                    st.merch_cash[i] -= cash;
                    st.merch_cash[j] += cash;
                    for g in 0..inv.cols {
                        let stock = inv[(i, g)] * frac;
                        st.inv[(i, g)] -= stock;
                        st.inv[(j, g)] += stock;
                        let sales = outflow[(i, g)] * frac;
                        st.outflow[(i, g)] -= sales;
                        st.outflow[(j, g)] += sales;
                    }
                }
            }
        }
        // the town's wage starts the year as the average of what its old hands and its
        // new were paid (a new town has no old hands: it starts at the newcomers' wage)
        for j in 0..T {
            if came[j] > 0.0 {
                let own = if st.town_new[j] { 0.0 } else { pop[j] };
                st.wage[j] = (own * wage[j] + earned[j]) / (own + came[j]);
            }
        }
    }
    st.town_new.iter_mut().for_each(|v| *v = false);
}

/// The going price goes with the trade. A market's price for coal was made by
/// the coal it handled. When a coalfield comes over to it from another market
/// (a road was made, a town was founded) the price has to answer to the
/// newcomers too, or pits that hardly paid yesterday are offered a buyer's price
/// at the pit mouth today and want more hands than the town has: the wage in
/// Kelrigg on seed 7 went to a hundred times the island's that way in 1752.
///
/// So each market's price for each good, and the price expected there, becomes
/// the average of three, weighted by a month's trade:
///
/// - its own price, by its own turnover (a new market has none);
/// - for what the arriving land yields, the price at which its producers are no
///   better or worse off than they were (their old market's price, less what they
///   now save in carting), by the produce they bring;
/// - for what the arriving people buy, the price at which their old market can
///   still supply them (its price, plus the cart from there and the merchant's
///   handling), by what they used to ask for.
///
/// From there the market's own rule takes over. `share[(i, j)]` is the part of
/// town i's people who go to town j.
fn prices_go_with_trade(st: &mut State, C: &Content, P: &Params, share: &Mat) {
    let (T, G) = (st.T_max, C.G);
    if st.pp_dcatch_was.len() != st.N {
        return;
    }
    // per market and good: the month's trade arriving, and its worth at the price
    // it is used to and at the price expected
    let (mut trade, mut worth, mut hoped) = (Mat::zeros(T, G), Mat::zeros(T, G), Mat::zeros(T, G));
    for u in 0..st.ex_cell.len().min(st.ex_out.len()) {
        let c = st.ex_cell[u];
        let (i, j) = (st.prev_catch[c], st.catch[c]);
        if i < 0 || j < 0 || i == j || !(st.ex_out[u] > 0.0) || !st.pp_dcatch_was[c].is_finite() {
            continue;
        }
        let (i, j, g) = (i as usize, j as usize, C.sector_good[st.ex_sector[u]]);
        let saved = P.freight * C.weight[g] * (st.pp_dcatch_was[c] - st.dcatch[c]);
        trade[(j, g)] += st.ex_out[u];
        worth[(j, g)] += st.ex_out[u] * (st.price[(i, g)] - saved);
        hoped[(j, g)] += st.ex_out[u] * (st.pexp[(i, g)] - saved);
    }
    for i in 0..T {
        for j in 0..T {
            if share[(i, j)] <= 0.0 || !st.town_cost[(i, j)].is_finite() {
                continue;
            }
            for g in 0..G {
                let asked = share[(i, j)] * st.requests[(i, g)];
                // what `econ::step_month` charges a merchant to bring a unit from i to j
                let fetch = P.freight * C.weight[g] * st.town_cost[(i, j)] + P.handling * st.price[(i, g)];
                trade[(j, g)] += asked;
                worth[(j, g)] += asked * (st.price[(i, g)] + fetch);
                hoped[(j, g)] += asked * (st.pexp[(i, g)] + fetch);
            }
        }
    }
    for j in 0..T {
        for g in 0..G {
            if trade[(j, g)] > 0.0 {
                let own = st.outflow[(j, g)].max(0.0);
                let (lo, hi) = (P.price_floor * C.base_price[g], P.price_ceiling * C.base_price[g]);
                for (v, sum) in [(&mut st.price, &worth), (&mut st.pexp, &hoped)] {
                    v[(j, g)] = ((own * v[(j, g)] + sum[(j, g)]) / (own + trade[(j, g)])).clamp(lo, hi);
                }
            }
        }
    }
}
