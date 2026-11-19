//! The core of the island (docs/core.md, `isostate-core ablate`): an arm with a
//! rule off is the sweep's run with the same `--set`, fact for fact, and the
//! command writes its table and its page. The made-up cases (a rule that is
//! idle, two that cover for each other) are the module's own tests, in
//! `src/ablate.rs`.
//!
//! The case is played under the game's rules with six families, on islands 1
//! and 2, to 1800. It sets the command against the sweep on the same islands
//! with the same rule off, so it leans on no rule's history: it holds whatever
//! rules are on. The rule turned off is `works.keep_order`, the first of the
//! lesser switches; any asked rule would do.
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use isostate_core::params::{default_params, game_params};
use isostate_core::{ablate, facts, sweep};
use serde_json::Value;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("isostate-ablate-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().copied().map(String::from).collect()
}

/// The groups of a sweep's facts, by name, from its facts.json.
fn groups(dir: &Path) -> HashMap<String, String> {
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("facts.json")).unwrap()).unwrap();
    doc["facts"].as_array().unwrap().iter().map(|f| (f["name"].as_str().unwrap().to_string(), f["group"].as_str().unwrap().to_string())).collect()
}

#[test]
fn an_arm_is_the_sweep_with_the_same_set() {
    let rule = "works.keep_order";
    let (sd, ad) = (scratch("sweep"), scratch("ablate"));
    let page = ad.join("page").join("core.md");
    std::fs::create_dir_all(page.parent().unwrap()).unwrap();
    let common = ["--seeds", "1,2", "--families", "6", "--end-year", "1800", "--quiet"];
    let mut args = strings(&common);
    args.extend(strings(&["--rules", "game", "--out", sd.to_str().unwrap(), "--set", &format!("{rule}=false")]));
    sweep::command(&args, false).expect("the sweep runs");
    let mut args = strings(&common);
    args.extend(strings(&["--out", ad.to_str().unwrap(), "--only", rule, "--page", page.to_str().unwrap()]));
    ablate::command(&args, false).expect("the command runs");

    // the arm's facts are the sweep's, bit for bit, outside Speed
    let arm = ad.join("off-1");
    assert_eq!(std::fs::read_to_string(arm.join("off.txt")).unwrap().trim(), rule);
    let (s, a) = (sweep::read_then(&sd).unwrap(), sweep::read_then(&arm).unwrap());
    let (s, a) = (&s[0], &a[0]);
    assert_eq!(s.seeds, a.seeds);
    let (gs, ga) = (groups(&sd), groups(&arm));
    let mut names: Vec<&String> = gs.keys().collect();
    names.sort();
    let mut other: Vec<&String> = ga.keys().collect();
    other.sort();
    assert_eq!(names, other, "the same facts");
    let mut compared = 0;
    for name in names.into_iter().filter(|n| gs[*n] != facts::SPEED) {
        let bits = |v: Option<&Vec<(u64, f64)>>| v.map(|v| v.iter().map(|(s, x)| (*s, x.to_bits())).collect::<Vec<_>>());
        assert_eq!(bits(s.values.get(name)), bits(a.values.get(name)), "{name}");
        compared += 1;
    }
    eprintln!("{compared} facts the same on islands 1 and 2");
    assert!(compared > 100);

    // the table, the page and its chart are written, in the out directory and at --page
    let txt = std::fs::read_to_string(ad.join("ablate.txt")).unwrap();
    println!("{txt}");
    assert!(txt.starts_with("The core of the island: 1 of the "), "{txt}");
    assert!(txt.contains(rule));
    let md = std::fs::read_to_string(&page).unwrap();
    assert_eq!(md, std::fs::read_to_string(ad.join("core.md")).unwrap());
    assert!(md.starts_with("# The core of the island\n\nWritten by `isostate-core ablate`, played on "), "{md}");
    // the page keeps no path of the machine it was made on
    assert!(md.contains("This one was made by `isostate-core ablate --seeds 1,2 --families 6 --end-year 1800 --quiet --out DIR --only works.keep_order --page FILE`."), "{md}");
    assert!(!md.contains(ad.to_str().unwrap()) && !md.contains(&ad.to_str().unwrap().replace('\\', "/")));
    for part in ["## Every rule, by kind", "## The idle rules, and the second pass", "## What was left out, and why"] {
        assert!(md.contains(part), "the page has no {part}");
    }
    // every name left out is named on the page, whichever the rules leave out today
    let (_, left_out) = ablate::asked(&game_params(), &default_params());
    for l in &left_out {
        assert!(md.contains(&format!("`{}`", l.name)), "the page does not name {}, which is left out", l.name);
    }
    assert!(page.with_file_name("core.svg").exists() && std::fs::read_to_string(ad.join("core.svg")).unwrap().starts_with("<svg"));
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(ad.join("ablate.json")).unwrap()).unwrap();
    assert_eq!(doc["rules"].as_array().unwrap().len(), 1);
    // with two islands a side nothing can stand apart (it takes four a side)
    assert_eq!(doc["rules"][0]["facts_apart"].as_array().map(|a| a.len()), Some(0));
    assert_eq!(doc["rules"][0]["label"], rule);

    // --again reads the table back from its files and gives the same table, playing nothing
    ablate::command(&strings(&["--again", ad.to_str().unwrap()]), false).expect("the table is read again");
    assert_eq!(std::fs::read_to_string(ad.join("again").join("ablate.txt")).unwrap(), txt);
    let again = std::fs::read_to_string(ad.join("again").join("core.md")).unwrap();
    assert!(again.contains("and read again from its files on "), "{again}");
}

#[test]
fn what_is_wrong_is_refused_before_anything_runs() {
    let out = scratch("refused");
    let try_ = |extra: &[&str]| {
        let mut args = strings(&["--out", out.to_str().unwrap(), "--seeds", "1", "--end-year", "1751", "--quiet"]);
        args.extend(strings(extra));
        ablate::command(&args, false).err().unwrap_or_default()
    };
    // a name left out is refused with why, whichever the rules leave out today
    let (_, left_out) = ablate::asked(&game_params(), &default_params());
    if let Some(l) = left_out.first() {
        let e = try_(&["--only", &l.name]);
        assert!(e.contains("left out") && e.contains(&l.why), "{e}");
    }
    let e = try_(&["--only", "no_such.rule"]);
    assert!(e.contains("not a rule switched on under the game's rules"), "{e}");
    let e = try_(&["--rules", "port"]);
    assert!(e.contains("--rules"), "{e}");
    assert!(!out.exists(), "a refused command wrote its directory");
}
