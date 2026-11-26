//! Shops, offices and care in the blocks of houses (`docs/rules/works.md`,
//! part J): the squares by tenant, the largest works, and the offices that
//! grow in pairs.
//!
//!     cargo run --profile work -p isostate-core --example works_blocks -- [--seeds 1-8]
//!         [--years 1900,1950,2000,2050] [--families 6] [--set JSON]... [--top N] [--start landfall]
//!
//! Each island runs under the game's rules on its own thread. For each year
//! asked and each island:
//!
//! - the squares by tenant: works (and of them, on a town's cells, those of
//!   shops, offices, care and the builders), house blocks, and of the house
//!   blocks those with floor in other use (`Table::floor`, in eighths of a
//!   square's floor), park, pits, derelict and open ground;
//! - for the last year: the shops, offices and care works by size (the five
//!   largest, the median, how many there are) and each town's largest
//!   employer as a share of the town's works' hands and of its people in work;
//! - for the last year: works of one method in one town within a hundredth
//!   of each other's size (the pairs), with where each came from, and the
//!   largest offices' hands every ten years.
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use isostate_core::squares::{self, PER_CELL, Tenant};
use isostate_core::*;

struct Opts {
    seeds: Vec<u64>,
    years: Vec<i64>,
    P: Params,
    top: usize,
}

fn parse(args: &[String]) -> Opts {
    let mut o = Opts { seeds: (1..=8).collect(), years: vec![1900, 1950, 2000, 2050], P: game_params(), top: 5 };
    let (mut families, mut sets, mut landfall) = (6usize, Vec::new(), false);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => o.seeds = isostate_core::sweep::parse_seeds(&next()).expect("--seeds 1-8 or 1,2,7"),
            "--years" => o.years = next().split(',').map(|s| s.parse().expect("--years 1900,2050")).collect(),
            "--families" => families = next().parse().unwrap(),
            "--set" => sets.push(next()),
            "--top" => o.top = next().parse().unwrap(),
            "--start" => landfall = next() == "landfall",
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    if landfall {
        o.P = o.P.with_json_overrides(r#"{"landfall":{"enabled":true}}"#).expect("the landfall start");
    }
    for s in &sets {
        o.P = o.P.with_json_overrides(s).expect("--set takes JSON, as {\"works\":{\"in_blocks\":false}}");
    }
    o.P.n_families = families;
    o.years.sort_unstable();
    o.P.end_year = o.years.last().copied();
    o
}

/// The kind of a works for these counts, by its method.
fn kind(C: &Content, m: usize) -> &'static str {
    let (name, good) = (C.method_name[m].as_str(), C.good_name[C.method_out[m]].as_str());
    match (name, good) {
        ("offices", _) => "office",
        (_, "services") => "shop",
        (_, "care") => "care",
        (_, "building") => "builder",
        _ => "other",
    }
}

const KINDS: [&str; 5] = ["shop", "office", "care", "builder", "other"];

#[derive(Clone, Default)]
struct Count {
    works: usize,
    /// works' squares on a town's cells, by kind
    works_town: [usize; 5],
    houses: usize,
    /// house blocks with some floor in other use, and the eighths of floor in other use, by use
    floor_squares: usize,
    floor_eighths: [usize; 8],
    park: usize,
    pit: usize,
    derelict: usize,
    open: usize,
}

fn count(st: &State, C: &Content) -> Count {
    let mut n = Count::default();
    let b = &st.sq;
    for (c, &k) in b.index.iter().enumerate() {
        if k == 0 {
            continue;
        }
        let t = &b.tables[k as usize - 1];
        let town = st.urban.get(c).copied().unwrap_or(false);
        for q in 0..PER_CELL {
            match Tenant::of(t.tenant[q]) {
                Tenant::None => {}
                Tenant::Works(f) => {
                    n.works += 1;
                    if town {
                        let j = KINDS.iter().position(|&x| x == kind(C, st.f_method[f as usize])).unwrap();
                        n.works_town[j] += 1;
                    }
                }
                Tenant::Houses(_) => {
                    n.houses += 1;
                    let fl = t.floor[q];
                    if fl & 0x0f > 0 {
                        n.floor_squares += 1;
                        n.floor_eighths[(fl >> 4) as usize & 7] += (fl & 0x0f) as usize;
                    }
                }
                Tenant::Park(_) => n.park += 1,
                Tenant::Pit(_) => n.pit += 1,
                Tenant::Derelict(_) => n.derelict += 1,
                Tenant::Open(_) => n.open += 1,
            }
        }
    }
    n
}

#[derive(Clone)]
struct Big {
    seed: u64,
    f: usize,
    name: String,
    town: String,
    kind: &'static str,
    hands: f64,
    town_share: f64,
    people_share: f64,
    island_share: f64,
    squares: usize,
    km2: f64,
    born: i64,
    parent: i64,
    owner: i64,
    cell: usize,
    K: f64,
    order: f64,
    prod: f64,
}

struct Island {
    seed: u64,
    years: Vec<(i64, Count)>,
    by_kind: BTreeMap<&'static str, Vec<f64>>,
    big: Vec<Big>,
    /// each town's largest employer: (town, its works' hands, its people in work, the largest's hands, its kind)
    towns: Vec<(String, f64, f64, f64, &'static str)>,
    pairs: Vec<(Big, Big)>,
    history: BTreeMap<usize, Vec<(i64, f64, f64)>>,
    cpu: f64,
}

fn big_of(st: &State, C: &Content, P: &Params, seed: u64, f: usize, town_hands: &[f64], made: &[f64]) -> Big {
    let m = st.f_method[f];
    let i = st.f_town[f];
    let people = who_works::hands(st, P, i);
    Big {
        seed,
        f,
        name: st.f_name[f].clone(),
        town: st.town_name[i].clone(),
        kind: kind(C, m),
        hands: st.f_labor[f],
        town_share: st.f_labor[f] / town_hands[i].max(1e-9),
        people_share: st.f_labor[f] / people.max(1e-9),
        island_share: st.f_labor[f] / made[m].max(1e-9),
        squares: st.sq.works.get(f).map_or(0, |h| h.squares.len()),
        km2: works::land_held(st, C, P, f) * 4.0,
        born: st.f_born[f],
        parent: st.f_parent[f],
        owner: st.f_owner.get(f).copied().unwrap_or(-1),
        cell: st.f_cell[f],
        K: st.f_K[f],
        order: st.f_order_K[f],
        prod: st.f_prod[f],
    }
}

fn run(seed: u64, o: &Opts, C0: &Content) -> Island {
    let mut P = o.P.clone();
    P.seed = seed;
    let t0 = isostate_core::sweep::thread_cpu_seconds().unwrap_or(0.0);
    let mut s = sim::start(&P, C0.clone());
    let mut isl = Island { seed, years: Vec::new(), by_kind: BTreeMap::new(), big: Vec::new(), towns: Vec::new(), pairs: Vec::new(), history: BTreeMap::new(), cpu: 0.0 };
    let last = *o.years.last().unwrap();
    while s.year <= last {
        let year = s.year;
        for month in 0..12 {
            sim::run_month(&mut s, month).expect("money is conserved");
        }
        sim::end_year(&mut s);
        let (st, C, PP) = (&s.st, &s.content, &s.params);
        if year % 10 == 0 && year > 1900 {
            for f in 0..st.F_count {
                if st.f_alive[f] && kind(C, st.f_method[f]) == "office" {
                    isl.history.entry(f).or_default().push((year, st.f_labor[f], st.f_K[f] + st.f_order_K[f]));
                }
            }
        }
        if o.years.contains(&year) {
            isl.years.push((year, count(st, C)));
        }
        if year == last {
            let mut town_hands = vec![0.0; st.T_max];
            let mut by_method = vec![0.0; C.M];
            for f in 0..st.F_count {
                if st.f_alive[f] {
                    town_hands[st.f_town[f]] += st.f_labor[f];
                    by_method[st.f_method[f]] += st.f_labor[f];
                }
            }
            // the island's hands of each kind, for a works' share of its kind
            let mut made = vec![0.0; C.M];
            for m in 0..C.M {
                made[m] = (0..C.M).filter(|&n| kind(C, n) == kind(C, m)).map(|n| by_method[n]).sum();
            }
            let mut all: Vec<Big> = Vec::new();
            for f in 0..st.F_count {
                if st.f_alive[f] {
                    let b = big_of(st, C, PP, seed, f, &town_hands, &made);
                    isl.by_kind.entry(b.kind).or_default().push(b.hands);
                    all.push(b);
                }
            }
            all.sort_by(|a, b| b.hands.total_cmp(&a.hands).then(a.f.cmp(&b.f)));
            for t in 0..st.T_max {
                if !st.town_active[t] || town_hands[t] <= 0.0 {
                    continue;
                }
                if let Some(b) = all.iter().find(|b| st.f_town[b.f] == t) {
                    isl.towns.push((st.town_name[t].clone(), town_hands[t], who_works::hands(st, PP, t), b.hands, b.kind));
                }
            }
            for (j, a) in all.iter().enumerate().take(40) {
                for b in all.iter().skip(j + 1).take(40) {
                    if st.f_method[a.f] == st.f_method[b.f] && st.f_town[a.f] == st.f_town[b.f] && a.hands > 500.0 && (a.hands - b.hands).abs() < 0.01 * a.hands {
                        isl.pairs.push((a.clone(), b.clone()));
                    }
                }
            }
            let tops: Vec<usize> = all.iter().filter(|b| b.kind == "office").take(4).map(|b| b.f).collect();
            isl.history.retain(|f, _| tops.contains(f));
            isl.big = all.into_iter().take(o.top.max(10)).collect();
        }
    }
    isl.cpu = isostate_core::sweep::thread_cpu_seconds().unwrap_or(0.0) - t0;
    isl
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let o = parse(&args);
    let C0 = game_content();
    println!("works in the blocks: seeds {:?}, {} families, years {:?}", o.seeds, o.P.n_families, o.years);
    let runs: Vec<Island> = std::thread::scope(|sc| {
        let hs: Vec<_> = o.seeds.iter().map(|&seed| { let (o, C) = (&o, &C0); sc.spawn(move || run(seed, o, C)) }).collect();
        hs.into_iter().map(|h| h.join().expect("a run")).collect()
    });
    println!("\n== the squares by tenant ==");
    println!(
        "{:>6} {:>5} {:>8} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8} {:>7} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
        "island", "year", "works", "w.shop", "w.off", "w.care", "w.bld", "w.oth", "houses", "h.floor", "e.shop", "e.off", "e.care", "e.bld", "park", "pit", "derel", "open"
    );
    let mut sums: BTreeMap<i64, Count> = BTreeMap::new();
    for isl in &runs {
        for (year, n) in &isl.years {
            println!(
                "{:>6} {:>5} {:>8} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8} {:>7} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
                isl.seed, year, n.works, n.works_town[0], n.works_town[1], n.works_town[2], n.works_town[3], n.works_town[4], n.houses, n.floor_squares,
                n.floor_eighths[1], n.floor_eighths[2], n.floor_eighths[3], n.floor_eighths[4], n.park, n.pit, n.derelict, n.open
            );
            let s = sums.entry(*year).or_default();
            s.works += n.works;
            for j in 0..5 {
                s.works_town[j] += n.works_town[j];
            }
            s.houses += n.houses;
            s.floor_squares += n.floor_squares;
            for j in 0..8 {
                s.floor_eighths[j] += n.floor_eighths[j];
            }
            s.park += n.park;
            s.pit += n.pit;
            s.derelict += n.derelict;
            s.open += n.open;
        }
    }
    let k = runs.len().max(1);
    for (year, n) in &sums {
        println!(
            "{:>6} {:>5} {:>8} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8} {:>7} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
            "mean", year, n.works / k, n.works_town[0] / k, n.works_town[1] / k, n.works_town[2] / k, n.works_town[3] / k, n.works_town[4] / k, n.houses / k,
            n.floor_squares / k, n.floor_eighths[1] / k, n.floor_eighths[2] / k, n.floor_eighths[3] / k, n.floor_eighths[4] / k, n.park / k, n.pit / k, n.derelict / k, n.open / k
        );
        println!("{:>6} {:>5} works over houses {:.3}", "", year, n.works as f64 / n.houses.max(1) as f64);
    }
    let year = *o.years.last().unwrap();
    println!("\n== {year}: the shops, offices and care by size (hands at work, December) ==");
    for kd in ["shop", "office", "care", "builder"] {
        let mut all: Vec<(f64, u64)> = Vec::new();
        for isl in &runs {
            if let Some(v) = isl.by_kind.get(kd) {
                all.extend(v.iter().map(|&h| (h, isl.seed)));
            }
        }
        all.sort_by(|a, b| b.0.total_cmp(&a.0));
        let median = if all.is_empty() { 0.0 } else { all[all.len() / 2].0 };
        let five: Vec<String> = all.iter().take(5).map(|(h, s)| format!("{h:.0} (island {s})")).collect();
        println!("{kd}: {} works on {} islands ({:.0} an island); the median {:.0}; the five largest {}", all.len(), runs.len(), all.len() as f64 / runs.len() as f64, median, five.join(", "));
        for isl in &runs {
            let mut v = isl.by_kind.get(kd).cloned().unwrap_or_default();
            v.sort_by(|a, b| b.total_cmp(a));
            let med = if v.is_empty() { 0.0 } else { v[v.len() / 2] };
            println!("  island {}: {} works, the largest {:.0}, the median {:.0}", isl.seed, v.len(), v.first().copied().unwrap_or(0.0), med);
        }
    }
    println!("\n== {year}: the largest works ==");
    let mut big: Vec<Big> = runs.iter().flat_map(|i| i.big.iter().cloned()).collect();
    big.sort_by(|a, b| b.hands.total_cmp(&a.hands));
    for b in big.iter().take(o.top) {
        println!(
            "island {}: {} ({}) at {}, {:.0} hands; {:.3} of its town's works' hands, {:.3} of its people in work, {:.3} of the island's {} work; {} squares, {:.3} km2 in the books; born {}, parent {}, owner {}, cell {}",
            b.seed, b.name, b.kind, b.town, b.hands, b.town_share, b.people_share, b.island_share, b.kind, b.squares, b.km2, b.born, b.parent, b.owner, b.cell
        );
    }
    println!("\n== {year}: each town's largest employer (towns of 2,000 works' hands or more) ==");
    let mut most: (f64, String) = (0.0, String::new());
    for isl in &runs {
        let mut line: Vec<String> = Vec::new();
        for (t, h, p, b, kd) in &isl.towns {
            if *h >= 2000.0 {
                line.push(format!("{t} {b:.0} {kd} {:.2}/{:.2}", b / h, b / p.max(1e-9)));
                if b / h > most.0 {
                    most = (b / h, format!("island {} {t}: {b:.0} {kd} of {h:.0} works' hands ({:.2} of {p:.0} in work)", isl.seed, b / p.max(1e-9)));
                }
            }
        }
        println!("island {}: {}", isl.seed, line.join("; "));
    }
    println!("the largest share: {:.3}, {}", most.0, most.1);
    println!("\n== {year}: pairs (one method, one town, within a hundredth) ==");
    for isl in &runs {
        for (a, b) in &isl.pairs {
            for x in [a, b] {
                println!(
                    "island {}: works {} {} ({}) at {}: {:.1} hands, K {:.3} on order {:.3}, prod {:.4}; born {}, parent {}, owner {}, cell {}",
                    x.seed, x.f, x.name, x.kind, x.town, x.hands, x.K, x.order, x.prod, x.born, x.parent, x.owner, x.cell
                );
            }
        }
    }
    println!("\n== the largest offices of {year}, their hands every ten years (hands, capacity) ==");
    for isl in &runs {
        for (f, h) in &isl.history {
            let s: Vec<String> = h.iter().map(|(y, l, k)| format!("{y} {l:.0}/{k:.0}")).collect();
            println!("island {} works {}: {}", isl.seed, f, s.join(", "));
        }
    }
    println!("\n== processor time ==");
    for isl in &runs {
        println!("island {}: {:.1} s", isl.seed, isl.cpu);
    }
    let _ = squares::PER_CELL;
}
