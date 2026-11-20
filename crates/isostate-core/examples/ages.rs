//! The people's ages on one island, reckoned beneath the played totals
//! (docs/rules/ages.md): the island's figures every `--every` years, and the
//! rows of its largest town at the end (its first city) at the years asked.
//!
//!     cargo run --release -p isostate-core --example ages -- --seed 7 --end-year 2050 \
//!         --years 1900,1950,2050 [--every 25] [--csv FILE]
//!
//! `--csv` writes one line for each year asked: the town, its people, its
//! seventeen groups and its working ages' years of school, and the island's
//! seventeen groups after them.
#![allow(non_snake_case)]

use isostate_core::ages::{self, GROUPS};
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let seed: u64 = arg("--seed").map_or(7, |v| v.parse().unwrap());
    let end: i64 = arg("--end-year").map_or(2050, |v| v.parse().unwrap());
    let every: i64 = arg("--every").map_or(25, |v| v.parse().unwrap());
    let years: Vec<i64> = arg("--years").map_or(vec![1900, 1950, end], |v| v.split(',').map(|y| y.parse().unwrap()).collect());
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end);
    let mut sim = sim::start(&P, Rules::Game.content());
    // each year asked: every town's row and schooling, to pick the largest town at the end
    let mut kept: Vec<(i64, Vec<[f64; GROUPS]>, Vec<[f64; GROUPS]>)> = Vec::new();
    println!("year   people  births_k deaths_k  life  under15 15-64 over64 school");
    while sim.year <= sim.end_year {
        let year = sim.year;
        sim::run_year(&mut sim).unwrap();
        let st = &sim.st;
        if let Some(v) = st.ages.years.last().filter(|_| (year - 1750) % every == 0 || year == end) {
            println!(
                "{year} {:9.0} {:7.1} {:7.1} {:6.1} {:6.3} {:6.3} {:6.3} {:5.2}",
                v.people, v.births_k, v.deaths_k, v.life, v.under15, v.working, v.over64, v.school
            );
        }
        if years.contains(&year) {
            kept.push((year, st.ages.rows.clone(), st.ages.school.clone()));
        }
    }
    let st = &sim.st;
    let town = (0..st.T_max).max_by(|&a, &b| st.pop[a].total_cmp(&st.pop[b])).unwrap();
    println!("\nthe largest town at the end: {} (town {town})", st.town_name[town]);
    let mut csv = String::from("year,town,name,people,");
    csv += &(0..GROUPS).map(|a| format!("g{a:02}")).collect::<Vec<_>>().join(",");
    csv += ",school,";
    csv += &(0..GROUPS).map(|a| format!("island_g{a:02}")).collect::<Vec<_>>().join(",");
    csv += "\n";
    for (year, rows, school) in &kept {
        let r = rows[town];
        let n: f64 = r.iter().sum();
        let sch = ages::working_school(&r, &school[town]);
        let isl: Vec<f64> = (0..GROUPS).map(|a| rows.iter().map(|r| r[a]).sum()).collect();
        let sh = |rg: std::ops::Range<usize>| r[rg].iter().sum::<f64>() / n.max(1.0);
        println!("{year}: {n:9.0} people, under 15 {:.3}, 15 to 64 {:.3}, over 64 {:.3}, school {sch:.2}", sh(ages::YOUNG), sh(ages::WORKING), sh(ages::OLD));
        csv += &format!("{year},{town},{},{n}", st.town_name[town]);
        for v in r.iter().chain(std::iter::once(&sch)).chain(isl.iter()) {
            csv += &format!(",{v}");
        }
        csv += "\n";
    }
    if let Some(path) = arg("--csv") {
        std::fs::write(&path, csv).unwrap();
        println!("wrote {path}");
    }
}
