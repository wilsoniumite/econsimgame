//! The numbers the machines mechanism is judged by (docs/rules/machines.md), which
//! the sweep's scorecard does not have: output per worker on the land, the real
//! wage on a chained index, how far machinery's price runs away, how fast the
//! engine works answer, and when the converter and the steel rail mill arrive.
//!
//!     cargo run -p isostate-core --release --example machines -- [--seeds 1-8] [--rules port|game]
//!         [--content port|game] [--families N] [--end-year Y] [--set key=value]... [--jobs N] [--out DIR]
//!
//! `--set` takes one value per key (`--set machines.enabled=true`), by the same
//! names as the sweep. `--content` picks the content apart from the rules, so the
//! mechanism's rules can be measured with and without its content changes. With
//! `--out` it writes `years.csv`, one row per seed and year, and `summary.txt`,
//! the table it prints.
//!
//! Real wages are five-year means (1800 is 1798 to 1802, 1900 is 1896 to 1900)
//! over the mean of 1750 to 1752, because single years ride the iron cycle. They
//! are given on the model's own index (the 1750 basket) and on a chained Fisher
//! index of what households and landlords bought, linked year by year.
#![allow(non_snake_case)]

use std::fmt::Write as _;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use isostate_core::content::Content;
use isostate_core::observer::{self, annual};
use isostate_core::params::Rules;
use isostate_core::sim::{self, Sim};
use isostate_core::sweep::{parse_seeds, parse_value, set_field};
use isostate_core::*;

/// The land sectors reported, by the content's names.
const SECTORS: [&str; 5] = ["farm", "quarry", "claypit", "coalmine", "ironmine"];

#[derive(Clone, Default)]
struct Year {
    year: i64,
    wage_model: f64,
    wage_nominal: f64,
    /// Chained Fisher price index of what was bought, 1 in the first year.
    chain: f64,
    /// Output per worker-month in each of `SECTORS`.
    per_worker: [f64; 5],
    /// Share of all employment in each of `SECTORS`.
    labor_share: [f64; 5],
    price_mach: f64,
    out_mach: f64,
    /// Machinery makers at the year's end: built capacity, capacity on order,
    /// works, and how many of them count as building (and so may not grow).
    mach_K: f64,
    mach_order: f64,
    mach_works: usize,
    mach_building: usize,
    quality: f64,
    /// Share of farm and of quarry output worked with machines (output-weighted over towns).
    farm_worked: f64,
    quarry_worked: f64,
    out_steel: f64,
    out_bessemer: f64,
    out_rails: f64,
    out_steel_rolling: f64,
}

struct Run {
    seed: u64,
    years: Vec<Year>,
    first: [Option<i64>; 4],
    money_ok: Result<(), String>,
}

/// A year of the run, as `sim::run_year` steps it, noting what the observer does not keep.
fn step_year(s: &mut Sim, C: &Content, basket: &mut Vec<(Vec<f64>, Vec<f64>)>) -> Result<Year, String> {
    let (G, NS) = (C.G, C.NS);
    let sec_idx: Vec<Option<usize>> = SECTORS.iter().map(|n| C.sector_name.iter().position(|x| x == n)).collect();
    let (mut out, mut lab) = (vec![0.0; NS], vec![0.0; NS]);
    let (mut q, mut pq, mut emp) = (vec![0.0; G], vec![0.0; G], 0.0);
    for month in 0..12 {
        sim::run_month(s, month)?;
        let st = &s.st;
        for u in 0..st.ex_cell.len() {
            out[st.ex_sector[u]] += st.ex_out[u];
        }
        let ms = &st.month_stats;
        for k in 0..NS {
            lab[k] += ms.ex_labor_by_sector[k];
        }
        emp += ms.employment.iter().sum::<f64>();
        for t in 0..st.T_max {
            for g in 0..G {
                let x = ms.hh_got[(t, g)] + ms.lord_got[(t, g)];
                q[g] += x;
                pq[g] += x * st.price[(t, g)];
            }
        }
    }
    // output of each (town, sector) in December, to weight the towns' adoption
    // (the yearly step redraws the land units, so it is taken before it)
    let mut town_out = vec![0.0; s.st.T_max * NS];
    for u in 0..s.st.ex_cell.len() {
        town_out[s.st.ex_town[u] * NS + s.st.ex_sector[u]] += s.st.ex_out[u];
    }
    let year = s.year;
    {
        let Sim { st, content, params, rngs, events, rec, .. } = s;
        sim::yearly(st, content, params, rngs, year, events, false);
        observer::yearly(st, content, params, year, events);
        record::snapshot(rec, st, content, params, (year - content.start_year) as usize);
    }
    s.year += 1;
    // the year's average price of each good, weighted by what was bought
    let p: Vec<f64> = (0..G).map(|g| if q[g] > 0.0 { pq[g] / q[g] } else { 0.0 }).collect();
    basket.push((p, q));
    let (st, P) = (&s.st, &s.params);
    let mut y = Year { year, ..Year::default() };
    for (k, si) in sec_idx.iter().enumerate() {
        if let Some(si) = *si {
            y.per_worker[k] = out[si] / lab[si].max(1e-9);
            y.labor_share[k] = lab[si] / emp.max(1e-9);
        }
    }
    let gm = C.gi("machinery");
    for f in (0..st.F_count).filter(|&f| st.f_alive[f] && C.method_out[st.f_method[f]] == gm) {
        y.mach_K += st.f_K[f];
        y.mach_order += st.f_order_K[f];
        y.mach_works += 1;
        y.mach_building += (st.f_order_K[f] > 0.0) as usize;
    }
    y.quality = machines::quality(st, C, P);
    let worked = |names: &[&str]| -> f64 {
        let (mut num, mut den) = (0.0, 0.0);
        for si in (0..NS).filter(|&si| names.contains(&C.sector_name[si].as_str())) {
            for t in (0..st.T_max).filter(|&t| st.town_active[t]) {
                let w = town_out[t * NS + si];
                den += w;
                if P.machines.enabled {
                    // the machines standing, for the uses bought where they pay
                    let stock = |u: &machines::Use| if st.machine_stock.rows > 0 { st.machine_stock[(t, si * machines::SLOTS + u.slot)] } else { 0.0 };
                    num += w * machines::uses(st, C, P, si, t, &st.pexp).iter().filter(|u| u.priced).map(stock).sum::<f64>();
                } else if C.sector_name[si] == "farm" {
                    let k = st.know[(t, st.t_fmach)];
                    num += w * if k > 0.05 { k } else { 0.0 };
                }
            }
        }
        if den > 0.0 { num / den } else { 0.0 }
    };
    y.farm_worked = worked(&["farm"]);
    y.quarry_worked = worked(&["quarry", "claypit"]);
    Ok(y)
}

fn one(P: &Params, content: Content) -> Run {
    let mut s = sim::start(P, content);
    let C = s.content.clone();
    let mut years = Vec::new();
    let mut basket = Vec::new();
    let mut money_ok = Ok(());
    while s.year <= s.end_year {
        match step_year(&mut s, &C, &mut basket) {
            Ok(y) => years.push(y),
            Err(e) => {
                money_ok = Err(e);
                break;
            }
        }
    }
    // chained Fisher index, linked year by year
    let mut chain = 1.0;
    for k in 0..years.len() {
        if k > 0 {
            let ((p0, q0), (p1, q1)) = (&basket[k - 1], &basket[k]);
            let dot = |p: &[f64], q: &[f64]| p.iter().zip(q).map(|(a, b)| a * b).sum::<f64>();
            let (lasp, paas) = (dot(p1, q0) / dot(p0, q0).max(1e-12), dot(p1, q1) / dot(p0, q1).max(1e-12));
            chain *= (lasp * paas).sqrt();
        }
        years[k].chain = chain;
    }
    let y0 = C.start_year;
    let st = &s.st;
    let series = |name: &str| -> Vec<f64> {
        match st.series.index.get(name) {
            Some(&i) => (0..years.len()).map(|k| annual(&st.series.data[i], y0, y0 + k as i64)).collect(),
            None => vec![0.0; years.len()],
        }
    };
    let cols = [
        series("real_wage"),
        series("wage"),
        series("price_machinery"),
        series("out_machinery"),
        series("out_steel"),
        series("mout_bessemer"),
        series("out_rails"),
        series("mout_steel_rolling"),
    ];
    for (k, y) in years.iter_mut().enumerate() {
        y.wage_model = cols[0][k];
        y.wage_nominal = cols[1][k];
        y.price_mach = cols[2][k];
        y.out_mach = cols[3][k];
        y.out_steel = cols[4][k];
        y.out_bessemer = cols[5][k];
        y.out_rails = cols[6][k];
        y.out_steel_rolling = cols[7][k];
    }
    let found = |name: &str| C.method_name.iter().position(|x| x == name).map(|m| st.tech_year[m]).filter(|&y| y >= y0);
    let built = |v: &dyn Fn(&Year) -> f64| years.iter().find(|y| v(y) > 0.0).map(|y| y.year);
    let first = [found("bessemer"), built(&|y| y.out_bessemer), found("steel_rolling"), built(&|y| y.out_steel_rolling)];
    Run { seed: P.seed, years, first, money_ok }
}

fn mean(v: &[f64]) -> f64 {
    let v: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
}

fn median(v: &[f64]) -> f64 {
    let mut v: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    v.sort_by(f64::total_cmp);
    if v.is_empty() { f64::NAN } else { 0.5 * (v[(v.len() - 1) / 2] + v[v.len() / 2]) }
}

/// The mean of `f` over the years a to b (those the run has).
fn span(r: &Run, a: i64, b: i64, f: &dyn Fn(&Year) -> f64) -> f64 {
    mean(&r.years.iter().filter(|y| y.year >= a && y.year <= b).map(f).collect::<Vec<_>>())
}

fn at(r: &Run, year: i64, f: &dyn Fn(&Year) -> f64) -> f64 {
    r.years.iter().find(|y| y.year == year).map_or(f64::NAN, f)
}

/// One row of the table: a label, a value per seed, then the mean and the range.
fn row(out: &mut String, label: &str, vals: &[f64], dp: usize) {
    let fin: Vec<f64> = vals.iter().copied().filter(|v| v.is_finite()).collect();
    let _ = write!(out, "{label:<38}");
    for v in vals {
        if v.is_finite() {
            let _ = write!(out, " {v:>7.dp$}");
        } else {
            let _ = write!(out, " {:>7}", "-");
        }
    }
    if fin.is_empty() {
        let _ = writeln!(out, " | {:>7}", "-");
    } else {
        let (lo, hi) = (fin.iter().copied().fold(f64::INFINITY, f64::min), fin.iter().copied().fold(f64::NEG_INFINITY, f64::max));
        let note = if fin.len() < vals.len() { format!(" ({}/{})", fin.len(), vals.len()) } else { String::new() };
        let _ = writeln!(out, " | {:>7.dp$} [{lo:.dp$} .. {hi:.dp$}]{note}", mean(&fin));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut seeds: Vec<u64> = (1..=8).collect();
    let mut rules = Rules::Port;
    let mut content: Option<Rules> = None;
    let (mut families, mut end_year, mut jobs, mut out_dir) = (None, None, 8usize, None);
    let mut sets: Vec<(String, String)> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = || it.next().cloned().unwrap_or_else(|| panic!("{a} needs a value"));
        match a.as_str() {
            "--seeds" => seeds = parse_seeds(&value()).unwrap_or_else(|e| panic!("{e}")),
            "--rules" => rules = Rules::parse(&value()).expect("--rules port|game"),
            "--content" => content = Some(Rules::parse(&value()).expect("--content port|game")),
            "--families" => families = Some(value().parse::<usize>().expect("--families N")),
            "--end-year" => end_year = Some(value().parse::<i64>().expect("--end-year Y")),
            "--jobs" => jobs = value().parse::<usize>().expect("--jobs N").max(1),
            "--out" => out_dir = Some(std::path::PathBuf::from(value())),
            "--set" => {
                let v = value();
                let (k, x) = v.split_once('=').unwrap_or_else(|| panic!("--set {v}: write it as key=value"));
                sets.push((k.trim().to_string(), x.trim().to_string()));
            }
            other => panic!("unknown option {other}"),
        }
    }
    let mut P = rules.params();
    for (k, v) in &sets {
        P = set_field(&P, k, &parse_value(v)).unwrap_or_else(|e| panic!("{e}"));
    }
    if let Some(n) = families {
        P.n_families = n;
    }
    if end_year.is_some() {
        P.end_year = end_year;
    }
    let C = content.unwrap_or(rules).content();

    let next = AtomicUsize::new(0);
    let slots: Mutex<Vec<Option<Run>>> = Mutex::new((0..seeds.len()).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..jobs.min(seeds.len()) {
            scope.spawn(|| {
                loop {
                    let id = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&seed) = seeds.get(id) else { break };
                    let mut P = P.clone();
                    P.seed = seed;
                    let r = one(&P, C.clone());
                    slots.lock().unwrap()[id] = Some(r);
                }
            });
        }
    });
    let runs: Vec<Run> = slots.into_inner().unwrap().into_iter().map(|r| r.expect("every run reports")).collect();

    let mut t = String::new();
    let what = format!(
        "{} rules, {} content{}{}",
        rules.as_str(),
        content.unwrap_or(rules).as_str(),
        if P.n_families > 0 { format!(", {} families", P.n_families) } else { String::new() },
        sets.iter().map(|(k, v)| format!(", {k}={v}")).collect::<String>()
    );
    let _ = writeln!(t, "{what}");
    let _ = write!(t, "{:<38}", "seed");
    for r in &runs {
        let _ = write!(t, " {:>7}", r.seed);
    }
    let _ = writeln!(t, " |    mean [least .. greatest]");
    let failed: Vec<String> = runs.iter().filter_map(|r| r.money_ok.as_ref().err().map(|e| format!("seed {}: {e}", r.seed))).collect();
    let col = |f: &dyn Fn(&Run) -> f64| -> Vec<f64> { runs.iter().map(f).collect() };
    for (label, wage) in [("model's index", 0), ("chained index", 1)] {
        let w = move |y: &Year| if wage == 0 { y.wage_model } else { y.wage_nominal / y.chain };
        let base = |r: &Run| span(r, 1750, 1752, &w);
        let _ = writeln!(t, "real wage, {label} (1750-52 = 1)");
        row(&mut t, "  1800 (1798-1802)", &col(&|r| span(r, 1798, 1802, &w) / base(r)), 2);
        row(&mut t, "  1850 (1848-1852)", &col(&|r| span(r, 1848, 1852, &w) / base(r)), 2);
        row(&mut t, "  1900 (1896-1900)", &col(&|r| span(r, 1896, 1900, &w) / base(r)), 2);
        row(&mut t, "  lowest 5-year mean, 1855-1885", &col(&|r| (1855..=1881).map(|a| span(r, a, a + 4, &w)).fold(f64::INFINITY, f64::min) / base(r)), 2);
        row(&mut t, "  1890s over 1850", &col(&|r| span(r, 1890, 1899, &w) / span(r, 1848, 1852, &w)), 2);
        row(&mut t, "  1850 over 1800", &col(&|r| span(r, 1848, 1852, &w) / span(r, 1798, 1802, &w)), 2);
    }
    let _ = writeln!(t, "machinery");
    let usual = |r: &Run| median(&r.years.iter().filter(|y| y.year >= 1836 && y.year <= 1850).map(|y| y.price_mach).collect::<Vec<_>>());
    let peak = |r: &Run| r.years.iter().filter(|y| y.year > 1850).map(|y| y.price_mach).fold(f64::NEG_INFINITY, f64::max);
    row(&mut t, "  usual price, median of 1836-50", &col(&usual), 0);
    row(&mut t, "  peak yearly price after 1850", &col(&peak), 0);
    row(&mut t, "  peak over usual", &col(&|r| peak(r) / usual(r)), 1);
    row(&mut t, "  years after 1850 above 3x usual", &col(&|r| r.years.iter().filter(|y| y.year > 1850 && y.price_mach > 3.0 * usual(r)).count() as f64), 0);
    for y in [1800, 1850, 1900] {
        row(&mut t, &format!("  output a month, {y}"), &col(&|r| span(r, y - 2, (y + 2).min(1900), &|v| v.out_mach)), 0);
    }
    row(&mut t, "  quality of machines, 1850", &col(&|r| at(r, 1850, &|v| v.quality)), 2);
    row(&mut t, "  quality of machines, 1900", &col(&|r| at(r, 1900, &|v| v.quality)), 2);
    for y in [1860, 1880, 1900] {
        row(&mut t, &format!("  farm output worked by machine, {y}"), &col(&|r| at(r, y, &|v| v.farm_worked)), 2);
    }
    for y in [1850, 1900] {
        row(&mut t, &format!("  quarry and pit output so worked, {y}"), &col(&|r| at(r, y, &|v| v.quarry_worked)), 2);
    }
    let _ = writeln!(t, "output per worker-month (five-year means)");
    for (k, name) in SECTORS.iter().enumerate() {
        for y in [1800, 1850, 1900] {
            row(&mut t, &format!("  {name}, {y}"), &col(&|r| span(r, y - 2, (y + 2).min(1900), &|v| v.per_worker[k])), 2);
        }
        row(&mut t, &format!("  {name}, 1900 over 1850"), &col(&|r| span(r, 1896, 1900, &|v| v.per_worker[k]) / span(r, 1848, 1852, &|v| v.per_worker[k])), 2);
        row(&mut t, &format!("  {name}, share of all work in 1900"), &col(&|r| span(r, 1896, 1900, &|v| v.labor_share[k])), 3);
    }
    let _ = writeln!(t, "steel");
    row(&mut t, "  converter found", &col(&|r| r.first[0].map_or(f64::NAN, |y| y as f64)), 0);
    row(&mut t, "  converter first built", &col(&|r| r.first[1].map_or(f64::NAN, |y| y as f64)), 0);
    row(&mut t, "  converter's share of steel, 1880", &col(&|r| at(r, 1880, &|v| v.out_bessemer / v.out_steel.max(1e-9))), 2);
    row(&mut t, "  converter's share of steel, 1900", &col(&|r| at(r, 1900, &|v| v.out_bessemer / v.out_steel.max(1e-9))), 2);
    row(&mut t, "  steel rail mill found", &col(&|r| r.first[2].map_or(f64::NAN, |y| y as f64)), 0);
    row(&mut t, "  steel rail mill first built", &col(&|r| r.first[3].map_or(f64::NAN, |y| y as f64)), 0);
    row(&mut t, "  steel rails' share of rails, 1890", &col(&|r| at(r, 1890, &|v| v.out_steel_rolling / v.out_rails.max(1e-9))), 2);
    row(&mut t, "  steel rails' share of rails, 1900", &col(&|r| at(r, 1900, &|v| v.out_steel_rolling / v.out_rails.max(1e-9))), 2);
    for y in [1850, 1900] {
        row(&mut t, &format!("  steel output a month, {y}"), &col(&|r| span(r, y - 2, (y + 2).min(1900), &|v| v.out_steel)), 0);
    }
    if failed.is_empty() {
        let _ = writeln!(t, "no run failed the monthly money check");
    } else {
        let _ = writeln!(t, "FAILED: {}", failed.join("; "));
    }
    print!("{t}");

    if let Some(dir) = out_dir {
        std::fs::create_dir_all(&dir).expect("create --out");
        let mut csv = String::from(
            "seed,year,real_wage_model,real_wage_chained,price_machinery,out_machinery,mach_K,mach_order,mach_works,mach_building,quality,farm_worked,quarry_worked,out_steel,out_bessemer,out_rails,out_steel_rolling",
        );
        for name in SECTORS {
            let _ = write!(csv, ",per_worker_{name},labor_share_{name}");
        }
        csv.push('\n');
        for r in &runs {
            for y in &r.years {
                let _ = write!(
                    csv,
                    "{},{},{:.5},{:.5},{:.4},{:.4},{:.4},{:.4},{},{},{:.5},{:.5},{:.5},{:.3},{:.3},{:.3},{:.3}",
                    r.seed,
                    y.year,
                    y.wage_model,
                    y.wage_nominal / y.chain,
                    y.price_mach,
                    y.out_mach,
                    y.mach_K,
                    y.mach_order,
                    y.mach_works,
                    y.mach_building,
                    y.quality,
                    y.farm_worked,
                    y.quarry_worked,
                    y.out_steel,
                    y.out_bessemer,
                    y.out_rails,
                    y.out_steel_rolling
                );
                for k in 0..SECTORS.len() {
                    let _ = write!(csv, ",{:.5},{:.5}", y.per_worker[k], y.labor_share[k]);
                }
                csv.push('\n');
            }
        }
        std::fs::write(dir.join("years.csv"), csv).expect("write years.csv");
        std::fs::write(dir.join("summary.txt"), &t).expect("write summary.txt");
    }
    if !failed.is_empty() {
        std::process::exit(1);
    }
}
