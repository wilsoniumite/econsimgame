//! Care (not in the Python; docs/rules/care.md): a need for work of presence in
//! the towns, served by one good made by one method, which no machine of the
//! tasks' rule takes, which is never kept, and which the island finds about
//! 1911.
//!
//! The island runs are played on island 7 to 1935 under the game's rules with
//! six families, with the switches their cases lean on set by name
//! (`care.enabled`, `engel.enabled`, `engel.lean`, `ages.watch`, `accounts.enabled`,
//! and `tasks.tending` off) and every other rule as the game has it. What they
//! check holds on any island's history (one run against another, books that add
//! up, a stock that is never more than a month's making), but for one thing:
//! that island 7 finds care and buys it by 1935, which the tests say if a later
//! rule stops it.
#![allow(non_snake_case)]

use isostate_core::accounts::{GOODS_FIELDS, KG, KM, METHOD_FIELDS};
use isostate_core::care::{self, CareParams};
use isostate_core::content::Begins;
use isostate_core::tasks;
use isostate_core::*;

const TO: i64 = 1935;

fn params(care_on: bool) -> Params {
    let mut P = game_params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(TO);
    P.engel.enabled = true;
    P.engel.lean = true;
    P.ages.watch = true;
    P.accounts.enabled = true;
    P.tasks.tending = false;
    P.care = CareParams { enabled: care_on, ..CareParams::game() };
    P
}

/// The game's content as a run under `P` plays it (with the tree laid).
fn content(P: &Params) -> Content {
    isostate_core::tree::content(P, game_content()).expect("the game's tree lays")
}

fn run(P: &Params) -> sim::RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(P, Rules::Game.content(), false, true).expect("money is conserved every month")
}

/// Off under the port's rules and left out of its file, on under the game's. The
/// port's content has no care. The game's has a ninth need, care, with no share
/// in any set the content gives (so nothing moves before the rule gives it one),
/// one good that serves it, not carried and not kept, beginning with its method;
/// and the method, every hour of it presence, the class no machine may enter,
/// learning nothing by doing, found from 1908.
#[test]
fn care_stands_behind_its_switch() {
    let port = default_params();
    assert_eq!(port.care, CareParams::default());
    assert!(!port.care.enabled);
    assert!(game_params().care.enabled, "care is on under the game's rules");
    assert!(!serde_json::to_string(&port).unwrap().contains("care"), "the port's file says nothing of care");
    assert!(!care::present(&default_content()), "the port's content has no care");

    let P = params(true);
    let C = content(&P);
    assert!(care::present(&C));
    let (s, g, m) = (care::need(&C).unwrap(), care::good(&C).unwrap(), care::method(&C).unwrap());
    assert_eq!(s, C.S - 1, "care is the last need, after the eight it was added to");
    for (what, set) in [("subsistence", &C.subsistence), ("shares", &C.shares), ("shares_rich", &C.shares_rich), ("shares_lean", &C.shares_lean), ("lord_shares", &C.lord_shares)] {
        assert_eq!(set[s], 0.0, "care has no share in {what}");
    }
    for k in 0..C.G {
        assert_eq!(C.props[(k, s)] > 0.0, k == g, "only care serves the need of care");
    }
    assert_eq!(C.props[(g, s)], 1.0);
    assert!(g >= C.G_base && m >= C.M_base, "care and its method come after the goods and methods of the start");
    assert!(!C.traded[g] && C.stock_months[g] == Some(0.0) && C.opens_dear[g]);
    let t = content::tech_index(&C, care::METHOD);
    assert_eq!(C.good_begins[g], Begins::Idea(t), "care begins with its method");
    assert!(care::is_idea(&C, t));
    assert_eq!(C.tech_after[t], 1908);
    assert_eq!(C.m_b[m], 0.0, "care learns nothing by doing");
    assert_eq!(C.method_out[m], g);
    let classes = C.tasks.method[m];
    assert_eq!(classes[tasks::PRESENCE], 1.0, "every hour of care is presence");
    assert!(C.tasks.closed[tasks::PRESENCE], "and presence is closed to machines");
    assert!(classes[..tasks::PRESENCE].iter().all(|&v| v == 0.0));
}

/// The want gives care room in the richer set: the other needs give it in
/// proportion, the set still adds up to what it did, and no share falls below
/// nothing; with no want the set is what it was, to the bit.
#[test]
fn the_want_makes_room_in_proportion() {
    let C = content(&params(true));
    let s = care::need(&C).unwrap();
    let rich = C.shares_lean.clone();
    let sum = |v: &[f64]| v.iter().sum::<f64>();
    for w in [0.0, 0.01, 0.1, 0.3, 0.5, 1.0] {
        let row = care::with_care(&rich, s, w);
        assert!((sum(&row) - sum(&rich)).abs() < 1e-12, "adds up at a want of {w}");
        assert!(row.iter().all(|&v| v >= 0.0));
        assert!((row[s] - w).abs() < 1e-15);
        for k in (0..rich.len()).filter(|&k| k != s) {
            assert!((row[k] - rich[k] * (1.0 - w)).abs() < 1e-15);
        }
    }
    let same = care::with_care(&rich, s, 0.0);
    assert!(same.iter().zip(&rich).all(|(a, b)| a.to_bits() == b.to_bits()));
}

/// Care on against care off, island 7 to 1935: the two runs are the same island
/// to the end of the year before care is found (it is sought with draws of its own and
/// nothing is offered for it before a town knows it), every month of every
/// series to the bit; then care is bought, never kept beyond a month, and its
/// books balance: what was made is what households bought and what was lost at a
/// month's end, nobody else buys it, and the works' value is what they paid out
/// in wages, materials, fuel, carting and rent and kept as profit.
#[test]
fn nothing_moves_before_care_and_its_books_balance() {
    let (on, off) = std::thread::scope(|sc| {
        let a = sc.spawn(|| run(&params(true)));
        let b = sc.spawn(|| run(&params(false)));
        (a.join().unwrap(), b.join().unwrap())
    });
    let C = &on.content;
    let (s, g, m) = (care::need(C).unwrap(), care::good(C).unwrap(), care::method(C).unwrap());
    let t = content::tech_index(C, care::METHOD);
    let found = on.state.tech_year[t];
    assert!(found >= 1908 && found <= TO - 5, "island 7 finds care between 1908 and {} (found: {found})", TO - 5);
    assert!(off.state.tech_year[t] < 0, "with the rule off care is never found");

    // the same island to the end of the year it is found, every series to the bit
    let months = ((found - C.start_year) * 12) as usize;
    let (a, b) = (&on.state.series, &off.state.series);
    assert_eq!(a.names, b.names, "the same series, the rule on or off");
    for (k, name) in a.names.iter().enumerate() {
        for i in 0..months {
            let (x, y) = (a.data[k][i], b.data[k][i]);
            assert!(x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan()), "{name} parts in month {i} ({} against {})", x, y);
        }
    }
    // and after it the island is another
    assert!(a.names.iter().enumerate().any(|(k, _)| (months..a.data[k].len()).any(|i| a.data[k][i].to_bits() != b.data[k][i].to_bits())));

    // bought, by households alone, and never kept beyond a month's making
    let st = &on.state;
    let last = st.care.years.last().unwrap();
    assert_eq!(last.year, TO);
    assert!(last.hands > 0.005 && last.spend > 0.005, "care is made and bought by {TO}: {} of hands, {} of spending", last.hands, last.spend);
    assert!(st.care.years.iter().all(|y| y.kept <= 1.0 + 1e-9), "never more than December's making in stock at a year's end");
    let (T, G, M, S) = (st.T_max, C.G, C.M, C.S);
    let col = |name: &str| GOODS_FIELDS.iter().position(|f| *f == name).unwrap();
    let mcol = |name: &str| METHOD_FIELDS.iter().position(|f| *f == name).unwrap();
    let mut seen = 0;
    for k in 0..st.acct.years.len() {
        let rec = st.acct.year(k).unwrap();
        let year = st.acct.years[k].year;
        let (mut made, mut hh, mut lost, mut value, mut paid, mut need) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for i in 0..T {
            let v = |name: &str| rec.goods[(i * G + g) * KG + col(name)] as f64;
            made += v("made");
            hh += v("households");
            lost += v("spoiled");
            for other in ["landlords", "families", "works_inputs", "works_fuel", "land", "building", "projects", "sent", "received"] {
                assert_eq!(v(other), 0.0, "{year} town {i}: nobody but households has care ({other})");
            }
            let mv = |name: &str| rec.methods[(i * M + m) * KM + mcol(name)] as f64;
            value += mv("value");
            paid += ["wages", "inputs", "fuel", "cart_in", "cart_out", "rent", "managers", "profit"].iter().map(|n| mv(n)).sum::<f64>();
            need += rec.needs[(i * S + s) * 2] as f64;
            assert_eq!(rec.needs[(i * S + s) * 2 + 1], 0.0, "landlords and families buy no care");
        }
        if year < found {
            assert!(made == 0.0 && need == 0.0 && value == 0.0, "{year}: nothing of care before the year it is found");
            continue;
        }
        if made > 0.0 {
            seen += 1;
            let open: f64 = (0..T).map(|i| rec.goods[(i * G + g) * KG + col("stock_open")] as f64).sum();
            let close: f64 = (0..T).map(|i| rec.goods[(i * G + g) * KG + col("stock")] as f64).sum();
            let left = open + made - hh - lost - close;
            assert!(left.abs() <= 1e-4 * (open + made), "{year}: {left} of care unplaced of {made} made");
            // (the books reckon a works' materials at the lower of its own price and the
            // town's, and its value at the town's price, so the two sides are not one
            // identity to the penny; the sums that are, every month, are the money's)
            let wages: f64 = (0..T).map(|i| rec.methods[(i * M + m) * KM + mcol("wages")] as f64).sum();
            eprintln!("{year}: care made {made:.0}, bought {hh:.0}, lost {lost:.0}; value {value:.0}, paid out and kept {paid:.0}, wages {wages:.0}; households' need {need:.0}");
            assert!((value - paid).abs() <= 0.1 * value, "{year}: the care works' value {value} against {paid} paid out and kept");
            if year >= found + 10 {
                assert!(wages >= 0.5 * value && wages <= value, "{year}: care's hands are paid {wages} of the {value} it sells for");
            }
            assert!(need > 0.0, "{year}: households spent on the need of care");
        }
    }
    assert!(seen >= 3, "care's books were read in {seen} years");
}
