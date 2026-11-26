//! Births that answer to free land, and towns founded at a hamlet's size where
//! the land is free (not in the Python; `docs/rules/frontier.md`).
//!
//! In history a colony with land to spare grew by its own births at 2.5 to 3
//! per cent a year, where an old country with the same incomes grew at well
//! under one, and the rate fell as the land filled: people married younger and
//! had more children where a son could clear a farm of his own (the page gives
//! the sources, from memory). Under the game's people (`peopling::growth`) a
//! town's births less deaths from income alone is
//! `growth_top * (1 - (y_still / y)^ease)`: one ceiling, the same on a full
//! island and an empty one.
//!
//! With `enabled`, once a year and town by town, the ceiling is raised by
//! `births` times the town's share of free land: the fertile land within
//! `reach` cells of its market place, in its own market area, weighed by how
//! fertile each cell is and by how far it is from holding `full` hands at work
//! on the land. The extra births are the farm families': the lift is full while
//! half the town's hands or more work its land (`farmers`), and falls in
//! proportion as fewer do, so a town of works and shops does not breed as a
//! frontier does because the country round it is thinly farmed. So at a bare
//! living (`y_still`) births still balance deaths, and the free land adds only
//! to what income gives; and the transition, which multiplies the income term,
//! takes the extra births down with the rest.
//!
//! With `hamlets`, a district whose land is half free or more may be made a
//! town at `hamlet_people` people, giving up to `hamlet_donor` of any one
//! town's people, where the game's founding rule asks 3,000 and a fifth
//! (`peopling::found_towns`); every other test of a founding stands.
//!
//! Both are off under the game's rules, so the settled island is as it was:
//! they come with the landfall start (`landfall::landfall`). Off, nothing here
//! runs, and `State::frontier` stays empty.

use serde::{Deserialize, Serialize};

use crate::params::Params;
use crate::state::State;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FrontierParams {
    /// Births that answer to free land.
    pub enabled: bool,
    /// What free land adds to the ceiling of births less deaths a year, where
    /// all the land within reach is free. 0.018 lifts the island's ceiling of
    /// 0.016 to 0.034, about a colony's crude birth rate of 50 a thousand less
    /// deaths of 15 (from memory); at the incomes where the island's rate is
    /// about four fifths of its ceiling, that is 2.7 per cent a year.
    pub births: f64,
    /// How far from its market place a town's free land counts, in cells (2 km).
    /// 20 cells, 40 km: a day or two's walk, as far as a son would go to clear a
    /// farm and still sell at his father's market.
    pub reach: f64,
    /// Hands at work on the land of a cell (4 km2) at which the cell counts as
    /// taken. 10 is about what the settled island has on its land in 1750 (60,000
    /// land hands on 5,800 cells of island 7, many of them on the poorer cells):
    /// the settled island is what full looks like at 1750's farming.
    pub full: f64,
    /// The share of a town's hands on its land at and above which the lift is
    /// whole; below it the lift falls in proportion. 0.5: about what the first
    /// town has on its land in its first years (819 of 1,590 hands on island 7
    /// in 1753). History's colonies were nine in ten farmers; the game counts
    /// their crafts, shops and services among its works.
    pub farmers: f64,
    /// Towns may be founded at a hamlet's size where the district's land is
    /// half free or more.
    pub hamlets: bool,
    /// The people a district must hold to be made a town where its land is free.
    /// 500: a New England township was granted to some fifty or sixty families,
    /// and a market town of the colonies was a few hundred to a thousand or two
    /// (from memory).
    pub hamlet_people: f64,
    /// The most of any one town's people a hamlet's district may take.
    pub hamlet_donor: f64,
}

impl Default for FrontierParams {
    /// Off: what the port and the settled island run with.
    fn default() -> FrontierParams {
        FrontierParams { enabled: false, births: 0.018, reach: 20.0, full: 10.0, farmers: 0.5, hamlets: false, hamlet_people: 500.0, hamlet_donor: 0.5 }
    }
}

impl FrontierParams {
    /// What the landfall start plays with: both on.
    pub fn game() -> FrontierParams {
        FrontierParams { enabled: true, hamlets: true, ..FrontierParams::default() }
    }
}

/// What the rule keeps between years (`st.frontier`). Empty while it is off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrontierState {
    /// Each town's share of free land within its reach, as the year's people were reckoned.
    pub free: Vec<f64>,
    /// And that share weighed by how far the town lives off its land: what lifts its births.
    pub lifted: Vec<f64>,
}

/// How free a cell's land is: 1 with no hands on it, 0 at `full` and above.
fn free_cell(P: &Params, hands: f64) -> f64 {
    (1.0 - hands / P.frontier.full.max(1e-9)).clamp(0.0, 1.0)
}

/// Whether a cell's land counts: land, below the mountains, with some fertility.
fn counts(st: &State, c: usize) -> bool {
    st.world.land[c] && st.world.terrain[c] < 3 && st.world.fertility[c] > 0.0
}

/// Each town's share of free land within `reach` of its market place, in its
/// own market area, weighed by fertility; 0 for a town with no such land.
pub fn free_shares(st: &State, P: &Params) -> Vec<f64> {
    let (T, W) = (st.T_max, st.world.W);
    let (mut free, mut all) = (vec![0.0; T], vec![0.0; T]);
    let r = P.frontier.reach;
    for c in 0..st.N {
        let t = st.catch.get(c).copied().unwrap_or(-1);
        if t < 0 || !counts(st, c) {
            continue;
        }
        let t = t as usize;
        let node = st.town_node[t];
        if ((c / W) as f64 - (node / W) as f64).hypot((c % W) as f64 - (node % W) as f64) > r {
            continue;
        }
        let f = st.world.fertility[c];
        all[t] += f;
        free[t] += f * free_cell(P, st.cell_labor.get(c).copied().unwrap_or(0.0));
    }
    (0..T).map(|t| if all[t] > 0.0 { free[t] / all[t] } else { 0.0 }).collect()
}

/// The year's shares, kept for the record and the probes (`peopling::growth`,
/// before the rates are set), each weighed by how far the town lives off its
/// land: its hands on the land over all its hands at work (land and works,
/// `st.ag_labor` and `st.ind_labor`, last year's), over `farmers`, at most 1.
/// None while the rule is off.
pub fn year(st: &mut State, P: &Params) -> Option<Vec<f64>> {
    if !P.frontier.enabled {
        return None;
    }
    let f = free_shares(st, P);
    st.frontier.free = f.clone();
    let w: Vec<f64> = (0..f.len())
        .map(|t| {
            let land = st.ag_labor.get(t).copied().unwrap_or(0.0);
            let all = land + st.ind_labor.get(t).copied().unwrap_or(0.0);
            let share = if all > 0.0 { land / all } else { 1.0 };
            f[t] * (share / P.frontier.farmers.max(1e-9)).min(1.0)
        })
        .collect();
    st.frontier.lifted = w.clone();
    Some(w)
}

/// The ceiling's lift for a town with free share `free`: what the income term
/// is multiplied by (`peopling::growth`).
pub fn lift(P: &Params, free: f64) -> f64 {
    1.0 + P.frontier.births * free / P.people.growth_top.max(1e-9)
}

/// The share of free land in a set of cells, weighed by fertility (for a
/// district that might become a town).
pub fn district_free(st: &State, P: &Params, cells: impl Iterator<Item = usize>) -> f64 {
    let (mut free, mut all) = (0.0, 0.0);
    for c in cells {
        if counts(st, c) {
            let f = st.world.fertility[c];
            all += f;
            free += f * free_cell(P, st.cell_labor.get(c).copied().unwrap_or(0.0));
        }
    }
    if all > 0.0 { free / all } else { 0.0 }
}

/// The people and the donor's share a district must pass to be made a town:
/// the hamlet's where its land is half free or more and `hamlets` is on, the
/// game's founding rule's otherwise.
pub fn founding_needs(P: &Params, free: f64) -> (f64, f64) {
    if P.frontier.hamlets && free >= 0.5 {
        (P.frontier.hamlet_people, P.frontier.hamlet_donor)
    } else {
        (P.people.found_people, P.people.donor_max)
    }
}
