//! The lab (docs/lab.md): the control is the sweep's plain run to the last
//! bit, a treated arm is the run with its edits, the arms part at the edit and
//! not before, and whatever is wrong with the command line is refused before
//! anything is written.
//!
//! Every case is played under the game's rules with six families, on islands
//! 1 and 2 (7 for (b)), to 1800 at most. They set the lab against the sweep and
//! against `edits::run_with` on the same island, so they lean on no rule's
//! history: they hold whatever rules are on.
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use isostate_core::edits::{self, Dated, Edit};
use isostate_core::lab;
use isostate_core::params::Rules;
use isostate_core::{facts, sweep};
use serde_json::Value;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("isostate-lab-test-{}-{name}", std::process::id()));
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
fn the_control_is_the_plain_run() {
    // (a) the sweep and the lab on the same islands; every fact outside Speed equal, bit for bit
    let (sd, ld) = (scratch("plain-sweep"), scratch("plain-lab"));
    let common = ["--seeds", "1,2", "--rules", "game", "--families", "6", "--end-year", "1800", "--quiet"];
    let mut args = strings(&common);
    args.extend(strings(&["--out", sd.to_str().unwrap()]));
    sweep::command(&args, false).expect("the sweep runs");
    let mut args = strings(&common);
    args.extend(strings(&["--out", ld.to_str().unwrap(), "--rule", "money.growth=0@1790"]));
    lab::command(&args, false).expect("the lab runs");

    let control = ld.join("control");
    let (s, c) = (sweep::read_then(&sd).unwrap(), sweep::read_then(&control).unwrap());
    assert_eq!((s.len(), c.len()), (1, 1));
    let (s, c) = (&s[0], &c[0]);
    assert_eq!(s.seeds, c.seeds);
    let (gs, gc) = (groups(&sd), groups(&control));
    let mut names: Vec<&String> = gs.keys().collect();
    names.sort();
    assert_eq!(names, { let mut n: Vec<&String> = gc.keys().collect(); n.sort(); n }, "the same facts, so no news_hand in the control");
    assert!(!gc.contains_key("news_hand"));
    let mut compared = 0;
    for name in names.into_iter().filter(|n| gs[*n] != facts::SPEED) {
        let bits = |v: Option<&Vec<(u64, f64)>>| v.map(|v| v.iter().map(|(s, x)| (*s, x.to_bits())).collect::<Vec<_>>());
        assert_eq!(bits(s.values.get(name)), bits(c.values.get(name)), "{name}");
        compared += 1;
    }
    eprintln!("{compared} facts the same on islands 1 and 2");
    assert!(compared > 100);
    // and the treated arm has its news of the hand
    assert!(groups(&ld.join("arm-1")).contains_key("news_hand"));
}

#[test]
fn an_arm_is_the_run_with_its_edits() {
    // (b) island 7: the treated arm's yearly means are those of edits::run_with
    let list = vec![Dated { before: (1790, 1), edit: Edit::Rule { name: "money.growth".into(), value: serde_json::json!(0) } }];
    let o = lab::parse_args(&strings(&["--out", "unused", "--seeds", "7", "--end-year", "1800", "--rule", "money.growth=0@1790", "--quiet"])).unwrap();
    let plan = lab::plan(&o).unwrap();
    assert_eq!(plan.arms[1].edits, list);
    let done = lab::run(&o, &plan);
    let arm = done.sweeps[1].runs[0].outcome.as_ref().expect("the arm runs");
    // the island the lab plays, as a sweep plays it: the game's rules, six families, to 1800, island 7
    let mut P = Rules::Game.params();
    (P.seed, P.n_families, P.end_year) = (7, 6, Some(1800));
    assert_eq!(P, { let mut Q = plan.params.clone(); Q.seed = 7; Q });
    let plain = facts::annual_means(&edits::run_with(&P, Rules::Game.content(), &list).unwrap());
    let bits = |a: &Vec<Vec<f64>>| a.iter().map(|s| s.iter().map(|x| x.to_bits()).collect::<Vec<_>>()).collect::<Vec<_>>();
    assert_eq!(bits(&arm.annual), bits(&plain));
}

#[test]
fn the_arms_part_at_the_edit() {
    // (c) money.growth set to nothing before January 1790: the arms part in 1790, and not before
    let o = lab::parse_args(&strings(&["--out", "unused", "--seeds", "1,2", "--end-year", "1795", "--rule", "money.growth=0@1790", "--quiet"])).unwrap();
    let plan = lab::plan(&o).unwrap();
    let done = lab::run(&o, &plan);
    for k in 0..2 {
        let (c, t) = (done.sweeps[0].runs[k].outcome.as_ref().unwrap(), done.sweeps[1].runs[k].outcome.as_ref().unwrap());
        assert_eq!(done.sweeps[0].runs[k].seed, done.sweeps[1].runs[k].seed);
        assert_eq!(lab::parting_year(c, t), Some(1790), "island {}", done.sweeps[0].runs[k].seed);
        for (a, b) in c.annual.iter().zip(&t.annual) {
            for y in 0..40 {
                assert!(a[y].to_bits() == b[y].to_bits() || (a[y].is_nan() && b[y].is_nan()), "before 1790 every yearly mean is the same");
            }
        }
    }
    let text = lab::table(&done, &plan);
    assert!(text.contains("the arms part in 1790 on 2 of 2 islands"), "{text}");
}

#[test]
fn what_is_wrong_is_refused_before_anything_is_written() {
    // (d) each with a sentence, and the output directory not made
    let dir = scratch("refused");
    let out = dir.to_str().unwrap();
    for (rule, says) in [
        (vec!["--rule", "seed=3@1790"], "sizes the island"),
        (vec!["--rule", "tasks.dial=10"], "sweep --set"),
        (vec!["--rule", "money.growth=0@1890"], "outside the run's years"),
        (vec!["--rule", "tasks.dial=3,10@1790", "--rule", "money.growth=0,0.01@1790"], "only one --rule may"),
        (vec![], "at least one edit"),
    ] {
        let mut args = strings(&["--out", out, "--seeds", "1", "--end-year", "1800", "--quiet"]);
        args.extend(strings(&rule));
        let why = lab::command(&args, false).expect_err(&format!("{rule:?} is refused"));
        eprintln!("{rule:?}: {why}");
        assert!(why.contains(says), "{rule:?}: {why}");
        assert!(!dir.exists(), "{rule:?}: nothing is written");
    }
    // a rule edit checked against the rules as the earlier ones leave them: set twice to the same
    let file = std::env::temp_dir().join(format!("isostate-lab-test-{}-twice.json", std::process::id()));
    std::fs::write(&file, r#"[{"before": [1780, 1], "edit": {"what": "rule", "name": "money.growth", "value": 0}}]"#).unwrap();
    let args = strings(&["--out", out, "--seeds", "1", "--end-year", "1800", "--quiet", "--edits", file.to_str().unwrap(), "--rule", "money.growth=0@1790"]);
    let why = lab::command(&args, false).unwrap_err();
    eprintln!("set twice: {why}");
    assert!(why.contains("is 0 already") && why.contains("before January 1790"), "{why}");
    assert!(!dir.exists());
}

#[test]
fn rules_and_their_values_parse() {
    // (e)
    let r = lab::parse_rule("tasks.dial=3,10,100@1950").unwrap();
    assert_eq!((r.name.as_str(), r.values.clone(), r.year), ("tasks.dial", vec!["3".to_string(), "10".into(), "100".into()], 1950));
    let r = lab::parse_rule("road_thresholds=[1500,9000,40000]@1800").unwrap();
    assert_eq!((r.name.as_str(), r.values.clone(), r.year), ("road_thresholds", vec!["[1500,9000,40000]".to_string()], 1800));
    let r = lab::parse_rule("road_thresholds=[1500,9000,40000],[1000,6000,30000]@1800").unwrap();
    assert_eq!(r.values.len(), 2);
    assert!(lab::parse_rule("tasks.dial=10@then").is_err());
    assert!(lab::parse_rule("tasks.dial@1900").is_err());
    assert!(lab::parse_rule("tasks.dial=3,3@1900").unwrap_err().contains("twice"));
    // one arm a value, labelled by it
    let o = lab::parse_args(&strings(&["--out", "unused", "--rule", "tasks.dial=3,10@1950"])).unwrap();
    let arms = lab::arms(&o, &[]);
    let labels: Vec<&str> = arms.iter().map(|a| a.label.as_str()).collect();
    assert_eq!(labels, ["control", "tasks.dial=3 from 1950", "tasks.dial=10 from 1950"]);
    assert_eq!((o.end_year, o.families, o.seeds.len(), o.rules), (2050, 6, 8, Rules::Game));
}
