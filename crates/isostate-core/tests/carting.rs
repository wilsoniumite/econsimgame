//! Carting takes labour (not in the Python; docs/rules/carting.md): the carters
//! come out of each town's supply of hands, measuring alone changes nothing,
//! money is conserved and a run is deterministic. With the switch off the
//! golden test (tests/golden.rs) covers that nothing changed.
#![allow(non_snake_case)]

use isostate_core::carting::CartingParams;
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

    // the carters a town needs are last month's bill at last month's wage, never
    // more than `max_share` of its hands; this month's carters are last month's,
    // moved `adjust` of the way there (all the way at 1, as the first version did)
    for adjust in [CartingParams::game().adjust, 1.0] {
        let mut P1 = params(true, 1.0, 1751);
        P1.carting.adjust = adjust;
        let mut s = sim::start(&P1, default_content());
        sim::run_year(&mut s).expect("money conserved");
        let (bill, wage, had) = (s.st.cart_bill.clone(), s.st.wage.clone(), s.st.carters.clone());
        assert!(bill.iter().sum::<f64>() > 0.0);
        step_month(&mut s.st, &s.content, &s.params);
        for t in (0..s.st.T_max).filter(|&t| s.st.town_active[t]) {
            let hands = s.params.participation * s.st.pop[t];
            let need = (bill[t] / wage[t]).min(s.params.carting.max_share * hands);
            let want = if adjust >= 1.0 { need } else { had[t] + adjust * (need - had[t]) };
            assert!((s.st.carters[t] - want).abs() <= 1e-9 * want.max(1.0), "town {t}, adjust {adjust}: {} carters against {want}", s.st.carters[t]);
            assert!(s.st.carters[t] < hands);
        }
        assert!(s.st.carters.iter().sum::<f64>() > 0.0);
    }

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

/// The share of town-months, February to December, in which a town's wage moved
/// by half or more from the month before.
fn wage_flips(P: &Params) -> f64 {
    let mut s = sim::start(P, default_content());
    s.check_money = true;
    let (mut flips, mut months) = (0usize, 0usize);
    let mut last = s.st.wage.clone();
    while s.year <= s.end_year {
        let year = s.year;
        for month in 0..12 {
            sim::run_month(&mut s, month).expect("money is conserved every month");
            for t in (0..s.st.T_max).filter(|&t| s.st.town_active[t] && s.st.pop[t] > 0.0) {
                if month > 0 && last[t] > 0.0 {
                    months += 1;
                    flips += ((s.st.wage[t] / last[t]).ln().abs() >= 1.5f64.ln()) as usize;
                }
            }
            last = s.st.wage.clone();
        }
        let Sim { st, content: C, params: P, rngs, events, rec, .. } = &mut s;
        sim::yearly(st, C, P, rngs, year, events, false);
        observer::yearly(st, C, P, year, events);
        record::snapshot(rec, st, C, P, (year - C.start_year) as usize);
        s.year += 1;
    }
    flips as f64 / months.max(1) as f64
}

#[test]
fn carters_taken_on_by_degrees_do_not_set_the_wage_flipping() {
    // Carters hired all at once on last month's bill set a town with a heavy bill
    // swinging: many carters, few hands for the rest, the wage doubles, the bill
    // buys half the carters, the wage halves, month about. Taken on a quarter of
    // the way each month they do not.
    let mut at_once = params(true, 1.0, 1789);
    at_once.seed = 4;
    at_once.carting.adjust = 1.0;
    let mut by_degrees = at_once.clone();
    by_degrees.carting.adjust = CartingParams::game().adjust;
    let mut none = at_once.clone();
    none.carting.enabled = false;
    let (was, now, port) = (wage_flips(&at_once), wage_flips(&by_degrees), wage_flips(&none));
    assert!(was > 0.03, "the test sees the flipping it is for: {was:.4} of town-months with every carter hired at once");
    assert!(now < 0.005, "{now:.4} of town-months have the wage move by half or more");
    // what is left is the port's own (a new town's first months), give or take
    assert!(now <= port + 0.003, "{now:.4} with carters against {port:.4} without");
}
