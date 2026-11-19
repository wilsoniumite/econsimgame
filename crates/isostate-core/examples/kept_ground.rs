//! Prints, for docs/probes/tr-loose-b.md (a), what kept town ground does to the
//! things that could carry its cost to the wage, as means over twenty-year spans
//! and the islands given: the farmland and all open land (cells' worth of the
//! land rule's shares), the cells of town ground and the people on them, the
//! households' house rent over the wage bill, the land's rents over income, the
//! grain made a head and its price, the hands in works and the works standing.
//!
//!     cargo run --release -p isostate-core --example kept_ground -- [--seeds 1-16] [--families 6] [--set JSON]
//!
//! The game's rules; `--set` lays a JSON object over them, e.g.
//! `--set '{"towns":{"enabled":false}}'`. Run it once for each setting and set
//! the outputs side by side. Nothing here draws a random number of the run's or
//! changes its state.
#![allow(non_snake_case)]
use isostate_core::*;

const SPANS: [(i64, i64); 5] = [(1790, 1810), (1840, 1859), (1860, 1879), (1880, 1900), (1800, 1900)];
const COLS: [&str; 11] = [
    "farmland, cells' worth",
    "open land, cells' worth",
    "town cells",
    "people on town ground, thousands",
    "house rent over the wage bill",
    "rent of farms, woods and pits over income",
    "grain made a head a year",
    "grain's price, island mean",
    "hands in works, share of all people",
    "works standing",
    "people, thousands",
];

fn trace(seed: u64, P: &Params) -> Vec<(i64, [f64; COLS.len()])> {
    let mut P = P.clone();
    P.seed = seed;
    let mut s = sim::start(&P, Rules::Game.content());
    let C = s.content.clone();
    let farm = C.sector_name.iter().position(|n| n == "farm").unwrap();
    let grain = C.gi("grain");
    let mut rows = Vec::new();
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved every month");
        let year = s.year - 1;
        let st = &s.st;
        let act: Vec<usize> = (0..st.T_max).filter(|&i| st.town_active[i]).collect();
        let pop: f64 = act.iter().map(|&i| st.pop[i]).sum();
        let iy = accounts::island_year(st).expect("the books are kept");
        let farmland: f64 = (0..st.N).map(|c| st.lam[(farm, c)]).sum();
        let open: f64 = (0..st.N).map(|c| (0..C.NS).map(|r| st.lam[(r, c)]).sum::<f64>()).sum();
        let grain_made: f64 = act.iter().map(|&i| st.town_prod_year[(i, grain)]).sum();
        let price = act.iter().map(|&i| st.pbar[(i, grain)]).sum::<f64>() / act.len().max(1) as f64;
        let ind: f64 = act.iter().map(|&i| st.ind_labor[i]).sum();
        rows.push((
            year,
            [
                farmland,
                open,
                st.urban.iter().filter(|&&u| u).count() as f64,
                act.iter().map(|&i| st.urban_pop[i]).sum::<f64>() / 1000.0,
                iy.rent_town / iy.wage_bill.max(1e-9),
                (iy.rent_farm + iy.rent_pits) / iy.income.max(1e-9),
                grain_made / pop.max(1.0),
                price,
                ind / pop.max(1.0),
                (0..st.F_count).filter(|&f| st.f_alive[f]).count() as f64,
                pop / 1000.0,
            ],
        ));
    }
    rows
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut families, mut set): (Vec<u64>, usize, Option<String>) = ((1..=16).collect(), 6, None);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => seeds = sweep::parse_seeds(&next()).unwrap(),
            "--families" => families = next().parse().unwrap(),
            "--set" => set = Some(next()),
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let mut P = Rules::Game.params();
    P.n_families = families;
    P.accounts.enabled = true;
    if let Some(j) = &set {
        P = P.with_json_overrides(j).expect("a JSON object of parameters");
    }
    let runs: Vec<Vec<(i64, [f64; COLS.len()])>> = std::thread::scope(|sc| {
        let hs: Vec<_> = seeds.iter().map(|&seed| {
            let P = P.clone();
            sc.spawn(move || trace(seed, &P))
        }).collect();
        hs.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });
    println!("the game's rules, {families} families, seeds {}-{}{}", seeds[0], seeds[seeds.len() - 1], set.as_ref().map_or(String::new(), |s| format!(", with {s}")));
    println!("means over the islands of each island's mean over the span");
    print!("{:<44}", "");
    for (lo, hi) in SPANS {
        print!("{:>12}", format!("{lo}-{hi}"));
    }
    println!();
    for (k, name) in COLS.iter().enumerate() {
        print!("{name:<44}");
        for (lo, hi) in SPANS {
            let m: f64 = runs
                .iter()
                .map(|r| {
                    let v: Vec<f64> = r.iter().filter(|(y, _)| *y >= lo && *y <= hi).map(|(_, x)| x[k]).collect();
                    v.iter().sum::<f64>() / v.len().max(1) as f64
                })
                .sum::<f64>()
                / runs.len() as f64;
            print!("{:>12.4}", m);
        }
        println!();
    }
}
