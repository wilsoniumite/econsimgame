//! PACE: works whose plant comes in over a year (not in the Python; see
//! `docs/rules/pace.md` and `docs/probes/tr-timber.md`).
//!
//! Under the port a works that orders plant asks the merchants each month for
//! a quarter of what is still to come (`econ::step_month`, the orders for
//! capacity), so most of a mill's timber and bricks are asked for in the first
//! three or four months after the year's review. After 1900 the households
//! have left timber and bricks, and the works' orders for plant are nearly all
//! of both markets; a year's orders for a mill trade can be a whole year of the
//! woods' cut, asked for in a season. That is the timber-and-brick cycle the
//! trace found: timber's and bricks' prices swing fivefold every four or five
//! years after 1900.
//!
//! With `enabled`, from the year `from` on, a works asks each month for
//! `1 / months` of what is still to come: with `months` 12 half the plant is
//! in by the eighth month, a third is still to come after a year, and the
//! mean wait is a year, where the port's is four months. Each year's orders
//! are spread over the next two or three, so the waves of the four-to-five-year
//! cycle overlap and the merchants never see a whole year's orders in a season.
//! (The check found that it is the year's mean wait that cures: plant brought
//! in evenly within one year does little for timber, and evenly over two does
//! what this does; docs/probes/tr-timber.md, "The check".) Nothing else is
//! touched: the size of each order, the year's room, the review, the price rule
//! and the woods are as they were.
//!
//! Off is off: the share asked for is the port's quarter, and the run is the
//! run to the bit. On under the game's rules since its check (26 November
//! 2026), from 1901, so the played century is untouched.

use serde::{Deserialize, Serialize};

use crate::params::Params;
use crate::state::State;

/// The share of what is still to come that a works asks for in a month under
/// the port.
pub const PORT_SHARE: f64 = 0.25;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PaceParams {
    /// Works ask for their plant over `months` from the year `from` on.
    pub enabled: bool,
    /// Over how many months a works asks for its plant: each month it asks for
    /// one part in `months` of what is still to come.
    pub months: f64,
    /// The first year whose months are paced. The year's orders placed at the
    /// end of the year before are paced from its January.
    pub from: i64,
}

impl Default for PaceParams {
    /// Off: what the port runs with.
    fn default() -> PaceParams {
        PaceParams { enabled: false, months: 12.0, from: 1901 }
    }
}

impl PaceParams {
    /// The rule switched on with its numbers, as `game_params` has it.
    pub fn on() -> PaceParams {
        PaceParams { enabled: true, ..PaceParams::default() }
    }
}

/// The share of what is still to come of its plant that a works asks the
/// merchants for this month: the port's quarter, or one part in `months`
/// once the rule acts.
pub fn share(st: &State, P: &Params) -> f64 {
    let p = &P.pace;
    if p.enabled && st.engel.year_now >= p.from && p.months >= 1.0 { 1.0 / p.months } else { PORT_SHARE }
}
