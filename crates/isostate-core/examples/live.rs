//! A whole run driven through the live API, as the viewer will drive it: six
//! families, `live_year` after every year, then the time it took and a short
//! table of the families and how much of the economy they own.
//!
//!     cargo run -p isostate-core --release --example live [N_FAMILIES] [END_YEAR]
#![allow(non_snake_case)]
use std::time::Instant;

use isostate_core::families::{self, Strategy};
use isostate_core::*;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut P = default_params();
    P.n_families = args.next().map_or(6, |s| s.parse().expect("number of families"));
    if let Some(y) = args.next() {
        P.end_year = Some(y.parse().expect("end year"));
    }
    let t0 = Instant::now();
    let mut s = sim::start(&P, default_content());
    let t_start = t0.elapsed().as_secs_f64();
    let stat = live_static(&s.st, &s.content);
    let mut cursor = LiveCursor::new(&s.content);
    let (mut t_sim, mut t_live, mut worst) = (0.0, 0.0, (0.0f64, 0i64));
    let mut events = 0;
    let mut shares = Vec::new();
    while s.year <= s.end_year {
        let t = Instant::now();
        sim::run_year(&mut s).expect("money is conserved");
        t_sim += t.elapsed().as_secs_f64();
        let t = Instant::now();
        let y = live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        let dt = t.elapsed().as_secs_f64();
        t_live += dt;
        if dt > worst.0 {
            worst = (dt, y.year);
        }
        events += y.events.len();
        if y.year % 50 == 0 || y.year == s.end_year {
            shares.push((y.year, families::ownership_shares(&s.st, &s.content)));
        }
    }
    let years = (s.end_year - s.content.start_year + 1) as f64;
    println!(
        "{} years, {} families: start {:.2} s, simulation {:.2} s, live_year {:.1} ms a year on average (worst {:.1} ms in {}); {} cells, {} series, {} events sent",
        years,
        P.n_families,
        t_start,
        t_sim,
        1e3 * t_live / years,
        1e3 * worst.0,
        worst.1,
        stat.W * stat.H,
        stat.series.len(),
        events
    );
    println!("\nshare owned by families:   works' capital   rent   money");
    for (y, (w, r, m)) in shares {
        println!("  {y}                       {:>6.1}%     {:>6.1}%  {:>6.1}%", 100.0 * w, 100.0 * r, 100.0 * m);
    }
    println!("\nfamily        strategy    seat          cash       works        land   net worth  firms cells members  income/yr");
    for k in 0..s.st.families.len() {
        let r = family_report(&s.st, &s.content, &s.params, k).unwrap();
        let strategy = match r.strategy {
            Strategy::Player => "player",
            Strategy::Landed => "landed",
            Strategy::Industrial => "industrial",
        };
        println!(
            "{:<13} {:<11} {:<12} {:>10.0} {:>11.0} {:>11.0} {:>11.0} {:>6} {:>5} {:>4}/{:<2} {:>10.0}",
            r.surname,
            strategy,
            r.seat_name,
            r.cash,
            r.works_value,
            r.land_value,
            r.net_worth,
            r.firms.len(),
            r.cells.len(),
            r.members.iter().filter(|m| m.alive).count(),
            r.members.len(),
            r.last_year.earned()
        );
    }
}
