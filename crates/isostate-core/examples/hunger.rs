//! A town's bread, month by month (docs/rules/tending.md, "The hunger after
//! 1990"): one island under the game's rules with six families, any setting by
//! name, and a row a month from FROM to TO for every town whose households got
//! less than `under` of their food in some month of that year.
//!
//!     cargo run --profile work -p isostate-core --example hunger -- SEED FROM TO OUT.csv [key=value ...]
//!
//! Each row: the town, its people, the share of the food its households got,
//! the grain's price, the merchants' stock of grain and what share of the
//! month's asking it filled, grain made in the town, the wage, the households'
//! cash a head, the harvest's multiplier this month, the price of grain over
//! the wage, the households' income of the month before a head, and the
//! port's house rent a head.
#![allow(non_snake_case)]
use std::io::Write as _;

use isostate_core::params::Rules;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args[1].parse().unwrap();
    let from: i64 = args[2].parse().unwrap();
    let to: i64 = args[3].parse().unwrap();
    let out = std::path::PathBuf::from(&args[4]);
    let mut P = Rules::Game.params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(to);
    for kv in &args[5..] {
        let (k, v) = kv.split_once('=').expect("key=value");
        P = isostate_core::sweep::set_field(&P, k, &isostate_core::sweep::parse_value(v)).unwrap();
    }
    let C = Rules::Game.content();
    let mut year_rows: Vec<(usize, f64, String)> = Vec::new();
    let mut rows: Vec<String> = Vec::new();
    let mut last_prod: Vec<f64> = Vec::new();
    let res = isostate_core::edits::play_noting(&P, C.clone(), &[], |sim, month| {
        let st = &sim.st;
        let Cn = &sim.content;
        let g = Cn.gi("grain");
        match month {
            Some(m) => {
                if sim.year < from || sim.year > to {
                    return;
                }
                if last_prod.len() != st.T_max {
                    last_prod = vec![0.0; st.T_max];
                }
                for t in 0..st.T_max {
                    if !st.town_active[t] {
                        continue;
                    }
                    let made = st.town_prod[(t, g)] - last_prod[t];
                    last_prod[t] = st.town_prod[(t, g)];
                    let pop = st.pop[t].max(1.0);
                    year_rows.push((
                        t,
                        st.food_ratio[t],
                        format!(
                            "{seed},{},{m},{t},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                            sim.year,
                            st.town_name[t],
                            st.pop[t],
                            st.food_ratio[t],
                            st.price[(t, g)],
                            st.inv[(t, g)] / pop,
                            st.fill[(t, g)],
                            made / pop,
                            st.wage[t],
                            st.hh_cash[t] / pop,
                            st.harvest_town_now.get(t).copied().unwrap_or(f64::NAN),
                            st.price[(t, g)] / st.wage[t].max(1e-9),
                            st.hh_income_prev[t] / pop,
                            st.urban_rent[t] / pop,
                        ),
                    ));
                }
            }
            None => {
                // a year turned: keep the months of the towns that went short in it
                let worst: std::collections::BTreeMap<usize, f64> = year_rows.iter().fold(Default::default(), |mut a, (t, f, _)| {
                    let e = a.entry(*t).or_insert(9.0);
                    *e = e.min(*f);
                    a
                });
                for (t, _, r) in year_rows.drain(..) {
                    if worst.get(&t).is_some_and(|&w| w < 0.82) {
                        rows.push(r);
                    }
                }
            }
        }
    });
    if let Err(e) = res {
        eprintln!("seed {seed}: FAILED {e}");
        std::process::exit(1);
    }
    let mut f = std::io::BufWriter::new(std::fs::File::create(&out).unwrap());
    writeln!(f, "seed,year,month,town,name,pop,food_ratio,price_grain,stock_grain_a_head,fill_grain,made_grain_a_head,wage,cash_a_head,harvest_now,grain_over_wage,income_prev_a_head,urban_rent_a_head").unwrap();
    for r in rows {
        writeln!(f, "{r}").unwrap();
    }
}
