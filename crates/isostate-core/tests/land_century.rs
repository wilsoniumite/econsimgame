//! The land's twentieth century: oil made from coal, the tractor and the cutter
//! (docs/rules/tasks.md), nitrogen and breeding (docs/rules/machines.md,
//! "Nitrogen and breeding"). All of it is stage 6 of the game's overlay, and
//! with `tree.stage = 1` none of it is laid: tests/power.rs holds the island at
//! stage 1 to its fingerprint from before this stage, and the port's golden
//! test that nothing reaches the port.
#![allow(non_snake_case)]

use isostate_core::machines;
use isostate_core::mat::Mat;
use isostate_core::sim::{self, Sim};
use isostate_core::*;

fn game(stage: u32, seed: u64, end_year: i64) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end_year);
    P.tree.stage = stage;
    P
}

/// January 1750 at a stage of the tree, and its first active town.
fn at_start(stage: u32) -> (Sim, usize) {
    let P = game(stage, 3, 1751);
    let s = sim::start(&P, tree::content(&P, game_content()).unwrap());
    let town = (0..s.st.T_max).find(|&t| s.st.town_active[t]).expect("a town");
    (s, town)
}

fn sector(C: &Content, name: &str) -> usize {
    C.sector_name.iter().position(|n| n == name).unwrap_or_else(|| panic!("no sector {name}"))
}

/// The land machines' table: with `tree.stage = 1` its old rows only, however
/// much every town knows; with the content after 1900 nitrogen on the farms
/// and breeding on the farms and pastures, known as the nitrogen works and the
/// invention are known, with fixed boosts that do not grow with the machines'
/// quality.
#[test]
fn the_land_machines_table_has_its_old_rows_at_stage_one() {
    for stage in [1, 6] {
        let (mut s, town) = at_start(stage);
        let (C, P) = (s.content.clone(), s.params.clone());
        for k in 0..s.st.know.cols {
            s.st.know[(town, k)] = 1.0;
        }
        let kinds = |s: &Sim, name: &str| -> Vec<&'static str> { machines::uses(&s.st, &C, &P, sector(&C, name), town, &s.st.price).iter().map(|u| u.kind).collect() };
        if stage == 1 {
            assert_eq!(kinds(&s, "farm"), ["farm machinery"]);
            assert!(kinds(&s, "pasture").is_empty());
        } else {
            assert_eq!(kinds(&s, "farm"), ["farm machinery", "nitrogen", "breeding"]);
            assert_eq!(kinds(&s, "pasture"), ["breeding"]);
            // the boosts are fixed: the same with the machines of 1750 and with the best
            let boosts = |s: &Sim| -> Vec<f64> { machines::uses(&s.st, &C, &P, sector(&C, "farm"), town, &s.st.price).iter().filter(|u| u.improves).map(|u| u.boost).collect() };
            let before = boosts(&s);
            assert_eq!(before, [machines::NITROGEN.1, machines::BREEDING.1]);
            s.st.method_Q.iter_mut().for_each(|q| *q = 1e12);
            assert!(machines::quality(&s.st, &C, &P) > 2.0);
            assert_eq!(boosts(&s), before);
        }
        // and nothing else buys anything but machinery and rails at stage 1
        for k in 0..C.NS {
            let more = machines::bought_more(&s.st, &C, &P, k, town, &s.st.price);
            assert!(stage == 6 || more.is_empty(), "stage {stage}: {} buys {more:?}", C.sector_name[k]);
        }
    }
}

/// Nitrogen multiplies what the farm makes with its machines, and the farms
/// that keep it buy chemicals for their whole output.
#[test]
fn nitrogen_multiplies_the_crop_and_is_paid_for_in_chemicals() {
    let (mut s, town) = at_start(6);
    let (C, P) = (s.content.clone(), s.params.clone());
    let farm = sector(&C, "farm");
    let nitrogen = C.tech_name.iter().position(|n| n == machines::NITROGEN.0).unwrap();
    let gc = C.gi("chemicals");
    s.st.machine_stock = Mat::default(); // no month counted yet: the improvements standing are those kept up
    // chemicals for nothing: every farm that knows keeps it
    let mut price = s.st.price.clone();
    price[(town, gc)] = 1e-9;
    let without = econ::sector_tfp(&s.st, &C, &P, farm, town, &price, &s.st.fill);
    s.st.know[(town, nitrogen)] = 0.5;
    let u = machines::uses(&s.st, &C, &P, farm, town, &price).into_iter().find(|u| u.kind == "nitrogen").expect("nitrogen is known");
    assert!((u.kept() - 0.5).abs() < 1e-12);
    let with = econ::sector_tfp(&s.st, &C, &P, farm, town, &price, &s.st.fill);
    assert!((with.0 / without.0 - (1.0 + machines::NITROGEN.1 * 0.5)).abs() < 1e-12, "{} over {}", with.0, without.0);
    // half the farms keep it, each buying for its own output (1.6 times an
    // unimproved farm's) while the town's average farm makes 1.3 times
    let more = machines::bought_more(&s.st, &C, &P, farm, town, &price);
    assert_eq!(more.len(), 1);
    assert_eq!(more[0].0, gc);
    let own = (1.0 + machines::NITROGEN.1) / (1.0 + 0.5 * machines::NITROGEN.1);
    assert!((more[0].1 - machines::NITROGEN.2 * 0.5 * own).abs() < 1e-12);
    // dearer than it earns, nobody keeps it and nothing is bought
    price[(town, gc)] = 1e9;
    assert!(machines::bought_more(&s.st, &C, &P, farm, town, &price).is_empty() || machines::bought_more(&s.st, &C, &P, farm, town, &price)[0].1 == 0.0);
}

/// Island 7 under the game's rules to 1935, played a year at a time (money is
/// checked every month in `run_year`): no farm, pasture or wood means to hold
/// any of its hauling before the tractor is found, nor any pit before the
/// cutter, and both come to hold some once they are found. This leans on the
/// island's history under every rule of the game as it is (oil made from coal
/// before 1930 on island 7, and electrical goods enough for the cutter); if a
/// later rule moves the finds past 1935 this test must be set again.
#[test]
fn the_land_waits_for_its_machines() {
    let P = game(6, 7, 1935);
    let mut s = sim::start(&P, tree::content(&P, game_content()).unwrap());
    let (tractor, cutter) = (1, 2);
    assert_eq!(s.content.tasks.uses[tractor].name, "tractor");
    assert_eq!(s.content.tasks.uses[cutter].name, "cutter");
    let (mut farm_most, mut pit_most): (f64, f64) = (0.0, 0.0);
    while s.year <= s.end_year {
        let year = s.year;
        sim::run_year(&mut s).expect("money is conserved every month");
        let (st, C) = (&s.st, &s.content);
        let found = |u: usize| st.tasks.found.get(u).is_some_and(|&y| y >= 0 && y <= year);
        for row in &st.tasks.land_auto {
            for (k, auto) in row.iter().enumerate() {
                let x = auto[tasks::HAUL];
                if C.tasks.uses[tractor].on_sector[k] {
                    assert!(found(tractor) || x == 0.0, "{year}: {} holds {x} before the tractor", C.sector_name[k]);
                    farm_most = farm_most.max(x);
                }
                if C.tasks.uses[cutter].on_sector[k] {
                    assert!(found(cutter) || x == 0.0, "{year}: {} holds {x} before the cutter", C.sector_name[k]);
                    pit_most = pit_most.max(x);
                }
            }
        }
    }
    assert!(s.st.tasks.found[tractor] >= 1915 && s.st.tasks.found[cutter] >= 1920, "found {:?}", s.st.tasks.found);
    assert!(farm_most > 0.05 && pit_most > 0.05, "by 1935 the fields mean to hold {farm_most} of their hauling and the pits {pit_most}");
    let last = s.st.tasks.years.last().unwrap();
    assert!(last.nitrogen_out > 0.0, "no farm keeps nitrogen by 1935");
}
