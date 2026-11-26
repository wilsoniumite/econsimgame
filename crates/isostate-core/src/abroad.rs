//! Ideas by sea (not in the Python; `docs/rules/abroad.md`).
//!
//! The island finds an idea by its own make of the goods the idea is found on
//! (`tech::discovery`), and an idea spreads from town to town by contact over
//! the cost of the way between them (`tech::diffusion`). With `ideas`, every
//! port has one more neighbour: **abroad**, across a sea whose cost is `sea`,
//! in the same units as the cost of the way between two towns. Abroad knows
//! each idea from the year `data/abroad.toml` gives, and from that year its
//! know of it grows as a town's does that has found an idea and has nobody to
//! learn it from: a quarter at once, then `diffusion_rate` of what it does not
//! yet know times what it knows, each year. An idea abroad knows counts as
//! found for the spreading, so a port learns it by contact with abroad as it
//! would from a town across a bay, and the island's other towns learn it from
//! the port. The idea counts as found on the island (`State::tech_year`) in
//! the year a town's know of it first passes `USE`, the know at which a works
//! may first take a method (`firms::review`).
//!
//! The uses of the tasks rule (`data/tasks.toml`) have no know to spread: a
//! use is found at the year's turn `lag` years after abroad has it, where the
//! island has not found it first, as long as the idea it waits on is found
//! and its switch is on.
//!
//! The island's own finding stays as it was: the colony may find a thing
//! before abroad does, as history's colonies found labour-saving machines of
//! their own. A pin by hand (`edits::Pin`) that holds an idea back holds it
//! back from the sea too.
//!
//! Off, nothing here runs and nothing is read. It is off under both sets of
//! rules, which is the isolated state, and comes with the landfall start
//! (`landfall::landfall`). When a real second island stands across the sea it
//! takes the table's place: its towns' know is what the table stands in for,
//! and no rule changes.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::params::Params;
use crate::state::{Event, State};

/// The table of years built in (`data/abroad.toml`).
pub const ABROAD_TOML: &str = include_str!("../../../data/abroad.toml");

/// The know at which an idea counts as found on the island when it came by
/// sea: 0.02, the know at which a works may first take a method
/// (`firms::review`, `families`).
pub const USE: f64 = 0.02;

/// The know a town has of an idea in the year it finds it (`tech::discovery`).
const FOUND: f64 = 0.25;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AbroadParams {
    /// Ideas come by sea: every port has abroad as one more neighbour.
    pub ideas: bool,
    /// The cost of the sea between a port and abroad, in the units of the
    /// cost of the way between two towns (`State::town_cost`; a cell of
    /// lowland with no road is 1, of open sea 0.22). 105: three times the
    /// distance over which contact falls by e (`diffusion_dist`, 35), so a
    /// port alone weighs abroad at 0.05. With it the colony's port first
    /// uses an idea four to seven years after abroad has found it and knows it
    /// as well as its finder did (a quarter) twelve to seventeen years after:
    /// history's colonies had the old country's manufactures 10 to 40 years
    /// late in the eighteenth century and 3 to 10 by the middle of the
    /// nineteenth (docs/rules/abroad.md, "Its numbers").
    pub sea: f64,
    /// The years after abroad that a use of the tasks rule is found: 5. The
    /// uses all come after 1900, with steamships and the cable: the tractor and
    /// the lorry reached the colonies of settlement within a few years of the
    /// old countries (from memory).
    pub lag: i64,
}

impl Default for AbroadParams {
    /// Off: the island alone, the isolated state.
    fn default() -> AbroadParams {
        AbroadParams { ideas: false, sea: 105.0, lag: 5 }
    }
}

impl AbroadParams {
    /// On, with the numbers above (the landfall start's).
    pub fn game() -> AbroadParams {
        AbroadParams { ideas: true, ..AbroadParams::default() }
    }
}

/// The year abroad knows each idea of the tree and each use of the tasks rule,
/// by name.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    #[serde(default)]
    pub idea: BTreeMap<String, i64>,
    #[serde(default, rename = "use")]
    pub uses: BTreeMap<String, i64>,
}

/// A table from its text.
pub fn table_from(text: &str) -> Result<Table, String> {
    toml::from_str(text).map_err(|e| format!("abroad: {e}"))
}

/// The table built in.
pub fn table() -> &'static Table {
    static T: OnceLock<Table> = OnceLock::new();
    T.get_or_init(|| table_from(ABROAD_TOML).expect("embedded data/abroad.toml is valid"))
}

/// The year abroad knows idea `t` of the tree, if it does.
pub fn idea_year(C: &Content, t: usize) -> Option<i64> {
    table().idea.get(&C.tech_name[t]).copied()
}

/// The year abroad has use `u` of the tasks rule, if it does.
pub fn use_year(C: &Content, u: usize) -> Option<i64> {
    table().uses.get(&C.tasks.uses[u].name).copied()
}

/// Abroad's know of an idea it has known since `since`, in `year`: nothing
/// before, a quarter in the year itself, then the growth of a town that found
/// it alone, `diffusion_rate` of what it lacks times what it knows a year
/// (the logistic through a quarter).
pub fn know(P: &Params, since: i64, year: i64) -> f64 {
    if year < since {
        return 0.0;
    }
    let odds = (1.0 - FOUND) / FOUND;
    1.0 / (1.0 + odds * (-P.diffusion_rate * (year - since) as f64).exp())
}

/// What the sea brings to a year's spreading of ideas (`tech::diffusion_with`).
pub struct Sea {
    /// Abroad's weight as a neighbour of each town before the towns' weights
    /// are shared out: `exp(-sea / diffusion_dist)` for a port, 0 inland.
    pub weight: Vec<f64>,
    /// Abroad's know of each idea this year (0 where the island may not have
    /// it from the sea: not known abroad, not yet sought, or held by a pin).
    pub know: Vec<f64>,
}

/// The sea of `year`, or None while the rule is off.
pub fn sea(st: &State, C: &Content, P: &Params, year: i64) -> Option<Sea> {
    if !P.abroad.ideas {
        return None;
    }
    let w = (-P.abroad.sea / P.diffusion_dist).exp();
    let weight = (0..st.T_max).map(|i| if st.town_active[i] && st.world.coast[st.town_node[i]] { w } else { 0.0 }).collect();
    let know = (0..C.NT).map(|t| if may_come(st, C, P, t, year) { idea_year(C, t).map_or(0.0, |y| know(P, y, year)) } else { 0.0 }).collect();
    Some(Sea { weight, know })
}

/// Whether idea `t` may come by sea in `year`: the island may seek it at all
/// (the rules that keep an idea back while their switch is off, as
/// `tech::discovery` asks them), not before the content's first year for it,
/// and not while a pin by hand holds it back.
fn may_come(st: &State, C: &Content, P: &Params, t: usize, year: i64) -> bool {
    if C.tech_known[t] {
        return false;
    }
    if crate::tasks::idea_of_use(C, t) && !crate::tasks::idea_sought(C, P, t) {
        return false;
    }
    if crate::care::is_idea(C, t) && !crate::care::sought(P) {
        return false;
    }
    if !crate::woods::sought(C, P, t) || year < C.tech_after.get(t).copied().unwrap_or(0) {
        return false;
    }
    st.by_hand.pins.is_empty() || crate::edits::pin_says(st, crate::edits::Idea::Tech(t), year) != crate::edits::Pinned::Hold
}

/// After the year's spreading: each idea that came by sea and that a town now
/// knows past `USE` is found on the island this year, in the town that knows
/// it best, with the news of it.
pub fn landed(st: &mut State, C: &Content, sea: &Sea, year: i64, events: &mut Vec<Event>) {
    for t in 0..C.NT {
        if st.tech_year[t] >= 0 || sea.know[t] <= 0.0 {
            continue;
        }
        let mut best: Option<(f64, usize)> = None;
        for i in (0..st.T_max).filter(|&i| st.town_active[i]) {
            if best.is_none_or(|(b, _)| st.know[(i, t)] > b) {
                best = Some((st.know[(i, t)], i));
            }
        }
        let Some((k, i)) = best else { continue };
        if k <= USE {
            continue;
        }
        st.tech_year[t] = year;
        st.tech_town[t] = i as i64;
        let (label, town) = (C.tech_label[t].clone(), st.town_name[i].clone());
        let mut ev = Event::new(
            year as f64 + 0.5,
            "discovery",
            0.9,
            format!("{label} comes by sea to {town}"),
            format!("{label} was known abroad from {}, and word of it has come by sea to {town}: masters and men who have seen it worked, books and letters, a machine or two. Enough is known here now to try it, and it spreads from {town} along the island's trade as any idea does.", idea_year(C, t).unwrap_or(year)),
        );
        ev.town = Some(i);
        ev.cell = Some(st.town_node[i]);
        ev.tech = Some(t);
        events.push(ev);
    }
}

/// Whether use `u` of the tasks rule comes by sea at the turn into `next`:
/// `lag` years after abroad has it.
pub fn use_comes(C: &Content, P: &Params, u: usize, next: i64) -> bool {
    P.abroad.ideas && use_year(C, u).is_some_and(|y| next >= y + P.abroad.lag)
}
