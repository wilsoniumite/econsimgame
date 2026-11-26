//! What haulage costs a run (docs/rules/haulage.md): island 7 under the game's
//! rules with six families, played to the end year twice side by side on two
//! threads, with `haulage.handling` and `haulage.wear` on and off. For each it
//! prints the processor time of the months (twelve `sim::run_month` a year) and
//! of the years' ends (`sim::end_year`, where the cost graph is made and
//! searched) from 1901, a month's time over the people it served, and then, on
//! each run's last state, the year's cost graph and its searches
//! (`land::update_transport` and `land::delta_fields`) made again five times.
//!
//!     cargo run --profile work -p isostate-core --example haulage -- [END_YEAR]
#![allow(non_snake_case)]
use std::time::Instant;

use isostate_core::haulage::HaulageParams;
use isostate_core::*;

fn play(on: bool, end: i64) -> String {
    let rules = Rules::Game;
    let mut P = rules.params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(end);
    P.haulage = if on { HaulageParams::game() } else { HaulageParams::default() };
    let mut s = sim::start(&P, rules.content());
    let (mut months, mut ends, mut heads, mut n) = (0.0, 0.0, 0.0, 0usize);
    while s.year <= s.end_year {
        let year = s.year;
        let t = Instant::now();
        for m in 0..12 {
            sim::run_month(&mut s, m).expect("money is conserved");
        }
        let tm = t.elapsed().as_secs_f64();
        let t = Instant::now();
        sim::end_year(&mut s);
        let te = t.elapsed().as_secs_f64();
        if year >= 1901 {
            months += tm;
            ends += te;
            heads += s.st.pop.iter().zip(&s.st.town_active).filter(|x| *x.1).map(|x| *x.0).sum::<f64>();
            n += 1;
        }
    }
    let mut st = s.st.clone();
    let t = Instant::now();
    for _ in 0..5 {
        land::update_transport(&mut st, &s.content, &s.params);
        land::delta_fields(&mut st, &s.content, &s.params);
    }
    let graph = t.elapsed().as_secs_f64() / 5.0;
    format!(
        "haulage {}: 1901-{end}, a month {:.2} ms, a year's end {:.1} ms, a month a million people {:.2} ms; the year's graph and its searches {:.1} ms (trains: {})",
        if on { "on " } else { "off" },
        1e3 * months / (12 * n) as f64,
        1e3 * ends / n as f64,
        1e3 * months / 12.0 / (heads / 1e6),
        1e3 * graph,
        st.graph.has_train()
    )
}

fn main() {
    let end: i64 = std::env::args().nth(1).map_or(2050, |y| y.parse().expect("a year"));
    let (a, b) = std::thread::scope(|s| {
        let a = s.spawn(|| play(true, end));
        let b = s.spawn(|| play(false, end));
        (a.join().unwrap(), b.join().unwrap())
    });
    println!("{a}\n{b}");
}
