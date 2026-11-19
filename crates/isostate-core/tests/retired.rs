//! The settings retired in stage 5 of the pass over the rules
//! (`docs/designs/rules-pass.md`, section 4.5): sixteen numbers the sweeps found
//! to move nothing are constants of their modules now, with the values they had,
//! and two pairs that were one number under two names are folded. A settings
//! file or a `--set` that still names one is refused by every road, in a
//! sentence that says it is no longer a setting and what it became, not as a
//! name nobody has heard of. No island is played here: the runs are held where
//! they were by the golden tests and by the sweep.
#![allow(non_snake_case)]

use std::path::PathBuf;

use isostate_core::params::{self, RETIRED};
use isostate_core::*;
use serde_json::json;

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("isostate-retired-test-{}-{name}", std::process::id()))
}

/// The refusal says which setting, that it is retired, and what became of it,
/// and is not serde's word for a field it does not know.
fn says_retired(err: &str, key: &str, became: &str) {
    assert!(err.contains(&format!("{key} is no longer a setting since stage 5 of the pass over the rules")), "{key}: {err}");
    assert!(err.contains(became), "{key}: the refusal does not say what it became: {err}");
    assert!(!err.contains("unknown field") && !err.contains("there is no parameter"), "{key}: refused as an unknown name: {err}");
}

/// `canals.induced` as `[canals]` with `induced = 1` in TOML, and as
/// `{"canals": {"induced": 1}}` in JSON.
fn as_files(key: &str) -> (String, String) {
    let (group, name) = key.split_once('.').expect("every retired setting belongs to a mechanism");
    (format!("[{group}]\n{name} = 1\n"), format!("{{\"{group}\": {{\"{name}\": 1}}}}"))
}

#[test]
fn every_retired_setting_is_refused_by_every_road_and_says_what_it_became() {
    assert_eq!(RETIRED.len(), 18, "sixteen constants and two folded pairs");
    for (n, &(key, became)) in RETIRED.iter().enumerate() {
        // the sweep's --set, over either set of rules (a mechanism left at its default is started from there)
        for P in [game_params(), default_params()] {
            let err = sweep::set_field(&P, key, &json!(1)).expect_err(key);
            assert!(err.starts_with(&format!("--set {key}: ")), "{err}");
            says_retired(&err, key, became);
        }
        // and the whole of a sweep's settings, refused before anything runs
        let args: Vec<String> = ["--out", "x", "--rules", "game", "--set", &format!("{key}=1")].iter().map(|s| s.to_string()).collect();
        let o = sweep::parse_args(&args).unwrap();
        says_retired(&sweep::settings(&o).err().expect(key), key, became);

        // a settings file, in TOML and in JSON, laid over the game's rules or read on its own
        let (toml, json) = as_files(key);
        for err in [
            game_params().with_toml_overrides(&toml).expect_err(key),
            game_params().with_json_overrides(&json).expect_err(key),
            Params::from_toml_str(&toml).expect_err(key),
            Params::from_json_str(&json).expect_err(key),
        ] {
            assert!(err.starts_with("params: "), "{err}");
            says_retired(&err, key, became);
        }
        // and from a file on disk, which names the file; every other one a JSON file
        let file = scratch(&format!("{n}.{}", if n % 2 == 0 { "toml" } else { "json" }));
        std::fs::write(&file, if n % 2 == 0 { &toml } else { &json }).unwrap();
        let err = game_params().load_over(&file).expect_err(key);
        let _ = std::fs::remove_file(&file);
        assert!(err.contains(&file.display().to_string()), "{err}");
        says_retired(&err, key, became);
    }
}

/// The command lines refuse one before anything is run or written.
#[test]
fn the_command_lines_refuse_a_retired_setting_before_anything_runs() {
    let out = scratch("out");
    let dir = out.to_str().unwrap();
    let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let err = sweep::command(&args(&["--seeds", "1", "--rules", "game", "--set", "mines.payback=4", "--out", dir, "--quiet"]), false).unwrap_err();
    assert!(err.contains("mines.payback is no longer a setting") && err.contains("mines.open_at"), "{err}");
    let file = scratch("params.toml");
    std::fs::write(&file, "[canals]\nhurdle = 0.1\ninduced = 1.0\n").unwrap();
    let err = cli::main_with_args(&args(&["run", "--out", dir, "--rules", "game", "--params", file.to_str().unwrap()])).unwrap_err();
    let _ = std::fs::remove_file(&file);
    assert!(err.contains("params.toml") && err.contains("canals.induced is no longer a setting") && err.contains("folded into canals.hurdle"), "{err}");
    assert!(!out.exists(), "nothing is written when a setting is refused");
}

/// No setting that is still a setting is on the list: a name that came back
/// as a setting while the list still refused it could never be set.
#[test]
fn no_retired_name_is_a_setting_again() {
    for P in [game_params(), default_params()] {
        let doc = serde_json::to_value(&P).unwrap();
        for &(key, _) in RETIRED {
            let (group, name) = key.split_once('.').unwrap();
            assert!(doc.get(group).and_then(|g| g.get(name)).is_none(), "{key} is a setting again: take it off params::RETIRED");
        }
        assert_eq!(params::retired(&doc), None);
    }
    // the settings that stay beside them are still settings (the critic's three wait for stage 6)
    for key in ["people.found_cells", "people.found_spacing", "carting.max_share", "canals.hurdle", "mines.sink", "mines.open_at", "railways.districts"] {
        sweep::set_field(&game_params(), key, &json!(2)).unwrap_or_else(|e| panic!("{key}: {e}"));
    }
}

/// The constants have the values the settings had, and are what the planners
/// and the mechanisms are given.
#[test]
fn the_constants_are_the_values_the_settings_had() {
    assert_eq!(railways::MAX_PROJECTS, 40);
    assert_eq!((canals::MAX_PROJECTS, canals::DISTRICTS, canals::COOL_YEARS), (12, 6, 5));
    assert_eq!((canals::BRANCH_GAP, canals::DISTRICT_SPACING, canals::NAVVY_MAX_SHARE, canals::NAVVY_ADJUST), (3.0, 6.0, 0.2, 0.25));
    assert_eq!(mines::CELL_LIMIT, 0.5);
    assert_eq!(peopling::GROWTH_FLOOR, -0.06);
    assert_eq!((towns::DECLINE_YEARS, towns::BACK_SPAN), (15, 40));
    assert_eq!(prices::GLUT_FLOOR, 0.0);
    assert_eq!((chronicle::AT_LEAST, chronicle::FILL_FLOOR, chronicle::BASE_YEARS), (2, 0.45, 5));
    // the mines' threshold is the old one to the last bit: sink over payback, 40 over 4
    let m = mines::MineParams::game();
    assert_eq!((m.sink, m.open_at.to_bits()), (40.0, (40.0f64 / 4.0).to_bits()));

    let (C, P, st) = (game_content(), game_params(), State::default());
    let w = railways::way(&st, &C, &P);
    assert_eq!(w.max_projects, railways::MAX_PROJECTS);
    // the railway keeps the port's two numbers that are one (docs/rules/railways.md)
    assert_eq!((w.hurdle, w.induced), (P.rail_hurdle, P.rail_induced));
    let w = canals::way(&st, &C, &P);
    assert_eq!((w.max_projects, w.districts, w.cool_years), (canals::MAX_PROJECTS, canals::DISTRICTS, canals::COOL_YEARS));
    assert_eq!((w.branch_gap, w.district_spacing), (canals::BRANCH_GAP, canals::DISTRICT_SPACING));
    // the canals' induced traffic is folded into the hurdle: one times the saving
    assert_eq!((w.hurdle, w.induced), (P.canals.hurdle, 1.0));
}
