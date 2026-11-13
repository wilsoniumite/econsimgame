//! New money issued on the island's true total (not in the Python;
//! `docs/rules/money.md`).
//!
//! Each month the port issues `money_growth / 12` times the money it counts,
//! and counts it at a moment when the households' whole income for the month
//! (their pay, their carting and both kinds of dividend, 11.5 to 14.7 per cent
//! of the money) is held in a local vector in `econ::step_month` and counted
//! nowhere. So the money grows 1.74 to 1.78 per cent a year where the setting
//! says 2, the shortfall wanders with what the merchants owe, and a month whose
//! incomes are larger than the money issues less than nothing
//! (`docs/probes/tr-money.md`).
//!
//! With `money.true_total` the month's issue is `money.growth / 12` times the
//! money as it stood when the month began, which is the whole of it, since
//! money only moves between purses inside a month. The port's `money_growth`
//! is then not read. `money.growth` is 0.0175, which issued monthly is 1.764
//! per cent a year: the rate today's islands really had, so that the island
//! the other mechanisms were tuned on keeps the money it had, now on every
//! island alike and in every month (`docs/designs/rules-pass.md`, 4.3).

use serde::{Deserialize, Serialize};

use crate::params::Params;
use crate::state::State;

/// The issue of new money.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MoneyParams {
    /// The month's issue is reckoned on all the money there is when the month
    /// begins, at `growth`. Off (the port): on the money counted while the
    /// month's incomes are between purses, at `Params::money_growth`.
    pub true_total: bool,
    /// New money a year as a share of the money, issued a twelfth each month.
    pub growth: f64,
}

impl Default for MoneyParams {
    /// Off: what the port runs with.
    fn default() -> MoneyParams {
        MoneyParams { true_total: false, growth: 0.0175 }
    }
}

impl MoneyParams {
    /// The game's issue: on the true total, at the rate the island has had.
    pub fn game() -> MoneyParams {
        MoneyParams { true_total: true, ..MoneyParams::default() }
    }
}

/// The yearly rate the issue is reckoned at: `money.growth` under the rule,
/// the port's `money_growth` without it.
pub fn rate(P: &Params) -> f64 {
    if P.money.true_total { P.money.growth } else { P.money_growth }
}

/// The money the month's issue is reckoned on, taken as the month begins:
/// `econ::money_total` then, while nothing is between purses. `None` without
/// the rule, which counts later in the month (`econ::step_month`, "dividends,
/// common fund, issuance"), and nothing is added up here.
pub fn month_base(st: &State, P: &Params) -> Option<f64> {
    if P.money.true_total { Some(crate::econ::money_total(st)) } else { None }
}
