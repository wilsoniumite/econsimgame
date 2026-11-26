//! The lorry's roads. Not in the Python (`docs/rules/lorry.md`).
//!
//! The lorry itself is a machine for the carters' tasks (`tasks.lorry`, a
//! `[[use]]` of `data/tasks.toml` on the carting, made of motor vehicles and run
//! on oil): each town's carting holds a share of its hauling with lorries, an
//! hour of its work then takes fewer hands and costs less, and the cost graph's
//! land squares are cheaper by what an hour of the carting's work has come to
//! cost over the wage (`tasks::TasksState::cart_kappa`), so a load costs less
//! where lorries run. This module holds the two rules of the ways that go with
//! it, each behind a switch of its own:
//!
//! - **A lorry wants a made road** (`lorry.roads`): the lorry's saving is had
//!   in full on a made road or a turnpike, a quarter of it on a track, and none
//!   on open ground, so that a made road matters again. With it off the saving
//!   is had on every land square alike.
//! - **Nothing unmade by quiet** (`lorry.keep`): from 1901 a made road or a
//!   turnpike no longer falls back for want of traffic. Only a track fades.
//!
//! Off, nothing here acts: the cost graph is built with the port's multipliers
//! and every road falls back as it always has.

use serde::{Deserialize, Serialize};

use crate::facts::Kind;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;
use crate::transport::ROAD_MULT;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LorryParams {
    /// The lorry's saving is had by the road under it: in full on a made road
    /// or turnpike, `SURFACE` of it elsewhere.
    pub roads: bool,
    /// From `KEEP_FROM` a made road or a turnpike no longer falls back for want
    /// of traffic.
    pub keep: bool,
}

impl Default for LorryParams {
    /// Off: what the port runs with.
    fn default() -> LorryParams {
        LorryParams { roads: false, keep: false }
    }
}

impl LorryParams {
    /// The game's.
    pub fn game() -> LorryParams {
        LorryParams { roads: true, keep: true }
    }
}

/// The share of the lorry's saving had on each level of road (none, track,
/// made road, turnpike) under `lorry.roads`: a lorry on open ground is no
/// better than a cart, on a track it sinks to its axles in a wet month, and on
/// a made surface it runs as it was built to.
pub const SURFACE: [f64; 4] = [0.0, 0.25, 1.0, 1.0];

/// The first year in which a made road is kept whatever its traffic.
pub const KEEP_FROM: i64 = 1901;

/// The cost of a land square by its road level, for the year's cost graph
/// (`land::update_transport`): the port's `ROAD_MULT`, each level's cut by the
/// share of the lorry's saving had there. The saving is `1 - kappa`, kappa the
/// cost of an hour of the carting's work over the wage in the year just closed;
/// before the lorry carts anything kappa is unset and this is `ROAD_MULT`.
pub fn road_mult(st: &State, P: &Params) -> [f64; 4] {
    let k = st.tasks.cart_kappa;
    if !(P.tasks.lorry && k > 0.0 && k < 1.0) {
        return ROAD_MULT;
    }
    let g = if P.lorry.roads { SURFACE } else { [1.0; 4] };
    std::array::from_fn(|l| ROAD_MULT[l] * (1.0 - g[l] * (1.0 - k)))
}

/// Whether a road of level `level` is kept this year whatever its traffic
/// (`sim::update_roads`): a made road or a turnpike, from 1901, under `lorry.keep`.
pub fn kept(P: &Params, year: i64, level: u8) -> bool {
    P.lorry.keep && year >= KEEP_FROM && level >= 2
}

/// The lorry's series, after the others, only while the lorry may be found (a
/// run with it off keeps the list it kept): `lorry_share`, the share of the
/// carting's hauling the lorries in place did in the month, each town by its
/// carting's work (nought before the find), which the viewer's Traffic view
/// reads to tell road freight by lorry from road freight by cart.
pub fn add_series(reg: &mut crate::observer::Series, P: &Params, n_months: usize) {
    if P.tasks.enabled && P.tasks.lorry && P.carting.enabled {
        reg.add("lorry_share", "Hauling done by lorry, of the carting's", "transport", "share", n_months);
    }
}

/// The month's figure (`observer::record_month`).
pub fn record_month(st: &State, P: &Params, s: &mut crate::observer::Series, t: usize) {
    if !(P.tasks.enabled && P.tasks.lorry && P.carting.enabled) {
        return;
    }
    let tk = &st.tasks;
    let (mut a, mut b) = (0.0, 0.0);
    for i in 0..tk.cart_work.len().min(tk.cart_held.len()).min(st.town_active.len()) {
        if st.town_active[i] && tk.cart_work[i] > 0.0 {
            a += tk.cart_work[i] * tk.cart_held[i][crate::tasks::HAUL];
            b += tk.cart_work[i];
        }
    }
    s.set("lorry_share", t, if b > 0.0 { a / b } else { 0.0 });
}

/// The rule's facts, every tenth year from 1900 to the end: the share of the
/// carting's hauling the lorries in place did (`lorry_share_Y`), the hands an
/// hour of the carting's work took, 1 by hand (`cart_hands_a_load_Y`), and its
/// cost over the wage (`cart_kappa_Y`); and the first year the hands a load
/// takes are half what they were by hand (`cart_hands_half_year`). Nothing
/// while the lorry is off.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C, P) = (&r.state, &r.content, &r.params);
    let mut out = Vec::new();
    if !(P.tasks.enabled && P.tasks.lorry) || !C.tasks.uses.iter().any(|u| u.on_carting) {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    let at = |y: i64| st.tasks.years.iter().find(|t| t.year == y);
    let years: Vec<i64> = (1900..=y1).step_by(10).filter(|&y| y >= y0).collect();
    let none = |v: f64| if v > 0.0 { v } else { f64::NAN };
    for &y in &years {
        out.push((format!("lorry_share_{y}"), Kind::Share, at(y).map_or(f64::NAN, |t| if t.cart_work > 0.0 { t.cart_held } else { 0.0 })));
    }
    for &y in &years {
        out.push((format!("cart_hands_a_load_{y}"), Kind::Share, at(y).map_or(f64::NAN, |t| if t.cart_work > 0.0 { t.cart_hands } else { 1.0 })));
    }
    for &y in &years {
        out.push((format!("cart_kappa_{y}"), Kind::Share, at(y).map_or(f64::NAN, |t| none(t.cart_kappa))));
    }
    let half = st.tasks.years.iter().find(|t| t.cart_work > 0.0 && t.cart_hands <= 0.5).map_or(f64::NAN, |t| t.year as f64);
    out.push(("cart_hands_half_year".to_string(), Kind::Year, half));
    // the facts of freight (docs/rules/traffic.md) gain the lorry: of the freight by
    // road, millions of weight-km, the part the lorries' share of the hauling carried
    for y in crate::facts::marks(y0, y1).into_iter().filter(|&y| y >= 1900) {
        let road = st.tf.years.iter().find(|t| t.year == y).map_or(f64::NAN, |t| t.weight_squares[0] * 2.0 / 1e6);
        let share = at(y).map_or(f64::NAN, |t| if t.cart_work > 0.0 { t.cart_held } else { 0.0 });
        out.push((format!("freight_lorry_{y}"), Kind::Real, road * share));
    }
    out
}
