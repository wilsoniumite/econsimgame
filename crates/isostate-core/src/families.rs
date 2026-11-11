//! Families: the player's and its rivals'. Not in the Python.
//!
//! A family owns works and land, draws their dividends and rents, spends part of
//! its income and grows through its people. Members are posted: children, then
//! learners (an apprenticeship in a trade, or university), then managers of a
//! trade in a town, one of them the head, who lives at the family's seat. A
//! family knows a set of trades; a branch is a trade in a town where someone
//! runs it, and each branch grows, holds or winds down as the family says. The
//! player (family 0) chooses only from short lists: training, posts, stances,
//! the seat, successors, and works at the seat; everything else runs itself.
//! Families 1 and up play by the same rules and choose for themselves.
//!
//! Everything here is off by default (`n_families = 0`): the state then has no
//! families and no owner tables, every hook returns at once, and the simulation
//! is exactly the port of the Python. See FAMILIES.md for the design.

use std::collections::{BTreeMap, BTreeSet};

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

/// The trades: industry groups a family can know and run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, serde::Deserialize)]
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
    /// Family 0: chooses through the API (or takes the defaults).
    Player,
    /// Starts with estates and the estates trade.
    Landed,
    /// Starts with works and the strongest trade of its seat.
    Industrial,
}

/// Members manage from 18, choose a training at 14 and train for four years.
pub const ADULT_AGE: i64 = 18;
pub const TRAINING_AGE: i64 = 14;
pub const TRAINING_YEARS: i64 = 4;
/// Families stop having children at this many living members.
pub const FULL_FAMILY: usize = 10;

/// What a member trains in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Path {
    /// The family learns the trade, and the member is a natural manager of it.
    Apprentice { group: Group, town: usize },
    /// A large gain in skill, and the member is educated.
    University,
}

/// What a member does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Post {
    Child,
    /// Training until the end of year `until`.
    Learning { path: Path, until: i64 },
    /// Runs the family's branch of `group` in the member's town.
    Manager { group: Group },
    /// Runs the family from its seat, and every known trade there.
    Head,
    Unposted,
}

/// What a branch does each year.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stance {
    /// Found works of its trade in its town (estates: buy land there).
    Grow,
    /// Nothing new.
    Hold,
    /// Sell one works (estates: one cell) a year, the smallest first.
    WindDown,
}

/// Where the player can post an adult.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Posting {
    Manager { group: Group, town: usize },
    Unposted,
}

#[derive(Clone, Debug, Serialize)]
pub struct Member {
    pub first: String,
    /// Year of birth.
    pub born: i64,
    pub age: i64,
    /// 0 to 1: how much of the full bonus the member's management gives.
    pub skill: f64,
    pub alive: bool,
    /// The town the member lives in.
    pub town: usize,
    pub post: Post,
    /// Has been to university.
    pub educated: bool,
    /// The trade the member apprenticed in, and so manages best.
    pub trade: Option<Group>,
    /// A move to another town, with its post, at the next year end.
    pub next: Option<(usize, Post)>,
    /// The parent (a member index), None for the founding generation.
    pub parent: Option<usize>,
    /// The age the member dies at, drawn at birth and not shown to players.
    #[serde(skip)]
    pub(crate) lifespan: i64,
}

/// One year's money in and out of a family.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Income {
    /// Dividends from the family's works, by trade (`Group::index`).
    pub dividends: [f64; N_GROUPS],
    /// Dividends from works in no trade (shops and trades, if any).
    pub other_dividends: f64,
    /// Rent from the family's land.
    pub rents: f64,
    /// Received for works and land sold.
    pub sales: f64,
    /// Spent on luxuries, through the seat town's landlords.
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

/// One answer to a decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Choice {
    /// The family's first trade.
    Trade { group: Group },
    Apprentice { group: Group, town: usize },
    University,
    /// Who heads the family.
    Head { member: usize },
}

#[derive(Clone, Debug, Serialize)]
pub struct DecisionOption {
    pub label: String,
    pub choice: Choice,
}

/// A choice waiting for the family. Unanswered, it takes its default at the end
/// of year `due`.
#[derive(Clone, Debug, Serialize)]
pub struct Decision {
    pub id: u64,
    pub prompt: String,
    /// The member it is about, if any.
    pub member: Option<usize>,
    pub options: Vec<DecisionOption>,
    pub default: usize,
    pub due: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Family {
    pub surname: String,
    pub strategy: Strategy,
    /// The town the head lives in.
    pub seat: usize,
    /// A new seat, from the next year end.
    pub next_seat: Option<usize>,
    pub cash: f64,
    pub members: Vec<Member>,
    /// The trades the family knows, by `Group::index`.
    pub known: [bool; N_GROUPS],
    /// The family's first trade: apprentices learn it by default.
    pub first_trade: Group,
    /// This year so far.
    pub income: Income,
    /// The last complete year.
    pub last_year: Income,
    /// Dividends and rents of the last complete month (the base of the spending rule).
    pub month_income: f64,
    /// Net worth at the last year end.
    pub net_worth: f64,
    pub decisions: Vec<Decision>,
    /// Branch stances set by the family; Grow where none is set.
    #[serde(skip)]
    pub(crate) stances: BTreeMap<(Group, usize), Stance>,
    #[serde(skip)]
    pub(crate) month_acc: f64,
    /// Dividends by (trade, town) this year and last year.
    #[serde(skip)]
    pub(crate) branch_acc: BTreeMap<(Group, usize), f64>,
    #[serde(skip)]
    pub(crate) branch_last: BTreeMap<(Group, usize), f64>,
    #[serde(skip)]
    pub(crate) next_id: u64,
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

/// A works at depreciated capital cost: its capacity, built and on order, at
/// today's capital-goods prices. This is also its book value.
pub fn works_value(st: &State, C: &Content, f: usize) -> f64 {
    (st.f_K[f] + st.f_order_K[f]) * unit_capital_cost(st, C, st.f_method[f], st.f_town[f])
}

/// What a cell costs: `years_purchase` times the rent it paid over the last year,
/// never counted below `land_rent_floor` town wages a month. The rent actually
/// paid, not the map's rent: in the year a town is founded its mean wage is zero
/// and the land bids around it are astronomical (see FAMILIES.md).
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

/// The head, if the family has one alive.
pub fn head(fam: &Family) -> Option<usize> {
    fam.members.iter().position(|m| m.alive && m.post == Post::Head)
}

fn adult(m: &Member) -> bool {
    m.alive && m.age >= ADULT_AGE
}

/// A member's skill in a trade: natural managers (apprenticed in it) are a quarter better.
fn skill_in(m: &Member, g: Group) -> f64 {
    if m.trade == Some(g) { (1.25 * m.skill).min(1.0) } else { m.skill }
}

/// Whether the family has a branch of trade g in town t: a manager of g living
/// there, or the seat for every known trade.
pub fn has_branch(fam: &Family, g: Group, t: usize) -> bool {
    (t == fam.seat && fam.known[g.index()])
        || fam.members.iter().any(|m| m.alive && m.town == t && m.post == Post::Manager { group: g })
}

/// Who runs a branch: its manager there, else the head at the seat.
pub fn runner(fam: &Family, g: Group, t: usize) -> Option<usize> {
    let manager = fam
        .members
        .iter()
        .enumerate()
        .filter(|(_, m)| m.alive && m.town == t && m.post == Post::Manager { group: g })
        .max_by(|a, b| skill_in(a.1, g).total_cmp(&skill_in(b.1, g)))
        .map(|(j, _)| j);
    manager.or_else(|| if t == fam.seat && fam.known[g.index()] { head(fam) } else { None })
}

/// The efficiency bonus of a family's works of trade g in town t: a manager's
/// `family_bonus` times skill, or at the seat the head's `family_seat_bonus`
/// times skill, whichever is larger; 0 without a branch or an adult to run it.
pub fn branch_bonus(fam: &Family, P: &Params, g: Group, t: usize) -> f64 {
    let mut b: f64 = 0.0;
    for m in fam.members.iter().filter(|m| adult(m) && m.town == t) {
        match m.post {
            Post::Manager { group } if group == g => b = b.max(P.family_bonus * skill_in(m, g)),
            Post::Head if t == fam.seat && fam.known[g.index()] => b = b.max(P.family_seat_bonus * skill_in(m, g)),
            _ => {}
        }
    }
    b
}

/// Every branch of a family: (trade, town), the seat's first.
pub fn branches(fam: &Family) -> Vec<(Group, usize)> {
    let mut out: Vec<(Group, usize)> = Group::ALL.into_iter().filter(|g| fam.known[g.index()]).map(|g| (g, fam.seat)).collect();
    let mut seen: BTreeSet<(Group, usize)> = out.iter().copied().collect();
    for m in fam.members.iter().filter(|m| m.alive) {
        if let Post::Manager { group } = m.post {
            if seen.insert((group, m.town)) {
                out.push((group, m.town));
            }
        }
    }
    out
}

/// Whether a works may open a sister works next door (`firms::spawn_sister`):
/// anyone's may, except a family works whose branch is not growing.
pub fn may_spawn(st: &State, C: &Content, f: usize) -> bool {
    let Some(o) = owner_of_firm(st, f) else { return true };
    let fam = &st.families[o];
    match Group::of_method(C, st.f_method[f]) {
        Some(g) => has_branch(fam, g, st.f_town[f]) && stance(fam, g, st.f_town[f]) == Stance::Grow,
        None => false,
    }
}

pub fn stance(fam: &Family, g: Group, t: usize) -> Stance {
    fam.stances.get(&(g, t)).copied().unwrap_or(Stance::Grow)
}

/// Per family, (has a branch, bonus) for every (trade, town), `[g * T + t]`.
fn branch_tables(st: &State, P: &Params) -> Vec<(Vec<bool>, Vec<f64>)> {
    let T = st.T_max;
    st.families
        .iter()
        .map(|fam| {
            let mut has = vec![false; N_GROUPS * T];
            let mut bonus = vec![0.0; N_GROUPS * T];
            for (g, t) in branches(fam) {
                has[g.index() * T + t] = true;
                bonus[g.index() * T + t] = branch_bonus(fam, P, g, t);
            }
            (has, bonus)
        })
        .collect()
}

// ---------------------------------------------------------------- the start

fn new_member(rng: &mut Rng, used: &mut BTreeSet<String>, age: i64, year: i64, town: usize, skill: f64) -> Member {
    let first = names::member_name(rng, used);
    let lifespan = (55.0 + 25.0 * random(rng)) as i64;
    let post = if age < TRAINING_AGE { Post::Child } else { Post::Unposted };
    Member {
        first,
        born: year - age,
        age,
        skill,
        alive: true,
        town,
        post,
        educated: false,
        trade: None,
        next: None,
        parent: None,
        lifespan: lifespan.max(age + 2),
    }
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

/// Works' value of each trade in each town, `[g][t]` (all works, anyone's).
pub fn trade_values(st: &State, C: &Content) -> Vec<Vec<f64>> {
    let mut v = vec![vec![0.0; st.T_max]; N_GROUPS];
    for f in 0..st.F_count {
        if st.f_alive[f] {
            if let Some(g) = Group::of_method(C, st.f_method[f]) {
                v[g.index()][st.f_town[f]] += works_value(st, C, f);
            }
        }
    }
    v
}

/// The strongest trade of a town: the most works' value (estates excluded).
fn strongest_trade(st: &State, C: &Content, t: usize) -> Group {
    let v = trade_values(st, C);
    Group::ALL
        .into_iter()
        .filter(|&g| g != Group::Estates)
        .max_by(|a, b| v[a.index()][t].total_cmp(&v[b.index()][t]).then(b.cmp(a)))
        .unwrap_or(Group::Textiles)
}

/// Create the families at the start of the recorded run (after the warm-up).
///
/// Seats go to the families in order of town size. Each family has three to six
/// members: the eldest adult is head, teenagers are apprentices in the first
/// trade at the seat, younger ones children. The player and the landed families
/// get the farmland nearest their seat until they hold `family_start_share` of
/// all rent; landed families take estates as their first trade, industrial
/// families their seat's strongest trade and that trade's largest works there,
/// up to the same share of all works' capital. The player's first trade is a
/// decision, defaulting to the trade with the best prospects at the seat. Every
/// family also gets `family_start_cash` of the money stock, taken from the
/// towns' savers.
pub fn init(st: &mut State, C: &Content, P: &Params, rng: &mut Rng) {
    let n = P.n_families;
    if n == 0 {
        return;
    }
    let year = C.start_year - 1;
    st.f_owner = vec![-1; st.f_alive.len()];
    st.f_paid = vec![0.0; st.f_alive.len()];
    st.cell_owner = vec![-1; st.N];
    st.cell_rent_paid = vec![0.0; st.N];
    st.cell_rent_year = st.cell_rent.iter().map(|&r| 12.0 * r).collect();
    let mut towns: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    towns.sort_by(|&a, &b| st.pop[b].total_cmp(&st.pop[a]));
    let mut used_surnames = BTreeSet::new();
    for k in 0..n {
        let strategy = match k {
            0 => Strategy::Player,
            k if k % 2 == 1 => Strategy::Landed,
            _ => Strategy::Industrial,
        };
        let seat = towns[k % towns.len()];
        let first_trade = if strategy == Strategy::Landed { Group::Estates } else { strongest_trade(st, C, seat) };
        let surname = names::family_surname(rng, &mut used_surnames);
        let n_members = 3 + integers(rng, 4);
        let mut used = BTreeSet::new();
        let mut members = Vec::with_capacity(n_members);
        for j in 0..n_members {
            let age = (if j == 0 { 40 + integers(rng, 21) } else { 5 + integers(rng, 36) }) as i64;
            let skill = 0.2 + 0.7 * random(rng);
            let mut m = new_member(rng, &mut used, age, year, seat, skill);
            if (TRAINING_AGE..ADULT_AGE).contains(&age) {
                let path = Path::Apprentice { group: first_trade, town: seat };
                m.post = Post::Learning { path, until: year + ADULT_AGE - age };
            }
            members.push(m);
        }
        members[0].post = Post::Head;
        let mut known = [false; N_GROUPS];
        known[first_trade.index()] = true;
        st.families.push(Family {
            surname,
            strategy,
            seat,
            next_seat: None,
            cash: 0.0,
            members,
            known,
            first_trade,
            income: Income::default(),
            last_year: Income::default(),
            month_income: 0.0,
            net_worth: 0.0,
            decisions: Vec::new(),
            stances: BTreeMap::new(),
            month_acc: 0.0,
            branch_acc: BTreeMap::new(),
            branch_last: BTreeMap::new(),
            next_id: 0,
        });
    }

    let works_total: f64 = (0..st.F_count).filter(|&f| st.f_alive[f]).map(|f| works_value(st, C, f)).sum();
    let rent_total: f64 = (0..st.N).filter(|&c| st.world.land[c]).map(|c| st.cell_rent_year[c]).sum();
    let each = P.family_start_cash * money_total(st);
    let taken = take_from_savers(st, each * n as f64);
    for k in 0..n {
        st.families[k].cash = taken / n as f64;
    }

    // the player starts as landed gentry, like the landed rivals
    for k in 0..n {
        let seat = st.families[k].seat;
        if st.families[k].strategy != Strategy::Industrial {
            let mut cells: Vec<usize> = (0..st.N)
                .filter(|&c| st.catch[c] == seat as i64 && is_farmland(st, C, c) && st.cell_rent_year[c] > 0.0 && st.cell_owner[c] < 0)
                .collect();
            cells.sort_by(|&a, &b| st.dist[(seat, a)].total_cmp(&st.dist[(seat, b)]));
            let mut held = 0.0;
            for c in cells {
                if held >= P.family_start_share * rent_total {
                    break;
                }
                st.cell_owner[c] = k as i64;
                held += st.cell_rent_year[c];
            }
        } else {
            let g = st.families[k].first_trade;
            let mut works: Vec<usize> = (0..st.F_count)
                .filter(|&f| st.f_alive[f] && st.f_town[f] == seat && st.f_owner[f] < 0 && Group::of_method(C, st.f_method[f]) == Some(g))
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

    for k in 0..n {
        st.families[k].net_worth = net_worth(st, C, P, k);
    }
    // the player's first decision: its trade, defaulting to the best prospects at the seat
    let seat = st.families[0].seat;
    let pr = prospects(st, C, P, seat);
    let default = Group::ALL.into_iter().max_by(|a, b| pr[a.index()].total_cmp(&pr[b.index()]).then(b.cmp(a))).unwrap_or(Group::Textiles);
    st.families[0].first_trade = default;
    st.families[0].known = [false; N_GROUPS];
    st.families[0].known[default.index()] = true;
    for m in st.families[0].members.iter_mut() {
        if let Post::Learning { path: Path::Apprentice { group, .. }, .. } = &mut m.post {
            *group = default;
        }
    }
    let options: Vec<DecisionOption> =
        Group::ALL.iter().map(|&g| DecisionOption { label: g.label().to_string(), choice: Choice::Trade { group: g } }).collect();
    let prompt = format!("Which trade does the {} family start in, at {}?", st.families[0].surname, st.town_name[seat]);
    // due at the end of the second year: a viewer runs the first year before the
    // player sees the map; until it is answered the player's branches hold
    raise(st, 0, prompt, None, options, default.index(), C.start_year + 1);
}

/// Farmland, pasture or woodland (not urban, industrial or derelict).
fn is_farmland(st: &State, C: &Content, c: usize) -> bool {
    if !st.world.land[c] || st.urban[c] {
        return false;
    }
    let u = st.land_use[c];
    u >= USE_SECTOR0 && ((u - USE_SECTOR0) as usize) < C.NS && Group::of_sector(C, (u - USE_SECTOR0) as usize) == Some(Group::Estates)
}

// ---------------------------------------------------------------- the month (econ::step_month)

/// Each family's luxury budget this month (empty without families): a share of
/// last month's income and a share of its cash, never more than its cash. Land
/// and works are not drawn on: a landed family saves what it does not spend of
/// its rents. The budgets are spent through the seat towns' landlords.
pub fn spending_budgets(st: &State, P: &Params) -> Vec<f64> {
    st.families.iter().map(|f| f.cash.max(0.0).min(monthly_spending(f, P))).collect()
}

/// What a family means to spend in a month: `family_cons_income` of last month's
/// income and `family_cons_wealth` of its cash.
fn monthly_spending(f: &Family, P: &Params) -> f64 {
    P.family_cons_income * f.month_income.max(0.0) + P.family_cons_wealth * f.cash.max(0.0)
}

fn net_worth(st: &State, C: &Content, P: &Params, k: usize) -> f64 {
    let works: f64 = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_owner[f] == k as i64).map(|f| works_value(st, C, f)).sum();
    let land: f64 = (0..st.N).filter(|&c| st.cell_owner[c] == k as i64).map(|c| land_price(st, P, c)).sum();
    st.families[k].cash + works + land
}

/// The budgets added up by seat town.
pub fn budgets_by_town(st: &State, budgets: &[f64], T: usize) -> Vec<f64> {
    let mut out = vec![0.0; T];
    for (f, b) in st.families.iter().zip(budgets) {
        out[f.seat] += b;
    }
    out
}

/// Families pay their part of what their seat town's landlords bought this
/// month: the landlords paid the merchants for everything, the families repay
/// them in proportion to the budget each put in.
pub fn pay_spending(st: &mut State, budgets: &[f64], lord_budget: &[f64], lord_spend: &[f64]) {
    for (k, &b) in budgets.iter().enumerate() {
        let seat = st.families[k].seat;
        if b <= 0.0 || lord_budget[seat] <= 0.0 {
            continue;
        }
        let pay = lord_spend[seat] * b / lord_budget[seat];
        let fam = &mut st.families[k];
        fam.cash -= pay;
        fam.income.spending += pay;
        st.lord_cash[seat] += pay;
    }
}

/// Dividends of family works go to the family; their entries in `div` are zeroed
/// so the rest still goes to the households of each town.
pub fn take_dividends(st: &mut State, C: &Content, fi: &[usize], div: &mut [f64]) {
    for (k, &f) in fi.iter().enumerate() {
        let Some(o) = owner_of_firm(st, f) else { continue };
        let d = div[k];
        let group = Group::of_method(C, st.f_method[f]);
        let town = st.f_town[f];
        st.f_paid[f] += d;
        let fam = &mut st.families[o];
        fam.cash += d;
        fam.month_acc += d;
        match group {
            Some(g) => {
                fam.income.dividends[g.index()] += d;
                *fam.branch_acc.entry((g, town)).or_insert(0.0) += d;
            }
            None => fam.income.other_dividends += d,
        }
        div[k] = 0.0;
    }
}

/// A family works with no branch over it runs under a hired manager, who takes
/// `absentee_cut` of its profit (paid to the town's households). Returns the cut
/// for each of the firms `fi`, or None without families.
pub fn hired_cut(st: &State, C: &Content, P: &Params, fi: &[usize]) -> Option<Vec<f64>> {
    if !active(st) {
        return None;
    }
    let tables = branch_tables(st, P);
    let T = st.T_max;
    Some(
        fi.iter()
            .map(|&f| match owner_of_firm(st, f) {
                Some(o) => match Group::of_method(C, st.f_method[f]) {
                    Some(g) if tables[o].0[g.index() * T + st.f_town[f]] => 0.0,
                    _ => P.absentee_cut,
                },
                None => 0.0,
            })
            .collect(),
    )
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

/// Family works under a branch need `1 - bonus` of the inputs, heat and labor
/// per unit that other works need.
pub fn scale_firm_inputs(st: &State, C: &Content, P: &Params, fi: &[usize], a: &mut Mat, h: &mut [f64], l: &mut [f64]) {
    let tables = branch_tables(st, P);
    let T = st.T_max;
    for (k, &f) in fi.iter().enumerate() {
        let Some(o) = owner_of_firm(st, f) else { continue };
        let Some(g) = Group::of_method(C, st.f_method[f]) else { continue };
        let b = tables[o].1[g.index() * T + st.f_town[f]];
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

/// Productivity of the land units on family land under a branch of their trade
/// goes up by the bonus (None without families).
pub fn land_boost(st: &State, C: &Content, P: &Params) -> Option<Vec<f64>> {
    if !active(st) {
        return None;
    }
    let tables = branch_tables(st, P);
    let T = st.T_max;
    Some(
        (0..st.ex_cell.len())
            .map(|u| match (owner_of_cell(st, st.ex_cell[u]), Group::of_sector(C, st.ex_sector[u])) {
                (Some(o), Some(g)) => 1.0 + tables[o].1[g.index() * T + st.ex_town[u]],
                _ => 1.0,
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

// ---------------------------------------------------------------- site sums

/// The sums behind building one method on one cell, per unit of output unless
/// stated: the same numbers `firms::evaluate_sites` gives entry, with the
/// family's bonus for a branch there applied.
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
    /// Expected yearly return on the capacity's cost (what entry compares with the
    /// hurdle rate): a snapshot at last year's prices and 85 per cent utilisation.
    pub expected_return: f64,
    /// The family's bonus included in these sums.
    pub bonus: f64,
    /// Agglomeration multiplier from works of the same kind nearby.
    pub agglo: f64,
    /// What the works would make in a month at 85 per cent of capacity.
    pub output: f64,
    /// What the town's buyers asked for of that good last month, got or not.
    pub town_demand: f64,
    /// What the town made of it last year, per month.
    pub town_output: f64,
    /// The new works' output over the town's demand: above a few tenths the works
    /// would flood its market and its price.
    pub market_share: f64,
    /// The town's mean price of the good last year over its base price: far above
    /// 1 the good is scarce, and the price will not last once someone makes it.
    pub price_ratio: f64,
}

/// One method on one cell for one family.
#[derive(Clone, Debug, Serialize)]
pub struct SiteOption {
    pub method: usize,
    pub name: String,
    pub label: String,
    pub group: Option<Group>,
    pub sums: Result<SiteSums, String>,
    /// Whether `found_works` would build it (cash aside), or why not.
    pub allowed: Result<(), String>,
}

/// How deep the market for method m's output is in town i, for a works of
/// capacity K: (output a month, town demand a month, town output a month,
/// market share, price ratio). The price ratio is the good's price over its base
/// price, relative to the same ratio for the town's typical good (the median over
/// goods), so a general rise in prices does not count as scarcity.
pub fn market_depth(st: &State, C: &Content, i: usize, m: usize, K: f64) -> (f64, f64, f64, f64, f64) {
    let g = C.method_out[m];
    let output = 0.85 * K;
    let demand = if st.requests.rows > 0 { st.requests[(i, g)] } else { 0.0 };
    let made = if st.town_prod_year.rows > 0 { st.town_prod_year[(i, g)] / 12.0 } else { 0.0 };
    let share = (output / demand.max(1e-9)).min(1e6);
    let p = prices(st);
    let mut ratios: Vec<f64> = (0..C.G).map(|x| p[(i, x)] / C.base_price[x]).collect();
    ratios.sort_by(|a, b| a.total_cmp(b));
    let typical = ratios[ratios.len() / 2].max(1e-9);
    (output, demand, made, share, p[(i, g)] / C.base_price[g] / typical)
}

/// The best expected return each trade offers in town t on any cell in its
/// reach, where the market is deep enough for a family to build
/// (`family_max_share`, `family_max_price_ratio`); estates offer the land
/// yield, one over `years_purchase`.
pub fn prospects(st: &State, C: &Content, P: &Params, t: usize) -> [f64; N_GROUPS] {
    prospect_works(st, C, P, t).map(|p| p.roi)
}

/// What the best works of a trade in a town would be: its expected yearly
/// return, and what founding it would cost (zero for estates, which have no
/// works). A return is only as big as the works it is on, so the cost puts it
/// in proportion.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Prospect {
    pub roi: f64,
    pub cost: f64,
}

/// `prospects`, with the cost of founding the works each return is for.
pub fn prospect_works(st: &State, C: &Content, P: &Params, t: usize) -> [Prospect; N_GROUPS] {
    let mut best = [Prospect { roi: f64::NEG_INFINITY, cost: 0.0 }; N_GROUPS];
    best[Group::Estates.index()].roi = 1.0 / P.years_purchase;
    let cells: Vec<usize> = (0..st.N)
        .filter(|&c| st.catch[c] == t as i64 && st.cell_K_share[c] < 0.9 && st.land_use[c] != 0 && st.land_use[c] != USE_DERELICT)
        .collect();
    if cells.is_empty() {
        return best;
    }
    for m in 0..C.M {
        let Some(g) = Group::of_method(C, m) else { continue };
        if !(st.know[(t, m)] > 0.02) {
            continue;
        }
        let K = P.entry_K_frac * C.m_K0[m];
        let (_, _, _, share, ratio) = market_depth(st, C, t, m, K);
        if share > P.family_max_share || ratio > P.family_max_price_ratio {
            continue;
        }
        let ev = firms::evaluate_sites(st, C, P, t, m, &cells, K, prices(st), wages(st));
        let r = ev.roi.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if r > best[g.index()].roi {
            // what found_works would charge: the capacity at capital-goods prices, plus 15 per cent
            best[g.index()] = Prospect { roi: r, cost: ev.cap_cost * 1.15 };
        }
    }
    best
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
    let b = match Group::of_method(C, m) {
        Some(g) => branch_bonus(&st.families[family], P, g, i),
        None => 0.0,
    };
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

#[allow(clippy::too_many_arguments)]
fn sums_of(st: &State, C: &Content, m: usize, ev: &SiteEval, b: f64, K: f64, i: usize, wage: f64) -> SiteSums {
    let (output, town_demand, town_output, market_share, price_ratio) = market_depth(st, C, i, m, K);
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
        output,
        town_demand,
        town_output,
        market_share,
        price_ratio,
    }
}

/// Whether the player may found method m in town i directly: only at the seat,
/// only in a known trade.
fn may_found(st: &State, C: &Content, family: usize, m: usize, i: usize) -> Result<(), String> {
    let fam = &st.families[family];
    let Some(g) = Group::of_method(C, m) else {
        return Err(format!("{} is not a family trade", C.method_label[m]));
    };
    if !fam.known[g.index()] {
        return Err(format!("the {} family does not know {} yet", fam.surname, g.label().to_lowercase()));
    }
    if i != fam.seat {
        return Err(format!("works are chosen directly only at the seat, {}", st.town_name[fam.seat]));
    }
    Ok(())
}

/// For every method, what building it on `cell` would look like for `family`,
/// and whether `found_works` would build it there. Errors if nothing can be built
/// on the cell.
pub fn site_options(st: &State, C: &Content, P: &Params, family: usize, cell: usize) -> Result<Vec<SiteOption>, String> {
    check_family(st, family)?;
    let i = buildable(st, cell)?;
    Ok((0..C.M)
        .map(|m| {
            let known_here = st.know[(i, m)] > 0.02;
            let sums = if !known_here {
                Err(format!("{} is not known in {} yet", C.method_label[m], st.town_name[i]))
            } else {
                let (ev, b) = evaluate(st, C, P, family, m, cell, i);
                Ok(sums_of(st, C, m, &ev, b, P.entry_K_frac * C.m_K0[m], i, wages(st)[i]))
            };
            let allowed = match &sums {
                Err(e) => Err(e.clone()),
                Ok(_) => may_found(st, C, family, m, i),
            };
            SiteOption { method: m, name: C.method_name[m].clone(), label: C.method_label[m].clone(), group: Group::of_method(C, m), sums, allowed }
        })
        .collect())
}

/// Build method m on `cell` for `family`, paid from the family's cash, as entry
/// builds (capacity `entry_K_frac` of the method's full size). The player may
/// do this only at the seat and only in a known trade; branches growing do it
/// for themselves anywhere they are. Returns the new firm's id.
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
    may_found(st, C, family, method, i)?;
    found_at(st, C, P, family, method, cell, year, events)
}

#[allow(clippy::too_many_arguments)]
fn found_at(st: &mut State, C: &Content, P: &Params, family: usize, method: usize, cell: usize, year: i64, events: &mut Vec<Event>) -> Result<usize, String> {
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
    let first_of_kind = !(0..st.F_count).any(|f| st.f_owner[f] == family as i64 && st.f_method[f] == method);
    let reason = firms::reason_record(st, i, method, cell, &ev, 0, K, year, "family");
    // the name is given, so create_firm draws nothing from this stream
    let mut unused = Rng::from_seed([0; 32]);
    st.families[family].cash -= cost;
    st.families[family].income.invested += cost;
    let fid = firms::create_firm(st, C, P, &mut unused, i, method, cell, K, ev.rent_bid[0], reason, year, cost, Some(name), None)
        .expect("checked that the firm table has room");
    st.f_owner[fid] = family as i64;
    st.new_firms_year.push(fid);
    // the player hears of every works of its own, and of a rival's first of each kind
    if family == 0 || first_of_kind {
        let fam = &st.families[family];
        let what = C.method_label[method].to_lowercase();
        let title = if family == 0 {
            format!("The {} family opens {} {} at {}", fam.surname, article(&what), what, st.town_name[i])
        } else {
            format!("The {} family opens its first {} at {}", fam.surname, what, st.town_name[i])
        };
        let text = format!("{} is built for {:.0} and expects to return {:.0} per cent a year.", st.f_name[fid], cost, 100.0 * ev.roi[0]);
        let mut e = Event::new(year as f64 + 0.95, "family", if family == 0 { 0.6 } else { 0.35 }, title, text);
        e.town = Some(i);
        e.cell = Some(cell);
        e.firm = Some(fid);
        e.family = Some(family);
        events.push(e);
    }
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

/// Whether `family` may buy `cell` directly (cash aside, like `SiteOption::allowed`):
/// land is bought directly only in the seat's reach; growing estates branches buy
/// it elsewhere.
pub fn may_buy_land(st: &State, family: usize, cell: usize) -> Result<(), String> {
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
    let fam = &st.families[family];
    if owner_of_cell(st, cell) == Some(family) {
        return Err(format!("the {} family already owns this land", fam.surname));
    }
    if t as usize != fam.seat {
        return Err(format!(
            "land is bought directly only around the seat, {}; a growing estates branch buys it elsewhere",
            st.town_name[fam.seat]
        ));
    }
    Ok(())
}

/// Buy `cell` for `family` at `land_price`, only in the seat's reach
/// (`may_buy_land`). The seller is paid: another family, or, for land of the
/// anonymous gentry, whoever its rent goes to now (the landlords of its town, or
/// the common fund under the commons regime). Returns the price paid.
pub fn buy_land(st: &mut State, _C: &Content, P: &Params, family: usize, cell: usize) -> Result<f64, String> {
    may_buy_land(st, family, cell)?;
    buy_at(st, P, family, cell)
}

/// A purchase anywhere in a town's reach (estates branches use this directly).
fn buy_at(st: &mut State, P: &Params, family: usize, cell: usize) -> Result<f64, String> {
    let t = st.catch[cell];
    if cell >= st.N || !st.world.land[cell] || t < 0 {
        return Err("the cell cannot be bought".to_string());
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
            st.families[j].income.sales += price;
        }
        None => match P.rent_regime {
            RentRegime::Landlords => st.lord_cash[t as usize] += price,
            RentRegime::Commons => st.fund_cash += price,
        },
    }
    st.cell_owner[cell] = family as i64;
    Ok(price)
}

// ---------------------------------------------------------------- the player's choices

/// Post an adult who is not training (and is not the head) as manager of a known
/// trade in a town, or unpost them. A move to another town takes effect at the
/// next year end.
pub fn post_member(st: &mut State, family: usize, member: usize, posting: Posting) -> Result<(), String> {
    check_family(st, family)?;
    let T = st.T_max;
    let town_ok = |t: usize| t < T && st.town_active[t];
    let fam = &st.families[family];
    let Some(m) = fam.members.get(member) else {
        return Err(format!("the {} family has no member {member}", fam.surname));
    };
    let who = format!("{} {}", m.first, fam.surname);
    if !m.alive {
        return Err(format!("{who} is dead"));
    }
    if m.age < ADULT_AGE {
        return Err(format!("{who} is {} and too young for a post", m.age));
    }
    if let Post::Learning { until, .. } = m.post {
        return Err(format!("{who} is training until {until}"));
    }
    if m.post == Post::Head {
        return Err(format!("{who} heads the family and lives at the seat; move the seat instead"));
    }
    let (town, post) = match posting {
        Posting::Unposted => (m.town, Post::Unposted),
        Posting::Manager { group, town } => {
            if !fam.known[group.index()] {
                return Err(format!("the {} family does not know {} yet", fam.surname, group.label().to_lowercase()));
            }
            if !town_ok(town) {
                return Err(format!("there is no town {town}"));
            }
            (town, Post::Manager { group })
        }
    };
    let m = &mut st.families[family].members[member];
    if town == m.town {
        m.post = post;
        m.next = None;
    } else {
        m.next = Some((town, post));
    }
    Ok(())
}

/// Set what a branch does each year.
pub fn set_stance(st: &mut State, family: usize, group: Group, town: usize, stance: Stance) -> Result<(), String> {
    check_family(st, family)?;
    let fam = &mut st.families[family];
    if !has_branch(fam, group, town) {
        return Err(format!("the {} family has no {} branch there", fam.surname, group.label().to_lowercase()));
    }
    fam.stances.insert((group, town), stance);
    Ok(())
}

/// Move the family's seat (and its head) to another town at the next year end.
pub fn move_seat(st: &mut State, family: usize, town: usize) -> Result<(), String> {
    check_family(st, family)?;
    if town >= st.T_max || !st.town_active[town] {
        return Err(format!("there is no town {town}"));
    }
    let fam = &mut st.families[family];
    fam.next_seat = if town == fam.seat { None } else { Some(town) };
    Ok(())
}

/// The decisions waiting for a family.
pub fn pending(st: &State, family: usize) -> Vec<Decision> {
    st.families.get(family).map(|f| f.decisions.clone()).unwrap_or_default()
}

fn raise(st: &mut State, k: usize, prompt: String, member: Option<usize>, options: Vec<DecisionOption>, default: usize, due: i64) {
    let fam = &mut st.families[k];
    let id = fam.next_id;
    fam.next_id += 1;
    fam.decisions.push(Decision { id, prompt, member, options, default, due });
}

/// Answer decision `id` with option `option`.
pub fn decide(st: &mut State, C: &Content, family: usize, id: u64, option: usize) -> Result<(), String> {
    check_family(st, family)?;
    let fam = &st.families[family];
    let Some(pos) = fam.decisions.iter().position(|d| d.id == id) else {
        return Err(format!("the {} family has no decision {id}", fam.surname));
    };
    let d = &fam.decisions[pos];
    let Some(opt) = d.options.get(option) else {
        return Err(format!("decision {id} has {} options", d.options.len()));
    };
    let (choice, member, due) = (opt.choice, d.member, d.due);
    st.families[family].decisions.remove(pos);
    apply_choice(st, C, family, choice, member, due - 1, &mut Vec::new());
    Ok(())
}

/// Carry out a choice. `year` is the year the decision was raised in.
fn apply_choice(st: &mut State, C: &Content, k: usize, choice: Choice, member: Option<usize>, year: i64, events: &mut Vec<Event>) {
    let seat = st.families[k].seat;
    let fam = &mut st.families[k];
    match choice {
        Choice::Trade { group } => {
            // the first trade: replaces the provisional one if nothing else is known yet,
            // and the teenagers apprenticed in it at the seat switch with it
            let old = fam.first_trade;
            if fam.known.iter().filter(|&&x| x).count() <= 1 {
                fam.known = [false; N_GROUPS];
            }
            fam.known[group.index()] = true;
            fam.first_trade = group;
            for m in fam.members.iter_mut() {
                if let Post::Learning { path: Path::Apprentice { group: g, town }, .. } = &mut m.post {
                    if *g == old && *town == seat {
                        *g = group;
                    }
                }
            }
            // training decisions already waiting offer the new first trade instead
            let waiting: Vec<usize> = (0..fam.decisions.len()).filter(|&i| fam.decisions[i].member.is_some()).collect();
            if !waiting.is_empty() {
                let values = trade_values(st, C);
                for &i in &waiting {
                    st.families[k].decisions[i].options.clear();
                }
                for i in waiting {
                    let options = training_options(st, &values, k);
                    let default = training_default(st, &values, k, &options);
                    let d = &mut st.families[k].decisions[i];
                    d.options = options;
                    d.default = default;
                }
            }
        }
        Choice::Apprentice { .. } | Choice::University => {
            let Some(j) = member else { return };
            let m = &mut fam.members[j];
            if !m.alive || m.age < TRAINING_AGE || m.post != Post::Child {
                return;
            }
            let path = match choice {
                Choice::Apprentice { group, town } => Path::Apprentice { group, town },
                _ => Path::University,
            };
            m.post = Post::Learning { path, until: year + 1 + TRAINING_YEARS };
            if let Path::Apprentice { town, .. } = path {
                m.town = town;
            }
        }
        Choice::Head { member: j } => {
            if j >= fam.members.len() || !fam.members[j].alive || fam.members[j].post == Post::Head {
                return;
            }
            for m in fam.members.iter_mut() {
                if m.post == Post::Head {
                    m.post = Post::Unposted;
                }
            }
            let m = &mut fam.members[j];
            m.post = Post::Head;
            m.town = seat;
            m.next = None;
            let _ = events;
        }
    }
}

// ---------------------------------------------------------------- reports

#[derive(Clone, Debug, Serialize)]
pub struct MemberReport {
    pub index: usize,
    pub first: String,
    /// Year of birth.
    pub born: i64,
    pub age: i64,
    pub skill: f64,
    pub alive: bool,
    pub town: usize,
    pub town_name: String,
    pub post: Post,
    /// The year training ends, while training.
    pub training_until: Option<i64>,
    pub educated: bool,
    pub trade: Option<Group>,
    /// A move at the next year end: (town, post).
    pub next: Option<(usize, Post)>,
    /// The parent (a member index), None for the founding generation.
    pub parent: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BranchReport {
    pub group: Group,
    pub town: usize,
    pub town_name: String,
    /// The member running it (the head at the seat, unless a manager lives there too).
    pub runner: Option<usize>,
    pub stance: Stance,
    /// The bonus its works get.
    pub bonus: f64,
    /// The family's works of this trade in this town (for estates: none).
    pub works: Vec<usize>,
    /// For estates: the family's cells in this town's reach.
    pub cells: usize,
    /// Dividends its works paid last year.
    pub last_year_profit: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct FamilyReport {
    pub index: usize,
    pub surname: String,
    pub player: bool,
    pub strategy: Strategy,
    pub seat: usize,
    pub seat_name: String,
    pub next_seat: Option<usize>,
    pub head: Option<usize>,
    pub known: Vec<Group>,
    pub first_trade: Group,
    pub cash: f64,
    /// The family's works at depreciated capital cost (capacity built and on
    /// order, at today's capital-goods prices).
    pub works_value: f64,
    /// The family's land at `land_price`.
    pub land_value: f64,
    pub net_worth: f64,
    /// The last complete year: dividends by trade, rents, sales, spending, investment.
    pub last_year: Income,
    /// Alive works the family owns (firm ids).
    pub firms: Vec<usize>,
    /// Of them, those with no branch over them (run by hired managers).
    pub absentee: Vec<usize>,
    /// Cells the family owns.
    pub cells: Vec<usize>,
    pub members: Vec<MemberReport>,
    pub branches: Vec<BranchReport>,
    pub pending: Vec<Decision>,
}

pub fn family_report(st: &State, C: &Content, P: &Params, family: usize) -> Result<FamilyReport, String> {
    check_family(st, family)?;
    let fam = &st.families[family];
    let firms: Vec<usize> = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_owner[f] == family as i64).collect();
    let cells: Vec<usize> = (0..st.N).filter(|&c| st.cell_owner[c] == family as i64).collect();
    let works_value: f64 = firms.iter().map(|&f| works_value(st, C, f)).sum();
    let land_value: f64 = cells.iter().map(|&c| land_price(st, P, c)).sum();
    let absentee: Vec<usize> = firms
        .iter()
        .copied()
        .filter(|&f| Group::of_method(C, st.f_method[f]).is_none_or(|g| !has_branch(fam, g, st.f_town[f])))
        .collect();
    let branch_reports = branches(fam)
        .into_iter()
        .map(|(g, t)| BranchReport {
            group: g,
            town: t,
            town_name: st.town_name[t].clone(),
            runner: runner(fam, g, t),
            stance: stance(fam, g, t),
            bonus: branch_bonus(fam, P, g, t),
            works: firms.iter().copied().filter(|&f| st.f_town[f] == t && Group::of_method(C, st.f_method[f]) == Some(g)).collect(),
            cells: if g == Group::Estates { cells.iter().filter(|&&c| st.catch[c] == t as i64).count() } else { 0 },
            last_year_profit: fam.branch_last.get(&(g, t)).copied().unwrap_or(0.0),
        })
        .collect();
    Ok(FamilyReport {
        index: family,
        surname: fam.surname.clone(),
        player: family == 0,
        strategy: fam.strategy,
        seat: fam.seat,
        seat_name: st.town_name[fam.seat].clone(),
        next_seat: fam.next_seat,
        head: head(fam),
        known: Group::ALL.into_iter().filter(|g| fam.known[g.index()]).collect(),
        first_trade: fam.first_trade,
        cash: fam.cash,
        works_value,
        land_value,
        net_worth: fam.cash + works_value + land_value,
        last_year: fam.last_year.clone(),
        firms,
        absentee,
        cells,
        members: fam
            .members
            .iter()
            .enumerate()
            .map(|(index, m)| MemberReport {
                index,
                first: m.first.clone(),
                born: m.born,
                age: m.age,
                skill: m.skill,
                alive: m.alive,
                town: m.town,
                town_name: st.town_name[m.town].clone(),
                post: m.post,
                training_until: if let Post::Learning { until, .. } = m.post { Some(until) } else { None },
                educated: m.educated,
                trade: m.trade,
                next: m.next,
                parent: m.parent,
            })
            .collect(),
        branches: branch_reports,
        pending: fam.decisions.clone(),
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

/// A family's year end: overdue decisions take their defaults, moves happen,
/// members age, train, die and are born, AI families choose, branches grow,
/// hold or wind down, and the books close.
pub fn yearly(st: &mut State, C: &Content, P: &Params, rng: &mut Rng, year: i64, events: &mut Vec<Event>) {
    if !active(st) {
        return;
    }
    // the year's rents are in: land is priced on them from now on
    st.cell_rent_year = std::mem::replace(&mut st.cell_rent_paid, vec![0.0; st.N]);
    let values = trade_values(st, C);
    for k in 0..st.families.len() {
        take_defaults(st, C, k, year, events);
        moves(st, k);
        people(st, C, P, rng, k, year, events);
        if k != 0 {
            ai_choose(st, C, k, year, &values, events);
        }
        run_branches(st, C, P, rng, k, year, events);
        st.families[k].net_worth = net_worth(st, C, P, k);
        let fam = &mut st.families[k];
        fam.last_year = std::mem::take(&mut fam.income);
        fam.branch_last = std::mem::take(&mut fam.branch_acc);
    }
}

fn take_defaults(st: &mut State, C: &Content, k: usize, year: i64, events: &mut Vec<Event>) {
    let due: Vec<Decision> = st.families[k].decisions.iter().filter(|d| d.due <= year).cloned().collect();
    st.families[k].decisions.retain(|d| d.due > year);
    for d in due {
        let choice = d.options[d.default].choice;
        apply_choice(st, C, k, choice, d.member, d.due - 1, events);
    }
}

fn moves(st: &mut State, k: usize) {
    let fam = &mut st.families[k];
    if let Some(s) = fam.next_seat.take() {
        fam.seat = s;
        for m in fam.members.iter_mut().filter(|m| m.alive && m.post == Post::Head) {
            m.town = s;
            m.next = None;
        }
    }
    for m in fam.members.iter_mut().filter(|m| m.alive) {
        if let Some((town, post)) = m.next.take() {
            m.town = town;
            m.post = post;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn people(st: &mut State, C: &Content, P: &Params, rng: &mut Rng, k: usize, year: i64, events: &mut Vec<Event>) {
    let seat = st.families[k].seat;
    let surname = st.families[k].surname.clone();
    let mut news_items: Vec<Option<Event>> = Vec::new();
    let mut raises: Vec<(String, usize)> = Vec::new();
    let mut head_died = false;
    {
        let fam = &mut st.families[k];
        // ageing and deaths
        for m in fam.members.iter_mut().filter(|m| m.alive) {
            m.age += 1;
            if m.age >= m.lifespan {
                m.alive = false;
                let was_head = m.post == Post::Head;
                head_died |= was_head;
                let ran = match m.post {
                    Post::Head => " head of the family,".to_string(),
                    Post::Manager { group } => format!(" who ran the family's {} at {},", group.label().to_lowercase(), st.town_name[m.town]),
                    _ => String::new(),
                };
                m.post = Post::Unposted;
                m.next = None;
                let ai = if was_head { Some(0.3) } else { None };
                news_items.push(news_at(k, ai, year, format!("{} {} dies at {}", m.first, surname, m.age), format!("{} {},{} is buried at {}.", m.first, surname, ran, st.town_name[m.town]), m.town));
            }
        }
    }
    // training ends
    for j in 0..st.families[k].members.len() {
        let m = &st.families[k].members[j];
        let Post::Learning { path, until } = m.post else { continue };
        if !m.alive || until > year {
            continue;
        }
        let first = m.first.clone();
        match path {
            Path::Apprentice { group, town } => {
                let new_trade = !st.families[k].known[group.index()];
                let fam = &mut st.families[k];
                fam.known[group.index()] = true;
                let m = &mut fam.members[j];
                m.trade = Some(group);
                m.town = town;
                m.post = Post::Manager { group };
                news_items.push(news_at(
                    k,
                    None,
                    year,
                    format!("{first} {surname} finishes an apprenticeship in {} at {}", group.label().to_lowercase(), st.town_name[town]),
                    format!("{first} now runs the family's {} there.", group.label().to_lowercase()),
                    town,
                ));
                if new_trade {
                    news_items.push(news_at(
                        k,
                        Some(0.3),
                        year,
                        format!("The {surname} family takes up {}", group.label().to_lowercase()),
                        format!("{first} {surname} brings the trade home from {}.", st.town_name[town]),
                        town,
                    ));
                }
            }
            Path::University => {
                let m = &mut st.families[k].members[j];
                m.skill = (m.skill + P.university_gain).min(1.0);
                m.educated = true;
                m.post = Post::Unposted;
                let town = m.town;
                news_items.push(news_at(k, None, year, format!("{first} {surname} comes down from university"), format!("{first} is educated and waits for a post."), town));
            }
        }
    }
    // coming of age: a training to choose; adults never trained get no post
    for j in 0..st.families[k].members.len() {
        let m = &st.families[k].members[j];
        if !m.alive || m.post != Post::Child {
            continue;
        }
        if m.age >= ADULT_AGE {
            st.families[k].members[j].post = Post::Unposted;
        } else if m.age >= TRAINING_AGE && !st.families[k].decisions.iter().any(|d| d.member == Some(j)) {
            let first = m.first.clone();
            let town = m.town;
            news_items.push(news_at(k, None, year, format!("{first} {surname} turns fourteen"), format!("{first} is old enough to be trained."), town));
            raises.push((first, j));
        }
    }
    // births
    let living = st.families[k].members.iter().filter(|m| m.alive).count();
    if living < FULL_FAMILY {
        let parents: Vec<usize> = (0..st.families[k].members.len())
            .filter(|&j| {
                let m = &st.families[k].members[j];
                m.alive && (22..=42).contains(&m.age)
            })
            .collect();
        let mut born = 0;
        for j in parents {
            if living + born >= FULL_FAMILY || random(rng) >= P.family_birth_rate {
                continue;
            }
            let (pskill, ptown) = (st.families[k].members[j].skill, st.families[k].members[j].town);
            let mut used: BTreeSet<String> = st.families[k].members.iter().filter(|m| m.alive).map(|m| m.first.clone()).collect();
            let skill = 0.5 * pskill + 0.5 * (0.2 + 0.7 * random(rng));
            let mut child = new_member(rng, &mut used, 0, year, ptown, skill);
            child.parent = Some(j);
            news_items.push(news_at(k, None, year, format!("{} is born to the {surname} family", child.first), format!("A child at {}.", st.town_name[ptown]), ptown));
            st.families[k].members.push(child);
            born += 1;
        }
    }
    // nobody left: a cousin of age takes the name on
    if st.families[k].members.iter().all(|m| !m.alive) {
        let mut used = BTreeSet::new();
        let skill = 0.2 + 0.7 * random(rng);
        let mut m = new_member(rng, &mut used, ADULT_AGE, year, seat, skill);
        m.post = Post::Head;
        st.families[k].members.push(m);
        head_died = false;
    }
    // succession: the eldest adult (or the eldest of all) heads at once; the player may choose another
    if head_died || head(&st.families[k]).is_none() {
        let fam = &st.families[k];
        let mut cands: Vec<usize> = (0..fam.members.len()).filter(|&j| adult(&fam.members[j]) && !matches!(fam.members[j].post, Post::Learning { .. })).collect();
        if cands.is_empty() {
            cands = (0..fam.members.len()).filter(|&j| fam.members[j].alive).collect();
        }
        cands.sort_by(|&a, &b| fam.members[b].age.cmp(&fam.members[a].age));
        if let Some(&eldest) = cands.first() {
            apply_choice(st, C, k, Choice::Head { member: eldest }, None, year, events);
            let first = st.families[k].members[eldest].first.clone();
            news_items.push(news_at(k, Some(0.3), year, format!("{first} {surname} heads the family"), format!("The {surname}s are led from {}.", st.town_name[seat]), seat));
            if k == 0 && cands.len() > 1 {
                let fam = &st.families[k];
                let options: Vec<DecisionOption> = cands
                    .iter()
                    .map(|&j| DecisionOption {
                        label: format!("{} {}, {}", fam.members[j].first, surname, fam.members[j].age),
                        choice: Choice::Head { member: j },
                    })
                    .collect();
                raise(st, k, format!("Who heads the {surname} family?"), None, options, 0, year + 1);
            }
        }
    }
    if !raises.is_empty() {
        let values = trade_values(st, C);
        for (first, j) in raises {
            if k == 0 {
                let options = training_options(st, &values, k);
                let default = training_default(st, &values, k, &options);
                raise(st, k, format!("What should {first} {surname} learn?"), Some(j), options, default, year + 1);
            } else {
                raises_ai(st, C, &values, k, j, year);
            }
        }
    }
    events.extend(news_items.into_iter().flatten());
}

fn news_at(k: usize, ai: Option<f64>, year: i64, title: String, text: String, town: usize) -> Option<Event> {
    let w = if k == 0 { 0.6 } else { ai? };
    let mut e = Event::new(year as f64 + 0.95, "family", w, title, text);
    e.town = Some(town);
    e.family = Some(k);
    Some(e)
}

/// Training options: an apprenticeship in the family's first trade at the seat,
/// in every other trade where it is strongest and the family has no branch of it
/// yet (nor an apprentice), and university.
fn training_options(st: &State, values: &[Vec<f64>], k: usize) -> Vec<DecisionOption> {
    let fam = &st.families[k];
    let taken = places_in_hand(fam);
    let mut out = vec![DecisionOption {
        label: format!("Apprentice in {} at {}", fam.first_trade.label().to_lowercase(), st.town_name[fam.seat]),
        choice: Choice::Apprentice { group: fam.first_trade, town: fam.seat },
    }];
    for g in Group::ALL.into_iter().filter(|&g| g != fam.first_trade) {
        let town = best_town(st, values, g, &taken).unwrap_or(fam.seat);
        out.push(DecisionOption {
            label: format!("Apprentice in {} at {}", g.label().to_lowercase(), st.town_name[town]),
            choice: Choice::Apprentice { group: g, town },
        });
    }
    out.push(DecisionOption { label: "University".to_string(), choice: Choice::University });
    out
}

/// The player's default training: an apprenticeship in the family's first trade
/// at the seat while nobody living is trained or training in it (or waiting to
/// be); after that, what an AI family would choose (`new_trade`).
fn training_default(st: &State, values: &[Vec<f64>], k: usize, options: &[DecisionOption]) -> usize {
    let fam = &st.families[k];
    if !in_hand(fam, false)[fam.first_trade.index()] {
        return 0;
    }
    let choice = new_trade(st, values, fam);
    options.iter().position(|o| o.choice == choice).unwrap_or(0)
}

/// The trades a family's living members are trained or training in, with those
/// that waiting decisions default to; with `known`, the family's known trades too.
fn in_hand(fam: &Family, known: bool) -> [bool; N_GROUPS] {
    let mut out = if known { fam.known } else { [false; N_GROUPS] };
    for m in fam.members.iter().filter(|m| m.alive) {
        if let Some(g) = m.trade {
            out[g.index()] = true;
        }
        if let Post::Learning { path: Path::Apprentice { group, .. }, .. } = m.post {
            out[group.index()] = true;
        }
    }
    for d in fam.decisions.iter().filter(|d| d.member.is_some()) {
        if let Some(DecisionOption { choice: Choice::Apprentice { group, .. }, .. }) = d.options.get(d.default) {
            out[group.index()] = true;
        }
    }
    out
}

/// Where a family has or will have someone in a trade: its branches, its
/// apprentices' trades and towns, and those of the trainings that waiting
/// decisions default to.
fn places_in_hand(fam: &Family) -> BTreeSet<(Group, usize)> {
    let mut out: BTreeSet<(Group, usize)> = branches(fam).into_iter().collect();
    for m in fam.members.iter().filter(|m| m.alive) {
        if let Post::Learning { path: Path::Apprentice { group, town }, .. } = m.post {
            out.insert((group, town));
        }
    }
    for d in fam.decisions.iter().filter(|d| d.member.is_some()) {
        if let Some(DecisionOption { choice: Choice::Apprentice { group, town }, .. }) = d.options.get(d.default) {
            out.insert((*group, *town));
        }
    }
    out
}

/// An apprenticeship in the most valuable trade (estates aside) the family neither
/// knows nor is learning, where that trade is strongest. Once it has them all:
/// an apprenticeship in a trade (estates and the first trade aside) in the town
/// where it is worth most and the family has nobody in it yet; failing that, its
/// first trade at the seat.
fn new_trade(st: &State, values: &[Vec<f64>], fam: &Family) -> Choice {
    let trades = in_hand(fam, true);
    let taken = places_in_hand(fam);
    let total = |g: Group| -> f64 { values[g.index()].iter().sum() };
    if let Some(g) = Group::ALL.into_iter().filter(|&g| !trades[g.index()] && g != Group::Estates).max_by(|a, b| total(*a).total_cmp(&total(*b))) {
        return Choice::Apprentice { group: g, town: best_town(st, values, g, &taken).unwrap_or(fam.seat) };
    }
    let best = Group::ALL
        .into_iter()
        .filter(|&g| g != Group::Estates && g != fam.first_trade)
        .filter_map(|g| best_town(st, values, g, &taken).map(|t| (values[g.index()][t], g, t)))
        .filter(|&(v, g, t)| v > 0.0 && !taken.contains(&(g, t)))
        .max_by(|a, b| a.0.total_cmp(&b.0));
    match best {
        Some((_, g, t)) => Choice::Apprentice { group: g, town: t },
        None => Choice::Apprentice { group: fam.first_trade, town: fam.seat },
    }
}

/// The active town where a trade's works are worth most (for estates: the most
/// rent), leaving out the places in `taken`; any town if every one is taken.
fn best_town(st: &State, values: &[Vec<f64>], g: Group, taken: &BTreeSet<(Group, usize)>) -> Option<usize> {
    let score = |t: usize| -> f64 {
        if g == Group::Estates {
            (0..st.N).filter(|&c| st.catch[c] == t as i64).map(|c| st.cell_rent_year[c]).sum()
        } else {
            values[g.index()][t]
        }
    };
    let best = |free: bool| (0..st.T_max).filter(|&t| st.town_active[t] && (!free || !taken.contains(&(g, t)))).max_by(|&a, &b| score(a).total_cmp(&score(b)).then(b.cmp(&a)));
    best(true).or_else(|| best(false))
}

// ---------------------------------------------------------------- AI families

/// An AI family's training choice: a gifted child to university if nobody in the
/// family is educated; otherwise the most valuable trade it neither knows nor is
/// learning, where that trade is strongest; otherwise its first trade at the seat.
fn raises_ai(st: &mut State, C: &Content, values: &[Vec<f64>], k: usize, j: usize, year: i64) {
    let fam = &st.families[k];
    let choice = if fam.members[j].skill >= 0.75 && !fam.members.iter().any(|m| m.alive && m.educated) {
        Choice::University
    } else {
        new_trade(st, values, fam)
    };
    apply_choice(st, C, k, choice, Some(j), year, &mut Vec::new());
}

/// AI families post every unposted adult as manager of a known trade in the town
/// where that trade is worth most and the family has no branch of it yet.
fn ai_choose(st: &mut State, _C: &Content, k: usize, _year: i64, values: &[Vec<f64>], _events: &mut Vec<Event>) {
    let idle: Vec<usize> = {
        let fam = &st.families[k];
        (0..fam.members.len()).filter(|&j| adult(&fam.members[j]) && fam.members[j].post == Post::Unposted && fam.members[j].next.is_none()).collect()
    };
    for j in idle {
        let fam = &st.families[k];
        let mut best: Option<(f64, Group, usize)> = None;
        for g in Group::ALL.into_iter().filter(|g| fam.known[g.index()] && *g != Group::Estates) {
            for t in (0..st.T_max).filter(|&t| st.town_active[t] && !has_branch(fam, g, t)) {
                let v = values[g.index()][t];
                if v > 0.0 && best.is_none_or(|b| v > b.0) {
                    best = Some((v, g, t));
                }
            }
        }
        if let Some((_, g, t)) = best {
            let _ = post_member(st, k, j, Posting::Manager { group: g, town: t });
        }
    }
}

// ---------------------------------------------------------------- branches at work

/// Cash a family keeps back: `family_reserve_months` of its spending.
fn reserve(fam: &Family, P: &Params) -> f64 {
    P.family_reserve_months * monthly_spending(fam, P)
}

fn run_branches(st: &mut State, C: &Content, P: &Params, rng: &mut Rng, k: usize, year: i64, events: &mut Vec<Event>) {
    let fam = &st.families[k];
    if fam.decisions.iter().any(|d| d.options.iter().any(|o| matches!(o.choice, Choice::Trade { .. }))) {
        return; // the first trade is not chosen yet: every branch holds
    }
    let mut spare = fam.cash - reserve(fam, P);
    for (g, t) in branches(fam) {
        let fam = &st.families[k];
        let skill = runner(fam, g, t).map_or(0.0, |j| skill_in(&fam.members[j], g));
        match stance(fam, g, t) {
            Stance::Hold => {}
            Stance::Grow if spare > 0.0 => {
                // a skilled runner does twice as much
                let n = 1 + (skill > 0.6) as usize;
                if g == Group::Estates {
                    buy_farms(st, C, P, k, t, &mut spare, n);
                } else {
                    grow(st, C, P, rng, k, g, t, &mut spare, n, year, events);
                }
            }
            Stance::Grow => {}
            Stance::WindDown => wind_down(st, C, P, k, g, t, year, events),
        }
    }
}

/// A growing branch founds up to n works of its trade in its town's reach, the
/// best expected returns above the hurdle rate first, only where the market is
/// deep (`family_max_share`, `family_max_price_ratio`) and cash allows.
#[allow(clippy::too_many_arguments)]
fn grow(st: &mut State, C: &Content, P: &Params, rng: &mut Rng, k: usize, g: Group, t: usize, spare: &mut f64, n: usize, year: i64, events: &mut Vec<Event>) {
    let b = branch_bonus(&st.families[k], P, g, t);
    let mut options: Vec<(f64, usize, usize, f64)> = Vec::new();
    for m in (0..C.M).filter(|&m| Group::of_method(C, m) == Some(g) && st.know[(t, m)] > 0.02) {
        let K = P.entry_K_frac * C.m_K0[m];
        let (_, _, _, share, ratio) = market_depth(st, C, t, m, K);
        if share > P.family_max_share || ratio > P.family_max_price_ratio {
            continue;
        }
        let cells = firms::candidate_cells(st, C, P, t, m, rng);
        if cells.is_empty() {
            continue;
        }
        let ev = firms::evaluate_sites(st, C, P, t, m, &cells, K, prices(st), wages(st));
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
    options.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut built = 0;
    let mut used = BTreeSet::new();
    for (roi, m, c, cost) in options {
        if built >= n || roi <= P.hurdle {
            break;
        }
        if cost > *spare || !used.insert(c) {
            continue;
        }
        if found_at(st, C, P, k, m, c, year, events).is_ok() {
            *spare -= cost;
            built += 1;
        }
    }
}

/// A growing estates branch buys up to n cells of farmland in its town's reach,
/// next to the family's land where it can, the highest rents first.
fn buy_farms(st: &mut State, C: &Content, P: &Params, k: usize, t: usize, spare: &mut f64, n: usize) {
    let (H, W) = (st.world.H, st.world.W);
    let next_to_own = |st: &State, c: usize| -> bool {
        let (y, x) = ((c / W) as isize, (c % W) as isize);
        (-1..=1).any(|dy| {
            (-1..=1).any(|dx| {
                let (ny, nx) = (y + dy, x + dx);
                (dy != 0 || dx != 0) && ny >= 0 && nx >= 0 && (ny as usize) < H && (nx as usize) < W && st.cell_owner[ny as usize * W + nx as usize] == k as i64
            })
        })
    };
    let mut cells: Vec<(bool, f64, usize)> = (0..st.N)
        .filter(|&c| st.catch[c] == t as i64 && st.cell_owner[c] < 0 && is_farmland(st, C, c) && st.cell_rent_year[c] > 0.0)
        .map(|c| (next_to_own(st, c), st.cell_rent_year[c], c))
        .collect();
    cells.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.total_cmp(&a.1)).then(a.2.cmp(&b.2)));
    let mut bought = 0;
    for (_, _, c) in cells {
        if bought >= n {
            break;
        }
        let price = land_price(st, P, c);
        if price > *spare {
            continue;
        }
        if buy_at(st, P, k, c).is_ok() {
            *spare -= price;
            bought += 1;
        }
    }
}

/// A branch winding down sells one works a year, the smallest first, to the
/// town's savers at book value (estates: one cell, the poorest, at its price).
/// If the savers cannot pay the whole price, nothing is sold that year.
#[allow(clippy::too_many_arguments)]
fn wind_down(st: &mut State, C: &Content, P: &Params, k: usize, g: Group, t: usize, year: i64, events: &mut Vec<Event>) {
    let surname = st.families[k].surname.clone();
    if g == Group::Estates {
        let cell = (0..st.N)
            .filter(|&c| st.cell_owner[c] == k as i64 && st.catch[c] == t as i64)
            .min_by(|&a, &b| st.cell_rent_year[a].total_cmp(&st.cell_rent_year[b]).then(a.cmp(&b)));
        let Some(c) = cell else { return };
        let price = land_price(st, P, c);
        if firms::town_pool(st, P, t) < price {
            return;
        }
        let got = firms::draw_pool(st, P, t, price);
        st.families[k].cash += got;
        st.families[k].income.sales += got;
        st.cell_owner[c] = -1;
        events.extend(news_at(k, None, year, format!("The {surname} family sells a farm near {}", st.town_name[t]), format!("Sold for {got:.0}."), t));
        return;
    }
    let works = (0..st.F_count)
        .filter(|&f| st.f_alive[f] && st.f_owner[f] == k as i64 && st.f_town[f] == t && Group::of_method(C, st.f_method[f]) == Some(g))
        .min_by(|&a, &b| works_value(st, C, a).total_cmp(&works_value(st, C, b)).then(a.cmp(&b)));
    let Some(f) = works else { return };
    let price = works_value(st, C, f);
    if firms::town_pool(st, P, t) < price {
        return;
    }
    let got = firms::draw_pool(st, P, t, price);
    st.families[k].cash += got;
    st.families[k].income.sales += got;
    st.f_owner[f] = -1;
    events.extend(news_at(k, None, year, format!("The {surname} family sells {}", st.f_name[f]), format!("Sold to investors at {} for {got:.0}.", st.town_name[t]), t));
}
