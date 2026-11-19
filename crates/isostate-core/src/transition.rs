//! Fewer children as a town's own people live better, and fewer deaths from
//! crowding once drains and clean water come with income. Not in the Python
//! (`docs/rules/transition.md`).
//!
//! Under the game's people (`peopling::growth`) a town's rate of increase is
//! what its income gives, less what crowding costs, less dearth: one resting
//! point, where food runs short. An island that grows richer only grows
//! larger.
//!
//! With `transition.enabled`, once a year and town by town, with `y` the
//! town's baskets a head:
//!
//! - `z` is the share of its people living on town ground times `y`: how well
//!   its townsfolk live, where a child costs more than it earns;
//! - `zbar` follows `z` slowly, a generation's lag: `zbar += (z - zbar) /
//!   lag_years` (a town's first year sets it to `z`);
//! - the income term is multiplied by `F = floor + (1 - floor) / (1 + (zbar /
//!   z_half)^z_power)`, which is 1 on a poor island, a half at `z_half` and
//!   near `floor` well past it;
//! - the crowding term is multiplied by `1 - S`, with `S = clamp((y -
//!   clean_from) / clean_span, 0, 1)`: drains and clean water, which come with
//!   income.
//!
//! Dearth and sailings are as they were. `zbar` goes where people go, as their
//! cash does: with migrants (`people::migration`), with the districts a town
//! takes (`peopling::move_with_cells`), and to a new town from its donor
//! (`peopling::open_town`). It needs the game's people (`people.enabled`):
//! the port's capped rule never reads it.
//!
//! With the switch off nothing here runs and `peopling::growth` takes its old
//! branch, so a run is the same to the byte.

use serde::{Deserialize, Serialize};

use crate::facts::Kind;
use crate::mat::Mat;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;

/// `zbar` of a town that has not yet had a year under the rule.
const UNSET: f64 = -1.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TransitionParams {
    /// Births answer to how well a town's townsfolk live, and crowding to
    /// its income.
    pub enabled: bool,
    /// Years over which `zbar` follows `z`: a generation.
    pub lag_years: f64,
    /// The `zbar` at which the income term is halved (between `floor` and 1).
    pub z_half: f64,
    /// How sharply the income term falls about `z_half`.
    pub z_power: f64,
    /// The least share of the income term that is kept, however well people live.
    pub floor: f64,
    /// Baskets a head at which drains and clean water begin...
    pub clean_from: f64,
    /// ...and how many more baskets a head make them complete, when a crowded
    /// town costs no more lives than the country.
    pub clean_span: f64,
}

impl Default for TransitionParams {
    /// Off: what the port runs with.
    fn default() -> TransitionParams {
        TransitionParams { enabled: false, lag_years: 25.0, z_half: 4.0, z_power: 4.0, floor: 0.04, clean_from: 4.0, clean_span: 6.0 }
    }
}

impl TransitionParams {
    /// The game's: the income term halved at a `zbar` of 4.5, not 4. At 4 the
    /// towns' share of people in 1900 stood above every island's with the rule
    /// off; raised a quarter at a time, 4.5 was the first step to bring all
    /// twelve facts of the standing check inside (docs/rules/transition.md,
    /// "Its numbers").
    pub fn game() -> TransitionParams {
        TransitionParams { enabled: true, z_half: 4.5, ..TransitionParams::default() }
    }
}

/// What the rule keeps between years (`st.transition`). Empty while it is off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransitionState {
    /// Each town's slow measure of how well its townsfolk live (`UNSET` until
    /// its first year under the rule).
    pub zbar: Vec<f64>,
    /// For the facts: each year's people-weighted island means of `F` and of
    /// `z`, with the year.
    pub years: Vec<(i64, f64, f64)>,
}

/// The share of the income term kept at a `zbar`.
pub fn fertility(tp: &TransitionParams, zbar: f64) -> f64 {
    tp.floor + (1.0 - tp.floor) / (1.0 + (zbar.max(0.0) / tp.z_half).powf(tp.z_power))
}

/// How far drains and clean water have come at `y` baskets a head, 0 to 1.
pub fn drains(tp: &TransitionParams, y: f64) -> f64 {
    ((y - tp.clean_from) / tp.clean_span.max(1e-9)).clamp(0.0, 1.0)
}

/// How well a town's townsfolk live: their share of its people times its
/// baskets a head.
pub fn z_of(st: &State, t: usize, y: f64) -> f64 {
    if st.pop[t] <= 0.0 {
        return 0.0;
    }
    (st.urban_pop[t].max(0.0) / st.pop[t]).min(1.0) * y
}

/// Once a year, from `peopling::growth` with each town's baskets a head:
/// move every active town's `zbar` toward this year's `z`, and give back each
/// town's `(F, S)`. None while the rule is off.
pub fn year(st: &mut State, P: &Params, y: &[f64], year: i64) -> Option<Vec<(f64, f64)>> {
    let tp = &P.transition;
    if !tp.enabled {
        return None;
    }
    let T = st.T_max;
    if st.transition.zbar.len() != T {
        st.transition.zbar.resize(T, UNSET);
    }
    let mut fs = vec![(1.0, 0.0); T];
    let (mut people, mut f_sum, mut z_sum) = (0.0, 0.0, 0.0);
    for t in 0..T {
        if !st.town_active[t] {
            continue;
        }
        let z = z_of(st, t, y[t]);
        let zb = &mut st.transition.zbar[t];
        if *zb < 0.0 {
            *zb = z;
        } else {
            *zb += (z - *zb) / tp.lag_years.max(1.0);
        }
        let f = fertility(tp, *zb);
        fs[t] = (f, drains(tp, y[t]));
        let w = st.pop[t].max(0.0);
        people += w;
        f_sum += w * f;
        z_sum += w * z;
    }
    let mean = |s: f64| if people > 0.0 { s / people } else { f64::NAN };
    st.transition.years.push((year, mean(f_sum), mean(z_sum)));
    Some(fs)
}

/// The people-weighted mean of what a town keeps and what comes in, each
/// `(people, zbar)`; a part not yet under the rule counts for nothing.
fn blend(parts: impl Iterator<Item = (f64, f64)>) -> Option<f64> {
    let (mut w, mut s) = (0.0, 0.0);
    for (p, zb) in parts {
        if p > 0.0 && zb >= 0.0 {
            w += p;
            s += p * zb;
        }
    }
    if w > 0.0 { Some(s / w) } else { None }
}

/// The year's migrants carry their towns' `zbar` (`people::migration`, before
/// the people are moved). `flow[(i, j)]` is from `act[i]` to `act[j]`.
pub fn migrate(st: &mut State, P: &Params, act: &[usize], flow: &Mat) {
    if !P.transition.enabled || st.transition.zbar.len() != st.T_max {
        return;
    }
    let n = act.len();
    let was: Vec<f64> = act.iter().map(|&t| st.transition.zbar[t]).collect();
    for k in 0..n {
        let t = act[k];
        let outflow: f64 = (0..n).map(|j| flow[(k, j)]).sum();
        let kept = (st.pop[t] - outflow).max(0.0);
        let parts = std::iter::once((kept, was[k])).chain((0..n).filter(|&i| i != k).map(|i| (flow[(i, k)], was[i])));
        if let Some(v) = blend(parts) {
            st.transition.zbar[t] = v;
        }
    }
}

/// The people of the districts that changed market carry their towns' `zbar`
/// (`peopling::move_with_cells`, after the people are moved). `pop` is the
/// people before; `share[(i, j)]` the part of town i's people who went to town
/// j. A town founded this year keeps none of its own: it is what came.
pub fn move_with_cells(st: &mut State, P: &Params, pop: &[f64], share: &Mat) {
    if !P.transition.enabled || st.transition.zbar.len() != st.T_max {
        return;
    }
    let T = st.T_max;
    let was = st.transition.zbar.clone();
    for j in 0..T {
        let came: f64 = (0..T).filter(|&i| i != j).map(|i| pop[i] * share[(i, j)]).sum();
        if came <= 0.0 {
            continue;
        }
        let left: f64 = (0..T).filter(|&k| k != j).map(|k| share[(j, k)]).sum();
        let kept = if st.town_new[j] { 0.0 } else { (pop[j] * (1.0 - left)).max(0.0) };
        let parts = std::iter::once((kept, was[j])).chain((0..T).filter(|&i| i != j).map(|i| (pop[i] * share[(i, j)], was[i])));
        if let Some(v) = blend(parts) {
            st.transition.zbar[j] = v;
        }
    }
}

/// A new town starts with its donor's `zbar` (`peopling::open_town`), until
/// its people come with their cells and bring their own.
pub fn open_town(st: &mut State, P: &Params, n: usize, donor: usize) {
    if !P.transition.enabled {
        return;
    }
    if st.transition.zbar.len() != st.T_max {
        st.transition.zbar.resize(st.T_max, UNSET);
    }
    st.transition.zbar[n] = st.transition.zbar[donor];
}

/// Facts for the sweep's table, under "People and towns": `transition_F_Y` and
/// `transition_z_Y`, the people-weighted island means of `F` and `z` at the
/// sweep's marks (only with the rule on).
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C, P) = (&r.state, &r.content, &r.params);
    let mut out = Vec::new();
    if !P.transition.enabled {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    for y in crate::facts::marks(y0, y1) {
        let (f, z) = st.transition.years.iter().rev().find(|v| v.0 == y).map(|v| (v.1, v.2)).unwrap_or((f64::NAN, f64::NAN));
        out.push((format!("transition_F_{y}"), Kind::Share, f));
        out.push((format!("transition_z_{y}"), Kind::Ratio, z));
    }
    out
}
