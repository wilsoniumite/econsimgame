//! Who works, by age (not in the Python; docs/rules/who-works.md): off is off;
//! to 1900 nothing moves, and from 1901 the hands are counted by age; and
//! nobody is counted twice, the university's students least of all.
//!
//! The runs are played under the game's rules as they stand, with the rule's
//! switch and those of the rules its case depends on set by name: the ages'
//! (`ages.watch`, which it counts from), the purse's (`state.enabled`, on the
//! default book, whose first school act and first pension come at the end of
//! 1900 on every island) and the university's (`state.university`, founded by
//! the book five years after the first school act).
#![allow(non_snake_case)]

use isostate_core::*;

fn params(seed: u64, end_year: i64, on: bool) -> Params {
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end_year);
    P.ages.watch = true;
    P.state.enabled = true;
    P.state.university = true;
    P.who_works.enabled = on;
    P
}

fn run(P: &Params) -> sim::RunResult {
    sim::run_with(P, Rules::Game.content(), false, true).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn off_is_off_and_to_1900_nothing_moves() {
    // island 5 to 1903, the rule off and on: every series both keep is the same
    // to the bit in every month to December 1900, and so is the news; from 1901
    // the hands differ. Off, the rule keeps nothing and has no series; on, its
    // series are nought to 1900 and its count of 1900 is about the half
    let off = run(&params(5, 1903, false));
    let on = run(&params(5, 1903, true));
    assert!(off.state.who_works.years.is_empty() && off.state.who_works.share.is_empty());
    assert!(!off.state.series.index.keys().any(|n| n.starts_with("work_")));
    let to_1900 = (1900 - 1750 + 1) * 12;
    let (a, b) = (&off.state.series, &on.state.series);
    let mut compared = 0;
    let mut moved_after = 0;
    for (name, &i) in &a.index {
        let j = b.index[name];
        let same = a.data[i][..to_1900].iter().zip(&b.data[j][..to_1900]).all(|(x, y)| x.to_bits() == y.to_bits());
        assert!(same, "{name} differs before 1901");
        compared += 1;
        moved_after += (a.data[i][to_1900..].iter().zip(&b.data[j][to_1900..]).any(|(x, y)| x.to_bits() != y.to_bits())) as usize;
    }
    assert!(compared > 100 && moved_after > 10, "{compared} compared, {moved_after} moved after 1900");
    for (name, &j) in &b.index {
        if name.starts_with("work_") {
            assert!(b.data[j][..to_1900].iter().all(|&v| v == 0.0), "{name} is not nothing to 1900");
            assert!(b.data[j][to_1900..].iter().all(|&v| v > 0.0 || name == "work_under15"), "{name} is nothing after 1900");
        }
    }
    let news = |r: &sim::RunResult| -> Vec<(u64, String)> { r.events.iter().filter(|e| e.year < 1901.0).map(|e| (e.year.to_bits(), e.title.clone())).collect() };
    assert_eq!(news(&off), news(&on));
    // the count of 1900, counted at the end of 1899, is about the half it plays
    let y1900 = on.state.who_works.years.iter().find(|v| v.year == 1900).expect("a count for 1900");
    let share = y1900.hands / y1900.people;
    assert!((share - 0.5).abs() < 0.015, "the count of 1900 is {share}");
    assert_eq!(y1900.played, 0.5);
    // 1901 plays a tenth of the way to the count
    let y1901 = on.state.who_works.years.iter().find(|v| v.year == 1901).unwrap();
    assert!((y1901.blend - 0.1).abs() < 1e-12 && (y1901.played - (0.9 * 0.5 + 0.1 * y1901.hands / y1901.people)).abs() < 1e-12);
}

#[test]
fn nobody_is_counted_twice() {
    // island 5 to 1912: the university is founded in 1905 and the rule is in
    // whole from 1910. Every year from 1901: the count's students are the
    // university's; nobody counted works more than the people of an age; the
    // hands the university still holds for its students are the part not yet
    // counted by age, so none once the rule is in whole
    let r = run(&params(5, 1912, true));
    let w = &r.state.who_works;
    let mut seen_students = false;
    for v in w.years.iter().filter(|v| v.year > 1900 && v.year <= 1912) {
        for k in 0..4 {
            assert!(v.at_work[k] >= 0.0 && v.at_work[k] <= v.of_age[k], "{}: band {k} {} at work of {}", v.year, v.at_work[k], v.of_age[k]);
        }
        assert!(v.hands <= 0.6 * v.people, "{v:?}");
        seen_students |= v.students > 0.0;
    }
    assert!(seen_students, "no year had students");
    // the last year counted is 1913's, for the year after the run; 1912's students
    // are the university's of 1912
    let students: f64 = r.state.purse.uni.students.iter().sum();
    let last = w.years.last().unwrap();
    assert!(students > 0.0 && (last.students - students).abs() < 1e-6 * students, "{} counted, {} at the university", last.students, students);
    // in 1912 the rule is in whole: the university holds no student out of the hands
    let held: f64 = r.state.purse.uni.held.iter().sum();
    assert_eq!(held, 0.0, "the university held {held} hands for students the count has out already");
    let staff: f64 = r.state.purse.uni.staff.iter().sum();
    assert!(staff > 0.0, "its staff are still hired out of the hands");
}
