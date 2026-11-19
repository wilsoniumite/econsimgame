//! WOODS: managed woodland, and the woods' cover recorded (not in the Python;
//! see `docs/rules/woods.md`).
//!
//! Under the port the woods grow back on their own, at `forest_regrow` a year
//! on what stands (`land::update_land_use`), and are cut by the woodmen and by
//! clearing. On the island run on past 1900 they are cut faster than they grow
//! from about 1950, timber's price goes from a fifth of a month's wage in 1900
//! to six months' in 2050, and with it the plant of the mills and breweries and
//! the builders' work (the diagnosis of 19 November 2026).
//!
//! The forester's craft is an invention of the game's tree, `managed_woodland`
//! (`data/game.toml`, stage 6): woods planted, thinned and felled in rotation.
//! It is found as the tree's inventions are, on what the island has made (here
//! timber), but not before its `found_after`, and spreads between towns as they
//! all do. Where a town knows it, the woods of its market area grow back faster:
//! the regrowth term is multiplied by `1 + (regrow - 1) * know`, `know` being the
//! share of the town that knows the craft, so at `regrow` 3 and full knowledge
//! the port's 0.12 a year is 0.36. Nothing else about how the woods are cut, or
//! what uses timber, is changed.
//!
//! With `record`, the observer keeps two series, the woods' standing cover (what
//! stands on the woodland and wild land against what that land could carry) and
//! timber's price in months of the average wage, and the facts read them by
//! decade. Recording never touches the economy.
//!
//! Off is off: with `enabled` false the invention is never sought (no number is
//! drawn for it, so every other invention is found as it was) and the woods grow
//! back at the port's rate.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::params::Params;
use crate::state::State;

/// The invention's name in the overlay.
pub const MANAGED: &str = "managed_woodland";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WoodsParams {
    /// Managed woodland may be found, and where it is known the woods grow
    /// back faster.
    pub enabled: bool,
    /// How many times as fast the woods grow back where the whole town knows
    /// the craft.
    pub regrow: f64,
    /// The observer keeps the woods' cover and timber's price in months of the
    /// wage, and the facts read them. It watches and never touches the economy.
    pub record: bool,
}

impl Default for WoodsParams {
    /// Off: what the port runs with.
    fn default() -> WoodsParams {
        WoodsParams { enabled: false, regrow: 3.0, record: false }
    }
}

impl WoodsParams {
    /// The game's woods: managed woodland, and the cover recorded.
    pub fn game() -> WoodsParams {
        WoodsParams { enabled: true, record: true, ..WoodsParams::default() }
    }
}

/// Whether invention `t` may be sought this year: not managed woodland while
/// the rule is off.
pub fn sought(C: &Content, P: &Params, t: usize) -> bool {
    P.woods.enabled || C.tech_name[t] != MANAGED
}

/// Managed woodland's place among the inventions, where the rule is on and the
/// content has it.
pub fn managed(C: &Content, P: &Params) -> Option<usize> {
    if !P.woods.enabled {
        return None;
    }
    C.tech_name.iter().position(|n| n == MANAGED)
}

/// The rate the woods of cell `c` grow back at: the port's, times what the
/// craft adds where its market town knows it (`managed` from [`managed`]).
pub fn regrow(st: &State, P: &Params, managed: Option<usize>, c: usize) -> f64 {
    match managed {
        None => P.forest_regrow,
        Some(t) => {
            let know = if st.catch[c] >= 0 { st.know[(st.catch[c] as usize, t)] } else { 0.0 };
            P.forest_regrow * (1.0 + (P.woods.regrow - 1.0) * know)
        }
    }
}

/// The share of the town that knows managed woodland, for the inspector's card
/// of a square: None where the rule is off, the content lacks it or the square
/// has no market.
pub fn known_at(st: &State, C: &Content, P: &Params, c: usize) -> Option<f64> {
    let t = managed(C, P)?;
    (st.catch[c] >= 0).then(|| st.know[(st.catch[c] as usize, t)])
}

/// The woods' standing cover: what stands on the woodland and wild land, cell
/// by cell, against what that land could carry (`World::forest_pot`), each cell
/// weighted by the share of it under woods and wild. 1 is every wood standing
/// as thick as its ground allows.
pub fn cover(st: &State) -> f64 {
    let N = st.N;
    let lam = &st.lam;
    if lam.cols != N || lam.rows == 0 {
        return f64::NAN;
    }
    let (fi, wild) = (st.forestry_sector, lam.rows - 1);
    let (mut stands, mut could) = (0.0, 0.0);
    for c in 0..N {
        if !st.world.land[c] {
            continue;
        }
        let w = lam[(fi, c)] + lam[(wild, c)];
        stands += w * st.cover[c];
        could += w * st.world.forest_pot[c];
    }
    if could > 0.0 { stands / could } else { f64::NAN }
}

/// The observer's series, only with `record` (a port run's list stays as it is).
pub fn add_series(reg: &mut crate::observer::Series, P: &Params, n_months: usize) {
    if P.woods.record {
        reg.add("woods_cover", "The woods' cover", "land", "share", n_months);
        reg.add("timber_in_wages", "Timber in months of the wage", "prices", "months", n_months);
    }
}

/// The month's two figures: the cover, and a unit of timber at the island's
/// price over the month's average wage.
pub fn record_month(st: &State, C: &Content, P: &Params, s: &mut crate::observer::Series, t: usize, wage: f64, P_nat: &[f64]) {
    if !P.woods.record {
        return;
    }
    s.set("woods_cover", t, cover(st));
    let timber = C.good_index.get("timber").map_or(f64::NAN, |&g| P_nat[g]);
    s.set("timber_in_wages", t, if wage > 0.0 { timber / wage } else { f64::NAN });
}

/// The facts: `woods_cover_Y` and `timber_in_wages_Y` at the marks (every
/// fifty years to 1900 and every ten after), the year's means; nothing in a run
/// that did not record them.
pub fn facts(r: &crate::sim::RunResult) -> Vec<(String, crate::facts::Kind, f64)> {
    use crate::facts::Kind;
    let (st, C) = (&r.state, &r.content);
    let mut out = Vec::new();
    if !st.series.index.contains_key("woods_cover") {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    let at = |name: &str, y: i64| st.series.index.get(name).map_or(f64::NAN, |&k| crate::observer::annual(&st.series.data[k], y0, y));
    for y in crate::facts::marks(y0, y1) {
        out.push((format!("woods_cover_{y}"), Kind::Share, at("woods_cover", y)));
    }
    for y in crate::facts::marks(y0, y1) {
        out.push((format!("timber_in_wages_{y}"), Kind::Real, at("timber_in_wages", y)));
    }
    out
}
