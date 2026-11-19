//! A town cannot hire hands it does not have (`docs/rules/hands.md`, not in the
//! Python): switched off it is the port (tests/golden.rs holds that to the
//! byte); switched on, a town short of hands at the month's highest wage is
//! held at that wage and every employer in it is cut by the same share to the
//! hands it has, a town with nothing left hires nobody, money is conserved, and
//! the one town of 1751 that hired more than it had on island 3 does not.
//! Each test also shows the fault it guards against with the rule off, so that
//! it fails if the rule is taken away.
#![allow(non_snake_case)]

use isostate_core::hands::HandsParams;
use isostate_core::*;

/// The game's rules on island 3, to the end of `end_year`.
fn params(ration: bool, n_families: usize, end_year: i64) -> Params {
    let mut P = game_params();
    P.seed = 3;
    P.n_families = n_families;
    P.end_year = Some(end_year);
    P.hands.ration = ration;
    P
}

/// The island warmed up, at the first recorded month.
fn started(P: &Params) -> sim::Sim {
    sim::start(P, game_content())
}

/// A town's hands at work on the land and in the works (not the carters, sinkers or navvies).
fn hired(st: &State, t: usize) -> f64 {
    let land: f64 = (0..st.ex_town.len()).filter(|&u| st.ex_town[u] == t).map(|u| st.ex_Lprev[u]).sum();
    let works: f64 = (0..st.f_alive.len()).filter(|&f| st.f_alive[f] && st.f_town[f] == t).map(|f| st.f_labor[f]).sum();
    land + works
}

/// The most populous town.
fn largest_town(st: &State) -> usize {
    (0..st.T_max).filter(|&t| st.town_active[t]).max_by(|&a, &b| st.pop[a].total_cmp(&st.pop[b])).unwrap()
}

#[test]
fn the_switch_is_off_under_the_port_and_on_in_the_game() {
    assert_eq!(default_params().hands, HandsParams::default());
    assert!(!default_params().hands.ration && game_params().hands.ration);
    // a port run's params.json never mentions it
    let json = serde_json::to_string(&default_params()).unwrap();
    assert!(!json.contains("\"hands\""));
    assert!(serde_json::to_string(&game_params()).unwrap().contains("\"hands\":{\"ration\":true}"));
}

#[test]
fn a_town_short_of_hands_shares_out_the_hands_it_has() {
    // the largest town loses four fifths of its people overnight: the hands its
    // employers carry from last month are far more than it has, at any wage
    let mut on = started(&params(true, 0, 1750));
    let t = largest_town(&on.st);
    on.st.pop[t] *= 0.2;
    let mut off = on.clone();
    off.params.hands.ration = false;
    let prev_w = on.st.wage[t];
    let top = (2.0 * prev_w).min(500.0);
    let units: Vec<usize> = (0..on.st.ex_town.len()).filter(|&u| on.st.ex_town[u] == t).collect();
    sim::run_month(&mut on, 0).expect("money is conserved in the month of the cut");
    sim::run_month(&mut off, 0).expect("money is conserved without the rule");
    let have = on.params.participation * on.st.pop[t];

    // without the rule the town has more at work than it has people: the fault
    let (at_work_off, at_work_on) = (off.st.month_stats.employment[t], on.st.month_stats.employment[t]);
    assert!(at_work_off > 1.5 * have, "without the rule {at_work_off} at work of {have}");
    // with it, no more at work than there are people, and on the land and in the
    // works exactly the hands left after the carters, sinkers and navvies
    assert!(at_work_on <= have * (1.0 + 1e-9), "{at_work_on} at work of {have}");
    let supply = on.st.month_stats.supply[t];
    let got = hired(&on.st, t);
    assert!(got <= supply * (1.0 + 1e-9) && got >= 0.5 * supply, "{got} hired of {supply} to spare");
    // the wage stands at the top of its month's range
    assert!((on.st.wage[t] - top).abs() <= 1e-12 * top, "wage {} against the top {top}", on.st.wage[t]);
    assert!((off.st.wage[t] - top).abs() <= 1e-9 * top, "without the rule the search also stops at the top");

    // every land unit is cut by the same share of what it asked for (the
    // works' hands are cut alike, then laid off further when short of materials),
    // and next month it carries what it got
    let shares: Vec<f64> = units.iter().filter(|&&u| off.st.ex_Lprev[u] > 1e-6).map(|&u| on.st.ex_Lprev[u] / off.st.ex_Lprev[u]).collect();
    assert!(shares.len() > 10, "the town has land units at work");
    let s0 = shares[0];
    assert!(s0 < 0.5, "a deep cut: {s0}");
    assert!(shares.iter().all(|s| (s - s0).abs() <= 1e-9 * s0), "one share for every unit: {:?}", &shares[..5]);

    // the other towns were not short, and are as they would have been
    for u in (0..on.st.ex_town.len()).filter(|&u| on.st.ex_town[u] != t) {
        let (a, b) = (on.st.ex_Lprev[u], off.st.ex_Lprev[u]);
        assert!((a - b).abs() <= 1e-9 * b.max(1.0), "unit {u} of town {} moved: {a} against {b}", on.st.ex_town[u]);
    }
}

#[test]
fn a_town_with_no_hands_left_hires_nobody() {
    // the carters take every hand the largest town has: let them have all of
    // them (`max_share` 1) and give the town a carting bill nobody could meet
    let mut on = started(&params(true, 0, 1750));
    let t = largest_town(&on.st);
    on.params.carting.max_share = 1.0;
    on.st.cart_bill[t] = 1e12;
    on.st.carters[t] = 1e12;
    let mut off = on.clone();
    off.params.hands.ration = false;
    let top = (2.0 * on.st.wage[t]).min(500.0);
    sim::run_month(&mut on, 0).expect("money is conserved in a town that hires nobody");
    sim::run_month(&mut off, 0).expect("money is conserved without the rule");
    assert_eq!(on.st.month_stats.supply[t], 0.0, "nothing is left for hire");

    // without the rule the town's farms and works hire men who do not exist
    assert!(hired(&off.st, t) > 100.0, "without the rule {} hired of none", hired(&off.st, t));
    // with it nobody is hired, and the wage stands at the top of its range
    for u in (0..on.st.ex_town.len()).filter(|&u| on.st.ex_town[u] == t) {
        assert_eq!(on.st.ex_Lprev[u], 0.0, "unit {u} hired of nobody");
    }
    for f in (0..on.st.f_alive.len()).filter(|&f| on.st.f_alive[f] && on.st.f_town[f] == t) {
        assert_eq!(on.st.f_labor[f], 0.0, "works {f} hired of nobody");
    }
    assert!((on.st.wage[t] - top).abs() <= 1e-12 * top);
    // the carters are at work, and nobody else
    let have = on.params.participation * on.st.pop[t];
    assert!(on.st.month_stats.employment[t] <= have * (1.0 + 1e-9));
    // and the next month, with the carters as the bill makes them, goes on
    on.params.carting.max_share = game_params().carting.max_share;
    sim::run_month(&mut on, 1).expect("the month after goes on");
}

#[test]
fn brambank_hires_no_more_than_it_has() {
    // island 3 with six families: Brambank, founded on the coalfield at the end
    // of 1750, asks in January 1751 for half as many hands again as it has
    // (docs/rules/mines.md, "When a working hires more hands than its town has").
    // That is island 3's history under the game's rules as they stood before the
    // works' two mends of the pass (works.md, parts G and H), which change it from
    // the warm-up on: the case is played with works.spread_first and
    // works.keep_order off, and every other rule as the game has it.
    for ration in [false, true] {
        let mut P = params(ration, 6, 1751);
        P.works.spread_first = false;
        P.works.keep_order = false;
        let mut s = started(&P);
        sim::run_year(&mut s).expect("money is conserved through 1750");
        let mut worst = (0.0f64, String::new());
        for month in 0..3 {
            sim::run_month(&mut s, month).expect("money is conserved");
            for t in (0..s.st.T_max).filter(|&t| s.st.town_active[t] && s.st.pop[t] > 0.0) {
                let r = s.st.month_stats.employment[t] / (s.params.participation * s.st.pop[t]);
                if r > worst.0 {
                    worst = (r, format!("{} in month {}", s.st.town_name[t], month + 1));
                }
            }
        }
        if ration {
            assert!(worst.0 <= 1.0 + 1e-9, "with the rule {:.3} times its people at work ({})", worst.0, worst.1);
        } else {
            assert!(worst.0 > 1.3 && worst.1.starts_with("Brambank"), "without the rule the fault is there: {:.3} ({})", worst.0, worst.1);
        }
    }
}

#[test]
fn rationed_runs_conserve_money_and_are_deterministic() {
    let P = params(true, 6, 1755);
    let run = || sim::run_with(&P, game_content(), false, true).expect("money is conserved every month");
    let (a, b) = (run(), run());
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    for (k, name) in st.series.names.iter().enumerate() {
        let (x, y) = (&st.series.data[k], &b.state.series.data[k]);
        assert!(x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()), "series {name} differs between identical runs");
    }
}
