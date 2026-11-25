//! The people's ages, reckoned beneath the played totals (not in the Python;
//! docs/rules/ages.md): they sum to every town's people at every year's end,
//! they change nothing in the run they watch, and they are nothing while off.
//!
//! The runs here are played under the game's rules as they stand; nothing in
//! them leans on one island's history beyond its being an island that runs,
//! so no newer rule's switch needs to be set by name.
#![allow(non_snake_case)]

use isostate_core::ages::{self, GROUPS};
use isostate_core::*;

fn params(rules: Rules, seed: u64, end_year: i64, watch: bool) -> Params {
    let mut P = rules.params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end_year);
    P.ages.watch = watch;
    // played with the state's purse off (docs/rules/state.md), which reads the
    // ages for its pupils and pensioners and gives cohorts their schooling: the
    // ages are a watcher only without it, and its series come after theirs
    P.state.enabled = false;
    // and with who works by age off (docs/rules/who-works.md), which counts the
    // hands from the ages after 1900 and adds its series after theirs
    P.who_works.enabled = false;
    P
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

#[test]
fn the_groups_sum_to_every_towns_people_at_every_years_end() {
    // island 3 to 1830, year by year: in every town at every year's end the
    // seventeen groups hold its people, the rows needed no putting right beyond
    // rounding (every way people go was followed: births, deaths, sailings,
    // migrants, districts and new towns), and nobody is below nothing
    let P = params(Rules::Game, 3, 1830, true);
    let mut sim = sim::start(&P, Rules::Game.content());
    while sim.year <= sim.end_year {
        sim::run_year(&mut sim).unwrap();
        let st = &sim.st;
        assert_eq!(st.ages.rows.len(), st.T_max);
        for t in 0..st.T_max {
            let n = if st.town_active[t] { st.pop[t] } else { 0.0 };
            let s: f64 = st.ages.rows[t].iter().sum();
            assert!((s - n).abs() <= 1e-9 * n.max(1.0), "town {t} in {}: {s} for {n}", sim.year - 1);
            assert!(st.ages.rows[t].iter().all(|&v| v >= 0.0), "town {t} in {}: {:?}", sim.year - 1, st.ages.rows[t]);
        }
    }
    assert!(sim.st.ages.drift < 1e-9, "a row was {} out before it was put right", sim.st.ages.drift);
    assert_eq!(sim.st.ages.filled, 0, "a town had people and its row had none");
    let st = &sim.st;
    assert!((0..st.T_max).any(|t| st.town_active[t] && st.town_founded[t] > 1750), "a town was founded on the way");
    // and the year's figures are kept
    let last = sim.st.ages.years.last().unwrap();
    assert_eq!(last.year, 1830);
    assert!((last.under15 + last.working + last.over64 - 1.0).abs() < 1e-12);
    assert!(last.births_k > 20.0 && last.births_k < 45.0 && last.deaths_k > 10.0 && last.deaths_k < 35.0, "{last:?}");
    assert!(last.life > 30.0 && last.life < 60.0 && last.school >= 2.0 && last.school <= 5.0, "{last:?}");
    let town = (0..sim.st.T_max).max_by(|&a, &b| sim.st.pop[a].total_cmp(&sim.st.pop[b])).unwrap();
    let a = ages::town(&sim.st, town).unwrap();
    assert!((a.under15 + a.working + a.over64 - sim.st.pop[town]).abs() < 1e-6 * sim.st.pop[town]);
}

#[test]
fn a_run_is_the_same_with_the_ages_watched_or_not() {
    // every series but its own, the news and the people are bit for bit the same
    let off = sim::run_with(&params(Rules::Game, 7, 1780, false), Rules::Game.content(), false, true).unwrap();
    let on = sim::run_with(&params(Rules::Game, 7, 1780, true), Rules::Game.content(), false, true).unwrap();
    let (a, b) = (&off.state, &on.state);
    for (k, name) in a.series.names.iter().enumerate() {
        assert_eq!(bits(&a.series.data[k]), bits(b.series.get(name)), "series {name} moved");
    }
    assert_eq!(b.series.names.len(), a.series.names.len() + ages::N_SERIES, "the ages add their series after everyone's");
    assert!(b.series.names[a.series.names.len()..].iter().all(|n| n.starts_with("ages_") && accounts::own_series(n)));
    assert_eq!(serde_json::to_string(&off.events).unwrap(), serde_json::to_string(&on.events).unwrap());
    assert_eq!(bits(&a.pop), bits(&b.pop));
    assert_eq!(bits(&a.hh_cash), bits(&b.hh_cash));
    assert_eq!(bits(&a.price.data), bits(&b.price.data));
}

#[test]
fn off_under_the_port_and_on_in_the_game() {
    assert!(!default_params().ages.watch && game_params().ages.watch);
    let r = sim::run_with(&params(Rules::Port, 3, 1755, false), Rules::Port.content(), false, true).unwrap();
    assert!(r.state.ages.rows.is_empty() && r.state.ages.years.is_empty());
    assert!(r.state.series.names.iter().all(|n| !n.starts_with("ages_")));
    assert!(ages::facts(&r).is_empty());
    assert!(!serde_json::to_string(&r.params).unwrap().contains("ages"), "params.json of a port run does not mention the ages");
    // thrown on under the port, it reckons beneath the port's people too (a new town there
    // takes its people some way the watcher does not follow, and is given the island's shape)
    let r = sim::run_with(&params(Rules::Port, 3, 1760, true), Rules::Port.content(), false, true).unwrap();
    let st = &r.state;
    for t in 0..st.T_max {
        let n = if st.town_active[t] { st.pop[t] } else { 0.0 };
        assert!((st.ages.rows[t].iter().sum::<f64>() - n).abs() <= 1e-9 * n.max(1.0));
    }
    assert_eq!(st.ages.years.len(), 11);
    assert_eq!(st.ages.rows[0].len(), GROUPS);
}
