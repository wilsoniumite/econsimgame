//! The floor of chance under the scorecard: how far its facts move between
//! histories of the same rules that differ by chance alone, and how far other
//! sweeps move them beside that (docs/probes/scorecard-baseline.md, where these
//! numbers are quoted).
//!
//!     cargo run --release -p isostate-core --example chance_floor -- HISTORY_DIR... [--against DIR]... [--facts card|all]
//!
//! Each HISTORY_DIR is a sweep (`isostate-core sweep`), and every setting in it
//! is one history: run the rules as they are, then again with a setting moved by
//! a few millionths, which changes nothing but the draws
//! (`--set harvest.island_sd=0.0499998,0.0499999,0.0500001,0.0500002`). Only the
//! seeds every history finished are read.
//!
//! For each fact, over the histories:
//!
//! - its mean across the seeds in each;
//! - **floor**, the largest gap between two of those means: how far a sweep's
//!   mean moves by chance alone, and so what a change must move it by before it
//!   is read as the change's doing. With five histories it is about two and a
//!   third standard deviations of a mean, a little more than chance gives two
//!   sweeps nine times in ten;
//! - `sd`, the standard deviation of those means (n - 1 below the line);
//! - one island's move: across the islands, the median and the largest of the
//!   gap between that island's highest and lowest value over the histories;
//! - `between`, the standard deviation across the islands of the first history,
//!   which is the scorecard's tolerance, and the island's median move as a share
//!   of it: near one, chance moves an island as far as the islands differ.
//!
//! `--facts card` (the default) reads the facts the scorecard reads (its aims,
//! the facts beside them and those without an aim) and every twenty-year mean of
//! the wage, output per head, the towns' share and work's share of income;
//! `--facts all` every fact the first history has. The loss is read too, once
//! with each history's own tolerances and once with the first history's
//! (`scorecard --spread-from`), and each aim's miss in tolerances.
//!
//! With `--against DIR`, every setting of that sweep is set beside the first
//! history on the seeds both have: its mean, the change, the change in floors,
//! and the islands up and down. `beyond` marks a change larger than the floor.
#![allow(non_snake_case)]
use isostate_core::scorecard::{judge, loss, AIMS, WITHOUT_AIM};
use isostate_core::sweep::{read_then, Then};
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

type Values = HashMap<String, Vec<(u64, f64)>>;

/// A name for one of the twenty-year means the floor is asked for.
fn twenty_year_mean(name: &str) -> bool {
    let rest = ["real_wage_", "output_per_head_", "urban_share_", "income_share_work_"].iter().find_map(|p| name.strip_prefix(p));
    let Some(rest) = rest else { return false };
    let rest = rest.strip_suffix("_vs_1750").unwrap_or(rest);
    let b = rest.as_bytes();
    b.len() == 9 && b[4] == b'_' && b.iter().enumerate().all(|(i, c)| i == 4 || c.is_ascii_digit())
}

/// Four significant figures, without a run of zeros after the point.
fn num(v: f64) -> String {
    if !v.is_finite() {
        return "-".to_string();
    }
    let a = v.abs();
    if a >= 10000.0 {
        format!("{:.0}", v)
    } else if a >= 100.0 {
        format!("{:.1}", v)
    } else if a >= 1.0 {
        format!("{:.3}", v)
    } else if a >= 0.001 || a == 0.0 {
        format!("{:.4}", v)
    } else {
        format!("{:.2e}", v)
    }
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
}

fn sd(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return f64::NAN;
    }
    let m = mean(v);
    (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { 0.5 * (v[n / 2 - 1] + v[n / 2]) }
}

/// One history's values of a fact, by seed, on the seeds read.
fn by_seed(h: &Values, fact: &str, seeds: &BTreeSet<u64>) -> HashMap<u64, f64> {
    h.get(fact).into_iter().flatten().filter(|(s, v)| seeds.contains(s) && v.is_finite()).map(|&(s, v)| (s, v)).collect()
}

/// Only the seeds in `seeds`, for handing to the scorecard.
fn only(h: &Values, seeds: &BTreeSet<u64>) -> Values {
    h.iter().map(|(k, v)| (k.clone(), v.iter().copied().filter(|(s, _)| seeds.contains(s)).collect())).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut dirs, mut against, mut all) = (Vec::<PathBuf>::new(), Vec::<PathBuf>::new(), false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--against" => against.push(PathBuf::from(it.next().expect("--against needs a sweep's folder"))),
            "--facts" => match it.next().map(String::as_str) {
                Some("card") => all = false,
                Some("all") => all = true,
                other => panic!("--facts is card or all, not {other:?}"),
            },
            _ => dirs.push(PathBuf::from(a)),
        }
    }
    if dirs.is_empty() {
        eprintln!("usage: chance_floor HISTORY_DIR... [--against DIR]... [--facts card|all]");
        std::process::exit(2);
    }
    let mut histories: Vec<(String, Then)> = Vec::new();
    for d in &dirs {
        for t in read_then(d).unwrap_or_else(|e| panic!("{e}")) {
            let name = d.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            histories.push((format!("{name}: {}", t.label), t));
        }
    }
    let H = histories.len();
    let mut seeds: BTreeSet<u64> = histories[0].1.seeds.iter().copied().collect();
    for (_, t) in &histories[1..] {
        let here: BTreeSet<u64> = t.seeds.iter().copied().collect();
        seeds = seeds.intersection(&here).copied().collect();
    }
    println!("{H} histories on {} seeds ({})", seeds.len(), seeds.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" "));
    for (k, (name, _)) in histories.iter().enumerate() {
        println!("  h{}  {name}", k + 1);
    }
    if H < 2 {
        eprintln!("one history has no floor: give two or more");
        std::process::exit(2);
    }

    // the facts to read, in the scorecard's order, then the twenty-year means
    let first = &histories[0].1.values;
    let mut facts: Vec<String> = Vec::new();
    let push = |f: &str, facts: &mut Vec<String>| {
        if !f.is_empty() && first.contains_key(f) && !facts.iter().any(|x| x == f) {
            facts.push(f.to_string());
        }
    };
    if all {
        let mut names: Vec<&String> = first.keys().collect();
        names.sort();
        for f in names {
            push(f, &mut facts);
        }
    } else {
        for a in AIMS.iter() {
            push(a.fact, &mut facts);
            push(a.beside, &mut facts);
        }
        for f in WITHOUT_AIM.iter() {
            push(f, &mut facts);
        }
        let mut means: Vec<&String> = first.keys().filter(|f| twenty_year_mean(f)).collect();
        means.sort();
        for f in means {
            push(f, &mut facts);
        }
    }

    // the floor of each fact
    let mut floors: HashMap<String, f64> = HashMap::new();
    let width = facts.iter().map(|f| f.len()).max().unwrap_or(4).max(28);
    println!();
    print!("{:<width$}", "fact");
    for k in 0..H {
        print!("  {:>9}", format!("h{}", k + 1));
    }
    println!("  {:>9}  {:>9}  {:>9}  {:>9}  {:>9}  {:>7}", "floor", "sd", "island", "most", "between", "isl/btw");
    for f in &facts {
        let per: Vec<HashMap<u64, f64>> = histories.iter().map(|(_, t)| by_seed(&t.values, f, &seeds)).collect();
        let means: Vec<f64> = per.iter().map(|m| mean(&m.values().copied().collect::<Vec<_>>())).collect();
        let finite: Vec<f64> = means.iter().copied().filter(|x| x.is_finite()).collect();
        let floor = if finite.len() >= 2 {
            finite.iter().copied().fold(f64::NEG_INFINITY, f64::max) - finite.iter().copied().fold(f64::INFINITY, f64::min)
        } else {
            f64::NAN
        };
        // one island's move: on the seeds every history has a value for
        let moves: Vec<f64> = seeds
            .iter()
            .filter_map(|s| {
                let v: Vec<f64> = per.iter().filter_map(|m| m.get(s).copied()).collect();
                (v.len() == H).then(|| v.iter().copied().fold(f64::NEG_INFINITY, f64::max) - v.iter().copied().fold(f64::INFINITY, f64::min))
            })
            .collect();
        let between = sd(&per[0].values().copied().collect::<Vec<_>>());
        let island = median(moves.clone());
        let most = moves.iter().copied().fold(f64::NAN, f64::max);
        print!("{f:<width$}");
        for m in &means {
            print!("  {:>9}", num(*m));
        }
        let ratio = if between > 0.0 { format!("{:.2}", island / between) } else { "-".to_string() };
        println!("  {:>9}  {:>9}  {:>9}  {:>9}  {:>9}  {:>7}", num(floor), num(sd(&finite)), num(island), num(most), num(between), ratio);
        floors.insert(f.clone(), floor);
    }

    // the aims and the loss in each history
    println!("\nThe scorecard in each history (own tolerances; in brackets, the first history's)");
    print!("{:<width$}", "aim: misses by, in tolerances");
    for k in 0..H {
        print!("  {:>15}", format!("h{}", k + 1));
    }
    println!();
    let own: Vec<Vec<isostate_core::scorecard::Judged>> = histories.iter().map(|(_, t)| judge(&only(&t.values, &seeds), None)).collect();
    let fixed: Vec<Vec<isostate_core::scorecard::Judged>> =
        histories.iter().map(|(_, t)| judge(&only(&t.values, &seeds), Some(&only(first, &seeds)))).collect();
    for (a, aim) in AIMS.iter().enumerate() {
        print!("{:<width$}", aim.fact);
        for k in 0..H {
            let met = if own[k][a].met { "met" } else { "no" };
            print!("  {:>15}", format!("{:.2} ({:.2}) {met}", own[k][a].misses_by, fixed[k][a].misses_by));
        }
        println!();
    }
    print!("{:<width$}", "loss");
    for k in 0..H {
        print!("  {:>15}", format!("{:.1} ({:.1})", loss(&own[k]), loss(&fixed[k])));
    }
    println!();
    let l_own: Vec<f64> = own.iter().map(|j| loss(j)).collect();
    let l_fix: Vec<f64> = fixed.iter().map(|j| loss(j)).collect();
    let gap = |v: &[f64]| v.iter().copied().fold(f64::NEG_INFINITY, f64::max) - v.iter().copied().fold(f64::INFINITY, f64::min);
    println!("floor of the loss: {:.1} with each history's own tolerances, {:.1} with the first history's", gap(&l_own), gap(&l_fix));

    // other sweeps beside the first history
    for d in &against {
        for t in read_then(d).unwrap_or_else(|e| panic!("{e}")) {
            let common: BTreeSet<u64> = t.seeds.iter().copied().filter(|s| seeds.contains(s)).collect();
            println!("\n{}: {}, against h1 on {} seeds", d.display(), t.label, common.len());
            println!("{:<width$}  {:>9}  {:>9}  {:>9}  {:>9}  {:>7}  {:>7}", "fact", "h1", "this", "change", "floor", "floors", "up/down");
            for f in &facts {
                let then = by_seed(first, f, &common);
                let now = by_seed(&t.values, f, &common);
                let (m0, m1) = (mean(&then.values().copied().collect::<Vec<_>>()), mean(&now.values().copied().collect::<Vec<_>>()));
                let floor = floors.get(f).copied().unwrap_or(f64::NAN);
                let change = m1 - m0;
                let (mut up, mut down) = (0, 0);
                for (s, v) in &now {
                    if let Some(w) = then.get(s) {
                        if v > w {
                            up += 1;
                        } else if v < w {
                            down += 1;
                        }
                    }
                }
                let times = if floor > 0.0 { change / floor } else { f64::NAN };
                let mark = if times.is_finite() && times.abs() > 1.0 { "  beyond" } else { "" };
                println!(
                    "{f:<width$}  {:>9}  {:>9}  {:>9}  {:>9}  {:>7}  {:>7}{mark}",
                    num(m0),
                    num(m1),
                    num(change),
                    num(floor),
                    if times.is_finite() { format!("{times:.1}") } else { "-".to_string() },
                    format!("{up}/{down}")
                );
            }
            let j_own = judge(&only(&t.values, &common), None);
            let j_fix = judge(&only(&t.values, &common), Some(&only(first, &common)));
            println!("loss: {:.1} with its own tolerances, {:.1} with h1's", loss(&j_own), loss(&j_fix));
        }
    }
}
