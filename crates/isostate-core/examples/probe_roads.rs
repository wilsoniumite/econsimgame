//! What the roads are, who pays for them and what carting over them is worth
//! (`docs/probes/dx-roads.md`). Runs several islands side by side and prints,
//! decade by decade, pooled over the islands:
//!
//! - kilometres of track, road and turnpike, and of railway and canal;
//! - cells improved (and fallen back) in a year;
//! - where roads cross a river, and how wide the rivers are there;
//! - the carting paid over each kind of way in a year, and what each step of
//!   road saves, as shares of the year's output;
//! - the railway's and the canal's fares, and what tolls would come to;
//! - with `roads.enabled`, the trusts, the parishes' works and the purses.
//!
//!     cargo run --release -p isostate-core --example probe_roads -- [--seeds 1-8] [--rules port|game]
//!         [--families N] [--land] [--set NAME=VALUE]...
//!
//! The rules default to the game's with six families, which is what the sweep
//! reads. `--land` switches on the second island generator (real rivers).
#![allow(non_snake_case)]
use isostate_core::roads::{crossings, river_width};
use isostate_core::transport::{RAIL_COST, ROAD_MULT, TERRAIN_COST};
use isostate_core::*;

/// One island's numbers for one year.
#[derive(Clone, Debug, Default)]
struct Year {
    year: i64,
    cells: [f64; 4], // land cells at road level 0..3 (no rail)
    rail: f64,
    canal: f64,
    up: [f64; 4],  // cells that rose to level k this year
    down: f64,     // cells that fell a level
    cart: [f64; 6], // carting paid over: no road, track, road, turnpike, navigable river, railway
    saved: [f64; 4], // what each road level saves on the level below, on its traffic
    gdp: f64,      // the year's output, money
    cross: [Vec<f64>; 4], // widths of the crossings by roads of level >= k (k = 1..3)
    turnpike_traffic: Vec<f64>,
    rail_fares: f64,
    rail_capital: f64,
    canal_fares: f64,
    canal_capital: f64,
    lords: f64,
    hh: f64,
    roads_spent: f64,
    tolls: f64,
    trusts: f64,
    menders: f64,
    pop: f64,
    real_wage: f64,
}

fn island(P: Params, C: Content) -> (Vec<Year>, f64) {
    let mut sim = sim::start(&P, C);
    let mut out: Vec<Year> = Vec::new();
    let mut prev_road = sim.st.road.clone();
    while sim.year <= sim.end_year {
        sim::run_year(&mut sim).unwrap_or_else(|e| panic!("{e}"));
        let st = &sim.st;
        let w = &st.world;
        let year = sim.year - 1;
        let mut y = Year { year, ..Year::default() };
        let fr = sim.params.freight;
        for c in 0..st.N {
            if !w.land[c] {
                continue;
            }
            let r = st.road[c] as usize;
            if st.rail[c] > 0 {
                y.rail += 1.0;
            } else {
                y.cells[r] += 1.0;
            }
            if st.canal.get(c).copied().unwrap_or(0) > 0 {
                y.canal += 1.0;
            }
            if st.road[c] > prev_road[c] {
                y.up[r] += 1.0;
            } else if st.road[c] < prev_road[c] {
                y.down += 1.0;
            }
            let terr = TERRAIN_COST[w.terrain[c] as usize];
            let tf = st.traffic[c];
            let kind = if st.rail[c] > 0 {
                5
            } else if w.navigable[c] {
                4
            } else {
                r
            };
            let cost = match kind {
                5 => RAIL_COST,
                4 => 0.3,
                0 => terr * if st.cover[c] > 0.6 { 1.25 } else { 1.0 },
                _ => terr * ROAD_MULT[r],
            };
            y.cart[kind] += fr * tf * cost;
            if kind >= 1 && kind <= 3 {
                let below = if r == 1 { terr * if st.cover[c] > 0.6 { 1.25 } else { 1.0 } } else { terr * ROAD_MULT[r - 1] };
                y.saved[r] += fr * tf * (below - terr * ROAD_MULT[r]);
            }
            if kind == 3 {
                y.turnpike_traffic.push(tf);
            }
        }
        if year % 10 == 0 || year == sim.end_year {
            for k in 1..4 {
                y.cross[k] = crossings(w, &st.road, k as u8).into_iter().map(|(_, wd)| wd).collect();
            }
        }
        if let Some(n) = st.rw.years.last().filter(|n| n.year == year) {
            y.rail_fares = n.fares;
            y.rail_capital = n.capital;
        }
        if let Some(n) = st.cn.book.years.last().filter(|n| n.year == year) {
            y.canal_fares = n.fares;
            y.canal_capital = n.capital;
        }
        y.lords = st.lord_cash.iter().sum();
        y.hh = st.hh_cash.iter().sum();
        if let Some(n) = st.rd.years.last().filter(|n| n.year == year) {
            y.roads_spent = n.spent;
            y.tolls = n.tolls;
            y.trusts = n.trusts as f64;
            y.menders = n.menders;
        }
        out.push(y);
        prev_road = st.road.clone();
    }
    let st = &sim.st;
    let y0 = sim.content.start_year;
    let series = |name: &str| st.series.index.get(name).map_or(&[][..], |&k| &st.series.data[k][..]);
    for y in out.iter_mut() {
        y.gdp = 12.0 * observer::annual(series("gdp_nom"), y0, y.year);
        y.pop = observer::annual(series("pop"), y0, y.year);
        y.real_wage = observer::annual(series("real_wage"), y0, y.year);
    }
    // the second generator's flow per cell of area upstream, on its river cells
    let w = &st.world;
    let ratio = match &w.landform {
        Some(lf) => {
            let mut r: Vec<f64> = (0..st.N).filter(|&c| w.river[c] && w.land[c] && w.acc[c] > 0.0 && !lf.estuary[c]).map(|c| lf.discharge[c] as f64 / w.acc[c]).collect();
            r.sort_by(f64::total_cmp);
            if r.is_empty() { f64::NAN } else { r[r.len() / 2] }
        }
        None => f64::NAN,
    };
    (out, ratio)
}

fn quant(v: &mut Vec<f64>, q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

/// mean [least .. greatest] over islands
fn spread(v: &[f64]) -> String {
    let ok: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if ok.is_empty() {
        return "-".into();
    }
    let m = ok.iter().sum::<f64>() / ok.len() as f64;
    let lo = ok.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = ok.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let f = |x: f64| if x.abs() >= 100.0 { format!("{x:.0}") } else if x.abs() >= 1.0 { format!("{x:.2}") } else { format!("{x:.4}") };
    format!("{} [{} .. {}]", f(m), f(lo), f(hi))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut rules = Rules::Game;
    let (mut seeds, mut families, mut land) = ((1..=8).collect::<Vec<u64>>(), 6usize, false);
    let mut sets: Vec<(String, String)> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--rules" => {
                rules = Rules::parse(&args[i + 1]).expect("--rules port|game");
                i += 1;
            }
            "--families" => {
                families = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--seeds" => {
                let s = &args[i + 1];
                seeds = match s.split_once('-') {
                    Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                    None => s.split(',').map(|x| x.parse().unwrap()).collect(),
                };
                i += 1;
            }
            "--set" => {
                let (k, v) = args[i + 1].split_once('=').expect("--set NAME=VALUE");
                sets.push((k.to_string(), v.to_string()));
                i += 1;
            }
            "--land" => land = true,
            s => panic!("what is {s}?"),
        }
        i += 1;
    }
    let mut P = rules.params();
    P.n_families = families;
    P.land.enabled = land || P.land.enabled;
    for (k, v) in &sets {
        let mut json = v.clone();
        for part in k.rsplit('.') {
            json = format!("{{\"{part}\": {json}}}");
        }
        P = P.with_json_overrides(&json).unwrap_or_else(|e| panic!("--set {k}={v}: {e}"));
    }
    let C = rules.content();
    println!(
        "{} rules, {families} families, seeds {:?}, land {}, roads {}{}",
        rules.as_str(),
        seeds,
        if P.land.enabled { "on" } else { "off" },
        if P.roads.enabled { "on" } else { "off" },
        sets.iter().map(|(k, v)| format!(", {k}={v}")).collect::<String>()
    );
    let runs: Vec<(Vec<Year>, f64)> = std::thread::scope(|s| {
        let hs: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let mut P = P.clone();
                P.seed = seed;
                let C = C.clone();
                s.spawn(move || island(P, C))
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });
    let years: Vec<i64> = runs[0].0.iter().map(|y| y.year).filter(|y| y % 10 == 0 || *y == 1900).collect();
    let get = |y: i64, f: &dyn Fn(&Year) -> f64| -> Vec<f64> { runs.iter().map(|(r, _)| r.iter().find(|x| x.year == y).map_or(f64::NAN, f)).collect() };
    // over the ten years ending in y
    let decade = |y: i64, f: &dyn Fn(&Year) -> f64| -> Vec<f64> {
        runs.iter().map(|(r, _)| r.iter().filter(|x| x.year > y - 10 && x.year <= y).map(f).sum::<f64>() / 10.0).collect()
    };

    println!("\n## Kilometres of each kind (a cell is 2 km), mean [least .. greatest] over islands");
    println!("{:>5}  {:>26} {:>26} {:>26} {:>26} {:>26}", "year", "track", "road", "turnpike", "railway", "canal");
    for &y in &years {
        let km = |k: usize| get(y, &|x: &Year| 2.0 * x.cells[k]);
        println!(
            "{y:>5}  {:>26} {:>26} {:>26} {:>26} {:>26}",
            spread(&km(1)),
            spread(&km(2)),
            spread(&km(3)),
            spread(&get(y, &|x: &Year| 2.0 * x.rail)),
            spread(&get(y, &|x: &Year| 2.0 * x.canal))
        );
    }

    println!("\n## Cells improved in a year (mean of the ten years to the date), and fallen back a level");
    println!("{:>5}  {:>22} {:>22} {:>22} {:>22}", "year", "to track", "to road", "to turnpike", "fell back");
    for &y in &years[1..] {
        println!(
            "{y:>5}  {:>22} {:>22} {:>22} {:>22}",
            spread(&decade(y, &|x: &Year| x.up[1])),
            spread(&decade(y, &|x: &Year| x.up[2])),
            spread(&decade(y, &|x: &Year| x.up[3])),
            spread(&decade(y, &|x: &Year| x.down))
        );
    }

    println!("\n## Where roads cross a river: crossings, and the river's width there (m), pooled over islands");
    println!("{:>5} {:>6}  {:>22} {:>8} {:>8} {:>8} {:>8} {:>10}", "year", "level", "crossings an island", "median", "p90", "largest", ">= 20 m", "metres");
    for &y in &years {
        if y % 50 != 0 && y != 1900 && y != 1820 && y != 1870 {
            continue;
        }
        for k in 1..4 {
            let n = get(y, &|x: &Year| x.cross[k].len() as f64);
            let mut all: Vec<f64> = runs.iter().filter_map(|(r, _)| r.iter().find(|x| x.year == y)).flat_map(|x| x.cross[k].clone()).collect();
            let wide = all.iter().filter(|&&w| w >= 20.0).count() as f64 / runs.len() as f64;
            let metres = all.iter().sum::<f64>() / runs.len() as f64;
            let (m, p9, mx) = (quant(&mut all, 0.5), quant(&mut all, 0.9), quant(&mut all, 1.0));
            println!("{y:>5} {:>6}  {:>22} {m:>8.1} {p9:>8.1} {mx:>8.1} {wide:>8.1} {metres:>10.0}", [">=", "track", "road", "turnp."][k], spread(&n));
        }
    }

    println!("\n## Carting paid over each kind of way, as a share of the year's output");
    println!("{:>5}  {:>22} {:>22} {:>22} {:>22} {:>22} {:>22}", "year", "no road", "track", "road", "turnpike", "navigable river", "railway");
    for &y in &years[1..] {
        let share = |k: usize| get(y, &move |x: &Year| x.cart[k] / x.gdp);
        println!(
            "{y:>5}  {:>22} {:>22} {:>22} {:>22} {:>22} {:>22}",
            spread(&share(0)),
            spread(&share(1)),
            spread(&share(2)),
            spread(&share(3)),
            spread(&share(4)),
            spread(&share(5))
        );
    }

    println!("\n## What each step of road saves on the step below, on its own traffic, share of output");
    println!("{:>5}  {:>22} {:>22} {:>22}   {:>22} {:>22}", "year", "track", "road", "turnpike", "turnpike traffic, median", "tolls of 0.05 a cell");
    for &y in &years[1..] {
        let share = |k: usize| get(y, &move |x: &Year| x.saved[k] / x.gdp);
        let med = get(y, &|x: &Year| quant(&mut x.turnpike_traffic.clone(), 0.5));
        let toll = get(y, &|x: &Year| 0.05 * 0.015 * x.turnpike_traffic.iter().sum::<f64>() / x.gdp);
        println!("{y:>5}  {:>22} {:>22} {:>22}   {:>22} {:>22}", spread(&share(1)), spread(&share(2)), spread(&share(3)), spread(&med), spread(&toll));
    }

    println!("\n## Fares: the railway's and the canal's, as a share of output and of the capital laid out");
    println!("{:>5}  {:>22} {:>22} {:>22} {:>22}", "year", "rail fares / output", "rail fares / capital", "canal fares / output", "canal fares / capital");
    for &y in &years[1..] {
        println!(
            "{y:>5}  {:>22} {:>22} {:>22} {:>22}",
            spread(&get(y, &|x: &Year| x.rail_fares / x.gdp)),
            spread(&get(y, &|x: &Year| if x.rail_capital > 0.0 { x.rail_fares / x.rail_capital } else { f64::NAN })),
            spread(&get(y, &|x: &Year| x.canal_fares / x.gdp)),
            spread(&get(y, &|x: &Year| if x.canal_capital > 0.0 { x.canal_fares / x.canal_capital } else { f64::NAN }))
        );
    }

    println!("\n## Purses and people");
    println!("{:>5}  {:>22} {:>22} {:>22} {:>22}", "year", "landlords' cash / output", "households' cash / output", "people", "real wage");
    for &y in &years {
        println!(
            "{y:>5}  {:>22} {:>22} {:>22} {:>22}",
            spread(&get(y, &|x: &Year| x.lords / x.gdp)),
            spread(&get(y, &|x: &Year| x.hh / x.gdp)),
            spread(&get(y, &|x: &Year| x.pop)),
            spread(&get(y, &|x: &Year| x.real_wage))
        );
    }

    if P.roads.enabled {
        println!("\n## Roads paid for (roads.enabled): spent, tolls taken, trusts, menders");
        println!("{:>5}  {:>22} {:>22} {:>22} {:>22}", "year", "spent / output", "tolls / output", "trusts", "menders (hands)");
        for &y in &years[1..] {
            println!(
                "{y:>5}  {:>22} {:>22} {:>22} {:>22}",
                spread(&decade(y, &|x: &Year| x.roads_spent / x.gdp)),
                spread(&decade(y, &|x: &Year| x.tolls / x.gdp)),
                spread(&get(y, &|x: &Year| x.trusts)),
                spread(&decade(y, &|x: &Year| x.menders))
            );
        }
    }

    let ratios: Vec<f64> = runs.iter().map(|r| r.1).filter(|r| r.is_finite()).collect();
    if !ratios.is_empty() {
        println!("\nflow per cell of area upstream on river cells (median per island): {}", spread(&ratios));
    }
    let _ = river_width;
}
