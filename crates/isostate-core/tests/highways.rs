//! The state's ways (not in the Python; docs/rules/highways.md): from 1909 the
//! treasury betters the ways between the large towns into trunk roads with
//! their fords bridged (`highways.trunk`), and from 1920 bypasses a town on
//! one or widens its high street where what they save pays (`highways.bypass`).
//!
//! Played on island 7 under the game's rules with six families and the books
//! kept, with the two switches set by name, and the rules the case leans on
//! set by name too: the traffic record (`traffic.enabled`), the lorry
//! (`tasks.lorry`, `lorry.roads`, `lorry.keep`), haulage (`haulage.handling`,
//! `haulage.wear`), the crossings and the state's purse, all as the game has
//! them on 26 November 2026. What it checks holds on any island's history
//! (a run against another run, the books, the money, what a works leaves),
//! but for one thing: that island 7 has a trunk road open by 1935 (it has
//! 148 km in 1930 under these rules).
#![allow(non_snake_case)]

use isostate_core::highways::{self, HighwayParams, NOT};
use isostate_core::*;

/// Island 7 to `end`, with the state's ways on or off and every rule its case leans on set by name.
fn island(on: bool, end: i64) -> Sim {
    let rules = Rules::Game;
    let mut P = rules.params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(end);
    P.accounts.enabled = true;
    P.traffic.enabled = true;
    P.tasks.lorry = true;
    P.lorry = lorry::LorryParams::game();
    P.haulage = haulage::HaulageParams::game();
    P.crossings.enabled = true;
    P.state.enabled = true;
    P.highways = if on { HighwayParams::game() } else { HighwayParams::default() };
    sim::start(&P, rules.content())
}

/// The switches are off under the port's rules and left out of its file, and
/// on under the game's. Island 7 to the end of 1909 is the same island with
/// them on and off, to the bit (the first trunk road is put in hand at the end
/// of 1909 at the soonest). On from there to 1935: money is conserved every
/// month and the books balance every year; a trunk road is open; every square
/// of one is land with a made road or better; no link of a trunk road between
/// two of its squares fords or ferries; a works that opened has given back its
/// cash and none of the state's is left without its entry; and the treasury
/// is never below nothing.
#[test]
fn island_7_stands_to_1909_and_the_state_builds_after() {
    let port = default_params();
    assert!(port.highways == HighwayParams::default() && !port.highways.trunk && !port.highways.bypass);
    assert!(!serde_json::to_string(&port).unwrap().contains("highways"), "the port's file says nothing of the state's ways");
    assert_eq!(Rules::Game.params().highways, HighwayParams::game());
    assert_eq!((highways::TRUNK_FROM, highways::BYPASS_FROM), (1909, 1920));
    let (mut on, off) = std::thread::scope(|s| {
        let a = s.spawn(|| {
            let mut sim = island(true, 1935);
            while sim.year <= 1909 {
                sim::run_year(&mut sim).expect("money is conserved every month");
            }
            sim
        });
        let b = s.spawn(|| {
            let mut sim = island(false, 1909);
            while sim.year <= sim.end_year {
                sim::run_year(&mut sim).expect("money is conserved every month");
            }
            sim
        });
        (a.join().unwrap(), b.join().unwrap())
    });
    // to the end of 1909, the same island
    assert_eq!(on.st.series.data.len(), off.st.series.data.len());
    for (k, (a, b)) in on.st.series.data.iter().zip(&off.st.series.data).enumerate() {
        assert!(a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits()), "series {} differs to 1909", on.st.series.names[k]);
    }
    assert_eq!(on.st.road, off.st.road);
    assert_eq!(on.st.purse.treasury.to_bits(), off.st.purse.treasury.to_bits());
    assert!(on.st.hw.trunk.iter().all(|&y| y == NOT), "no trunk road is open by the end of 1909");
    // and on from there
    while on.year <= on.end_year {
        let year = on.year;
        sim::run_year(&mut on).expect("money is conserved every month, with the state's works");
        let st = &on.st;
        assert!(st.purse.treasury >= 0.0, "{year}: the treasury is below nothing");
        let in_hand = st.projects.iter().filter(|pr| pr.way == ways::ROAD && matches!(pr.serves, highways::TRUNK | highways::BYPASS | highways::WIDEN)).count();
        assert_eq!(in_hand, st.hw.works.len(), "{year}: every works of the state's has its entry");
    }
    let st = &on.st;
    for y in &st.acct.years {
        assert!(y.balance_goods < 1e-9 && y.balance_income < 1e-9 && y.balance_spending < 1e-9, "{}: {} {} {}", y.year, y.balance_goods, y.balance_income, y.balance_spending);
    }
    let open: Vec<usize> = (0..st.N).filter(|&c| st.hw.trunk[c] != NOT).collect();
    assert!(!open.is_empty(), "island 7 has a trunk road by 1935");
    for &c in &open {
        assert!(st.world.land[c] && st.road[c] >= 2, "square {c}: a trunk road on {} with road {}", if st.world.land[c] { "land" } else { "water" }, st.road[c]);
        assert!(st.hw.trunk[c] >= 1909 && st.hw.trunk[c] <= 1935);
    }
    let W = st.world.W;
    for r in &st.hw.routes {
        for p in r.cells.windows(2) {
            let (a, b) = (p[0] as usize, p[1] as usize);
            if st.hw.trunk[a] == NOT || st.hw.trunk[b] == NOT {
                continue;
            }
            if let Some(li) = st.xs.table.between(W, a, b) {
                let l = &st.xs.table.links[li];
                assert!(l.n == 0 || st.xs.built[li] != crossings::NOT_BUILT, "the trunk road {} - {} fords between {a} and {b}", r.a, r.b);
            }
        }
    }
    // the treasury paid for what was built, and the works spent it on the towns' stone and men
    let paid: f64 = st.hw.years.iter().map(|y| y.paid).sum();
    let spent: f64 = st.hw.years.iter().map(|y| y.spent).sum::<f64>() + st.hw.now.spent;
    assert!(paid > 0.0 && spent > 0.0 && spent <= paid + st.hw.now.paid + 1e-6 * paid, "paid {paid}, spent {spent}");
}
