//! What the `canals` rules (docs/rules/canals.md) do in one run: every canal dug
//! with what it runs to, what it cost and what was expected of it, then the
//! network and what it carried decade by decade, beside the railway.
//!
//!     cargo run --release -p isostate-core --example canals -- [SEED] [--rules port|game]
//!         [--families N] [--end-year Y] [--set NAME=VALUE]... [--quiet]
//!
//! The rules default to the port with only the canals switched on, which is how
//! the sweep measures them alone; `--rules game` runs them in company. `--set`
//! takes any parameter by its dotted name (`canals.wharf=0.6`,
//! `railways.enabled=true`). `--quiet` leaves the list of canals out.
#![allow(non_snake_case)]
use isostate_core::ways::{SERVES_TOWN, SERVES_TRUNK, serves_word};
use isostate_core::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut rules = Rules::Port;
    let (mut seed, mut families, mut end_year, mut quiet) = (7u64, 0usize, None, false);
    let mut sets: Vec<(String, String)> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--rules" => {
                rules = Rules::parse(&args[i + 1]).expect("--rules port|game");
                i += 1;
            }
            "--families" => {
                families = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--end-year" => {
                end_year = Some(args[i + 1].parse().unwrap());
                i += 1;
            }
            "--set" => {
                let (k, v) = args[i + 1].split_once('=').expect("--set NAME=VALUE");
                sets.push((k.to_string(), v.to_string()));
                i += 1;
            }
            "--quiet" => quiet = true,
            s => seed = s.parse().expect("the seed is a number"),
        }
        i += 1;
    }
    let mut P = rules.params();
    P.canals.enabled = true;
    P.seed = seed;
    P.n_families = families;
    P.end_year = end_year;
    for (k, v) in &sets {
        // a dotted name becomes nested JSON: canals.wharf=0.6 is {"canals": {"wharf": 0.6}}
        let mut json = v.clone();
        for part in k.rsplit('.') {
            json = format!("{{\"{part}\": {json}}}");
        }
        P = P.with_json_overrides(&json).unwrap_or_else(|e| panic!("--set {k}={v}: {e}"));
    }
    let C = rules.content();
    let r = sim::run_with(&P, C, false, true).unwrap_or_else(|e| panic!("{e}"));
    let (st, C) = (&r.state, &r.content);
    let y0 = C.start_year;
    let series = |name: &str| st.series.index.get(name).map_or(&[][..], |&k| &st.series.data[k][..]);
    let at = |name: &str, y: i64| observer::annual(series(name), y0, y);

    println!("seed {seed}, {} rules, {families} families, canals {}", rules.as_str(), if P.canals.enabled { "on" } else { "off" });
    let book = &st.cn.book;
    match st.cn.found {
        Some((year, town)) => println!("a canal is first surveyed in {year}, at {}", st.town_name[town]),
        None => println!("nobody surveys a canal"),
    }
    if !quiet {
        println!("\nbegun  opened  km  serves                      expected    spent  return");
        for l in &book.lines {
            let what = match l.serves {
                SERVES_TRUNK => format!("{} to {}", st.town_name[l.a], st.town_name[l.b]),
                SERVES_TOWN => format!("{} (its first)", st.town_name[l.a]),
                s => format!("{} near {}", serves_word(s), st.town_name[l.a]),
            };
            println!("{}  {:6.1}  {:3.0}  {:<26} {:>9.0} {:>8.0}  {:5.2}", l.start, l.opened, l.km, what, l.cost, l.spent, l.roi);
        }
    }
    println!("\n      canal  cells   rail  digging  navvies  boat Mt-km  haul km  fares/capital  | of what the boats carry: coal  stone+clay  grain  timber  ore+iron  works'");
    let good = |name: &str| C.good_index.get(name).copied();
    let sector = |name: &str| C.sector_name.iter().position(|n| n == name);
    for y in (y0 + 10..=y0 + r.rec.n_years as i64 - 1).step_by(10) {
        let note = book.years.iter().find(|n| n.year == y);
        let ys: Vec<&canals::CanalYear> = st.cn.years.iter().filter(|c| c.year > y - 10 && c.year <= y).collect();
        let n = ys.len().max(1) as f64;
        let sum = |f: &dyn Fn(&canals::CanalYear) -> f64| ys.iter().map(|c| f(c)).sum::<f64>() / n;
        let all = sum(&|c| c.trade.iter().sum::<f64>() + c.land.iter().sum::<f64>() + c.works);
        let part = |goods: &[&str], sectors: &[&str]| {
            let v = sum(&|c| goods.iter().filter_map(|g| good(g)).map(|g| c.trade[g]).sum::<f64>() + sectors.iter().filter_map(|s| sector(s)).map(|s| c.land[s]).sum::<f64>());
            if all > 0.0 { v / all } else { f64::NAN }
        };
        let (tc, boarded) = (sum(&|c| c.ton_cells), sum(&|c| c.boarded));
        let capital: f64 = book.lines.iter().filter(|l| l.opened < (y + 1) as f64).map(|l| l.spent * at("cpi", y) / at("cpi", (l.opened.floor() as i64).max(y0))).sum();
        println!(
            "{y}  {:5.0}  {:>5}  {:5.0}  {:>7}  {:7.0}  {:10.2}  {:7.1}  {:13.3}  | {:28.2}  {:10.2}  {:5.2}  {:6.2}  {:8.2}  {:6.2}",
            note.map_or(0.0, |n| n.km),
            ys.last().map_or(0, |c| c.busy_cells),
            at("rail_km", y),
            note.map_or(0, |n| n.building),
            sum(&|c| c.navvies),
            tc * 2.0 / 1e6,
            if boarded > 0.0 { tc * 2.0 / boarded } else { f64::NAN },
            note.map_or(f64::NAN, |n| if capital > 0.0 { n.fares / capital } else { f64::NAN }),
            part(&["coal"], &["coalmine"]),
            part(&["stone", "clay"], &["quarry", "claypit"]),
            part(&["grain"], &["farm"]),
            part(&["timber"], &["forestry"]),
            part(&["iron_ore", "iron"], &["ironmine"]),
            if all > 0.0 { sum(&|c| c.works) / all } else { f64::NAN },
        );
    }
    println!("\n(cells: cells of canal that carried anything in the decade's last year. works': what the works move beyond the land's own produce.)");
    let news: Vec<&Event> = r.events.iter().filter(|e| e.kind == "canal" || e.kind == "canal_plan").collect();
    println!("\n{} news items about canals; the first five:", news.len());
    for e in news.iter().take(5) {
        println!("  {:.1} [{:.2}] {}. {}", e.year, e.weight, e.title, e.text);
    }
    for y in [1800, 1850, 1900].into_iter().filter(|&y| y < y0 + r.rec.n_years as i64) {
        println!("{y}: output per head {:.3}, real wage {:.3}", at("gdp_pc", y), at("real_wage", y));
    }
}
