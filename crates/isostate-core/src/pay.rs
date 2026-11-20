//! Pay by class of task, reckoned and not played. Not in the Python; it only
//! watches. `docs/rules/pay.md` is the longer account.
//!
//! The island has one wage a town, so it cannot say who loses when machines
//! take tasks. This reckons it from the hands the island plays, as the
//! analysis of 20 November 2026 did by hand (`docs/rules/pay.md`, "Where it
//! comes from"), with its two dials in the rules:
//!
//! - **Four classes**: hauling; tending, with the work only a person may do and
//!   every machine's minders; clerking; judging. Their hands are the works',
//!   the land's and the carters' together (`hands_of`, from `TasksYear`).
//! - **What the island hires of a class at one wage is the demand at equal
//!   pay.** Demand at pay `p` is that hiring times `p` to the power of minus
//!   `bend`.
//! - **Hands follow the work.** In 1900 each class has the hands its work
//!   takes; each year after, its hands move one part in `follow_years` of the
//!   way to where the work is.
//! - **Pay clears**: a class's pay is where its demand meets its hands,
//!   `(work share / hands share)^(1 / bend)`, scaled so that the hands' mean
//!   pay is the one wage. So the figures are each class's pay over the mean.
//! - **Two kinds beside the four**: the plain (hauling and tending) and the
//!   schooled (clerking and judging), reckoned in the same way as two markets
//!   of their own.
//!
//! Real pay is a class's pay over the mean times the linked real wage
//! (`linked.rs`; the chained one in a run that does not keep it).
//!
//! It keeps no state. At each year's end from 1900 (`close_year`) it plays the
//! reckoning afresh from the years' record of the tasks and writes every year's
//! figures since 1900, so a dial moved in a running island reads the whole
//! century at the new dial. Nothing reads its series but the record, the facts
//! and the viewer: a run with it on is the same run as one with it off.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::facts::Kind;
use crate::observer::Series;
use crate::params::Params;
use crate::sim::RunResult;
use crate::state::State;
use crate::tasks::{TasksYear, CLERK, HAUL, JUDGE, PRESENCE, TEND};

/// The first year reckoned: hands start where the work is.
pub const FROM: i64 = 1900;

/// The classes reckoned, in the order every table here keeps them.
pub const CLASSES: [&str; 4] = ["haul", "tend", "clerk", "judge"];

/// The two kinds, plain (hauling and tending) and schooled (clerking and
/// judging), as groups of the classes.
pub const KINDS: [&str; 2] = ["plain", "schooled"];
const KIND_OF: [&[usize]; 2] = [&[0, 1], &[2, 3]];

/// The least bend and the fewest years the reckoning takes: below them a pay
/// runs out of the numbers a computer holds.
const LEAST_BEND: f64 = 0.05;
const FEWEST_YEARS: f64 = 1.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PayParams {
    /// Reckon each class's pay from 1900 and keep it as series and facts. It
    /// never touches the economy.
    pub watch: bool,
    /// How far demand for a class bends to its pay: the island hires its
    /// one-wage hands of a class times pay to the power of minus this. The
    /// island's own machines bend it about 0.3 at their margin (docs/rules/pay.md).
    pub bend: f64,
    /// Years hands take to follow the work: each year a class's hands move one
    /// part in this many of the way to where the work is.
    pub follow_years: f64,
}

impl Default for PayParams {
    /// Off: what the port runs with.
    fn default() -> PayParams {
        PayParams { watch: false, bend: 0.3, follow_years: 10.0 }
    }
}

impl PayParams {
    /// The game's watcher.
    pub fn game() -> PayParams {
        PayParams { watch: true, ..PayParams::default() }
    }
}

/// A year's hands at the four classes from the record of the tasks: the works',
/// the land's and the carters' (by the carters' own shares of the classes,
/// `data/tasks.toml`), with presence and every minder counted as tending.
pub fn hands_of(t: &TasksYear, C: &Content) -> [f64; 4] {
    let cart = C.tasks.carting;
    let at = |k: usize| t.hands[k] + t.land_hands[k] + t.carters * cart[k];
    [at(HAUL), at(TEND) + at(PRESENCE) + t.minders + t.land_minders, at(CLERK), at(JUDGE)]
}

/// One year of the reckoning: each class's pay over the mean, and each kind's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Reckoned {
    pub pay: [f64; 4],
    pub kinds: [f64; 2],
}

impl Reckoned {
    /// The schooled's pay over the plain's.
    pub fn schooled_over_plain(&self) -> f64 {
        self.kinds[1] / self.kinds[0]
    }
}

/// Each group's pay over the mean, year by year, for hands `hands` (a year a
/// row, the four classes in it) and the classes grouped as `groups`. The first
/// year's hands are where its work is; a year with no hands at all gives no
/// figure and moves nobody.
fn play(hands: &[[f64; 4]], groups: &[&[usize]], bend: f64, follow_years: f64) -> Vec<Vec<f64>> {
    let (bend, tau) = (bend.max(LEAST_BEND), follow_years.max(FEWEST_YEARS));
    let G = groups.len();
    let mut n: Vec<f64> = Vec::new();
    let mut out = Vec::with_capacity(hands.len());
    for h in hands {
        let all: f64 = h.iter().sum();
        if !(all > 0.0) {
            out.push(vec![f64::NAN; G]);
            continue;
        }
        // the work's shares of the groups: where the hands would be at one wage
        let d: Vec<f64> = groups.iter().map(|g| g.iter().map(|&i| h[i]).sum::<f64>() / all).collect();
        if n.is_empty() {
            n = d.clone();
        }
        let p: Vec<f64> = (0..G).map(|j| (d[j] / n[j].max(1e-12)).powf(1.0 / bend)).collect();
        let mean: f64 = (0..G).map(|j| n[j] * p[j]).sum();
        out.push(p.iter().map(|x| x / mean).collect());
        // hands follow the work: one part in tau of the way a year
        for j in 0..G {
            n[j] += (d[j] - n[j]) / tau;
        }
        let s: f64 = n.iter().sum();
        n.iter_mut().for_each(|x| *x /= s);
    }
    out
}

/// The reckoning (docs/rules/pay.md): for a year a row of hands at the four
/// classes, the first year's where its work is, each class's pay over the mean
/// and each kind's, year by year, at the dials `bend` and `follow_years`.
pub fn reckon(hands: &[[f64; 4]], bend: f64, follow_years: f64) -> Vec<Reckoned> {
    let four = play(hands, &[&[0], &[1], &[2], &[3]], bend, follow_years);
    let two = play(hands, &KIND_OF, bend, follow_years);
    four.iter().zip(&two).map(|(p, k)| Reckoned { pay: [p[0], p[1], p[2], p[3]], kinds: [k[0], k[1]] }).collect()
}

/// The years' hands at the four classes from 1900 to `last`, from the record
/// of the tasks, stopping at the first year it does not hold (a run with the
/// tasks thrown off part way).
pub fn hands_from(st: &State, C: &Content, last: i64) -> Vec<[f64; 4]> {
    let mut out = Vec::new();
    for y in FROM..=last {
        match st.tasks.years.iter().rev().find(|t| t.year == y) {
            Some(t) => out.push(hands_of(t, C)),
            None => break,
        }
    }
    out
}

/// The series' names: the hands at each class, each class's pay over the mean
/// and its real pay, and the schooled over the plain.
fn names(prefix: &str) -> [String; 4] {
    CLASSES.map(|c| format!("{prefix}{c}"))
}

/// The rule's series, after the others so that theirs keep their places. Each
/// holds a year's figure in its twelve months, from 1900; nothing before.
pub fn add_series(reg: &mut Series, P: &Params, n_months: usize) {
    if !P.pay.watch {
        return;
    }
    let words = ["hauling", "tending", "clerking", "judging"];
    for (name, w) in names("pay_hands_").iter().zip(words) {
        reg.add(name, &format!("Hands {w}, works, land and carters"), "pay", "hands", n_months);
    }
    for (name, w) in names("pay_").iter().zip(words) {
        reg.add(name, &format!("Pay for {w} over the mean, reckoned"), "pay", "ratio", n_months);
    }
    for (name, w) in names("pay_real_").iter().zip(words) {
        reg.add(name, &format!("Real pay for {w}, reckoned"), "pay", "1750 = 1", n_months);
    }
    reg.add("pay_schooled_over_plain", "Schooled pay over plain, reckoned", "pay", "ratio", n_months);
}

/// The wage real pay is reckoned on, year by year from `y0`: the linked real
/// wage, or the chained one in a run that does not keep it (NaN with neither).
fn wage(st: &State, y0: i64, years: std::ops::RangeInclusive<i64>) -> Vec<f64> {
    let name = ["real_wage_linked", "real_wage_chained"].into_iter().find(|n| st.series.index.contains_key(*n));
    years.map(|y| name.map_or(f64::NAN, |n| crate::observer::annual(st.series.get(n), y0, y))).collect()
}

/// At a year's end (`sim::end_year`, after the tasks close the year): the
/// reckoning played afresh from 1900 at the rules' dials, and every year's
/// figures since written into its twelve months.
pub fn close_year(st: &mut State, C: &Content, P: &Params, year: i64) {
    if !P.pay.watch || year < FROM || !st.series.index.contains_key("pay_schooled_over_plain") {
        return;
    }
    let hands = hands_from(st, C, year);
    let rows = reckon(&hands, P.pay.bend, P.pay.follow_years);
    let y0 = C.start_year;
    let w = wage(st, y0, FROM..=FROM + rows.len() as i64 - 1);
    let n_months = st.series.data.first().map_or(0, |d| d.len());
    let (h_names, p_names, r_names) = (names("pay_hands_"), names("pay_"), names("pay_real_"));
    for (i, (r, h)) in rows.iter().zip(&hands).enumerate() {
        let first = ((FROM - y0) as usize + i) * 12;
        for t in first..(first + 12).min(n_months) {
            for k in 0..4 {
                st.series.set(&h_names[k], t, h[k]);
                st.series.set(&p_names[k], t, r.pay[k]);
                st.series.set(&r_names[k], t, r.pay[k] * w[i]);
            }
            st.series.set("pay_schooled_over_plain", t, r.schooled_over_plain());
        }
    }
}

/// The year's figures of the rule's series, by their places, for a live game's
/// year (`live::LiveYear::year_series`); none before 1900 or while it is off.
pub fn year_series(st: &State, C: &Content, year: i64) -> Vec<(usize, f64)> {
    let Some(&last) = st.series.index.get("pay_schooled_over_plain") else { return Vec::new() };
    if year < FROM {
        return Vec::new();
    }
    let t = ((year - C.start_year) * 12) as usize;
    let first = st.series.index["pay_hands_haul"];
    (first..=last).filter_map(|i| st.series.data[i].get(t).map(|&v| (i, v))).collect()
}

/// The years the facts are read at.
pub const MARKS: [i64; 4] = [1950, 1970, 2000, 2050];

/// The facts (docs/rules/pay.md), each the mean of the nine years round its
/// year (`linked::nine_years`; the last nine for the run's last year), at the
/// run's dials: `pay_CLASS_Y`, each class's pay over the mean, and
/// `pay_schooled_over_plain_Y`, at `MARKS` inside the run; and with 1970 and
/// 2050 both inside it, `pay_real_CLASS_2050_vs_1970`, each class's real pay
/// in 2050 over 1970's, and `pay_real_mean_2050_vs_1970`, the wage's. Nothing
/// in a run that did not keep them.
pub fn facts(r: &RunResult) -> Vec<(String, Kind, f64)> {
    let (st, C, P) = (&r.state, &r.content, &r.params);
    let mut out = Vec::new();
    if !st.series.index.contains_key("pay_schooled_over_plain") {
        return out;
    }
    let y0 = C.start_year;
    let y1 = y0 + r.rec.n_years as i64 - 1;
    let hands = hands_from(st, C, y1);
    let rows = reckon(&hands, P.pay.bend, P.pay.follow_years);
    if rows.is_empty() {
        return out;
    }
    let last = FROM + rows.len() as i64 - 1;
    let w = wage(st, y0, FROM..=last);
    // the mean over the nine years round y of f(row, wage), within the years reckoned
    let mean = |y: i64, f: &dyn Fn(&Reckoned, f64) -> f64| -> f64 {
        let (a, z) = crate::linked::nine_years(FROM, last, y);
        let xs: Vec<f64> = (a..=z).map(|y| f(&rows[(y - FROM) as usize], w[(y - FROM) as usize])).filter(|x| x.is_finite()).collect();
        if xs.is_empty() { f64::NAN } else { xs.iter().sum::<f64>() / xs.len() as f64 }
    };
    let marks: Vec<i64> = MARKS.iter().copied().filter(|&y| y > FROM && y <= last).collect();
    for k in 0..4 {
        for &y in &marks {
            out.push((format!("pay_{}_{y}", CLASSES[k]), Kind::Ratio, mean(y, &|r, _| r.pay[k])));
        }
    }
    for &y in &marks {
        out.push((format!("pay_schooled_over_plain_{y}"), Kind::Ratio, mean(y, &|r, _| r.schooled_over_plain())));
    }
    if marks.contains(&1970) && marks.contains(&2050) {
        for k in 0..4 {
            out.push((format!("pay_real_{}_2050_vs_1970", CLASSES[k]), Kind::Ratio, mean(2050, &|r, w| r.pay[k] * w) / mean(1970, &|r, w| r.pay[k] * w)));
        }
        out.push(("pay_real_mean_2050_vs_1970".to_string(), Kind::Ratio, mean(2050, &|_, w| w) / mean(1970, &|_, w| w)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hands_where_the_work_is_are_paid_alike() {
        // the work's shares never move: every class is paid the mean in every year
        let h = vec![[30.0, 40.0, 10.0, 20.0]; 5];
        for r in reckon(&h, 0.3, 10.0) {
            for p in r.pay {
                assert!((p - 1.0).abs() < 1e-12, "{r:?}");
            }
            assert!((r.schooled_over_plain() - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn a_class_whose_work_shrinks_is_paid_less_and_its_hands_follow() {
        // two years: in the second, hauling's work halves and clerking's doubles
        let h = vec![[40.0, 40.0, 10.0, 10.0], [20.0, 40.0, 20.0, 10.0]];
        let (bend, tau) = (0.5, 4.0);
        let rows = reckon(&h, bend, tau);
        assert_eq!(rows[0].pay, [1.0; 4]);
        // the hands of 1900 are the work's of 1900, moved a quarter of the way to its
        // shares of the first year (they are the same), so in the second year they are
        // still 0.4, 0.4, 0.1, 0.1; the work is 2/9, 4/9, 2/9, 1/9
        let n: [f64; 4] = [0.4, 0.4, 0.1, 0.1];
        let d: [f64; 4] = [2.0 / 9.0, 4.0 / 9.0, 2.0 / 9.0, 1.0 / 9.0];
        let p: Vec<f64> = (0..4).map(|k| (d[k] / n[k]).powf(1.0 / bend)).collect();
        let m: f64 = (0..4).map(|k| n[k] * p[k]).sum();
        for k in 0..4 {
            assert!((rows[1].pay[k] - p[k] / m).abs() < 1e-12, "{k}: {:?}", rows[1]);
        }
        assert!(rows[1].pay[0] < 1.0 && rows[1].pay[2] > 1.0);
        // the hands' mean pay is the one wage
        let mean: f64 = (0..4).map(|k| n[k] * rows[1].pay[k]).sum();
        assert!((mean - 1.0).abs() < 1e-12);
        // a third year at the second's work: the hands have moved a quarter of the way, and
        // the gaps narrow
        let h3 = vec![h[0], h[1], h[1]];
        let rows3 = reckon(&h3, bend, tau);
        assert!(rows3[2].pay[0] > rows3[1].pay[0] && rows3[2].pay[0] < 1.0);
        assert!(rows3[2].pay[2] < rows3[1].pay[2] && rows3[2].pay[2] > 1.0);
        // the more demand bends, the less pay moves
        let loose = reckon(&h, 2.0, tau);
        assert!(loose[1].pay[0] > rows[1].pay[0] && loose[1].pay[2] < rows[1].pay[2]);
        // the schooled over the plain: clerks' and judges' work grew from 0.2 to 1/3
        assert!(rows[1].schooled_over_plain() > 1.0);
    }

    #[test]
    fn a_year_with_no_hands_gives_no_figure_and_dials_past_their_ends_are_held() {
        let h = vec![[1.0, 1.0, 1.0, 1.0], [0.0; 4], [2.0, 1.0, 1.0, 1.0]];
        let rows = reckon(&h, 0.0, 0.0);
        assert!(rows[1].pay.iter().all(|p| p.is_nan()));
        assert!(rows[2].pay.iter().all(|p| p.is_finite()), "{:?}", rows[2]);
    }
}
