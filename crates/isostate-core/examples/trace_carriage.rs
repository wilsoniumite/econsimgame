//! Prints what docs/probes/tr-carriage.md reads from the books: for each setting,
//! the island's income by kind in 1750 money, the carting bill and its handling
//! charge by decade, coal and timber held in long stocks and what spoils, and the
//! hungry years split into hunger and sailings; with two settings or more, each
//! beside the first, seed for seed.
//!
//!     cargo run --release -p isostate-core --example trace_carriage -- [--seeds 1-8] [--families 6]
//!         [--rules game] [--set key=v1,v2 ...] [--threads 8] [--out DIR]
//!
//! `--set` takes any parameter by its dotted name, as the sweep does, and every
//! combination of the values given is a setting. `--out` writes `years.csv` (a row
//! for every run and year) and `hunger.csv` (a row for every town and year in
//! which a town that counts went hungry, with the grain it had), and
//! `hunger_months.csv` (those towns month by month, from the January of the year
//! before: months -11 to 0 are that year, 1 to 12 the hungry one), `town_years.csv`
//! (every town that counts, every year: the share of its people moved with
//! redrawn districts at the year's end before, its worst month's food) and
//! `towns.csv` (the year each town was founded) for reading further. The books are switched on whatever the rules say; they change
//! nothing in the run (docs/rules/accounts.md).
#![allow(non_snake_case)]
use isostate_core::accounts::{GOODS_FIELDS, KG, KL, KT, LAND_FIELDS, TOWN_FIELDS};
use isostate_core::*;
use std::fmt::Write as _;

/// The columns of a year's row, in order.
const COLS: &[&str] = &[
    "year", "pop", "cpi", "gdp_pc", "real_wage", "wage", "rail_km", "canal_km",
    // the island's income account, the year's totals in money of the day
    "income", "pay_works", "pay_land", "pay_projects", "carting", "managers", "works_profit", "merchants", "rent_farm", "rent_pits", "rent_sites",
    "rent_town", "issue", "fund_div", "spend_households", "food_households", "warmth_households",
    // hands, means over the year, all towns together
    "carters", "supply", "hands_land", "hands_works", "navvies",
    // the carting bill by kind, and trade
    "cart_works", "cart_land", "cart_trade", "trade_value", "trade_carting", "handling",
    // what households bought, at the year's prices and at the content's base prices
    "hh_value", "hh_base",
    // grain, coal and timber: made, bought (all eight buyers), sent, spoiled, stock
    // at the year's end, stock beyond four months of the town's sales and shipments,
    // stock in towns holding more than twelve months of them, the hands of the land
    // sector that raises it and what that sector raised, the good's mean price
    "grain_made", "grain_bought", "grain_sent", "grain_spoiled", "grain_stock", "grain_over4", "grain_long12", "grain_hands", "grain_out", "grain_price",
    "coal_made", "coal_bought", "coal_sent", "coal_spoiled", "coal_stock", "coal_over4", "coal_long12", "coal_hands", "coal_out", "coal_price",
    "timber_made", "timber_bought", "timber_sent", "timber_spoiled", "timber_stock", "timber_over4", "timber_long12", "timber_hands", "timber_out", "timber_price",
    // timber from woodland cleared at the year's end (a town's new ground, a works' or
    // a working's), all of it and in the towns whose stock ended beyond four months
    "timber_cleared", "timber_cleared_over4",
    // grain carried between towns: units, their worth where they left, the carting paid
    "grain_trade_qty", "grain_trade_value", "grain_trade_cart",
    // hunger: towns that count (founded more than two years before) whose worst
    // month's food was under 0.82 of need; town-months under it; the year's worst
    // town; the news items of kind "famine", told apart
    "hungry_towns", "hungry_months", "worst_food", "news_hunger", "news_sailing", "emigrants",
];

fn col(name: &str) -> usize {
    COLS.iter().position(|&c| c == name).unwrap_or_else(|| panic!("no column {name}"))
}

/// What is kept of every town's month, for following a hungry year.
const MONTH_COLS: [&str; 9] = ["food", "wage", "grain_price", "grain_stock", "pop", "hh_cash", "hh_income_prev", "carters", "supply"];
const MW: usize = MONTH_COLS.len();

/// One town's hunger in one year, with the grain it had.
struct Hunger {
    months_seen: Vec<[f64; MW]>,
    year: i64,
    town: usize,
    name: String,
    founded: i64,
    pop: f64,
    worst: f64,
    months: usize,
    grain_made: f64,
    grain_received: f64,
    grain_sent: f64,
    grain_bought_hh: f64,
    grain_stock_open: f64,
    grain_price: f64,
    island_grain_price: f64,
}

struct Run {
    setting: usize,
    seed: u64,
    rows: Vec<Vec<f64>>,
    hunger: Vec<Hunger>,
    /// Every town that counts, every year: (year, town, the share of its people that came
    /// or went with redrawn districts at the year's end before, its worst month's food,
    /// its months under 0.82).
    town_years: Vec<(i64, usize, f64, f64, usize)>,
    /// Every town standing at the end, with the year it was founded.
    founded: Vec<(usize, i64)>,
}

fn one_run(P: &Params, rules: Rules, setting: usize) -> Run {
    let mut sim = sim::start(P, rules.content());
    // the hunger rule's measure, month by month, before people::growth clears it
    let mut food: Vec<(Vec<f64>, Vec<usize>)> = Vec::new();
    let mut ends: Vec<(f64, f64)> = Vec::new();
    // every town's month, `MONTH_COLS`, kept so that a hungry year can be followed month by month
    let mut watch: Vec<Vec<[f64; MW]>> = Vec::new();
    let mut moved: Vec<Vec<f64>> = Vec::new();
    while sim.year <= sim.end_year {
        let T = sim.st.T_max;
        let (mut worst, mut months) = (vec![9.0f64; T], vec![0usize; T]);
        let mut seen = vec![[0.0f64; MW]; T * 12];
        let grain = sim.content.gi("grain");
        for m in 0..12 {
            sim::run_month(&mut sim, m).expect("money is conserved every month");
            for t in 0..T {
                if sim.st.town_active[t] {
                    let s = &sim.st;
                    seen[t * 12 + m] = [
                        s.food_ratio[t],
                        s.wage[t],
                        s.price[(t, grain)],
                        s.inv[(t, grain)],
                        s.pop[t],
                        s.hh_cash[t],
                        s.hh_income_prev[t],
                        s.carters.get(t).copied().unwrap_or(0.0),
                        s.month_stats.supply.get(t).copied().unwrap_or(0.0),
                    ];
                    let fr = sim.st.food_ratio[t];
                    worst[t] = worst[t].min(fr);
                    if fr < 0.82 {
                        months[t] += 1;
                    }
                }
            }
        }
        food.push((worst, months));
        watch.push(seen);
        sim::end_year(&mut sim);
        // the share of each town's people that came or went with redrawn districts at the year's end
        let T2 = sim.st.T_max;
        moved.push((0..T2).map(|t| sim.st.pp_moved.get(t).copied().unwrap_or(0.0)).collect());
        // the canal open as the year ends, and the emigrants so far
        ends.push((2.0 * sim.st.canal.iter().filter(|&&c| c > 0).count() as f64, sim.st.pp_emigrants));
    }
    let r = sim::finish(sim);
    let (st, C, Pr) = (&r.state, &r.content, &r.params);
    let ann = facts::annual_means(&r);
    let at = |name: &str, k: usize| st.series.index.get(name).map_or(f64::NAN, |&s| ann[s][k]);
    let (T, G, NS) = (st.T_max, C.G, C.NS);
    let gc = |name: &str| GOODS_FIELDS.iter().position(|&c| c == name).unwrap();
    let lc = |name: &str| LAND_FIELDS.iter().position(|&c| c == name).unwrap();
    let tc = |name: &str| TOWN_FIELDS.iter().position(|&c| c == name).unwrap();
    let bought_cols: Vec<usize> = (gc("households")..=gc("projects")).collect();
    let food_s = C.service_name.iter().position(|s| s == "food").unwrap();
    let warmth_s = C.service_name.iter().position(|s| s == "warmth").unwrap();
    let watched = [("grain", "farm"), ("coal", "coalmine"), ("timber", "forestry")];
    let mut rows = Vec::new();
    let mut hunger = Vec::new();
    let mut town_years = Vec::new();
    for (k, y) in st.acct.years.iter().enumerate() {
        let year = y.year;
        let rec = st.acct.year(k).expect("a complete year");
        let mut row = vec![0.0f64; COLS.len()];
        let mut set = |name: &str, v: f64| row[col(name)] = v;
        set("year", year as f64);
        set("pop", at("pop", k));
        set("cpi", at("cpi", k));
        set("gdp_pc", at("gdp_pc", k));
        set("real_wage", at("real_wage", k));
        set("wage", at("wage", k));
        set("rail_km", at("rail_km", k));
        set("canal_km", ends[k].0);
        set("emigrants", ends[k].1 - if k > 0 { ends[k - 1].1 } else { 0.0 });
        set("income", y.income);
        set("pay_works", y.pay_works);
        set("pay_land", y.pay_land);
        set("pay_projects", y.pay_projects);
        set("carting", y.carting);
        set("managers", y.managers);
        set("works_profit", y.div_households + y.div_families + y.works_kept);
        set("merchants", y.merchants_div + y.merchants_kept);
        set("rent_farm", y.rent_farm);
        set("rent_pits", y.rent_pits);
        set("rent_sites", y.rent_sites);
        set("rent_town", y.rent_town);
        set("issue", y.issue);
        set("fund_div", y.fund_div);
        set("spend_households", y.spend_households);
        set("food_households", y.need_households[food_s]);
        set("warmth_households", y.need_households[warmth_s]);
        // the towns' own figures
        for name in ["carters", "supply", "hands_land", "hands_works", "navvies", "cart_works", "cart_land", "cart_trade"] {
            let c = tc(name);
            set(name, (0..T).map(|t| rec.towns[t * KT + c] as f64).sum());
        }
        let trade_value: f64 = rec.trade.iter().map(|f| f.value as f64).sum();
        set("trade_value", trade_value);
        set("trade_carting", rec.trade.iter().map(|f| f.carting as f64).sum());
        set("handling", Pr.handling * trade_value);
        let grain_g = C.gi("grain") as u16;
        let gf = || rec.trade.iter().filter(|f| f.good == grain_g);
        set("grain_trade_qty", gf().map(|f| f.qty as f64).sum());
        set("grain_trade_value", gf().map(|f| f.value as f64).sum());
        set("grain_trade_cart", gf().map(|f| f.carting as f64).sum());
        let (mut hh_value, mut hh_base) = (0.0, 0.0);
        for t in 0..T {
            for g in 0..G {
                let v = &rec.goods[(t * G + g) * KG..][..KG];
                hh_value += v[gc("households")] as f64 * v[gc("price")] as f64;
                hh_base += v[gc("households")] as f64 * C.base_price[g];
            }
        }
        set("hh_value", hh_value);
        set("hh_base", hh_base);
        for (good, sector) in watched {
            let g = C.gi(good);
            let s = C.sector_name.iter().position(|n| n == sector).unwrap();
            let (mut made, mut bought, mut sent, mut spoiled, mut stock, mut over4, mut long12, mut pv, mut pw) = (0.0f64, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
            let (mut cleared, mut cleared_over4) = (0.0f64, 0.0);
            for t in 0..T {
                let v = &rec.goods[(t * G + g) * KG..][..KG];
                let b: f64 = bought_cols.iter().map(|&c| v[c] as f64).sum();
                let out = b + v[gc("sent")] as f64;
                made += v[gc("made")] as f64;
                bought += b;
                sent += v[gc("sent")] as f64;
                spoiled += v[gc("spoiled")] as f64;
                let s_end = v[gc("stock")] as f64;
                stock += s_end;
                over4 += (s_end - 4.0 * out / 12.0).max(0.0);
                if s_end > 12.0 * out / 12.0 {
                    long12 += s_end;
                }
                let tp = rec.towns[t * KT + tc("pop")] as f64;
                pv += v[gc("price")] as f64 * tp;
                pw += tp;
                if good == "timber" {
                    let cl = v[gc("cleared")] as f64;
                    cleared += cl;
                    if s_end > 4.0 * out / 12.0 {
                        cleared_over4 += cl;
                    }
                }
            }
            if good == "timber" {
                set("timber_cleared", cleared);
                set("timber_cleared_over4", cleared_over4);
            }
            let (mut hands, mut out) = (0.0, 0.0);
            for t in 0..T {
                let l = &rec.land[(t * NS + s) * KL..][..KL];
                hands += l[lc("hands")] as f64;
                out += l[lc("out")] as f64;
            }
            for (field, v) in [("made", made), ("bought", bought), ("sent", sent), ("spoiled", spoiled), ("stock", stock), ("over4", over4), ("long12", long12), ("hands", hands), ("out", out), ("price", pv / pw.max(1e-9))] {
                set(&format!("{good}_{field}"), v);
            }
        }
        // hunger, as people::growth sees it: towns founded more than two years before
        let (worst, months) = &food[k];
        let counts: Vec<usize> = (0..T).filter(|&t| st.town_founded[t] < year - 2 && worst[t] < 9.0).collect();
        let hungry: Vec<usize> = counts.iter().copied().filter(|&t| worst[t] < 0.82).collect();
        set("hungry_towns", hungry.len() as f64);
        set("hungry_months", counts.iter().map(|&t| months[t] as f64).sum());
        set("worst_food", counts.iter().map(|&t| worst[t]).fold(9.0, f64::min));
        let news = |sailing: bool| r.events.iter().filter(|e| e.kind == "famine" && e.year.floor() as i64 == year && e.title.contains("leave the island") == sailing).count() as f64;
        set("news_hunger", news(false));
        set("news_sailing", news(true));
        rows.push(row);
        for &t in &counts {
            town_years.push((year, t, if k > 0 { moved[k - 1][t] } else { 0.0 }, worst[t], months[t]));
        }
        let g = C.gi("grain");
        let island_price = rows.last().unwrap()[col("grain_price")];
        for &t in &hungry {
            let v = &rec.goods[(t * G + g) * KG..][..KG];
            hunger.push(Hunger {
                // the year before (months -11 to 0; nothing before the first year), then the year
                months_seen: (if k > 0 { watch[k - 1][t * 12..(t + 1) * 12].to_vec() } else { vec![[0.0; MW]; 12] })
                    .into_iter()
                    .chain(watch[k][t * 12..(t + 1) * 12].iter().copied())
                    .collect(),
                year,
                town: t,
                name: st.town_name[t].clone(),
                founded: st.town_founded[t],
                pop: rec.towns[t * KT + tc("pop")] as f64,
                worst: worst[t],
                months: months[t],
                grain_made: v[gc("made")] as f64,
                grain_received: v[gc("received")] as f64,
                grain_sent: v[gc("sent")] as f64,
                grain_bought_hh: v[gc("households")] as f64,
                grain_stock_open: v[gc("stock_open")] as f64,
                grain_price: v[gc("price")] as f64,
                island_grain_price: island_price,
            });
        }
    }
    let founded = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| (t, st.town_founded[t])).collect();
    Run { setting, seed: Pr.seed, rows, hunger, town_years, founded }
}

fn mean(v: &[f64]) -> f64 {
    let v: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
}

type Measure = (&'static str, fn(&dyn Fn(&str) -> f64) -> f64);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut rules, mut families, mut threads, mut out) = (vec![1u64, 2, 3, 4, 5, 6, 7, 8], Rules::Game, 6usize, 8usize, None::<String>);
    let mut sets: Vec<(String, Vec<serde_json::Value>)> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let v = args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => {
                seeds = match v.split_once('-') {
                    Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                    None => v.split(',').map(|s| s.parse().unwrap()).collect(),
                }
            }
            "--rules" => rules = Rules::parse(&v).expect("port or game"),
            "--families" => families = v.parse().unwrap(),
            "--threads" => threads = v.parse().unwrap(),
            "--out" => out = Some(v),
            "--set" => {
                let (key, vals) = v.split_once('=').expect("--set key=v1,v2");
                let vals = vals.split(',').map(|s| serde_json::from_str(s).unwrap_or(serde_json::Value::String(s.to_string()))).collect();
                sets.push((key.to_string(), vals));
            }
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let mut base = rules.params();
    base.n_families = families;
    base.accounts.enabled = true;
    // every combination of the --set values, the first key varying slowest
    let mut settings: Vec<(String, Params)> = vec![(String::from("as the rules are"), base.clone())];
    for (key, vals) in &sets {
        let mut next = Vec::new();
        for (label, p) in &settings {
            for v in vals {
                let q = sweep::set_field(p, key, v).unwrap_or_else(|e| panic!("{e}"));
                let l = if label == "as the rules are" { format!("{key}={v}") } else { format!("{label} {key}={v}") };
                next.push((l, q));
            }
        }
        settings = next;
    }
    let jobs: Vec<(usize, u64)> = (0..settings.len()).flat_map(|s| seeds.iter().map(move |&seed| (s, seed))).collect();
    let queue = std::sync::Mutex::new(jobs.into_iter());
    let done = std::sync::Mutex::new(Vec::<Run>::new());
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let Some((k, seed)) = queue.lock().unwrap().next() else { break };
                    let mut P = settings[k].1.clone();
                    P.seed = seed;
                    let r = one_run(&P, rules, k);
                    eprintln!("done: {} seed {seed}", settings[k].0);
                    done.lock().unwrap().push(r);
                }
            });
        }
    });
    let mut runs = done.into_inner().unwrap();
    runs.sort_by_key(|r| (r.setting, r.seed));

    if let Some(dir) = &out {
        std::fs::create_dir_all(dir).expect("the --out folder");
        let mut csv = String::from("setting,seed,");
        csv += &COLS.join(",");
        csv.push('\n');
        for r in &runs {
            for row in &r.rows {
                let _ = write!(csv, "{},{}", r.setting, r.seed);
                for v in row {
                    let _ = write!(csv, ",{v}");
                }
                csv.push('\n');
            }
        }
        std::fs::write(format!("{dir}/years.csv"), csv).unwrap();
        let mut h = String::from("setting,seed,year,town,name,founded,pop,worst,months,grain_made,grain_received,grain_sent,grain_bought_hh,grain_stock_open,grain_price,island_grain_price\n");
        for r in &runs {
            for x in &r.hunger {
                let _ = writeln!(
                    h,
                    "{},{},{},{},{},{},{:.0},{:.4},{},{:.1},{:.1},{:.1},{:.1},{:.1},{:.4},{:.4}",
                    r.setting, r.seed, x.year, x.town, x.name, x.founded, x.pop, x.worst, x.months, x.grain_made, x.grain_received, x.grain_sent, x.grain_bought_hh, x.grain_stock_open, x.grain_price, x.island_grain_price
                );
            }
        }
        std::fs::write(format!("{dir}/hunger.csv"), h).unwrap();
        let mut hm = format!("setting,seed,year,town,month,{}\n", MONTH_COLS.join(","));
        for r in &runs {
            for x in &r.hunger {
                for (m, v) in x.months_seen.iter().enumerate() {
                    let _ = write!(hm, "{},{},{},{},{}", r.setting, r.seed, x.year, x.town, m as i64 - 11);
                    for c in v {
                        let _ = write!(hm, ",{c:.6}");
                    }
                    hm.push('\n');
                }
            }
        }
        std::fs::write(format!("{dir}/hunger_months.csv"), hm).unwrap();
        let mut ty = String::from("setting,seed,year,town,moved_before,worst,months\n");
        for r in &runs {
            for &(year, t, mv, w, m) in &r.town_years {
                let _ = writeln!(ty, "{},{},{year},{t},{mv:.5},{w:.4},{m}", r.setting, r.seed);
            }
        }
        std::fs::write(format!("{dir}/town_years.csv"), ty).unwrap();
        let mut tf = String::from("setting,seed,town,founded\n");
        for r in &runs {
            for &(t, y) in &r.founded {
                let _ = writeln!(tf, "{},{},{t},{y}", r.setting, r.seed);
            }
        }
        std::fs::write(format!("{dir}/towns.csv"), tf).unwrap();
        let mut names = String::new();
        for (k, (label, _)) in settings.iter().enumerate() {
            let _ = writeln!(names, "{k}: {label}");
        }
        std::fs::write(format!("{dir}/settings.txt"), names).unwrap();
    }

    println!("{} rules, {} families, seeds {:?}; the books on", rules.as_str(), families, seeds);
    for (k, (label, _)) in settings.iter().enumerate() {
        println!("  setting {k}: {label}");
    }

    // a measure of one run over a span of years: the mean of its yearly values
    let over = |r: &Run, a: i64, b: i64, f: fn(&dyn Fn(&str) -> f64) -> f64| -> f64 {
        let v: Vec<f64> = r.rows.iter().filter(|row| (a..=b).contains(&(row[0] as i64))).map(|row| f(&|n: &str| row[col(n)])).collect();
        mean(&v)
    };
    // and of a ratio of two sums over the span (shares of a bill, months of stock)
    let table = |title: &str, spans: &[(i64, i64)], measures: &[Measure]| {
        println!("\n{title}");
        for &(name, f) in measures {
            println!("  {name}");
            for &(a, b) in spans {
                let mut line = format!("    {a}-{b}");
                let per: Vec<Vec<f64>> = (0..settings.len()).map(|s| runs.iter().filter(|r| r.setting == s).map(|r| over(r, a, b, f)).collect()).collect();
                for (s, v) in per.iter().enumerate() {
                    let lo = v.iter().copied().fold(f64::INFINITY, f64::min);
                    let hi = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    let _ = write!(line, "   [{s}] {:>10.4} ({:.4} .. {:.4})", mean(v), lo, hi);
                    if s > 0 {
                        let up = v.iter().zip(&per[0]).filter(|(x, y)| x > y).count();
                        let down = v.iter().zip(&per[0]).filter(|(x, y)| x < y).count();
                        let _ = write!(line, " {up}/{down}");
                    }
                }
                println!("{line}");
            }
        }
    };
    let decades: Vec<(i64, i64)> = (1750..1900).step_by(10).map(|a| (a, a + 9)).collect();
    let twenty: Vec<(i64, i64)> = vec![(1760, 1779), (1780, 1799), (1800, 1819), (1820, 1839), (1840, 1859), (1860, 1879), (1880, 1899)];

    // (a) where the gain goes: income by kind, a head a month, in 1750 money (the observer's cpi)
    let a: &[Measure] = &[
        ("people, thousands", |v| v("pop") / 1000.0),
        ("output a head (observer's gdp_pc)", |v| v("gdp_pc")),
        ("income a head a month, 1750 money (the books)", |v| v("income") / 12.0 / v("cpi") / v("pop")),
        ("  pay but carting (works, land, railways and canals, managers)", |v| (v("pay_works") + v("pay_land") + v("pay_projects") + v("managers")) / 12.0 / v("cpi") / v("pop")),
        ("  carting", |v| v("carting") / 12.0 / v("cpi") / v("pop")),
        ("  works' profit", |v| v("works_profit") / 12.0 / v("cpi") / v("pop")),
        ("  merchants", |v| v("merchants") / 12.0 / v("cpi") / v("pop")),
        ("  rent of farms, woods, pits and mines", |v| (v("rent_farm") + v("rent_pits")) / 12.0 / v("cpi") / v("pop")),
        ("  rent of works' sites", |v| v("rent_sites") / 12.0 / v("cpi") / v("pop")),
        ("  rent of town houses", |v| v("rent_town") / 12.0 / v("cpi") / v("pop")),
        ("real wage (1750 = 1)", |v| v("real_wage")),
        ("money wage", |v| v("wage")),
        ("cost of living (cpi)", |v| v("cpi")),
        ("households' goods a head a month at base prices", |v| v("hh_base") / 12.0 / v("pop")),
        ("households' prices (their goods at the year's prices over base)", |v| v("hh_value") / v("hh_base")),
        ("carters over all hands", |v| v("carters") / (v("carters") + v("supply") + v("navvies"))),
        ("hands on the land over all hands", |v| v("hands_land") / (v("carters") + v("supply") + v("navvies"))),
        ("hands in works over all hands", |v| v("hands_works") / (v("carters") + v("supply") + v("navvies"))),
        ("hands for hire not at work (supply less land and works)", |v| (v("supply") - v("hands_land") - v("hands_works")) / (v("carters") + v("supply") + v("navvies"))),
        ("work's share of income", |v| (v("pay_works") + v("pay_land") + v("pay_projects") + v("managers") + v("carting")) / v("income")),
        ("railway, km", |v| v("rail_km")),
    ];
    table("(a) the island's income by kind, a head a month in 1750 money; [k] is the setting, then mean (least .. greatest), then up/down against setting 0", &twenty, a);

    // (b) the carting bill and its handling charge
    let b: &[Measure] = &[
        ("carting bill over income", |v| (v("cart_works") + v("cart_land") + v("cart_trade")) / v("income")),
        ("  land's produce to market, share of the bill", |v| v("cart_land") / (v("cart_works") + v("cart_land") + v("cart_trade"))),
        ("  works' carting in and out, share of the bill", |v| v("cart_works") / (v("cart_works") + v("cart_land") + v("cart_trade"))),
        ("  trade between towns, share of the bill", |v| v("cart_trade") / (v("cart_works") + v("cart_land") + v("cart_trade"))),
        ("  handling, share of the bill", |v| v("handling") / (v("cart_works") + v("cart_land") + v("cart_trade"))),
        ("  handling, share of trade's carting", |v| v("handling") / v("trade_carting")),
        ("trade's carting (books' flows) over the towns' cart_trade (should be 1)", |v| v("trade_carting") / v("cart_trade")),
        ("carters over all hands", |v| v("carters") / (v("carters") + v("supply") + v("navvies"))),
        ("  of which carrying the handling (carters times handling over the bill)", |v| v("carters") * v("handling") / (v("cart_works") + v("cart_land") + v("cart_trade")) / (v("carters") + v("supply") + v("navvies"))),
        ("trade between towns over income", |v| v("trade_value") / v("income")),
    ];
    table("(b) the carting bill and the handling charge, by decade", &decades, b);

    // (c) coal and timber held long, and what spoils
    let c: &[Measure] = &[
        ("coal: island stock in months of sales and shipments", |v| v("coal_stock") / ((v("coal_bought") + v("coal_sent")) / 12.0)),
        ("coal: share of the stock held in towns with over twelve months", |v| v("coal_long12") / v("coal_stock")),
        ("coal: stock beyond four months, in months of the island's sales", |v| v("coal_over4") / ((v("coal_bought") + v("coal_sent")) / 12.0)),
        ("coal: spoiled over made", |v| v("coal_spoiled") / v("coal_made")),
        ("coal: hands raising what spoiled, over all hands", |v| v("coal_spoiled") * v("coal_hands") / v("coal_out").max(1e-9) / (v("carters") + v("supply") + v("navvies"))),
        ("coal: collieries' hands over all hands", |v| v("coal_hands") / (v("carters") + v("supply") + v("navvies"))),
        ("coal: price", |v| v("coal_price")),
        ("timber: island stock in months of sales and shipments", |v| v("timber_stock") / ((v("timber_bought") + v("timber_sent")) / 12.0)),
        ("timber: share of the stock held in towns with over twelve months", |v| v("timber_long12") / v("timber_stock")),
        ("timber: stock beyond four months, in months of the island's sales", |v| v("timber_over4") / ((v("timber_bought") + v("timber_sent")) / 12.0)),
        ("timber: spoiled over made", |v| v("timber_spoiled") / v("timber_made")),
        ("timber: hands raising what spoiled, over all hands", |v| v("timber_spoiled") * v("timber_hands") / v("timber_out").max(1e-9) / (v("carters") + v("supply") + v("navvies"))),
        ("timber: cleared woodland over all timber made", |v| v("timber_cleared") / v("timber_made")),
        ("timber: of the cleared, the share landing in towns that end the year beyond four months", |v| v("timber_cleared_over4") / v("timber_cleared").max(1e-9)),
        ("timber: woods' hands over all hands", |v| v("timber_hands") / (v("carters") + v("supply") + v("navvies"))),
        ("timber: price", |v| v("timber_price")),
        ("grain: island stock in months of sales and shipments", |v| v("grain_stock") / ((v("grain_bought") + v("grain_sent")) / 12.0)),
        ("grain: share of the stock held in towns with over twelve months", |v| v("grain_long12") / v("grain_stock")),
        ("grain: spoiled over made", |v| v("grain_spoiled") / v("grain_made")),
    ];
    table("(c) coal, timber and, for comparison, grain held in store", &twenty, c);

    // (d) hunger, by period: counts are per run and summed over the span, so the
    // measure here is the yearly mean times the span's length
    let d: &[Measure] = &[
        ("years with a hungry town (people::growth's rule)", |v| if v("hungry_towns") > 0.0 { 1.0 } else { 0.0 }),
        ("hungry town-years", |v| v("hungry_towns")),
        ("town-months under 0.82 of food", |v| v("hungry_months")),
        ("the year's worst town's food (worst month)", |v| v("worst_food")),
        ("years with a news item of hunger", |v| if v("news_hunger") > 0.0 { 1.0 } else { 0.0 }),
        ("years with a news item of sailing", |v| if v("news_sailing") > 0.0 { 1.0 } else { 0.0 }),
        ("years with either (the sweep's famine_years)", |v| if v("news_hunger") + v("news_sailing") > 0.0 { 1.0 } else { 0.0 }),
        ("grain price", |v| v("grain_price")),
        ("households' food over their spending", |v| v("food_households") / v("spend_households")),
        ("households' warmth over their spending", |v| v("warmth_households") / v("spend_households")),
        ("grain sent between towns over grain made", |v| v("grain_sent") / v("grain_made")),
        ("grain's carting between towns, a unit", |v| v("grain_trade_cart") / v("grain_trade_qty")),
        ("  the same over the grain's worth where it left", |v| v("grain_trade_cart") / v("grain_trade_value")),
    ];
    let halves: Vec<(i64, i64)> = vec![(1751, 1800), (1801, 1850), (1851, 1900), (1751, 1900)];
    table("(d) hunger (counts are yearly means: multiply by the span's years for a run's count)", &halves, d);
}
