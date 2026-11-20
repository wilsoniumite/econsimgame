//! The state's purse: a treasury that balances by rule, one tax on pay, and
//! two programmes, schools and pensions, under a book of acts the island meets
//! by itself. Not in the Python; `docs/rules/state.md` is the longer account.
//!
//! - **The treasury** is a purse like any other: `econ::money_total` counts it,
//!   and every flow in or out of it is a transfer with a household purse in the
//!   same month, so the monthly money check holds.
//! - **The rate on pay** is taken where pay is banked (`econ::step_month`, where
//!   `hh_inc` holds the month's pay and nothing else until the dividends are
//!   added): works' and the land's wages, the sinkers, every kind of carting,
//!   the navvies, the hired managers, and the teachers' own pay.
//! - **It balances by rule.** At each year's end the rate for the next year is
//!   set so that the next year's revenue covers the programmes and brings the
//!   treasury to `reserve_months` of its spending; what it holds above that
//!   lowers the rate. No debt: the treasury never pays out more than it holds.
//!   If the rate would pass `rate_cap`, every programme is cut in proportion
//!   and the news says so.
//! - **Schools.** The treasury hires each town's teachers, `school_hands` a
//!   pupil, as the canals hire navvies: taken out of the town's hands for hire
//!   before the wage is sought, counted at work, and paid the town's wage. The
//!   years of school a cohort gets are the years the treasury paid for while it
//!   was at school, where that is more than its families' income bought
//!   (`ages::school_years`). A programme grows toward its law by at most
//!   `year_room` of the law a year, what a trade may add.
//! - **Pensions.** A share of the town's wage to each person over the pension's
//!   age, paid into the household purse of the town they live in.
//! - **The book** (`StateParams::book`): the acts the island meets by itself,
//!   none before 1900. Three chips (`school_years`, `pension_age`,
//!   `pension_share`) override it from the year they are set.
//!
//! Not built, and where each will hang: the tax on spending (as households,
//! landlords and families settle, beside `hh_spend` in `econ::step_month`); the
//! tax on returns with expensing and the fund's stake (`firms::entry`,
//! `works::review`, the works' dividend line); the tax on rent's growth
//! (`econ::pay_cell_rents`, `pay_rent`); the fund and its dividend (beside
//! `fund_div`); relief, health and public works (programmes in `Law`, bought as
//! the schools are); statutes as orders (`edits`, replayed as the player's).
//!
//! With `state.enabled` off every hook returns at once and nothing is drawn:
//! a run is the same to the byte. With it on and the book at `none`, every rate
//! and programme stays at nothing, and the run is again the same (the proof at
//! the top of the page).

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::facts::Kind;
use crate::observer::Series;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::{Event, State};

/// Which book of acts the island runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Book {
    /// No act: every rate and programme stays at nothing unless a chip sets it.
    None,
    /// The design's default book for schools and pensions: nothing before 1900.
    Default,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StateParams {
    /// The treasury, its tax and its programmes.
    pub enabled: bool,
    /// The book of acts the island meets by itself.
    pub book: Book,
    /// Months of its own spending the treasury means to hold.
    pub reserve_months: f64,
    /// The highest rate on pay; past it the programmes are cut.
    pub rate_cap: f64,
    /// How far a programme may grow toward its law in a year, as a share of
    /// the law (what a trade may add, `works.year_room`).
    pub year_room: f64,
    /// Hands a pupil: a teacher to fourteen, and those who keep the rooms.
    pub school_hands: f64,
    /// Years of school by hand, overriding the book from the year it is set;
    /// below nothing, the book's.
    pub school_years: f64,
    /// The pension's age by hand; below nothing, the book's.
    pub pension_age: f64,
    /// The pension as a share of the wage, by hand; below nothing, the book's.
    pub pension_share: f64,
}

impl Default for StateParams {
    /// Off: what the port runs with.
    fn default() -> StateParams {
        StateParams {
            enabled: false,
            book: Book::Default,
            reserve_months: 2.0,
            rate_cap: 0.5,
            year_room: 0.15,
            school_hands: 0.1,
            school_years: -1.0,
            pension_age: -1.0,
            pension_share: -1.0,
        }
    }
}

impl StateParams {
    /// The game's purse, on the default book.
    pub fn game() -> StateParams {
        StateParams { enabled: true, ..StateParams::default() }
    }
}

/// The book's acts, as the design dates and conditions them.
pub mod book {
    /// No act is met before this year.
    pub const FROM: i64 = 1900;
    /// Schools: the towns hold this share of the people.
    pub const SCHOOLS_TOWNS: f64 = 0.30;
    /// The years of school of the three school acts.
    pub const SCHOOL_STEPS: [f64; 3] = [6.0, 9.0, 11.0];
    /// Each longer schooling a generation after the last.
    pub const GENERATION: i64 = 25;
    /// The first pension: over-64s more than this share of the people.
    pub const PENSION_OLD: f64 = 0.055;
    /// The first pension's age and share of the wage.
    pub const PENSION_FIRST: (f64, f64) = (70.0, 0.2);
    /// The later pension: baskets a head over this.
    pub const PENSION_RICH: f64 = 12.0;
    /// The later pension's age and share of the wage.
    pub const PENSION_LATER: (f64, f64) = (65.0, 0.25);
}

/// Children start school at five.
pub const SCHOOL_FROM: f64 = 5.0;
/// The years over which the budget reckons how fast pay grows.
pub const GROWTH_YEARS: usize = 10;
/// The most hands for hire a town gives to its schools in a month.
pub const TEACHERS_MOST: f64 = 0.25;

/// One act of the book, as it was met.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Act {
    pub year: i64,
    /// `schools` or `pension`
    pub what: String,
    /// years of school, or the pension's age
    pub at: f64,
    /// the pension's share of the wage (nothing for schools)
    pub share: f64,
}

/// The month's flows, for the observer and the books.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Month {
    pub tax: f64,
    pub schools: f64,
    pub pensions: f64,
    pub teachers: f64,
    pub pensioners: f64,
    /// pay the rate was taken on
    pub base: f64,
}

/// The year's sums, from January to the year's close.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct YearSums {
    pub tax: f64,
    pub schools: f64,
    pub pensions: f64,
    pub base: f64,
    /// teachers' pay within `base`
    pub base_schools: f64,
    pub teachers: f64,
    pub teachers_want: f64,
    pub pensioners: f64,
    pub pupils: f64,
    pub hands: f64,
    pub months: usize,
    /// months the treasury could not pay all it owed
    pub short_months: usize,
    /// each town's households' income before the state's flows, and the state's net flow to them
    pub town_before: Vec<f64>,
    pub town_net: Vec<f64>,
}

/// One year of the purse.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct PurseYear {
    pub year: i64,
    /// the rate on pay in force through the year
    pub rate: f64,
    pub tax: f64,
    pub schools: f64,
    pub pensions: f64,
    /// pay the rate was taken on
    pub base: f64,
    /// the treasury at the year's end
    pub treasury: f64,
    /// the treasury at the year's end in months of the year's spending
    pub months_held: f64,
    /// what the island earned in the year (twelve months of `gdp_nom`)
    pub income: f64,
    /// teachers (hands, mean of the months) and all hands at work
    pub teachers: f64,
    pub teachers_want: f64,
    pub hands: f64,
    pub pensioners: f64,
    pub pupils: f64,
    /// the years of school the law gives, and the programme's reach this year
    pub school_law: f64,
    pub school_level: f64,
    pub pension_age: f64,
    pub pension_share: f64,
    /// the share of the programmes paid (1 unless the rate passed its cap)
    pub cut: f64,
    pub short_months: usize,
    /// the state's net flow (teachers' pay and pensions less the tax) over the
    /// households' income before it, in the towns holding the older half of
    /// the people and the younger half, and the half with more pupils and fewer
    pub net_old_many: f64,
    pub net_old_few: f64,
    pub net_pupils_many: f64,
    pub net_pupils_few: f64,
}

/// What the purse keeps (`st.purse`). Empty while it is off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PurseState {
    pub treasury: f64,
    /// the rate on pay in force
    pub rate: f64,
    pub school_law: f64,
    /// the years of school the programme reaches this year
    pub school_level: f64,
    pub pension_age: f64,
    pub pension_share: f64,
    /// the share of the programmes paid this year
    pub cut: f64,
    /// the book's steps met: schools 0 to 3, pensions 0 to 2
    pub schools_step: usize,
    pub schools_year: i64,
    pub pension_step: usize,
    pub acts: Vec<Act>,
    /// each town's teachers wanted this year, and hired this month
    pub teachers_want: Vec<f64>,
    pub teachers: Vec<f64>,
    pub pupils: Vec<f64>,
    pub pensioners: Vec<f64>,
    /// the school level paid in each of the last ten years, oldest first
    pub level_hist: Vec<f64>,
    pub month: Month,
    pub now: YearSums,
    /// last year's pay base, and the last few years' (oldest first), for the
    /// growth the budget reckons on
    pub base_last: f64,
    pub base_hist: Vec<f64>,
    pub years: Vec<PurseYear>,
}

fn on(P: &Params) -> bool {
    P.state.enabled
}

/// Whether anything is paid or taken this year.
fn acting(st: &State) -> bool {
    st.purse.rate > 0.0 || st.purse.school_level > 0.0 || st.purse.pension_share > 0.0 || st.purse.treasury != 0.0
}

/// The share of the people of one five-year group `g` aged `from` or more.
fn above(g: usize, from: f64) -> f64 {
    let lo = 5.0 * g as f64;
    if g + 1 == crate::ages::GROUPS {
        return if from <= lo { 1.0 } else { ((lo + 5.0 - from) / 5.0).clamp(0.0, 1.0) };
    }
    ((lo + 5.0 - from) / 5.0).clamp(0.0, 1.0)
}

/// Pupils in a row of the ages at `level` years of school from five: each age
/// from five on is at school as far as the level reaches it.
pub fn pupils(row: &[f64; crate::ages::GROUPS], level: f64) -> f64 {
    let mut n = 0.0;
    for a in 5..20 {
        let at = (level - (a as f64 - SCHOOL_FROM)).clamp(0.0, 1.0);
        if at > 0.0 {
            n += at * row[a / 5] / 5.0;
        }
    }
    n
}

/// People in a row aged `age` or more.
pub fn over(row: &[f64; crate::ages::GROUPS], age: f64) -> f64 {
    (0..crate::ages::GROUPS).map(|g| row[g] * above(g, age)).sum()
}

/// The years of school the treasury paid for the cohort turning fifteen this
/// year, from what it paid at each age from five (`ages::year` takes the more
/// of this and what its families bought). Nothing while the purse is off or
/// has paid for none.
pub fn cohort_years(st: &State) -> f64 {
    let h = &st.purse.level_hist;
    if h.is_empty() {
        return 0.0;
    }
    // h holds the last ten years, this year last; the cohort was 14 this year,
    // 13 the year before, and five nine years ago
    let n = h.len();
    let mut years = 0.0;
    for a in 5..15usize {
        let back = 14 - a;
        let level = if back < n { h[n - 1 - back] } else { 0.0 };
        years += (level - (a as f64 - SCHOOL_FROM)).clamp(0.0, 1.0);
    }
    // the years past fifteen at this year's level
    years + (h[n - 1] - 10.0).max(0.0)
}

/// The teachers are not for hire: each town's teachers this month are taken out
/// of its hands before the wage is sought (`econ::step_month`, after the
/// navvies and the menders), at most `TEACHERS_MOST` of them.
pub fn take_teachers(st: &mut State, P: &Params, supply: &mut [f64]) {
    if !on(P) {
        return;
    }
    if st.purse.teachers_want.is_empty() {
        st.purse.teachers.iter_mut().for_each(|v| *v = 0.0);
        return;
    }
    let T = supply.len();
    st.purse.teachers.resize(T, 0.0);
    for t in 0..T {
        let want = st.purse.teachers_want.get(t).copied().unwrap_or(0.0);
        let most = (TEACHERS_MOST * supply[t]).max(0.0);
        let hands = want.min(most).max(0.0);
        supply[t] -= hands;
        st.purse.teachers[t] = hands;
    }
}

/// The teachers are at work (`econ::step_month`, beside the navvies); and the
/// month's hands, for the teachers' share.
pub fn count_teachers(st: &mut State, P: &Params, employment: &mut [f64]) {
    if !on(P) {
        return;
    }
    for (e, n) in employment.iter_mut().zip(&st.purse.teachers) {
        *e += n;
    }
    st.purse.now.hands += employment.iter().sum::<f64>();
}

/// The month's flows (`econ::step_month`, once the books have taken the month's
/// incomes and before the purses are credited): the teachers paid at the
/// town's wage, the rate taken on all pay with theirs, the pensions paid; each
/// a transfer between the treasury and the town's households' income `hh_inc`.
/// `pay` is each town's pay as it was banked, before the dividends.
pub fn month(st: &mut State, P: &Params, w: &[f64], pay: &[f64], hh_inc: &mut [f64]) {
    if !on(P) {
        return;
    }
    let T = hh_inc.len();
    let p = &mut st.purse;
    if p.now.town_before.len() != T {
        p.now.town_before.resize(T, 0.0);
        p.now.town_net.resize(T, 0.0);
    }
    p.teachers.resize(T, 0.0);
    // what pay was banked, before anything of the state's: the base of next
    // year's budget, whatever this year's rate
    let mut m = Month::default();
    let teach: Vec<f64> = (0..T).map(|t| p.teachers[t] * w[t].max(0.0)).collect();
    let pens: Vec<f64> = (0..T)
        .map(|t| {
            let n = p.pensioners.get(t).copied().unwrap_or(0.0);
            if p.pension_share > 0.0 && n > 0.0 { n * p.pension_share * p.cut * w[t].max(0.0) } else { 0.0 }
        })
        .collect();
    let base: Vec<f64> = (0..T).map(|t| pay[t].max(0.0) + teach[t]).collect();
    let tax: Vec<f64> = base.iter().map(|b| p.rate * b).collect();
    let (all_teach, all_pens, all_tax): (f64, f64, f64) = (teach.iter().sum(), pens.iter().sum(), tax.iter().sum());
    // no debt: the treasury pays what it holds, the teachers first
    let cash = p.treasury + all_tax;
    let (kt, kp) = if all_teach + all_pens <= cash {
        (1.0, 1.0)
    } else if all_teach <= cash {
        (1.0, if all_pens > 0.0 { ((cash - all_teach) / all_pens).clamp(0.0, 1.0) } else { 1.0 })
    } else {
        (if all_teach > 0.0 { (cash / all_teach).clamp(0.0, 1.0) } else { 1.0 }, 0.0)
    };
    if kt < 1.0 || kp < 1.0 {
        p.now.short_months += 1;
    }
    for t in 0..T {
        p.now.town_before[t] += hh_inc[t];
        let net = kt * teach[t] - tax[t] + kp * pens[t];
        // (nothing is added where nothing flows, so a purse with nothing in it
        // leaves every income as it was, to the bit)
        if net != 0.0 {
            p.now.town_net[t] += net;
            hh_inc[t] += net;
        }
        m.schools += kt * teach[t];
        m.pensions += kp * pens[t];
        m.tax += tax[t];
        m.base += base[t];
        m.teachers += p.teachers[t];
        m.pensioners += p.pensioners.get(t).copied().unwrap_or(0.0);
    }
    if !(m.pensions > 0.0) {
        m.pensioners = 0.0;
    }
    p.treasury += m.tax - m.schools - m.pensions;
    if p.treasury < 0.0 && p.treasury > -1e-9 * cash.abs().max(1.0) {
        p.treasury = 0.0;
    }
    let s = &mut p.now;
    s.tax += m.tax;
    s.schools += m.schools;
    s.pensions += m.pensions;
    s.base += m.base;
    s.base_schools += all_teach;
    s.teachers += m.teachers;
    s.teachers_want += p.teachers_want.iter().sum::<f64>();
    s.pensioners += m.pensioners;
    s.pupils += p.pupils.iter().sum::<f64>();
    s.months += 1;
    p.month = m;
    // accounts (docs/rules/accounts.md): the teachers' pay among the wage bill
    // and the state's buying, the pensions and the tax among the transfers
    let (sch, pen, tx) = (st.purse.month.schools, st.purse.month.pensions, st.purse.month.tax);
    crate::accounts::note_state(st, sch, pen, tx);
}

/// What the state bought this month (the teachers), for the observer's output.
pub fn month_buying(st: &State) -> f64 {
    st.purse.month.schools
}

/// The purse's series, after the others: the rate on pay, tax, schools and
/// pensions over the month's income, the treasury in months of its spending,
/// the teachers and the pensioners.
pub fn add_series(reg: &mut Series, P: &Params, n_months: usize) {
    if !on(P) {
        return;
    }
    reg.add("state_rate", "The rate on pay", "state", "share", n_months);
    reg.add("state_tax_share", "Tax over income", "state", "share", n_months);
    reg.add("state_schools_share", "Schools over income", "state", "share", n_months);
    reg.add("state_pensions_share", "Pensions over income", "state", "share", n_months);
    reg.add("state_months_held", "The treasury, in months of its spending", "state", "months", n_months);
    reg.add("state_teachers", "Teachers", "state", "hands", n_months);
    reg.add("state_pensioners", "Pensioners", "state", "people", n_months);
}

/// The month's figures (`observer::record_month`), with `gdp` the month's output.
pub fn record_month(st: &State, P: &Params, s: &mut Series, t: usize, gdp: f64) {
    if !on(P) {
        return;
    }
    let p = &st.purse;
    let m = &p.month;
    let share = |x: f64| if x != 0.0 && gdp > 0.0 { x / gdp } else { 0.0 };
    s.set("state_rate", t, p.rate);
    s.set("state_tax_share", t, share(m.tax));
    s.set("state_schools_share", t, share(m.schools));
    s.set("state_pensions_share", t, share(m.pensions));
    let spend = m.schools + m.pensions;
    s.set("state_months_held", t, if spend > 0.0 { p.treasury / spend } else { 0.0 });
    s.set("state_teachers", t, m.teachers);
    s.set("state_pensioners", t, m.pensioners);
}

/// The island's people's share in towns, its share over 64 and its baskets a
/// head, as the year closes.
fn island_now(st: &State) -> (f64, f64, f64) {
    let (mut pop, mut urban, mut y) = (0.0, 0.0, 0.0);
    for t in 0..st.T_max {
        if st.town_active[t] {
            pop += st.pop[t];
            urban += st.urban_pop[t];
            y += st.pop[t] * st.y_ratio.get(t).copied().unwrap_or(0.0);
        }
    }
    let old = st.ages.years.last().map_or(0.0, |a| a.over64);
    if pop > 0.0 { (urban / pop, old, y / pop) } else { (0.0, old, 0.0) }
}

/// At a year's end (`sim::end_year`, after the ages close): the year put away,
/// the book read, next year's laws and their reach, each town's pupils,
/// teachers and pensioners, and the rate that balances next year's budget.
pub fn close_year(st: &mut State, C: &Content, P: &Params, year: i64, events: &mut Vec<Event>) {
    if !on(P) {
        return;
    }
    let T = st.T_max;
    let sp = &P.state;
    if st.purse.years.is_empty() && st.purse.cut == 0.0 {
        st.purse.cut = 1.0;
    }
    // ---- the year put away
    let sums = std::mem::take(&mut st.purse.now);
    let n = sums.months.max(1) as f64;
    let income = st.series.index.get("gdp_nom").map_or(f64::NAN, |&s| {
        let first = ((year - C.start_year) * 12).max(0) as usize;
        st.series.data[s].iter().skip(first).take(12).sum::<f64>()
    });
    let spend = sums.schools + sums.pensions;
    let halves = |key: &dyn Fn(usize) -> f64| -> (f64, f64) {
        // towns ranked by `key`, split where half the people are reached
        let mut ts: Vec<usize> = (0..T).filter(|&t| st.town_active[t] && st.pop[t] > 0.0 && t < sums.town_before.len()).collect();
        ts.sort_by(|&a, &b| key(b).total_cmp(&key(a)).then(a.cmp(&b)));
        let all: f64 = ts.iter().map(|&t| st.pop[t]).sum();
        let (mut seen, mut hi, mut lo) = (0.0, (0.0, 0.0), (0.0, 0.0));
        for &t in &ts {
            let side = if seen < 0.5 * all { &mut hi } else { &mut lo };
            side.0 += sums.town_net[t];
            side.1 += sums.town_before[t];
            seen += st.pop[t];
        }
        let r = |x: (f64, f64)| if x.1 > 0.0 { x.0 / x.1 } else { f64::NAN };
        (r(hi), r(lo))
    };
    let share_of = |t: usize, f: &dyn Fn(&[f64; crate::ages::GROUPS]) -> f64| -> f64 {
        st.ages.rows.get(t).map_or(0.0, |r| {
            let all: f64 = r.iter().sum();
            if all > 0.0 { f(r) / all } else { 0.0 }
        })
    };
    let (old_many, old_few) = halves(&|t| share_of(t, &|r| over(r, 65.0)));
    let (pupils_many, pupils_few) = halves(&|t| share_of(t, &|r| r[1] + r[2]));
    let p = &st.purse;
    let row = PurseYear {
        year,
        rate: p.rate,
        tax: sums.tax,
        schools: sums.schools,
        pensions: sums.pensions,
        base: sums.base,
        treasury: p.treasury,
        months_held: if spend > 0.0 { p.treasury / (spend / n) } else { 0.0 },
        income,
        teachers: sums.teachers / n,
        teachers_want: sums.teachers_want / n,
        hands: sums.hands / n,
        pensioners: sums.pensioners / n,
        pupils: sums.pupils / n,
        school_law: p.school_law,
        school_level: p.school_level,
        pension_age: p.pension_age,
        pension_share: p.pension_share,
        cut: p.cut,
        short_months: sums.short_months,
        net_old_many: old_many,
        net_old_few: old_few,
        net_pupils_many: pupils_many,
        net_pupils_few: pupils_few,
    };
    st.purse.years.push(row);
    let base_now = sums.base - sums.base_schools;
    st.purse.base_last = base_now;
    st.purse.base_hist.push(base_now);
    if st.purse.base_hist.len() > GROWTH_YEARS + 1 {
        st.purse.base_hist.remove(0);
    }
    // what this year's programmes cost a month at December's wages, as they
    // were paid (cut and all): next year's are set against it, so that the
    // season's wage and the year's noise cancel
    let wage = |t: usize| st.wage.get(t).copied().unwrap_or(0.0).max(0.0);
    let cost_now: f64 = {
        let p = &st.purse;
        (0..T)
            .map(|t| {
                let teach = p.teachers_want.get(t).copied().unwrap_or(0.0);
                let pens = p.pensioners.get(t).copied().unwrap_or(0.0) * p.pension_share * p.cut;
                (teach + pens) * wage(t)
            })
            .sum()
    };

    // ---- the book: the acts met this year
    let (towns, old, baskets) = island_now(st);
    let p = &mut st.purse;
    if sp.book == Book::Default && year >= book::FROM {
        let mut said: Vec<(String, String)> = Vec::new();
        if p.schools_step == 0 && towns > book::SCHOOLS_TOWNS
            || (1..3).contains(&p.schools_step) && year >= p.schools_year + book::GENERATION
        {
            let y = book::SCHOOL_STEPS[p.schools_step];
            p.schools_step += 1;
            p.schools_year = year;
            p.acts.push(Act { year, what: "schools".into(), at: y, share: 0.0 });
            said.push((
                format!("A schools act: {y} years of school for every child"),
                if p.schools_step == 1 {
                    format!("With {:.0} in a hundred of the island's people in its towns, the treasury is to pay for every child's schooling from five, {y} years of it, a teacher to fourteen pupils, out of a rate on pay. The schools are to grow to it as fast as a trade can.", 100.0 * towns)
                } else {
                    format!("A generation after the last schools act, the years of school the treasury pays for every child are raised to {y}.")
                },
            ));
        }
        if p.pension_step == 0 && old > book::PENSION_OLD {
            p.pension_step = 1;
            let (age, share) = book::PENSION_FIRST;
            p.acts.push(Act { year, what: "pension".into(), at: age, share });
            said.push((
                format!("A pension at {age}"),
                format!("With {:.1} in a hundred of the people over 64, everyone of {age} or more is to be paid {:.0} in a hundred of their town's wage out of the treasury.", 100.0 * old, 100.0 * share),
            ));
        }
        if p.pension_step == 1 && baskets > book::PENSION_RICH {
            p.pension_step = 2;
            let (age, share) = book::PENSION_LATER;
            p.acts.push(Act { year, what: "pension".into(), at: age, share });
            said.push((
                format!("The pension paid from {age}"),
                format!("The island's households spend {baskets:.1} baskets a head: the pension is paid from {age}, at a quarter of the town's wage."),
            ));
        }
        for (title, text) in said {
            events.push(Event::new(year as f64 + 0.95, "state", 0.7, title, text));
        }
    }
    // the laws: the book's, or a chip's from the year it is set
    let book_years = if p.schools_step > 0 { book::SCHOOL_STEPS[p.schools_step - 1] } else { 0.0 };
    let (book_age, book_share) = match p.pension_step {
        0 => (0.0, 0.0),
        1 => book::PENSION_FIRST,
        _ => book::PENSION_LATER,
    };
    p.school_law = if sp.school_years >= 0.0 { sp.school_years } else { book_years };
    p.pension_age = if sp.pension_age >= 0.0 { sp.pension_age } else { book_age };
    p.pension_share = if sp.pension_share >= 0.0 { sp.pension_share } else { book_share };
    if p.pension_age <= 0.0 && p.pension_share > 0.0 {
        // a share by hand with no age from the book: the book's first age
        p.pension_age = book::PENSION_FIRST.0;
    }
    // the programme grows toward its law by what a trade may add in a year
    let step = sp.year_room * p.school_law;
    p.school_level = if p.school_level < p.school_law { (p.school_level + step).min(p.school_law) } else { p.school_law };

    // ---- next year's pupils, teachers and pensioners, town by town
    p.pupils.resize(T, 0.0);
    p.pensioners.resize(T, 0.0);
    p.teachers_want.resize(T, 0.0);
    for t in 0..T {
        let row = if st.town_active[t] { st.ages.rows.get(t).copied() } else { None };
        let (pu, pe) = match row {
            Some(r) => (pupils(&r, p.school_level), if p.pension_share > 0.0 { over(&r, p.pension_age) } else { 0.0 }),
            None => (0.0, 0.0),
        };
        p.pupils[t] = pu;
        p.pensioners[t] = pe;
    }

    // ---- the budget: next year's rate, so that its revenue covers the
    // programmes and brings the treasury to `reserve_months` of its spending
    // pay grows as it has grown over the last `GROWTH_YEARS` (a single year's
    // growth runs back the next)
    let h = &p.base_hist;
    let g = match (h.first(), h.last()) {
        (Some(&a), Some(&z)) if h.len() > 1 && a > 0.0 && z > 0.0 => ((z / a).powf(1.0 / (h.len() - 1) as f64) - 1.0).clamp(-0.05, 0.1),
        _ => 0.0,
    };
    // next year's programmes at December's wages, at their whole law
    let teach_dec: f64 = (0..T).map(|t| p.pupils[t] * sp.school_hands * wage(t)).sum();
    let pens_dec: f64 = (0..T).map(|t| p.pensioners[t] * p.pension_share * wage(t)).sum();
    // scaled to what this year's cost at December's wages was really paid over the
    // year; the first year of a programme has nothing to scale on, and takes twelve
    // months at December's wages
    let spent_now = sums.schools + sums.pensions;
    let scale = if cost_now > 0.0 && spent_now > 0.0 { spent_now / cost_now } else { 12.0 };
    let teach_next = teach_dec * scale * (1.0 + g);
    let spend_next = (teach_dec + pens_dec) * scale * (1.0 + g);
    let base_next = base_now * (1.0 + g) + teach_next;
    let keep = sp.reserve_months / 12.0;
    let (mut rate, mut cut) = (0.0, 1.0);
    if spend_next > 0.0 && base_next > 0.0 {
        rate = (spend_next * (1.0 + keep) - p.treasury) / base_next;
        if rate > sp.rate_cap {
            // the programmes cut in proportion, so that the cap covers them and the reserve
            cut = ((sp.rate_cap * base_next + p.treasury) / (spend_next * (1.0 + keep))).clamp(0.0, 1.0);
            rate = sp.rate_cap;
            events.push(Event::new(
                year as f64 + 0.96,
                "state",
                0.8,
                "The programmes are cut".to_string(),
                format!(
                    "Paying for the schools and pensions in full would want a rate of {:.0} in a hundred on pay, more than the {:.0} the treasury may take. Every programme is cut to {:.0} in a hundred of its law for the coming year.",
                    100.0 * (spend_next * (1.0 + keep) - p.treasury) / base_next,
                    100.0 * sp.rate_cap,
                    100.0 * cut
                ),
            ));
        }
        rate = rate.max(0.0);
    }
    p.rate = rate;
    p.cut = cut;
    for t in 0..T {
        p.teachers_want[t] = p.pupils[t] * sp.school_hands * cut;
    }
    // the years of school paid this coming year, for the cohorts at school
    if p.school_level > 0.0 || !p.level_hist.is_empty() {
        p.level_hist.push(p.school_level * cut);
        if p.level_hist.len() > 10 {
            p.level_hist.remove(0);
        }
    }
    if !acting(st) {
        st.purse.teachers_want.clear();
    }
}

/// The years the facts are read at, with the run's last.
pub const MARKS: [i64; 5] = [1925, 1950, 1975, 2000, 2025];

/// The facts (docs/rules/state.md), under "Work and wages": at `MARKS` inside
/// the run and its last year, the treasury in months of its spending, the rate
/// on pay, tax, schools and pensions over income, teachers' share of all
/// hands, pensioners, the working ages' years of school, and the state's net
/// flow in the older and younger towns and those with more and fewer pupils;
/// over the run the year of each act, the least and most months held from the
/// first act, the years cut and the months the treasury fell short. Nothing in
/// a run without the purse.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let st = &r.state;
    let mut out = Vec::new();
    let ys = &st.purse.years;
    if !r.params.state.enabled || ys.is_empty() {
        return out;
    }
    let (y0, y1) = (ys[0].year, ys[ys.len() - 1].year);
    let at = |y: i64| ys.iter().find(|v| v.year == y);
    let mut marks: Vec<i64> = MARKS.iter().copied().filter(|&y| y > y0 && y < y1).collect();
    marks.push(y1);
    let over_income = |v: &PurseYear, x: f64| if v.income > 0.0 { x / v.income } else { f64::NAN };
    for y in marks {
        let v = at(y);
        let f = |g: &dyn Fn(&PurseYear) -> f64| v.map_or(f64::NAN, g);
        out.push((format!("state_months_held_{y}"), Kind::Real, f(&|v| v.months_held)));
        out.push((format!("state_rate_{y}"), Kind::Share, f(&|v| v.rate)));
        out.push((format!("state_tax_share_{y}"), Kind::Share, f(&|v| over_income(v, v.tax))));
        out.push((format!("state_schools_share_{y}"), Kind::Share, f(&|v| over_income(v, v.schools))));
        out.push((format!("state_pensions_share_{y}"), Kind::Share, f(&|v| over_income(v, v.pensions))));
        out.push((format!("state_teachers_share_{y}"), Kind::Share, f(&|v| if v.hands > 0.0 { v.teachers / v.hands } else { f64::NAN })));
        out.push((format!("state_pensioners_{y}"), Kind::Count, f(&|v| v.pensioners)));
        let school = st.ages.years.iter().find(|a| a.year == y).map_or(f64::NAN, |a| a.school);
        out.push((format!("state_school_{y}"), Kind::Real, school));
        out.push((format!("state_net_old_many_{y}"), Kind::Real, f(&|v| v.net_old_many)));
        out.push((format!("state_net_old_few_{y}"), Kind::Real, f(&|v| v.net_old_few)));
        out.push((format!("state_net_pupils_many_{y}"), Kind::Real, f(&|v| v.net_pupils_many)));
        out.push((format!("state_net_pupils_few_{y}"), Kind::Real, f(&|v| v.net_pupils_few)));
    }
    let act = |what: &str, at: f64| st.purse.acts.iter().find(|a| a.what == what && a.at == at).map_or(f64::NAN, |a| a.year as f64);
    for y in book::SCHOOL_STEPS {
        out.push((format!("state_act_schools_{y}"), Kind::Year, act("schools", y)));
    }
    out.push(("state_act_pension_70".into(), Kind::Year, act("pension", book::PENSION_FIRST.0)));
    out.push(("state_act_pension_65".into(), Kind::Year, act("pension", book::PENSION_LATER.0)));
    // from the first year anything was spent: the least and most months held at a year's end
    let paying: Vec<&PurseYear> = ys.iter().filter(|v| v.schools + v.pensions > 0.0).collect();
    let held: Vec<f64> = paying.iter().map(|v| v.months_held).collect();
    out.push(("state_first_paid".into(), Kind::Year, paying.first().map_or(f64::NAN, |v| v.year as f64)));
    out.push(("state_months_held_least".into(), Kind::Real, held.iter().copied().fold(f64::NAN, f64::min)));
    out.push(("state_months_held_most".into(), Kind::Real, held.iter().copied().fold(f64::NAN, f64::max)));
    out.push(("state_cut_years".into(), Kind::Count, ys.iter().filter(|v| v.cut < 1.0).count() as f64));
    out.push(("state_short_months".into(), Kind::Count, ys.iter().map(|v| v.short_months).sum::<usize>() as f64));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pupils_and_pensioners_are_counted_by_age() {
        let row = [10.0; crate::ages::GROUPS];
        // six years from five: ages 5 to 10, two a year of each group's ten
        assert!((pupils(&row, 6.0) - 12.0).abs() < 1e-12);
        assert!((pupils(&row, 0.0)).abs() < 1e-12);
        // eleven years reach 15
        assert!((pupils(&row, 11.0) - 22.0).abs() < 1e-12);
        // 70 and over: 70-74, 75-79 and 80+
        assert!((over(&row, 70.0) - 30.0).abs() < 1e-12);
        assert!((over(&row, 67.5) - 35.0).abs() < 1e-12);
    }

    #[test]
    fn a_cohort_is_given_the_years_paid_while_it_was_at_school() {
        let mut st = State::default();
        assert_eq!(cohort_years(&st), 0.0);
        st.purse.level_hist = vec![6.0; 10];
        assert!((cohort_years(&st) - 6.0).abs() < 1e-12);
        // a law of six two years ago: the cohort was 12 then, past six years from five
        st.purse.level_hist = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 6.0, 6.0];
        assert!(cohort_years(&st).abs() < 1e-12);
        st.purse.level_hist = vec![11.0; 10];
        assert!((cohort_years(&st) - 11.0).abs() < 1e-12);
    }
}
