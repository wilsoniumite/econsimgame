//! Prints, for docs/probes/tr-loose-b.md (c), where the merchants' income comes
//! from, year by year, as a CSV on standard output: the island's income, the
//! merchants' (their dividend and what they kept), and three parts of it read
//! from the books: the spread they take on everything the land and the works
//! sell them (`merchant_spread` of its worth), the margin on what they carry
//! between towns (what it fetches where it arrives, at that town's mean price
//! of the year, less its worth where it left and the carting), and what their
//! stock gained in worth over the year (which they paid for and have not sold).
//! What is left over is the rest: sales out of stock at other prices than the
//! year's mean, goods sold that were not in store, and rounding.
//!
//!     cargo run --release -p isostate-core --example merchants_take -- [--seeds 1-8] [--rules port|game]
//!         [--families 0] [--set key=value ...] > OUT.csv
//!
//! `--set` takes a parameter by its dotted name, as the sweep does, one value
//! each (`--set carting.enabled=true --set rail_know=99`). The books are kept
//! whatever the rules say; they change nothing in the run.
#![allow(non_snake_case)]
use isostate_core::accounts::{KG, KL, KM};
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut rules, mut families, mut sets): (Vec<u64>, String, usize, Vec<String>) = ((1..=8).collect(), "game".into(), 6, Vec::new());
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => seeds = sweep::parse_seeds(&next()).unwrap(),
            "--rules" => rules = next(),
            "--families" => families = next().parse().unwrap(),
            "--set" => sets.push(next()),
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let r = if rules == "port" { Rules::Port } else { Rules::Game };
    let mut P = r.params();
    P.n_families = families;
    for s in &sets {
        let (k, v) = s.split_once('=').expect("key=value");
        let value = serde_json::from_str::<serde_json::Value>(v).unwrap_or_else(|_| serde_json::Value::String(v.to_string()));
        P = sweep::set_field(&P, k, &value).expect("a parameter and a value");
    }
    P.accounts.enabled = true;
    let rows: Vec<Vec<String>> = std::thread::scope(|sc| {
        let hs: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let mut P = P.clone();
                P.seed = seed;
                sc.spawn(move || {
                    let mut s = sim::start(&P, r.content());
                    let C = s.content.clone();
                    let spread = s.params.merchant_spread;
                    let mut out = Vec::new();
                    while s.year <= s.end_year {
                        sim::run_year(&mut s).expect("money is conserved every month");
                        let year = s.year - 1;
                        let st = &s.st;
                        let iy = accounts::island_year(st).expect("the books are kept");
                        let rec = st.acct.year(st.acct.years.len() - 1).expect("the books are kept");
                        let T = st.T_max;
                        let mut sold_to_merchants = 0.0;
                        let mut stock_gain = 0.0;
                        for t in 0..T {
                            for sct in 0..C.NS {
                                sold_to_merchants += rec.land[(t * C.NS + sct) * KL + 1] as f64;
                            }
                            for m in 0..C.M {
                                sold_to_merchants += rec.methods[(t * C.M + m) * KM + 1] as f64;
                            }
                            for g in 0..C.G {
                                let row = &rec.goods[(t * C.G + g) * KG..(t * C.G + g + 1) * KG];
                                stock_gain += (row[16] as f64 - row[15] as f64) * row[18] as f64;
                            }
                        }
                        let price = |t: usize, g: usize| rec.goods[(t * C.G + g) * KG + 18] as f64;
                        let margin: f64 = rec.trade.iter().map(|f| f.qty as f64 * price(f.to as usize, f.good as usize) - f.value as f64 - f.carting as f64).sum();
                        let merchants = iy.merchants_div + iy.merchants_kept;
                        out.push(vec![
                            seed.to_string(),
                            year.to_string(),
                            format!("{:.6e}", iy.income),
                            format!("{:.6e}", merchants),
                            format!("{:.6e}", spread * sold_to_merchants),
                            format!("{:.6e}", margin),
                            format!("{:.6e}", stock_gain),
                        ]);
                    }
                    out
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().expect("a run panicked")).collect()
    });
    println!("seed,year,income,merchants,spread,margin,stock_gain");
    for r in rows {
        println!("{}", r.join(","));
    }
}
