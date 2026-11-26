//! The colony's pits month by month (docs/rules/mines.md, "G. At the size of
//! the market"): a line of CSV a month, the whole island, under the game's rules
//! with six families.
//!
//!     cargo run --profile work -p isostate-core --example pits -- SEED OUT.csv
//!         [--start settled|landfall] [--set key=value]... [--end YEAR]
//!
//! Each line has what the merchants owe over the island's money (their purses
//! below nothing, summed), a price index (the island's sales of the first month,
//! priced each month at the towns' prices weighted by their people, the first
//! month = 1), the grain price and the mean wage, the grain asked for, sold and
//! on the shelves; and for each of stone, clay,
//! iron ore and coal: what the workings raised that month, what the towns'
//! buyers asked for, what they got, the stock on the shelves, and the hands at
//! the workings. It ends at December of `--end` (1760 if not given).
#![allow(non_snake_case)]
use isostate_core::*;
use std::io::Write;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let mut P = game_params();
    P.seed = a[0].parse().unwrap();
    P.n_families = 6;
    P.end_year = Some(1760);
    let out = a[1].clone();
    let mut i = 2;
    let mut sets: Vec<(String, String)> = Vec::new();
    while i < a.len() {
        match a[i].as_str() {
            "--start" => landfall::set_start(&mut P, &a[i + 1]).unwrap(),
            "--set" => {
                let (k, v) = a[i + 1].split_once('=').expect("--set key=value");
                sets.push((k.to_string(), v.to_string()));
            }
            "--end" => P.end_year = Some(a[i + 1].parse().unwrap()),
            s => panic!("unknown {s}"),
        }
        i += 2;
    }
    for (k, v) in &sets {
        P = sweep::set_field(&P, k, &sweep::parse_value(v)).unwrap();
    }
    let mut sim = sim::start(&P, game_content());
    sim.st.watch_markets = true;
    let C = sim.content.clone();
    let (G, T) = (C.G, sim.st.T_max);
    let names = ["stone", "clay", "iron_ore", "coal"];
    let goods: Vec<usize> = names.iter().map(|n| C.gi(n)).collect();
    let grain = C.gi("grain");
    let mut f = std::fs::File::create(&out).unwrap();
    let mut head = String::from("year,month,pop,towns,money,owe,owe_over_money,pindex,grain_p,wage,grain_asked,grain_sold,grain_stock");
    for n in names {
        head += &format!(",{n}_made,{n}_asked,{n}_sold,{n}_stock,{n}_hands");
    }
    writeln!(f, "{head}").unwrap();
    let mut basket: Option<(Vec<f64>, f64)> = None;
    // the workings' books add up over the months to the year's review: a month's make is the change
    let mut was: Vec<(f64, f64)> = vec![(0.0, 0.0); goods.len()];
    while sim.year <= sim.end_year {
        let year = sim.year;
        for month in 0..12 {
            if let Err(e) = sim::run_month(&mut sim, month) {
                writeln!(f, "failed {year}-{month}: {e}").unwrap();
                return;
            }
            let st = &sim.st;
            let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t]).collect();
            let pop: f64 = act.iter().map(|&t| st.pop[t]).sum();
            let priced = |g: usize| act.iter().map(|&t| st.price[(t, g)] * st.pop[t]).sum::<f64>() / pop.max(1.0);
            if basket.is_none() {
                let q: Vec<f64> = (0..G).map(|g| act.iter().map(|&t| st.outflow[(t, g)]).sum()).collect();
                let v: f64 = (0..G).map(|g| q[g] * priced(g)).sum();
                basket = Some((q, v));
            }
            let (q, v0) = basket.as_ref().unwrap();
            let pindex = (0..G).map(|g| q[g] * priced(g)).sum::<f64>() / v0.max(1e-12);
            let money = econ::money_total(st);
            let owe: f64 = st.merch_cash.iter().map(|&c| (-c).max(0.0)).sum();
            let wage = act.iter().map(|&t| st.wage[t] * st.pop[t]).sum::<f64>() / pop.max(1.0);
            let mut line = format!(
                "{year},{month},{pop:.0},{},{money:.0},{owe:.0},{:.4},{pindex:.4},{:.4},{wage:.4}",
                act.len(),
                owe / money.max(1e-9),
                priced(grain)
            );
            let sum = |m: &mat::Mat, g: usize| if m.rows > 0 { act.iter().map(|&t| m[(t, g)]).sum::<f64>() } else { 0.0 };
            line += &format!(",{:.1},{:.1},{:.1}", sum(&st.month_requests, grain), sum(&st.month_sold, grain), sum(&st.inv, grain));
            let R = st.mine_sectors.len();
            for (k, &g) in goods.iter().enumerate() {
                let r = st.mine_sectors.iter().position(|&s| C.sector_good[s] == g);
                let (made_sum, hands_sum) = match r {
                    Some(r) if st.mine_acc.rows == 3 * R => (st.mine_acc.row(R + r).iter().sum::<f64>(), st.mine_acc.row(r).iter().sum::<f64>()),
                    _ => (0.0, 0.0),
                };
                let (made, hands) = if st.mine_months <= 1 { (made_sum, hands_sum) } else { (made_sum - was[k].0, hands_sum - was[k].1) };
                was[k] = (made_sum, hands_sum);
                line += &format!(",{made:.1},{:.1},{:.1},{:.1},{hands:.1}", sum(&st.month_requests, g), sum(&st.month_sold, g), sum(&st.inv, g));
            }
            writeln!(f, "{line}").unwrap();
        }
        sim::end_year(&mut sim);
    }
}
