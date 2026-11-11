//! A life cycle for works (not in the Python; `docs/rules/works.md`).
//!
//! Under the port a works is founded small, counts as still building for five to
//! seven years after every order (an order is delivered a quarter of the remainder
//! a month, so it never quite finishes), may not grow meanwhile, and stops at four
//! times its method's usual size. Demand that grows is therefore met by more small
//! works, each taking a quarter of a cell whatever its trade: by 1900 thousands of
//! them stand on a sixth of the land.
//!
//! With `works.enabled` a works has a life instead:
//!
//! - A. it has finished building when under `built_share` of its capacity is
//!   still on order, so a works that earns its keep can grow every year;
//! - B. it takes land by its hands (`hands_per_cell` fill a cell), grows where it
//!   stands until it would take `cell_limit` of the cell, and after that opens a
//!   sister works the size of the growth it was refused, on a neighbouring cell
//!   with room;
//! - C. entry counts the merchant's spread, so the return a site promises is
//!   the return a works there gets;
//! - D. each trade in each town has a year's room for new capacity, a share
//!   `year_room` of the town's monthly trade in the good: the works already
//!   there take it first, the best earners at the head of the queue, and a new
//!   works takes what is left;
//! - E. a growing family branch extends a works it has before it founds another;
//! - F. hands walk to work: labour per unit rises by `walk` for each unit of
//!   path cost between the works and its market, so light trades stand in and
//!   by the town and only what is tied to coal or clay stays out on it.
//!
//! Nothing here pays or charges anyone outside the purses `econ::money_total`
//! counts: an order is paid from the works' cash, its backers' or its family's,
//! as before, and the land a works grows onto is rented at the cell's rent that
//! year. Switched off, every function below gives the port's number and the
//! state's two tables stay empty.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::families;
use crate::firms;
use crate::mat::Mat;
use crate::params::Params;
use crate::state::{Event, State};
use crate::util::pairwise_sum_by;

/// The numbers of the life cycle. Each lettered part has its own number or
/// switch, so `--set works.walk=0` (say) measures the rest without it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WorksParams {
    /// Works have a life cycle.
    pub enabled: bool,
    /// A. A works has finished building when less than this share of its capacity
    /// (built and on order) is still on order. 0: any open order counts, as in the port.
    pub built_share: f64,
    /// B. Hands that fill a cell: a works of capacity K takes `K * labor / hands_per_cell`
    /// of its cell. 0: land by size, a quarter of a cell at the usual size, as in the port.
    pub hands_per_cell: f64,
    /// B. A works grows where it stands until it would take this much of its cell.
    pub cell_limit: f64,
    /// C. Entry's promised return counts the merchant's spread.
    pub count_spread: bool,
    /// D. The capacity one trade in one town may add in a year, as a share of the
    /// town's monthly trade in the good (never less than one smallest works). 0: no limit.
    pub year_room: f64,
    /// D. The most of a year's room one works' growth may take.
    pub room_share: f64,
    /// D. A new works is sized at this share of the town's monthly trade in the good,
    /// within the room left and the cell (never below the smallest works). 0: the
    /// port's size, doubled where the promise is high.
    pub entry_share: f64,
    /// E. A growing family branch extends a works it has before founding another.
    pub family_extends: bool,
    /// F. Labour per unit rises by this for each unit of path cost from the works
    /// to its market. 0: hands cost the same anywhere.
    pub walk: f64,
}

impl Default for WorksParams {
    /// Off: what the port runs with. The numbers are the game's, so that
    /// `--set works.enabled=true` alone measures the mechanism as it is played.
    fn default() -> WorksParams {
        WorksParams {
            enabled: false,
            built_share: 0.05,
            hands_per_cell: 2000.0,
            cell_limit: 0.8,
            count_spread: true,
            year_room: 0.15,
            room_share: 0.34,
            entry_share: 0.10,
            family_extends: true,
            walk: 0.02,
        }
    }
}

impl WorksParams {
    /// The game's life cycle.
    pub fn game() -> WorksParams {
        WorksParams { enabled: true, ..WorksParams::default() }
    }
}

/// The state's two tables, sized once the run knows its towns and methods
/// (`sim::init_before_firms`). Left empty when the life cycle is off.
pub fn init(st: &mut State, C: &Content, P: &Params) {
    if !P.works.enabled {
        return;
    }
    st.works_added = Mat::zeros(st.T_max, C.M);
    // no works stands yet when a run starts; a state that takes the life cycle
    // up part-way (`new_year` calls this then) has works already on their land,
    // and their leases cover it
    st.works_lease = (0..st.f_alive.len()).map(|f| if st.f_alive[f] { land_held(st, C, P, f) } else { 0.0 }).collect();
}

// ---------------------------------------------------------------- B. land

/// The share of a cell a works of capacity K takes. This is the one place that
/// says how much land a works needs: the rent bid, the cell's built share, the
/// size limit and the room a newcomer looks for all come from it.
pub fn land_take(C: &Content, P: &Params, m: usize, K: f64) -> f64 {
    let w = &P.works;
    if w.enabled && w.hands_per_cell > 0.0 {
        // by its hands at the method's book labour: a smithy of five men takes a
        // yard, a mill of a thousand hands half a cell
        K * C.m_labor[m] / w.hands_per_cell
    } else {
        K / (P.footprint_K_mult * C.m_K0[m])
    }
}

/// The share of its cell works f holds. Under the port that is its built
/// capacity alone. Here the ground is taken when the building starts, so
/// capacity on order counts too: the works pays rent on it from that day, and
/// nobody else can plan on the same yard meanwhile.
pub fn land_held(st: &State, C: &Content, P: &Params, f: usize) -> f64 {
    let K = if P.works.enabled { st.f_K[f] + st.f_order_K[f] } else { st.f_K[f] };
    land_take(C, P, st.f_method[f], K)
}

/// The largest a works grows where it stands. Beyond it, more capacity means a
/// sister works on another cell.
pub fn max_K(C: &Content, P: &Params, m: usize) -> f64 {
    let w = &P.works;
    if w.enabled && w.hands_per_cell > 0.0 {
        let per_unit = land_take(C, P, m, 1.0);
        if per_unit > 0.0 { w.cell_limit / per_unit } else { f64::INFINITY }
    } else {
        P.footprint_K_mult * C.m_K0[m]
    }
}

/// Whether `cell` has room for K more capacity of method m. The port never
/// asks (it looks only at how full the cell is already).
pub fn cell_fits(st: &State, C: &Content, P: &Params, cell: usize, m: usize, K: f64) -> bool {
    !P.works.enabled || st.cell_K_share[cell] + land_take(C, P, m, K) <= 1.0
}

/// `cell_fits` for someone who will be told why not (the family's site list).
pub fn room_on_cell(st: &State, C: &Content, P: &Params, cell: usize, m: usize, K: f64) -> Result<(), String> {
    if cell_fits(st, C, P, cell, m, K) {
        Ok(())
    } else {
        Err(format!("the cell has no room left for {} of this size", C.method_label[m].to_lowercase()))
    }
}

/// Whether a works that closes leaves its cell derelict: under the port one of
/// at least the usual size (a quarter of a cell); with land by hands, one that
/// took the quarter of a cell that makes the cell industrial on the map.
pub fn leaves_derelict(C: &Content, P: &Params, m: usize, K: f64) -> bool {
    let w = &P.works;
    if w.enabled && w.hands_per_cell > 0.0 { land_take(C, P, m, K) >= 0.25 } else { K >= C.m_K0[m] }
}

/// The capacity of a sister works that stands in for a growth step of dK. The
/// port's is always the usual size; here it is the step itself, and never less
/// than the smallest works anyone builds.
pub fn sister_K(C: &Content, P: &Params, m: usize, dK: f64) -> f64 {
    if P.works.enabled { dK.max(P.entry_K_frac * C.m_K0[m]) } else { C.m_K0[m] }
}

/// The works a concern is traced back to: the first of its line of sister works.
fn root(st: &State, mut f: usize) -> usize {
    for _ in 0..st.F_count {
        match st.f_parent[f] {
            p if p >= 0 => f = p as usize,
            _ => break,
        }
    }
    f
}

/// A works with no room where it stands puts its growth of dK into a works of
/// the same concern (its sisters, their sisters, the works they came from)
/// that has room: the largest such in its town. The money is found as for a
/// new sister works: four fifths of the parent's cash first, then its backers.
/// False if there is no such works, or under the port, and then a new sister
/// works is tried. True when there is one, whether or not the money was found.
pub fn extend_sister(st: &mut State, C: &Content, P: &Params, parent: usize, dK: f64, unit_cap: f64) -> bool {
    if !P.works.enabled {
        return false;
    }
    let (i, m, owner) = (st.f_town[parent], st.f_method[parent], st.f_owner.get(parent).copied().unwrap_or(-1));
    let line = root(st, parent);
    let target = (0..st.F_count)
        .filter(|&f| {
            f != parent
                && st.f_alive[f]
                && st.f_method[f] == m
                && st.f_town[f] == i
                && st.f_owner.get(f).copied().unwrap_or(-1) == owner
                && st.f_K[f] > 0.0
                && st.f_K[f] + st.f_order_K[f] + dK <= max_K(C, P, m)
                && cell_fits(st, C, P, st.f_cell[f], m, dK)
                && root(st, f) == line
        })
        .max_by(|&a, &b| st.f_K[a].total_cmp(&st.f_K[b]).then(b.cmp(&a)));
    let Some(f) = target else { return false };
    let cost = dK * unit_cap * 1.15;
    let own = (st.f_cash[parent].max(0.0) * 0.8).min(cost);
    // what its backers could add: the family's cash, or what the town's savers offer
    let offered = match families::owner_of_firm(st, parent) {
        Some(k) => st.families[k].cash.max(0.0),
        None => firms::town_pool(st, P, i),
    };
    if own + offered.min(cost - own) < 0.6 * cost {
        // the concern cannot pay for the growth this year. Nobody's money is
        // touched, and a new sister works, which would cost no less, is not
        // tried either.
        return true;
    }
    let backed = if own < cost { firms::back_firm(st, P, parent, cost - own) } else { 0.0 };
    st.f_cash[parent] -= own;
    st.f_cash[f] += own + backed;
    for g in 0..C.G {
        st.f_order[(f, g)] += dK * C.m_cap[(m, g)];
    }
    st.f_order_K[f] += dK;
    ordered(st, C, P, f, dK);
    true
}

// ---------------------------------------------------------------- A. building

/// Whether a works with K built and `on_order` still to come counts as building
/// (and so may not grow, shrink or be judged idle).
pub fn building(P: &Params, K: f64, on_order: f64) -> bool {
    let w = &P.works;
    // a works with nothing built yet is building whatever the share
    if w.enabled { on_order > 0.0 && (K <= 0.0 || on_order > w.built_share * (K + on_order)) } else { on_order > 0.0 }
}

// ---------------------------------------------------------------- C. the spread

/// What entry takes off a unit's price for the merchant, where the town's
/// price is p: his spread, as the works' revenue will have it taken
/// (`econ::step_month`), or nothing under the port.
pub fn entry_spread(P: &Params, p: f64) -> f64 {
    if P.works.enabled && P.works.count_spread { P.merchant_spread * p } else { 0.0 }
}

// ---------------------------------------------------------------- F. the walk

/// How much more labour a unit of output takes on `cell` than in the town
/// itself: hands live in the town and walk out, and the walk is paid for.
pub fn walk(st: &State, P: &Params, cell: usize) -> f64 {
    let d = st.dcatch[cell];
    if P.works.enabled && P.works.walk > 0.0 && d.is_finite() { 1.0 + P.works.walk * d } else { 1.0 }
}

/// `walk` applied to the labour per unit of works standing on `cells`.
pub fn walk_to_work(st: &State, P: &Params, cells: &[usize], l: &mut [f64]) {
    if !(P.works.enabled && P.works.walk > 0.0) {
        return;
    }
    for (x, &c) in l.iter_mut().zip(cells) {
        *x *= walk(st, P, c);
    }
}

// ---------------------------------------------------------------- D. the year's room

fn rationed(P: &Params) -> bool {
    P.works.enabled && P.works.year_room > 0.0
}

/// Whether new works are sized to their market (and so come one at a time, with
/// no doubling where the promise is high).
pub fn sized_entry(P: &Params) -> bool {
    P.works.enabled && P.works.entry_share > 0.0
}

/// The town's monthly trade in what method m makes: what its market sold and
/// shipped out, smoothed over the last months (`st.outflow`).
pub fn town_trade(st: &State, C: &Content, i: usize, m: usize) -> f64 {
    st.outflow[(i, C.method_out[m])]
}

/// The capacity method m may add in town i in a year: `year_room` of the trade,
/// and never less than one smallest works, so a trade nobody is in yet can start.
pub fn year_budget(st: &State, C: &Content, P: &Params, i: usize, m: usize) -> f64 {
    (P.works.year_room * town_trade(st, C, i, m)).max(P.entry_K_frac * C.m_K0[m])
}

/// What is left of the year's room (without limit under the port).
pub fn room_left(st: &State, C: &Content, P: &Params, i: usize, m: usize) -> f64 {
    if rationed(P) && st.works_added.rows > 0 {
        (year_budget(st, C, P, i, m) - st.works_added[(i, m)]).max(0.0)
    } else {
        f64::INFINITY
    }
}

/// The capacity a new works of method m in town i is built with. Under the port,
/// `entry_K_frac` of the usual size; here a share of the town's trade in the good,
/// within the year's room and what a cell holds, and never below the port's size.
/// Families found works of this size too, and their site sums are for it.
pub fn entry_K(st: &State, C: &Content, P: &Params, i: usize, m: usize) -> f64 {
    let base = P.entry_K_frac * C.m_K0[m];
    if sized_entry(P) {
        (P.works.entry_share * town_trade(st, C, i, m)).min(room_left(st, C, P, i, m)).min(max_K(C, P, m)).max(base)
    } else {
        base
    }
}

/// A new works of capacity K cut down to what `purse` pays for, where a unit of
/// capacity costs `unit_cost` in capital goods (and founding 15 per cent over
/// that), but never below the smallest works. A family that cannot afford a
/// works sized to its market founds the one it can afford. Under the port, K.
pub fn within_purse(C: &Content, P: &Params, m: usize, K: f64, purse: f64, unit_cost: f64) -> f64 {
    if !sized_entry(P) {
        return K;
    }
    K.min(0.999 * purse.max(0.0) / (1.15 * unit_cost).max(1e-12)).max(P.entry_K_frac * C.m_K0[m])
}

/// The candidate cells entry may propose a works of capacity K on: none if the
/// works already there took the year's room, otherwise those with room for it.
/// Under the port, all of them.
pub fn sites_with_room(st: &State, C: &Content, P: &Params, i: usize, m: usize, K: f64, mut cells: Vec<usize>) -> Vec<usize> {
    if !P.works.enabled {
        return cells;
    }
    if room_left(st, C, P, i, m) < 0.999 * P.entry_K_frac * C.m_K0[m] {
        cells.clear();
    } else {
        cells.retain(|&c| cell_fits(st, C, P, c, m, K));
    }
    cells
}

/// The start of the yearly review: the year's room is whole again, and the
/// works are put in the order they will be served, the best return on capacity
/// over the past year first (the port reviews them in the order they were founded).
pub fn new_year(st: &mut State, C: &Content, P: &Params, pbar: &Mat, alive: &mut [usize]) {
    if !P.works.enabled {
        return;
    }
    if st.works_added.rows != st.T_max || st.works_lease.len() != st.f_alive.len() {
        init(st, C, P);
    }
    st.works_added.fill(0.0);
    if !rationed(P) {
        return;
    }
    let mut r = vec![0.0; st.f_alive.len()];
    for &f in alive.iter() {
        let unit_cap = pairwise_sum_by(C.G, |g| C.m_cap[(st.f_method[f], g)] * pbar[(st.f_town[f], g)]);
        r[f] = st.f_profit_acc[f] / (st.f_K[f].max(1e-9) * unit_cap).max(1e-9);
    }
    alive.sort_by(|&a, &b| r[b].total_cmp(&r[a]).then(a.cmp(&b)));
}

/// The growth a works may order this year when it would like dK more: what the
/// year's room in its town and trade still allows, and no more than `room_share`
/// of the whole room. Zero when that is nothing worth an order.
pub fn growth_allowed(st: &State, C: &Content, P: &Params, f: usize, dK: f64) -> f64 {
    if !rationed(P) {
        return dK;
    }
    let (i, m) = (st.f_town[f], st.f_method[f]);
    let allowed = dK.min(room_left(st, C, P, i, m)).min(P.works.room_share * year_budget(st, C, P, i, m));
    if allowed < 1e-6 * st.f_K[f] { 0.0 } else { allowed }
}

/// A works was opened with capacity K on order: the capacity counts against
/// the year's room, and its lease covers the land that capacity takes.
pub fn opened(st: &mut State, C: &Content, P: &Params, f: usize, K: f64) {
    if !P.works.enabled {
        return;
    }
    let (i, m) = (st.f_town[f], st.f_method[f]);
    if st.works_added.rows > 0 {
        st.works_added[(i, m)] += K;
    }
    if f < st.works_lease.len() {
        st.works_lease[f] = land_take(C, P, m, K);
    }
}

/// A standing works ordered dK more capacity: it counts against the year's
/// room, and if the works now needs more land than its lease covers, it rents
/// the rest at what the cell's land earns this year (the same bid a new works
/// makes, without its floor), and never for less than it pays for the ground
/// it has: once the works have taken most of a cell no other use is left there
/// to set a price, and the ground would come free. The rent is paid monthly
/// like the rest.
///
/// In the year a town is founded its mean wage is zero and the land bids around
/// it are astronomical (FAMILIES.md, "Surprises"); a new works would see that
/// and stay away, and a standing one extends its lease at the rate it already
/// pays.
pub fn ordered(st: &mut State, C: &Content, P: &Params, f: usize, dK: f64) {
    if !P.works.enabled {
        return;
    }
    let (i, m, c) = (st.f_town[f], st.f_method[f], st.f_cell[f]);
    if st.works_added.rows > 0 {
        st.works_added[(i, m)] += dK;
    }
    // the ground is taken now, so the next works to ask this year finds the cell that much fuller
    st.cell_K_share[c] += land_take(C, P, m, dK);
    if f < st.works_lease.len() {
        let need = land_take(C, P, m, st.f_K[f] + st.f_order_K[f]);
        let lease = st.works_lease[f];
        if need > lease {
            let priced = st.wbar.get(i).is_some_and(|&w| w > 0.0);
            // a cell's worth of ground a month: at the rate it pays now, at the cell's rent this year
            let own = if lease > 0.0 { st.f_rent[f] / lease } else { 0.0 };
            let cell = if priced { 1.25 * st.land_rent_only[c] } else { 0.0 };
            st.f_rent[f] += own.max(cell) * (need - lease);
            st.works_lease[f] = need;
        }
    }
}

// ---------------------------------------------------------------- E. families extend

/// The works a growing branch of family k would extend by K in town t instead
/// of founding another of method m: its largest there that would still fit its
/// cell. None under the port, or with `family_extends` off.
pub fn family_extension(st: &State, C: &Content, P: &Params, k: usize, t: usize, m: usize, K: f64) -> Option<usize> {
    if !(P.works.enabled && P.works.family_extends) {
        return None;
    }
    (0..st.F_count)
        .filter(|&f| {
            st.f_alive[f]
                && st.f_owner[f] == k as i64
                && st.f_town[f] == t
                && st.f_method[f] == m
                && st.f_K[f] + st.f_order_K[f] + K <= max_K(C, P, m)
                && cell_fits(st, C, P, st.f_cell[f], m, K)
        })
        .max_by(|&a, &b| st.f_K[a].total_cmp(&st.f_K[b]).then(b.cmp(&a)))
}

/// Family k orders K more capacity for its works f and pays `cost` into the
/// works' cash for it, as it would have paid to found a works of that size. The
/// player hears of it: at once, or with the chronicle in the year's summary of
/// what the branches did (`chronicle::branch_summary`).
#[allow(clippy::too_many_arguments)]
pub fn extend_for_family(st: &mut State, C: &Content, P: &Params, k: usize, f: usize, K: f64, cost: f64, year: i64, events: &mut Vec<Event>) {
    let m = st.f_method[f];
    st.families[k].cash -= cost;
    st.families[k].income.invested += cost;
    st.f_cash[f] += cost;
    for g in 0..C.G {
        st.f_order[(f, g)] += K * C.m_cap[(m, g)];
    }
    st.f_order_K[f] += K;
    ordered(st, C, P, f, K);
    // chronicle: an extension is a branch's own doing and goes into the year's summary
    if k == 0 && !crate::chronicle::works_extended(st, P, k, f, cost) {
        let fam = &st.families[k];
        let title = format!("The {} family extends {}", fam.surname, st.f_name[f]);
        let text = format!("More capacity is ordered for {cost:.0}.");
        let mut e = Event::new(year as f64 + 0.95, "family", 0.4, title, text);
        e.town = Some(st.f_town[f]);
        e.cell = Some(st.f_cell[f]);
        e.firm = Some(f);
        e.family = Some(k);
        events.push(e);
    }
}
