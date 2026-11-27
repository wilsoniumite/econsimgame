//! Carriage in hands, year by year (docs/rules/carriage.md, "What it does"):
//! one island under the game's rules with six families, any setting by name, a
//! JSON line a year. Reads the run and changes nothing in it.
//!
//!     cargo run --profile work -p isostate-core --example carriage_probe -- SEED OUT.jsonl [--start settled|landfall] [--set key=value]... [--end YEAR]
//!
//! Each line: the island's people and towns; the years' hunger (towns the
//! `famine` news names); the largest rise of a town's yearly mean wage over the
//! year before (towns with people in the two years before, as the written
//! check's fact reads it); the small poor towns (under 2,000 people at under a
//! fifth of the island's wage, by people); a town's price index (1750's basket
//! at the year's mean prices) over its city's, the least and the median of the
//! towns; hands a 1,000 ton-cells as the crow flies (the written check's
//! measure) and a 1,000 of weight times path cost; carters' share of hands; the
//! hands an hour of carting takes once the lorry is found (`tasks.cart_hands`,
//! weighted by the hours); the island's freight and its wage; the merchants'
//! share of income (the books). The last line is the run's stylised facts.
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
            s => panic!("unknown argument {s}"),
        }
        i += 1;
    }
    for (k, v) in &sets {
        P = sweep::set_field(&P, k, &sweep::parse_value(v)).unwrap();
    }
    let mut fy = std::fs::File::create(&out).unwrap();
    let t0 = std::time::Instant::now();
    let mut sim = sim::start(&P, game_content());
    let mut g = facts::Gathered::default();
    let T = sim.st.T_max;
    let G = sim.content.G;
    let grain = sim.content.gi("grain");
    let mut wbar_was = vec![0.0f64; T];
    let mut lived = vec![0usize; T];
    while sim.year <= sim.end_year {
        let year = sim.year;
        let ev0 = sim.events.len();
        let mut cart_hours = 0.0;
        let mut cart_hands = 0.0;
        let mut failed = None;
        for month in 0..12 {
            if let Err(e) = sim::run_month(&mut sim, month) {
                failed = Some(format!("{year}-{month}: {e}"));
                break;
            }
            facts::gather_month(&mut g, &sim, month);
            let st = &sim.st;
            for t in 0..T.min(st.tasks.cart_work.len()).min(st.tasks.cart_hands.len()) {
                if st.town_active[t] {
                    cart_hours += st.tasks.cart_work[t];
                    cart_hands += st.tasks.cart_work[t] * st.tasks.cart_hands[t];
                }
            }
        }
        if let Some(f) = failed {
            writeln!(fy, "{}", serde_json::json!({"year": year, "failed": f})).unwrap();
            return;
        }
        sim::end_year(&mut sim);
        facts::gather_year(&mut g, &sim);
        let (st, Pr) = (&sim.st, &sim.params);
        let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t] && st.pop[t] > 0.0).collect();
        let pop: f64 = act.iter().map(|&t| st.pop[t]).sum();
        let hungry: Vec<usize> = sim.events[ev0..].iter().filter(|e| e.kind == "famine").filter_map(|e| e.town).collect();
        let mean_w = act.iter().map(|&t| st.pop[t] * st.wbar[t]).sum::<f64>() / pop.max(1e-9);
        let poor: Vec<(f64, f64)> =
            act.iter().filter(|&&t| st.pop[t] < 2000.0 && st.wbar[t] < 0.2 * mean_w).map(|&t| (st.pop[t], st.wbar[t] / mean_w)).collect();
        let mut rise = (0.0f64, -1i64);
        for &t in &act {
            if lived[t] >= 2 && wbar_was[t] > 0.0 && st.wbar[t] > 0.0 {
                let r = st.wbar[t] / wbar_was[t];
                if r > rise.0 {
                    rise = (r, t as i64);
                }
            }
        }
        // a town's price index over its city's (the largest town by people)
        let basket = &st.cpi_basket;
        let (mut least, mut median) = (f64::NAN, f64::NAN);
        if basket.len() == G && act.len() > 1 {
            let city = *act.iter().max_by(|&&x, &&y| st.pop[x].total_cmp(&st.pop[y])).unwrap();
            let cost = |t: usize| (0..G).map(|k| basket[k] * st.pbar[(t, k)]).sum::<f64>();
            let at = cost(city);
            let mut r: Vec<f64> = act.iter().filter(|&&t| t != city).map(|&t| cost(t) / at).filter(|v| v.is_finite()).collect();
            r.sort_by(f64::total_cmp);
            if !r.is_empty() {
                least = r[0];
                median = r[r.len() / 2];
            }
        }
        let carters = st.cart_years.last().map_or(f64::NAN, |&(_, c)| c / 12.0);
        let wc = g.months.weight_cells.last().copied().unwrap_or(f64::NAN);
        let wp = g.months.weight_path.last().copied().unwrap_or(f64::NAN);
        let hands: f64 = (0..T).filter(|&t| st.town_active[t]).map(|t| st.month_stats.supply.get(t).copied().unwrap_or(0.0) + st.carters.get(t).copied().unwrap_or(0.0)).sum();
        let iy = accounts::island_year(st);
        let (income, merchants) = iy.as_ref().map_or((f64::NAN, f64::NAN), |y| (y.income, y.merchants_div + y.merchants_kept));
        let wage = observer::annual(st.series.get("wage"), sim.content.start_year, year);
        let row = serde_json::json!({
            "year": year, "pop": pop, "towns": act.len(),
            "hungry_towns": hungry.len(),
            "rise_max": rise.0, "rise_pop": if rise.1 >= 0 { st.pop[rise.1 as usize] } else { f64::NAN },
            "poor": poor.len(), "poor_list": poor,
            "price_vs_city_least": least, "price_vs_city_median": median,
            "hands_wc": 1000.0 * carters / wc, "hands_wp": 1000.0 * carters / wp,
            "carters_share": carters / hands.max(1e-9),
            "cart_hands": if cart_hours > 0.0 { cart_hands / cart_hours } else { 1.0 },
            "freight": longrun::freight(st, Pr), "island_wage": carriage::island_wage(st), "mean_wbar": mean_w,
            "wage_in_grain": wage / observer::national_prices(st)[grain].max(1e-12),
            "merchants_share": merchants / income,
        });
        writeln!(fy, "{row}").unwrap();
        for t in 0..T {
            let on = st.town_active[t] && st.pop[t] > 0.0;
            wbar_was[t] = if on { st.wbar[t] } else { 0.0 };
            lived[t] = if on { lived[t] + 1 } else { 0 };
        }
    }
    let secs = t0.elapsed().as_secs_f64();
    let r = sim::finish(sim);
    let found = facts::stylised_facts(&r, &g, secs);
    let mut m = serde_json::Map::new();
    for f in &found.facts {
        m.insert(f.name.clone(), serde_json::json!(f.value));
    }
    writeln!(fy, "{}", serde_json::json!({"facts": serde_json::Value::Object(m)})).unwrap();
}
