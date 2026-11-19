//! The guards that must never be fooled: the month's money check (`sim::run_month`)
//! and the books' balances (`accounts`). A value that is not a number, or an
//! infinity, compares false with everything, so a check written as "stop if the
//! drift is too large" lets it through. Each case here plants one where a check
//! looks and sees the check refuse it, saying where it was.
//!
//! Played under the port's rules, with the books switched on where they are read:
//! the checks are the same under either set of rules, and nothing here depends on
//! an island's history beyond its first year.
#![allow(non_snake_case)]

use isostate_core::*;

fn started(books: bool) -> sim::Sim {
    let mut P = Rules::Port.params();
    P.seed = 3;
    P.end_year = Some(1751);
    P.accounts.enabled = books;
    sim::start(&P, Rules::Port.content())
}

/// A town that is on the map in 1750.
fn a_town(s: &sim::Sim) -> usize {
    (0..s.st.T_max).find(|&t| s.st.town_active[t] && s.st.pop[t] > 0.0).expect("a town in 1750")
}

/// Play the first month, which must pass, then plant a value with `plant` and play
/// the second, which must not.
fn refused(plant: impl FnOnce(&mut sim::Sim)) -> String {
    let mut s = started(false);
    sim::run_month(&mut s, 0).expect("the first month passes the money check");
    plant(&mut s);
    match sim::run_month(&mut s, 1) {
        Ok(()) => panic!("the money check let the planted value through"),
        Err(e) => {
            eprintln!("refused: {e}");
            e
        }
    }
}

#[test]
fn the_money_check_refuses_a_value_that_is_not_a_number() {
    // in the hand's account, which only the check reads
    let e = refused(|s| s.st.by_hand.money = f64::NAN);
    assert!(e.contains("the hand's account"), "{e}");
    // in the count of money issued
    let e = refused(|s| s.st.issued = f64::NAN);
    assert!(e.contains("money issued"), "{e}");
    // two infinities, whose difference is not a number
    let e = refused(|s| {
        s.st.issued = f64::INFINITY;
        s.st.by_hand.money = f64::INFINITY;
    });
    assert!(e.contains("money issued") || e.contains("the hand's account"), "{e}");
    // in a town's households' purse
    let e = refused(|s| {
        let t = a_town(s);
        s.st.hh_cash[t] = f64::NAN;
    });
    assert!(e.contains("not a number"), "{e}");
}

#[test]
fn the_money_check_says_which_purse_is_not_a_number() {
    let mut s = started(false);
    sim::run_month(&mut s, 0).expect("the first month passes the money check");
    let t = a_town(&s);
    for (what, plant) in [
        ("households' cash", (|st: &mut state::State, t: usize| st.hh_cash[t] = f64::NAN) as fn(&mut state::State, usize)),
        ("landlords' cash", |st, t| st.lord_cash[t] = f64::INFINITY),
        ("merchants' cash", |st, t| st.merch_cash[t] = f64::NEG_INFINITY),
        ("the fund", |st, _| st.fund_cash = f64::NAN),
    ] {
        let mut st = s.st.clone();
        plant(&mut st, t);
        st.money_now = econ::money_total(&st);
        let e = sim::check_money(&st, s.year, 1).expect_err(what);
        eprintln!("refused: {e}");
        assert!(e.contains(what), "{what}: {e}");
    }
    // and the state as it was passes
    s.st.money_now = econ::money_total(&s.st);
    sim::check_money(&s.st, s.year, 1).expect("the island as it stands passes");
}

#[test]
fn the_books_do_not_read_a_value_that_is_not_a_number_as_balanced() {
    // the goods: a town's closing stock that is not a number leaves its balance
    // not a number, which the books must keep as the year's worst, at that town and good
    let mut s = started(true);
    for month in 0..12 {
        sim::run_month(&mut s, month).expect("money is conserved");
    }
    let t = a_town(&s);
    let g = 0;
    s.st.inv[(t, g)] = f64::NAN;
    let (C, P) = (s.content.clone(), s.params.clone());
    accounts::close_year(&mut s.st, &C, &P, s.year);
    let y = s.st.acct.years.last().expect("the year's books");
    assert!(!(y.balance_goods < 1e-9), "a stock that is not a number reads as balanced: {}", y.balance_goods);
    assert_eq!(y.balance_goods_at, (t, g), "the books say where");

    // the incomes: what a town's households are paid that is not a number
    let mut s = started(true);
    for month in 0..11 {
        sim::run_month(&mut s, month).expect("money is conserved");
    }
    let T = s.st.T_max;
    let mut hh_inc = vec![0.0; T];
    hh_inc[a_town(&s)] = f64::NAN;
    let zero = vec![0.0; T];
    accounts::close_month(&mut s.st, &[], &[], &[], &zero, &hh_inc, &zero, &zero, &[], 0.0, 0.0, 1.0);
    let (C, P) = (s.content.clone(), s.params.clone());
    accounts::close_year(&mut s.st, &C, &P, s.year);
    let y = s.st.acct.years.last().expect("the year's books");
    assert!(!(y.balance_income < 1e-9), "an income that is not a number reads as balanced: {}", y.balance_income);
}
