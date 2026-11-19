//! What the gauges (docs/rules/gauges.md) read off one island's books: a line
//! for every tenth year (or every year), and the cost system of one year good
//! by good.
//!
//!     cargo run --release -p isostate-core --example gauges -- [--seed 7] [--rules game]
//!         [--families 6] [--end-year Y] [--year 1850] [--every N] [--csv FILE]
//!
//! `--every 1` prints every year. `--csv FILE` writes every year's gauges to a
//! file as well, one row a year.
#![allow(non_snake_case)]
use isostate_core::gauges::GaugeYear;
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seed, mut rules, mut families, mut end_year, mut year, mut every, mut csv) = (7u64, Rules::Game, 6usize, None, 1850i64, 10i64, None::<String>);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seed" => seed = next().parse().unwrap(),
            "--rules" => rules = Rules::parse(&next()).expect("port or game"),
            "--families" => families = next().parse().unwrap(),
            "--end-year" => end_year = Some(next().parse().unwrap()),
            "--year" => year = next().parse().unwrap(),
            "--every" => every = next().parse().unwrap(),
            "--csv" => csv = Some(next()),
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let mut P = rules.params();
    P.seed = seed;
    P.n_families = families;
    P.end_year = end_year;
    P.accounts.enabled = true;
    P.gauges.enabled = true;
    let r = sim::run_with(&P, rules.content(), false, true).expect("money is conserved every month");
    let (st, C) = (&r.state, &r.content);
    let ys = &st.gauges.years;
    let cols = "year  gross   net  invest  spend basket bypay  floor  plant  mach  plant_lab  made   raw   all  hand  new   lost  fam_w fam_l  chain";
    println!("seed {seed}, {} rules, {families} families: the gauges, every {every} years\n{cols}", rules.as_str());
    let line = |g: &GaugeYear| {
        let fw: f64 = g.families_works.iter().sum();
        let fl: f64 = g.families_land.iter().sum();
        format!(
            "{}  {:.3} {:.3}  {:.3}  {:.3}  {:.3} {:.3}  {:.3}  {:.2}  {:.3}  {:.3}   {:5.2} {:5.2} {:5.2}  {:.2} {:.3}  {:.3}  {:.3} {:.3}  {:.1e}",
            g.year, g.work_gross, g.work_net, g.invest_share, g.spend_work, g.basket_work, g.spend_by_pay, g.floor, g.plant_years, g.labour_in_machinery,
            g.labour_in_plant, g.fork_made, g.fork_raw, g.fork_all, g.hand_hours, g.new_work, g.lost_services, fw, fl, g.chain_error
        )
    };
    for g in ys.iter().filter(|g| (g.year - C.start_year) % every == 0 || g.year == ys.last().unwrap().year) {
        println!("{}", line(g));
    }
    if let Some(path) = csv {
        let mut out = format!("{}\n", cols.split_whitespace().collect::<Vec<_>>().join(","));
        for g in ys {
            out.push_str(&line(g).split_whitespace().collect::<Vec<_>>().join(","));
            out.push('\n');
        }
        std::fs::write(&path, out).expect("the csv is written");
    }
    let Some(g) = ys.iter().find(|g| g.year == year) else { return };
    println!("\nthe cost system of {year}: of a unit of each good's price");
    println!("  good             value     wages  rent  rest  inputs  wear  | chain: wages  rent  rest  | lost  hand hours");
    for c in &g.goods {
        let inp: f64 = c.inputs.iter().filter(|v| v.is_finite()).sum();
        println!(
            "  {:<14} {:>9.0}    {:.3} {:.3} {:6.3}  {:.3} {:.3}  |        {:.3} {:.3} {:6.3}  | {:.3}  {:.3}",
            C.good_name[c.good], c.value, c.wages, c.rent, c.rest, inp, c.wear, c.chain_wages, c.chain_rent, c.chain_rest, c.lost, c.hand_hours
        );
    }
    println!("\nworks' plant worn out {:.0}, the floor's work at today's prices {:.3}", g.wear, g.presence_today);
}
