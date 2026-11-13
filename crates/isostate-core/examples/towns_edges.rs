//! The edges of kept town ground (`docs/rules/towns.md`): what the census and
//! the sweep's means do not show. One line a seed, over the whole run:
//!
//! - the dearest town-year: the households' rent bill over the town people's
//!   wages, where and when, and how many town-years cost a quarter of the wages
//!   or more than half the households' purse (which is all they pay);
//! - the floor under a town cell's rent: the share of town cells and years it
//!   binds on, and what it adds to the households' bill;
//! - the most crowded town-year (people over room) and how many stand above
//!   their room, the fullest cell (the record's byte stops at 2,040);
//! - ground that changed town or was given up, and at the end of the run the
//!   cells that touch no other cell of their town, the towns on one cell, and
//!   the town cells a quarter and nine tenths under works.
//!
//!     cargo run -p isostate-core --release --example towns_edges -- [--rules port|game]
//!         [--families N] [--set JSON] [SEED ...]
//!
//! The rules are the port's with the life cycle for works and kept town ground
//! switched on, or with `--rules game` the game's as they stand (kept ground
//! beside everything else, as it is played); `--set` lays a JSON object of
//! overrides over them, as the census does:
//! `--set '{"towns":{"works_room":0,"empty":0}}'`. A run that fails the
//! monthly money check says so and reports what it had.
#![allow(non_snake_case)]

use isostate_core::params::Rules;
use isostate_core::*;

fn main() {
    let mut rules = Rules::Port;
    let mut families = 0usize;
    let mut set: Option<String> = None;
    let mut seeds: Vec<u64> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--rules" => rules = Rules::parse(&args.next().expect("--rules port|game")).expect("--rules port|game"),
            "--families" => families = args.next().and_then(|v| v.parse().ok()).expect("--families N"),
            "--set" => set = Some(args.next().expect("--set JSON")),
            _ => seeds.push(a.parse().unwrap_or_else(|_| panic!("{a} is not a seed"))),
        }
    }
    if seeds.is_empty() {
        seeds = vec![3, 7];
    }
    let mut P = rules.params();
    if rules == Rules::Port {
        P.works = works::WorksParams::game();
        P.towns = towns::TownsParams::game();
    }
    if let Some(text) = &set {
        P = P.with_json_overrides(text).unwrap_or_else(|e| panic!("{e}"));
    }
    P.n_families = families;
    let C = rules.content();
    let lines: Vec<String> = std::thread::scope(|sc| {
        let handles: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let (mut P, C) = (P.clone(), C.clone());
                P.seed = seed;
                sc.spawn(move || edges(&P, C))
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap_or_else(|_| "the run panicked".to_string())).collect()
    });
    for (s, l) in seeds.iter().zip(lines) {
        println!("seed {s}: {l}");
    }
}

fn edges(P: &Params, C: Content) -> String {
    let mut s = sim::start(P, C);
    let N = s.st.N;
    let W = s.st.world.W as i64;
    let (mut dearest, mut dearest_at) = (0.0f64, String::new());
    let (mut quarter, mut half_purse, mut town_years) = (0usize, 0usize, 0usize);
    let (mut crowded, mut crowded_at, mut above_room) = (0.0f64, String::new(), 0usize);
    let mut fullest = 0.0f64;
    let (mut floor_cells, mut cell_years, mut floor_adds, mut bills) = (0usize, 0usize, 0.0f64, 0.0f64);
    let (mut moved, mut given_up) = (0usize, 0usize);
    let mut owner_before: Vec<i64> = vec![-1; N];
    let mut failed = String::new();
    while s.year <= s.end_year {
        if let Err(e) = sim::run_year(&mut s) {
            failed = format!("FAILED, {e}; ");
            break;
        }
        let y = s.year - 1;
        let st = &s.st;
        let mut owner = vec![-1i64; N];
        for i in (0..st.T_max).filter(|&i| st.town_active[i]) {
            town_years += 1;
            let list = &st.town_ground[i];
            let room: f64 = list.iter().map(|&c| st.town_room[c]).sum();
            let per_room = st.urban_pop[i] / room.max(1.0);
            let dmax = list.iter().map(|&c| st.dist[(i, c)]).fold(0.0, f64::max);
            let wages = st.urban_pop[i] * st.wbar[i];
            for &c in list {
                owner[c] = i as i64;
                fullest = fullest.max(st.urban_res[c]);
                if wages > 0.0 {
                    // what the cell's residents would pay by the port's rule alone
                    let per_head = P.urban_rent_rate * st.wbar[i] * (1.0 - st.dist[(i, c)] / (dmax + 1.0)).max(0.0);
                    let houses = st.town_room[c] * per_room * per_head;
                    cell_years += 1;
                    if st.cell_urban_rent[c] > houses * (1.0 + 1e-9) {
                        floor_cells += 1;
                        floor_adds += st.cell_urban_rent[c] - houses;
                    }
                }
            }
            if wages > 0.0 {
                bills += st.urban_rent[i];
                let b = st.urban_rent[i] / wages;
                if b > dearest {
                    dearest = b;
                    dearest_at = format!("{} {y} ({} cells, {:.0} town people)", st.town_name[i], list.len(), st.urban_pop[i]);
                }
                if b > 0.25 {
                    quarter += 1;
                }
                // `econ::step_month`: the households pay the bill or half their budget, whichever is less
                let cash = st.hh_cash[i].max(0.0);
                let budget = cash.min(P.cons_income * st.hh_income_prev[i].max(0.0) + P.cons_wealth * cash);
                if st.urban_rent[i] > 0.5 * budget {
                    half_purse += 1;
                }
            }
            if per_room > crowded {
                crowded = per_room;
                crowded_at = format!("{} {y}", st.town_name[i]);
            }
            if per_room > 1.05 {
                above_room += 1;
            }
        }
        for c in 0..N {
            if owner_before[c] >= 0 && owner[c] >= 0 && owner[c] != owner_before[c] {
                moved += 1;
            }
            if owner_before[c] >= 0 && owner[c] < 0 {
                given_up += 1;
            }
        }
        owner_before = owner;
    }
    let st = &s.st;
    let (mut cells, mut one_cell, mut apart, mut quarter_works, mut all_works) = (0usize, 0usize, 0usize, 0usize, 0usize);
    for i in (0..st.T_max).filter(|&i| st.town_active[i]) {
        let list = &st.town_ground[i];
        cells += list.len();
        if list.len() == 1 {
            one_cell += 1;
        }
        for &c in list {
            if st.cell_K_share[c] >= 0.25 {
                quarter_works += 1;
            }
            if st.cell_K_share[c] >= 0.9 {
                all_works += 1;
            }
            let (y, x) = (c as i64 / W, c as i64 % W);
            let touches = list.iter().any(|&d| d != c && (d as i64 / W - y).abs() <= 1 && (d as i64 % W - x).abs() <= 1);
            if !touches && list.len() > 1 {
                apart += 1;
            }
        }
    }
    format!(
        "{failed}dearest town-year {dearest:.3} of wages, {dearest_at}; {quarter} of {town_years} town-years over a quarter, \
         {half_purse} over half the purse; floor binds on {:.3} of cell-years and adds {:.4} to the bill; \
         most crowded {crowded:.2} of room, {crowded_at}; {above_room} town-years above their room; fullest cell {fullest:.0}; \
         {moved} cell-years changed town, {given_up} given up; at the end {cells} town cells, {one_cell} towns on one cell, \
         {apart} cells apart from their town, {quarter_works} a quarter under works, {all_works} nine tenths",
        floor_cells as f64 / cell_years.max(1) as f64,
        floor_adds / bills.max(1e-9),
    )
}
