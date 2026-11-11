//! An Engel curve in what households buy, and a chained price index. Not in the
//! Python; `docs/rules/engel.md` is the longer account.
//!
//! The port's households split whatever they have above subsistence by fixed
//! shares (`C.shares`: 28 per cent on food, however rich they are). So a town
//! three times as well off eats nearly three times as much, the farms keep their
//! hands, and neither the works nor the towns ever gain on the land.
//!
//! Here the split of each further shilling depends on how well off the town
//! already is. Up to a reference income (`y_ref`, in subsistence bundles a head:
//! about where the island stands in 1750) it is the content's shares. Past it,
//! each further shilling leans toward a second, richer set (`C.shares_rich`: less
//! food and warmth, more comfort, a little more clothing and shelter), by a
//! weight that rises smoothly from 0 toward 1:
//!
//! ```text
//! weight(y) = 1 - (y_ref / y)^strength        for y > y_ref, else 0
//! ```
//!
//! What a household spends on a need is its subsistence, plus the sum of those
//! shillings one by one (the integral of the marginal shares from subsistence up
//! to its income). The integral has a closed form, `richer_part`. Because both
//! sets of shares add up to the same total, the budget is spent exactly at every
//! income; because every marginal share lies between the two sets, more income
//! never buys less of anything.
//!
//! Landlords keep their own shares (`C.lord_shares`), and families spend as
//! landlords do. Nothing else changes hands differently: the households' orders
//! go to the same town markets at the same prices as before.
//!
//! The second half is the observer's. With the 1750 basket the cost of living is
//! weighed by firewood and homespun for 150 years, and with an Engel curve the
//! basket moves faster still. So when the mechanism is on the observer also keeps
//! a chained index: one Fisher link a year on what households and landlords
//! actually bought, and the real wage on it, beside the old fixed-basket series.
//! It also keeps the households' budget shares by need and their real income,
//! which are the two axes of the curve.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::mat::Mat;
use crate::observer::Series;
use crate::state::State;
use crate::util::{pairwise_sum, pairwise_sum_by};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EngelParams {
    /// Households' marginal budget shares move with their real income.
    pub enabled: bool,
    /// Real income a head (goods budget over the cost of subsistence) up to which
    /// the content's shares hold. About the island's level in 1750. Anything
    /// under 1 is taken as 1: subsistence is always bought by the subsistence rule.
    pub y_ref: f64,
    /// How fast the marginal shares turn past `y_ref`: at twice the reference
    /// income they are `1 - 0.5^strength` of the way to the richer set. 0 is a
    /// flat curve (the port's households, with the new series recorded).
    pub strength: f64,
}

impl Default for EngelParams {
    /// Off: what the port runs with.
    fn default() -> EngelParams {
        EngelParams { enabled: false, y_ref: 2.0, strength: 3.0 }
    }
}

impl EngelParams {
    /// The game's Engel curve.
    pub fn game() -> EngelParams {
        EngelParams { enabled: true, ..EngelParams::default() }
    }
}

/// The richer set of marginal shares for the port's six needs, used when a
/// content file gives no `shares_rich` of its own. Against the content's
/// (0.28, 0.14, 0.22, 0.19, 0.07, 0.10): bread and firing take much less of a
/// further shilling, shelter and clothing a little more, tools the same (they
/// are what makes a land worker better, and that channel is left alone), and
/// comfort (shops, trades, service) takes what is freed.
pub const RICH_SHARES: [(&str, f64); 6] =
    [("food", 0.05), ("warmth", 0.06), ("shelter", 0.24), ("clothing", 0.22), ("implements", 0.07), ("comfort", 0.36)];

/// The richer set for a content file that has none: each need named in
/// `RICH_SHARES` takes its share from there and a need that is not keeps the
/// content's share; the named ones are then scaled so that the whole adds up to
/// what `shares` adds up to (to nothing, if the needs it does not know already
/// take the whole of it). For the port's six needs that is `RICH_SHARES` as
/// written.
pub fn default_shares_rich(names: &[String], shares: &[f64]) -> Vec<f64> {
    let named: Vec<Option<f64>> = names.iter().map(|n| RICH_SHARES.iter().find(|(r, _)| r == n).map(|&(_, v)| v)).collect();
    let mut rich: Vec<f64> = named.iter().zip(shares).map(|(v, &b)| v.unwrap_or(b)).collect();
    let want = pairwise_sum(shares);
    let known: f64 = named.iter().flatten().sum();
    let rest = pairwise_sum(&rich) - known;
    if known > 0.0 && (known + rest - want).abs() > 1e-12 {
        let scale = ((want - rest) / known).max(0.0);
        for (v, n) in rich.iter_mut().zip(&named) {
            if n.is_some() {
                *v *= scale;
            }
        }
    }
    rich
}

/// How far a further shilling leans toward the richer shares at real income `y`:
/// 0 up to `y_ref`, then `1 - (y_ref / y)^strength`, rising toward 1.
pub fn weight(y: f64, E: &EngelParams) -> f64 {
    let y_ref = E.y_ref.max(1.0);
    if E.strength <= 0.0 || !(y > y_ref) {
        return 0.0;
    }
    1.0 - (y_ref / y).powf(E.strength)
}

/// The integral of `weight` from `y_ref` to `y`: of a household's income (in
/// subsistence bundles), the part that is spent by the richer shares.
///
/// With x = y / y_ref and k = strength it is
/// `(y - y_ref) - y_ref (x^(1-k) - 1) / (1 - k)`, and `(y - y_ref) - y_ref ln x`
/// at k = 1; written through `exp_m1` so that it is one formula for every k.
pub fn richer_part(y: f64, E: &EngelParams) -> f64 {
    let y_ref = E.y_ref.max(1.0);
    if E.strength <= 0.0 || !(y > y_ref) {
        return 0.0;
    }
    let lx = (y / y_ref).ln();
    let a = (1.0 - E.strength) * lx;
    // (x^(1-k) - 1) / (1 - k) = ln x * (e^a - 1) / a
    let g = lx * if a.abs() < 1e-8 { 1.0 + 0.5 * a } else { a.exp_m1() / a };
    ((y - y_ref) - y_ref * g).clamp(0.0, y - y_ref)
}

/// `econ::les` with marginal shares that move with income: the linear expenditure
/// system over services, where the budget above subsistence is split by `beta` up
/// to the reference income and leans toward `rich` past it.
/// Returns quantities (T, S) and the cost of the subsistence bundle per head.
///
/// A town at or below the reference income gets what `econ::les` gives it, bit
/// for bit.
pub fn les(budget_pc: &[f64], unit: &Mat, gamma: &[f64], beta: &[f64], rich: &[f64], E: &EngelParams) -> (Mat, Vec<f64>) {
    let (T, S) = (unit.rows, unit.cols);
    let mut q = Mat::zeros(T, S);
    let mut sub = vec![0.0; T];
    for t in 0..T {
        sub[t] = pairwise_sum_by(S, |s| gamma[s] * unit[(t, s)]);
        let extra = (budget_pc[t] - sub[t]).max(0.0);
        let scale = if budget_pc[t] < sub[t] { budget_pc[t] / sub[t].max(1e-12) } else { 1.0 };
        // money a head spent by the richer shares; never more than there is above subsistence
        let moved = if sub[t] > 0.0 { (sub[t] * richer_part(budget_pc[t] / sub[t], E)).min(extra) } else { 0.0 };
        for s in 0..S {
            q[(t, s)] = gamma[s] * scale + (beta[s] * extra + (rich[s] - beta[s]) * moved) / unit[(t, s)].max(1e-12);
        }
    }
    (q, sub)
}

/// A Fisher price index from (p0, q0) to (p1, q1): the geometric mean of the
/// index on the old basket and the index on the new one. 1 if nothing was bought.
pub fn fisher(p0: &[f64], q0: &[f64], p1: &[f64], q1: &[f64]) -> f64 {
    let n = p0.len();
    let old = pairwise_sum_by(n, |g| p1[g] * q0[g]) / pairwise_sum_by(n, |g| p0[g] * q0[g]);
    let new = pairwise_sum_by(n, |g| p1[g] * q1[g]) / pairwise_sum_by(n, |g| p0[g] * q1[g]);
    match (old.is_finite() && old > 0.0, new.is_finite() && new > 0.0) {
        (true, true) => (old * new).sqrt(),
        (true, false) => old,
        (false, true) => new,
        (false, false) => 1.0,
    }
}

/// What the mechanism keeps between months (`st.engel`). All of it is the
/// observer's: the simulation never reads it. Empty while the mechanism is off.
#[derive(Clone, Debug, Default)]
pub struct EngelState {
    /// (S): what households spent on each need last month, all towns together.
    pub need_spend: Vec<f64>,
    /// Households' goods budget over the cost of their subsistence last month, all towns together.
    pub real_income: f64,
    /// The chained index at the end of the last complete year (the 1750 base = 1).
    pub chain: f64,
    /// (G): the prices and the monthly purchases the next link starts from: the
    /// last complete year's means, or before 1751 the fixed basket's own base.
    pub base_p: Vec<f64>,
    pub base_q: Vec<f64>,
    /// (G): this year's prices and purchases, summed month by month.
    pub acc_p: Vec<f64>,
    pub acc_q: Vec<f64>,
    pub months: usize,
}

/// Called from `econ::step_month` once the month's sales are known and before
/// prices move: what households spent on each need, and their real income.
pub fn note_month(st: &mut State, C: &Content, pc: &[f64], subcost: &[f64], q_hh: &Mat, share: &Mat, fill: &Mat) {
    let (T, G, S) = (st.T_max, C.G, C.S);
    let mut need = vec![0.0; S];
    let (mut budget, mut floor) = (0.0, 0.0);
    for t in 0..T {
        if !st.town_active[t] {
            continue;
        }
        budget += pc[t] * st.pop[t];
        floor += subcost[t] * st.pop[t];
        for s in 0..S {
            let mut paid = 0.0;
            for g in 0..G {
                let p = C.props[(g, s)];
                if p > 0.0 {
                    paid += share[(t, g * S + s)] / p * fill[(t, g)] * st.price[(t, g)];
                }
            }
            need[s] += q_hh[(t, s)] * st.pop[t] * paid;
        }
    }
    st.engel.need_spend = need;
    st.engel.real_income = if floor > 0.0 { budget / floor } else { f64::NAN };
}

/// The mechanism's series, added after the port's so those keep their places.
pub fn add_series(reg: &mut Series, C: &Content, n_months: usize) {
    reg.add("cpi_chained", "Cost of living, chained", "prices", "1750 = 1", n_months);
    reg.add("real_wage_chained", "Real wage, chained", "people", "1750 = 1", n_months);
    reg.add("real_income", "Household budget over subsistence", "people", "subsistence = 1", n_months);
    for (s, name) in C.service_name.iter().enumerate() {
        reg.add(&format!("budget_{name}"), &C.service_label[s], "budget", "share", n_months);
    }
}

/// Called from `observer::record_month`: the chained index and the real wage on
/// it, the households' real income and their budget shares by need.
///
/// The chain has one link a year. A month is read against the last complete
/// year (its mean prices and mean monthly purchases), so its value is known at
/// once; when the year's twelfth month is in, the year's own means close the
/// link and become the base of the next.
pub fn record_month(st: &mut State, C: &Content, s: &mut Series, t: usize, wage: f64, P_nat: &[f64]) {
    let (T, G) = (st.T_max, C.G);
    let ms = &st.month_stats;
    let mut bought = vec![0.0; G];
    for i in 0..T {
        for g in 0..G {
            bought[g] += ms.hh_got[(i, g)] + ms.lord_got[(i, g)];
        }
    }
    let e = &mut st.engel;
    if e.base_p.is_empty() {
        // the first link starts where the fixed basket does: the last month of the warm-up
        e.chain = 1.0;
        e.base_p = st.cpi_base_prices.clone();
        e.base_q = st.cpi_basket.clone();
        e.acc_p = vec![0.0; G];
        e.acc_q = vec![0.0; G];
        e.months = 0;
    }
    let cpi = e.chain * fisher(&e.base_p, &e.base_q, P_nat, &bought);
    s.set("cpi_chained", t, cpi);
    s.set("real_wage_chained", t, wage / cpi.max(1e-9) / st.wage_base);
    s.set("real_income", t, e.real_income);
    let spent = pairwise_sum(&e.need_spend);
    for (k, name) in C.service_name.iter().enumerate() {
        let v = e.need_spend.get(k).copied().unwrap_or(0.0);
        s.set(&format!("budget_{name}"), t, if spent > 1e-9 { v / spent } else { f64::NAN });
    }
    for g in 0..G {
        e.acc_p[g] += P_nat[g];
        e.acc_q[g] += bought[g];
    }
    e.months += 1;
    if e.months == 12 {
        let p: Vec<f64> = e.acc_p.iter().map(|v| v / 12.0).collect();
        let q: Vec<f64> = e.acc_q.iter().map(|v| v / 12.0).collect();
        e.chain *= fisher(&e.base_p, &e.base_q, &p, &q);
        e.base_p = p;
        e.base_q = q;
        e.acc_p.iter_mut().for_each(|v| *v = 0.0);
        e.acc_q.iter_mut().for_each(|v| *v = 0.0);
        e.months = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on(strength: f64) -> EngelParams {
        EngelParams { enabled: true, y_ref: 2.0, strength }
    }

    #[test]
    fn the_weight_starts_at_nought_and_rises_toward_one() {
        let E = on(3.0);
        assert_eq!(weight(1.0, &E), 0.0);
        assert_eq!(weight(2.0, &E), 0.0);
        assert!((weight(4.0, &E) - 0.875).abs() < 1e-12);
        assert!(weight(200.0, &E) > 0.999_99);
        let mut last = 0.0;
        for k in 0..400 {
            let w = weight(2.0 + 0.05 * k as f64, &E);
            assert!(w >= last && w < 1.0);
            last = w;
        }
        assert_eq!(weight(9.0, &on(0.0)), 0.0);
    }

    #[test]
    fn the_richer_part_is_the_integral_of_the_weight() {
        for k in [0.25, 0.999_999_999, 1.0, 1.000_001, 2.0, 3.0, 8.0, 40.0] {
            let E = on(k);
            for y in [2.0f64, 2.000_001, 2.3, 3.7, 5.5, 12.0, 80.0] {
                // Simpson's rule on 20,000 slices of ln y (a steep curve does all its turning near y_ref)
                let n = 20_000;
                let h = (y / 2.0).ln() / n as f64;
                let at = |i: usize| {
                    let x = 2.0 * (h * i as f64).exp();
                    weight(x, &E) * x
                };
                let mut sum = at(0) + at(n);
                for i in 1..n {
                    sum += at(i) * if i % 2 == 1 { 4.0 } else { 2.0 };
                }
                let want = sum * h / 3.0;
                let got = richer_part(y, &E);
                assert!((got - want).abs() < 1e-9 * (1.0 + want), "k = {k}, y = {y}: {got} against {want}");
                assert!(got >= 0.0 && got <= y - 2.0);
            }
        }
        assert_eq!(richer_part(1.5, &on(3.0)), 0.0);
        assert_eq!(richer_part(7.0, &on(0.0)), 0.0);
    }

    #[test]
    fn the_default_richer_set_is_by_name_and_adds_up() {
        let names: Vec<String> = RICH_SHARES.iter().map(|(n, _)| n.to_string()).collect();
        let shares = [0.28, 0.14, 0.22, 0.19, 0.07, 0.10];
        let rich = default_shares_rich(&names, &shares);
        assert_eq!(rich, RICH_SHARES.iter().map(|&(_, v)| v).collect::<Vec<_>>());
        assert!((pairwise_sum(&rich) - 1.0).abs() < 1e-12);
        // a need the table does not know keeps its share, and the rest make room for it
        let names: Vec<String> = ["food", "light", "comfort"].iter().map(|s| s.to_string()).collect();
        let rich = default_shares_rich(&names, &[0.5, 0.2, 0.3]);
        assert!((pairwise_sum(&rich) - 1.0).abs() < 1e-12);
        assert_eq!(rich[1], 0.2);
        assert!(rich[0] < 0.5 && rich[2] > 0.3);
        // where the needs it does not know take every further shilling, the named ones get none
        let rich = default_shares_rich(&names, &[0.0, 1.0, 0.0]);
        assert_eq!(rich, vec![0.0, 1.0, 0.0]);
        // and a content that names none of them keeps its own shares
        let names: Vec<String> = ["bread", "light"].iter().map(|s| s.to_string()).collect();
        assert_eq!(default_shares_rich(&names, &[0.6, 0.4]), vec![0.6, 0.4]);
    }

    #[test]
    fn a_fisher_index_of_doubled_prices_is_two() {
        let (p0, q0) = ([1.0, 4.0, 0.5], [10.0, 2.0, 0.0]);
        let p1: Vec<f64> = p0.iter().map(|p| 2.0 * p).collect();
        assert!((fisher(&p0, &q0, &p1, &[3.0, 9.0, 1.0]) - 2.0).abs() < 1e-12);
        // with one basket it is the fixed-basket index
        let p2 = [1.5, 4.0, 0.5];
        assert!((fisher(&p0, &q0, &p2, &q0) - 23.0 / 18.0).abs() < 1e-12);
        // a buyer who moves away from what grew dear is hurt less than the old basket says
        let moved = fisher(&p0, &q0, &p2, &[4.0, 3.5, 0.0]);
        assert!(moved < 23.0 / 18.0 && moved > 1.0);
        assert_eq!(fisher(&p0, &[0.0; 3], &p1, &[0.0; 3]), 1.0);
    }
}
