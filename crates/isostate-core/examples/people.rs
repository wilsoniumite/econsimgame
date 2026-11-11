//! What the `people` rules (docs/rules/people.md) do in one run: people, towns and
//! growth rates by decade, every founding with its reason, and how big the towns
//! founded after 1800 are at the end.
//!
//!     cargo run --release -p isostate-core --example people -- [SEED] [--families N] [--off]
//!         [--end-year Y] [--town NAME_OR_INDEX] [--every-year] [--candidates]
//!         [--set people.NAME=VALUE]...
//!
//! `--off` runs the port's rules for comparison. `--town` follows one town's
//! people (the default is town 0), with the cells of its market area. A line is
//! printed every ten years, or every year with `--every-year`. `--candidates`
//! prints, every ten years, the sites the founding rule looked at, with what the
//! rule weighs for each.
#![allow(non_snake_case)]
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut P = game_params();
    let mut sets: Vec<String> = Vec::new();
    let (mut follow, mut show_cands, mut every) = (String::from("0"), false, false);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--families" => {
                P.n_families = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--end-year" => {
                P.end_year = Some(args[i + 1].parse().unwrap());
                i += 1;
            }
            "--off" => P.people.enabled = false,
            "--candidates" => show_cands = true,
            "--every-year" => every = true,
            "--town" => {
                follow = args[i + 1].clone();
                i += 1;
            }
            "--set" => {
                let (k, v) = args[i + 1].split_once('=').expect("--set people.NAME=VALUE");
                sets.push(format!("\"{}\": {v}", k.trim_start_matches("people.")));
                i += 1;
            }
            s => P.seed = s.parse().expect("the seed is a number"),
        }
        i += 1;
    }
    if !sets.is_empty() {
        P = P.with_json_overrides(&format!("{{\"people\": {{{}}}}}", sets.join(", "))).unwrap();
    }
    let mut sim = sim::start(&P, game_content());
    let T = sim.st.T_max;
    let followed = |st: &State| -> usize { follow.parse().unwrap_or_else(|_| st.town_name.iter().position(|n| *n == follow).expect("no town of that name")) };
    let home = followed(&sim.st);
    println!("seed {}, {} families, people {}", P.seed, P.n_families, if P.people.enabled { "on" } else { "off" });
    println!("year  towns   people  | growth a year: least  mean  most | {:>12}  cells | real wage  emigrants", sim.st.town_name[home]);
    let mut before: Vec<f64> = sim.st.pop.clone();
    let mut rates: Vec<(f64, f64, f64)> = Vec::new();
    while sim.year <= sim.end_year {
        let year = sim.year;
        for month in 0..12 {
            sim::run_month(&mut sim, month).unwrap();
        }
        if show_cands && P.people.enabled && year % 10 == 0 {
            // what the rule is about to see (it counts this year's carting on last year's paths)
            sim.st.cell_flow_year = sim.st.cell_flow.clone();
            for c in peopling::candidates(&sim.st, &sim.content, &sim.params) {
                println!(
                    "    {year} site {:5} {:5}: saves {:.3} of wages, {:4} cells, {:6.0} people, takes {:.2} of a town",
                    c.cell,
                    c.kind,
                    c.saved / c.wages.max(1e-9),
                    c.cells,
                    c.people,
                    c.donor_share
                );
            }
        }
        {
            let Sim { st, content: C, params: P, rngs, events, rec, .. } = &mut sim;
            sim::yearly(st, C, P, rngs, year, events, false);
            observer::yearly(st, C, P, year, events);
            record::snapshot(rec, st, C, P, (year - C.start_year) as usize);
        }
        sim.year += 1;
        let st = &sim.st;
        let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t]).collect();
        let pop: f64 = act.iter().map(|&t| st.pop[t]).sum();
        // the island's growth, and the spread of the towns' own rates of increase
        let was: f64 = before.iter().sum();
        let g: Vec<f64> = if st.pp_growth.is_empty() { vec![f64::NAN] } else { act.iter().map(|&t| st.pp_growth[t]).collect() };
        rates.push((g.iter().cloned().fold(f64::INFINITY, f64::min), pop / was - 1.0, g.iter().cloned().fold(f64::NEG_INFINITY, f64::max)));
        before = st.pop.clone();
        if every || year % 10 == 0 || year == sim.end_year {
            let k = if every { 1 } else { rates.len().min(10) };
            let last = &rates[rates.len() - k..];
            let mean = |f: &dyn Fn(&(f64, f64, f64)) -> f64| last.iter().map(f).sum::<f64>() / k as f64;
            let wage = st.series.get("real_wage");
            let w = observer::annual(wage, sim.content.start_year, year);
            println!(
                "{year} {:6} {:8.0}  |               {:7.4} {:6.4} {:6.4} | {:12.0} {:6} | {:9.2} {:10.0}",
                act.len(),
                pop,
                mean(&|r| r.0),
                mean(&|r| r.1),
                mean(&|r| r.2),
                st.pop[home],
                st.town_cells[home].len(),
                w,
                st.pp_emigrants
            );
        }
    }
    let st = &sim.st;
    println!("\ntowns founded under the people rules:");
    println!("year  name                 kind   port rail  cells  people then  saving | people in {}", sim.end_year);
    for f in &st.pp_foundings {
        println!(
            "{} {:20} {:6} {:4} {:4} {:6} {:12.0} {:7.3} | {:8.0}",
            f.year,
            st.town_name[f.town],
            f.kind,
            if f.port { "yes" } else { "" },
            if f.rail { "yes" } else { "" },
            f.cells,
            f.people,
            f.saving,
            st.pop[f.town]
        );
    }
    if st.pp_foundings.is_empty() {
        for t in 0..T {
            if st.town_active[t] && st.town_founded[t] >= sim.content.start_year {
                println!("{} {:20} (the port's rule) | {:8.0}", st.town_founded[t], st.town_name[t], st.pop[t]);
            }
        }
    }
    let least = (0..T).filter(|&t| st.town_active[t]).map(|t| st.town_cells[t].len()).min().unwrap();
    println!("smallest market area at the end: {least} cells");
}
