//! The viewer's Work charts (the Markets panel's third half), reckoned straight
//! from the core: a sandbox's game on one island played to a year, and at each
//! fifty-year mark and the last year the line a script's `print` gives of the
//! panel, every figure taken from the game's own state rather than from the
//! series the viewer reads. Set beside the viewer's `work:` lines it says
//! whether the charts are true. It only reads.
//!
//!     cargo run --release -p isostate-core --example work_charts -- [--seed 7] [--to 2050]
//!         [--rule tasks.dial=10 --at 1901]
//!
//! The game is begun as the viewer begins a sandbox of six families
//! (`save::Game::under_to` under the game's rules). `--rule NAME=VALUE --at
//! YEAR` sets a rule by hand at the turn into YEAR, as the Rules tab's chip does
//! (`Order::ByHand` with `Edit::Rule`); VALUE is read as JSON.
//!
//! Where the figures come from: work's share of income, net of wear and of
//! spending, and the fork, from `st.gauges.years`; the shares of income paid to
//! the works' owners, the land's and the merchants, from `st.acct.years`; the
//! part of each class of task machines did, works and land together, from
//! `st.tasks.years`; each holder's part of the works' book from
//! `st.owners.years`; and the chained wage from `st.series`, the year's mean of
//! its months (`observer::annual`) over 1750's.
#![allow(non_snake_case)]
use isostate_core::edits::Edit;
use isostate_core::observer::annual;
use isostate_core::params::Rules;
use isostate_core::save::{Game, Order};

fn ratio(a: f64, b: f64) -> f64 {
    if b.abs() > 1e-12 { a / b } else { f64::NAN }
}

fn fig(v: f64) -> String {
    if v.is_finite() { format!("{v:.4}") } else { "none".to_string() }
}

/// The `work:` line of a year, as the viewer's `work_charts::said` writes it.
fn line(g: &Game, y: i64) -> String {
    let st = &g.sim.st;
    let y0 = g.sim.content.start_year;
    let ga = st.gauges.years.iter().find(|x| x.year == y);
    let ac = st.acct.years.iter().find(|x| x.year == y);
    let ta = st.tasks.years.iter().find(|x| x.year == y);
    let ow = st.owners.years.iter().find(|x| x.year == y);
    let gv = |f: fn(&isostate_core::gauges::GaugeYear) -> f64| ga.map_or(f64::NAN, f);
    let av = |f: fn(&isostate_core::IslandYear) -> f64| ac.map_or(f64::NAN, f);
    let wage = match st.series.index.get("real_wage_chained") {
        Some(&s) => annual(&st.series.data[s], y0, y) / annual(&st.series.data[s], y0, y0),
        None => f64::NAN,
    };
    let classes = ["hauling", "tending", "clerking", "judging"];
    let machines: Vec<String> = classes
        .iter()
        .enumerate()
        .map(|(k, w)| {
            let v = ta.map_or(f64::NAN, |t| {
                let held = t.held[k] + t.land_held.iter().map(|s| s[k]).sum::<f64>();
                let work = t.work[k] + t.land_work.iter().map(|s| s[k]).sum::<f64>();
                ratio(held, work)
            });
            format!("{w} {}", fig(v))
        })
        .collect();
    let held: Vec<String> = ["families", "landlords", "households"].iter().enumerate().map(|(i, w)| format!("{w} {}", fig(ow.map_or(f64::NAN, |o| ratio(o.book[i], o.book.iter().sum()))))).collect();
    format!(
        "work: {y} | wage {} | wages: of income {}, less wear {}, of spending {} | by machine: {} | to owners: works {}, land {}, merchants {} | works held: {} | a wage buys: made {}, the land's {}, both {}",
        fig(wage),
        fig(gv(|g| g.work_gross)),
        fig(gv(|g| g.work_net)),
        fig(gv(|g| g.spend_work)),
        machines.join(", "),
        fig(av(|a| a.share_works)),
        fig(av(|a| a.share_land)),
        fig(av(|a| a.share_merchants)),
        held.join(", "),
        fig(gv(|g| g.fork_made)),
        fig(gv(|g| g.fork_raw)),
        fig(gv(|g| g.fork_all)),
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seed, mut to, mut rule, mut at) = (7u64, 2050i64, None::<(String, serde_json::Value)>, None::<i64>);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seed" => seed = next().parse().expect("a seed"),
            "--to" => to = next().parse().expect("a year"),
            "--rule" => {
                let v = next();
                let (name, value) = v.split_once('=').expect("--rule NAME=VALUE");
                rule = Some((name.to_string(), serde_json::from_str(value).unwrap_or_else(|_| serde_json::json!(value))));
            }
            "--at" => at = Some(next().parse().expect("a year")),
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    if rule.is_some() != at.is_some() {
        panic!("--rule and --at go together");
    }
    let t0 = std::time::Instant::now();
    let mut g = Game::under_to(Rules::Game, seed, 6, 0, to).expect("a game to that year");
    let y0 = g.sim.content.start_year;
    while !g.over() {
        // at the turn into `at`, before its January is played
        if let (Some((name, value)), Some(y)) = (&rule, at) {
            if g.last_year() + 1 == y {
                let done = g.give(Order::ByHand { edit: Edit::Rule { name: name.clone(), value: value.clone() } });
                eprintln!("at the turn into {y}: {name} = {value}: {}", match &done { Ok(_) => "carried out".to_string(), Err(why) => format!("refused: {why}") });
            }
        }
        g.year().expect("the year is played");
        g.turn();
        let y = g.last_year();
        if (y - y0) % 50 == 0 || g.over() {
            println!("{}", line(&g, y));
        }
    }
    // the fixed basket of 1750, which the island's chart drew as what a wage buys before the chained wage
    let st = &g.sim.st;
    if let Some(&s) = st.series.index.get("real_wage") {
        let (y, d) = (g.last_year(), &st.series.data[s]);
        eprintln!("the fixed basket of 1750 (real_wage) in {y}: {:.4}, and over its own 1750: {:.4}", annual(d, y0, y), annual(d, y0, y) / annual(d, y0, y0));
    }
    eprintln!("island {seed}, six families, the game's rules, to {to}: {:.0} s", t0.elapsed().as_secs_f64());
}
