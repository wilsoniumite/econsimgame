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
            "mine_share_1760",
            "grain_output_1760_vs_1750",
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
            "price_tools_dearest_vs_median",
            "price_tools_dearest_year",
            "price_tools_years_over_twice_median",
            "real_wage_worst_year_median_town",
            "farm_zigzag_beyond_weather",
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
        // the facts named by their place in the run are the facts of those years, and a run of
        // ten years has no twenty-year mean
        for (place, year) in [("pop_last", "pop_1760"), ("pop_mid", "pop_1755"), ("urban_share_first", "urban_share_1750"), ("works_last", "works_1760"),
            ("real_wage_last_vs_first", "real_wage_1760_vs_1750"), ("rail_km_last", "rail_km_1760"), ("works_size_p90_last", "works_size_p90_1760")]
        {
            let same = if year == "pop_1755" { facts.get(place).and_then(Value::as_f64).is_some() } else { facts.get(place) == facts.get(year) };
            assert!(same, "{place} = {:?} against {year} = {:?}", facts.get(place), facts.get(year));
        }
        assert!(!facts.contains_key("urban_share_last20") && !facts.keys().any(|k| k.starts_with("urban_share_17") && k.len() > "urban_share_1750".len()),"{:?}", facts.keys());
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
    // the scorecard is written beside them, and says which aims a run to 1760 cannot be read on
    let card = std::fs::read_to_string(dir.join("scorecard.txt")).unwrap();
    assert!(card.contains("Scorecard: port rules, 2 runs (seeds 1 2)") && card.contains("fewer than 16 islands"), "{card}");
    assert!(card.contains("urban_share_1881_1900") && card.contains("PLAN.md 1.2") && card.contains("\nloss: "), "{card}");
    assert_eq!(isostate_core::scorecard::card(&dir, None).unwrap(), card, "the command prints what the sweep wrote");
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

/// Every fact a run under the game's rules had on 12 November 2026 (main at 295a379), for a
/// run to 1760 with six families, the news aside (its kinds come and go with the run).
const FACTS_OF_MAIN: &str = "
    pop_1750 pop_1760 growth_capped_years growth_capped_people town_growth_pct_p10 town_growth_pct_p50
    town_growth_pct_p90 town_growth_near_old_cap urban_share_1750 urban_share_1760 towns_1750 towns_1760
    last_town_founded late_towns late_towns_coal late_towns_iron late_towns_stone late_towns_works
    late_towns_farm late_towns_port late_towns_rail late_town_people_median late_town_people_largest emigrants
    farm_share_1750 farm_share_1760 industry_share_1750 industry_share_1760 real_wage_1760_vs_1750
    output_per_head_1760_vs_1750 output_per_head_peak_year mine_share_1750 mine_share_1760
    grain_output_1760_vs_1750 real_wage_chained_1760_vs_1750 real_income_1750 real_income_1760 budget_food_1750
    budget_food_1760 budget_warmth_1750 budget_warmth_1760 budget_shelter_1750 budget_shelter_1760
    budget_clothing_1750 budget_clothing_1760 budget_implements_1750 budget_implements_1760 budget_comfort_1750
    budget_comfort_1760 baskets_per_head_1750 baskets_per_head_1760 goods_per_head_1750 goods_per_head_1760
    carting_share_of_output_1750 carters_share_of_hands_1750 carting_share_of_output_1760
    carters_share_of_hands_1760 income_share_work_1750 income_share_work_1760 income_share_works_1750
    income_share_works_1760 income_share_merchants_1750 income_share_merchants_1760 income_share_land_1750
    income_share_land_1760 income_pay_share_1750 income_pay_share_1760 income_ownership_years_1750
    income_ownership_years_1760 books_goods_worst books_income_worst rail_first_year rail_km_1760
    rail_last_growth_year rail_towns_with_track_1760 rail_traffic_near_track_1760 rail_cost_in_years_of_output
    rail_most_lines_building rail_slowest_line_years rail_lines_stuck_at_end rail_fares_over_capital_1760
    rail_spent_over_expected rail_worst_overrun rail_lines_trunk rail_lines_to_towns rail_lines_to_farms
    rail_lines_to_pastures rail_lines_to_quarries rail_lines_to_mines rail_lines_to_works rail_lines_to_woods
    rail_km_share_branches works_1750 works_1760 works_ex_services_1750 works_ex_services_1760
    works_per_1000_people_1760 works_size_median_1760 works_size_p90_1760 works_size_largest_1760
    works_top_1pct_capacity_share_1760 works_founded works_closed works_closed_median_life works_utilisation_1760
    cells_urban_1750 cells_urban_1760 cells_industry_1750 cells_industry_1760 cells_derelict_1750
    cells_derelict_1760 area_farm_1750 area_farm_1760 area_pasture_1750 area_pasture_1760 area_forestry_1750
    area_forestry_1760 area_mines_1750 area_mines_1760 area_wild_1750 area_wild_1760 workings_1750
    workings_quarry_1750 workings_claypit_1750 workings_ironmine_1750 workings_coalmine_1750
    workings_share_of_cell_1750 workings_most_of_cell_1750 hands_quarry_1750 hands_claypit_1750
    hands_ironmine_1750 hands_coalmine_1750 out_stone_per_head_1750 price_stone_in_wages_1750
    out_clay_per_head_1750 price_clay_in_wages_1750 out_iron_ore_per_head_1750 price_iron_ore_in_wages_1750
    out_coal_per_head_1750 price_coal_in_wages_1750 royalties_share_of_output_1750 workings_1760
    workings_quarry_1760 workings_claypit_1760 workings_ironmine_1760 workings_coalmine_1760
    workings_share_of_cell_1760 workings_most_of_cell_1760 hands_quarry_1760 hands_claypit_1760
    hands_ironmine_1760 hands_coalmine_1760 out_stone_per_head_1760 price_stone_in_wages_1760
    out_clay_per_head_1760 price_clay_in_wages_1760 out_iron_ore_per_head_1760 price_iron_ore_in_wages_1760
    out_coal_per_head_1760 price_coal_in_wages_1760 royalties_share_of_output_1760 urban_churn_10y
    discovered_coke_furnace first_output_coke_furnace half_of_output_coke_furnace discovered_textile_mill
    first_output_textile_mill half_of_output_textile_mill discovered_engine_works first_output_engine_works
    half_of_output_engine_works discovered_crucible_steel first_output_crucible_steel
    half_of_output_crucible_steel discovered_bessemer first_output_bessemer half_of_output_bessemer
    discovered_rolling_mill first_output_rolling_mill half_of_output_rolling_mill discovered_steel_rolling
    first_output_steel_rolling half_of_output_steel_rolling discovered_railway discovered_improved_husbandry
    discovered_farm_machinery price_swing_grain price_swing_iron price_swing_rails price_swing_cloth farm_zigzag
    output_swing_grain price_ac1_grain output_ac1_grain real_wage_worst_year real_wage_swing harvest_sd
    grain_output_on_harvest grain_price_on_harvest real_wage_on_harvest grain_price_year_after_harvest
    real_wage_year_after_harvest famine_years seconds canal_first_year canal_first_begun canal_km_1760
    canal_last_begun canal_before_railway canal_years_before_railway canal_mton_km_1760 canal_haul_km_1760
    canal_cells_in_use_1760 canal_carries_coal_1760 canal_carries_stone_clay_1760 canal_carries_grain_1760
    canal_carries_timber_1760 canal_carries_iron_ore_1760 canal_carries_works_goods_1760 canal_traffic_peak_year
    canal_traffic_end_vs_peak canal_works_near_1760 canal_land_near canal_km_beside_track_1760 canal_lines_trunk
    canal_junctions_cut canal_lines_to_collieries canal_lines_to_quarries canal_lines_to_iron_mines
    canal_lines_to_farms canal_lines_to_woods canal_lines_to_works canal_cost_in_years_of_output
    canal_spent_over_expected canal_slowest_years canal_most_digging canal_most_navvies
    canal_fares_over_capital_1760";

/// Facts are added and never renamed (docs/rules/scorecard.md): a fact a sweep had
/// yesterday is one today's sweep can be compared on. A run to 1760 is enough to see
/// a name, since the names of years are made the same way whatever the end year.
#[test]
#[allow(non_snake_case)]
fn no_fact_main_had_is_lost_or_renamed() {
    let rules = isostate_core::params::Rules::Game;
    let mut P = rules.params();
    P.seed = 1;
    P.n_families = 6;
    P.end_year = Some(1760);
    let mut s = sim::start(&P, rules.content());
    let mut gathered = facts::Gathered::default();
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved");
        facts::gather_year(&mut gathered, &s);
    }
    let f = facts::stylised_facts(&sim::finish(s), &gathered, 0.0);
    let missing: Vec<&str> = FACTS_OF_MAIN.split_whitespace().filter(|&name| !f.facts.iter().any(|x| x.name == name)).collect();
    assert!(missing.is_empty(), "facts main had and this code does not give: {missing:?}");
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
