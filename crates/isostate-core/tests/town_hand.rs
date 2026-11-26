//! The hand on a town's making (docs/designs/sandbox.md, "Built: the hand on
//! a town's making"; edits.rs, TOWN-FLOW-HAND): a method turned off in a town
//! founds no works there until it is turned on again; a kind closed and
//! forbidden waits for the year's turn and leaves no works of it standing; the
//! panel's offers are the tray's own edits, judged by the same function; a
//! session with the panel's edits saved and loaded, and a run with no window
//! given the same edits as a dated list, come back the same to the last digit;
//! and asking the making changes nothing.
//!
//! Every case is played on island 7 under the game's rules with six families
//! (`Game::new`), and compares a game with a copy of itself. They lean on the
//! works rule (`works.enabled`), which sizes a works by hand and says whether a
//! square has room, and on the books (`accounts.enabled`), which the making is
//! drawn from; `at` asserts that both are on.
#![allow(non_snake_case)]

use isostate_core::econ;
use isostate_core::edits::{self, Dated, Edit};
use isostate_core::inspect::{self, Thing};
use isostate_core::making::Making;
use isostate_core::params::Rules;
use isostate_core::save::{self, Done, Game, Order, Save};

fn by_hand(edit: Edit) -> Order {
    Order::ByHand { edit }
}

/// Island 7 played `years` years from its start, at the turn.
fn at(years: usize) -> Game {
    let mut g = Game::new(7, 6, 1_000);
    for _ in 0..years {
        g.year().unwrap();
        g.turn();
    }
    assert!(g.sim.params.works.enabled && g.sim.params.accounts.enabled, "played under the works rule, with the books open");
    g
}

fn biggest(g: &Game) -> usize {
    let st = &g.sim.st;
    (0..st.T_max).filter(|&t| st.town_active[t]).max_by(|&a, &b| st.pop[a].total_cmp(&st.pop[b])).unwrap()
}

fn making(g: &Game, t: usize) -> Making {
    inspect::tell(&g.sim, Thing::Making(t)).unwrap().making.unwrap()
}

fn drift(g: &Game) -> f64 {
    let st = &g.sim.st;
    let now = econ::money_total(st);
    (now - (st.money_start + st.issued + st.by_hand.money)).abs() / now.max(1.0)
}

/// Works founded in town t of method m from `year` on.
fn founded(g: &Game, t: usize, m: usize, year: i64) -> usize {
    let st = &g.sim.st;
    (0..st.F_count).filter(|&f| st.f_town[f] == t && st.f_method[f] == m && st.f_born[f] >= year).count()
}

#[test]
fn a_method_turned_off_founds_no_works_in_its_town_until_it_is_turned_on() {
    let base = at(40);
    let t = biggest(&base);
    // the method the island founds most of in its biggest town over the next eight years, untouched
    let mut plain = base.clone();
    for _ in 0..8 {
        plain.year().unwrap();
        plain.turn();
    }
    let C = &base.sim.content;
    let m = (0..C.M).max_by_key(|&m| (founded(&plain, t, m, 1790), std::cmp::Reverse(m))).unwrap();
    let untouched = founded(&plain, t, m, 1790);
    eprintln!("{} untouched, 1790-1797: {untouched} new works of {}", base.sim.st.town_name[t], C.method_label[m]);
    assert!(untouched > 0, "the town founds works of some method in eight years");

    let mut g = base.clone();
    let method = C.method_name[m].clone();
    assert_eq!(g.give(by_hand(Edit::Forbid { town: t, method: method.clone() })).unwrap(), Done::Done);
    let ev = g.sim.events.last().unwrap();
    eprintln!("{}\n  {}", ev.title, ev.text);
    assert_eq!((ev.kind.as_str(), ev.town), ("hand", Some(t)));
    assert!(ev.title.starts_with("By hand: ") && ev.title.contains("turned off in"), "{}", ev.title);
    assert!(edits::forbidden(&g.sim.st, t, m));
    // a second time is refused in its own words, and so is a works of it by hand on any square of the town
    let again = g.give(by_hand(Edit::Forbid { town: t, method: method.clone() })).unwrap_err();
    assert!(again.contains("turned off in") && again.contains("already"), "{again}");
    let st = &g.sim.st;
    let square = st.town_cells[t].iter().copied().find(|&c| edits::works_offered(&g.sim, c).is_ok()).expect("a square a works may stand on");
    let why = g.clone().give(by_hand(Edit::BuildWorks { method: method.clone(), cell: square })).unwrap_err();
    eprintln!("refused: {why}");
    assert!(why.contains("may not be built in") && why.contains("turned off there by hand"), "{why}");
    // and the panel's menu says so
    let mk = making(&g, t);
    let kind = mk.hand_of(C.method_out[m]).expect("the kind is on the chart");
    assert!(kind.methods.iter().any(|x| x.method == m && x.forbidden));
    assert!(kind.by_hand.iter().any(|l| l.contains("turned off in")), "{:?}", kind.by_hand);

    let mut on = g.clone();
    for _ in 0..8 {
        g.year().unwrap();
        g.turn();
    }
    let after = founded(&g, t, m, 1790);
    eprintln!("turned off at the turn of 1790: {after} new works of it to 1797");
    assert_eq!(after, 0, "no works of a method turned off is founded in its town");
    assert!(drift(&g) < 1e-6);

    // turned on again, it is as the island judges it; a second time is refused
    let why = on.clone().give(by_hand(Edit::Allow { town: t, method: "no_such_method".into() })).unwrap_err();
    assert!(why.contains("no method called"), "{why}");
    assert_eq!(on.give(by_hand(Edit::Allow { town: t, method: method.clone() })).unwrap(), Done::Done);
    assert!(on.sim.events.last().unwrap().title.contains("turned on again in"));
    assert!(on.sim.st.by_hand.forbidden.is_empty());
    let why = on.give(by_hand(Edit::Allow { town: t, method })).unwrap_err();
    assert!(why.contains("is not turned off in"), "{why}");
}

#[test]
fn a_kind_closed_and_forbidden_waits_for_the_turn_and_leaves_none_standing() {
    let mut g = at(40);
    let t = biggest(&g);
    let C = g.sim.content.clone();
    // the kind of the town with most works standing
    let mk = making(&g, t);
    let kind = mk.hand.iter().max_by_key(|k| (k.standing.len(), std::cmp::Reverse(k.good))).unwrap().clone();
    let g_ = kind.good;
    let n = kind.standing.len();
    eprintln!("{}: {} works making {} in January 1790", g.sim.st.town_name[t], n, C.good_label[g_]);
    assert!(n >= 2);
    for _ in 0..3 {
        g.month().unwrap();
    }
    let edit = Edit::CloseKind { town: t, good: kind.good_name.clone() };
    assert_eq!(g.give(by_hand(edit.clone())).unwrap(), Done::Waits, "given in April it waits for the turn");
    assert_eq!(g.waiting.len(), 1);
    assert!(edits::kind_works(&g.sim.st, &C, t, g_).len() >= n - 1, "nothing closes before the turn");
    g.year().unwrap();
    let out = g.turn();
    eprintln!("{out:?}");
    let ev = g.sim.events.iter().rev().find(|e| e.kind == "hand").unwrap();
    eprintln!("{}\n  {}", ev.title, ev.text);
    assert!(ev.title.starts_with("By hand: the works making") && ev.title.contains("no more may be built there"), "{}", ev.title);
    assert!(edits::kind_works(&g.sim.st, &C, t, g_).is_empty(), "none stands after the turn");
    assert!((0..C.M).filter(|&m| C.method_out[m] == g_).all(|m| edits::forbidden(&g.sim.st, t, m)));
    for _ in 0..4 {
        g.year().unwrap();
        g.turn();
    }
    assert!(edits::kind_works(&g.sim.st, &C, t, g_).is_empty(), "and none is founded in four years");
    assert!(drift(&g) < 1e-6);
    // a second time there is nothing to do, and an unknown good is refused
    let why = g.clone().give(by_hand(edit)).unwrap_err();
    assert!(why.contains("every way of making it is turned off there already"), "{why}");
    let why = g.give(by_hand(Edit::CloseKind { town: t, good: "moonshine".into() })).unwrap_err();
    assert!(why.contains("no good called"), "{why}");
}

#[test]
fn the_panels_offers_are_the_trays_edits_and_a_session_with_them_plays_again_the_same() {
    let mut g = at(40);
    let t = biggest(&g);
    let mk = making(&g, t);
    // "Build another here": the very works the tray lists on that square, at the same cost and return
    let (kind, offer) = mk.hand.iter().find_map(|k| k.build.as_ref().ok().map(|o| (k.clone(), o.clone()))).expect("a works offered");
    let listed = edits::works_offered(&g.sim, offer.cell).unwrap();
    let o = listed.iter().find(|o| o.name == offer.method).unwrap();
    assert!(o.allowed.is_ok());
    let sums = o.sums.as_ref().unwrap();
    assert_eq!((sums.capital_cost, sums.expected_return), (offer.cost, offer.ret));
    eprintln!("build another {} here: {} on square {}, {:.0}, {:.0}% a year", g.sim.content.good_label[kind.good], offer.label, offer.cell, offer.cost, 100.0 * offer.ret);
    // and no square of the district offers that method a better return
    let best = g.sim.st.town_cells[t]
        .iter()
        .filter_map(|&c| edits::works_offered(&g.sim, c).ok())
        .filter_map(|l| l.into_iter().find(|o| o.name == offer.method && o.allowed.is_ok()))
        .filter_map(|o| o.sums.ok().map(|s| s.expected_return))
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(best <= offer.ret + 1e-12, "a square of the district offers {best} against {}", offer.ret);

    // the panel's three kinds of edit, as it sends them: built at the turn, a method off, one closed in mid-year
    let ask_check = save::check(&g.sim);
    let _ = making(&g, t);
    assert_eq!(save::check(&g.sim), ask_check, "asking the making changes nothing");
    let build = Edit::BuildWorks { method: offer.method.clone(), cell: offer.cell };
    g.give(by_hand(build.clone())).unwrap();
    let off = mk.hand.iter().flat_map(|k| k.methods.iter()).find(|x| x.standing > 0).unwrap().clone();
    let forbid = Edit::Forbid { town: t, method: off.name.clone() };
    g.give(by_hand(forbid.clone())).unwrap();
    for _ in 0..2 {
        g.month().unwrap();
    }
    let mk = making(&g, t);
    let close = Edit::CloseWorks { works: mk.hand.iter().find(|k| !k.standing.is_empty()).unwrap().standing[0].works };
    assert_eq!(g.give(by_hand(close.clone())).unwrap(), Done::Waits);
    g.year().unwrap();
    g.turn();
    g.year().unwrap();
    g.turn();
    assert_eq!(g.sim.events.iter().filter(|e| e.kind == "hand").count(), 3);

    // saved and loaded, the same to the last digit
    let b = save::load(&Save::from_text(&g.save(2_000).to_text()).unwrap(), |_| {}).unwrap();
    assert!(!b.drifted && b.refused.is_empty(), "{:?}", b.refused);
    assert_eq!(save::check(&b.game.sim), save::check(&g.sim));
    assert_eq!(b.game.sim.st.by_hand, g.sim.st.by_hand);

    // a run with no window given the same edits, dated as the log has them
    let mut P = save::params(Rules::Game, 7, 6);
    P.end_year = Some(1791);
    let list = vec![
        Dated { before: (1790, 1), edit: build },
        Dated { before: (1790, 1), edit: forbid },
        Dated { before: (1791, 1), edit: close },
    ];
    let run = edits::play(&P, Rules::Game.content(), &list).unwrap();
    assert_eq!(run.year, g.sim.year);
    assert_eq!(save::check(&run), save::check(&g.sim), "the panel's edits and the same list played with no window give the same island");
}
