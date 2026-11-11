//! The market of every good and method, for reading what the production tree
//! does in a run (`docs/rules/tree.md`).
//!
//!     cargo run -p isostate-core --release --example tree_market -- [--rules port|game]
//!         [--families N] [--set JSON] [--end-year Y] [--time] [SEED ...]
//!
//! `--set` takes a JSON object of parameter overrides laid over the chosen
//! rules: `--set '{"tree":{"stage":0}}'` is the game without the tree, and
//! `--set '{"tree":{"file":"my.toml"}}'` tries an overlay file of your own.
//! Seeds default to 1 to 8; each runs on its own thread and the tables are means
//! over seeds. With `--time` the seeds run one after another and the last line
//! says what a simulated year cost.
//!
//! It prints, at 1750, 1800, 1850 and the last year:
//!
//! - for each method: the year it is aimed at, the year found and first built
//!   (the mean over the seeds where that happened, and in how many), then works
//!   standing, output a month and its share of its good's output;
//! - for each good: its price over its base price, output a month, and the share
//!   of town-years its price stood at its floor or its ceiling;
//! - for each need: the share of household spending on it, and the share of the
//!   need each good serves;
//! - hands by land sector and by trade (the works that make one good), as shares
//!   of all hands at work.
#![allow(non_snake_case)]

use std::time::Instant;

use isostate_core::params::Rules;
use isostate_core::*;

/// One run, read at the marks.
struct Run {
    C: Content,
    marks: Vec<i64>,
    found: Vec<i64>,
    first: Vec<i64>,
    /// [mark][method]
    works: Vec<Vec<f64>>,
    mout: Vec<Vec<f64>>,
    hands_m: Vec<Vec<f64>>,
    /// [mark][sector]
    hands_s: Vec<Vec<f64>>,
    /// [mark]
    hands: Vec<f64>,
    carters: Vec<f64>,
    /// [mark][good]
    price: Vec<Vec<f64>>,
    out: Vec<Vec<f64>>,
    /// [good]: town-years at the floor, at the ceiling, and all town-years
    floor: Vec<f64>,
    ceiling: Vec<f64>,
    town_years: f64,
    /// [mark][need], and [mark][good * S + need]
    budget: Vec<Vec<f64>>,
    serve: Vec<Vec<f64>>,
    wage: Vec<f64>,
    real_wage: Vec<f64>,
    urban: Vec<f64>,
    /// [year][good]: everything made since 1750, at the end of each year
    made: Vec<Vec<f64>>,
    /// (NT, G): the thresholds as the run resolved them
    thresholds: mat::Mat,
    seconds_a_year: f64,
    /// With `--time`: the seconds spent in the months and in the yearly step.
    seconds_months: f64,
    seconds_yearly: f64,
}

fn run(P: &Params, C: Content, timed: bool) -> Run {
    let mut sim = sim::start(P, C);
    let C = sim.content.clone();
    let (y0, y1) = (C.start_year, sim.end_year);
    let marks: Vec<i64> = [1750, 1800, 1850, 1900].iter().map(|&y| (y as i64).min(y1)).filter(|&y| y >= y0).collect();
    let mut marks = marks;
    marks.dedup();
    let mut r = Run {
        marks: marks.clone(),
        found: vec![-1; C.M],
        first: vec![-1; C.M],
        works: Vec::new(),
        mout: Vec::new(),
        hands_m: Vec::new(),
        hands_s: Vec::new(),
        hands: Vec::new(),
        carters: Vec::new(),
        price: Vec::new(),
        out: Vec::new(),
        floor: vec![0.0; C.G],
        ceiling: vec![0.0; C.G],
        town_years: 0.0,
        budget: Vec::new(),
        serve: Vec::new(),
        wage: Vec::new(),
        real_wage: Vec::new(),
        urban: Vec::new(),
        made: Vec::new(),
        thresholds: sim.st.disc_X0.clone(),
        seconds_a_year: 0.0,
        seconds_months: 0.0,
        seconds_yearly: 0.0,
        C: C.clone(),
    };
    let t0 = Instant::now();
    while sim.year <= sim.end_year {
        let year = sim.year;
        if timed {
            // the same year as `sim::run_year`, taken apart to see where the time goes
            let t = Instant::now();
            for month in 0..12 {
                sim::run_month(&mut sim, month).unwrap_or_else(|e| panic!("seed {}: {e}", P.seed));
            }
            r.seconds_months += t.elapsed().as_secs_f64();
            let t = Instant::now();
            let sim::Sim { st, content: C, params: P, rngs, events, rec, .. } = &mut sim;
            sim::yearly(st, C, P, rngs, year, events, false);
            r.seconds_yearly += t.elapsed().as_secs_f64();
            observer::yearly(st, C, P, year, events);
            chronicle::yearly(st, C, P, year, events);
            record::snapshot(rec, st, C, P, (year - C.start_year) as usize);
            harvest::snapshot(rec, st, P, (year - C.start_year) as usize);
            sim.year += 1;
        } else {
            sim::run_year(&mut sim).unwrap_or_else(|e| panic!("seed {}: {e}", P.seed));
        }
        let st = &sim.st;
        let ms = &st.month_stats;
        r.made.push(st.cumprod.clone());
        for m in 0..C.M {
            if r.first[m] < 0 && ms.method_out[m] > 0.0 {
                r.first[m] = year;
            }
        }
        for t in (0..st.T_max).filter(|&t| st.town_active[t]) {
            r.town_years += 1.0;
            for g in 0..C.G {
                let p = st.price[(t, g)] / C.base_price[g];
                if p <= 1.001 * P.price_floor {
                    r.floor[g] += 1.0;
                }
                if p >= 0.999 * P.price_ceiling {
                    r.ceiling[g] += 1.0;
                }
            }
        }
        if !marks.contains(&year) {
            continue;
        }
        let mut works = vec![0.0; C.M];
        for f in (0..st.f_alive.len()).filter(|&f| st.f_alive[f]) {
            works[st.f_method[f]] += 1.0;
        }
        r.works.push(works);
        // the year's means, from the observer's monthly series
        let mean = |name: &str| -> f64 {
            if st.series.index.contains_key(name) { observer::annual(st.series.get(name), y0, year) } else { f64::NAN }
        };
        r.mout.push(C.method_name.iter().map(|n| mean(&format!("mout_{n}"))).collect());
        r.price.push(C.good_name.iter().map(|n| mean(&format!("price_{n}"))).collect());
        r.out.push(C.good_name.iter().map(|n| mean(&format!("out_{n}"))).collect());
        r.budget.push(C.service_name.iter().map(|n| mean(&format!("budget_{n}"))).collect());
        r.wage.push(mean("wage"));
        r.real_wage.push(mean("real_wage"));
        r.urban.push(mean("urban_share"));
        // the share of each need asked of each good in December (the logit's
        // mix at each town's prices), weighted by the towns' people
        let mut serve = vec![0.0; C.G * C.S];
        let people: f64 = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| st.pop[t]).sum();
        for t in (0..st.T_max).filter(|&t| st.town_active[t]) {
            for k in 0..C.G * C.S {
                serve[k] += ms.share[(t, k)] * st.pop[t] / people.max(1.0);
            }
        }
        r.serve.push(serve);
        let hands: f64 = ms.employment.iter().sum();
        r.hands.push(hands);
        r.hands_m.push(ms.firm_labor_by_method.clone());
        r.hands_s.push(ms.ex_labor_by_sector.clone());
        let on_land: f64 = ms.ex_labor_by_sector.iter().sum();
        let in_works: f64 = ms.firm_labor_by_method.iter().sum();
        r.carters.push((hands - on_land - in_works).max(0.0));
    }
    r.seconds_a_year = t0.elapsed().as_secs_f64() / (y1 - y0 + 1).max(1) as f64;
    for m in 0..C.M {
        r.found[m] = if C.tech_known[m] { -1 } else { sim.st.tech_year[m] };
    }
    r
}

/// The mean of the finite values, and how many there were.
fn mean(v: impl Iterator<Item = f64>) -> (f64, usize) {
    let v: Vec<f64> = v.filter(|x| x.is_finite()).collect();
    if v.is_empty() { (f64::NAN, 0) } else { (v.iter().sum::<f64>() / v.len() as f64, v.len()) }
}

fn num(x: f64, w: usize, d: usize) -> String {
    if x.is_finite() { format!("{x:>w$.d$}") } else { format!("{:>w$}", "-") }
}

fn main() {
    let mut rules = Rules::Game;
    let mut families = 6;
    let mut seeds: Vec<u64> = Vec::new();
    let mut set: Option<String> = None;
    let mut end_year: Option<i64> = None;
    let mut time = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--rules" => rules = Rules::parse(&args.next().expect("--rules port|game")).expect("--rules port|game"),
            "--families" => families = args.next().and_then(|v| v.parse().ok()).expect("--families N"),
            "--set" => set = Some(args.next().expect("--set JSON")),
            "--end-year" => end_year = Some(args.next().and_then(|v| v.parse().ok()).expect("--end-year Y")),
            "--time" => time = true,
            _ => seeds.push(a.parse().unwrap_or_else(|_| panic!("{a} is not a seed"))),
        }
    }
    if seeds.is_empty() {
        seeds = (1..=8).collect();
    }
    let mut P = rules.params();
    if let Some(text) = &set {
        P = P.with_json_overrides(text).unwrap_or_else(|e| panic!("{e}"));
    }
    P.n_families = families;
    if end_year.is_some() {
        P.end_year = end_year;
    }
    let base = rules.content();
    let all: Vec<Run> = if time {
        seeds
            .iter()
            .map(|&seed| {
                let mut P = P.clone();
                P.seed = seed;
                run(&P, base.clone(), true)
            })
            .collect()
    } else {
        std::thread::scope(|sc| {
            let handles: Vec<_> = seeds
                .iter()
                .map(|&seed| {
                    let (mut P, C) = (P.clone(), base.clone());
                    P.seed = seed;
                    sc.spawn(move || run(&P, C, false))
                })
                .collect();
            handles.into_iter().map(|h| h.join().expect("a run panicked")).collect()
        })
    };
    let C = &all[0].C;
    let marks = &all[0].marks;
    let n = all.len();
    let over = |f: &dyn Fn(&Run) -> f64| mean(all.iter().map(f)).0;
    println!(
        "{} rules, tree stage {}, {} families, seeds {:?}{}",
        rules.as_str(),
        C.stage,
        families,
        seeds,
        set.as_ref().map_or(String::new(), |s| format!(", {s}"))
    );
    let head: String = marks.iter().map(|y| format!("{y:>9}")).collect();

    println!("\nMETHODS: found and first built (mean year, and in how many of {n} seeds); works standing; output a month; share of its good");
    println!("{:<20}{:<10}{:>5}{:>11}{:>11}  works{head}  output{head}  share{head}", "method", "makes", "aim", "found", "built", head = &head[5.min(head.len())..]);
    for m in 0..C.M {
        let (found, nf) = mean(all.iter().map(|r| if r.found[m] >= 0 { r.found[m] as f64 } else { f64::NAN }));
        let (first, nb) = mean(all.iter().map(|r| if r.first[m] >= 0 { r.first[m] as f64 } else { f64::NAN }));
        let aim = if C.tech_target[m] > 0 { C.tech_target[m].to_string() } else { String::new() };
        let when = |x: f64, k: usize| if C.tech_known[m] && k == n && x.is_finite() && x <= C.start_year as f64 { format!("{:>11}", "1750") } else if k == 0 { format!("{:>11}", "never") } else { format!("{x:>6.0} ({k}/{n})") };
        let mut line = format!("{:<20}{:<10}{:>5}{}{}", C.method_name[m], C.good_name[C.method_out[m]], aim, if C.tech_known[m] { format!("{:>11}", "known") } else { when(found, nf) }, when(first, nb));
        line.push_str("  ");
        for k in 0..marks.len() {
            line.push_str(&num(over(&|r| r.works[k][m]), 9, 1));
        }
        line.push_str("  ");
        for k in 0..marks.len() {
            line.push_str(&num(over(&|r| r.mout[k][m]), 9, 0));
        }
        line.push_str("  ");
        for k in 0..marks.len() {
            let g = C.method_out[m];
            line.push_str(&num(over(&|r| r.mout[k][m] / r.out[k][g].max(1e-9)), 9, 2));
        }
        println!("{line}");
    }

    // for whoever fits thresholds: what the island had made of each threshold's
    // good four years before the year aimed at, and when the threshold was met
    println!("\nTHRESHOLDS of the inventions that carry an aim: the threshold; what the island had made four years before the aim; the year the threshold was met");
    let y0 = C.start_year;
    for t in (0..C.NT).filter(|&t| C.tech_target[t] > 0) {
        let k = (C.tech_target[t] - 4 - y0).max(0) as usize;
        let (found, nf) = mean(all.iter().map(|r| if t < C.M && r.found[t] >= 0 { r.found[t] as f64 } else { f64::NAN }));
        let mut line = format!("{:<20}aim {}  found {}", C.tech_name[t], C.tech_target[t], if nf == 0 { "never".to_string() } else { format!("{found:.0}") });
        for g in (0..C.G).filter(|&g| all[0].thresholds[(t, g)] > 0.0) {
            let x = over(&|r| r.thresholds[(t, g)]);
            let had = over(&|r| r.made.get(k).map_or(f64::NAN, |v| v[g]));
            let (met, nm) = mean(all.iter().map(|r| r.made.iter().position(|v| v[g] >= r.thresholds[(t, g)]).map_or(f64::NAN, |i| (y0 + i as i64) as f64)));
            line.push_str(&format!("   {}: {:.3e}, had {:.3e}, met {}", C.good_name[g], x, had, if nm == 0 { "never".to_string() } else { format!("{met:.0} ({nm}/{n})") }));
        }
        println!("{line}");
    }

    println!("\nGOODS: price over base price; output a month; share of town-years at the floor and at the ceiling");
    println!("{:<12}{:>7}  p/base{head}  output{head}   floor ceiling", "good", "base", head = &head[5.min(head.len())..]);
    for g in 0..C.G {
        let mut line = format!("{:<12}{:>7.2}  ", C.good_name[g], C.base_price[g]);
        for k in 0..marks.len() {
            line.push_str(&num(over(&|r| r.price[k][g] / C.base_price[g]), 9, 2));
        }
        line.push_str("  ");
        for k in 0..marks.len() {
            line.push_str(&num(over(&|r| r.out[k][g]), 9, 0));
        }
        line.push_str(&format!("  {:>6.2}{:>8.2}", over(&|r| r.floor[g] / r.town_years.max(1.0)), over(&|r| r.ceiling[g] / r.town_years.max(1.0))));
        println!("{line}");
    }

    println!("\nNEEDS: share of household spending (the Engel rule's series; blank without it), then the share of the need each good serves");
    for s in 0..C.S {
        let mut line = format!("{:<32}", format!("{} ({})", C.service_label[s], C.service_name[s]));
        for k in 0..marks.len() {
            line.push_str(&num(over(&|r| r.budget[k][s]), 9, 3));
        }
        println!("{line}{}", if s == 0 { format!("    {}", marks.iter().map(|y| y.to_string()).collect::<Vec<_>>().join(" ")) } else { String::new() });
        for g in (0..C.G).filter(|&g| C.props[(g, s)] > 0.0) {
            let mut line = format!("    {:<28}", C.good_name[g]);
            for k in 0..marks.len() {
                line.push_str(&num(over(&|r| r.serve[k][g * C.S + s]), 9, 3));
            }
            println!("{line}");
        }
    }

    println!("\nHANDS: share of all hands at work, in December of each year");
    println!("{:<32}{head}", "");
    for s in 0..C.NS {
        let mut line = format!("{:<32}", format!("land: {}", C.sector_name[s]));
        for k in 0..marks.len() {
            line.push_str(&num(over(&|r| r.hands_s[k][s] / r.hands[k].max(1e-9)), 9, 3));
        }
        println!("{line}");
    }
    for g in 0..C.G {
        let ms: Vec<usize> = (0..C.M).filter(|&m| C.method_out[m] == g).collect();
        if ms.is_empty() {
            continue;
        }
        let mut line = format!("{:<32}", format!("works: {}", C.good_name[g]));
        for k in 0..marks.len() {
            line.push_str(&num(over(&|r| ms.iter().map(|&m| r.hands_m[k][m]).sum::<f64>() / r.hands[k].max(1e-9)), 9, 3));
        }
        println!("{line}");
    }
    for (label, f) in [
        ("carters and the rest", (&|r: &Run, k: usize| r.carters[k] / r.hands[k].max(1e-9)) as &dyn Fn(&Run, usize) -> f64),
        ("all land", &|r: &Run, k: usize| r.hands_s[k].iter().sum::<f64>() / r.hands[k].max(1e-9)),
        ("all works", &|r: &Run, k: usize| r.hands_m[k].iter().sum::<f64>() / r.hands[k].max(1e-9)),
        ("hands at work", &|r: &Run, k: usize| r.hands[k]),
        ("wage", &|r: &Run, k: usize| r.wage[k]),
        ("real wage, 1750 = 1", &|r: &Run, k: usize| r.real_wage[k]),
        ("living in towns", &|r: &Run, k: usize| r.urban[k]),
    ] {
        let mut line = format!("{label:<32}");
        for k in 0..marks.len() {
            let v = over(&|r| f(r, k));
            line.push_str(&num(v, 9, if v.abs() >= 100.0 { 0 } else { 3 }));
        }
        println!("{line}");
    }
    println!(
        "\na simulated year took {:.3} s on average{}",
        over(&|r| r.seconds_a_year),
        if time { " (one run at a time)" } else { " (the seeds ran side by side: for timing use --time)" }
    );
    if time {
        let years = (all[0].made.len() as f64).max(1.0);
        println!(
            "of which the twelve months {:.3} s and the yearly step {:.3} s",
            over(&|r| r.seconds_months) / years,
            over(&|r| r.seconds_yearly) / years
        );
    }
}
