//! Who works, by age: school, the university and the pension take hands out of
//! work. Not in the Python; `docs/rules/who-works.md` is the longer account.
//!
//! Under the port a town's hands for hire are half its people, whatever their
//! ages (`participation`). Under this rule, from 1901, they are counted from
//! the town's people by age (`ages.rs`, seventeen five-year groups) as each
//! year closes, for the year to come:
//!
//! - **Children** under ten do not work. Of those aged ten to fourteen, those
//!   still at school do not work, and the rest work at `child`. A child is at
//!   school from five for the years its town gives it: what the families'
//!   income buys (`ages::school_years`) or what the treasury pays for this year
//!   (`purse`), whichever is more.
//! - **The working ages**, fifteen to 64, work at `working`, less those still
//!   at school past fifteen and the university's students.
//! - **The old**, 65 and over, work at `old`. Those a pension reaches (over the
//!   pension's age, while one is paid, as far as the programmes are paid in
//!   full) leave work, all but `pensioned` of them. A pension's age set under
//!   65 by hand reaches the working ages too.
//!
//! One kind of household, and no separate rate for women. `working` is fitted
//! once so that the island's share in work in 1900, counted this way, is the
//! half it played: on islands 1 to 8 under the game's rules.
//!
//! The rule acts from `FROM` and comes in over `BLEND_YEARS`: in each of those
//! years a town's share in work moves a further tenth of the way from the half
//! to its count by age, so that no town's hands jump in a year. To 1900 every
//! town's hands are exactly half its people.
//!
//! It is read in three places, each of which reads what it read before by an
//! early return while the rule does not act: the labour supply
//! (`econ::step_month`, [`supply`]), and the two places that turn the works'
//! hands into townsfolk (`towns::update_urban` and the port's
//! `land::update_urban`, [`hands`]). The university's students, counted out of
//! work here, are no longer taken out of the hands a second time
//! (`university::take_hands`, [`students_held`]).
//!
//! The reckoning runs from the first year the ages are kept, so that the facts
//! can say what the rule would count before it acts; nothing is played from it
//! before `FROM`, and its series are nothing until then. It needs `ages.watch`:
//! without the ages the half stands.

use serde::{Deserialize, Serialize};

use crate::ages::{GROUPS, OLD, WORKING};
use crate::facts::Kind;
use crate::observer::Series;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;

/// The first year the rule acts; to the year before, hands are half the people.
pub const FROM: i64 = 1901;
/// The years over which it comes in, a tenth of the way a year.
pub const BLEND_YEARS: f64 = 10.0;
/// The group of ten to fourteen, the only children who work.
const CHILDREN: usize = 2;
/// The groups of fifteen to 24, from which the students are counted out.
const YOUNG_ADULTS: std::ops::Range<usize> = 3..5;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WhoWorksParams {
    /// Count the hands by age from 1901, so that school, the university and
    /// the pension take hands out of work.
    pub enabled: bool,
    /// Of those aged ten to fourteen and not at school, the share at work.
    pub child: f64,
    /// Of those aged fifteen to 64 and not at school or the university, the
    /// share at work: fitted once so that the island's share in work in 1900
    /// is the half it played.
    pub working: f64,
    /// Of those 65 and over whom no pension reaches, the share at work.
    pub old: f64,
    /// Of those a pension reaches, the share who work on.
    pub pensioned: f64,
}

impl Default for WhoWorksParams {
    /// Off: what the port runs with.
    fn default() -> WhoWorksParams {
        WhoWorksParams { enabled: false, child: 0.25, working: 0.705, old: 0.55, pensioned: 0.08 }
    }
}

impl WhoWorksParams {
    /// The game's rule.
    pub fn game() -> WhoWorksParams {
        WhoWorksParams { enabled: true, ..WhoWorksParams::default() }
    }
}

/// One year of the reckoning, for the island, as the year before it closed
/// counted it for this year.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct WhoYear {
    pub year: i64,
    pub people: f64,
    /// people in work by the count by age
    pub hands: f64,
    /// the share of the people in work that the year plays: the half, or the
    /// count by age as far as the rule has come in (`blend`)
    pub played: f64,
    pub blend: f64,
    /// people in work by age, and the people of each age: under 15, 15 to 24,
    /// 25 to 64, 65 and over
    pub at_work: [f64; 4],
    pub of_age: [f64; 4],
    /// the hands each takes out of work: school (those of ten and over at
    /// school), the university (its students), the pension (those it reaches)
    pub cost_school: f64,
    pub cost_university: f64,
    pub cost_pension: f64,
    /// those aged ten to fourteen at school, the students, those a pension reaches
    pub at_school: f64,
    pub students: f64,
    pub pensioned: f64,
    /// the working ages' people in work, for the fit: what `working` multiplies
    pub working_free: f64,
}

/// What the rule keeps (`st.who_works`). Empty while it is off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WhoState {
    /// Each town's share of its people in work this year, as played: empty
    /// until the rule acts, and below nothing for a town with no count (a town
    /// founded since, which takes the island's).
    pub share: Vec<f64>,
    /// Each town's share in work by the count by age alone, before the blend.
    pub count: Vec<f64>,
    /// The island's share in work this year, as played, and by the count.
    pub island: f64,
    pub island_count: f64,
    /// How far the rule has come in this year, 0 to 1.
    pub blend: f64,
    pub years: Vec<WhoYear>,
}

fn on(P: &Params) -> bool {
    P.who_works.enabled && P.ages.watch
}

/// Whether the rule acts this year: each town's share is set.
pub fn acting(st: &State, P: &Params) -> bool {
    on(P) && !st.who_works.share.is_empty()
}

/// How far the rule has come in, in `year`: nothing before `FROM`, a tenth more
/// each year after.
pub fn blend(year: i64) -> f64 {
    if year < FROM { 0.0 } else { ((year - FROM + 1) as f64 / BLEND_YEARS).min(1.0) }
}

/// Town `t`'s share of its people in work this year (the island's for a town
/// with no count of its own).
fn share_of(st: &State, t: usize) -> f64 {
    match st.who_works.share.get(t) {
        Some(&s) if s >= 0.0 => s,
        _ => st.who_works.island,
    }
}

/// The labour supply (`econ::step_month`, before the carters and the others
/// are taken out): each town's people in work, where the rule acts. `supply`
/// comes in as `participation` times each town's people, and is left so while
/// it does not.
pub fn supply(st: &State, P: &Params, supply: &mut [f64]) {
    if !acting(st, P) {
        return;
    }
    for (t, s) in supply.iter_mut().enumerate() {
        *s = share_of(st, t) * st.pop[t];
    }
}

/// Town `t`'s hands, as the two places that turn works' hands into townsfolk
/// read them (`towns::update_urban`, `land::update_urban`): half its people
/// while the rule does not act, its people in work while it does.
pub fn hands(st: &State, P: &Params, t: usize) -> f64 {
    if !acting(st, P) {
        return P.participation * st.pop[t];
    }
    share_of(st, t) * st.pop[t]
}

/// The hands of town `t`'s people other than its university's students, `rest`
/// of them (`towns::update_urban`): half of them while the rule does not act;
/// while it does, the town's people in work, of whom no student is one.
pub fn hands_of_rest(st: &State, P: &Params, t: usize, rest: f64) -> f64 {
    if !acting(st, P) {
        return P.participation * rest;
    }
    // as the rule comes in, the half of the rest for the part it has not
    let w = &st.who_works;
    let count = match w.count.get(t) {
        Some(&s) if s >= 0.0 => s,
        _ => w.island_count,
    };
    (1.0 - w.blend) * P.participation * rest + w.blend * count * st.pop[t]
}

/// The part of the university's students that `university::take_hands` still
/// takes out of the hands: all of them while the rule does not act; while it
/// does, only the part of the hands still reckoned as the half, since the count
/// by age has the students out of work already. None (all of them) while it
/// does not act, so that its sum is the sum it was.
pub fn students_held(st: &State, P: &Params) -> Option<f64> {
    if !acting(st, P) {
        return None;
    }
    Some(1.0 - st.who_works.blend)
}

/// The share of a single year of age `a` at school, with `years` of school from five.
fn at_school(years: f64, a: usize) -> f64 {
    (years - (a as f64 - crate::purse::SCHOOL_FROM)).clamp(0.0, 1.0)
}

/// One town's count: its people in work by group, and what school, the
/// university and the pension take. `years` is the years of school a child
/// gets this year, `students` the university's students living there,
/// `reach` the share of those over `pension_age` a pension reaches (nothing
/// with no pension).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Count {
    pub at_work: [f64; GROUPS],
    pub school: f64,
    pub university: f64,
    pub pension: f64,
    pub at_school: f64,
    pub students: f64,
    pub pensioned: f64,
    pub working_free: f64,
}

pub fn count(wp: &WhoWorksParams, row: &[f64; GROUPS], years: f64, students: f64, pension_age: f64, reach: f64) -> Count {
    let mut c = Count::default();
    let mut students_left = students.max(0.0);
    for g in 0..GROUPS {
        let n = row[g].max(0.0);
        if n <= 0.0 {
            continue;
        }
        // at school: each single year of age of the group, a fifth of it
        let schooled: f64 = (5 * g..5 * g + 5).map(|a| at_school(years, a)).sum::<f64>() * n / 5.0;
        let (rate, mut free) = if g < CHILDREN {
            (0.0, 0.0)
        } else if g == CHILDREN {
            c.at_school += schooled;
            c.school += wp.child * schooled;
            (wp.child, n - schooled)
        } else if g < OLD.start {
            c.school += wp.working * schooled;
            (wp.working, n - schooled)
        } else {
            (wp.old, n)
        };
        // the university's students, out of the youngest adults first
        if YOUNG_ADULTS.contains(&g) && students_left > 0.0 {
            let s = students_left.min(free);
            free -= s;
            students_left -= s;
            c.students += s;
            c.university += rate * s;
        }
        // those a pension reaches leave work, all but `pensioned` of them
        let covered = if reach > 0.0 { free * crate::purse::above(g, pension_age) * reach } else { 0.0 };
        let work = rate * (free - covered) + wp.pensioned * covered;
        if covered > 0.0 {
            c.pensioned += covered;
            c.pension += (rate - wp.pensioned) * covered;
        }
        if WORKING.contains(&g) {
            c.working_free += free - covered;
        }
        c.at_work[g] = work;
    }
    c
}

/// The four bands the facts and the chart read: under 15, 15 to 24, 25 to 64, 65 and over.
pub const BANDS: [std::ops::Range<usize>; 4] = [0..3, 3..5, 5..13, 13..17];

/// At a year's end (`sim::end_year`, after the purse has set next year's laws
/// and the university its students): each town counted for the year to come,
/// and from `FROM` its share in work set, blended in. Nothing while the rule
/// or the ages are off.
pub fn close_year(st: &mut State, P: &Params, year: i64) {
    if !on(P) || st.ages.rows.len() != st.T_max {
        return;
    }
    let next = year + 1;
    let T = st.T_max;
    let wp = &P.who_works;
    let purse = &st.purse;
    let level = if P.state.enabled { purse.school_level * purse.cut } else { 0.0 };
    let (pension_age, reach) = if P.state.enabled && purse.pension_share > 0.0 { (purse.pension_age, purse.cut.clamp(0.0, 1.0)) } else { (0.0, 0.0) };
    let mut row_y = WhoYear { year: next, ..WhoYear::default() };
    let mut counted = vec![-1.0; T];
    for t in 0..T {
        if !st.town_active[t] {
            continue;
        }
        let r = st.ages.rows[t];
        let n: f64 = r.iter().sum();
        if !(n > 0.0) {
            continue;
        }
        let years = crate::ages::school_years(st.ages.ybar.get(t).copied().unwrap_or(0.0)).max(level);
        let students = crate::university::living(st, P, t);
        let c = count(wp, &r, years, students, pension_age, reach);
        let work: f64 = c.at_work.iter().sum();
        counted[t] = work / n;
        row_y.people += n;
        row_y.hands += work;
        for (k, band) in BANDS.iter().enumerate() {
            row_y.at_work[k] += c.at_work[band.clone()].iter().sum::<f64>();
            row_y.of_age[k] += r[band.clone()].iter().sum::<f64>();
        }
        row_y.cost_school += c.school;
        row_y.cost_university += c.university;
        row_y.cost_pension += c.pension;
        row_y.at_school += c.at_school;
        row_y.students += c.students;
        row_y.pensioned += c.pensioned;
        row_y.working_free += c.working_free;
    }
    let b = blend(next);
    let half = P.participation;
    let island_count = if row_y.people > 0.0 { row_y.hands / row_y.people } else { half };
    row_y.blend = b;
    row_y.played = (1.0 - b) * half + b * island_count;
    let w = &mut st.who_works;
    w.blend = b;
    if b > 0.0 {
        w.share = counted.iter().map(|&s| if s >= 0.0 { (1.0 - b) * half + b * s } else { -1.0 }).collect();
        w.count = counted;
        w.island = row_y.played;
        w.island_count = island_count;
    } else {
        w.share.clear();
        w.count.clear();
        w.island = half;
        w.island_count = half;
    }
    w.years.push(row_y);
}

/// The rule's series, after the others: the share of the people in work as
/// played, and by age as counted; the people in work, and those at work.
/// Nothing (nought) until the rule acts.
pub fn add_series(reg: &mut Series, P: &Params, n_months: usize) {
    if !on(P) {
        return;
    }
    reg.add("work_share", "People in work, of all people", "people", "share", n_months);
    reg.add("work_under15", "In work, of those under 15", "people", "share", n_months);
    reg.add("work_15_24", "In work, of those 15 to 24", "people", "share", n_months);
    reg.add("work_25_64", "In work, of those 25 to 64", "people", "share", n_months);
    reg.add("work_over64", "In work, of those 65 and over", "people", "share", n_months);
    reg.add("work_hands", "People in work", "people", "people", n_months);
    reg.add("work_employed", "People at work", "people", "people", n_months);
}

/// The month's figures (`observer::record_month`): nought until the rule acts.
pub fn record_month(st: &State, P: &Params, s: &mut Series, t: usize) {
    if !on(P) {
        return;
    }
    let names = ["work_share", "work_under15", "work_15_24", "work_25_64", "work_over64", "work_hands", "work_employed"];
    let v = match st.who_works.years.last() {
        Some(v) if acting(st, P) => v,
        _ => {
            for n in names {
                s.set(n, t, 0.0);
            }
            return;
        }
    };
    let (mut people, mut hands) = (0.0, 0.0);
    for i in 0..st.T_max {
        if st.town_active[i] {
            people += st.pop[i];
            hands += share_of(st, i) * st.pop[i];
        }
    }
    s.set("work_share", t, if people > 0.0 { hands / people } else { 0.0 });
    for (k, n) in names[1..5].iter().enumerate() {
        s.set(n, t, if v.of_age[k] > 0.0 { v.at_work[k] / v.of_age[k] } else { 0.0 });
    }
    s.set("work_hands", t, hands);
    s.set("work_employed", t, st.month_stats.employment.iter().sum());
}

/// The years the facts are read at, with the run's last.
pub const MARKS: [i64; 8] = [1800, 1850, 1900, 1925, 1950, 1975, 2000, 2025];

/// The facts (docs/rules/who-works.md), under "Work and wages": at `MARKS`
/// inside the run and its last year, the share of all people in work by the
/// count (`work_share_Y`, before 1901 what the rule would count, since the
/// half is played), and of each age (`work_under15_Y`, `work_15_24_Y`,
/// `work_25_64_Y`, `work_over64_Y`); the share played (`work_played_Y`); and
/// the hands school, the university and the pension take out of work
/// (`work_cost_school_Y`, `_university_Y`, `_pension_Y`, people). Nothing in a
/// run without the rule.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let ys = &r.state.who_works.years;
    let mut out = Vec::new();
    if !on(&r.params) || ys.is_empty() {
        return out;
    }
    let (y0, y1) = (ys[0].year, ys[ys.len() - 1].year.min(r.content.start_year + r.rec.n_years as i64 - 1));
    let at = |y: i64| ys.iter().find(|v| v.year == y);
    let mut marks: Vec<i64> = MARKS.iter().copied().filter(|&y| y >= y0 && y < y1).collect();
    marks.push(y1);
    let share = |a: f64, b: f64| if b > 0.0 { a / b } else { f64::NAN };
    for y in marks {
        let v = at(y);
        let f = |g: &dyn Fn(&WhoYear) -> f64| v.map_or(f64::NAN, g);
        out.push((format!("work_share_{y}"), Kind::Share, f(&|v| share(v.hands, v.people))));
        out.push((format!("work_played_{y}"), Kind::Share, f(&|v| v.played)));
        for (k, name) in ["under15", "15_24", "25_64", "over64"].iter().enumerate() {
            out.push((format!("work_{name}_{y}"), Kind::Share, f(&|v| share(v.at_work[k], v.of_age[k]))));
        }
        out.push((format!("work_cost_school_{y}"), Kind::Count, f(&|v| v.cost_school)));
        out.push((format!("work_cost_university_{y}"), Kind::Count, f(&|v| v.cost_university)));
        out.push((format!("work_cost_pension_{y}"), Kind::Count, f(&|v| v.cost_pension)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nobody_is_counted_twice_and_nobody_works_more_than_once() {
        let wp = WhoWorksParams::game();
        let row = [100.0; GROUPS];
        // eleven years of school, 30 students, a pension at 65 paid in full
        let c = count(&wp, &row, 11.0, 30.0, 65.0, 1.0);
        // no group has more at work than it has people
        assert!(c.at_work.iter().zip(&row).all(|(w, n)| *w >= 0.0 && *w <= *n), "{:?}", c.at_work);
        // under ten nobody works; ten to fourteen all at school
        assert_eq!(c.at_work[0] + c.at_work[1] + c.at_work[2], 0.0);
        assert!((c.at_school - 100.0).abs() < 1e-12);
        // the fifteen-year-olds at school (a fifth of 15 to 19), then the students out of the rest
        let free_15_19 = 80.0 - 30.0;
        assert!((c.at_work[3] - wp.working * free_15_19).abs() < 1e-9, "{}", c.at_work[3]);
        assert!((c.students - 30.0).abs() < 1e-12);
        // the old: all reached, `pensioned` of them at work
        for g in OLD {
            assert!((c.at_work[g] - wp.pensioned * 100.0).abs() < 1e-9);
        }
        // what each takes, added back, is what would work with none of them
        let none = count(&wp, &row, 0.0, 0.0, 0.0, 0.0);
        let at = |c: &Count| c.at_work.iter().sum::<f64>();
        assert!((at(&none) - at(&c) - c.school - c.university - c.pension).abs() < 1e-9, "{} {} {} {} {}", at(&none), at(&c), c.school, c.university, c.pension);
    }

    #[test]
    fn a_pension_at_seventy_leaves_the_sixty_fives_at_work_and_a_cut_pension_reaches_fewer() {
        let wp = WhoWorksParams::game();
        let row = [100.0; GROUPS];
        let c = count(&wp, &row, 5.0, 0.0, 70.0, 1.0);
        assert!((c.at_work[13] - wp.old * 100.0).abs() < 1e-9);
        assert!((c.at_work[14] - wp.pensioned * 100.0).abs() < 1e-9);
        let half = count(&wp, &row, 5.0, 0.0, 70.0, 0.5);
        assert!(half.at_work[14] > c.at_work[14] && half.pension < c.pension);
    }

    #[test]
    fn the_rule_comes_in_a_tenth_a_year_from_1901() {
        assert_eq!(blend(1900), 0.0);
        assert!((blend(1901) - 0.1).abs() < 1e-12);
        assert_eq!(blend(1910), 1.0);
        assert_eq!(blend(2050), 1.0);
    }
}
