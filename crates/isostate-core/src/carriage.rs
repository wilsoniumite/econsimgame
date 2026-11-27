//! Carriage in hands: a load takes hands, and they are paid a wage. Not in the
//! Python (`docs/rules/carriage.md`).
//!
//! In the port carrying a unit of weight over a unit of the path's cost (a cell
//! of open lowland with no road) costs `freight`, 0.015 coins, whatever the
//! year. Under the game's rules the settled island keeps that fixed coin until
//! 1900 and then lets it ride on the cost of living (`longrun.freight_from`);
//! landfall rides on the cost of living from 1750 (`landfall::FREIGHT_FROM`).
//! Neither is what carriage is: hands and a cart for a day, paid what hands
//! are paid.
//!
//! With `carriage.in_hands` a load over a unit of path takes `hands` hands for
//! a month, the same number on both starts (0.015 coins over the settled
//! island's mean wage in 1750, 4.49 coins a month), and the bill is those hands
//! at a wage. Better ways lower the hands as they always lowered the bill: the
//! road, the canal, the train, the sea and the lorry's cheaper squares are in
//! the path's cost (`transport.rs`), and so in the number of units a load is
//! carried over. Whose wage is the one setting, `carriage.wage`:
//!
//! - `town`: a reckoning made for one town's carting (a works' or a farm's gate
//!   price, the land's bids and its delivered prices, the trade a town sends,
//!   the founding test of a district, the prices a district takes along) reads
//!   that town's wage of the month (`freight_at`). The town's carters are then
//!   its bill over its wage (`carting::take_carters`), which is the hands the
//!   loads took, to the bit, until the lorry is found.
//! - `island`: every reckoning reads the island's mean wage, weighted by people,
//!   of the last year (`longrun`'s own sum).
//!
//! Reckonings made over the island, not for one town, read the island's wage
//! under either setting: the promoters of roads, canals, railways, bridges and
//! trunk roads, the port's own railway planner, and the year's book of fares.
//!
//! With the switch on, `longrun.freight_from` and landfall's stopgap are not
//! read, and the railway's station charge (`haulage.handling`) is laid from the
//! first year, since it is only ever laid on a train's edges
//! (`haulage::on`). Off, every function here returns what `longrun::freight`
//! returns, to the bit, and nothing else is read.

use serde::{Deserialize, Serialize};

use crate::params::Params;
use crate::state::State;
use crate::util::pairwise_sum_by;

/// The hands a unit of weight takes for a month over a unit of the path's
/// cost: the port's freight, 0.015 coins, over the settled island's mean wage
/// in 1750 (4.49 coins a month, islands 1 to 8, `annual.csv`), so that the
/// settled island's first year carts at about the port's coins.
pub const HANDS: f64 = 0.015 / 4.49;

/// Whose wage a load's hands are paid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Wage {
    /// The carting town's own wage of the month, where the reckoning is for one town.
    Town,
    /// The island's mean wage of the last year, weighted by people.
    Island,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CarriageParams {
    /// A load takes `hands` and they are paid a wage; `freight` and the
    /// cost of living are not read.
    pub in_hands: bool,
    /// Hands for a month to carry a unit of weight over a unit of the path's cost.
    pub hands: f64,
    /// Whose wage they are paid.
    pub wage: Wage,
}

impl Default for CarriageParams {
    /// Off, under every set of rules for now.
    fn default() -> CarriageParams {
        CarriageParams { in_hands: false, hands: HANDS, wage: Wage::Town }
    }
}

/// The island's mean wage, weighted by people: the last closed year's, as
/// `longrun` sums it, while that mechanism keeps one (1750's first year reads
/// the warm-up's last month); otherwise this month's.
pub fn island_wage(st: &State) -> f64 {
    let lr = &st.longrun;
    if let Some(&w) = lr.wages.last() {
        if lr.wage0 > 0.0 && w > 0.0 {
            return w * lr.wage0;
        }
    }
    let T = st.T_max.min(st.wage.len()).min(st.pop.len());
    let pop = |i: usize| if st.town_active[i] { st.pop[i] } else { 0.0 };
    let people = pairwise_sum_by(T, pop);
    pairwise_sum_by(T, |i| st.wage[i] * pop(i)) / people.max(1.0)
}

/// The island's carriage, a unit of weight over a unit of path, under
/// `carriage.in_hands` (`longrun::freight` reads it first).
pub fn island(st: &State, P: &Params) -> f64 {
    P.carriage.hands * island_wage(st)
}

/// What it costs town `t` to cart a unit of weight over a unit of path: under
/// `carriage.in_hands` with the town's wage, its hands at `t`'s wage of the
/// month (the island's where `t` has none); otherwise `longrun::freight`.
pub fn freight_at(st: &State, P: &Params, t: usize) -> f64 {
    if P.carriage.in_hands && P.carriage.wage == Wage::Town {
        if let Some(&w) = st.wage.get(t) {
            if w > 0.0 && w.is_finite() && st.town_active.get(t).copied().unwrap_or(false) {
                return P.carriage.hands * w;
            }
        }
    }
    crate::longrun::freight(st, P)
}
