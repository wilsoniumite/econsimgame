//! The rules as a list (not in the Python; docs/designs/sandbox.md, sections
//! 4.6 and 6.6): every name the sweep can set is walked, in the order of
//! `Params`, rules that are off among them; every value walked is the one the
//! parameters hold; the pinned names and every rule's switch are there; a name
//! fixed when the island is made offers nothing; what the walk says of an
//! offer is what an edit by hand does with it; and the walk is quick enough to
//! run on the simulation's thread between two months.
//!
//! The case played on an island (island 7 at the turn into 1790, under the
//! game's rules with six families) compares the walk with the edit on the same
//! game, so it leans on no rule's history: whichever offers the island's rules
//! make, the two must agree.
#![allow(non_snake_case)]

use std::collections::BTreeSet;

use isostate_core::edits::{self, Edit, SWITCHABLE, SWITCHES};
use isostate_core::params::{default_params, game_params};
use isostate_core::rulebook::{rulebook, unfound, Entry, Kind, FIXED, PINNED};
use isostate_core::save::Game;
use isostate_core::sweep::set_field;

fn entry<'a>(book: &'a [Entry], name: &str) -> Option<&'a Entry> {
    book.iter().find(|e| e.name == name)
}

#[test]
fn every_name_walked_holds_the_value_the_rules_hold_and_comes_in_their_order() {
    for P in [game_params(), default_params()] {
        let book = rulebook(&P);
        // (the reader's own list of the fields is how the walk knows the order: seed first)
        assert_eq!(book.first().map(|e| e.name.as_str()), Some("seed"));
        let names: BTreeSet<&str> = book.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names.len(), book.len(), "a name is walked twice");
        for e in &book {
            // a field the serialiser leaves out and the walk could not fill would be null:
            // only a name that is truly "nothing" may be (end_year, the content's)
            assert!(!e.value.is_null() || e.name == "end_year", "{} was not found: add its line to rulebook::skipped", e.name);
            let again = set_field(&P, &e.name, &e.value).unwrap_or_else(|why| panic!("{}: {why}", e.name));
            assert!(again == P, "{} set to its own value {} changes the rules", e.name, e.value);
        }
        // the rules that are off under the port are walked with their numbers too
        assert!(entry(&book, "canals.enabled").is_some() && entry(&book, "roads.enabled").is_some());
        assert!(book.iter().filter(|e| e.rule == "money").count() >= 2);
    }
}

#[test]
fn the_pinned_names_and_every_rules_switch_are_there() {
    let book = rulebook(&game_params());
    for name in PINNED {
        let e = entry(&book, name).unwrap_or_else(|| panic!("{name} is pinned and not a rule's name"));
        assert!(matches!(e.kind, Kind::Number | Kind::Choice), "{name} is pinned and is {:?}", e.kind);
        assert!(!e.offers.is_empty(), "{name} is pinned and offers nothing");
    }
    for &name in SWITCHES {
        let e = entry(&book, name).unwrap_or_else(|| panic!("{name} is a switch and not walked"));
        assert_eq!(e.kind, Kind::Switch { throwable: SWITCHABLE.contains(&name) }, "{name}");
    }
    let rent = entry(&book, "rent_regime").unwrap();
    assert_eq!(rent.kind, Kind::Choice);
    assert_eq!(rent.offers.iter().map(|o| o.word.as_str()).collect::<Vec<_>>(), ["Landlords", "Commons"]);
    // the game's money.growth, 0.0175: None, Half, As set, Half again, Double
    let growth = entry(&book, "money.growth").unwrap();
    let offered: Vec<(&str, f64)> = growth.offers.iter().map(|o| (o.word.as_str(), o.value.as_f64().unwrap())).collect();
    assert_eq!(offered, [("None", 0.0), ("Half", 0.00875), ("As set", 0.0175), ("Half again", 0.0175 * 1.5), ("Double", 0.035)]);
    // a whole number is offered whole, and once: rail_max_projects 6 is 0, 3, 6, 9, 12
    let projects = entry(&book, "rail_max_projects").unwrap();
    assert!(projects.offers.iter().all(|o| o.value.is_u64()), "{:?}", projects.offers);
    assert_eq!(projects.offers.iter().map(|o| o.value.as_u64().unwrap()).collect::<Vec<_>>(), [0, 3, 6, 9, 12]);
}

#[test]
fn a_name_fixed_when_the_island_is_made_offers_nothing() {
    let book = rulebook(&game_params());
    for name in ["seed", "W", "n_families", "tree.stage", "land.enabled", "accounts.enabled", "family_start_cash"] {
        let e = entry(&book, name).unwrap_or_else(|| panic!("{name} not walked"));
        assert_eq!(e.fixed.as_deref(), Some(FIXED), "{name}");
        assert!(e.offers.is_empty(), "{name} is fixed and offers {:?}", e.offers);
    }
    for e in book.iter().filter(|e| e.fixed.is_some()) {
        assert!(e.offers.is_empty(), "{} is fixed and offers {:?}", e.name, e.offers);
        assert!(edits::rule_refused(&game_params(), &e.name, &e.value).is_some(), "{} is fixed and an edit of it is not refused", e.name);
    }
    // a number that is nothing under the game's rules offers nothing, and so do lists and words
    for e in &book {
        if e.kind == Kind::Number && e.game.as_f64() == Some(0.0) || e.kind == Kind::Other {
            assert!(e.offers.is_empty(), "{} offers {:?}", e.name, e.offers);
        }
    }
}

#[test]
fn what_the_walk_says_of_an_offer_is_what_the_edit_does_with_it() {
    // island 7 under the game's rules with six families, at the turn into 1790
    let mut g = Game::new(7, 6, 1_000);
    for _ in 0..40 {
        g.year().unwrap();
        g.turn();
    }
    assert_eq!(g.last_year(), 1789);
    let book = rulebook(&g.sim.params);
    // ten rules, each with an offer the walk calls fine and one it refuses: first those
    // with an offer refused for more than having the value already (a number that the
    // rules' check will not run with another), then the rest
    let already = |o: &&isostate_core::rulebook::Offer| o.refused.as_deref().is_some_and(|w| w.ends_with("already"));
    let other = |e: &&Entry| e.offers.iter().any(|o| o.refused.is_some() && !already(&o));
    let order: Vec<&Entry> = book.iter().filter(other).chain(book.iter().filter(|e| !other(e))).collect();
    let mut rules: Vec<&str> = Vec::new();
    let mut tried = 0;
    for e in order {
        if rules.len() == 10 {
            break;
        }
        if rules.contains(&e.rule.as_str()) {
            continue;
        }
        let refused = e.offers.iter().find(|o| o.refused.is_some() && !already(o)).or_else(|| e.offers.iter().find(|o| o.refused.is_some()));
        let (Some(fine), Some(refused)) = (e.offers.iter().find(|o| o.refused.is_none()), refused) else { continue };
        rules.push(&e.rule);
        let mut s = g.sim.clone();
        let done = edits::carry_out(&mut s, &Edit::Rule { name: e.name.clone(), value: fine.value.clone() });
        assert!(done.is_ok(), "{} {}: the walk calls it fine and the edit refuses it: {:?}", e.name, fine.word, done);
        assert!(set_field(&g.sim.params, &e.name, &fine.value).unwrap() == s.params, "{} {}", e.name, fine.word);
        let mut s = g.sim.clone();
        let said = edits::carry_out(&mut s, &Edit::Rule { name: e.name.clone(), value: refused.value.clone() });
        assert_eq!(said.err(), refused.refused, "{} {}", e.name, refused.word);
        assert!(s.params == g.sim.params, "{} {}: a refused edit changed the rules", e.name, refused.word);
        eprintln!("{:24} {:10} carried out; {:10} refused: {}", e.name, fine.word, refused.word, refused.refused.as_deref().unwrap_or_default());
        tried += 1;
    }
    assert_eq!(tried, 10, "only {} rules had an offer of each kind: {rules:?}", tried);
    // a switch that may not be thrown is refused in the edit's own words
    let throwable: Vec<&Entry> = book.iter().filter(|e| e.kind == Kind::Switch { throwable: true }).collect();
    assert_eq!(throwable.len(), SWITCHABLE.len());
    // the ideas not yet found: none known before 1750, none found already
    let C = &g.sim.content;
    let left = unfound(&g.sim);
    assert!(!left.is_empty() && left.len() < C.tech_name.len());
    for (name, label) in &left {
        let t = C.tech_name.iter().position(|n| n == name).unwrap();
        assert!(!C.tech_known[t] && g.sim.st.tech_year[t] < 0 && *label == C.tech_label[t]);
    }
}

#[test]
fn the_walk_of_the_games_rules_is_quick() {
    let P = game_params();
    let _ = rulebook(&P);
    let t0 = std::time::Instant::now();
    let n = 5;
    let mut book = Vec::new();
    for _ in 0..n {
        book = rulebook(&P);
    }
    let ms = t0.elapsed().as_secs_f64() * 1e3 / n as f64;
    let count = |k: fn(&Entry) -> bool| book.iter().filter(|e| k(e)).count();
    eprintln!(
        "the walk of game_params(): {} entries, {} switches ({} throwable), {} other switches, {} numbers ({} offering nothing), 1 choice, {} others, {} fixed; {:.1} ms",
        book.len(),
        count(|e| matches!(e.kind, Kind::Switch { .. })),
        count(|e| e.kind == Kind::Switch { throwable: true }),
        count(|e| e.kind == Kind::Flag),
        count(|e| e.kind == Kind::Number),
        count(|e| e.kind == Kind::Number && e.offers.is_empty()),
        count(|e| e.kind == Kind::Other),
        count(|e| e.fixed.is_some()),
        ms
    );
    let offers: Vec<_> = book.iter().flat_map(|e| e.offers.iter()).collect();
    let refused: Vec<&str> = offers.iter().filter_map(|o| o.refused.as_deref()).collect();
    let mut other: Vec<&str> = refused.iter().copied().filter(|w| !w.ends_with("already")).collect();
    other.sort();
    other.dedup();
    eprintln!("{} offers, {} refused ({} for the value it has already); the other refusals: {other:?}", offers.len(), refused.len(), refused.iter().filter(|w| w.ends_with("already")).count());
    // The Rules tab walks the rules when it is opened, so the walk must not be felt: it took
    // 35 ms alone in the dev profile on 19 November 2026. It is held to half a second, not to
    // 50 ms as it was, since the core's tests now run all at once (crates/isostate-core/run_tests.py)
    // and the others beside it took it to 111 to 138 ms with nothing wrong; half a second still
    // catches a walk grown ten times slower, which is what this is for.
    assert!(ms < 500.0, "the walk took {ms:.1} ms");
}
