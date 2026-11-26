//! A measure with long links: a cost of living, the real wage on it and what is
//! made, each linked every `link` years. Not in the Python; it only watches.
//! `docs/rules/measures.md` is the longer account, with the test it was chosen by.
//!
//! The island has two older measures of what money buys, and after 1900 both
//! mislead. The observer's `cpi` prices the basket of 1750 for three hundred
//! years, so by the 1970s it is mostly a price of timber, whose price swings
//! fivefold every four or five years; real output, spending over it, then reads a
//! depression every time timber is dear. The chained index of `engel.rs` makes a
//! new link every year, and a link every year drifts when prices bounce and the
//! buyers answer them: a dear-timber year's basket is priced against a
//! cheap-timber year's, and back, and the two halves of the bounce do not cancel.
//!
//! Here a link holds one basket for `link` years. Its basket and its prices are
//! the means of the `window` years before the link begins, so that no link
//! begins on a dear-timber year (the means must be known when the link begins,
//! so they are the years before it, not round it). Prices are each good's price
//! in each town weighted by what was bought there (a unit value), not by people
//! as the observer's are. Three series:
//!
//! - `cpi_linked`, the cost of living: the link's basket at the month's prices
//!   over the same basket at the link's prices, times the level the link began
//!   at. 1 at the end of the warm-up, where the chained index begins too.
//! - `real_wage_linked`: the mean wage over it, 1 at the end of the warm-up.
//! - `made_linked`: every good made in the month (gross, as `out_*`) valued at
//!   the link's prices (the observer's island prices, the means of the window),
//!   in money of the warm-up's last month.
//!
//! A new link is joined to the old one over its window: the old link's mean over
//! the window is the new link's mean over it, so the series has no step there.
//! For the cost of living that is the old basket at the window's mean prices; for
//! what is made, the window's mean output at the old link's prices.
//!
//! Under the game's rules the slump clock of the news reads `made_linked`
//! (`observer::yearly`), and the facts read all three as nine-year means.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::observer::Series;
use crate::state::State;
use crate::util::pairwise_sum_from;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LinkedParams {
    /// The observer keeps the linked cost of living, the real wage on it and
    /// what is made, and the slump clock reads what is made. It never touches
    /// the economy.
    pub enabled: bool,
    /// Years a link keeps its basket. Links begin in the first recorded year
    /// and every `link` years after (1755, 1760 and on for 5). Under 1 is 1.
    pub link: i64,
    /// The years before a link begins whose means make its basket and prices.
    /// Under 1 is 1; fewer are used where fewer have been recorded.
    pub window: i64,
}

impl Default for LinkedParams {
    /// Off: what the port runs with.
    fn default() -> LinkedParams {
        LinkedParams { enabled: false, link: 5, window: 5 }
    }
}

impl LinkedParams {
    /// The game's measure.
    pub fn game() -> LinkedParams {
        LinkedParams { enabled: true, ..LinkedParams::default() }
    }
}

/// One stretch of months summed (a year, or the warm-up's last month alone):
/// what households and landlords bought of each good, each good's price
/// weighted by what was bought, the island's price as the observer has it, and
/// what was made, each summed month by month.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sums {
    pub bought: Vec<f64>,
    pub price: Vec<f64>,
    pub island_price: Vec<f64>,
    pub made: Vec<f64>,
    pub months: usize,
}

impl Sums {
    fn zeros(G: usize) -> Sums {
        Sums { bought: vec![0.0; G], price: vec![0.0; G], island_price: vec![0.0; G], made: vec![0.0; G], months: 0 }
    }

    fn add(&mut self, m: &Month) {
        for g in 0..self.bought.len() {
            self.bought[g] += m.bought[g];
            self.price[g] += m.price[g];
            self.island_price[g] += m.island_price[g];
            self.made[g] += m.made[g];
        }
        self.months += 1;
    }
}

/// One month's figures, as `Sums` keeps them.
struct Month {
    bought: Vec<f64>,
    price: Vec<f64>,
    island_price: Vec<f64>,
    made: Vec<f64>,
}

/// What the measure keeps between months (`st.linked`). The observer's alone:
/// the simulation never reads it. Empty while the rule is off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LinkedState {
    /// (G) the link's basket: the window's mean monthly purchases.
    pub basket: Vec<f64>,
    /// (G) the link's prices for the basket: the window's mean prices by what was bought.
    pub prices: Vec<f64>,
    /// The cost of living's level when the link began (the warm-up's end = 1).
    pub level: f64,
    /// (G) the link's prices for what is made: the window's mean island prices.
    pub made_prices: Vec<f64>,
    /// Money of the link's prices in money of the warm-up's (1 in the first link).
    pub made_scale: f64,
    /// Each complete year's sums, the newest last; no more than the window needs.
    pub years: Vec<Sums>,
    /// This year's sums so far.
    pub year: Sums,
}

/// The month's figures: purchases by good, prices weighted by them (the island
/// price where nothing was bought), the island prices `P_nat` and output.
fn month(st: &State, C: &Content, P_nat: &[f64]) -> Month {
    let (T, G) = (st.T_max, C.G);
    let ms = &st.month_stats;
    let mut bought = vec![0.0; G];
    let mut paid = vec![0.0; G];
    for t in 0..T {
        if !st.town_active[t] {
            continue;
        }
        for g in 0..G {
            let q = ms.hh_got[(t, g)] + ms.lord_got[(t, g)];
            bought[g] += q;
            paid[g] += q * st.price[(t, g)];
        }
    }
    let price: Vec<f64> = (0..G).map(|g| if bought[g] > 0.0 && paid[g].is_finite() { paid[g] / bought[g] } else { P_nat[g] }).collect();
    let made = ms.prod.iter().copied().take(G).collect();
    Month { bought, price, island_price: P_nat.to_vec(), made }
}

/// Called once the warm-up is over, before the first recorded month (from
/// `sim::start_from`): the first link's basket and prices are the warm-up's
/// last month, where the observer's basket of 1750 and the chained index start.
pub fn start(st: &mut State, C: &Content, P: &crate::params::Params) {
    if !P.linked.enabled {
        return;
    }
    let P_nat = crate::observer::national_prices(st);
    let m = month(st, C, &P_nat);
    st.linked = LinkedState {
        basket: m.bought,
        prices: m.price,
        level: 1.0,
        made_prices: m.island_price,
        made_scale: 1.0,
        years: Vec::new(),
        year: Sums::zeros(C.G),
    };
}

/// The rule's series, after the others so that theirs keep their places.
pub fn add_series(reg: &mut Series, P: &crate::params::Params, n_months: usize) {
    if P.linked.enabled {
        reg.add("cpi_linked", "Cost of living, linked", "prices", "1750 = 1", n_months);
        reg.add("real_wage_linked", "Real wage, linked", "people", "1750 = 1", n_months);
        reg.add("made_linked", "What is made, linked", "growth", "1750 money/month", n_months);
    }
}

/// Whether a link begins with `year`: every `link` years from the first.
pub fn begins(P: &crate::params::Params, y0: i64, year: i64) -> bool {
    year > y0 && (year - y0) % P.linked.link.max(1) == 0
}

/// Begin the next link on the window's means and join it to the one in hand.
fn new_link(l: &mut LinkedState, window: usize, base: usize) {
    let take = l.years.len().min(window);
    if take == 0 {
        return;
    }
    let G = l.basket.len();
    let w = &l.years[l.years.len() - take..];
    let months: f64 = w.iter().map(|s| s.months as f64).sum();
    let mean = |f: &dyn Fn(&Sums) -> &Vec<f64>| -> Vec<f64> { (0..G).map(|g| w.iter().map(|s| f(s)[g]).sum::<f64>() / months).collect() };
    let (bought, price, island_price, made) = (mean(&|s| &s.bought), mean(&|s| &s.price), mean(&|s| &s.island_price), mean(&|s| &s.made));
    // the old basket at the window's prices against its own: the old link's mean over the window
    let old = pairwise_sum_from(base, G, |g| l.basket[g] * price[g]) / pairwise_sum_from(base, G, |g| l.basket[g] * l.prices[g]);
    if old.is_finite() && old > 0.0 {
        l.level *= old;
    }
    // what the window made at the old link's prices, over the same at the new: the scale that
    // gives the new link the old one's mean over the window
    let scale = pairwise_sum_from(base, G, |g| l.made_prices[g] * made[g]) / pairwise_sum_from(base, G, |g| island_price[g] * made[g]);
    if scale.is_finite() && scale > 0.0 {
        l.made_scale *= scale;
    }
    l.basket = bought;
    l.prices = price;
    l.made_prices = island_price;
}

/// Called from `observer::record_month` after the observer's own series: the
/// month's three figures, and at a year's end the year put by and, where the
/// next year begins a link, the new link made.
pub fn record_month(st: &mut State, C: &Content, P: &crate::params::Params, s: &mut Series, t: usize, wage: f64, P_nat: &[f64]) {
    if !P.linked.enabled || st.linked.basket.is_empty() {
        return;
    }
    let m = month(st, C, P_nat);
    let l = &mut st.linked;
    let G = C.G;
    let base = C.sum_goods(|g| l.basket[g] * l.prices[g]);
    let cpi = l.level * C.sum_goods(|g| l.basket[g] * m.price[g]) / base;
    s.set("cpi_linked", t, cpi);
    s.set("real_wage_linked", t, wage / cpi.max(1e-9) / st.wage_base);
    s.set("made_linked", t, l.made_scale * C.sum_goods(|g| l.made_prices[g] * m.made[g]));
    l.year.add(&m);
    if l.year.months == 12 {
        let done = std::mem::replace(&mut l.year, Sums::zeros(G));
        l.years.push(done);
        let window = P.linked.window.max(1) as usize;
        if l.years.len() > window {
            l.years.remove(0);
        }
        // the year that has just ended is year0 + t / 12; the next begins a link?
        let next = C.start_year + (t / 12) as i64 + 1;
        if begins(P, C.start_year, next) {
            new_link(l, window, C.G_base);
        }
    }
}

/// The years the facts are read at, those inside the run, and the run's last
/// year beside them.
pub const MARKS: [i64; 6] = [1850, 1900, 1950, 1970, 2000, 2050];

/// The years a fact at `y` is the mean of: the nine years round it, or for the
/// run's last year the last nine, each kept inside the run (`y0` to `y1`).
pub fn nine_years(y0: i64, y1: i64, y: i64) -> (i64, i64) {
    if y >= y1 { ((y1 - 8).max(y0), y1) } else { ((y - 4).max(y0), (y + 4).min(y1)) }
}

/// The facts (docs/rules/measures.md): `real_wage_linked_Y_vs_1750` and
/// `made_linked_pc_Y_vs_1750`, what is made a head, each the mean of the nine
/// years round Y (`nine_years`) over the first year's, at `MARKS` and the run's
/// last year. Nothing in a run that did not keep them.
pub fn facts(r: &crate::sim::RunResult) -> Vec<(String, crate::facts::Kind, f64)> {
    use crate::facts::Kind;
    let (st, C) = (&r.state, &r.content);
    let mut out = Vec::new();
    if !st.series.index.contains_key("real_wage_linked") {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    let yearly = |name: &str| -> Vec<f64> { (y0..=y1).map(|y| crate::observer::annual(st.series.get(name), y0, y)).collect() };
    let (wage, made, pop) = (yearly("real_wage_linked"), yearly("made_linked"), yearly("pop"));
    let pc: Vec<f64> = made.iter().zip(&pop).map(|(m, p)| m / p).collect();
    let mut marks: Vec<i64> = MARKS.iter().copied().filter(|&y| y > y0 && y <= y1).collect();
    if y1 > y0 && !marks.contains(&y1) {
        marks.push(y1);
    }
    let mean = |v: &[f64], (a, z): (i64, i64)| -> f64 {
        let xs: Vec<f64> = (a..=z).map(|y| v[(y - y0) as usize]).filter(|x| x.is_finite()).collect();
        if xs.is_empty() { f64::NAN } else { xs.iter().sum::<f64>() / xs.len() as f64 }
    };
    for (label, v) in [("real_wage_linked", &wage), ("made_linked_pc", &pc)] {
        for &y in &marks {
            out.push((format!("{label}_{y}_vs_{y0}"), Kind::Ratio, mean(v, nine_years(y0, y1, y)) / v[0]));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sums(bought: &[f64], price: &[f64], made: &[f64]) -> Sums {
        // twelve months alike
        let k = |v: &[f64]| v.iter().map(|x| 12.0 * x).collect::<Vec<_>>();
        Sums { bought: k(bought), price: k(price), island_price: k(price), made: k(made), months: 12 }
    }

    #[test]
    fn a_new_link_has_the_old_ones_mean_over_its_window() {
        // two goods; the old link bought 10 and 2 at prices 1 and 4
        let mut l = LinkedState {
            basket: vec![10.0, 2.0],
            prices: vec![1.0, 4.0],
            level: 1.5,
            made_prices: vec![1.0, 4.0],
            made_scale: 2.0,
            years: vec![sums(&[9.0, 3.0], &[2.0, 4.0], &[5.0, 1.0]), sums(&[7.0, 3.0], &[2.0, 8.0], &[5.0, 3.0])],
            year: Sums::zeros(2),
        };
        // the old cost of living over the window: mean prices (2, 6), old basket 10*2 + 2*6 = 32 against 18
        let old_level = 1.5 * 32.0 / 18.0;
        // what was made over the window at the old prices: mean (5, 2): 5 + 8 = 13, times 2
        let old_made = 2.0 * 13.0;
        new_link(&mut l, 5, 2);
        assert!((l.level - old_level).abs() < 1e-12);
        assert_eq!(l.basket, vec![8.0, 3.0]);
        assert_eq!(l.prices, vec![2.0, 6.0]);
        // the new link at the window's mean prices is the level it began at
        let new_cpi = l.level * (8.0 * 2.0 + 3.0 * 6.0) / (8.0 * 2.0 + 3.0 * 6.0);
        assert!((new_cpi - old_level).abs() < 1e-12);
        // and what the window made, at the new link's prices, is what it made at the old
        let new_made = l.made_scale * (2.0 * 5.0 + 6.0 * 2.0);
        assert!((new_made - old_made).abs() < 1e-12, "{new_made} against {old_made}");
        // a window of one year takes the last year only
        let mut one = LinkedState { basket: vec![1.0, 1.0], prices: vec![1.0, 1.0], level: 1.0, made_prices: vec![1.0, 1.0], made_scale: 1.0, years: l.years.clone(), year: Sums::zeros(2) };
        new_link(&mut one, 1, 2);
        assert_eq!(one.basket, vec![7.0, 3.0]);
        assert_eq!(one.prices, vec![2.0, 8.0]);
    }

    #[test]
    fn links_begin_every_link_years_from_the_first() {
        let mut P = crate::params::game_params();
        P.linked.link = 10;
        let at: Vec<i64> = (1750..=1800).filter(|&y| begins(&P, 1750, y)).collect();
        assert_eq!(at, vec![1760, 1770, 1780, 1790, 1800]);
        P.linked.link = 0;
        assert!(begins(&P, 1750, 1751));
    }
}
