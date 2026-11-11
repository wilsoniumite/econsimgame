//! Carting takes labour. Not in the Python (`docs/rules/carting.md`).
//!
//! In the port every unit of weight moved costs money, and that money is paid to
//! the town's households as income, but nobody's time goes into it: the carters
//! are paid and do not exist. So a railway that cuts the cost of carriage by
//! three quarters moves money from one pocket to another and leaves output per
//! head where it was.
//!
//! With `carting.enabled` the month's carting bill of each town (its cells'
//! produce to market, its works' inputs and output, the trade it sends out) is
//! the wages of carters at the town's wage, and those hands are not there to be
//! hired by farms and works the month after. Nothing else changes: the money
//! still goes to households, who are now the carters. A better road, a canal or
//! a railway lowers the bill and gives the hands back.

use serde::{Deserialize, Serialize};

use crate::facts::Kind;
use crate::observer::annual;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CartingParams {
    /// Carters come out of each town's labour supply.
    pub enabled: bool,
    /// The share of the carting bill that is carters' wages. 1 makes every penny
    /// of it a man's time; 0 measures the bill and changes nothing.
    pub labor: f64,
    /// At most this share of a town's hands can be on the road.
    pub max_share: f64,
}

impl Default for CartingParams {
    /// Off: what the port runs with.
    fn default() -> CartingParams {
        CartingParams { enabled: false, labor: 1.0, max_share: 0.5 }
    }
}

impl CartingParams {
    /// The game's carting.
    pub fn game() -> CartingParams {
        CartingParams { enabled: true, ..CartingParams::default() }
    }
}

/// The start of a month's labour market: last month's carting bill, at last
/// month's wage, is this month's carters, and they leave the town's supply of
/// hands. (A month's lag, because the bill is only known once the month's goods
/// have moved.)
pub fn take_carters(st: &mut State, P: &Params, supply: &mut [f64]) {
    if !P.carting.enabled {
        return;
    }
    let T = supply.len();
    if st.cart_bill.len() != T {
        st.cart_bill = vec![0.0; T];
    }
    let mut carters = vec![0.0; T];
    for t in 0..T {
        if st.town_active[t] && st.wage[t] > 0.0 {
            carters[t] = (P.carting.labor * st.cart_bill[t] / st.wage[t]).min(P.carting.max_share * supply[t]);
            supply[t] -= carters[t];
        }
    }
    // the year's totals, for the facts
    if st.cart_months % 12 == 0 {
        st.cart_years.push((0.0, 0.0));
    }
    st.cart_months += 1;
    let year = st.cart_years.last_mut().expect("pushed above");
    year.0 += st.cart_bill.iter().sum::<f64>();
    year.1 += carters.iter().sum::<f64>();
    st.carters = carters;
    st.cart_bill.iter_mut().for_each(|v| *v = 0.0);
}

/// Add carting paid this month to the bill of each payer's town.
pub fn add_bill(st: &mut State, P: &Params, town: &[usize], paid: impl Iterator<Item = f64>) {
    if !P.carting.enabled {
        return;
    }
    for (&t, v) in town.iter().zip(paid) {
        st.cart_bill[t] += v;
    }
}

/// Add carting paid this month to one town's bill.
pub fn add_bill_town(st: &mut State, P: &Params, t: usize, paid: f64) {
    if P.carting.enabled {
        st.cart_bill[t] += paid;
    }
}

/// Carters are at work: count them in each town's employment, so the shares of
/// hands on the land and in industry are shares of everyone working.
pub fn count_carters(st: &State, P: &Params, employment: &mut [f64]) {
    if !P.carting.enabled {
        return;
    }
    for (e, c) in employment.iter_mut().zip(&st.carters) {
        *e += c;
    }
}

/// Facts for the sweep's table, under "Work and wages".
///
/// Two of them measure the standard of living in units that do not depend on a
/// run's own 1750 prices, so that runs whose 1750 differs (carting on and off)
/// can be set side by side: `baskets_per_head_Y`, the households' monthly budget
/// for goods over the cost of the subsistence basket, averaged over people; and
/// `goods_per_head_Y`, everything made in a month at the content's base prices,
/// per head (gross: iron is counted and so are the tools made of it).
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C, rec, P) = (&r.state, &r.content, &r.rec, &r.params);
    let y0 = C.start_year;
    let y1 = y0 + rec.n_years as i64 - 1;
    let series = |name: &str| st.series.index.get(name).map_or(&[][..], |&k| &st.series.data[k][..]);
    let at = |name: &str, y: i64| annual(series(name), y0, y);
    let marks = crate::facts::marks(y0, y1);
    let mut out: Vec<(String, Kind, f64)> = Vec::new();
    let T = rec.T;
    for &y in &marks {
        let k = (y - y0) as usize;
        let (mut num, mut den) = (0.0, 0.0);
        for t in 0..T {
            if rec.town_active[k * T + t] > 0 {
                num += rec.town_y[k * T + t] as f64 * rec.town_pop[k * T + t] as f64;
                den += rec.town_pop[k * T + t] as f64;
            }
        }
        out.push((format!("baskets_per_head_{y}"), Kind::Real, if den > 0.0 { num / den } else { f64::NAN }));
    }
    for &y in &marks {
        let goods: f64 = (0..C.G).map(|g| at(&format!("out_{}", C.good_name[g]), y) * C.base_price[g]).sum();
        out.push((format!("goods_per_head_{y}"), Kind::Real, goods / at("pop", y)));
    }
    // the carting bill over output, and carters over all hands
    let warm = P.warmup_years.max(0) as usize;
    for &y in &marks {
        let (bill, hands) = match st.cart_years.get(warm + (y - y0) as usize) {
            Some(&(bill, carters)) => (bill / (12.0 * at("gdp_nom", y)), carters / 12.0 / (P.participation * at("pop", y))),
            None => (f64::NAN, f64::NAN),
        };
        out.push((format!("carting_share_of_output_{y}"), Kind::Share, bill));
        out.push((format!("carters_share_of_hands_{y}"), Kind::Share, hands));
    }
    out
}
