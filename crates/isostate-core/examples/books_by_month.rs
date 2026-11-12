//! What the books (docs/rules/accounts.md) cost a game that is played a month
//! at a time: one island under the game's rules, stepped as a live game steps
//! it (`sim::run_month` and a `live::live_month` twelve times, then
//! `sim::end_year` and a `live::live_year`), with the books on and with them
//! off, turn about. It says how long a month takes, how long a year's end,
//! how long the two messages, how much of that the books' own clock counts,
//! and how long a copy of the island takes (an order given in the middle of a
//! year is tried on one: `save::Game::give`).
//!
//!     cargo run --release -p isostate-core --example books_by_month -- [--seed 7] [--families 6]
//!         [--end-year 1900] [--passes 2]
#![allow(non_snake_case)]
use std::time::Instant;

use isostate_core::*;

/// One game's times, in seconds, with the year each was taken in.
#[derive(Default)]
struct Took {
    /// January to December: the month's step alone.
    months: Vec<(i64, f64)>,
    /// The year's end, after December.
    ends: Vec<(i64, f64)>,
    live_month: Vec<(i64, f64)>,
    live_year: Vec<(i64, f64)>,
    /// The books' own clock: in the twelve months of each year, and in closing it.
    books_months: Vec<(i64, f64)>,
    books_close: Vec<(i64, f64)>,
    /// A copy of the island, at a few years.
    copies: Vec<(i64, f64)>,
    /// What the copy took of the books, in megabytes; the works there were; and
    /// the time a copy of the books alone took.
    copied: Vec<(i64, f64, usize, f64)>,
    /// What the books hold as the game ends, in megabytes.
    kept: f64,
    check: String,
}

fn play(seed: u64, families: usize, end_year: Option<i64>, books: bool) -> Took {
    let mut P = save::params(Rules::Game, seed, families);
    P.end_year = end_year;
    P.accounts.enabled = books;
    let mut s = sim::start(&P, Rules::Game.content());
    let mut cursor = live::LiveCursor::new(&s.content);
    let mut t = Took::default();
    while s.year <= s.end_year {
        let year = s.year;
        let b0 = s.st.acct.seconds;
        for month in 0..12 {
            let t0 = Instant::now();
            sim::run_month(&mut s, month).expect("money is conserved every month");
            t.months.push((year, t0.elapsed().as_secs_f64()));
            let t0 = Instant::now();
            let m = live::live_month(&mut cursor, &s.st, &s.content, &s.params, &s.events, s.t - 1);
            t.live_month.push((year, t0.elapsed().as_secs_f64()));
            std::hint::black_box(m);
        }
        let b1 = s.st.acct.seconds;
        let t0 = Instant::now();
        sim::end_year(&mut s);
        t.ends.push((year, t0.elapsed().as_secs_f64()));
        t.books_months.push((year, b1 - b0));
        t.books_close.push((year, s.st.acct.seconds - b1));
        let t0 = Instant::now();
        let y = live::live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        t.live_year.push((year, t0.elapsed().as_secs_f64()));
        std::hint::black_box(y);
        if year % 25 == 0 && year > s.content.start_year {
            // the quickest of three
            let quickest = (0..3)
                .map(|_| {
                    let t0 = Instant::now();
                    let copy = s.st.clone();
                    let took = t0.elapsed().as_secs_f64();
                    std::hint::black_box(copy);
                    took
                })
                .fold(f64::INFINITY, f64::min);
            t.copies.push((year, quickest));
            // and of the books alone
            let books = (0..3)
                .map(|_| {
                    let t0 = Instant::now();
                    let copy = s.st.acct.clone();
                    let took = t0.elapsed().as_secs_f64();
                    std::hint::black_box(copy);
                    took
                })
                .fold(f64::INFINITY, f64::min);
            t.copied.push((year, s.st.acct.copied_bytes() as f64 / 1e6, s.st.F_count, books));
        }
    }
    let h = &s.st.acct.hist;
    t.kept = (4 * (h.goods.len() + h.land.len() + h.methods.len() + h.towns.len() + h.needs.len() + h.works.len() + h.works_id.len()) + 20 * (h.trade.len() + h.uses.len())) as f64 / 1e6;
    t.check = save::check(&s);
    t
}

/// The mean and the middle of the times taken in years `from..=to`, in milliseconds.
fn of(v: &[(i64, f64)], from: i64, to: i64) -> (f64, f64) {
    let mut x: Vec<f64> = v.iter().filter(|(y, _)| (from..=to).contains(y)).map(|&(_, s)| 1e3 * s).collect();
    if x.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    x.sort_by(|a, b| a.total_cmp(b));
    (x.iter().sum::<f64>() / x.len() as f64, x[x.len() / 2])
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seed, mut families, mut end_year, mut passes) = (7u64, 6usize, None, 2usize);
    for pair in args.chunks(2) {
        let value = pair.get(1).expect("a value after the option");
        match pair[0].as_str() {
            "--seed" => seed = value.parse().unwrap(),
            "--families" => families = value.parse().unwrap(),
            "--end-year" => end_year = Some(value.parse().unwrap()),
            "--passes" => passes = value.parse().unwrap(),
            other => panic!("unknown option {other}"),
        }
    }
    println!("island {seed}, {families} families, the game's rules, stepped by the month; milliseconds, mean (middle)");
    let mut checks = Vec::new();
    for pass in 1..=passes {
        let (off, on) = (play(seed, families, end_year, false), play(seed, families, end_year, true));
        let (y0, y1) = (on.ends[0].0, on.ends[on.ends.len() - 1].0);
        println!("\npass {pass}");
        println!("{:>12} {:>7}  {:>15} {:>15}  {:>15} {:>15}  {:>13} {:>13}", "years", "books", "a month", "the year's end", "live_month", "live_year", "books, month", "books, close");
        let mut eras: Vec<(i64, i64)> = (0..).map(|k| (y0 + 50 * k, (y0 + 50 * k + 49).min(y1))).take_while(|&(a, _)| a <= y1).collect();
        if eras.len() > 1 {
            eras.push((y0, y1));
        }
        for &(a, b) in &eras {
            for (name, t) in [("off", &off), ("on", &on)] {
                let cell = |v: &[(i64, f64)]| {
                    let (mean, mid) = of(v, a, b);
                    format!("{mean:7.2} ({mid:5.2})")
                };
                // the books' clock in a year's twelve months, as milliseconds a month
                let hooks = of(&t.books_months, a, b).0 / 12.0;
                // (the month's message in microseconds: it is a few hundred numbers)
                let message = of(&t.live_month, a, b);
                let message = format!("{:5.1} ({:4.1}) us", 1e3 * message.0, 1e3 * message.1);
                println!("{:>12} {:>7}  {:>15} {:>15}  {:>15} {:>15}  {:>13.3} {:>13.3}", format!("{a}-{b}"), name, cell(&t.months), cell(&t.ends), message, cell(&t.live_year), hooks, of(&t.books_close, a, b).0);
            }
            let (m_on, m_off, e_on, e_off) = (of(&on.months, a, b).0, of(&off.months, a, b).0, of(&on.ends, a, b).0, of(&off.ends, a, b).0);
            println!("{:>12} {:>7}  {:>+15.2} {:>+15.2}   a year of months and its end: {:.0} ms with the books, {:.0} without ({:+.1} per cent)", "", "on-off", m_on - m_off, e_on - e_off, 12.0 * m_on + e_on, 12.0 * m_off + e_off, 100.0 * ((12.0 * m_on + e_on) / (12.0 * m_off + e_off) - 1.0));
        }
        let longest = |t: &Took| t.months.iter().map(|&(_, s)| 1e3 * s).fold(0.0, f64::max);
        println!("the longest month: {:.1} ms with the books, {:.1} without", longest(&on), longest(&off));
        let copies: Vec<String> = on.copies.iter().zip(&off.copies).map(|(&(y, a), &(_, b))| format!("{y}: {:.1} with, {:.1} without", 1e3 * a, 1e3 * b)).collect();
        println!("a copy of the island, ms: {}", copies.join("; "));
        let copied: Vec<String> = on.copied.iter().map(|&(y, mb, works, took)| format!("{y}: {mb:.2} MB ({works} works), {:.2} ms", 1e3 * took)).collect();
        println!("of which the books, all but the years put away: {}", copied.join("; "));
        println!("the books hold {:.1} MB as the game ends", on.kept);
        checks.push((off.check, on.check));
    }
    assert!(checks.iter().all(|(off, on)| off == on && *on == checks[0].1), "the games were not the same game: {checks:?}");
    println!("\nthe same island every time, with the books and without (the save's check {})", checks[0].1);
}
