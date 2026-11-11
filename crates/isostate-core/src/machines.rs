//! Machines: bought where they pay, better with every ton of converter steel, and
//! new methods that start from what the trade already knows. Not in the Python;
//! `docs/rules/machines.md` is the longer account.
//!
//! The port gives the land three kinds of machine: pumps and wagonways in the
//! mines, and late in the century machinery on the farms. Each is bought per
//! unit of output whatever it costs, each adds a fixed amount for ever, and
//! quarries and clay pits get none. Three things follow, and this module answers
//! each of them:
//!
//! 1. *A machinery famine.* When farm machinery is found every farm buys it at
//!    once and at any price. Here a machine is taken on the share of output
//!    where it earns more than it costs at today's prices ([`uses`]), so
//!    adoption follows the price of machinery instead of driving it. And a
//!    machine stands for years ([`end_month`]): what it adds follows what was
//!    bought over those years, not what the market could deliver last month.
//! 2. *Nothing gets better after 1800.* Here machines have a learning curve of
//!    their own, on the steel the converters have poured ([`quality`]), in the
//!    same form as the methods' curves. A better machine adds more in the mine
//!    and on the farm, and in time is good enough to pay in a quarry or a clay
//!    pit.
//! 3. *A converter nobody builds.* A new way of making a good the island
//!    already makes starts from part of the skill of the old way
//!    ([`inherit_skill`]). The rest of that story is content: what a converter
//!    costs to build and what a steel rail is worth (`content::machines_content`).
//!
//! Switched off, none of these functions does anything and the state they keep
//! stays empty.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::mat::Mat;
use crate::params::Params;
use crate::state::State;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MachineParams {
    /// Machines are bought where they pay, improve with the steel poured, and a
    /// new method inherits part of its predecessor's skill.
    pub enabled: bool,
    /// How steeply adoption follows the return. A machine is taken on no output
    /// where it only earns its cost, on all of it where it earns its cost
    /// `1 + adopt_span` times over, and in proportion between.
    pub adopt_span: f64,
    /// Pumps and wagonways in the mines are bought by the same rule. False: as
    /// the port buys them, per unit of output whatever they cost.
    pub adopt_mines: bool,
    /// Months a machine stands. The machines at work move towards what is being
    /// bought by one part in this many each month.
    pub life_months: f64,
    /// The machines' own learning curve: quality = (1 + Q / quality_Q0)^quality_b,
    /// where Q is the steel made so far by every way of making steel found after
    /// the first (on this island, the converter).
    pub quality_Q0: f64,
    pub quality_b: f64,
    /// Quality stops here (as the methods stop at `learn_cap`).
    pub quality_cap: f64,
    /// What a machine of quality q adds to a quarry's or a clay pit's output:
    /// `quarry_reach * (q - 1)`. The engines of 1750 (q = 1) add nothing there.
    pub quarry_reach: f64,
    /// Machinery a quarry or clay pit buys per unit of output worked by machine.
    pub quarry_input: f64,
    /// A learned method starts from this share of the skill of an earlier
    /// learned method of making the same good (0 = from its own A0 only).
    pub inherit: f64,
}

impl Default for MachineParams {
    /// Off: what the port runs with.
    fn default() -> MachineParams {
        MachineParams {
            enabled: false,
            adopt_span: 1.0,
            adopt_mines: false,
            life_months: 48.0,
            quality_Q0: 250_000.0,
            quality_b: 0.3,
            quality_cap: 4.0,
            quarry_reach: 1.0,
            quarry_input: 0.002,
            inherit: 0.5,
        }
    }
}

impl MachineParams {
    /// The game's machines.
    pub fn game() -> MachineParams {
        MachineParams { enabled: true, ..MachineParams::default() }
    }
}

/// A method that starts clumsy (A0 below 1) and climbs a learning curve.
fn learned(C: &Content, m: usize) -> bool {
    C.m_b[m] > 0.0 && C.m_A0[m] < 1.0
}

/// The learned methods of making the same good that were found before method
/// `m` was (two found in the same year take the content's order). Empty until
/// `m` itself is found, and for a method that was the first of its kind.
fn predecessors<'a>(st: &'a State, C: &'a Content, m: usize) -> impl Iterator<Item = usize> + 'a {
    let found = learned(C, m) && st.tech_year[m] >= 0;
    (0..C.M).filter(move |&m2| {
        found && m2 != m && learned(C, m2) && C.method_out[m2] == C.method_out[m] && st.tech_year[m2] >= 0 && (st.tech_year[m2], m2) < (st.tech_year[m], m)
    })
}

/// How good machines are now, 1 being the machines of 1750.
///
/// Learning by doing, embodied in the machines. Crucible steel is poured by the
/// pound and goes into razors and files; a machine is built of steel only once
/// steel is made by the ton. So the experience counted is the steel made by
/// every method of steelmaking that had a predecessor (on this island, the
/// converter): each ton teaches the engine builders something about boilers,
/// shafts and cutting edges, as each unit a method makes teaches the method.
pub fn quality(st: &State, C: &Content, P: &Params) -> f64 {
    let M = &P.machines;
    if !M.enabled || M.quality_b <= 0.0 {
        return 1.0;
    }
    let Some(&steel) = C.good_index.get("steel") else { return 1.0 };
    let Q: f64 = (0..C.M).filter(|&m| C.method_out[m] == steel && predecessors(st, C, m).next().is_some()).map(|m| st.method_Q[m]).sum();
    (1.0 + Q / M.quality_Q0.max(1e-9)).powf(M.quality_b).min(M.quality_cap.max(1.0))
}

/// One way a land sector can use machines in one town, at given prices.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Use {
    /// "pump", "wagonway", "farm machinery" or "quarry machinery".
    pub kind: &'static str,
    /// Its column among the sector's uses in `State::machine_stock` (0 or 1).
    pub slot: usize,
    /// The good bought: machinery or rails.
    pub good: usize,
    /// What the machine adds to the output of those who have it, as a share.
    pub boost: f64,
    /// Units of the good bought per unit of output by those who keep it up.
    pub input: f64,
    /// Share of the sector's output worked by people who know how.
    pub know: f64,
    /// A unit of output's gain from the machine and the cost of the input for
    /// it, in money; zero for a use that is not bought by this rule.
    pub gain: f64,
    pub cost: f64,
    /// Share of those who know how that keep the machine up at these prices
    /// (1 for a use bought the port's way).
    pub share: f64,
    /// Bought by the rule (true) or the port's way, per unit of output whatever it costs.
    pub priced: bool,
}

/// At most this many machines per land sector.
pub const SLOTS: usize = 2;

impl Use {
    /// Share of the sector's output that keeps the machine up.
    pub fn kept(&self) -> f64 {
        self.know * self.share
    }

    /// Units of the good asked for per unit of the sector's output. The port
    /// charges a use it does not price in full, whatever the know-how.
    pub fn asked(&self) -> f64 {
        if self.priced { self.input * self.kept() } else { self.input }
    }
}

/// The machines sector `sec` can use in `town`, and how far each is taken at
/// these prices.
///
/// A machine's gain on a unit of output is the output's price times the share
/// of that output the machine is owed, `boost / (1 + all boosts)`; its cost is
/// the input for that unit. Farms and pits differ (the size of a field, the
/// depth of a seam), so they do not all cross from "does not pay" to "pays" at
/// once: the share that takes the machine rises from none where gain equals
/// cost to all where gain is `1 + adopt_span` times cost.
pub fn uses(st: &State, C: &Content, P: &Params, sec: usize, town: usize, price: &Mat) -> Vec<Use> {
    let M = &P.machines;
    let (gm, gr) = (C.gi("machinery"), C.gi("rails"));
    let name = C.sector_name[sec].as_str();
    let k = &st.know;
    let (kp, kw, kf) = (k[(town, st.t_pump)], k[(town, st.t_wagon)], k[(town, st.t_fmach)]);
    let q = quality(st, C, P);
    let mut out: Vec<Use> = Vec::with_capacity(SLOTS);
    let mut add = |kind: &'static str, slot: usize, good: usize, boost: f64, input: f64, know: f64, priced: bool| {
        if know > 0.05 && boost > 0.0 {
            out.push(Use { kind, slot, good, boost, input, know, gain: 0.0, cost: 0.0, share: 1.0, priced });
        }
    };
    if C.sector_mine[sec] {
        add("pump", 0, gm, P.pump_boost * q, P.pump_input, kp, M.adopt_mines);
        add("wagonway", 1, gr, P.wagon_boost * q, P.wagon_input, kw, M.adopt_mines);
    }
    if name == "farm" {
        add("farm machinery", 0, gm, P.farm_mach_boost * q, P.farm_mach_input, kf, true);
    }
    if name == "quarry" || name == "claypit" {
        // the steam engine is what a quarry's cranes and a pit's pug mills run on
        add("quarry machinery", 0, gm, M.quarry_reach * (q - 1.0), M.quarry_input, kp, true);
    }
    // a pit or farm with every machine it knows: what each one is owed of its output
    let all = 1.0 + out.iter().map(|u| u.boost).sum::<f64>();
    let p_out = price[(town, C.sector_good[sec])];
    for u in out.iter_mut().filter(|u| u.priced) {
        u.gain = p_out * u.boost / all;
        u.cost = u.input * price[(town, u.good)];
        u.share = if u.cost > 0.0 { ((u.gain / u.cost - 1.0) / M.adopt_span.max(1e-9)).clamp(0.0, 1.0) } else { 1.0 };
    }
    out
}

/// `econ::sector_tfp` with the machines switched on: `mult` is the sector's
/// multiplier before any machine (tools, husbandry).
///
/// What the machines add comes from the machines standing (`State::machine_stock`);
/// what is bought comes from who keeps theirs up at these prices.
///
/// Returns what `sector_tfp` returns: (mult, machinery per unit, rails per unit, input cost per unit).
pub fn sector_tfp(st: &State, C: &Content, P: &Params, sec: usize, town: usize, price: &Mat, mult: f64) -> (f64, f64, f64, f64) {
    let (gm, gr) = (C.gi("machinery"), C.gi("rails"));
    let (mut boost, mut mach, mut rails) = (0.0, 0.0, 0.0);
    for u in uses(st, C, P, sec, town, price) {
        // before the first month has been counted: the machines they would keep
        let standing = if st.machine_stock.rows > 0 { st.machine_stock[(town, sec * SLOTS + u.slot)] } else { u.know * u.share };
        boost += u.boost * standing;
        if u.good == gm {
            mach += u.asked();
        } else {
            rails += u.asked();
        }
    }
    let unit_cost = mach * price[(town, gm)] + rails * price[(town, gr)];
    (mult * (1.0 + boost), mach, rails, unit_cost)
}

/// The machines standing, moved a month on: in every town and land sector the
/// share of output worked by machine goes one part in `life_months` of the way
/// to the share that kept its machines up this month and was supplied
/// (`st.fill`). A month the engine works cannot deliver does not flood the pits;
/// years of it do. Called at the end of `econ::step_month`.
pub fn end_month(st: &mut State, C: &Content, P: &Params) {
    let M = &P.machines;
    if !M.enabled {
        return;
    }
    let (T, NS) = (st.T_max, C.NS);
    let first = st.machine_stock.rows != T || st.machine_stock.cols != NS * SLOTS;
    if first {
        st.machine_stock = Mat::zeros(T, NS * SLOTS);
    }
    let step = if first { 1.0 } else { 1.0 / M.life_months.max(1.0) };
    for town in 0..T {
        for sec in 0..NS {
            let now = uses(st, C, P, sec, town, &st.pexp);
            for slot in 0..SLOTS {
                let target = now.iter().find(|u| u.slot == slot).map_or(0.0, |u| u.know * u.share * st.fill[(town, u.good)]);
                let s = &mut st.machine_stock[(town, sec * SLOTS + slot)];
                *s += step * (target - *s);
            }
        }
    }
}

/// A learned method starts from part of the skill of the learned methods of the
/// same good that were found before it: the first converters are run by people
/// who have poured crucible steel for two generations. `A` is each method's own
/// learning, as `firms::learning` gives it, and is raised in place.
pub fn inherit_skill(st: &State, C: &Content, P: &Params, A: &mut [f64]) {
    let share = P.machines.inherit;
    if !P.machines.enabled || share <= 0.0 {
        return;
    }
    let own = A.to_vec();
    for m in 0..C.M {
        for m2 in predecessors(st, C, m) {
            A[m] = A[m].max(share * own[m2]);
        }
    }
}
