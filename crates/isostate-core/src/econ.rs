//! The monthly economy: wages, production, consumption, markets, trade, money.
//! Python: `econ.py`.
//!
//! Every payment is a transfer between two accounts (households, landlords,
//! merchants, firms, the land fund, rail projects), so total money only changes
//! through explicit issuance. `money_total` checks that invariant.

use crate::content::Content;
use crate::engel;
use crate::families;
use crate::harvest;
use crate::mat::Mat;
use crate::params::{Params, RentRegime};
use crate::prices;
use crate::state::{MonthStats, State};
use crate::works;
use crate::util::{bincount, bool_f, pairwise_sum, pairwise_sum_by};

pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x.clamp(-50.0, 50.0)).exp())
}

/// Logit mix of goods for each household service.
///
/// Returns share (T, G * S), the fraction of a service bought from each good
/// (`share[(t, g * S + s)]`), and unit (T, S), the average price of one unit of service.
pub fn service_mix(price: &Mat, props: &Mat, theta: f64) -> (Mat, Mat) {
    let (T, G, S) = (price.rows, price.cols, props.cols);
    let mut share = Mat::zeros(T, G * S);
    let mut unit = Mat::zeros(T, S);
    let mut sv = vec![0.0; G];
    let mut z = vec![0.0; G];
    for t in 0..T {
        for s in 0..S {
            // price per unit of service; goods without the property cannot supply it
            let mut smin = f64::INFINITY;
            for g in 0..G {
                let v = if props[(g, s)] > 0.0 { price[(t, g)] / props[(g, s)] } else { f64::INFINITY };
                sv[g] = if v.is_finite() { v } else { f64::INFINITY };
                smin = smin.min(sv[g]);
            }
            let mut zsum = 0.0;
            for g in 0..G {
                z[g] = if sv[g].is_finite() { (-(sv[g] / smin - 1.0) / theta).exp() } else { 0.0 };
                zsum += z[g];
            }
            let mut u = 0.0;
            for g in 0..G {
                let sh = z[g] / zsum;
                share[(t, g * S + s)] = sh;
                u += sh * if sv[g].is_finite() { sv[g] } else { 0.0 };
            }
            unit[(t, s)] = u;
        }
    }
    (share, unit)
}

/// Logit mix of fuels for one buyer: writes the shares, returns the price of one unit of heat.
pub fn heat_mix(fuel_price: &[f64], heatval: &[f64], theta: f64, share: &mut [f64]) -> f64 {
    let n = fuel_price.len();
    let mut s = [0.0f64; 8];
    let mut z = [0.0f64; 8];
    let mut smin = f64::INFINITY;
    for k in 0..n {
        s[k] = fuel_price[k] / heatval[k];
        smin = smin.min(s[k]);
    }
    for k in 0..n {
        z[k] = (-(s[k] / smin - 1.0) / theta).exp();
    }
    let zsum = pairwise_sum(&z[..n]);
    for k in 0..n {
        share[k] = z[k] / zsum;
    }
    pairwise_sum_by(n, |k| share[k] * s[k])
}

/// Linear expenditure system over services with a subsistence floor.
/// Returns quantities (T, S) and the cost of the subsistence bundle per head.
pub fn les(budget_pc: &[f64], unit: &Mat, gamma: &[f64], beta: &[f64]) -> (Mat, Vec<f64>) {
    let (T, S) = (unit.rows, unit.cols);
    let mut q = Mat::zeros(T, S);
    let mut sub = vec![0.0; T];
    for t in 0..T {
        sub[t] = pairwise_sum_by(S, |s| gamma[s] * unit[(t, s)]);
        let extra = (budget_pc[t] - sub[t]).max(0.0);
        let scale = if budget_pc[t] < sub[t] { budget_pc[t] / sub[t].max(1e-12) } else { 1.0 };
        for s in 0..S {
            q[(t, s)] = gamma[s] * scale + beta[s] * extra / unit[(t, s)].max(1e-12);
        }
    }
    (q, sub)
}

/// Goods (T, G) needed to deliver service quantities q (T, S) with the given mix.
pub fn goods_from_services(q: &Mat, share: &Mat, props: &Mat) -> Mat {
    let (T, S) = (q.rows, q.cols);
    let G = props.rows;
    let mut d = Mat::zeros(T, G);
    for t in 0..T {
        for g in 0..G {
            let mut acc = 0.0;
            for s in 0..S {
                let p = props[(g, s)];
                let inv_prop = if p > 0.0 { 1.0 / p } else { 0.0 };
                acc += q[(t, s)] * share[(t, g * S + s)] * inv_prop;
            }
            d[(t, g)] = acc;
        }
    }
    d
}

/// Per-unit inputs (n, G), heat and labor; small firms and new methods are worse.
pub fn firm_coefficients(C: &Content, method: &[usize], K: &[f64], prod: &[f64]) -> (Mat, Vec<f64>, Vec<f64>) {
    let n = method.len();
    let G = C.G;
    let mut a = Mat::zeros(n, G);
    let mut h = vec![0.0; n];
    let mut l = vec![0.0; n];
    for i in 0..n {
        let m = method[i];
        let K0 = C.m_K0[m];
        let r = (K0 / K[i].max(1e-9)).sqrt().min(10.0);
        let f = (1.0 + 0.1 * r) / prod[i].powf(0.35);
        for g in 0..G {
            a[(i, g)] = C.m_in[(m, g)] * f;
        }
        // numpy turns `prod ** 0.5` into sqrt
        h[i] = C.m_heat[m] * (1.0 + 0.2 * r) / prod[i].sqrt();
        l[i] = C.m_labor[m] * (1.0 + 0.8 * r) / prod[i];
    }
    (a, h, l)
}

/// Productivity multiplier and per-unit machine inputs for one land sector in one town.
///
/// Tools make every land worker better (households' implements), improved
/// husbandry lifts farms and pastures, steam pumps and iron wagonways lift
/// mines, and farm machinery arrives late. Machine inputs are bought per unit
/// of output; the boost scales with how much of them the market could supply.
///
/// Returns (mult, machinery per unit, rails per unit, input cost per unit).
pub fn sector_tfp(st: &State, C: &Content, P: &Params, sec: usize, town: usize, price: &Mat, fill: &Mat) -> (f64, f64, f64, f64) {
    let gm = C.gi("machinery");
    let gr = C.gi("rails");
    let name = C.sector_name[sec].as_str();
    let k = &st.know;
    let mut mult = st.tools_mult[town];
    let ag = name == "farm" || name == "pasture";
    if ag {
        mult *= 1.0 + P.husbandry_boost * k[(town, st.t_husb)];
    }
    // machines (docs/rules/machines.md): bought where they pay, and better with the steel poured
    if P.machines.enabled {
        return crate::machines::sector_tfp(st, C, P, sec, town, price, mult);
    }
    let mine = C.sector_mine[sec];
    let (kp, kw, kf) = (k[(town, st.t_pump)], k[(town, st.t_wagon)], k[(town, st.t_fmach)]);
    let use_pump = bool_f(mine && kp > 0.05);
    let use_wagon = bool_f(mine && kw > 0.05);
    let use_fm = bool_f(name == "farm" && kf > 0.05);
    mult *= 1.0
        + use_pump * P.pump_boost * kp * fill[(town, gm)]
        + use_wagon * P.wagon_boost * kw * fill[(town, gr)]
        + use_fm * P.farm_mach_boost * kf * fill[(town, gm)];
    let mach = use_pump * P.pump_input + use_fm * P.farm_mach_input;
    let rails = use_wagon * P.wagon_input;
    let unit_cost = mach * price[(town, gm)] + rails * price[(town, gr)];
    (mult, mach, rails, unit_cost)
}

/// Land rent goes to the landlords of a town, or to the common fund.
pub fn pay_rent(st: &mut State, P: &Params, amount: &[f64]) {
    let total = pairwise_sum(amount);
    match P.rent_regime {
        RentRegime::Commons => st.fund_cash += total,
        RentRegime::Landlords => {
            for (li, a) in st.lord_income.iter_mut().zip(amount) {
                *li += a;
            }
        }
    }
    st.m_rent += total;
}

/// Midpoints this close to the root of excess demand are evaluated, not predicted.
/// Newton finds the root to about 1e-15; the final bisection interval is 1.3e-12 wide.
pub const ROOT_MARGIN: f64 = 1e-11;

/// Python's labour-market bisection for one town: 40 halvings of [lo, hi] in log
/// wage, moving up whenever demand at the midpoint exceeds supply; the result is
/// the midpoint of the last interval.
///
/// Each halving needs only the sign of excess demand at its midpoint, and excess
/// demand falls as the wage rises. Given `root`, the log wage where it changes
/// sign, a midpoint further than `ROOT_MARGIN` below the root must move up and one
/// further above must move down, so only the few midpoints within the margin are
/// evaluated, exactly as the plain bisection evaluates every one. With `root` None
/// every midpoint is evaluated.
pub fn bisect_log_wage(mut lo: f64, mut hi: f64, too_much: &dyn Fn(f64) -> bool, root: Option<f64>) -> f64 {
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        let up = match root {
            Some(r) if mid < r - ROOT_MARGIN => true,
            Some(r) if mid > r + ROOT_MARGIN => false,
            _ => too_much(mid),
        };
        if up {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Where excess demand changes sign in (lo, hi): -inf if it never exceeds supply
/// above lo, +inf if it still does at hi, otherwise the root found by Newton steps
/// on `f` (excess demand and its slope), kept inside the bracket by bisection.
/// None if the steps do not settle (the bisection then evaluates every midpoint).
pub fn excess_demand_root(lo: f64, hi: f64, too_much: &dyn Fn(f64) -> bool, f: impl Fn(f64) -> (f64, f64)) -> Option<f64> {
    if !too_much(lo) {
        return Some(f64::NEG_INFINITY);
    }
    if too_much(hi) {
        return Some(f64::INFINITY);
    }
    let (mut a, mut b) = (lo, hi);
    let mut x = 0.5 * (lo + hi);
    for _ in 0..100 {
        let (fx, dfx) = f(x);
        if !fx.is_finite() || !dfx.is_finite() {
            return None;
        }
        if fx > 0.0 {
            a = x;
        } else {
            b = x;
        }
        let mut xn = x - fx / dfx;
        if !(dfx < 0.0 && xn > a && xn < b) {
            xn = 0.5 * (a + b);
        }
        if (xn - x).abs() <= 1e-15 * (1.0 + x.abs()) || b - a <= 1e-15 * (1.0 + a.abs()) {
            return Some(xn);
        }
        x = xn;
    }
    None
}

pub fn money_total(st: &State) -> f64 {
    let firm_cash: Vec<f64> =
        (0..st.f_cash.len()).filter(|&f| st.f_alive[f] || st.f_cash[f] != 0.0).map(|f| st.f_cash[f]).collect();
    let mut projects = 0.0;
    for pr in &st.projects {
        projects += pr.cash;
    }
    let total = pairwise_sum(&st.hh_cash)
        + pairwise_sum(&st.lord_cash)
        + pairwise_sum(&st.merch_cash)
        + pairwise_sum(&firm_cash)
        + st.fund_cash
        + projects
        + pairwise_sum(&st.hh_income)
        + pairwise_sum(&st.lord_income);
    if families::active(st) { total + families::cash_total(st) } else { total }
}

/// Rents paid cell by cell (by firms, or by land units): those on family land go
/// to the family, the rest to each town's landlords as `pay_rent` does.
fn pay_cell_rents(st: &mut State, P: &Params, town: &[usize], cell: &[usize], rent: &[f64], T: usize) {
    if families::active(st) {
        let rest = families::take_rents(st, cell, rent);
        pay_rent(st, P, &bincount(town, &rest, T));
    } else {
        pay_rent(st, P, &bincount(town, rent, T));
    }
}

/// One month of the economy: demand, the labour market, requests to each town's
/// market, production, money, stocks, trade, prices, dividends and the issue of
/// money, then the month's bookkeeping.
///
/// The order of what follows is part of the result, and nothing but the order
/// says so. A read moved across a write below makes a different island without
/// failing any rule's own test; the game's golden test (`tests/golden_game.rs`)
/// is what sees it, and the port's (`tests/golden.rs`) only where the port is
/// touched.
///
/// Last month's numbers. These are read before this month writes them, and are
/// meant to be last month's:
/// - `st.wage`, until it is replaced near the end. The labour market's bracket
///   for each town's wage is half to twice it (kept within 0.02 and 500), and
///   the carters, the navvies and the menders are last month's money divided
///   by it, each only where it is above nothing (`carting::take_carters`,
///   `canals::take_navvies`, `roads::take_menders`).
/// - `st.fill`, until it is replaced beside the wage. Under the port the inputs
///   that lift the land's output (`sector_tfp`) are counted at last month's
///   fill. Under the game's machines (`machines::sector_tfp`) the lift is the
///   machines standing, `st.machine_stock`, which `machines::end_month` moved
///   at the end of last month on that month's fill. Either way the lift also
///   reads `st.tools_mult`, written in the bookkeeping at the month's end.
/// - `st.hh_income_prev` and `st.lord_income_prev`, which set this month's
///   budgets and are written only once this month's incomes are paid.
/// - `st.f_fill` (`phi_prev`), each works' smoothed fill, read before execution
///   writes this month's into it.
/// - `st.ex_Lprev` and `st.f_labor`, the hands each of the land's units and each
///   works had last month, from which this month's demand for hands moves only
///   part of the way (`labor_adjust_ex`, `labor_adjust_firm`).
/// - `st.pexp`, the prices expected, at which the land's units reckon what their
///   crop and their machines are worth; it follows this month's prices only in
///   "prices respond".
/// - `st.cart_bill`, read and emptied by `take_carters` at the start of the
///   labour market and filled again after it by the works', the land's and the
///   trade's carting; and each project's `navvy_pay`, set when the project pays
///   this month and read by the navvies and menders next month.
///
/// The labour market. The four takers of hands come in this order before the
/// wage is sought: carters, sinkers (`mines::take_crews`), navvies, menders.
/// The order is part of the result, since each of the last three caps itself
/// on the hands the ones before it left. The merchants' take
/// (`prices::merchants_take`) is fixed before the wage is sought and does not
/// depend on it. Once the wage is found, and before the hands are counted at
/// it, a town short of hands even at the top of its bracket is held there
/// (`hands::short_towns`, reading the bracket and the supply the four takers
/// left); after the hands are counted its employers are cut to the hands it has
/// (`hands::ration`), and only then is `st.ex_Lprev` written.
///
/// Within the month:
/// - Prices are read all month and move only in "prices respond", after trade
///   and after the books have taken the month: everything bought is paid at the
///   prices the month began with, and `st.pexp` follows the new price.
/// - The fill is the stock at the start of the month against all that was
///   asked of it, so `st.requests` is copied only once every request is in
///   (households, landlords, works, the land's machines and rails, orders for
///   capacity, projects). Production joins the stocks after the sales leave
///   and decay is taken, and trade moves the stocks after that.
/// - The harvest is half foreseen before the hands are hired
///   (`harvest::foreseen`) and felt after the machines and rails are asked for
///   on an average year's yield (`harvest::brought_in`); from there on `Y` is
///   the crop brought in.
/// - The sinkers are paid before the royalty is worked out (`mines::pay_crews`)
///   and counted as hands after `st.ex_labor` is written (`mines::add_crews`).
/// - The issue of new money is a share of `money_total` counted after the
///   works', the merchants' and the common fund's dividends have left their
///   purses and before the households' income is credited (see the money check
///   in `sim::run_month`).
/// - A works' dividend is reckoned after the month's deliveries of plant and
///   after prices move, so what it keeps back for its order
///   (`works::kept_for_order`) is what is still to come, at the new prices.
/// - `towns::month` reads `st.f_labor` after it is written; `machines::end_month`
///   comes last because it reads this month's `st.fill` and `st.pexp`.
/// - The books only watch. `accounts::open_month` opens them before anything
///   moves, and every other call into `accounts` comes before what it reads is
///   changed: the trade inside the trade, the month before prices move, the
///   dividends before the families take theirs out of the list, the incomes
///   before the purses are credited.
/// - The families' hooks stand inside the port's own steps (the budgets, the
///   land's yield, the works' inputs, rents, spending, hired managers,
///   dividends), each where the port's arithmetic it shares out stands, and
///   `families::end_month` comes after the purses are credited.
///
/// Chance. The month draws no random number and is handed no stream. What
/// chance it feels was drawn in the year (`sim::yearly`), from the streams of
/// `rng.rs`: the harvest's weather is drawn by `harvest::yearly` from its own
/// stream and read here from `st.harvest_mult`. A rule that wants chance in
/// the month draws it there and keeps it in the state, as the harvest does; a
/// draw made here, or moved within the year, would change every draw after it
/// on that stream.
pub fn step_month(st: &mut State, C: &Content, P: &Params) {
    let T = st.T_max;
    let G = C.G;
    let gm = C.gi("machinery");
    let gr = C.gi("rails");
    let fuels = C.fuel_goods.clone();
    let NF = fuels.len();
    let heat_f: Vec<f64> = fuels.iter().map(|&g| C.heat[g]).collect();
    st.m_rent = 0.0;
    // accounts (docs/rules/accounts.md): the books open on the month's stocks
    crate::accounts::open_month(st);
    // money (docs/rules/money.md): the month's issue is reckoned on the money
    // there is now, before any of it is between purses
    let money_base = crate::money::month_base(st, P);

    // ---------------- households: what they want to buy
    let (share, unit) = service_mix(&st.price, &C.props, P.logit_theta);
    let mut urban_pay = vec![0.0; T];
    let mut pc = vec![0.0; T];
    for t in 0..T {
        let hh_cash = st.hh_cash[t].max(0.0);
        let budget = hh_cash.min(P.cons_income * st.hh_income_prev[t].max(0.0) + P.cons_wealth * hh_cash);
        urban_pay[t] = st.urban_rent[t].min(0.5 * budget);
        let goods_budget = budget - urban_pay[t];
        pc[t] = if st.pop[t] > 0.0 { goods_budget / st.pop[t].max(1.0) } else { 0.0 };
    }
    // engel: past a reference income the households' marginal shares lean toward
    // a richer set (docs/rules/engel.md); landlords, below, keep their own
    let (q_hh, subcost) = if P.engel.enabled {
        engel::les(&pc, &unit, &C.subsistence, &C.shares, &C.shares_rich, &P.engel)
    } else {
        les(&pc, &unit, &C.subsistence, &C.shares)
    };
    let mut d_hh = goods_from_services(&q_hh, &share, &C.props);
    for t in 0..T {
        for v in d_hh.row_mut(t) {
            *v *= st.pop[t];
        }
    }

    let lord_pop: Vec<f64> = st.pop.iter().map(|&p| P.lord_share * p).collect();
    // families (if any) spend their luxury budgets through their home town's landlords
    let fam_budget = families::spending_budgets(st, P);
    let fam_by_town = families::budgets_by_town(st, &fam_budget, T);
    let mut lord_budget = vec![0.0; T];
    let mut lpc = vec![0.0; T];
    for t in 0..T {
        let lord_cash = st.lord_cash[t].max(0.0);
        let mut lbudget = lord_cash.min(P.lord_cons_income * st.lord_income_prev[t].max(0.0) + P.lord_cons_wealth * lord_cash);
        if !fam_budget.is_empty() {
            lbudget += fam_by_town[t];
        }
        lord_budget[t] = lbudget;
        lpc[t] = if lord_pop[t] > 0.0 { lbudget / lord_pop[t].max(1e-9) } else { 0.0 };
    }
    let (q_l, _) = les(&lpc, &unit, &C.subsistence, &C.lord_shares);
    let mut d_lord = goods_from_services(&q_l, &share, &C.props);
    for t in 0..T {
        for v in d_lord.row_mut(t) {
            *v *= lord_pop[t];
        }
    }

    // ---------------- land-based production: cell enterprises hire at the town wage
    let ex = st.ex_cell.clone();
    let sec = st.ex_sector.clone();
    let et = st.ex_town.clone();
    let NE = ex.len();
    let NS = C.NS;
    let pe = st.pexp.clone();
    // sector_tfp depends only on (sector, town): evaluate it once per pair
    let mut tfp = vec![(0.0, 0.0, 0.0, 0.0); T * NS];
    let mut tfp_done = vec![false; T * NS];
    let mut g_s = vec![0usize; NE];
    let mut alpha = vec![0.0; NE];
    let mut f = vec![0.0; NE];
    let mut A = vec![0.0; NE];
    let mut mach_unit = vec![0.0; NE];
    let mut rail_unit = vec![0.0; NE];
    let mut Cc = vec![0.0; NE];
    let land_boost = families::land_boost(st, C, P);
    for u in 0..NE {
        let (s, t) = (sec[u], et[u]);
        g_s[u] = C.sector_good[s];
        alpha[u] = C.sector_alpha[s];
        f[u] = if s == st.forestry_sector { st.cover[ex[u]] } else { st.ex_field[u] };
        if !tfp_done[t * NS + s] {
            tfp[t * NS + s] = sector_tfp(st, C, P, s, t, &pe, &st.fill);
            tfp_done[t * NS + s] = true;
        }
        let (mult, mu, ru, in_unit) = tfp[t * NS + s];
        mach_unit[u] = mu;
        rail_unit[u] = ru;
        A[u] = C.sector_A[s] * mult;
        if let Some(b) = &land_boost {
            A[u] *= b[u];
        }
        let npr = pe[(t, g_s[u])] - st.ex_tau[u] - in_unit;
        Cc[u] = if npr > 0.0 && f[u] > 0.0 { (1.0 - alpha[u]) * A[u] * f[u] * npr } else { 0.0 };
    }
    // harvest: what can be seen of the year's weather when the hands are hired
    // (half of it in the game, nothing at `harvest.foreseen` 0)
    harvest::foreseen(st, P, &mut Cc);
    let lam = st.ex_lam.clone();

    // ---------------- firms: unit economics at their own location
    let fi: Vec<usize> = (0..st.f_alive.len()).filter(|&i| st.f_alive[i] && st.f_K[i] > 1e-9).collect();
    let NFI = fi.len();
    let ft: Vec<usize> = fi.iter().map(|&i| st.f_town[i]).collect();
    let fm: Vec<usize> = fi.iter().map(|&i| st.f_method[i]).collect();
    let fc: Vec<usize> = fi.iter().map(|&i| st.f_cell[i]).collect();
    let K: Vec<f64> = fi.iter().map(|&i| st.f_K[i]).collect();
    let fprod: Vec<f64> = fi.iter().map(|&i| st.f_prod[i]).collect();
    let (mut a, mut h, mut l) = firm_coefficients(C, &fm, &K, &fprod);
    works::walk_to_work(st, P, &fc, &mut l); // works: hands walk out from the town
    if families::active(st) {
        families::scale_firm_inputs(st, C, P, &fi, &mut a, &mut h, &mut l);
    }
    let out: Vec<usize> = fm.iter().map(|&m| C.method_out[m]).collect();
    let p = &st.price;
    let mut pin = Mat::zeros(NFI, G);
    for i in 0..NFI {
        for g in 0..G {
            pin[(i, g)] = (p[(ft[i], g)] + st.delta[(g, fc[i])]).max(0.25 * p[(ft[i], g)]);
        }
    }
    let mut hshare = Mat::zeros(NFI, NF);
    let mut hunit = vec![0.0; NFI];
    let mut fuel_price = vec![0.0; NF];
    for i in 0..NFI {
        for (k, &g) in fuels.iter().enumerate() {
            fuel_price[k] = pin[(i, g)];
        }
        hunit[i] = heat_mix(&fuel_price, &heat_f, P.heat_theta, hshare.row_mut(i));
    }
    let pout: Vec<f64> =
        (0..NFI).map(|i| p[(ft[i], out[i])] - P.freight * C.weight[out[i]] * st.dcatch[fc[i]]).collect();
    let nonlabor: Vec<f64> =
        (0..NFI).map(|i| pairwise_sum_by(G, |g| a[(i, g)] * pin[(i, g)]) + h[i] * hunit[i]).collect();
    let phi_prev: Vec<f64> = fi.iter().map(|&i| st.f_fill[i]).collect();

    // ---------------- labour market: one wage per town clears supply and demand
    let mut supply: Vec<f64> = st.pop.iter().map(|&p| P.participation * p).collect();
    // carting (docs/rules/carting.md): the hands that carted last month's goods are not for hire
    crate::carting::take_carters(st, P, &mut supply);
    // mines (docs/rules/mines.md): nor are the hands sinking new workings
    crate::mines::take_crews(st, P, &mut supply);
    // canals (docs/rules/canals.md): nor are the navvies digging a canal. They come
    // last of the three: each caps itself on the hands still left, the carters and
    // the sinkers are at work the town already lives by, and a canal can wait
    crate::canals::take_navvies(st, P, &mut supply);
    // roads (docs/probes/dx-roads.md): nor are the men making and mending roads
    crate::roads::take_menders(st, P, &mut supply);
    let supply = supply;
    // Short-run labour demand moves only part of the way to the long-run optimum,
    // so a price blip does not empty the fields in a month.
    let prev_w = st.wage.clone();
    let mut lo = vec![0.0; T];
    let mut hi = vec![0.0; T];
    for t in 0..T {
        let has = prev_w[t] > 0.0;
        let safe_w = if has { prev_w[t] } else { 1.0 };
        lo[t] = if has { (safe_w / 2.0).max(0.02).ln() } else { 0.02f64.ln() };
        hi[t] = if has { (safe_w * 2.0).min(500.0).ln() } else { 500.0f64.ln() };
    }
    let inv_alpha: Vec<f64> = alpha.iter().map(|&x| 1.0 / x).collect();
    let (kx, kf) = (P.labor_adjust_ex, P.labor_adjust_firm);
    let Lprev_ex = st.ex_Lprev.clone();
    let Lprev_f: Vec<f64> = fi.iter().map(|&i| st.f_labor[i]).collect();

    // prices: the share of each works' capacity the merchants will take this month,
    // less than 1 only where their stock of its good is long (all ones while the
    // mechanism is off). It is fixed before the wage is sought and does not depend
    // on it, so demand still falls as the wage rises.
    let take = prices::merchants_take(st, P, &ft, &out);

    // Firm demand at its town's wage w: utilisation follows the margin through a sigmoid.
    let firm_demand = |w: f64, i: usize| -> (f64, f64) {
        let m = pout[i] - nonlabor[i] - l[i] * w;
        let u = sigmoid((m / pout[i].max(1e-9) - P.util_target_margin) / P.util_band) * bool_f(pout[i] > 0.0) * take[i];
        let Lf = Lprev_f[i] + kf * (K[i] * u * l[i] * phi_prev[i] - Lprev_f[i]);
        (Lf.max(0.0), u)
    };

    // The bisection asks 40 times per town whether demand exceeds supply. The land part,
    //   Lx = Lprev + kx * (lam * (Cc / w)^(1/alpha) - Lprev),
    // is linear in w^(-1/alpha) once lam * Cc^(1/alpha) is known, so it is summed
    // per (town, sector) once and each step costs NS powers instead of one per
    // unit. Python clamps Lx at zero, which never binds for 0 <= kx <= 1 (Lprev,
    // lam and Cc are not negative); otherwise the units are summed one by one.
    let aggregate = (0.0..=1.0).contains(&kx);
    let mut x_pow = vec![0.0; NE]; // lam * Cc^(1/alpha)
    let mut sum_x = vec![0.0; T * NS];
    let mut sum_l = vec![0.0; T];
    let sector_inv_alpha: Vec<f64> = C.sector_alpha.iter().map(|&x| 1.0 / x).collect();
    let mut town_units: Vec<Vec<usize>> = vec![Vec::new(); T];
    for u in 0..NE {
        x_pow[u] = lam[u] * Cc[u].powf(inv_alpha[u]);
        sum_x[et[u] * NS + sec[u]] += x_pow[u];
        sum_l[et[u]] += Lprev_ex[u];
        if !aggregate {
            town_units[et[u]].push(u);
        }
    }
    // each town's firms, in the order the bisection sums them
    let mut town_firms: Vec<Vec<usize>> = vec![Vec::new(); T];
    for i in 0..NFI {
        town_firms[ft[i]].push(i);
    }
    // Labour demand of town t at log wage x, summed as one bisection step sums it,
    // and (with `slope`) its derivative in x for the Newton steps.
    let town_demand = |t: usize, x: f64, slope: bool| -> (f64, f64) {
        let wt = x.exp();
        let (mut d, mut dd) = (0.0, 0.0);
        if aggregate {
            let mut s_tw = 0.0;
            for s in 0..NS {
                let sx = sum_x[t * NS + s];
                if sx != 0.0 {
                    let wp = wt.powf(-sector_inv_alpha[s]);
                    s_tw += sx * wp;
                    dd -= kx * sector_inv_alpha[s] * sx * wp;
                }
            }
            d = sum_l[t] + kx * (s_tw - sum_l[t]);
        } else {
            for &u in &town_units[t] {
                let Lx = Lprev_ex[u] + kx * (lam[u] * (Cc[u] / wt).powf(inv_alpha[u]) - Lprev_ex[u]);
                d += Lx.max(0.0);
                if Lx > 0.0 {
                    dd -= kx * inv_alpha[u] * lam[u] * (Cc[u] / wt).powf(inv_alpha[u]);
                }
            }
        }
        let mut df = 0.0;
        for &i in &town_firms[t] {
            let (lf, u) = firm_demand(wt, i);
            df += lf;
            if slope && lf > 0.0 && u > 0.0 {
                // dLf/dx = kf K l phi du/dx, du/dx = u (1 - u) dz/dx, dz/dx = -l w / (pout band)
                let z = ((pout[i] - nonlabor[i] - l[i] * wt) / pout[i].max(1e-9) - P.util_target_margin) / P.util_band;
                if z.abs() < 50.0 {
                    let dz = -l[i] * wt / pout[i].max(1e-9) / P.util_band;
                    // prices: u is the merchants' take times the sigmoid s, so du/dz is
                    // take s (1 - s) = u (1 - u / take); with a take of 1, the port's line
                    dd += kf * K[i] * l[i] * phi_prev[i] * u * (1.0 - u / take[i]) * dz;
                }
            }
        }
        (d + df, dd)
    };
    let mut w = vec![0.0; T];
    for t in 0..T {
        let too_much = |x: f64| town_demand(t, x, false).0 > supply[t];
        // prices: a test may ask for the plain bisection, to check the shortcut against it
        let root = if st.plain_bisection {
            None
        } else {
            excess_demand_root(lo[t], hi[t], &too_much, |x| {
                let (d, dd) = town_demand(t, x, true);
                (d - supply[t], dd)
            })
        };
        w[t] = bisect_log_wage(lo[t], hi[t], &too_much, root).exp();
        if !st.town_active[t] {
            w[t] = 0.0;
        }
    }
    // hands (docs/rules/hands.md): a town still short of hands at the month's
    // highest wage is held at that wage, and below its employers are cut to the
    // hands it has (no town is short, and nothing is asked, while the rule is off)
    let short = crate::hands::short_towns(st, P, &hi, &supply, &|t, x| town_demand(t, x, false).0, &mut w);
    // final demand at the clearing wage, unit by unit. Python evaluates
    // lam * (Cc / w)^(1/alpha); lam * Cc^(1/alpha) * w^(-1/alpha) is the same number
    // (to an ulp or two) and reuses the power already taken for the bisection.
    let wq: Vec<f64> = w.iter().map(|&x| x.max(1e-9)).collect();
    let mut wpow = vec![0.0; T * NS];
    for t in 0..T {
        for s in 0..NS {
            wpow[t * NS + s] = wq[t].powf(-sector_inv_alpha[s]);
        }
    }
    let mut Lex: Vec<f64> = (0..NE)
        .map(|u| (Lprev_ex[u] + kx * (x_pow[u] * wpow[et[u] * NS + sec[u]] - Lprev_ex[u])).max(0.0))
        .collect();
    let mut Lf = vec![0.0; NFI];
    let mut ufirm = vec![0.0; NFI];
    for i in 0..NFI {
        let (x, u) = firm_demand(wq[ft[i]], i);
        Lf[i] = x;
        ufirm[i] = u;
    }
    // hands: every employer of a short town gets the same share of what it asked
    // for, and carries into next month what it got
    crate::hands::ration(&short, &supply, &et, &ft, &mut Lex, &mut Lf);
    st.ex_Lprev = Lex.clone();
    // Python: A * f * lam**alpha * Lex**(1 - alpha). lam > 0 on every unit, so
    // lam * (Lex / lam)**(1 - alpha) is the same number with one power instead of two.
    let Y: Vec<f64> = (0..NE).map(|u| A[u] * f[u] * (lam[u] * (Lex[u] / lam[u]).powf(1.0 - alpha[u]))).collect();
    let q_des: Vec<f64> = (0..NFI)
        .map(|i| {
            let q = if l[i] > 0.0 { Lf[i] / (l[i] * phi_prev[i].max(0.05)).max(1e-12) } else { 0.0 };
            q.min(K[i])
        })
        .collect();

    // ---------------- requests to each town market
    let mut R = Mat::zeros(T, G);
    for t in 0..T {
        for g in 0..G {
            R[(t, g)] = d_hh[(t, g)] + d_lord[(t, g)];
        }
    }
    let mut req_f = Mat::zeros(NFI, G);
    for i in 0..NFI {
        for g in 0..G {
            req_f[(i, g)] = q_des[i] * a[(i, g)];
        }
        let heat_need = q_des[i] * h[i];
        for (k, &g) in fuels.iter().enumerate() {
            req_f[(i, g)] += heat_need * hshare[(i, k)] / heat_f[k];
        }
    }
    add_by_town(&mut R, &ft, &req_f);
    let mine_mach: Vec<f64> = (0..NE).map(|u| Y[u] * mach_unit[u]).collect();
    let mine_rail: Vec<f64> = (0..NE).map(|u| Y[u] * rail_unit[u]).collect();
    add_column(&mut R, gm, &bincount(&et, &mine_mach, T));
    add_column(&mut R, gr, &bincount(&et, &mine_rail, T));
    // harvest: the year's weather is felt here, after the hands are hired and the
    // machines asked for on an average year's yield. From here on Y is the crop
    // actually brought in.
    let Y = harvest::brought_in(st, C, P, Y);
    let oi: Vec<usize> = (0..st.f_alive.len()).filter(|&i| st.f_alive[i] && st.f_order_K[i] > 0.0).collect();
    let ot: Vec<usize> = oi.iter().map(|&i| st.f_town[i]).collect();
    let mut req_inv = Mat::zeros(oi.len(), G);
    for (r, &i) in oi.iter().enumerate() {
        for g in 0..G {
            req_inv[(r, g)] = st.f_order[(i, g)] * 0.25;
        }
    }
    add_by_town(&mut R, &ot, &req_inv);
    let mut proj_req: Vec<Vec<f64>> = Vec::with_capacity(st.projects.len());
    for pr in &st.projects {
        // railways (docs/rules/railways.md): the game's lines say how long they take,
        // and order no more than the cash in hand will pay for
        let months = if pr.months > 0.0 { pr.months } else { 10.0 };
        let mut rq: Vec<f64> = (0..G).map(|g| pr.order[g].min(pr.need[g] / months)).collect();
        crate::ways::within_means(pr, &mut rq, st.price.row(pr.town));
        for g in 0..G {
            R[(pr.town, g)] += rq[g];
        }
        proj_req.push(rq);
    }

    // people (docs/rules/people.md) keeps it too: what a district's buyers asked for
    // goes with them when the district changes market
    if families::active(st) || P.people.enabled {
        // what each town's buyers asked for, for the families' site sums
        st.requests = R.clone();
    }
    let mut fill = Mat::zeros(T, G);
    for t in 0..T {
        for g in 0..G {
            let r = R[(t, g)];
            fill[(t, g)] = if r > 1e-12 { (st.inv[(t, g)] / r.max(1e-12)).min(1.0) } else { 1.0 };
        }
    }

    // ---------------- execute: firms produce what their scarcest input allows
    let mut q = vec![0.0; NFI];
    let mut use_f = Mat::zeros(NFI, G);
    let mut labor_f = vec![0.0; NFI];
    for i in 0..NFI {
        let t = ft[i];
        let mut fill_in = f64::INFINITY;
        for g in 0..G {
            fill_in = fill_in.min(if a[(i, g)] > 0.0 { fill[(t, g)] } else { 1.0 });
        }
        let fill_heat = if h[i] > 0.0 { pairwise_sum_by(NF, |k| hshare[(i, k)] * fill[(t, fuels[k])]) } else { 1.0 };
        let phi = fill_in.min(fill_heat);
        q[i] = q_des[i] * phi;
        for g in 0..G {
            use_f[(i, g)] = q[i] * a[(i, g)];
        }
        for (k, &g) in fuels.iter().enumerate() {
            use_f[(i, g)] += q[i] * h[i] * hshare[(i, k)] / heat_f[k];
        }
        let lf = Lf[i] * if phi_prev[i] > 0.0 { (phi / phi_prev[i].max(0.05)).min(1.0) } else { 0.0 };
        labor_f[i] = lf.min(q[i] * l[i] * 1.5 + 1e-9);
        let fid = fi[i];
        st.f_fill[fid] = 0.7 * phi_prev[i] + 0.3 * phi;
        st.f_u[fid] = 0.9 * st.f_u[fid] + 0.1 * ufirm[i] * phi;
    }

    let mut hh_got = Mat::zeros(T, G);
    let mut lord_got = Mat::zeros(T, G);
    let mut sold = Mat::zeros(T, G);
    for t in 0..T {
        for g in 0..G {
            hh_got[(t, g)] = d_hh[(t, g)] * fill[(t, g)];
            lord_got[(t, g)] = d_lord[(t, g)] * fill[(t, g)];
            sold[(t, g)] = hh_got[(t, g)] + lord_got[(t, g)];
        }
    }
    let mach_got: Vec<f64> = (0..NE).map(|u| mine_mach[u] * fill[(et[u], gm)]).collect();
    let rail_got: Vec<f64> = (0..NE).map(|u| mine_rail[u] * fill[(et[u], gr)]).collect();
    let mut inv_got = Mat::zeros(oi.len(), G);
    for r in 0..oi.len() {
        for g in 0..G {
            inv_got[(r, g)] = req_inv[(r, g)] * fill[(ot[r], g)];
        }
    }
    // per good: sold += bincount(ft, use_f[:, g]); sold += bincount(ot, inv_got[:, g])
    let by_firm = sum_by_town(T, &ft, &use_f);
    let by_order = sum_by_town(T, &ot, &inv_got);
    for k in 0..T * G {
        sold.data[k] += by_firm.data[k];
        sold.data[k] += by_order.data[k];
    }
    add_column(&mut sold, gm, &bincount(&et, &mach_got, T));
    add_column(&mut sold, gr, &bincount(&et, &rail_got, T));
    let mut proj_got: Vec<Vec<f64>> = Vec::with_capacity(st.projects.len());
    for (pr, rq) in st.projects.iter().zip(&proj_req) {
        let got: Vec<f64> = (0..G).map(|g| rq[g] * fill[(pr.town, g)]).collect();
        for g in 0..G {
            sold[(pr.town, g)] += got[g];
        }
        proj_got.push(got);
    }

    // ---------------- money flows
    let mut merch = vec![0.0; T];
    let mut hh_inc = vec![0.0; T];
    let mut sales_value = vec![0.0; T];

    // firms
    let p = &st.price;
    let pay_in: Vec<f64> = (0..NFI).map(|i| pairwise_sum_by(G, |g| use_f[(i, g)] * pin[(i, g)])).collect();
    let cart_in: Vec<f64> =
        (0..NFI).map(|i| pairwise_sum_by(G, |g| use_f[(i, g)] * (pin[(i, g)] - p[(ft[i], g)]).max(0.0))).collect();
    let spread = P.merchant_spread;
    let rev: Vec<f64> = (0..NFI).map(|i| q[i] * (pout[i] - spread * p[(ft[i], out[i])])).collect();
    let cart_out: Vec<f64> = (0..NFI).map(|i| q[i] * (p[(ft[i], out[i])] - pout[i])).collect();
    let wages_f: Vec<f64> = (0..NFI).map(|i| labor_f[i] * w[ft[i]]).collect();
    let rent_f: Vec<f64> = fi.iter().map(|&i| st.f_rent[i]).collect();
    let margin_v: Vec<f64> = (0..NFI).map(|i| q[i] * spread * p[(ft[i], out[i])]).collect();
    let merch_f: Vec<f64> = (0..NFI).map(|i| pay_in[i] - cart_in[i] - rev[i] - cart_out[i]).collect();
    let hh_f: Vec<f64> = (0..NFI).map(|i| cart_in[i] + cart_out[i] + wages_f[i]).collect();
    for (k, &i) in fi.iter().enumerate() {
        let profit = rev[k] - pay_in[k] - wages_f[k] - rent_f[k];
        st.f_cash[i] += profit;
        st.f_profit_acc[i] += profit;
        st.f_out_acc[i] += q[k];
        st.f_cost_m[i] = pay_in[k] + wages_f[k] + rent_f[k];
    }
    // family works with no branch over them pay a hired manager out of their profit
    // (accounts: the cuts are kept until the books have read them)
    let hired = families::hired_cut(st, C, P, &fi);
    if let Some(cut) = &hired {
        for (k, &i) in fi.iter().enumerate() {
            let profit = rev[k] - pay_in[k] - wages_f[k] - rent_f[k];
            if cut[k] > 0.0 && profit > 0.0 {
                let pay = cut[k] * profit;
                st.f_cash[i] -= pay;
                hh_inc[ft[k]] += pay;
            }
        }
    }
    add_to(&mut merch, &bincount(&ft, &merch_f, T));
    st.m_merch_margin = pairwise_sum(&margin_v);
    add_to(&mut sales_value, &bincount(&ft, &pay_in, T));
    add_to(&mut hh_inc, &bincount(&ft, &hh_f, T));
    // carting (docs/rules/carting.md): the works' carting, in and out, goes on the town's bill
    crate::carting::add_bill(st, P, &ft, (0..NFI).map(|i| cart_in[i] + cart_out[i]));
    pay_cell_rents(st, P, &ft, &fc, &rent_f, T);

    // land-based enterprises
    let p = &st.price;
    let value: Vec<f64> = (0..NE).map(|u| p[(et[u], g_s[u])] * Y[u]).collect();
    let in_paid: Vec<f64> = (0..NE).map(|u| mach_got[u] * p[(et[u], gm)] + rail_got[u] * p[(et[u], gr)]).collect();
    let mut wage_ex: Vec<f64> = (0..NE).map(|u| w[et[u]] * Lex[u]).collect();
    crate::mines::pay_crews(st, P, &w, &mut wage_ex); // mines: the sinkers are paid before the royalty
    let cart_ex: Vec<f64> = (0..NE).map(|u| st.ex_tau[u] * Y[u]).collect();
    let rent_ex: Vec<f64> = (0..NE).map(|u| value[u] * (1.0 - spread) - cart_ex[u] - wage_ex[u] - in_paid[u]).collect();
    let merch_ex: Vec<f64> = (0..NE).map(|u| in_paid[u] - value[u] * (1.0 - spread)).collect();
    let hh_ex: Vec<f64> = (0..NE).map(|u| wage_ex[u] + cart_ex[u]).collect();
    add_to(&mut merch, &bincount(&et, &merch_ex, T));
    add_to(&mut sales_value, &bincount(&et, &in_paid, T));
    add_to(&mut hh_inc, &bincount(&et, &hh_ex, T));
    // carting: so does the land's produce on its way to market
    crate::carting::add_bill(st, P, &et, cart_ex.iter().copied());
    // mines: a working's royalty goes to the cells whose deposit it works
    // (accounts: the lists are kept until the books have read them)
    let royalties = crate::mines::royalty_cells(st, P, &et, &ex, &rent_ex);
    match &royalties {
        Some((towns, cells, parts)) => pay_cell_rents(st, P, towns, cells, parts, T),
        None => pay_cell_rents(st, P, &et, &ex, &rent_ex, T),
    }
    crate::mines::note_month(st, P, &Y, &Lex, &rent_ex); // mines: each working's books for the year

    // engel: what households spent on each need, noted for the observer before prices move
    if P.engel.enabled {
        engel::note_month(st, C, &pc, &subcost, &q_hh, &share, &fill);
    }

    // households and landlords buy goods; households pay urban rent
    let p = &st.price;
    let hh_spend: Vec<f64> = (0..T).map(|t| pairwise_sum_by(G, |g| hh_got[(t, g)] * p[(t, g)])).collect();
    let lord_spend: Vec<f64> = (0..T).map(|t| pairwise_sum_by(G, |g| lord_got[(t, g)] * p[(t, g)])).collect();
    for t in 0..T {
        st.hh_cash[t] -= hh_spend[t] + urban_pay[t];
        st.lord_cash[t] -= lord_spend[t];
        merch[t] += hh_spend[t] + lord_spend[t];
        sales_value[t] += hh_spend[t] + lord_spend[t];
    }
    if families::active(st) {
        families::pay_spending(st, &fam_budget, &lord_budget, &lord_spend);
        let rest = families::take_urban_rents(st, &urban_pay);
        pay_rent(st, P, &rest);
    } else {
        pay_rent(st, P, &urban_pay);
    }

    // investment deliveries: firms pay for capital goods, capacity arrives pro rata
    if !oi.is_empty() {
        let p = &st.price;
        let inv_pay: Vec<f64> = (0..oi.len()).map(|r| pairwise_sum_by(G, |g| inv_got[(r, g)] * p[(ot[r], g)])).collect();
        let remaining_value: Vec<f64> =
            oi.iter().zip(&ot).map(|(&i, &t)| pairwise_sum_by(G, |g| st.f_order[(i, g)] * p[(t, g)])).collect();
        for (r, &i) in oi.iter().enumerate() {
            let frac = if remaining_value[r] > 1e-12 { inv_pay[r] / remaining_value[r].max(1e-12) } else { 1.0 };
            let addK = st.f_order_K[i] * frac.clamp(0.0, 1.0);
            st.f_K[i] += addK;
            st.f_order_K[i] -= addK;
            for g in 0..G {
                st.f_order[(i, g)] = (st.f_order[(i, g)] - inv_got[(r, g)]).max(0.0);
            }
            let done = pairwise_sum(st.f_order.row(i)) < 1e-6 || st.f_order_K[i] < 1e-9;
            if done {
                st.f_order_K[i] = 0.0;
                st.f_order.row_mut(i).iter_mut().for_each(|v| *v = 0.0);
            }
            st.f_cash[i] -= inv_pay[r];
        }
        let by_town = bincount(&ot, &inv_pay, T);
        add_to(&mut merch, &by_town);
        add_to(&mut sales_value, &by_town);
        st.m_invest = pairwise_sum(&inv_pay);
    } else {
        st.m_invest = 0.0;
    }

    let mut proj_spend = 0.0;
    for (pr, got) in st.projects.iter_mut().zip(&proj_got) {
        let pay = pairwise_sum_by(G, |g| got[g] * st.price[(pr.town, g)]);
        // canals (docs/rules/canals.md): a way says what its navvies are paid on each
        // shilling of materials; the port's and the railways' three tenths otherwise
        let wages = crate::ways::navvy_share(pr) * pay;
        pr.navvy_pay = wages;
        pr.cash -= pay + wages;
        for g in 0..G {
            pr.order[g] = (pr.order[g] - got[g]).max(0.0);
        }
        merch[pr.town] += pay;
        sales_value[pr.town] += pay;
        hh_inc[pr.town] += wages;
        proj_spend += pay + wages;
    }
    st.m_invest += proj_spend;

    // ---------------- inventories: sales out, production in
    // Python adds one bincount per sector good, then one per method output
    let mut prod_TG = Mat::zeros(T, G);
    for u in 0..NE {
        prod_TG[(et[u], g_s[u])] += Y[u];
    }
    let mut prod_firms = Mat::zeros(T, G);
    for i in 0..NFI {
        prod_firms[(ft[i], out[i])] += q[i];
    }
    for (x, y) in prod_TG.data.iter_mut().zip(&prod_firms.data) {
        *x += y;
    }
    for t in 0..T {
        for g in 0..G {
            let v = &mut st.inv[(t, g)];
            *v -= sold[(t, g)];
            *v *= 1.0 - P.decay[g];
            *v += prod_TG[(t, g)];
            *v = v.max(0.0);
        }
    }

    // forests are depleted by harvesting
    for u in 0..NE {
        if sec[u] == st.forestry_sector {
            let c = ex[u];
            st.cover[c] = (st.cover[c] - Y[u] / (P.forest_capacity * lam[u].max(0.05))).max(0.0);
        }
    }

    // ---------------- trade between towns
    let mut exports = Mat::zeros(T, G);
    let mut imports = Mat::zeros(T, G);
    let A_idx: Vec<usize> = (0..T).filter(|&t| st.town_active[t]).collect();
    let na = A_idx.len();
    if na > 1 {
        let p = &st.price;
        // tau, want and qt are (na, na, G), indexed [(i * na + j) * G + g]
        let mut tau = vec![0.0; na * na * G];
        let mut want = vec![0.0; na * na * G];
        for (i, &ti) in A_idx.iter().enumerate() {
            for (j, &tj) in A_idx.iter().enumerate() {
                let c0 = st.town_cost[(ti, tj)];
                let c = if c0.is_finite() { c0 } else { 1e9 };
                for g in 0..G {
                    let k = (i * na + j) * G + g;
                    tau[k] = P.freight * C.weight[g] * c + P.handling * p[(ti, g)];
                    let gap = p[(tj, g)] - p[(ti, g)] - tau[k];
                    let frac = if i == j { 0.0 } else { (gap / p[(tj, g)].max(1e-9)).clamp(0.0, 0.5) * P.trade_k };
                    want[k] = frac * st.inv[(ti, g)];
                }
            }
        }
        let mut scale = vec![0.0; na * G];
        for i in 0..na {
            for g in 0..G {
                let mut tot = 0.0;
                for j in 0..na {
                    tot += want[(i * na + j) * G + g];
                }
                let cap = P.trade_cap * st.inv[(A_idx[i], g)];
                scale[i * G + g] = if tot > cap { cap / tot.max(1e-12) } else { 1.0 };
            }
        }
        let mut qt = want;
        for i in 0..na {
            for j in 0..na {
                for g in 0..G {
                    qt[(i * na + j) * G + g] *= scale[i * G + g];
                }
            }
        }
        let mut ex_q = Mat::zeros(na, G);
        let mut im_q = Mat::zeros(na, G);
        for i in 0..na {
            for j in 0..na {
                for g in 0..G {
                    let v = qt[(i * na + j) * G + g];
                    ex_q[(i, g)] += v;
                    im_q[(j, g)] += v;
                }
            }
        }
        for (i, &ti) in A_idx.iter().enumerate() {
            for g in 0..G {
                st.inv[(ti, g)] += im_q[(i, g)] - ex_q[(i, g)];
                exports[(ti, g)] = ex_q[(i, g)];
                imports[(ti, g)] = im_q[(i, g)];
            }
        }
        let mut pay = Mat::zeros(na, na);
        let mut tcost = Mat::zeros(na, na);
        for (i, &ti) in A_idx.iter().enumerate() {
            for (j, &tj) in A_idx.iter().enumerate() {
                let base = (i * na + j) * G;
                pay[(i, j)] = pairwise_sum_by(G, |g| qt[base + g] * p[(ti, g)]);
                tcost[(i, j)] = pairwise_sum_by(G, |g| qt[base + g] * tau[base + g]);
                let wflow = pairwise_sum_by(G, |g| qt[base + g] * C.weight[g]);
                st.trade_F[(ti, tj)] += wflow;
            }
        }
        for (i, &ti) in A_idx.iter().enumerate() {
            let pay_out = pairwise_sum(pay.row(i));
            let tc_out = pairwise_sum(tcost.row(i));
            let mut pay_in_ = 0.0;
            let mut tc_in = 0.0;
            for j in 0..na {
                pay_in_ += pay[(j, i)];
                tc_in += tcost[(j, i)];
            }
            merch[ti] += pay_out - pay_in_ - tc_in;
            hh_inc[ti] += tc_out;
            // carting: and the trade the town sends out
            crate::carting::add_bill_town(st, P, ti, tc_out);
        }
        // canals (docs/rules/canals.md): what the month's trade put on the water, good by good
        crate::canals::note_trade(st, C, P, &A_idx, &qt);
        st.m_trade_value = pairwise_sum(&pay.data);
        // accounts: what went from each town to each other, and the carting paid on it
        crate::accounts::note_trade(&mut st.acct, &A_idx, &qt, &tau, &st.price);
    } else {
        st.m_trade_value = 0.0;
    }
    for v in st.inv.data.iter_mut() {
        *v = v.max(0.0);
    }

    // accounts: the month's goods and payments go into the books before prices
    // move. Everything handed over is a local of this function; nothing comes back.
    if st.acct.on {
        #[rustfmt::skip]
        crate::accounts::note_month(st, C, P, &crate::accounts::Month {
            et: &et, sec: &sec, Y: &Y, Lex: &Lex, cart_ex: &cart_ex, mach_got: &mach_got, rail_got: &rail_got,
            land_rents: match &royalties {
                Some((towns, cells, parts)) => (&towns[..], &cells[..], &parts[..]),
                None => (&et[..], &ex[..], &rent_ex[..]),
            },
            fi: &fi, ft: &ft, fm: &fm, fc: &fc, out: &out, K: &K, q: &q, labor_f: &labor_f, wages_f: &wages_f,
            rent_f: &rent_f, pay_in: &pay_in, cart_in: &cart_in, cart_out: &cart_out, rev: &rev, cut: hired.as_deref(),
            a: &a, use_f: &use_f, pin: &pin,
            hh_got: &hh_got, lord_got: &lord_got, fam_by_town: &fam_by_town, lord_budget: &lord_budget,
            hh_spend: &hh_spend, lord_spend: &lord_spend, urban_pay: &urban_pay,
            q_hh: &q_hh, q_l: &q_l, share: &share, fill: &fill, lord_pop: &lord_pop,
            oi: &oi, ot: &ot, inv_got: &inv_got, proj_got: &proj_got,
            R: &R, sold: &sold, prod_TG: &prod_TG, exports: &exports, imports: &imports, w: &w, supply: &supply,
        });
    }

    // ---------------- prices respond to inventories (level) and to this month's
    // imbalance between wanted and arriving goods (flow), which damps cycles
    for t in 0..T {
        for g in 0..G {
            let outflow = sold[(t, g)] + exports[(t, g)];
            let of = 0.8 * st.outflow[(t, g)] + 0.2 * outflow;
            st.outflow[(t, g)] = of;
            let target = P.price_target_months * of;
            let iv = st.inv[(t, g)];
            let gap = (target - iv) / target.max(iv).max(1e-9);
            let arriving = prod_TG[(t, g)] + imports[(t, g)];
            let r = R[(t, g)];
            let imbalance = if r + arriving > 1e-9 { (r - arriving) / (r + arriving).max(1e-9) } else { 0.0 };
            if st.town_active[t] {
                let lo_p = P.price_floor * C.base_price[g];
                let hi_p = P.price_ceiling * C.base_price[g];
                if P.prices.enabled {
                    // prices: the rule below moves a slow anchor, and the price
                    // stands above or below it with the merchants' stock (none
                    // on a good that spoils, under `prices.no_spoil_markup`)
                    prices::mark(st, P, t, g, gap, imbalance, lo_p, hi_p, P.decay[g]);
                } else {
                    let p0 = st.price[(t, g)];
                    let newp = p0 * (P.price_speed * gap + P.price_flow * imbalance).clamp(-0.12, 0.12).exp();
                    st.price[(t, g)] = newp.clamp(lo_p, hi_p);
                }
            }
            st.pexp[(t, g)] = (1.0 - P.expect_rate) * st.pexp[(t, g)] + P.expect_rate * st.price[(t, g)];
        }
    }

    // prices: a probe may ask for the month's requests and sales to be kept
    if st.watch_markets {
        st.month_requests = R;
        st.month_sold = sold;
    }

    // ---------------- dividends, common fund, issuance
    add_to(&mut st.merch_cash, &merch);
    let mut fdiv = vec![0.0; T];
    if NFI > 0 {
        let mut div = vec![0.0; NFI];
        for (k, &i) in fi.iter().enumerate() {
            // works: the plant still on order is paid for before the owners are (part H)
            let excess = st.f_cash[i] - P.firm_buffer_months * st.f_cost_m[i] - crate::works::kept_for_order(st, C, P, i);
            div[k] = P.firm_div_rate * excess.max(0.0);
            st.f_cash[i] -= div[k];
        }
        // accounts: every works' dividend, before the families take theirs out of the list
        crate::accounts::note_dividends(&mut st.acct, &div);
        if families::active(st) {
            families::take_dividends(st, C, &fi, &mut div);
        }
        fdiv = bincount(&ft, &div, T);
    }
    let mut mdiv = vec![0.0; T];
    for t in 0..T {
        let mexcess = st.merch_cash[t] - P.merch_buffer_months * sales_value[t];
        mdiv[t] = if st.town_active[t] { P.merch_div_rate * mexcess.max(0.0) } else { 0.0 };
        st.merch_cash[t] -= mdiv[t];
    }
    let pop_total = pairwise_sum(&st.pop).max(1.0);
    // commons: a common fund in debt pays no dividend, and asks nobody for one
    let fund_div = 0.9 * crate::commons::fund_to_share(st, P);
    st.fund_cash -= fund_div;
    // the port counts here, while the month's incomes are in `hh_inc` and the
    // dividends just taken out are in `fdiv` and `mdiv`, and so counts short
    let m_tot = match money_base {
        Some(m) => m,
        None => money_total(st),
    };
    let issue = crate::money::rate(P) / 12.0 * m_tot;
    st.issued += issue;
    for t in 0..T {
        let popshare = st.pop[t] / pop_total;
        hh_inc[t] += fdiv[t] + mdiv[t] + (fund_div + issue) * popshare;
    }
    // accounts: the month's incomes, before the purses are credited
    crate::accounts::close_month(st, &fi, &ft, &fm, &merch, &hh_inc, &fdiv, &mdiv, fund_div, issue, pop_total);

    for t in 0..T {
        st.hh_income[t] += hh_inc[t];
        st.hh_cash[t] += st.hh_income[t];
        st.lord_cash[t] += st.lord_income[t];
    }
    st.hh_income_prev = st.hh_income.clone();
    st.lord_income_prev = st.lord_income.clone();
    st.hh_income.iter_mut().for_each(|v| *v = 0.0);
    st.lord_income.iter_mut().for_each(|v| *v = 0.0);
    families::end_month(st);

    // ---------------- bookkeeping for the year and for observers
    let S = C.S;
    let mut serv_deliv = Mat::zeros(T, S);
    for t in 0..T {
        let popd = st.pop[t].max(1.0);
        for s in 0..S {
            let mut acc = 0.0;
            for g in 0..G {
                acc += hh_got[(t, g)] * C.props[(g, s)];
            }
            serv_deliv[(t, s)] = acc / popd;
        }
    }
    // service 4 is implements, service 0 is food (as hard-coded in Python)
    st.food_ratio = (0..T).map(|t| serv_deliv[(t, 0)] / C.subsistence[0]).collect();
    for t in 0..T {
        st.imp_pc[t] = 0.9 * st.imp_pc[t] + 0.1 * serv_deliv[(t, 4)];
        st.tools_mult[t] = 1.0 + P.tools_boost * (1.0 - (-st.imp_pc[t] / P.tools_scale).exp());
        st.y_acc[t] += if subcost[t] > 0.0 { pc[t] / subcost[t].max(1e-12) } else { 0.0 };
        let fr = if st.town_active[t] { st.food_ratio[t] } else { 9.0 };
        st.food_min_acc[t] = st.food_min_acc[t].min(fr);
        st.wage_acc[t] += w[t];
        for g in 0..G {
            st.price_acc[(t, g)] += st.price[(t, g)];
            st.town_prod[(t, g)] += prod_TG[(t, g)];
        }
    }
    st.fill = fill;
    st.wage = w;
    st.subcost = subcost;
    let mut prod_g = vec![0.0; G];
    for t in 0..T {
        for g in 0..G {
            prod_g[g] += prod_TG[(t, g)];
        }
    }
    for g in 0..G {
        st.cumprod[g] += prod_g[g];
    }
    let mo = bincount(&fm, &q, C.M);
    for m in 0..C.M {
        st.method_Q[m] += mo[m];
    }
    st.ex_labor = Lex.clone();
    crate::mines::add_crews(st, P); // mines: the sinkers are hands at their workings
    st.ex_out = Y.clone();
    st.f_labor.iter_mut().for_each(|v| *v = 0.0);
    for (k, &i) in fi.iter().enumerate() {
        st.f_labor[i] = labor_f[k];
    }
    crate::towns::month(st, P); // towns: works' hands this month, for the year's mean
    let mut flow = vec![0.0; st.N];
    for u in 0..NE {
        flow[ex[u]] += Y[u] * C.weight[g_s[u]];
    }
    for i in 0..NFI {
        flow[fc[i]] += pairwise_sum_by(G, |g| use_f[(i, g)] * C.weight[g]) + q[i] * C.weight[out[i]];
    }
    for c in 0..st.N {
        st.cell_flow[c] += flow[c];
    }

    let mut employment = bincount(&et, &st.ex_labor, T);
    add_to(&mut employment, &bincount(&ft, &labor_f, T));
    // carting: carters are at work too
    crate::carting::count_carters(st, P, &mut employment);
    // canals: and so are the navvies
    crate::canals::count_navvies(st, P, &mut employment);
    // roads: and so are the menders
    crate::roads::count_menders(st, P, &mut employment);
    let mut heat_use = vec![0.0; NF];
    for i in 0..NFI {
        for k in 0..NF {
            heat_use[k] += q[i] * h[i] * hshare[(i, k)];
        }
    }
    st.month_stats = MonthStats {
        prod: prod_g,
        method_out: mo,
        hh_got,
        lord_got,
        share,
        hh_spend: pairwise_sum(&hh_spend),
        lord_spend: pairwise_sum(&lord_spend),
        urban_rent: pairwise_sum(&urban_pay),
        employment,
        ex_labor_by_sector: bincount(&sec, &st.ex_labor, NS),
        firm_labor_by_method: bincount(&fm, &labor_f, C.M),
        heat_use,
        supply,
    };
    // machines (docs/rules/machines.md): the machines standing move towards what was bought
    crate::machines::end_month(st, C, P);
}

/// For each good, `np.bincount(town, weights=rows[:, g], minlength=T)`, in one pass
/// (each (town, good) total still sums the rows in order).
fn sum_by_town(T: usize, town: &[usize], rows: &Mat) -> Mat {
    let G = rows.cols;
    let mut acc = Mat::zeros(T, G);
    for (r, &t) in town.iter().enumerate() {
        for g in 0..G {
            acc[(t, g)] += rows[(r, g)];
        }
    }
    acc
}

fn add_to(a: &mut [f64], b: &[f64]) {
    for (x, y) in a.iter_mut().zip(b) {
        *x += y;
    }
}

fn add_column(m: &mut Mat, g: usize, v: &[f64]) {
    for (t, x) in v.iter().enumerate() {
        m[(t, g)] += x;
    }
}

/// `for g: R[:, g] += np.bincount(town, weights=rows[:, g], minlength=T)`.
fn add_by_town(R: &mut Mat, town: &[usize], rows: &Mat) {
    let acc = sum_by_town(R.rows, town, rows);
    for (x, y) in R.data.iter_mut().zip(&acc.data) {
        *x += y;
    }
}
