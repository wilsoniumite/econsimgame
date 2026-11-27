//! What is carried out is asked (docs/rules/prices.md, "What is carried out is
//! asked"; not in the Python). Under `prices.exports_ask` the flow term of a
//! town's price rule counts what merchants carried out of the town in the month
//! among what is asked of its market. The switch is off under both sets of rules
//! until carriage is paid in hands; with it off, tests/golden.rs and
//! tests/golden_game.rs hold that nothing moved. These tests hold that it is off
//! and unwritten, and that on it moves only the prices of what was carried out,
//! and those up. Played under the game's rules on island 7 with six families,
//! the switch set by name in each case.
#![allow(non_snake_case)]

use isostate_core::*;

#[test]
fn off_by_default_and_in_the_game_and_not_written_while_off() {
    assert!(!default_params().prices.exports_ask);
    assert!(!game_params().prices.exports_ask);
    for P in [default_params(), game_params()] {
        let j = serde_json::to_string(&P).unwrap();
        assert!(!j.contains("exports_ask"), "the parameters name the switch while it is off");
    }
    // on, it is written, and read back as on
    let mut P = game_params();
    P.prices.exports_ask = true;
    let j = serde_json::to_string(&P).unwrap();
    assert!(j.contains("\"exports_ask\":true"));
    let back: Params = serde_json::from_str(&j).unwrap();
    assert!(back.prices.exports_ask);
}

#[test]
fn what_is_asked_is_the_own_buyers_and_on_what_was_carried_out() {
    let mut p = prices::PriceParams::game();
    assert_eq!(prices::asked(&p, 3.0, 5.0), 3.0);
    p.exports_ask = true;
    assert_eq!(prices::asked(&p, 3.0, 5.0), 8.0);
    assert_eq!(prices::asked(&p, 3.0, 0.0), 3.0);
}

/// One month of an island of 1756 stepped twice from the same state, the switch
/// off and on: everything the month does is the same but the prices it sets for
/// next month, and those only where merchants carried goods out, upwards.
#[test]
fn on_a_month_moves_only_the_prices_of_what_was_carried_out_and_those_up() {
    let mut P = game_params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(1755);
    P.prices.exports_ask = false;
    let r = sim::run_with(&P, game_content(), false, true).expect("the run finishes");
    let (st0, C) = (r.state, r.content);
    let mut off = st0.clone();
    off.watch_merchants = true;
    econ::step_month(&mut off, &C, &P);
    let mut Pon = P.clone();
    Pon.prices.exports_ask = true;
    let mut on = st0.clone();
    on.watch_merchants = true;
    econ::step_month(&mut on, &C, &Pon);

    // the month itself is the same: stocks, wages, the merchants' takings
    assert_eq!(off.inv, on.inv, "the month's stocks moved with the switch");
    assert_eq!(off.wage, on.wage, "the month's wages moved with the switch");
    assert_eq!(off.month_merchants, on.month_merchants, "the merchants' month moved with the switch");

    // what the merchants carried out, town by town, in money; the prices are
    // the anchors' (prices.enabled): the anchor is what the flow term moves
    assert!(P.prices.enabled);
    let sent = econ::MERCHANT_TERMS.iter().position(|&c| c == "sent").unwrap();
    let (mut up, mut same) = (0, 0);
    for t in 0..off.T_max {
        if !st0.town_active[t] {
            continue;
        }
        for g in 0..C.G {
            let (a, b) = (off.price_anchor[(t, g)], on.price_anchor[(t, g)]);
            assert!(b >= a, "town {t}, good {g}: the anchor fell with the switch on ({a} to {b})");
            if b > a {
                up += 1;
                assert!(on.month_merchants[(t, sent)] > 0.0, "town {t} carried nothing out, and its anchor of good {g} rose");
            } else {
                same += 1;
            }
        }
    }
    assert!(up > 0, "no anchor rose: nothing carried out was asked");
    assert!(same > 0, "every anchor rose");
}
