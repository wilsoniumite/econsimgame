//! The two mends of the works in the pass over the rules (`docs/rules/works.md`,
//! parts G and H), measured on their own faults. For each island it prints:
//!
//! - the twin works: works that are exact copies of another (the same cell,
//!   method, year opened and year closed), by where each came from (`old`, the
//!   works of 1749; `new`, the investors'; `expansion`, a sister works;
//!   `family`) and by the decade they opened; and how many of the works of 1749
//!   share their cell with another of their method in their town;
//! - the orders of more capacity for works already standing, and those whose
//!   works closed within three years of the order, a family's among them;
//! - the families' worth at the end, all six and the player's.
//!
//!     cargo run --release -p isostate-core --example works_mends -- [--seeds 1-16] [--rules game|port]
//!         [--families 6] [--end-year Y] [--set JSON]... [--list N] [--cells X,Z,R]
//!
//! `--set` lays JSON over the rules, as `{"works":{"spread_first":false}}`; `--list N`
//! names the N largest orders that closed within three years on each island, and N of
//! the cells where works of 1749 share a cell (with the map cell to point a picture
//! at); `--cells X,Z,R` prints the land and the works of every cell within R of
//! cell X,Z at the run's end.
//! Nothing here changes what a run does: it steps the island as `sim::run_year`
//! does and reads the state on either side of each year's end.
#![allow(non_snake_case)]
use std::collections::BTreeMap;

use isostate_core::*;

struct Opts {
    seeds: Vec<u64>,
    P: Params,
    rules: Rules,
    list: usize,
    spot: Option<(i64, i64, i64)>,
}

fn parse(args: &[String]) -> Opts {
    let mut o = Opts { seeds: (1..=16).collect(), P: game_params(), rules: Rules::Game, list: 0, spot: None };
    let (mut families, mut end_year, mut sets) = (6usize, None, Vec::new());
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => {
                let v = next();
                o.seeds = match v.split_once('-') {
                    Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                    None => v.split(',').map(|s| s.parse().unwrap()).collect(),
                };
            }
            "--rules" => o.rules = Rules::parse(&next()).expect("port or game"),
            "--families" => families = next().parse().unwrap(),
            "--end-year" => end_year = Some(next().parse().unwrap()),
            "--set" => sets.push(next()),
            "--list" => o.list = next().parse().unwrap(),
            "--cells" => {
                let v: Vec<i64> = next().split(',').map(|s| s.parse().expect("--cells X,Z,R in whole cells")).collect();
                o.spot = Some((v[0], v[1], v[2]));
            }
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    o.P = o.rules.params();
    for s in &sets {
        o.P = o.P.with_json_overrides(s).expect("--set takes JSON, as {\"works\":{\"keep_order\":false}}");
    }
    o.P.n_families = families;
    o.P.end_year = end_year;
    o
}

#[derive(Default)]
struct Island {
    works: usize,
    twins: usize,
    sets: usize,
    /// twins, then all works, by where they came from
    by_kind: BTreeMap<String, (usize, usize)>,
    /// twins by the decade they opened (1740 for the works of 1749)
    by_decade: BTreeMap<i64, usize>,
    first: usize,
    first_sharing: usize,
    orders: usize,
    orders_cost: f64,
    closed3: usize,
    closed3_cost: f64,
    closed3_fam: usize,
    closed3_fam_cost: f64,
    largest: Vec<(f64, String)>,
    worth: Vec<f64>,
    /// the works of 1749 that share a cell, set by set, the player's home town first
    sharing: Vec<String>,
    /// with `--cells X,Z,R`: the land about that spot at the run's end, a line a cell
    cells: Vec<String>,
}

fn island(seed: u64, mut sim: Sim, spot: Option<(i64, i64, i64)>) -> Island {
    let _ = seed;
    let C = sim.content.clone();
    let mut ordered: BTreeMap<usize, Vec<(i64, f64)>> = BTreeMap::new(); // works -> (year, cost of what was ordered)
    while sim.year <= sim.end_year {
        for month in 0..12 {
            sim::run_month(&mut sim, month).expect("money is conserved");
        }
        let year = sim.year;
        let before: Vec<(f64, f64, bool)> = (0..sim.st.F_count).map(|f| (sim.st.f_order_K[f], sim.st.f_K[f], sim.st.f_alive[f])).collect();
        sim::end_year(&mut sim);
        let st = &sim.st;
        for (f, &(ok0, k0, alive0)) in before.iter().enumerate() {
            if alive0 && st.f_alive[f] && k0 > 0.0 && st.f_order_K[f] > ok0 + 1e-9 {
                let dK = st.f_order_K[f] - ok0;
                let (i, m) = (st.f_town[f], st.f_method[f]);
                let unit: f64 = (0..C.G).map(|g| C.m_cap[(m, g)] * st.price[(i, g)]).sum();
                ordered.entry(f).or_default().push((year, dK * unit));
            }
        }
    }
    let st = &sim.st;
    let mut out = Island { works: st.F_count, ..Island::default() };
    let mut groups: BTreeMap<(usize, usize, i64, i64), Vec<usize>> = BTreeMap::new();
    for f in 0..st.F_count {
        groups.entry((st.f_cell[f], st.f_method[f], st.f_born[f], st.f_died[f])).or_default().push(f);
    }
    for (key, fs) in &groups {
        let e = out.by_kind.entry(st.f_reason[fs[0]].kind.clone()).or_default();
        e.1 += fs.len();
        if fs.len() > 1 {
            e.0 += fs.len();
            out.twins += fs.len();
            out.sets += 1;
            *out.by_decade.entry(key.2.div_euclid(10) * 10).or_default() += fs.len();
        }
    }
    let mut on_cell: BTreeMap<(usize, usize, usize), usize> = BTreeMap::new();
    for f in (0..st.F_count).filter(|&f| st.f_reason[f].kind == "old") {
        out.first += 1;
        *on_cell.entry((st.f_town[f], st.f_method[f], st.f_cell[f])).or_default() += 1;
    }
    out.first_sharing = on_cell.values().filter(|&&n| n > 1).sum();
    let seat = st.families.first().map(|k| k.seat);
    let W = st.world.W;
    let mut sets: Vec<(bool, String)> = on_cell
        .iter()
        .filter(|&(_, &n)| n > 1)
        .map(|(&(t, m, c), &n)| {
            let home = Some(t) == seat;
            let x = (c % W) as f64 + 0.5;
            let z = (c / W) as f64 + 0.5;
            (!home, format!("{n} {} of 1749 on one cell of {}{} at {x},{z}", C.method_name[m], st.town_name[t], if home { " (the home town)" } else { "" }))
        })
        .collect();
    sets.sort();
    out.sharing = sets.into_iter().map(|(_, s)| s).collect();
    if let Some((x0, z0, r)) = spot {
        for z in (z0 - r).max(0)..=(z0 + r).min(st.world.H as i64 - 1) {
            for x in (x0 - r).max(0)..=(x0 + r).min(W as i64 - 1) {
                let c = z as usize * W + x as usize;
                let here: Vec<String> = (0..st.F_count)
                    .filter(|&f| st.f_cell[f] == c && (st.f_alive[f] || st.f_reason[f].kind == "old"))
                    .map(|f| format!("{} ({}, {}{})", C.method_name[st.f_method[f]], st.f_reason[f].kind, st.f_born[f], if st.f_alive[f] { String::new() } else { format!(", closed {}", st.f_died[f]) }))
                    .collect();
                out.cells.push(format!("cell {x},{z}: {} of {}, works' share {:.2}; {}", isostate_core::land::USE_NAMES[st.land_use[c] as usize], if st.catch[c] >= 0 { st.town_name[st.catch[c] as usize].as_str() } else { "-" }, st.cell_K_share[c], here.join("; ")));
            }
        }
    }
    for (&f, orders) in &ordered {
        for &(y, cost) in orders {
            out.orders += 1;
            out.orders_cost += cost;
            let died = st.f_died[f];
            if died > y && died - y <= 3 {
                out.closed3 += 1;
                out.closed3_cost += cost;
                if st.f_owner[f] >= 0 {
                    out.closed3_fam += 1;
                    out.closed3_fam_cost += cost;
                }
                out.largest.push((cost, format!("{} ({}, {}, owner {}) given {:.0} in {y}, closed {died}", st.f_name[f], C.method_name[st.f_method[f]], st.town_name[st.f_town[f]], st.f_owner[f], cost)));
            }
        }
    }
    out.largest.sort_by(|a, b| b.0.total_cmp(&a.0));
    out.worth = st.families.iter().map(|k| k.net_worth).collect();
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let o = parse(&args);
    println!("{} rules, {} families, seeds {:?}; works.spread_first {}, works.keep_order {}", o.rules.as_str(), o.P.n_families, o.seeds, o.P.works.spread_first, o.P.works.keep_order);
    let runs: Vec<(u64, Island)> = std::thread::scope(|s| {
        let hs: Vec<_> = o
            .seeds
            .iter()
            .map(|&seed| {
                let mut P = o.P.clone();
                P.seed = seed;
                let content = o.rules.content();
                let spot = o.spot;
                s.spawn(move || (seed, island(seed, sim::start(&P, content), spot)))
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });
    let mut decades: BTreeMap<i64, usize> = BTreeMap::new();
    let mut kinds: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let (mut twins, mut first, mut sharing, mut orders, mut cost, mut c3, mut c3c, mut c3f, mut c3fc) = (0, 0, 0, 0, 0.0, 0, 0.0, 0, 0.0);
    let (mut worth_all, mut worth_player) = (0.0, 0.0);
    println!("island  works  twins  sets  of-1749 sharing  orders  worth-ordered  closed<=3y  worth   a-family's  worth   families'-worth  player's");
    for (seed, r) in &runs {
        let w_all: f64 = r.worth.iter().sum();
        let w_p = r.worth.first().copied().unwrap_or(0.0);
        println!(
            "{seed:>6} {:>6} {:>6} {:>5} {:>8} {:>7} {:>7} {:>14.0} {:>11} {:>9.0} {:>10} {:>9.0} {:>16.0} {:>9.0}",
            r.works, r.twins, r.sets, r.first, r.first_sharing, r.orders, r.orders_cost, r.closed3, r.closed3_cost, r.closed3_fam, r.closed3_fam_cost, w_all, w_p
        );
        for (d, n) in &r.by_decade {
            *decades.entry(*d).or_default() += n;
        }
        for (k, (t, a)) in &r.by_kind {
            let e = kinds.entry(k.clone()).or_default();
            e.0 += t;
            e.1 += a;
        }
        twins += r.twins;
        first += r.first;
        sharing += r.first_sharing;
        orders += r.orders;
        cost += r.orders_cost;
        c3 += r.closed3;
        c3c += r.closed3_cost;
        c3f += r.closed3_fam;
        c3fc += r.closed3_fam_cost;
        worth_all += w_all;
        worth_player += w_p;
    }
    let n = runs.len().max(1) as f64;
    println!("all {} islands: {twins} twins; of the {first} works of 1749, {sharing} share their cell with one of their kind", runs.len());
    println!("    {orders} orders for standing works worth {cost:.0}; {c3} of them ({c3c:.0}) for works that closed within three years, {c3f} ({c3fc:.0}) a family's");
    println!("    the families' worth at the end, the mean of an island: {:.0} (the player's {:.0})", worth_all / n, worth_player / n);
    println!("twins by where they came from (twins of all works):");
    for (k, (t, a)) in &kinds {
        println!("    {k:>10}: {t} of {a}");
    }
    println!("twins by the decade they opened (1740: the works of 1749 and the warm-up's):");
    for (d, k) in &decades {
        println!("    {d}s: {k}");
    }
    if o.list > 0 {
        println!("the largest orders for works that closed within three years:");
        for (seed, r) in &runs {
            for (_, l) in r.largest.iter().take(o.list) {
                println!("    island {seed}: {l}");
            }
        }
        println!("works of 1749 sharing a cell with one of their kind in their town, by the towns of the run's end:");
        for (seed, r) in &runs {
            for l in r.sharing.iter().take(o.list) {
                println!("    island {seed}: {l}");
            }
        }
    }
    for (seed, r) in &runs {
        for l in &r.cells {
            println!("island {seed}, {l}");
        }
    }
}
