//! The lorries' bill and the households' purse, town by town and month by month
//! (docs/rules/lorry.md, "The lorry's bill"): islands under the game's rules
//! with six families to 2050, any setting by name, each on a thread of its own.
//!
//!     cargo run --profile work -p isostate-core --example lorry_bill -- SEEDS OUT_DIR [key=value ...]
//!
//! SEEDS is a list such as `1-8` or `3,7`. For each island it writes
//! `OUT_DIR/seedN.csv`, a row for every town-month from the lorry's find whose
//! households ended the month below nothing or got less than 0.82 of their
//! food, and `OUT_DIR/seedN.txt`: the years with hunger in the news, by decade;
//! the town-months whose households ended below nothing, and how many of them
//! the lorries' bill alone put there (the purse after spending and the month's
//! income would have stood at nothing or above without it), and those of the
//! whole run from 1750 that ended below nothing with no lorries' bill that month
//! (another path), the first twelve named; the lorries' bill
//! and the carting's pay over the run, and in towns under a thousand people; the
//! least the hauliers' till held.
//!
//! Each row: the town, its people, the share of the food its households got,
//! the households' cash a head at the month's start and end, what the town's
//! households were paid for carting that month a head, what its lorries' oil
//! and plant cost that month a head, the households' income of the month a
//! head, the carting's work (worker-months), the share of hauling its carting
//! means to hold and holds, and the hauliers' till a head (nothing without it).
#![allow(non_snake_case)]
use std::fmt::Write as _;

use isostate_core::params::Rules;

#[global_allocator]
static HEAPS: isostate_core::heaps::ThreadHeaps = isostate_core::heaps::ThreadHeaps;

fn seeds(s: &str) -> Vec<u64> {
    let mut out = Vec::new();
    for part in s.split(',') {
        match part.split_once('-') {
            Some((a, b)) => out.extend(a.parse::<u64>().unwrap()..=b.parse::<u64>().unwrap()),
            None => out.push(part.parse().unwrap()),
        }
    }
    out
}

fn one(seed: u64, sets: &[String], dir: &std::path::Path) {
    let mut P = Rules::Game.params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(2050);
    for kv in sets {
        let (k, v) = kv.split_once('=').expect("key=value");
        P = isostate_core::sweep::set_field(&P, k, &isostate_core::sweep::parse_value(v)).unwrap();
    }
    let C = Rules::Game.content();
    let mut rows = String::new();
    let mut cash0: Vec<f64> = Vec::new();
    // town-months, below nothing, below nothing by the bill alone, hungry; the
    // bill and the carting's pay, all towns and towns under a thousand
    let (mut n, mut neg, mut by_bill, mut hungry) = (0usize, 0usize, 0usize, 0usize);
    let (mut bill, mut pay, mut bill_small, mut pay_small, mut plant) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut till_least = f64::INFINITY;
    let mut neg_after_bill_month = 0usize;
    let (mut other, mut others) = (0usize, String::new());
    let res = isostate_core::edits::play_noting(&P, C.clone(), &[], |sim, month| {
        let st = &sim.st;
        let T = st.T_max;
        if cash0.len() != T {
            cash0.resize(T, 0.0);
        }
        let Some(m) = month else { return };
        let lorries = st.tasks.cart_paid_run.len() == T;
        // every month of the run: a town's households below nothing with no
        // lorries' bill that month came there by another path
        for t in (0..T).filter(|&t| st.town_active[t] && st.hh_cash[t] < 0.0) {
            let b = if lorries { st.tasks.cart_paid_run[t] + st.tasks.cart_paid_plant[t] } else { 0.0 };
            if b <= 0.0 {
                other += 1;
                if other <= 12 {
                    let _ = writeln!(others, "  {} month {m}, {} ({:.0} people): cash a head {:.3} at the start, {:.3} at the end", sim.year, st.town_name[t], st.pop[t], cash0[t] / st.pop[t].max(1.0), st.hh_cash[t] / st.pop[t].max(1.0));
                }
            }
        }
        if lorries {
            for t in (0..T).filter(|&t| st.town_active[t]) {
                let pop = st.pop[t].max(1.0);
                let (run, pl) = (st.tasks.cart_paid_run[t], st.tasks.cart_paid_plant[t]);
                let b = run + pl;
                let cp = st.cart_bill.get(t).copied().unwrap_or(0.0);
                let inc = st.hh_income_prev[t];
                let till = st.tasks.cart_till.get(t).copied().unwrap_or(0.0);
                till_least = till_least.min(till);
                n += 1;
                bill += b;
                pay += cp;
                plant += pl;
                if st.pop[t] < 1000.0 {
                    bill_small += b;
                    pay_small += cp;
                }
                let end = st.hh_cash[t];
                if end < 0.0 {
                    neg += 1;
                    if end + b >= 0.0 && b > 0.0 {
                        by_bill += 1;
                    }
                    if b > 0.0 {
                        neg_after_bill_month += 1;
                    }
                }
                let food = st.food_ratio[t];
                if food < 0.82 {
                    hungry += 1;
                }
                if end < 0.0 || food < 0.82 {
                    let _ = writeln!(
                        rows,
                        "{seed},{},{m},{t},{},{:.0},{:.4},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.1},{:.4},{:.4},{:.3}",
                        sim.year,
                        st.town_name[t],
                        st.pop[t],
                        food,
                        cash0[t] / pop,
                        end / pop,
                        cp / pop,
                        run / pop,
                        pl / pop,
                        inc / pop,
                        st.tasks.cart_work[t],
                        st.tasks.cart_auto[t][isostate_core::tasks::HAUL],
                        st.tasks.cart_held[t][isostate_core::tasks::HAUL],
                        till / pop,
                    );
                }
            }
        }
        cash0.copy_from_slice(&st.hh_cash[..T]);
    });
    let sim = match res {
        Ok(s) => s,
        Err(e) => {
            eprintln!("seed {seed}: FAILED {e}");
            std::process::exit(1);
        }
    };
    let mut years: Vec<i64> = sim.events.iter().filter(|e| e.kind == "famine").map(|e| e.year.floor() as i64).collect();
    years.dedup();
    let mut txt = String::new();
    let _ = writeln!(txt, "seed {seed}, settings {sets:?}");
    let _ = writeln!(txt, "years with hunger in the news: {} ({} from 1990): {years:?}", years.len(), years.iter().filter(|&&y| y >= 1990).count());
    for d in (1750..2050).step_by(10) {
        let k = years.iter().filter(|&&y| y >= d && y < d + 10).count();
        if k > 0 {
            let _ = writeln!(txt, "  {d}s: {k}");
        }
    }
    let _ = writeln!(txt, "town-months from the lorry's find: {n}; hungry {hungry}; ending below nothing {neg}, of which with a lorries' bill that month {neg_after_bill_month}, and put there by the bill alone {by_bill}");
    let _ = writeln!(txt, "the lorries' bill {bill:.0} (plant {plant:.0}) against the carting's pay {pay:.0}: {:.3}; in towns under 1,000 people {bill_small:.0} against {pay_small:.0}: {:.3}", bill / pay, bill_small / pay_small);
    let _ = writeln!(txt, "the least any hauliers' till held: {}", if till_least.is_finite() { format!("{till_least:e}") } else { "-".into() });
    let _ = writeln!(txt, "town-months of the whole run ending below nothing with no lorries' bill that month: {other}");
    txt.push_str(&others);
    std::fs::write(dir.join(format!("seed{seed}.txt")), &txt).unwrap();
    let head = "seed,year,month,town,name,pop,food,cash_start_ph,cash_end_ph,cart_pay_ph,oil_ph,plant_ph,income_ph,cart_work,auto_haul,held_haul,till_ph\n";
    std::fs::write(dir.join(format!("seed{seed}.csv")), format!("{head}{rows}")).unwrap();
    print!("{txt}");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let list = seeds(&args[1]);
    let dir = std::path::PathBuf::from(&args[2]);
    std::fs::create_dir_all(&dir).unwrap();
    let sets: Vec<String> = args[3..].to_vec();
    let hs: Vec<_> = list
        .into_iter()
        .map(|seed| {
            let (sets, dir) = (sets.clone(), dir.clone());
            std::thread::Builder::new().stack_size(64 << 20).spawn(move || one(seed, &sets, &dir)).unwrap()
        })
        .collect();
    for h in hs {
        h.join().unwrap();
    }
}
