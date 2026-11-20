//! The university (not in the Python; docs/rules/state.md, "The university"):
//! founded by the book in the island's seat five years after the first schools
//! act, with students who move to it and are townsfolk there, staff and rooms
//! the treasury pays for, the books still balancing and no money lost; and one
//! founded and closed by hand.
//!
//! The runs are played under the game's rules as they stand, with the switches
//! the case depends on set by name: the purse's (`state.enabled`, on the default
//! book), the university's (`state.university`) and the ages' (`ages.watch`),
//! which the students are counted from. The book's first schools act leans on
//! the island only in that its towns hold more than three in ten of its people
//! by 1900, which every island of sixteen does.
#![allow(non_snake_case)]

use isostate_core::edits::{self, Edit};
use isostate_core::purse::Book;
use isostate_core::*;

fn params(seed: u64, end_year: i64) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end_year);
    P.ages.watch = true;
    P.state.enabled = true;
    P.state.book = Book::Default;
    P.state.university = true;
    P
}

#[test]
fn the_book_founds_a_university_in_the_seat_and_its_students_come() {
    // island 5 to 1910: the seat chosen at the end of 1900, when the first acts
    // are met; the university founded there at the end of 1905 and news in
    // January 1906; from then students, staff and rooms, paid for; the treasury
    // still within one and three months; the books balance; money is checked
    // every month (`run_with`)
    let r = sim::run_with(&params(5, 1910), Rules::Game.content(), false, true).unwrap_or_else(|e| panic!("{e}"));
    let st = &r.state;
    let u = &st.purse.uni;
    let seat = u.seat.expect("a seat");
    assert_eq!(u.seat_year, 1900);
    assert_eq!(u.unis.len(), 1, "{:?}", u.unis);
    let v = &u.unis[0];
    assert_eq!((v.town, v.founded, v.by_hand, v.closed), (seat, 1905, false, None));
    let name = format!("The University of {}", st.town_name[seat]);
    assert!(r.events.iter().any(|e| e.kind == "state" && e.title == name && e.year > 1906.0 && e.year < 1906.1), "its founding is news in January 1906");
    for y in &u.years {
        if y.year < 1905 {
            assert!(y.students == 0.0 && y.staff == 0.0 && y.share == 0.0, "{y:?}");
            continue;
        }
        assert!(y.share > 0.01 && y.share < 0.03 && y.intake > 0.0 && y.moved > 0.0 && y.moved < y.intake, "{y:?}");
        if y.year > 1905 {
            assert!(y.students > 0.0 && y.staff > 0.0 && y.staff_pay > 0.0 && y.rooms_paid > 0.0 && y.rooms > 0.0, "{y:?}");
            assert!((y.staff - 0.15 * y.students).abs() < 1e-6 * y.students, "{y:?}");
        }
    }
    // three years of intake make the students, who live in the seat and are its townsfolk
    assert_eq!(v.intake.len(), 3);
    assert!(u.students[seat] > 0.0 && st.urban_pop[seat] > u.students[seat]);
    for p in st.purse.years.iter().filter(|p| p.year > 1900) {
        assert!(p.months_held >= 1.0 && p.months_held <= 3.0 && p.short_months == 0, "{}: {} months held", p.year, p.months_held);
    }
    let books = st.acct.years.iter().find(|y| y.year == 1908).expect("the books of 1908");
    assert!(books.balance_income < 1e-9 && books.balance_spending < 1e-9, "{} {}", books.balance_income, books.balance_spending);
}

#[test]
fn a_university_is_founded_and_closed_by_hand() {
    // island 5: at the turn of 1903 a university founded by hand in the largest
    // town that is not the seat; it takes its students at the end of 1903, the
    // book founds the seat's in 1905, and the hand closes its own at the turn of
    // 1907, after which its town has no students
    let P = params(5, 1908);
    let mut sim = sim::start(&P, Rules::Game.content());
    sim.check_money = true;
    while sim.year < 1903 {
        sim::run_year(&mut sim).unwrap();
    }
    let seat = sim.st.purse.uni.seat.expect("a seat");
    let t = (0..sim.st.T_max).filter(|&t| sim.st.town_active[t] && t != seat).max_by(|&a, &b| sim.st.pop[a].total_cmp(&sim.st.pop[b])).unwrap();
    let found = Edit::FoundUniversity { cell: sim.st.town_node[t] };
    edits::carry_out(&mut sim, &found).unwrap();
    assert!(edits::carry_out(&mut sim, &found).unwrap_err().contains("has a university already"));
    assert!(edits::carry_out(&mut sim, &Edit::CloseUniversity { town: seat }).unwrap_err().contains("no university open"));
    while sim.year < 1907 {
        sim::run_year(&mut sim).unwrap();
    }
    let u = &sim.st.purse.uni;
    assert_eq!(u.unis.len(), 2, "{:?}", u.unis);
    assert!(u.unis[0].by_hand && u.unis[0].town == t && u.unis[0].founded == 1903);
    assert!(!u.unis[1].by_hand && u.unis[1].town == seat && u.unis[1].founded == 1905);
    assert!(u.students[t] > 0.0 && u.students[seat] > 0.0);
    edits::carry_out(&mut sim, &Edit::CloseUniversity { town: t }).unwrap();
    assert_eq!(sim.st.purse.uni.unis[0].closed, Some(1907));
    sim::run_year(&mut sim).unwrap();
    let u = &sim.st.purse.uni;
    assert_eq!(u.students[t], 0.0);
    assert!(u.students[seat] > 0.0);
}

#[test]
fn with_the_university_off_the_purse_has_its_seat_and_nothing_more() {
    let mut P = params(5, 1906);
    P.state.university = false;
    let r = sim::run_with(&P, Rules::Game.content(), false, true).unwrap_or_else(|e| panic!("{e}"));
    let u = &r.state.purse.uni;
    assert!(u.seat.is_some());
    assert!(u.unis.is_empty() && u.years.is_empty() && u.students.is_empty());
    assert!(!r.state.series.index.contains_key("state_students"));
}
