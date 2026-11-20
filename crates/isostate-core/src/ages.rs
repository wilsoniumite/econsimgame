//! The people's ages, reckoned under the totals the island plays. Not in the
//! Python; it only watches. `docs/rules/ages.md` is the longer account.
//!
//! A town's people are one number in the game, and that number is what the
//! game's rules of births, crowding, dearth, sailings, migrants and districts
//! make it. This keeps beneath it seventeen five-year groups (0 to 4, 5 to 9,
//! and so on to 80 and over) whose sum is that number in every town at every
//! year's end, so that a pension, a school or a hand can be counted by age:
//!
//! - **Deaths** by group come from yearly hazards by age that fall with food
//!   (the town's baskets a head) and with medicine. The content has no
//!   inventions of medicine yet, so medicine is a stand-in: known by the year,
//!   as history found it, and reached by a town as far as its food is full
//!   (`MEDICINE`). Crowding's deaths are the played rule's, shared out by
//!   `toll`; a dearth's are the played rule's, twice as heavy under five and
//!   over 64.
//! - **A fifth of each group moves up** a year; the last group is open.
//! - **Births are the remainder**: what the played total needs once the
//!   deaths, the leavers and the moves are counted, into the youngest group.
//!   Where that would be less than nothing, the shortfall is taken as more
//!   people leaving, drawn as movers are, if the played rule says people took
//!   ship from the town that year, and as more deaths otherwise (`short_*`).
//! - **People who move take their ages**: emigrants and migrants between towns
//!   are drawn `movers` times as heavily from ages 15 to 34; a new town's first
//!   people likewise; a district that changes market goes whole, as it is.
//! - **Schooling** is carried by cohort: those who turn fifteen take the years
//!   of school their town's income (about the last ten years') bought, and keep
//!   them as they age and move.
//!
//! At the start each town's row is the stable shape of the first year's
//! hazards and of its played rate, scaled to its people.
//!
//! Nothing in the economy reads `State::ages` and no random number is drawn:
//! a run with it on is the same run as one with it off. Its series are counted
//! among the books' own (`accounts::own_series`).

use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::facts::Kind;
use crate::mat::Mat;
use crate::observer::Series;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;

/// Five-year groups: 0 to 4, ..., 75 to 79, and 80 and over.
pub const GROUPS: usize = 17;

/// Under fifteen; fifteen to 64; 65 and over (as places in a row).
pub const YOUNG: Range<usize> = 0..3;
pub const WORKING: Range<usize> = 3..13;
pub const OLD: Range<usize> = 13..17;

/// The groups movers are drawn from more heavily: 15 to 34.
pub const MOVERS: Range<usize> = 3..7;

/// The groups a dearth falls on more heavily: under five, and 65 and over.
fn frail(a: usize) -> bool {
    a == 0 || a >= OLD.start
}

/// Each group's hazard over its broad group's (children, working ages, the
/// old): the first five years and the last are the dangerous ones, and the
/// years of middle age more so than youth.
pub const PROFILE: [f64; GROUPS] = [2.4, 0.3, 0.3, 0.6, 0.6, 0.6, 0.6, 0.6, 0.6, 1.6, 1.6, 1.6, 1.6, 0.6, 0.9, 1.5, 2.6];

/// The broad group of a five-year group: 0 children, 1 working ages, 2 the old.
pub fn broad(a: usize) -> usize {
    if a < WORKING.start {
        0
    } else if a < OLD.start {
        1
    } else {
        2
    }
}

/// A stand-in for medicine, until the content has its inventions: three
/// waves, each `(the year half the island knows it, the years it takes to
/// spread, how far it lowers the hazard of children, working ages and the old)`.
/// Sanitary reform and vaccines; antisepsis and antibiotics; modern medicine.
/// A town reaches a wave as far as its food is full, so a poor town gains
/// less of it.
pub const MEDICINE: [(f64, f64, [f64; 3]); 3] =
    [(1900.0, 50.0, [1.25, 0.45, 0.05]), (1945.0, 20.0, [1.2, 0.55, 0.15]), (1990.0, 60.0, [0.6, 0.7, 0.85])];

/// How much of a wave of medicine the island knows in a year, 0 to 1.
pub fn known(year: i64, mid: f64, width: f64) -> f64 {
    1.0 / (1.0 + (-4.0 * (year as f64 - mid) / width).exp())
}

/// Years of school a child gets at `y` baskets a head: two at a bare living,
/// five where eight baskets a head buy three more (until a state pays for schools).
pub fn school_years(y: f64) -> f64 {
    2.0 + 3.0 * ((y - 3.0) / 5.0).clamp(0.0, 1.0)
}

/// The years over which a town's income is averaged for its children's school.
pub const SCHOOL_YEARS_BACK: f64 = 10.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AgesParams {
    /// Reckon the people's ages beneath the played totals and keep them as
    /// series and facts. It never touches the economy.
    pub watch: bool,
    /// Yearly hazard of the children, the working ages and the old at a bare
    /// living with nothing known of medicine (times `PROFILE` by group).
    pub hazard: [f64; 3],
    /// How far full food lowers each broad group's hazard: by `exp(-food * this)`.
    pub food: [f64; 3],
    /// Food is `ln(y / y_still) / ln(food_full)`, from nothing at the people's
    /// resting income to full at this many times it.
    pub food_full: f64,
    /// Each broad group's part of crowding's deaths, against the played rule's
    /// toll a head.
    pub toll: [f64; 3],
    /// How many times as heavily movers (emigrants, migrants, a new town's first
    /// people) are drawn from 15 to 34 as from the other ages.
    pub movers: f64,
    /// How many times as heavily a dearth's deaths fall under five and over 64.
    pub dearth: f64,
}

impl Default for AgesParams {
    /// Off: what the port runs with.
    fn default() -> AgesParams {
        AgesParams { watch: false, hazard: [0.033, 0.0165, 0.105], food: [0.75, 0.40, 0.05], food_full: 6.0, toll: [1.6, 0.7, 1.0], movers: 3.0, dearth: 2.0 }
    }
}

impl AgesParams {
    /// The game's watcher.
    pub fn game() -> AgesParams {
        AgesParams { watch: true, ..AgesParams::default() }
    }
}

/// The year's sums, from the year's step to its close.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct YearSums {
    /// People in each group as the year's step began.
    pub people: [f64; GROUPS],
    /// Deaths in each group (by hazard, dearth and a shortfall taken as deaths).
    pub deaths: [f64; GROUPS],
    pub births: f64,
    /// Those who took ship, and a shortfall taken as more of them.
    pub left: f64,
    /// The island's people after the year's step.
    pub after: f64,
    /// Towns whose remainder was less than nothing, and the people it lacked;
    /// how many of them were taken as leavers.
    pub short_towns: usize,
    pub short: f64,
    pub short_left: f64,
}

/// One year of the reckoning, for the island.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct AgesYear {
    pub year: i64,
    pub people: f64,
    /// Each group's people at the year's end.
    pub groups: Vec<f64>,
    pub births_k: f64,
    pub deaths_k: f64,
    /// Life at birth at the year's hazards (deaths over people, group by group).
    pub life: f64,
    pub under15: f64,
    pub working: f64,
    pub over64: f64,
    /// Mean years of school of the people aged 15 to 64.
    pub school: f64,
    pub short_towns: usize,
    pub short: f64,
    pub short_left: f64,
}

/// What the reckoning keeps between years (`st.ages`). Empty while it is off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AgesState {
    /// Each town's people in the seventeen groups.
    pub rows: Vec<[f64; GROUPS]>,
    /// Each group's mean years of school (nought below fifteen, not yet reckoned).
    pub school: Vec<[f64; GROUPS]>,
    /// Each town's baskets a head, averaged over about `SCHOOL_YEARS_BACK` years.
    pub ybar: Vec<f64>,
    /// This year's emigrants from each town (`peopling::growth`).
    pub left: Vec<f64>,
    /// The year in hand.
    pub now: Option<YearSums>,
    pub years: Vec<AgesYear>,
    /// The worst mismatch between a row and its town's people found at a year's
    /// end before the rows were put right, relative: nought to rounding when
    /// every way people go has been followed.
    pub drift: f64,
    /// Rows given the island's shape at a year's end because their town had
    /// people and the row had none (a way people came that is not followed).
    pub filled: usize,
}

/// Each group's yearly hazard in a town with `y` baskets a head, played
/// crowding deaths a head `crowd`, in `year`.
pub fn hazards(ap: &AgesParams, y_still: f64, y: f64, crowd: f64, year: i64) -> [f64; GROUPS] {
    let food = ((y.max(1e-3) / y_still).ln() / ap.food_full.max(1.0001).ln()).clamp(0.0, 1.0);
    let mut hz = [0.0; 3];
    for g in 0..3 {
        let med: f64 = MEDICINE.iter().map(|&(mid, width, k)| k[g] * known(year, mid, width) * food).sum();
        hz[g] = ap.hazard[g] * (-(ap.food[g] * food + med)).exp() + ap.toll[g] * crowd.max(0.0);
    }
    let mut h = [0.0; GROUPS];
    for a in 0..GROUPS {
        h[a] = (PROFILE[a] * hz[broad(a)]).min(0.9);
    }
    h
}

/// Life at birth at hazards `h` (a life table of five-year groups, the last open).
pub fn life_at_birth(h: &[f64; GROUPS]) -> f64 {
    let (mut e0, mut alive) = (0.0, 1.0);
    for &ha in &h[..GROUPS - 1] {
        let s = (-5.0 * ha).exp();
        e0 += alive * 5.0 * (1.0 + s) / 2.0;
        alive *= s;
    }
    e0 + alive / h[GROUPS - 1].max(1e-6)
}

/// The stable shape of hazards `h` for a people growing `rate` a year, one in all.
pub fn stable(h: &[f64; GROUPS], rate: f64) -> [f64; GROUPS] {
    let lam = 1.0 + rate;
    let mut n = [0.0; GROUPS];
    n[0] = 1.0;
    for a in 1..GROUPS {
        let stay = if a < GROUPS - 1 { 0.8 * (1.0 - h[a]) } else { 1.0 - h[a] };
        n[a] = n[a - 1] * 0.2 * (1.0 - h[a - 1]) / (lam - stay).max(1e-3);
    }
    let s: f64 = n.iter().sum();
    n.map(|v| v / s)
}

/// `m` people drawn from row `r`, `heavy` times as heavily from `from` as from
/// the rest; never more of a group than it has.
pub fn draw(r: &[f64; GROUPS], m: f64, heavy: f64, from: Range<usize>) -> [f64; GROUPS] {
    let all: f64 = r.iter().sum();
    if m <= 0.0 || all <= 0.0 {
        return [0.0; GROUPS];
    }
    if m >= all {
        return *r;
    }
    let w = |a: usize| if from.contains(&a) { heavy } else { 1.0 };
    let tot: f64 = (0..GROUPS).map(|a| w(a) * r[a].max(0.0)).sum();
    let mut took = [0.0; GROUPS];
    let mut got = 0.0;
    for a in 0..GROUPS {
        took[a] = (m * w(a) * r[a].max(0.0) / tot).min(r[a].max(0.0));
        got += took[a];
    }
    // what the heavy groups could not give comes from the rest in proportion
    let rest = m - got;
    if rest > 0.0 && all - got > 0.0 {
        for a in 0..GROUPS {
            took[a] += rest * (r[a].max(0.0) - took[a]) / (all - got);
        }
    }
    took
}

fn sum(r: &[f64; GROUPS]) -> f64 {
    r.iter().sum()
}

/// The played crowding deaths a head in town `t` (the game's people, less
/// what drains and clean water save under the transition).
fn crowd(st: &State, P: &Params, t: usize, y: f64) -> f64 {
    let c = crate::peopling::crowding_term(&P.people, st.urban_pop[t].max(0.0), st.pop[t].max(1e-9));
    let s = if P.transition.enabled { crate::transition::drains(&P.transition, y) } else { 0.0 };
    c * (1.0 - s)
}

/// The rate a town's stable shape at the start is made for: the played
/// income term (births less deaths from income alone).
fn start_rate(P: &Params, y: f64) -> f64 {
    if P.people.enabled { crate::peopling::income_term(&P.people, y) } else { 0.005 }
}

/// Every town's row at the start: the stable shape of its first hazards,
/// scaled to its people (`sim::start_from`, after the warm-up, with each town's
/// baskets a head). Nothing while the watcher is off.
pub fn start(st: &mut State, P: &Params, y: &[f64], year: i64) {
    if !P.ages.watch {
        return;
    }
    let T = st.T_max;
    let a = &mut st.ages;
    a.rows = vec![[0.0; GROUPS]; T];
    a.school = vec![[0.0; GROUPS]; T];
    a.ybar = (0..T).map(|t| y.get(t).copied().unwrap_or(0.0)).collect();
    a.left = vec![0.0; T];
    for t in 0..T {
        let n = if st.town_active[t] { st.pop[t].max(0.0) } else { 0.0 };
        if n <= 0.0 {
            continue;
        }
        let yt = y.get(t).copied().unwrap_or(1.0);
        let h = hazards(&P.ages, P.people.y_still, yt, crowd(st, P, t, yt), year);
        let shape = stable(&h, start_rate(P, yt));
        st.ages.rows[t] = shape.map(|v| v * n);
        let sch = school_years(yt);
        for g in WORKING.start..GROUPS {
            st.ages.school[t][g] = sch;
        }
    }
}

/// The emigrants of the year from each town, as `peopling::growth` reckons them.
pub fn sailed(st: &mut State, P: &Params, left: &[f64]) {
    if P.ages.watch {
        st.ages.left = left.to_vec();
    }
}

/// The year, town by town (`people::growth`, once the played rate has met
/// `st.pop`, before the year's food and income are cleared): the leavers,
/// the deaths, a fifth of each group up, and the births that make the played
/// total. `y` is each town's baskets a head.
pub fn year(st: &mut State, P: &Params, y: &[f64], year: i64) {
    if !P.ages.watch {
        return;
    }
    let T = st.T_max;
    if st.ages.rows.len() != T {
        start(st, P, y, year);
    }
    let ap = &P.ages;
    let mut sums = YearSums::default();
    let left_now = std::mem::take(&mut st.ages.left);
    for t in 0..T {
        let n1 = if st.town_active[t] { st.pop[t].max(0.0) } else { 0.0 };
        let yt = y.get(t).copied().unwrap_or(0.0);
        if st.town_active[t] {
            let yb = &mut st.ages.ybar[t];
            *yb += (yt - *yb) / SCHOOL_YEARS_BACK;
        }
        let mut r = st.ages.rows[t];
        let s = st.ages.school[t];
        let n0 = sum(&r);
        sums.after += n1;
        if n0 <= 0.0 {
            // nobody to reckon from (a town that gained its first people some way not
            // followed): its year's end gives it the island's shape
            continue;
        }
        for a in 0..GROUPS {
            sums.people[a] += r[a];
        }
        // those who took ship
        let left = left_now.get(t).copied().unwrap_or(0.0).clamp(0.0, n0);
        if left > 0.0 {
            let took = draw(&r, left, ap.movers, MOVERS);
            for a in 0..GROUPS {
                r[a] -= took[a];
            }
            sums.left += left;
        }
        // deaths by hazard, and the played dearth's, heavier on the frail
        let h = hazards(ap, P.people.y_still, yt, crowd(st, P, t, yt), year);
        let mut dead = [0.0; GROUPS];
        for a in 0..GROUPS {
            dead[a] = (h[a] * r[a]).min(r[a]).max(0.0);
        }
        if P.people.enabled {
            let x = P.people.dearth * crate::peopling::dearth_depth(&P.people, st.food_min_acc[t]) * n0;
            let rest: [f64; GROUPS] = std::array::from_fn(|a| r[a] - dead[a]);
            let more = draw_frail(&rest, x, ap.dearth);
            for a in 0..GROUPS {
                dead[a] += more[a];
            }
        }
        // a fifth of each group up; those turning fifteen take their schooling:
        // what their families bought, or what the treasury paid for while they
        // were at school (docs/rules/state.md), whichever is more
        let sch = school_years(st.ages.ybar[t]).max(crate::purse::cohort_years(st));
        let (mut new, mut ns) = ([0.0; GROUPS], [0.0; GROUPS]);
        for a in 0..GROUPS {
            let stay = (r[a] - dead[a]).max(0.0);
            let up = if a < GROUPS - 1 { stay / 5.0 } else { 0.0 };
            new[a] += stay - up;
            ns[a] += (stay - up) * s[a];
            if a < GROUPS - 1 {
                new[a + 1] += up;
                ns[a + 1] += up * if a + 1 == WORKING.start { sch } else { s[a] };
            }
        }
        let school: [f64; GROUPS] = std::array::from_fn(|a| if new[a] > 0.0 { ns[a] / new[a] } else { s[a] });
        // births are the remainder
        let b = n1 - sum(&new);
        if b >= 0.0 {
            new[0] += b;
            sums.births += b;
        } else {
            let lack = (-b).min(sum(&new));
            sums.short_towns += 1;
            sums.short += lack;
            let took = if left > 0.0 {
                sums.short_left += lack;
                sums.left += lack;
                draw(&new, lack, ap.movers, MOVERS)
            } else {
                let took = draw_frail(&new, lack, ap.dearth);
                for a in 0..GROUPS {
                    dead[a] += took[a];
                }
                took
            };
            for a in 0..GROUPS {
                new[a] = (new[a] - took[a]).max(0.0);
            }
        }
        for a in 0..GROUPS {
            sums.deaths[a] += dead[a];
        }
        // held to the played total exactly (the steps above lose only rounding)
        let got = sum(&new);
        if got > 0.0 {
            new = new.map(|v| v * n1 / got);
        }
        st.ages.rows[t] = new;
        st.ages.school[t] = school;
    }
    st.ages.left = vec![0.0; T];
    st.ages.now = Some(sums);
}

/// `m` deaths from row `r`, `heavy` times as heavily under five and over 64.
fn draw_frail(r: &[f64; GROUPS], m: f64, heavy: f64) -> [f64; GROUPS] {
    // draw with the heavy groups as "from": the frail are not a range, so weigh by hand
    let all = sum(r);
    if m <= 0.0 || all <= 0.0 {
        return [0.0; GROUPS];
    }
    if m >= all {
        return *r;
    }
    let w = |a: usize| if frail(a) { heavy } else { 1.0 };
    let tot: f64 = (0..GROUPS).map(|a| w(a) * r[a].max(0.0)).sum();
    let mut took = [0.0; GROUPS];
    let mut got = 0.0;
    for a in 0..GROUPS {
        took[a] = (m * w(a) * r[a].max(0.0) / tot).min(r[a].max(0.0));
        got += took[a];
    }
    let rest = m - got;
    if rest > 0.0 && all - got > 0.0 {
        for a in 0..GROUPS {
            took[a] += rest * (r[a].max(0.0) - took[a]) / (all - got);
        }
    }
    took
}

/// Add `came` people with schooling `came_school` to town `j`'s row.
fn arrive(st: &mut State, j: usize, came: &[f64; GROUPS], came_school: &[f64; GROUPS]) {
    let (r, s) = (&mut st.ages.rows[j], &mut st.ages.school[j]);
    for a in 0..GROUPS {
        let n = r[a] + came[a];
        if n > 0.0 {
            s[a] = (r[a] * s[a] + came[a] * came_school[a]) / n;
        }
        r[a] = n;
    }
}

/// The year's migrants take their ages and schooling (`people::migration`,
/// before the people are moved). `flow[(i, j)]` is from `act[i]` to `act[j]`.
pub fn migrate(st: &mut State, P: &Params, act: &[usize], flow: &Mat) {
    if !P.ages.watch || st.ages.rows.len() != st.T_max {
        return;
    }
    let n = act.len();
    let was: Vec<[f64; GROUPS]> = act.iter().map(|&t| st.ages.rows[t]).collect();
    let was_school: Vec<[f64; GROUPS]> = act.iter().map(|&t| st.ages.school[t]).collect();
    // each town's movers, by age, one in all
    let mut shape = vec![[0.0; GROUPS]; n];
    for k in 0..n {
        let out: f64 = (0..n).map(|j| flow[(k, j)]).sum();
        if out <= 0.0 {
            continue;
        }
        let took = draw(&was[k], out, P.ages.movers, MOVERS);
        let got = sum(&took);
        if got > 0.0 {
            shape[k] = took.map(|v| v / got);
        }
        let r = &mut st.ages.rows[act[k]];
        for a in 0..GROUPS {
            r[a] = (r[a] - took[a]).max(0.0);
        }
    }
    for j in 0..n {
        for i in 0..n {
            let m = flow[(i, j)];
            if i == j || m <= 0.0 {
                continue;
            }
            let came = shape[i].map(|v| v * m);
            arrive(st, act[j], &came, &was_school[i]);
        }
    }
}

/// The people of the districts that changed market take their ages
/// (`peopling::move_with_cells`, after the people are moved). `pop` is the
/// people before; `share[(i, j)]` the part of town i's people who went to town
/// j. A district goes whole, as it is; a new town's first people are drawn as
/// movers are.
pub fn move_with_cells(st: &mut State, P: &Params, pop: &[f64], share: &Mat) {
    if !P.ages.watch || st.ages.rows.len() != st.T_max {
        return;
    }
    let T = st.T_max;
    let was = st.ages.rows.clone();
    let was_school = st.ages.school.clone();
    let mut moves: Vec<(usize, usize, [f64; GROUPS])> = Vec::new();
    for i in 0..T {
        let n0 = sum(&was[i]);
        if n0 <= 0.0 {
            continue;
        }
        // first the new towns' people, drawn as movers; then the districts, from the rest
        let to_new: f64 = (0..T).filter(|&j| j != i && st.town_new[j]).map(|j| pop[i] * share[(i, j)]).sum();
        let drawn = draw(&was[i], to_new.min(n0), P.ages.movers, MOVERS);
        let rest: [f64; GROUPS] = std::array::from_fn(|a| (was[i][a] - drawn[a]).max(0.0));
        let (dn, rn) = (sum(&drawn), sum(&rest));
        for j in 0..T {
            let m = pop[i] * share[(i, j)];
            if j == i || m <= 0.0 {
                continue;
            }
            let came = if st.town_new[j] {
                if dn > 0.0 { drawn.map(|v| v * m / dn) } else { [0.0; GROUPS] }
            } else if rn > 0.0 {
                rest.map(|v| v * m / rn)
            } else {
                [0.0; GROUPS]
            };
            moves.push((i, j, came));
        }
    }
    for (i, j, came) in moves {
        let r = &mut st.ages.rows[i];
        for a in 0..GROUPS {
            r[a] = (r[a] - came[a]).max(0.0);
        }
        arrive(st, j, &came, &was_school[i]);
    }
}

/// The island's people by group and their mean schooling by group.
fn island(st: &State) -> ([f64; GROUPS], [f64; GROUPS]) {
    let (mut n, mut ns) = ([0.0; GROUPS], [0.0; GROUPS]);
    for (r, s) in st.ages.rows.iter().zip(&st.ages.school) {
        for a in 0..GROUPS {
            n[a] += r[a];
            ns[a] += r[a] * s[a];
        }
    }
    let school = std::array::from_fn(|a| if n[a] > 0.0 { ns[a] / n[a] } else { 0.0 });
    (n, school)
}

/// The mean years of school of the people aged 15 to 64 in a row.
pub fn working_school(r: &[f64; GROUPS], s: &[f64; GROUPS]) -> f64 {
    let n: f64 = r[WORKING].iter().sum();
    let ns: f64 = WORKING.map(|a| r[a] * s[a]).sum();
    if n > 0.0 { ns / n } else { f64::NAN }
}

/// A town's people by the three ages, and its working ages' schooling.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct TownAges {
    pub under15: f64,
    pub working: f64,
    pub over64: f64,
    /// Mean years of school of those aged 15 to 64.
    pub school: f64,
}

/// Town `t`'s ages as the last year's end left them (None while the watcher
/// is off or the town has nobody). The state's purse reads this.
pub fn town(st: &State, t: usize) -> Option<TownAges> {
    let r = st.ages.rows.get(t)?;
    if sum(r) <= 0.0 {
        return None;
    }
    let s = &st.ages.school[t];
    Some(TownAges { under15: r[YOUNG].iter().sum(), working: r[WORKING].iter().sum(), over64: r[OLD].iter().sum(), school: working_school(r, s) })
}

/// The rule's series, after the others so that theirs keep their places: the
/// island's people in each group, births and deaths a thousand, life at birth,
/// the three shares and the working ages' schooling, a year's figure in its
/// twelve months.
pub fn add_series(reg: &mut Series, P: &Params, n_months: usize) {
    if !P.ages.watch {
        return;
    }
    for a in 0..GROUPS {
        let label = if a + 1 < GROUPS { format!("People aged {} to {}, reckoned", 5 * a, 5 * a + 4) } else { format!("People aged {} and over, reckoned", 5 * a) };
        reg.add(&group_name(a), &label, "ages", "people", n_months);
    }
    reg.add("ages_births_k", "Births a thousand people, reckoned", "ages", "a thousand", n_months);
    reg.add("ages_deaths_k", "Deaths a thousand people, reckoned", "ages", "a thousand", n_months);
    reg.add("ages_life", "Life at birth, reckoned", "ages", "years", n_months);
    reg.add("ages_under15", "Under 15, reckoned", "ages", "share", n_months);
    reg.add("ages_working", "Aged 15 to 64, reckoned", "ages", "share", n_months);
    reg.add("ages_over64", "Over 64, reckoned", "ages", "share", n_months);
    reg.add("ages_school", "Years of school, aged 15 to 64, reckoned", "ages", "years", n_months);
}

/// How many series the watcher keeps: the seventeen groups and seven more.
pub const N_SERIES: usize = GROUPS + 7;

/// `ages_g00` to `ages_g16`: the series of the island's people in group `a`.
pub fn group_name(a: usize) -> String {
    format!("ages_g{a:02}")
}

/// At a year's end (`sim::end_year`): every row held to its town's people,
/// and the year's figures kept and written into their twelve months.
pub fn close_year(st: &mut State, C: &Content, P: &Params, year: i64) {
    if !P.ages.watch {
        return;
    }
    let T = st.T_max;
    if st.ages.rows.len() != T {
        let y = st.y_ratio.clone();
        start(st, P, &y, year);
    }
    let (shape, shape_school) = {
        let (n, s) = island(st);
        let all = sum(&n);
        (if all > 0.0 { n.map(|v| v / all) } else { [1.0 / GROUPS as f64; GROUPS] }, s)
    };
    let mut worst: f64 = 0.0;
    for t in 0..T {
        let n = if st.town_active[t] { st.pop[t].max(0.0) } else { 0.0 };
        let s = sum(&st.ages.rows[t]);
        if n <= 0.0 {
            worst = worst.max(s / st.pop.iter().sum::<f64>().max(1.0));
            st.ages.rows[t] = [0.0; GROUPS];
            st.ages.school[t] = [0.0; GROUPS];
        } else if s <= 0.0 {
            st.ages.filled += 1;
            st.ages.rows[t] = shape.map(|v| v * n);
            st.ages.school[t] = shape_school;
        } else {
            worst = worst.max((s / n - 1.0).abs());
            st.ages.rows[t] = st.ages.rows[t].map(|v| v * n / s);
        }
    }
    st.ages.drift = st.ages.drift.max(worst);
    let sums = st.ages.now.take().unwrap_or_default();
    let (n, sch) = island(st);
    let people = sum(&n);
    let before = sum(&sums.people);
    let mid = 0.5 * (before + sums.after);
    let k = |v: f64| if mid > 0.0 { 1000.0 * v / mid } else { f64::NAN };
    let h: [f64; GROUPS] = std::array::from_fn(|a| if sums.people[a] > 0.0 { sums.deaths[a] / sums.people[a] } else { 0.0 });
    let share = |rg: Range<usize>| if people > 0.0 { n[rg].iter().sum::<f64>() / people } else { f64::NAN };
    let school = working_school(&n, &sch);
    let row = AgesYear {
        year,
        people,
        groups: n.to_vec(),
        births_k: k(sums.births),
        deaths_k: k(sum(&sums.deaths)),
        life: if before > 0.0 { life_at_birth(&h) } else { f64::NAN },
        under15: share(YOUNG),
        working: share(WORKING),
        over64: share(OLD),
        school,
        short_towns: sums.short_towns,
        short: sums.short,
        short_left: sums.short_left,
    };
    if st.series.index.contains_key("ages_school") {
        let n_months = st.series.data.first().map_or(0, |d| d.len());
        let first = ((year - C.start_year) * 12).max(0) as usize;
        for t in first..(first + 12).min(n_months) {
            for a in 0..GROUPS {
                st.series.set(&group_name(a), t, n[a]);
            }
            st.series.set("ages_births_k", t, row.births_k);
            st.series.set("ages_deaths_k", t, row.deaths_k);
            st.series.set("ages_life", t, row.life);
            st.series.set("ages_under15", t, row.under15);
            st.series.set("ages_working", t, row.working);
            st.series.set("ages_over64", t, row.over64);
            st.series.set("ages_school", t, row.school);
        }
    }
    st.ages.years.push(row);
}

/// The year's figures of the rule's series, by their places, for a live
/// game's year (`live::LiveYear::year_series`); none while it is off.
pub fn year_series(st: &State, C: &Content, year: i64) -> Vec<(usize, f64)> {
    let Some(&last) = st.series.index.get("ages_school") else { return Vec::new() };
    let t = ((year - C.start_year) * 12).max(0) as usize;
    let first = st.series.index["ages_g00"];
    (first..=last).filter_map(|i| st.series.data[i].get(t).map(|&v| (i, v))).collect()
}

/// The years the facts are read at, with the run's last.
pub const MARKS: [i64; 5] = [1800, 1850, 1900, 1950, 2000];

/// The facts (docs/rules/ages.md), under "People and towns": at `MARKS` inside
/// the run and its last year, `ages_births_k_Y`, `ages_deaths_k_Y` and
/// `ages_life_Y`, each the mean of the nine years round Y (`linked::nine_years`),
/// and that year's `ages_under15_Y`, `ages_over64_Y` and `ages_school_Y`;
/// over the whole run `ages_short_town_years` (towns whose remainder was less
/// than nothing, counted once a year each), `ages_short_people` (what they
/// lacked), `ages_short_left_share` (the part of it taken as leavers) and
/// `ages_drift` (the worst mismatch found at a year's end). Nothing in a run
/// that did not keep them.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let st = &r.state;
    let mut out = Vec::new();
    let ys = &st.ages.years;
    if ys.is_empty() {
        return out;
    }
    let (y0, y1) = (ys[0].year, ys[ys.len() - 1].year);
    let at = |y: i64| ys.iter().find(|v| v.year == y);
    let mean = |y: i64, f: &dyn Fn(&AgesYear) -> f64| -> f64 {
        let (a, z) = crate::linked::nine_years(y0, y1, y);
        let xs: Vec<f64> = ys.iter().filter(|v| v.year >= a && v.year <= z).map(f).filter(|x| x.is_finite()).collect();
        if xs.is_empty() { f64::NAN } else { xs.iter().sum::<f64>() / xs.len() as f64 }
    };
    let mut marks: Vec<i64> = MARKS.iter().copied().filter(|&y| y > y0 && y < y1).collect();
    marks.push(y1);
    for y in marks {
        out.push((format!("ages_births_k_{y}"), Kind::Real, mean(y, &|v| v.births_k)));
        out.push((format!("ages_deaths_k_{y}"), Kind::Real, mean(y, &|v| v.deaths_k)));
        out.push((format!("ages_life_{y}"), Kind::Real, mean(y, &|v| v.life)));
        let v = at(y);
        out.push((format!("ages_under15_{y}"), Kind::Share, v.map_or(f64::NAN, |v| v.under15)));
        out.push((format!("ages_over64_{y}"), Kind::Share, v.map_or(f64::NAN, |v| v.over64)));
        out.push((format!("ages_school_{y}"), Kind::Real, v.map_or(f64::NAN, |v| v.school)));
    }
    let short: f64 = ys.iter().map(|v| v.short).sum();
    let short_left: f64 = ys.iter().map(|v| v.short_left).sum();
    out.push(("ages_short_town_years".to_string(), Kind::Count, ys.iter().map(|v| v.short_towns).sum::<usize>() as f64));
    out.push(("ages_short_people".to_string(), Kind::Count, short));
    out.push(("ages_short_left_share".to_string(), Kind::Share, if short > 0.0 { short_left / short } else { f64::NAN }));
    out.push(("ages_drift".to_string(), Kind::Real, st.ages.drift));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stable_shape_is_its_own_next_year() {
        // a people growing one per cent a year at its stable shape is the same shape a year on
        let ap = AgesParams::game();
        let h = hazards(&ap, 1.3, 2.0, 0.002, 1750);
        let rate = 0.01;
        let n = stable(&h, rate);
        let mut new = [0.0; GROUPS];
        for a in 0..GROUPS {
            let stay = n[a] * (1.0 - h[a]);
            let up = if a < GROUPS - 1 { stay / 5.0 } else { 0.0 };
            new[a] += stay - up;
            if a < GROUPS - 1 {
                new[a + 1] += up;
            }
        }
        // the births that keep it growing at the rate
        new[0] += (1.0 + rate) - sum(&new);
        for a in 0..GROUPS {
            assert!((new[a] / (1.0 + rate) - n[a]).abs() < 1e-12, "{a}: {} {}", new[a] / (1.0 + rate), n[a]);
        }
    }

    #[test]
    fn movers_are_drawn_heavily_from_fifteen_to_thirty_four_and_never_more_than_there_are() {
        let r = [10.0; GROUPS];
        let d = draw(&r, 17.0, 3.0, MOVERS);
        assert!((sum(&d) - 17.0).abs() < 1e-12);
        assert!((d[4] / d[0] - 3.0).abs() < 1e-12);
        // more than the heavy groups can give at three times: they give all, the rest the remainder
        let d = draw(&r, 150.0, 3.0, MOVERS);
        assert!((sum(&d) - 150.0).abs() < 1e-9 && d.iter().zip(&r).all(|(a, b)| *a <= *b + 1e-12), "{d:?}");
        assert_eq!(draw(&r, 500.0, 3.0, MOVERS), r);
    }

    #[test]
    fn plenty_and_medicine_lengthen_life() {
        let ap = AgesParams::game();
        let poor = life_at_birth(&hazards(&ap, 1.3, 1.4, 0.0, 1750));
        let fed = life_at_birth(&hazards(&ap, 1.3, 8.0, 0.0, 1750));
        let cured = life_at_birth(&hazards(&ap, 1.3, 8.0, 0.0, 2050));
        assert!(poor < fed && fed < cured, "{poor} {fed} {cured}");
        assert!(poor > 25.0 && poor < 40.0 && cured > 70.0, "{poor} {cured}");
    }
}
