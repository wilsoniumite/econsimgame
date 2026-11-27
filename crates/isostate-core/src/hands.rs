//! A town cannot hire hands it does not have. Not in the Python
//! (`docs/rules/hands.md`; stage 2 of `docs/designs/rules-pass.md`, 4.1).
//!
//! Each month the port looks for a town's wage between half and twice last
//! month's, and never above 500. The land's hands move only `labor_adjust_ex`
//! (0.15) of the way to what their employers want, so most of a month's demand
//! is last month's hands, whatever the wage. When those carried hands alone are
//! more than the town has for hire, no wage in the bracket clears the market,
//! the search stops at the top, and everyone asked for is put to work and paid:
//! the town has more people at work than it has people, its wage doubles, and
//! next month it can double again.
//!
//! With `hands.ration` the hands a town has to spare (its people of working
//! age, less its carters, its sinkers and its navvies, as before) are shared
//! out. If at the highest wage the month allows the hands asked for are still
//! more than that, the wage is set at the highest wage and every employer in
//! the town, farm, wood, pit and works alike, is cut by the same share to the
//! hands there are. What an employer carries into next month is what it got,
//! not what it asked for, and that is what cures it: next month's demand is at
//! most the town's hands plus a little, and a wage a few tenths higher clears.
//! A town with nothing left after the others are taken out hires nobody.
//!
//! Nothing else changes: production, the materials asked for, the wages paid
//! and the rents follow from the hands got, as they follow from the hands
//! asked for in the port.

use serde::{Deserialize, Serialize};

use crate::params::Params;
use crate::state::State;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HandsParams {
    /// No employer holds a hand its town does not have: a town short of hands
    /// at the month's highest wage shares out the hands there are. Off (the
    /// port): everyone asked for is hired.
    pub ration: bool,
    /// The most a short town's wage may rise in a month, as a multiple of last
    /// month's (docs/rules/landfall.md, "The trap"). 2, the month's own bound,
    /// leaves the rule as it was: a town short several months running then
    /// doubles its wage each month. Both starts keep 2; the landfall start
    /// set 1.06 until carriage and pits were mended, and a run's params.json
    /// written then still names it. Below 2 a short town's wage rises at most this
    /// much a month, and the rationing alone shares out the hands: a knob for
    /// a study (`--set hands.short_rise=1.5`). Read only below 2, and not
    /// written out at 2, so that the game's parameters read as they did.
    #[serde(skip_serializing_if = "at_bound")]
    pub short_rise: f64,
}

/// `short_rise` at the month's own bound: the rule as it was.
fn at_bound(v: &f64) -> bool {
    *v == 2.0
}

impl Default for HandsParams {
    /// Off: what the port runs with.
    fn default() -> HandsParams {
        HandsParams { ration: false, short_rise: 2.0 }
    }
}

impl HandsParams {
    /// The game's labour market, which rations.
    pub fn game() -> HandsParams {
        HandsParams { ration: true, ..HandsParams::default() }
    }
}

/// After the month's wages are found: the towns that are short of hands even
/// at the highest wage the month allows (`hi`, a log wage), where `demand(t,
/// x)` is town `t`'s hands asked for at log wage `x`, as the search sums them.
/// Each such town's wage is set at that highest wage. Empty, and nothing asked,
/// while the rule is off.
pub fn short_towns(st: &State, P: &Params, hi: &[f64], supply: &[f64], demand: &dyn Fn(usize, f64) -> f64, w: &mut [f64]) -> Vec<bool> {
    if !P.hands.ration {
        return Vec::new();
    }
    let mut short = vec![false; w.len()];
    for t in 0..w.len() {
        if st.town_active[t] && demand(t, hi[t]) > supply[t] {
            short[t] = true;
            w[t] = hi[t].exp();
            // the guard (docs/rules/landfall.md, "The trap"): a short town's wage rises
            // at most `short_rise` a month; at 2 nothing is read, and the month is as it was
            if P.hands.short_rise < 2.0 && st.wage[t] > 0.0 {
                w[t] = w[t].min(st.wage[t] * P.hands.short_rise);
            }
        }
    }
    short
}

/// Cut every employer in a short town by the same share, to the hands the town
/// has to spare: the land's units (`ex_town`, `lex`) and the works (`f_town`,
/// `lf`), at the wage `short_towns` set. A town with nothing to spare hires
/// nobody. Does nothing when no town is short or the rule is off.
pub fn ration(short: &[bool], supply: &[f64], ex_town: &[usize], f_town: &[usize], lex: &mut [f64], lf: &mut [f64]) {
    if !short.iter().any(|&s| s) {
        return;
    }
    let mut asked = vec![0.0; short.len()];
    for (&t, &x) in ex_town.iter().zip(lex.iter()) {
        asked[t] += x;
    }
    for (&t, &x) in f_town.iter().zip(lf.iter()) {
        asked[t] += x;
    }
    let mut share = vec![1.0; short.len()];
    for t in 0..short.len() {
        let have = supply[t].max(0.0);
        if short[t] && asked[t] > have {
            share[t] = have / asked[t];
        }
    }
    for (&t, x) in ex_town.iter().zip(lex.iter_mut()) {
        *x *= share[t];
    }
    for (&t, x) in f_town.iter().zip(lf.iter_mut()) {
        *x *= share[t];
    }
}
