//! An island that can last three centuries (not in the Python;
//! docs/rules/longrun.md): the bounds on prices and the wage, and the freight,
//! ride on a chained cost of living the simulation keeps for itself. With the
//! switch off the golden test (tests/golden.rs) covers that nothing changed.
//!
//! The runs here are short islands under the game's rules with the switches
//! they lean on set by name: `longrun.enabled` (and `freight_from`), and
//! `engel.enabled`, whose observer keeps the chain the anchor is checked
//! against. Nothing here leans on how a later rule shapes an island's history.
#![allow(non_snake_case)]

use isostate_core::longrun::{self, LongrunParams, STANDIN_CAP, WAGE_HI, WAGE_LO};
use isostate_core::*;

fn params(seed: u64) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 0;
    P.longrun = LongrunParams { enabled: true, ..LongrunParams::game() };
    P.engel.enabled = true;
    P
}

/// `years` recorded years of island `seed` under `P`, a year at a time.
fn play(P: &Params, years: i64) -> sim::Sim {
    let mut sim = sim::start(P, game_content());
    for _ in 0..years {
        sim::run_year(&mut sim).expect("money is conserved every month");
    }
    sim
}

#[test]
fn the_switch_is_off_under_the_port_and_on_in_the_game() {
    let (port, game) = (default_params(), game_params());
    assert_eq!(port.longrun, LongrunParams::default());
    assert!(!port.longrun.enabled && game.longrun.enabled);
    assert!(!port.longrun.standin_growth && !game.longrun.standin_growth);
    assert_eq!((game.longrun.anchor_years, game.longrun.freight_from), (5, 1900));
    // a port run's params.json does not mention it
    assert!(!serde_json::to_string(&port).unwrap().contains("\"longrun\""));
    // and a live game never runs the stand-in, whatever its rules say
    assert!(!save::params(Rules::Game, 7, 6).longrun.standin_growth);
}

#[test]
fn off_the_bounds_and_the_freight_are_the_ports_constants() {
    let P = default_params();
    let C = default_content();
    let mut st = State::default();
    // whatever the state holds, an island with the switch off reads the constants
    st.longrun.anchor = 3.0;
    st.longrun.wage_scale = 7.0;
    st.longrun.levels = vec![3.0];
    st.longrun.freight_base = 1.0;
    for g in 0..C.G {
        assert_eq!(longrun::ceiling(&st, &P, &C, g).to_bits(), (P.price_ceiling * C.base_price[g]).to_bits());
        assert_eq!(longrun::floor(&st, &P, &C, g).to_bits(), (P.price_floor * C.base_price[g]).to_bits());
    }
    assert_eq!(longrun::wage_bounds(&st, &P), (WAGE_LO, WAGE_HI));
    assert_eq!(longrun::freight(&st, &P).to_bits(), P.freight.to_bits());
    assert_eq!(longrun::learn_cap(&P), P.learn_cap);
    assert_eq!(longrun::quality_cap(&P), P.machines.quality_cap);
}

#[test]
fn on_the_bounds_ride_on_the_anchor_and_the_wage_scale() {
    let mut P = default_params();
    P.longrun = LongrunParams { enabled: true, freight_from: 1750, ..LongrunParams::default() };
    let C = default_content();
    let mut st = State::default();
    // before the warm-up ends nothing has moved
    assert_eq!(longrun::ceiling(&st, &P, &C, 0), P.price_ceiling * C.base_price[0]);
    assert_eq!(longrun::freight(&st, &P), P.freight);
    st.longrun.levels = vec![1.0];
    st.longrun.freight_base = 1.0;
    // a cost of living of half, and a wage of three times, 1750's
    st.longrun.anchor = 0.5;
    st.longrun.wage_scale = 3.0;
    let g = 0;
    assert_eq!(longrun::ceiling(&st, &P, &C, g), P.price_ceiling * C.base_price[g] * 3.0);
    assert_eq!(longrun::floor(&st, &P, &C, g), P.price_floor * C.base_price[g] * 0.5);
    assert_eq!(longrun::wage_bounds(&st, &P), (WAGE_LO * 0.5, WAGE_HI * 3.0));
    assert_eq!(longrun::freight(&st, &P), P.freight * 0.5);
    // the ceilings ride on the higher of the two
    st.longrun.wage_scale = 0.25;
    assert_eq!(longrun::ceiling(&st, &P, &C, g), P.price_ceiling * C.base_price[g] * 0.5);
    assert_eq!(longrun::wage_bounds(&st, &P).1, WAGE_HI * 0.5);
    // the freight counts from the anchor of its first year
    st.longrun.freight_base = 0.25;
    assert_eq!(longrun::freight(&st, &P), P.freight * 2.0);
    st.longrun.freight_base = 0.0;
    assert_eq!(longrun::freight(&st, &P), P.freight);
    // the stand-in lifts both caps, with or without the anchors
    P.longrun.standin_growth = true;
    P.longrun.enabled = false;
    assert_eq!((longrun::learn_cap(&P), longrun::quality_cap(&P)), (STANDIN_CAP, STANDIN_CAP));
}

/// The anchor's chain is the observer's chained cost of living, made again:
/// at every year's end the two agree, and the anchor is the mean of the last
/// five years' levels (the end of the warm-up's 1 among them at first).
#[test]
fn the_chain_is_the_observers_and_the_anchor_its_mean_of_five_years() {
    let P = params(3);
    let mut sim = sim::start(&P, game_content());
    let mut levels = vec![1.0];
    for _ in 0..12 {
        sim::run_year(&mut sim).expect("money is conserved every month");
        let lr = &sim.st.longrun;
        let e = sim.st.engel.chain;
        assert!((lr.chain - e).abs() <= 1e-12 * e, "{}: the anchor's chain {} and the observer's {e}", sim.year - 1, lr.chain);
        levels.push(lr.chain);
        let last: Vec<f64> = levels.iter().rev().take(5).copied().collect();
        let mean = last.iter().sum::<f64>() / last.len() as f64;
        assert!((lr.anchor - mean).abs() <= 1e-12, "{}: anchor {} for a mean of {mean}", sim.year - 1, lr.anchor);
        assert_eq!(lr.years.len() as i64, sim.year - 1750);
        assert!(lr.wage_scale > 0.2 && lr.wage_scale < 5.0, "{}: wage scale {}", sim.year - 1, lr.wage_scale);
    }
    // the chain moved, so the anchor did
    assert!(sim.st.longrun.anchor != 1.0);
}

/// Before `freight_from` the freight is the constant; from it, the anchor
/// over the anchor as that year began.
#[test]
fn the_freight_waits_for_its_year() {
    let mut P = params(5);
    P.longrun.freight_from = 1755;
    let mut sim = sim::start(&P, game_content());
    for _ in 0..5 {
        assert_eq!(longrun::freight(&sim.st, &P).to_bits(), P.freight.to_bits(), "{}", sim.year);
        sim::run_year(&mut sim).expect("money is conserved every month");
    }
    // 1755 begins on the anchor 1754 closed with
    let base = sim.st.longrun.years[4].0;
    assert_eq!(sim.st.longrun.freight_base, base);
    assert_eq!(longrun::freight(&sim.st, &P), P.freight);
    let sim = {
        let mut sim = sim;
        sim::run_year(&mut sim).expect("money is conserved every month");
        sim
    };
    let now = sim.st.longrun.anchor;
    assert!(now != base);
    assert_eq!(longrun::freight(&sim.st, &P), P.freight * (now / base));
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

/// The facts are there under either setting, and the anchors only with it on.
#[test]
fn the_facts() {
    let mut P = params(2);
    P.end_year = Some(1805);
    let on = facts_of(&P);
    P.longrun.enabled = false;
    let off = facts_of(&P);
    for f in [&on, &off] {
        assert!(f.get("months_at_bound_timber").is_some_and(|v| (0.0..=1.0).contains(&v)));
        assert!(f.get("months_at_bound_wage").is_some());
        assert!(f.get("works_slots_used").is_some_and(|v| v > 0.0));
    }
    assert!(on.get("anchor_1800").is_some_and(|v| v > 0.0) && on.get("wage_scale_1805").is_some());
    assert!(off.get("anchor_1800").is_none());
}

#[test]
fn a_short_run_is_conserved_with_the_stand_in() {
    let mut P = params(4);
    P.longrun.standin_growth = true;
    let sim = play(&P, 3);
    assert_eq!(sim.year, 1753);
}
