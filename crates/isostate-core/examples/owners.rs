//! Who holds one island's works, and who is paid by them (docs/rules/owners.md):
//! a line for every tenth year (or every year).
//!
//!     cargo run --release -p isostate-core --example owners -- [--seed 7] [--rules game]
//!         [--families 6] [--end-year Y] [--every N] [--off] [--csv FILE]
//!
//! The columns: the shares of the works' book held by families, landlords and
//! households; the landlords' share of the year's dividends of all works;
//! their dividends over their whole income; the works standing; and the
//! largest gap from one of any works' shares over the whole run. `--off` plays the island with
//! the rule off (its shares are then not kept, and only the works are shown).
//! `--csv FILE` writes every year as well, one row a year.
#![allow(non_snake_case)]
use isostate_core::owners::OwnersYear;
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seed, mut rules, mut families, mut end_year, mut every, mut csv, mut off) = (7u64, Rules::Game, 6usize, None, 10i64, None::<String>, false);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seed" => seed = next().parse().unwrap(),
            "--rules" => rules = Rules::parse(&next()).expect("port or game"),
            "--families" => families = next().parse().unwrap(),
            "--end-year" => end_year = Some(next().parse().unwrap()),
            "--every" => every = next().parse().unwrap(),
            "--csv" => csv = Some(next()),
            "--off" => {
                off = true;
                i += 1;
                continue;
            }
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let mut P = rules.params();
    P.seed = seed;
    P.n_families = families;
    P.end_year = end_year;
    P.owners.enabled = !off;
    // the works standing each year, counted as the run goes
    let mut s = sim::start(&P, rules.content());
    let mut works: Vec<(i64, usize)> = Vec::new();
    while s.year <= s.end_year {
        let y = s.year;
        sim::run_year(&mut s).expect("money is conserved every month");
        works.push((y, (0..s.st.F_count).filter(|&f| s.st.f_alive[f]).count()));
    }
    let r = sim::finish(s);
    let (st, C) = (&r.state, &r.content);
    let cols = "year  families  lords     hh  div_lords  lords_from_works  works  worst";
    println!("seed {seed}, {} rules, {families} families, owners {}: who holds the works, every {every} years\n{cols}", rules.as_str(), if off { "off" } else { "on" });
    let by_year = |y: i64| st.owners.years.iter().find(|o| o.year == y);
    let line = |y: i64, n: usize| {
        let nan = OwnersYear { year: y, book: [f64::NAN; 3], div: [f64::NAN; 3], lords_income: f64::NAN };
        let o = by_year(y).unwrap_or(&nan);
        let book: f64 = o.book.iter().sum();
        let div: f64 = o.div.iter().sum();
        format!(
            "{y}     {:.3}  {:.3}  {:.3}      {:.3}             {:.3}  {n:5}  {:.1e}",
            o.book[0] / book,
            o.book[1] / book,
            o.book[2] / book,
            o.div[1] / div,
            o.div[1] / o.lords_income,
            st.owners.worst
        )
    };
    let last = works.last().map_or(C.start_year, |w| w.0);
    for &(y, n) in works.iter().filter(|w| (w.0 - C.start_year) % every == 0 || w.0 == last) {
        println!("{}", line(y, n));
    }
    if let Some(path) = csv {
        let mut out = format!("{}\n", cols.split_whitespace().collect::<Vec<_>>().join(","));
        for &(y, n) in &works {
            out.push_str(&line(y, n).split_whitespace().collect::<Vec<_>>().join(","));
            out.push('\n');
        }
        std::fs::write(&path, out).expect("the csv is written");
    }
}
