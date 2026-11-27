//! The hands that come with a district (`people.carry_hands`; docs/rules/people.md,
//! "The hands that come with a district"; not in the Python). In the first month
//! after the year's turn a town that took a district counts as hands for hire at
//! least the hands its land and its works carry into the month. These tests hold
//! that the switch is off on both starts and not written out, that the rule
//! raises a short supply to the hands carried and no further, that it reads its
//! towns once, and that on a real island the month after a turn has no town
//! that took a district short of hands it has. Each fails with the rule taken
//! out. With the switch off, tests/golden.rs and tests/golden_game.rs hold that
//! nothing changed.
#![allow(non_snake_case)]

use isostate_core::*;

#[test]
fn off_on_both_starts_and_not_written_out() {
    assert!(!default_params().people.carry_hands);
    assert!(!game_params().people.carry_hands);
    let mut P = game_params();
    landfall::set_start(&mut P, "landfall").unwrap();
    assert!(!P.people.carry_hands, "landfall leaves the year's turn as it was");
    for P in [default_params(), game_params()] {
        let j = serde_json::to_string(&P).unwrap();
        assert!(!j.contains("carry_hands"), "the parameters name the switch while it is off");
    }
    let mut P = game_params();
    P.people.carry_hands = true;
    assert!(serde_json::to_string(&P).unwrap().contains("\"carry_hands\":true"));
}

/// The hands each town's land and works carry into the coming month.
fn carried(st: &State) -> Vec<f64> {
    let mut c = vec![0.0; st.T_max];
    for u in 0..st.ex_town.len().min(st.ex_Lprev.len()) {
        c[st.ex_town[u]] += st.ex_Lprev[u];
    }
    for f in 0..st.F_count {
        if st.f_alive[f] {
            c[st.f_town[f]] += st.f_labor[f];
        }
    }
    c
}

/// The island of seed 3 played to the end of 1760 under the game's rules with
/// six families (`people.carry_hands` as given), ready for 1761's January.
fn island(carry: bool) -> sim::Sim {
    let mut P = game_params();
    P.seed = 3;
    P.n_families = 6;
    P.end_year = Some(1801);
    P.people.carry_hands = carry;
    let mut sim = sim::start(&P, game_content());
    while sim.year <= 1760 {
        for m in 0..12 {
            sim::run_month(&mut sim, m).expect("the month runs");
        }
        sim::end_year(&mut sim);
    }
    sim
}

#[test]
fn a_short_supply_is_raised_to_the_hands_carried_and_no_further() {
    let sim = island(false);
    let mut st = sim.st.clone();
    let mut P = sim.params.clone();
    let c = carried(&st);
    let towns: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t] && c[t] > 10.0).collect();
    assert!(towns.len() >= 2, "the island has towns with hands at work");
    let (short, long) = (towns[0], towns[1]);
    // the one town short of what it carries, the other long, and both took a district
    let mut supply = vec![0.0; st.T_max];
    supply[short] = 0.5 * c[short];
    supply[long] = 2.0 * c[long];
    let before = supply.clone();
    st.pp_took = (0..st.T_max).map(|t| t == short || t == long).collect();
    // off: nothing is read or changed
    peopling::carry_hands(&mut st, &P, &mut supply);
    assert_eq!(supply, before);
    assert!(!st.pp_took.is_empty(), "the towns are kept while the rule is off");
    // on: the short town has its carried hands, the long one keeps its own
    P.people.carry_hands = true;
    peopling::carry_hands(&mut st, &P, &mut supply);
    assert!((supply[short] - c[short]).abs() <= 1e-9 * c[short], "{} against {}", supply[short], c[short]);
    assert_eq!(supply[long], before[long]);
    assert!(st.pp_took.is_empty(), "the towns are forgotten once read");
    // and a second month reads nothing
    let mut again = before.clone();
    peopling::carry_hands(&mut st, &P, &mut again);
    assert_eq!(again, before);
}

#[test]
fn no_town_that_took_a_district_is_short_of_hands_it_has_in_january() {
    // played under the game's rules with only `people.carry_hands` set by name.
    // Most Januaries no town that took a district counts fewer hands for hire
    // than it carries (docs/rules/people.md: the newcomers' hands already count
    // as supply); so each January is played twice from the same island, once
    // with the rule and once without, until one is found where the rule binds
    let mut sim = island(true);
    let mut bound = 0;
    while sim.year <= 1800 && bound == 0 {
        let took: Vec<usize> = (0..sim.st.T_max).filter(|&t| sim.st.pp_took.get(t).copied().unwrap_or(false) && sim.st.town_active[t]).collect();
        let c = carried(&sim.st);
        let mut without = sim.clone();
        without.params.people.carry_hands = false;
        sim::run_month(&mut without, 0).expect("January runs");
        sim::run_month(&mut sim, 0).expect("January runs");
        assert!(sim.st.pp_took.is_empty(), "January read the list");
        for &t in &took {
            assert!(sim.st.month_stats.supply[t] >= c[t] * (1.0 - 1e-12), "{}, town {t}: {} for hire against {} carried", sim.year, sim.st.month_stats.supply[t], c[t]);
            if without.st.month_stats.supply[t] < c[t] * (1.0 - 1e-9) {
                bound += 1;
            }
        }
        for m in 1..12 {
            sim::run_month(&mut sim, m).expect("the month runs");
        }
        sim::end_year(&mut sim);
    }
    assert!(bound > 0, "no January to 1800 where a town that took a district was short of what it carries");
}
