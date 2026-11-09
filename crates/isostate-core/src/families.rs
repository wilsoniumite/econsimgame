//! Families: the player's and its rivals'. Not in the Python.
//!
//! A family owns works and land, draws their dividends and rents, spends part of
//! its income on luxuries and puts the rest into new works or more land. Index 0
//! is the player, who acts through `site_options`, `found_works`, `buy_land` and
//! `assign_member`; families 1 and up act by themselves once a year. Members
//! manage industry groups, and a family's works in a managed group need a little
//! less of everything per unit of output.
//!
//! Everything here is off by default (`n_families = 0`): the state then has no
//! families and no owner tables, every hook returns at once, and the simulation
//! is exactly the port of the Python. See FAMILIES.md for the design.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::content::Content;
use crate::econ::money_total;
use crate::firms::{self, SiteEval};
use crate::land::{USE_DERELICT, USE_SECTOR0};
use crate::mat::Mat;
use crate::names;
use crate::params::{Params, RentRegime};
use crate::rng::{Rng, integers, random};
use crate::state::{Event, State};
use crate::util::{pairwise_sum, pairwise_sum_by};

// ---------------------------------------------------------------- types

/// The industry groups a family member can manage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Group {
    Iron,
    Textiles,
    CoalMining,
    Engineering,
    Building,
    Estates,
}

pub const N_GROUPS: usize = 6;

impl Group {
    pub const ALL: [Group; N_GROUPS] =
        [Group::Iron, Group::Textiles, Group::CoalMining, Group::Engineering, Group::Building, Group::Estates];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Group::Iron => "iron",
            Group::Textiles => "textiles",
            Group::CoalMining => "coal_mining",
            Group::Engineering => "engineering",
            Group::Building => "building",
            Group::Estates => "estates",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Group::Iron => "Iron and steel",
            Group::Textiles => "Textiles",
            Group::CoalMining => "Coal and mining",
            Group::Engineering => "Engineering",
            Group::Building => "Building materials",
            Group::Estates => "Estates",
        }
    }

    /// The group a method's works belong to; shops and trades belong to none.
    pub fn of_method(C: &Content, m: usize) -> Option<Group> {
        match C.method_name[m].as_str() {
            "bloomery" | "charcoal_furnace" | "coke_furnace" | "crucible_steel" | "bessemer" | "forge_rails"
            | "rolling_mill" | "steel_rolling" => Some(Group::Iron),
            "hand_loom" | "textile_mill" => Some(Group::Textiles),
            "charcoal_burning" => Some(Group::CoalMining),
            "smithy" | "millwright" | "engine_works" => Some(Group::Engineering),
            "brickworks" => Some(Group::Building),
            _ => None,
        }
    }

    /// The group a land sector belongs to, for the units on a family's land.
    pub fn of_sector(C: &Content, s: usize) -> Option<Group> {
        match C.sector_name[s].as_str() {
            "farm" | "pasture" | "forestry" => Some(Group::Estates),
            "quarry" | "claypit" => Some(Group::Building),
            "ironmine" | "coalmine" => Some(Group::CoalMining),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// Family 0: acts only through the API.
    Player,
    /// Buys land near home and builds only on very good prospects.
    Landed,
    /// Builds works near home.
    Industrial,
}

/// Adults can manage an industry group.
pub const ADULT_AGE: i64 = 18;

#[derive(Clone, Debug, Serialize)]
pub struct Member {
    pub first: String,
    pub age: i64,
    /// 0 to 1: how much of the full bonus (`family_bonus`) the member's management gives.
    pub skill: f64,
    pub alive: bool,
    pub group: Option<Group>,
    /// The age the member dies at, drawn at birth and not shown to players.
    #[serde(skip)]
    pub(crate) lifespan: i64,
}

/// One year's money in and out of a family.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Income {
    /// Dividends from the family's works, by industry group (`Group::index`).
    pub dividends: [f64; N_GROUPS],
    /// Dividends from works in no group (shops and trades).
    pub other_dividends: f64,
    /// Rent from the family's land.
    pub rents: f64,
    /// Received for land sold to another family.
    pub land_sales: f64,
    /// Spent on luxuries, through the home town's landlords.
    pub spending: f64,
    /// Spent founding works and buying land.
    pub invested: f64,
}

impl Income {
    /// Dividends and rents.
    pub fn earned(&self) -> f64 {
        self.dividends.iter().sum::<f64>() + self.other_dividends + self.rents
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Family {
    pub surname: String,
    pub strategy: Strategy,
    /// Home town: AI families build and buy around it.
    pub home: usize,
    pub cash: f64,
    pub members: Vec<Member>,
    /// This year so far.
    pub income: Income,
    /// The last complete year.
    pub last_year: Income,
    /// Dividends and rents of the last complete month (the base of the spending rule).
    pub month_income: f64,
    /// This month's dividends and rents so far.
    #[serde(skip)]
    pub(crate) month_acc: f64,
}

// ---------------------------------------------------------------- lookups

pub fn active(st: &State) -> bool {
    !st.families.is_empty()
}

pub fn owner_of_firm(st: &State, f: usize) -> Option<usize> {
    st.f_owner.get(f).and_then(|&o| (o >= 0).then_some(o as usize))
}

pub fn owner_of_cell(st: &State, c: usize) -> Option<usize> {
    st.cell_owner.get(c).and_then(|&o| (o >= 0).then_some(o as usize))
}

/// Total cash of all families (part of the money stock, `econ::money_total`).
pub fn cash_total(st: &State) -> f64 {
    let cash: Vec<f64> = st.families.iter().map(|f| f.cash).collect();
    pairwise_sum(&cash)
}

/// The member managing group g, if any.
pub fn manager(st: &State, family: usize, g: Group) -> Option<usize> {
    st.families[family].members.iter().position(|m| m.alive && m.group == Some(g))
}

/// Efficiency bonus of a family's works in group g: `family_bonus` times the
/// manager's skill, 0 when nobody manages the group.
pub fn bonus(st: &State, P: &Params, family: usize, g: Option<Group>) -> f64 {
    let Some(g) = g else { return 0.0 };
    match manager(st, family, g) {
        Some(k) => P.family_bonus * st.families[family].members[k].skill,
        None => 0.0,
    }
}

fn prices(st: &State) -> &Mat {
    if st.pbar.rows > 0 { &st.pbar } else { &st.price }
}

fn wages(st: &State) -> &[f64] {
    if st.wbar.is_empty() { &st.wage } else { &st.wbar }
}

/// Capital-goods cost of one unit of capacity of method m in town i.
fn unit_capital_cost(st: &State, C: &Content, m: usize, i: usize) -> f64 {
    let p = prices(st);
    pairwise_sum_by(C.G, |g| C.m_cap[(m, g)] * p[(i, g)])
}

/// What a cell costs: `years_purchase` times the rent it paid over the last year,
/// that rent never counted below `land_rent_floor` town wages a month (so unused
/// land is cheap, not free).
///
/// The rent actually paid, not the map's rent (`cell_rent`, from the land bids):
/// in the year a town is founded its mean wage is still zero and the bids around
/// it are astronomical (see FAMILIES.md).
pub fn land_price(st: &State, P: &Params, cell: usize) -> f64 {
    let t = st.catch[cell];
    let floor = if t >= 0 { 12.0 * P.land_rent_floor * st.wage[t as usize] } else { 0.0 };
    P.years_purchase * st.cell_rent_year[cell].max(floor)
}

/// Cash taken from a family, up to what it has (for its works' needs).
pub fn draw_cash(st: &mut State, family: usize, amount: f64) -> f64 {
    let fam = &mut st.families[family];
    let got = amount.min(fam.cash.max(0.0)).max(0.0);
    fam.cash -= got;
    got
}

// ---------------------------------------------------------------- the start

fn new_member(rng: &mut Rng, used: &mut BTreeSet<String>, age: i64) -> Member {
    let first = names::member_name(rng, used);
    let skill = 0.2 + 0.7 * random(rng);
    let lifespan = (55.0 + 25.0 * random(rng)) as i64;
    Member { first, age, skill, alive: true, group: None, lifespan: lifespan.max(age + 2) }
}

/// Take `amount` from the households' and landlords' cash of every town, in
/// proportion to what each holds. Returns what was taken.
fn take_from_savers(st: &mut State, amount: f64) -> f64 {
    let pos = |v: &[f64]| -> f64 { v.iter().map(|x| x.max(0.0)).sum() };
    let total = pos(&st.hh_cash) + pos(&st.lord_cash);
    if total <= 0.0 {
        return 0.0;
    }
    let frac = (amount / total).min(0.5);
    let mut taken = 0.0;
    for v in st.hh_cash.iter_mut().chain(st.lord_cash.iter_mut()) {
        let x = v.max(0.0) * frac;
        *v -= x;
        taken += x;
    }
    taken
}

/// Create the families at the start of the recorded run (after the warm-up).
///
/// Home towns go to the families in order of town size. Every family gets
/// `family_start_cash` of the money stock, taken from the towns' savers. Landed
/// families get the farmland nearest their home town until they hold
/// `family_start_share` of all rent; industrial families get the largest works
/// of their home town until they hold that share of all works' capital. The
/// player gets the same share of works' capital in cash, to build with.
pub fn init(st: &mut State, C: &Content, P: &Params, rng: &mut Rng) {
    let n = P.n_families;
    if n == 0 {
        return;
    }
    st.f_owner = vec![-1; st.f_alive.len()];
    st.cell_owner = vec![-1; st.N];
    // no rent has been booked cell by cell yet: start from a year of the map's rent
    st.cell_rent_paid = vec![0.0; st.N];
    st.cell_rent_year = st.cell_rent.iter().map(|&r| 12.0 * r).collect();
    let mut towns: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    towns.sort_by(|&a, &b| st.pop[b].total_cmp(&st.pop[a]));
    let mut used_surnames = BTreeSet::new();
    let mut used_firsts = BTreeSet::new();
    for k in 0..n {
        let strategy = match k {
            0 => Strategy::Player,
            k if k % 2 == 1 => Strategy::Landed,
            _ => Strategy::Industrial,
        };
        let surname = names::family_surname(rng, &mut used_surnames);
        let n_members = 3 + integers(rng, 4);
        used_firsts.clear();
        let mut members = Vec::with_capacity(n_members);
        for j in 0..n_members {
            let age = if j == 0 { 40 + integers(rng, 21) } else { 5 + integers(rng, 36) } as i64;
            members.push(new_member(rng, &mut used_firsts, age));
        }
        st.families.push(Family {
            surname,
            strategy,
            home: towns[k % towns.len()],
            cash: 0.0,
            members,
            income: Income::default(),
            last_year: Income::default(),
            month_income: 0.0,
            month_acc: 0.0,
        });
    }

    let works_total: f64 = (0..st.F_count).filter(|&f| st.f_alive[f]).map(|f| works_value(st, C, f)).sum();
    let rent_total: f64 = (0..st.N).filter(|&c| st.world.land[c]).map(|c| st.cell_rent_year[c]).sum();
    let each = P.family_start_cash * money_total(st);
    let player_extra = P.family_start_share * works_total;
    let taken = take_from_savers(st, each * n as f64 + player_extra);
    let scale = taken / (each * n as f64 + player_extra).max(1e-12);
    for k in 0..n {
        let extra = if k == 0 { player_extra } else { 0.0 };
        st.families[k].cash = (each + extra) * scale;
    }

    for k in 1..n {
        let home = st.families[k].home;
        match st.families[k].strategy {
            Strategy::Landed => {
                let mut cells: Vec<usize> = (0..st.N)
                    .filter(|&c| st.catch[c] == home as i64 && is_farmland(st, C, c) && st.cell_rent_year[c] > 0.0)
                    .filter(|&c| st.cell_owner[c] < 0)
                    .collect();
                cells.sort_by(|&a, &b| st.dist[(home, a)].total_cmp(&st.dist[(home, b)]));
                let mut held = 0.0;
                for c in cells {
                    if held >= P.family_start_share * rent_total {
                        break;
                    }
                    st.cell_owner[c] = k as i64;
                    held += st.cell_rent_year[c];
                }
            }
            _ => {
                let mut works: Vec<usize> = (0..st.F_count)
                    .filter(|&f| st.f_alive[f] && st.f_town[f] == home && st.f_owner[f] < 0)
                    .filter(|&f| Group::of_method(C, st.f_method[f]).is_some())
                    .collect();
                works.sort_by(|&a, &b| works_value(st, C, b).total_cmp(&works_value(st, C, a)));
                let mut held = 0.0;
                for f in works {
                    if held >= P.family_start_share * works_total {
                        break;
                    }
                    st.f_owner[f] = k as i64;
                    held += works_value(st, C, f);
                }
            }
        }
        assign_managers(st, C, P, k);
    }
}

/// Farmland, pasture or woodland (not urban, industrial or derelict).
fn is_farmland(st: &State, C: &Content, c: usize) -> bool {
    if !st.world.land[c] || st.urban[c] {
        return false;
    }
    let u = st.land_use[c];
    u >= USE_SECTOR0 && ((u - USE_SECTOR0) as usize) < C.NS && Group::of_sector(C, (u - USE_SECTOR0) as usize) == Some(Group::Estates)
}

/// A works at depreciated capital cost: its capacity, built and on order, at
/// today's capital-goods prices.
fn works_value(st: &State, C: &Content, f: usize) -> f64 {
    (st.f_K[f] + st.f_order_K[f]) * unit_capital_cost(st, C, st.f_method[f], st.f_town[f])
}

// ---------------------------------------------------------------- the month (econ::step_month)

/// Each family's luxury budget this month (empty without families): like the
/// landlords' budget, a share of last month's income and of its cash, never more
/// than its cash. The budgets are spent through the home towns' landlords.
pub fn spending_budgets(st: &State, P: &Params) -> Vec<f64> {
    st.families
        .iter()
        .map(|f| {
            let cash = f.cash.max(0.0);
            cash.min(P.family_cons_income * f.month_income.max(0.0) + P.family_cons_wealth * cash)
        })
        .collect()
}

/// The budgets added up by home town.
pub fn budgets_by_town(st: &State, budgets: &[f64], T: usize) -> Vec<f64> {
    let mut out = vec![0.0; T];
    for (f, b) in st.families.iter().zip(budgets) {
        out[f.home] += b;
    }
    out
}

/// Families pay their part of what their home town's landlords bought this
/// month: the landlords paid the merchants for everything, the families repay
/// them in proportion to the budget each put in.
pub fn pay_spending(st: &mut State, budgets: &[f64], lord_budget: &[f64], lord_spend: &[f64]) {
    for (k, &b) in budgets.iter().enumerate() {
        let home = st.families[k].home;
        if b <= 0.0 || lord_budget[home] <= 0.0 {
            continue;
        }
        let pay = lord_spend[home] * b / lord_budget[home];
        let fam = &mut st.families[k];
        fam.cash -= pay;
        fam.income.spending += pay;
        st.lord_cash[home] += pay;
    }
}

/// Dividends of family works go to the family; their entries in `div` are zeroed
/// so the rest still goes to the households of each town.
pub fn take_dividends(st: &mut State, C: &Content, fi: &[usize], div: &mut [f64]) {
    for (k, &f) in fi.iter().enumerate() {
        let Some(o) = owner_of_firm(st, f) else { continue };
        let d = div[k];
        let group = Group::of_method(C, st.f_method[f]);
        let fam = &mut st.families[o];
        fam.cash += d;
        fam.month_acc += d;
        match group {
            Some(g) => fam.income.dividends[g.index()] += d,
            None => fam.income.other_dividends += d,
        }
        div[k] = 0.0;
    }
}

/// Rents paid on family land go to the family. Returns the rents left for the
/// anonymous gentry (or the common fund), with the family cells' entries zeroed.
pub fn take_rents(st: &mut State, cells: &[usize], rent: &[f64]) -> Vec<f64> {
    let mut rest = rent.to_vec();
    let mut to_families = 0.0;
    for (k, &c) in cells.iter().enumerate() {
        st.cell_rent_paid[c] += rent[k];
        let Some(o) = owner_of_cell(st, c) else { continue };
        let fam = &mut st.families[o];
        fam.cash += rent[k];
        fam.month_acc += rent[k];
        fam.income.rents += rent[k];
        to_families += rent[k];
        rest[k] = 0.0;
    }
    st.m_rent += to_families;
    rest
}

/// Urban rent is paid town by town; a family owning urban cells gets their share
/// of it (each cell's share of the town's urban rent bill). Returns the rest.
pub fn take_urban_rents(st: &mut State, urban_pay: &[f64]) -> Vec<f64> {
    let mut rest = urban_pay.to_vec();
    let mut to_families = 0.0;
    for c in 0..st.N {
        let t = st.catch[c];
        if !st.urban[c] || t < 0 || st.urban_rent[t as usize] <= 0.0 {
            continue;
        }
        let t = t as usize;
        let r = urban_pay[t] * st.cell_urban_rent[c] / st.urban_rent[t];
        st.cell_rent_paid[c] += r;
        let Some(o) = owner_of_cell(st, c) else { continue };
        let fam = &mut st.families[o];
        fam.cash += r;
        fam.month_acc += r;
        fam.income.rents += r;
        to_families += r;
        rest[t] -= r;
    }
    st.m_rent += to_families;
    rest
}

/// Family works in a managed group need `1 - bonus` of the inputs, heat and
/// labor per unit that other works need.
pub fn scale_firm_inputs(st: &State, C: &Content, P: &Params, fi: &[usize], a: &mut Mat, h: &mut [f64], l: &mut [f64]) {
    for (k, &f) in fi.iter().enumerate() {
        let Some(o) = owner_of_firm(st, f) else { continue };
        let b = bonus(st, P, o, Group::of_method(C, st.f_method[f]));
        if b == 0.0 {
            continue;
        }
        let e = 1.0 - b;
        for v in a.row_mut(k) {
            *v *= e;
        }
        h[k] *= e;
        l[k] *= e;
    }
}

/// Productivity of the land units on family land whose group the family
/// manages goes up by the bonus (None without families).
pub fn land_boost(st: &State, C: &Content, P: &Params) -> Option<Vec<f64>> {
    if !active(st) {
        return None;
    }
    Some(
        (0..st.ex_cell.len())
            .map(|u| match owner_of_cell(st, st.ex_cell[u]) {
                Some(o) => 1.0 + bonus(st, P, o, Group::of_sector(C, st.ex_sector[u])),
                None => 1.0,
            })
            .collect(),
    )
}

/// The month's income becomes the base of next month's budget.
pub fn end_month(st: &mut State) {
    for f in &mut st.families {
        f.month_income = f.month_acc;
        f.month_acc = 0.0;
    }
}

// ---------------------------------------------------------------- what a family can do

/// The sums behind building one method on one cell, per unit of output unless
/// stated: the same numbers `firms::evaluate_sites` gives entry, with the
/// family's management bonus applied.
#[derive(Clone, Debug, Serialize)]
pub struct SiteSums {
    /// The town whose market the works would trade in (the cell's catchment).
    pub town: usize,
    /// Capacity it would be built with.
    pub K: f64,
    /// What the output fetches at the works' gate: the town price less carting.
    pub gate_price: f64,
    /// Inputs: (good, units per unit of output, delivered price).
    pub inputs: Vec<(usize, f64, f64)>,
    pub input_cost: f64,
    /// Heat per unit and the price of a unit of heat from the cheapest fuel mix.
    pub heat: f64,
    pub heat_price: f64,
    pub heat_cost: f64,
    /// Worker-months per unit and the town wage.
    pub labor: f64,
    pub wage: f64,
    pub labor_cost: f64,
    /// Gate price less input, heat and labor cost.
    pub margin: f64,
    /// Rent bid to take the cell from its current use, per month.
    pub rent_bid: f64,
    /// What founding it costs: the capacity at capital-goods prices, plus 15 per cent.
    pub capital_cost: f64,
    /// Expected yearly return on the capacity's cost (the number entry compares
    /// with the hurdle rate).
    pub expected_return: f64,
    /// The family's management bonus included in these sums.
    pub bonus: f64,
    /// Agglomeration multiplier from works of the same kind nearby.
    pub agglo: f64,
}

/// One method on one cell for one family: the sums, or why not.
#[derive(Clone, Debug, Serialize)]
pub struct SiteOption {
    pub method: usize,
    pub name: String,
    pub label: String,
    pub group: Option<Group>,
    pub sums: Result<SiteSums, String>,
}

fn check_family(st: &State, family: usize) -> Result<(), String> {
    if family < st.families.len() {
        Ok(())
    } else if st.families.is_empty() {
        Err("there are no families in this run (n_families is 0)".to_string())
    } else {
        Err(format!("there is no family {family} (there are {})", st.families.len()))
    }
}

/// Why nothing can be built on a cell, if so; otherwise its town.
fn buildable(st: &State, cell: usize) -> Result<usize, String> {
    if cell >= st.N {
        return Err(format!("cell {cell} is off the map"));
    }
    if !st.world.land[cell] {
        return Err("the cell is sea".to_string());
    }
    let t = st.catch[cell];
    if t < 0 {
        return Err("the cell is out of every town's reach".to_string());
    }
    if st.land_use[cell] == USE_DERELICT {
        return Err("the cell is derelict ground".to_string());
    }
    if st.cell_K_share[cell] >= 0.9 {
        return Err("the cell is already built up".to_string());
    }
    Ok(t as usize)
}

/// The evaluation of method m on one cell with the family's bonus applied.
fn evaluate(st: &State, C: &Content, P: &Params, family: usize, m: usize, cell: usize, i: usize) -> (SiteEval, f64) {
    let K = P.entry_K_frac * C.m_K0[m];
    let mut ev = firms::evaluate_sites(st, C, P, i, m, &[cell], K, prices(st), wages(st));
    let b = bonus(st, P, family, Group::of_method(C, m));
    if b > 0.0 {
        let e = 1.0 - b;
        for v in ev.a.row_mut(0) {
            *v *= e;
        }
        ev.h[0] *= e;
        ev.l[0] *= e;
        ev.in_cost[0] *= e;
        ev.heat_cost[0] *= e;
        ev.labor_cost[0] *= e;
        ev.margin[0] = ev.pout[0] - ev.in_cost[0] - ev.heat_cost[0] - ev.labor_cost[0];
        ev.roi[0] = firms::site_roi(K, ev.margin[0], ev.rent_bid[0], ev.cap_cost);
    }
    (ev, b)
}

fn sums_of(C: &Content, ev: &SiteEval, b: f64, K: f64, i: usize, wage: f64) -> SiteSums {
    SiteSums {
        town: i,
        K,
        gate_price: ev.pout[0],
        inputs: (0..C.G).filter(|&g| ev.a[(0, g)] > 0.0).map(|g| (g, ev.a[(0, g)], ev.pin[(0, g)])).collect(),
        input_cost: ev.in_cost[0],
        heat: ev.h[0],
        heat_price: ev.hunit[0],
        heat_cost: ev.heat_cost[0],
        labor: ev.l[0],
        wage,
        labor_cost: ev.labor_cost[0],
        margin: ev.margin[0],
        rent_bid: ev.rent_bid[0],
        capital_cost: ev.cap_cost * 1.15,
        expected_return: ev.roi[0],
        bonus: b,
        agglo: ev.agglo[0],
    }
}

/// For every method, what building it on `cell` would look like for `family`,
/// or why it cannot be built there. Errors if nothing can be built on the cell.
pub fn site_options(st: &State, C: &Content, P: &Params, family: usize, cell: usize) -> Result<Vec<SiteOption>, String> {
    check_family(st, family)?;
    let i = buildable(st, cell)?;
    Ok((0..C.M)
        .map(|m| {
            let sums = if !(st.know[(i, m)] > 0.02) {
                Err(format!("{} is not known in {} yet", C.method_label[m], st.town_name[i]))
            } else {
                let (ev, b) = evaluate(st, C, P, family, m, cell, i);
                Ok(sums_of(C, &ev, b, P.entry_K_frac * C.m_K0[m], i, wages(st)[i]))
            };
            SiteOption {
                method: m,
                name: C.method_name[m].clone(),
                label: C.method_label[m].clone(),
                group: Group::of_method(C, m),
                sums,
            }
        })
        .collect())
}

/// Build method m on `cell` for `family`, paid from the family's cash, as entry
/// builds (`firms::create_firm`, capacity `entry_K_frac` of the method's K0).
/// Returns the new firm's id. `year` is the year the works is founded in
/// (normally `sim.year`); the news item goes to `events`.
#[allow(clippy::too_many_arguments)]
pub fn found_works(
    st: &mut State,
    C: &Content,
    P: &Params,
    family: usize,
    method: usize,
    cell: usize,
    year: i64,
    events: &mut Vec<Event>,
) -> Result<usize, String> {
    check_family(st, family)?;
    if method >= C.M {
        return Err(format!("there is no method {method}"));
    }
    let i = buildable(st, cell)?;
    if !(st.know[(i, method)] > 0.02) {
        return Err(format!("{} is not known in {} yet", C.method_label[method], st.town_name[i]));
    }
    if st.F_count >= st.f_alive.len() {
        return Err("the firm table is full".to_string());
    }
    let K = P.entry_K_frac * C.m_K0[method];
    let (ev, _) = evaluate(st, C, P, family, method, cell, i);
    let cost = ev.cap_cost * 1.15;
    let fam = &st.families[family];
    if fam.cash < cost {
        return Err(format!("it costs {cost:.0} and the {} family has {:.0}", fam.surname, fam.cash.max(0.0)));
    }
    let name = family_works_name(st, C, family, method, i);
    let reason = firms::reason_record(st, i, method, cell, &ev, 0, K, year, "family");
    // the name is given, so create_firm draws nothing from this stream
    let mut unused = Rng::from_seed([0; 32]);
    let funding = cost;
    st.families[family].cash -= funding;
    st.families[family].income.invested += funding;
    let fid = firms::create_firm(st, C, P, &mut unused, i, method, cell, K, ev.rent_bid[0], reason, year, funding, Some(name), None)
        .expect("checked that the firm table has room");
    st.f_owner[fid] = family as i64;
    st.new_firms_year.push(fid);
    let fam = &st.families[family];
    let what = C.method_label[method].to_lowercase();
    let mut e = Event::new(
        year as f64 + 0.95,
        "family",
        if family == 0 { 0.6 } else { 0.35 },
        format!("The {} family opens {} {} at {}", fam.surname, article(&what), what, st.town_name[i]),
        format!(
            "{} is built for {:.0} and expects to return {:.0} per cent a year.",
            st.f_name[fid],
            cost,
            100.0 * ev.roi[0]
        ),
    );
    e.town = Some(i);
    e.cell = Some(cell);
    e.firm = Some(fid);
    e.family = Some(family);
    events.push(e);
    Ok(fid)
}

fn article(word: &str) -> &'static str {
    if word.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" }
}

/// "Ashby Ironworks", or "Ashby Ironworks No. 2" if the family already has one.
fn family_works_name(st: &State, C: &Content, family: usize, m: usize, i: usize) -> String {
    let base = names::family_firm_name(&C.method_name[m], &st.families[family].surname, &st.town_name[i]);
    let count = st.f_name.iter().filter(|n| n.starts_with(&base)).count();
    if count == 0 { base } else { format!("{base} No. {}", count + 1) }
}

/// Buy `cell` for `family` at `land_price`. The seller is paid: another family,
/// or, for land of the anonymous gentry, whoever its rent goes to now (the
/// landlords of its town, or the common fund under the commons regime). Returns
/// the price paid.
pub fn buy_land(st: &mut State, _C: &Content, P: &Params, family: usize, cell: usize) -> Result<f64, String> {
    check_family(st, family)?;
    if cell >= st.N {
        return Err(format!("cell {cell} is off the map"));
    }
    if !st.world.land[cell] {
        return Err("the cell is sea".to_string());
    }
    let t = st.catch[cell];
    if t < 0 {
        return Err("the cell is out of every town's reach".to_string());
    }
    let seller = owner_of_cell(st, cell);
    if seller == Some(family) {
        return Err(format!("the {} family already owns this land", st.families[family].surname));
    }
    let price = land_price(st, P, cell);
    let fam = &st.families[family];
    if fam.cash < price {
        return Err(format!("the land costs {price:.0} and the {} family has {:.0}", fam.surname, fam.cash.max(0.0)));
    }
    st.families[family].cash -= price;
    st.families[family].income.invested += price;
    match seller {
        Some(j) => {
            st.families[j].cash += price;
            st.families[j].income.land_sales += price;
        }
        None => match P.rent_regime {
            RentRegime::Landlords => st.lord_cash[t as usize] += price,
            RentRegime::Commons => st.fund_cash += price,
        },
    }
    st.cell_owner[cell] = family as i64;
    Ok(price)
}

/// Put `member` in charge of `group` (None: of nothing). A group has one
/// manager, so whoever managed it before stands down.
pub fn assign_member(st: &mut State, family: usize, member: usize, group: Option<Group>) -> Result<(), String> {
    check_family(st, family)?;
    let fam = &mut st.families[family];
    let Some(m) = fam.members.get(member) else {
        return Err(format!("the {} family has no member {member}", fam.surname));
    };
    if !m.alive {
        return Err(format!("{} {} is dead", m.first, fam.surname));
    }
    if group.is_some() && m.age < ADULT_AGE {
        return Err(format!("{} {} is {} and too young to manage anything", m.first, fam.surname, m.age));
    }
    if group.is_some() {
        for other in fam.members.iter_mut() {
            if other.group == group {
                other.group = None;
            }
        }
    }
    fam.members[member].group = group;
    Ok(())
}

// ---------------------------------------------------------------- reports

#[derive(Clone, Debug, Serialize)]
pub struct MemberReport {
    pub index: usize,
    pub first: String,
    pub age: i64,
    pub skill: f64,
    pub alive: bool,
    pub group: Option<Group>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FamilyReport {
    pub index: usize,
    pub surname: String,
    pub player: bool,
    pub strategy: Strategy,
    pub home: usize,
    pub home_name: String,
    pub cash: f64,
    /// The family's works at depreciated capital cost (capacity built and on
    /// order, at today's capital-goods prices).
    pub works_value: f64,
    /// The family's land at `years_purchase` times its annual rent (`land_price`).
    pub land_value: f64,
    pub net_worth: f64,
    /// The last complete year: dividends by group, rents, spending, investment.
    pub last_year: Income,
    /// Alive works the family owns (firm ids).
    pub firms: Vec<usize>,
    /// Cells the family owns.
    pub cells: Vec<usize>,
    pub members: Vec<MemberReport>,
    /// The bonus the family's works in each group get now (`Group::index`).
    pub bonus: [f64; N_GROUPS],
}

pub fn family_report(st: &State, C: &Content, P: &Params, family: usize) -> Result<FamilyReport, String> {
    check_family(st, family)?;
    let fam = &st.families[family];
    let firms: Vec<usize> = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_owner[f] == family as i64).collect();
    let cells: Vec<usize> = (0..st.N).filter(|&c| st.cell_owner[c] == family as i64).collect();
    let works_value: f64 = firms.iter().map(|&f| works_value(st, C, f)).sum();
    let land_value: f64 = cells.iter().map(|&c| land_price(st, P, c)).sum();
    let mut b = [0.0; N_GROUPS];
    for g in Group::ALL {
        b[g.index()] = bonus(st, P, family, Some(g));
    }
    Ok(FamilyReport {
        index: family,
        surname: fam.surname.clone(),
        player: family == 0,
        strategy: fam.strategy,
        home: fam.home,
        home_name: st.town_name[fam.home].clone(),
        cash: fam.cash,
        works_value,
        land_value,
        net_worth: fam.cash + works_value + land_value,
        last_year: fam.last_year.clone(),
        firms,
        cells,
        members: fam
            .members
            .iter()
            .enumerate()
            .map(|(index, m)| MemberReport {
                index,
                first: m.first.clone(),
                age: m.age,
                skill: m.skill,
                alive: m.alive,
                group: m.group,
            })
            .collect(),
        bonus: b,
    })
}

/// How much of the economy the families own: (share of all works' capital,
/// share of the rent paid last year, share of the money stock), each summed over
/// families.
pub fn ownership_shares(st: &State, C: &Content) -> (f64, f64, f64) {
    if !active(st) {
        return (0.0, 0.0, 0.0);
    }
    let (mut works, mut works_fam) = (0.0, 0.0);
    for f in 0..st.F_count {
        if st.f_alive[f] {
            let v = works_value(st, C, f);
            works += v;
            if st.f_owner[f] >= 0 {
                works_fam += v;
            }
        }
    }
    let (mut rent, mut rent_fam) = (0.0, 0.0);
    for c in 0..st.N {
        if st.world.land[c] {
            rent += st.cell_rent_year[c];
            if st.cell_owner[c] >= 0 {
                rent_fam += st.cell_rent_year[c];
            }
        }
    }
    (works_fam / works.max(1e-12), rent_fam / rent.max(1e-12), cash_total(st) / money_total(st).max(1e-12))
}

// ---------------------------------------------------------------- the year (sim::yearly)

/// Members age and some die; families with fewer than three living members get
/// an heir coming of age. AI families put managers in charge and invest. The
/// year's books close.
#[allow(clippy::too_many_arguments)]
pub fn yearly(st: &mut State, C: &Content, P: &Params, rng: &mut Rng, year: i64, events: &mut Vec<Event>) {
    if !active(st) {
        return;
    }
    // the year's rents are in: land is priced on them from now on
    st.cell_rent_year = std::mem::replace(&mut st.cell_rent_paid, vec![0.0; st.N]);
    for k in 0..st.families.len() {
        people(st, C, rng, k, year, events);
    }
    for k in 1..st.families.len() {
        assign_managers(st, C, P, k);
        invest(st, C, P, rng, k, year, events);
    }
    for f in &mut st.families {
        f.last_year = std::mem::take(&mut f.income);
    }
}

fn people(st: &mut State, _C: &Content, rng: &mut Rng, k: usize, year: i64, events: &mut Vec<Event>) {
    let home = st.families[k].home;
    let town = st.town_name[home].clone();
    let fam = &mut st.families[k];
    for m in fam.members.iter_mut().filter(|m| m.alive) {
        m.age += 1;
        if m.age >= m.lifespan {
            m.alive = false;
            let ran = m.group.map(|g| format!(", who ran the family's {}", g.label().to_lowercase())).unwrap_or_default();
            m.group = None;
            let mut e = Event::new(
                year as f64 + 0.5,
                "family",
                if k == 0 { 0.5 } else { 0.2 },
                format!("{} {} dies at {}", m.first, fam.surname, m.age),
                format!("{} {}{}, is buried at {}.", m.first, fam.surname, ran, town),
            );
            e.town = Some(home);
            e.family = Some(k);
            events.push(e);
        }
    }
    let living = fam.members.iter().filter(|m| m.alive).count();
    if living < 3 {
        let mut used: BTreeSet<String> = fam.members.iter().filter(|m| m.alive).map(|m| m.first.clone()).collect();
        let heir = new_member(rng, &mut used, ADULT_AGE);
        fam.members.push(heir);
    }
}

/// What a family holds in each group: its works at capital cost, and its land
/// (farms count as estates, quarries and clay pits as building materials, mines
/// as coal and mining) at its price.
fn holdings_by_group(st: &State, C: &Content, P: &Params, k: usize) -> [f64; N_GROUPS] {
    let mut v = [0.0; N_GROUPS];
    for f in 0..st.F_count {
        if st.f_alive[f] && st.f_owner[f] == k as i64 {
            if let Some(g) = Group::of_method(C, st.f_method[f]) {
                v[g.index()] += works_value(st, C, f);
            }
        }
    }
    for c in 0..st.N {
        if st.cell_owner[c] == k as i64 {
            let u = st.land_use[c];
            let g = if u >= USE_SECTOR0 && ((u - USE_SECTOR0) as usize) < C.NS {
                Group::of_sector(C, (u - USE_SECTOR0) as usize).unwrap_or(Group::Estates)
            } else {
                Group::Estates
            };
            v[g.index()] += land_price(st, P, c);
        }
    }
    v
}

/// The most skilled adults run the groups the family holds most in.
fn assign_managers(st: &mut State, C: &Content, P: &Params, k: usize) {
    let value = holdings_by_group(st, C, P, k);
    let mut groups: Vec<Group> = Group::ALL.into_iter().filter(|g| value[g.index()] > 0.0).collect();
    groups.sort_by(|a, b| value[b.index()].total_cmp(&value[a.index()]));
    let fam = &mut st.families[k];
    let mut adults: Vec<usize> = (0..fam.members.len()).filter(|&j| fam.members[j].alive && fam.members[j].age >= ADULT_AGE).collect();
    adults.sort_by(|&a, &b| fam.members[b].skill.total_cmp(&fam.members[a].skill));
    for m in fam.members.iter_mut() {
        m.group = None;
    }
    for (&j, &g) in adults.iter().zip(&groups) {
        fam.members[j].group = Some(g);
    }
}

/// Cash an AI family keeps back: `family_reserve_months` of its spending.
fn reserve(fam: &Family, P: &Params) -> f64 {
    P.family_reserve_months * (P.family_cons_income * fam.month_income.max(0.0) + P.family_cons_wealth * fam.cash.max(0.0))
}

/// An AI family's yearly investment. Industrial families build up to three of
/// the best works they find around home; landed families first buy up to four
/// cells of land next to their estates, then build up to two works where the
/// return is half as much again as the hurdle rate.
fn invest(st: &mut State, C: &Content, P: &Params, rng: &mut Rng, k: usize, year: i64, events: &mut Vec<Event>) {
    let fam = &st.families[k];
    let mut spare = fam.cash - reserve(fam, P);
    if spare <= 0.0 {
        return;
    }
    let landed = fam.strategy == Strategy::Landed;
    if landed {
        buy_estates(st, C, P, k, &mut spare, 4, year, events);
    }
    let (max_works, hurdle) = if landed { (2, 1.5 * P.hurdle) } else { (3, P.hurdle) };
    build_best(st, C, P, rng, k, &mut spare, max_works, hurdle, year, events);
}

/// Up to `max` cells of farmland in the home town's reach, next to the family's
/// land where possible, then the highest rents.
#[allow(clippy::too_many_arguments)]
fn buy_estates(st: &mut State, C: &Content, P: &Params, k: usize, spare: &mut f64, max: usize, year: i64, events: &mut Vec<Event>) {
    let home = st.families[k].home;
    let (H, W) = (st.world.H, st.world.W);
    let next_to_own = |st: &State, c: usize| -> bool {
        let (y, x) = ((c / W) as isize, (c % W) as isize);
        (-1..=1).any(|dy| {
            (-1..=1).any(|dx| {
                let (ny, nx) = (y + dy, x + dx);
                (dy != 0 || dx != 0)
                    && ny >= 0
                    && nx >= 0
                    && (ny as usize) < H
                    && (nx as usize) < W
                    && st.cell_owner[ny as usize * W + nx as usize] == k as i64
            })
        })
    };
    let mut cells: Vec<(bool, f64, usize)> = (0..st.N)
        .filter(|&c| st.catch[c] == home as i64 && st.cell_owner[c] < 0 && is_farmland(st, C, c) && st.cell_rent_year[c] > 0.0)
        .map(|c| (next_to_own(st, c), st.cell_rent_year[c], c))
        .collect();
    cells.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.total_cmp(&a.1)).then(a.2.cmp(&b.2)));
    let mut bought = 0;
    for (_, _, c) in cells {
        if bought >= max {
            break;
        }
        let price = land_price(st, P, c);
        if price > *spare {
            continue;
        }
        if buy_land(st, C, P, k, c).is_ok() {
            *spare -= price;
            bought += 1;
        }
    }
    if bought > 0 {
        let fam = &st.families[k];
        let mut e = Event::new(
            year as f64 + 0.95,
            "family",
            0.15,
            format!("The {} family buys {} more {} near {}", fam.surname, bought, if bought == 1 { "farm" } else { "farms" }, st.town_name[home]),
            format!("The {} estates grow to {} holdings.", fam.surname, st.cell_owner.iter().filter(|&&o| o == k as i64).count()),
        );
        e.town = Some(home);
        e.family = Some(k);
        events.push(e);
    }
}

/// The best works the family can afford in its home town and the nearest other
/// town: a few candidate cells per method (`firms::candidate_cells`), the best
/// return above `hurdle` first.
#[allow(clippy::too_many_arguments)]
fn build_best(
    st: &mut State,
    C: &Content,
    P: &Params,
    rng: &mut Rng,
    k: usize,
    spare: &mut f64,
    max: usize,
    hurdle: f64,
    year: i64,
    events: &mut Vec<Event>,
) {
    let home = st.families[k].home;
    let mut towns = vec![home];
    let nearest = (0..st.T_max)
        .filter(|&t| t != home && st.town_active[t] && st.town_cost[(home, t)].is_finite())
        .min_by(|&a, &b| st.town_cost[(home, a)].total_cmp(&st.town_cost[(home, b)]));
    towns.extend(nearest);
    let mut options: Vec<(f64, usize, usize, f64)> = Vec::new(); // (return, method, cell, cost)
    for &i in &towns {
        for m in 0..C.M {
            if !(st.know[(i, m)] > 0.02) || Group::of_method(C, m).is_none() {
                continue;
            }
            let cells = firms::candidate_cells(st, C, P, i, m, rng);
            if cells.is_empty() {
                continue;
            }
            let K = P.entry_K_frac * C.m_K0[m];
            let ev = firms::evaluate_sites(st, C, P, i, m, &cells, K, prices(st), wages(st));
            let b = bonus(st, P, k, Group::of_method(C, m));
            let e = 1.0 - b;
            let mut best: Option<(f64, usize)> = None;
            for j in 0..cells.len() {
                let margin = ev.pout[j] - e * (ev.in_cost[j] + ev.heat_cost[j] + ev.labor_cost[j]);
                let roi = firms::site_roi(K, margin, ev.rent_bid[j], ev.cap_cost);
                if best.is_none_or(|(r, _)| roi > r) {
                    best = Some((roi, cells[j]));
                }
            }
            if let Some((roi, c)) = best {
                options.push((roi, m, c, ev.cap_cost * 1.15));
            }
        }
    }
    options.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut built = 0;
    let mut used = BTreeSet::new();
    for (roi, m, c, cost) in options {
        if built >= max || roi <= hurdle {
            break;
        }
        if cost > *spare || !used.insert(c) {
            continue;
        }
        if found_works(st, C, P, k, m, c, year, events).is_ok() {
            *spare -= cost;
            built += 1;
        }
    }
}
