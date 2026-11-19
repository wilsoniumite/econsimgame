//! The gauges (not in the Python; docs/rules/gauges.md): they change nothing in
//! the run they read, their reckoning adds up, and they need the books. With
//! the switch off the golden test (tests/golden.rs) covers the port.
//!
//! The runs here are played under the game's rules as they stand; nothing in
//! them leans on one island's history beyond its being an island that runs,
//! so no newer rule's switch needs to be set by name.
#![allow(non_snake_case)]

use isostate_core::gauges::{self, SERIES};
use isostate_core::*;

fn params(rules: Rules, seed: u64, families: usize, end_year: i64, gauges_on: bool) -> Params {
    let mut P = rules.params();
    P.seed = seed;
    P.n_families = families;
    P.end_year = Some(end_year);
    P.accounts.enabled = true;
    P.gauges.enabled = gauges_on;
    P
}

fn run(rules: Rules, P: &Params) -> RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(P, rules.content(), false, true).expect("money is conserved every month")
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn header(blob: &[u8]) -> serde_json::Value {
    let n = u32::from_le_bytes(blob[..4].try_into().unwrap()) as usize;
    serde_json::from_slice(&blob[4..4 + n]).unwrap()
}

#[test]
fn off_under_the_port_and_on_in_the_game() {
    assert!(!default_params().gauges.enabled && game_params().gauges.enabled);
    let r = run(Rules::Port, &{
        let mut P = default_params();
        P.end_year = Some(1752);
        P
    });
    assert!(!r.state.gauges.on && r.state.gauges.years.is_empty());
    assert!(r.state.series.names.iter().all(|n| !n.starts_with("gauge_")));
    let h = header(&record::pack(&r.state, &r.content, &r.params, &r.events, &r.rec, "x", false));
    assert!(h.get("gauges").is_none());
    assert!(!serde_json::to_string(&r.params).unwrap().contains("gauges"), "params.json of a port run does not mention the gauges");
}

#[test]
fn without_the_books_the_gauges_do_nothing() {
    let mut P = params(Rules::Game, 7, 0, 1752, true);
    P.accounts.enabled = false;
    let r = run(Rules::Game, &P);
    assert!(!r.state.gauges.on && r.state.gauges.years.is_empty());
    assert!(r.state.series.names.iter().all(|n| !n.starts_with("gauge_")));
    assert!(gauges::facts(&r).is_empty());
}

/// The name of this test is the promise the gauges make: a run with them on
/// is the same run as one with them off.
#[test]
fn runs_identical_with_the_gauges_on_and_off() {
    // two islands under the game's rules, with and without families, and one
    // under the port's with the books and gauges thrown on: every series, every
    // piece of news and the state the run ends in are bit for bit what they are
    // with the gauges off
    for (rules, seed, families, end) in [(Rules::Game, 7, 6, 1772), (Rules::Game, 5, 0, 1762), (Rules::Port, 3, 6, 1760)] {
        let off = run(rules, &params(rules, seed, families, end, false));
        let on = run(rules, &params(rules, seed, families, end, true));
        let (a, b) = (&off.state, &on.state);
        for (k, name) in a.series.names.iter().enumerate() {
            assert_eq!(bits(&a.series.data[k]), bits(b.series.get(name)), "series {name} moved, seed {seed}");
        }
        assert_eq!(b.series.names.len(), a.series.names.len() + SERIES.len(), "the gauges add their series after everyone's");
        assert_eq!(serde_json::to_string(&off.events).unwrap(), serde_json::to_string(&on.events).unwrap());
        for (what, x, y) in [
            ("people", &a.pop, &b.pop),
            ("households' cash", &a.hh_cash, &b.hh_cash),
            ("landlords' cash", &a.lord_cash, &b.lord_cash),
            ("merchants' cash", &a.merch_cash, &b.merch_cash),
            ("works' cash", &a.f_cash, &b.f_cash),
            ("works' capacity", &a.f_K, &b.f_K),
            ("prices", &a.price.data, &b.price.data),
            ("stocks", &a.inv.data, &b.inv.data),
            ("wages", &a.wage, &b.wage),
            ("land shares", &a.lam.data, &b.lam.data),
            ("forest cover", &a.cover, &b.cover),
        ] {
            assert_eq!(bits(x), bits(y), "{what} moved, seed {seed}");
        }
        let cash = |st: &State| st.families.iter().map(|f| f.cash.to_bits()).collect::<Vec<_>>();
        assert_eq!(cash(a), cash(b));
        assert_eq!((a.F_count, a.issued.to_bits(), a.money_now.to_bits()), (b.F_count, b.issued.to_bits(), b.money_now.to_bits()));
        assert_eq!(off.rec.layers, on.rec.layers);
        assert_eq!(serde_json::to_string(&a.acct.years).unwrap(), serde_json::to_string(&b.acct.years).unwrap(), "the books are the books they were");
        assert!(b.gauges.on && b.gauges.years.len() == (end - 1749) as usize);
        // and the save's fingerprint does not see them
        assert!(accounts::own_series("gauge_floor"));
    }
}

#[test]
fn the_reckoning_adds_up() {
    let r = run(Rules::Game, &params(Rules::Game, 3, 6, 1800, true));
    let st = &r.state;
    assert_eq!(st.gauges.years.len(), 51);
    for (g, books) in st.gauges.years.iter().zip(&st.acct.years) {
        assert_eq!(g.year, books.year);
        // the solve's own check: of each good's price, wages, rent and the rest through the chain are all of it
        assert!(g.chain_error < 1e-6, "{}: the chain is out by {}", g.year, g.chain_error);
        for c in g.goods.iter().filter(|c| c.wages.is_finite()) {
            let direct = c.wages + c.rent + c.rest + c.inputs.iter().sum::<f64>();
            // (the books' tables are kept in single precision)
            assert!((direct - 1.0).abs() < 1e-5, "{} {}: the direct shares add up to {direct}", g.year, c.name);
            assert!(c.wear <= c.inputs.iter().sum::<f64>() + 1e-12);
        }
        // spending seen through the chain and income seen in the books are two
        // views of one year: they differ by what went into stock, what spoiled
        // and the merchants' trade, not by more
        assert!((g.spend_work - g.work_gross).abs() < 0.06, "{}: spending {} against income {}", g.year, g.spend_work, g.work_gross);
        assert!(g.work_net >= g.work_gross && g.wear >= 0.0);
        assert!(g.floor >= 0.0 && g.floor < g.spend_work, "{}: the floor {} is under work's share of spending", g.year, g.floor);
        assert!(g.floor >= g.presence_today - 1e-12, "the floor is where prices have fallen");
        assert!((g.spend_work + g.spend_rent + g.spend_rest - 1.0).abs() < 1e-6);
        let fw: f64 = g.families_works.iter().sum();
        let fl: f64 = g.families_land.iter().sum();
        assert!((0.0..=1.0 + 1e-9).contains(&fw) && (0.0..=1.0 + 1e-9).contains(&fl));
        // (a town's own share can stand outside nothing and one in a year its land's rent or a works' profit is below nothing)
        assert!(!g.towns.is_empty() && g.towns.iter().filter(|t| t.work_gross > 0.0 && t.work_gross < 1.0).count() * 2 > g.towns.len());
    }
    // the first year is the chain's first link
    let y0 = &st.gauges.years[0];
    assert_eq!((y0.price_made, y0.price_raw, y0.price_all, y0.wage_index), (1.0, 1.0, 1.0, 1.0));
    // the hand methods of the first year: the bloomery for iron, the hand loom for cloth
    let hand = gauges::hand_methods(&r.content);
    let (iron, cloth) = (r.content.gi("iron"), r.content.gi("cloth"));
    assert_eq!(r.content.method_name[hand[iron].unwrap()], "bloomery");
    assert_eq!(r.content.method_name[hand[cloth].unwrap()], "hand_loom");
    assert!(hand[r.content.gi("grain")].is_none(), "the land's goods have none");
}

#[test]
fn the_gauges_go_into_the_run_file_the_facts_and_the_live_view() {
    let r = run(Rules::Game, &params(Rules::Game, 7, 6, 1760, true));
    let h = header(&record::pack(&r.state, &r.content, &r.params, &r.events, &r.rec, "x", false));
    assert_eq!(h["gauges"]["years"].as_array().unwrap().len(), 11);
    let facts = gauges::facts(&r);
    assert!(facts.iter().any(|(n, _, v)| n == "gauge_floor_1760" && v.is_finite()));
    assert!(facts.iter().any(|(n, _, _)| n == "gauge_labour_in_machinery_1775_1900"));
    // each series holds the year's figure in all twelve of its months
    let s = r.state.series.get("gauge_spend_work");
    assert!(s.iter().all(|v| v.is_finite()));
    assert!(s[..12].iter().all(|&v| v == s[0]));
    assert_eq!(gauges::year_series(&r.state, 1760).len(), SERIES.len());
    assert_eq!(gauges::gauge_year(&r.state).unwrap().year, 1760);
}
