//! How often a town has more hands at work than it has people of working age,
//! and by how much (docs/rules/mines.md, "When a working hires more hands than
//! its town has"). One line a seed, the seeds run side by side.
//!
//!     cargo run --release -p isostate-core --example overhire -- [port] [commons] [FAMILIES] [FIRST_SEED LAST_SEED]
//!         [MECHANISM.NAME=VALUE]...
//!
//! The defaults are the game's rules under the landlords, six families and
//! seeds 1 to 16. `port` runs the port's rules, `commons` the other regime,
//! and `mines.enabled=false` takes a mechanism out (or, over the port, puts
//! one in).
//!
//! The port's labour market looks for each town's wage between half and twice
//! last month's. When even twice last month's wage leaves more hands wanted
//! than the town has for hire, the wage stops there and every hand wanted is
//! hired, so a town can have more people at work than it has. For each run:
//!
//! - the town-months with more at work (in the fields, the works, the
//!   workings, carting and digging canals) than the town's people of working
//!   age, and those with a tenth, a half and twice as many again;
//! - the hands at work beyond the people there are, summed over the run, in
//!   hand-years, against all the hands at work;
//! - the worst town-month: how many times its people were at work, where and
//!   when, the town's founding year, the sector with most of its hands, and
//!   the largest single working there against the town's people of working age;
//! - the town-months in which one working alone had more hands than the town.
//!
//! A run that fails the monthly money check stops there and says so.
#![allow(non_snake_case)]
use isostate_core::*;

/// What one run showed.
#[derive(Default)]
struct Watch {
    months: usize,
    over: [usize; 4],
    excess: f64,
    at_work: f64,
    worst: (f64, String),
    one_working: usize,
    failed: String,
}

fn watch(P: &Params, port: bool) -> Watch {
    let mut sim = sim::start(P, if port { default_content() } else { game_content() });
    sim.check_money = true;
    let C = sim.content.clone();
    let T = sim.st.T_max;
    let mut w = Watch::default();
    while sim.year <= sim.end_year {
        let year = sim.year;
        for month in 0..12 {
            if let Err(e) = sim::run_month(&mut sim, month) {
                w.failed = format!("; FAILED: {e}");
                return w;
            }
            let st = &sim.st;
            let mut by_sector = vec![0.0; T * C.NS];
            let mut largest = vec![0.0f64; T];
            for u in 0..st.ex_cell.len().min(st.ex_labor.len()) {
                let t = st.ex_town[u];
                by_sector[t * C.NS + st.ex_sector[u]] += st.ex_labor[u];
                if mines::is_extractive(&C.sector_name[st.ex_sector[u]]) {
                    largest[t] = largest[t].max(st.ex_labor[u]);
                }
            }
            for t in (0..T).filter(|&t| st.town_active[t] && st.pop[t] > 0.0) {
                let (work, have) = (st.month_stats.employment[t], P.participation * st.pop[t]);
                w.months += 1;
                w.at_work += work / 12.0;
                let r = work / have;
                for (k, &x) in [1.0, 1.1, 1.5, 2.0].iter().enumerate() {
                    w.over[k] += (r > x * (1.0 + 1e-9)) as usize;
                }
                w.excess += (work - have).max(0.0) / 12.0;
                w.one_working += (largest[t] > have) as usize;
                if r > w.worst.0 {
                    let s = (0..C.NS).max_by(|&a, &b| by_sector[t * C.NS + a].total_cmp(&by_sector[t * C.NS + b])).unwrap();
                    w.worst = (
                        r,
                        format!(
                            "{} {year}-{:02}, founded {}, {:.0} at work of {:.0}, most in {}, the largest working {:.0}",
                            st.town_name[t],
                            month + 1,
                            st.town_founded[t],
                            work,
                            have,
                            C.sector_name[s],
                            largest[t]
                        ),
                    );
                }
            }
        }
        sim::end_year(&mut sim);
    }
    w
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |word: &str| args.iter().any(|a| a == word);
    let (sets, plain): (Vec<&String>, Vec<&String>) =
        args.iter().filter(|a| !["port", "commons"].contains(&a.as_str())).partition(|a| a.contains('='));
    let families: usize = plain.first().map(|s| s.parse().expect("the number of families")).unwrap_or(6);
    let first: u64 = plain.get(1).map(|s| s.parse().expect("the first seed")).unwrap_or(1);
    let last: u64 = plain.get(2).map(|s| s.parse().expect("the last seed")).unwrap_or(16);
    let port = has("port");
    let mut base = if port { default_params() } else { game_params() };
    base.n_families = families;
    base.rent_regime = if has("commons") { RentRegime::Commons } else { RentRegime::Landlords };
    for kv in &sets {
        let (k, v) = kv.split_once('=').unwrap();
        let (mechanism, name) = k.split_once('.').expect("MECHANISM.NAME=VALUE");
        base = base.with_json_overrides(&format!("{{\"{mechanism}\": {{\"{name}\": {v}}}}}")).expect("MECHANISM.NAME=VALUE");
    }
    println!("{} rules under the {}, {families} families", if port { "port" } else { "game" }, base.rent_regime.as_str());
    let runs: Vec<(u64, Watch)> = std::thread::scope(|s| {
        let handles: Vec<_> = (first..=last)
            .map(|seed| {
                let mut P = base.clone();
                P.seed = seed;
                (seed, s.spawn(move || watch(&P, port)))
            })
            .collect();
        handles.into_iter().map(|(seed, h)| (seed, h.join().unwrap())).collect()
    });
    let (mut months, mut over, mut excess, mut at_work, mut one) = (0, [0; 4], 0.0, 0.0, 0);
    for (seed, w) in &runs {
        println!(
            "seed {seed:2}: over its people in {:3} town-months of {} ({} by a tenth, {} by half, {} twice); {:.0} hand-years beyond them of {:.0}; one working over the town in {}; worst {:.2}x ({}){}",
            w.over[0], w.months, w.over[1], w.over[2], w.over[3], w.excess, w.at_work, w.one_working, w.worst.0, w.worst.1, w.failed
        );
        months += w.months;
        for k in 0..4 {
            over[k] += w.over[k];
        }
        excess += w.excess;
        at_work += w.at_work;
        one += w.one_working;
    }
    println!(
        "all {} runs: over its people in {} town-months of {months} ({} by a tenth, {} by half, {} twice); {excess:.0} hand-years beyond them of {at_work:.0} ({:.2e}); one working over the town in {one}",
        runs.len(),
        over[0],
        over[1],
        over[2],
        over[3],
        excess / at_work.max(1.0)
    );
}
