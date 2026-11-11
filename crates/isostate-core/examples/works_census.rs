//! A census of the works, for reading what the life cycle for works does
//! (`docs/rules/works.md`): who stands where, how large, and how they came and went.
//!
//!     cargo run -p isostate-core --release --example works_census -- [--rules port|game]
//!         [--families N] [--set JSON] [--decades] [SEED ...]
//!
//! `--set` takes a JSON object of parameter overrides, laid over the chosen
//! rules as a `--params` file is: `--set '{"works":{"enabled":true,"walk":0}}'`.
//! Seeds default to 3 and 7; each runs on its own thread and the tables are the
//! means over seeds.
//!
//! For 1750, 1800, 1850 and 1900 (every 25 years with `--decades`, which also
//! prints output per head, the real wage and the flows decade by decade), one
//! row per method that has works standing:
//!
//! - works, and how many of them are sister works and family works;
//! - size as built capacity over the method's usual size: median, 90th
//!   percentile, largest;
//! - hands, the share of them at works at their cell's limit, and utilisation;
//! - where: the hands' mean path cost to their market, the share of hands
//!   within 2 of it, and the share on town ground.
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use isostate_core::params::Rules;
use isostate_core::*;

#[derive(Clone, Default)]
struct Row {
    n: f64,
    sisters: f64,
    family: f64,
    med: f64,
    p90: f64,
    max: f64,
    hands: f64,
    at_limit: f64,
    util: f64,
    dist: f64,
    near: f64,
    town: f64,
}

#[derive(Clone, Default)]
struct Decade {
    years: f64,
    works: f64,
    gdp_pc: f64,
    real_wage: f64,
    new: f64,
    sister: f64,
    family: f64,
    closed: f64,
    closed_big: f64,
    industry_cells: f64,
    hands_at_limit: f64,
    capacity_idle: f64,
}

struct Census {
    marks: BTreeMap<i64, Vec<Row>>,
    decades: BTreeMap<i64, Decade>,
    life: Vec<f64>,
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let pos = q * (sorted.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
}

fn rows(st: &State, C: &Content, P: &Params) -> Vec<Row> {
    (0..C.M)
        .map(|m| {
            let ids: Vec<usize> = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_method[f] == m).collect();
            let mut size: Vec<f64> = ids.iter().filter(|&&f| st.f_K[f] > 0.0).map(|&f| st.f_K[f] / C.m_K0[m]).collect();
            size.sort_by(f64::total_cmp);
            let hands: f64 = ids.iter().map(|&f| st.f_labor[f]).sum();
            let limit = works::max_K(C, P, m);
            let sum = |w: &dyn Fn(usize) -> f64| ids.iter().map(|&f| st.f_labor[f] * w(f)).sum::<f64>() / hands.max(1e-9);
            let cap: f64 = ids.iter().map(|&f| st.f_K[f]).sum();
            Row {
                n: ids.len() as f64,
                sisters: ids.iter().filter(|&&f| st.f_parent[f] >= 0).count() as f64,
                family: ids.iter().filter(|&&f| st.f_owner.get(f).is_some_and(|&o| o >= 0)).count() as f64,
                med: quantile(&size, 0.5),
                p90: quantile(&size, 0.9),
                max: quantile(&size, 1.0),
                hands,
                at_limit: sum(&|f| if st.f_K[f] >= 0.9 * limit { 1.0 } else { 0.0 }),
                util: ids.iter().map(|&f| st.f_K[f] * st.f_u[f]).sum::<f64>() / cap.max(1e-9),
                dist: sum(&|f| st.dcatch[st.f_cell[f]]),
                near: sum(&|f| if st.dcatch[st.f_cell[f]] <= 2.0 { 1.0 } else { 0.0 }),
                town: sum(&|f| if st.urban[st.f_cell[f]] { 1.0 } else { 0.0 }),
            }
        })
        .collect()
}

fn census(P: &Params, C: Content, step: i64) -> Census {
    let mut s = sim::start(P, C);
    let C = s.content.clone();
    let y0 = C.start_year;
    let mut out = Census { marks: BTreeMap::new(), decades: BTreeMap::new(), life: Vec::new() };
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved");
        let y = s.year - 1;
        let st = &s.st;
        let d = out.decades.entry(y0 + (y - y0) / 10 * 10).or_default();
        for &f in &st.new_firms_year {
            match st.f_reason[f].kind.as_str() {
                "expansion" => d.sister += 1.0,
                "family" => d.family += 1.0,
                _ => d.new += 1.0,
            }
        }
        for &f in &st.closed_year {
            d.closed += 1.0;
            if st.f_K[f] > 1.5 * C.m_K0[st.f_method[f]] {
                d.closed_big += 1.0;
            }
            out.life.push((y - st.f_born[f]) as f64);
        }
        let k = (y - y0) as usize;
        let series = |name: &str| -> f64 {
            let v = &st.series.data[st.series.index[name]];
            v[k * 12..k * 12 + 12].iter().sum::<f64>() / 12.0
        };
        // summed here, divided by the decade's years when printed
        d.years += 1.0;
        d.works += (0..st.F_count).filter(|&f| st.f_alive[f]).count() as f64;
        d.gdp_pc += series("gdp_pc");
        d.real_wage += series("real_wage");
        d.industry_cells += st.land_use.iter().filter(|&&u| u == land::USE_INDUSTRY).count() as f64;
        let (mut hands, mut limit, mut cap, mut used) = (0.0, 0.0, 0.0, 0.0);
        for f in (0..st.F_count).filter(|&f| st.f_alive[f]) {
            hands += st.f_labor[f];
            if st.f_K[f] >= 0.9 * works::max_K(&C, P, st.f_method[f]) {
                limit += st.f_labor[f];
            }
            cap += st.f_K[f];
            used += st.f_K[f] * st.f_u[f];
        }
        d.hands_at_limit += limit / hands.max(1e-9);
        d.capacity_idle += 1.0 - used / cap.max(1e-9);
        if (y - y0) % step == 0 || y == s.end_year {
            out.marks.insert(y, rows(st, &C, P));
        }
    }
    out
}

fn main() {
    let mut rules = Rules::Port;
    let mut families = 0usize;
    let mut set: Option<String> = None;
    let mut decades = false;
    let mut seeds: Vec<u64> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--rules" => rules = Rules::parse(&args.next().expect("--rules port|game")).expect("--rules port|game"),
            "--families" => families = args.next().and_then(|v| v.parse().ok()).expect("--families N"),
            "--set" => set = Some(args.next().expect("--set JSON")),
            "--decades" => decades = true,
            _ => seeds.push(a.parse().unwrap_or_else(|_| panic!("{a} is not a seed"))),
        }
    }
    if seeds.is_empty() {
        seeds = vec![3, 7];
    }
    let mut P = rules.params();
    if let Some(text) = &set {
        P = P.with_json_overrides(text).unwrap_or_else(|e| panic!("{e}"));
    }
    P.n_families = families;
    let C = rules.content();
    let step = if decades { 25 } else { 50 };
    let all: Vec<Census> = std::thread::scope(|sc| {
        let handles: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let (mut P, C) = (P.clone(), C.clone());
                P.seed = seed;
                sc.spawn(move || census(&P, C, step))
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });
    let n = all.len() as f64;
    println!("{} rules, {} families, seeds {:?}{}", rules.as_str(), families, seeds, set.as_ref().map_or(String::new(), |s| format!(", {s}")));
    for &y in all[0].marks.keys() {
        println!("\n{y}");
        println!(
            "{:<18}{:>7}{:>8}{:>7}{:>8}{:>7}{:>7}{:>9}{:>9}{:>6}{:>7}{:>6}{:>6}",
            "method", "works", "sisters", "family", "median", "p90", "max", "hands", "at limit", "util", "dist", "near", "town"
        );
        let mut total = Row::default();
        for m in 0..C.M {
            // the mean over seeds; sizes and places only over the seeds that have such works
            let rs: Vec<&Row> = all.iter().filter_map(|c| c.marks.get(&y)).map(|r| &r[m]).collect();
            let have: Vec<&&Row> = rs.iter().filter(|r| r.n > 0.0).collect();
            if have.is_empty() {
                continue;
            }
            let all_mean = |f: &dyn Fn(&Row) -> f64| rs.iter().map(|r| f(r)).sum::<f64>() / n;
            let some = |f: &dyn Fn(&Row) -> f64| have.iter().map(|r| f(r)).sum::<f64>() / have.len() as f64;
            let r = Row {
                n: all_mean(&|r| r.n),
                sisters: all_mean(&|r| r.sisters),
                family: all_mean(&|r| r.family),
                med: some(&|r| r.med),
                p90: some(&|r| r.p90),
                max: some(&|r| r.max),
                hands: all_mean(&|r| r.hands),
                at_limit: some(&|r| r.at_limit),
                util: some(&|r| r.util),
                dist: some(&|r| r.dist),
                near: some(&|r| r.near),
                town: some(&|r| r.town),
            };
            println!(
                "{:<18}{:>7.0}{:>8.0}{:>7.0}{:>8.2}{:>7.1}{:>7.1}{:>9.0}{:>9.2}{:>6.2}{:>7.1}{:>6.2}{:>6.2}",
                C.method_name[m], r.n, r.sisters, r.family, r.med, r.p90, r.max, r.hands, r.at_limit, r.util, r.dist, r.near, r.town
            );
            total.n += r.n;
            total.sisters += r.sisters;
            total.family += r.family;
            total.hands += r.hands;
            total.at_limit += r.at_limit * r.hands;
            total.dist += r.dist * r.hands;
            total.near += r.near * r.hands;
            total.town += r.town * r.hands;
        }
        let h = total.hands.max(1e-9);
        println!(
            "{:<18}{:>7.0}{:>8.0}{:>7.0}{:>8}{:>7}{:>7}{:>9.0}{:>9.2}{:>6}{:>7.1}{:>6.2}{:>6.2}",
            "all", total.n, total.sisters, total.family, "", "", "", total.hands, total.at_limit / h, "", total.dist / h, total.near / h, total.town / h
        );
    }
    let mut life: Vec<f64> = all.iter().flat_map(|c| c.life.iter().copied()).collect();
    life.sort_by(f64::total_cmp);
    println!("\nworks closed in a run: {:.0}, their median life {:.1} years", life.len() as f64 / n, quantile(&life, 0.5));
    if decades {
        println!(
            "\n{:<8}{:>7}{:>9}{:>8}{:>6}{:>8}{:>8}{:>8}{:>6}{:>9}{:>10}{:>7}",
            "decade", "works", "out/head", "r.wage", "new", "sisters", "family", "closed", "big", "ind.cell", "at limit", "idle"
        );
        for &y in all[0].decades.keys() {
            let m = |f: &dyn Fn(&Decade) -> f64| all.iter().filter_map(|c| c.decades.get(&y)).map(|d| f(d)).sum::<f64>() / n;
            println!(
                "{:<8}{:>7.0}{:>9.2}{:>8.2}{:>6.0}{:>8.0}{:>8.0}{:>8.0}{:>6.0}{:>9.0}{:>10.2}{:>7.2}",
                y,
                m(&|d| d.works / d.years),
                m(&|d| d.gdp_pc / d.years),
                m(&|d| d.real_wage / d.years),
                m(&|d| d.new),
                m(&|d| d.sister),
                m(&|d| d.family),
                m(&|d| d.closed),
                m(&|d| d.closed_big),
                m(&|d| d.industry_cells / d.years),
                m(&|d| d.hands_at_limit / d.years),
                m(&|d| d.capacity_idle / d.years),
            );
        }
    }
}
