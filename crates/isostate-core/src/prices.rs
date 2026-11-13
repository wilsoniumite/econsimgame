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

/// Read through [`PriceSettings`], so that a setting that makes no sense on its
/// own is refused as it is read, by every road (a file, an override, the
/// sweep's `--set`, a params.json read back).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PriceSettings")]
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
    pub glut_months: f64,
    /// At this many months they take only `GLUT_FLOOR` of the works' capacity.
    /// It has to stand above [`brake_floor`], which two numbers of the port's
    /// set, and a setting that does not is refused where it is read ([`check`]).
    pub glut_stop_months: f64,
}

impl Default for PriceParams {
    /// Off: what the port runs with.
    fn default() -> PriceParams {
        PriceParams { enabled: false, markup: 0.5, glut_months: 4.0, glut_stop_months: 8.0 }
    }
}

/// The least share of a works' capacity merchants go on taking in the worst
/// glut: nothing. A setting, `glut_floor`, until stage 5 of the pass over the
/// rules: at 0.1, 0.25 and 0.5 it moved nothing beyond chance
/// (docs/probes/sens-prices.md).
pub const GLUT_FLOOR: f64 = 0.0;

impl PriceParams {
    /// The game's merchants.
    pub fn game() -> PriceParams {
        PriceParams { enabled: true, ..PriceParams::default() }
    }
}

/// [`PriceParams`] as written in a file, before it is checked. The same fields;
/// what is left out takes the default.
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PriceSettings {
    enabled: bool,
    markup: f64,
    glut_months: f64,
    glut_stop_months: f64,
}

impl Default for PriceSettings {
    fn default() -> PriceSettings {
        let PriceParams { enabled, markup, glut_months, glut_stop_months } = PriceParams::default();
        PriceSettings { enabled, markup, glut_months, glut_stop_months }
    }
}

impl TryFrom<PriceSettings> for PriceParams {
    type Error = String;

    /// Refuse a setting that makes no sense whatever else is set, in one
    /// sentence that names it. Whether the switch is on does not matter: a
    /// file that says a nonsense number has a mistake in it either way.
    fn try_from(s: PriceSettings) -> Result<PriceParams, String> {
        let PriceSettings { enabled, markup, glut_months, glut_stop_months } = s;
        for (name, x) in [("markup", markup), ("glut_months", glut_months), ("glut_stop_months", glut_stop_months)] {
            if !x.is_finite() {
                return Err(format!("prices.{name} is {x}, and it has to be an ordinary number"));
            }
        }
        if markup < 0.0 {
            return Err(format!("prices.markup is {markup}, and a negative markup asks more as the shelves fill, which wrecks the island: give 0 (the port's rule) or more, 0.3 to 0.8 being where it steadies prices"));
        }
        if glut_months < 0.0 {
            return Err(format!("prices.glut_months is {glut_months}, and it has to be a number of months of stock, or 0 for merchants who always take all"));
        }
        if glut_months > 0.0 && glut_stop_months < glut_months {
            return Err(format!("prices.glut_stop_months is {glut_stop_months}, under glut_months ({glut_months}), and merchants cannot stop taking before they start to take less: give it glut_months or more (the same is a brake that drops from all to nothing at once)"));
        }
        Ok(PriceParams { enabled, markup, glut_months, glut_stop_months })
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

/// A price that something other than the market's own rule has just set takes
/// its anchor with it. `people` does this once a year: a district that changes
/// market brings the prices it was used to (`peopling::move_with_cells`), and
/// town `t`'s price of good `g`, which was `was`, becomes an average of its own
/// and the newcomers'. The anchor moves in the same proportion, so the price
/// stands as far above or below it as it did. That distance is the merchants'
/// stock speaking, and the stock is what it was (a new town's is its share of
/// its mother town's, as many months of sales as hers).
///
/// Without this the new price would be walked back toward the old anchor at 12
/// per cent a month: for a new town, toward what the mother town thought the
/// good was worth, which is the mistake the average was made to avoid.
pub fn anchor_follows(st: &mut State, C: &Content, P: &Params, t: usize, g: usize, was: f64) {
    if !P.prices.enabled || !(was > 0.0) {
        return;
    }
    ensure_anchor(st);
    let (lo, hi) = (P.price_floor * C.base_price[g], P.price_ceiling * C.base_price[g]);
    st.price_anchor[(t, g)] = (st.price_anchor[(t, g)] * st.price[(t, g)] / was).clamp(lo, hi);
}

/// The stock, in months of sales and shipments, at or under which the brake on
/// a glut must not yet be fully on. It leans on two numbers the brake does not
/// own. Merchants mean to hold `price_target_months` of a good (2), so a brake
/// that is full on by then falls on every trade: from 0.5 to 0.6 months every
/// works is on short time for ever (docs/rules/prices.md). And a town may ship
/// at most `trade_cap` of its stock a month (0.3), so a town that only ships a
/// good never holds less than `1 / trade_cap` months of it (3.3), and a brake
/// full on by then falls on every exporter. The brake may start below the floor,
/// as long as it is still easing off there: from 0.5 to 40 months, and from 2
/// to 8, measure as well as the game's 4 to 8 (docs/rules/prices.md,
/// docs/probes/sens-prices.md). Whoever moves either number moves this floor
/// with it, and [`check`] refuses a `glut_stop_months` left at or under it.
pub fn brake_floor(P: &Params) -> f64 {
    let ships = if P.trade_cap > 0.0 { 1.0 / P.trade_cap } else { 0.0 };
    P.price_target_months.max(ships)
}

/// Settings of the merchants' prices that cannot be run with the rest of the
/// parameters, refused in one sentence that names the setting. Called by
/// `Params::check` wherever parameters are read from a file or an override; a
/// `Params` built in code is not checked, so a test may still try a brake that
/// bites where the game's never would.
pub fn check(P: &Params) -> Result<(), String> {
    let pp = &P.prices;
    if !pp.enabled || pp.glut_months == 0.0 {
        return Ok(());
    }
    let floor = brake_floor(P);
    if !(pp.glut_stop_months > floor) {
        return Err(format!(
            "prices.glut_stop_months is {}, and the brake on a glut must still be easing off above {} months of stock, the larger of price_target_months ({}) and the stock a town that only ships holds at a trade_cap of {} (1 / trade_cap): raise glut_stop_months with them",
            pp.glut_stop_months,
            short(floor),
            P.price_target_months,
            P.trade_cap
        ));
    }
    Ok(())
}

/// A number worked out, as a person would write it: at most two decimals, none
/// trailing. (Settings are written as they were given.)
fn short(x: f64) -> String {
    let s = format!("{x:.2}");
    if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s }
}

/// The share of capacity merchants will take at a stock of `stock` against
/// smoothed monthly sales and shipments of `outflow`: 1 up to `glut_months` of
/// sales, falling in a straight line to `GLUT_FLOOR` at `glut_stop_months`.
/// A stock nobody is buying from at all counts as the worst glut; no stock is
/// no glut.
pub fn take_at(pp: &PriceParams, stock: f64, outflow: f64) -> f64 {
    if stock <= 0.0 {
        return 1.0;
    }
    let months = stock / outflow.max(1e-12);
    let width = (pp.glut_stop_months - pp.glut_months).max(1e-9);
    (1.0 - (months - pp.glut_months) / width).clamp(GLUT_FLOOR, 1.0)
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
        assert_eq!(take_at(&pp, 800.0, 100.0), GLUT_FLOOR);
        assert_eq!(take_at(&pp, 5000.0, 100.0), GLUT_FLOOR);
        assert_eq!(take_at(&pp, 5.0, 0.0), GLUT_FLOOR, "a stock nobody buys from is the worst glut");
        // never rising with the stock
        let mut last = 1.0;
        for k in 0..200 {
            let v = take_at(&pp, k as f64 * 10.0, 100.0);
            assert!(v <= last && (0.0..=1.0).contains(&v));
            last = v;
        }
    }

    #[test]
    fn an_anchor_follows_a_price_set_from_outside() {
        let C = crate::default_content();
        let P = crate::game_params();
        let base = C.base_price[0];
        let mut st = State { price: crate::mat::Mat::filled(2, C.G, base), price_anchor: crate::mat::Mat::filled(2, C.G, 1.25 * base), ..State::default() };
        // town 1's price of good 0 is set a fifth lower by a district that came over
        st.price[(1, 0)] = 0.8 * base;
        anchor_follows(&mut st, &C, &P, 1, 0, base);
        assert!((st.price_anchor[(1, 0)] - base).abs() < 1e-12, "the anchor went a fifth lower with it");
        assert!((st.price[(1, 0)] / st.price_anchor[(1, 0)] - 0.8).abs() < 1e-12, "and the price stands off it as it did");
        assert_eq!(st.price_anchor[(0, 0)], 1.25 * base, "no other town's moved");
        assert_eq!(st.price_anchor[(1, 1)], 1.25 * base, "nor any other good's");
        // never past the floor and ceiling prices keep to
        st.price[(1, 0)] = P.price_ceiling * base;
        anchor_follows(&mut st, &C, &P, 1, 0, 0.8 * base);
        assert_eq!(st.price_anchor[(1, 0)], P.price_ceiling * base);
        // and with the merchants' prices off there is no anchor to move
        let mut off = State { price: crate::mat::Mat::filled(2, C.G, base), ..State::default() };
        anchor_follows(&mut off, &C, &crate::default_params(), 1, 0, 2.0 * base);
        assert_eq!(off.price_anchor.rows, 0);
    }

    /// Nonsense settings are refused as they are read, each in one sentence
    /// naming it; sense, the edges included, is read as written.
    #[test]
    fn nonsense_settings_are_refused_where_they_are_read() {
        let read = |text: &str| crate::game_params().with_toml_overrides(text);
        for (text, name) in [
            ("prices = { markup = -0.5 }", "prices.markup is -0.5"),
            ("prices = { glut_months = -1.0 }", "prices.glut_months is -1"),
            ("prices = { glut_stop_months = 3.0 }", "prices.glut_stop_months is 3"),
        ] {
            let err = read(text).expect_err(text);
            assert!(err.contains(name), "{text}: {err}");
            assert_eq!(err.matches(". ").count(), 0, "{text}: more than one sentence: {err}");
        }
        // nonsense is nonsense with the switch off too, and a number that is
        // not one (TOML can write it; it reaches the struct only when TOML is read
        // straight into the parameters, as here: a --params file goes by way of
        // JSON, which has no such numbers, and is refused there as "invalid type: null")
        assert!(crate::default_params().with_json_overrides(r#"{"prices": {"markup": -1}}"#).is_err());
        let err = crate::Params::from_toml_str("[prices]\nmarkup = nan\n").expect_err("a markup that is not a number");
        assert!(err.contains("prices.markup is NaN, and it has to be an ordinary number"), "{err}");
        let err = crate::Params::from_toml_str("[prices]\nglut_stop_months = inf\n").expect_err("a brake that never stops");
        assert!(err.contains("prices.glut_stop_months is inf, and it has to be an ordinary number"), "{err}");
        // the sweep's own reader meets the same refusal
        let err = crate::sweep::set_field(&crate::game_params(), "prices.markup", &serde_json::json!(-1.0)).expect_err("a negative markup by --set");
        assert!(err.contains("prices.markup is -1"), "{err}");
        // the edges are sense: no markup, a brake that is off, a cliff at the start
        for text in ["prices = { markup = 0.0 }", "prices = { glut_months = 0.0, glut_stop_months = 0.0 }", "prices = { glut_stop_months = 4.0 }"] {
            read(text).unwrap_or_else(|e| panic!("{text}: {e}"));
        }
        // and what is written is what is read, with the rest the game's
        let p = read("prices = { markup = 0.8 }").unwrap().prices;
        assert_eq!(p, PriceParams { markup: 0.8, ..PriceParams::game() });
        // the port's and the game's numbers go out and come back unchanged
        for P in [crate::default_params(), crate::game_params()] {
            let back: crate::Params = serde_json::from_value(serde_json::to_value(&P).unwrap()).unwrap();
            assert_eq!(back.prices, P.prices);
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
