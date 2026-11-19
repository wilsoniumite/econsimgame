//! Hands and machines in the works (not in the Python; docs/rules/tasks.md).
//! With the switch off the golden test (tests/golden.rs) covers that the port
//! did not move.
//!
//! The island runs here are played on island 7 to 1925 (and to 1949 for the
//! ceiling on yield) under the game's rules with six families, with the
//! switches they lean on set by name (`tasks.enabled`, `tasks.land`,
//! `tasks.yield_cap`) and every other rule as the game has it. What they check
//! holds on any island's history (a run against another run, a new works
//! against its town, shares within their bounds, money that adds up), so a
//! later rule that changes island 7's history cannot break them; only one
//! that stopped the hoist being found by 1925 would, and the test says so.
#![allow(non_snake_case)]

use isostate_core::tasks::{self, TasksParams};
use isostate_core::*;

const END: i64 = 1925;

fn params(on: bool) -> Params {
    let mut P = game_params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(END);
    P.tasks = TasksParams { enabled: on, ..TasksParams::game() };
    P
}

/// The game's content as a run under `P` plays it (with the tree laid).
fn content(P: &Params) -> Content {
    isostate_core::tree::content(P, game_content()).expect("the game's tree lays")
}

#[test]
fn the_switch_is_off_under_the_port_and_on_in_the_game() {
    let (port, game) = (default_params(), game_params());
    assert_eq!(port.tasks, TasksParams::default());
    assert!(!port.tasks.enabled && game.tasks.enabled);
    assert!(!serde_json::to_string(&port).unwrap().contains("\"tasks\""));
    // only the game's content has the classes
    assert!(default_content().tasks.is_empty());
    let C = content(&game);
    assert_eq!(C.tasks.method.len(), C.M);
    assert!(C.tasks.closed[tasks::PRESENCE] && C.tasks.spread[tasks::PRESENCE].is_empty());
    // every method of the game, the tree's among them, has its labour in classes that sum to one
    for m in 0..C.M {
        let s: f64 = C.tasks.method[m].iter().sum();
        assert!((s - 1.0).abs() < 1e-9, "{} has shares summing to {s}", C.method_name[m]);
    }
    assert_eq!(C.M, 48, "the 44 of the tree's stage 1 and the four of stage 6 (docs/rules/tree.md, \"After 1900: electricity\" and \"Oil, nitrogen and the land\")");
    // the content after 1900 lays the tractor and the cutter; the cutter takes the
    // pits from the hoist (`replaces`), which keeps the works
    let names: Vec<&str> = C.tasks.uses.iter().map(|u| u.name.as_str()).collect();
    assert_eq!(names, ["hoist", "tractor", "cutter"]);
    let sector = |n: &str| C.sector_name.iter().position(|x| x == n).unwrap();
    let (hoist, tractor, cutter) = (&C.tasks.uses[0], &C.tasks.uses[1], &C.tasks.uses[2]);
    assert!(hoist.on_works && hoist.on_sector.iter().all(|&b| !b), "the hoist keeps the works and leaves the pits");
    assert_eq!(cutter.replaces.as_deref(), Some("hoist"));
    for n in ["quarry", "claypit", "ironmine", "coalmine"] {
        assert!(cutter.on_sector[sector(n)] && !tractor.on_sector[sector(n)]);
    }
    for n in ["farm", "pasture", "forestry"] {
        assert!(tractor.on_sector[sector(n)] && !cutter.on_sector[sector(n)]);
    }
    assert!(!tractor.on_works && !cutter.on_works);
    // without the content after 1900 (tree.stage = 1) a use naming its goods is not
    // laid, and the hoist reaches the works and the pits as it did
    let mut one = game.clone();
    one.tree.stage = 1;
    let C1 = content(&one);
    assert_eq!(C1.tasks.uses.len(), 1);
    assert_eq!(C1.tasks.uses[0].name, "hoist");
    assert!(C1.tasks.uses[0].on_works && C1.tasks.uses[0].on_sector.iter().filter(|&&b| b).count() == 4);
}

/// `replaces`, and a use whose goods are missing, read from a file of their own.
#[test]
fn a_use_replaces_another_where_it_is_laid() {
    let C = content(&game_params());
    let head = "[tasks]\nclasses = [\"haul\", \"tend\", \"clerk\", \"judge\", \"presence\"]\nclosed = [\"presence\"]\n[tasks.spread]\nhaul = [[0.0, 0.3], [1.0, 100.0]]\n";
    let one = |name: &str, on: &str, good: &str, replaces: &str| {
        format!(
            "[[use]]\nname = \"{name}\"\nclass = \"haul\"\non = [{on}]\nfrom = 0.0\nreach = 1.0\nplant = {{ {good} = 1.0 }}\nrun = {{}}\nminders = 0.1\nlife = 10.0\npace = 0.0\ncap = 1.0\nfound_after = 1900\ndiscover = {{}}\nscale = 1.0\n{replaces}\n"
        )
    };
    let old = one("old", "\"works\", \"farm\", \"quarry\"", "machinery", "");
    let farm = C.sector_name.iter().position(|x| x == "farm").unwrap();
    let quarry = C.sector_name.iter().position(|x| x == "quarry").unwrap();
    // laid: the old use loses the places the new one is used, and keeps the rest
    let t = tasks::table_from(&C, &format!("{head}{old}{}", one("new", "\"quarry\"", "machinery", "replaces = \"old\""))).unwrap();
    assert!(t.uses[0].on_works && t.uses[0].on_sector[farm] && !t.uses[0].on_sector[quarry]);
    assert!(t.uses[1].on_sector[quarry]);
    // not laid (a good the content lacks): nothing is replaced
    let t = tasks::table_from(&C, &format!("{head}{old}{}", one("new", "\"quarry\"", "unobtainium", "replaces = \"old\""))).unwrap();
    assert_eq!(t.uses.len(), 1);
    assert!(t.uses[0].on_sector[quarry]);
    // a name that is no use of the file is an error
    assert!(tasks::table_from(&C, &format!("{head}{old}{}", one("new", "\"quarry\"", "machinery", "replaces = \"nothing\""))).is_err());
}

#[test]
fn the_schedule_adds_up() {
    let haul = [(0.0, 0.3), (1.0, 100.0)];
    let (lo, hi): (f64, f64) = (0.3, 100.0);
    // units for a part of an even spread: lo * ((hi/lo)^b - (hi/lo)^a) / ln(hi/lo)
    for (a, b) in [(0.0, 1.0), (0.0, 0.2), (0.2, 0.5), (0.37, 0.91)] {
        let want = lo * ((hi / lo).powf(b) - (hi / lo).powf(a)) / (hi / lo).ln();
        let got = tasks::units(&haul, a, b);
        assert!((got - want).abs() < 1e-12 * want.max(1.0), "[{a}, {b}]: {got} against {want}");
    }
    assert_eq!(tasks::units(&haul, 0.5, 0.5), 0.0);
    // a part split in two holds what the whole holds, also across a bend
    let bent = [(0.0, 1.0), (0.4, 30.0), (1.0, 3000.0)];
    let whole = tasks::units(&bent, 0.1, 0.9);
    let parts = tasks::units(&bent, 0.1, 0.4) + tasks::units(&bent, 0.4, 0.9);
    assert!((whole - parts).abs() < 1e-12 * whole);
    // the share held at a given wage and cost: a hand worth w/c units
    for (w, c) in [(8.18, 8.50), (10.0, 2.0), (1.0, 5.0), (100.0, 0.5)] {
        let x = tasks::share_at(&haul, w / c);
        let want = ((w / c / lo).ln() / (hi / lo).ln()).clamp(0.0, 1.0);
        assert!((x - want).abs() < 1e-12, "{w}/{c}: {x} against {want}");
        if x > 0.0 && x < 1.0 {
            assert!((tasks::worth(&haul, x) - w / c).abs() < 1e-9 * w / c);
        }
    }
    // a unit costing 1.04 of a month's wage takes a fifth of hauling
    assert!((tasks::share_at(&haul, 1.0 / 1.04) - 0.20).abs() < 0.005);
    // the inverse on a bent schedule
    let x = tasks::share_at(&bent, 100.0);
    assert!((tasks::worth(&bent, x) - 100.0).abs() < 1e-9);
    assert!((tasks::rank_correlation(&[1.0, 2.0, 3.0, 4.0], &[10.0, 20.0, 15.0, 40.0]) - 0.8).abs() < 1e-12);
}

/// With the table of uses emptied in code, the game's run on island 7 to 1925
/// is the run with the switch off, to the last digit; with the hoist it is
/// not. And in the run with the hoist: it is found, a works built after that
/// begins at its town's target, every works' share stays between the use's
/// `from` and `reach`, and money is conserved every month (`run_year` checks).
#[test]
fn island_7_to_1925() {
    let on = std::thread::spawn(|| {
        let P = params(true);
        let mut s = sim::start(&P, content(&P));
        let (mut at_target, mut found) = (0, None);
        let us = s.content.tasks.uses[0].clone();
        while s.year <= s.end_year {
            let year = s.year;
            sim::run_year(&mut s).expect("money is conserved every month");
            let st = &s.st;
            if found.is_none() && st.tasks.found[0] >= 0 {
                found = Some(st.tasks.found[0]);
            }
            for &f in &st.new_firms_year {
                if !st.f_alive[f] || st.tasks.target.is_empty() {
                    continue;
                }
                let t = st.tasks.target[st.f_town[f]];
                assert_eq!(st.tasks.f_auto[f], t, "{year}: works {f} begins off its town's target");
                if t[tasks::HAUL] > 0.0 {
                    at_target += 1;
                }
            }
            for f in 0..st.F_count.min(st.tasks.f_auto.len()) {
                let x = st.tasks.f_auto[f][tasks::HAUL];
                assert!(x == 0.0 || (us.from..=us.reach).contains(&x), "{year}: works {f} holds {x}");
                assert!(st.tasks.f_auto[f].iter().enumerate().all(|(k, &v)| k == tasks::HAUL || v == 0.0));
            }
        }
        let y = found.expect("island 7 finds the hoist by 1925 (if a later rule stops that, this test must be set again)");
        assert!((1901..=1925).contains(&y));
        assert!(at_target > 0, "no works was built with machines after {y}");
        let last = s.st.tasks.years.last().unwrap();
        assert!(last.held[tasks::HAUL] > 0.0 && last.book > 0.0, "{last:?}");
        save::check(&s)
    });
    let check_of = |P: Params, C: Content| {
        std::thread::spawn(move || {
            let mut s = sim::start(&P, C);
            while s.year <= s.end_year {
                sim::run_year(&mut s).expect("money is conserved every month");
            }
            save::check(&s)
        })
    };
    let P = params(true);
    let mut empty = content(&P);
    empty.tasks.uses.clear();
    empty.tasks.plant_goods.clear();
    let none = check_of(P, empty);
    let off = check_of(params(false), content(&params(false)));
    let (on, none, off) = (on.join().unwrap(), none.join().unwrap(), off.join().unwrap());
    assert_eq!(none, off, "with no use the rule moved the run");
    assert_ne!(on, off, "the hoist moved nothing");
}

/// Runs `P` on content `C` to its end, and gives `save::check` of it.
fn check_of(P: Params, C: Content) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut s = sim::start(&P, C);
        while s.year <= s.end_year {
            sim::run_year(&mut s).expect("money is conserved every month");
        }
        save::check(&s)
    })
}

/// The land ("On the land"), on island 7 to 1925 with `tasks.enabled` on and
/// `tasks.land` set by name. With no use reaching any land sector (every
/// use's sectors cleared in code) the run is the run with `tasks.land` off,
/// to the last digit; with the hoist at the pits it is not. In the run with
/// the hoist: a sector no use reaches holds nothing in any town in any year,
/// the pits' shares stay between the hoist's `from` and `reach`, and some
/// pit comes to hold part of its hauling. Played with `tree.stage = 1`, the
/// island without the content after 1900, where the hoist is the land's only
/// machine (with the content after 1900 the cutter takes the pits from it in
/// the 1920s and the tractor the fields: tests/land_century.rs).
#[test]
fn island_7_to_1925_on_the_land() {
    let land = |on: bool| {
        let mut P = params(true);
        P.tasks.land = on;
        P.tree.stage = 1;
        P
    };
    let on = std::thread::spawn(move || {
        let P = land(true);
        let mut s = sim::start(&P, content(&P));
        let us = s.content.tasks.uses[0].clone();
        let mut most: f64 = 0.0;
        while s.year <= s.end_year {
            let year = s.year;
            sim::run_year(&mut s).expect("money is conserved every month");
            let (st, C) = (&s.st, &s.content);
            for row in &st.tasks.land_auto {
                for (k, auto) in row.iter().enumerate() {
                    if us.on_sector[k] {
                        let x = auto[tasks::HAUL];
                        assert!(x == 0.0 || (us.from..=us.reach).contains(&x), "{year}: {} holds {x}", C.sector_name[k]);
                        assert!(auto.iter().enumerate().all(|(c, &v)| c == tasks::HAUL || v == 0.0));
                        most = most.max(x);
                    } else {
                        assert_eq!(*auto, [0.0; tasks::K], "{year}: {}, which no use reaches, holds {auto:?}", C.sector_name[k]);
                    }
                }
            }
        }
        assert!(most > 0.0, "no pit came to hold any of its hauling by {END}");
        save::check(&s)
    });
    let P = land(true);
    let mut none = content(&P);
    none.tasks.uses.iter_mut().for_each(|u| u.on_sector.iter_mut().for_each(|b| *b = false));
    let none = check_of(P, none);
    let off = check_of(land(false), content(&land(false)));
    let (on, none, off) = (on.join().unwrap(), none.join().unwrap(), off.join().unwrap());
    assert_eq!(none, off, "with no use on the land the land's switch moved the run");
    assert_ne!(on, off, "the hoist at the pits moved nothing");
}

/// The year island 7 finds the hoist under `params(true)`, as the tests below
/// lean on it: they stop ten years on.
fn starts(s: &sim::Sim) -> Option<i64> {
    s.st.tasks.found.first().copied().filter(|&y| y >= 0)
}

/// A machine works once it stands, on island 7 played a month at a time to
/// ten years past the find (under the game's rules with `tasks.enabled` and
/// `tasks.land` set by name). Every month: no works and no town's sector
/// holds more than it means to (`f_held` against `f_auto`, `land_held`
/// against `land_auto`); a works whose hands a unit fell had plant in place
/// when the month began; and a works founded after the find, which means to
/// hold its town's target, works its first month by hand at its recipe and
/// holds nothing then. Some of those come to hold part of their hauling once
/// their plant is delivered.
#[test]
fn machines_work_once_they_stand() {
    let P = params(true);
    let mut s = sim::start(&P, content(&P));
    let mut fresh: Vec<usize> = Vec::new();
    let (mut first_months, mut came) = (0, 0);
    let mut seen: Vec<usize> = Vec::new();
    while s.year <= s.end_year && starts(&s).is_none_or(|y| s.year <= y + 10) {
        let year = s.year;
        for month in 0..12 {
            let plant: Vec<bool> = s.st.tasks.f_plant.iter().map(|p| p.iter().any(|&v| v > 0.0)).collect();
            sim::run_month(&mut s, month).expect("money is conserved every month");
            let st = &s.st;
            for f in 0..st.F_count.min(st.tasks.f_held.len()) {
                let (held, auto) = (st.tasks.f_held[f], st.tasks.f_auto[f]);
                assert!((0..tasks::K).all(|k| held[k] <= auto[k]), "{year}-{month}: works {f} holds {held:?}, more than it means to, {auto:?}");
                if st.f_alive[f] && st.tasks.f_factor[f] < 1.0 {
                    assert!(plant.get(f) == Some(&true), "{year}-{month}: works {f}'s hands fell with no plant in place");
                }
            }
            for f in fresh.drain(..) {
                if st.f_alive[f] {
                    assert_eq!(st.tasks.f_factor[f], 1.0, "{year}-{month}: works {f}'s first month is not by hand");
                    assert_eq!(st.tasks.f_held[f], [0.0; tasks::K]);
                    first_months += 1;
                }
            }
            for (t, row) in st.tasks.land_held.iter().enumerate() {
                for (k, held) in row.iter().enumerate() {
                    let auto = st.tasks.land_auto[t][k];
                    assert!((0..tasks::K).all(|c| held[c] <= auto[c]), "{year}-{month}: sector {k} of town {t} holds {held:?} over {auto:?}");
                }
            }
        }
        sim::end_year(&mut s);
        let st = &s.st;
        for &f in &st.new_firms_year {
            if st.f_alive[f] && starts(&s).is_some() && st.tasks.f_auto.get(f).is_some_and(|a| a[tasks::HAUL] > 0.0) {
                fresh.push(f);
                seen.push(f);
            }
        }
    }
    let y = starts(&s).expect("island 7 finds the hoist (if a later rule stops that, this test must be set again)");
    for &f in &seen {
        if s.st.f_alive[f] && s.st.tasks.f_held[f][tasks::HAUL] > 0.0 {
            came += 1;
        }
    }
    assert!(first_months > 0, "no works was founded meaning to hold machines after {y}");
    assert!(came > 0, "no works founded after {y} came to hold any of its hauling");
}

/// With the plant starved by hand (every delivery to the works and the land
/// taken away again before the next month, in code), no works and no town's
/// sector holds any part of any class in the ten years after the find, while
/// what they mean to hold rises. Island 7, played under the game's rules with
/// `tasks.enabled` and `tasks.land` set by name, and `tree.stage = 1`: the
/// ten years after the hoist's find are years its pits mean to hold machines
/// only without the content after 1900, whose cutter comes later.
#[test]
fn starved_of_plant_nothing_is_held() {
    let mut P = params(true);
    P.tree.stage = 1;
    let mut s = sim::start(&P, content(&P));
    let (mut most_auto, mut most_land): (f64, f64) = (0.0, 0.0);
    while s.year <= s.end_year && starts(&s).is_none_or(|y| s.year <= y + 10) {
        let year = s.year;
        for month in 0..12 {
            sim::run_month(&mut s, month).expect("money is conserved every month");
            let st = &mut s.st;
            assert!(st.tasks.f_held.iter().all(|h| *h == [0.0; tasks::K]), "{year}-{month}: a works holds machines it was never given");
            assert!(st.tasks.land_held.iter().flatten().all(|h| *h == [0.0; tasks::K]), "{year}-{month}: a pit holds machines it was never given");
            for f in 0..st.F_count.min(st.tasks.f_auto.len()) {
                if st.f_alive[f] {
                    most_auto = most_auto.max(st.tasks.f_auto[f][tasks::HAUL]);
                }
            }
            most_land = st.tasks.land_auto.iter().flatten().fold(most_land, |m, a| m.max(a[tasks::HAUL]));
            // the starving: what was delivered is taken away again
            st.tasks.f_plant.iter_mut().flatten().for_each(|v| *v = 0.0);
            st.tasks.land_plant.iter_mut().flatten().flatten().for_each(|v| *v = 0.0);
        }
        sim::end_year(&mut s);
    }
    let y = starts(&s).expect("island 7 finds the hoist (if a later rule stops that, this test must be set again)");
    assert!(most_auto > 0.05 && most_land > 0.05, "in the ten years after {y} what was meant rose only to {most_auto} in the works and {most_land} on the land");
    let last = s.st.tasks.years.last().unwrap();
    assert_eq!(last.held[tasks::HAUL], 0.0);
    assert!(last.aim[tasks::HAUL] > 0.0, "{last:?}");
}

/// The ceiling on yield is inert before 1950 by construction: island 7 to
/// 1949, played under the game's rules with `tasks.yield_cap` set by name to
/// 2, is the run with it unset to the last digit; and by then the ground
/// worked has its best years kept.
#[test]
fn the_ceiling_waits_for_1950() {
    let cap = |c: Option<f64>| {
        let mut P = params(true);
        P.end_year = Some(1949);
        P.tasks.yield_cap = c;
        P
    };
    let kept = std::thread::spawn(move || {
        let P = cap(Some(2.0));
        let mut s = sim::start(&P, content(&P));
        while s.year <= s.end_year {
            sim::run_year(&mut s).expect("money is conserved every month");
        }
        let kept = s.st.tasks.best.iter().filter(|&&b| b > 0.0).count();
        assert!(kept > 100, "only {kept} grounds have a best year kept");
        assert_eq!(s.st.tasks.year_now, 1950);
        save::check(&s)
    });
    let unset = check_of(cap(None), content(&cap(None)));
    assert_eq!(kept.join().unwrap(), unset.join().unwrap(), "the ceiling moved the run before 1950");
}
