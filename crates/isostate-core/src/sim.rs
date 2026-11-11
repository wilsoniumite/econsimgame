//! Set up a world and run it. Python: `sim.py`.
//!
//! `run(params)` is the whole history in one call. The same loop is also split
//! into `start` (world, warm-up and the 1750 baselines), `run_month`, `run_year`
//! and `finish`, so a game loop or a Python binding can step through it.

use crate::content::{Content, default_content, tech_index};
use crate::econ;
use crate::families;
use crate::firms;
use crate::harvest;
use crate::land;
use crate::mat::Mat;
use crate::names::town_name;
use crate::ndimage::{self, Mode};
use crate::observer;
use crate::params::Params;
use crate::people;
use crate::rail;
use crate::record::{self, Record};
use crate::rng::{Rngs, make_rngs};
use crate::state::{Event, State};
use crate::tech;
use crate::transport as tr;
use crate::util::{argsort, bincount, pairwise_sum};
use crate::works;
use crate::world::{World, generate_world, site_context};

/// The initial state: world, towns, the first land allocation and the first firms.
pub fn init_state(P: &Params, C: &Content) -> (State, Rngs) {
    init_state_with(P, C, make_rngs(P.seed))
}

/// `init_state` with the random streams given (the parity tests replay numpy's draws).
pub fn init_state_with(P: &Params, C: &Content, mut rng: Rngs) -> (State, Rngs) {
    let world = generate_world(&mut rng.world, P.W, P.H, P.n_towns, 0.58);
    let (mut st, pbar, wbar) = init_before_firms(P, C, world, &mut rng);
    seed_firms(&mut st, C, P, &mut rng, &pbar, &wbar);
    (st, rng)
}

/// Everything in `init_state` between generating the world and `seed_firms`:
/// towns, the first roads and the first land allocation. Returns the state and
/// the prices and wages the first firms are planned with.
pub fn init_before_firms(P: &Params, C: &Content, world: World, rng: &mut Rngs) -> (State, Mat, Vec<f64>) {
    let (W, H) = (world.W, world.H);
    let (N, T, G, Fm) = (W * H, P.T_max, C.G, P.F_max);
    let mut st = State { N, T_max: T, ..State::default() };

    // towns
    st.town_active = vec![false; T];
    st.town_node = vec![0; T];
    st.town_founded = vec![-1; T];
    st.town_new = vec![false; T];
    for (k, &c) in world.town_sites.iter().enumerate() {
        st.town_active[k] = true;
        st.town_node[k] = c;
        let name = town_name(&mut rng.names, site_context(&world, c), &mut st.used_names);
        st.town_name.push(name);
        st.town_founded[k] = C.start_year - 1;
    }
    let wts: Vec<f64> = world
        .town_sites
        .iter()
        .map(|&c| {
            let (ty, tx) = ((c / W) as f64, (c % W) as f64);
            let masked: Vec<f64> = (0..N)
                .map(|i| {
                    let near = ((i / W) as f64 - ty).hypot((i % W) as f64 - tx) < 12.0;
                    world.fertility[i] * if near { 1.0 } else { 0.0 }
                })
                .collect();
            pairwise_sum(&masked)
        })
        .collect();
    let wsum = pairwise_sum(&wts);
    st.pop = vec![0.0; T];
    for (k, &w) in wts.iter().enumerate() {
        st.pop[k] = P.pop0 as f64 * w / wsum;
    }

    // cells
    st.land_use = world.land.iter().map(|&l| if l { land::USE_WILD } else { land::USE_SEA }).collect();
    st.cover = world.forest0.clone();
    st.road = vec![0; N];
    st.rail = vec![0; N];
    st.road_quiet = vec![0; N];
    st.firm_at = vec![-1; N];
    st.cell_K_share = vec![0.0; N];
    st.land_rent_only = vec![0.0; N];
    st.derelict_timer = vec![0; N];
    st.cell_rent = vec![0.0; N];
    st.cell_flow = vec![0.0; N];
    st.traffic = vec![0.0; N];
    st.urban = vec![false; N];
    st.urban_res = vec![0.0; N];
    st.water = (0..N).map(|c| world.coast[c] || world.river[c]).collect();
    st.world = world;

    // firms
    st.F_count = 0;
    st.f_alive = vec![false; Fm];
    st.f_method = vec![0; Fm];
    st.f_cell = vec![0; Fm];
    st.f_town = vec![0; Fm];
    st.f_born = vec![0; Fm];
    st.f_died = vec![0; Fm];
    st.f_loss_years = vec![0; Fm];
    st.f_parent = vec![0; Fm];
    for v in [
        &mut st.f_K,
        &mut st.f_order_K,
        &mut st.f_cash,
        &mut st.f_rent,
        &mut st.f_u,
        &mut st.f_fill,
        &mut st.f_prod,
        &mut st.f_profit_acc,
        &mut st.f_out_acc,
        &mut st.f_cost_m,
        &mut st.f_labor,
    ] {
        *v = vec![0.0; Fm];
    }
    st.f_order = Mat::zeros(Fm, G);
    works::init(&mut st, C, P); // works: its two tables (empty while it is off)

    // money and markets
    let pop = st.pop.clone();
    st.hh_cash = pop.iter().map(|p| 6.0 * p).collect();
    st.lord_cash = pop.iter().map(|p| 6.0 * p).collect();
    st.merch_cash = pop.iter().map(|p| 2.0 * p).collect();
    st.hh_income = vec![0.0; T];
    st.lord_income = vec![0.0; T];
    st.hh_income_prev = pop.iter().map(|p| 1.8 * p).collect();
    st.lord_income_prev = pop.iter().map(|p| 0.4 * p).collect();
    st.fund_cash = 0.0;
    st.issued = 0.0;
    st.price = Mat::zeros(T, G);
    for t in 0..T {
        st.price.row_mut(t).copy_from_slice(&C.base_price);
    }
    st.pexp = st.price.clone();
    st.lam = Mat::zeros(C.NS + 1, N);
    let guess = [1.3, 0.06, 0.45, 0.06, 0.03, 0.02, 0.03, 0.04, 0.01, 0.0, 0.006, 0.02, 0.02, 0.0, 0.0, 0.05];
    st.inv = Mat::zeros(T, G);
    st.outflow = Mat::zeros(T, G);
    for t in 0..T {
        for g in 0..G {
            st.inv[(t, g)] = 2.0 * pop[t] * guess[g];
            st.outflow[(t, g)] = pop[t] * guess[g];
        }
    }
    st.fill = Mat::filled(T, G, 1.0);
    st.wage = (0..T).map(|t| if st.town_active[t] { 2.5 } else { 0.0 }).collect();
    st.tools_mult = vec![1.2; T];
    st.imp_pc = vec![0.006; T];
    st.urban_rent = vec![0.0; T];
    st.urban_pop = pop.iter().map(|p| 0.1 * p).collect();
    st.ag_labor = pop.iter().map(|p| 0.7 * P.participation * p).collect();
    st.ind_labor = pop.iter().map(|p| 0.08 * P.participation * p).collect();
    st.y_ratio = vec![1.0; T];
    st.subcost = vec![0.0; T];
    st.food_ratio = vec![0.0; T];

    // knowledge
    st.know = Mat::zeros(T, C.NT);
    for t in 0..T {
        for j in 0..C.NT {
            st.know[(t, j)] = if C.tech_known[j] { 1.0 } else { 0.0 };
        }
    }
    st.tech_year = C.tech_known.iter().map(|&k| if k { C.start_year - 100 } else { -1 }).collect();
    st.tech_town = vec![-1; C.NT];
    st.t_pump = tech_index(C, "steam_pumping");
    st.t_wagon = tech_index(C, "wagonway");
    st.t_rail = tech_index(C, "railway");
    st.t_husb = tech_index(C, "improved_husbandry");
    st.t_fmach = tech_index(C, "farm_machinery");
    st.forestry_sector = C.sector_name.iter().position(|n| n == "forestry").expect("a forestry sector");
    st.method_Q = vec![0.0; C.M];
    st.method_A = vec![1.0; C.M];

    // accumulators
    st.y_acc = vec![0.0; T];
    st.price_acc = Mat::zeros(T, G);
    st.wage_acc = vec![0.0; T];
    st.town_prod = Mat::zeros(T, G);
    st.town_prod_year = Mat::zeros(T, G);
    st.cumprod = vec![0.0; G];
    st.trade_F = Mat::zeros(T, T);
    st.trade_F_year = Mat::zeros(T, T);
    st.food_min_acc = vec![9.0; T];
    st.cell_flow_year = vec![0.0; N];
    st.m_rent = 0.0;
    st.m_invest = 0.0;

    // first roads: tracks between each town and its two nearest neighbours
    land::update_transport(&mut st, C, P);
    let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t]).collect();
    for &i in &act {
        let costs: Vec<f64> = act.iter().map(|&j| st.town_cost[(i, j)]).collect();
        for &jj in argsort(&costs).iter().skip(1).take(2) {
            let j = act[jj];
            let path = tr::path_cells(&st.pred[i], st.town_node[i], st.town_node[j]);
            for c in path {
                if st.world.land[c] {
                    st.road[c] = st.road[c].max(1);
                }
            }
        }
    }
    land::update_transport(&mut st, C, P);
    after_transport(&mut st, C, P);

    let pbar = st.price.clone();
    let wbar = st.wage.clone();
    firms::update_productivity(&mut st, C, P);
    land::update_land_use(&mut st, C, P, C.start_year, &pbar, &wbar, true);
    land::delta_fields(&mut st, C, P);
    (st, pbar, wbar)
}

/// Bookkeeping that depends on catchments.
pub fn after_transport(st: &mut State, C: &Content, P: &Params) {
    let (N, T) = (st.N, st.T_max);
    let alive: Vec<usize> = (0..st.f_alive.len()).filter(|&f| st.f_alive[f]).collect();
    for &f in &alive {
        let ct = st.catch[st.f_cell[f]];
        if ct >= 0 {
            st.f_town[f] = ct as usize;
        }
    }
    let mut town_cells: Vec<Vec<usize>> = vec![Vec::new(); T];
    for c in 0..N {
        if st.catch[c] >= 0 {
            town_cells[st.catch[c] as usize].push(c);
        }
    }
    st.town_cells = town_cells;
    let (H, W) = (st.world.H, st.world.W);
    let mut cnt = vec![0.0; N];
    for &f in &alive {
        cnt[st.f_cell[f]] += 1.0;
    }
    st.any_counts = ndimage::uniform_filter(&cnt, H, W, 5, Mode::Constant).iter().map(|v| v * 25.0).collect();
    let foot: Vec<f64> = alive.iter().map(|&f| works::land_held(st, C, P, f)).collect(); // works: the one sum of a works' land
    let mut ks = vec![0.0; N];
    for (k, &f) in alive.iter().enumerate() {
        ks[st.f_cell[f]] += foot[k];
    }
    st.cell_K_share = ks.iter().map(|&v| v.min(1.0)).collect();
    // the firm with the largest footprint on each cell
    let mut main = vec![-1i64; N];
    for &k in &argsort(&foot) {
        main[st.f_cell[alive[k]]] = alive[k] as i64;
    }
    st.firm_at = main;
    st.cell_firms = cnt;
}

/// The artisans of 1750: enough smiths, weavers, colliers and furnaces for local demand.
pub fn seed_firms(st: &mut State, C: &Content, P: &Params, rng: &mut Rngs, pbar: &Mat, wbar: &[f64]) {
    let T = st.T_max;
    let (share, unit) = econ::service_mix(pbar, &C.props, P.logit_theta);
    let (q, _) = econ::les(&vec![2.1; T], &unit, &C.subsistence, &C.shares);
    let d = econ::goods_from_services(&q, &share, &C.props);
    let mut need = Mat::zeros(T, C.G);
    for t in 0..T {
        for g in 0..C.G {
            need[(t, g)] = d[(t, g)] * st.pop[t];
        }
    }
    let (iron, tools, charcoal, bricks) = (C.gi("iron"), C.gi("tools"), C.gi("charcoal"), C.gi("bricks"));
    for t in 0..T {
        need[(t, iron)] += 0.55 * need[(t, tools)];
        need[(t, charcoal)] += 2.3 * need[(t, iron)];
        need[(t, bricks)] += 0.03 * st.pop[t];
    }
    let plan = ["tools", "cloth", "iron", "charcoal", "bricks", "services"];
    for i in 0..T {
        if !st.town_active[i] {
            continue;
        }
        for gname in plan {
            let g = C.gi(gname);
            let target = need[(i, g)] * 1.25;
            let meths: Vec<usize> = (0..C.M).filter(|&m| C.method_out[m] == g && C.tech_known[m]).collect();
            let mut best: Option<(f64, usize)> = None;
            for &m in &meths {
                let K = target.min(2.0 * C.m_K0[m]);
                let cells = firms::candidate_cells(st, C, P, i, m, &mut rng.firms);
                if cells.is_empty() {
                    continue;
                }
                let ev = firms::evaluate_sites(st, C, P, i, m, &cells, K, pbar, wbar);
                let j = crate::util::argmax(&ev.roi);
                if best.is_none() || ev.roi[j] > best.unwrap().0 {
                    best = Some((ev.roi[j], m));
                }
            }
            let Some((_, m)) = best else { continue };
            let n_firms = ((target / (2.0 * C.m_K0[m])).ceil().clamp(1.0, 6.0)) as usize;
            let K_each = target / n_firms as f64;
            for _ in 0..n_firms {
                let cells = firms::candidate_cells(st, C, P, i, m, &mut rng.firms);
                if cells.is_empty() {
                    break;
                }
                let ev = firms::evaluate_sites(st, C, P, i, m, &cells, K_each, pbar, wbar);
                let j = crate::util::argmax(&ev.roi);
                let rent_bid_one = ev.rent_bid[j];
                let reason = firms::reason_record(st, i, m, cells[j], &ev, j, K_each, C.start_year - 1, "old");
                let cost_m = K_each * pbar[(i, C.method_out[m])];
                let Some(fid) = firms::create_firm(
                    st,
                    C,
                    P,
                    &mut rng.firms,
                    i,
                    m,
                    cells[j],
                    K_each,
                    rent_bid_one,
                    reason,
                    C.start_year - 1,
                    6.0 * cost_m,
                    None,
                    None,
                ) else {
                    break;
                };
                st.f_K[fid] = K_each;
                st.f_order.row_mut(fid).iter_mut().for_each(|v| *v = 0.0);
                st.f_order_K[fid] = 0.0;
                st.f_u[fid] = 0.8;
            }
            st.same_counts = firms::same_method_counts(st, C);
        }
    }
    after_transport(st, C, P);
    land::delta_fields(st, C, P);
}

/// Traffic wears tracks; busy tracks become roads, roads become turnpikes.
pub fn update_roads(st: &mut State, _C: &Content, P: &Params) {
    let N = st.N;
    let mut traffic = vec![0.0; N];
    for i in 0..st.T_max {
        if !st.town_active[i] {
            continue;
        }
        let fl: Vec<f64> = (0..N).map(|c| if st.catch[c] == i as i64 { st.cell_flow[c] } else { 0.0 }).collect();
        if pairwise_sum(&fl) <= 0.0 {
            continue;
        }
        // Python walks the reachable cells by falling distance; the search's settle
        // order backwards is also children before parents (sums agree to rounding)
        let order: Vec<usize> = st.settle[i].iter().rev().map(|&c| c as usize).collect();
        tr::tree_traffic(&st.pred[i], &order, &fl, &mut traffic);
    }
    let mut F = st.trade_F_year.clone();
    for (f, c) in F.data.iter_mut().zip(&st.town_cost.data) {
        if !c.is_finite() {
            *f = 0.0;
        }
    }
    tr::pair_traffic(&st.pred, &st.town_node, &F, &mut traffic);
    for c in 0..N {
        if !st.world.land[c] {
            traffic[c] = 0.0;
        }
    }
    let [t1, t2, t3] = P.road_thresholds;
    let thr = [0.0, t1, t2, t3];
    for c in 0..N {
        let lvl = (traffic[c] >= t1) as u8 + (traffic[c] >= t2) as u8 + (traffic[c] >= t3) as u8;
        if lvl > st.road[c] {
            st.road[c] = lvl;
        }
        let quiet = st.road[c] > 0 && traffic[c] < 0.2 * thr[st.road[c] as usize];
        st.road_quiet[c] = if quiet { st.road_quiet[c] + 1 } else { 0 };
        if st.road_quiet[c] >= 6 {
            st.road[c] -= 1;
            st.road_quiet[c] = 0;
        }
    }
    st.traffic = traffic;
    st.cell_flow.iter_mut().for_each(|v| *v = 0.0);
}

/// The yearly step: people, knowledge, roads, transport, land, firms and railways.
/// With `warm`, the 1740s warm-up: no growth, discoveries, new towns or railways.
#[allow(clippy::too_many_arguments)]
pub fn yearly(st: &mut State, C: &Content, P: &Params, rng: &mut Rngs, year: i64, events: &mut Vec<Event>, warm: bool) {
    let (N, T) = (st.N, st.T_max);
    let mut pbar = st.price_acc.clone();
    pbar.data.iter_mut().for_each(|v| *v /= 12.0);
    let wbar: Vec<f64> = st.wage_acc.iter().map(|v| v / 12.0).collect();
    st.pbar = pbar.clone();
    st.wbar = wbar.clone();
    st.price_acc.fill(0.0);
    st.wage_acc.iter_mut().for_each(|v| *v = 0.0);
    st.trade_F_year = st.trade_F.clone();
    st.trade_F.fill(0.0);
    st.town_prod_year = st.town_prod.clone();
    st.town_prod.fill(0.0);
    let ag_w: Vec<f64> = (0..st.ex_cell.len()).map(|u| if st.ex_sector[u] <= 2 { st.ex_labor[u] } else { 0.0 }).collect();
    st.ag_labor = bincount(&st.ex_town, &ag_w, T);
    let ind_w: Vec<f64> = (0..st.f_alive.len()).map(|f| if st.f_alive[f] { st.f_labor[f] } else { 0.0 }).collect();
    st.ind_labor = bincount(&st.f_town, &ind_w, T);
    st.cell_labor = bincount(&st.ex_cell, &st.ex_labor, N);

    let mut founded = Vec::new();
    if !warm {
        people::growth(st, C, P, year, events);
        people::migration(st, C, P);
        tech::discovery(st, C, P, &mut rng.tech, year, events);
        // people (docs/rules/people.md): its own founding rule runs below, once the year's roads are known
        if !P.people.enabled {
            founded = people::found_towns(st, C, P, &mut rng.people, year, events);
        }
    } else {
        st.y_ratio = st.y_acc.iter().map(|v| v / 12.0).collect();
        st.y_acc.iter_mut().for_each(|v| *v = 0.0);
        st.food_min_acc.iter_mut().for_each(|v| *v = 9.0);
    }
    tech::diffusion(st, C, P);
    st.cell_flow_year = st.cell_flow.clone();
    update_roads(st, C, P);
    // canals (docs/rules/canals.md): what the year's goods put on the water, on the
    // paths they took (before the paths are worked out again)
    crate::canals::year_traffic(st, C, P, year, warm);
    if !warm && P.people.enabled {
        founded = crate::peopling::found_towns(st, C, P, year, events);
        // people: what each cell paid to reach its market, before the paths are worked out again
        st.pp_dcatch_was = st.dcatch.clone();
    }
    st.prev_catch = st.catch.clone();
    land::update_transport(st, C, P);
    if P.people.enabled {
        // people: households go with their cells every year, not only at a founding
        if !warm {
            crate::peopling::move_with_cells(st, C, P);
        }
    } else if !founded.is_empty() {
        people::resettle_new_towns(st, C, P);
    }
    after_transport(st, C, P);
    firms::update_productivity(st, C, P);
    land::update_land_use(st, C, P, year, &pbar, &wbar, false);
    land::delta_fields(st, C, P);
    st.new_firms_year.clear();
    st.closed_year.clear();
    let closed = firms::review(st, C, P, &mut rng.firms, year, &pbar, &wbar);
    firms::entry(st, C, P, &mut rng.firms, year, &pbar, &wbar, None);
    after_transport(st, C, P);
    // harvest: the books close on the year's crop and the coming year's weather is
    // drawn, on the land as it has just been shared out and before the families
    // act (they price land on the year's rents)
    harvest::yearly(st, C, P, &mut rng.harvest, year, events, warm);
    if !warm {
        rail::plan(st, C, P, &mut rng.rail, year, &pbar, events);
        rail::plan_spurs(st, C, P, &mut rng.rail, year, &pbar, events);
        // canals (docs/rules/canals.md): their promoters look round too
        crate::canals::plan(st, C, P, year, &pbar, events);
        notable_closures(st, C, P, &closed, year, events);
        // the families (if any) act last, on the year's prices; they draw from the
        // misc stream, which the Python never uses
        families::yearly(st, C, P, &mut rng.misc, year, events);
    }
}

pub fn notable_closures(st: &State, C: &Content, _P: &Params, closed: &[usize], year: i64, events: &mut Vec<Event>) {
    let big: Vec<usize> = closed.iter().copied().filter(|&f| st.f_K[f] > 1.5 * C.m_K0[st.f_method[f]]).collect();
    for &fid in big.iter().take(2) {
        let i = st.f_town[fid];
        let owner = families::owner_of_firm(st, fid);
        let title = match owner {
            Some(k) => format!("The {} family's {} closes", st.families[k].surname, st.f_name[fid]),
            None => format!("{} closes", st.f_name[fid]),
        };
        let mut ev = Event::new(
            year as f64 + 0.9,
            "closure",
            0.4,
            title,
            format!(
                "The {} near {}, open since {}, can no longer cover its costs.",
                C.method_label[st.f_method[fid]].to_lowercase(),
                st.town_name[i],
                st.f_born[fid]
            ),
        );
        ev.town = Some(i);
        ev.cell = Some(st.f_cell[fid]);
        ev.firm = Some(fid);
        ev.family = owner;
        events.push(ev);
    }
}

/// Everything a run produces. Python: the dict `run()` returns.
#[derive(Clone, Debug)]
pub struct RunResult {
    pub state: State,
    pub content: Content,
    pub params: Params,
    pub events: Vec<Event>,
    pub rec: Record,
}

/// A run in progress: the state plus what the loop around it keeps.
#[derive(Clone, Debug)]
pub struct Sim {
    pub st: State,
    pub content: Content,
    pub params: Params,
    pub rngs: Rngs,
    pub events: Vec<Event>,
    pub rec: Record,
    /// The next year to simulate.
    pub year: i64,
    pub end_year: i64,
    /// Months recorded so far (the observer's time index).
    pub t: usize,
    /// Check every month that money only changed by what was issued (as Python's run does).
    pub check_money: bool,
}

/// World, warm-up and the 1750 baselines: everything before the first recorded month.
pub fn start(P: &Params, content: Content) -> Sim {
    let (st, rngs) = init_state(P, &content);
    start_from(P, content, st, rngs)
}

/// `start` from a given initial state and streams (the replay test uses
/// Python's world and draws).
pub fn start_from(P: &Params, content: Content, mut st: State, mut rngs: Rngs) -> Sim {
    let mut C = content;
    let y0 = C.start_year;
    let y1 = P.end_year.unwrap_or(C.end_year);
    C.end_year = y1;
    // warm-up: the 1740s settle into an equilibrium nobody records
    for wy in 0..P.warmup_years {
        for _ in 0..12 {
            econ::step_month(&mut st, &C, P);
        }
        yearly(&mut st, &C, P, &mut rngs, y0 - P.warmup_years + wy, &mut Vec::new(), true);
    }
    st.cumprod_1750 = st.cumprod.clone();
    let rate: Vec<f64> = st.cumprod.iter().map(|v| v / P.warmup_years.max(1) as f64).collect();
    let mut disc = C.tech_disc.clone();
    for t in 0..C.NT {
        for g in 0..C.G {
            let v = disc[(t, g)];
            if v < 0.0 {
                let x = -v * rate[g];
                disc[(t, g)] = if x <= 0.0 { 1e18 } else { x };
            }
        }
    }
    st.disc_X0 = disc;
    st.cumprod.iter_mut().for_each(|v| *v = 0.0);
    let ms = &st.month_stats;
    let mut basket = vec![0.0; C.G];
    let mut lord = vec![0.0; C.G];
    for t in 0..st.T_max {
        for g in 0..C.G {
            basket[g] += ms.hh_got[(t, g)];
            lord[g] += ms.lord_got[(t, g)];
        }
    }
    for g in 0..C.G {
        basket[g] += lord[g];
    }
    st.cpi_basket = basket;
    st.cpi_base_prices = observer::national_prices(&st);
    let T = st.T_max;
    let w: Vec<f64> = (0..T).map(|t| st.wage[t] * st.pop[t] * if st.town_active[t] { 1.0 } else { 0.0 }).collect();
    let pa: Vec<f64> = (0..T).map(|t| if st.town_active[t] { st.pop[t] } else { 0.0 }).collect();
    st.wage_base = pairwise_sum(&w) / pairwise_sum(&pa).max(1.0);
    let n_years = (y1 - y0 + 1) as usize;
    observer::init(&mut st, &C, P, n_years * 12);
    let rec = record::init(&st, &C, P, n_years);
    // families (if any) start here, on the warmed-up economy
    families::init(&mut st, &C, P, &mut rngs.misc);
    // chronicle: what the families start with counts as held
    crate::chronicle::init(&mut st, &C, P);
    st.money_start = econ::money_total(&st);
    st.issued = 0.0;
    Sim { st, content: C, params: P.clone(), rngs, events: Vec::new(), rec, year: y0, end_year: y1, t: 0, check_money: true }
}

/// One month of the recorded run: the economy, railway construction, the money check, the observer.
pub fn run_month(sim: &mut Sim, month: usize) -> Result<(), String> {
    let Sim { st, content: C, params: P, events, .. } = sim;
    econ::step_month(st, C, P);
    rail::progress(st, C, P, sim.year as f64 + (month as f64 + 0.5) / 12.0, events);
    // canals (docs/rules/canals.md): canals that are dug open
    crate::canals::progress(st, C, P, sim.year as f64 + (month as f64 + 0.5) / 12.0, events);
    st.money_now = econ::money_total(st);
    if sim.check_money {
        let drift = st.money_now - (st.money_start + st.issued);
        if drift.abs() >= 1e-6 * st.money_now.max(1.0) {
            return Err(format!("money not conserved: drift {drift:.6e} in {}-{month}", sim.year));
        }
    }
    observer::record_month(st, C, P, sim.t);
    sim.t += 1;
    Ok(())
}

/// Twelve months, then the yearly step, the observer's detectors and the yearly snapshot.
pub fn run_year(sim: &mut Sim) -> Result<(), String> {
    for month in 0..12 {
        run_month(sim, month)?;
    }
    let year = sim.year;
    let Sim { st, content: C, params: P, rngs, events, rec, .. } = sim;
    yearly(st, C, P, rngs, year, events, false);
    observer::yearly(st, C, P, year, events);
    // chronicle: the towns' stories, and the decades ranked with what the year added
    crate::chronicle::yearly(st, C, P, year, events);
    record::snapshot(rec, st, C, P, (year - C.start_year) as usize);
    harvest::snapshot(rec, st, P, (year - C.start_year) as usize);
    sim.year += 1;
    Ok(())
}

pub fn finish(sim: Sim) -> RunResult {
    let mut events = sim.events;
    events.sort_by(|a, b| a.year.total_cmp(&b.year));
    // chronicle: the marks a live game has reached, on the sorted list
    crate::chronicle::rank(&mut events, &sim.params, sim.content.start_year, sim.content.end_year, sim.year - 1);
    RunResult { state: sim.st, content: sim.content, params: sim.params, events, rec: sim.rec }
}

/// The whole history, with progress on stderr if asked. Errors if money is not conserved.
pub fn run_with(P: &Params, content: Content, progress: bool, check_money: bool) -> Result<RunResult, String> {
    let mut sim = start(P, content);
    sim.check_money = check_money;
    while sim.year <= sim.end_year {
        run_year(&mut sim)?;
        if progress && (sim.year % 10 == 0 || sim.year > sim.end_year) {
            eprintln!("year {} of {}", sim.year - 1, sim.end_year);
        }
    }
    Ok(finish(sim))
}

/// `run(params)`: the whole history in one call, with the content shipped in `data/content.toml`.
pub fn run(P: &Params) -> RunResult {
    run_with(P, default_content(), false, true).unwrap_or_else(|e| panic!("{e}"))
}
