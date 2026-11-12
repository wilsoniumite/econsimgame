//! Prints, for docs/probes/tr-steel.md, when each invention is found on each island and why then (how near its threshold the island stood, which ingredient held it back, and which town drew it), and what a steel rail mill would earn beside an iron rolling mill in the town that would do best by it, year by year, with the books of every rail and steel trade beside it.
//!
//!     cargo run --release -p isostate-core --example steel_trace -- [--seeds 1-8] [--families 6] [--end-year Y]
//!         [--set JSON] [--worth W] [--plant S] [--method NAME.FIELD=V ...] [--disc TECH.GOOD=X ...] [--csv DIR] [--quiet] [--works]
//!
//! The game's rules. `--set` lays a JSON object over the game's parameters
//! (`{"machines":{"inherit":0}}`, `{"discovery_rate":0.5}`). `--worth W` enters a
//! ton of steel rail as W iron rails where the game says 3 (the steel rail
//! mill's steel a rail is the port's over W); `--plant S` gives the converter
//! and the steel rail mill S of the port's plant where the game gives 0.3.
//! `--method` sets one number of one method, after those two: FIELD is `labor`,
//! `K0`, `A0`, `Q0` or `b` (`--method steel_rolling.K0=40`); `--disc` sets one
//! discovery threshold as the content file writes it (`--disc bessemer.steel=1400000`,
//! `--disc railway.machinery=x20`). Both may be given more than once. All four
//! change the content in this program only. `--csv
//! DIR` writes `disc.csv` (a row for every invention and island) and
//! `years.csv` (a row for every island and year) for reading elsewhere;
//! `--quiet` prints only the summary tables; `--works` adds, for each island,
//! the books of the steel rail mill that made the most in its life.
//!
//! The site sums are `firms::evaluate_sites`, the sums entry makes, taken after
//! each year's end on that year's mean prices and wage, for a new works sized
//! as entry would size it before the year's room is spent. Nothing here draws
//! a random number of the run's or changes its state: the runs are the game's.
#![allow(non_snake_case)]
use std::fmt::Write as _;

use isostate_core::accounts::{self, KM, METHOD_FIELDS};
use isostate_core::content::{MACHINES_PLANT_SCALE, MACHINES_STEEL_RAIL_WORTH};
use isostate_core::*;

/// What one invention's finding looked like.
#[derive(Clone, Debug, Default)]
struct Found {
    tech: usize,
    /// Year found, or None if never.
    year: Option<i64>,
    town: Option<usize>,
    /// Progress (the least of cumulative output over threshold) in the year found.
    p_at: f64,
    /// First year progress reached a half and the whole.
    half: Option<i64>,
    whole: Option<i64>,
    /// Sum of the yearly hazards before progress reached the whole: the chance of a fluke.
    early_hazard: f64,
    /// The ingredient that held progress back in the year found (or the last year).
    binding: usize,
    /// The chosen town's share of the draw's weight, the greatest share, and
    /// whether the chosen town was the one with the greatest weight, the most
    /// hands in works, the most people.
    w_share: f64,
    w_top: f64,
    top_w: bool,
    top_ind: bool,
    top_pop: bool,
}

/// A would-be works' sums in one town.
#[derive(Clone, Debug, Default)]
struct Site {
    town: usize,
    K: f64,
    A: f64,
    know: f64,
    prod: f64,
    /// sqrt(K0 / K): the small works' penalty.
    r: f64,
    /// the material's coefficient and its delivered price
    a_mat: f64,
    p_mat: f64,
    pout: f64,
    in_cost: f64,
    heat_cost: f64,
    labor_cost: f64,
    margin: f64,
    /// capital cost a unit of monthly capacity
    cap_unit: f64,
    rent: f64,
    roi: f64,
    need: f64,
}

/// The books of one method over the whole island in one year.
#[derive(Clone, Debug, Default)]
struct MethodYear {
    v: [f64; KM],
}

#[derive(Clone, Debug, Default)]
struct YearRow {
    year: i64,
    /// progress of every tech not known at the start
    p: Vec<f64>,
    methods: Vec<MethodYear>,
    /// island means of the year's prices over active towns, by good
    price: Vec<f64>,
    wage: f64,
    /// a steel rail mill in the town where it would do best, and an iron
    /// rolling mill of the same size in that town
    steel_site: Option<Site>,
    iron_site: Option<Site>,
    /// towns that know the steel rail mill, and how many could found one
    towns_know: usize,
    towns_clear: usize,
    /// quality of machines
    quality: f64,
}

struct Trace {
    seed: u64,
    found: Vec<Found>,
    years: Vec<YearRow>,
    /// distinct towns that found something, and the most any one town found
    disc_towns: usize,
    disc_most: usize,
    /// the books of the steel rail mill that made the most, as lines to print
    mill: String,
}

struct Opts {
    seeds: Vec<u64>,
    families: usize,
    end_year: Option<i64>,
    set: Option<String>,
    worth: f64,
    plant: f64,
    methods: Vec<String>,
    disc: Vec<String>,
    csv: Option<String>,
    quiet: bool,
    works: bool,
}

fn content(o: &Opts) -> Content {
    let mut c = Rules::Game.content();
    let bessemer = c.method_name.iter().position(|n| n == "bessemer").unwrap();
    let steel_rolling = c.method_name.iter().position(|n| n == "steel_rolling").unwrap();
    let steel = c.gi("steel");
    if o.plant != MACHINES_PLANT_SCALE {
        for m in [bessemer, steel_rolling] {
            for g in 0..c.G {
                c.m_cap[(m, g)] *= o.plant / MACHINES_PLANT_SCALE;
            }
        }
    }
    if o.worth != MACHINES_STEEL_RAIL_WORTH {
        c.m_in[(steel_rolling, steel)] *= MACHINES_STEEL_RAIL_WORTH / o.worth;
    }
    for s in &o.methods {
        let (key, v) = s.split_once('=').expect("NAME.FIELD=V");
        let (name, field) = key.split_once('.').expect("NAME.FIELD=V");
        let m = c.method_name.iter().position(|n| n == name).unwrap_or_else(|| panic!("no method {name}"));
        let v: f64 = v.parse().expect("a number");
        match field {
            "labor" => c.m_labor[m] = v,
            "K0" => c.m_K0[m] = v,
            "A0" => c.m_A0[m] = v,
            "Q0" => c.m_Q0[m] = v,
            "b" => c.m_b[m] = v,
            other => panic!("no field {other}: labor, K0, A0, Q0 or b"),
        }
    }
    for s in &o.disc {
        let (key, v) = s.split_once('=').expect("TECH.GOOD=X");
        let (name, good) = key.split_once('.').expect("TECH.GOOD=X");
        let t = c.tech_name.iter().position(|n| n == name).unwrap_or_else(|| panic!("no invention {name}"));
        let g = c.gi(good);
        // as in the content file: a plain number is cumulative output, "x30" thirty years of 1750's
        c.tech_disc[(t, g)] = match v.strip_prefix('x') {
            Some(x) => -x.parse::<f64>().expect("a number"),
            None => v.parse().expect("a number"),
        };
    }
    c
}

fn progress(st: &State, C: &Content, t: usize) -> (f64, usize) {
    let mut best = (f64::INFINITY, 0);
    for g in 0..C.G {
        let x = st.disc_X0[(t, g)];
        if x > 0.0 {
            let r = st.cumprod[g] / x;
            if r < best.0 {
                best = (r, g);
            }
        }
    }
    (best.0.max(0.0), best.1)
}

/// The sums entry would make for a works of method m in town i, on every cell
/// it could stand on, at size K; the best cell's.
fn site(st: &State, C: &Content, P: &Params, i: usize, m: usize, K: f64) -> Option<Site> {
    let cells: Vec<usize> = st.town_cells[i]
        .iter()
        .copied()
        .filter(|&c| st.cell_K_share[c] < 0.9 && st.land_use[c] != 0 && st.land_use[c] != land::USE_DERELICT)
        .collect();
    if cells.is_empty() {
        return None;
    }
    let ev = firms::evaluate_sites(st, C, P, i, m, &cells, K, &st.pbar, &st.wbar);
    let j = util::argmax(&ev.roi);
    let mat = (0..C.G).max_by(|&a, &b| C.m_in[(m, a)].total_cmp(&C.m_in[(m, b)])).unwrap();
    let A = st.method_A[m];
    let know = st.know[(i, m)];
    let agglo = ev.agglo[j];
    Some(Site {
        town: i,
        K,
        A,
        know,
        prod: A * (0.5 + 0.5 * know) * agglo,
        r: (C.m_K0[m] / K.max(1e-9)).sqrt().min(10.0),
        a_mat: ev.a[(j, mat)],
        p_mat: ev.pin[(j, mat)],
        pout: ev.pout[j],
        in_cost: ev.in_cost[j],
        heat_cost: ev.heat_cost[j],
        labor_cost: ev.labor_cost[j],
        margin: ev.margin[j],
        cap_unit: ev.cap_cost / K,
        rent: ev.rent_bid[j],
        roi: ev.roi[j],
        need: P.hurdle,
    })
}

/// A new works' size as entry sets it before the year's room is spent.
fn entry_size(st: &State, C: &Content, P: &Params, i: usize, m: usize) -> f64 {
    let base = P.entry_K_frac * C.m_K0[m];
    (P.works.entry_share * works::town_trade(st, C, i, m)).min(works::max_K(C, P, m)).max(base)
}

fn trace(seed: u64, P: &Params, C: Content) -> Trace {
    let mut P = P.clone();
    P.seed = seed;
    let mut s = sim::start(&P, C);
    let C = s.content.clone();
    let unknown: Vec<usize> = (0..C.NT).filter(|&t| !C.tech_known[t]).collect();
    let mut found: Vec<Found> = unknown.iter().map(|&t| Found { tech: t, ..Default::default() }).collect();
    let steel_rolling = C.method_name.iter().position(|n| n == "steel_rolling").unwrap();
    let rolling_mill = C.method_name.iter().position(|n| n == "rolling_mill").unwrap();
    let mut years = Vec::new();
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved every month");
        let year = s.year - 1;
        let st = &s.st;
        let P = &s.params;
        let act: Vec<usize> = (0..st.T_max).filter(|&i| st.town_active[i]).collect();
        let mut row = YearRow { year, ..Default::default() };
        for f in found.iter_mut() {
            let t = f.tech;
            let (p, g) = progress(st, &C, t);
            row.p.push(p);
            if f.half.is_none() && p >= 0.5 {
                f.half = Some(year);
            }
            if f.whole.is_none() && p >= 1.0 {
                f.whole = Some(year);
            }
            if f.year.is_none() {
                f.binding = g;
                if f.whole.is_none() || f.whole == Some(year) {
                    // the hazard the draw of this year was made at, before the threshold
                    let p4 = p.powi(4);
                    if p < 1.0 {
                        f.early_hazard += P.discovery_rate * p4 / (1.0 + p4);
                    }
                }
            }
            if f.year.is_none() && st.tech_year[t] == year {
                f.year = Some(year);
                f.p_at = p;
                let town = st.tech_town[t] as usize;
                f.town = Some(town);
                // the draw's weights, as tech::discovery makes them
                let req: Vec<usize> = (0..C.G).filter(|&g| st.disc_X0[(t, g)] > 0.0).collect();
                let w: Vec<f64> = act
                    .iter()
                    .map(|&i| {
                        let score: f64 = req.iter().map(|&g| st.town_prod_year[(i, g)] / st.disc_X0[(t, g)]).sum();
                        let v = score + 1e-9 * st.pop[i];
                        v * v
                    })
                    .collect();
                let total: f64 = w.iter().sum();
                let k = act.iter().position(|&i| i == town).unwrap_or(0);
                f.w_share = w[k] / total;
                f.w_top = w.iter().cloned().fold(0.0, f64::max) / total;
                let top = |v: &dyn Fn(usize) -> f64| act.iter().copied().max_by(|&a, &b| v(a).total_cmp(&v(b))).unwrap();
                f.top_w = act[util::argmax(&w)] == town;
                f.top_ind = top(&|i| st.ind_labor[i]) == town;
                f.top_pop = top(&|i| st.pop[i]) == town;
            }
        }
        // the books of every method, summed over the towns
        let Some(rec) = st.acct.year(st.acct.years.len() - 1) else { panic!("the books are kept") };
        for m in 0..C.M {
            let mut my = MethodYear::default();
            for i in 0..st.T_max {
                let base = (i * C.M + m) * KM;
                for k in 0..KM {
                    my.v[k] += rec.methods[base + k] as f64;
                }
            }
            row.methods.push(my);
        }
        let n = act.len().max(1) as f64;
        row.price = (0..C.G).map(|g| act.iter().map(|&i| st.pbar[(i, g)]).sum::<f64>() / n).collect();
        row.wage = act.iter().map(|&i| st.wbar[i]).sum::<f64>() / n;
        row.quality = machines::quality(st, &C, P);
        // a steel rail mill where it would do best
        if st.tech_year[steel_rolling] >= 0 {
            let mut best: Option<Site> = None;
            for &i in &act {
                if st.know[(i, steel_rolling)] <= 0.02 {
                    continue;
                }
                row.towns_know += 1;
                let K = entry_size(st, &C, P, i, steel_rolling);
                if let Some(x) = site(st, &C, P, i, steel_rolling, K) {
                    if x.roi > x.need {
                        row.towns_clear += 1;
                    }
                    if best.as_ref().is_none_or(|b| x.roi > b.roi) {
                        best = Some(x);
                    }
                }
            }
            if let Some(b) = &best {
                row.iron_site = site(st, &C, P, b.town, rolling_mill, b.K);
            }
            row.steel_site = best;
        }
        years.push(row);
    }
    let mut per_town = std::collections::BTreeMap::new();
    for f in &found {
        if let Some(t) = f.town {
            *per_town.entry(t).or_insert(0usize) += 1;
        }
    }
    // the steel rail mill that made the most in its life, year by year
    let st = &s.st;
    let mut mill = String::new();
    let biggest = (0..st.F_count)
        .filter(|&f| st.f_method[f] == steel_rolling)
        .filter_map(|f| accounts::works_books(st, &C, f))
        .max_by(|a, b| a.years.iter().map(|y| y.out).sum::<f64>().total_cmp(&b.years.iter().map(|y| y.out).sum::<f64>()));
    let mills = (0..st.F_count).filter(|&f| st.f_method[f] == steel_rolling).count();
    match biggest {
        Some(wb) => {
            let f = wb.firm;
            writeln!(mill, "seed {seed}: {mills} steel rail mills in the run; the one that made most, {} at {} (founded {}, {}), year by year", wb.name, st.town_name[st.f_town[f]], st.f_born[f], if st.f_died[f] >= 0 { format!("closed {}", st.f_died[f]) } else { "standing in the end".into() }).unwrap();
            writeln!(mill, "  year  owner  capacity      out   util  revenue  inputs    fuel   hands    wages    rent   profit   profit/plant   cash").unwrap();
            for y in &wb.years {
                let plant = y.capacity * (0..C.G).map(|g| C.m_cap[(steel_rolling, g)] * s.st.pbar[(st.f_town[f], g)]).sum::<f64>();
                writeln!(
                    mill,
                    "  {}  {:>5}  {:8.1} {:8.1}  {:5.2} {:8.0} {:7.0} {:7.0} {:7.1} {:8.0} {:7.0} {:8.0}   {:12.3} {:6.0}",
                    y.year,
                    y.owner.map_or("-".to_string(), |o| o.to_string()),
                    y.capacity,
                    y.out,
                    y.util,
                    y.revenue,
                    y.inputs,
                    y.fuel,
                    y.hands,
                    y.wages,
                    y.rent,
                    y.profit,
                    y.profit / plant.max(1e-9),
                    y.cash
                )
                .unwrap();
            }
        }
        None => writeln!(mill, "seed {seed}: no steel rail mill in the run").unwrap(),
    }
    Trace { seed, found, years, disc_towns: per_town.len(), disc_most: per_town.values().copied().max().unwrap_or(0), mill }
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
}

fn range(v: &[f64]) -> String {
    if v.is_empty() {
        return "-".into();
    }
    let lo = v.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let m = mean(v);
    let sd = (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64).sqrt();
    format!("{:.0} [{:.0} .. {:.0}] sd {:.1}", m, lo, hi, sd)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut o = Opts { seeds: (1..=8).collect(), families: 6, end_year: None, set: None, worth: MACHINES_STEEL_RAIL_WORTH, plant: MACHINES_PLANT_SCALE, methods: Vec::new(), disc: Vec::new(), csv: None, quiet: false, works: false };
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => o.seeds = sweep::parse_seeds(&next()).unwrap(),
            "--families" => o.families = next().parse().unwrap(),
            "--end-year" => o.end_year = Some(next().parse().unwrap()),
            "--set" => o.set = Some(next()),
            "--worth" => o.worth = next().parse().unwrap(),
            "--plant" => o.plant = next().parse().unwrap(),
            "--method" => o.methods.push(next()),
            "--disc" => o.disc.push(next()),
            "--csv" => o.csv = Some(next()),
            "--quiet" | "--works" => {
                if args[i] == "--quiet" {
                    o.quiet = true;
                } else {
                    o.works = true;
                }
                i += 1;
                continue;
            }
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let mut P = Rules::Game.params();
    P.n_families = o.families;
    P.end_year = o.end_year;
    P.accounts.enabled = true;
    if let Some(j) = &o.set {
        P = P.with_json_overrides(j).expect("a JSON object of parameters");
    }
    let C = content(&o);
    let traces: Vec<Trace> = std::thread::scope(|s| {
        let hs: Vec<_> = o.seeds.iter().map(|&seed| {
            let (P, C) = (P.clone(), C.clone());
            s.spawn(move || trace(seed, &P, C))
        }).collect();
        hs.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });
    let C = &C;
    let mi = |name: &str| C.method_name.iter().position(|n| n == name).unwrap();
    let col = |name: &str| METHOD_FIELDS.iter().position(|&n| n == name).unwrap();
    println!(
        "the game's rules, {} families, seeds {:?}{}; a ton of steel rail is {} iron rails, the converter's and the steel rail mill's plant {} of the port's{}",
        o.families,
        o.seeds,
        o.set.as_ref().map_or(String::new(), |s| format!(", with {s}")),
        o.worth,
        o.plant,
        if o.methods.is_empty() && o.disc.is_empty() { String::new() } else { format!("; {}", o.methods.iter().chain(&o.disc).cloned().collect::<Vec<_>>().join(", ")) }
    );

    // ---- 1. when each invention came
    println!("\n1. When each invention came, mean [least .. greatest] and standard deviation over the islands");
    println!("   found: the year; whole: the year cumulative output first reached every threshold (progress 1); found - whole: years from that to the find");
    println!("   at find: progress in the year found; early: islands found before progress reached a half; fluke odds: the chance, summed over the years before the threshold, that the draw came early");
    println!("   town: the share of finds in the town the draw weighted most, in the town with the most hands in works, in the most populous; the drawn town's weight, mean");
    println!("{:>20} {:>32} {:>26} {:>20} {:>9} {:>6} {:>10} {:>18} {:>7}  {}", "", "found", "whole", "found - whole", "at find", "early", "fluke odds", "top w / ind / pop", "weight", "held back by (islands)");
    for (k, f0) in traces[0].found.iter().enumerate() {
        let t = f0.tech;
        let fs: Vec<&Found> = traces.iter().map(|tr| &tr.found[k]).collect();
        let yrs: Vec<f64> = fs.iter().filter_map(|f| f.year.map(|y| y as f64)).collect();
        let whole: Vec<f64> = fs.iter().filter_map(|f| f.whole.map(|y| y as f64)).collect();
        let lag: Vec<f64> = fs.iter().filter_map(|f| Some((f.year? - f.whole?) as f64)).collect();
        let at: Vec<f64> = fs.iter().filter(|f| f.year.is_some()).map(|f| f.p_at).collect();
        let early = fs.iter().filter(|f| f.year.is_some() && f.p_at < 0.5).count();
        let odds = mean(&fs.iter().map(|f| 1.0 - (-f.early_hazard).exp()).collect::<Vec<_>>());
        let n = fs.iter().filter(|f| f.year.is_some()).count().max(1) as f64;
        let share = |b: &dyn Fn(&Found) -> bool| fs.iter().filter(|f| f.year.is_some() && b(f)).count() as f64 / n;
        let mut held: std::collections::BTreeMap<String, usize> = Default::default();
        for f in &fs {
            *held.entry(C.good_name[f.binding].clone()).or_default() += 1;
        }
        let held: Vec<String> = held.into_iter().map(|(g, c)| format!("{g} {c}")).collect();
        println!(
            "{:>20} {:>32} {:>26} {:>20} {:>9.2} {:>6} {:>10.3} {:>18} {:>7.2}  {}",
            C.tech_name[t],
            format!("{} ({}/{})", range(&yrs), yrs.len(), fs.len()),
            range(&whole),
            range(&lag),
            mean(&at),
            early,
            odds,
            format!("{:.2} / {:.2} / {:.2}", share(&|f| f.top_w), share(&|f| f.top_ind), share(&|f| f.top_pop)),
            mean(&fs.iter().filter(|f| f.year.is_some()).map(|f| f.w_share).collect::<Vec<_>>()),
            held.join(", ")
        );
    }
    println!("   towns that found something on an island: {}; the most any one town found: {}", range(&traces.iter().map(|t| t.disc_towns as f64).collect::<Vec<_>>()), range(&traces.iter().map(|t| t.disc_most as f64).collect::<Vec<_>>()));

    if !o.quiet {
        println!("\n   island by island: the year found (progress then; the year progress reached the whole)");
        print!("{:>6}", "seed");
        for f in &traces[0].found {
            print!(" {:>19}", C.tech_name[f.tech]);
        }
        println!();
        for tr in &traces {
            print!("{:>6}", tr.seed);
            for f in &tr.found {
                let s = match f.year {
                    Some(y) => format!("{y} ({:.2}; {})", f.p_at, f.whole.map_or("-".into(), |w| w.to_string())),
                    None => format!("- ({})", f.whole.map_or("-".into(), |w| w.to_string())),
                };
                print!(" {s:>19}");
            }
            println!();
        }
    }

    // ---- 2. rails and steel, from the books
    let (forge, roll, steel_roll, crucible, bessemer) = (mi("forge_rails"), mi("rolling_mill"), mi("steel_rolling"), mi("crucible_steel"), mi("bessemer"));
    let (g_steel, g_iron, g_rails) = (C.gi("steel"), C.gi("iron"), C.gi("rails"));
    println!("\n2. Rails and steel by method, from the books (all islands' towns summed, then the ratio)");
    println!("   share: of the good made; util: made over twelve months' capacity; profit/value: the works' profit over the value of what they made; plant: capacity at the year's mean prices of the capital goods (port's content scaled as run); profit/plant: the year's return on it");
    println!("{:>6} {:>14} {:>7} {:>6} {:>13} {:>13}   {:>14} {:>7} {:>6} {:>13} {:>13}   {:>9} {:>9}  {:>8} {:>8}", "year", "steel rails", "share", "util", "profit/value", "profit/plant", "iron rolled", "share", "util", "profit/value", "profit/plant", "converter", "crucible", "p steel", "p iron");
    let first = traces[0].years[0].year;
    let last = traces[0].years.last().unwrap().year;
    let mut y = 1830;
    while y <= last {
        let k = (y - first) as usize;
        let sum = |m: usize, c: usize| traces.iter().map(|t| t.years[k].methods[m].v[c]).sum::<f64>();
        let plant = |m: usize| traces.iter().map(|t| {
            let r = &t.years[k];
            r.methods[m].v[col("capacity")] * (0..C.G).map(|g| C.m_cap[(m, g)] * r.price[g]).sum::<f64>()
        }).sum::<f64>();
        let rails: f64 = [forge, roll, steel_roll].iter().map(|&m| sum(m, col("out"))).sum();
        let steel: f64 = [crucible, bessemer].iter().map(|&m| sum(m, col("out"))).sum();
        let cells = |m: usize, tot: f64| {
            let out = sum(m, col("out"));
            let cap = sum(m, col("capacity"));
            format!(
                "{:>14.0} {:>7.3} {:>6.2} {:>13.3} {:>13.3}",
                out / traces.len() as f64,
                out / tot.max(1e-9),
                out / (12.0 * cap).max(1e-9),
                sum(m, col("profit")) / sum(m, col("value")).max(1e-9),
                sum(m, col("profit")) / plant(m).max(1e-9)
            )
        };
        let pm = |g: usize| mean(&traces.iter().map(|t| t.years[k].price[g]).collect::<Vec<_>>());
        println!(
            "{y:>6} {}   {}   {:>9.3} {:>9.3}  {:>8.2} {:>8.2}",
            cells(steel_roll, rails),
            cells(roll, rails),
            sum(bessemer, col("out")) / steel.max(1e-9),
            sum(crucible, col("out")) / steel.max(1e-9),
            pm(g_steel),
            pm(g_iron)
        );
        y += 5;
    }

    // ---- 3. a steel rail mill's sums
    println!("\n3. A new steel rail mill in the town where it would do best, and an iron rolling mill of the same size there (means over the islands that know it)");
    println!("   K: size a month; A: the method's skill; prod: with the town's know-how and neighbours; r: sqrt(K0/K), the small works' penalty;");
    println!("   a x p: the material a rail and its delivered price; costs a rail: material, heat, hands; pout: the price less carting and the spread; roi against the hurdle; clear: towns where it clears / towns that know it");
    println!("{:>6} {:>4} | {:>6} {:>5} {:>5} {:>5} {:>14} {:>6} {:>6} {:>6} {:>6} {:>7} {:>8} {:>7} | {:>5} {:>5} {:>14} {:>6} {:>6} {:>6} {:>7} {:>8} {:>7} {:>9}", "year", "n", "K", "A", "prod", "r", "steel a x p", "mat", "heat", "hands", "pout", "margin", "plant/K", "roi", "A", "prod", "iron a x p", "mat", "heat", "hands", "margin", "plant/K", "roi", "clear");
    let mut y = 1820;
    while y <= last {
        let k = (y - first) as usize;
        let rows: Vec<(&Site, &Site, usize, usize)> = traces
            .iter()
            .filter_map(|t| {
                let r = &t.years[k];
                Some((r.steel_site.as_ref()?, r.iron_site.as_ref()?, r.towns_clear, r.towns_know))
            })
            .collect();
        if !rows.is_empty() {
            let m = |f: &dyn Fn(&Site) -> f64, iron: bool| mean(&rows.iter().map(|r| f(if iron { r.1 } else { r.0 })).collect::<Vec<_>>());
            println!(
                "{y:>6} {:>4} | {:>6.1} {:>5.2} {:>5.2} {:>5.2} {:>14} {:>6.2} {:>6.2} {:>6.2} {:>6.2} {:>7.2} {:>8.1} {:>7.3} | {:>5.2} {:>5.2} {:>14} {:>6.2} {:>6.2} {:>6.2} {:>7.2} {:>8.1} {:>7.3} {:>9}",
                rows.len(),
                m(&|s| s.K, false),
                m(&|s| s.A, false),
                m(&|s| s.prod, false),
                m(&|s| s.r, false),
                format!("{:.3} x {:.2}", m(&|s| s.a_mat, false), m(&|s| s.p_mat, false)),
                m(&|s| s.in_cost, false),
                m(&|s| s.heat_cost, false),
                m(&|s| s.labor_cost, false),
                m(&|s| s.pout, false),
                m(&|s| s.margin, false),
                m(&|s| s.cap_unit, false),
                m(&|s| s.roi, false),
                m(&|s| s.A, true),
                m(&|s| s.prod, true),
                format!("{:.3} x {:.2}", m(&|s| s.a_mat, true), m(&|s| s.p_mat, true)),
                m(&|s| s.in_cost, true),
                m(&|s| s.heat_cost, true),
                m(&|s| s.labor_cost, true),
                m(&|s| s.margin, true),
                m(&|s| s.cap_unit, true),
                m(&|s| s.roi, true),
                format!("{}/{}", rows.iter().map(|r| r.2).sum::<usize>(), rows.iter().map(|r| r.3).sum::<usize>())
            );
        }
        y += 5;
    }
    println!("   (the hurdle is {:.2}; a town with as many works of the method as 2 + people/40,000 asks three times it)", P.hurdle);
    if o.works {
        println!("\n4. The steel rail mill that made the most on each island: its books (profit/plant: plant at the last year's prices in its town)");
        for tr in &traces {
            print!("{}", tr.mill);
        }
    }

    // ---- the CSVs
    if let Some(dir) = &o.csv {
        std::fs::create_dir_all(dir).unwrap();
        let mut d = String::from("seed,tech,year,town,p_at,half,whole,early_hazard,binding,w_share,w_top,top_w,top_ind,top_pop\n");
        for tr in &traces {
            for f in &tr.found {
                let opt = |v: Option<i64>| v.map_or(String::new(), |y| y.to_string());
                writeln!(
                    d,
                    "{},{},{},{},{:.4},{},{},{:.5},{},{:.4},{:.4},{},{},{}",
                    tr.seed,
                    C.tech_name[f.tech],
                    opt(f.year),
                    f.town.map_or(String::new(), |t| t.to_string()),
                    f.p_at,
                    opt(f.half),
                    opt(f.whole),
                    f.early_hazard,
                    C.good_name[f.binding],
                    f.w_share,
                    f.w_top,
                    f.top_w as u8,
                    f.top_ind as u8,
                    f.top_pop as u8
                )
                .unwrap();
            }
        }
        std::fs::write(format!("{dir}/disc.csv"), d).unwrap();
        let mut h = String::from("seed,year,quality,wage,p_steel,p_iron,p_rails,p_machinery");
        for f in &traces[0].found {
            write!(h, ",p_{}", C.tech_name[f.tech]).unwrap();
        }
        for m in [forge, roll, steel_roll, crucible, bessemer] {
            for c in ["out", "value", "capacity", "works", "hands", "wages", "inputs", "fuel", "profit"] {
                write!(h, ",{}_{}", C.method_name[m], c).unwrap();
            }
        }
        for side in ["s", "i"] {
            for c in ["town", "K", "A", "know", "prod", "r", "a_mat", "p_mat", "pout", "in_cost", "heat_cost", "labor_cost", "margin", "cap_unit", "rent", "roi"] {
                write!(h, ",{side}_{c}").unwrap();
            }
        }
        h.push_str(",towns_know,towns_clear\n");
        let g_mach = C.gi("machinery");
        for tr in &traces {
            for r in &tr.years {
                write!(h, "{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}", tr.seed, r.year, r.quality, r.wage, r.price[g_steel], r.price[g_iron], r.price[g_rails], r.price[g_mach]).unwrap();
                for p in &r.p {
                    write!(h, ",{p:.5}").unwrap();
                }
                for m in [forge, roll, steel_roll, crucible, bessemer] {
                    for c in ["out", "value", "capacity", "works", "hands", "wages", "inputs", "fuel", "profit"] {
                        write!(h, ",{:.3}", r.methods[m].v[col(c)]).unwrap();
                    }
                }
                for s in [&r.steel_site, &r.iron_site] {
                    match s {
                        Some(s) => {
                            for v in [s.town as f64, s.K, s.A, s.know, s.prod, s.r, s.a_mat, s.p_mat, s.pout, s.in_cost, s.heat_cost, s.labor_cost, s.margin, s.cap_unit, s.rent, s.roi] {
                                write!(h, ",{v:.5}").unwrap();
                            }
                        }
                        None => h.push_str(&",".repeat(16)),
                    }
                }
                writeln!(h, ",{},{}", r.towns_know, r.towns_clear).unwrap();
            }
        }
        std::fs::write(format!("{dir}/years.csv"), h).unwrap();
        println!("\nwrote {dir}/disc.csv and {dir}/years.csv");
    }
    let _ = accounts::KG;
}
