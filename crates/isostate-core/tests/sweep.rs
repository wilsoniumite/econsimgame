//! The sweep end to end: a tiny one writes its files, every fact it gives is a
//! finite number, the facts every run must have are there, and a second sweep can
//! be set beside the first.

use std::path::PathBuf;

use isostate_core::cli::main_with_args;
use isostate_core::{default_content, default_params, facts, sim};
use serde_json::Value;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("isostate-sweep-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// `isostate-core sweep ARGS`, without the table on stdout (the harness would not hide it).
fn sweep(args: &[&str]) -> Result<(), String> {
    let args: Vec<String> = args.iter().copied().map(String::from).collect();
    isostate_core::sweep::command(&args, false)
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
            "area_farm_1760",
            "area_forestry_1760",
            "area_mines_1760",
            "area_wild_1760",
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

    // the first sweep again, over itself: what was there is read, compared and then replaced,
    // and since a run is the same run every time, nothing has changed
    sweep(&["--seeds", "1,2", "--end-year", "1760", "--out", out, "--compare", out, "--jobs", "1", "--quiet"]).expect("the sweep runs again");
    let again = std::fs::read_to_string(dir.join("summary.txt")).unwrap();
    assert!(again.contains("now: port rules (2 runs); then: port rules (2 runs)"), "{again}");
    assert!(again.contains("the seconds aside, 0 differ and 0 of those are starred"), "{again}");
    assert_eq!(std::fs::read_to_string(dir.join("annual.csv")).unwrap().lines().collect::<Vec<_>>(), lines);

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);
}

/// Four facts cannot be worked out from the files a run writes. Here they are
/// worked out a second way from what the run keeps in memory.
#[test]
#[allow(non_snake_case)]
fn the_facts_no_file_holds_agree_with_the_record() {
    let mut P = default_params();
    P.seed = 3;
    P.end_year = Some(1790);
    let mut s = sim::start(&P, default_content());
    let mut gathered = facts::Gathered::default();
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved");
        facts::gather_year(&mut gathered, &s);
    }
    let r = sim::finish(s);
    let f = facts::stylised_facts(&r, &gathered, 0.0);
    let (st, C, rec) = (&r.state, &r.content, &r.rec);

    // growth against the cap: the sweep notes it between years from the live state. The yearly
    // snapshot keeps each town's income ratio and people (as f32), which is another road to it.
    let T = rec.T;
    let mut share = Vec::new();
    for k in 0..rec.n_years {
        let (mut capped, mut all) = (0.0, 0.0);
        for t in (0..T).filter(|&t| rec.town_active[k * T + t] == 1 && rec.town_y[k * T + t] > 0.0) {
            let (y, pop) = (rec.town_y[k * T + t] as f64, rec.town_pop[k * T + t] as f64);
            all += pop;
            if P.growth_slope * y.ln() + P.growth_base >= P.growth_cap {
                capped += pop;
            }
        }
        share.push(capped / all);
    }
    let people = share.iter().sum::<f64>() / share.len() as f64;
    let years = share.iter().filter(|&&v| v >= 0.5).count() as f64 / share.len() as f64;
    // not every year of this run sits at the cap, or the check would show nothing
    assert!(people > 0.5 && people < 0.999, "{people}");
    assert!((f.get("growth_capped_people").unwrap() - people).abs() < 1e-4, "{:?} against {people}", f.get("growth_capped_people"));
    assert_eq!(f.get("growth_capped_years"), Some(years));

    // utilisation and the top hundredth's share of capacity, from the final state
    let alive: Vec<usize> = (0..st.F_count).filter(|&w| st.f_alive[w]).collect();
    let built: Vec<f64> = alive.iter().filter(|&&w| st.f_K[w] > 0.0).map(|&w| st.f_u[w]).collect();
    let util = built.iter().sum::<f64>() / built.len() as f64;
    assert!((f.get("works_utilisation_1790").unwrap() - util).abs() < 1e-12);
    let mut value: Vec<f64> = Vec::new();
    for &w in &alive {
        let mut unit = 0.0;
        for g in 0..C.G {
            unit += C.m_cap[(st.f_method[w], g)] * st.pbar[(st.f_town[w], g)];
        }
        value.push((st.f_K[w] + st.f_order_K[w]) * unit);
    }
    value.sort_by(f64::total_cmp);
    let top = value.len().div_ceil(100);
    let share = value[value.len() - top..].iter().sum::<f64>() / value.iter().sum::<f64>();
    assert!(top >= 5 && share > 0.01 && share < 0.5, "{top} works hold {share}");
    assert!((f.get("works_top_1pct_capacity_share_1790").unwrap() - share).abs() < 1e-12);

    // the land by area: the last year's is the final state's shares summed, there are woods
    // though hardly a cell has them as its main use, and the whole is no more than the island
    let area = |name: &str| f.get(&format!("area_{name}_1790")).unwrap();
    let row = |r: usize| (0..st.N).map(|c| st.lam[(r, c)]).sum::<f64>();
    let sector = |name: &str| C.sector_name.iter().position(|s| s == name).unwrap();
    assert!((area("farm") - row(sector("farm"))).abs() < 1e-9);
    assert!((area("forestry") - row(sector("forestry"))).abs() < 1e-9);
    assert!((area("wild") - row(C.NS)).abs() < 1e-9);
    let woods_lead = (0..st.N).filter(|&c| st.land_use[c] == isostate_core::land::USE_SECTOR0 + sector("forestry") as u8).count();
    assert!(area("forestry") > 500.0 && woods_lead < 50, "{} cells' worth of woods, the main use of {woods_lead}", area("forestry"));
    let shared: f64 = ["farm", "pasture", "forestry", "mines", "wild"].iter().map(|n| area(n)).sum();
    let land = st.world.land.iter().filter(|&&l| l).count() as f64;
    assert!(shared > 0.5 * land && shared <= land + 1e-6, "{shared} cells' worth shared out on an island of {land}");
}

#[test]
fn a_bad_setting_is_refused_before_anything_runs() {
    let dir = scratch("bad");
    let out = dir.to_str().unwrap();
    let e = sweep(&["--seeds", "1", "--set", "land_thetta=2,3", "--out", out, "--quiet"]).unwrap_err();
    assert!(e.contains("Did you mean land_theta?"), "{e}");
    // the same through the command line's front door
    let front: Vec<String> = ["sweep", "--seeds", "1", "--set", "land_thetta=2,3", "--out", out].iter().map(|s| s.to_string()).collect();
    assert_eq!(main_with_args(&front).unwrap_err(), e);
    let e = sweep(&["--seeds", "1", "--set", "land_theta=wide", "--out", out, "--quiet"]).unwrap_err();
    assert!(e.contains("land_theta") && e.contains("invalid type"), "{e}");
    let e = sweep(&["--seeds", "1", "--compare", out, "--out", out, "--quiet"]).unwrap_err();
    assert!(e.contains("--compare") && e.contains("facts.json"), "{e}");
    let e = sweep(&["--seeds", "1", "--set", "land_theta=2,3,2", "--out", out, "--quiet"]).unwrap_err();
    assert!(e.contains("listed twice"), "{e}");
    let e = sweep(&["--seeds", "0-99999999999", "--out", out, "--quiet"]).unwrap_err();
    assert!(e.contains("more than 10000 seeds"), "{e}");
    // a run that would end before it starts
    let e = sweep(&["--seeds", "1", "--end-year", "1749", "--out", out, "--quiet"]).unwrap_err();
    assert!(e.contains("1749") && e.contains("before the start year, 1750"), "{e}");
    let e = sweep(&["--seeds", "1", "--set", "end_year=1760,1700", "--out", out, "--quiet"]).unwrap_err();
    assert!(e.contains("1700") && e.contains("before the start year"), "{e}");
    assert!(!dir.exists(), "nothing is written when the command line is wrong");

    // a place that cannot be written to is found out before the runs, not after them
    std::fs::write(&dir, "a file where the directory should go").unwrap();
    let t0 = std::time::Instant::now();
    let e = sweep(&["--seeds", "1-8", "--out", out, "--quiet"]).unwrap_err();
    assert!(e.contains("--out"), "{e}");
    assert!(t0.elapsed().as_secs_f64() < 5.0, "eight whole runs were made before --out was found to be a file");
    let _ = std::fs::remove_file(&dir);
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
