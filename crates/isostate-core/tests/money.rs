//! New money issued on the island's true total (not in the Python;
//! docs/rules/money.md). Under the rule the money as it stands is the money
//! the island began with grown at `money.growth`, a twelfth of it a month,
//! from the first month of the warm-up: its level as well as its rate, on
//! every island alike and whatever it began with. Each test fails with the
//! switch off, which is the port's count. With the switch off the golden test
//! (tests/golden.rs) covers that nothing changed.
#![allow(non_snake_case)]

use isostate_core::money::{self, MoneyParams};
use isostate_core::*;

fn params(seed: u64, true_total: bool, families: usize) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = families;
    P.money.true_total = true_total;
    P
}

/// The money at the top of every recorded month from January 1750, and at the
/// end of the last, for `years` years; `scale` multiplies every purse the
/// island begins with, before the warm-up.
fn money_by_month(P: &Params, years: i64, scale: f64) -> (f64, Vec<f64>) {
    let C = game_content();
    let (mut st, rngs) = init_state(P, &C);
    if scale != 1.0 {
        for v in [&mut st.hh_cash, &mut st.lord_cash, &mut st.merch_cash, &mut st.f_cash] {
            v.iter_mut().for_each(|x| *x *= scale);
        }
    }
    let m_init = econ::money_total(&st);
    let mut sim = sim::start_from(P, C, st, rngs);
    let mut money = vec![econ::money_total(&sim.st)];
    for _ in 0..years {
        for month in 0..12 {
            sim::run_month(&mut sim, month).expect("money is conserved every month");
            money.push(sim.st.money_now);
        }
        sim::end_year(&mut sim);
    }
    (m_init, money)
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs().max(1e-300)
}

#[test]
fn the_switch_is_off_under_the_port_and_on_in_the_game() {
    let (port, game) = (default_params(), game_params());
    assert_eq!(port.money, MoneyParams::default());
    assert!(!port.money.true_total && game.money.true_total);
    assert_eq!(game.money.growth, 0.0175);
    // the port's rate is the port's number, to the last bit; the game's is its own
    assert_eq!(money::rate(&port).to_bits(), port.money_growth.to_bits());
    assert_eq!(money::rate(&game), 0.0175);
    // a port run's params.json does not mention it
    assert!(!serde_json::to_string(&port).unwrap().contains("\"money\""));
}

#[test]
fn the_level_of_money_is_what_the_island_began_with_grown_at_the_rate() {
    // every month from the first of the warm-up: the money is the money the
    // island began with times (1 + growth / 12) to the power of the months gone
    let years = 6;
    for families in [0, 6] {
        let P = params(3, true, families);
        let g = P.money.growth / 12.0;
        let (m_init, money) = money_by_month(&P, years, 1.0);
        let warm = 12 * P.warmup_years as i32;
        for (k, &m) in money.iter().enumerate() {
            let want = m_init * (1.0 + g).powi(warm + k as i32);
            assert!(rel(m, want) < 1e-9, "{families} families, month {k} of the record: money {m:.6e}, the rule says {want:.6e}");
        }
    }
    // with the switch off, the port's count falls short of its own 2 per cent
    // and lands nowhere in particular: the level is not the rule's
    let P = params(3, false, 0);
    let (m_init, money) = money_by_month(&P, years, 1.0);
    let warm = 12 * P.warmup_years as i32;
    let at_2 = m_init * (1.0 + P.money_growth / 12.0).powi(warm + 12 * years as i32);
    let at_175 = m_init * (1.0 + 0.0175 / 12.0f64).powi(warm + 12 * years as i32);
    let last = *money.last().unwrap();
    assert!(last < at_2 * (1.0 - 1e-3), "the port's count issues less than its setting says");
    assert!(rel(last, at_175) > 1e-6, "and its level is not the rule's");
}

#[test]
fn the_rate_is_1_764_per_cent_a_year_on_every_island() {
    let P = params(1, true, 6);
    let (_, money) = money_by_month(&P, 4, 1.0);
    let want = (1.0 + 0.0175 / 12.0f64).powi(12) - 1.0;
    assert!((want - 0.01764).abs() < 5e-6, "0.0175 issued monthly is 1.764 per cent a year ({want})");
    for y in 0..4 {
        let rate = money[12 * (y + 1)] / money[12 * y] - 1.0;
        assert!((rate - want).abs() < 1e-9, "year {y}: {rate}");
    }
    // the same money on two islands, month for month: each begins with the
    // same purses for the same people, and nothing but the rule moves the total
    let (_, other) = money_by_month(&params(2, true, 6), 4, 1.0);
    for (k, (a, b)) in money.iter().zip(&other).enumerate() {
        assert!(rel(*a, *b) < 1e-9, "month {k}: {a:.9e} on island 1 and {b:.9e} on island 2");
    }
    // the port's count: a different rate on each island, and short of its 2 per cent
    let (_, one) = money_by_month(&params(1, false, 6), 4, 1.0);
    let (_, two) = money_by_month(&params(2, false, 6), 4, 1.0);
    let r = |v: &[f64]| (v[48] / v[0]).powf(0.25) - 1.0;
    assert!(r(&one) < 0.0199 && r(&two) < 0.0199, "{} and {}", r(&one), r(&two));
    assert!(rel(one[48], two[48]) > 1e-9, "the port's money differs between islands");
}

#[test]
fn twice_the_money_at_the_start_is_twice_the_money_in_every_month() {
    // the critic's test of the level as well as the rate (docs/designs/rules-pass.md,
    // 10.9): an island that begins with every purse doubled has, under the
    // rule, exactly twice the money in every month after, whatever its prices
    // and wages then do. Whether anything real moves with the level is
    // section 7, task 1, after the freight follows the wage; this holds the
    // money's side.
    let P = params(4, true, 0);
    let (a0, one) = money_by_month(&P, 4, 1.0);
    let (b0, two) = money_by_month(&P, 4, 2.0);
    assert!(rel(b0, 2.0 * a0) < 1e-12, "every purse doubled: {b0:.9e} against {a0:.9e}");
    for (k, (a, b)) in one.iter().zip(&two).enumerate() {
        assert!(rel(*b, 2.0 * a) < 1e-9, "month {k}: {b:.9e} against twice {a:.9e}");
    }
    // the port's count leaves out a share of the money that moves with what
    // the purses do, so twice the start is not twice the money
    let P = params(4, false, 0);
    let (_, one) = money_by_month(&P, 4, 1.0);
    let (_, two) = money_by_month(&P, 4, 2.0);
    assert!(rel(*two.last().unwrap(), 2.0 * one.last().unwrap()) > 1e-9);
}

#[test]
fn the_issue_is_never_below_nothing() {
    // money that only grows cannot go below nothing (tr-money.md, section 2):
    // every month's issue is a share of a positive total. The machines' extreme
    // setting is the case: once island 7's machines become fifty times better,
    // a month's income for the households can be larger than the money, and
    // the port's count then issues less than nothing; in November 1845 the
    // money is below nothing and the check stops the island. (It was March to
    // October 1766 before towns stopped hiring hands they do not have,
    // docs/rules/hands.md, which slowed the road and did not close it. The
    // shelves going with the district, docs/rules/shelves.md, give the island
    // another history, in which the port's count has not gone below nothing
    // by 1846, so the case is played with that rule off. So are the two mends
    // of 13 November, no stock markup on services (docs/rules/prices.md, "Goods
    // that spoil") and land priced on the rent it paid (docs/rules/harvest.md,
    // "The price of land"), which give it other histories again.)
    fn extreme(true_total: bool) -> Params {
        let mut P = params(7, true_total, 6);
        P.machines.quality_b = 2.0;
        P.machines.quality_cap = 50.0;
        P.machines.quality_Q0 = 0.0;
        P.people.shelves_go = false;
        P.prices.no_spoil_markup = false;
        P.harvest.rent_as_paid = false;
        P.end_year = Some(1846);
        P
    }
    let P = extreme(true);
    let (_, money) = money_by_month(&P, 1846 - 1750 + 1, 1.0);
    assert!(money.windows(2).all(|w| w[1] > w[0]), "the money grows every month");
    let port = sim::run_with(&extreme(false), game_content(), false, true);
    assert!(port.is_err(), "the port's count takes island 7's money below nothing in 1845");
}

#[test]
fn what_the_player_does_moves_money_between_purses_and_makes_none() {
    // the player's family takes up a trade, and at the turn of each of its
    // first three years founds a works and buys land: what it pays goes to other
    // purses, so the money of every month is still the money the island began
    // with grown at the rule's rate
    let P = params(2, true, 6);
    let C = game_content();
    let (st, rngs) = init_state(&P, &C);
    let m_init = econ::money_total(&st);
    let mut sim = sim::start_from(&P, C, st, rngs);
    let g = P.money.growth / 12.0;
    let warm = 12 * P.warmup_years as i32;
    let (mut founded, mut bought, mut k) = (0, 0, 0);
    for _ in 0..3 {
        // a trade other than the estates, so that there is a works to found
        for d in families::pending(&sim.st, 0) {
            let pick = d.options.iter().position(|o| matches!(o.choice, families::Choice::Trade { group } if group != families::Group::Estates));
            let Sim { st, content, params, events, .. } = &mut sim;
            let _ = families::decide(st, content, params, 0, d.id, pick.unwrap_or(d.default), events);
        }
        let seat = sim.st.families[0].seat;
        let cells: Vec<usize> = (0..sim.st.N).filter(|&c| sim.st.catch[c] == seat as i64).collect();
        let mut options: Vec<(f64, usize, usize)> = Vec::new();
        for &c in &cells {
            let Ok(opts) = families::site_options(&sim.st, &sim.content, &sim.params, 0, c) else { continue };
            for o in opts {
                if let (Ok(()), Ok(x)) = (&o.allowed, &o.sums) {
                    options.push((x.capital_cost, o.method, c));
                }
            }
        }
        // the dearest works the family can pay for, so that a good part of its purse moves
        options.retain(|o| o.0 <= sim.st.families[0].cash);
        options.sort_by(|a, b| b.0.total_cmp(&a.0));
        let before = econ::money_total(&sim.st);
        let Sim { st, content, params, events, year, .. } = &mut sim;
        if options.iter().any(|&(_, m, c)| families::found_works(st, content, params, 0, m, c, *year, events).is_ok()) {
            founded += 1;
        }
        if cells.iter().any(|&c| st.cell_owner[c] < 0 && families::buy_land(st, content, params, 0, c).is_ok()) {
            bought += 1;
        }
        assert!(rel(econ::money_total(&sim.st), before) < 1e-12, "the orders made or lost money");
        for month in 0..12 {
            sim::run_month(&mut sim, month).expect("money is conserved every month");
            k += 1;
            let want = m_init * (1.0 + g).powi(warm + k);
            assert!(rel(sim.st.money_now, want) < 1e-9, "month {k}: money {:.9e}, the rule says {want:.9e}", sim.st.money_now);
        }
        sim::end_year(&mut sim);
    }
    assert!(founded > 0 && bought > 0, "the case is gone: the family founded {founded} works and bought land {bought} times");
}
