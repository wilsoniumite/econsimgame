//! Hands and machines in the works (not in the Python; docs/rules/tasks.md).
//! With the switch off the golden test (tests/golden.rs) covers that the port
//! did not move.
//!
//! The island runs here are played on island 7 to 1925 under the game's rules
//! with six families, with the switch they lean on set by name
//! (`tasks.enabled`) and every other rule as the game has it. What they check
//! holds on any island's history (a run against another run, a new works
//! against its town, shares within their bounds, money that adds up), so a
//! later rule that changes island 7's history cannot break them; only one
//! that stopped the hoist being found by 1925 would, and the test says so.
#![allow(non_snake_case)]

use isostate_core::tasks::{self, TasksParams};
use isostate_core::*;

const END: i64 = 1925;

fn params(on: bool) -> Params {
    let mut P = game_params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(END);
    P.tasks = TasksParams { enabled: on, ..TasksParams::game() };
    P
}

/// The game's content as a run under `P` plays it (with the tree laid).
fn content(P: &Params) -> Content {
    isostate_core::tree::content(P, game_content()).expect("the game's tree lays")
}

#[test]
fn the_switch_is_off_under_the_port_and_on_in_the_game() {
    let (port, game) = (default_params(), game_params());
    assert_eq!(port.tasks, TasksParams::default());
    assert!(!port.tasks.enabled && game.tasks.enabled);
    assert!(!serde_json::to_string(&port).unwrap().contains("\"tasks\""));
    // only the game's content has the classes
    assert!(default_content().tasks.is_empty());
    let C = content(&game);
    assert_eq!(C.tasks.method.len(), C.M);
    assert!(C.tasks.closed[tasks::PRESENCE] && C.tasks.spread[tasks::PRESENCE].is_empty());
    // every method of the game, the tree's among them, has its labour in classes that sum to one
    for m in 0..C.M {
        let s: f64 = C.tasks.method[m].iter().sum();
        assert!((s - 1.0).abs() < 1e-9, "{} has shares summing to {s}", C.method_name[m]);
    }
    assert_eq!(C.M, 44);
    assert_eq!(C.tasks.uses.len(), 1);
    assert_eq!(C.tasks.uses[0].name, "hoist");
}

#[test]
fn the_schedule_adds_up() {
    let haul = [(0.0, 0.3), (1.0, 100.0)];
    let (lo, hi): (f64, f64) = (0.3, 100.0);
    // units for a part of an even spread: lo * ((hi/lo)^b - (hi/lo)^a) / ln(hi/lo)
    for (a, b) in [(0.0, 1.0), (0.0, 0.2), (0.2, 0.5), (0.37, 0.91)] {
        let want = lo * ((hi / lo).powf(b) - (hi / lo).powf(a)) / (hi / lo).ln();
        let got = tasks::units(&haul, a, b);
        assert!((got - want).abs() < 1e-12 * want.max(1.0), "[{a}, {b}]: {got} against {want}");
    }
    assert_eq!(tasks::units(&haul, 0.5, 0.5), 0.0);
    // a part split in two holds what the whole holds, also across a bend
    let bent = [(0.0, 1.0), (0.4, 30.0), (1.0, 3000.0)];
    let whole = tasks::units(&bent, 0.1, 0.9);
    let parts = tasks::units(&bent, 0.1, 0.4) + tasks::units(&bent, 0.4, 0.9);
    assert!((whole - parts).abs() < 1e-12 * whole);
    // the share held at a given wage and cost: a hand worth w/c units
    for (w, c) in [(8.18, 8.50), (10.0, 2.0), (1.0, 5.0), (100.0, 0.5)] {
        let x = tasks::share_at(&haul, w / c);
        let want = ((w / c / lo).ln() / (hi / lo).ln()).clamp(0.0, 1.0);
        assert!((x - want).abs() < 1e-12, "{w}/{c}: {x} against {want}");
        if x > 0.0 && x < 1.0 {
            assert!((tasks::worth(&haul, x) - w / c).abs() < 1e-9 * w / c);
        }
    }
    // a unit costing 1.04 of a month's wage takes a fifth of hauling
    assert!((tasks::share_at(&haul, 1.0 / 1.04) - 0.20).abs() < 0.005);
    // the inverse on a bent schedule
    let x = tasks::share_at(&bent, 100.0);
    assert!((tasks::worth(&bent, x) - 100.0).abs() < 1e-9);
    assert!((tasks::rank_correlation(&[1.0, 2.0, 3.0, 4.0], &[10.0, 20.0, 15.0, 40.0]) - 0.8).abs() < 1e-12);
}

/// With the table of uses emptied in code, the game's run on island 7 to 1925
/// is the run with the switch off, to the last digit; with the hoist it is
/// not. And in the run with the hoist: it is found, a works built after that
/// begins at its town's target, every works' share stays between the use's
/// `from` and `reach`, and money is conserved every month (`run_year` checks).
#[test]
fn island_7_to_1925() {
    let on = std::thread::spawn(|| {
        let P = params(true);
        let mut s = sim::start(&P, content(&P));
        let (mut at_target, mut found) = (0, None);
        let us = s.content.tasks.uses[0].clone();
        while s.year <= s.end_year {
            let year = s.year;
            sim::run_year(&mut s).expect("money is conserved every month");
            let st = &s.st;
            if found.is_none() && st.tasks.found[0] >= 0 {
                found = Some(st.tasks.found[0]);
            }
            for &f in &st.new_firms_year {
                if !st.f_alive[f] || st.tasks.target.is_empty() {
                    continue;
                }
                let t = st.tasks.target[st.f_town[f]];
                assert_eq!(st.tasks.f_auto[f], t, "{year}: works {f} begins off its town's target");
                if t[tasks::HAUL] > 0.0 {
                    at_target += 1;
                }
            }
            for f in 0..st.F_count.min(st.tasks.f_auto.len()) {
                let x = st.tasks.f_auto[f][tasks::HAUL];
                assert!(x == 0.0 || (us.from..=us.reach).contains(&x), "{year}: works {f} holds {x}");
                assert!(st.tasks.f_auto[f].iter().enumerate().all(|(k, &v)| k == tasks::HAUL || v == 0.0));
            }
        }
        let y = found.expect("island 7 finds the hoist by 1925 (if a later rule stops that, this test must be set again)");
        assert!((1901..=1925).contains(&y));
        assert!(at_target > 0, "no works was built with machines after {y}");
        let last = s.st.tasks.years.last().unwrap();
        assert!(last.held[tasks::HAUL] > 0.0 && last.book > 0.0, "{last:?}");
        save::check(&s)
    });
    let check_of = |P: Params, C: Content| {
        std::thread::spawn(move || {
            let mut s = sim::start(&P, C);
            while s.year <= s.end_year {
                sim::run_year(&mut s).expect("money is conserved every month");
            }
            save::check(&s)
        })
    };
    let P = params(true);
    let mut empty = content(&P);
    empty.tasks.uses.clear();
    empty.tasks.plant_goods.clear();
    let none = check_of(P, empty);
    let off = check_of(params(false), content(&params(false)));
    let (on, none, off) = (on.join().unwrap(), none.join().unwrap(), off.join().unwrap());
    assert_eq!(none, off, "with no use the rule moved the run");
    assert_ne!(on, off, "the hoist moved nothing");
}
