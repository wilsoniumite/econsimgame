//! IDEAS-PLATE: the island's ideas as a timeline (`island_ideas.rs`, the
//! viewer's panel of the island's ideas; asked for as `inspect::Thing::Ideas`).
//!
//! - Every idea of the data the run plays has one row: each method and
//!   technique of the tree (those known in 1750 among them) and each use of
//!   the tasks rule, and nothing else.
//! - The years on the panel are the game's own finds: island 7 played to 1830
//!   with the thread's watch of the towns (`inspect::Learnt`) gives, row for
//!   row, the year and town of every find and the year every town came to know
//!   each idea that the atlas's third plate notes of the same island
//!   (`atlas_ideas::one_island`), which watches the run for itself.
//! - An idea held back is not found before its year: the railway, which island
//!   7 finds before 1880 untouched, is held back from 1800 until 1880 and is
//!   not found before it.
//! - A drop offers nothing that does nothing, and says why nothing can be done
//!   to a find.
//!
//! Played under the game's rules with six families, as `atlas ideas` plays
//! them. The second case leans on nothing of the island's history; the third
//! leans on when island 7 finds the railway untouched under today's rules
//! (it asserts it), and plays with `tasks.enabled` and `woods.enabled` as the
//! game's rules have them, since neither the railway nor its thresholds are
//! theirs.
#![allow(non_snake_case)]

use isostate_core::atlas_ideas;
use isostate_core::edits::{self, Dated, Edit, PinWhen};
use isostate_core::inspect::{Learnt, Thing, tell_with};
use isostate_core::island_ideas::{IslandIdeas, drop_choices};
use isostate_core::params::{Params, Rules};
use isostate_core::sim::Sim;

fn game(end: i64) -> (Params, isostate_core::content::Content) {
    let mut P = Rules::Game.params();
    P.n_families = 6;
    P.seed = 7;
    P.end_year = Some(end);
    (P, Rules::Game.content())
}

fn play(P: &Params, content: isostate_core::content::Content, list: &[Dated]) -> (Sim, Learnt) {
    let mut learnt = Learnt::default();
    let sim = edits::play_noting(P, content, list, |s, m| {
        if m.is_none() {
            learnt.year_end(s);
        }
    })
    .expect("the island plays");
    (sim, learnt)
}

/// The island as it stands in January of `until`, in a run to `end`, watched as the thread watches it.
fn to_january(end: i64, until: i64) -> (Sim, Learnt) {
    let (P, content) = game(end);
    let mut sim = isostate_core::sim::start(&P, content);
    sim.check_money = true;
    let mut learnt = Learnt::default();
    while sim.year < until {
        for month in 0..12 {
            isostate_core::sim::run_month(&mut sim, month).expect("a month plays");
        }
        isostate_core::sim::end_year(&mut sim);
        learnt.year_end(&sim);
    }
    (sim, learnt)
}

fn ideas(sim: &Sim, learnt: &Learnt) -> IslandIdeas {
    tell_with(sim, Thing::Ideas, Some(learnt)).expect("the island's ideas are told").timeline.expect("a timeline")
}

#[test]
fn every_idea_has_a_row_and_the_years_are_the_games_own_finds() {
    let (P, content) = game(1830);
    let (sim, learnt) = play(&P, content.clone(), &[]);
    let (st, C) = (&sim.st, &sim.content);
    let t0 = std::time::Instant::now();
    for _ in 0..50 {
        let _ = ideas(&sim, &learnt);
    }
    eprintln!("the island's ideas told in {:.3} ms on the mean of 50", t0.elapsed().as_secs_f64() * 1e3 / 50.0);
    let i = ideas(&sim, &learnt);
    assert_eq!((i.from, i.now, i.end), (1750, sim.year, 1830));

    // a row for every idea of the data, and for nothing else
    // (a machine that tends is a use with the name of the idea of the tree it waits on: two ideas, two rows)
    assert_eq!(i.rows().count(), C.NT + C.tasks.uses.len(), "one row an idea: {} of the tree and {} uses", C.NT, C.tasks.uses.len());
    for n in &C.tech_name {
        assert_eq!(i.rows().filter(|r| r.kind != "use" && r.name == *n).count(), 1, "{n} has one row");
    }
    for u in &C.tasks.uses {
        assert_eq!(i.rows().filter(|r| r.kind == "use" && r.name == u.name).count(), 1, "{} has one row", u.name);
    }
    assert!(i.groups.iter().all(|g| !g.rows.is_empty() && !g.name.is_empty()));
    assert_eq!(i.groups.last().map(|g| g.name.as_str()), Some("Machines for tasks"));

    // the atlas's third plate, watching the same island for itself
    let (isl, _) = atlas_ideas::one_island(&P, content, &[], 7).expect("the atlas plays the island");
    let mut found = 0;
    for k in 0..C.NT {
        let r = i.row(&C.tech_name[k]).unwrap();
        assert_eq!(r.found, isl.found[k], "{}: the year found", r.name);
        assert_eq!(r.town.as_ref().map(|t| t.0), isl.town[k], "{}: the town found in", r.name);
        assert_eq!(r.at_start, C.tech_known[k]);
        assert_eq!(r.target, C.tech_target[k].gt(&0).then_some(C.tech_target[k]));
        if r.found.is_some() && !r.at_start {
            found += 1;
            assert!(r.drag.is_err(), "{}: a find cannot be dragged", r.name);
            // every town that knows it, with the year the atlas saw it come to know it
            for (name, y) in &r.learnt {
                let t = isl.towns.iter().find(|t| &t.name == name && st.town_active[t.index]).unwrap_or_else(|| panic!("{name}"));
                assert_eq!(*y, t.knew[k], "{}: the year {name} came to know it", r.name);
            }
            let knowing = isl.towns.iter().filter(|t| st.town_active[t.index] && st.know[(t.index, k)] > atlas_ideas::KNOWS).count();
            assert_eq!(r.learnt.len(), knowing, "{}: the towns that know it", r.name);
        } else if r.found.is_none() {
            assert!(!r.waits.is_empty(), "{}: says what it waits on", r.name);
        }
    }
    assert!(found >= 5, "island 7 finds several ideas by 1830 ({found})");
    for (u, us) in C.tasks.uses.iter().enumerate() {
        assert_eq!(i.use_row(&us.name).unwrap().found, isl.use_found[u], "{}", us.name);
    }
}

#[test]
fn an_idea_held_back_is_not_found_before_its_year() {
    let (P, content) = game(1885);
    assert!(P.tasks.enabled && P.woods.enabled, "the case is played with the game's rules as they stand");
    let (sim, learnt) = play(&P, content.clone(), &[]);
    // (by its place in the content the run plays, with the tree laid over it)
    let rail = sim.content.tech_name.iter().position(|n| n == "railway").expect("the railway is an idea of the data");
    let untouched = sim.st.tech_year[rail];
    assert!((1750..1880).contains(&untouched), "untouched, island 7 finds the railway before 1880 ({untouched})");

    // the panel's drop at 1880, as the island stood in January 1800
    let (at, l0) = to_january(1885, 1800);
    let row = ideas(&at, &l0).row("railway").cloned().expect("a row for the railway");
    if at.st.tech_year[rail] >= 0 {
        panic!("island 7 has found the railway by 1800 ({}): the case wants it unfound", at.st.tech_year[rail]);
    }
    let offers = drop_choices(&row, 1880, 1885).expect("a drop on 1880 offers something");
    let hold = offers.iter().find(|o| o.label == "Hold it back until 1880").expect("a hold until 1880");
    assert_eq!(hold.edit, Edit::Pin { idea: "railway".into(), year: 1880, when: PinWhen::NotBefore });
    assert!(offers.iter().any(|o| o.label == "Find it in 1880"));

    let held = [Dated { before: (1800, 1), edit: hold.edit.clone() }];
    let (sim2, learnt2) = play(&P, content, &held);
    let y = sim2.st.tech_year[rail];
    assert!(y < 0 || y >= 1880, "held back until 1880, the railway is found in {y}");
    let r2 = ideas(&sim2, &learnt2);
    let row2 = r2.row("railway").unwrap();
    assert_eq!(row2.pin, Some((1880, PinWhen::NotBefore)), "the hold is on the row, with its year");
    assert_eq!(row2.found, (y >= 0).then_some(y));
    let _ = learnt;
}

#[test]
fn a_drop_offers_nothing_that_does_nothing() {
    let (sim, learnt) = to_january(2050, 1800);
    let i = ideas(&sim, &learnt);
    // a find, and an idea known in 1750: nothing to drag, and the sentence says so
    let start = i.rows().find(|r| r.at_start).expect("ideas known in 1750");
    assert!(drop_choices(start, 1820, i.end).unwrap_err().contains("known before"));
    if let Some(f) = i.rows().find(|r| r.found.is_some() && !r.at_start && r.kind != "use") {
        assert!(drop_choices(f, 1820, i.end).unwrap_err().contains("cannot be unmade but by taking it back"));
    }
    // an idea of the tree not yet found: now is found now; a year ahead is found or held then
    let r = i.rows().find(|r| r.kind != "use" && r.found.is_none() && r.drag.is_ok()).expect("an idea not yet found that may be dragged");
    let now = drop_choices(r, i.now, i.end).unwrap();
    assert_eq!(now.len(), 1);
    assert!(matches!(&now[0].edit, Edit::Discover { tech, .. } if *tech == r.name), "{:?}", now[0]);
    let ahead = drop_choices(r, i.now + 5, i.end).unwrap();
    assert_eq!(ahead.len(), 2);
    // pinned already to that year: that chip is not offered again
    let mut pinned = r.clone();
    pinned.pin = Some((i.now + 5, PinWhen::In));
    let again = drop_choices(&pinned, i.now + 5, i.end).unwrap();
    assert_eq!(again.len(), 1);
    assert!(matches!(again[0].edit, Edit::Pin { when: PinWhen::NotBefore, .. }));
    // a use is found at a year's turn, so the soonest is the turn into next year; a hold must hold something back
    if let Some(u) = i.rows().find(|r| r.kind == "use" && r.found.is_none() && r.drag.is_ok()) {
        let soon = drop_choices(u, i.now, i.end).unwrap();
        assert_eq!(soon.len(), 1);
        assert_eq!(soon[0].edit, Edit::Pin { idea: u.name.clone(), year: i.now + 1, when: PinWhen::In });
        let next = drop_choices(u, i.now + 2, i.end).unwrap();
        assert_eq!(next.len(), 2);
    }
    // a drop past the run's end is a drop on its last year
    let last = drop_choices(r, i.end + 40, i.end).unwrap();
    assert!(last.iter().all(|o| matches!(o.edit, Edit::Pin { year, .. } if year == i.end)));
}

#[test]
fn a_pin_by_drag_gives_the_island_of_the_same_dated_edit_from_the_command_line() {
    use isostate_core::save::{self, Game, Order, Save};
    // a game as the viewer plays one, two months into 1801
    let mut g = Game::new(7, 6, 1_000);
    let mut learnt = Learnt::default();
    for _ in 0..51 {
        g.year().unwrap();
        g.turn();
        learnt.year_end(&g.sim);
    }
    assert_eq!(g.sim.year, 1801);
    for _ in 0..2 {
        g.month().unwrap();
    }
    let ask = save::check(&g.sim);
    let i = ideas(&g.sim, &learnt);
    assert_eq!(save::check(&g.sim), ask, "asking for the island's ideas changes nothing");
    // the first idea of the tree not yet found, dropped on 1880 and held back
    let row = i.rows().find(|r| r.kind != "use" && r.found.is_none() && r.drag.is_ok()).expect("an idea of the tree not yet found").clone();
    let offers = drop_choices(&row, 1880, i.end).unwrap();
    let hold = offers.iter().find(|o| matches!(o.edit, Edit::Pin { when: PinWhen::NotBefore, .. })).unwrap().edit.clone();
    eprintln!("{} held back until 1880 in March 1801", row.label);
    g.give(Order::ByHand { edit: hold.clone() }).unwrap();
    assert_eq!(ideas(&g.sim, &learnt).row(&row.name).unwrap().pin, Some((1880, PinWhen::NotBefore)));
    // and another found now, by a drop on this year
    let now_row = i.rows().find(|r| r.kind != "use" && r.found.is_none() && r.drag.is_ok() && r.name != row.name).expect("a second idea not yet found").clone();
    let find = drop_choices(&now_row, i.now, i.end).unwrap()[0].edit.clone();
    assert!(matches!(find, Edit::Discover { .. }), "{find:?}");
    g.give(Order::ByHand { edit: find.clone() }).unwrap();
    let k = g.sim.content.tech_name.iter().position(|n| *n == now_row.name).unwrap();
    assert_eq!(g.sim.st.tech_year[k], 1801, "{} found now", now_row.label);
    let found = ideas(&g.sim, &learnt);
    let r = found.row(&now_row.name).unwrap();
    assert_eq!((r.found, r.by_hand), (Some(1801), true), "the row says it was found by hand");
    for _ in 0..10 {
        g.year().unwrap();
        g.turn();
    }
    assert_eq!(g.sim.year, 1811);
    // saved and loaded, the same; and a run with no window given the same edits, dated as the log has them
    let b = save::load(&Save::from_text(&g.save(2_000).to_text()).unwrap(), |_| {}).unwrap();
    assert!(!b.drifted && b.refused.is_empty());
    assert_eq!(save::check(&b.game.sim), save::check(&g.sim));
    let mut P = save::params(Rules::Game, 7, 6);
    P.end_year = Some(1810);
    let list = vec![Dated { before: (1801, 3), edit: hold }, Dated { before: (1801, 3), edit: find }];
    let run = edits::play(&P, Rules::Game.content(), &list).unwrap();
    assert_eq!(run.year, g.sim.year);
    assert_eq!(save::check(&run), save::check(&g.sim), "the drags' edits and the same list played with no window give the same island");
}
