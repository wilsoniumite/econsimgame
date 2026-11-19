//! Who is paid for what they put up into a works (not in the Python;
//! docs/rules/owners.md). With the switch off the golden test (tests/golden.rs)
//! covers that the port did not move.
//!
//! The island runs here are played under the game's rules with the switches
//! they lean on set by name: `owners.enabled`, `accounts.enabled` (the books
//! the dividends are set against) and six families (whose works are not held
//! by shares). What they check holds on any island's history (shares that sum
//! to one, money that adds up, one run against another), so no later rule can
//! break them by changing island 7's.
#![allow(non_snake_case)]

use isostate_core::owners::{self, OwnersParams};
use isostate_core::*;

fn params(seed: u64, on: bool, end: i64) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end);
    P.accounts.enabled = true;
    P.owners = OwnersParams { enabled: on };
    P
}

#[test]
fn the_switch_is_off_under_the_port_and_on_in_the_game() {
    let (port, game) = (default_params(), game_params());
    assert_eq!(port.owners, OwnersParams::default());
    assert!(!port.owners.enabled && game.owners.enabled);
    // a port run's params.json does not mention it
    assert!(!serde_json::to_string(&port).unwrap().contains("\"owners\""));
}

/// Island 7 to 1850: at every year's end each works no family holds has
/// shares between nothing and one that sum to one, and the year's dividends
/// the rule paid are the books' own.
#[test]
fn island_7_to_1850_holds_its_works_by_shares_that_add_up() {
    let P = params(7, true, 1850);
    let mut s = sim::start(&P, game_content());
    let mut held = 0;
    while s.year <= s.end_year {
        let year = s.year;
        sim::run_year(&mut s).expect("money is conserved every month");
        let st = &s.st;
        for f in 0..st.F_count {
            if !st.f_alive[f] || st.f_owner[f] >= 0 {
                continue;
            }
            match st.owners.stake.get(f).filter(|v| v.iter().any(|&x| x != 0.0)) {
                Some(v) => {
                    let sum: f64 = v.iter().sum();
                    assert!((sum - 1.0).abs() < 1e-9, "{year}: works {f}'s shares {v:?} sum to {sum}");
                    assert!(v.iter().all(|&x| (-1e-12..=1.0 + 1e-12).contains(&x)), "{year}: works {f}'s shares {v:?}");
                    // the savers' fund and the public hold nothing in this step
                    assert!(v[owners::SAVERS] == 0.0 && v[owners::PUBLIC] == 0.0);
                    held += 1;
                }
                // a works whose shares were never read has nothing built yet
                None => assert!(st.f_K[f] <= 1e-9, "{year}: works {f} stands with no shares"),
            }
        }
        // the year's dividends: to landlords and households, what the works paid less the families' part
        let o = st.owners.years.last().expect("a year of the rule");
        let y = st.acct.years.last().expect("a year of the books");
        assert_eq!((o.year, y.year), (year, year));
        let near = |a: f64, b: f64| (a - b).abs() <= 1e-9 * b.abs().max(1.0);
        assert!(near(o.div[1], y.div_landlords) && near(o.div[2], y.div_households), "{year}: {:?} against the books' {} and {}", o.div, y.div_landlords, y.div_households);
        assert!(near(o.div[0], y.div_families), "{year}: families {} against the books' {}", o.div[0], y.div_families);
        assert!(o.div.iter().all(|&d| d >= 0.0) && o.lords_income >= o.div[1]);
        assert!(st.owners.worst < 1e-9);
    }
    assert!(held > 0);
    // landlords hold a part of the works, and are paid a part of their dividends
    let o = s.st.owners.years.last().unwrap();
    assert!(o.book[1] > 0.0 && o.div[1] > 0.0, "{o:?}");
}

/// Switched off, the rule keeps nothing and the books pay the landlords no
/// dividend; switched on, the island is another (`save::check`).
#[test]
fn off_is_the_game_without_it() {
    let play = |P: &Params| {
        let mut sim = sim::start(P, game_content());
        while sim.year <= sim.end_year {
            sim::run_year(&mut sim).expect("money is conserved every month");
        }
        sim
    };
    let x = play(&params(3, false, 1765));
    assert!(x.st.owners.stake.is_empty() && x.st.owners.years.is_empty() && x.st.owners.worst == 0.0);
    assert!(x.st.acct.years.iter().all(|y| y.div_landlords == 0.0));
    // and the run file's books do not mention it
    let header = serde_json::to_string(&x.st.acct.years).unwrap();
    assert!(!header.contains("div_landlords"));
    let y = play(&params(3, true, 1765));
    assert!(y.st.acct.years.iter().any(|y| y.div_landlords > 0.0));
    assert_ne!(save::check(&x), save::check(&y));
}
