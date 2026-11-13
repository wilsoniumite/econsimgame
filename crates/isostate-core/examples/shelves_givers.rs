//! Prints, for docs/rules/shelves.md, what becomes of a town's merchants in the
//! year after a district leaves it: the critic's check on stage 3 of the pass
//! over the rules (docs/designs/rules-pass.md, 10.3 and 10.9). Under
//! `people.shelves_go` the stock and the measure of sales go with the district
//! and the merchants' cash goes only "above nothing", so a town whose merchants
//! owe keeps the whole debt on a smaller trade. Is that a harm the town feels?
//!
//!     cargo run --release -p isostate-core --example shelves_givers -- [--seeds 1-16] [--families 6] [--set JSON] [--list]
//!
//! A giver is a town that stood before the year began and has a tenth fewer
//! people after the year's end than before it. For each, from the state either
//! side of the year's end and from the books of the year before and the year
//! after:
//!
//! - the share of its people that went;
//! - its merchants' cash a head before and after, and whether they owed;
//! - their stock and their measure of sales, each at December's prices, before
//!   and after, and the stock in months of sales (what the merchants' prices are
//!   marked by under `prices`);
//! - in the twelve months after: the worst month's food got over need, the
//!   months in which the merchants owed, and grain's mean price over the year
//!   before's;
//! - the merchants' dividends a head, the year after over the year before.
//!
//! It ends with the means over every giver, and over the givers whose merchants
//! owed. The game's rules with the books on. `--set` lays a JSON object over the
//! game's parameters, e.g. `--set '{"people":{"shelves_go":false}}'`. `--list`
//! prints every giver. Nothing here draws a random number of the run's or
//! changes its state.
#![allow(non_snake_case)]
use std::fmt::Write as _;

use isostate_core::accounts::{KT, TOWN_FIELDS};
use isostate_core::*;

/// One town that gave a district at a year's end.
#[derive(Clone, Default)]
struct Giver {
    seed: u64,
    year: i64,
    town: String,
    went: f64,
    cash_before: f64,
    cash_after: f64,
    stock_before: f64,
    stock_after: f64,
    sales_before: f64,
    sales_after: f64,
    food_worst: f64,
    months_owing: usize,
    grain_price: f64,
    div_ratio: f64,
}

/// What a town's merchants hold at a moment: cash, stock at the prices standing,
/// their measure of a month's sales at the same prices.
fn merchants(st: &State, t: usize) -> (f64, f64, f64) {
    let G = st.inv.cols;
    let stock = (0..G).map(|g| st.inv[(t, g)] * st.price[(t, g)]).sum();
    let sales = (0..G).map(|g| st.outflow[(t, g)] * st.price[(t, g)]).sum();
    (st.merch_cash[t], stock, sales)
}

fn trace(seed: u64, P: &Params) -> Vec<Giver> {
    let mut P = P.clone();
    P.seed = seed;
    let mut s = sim::start(&P, Rules::Game.content());
    let C = s.content.clone();
    let T = s.st.T_max;
    let grain = C.gi("grain");
    let col = |name: &str| TOWN_FIELDS.iter().position(|f| *f == name).unwrap();
    let (c_div, c_pop) = (col("div_merchants"), col("pop"));
    // the givers of last year's end, waiting for the year after to be read
    let mut open: Vec<(usize, Giver, f64, f64)> = Vec::new(); // (town, giver, grain's price the year before, dividends a head the year before)
    let mut out = Vec::new();
    // each town's mean grain price over the year just run, and its dividends a head
    let (mut price_year, mut div_year) = (vec![0.0; T], vec![0.0; T]);
    while s.year <= s.end_year {
        let year = s.year;
        let (mut food_worst, mut owing, mut price_sum) = (vec![9.0f64; T], vec![0usize; T], vec![0.0; T]);
        for month in 0..12 {
            sim::run_month(&mut s, month).expect("money is conserved");
            let st = &s.st;
            for t in 0..T {
                if st.town_active[t] {
                    food_worst[t] = food_worst[t].min(st.food_ratio[t]);
                    owing[t] += (st.merch_cash[t] < 0.0) as usize;
                    price_sum[t] += st.price[(t, grain)] / 12.0;
                }
            }
        }
        // the year after a giving: read and close
        for (t, mut g, price_before, div_before) in open.drain(..) {
            g.food_worst = food_worst[t];
            g.months_owing = owing[t];
            g.grain_price = price_sum[t] / price_before.max(1e-12);
            out.push((t, g, div_before));
        }
        let before: Vec<(f64, (f64, f64, f64), bool)> = (0..T).map(|t| (s.st.pop[t], merchants(&s.st, t), s.st.town_active[t] && s.st.town_founded[t] < year)).collect();
        sim::end_year(&mut s);
        let st = &s.st;
        let rec = st.acct.year(st.acct.years.len() - 1).expect("the books are kept");
        for t in 0..T {
            let pop = rec.towns[t * KT + c_pop] as f64;
            div_year[t] = if pop > 0.0 { rec.towns[t * KT + c_div] as f64 / pop } else { 0.0 };
            price_year[t] = price_sum[t];
        }
        for t in 0..T {
            let (pop0, (cash0, stock0, sales0), stood) = before[t];
            if !stood || !st.town_active[t] || pop0 <= 0.0 || st.pop[t] > 0.9 * pop0 {
                continue;
            }
            let (cash1, stock1, sales1) = merchants(st, t);
            let g = Giver {
                seed,
                year,
                town: st.town_name[t].clone(),
                went: 1.0 - st.pop[t] / pop0,
                cash_before: cash0 / pop0,
                cash_after: cash1 / st.pop[t].max(1.0),
                stock_before: stock0,
                stock_after: stock1,
                sales_before: sales0,
                sales_after: sales1,
                ..Giver::default()
            };
            open.push((t, g, price_year[t], div_year[t]));
        }
    }
    // the dividends a head of the year after: the books of the year after the giving
    let st = &s.st;
    let years = st.acct.years.len();
    let y0 = s.content.start_year;
    out.into_iter()
        .map(|(t, mut g, div_before)| {
            let k = (g.year + 1 - y0) as usize;
            if k < years {
                let rec = st.acct.year(k).unwrap();
                let pop = rec.towns[t * KT + c_pop] as f64;
                let div_after = if pop > 0.0 { rec.towns[t * KT + c_div] as f64 / pop } else { 0.0 };
                g.div_ratio = if div_before > 0.0 { div_after / div_before } else { f64::NAN };
            } else {
                g.div_ratio = f64::NAN;
            }
            g
        })
        .collect()
}

fn mean(v: &[f64]) -> f64 {
    let v: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
}

fn median(v: &[f64]) -> f64 {
    let mut v: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn summary(label: &str, gs: &[&Giver]) -> String {
    let mut s = String::new();
    let f = |k: &dyn Fn(&Giver) -> f64| gs.iter().map(|g| k(g)).collect::<Vec<f64>>();
    writeln!(s, "  {label}: {} givers", gs.len()).unwrap();
    if gs.is_empty() {
        return s;
    }
    writeln!(s, "    share of the people that went, mean                      {:.3}", mean(&f(&|g| g.went))).unwrap();
    writeln!(s, "    merchants owing before the year's end                    {}", gs.iter().filter(|g| g.cash_before < 0.0).count()).unwrap();
    writeln!(s, "    merchants' cash a head, before and after (medians)       {:.2} and {:.2}", median(&f(&|g| g.cash_before)), median(&f(&|g| g.cash_after))).unwrap();
    writeln!(s, "    stock kept, of what they held (mean)                     {:.3}", mean(&f(&|g| g.stock_after / g.stock_before.max(1e-12)))).unwrap();
    writeln!(s, "    measure of sales kept (mean)                             {:.3}", mean(&f(&|g| g.sales_after / g.sales_before.max(1e-12)))).unwrap();
    writeln!(s, "    stock in months of sales, before and after (medians)     {:.2} and {:.2}", median(&f(&|g| g.stock_before / g.sales_before.max(1e-12))), median(&f(&|g| g.stock_after / g.sales_after.max(1e-12)))).unwrap();
    writeln!(s, "    the year after: worst month's food got (mean; under 0.82) {:.2}; {}", mean(&f(&|g| g.food_worst)), gs.iter().filter(|g| g.food_worst < 0.82).count()).unwrap();
    writeln!(s, "    the year after: months the merchants owed (mean)         {:.2}", mean(&f(&|g| g.months_owing as f64))).unwrap();
    writeln!(s, "    the year after: grain's price over the year before's     {:.3}", mean(&f(&|g| g.grain_price))).unwrap();
    writeln!(s, "    merchants' dividends a head, the year after over before  {:.3} (median {:.3})", mean(&f(&|g| g.div_ratio)), median(&f(&|g| g.div_ratio))).unwrap();
    s
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut families, mut set, mut list): (Vec<u64>, usize, Option<String>, bool) = ((1..=16).collect(), 6, None, false);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => seeds = sweep::parse_seeds(&next()).unwrap(),
            "--families" => families = next().parse().unwrap(),
            "--set" => set = Some(next()),
            "--list" => {
                list = true;
                i += 1;
                continue;
            }
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let mut P = Rules::Game.params();
    P.n_families = families;
    P.accounts.enabled = true;
    if let Some(j) = &set {
        P = P.with_json_overrides(j).expect("a JSON object of parameters");
    }
    let all: Vec<Giver> = std::thread::scope(|sc| {
        let hs: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let P = P.clone();
                sc.spawn(move || trace(seed, &P))
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().expect("a run panicked")).collect()
    });
    println!(
        "the game's rules, {families} families, seeds {}-{}, shelves_go {}{}",
        seeds[0],
        seeds[seeds.len() - 1],
        P.people.shelves_go,
        set.as_ref().map_or(String::new(), |s| format!(", with {s}"))
    );
    if list {
        println!();
        println!("  seed year town            went  cash a head before/after  stock kept  sales kept  food worst  months owing  grain price  dividends");
        for g in &all {
            println!(
                "  {:4} {:4} {:14} {:6.3}  {:9.2} {:9.2}      {:9.3}  {:9.3}  {:9.2}  {:11}  {:11.3}  {:9.3}",
                g.seed,
                g.year,
                g.town,
                g.went,
                g.cash_before,
                g.cash_after,
                g.stock_after / g.stock_before.max(1e-12),
                g.sales_after / g.sales_before.max(1e-12),
                g.food_worst,
                g.months_owing,
                g.grain_price,
                g.div_ratio
            );
        }
    }
    println!();
    let every: Vec<&Giver> = all.iter().collect();
    print!("{}", summary("every giver", &every));
    let owing: Vec<&Giver> = all.iter().filter(|g| g.cash_before < 0.0).collect();
    print!("{}", summary("givers whose merchants owed", &owing));
    let late: Vec<&Giver> = all.iter().filter(|g| g.year >= 1850).collect();
    print!("{}", summary("givers from 1850", &late));
}
