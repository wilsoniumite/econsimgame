//! What the simulation knows of a thing (`inspect::tell`, docs/designs/sandbox.md
//! section 6.2): on island 7, played as the viewer plays a game (the game's rules,
//! six families), at the turn into 1760 and five months into 1790, every town
//! that stands, forty works and two hundred squares are told without a figure
//! that is empty, NaN or infinite; a town's people and its three purses read as
//! the state's own figures put through the same function; and a thing that is
//! not there is refused in a sentence. The island's "By hand" rows read the
//! hand's ledger, "none yet" before any edit; and a question and its card can
//! cross from one thread to another. A town's card has every idea of the data
//! once, in the state the town's knowledge gives it, in 1790 and in 1821, and
//! one it says may be found there by hand is, by the hand's edit for an idea.
//!
//! Nothing here leans on what becomes of the island: whatever towns, works and
//! squares there are, each is asked about. Which groups a card has follows the
//! rules' own switches (the harvest's, the books'), read from the run's params,
//! so a later rule that changes the island's history changes nothing here.
#![allow(non_snake_case)]

use isostate_core::econ;
use isostate_core::edits::{Edit, Purse};
use isostate_core::inspect::{KNOWS, Learnt, Stands, Thing, Told, tell, tell_with};
use isostate_core::save::{Game, Order};
use isostate_core::sim::Sim;
use isostate_core::util::{fmt_pct, fmt_thousands_f0};

// The viewer asks on its own thread and the simulation answers on another: a
// question and its card must both be `Send`, and a question must serve as a
// key (`Copy`, `Eq`, `Hash`). This does not compile if either is lost.
const _: () = {
    const fn sendable<T: Send>() {}
    const fn key<T: Copy + Eq + std::hash::Hash>() {}
    sendable::<Thing>();
    sendable::<Told>();
    key::<Thing>();
};

/// Every row has a name and a figure, and no figure is NaN or infinite.
fn sound(told: &Told, what: &str) {
    assert!(!told.title.is_empty(), "{what}: no title");
    assert!(!told.as_of.is_empty(), "{what}: no date");
    assert!(!told.groups.is_empty(), "{what}: no groups");
    for g in &told.groups {
        assert!(!g.name.is_empty(), "{what}: a group with no name");
        assert!(!g.rows.is_empty(), "{what}: group {} is empty", g.name);
        for (name, figure) in &g.rows {
            assert!(!name.trim().is_empty(), "{what}, {}: a row with no name ({figure})", g.name);
            assert!(!figure.trim().is_empty(), "{what}, {}: {name} has no figure", g.name);
            // by words, not letters: a works may be "Tennant, Carpenter and Builder"
            for word in figure.split(|c: char| !c.is_alphanumeric()) {
                assert!(!["NaN", "nan", "inf"].contains(&word), "{what}, {}: {name} reads {figure:?}", g.name);
            }
        }
    }
}

fn row<'a>(told: &'a Told, group: &str, name: &str) -> &'a str {
    let g = told.groups.iter().find(|g| g.name == group).unwrap_or_else(|| panic!("{}: no group {group}", told.title));
    g.rows.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str()).unwrap_or_else(|| panic!("{}: no row {name} in {group}", told.title))
}

fn has_group(told: &Told, prefix: &str) -> bool {
    told.groups.iter().any(|g| g.name.starts_with(prefix))
}

/// A refusal is a sentence: it begins with a capital and ends with a full stop.
fn refused(sim: &Sim, thing: Thing) {
    match tell(sim, thing) {
        Ok(t) => panic!("{thing:?} was told ({}), not refused", t.title),
        Err(e) => {
            assert!(e.chars().next().is_some_and(|c| c.is_uppercase()) && e.ends_with('.'), "{thing:?}: refused as {e:?}");
        }
    }
}

fn check(sim: &Sim, as_of: &str) {
    let (st, P) = (&sim.st, &sim.params);

    // every town that stands
    let towns: Vec<usize> = (0..st.town_name.len()).filter(|&t| st.town_active[t]).collect();
    assert!(!towns.is_empty());
    for &t in &towns {
        let told = tell(sim, Thing::Town(t)).unwrap_or_else(|e| panic!("{}: {e}", st.town_name[t]));
        sound(&told, &st.town_name[t]);
        assert_eq!(told.title, st.town_name[t]);
        assert_eq!(told.as_of, as_of);
        assert_eq!(row(&told, "People", "people in its market area"), fmt_thousands_f0(st.pop[t]));
        assert_eq!(row(&told, "People", "people in the town"), fmt_thousands_f0(st.urban_pop[t]));
        assert_eq!(row(&told, "Money and prices", "the households' money"), fmt_thousands_f0(st.hh_cash[t]));
        assert_eq!(row(&told, "Money and prices", "the landlords' money"), fmt_thousands_f0(st.lord_cash[t]));
        assert_eq!(row(&told, "Money and prices", "the merchants' money"), fmt_thousands_f0(st.merch_cash[t]));
        assert_eq!(has_group(&told, "This year's weather"), P.harvest.enabled, "{}: the weather follows its rule", told.title);
        assert_eq!(has_group(&told, "The year's trade"), P.accounts.enabled && st.acct.hist.n > 0, "{}: the trade follows the books", told.title);
    }

    // forty works, spread over those standing
    let standing: Vec<usize> = (0..st.F_count).filter(|&f| st.f_alive[f]).collect();
    assert!(!standing.is_empty());
    let step = (standing.len() / 40).max(1);
    let mut n = 0;
    for &f in standing.iter().step_by(step).take(40) {
        let told = tell(sim, Thing::Works(f)).unwrap_or_else(|e| panic!("works {f}: {e}"));
        sound(&told, &st.f_name[f]);
        assert_eq!(told.title, st.f_name[f]);
        assert_eq!(row(&told, "Money", "cash"), fmt_thousands_f0(st.f_cash[f]));
        n += 1;
    }
    assert_eq!(n, standing.len().min(40));

    // two hundred squares of land, spread over the island
    let land: Vec<usize> = (0..st.N).filter(|&c| st.world.land[c]).collect();
    let step = (land.len() / 200).max(1);
    for &c in land.iter().step_by(step).take(200) {
        let told = tell(sim, Thing::Cell(c)).unwrap_or_else(|e| panic!("square {c}: {e}"));
        sound(&told, &told.title);
        let figure = row(&told, "Land", "fertility");
        assert!(figure == "none" || ["poor", "fair", "rich"].iter().any(|w| figure.ends_with(w)), "{}: fertility reads {figure}", told.title);
    }

    // the island
    let told = tell(sim, Thing::Island).unwrap();
    sound(&told, "the island");
    assert_eq!(row(&told, "People", "market towns"), fmt_thousands_f0(towns.len() as f64));
    assert_eq!(row(&told, "Works", "standing"), fmt_thousands_f0(standing.len() as f64));

    // what is not there
    refused(sim, Thing::Town(st.town_name.len()));
    refused(sim, Thing::Town(usize::MAX));
    if let Some(t) = (0..st.town_name.len()).find(|&t| !st.town_active[t]) {
        refused(sim, Thing::Town(t));
    }
    refused(sim, Thing::Works(st.F_count));
    if let Some(f) = (0..st.F_count).find(|&f| !st.f_alive[f]) {
        refused(sim, Thing::Works(f));
    }
    refused(sim, Thing::Cell(st.N));
    let sea = (0..st.N).find(|&c| !st.world.land[c]).expect("the island has a shore");
    refused(sim, Thing::Cell(sea));
}

#[test]
fn every_thing_is_told_and_what_is_not_there_is_refused() {
    let mut g = Game::new(7, 6, 0);
    while g.sim.year < 1760 {
        g.year().unwrap();
        g.turn();
    }
    check(&g.sim, "January 1760");
    while g.sim.year < 1790 {
        g.year().unwrap();
        g.turn();
    }
    for _ in 0..5 {
        g.month().unwrap();
    }
    check(&g.sim, "June 1790");
    // five months of the year played: a works' profit so far says so
    let f = (0..g.sim.st.F_count).find(|&f| g.sim.st.f_alive[f]).unwrap();
    let told = tell(&g.sim, Thing::Works(f)).unwrap();
    assert!(row(&told, "Money", "profit so far this year").ends_with("in 5 months"));
}

#[test]
fn telling_changes_nothing() {
    // two games alike; one is told of everything every year, and both stand the same after
    let (mut a, mut b) = (Game::new(7, 6, 0), Game::new(7, 6, 0));
    for _ in 0..12 {
        let st = &a.sim.st;
        for t in 0..st.town_name.len() {
            let _ = tell(&a.sim, Thing::Town(t));
        }
        for f in 0..st.F_count {
            let _ = tell(&a.sim, Thing::Works(f));
        }
        for c in (0..st.N).step_by(37) {
            let _ = tell(&a.sim, Thing::Cell(c));
        }
        let _ = tell(&a.sim, Thing::Island);
        a.year().unwrap();
        a.turn();
        b.year().unwrap();
        b.turn();
    }
    let bits = |g: &Game| g.sim.st.series.data.iter().map(|d| d.iter().map(|x| x.to_bits()).collect::<Vec<u64>>()).collect::<Vec<_>>();
    assert!(bits(&a) == bits(&b), "a game that was told of things went on differently");
}

/// Every idea of the data, once, in the one state the state's own figures
/// give it, with the years in order.
fn ideas_sound(g: &Game, learnt: &Learnt) {
    let (sim, st, C) = (&g.sim, &g.sim.st, &g.sim.content);
    let towns: Vec<usize> = (0..st.town_name.len()).filter(|&t| st.town_active[t]).collect();
    for &t in &towns {
        let told = tell_with(sim, Thing::Town(t), Some(learnt)).unwrap();
        let ideas = told.ideas.as_ref().unwrap_or_else(|| panic!("{}: a town's card has its ideas", st.town_name[t]));
        assert_eq!((ideas.town, ideas.from, ideas.to), (t, C.start_year, sim.year));
        let mut names: Vec<&str> = ideas.all().map(|i| i.name.as_str()).collect();
        names.sort_unstable();
        let mut data: Vec<&str> = C.tech_name.iter().map(String::as_str).chain(C.tasks.uses.iter().map(|u| u.name.as_str())).collect();
        data.sort_unstable();
        assert_eq!(names, data, "{}: every idea of the data, once", st.town_name[t]);
        for i in ideas.all() {
            assert!(!i.said.is_empty() && !i.how.is_empty() && !i.said.contains("NaN") && !i.said.contains("inf "), "{}: {} says {:?}", st.town_name[t], i.name, i.said);
            let want = match C.tech_name.iter().position(|n| *n == i.name) {
                Some(k) if st.know[(t, k)] > KNOWS => Stands::Here,
                Some(k) if C.tech_known[k] || st.tech_year[k] >= 0 => Stands::Elsewhere,
                Some(_) => Stands::Nowhere,
                None => {
                    let u = C.tasks.uses.iter().position(|u| u.name == i.name).unwrap();
                    if st.tasks.found.get(u).is_some_and(|&y| y >= 0) { Stands::Here } else { Stands::Nowhere }
                }
            };
            assert_eq!(i.stands, want, "{}: {}", st.town_name[t], i.name);
            // only an idea of the tree found nowhere may be found here by hand
            assert_eq!(i.discover, want == Stands::Nowhere && i.kind != "use", "{}: {}", st.town_name[t], i.name);
            match i.stands {
                Stands::Here => assert!(i.year.is_some_and(|y| y >= C.start_year && y <= sim.year), "{}: {} known here in {:?}", st.town_name[t], i.name, i.year),
                Stands::Elsewhere => assert!(i.year.is_some()),
                Stands::Nowhere => assert_eq!(i.year, None),
            }
        }
    }
}

#[test]
fn a_towns_ideas_are_every_idea_in_one_of_three_states_and_one_found_by_hand_is_known_there() {
    // island 7, six families, the game's rules as `Game::new` sets them (the
    // tree's overlay and the tasks rule give the ideas); nothing here leans on
    // which ideas the island finds when: each is checked against the state's
    // own knowledge as it stands, in 1790 and in 1821
    let mut g = Game::new(7, 6, 0);
    let mut learnt = Learnt::default();
    while g.sim.year < 1821 {
        g.year().unwrap();
        learnt.year_end(&g.sim);
        g.turn();
        if g.sim.year == 1790 {
            ideas_sound(&g, &learnt);
        }
    }
    ideas_sound(&g, &learnt);
    // without what was watched, a town's ideas stand the same, and only the years (and so the
    // order within a group) may be missing
    let st = &g.sim.st;
    let t = (0..st.T_max).filter(|&t| st.town_active[t]).max_by(|&a, &b| st.pop[a].total_cmp(&st.pop[b])).unwrap();
    let (with, without) = (tell_with(&g.sim, Thing::Town(t), Some(&learnt)).unwrap(), tell(&g.sim, Thing::Town(t)).unwrap());
    let states = |told: &Told| {
        let mut s = told.ideas.as_ref().unwrap().all().map(|i| (i.name.clone(), i.stands, i.discover)).collect::<Vec<_>>();
        s.sort_by(|a, b| a.0.cmp(&b.0));
        s
    };
    assert_eq!(states(&with), states(&without));

    // the hand's edit for an idea, on one the card says may be found here, is carried out, and the
    // card then has it known here, found here by hand in the year in progress
    let ideas = with.ideas.unwrap();
    let idea = ideas.all().find(|i| i.discover).expect("in 1821 some idea of the tree is found nowhere yet").clone();
    let line = g.give(Order::ByHand { edit: Edit::Discover { tech: idea.name.clone(), town: t } }).map(|_| g.sim.events.last().unwrap().title.clone()).unwrap();
    println!("{line}");
    let after = tell_with(&g.sim, Thing::Town(t), Some(&learnt)).unwrap().ideas.unwrap();
    let now = after.all().find(|i| i.name == idea.name).unwrap();
    assert_eq!((now.stands, now.year, now.discover), (Stands::Here, Some(1821), false));
    assert!(now.how.starts_with("found here in 1821, by hand"), "{}", now.how);
    println!("{}", now.said);
}

#[test]
fn the_island_tells_what_the_hand_has_done() {
    // island 7, six families, the game's rules as `Game::new` sets them; the
    // ledger is kept under every rule, so nothing here leans on which rules are
    // on or on what becomes of the island: whatever its money and its towns,
    // the rows must read the ledger's own figures
    let mut g = Game::new(7, 6, 0);
    while g.sim.year < 1760 {
        g.year().unwrap();
        g.turn();
    }
    let told = tell(&g.sim, Thing::Island).unwrap();
    for name in ["edits carried out", "money made by hand", "people made by hand"] {
        assert_eq!(row(&told, "By hand", name), "none yet");
    }

    // at the turn into 1760: every household purse doubled, and the largest
    // town's people halved
    let st = &g.sim.st;
    let town = (0..st.T_max).filter(|&t| st.town_active[t]).max_by(|&a, &b| st.pop[a].total_cmp(&st.pop[b])).unwrap();
    g.give(Order::ByHand { edit: Edit::Money { purse: Purse::Households, town: None, factor: 2.0 } }).unwrap();
    g.give(Order::ByHand { edit: Edit::People { town, factor: 0.5 } }).unwrap();
    let st = &g.sim.st;
    assert_eq!(st.by_hand.count, 2);
    assert!(st.by_hand.money > 0.0 && st.by_hand.people < 0.0);

    let told = tell(&g.sim, Thing::Island).unwrap();
    sound(&told, "the island after two edits");
    assert_eq!(row(&told, "By hand", "edits carried out"), "2");
    let share = fmt_pct(st.by_hand.money / econ::money_total(st), 0);
    assert_eq!(row(&told, "By hand", "money made by hand"), format!("+{}, {share} of the island's money", fmt_thousands_f0(st.by_hand.money)));
    assert_eq!(row(&told, "By hand", "people made by hand"), fmt_thousands_f0(st.by_hand.people));
    for name in ["edits carried out", "money made by hand", "people made by hand"] {
        println!("{name}: {}", row(&told, "By hand", name));
    }

    // a question is a key, and a card made on one thread is read on another
    let mut asked = std::collections::HashSet::new();
    asked.insert(Thing::Island);
    asked.insert(Thing::Town(town));
    assert!(asked.contains(&Thing::Island));
    let card = std::thread::spawn(move || told).join().unwrap();
    assert_eq!(card.title, "The island");
}
