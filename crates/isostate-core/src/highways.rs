//! The state's ways: the treasury buys road-making, trunk roads between the
//! large towns, and a bypass where the through traffic pays for one. Not in the
//! Python (`docs/rules/highways.md`).
//!
//! After 1900 the builder of main roads is the state (docs/designs/ways.md, part
//! 4), and the state counts everybody's time. Two rules, each behind a switch
//! of its own and each acting only after 1900:
//!
//! - **Trunk roads** (`highways.trunk`, from `TRUNK_FROM`). Each year the state
//!   looks at the cheapest way over the ground between every two large towns
//!   (`LARGE` of all townsfolk or more, the `LARGE_MOST` largest), and prices
//!   bettering the part of it not yet a trunk road: `WORK_YEARS` men's years
//!   at the town's wage a square of lowland (hills and mountains their carting
//!   cost's multiple), a third of it stone and two thirds wages, and a bridge
//!   on every link of it that fords or ferries. Through a third town's ground
//!   the old road is that town's high street and is left as it is. What it
//!   saves a year is the year's weight by road over each square bettered times
//!   what a trunk road takes off the square's cost, and the fords' charge on
//!   the links it bridges. A trunk road's square costs a turnpike's, less a
//!   third of it on the share of the hauling the lorries do (`trunk_mult`): a
//!   horse goes no faster on a surfaced road, a motor does. It builds the ways
//!   that save `HURDLE` a year on their cost, best first, as far as the year's
//!   budget for works goes. A trunk road is bettered in place, on the old
//!   road's squares, and no trunk road fords a river.
//! - **Bypasses** (`highways.bypass`, from `BYPASS_FROM`). For each town a
//!   finished trunk road passes through, the state prices two cures for the
//!   town's stretch of it, the high street the trunk road was not bettered
//!   through: a way round over the cheapest squares outside every town's
//!   ground, at its work and the land it takes (`LAND_TAKEN` of each square's
//!   value), which saves the through traffic (the trade between two other
//!   towns, by road) what crossing the town costs over going round on a trunk
//!   road; or widening the high street into a trunk road, at `FRONTAGE` of the
//!   value of the houses along it and half the work, which saves every load
//!   over it what a trunk road takes off a square. It builds the better cure
//!   where it saves `HURDLE` a year on its cost. So a bypass lies on the side
//!   the traffic passes, as one arc, and no town gets one without the traffic.
//!
//! **The treasury buys.** A state's works is a road works (`ways::ROAD`, as a
//! parish's bridge is): its stone is bought at the market of the town whose
//! district holds the squares, and its wages are paid to that town's men, who
//! are hands taken from its farms and works (`roads::take_menders`). So
//! road-making is work in the towns. The treasury pays each works its month's
//! bill out of what it holds after the month's teachers and pensions, keeping
//! a month of their spending in hand, and never borrows. The year's budget for
//! works is `WORKS_SHARE` of last year's pay; at each year's end the purse
//! reckons next year's bill for the works in hand with its programmes, so the
//! rate on pay rises to pay for them (`purse::close_year`).
//!
//! Watched whatever the switches, while the traffic record is kept: the
//! through traffic of each square and each town (what passes a town's ground
//! by road on its way between two other towns), and the treasury's year.
//!
//! Off, nothing here acts: the cost graph, the roads and the purse are as they
//! were.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::facts::Kind;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::{Event, Project, State};
use crate::transport as tr;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HighwayParams {
    /// From `TRUNK_FROM` the treasury betters the ways between the large towns
    /// into trunk roads, where the traffic pays for them.
    pub trunk: bool,
    /// From `BYPASS_FROM` the treasury bypasses a town a trunk road passes
    /// through, or widens its high street, where what it saves pays.
    pub bypass: bool,
}

impl Default for HighwayParams {
    /// Off: what the port runs with.
    fn default() -> HighwayParams {
        HighwayParams { trunk: false, bypass: false }
    }
}

impl HighwayParams {
    /// The game's: both on.
    pub fn game() -> HighwayParams {
        HighwayParams { trunk: true, bypass: true }
    }
}

/// The first year at whose end the state puts a trunk road in hand: the year
/// of Britain's Road Board.
pub const TRUNK_FROM: i64 = 1909;
/// The first year at whose end a bypass may be put in hand.
pub const BYPASS_FROM: i64 = 1920;
/// What a square of trunk road costs to pass, against 0.3 for a turnpike: a
/// wider, graded, surfaced road with its fords bridged.
pub const TRUNK_MULT: f64 = 0.2;
/// Men's years of work, at the town's wage, to better a square of lowland
/// into a trunk road, its stone and its wages together (hills and mountains
/// their carting cost's multiple): about thirty a kilometre, a fifth of what
/// an arterial road of the 1920s cost on new ground, since the trunk road is
/// the old road bettered where it runs.
pub const WORK_YEARS: f64 = 60.0;
/// Wages paid on every shilling of stone: road-making is mostly labour.
pub const WAGES: f64 = 2.0;
/// What a way must save a year over what it costs.
pub const HURDLE: f64 = 0.05;
/// The most of last year's pay the treasury puts into works in hand.
pub const WORKS_SHARE: f64 = 0.01;
/// A large town holds this share of all townsfolk or more.
pub const LARGE: f64 = 0.04;
/// And no more than this many towns are large.
pub const LARGE_MOST: usize = 10;
/// The share of the houses' value along a high street that widening it pulls down.
pub const FRONTAGE: f64 = 0.15;
/// The share of a square's land a road takes.
pub const LAND_TAKEN: f64 = 0.02;
/// `Project::serves` of the state's works (`roads.rs` uses 2, 3 and 9, `crossings.rs` 4).
pub const TRUNK: u8 = 5;
pub const BYPASS: u8 = 6;
pub const WIDEN: u8 = 7;
/// A square that is not (yet) a trunk road or a bypass.
pub const NOT: i32 = i32::MAX;

/// A trunk road the state put in hand: the way between two large towns.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Route {
    pub a: usize,
    pub b: usize,
    /// The year it was put in hand.
    pub year: i64,
    /// Its squares from `a` to `b`, the old road's.
    pub cells: Vec<u32>,
    /// The squares bettered by this route (the rest were trunk road already).
    pub new: u32,
    /// What it was reckoned to save a year and to cost, money.
    pub gain: f64,
    pub cost: f64,
}

/// A town's bypass, or its high street widened.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Bypass {
    pub town: usize,
    /// The trunk road it serves (its place in `HighwayState::routes`).
    pub route: usize,
    /// The year it was put in hand, and the year it opened (`NOT` until then).
    pub year: i64,
    pub opened: i64,
    /// Widened, not bypassed.
    pub widen: bool,
    /// The squares of the way round, or of the high street widened.
    pub cells: Vec<u32>,
    /// The through traffic over the town's squares of the trunk road that
    /// year, weight-squares, and the town's place by through traffic (1 the most).
    pub through: f64,
    pub rank: usize,
    pub gain: f64,
    pub cost: f64,
}

/// A works of the state's in hand, matched to its project by (serves, town, target).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Works {
    pub serves: u8,
    pub town: usize,
    pub target: usize,
    pub cells: Vec<usize>,
    /// Links over water it bridges (the crossings' table).
    pub links: Vec<u32>,
    /// The bypass it builds (its place in `HighwayState::bypasses`), if it is one.
    pub bypass: Option<usize>,
}

/// One year of the state's ways.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct HwYear {
    pub year: i64,
    /// Squares of trunk road open, of them of bypasses, and bypasses and high streets widened open.
    pub trunk_squares: usize,
    pub bypass_squares: usize,
    pub bypasses: usize,
    pub widened: usize,
    /// What the treasury paid into its works, and what they spent on stone and wages.
    pub paid: f64,
    pub spent: f64,
    /// Hands at the state's road-making, mean of the months.
    pub menders: f64,
    /// Through traffic by road, weight-squares: all, and the three towns with most, (town, weight-squares).
    pub through: f64,
    pub through_top: Vec<(usize, f64)>,
}

/// `State::hw`: empty while the rule neither acts nor watches.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HighwayState {
    /// By square: the year it became a trunk road, and a bypass, or `NOT`.
    pub trunk: Vec<i32>,
    pub bypass: Vec<i32>,
    /// By square: its high street was widened into a trunk road.
    pub widened: Vec<bool>,
    /// By square: last year's weight by road (`sim::update_roads`'s wear).
    pub wear: Vec<f64>,
    /// By square: last year's through traffic by road; and by town, weight-squares.
    pub thru: Vec<f64>,
    pub town_thru: Vec<f64>,
    pub routes: Vec<Route>,
    pub bypasses: Vec<Bypass>,
    pub works: Vec<Works>,
    /// The year running, and every year closed.
    pub now: HwYear,
    pub years: Vec<HwYear>,
}

/// Whether either rule acts.
pub fn on(P: &Params) -> bool {
    P.highways.trunk || P.highways.bypass
}

/// Whether the through traffic and the treasury's year are watched.
fn watching(P: &Params) -> bool {
    on(P) || P.traffic.enabled
}

fn ensure(st: &mut State) {
    let N = st.N;
    let h = &mut st.hw;
    if h.trunk.len() != N {
        h.trunk = vec![NOT; N];
        h.bypass = vec![NOT; N];
        h.widened = vec![false; N];
    }
}

fn is_state(pr: &Project) -> bool {
    pr.way == crate::ways::ROAD && matches!(pr.serves, TRUNK | BYPASS | WIDEN)
}

fn terrain(st: &State, c: usize) -> f64 {
    tr::TERRAIN_COST[st.world.terrain[c] as usize].max(1.0)
}

/// What a square of trunk road costs to pass this year: a turnpike's
/// (`lorry::road_mult`), less what the trunk road's surface saves a motor
/// over a turnpike (`TRUNK_MULT` against 0.3) on the share `motor` of the
/// hauling the lorries do. A cart's horse goes no faster on it.
pub fn trunk_mult(mult: &[f64; 4], motor: f64) -> f64 {
    mult[3] * (1.0 - (1.0 - TRUNK_MULT / tr::ROAD_MULT[3]) * motor.clamp(0.0, 1.0))
}

/// The share of the island's hauling the lorries did in the last month (as
/// `lorry::record_month` reckons it; nothing before the lorry).
pub fn motor_share(st: &State) -> f64 {
    let tk = &st.tasks;
    let (mut a, mut b) = (0.0, 0.0);
    for i in 0..tk.cart_work.len().min(tk.cart_held.len()).min(st.town_active.len()) {
        if st.town_active[i] && tk.cart_work[i] > 0.0 {
            a += tk.cart_work[i] * tk.cart_held[i][crate::tasks::HAUL];
            b += tk.cart_work[i];
        }
    }
    if b > 0.0 { a / b } else { 0.0 }
}

/// This year's trunk road's cost a square.
fn trunk_now(st: &State, mult: &[f64; 4]) -> f64 {
    trunk_mult(mult, motor_share(st))
}

fn is_trunk(st: &State, c: usize) -> bool {
    st.hw.trunk.get(c).is_some_and(|&y| y != NOT)
}

/// The cell's part of an edge's cost, as `transport::build_graph_with` reckons it.
fn cell_cost(st: &State, mult: &[f64; 4], c: usize) -> f64 {
    let woods = if st.cover[c] > 0.6 && st.road[c] == 0 { 0.25 } else { 0.0 };
    tr::TERRAIN_COST[st.world.terrain[c] as usize] * mult[st.road[c] as usize] * (1.0 + woods)
}

fn cell_cost_now(st: &State, mult: &[f64; 4], tm: f64, c: usize) -> f64 {
    if is_trunk(st, c) { tr::TERRAIN_COST[st.world.terrain[c] as usize] * tm } else { cell_cost(st, mult, c) }
}

/// Whether square `c` is a town's ground, which a trunk road is not bettered
/// through: the old road's line there is a high street, with houses along it.
fn in_town(st: &State, c: usize) -> bool {
    st.urban[c] && st.catch[c] >= 0
}

/// What bettering square `c` into a trunk road costs, money, at town `t`'s wage.
fn work_money(st: &State, c: usize, t: usize) -> f64 {
    WORK_YEARS * 12.0 * st.wbar.get(t).copied().unwrap_or(0.0).max(0.0) * terrain(st, c)
}

/// The year's cost graph (`haulage::build_graph`, before the canals, the tolls
/// and the fords are laid on it): a trunk road's squares cost `trunk_mult`, on
/// the ground's edges (not by train or boat).
pub fn graph(st: &State, P: &Params, g: &mut tr::Graph, mult: &[f64; 4]) {
    if !on(P) || st.hw.trunk.len() != st.N || !st.hw.trunk.iter().any(|&y| y != NOT) {
        return;
    }
    let (N, H, W) = (st.N, st.world.H, st.world.W);
    let tm = trunk_now(st, mult);
    let land = &st.world.land;
    let nav = &st.world.navigable;
    let mut changed = false;
    for a in 0..N {
        if !land[a] {
            continue;
        }
        for k in 0..8 {
            let Some(b) = tr::neighbour(H, W, a, k) else { continue };
            let e = a * 8 + k;
            if !land[b] || g.railed[e] || !g.w[e].is_finite() || (nav[a] && nav[b]) {
                continue;
            }
            if !(is_trunk(st, a) || is_trunk(st, b)) {
                continue;
            }
            let old = cell_cost(st, mult, a) + cell_cost(st, mult, b);
            let new = cell_cost_now(st, mult, tm, a) + cell_cost_now(st, mult, tm, b);
            let add = tr::step_len(k) * 0.5 * (new - old);
            if add != 0.0 {
                g.w[e] = (g.w[e] + add).max(1e-3);
                changed = true;
            }
        }
    }
    if changed {
        let finite = g.w.iter().copied().filter(|x| x.is_finite());
        g.min_w = finite.clone().fold(f64::INFINITY, f64::min);
        g.max_w = finite.fold(0.0, f64::max);
    }
}

/// The year's weight by road over each square, as `sim::update_roads` wore the roads with it.
pub fn note_wear(st: &mut State, P: &Params, wear: &[f64]) {
    if on(P) {
        st.hw.wear.clear();
        st.hw.wear.extend_from_slice(wear);
    }
}

/// The ground alone of this year's cost graph: no trains, no boats; an edge of
/// the port's railway (both ends with track) costs what the ground costs.
fn ground(st: &State, P: &Params) -> tr::Graph {
    let mut g = st.graph.clone();
    g.train = Vec::new();
    g.handling = 0.0;
    g.boat = Vec::new();
    g.wharf = 0.0;
    let mult = crate::lorry::road_mult(st, P);
    let tm = trunk_now(st, &mult);
    let (N, H, W) = (st.N, st.world.H, st.world.W);
    for a in 0..N {
        for k in 0..8 {
            let e = a * 8 + k;
            if !g.railed.get(e).copied().unwrap_or(false) {
                continue;
            }
            let Some(b) = tr::neighbour(H, W, a, k) else { continue };
            g.w[e] = (tr::step_len(k) * 0.5 * (cell_cost_now(st, &mult, tm, a) + cell_cost_now(st, &mult, tm, b))).max(1e-3);
            g.railed[e] = false;
        }
    }
    let finite = g.w.iter().copied().filter(|x| x.is_finite());
    g.min_w = finite.clone().fold(f64::INFINITY, f64::min);
    g.max_w = finite.fold(0.0, f64::max);
    g
}

/// The year's through traffic by road (watched): for each trade between two
/// towns, the squares of a third town's ground its load passes by road.
fn through(st: &State) -> (Vec<f64>, Vec<f64>) {
    let (N, T) = (st.N, st.T_max);
    let mut thru = vec![0.0; N];
    let mut town = vec![0.0; T];
    if st.trade_F_year.rows != T || st.town_cost.rows != T || st.catch.len() != N {
        return (thru, town);
    }
    let mut water = crate::canals::Water::new(st);
    let mut rails = crate::haulage::Rails::new(st);
    for i in 0..T {
        if st.pred[i].len() != N {
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
                    let (p, nn) = (p as usize, n as usize);
                    let s = crate::haulage::step(st, &mut water, &mut rails, i, p, nn);
                    if s.mode == crate::traffic::CART && !s.jump && st.urban[nn] {
                        let k = st.catch[nn];
                        if k >= 0 && k as usize != i && k as usize != j {
                            thru[nn] += f;
                            town[k as usize] += f * s.squares;
                        }
                    }
                }
                n = p;
                steps += 1;
            }
        }
    }
    (thru, town)
}

/// What the state's works in hand still have to buy, at this month's prices, stone and wages.
fn in_hand(st: &State) -> f64 {
    st.projects
        .iter()
        .filter(|pr| is_state(pr))
        .map(|pr| (1.0 + pr.wages) * pr.order.iter().enumerate().map(|(g, q)| q * st.price[(pr.town, g)]).sum::<f64>())
        .sum()
}

/// The town that buys for square `c`: the one whose district holds it, else `or`.
fn buyer(st: &State, c: usize, or: usize) -> usize {
    match st.catch.get(c) {
        Some(&t) if t >= 0 && st.town_active[t as usize] => t as usize,
        _ => or,
    }
}

/// The links over water between consecutive squares of `cells` with no
/// bridge and none in hand: (link, its town, its stone, its charge).
fn fords(st: &State, cells: &[usize], or: usize, held: &[bool]) -> Vec<(u32, usize)> {
    let tab = &st.xs.table;
    if tab.links.is_empty() {
        return Vec::new();
    }
    let W = st.world.W;
    let mut out = Vec::new();
    for p in cells.windows(2) {
        if let Some(li) = tab.between(W, p[0], p[1]) {
            if tab.links[li].n > 0 && st.xs.built[li] == crate::crossings::NOT_BUILT && !held[li] {
                out.push((li as u32, buyer(st, p[0], or)));
            }
        }
    }
    out
}

/// Links in hand already, the parishes' and the state's.
fn links_held(st: &State) -> Vec<bool> {
    let mut held = vec![false; st.xs.table.links.len()];
    for (_, ls) in &st.xs.works {
        for &l in ls {
            held[l as usize] = true;
        }
    }
    for w in &st.hw.works {
        for &l in &w.links {
            held[l as usize] = true;
        }
    }
    held
}

/// Stone bought at town `t` for squares `cells` bettered into a trunk road and
/// `links` bridged: the squares' `work_money` less its wages, at the stone's
/// price there, and the parishes' stone a metre of bridge.
fn stone_of(st: &State, P: &Params, gs: usize, t: usize, cells: &[usize], links: &[u32]) -> f64 {
    let p = st.pbar[(t, gs)];
    let s: f64 = if p > 0.0 { cells.iter().map(|&c| work_money(st, c, t) / ((1.0 + WAGES) * p)).sum() } else { 0.0 };
    s + links.iter().map(|&l| P.crossings.bridge_stone * st.xs.table.links[l as usize].span as f64).sum::<f64>()
}

/// Put works in hand: one project a town, for its squares and links.
#[allow(clippy::too_many_arguments)]
fn put_in_hand(st: &mut State, C: &Content, P: &Params, serves: u8, cells: &[usize], links: &[(u32, usize)], or: usize, months: f64, year: i64, bypass: Option<usize>) {
    let gs = C.gi("stone");
    let mut towns: Vec<usize> = cells.iter().map(|&c| buyer(st, c, or)).chain(links.iter().map(|l| l.1)).collect();
    towns.sort();
    towns.dedup();
    for t in towns {
        let cs: Vec<usize> = cells.iter().copied().filter(|&c| buyer(st, c, or) == t).collect();
        let ls: Vec<u32> = links.iter().filter(|l| l.1 == t).map(|l| l.0).collect();
        let mut stone = stone_of(st, P, gs, t, &cs, &ls);
        if serves == WIDEN {
            // a street is widened on the line it runs: half the stone of a new way
            stone *= 0.5;
        }
        if !(stone > 0.0) {
            continue;
        }
        let target = cs.first().copied().unwrap_or_else(|| st.xs.table.links[ls[0] as usize].a as usize);
        let mut need = vec![0.0; C.G];
        need[gs] = stone;
        st.projects.push(Project {
            a: t,
            b: t,
            target,
            town: t,
            length_km: 2.0 * cs.len() as f64,
            cells: cs.clone(),
            new: if serves == WIDEN { Vec::new() } else { cs.clone() },
            order: need.clone(),
            need,
            cash: 0.0,
            start: year,
            cost: stone * st.pbar[(t, gs)] * (1.0 + WAGES),
            way: crate::ways::ROAD,
            months,
            serves,
            wages: WAGES,
            ..Project::default()
        });
        st.hw.works.push(Works { serves, town: t, target, cells: cs, links: ls, bypass });
    }
}

/// The year's state ways (`sim::yearly`, after the crossings, on the year's
/// traffic and before the paths are made again): the through traffic
/// watched, the year put away, and the trunk roads and bypasses that pay put
/// in hand.
pub fn yearly(st: &mut State, C: &Content, P: &Params, year: i64, warm: bool) {
    if !watching(P) || warm {
        return;
    }
    ensure(st);
    let (thru, town) = through(st);
    let mut now = std::mem::take(&mut st.hw.now);
    now.year = year;
    now.menders /= 12.0;
    now.trunk_squares = st.hw.trunk.iter().filter(|&&y| y != NOT).count();
    now.bypass_squares = st.hw.bypass.iter().filter(|&&y| y != NOT).count();
    now.bypasses = st.hw.bypasses.iter().filter(|b| !b.widen && b.opened != NOT as i64).count();
    now.widened = st.hw.bypasses.iter().filter(|b| b.widen && b.opened != NOT as i64).count();
    now.through = town.iter().sum();
    let mut top: Vec<(usize, f64)> = town.iter().copied().enumerate().filter(|x| x.1 > 0.0).collect();
    top.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    top.truncate(3);
    now.through_top = top;
    st.hw.years.push(now);
    st.hw.now = HwYear { year: year + 1, ..HwYear::default() };
    st.hw.thru = thru;
    st.hw.town_thru = town;
    if !on(P) || st.hw.wear.len() != st.N || !P.state.enabled {
        return;
    }
    let budget = WORKS_SHARE * st.purse.base_last;
    if !(budget > 0.0) {
        return;
    }
    let mut room = budget - in_hand(st);
    let mut started = false;
    if P.highways.trunk && year >= TRUNK_FROM {
        plan_trunk(st, C, P, year, budget, &mut room, &mut started);
    }
    if P.highways.bypass && year >= BYPASS_FROM {
        plan_bypass(st, C, P, year, budget, &mut room, &mut started);
    }
    fund(st, C);
}

/// The large towns: the `LARGE_MOST` with most townsfolk of those holding `LARGE` of all.
fn large_towns(st: &State) -> Vec<usize> {
    let all: f64 = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| st.urban_pop[t]).sum();
    let mut l: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t] && st.urban_pop[t] >= LARGE * all && all > 0.0).collect();
    l.sort_by(|&a, &b| st.urban_pop[b].total_cmp(&st.urban_pop[a]).then(a.cmp(&b)));
    l.truncate(LARGE_MOST);
    l.sort();
    l
}

/// The months a works of `cost` takes, at the year's budget.
fn months_for(cost: f64, budget: f64) -> f64 {
    12.0 * (cost / budget).max(1.0)
}

fn plan_trunk(st: &mut State, C: &Content, P: &Params, year: i64, budget: f64, room: &mut f64, started: &mut bool) {
    let large = large_towns(st);
    if large.len() < 2 {
        return;
    }
    let gs = C.gi("stone");
    let freight = crate::longrun::freight(st, P);
    let mult = crate::lorry::road_mult(st, P);
    let tm = trunk_now(st, &mult);
    let g = ground(st, P);
    let N = st.N;
    let mut busy = vec![false; N];
    for w in &st.hw.works {
        for &c in &w.cells {
            busy[c] = true;
        }
    }
    let held = links_held(st);
    // (ratio, a, b, its squares, those to better, its fords, gain, cost)
    let mut cand: Vec<(f64, usize, usize, Vec<usize>, Vec<usize>, Vec<(u32, usize)>, f64, f64)> = Vec::new();
    for (x, &i) in large.iter().enumerate() {
        let src = st.town_node[i];
        let paths = tr::dijkstra(&g, &[(src, 0.0)], None);
        for &j in &large[x + 1..] {
            let cells = tr::path_cells(&paths.pred, src, st.town_node[j]);
            if cells.len() < 2 || cells.iter().any(|&c| busy[c] || !st.world.land[c]) {
                continue;
            }
            // (into the two towns' own middles, as an arterial road; but not through
            // a third town's ground, where the old road is its high street)
            let third = |c: usize| in_town(st, c) && st.catch[c] as usize != i && st.catch[c] as usize != j;
            let new: Vec<usize> = cells.iter().copied().filter(|&c| !is_trunk(st, c) && !third(c)).collect();
            let links = fords(st, &cells, i, &held);
            if new.is_empty() && links.is_empty() {
                continue;
            }
            let mut gain: f64 = new.iter().map(|&c| freight * st.hw.wear[c] * tr::TERRAIN_COST[st.world.terrain[c] as usize] * (mult[st.road[c] as usize] - tm).max(0.0)).sum();
            gain += links.iter().map(|l| freight * st.xs.traffic.get(l.0 as usize).copied().unwrap_or(0.0) * st.xs.table.links[l.0 as usize].charge as f64).sum::<f64>();
            let mut cost = 0.0;
            for &c in &new {
                cost += work_money(st, c, buyer(st, c, i));
            }
            for l in &links {
                cost += P.crossings.bridge_stone * st.xs.table.links[l.0 as usize].span as f64 * st.pbar[(l.1, gs)] * (1.0 + WAGES);
            }
            if cost > 0.0 && gain >= HURDLE * cost {
                cand.push((gain / cost, i, j, cells, new, links, gain, cost));
            }
        }
    }
    cand.sort_by(|a, b| b.0.total_cmp(&a.0).then((a.1, a.2).cmp(&(b.1, b.2))));
    for (_, i, j, cells, new, links, gain, cost) in cand {
        if new.iter().any(|&c| busy[c]) {
            continue;
        }
        let first = !*started && in_hand(st) <= 0.0;
        if cost > *room && !first {
            continue;
        }
        for &c in &new {
            busy[c] = true;
        }
        *room -= cost;
        *started = true;
        st.hw.routes.push(Route { a: i, b: j, year, cells: cells.iter().map(|&c| c as u32).collect(), new: new.len() as u32, gain, cost });
        put_in_hand(st, C, P, TRUNK, &new, &links, i, months_for(cost, budget), year, None);
    }
}

fn plan_bypass(st: &mut State, C: &Content, P: &Params, year: i64, budget: f64, room: &mut f64, started: &mut bool) {
    let N = st.N;
    let gs = C.gi("stone");
    let freight = crate::longrun::freight(st, P);
    let mult = crate::lorry::road_mult(st, P);
    let tm = trunk_now(st, &mult);
    // the towns by their through traffic, for the record
    let mut ranked: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    ranked.sort_by(|&a, &b| st.hw.town_thru[b].total_cmp(&st.hw.town_thru[a]).then(a.cmp(&b)));
    let rank = |t: usize| ranked.iter().position(|&x| x == t).map_or(usize::MAX, |p| p + 1);
    let mut busy = vec![false; N];
    for w in &st.hw.works {
        for &c in &w.cells {
            busy[c] = true;
        }
    }
    let held = links_held(st);
    let mut round: Option<tr::Graph> = None;
    // (ratio, town, route, widen, squares, fords, through, gain, cost)
    let mut cand: Vec<(f64, usize, usize, bool, Vec<usize>, Vec<(u32, usize)>, f64, f64, f64)> = Vec::new();
    for (r, route) in st.hw.routes.iter().enumerate() {
        let cells: Vec<usize> = route.cells.iter().map(|&c| c as usize).collect();
        // a trunk road is bypassed once it is open all its length outside the towns
        if !cells.iter().all(|&c| is_trunk(st, c) || in_town(st, c)) {
            continue;
        }
        let mut towns: Vec<usize> = cells.iter().filter(|&&c| in_town(st, c)).map(|&c| st.catch[c] as usize).collect();
        towns.sort();
        towns.dedup();
        for k in towns {
            if st.hw.bypasses.iter().any(|b| b.town == k && b.route == r) {
                continue;
            }
            let ours = |c: usize| st.urban[c] && st.catch[c] == k as i64;
            let (Some(f), Some(l)) = (cells.iter().position(|&c| ours(c)), cells.iter().rposition(|&c| ours(c))) else { continue };
            if f == 0 || l + 1 >= cells.len() {
                continue;
            }
            let S = &cells[f..=l];
            if S.iter().any(|&c| busy[c] || is_trunk(st, c)) || !S.iter().any(|&c| st.hw.thru[c] > 0.0) {
                continue;
            }
            let thru: f64 = S.iter().map(|&c| st.hw.thru[c]).sum::<f64>() / S.len() as f64;
            let (entry, exit) = (cells[f - 1], cells[l + 1]);
            // the way round, outside every town's ground
            let g = round.get_or_insert_with(|| {
                let mut g = ground(st, P);
                for a in 0..N {
                    for kk in 0..8 {
                        if let Some(b) = tr::neighbour(st.world.H, st.world.W, a, kk) {
                            if st.urban[b] || busy[b] {
                                g.w[a * 8 + kk] = f64::INFINITY;
                            }
                        }
                    }
                }
                g
            });
            // (the ends lie on the trunk road outside the town, and may be another
            // town's ground: the way round leaves the one and ends beside the other)
            let paths = tr::dijkstra(g, &[(entry, 0.0)], None);
            let mut best: Option<(f64, usize)> = None;
            for kk in 0..8 {
                if let Some(b) = tr::neighbour(st.world.H, st.world.W, exit, kk) {
                    if paths.dist[b].is_finite() && (best.is_none() || paths.dist[b] < best.unwrap().0) {
                        best = Some((paths.dist[b], b));
                    }
                }
            }
            let Some((_, last)) = best else { continue };
            let mut way = tr::path_cells(&paths.pred, entry, last);
            if way.len() < 2 {
                continue;
            }
            way.push(exit);
            let inner: Vec<usize> = way[1..way.len() - 1].iter().copied().filter(|&c| !is_trunk(st, c)).collect();
            if inner.is_empty() {
                continue;
            }
            let len_cost = |ws: &[usize], cost: &dyn Fn(usize) -> f64| -> f64 {
                ws.windows(2).map(|p| tr::step_len(crate::traffic::direction(st.world.W, p[0], p[1]).unwrap_or(0)) * 0.5 * (cost(p[0]) + cost(p[1]))).sum()
            };
            let mut town_way = vec![entry];
            town_way.extend_from_slice(S);
            town_way.push(exit);
            let c_town = len_cost(&town_way, &|c| cell_cost_now(st, &mult, tm, c));
            let c_round = len_cost(&way, &|c| tr::TERRAIN_COST[st.world.terrain[c] as usize] * tm);
            // the way round saves the through traffic what crossing the town's
            // squares costs over going round on a trunk road; widening the high
            // street saves every load over it what a trunk road takes off a square
            let save_by = freight * thru * (c_town - c_round);
            let save_wi: f64 = S.iter().map(|&c| freight * st.hw.wear[c] * (cell_cost(st, &mult, c) - tr::TERRAIN_COST[st.world.terrain[c] as usize] * tm).max(0.0)).sum();
            let links = fords(st, &way, k, &held);
            // (the rents are a month's)
            let mut cost_by: f64 = inner.iter().map(|&c| work_money(st, c, buyer(st, c, k)) + P.years_purchase * 12.0 * st.cell_rent[c].max(0.0) * LAND_TAKEN).sum();
            cost_by += links.iter().map(|l| P.crossings.bridge_stone * st.xs.table.links[l.0 as usize].span as f64 * st.pbar[(l.1, gs)] * (1.0 + WAGES)).sum::<f64>();
            let cost_wi: f64 = S.iter().map(|&c| P.years_purchase * 12.0 * st.cell_urban_rent[c].max(0.0) * FRONTAGE + 0.5 * work_money(st, c, buyer(st, c, k))).sum();
            let r_by = if cost_by > 0.0 { save_by / cost_by } else { 0.0 };
            let r_wi = if cost_wi > 0.0 { save_wi / cost_wi } else { 0.0 };
            if r_by >= r_wi && r_by >= HURDLE {
                cand.push((r_by, k, r, false, inner, links, thru, save_by, cost_by));
            } else if r_wi > r_by && r_wi >= HURDLE {
                cand.push((r_wi, k, r, true, S.to_vec(), Vec::new(), thru, save_wi, cost_wi));
            }
        }
    }
    cand.sort_by(|a, b| b.0.total_cmp(&a.0).then((a.1, a.2).cmp(&(b.1, b.2))));
    for (_, k, r, widen, cells, links, thru, gain, cost) in cand {
        if cells.iter().any(|&c| busy[c]) || st.hw.bypasses.iter().any(|b| b.town == k && b.route == r) {
            continue;
        }
        let first = !*started && in_hand(st) <= 0.0;
        if cost > *room && !first {
            continue;
        }
        for &c in &cells {
            busy[c] = true;
        }
        *room -= cost;
        *started = true;
        let b = st.hw.bypasses.len();
        st.hw.bypasses.push(Bypass { town: k, route: r, year, opened: NOT as i64, widen, cells: cells.iter().map(|&c| c as u32).collect(), through: thru, rank: rank(k), gain, cost });
        put_in_hand(st, C, P, if widen { WIDEN } else { BYPASS }, &cells, &links, k, months_for(cost, budget), year, Some(b));
    }
}

/// The treasury pays each works in hand its coming month's bill, out of what
/// it holds above a month of its programmes' spending.
fn fund(st: &mut State, C: &Content) {
    let _ = C;
    let p = &st.purse;
    let floor = p.month.schools + p.month.pensions + p.month.university + p.uni.month_rooms;
    let mut free = (st.purse.treasury - floor).max(0.0);
    let mut paid = 0.0;
    for pr in st.projects.iter_mut().filter(|pr| is_state(pr)) {
        if free <= 0.0 {
            break;
        }
        let months = pr.months.max(1.0);
        let bill: f64 = (1.0 + pr.wages) * (0..pr.order.len()).map(|g| pr.order[g].min(pr.need[g] / months) * st.price[(pr.town, g)]).sum::<f64>();
        let put = (bill - pr.cash).max(0.0).min(free);
        if put > 0.0 {
            pr.cash += put;
            pr.paid_in += put;
            free -= put;
            paid += put;
        }
    }
    if paid > 0.0 {
        st.purse.treasury -= paid;
        if st.purse.treasury < 0.0 && st.purse.treasury > -1e-9 * paid {
            st.purse.treasury = 0.0;
        }
        st.hw.now.paid += paid;
    }
}

/// The month's end for the state's ways (after `econ::step_month`): what the
/// works spent, the treasury's next payment, and finished works open.
pub fn month(st: &mut State, C: &Content, P: &Params, year_float: f64, events: &mut Vec<Event>) {
    if !on(P) || st.hw.works.is_empty() {
        return;
    }
    for pr in st.projects.iter().filter(|pr| is_state(pr)) {
        st.hw.now.spent += pr.navvy_pay * (1.0 + 1.0 / pr.wages.max(1e-9));
        if st.wage[pr.town] > 0.0 {
            st.hw.now.menders += pr.navvy_pay / st.wage[pr.town];
        }
    }
    let year = year_float.floor() as i64;
    let projects = std::mem::take(&mut st.projects);
    let mut keep = Vec::with_capacity(projects.len());
    let mut back = 0.0;
    for mut pr in projects {
        if !is_state(&pr) || pr.order.iter().sum::<f64>() > 0.01 * pr.need.iter().sum::<f64>() {
            keep.push(pr);
            continue;
        }
        let Some(w) = st.hw.works.iter().position(|w| w.serves == pr.serves && w.town == pr.town && w.target == pr.target) else {
            keep.push(pr);
            continue;
        };
        let w = st.hw.works.remove(w);
        back += pr.cash;
        pr.cash = 0.0;
        // (a high street widened is a trunk road through the town from now on)
        for &c in &w.cells {
            st.road[c] = st.road[c].max(3);
            st.road_quiet[c] = 0;
            if st.hw.trunk[c] == NOT {
                st.hw.trunk[c] = year as i32;
            }
            match w.serves {
                WIDEN => st.hw.widened[c] = true,
                BYPASS if st.hw.bypass[c] == NOT => st.hw.bypass[c] = year as i32,
                _ => {}
            }
        }
        let span: f64 = w.links.iter().map(|&l| st.xs.table.links[l as usize].span as f64).sum();
        let spent = pr.paid_in;
        for &l in &w.links {
            if st.xs.built[l as usize] == crate::crossings::NOT_BUILT {
                st.xs.built[l as usize] = year as i32;
                let s = st.xs.table.links[l as usize].span as f64;
                st.xs.bridges.push(crate::crossings::Built { link: l, year, town: w.town, lords: false, cost: if span > 0.0 { spent * s / span } else { 0.0 }, span: s, inherited: false });
            }
        }
        // a bypass or a widened street is news when the last of its works opens
        if let Some(b) = w.bypass {
            if !st.hw.works.iter().any(|x| x.bypass == Some(b)) {
                st.hw.bypasses[b].opened = year;
                let by = &st.hw.bypasses[b];
                let t = by.town;
                let (title, text) = if by.widen {
                    (format!("{}'s high street is widened", st.town_name[t]), format!("The traffic between other towns along the trunk road through {} was held up in its streets; the treasury has widened the high street, pulling down the fronts of its houses, where a way round would have cost more.", st.town_name[t]))
                } else {
                    (format!("{} is bypassed", st.town_name[t]), format!("The treasury has built {:.0} km of new road round {}, over the fields at its edge, so that the traffic between other towns along the trunk road no longer goes through its streets.", 2.0 * by.cells.len() as f64, st.town_name[t]))
                };
                let mut ev = Event::new(year_float, "highway", 0.35, title, text);
                ev.town = Some(t);
                ev.cell = by.cells.first().map(|&c| c as usize);
                events.push(ev);
            }
        } else if w.serves == TRUNK && st.hw.trunk.iter().filter(|&&y| y != NOT).count() == w.cells.len() && !w.cells.is_empty() {
            let t = w.town;
            let mut ev = Event::new(
                year_float,
                "highway",
                0.4,
                "The first trunk road".to_string(),
                format!("The treasury has bettered {:.0} km of the old road near {} into a trunk road, wider, graded and surfaced, with its fords bridged, the first of the ways between the island's large towns.", 2.0 * w.cells.len() as f64, st.town_name[t]),
            );
            ev.town = Some(t);
            ev.cell = w.cells.first().copied();
            events.push(ev);
        }
    }
    st.projects = keep;
    if back != 0.0 {
        st.purse.treasury += back;
    }
    let _ = P;
    fund(st, C);
}

/// What the works in hand will want of the treasury next year, at December's
/// prices: the rest of each one's bill, no more than twelve months of it
/// (`purse::close_year` adds it to next year's spending).
pub fn spend_next(st: &State, P: &Params) -> f64 {
    if !on(P) {
        return 0.0;
    }
    st.projects
        .iter()
        .filter(|pr| is_state(pr))
        .map(|pr| {
            let months = pr.months.max(1.0);
            (1.0 + pr.wages) * (0..pr.order.len()).map(|g| pr.order[g].min(12.0 * pr.need[g] / months) * st.price[(pr.town, g)]).sum::<f64>()
        })
        .sum()
}

/// What the treasury paid its works in `year` (nothing while the rules are off).
pub fn paid_in(st: &State, year: i64) -> f64 {
    st.hw.years.iter().rev().find(|y| y.year == year).map_or(0.0, |y| y.paid)
}

/// The record's road byte: 64 marks a trunk road's square, 128 a bypass's
/// (`record::layers`; nothing while the rules are off).
pub fn mark(st: &State, trans: &mut [u8]) {
    if st.hw.trunk.len() != trans.len() {
        return;
    }
    for (c, t) in trans.iter_mut().enumerate() {
        if st.hw.trunk[c] != NOT {
            *t |= 64;
        }
        if st.hw.bypass[c] != NOT {
            *t |= 128;
        }
    }
}

/// The trunk roads and bypasses as they stand, for the run's header.
pub fn header(st: &State, P: &Params) -> Option<serde_json::Value> {
    if !on(P) || st.hw.routes.is_empty() && st.hw.bypasses.is_empty() {
        return None;
    }
    Some(serde_json::json!({ "routes": st.hw.routes, "bypasses": st.hw.bypasses }))
}

/// The rule's facts, every tenth year from 1900 (watched whatever the
/// switches, while the traffic record is kept): km of trunk road and of
/// bypass open, bypasses and high streets widened; the through traffic by
/// road (millions of weight-km), the share of it through the three towns with
/// most; the treasury's tax and programmes over the year's income, its months
/// held, and over the ten years to the mark its works over income and over its
/// spending, and the hands at its works over all hands. Over the run: the trunk road's links
/// that ford or ferry (none by construction), the year of the first trunk
/// road, and the mean place by through traffic of the towns bypassed.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C, P) = (&r.state, &r.content, &r.params);
    let mut out = Vec::new();
    if !watching(P) || st.hw.years.is_empty() {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    let at = |y: i64| st.hw.years.iter().find(|v| v.year == y);
    let pu = |y: i64| st.purse.years.iter().find(|v| v.year == y);
    let share = |a: f64, b: f64| if b > 0.0 { a / b } else { f64::NAN };
    for y in (1900..=y1).step_by(10).filter(|&y| y >= y0) {
        let v = at(y);
        let f = |g: &dyn Fn(&HwYear) -> f64| v.map_or(f64::NAN, g);
        out.push((format!("trunk_km_{y}"), Kind::Real, f(&|v| 2.0 * v.trunk_squares as f64)));
        out.push((format!("bypass_km_{y}"), Kind::Real, f(&|v| 2.0 * v.bypass_squares as f64)));
        out.push((format!("bypasses_{y}"), Kind::Count, f(&|v| v.bypasses as f64)));
        out.push((format!("streets_widened_{y}"), Kind::Count, f(&|v| v.widened as f64)));
        out.push((format!("through_traffic_{y}"), Kind::Real, f(&|v| v.through * 2.0 / 1e6)));
        out.push((format!("through_top3_share_{y}"), Kind::Share, f(&|v| share(v.through_top.iter().map(|x| x.1).sum(), v.through))));
        let p = pu(y);
        let g = |h: &dyn Fn(&crate::purse::PurseYear) -> f64| p.map_or(f64::NAN, h);
        out.push((format!("state_tax_over_income_{y}"), Kind::Share, g(&|p| share(p.tax, p.income))));
        out.push((format!("state_programmes_over_income_{y}"), Kind::Share, g(&|p| share(p.schools + p.pensions + p.university, p.income))));
        // the works over the ten years to y, since a works is built in two or three years of ten
        let (mut works, mut menders, mut income, mut programmes, mut hands) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for z in y - 9..=y {
            if let (Some(v), Some(p)) = (at(z), pu(z)) {
                works += v.spent;
                menders += v.menders;
                income += p.income;
                programmes += p.schools + p.pensions + p.university;
                hands += p.hands;
            }
        }
        out.push((format!("state_works_over_income_{y}"), Kind::Share, share(works, income)));
        out.push((format!("state_works_over_spending_{y}"), Kind::Share, share(works, programmes + works)));
        out.push((format!("state_works_hands_{y}"), Kind::Share, share(menders, hands)));
        out.push((format!("state_months_held_dec_{y}"), Kind::Real, g(&|p| p.months_held)));
    }
    if on(P) {
        let mut fording = 0usize;
        let W = st.world.W;
        for route in &st.hw.routes {
            for p in route.cells.windows(2) {
                let (a, b) = (p[0] as usize, p[1] as usize);
                if !(is_trunk(st, a) && is_trunk(st, b)) {
                    continue;
                }
                if let Some(li) = st.xs.table.between(W, a, b) {
                    if st.xs.table.links[li].n > 0 && st.xs.built[li] == crate::crossings::NOT_BUILT {
                        fording += 1;
                    }
                }
            }
        }
        out.push(("trunk_links_fording".into(), Kind::Count, fording as f64));
        let first = st.hw.trunk.iter().copied().filter(|&y| y != NOT).min();
        out.push(("trunk_first_year".into(), Kind::Year, first.map_or(f64::NAN, |y| y as f64)));
        let ranks: Vec<f64> = st.hw.bypasses.iter().filter(|b| !b.widen && b.opened != NOT as i64).map(|b| b.rank as f64).collect();
        out.push(("bypassed_rank_mean".into(), Kind::Real, if ranks.is_empty() { f64::NAN } else { ranks.iter().sum::<f64>() / ranks.len() as f64 }));
        let mut towns: Vec<usize> = st.hw.bypasses.iter().filter(|b| !b.widen && b.opened != NOT as i64).map(|b| b.town).collect();
        towns.sort();
        towns.dedup();
        out.push(("bypassed_towns".into(), Kind::Count, towns.len() as f64));
        // and of every cure opened, a bypass or a high street widened
        let cured: Vec<f64> = st.hw.bypasses.iter().filter(|b| b.opened != NOT as i64).map(|b| b.rank as f64).collect();
        out.push(("cured_rank_mean".into(), Kind::Real, if cured.is_empty() { f64::NAN } else { cured.iter().sum::<f64>() / cured.len() as f64 }));
        out.push(("cured_rank_most".into(), Kind::Real, cured.iter().copied().fold(f64::NAN, f64::max)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_trunk_road_saves_a_motor_what_it_saves_and_a_cart_nothing() {
        // a cart's horse goes no faster on a trunk road than on a turnpike
        assert!((trunk_mult(&tr::ROAD_MULT, 0.0) - tr::ROAD_MULT[3]).abs() < 1e-12);
        // every load by lorry: the trunk road's whole cost
        assert!((trunk_mult(&tr::ROAD_MULT, 1.0) - TRUNK_MULT).abs() < 1e-12);
        // cheapened by the lorry as a turnpike is
        let half = [1.0, 0.7, 0.225, 0.15];
        assert!((trunk_mult(&half, 1.0) - 0.5 * TRUNK_MULT).abs() < 1e-12);
        assert!(months_for(5.0, 10.0) == 12.0 && months_for(30.0, 10.0) == 36.0);
    }
}
