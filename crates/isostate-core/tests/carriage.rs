//! Carriage in hands (docs/rules/carriage.md; not in the Python). Under
//! `carriage.in_hands` a load takes `carriage.hands` hands over a unit of the
//! path's cost and they are paid a wage, the carting town's or the island's
//! (`carriage.wage`); the railway's station charge needs no year. Beside it,
//! `prices.spoils_unkept`: a good that spoils is not stocked. Both are off
//! under both sets of rules; with them off, tests/golden.rs and
//! tests/golden_game.rs hold that nothing moved. These tests hold that they are
//! off and unwritten, what each reckons, and that an island with both on and
//! `prices.exports_ask` beside them runs. The runs are island 7 under the
//! game's rules with six families, each switch set by name.
#![allow(non_snake_case)]

use isostate_core::*;

#[test]
fn off_by_default_and_in_the_game_and_not_written_while_off() {
    for P in [default_params(), game_params()] {
        assert!(!P.carriage.in_hands && !P.prices.spoils_unkept);
        let j = serde_json::to_string(&P).unwrap();
        assert!(!j.contains("carriage") && !j.contains("spoils_unkept"), "the parameters name a switch while it is off");
    }
    // on, both are written and read back, and the wage is said in a word
    let mut P = game_params();
    P.carriage.in_hands = true;
    P.carriage.wage = carriage::Wage::Island;
    P.prices.spoils_unkept = true;
    let j = serde_json::to_string(&P).unwrap();
    assert!(j.contains("\"in_hands\":true") && j.contains("\"wage\":\"island\"") && j.contains("\"spoils_unkept\":true"), "{j}");
    let back: Params = serde_json::from_str(&j).unwrap();
    assert_eq!(back.carriage, P.carriage);
    assert!(back.prices.spoils_unkept);
    // the sweep's --set takes the word
    let Q = sweep::set_field(&game_params(), "carriage.wage", &sweep::parse_value("island")).unwrap();
    assert_eq!(Q.carriage.wage, carriage::Wage::Island);
    assert!(sweep::set_field(&game_params(), "carriage.wage", &sweep::parse_value("county")).is_err());
}

#[test]
fn the_hands_a_load_takes_are_the_ports_freight_at_the_settled_wage_of_1750() {
    assert!((carriage::HANDS * 4.49 - 0.015).abs() < 1e-15);
    assert_eq!(game_params().carriage.hands, carriage::HANDS);
}

#[test]
fn the_station_charge_needs_no_year_in_hands() {
    let mut P = game_params();
    let mut st = state::State::default();
    haulage::year(&mut st, 1800);
    assert!(!haulage::on(&st, &P), "off, the charge waits for 1901");
    P.carriage.in_hands = true;
    assert!(haulage::on(&st, &P), "in hands, the charge is laid from the first year");
    P.haulage.handling = false;
    assert!(!haulage::on(&st, &P), "the station's own switch still rules");
}

#[test]
fn a_good_that_spoils_is_not_stocked_under_its_switch() {
    let (C, mut P) = (game_content(), game_params());
    let (services, grain) = (C.gi("services"), C.gi("grain"));
    let off = tree::month_loss(&C, &P);
    assert!(off[services] > prices::SPOILS && off[services] < 1.0);
    // a stock of services against its target: the shelves are read while kept
    let kept = tree::price_gap(&C, &P, services, 1.0, 1.0, 9.0, 1.0);
    assert!((kept - 0.5).abs() < 1e-12, "{kept}");
    P.prices.spoils_unkept = true;
    let on = tree::month_loss(&C, &P);
    assert_eq!(on[services], 1.0, "what is not sold of a service in the month is lost");
    assert_eq!(on[grain], off[grain]);
    // building work and town gas are the overlay's: building loses a tenth a month and is kept, gas more and is not
    assert!(!prices::unkept(&P.prices, 0.1) && prices::unkept(&P.prices, 0.15));
    // not kept, the month's asking against its making, and the shelves are not read
    assert_eq!(tree::price_gap(&C, &P, services, 1.0, 1.0, 9.0, 1.0), 8.0 / 9.0);
    assert_eq!(tree::price_gap(&C, &P, services, 50.0, 1.0, 3.0, 3.0), 0.0);
    // grain still reads its stock
    assert_eq!(tree::price_gap(&C, &P, grain, 1.0, 1.0, 9.0, 1.0), tree::price_gap(&C, &game_params(), grain, 1.0, 1.0, 9.0, 1.0));
}

/// Island 7 run to 1754 from a start with the three switches on: it runs, the
/// island's freight is its hands at the island's wage, and a town's is its
/// hands at its own wage of the month.
fn run_on(start: &str, wage: carriage::Wage) -> sim::RunResult {
    let mut P = game_params();
    landfall::set_start(&mut P, start).unwrap();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(1754);
    P.carriage.in_hands = true;
    P.carriage.wage = wage;
    P.prices.exports_ask = true;
    P.prices.spoils_unkept = true;
    sim::run_with(&P, game_content(), false, true).expect("the run finishes")
}

#[test]
fn in_hands_the_freight_is_the_hands_at_a_wage_on_both_starts() {
    for (start, first) in [("settled", (0.012, 0.018)), ("landfall", (0.027, 0.040))] {
        let r = run_on(start, carriage::Wage::Town);
        let (st, P) = (&r.state, &r.params);
        let w = carriage::island_wage(st);
        assert!(w > 0.0);
        assert_eq!(longrun::freight(st, P), P.carriage.hands * w);
        // the first year carts at the 1750 wage: about the port's coin on the settled island, twice it at landfall
        let w0 = st.longrun.wage0 * P.carriage.hands;
        assert!(w0 > first.0 && w0 < first.1, "{start}: the first year's freight is {w0}");
        let towns: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t] && st.wage[t] > 0.0).collect();
        assert!(!towns.is_empty());
        for &t in &towns {
            assert_eq!(carriage::freight_at(st, P, t), P.carriage.hands * st.wage[t], "{start}, town {t}");
        }
        // with the island's wage every town carts at the island's freight
        let mut Q = P.clone();
        Q.carriage.wage = carriage::Wage::Island;
        for &t in &towns {
            assert_eq!(carriage::freight_at(st, &Q, t), longrun::freight(st, &Q));
        }
        // off, it is the old freight, to the bit
        let mut O = P.clone();
        O.carriage.in_hands = false;
        for &t in &towns {
            assert_eq!(carriage::freight_at(st, &O, t), longrun::freight(st, &O));
        }
    }
}
