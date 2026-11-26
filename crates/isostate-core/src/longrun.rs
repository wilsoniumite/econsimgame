//! An island that can last three centuries (not in the Python;
//! `docs/rules/longrun.md`).
//!
//! A handful of the port's constants are sums of money fixed in 1750's prices:
//! the price ceiling and floor of every good (40 and 0.05 times its base
//! price), the wage's bounds (0.02 and 500 a month), and the freight, what it
//! costs to cart a unit of weight one unit of path. Over 150 years they do no
//! harm. Run on to 2050 the money wage and the prices move far enough that
//! timber stands at its ceiling in every month from 2025 and tools come within
//! a few per cent of theirs, and a rich island's shops would stand there from
//! about 2000.
//!
//! With `longrun.enabled` those constants ride on two numbers the simulation
//! keeps for itself, each 1 at the end of the warm-up and set once a year:
//!
//! - the **anchor**, a chained cost of living: one Fisher link a year on what
//!   households and landlords bought, at the year's mean prices, averaged over
//!   the last `anchor_years` years. It is the same sum the Engel curve's
//!   observer makes (`engel::record_month`), made again here, because the
//!   observer's is never read by the economy and must stay so.
//! - the **wage scale**, the island's mean wage weighted by people, averaged
//!   over the last `anchor_years` years, over its value at the end of the
//!   warm-up.
//!
//! A good's ceiling is `price_ceiling * base * max(anchor, wage scale)`, its
//! floor `price_floor * base * anchor`; the wage's bounds are `0.02 * anchor`
//! and `500 * max(anchor, wage scale)`; freight is `freight * anchor`. With the
//! switch off each of the functions below returns the constant itself, by an
//! early return and not by multiplying by one.
//!
//! `standin_growth` is a stand-in, to be deleted: it lifts the two learning
//! caps (`learn_cap` and `machines.quality_cap`) to 64 so that the anchors can
//! be tried on an island that goes on growing before anything that would make
//! it grow is built. It is for sweeps only; a live game never runs it.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::facts::{Gathered, Kind};
use crate::observer::national_prices;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;
use crate::util::pairwise_sum_by;

/// What the stand-in lifts both learning caps to.
pub const STANDIN_CAP: f64 = 64.0;

/// The wage's bounds in 1750's money: what `econ::step_month` brackets each
/// town's wage within.
pub const WAGE_LO: f64 = 0.02;
pub const WAGE_HI: f64 = 500.0;

/// The facts count prices and wages at a bound from this year on: the first
/// fifty years are the port's own and not what the anchors are for.
pub const BOUND_FROM: i64 = 1801;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LongrunParams {
    /// The bounds on prices and the wage, and the freight, ride on the anchor
    /// and the wage scale.
    pub enabled: bool,
    /// How many years the anchor and the wage scale are averaged over.
    pub anchor_years: usize,
    /// A stand-in, to be deleted: both learning caps read as `STANDIN_CAP`.
    /// For sweeps only; it does not need `enabled`, so that a run with it and
    /// without the anchors can be set beside one with both.
    pub standin_growth: bool,
    /// The first year whose freight rides on the anchor, as the anchor stands
    /// against its value when that year began. Before it the freight is the
    /// port's constant. The first year of the run (1750) is the whole rule.
    pub freight_from: i64,
}

impl Default for LongrunParams {
    /// Off: what the port runs with.
    fn default() -> LongrunParams {
        LongrunParams { enabled: false, anchor_years: 5, standin_growth: false, freight_from: 1750 }
    }
}

impl LongrunParams {
    /// The game's: the anchors on, the freight on them from 1900, the stand-in off.
    ///
    /// The freight starts in 1900 and not in 1750 because the island's chained
    /// cost of living falls to about 0.45 by 1900: carriage riding on it from
    /// 1750 is half as dear by 1900, and on eight islands that left 190 km less
    /// railway in 1900, outside the spread of seeds, and the last town founded
    /// 35 years sooner (docs/rules/longrun.md, "Why the freight waits for 1900").
    pub fn game() -> LongrunParams {
        LongrunParams { enabled: true, freight_from: 1900, ..LongrunParams::default() }
    }
}

/// What the mechanism keeps between months (`st.longrun`). Empty while it is
/// off, and until the warm-up ends (`start`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LongrunState {
    /// The two numbers the bounds ride on: 1 until the first year is closed.
    pub anchor: f64,
    pub wage_scale: f64,
    /// The chained cost of living at the end of the last complete year (the
    /// end of the warm-up = 1).
    pub chain: f64,
    /// The last `anchor_years` levels of the chain, and of the year's mean
    /// wage over the warm-up's, the warm-up's own (1) first.
    pub levels: Vec<f64>,
    pub wages: Vec<f64>,
    /// The warm-up's last month's mean wage, weighted by people.
    pub wage0: f64,
    /// (G): the prices and the monthly purchases the next link starts from.
    pub base_p: Vec<f64>,
    pub base_q: Vec<f64>,
    /// (G): this year's prices and purchases, and the mean wage, summed month by month.
    pub acc_p: Vec<f64>,
    pub acc_q: Vec<f64>,
    pub acc_w: f64,
    pub months: usize,
    /// For the facts: the anchor and the wage scale each year ends with.
    pub years: Vec<(f64, f64)>,
    /// The first year of the run, and the anchor the freight is reckoned
    /// against from `freight_from` on (0 before then).
    pub y0: i64,
    pub freight_base: f64,
}

impl LongrunState {
    fn started(&self) -> bool {
        !self.levels.is_empty()
    }
}

/// The anchor: 1 while the mechanism is off or the warm-up is still running.
pub fn anchor(st: &State, P: &Params) -> f64 {
    if !P.longrun.enabled || !st.longrun.started() {
        return 1.0;
    }
    st.longrun.anchor
}

/// The higher of the anchor and the wage scale, which the ceilings ride on.
fn upper(st: &State) -> f64 {
    if !st.longrun.started() {
        return 1.0;
    }
    st.longrun.anchor.max(st.longrun.wage_scale)
}

/// Good `g`'s price ceiling.
pub fn ceiling(st: &State, P: &Params, C: &Content, g: usize) -> f64 {
    if !P.longrun.enabled {
        return P.price_ceiling * C.base_price[g];
    }
    P.price_ceiling * C.base_price[g] * upper(st)
}

/// Good `g`'s price floor.
pub fn floor(st: &State, P: &Params, C: &Content, g: usize) -> f64 {
    if !P.longrun.enabled {
        return P.price_floor * C.base_price[g];
    }
    P.price_floor * C.base_price[g] * anchor(st, P)
}

/// The wage's bounds, lowest and highest.
pub fn wage_bounds(st: &State, P: &Params) -> (f64, f64) {
    if !P.longrun.enabled {
        return (WAGE_LO, WAGE_HI);
    }
    (WAGE_LO * anchor(st, P), WAGE_HI * upper(st))
}

/// What it costs to cart a unit of weight one unit of path.
pub fn freight(st: &State, P: &Params) -> f64 {
    if !P.longrun.enabled {
        return P.freight;
    }
    let lr = &st.longrun;
    if !lr.started() || lr.freight_base <= 0.0 {
        return P.freight;
    }
    P.freight * (lr.anchor / lr.freight_base)
}

/// The methods' learning cap (`firms::learning`).
pub fn learn_cap(P: &Params) -> f64 {
    if !P.longrun.standin_growth {
        return P.learn_cap;
    }
    STANDIN_CAP
}

/// The machines' quality cap (`machines::quality`).
pub fn quality_cap(P: &Params) -> f64 {
    if !P.longrun.standin_growth {
        return P.machines.quality_cap;
    }
    STANDIN_CAP
}

/// What households and landlords bought this month, all towns together, and
/// their mean wage weighted by people: the observer's sums, made again.
fn month_sums(st: &State, C: &Content) -> (Vec<f64>, f64) {
    let (T, G) = (st.T_max, C.G);
    let ms = &st.month_stats;
    let mut bought = vec![0.0; G];
    for i in 0..T {
        for g in 0..G {
            bought[g] += ms.hh_got[(i, g)] + ms.lord_got[(i, g)];
        }
    }
    let pop: Vec<f64> = (0..T).map(|i| if st.town_active[i] { st.pop[i] } else { 0.0 }).collect();
    let pop_sum = crate::util::pairwise_sum(&pop);
    let wage = pairwise_sum_by(T, |i| st.wage[i] * pop[i]) / pop_sum.max(1.0);
    (bought, wage)
}

/// The warm-up is over: the chain starts where the fixed basket does, and the
/// wage scale from the warm-up's last month (`sim::start_from`, after the 1750
/// basket and the wage base are fixed).
pub fn start(st: &mut State, P: &Params, y0: i64) {
    if !P.longrun.enabled {
        return;
    }
    let G = st.cpi_basket.len();
    let wage0 = st.wage_base;
    let lr = &mut st.longrun;
    lr.anchor = 1.0;
    lr.wage_scale = 1.0;
    lr.chain = 1.0;
    lr.levels = vec![1.0];
    lr.wages = vec![1.0];
    lr.wage0 = wage0;
    lr.base_p = st.cpi_base_prices.clone();
    lr.base_q = st.cpi_basket.clone();
    lr.acc_p = vec![0.0; G];
    lr.acc_q = vec![0.0; G];
    lr.acc_w = 0.0;
    lr.months = 0;
    lr.years.clear();
    lr.y0 = y0;
    lr.freight_base = if P.longrun.freight_from <= y0 { 1.0 } else { 0.0 };
}

/// After each month of the recorded run (`sim::run_month`): the month's
/// prices, purchases and wage go into the year's sums, and when the twelfth is
/// in, the year closes a link of the chain and sets the anchor and the wage
/// scale that the next year's bounds and freight ride on.
pub fn note_month(st: &mut State, C: &Content, P: &Params) {
    if !P.longrun.enabled || !st.longrun.started() {
        return;
    }
    let p_nat = national_prices(st);
    let (bought, wage) = month_sums(st, C);
    let keep = P.longrun.anchor_years.max(1);
    let lr = &mut st.longrun;
    for g in 0..lr.acc_p.len() {
        lr.acc_p[g] += p_nat[g];
        lr.acc_q[g] += bought[g];
    }
    lr.acc_w += wage;
    lr.months += 1;
    if lr.months < 12 {
        return;
    }
    let p: Vec<f64> = lr.acc_p.iter().map(|v| v / 12.0).collect();
    let q: Vec<f64> = lr.acc_q.iter().map(|v| v / 12.0).collect();
    lr.chain *= crate::engel::fisher_from(C.G_base, &lr.base_p, &lr.base_q, &p, &q);
    lr.base_p = p;
    lr.base_q = q;
    let w = lr.acc_w / 12.0 / lr.wage0.max(1e-12);
    lr.levels.push(lr.chain);
    lr.wages.push(w);
    for v in [&mut lr.levels, &mut lr.wages] {
        if v.len() > keep {
            v.drain(..v.len() - keep);
        }
    }
    lr.anchor = lr.levels.iter().sum::<f64>() / lr.levels.len() as f64;
    lr.wage_scale = lr.wages.iter().sum::<f64>() / lr.wages.len() as f64;
    lr.years.push((lr.anchor, lr.wage_scale));
    // the year that begins now is `freight_from`: its freight starts from here
    if lr.y0 + lr.years.len() as i64 == P.longrun.freight_from {
        lr.freight_base = lr.anchor;
    }
    lr.acc_p.iter_mut().for_each(|v| *v = 0.0);
    lr.acc_q.iter_mut().for_each(|v| *v = 0.0);
    lr.acc_w = 0.0;
    lr.months = 0;
}

/// What the sweep notes month by month, under the rules on or off, since what
/// it counts is the question the mechanism answers: town-months from
/// `BOUND_FROM` on, and of them those in which each good's price, and the
/// wage, stood at a bound.
#[derive(Clone, Debug, Default)]
pub struct BoundNotes {
    pub town_months: usize,
    pub goods: Vec<usize>,
    pub wage: usize,
}

/// A price or wage within a millionth (in its logarithm) of a bound stands at it.
fn at(x: f64, lo: f64, hi: f64) -> bool {
    x <= lo * (1.0 + 1e-6) || x >= hi * (1.0 - 1e-6)
}

/// Called from `facts::gather_month`, after each month of the recorded run.
pub fn gather_month(n: &mut BoundNotes, st: &State, C: &Content, P: &Params, year: i64) {
    if year < BOUND_FROM {
        return;
    }
    if n.goods.len() != C.G {
        n.goods = vec![0; C.G];
    }
    let (wlo, whi) = wage_bounds(st, P);
    let lohi: Vec<(f64, f64)> = (0..C.G).map(|g| (floor(st, P, C, g), ceiling(st, P, C, g))).collect();
    for t in (0..st.T_max).filter(|&t| st.town_active[t] && st.pop[t] > 0.0) {
        n.town_months += 1;
        for (g, &(lo, hi)) in lohi.iter().enumerate() {
            n.goods[g] += at(st.price[(t, g)], lo, hi) as usize;
        }
        n.wage += at(st.wage[t], wlo, whi) as usize;
    }
}

/// Facts for the sweep's table, under "Prices and cycles" (`facts::stylised_facts`).
///
/// - `months_at_bound_GOOD`: the share of town-months from 1801 on in which
///   the good's price stood at its floor or ceiling, and `months_at_bound_wage`
///   the same for the wage, whatever the switch (with it off the bounds are the
///   port's constants);
/// - `anchor_Y` and `wage_scale_Y` at the sweep's marks, as the year ended
///   (only with the mechanism on).
///
/// (`works_slots_used`, the places in the works' table taken by the run's end,
/// is among the works' facts.)
pub fn facts(r: &RunResult, g: &Gathered) -> Vec<(String, Kind, f64)> {
    let (st, C, P) = (&r.state, &r.content, &r.params);
    let mut out = Vec::new();
    let n = &g.months.bounds;
    let share = |k: usize| if n.town_months > 0 { k as f64 / n.town_months as f64 } else { f64::NAN };
    for (k, name) in C.good_name.iter().enumerate() {
        out.push((format!("months_at_bound_{name}"), Kind::Share, share(n.goods.get(k).copied().unwrap_or(0))));
    }
    out.push(("months_at_bound_wage".into(), Kind::Share, share(n.wage)));
    if P.longrun.enabled {
        let y0 = C.start_year;
        let y1 = y0 + r.rec.n_years as i64 - 1;
        for y in crate::facts::marks(y0, y1) {
            let (a, w) = st.longrun.years.get((y - y0) as usize).copied().unwrap_or((f64::NAN, f64::NAN));
            out.push((format!("anchor_{y}"), Kind::Ratio, a));
            out.push((format!("wage_scale_{y}"), Kind::Ratio, w));
        }
    }
    out
}
