//! Fewer children as a town's townsfolk live better, and less crowding once
//! drains come with income (not in the Python; docs/rules/transition.md). With
//! the switch off the golden test (tests/golden.rs) covers that the port did
//! not move.
//!
//! The island runs here are played under the game's rules with the switches
//! they lean on set by name: `transition.enabled` and `people.enabled`, whose
//! growth rule is the one the transition works inside. What they check holds
//! on any island's history (bounds, and one run against another), so no later
//! rule can break them by changing island 7's.
#![allow(non_snake_case)]

use isostate_core::mat::Mat;
use isostate_core::transition::{self, TransitionParams};
use isostate_core::*;

fn params(seed: u64, on: bool) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 0;
    P.people.enabled = true;
    P.transition = TransitionParams { enabled: on, ..TransitionParams::game() };
    P
}

#[test]
fn the_switch_is_off_under_the_port_and_on_in_the_game() {
    let (port, game) = (default_params(), game_params());
    assert_eq!(port.transition, TransitionParams::default());
    assert!(!port.transition.enabled && game.transition.enabled);
    let t = &game.transition;
    assert_eq!((t.lag_years, t.z_half, t.z_power, t.floor, t.clean_from, t.clean_span), (25.0, 4.5, 4.0, 0.04, 4.0, 6.0));
    assert_eq!(TransitionParams::default().z_half, 4.0);
    // a port run's params.json does not mention it
    assert!(!serde_json::to_string(&port).unwrap().contains("\"transition\""));
}

/// The two curves at the points the page quotes, with the income term halved
/// at a `zbar` of 4 (the default; the game's is 4.5).
#[test]
fn the_share_of_births_kept_and_the_drains() {
    let tp = TransitionParams { enabled: true, ..TransitionParams::default() };
    assert_eq!(transition::fertility(&tp, 0.0), 1.0);
    assert!((transition::fertility(&tp, 4.0) - 0.52).abs() < 1e-12);
    let far = transition::fertility(&tp, 40.0);
    assert!(far > 0.04 && far < 0.0402, "{far}");
    // it only falls as townsfolk live better
    let mut last = 1.0;
    for k in 1..=100 {
        let f = transition::fertility(&tp, k as f64 * 0.2);
        assert!(f < last);
        last = f;
    }
    assert_eq!(transition::drains(&tp, 4.0), 0.0);
    assert_eq!(transition::drains(&tp, 10.0), 1.0);
    assert_eq!(transition::drains(&tp, 7.0), 0.5);
    assert_eq!((transition::drains(&tp, 1.0), transition::drains(&tp, 30.0)), (0.0, 1.0));
}

/// Migrants carry their towns' measure: a town's becomes the people-weighted
/// mean of what it kept and what came in.
#[test]
fn migrants_carry_their_towns_measure() {
    let P = params(1, true);
    let mut st = State::default();
    st.T_max = 3;
    st.pop = vec![1000.0, 500.0, 200.0];
    st.transition.zbar = vec![2.0, 4.0, 1.0];
    let act = [0, 1, 2];
    // 100 from town 0 to town 1, 50 from town 2 to town 1
    let mut flow = Mat::zeros(3, 3);
    flow[(0, 1)] = 100.0;
    flow[(2, 1)] = 50.0;
    transition::migrate(&mut st, &P, &act, &flow);
    let want = (500.0 * 4.0 + 100.0 * 2.0 + 50.0 * 1.0) / 650.0;
    assert!((st.transition.zbar[1] - want).abs() < 1e-12, "{}", st.transition.zbar[1]);
    // those who only lost people keep their own
    assert_eq!((st.transition.zbar[0], st.transition.zbar[2]), (2.0, 1.0));
    // and with the switch off nothing moves
    let mut off = State::default();
    off.T_max = 3;
    off.pop = st.pop.clone();
    off.transition.zbar = vec![2.0, 4.0, 1.0];
    transition::migrate(&mut off, &params(1, false), &act, &flow);
    assert_eq!(off.transition.zbar, vec![2.0, 4.0, 1.0]);
}

/// A district that changes market brings its town's measure; a town founded
/// this year is only what came.
#[test]
fn a_district_brings_its_measure_and_a_new_town_is_what_came() {
    let P = params(1, true);
    let mut st = State::default();
    st.T_max = 3;
    st.town_new = vec![false, false, true];
    st.transition.zbar = vec![2.0, 6.0, 9.0];
    let pop = [1000.0, 400.0, 0.0];
    let mut share = Mat::zeros(3, 3);
    // a tenth of town 0 goes to town 1; a fifth of town 0 and a quarter of town 1 found town 2
    share[(0, 1)] = 0.1;
    share[(0, 2)] = 0.2;
    share[(1, 2)] = 0.25;
    transition::move_with_cells(&mut st, &P, &pop, &share);
    let one = (400.0 * 0.75 * 6.0 + 100.0 * 2.0) / (300.0 + 100.0);
    let two = (200.0 * 2.0 + 100.0 * 6.0) / 300.0;
    assert!((st.transition.zbar[1] - one).abs() < 1e-12, "{}", st.transition.zbar[1]);
    assert!((st.transition.zbar[2] - two).abs() < 1e-12, "{}", st.transition.zbar[2]);
    assert_eq!(st.transition.zbar[0], 2.0);
}

/// Island 7 under the game's rules to 1820 with the rule on: every active
/// town has a finite measure every year, at least 0 and at most the most
/// baskets a head any town has had so far (a town's `z` is its townsfolk's
/// share times its baskets a head, so never more than those; and `zbar` is a
/// mean of `z`s). The share of births kept is between the floor and 1.
#[test]
fn island_7_keeps_a_sound_measure_in_every_town() {
    let mut P = params(7, true);
    P.end_year = Some(1820);
    let mut sim = sim::start(&P, game_content());
    let mut most_y: f64 = 0.0;
    let mut years = 0;
    while sim.year <= sim.end_year {
        sim::run_year(&mut sim).expect("money is conserved every month");
        let st = &sim.st;
        for t in (0..st.T_max).filter(|&t| st.town_active[t]) {
            most_y = most_y.max(st.y_ratio[t]);
        }
        assert_eq!(st.transition.zbar.len(), st.T_max);
        for t in (0..st.T_max).filter(|&t| st.town_active[t]) {
            let zb = st.transition.zbar[t];
            assert!(zb.is_finite() && zb >= 0.0 && zb <= most_y + 1e-9, "{}: {} has zbar {zb}, the most baskets a head so far {most_y}", sim.year - 1, st.town_name[t]);
        }
        let &(year, f, z) = st.transition.years.last().unwrap();
        assert_eq!(year, sim.year - 1);
        assert!(f >= P.transition.floor && f <= 1.0 && z >= 0.0, "{year}: F {f}, z {z}");
        years += 1;
    }
    assert_eq!(years, 71);
    // and the facts are there: the island's means at the marks, and the peak of people
    let mut P = params(7, true);
    P.end_year = Some(1760);
    let on = facts_of(&P);
    assert!(on.get("transition_F_1750").is_some_and(|f| f > 0.0 && f <= 1.0));
    assert!(on.get("transition_z_1760").is_some_and(|z| z >= 0.0));
    assert!(on.get("people_peak").is_some_and(|p| p > 100_000.0));
    assert!(on.get("people_peak_year").is_some_and(|y| (1750.0..=1760.0).contains(&y)));
    let off = facts_of(&params(7, false).tap(|P| P.end_year = Some(1760)));
    assert!(off.get("transition_F_1750").is_none() && off.get("people_peak_year").is_some());
}

/// A run's facts, gathered as the sweep gathers them.
fn facts_of(P: &Params) -> facts::Facts {
    let mut s = sim::start(P, game_content());
    let mut g = facts::Gathered::default();
    while s.year <= s.end_year {
        facts::run_year_noting(&mut s, &mut g).expect("money is conserved every month");
    }
    facts::stylised_facts(&sim::finish(s), &g, 0.0)
}

trait Tap: Sized {
    fn tap(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}
impl Tap for Params {}

/// Switched off, the rule's numbers reach nothing: the game with the switch off
/// and with every number of it changed is the same history to the last digit
/// (`save::check`), and its state stays empty.
#[test]
fn off_is_the_game_without_it() {
    let mut a = params(3, false);
    a.end_year = Some(1765);
    let mut b = a.clone();
    b.transition = TransitionParams { enabled: false, lag_years: 2.0, z_half: 0.5, z_power: 1.0, floor: 0.9, clean_from: 0.0, clean_span: 0.1 };
    let play = |P: &Params| {
        let mut sim = sim::start(P, game_content());
        while sim.year <= sim.end_year {
            sim::run_year(&mut sim).expect("money is conserved every month");
        }
        sim
    };
    let (x, y) = (play(&a), play(&b));
    assert_eq!(save::check(&x), save::check(&y));
    assert!(x.st.transition.zbar.is_empty() && x.st.transition.years.is_empty());
    // and with it on, the island is another
    let mut c = a.clone();
    c.transition.enabled = true;
    assert_ne!(save::check(&play(&c)), save::check(&x));
}
