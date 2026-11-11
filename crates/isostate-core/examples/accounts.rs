//! What the books (docs/rules/accounts.md) say of a few islands: the island's
//! income shares at the fifty-year marks, the books' own checks, what they cost
//! in time, and one town's books for one good as a worked example.
//!
//!     cargo run --release -p isostate-core --example accounts -- [--seeds 1-8] [--rules port|game]
//!         [--families N] [--end-year Y] [--good iron] [--year 1850] [--town NAME_OR_INDEX]
//!         [--works] [--island] [--years] [--no-works-books]
//!
//! The seeds run side by side. The worked example is printed for the first seed:
//! the town that made the most of `--good` in `--year` unless `--town` says
//! which. `--works` adds the books of the largest works making that good there,
//! `--island` the whole income account of that year, `--years` a line for every
//! year of the first seed. `--no-works-books` keeps the towns' books only
//! (`accounts.works = false`), to see what the works' part costs.
#![allow(non_snake_case)]
use isostate_core::accounts::{self, IslandYear};
use isostate_core::*;

fn run(P: &Params, rules: Rules) -> (RunResult, f64) {
    let t0 = std::time::Instant::now();
    let r = sim::run_with(P, rules.content(), false, true).expect("money is conserved every month");
    (r, t0.elapsed().as_secs_f64())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut rules, mut families, mut end_year) = (vec![1u64, 2, 3, 4, 5, 6, 7, 8], Rules::Port, 6usize, None);
    let (mut good, mut year, mut town, mut show_works, mut show_island, mut show_years) = (String::from("iron"), 1850i64, None::<String>, false, false, false);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => {
                let v = next();
                seeds = match v.split_once('-') {
                    Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                    None => v.split(',').map(|s| s.parse().unwrap()).collect(),
                };
            }
            "--rules" => rules = Rules::parse(&next()).expect("port or game"),
            "--families" => families = next().parse().unwrap(),
            "--end-year" => end_year = Some(next().parse().unwrap()),
            "--good" => good = next(),
            "--year" => year = next().parse().unwrap(),
            "--town" => town = Some(next()),
            "--works" => show_works = true,
            "--island" => show_island = true,
            "--years" => show_years = true,
            "--no-works-books" => {}
            other => panic!("unknown option {other}"),
        }
        i += if matches!(args[i].as_str(), "--works" | "--island" | "--years" | "--no-works-books") { 1 } else { 2 };
    }
    let mut P = rules.params();
    P.n_families = families;
    P.end_year = end_year;
    P.accounts.enabled = true;
    P.accounts.works = !args.iter().any(|a| a == "--no-works-books");

    let runs: Vec<(RunResult, f64)> = std::thread::scope(|s| {
        let hs: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let mut P = P.clone();
                P.seed = seed;
                s.spawn(move || run(&P, rules))
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });

    println!("{} rules, {} families, seeds {:?}; the books on", rules.as_str(), families, seeds);
    let y0 = runs[0].0.content.start_year;
    let y1 = y0 + runs[0].0.rec.n_years as i64 - 1;
    let marks = facts::marks(y0, y1);
    let table: [(&str, fn(&IslandYear) -> f64); 9] = [
        ("work's share of income", |y| y.share_work),
        ("works' owners' share", |y| y.share_works),
        ("merchants' share", |y| y.share_merchants),
        ("land's owners' share", |y| y.share_land),
        ("household income that is pay", |y| y.pay_share),
        ("works, years of the wage bill", |y| y.works_years),
        ("land, years of the wage bill", |y| y.land_years),
        ("works and land, years", |y| y.ownership_years),
        ("families' share of works' value", |y| y.works_value_families / y.works_value),
    ];
    println!("\nmean over seeds [least .. greatest]");
    print!("{:32}", "");
    for y in &marks {
        print!("{:>26}", y);
    }
    println!();
    for (label, f) in table {
        print!("{label:32}");
        for &y in &marks {
            let v: Vec<f64> = runs.iter().map(|(r, _)| f(&r.state.acct.years[(y - y0) as usize])).collect();
            let (lo, hi) = (v.iter().copied().fold(f64::INFINITY, f64::min), v.iter().copied().fold(f64::NEG_INFINITY, f64::max));
            print!("{:>26}", format!("{:.3} [{:.3} .. {:.3}]", v.iter().sum::<f64>() / v.len() as f64, lo, hi));
        }
        println!();
    }

    println!("\nthe books' checks, worst year of each run, and what they cost");
    println!("seed   goods (share)  (units, town, good, year)        incomes     income=spending   oversold   books s   run s   share   MB kept");
    for ((r, secs), seed) in runs.iter().zip(&seeds) {
        let b = &r.state.acct;
        let worst = b.years.iter().max_by(|a, c| a.balance_goods.total_cmp(&c.balance_goods)).unwrap();
        let inc = b.years.iter().map(|y| y.balance_income).fold(0.0, f64::max);
        let spend = b.years.iter().map(|y| y.balance_spending).fold(0.0, f64::max);
        let kept: usize = b.hist.iter().map(|h| 4 * (h.goods.len() + h.land.len() + h.methods.len() + h.towns.len() + h.needs.len() + h.works.len()) + 4 * h.works_id.len() + 16 * h.trade.len()).sum();
        println!(
            "{seed:4}   {:.2e}       {:+.3e} {:>10} {:>9} {}   {:.2e}    {:.2e}        {:.2e}  {:6.2}  {:6.1}  {:.4}  {:6.1}",
            worst.balance_goods,
            worst.balance_goods_units,
            r.state.town_name[worst.balance_goods_at.0],
            r.content.good_name[worst.balance_goods_at.1],
            worst.year,
            inc,
            spend,
            b.years.iter().map(|y| y.oversold_share).fold(0.0, f64::max),
            b.seconds,
            secs,
            b.seconds / secs,
            kept as f64 / 1e6
        );
    }
    let years = runs[0].0.rec.n_years as f64;
    let mean_books = runs.iter().map(|(r, _)| r.state.acct.seconds).sum::<f64>() / runs.len() as f64;
    println!("the books take {:.1} ms a simulated year on average", 1000.0 * mean_books / years);

    // ---- the worked example, on the first seed
    let (r, _) = &runs[0];
    let (st, C) = (&r.state, &r.content);
    if show_years {
        println!("
seed {}, year by year", seeds[0]);
        println!("year   work  works  merch   land  pay share  works yrs  land yrs     wage bill         rents  landlords' cash  towns");
        for (k, y) in st.acct.years.iter().enumerate() {
            let towns = (0..st.T_max).filter(|&t| st.acct.hist[k].towns[t * accounts::KT] > 0.0).count();
            println!(
                "{}  {:5.3}  {:5.3}  {:5.3}  {:5.3}      {:5.3}      {:5.2}     {:5.2}  {:12.0}  {:12.0}     {:12.0}  {:5}",
                y.year, y.share_work, y.share_works, y.share_merchants, y.share_land, y.pay_share, y.works_years, y.land_years, y.wage_bill, y.rent_farm + y.rent_pits + y.rent_sites + y.rent_town, y.money.landlords, towns
            );
        }
        // the year the land's owners did worst, town by town
        let (k, y) = st.acct.years.iter().enumerate().min_by(|a, b| a.1.share_land.total_cmp(&b.1.share_land)).unwrap();
        println!("
{}, the year land's share was lowest ({:.3}), town by town", y.year, y.share_land);
        println!("{:>14}  founded   people  mean wage  hands on land  wages on land  value of produce  rents of land", "");
        for t in 0..st.town_name.len() {
            let Some(tb) = accounts::town_books(st, C, t, Some(y.year)) else { continue };
            let fig = |name: &str| tb.figures.iter().find(|(n, _)| n == name).map_or(f64::NAN, |(_, v)| *v);
            if fig("pop") > 0.0 {
                let value: f64 = tb.land.iter().map(|l| l.value).sum();
                println!("{:>14}  {:7}  {:7.0}  {:9.2}  {:13.0}  {:13.0}  {:15.0}  {:13.0}", tb.name, st.town_founded[t], fig("pop"), fig("wage"), fig("hands_land"), fig("pay_land"), value, fig("rent_land"));
            }
        }
        let _ = k;
    }
    let g = C.gi(&good);
    let year = year.clamp(y0, y1);
    let k = (year - y0) as usize;
    let (T, G) = (st.T_max, C.G);
    let t = match &town {
        Some(name) => name.parse().unwrap_or_else(|_| st.town_name.iter().position(|n| n == name).expect("no town of that name")),
        None => (0..st.town_name.len()).max_by(|&a, &b| st.acct.hist[k].goods[(a * G + g) * accounts::KG].total_cmp(&st.acct.hist[k].goods[(b * G + g) * accounts::KG])).unwrap(),
    };
    let _ = T;
    let tb = accounts::town_books(st, C, t, Some(year)).expect("the town's books");
    let gb = &tb.goods[g];
    println!("\n{} in {}, {} (seed {}): the year's books", C.good_label[g], tb.name, year, seeds[0]);
    println!("  in store as the year began {:14.1}", gb.stock_open);
    println!("  made                       {:14.1}", gb.made);
    for m in tb.methods.iter().filter(|m| m.good == g) {
        println!("    by {:<20} {:12.1}   {:.1} works, {:.0} hands, wages {:.0}, materials {:.0}, fuel {:.0}, carting in {:.0} and out {:.0}, rent {:.0}, profit {:.0}, dividends {:.0}", C.method_label[m.method], m.out, m.works, m.hands, m.wages, m.inputs, m.fuel, m.cart_in, m.cart_out, m.rent, m.profit, m.dividends);
    }
    for l in tb.land.iter().filter(|l| l.good == g && l.out > 0.0) {
        println!("    by {:<20} {:12.1}   {:.0} hands, wages {:.0}, carting {:.0}, rent {:.0}", C.sector_label[l.sector], l.out, l.hands, l.wages, l.carting, l.rent);
    }
    println!("  received from other towns  {:14.1}", gb.received);
    for f in tb.received.iter().filter(|f| f.good == g) {
        println!("    from {:<18} {:12.1}   worth {:.0} there, carting {:.0} ({:.3} a unit)", st.town_name[f.town], f.qty, f.value, f.carting, f.carting / f.qty);
    }
    println!("  came with districts        {:14.1}", gb.moved);
    println!("  sold though not in store   {:14.1}", gb.oversold);
    println!("  bought                     {:14.1}", gb.bought);
    for (label, v) in [("households", gb.households), ("landlords", gb.landlords), ("families", gb.families), ("works, as material", gb.works_inputs), ("works, as fuel", gb.works_fuel), ("farms, pits and mines", gb.land), ("works being built", gb.building), ("railways being built", gb.projects)] {
        if v != 0.0 {
            println!("    by {:<20} {:12.1}", label, v);
        }
    }
    println!("  sent to other towns        {:14.1}", gb.sent);
    for f in tb.sent.iter().filter(|f| f.good == g) {
        println!("    to {:<20} {:12.1}   worth {:.0} here, carting {:.0} ({:.3} a unit)", st.town_name[f.town], f.qty, f.value, f.carting, f.carting / f.qty);
    }
    println!("  spoiled in store           {:14.1}", gb.spoiled);
    println!("  in store as the year ended {:14.1}", gb.stock);
    println!("  left unplaced by the books {:14.4}", gb.unplaced);
    println!("  asked for {:.1}, of which unmet {:.1}", gb.asked, gb.unmet);
    println!("  price {:.3} (from {:.3} to {:.3}){}", gb.price, gb.price_low, gb.price_high, gb.anchor.map_or(String::new(), |a| format!(", the merchants' anchor {a:.3}")));
    let fig = |name: &str| tb.figures.iter().find(|(n, _)| n == name).map_or(f64::NAN, |(_, v)| *v);
    println!("  the town: {:.0} people, wage {:.3}, {:.0} hands on the land, {:.0} in works, {:.0} carting", fig("pop"), fig("wage"), fig("hands_land"), fig("hands_works"), fig("carters"));

    if show_works {
        // the largest works making the good in that town that year
        let rec = &st.acct.hist[k];
        let best = (0..rec.works_id.len())
            .filter(|&j| {
                let w = &rec.works[j * accounts::KW..(j + 1) * accounts::KW];
                w[2] as usize == t && C.method_out[w[1] as usize] == g
            })
            .max_by(|&a, &b| rec.works[a * accounts::KW + 4].total_cmp(&rec.works[b * accounts::KW + 4]));
        match best.and_then(|j| accounts::works_books(st, C, rec.works_id[j] as usize)) {
            Some(wb) => {
                println!("\n{} ({}), year by year", wb.name, C.method_label[wb.method]);
                println!("  year  owner  capacity      out  revenue   inputs     fuel  cart in  cart out    hands    wages     rent   profit  dividends  invest  util");
                for y in wb.years.iter().filter(|y| (y.year - year).abs() <= 3) {
                    println!(
                        "  {}  {:>5}  {:8.1} {:8.1} {:8.0} {:8.0} {:8.0} {:8.0} {:9.0} {:8.1} {:8.0} {:8.0} {:8.0} {:10.0} {:7.0}  {:.2}",
                        y.year,
                        y.owner.map_or("-".to_string(), |o| st.families[o].surname.chars().take(5).collect()),
                        y.capacity,
                        y.out,
                        y.revenue,
                        y.inputs,
                        y.fuel,
                        y.cart_in,
                        y.cart_out,
                        y.hands,
                        y.wages,
                        y.rent,
                        y.profit,
                        y.dividends,
                        y.invest,
                        y.util
                    );
                }
                for inp in &wb.inputs {
                    println!("  in {}: {:.1} of {} as material, {:.1} as fuel, costing {:.0} at the works", y1, inp.used, inp.name, inp.burnt, inp.cost);
                }
            }
            None => println!("\nno works made {} in {} in {}", good, tb.name, year),
        }
    }
    if show_island {
        println!("\nthe island's account for {year}:\n{}", serde_json::to_string_pretty(&st.acct.years[k]).unwrap());
    }
}
