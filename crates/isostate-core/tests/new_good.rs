//! A good added after today's moves nothing until it is made, bought or held
//! (docs/rules/goods.md).
//!
//! Until 26 November 2026 a good that nothing made and nobody wanted moved a run
//! from its first years: the families' typical good was the median over every
//! good, and every sum over the goods was taken in numpy's pairwise order, which
//! rounds the same terms in other groups when the goods pass a multiple of eight.
//! Each test here runs an island twice, once on the content as it is and once
//! with test goods added, and holds the two equal: every fact the sweep takes
//! and every yearly mean of every series, to the last digit, but those of the
//! test goods themselves. Under either set of rules, whatever its switches: the
//! two runs are played under the same rules, so no rule's history is leaned on.
//!
//! The test goods are eight, so that the sums pass a row of eight in the
//! port's content (16 goods to 24) and the game's (29 to 37). One opens at its
//! ceiling, as a good nobody has until it is found does; one does not keep and
//! is not carried, as electricity; the rest are plain.
//!
//! The default run holds one island of each case, about a run to 2050's time.
//! The whole proof, islands 1 to 8 to 2050 under both sets of rules and to 1949
//! with a method of 1950, is one command (some 25 minutes of processor time,
//! six or seven on the clock with cores to spare):
//!
//! ```text
//! cargo test --profile work -p isostate-core --test new_good -- --ignored --nocapture
//! ```
//!
//! `probe` (ignored, run by name) is the tool the leaks were found with: two
//! islands stepped together, the state compared bit for bit each month, which
//! says the month and the field that parts first.

#![allow(non_snake_case)]

use std::collections::BTreeMap;

use isostate_core::content::{Begins, Content, content_from_str, DEFAULT_CONTENT_TOML};
use isostate_core::mat::Mat;
use isostate_core::sim::{self, Sim};
use isostate_core::sweep::{self, Sweep};
use isostate_core::util::{pairwise_sum_by, pairwise_sum_from};
use isostate_core::{game_content, game_params, tree};

/// The test goods, in the form of a line of `goods`; `begins` is said by the caller.
fn good_lines(n: usize, begins: &str) -> String {
    let mut s = String::new();
    for k in 0..n {
        let kind = match k {
            0 => "decay = 0.002, stock0 = 0.0, opens_dear = true, ",
            1 => "stock0 = 0.0, traded = false, stock_months = 0.0, ",
            _ => "decay = 0.002, stock0 = 0.0, ",
        };
        s.push_str(&format!(
            "\n  {{ name = \"test_good{k}\", label = \"Test good {k}\", weight = 1.0, price = {}, color = \"#777777\", {kind}begins = {begins} }},",
            2.0 + k as f64
        ));
    }
    s
}

/// `text` with `lines` put at the end of its list of goods.
fn with_goods(text: &str, lines: &str) -> String {
    let start = text.find("goods = [").expect("a list of goods");
    let close = start + text[start..].find("\n]").expect("the list's end");
    format!("{}{}{}", &text[..close], lines, &text[close..])
}

/// A method that makes test good 0, found from 1950 on a little iron.
const TEST_METHOD: &str = "
[[method]]
name = \"test_works\"
label = \"Test works\"
output = \"test_good0\"
inputs = { iron = 0.5 }
heat = 0.1
labor = 0.1
capital = { tools = 1.0, bricks = 2.0 }
K0 = 100.0
known = false
A0 = 1.0
Q0 = 1000000.0
b = 0.2
discover = { iron = 1000.0 }
found_after = 1950
group = \"engineering\"
headline = \"Test works at {town}\"
story = \"{who} opens a test works at {town}.\"
";

/// The game's content with the test goods (and the test method) laid with the tree.
fn game_with(lines: &str, method: bool) -> Content {
    let mut over = with_goods(tree::GAME_OVERLAY_TOML, lines);
    if method {
        over.push_str(TEST_METHOD);
    }
    tree::lay(&game_content(), &over, game_params().tree.stage).expect("the overlay with test goods is valid")
}

/// The port's content with the test goods.
fn port_with(lines: &str) -> Content {
    content_from_str(&with_goods(DEFAULT_CONTENT_TOML, lines)).expect("the content with test goods is valid")
}

/// A sweep as the command line runs it, on the content given (or the rules' own).
fn sweep_of(rules: &str, seeds: &str, end: i64, content: Option<Content>) -> Sweep {
    let args: Vec<String> = ["--out", "unused", "--seeds", seeds, "--rules", rules, "--families", "6", "--end-year", &end.to_string(), "--quiet"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let o = sweep::parse_args(&args).unwrap();
    let mut plan = sweep::plan(&o).unwrap();
    if let Some(c) = content {
        plan.content = c;
    }
    let jobs = std::thread::available_parallelism().map_or(1, |n| n.get());
    let mut o = o;
    o.jobs = Some(jobs);
    sweep::run(&o, plan)
}

fn same(a: Option<f64>, b: Option<f64>) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan()),
        (None, None) => true,
        _ => false,
    }
}

/// Every fact and every yearly mean the two sweeps share, seed for seed, to the
/// last digit; those only one has must be the test goods' and methods' own.
/// Returns the number of numbers set side by side.
fn hold_equal(what: &str, a: &Sweep, b: &Sweep) -> usize {
    let mut wrong = Vec::new();
    let mut compared = 0;
    let own = |name: &str| name.contains("test_good") || name.contains("test_works");
    for (ra, rb) in a.runs.iter().zip(&b.runs) {
        assert_eq!(ra.seed, rb.seed);
        let seed = ra.seed;
        let (da, db) = match (&ra.outcome, &rb.outcome) {
            (Ok(x), Ok(y)) => (x, y),
            (x, y) => panic!("{what}, island {seed}: a run failed: {:?} / {:?}", x.as_ref().err(), y.as_ref().err()),
        };
        let fa: BTreeMap<&str, Option<f64>> = da.facts.facts.iter().map(|f| (f.name.as_str(), f.value)).collect();
        let fb: BTreeMap<&str, Option<f64>> = db.facts.facts.iter().map(|f| (f.name.as_str(), f.value)).collect();
        for (name, va) in &fa {
            if matches!(*name, "seconds" | "cpu_seconds") {
                continue;
            }
            match fb.get(name) {
                Some(vb) => {
                    compared += 1;
                    if !same(*va, *vb) {
                        wrong.push(format!("island {seed}, {name}: {va:?} against {vb:?}"));
                    }
                }
                None => wrong.push(format!("island {seed}, {name}: has no value with the test goods")),
            }
        }
        for name in fb.keys().filter(|n| !fa.contains_key(*n) && !own(n)) {
            wrong.push(format!("island {seed}, {name}: only with the test goods, and not their own"));
        }
        if da.facts.news != db.facts.news {
            wrong.push(format!("island {seed}: the news differ: {:?} against {:?}", da.facts.news, db.facts.news));
        }
        let built_b: Vec<&String> = db.facts.never_built.iter().filter(|n| !own(n)).collect();
        if da.facts.never_built.iter().collect::<Vec<_>>() != built_b {
            wrong.push(format!("island {seed}: the methods never built differ"));
        }
        let ib: BTreeMap<&str, usize> = db.names.iter().enumerate().map(|(k, n)| (n.as_str(), k)).collect();
        for (k, name) in da.names.iter().enumerate() {
            let Some(&j) = ib.get(name.as_str()) else {
                wrong.push(format!("island {seed}: the series {name} is missing with the test goods"));
                continue;
            };
            for (y, (x, z)) in da.annual[k].iter().zip(&db.annual[j]).enumerate() {
                compared += 1;
                if !same(Some(*x), Some(*z)) {
                    wrong.push(format!("island {seed}, {name} in {}: {x:e} against {z:e}", da.start_year + y as i64));
                    break;
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{what}: {} numbers differ, the first: {:#?}", wrong.len(), &wrong[..wrong.len().min(12)]);
    eprintln!("{what}: {compared} numbers, every one equal");
    compared
}

/// Test goods that begin with the run and nothing makes leave the run as it was.
fn idle_goods(rules: &str, seeds: &str, end: i64) {
    let lines = good_lines(8, "1750");
    let with = if rules == "game" { game_with(&lines, false) } else { port_with(&lines) };
    let base = if rules == "game" { game_with("", false) } else { port_with("") };
    assert_eq!(with.G, base.G + 8);
    assert_eq!(with.G_base, base.G);
    let a = sweep_of(rules, seeds, end, Some(base));
    let b = sweep_of(rules, seeds, end, Some(with));
    hold_equal(&format!("{rules} rules, islands {seeds} to {end}, eight idle goods"), &a, &b);
}

/// A test good whose only method may be found from 1950 leaves the run to 1949 as it was.
fn good_of_1950(seeds: &str) {
    let with = game_with(&good_lines(1, "\"test_works\""), true);
    assert_eq!(with.good_begins[with.gi("test_good0")], Begins::Idea(with.tech_name.iter().position(|n| n == "test_works").unwrap()));
    let a = sweep_of("game", seeds, 1949, Some(game_with("", false)));
    let b = sweep_of("game", seeds, 1949, Some(with));
    hold_equal(&format!("game rules, islands {seeds} to 1949, a good whose method comes in 1950"), &a, &b);
}

#[test]
fn idle_goods_leave_island_2_as_it_was_to_2050_under_the_game() {
    idle_goods("game", "2", 2050);
}

#[test]
fn idle_goods_leave_island_7_as_it_was_under_the_port() {
    idle_goods("port", "7", 1900);
}

#[test]
fn a_good_of_1950_leaves_island_7_as_it_was_to_1949() {
    good_of_1950("7");
}

#[test]
#[ignore]
fn proof_idle_goods_leave_islands_1_to_8_as_they_were_to_2050_under_the_game() {
    idle_goods("game", "1-8", 2050);
}

#[test]
#[ignore]
fn proof_idle_goods_leave_islands_1_to_8_as_they_were_to_2050_under_the_port() {
    idle_goods("port", "1-8", 2050);
}

#[test]
#[ignore]
fn proof_a_good_of_1950_leaves_islands_1_to_8_as_they_were_to_1949() {
    good_of_1950("1-8");
}

// ---------------------------------------------------------------- the content and the sums

#[test]
fn a_good_that_begins_goes_after_every_good_of_the_start() {
    // written before the port's last good, it is put after it
    let text = DEFAULT_CONTENT_TOML.replacen(
        "  { name = \"services\",",
        "  { name = \"late\", label = \"Late\", weight = 1.0, price = 1.0, color = \"#000000\", begins = 1950 },\n  { name = \"services\",",
        1,
    );
    let c = content_from_str(&text).unwrap();
    let port = content_from_str(DEFAULT_CONTENT_TOML).unwrap();
    assert_eq!((c.G, c.G_base), (port.G + 1, port.G));
    assert_eq!(c.good_name[..port.G], port.good_name[..]);
    assert_eq!(c.good_name[port.G], "late");
    assert_eq!(c.good_begins[port.G], Begins::Year(1950));
    assert!(port.good_begins.iter().all(|&b| b == Begins::Start) && port.G_base == port.G);
    // begun from its year
    assert!(!c.good_begun(port.G, 1949, &[]) && c.good_begun(port.G, 1950, &[]));
    // and the game's content as it stood keeps every good among those of the start
    let game = game_with("", false);
    assert_eq!(game.G_base, game.G);
}

#[test]
fn a_good_that_begins_with_an_idea_begins_when_it_is_found() {
    let c = game_with(&good_lines(1, "\"test_works\""), true);
    let g = c.gi("test_good0");
    let t = c.tech_name.iter().position(|n| n == "test_works").unwrap();
    let mut found = vec![-1i64; c.NT];
    assert!(!c.good_begun(g, 2000, &found));
    found[t] = 1953;
    assert!(!c.good_begun(g, 1952, &found) && c.good_begun(g, 1953, &found));
    // an idea the content does not have is refused
    let over = with_goods(tree::GAME_OVERLAY_TOML, &good_lines(1, "\"no_such_idea\""));
    let e = tree::lay(&game_content(), &over, game_params().tree.stage).unwrap_err();
    assert!(e.contains("no_such_idea"), "{e}");
}

#[test]
fn a_term_of_nothing_after_the_goods_of_the_start_moves_no_sum() {
    // 29 terms, as the game's goods: three rows of eight and five more
    let v: Vec<f64> = (0..29).map(|k| (k + 1) as f64 / 7.0).collect();
    let old = pairwise_sum_by(29, |k| v[k]);
    // three terms of nothing make 32: four rows, and numpy's order rounds otherwise
    let mut w = v.clone();
    w.extend([0.0; 3]);
    assert_ne!(pairwise_sum_by(32, |k| w[k]).to_bits(), old.to_bits(), "the case this guards: if it no longer parts, choose terms that do");
    // taken after the 29, they move nothing; and with no later term it is numpy's sum itself
    assert_eq!(pairwise_sum_from(29, 32, |k| w[k]).to_bits(), old.to_bits());
    assert_eq!(pairwise_sum_from(29, 29, |k| v[k]).to_bits(), old.to_bits());
    assert_eq!(pairwise_sum_from(64, 29, |k| v[k]).to_bits(), old.to_bits());
}

// ---------------------------------------------------------------- the probe

fn env(name: &str, d: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| d.to_string())
}

struct Diff(Vec<String>);

impl Diff {
    fn vec(&mut self, name: &str, a: &[f64], b: &[f64]) {
        if a.len() != b.len() {
            self.0.push(format!("{name}: length {} against {}", a.len(), b.len()));
        } else if let Some(i) = (0..a.len()).find(|&i| a[i].to_bits() != b[i].to_bits()) {
            self.0.push(format!("{name}[{i}]: {:e} against {:e}", a[i], b[i]));
        }
    }
    /// (rows, goods) tables, the goods of `ca` found by name in `cb`
    fn by_goods(&mut self, name: &str, a: &Mat, b: &Mat, ca: &Content, cb: &Content) {
        if a.rows != b.rows || a.cols != ca.G || b.cols != cb.G {
            return self.vec(name, &a.data, &b.data);
        }
        for r in 0..a.rows {
            for g in 0..ca.G {
                let h = cb.gi(&ca.good_name[g]);
                if a[(r, g)].to_bits() != b[(r, h)].to_bits() {
                    self.0.push(format!("{name}[{r}, {}]: {:e} against {:e}", ca.good_name[g], a[(r, g)], b[(r, h)]));
                    return;
                }
            }
        }
    }
}

fn compare(a: &Sim, b: &Sim) -> Vec<String> {
    let (s, t, ca, cb) = (&a.st, &b.st, &a.content, &b.content);
    let mut d = Diff(Vec::new());
    for (name, x, y) in [
        ("pop", &s.pop, &t.pop),
        ("wage", &s.wage, &t.wage),
        ("hh_cash", &s.hh_cash, &t.hh_cash),
        ("lord_cash", &s.lord_cash, &t.lord_cash),
        ("merch_cash", &s.merch_cash, &t.merch_cash),
        ("hh_income", &s.hh_income, &t.hh_income),
        ("f_K", &s.f_K, &t.f_K),
        ("f_order_K", &s.f_order_K, &t.f_order_K),
        ("f_cash", &s.f_cash, &t.f_cash),
        ("f_u", &s.f_u, &t.f_u),
        ("f_prod", &s.f_prod, &t.f_prod),
        ("wbar", &s.wbar, &t.wbar),
        ("cell_rent", &s.cell_rent, &t.cell_rent),
        ("ex_labor", &s.ex_labor, &t.ex_labor),
        ("cover", &s.cover, &t.cover),
        ("traffic", &s.traffic, &t.traffic),
    ] {
        d.vec(name, x, y);
    }
    d.vec("money", &[s.issued, s.fund_cash, s.money_now], &[t.issued, t.fund_cash, t.money_now]);
    d.vec("lam", &s.lam.data, &t.lam.data);
    d.vec("dist", &s.dist.data, &t.dist.data);
    for (name, x, y) in [("price", &s.price, &t.price), ("inv", &s.inv, &t.inv), ("fill", &s.fill, &t.fill), ("pbar", &s.pbar, &t.pbar), ("f_order", &s.f_order, &t.f_order)] {
        d.by_goods(name, x, y, ca, cb);
    }
    let cash = |st: &isostate_core::state::State| st.families.iter().map(|f| f.cash).collect::<Vec<f64>>();
    d.vec("family_cash", &cash(s), &cash(t));
    d.0
}

/// PROBE_SEED (2), PROBE_RULES (game or port), PROBE_GOODS (8), PROBE_BEGINS
/// (1750; "none" for goods with no `begins`, which shows the leaks of old),
/// PROBE_END (1800):
///
/// ```text
/// PROBE_GOODS=3 PROBE_BEGINS=none cargo test --profile work -p isostate-core --test new_good probe -- --ignored --nocapture
/// ```
#[test]
#[ignore]
fn probe() {
    let seed: u64 = env("PROBE_SEED", "2").parse().unwrap();
    let rules = env("PROBE_RULES", "game");
    let n: usize = env("PROBE_GOODS", "8").parse().unwrap();
    let end: i64 = env("PROBE_END", "1800").parse().unwrap();
    let begins = env("PROBE_BEGINS", "1750");
    let mut lines = good_lines(n, "1750");
    if begins == "none" {
        lines = lines.replace(", begins = 1750", "");
    } else {
        lines = lines.replace("begins = 1750", &format!("begins = {begins}"));
    }
    let (mut P, ca, cb) = if rules == "port" {
        (isostate_core::default_params(), port_with(""), port_with(&lines))
    } else {
        (game_params(), game_with("", false), game_with(&lines, begins.contains("test_works")))
    };
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end);
    eprintln!("goods {} against {} ({} of the start), methods {} against {}", ca.G, cb.G, cb.G_base, ca.M, cb.M);
    let mut a = sim::start(&P, ca);
    let mut b = sim::start(&P, cb);
    let first = compare(&a, &b);
    assert!(first.is_empty(), "parted before 1750: {first:#?}");
    while a.year <= end {
        for month in 0..12 {
            sim::run_month(&mut a, month).unwrap();
            sim::run_month(&mut b, month).unwrap();
            let d = compare(&a, &b);
            assert!(d.is_empty(), "parted in {}-{:02}: {d:#?}", a.year, month + 1);
        }
        sim::end_year(&mut a);
        sim::end_year(&mut b);
        let d = compare(&a, &b);
        assert!(d.is_empty(), "parted at the end of {}: {d:#?}", a.year - 1);
    }
    eprintln!("equal to the end of {end}");
}
