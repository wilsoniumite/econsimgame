//! Care: work of presence in the towns. Not in the Python (`docs/rules/care.md`).
//!
//! Nursing, doctoring and minding the old and the sick: work that is done by
//! being there, whose output a head hardly rises in a century, whose price
//! therefore rises with the wage, and which richer households buy more of.
//! (Teaching is the treasury's already: its schools hire teachers out of the
//! rate, docs/rules/state.md.)
//!
//! The content holds it: a need, `care`, which no good serves but the good
//! `care`, made in towns by the method `care_homes` with hands and a little
//! plant, not carried between towns and not kept (`traded = false`,
//! `stock_months = 0`), every hour of it of the closed class of presence
//! (`data/tasks.toml`), so that no machine of the tasks' rule takes it, and
//! with no learning by doing (`b` 0). The good begins with its method
//! (`begins`), found by the tree's hazard on what the island has made, so it
//! moves nothing before the island finds it.
//!
//! This module holds what the content cannot say:
//!
//! - **The want** (`town_rich`): a household's marginal share on care, within
//!   the Engel rule's richer set, is `base + per_old * the town's share over
//!   64`, times what the town knows of care (from nothing to one as the idea
//!   spreads), and the other needs of the richer set give it room in
//!   proportion. So a town below the Engel rule's reference income buys none,
//!   a richer town more, an older town more, and a town that has not heard of
//!   it none.
//! - **The idea's own draws** (`is_idea`, `sought`): the method is sought only
//!   while the rule is on, and its yearly draw is made from a stream of its
//!   own (`rng::idea_stream`), not the tree's, so that every other idea is
//!   found when it was. With the rule off nothing of care is sought, wanted,
//!   built or bought, and the run is the island without it.
//!
//! Households alone pay. Landlords and families keep their own shares, which
//! have none on care.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::facts::Kind;
use crate::mat::Mat;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;

/// The name of the need, of the good and of the method (an idea of the tree).
pub const NEED: &str = "care";
pub const GOOD: &str = "care";
pub const METHOD: &str = "care_homes";

/// A town's old, weighed as `old_need` weighs them, read where the ages are not kept.
pub const OLD_WITHOUT_AGES: f64 = 0.25;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CareParams {
    /// Care is sought, wanted, made and bought.
    pub enabled: bool,
    /// A household's marginal share on care in the Engel rule's richer set,
    /// where nobody is old.
    pub base: f64,
    /// What each share of the town's old adds to it, the old weighed by
    /// `over80` (`old_need`).
    pub per_old: f64,
    /// How much more care one of the town's people of 80 and over needs than
    /// one of 65 to 79, beyond the one share each counts for.
    pub over80: f64,
    /// The most of the richer set care may take.
    pub cap: f64,
}

impl Default for CareParams {
    /// Off: what the port runs with.
    fn default() -> CareParams {
        CareParams { enabled: false, base: 0.0, per_old: 0.65, over80: 3.0, cap: 0.5 }
    }
}

impl CareParams {
    /// The game's care.
    pub fn game() -> CareParams {
        CareParams { enabled: true, ..CareParams::default() }
    }
}

/// The method of care, where the content has it (the game's content at stage 6).
pub fn method(C: &Content) -> Option<usize> {
    C.method_name.iter().position(|n| n == METHOD)
}

/// The need of care, where the content has it.
pub fn need(C: &Content) -> Option<usize> {
    C.service_name.iter().position(|n| n == NEED)
}

/// The good of care, where the content has it.
pub fn good(C: &Content) -> Option<usize> {
    C.good_index.get(GOOD).copied()
}

/// Whether idea `t` of the tree is care's method: `tech::discovery` then seeks it
/// only while [`sought`] says so, with draws of its own (`rng::idea_stream`).
pub fn is_idea(C: &Content, t: usize) -> bool {
    C.tech_kind.get(t).is_some_and(|k| k == "method") && C.tech_name[t] == METHOD
}

/// Whether care's idea may be sought: the rule and the Engel rule are on (care is
/// wanted through the Engel rule's richer set, so with it off nobody would buy it).
pub fn sought(P: &Params) -> bool {
    P.care.enabled && P.engel.enabled
}

/// Whether the content has care: its method and its need (the game's, at stage 6).
pub fn present(C: &Content) -> bool {
    method(C).is_some() && need(C).is_some()
}

/// Whether the rule acts on this content.
pub fn active(C: &Content, P: &Params) -> bool {
    sought(P) && present(C)
}

/// A town's old, as the last year's end left its ages: its people over 64 and
/// `over80` more for each of 80 and over, over all its people. (The ages keep
/// seventeen groups; the last is 80 and over.)
pub fn old_need(st: &State, P: &Params, t: usize) -> f64 {
    let Some(r) = st.ages.rows.get(t) else { return OLD_WITHOUT_AGES };
    let all: f64 = r.iter().sum();
    if !(all > 0.0) {
        return OLD_WITHOUT_AGES;
    }
    let old: f64 = r[crate::ages::OLD].iter().sum();
    (old + P.care.over80 * r[crate::ages::GROUPS - 1]) / all
}

/// The share of the richer set a town's households give care this month: the
/// want, `base + per_old * old_need`, held to `cap`, times what the town knows
/// of care.
pub fn want(st: &State, P: &Params, m: usize, t: usize) -> f64 {
    let know = st.know[(t, m)];
    if !(know > 0.0) {
        return 0.0;
    }
    let cp = &P.care;
    (know.min(1.0) * (cp.base + cp.per_old * old_need(st, P, t))).clamp(0.0, cp.cap.clamp(0.0, 1.0))
}

/// A richer set with care in it: `rich` scaled by one less `w`, and `w` added to
/// the need of care. It adds up to what `rich` adds up to, and has no share under
/// nothing where `rich` has none and `w` is between nought and one.
pub fn with_care(rich: &[f64], s_care: usize, w: f64) -> Vec<f64> {
    let mut row: Vec<f64> = rich.iter().map(|&r| r * (1.0 - w)).collect();
    row[s_care] += w;
    row
}

/// Each town's richer set with care in it (T, S): `rich` scaled by one less the
/// town's want, and the want on care. Both add up to what `rich` adds up to, and
/// none is under nothing, so the Engel rule's budget is still spent exactly.
/// None while the rule does not act, or no town wants care yet (then every town
/// keeps `rich` as it is, to the bit).
pub fn town_rich(st: &State, C: &Content, P: &Params, rich: &[f64]) -> Option<Mat> {
    if !active(C, P) {
        return None;
    }
    let (m, s_care) = (method(C)?, need(C)?);
    let T = st.T_max;
    let w: Vec<f64> = (0..T).map(|t| if st.town_active[t] { want(st, P, m, t) } else { 0.0 }).collect();
    if !w.iter().any(|&v| v > 0.0) {
        return None;
    }
    let S = rich.len();
    let mut rows = Mat::zeros(T, S);
    for t in 0..T {
        let row = if w[t] > 0.0 { with_care(rich, s_care, w[t]) } else { rich.to_vec() };
        for s in 0..S {
            rows[(t, s)] = row[s];
        }
    }
    Some(rows)
}

// ---------------------------------------------------------------- what is watched

/// What the rule keeps of a year, for the facts (`st.care.years`).
#[derive(Clone, Debug, Default)]
pub struct CareYear {
    pub year: i64,
    /// Hands in care's works over all hands at work, the year's mean.
    pub hands: f64,
    /// What households paid for care over what they paid for goods, the year's sum.
    pub spend: f64,
    /// Care's price, weighed by what was bought where, over the wage, the year's mean.
    pub price_wage: f64,
    /// Care's price over the price of the households' other goods (on the basket
    /// they bought in the base year), each weighed by what was bought where, over
    /// the same in the base year, `BASE_AFTER` years after care was first bought.
    pub price_goods: f64,
    /// Care's stock at the year's end over December's making: what December made
    /// waits for January's buyers and is lost at the month's end if they do not
    /// buy it, so this is never more than one.
    pub kept: f64,
}

/// The rule's watcher (`st.care`): the year's sums so far and every year closed.
#[derive(Clone, Debug, Default)]
pub struct CareState {
    pub acc_hands: f64,
    pub acc_spend: f64,
    pub acc_goods: f64,
    pub acc_care_paid: f64,
    pub acc_care_got: f64,
    pub acc_goods_paid: f64,
    pub acc_goods_base: f64,
    pub acc_wage: f64,
    pub months: usize,
    /// (G): the households' purchases of each good other than care, a month's
    /// mean in the base year (`BASE_AFTER` years after care was first bought):
    /// the basket the goods' price is read on.
    pub basket: Vec<f64>,
    /// (G): the national prices of that year, by what was bought where.
    pub basket_p: Vec<f64>,
    /// Care's price in the base year, by what was bought where (nought before it).
    pub first_care_p: f64,
    /// The first year care was bought (nought before it).
    pub first_bought: i64,
    pub years: Vec<CareYear>,
}

/// Years from care's first purchase to the base year of its price against goods'.
pub const BASE_AFTER: i64 = 10;

/// The month's sums, at the end of the month (`observer::record_month`).
pub fn record_month(st: &mut State, C: &Content, P: &Params, s: &mut crate::observer::Series, t: usize) {
    if !present(C) || !P.engel.enabled {
        return;
    }
    if !active(C, P) {
        s.set("care_hands", t, 0.0);
        s.set("care_spend", t, 0.0);
        s.set("care_price_wage", t, 0.0);
        return;
    }
    let (Some(m), Some(g)) = (method(C), good(C)) else { return };
    let ms = &st.month_stats;
    let emp: f64 = ms.employment.iter().sum::<f64>().max(1e-9);
    let hands = ms.firm_labor_by_method.get(m).copied().unwrap_or(0.0);
    let T = st.T_max;
    let (mut care_paid, mut care_got, mut goods_paid) = (0.0, 0.0, 0.0);
    let mut got_g = vec![0.0; C.G];
    let mut paid_g = vec![0.0; C.G];
    let mut wage_bill = 0.0;
    let mut pop = 0.0;
    for i in 0..T {
        if !st.town_active[i] {
            continue;
        }
        for k in 0..C.G {
            let q = ms.hh_got[(i, k)];
            let v = q * st.price[(i, k)];
            if k == g {
                care_paid += v;
                care_got += q;
            } else {
                goods_paid += v;
                got_g[k] += q;
                paid_g[k] += v;
            }
        }
        wage_bill += st.wage[i] * st.pop[i];
        pop += st.pop[i];
    }
    let wage = if pop > 0.0 { wage_bill / pop } else { 0.0 };
    s.set("care_hands", t, hands / emp);
    s.set("care_spend", t, if goods_paid + care_paid > 0.0 { care_paid / (goods_paid + care_paid) } else { 0.0 });
    s.set("care_price_wage", t, if care_got > 0.0 && wage > 0.0 { care_paid / care_got / wage } else { 0.0 });
    let cs = &mut st.care;
    cs.acc_hands += hands / emp;
    cs.acc_spend += care_paid;
    cs.acc_goods += goods_paid;
    cs.acc_care_paid += care_paid;
    cs.acc_care_got += care_got;
    cs.acc_wage += wage;
    cs.months += 1;
    if cs.basket.is_empty() {
        cs.basket = vec![0.0; C.G];
        cs.basket_p = vec![0.0; C.G];
    }
    // the goods' basket: what households bought of each good in the first year
    // care was bought, priced each month at that month's prices by what was bought where
    if cs.first_care_p > 0.0 {
        let now: f64 = (0..C.G).filter(|&k| k != g).map(|k| cs.basket[k] * if got_g[k] > 0.0 { paid_g[k] / got_g[k] } else { cs.basket_p[k] }).sum();
        cs.acc_goods_paid += now;
        cs.acc_goods_base += (0..C.G).filter(|&k| k != g).map(|k| cs.basket[k] * cs.basket_p[k]).sum::<f64>();
    } else {
        for k in 0..C.G {
            cs.basket[k] += got_g[k];
            cs.basket_p[k] += paid_g[k];
        }
    }
}

/// Closes the year (`sim::end_year`, after December's `record_month`).
pub fn end_year(st: &mut State, C: &Content, P: &Params, year: i64) {
    if !active(C, P) {
        return;
    }
    let g = good(C);
    let cs = &mut st.care;
    let n = cs.months.max(1) as f64;
    let care_p = if cs.acc_care_got > 0.0 { cs.acc_care_paid / cs.acc_care_got } else { f64::NAN };
    let wage = cs.acc_wage / n;
    let mut price_goods = f64::NAN;
    if cs.first_care_p > 0.0 {
        if cs.acc_goods_base > 0.0 && care_p.is_finite() {
            price_goods = (care_p / cs.first_care_p) / (cs.acc_goods_paid / cs.acc_goods_base);
        }
    } else if cs.acc_care_got > 0.0 && cs.first_bought > 0 && year >= cs.first_bought + BASE_AFTER {
        // the base year, `BASE_AFTER` years after care was first bought, once its
        // opening price is behind it: its prices and basket are the base
        cs.first_care_p = care_p;
        for k in 0..cs.basket.len() {
            cs.basket_p[k] = if cs.basket[k] > 0.0 { cs.basket_p[k] / cs.basket[k] } else { 0.0 };
            cs.basket[k] /= n;
        }
        price_goods = 1.0;
    } else {
        if cs.acc_care_got > 0.0 && cs.first_bought == 0 {
            cs.first_bought = year;
        }
        cs.basket.iter_mut().for_each(|v| *v = 0.0);
        cs.basket_p.iter_mut().for_each(|v| *v = 0.0);
    }
    let kept = match g {
        Some(g) => {
            let stock: f64 = (0..st.T_max).filter(|&i| st.town_active[i]).map(|i| st.inv[(i, g)]).sum();
            let made = st.month_stats.prod.get(g).copied().unwrap_or(0.0);
            if made > 0.0 { stock / made } else { 0.0 }
        }
        None => 0.0,
    };
    let cs = &mut st.care;
    cs.years.push(CareYear {
        year,
        hands: cs.acc_hands / n,
        spend: if cs.acc_spend + cs.acc_goods > 0.0 { cs.acc_spend / (cs.acc_spend + cs.acc_goods) } else { 0.0 },
        price_wage: if wage > 0.0 { care_p / wage } else { f64::NAN },
        price_goods,
        kept,
    });
    let (basket, basket_p, first, bought, years) =
        (std::mem::take(&mut cs.basket), std::mem::take(&mut cs.basket_p), cs.first_care_p, cs.first_bought, std::mem::take(&mut cs.years));
    *cs = CareState { basket, basket_p, first_care_p: first, first_bought: bought, years, ..CareState::default() };
}

/// The rule's series, after the others so that theirs keep their places.
///
/// Kept wherever the content has care and the Engel rule is on, the rule on or
/// off, so that two runs with it on and off have the same series (nought while
/// off; its price nought while nothing of it is bought).
pub fn add_series(reg: &mut crate::observer::Series, C: &Content, P: &Params, n_months: usize) {
    if !present(C) || !P.engel.enabled {
        return;
    }
    reg.add("care_hands", "Hands in care, of all at work", "people", "share", n_months);
    reg.add("care_spend", "Care, of what households pay for goods", "budget", "share", n_months);
    reg.add("care_price_wage", "Care's price over the wage", "prices", "wages", n_months);
}

/// The rule's facts, every tenth year from 1900 to the end: care's hands over
/// all at work (`care_hands_Y`), its part of what households pay for goods
/// (`care_spend_Y`), its price over the wage (`care_price_wage_Y`) and over the
/// households' goods (`care_price_goods_Y`, 1 in the base year, `BASE_AFTER`
/// years after it was first bought); the first year it was bought
/// (`care_first_year`) and the most of December's making in stock at a year's
/// end (`care_kept_most`, never more than one). Kept wherever the content
/// has care, without a value while the rule is off.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C) = (&r.state, &r.content);
    let mut out = Vec::new();
    if !present(C) {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    let at = |y: i64| st.care.years.iter().find(|c| c.year == y);
    let years: Vec<i64> = (1900..=y1).step_by(10).filter(|&y| y >= y0).collect();
    for &y in &years {
        out.push((format!("care_hands_{y}"), Kind::Share, at(y).map_or(f64::NAN, |c| c.hands)));
    }
    for &y in &years {
        out.push((format!("care_spend_{y}"), Kind::Share, at(y).map_or(f64::NAN, |c| c.spend)));
    }
    for &y in &years {
        out.push((format!("care_price_wage_{y}"), Kind::Real, at(y).map_or(f64::NAN, |c| c.price_wage)));
    }
    for &y in &years {
        out.push((format!("care_price_goods_{y}"), Kind::Real, at(y).map_or(f64::NAN, |c| c.price_goods)));
    }
    let first = st.care.years.iter().find(|c| c.spend > 0.0).map_or(f64::NAN, |c| c.year as f64);
    out.push(("care_first_year".to_string(), Kind::Year, first));
    let kept = st.care.years.iter().map(|c| c.kept).fold(0.0, f64::max);
    out.push(("care_kept_most".to_string(), Kind::Real, kept));
    out
}
