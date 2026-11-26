//! A town's month, year by year: what its wage, prices, food and purses did
//! (docs/rules/landfall.md, "The trap"). A trace for one town of one run.
//!
//!     cargo run --profile work -p isostate-core --example landfall_wage -- SEED FROM TO [TOWN]
//!         [--start settled|landfall] [--set key=value]...
#![allow(non_snake_case)]
use isostate_core::*;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let mut P = game_params();
    P.seed = a[0].parse().unwrap();
    P.n_families = 6;
    let (from, to): (i64, i64) = (a[1].parse().unwrap(), a[2].parse().unwrap());
    P.end_year = Some(to);
    let mut i = 3;
    let mut town = 0usize;
    if i < a.len() && !a[i].starts_with("--") {
        town = a[i].parse().unwrap();
        i += 1;
    }
    while i < a.len() {
        match a[i].as_str() {
            "--start" => landfall::set_start(&mut P, &a[i + 1]).unwrap(),
            "--set" => {
                let (k, v) = a[i + 1].split_once('=').unwrap();
                P = sweep::set_field(&P, k, &sweep::parse_value(v)).unwrap();
            }
            s => panic!("unknown {s}"),
        }
        i += 2;
    }
    let C0 = game_content();
    let mut sim = sim::start(&P, C0);
    let grain = sim.content.gi("grain");
    let food: Vec<usize> = (0..sim.content.G).collect();
    println!("year month | wage | grain price, stock | hh cash, merch cash, lord cash | money | pop, urban | harvest | farm hands, works hands");
    while sim.year <= to {
        let year = sim.year;
        for month in 0..12 {
            sim::run_month(&mut sim, month).unwrap();
            if year >= from {
                let st = &sim.st;
                let t = town;
                let farm: f64 = (0..st.ex_cell.len()).filter(|&u| st.ex_town[u] == t).map(|u| st.ex_labor[u]).sum();
                let works: f64 = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_town[f] == t).map(|f| st.f_labor[f]).sum();
                let units = (0..st.ex_cell.len()).filter(|&u| st.ex_town[u] == t).count();
                let lam: f64 = (0..st.ex_cell.len()).filter(|&u| st.ex_town[u] == t).map(|u| st.ex_lam[u]).sum();
                let nf = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_town[f] == t).count();
                // the hands for hire as the next month will reckon them, on a copy
                let mut cp = st.clone();
                let mut sup: Vec<f64> = cp.pop.iter().map(|&p| P.participation * p).collect();
                let s0 = sup[t];
                carting::take_carters(&mut cp, &sim.content, &P, &mut sup);
                let s1 = sup[t];
                mines::take_crews(&mut cp, &P, &mut sup);
                let s2 = sup[t];
                canals::take_navvies(&mut cp, &P, &mut sup);
                roads::take_menders(&mut cp, &P, &mut sup);
                print!("hands {s0:.0} carters {:.0} crews {:.0} other {:.0} | ", s0 - s1, s1 - s2, s2 - sup[t]);
                print!("units {units} lam {lam:.0} pexp {:.3} firms {nf} know {:.2} | ", st.pexp[(t, grain)], (0..st.know.cols).map(|k| st.know[(t, k)]).sum::<f64>());
                println!(
                    "{year} {month:2} | {:8.3} | {:7.3} {:9.1} | {:10.0} {:10.0} {:10.0} | {:10.0} | {:7.0} {:6.0} | {:5.2} | {:7.0} {:7.0}",
                    st.wage[t],
                    st.price[(t, grain)],
                    st.inv[(t, grain)],
                    st.hh_cash[t],
                    st.merch_cash[t],
                    st.lord_cash[t],
                    econ::money_total(st),
                    st.pop[t],
                    st.urban_pop.get(t).copied().unwrap_or(0.0),
                    st.harvest_town_now.get(t).copied().unwrap_or(st.harvest_now),
                    farm,
                    works
                );
            }
        }
        let _ = &food;
        sim::end_year(&mut sim);
        if year >= from {
            let st = &sim.st;
            let act: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
            println!("-- {year} end: towns {:?}, catch cells of town: {}", act.iter().map(|&t| (t, st.pop[t].round(), st.town_founded[t])).collect::<Vec<_>>(), st.catch.iter().filter(|&&c| c == town as i64).count());
        }
    }
}
