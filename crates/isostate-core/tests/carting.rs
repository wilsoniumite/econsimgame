//! Carting takes labour (not in the Python; docs/rules/carting.md): the carters
//! come out of each town's supply of hands, measuring alone changes nothing,
//! money is conserved and a run is deterministic. With the switch off the
//! golden test (tests/golden.rs) covers that nothing changed.
#![allow(non_snake_case)]

use isostate_core::observer::annual;
use isostate_core::*;

fn params(on: bool, labor: f64, end_year: i64) -> Params {
    let mut P = default_params();
    P.seed = 3;
    P.end_year = Some(end_year);
    P.carting.enabled = on;
    P.carting.labor = labor;
    P
}

fn run(P: &Params) -> RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(P, default_content(), false, true).expect("money is conserved every month")
}

fn series<'a>(r: &'a RunResult, name: &str) -> &'a [f64] {
    &r.state.series.data[r.state.series.index[name]]
}

#[test]
fn carting_conserves_money_and_runs_are_deterministic() {
    // forty years with carters on the road
    let P = params(true, 1.0, 1789);
    let (a, b) = (run(&P), run(&P));
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    for (k, name) in st.series.names.iter().enumerate() {
        let (x, y) = (&st.series.data[k], &b.state.series.data[k]);
        assert!(x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()), "series {name} differs between identical runs");
    }
    assert!(st.carters == b.state.carters && st.cart_years == b.state.cart_years);
}

#[test]
fn measuring_the_bill_changes_nothing() {
    // with `labor = 0` the bill is added up and nobody is taken off other work
    let (off, measured) = (run(&params(false, 1.0, 1756)), run(&params(true, 0.0, 1756)));
    for (k, name) in off.state.series.names.iter().enumerate() {
        let (x, y) = (&off.state.series.data[k], &measured.state.series.data[k]);
        assert!(x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()), "series {name} moved");
    }
    assert!(off.state.cart_years.is_empty() && off.state.cart_bill.is_empty() && off.state.carters.is_empty(), "nothing is kept while the switch is off");
    let years = &measured.state.cart_years;
    assert!(years.iter().all(|&(bill, carters)| bill > 0.0 && carters == 0.0), "a bill every year and no carters");
}

#[test]
fn carters_come_out_of_the_supply_of_hands() {
    let P = params(true, 1.0, 1759);
    let (off, on) = (run(&params(false, 1.0, 1759)), run(&P));
    let st = &on.state;
    let y0 = on.content.start_year;

    // this month's carters are last month's bill at last month's wage, town by
    // town, and never more than `max_share` of a town's hands
    let mut s = sim::start(&params(true, 1.0, 1751), default_content());
    sim::run_year(&mut s).expect("money conserved");
    let (bill, wage) = (s.st.cart_bill.clone(), s.st.wage.clone());
    assert!(bill.iter().sum::<f64>() > 0.0);
    step_month(&mut s.st, &s.content, &s.params);
    for t in (0..s.st.T_max).filter(|&t| s.st.town_active[t]) {
        let hands = s.params.participation * s.st.pop[t];
        let want = (bill[t] / wage[t]).min(s.params.carting.max_share * hands);
        assert!((s.st.carters[t] - want).abs() <= 1e-9 * want.max(1.0), "town {t}: {} carters against {want}", s.st.carters[t]);
        assert!(s.st.carters[t] < hands);
    }
    assert!(s.st.carters.iter().sum::<f64>() > 0.0);

    // over the first ten years: a believable share of all hands is on the road...
    let hands: f64 = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| P.participation * st.pop[t]).sum();
    let share = st.carters.iter().sum::<f64>() / hands;
    assert!(share > 0.02 && share < 0.25, "carters are {share:.3} of all hands");
    // ...the whole warm-up and every year since has its total...
    assert_eq!(st.cart_years.len(), (P.warmup_years + 1759 - y0 + 1) as usize);
    // ...and with fewer hands on the land and in the works, less is made
    let mean = |r: &RunResult, name: &str| (y0..=1759).map(|y| annual(series(r, name), y0, y)).sum::<f64>() / 10.0;
    let goods = |r: &RunResult| (0..r.content.G).map(|g| mean(r, &format!("out_{}", r.content.good_name[g])) * r.content.base_price[g]).sum::<f64>();
    assert!(goods(&on) < 0.99 * goods(&off), "{} of goods a month with carters against {} without", goods(&on), goods(&off));
}
