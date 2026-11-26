//! The ground works hold (`docs/rules/works.md`, "The ground a works takes"):
//! by kind and by method, in the books and on the squares, against the ground
//! of the houses, in the years asked for.
//!
//!     cargo run --profile work -p isostate-core --example works_ground -- [--seeds 1-8]
//!         [--years 1850,1900,1950,2000,2050] [--families 6] [--set JSON]... [--cell C] [--top N]
//!
//! Each island runs under the game's rules on its own thread. For each year
//! asked, summed over the islands:
//!
//! - by kind of works (the content's group) and by method: works standing,
//!   the ground the books give them (km²) and the squares they hold, hands at
//!   work (the year's December) and the hands their plant is built for (its
//!   capacity at the method's book labour), and square metres a hand at work;
//! - the houses' squares, and the works' ground over the houses';
//! - the rent the works pay as a share of what they sell and of their profit
//!   before rent, what landowners take from works in a year, and the mean rent
//!   a town cell's land earns;
//! - the largest works by hands at work: its method, its hands, its share of
//!   its town's works' hands and of the island's making of its good, its ground
//!   and whether it stands at its own size limit or on a full cell.
//!
//! And for the whole run: the squares' log, its changes and those that give a
//! square back to a tenant it had in the three years before (the flicker).
//! With `--cell C` the first island's cell C is printed square by square in
//! the last year asked; without it, the cell of the largest town with the
//! most works squares.
#![allow(non_snake_case)]

use std::collections::{BTreeMap, HashMap};

use isostate_core::squares::{self, PER_CELL, Tenant};
use isostate_core::*;

/// Square metres of a 2 km cell, and of a 125 m square.
const CELL_M2: f64 = 4.0e6;
const SQUARE_KM2: f64 = 0.015625;

struct Opts {
    seeds: Vec<u64>,
    years: Vec<i64>,
    P: Params,
    cell: Option<usize>,
    top: usize,
}

fn parse(args: &[String]) -> Opts {
    let mut o = Opts { seeds: (1..=8).collect(), years: vec![1850, 1900, 1950, 2000, 2050], P: game_params(), cell: None, top: 5 };
    let (mut families, mut sets) = (6usize, Vec::new());
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => o.seeds = isostate_core::sweep::parse_seeds(&next()).expect("--seeds 1-8 or 1,2,7"),
            "--years" => o.years = next().split(',').map(|s| s.parse().expect("--years 1900,2050")).collect(),
            "--families" => families = next().parse().unwrap(),
            "--set" => sets.push(next()),
            "--cell" => o.cell = Some(next().parse().unwrap()),
            "--top" => o.top = next().parse().unwrap(),
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    for s in &sets {
        o.P = o.P.with_json_overrides(s).expect("--set takes JSON, as {\"works\":{\"by_method\":false}}");
    }
    o.P.n_families = families;
    o.years.sort_unstable();
    o.P.end_year = o.years.last().copied();
    o
}

#[derive(Clone, Default)]
struct Row {
    works: usize,
    km2: f64,
    squares: usize,
    hands: f64,
    book: f64,
    rent: f64,
    sales: f64,
    before_rent: f64,
}

impl Row {
    fn add(&mut self, o: &Row) {
        self.works += o.works;
        self.km2 += o.km2;
        self.squares += o.squares;
        self.hands += o.hands;
        self.book += o.book;
        self.rent += o.rent;
        self.sales += o.sales;
        self.before_rent += o.before_rent;
    }
}

#[derive(Clone)]
struct Big {
    seed: u64,
    name: String,
    method: String,
    hands: f64,
    book: f64,
    town_share: f64,
    good_share: f64,
    km2: f64,
    squares: usize,
    at_limit: bool,
    cell_full: f64,
}

#[derive(Clone, Default)]
struct Year {
    by_method: Vec<Row>,
    houses: usize,
    works_sq: usize,
    town_rent: f64,
    town_cells: usize,
    big: Vec<Big>,
    /// Works under one square of ground in the books, and those of them that
    /// hold no square of their own.
    small: usize,
    small_none: usize,
    /// Works whose cell is at its limit (the books' built share 0.95 or more).
    on_full: usize,
}

struct Island {
    seed: u64,
    years: Vec<(i64, Year)>,
    changes: u64,
    flicker: u64,
    flicker_works: u64,
    flicker_houses: u64,
    picture: Option<String>,
    first_seen: Vec<i64>,
    cpu: f64,
}

/// `out_acc` and `profit` are each works' output and profit over the year just
/// played, read before the review sets them back to nothing (`firms::review`).
#[allow(clippy::too_many_arguments)]
fn look(st: &State, C: &Content, P: &Params, out_acc: &[f64], profit: &[f64], top: usize, seed: u64) -> Year {
    let mut y = Year { by_method: vec![Row::default(); C.M], ..Year::default() };
    let census = squares::census(st);
    y.houses = census.houses;
    y.works_sq = census.works;
    let spread = P.merchant_spread;
    let mut town_hands = vec![0.0; st.T_max];
    let mut made = vec![0.0; C.G];
    for f in 0..st.F_count {
        if st.f_alive[f] {
            town_hands[st.f_town[f]] += st.f_labor[f];
            made[C.method_out[st.f_method[f]]] += out_acc[f];
        }
    }
    let mut bigs: Vec<(f64, usize)> = Vec::new();
    for f in 0..st.F_count {
        if !st.f_alive[f] {
            continue;
        }
        let m = st.f_method[f];
        let (i, g) = (st.f_town[f], C.method_out[m]);
        let held = works::land_held(st, C, P, f);
        let squares = st.sq.works.get(f).map_or(0, |h| h.squares.len());
        let p = st.pbar[(i, g)];
        let sales = out_acc[f] * p * (1.0 - spread);
        let r = &mut y.by_method[m];
        r.works += 1;
        r.km2 += held * CELL_M2 / 1e6;
        r.squares += squares;
        r.hands += st.f_labor[f];
        r.book += (st.f_K[f] + st.f_order_K[f]) * C.m_labor[m];
        r.rent += 12.0 * st.f_rent[f];
        r.sales += sales;
        r.before_rent += profit[f] + 12.0 * st.f_rent[f];
        if held * (PER_CELL as f64) < 1.0 {
            y.small += 1;
            if squares == 0 {
                y.small_none += 1;
            }
        }
        if st.cell_K_share[st.f_cell[f]] >= 0.95 {
            y.on_full += 1;
        }
        bigs.push((st.f_labor[f], f));
    }
    bigs.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    for &(_, f) in bigs.iter().take(top) {
        let m = st.f_method[f];
        y.big.push(Big {
            seed,
            name: st.f_name[f].clone(),
            method: C.method_label[m].clone(),
            hands: st.f_labor[f],
            book: (st.f_K[f] + st.f_order_K[f]) * C.m_labor[m],
            town_share: st.f_labor[f] / town_hands[st.f_town[f]].max(1e-9),
            good_share: out_acc[f] / made[C.method_out[m]].max(1e-9),
            km2: works::land_held(st, C, P, f) * CELL_M2 / 1e6,
            squares: st.sq.works.get(f).map_or(0, |h| h.squares.len()),
            at_limit: st.f_K[f] + st.f_order_K[f] >= 0.98 * works::max_K(C, P, m),
            cell_full: st.cell_K_share[st.f_cell[f]],
        });
    }
    for c in 0..st.N {
        if st.urban[c] {
            y.town_rent += st.land_rent_only[c];
            y.town_cells += 1;
        }
    }
    y
}

/// Cell c's squares as letters: works a letter each (by works), H houses,
/// P park, X pit, D derelict, . the town's open ground, a space nobody's;
/// lower case where the square is under water.
fn picture(st: &State, C: &Content, c: usize) -> String {
    let Some(t) = st.sq.table(c) else { return format!("cell {c} keeps no squares\n") };
    let mut letters: BTreeMap<u32, char> = BTreeMap::new();
    let mut s = String::new();
    for row in 0..squares::SIDE {
        for col in 0..squares::SIDE {
            let k = row * squares::SIDE + col;
            let ch = match Tenant::of(t.tenant[k]) {
                Tenant::None => ' ',
                Tenant::Works(f) => {
                    let n = letters.len() as u8;
                    *letters.entry(f).or_insert((b'A' + (n % 26)) as char)
                }
                Tenant::Houses(_) => 'H',
                Tenant::Park(_) => 'P',
                Tenant::Pit(_) => 'X',
                Tenant::Derelict(_) => 'D',
                Tenant::Open(_) => '.',
            };
            s.push(if t.water[k] && ch != ' ' { ch.to_ascii_lowercase() } else if t.water[k] { '~' } else { ch });
        }
        s.push('\n');
    }
    for (f, ch) in &letters {
        let f = *f as usize;
        s.push_str(&format!(
            "  {ch}: {} ({}), {:.0} hands at work, {:.0} by its plant, {} squares\n",
            st.f_name[f],
            C.method_label[st.f_method[f]],
            st.f_labor[f],
            (st.f_K[f] + st.f_order_K[f]) * C.m_labor[st.f_method[f]],
            st.sq.works[f].squares.len()
        ));
    }
    s
}

fn run(seed: u64, o: &Opts, C0: &Content) -> Island {
    let mut P = o.P.clone();
    P.seed = seed;
    let t0 = cpu_now();
    let mut s = sim::start(&P, C0.clone());
    let mut isl = Island { seed, years: Vec::new(), changes: 0, flicker: 0, flicker_works: 0, flicker_houses: 0, picture: None, first_seen: vec![0; s.content.M], cpu: 0.0 };
    // the flicker: per square, the tenants it had and the year each tenancy ended
    let mut past: HashMap<u32, Vec<(u32, i64)>> = HashMap::new();
    let mut now: HashMap<u32, (u32, i64)> = HashMap::new();
    let last = *o.years.last().unwrap();
    while s.year <= last {
        let year = s.year;
        for month in 0..12 {
            sim::run_month(&mut s, month).expect("money is conserved");
        }
        let out_acc = s.st.f_out_acc.clone();
        let profit = s.st.f_profit_acc.clone();
        sim::end_year(&mut s);
        let (st, C) = (&s.st, &s.content);
        for f in 0..st.F_count {
            let m = st.f_method[f];
            if st.f_alive[f] && isl.first_seen[m] == 0 {
                isl.first_seen[m] = year;
            }
        }
        for ch in &st.sq.changes {
            isl.changes += 1;
            let tenant = ch.tenant;
            let was = now.get(&ch.at).copied();
            if was.is_some_and(|(t, _)| t == tenant) {
                continue;
            }
            let hist = past.entry(ch.at).or_default();
            hist.retain(|&(_, end)| end >= year - 3);
            if tenant != 0 && hist.iter().any(|&(t, _)| t == tenant) {
                isl.flicker += 1;
                match Tenant::of(tenant) {
                    Tenant::Works(_) => isl.flicker_works += 1,
                    Tenant::Houses(_) => isl.flicker_houses += 1,
                    _ => {}
                }
            }
            if let Some((t, _)) = was {
                if t != 0 {
                    hist.push((t, year));
                }
            }
            now.insert(ch.at, (tenant, year));
        }
        if o.years.contains(&year) {
            let PP = &s.params;
            isl.years.push((year, look(st, C, PP, &out_acc, &profit, o.top, seed)));
        }
        if year == last && isl.picture.is_none() {
            let c = o.cell.unwrap_or_else(|| {
                let i = (0..st.T_max).max_by(|&a, &b| st.urban_pop[a].total_cmp(&st.urban_pop[b]).then(b.cmp(&a))).unwrap_or(0);
                let list = &st.town_ground[i];
                let works_on = |c: usize| st.sq.table(c).map_or(0, |t| t.tenant.iter().filter(|&&x| matches!(Tenant::of(x), Tenant::Works(_))).count());
                list.iter().copied().max_by(|&a, &b| works_on(a).cmp(&works_on(b)).then(b.cmp(&a))).unwrap_or(0)
            });
            isl.picture = Some(format!("island {seed}, {year}, cell {c}:\n{}", picture(st, C, c)));
        }
    }
    isl.cpu = cpu_now() - t0;
    isl
}

/// Processor time of this thread, in seconds (wall clock where the platform will not say).
fn cpu_now() -> f64 {
    isostate_core::sweep::thread_cpu_seconds().unwrap_or(0.0)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let o = parse(&args);
    let C0 = game_content();
    // the content as a run lays it (the game's overlay over the port's)
    let C = isostate_core::tree::content(&o.P, C0.clone()).expect("the game's content");
    println!(
        "works' ground: seeds {:?}, {} families, years {:?}",
        o.seeds, o.P.n_families, o.years
    );
    let runs: Vec<Island> = std::thread::scope(|sc| {
        let hs: Vec<_> = o.seeds.iter().map(|&seed| { let (o, C) = (&o, &C0); sc.spawn(move || run(seed, o, C)) }).collect();
        hs.into_iter().map(|h| h.join().expect("a run")).collect()
    });
    let n = runs.len() as f64;
    for (j, &year) in o.years.iter().enumerate() {
        let mut by_method = vec![Row::default(); C.M];
        let (mut houses, mut works_sq, mut small, mut small_none, mut on_full, mut town_rent, mut town_cells) = (0, 0, 0, 0, 0, 0.0, 0);
        let mut big: Vec<Big> = Vec::new();
        for isl in &runs {
            let Some((_, y)) = isl.years.get(j) else { continue };
            for (m, r) in y.by_method.iter().enumerate() {
                by_method[m].add(r);
            }
            houses += y.houses;
            works_sq += y.works_sq;
            small += y.small;
            small_none += y.small_none;
            on_full += y.on_full;
            town_rent += y.town_rent;
            town_cells += y.town_cells;
            big.extend(y.big.iter().cloned());
        }
        let mut groups: BTreeMap<String, Row> = BTreeMap::new();
        let mut all = Row::default();
        for m in 0..C.M {
            let g = if C.method_group[m].is_empty() { C.method_name[m].clone() } else { C.method_group[m].clone() };
            groups.entry(g).or_default().add(&by_method[m]);
            all.add(&by_method[m]);
        }
        println!("\n== {year}: the mean of {} islands ==", runs.len());
        println!(
            "works {:.0}; ground in the books {:.1} km2, on the squares {:.0} squares ({:.1} km2); houses {:.0} squares ({:.1} km2); works over houses {:.2} (squares)",
            all.works as f64 / n,
            all.km2 / n,
            works_sq as f64 / n,
            works_sq as f64 * SQUARE_KM2 / n,
            houses as f64 / n,
            houses as f64 * SQUARE_KM2 / n,
            works_sq as f64 / (houses as f64).max(1.0)
        );
        println!(
            "hands at work {:.0}, by their plant {:.0}; m2 a hand at work {:.0}, a hand of plant {:.0}",
            all.hands / n,
            all.book / n,
            all.km2 * 1e6 / all.hands.max(1e-9),
            all.km2 * 1e6 / all.book.max(1e-9)
        );
        println!(
            "rent: {:.1} per cent of the works' sales, {:.1} per cent of their profit before rent; landowners take {:.3e} a year from works; a town cell's land earns {:.1} a month (mean of {:.0} cells)",
            100.0 * all.rent / all.sales.max(1e-9),
            100.0 * all.rent / all.before_rent.max(1e-9),
            all.rent / n,
            town_rent / (town_cells as f64).max(1.0),
            town_cells as f64 / n
        );
        println!(
            "works under a square in the books {:.0} ({:.0} of them with no square of their own); works on a cell 0.95 built or more {:.0}",
            small as f64 / n,
            small_none as f64 / n,
            on_full as f64 / n
        );
        println!("{:<28} {:>6} {:>9} {:>8} {:>9} {:>9} {:>7} {:>7} {:>6}", "kind", "works", "km2", "squares", "hands", "plant", "m2/h", "m2/pl", "rent%");
        let line = |name: &str, r: &Row| {
            println!(
                "{:<28} {:>6.0} {:>9.1} {:>8.0} {:>9.0} {:>9.0} {:>7.0} {:>7.0} {:>6.1}",
                name,
                r.works as f64 / n,
                r.km2 / n,
                r.squares as f64 / n,
                r.hands / n,
                r.book / n,
                r.km2 * 1e6 / r.hands.max(1e-9),
                r.km2 * 1e6 / r.book.max(1e-9),
                100.0 * r.rent / r.sales.max(1e-9)
            )
        };
        for (g, r) in &groups {
            if r.works > 0 {
                line(g, r);
            }
        }
        println!("-- by method");
        for m in 0..C.M {
            if by_method[m].works > 0 {
                line(&C.method_name[m], &by_method[m]);
            }
        }
        big.sort_by(|a, b| b.hands.total_cmp(&a.hands));
        println!("-- the largest works by hands at work (of all islands)");
        for b in big.iter().take(o.top) {
            println!(
                "island {}: {} ({}), {:.0} hands at work, {:.0} by its plant; {:.2} of its town's works' hands, {:.2} of the island's {}; {:.2} km2, {} squares; {}; its cell {:.2} built",
                b.seed,
                b.name,
                b.method,
                b.hands,
                b.book,
                b.town_share,
                b.good_share,
                "good",
                b.km2,
                b.squares,
                if b.at_limit { "at its own limit" } else { "under its limit" },
                b.cell_full
            );
        }
    }
    println!("\n== the squares' log over the run ==");
    for isl in &runs {
        println!(
            "island {}: {} changes, {} of them a square given back to a tenant it had in the three years before ({} to works, {} to houses); processor {:.1} s",
            isl.seed, isl.changes, isl.flicker, isl.flicker_works, isl.flicker_houses, isl.cpu
        );
    }
    println!("\n== the first year a works of each method stood, by island ==");
    for m in 0..C.M {
        let ys: Vec<String> = runs.iter().map(|i| if i.first_seen[m] == 0 { "-".into() } else { i.first_seen[m].to_string() }).collect();
        println!("{:<22} {}", C.method_name[m], ys.join(" "));
    }
    if let Some(p) = runs.first().and_then(|i| i.picture.as_ref()) {
        println!("\n== a town cell ==\n{p}");
    }
}
