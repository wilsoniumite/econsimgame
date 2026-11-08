//! Time a full run by part (the same loop as `sim::run_year`).
//!
//!     cargo run -p isostate-core --release --example timing [END_YEAR]
#![allow(non_snake_case)]
use std::time::Instant;
use isostate_core::*;
fn main() {
    let mut P = default_params();
    if let Some(y) = std::env::args().nth(1) { P.end_year = Some(y.parse().unwrap()); }
    let t0 = Instant::now();
    let mut sim = sim::start(&P, default_content());
    let t_start = t0.elapsed().as_secs_f64();
    let (mut t_month, mut t_progress, mut t_obs, mut t_year, mut t_obs_y, mut t_snap) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    while sim.year <= sim.end_year {
        let ty = Instant::now();
        for month in 0..12 {
            let t = Instant::now();
            econ::step_month(&mut sim.st, &sim.content, &sim.params);
            t_month += t.elapsed().as_secs_f64();
            let t = Instant::now();
            rail::progress(&mut sim.st, &sim.content, &sim.params, sim.year as f64 + (month as f64 + 0.5) / 12.0, &mut sim.events);
            sim.st.money_now = econ::money_total(&sim.st);
            t_progress += t.elapsed().as_secs_f64();
            let t = Instant::now();
            observer::record_month(&mut sim.st, &sim.content, &sim.params, sim.t);
            sim.t += 1;
            t_obs += t.elapsed().as_secs_f64();
        }
        let year = sim.year;
        let t = Instant::now();
        sim::yearly(&mut sim.st, &sim.content, &sim.params, &mut sim.rngs, year, &mut sim.events, false);
        t_year += t.elapsed().as_secs_f64();
        let t = Instant::now();
        observer::yearly(&mut sim.st, &sim.content, &sim.params, year, &mut sim.events);
        t_obs_y += t.elapsed().as_secs_f64();
        let t = Instant::now();
        record::snapshot(&mut sim.rec, &sim.st, &sim.content, &sim.params, (year - 1750) as usize);
        t_snap += t.elapsed().as_secs_f64();
        sim.year += 1;
        if year % 25 == 0 { println!("{year}: year took {:.3}s, firms {}", ty.elapsed().as_secs_f64(), sim.st.f_alive.iter().filter(|&&a| a).count()); }
    }
    println!("start (init + warm-up) {t_start:.2}s\nstep_month {t_month:.2}s\nrail.progress+money {t_progress:.2}s\nrecord_month {t_obs:.2}s\nyearly {t_year:.2}s\nobserver.yearly {t_obs_y:.2}s\nsnapshot {t_snap:.2}s\ntotal {:.2}s", t0.elapsed().as_secs_f64());
}
