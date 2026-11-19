//! Who is paid for what they put up into a works. Not in the Python
//! (`docs/rules/owners.md`).
//!
//! Under the port a works no family owns is paid for out of its town's pool,
//! which is the households' and the landlords' money in proportion to what
//! each offers (`firms::draw_pool`), and yet every penny of its dividends, and
//! its cash when it closes, goes to the town's households.
//!
//! With `owners.enabled` each such works keeps its shares, `stake[f]`: the
//! parts of it held by its town's households, its landlords, the savers' fund
//! and the public, in that order. The last two stay at nothing in this step,
//! and are kept so that nothing need be laid out again when their money comes.
//!
//! - **Paying in.** Where money is drawn for a works (a new works in
//!   `firms::entry`; `firms::back_firm` when no family owns it; a sister works,
//!   paid for partly from its parent's cash and partly from the pool, in
//!   `firms::spawn_sister` and `works::extend_sister`), the shares move. With
//!   `B` the works' book before the money comes in (`families::works_value`
//!   plus its cash above nothing) and `x_h` what holder `h` paid,
//!   `share_h = (share_h * B + x_h) / (B + sum x)`. A new works' shares are
//!   what each paid over the whole; a sister's part paid from its parent's
//!   cash is held as the parent is.
//! - **Being paid.** A works' dividend (`econ::step_month`), its cash when it
//!   closes (`firms::close_firm`) and the money a sister works that is not
//!   founded gives back are paid by its shares: the households' part to the
//!   households, the landlords' to the landlords, of the town it stands in.
//! - **Shares never set.** A works whose shares are all nothing (the works of
//!   the warm-up, and any that came by a path that pays through no pool)
//!   takes its town's pool ratio, `invest_frac_hh * hh_cash :
//!   invest_frac_lord * lord_cash`, the first time they are read.
//! - **Families.** A family's works pay the family, and their shares are not
//!   read. A works a family winds down to the town's savers
//!   (`families::wind_down`) is then held as households and landlords paid
//!   for it.
//!
//! A railway's subscription earns no share. With the switch off every one of
//! these places takes its old branch, so a run is the same to the byte.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::facts::Kind;
use crate::families;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;

/// The holders, in the order `stake` keeps them.
pub const HOUSEHOLDS: usize = 0;
pub const LANDLORDS: usize = 1;
pub const SAVERS: usize = 2;
pub const PUBLIC: usize = 3;
pub const HOLDERS: [&str; 4] = ["households", "landlords", "savers", "public"];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OwnersParams {
    /// Whoever put money into a works holds a share of it and is paid by it.
    pub enabled: bool,
}

impl Default for OwnersParams {
    /// Off: what the port runs with.
    fn default() -> OwnersParams {
        OwnersParams { enabled: false }
    }
}

impl OwnersParams {
    /// The game's.
    pub fn game() -> OwnersParams {
        OwnersParams { enabled: true }
    }
}

/// One year of who held the works and who was paid by them, for the facts.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct OwnersYear {
    pub year: i64,
    /// The book of every works standing at the year's end
    /// (`families::works_value`), held by families, landlords and households.
    pub book: [f64; 3],
    /// The year's dividends of all works, paid to families, landlords and households.
    pub div: [f64; 3],
    /// What the landlords received in the year: rents and dividends.
    pub lords_income: f64,
}

/// What the rule keeps (`st.owners`). Empty while it is off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OwnersState {
    /// Each works' shares, by its place in the works' table: households,
    /// landlords, the savers' fund, the public. All nothing until first read
    /// or paid into; a family's works are never read.
    pub stake: Vec<[f64; 4]>,
    /// Every year's holdings and dividends.
    pub years: Vec<OwnersYear>,
    /// The largest gap from one of any works' shares, over the run.
    pub worst: f64,
    /// The year so far: dividends to families, landlords and households, and
    /// the landlords' whole income.
    acc: [f64; 4],
}

/// A town's pool ratio: what its households and its landlords offer to new works.
pub fn pool_ratio(st: &State, P: &Params, t: usize) -> [f64; 4] {
    let hh = P.invest_frac_hh * st.hh_cash[t].max(0.0);
    let lo = P.invest_frac_lord * st.lord_cash[t].max(0.0);
    if hh + lo > 0.0 { [hh / (hh + lo), lo / (hh + lo), 0.0, 0.0] } else { [1.0, 0.0, 0.0, 0.0] }
}

fn slot(st: &mut State, f: usize) -> &mut [f64; 4] {
    if st.owners.stake.len() <= f {
        st.owners.stake.resize(f + 1, [0.0; 4]);
    }
    &mut st.owners.stake[f]
}

fn unset(s: &[f64; 4]) -> bool {
    s.iter().all(|&v| v == 0.0)
}

/// A works' shares, as they stand: those never set are its town's pool ratio,
/// which this sets.
pub fn shares(st: &mut State, P: &Params, f: usize) -> [f64; 4] {
    let s = *slot(st, f);
    if !unset(&s) {
        return s;
    }
    let s = pool_ratio(st, P, st.f_town[f]);
    *slot(st, f) = s;
    s
}

/// A works' shares without setting them: those never set read as its town's
/// pool ratio today.
pub fn peek(st: &State, P: &Params, f: usize) -> [f64; 4] {
    match st.owners.stake.get(f) {
        Some(s) if !unset(s) => *s,
        _ => pool_ratio(st, P, st.f_town[f]),
    }
}

/// A works' book: its capacity, built and on order, at today's prices, and its cash above nothing.
pub fn book(st: &State, C: &Content, f: usize) -> f64 {
    families::works_value(st, C, f) + st.f_cash[f].max(0.0)
}

/// Holder by holder, `x` is paid into works `f`, before the money reaches its
/// cash: each share moves to what that holder now has in the works' book.
pub fn paid_in(st: &mut State, C: &Content, P: &Params, f: usize, x: [f64; 4]) {
    let paid: f64 = x.iter().sum();
    if paid <= 0.0 {
        return;
    }
    let b = book(st, C, f);
    let s = shares(st, P, f);
    *slot(st, f) = std::array::from_fn(|h| (s[h] * b + x[h]) / (b + paid));
}

/// A works founded, or bought from a family, with `x` paid by each holder:
/// its shares are what each paid over the whole. Left unset if nobody paid.
pub fn founded(st: &mut State, f: usize, x: [f64; 4]) {
    let paid: f64 = x.iter().sum();
    if paid > 0.0 {
        *slot(st, f) = x.map(|v| v / paid);
    }
}

/// The landlords' part of `amount` paid out by shares `s`; the rest is the
/// households'. (The savers' fund and the public hold nothing in this step:
/// were they to, their part would go to the households.)
pub fn lords_part(s: &[f64; 4], amount: f64) -> f64 {
    let all: f64 = s.iter().sum();
    if all > 0.0 { amount * s[LANDLORDS] / all } else { 0.0 }
}

/// `amount` paid into the cash of town `t`'s households and landlords by shares `s`.
pub fn pay_by(st: &mut State, t: usize, s: &[f64; 4], amount: f64) {
    let lords = lords_part(s, amount);
    st.lord_cash[t] += lords;
    st.hh_cash[t] += amount - lords;
}

/// A closing works' cash, paid out by its shares to the town it stands in.
pub fn pay_out(st: &mut State, P: &Params, f: usize, amount: f64) {
    let s = shares(st, P, f);
    pay_by(st, st.f_town[f], &s, amount);
}

/// The month's dividends (`econ::step_month`), after the families have taken
/// theirs out of `div` (which then holds nothing for their works): for each
/// works no family holds, the landlords' part is taken out of `div`, so that
/// what is left is the households', and is given back town by town, to be
/// paid to the landlords with the month's rents (`credit`). `all` is what the
/// works paid before the families took theirs. Empty while the rule is off.
pub fn dividends(st: &mut State, P: &Params, fi: &[usize], div: &mut [f64], all: f64) -> Vec<f64> {
    if !P.owners.enabled {
        return Vec::new();
    }
    let mut lords = vec![0.0; st.T_max];
    let (mut to_lords, mut to_hh) = (0.0, 0.0);
    for (k, &f) in fi.iter().enumerate() {
        if families::owner_of_firm(st, f).is_some() {
            continue;
        }
        // every works no family holds has its shares set by now
        let s = shares(st, P, f);
        let l = lords_part(&s, div[k]);
        div[k] -= l;
        lords[st.f_town[f]] += l;
        to_lords += l;
        to_hh += div[k];
    }
    let a = &mut st.owners.acc;
    a[0] += all - to_lords - to_hh;
    a[1] += to_lords;
    a[2] += to_hh;
    lords
}

/// The start of the played years (`sim::start_from`, after the warm-up): what
/// the warm-up paid is not counted in 1750's sums. The shares it set stand.
pub fn start(st: &mut State) {
    st.owners.acc = [0.0; 4];
}

/// The landlords' dividends of the month into their income, beside the month's
/// rents; and the landlords' whole income of the month into the year's sums.
pub fn credit(st: &mut State, lords: &[f64]) {
    if lords.is_empty() {
        return;
    }
    for (t, &l) in lords.iter().enumerate() {
        st.lord_income[t] += l;
    }
    st.owners.acc[3] += st.lord_income.iter().sum::<f64>();
}

/// The year's end (`sim::end_year`): who holds the works' book, what they were
/// paid, and how far any works' shares stand from one.
pub fn close_year(st: &mut State, C: &Content, P: &Params, year: i64) {
    if !P.owners.enabled {
        return;
    }
    let mut book_by = [0.0; 3];
    let mut worst = st.owners.worst;
    for f in 0..st.F_count {
        if !st.f_alive[f] {
            continue;
        }
        let v = families::works_value(st, C, f);
        if families::owner_of_firm(st, f).is_some() {
            book_by[0] += v;
            continue;
        }
        let s = peek(st, P, f);
        let all: f64 = s.iter().sum();
        if st.owners.stake.get(f).is_some_and(|s| !unset(s)) {
            worst = worst.max((all - 1.0).abs());
        }
        let l = lords_part(&s, v);
        book_by[1] += l;
        book_by[2] += v - l;
    }
    let a = std::mem::take(&mut st.owners.acc);
    st.owners.worst = worst;
    st.owners.years.push(OwnersYear { year, book: book_by, div: [a[0], a[1], a[2]], lords_income: a[3] });
}

fn ratio(a: f64, b: f64) -> f64 {
    if b.abs() > 1e-12 { a / b } else { f64::NAN }
}

/// The rule's facts at the sweep's marks: the shares of the works' book held by
/// families, landlords and households; the landlords' share of the year's
/// dividends; their dividends over their whole income; and the largest gap
/// from one of any works' shares. None unless the rule was on.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C) = (&r.state, &r.content);
    let mut out = Vec::new();
    if !r.params.owners.enabled {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    let marks = crate::facts::marks(y0, y1);
    let at = |y: i64| st.owners.years.iter().find(|o| o.year == y);
    for (k, who) in ["families", "lords", "hh"].iter().enumerate() {
        for &y in &marks {
            let v = at(y).map_or(f64::NAN, |o| ratio(o.book[k], o.book.iter().sum()));
            out.push((format!("owners_works_{who}_{y}"), Kind::Share, v));
        }
    }
    for &y in &marks {
        let v = at(y).map_or(f64::NAN, |o| ratio(o.div[1], o.div.iter().sum()));
        out.push((format!("owners_div_lords_{y}"), Kind::Share, v));
    }
    for &y in &marks {
        let v = at(y).map_or(f64::NAN, |o| ratio(o.div[1], o.lords_income));
        out.push((format!("owners_lords_from_works_{y}"), Kind::Share, v));
    }
    out.push(("owners_stake_worst".into(), Kind::Real, st.owners.worst));
    out
}
