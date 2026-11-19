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
use isostate_core::edits::{self, Dated, Edit, Purse};
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

// ---------------------------------------------------------------- stage 3 (section 6.3)

/// The first idea neither known at the start nor found yet.
fn unfound(g: &Game) -> String {
    let C = &g.sim.content;
    let t = (0..C.NT).find(|&t| !C.tech_known[t] && g.sim.st.tech_year[t] < 0).expect("an idea still to be found");
    C.tech_name[t].clone()
}

/// A square to found a town on: on land, in a market area, not a town's
/// market place, and as far from every town as such a square is on plain
/// ground, so that its district is worth the name.
fn open_ground(g: &Game) -> usize {
    let st = &g.sim.st;
    let W = st.world.W;
    let nodes: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| st.town_node[t]).collect();
    let apart = |a: usize, b: usize| ((a % W) as f64 - (b % W) as f64).hypot((a / W) as f64 - (b / W) as f64);
    (0..st.N)
        .filter(|&c| st.world.land[c] && !st.world.coast[c] && st.world.terrain[c] < 3 && st.catch[c] >= 0 && !nodes.contains(&c))
        .max_by(|&a, &b| {
            let near = |c: usize| nodes.iter().map(|&n| apart(c, n)).fold(f64::INFINITY, f64::min);
            near(a).total_cmp(&near(b))
        })
        .expect("open ground")
}

#[test]
fn the_new_edits_saved_and_loaded_are_the_same_to_the_last_digit() {
    // as (a): an idea found at a turn, a rule set and a town ordered five
    // months into the year, played on, saved to text and loaded. Played under
    // the game's rules, where the founding rule is people.enabled
    let mut a = played(8);
    assert!(a.sim.params.people.enabled, "a town is founded by hand under the game's rule of people");
    let town = biggest_towns(&a)[0];
    let tech = unfound(&a);
    a.give(by_hand(Edit::Discover { tech: tech.clone(), town })).unwrap();
    for _ in 0..5 {
        a.month().unwrap();
    }
    a.give(by_hand(Edit::Rule { name: "money.growth".into(), value: serde_json::json!(0.03) })).unwrap();
    let site = open_ground(&a);
    a.give(by_hand(Edit::FoundTown { cell: site })).unwrap();
    let news: Vec<&str> = a.sim.events.iter().filter(|e| e.kind == "hand").map(|e| e.title.as_str()).collect();
    eprintln!("the hand's news: {news:?}");
    assert_eq!(news.len(), 3);
    assert!(news[1].contains("money.growth") && news[1].contains("0.0175 to 0.03"), "{}", news[1]);
    for _ in 0..3 {
        a.year().unwrap();
        a.turn();
    }
    let text = a.save(2_000).to_text();
    for what in ["discover", "rule", "found_town"] {
        assert!(text.contains(&format!(r#""kind":"by_hand","edit":{{"what":"{what}""#)), "{what}: {text}");
    }
    let b = save::load(&Save::from_text(&text).unwrap(), |_| {}).unwrap();
    assert!(!b.drifted, "a session with the new edits loads without drift");
    assert!(b.refused.is_empty(), "{:?}", b.refused);
    let mut b = b.game;
    assert_eq!(save::check(&b.sim), save::check(&a.sim));
    assert_eq!(series_bits(&b), series_bits(&a));
    assert_eq!(b.sim.params, a.sim.params);
    assert_eq!(b.sim.params.money.growth, 0.03);
    for g in [&mut a, &mut b] {
        g.year().unwrap();
        g.turn();
    }
    assert_eq!(save::check(&b.sim), save::check(&a.sim));
    // and the line of each reads back equal
    for edit in [
        Edit::Discover { tech, town: 3 },
        Edit::Rule { name: "rent_regime".into(), value: serde_json::json!("commons") },
        Edit::Rule { name: "hurdle".into(), value: serde_json::json!(0) },
        Edit::FoundTown { cell: 1234 },
    ] {
        let given = Given { year: 1800, month: 4, order: by_hand(edit) };
        let line = serde_json::to_string(&given).unwrap();
        eprintln!("{line}");
        assert_eq!(serde_json::from_str::<Given>(&line).unwrap(), given);
    }
}

#[test]
fn an_idea_found_by_hand_spreads_to_a_second_town() {
    // played under the game's rules; diffusion is the port's, with no switch
    let mut g = played(10);
    let town = biggest_towns(&g)[0];
    let tech = unfound(&g);
    let C = &g.sim.content;
    let t = C.tech_name.iter().position(|n| *n == tech).unwrap();
    g.give(by_hand(Edit::Discover { tech: tech.clone(), town })).unwrap();
    let st = &g.sim.st;
    assert_eq!((st.tech_year[t], st.tech_town[t]), (1760, town as i64));
    assert!(st.know[(town, t)] >= 0.25);
    let ev = g.sim.events.last().unwrap();
    assert_eq!((ev.kind.as_str(), ev.tech, ev.town), ("hand", Some(t), Some(town)));
    eprintln!("{}", ev.title);
    // found once is found: a second time is refused
    let why = g.give(by_hand(Edit::Discover { tech: tech.clone(), town })).unwrap_err();
    eprintln!("again: {why}");
    for _ in 0..10 {
        g.year().unwrap();
        g.turn();
    }
    let st = &g.sim.st;
    let others: Vec<(String, f64)> = (0..st.T_max).filter(|&i| i != town && st.town_active[i] && st.know[(i, t)] > 0.0).map(|i| (st.town_name[i].clone(), st.know[(i, t)])).collect();
    eprintln!("{tech} ten years on: {:.3} in the town it was found in; elsewhere {others:?}", st.know[(town, t)]);
    assert!(!others.is_empty(), "{tech} has reached a second town ten years on");
}

#[test]
fn money_growth_set_to_nothing_by_hand_stops_the_issue() {
    // played under the game's rules with its issue on the true total
    // (money.true_total), which reckons the issue at money.growth
    let mut g = played(10);
    assert!(g.sim.params.money.true_total);
    let issued = g.sim.st.issued;
    g.year().unwrap();
    g.turn();
    assert!(g.sim.st.issued > issued, "the island issues money before the edit");
    let issued = g.sim.st.issued;
    let title = match g.give(by_hand(Edit::Rule { name: "money.growth".into(), value: serde_json::json!(0) })) {
        Ok(_) => g.sim.events.last().unwrap().title.clone(),
        Err(why) => panic!("{why}"),
    };
    eprintln!("{title}");
    for _ in 0..3 {
        g.year().unwrap();
        g.turn();
    }
    eprintln!("issued {issued:.0} at the edit and {:.0} three years on", g.sim.st.issued);
    assert_eq!(g.sim.st.issued, issued, "no money is issued once money.growth is nothing");
}

#[test]
fn a_town_founded_by_hand_stands_at_the_years_end_with_people_in_it() {
    // played under the game's rules, whose founding rule is people.enabled
    let mut g = played(20);
    assert!(g.sim.params.people.enabled);
    let site = open_ground(&g);
    let towns_before = g.sim.st.town_active.iter().filter(|&&a| a).count();
    for _ in 0..4 {
        g.month().unwrap();
    }
    g.give(by_hand(Edit::FoundTown { cell: site })).unwrap();
    eprintln!("{}", g.sim.events.last().unwrap().title);
    // one founding a year
    let why = g.give(by_hand(Edit::FoundTown { cell: site })).unwrap_err();
    eprintln!("again: {why}");
    g.year().unwrap();
    g.turn();
    let st = &g.sim.st;
    let t = (0..st.T_max).find(|&t| st.town_active[t] && st.town_node[t] == site).expect("a town stands on the square");
    eprintln!("{} founded {} on square {site}: {:.0} people on {} cells", st.town_name[t], st.town_founded[t], st.pop[t], st.town_cells[t].len());
    assert_eq!(st.town_founded[t], 1770);
    assert_eq!(st.town_active.iter().filter(|&&a| a).count(), towns_before + 1);
    assert!(st.pop[t] > 0.0, "the town has people in it");
}

#[test]
fn a_new_edit_refused_leaves_no_trace() {
    // section 2.7 for the new edits
    let mut g = played(3);
    let st = &g.sim.st;
    let town = biggest_towns(&g)[0];
    let sea = (0..st.N).find(|&c| !st.world.land[c]).unwrap();
    let nowhere = (0..st.N).find(|&c| st.world.land[c] && st.catch[c] < 0);
    let known = (0..g.sim.content.NT).find(|&t| g.sim.content.tech_known[t]).map(|t| g.sim.content.tech_name[t].clone());
    let mut refused = vec![
        Edit::Discover { tech: "no such idea".into(), town },
        Edit::Discover { tech: unfound(&g), town: st.T_max + 1 },
        Edit::Rule { name: "seed".into(), value: serde_json::json!(8) },
        Edit::Rule { name: "T_max".into(), value: serde_json::json!(60) },
        Edit::Rule { name: "land.enabled".into(), value: serde_json::json!(true) },
        Edit::Rule { name: "accounts.works".into(), value: serde_json::json!(false) },
        Edit::Rule { name: "tree.stage".into(), value: serde_json::json!(0) },
        Edit::Rule { name: "no_such_rule".into(), value: serde_json::json!(1) },
        Edit::Rule { name: "money.growth".into(), value: serde_json::json!("a lot") },
        Edit::Rule { name: "prices".into(), value: serde_json::json!({"enabled": false}) },
        Edit::Rule { name: "prices.glut_stop_months".into(), value: serde_json::json!(0.1) },
        Edit::Rule { name: "money.growth".into(), value: serde_json::json!(0.0175) },
        Edit::Rule { name: "canals.max_projects".into(), value: serde_json::json!(3) },
        Edit::FoundTown { cell: sea },
        Edit::FoundTown { cell: st.N + 5 },
        Edit::FoundTown { cell: st.town_node[town] },
    ];
    if let Some(name) = known {
        refused.push(Edit::Discover { tech: name, town });
    }
    if let Some(c) = nowhere {
        refused.push(Edit::FoundTown { cell: c });
    }
    // a switch not on the list
    if let Some(s) = isostate_core::edits::SWITCHES.iter().find(|s| !isostate_core::edits::SWITCHABLE.contains(s) && !s.starts_with("land") && !s.starts_with("accounts")) {
        refused.push(Edit::Rule { name: s.to_string(), value: serde_json::json!(false) });
    }
    for at in ["the turn", "March"] {
        if at == "March" {
            for _ in 0..3 {
                g.month().unwrap();
            }
        }
        for edit in &refused {
            let (check, ledger, log, news, P, next) = (save::check(&g.sim), g.sim.st.by_hand.clone(), g.orders.clone(), g.sim.events.len(), g.sim.params.clone(), g.sim.st.pp_found_next);
            let why = g.give(by_hand(edit.clone())).expect_err(&format!("{edit:?} at {at}"));
            eprintln!("{edit:?} at {at}: {why}");
            assert_eq!(save::check(&g.sim), check);
            assert_eq!(g.sim.st.by_hand, ledger);
            assert_eq!(g.orders, log);
            assert_eq!(g.sim.events.len(), news);
            assert_eq!(g.sim.params, P);
            assert_eq!(g.sim.st.pp_found_next, next);
        }
    }
    // a town by hand under rules with no founding rule of the game's
    let mut port = Game::under(Rules::Port, 7, 6, 0);
    let st = &port.sim.st;
    let cell = (0..st.N).find(|&c| st.world.land[c] && st.catch[c] >= 0 && !st.town_node.contains(&c)).unwrap();
    let why = port.give(by_hand(Edit::FoundTown { cell })).unwrap_err();
    eprintln!("under the port's rules: {why}");
    assert!(why.contains("people.enabled"));
    assert!(port.orders.is_empty());
}

#[test]
fn a_run_with_an_empty_list_of_edits_is_the_run_without() {
    // any rules would do; the game's, with six families, to 1765
    let mut P = save::params(Rules::Game, 7, 6);
    P.end_year = Some(1765);
    let mut plain = isostate_core::sim::start(&P, Rules::Game.content());
    while plain.year <= plain.end_year {
        isostate_core::sim::run_year(&mut plain).unwrap();
    }
    let listed = edits::play(&P, Rules::Game.content(), &[]).unwrap();
    assert_eq!(save::check(&listed), save::check(&plain));
    assert_eq!(listed.events.len(), plain.events.len());
    // and a list is carried out in its months, in the order of its dates
    let list: Vec<Dated> = serde_json::from_str(
        r#"[{"before": [1760, 1], "edit": {"what": "rule", "name": "money.growth", "value": 0}},
            {"before": [1755, 7], "edit": {"what": "money", "purse": "households", "factor": 2.0}},
            {"before": [1762, 4], "edit": {"what": "rule", "name": "hands.ration", "value": false}}]"#,
    )
    .unwrap();
    let edited = edits::play(&P, Rules::Game.content(), &list).unwrap();
    let hand: Vec<(f64, &str)> = edited.events.iter().filter(|e| e.kind == "hand").map(|e| (e.year, e.title.as_str())).collect();
    eprintln!("{hand:?}");
    assert_eq!(hand.len(), 3);
    assert!((hand[0].0 - (1755.0 + 6.5 / 12.0)).abs() < 1e-9 && hand[0].1.contains("households'"));
    assert!((hand[1].0 - (1760.0 + 0.5 / 12.0)).abs() < 1e-9 && hand[1].1.contains("money.growth"));
    assert!(hand[2].1.contains("hands.ration, true to false"), "{}", hand[2].1);
    assert!(!edited.params.hands.ration);
    assert_ne!(save::check(&edited), save::check(&plain));
    let run = edits::run_with(&P, Rules::Game.content(), &list).unwrap();
    assert_eq!(run.params, P, "the run's parameters are those it began with");
    assert_eq!(edited.params.money.growth, 0.0);
    // a refused edit stops the run, in a sentence that names it
    let bad = vec![Dated { before: (1758, 3), edit: Edit::Rule { name: "seed".into(), value: serde_json::json!(3) } }];
    let why = edits::play(&P, Rules::Game.content(), &bad).unwrap_err();
    eprintln!("{why}");
    assert!(why.contains("before March 1758") && why.contains("seed"), "{why}");
    let late = vec![Dated { before: (1790, 1), edit: Edit::Harvest { town: None, factor: 0.9 } }];
    assert!(edits::play(&P, Rules::Game.content(), &late).is_err());
}

/// A rule's switch thrown in a running island `months` months into a year,
/// five years played, thrown back, five more: whether it comes through, and
/// the island's fingerprint (`save::check`) at the turn ten years on if it does.
fn switch_trial(base: &Game, name: &str, months: usize) -> Result<String, String> {
    let mut g = base.clone();
    for _ in 0..months {
        g.month()?;
    }
    let throw = |g: &mut Game| -> Result<(), String> {
        let now = isostate_core::sweep::set_field(&g.sim.params, name, &serde_json::json!(true)).unwrap();
        let on = now == g.sim.params;
        let value = serde_json::json!(!on);
        g.sim.params = isostate_core::sweep::set_field(&g.sim.params, name, &value)?;
        Ok(())
    };
    let played = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || -> Result<String, String> {
        for _ in 0..2 {
            throw(&mut g)?;
            for _ in 0..5 {
                g.year()?;
                g.turn();
            }
        }
        // (written so that a NaN fails: the monthly money check lets one by)
        let st = &g.sim.st;
        if !(drift(&g) < 1e-6) || !(econ::money_total(st) > 0.0) {
            return Err(format!("the money is {:.6e}, and drifted by {:.3e} of the stock", econ::money_total(st), drift(&g)));
        }
        let towns: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
        if let Some(&t) = towns.iter().find(|&&t| !st.pop[t].is_finite() || !st.wage[t].is_finite() || (0..st.price.cols).any(|k| !st.price[(t, k)].is_finite())) {
            return Err(format!("{}'s people, wage or prices are not a number", st.town_name[t]));
        }
        Ok(save::check(&g.sim))
    }));
    match played {
        Ok(r) => r,
        Err(p) => Err(format!("panicked: {}", p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default())),
    }
}

#[test]
fn which_switches_can_be_thrown_in_a_running_island() {
    // section 6.3: island 7 under the game's rules with six families, played
    // to 1790 once; for each rule's switch, on a copy, thrown the other way,
    // five years, thrown back, five more: once at the year's turn and once
    // five months into it, since an edit falls between any two months. The
    // land and the books are left out untried. Every switch on SWITCHABLE
    // must come through both times; one that comes through and is not on it
    // is said, so that the list can grow. (It is played under every rule of
    // the game's, which is the point: a rule added later is tried with the rest)
    let base = played(41);
    assert_eq!(base.last_year(), 1790);
    // the island untouched at the turn of 1800, to say whether a switch moved it
    let mut plain = base.clone();
    for _ in 0..10 {
        plain.year().unwrap();
        plain.turn();
    }
    let plain = save::check(&plain.sim);
    // a switch that changes nothing in the 1790s (no railway is planned yet,
    // no town is short of hands) is tried again from 1850, where it may; and
    // one that changes nothing in the 1850s either is tried a third time from
    // 1905, after island 7 has found the first machine for tasks (1903,
    // docs/rules/tasks.md), so that tasks.enabled is thrown where it acts (on a
    // game made to be played to 1920, since one made to 1900 cannot go past it)
    let mut later: Option<(Game, String)> = None;
    let mut latest: Option<(Game, String)> = None;
    let mut through = Vec::new();
    for &name in edits::SWITCHES {
        if name.starts_with("land.") || name.starts_with("accounts.") {
            continue;
        }
        let mut ok = true;
        let mut moved = false;
        for months in [0, 5] {
            let t0 = std::time::Instant::now();
            let r = switch_trial(&base, name, months);
            moved |= matches!(&r, Ok(check) if *check != plain);
            eprintln!("{name:20} thrown {months} months into 1791: {:5.1} s  {}", t0.elapsed().as_secs_f64(), match &r {
                Ok(check) if *check == plain => "came through, and the island stands as it would have".to_string(),
                Ok(_) => "came through, with another history".to_string(),
                Err(why) => why.chars().take(200).collect::<String>(),
            });
            ok &= r.is_ok();
        }
        if ok && !moved {
            let (base, plain) = later.get_or_insert_with(|| {
                let mut g = base.clone();
                for _ in 0..59 {
                    g.year().unwrap();
                    g.turn();
                }
                let mut p = g.clone();
                for _ in 0..10 {
                    p.year().unwrap();
                    p.turn();
                }
                let check = save::check(&p.sim);
                (g, check)
            });
            assert_eq!(base.last_year(), 1849);
            for months in [0, 5] {
                let r = switch_trial(base, name, months);
                moved |= matches!(&r, Ok(check) if check != plain);
                eprintln!("{name:20} thrown {months} months into 1850: {}", match &r {
                    Ok(check) if check == plain => "came through, and the island stands as it would have".to_string(),
                    Ok(_) => "came through, with another history".to_string(),
                    Err(why) => why.chars().take(200).collect::<String>(),
                });
                ok &= r.is_ok();
            }
        }
        if ok && !moved {
            let (base, plain) = latest.get_or_insert_with(|| {
                // (a game played past 1900 is made so from its start: `Game::under_to`)
                let mut g = Game::under_to(Rules::Game, 7, 6, 1_000, 1920).unwrap();
                for _ in 0..155 {
                    g.year().unwrap();
                    g.turn();
                }
                let mut p = g.clone();
                for _ in 0..10 {
                    p.year().unwrap();
                    p.turn();
                }
                let check = save::check(&p.sim);
                (g, check)
            });
            assert_eq!(base.last_year(), 1904);
            for months in [0, 5] {
                let r = switch_trial(base, name, months);
                eprintln!("{name:20} thrown {months} months into 1905: {}", match &r {
                    Ok(check) if check == plain => "came through, and the island stands as it would have".to_string(),
                    Ok(_) => "came through, with another history".to_string(),
                    Err(why) => why.chars().take(200).collect::<String>(),
                });
                ok &= r.is_ok();
            }
        }
        if ok {
            through.push(name);
        }
    }
    eprintln!("came through: {through:?}");
    for name in edits::SWITCHABLE {
        assert!(through.contains(name), "{name} is on edits::SWITCHABLE and did not come through");
    }
    for name in &through {
        if !edits::SWITCHABLE.contains(name) {
            eprintln!("{name} came through and is not on edits::SWITCHABLE: it could be added");
        }
    }
}
