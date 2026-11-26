//! The state's ways on one island (docs/rules/highways.md): played under the
//! game's rules with six families to the end year, with `highways.trunk` and
//! `highways.bypass` both on (or both off with `--off`), it
//! prints the large towns, every trunk road and bypass put in hand with its
//! year, its reckoned saving and cost, and each tenth year the treasury's
//! works, the hands at them, km of trunk road and the towns with most through
//! traffic; and the processor time of the months and the years' ends from 1901.
//!
//!     cargo run --profile work -p isostate-core --example highways -- [SEED] [END_YEAR] [--off]
#![allow(non_snake_case)]
use std::time::Instant;

use isostate_core::highways::{HighwayParams, NOT};
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let off = args.iter().any(|a| a == "--off");
    let nums: Vec<i64> = args.iter().filter_map(|a| a.parse().ok()).collect();
    let (seed, end) = (nums.first().copied().unwrap_or(7), nums.get(1).copied().unwrap_or(2050));
    let rules = Rules::Game;
    let mut P = rules.params();
    P.seed = seed as u64;
    P.n_families = 6;
    P.end_year = Some(end);
    P.highways = if off { HighwayParams::default() } else { HighwayParams::game() };
    let mut s = sim::start(&P, rules.content());
    let (mut months, mut ends, mut n) = (0.0, 0.0, 0usize);
    while s.year <= s.end_year {
        let year = s.year;
        let t = cpu();
        for m in 0..12 {
            sim::run_month(&mut s, m).expect("money is conserved");
        }
        let tm = cpu() - t;
        let t = cpu();
        sim::end_year(&mut s);
        let te = cpu() - t;
        if year >= 1901 {
            months += tm;
            ends += te;
            n += 1;
        }
        let st = &s.st;
        if year >= 1900 && year % 10 == 0 {
            let h = st.hw.years.iter().find(|v| v.year == year);
            let p = st.purse.years.iter().find(|v| v.year == year);
            let all: f64 = (0..st.T_max).filter(|&t| st.town_active[t]).map(|t| st.urban_pop[t]).sum();
            let top: Vec<String> = h.map_or(Vec::new(), |h| h.through_top.iter().map(|&(t, w)| format!("{} {:.2} M wkm ({:.3} of townsfolk)", st.town_name[t], 2.0 * w / 1e6, st.urban_pop[t] / all)).collect());
            println!(
                "{year}: trunk {:.0} km, bypass {:.0} km, {} bypasses, {} widened; works {:.5} of income, {:.0} hands ({:.5} of all); through by road {:.2} M wkm; most through: {}",
                h.map_or(0.0, |h| 2.0 * h.trunk_squares as f64),
                h.map_or(0.0, |h| 2.0 * h.bypass_squares as f64),
                h.map_or(0, |h| h.bypasses),
                h.map_or(0, |h| h.widened),
                match (h, p) { (Some(h), Some(p)) if p.income > 0.0 => h.spent / p.income, _ => f64::NAN },
                h.map_or(0.0, |h| h.menders),
                match (h, p) { (Some(h), Some(p)) if p.hands > 0.0 => h.menders / p.hands, _ => f64::NAN },
                h.map_or(0.0, |h| 2.0 * h.through / 1e6),
                top.join("; ")
            );
        }
    }
    let st = &s.st;
    for r in &st.hw.routes {
        println!(
            "trunk road {} - {}: in hand {}, {} squares ({} new), saves {:.1} a year on {:.1} ({:.3})",
            st.town_name[r.a], st.town_name[r.b], r.year, r.cells.len(), r.new, r.gain, r.cost, r.gain / r.cost.max(1e-9)
        );
    }
    for b in &st.hw.bypasses {
        println!(
            "{} {} (trunk road {}): in hand {}, open {}, {} squares, through {:.0} weight a square, place {} by through traffic, saves {:.1} a year on {:.1}",
            if b.widen { "high street widened in" } else { "bypass of" },
            st.town_name[b.town],
            b.route,
            b.year,
            if b.opened == NOT as i64 { "not yet".to_string() } else { b.opened.to_string() },
            b.cells.len(),
            b.through,
            b.rank,
            b.gain,
            b.cost
        );
    }
    println!("1901-{end}: a month {:.2} ms, a year's end {:.1} ms of processor time", 1e3 * months / (12 * n) as f64, 1e3 * ends / n as f64);
    let _ = Instant::now();
}

/// The thread's processor time so far, seconds (the wall clock where it cannot be read).
fn cpu() -> f64 {
    thread_local!(static START: Instant = Instant::now());
    sweep::thread_cpu_seconds().unwrap_or_else(|| START.with(|s| s.elapsed().as_secs_f64()))
}
