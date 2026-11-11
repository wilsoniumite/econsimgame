//! What the sweep's scorecard does not show about prices and cycles: swings by
//! half-century, stocks and unmet requests month by month, how fast the economy
//! answers a discovery, and what a sudden loss of iron capacity does. Written to
//! measure `prices` (docs/rules/prices.md); it works on any rules.
//!
//!     cargo run --release -p isostate-core --example prices_probe -- --out FILE.json
//!         [--seeds 1-8] [--rules port|game] [--families N] [--set key=value]...
//!         [--shock] [--jobs N] [--compare OTHER.json]
//!
//! One setting a call (each `--set` takes one value, by the sweep's names). Every
//! seed is run to 1900 and reduced to a list of numbers; the table gives each
//! number's mean and its least and greatest value across the seeds, and with
//! `--compare` sets the mean beside the other file's, with a star where the two
//! ranges across seeds do not overlap.
//!
//! With `--shock` every seed is run a second time, and at the end of 1815 every
//! works that makes iron loses 30 per cent of its capacity (a flood, a fire).
//! The table then follows the shocked run against the unshocked one for twenty
//! years, as the mean over seeds of the log of the ratio. The two runs draw
//! different random numbers after the shock, so a single seed's path is noisy.
//!
//! The numbers:
//!
//! - `swing_SERIES_PERIOD`: the standard deviation of the yearly change in the log
//!   of the series' annual mean, within 1750-99, 1800-49 and 1850-1900.
//! - `zigzag_PERIOD`: the sweep's farm zigzag (mean absolute second difference of
//!   the farm share of employment, times 100) within each period.
//! - `news_*`: alarms, verdicts (and the depressions among them), and the booms,
//!   gluts, shortages and slumps the observer called, per run.
//! - `years_output_falls_2pc`, `_4pc`, `years_iron_price_moves_20pc`.
//! - `unmet_GOOD_PERIOD`: the share of what buyers asked for that they did not
//!   get; `worst_fill_GOOD`: the worst year's share got; `stock_months_GOOD_PERIOD`:
//!   the merchants' stock over a month's sales; `utilisation_GOOD_PERIOD`: output
//!   over capacity; `glut_made_GOOD`: the share of output made in towns holding
//!   over six months of sales and shipments of it.
//! - `wage_baskets_DECADE`: the average wage in subsistence baskets (the town's
//!   wage over the cost of its subsistence bundle, weighted by people). Unlike the
//!   observer's real wage it is not scaled to the run's own 1750, so it compares
//!   levels between rules. `out_GOOD_DECADE`: output a month.
//! - `years_to_half_METHOD`: from discovery to half of its good's output;
//!   `rail_km_plus10`, `_plus25`: line open ten and twenty-five years after the
//!   locomotive.
//! - with families: the player's rank among them in 1900 (taking every default)
//!   and the families' shares of works, rent and money.
//!
//! The yearly loop below is `sim::run_year` with a sample taken after each
//! month; if that function changes, change this with it.
#![allow(non_snake_case)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use isostate_core::facts::{self, Gathered};
use isostate_core::sweep::{parse_seeds, parse_value, set_field};
use isostate_core::*;
use serde_json::{Value, json};

const PERIODS: [(&str, i64, i64); 3] = [("1750-99", 1750, 1799), ("1800-49", 1800, 1849), ("1850-1900", 1850, 1900)];
const SWING: [&str; 9] =
    ["price_iron", "price_rails", "price_machinery", "price_cloth", "price_grain", "out_iron", "out_grain", "ag_share", "gdp_real"];
const WATCHED: [&str; 3] = ["iron", "rails", "machinery"];
const DECADES: [i64; 4] = [1750, 1790, 1840, 1890];
const OUTPUTS: [&str; 5] = ["iron", "cloth", "grain", "coal", "machinery"];
const RESPONSE: [&str; 3] = ["coke_furnace", "textile_mill", "rolling_mill"];
const SHOCK_YEAR: i64 = 1815;
const SHOCK_KEEP: f64 = 0.7;
const SHOCK_SPAN: usize = 20;

struct Options {
    out: PathBuf,
    seeds: Vec<u64>,
    rules: Rules,
    families: Option<usize>,
    sets: Vec<(String, String)>,
    shock: bool,
    jobs: usize,
    compare: Option<PathBuf>,
    command: String,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut o = Options {
        out: PathBuf::new(),
        seeds: (1..=8).collect(),
        rules: Rules::Port,
        families: None,
        sets: Vec::new(),
        shock: false,
        jobs: 8,
        compare: None,
        command: args.join(" "),
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = || it.next().cloned().ok_or(format!("{a} needs a value"));
        match a.as_str() {
            "--out" => o.out = PathBuf::from(value()?),
            "--seeds" => o.seeds = parse_seeds(&value()?)?,
            "--rules" => o.rules = Rules::parse(&value()?).ok_or("--rules takes port or game")?,
            "--families" => o.families = Some(value()?.parse().map_err(|e| format!("--families: {e}"))?),
            "--set" => {
                let v = value()?;
                let (k, x) = v.split_once('=').ok_or(format!("--set {v}: write it as key=value"))?;
                o.sets.push((k.to_string(), x.to_string()));
            }
            "--shock" => o.shock = true,
            "--jobs" => o.jobs = value()?.parse().map_err(|e| format!("--jobs: {e}"))?,
            "--compare" => o.compare = Some(PathBuf::from(value()?)),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    if o.out.as_os_str().is_empty() {
        return Err("--out FILE.json is needed".to_string());
    }
    Ok(o)
}

/// One year of one run, as the monthly samples add up.
#[derive(Clone, Default)]
struct YearSample {
    /// Per watched good: requests, sales, stock (summed over the months), output,
    /// capacity, and output made in towns with over six months of stock.
    req: Vec<f64>,
    sold: Vec<f64>,
    stock: Vec<f64>,
    made: Vec<f64>,
    cap: Vec<f64>,
    glut_made: Vec<f64>,
    /// The wage in subsistence baskets, summed over the months.
    baskets: f64,
    /// Capacity of all iron works at the year's end.
    iron_K: f64,
}

struct OneRun {
    result: RunResult,
    gathered: Gathered,
    years: Vec<YearSample>,
    seconds: f64,
}

fn capacity_of(st: &State, C: &Content, g: usize) -> f64 {
    (0..st.F_count).filter(|&f| st.f_alive[f] && C.method_out[st.f_method[f]] == g).map(|f| st.f_K[f]).sum()
}

/// A whole run, sampled every month; with `shock`, iron works lose capacity at the end of `SHOCK_YEAR`.
fn one_run(P: &Params, content: Content, shock: bool) -> Result<OneRun, String> {
    let t0 = std::time::Instant::now();
    let mut s = sim::start(P, content);
    s.st.watch_markets = true;
    let C = s.content.clone();
    let goods: Vec<usize> = WATCHED.iter().map(|n| C.gi(n)).collect();
    let iron = C.gi("iron");
    let n = goods.len();
    let mut gathered = Gathered::default();
    let mut years = Vec::new();
    while s.year <= s.end_year {
        let mut ys = YearSample { req: vec![0.0; n], sold: vec![0.0; n], stock: vec![0.0; n], made: vec![0.0; n], cap: vec![0.0; n], glut_made: vec![0.0; n], ..Default::default() };
        let mut prod_before = mat::Mat::zeros(s.st.T_max, C.G);
        for month in 0..12 {
            sim::run_month(&mut s, month)?;
            let st = &s.st;
            for (k, &g) in goods.iter().enumerate() {
                for t in 0..st.T_max {
                    ys.req[k] += st.month_requests[(t, g)];
                    ys.sold[k] += st.month_sold[(t, g)];
                    ys.stock[k] += st.inv[(t, g)];
                    let made = st.town_prod[(t, g)] - prod_before[(t, g)];
                    if st.inv[(t, g)] > 6.0 * st.outflow[(t, g)] {
                        ys.glut_made[k] += made;
                    }
                }
                ys.made[k] += st.month_stats.prod[g];
                ys.cap[k] += capacity_of(st, &C, g);
            }
            prod_before = st.town_prod.clone();
            let (mut w, mut p) = (0.0, 0.0);
            for t in 0..st.T_max {
                if st.town_active[t] && st.subcost[t] > 0.0 {
                    w += st.pop[t] * st.wage[t] / st.subcost[t];
                    p += st.pop[t];
                }
            }
            ys.baskets += w / p.max(1.0);
        }
        // the rest of sim::run_year
        let year = s.year;
        {
            let Sim { st, content, params, rngs, events, rec, .. } = &mut s;
            sim::yearly(st, content, params, rngs, year, events, false);
            observer::yearly(st, content, params, year, events);
            record::snapshot(rec, st, content, params, (year - content.start_year) as usize);
        }
        s.year += 1;
        facts::gather_year(&mut gathered, &s);
        if shock && year == SHOCK_YEAR {
            for f in 0..s.st.F_count {
                if s.st.f_alive[f] && C.method_out[s.st.f_method[f]] == iron {
                    s.st.f_K[f] *= SHOCK_KEEP;
                }
            }
        }
        ys.iron_K = capacity_of(&s.st, &C, iron);
        years.push(ys);
    }
    Ok(OneRun { result: sim::finish(s), gathered, years, seconds: t0.elapsed().as_secs_f64() })
}

fn mean(v: &[f64]) -> f64 {
    let f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if f.is_empty() { f64::NAN } else { f.iter().sum::<f64>() / f.len() as f64 }
}

fn sd(v: &[f64]) -> f64 {
    let f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if f.len() < 2 {
        return f64::NAN;
    }
    let m = mean(&f);
    (f.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / f.len() as f64).sqrt()
}

/// The numbers of one run, in printing order.
fn metrics(run: &OneRun) -> Vec<(String, f64)> {
    let r = &run.result;
    let (st, C) = (&r.state, &r.content);
    let y0 = C.start_year;
    let ann = facts::annual_means(r);
    let series = |name: &str| -> &[f64] { st.series.index.get(name).map_or(&[][..], |&k| &ann[k]) };
    let span = |a: i64, b: i64| ((a - y0) as usize, ((b - y0) as usize + 1).min(run.years.len()));
    let mut m: Vec<(String, f64)> = Vec::new();

    // swings by half-century: changes into the years of the period
    for name in SWING {
        let v = series(name);
        for (label, a, b) in PERIODS {
            let (i, j) = span(a.max(y0 + 1), b);
            let d: Vec<f64> = (i..j.min(v.len())).map(|k| (v[k] / v[k - 1]).ln()).collect();
            m.push((format!("swing_{name}_{label}"), sd(&d)));
        }
    }
    let farm = series("ag_share");
    for (label, a, b) in PERIODS {
        let (i, j) = span(a.max(y0 + 2), b);
        let z: Vec<f64> = (i..j.min(farm.len())).map(|k| (farm[k] - 2.0 * farm[k - 1] + farm[k - 2]).abs()).collect();
        m.push((format!("zigzag_{label}"), 100.0 * mean(&z)));
    }

    // the news
    let mut count: BTreeMap<&str, f64> = BTreeMap::new();
    for e in &r.events {
        match e.kind.as_str() {
            "alarm" => *count.entry("alarms").or_default() += 1.0,
            "verdict" => {
                *count.entry("verdicts").or_default() += 1.0;
                if e.title.contains("depression") {
                    *count.entry("depressions").or_default() += 1.0;
                }
            }
            "steep" => {
                // one good ("Iron boom") or several under one headline ("Booms in 3 industries")
                let words: Vec<&str> = e.title.split_whitespace().collect();
                let (kind, n) = match words.as_slice() {
                    [what, "in", n, "industries"] => (what.to_lowercase(), n.parse::<f64>().unwrap_or(1.0)),
                    _ => (format!("{}s", words.last().unwrap_or(&"")), 1.0),
                };
                for k in ["booms", "gluts", "shortages", "slumps"] {
                    if kind == k {
                        *count.entry(k).or_default() += n;
                    }
                }
            }
            _ => {}
        }
    }
    for k in ["alarms", "verdicts", "depressions", "booms", "gluts", "shortages", "slumps"] {
        m.push((format!("news_{k}"), count.get(k).copied().unwrap_or(0.0)));
    }
    let out = series("gdp_real");
    let falls = |by: f64| (1..out.len()).filter(|&k| out[k] / out[k - 1] - 1.0 < -by).count() as f64;
    m.push(("years_output_falls_2pc".into(), falls(0.02)));
    m.push(("years_output_falls_4pc".into(), falls(0.04)));
    let pi = series("price_iron");
    m.push(("years_iron_price_moves_20pc".into(), (1..pi.len()).filter(|&k| (pi[k] / pi[k - 1]).ln().abs() > 0.2).count() as f64));

    // markets, month by month
    for (k, good) in WATCHED.iter().enumerate() {
        let ys = &run.years;
        for (label, a, b) in PERIODS {
            let (i, j) = span(a, b);
            let sum = |f: &dyn Fn(&YearSample) -> f64| ys[i..j].iter().map(f).sum::<f64>();
            let (req, sold) = (sum(&|y| y.req[k]), sum(&|y| y.sold[k]));
            m.push((format!("unmet_{good}_{label}"), if req > 0.0 { 1.0 - sold / req } else { f64::NAN }));
            m.push((format!("stock_months_{good}_{label}"), if sold > 0.0 { sum(&|y| y.stock[k]) / sold } else { f64::NAN }));
            let cap = sum(&|y| y.cap[k]);
            m.push((format!("utilisation_{good}_{label}"), if cap > 0.0 { sum(&|y| y.made[k]) / cap } else { f64::NAN }));
        }
        let fills: Vec<f64> = ys.iter().filter(|y| y.req[k] > 0.0).map(|y| y.sold[k] / y.req[k]).collect();
        m.push((format!("worst_fill_{good}"), fills.iter().copied().fold(f64::NAN, f64::min)));
        let made: f64 = ys.iter().map(|y| y.made[k]).sum();
        m.push((format!("glut_made_{good}"), if made > 0.0 { ys.iter().map(|y| y.glut_made[k]).sum::<f64>() / made } else { f64::NAN }));
    }

    // levels
    let decade = |v: &[f64], d: i64| -> f64 {
        let (i, j) = span(d, d + 9);
        mean(&v[i.min(v.len())..j.min(v.len())])
    };
    let baskets: Vec<f64> = run.years.iter().map(|y| y.baskets / 12.0).collect();
    for d in DECADES {
        m.push((format!("wage_baskets_{d}s"), decade(&baskets, d)));
    }
    for d in &DECADES[1..] {
        m.push((format!("real_wage_{d}s_vs_1750"), decade(series("real_wage"), *d)));
    }
    for good in OUTPUTS {
        for d in &DECADES[1..] {
            m.push((format!("out_{good}_{d}s"), decade(series(&format!("out_{good}")), *d)));
        }
    }
    m.push(("pop_1900".into(), series("pop").last().copied().unwrap_or(f64::NAN)));

    // how fast a real change is answered
    let f = facts::stylised_facts(r, &run.gathered, run.seconds);
    for name in RESPONSE {
        let years = match (f.get(&format!("discovered_{name}")), f.get(&format!("half_of_output_{name}"))) {
            (Some(d), Some(h)) => h - d,
            _ => f64::NAN,
        };
        m.push((format!("years_to_half_{name}"), years));
    }
    let loco = f.get("discovered_railway");
    m.push(("discovered_railway".into(), loco.unwrap_or(f64::NAN)));
    let km = series("rail_km");
    for plus in [10, 25] {
        let v = loco.and_then(|y| km.get((y as i64 + plus - y0) as usize).copied()).unwrap_or(f64::NAN);
        m.push((format!("rail_km_plus{plus}"), v));
    }
    for name in ["works_1900", "works_closed", "works_closed_median_life", "farm_zigzag"] {
        m.push((name.to_string(), f.get(name).unwrap_or(f64::NAN)));
    }

    // the families, if there are any
    if families::active(st) {
        let worth: Vec<f64> = (0..st.families.len()).map(|k| family_report(st, C, &r.params, k).map_or(f64::NAN, |x| x.net_worth)).collect();
        m.push(("player_rank_1900".into(), 1.0 + worth.iter().filter(|&&w| w > worth[0]).count() as f64));
        let (works, rent, money) = families::ownership_shares(st, C);
        m.push(("families_share_of_works".into(), works));
        m.push(("families_share_of_rent".into(), rent));
        m.push(("families_share_of_money".into(), money));
    }
    m
}

/// The shocked run against the unshocked one: for each of `SHOCK_SPAN` years after
/// the shock, the log of the ratio of iron's price, capacity and output and of real output.
fn shock_paths(base: &OneRun, shocked: &OneRun) -> Vec<(&'static str, Vec<f64>)> {
    let y0 = base.result.content.start_year;
    let k0 = (SHOCK_YEAR - y0) as usize;
    let path = |get: &dyn Fn(&OneRun, usize) -> f64| -> Vec<f64> { (1..=SHOCK_SPAN).map(|d| (get(shocked, k0 + d) / get(base, k0 + d)).ln()).collect() };
    let ann_b = facts::annual_means(&base.result);
    let ann_s = facts::annual_means(&shocked.result);
    let (ann_b, ann_s) = (&ann_b, &ann_s);
    let series = |name: &'static str| {
        move |run: &OneRun, k: usize| -> f64 {
            let ann = if std::ptr::eq(run, shocked) { ann_s } else { ann_b };
            run.result.state.series.index.get(name).map_or(f64::NAN, |&i| ann[i][k])
        }
    };
    vec![
        ("iron price", path(&series("price_iron"))),
        // capacity at the end of the year before: the shock itself is year 0
        ("iron capacity", path(&|run, k| run.years[k - 1].iron_K)),
        ("iron output", path(&series("out_iron"))),
        ("real output", path(&series("gdp_real"))),
    ]
}

fn fmt(v: f64) -> String {
    if !v.is_finite() {
        "-".to_string()
    } else if v.abs() >= 1000.0 {
        format!("{v:.0}")
    } else if v.abs() >= 20.0 {
        format!("{v:.1}")
    } else if v.abs() >= 2.0 {
        format!("{v:.2}")
    } else {
        format!("{v:.3}")
    }
}

fn range(v: &[f64]) -> (f64, f64) {
    let f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    (f.iter().copied().fold(f64::INFINITY, f64::min), f.iter().copied().fold(f64::NEG_INFINITY, f64::max))
}

fn main_with(args: &[String]) -> Result<(), String> {
    let o = parse_args(args)?;
    let mut P = o.rules.params();
    if let Some(n) = o.families {
        P.n_families = n;
    }
    for (k, v) in &o.sets {
        P = set_field(&P, k, &parse_value(v))?;
    }
    let content = o.rules.content();
    let todo: Vec<(u64, bool)> = o.seeds.iter().flat_map(|&s| if o.shock { vec![(s, false), (s, true)] } else { vec![(s, false)] }).collect();
    let next = AtomicUsize::new(0);
    let slots: Mutex<Vec<Option<Result<OneRun, String>>>> = Mutex::new((0..todo.len()).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..o.jobs.clamp(1, todo.len()) {
            scope.spawn(|| {
                loop {
                    let id = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&(seed, shock)) = todo.get(id) else { break };
                    let mut P = P.clone();
                    P.seed = seed;
                    let res = one_run(&P, content.clone(), shock);
                    eprintln!(
                        "seed {seed}{}: {}",
                        if shock { ", shocked" } else { "" },
                        match &res {
                            Ok(r) => format!("{:.0} s", r.seconds),
                            Err(e) => format!("FAILED: {e}"),
                        }
                    );
                    slots.lock().unwrap()[id] = Some(res);
                }
            });
        }
    });
    let mut runs: BTreeMap<(u64, bool), OneRun> = BTreeMap::new();
    for (key, res) in todo.iter().zip(slots.into_inner().unwrap()) {
        runs.insert(*key, res.expect("every run reports")?);
    }

    // per metric, one value per seed
    let mut names: Vec<String> = Vec::new();
    let mut table: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for &seed in &o.seeds {
        for (name, v) in metrics(&runs[&(seed, false)]) {
            if !table.contains_key(&name) {
                names.push(name.clone());
            }
            table.entry(name).or_default().push(v);
        }
    }
    let mut shock: Vec<(&'static str, Vec<Vec<f64>>)> = Vec::new();
    if o.shock {
        for &seed in &o.seeds {
            for (k, (name, path)) in shock_paths(&runs[&(seed, false)], &runs[&(seed, true)]).into_iter().enumerate() {
                if shock.len() <= k {
                    shock.push((name, Vec::new()));
                }
                shock[k].1.push(path);
            }
        }
    }

    let then: Option<Value> = match &o.compare {
        Some(p) => Some(serde_json::from_str(&std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?).map_err(|e| format!("{}: {e}", p.display()))?),
        None => None,
    };
    let then_of = |name: &str| -> Option<Vec<f64>> {
        let v = then.as_ref()?.get("metrics")?.get(name)?.as_array()?;
        Some(v.iter().map(|x| x.as_f64().unwrap_or(f64::NAN)).collect())
    };
    let mut text = format!("{} seeds: {:?}; {}\n", o.seeds.len(), o.seeds, o.command);
    text.push_str("each number: mean [least .. greatest] across seeds");
    if then.is_some() {
        text.push_str("; then the other file's mean, the change, and * where the ranges across seeds do not overlap");
    }
    text.push('\n');
    for name in &names {
        let v = &table[name];
        let (lo, hi) = range(v);
        let mut line = format!("  {name:<34} {:>9} [{} .. {}]", fmt(mean(v)), fmt(lo), fmt(hi));
        if let Some(t) = then_of(name) {
            let (tlo, thi) = range(&t);
            let star = if lo > thi || hi < tlo { " *" } else { "" };
            line = format!("{line:<70} then {:>9}   {:>+9}{star}", fmt(mean(&t)), fmt(mean(v) - mean(&t)));
        }
        text.push_str(&line);
        text.push('\n');
    }
    if o.shock {
        text.push_str(&format!(
            "\n{:.0} per cent of iron capacity lost at the end of {SHOCK_YEAR}: log of shocked over unshocked, mean of seeds, years 1 to {SHOCK_SPAN}\n",
            100.0 * (1.0 - SHOCK_KEEP)
        ));
        for (name, paths) in &shock {
            let means: Vec<f64> = (0..SHOCK_SPAN).map(|d| mean(&paths.iter().map(|p| p[d]).collect::<Vec<_>>())).collect();
            text.push_str(&format!("  {name:<14}{}\n", means.iter().map(|v| format!(" {v:+.2}")).collect::<String>()));
        }
    }
    print!("{text}");

    let doc = json!({
        "command": o.command,
        "seeds": o.seeds,
        "metrics": names.iter().map(|n| (n.clone(), json!(table[n].iter().map(|v| if v.is_finite() { json!(v) } else { Value::Null }).collect::<Vec<_>>()))).collect::<serde_json::Map<_, _>>(),
        "shock": shock.iter().map(|(n, p)| (n.to_string(), json!(p))).collect::<serde_json::Map<_, _>>(),
    });
    if let Some(dir) = o.out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(&o.out, serde_json::to_string_pretty(&doc).unwrap()).map_err(|e| format!("{}: {e}", o.out.display()))?;
    std::fs::write(o.out.with_extension("txt"), &text).map_err(|e| format!("{}: {e}", o.out.display()))?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = main_with(&args) {
        eprintln!("prices_probe: {e}");
        std::process::exit(1);
    }
}
