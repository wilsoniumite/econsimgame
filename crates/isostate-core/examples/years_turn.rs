//! The year's turn, town by town (docs/rules/people.md, "The hands that come
//! with a district"): one island under the game's rules with six families to
//! 2050, any setting by name, and three files.
//!
//!     cargo run --profile work -p isostate-core --example years_turn -- SEED OUT_PREFIX [--start settled|landfall] [--set key=value]... [--end YEAR]
//!
//! - `OUT_PREFIX.years.jsonl`, a line a year: the island's people, towns and
//!   towns founded, the hungry towns (the `famine` news), the linked wage, the
//!   largest rise of a town's yearly mean wage over the year before (for towns
//!   with people in the two years before, as `landfall::facts` reads it), and
//!   the small poor towns (under 2,000 people at under a fifth of the island's
//!   wage) with the share of their hands that cart;
//! - `OUT_PREFIX.rises.jsonl`, a line for each town-year whose mean wage rose
//!   three times or more: the town's months of that year and the year before
//!   (people, hands for hire, carters, hands carried into the month on the land
//!   and in the works, hands at work, the wage, whether it was held at the
//!   month's top, grain made, grain bought, the share of food got), and the town
//!   just before and just after the year's turn;
//! - `OUT_PREFIX.facts.json`, the run's stylised facts.
#![allow(non_snake_case)]
use isostate_core::*;
use std::io::Write;

/// One town's month, as the probe keeps it.
#[derive(Clone, Copy, Default)]
struct Month {
    pop: f64,
    hire: f64,
    carters: f64,
    carried_land: f64,
    carried_works: f64,
    at_work: f64,
    wage: f64,
    top: bool,
    grain_made: f64,
    grain_bought: f64,
    food: f64,
    /// hands at work on the land, by sector, and the price of each sector's good
    by_sector: [f64; 12],
    price: [f64; 12],
}

impl Month {
    fn json(&self) -> serde_json::Value {
        serde_json::json!([self.pop, self.hire, self.carters, self.carried_land, self.carried_works, self.at_work,
            self.wage, self.top, self.grain_made, self.grain_bought, self.food, self.by_sector.to_vec(), self.price.to_vec()])
    }
}

/// A town at an instant: people, hands carried on the land and in the works,
/// carters and the wage.
fn snap(st: &state::State, C: &content::Content, t: usize) -> serde_json::Value {
    let land: f64 = (0..st.ex_cell.len().min(st.ex_Lprev.len())).filter(|&u| st.ex_town[u] == t).map(|u| st.ex_Lprev[u]).sum();
    let works: f64 = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_town[f] == t).map(|f| st.f_labor[f]).sum();
    // the price of each sector's good here, and its median over the towns with people
    let act: Vec<usize> = (0..st.T_max).filter(|&k| st.town_active[k] && st.pop[k] > 0.0).collect();
    let here: Vec<f64> = (0..C.NS).map(|s| st.price[(t, C.sector_good[s])]).collect();
    let median: Vec<f64> = (0..C.NS)
        .map(|s| {
            let mut v: Vec<f64> = act.iter().map(|&k| st.price[(k, C.sector_good[s])]).collect();
            v.sort_by(f64::total_cmp);
            if v.is_empty() { f64::NAN } else { v[v.len() / 2] }
        })
        .collect();
    let mut wages: Vec<f64> = act.iter().map(|&k| st.wage[k]).collect();
    wages.sort_by(f64::total_cmp);
    serde_json::json!({"pop": st.pop[t], "carried_land": land, "carried_works": works,
        "carters": st.carters.get(t).copied().unwrap_or(0.0), "wage": st.wage[t],
        "hh_income_prev": st.hh_income_prev[t], "price": here, "price_median": median,
        "wage_median": wages.get(wages.len() / 2).copied().unwrap_or(f64::NAN)})
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let mut P = game_params();
    P.seed = a[0].parse().unwrap();
    P.n_families = 6;
    P.end_year = Some(2050);
    let out = a[1].clone();
    let mut i = 2;
    let mut sets: Vec<(String, String)> = Vec::new();
    while i < a.len() {
        match a[i].as_str() {
            "--start" => {
                landfall::set_start(&mut P, &a[i + 1]).unwrap();
                i += 1;
            }
            "--set" => {
                let (k, v) = a[i + 1].split_once('=').expect("--set key=value");
                sets.push((k.to_string(), v.to_string()));
                i += 1;
            }
            "--end" => {
                P.end_year = Some(a[i + 1].parse().unwrap());
                i += 1;
            }
            s => panic!("unknown {s}"),
        }
        i += 1;
    }
    for (k, v) in &sets {
        P = sweep::set_field(&P, k, &sweep::parse_value(v)).unwrap();
    }
    let mut fy = std::fs::File::create(format!("{out}.years.jsonl")).unwrap();
    let mut fr = std::fs::File::create(format!("{out}.rises.jsonl")).unwrap();
    let t0 = std::time::Instant::now();
    let mut sim = sim::start(&P, game_content());
    let mut g = facts::Gathered::default();
    let T = sim.st.T_max;
    let grain = sim.content.gi("grain");
    let mut wbar_was = vec![0.0f64; T];
    let mut lived = vec![0usize; T]; // years running the town has had people
    let mut last: Vec<[Month; 12]> = vec![[Month::default(); 12]; T];
    // each town just before and just after the year's turn into this year
    let mut turn: Vec<serde_json::Value> = vec![serde_json::Value::Null; T];
    let mut failed: Option<String> = None;
    while sim.year <= sim.end_year {
        let year = sim.year;
        let ev0 = sim.events.len();
        let mut now: Vec<[Month; 12]> = vec![[Month::default(); 12]; T];
        // `town_prod` is set back to nothing at each year's end (`sim::yearly`)
        let mut made_was = vec![0.0f64; T];
        for month in 0..12 {
            let st = &sim.st;
            let prev = st.wage.clone();
            let (_, w_hi) = longrun::wage_bounds(st, &sim.params);
            let mut cl = vec![0.0; T];
            for u in 0..st.ex_cell.len().min(st.ex_Lprev.len()) {
                cl[st.ex_town[u]] += st.ex_Lprev[u];
            }
            let mut cw = vec![0.0; T];
            for f in 0..st.F_count {
                if st.f_alive[f] {
                    cw[st.f_town[f]] += st.f_labor[f];
                }
            }
            if let Err(e) = sim::run_month(&mut sim, month) {
                failed = Some(format!("{year}-{month}: {e}"));
                break;
            }
            facts::gather_month(&mut g, &sim, month);
            let st = &sim.st;
            let sr = sim.params.hands.short_rise;
            for t in 0..T {
                if !st.town_active[t] || st.pop[t] <= 0.0 {
                    continue;
                }
                let top = if prev[t] > 0.0 {
                    let hi = (prev[t] * 2.0).min(w_hi);
                    let top = if sr < 2.0 { hi.min(prev[t] * sr) } else { hi };
                    (st.wage[t] - top).abs() <= 1e-9 * top
                } else {
                    false
                };
                let made = st.town_prod[(t, grain)] - made_was[t];
                made_was[t] = st.town_prod[(t, grain)];
                let (mut by_sector, mut price) = ([0.0; 12], [0.0; 12]);
                for u in 0..st.ex_cell.len().min(st.ex_labor.len()) {
                    if st.ex_town[u] == t && st.ex_sector[u] < 12 {
                        by_sector[st.ex_sector[u]] += st.ex_labor[u];
                    }
                }
                for s in 0..sim.content.NS.min(12) {
                    price[s] = st.price[(t, sim.content.sector_good[s])];
                }
                now[t][month] = Month {
                    pop: st.pop[t],
                    hire: st.month_stats.supply.get(t).copied().unwrap_or(f64::NAN),
                    carters: st.carters.get(t).copied().unwrap_or(0.0),
                    carried_land: cl[t],
                    carried_works: cw[t],
                    at_work: st.month_stats.employment.get(t).copied().unwrap_or(f64::NAN),
                    wage: st.wage[t],
                    top,
                    grain_made: made,
                    grain_bought: st.month_stats.hh_got[(t, grain)] + st.month_stats.lord_got[(t, grain)],
                    food: st.food_ratio[t],
                    by_sector,
                    price,
                };
            }
        }
        if failed.is_some() {
            writeln!(fy, "{}", serde_json::json!({"failed": failed})).unwrap();
            break;
        }
        // the town just before the year's turn (December's state)
        let before: Vec<serde_json::Value> = (0..T).map(|t| snap(&sim.st, &sim.content, t)).collect();
        sim::end_year(&mut sim);
        facts::gather_year(&mut g, &sim);
        let st = &sim.st;
        let s = |n: &str| observer::annual(st.series.get(n), sim.content.start_year, year);
        let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t] && st.pop[t] > 0.0).collect();
        let pop: f64 = act.iter().map(|&t| st.pop[t]).sum();
        let founded = act.iter().filter(|&&t| st.town_founded[t] == year).count();
        let mut hungry: Vec<serde_json::Value> = Vec::new();
        for e in &sim.events[ev0..] {
            if e.kind == "famine" {
                let t = e.town;
                hungry.push(serde_json::json!({"town": t, "name": t.map(|t| st.town_name[t].clone()),
                    "pop": t.map(|t| st.pop[t]), "wbar": t.map(|t| st.wbar[t]),
                    "rise": t.map(|t| if wbar_was[t] > 0.0 { st.wbar[t] / wbar_was[t] } else { f64::NAN })}));
            }
        }
        // the island's mean wage, by people
        let (mut sw, mut sp) = (0.0, 0.0);
        for &t in &act {
            sw += st.pop[t] * st.wbar[t];
            sp += st.pop[t];
        }
        let mean_w = sw / sp.max(1e-9);
        // small poor towns: under 2,000 people at under a fifth of the island's wage
        let mut poor = Vec::new();
        for &t in &act {
            if st.pop[t] < 2000.0 && st.wbar[t] < 0.2 * mean_w {
                let carting: f64 = now[t].iter().map(|m| m.carters).sum();
                let hands: f64 = now[t].iter().map(|m| m.carters + m.hire).sum();
                poor.push(serde_json::json!({"town": t, "name": st.town_name[t], "pop": st.pop[t],
                    "wage_share": st.wbar[t] / mean_w, "carting_share": carting / hands.max(1e-9),
                    "founded": st.town_founded[t]}));
            }
        }
        // yearly rises of the towns' mean wages
        let mut worst = (0.0f64, -1i64);
        for &t in &act {
            let w = st.wbar[t];
            if lived[t] >= 2 && wbar_was[t] > 0.0 && w > 0.0 {
                let r = w / wbar_was[t];
                if r > worst.0 {
                    worst = (r, t as i64);
                }
                if r >= 3.0 {
                    writeln!(
                        fr,
                        "{}",
                        serde_json::json!({
                            "year": year, "town": t, "name": st.town_name[t], "founded": st.town_founded[t],
                            "rise": r, "wbar_was": wbar_was[t], "wbar": w, "island_wage": mean_w,
                            "columns": ["pop", "hire", "carters", "carried_land", "carried_works", "at_work", "wage", "top", "grain_made", "grain_bought", "food", "by_sector", "price"],
                            "sectors": sim.content.sector_name,
                            "months": now[t].iter().map(Month::json).collect::<Vec<_>>(),
                            "months_before": last[t].iter().map(Month::json).collect::<Vec<_>>(),
                            "turn": turn[t],
                        })
                    )
                    .unwrap();
                }
            }
        }
        let row = serde_json::json!({
            "year": year, "pop": pop, "towns": act.len(), "founded": founded, "hunger": hungry.len(), "hungry": hungry,
            "linked": s("real_wage_linked"), "wage": s("wage"),
            "wage_in_grain": s("wage") / observer::national_prices(st)[grain].max(1e-12),
            "rise_max": worst.0, "rise_town": worst.1, "rise_name": if worst.1 >= 0 { st.town_name[worst.1 as usize].clone() } else { String::new() },
            "poor": poor,
        });
        writeln!(fy, "{row}").unwrap();
        // the turn into next year, for next year's lines
        // and who gave each town its new cells: the giver, its cells, and the giver before the turn
        let mut gave = vec![std::collections::BTreeMap::<usize, usize>::new(); T];
        for c in 0..st.N.min(st.prev_catch.len()).min(st.catch.len()) {
            let (i, j) = (st.prev_catch[c], st.catch[c]);
            if i >= 0 && j >= 0 && i != j {
                *gave[j as usize].entry(i as usize).or_insert(0) += 1;
            }
        }
        turn = (0..T)
            .map(|t| {
                let givers: Vec<serde_json::Value> =
                    gave[t].iter().map(|(&i, &n)| serde_json::json!({"town": i, "name": st.town_name[i], "cells": n, "before": before[i]})).collect();
                serde_json::json!({"before": before[t], "after": snap(st, &sim.content, t), "givers": givers})
            })
            .collect();
        last = now;
        for t in 0..T {
            let on = st.town_active[t] && st.pop[t] > 0.0;
            wbar_was[t] = if on { st.wbar[t] } else { 0.0 };
            lived[t] = if on { lived[t] + 1 } else { 0 };
        }
    }
    if failed.is_none() {
        let secs = t0.elapsed().as_secs_f64();
        let r = sim::finish(sim);
        let found = facts::stylised_facts(&r, &g, secs);
        let mut m = serde_json::Map::new();
        for f in &found.facts {
            m.insert(f.name.clone(), serde_json::json!(f.value));
        }
        std::fs::write(format!("{out}.facts.json"), serde_json::Value::Object(m).to_string()).unwrap();
    }
}
