//! The table of years abroad knows each idea (`data/abroad.toml`,
//! docs/rules/abroad.md), made from the content and a sweep of the settled
//! island:
//!
//!     isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --out DIR --quiet
//!     cargo run --profile work -p isostate-core --example abroad_table -- DIR/facts.json > data/abroad.toml
//!
//! An idea of the tree or a use of the tasks rule takes history's year, its
//! `target` (`data/game.toml`, `data/tasks.toml`), where the content gives
//! one, and never before the content's first year for it (`found_after`).
//! Else it takes the median of the years the sweep's islands found it
//! (`discovered_NAME`, `use_found_year_NAME`), an island that never found it
//! counted as the latest; where that median is "never", abroad does not know
//! it and it is left out.
#![allow(non_snake_case)]
use isostate_core::*;

fn median(ys: &[Option<i64>]) -> Option<i64> {
    let mut v: Vec<i64> = ys.iter().map(|y| y.unwrap_or(i64::MAX)).collect();
    v.sort();
    let n = v.len();
    if n == 0 {
        return None;
    }
    let (a, b) = (v[(n - 1) / 2], v[n / 2]);
    if b == i64::MAX {
        return None;
    }
    Some(((a + b) as f64 / 2.0).round() as i64)
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let text = std::fs::read_to_string(&a[0]).expect("a sweep's facts.json");
    let facts: serde_json::Value = serde_json::from_str(&text).unwrap();
    let runs = facts["runs"].as_array().expect("runs");
    let years = |key: &str| -> Vec<Option<i64>> { runs.iter().map(|r| r["facts"][key].as_f64().map(|y| y as i64)).collect() };
    let seeds: Vec<String> = runs.iter().map(|r| r["seed"].to_string()).collect();
    let P = game_params();
    let C = tree::content(&P, game_content()).unwrap();
    let show = |ys: &[Option<i64>]| ys.iter().map(|y| y.map_or("never".to_string(), |y| y.to_string())).collect::<Vec<_>>().join(" ");
    println!("# The year abroad knows each idea of the tree and each use of the tasks rule");
    println!("# (docs/rules/abroad.md): history's year, the content's `target`, where it gives one");
    println!("# (never before its `found_after`), else the median of the years the settled island");
    println!("# found it under the game's rules with six families on islands {} to 2050.", seeds.join(", "));
    println!("# An idea that median leaves at \"never\" is not known abroad and has no line.");
    println!("# Made by examples/abroad_table.rs; tests/abroad.rs says when it has gone stale.");
    println!();
    println!("[idea]");
    for t in (0..C.NT).filter(|&t| !C.tech_known[t]) {
        let name = &C.tech_name[t];
        let after = C.tech_after.get(t).copied().unwrap_or(0);
        let ys = years(&format!("discovered_{name}"));
        match C.tech_target.get(t).copied().filter(|&y| y > 0) {
            Some(y) => println!("{name} = {}  # target {y}{}", y.max(after), if after > y { format!(", found after {after}") } else { String::new() }),
            None => match median(&ys) {
                Some(y) => println!("{name} = {y}  # the settled island: {}", show(&ys)),
                None => println!("# {name}: not known abroad (the settled island: {})", show(&ys)),
            },
        }
    }
    println!();
    println!("[use]");
    for u in &C.tasks.uses {
        let ys = years(&format!("use_found_year_{}", u.name));
        match u.target {
            Some(y) => println!("{} = {}  # target {y}{}", u.name, y.max(u.found_after), if u.found_after > y { format!(", found after {}", u.found_after) } else { String::new() }),
            None => match median(&ys) {
                Some(y) => println!("{} = {y}  # the settled island: {}", u.name, show(&ys)),
                None => println!("# {}: not known abroad (the settled island: {})", u.name, show(&ys)),
            },
        }
    }
}
