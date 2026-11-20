//! The state's purse (not in the Python; docs/rules/state.md): with nothing in
//! it the run is the run without it; with the book it pays from 1901, keeps
//! its treasury within one and three months of its spending, loses no money
//! and keeps the books; and a chip overrides the book.
//!
//! The runs are played under the game's rules as they stand, with the purse's
//! switch and the ages' (which it counts its pupils and pensioners from) set by
//! name. The book's acts lean on the island only in that its towns hold more
//! than three in ten of its people and its over-64s more than 0.055 of them by
//! 1900, which every island of sixteen does (docs/rules/state.md).
#![allow(non_snake_case)]

use isostate_core::purse::Book;
use isostate_core::*;

fn params(seed: u64, end_year: i64, on: bool, book: Book) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end_year);
    P.ages.watch = true;
    P.state.enabled = on;
    P.state.book = book;
    P
}

fn run(P: &Params) -> sim::RunResult {
    sim::run_with(P, Rules::Game.content(), false, true).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn the_state_with_nothing_in_it_is_the_run_without_it() {
    // island 5 to 1906: every series both runs keep, to the bit, the news, and
    // the money; the purse's own series are nothing in every month
    let off = run(&params(5, 1906, false, Book::Default));
    let none = run(&params(5, 1906, true, Book::None));
    let (a, b) = (&off.state.series, &none.state.series);
    let mut compared = 0;
    for (name, &i) in &a.index {
        let j = b.index[name];
        let same = a.data[i].iter().zip(&b.data[j]).all(|(x, y)| x.to_bits() == y.to_bits());
        assert!(same, "{name} differs");
        compared += 1;
    }
    assert!(compared > 100);
    for (name, &j) in &b.index {
        if name.starts_with("state_") {
            assert!(b.data[j].iter().all(|&v| v == 0.0), "{name} is not nothing");
        }
    }
    assert_eq!(off.events.len(), none.events.len());
    for (x, y) in off.events.iter().zip(&none.events) {
        assert_eq!((x.year.to_bits(), &x.title), (y.year.to_bits(), &y.title));
    }
    assert_eq!(off.state.money_now.to_bits(), none.state.money_now.to_bits());
    assert_eq!(none.state.purse.treasury, 0.0);
    assert!(none.state.purse.acts.is_empty());
}

#[test]
fn the_book_pays_from_1901_and_the_treasury_balances() {
    // island 5 to 1910 on the default book: the first school act and the first
    // pension met at the end of 1900 and news in January 1901; every year from
    // 1901 the treasury ends within one and three months of its spending and was
    // never short; the money is checked every month (`run_with`); the books
    // still balance, with the teachers' pay in the wage bill and the spending
    let r = run(&params(5, 1910, true, Book::Default));
    let p = &r.state.purse;
    let acts: Vec<(i64, &str)> = p.acts.iter().map(|a| (a.year, a.what.as_str())).collect();
    assert!(acts.contains(&(1900, "schools")) && acts.contains(&(1900, "pension")), "{acts:?}");
    assert!(r.events.iter().any(|e| e.kind == "state" && e.year > 1901.0 && e.year < 1901.1), "the acts are news in January 1901");
    assert!(r.events.iter().all(|e| e.kind != "state" || e.year > 1901.0));
    for v in &p.years {
        if v.year <= 1900 {
            assert!(v.tax == 0.0 && v.schools == 0.0 && v.pensions == 0.0 && v.rate == 0.0, "{v:?}");
            continue;
        }
        assert!(v.schools > 0.0 && v.pensions > 0.0 && v.tax > 0.0, "{v:?}");
        assert!(v.months_held >= 1.0 && v.months_held <= 3.0, "{}: {} months held", v.year, v.months_held);
        assert_eq!(v.short_months, 0, "{v:?}");
        assert!(v.rate > 0.0 && v.rate < 0.5 && v.cut == 1.0, "{v:?}");
        // the programme grows toward its law a step a year
        assert!(v.school_level <= v.school_law && v.school_law == 6.0, "{v:?}");
    }
    let last = p.years.last().unwrap();
    assert_eq!(last.school_level, 6.0);
    assert!(last.teachers > 0.0 && (last.teachers - last.teachers_want).abs() < 1e-6 * last.teachers_want, "{last:?}");
    let books = r.state.acct.years.iter().find(|y| y.year == 1905).expect("the books of 1905");
    assert!(books.pay_state > 0.0 && books.pensions > 0.0 && books.tax_pay > 0.0);
    assert!(books.balance_income < 1e-9 && books.balance_spending < 1e-9, "{} {}", books.balance_income, books.balance_spending);
    assert!(books.money.treasury > 0.0);
    // the cohorts at school when the act came in are given more years than
    // their families bought only from the years they were at school under it
    let school = |y: i64| r.state.ages.years.iter().find(|a| a.year == y).unwrap().school;
    assert!(school(1910) > school(1900), "{} {}", school(1900), school(1910));
}

#[test]
fn a_chip_overrides_the_book_from_the_year_it_is_set() {
    // set from the start: nine years of school and no pension, whatever the book says
    let mut P = params(5, 1901, true, Book::Default);
    P.state.school_years = 9.0;
    P.state.pension_share = 0.0;
    let r = run(&P);
    let p = &r.state.purse;
    assert!(p.acts.iter().any(|a| a.what == "schools"), "the book still meets its act, and the chip stands over it");
    // a chip is not the book: set in 1750, it is the law from 1751
    for v in p.years.iter().filter(|v| v.year > 1750) {
        assert_eq!(v.school_law, 9.0);
        assert_eq!(v.pensions, 0.0);
        assert!(v.schools > 0.0);
    }
    // the programme's first year reaches the room a trade has, of the law of nine,
    // and the law in seven years
    let level = |y: i64| p.years.iter().find(|v| v.year == y).unwrap().school_level;
    assert!((level(1751) - 0.15 * 9.0).abs() < 1e-12, "{}", level(1751));
    assert!(level(1756) < 9.0 && level(1757) == 9.0);
}
