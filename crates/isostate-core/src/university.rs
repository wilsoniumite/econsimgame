//! The university: a programme of the treasury with a place. Not in the
//! Python; `docs/rules/state.md`, "The university", is the longer account.
//!
//! - **The seat.** When the treasury's first programme begins, the island's
//!   seat is the town with the most townsfolk that year, and it stays the seat.
//! - **Founded by the book** (`university`, under `state.university`): the
//!   first in the seat `book::FIRST_AFTER` years after the first schools act,
//!   a second in the largest other town `book::SECOND_AFTER` years after that.
//!   By hand (the sandbox's Works tool) one may be founded in any town and any
//!   one closed.
//! - **Students.** At each year's end the year's intake, the book's share of
//!   each town's eighteen-year-olds (`book::SHARE`, history's), goes from its
//!   town to the nearest university with its cash a head, its ages and
//!   schooling and its town's measure of how well it lives, as movers do. A
//!   student is taught `university_years`; while taught it is townsfolk of the
//!   university's town and not one of its hands for hire. After, it is one of
//!   the town's people and follows jobs like anyone. A cohort's years at the
//!   university are added to its years of school as it turns fifteen.
//! - **Its staff** (`university_hands` a student, two thirds teaching, which
//!   is judging, and a third clerking) are hired as the schools' teachers are:
//!   out of the town's hands before the wage is sought, counted at work, paid
//!   the town's wage out of the treasury.
//! - **Its rooms** (`university_rooms` units of building work a student) are
//!   plant built by the builders of its town: the treasury orders building
//!   work there as a railway orders its materials, pays the merchants for what
//!   comes, and the rooms wear out at `ROOMS_WEAR` a year.
//!
//! What it costs falls on the rate on pay with the schools and the pensions
//! (`purse::close_year` sets the budget with it). With the switch off, or the
//! purse off, every hook returns at once: a run is the same to the byte.

use serde::Serialize;

use crate::content::Content;
use crate::facts::Kind;
use crate::ledger::{Fold, Line};
use crate::mat::Mat;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;

/// The book's dates and history's shares.
pub mod book {
    /// The first university is founded this many years after the first schools act.
    pub const FIRST_AFTER: i64 = 5;
    /// The second, about two generations after the first.
    pub const SECOND_AFTER: i64 = 50;
    /// History's share of an age group entering higher education, a country of
    /// Britain's institutions: the Robbins report's table for 1900 to 1962 (all
    /// full-time higher education, since the island's one university stands for
    /// all of it), the age participation index for 1970 to 2000, and the
    /// participation rate after. Between the years it is drawn straight; before
    /// the first and after the last it stands.
    pub const SHARE: [(i64, f64); 11] = [
        (1900, 0.012),
        (1938, 0.027),
        (1954, 0.058),
        (1962, 0.085),
        (1970, 0.14),
        (1980, 0.13),
        (1990, 0.19),
        (2000, 0.33),
        (2010, 0.45),
        (2020, 0.50),
        (2050, 0.50),
    ];
}

/// The book's share of a cohort in `year`.
pub fn book_share(year: i64) -> f64 {
    let s = &book::SHARE;
    if year <= s[0].0 {
        return s[0].1;
    }
    for w in s.windows(2) {
        let ((y0, a), (y1, b)) = (w[0], w[1]);
        if year <= y1 {
            return a + (b - a) * (year - y0) as f64 / (y1 - y0) as f64;
        }
    }
    s[s.len() - 1].1
}

/// The share of its rooms that wears out in a year.
pub const ROOMS_WEAR: f64 = 0.02;
/// The share of its staff that teaches (judging); the rest are its clerks.
pub const JUDGING: f64 = 2.0 / 3.0;
/// The most of a town's hands left after the teachers that its university's
/// students and staff may take out in a month.
pub const HANDS_MOST: f64 = 0.25;
/// The group a student is of: fifteen to nineteen.
const STUDENTS_GROUP: usize = crate::ages::WORKING.start;
/// The most of its money the treasury puts into rooms in a month: a sixth.
const ROOMS_PURSE: f64 = 1.0 / 6.0;

/// One university.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Uni {
    pub town: usize,
    /// its square: the town's centre as it was founded
    pub cell: usize,
    pub founded: i64,
    /// the year it was closed by hand
    pub closed: Option<i64>,
    pub by_hand: bool,
    /// the intake of each of its last years, oldest first (students are the sum)
    pub intake: Vec<f64>,
    /// its students at each year's end, as the year's intake came: (year, students)
    pub history: Vec<(i64, f64)>,
}

impl Uni {
    pub fn open(&self) -> bool {
        self.closed.is_none()
    }
    pub fn students(&self) -> f64 {
        self.intake.iter().sum()
    }
}

/// One year of the university, as it closed.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct UniYear {
    pub year: i64,
    /// the share of the cohort taken in at the year's end
    pub share: f64,
    /// students over the year (mean of the months), and staff
    pub students: f64,
    pub staff: f64,
    /// what the treasury paid its staff and for its rooms over the year
    pub staff_pay: f64,
    pub rooms_paid: f64,
    /// building work standing as rooms at the year's end
    pub rooms: f64,
    /// the share of those aged 15 to 64 who have been taken in
    pub been: f64,
    /// the year's intake, and how many of it moved town
    pub intake: f64,
    pub moved: f64,
}

/// What the university keeps (`st.purse.uni`). Empty while it is off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UniState {
    /// the island's seat, and the year it was chosen
    pub seat: Option<usize>,
    pub seat_year: i64,
    pub unis: Vec<Uni>,
    /// the book's foundings met: 0 to 2
    pub book_step: usize,
    /// this year's share of a cohort
    pub share: f64,
    /// each town's students, staff wanted and staff hired this month
    pub students: Vec<f64>,
    pub staff_want: Vec<f64>,
    pub staff: Vec<f64>,
    /// each town's hands held out of work this month as students
    pub held: Vec<f64>,
    /// each town's rooms standing, in units of building work
    pub rooms: Vec<f64>,
    /// the share of each cohort taken in, by the year it was taken in
    pub been: Vec<(i64, f64)>,
    /// the month's pay of the staff and the rooms paid for
    pub month_pay: f64,
    pub month_rooms: f64,
    /// the year's sums: staff pay, rooms paid for, students and staff (sums of the months), months
    pub sum_pay: f64,
    pub sum_rooms: f64,
    pub sum_students: f64,
    pub sum_staff: f64,
    pub sum_months: usize,
    pub years: Vec<UniYear>,
}

/// Whether the university acts.
pub fn on(P: &Params) -> bool {
    P.state.enabled && P.state.university
}

/// The years a student is taught, as a whole number of years' intakes.
fn years_taught(P: &Params) -> usize {
    P.state.university_years.round().max(1.0) as usize
}

/// The seat: when the treasury's first programme begins (the first act is met),
/// the town with the most townsfolk that year, kept from then on. Called from
/// `purse::close_year` once the book is read, whenever the purse is on.
pub fn choose_seat(st: &mut State, year: i64) {
    if st.purse.uni.seat.is_some() || st.purse.acts.is_empty() {
        return;
    }
    let seat = (0..st.T_max).filter(|&t| st.town_active[t]).max_by(|&a, &b| st.urban_pop[a].total_cmp(&st.urban_pop[b]).then(b.cmp(&a)));
    if let Some(t) = seat {
        st.purse.uni.seat = Some(t);
        st.purse.uni.seat_year = year;
    }
}

/// The square a university of town `t` stands on: the nearest to the town's
/// centre (within three squares each way) that is of its district, on which a
/// works could be built (`families::buildable`) and that has no water beside it
/// or through it (the sea or a lake beside it, a tideway or a lake in it), the
/// ground that is not yet the town's first, as a college takes a field at the
/// town's edge, and then the least built on by works; failing all, the centre.
fn site(st: &State, t: usize) -> usize {
    let node = st.town_node[t];
    let W = st.world.W as i64;
    let H = (st.N as i64) / W.max(1);
    let (x0, y0) = (node as i64 % W, node as i64 / W);
    let lf = st.world.landform.as_ref();
    // a square by water, or with water through it: the sea or a lake beside it, a lake or a tideway in it
    let wet = |c: usize| {
        let (x, y) = (c as i64 % W, c as i64 / W);
        let by_sea = [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|&(dx, dy)| {
            let (a, b) = (x + dx, y + dy);
            a < 0 || b < 0 || a >= W || b >= H || !st.world.land[(b * W + a) as usize]
        });
        by_sea || lf.is_some_and(|l| l.shore.get(c).copied().unwrap_or(false) || l.estuary.get(c).copied().unwrap_or(false) || l.lake_share.get(c).copied().unwrap_or(0.0) > 0.05)
    };
    let mut best: Option<(i64, i64, f64, usize)> = None;
    for dy in -3..=3i64 {
        for dx in -3..=3i64 {
            let (x, y) = (x0 + dx, y0 + dy);
            if x < 0 || y < 0 || x >= W || y >= H {
                continue;
            }
            let c = (y * W + x) as usize;
            if crate::families::buildable(st, c) != Ok(t) || wet(c) {
                continue;
            }
            // ground not yet the town's first, as a college takes a field at the town's edge
            let built = st.town_built.get(c).is_some_and(|&y| y != 0) as i64;
            let key = (built, dx * dx + dy * dy, st.cell_K_share[c], c);
            if best.is_none_or(|b| (key.0, key.1, key.2) < (b.0, b.1, b.2)) {
                best = Some(key);
            }
        }
    }
    best.map_or(node, |b| b.3)
}

/// A university founded in town `t`: its place, and the title and text of its
/// news. Founded by the book at a year's end, its first students come at once;
/// by hand at the year's turn, at the year's end.
pub fn found(st: &mut State, t: usize, year: i64, by_hand: bool) -> (String, String) {
    let cell = site(st, t);
    let name = st.town_name[t].clone();
    let n = st.purse.uni.unis.len();
    st.purse.uni.unis.push(Uni { town: t, cell, founded: year, closed: None, by_hand, intake: Vec::new(), history: Vec::new() });
    let seat = st.purse.uni.seat == Some(t);
    let title = format!("The University of {name}");
    let text = if by_hand {
        format!("A university is founded by hand in {name}. The treasury pays its teachers and clerks and has its rooms built; at the year's end the young of the towns nearer it than any other go there to be taught after school.")
    } else if n == 0 {
        format!(
            "{} the treasury founds the island's first university in {name}{}. It pays its teachers and clerks out of the rate on pay and has its rooms built by the town's builders; every town sends it some of its young after school, and they live in {name} while they are taught.",
            if seat { "In the island's seat," } else { "In" },
            if seat { "" } else { ", the largest town" }
        )
    } else {
        format!("Two generations after the first, the treasury founds a second university in {name}, the largest town without one. The young of the towns nearer it than the first go there.")
    };
    (title, text)
}

/// A university closed by hand: its students are people of its town from now on,
/// and its rooms stand empty.
pub fn close(st: &mut State, t: usize, year: i64) -> Option<(String, String)> {
    let u = st.purse.uni.unis.iter_mut().find(|u| u.town == t && u.open())?;
    u.closed = Some(year);
    let had = u.students();
    u.intake.clear();
    let u = &mut st.purse.uni;
    for v in [&mut u.students, &mut u.staff_want] {
        if let Some(x) = v.get_mut(t) {
            *x = 0.0;
        }
    }
    let name = st.town_name[t].clone();
    let title = format!("The University of {name} is closed");
    let text = format!("The university in {name} is closed by hand. Its {} students are people of the town from now on, and its rooms stand empty.", crate::util::fmt_thousands_f0(had));
    Some((title, text))
}

/// The universities for the record's header (`record::pack`): each one's town,
/// square, founding, closing and students year by year. Nothing while the
/// university is off.
pub fn header(st: &State, P: &Params) -> Option<serde_json::Value> {
    if !on(P) {
        return None;
    }
    let u = &st.purse.uni;
    let unis: Vec<serde_json::Value> = u
        .unis
        .iter()
        .map(|v| {
            serde_json::json!({"town": v.town, "cell": v.cell, "founded": v.founded, "closed": v.closed, "by_hand": v.by_hand,
                "name": format!("The University of {}", st.town_name.get(v.town).map_or("", |s| s.as_str())), "students": v.history})
        })
        .collect();
    Some(serde_json::json!({"seat": u.seat, "seat_year": u.seat_year, "universities": unis}))
}

/// Whether a university may be founded in town `t`, or why not.
pub fn may_found(st: &State, P: &Params, t: usize) -> Result<(), String> {
    if !on(P) {
        return Err("the university is not among the island's rules (state.university is off)".into());
    }
    if !(t < st.T_max && st.town_active[t]) {
        return Err(format!("there is no town {t} standing on the island"));
    }
    if st.purse.uni.unis.iter().any(|u| u.town == t && u.open()) {
        return Err(format!("{} has a university already", st.town_name[t]));
    }
    Ok(())
}

/// Whether town `t` has a university to close, or why not.
pub fn may_close(st: &State, P: &Params, t: usize) -> Result<(), String> {
    if !on(P) {
        return Err("the university is not among the island's rules (state.university is off)".into());
    }
    if t >= st.T_max || !st.purse.uni.unis.iter().any(|u| u.town == t && u.open()) {
        return Err(format!("there is no university open in {}", st.town_name.get(t).map_or("that town", |s| s.as_str())));
    }
    Ok(())
}

/// The town of the open university nearest town `t` (by the cost of the way),
/// if any is open.
fn nearest(st: &State, t: usize) -> Option<usize> {
    let open: Vec<usize> = st.purse.uni.unis.iter().filter(|u| u.open() && st.town_active[u.town]).map(|u| u.town).collect();
    open.iter()
        .copied()
        .min_by(|&a, &b| {
            let c = |u: usize| if u == t { 0.0 } else { st.town_cost[(t, u)] };
            let (ca, cb) = (c(a), c(b));
            let k = |x: f64| if x.is_finite() { x } else { f64::MAX };
            k(ca).total_cmp(&k(cb)).then(a.cmp(&b))
        })
}

/// The share of those aged 15 to 64 who have been taken in, on the island's
/// rows: an age `a` this year was eighteen in `year - a + 18`.
fn been_share(st: &State, year: i64) -> f64 {
    let rows = &st.ages.rows;
    if rows.is_empty() || st.purse.uni.been.is_empty() {
        return 0.0;
    }
    let mut g = [0.0; crate::ages::GROUPS];
    for (t, r) in rows.iter().enumerate() {
        if st.town_active.get(t).copied().unwrap_or(false) {
            for a in 0..crate::ages::GROUPS {
                g[a] += r[a];
            }
        }
    }
    let share_in = |y: i64| st.purse.uni.been.iter().find(|b| b.0 == y).map_or(0.0, |b| b.1);
    let (mut been, mut all) = (0.0, 0.0);
    for age in 15..65i64 {
        let n = g[(age / 5) as usize] / 5.0;
        all += n;
        if age >= 18 {
            been += n * share_in(year - age + 18);
        }
    }
    if all > 0.0 { been / all } else { 0.0 }
}

/// What next year's university will cost a month at December's wages and
/// prices, for the budget: its staff, and the rooms it will order over the year
/// (`purse::close_year`).
pub struct Cost {
    pub staff_now: f64,
    pub staff_next: f64,
    pub rooms_next: f64,
    pub paid: f64,
}

/// At a year's end (`purse::close_year`, after the laws are set and before the
/// budget): the year put away, the book's foundings, the year's intake moved
/// to the universities, and next year's students and staff. `news` takes the
/// foundings. Returns what the budget needs.
pub fn close_year(st: &mut State, C: &Content, P: &Params, year: i64, news: &mut Vec<(f64, String, String)>) -> Cost {
    let none = Cost { staff_now: 0.0, staff_next: 0.0, rooms_next: 0.0, paid: 0.0 };
    if !on(P) {
        return none;
    }
    let T = st.T_max;
    let wage = |st: &State, t: usize| st.wage.get(t).copied().unwrap_or(0.0).max(0.0);
    // what this year's staff cost a month at December's wages, before next year's are set
    let staff_now: f64 = (0..T).map(|t| st.purse.uni.staff_want.get(t).copied().unwrap_or(0.0) * wage(st, t)).sum();
    // ---- the year put away
    let u = &mut st.purse.uni;
    let n = u.sum_months.max(1) as f64;
    let mut row = UniYear {
        year,
        share: 0.0,
        students: u.sum_students / n,
        staff: u.sum_staff / n,
        staff_pay: u.sum_pay,
        rooms_paid: u.sum_rooms,
        rooms: u.rooms.iter().sum(),
        been: 0.0,
        intake: 0.0,
        moved: 0.0,
    };
    let paid = u.sum_pay;
    (u.sum_pay, u.sum_rooms, u.sum_students, u.sum_staff, u.sum_months) = (0.0, 0.0, 0.0, 0.0, 0);

    // ---- the book's foundings
    if P.state.book == crate::purse::Book::Default {
        let first_schools = st.purse.acts.iter().find(|a| a.what == "schools").map(|a| a.year);
        let u = &st.purse.uni;
        let want = match (u.book_step, first_schools, u.seat) {
            (0, Some(y), Some(seat)) if year >= y + book::FIRST_AFTER && !u.unis.iter().any(|v| v.town == seat && v.open()) => Some(seat),
            (1, _, _) => {
                let first = u.unis.iter().find(|v| !v.by_hand).map_or(year, |v| v.founded);
                if year >= first + book::SECOND_AFTER {
                    (0..T)
                        .filter(|&t| st.town_active[t] && !u.unis.iter().any(|v| v.town == t && v.open()))
                        .max_by(|&a, &b| st.urban_pop[a].total_cmp(&st.urban_pop[b]).then(b.cmp(&a)))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(t) = want {
            st.purse.uni.book_step += 1;
            let (title, text) = found(st, t, year, false);
            // the island's first university is among the heaviest news of its decade
            // (docs/rules/chronicle.md, "Ranking"), the second a little lighter
            news.push((if st.purse.uni.book_step == 1 { 0.95 } else { 0.85 }, title, text));
        } else if st.purse.uni.book_step == 0 && first_schools.is_some_and(|y| year >= y + book::FIRST_AFTER) && st.purse.uni.unis.iter().any(|v| Some(v.town) == st.purse.uni.seat) {
            // (the seat has one by hand already: the book's first is met by it)
            st.purse.uni.book_step = 1;
        }
    }

    // ---- the year's intake, to the nearest university
    let any_open = st.purse.uni.unis.iter().any(|v| v.open() && st.town_active[v.town]);
    let share = if !any_open {
        0.0
    } else if P.state.university_share >= 0.0 {
        P.state.university_share
    } else {
        book_share(year)
    };
    st.purse.uni.share = share;
    let keep = years_taught(P);
    let mut moves: Vec<(usize, usize, f64)> = Vec::new();
    let mut into = vec![0.0; T];
    if share > 0.0 && st.ages.rows.len() == T {
        for t in (0..T).filter(|&t| st.town_active[t]) {
            let Some(to) = nearest(st, t) else { continue };
            let n = (share.min(1.0) * st.ages.rows[t][STUDENTS_GROUP] / 5.0).min(st.pop[t].max(0.0));
            if !(n > 0.0) {
                continue;
            }
            into[to] += n;
            row.intake += n;
            if to != t {
                moves.push((t, to, n));
                row.moved += n;
            }
        }
    }
    for v in st.purse.uni.unis.iter_mut().filter(|v| v.open()) {
        v.intake.push(into[v.town]);
        while v.intake.len() > keep {
            v.intake.remove(0);
        }
        let n = v.students();
        v.history.push((year, n));
    }
    if share > 0.0 {
        st.purse.uni.been.push((year, share));
    }
    move_students(st, P, &moves);
    row.share = share;
    row.been = been_share(st, year);
    st.purse.uni.years.push(row);

    // ---- next year's students, staff and rooms, town by town
    let u = &mut st.purse.uni;
    u.students = vec![0.0; T];
    for v in u.unis.iter().filter(|v| v.open()) {
        u.students[v.town] += v.students();
    }
    u.staff_want = u.students.iter().map(|s| s * P.state.university_hands).collect();
    u.staff.resize(T, 0.0);
    u.held.resize(T, 0.0);
    u.rooms.resize(T, 0.0);
    let staff_next: f64 = (0..T).map(|t| st.purse.uni.staff_want[t] * wage(st, t)).sum();
    // the rooms it will order over the year, at December's prices
    let rooms_next: f64 = match C.good_index.get("building") {
        Some(&g) => (0..T)
            .map(|t| {
                let u = &st.purse.uni;
                let want = u.students[t] * P.state.university_rooms;
                ((want - u.rooms[t]).max(0.0) + u.rooms[t] * ROOMS_WEAR) * st.price[(t, g)]
            })
            .sum(),
        None => 0.0,
    };
    Cost { staff_now, staff_next, rooms_next, paid }
}

/// The year's intake moved to its universities, as migrants move
/// (`people::migration`): its towns' measure of how well they live, its ages
/// and schooling, its people and its cash a head.
fn move_students(st: &mut State, P: &Params, moves: &[(usize, usize, f64)]) {
    if moves.is_empty() {
        return;
    }
    let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    let at: Vec<Option<usize>> = (0..st.T_max).map(|t| act.iter().position(|&a| a == t)).collect();
    let mut flow = Mat::zeros(act.len(), act.len());
    for &(i, j, n) in moves {
        if let (Some(a), Some(b)) = (at[i], at[j]) {
            flow[(a, b)] += n;
        }
    }
    crate::transition::migrate(st, P, &act, &flow);
    crate::ages::students(st, P, moves);
    let cash_pc: Vec<f64> = (0..st.T_max).map(|t| st.hh_cash[t].max(0.0) / st.pop[t].max(1.0)).collect();
    for &(i, j, n) in moves {
        let money = n * cash_pc[i];
        st.pop[i] -= n;
        st.pop[j] += n;
        st.hh_cash[i] -= money;
        st.hh_cash[j] += money;
    }
}

/// The students living in town `t`, who are its townsfolk whatever its works
/// (`towns::update_urban`); nothing while the university is off.
pub fn living(st: &State, P: &Params, t: usize) -> f64 {
    if !on(P) {
        return 0.0;
    }
    st.purse.uni.students.get(t).copied().unwrap_or(0.0)
}

/// The years at the university a cohort turning fifteen this year is offered
/// (`ages::year`, added to its years of school): this year's share times the
/// years taught.
pub fn cohort_years(st: &State, P: &Params) -> f64 {
    if !on(P) {
        return 0.0;
    }
    st.purse.uni.share * P.state.university_years
}

/// The students and the staff are not for hire (`econ::step_month`, after the
/// schools' teachers): each town's students are taken out of its hands at the
/// working ages' rate of work, and its staff are hired out of what is left,
/// together at most `HANDS_MOST` of it.
pub fn take_hands(st: &mut State, P: &Params, supply: &mut [f64]) {
    if !on(P) || st.purse.uni.students.is_empty() {
        return;
    }
    let T = supply.len();
    let held_part = crate::who_works::students_held(st, P);
    let u = &mut st.purse.uni;
    u.staff.resize(T, 0.0);
    u.held.resize(T, 0.0);
    for t in 0..T {
        let s = u.students.get(t).copied().unwrap_or(0.0);
        let want = u.staff_want.get(t).copied().unwrap_or(0.0) * st.purse.cut;
        if s <= 0.0 && want <= 0.0 {
            u.staff[t] = 0.0;
            u.held[t] = 0.0;
            continue;
        }
        // a student would be a hand at the working ages' rate: the island's
        // participation over the town's share at the working ages
        let rate = match st.ages.rows.get(t) {
            Some(r) => {
                let all: f64 = r.iter().sum();
                let working: f64 = r[crate::ages::WORKING].iter().sum();
                if working > 0.0 { (P.participation * all / working).min(1.0) } else { P.participation }
            }
            None => P.participation,
        };
        let most = (HANDS_MOST * supply[t]).max(0.0);
        let held = (s * rate).min(most);
        // who works (docs/rules/who-works.md): the count by age has the students
        // out of work already, so only the part still reckoned as the half is held
        let held = match held_part {
            Some(k) => (k * s * rate).min(most),
            None => held,
        };
        let staff = want.min(most - held).max(0.0);
        supply[t] -= held + staff;
        u.held[t] = held;
        u.staff[t] = staff;
    }
}

/// The staff are at work (`econ::step_month`, beside the teachers).
pub fn count_staff(st: &State, P: &Params, employment: &mut [f64]) {
    if !on(P) {
        return;
    }
    for (e, n) in employment.iter_mut().zip(&st.purse.uni.staff) {
        *e += n;
    }
}

/// What each town's university orders of building work this month, in units:
/// a twelfth of the rooms it lacks and what wears out, no more than
/// `ROOMS_PURSE` of the treasury will pay for at the town's price: the
/// ledger's line of the university's rooms (ledger.rs), a row for each town
/// that orders, in one column, building. None while nothing is wanted.
pub fn rooms_order(st: &State, C: &Content, P: &Params) -> Option<Line> {
    if !on(P) || st.purse.uni.students.is_empty() {
        return None;
    }
    let &g = C.good_index.get("building")?;
    let u = &st.purse.uni;
    let per = P.state.university_rooms * st.purse.cut;
    let mut out = Vec::new();
    let mut purse = (st.purse.treasury * ROOMS_PURSE).max(0.0);
    for t in 0..st.T_max {
        let want = u.students.get(t).copied().unwrap_or(0.0) * per;
        let have = u.rooms.get(t).copied().unwrap_or(0.0);
        if want <= 0.0 {
            continue;
        }
        let units = ((want - have).max(0.0) + have * ROOMS_WEAR) / 12.0;
        let price = st.price[(t, g)].max(1e-9);
        let units = units.min(purse / price);
        if units > 0.0 {
            purse -= units * price;
            out.push((t, units));
        }
    }
    if out.is_empty() {
        return None;
    }
    let town: Vec<usize> = out.iter().map(|x| x.0).collect();
    let ask = Mat::from_vec(out.len(), 1, out.iter().map(|x| x.1).collect());
    Some(Line::of_goods(vec![g], town.clone(), town, Fold::EachRow, ask))
}

/// The rooms that came this month (`econ::step_month`, beside the railways'
/// materials): put up, and paid for out of the treasury into the town's
/// merchants. `rooms` is the ledger's line of them, its got each order's units
/// got; the pay goes into `merch` and `sales_value`. Then every town's rooms
/// wear a month.
pub fn rooms_got(st: &mut State, P: &Params, rooms: &Line, merch: &mut [f64], sales_value: &mut [f64]) {
    if !on(P) {
        return;
    }
    let mut paid = 0.0;
    let g = rooms.good(0);
    for (r, &t) in rooms.town.iter().enumerate() {
        let n = rooms.got[(r, 0)];
        let pay = n * st.price[(t, g)];
        if !(pay > 0.0) {
            continue;
        }
        st.purse.uni.rooms[t] += n;
        merch[t] += pay;
        sales_value[t] += pay;
        paid += pay;
    }
    st.purse.treasury -= paid;
    if st.purse.treasury < 0.0 && st.purse.treasury > -1e-9 * paid.max(1.0) {
        st.purse.treasury = 0.0;
    }
    st.purse.uni.month_rooms = paid;
    st.purse.uni.sum_rooms += paid;
    let wear = 1.0 - (1.0 - ROOMS_WEAR).powf(1.0 / 12.0);
    for r in st.purse.uni.rooms.iter_mut() {
        *r *= 1.0 - wear;
    }
}

/// Each town's staff's pay this month at the town's wage, before any cut for
/// want of money (`purse::month`).
pub fn staff_pay(st: &State, P: &Params, w: &[f64]) -> Vec<f64> {
    let T = w.len();
    if !on(P) || st.purse.uni.staff.is_empty() {
        return vec![0.0; T];
    }
    (0..T).map(|t| st.purse.uni.staff.get(t).copied().unwrap_or(0.0) * w[t].max(0.0)).collect()
}

/// The month's sums (`purse::month`, once the staff are paid): `paid` is what
/// they were paid.
pub fn note_month(st: &mut State, P: &Params, paid: f64) {
    if !on(P) {
        return;
    }
    let u = &mut st.purse.uni;
    u.month_pay = paid;
    u.sum_pay += paid;
    u.sum_students += u.students.iter().sum::<f64>();
    u.sum_staff += u.staff.iter().sum::<f64>();
    u.sum_months += 1;
}

/// The staff of `year` at the four classes of pay's reckoning (hauling,
/// tending, clerking, judging), for `pay::hands_from`.
pub fn hands_by_class(st: &State, year: i64) -> [f64; 4] {
    match st.purse.uni.years.iter().rev().find(|v| v.year == year) {
        Some(v) if v.staff > 0.0 => [0.0, 0.0, v.staff * (1.0 - JUDGING), v.staff * JUDGING],
        _ => [0.0; 4],
    }
}

/// The marks the facts are read at, with the run's last year.
pub const MARKS: [i64; 4] = [1925, 1950, 2000, 2025];

/// The facts (docs/rules/state.md, "The university"), under "Work and wages":
/// the seat (its year, and at 1950, 2000 and the last year whether it is the
/// largest town, its townsfolk and their share of all), the universities'
/// foundings, and at `MARKS` and the last year the students, the share of a
/// cohort taken in, the share of the working ages who have been, its staff
/// over all hands and what it costs over income. Nothing without the purse.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, P, rec) = (&r.state, &r.params, &r.rec);
    let mut out = Vec::new();
    if !P.state.enabled {
        return out;
    }
    let u = &st.purse.uni;
    let y0 = r.content.start_year;
    let y1 = y0 + rec.n_years as i64 - 1;
    out.push(("state_seat_year".into(), Kind::Year, if u.seat.is_some() { u.seat_year as f64 } else { f64::NAN }));
    if let Some(seat) = u.seat {
        let T = rec.T;
        for y in [1900, 1950, 2000, y1].into_iter().filter(|&y| y >= u.seat_year && y <= y1) {
            let k = (y - y0) as usize;
            let towns: Vec<(usize, f64)> = (0..T).filter(|&t| rec.town_active[k * T + t] != 0).map(|t| (t, rec.town_urban[k * T + t] as f64)).collect();
            let all: f64 = towns.iter().map(|x| x.1).sum();
            let first = towns.iter().max_by(|a, b| a.1.total_cmp(&b.1)).map(|x| x.0);
            let mine = towns.iter().find(|x| x.0 == seat).map_or(0.0, |x| x.1);
            out.push((format!("state_seat_is_first_{y}"), Kind::Count, if first == Some(seat) { 1.0 } else { 0.0 }));
            out.push((format!("state_seat_townsfolk_{y}"), Kind::Count, mine));
            out.push((format!("state_seat_share_{y}"), Kind::Share, if all > 0.0 { mine / all } else { f64::NAN }));
        }
    }
    if !(P.state.university) {
        return out;
    }
    let book: Vec<&Uni> = u.unis.iter().filter(|v| !v.by_hand).collect();
    out.push(("uni_founded_first".into(), Kind::Year, book.first().map_or(f64::NAN, |v| v.founded as f64)));
    out.push(("uni_founded_second".into(), Kind::Year, book.get(1).map_or(f64::NAN, |v| v.founded as f64)));
    out.push(("uni_first_in_seat".into(), Kind::Count, match (book.first(), u.seat) {
        (Some(v), Some(s)) => (v.town == s) as i32 as f64,
        _ => f64::NAN,
    }));
    let mut marks: Vec<i64> = MARKS.iter().copied().filter(|&y| y < y1).collect();
    marks.push(y1);
    let purse_at = |y: i64| st.purse.years.iter().find(|v| v.year == y);
    for y in marks {
        let v = u.years.iter().find(|v| v.year == y);
        let f = |g: &dyn Fn(&UniYear) -> f64| v.map_or(f64::NAN, g);
        out.push((format!("uni_students_{y}"), Kind::Count, f(&|v| v.students)));
        out.push((format!("uni_share_{y}"), Kind::Share, f(&|v| v.share)));
        out.push((format!("uni_been_{y}"), Kind::Share, f(&|v| v.been)));
        let p = purse_at(y);
        out.push((format!("uni_staff_share_{y}"), Kind::Share, match (v, p) {
            (Some(v), Some(p)) if p.hands > 0.0 => v.staff / p.hands,
            _ => f64::NAN,
        }));
        out.push((format!("uni_cost_share_{y}"), Kind::Share, match (v, p) {
            (Some(v), Some(p)) if p.income > 0.0 => (v.staff_pay + v.rooms_paid) / p.income,
            _ => f64::NAN,
        }));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_book_share_is_history_drawn_straight() {
        assert_eq!(book_share(1850), 0.012);
        assert!((book_share(1966) - (0.085 + 0.055 * 4.0 / 8.0)).abs() < 1e-12);
        assert_eq!(book_share(2000), 0.33);
        assert_eq!(book_share(2100), 0.50);
    }
}
