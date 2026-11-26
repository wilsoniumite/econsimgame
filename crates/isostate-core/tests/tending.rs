//! Machines that tend (not in the Python; docs/rules/tending.md): the moving
//! line, the automatic machine, the robot and the line that runs itself, four
//! uses of `data/tasks.toml` behind `tasks.tending`, each waiting on an idea of
//! the tree (`data/game.toml`).
//!
//! The island runs are played on island 7 to 1935 under the game's rules with
//! six families, with the switches their cases lean on set by name
//! (`tasks.enabled`, `tasks.land`, `tasks.office_machines`, `tasks.tending`)
//! and every other rule as the game has it. What they check holds on any
//! island's history (a run against another run, shares within their bounds,
//! money that adds up), but for one thing: that island 7 finds the moving line
//! by 1935, which the test says if a later rule stops it.
#![allow(non_snake_case)]

use isostate_core::tasks::{self, TasksParams};
use isostate_core::*;

const TO: i64 = 1935;

/// The four uses and the ideas they wait on, in the file's order.
const USES: [(&str, &str, f64, Option<&str>); 4] = [
    ("moving_line", "moving_line", 0.35, Some("electric_drive")),
    ("automatic_machine", "automatic_machine", 0.7, None),
    ("robot", "industrial_robot", 0.9, Some("office_computing")),
    ("lights_out_line", "lights_out_line", 1.0, Some("models")),
];

fn params(tending: bool) -> Params {
    let mut P = game_params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(TO);
    // the lorry (docs/rules/lorry.md), after the four in the file, off by name: these runs are played without it
    P.tasks = TasksParams { enabled: true, land: true, office_machines: true, tending, lorry: false, ..TasksParams::game() };
    P
}

/// The game's content as a run under `P` plays it (with the tree laid).
fn content(P: &Params) -> Content {
    isostate_core::tree::content(P, game_content()).expect("the game's tree lays")
}

fn use_of(C: &Content, name: &str) -> usize {
    C.tasks.uses.iter().position(|u| u.name == name).unwrap_or_else(|| panic!("no use {name:?} in the table"))
}

fn idea_of(C: &Content, name: &str) -> usize {
    C.tech_name.iter().position(|n| n == name).unwrap_or_else(|| panic!("no idea {name:?} in the content"))
}

/// The switch is off under the port's rules and under the game's (it stands
/// off: docs/rules/tending.md, "Why it stands off"), and is left out of their
/// files while off. The four uses are laid with the content after 1900, tend
/// in the works alone from the easiest task, each to its reach, each waiting on
/// its idea of the tree and the uses it needs; every idea is of the tree's
/// stage 6 with a floor after 1900, so the played century cannot move; and
/// with the switch off no use may be found and no idea is sought.
#[test]
fn tending_stands_behind_its_switch() {
    let (port, off) = (default_params(), game_params());
    assert!(!port.tasks.tending && !off.tasks.tending, "the machines that tend stand off under both sets of rules");
    let mut game = off.clone();
    game.tasks.tending = true;
    let json = serde_json::to_string(&off).unwrap();
    assert!(json.contains("\"tasks\"") && !json.contains("tending"), "a run with the switch off writes the file it wrote before the switch");
    assert!(serde_json::to_string(&game).unwrap().contains("\"tending\":true"));
    assert!(tasks::USE_SWITCHES.contains(&"tending"));
    let C = content(&game);
    for (name, idea, reach, needs) in USES {
        let u = use_of(&C, name);
        let us = &C.tasks.uses[u];
        assert_eq!(us.switch.as_deref(), Some("tending"), "{name}");
        assert_eq!((us.class, us.from, us.reach), (tasks::TEND, 0.0, reach), "{name}");
        assert!(us.on_works && us.on_sector.iter().all(|&b| !b), "{name} tends in the works alone");
        let t = idea_of(&C, idea);
        assert_eq!(us.idea, Some(t), "{name} waits on {idea}");
        assert!(t >= C.M && !C.tech_known[t], "{idea} is an invention of the tree, unknown in 1750");
        assert!(C.tech_after[t] >= 1901 && us.found_after >= 1901, "{name}: nothing is sought or found before 1901");
        let wanted: Vec<usize> = needs.iter().map(|n| use_of(&C, n)).collect();
        assert_eq!(us.needs, wanted, "{name} needs {needs:?}");
        assert!(us.allowed(&game) && !us.allowed(&off));
        assert!(tasks::idea_of_use(&C, t));
        assert!(tasks::idea_sought(&C, &game, t) && !tasks::idea_sought(&C, &off, t), "{idea} is sought only with the switch on");
    }
    // no other idea of the tree waits on a use, but the lorry's (docs/rules/lorry.md)
    let mut ours: Vec<usize> = USES.iter().map(|u| idea_of(&C, u.1)).collect();
    ours.push(idea_of(&C, "motor_vehicle"));
    assert!((0..C.NT).filter(|t| !ours.contains(t)).all(|t| !tasks::idea_of_use(&C, t)));
    // without the content after 1900 neither the uses nor their ideas are laid
    let mut one = game.clone();
    one.tree.stage = 1;
    let C1 = content(&one);
    assert!(C1.tasks.uses.iter().all(|u| u.switch.as_deref() != Some("tending")));
    assert!(USES.iter().all(|u| !C1.tech_name.iter().any(|n| n == u.1)));
}

/// Island 7 to 1935, three runs side by side:
///
/// - with the switch on: money is conserved every month (`run_year` checks);
///   the moving line is found in the year after its idea, and not before the
///   electric drive; no works means to hold more of its tending than the
///   line reaches, nor holds more than it means; some works comes to hold part
///   of its tending; and the run is the run with the switch off to the end of
///   1907, the year before the line's floor;
/// - with the switch off: no idea of the four is found;
/// - with the switch off and the four uses taken out of the table, their ideas
///   held back by a floor past the run: the same run as with the switch off, to
///   the end (with the switch off they draw nothing and touch nothing).
#[test]
fn island_7_to_1935() {
    let played = |P: Params, C: Content| {
        std::thread::spawn(move || {
            let mut s = sim::start(&P, C);
            let mut at1907 = String::new();
            while s.year <= s.end_year {
                let year = s.year;
                sim::run_year(&mut s).expect("money is conserved every month");
                if year == 1907 {
                    at1907 = save::check(&s);
                }
            }
            let ideas: Vec<i64> = USES.iter().map(|u| s.st.tech_year[idea_of(&s.content, u.1)]).collect();
            (at1907, save::check(&s), ideas)
        })
    };
    let on = std::thread::spawn(move || {
        let P = params(true);
        let mut s = sim::start(&P, content(&P));
        let (line, drive) = (use_of(&s.content, "moving_line"), use_of(&s.content, "electric_drive"));
        let idea = idea_of(&s.content, "moving_line");
        let us = s.content.tasks.uses[line].clone();
        let (mut at1907, mut most) = (String::new(), 0.0f64);
        while s.year <= s.end_year {
            let year = s.year;
            sim::run_year(&mut s).expect("money is conserved every month, through the find");
            if year == 1907 {
                at1907 = save::check(&s);
            }
            let st = &s.st;
            let found = st.tasks.found[line];
            if found >= 0 {
                assert!(st.tech_year[idea] >= 1908 && found == st.tech_year[idea] + 1, "{year}: the line found in {found}, its idea in {}", st.tech_year[idea]);
                assert!(st.tasks.found[drive] >= 0 && st.tasks.found[drive] <= found, "{year}: the line found before the drive");
            }
            for f in 0..st.F_count.min(st.tasks.f_auto.len()) {
                let (x, h) = (st.tasks.f_auto[f][tasks::TEND], st.tasks.f_held[f][tasks::TEND]);
                assert!((0.0..=us.reach + 1e-12).contains(&x), "{year}: works {f} means to hold {x} of its tending, past the line's reach");
                assert!((0.0..=x).contains(&h), "{year}: works {f} holds {h} of its tending, meaning {x}");
                if st.f_alive[f] {
                    most = most.max(h);
                }
            }
        }
        let y = s.st.tasks.found[line];
        assert!(y >= 1909, "island 7 finds the moving line by {TO} (if a later rule stops that, this test must be set again)");
        assert!(most > 0.0, "no works came to hold any of its tending by {TO}, the line found in {y}");
        eprintln!("the moving line found in {y}; the most a works held of its tending by {TO}: {most:.3}");
        at1907
    });
    let off = played(params(false), content(&params(false)));
    let mut out = content(&params(false));
    // the lorry, the table's last use, off in these runs, is taken out first
    let lorry = use_of(&out, "lorry");
    assert_eq!(lorry + 1, out.tasks.uses.len());
    out.tasks.uses.remove(lorry);
    let t = idea_of(&out, "motor_vehicle");
    out.tech_after[t] = 9999;
    for (name, idea, _, _) in USES.iter().rev() {
        let u = use_of(&out, name);
        assert_eq!(u + 1, out.tasks.uses.len(), "the four are the table's last uses, so that taking them out moves no other's place");
        out.tasks.uses.remove(u);
        let t = idea_of(&out, idea);
        out.tech_after[t] = 9999;
    }
    let out = played(params(false), out);
    let (on, off, out) = (on.join().unwrap(), off.join().unwrap(), out.join().unwrap());
    assert_eq!(on, off.0, "the switch on moved the run before 1908, though no idea of the four is sought before 1908");
    assert!(off.2.iter().all(|&y| y < 0), "with the switch off an idea of the four was found: {:?}", off.2);
    assert_eq!(out.1, off.1, "with the switch off the run to {TO} is not the run with the four uses taken out");
}

/// The later machines that tend reaching the land, the pits and the carts
/// (`tasks.tending_wide`, docs/rules/tending.md, "Where the hands go"): off
/// under both sets of rules and left out of their files while off; the robot
/// and the line that runs itself name every land sector and the carters in
/// their `wide`, the moving line and the automatic machine nothing; and a use
/// reaches its `wide` places only while this switch and its own (`tending`)
/// are both on, so with either off it reaches what its `on` names, the works,
/// and nothing more. (That the island with `tasks.tending` off is the same
/// island with this switch on or off, to the last digit, is shown by a sweep in
/// the page, "Where the hands go": the robot is found after 1978, so a run to
/// show it would take a run to 1980 at the least.)
#[test]
fn tending_wide_stands_behind_both_switches() {
    let (port, off) = (default_params(), game_params());
    assert!(!port.tasks.tending_wide && !off.tasks.tending_wide, "the wider reach stands off under both sets of rules");
    assert!(!serde_json::to_string(&off).unwrap().contains("tending_wide"), "a run with the switch off writes the file it wrote before the switch");
    let mut wide_only = off.clone();
    wide_only.tasks.tending_wide = true;
    let mut tending_only = off.clone();
    tending_only.tasks.tending = true;
    let mut both = tending_only.clone();
    both.tasks.tending_wide = true;
    let C = content(&both);
    let land = |us: &tasks::Use, P: &Params| (0..C.NS).filter(|&s| us.on_land(s, P)).count();
    for (name, _, _, _) in USES {
        let us = &C.tasks.uses[use_of(&C, name)];
        let wide = matches!(name, "robot" | "lights_out_line");
        assert_eq!(us.wide_sector.iter().filter(|&&b| b).count(), if wide { C.NS } else { 0 }, "{name}: every land sector or none");
        assert_eq!(us.wide_carting, wide, "{name}: the carters or not");
        for P in [&off, &wide_only, &tending_only] {
            assert!(land(us, P) == 0 && !us.on_carts(P) && !us.on_any_land(P), "{name}: reaches past the works with a switch off");
        }
        assert_eq!(land(us, &both), if wide { C.NS } else { 0 }, "{name} with both switches on");
        assert_eq!(us.on_carts(&both), wide, "{name} on the carts with both switches on");
    }
    // every other use reaches what its `on` names, whatever the switch
    for us in C.tasks.uses.iter().filter(|u| u.switch.as_deref() != Some("tending")) {
        assert!(us.wide_sector.iter().all(|&b| !b) && !us.wide_carting, "{} has a `wide`", us.name);
        assert_eq!((0..C.NS).map(|s| us.on_land(s, &both)).collect::<Vec<_>>(), us.on_sector, "{}", us.name);
        assert_eq!(us.on_carts(&both), us.on_carting, "{}", us.name);
    }
}
