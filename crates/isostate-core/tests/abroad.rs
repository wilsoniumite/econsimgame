//! Ideas by sea (`docs/rules/abroad.md`; not in the Python): every port has
//! abroad for one more neighbour, which knows each idea from the year
//! `data/abroad.toml` gives. Off under both sets of rules; on with the landfall
//! start.
#![allow(non_snake_case)]

use isostate_core::*;

/// The game's content as a run is played on it: the tree's overlay laid on.
fn content() -> Content {
    tree::content(&game_params(), game_content()).unwrap()
}

#[test]
fn the_table_has_not_gone_stale_against_the_content() {
    // Every idea of the tree not known in 1750, and every use of the tasks rule,
    // has a line in the table or a comment saying abroad does not know it; every
    // line names an idea or a use the content has; and every idea or use with
    // history's year (`target`) is known abroad from that year, never before its
    // `found_after`. When this fails, make the table again (examples/abroad_table.rs).
    let C = content();
    let table = abroad::table();
    let unknown: Vec<String> =
        abroad::ABROAD_TOML.lines().filter_map(|l| l.strip_prefix("# ")).filter_map(|l| l.split_once(": not known abroad")).map(|(n, _)| n.to_string()).collect();
    for t in (0..C.NT).filter(|&t| !C.tech_known[t]) {
        let name = &C.tech_name[t];
        let after = C.tech_after.get(t).copied().unwrap_or(0);
        match (table.idea.get(name), C.tech_target.get(t).copied().filter(|&y| y > 0)) {
            (Some(&y), Some(target)) => assert_eq!(y, target.max(after), "{name}: the table says {y}, the content's target {target} (found after {after})"),
            (Some(&y), None) => assert!(y >= after, "{name}: the table says {y}, before the content's first year {after}"),
            (None, Some(target)) => panic!("{name} has history's year {target} and no line in data/abroad.toml"),
            (None, None) => assert!(unknown.contains(name), "{name} is in the content and not in data/abroad.toml: make the table again"),
        }
    }
    for u in &C.tasks.uses {
        match (table.uses.get(&u.name), u.target) {
            (Some(&y), Some(target)) => assert_eq!(y, target.max(u.found_after), "use {}: the table says {y}, its target {target}", u.name),
            (Some(&y), None) => assert!(y >= u.found_after, "use {}: the table says {y}, before {}", u.name, u.found_after),
            (None, Some(target)) => panic!("use {} has history's year {target} and no line in data/abroad.toml", u.name),
            (None, None) => assert!(unknown.contains(&u.name), "use {} is not in data/abroad.toml: make the table again", u.name),
        }
    }
    for name in table.idea.keys() {
        assert!(C.tech_name.iter().position(|n| n == name).is_some_and(|t| !C.tech_known[t]), "data/abroad.toml names {name}, which is no idea to find");
    }
    for name in table.uses.keys() {
        assert!(C.tasks.uses.iter().any(|u| &u.name == name), "data/abroad.toml names the use {name}, which the tasks rule has not");
    }
}

#[test]
fn off_under_both_sets_of_rules_and_on_with_the_landfall_start() {
    for P in [game_params(), default_params()] {
        assert!(!P.abroad.ideas);
        assert!(!serde_json::to_string(&P).unwrap().contains("\"abroad\""), "abroad is written out while it is off");
    }
    let mut P = game_params();
    landfall::set_start(&mut P, "landfall").unwrap();
    assert!(P.abroad.ideas);
    // off, the sea brings nothing and no use comes by it
    let C = content();
    let off = game_params();
    let sim = sim::start(&{ let mut p = off.clone(); p.end_year = Some(1751); p }, game_content());
    assert!(abroad::sea(&sim.st, &C, &off, 1900).is_none());
    assert!((0..C.tasks.uses.len()).all(|u| !abroad::use_comes(&C, &off, u, 2050)));
    // on, a use comes `lag` years after abroad has it, and not a year before
    for u in 0..C.tasks.uses.len() {
        if let Some(y) = abroad::use_year(&C, u) {
            assert!(!abroad::use_comes(&C, &P, u, y + P.abroad.lag - 1) && abroad::use_comes(&C, &P, u, y + P.abroad.lag));
        }
    }
}

#[test]
fn abroad_knows_an_idea_as_a_town_that_found_it_alone() {
    // nothing before its year, a quarter in it, then the logistic of a town with
    // nobody to learn from (dk = diffusion_rate (1 - k) k), to nearly all of it
    let P = game_params();
    assert_eq!(abroad::know(&P, 1800, 1799), 0.0);
    assert!((abroad::know(&P, 1800, 1800) - 0.25).abs() < 1e-12);
    let mut was = 0.25;
    for y in 1801..1840 {
        let k = abroad::know(&P, 1800, y);
        assert!(k > was && k < 1.0);
        was = k;
    }
    assert!(was > 0.999);
    // the step of the logistic is the rule's own step, to within a hundredth
    let (k0, k1) = (abroad::know(&P, 1800, 1803), abroad::know(&P, 1800, 1804));
    assert!((k1 - k0 - P.diffusion_rate * (1.0 - k0) * k0).abs() < 0.01);
}

#[test]
fn a_sea_of_no_weight_spreads_ideas_as_the_island_alone() {
    // with abroad weighing nothing at every town and knowing nothing, the spreading
    // is the Python's, to the bit (seed 7 under the game's rules, played to 1760)
    let mut P = game_params();
    P.seed = 7;
    P.end_year = Some(1760);
    let mut sim = sim::start(&P, game_content());
    while sim.year <= sim.end_year {
        sim::run_year(&mut sim).unwrap();
    }
    let C = sim.content.clone();
    let mut a = sim.st.clone();
    let mut b = sim.st.clone();
    tech::diffusion(&mut a, &C, &P);
    let sea = abroad::Sea { weight: vec![0.0; b.T_max], know: vec![0.0; C.NT] };
    tech::diffusion_with(&mut b, &C, &P, Some(&sea));
    assert_eq!(a.know.data, b.know.data);
}

#[test]
fn at_landfall_ideas_come_by_sea_to_the_port() {
    // played under --start landfall (frontier.enabled, frontier.hamlets,
    // immigration.enabled) with ideas by sea (abroad.ideas) on
    // and off, island 7 to 1790, six families: with the sea, ideas abroad knew
    // come to a port, never before abroad knew them, and are news
    let mut P = game_params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(1790);
    landfall::set_start(&mut P, "landfall").unwrap();
    assert!(P.abroad.ideas && P.immigration.enabled && P.frontier.enabled);
    let play = |P: &Params| {
        let mut sim = sim::start(P, game_content());
        while sim.year <= sim.end_year {
            sim::run_year(&mut sim).expect("the money check holds");
        }
        sim
    };
    let on = play(&P);
    let mut alone = P.clone();
    alone.abroad.ideas = false;
    let off = play(&alone);
    let C = &on.content;
    let by_sea: Vec<&state::Event> = on.events.iter().filter(|e| e.kind == "discovery" && e.title.contains("comes by sea")).collect();
    assert!(!by_sea.is_empty(), "no idea came by sea by 1790");
    for e in &by_sea {
        let t = e.tech.unwrap();
        let town = e.town.unwrap();
        let since = abroad::idea_year(C, t).expect("came by sea, so known abroad");
        assert!(on.st.tech_year[t] >= since, "{} came in {}, before abroad knew it in {since}", C.tech_name[t], on.st.tech_year[t]);
        assert!(on.st.world.coast[on.st.town_node[town]] || on.st.tech_town[t] == town as i64);
    }
    // and the colony with the sea knows more ideas than the colony alone
    let found = |s: &sim::Sim| (0..C.NT).filter(|&t| !C.tech_known[t] && s.st.tech_year[t] >= 0).count();
    assert!(found(&on) > found(&off), "with the sea {} ideas, alone {}", found(&on), found(&off));
}
