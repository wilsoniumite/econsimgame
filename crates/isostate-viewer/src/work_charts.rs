//! The Work half of the Markets panel: the run to 2050's island, year by year
//! (docs/rules/gauges.md, tasks.md and owners.md). What a wage buys, work's
//! share three ways, the part of each class of task machines do, the income
//! paid to owners, who owns the works, and what a wage buys of each kind of
//! good.
//!
//! Most of it is read from the core's series, which every game and every run
//! recorded under the game's rules holds. Two things are not: the part of each
//! class machines did (`st.tasks.years`) and who holds the works
//! (`st.owners.years`). They are not made into the core's series, since a new
//! series moves the game's golden hashes; the core hands the year's figures
//! over in `LiveYear::tasks` and `LiveYear::owners`, and `take_year` writes them
//! into the replay's series table as rows of the viewer's own, named
//! `viewer_...` so that nobody takes them for the core's. A recorded run has
//! none of them, and the panel says so.
//!
//! `charts` builds the vectors once; the panel draws them and `said` prints
//! them (a script's `print`), so what is printed is what is drawn.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use isostate_core::live::LiveYear;
use isostate_core::owners::OwnersYear;
use isostate_core::tasks::TasksYear;

use crate::chart::{self, Lines, INK_U8};
use crate::replay::{self, Replay};
use crate::ui::{text, CHEAPER, DEARER, GOOD, INK, MUTED};

/// The viewer's rows of the part of each class of task done by machine, works
/// and land together: hauling, tending, clerking and judging (`tasks::CLASSES`
/// in order). Presence has none: no machine may take it.
pub const MACHINE_ROWS: [&str; 4] = ["viewer_machine_haul", "viewer_machine_tend", "viewer_machine_clerk", "viewer_machine_judge"];
/// The viewer's rows of each holder's part of the works' book: the families,
/// the landlords and the households (`OwnersYear::book` in order).
pub const OWNED_ROWS: [&str; 3] = ["viewer_owned_families", "viewer_owned_landlords", "viewer_owned_households"];

/// The core's series of the chained wage (engel.rs).
const CHAINED: &str = "real_wage_chained";
/// The core's series of the linked wage and of what is made, linked (linked.rs,
/// docs/rules/measures.md), which a run under the game's rules keeps since
/// 20 November 2026; the wage is read before the chained one where it is kept.
const LINKED: &str = "real_wage_linked";
const MADE: &str = "made_linked";

/// Room for a chart inside the side panel, as the island's charts have (ui.rs).
const CHART_W: f32 = 342.0;
const CHART_H: f32 = 88.0;

/// The part of class `k`'s work machines did in the year, in the works and on
/// the land together (both are worker-months a month, the year's mean). NaN
/// where the class had no work.
pub fn machine_share(t: &TasksYear, k: usize) -> f64 {
    let held = t.held[k] + t.land_held.iter().map(|s| s[k]).sum::<f64>();
    let work = t.work[k] + t.land_work.iter().map(|s| s[k]).sum::<f64>();
    if work.abs() > 1e-12 { held / work } else { f64::NAN }
}

/// Holder `i`'s part of the works' book at the year's end (NaN with no book).
pub fn owned_share(o: &OwnersYear, i: usize) -> f64 {
    let all: f64 = o.book.iter().sum();
    if all.abs() > 1e-12 { o.book[i] / all } else { f64::NAN }
}

/// Take in a year the core has sent: the viewer's own rows, each written into
/// the year's twelve months as the books' figures are (`live::apply_year`).
pub fn take_year(r: &mut Replay, y: &LiveYear) {
    take(r, y.index, y.tasks.as_ref(), y.owners.as_ref());
}

/// `take_year` on its parts: the year's place in the run and its two records.
fn take(r: &mut Replay, k: usize, tasks: Option<&TasksYear>, owners: Option<&OwnersYear>) {
    if k >= r.ny {
        return;
    }
    if let Some(t) = tasks {
        for (j, name) in MACHINE_ROWS.iter().enumerate() {
            write(r, name, k, machine_share(t, j));
        }
    }
    if let Some(o) = owners {
        for (i, name) in OWNED_ROWS.iter().enumerate() {
            write(r, name, k, owned_share(o, i));
        }
    }
}

/// Write a year's figure into the twelve months of row `name`, adding the row
/// at the end of the table the first time it comes (so the core's places in
/// it, which `apply_series` and the year's series go by, stay as they are).
fn write(r: &mut Replay, name: &str, k: usize, v: f64) {
    let i = match r.series_names.iter().position(|s| s == name) {
        Some(i) => i,
        None => {
            r.series_names.push(name.to_string());
            r.series.resize(r.series_names.len() * r.series_len, f32::NAN);
            r.series_names.len() - 1
        }
    };
    let row = i * r.series_len;
    for t in (k * 12..(k + 1) * 12).filter(|&t| t < r.series_len) {
        r.series[row + t] = v as f32;
    }
}

/// A series year by year over its own first year's figure: an index with the
/// first year at 1. NaN throughout when the first year has no figure.
pub fn over_first(vals: &[f32]) -> Vec<f32> {
    let base = vals.first().copied().filter(|b| b.is_finite() && *b > 0.0).unwrap_or(f32::NAN);
    vals.iter().map(|v| v / base).collect()
}

/// The wage the charts read: the linked one where the replay keeps it, else
/// the chained one; None for a run with neither (the port's rules).
pub fn wage_row(r: &Replay) -> Option<&'static str> {
    [LINKED, CHAINED].into_iter().find(|name| r.series_names.iter().any(|s| s == name))
}

/// What a wage buys against 1750: the linked wage, or the chained one in a run
/// without it (`wage_row`; the year's mean of its months), over its own figure
/// for the first year, year by year, or with `months` (the first month and how
/// many) month by month over the same figure. None for a run with neither (the
/// port's rules), whose charts keep the fixed basket of 1750.
pub fn true_wage(r: &Replay, months: Option<(usize, usize)>) -> Option<Vec<f32>> {
    let row = wage_row(r)?;
    let years = replay::yearly(r, row);
    let base = years.first().copied().filter(|b| b.is_finite() && *b > 0.0)?;
    Some(match months {
        Some((first, n)) => replay::monthly(r, row, first, n).iter().map(|v| v / base).collect(),
        None => over_first(&years),
    })
}

/// What is made a head against 1750: what is made, linked, over the people,
/// over its own figure for the first year, year by year or with `months` month
/// by month, as `true_wage`. None for a run that does not keep it.
pub fn made_a_head(r: &Replay, months: Option<(usize, usize)>) -> Option<Vec<f32>> {
    if !r.series_names.iter().any(|s| s == MADE) {
        return None;
    }
    let per = |m: Vec<f32>, p: Vec<f32>| -> Vec<f32> { m.iter().zip(&p).map(|(m, p)| m / p).collect() };
    let years = per(replay::yearly(r, MADE), replay::yearly(r, "pop"));
    let base = years.first().copied().filter(|b| b.is_finite() && *b > 0.0)?;
    Some(match months {
        Some((first, n)) => per(replay::monthly(r, MADE, first, n), replay::monthly(r, "pop", first, n)).iter().map(|v| v / base).collect(),
        None => over_first(&years),
    })
}

/// How a chart's figures are written.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Unit {
    /// a share, as per cent to one place
    Share,
    /// an index, to two places
    Ratio,
}

impl Unit {
    fn words(self, v: f32) -> String {
        match self {
            Unit::Share => format!("{:.1}%", v * 100.0),
            Unit::Ratio => format!("{v:.2}"),
        }
    }
}

/// One line of a chart: its words in the key, its word in `said`, its colour
/// and its figures year by year.
pub struct Line {
    pub words: &'static str,
    pub short: &'static str,
    pub colour: [u8; 3],
    pub vals: Vec<f32>,
    /// drawn on the chart (a class no machine has held so far is not)
    pub drawn: bool,
}

/// One chart of the Work half.
pub struct Plan {
    pub title: &'static str,
    /// its tag in `said`
    pub tag: &'static str,
    pub lines: Vec<Line>,
    unit: Unit,
    from_zero: bool,
    /// muted lines under the chart
    pub notes: Vec<String>,
}

impl Plan {
    /// Whether the chart is shown at all: some line has a figure somewhere.
    pub fn shown(&self) -> bool {
        self.lines.iter().any(|l| l.vals.iter().any(|v| v.is_finite()))
    }
}

/// The colours of a chart's lines, in order: the chart's ink, then the warm,
/// the cool and the green of the interface. Never the accent, which is the
/// hand's ink in a sandbox and a family's colour in the family game.
fn colours() -> [[u8; 3]; 4] {
    let u8s = |c: Color| {
        let a = c.to_srgba().to_u8_array();
        [a[0], a[1], a[2]]
    };
    [INK_U8, u8s(DEARER), u8s(CHEAPER), u8s(GOOD)]
}

/// The words of a class of task done by machine, in `tasks::CLASSES` order.
const CLASS_WORDS: [&str; 4] = ["hauling", "tending", "clerking", "judging"];

/// "tending, clerking or judging".
fn or_list(words: &[&str]) -> String {
    match words {
        [] => String::new(),
        [one] => one.to_string(),
        [rest @ .., last] => format!("{} or {last}", rest.join(", ")),
    }
}

/// The six charts as the replay holds them, at the year `k` on the clock
/// (which decides which classes machines have held so far).
pub fn charts(r: &Replay, k: usize) -> Vec<Plan> {
    let c = colours();
    let line = |words, short, colour, vals: Vec<f32>| Line { words, short, colour, vals, drawn: true };
    let y = |name: &str| replay::yearly(r, name);
    let mut out = Vec::new();
    out.push(Plan {
        title: "What a wage buys (1750 = 1)",
        tag: "wage",
        lines: vec![line(if wage_row(r) == Some(LINKED) { "the linked wage" } else { "the chained wage" }, "", c[0], true_wage(r, None).unwrap_or_default())],
        unit: Unit::Ratio,
        from_zero: false,
        notes: Vec::new(),
    });
    out.push(Plan {
        title: "Wages, as a share",
        tag: "wages",
        lines: vec![
            line("of income", "of income", c[0], y("acct_share_work")),
            line("of income less plant worn out", "less wear", c[1], y("gauge_work_net")),
            line("of spending", "of spending", c[2], y("gauge_spend_work")),
        ],
        unit: Unit::Share,
        from_zero: false,
        notes: Vec::new(),
    });
    // a class gets a line once some machine has held some of it, in any year up to the clock's
    let mut machines: Vec<Line> = MACHINE_ROWS.iter().zip(CLASS_WORDS).enumerate().map(|(j, (name, words))| line(words, words, c[j], y(name))).collect();
    for l in &mut machines {
        l.drawn = l.vals.iter().take(k + 1).any(|v| v.is_finite() && *v > 0.0);
    }
    let kept = machines.iter().any(|l| !l.vals.is_empty());
    let without: Vec<&str> = machines.iter().filter(|l| !l.drawn).map(|l| l.words).collect();
    let mut notes = Vec::new();
    if kept && !without.is_empty() {
        notes.push(format!("No machine yet for {}.", or_list(&without)));
    }
    if kept {
        notes.push("Work only a person may do is never a machine's.".to_string());
    }
    out.push(Plan { title: "Work done by machine", tag: "by machine", lines: machines, unit: Unit::Share, from_zero: true, notes });
    out.push(Plan {
        title: "Income paid to owners",
        tag: "to owners",
        lines: vec![
            line("works' owners", "works", c[0], y("acct_share_works")),
            line("landowners", "land", c[1], y("acct_share_land")),
            line("merchants", "merchants", c[2], y("acct_share_merchants")),
        ],
        unit: Unit::Share,
        from_zero: true,
        notes: Vec::new(),
    });
    out.push(Plan {
        title: "Who owns the works, by their worth",
        tag: "works held",
        lines: OWNED_ROWS.iter().zip([("the families", "families"), ("landlords", "landlords"), ("households", "households")]).enumerate().map(|(i, (name, (words, short)))| line(words, short, c[i], y(name))).collect(),
        unit: Unit::Share,
        from_zero: true,
        notes: Vec::new(),
    });
    out.push(Plan {
        title: "What a wage buys of each kind (1750 = 1)",
        tag: "a wage buys",
        lines: vec![
            line("goods made in works", "made", c[0], y("gauge_fork_made")),
            line("the land's goods", "the land's", c[1], y("gauge_fork_raw")),
            line("both", "both", c[2], y("gauge_fork_all")),
        ],
        unit: Unit::Ratio,
        from_zero: false,
        notes: vec!["Reckoned on what households bought each year, so it reads a little apart from the wage at the top.".to_string()],
    });
    out
}

/// Whether the replay keeps the viewer's rows: what machines do and who owns
/// the works (a game under the game's rules does; a recorded run does not).
pub fn kept(r: &Replay) -> bool {
    MACHINE_ROWS.iter().chain(OWNED_ROWS.iter()).any(|name| r.series_names.iter().any(|s| s == name))
}

/// What a script's `print` gives of the Work half: one line for each
/// fifty-year mark the replay holds and one for the year on the clock, each
/// figure to four places as the charts hold it, "none" where there is none,
/// and every class of task whether drawn or not.
pub fn said(r: &Replay, k: usize) -> Vec<String> {
    let plans = charts(r, k);
    let filled = r.filled.min(r.ny);
    let mut years: Vec<usize> = (0..filled).filter(|&i| (r.y0 + i as i32) % 50 == 0).collect();
    if k < filled && !years.contains(&k) {
        years.push(k);
    }
    years
        .iter()
        .map(|&i| {
            let parts: Vec<String> = plans
                .iter()
                .map(|p| {
                    let figs: Vec<String> = p
                        .lines
                        .iter()
                        .map(|l| {
                            let v = l.vals.get(i).copied().filter(|v| v.is_finite()).map_or("none".to_string(), |v| format!("{v:.4}"));
                            if l.short.is_empty() { v } else { format!("{} {v}", l.short) }
                        })
                        .collect();
                    if p.lines.len() == 1 && p.lines[0].short.is_empty() { format!("{} {}", p.tag, figs[0]) } else { format!("{}: {}", p.tag, figs.join(", ")) }
                })
                .collect();
            format!("work: {} | {}", r.y0 + i as i32, parts.join(" | "))
        })
        .collect()
}

/// A small round dot of a line's colour, for the key under a chart's title.
fn key_dot(p: &mut ChildSpawnerCommands, c: Color) {
    p.spawn((Node { width: Val::Px(8.0), height: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(4.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(c)));
}

/// The Work half of the Markets panel, at the year `k` on the clock. It goes by
/// the year only: its figures are whole years'.
pub fn panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, images: &mut Assets<Image>, r: &Replay, k: usize) {
    let year = r.y0 + k as i32;
    p.spawn(text(f, &format!("The island's work and its owners, year by year. The marker and the figures are {year}."), 13.0, MUTED));
    for plan in charts(r, k).iter().filter(|p| p.shown()) {
        p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), margin: UiRect::top(Val::Px(10.0)), ..default() }).with_children(|col| {
            col.spawn(text(f, plan.title, 15.0, INK));
            let drawn: Vec<&Line> = plan.lines.iter().filter(|l| l.drawn).collect();
            col.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(12.0), row_gap: Val::Px(2.0), padding: UiRect::left(Val::Px(4.0)), ..default() }).with_children(|keys| {
                for l in &drawn {
                    let colour = Color::srgb_u8(l.colour[0], l.colour[1], l.colour[2]);
                    let fig = l.vals.get(k).copied().filter(|v| v.is_finite()).map_or("none".to_string(), |v| plan.unit.words(v));
                    keys.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(5.0), ..default() }).with_children(|key| {
                        key_dot(key, colour);
                        key.spawn(text(f, &format!("{} {fig}", l.words), 13.0, colour));
                    });
                }
            });
            if !drawn.is_empty() {
                let lines: Vec<(&[f32], [u8; 3])> = drawn.iter().map(|l| (&l.vals[..], l.colour)).collect();
                let years = drawn.iter().map(|l| l.vals.len()).max().unwrap_or(0);
                let c = Lines { width: CHART_W, height: CHART_H, lines: &lines, mark: k, years, first_year: r.y0, from_zero: plan.from_zero };
                let unit = plan.unit;
                chart::spawn_lines(col, f, images, &c, &|v| unit.words(v));
            }
            for note in &plan.notes {
                col.spawn(text(f, note, 12.5, MUTED));
            }
        });
    }
    if !kept(r) {
        p.spawn((text(f, "This run did not keep what machines do or who owns the works.", 13.0, MUTED), Node { margin: UiRect::top(Val::Px(10.0)), ..default() }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run of `years` years from 1750 with the named rows, each a year's
    /// figure in all twelve of its months, made by `fig(row, year)`.
    fn run(years: usize, names: &[&str], fig: impl Fn(&str, usize) -> f32) -> Replay {
        let mut r = replay::tiny(1750);
        r.ny = years;
        r.filled = years;
        r.series_len = years * 12;
        r.series_names = names.iter().map(|s| s.to_string()).collect();
        r.series = names.iter().flat_map(|n| (0..years * 12).map(|t| fig(n, t / 12)).collect::<Vec<f32>>()).collect();
        r
    }

    #[test]
    fn a_year_goes_into_its_twelve_months_and_a_second_year_adds_no_row() {
        let mut r = run(3, &["pop"], |_, _| 1.0);
        let mut t = TasksYear { work: [10.0, 4.0, 0.0, 2.0, 5.0], held: [1.0, 0.0, 0.0, 0.0, 0.0], ..Default::default() };
        // the land's hauling: 6 of work, 3 by machine, in one sector
        t.land_work = vec![[6.0, 0.0, 0.0, 0.0, 0.0]];
        t.land_held = vec![[3.0, 0.0, 0.0, 0.0, 0.0]];
        let o = OwnersYear { book: [2.0, 1.0, 1.0], ..Default::default() };
        take(&mut r, 1, Some(&t), Some(&o));
        assert_eq!(r.series_names.len(), 1 + 4 + 3);
        assert_eq!(r.series.len(), r.series_names.len() * r.series_len);
        // the core's row is untouched, and is still first
        assert_eq!(r.series_names[0], "pop");
        assert!(r.series[..36].iter().all(|&v| v == 1.0));
        let haul = replay::monthly(&r, "viewer_machine_haul", 0, 36);
        assert!(haul[..12].iter().chain(&haul[24..]).all(|v| v.is_nan()), "only the year taken in has a figure");
        assert!(haul[12..24].iter().all(|&v| v == ((1.0 + 3.0) / (10.0 + 6.0)) as f32));
        assert_eq!(replay::annual(&r, "viewer_machine_tend", 1), Some(0.0));
        assert_eq!(replay::annual(&r, "viewer_machine_clerk", 1), None, "a class with no work has no figure");
        assert_eq!(replay::annual(&r, "viewer_owned_families", 1), Some(0.5));
        // a second year adds no row, and writes its own months only
        take(&mut r, 2, Some(&t), None);
        assert_eq!(r.series_names.len(), 8);
        assert_eq!(replay::annual(&r, "viewer_machine_haul", 2), Some(0.25));
        assert_eq!(replay::annual(&r, "viewer_owned_families", 2), None);
        // a year past the run's end is left alone
        take(&mut r, 3, Some(&t), Some(&o));
        assert_eq!(r.series.len(), 8 * 36);
    }

    /// Two hundred and one years, 1750 to 1950, of every series the charts read,
    /// with hauling by machine from 1903 and tending never.
    fn island() -> Replay {
        let names = [
            "real_wage_chained",
            "acct_share_work",
            "gauge_work_net",
            "gauge_spend_work",
            "acct_share_works",
            "acct_share_land",
            "acct_share_merchants",
            "gauge_fork_made",
            "gauge_fork_raw",
            "gauge_fork_all",
            "viewer_machine_haul",
            "viewer_machine_tend",
            "viewer_machine_clerk",
            "viewer_machine_judge",
            "viewer_owned_families",
            "viewer_owned_landlords",
            "viewer_owned_households",
        ];
        run(201, &names, |n, y| match n {
            "real_wage_chained" => 2.0 + y as f32 * 0.04,
            "viewer_machine_haul" => if y >= 153 { 0.25 } else { 0.0 },
            "viewer_machine_tend" | "viewer_machine_clerk" | "viewer_machine_judge" => 0.0,
            "viewer_owned_families" => 0.5,
            "viewer_owned_landlords" | "viewer_owned_households" => 0.25,
            _ => 0.75,
        })
    }

    #[test]
    fn said_gives_the_fifty_year_marks_and_the_clock_from_what_is_drawn() {
        let r = island();
        let lines = said(&r, 160);
        let years: Vec<&str> = lines.iter().map(|l| &l[6..10]).collect();
        assert_eq!(years, ["1750", "1800", "1850", "1900", "1950", "1910"]);
        // the wage over its own 1750: (2 + 200 * 0.04) / 2 = 5 in 1950
        assert!(lines[4].starts_with("work: 1950 | wage 5.0000 | wages: of income 0.7500, less wear 0.7500, of spending 0.7500 | by machine: hauling 0.2500, tending 0.0000, clerking 0.0000, judging 0.0000 | to owners: works 0.7500, land 0.7500, merchants 0.7500 | works held: families 0.5000, landlords 0.2500, households 0.2500 | a wage buys: made 0.7500, the land's 0.7500, both 0.7500"), "{}", lines[4]);
        assert!(lines[3].contains("by machine: hauling 0.0000,"), "{}", lines[3]);
        // and the figures are the very vectors the charts draw
        let plans = charts(&r, 160);
        assert_eq!(plans[0].lines[0].vals[200], 5.0);
    }

    #[test]
    fn a_class_no_machine_has_held_is_printed_and_not_drawn() {
        let r = island();
        // in 1910 hauling has been done by machine since 1903; the others never
        let plans = charts(&r, 160);
        let machines = &plans[2];
        assert_eq!(machines.lines.iter().map(|l| l.drawn).collect::<Vec<_>>(), [true, false, false, false]);
        assert_eq!(machines.notes, ["No machine yet for tending, clerking or judging.", "Work only a person may do is never a machine's."]);
        assert!(said(&r, 160)[5].contains("tending 0.0000, clerking 0.0000, judging 0.0000"));
        // in 1900, before the find, no class is drawn, and every one is named
        let before = charts(&r, 150);
        assert!(before[2].lines.iter().all(|l| !l.drawn));
        assert_eq!(before[2].notes[0], "No machine yet for hauling, tending, clerking or judging.");
        assert_eq!(or_list(&["judging"]), "judging");
    }

    #[test]
    fn a_run_without_the_rows_shows_four_charts_and_says_so() {
        let mut r = island();
        let keep = r.series_names.iter().position(|n| n.starts_with("viewer_")).unwrap();
        r.series_names.truncate(keep);
        r.series.truncate(keep * r.series_len);
        assert!(!kept(&r) && kept(&island()));
        let plans = charts(&r, 200);
        let shown: Vec<&str> = plans.iter().filter(|p| p.shown()).map(|p| p.title).collect();
        assert_eq!(shown, ["What a wage buys (1750 = 1)", "Wages, as a share", "Income paid to owners", "What a wage buys of each kind (1750 = 1)"]);
        // the machines' chart says nothing of classes it does not know
        assert!(plans[2].notes.is_empty());
        // and said gives every class as none
        assert!(said(&r, 200)[4].contains("by machine: hauling none, tending none, clerking none, judging none"));
        // a run without the chained wage has no true wage to give
        let names: Vec<String> = r.series_names.iter().filter(|n| *n != "real_wage_chained").cloned().collect();
        let r2 = run(3, &names.iter().map(|s| s.as_str()).collect::<Vec<_>>(), |_, _| 1.0);
        assert_eq!(true_wage(&r2, None), None);
        assert_eq!(over_first(&[2.0, 3.0, f32::NAN]).iter().take(2).copied().collect::<Vec<_>>(), [1.0, 1.5]);
    }

    #[test]
    fn the_linked_wage_is_read_before_the_chained_and_what_is_made_is_a_head() {
        let r = run(3, &["pop", "real_wage_chained", "real_wage_linked", "made_linked"], |n, y| match n {
            "pop" => 10.0 * (1 + y) as f32,
            "real_wage_chained" => 1.0 + y as f32,
            "real_wage_linked" => 2.0 + y as f32,
            _ => 40.0 * (1 + y) as f32 * (1 + y) as f32,
        });
        assert_eq!(wage_row(&r), Some("real_wage_linked"));
        assert_eq!(true_wage(&r, None).unwrap(), [1.0, 1.5, 2.0]);
        assert_eq!(charts(&r, 2)[0].lines[0].words, "the linked wage");
        // made over people: 4, 8, 12, over the first year's
        assert_eq!(made_a_head(&r, None).unwrap(), [1.0, 2.0, 3.0]);
        assert_eq!(made_a_head(&r, Some((12, 12))).unwrap(), [2.0; 12]);
        // a run with the chained wage alone reads it, and has nothing made to show
        let old = run(3, &["pop", "real_wage_chained"], |_, y| 1.0 + y as f32);
        assert_eq!(wage_row(&old), Some("real_wage_chained"));
        assert_eq!(charts(&old, 2)[0].lines[0].words, "the chained wage");
        assert_eq!(made_a_head(&old, None), None);
    }
}
