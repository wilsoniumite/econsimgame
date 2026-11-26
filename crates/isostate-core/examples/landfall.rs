//! One run under the game's rules with six families, a line of JSON a year and a
//! map every 25 years: the probe of the landfall start (docs/rules/landfall.md).
//! Begun from the trial of 26 November 2026's `one_town.rs`.
//!
//!     cargo run --profile work -p isostate-core --example landfall -- SEED OUT.jsonl
//!         [--start settled|landfall] [--set key=value]... [--end YEAR]
//!
//! Each `--set` is laid over the start as the sweep lays it. A year's line has
//! the island's people, towns, the share of land with five hands or more a
//! cell, how far the hands reach from the first town, the wage in grain, the
//! settlers and each town's free land, the ages' three broad groups, the
//! largest rise of a town's wage in the year, and the money check; what the
//! merchants owe in December over the island's money, carters over all hands,
//! money a head, the price level and the linked wage; every 25th year adds the
//! maps.
#![allow(non_snake_case)]
use isostate_core::*;
use std::io::Write;

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
    let mut f = std::fs::File::create(&out).unwrap();
    let mut sim = sim::start(&P, game_content());
    let (W, N, T) = (sim.st.world.W, sim.st.N, sim.st.T_max);
    let c0 = sim.st.town_node[0];
    let dist = |c: usize| ((c / W) as f64 - (c0 / W) as f64).hypot((c % W) as f64 - (c0 % W) as f64);
    let land: Vec<u8> = sim.st.world.land.iter().map(|&l| l as u8).collect();
    let n_land = land.iter().filter(|&&l| l == 1).count();
    writeln!(
        f,
        "{}",
        serde_json::json!({"head": true, "W": W, "H": sim.st.world.H, "land": land, "first": c0, "n_land": n_land,
        "river": sim.st.world.river, "seed": P.seed, "sets": sets, "landfall": P.start.landfall})
    )
    .unwrap();
    let mut failed: Option<String> = None;
    let mut wage_was: Vec<f64> = vec![0.0; T];
    let mut first_rail: Option<i64> = None;
    while sim.year <= sim.end_year {
        let year = sim.year;
        let ev0 = sim.events.len();
        for month in 0..12 {
            if let Err(e) = sim::run_month(&mut sim, month) {
                failed = Some(format!("{year}: {e}"));
                break;
            }
        }
        if failed.is_some() {
            writeln!(f, "{}", serde_json::json!({"failed": failed})).unwrap();
            break;
        }
        // the whole of the year's end, as a game plays it (the trial's probe left out the books,
        // the machines' ramp, the ages and the rest of `end_year`)
        sim::end_year(&mut sim);
        let st = &sim.st;
        let hunger = sim.events[ev0..].iter().filter(|e| e.kind == "famine").count();
        let sailed = sim.events[ev0..].iter().filter(|e| e.kind == "emigration").count();
        let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t]).collect();
        let pop: f64 = act.iter().map(|&t| st.pop[t]).sum();
        let far_t = act.iter().map(|&t| dist(st.town_node[t])).fold(0.0, f64::max);
        let (mut h5, mut far_h5, mut hands) = (0usize, 0.0f64, Vec::new());
        for c in 0..N {
            let l = st.cell_labor.get(c).copied().unwrap_or(0.0);
            if st.world.land[c] && l >= 5.0 {
                h5 += 1;
                far_h5 = far_h5.max(dist(c));
            }
            if l > 0.0 {
                hands.push((dist(c), l));
            }
        }
        hands.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let tot: f64 = hands.iter().map(|h| h.1).sum();
        let (mut acc, mut r90) = (0.0, 0.0);
        for h in &hands {
            acc += h.1;
            if acc >= 0.9 * tot {
                r90 = h.0;
                break;
            }
        }
        // the year's mean wage of each town (st.wbar was set at the year's turn): its rise
        // over the last year's, for towns that stood both years and are past their first two
        let mut worst = (0.0f64, -1i64);
        for &t in &act {
            let w = st.wbar.get(t).copied().unwrap_or(0.0);
            if wage_was[t] > 0.0 && w > 0.0 && st.town_founded[t] < year - 2 {
                let r = w / wage_was[t];
                if r > worst.0 {
                    worst = (r, t as i64);
                }
            }
        }
        for t in 0..T {
            wage_was[t] = if st.town_active[t] { st.wbar.get(t).copied().unwrap_or(0.0) } else { 0.0 };
        }
        let s = |n: &str| observer::annual(st.series.get(n), sim.content.start_year, year);
        let rail = s("rail_km");
        if first_rail.is_none() && rail > 0.0 {
            first_rail = Some(year);
        }
        let works = (0..st.F_count).filter(|&k| st.f_alive[k]).count();
        let ages = st.ages.years.last().map(|y| (y.under15, y.working, y.over64, y.births_k, y.deaths_k, y.life)).unwrap_or_default();
        let settlers = st.immigration.years.last().filter(|y| y.0 == year).map(|y| y.1).unwrap_or(0.0);
        let free0 = st.frontier.lifted.first().copied().unwrap_or(-1.0);
        let free_mean = {
            let (mut a, mut b) = (0.0, 0.0);
            for &t in &act {
                if let Some(&x) = st.frontier.free.get(t) {
                    a += x * st.pop[t];
                    b += st.pop[t];
                }
            }
            if b > 0.0 { a / b } else { -1.0 }
        };
        let y_mean = {
            let (mut a, mut b) = (0.0, 0.0);
            for &t in &act {
                a += st.y_ratio.get(t).copied().unwrap_or(0.0) * st.pop[t];
                b += st.pop[t];
            }
            if b > 0.0 { a / b } else { 0.0 }
        };
        let g_mean = {
            let (mut a, mut b) = (0.0, 0.0);
            for &t in &act {
                a += st.pp_growth.get(t).copied().unwrap_or(0.0) * st.pop[t];
                b += st.pop[t];
            }
            if b > 0.0 { a / b } else { 0.0 }
        };
        // carriage and money (docs/rules/landfall.md, "Carriage that follows prices"): what the
        // merchants owe in December over the island's money; carters over all hands (the works',
        // the land's and the carters'); money a head, in coins and in grain; the price level
        let owe: f64 = st.merch_cash.iter().map(|&c| (-c).max(0.0)).sum();
        let (works_h, land_h, cart_h) = (
            (0..st.F_count).filter(|&k| st.f_alive[k]).map(|k| st.f_labor[k]).sum::<f64>(),
            st.ex_labor.iter().sum::<f64>(),
            st.carters.iter().sum::<f64>(),
        );
        let grain_p = observer::national_prices(st)[sim.content.gi("grain")].max(1e-12);
        let mut row = serde_json::json!({
            "year": year, "pop": pop, "towns": act.len(),
            "far_town_cells": far_t, "real_wage": s("real_wage"), "wage": s("wage"), "cpi": s("cpi"),
            "gdp_pc": s("gdp_pc"), "urban_share": s("urban_share"),
            "works": works, "rail_km": rail, "road_km": s("road_km"), "hunger_items": hunger, "sailing_items": sailed,
            "wage_in_grain": s("wage") / observer::national_prices(st)[sim.content.gi("grain")].max(1e-12),
            "hands5": h5 as f64 / n_land as f64, "far_hands5_cells": far_h5, "r90_cells": r90, "hands_on_cells": tot,
            "settlers": settlers, "free_first": free0, "free_mean": free_mean, "y_mean": y_mean, "growth_mean": g_mean,
            "under15": ages.0, "working": ages.1, "over64": ages.2, "births_k": ages.3, "deaths_k": ages.4, "life": ages.5,
            "wage_rise_max": worst.0, "wage_rise_town": worst.1, "first_rail": first_rail,
            "emigrants": st.pp_emigrants, "settlers_total": st.immigration.total,
        });
        for (k, v) in [
            ("owe_over_money", owe / st.money_now.max(1e-9)),
            ("money", st.money_now),
            ("money_pc", st.money_now / pop.max(1.0)),
            ("money_pc_grain", st.money_now / pop.max(1.0) / grain_p),
            ("carters", cart_h),
            ("carters_share", cart_h / (works_h + land_h + cart_h).max(1e-9)),
            ("price_level", st.longrun.chain),
            ("real_wage_linked", s("real_wage_linked")),
            ("grain_price", grain_p),
            // pits (docs/rules/mines.md, "G"): coal, iron ore and iron made in the year, a head
            ("coal_pc", 12.0 * s("out_coal") / pop.max(1.0)),
            ("iron_ore_pc", 12.0 * s("out_iron_ore") / pop.max(1.0)),
            ("iron_pc", 12.0 * s("out_iron") / pop.max(1.0)),
        ] {
            row[k] = serde_json::json!(v);
        }
        if year % 25 == 0 {
            let towns: Vec<_> = act
                .iter()
                .map(|&t| serde_json::json!({"k": t, "node": st.town_node[t], "pop": st.pop[t], "founded": st.town_founded[t], "name": st.town_name[t]}))
                .collect();
            row["towns_list"] = serde_json::json!(towns);
            row["use"] = serde_json::json!(st.land_use);
            row["road"] = serde_json::json!(st.road);
            row["rail"] = serde_json::json!(st.rail);
            row["catch"] = serde_json::json!(st.catch);
            row["labor"] = serde_json::json!(st.cell_labor.iter().map(|&l| (l * 10.0).round() / 10.0).collect::<Vec<f64>>());
            row["groups"] = serde_json::json!(st.ages.years.last().map(|y| y.groups.clone()).unwrap_or_default());
        }
        // every founding, as it happens
        row["founded"] = serde_json::json!(act.iter().filter(|&&t| st.town_founded[t] == year).map(|&t| serde_json::json!({"k": t, "node": st.town_node[t], "km": 2.0 * dist(st.town_node[t]), "name": st.town_name[t]})).collect::<Vec<_>>());
        writeln!(f, "{row}").unwrap();
        if year % 25 == 0 {
            eprintln!("{year} pop {pop:.0} towns {} settlers {settlers:.0} free {free_mean:.2} y {y_mean:.2} g {g_mean:.4}", act.len());
        }
    }
}
