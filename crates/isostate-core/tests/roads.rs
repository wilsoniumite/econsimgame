//! Roads that somebody pays for (a probe, not in the Python; docs/probes/dx-roads.md):
//! with the switch on, money is conserved, a run is deterministic, the parishes
//! make roads, the trusts take tolls and pay their lenders, and a road works is
//! never listed as a railway building. With the switch off nothing is kept, and
//! the golden test (tests/golden.rs) covers that the port did not move.
#![allow(non_snake_case)]

use isostate_core::*;

fn params(on: bool) -> Params {
    let mut P = game_params();
    P.seed = 3;
    P.end_year = Some(1789);
    P.roads.enabled = on;
    P
}

fn run(on: bool) -> RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(&params(on), game_content(), false, true).expect("money is conserved every month")
}

#[test]
fn paid_roads_conserve_money_and_pay_their_way() {
    // forty years of the game's rules with the roads paid for
    let (a, b) = (run(true), run(true));
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    assert!(st.road == b.state.road && st.rd == b.state.rd, "the same roads are paid for both times");

    let sum = |f: &dyn Fn(&roads::RoadYear) -> f64| st.rd.years.iter().map(f).sum::<f64>();
    assert!(sum(&|y| y.made as f64) > 0.0, "the parishes make roads");
    assert!(sum(&|y| y.rates) > 0.0 && sum(&|y| y.spent) > 0.0, "out of a rate, spent on stone and wages");
    assert!(sum(&|y| y.tolls) > 0.0 && sum(&|y| y.dividends) > 0.0, "the trusts take tolls and pay their lenders");
    assert!(sum(&|y| y.dividends) <= sum(&|y| y.tolls), "a trust pays out no more than it took");
    assert!(st.rd.cell_trust.iter().any(|&t| t >= 0), "some road is under a trust at the end");
    // a works orders what its cash will pay for: it never owes anything
    assert!(st.projects.iter().all(|pr| pr.cash >= 0.0));

    // the record's list of railways building leaves out road works, and every
    // trust has a standing works for its repairs, so there is one to leave out
    let repairs = st.projects.iter().filter(|pr| pr.way == ways::ROAD && pr.serves == roads::REPAIRS).count();
    assert!(repairs > 0, "the trusts keep their roads");
    let lines = st.projects.iter().filter(|pr| pr.way == ways::PORT || pr.way == ways::RAILWAY).count();
    assert_eq!(a.rec.projects.last().expect("a year recorded").len(), lines, "only lines are listed as building");
}

#[test]
fn switched_off_nothing_is_kept() {
    let off = run(false);
    let st = &off.state;
    assert!(st.rd == roads::RoadState::default());
    assert!(st.projects.iter().all(|pr| pr.way != ways::ROAD));
    assert!(!off.events.iter().any(|e| e.kind == "turnpike"));
    assert!(!default_params().roads.enabled && !game_params().roads.enabled, "off under both sets of rules");
}
