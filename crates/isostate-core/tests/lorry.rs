//! The lorry (not in the Python; docs/rules/lorry.md): a use of `data/tasks.toml`
//! on the carting behind `tasks.lorry`, waiting on the motor vehicle, an idea of
//! the tree (`data/game.toml`); the lorry's saving had by the road under it
//! (`lorry.roads`); and from 1901 no made road unmade for want of traffic
//! (`lorry.keep`).
//!
//! The island runs are played on island 7 to 1935 under the game's rules with
//! six families, with the switches their cases lean on set by name
//! (`tasks.enabled`, `tasks.land`, `tasks.lorry`, `tasks.tending` off,
//! `carting.enabled`, `lorry.roads`, `lorry.keep`) and every other rule as the
//! game has it. What they check holds on any island's history (a run against
//! another run, shares within their bounds, money that adds up), but for one
//! thing: that island 7 finds the lorry by 1935, which the test says if a later
//! rule stops it.
#![allow(non_snake_case)]

use isostate_core::lorry::{self, LorryParams};
use isostate_core::tasks::{self, TasksParams};
use isostate_core::transport::ROAD_MULT;
use isostate_core::*;

const TO: i64 = 1935;

fn params(lorry: bool) -> Params {
    let mut P = game_params();
    P.seed = 7;
    P.n_families = 6;
    P.end_year = Some(TO);
    P.carting.enabled = true;
    P.tasks = TasksParams { enabled: true, land: true, tending: false, lorry, ..TasksParams::game() };
    P.lorry = LorryParams { roads: true, keep: true };
    P
}

/// The game's content as a run under `P` plays it (with the tree laid).
fn content(P: &Params) -> Content {
    isostate_core::tree::content(P, game_content()).expect("the game's tree lays")
}

/// The switches are off under the port's rules and left out of its file. The
/// lorry is laid with the content after 1900: it hauls for the carters alone,
/// from the easiest task to the whole of the class, waits on the motor vehicle
/// (an invention of the tree's stage 6 with a floor after 1900), and with its
/// switch off is never found and its idea never sought. The cost graph's land
/// squares are the port's until the lorry carts anything, and no road is kept
/// before 1901 or a track at any time.
#[test]
fn the_lorry_stands_behind_its_switches() {
    let port = default_params();
    assert!(!port.tasks.lorry && port.lorry == LorryParams::default());
    assert!(!port.lorry.roads && !port.lorry.keep);
    assert!(!serde_json::to_string(&port).unwrap().contains("lorry"), "the port's file says nothing of the lorry");
    let (on, off) = (params(true), params(false));
    assert!(serde_json::to_string(&on).unwrap().contains("\"lorry\":true"));
    assert!(tasks::USE_SWITCHES.contains(&"lorry"));
    let C = content(&on);
    let u = C.tasks.uses.iter().position(|u| u.name == "lorry").expect("the lorry is in the table");
    let us = &C.tasks.uses[u];
    assert_eq!(us.switch.as_deref(), Some("lorry"));
    assert_eq!((us.class, us.from, us.reach), (tasks::HAUL, 0.0, 1.0));
    assert!(us.on_carting && !us.on_works && us.on_sector.iter().all(|&b| !b), "the lorry hauls for the carters alone");
    let t = C.tech_name.iter().position(|n| n == "motor_vehicle").expect("the motor vehicle is an idea of the tree");
    assert_eq!(us.idea, Some(t));
    assert!(t >= C.M && !C.tech_known[t] && C.tech_after[t] >= 1901 && us.found_after >= 1901, "nothing is sought or found before 1901");
    assert!(us.allowed(&on) && !us.allowed(&off));
    assert!(tasks::idea_sought(&C, &on, t) && !tasks::idea_sought(&C, &off, t), "the motor vehicle is sought only with the lorry on");
    // the carters' sector is the only one the lorry reaches, and no other use reaches it
    assert!(C.tasks.uses.iter().filter(|v| v.on_carting).count() == 1);
    // the cost graph and the roads, before anything is carted by lorry
    let s = sim::start(&on, C.clone());
    assert_eq!(lorry::road_mult(&s.st, &on), ROAD_MULT, "no lorry has carted anything: the port's multipliers");
    assert!(!lorry::kept(&on, 1900, 3) && lorry::kept(&on, 1901, 2) && lorry::kept(&on, 1901, 3) && !lorry::kept(&on, 1950, 1));
    assert!(!lorry::kept(&port, 1950, 3));
    // without the content after 1900 neither the lorry nor its idea is laid
    let mut one = on.clone();
    one.tree.stage = 1;
    let C1 = content(&one);
    assert!(C1.tasks.uses.iter().all(|u| u.name != "lorry") && !C1.tech_name.iter().any(|n| n == "motor_vehicle"));
}

/// Island 7 to 1935, two runs side by side:
///
/// - with the lorry on: money is conserved every month (`run_year` checks);
///   the lorry is found in the year after the motor vehicle; no town's carting
///   means to hold more of its hauling than it can, nor holds more than it
///   means; an hour of the carting's work takes no more hands than by hand and
///   costs no more than the wage in any town; some town's carting comes to hold
///   part of its hauling, and the cost graph's land squares are cheaper the
///   year after; and the run is the run with the lorry off to the end of 1904,
///   the year before the motor vehicle's floor;
/// - with the lorry off: the motor vehicle is never found, and the cost graph
///   is the port's every year.
#[test]
fn island_7_to_1935() {
    let off = std::thread::spawn(move || {
        let P = params(false);
        let mut s = sim::start(&P, content(&P));
        let idea = s.content.tech_name.iter().position(|n| n == "motor_vehicle").unwrap();
        let mut at1904 = String::new();
        while s.year <= s.end_year {
            let year = s.year;
            sim::run_year(&mut s).expect("money is conserved every month");
            if year == 1904 {
                at1904 = save::check(&s);
            }
            assert_eq!(lorry::road_mult(&s.st, &P), ROAD_MULT, "{year}: the lorry off moved the cost graph");
        }
        assert!(s.st.tech_year[idea] < 0, "with the lorry off the motor vehicle was found");
        at1904
    });
    let P = params(true);
    let mut s = sim::start(&P, content(&P));
    let u = s.content.tasks.uses.iter().position(|u| u.name == "lorry").unwrap();
    let idea = s.content.tech_name.iter().position(|n| n == "motor_vehicle").unwrap();
    let (mut at1904, mut most, mut cheaper) = (String::new(), 0.0f64, false);
    while s.year <= s.end_year {
        let year = s.year;
        sim::run_year(&mut s).expect("money is conserved every month, through the find");
        if year == 1904 {
            at1904 = save::check(&s);
        }
        let st = &s.st;
        let found = st.tasks.found[u];
        if found >= 0 {
            assert!(st.tech_year[idea] >= 1905 && found == st.tech_year[idea] + 1, "{year}: the lorry found in {found}, its idea in {}", st.tech_year[idea]);
        }
        for t in 0..st.tasks.cart_auto.len() {
            let (x, h) = (st.tasks.cart_auto[t][tasks::HAUL], st.tasks.cart_held[t][tasks::HAUL]);
            assert!((0.0..=1.0 + 1e-12).contains(&x), "{year}: town {t}'s carting means to hold {x} of its hauling");
            assert!((0.0..=x + 1e-12).contains(&h), "{year}: town {t}'s carting holds {h} of its hauling, meaning {x}");
            if st.town_active[t] && st.tasks.cart_work[t] > 0.0 {
                let (hands, cost) = (st.tasks.cart_hands[t], st.tasks.cart_cost[t]);
                assert!(hands <= 1.0 + 1e-9 && hands > 0.0, "{year}: an hour of town {t}'s carting takes {hands} hands");
                assert!(cost >= 0.0 && cost.is_finite(), "{year}: town {t}'s lorries cost {cost} an hour");
                most = most.max(h);
            }
        }
        let m = lorry::road_mult(&st, &P);
        if m != ROAD_MULT {
            cheaper = true;
            for l in 0..4 {
                assert!(m[l] <= ROAD_MULT[l] && m[l] > 0.0, "{year}: a square of road level {l} costs {} by lorry, {} by cart", m[l], ROAD_MULT[l]);
            }
            assert!(m[0] == ROAD_MULT[0], "{year}: the lorry saves nothing on open ground under lorry.roads");
        }
    }
    let y = s.st.tasks.found[u];
    assert!(y >= 1906, "island 7 finds the lorry by {TO} (if a later rule stops that, this test must be set again)");
    assert!(most > 0.0, "no town's carting came to hold any of its hauling by {TO}, the lorry found in {y}");
    assert!(cheaper, "the lorry carted and the cost graph's land squares were never cheaper");
    eprintln!("the lorry found in {y}; the most a town's carting held of its hauling by {TO}: {most:.3}");
    assert_eq!(at1904, off.join().unwrap(), "the lorry on moved the run before 1905, though its idea is not sought before 1905");
}
