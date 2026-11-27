//! The start's two switches (docs/rules/landfall.md, "The start's switches"):
//! settlers by ship (`immigration.enabled`) and ideas by sea (`abroad.ideas`),
//! on at landfall and off on the settled island. A game may be begun with
//! either the other way (`save::Switches`, `Game::begin`), its save remembers
//! how it began, and either may be thrown in the middle of a game by an edit.
//!
//! Every case here is played under the game's rules from landfall
//! (`landfall::landfall`: births that answer to free land, hamlets, settlers,
//! the guard on a short town's wage, carriage from 1750, pits at their
//! market's size, land that stays taken, no settlers to a hungry port, ideas
//! by sea), or on the settled island under the game's rules, as each says.
//! They lean on no island's history: only on a game being the same twice.
#![allow(non_snake_case)]

use isostate_core::edits::Edit;
use isostate_core::params::Rules;
use isostate_core::save::{self, Game, Order, Save, Switches};

fn series_bits(g: &Game) -> Vec<Vec<u64>> {
    g.sim.st.series.data.iter().map(|d| d.iter().map(|x| x.to_bits()).collect()).collect()
}

fn years(g: &mut Game, n: usize) {
    for _ in 0..n {
        g.year().unwrap();
        g.turn();
    }
}

fn months(g: &mut Game, n: usize) {
    for _ in 0..n {
        g.month().unwrap();
    }
}

fn throw(name: &str, on: bool) -> Order {
    Order::ByHand { edit: Edit::Rule { name: name.to_string(), value: serde_json::json!(on) } }
}

const NO_SETTLERS: Switches = Switches { settlers: Some(false), ideas_by_sea: None };

#[test]
fn begun_with_the_starts_own_switches_a_game_is_the_one_the_old_ways_begin() {
    // (island 3 with two families: only the parameters and the first state are compared)
    let (seed, fam, to) = (3, 2, 1900);
    let plain = Switches::default();
    for (land, landfall) in [(true, false), (false, false), (true, true), (false, true)] {
        let old = match (land, landfall) {
            (_, true) => Game::landfall_to(Rules::Game, seed, fam, 0, to, land).unwrap(),
            (true, false) => Game::on_new_land_to(Rules::Game, seed, fam, 0, to).unwrap(),
            (false, false) => Game::on_old_land_to(Rules::Game, seed, fam, 0, to).unwrap(),
        };
        let new = Game::begin(Rules::Game, seed, fam, 0, to, land, landfall, plain).unwrap();
        assert_eq!(new.sim.params, old.sim.params, "land {land}, landfall {landfall}");
        assert_eq!(save::check(&new.sim), save::check(&old.sim));
        assert_eq!(new.begun, plain);
        // a switch asked for as the start has it is the start's own
        let same = Switches { settlers: Some(landfall), ideas_by_sea: Some(landfall) };
        let g = Game::begin(Rules::Game, seed, fam, 0, to, land, landfall, same).unwrap();
        assert_eq!((g.begun, &g.sim.params), (plain, &old.sim.params));
    }
    // and one asked for the other way is thrown, and nothing else is
    let g = Game::begin(Rules::Game, seed, fam, 0, to, true, true, Switches { settlers: Some(false), ideas_by_sea: Some(false) }).unwrap();
    let mut P = Game::landfall_to(Rules::Game, seed, fam, 0, to, true).unwrap().sim.params;
    assert!(P.immigration.enabled && P.abroad.ideas);
    P.immigration.enabled = false;
    P.abroad.ideas = false;
    assert_eq!(g.sim.params, P);
    // on the settled island the colony's switches may be thrown on, as the Rules tab throws them
    let g = Game::begin(Rules::Game, seed, fam, 0, to, true, false, Switches { settlers: Some(true), ideas_by_sea: None }).unwrap();
    assert!(g.sim.params.immigration.enabled && !g.sim.params.abroad.ideas && !g.sim.params.start.landfall);
    assert_eq!(g.begun, Switches { settlers: Some(true), ideas_by_sea: None });
}

#[test]
fn a_save_of_a_start_as_it_comes_says_nothing_of_the_switches() {
    // the settled island under the game's rules, and landfall as it comes: written as before
    let mut settled = Game::new(7, 2, 0);
    years(&mut settled, 1);
    let mut colony = Game::landfall_to(Rules::Game, 7, 2, 0, 1900, true).unwrap();
    years(&mut colony, 1);
    for (g, what) in [(&settled, "the settled island"), (&colony, "landfall")] {
        let text = g.save(0).to_text();
        assert!(!text.contains("settlers") && !text.contains("ideas_by_sea"), "{what}: {text}");
        let back = save::load(&Save::from_text(&text).unwrap(), |_| {}).unwrap();
        assert!(!back.drifted && back.game.begun.plain(), "{what}");
    }
    // a colony begun with no ships says so, and nothing of the switch it has as it comes
    let mut alone = Game::begin(Rules::Game, 7, 2, 0, 1900, true, true, NO_SETTLERS).unwrap();
    years(&mut alone, 1);
    let text = alone.save(0).to_text();
    assert!(text.contains("\"settlers\": false") && !text.contains("ideas_by_sea"), "{text}");
}

#[test]
fn a_colony_begun_with_no_settlers_and_gone_on_with_is_the_one_played_straight_through() {
    // island 7, six families, from landfall with no settlers by ship, to 1780. Ideas by
    // sea are thrown off by an edit five months into 1766, and on again at the turn of
    // 1769: one game is played straight through, the other saved three months into
    // 1771, written to text, read back and played again from its save to the same end.
    let mut straight = Game::begin(Rules::Game, 7, 6, 1_000, 1900, true, true, NO_SETTLERS).unwrap();
    assert!(!straight.sim.params.immigration.enabled && straight.sim.params.abroad.ideas);
    let edits = |g: &mut Game| {
        if g.last_year() == 1765 && g.month == 5 {
            g.give(throw("abroad.ideas", false)).unwrap();
        }
        if g.last_year() == 1768 && g.month == 0 {
            g.give(throw("abroad.ideas", true)).unwrap();
        }
    };
    let mut saved = None;
    while straight.last_year() < 1780 {
        edits(&mut straight);
        if (straight.last_year(), straight.month) == (1770, 3) {
            saved = Some(straight.save(0).to_text());
        }
        if straight.month().unwrap() {
            straight.turn();
        }
    }
    let text = saved.unwrap();
    assert!(text.contains("\"settlers\": false") && text.contains("\"landfall\": true"), "{text}");
    let back = save::load(&Save::from_text(&text).unwrap(), |_| {}).unwrap();
    assert!(!back.drifted && back.refused.is_empty(), "{:?}", back.refused);
    let mut again = back.game;
    assert_eq!((again.last_year(), again.month, again.begun), (1770, 3, NO_SETTLERS));
    assert_eq!(again.orders.len(), 2);
    while again.last_year() < 1780 {
        if again.month().unwrap() {
            again.turn();
        }
    }
    assert_eq!(save::check(&again.sim), save::check(&straight.sim));
    assert_eq!(series_bits(&again), series_bits(&straight));
    assert_eq!(again.sim.params, straight.sim.params);
    // nobody came by ship in either
    assert_eq!(straight.sim.st.immigration.total, 0.0);
    // where a colony as it comes had settlers in those years
    let mut ships = Game::landfall_to(Rules::Game, 7, 6, 1_000, 1900, true).unwrap();
    years(&mut ships, 31);
    assert_eq!(ships.last_year(), 1780);
    let came = ships.sim.st.immigration.total;
    assert!(came > 1_000.0, "settlers by 1780 in the colony with ships: {came}");
    eprintln!("island 7 to 1780 from landfall: {came:.0} settlers came with ships, none without; people {:.0} against {:.0}", ships.sim.st.pop.iter().sum::<f64>(), straight.sim.st.pop.iter().sum::<f64>());
}

#[test]
fn the_ships_may_stop_and_come_again_in_the_middle_of_a_colony() {
    // island 7, six families, from landfall as it comes, played to 1760; on a copy each
    // switch is thrown off by an edit (at the turn, and on another copy five months
    // into 1761), five years are played, it is thrown on again, five more: no money
    // check fails (`sim::run_month` checks it every month) and the money is a number
    let mut base = Game::landfall_to(Rules::Game, 7, 6, 1_000, 1900, true).unwrap();
    years(&mut base, 11);
    assert_eq!(base.last_year(), 1760);
    for name in ["immigration.enabled", "abroad.ideas"] {
        for into in [0, 5] {
            let mut g = base.clone();
            months(&mut g, into);
            g.give(throw(name, false)).unwrap();
            for _ in 0..5 {
                g.year().unwrap();
                g.turn();
            }
            g.give(throw(name, true)).unwrap();
            years(&mut g, 5);
            let money = isostate_core::econ::money_total(&g.sim.st);
            assert!(money.is_finite() && money > 0.0, "{name} thrown {into} months into 1761: money {money}");
            assert_eq!(g.orders.len(), 2, "{name}");
        }
    }
}

/// A save made before the start's switches, by main's own dev build of 27
/// November 2026 (55cd66c): the family game on island 7 from landfall, six
/// families, the demo's orders (the family's trade, a works at the seat), to
/// 2050, saved at the turn of 1765 (`--family --landfall --demo --advance 15
/// --save FILE --shot PATH`). It says nothing of the switches, and is played
/// again from landfall as it comes, with its settlers and ideas by sea.
///
/// It reaches the island main reached, to the last digit, only while the
/// landfall start's history is the one of that day: a later change that moves
/// it on purpose (it anchors `golden_landfall.txt` again) moves this one's too,
/// so the check is held only while the landfall start's hashes of 1750-1900
/// are the ones of that day (`GOLDEN_WHEN_MADE`); after that the test says so
/// and holds the rest.
#[test]
fn a_landfall_save_made_before_the_switches_is_played_as_it_was() {
    const GOLDEN_WHEN_MADE: &str = "1750-1900 economy  0x534ad2f7fd8080a0";
    let text = include_str!("data/landfall_main_1765.json");
    assert!(!text.contains("settlers") && !text.contains("ideas_by_sea"), "the file is as main wrote it");
    let saved = Save::from_text(text).unwrap();
    assert_eq!((saved.landfall, saved.land, saved.seed, saved.families, saved.year, saved.to), (true, true, 7, 6, 1765, Some(2050)));
    assert_eq!((saved.settlers, saved.ideas_by_sea), (None, None));
    let back = save::load(&saved, |_| {}).unwrap();
    let g = &back.game;
    assert!(g.at_landfall() && g.begun.plain() && g.sim.params.immigration.enabled && g.sim.params.abroad.ideas);
    assert!(g.sim.st.immigration.total > 0.0, "the ships came");
    // saved again it is written as main wrote it, the date and the version aside
    let again = g.save(0).to_text();
    assert!(!again.contains("settlers") && !again.contains("ideas_by_sea"), "{again}");
    let golden = include_str!("golden_landfall.txt");
    if golden.lines().any(|l| l == GOLDEN_WHEN_MADE) {
        assert!(back.refused.is_empty(), "{:?}", back.refused);
        assert!(!back.drifted, "the old save does not reach the island main reached: its check is {}, now {}", saved.check, save::check(&g.sim));
    } else {
        eprintln!("the landfall start's history has moved since the old save was put here (golden_landfall.txt's played century is not {GOLDEN_WHEN_MADE:?}): it is held to its start, not to its own check");
    }
}
