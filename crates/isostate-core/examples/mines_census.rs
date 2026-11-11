//! A census of the quarries, pits and mines, for reading what the mines
//! mechanism does (`docs/rules/mines.md`): how many workings there are, how much
//! of the surface they take, how many hands they have and where they stand.
//!
//!     cargo run -p isostate-core --release --example mines_census -- [--rules port|game]
//!         [--families N] [--set JSON] [--map SECTOR] [SEED ...]
//!
//! `--set` takes a JSON object of parameter overrides, laid over the chosen
//! rules as a `--params` file is: `--set '{"mines":{"enabled":true}}'`.
//! Seeds default to 3 and 7; each runs on its own thread and the tables are the
//! means over seeds.
//!
//! For 1750, 1800, 1850 and 1900, one row per extractive sector:
//!
//! - workings: the cells where the sector has at least one hand, and how many
//!   of them hold nine tenths of its hands;
//! - surface: the land the sector takes, in cells (a cell is 2 km square), and
//!   the largest share any one working takes of its cell;
//! - hands, and the hands of the median and the largest working;
//! - output a month, the price (the country's mean), and what the workings
//!   paid their landowners a month (the royalty: what is left of the sale
//!   after the merchant, the carting and the wages);
//! - where: the hands' mean path cost to their market, their mean height, and
//!   beside it the mean height of the whole deposit (weighted by richness), so
//!   that "at the foot of the hill" can be read off: hands below the deposit's
//!   mean stand on its lower edge.
//!
//! Under it, the land in each use, the share of all hands in each sector, farm
//! output and the real wage. `--map SECTOR` prints the last seed's island in
//! 1900 with that sector's workings on it (digits: tenths of the largest
//! working's hands; `.` deposit not worked; `^` high ground; `T` a town).
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use isostate_core::params::Rules;
use isostate_core::*;

#[derive(Clone, Default)]
struct Row {
    workings: f64,
    core: f64,
    surface: f64,
    max_share: f64,
    hands: f64,
    med: f64,
    max: f64,
    out: f64,
    price: f64,
    royalty: f64,
    dist: f64,
    height: f64,
    deposit_height: f64,
}

#[derive(Clone, Default)]
struct Mark {
    rows: Vec<Row>,
    /// Land by use, in cells: the land sectors in order, then wild.
    area: Vec<f64>,
    /// Hands in each land sector as a share of everyone at work.
    hands_share: Vec<f64>,
    grain: f64,
    real_wage: f64,
    map: Vec<String>,
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let pos = q * (sorted.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
}

fn mark(st: &State, C: &Content, P: &Params, k: usize, map_sector: Option<usize>) -> Mark {
    let (N, NS) = (st.N, C.NS);
    let series = |name: &str| -> f64 {
        let v = &st.series.data[st.series.index[name]];
        v[k * 12..k * 12 + 12].iter().sum::<f64>() / 12.0
    };
    // the units were made again when the year ended; the hands each had in
    // December went with them (`ex_Lprev`), the output did not
    let mut hands_at = vec![0.0; NS * N];
    for u in 0..st.ex_cell.len() {
        hands_at[st.ex_sector[u] * N + st.ex_cell[u]] = st.ex_Lprev[u];
    }
    let employed: f64 = st.month_stats.employment.iter().sum::<f64>().max(1e-9);
    let mut out = Mark { real_wage: series("real_wage"), grain: series("out_grain"), ..Mark::default() };
    for s in 0..NS {
        let name = C.sector_name[s].as_str();
        let extractive = mines::is_extractive(name);
        out.hands_share.push(st.month_stats.ex_labor_by_sector[s] / employed);
        out.area.push((0..N).map(|c| st.lam[(s, c)]).sum());
        if !extractive {
            continue;
        }
        let field = st.world.field(&C.sector_field[s]);
        let cells: Vec<usize> = (0..N).filter(|&c| hands_at[s * N + c] >= 1.0).collect();
        let mut h: Vec<f64> = cells.iter().map(|&c| hands_at[s * N + c]).collect();
        h.sort_by(f64::total_cmp);
        let hands: f64 = h.iter().sum();
        let mut acc = 0.0;
        let core = h.iter().rev().take_while(|&&x| { let go = acc < 0.9 * hands; acc += x; go }).count();
        let weighted = |f: &dyn Fn(usize) -> f64| cells.iter().map(|&c| hands_at[s * N + c] * f(c)).sum::<f64>() / hands.max(1e-9);
        let fsum: f64 = (0..N).filter(|&c| st.world.land[c]).map(|c| field[c]).sum();
        let g = C.sector_good[s];
        let gname = &C.good_name[g];
        out.rows.push(Row {
            workings: cells.len() as f64,
            core: core as f64,
            surface: *out.area.last().unwrap(),
            max_share: cells.iter().map(|&c| st.lam[(s, c)]).fold(0.0, f64::max),
            hands,
            med: quantile(&h, 0.5),
            max: quantile(&h, 1.0),
            out: series(&format!("out_{gname}")),
            price: series(&format!("price_{gname}")),
            royalty: mines::royalty_month(st, C, P, s),
            dist: weighted(&|c| st.dcatch[c]),
            height: weighted(&|c| st.world.h[c]),
            deposit_height: (0..N).filter(|&c| st.world.land[c]).map(|c| field[c] * st.world.h[c]).sum::<f64>() / fsum.max(1e-9),
        });
    }
    out.area.push((0..N).map(|c| st.lam[(NS, c)]).sum());
    if let Some(s) = map_sector {
        let field = st.world.field(&C.sector_field[s]);
        let top = (0..N).map(|c| hands_at[s * N + c]).fold(0.0, f64::max).max(1.0);
        let (H, W) = (st.world.H, st.world.W);
        let towns: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| st.town_node[t]).collect();
        for y in 0..H {
            let line: String = (0..W)
                .map(|x| {
                    let c = y * W + x;
                    let hd = hands_at[s * N + c];
                    if !st.world.land[c] {
                        ' '
                    } else if towns.contains(&c) {
                        'T'
                    } else if hd >= 1.0 {
                        char::from_digit(((hd / top * 9.0).ceil() as u32).min(9), 10).unwrap()
                    } else if st.rail[c] > 0 {
                        '='
                    } else if field[c] > 0.15 {
                        '.'
                    } else if st.world.h[c] > 0.55 {
                        '^'
                    } else {
                        ','
                    }
                })
                .collect();
            out.map.push(line);
        }
    }
    out
}

fn census(P: &Params, C: Content, map_sector: Option<usize>) -> BTreeMap<i64, Mark> {
    let mut s = sim::start(P, C);
    let C = s.content.clone();
    let y0 = C.start_year;
    let mut out = BTreeMap::new();
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved");
        let y = s.year - 1;
        if (y - y0) % 50 == 0 || y == s.end_year {
            out.insert(y, mark(&s.st, &C, P, (y - y0) as usize, map_sector.filter(|_| y == s.end_year)));
        }
    }
    out
}

fn main() {
    let mut rules = Rules::Port;
    let mut families = 0usize;
    let mut set: Option<String> = None;
    let mut map: Option<String> = None;
    let mut seeds: Vec<u64> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--rules" => rules = Rules::parse(&args.next().expect("--rules port|game")).expect("--rules port|game"),
            "--families" => families = args.next().and_then(|v| v.parse().ok()).expect("--families N"),
            "--set" => set = Some(args.next().expect("--set JSON")),
            "--map" => map = Some(args.next().expect("--map SECTOR")),
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
    let map_sector = map.as_ref().map(|m| C.sector_name.iter().position(|n| n == m).unwrap_or_else(|| panic!("{m} is not a sector")));
    let all: Vec<BTreeMap<i64, Mark>> = std::thread::scope(|sc| {
        let handles: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let (mut P, C) = (P.clone(), C.clone());
                P.seed = seed;
                sc.spawn(move || census(&P, C, map_sector))
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });
    let n = all.len() as f64;
    println!("{} rules, {} families, seeds {:?}{}", rules.as_str(), families, seeds, set.as_ref().map_or(String::new(), |s| format!(", {s}")));
    let ex: Vec<usize> = (0..C.NS).filter(|&s| mines::is_extractive(&C.sector_name[s])).collect();
    for &y in all[0].keys() {
        let marks: Vec<&Mark> = all.iter().filter_map(|c| c.get(&y)).collect();
        println!("\n{y}");
        println!(
            "{:<10}{:>9}{:>6}{:>9}{:>7}{:>9}{:>8}{:>8}{:>10}{:>7}{:>10}{:>6}{:>8}{:>9}",
            "sector", "workings", "core", "surface", "most", "hands", "median", "largest", "output", "price", "royalty", "dist", "height", "deposit"
        );
        for (j, &s) in ex.iter().enumerate() {
            let m = |f: &dyn Fn(&Row) -> f64| marks.iter().map(|k| f(&k.rows[j])).sum::<f64>() / n;
            println!(
                "{:<10}{:>9.0}{:>6.0}{:>9.1}{:>7.2}{:>9.0}{:>8.0}{:>8.0}{:>10.0}{:>7.2}{:>10.0}{:>6.1}{:>8.2}{:>9.2}",
                C.sector_name[s],
                m(&|r| r.workings),
                m(&|r| r.core),
                m(&|r| r.surface),
                m(&|r| r.max_share),
                m(&|r| r.hands),
                m(&|r| r.med),
                m(&|r| r.max),
                m(&|r| r.out),
                m(&|r| r.price),
                m(&|r| r.royalty),
                m(&|r| r.dist),
                m(&|r| r.height),
                m(&|r| r.deposit_height),
            );
        }
        let mean = |f: &dyn Fn(&Mark) -> f64| marks.iter().map(|k| f(k)).sum::<f64>() / n;
        let names: Vec<&str> = C.sector_name.iter().map(|s| s.as_str()).chain(["wild"]).collect();
        let area: Vec<String> = (0..names.len()).map(|r| format!("{} {:.0}", names[r], mean(&|k| k.area[r]))).collect();
        println!("land, in cells: {}", area.join(", "));
        let hands: Vec<String> = (0..C.NS).map(|r| format!("{} {:.3}", names[r], mean(&|k| k.hands_share[r]))).collect();
        println!("share of all hands: {}", hands.join(", "));
        println!("grain a month {:.0}, real wage {:.2}", mean(&|k| k.grain), mean(&|k| k.real_wage));
    }
    if let Some(last) = all.last().and_then(|c| c.values().last()) {
        for line in &last.map {
            println!("{line}");
        }
    }
}
