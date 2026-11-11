//! Merchants who mark prices with the stock they hold. Not in the Python;
//! `docs/rules/prices.md` has the account, the numbers and what a sweep showed.
//!
//! In the port a town's price of a good moves each month by how far the
//! merchants' stock is from two months of sales. The price is then the running
//! sum of past stock gaps, while the stock is the running sum of past surpluses:
//! two sums in a loop, which is a spring with almost no friction. That spring is
//! the eight-to-ten-year swing in iron, rails and machinery and the three-year
//! zigzag in farm work.
//!
//! Two things here, both off unless `prices.enabled`:
//!
//! - **The stock markup.** What the old rule moved becomes a slow *anchor*: the
//!   merchants' idea of what the good is worth, learnt from shortages and gluts
//!   that last. The price they ask stands above the anchor while their shelves
//!   are thin and below it while they are full, `anchor * exp(markup * gap)`,
//!   and moves toward that by at most 12 per cent a month. This is the friction
//!   the spring lacked ([`mark`]).
//! - **Merchants stop buying into a glut.** In the port they pay the gate price
//!   for everything a works makes, whether or not anyone buys it. Here they take
//!   all of it while their stock of the good is under `glut_months` of sales,
//!   less and less beyond that, and nothing more at `glut_stop_months`
//!   ([`merchants_take`]). The works sees it as short time: it hires less, so
//!   the town's wage sees it too. Works only: farms, pits and quarries still
//!   sell all they raise.
//!
//! Nothing here draws a random number, and no money changes hands that did not
//! before: a works on short time simply makes, buys and pays less.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::params::Params;
use crate::state::State;

/// A price moves by at most this much (in logs) a month, as in the port's rule.
pub const MAX_STEP: f64 = 0.12;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PriceParams {
    /// Merchants mark prices with their stock and stop buying into a glut.
    pub enabled: bool,
    /// How far the price stands off its anchor: `anchor * exp(markup * gap)`,
    /// where the gap runs from 1 (no stock) through 0 (the target stock) to -1.
    /// 0 leaves the port's rule. It steadies prices between about 0.3 and 0.8;
    /// from 1 up the markup itself over-corrects (the hands that follow a price
    /// arrive half a year late), and at 2 the island swings harder than the port's.
    pub markup: f64,
    /// Merchants take all a works makes while the town's stock of its good is
    /// under this many months of sales and shipments. 0: they always take all.
    /// It has to stand well above `price_target_months`, the stock merchants
    /// mean to hold: a brake that bites at the target chokes every trade.
    pub glut_months: f64,
    /// At this many months they take only `glut_floor` of the works' capacity.
    pub glut_stop_months: f64,
    /// The least share of a works' capacity merchants go on taking in the worst glut.
    pub glut_floor: f64,
}

impl Default for PriceParams {
    /// Off: what the port runs with.
    fn default() -> PriceParams {
        PriceParams { enabled: false, markup: 0.5, glut_months: 4.0, glut_stop_months: 8.0, glut_floor: 0.0 }
    }
}

impl PriceParams {
    /// The game's merchants.
    pub fn game() -> PriceParams {
        PriceParams { enabled: true, ..PriceParams::default() }
    }
}

/// The anchors start at the prices. Called wherever an anchor is about to be
/// read, so a state made by any road (a fresh island, a test's dump) has them
/// from its first month.
fn ensure_anchor(st: &mut State) {
    if st.price_anchor.rows != st.price.rows || st.price_anchor.cols != st.price.cols {
        st.price_anchor = st.price.clone();
    }
}

/// Next month's price of good `g` in town `t`, called from `econ::step_month`
/// in place of the port's rule.
///
/// `gap` is how far the stock is below its target (as a share of the larger of
/// the two) and `imbalance` this month's wanted against arriving goods, both as
/// the port's rule reads them; `lo_p` and `hi_p` are the floor and ceiling.
///
/// The anchor moves exactly as the port's price did. The price then goes toward
/// `anchor * exp(markup * gap)`. Read as a control loop, the port's rule is the
/// integral term alone and the markup adds the proportional one: with the stock
/// answering the price with elasticity e (supply and demand together), the
/// damping ratio is `markup / 2 * sqrt(e / (price_target_months * price_speed))`,
/// about 0.65 for iron and 1.7 for grain at a markup of 0.5, against 0.1 before.
#[allow(clippy::too_many_arguments)]
pub fn mark(st: &mut State, P: &Params, t: usize, g: usize, gap: f64, imbalance: f64, lo_p: f64, hi_p: f64) {
    ensure_anchor(st);
    let a0 = st.price_anchor[(t, g)];
    let a = (a0 * (P.price_speed * gap + P.price_flow * imbalance).clamp(-MAX_STEP, MAX_STEP).exp()).clamp(lo_p, hi_p);
    st.price_anchor[(t, g)] = a;
    let p0 = st.price[(t, g)];
    let want = a * (P.prices.markup * gap).exp();
    let newp = p0 * (want / p0).ln().clamp(-MAX_STEP, MAX_STEP).exp();
    st.price[(t, g)] = newp.clamp(lo_p, hi_p);
}

/// A new town takes its mother town's anchors along with her prices
/// (`people::found_towns`); without them it would start from a stale idea of
/// what things are worth.
///
/// And its people go on expecting what they have been getting (`pexp`, the
/// prices farms and pits hire on): the mother town's expected price less the
/// carriage from the new town's site to her, which is what a load from here
/// fetched the day before. So nobody's net price moves on the day the town is
/// founded; from then on its own market finds its own prices and what people
/// expect follows them.
///
/// The port leaves a town that is not yet founded expecting the prices of
/// 1750. That does little harm to a town founded in 1753. A town founded in
/// 1888, on an island where the machines have made grain three times cheaper,
/// expected 1750's grain price, asked for thirty-five times the hands it had
/// and paid them out of the landlords' purse: the wage doubled month after
/// month and the island's prices quadrupled in five years (seed 6 with six
/// families, found when the machines were merged; docs/rules/prices.md, "In
/// company with the machines").
pub fn found_town(st: &mut State, C: &Content, P: &Params, n: usize, donor: usize, site: usize) {
    if !P.prices.enabled {
        return;
    }
    ensure_anchor(st);
    let far = if st.dcatch[site].is_finite() { st.dcatch[site] } else { 0.0 };
    for g in 0..st.price.cols {
        st.price_anchor[(n, g)] = st.price_anchor[(donor, g)];
        let carriage = P.freight * C.weight[g] * far;
        st.pexp[(n, g)] = (st.pexp[(donor, g)] - carriage).max(P.price_floor * C.base_price[g]);
    }
}

/// The share of capacity merchants will take at a stock of `stock` against
/// smoothed monthly sales and shipments of `outflow`: 1 up to `glut_months` of
/// sales, falling in a straight line to `glut_floor` at `glut_stop_months`.
/// A stock nobody is buying from at all counts as the worst glut; no stock is
/// no glut.
pub fn take_at(pp: &PriceParams, stock: f64, outflow: f64) -> f64 {
    if stock <= 0.0 {
        return 1.0;
    }
    let months = stock / outflow.max(1e-12);
    let width = (pp.glut_stop_months - pp.glut_months).max(1e-9);
    (1.0 - (months - pp.glut_months) / width).clamp(pp.glut_floor.clamp(0.0, 1.0), 1.0)
}

/// For each works (its town in `ft`, its good in `out`): the share of its
/// capacity's output the town's merchants will take this month. All ones when
/// the mechanism is off or `glut_months` is 0.
///
/// `econ::step_month` multiplies each works' utilisation by this inside its
/// labour demand. It is read from last month's stock and sales, so within the
/// month it is a fixed number between 0 and 1 for each works: labour demand
/// still falls as the wage rises, which is what the wage's root-finder needs.
pub fn merchants_take(st: &State, P: &Params, ft: &[usize], out: &[usize]) -> Vec<f64> {
    let pp = &P.prices;
    if !pp.enabled || pp.glut_months <= 0.0 {
        return vec![1.0; ft.len()];
    }
    ft.iter().zip(out).map(|(&t, &g)| take_at(pp, st.inv[(t, g)], st.outflow[(t, g)])).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merchants_take_all_until_the_stock_is_long() {
        let pp = PriceParams::game();
        assert_eq!(take_at(&pp, 0.0, 0.0), 1.0, "no stock is no glut, even where nothing sells yet");
        assert_eq!(take_at(&pp, 300.0, 100.0), 1.0);
        assert_eq!(take_at(&pp, 400.0, 100.0), 1.0);
        assert!((take_at(&pp, 600.0, 100.0) - 0.5).abs() < 1e-12);
        assert_eq!(take_at(&pp, 800.0, 100.0), pp.glut_floor);
        assert_eq!(take_at(&pp, 5000.0, 100.0), pp.glut_floor);
        assert_eq!(take_at(&pp, 5.0, 0.0), pp.glut_floor, "a stock nobody buys from is the worst glut");
        // never rising with the stock
        let mut last = 1.0;
        for k in 0..200 {
            let v = take_at(&pp, k as f64 * 10.0, 100.0);
            assert!(v <= last && (0.0..=1.0).contains(&v));
            last = v;
        }
    }

    #[test]
    fn off_by_default_and_on_in_the_game() {
        assert!(!PriceParams::default().enabled);
        assert!(PriceParams::game().enabled);
        assert!(!crate::default_params().prices.enabled);
        assert!(crate::game_params().prices.enabled);
    }
}
