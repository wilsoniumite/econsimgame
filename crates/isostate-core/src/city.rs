//! THE FIRST CITY: three rules that act after 1900, each with a switch of its
//! own (not in the Python; see `docs/rules/city.md`).
//!
//! The island of 2050 had some twenty-four towns that were all alike: the
//! largest held a ninth of all townsfolk where an island country of a million
//! has a first city of a fifth to a half. Two things held them level. People
//! moved between districts only toward higher income a head, which local
//! prices keep within a few per cent across towns, so nobody had a reason to
//! move; and the rent burden could not rise, by the form of the ground's rule.
//! The three rules:
//!
//! - **People follow jobs** (`follow_jobs`). Migration between districts
//!   (`people::migration`) reads the wage a newcomer would earn in each town,
//!   the year's mean wage, raised where the town's works took on more hands
//!   over the year than it had (`jobs`), in place of income a head. A town
//!   whose works grow draws people from the whole island.
//! - **Work gathers where work is** (`gather`). Every works in a town of U
//!   townsfolk is `(U / size)^power` as productive (`firms::update_productivity`
//!   and `firms::evaluate_sites`), the power ramped in over `ramp_years`.
//! - **Room is wanted less than in proportion as it dears** (`dear_room`). In
//!   the ground's rent (`ground::month`), the room a head takes above its least
//!   room falls as the rent a unit of room rises against the town's own of 1900
//!   (both over the long run's anchor) with an elasticity `room_elasticity`
//!   under one, where the ground's rule has it one: so a household in a town
//!   whose ground has grown dear gives up more of its budget for less room.
//!
//! Nothing acts in the yearly step of `from` or before: migration and the
//! works' productivity from the yearly step of the year after, the rent from
//! the months of the year after (as the ground's own rule). So the played
//! century is untouched by construction. Switched off, every function
//! returns at once, nothing is drawn and `State::city` stays empty.

use serde::{Deserialize, Serialize};

use crate::facts::Kind;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;

/// The numbers of the three rules.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CityParams {
    /// The last year of the old rules: each acts after it.
    pub from: i64,
    /// People follow jobs: migration reads the wage and the works' new hands.
    pub follow_jobs: bool,
    /// How much a town's works' new hands over the year, as a share of its
    /// people of working age, add to the wage a newcomer reads: a town whose
    /// works took on a tenth more hands than it had reads a wage `jobs / 10`
    /// higher.
    pub jobs: f64,
    /// Work gathers where work is: works are more productive in larger towns.
    pub gather: bool,
    /// The power of townsfolk in a works' productivity: a doubling of a town
    /// is worth `2^power - 1` more from every works in it.
    pub power: f64,
    /// The townsfolk at which a works is as productive as without the rule.
    pub size: f64,
    /// Years over which the power rises from nothing to `power`.
    pub ramp_years: i64,
    /// Room is wanted less than in proportion as it dears.
    pub dear_room: bool,
    /// How the room a head takes above its least falls with the rent a unit of
    /// room (1 is the ground's own rule, where the spending on it is a fixed
    /// share).
    pub room_elasticity: f64,
}

impl Default for CityParams {
    /// Off: what the port runs with. The numbers are the game's.
    fn default() -> CityParams {
        CityParams {
            from: 1900,
            follow_jobs: false,
            jobs: 2.0,
            gather: false,
            power: 0.07,
            size: 20_000.0,
            ramp_years: 10,
            dear_room: false,
            room_elasticity: 0.5,
        }
    }
}

impl CityParams {
    /// The game's first city.
    pub fn game() -> CityParams {
        CityParams { follow_jobs: true, gather: true, dear_room: true, ..CityParams::default() }
    }
}

/// `State::city`: empty while every rule is off.
#[derive(Clone, Debug, Default)]
pub struct CityState {
    /// Each town's works' hands at the last yearly step (follow_jobs).
    pub hands_was: Vec<f64>,
    /// Each town's works' productivity factor for the year (gather); empty
    /// where the rule does not act.
    pub scale: Vec<f64>,
}

// ---------------------------------------------------------------- people follow jobs

/// `people::migration`: what each active town (`act`) pulls a mover with in the
/// yearly step of `year`, in place of its income a head; None where the rule
/// does not act. Each town's works' hands are put by for next year's step.
pub fn pull(st: &mut State, P: &Params, year: i64, act: &[usize]) -> Option<Vec<f64>> {
    let cp = &P.city;
    if !cp.follow_jobs || year < cp.from {
        return None;
    }
    let T = st.T_max;
    let was = std::mem::replace(&mut st.city.hands_was, st.ind_labor.clone());
    if year == cp.from || was.len() != T {
        return None;
    }
    let out = act
        .iter()
        .map(|&t| {
            let w = st.wbar.get(t).copied().unwrap_or(0.0).max(1e-9);
            let hands = (P.participation * st.pop[t]).max(1.0);
            let open = ((st.ind_labor[t] - was[t]) / hands).clamp(0.0, 0.5);
            w * (1.0 + cp.jobs * open)
        })
        .collect();
    Some(out)
}

// ---------------------------------------------------------------- work gathers where work is

/// The yearly step, before `firms::update_productivity`: each town's factor on
/// its works' productivity for the year, or nothing where the rule does not act.
pub fn year_scale(st: &mut State, P: &Params, year: i64) {
    let cp = &P.city;
    if !cp.gather || year <= cp.from {
        st.city.scale.clear();
        return;
    }
    let ramp = ((year - cp.from) as f64 / cp.ramp_years.max(1) as f64).min(1.0);
    let power = cp.power * ramp;
    st.city.scale = (0..st.T_max)
        .map(|t| if st.town_active[t] { (st.urban_pop[t].max(1000.0) / cp.size.max(1.0)).powf(power) } else { 1.0 })
        .collect();
}

/// The factor on the productivity of a works in town `t` (1 where the rule does not act).
pub fn scale(st: &State, t: usize) -> Option<f64> {
    st.city.scale.get(t).copied()
}

// ---------------------------------------------------------------- room that grows dear

/// `ground::month`: the elasticity of the room a head takes with the rent, in
/// the months of `year_now`; None where the rule does not act (the ground's
/// own, one).
pub fn room_elasticity(P: &Params, year_now: i64) -> Option<f64> {
    let cp = &P.city;
    if !cp.dear_room || year_now <= cp.from || cp.room_elasticity == 1.0 {
        return None;
    }
    Some(cp.room_elasticity.clamp(0.05, 1.0))
}

// ---------------------------------------------------------------- the facts

/// The facts (`docs/rules/city.md`, "Facts"), from the record, so a run with
/// the rules off has them too: in 1900, 1950, 2000 and the last year, the largest town's
/// townsfolk, its share of all townsfolk, and the second town's over it.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (C, P, rec) = (&r.content, &r.params, &r.rec);
    let mut out = Vec::new();
    if !P.towns.enabled || rec.n_years == 0 {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + rec.n_years as i64 - 1;
    let T = rec.T;
    for y in crate::facts::marks(y0, y1).into_iter().filter(|&y| y >= 1900 && (y % 50 == 0 || y == y1)) {
        let k = (y - y0) as usize;
        let mut u: Vec<f64> = (0..T).filter(|&t| rec.town_active[k * T + t] != 0).map(|t| rec.town_urban[k * T + t] as f64).collect();
        u.sort_by(|a, b| b.total_cmp(a));
        let all: f64 = u.iter().sum();
        let first = u.first().copied().unwrap_or(f64::NAN);
        let second = u.get(1).copied().unwrap_or(f64::NAN);
        out.push((format!("townsfolk_largest_{y}"), Kind::Count, first));
        out.push((format!("townsfolk_largest_share_{y}"), Kind::Share, if all > 0.0 { first / all } else { f64::NAN }));
        out.push((format!("townsfolk_second_over_first_{y}"), Kind::Ratio, second / first));
    }
    out
}
