//! A census of town ground, for reading what kept town ground does
//! (`docs/rules/towns.md`): how many cells the towns stand on, how often a cell
//! comes and goes, where the works' hands are, and what the ground pays.
//!
//!     cargo run -p isostate-core --release --example towns_census -- [--rules port|game]
//!         [--families N] [--set JSON] [--town NAME] [SEED ...]
//!
//! `--set` takes a JSON object of parameter overrides, laid over the chosen
//! rules as a `--params` file is: `--set '{"towns":{"enabled":true,"rent_floor":false}}'`.
//! Seeds default to 3 and 7; each runs on its own thread and the tables are the
//! means over seeds.
//!
//! For 1750, 1800, 1850 and 1900:
//!
//! - town cells and industry cells (a quarter of the cell under works, outside
//!   any town), and the share of people living in towns;
//! - the share of works' hands that work on town ground, and within two of
//!   their market;
//! - the households' rent bill over the town people's wages;
//! - the median town cell's rent (what the cell would pay if houses stood on
//!   all of it) beside the median rent of the open cells next to town ground,
//!   and the share of town cells that pay less than the open ground beside them;
//! - the residents of the fullest and the emptiest town cell (the mean over
//!   towns), the farm land, the real wage and output per head over 1750's.
//!
//! Then, over the run from 1760: the churn by decade, the cells that were ever
//! town ground, those that were so every year, and those with more than one
//! spell. Last, one town by half-century: the player's seat when there are
//! families, the largest town otherwise, or the one named by `--town`.
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use isostate_core::params::Rules;
use isostate_core::*;

#[derive(Clone, Default)]
struct Mark {
    town_cells: f64,
    industry_cells: f64,
    urban_share: f64,
    hands_town: f64,
    hands_near: f64,
    bill: f64,
    town_rent: f64,
    open_rent: f64,
    under: f64,
    fullest: f64,
    emptiest: f64,
    farm: f64,
    real_wage: f64,
    gdp_pc: f64,
}

struct Census {
    marks: BTreeMap<i64, Mark>,
    /// By decade from 1760: (cells that left, cells that entered, cells at the start).
    decades: BTreeMap<i64, (f64, f64, f64)>,
    ever: f64,
    always: f64,
    respelled: f64,
    home: String,
    /// The home town by half-century: (cells, people, town people).
    home_marks: BTreeMap<i64, (f64, f64, f64)>,
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// The eight cells round c.
fn round(st: &State, c: usize) -> impl Iterator<Item = usize> {
    let (W, H) = (st.world.W as i64, st.world.H as i64);
    let (y, x) = (c as i64 / W, c as i64 % W);
    (-1i64..=1)
        .flat_map(move |dy| (-1i64..=1).map(move |dx| (y + dy, x + dx)))
        .filter(move |&(ny, nx)| ny >= 0 && nx >= 0 && ny < H && nx < W && !(ny == y && nx == x))
        .map(move |(ny, nx)| (ny * W + nx) as usize)
}

fn mark(st: &State, C: &Content, series: &dyn Fn(&str) -> f64) -> Mark {
    let N = st.N;
    let town: Vec<usize> = (0..N).filter(|&c| st.urban[c]).collect();
    let (mut hands, mut on_town, mut near) = (0.0, 0.0, 0.0);
    for f in (0..st.F_count).filter(|&f| st.f_alive[f]) {
        hands += st.f_labor[f];
        if st.urban[st.f_cell[f]] {
            on_town += st.f_labor[f];
        }
        if st.dcatch[st.f_cell[f]] <= 2.0 {
            near += st.f_labor[f];
        }
    }
    let kept = towns::kept(st);
    // what the whole cell would pay under houses: kept ground knows it; the port's
    // town cell has no works on it to speak of, so its residents' rent is that
    let ground = |c: usize| if kept { st.town_ground_rent[c] } else { st.cell_urban_rent[c] };
    let open = |c: usize| st.world.land[c] && !st.urban[c] && st.catch[c] >= 0 && st.cell_K_share[c] < 0.25;
    let mut fringe: Vec<usize> = town.iter().flat_map(|&c| round(st, c)).filter(|&c| open(c)).collect();
    fringe.sort();
    fringe.dedup();
    let under = town
        .iter()
        .filter(|&&c| {
            let beside: Vec<f64> = round(st, c).filter(|&n| open(n)).map(|n| st.land_rent_only[n]).collect();
            !beside.is_empty() && ground(c) < beside.iter().sum::<f64>() / beside.len() as f64
        })
        .count();
    let wages: f64 = (0..st.T_max).filter(|&i| st.town_active[i]).map(|i| st.urban_pop[i] * st.wbar[i]).sum();
    let bill: f64 = (0..st.T_max).filter(|&i| st.town_active[i]).map(|i| st.urban_rent[i]).sum();
    let (mut fullest, mut emptiest, mut n_towns) = (0.0, 0.0, 0.0);
    for i in (0..st.T_max).filter(|&i| st.town_active[i]) {
        let res: Vec<f64> = (0..N).filter(|&c| st.urban[c] && st.catch[c] == i as i64).map(|c| st.urban_res[c]).collect();
        if !res.is_empty() {
            fullest += res.iter().cloned().fold(0.0, f64::max);
            emptiest += res.iter().cloned().fold(f64::INFINITY, f64::min);
            n_towns += 1.0;
        }
    }
    let farm = C.sector_name.iter().position(|n| n == "farm").map_or(f64::NAN, |s| (0..N).map(|c| st.lam[(s, c)]).sum());
    Mark {
        town_cells: town.len() as f64,
        industry_cells: st.land_use.iter().filter(|&&u| u == land::USE_INDUSTRY).count() as f64,
        urban_share: series("urban_share"),
        hands_town: on_town / hands.max(1e-9),
        hands_near: near / hands.max(1e-9),
        bill: bill / wages.max(1e-9),
        town_rent: median(town.iter().map(|&c| ground(c)).collect()),
        open_rent: median(fringe.iter().map(|&c| st.land_rent_only[c]).collect()),
        under: under as f64 / (town.len() as f64).max(1.0),
        fullest: fullest / f64::max(n_towns, 1.0),
        emptiest: emptiest / f64::max(n_towns, 1.0),
        farm,
        real_wage: series("real_wage"),
        gdp_pc: series("gdp_pc"),
    }
}

fn census(P: &Params, C: Content, town: Option<&str>) -> Census {
    let mut s = sim::start(P, C);
    let C = s.content.clone();
    let y0 = C.start_year;
    let N = s.st.N;
    let mut out = Census {
        marks: BTreeMap::new(),
        decades: BTreeMap::new(),
        ever: 0.0,
        always: 0.0,
        respelled: 0.0,
        home: String::new(),
        home_marks: BTreeMap::new(),
    };
    // from 1760 (the 1750s are the founding of the second towns): years as town
    // ground, spells of it, and whether the cell was town ground last year
    let (mut years, mut spells, mut was) = (vec![0usize; N], vec![0usize; N], vec![false; N]);
    let mut start: Vec<bool> = Vec::new();
    let mut counted = 0usize;
    let mut home: Option<usize> = None;
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved");
        let y = s.year - 1;
        let st = &s.st;
        let k = (y - y0) as usize;
        let series = |name: &str| -> f64 {
            let v = &st.series.data[st.series.index[name]];
            v[k * 12..k * 12 + 12].iter().sum::<f64>() / 12.0
        };
        if y >= y0 + 10 {
            counted += 1;
            for c in 0..N {
                if st.urban[c] {
                    years[c] += 1;
                    if !was[c] {
                        spells[c] += 1;
                    }
                }
                was[c] = st.urban[c];
            }
            if (y - y0) % 10 == 0 {
                if !start.is_empty() {
                    let left = (0..N).filter(|&c| start[c] && !st.urban[c]).count() as f64;
                    let entered = (0..N).filter(|&c| !start[c] && st.urban[c]).count() as f64;
                    let n = start.iter().filter(|&&u| u).count() as f64;
                    out.decades.insert(y - 10, (left, entered, n));
                }
                start = st.urban.clone();
            }
        }
        if (y - y0) % 50 == 0 || y == s.end_year {
            out.marks.insert(y, mark(st, &C, &series));
            let h = *home.get_or_insert_with(|| match town {
                Some(name) => st.town_name.iter().position(|n| n == name).unwrap_or_else(|| panic!("seed {}: no town called {name}", P.seed)),
                None if !st.families.is_empty() => st.families[0].seat,
                None => (0..st.T_max).filter(|&i| st.town_active[i]).max_by(|&a, &b| st.pop[a].total_cmp(&st.pop[b])).expect("a town"),
            });
            out.home = st.town_name[h].clone();
            let cells = (0..N).filter(|&c| st.urban[c] && st.catch[c] == h as i64).count() as f64;
            out.home_marks.insert(y, (cells, st.pop[h], st.urban_pop[h]));
        }
    }
    out.ever = years.iter().filter(|&&n| n > 0).count() as f64;
    out.always = years.iter().filter(|&&n| n == counted).count() as f64;
    out.respelled = spells.iter().filter(|&&n| n > 1).count() as f64;
    out
}

fn main() {
    let mut rules = Rules::Port;
    let mut families = 0usize;
    let mut set: Option<String> = None;
    let mut town: Option<String> = None;
    let mut seeds: Vec<u64> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--rules" => rules = Rules::parse(&args.next().expect("--rules port|game")).expect("--rules port|game"),
            "--families" => families = args.next().and_then(|v| v.parse().ok()).expect("--families N"),
            "--set" => set = Some(args.next().expect("--set JSON")),
            "--town" => town = Some(args.next().expect("--town NAME")),
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
    let all: Vec<Census> = std::thread::scope(|sc| {
        let handles: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let (mut P, C, town) = (P.clone(), C.clone(), town.as_deref());
                P.seed = seed;
                sc.spawn(move || census(&P, C, town))
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });
    let n = all.len() as f64;
    println!("{} rules, {} families, seeds {:?}{}", rules.as_str(), families, seeds, set.as_ref().map_or(String::new(), |s| format!(", {s}")));
    let years: Vec<i64> = all[0].marks.keys().copied().collect();
    let first = years[0];
    print!("\n{:<44}", "");
    for y in &years {
        print!("{y:>9}");
    }
    println!();
    let row = |label: &str, digits: usize, f: &dyn Fn(&Mark) -> f64| {
        print!("{label:<44}");
        for y in &years {
            let v = all.iter().map(|c| f(&c.marks[y])).sum::<f64>() / n;
            print!("{v:>9.digits$}");
        }
        println!();
    };
    row("town cells", 0, &|m| m.town_cells);
    row("industry cells", 0, &|m| m.industry_cells);
    row("people living in towns, share", 3, &|m| m.urban_share);
    row("works' hands on town ground, share", 2, &|m| m.hands_town);
    row("works' hands within 2 of their market", 2, &|m| m.hands_near);
    row("rent bill over town people's wages", 3, &|m| m.bill);
    row("median town cell's rent, a month", 0, &|m| m.town_rent);
    row("median rent of the open cells beside", 0, &|m| m.open_rent);
    row("town cells under the open ground beside", 2, &|m| m.under);
    row("residents of a town's fullest cell", 0, &|m| m.fullest);
    row("residents of a town's emptiest cell", 0, &|m| m.emptiest);
    row("farm land, cells", 0, &|m| m.farm);
    print!("{:<44}", format!("real wage over {first}"));
    for y in &years {
        print!("{:>9.2}", all.iter().map(|c| c.marks[y].real_wage / c.marks[&first].real_wage).sum::<f64>() / n);
    }
    print!("\n{:<44}", format!("output per head over {first}"));
    for y in &years {
        print!("{:>9.2}", all.iter().map(|c| c.marks[y].gdp_pc / c.marks[&first].gdp_pc).sum::<f64>() / n);
    }
    let ratio: Vec<f64> = all.iter().map(|c| c.marks[years.last().unwrap()].town_cells / c.marks[&first].town_cells).collect();
    println!(
        "\n\ntown cells, last year over first: {:.2} (by seed {:.2} to {:.2})",
        ratio.iter().sum::<f64>() / n,
        ratio.iter().cloned().fold(f64::INFINITY, f64::min),
        ratio.iter().cloned().fold(0.0, f64::max)
    );

    println!("\nfrom {}, per seed:", first + 10);
    let (mut left, mut moved, mut decs) = (0.0, 0.0, 0.0);
    for c in &all {
        for &(l, e, start) in c.decades.values() {
            left += l / start.max(1.0);
            moved += (l + e) / start.max(1.0);
            decs += 1.0;
        }
    }
    println!("cells leaving town ground in a decade, over the cells at its start: {:.3}", left / f64::max(decs, 1.0));
    println!("cells leaving or entering in a decade, over the cells at its start:  {:.3}", moved / f64::max(decs, 1.0));
    let spread = |f: &dyn Fn(&Census) -> f64| {
        let v: Vec<f64> = all.iter().map(f).collect();
        format!("{:.0} ({:.0} to {:.0})", v.iter().sum::<f64>() / n, v.iter().cloned().fold(f64::INFINITY, f64::min), v.iter().cloned().fold(0.0, f64::max))
    };
    println!("cells that were ever town ground:  {}", spread(&|c| c.ever));
    println!("cells that were town ground every year: {}", spread(&|c| c.always));
    println!("cells with more than one spell as town ground: {}", spread(&|c| c.respelled));

    for (c, seed) in all.iter().zip(&seeds) {
        print!("\nseed {seed}, {}:", c.home);
        for (y, (cells, pop, urban)) in &c.home_marks {
            print!("  {y}: {cells:.0} cells, {pop:.0} people ({urban:.0} in town)");
        }
    }
    println!();
}
