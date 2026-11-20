//! The state's purse on one island (docs/rules/state.md): every year from the
//! first act, the rate on pay, tax, schools and pensions over income, the
//! treasury in months of its spending, the laws, the teachers and pensioners,
//! and the working ages' years of school; then the acts as they were met.
//!
//!     cargo run --release -p isostate-core --example purse -- --seed 7 --end-year 2050 \
//!         [--every 1] [--set state.book=none] [--csv FILE]
//!
//! `--set` takes what the sweep's does, one name a time. `--csv` writes every
//! year of the purse as a line.
#![allow(non_snake_case)]

use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let seed: u64 = arg("--seed").map_or(7, |v| v.parse().unwrap());
    let end: i64 = arg("--end-year").map_or(2050, |v| v.parse().unwrap());
    let every: i64 = arg("--every").map_or(1, |v| v.parse().unwrap());
    let mut P = game_params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = Some(end);
    for (k, a) in args.iter().enumerate() {
        if a == "--set" {
            let (name, value) = args[k + 1].split_once('=').expect("--set name=value");
            P = sweep::set_field(&P, name, &sweep::parse_value(value)).unwrap_or_else(|e| panic!("{e}"));
        }
    }
    let r = sim::run_with(&P, Rules::Game.content(), false, true).unwrap_or_else(|e| panic!("{e}"));
    let st = &r.state;
    println!("year  rate   tax/inc schools pensions held  law  level  age share cut short teachers  pensioners school(15-64)");
    let mut csv = String::from("year,rate,tax,schools,pensions,base,treasury,months_held,income,teachers,teachers_want,hands,pensioners,pupils,school_law,school_level,pension_age,pension_share,cut,short_months,school\n");
    for v in &st.purse.years {
        let school = st.ages.years.iter().find(|a| a.year == v.year).map_or(f64::NAN, |a| a.school);
        let inc = |x: f64| if v.income > 0.0 { x / v.income } else { f64::NAN };
        csv += &format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            v.year, v.rate, v.tax, v.schools, v.pensions, v.base, v.treasury, v.months_held, v.income, v.teachers, v.teachers_want, v.hands, v.pensioners, v.pupils, v.school_law, v.school_level, v.pension_age, v.pension_share, v.cut, v.short_months, school
        );
        if v.year < 1899 || (v.year - 1900) % every != 0 && v.year != end {
            continue;
        }
        println!(
            "{} {:.3}  {:.4}  {:.4}  {:.4}  {:5.2} {:4.1} {:5.2} {:4.0} {:.2}  {:.2} {:3} {:9.0} {:10.0} {:5.2}",
            v.year, v.rate, inc(v.tax), inc(v.schools), inc(v.pensions), v.months_held, v.school_law, v.school_level, v.pension_age, v.pension_share, v.cut, v.short_months, v.teachers, v.pensioners, school
        );
    }
    println!("\nacts:");
    for a in &st.purse.acts {
        println!("  {} {} {} {}", a.year, a.what, a.at, a.share);
    }
    // the seat and the university (docs/rules/state.md, "The university")
    let u = &st.purse.uni;
    let name = |t: usize| st.town_name.get(t).cloned().unwrap_or_default();
    match u.seat {
        Some(s) => println!("
seat: {} (town {s}), chosen {}", name(s), u.seat_year),
        None => println!("
seat: none"),
    }
    for v in &u.unis {
        println!("  university of {} (town {}), founded {}{}{}", name(v.town), v.town, v.founded, if v.by_hand { " by hand" } else { "" }, v.closed.map_or(String::new(), |y| format!(", closed {y}")));
    }
    println!("year  share  students  staff   staff_pay   rooms_paid   rooms    been   intake   moved");
    for v in u.years.iter().filter(|v| v.share > 0.0 || v.students > 0.0) {
        if (v.year - 1900) % every != 0 && v.year != end {
            continue;
        }
        println!("{} {:.3} {:9.0} {:6.0} {:11.0} {:12.0} {:8.0} {:.3} {:8.0} {:7.0}", v.year, v.share, v.students, v.staff, v.staff_pay, v.rooms_paid, v.rooms, v.been, v.intake, v.moved);
    }
    if let Some(path) = arg("--csv") {
        std::fs::write(&path, csv).unwrap();
        println!("wrote {path}");
    }
}
