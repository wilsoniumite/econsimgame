//! Crossings (not in the Python; docs/rules/crossings.md): on the new land,
//! money is conserved with bridges being paid for, the parishes build bridges
//! and keep them, a bridge works is never listed as a railway building, and
//! the run is the same twice. On the old island the rule does nothing.
//!
//! Played under `game_params()` with the land switched on by name
//! (`land.enabled`, off in the game's rules for now) and the crossings by
//! name, on island 3 to 1779.
#![allow(non_snake_case)]

use isostate_core::*;

fn params(land: bool) -> Params {
    let mut P = game_params();
    P.seed = 3;
    P.end_year = Some(1779);
    P.land.enabled = land;
    P.crossings.enabled = true;
    P
}

fn run(land: bool) -> RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(&params(land), game_content(), false, true).expect("money is conserved every month")
}

#[test]
fn bridges_are_paid_for_and_money_is_conserved_on_the_new_land() {
    let (a, b) = (run(true), run(true));
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    assert!(st.xs == b.state.xs && st.road == b.state.road, "the same bridges are built both times");

    let xs = &st.xs;
    assert!(!xs.table.links.is_empty(), "the island's ways cross water");
    let built: Vec<&crossings::Built> = xs.bridges.iter().filter(|b| !b.inherited).collect();
    assert!(!built.is_empty(), "the parishes build bridges");
    assert!(built.iter().all(|b| b.cost > 0.0 && b.span > 0.0), "each bridge built cost money and spans water");
    assert!(xs.bridges.iter().any(|b| b.inherited), "1750's made roads stand with some bridges");
    // a bridge, once built, stays: every one is still standing, in the year it was built
    for b in &xs.bridges {
        assert_eq!(xs.built[b.link as usize], b.year as i32);
    }
    // a works orders what its cash will pay for: it never owes anything
    assert!(st.projects.iter().all(|pr| pr.cash >= 0.0));
    // the record's list of railways building leaves out the bridge works
    let lines = st.projects.iter().filter(|pr| pr.way == ways::PORT || pr.way == ways::RAILWAY).count();
    assert_eq!(a.rec.projects.last().expect("a year recorded").len(), lines, "only lines are listed as building");
}

#[test]
fn on_the_old_island_nothing_is_kept() {
    let off = run(false);
    let st = &off.state;
    assert!(st.xs == crossings::CrossingState::default());
    assert!(st.projects.iter().all(|pr| pr.way != ways::ROAD));
    assert!(!off.events.iter().any(|e| e.kind == "bridge"));
    assert!(!default_params().crossings.enabled && game_params().crossings.enabled, "off under the port's rules, on under the game's");
}
