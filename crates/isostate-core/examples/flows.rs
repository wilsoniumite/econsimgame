//! A market's trades as a flow, read from the books of account
//! (`docs/rules/accounts.md`): the measuring behind `docs/probes/dx-inspector.md`.
//! For a few towns and years it prints what the town makes, what it buys in and
//! sends out, from and to which towns, at what price and with what carting, and
//! who earns what from it; then holds the figures against one another and says
//! where they do not meet. It changes nothing and reads only the books.
//!
//!     cargo run --release -p isostate-core --example flows -- [--seed 7] [--rules game]
//!         [--families 6] [--end-year 1880] [--years 1800,1880] [--survey]
//!         [--towns NAME,NAME,...] [--good iron] [--works]
//!
//! `--survey` lists every town in each year, to choose from. `--towns` prints
//! each named town's books in each year. `--good` follows one good across the
//! island in each year: every town that makes, uses or trades it, and the
//! largest flows. `--works` prints the books of the largest works (by the value
//! it made in the last year asked for; of those making `--good`, if it is given)
//! in each of the towns named, with what it
//! used good by good, which is kept for the last complete year of the run only:
//! hence `--end-year`.
#![allow(non_snake_case)]
use isostate_core::accounts::{self, TownBooks, TownFlow};
use isostate_core::*;

fn fig(tb: &TownBooks, name: &str) -> f64 {
    tb.figures.iter().find(|(n, _)| n == name).map_or(f64::NAN, |(_, v)| *v)
}

/// A difference, with what it is a share of, in a few characters.
fn diff(a: f64, b: f64) -> String {
    let d = a - b;
    let scale = a.abs().max(b.abs()).max(1e-9);
    format!("{:>14.1} {:>14.1} {:>12.3} ({:+.1e})", a, b, d, d / scale)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seed, mut rules, mut families, mut end_year) = (7u64, Rules::Game, 6usize, Some(1880i64));
    let (mut years, mut towns, mut good, mut survey, mut show_works) = (vec![1800i64, 1880], Vec::<String>::new(), None::<String>, false, false);
    let mut history = false;
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seed" => seed = next().parse().unwrap(),
            "--rules" => rules = Rules::parse(&next()).expect("port or game"),
            "--families" => families = next().parse().unwrap(),
            "--end-year" => end_year = Some(next().parse().unwrap()),
            "--years" => years = next().split(',').map(|s| s.parse().unwrap()).collect(),
            "--towns" => towns = next().split(',').map(String::from).collect(),
            "--good" => good = Some(next()),
            "--survey" => survey = true,
            "--works" => show_works = true,
            "--history" => history = true,
            other => panic!("unknown option {other}"),
        }
        i += if matches!(args[i].as_str(), "--survey" | "--works" | "--history") { 1 } else { 2 };
    }
    let mut P = rules.params();
    P.seed = seed;
    P.n_families = families;
    P.end_year = end_year;
    P.accounts.enabled = true;
    let spread = P.merchant_spread;
    let t0 = std::time::Instant::now();
    let r = sim::run_with(&P, rules.content(), false, true).expect("money is conserved every month");
    let (st, C) = (&r.state, &r.content);
    println!("island {seed}, {} rules, {families} families, to {}; ran in {:.1} s", rules.as_str(), C.start_year + r.rec.n_years as i64 - 1, t0.elapsed().as_secs_f64());

    for &year in &years {
        // every town's books for the year, so that a flow can be read at both ends
        let all: Vec<Option<TownBooks>> = (0..st.town_name.len()).map(|t| accounts::town_books(st, C, t, Some(year)).filter(|tb| fig(tb, "pop") > 0.0)).collect();
        let price_at = |t: usize, g: usize| all[t].as_ref().map_or(f64::NAN, |tb| tb.goods[g].price);

        if survey {
            println!("\n==== {year}: every town");
            println!("{:>14}  founded   people    wage   hands land  hands works  carters    made on land   made in works        sent    received  partners  merchants' net  merchants' cash  chief makes, by value", "");
            let mut order: Vec<usize> = (0..all.len()).filter(|&t| all[t].is_some()).collect();
            order.sort_by(|&a, &b| fig(all[b].as_ref().unwrap(), "pop").total_cmp(&fig(all[a].as_ref().unwrap(), "pop")));
            for t in order {
                let tb = all[t].as_ref().unwrap();
                let land: f64 = tb.land.iter().map(|l| l.value).sum();
                let works: f64 = tb.methods.iter().map(|m| m.value).sum();
                let sent: f64 = tb.sent.iter().map(|f| f.value).sum();
                let recv: f64 = tb.received.iter().map(|f| f.value).sum();
                let mut partners: Vec<usize> = tb.sent.iter().chain(&tb.received).map(|f| f.town).collect();
                partners.sort();
                partners.dedup();
                let mut makes: Vec<(f64, usize)> = (0..C.G).map(|g| (tb.goods[g].made * tb.goods[g].price, g)).filter(|x| x.0 > 0.0).collect();
                makes.sort_by(|a, b| b.0.total_cmp(&a.0));
                let chief: Vec<String> = makes.iter().take(4).map(|&(v, g)| format!("{} {:.0}%", C.good_name[g], 100.0 * v / (land + works).max(1e-9))).collect();
                println!(
                    "{:>14}  {:7}  {:7.0}  {:6.2}  {:11.0}  {:11.0}  {:7.0}  {:14.0}  {:14.0}  {:10.0}  {:10.0}  {:8}  {:14.0}  {:15.0}  {}",
                    tb.name,
                    st.town_founded[t],
                    fig(tb, "pop"),
                    fig(tb, "wage"),
                    fig(tb, "hands_land"),
                    fig(tb, "hands_works"),
                    fig(tb, "carters"),
                    land,
                    works,
                    sent,
                    recv,
                    partners.len(),
                    fig(tb, "merchants_net"),
                    fig(tb, "cash_merchants"),
                    chief.join(", ")
                );
            }
        }

        for name in &towns {
            let t = st.town_name.iter().position(|n| n == name).unwrap_or_else(|| panic!("no town called {name}"));
            let Some(tb) = all[t].as_ref() else {
                println!("\n==== {name} has no books in {year}");
                continue;
            };
            town(st, C, tb, year, spread, &all, &price_at);
        }

        if let Some(gname) = &good {
            let g = C.gi(gname);
            println!("\n==== {} across the island, {year}", C.good_label[g]);
            println!("{:>14}  {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}  {:>7} {:>7} {:>7}", "", "made", "bought", "by works", "sent", "received", "stock in", "stock out", "price", "low", "high");
            let mut tot = [0.0f64; 5];
            for tb in all.iter().flatten() {
                let gb = &tb.goods[g];
                if gb.made + gb.bought + gb.sent + gb.received < 0.5 {
                    continue;
                }
                tot[0] += gb.made;
                tot[1] += gb.bought;
                tot[2] += gb.sent;
                tot[3] += gb.received;
                tot[4] += gb.made * gb.price;
                println!(
                    "{:>14}  {:10.1} {:10.1} {:10.1} {:10.1} {:10.1} {:10.1} {:10.1}  {:7.3} {:7.3} {:7.3}",
                    tb.name, gb.made, gb.bought, gb.works_inputs + gb.works_fuel, gb.sent, gb.received, gb.stock_open, gb.stock, gb.price, gb.price_low, gb.price_high
                );
            }
            println!("{:>14}  {:10.1} {:10.1} {:>10} {:10.1} {:10.1}   (made, at each town's price: {:.0})", "the island", tot[0], tot[1], "", tot[2], tot[3], tot[4]);
            let k = (year - C.start_year) as usize;
            let rec = st.acct.year(k).expect("the year's books");
            let mut flows: Vec<_> = rec.trade.iter().filter(|f| f.good as usize == g).collect();
            flows.sort_by(|a, b| b.qty.total_cmp(&a.qty));
            let (n, q, v, c) = (flows.len(), flows.iter().map(|f| f.qty as f64).sum::<f64>(), flows.iter().map(|f| f.value as f64).sum::<f64>(), flows.iter().map(|f| f.carting as f64).sum::<f64>());
            println!("  {n} flows between towns, {q:.1} units worth {v:.0} where they left, carting {c:.0} ({:.3} a unit)", c / q.max(1e-9));
            println!("  {:>12} -> {:<12} {:>9} {:>9} {:>9} {:>9}  {:>9} {:>9}", "from", "to", "qty", "price", "carting", "a unit", "delivered", "price");
            for f in flows.iter().take(15) {
                let (a, b) = (f.from as usize, f.to as usize);
                let up = f.value as f64 / f.qty as f64;
                let cu = f.carting as f64 / f.qty as f64;
                println!("  {:>12} -> {:<12} {:9.1} {:9.3} {:9.0} {:9.3}  {:9.3} {:9.3}", st.town_name[a], st.town_name[b], f.qty, up, f.carting, cu, up + cu, price_at(b, g));
            }
        }
    }

    if history {
        // the merchants of each named town over the years, with the one good they
        // cannot keep: services spoil by half a month in store
        let sv = C.gi("services");
        for name in &towns {
            let t = st.town_name.iter().position(|n| n == name).unwrap();
            println!("\n==== {name}'s merchants, every fifth year (services: made, received, spoiled, and what those received cost delivered)");
            println!("  year   people  merchants' net  their dividend  their cash  their stock's value   made    received    spoiled   cost delivered");
            for k in 0..st.acct.hist.n {
                let y = C.start_year + k as i64;
                if y % 5 != 0 {
                    continue;
                }
                let Some(tb) = accounts::town_books(st, C, t, Some(y)).filter(|tb| fig(tb, "pop") > 0.0) else { continue };
                let s = &tb.goods[sv];
                let cost: f64 = tb.received.iter().filter(|f| f.good == sv).map(|f| f.value + f.carting).sum();
                println!(
                    "  {y}  {:7.0}  {:14.0}  {:14.0}  {:10.0}  {:19.0}  {:7.0}  {:9.0}  {:9.0}  {:15.0}",
                    fig(&tb, "pop"),
                    fig(&tb, "merchants_net"),
                    fig(&tb, "div_merchants"),
                    fig(&tb, "cash_merchants"),
                    fig(&tb, "stock_value"),
                    s.made,
                    s.received,
                    s.spoiled,
                    cost
                );
            }
        }
    }

    if show_works {
        let year = *years.last().unwrap();
        let k = (year - C.start_year) as usize;
        let rec = st.acct.year(k).expect("the year's books");
        let col = |name: &str| accounts::WORKS_FIELDS.iter().position(|&n| n == name).unwrap();
        for name in &towns {
            let t = st.town_name.iter().position(|n| n == name).unwrap();
            let makes = |j: usize| good.as_ref().is_none_or(|g| C.method_out[rec.works[j * accounts::KW + col("method")] as usize] == C.gi(g));
            let best = (0..rec.works_id.len())
                .filter(|&j| rec.works[j * accounts::KW + col("town")] as usize == t && makes(j))
                .max_by(|&a, &b| rec.works[a * accounts::KW + col("value")].total_cmp(&rec.works[b * accounts::KW + col("value")]));
            let Some(wb) = best.and_then(|j| accounts::works_books(st, C, rec.works_id[j] as usize)) else { continue };
            println!("\n==== the largest works in {name} in {year}: {} ({}), founded {}", wb.name, C.method_label[wb.method], wb.years.first().map_or(0, |y| y.year));
            println!("  year  owner   capacity       out      value    revenue     inputs     fuel   cart in  cart out   hands     wages     rent  managers    profit  dividends    invest  util  months       cash");
            for y in &wb.years {
                println!(
                    "  {}  {:>5}  {:9.1} {:9.1} {:10.0} {:10.0} {:10.0} {:8.0} {:9.0} {:9.0} {:7.1} {:9.0} {:8.0} {:9.0} {:9.0} {:10.0} {:9.0}  {:.2}  {:6.0} {:10.0}",
                    y.year,
                    y.owner.map_or("-".to_string(), |o| st.families[o].surname.chars().take(5).collect()),
                    y.capacity,
                    y.out,
                    y.value,
                    y.revenue,
                    y.inputs,
                    y.fuel,
                    y.cart_in,
                    y.cart_out,
                    y.hands,
                    y.wages,
                    y.rent,
                    y.managers,
                    y.profit,
                    y.dividends,
                    y.invest,
                    y.util,
                    y.months,
                    y.cash
                );
                // the works' own identity: profit is what is left of its revenue
                let left = y.revenue - y.inputs - y.fuel - y.cart_in - y.wages - y.rent - y.managers - y.profit;
                let sold = y.value * (1.0 - spread) - y.cart_out - y.revenue;
                if left.abs() > 1e-3 * y.revenue.abs().max(1.0) || sold.abs() > 1e-3 * y.revenue.abs().max(1.0) {
                    println!("        does not meet: revenue less costs less profit {left:.3}; value less spread less cart out less revenue {sold:.3}");
                }
            }
            for inp in &wb.inputs {
                println!("  in {}: {:.1} of {} as material, {:.1} as fuel, costing {:.0} delivered ({:.3} a unit; {}'s price {:.3})", C.start_year + st.acct.hist.n as i64 - 1, inp.used, inp.name, inp.burnt, inp.cost, inp.cost / (inp.used + inp.burnt).max(1e-9), name, accounts::town_books(st, C, t, None).map_or(f64::NAN, |tb| tb.goods[inp.good].price));
            }
        }
    }
}

/// One town's books for a year, and the checks.
fn town(st: &State, C: &Content, tb: &TownBooks, year: i64, spread: f64, all: &[Option<TownBooks>], price_at: &dyn Fn(usize, usize) -> f64) {
    println!("\n==== {} in {year}", tb.name);
    println!(
        "  {:.0} people; wage {:.3} a month; {:.0} hands for hire, {:.0} on the land, {:.0} in works, {:.0} carting, {:.0} digging canals",
        fig(tb, "pop"),
        fig(tb, "wage"),
        fig(tb, "supply"),
        fig(tb, "hands_land"),
        fig(tb, "hands_works"),
        fig(tb, "carters"),
        fig(tb, "navvies")
    );

    // ---- what it makes
    println!("\n  -- made on the land (value at the town's price; the merchants pay value less {:.0} per cent)", 100.0 * spread);
    println!("  {:<18} {:<10} {:>10} {:>8} {:>10} {:>8} {:>9} {:>9} {:>9} {:>9}", "sector", "good", "out", "price", "value", "hands", "wages", "carting", "machines", "rent");
    for l in tb.land.iter().filter(|l| l.out > 0.0) {
        println!("  {:<18} {:<10} {:10.1} {:8.3} {:10.0} {:8.0} {:9.0} {:9.0} {:9.0} {:9.0}", C.sector_label[l.sector], C.good_name[l.good], l.out, tb.goods[l.good].price, l.value, l.hands, l.wages, l.carting, l.inputs, l.rent);
    }
    println!("\n  -- made in works");
    println!("  {:<22} {:<9} {:>5} {:>9} {:>9} {:>5} {:>9} {:>6} {:>9} {:>9} {:>7} {:>7} {:>7} {:>7} {:>7} {:>8} {:>9}", "method", "good", "works", "capacity", "out", "util", "value", "hands", "wages", "materials", "fuel", "cart in", "cartout", "rent", "manager", "profit", "dividends");
    for m in &tb.methods {
        println!(
            "  {:<22} {:<9} {:5.1} {:9.1} {:9.1} {:5.2} {:9.0} {:6.0} {:9.0} {:9.0} {:7.0} {:7.0} {:7.0} {:7.0} {:7.0} {:8.0} {:9.0}",
            C.method_label[m.method], C.good_name[m.good], m.works, m.capacity, m.out, m.out / (12.0 * m.capacity).max(1e-9), m.value, m.hands, m.wages, m.inputs, m.fuel, m.cart_in, m.cart_out, m.rent, m.managers, m.profit, m.dividends
        );
        for u in &m.uses {
            let q = u.used + u.burnt;
            println!("      uses {:<10} {:9.1} as material {:9.1} as fuel, {:9.0} delivered ({:.3} a unit; the town's price {:.3})", u.name, u.used, u.burnt, u.cost, u.cost / q.max(1e-9), tb.goods[u.good].price);
        }
    }

    // ---- the goods' balance
    println!("\n  -- every good: where it came from and where it went (units)");
    println!("  {:<10} {:>9} {:>9} {:>9} {:>7} {:>6} {:>9} {:>9} {:>7} {:>9} {:>8} {:>8} {:>8} {:>8} {:>8}", "good", "in store", "made", "received", "moved", "overs", "bought", "sent", "spoiled", "in store", "unplaced", "price", "low", "high", "unmet");
    let active = |g: usize| {
        let b = &tb.goods[g];
        b.made + b.received + b.sent + b.bought + b.stock + b.stock_open > 0.5
    };
    for b in tb.goods.iter().filter(|b| active(b.good)) {
        println!(
            "  {:<10} {:9.1} {:9.1} {:9.1} {:7.1} {:6.2} {:9.1} {:9.1} {:7.1} {:9.1} {:8.4} {:8.3} {:8.3} {:8.3} {:8.1}",
            b.name, b.stock_open, b.made, b.received, b.moved, b.oversold, b.bought, b.sent, b.spoiled, b.stock, b.unplaced, b.price, b.price_low, b.price_high, b.unmet
        );
    }
    println!("\n  -- who bought it (units)");
    println!("  {:<10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}", "good", "households", "landlords", "families", "materials", "fuel", "land", "building", "railways");
    for b in tb.goods.iter().filter(|b| b.bought > 0.5) {
        println!("  {:<10} {:10.1} {:10.1} {:10.1} {:10.1} {:10.1} {:10.1} {:10.1} {:10.1}", b.name, b.households, b.landlords, b.families, b.works_inputs, b.works_fuel, b.land, b.building, b.projects);
    }

    // ---- the trade, good by good
    let flows = |all_flows: &[TownFlow], inward: bool| {
        // the books keep every flow, however small: a twentieth of a unit and less is left out here
        let list: Vec<TownFlow> = all_flows.iter().filter(|f| f.qty >= 0.05).cloned().collect();
        let small = all_flows.len() - list.len();
        if small > 0 {
            println!("  ({small} flows of less than a twentieth of a unit left out, {:.3} units in all)", all_flows.iter().filter(|f| f.qty < 0.05).map(|f| f.qty).sum::<f64>());
        }
        let list = &list[..];
        let mut goods: Vec<usize> = list.iter().map(|f| f.good).collect();
        goods.sort();
        goods.dedup();
        goods.sort_by(|&a, &b| {
            let v = |g: usize| list.iter().filter(|f| f.good == g).map(|f| f.value).sum::<f64>();
            v(b).total_cmp(&v(a))
        });
        for g in goods {
            let mut fs: Vec<&TownFlow> = list.iter().filter(|f| f.good == g).collect();
            fs.sort_by(|a, b| b.qty.total_cmp(&a.qty));
            let (q, v, c) = (fs.iter().map(|f| f.qty).sum::<f64>(), fs.iter().map(|f| f.value).sum::<f64>(), fs.iter().map(|f| f.carting).sum::<f64>());
            println!(
                "  {:<10} {:9.1} units, worth {:.0} where they left ({:.3} a unit), carting {:.0} ({:.3} a unit), {} towns; the price here {:.3}",
                C.good_name[g],
                q,
                v,
                v / q.max(1e-9),
                c,
                c / q.max(1e-9),
                fs.len(),
                tb.goods[g].price
            );
            for f in fs.iter().take(8) {
                let up = f.value / f.qty.max(1e-9);
                let cu = f.carting / f.qty.max(1e-9);
                let (there, word) = if inward { (price_at(f.town, g), "from") } else { (price_at(f.town, g), "to") };
                if inward {
                    println!("      {word} {:<14} {:9.1}  at {:7.3} there, carting {:6.3}, delivered {:7.3}; the price there {:7.3}, here {:7.3}", st.town_name[f.town], f.qty, up, cu, up + cu, there, tb.goods[g].price);
                } else {
                    println!("      {word} {:<14} {:9.1}  at {:7.3} here, carting {:6.3}, delivered {:7.3}; the price there {:7.3}", st.town_name[f.town], f.qty, up, cu, up + cu, there);
                }
            }
            if fs.len() > 8 {
                println!("      and {} more, {:.1} units", fs.len() - 8, fs[8..].iter().map(|f| f.qty).sum::<f64>());
            }
        }
    };
    println!("\n  -- received from other towns");
    flows(&tb.received, true);
    println!("\n  -- sent to other towns");
    flows(&tb.sent, false);

    // ---- who earns what
    println!("\n  -- what the town's households received");
    let kinds = ["pay_works", "pay_land", "pay_projects", "cart_works", "cart_land", "cart_trade", "managers", "div_works", "div_merchants", "fund", "issue"];
    let hh: f64 = kinds.iter().map(|k| fig(tb, k)).sum();
    for k in kinds {
        println!("  {:<16} {:12.0}  {:5.1}%", k, fig(tb, k), 100.0 * fig(tb, k) / hh);
    }
    println!("  {:<16} {:12.0}", "all of it", hh);
    println!("\n  -- what went to owners and merchants");
    for k in ["div_families", "rent_land", "rent_sites", "rent_town", "rent_families", "merchants_net", "div_merchants", "stock_value"] {
        println!("  {:<16} {:12.0}", k, fig(tb, k));
    }
    println!("\n  -- spending, and cash at the year's end");
    for k in ["spend_households", "spend_landlords", "spend_families", "invest", "projects", "cash_households", "cash_landlords", "cash_merchants"] {
        println!("  {:<16} {:12.0}", k, fig(tb, k));
    }
    println!("\n  -- where a shilling of the town's make went (shares of its value at the town's price)");
    let lv: f64 = tb.land.iter().map(|l| l.value).sum();
    if lv > 0.0 {
        let s = |f: &dyn Fn(&accounts::LandBooks) -> f64| tb.land.iter().map(f).sum::<f64>() / lv;
        println!(
            "  the land ({:.0}): merchants {:.3}, wages {:.3}, carting {:.3}, machines and rails {:.3}, rent {:.3}",
            lv,
            spread,
            s(&|l| l.wages),
            s(&|l| l.carting),
            s(&|l| l.inputs),
            s(&|l| l.rent)
        );
    }
    let mv: f64 = tb.methods.iter().map(|m| m.value).sum();
    if mv > 0.0 {
        let s = |f: &dyn Fn(&accounts::MethodBooks) -> f64| tb.methods.iter().map(f).sum::<f64>() / mv;
        println!(
            "  the works ({:.0}): merchants {:.3}, carting out {:.3}, materials {:.3}, fuel {:.3}, carting in {:.3}, wages {:.3}, site rent {:.3}, managers {:.3}, profit {:.3} (paid out {:.3})",
            mv,
            spread,
            s(&|m| m.cart_out),
            s(&|m| m.inputs),
            s(&|m| m.fuel),
            s(&|m| m.cart_in),
            s(&|m| m.wages),
            s(&|m| m.rent),
            s(&|m| m.managers),
            s(&|m| m.profit),
            s(&|m| m.dividends)
        );
    }

    // ---- the checks
    println!("\n  -- the checks: the books' figures against one another ({:>14} {:>14} {:>12})", "this", "against", "difference");
    let worst = tb.goods.iter().max_by(|a, b| a.unplaced.abs().total_cmp(&b.unplaced.abs())).unwrap();
    println!("  each good's balance: the worst is {} with {:.4} units unplaced of {:.1} that passed through", worst.name, worst.unplaced, worst.stock_open + worst.made + worst.received);
    for b in tb.goods.iter().filter(|b| active(b.good)) {
        let land: f64 = tb.land.iter().filter(|l| l.good == b.good).map(|l| l.out).sum();
        let works: f64 = tb.methods.iter().filter(|m| m.good == b.good).map(|m| m.out).sum();
        let d = b.made - b.cleared - land - works;
        if d.abs() > 1e-3 * b.made.max(1.0) {
            println!("  {} made against the land's and works' output, clearing aside: {}", b.name, diff(b.made - b.cleared, land + works));
        }
        let s: f64 = tb.sent.iter().filter(|f| f.good == b.good).map(|f| f.qty).sum();
        let r: f64 = tb.received.iter().filter(|f| f.good == b.good).map(|f| f.qty).sum();
        if (s - b.sent).abs() > 1e-3 * b.sent.max(1.0) || (r - b.received).abs() > 1e-3 * b.received.max(1.0) {
            println!("  {} sent and received against the flows: {} / {}", b.name, diff(b.sent, s), diff(b.received, r));
        }
        if b.oversold > 0.0 {
            println!("  {} sold though not in store: {:.3} units, {:.2e} of what was bought", b.name, b.oversold, b.oversold / b.bought.max(1e-9));
        }
        if b.moved != 0.0 {
            println!("  {} came (+) or went (-) with districts at the year's end: {:.1} units", b.name, b.moved);
        }
    }
    for l in tb.land.iter().filter(|l| l.out > 0.0) {
        let d = l.value * (1.0 - spread) - l.wages - l.carting - l.inputs - l.rent;
        if d.abs() > 1e-3 * l.value {
            println!("  {}: value less the spread against wages, carting, machines and rent: {:.3}", C.sector_label[l.sector], d);
        }
    }
    for m in &tb.methods {
        let rhs = m.inputs + m.fuel + m.cart_in + m.wages + m.rent + m.managers + m.profit;
        println!("  {}: value less spread less carting out, against its costs and profit: {}", C.method_label[m.method], diff(m.value * (1.0 - spread) - m.cart_out, rhs));
        let used: f64 = m.uses.iter().map(|u| u.cost).sum();
        println!("  {}: materials, fuel and carting in, against what its uses cost delivered: {}", C.method_label[m.method], diff(m.inputs + m.fuel + m.cart_in, used));
        if m.dividends > m.profit.max(0.0) + 1.0 {
            println!("  {}: paid out {:.0} on a profit of {:.0}", C.method_label[m.method], m.dividends, m.profit);
        }
    }
    let sum_l = |f: &dyn Fn(&accounts::LandBooks) -> f64| tb.land.iter().map(f).sum::<f64>();
    let sum_m = |f: &dyn Fn(&accounts::MethodBooks) -> f64| tb.methods.iter().map(f).sum::<f64>();
    println!("  pay_land against the land's wages:           {}", diff(fig(tb, "pay_land"), sum_l(&|l| l.wages)));
    println!("  cart_land against the land's carting:        {}", diff(fig(tb, "cart_land"), sum_l(&|l| l.carting)));
    println!("  rent_land against the land's rent:           {}", diff(fig(tb, "rent_land"), sum_l(&|l| l.rent)));
    println!("  hands_land against the land's hands:        {}", diff(fig(tb, "hands_land"), sum_l(&|l| l.hands)));
    println!("  pay_works against the works' wages:          {}", diff(fig(tb, "pay_works"), sum_m(&|m| m.wages)));
    println!("  cart_works against the works' carting:       {}", diff(fig(tb, "cart_works"), sum_m(&|m| m.cart_in + m.cart_out)));
    println!("  rent_sites against the works' rent:          {}", diff(fig(tb, "rent_sites"), sum_m(&|m| m.rent)));
    println!("  managers against the works' managers:        {}", diff(fig(tb, "managers"), sum_m(&|m| m.managers)));
    println!("  hands_works against the works' hands:       {}", diff(fig(tb, "hands_works"), sum_m(&|m| m.hands)));
    println!("  div_works + div_families against dividends:  {}", diff(fig(tb, "div_works") + fig(tb, "div_families"), sum_m(&|m| m.dividends)));
    println!("  cart_trade against the carting on sent goods: {}", diff(fig(tb, "cart_trade"), tb.sent.iter().map(|f| f.carting).sum()));
    // the works' own rows, added up, against the method rows
    let k = (year - C.start_year) as usize;
    let rec = st.acct.year(k).expect("the year's books");
    let col = |name: &str| accounts::WORKS_FIELDS.iter().position(|&n| n == name).unwrap();
    let rows: Vec<&[f32]> = (0..rec.works_id.len()).map(|j| &rec.works[j * accounts::KW..(j + 1) * accounts::KW]).filter(|w| w[col("town")] as usize == tb.town).collect();
    let ws = |c: &str| rows.iter().map(|w| w[col(c)] as f64).sum::<f64>();
    println!("  the works' rows ({} of them) against the methods: out {}", rows.len(), diff(ws("out"), sum_m(&|m| m.out)));
    println!("                                                 wages {}", diff(ws("wages"), sum_m(&|m| m.wages)));
    println!("                                                profit {}", diff(ws("profit"), sum_m(&|m| m.profit)));
    println!("                                             dividends {}", diff(ws("dividends"), sum_m(&|m| m.dividends)));
    println!("                                     invest (the town's) {}", diff(ws("invest"), fig(tb, "invest")));
    // money: the purses at the year's end against the year before, and what the books saw go in and out
    if let Some(prev) = all.first().and(accounts::town_books(st, C, tb.town, Some(year - 1))) {
        let hh_in: f64 = ["pay_works", "pay_land", "pay_projects", "cart_works", "cart_land", "cart_trade", "managers", "div_works", "div_merchants", "fund", "issue"].iter().map(|k| fig(tb, k)).sum();
        let d_cash = fig(tb, "cash_households") - fig(&prev, "cash_households");
        println!("  households: received less spent on goods and house rent, against their cash's change: {}", diff(hh_in - fig(tb, "spend_households") - fig(tb, "rent_town"), d_cash));
        let d_m = fig(tb, "cash_merchants") - fig(&prev, "cash_merchants");
        println!("  merchants: their net less their dividend, against their cash's change: {}", diff(fig(tb, "merchants_net") - fig(tb, "div_merchants"), d_m));
        let d_s = fig(tb, "stock_value") - fig(&prev, "stock_value");
        println!("  merchants: their stock's value at the year's end less the year before: {:.0}", d_s);
        println!("  people: {:.0} at the end of {} and {:.0} at the end of {}", fig(&prev, "pop"), year - 1, fig(tb, "pop"), year);
    }
}
