//! Who holds the money, month by month: the purses `econ::money_total` counts,
//! the common fund, the month's rents, and every town's wage, people and
//! households' cash. Written to find why an island under the commons stopped
//! with "money not conserved" (docs/rules/mines.md, "Under the commons"), and
//! the place to look the next time one stops, on a leak or on its money going
//! below nothing: the monthly check is switched off here, so the run goes on
//! past the month it would have stopped in and says so.
//!
//!     cargo run --release -p isostate-core --example purses -- SEED [FAMILIES] [END_YEAR]
//!         [port] [landlords] [monthly] [from=YEAR] [MECHANISM.NAME=VALUE]...
//!
//! The defaults are the game's rules under the commons, no families, to 1900,
//! and a line each December. `port` runs the port's rules, `landlords` the other
//! regime, `monthly` prints every month, `from=1751` starts printing there, and
//! `mines.enabled=false` takes a mechanism out (or, over the port, puts one in).
//!
//! Each line: the money there is and how far the purses are from it (the
//! check allows a millionth of the money); the purses; the fund and the rents
//! paid in the month; then for each town its wage, its people and its
//! households' cash, the last two in thousands. A town with more hands at work
//! than it has is named after that, with the sector that has most of them.
//! The last line gives the least any town's households held, the fund's
//! deepest debt and its months in debt.
#![allow(non_snake_case)]
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |word: &str| args.iter().any(|a| a == word);
    let (sets, plain): (Vec<&String>, Vec<&String>) =
        args.iter().filter(|a| !["port", "landlords", "monthly"].contains(&a.as_str())).partition(|a| a.contains('='));
    let seed: u64 = plain.first().map(|s| s.parse().expect("the seed")).expect("purses SEED [FAMILIES] [END_YEAR] ...");
    let families: usize = plain.get(1).map(|s| s.parse().expect("the number of families")).unwrap_or(0);
    let end: i64 = plain.get(2).map(|s| s.parse().expect("the end year")).unwrap_or(1900);
    let port = has("port");
    let mut P = if port { default_params() } else { game_params() };
    let mut from = i64::MIN;
    for kv in &sets {
        let (k, v) = kv.split_once('=').unwrap();
        match k.split_once('.') {
            Some((mechanism, name)) => {
                P = P.with_json_overrides(&format!("{{\"{mechanism}\": {{\"{name}\": {v}}}}}")).expect("MECHANISM.NAME=VALUE");
            }
            None if k == "from" => from = v.parse().expect("from=YEAR"),
            None => panic!("{kv}: MECHANISM.NAME=VALUE or from=YEAR"),
        }
    }
    P.seed = seed;
    P.n_families = families;
    P.end_year = Some(end);
    P.rent_regime = if has("landlords") { RentRegime::Landlords } else { RentRegime::Commons };
    let mut sim = sim::start(&P, if port { default_content() } else { game_content() });
    sim.check_money = false;
    let C = sim.content.clone();
    let (mut least, mut deepest, mut months, mut stopped) = ((f64::INFINITY, String::new()), 0.0f64, 0, String::new());
    while sim.year <= sim.end_year {
        let year = sim.year;
        for month in 0..12 {
            sim::run_month(&mut sim, month).unwrap();
            let st = &sim.st;
            let T = st.T_max;
            let act: Vec<usize> = (0..T).filter(|&t| st.town_active[t]).collect();
            let drift = st.money_now - (st.money_start + st.issued);
            if stopped.is_empty() && st.money_now < 0.0 {
                stopped = format!("; the check would have stopped it in {year}-{month}, the money below nothing ({:.6e})", st.money_now);
            } else if stopped.is_empty() && drift.abs() >= 1e-6 * st.money_now.max(1.0) {
                stopped = format!("; the check would have stopped it in {year}-{month} (drift {drift:.6e})");
            }
            for &t in &act {
                if st.hh_cash[t] < least.0 {
                    least = (st.hh_cash[t], format!("{} in {year}-{month}", st.town_name[t]));
                }
            }
            deepest = deepest.max(-st.fund_cash);
            months += (st.fund_cash < 0.0) as usize;
            if year < from || !(month == 11 || has("monthly")) {
                continue;
            }
            let sum = |v: &[f64]| v.iter().sum::<f64>();
            let firms: f64 = (0..st.f_cash.len()).filter(|&f| st.f_alive[f] || st.f_cash[f] != 0.0).map(|f| st.f_cash[f]).sum();
            let projects: f64 = st.projects.iter().map(|p| p.cash).sum();
            let fams = if families::active(st) { families::cash_total(st) } else { 0.0 };
            println!(
                "{year}-{month:<2} money {:.0} drift {drift:+.1e} | households {:.0} landlords {:.0} merchants {:.0} works {:.0} projects {:.0} families {:.0} | fund {:.0} rents {:.0}",
                st.money_now,
                sum(&st.hh_cash),
                sum(&st.lord_cash),
                sum(&st.merch_cash),
                firms,
                projects,
                fams,
                st.fund_cash,
                st.m_rent
            );
            let towns: Vec<String> = act.iter().map(|&t| format!("{:.2}/{:.0}/{:.0}", st.wage[t], st.pop[t] / 1e3, st.hh_cash[t] / 1e3)).collect();
            println!("        wage/people/cash: {}", towns.join(" "));
            for &t in &act {
                let (work, have) = (st.month_stats.employment[t], P.participation * st.pop[t]);
                if work > 1.2 * have {
                    let mut hands = vec![0.0; C.NS];
                    for u in (0..st.ex_cell.len()).filter(|&u| st.ex_town[u] == t) {
                        hands[st.ex_sector[u]] += st.ex_labor[u];
                    }
                    let s = (0..C.NS).max_by(|&a, &b| hands[a].total_cmp(&hands[b])).unwrap();
                    println!(
                        "        {} (founded {}): {work:.0} hands at work of {have:.0}, {:.0} of them in the {} sector; wage {:.2}",
                        st.town_name[t], st.town_founded[t], hands[s], C.sector_name[s], st.wage[t]
                    );
                }
            }
        }
        sim::end_year(&mut sim);
    }
    println!(
        "seed {seed}, {families} families: the least a town's households held {:.0} ({}); the fund's deepest debt {deepest:.0}, {months} months in debt{stopped}",
        least.0, least.1
    );
}
