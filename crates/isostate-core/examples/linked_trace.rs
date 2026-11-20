//! What the test of the measures reads (docs/rules/measures.md, "The test"):
//! one island under the game's rules with six families, played to 2050, and a
//! row a year written to a CSV.
//!
//! Each row has the year's mean of every observer series (the chained and the
//! linked wage, the gauges' and the rest), and for each good what households
//! and landlords bought in a month (the year's mean), its price weighted by
//! what was bought in each town (`uv_`, the year's mean of the months), the
//! island's price as the observer has it (`price_`) and what was made (`out_`).
//! From those a script prices a fixed basket in every year.
//!
//!     cargo run --release -p isostate-core --example linked_trace -- SEED OUT.csv [--link YEARS]
#![allow(non_snake_case)]
use std::fmt::Write as _;

use isostate_core::params::Rules;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: linked_trace SEED OUT.csv [--link YEARS]");
        std::process::exit(2);
    }
    let seed: u64 = args[1].parse().expect("a seed");
    let out = std::path::PathBuf::from(&args[2]);
    let mut P = Rules::Game.params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(2050);
    if let Some(k) = args.iter().position(|a| a == "--link") {
        P.linked.link = args[k + 1].parse().expect("a number of years");
    }
    // the year's sums of the months: bought, and price weighted by it, over the goods of the
    // content the run is played on (the game's with the tree laid over it, `sim.content`)
    let (mut bought, mut uv): (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
    let mut rows: Vec<(i64, Vec<f64>, Vec<f64>)> = Vec::new();
    let res = isostate_core::edits::play_noting(&P, Rules::Game.content(), &[], |sim, month| {
        let st = &sim.st;
        let G = sim.content.G;
        if bought.len() != G {
            bought = vec![0.0; G];
            uv = vec![0.0; G];
        }
        if month.is_some() {
            let ms = &st.month_stats;
            let P_nat = isostate_core::observer::national_prices(st);
            for g in 0..G {
                let (mut q, mut v) = (0.0, 0.0);
                for t in 0..st.T_max {
                    if st.town_active[t] {
                        let x = ms.hh_got[(t, g)] + ms.lord_got[(t, g)];
                        q += x;
                        v += x * st.price[(t, g)];
                    }
                }
                bought[g] += q;
                uv[g] += if q > 0.0 { v / q } else { P_nat[g] };
            }
            return;
        }
        rows.push((sim.year - 1, bought.iter().map(|v| v / 12.0).collect(), uv.iter().map(|v| v / 12.0).collect()));
        bought.iter_mut().for_each(|v| *v = 0.0);
        uv.iter_mut().for_each(|v| *v = 0.0);
    });
    let sim = match res {
        Ok(sim) => sim,
        Err(e) => {
            eprintln!("seed {seed}: FAILED {e}");
            std::process::exit(1);
        }
    };
    let (st, C) = (&sim.st, &sim.content);
    let (y0, G) = (C.start_year, C.G);
    let mut text = String::from("seed,year");
    for name in &st.series.names {
        write!(text, ",{name}").unwrap();
    }
    for g in &C.good_name {
        write!(text, ",bought_{g},uv_{g}").unwrap();
    }
    text.push('\n');
    for (y, b, u) in &rows {
        write!(text, "{seed},{y}").unwrap();
        for k in 0..st.series.names.len() {
            write!(text, ",{}", isostate_core::observer::annual(&st.series.data[k], y0, *y)).unwrap();
        }
        for g in 0..G {
            write!(text, ",{},{}", b[g], u[g]).unwrap();
        }
        text.push('\n');
    }
    std::fs::write(&out, text).expect("the trace written");
}
