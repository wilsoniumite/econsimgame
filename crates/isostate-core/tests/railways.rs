//! Railways under the game's rules (not in the Python; docs/rules/railways.md):
//! the promoters of `ways.rs` build branches where the port's stop, no cell of
//! line is paid for twice, the news stays readable, money is conserved and a
//! run is deterministic. With the switch off the golden test (tests/golden.rs)
//! covers that nothing changed.
#![allow(non_snake_case)]

use std::sync::OnceLock;

use isostate_core::ways::{SERVES_TOWN, SERVES_TRUNK};
use isostate_core::*;

/// The year the runs here stop: far enough into the railway age for the rush to
/// be over and the branches to have begun.
const END: i64 = 1865;

fn params(on: bool) -> Params {
    let mut P = default_params();
    P.seed = 3;
    P.end_year = Some(END);
    P.railways.enabled = on;
    P
}

fn run(on: bool) -> RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(&params(on), default_content(), false, true).expect("money is conserved every month")
}

/// One run with the railways on, shared by the tests below.
fn railways() -> &'static RunResult {
    static RUN: OnceLock<RunResult> = OnceLock::new();
    RUN.get_or_init(|| run(true))
}

fn km(st: &State) -> f64 {
    2.0 * st.rail.iter().filter(|&&r| r > 0).count() as f64
}

#[test]
fn railways_conserve_money_and_runs_are_deterministic() {
    let (a, b) = (railways(), run(true));
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    for (k, name) in st.series.names.iter().enumerate() {
        let (x, y) = (&st.series.data[k], &b.state.series.data[k]);
        assert!(x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()), "series {name} differs between identical runs");
    }
    assert!(st.rail == b.state.rail && st.rw == b.state.rw, "the same lines are built both times");
    let titles = |r: &RunResult| r.events.iter().map(|e| e.title.clone()).collect::<Vec<_>>();
    assert_eq!(titles(a), titles(&b));
    // no line ever owes anything: it orders what its cash will pay for
    assert!(st.projects.iter().all(|pr| pr.cash >= 0.0));
}

#[test]
fn branches_carry_the_network_past_the_ports() {
    let (on, off) = (railways(), run(false));
    let (st, book) = (&on.state, &on.state.rw);
    assert!(off.state.rw == ways::WayBook::default(), "the promoters' book stays empty while the switch is off");
    assert!(km(st) > 1.2 * km(&off.state), "{} km with branches against the port's {}", km(st), km(&off.state));

    // trunks, and branches to districts: quarries, works, farms
    let trunks = book.lines.iter().filter(|l| l.serves == SERVES_TRUNK).count();
    let districts: Vec<_> = book.lines.iter().filter(|l| l.serves != SERVES_TRUNK && l.serves != SERVES_TOWN).collect();
    assert!(trunks >= 5 && districts.len() >= 10, "{trunks} trunks and {} district branches", districts.len());
    let kinds: std::collections::BTreeSet<u8> = districts.iter().map(|l| l.serves).collect();
    assert!(kinds.len() >= 3, "the branches serve more than one kind of district: {kinds:?}");
    // every town of any size has track
    let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    let with = act.iter().filter(|&&t| st.rail[st.town_node[t]] > 0).count();
    assert!(with + 1 >= act.len(), "{with} of {} towns have track", act.len());

    // every cell of line was paid for by exactly one line, opened or building
    let paid: usize = book.lines.iter().map(|l| l.new_cells).sum::<usize>() + st.projects.iter().map(|pr| pr.new.len()).sum::<usize>();
    let mut cells: Vec<usize> = (0..st.N).filter(|&c| st.rail[c] > 0).collect();
    cells.extend(st.projects.iter().flat_map(|pr| pr.new.iter().copied()));
    cells.sort_unstable();
    let before = cells.len();
    cells.dedup();
    assert_eq!(before, cells.len(), "two lines building never share a new cell");
    assert_eq!(paid, cells.len(), "cells paid for against cells laid or building");

    // each line cost its subscribers something, and no more than they put in
    for l in &book.lines {
        assert!(l.spent > 0.0 && l.km > 0.0 && l.opened >= l.start as f64, "{l:?}");
    }
    // the year's notes: the network never shrinks, and fares are taken on it
    assert!(book.years.windows(2).all(|w| w[1].km >= w[0].km));
    let last = book.years.last().expect("a note for every year");
    assert!(last.fares > 0.0 && last.capital > 0.0);
}

#[test]
fn a_line_opens_only_onto_track_that_is_there() {
    // year by year through the rush: every line that opened in the year runs
    // over track from end to end, its junction and any stretch it shares with
    // an older line included
    let mut s = sim::start(&params(true), default_content());
    let mut opened = 0;
    while s.year <= 1850 {
        let before = s.st.projects.clone();
        sim::run_year(&mut s).expect("money is conserved every month");
        for pr in before.iter().filter(|pr| !s.st.projects.iter().any(|q| q.new == pr.new)) {
            opened += 1;
            assert!(pr.cells.iter().all(|&c| s.st.rail[c] > 0), "a line begun in {} opened in {} onto a line still building", pr.start, s.year - 1);
        }
    }
    assert!(opened >= 20, "{opened} lines opened by 1850");
}

#[test]
fn the_news_keeps_the_trunks_and_folds_the_branches() {
    let r = railways();
    let book = &r.state.rw;
    let news: Vec<&Event> = r.events.iter().filter(|e| e.kind == "railway").collect();
    let first: Vec<&&Event> = news.iter().filter(|e| e.weight >= 1.0).collect();
    assert_eq!(first.len(), 1, "one first railway");
    assert!(first[0].title.starts_with("The first railway opens"), "{}", first[0].title);
    assert!(news.iter().all(|e| e.year >= first[0].year), "nothing opens before the first railway");
    // a trunk or a town's first line is an item of its own; the district branches
    // of a year share one
    let own = book.lines.iter().filter(|l| l.serves == SERVES_TRUNK || l.serves == SERVES_TOWN).count();
    let folded: Vec<&&Event> = news.iter().filter(|e| e.title.contains("branch line")).collect();
    assert_eq!(news.len(), own + folded.len());
    let mut years: Vec<i64> = folded.iter().map(|e| e.year.floor() as i64).collect();
    let n = years.len();
    years.dedup();
    assert_eq!(n, years.len(), "at most one item about branches a year");
    let branches = book.lines.len() - own;
    assert!(n >= 5 && n < branches, "{branches} branches made {n} items");
    // and none of them is called a mineral line for pits and works
    assert!(r.events.iter().all(|e| !e.title.contains("Mineral line")));
}
