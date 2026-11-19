//! The machines for tasks on some islands, a line a year (docs/rules/tasks.md).
//!
//!     cargo run --release -p isostate-core --example tasks -- [--seeds 1-8] [--end-year 2050]
//!         [--families 6] [--off] [--json '{"hurdle": 0.15}'] [--from 1895] [--csv FILE]
//!
//! Each island is played under the game's rules on a thread of its own. For
//! every year from `--from` it prints, and writes to `--csv`: the hoist's
//! quality (nothing until found), the share of the works' hauling machines
//! do, the same at the pits and quarries (`tasks.land`), the towns' target for it, the wage over a unit's cost (the
//! waterline), the margin's gap, the machines' book in years of the wage
//! bill, the shares of hands on the land and in the works, the share living
//! in towns, and the island's mean wage and price of machinery and coal.
//! `--off` plays with the rule off; `--json` lays settings over the game's.
#![allow(non_snake_case)]
use isostate_core::observer::annual;
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut families, mut end_year, mut csv, mut off, mut json, mut from) = (vec![7u64], 6usize, None, None::<String>, false, None::<String>, 1895i64);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => {
                let s = next();
                seeds = match s.split_once('-') {
                    Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                    None => s.split(',').map(|x| x.parse().unwrap()).collect(),
                }
            }
            "--families" => families = next().parse().unwrap(),
            "--end-year" => end_year = Some(next().parse().unwrap()),
            "--csv" => csv = Some(next()),
            "--json" => json = Some(next()),
            "--from" => from = next().parse().unwrap(),
            "--off" => {
                off = true;
                i += 1;
                continue;
            }
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let mut P = Rules::Game.params();
    if let Some(j) = &json {
        P = P.with_json_overrides(j).expect("the settings are known");
    }
    P.n_families = families;
    P.end_year = end_year;
    if off {
        P.tasks.enabled = false;
    }
    let handles: Vec<_> = seeds
        .iter()
        .map(|&seed| {
            let mut P = P.clone();
            P.seed = seed;
            std::thread::spawn(move || {
                let r = sim::run_with(&P, Rules::Game.content(), false, true).expect("money is conserved every month");
                rows(&r, seed, from)
            })
        })
        .collect();
    let head = "seed,year,quality,auto_haul,auto_haul_pits,target_haul,waterline,margin_gap,machines_years,land_share,works_share,urban_share,wage,price_machinery,price_coal";
    let mut out = vec![head.to_string()];
    for h in handles {
        let rs = h.join().expect("a run");
        for r in &rs {
            println!("{}", r.replace(',', "  "));
        }
        out.extend(rs);
    }
    if let Some(path) = csv {
        std::fs::write(&path, out.join("\n") + "\n").expect("the csv is written");
    }
}

fn rows(r: &RunResult, seed: u64, from: i64) -> Vec<String> {
    let (st, C) = (&r.state, &r.content);
    let y0 = C.start_year;
    let s = |name: &str, y: i64| st.series.index.get(name).map_or(f64::NAN, |&k| annual(&st.series.data[k], y0, y));
    let ratio = |a: f64, b: f64| if b.abs() > 1e-12 { a / b } else { f64::NAN };
    let mut out = Vec::new();
    for t in st.tasks.years.iter().filter(|t| t.year >= from) {
        let y = t.year;
        let q = match st.tasks.found.first() {
            Some(&f) if f >= 0 && f <= y => (1.0 + C.tasks.uses[0].pace).powf((y - f) as f64).min(C.tasks.uses[0].cap),
            _ => f64::NAN,
        };
        let bill = st.acct.years.iter().find(|a| a.year == y).map_or(f64::NAN, |a| a.wage_bill);
        let pits = (0..C.NS.min(t.land_work.len())).filter(|&k| mines::is_extractive(&C.sector_name[k]));
        let (ph, pw) = pits.fold((0.0, 0.0), |(h, w), k| (h + t.land_held[k][0], w + t.land_work[k][0]));
        out.push(format!(
            "{seed},{y},{q:.3},{:.4},{:.4},{:.4},{:.3},{:.3},{:.3},{:.4},{:.4},{:.4},{:.3},{:.3},{:.3}",
            ratio(t.held[0], t.work[0]),
            ratio(ph, pw),
            t.target[0],
            t.waterline[0],
            t.margin_gap[0],
            ratio(t.book, bill),
            s("ag_share", y),
            s("ind_share", y),
            s("urban_share", y),
            s("wage", y),
            s("price_machinery", y),
            s("price_coal", y),
        ));
    }
    out
}
