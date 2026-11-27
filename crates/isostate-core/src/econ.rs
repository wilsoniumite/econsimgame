//! The monthly economy: wages, production, consumption, markets, trade, money.
//! Python: `econ.py`.
//!
//! Every payment is a transfer between two accounts (households, landlords,
//! merchants, firms, the land fund, rail projects), so total money only changes
//! through explicit issuance. `money_total` checks that invariant.
//!
//! The month (`step_month`, which is `month_ledger`) goes in these phases, in
//! this order:
//!
//! 1. The books open; the month's issue of money and the freight are reckoned.
//! 2. What households and landlords want to buy: each a line of the month's
//!    ledger of buyers (ledger.rs).
//! 3. The land's units and the works reckon what a unit of their work earns at
//!    last month's prices (the land's machines and machines for tasks among it).
//! 4. The labour market: the hands for hire, less those the carters, sinkers,
//!    navvies, menders, teachers and the university take; one wage a town that
//!    clears it; a town short of hands rationed. The land's output follows.
//! 5. Every other buyer's asks, a line each: the works' inputs and fuel; the
//!    land's machinery and rails, its chemicals and its machines for tasks; the
//!    lorries; (the harvest felt); the works' orders for capacity and for their
//!    machines' plant; railways and canals; the university's rooms.
//! 6. The market: all that was asked of each town, the share its stock can give
//!    (the day's trade first under "plant waits"), and each line's got.
//! 7. The works make what their scarcest input allows; what they used is their
//!    line's got, and all that every line got is what the market sold.
//! 8. Money: the works, the land, the households and landlords, the lorries and
//!    the rents pay; plant, capacity, the ways' materials and the rooms are
//!    handed over and paid for.
//! 9. The stocks: sales out, decay, the month's making in; the forests felled;
//!    the trade between towns.
//! 10. The books take the month; prices respond.
//! 11. Dividends, the common fund, the new money, the state's purse and the
//!     hauliers' till; the purses credited.
//! 12. The year's sums and the observers' figures; the machines standing and
//!     the machines for tasks moved at the month's end.
//!
//! Before the ledger (27 November 2026) `step_month` was 1,403 lines, and each
//! kind of buyer was written out by hand at each of steps 6 to 10; with it
//! `month_ledger` is 1,272 lines, and what the buyers share is in ledger.rs.

use crate::content::Content;
use crate::engel;
use crate::families;
use crate::harvest;
use crate::ledger::{Buyer, Fold, Ledger, Line, Trade};
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
    service_mix_widths(price, props, &vec![theta; props.cols])
}

/// `service_mix` with a width of the logit for each need (tree: household goods
/// are less alike than fuels are). With one width for all it is `service_mix`.
pub fn service_mix_widths(price: &Mat, props: &Mat, widths: &[f64]) -> (Mat, Mat) {
    let (T, G, S) = (price.rows, price.cols, props.cols);
    let mut share = Mat::zeros(T, G * S);
    let mut unit = Mat::zeros(T, S);
    let mut sv = vec![0.0; G];
    let mut z = vec![0.0; G];
    for t in 0..T {
        for s in 0..S {
            let theta = widths[s];
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
    // care (docs/rules/care.md; not in the Python): a nurse's hours a patient are
    // the recipe's in a ward of four beds or of four hundred, and in a village or
    // the first city, so its hands a unit do not fall with its works' size or its
    // productivity (None, and every works as the port has it, without care)
    let care = crate::care::method(C);
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
        l[i] = if care == Some(m) { C.m_labor[m] } else { C.m_labor[m] * (1.0 + 0.8 * r) / prod[i] };
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
    // state (docs/rules/state.md): the treasury, nothing while the purse is off
    let total = if st.purse.treasury != 0.0 { total + st.purse.treasury } else { total };
    // the lorry (docs/rules/lorry.md): the hauliers' tills, none without `lorry.till`
    let total = if st.tasks.cart_till.is_empty() { total } else { total + pairwise_sum(&st.tasks.cart_till) };
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
///   for each town's wage is half to twice it (kept within 0.02 and 500, which `longrun` moves with its anchor), and
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
    month_ledger(st, C, P);
}

/// `step_month`, giving back the month's ledger of buyers (ledger.rs): what
/// each line asked and got, and what each market was asked and sold. For a
/// test that the ledger agrees with the books (`tests/ledger.rs`).
pub fn month_ledger(st: &mut State, C: &Content, P: &Params) -> Ledger {
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
    // longrun (docs/rules/longrun.md): the freight, and below the bounds on the
    // wage and on prices, ride on the year's anchor (the constants while it is off)
    let freight = crate::longrun::freight(st, P);

    // ---------------- households: what they want to buy
    // tree: a width for each need where the content gives one, else the port's one
    let (share, unit) = service_mix_widths(&st.price, &C.props, &crate::tree::widths(C, P));
    let mut urban_pay = vec![0.0; T];
    let mut pc = vec![0.0; T];
    let budgets: Vec<f64> = (0..T)
        .map(|t| {
            let hh_cash = st.hh_cash[t].max(0.0);
            hh_cash.min(P.cons_income * st.hh_income_prev[t].max(0.0) + P.cons_wealth * hh_cash)
        })
        .collect();
    // ground (docs/rules/ground.md): from 1901, house rent that clears the room there is
    let cleared = crate::ground::month(st, C, P, &budgets, &unit);
    for t in 0..T {
        let budget = budgets[t];
        urban_pay[t] = match &cleared {
            Some(pay) => pay[t],
            None => st.urban_rent[t].min(0.5 * budget),
        };
        let goods_budget = budget - urban_pay[t];
        pc[t] = if st.pop[t] > 0.0 { goods_budget / st.pop[t].max(1.0) } else { 0.0 };
    }
    // engel: past a reference income the households' marginal shares lean toward
    // a richer set (docs/rules/engel.md); landlords, below, keep their own. After
    // 1900 the richer set itself may lean toward comfort (engel's lean)
    let (q_hh, subcost) = if P.engel.enabled {
        let rich = engel::rich_now(C, &P.engel, st.engel.year_now);
        // care (docs/rules/care.md): each town's richer set gives care room by
        // its want, once the town knows of care (None, and the set as it is, before)
        match crate::care::town_rich(st, C, P, &rich) {
            Some(rows) => engel::les_rows(&pc, &unit, &C.subsistence, &C.shares, &rows, &P.engel),
            None => engel::les(&pc, &unit, &C.subsistence, &C.shares, &rich, &P.engel),
        }
    } else {
        les(&pc, &unit, &C.subsistence, &C.shares)
    };
    let mut d_hh = goods_from_services(&q_hh, &share, &C.props);
    for t in 0..T {
        for v in d_hh.row_mut(t) {
            *v *= st.pop[t];
        }
    }
    // the ledger (ledger.rs): every buyer's asks of the month, a line each
    let mut ledger = Ledger::new(T, G);
    let towns: Vec<usize> = (0..T).collect();
    ledger.ask((Buyer::Households, Trade::Day), Line::new(towns.clone(), towns.clone(), Fold::EachRow, d_hh));

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
    ledger.ask((Buyer::Landlords, Trade::Day), Line::new(towns.clone(), towns.clone(), Fold::EachRow, d_lord));

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
    // machines (docs/rules/machines.md, "Nitrogen and breeding"): what the land
    // buys a unit beside machinery and rails, by town and sector; empty without
    // the content after 1900
    let mut more_unit: Vec<Vec<(usize, f64)>> = vec![Vec::new(); T * NS];
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
            if P.machines.enabled {
                more_unit[t * NS + s] = crate::machines::bought_more(st, C, P, s, t, &pe);
            }
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
    // tasks (docs/rules/tasks.md, "On the land" and "The ceiling on yield"):
    // where machines hold part of a land sector's work, its units pay their
    // share for work to a mixture of hands and machine units, at the cost of
    // an hour's work `cL` in place of the wage; and while the ceiling is in
    // force, a unit that stood at it last month hires only the work it needs.
    // None, and the month as it was, while neither acts
    let lt = crate::tasks::land_month(st, C, P, &pe, &ex, &sec, &et, &A, &f, &lam, &alpha);
    // mines (docs/rules/mines.md, "G. At the size of the market"): under
    // `mines.glut` a working raises only the share s of its make at full time
    // that the merchants will take, and wants the hands for that: s^(1/(1 - alpha))
    // of them, which is its Cc times s^(alpha/(1 - alpha))
    if P.mines.glut {
        let make: Vec<f64> = (0..NE)
            .map(|u| {
                let (t, w) = (et[u], st.wage[et[u]]);
                if crate::mines::row(st, sec[u]).is_none() || !(Cc[u] > 0.0 && lam[u] > 0.0 && w > 0.0) {
                    return 0.0;
                }
                let work = match &lt {
                    Some(l) if l.capped[u] => l.cap_work[u],
                    Some(l) => lam[u] * (Cc[u] / l.mix_of(t, sec[u], NS).map_or(w, |m| m.cost_of_work(w))).powf(1.0 / alpha[u]),
                    None => lam[u] * (Cc[u] / w).powf(1.0 / alpha[u]),
                };
                A[u] * f[u] * lam[u] * (work / lam[u]).powf(1.0 - alpha[u])
            })
            .collect();
        if let Some(share) = crate::mines::merchants_take(st, C, P, &et, &sec, &make) {
            for u in 0..NE {
                if share[u] < 1.0 {
                    Cc[u] *= share[u].powf(alpha[u] / (1.0 - alpha[u]));
                }
            }
        }
    }

    // ---------------- firms: unit economics at their own location
    let fi: Vec<usize> = (0..st.F_count).filter(|&i| st.f_alive[i] && st.f_K[i] > 1e-9).collect();
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
    // tasks (docs/rules/tasks.md): where a works' machines hold part of a class,
    // fewer hands a unit, the machines' run goods among its inputs, and its
    // order for their plant (nothing while no works holds any); and what its
    // machines ask of the month, for the soft run below (empty while none do)
    let running = crate::tasks::works_month(st, C, P, &fi, &K, &mut a, &mut l);
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
        (0..NFI).map(|i| p[(ft[i], out[i])] - freight * C.weight[out[i]] * st.dcatch[fc[i]]).collect();
    let nonlabor: Vec<f64> =
        (0..NFI).map(|i| C.sum_goods(|g| a[(i, g)] * pin[(i, g)]) + h[i] * hunit[i]).collect();
    let phi_prev: Vec<f64> = fi.iter().map(|&i| st.f_fill[i]).collect();

    // ---------------- labour market: one wage per town clears supply and demand
    let mut supply: Vec<f64> = st.pop.iter().map(|&p| P.participation * p).collect();
    // who works (docs/rules/who-works.md): from 1901 the people in work, counted by age
    crate::who_works::supply(st, P, &mut supply);
    // carting (docs/rules/carting.md): the hands that carted last month's goods are not for hire
    crate::carting::take_carters(st, C, P, &mut supply);
    // mines (docs/rules/mines.md): nor are the hands sinking new workings
    crate::mines::take_crews(st, P, &mut supply);
    // canals (docs/rules/canals.md): nor are the navvies digging a canal. They come
    // last of the three: each caps itself on the hands still left, the carters and
    // the sinkers are at work the town already lives by, and a canal can wait
    crate::canals::take_navvies(st, P, &mut supply);
    // roads (docs/probes/dx-roads.md): nor are the men making and mending roads
    crate::roads::take_menders(st, P, &mut supply);
    // state (docs/rules/state.md): nor are the teachers the treasury hires
    crate::purse::take_teachers(st, P, &mut supply);
    // nor are the university's students and staff (docs/rules/state.md, "The university")
    crate::university::take_hands(st, P, &mut supply);
    // people (docs/rules/people.md, "The hands that come with a district"): in the
    // first month after the year's turn a town that took a district has at least
    // the hands its employers carry in (nothing while `people.carry_hands` is off)
    crate::peopling::carry_hands(st, P, &mut supply);
    let supply = supply;
    // Short-run labour demand moves only part of the way to the long-run optimum,
    // so a price blip does not empty the fields in a month.
    let prev_w = st.wage.clone();
    let mut lo = vec![0.0; T];
    let mut hi = vec![0.0; T];
    let (w_lo, w_hi) = crate::longrun::wage_bounds(st, P);
    for t in 0..T {
        let has = prev_w[t] > 0.0;
        let safe_w = if has { prev_w[t] } else { 1.0 };
        lo[t] = if has { (safe_w / 2.0).max(w_lo).ln() } else { w_lo.ln() };
        hi[t] = if has { (safe_w * 2.0).min(w_hi).ln() } else { w_hi.ln() };
    }
    let inv_alpha: Vec<f64> = alpha.iter().map(|&x| 1.0 / x).collect();
    let (kx, kf) = (P.labor_adjust_ex, P.labor_adjust_firm);
    let Lprev_ex = st.ex_Lprev.clone();
    let Lprev_f: Vec<f64> = fi.iter().map(|&i| st.f_labor[i]).collect();

    // prices: the share of each works' capacity the merchants will take this month,
    // less than 1 only where their stock of its good is long (all ones while the
    // mechanism is off). It is fixed before the wage is sought and does not depend
    // on it, so demand still falls as the wage rises.
    let take = prices::merchants_take(st, P, &ft, &out, &K);

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
    // tasks: the hands the units at the ceiling want, which no wage moves, by town
    let mut cap_l = vec![0.0; T];
    for u in 0..NE {
        x_pow[u] = lam[u] * Cc[u].powf(inv_alpha[u]);
        match &lt {
            Some(l) if l.capped[u] => cap_l[et[u]] += l.hands[u] * l.cap_work[u],
            _ => sum_x[et[u] * NS + sec[u]] += x_pow[u],
        }
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
    // tasks: the hands unit u wants at wage w where machines or the ceiling act,
    // and the derivative of that in log w (less its sign). Work wanted is
    // `lam * (Cc / cL)^(1/alpha)` at `cL = hands * w + cost`, and hands are
    // that times `hands`; a unit at the ceiling wants what the ceiling needs
    let land_want = |l: &crate::tasks::LandMonth, u: usize, w: f64| -> (f64, f64) {
        if l.capped[u] {
            return (l.hands[u] * l.cap_work[u], 0.0);
        }
        match l.mix_of(et[u], sec[u], NS) {
            Some(m) => {
                let cl = m.cost_of_work(w);
                let v = m.hands * lam[u] * (Cc[u] / cl).powf(inv_alpha[u]);
                (v, inv_alpha[u] * v * m.hands * w / cl)
            }
            None => {
                let v = lam[u] * (Cc[u] / w).powf(inv_alpha[u]);
                (v, inv_alpha[u] * v)
            }
        }
    };
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
                    // tasks: at the cost of an hour's work, and hands an hour of it
                    if let Some(m) = lt.as_ref().and_then(|l| l.mix_of(t, s, NS)) {
                        let cl = m.cost_of_work(wt);
                        let wp = cl.powf(-sector_inv_alpha[s]);
                        s_tw += sx * m.hands * wp;
                        dd -= kx * sector_inv_alpha[s] * sx * m.hands * wp * m.hands * wt / cl;
                    } else {
                        let wp = wt.powf(-sector_inv_alpha[s]);
                        s_tw += sx * wp;
                        dd -= kx * sector_inv_alpha[s] * sx * wp;
                    }
                }
            }
            if lt.is_some() {
                s_tw += cap_l[t];
            }
            d = sum_l[t] + kx * (s_tw - sum_l[t]);
        } else {
            for &u in &town_units[t] {
                if let Some(l) = &lt {
                    let (want, dw) = land_want(l, u, wt);
                    let Lx = Lprev_ex[u] + kx * (want - Lprev_ex[u]);
                    d += Lx.max(0.0);
                    if Lx > 0.0 {
                        dd -= kx * dw;
                    }
                    continue;
                }
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
    // tasks: where machines or the ceiling act, the hands at the cost of an hour's work
    if let Some(l) = &lt {
        for u in 0..NE {
            let want = if l.capped[u] {
                l.hands[u] * l.cap_work[u]
            } else if let Some(m) = l.mix_of(et[u], sec[u], NS) {
                x_pow[u] * m.hands * m.cost_of_work(wq[et[u]]).powf(-inv_alpha[u])
            } else {
                continue;
            };
            Lex[u] = (Lprev_ex[u] + kx * (want - Lprev_ex[u])).max(0.0);
        }
    }
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
    // (tasks: of the hands, the work is the hands over hands an hour of work)
    let mut Y: Vec<f64> = (0..NE)
        .map(|u| {
            let L = match &lt {
                Some(l) => Lex[u] / l.hands[u],
                None => Lex[u],
            };
            A[u] * f[u] * (lam[u] * (L / lam[u]).powf(1.0 - alpha[u]))
        })
        .collect();
    // tasks: no unit raises more than its ceiling; it stands at it next month if
    // the work it wants at this month's wage would take it there
    let at_ceiling = lt.as_ref().filter(|l| l.ceiling.iter().any(|c| c.is_finite())).map(|l| {
        let (mut flag, mut capped, mut all) = (vec![false; NE], 0.0, 0.0);
        for u in 0..NE {
            let c = l.ceiling[u];
            let p = st.price[(et[u], g_s[u])];
            if c.is_finite() {
                let w = wq[et[u]];
                let cl = l.mix_of(et[u], sec[u], NS).map_or(w, |m| m.cost_of_work(w));
                let want = x_pow[u] * cl.powf(-inv_alpha[u]);
                flag[u] = A[u] * f[u] * (lam[u] * (want / lam[u]).powf(1.0 - alpha[u])) >= c;
                if Y[u] >= c {
                    Y[u] = c;
                    capped += p * c;
                }
            }
            all += p * Y[u];
        }
        (flag, capped, all)
    });
    crate::tasks::note_yields(st, C, P, &ex, &sec, &Y, &lam);
    let q_des: Vec<f64> = (0..NFI)
        .map(|i| {
            let q = if l[i] > 0.0 { Lf[i] / (l[i] * phi_prev[i].max(0.05)).max(1e-12) } else { 0.0 };
            q.min(K[i])
        })
        .collect();

    // ---------------- the asks of every buyer, line by line (ledger.rs)
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
    // the works use what they get as their scarcest input allows (below, "execute")
    ledger.ask((Buyer::Works, Trade::Day), Line::new(ft.clone(), fi.clone(), Fold::ByTown, req_f).by_rule());
    // the land's machinery and rails, a column each, unit by unit
    let mut mine = Mat::zeros(NE, 2);
    for u in 0..NE {
        mine[(u, 0)] = Y[u] * mach_unit[u];
        mine[(u, 1)] = Y[u] * rail_unit[u];
    }
    ledger.ask((Buyer::LandMachinery, Trade::Day), Line::of_goods(vec![gm, gr], et.clone(), (0..NE).collect(), Fold::ByTown, mine));
    // machines: the land's other goods (the chemicals of nitrogen and breeding),
    // asked for with the machinery, by town and sector; and each unit's output
    // they were asked on, for what it pays
    let mut y_asked = None;
    if more_unit.iter().any(|m| !m.is_empty()) {
        let mut q = Mat::zeros(T * NS, G);
        for u in 0..NE {
            let r = et[u] * NS + sec[u];
            for &(g, x) in &more_unit[r] {
                q[(r, g)] += Y[u] * x;
            }
        }
        ledger.ask((Buyer::LandChemicals, Trade::Day), Line::new((0..T * NS).map(|r| r / NS).collect(), (0..T * NS).collect(), Fold::EachRow, q));
        y_asked = Some(Y.clone());
    }
    // tasks: the land's machines ask for their run goods with the day's trade,
    // and for their plant beside the works' orders, by town and sector
    let mut lbuy = lt.as_ref().and_then(|l| crate::tasks::land_orders(st, C, l, &et, &sec, &Lex));
    if let Some(b) = &mut lbuy {
        let rows = || ((0..T * NS).map(|r| r / NS).collect::<Vec<_>>(), (0..T * NS).collect::<Vec<_>>());
        let (town, who) = rows();
        ledger.ask((Buyer::LandTasks, Trade::Day), Line::new(town, who, Fold::EachRow, std::mem::take(&mut b.run)));
        let (town, who) = rows();
        ledger.ask((Buyer::LandTasks, Trade::Plant), Line::new(town, who, Fold::EachRow, std::mem::take(&mut b.plant)));
    }
    // the lorry (docs/rules/lorry.md): the carters' lorries ask for their oil with
    // the day's trade, and for their plant beside the works' orders, by town
    let mut cbuy = crate::tasks::cart_orders(st, C, P, &budgets);
    if let Some(b) = &mut cbuy {
        ledger.ask((Buyer::Lorries, Trade::Day), Line::new(towns.clone(), towns.clone(), Fold::EachRow, std::mem::take(&mut b.run)));
        ledger.ask((Buyer::Lorries, Trade::Plant), Line::new(towns.clone(), towns.clone(), Fold::EachRow, std::mem::take(&mut b.plant)));
    }
    // harvest: the year's weather is felt here, after the hands are hired and the
    // machines asked for on an average year's yield. From here on Y is the crop
    // actually brought in.
    let mut Y = harvest::brought_in(st, C, P, Y);
    let oi: Vec<usize> = (0..st.F_count).filter(|&i| st.f_alive[i] && st.f_order_K[i] > 0.0).collect();
    let ot: Vec<usize> = oi.iter().map(|&i| st.f_town[i]).collect();
    let mut req_inv = Mat::zeros(oi.len(), G);
    // pace (docs/rules/pace.md): a quarter of what is to come, or a builder's year's part of it
    let ask = crate::pace::share(st, P);
    for (r, &i) in oi.iter().enumerate() {
        for g in 0..G {
            req_inv[(r, g)] = st.f_order[(i, g)] * ask;
        }
    }
    ledger.ask((Buyer::WorksOrders, Trade::Plant), Line::new(ot.clone(), oi.clone(), Fold::ByTown, req_inv));
    // tasks: the works' orders for their machines' plant, a quarter of each a
    // month, beside the orders for capacity (and like them, waiting under the tree)
    if let Some(line) = crate::tasks::orders(st, C, P) {
        ledger.ask((Buyer::WorksTasks, Trade::Plant), line);
    }
    let mut proj_req = Mat::zeros(st.projects.len(), G);
    for (k, pr) in st.projects.iter().enumerate() {
        // railways (docs/rules/railways.md): the game's lines say how long they take,
        // and order no more than the cash in hand will pay for
        let months = if pr.months > 0.0 { pr.months } else { 10.0 };
        let rq = proj_req.row_mut(k);
        for g in 0..G {
            rq[g] = pr.order[g].min(pr.need[g] / months);
        }
        crate::ways::within_means(pr, rq, st.price.row(pr.town));
    }
    let proj_town: Vec<usize> = st.projects.iter().map(|pr| pr.town).collect();
    ledger.ask((Buyer::Ways, Trade::Plant), Line::new(proj_town, (0..st.projects.len()).collect(), Fold::EachRow, proj_req));
    // state (docs/rules/state.md, "The university"): the university's rooms,
    // ordered of its town's builders as a railway orders its materials
    if let Some(line) = crate::university::rooms_order(st, C, P) {
        ledger.ask((Buyer::University, Trade::Plant), line);
    }

    // ---------------- what each market was asked, and the share it could give
    // tree (docs/rules/tree.md, "Plant waits"): what the day's trade asked
    // (households, the works at work, the land), apart from the orders for
    // plant and for railways and canals. Only with the tree laid: without it
    // nothing that plant is built of is a works' daily input, and the rule
    // measured as no gain
    let R_now = ledger.add_asks(P.tree.plant_waits && C.stage > 0);
    // people (docs/rules/people.md) keeps it too: what a district's buyers asked for
    // goes with them when the district changes market
    // and the mines read it under `mines.glut` (docs/rules/mines.md, "G")
    if families::active(st) || P.people.enabled || P.mines.glut {
        // what each town's buyers asked for, for the families' site sums
        st.requests = ledger.asked.clone();
    }
    let mut fill = Mat::zeros(T, G);
    for t in 0..T {
        for g in 0..G {
            let r = ledger.asked[(t, g)];
            fill[(t, g)] = if r > 1e-12 { (st.inv[(t, g)] / r.max(1e-12)).min(1.0) } else { 1.0 };
        }
    }
    // tree: plant waits. Where the market is short, the day's trade is served
    // first (households, the works' own inputs and fuel, the land's machines)
    // and the orders for plant and for the materials of railways and canals
    // share what is left. A works half built can wait a season for its bricks;
    // a bricklayer with a house half up cannot. `fill` is then the day's
    // trade's and `fill_plant` the orders'; without the rule both are the
    // port's one number.
    let fill_plant = match &R_now {
        Some(now) => crate::tree::plant_waits(&st.inv, now, &ledger.asked, &mut fill),
        None => fill.clone(),
    };
    // every line's got, at its trade's fill
    ledger.fill(fill, fill_plant);
    let fill = &ledger.fill;
    // tasks: where the land's machines' run goods were short, the machines stood
    // idle for that part of their work, and the crop is the smaller for it
    // (docs/rules/tasks.md, "A machine short of its run good"): the hands do the
    // idle machines' part at the labour it took before them, so the work done is
    // all that was hired where the fill is no worse than the one the hands were
    // hired for, and what those hands can do where it is worse; and the fill is
    // kept for next month's hiring
    if let (Some(l), Some(_)) = (&lt, &lbuy) {
        let mut run_fill: Vec<(usize, usize, f64)> = Vec::new();
        for t in 0..T {
            for s in 0..NS {
                if let Some(m) = l.mix_of(t, s, NS) {
                    run_fill.push((t, s, m.run.iter().map(|&(g, _)| fill[(t, g)]).fold(1.0, f64::min)));
                }
            }
        }
        for u in 0..NE {
            if let Some(m) = l.mix_of(et[u], sec[u], NS) {
                let fr = m.run.iter().map(|&(g, _)| fill[(et[u], g)]).fold(1.0, f64::min);
                let done = m.done(fr);
                if done < 1.0 {
                    Y[u] *= done.powf(1.0 - alpha[u]);
                }
            }
        }
        for row in st.tasks.land_run_fill.iter_mut() {
            row.iter_mut().for_each(|v| *v = 1.0);
        }
        for (t, s, fr) in run_fill {
            st.tasks.land_run_fill[t][s] = fr;
        }
    }

    // ---------------- execute: firms produce what their scarcest input allows
    let mut q = vec![0.0; NFI];
    let mut use_f = Mat::zeros(NFI, G);
    let mut labor_f = vec![0.0; NFI];
    let own_run = P.tasks.own_run;
    for i in 0..NFI {
        let t = ft[i];
        let mut fill_in = f64::INFINITY;
        // tasks (docs/rules/tasks.md, "A machine short of its run good"): the
        // recipe is held to its own inputs; where the machines' run goods are
        // short, the machines stand idle in that measure, and nothing else does
        let run = running.get(i).filter(|r| !r.run.is_empty());
        let mut fr = 1.0f64;
        match run {
            None => {
                for g in 0..G {
                    fill_in = fill_in.min(if a[(i, g)] > 0.0 { fill[(t, g)] } else { 1.0 });
                }
            }
            Some(r) => {
                for g in 0..G {
                    fill_in = fill_in.min(if a[(i, g)] - r.of(g) > 0.0 { fill[(t, g)] } else { 1.0 });
                }
                // tasks.own_run (docs/rules/tending.md, "The slumps"): a works whose
                // machines run on the good it makes runs them on its own making, so
                // the market's shortage of that good does not stand them idle
                fr = r.run.iter().filter(|&&(g, _)| !(own_run && g == out[i])).map(|&(g, _)| fill[(t, g)]).fold(1.0, f64::min);
            }
        }
        let fill_heat = if h[i] > 0.0 { pairwise_sum_by(NF, |k| hshare[(i, k)] * fill[(t, fuels[k])]) } else { 1.0 };
        let phi = fill_in.min(fill_heat) * run.map_or(1.0, |r| r.output(fr));
        q[i] = q_des[i] * phi;
        for g in 0..G {
            use_f[(i, g)] = q[i] * a[(i, g)];
        }
        // the run goods are charged for what the machines used
        if let Some(r) = run {
            for &(g, v) in &r.run {
                // tasks.own_run: of its own good it buys no more than the market's
                // fill; the rest its machines take from what it makes
                let f = if own_run && g == out[i] { fill[(t, g)].min(fr) } else { fr };
                use_f[(i, g)] -= q[i] * v * (1.0 - f);
            }
        }
        if !running.is_empty() {
            st.tasks.f_run_fill[fi[i]] = fr;
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
    // the works' got is what they used; then every line's got into what each market sold
    ledger.set_got((Buyer::Works, Trade::Day), use_f);
    ledger.sell();
    let fill = &ledger.fill;
    let use_f = ledger.got((Buyer::Works, Trade::Day)).unwrap();
    let hh_got = ledger.got((Buyer::Households, Trade::Day)).unwrap();
    let mach_got = ledger.got((Buyer::LandMachinery, Trade::Day)).unwrap();
    // the two buyers of two lines each, whose gots are added into one
    const LAND_TASKS: [(Buyer, Trade); 2] = [(Buyer::LandTasks, Trade::Day), (Buyer::LandTasks, Trade::Plant)];
    const LORRIES: [(Buyer, Trade); 2] = [(Buyer::Lorries, Trade::Day), (Buyer::Lorries, Trade::Plant)];

    // ---------------- money flows
    let mut merch = vec![0.0; T];
    let mut hh_inc = vec![0.0; T];
    let mut sales_value = vec![0.0; T];

    // firms
    let p = &st.price;
    let pay_in: Vec<f64> = (0..NFI).map(|i| C.sum_goods(|g| use_f[(i, g)] * pin[(i, g)])).collect();
    let cart_in: Vec<f64> =
        (0..NFI).map(|i| C.sum_goods(|g| use_f[(i, g)] * (pin[(i, g)] - p[(ft[i], g)]).max(0.0))).collect();
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
    let mut in_paid: Vec<f64> = (0..NE).map(|u| mach_got[(u, 0)] * p[(et[u], gm)] + mach_got[(u, 1)] * p[(et[u], gr)]).collect();
    // machines: each unit pays for the other goods it asked for, as it got them
    if let Some(y_asked) = &y_asked {
        for u in 0..NE {
            let t = et[u];
            for &(g, x) in &more_unit[t * NS + sec[u]] {
                in_paid[u] += y_asked[u] * x * fill[(t, g)] * p[(t, g)];
            }
        }
    }
    // tasks: each unit pays for its sector's machines' goods by its part of the
    // work; the return on their plant stays in the rent
    if let Some(b) = &lbuy {
        let paid = ledger.paid(C, p, &LAND_TASKS);
        for u in 0..NE {
            let r = et[u] * NS + sec[u];
            if b.work[r] > 0.0 && paid[r] != 0.0 {
                in_paid[u] += paid[r] * (b.unit_work[u] / b.work[r]);
            }
        }
    }
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
        engel::note_month(st, C, &pc, &subcost, &q_hh, &share, fill);
    }

    // households and landlords buy goods; households pay urban rent
    let p = &st.price;
    let hh_spend = ledger.paid(C, p, &[(Buyer::Households, Trade::Day)]);
    let lord_spend = ledger.paid(C, p, &[(Buyer::Landlords, Trade::Day)]);
    for t in 0..T {
        st.hh_cash[t] -= hh_spend[t] + urban_pay[t];
        st.lord_cash[t] -= lord_spend[t];
        merch[t] += hh_spend[t] + lord_spend[t];
        sales_value[t] += hh_spend[t] + lord_spend[t];
    }
    // the lorry (docs/rules/lorry.md): the hauliers own their lorries and pay for
    // their oil and plant: under the hauliers' till out of the till their carting
    // is paid into (`lorry.till`), and without it out of the households' purse
    let cart_pay = ledger.paid(C, p, &LORRIES);
    for (t, &v) in cart_pay.iter().enumerate() {
        merch[t] += v;
        sales_value[t] += v;
    }
    if let (Some(b), Some((_, got))) = (&cbuy, ledger.together(&LORRIES)) {
        crate::tasks::cart_pay(st, P, b, &cart_pay);
        crate::tasks::note_cart_paid(st, &got, ledger.got(LORRIES[1]).unwrap());
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
        let inv_pay = ledger.paid(C, p, &[(Buyer::WorksOrders, Trade::Plant)]);
        let inv_got = ledger.got((Buyer::WorksOrders, Trade::Plant)).unwrap();
        let remaining_value: Vec<f64> =
            oi.iter().zip(&ot).map(|(&i, &t)| C.sum_goods(|g| st.f_order[(i, g)] * p[(t, g)])).collect();
        for (r, &i) in oi.iter().enumerate() {
            let frac = if remaining_value[r] > 1e-12 { inv_pay[r] / remaining_value[r].max(1e-12) } else { 1.0 };
            let addK = st.f_order_K[i] * frac.clamp(0.0, 1.0);
            st.f_K[i] += addK;
            st.f_order_K[i] -= addK;
            for g in 0..G {
                st.f_order[(i, g)] = (st.f_order[(i, g)] - inv_got[(r, g)]).max(0.0);
            }
            let done = C.sum_goods_of(st.f_order.row(i)) < 1e-6 || st.f_order_K[i] < 1e-9;
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
    // tasks: the works pay for the plant of their machines as it arrives
    if let Some(mo) = ledger.line((Buyer::WorksTasks, Trade::Plant)) {
        let pay = crate::tasks::deliver(st, C, mo);
        let by_town = bincount(&mo.town, &pay, T);
        add_to(&mut merch, &by_town);
        add_to(&mut sales_value, &by_town);
        st.m_invest += pairwise_sum(&pay);
    }
    // tasks: the land's plant goes into place as it arrives (paid above, in the rent)
    if let Some(plant) = ledger.line(LAND_TASKS[1]) {
        crate::tasks::land_deliver(st, C, plant);
    }
    // the lorry: its plant goes into place as it arrives (paid above, by the households)
    if let (Some(run), Some(plant)) = (ledger.line(LORRIES[0]), ledger.line(LORRIES[1])) {
        crate::tasks::cart_deliver(st, C, run, plant, fill);
    }

    let proj_pay = ledger.paid(C, &st.price, &[(Buyer::Ways, Trade::Plant)]);
    let proj_got = ledger.got((Buyer::Ways, Trade::Plant)).unwrap();
    let mut proj_spend = 0.0;
    for (k, pr) in st.projects.iter_mut().enumerate() {
        let (pay, got) = (proj_pay[k], proj_got.row(k));
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
    // state: the university's rooms paid for out of the treasury
    if let Some(rooms) = ledger.line((Buyer::University, Trade::Plant)) {
        crate::university::rooms_got(st, P, rooms, &mut merch, &mut sales_value);
    }

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
    let decay = crate::tree::decay(C, P); // tree: the content's rate for a good the port's table does not have
    // tree: and all that is left of a good that does not keep (electricity)
    let loss = crate::tree::month_loss(C, P);
    // a probe may ask for the merchants' month (`merchants_month`): the stock before sales
    let inv0 = st.watch_merchants.then(|| st.inv.clone());
    for t in 0..T {
        for g in 0..G {
            let v = &mut st.inv[(t, g)];
            *v -= ledger.sold[(t, g)];
            *v *= 1.0 - loss[g];
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
    // a probe's merchants' month: what they took before the trade, and each town's
    // trade as they paid for it (sent at its price, brought at the sender's, carriage)
    let merch_pre = if st.watch_merchants { merch.clone() } else { Vec::new() };
    let mut trade_m: Vec<[f64; 3]> = if st.watch_merchants { vec![[0.0; 3]; T] } else { Vec::new() };
    let A_idx: Vec<usize> = (0..T).filter(|&t| st.town_active[t]).collect();
    let na = A_idx.len();
    if na > 1 {
        let p = &st.price;
        // tau, want and qt are (na, na, G), indexed [(i * na + j) * G + g]
        let mut tau = vec![0.0; na * na * G];
        let mut want = vec![0.0; na * na * G];
        // tree: a good made where it is used (building work, town gas) is not
        // carried: nothing is wanted of it and it is left out of the sums. Under
        // the port every good is carried and `tg` is all of them in their order
        let tg = crate::tree::traded_goods(C);
        for (i, &ti) in A_idx.iter().enumerate() {
            for (j, &tj) in A_idx.iter().enumerate() {
                let c0 = st.town_cost[(ti, tj)];
                let c = if c0.is_finite() { c0 } else { 1e9 };
                for &g in &tg {
                    let k = (i * na + j) * G + g;
                    tau[k] = freight * C.weight[g] * c + P.handling * p[(ti, g)];
                    let gap = p[(tj, g)] - p[(ti, g)] - tau[k];
                    let frac = if i == j { 0.0 } else { (gap / p[(tj, g)].max(1e-9)).clamp(0.0, 0.5) * P.trade_k };
                    want[k] = frac * st.inv[(ti, g)];
                }
            }
        }
        let mut scale = vec![0.0; na * G];
        for i in 0..na {
            for &g in &tg {
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
                for &g in &tg {
                    qt[(i * na + j) * G + g] *= scale[i * G + g];
                }
            }
        }
        let mut ex_q = Mat::zeros(na, G);
        let mut im_q = Mat::zeros(na, G);
        for i in 0..na {
            for j in 0..na {
                for &g in &tg {
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
                pay[(i, j)] = C.sum_goods(|g| qt[base + g] * p[(ti, g)]);
                tcost[(i, j)] = C.sum_goods(|g| qt[base + g] * tau[base + g]);
                let wflow = C.sum_goods(|g| qt[base + g] * C.weight[g]);
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
            if let Some(r) = trade_m.get_mut(ti) {
                *r = [pay_out, pay_in_, tc_in];
            }
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
    if let Some(inv0) = inv0 {
        // what the merchants paid the makers: the works their output less the spread
        // (the carting out to market included), the land its produce less the spread
        let mut bought = bincount(&ft, &(0..NFI).map(|i| rev[i] + cart_out[i]).collect::<Vec<f64>>(), T);
        add_to(&mut bought, &bincount(&et, &value.iter().map(|v| v * (1.0 - spread)).collect::<Vec<f64>>(), T));
        let m = MerchantsMonth { net: &merch, before_trade: &merch_pre, bought: &bought, trade: &trade_m, inv0: &inv0, sold: &ledger.sold, made: &prod_TG, imports: &imports, loss: &loss };
        merchants_month(st, C, &m);
    }

    // accounts: the month's goods and payments go into the books before prices
    // move. Everything handed over is a local of this function, and what was
    // bought, got and sold they read from the ledger; nothing comes back.
    if st.acct.on {
        #[rustfmt::skip]
        crate::accounts::note_month(st, C, P, &crate::accounts::Month {
            et: &et, sec: &sec, Y: &Y, Lex: &Lex, cart_ex: &cart_ex,
            land_rents: match &royalties {
                Some((towns, cells, parts)) => (&towns[..], &cells[..], &parts[..]),
                None => (&et[..], &ex[..], &rent_ex[..]),
            },
            fi: &fi, ft: &ft, fm: &fm, fc: &fc, out: &out, K: &K, q: &q, labor_f: &labor_f, wages_f: &wages_f,
            rent_f: &rent_f, pay_in: &pay_in, cart_in: &cart_in, cart_out: &cart_out, rev: &rev, cut: hired.as_deref(),
            a: &a, pin: &pin,
            fam_by_town: &fam_by_town, lord_budget: &lord_budget,
            hh_spend: &hh_spend, lord_spend: &lord_spend, urban_pay: &urban_pay,
            q_hh: &q_hh, q_l: &q_l, share: &share, lord_pop: &lord_pop,
            ledger: &ledger,
            prod_TG: &prod_TG, exports: &exports, imports: &imports, w: &w, supply: &supply,
        });
    }

    // ---------------- prices respond to inventories (level) and to this month's
    // imbalance between wanted and arriving goods (flow), which damps cycles
    let (sold, R) = (&ledger.sold, &ledger.asked);
    for t in 0..T {
        for g in 0..G {
            let outflow = sold[(t, g)] + exports[(t, g)];
            let of = 0.8 * st.outflow[(t, g)] + 0.2 * outflow;
            st.outflow[(t, g)] = of;
            let r = R[(t, g)];
            // tree: the stock's gap from the months of sales held, or, for a good
            // that does not keep, the month's asking against its making
            let gap = crate::tree::price_gap(C, P, g, st.inv[(t, g)], of, r, prod_TG[(t, g)]);
            let arriving = prod_TG[(t, g)] + imports[(t, g)];
            // prices.exports_ask: and what merchants carried out is asked of the market too
            let r = prices::asked(&P.prices, r, exports[(t, g)]);
            let imbalance = if r + arriving > 1e-9 { (r - arriving) / (r + arriving).max(1e-9) } else { 0.0 };
            if st.town_active[t] {
                let lo_p = crate::longrun::floor(st, P, C, g);
                let hi_p = crate::longrun::ceiling(st, P, C, g);
                if P.prices.enabled {
                    // prices: the rule below moves a slow anchor, and the price
                    // stands above or below it with the merchants' stock (none
                    // on a good that spoils, under `prices.no_spoil_markup`)
                    prices::mark(st, P, t, g, gap, imbalance, lo_p, hi_p, decay[g]);
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
        st.month_requests = R.clone();
        st.month_sold = sold.clone();
    }

    // ---------------- dividends, common fund, issuance
    add_to(&mut st.merch_cash, &merch);
    let mut fdiv = vec![0.0; T];
    // owners (docs/rules/owners.md): the landlords' part of the works' dividends, by town
    let mut lord_div = Vec::new();
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
        let all = if P.owners.enabled { div.iter().sum() } else { 0.0 };
        if families::active(st) {
            families::take_dividends(st, C, &fi, &mut div);
        }
        // owners: a works no family holds pays its landlords their part, and
        // what is left in `div` is the households'
        lord_div = crate::owners::dividends(st, P, &fi, &mut div, all);
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
    // state (docs/rules/state.md): the month's pay as it was banked, before the
    // dividends, the fund and the new money join it; the rate on pay is taken on it
    let pay_banked = if P.state.enabled { hh_inc.clone() } else { Vec::new() };
    for t in 0..T {
        let popshare = st.pop[t] / pop_total;
        hh_inc[t] += fdiv[t] + mdiv[t] + (fund_div + issue) * popshare;
    }
    // owners: the landlords' dividends go into their income beside the month's
    // rents (after the port's count of the money, which they would otherwise enter)
    crate::owners::credit(st, &lord_div);
    // accounts: the month's incomes, before the purses are credited
    crate::accounts::close_month(st, &fi, &ft, &fm, &merch, &hh_inc, &fdiv, &mdiv, &lord_div, fund_div, issue, pop_total);
    // state (docs/rules/state.md): the teachers paid, the rate on pay taken and
    // the pensions paid, between the treasury and the households' income
    crate::purse::month(st, P, &w, &pay_banked, &mut hh_inc);
    // the lorry (docs/rules/lorry.md, "The lorry's bill"): the lorries' part of the
    // month's carting pay goes into the hauliers' till, and what it need not keep
    // comes back out to the households (nothing without `lorry.till`)
    crate::tasks::cart_till_close(st, P, cbuy.as_ref(), &mut hh_inc);

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
    st.fill = fill.clone();
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
    // tasks: the land's work this month, and who stood at the ceiling
    if lbuy.is_some() || at_ceiling.is_some() || st.tasks.land_work.iter().any(|&v| v != 0.0) {
        let ceiling = at_ceiling.as_ref().map(|(flag, c, a)| (&ex[..], &sec[..], &flag[..], *c, *a));
        crate::tasks::land_note(st, C, lbuy.as_ref().map(|b| &b.work[..]), ceiling);
    }
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
        flow[fc[i]] += C.sum_goods(|g| use_f[(i, g)] * C.weight[g]) + q[i] * C.weight[out[i]];
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
    // state: and so are the teachers
    crate::purse::count_teachers(st, P, &mut employment);
    let mut heat_use = vec![0.0; NF];
    for i in 0..NFI {
        for k in 0..NF {
            heat_use[k] += q[i] * h[i] * hshare[(i, k)];
        }
    }
    st.month_stats = MonthStats {
        prod: prod_g,
        method_out: mo,
        hh_got: hh_got.clone(),
        lord_got: ledger.got((Buyer::Landlords, Trade::Day)).unwrap().clone(),
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
    // tasks (docs/rules/tasks.md): each town's target at the month's wage and
    // prices, and each works' machines moved toward it and worn
    crate::tasks::end_month(st, C, P);
    ledger
}

/// The columns of `State::month_merchants`, which a probe asks for by setting
/// `State::watch_merchants` (examples/century_probe.rs). Each is a town's month,
/// in money, at the prices the month was traded at:
///
/// - `net`: the merchants' takings less what they paid out, as the books count
///   their income (`accounts`, `merchants_net`);
/// - `sales`: what the town's buyers paid them, of every kind; `at_price`: what
///   the buyers got, at the town's prices (the two differ where a works bought its
///   inputs below the town's price);
/// - `bought`: what they paid the town's land and works for their produce;
///   `made`: that produce at the town's prices (the gap is the merchants' spread);
/// - `sent`: what they carried out, at the town's prices; `brought_cost`: what
///   they paid for what they brought in, at the prices of the towns it came from;
///   `brought_value`: what they brought in, at this town's prices; `carriage`:
///   the carting and handling they paid on it;
/// - `spoiled`: what was lost of the stock left after the month's sales;
/// - `stock_open` and `stock_close`: the stock before the month's sales and after
///   its trade.
///
/// So that `net = spread * made + (brought_value - brought_cost - carriage)
/// - spoiled + (stock_open - stock_close) + (sales - at_price)`, less only what the
/// stock's floor at nothing makes up.
pub const MERCHANT_TERMS: [&str; 12] =
    ["net", "sales", "at_price", "bought", "made", "sent", "brought_cost", "brought_value", "carriage", "spoiled", "stock_open", "stock_close"];

/// What `merchants_month` is handed: the month's locals of `step_month`.
struct MerchantsMonth<'a> {
    net: &'a [f64],
    before_trade: &'a [f64],
    bought: &'a [f64],
    trade: &'a [[f64; 3]],
    inv0: &'a Mat,
    sold: &'a Mat,
    made: &'a Mat,
    imports: &'a Mat,
    loss: &'a [f64],
}

/// The merchants' month by town into `st.month_merchants` ([`MERCHANT_TERMS`]),
/// for a probe; called only while `st.watch_merchants` is set. Reads, changes
/// nothing of the run.
fn merchants_month(st: &mut State, C: &Content, m: &MerchantsMonth) {
    let (T, G) = (st.price.rows, C.G);
    let mut out = Mat::zeros(T, MERCHANT_TERMS.len());
    for t in 0..T {
        let p = st.price.row(t);
        let mut v = [0.0; 12];
        v[0] = m.net[t];
        v[1] = m.before_trade[t] + m.bought[t];
        v[3] = m.bought[t];
        if let Some(&[sent, brought_cost, carriage]) = m.trade.get(t) {
            (v[5], v[6], v[8]) = (sent, brought_cost, carriage);
        }
        for g in 0..G {
            v[2] += p[g] * m.sold[(t, g)];
            v[4] += p[g] * m.made[(t, g)];
            v[7] += p[g] * m.imports[(t, g)];
            v[9] += p[g] * m.loss[g] * (m.inv0[(t, g)] - m.sold[(t, g)]);
            v[10] += p[g] * m.inv0[(t, g)];
            v[11] += p[g] * st.inv[(t, g)];
        }
        for (k, x) in v.into_iter().enumerate() {
            out[(t, k)] = x;
        }
    }
    st.month_merchants = out;
}

fn add_to(a: &mut [f64], b: &[f64]) {
    for (x, y) in a.iter_mut().zip(b) {
        *x += y;
    }
}
