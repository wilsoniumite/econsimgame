//! The common fund that carries its debt (not in the Python;
//! `docs/rules/mines.md`, "Under the commons").
//!
//! Under the commons scenario (`RentRegime::Commons`) the rent of land nobody
//! owns goes into a common fund, and each month the fund shares out nine
//! tenths of what it holds, by head, to every household on the island. The
//! port shares out nine tenths whatever the sign, so a fund in debt takes money
//! from every household by head, whether the household has it or not. A farm's
//! bad month is small beside the island's rents and the port's fund is hardly
//! ever in debt. A working's can be large: a town founded on a coalfield has
//! asked for over three times its hands at a wage that doubled every month,
//! the fund owed three times the island's money, and the levy emptied every
//! purse on the island.
//!
//! With `commons.carry_debt` the fund carries the debt instead, as a town's
//! landlords do under the other regime: what it has to share out is what it
//! holds and never less than nothing, so a fund in debt pays no dividend until
//! its rents have made the debt good, and asks nobody for anything. Under the
//! landlords the fund never holds a penny and the switch does nothing.
//!
//! The rule came in with the mines, which were the first to put the fund deep
//! in debt, and hung on `mines.enabled` until it was given this switch of its
//! own: the levy is a hazard of the commons, not of the mines, and the port's
//! own mines with the machines switched on run into it too.

use serde::{Deserialize, Serialize};

use crate::params::Params;
use crate::state::State;

/// The common fund's rule.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CommonsParams {
    /// A fund in debt pays no dividend until its rents repay it, and charges
    /// nobody. Off (the port): it shares out nine tenths of its debt as a levy.
    pub carry_debt: bool,
}

impl CommonsParams {
    /// The game's fund, which carries its debt.
    pub fn game() -> CommonsParams {
        CommonsParams { carry_debt: true }
    }
}

/// What the fund has to share out this month, of which `econ::step_month`
/// pays nine tenths: what it holds, and with `carry_debt` never less than
/// nothing.
pub fn fund_to_share(st: &State, P: &Params) -> f64 {
    if P.commons.carry_debt { st.fund_cash.max(0.0) } else { st.fund_cash }
}
