//! Pace (docs/rules/pace.md): works that ask each month for a twelfth of the
//! plant still to come, not a quarter, so that it comes in over a year on the
//! mean. Off under the port's rules, on under the game's from 1901: the cure of
//! the timber-and-brick cycle (docs/probes/tr-timber.md). The port's golden
//! test holds that nothing reaches the port.
#![allow(non_snake_case)]

use isostate_core::pace::{self, PaceParams, PORT_SHARE};
use isostate_core::sim;
use isostate_core::*;

#[test]
fn the_rule_is_off_under_the_port_and_on_under_the_game() {
    assert_eq!(default_params().pace, PaceParams::default());
    assert_eq!(game_params().pace, PaceParams::on());
    let off = PaceParams::default();
    assert!(!off.enabled && off.months == 12.0 && off.from == 1901);
    assert!(PaceParams::on().enabled && PaceParams::on().months == 12.0 && PaceParams::on().from == 1901);
    // the port's params.json does not name it; the game's does
    assert!(serde_json::to_value(default_params()).unwrap().get("pace").is_none());
    assert!(serde_json::to_value(game_params()).unwrap().get("pace").is_some());
}

#[test]
fn a_works_asks_for_a_quarter_until_the_rule_acts_and_a_twelfth_after() {
    let mut P = default_params();
    let mut st = State::default();
    st.engel.year_now = 1950;
    assert_eq!(pace::share(&st, &P), PORT_SHARE);
    P = game_params();
    st.engel.year_now = 1900;
    assert_eq!(pace::share(&st, &P), PORT_SHARE);
    st.engel.year_now = 1901;
    assert_eq!(pace::share(&st, &P), 1.0 / 12.0);
    P.pace.months = 6.0;
    assert_eq!(pace::share(&st, &P), 1.0 / 6.0);
}

/// Island 3 under the game's rules with six families, played to 1775 with the
/// rule switched on from 1765 (`from` moved early, so that a short run sees it)
/// and off: the same island to the last digit until 1765, then another, with
/// the money checked every month. Played under every rule `game_params` has
/// on today; it leans on no history but its own.
#[test]
fn on_from_a_year_the_island_parts_there_and_keeps_its_money() {
    let play = |on: bool| {
        let mut P = save::params(Rules::Game, 3, 6);
        P.end_year = Some(1775);
        P.pace = if on { PaceParams { from: 1765, ..PaceParams::on() } } else { PaceParams::default() };
        let mut s = sim::start(&P, Rules::Game.content());
        let mut years = Vec::new();
        while s.year <= s.end_year {
            sim::run_year(&mut s).expect("money is conserved every month");
            years.push((s.year - 1, s.st.f_order_K.iter().sum::<f64>(), s.st.money_now));
        }
        years
    };
    let (off, on) = (play(false), play(true));
    for (a, b) in off.iter().zip(&on) {
        if a.0 < 1765 {
            assert_eq!(a, b, "the two islands part before the rule acts, in {}", a.0);
        }
    }
    assert!(off.iter().zip(&on).any(|(a, b)| a.0 >= 1765 && a != b), "the rule changed nothing");
}
