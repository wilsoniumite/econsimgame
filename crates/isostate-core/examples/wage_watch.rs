//! Does any town's wage run away, and do new towns eat? One line a seed, the
//! seeds run side by side under the port's rules with or without `people`
//! (docs/rules/people.md, where these numbers are quoted).
//!
//!     cargo run --release -p isostate-core --example wage_watch -- [game] [FAMILIES] [on|off] [FIRST_SEED LAST_SEED]
//!         [people.NAME=VALUE | MECHANISM.NAME=VALUE]...
//!
//! The defaults are six families, `people` on and seeds 1 to 16. With the word
//! `game` the rest of the game's rules are in force too, and
//! `carting.enabled=false` takes one of them out again. For each run:
//!
//! - the town-months in which a town's wage is over four times the middle town's;
//! - the town-months in which a town's wage moved by half or more from the month
//!   before (February to December: January begins on the year's new districts),
//!   as a share of all town-months;
//! - the greatest and least a town's wage ever is against the middle town's, and where;
//! - the town-months with half as many hands again at work as the town has
//!   (the labour market lets demand run ahead of the people when the wage is at
//!   its monthly limit);
//! - hungry town-years (a month with under 82 per cent of the food needed, the
//!   port's famine line), and how many of them fell on a town founded in the last
//!   two years.
//!
//! A run that fails the monthly money check stops there and says so.
#![allow(non_snake_case)]
use isostate_core::*;

/// What one run showed.
#[derive(Default)]
struct Watch {
    hot: usize,
    flips: usize,
    months: usize,
    worst: (f64, String),
    least: (f64, String),
    overworked: usize,
    hungry: usize,
    hungry_young: usize,
    failed: String,
}

fn watch(P: &Params, game: bool) -> Watch {
    let mut sim = sim::start(P, if game { game_content() } else { default_content() });
    sim.check_money = true;
    let T = sim.st.T_max;
    let mut w = Watch { least: (f64::INFINITY, String::new()), ..Watch::default() };
    let mut last = sim.st.wage.clone();
    while sim.year <= sim.end_year {
        let year = sim.year;
        for month in 0..12 {
            if let Err(e) = sim::run_month(&mut sim, month) {
                w.failed = format!("; FAILED: {e}");
                return w;
            }
            let st = &sim.st;
            let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t] && st.pop[t] > 0.0).collect();
            let mut wages: Vec<f64> = act.iter().map(|&t| st.wage[t]).collect();
            wages.sort_by(f64::total_cmp);
            let middle = wages[wages.len() / 2];
            let mut work = vec![0.0; T];
            for u in 0..st.ex_cell.len().min(st.ex_labor.len()) {
                work[st.ex_town[u]] += st.ex_labor[u];
            }
            for f in 0..st.f_alive.len() {
                if st.f_alive[f] {
                    work[st.f_town[f]] += st.f_labor[f];
                }
            }
            for &t in &act {
                let rel = st.wage[t] / middle;
                let at = || format!("{} {year}-{:02}, founded {}", st.town_name[t], month + 1, st.town_founded[t]);
                w.hot += (rel > 4.0) as usize;
                if month > 0 && last[t] > 0.0 {
                    w.months += 1;
                    w.flips += ((st.wage[t] / last[t]).ln().abs() >= 1.5f64.ln()) as usize;
                }
                if rel > w.worst.0 {
                    w.worst = (rel, at());
                }
                if rel < w.least.0 {
                    w.least = (rel, at());
                }
                w.overworked += (work[t] > 1.5 * P.participation * st.pop[t]) as usize;
            }
            last = st.wage.clone();
        }
        // the year's worst month for food, before the yearly step clears it
        for t in 0..T {
            if sim.st.town_active[t] && sim.st.food_min_acc[t] < 0.82 {
                w.hungry += 1;
                w.hungry_young += (sim.st.town_founded[t] >= sim.content.start_year && year - sim.st.town_founded[t] <= 2) as usize;
            }
        }
        // the rest of sim::run_year
        let Sim { st, content: C, params: P, rngs, events, rec, .. } = &mut sim;
        sim::yearly(st, C, P, rngs, year, events, false);
        observer::yearly(st, C, P, year, events);
        record::snapshot(rec, st, C, P, (year - C.start_year) as usize);
        sim.year += 1;
    }
    w
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let game = args.iter().any(|a| a == "game");
    let (sets, plain): (Vec<&String>, Vec<&String>) = args.iter().filter(|a| *a != "game").partition(|a| a.contains('='));
    let families: usize = plain.first().map(|s| s.parse().expect("the number of families")).unwrap_or(6);
    let on = plain.get(1).map(|s| s.as_str() != "off").unwrap_or(true);
    let first: u64 = plain.get(2).map(|s| s.parse().expect("the first seed")).unwrap_or(1);
    let last: u64 = plain.get(3).map(|s| s.parse().expect("the last seed")).unwrap_or(16);
    let mut base = if game { game_params() } else { default_params() };
    base.n_families = families;
    base.people.enabled = on;
    // a bare NAME is one of people's numbers; MECHANISM.NAME is another mechanism's
    for kv in &sets {
        let (k, v) = kv.split_once('=').unwrap();
        let (mechanism, name) = k.split_once('.').unwrap_or(("people", k));
        base = base.with_json_overrides(&format!("{{\"{mechanism}\": {{\"{name}\": {v}}}}}")).expect("MECHANISM.NAME=VALUE");
    }
    println!("{} rules, {families} families, people {}", if game { "game" } else { "port" }, if base.people.enabled { "on" } else { "off" });
    let runs: Vec<(u64, Watch)> = std::thread::scope(|s| {
        let handles: Vec<_> = (first..=last)
            .map(|seed| {
                let mut P = base.clone();
                P.seed = seed;
                (seed, s.spawn(move || watch(&P, game)))
            })
            .collect();
        handles.into_iter().map(|(seed, h)| (seed, h.join().unwrap())).collect()
    });
    for (seed, w) in runs {
        println!(
            "seed {seed:2}: {:3} town-months over 4x the middle wage; greatest {:6.1}x ({}), least {:4.2}x ({}); the wage moved by half or more in {:.3} of town-months; {:3} overworked town-months; {:3} hungry town-years, {} of them in new towns{}",
            w.hot,
            w.worst.0,
            w.worst.1,
            w.least.0,
            w.least.1,
            w.flips as f64 / w.months.max(1) as f64,
            w.overworked,
            w.hungry,
            w.hungry_young,
            w.failed
        );
    }
}
