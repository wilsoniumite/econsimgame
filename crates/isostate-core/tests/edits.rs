//! Edits by hand (not in the Python; docs/designs/sandbox.md, section 6.1).
//! An edit is one more order in a game's log, carried out between two months:
//! a session with edits saved and loaded comes back the same to the last digit;
//! money made by hand is booked where the money check reads it; a failed
//! harvest is that year's harvest and draws nothing; a refused edit leaves no
//! trace; and an edit is written in the log as the design says.
//!
//! Every case is played on island 7 under the game's rules with six families.
//! They compare a game with itself (saved and loaded, or copied and edited), so
//! they do not lean on the island's history beyond the rules each one names:
//! (c) needs the harvest rule (`harvest.enabled`), and asserts that it is on.
#![allow(non_snake_case)]

use isostate_core::econ;
use isostate_core::edits::{Edit, Purse};
use isostate_core::params::Rules;
use isostate_core::save::{self, Game, Given, Order, Save};

fn series_bits(g: &Game) -> Vec<Vec<u64>> {
    g.sim.st.series.data.iter().map(|d| d.iter().map(|x| x.to_bits()).collect()).collect()
}

/// A game on island 7 played `years` years from its start, at the turn.
fn played(years: usize) -> Game {
    let mut g = Game::new(7, 6, 1_000);
    for _ in 0..years {
        g.year().unwrap();
        g.turn();
    }
    g
}

/// The towns that stand, the most people first.
fn biggest_towns(g: &Game) -> Vec<usize> {
    let st = &g.sim.st;
    let mut towns: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    towns.sort_by(|a, b| st.pop[*b].total_cmp(&st.pop[*a]));
    towns
}

fn by_hand(edit: Edit) -> Order {
    Order::ByHand { edit }
}

/// The money the island holds against what it began with, what was issued
/// and what the hand made, as a share of the money.
fn drift(g: &Game) -> f64 {
    let st = &g.sim.st;
    let now = econ::money_total(st);
    (now - (st.money_start + st.issued + st.by_hand.money)).abs() / now.max(1.0)
}

#[test]
fn a_session_with_edits_saved_and_loaded_is_the_same_to_the_last_digit() {
    // (a) eight years, a Money edit at a turn, a People and a Harvest five
    // months into the year, two years more; saved to text and loaded
    let mut a = played(8);
    assert_eq!(a.last_year(), 1757);
    let towns = biggest_towns(&a);
    let (big, second) = (towns[0], towns[1]);
    a.give(by_hand(Edit::Money { purse: Purse::Landlords, town: Some(big), factor: 2.0 })).unwrap();
    // the line returned is the title of an item of news of kind "hand"
    let last = a.sim.events.last().unwrap();
    assert_eq!(last.kind, "hand");
    assert_eq!(last.weight, 0.6);
    assert!(last.title.starts_with("By hand: landlords' money in "), "{}", last.title);
    assert_eq!(last.town, Some(big));
    for _ in 0..5 {
        assert!(!a.month().unwrap());
    }
    let people_before = a.sim.st.pop[second];
    assert_eq!(a.give(by_hand(Edit::People { town: second, factor: 0.5 })).unwrap(), save::Done::Done);
    assert_eq!(a.sim.st.pop[second], people_before * 0.5);
    a.give(by_hand(Edit::Harvest { town: Some(big), factor: 0.7 })).unwrap();
    // given in mid-year, they are carried out at once and dated with the month
    assert!(a.waiting.is_empty());
    let dated: Vec<(i64, usize)> = a.orders.iter().map(|g| (g.year, g.month)).collect();
    assert_eq!(dated, vec![(1757, 0), (1757, 5), (1757, 5)]);
    assert_eq!(a.sim.st.by_hand.count, 3);
    assert!((a.sim.st.by_hand.people + people_before * 0.5).abs() < 1e-9 * people_before);
    let news: Vec<&str> = a.sim.events.iter().filter(|e| e.kind == "hand").map(|e| e.title.as_str()).collect();
    assert_eq!(news.len(), 3, "{news:?}");
    eprintln!("the hand's news: {news:?}");
    // the rest of 1758, and two years more
    for _ in 0..3 {
        a.year().unwrap();
        a.turn();
    }
    assert_eq!(a.last_year(), 1760);
    assert!(drift(&a) < 1e-6);

    let text = a.save(2_000).to_text();
    assert!(text.contains(r#""kind":"by_hand","edit":{"what":"people""#), "{text}");
    let b = save::load(&Save::from_text(&text).unwrap(), |_| {}).unwrap();
    assert!(!b.drifted, "a session with edits loads without drift");
    assert!(b.refused.is_empty(), "{:?}", b.refused);
    let mut b = b.game;
    assert_eq!(save::check(&b.sim), save::check(&a.sim));
    assert_eq!(series_bits(&b), series_bits(&a));
    assert_eq!(b.orders, a.orders);
    assert_eq!(b.sim.st.by_hand, a.sim.st.by_hand);
    // and they go on the same
    for g in [&mut a, &mut b] {
        for _ in 0..2 {
            g.year().unwrap();
            g.turn();
        }
    }
    assert_eq!(save::check(&b.sim), save::check(&a.sim));
    assert_eq!(series_bits(&b), series_bits(&a));
}

#[test]
fn money_by_hand_is_booked_where_the_money_check_reads_it() {
    // (b) every household's purse doubled
    let mut g = played(4);
    let before = econ::money_total(&g.sim.st);
    let households: f64 = g.sim.st.hh_cash.iter().filter(|&&c| c > 0.0).sum();
    for _ in 0..3 {
        g.month().unwrap();
    }
    let before_mid = econ::money_total(&g.sim.st);
    let households_mid: f64 = g.sim.st.hh_cash.iter().filter(|&&c| c > 0.0).sum();
    g.give(by_hand(Edit::Money { purse: Purse::Households, town: None, factor: 2.0 })).unwrap();
    let after = econ::money_total(&g.sim.st);
    let made = g.sim.st.by_hand.money;
    assert!(((after - before_mid) - made).abs() <= 1e-9 * after, "the money rose by {} and the ledger says {made}", after - before_mid);
    assert!((made - households_mid).abs() <= 1e-9 * after, "doubling the households' money made as much again as they held");
    eprintln!("the island's money {before:.0} at the turn, {before_mid:.0} in March, {after:.0} after; households held {households:.0}, then {households_mid:.0}; made by hand {made:.0}");
    // two years pass the money check (`Game::month` stops a run that fails it)
    for _ in 0..2 {
        g.year().unwrap();
        g.turn();
    }
    assert!(drift(&g) < 1e-6);
    assert_eq!(g.sim.st.by_hand.money, made);
}

#[test]
fn a_failed_harvest_by_hand_is_that_years_harvest_and_draws_nothing() {
    // (c) two games alike to 1790, one given a failure at the turn
    let mut plain = played(41);
    assert_eq!(plain.last_year(), 1790);
    assert!(plain.sim.params.harvest.enabled, "this case is played under the harvest rule");
    let mut edited = plain.clone();
    let line_before = edited.sim.st.harvest_now;
    edited.give(by_hand(Edit::Harvest { town: None, factor: 0.7 })).unwrap();
    assert!((edited.sim.st.harvest_now / line_before - 0.7).abs() < 1e-12);
    for g in [&mut plain, &mut edited] {
        g.year().unwrap();
        g.turn();
    }
    let (p, e) = (plain.sim.st.harvest_year, edited.sim.st.harvest_year);
    eprintln!("1791's harvest: {p:.6} untouched, {e:.6} with a failure by hand");
    assert!((e / p - 0.7).abs() < 1e-6, "{e} against {p}");
    let news: Vec<_> = edited.sim.events.iter().filter(|ev| ev.kind == "harvest" && ev.year >= 1791.0 && ev.year < 1792.0).collect();
    assert!(!news.is_empty(), "a failure by hand is the year's news");
    eprintln!("its news: {}", news[0].title);
    // the weather drawn for 1792 is the weather there would have been
    let bits = |g: &Game| g.sim.st.harvest_mult.data.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&plain), bits(&edited));
}

#[test]
fn a_refused_edit_leaves_no_trace() {
    // (d) each refusal of section 2.7 of the design, at a turn and in mid-year
    let mut g = played(2);
    let st = &g.sim.st;
    let town = biggest_towns(&g)[0];
    let gone = (0..st.T_max).find(|&t| !st.town_active[t]).unwrap_or(st.T_max);
    let refused = [
        Edit::Money { purse: Purse::Households, town: Some(town), factor: f64::NAN },
        Edit::Money { purse: Purse::Merchants, town: None, factor: f64::INFINITY },
        Edit::Money { purse: Purse::Landlords, town: Some(town), factor: 0.05 },
        Edit::Money { purse: Purse::Landlords, town: Some(town), factor: 11.0 },
        Edit::Money { purse: Purse::Households, town: Some(gone), factor: 2.0 },
        Edit::Money { purse: Purse::Households, town: Some(st.T_max + 3), factor: 2.0 },
        Edit::People { town, factor: 0.09 },
        Edit::People { town, factor: 10.5 },
        Edit::People { town: gone, factor: 2.0 },
        Edit::People { town, factor: -1.0 },
        Edit::Harvest { town: None, factor: 0.1 },
        Edit::Harvest { town: Some(town), factor: 2.5 },
        Edit::Harvest { town: Some(gone), factor: 0.9 },
        Edit::Harvest { town: None, factor: f64::NAN },
    ];
    for at in ["the turn", "March"] {
        if at == "March" {
            for _ in 0..3 {
                g.month().unwrap();
            }
        }
        for edit in &refused {
            let (check, ledger, log, news) = (save::check(&g.sim), g.sim.st.by_hand.clone(), g.orders.clone(), g.sim.events.len());
            let why = g.give(by_hand(edit.clone())).expect_err(&format!("{edit:?} at {at}"));
            eprintln!("{edit:?} at {at}: {why}");
            assert_eq!(save::check(&g.sim), check);
            assert_eq!(g.sim.st.by_hand, ledger);
            assert_eq!(g.orders, log);
            assert_eq!(g.sim.events.len(), news);
        }
    }
    // a harvest under rules with no weather
    let mut port = Game::under(Rules::Port, 7, 6, 0);
    let check = save::check(&port.sim);
    let why = port.give(by_hand(Edit::Harvest { town: None, factor: 0.9 })).unwrap_err();
    eprintln!("under the port's rules: {why}");
    assert!(why.contains("no weather"));
    assert_eq!(save::check(&port.sim), check);
    assert!(port.orders.is_empty());
}

#[test]
fn an_edit_is_written_in_the_log_as_the_design_says() {
    // (e) the line of section 2.1, and back
    let given = Given { year: 1820, month: 3, order: by_hand(Edit::People { town: 4, factor: 0.5 }) };
    let line = serde_json::to_string(&given).unwrap();
    assert_eq!(line, r#"{"year":1820,"month":3,"kind":"by_hand","edit":{"what":"people","town":4,"factor":0.5}}"#);
    assert_eq!(serde_json::from_str::<Given>(&line).unwrap(), given);
    for edit in [
        Edit::Money { purse: Purse::Households, town: None, factor: 2.0 },
        Edit::Money { purse: Purse::Merchants, town: Some(3), factor: 0.9 },
        Edit::Harvest { town: None, factor: 0.7 },
        Edit::Harvest { town: Some(11), factor: 1.1 },
    ] {
        let given = Given { year: 1800, month: 0, order: by_hand(edit) };
        let line = serde_json::to_string(&given).unwrap();
        eprintln!("{line}");
        assert_eq!(serde_json::from_str::<Given>(&line).unwrap(), given);
    }
    assert_eq!(
        serde_json::to_string(&Edit::Money { purse: Purse::Households, town: None, factor: 2.0 }).unwrap(),
        r#"{"what":"money","purse":"households","factor":2.0}"#
    );
    // an edit never waits and is never word to the family
    assert!(!by_hand(Edit::Harvest { town: None, factor: 0.7 }).for_year_end());
}
