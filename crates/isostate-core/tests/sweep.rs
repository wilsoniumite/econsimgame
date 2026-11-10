//! The sweep end to end: a tiny one writes its files, every fact it gives is a
//! finite number, the facts every run must have are there, and a second sweep can
//! be set beside the first.

use std::path::PathBuf;

use isostate_core::cli::main_with_args;
use serde_json::Value;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("isostate-sweep-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn sweep(args: &[&str]) -> Result<(), String> {
    let args: Vec<String> = std::iter::once("sweep").chain(args.iter().copied()).map(String::from).collect();
    main_with_args(&args)
}

#[test]
fn a_tiny_sweep_writes_finite_facts() {
    let dir = scratch("tiny");
    let out = dir.to_str().unwrap();
    sweep(&["--seeds", "1,2", "--end-year", "1760", "--out", out, "--quiet"]).expect("the sweep runs");

    let doc: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("facts.json")).unwrap()).unwrap();
    let runs = doc["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!((runs[0]["seed"].as_u64(), runs[1]["seed"].as_u64()), (Some(1), Some(2)));
    for run in runs {
        assert_eq!(run["ok"], Value::Bool(true), "run failed: {}", run["error"]);
        let facts = run["facts"].as_object().unwrap();
        // every fact is a finite number, or null where it has no value (no railway by 1760)
        for (name, v) in facts {
            assert!(v.is_null() || v.as_f64().is_some_and(f64::is_finite), "{name} = {v}");
        }
        // these have a value in any run
        for name in [
            "pop_1750",
            "pop_1760",
            "growth_capped_years",
            "growth_capped_people",
            "urban_share_1760",
            "towns_1760",
            "farm_share_1760",
            "industry_share_1760",
            "real_wage_1760_vs_1750",
            "output_per_head_1760_vs_1750",
            "output_per_head_peak_year",
            "rail_km_1760",
            "works_1760",
            "works_ex_services_1760",
            "works_per_1000_people_1760",
            "works_size_median_1760",
            "works_size_p90_1760",
            "works_size_largest_1760",
            "works_top_1pct_capacity_share_1760",
            "works_closed_median_life",
            "works_utilisation_1760",
            "cells_urban_1760",
            "cells_farm_1760",
            "cells_mines_1760",
            "urban_churn_10y",
            "price_swing_grain",
            "price_swing_iron",
            "price_swing_rails",
            "price_swing_cloth",
            "farm_zigzag",
            "news_total",
            "seconds",
        ] {
            assert!(facts.get(name).and_then(Value::as_f64).is_some(), "{name} is missing or has no value: {:?}", facts.get(name));
        }
        // and are of a sensible size
        let f = |name: &str| facts[name].as_f64().unwrap();
        assert_eq!(f("pop_1750"), 135_000.0);
        assert!(f("pop_1760") > 100_000.0 && f("pop_1760") < 200_000.0);
        assert!((0.0..=1.0).contains(&f("growth_capped_years")) && (0.0..=1.0).contains(&f("urban_churn_10y")));
        assert!(f("farm_share_1760") + f("industry_share_1760") <= 1.0 + 1e-9);
        assert!(f("works_ex_services_1760") <= f("works_1760"));
        assert!(f("works_size_median_1760") <= f("works_size_p90_1760") && f("works_size_p90_1760") <= f("works_size_largest_1760"));
        // nothing is invented in ten years that then has the time to be built everywhere
        assert!(run["never_built"].as_array().unwrap().iter().any(|m| m == "bessemer"));
        assert!(facts["discovered_railway"].is_null());
    }

    // annual.csv: a header, then eleven years for each of the two runs, every row as wide as the header
    let annual = std::fs::read_to_string(dir.join("annual.csv")).unwrap();
    let lines: Vec<&str> = annual.lines().collect();
    assert_eq!(lines.len(), 1 + 2 * 11);
    assert!(lines[0].starts_with("run,seed,year,pop,urban_share,"));
    let width = lines[0].split(',').count();
    assert!(lines.iter().all(|l| l.split(',').count() == width));
    assert!(lines[1].starts_with("0,1,1750,135000,") && lines[22].starts_with("1,2,1760,"));

    let facts_csv = std::fs::read_to_string(dir.join("facts.csv")).unwrap();
    assert_eq!(facts_csv.lines().count(), 3);
    assert!(facts_csv.lines().nth(1).unwrap().starts_with("0,1,ok,,135000,"));
    for name in ["summary.txt", "README.txt"] {
        assert!(std::fs::metadata(dir.join(name)).unwrap().len() > 0, "{name} is empty");
    }
    let summary = std::fs::read_to_string(dir.join("summary.txt")).unwrap();
    assert!(summary.contains("People and towns") && summary.contains("pop_1760") && summary.contains("never built, port rules:"));

    // a second sweep over two settings, set beside the first
    let dir2 = scratch("tiny2");
    sweep(&["--seeds", "1", "--end-year", "1755", "--set", "growth_cap=0.011,0.02", "--out", dir2.to_str().unwrap(), "--compare", out, "--quiet"])
        .expect("the second sweep runs");
    let summary = std::fs::read_to_string(dir2.join("summary.txt")).unwrap();
    assert!(summary.contains("growth_cap=0.011") && summary.contains("growth_cap=0.02"));
    assert!(summary.contains("now: growth_cap=0.02 (1 run); then: port rules (2 runs)"), "{summary}");
    let annual = std::fs::read_to_string(dir2.join("annual.csv")).unwrap();
    assert!(annual.starts_with("run,seed,growth_cap,year,pop,"));
    assert!(annual.lines().nth(7).unwrap().starts_with("1,1,0.02,1750,"));

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);
}

#[test]
fn a_bad_setting_is_refused_before_anything_runs() {
    let dir = scratch("bad");
    let out = dir.to_str().unwrap();
    let e = sweep(&["--seeds", "1", "--set", "land_thetta=2,3", "--out", out, "--quiet"]).unwrap_err();
    assert!(e.contains("Did you mean land_theta?"), "{e}");
    let e = sweep(&["--seeds", "1", "--set", "land_theta=wide", "--out", out, "--quiet"]).unwrap_err();
    assert!(e.contains("land_theta") && e.contains("invalid type"), "{e}");
    let e = sweep(&["--seeds", "1", "--compare", out, "--out", out, "--quiet"]).unwrap_err();
    assert!(e.contains("--compare") && e.contains("facts.json"), "{e}");
    assert!(!dir.exists(), "nothing is written when the command line is wrong");
}

#[test]
fn a_run_that_fails_does_not_stop_the_others() {
    // a town table of two slots cannot hold the six towns of 1750: that run panics
    // (its message appears among the test's output), and the other still finishes
    let dir = scratch("fail");
    let out = dir.to_str().unwrap();
    let r = sweep(&["--seeds", "1", "--end-year", "1751", "--set", "T_max=48,2", "--out", out, "--quiet"]);
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("facts.json")).unwrap()).unwrap();
    let runs = doc["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0]["ok"], Value::Bool(true));
    assert_eq!(runs[1]["ok"], Value::Bool(false), "six towns in a table of two should not run");
    assert!(runs[1]["error"].as_str().is_some_and(|e| e.starts_with("panicked: ")), "{}", runs[1]["error"]);
    assert!(r.unwrap_err().contains("1 of 2 runs failed"));
    let facts_csv = std::fs::read_to_string(dir.join("facts.csv")).unwrap();
    assert!(facts_csv.lines().nth(2).unwrap().starts_with("1,1,2,failed,"), "{facts_csv}");
    let summary = std::fs::read_to_string(dir.join("summary.txt")).unwrap();
    assert!(summary.contains("FAILED run 1 (seed 1, T_max=2): panicked: "), "{summary}");
    let _ = std::fs::remove_dir_all(&dir);
}
