//! The books (not in the Python; docs/rules/accounts.md): they change nothing
//! in the run they watch, they balance, they agree with what the observer and
//! the families keep for themselves, and they are the same books every time.
//! With the switch off the golden test (tests/golden.rs) covers the port.
#![allow(non_snake_case)]

use isostate_core::accounts::{self, GOODS_FIELDS, KG, KW, WORKS_FIELDS};
use isostate_core::observer::annual;
use isostate_core::*;

fn params(rules: Rules, seed: u64, families: usize, end_year: i64, books: bool) -> Params {
    let mut P = rules.params();
    P.seed = seed;
    P.n_families = families;
    P.end_year = Some(end_year);
    P.accounts.enabled = books;
    P
}

fn run(rules: Rules, P: &Params) -> RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(P, rules.content(), false, true).expect("money is conserved every month")
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn col(name: &str) -> usize {
    GOODS_FIELDS.iter().position(|&n| n == name).unwrap()
}

fn wcol(name: &str) -> usize {
    WORKS_FIELDS.iter().position(|&n| n == name).unwrap()
}

/// Names of the arrays in a packed run, and its header.
fn header(blob: &[u8]) -> serde_json::Value {
    let n = u32::from_le_bytes(blob[..4].try_into().unwrap()) as usize;
    serde_json::from_slice(&blob[4..4 + n]).unwrap()
}

#[test]
fn off_under_the_port_and_on_in_the_game() {
    assert!(!default_params().accounts.enabled && game_params().accounts.enabled);
    // a port run keeps no books and says nothing of them anywhere
    let P = params(Rules::Port, 7, 0, 1752, false);
    let r = run(Rules::Port, &P);
    let b = &r.state.acct;
    assert!(!b.on && b.years.is_empty() && b.hist.n == 0 && b.hist.goods.capacity() == 0 && b.last_months.is_empty() && b.seconds == 0.0);
    assert!(r.state.series.names.iter().all(|n| !n.starts_with("acct_")));
    assert!(accounts::town_books(&r.state, &r.content, 0, None).is_none());
    assert!(accounts::works_books(&r.state, &r.content, 0).is_none());
    let h = header(&record::pack(&r.state, &r.content, &r.params, &r.events, &r.rec, "x", false));
    assert!(h.get("accounts").is_none());
    assert!(h["arrays"].as_array().unwrap().iter().all(|a| !a["name"].as_str().unwrap().starts_with("acct_")));
    assert!(!serde_json::to_string(&r.params).unwrap().contains("accounts"), "params.json of a port run does not mention the books");
}

#[test]
fn the_books_change_nothing() {
    // three islands, with and without families, under the port's rules and the
    // game's: every series, every piece of news and the state the run ends in
    // are bit for bit what they are with the books off
    for (rules, seed, families, end) in [(Rules::Port, 3, 6, 1775), (Rules::Port, 11, 0, 1765), (Rules::Game, 7, 6, 1772), (Rules::Game, 5, 0, 1762)] {
        let off = run(rules, &params(rules, seed, families, end, false));
        let on = run(rules, &params(rules, seed, families, end, true));
        let (a, b) = (&off.state, &on.state);
        for (k, name) in a.series.names.iter().enumerate() {
            assert_eq!(bits(&a.series.data[k]), bits(b.series.get(name)), "series {name} moved, seed {seed}");
        }
        assert_eq!(b.series.names.len(), a.series.names.len() + 8, "the books add their eight series after everyone's");
        assert_eq!(serde_json::to_string(&off.events).unwrap(), serde_json::to_string(&on.events).unwrap());
        for (what, x, y) in [
            ("people", &a.pop, &b.pop),
            ("households' cash", &a.hh_cash, &b.hh_cash),
            ("landlords' cash", &a.lord_cash, &b.lord_cash),
            ("merchants' cash", &a.merch_cash, &b.merch_cash),
            ("works' cash", &a.f_cash, &b.f_cash),
            ("works' capacity", &a.f_K, &b.f_K),
            ("prices", &a.price.data, &b.price.data),
            ("stocks", &a.inv.data, &b.inv.data),
            ("wages", &a.wage, &b.wage),
            ("land shares", &a.lam.data, &b.lam.data),
            ("forest cover", &a.cover, &b.cover),
        ] {
            assert_eq!(bits(x), bits(y), "{what} moved, seed {seed}");
        }
        let cash = |st: &State| st.families.iter().map(|f| f.cash.to_bits()).collect::<Vec<_>>();
        assert_eq!(cash(a), cash(b));
        assert_eq!((a.F_count, a.issued.to_bits(), a.money_now.to_bits()), (b.F_count, b.issued.to_bits(), b.money_now.to_bits()));
        assert_eq!(off.rec.layers, on.rec.layers);
        assert!(b.acct.on && b.acct.years.len() == (end - 1749) as usize);
    }
}

/// Every year of a run: goods, incomes and spending all balance.
fn balances(rules: Rules, seed: u64, families: usize, end: i64) {
    let r = run(rules, &params(rules, seed, families, end, true));
    let (st, C) = (&r.state, &r.content);
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    let (T, G) = (st.T_max, C.G);
    for (k, y) in st.acct.years.iter().enumerate() {
        // made + received + moved + oversold = bought + sent + spoiled + added to stock,
        // for every town and good, as the books added it up in double precision
        assert!(y.balance_goods < 1e-9, "{}: {} of {} unplaced in town {}", y.year, y.balance_goods_units, C.good_name[y.balance_goods_at.1], y.balance_goods_at.0);
        // every month, what was paid as income is what households, landlords,
        // families and the fund received
        assert!(y.balance_income < 1e-9, "{}: incomes off by {:e}", y.year, y.balance_income);
        // and over the year
        let rents = y.rent_farm + y.rent_pits + y.rent_sites + y.rent_town;
        let paid = y.wage_bill + y.div_households + y.div_families + y.merchants_div + rents + y.fund_div + y.issue;
        let got = y.to_households + y.to_landlords + y.to_fund + y.to_families.iter().map(|f| f.dividends + f.rents).sum::<f64>();
        assert!((paid - got).abs() < 1e-9 * paid, "{}: {paid} paid as income and {got} received", y.year);
        // what the island earned is what it spent on goods, house rent and building
        assert!(y.balance_spending < 1e-9, "{}: income {} against spending {}", y.year, y.income, y.spending);
        assert!((y.share_work + y.share_works + y.share_merchants + y.share_land - 1.0).abs() < 1e-9);
        // (the shares themselves are not held to any range: in a year a town is founded
        // the port's land can pay more in wages than its produce fetches, and its
        // rent, the remainder, is then below nothing; docs/rules/accounts.md)
        assert!(y.share_work > 0.0 && y.share_work.is_finite() && y.share_land.is_finite());
        assert!(y.pay_share > 0.3 && y.pay_share <= 1.0);
        assert!(y.works_years > 0.0 && y.ownership_years.is_finite());
        assert!(y.oversold_share >= 0.0 && y.oversold_share < 1e-3, "{}: {} of all that was bought was not in store", y.year, y.oversold_share);
        // the same balance from the tables as they are kept (single precision)
        let rec = st.acct.year(k).unwrap();
        for t in 0..T {
            for g in 0..G {
                let v = |name: &str| rec.goods[(t * G + g) * KG + col(name)] as f64;
                let bought = v("households") + v("landlords") + v("families") + v("works_inputs") + v("works_fuel") + v("land") + v("building") + v("projects");
                let through = v("stock_open") + v("made") + v("received");
                let left = v("made") + v("received") + v("moved") + v("oversold") - bought - v("sent") - v("spoiled") - (v("stock") - v("stock_open"));
                assert!(left.abs() <= 1e-5 * through + 1e-6, "{} town {t} {}: {left} unplaced of {through}", y.year, C.good_name[g]);
                assert!(bought <= v("asked") * (1.0 + 1e-5) + 1e-6, "nobody buys more than was asked for");
                assert!(v("price_low") <= v("price") && v("price") <= v("price_high"));
            }
        }
        // every flow between towns is in the towns' own columns
        for t in 0..T {
            for g in 0..G {
                let sent: f64 = rec.trade.iter().filter(|f| f.from as usize == t && f.good as usize == g).map(|f| f.qty as f64).sum();
                let recv: f64 = rec.trade.iter().filter(|f| f.to as usize == t && f.good as usize == g).map(|f| f.qty as f64).sum();
                let v = |name: &str| rec.goods[(t * G + g) * KG + col(name)] as f64;
                assert!((sent - v("sent")).abs() <= 1e-5 * sent + 1e-6 && (recv - v("received")).abs() <= 1e-5 * recv + 1e-6);
            }
        }
    }
}

#[test]
fn the_books_balance_under_the_port() {
    balances(Rules::Port, 3, 6, 1789);
}

#[test]
fn the_books_balance_under_the_game() {
    balances(Rules::Game, 7, 6, 1784);
}

#[test]
fn the_books_balance_under_the_commons() {
    // rents go to the common fund and out again as a dividend to every household
    let mut P = params(Rules::Port, 2, 0, 1760, true);
    P.rent_regime = RentRegime::Commons;
    let r = run(Rules::Port, &P);
    for y in &r.state.acct.years {
        assert!(y.balance_goods < 1e-9 && y.balance_income < 1e-9 && y.balance_spending < 1e-9, "{}", y.year);
        assert!(y.to_landlords == 0.0 && y.to_fund > 0.0 && y.fund_div > 0.0);
    }
}

#[test]
fn the_books_agree_with_the_observer_and_the_families() {
    let r = run(Rules::Port, &params(Rules::Port, 3, 6, 1779, true));
    let (st, C) = (&r.state, &r.content);
    let y0 = C.start_year;
    let (T, G, M) = (st.T_max, C.G, C.M);
    for (k, y) in st.acct.years.iter().enumerate() {
        let rec = st.acct.year(k).unwrap();
        // what the towns made is what the observer's output series say, the
        // timber of cleared woodland aside (the observer does not see it)
        for g in 0..G {
            let made: f64 = (0..T).map(|t| (rec.goods[(t * G + g) * KG + col("made")] - rec.goods[(t * G + g) * KG + col("cleared")]) as f64).sum();
            let series = 12.0 * annual(st.series.get(&format!("out_{}", C.good_name[g])), y0, y.year);
            assert!((made - series).abs() <= 1e-5 * series + 1e-6, "{} {}: {made} in the books, {series} in the series", y.year, C.good_name[g]);
        }
        for m in 0..M {
            let made: f64 = (0..T).map(|t| rec.methods[(t * M + m) * accounts::KM] as f64).sum();
            let series = 12.0 * annual(st.series.get(&format!("mout_{}", C.method_name[m])), y0, y.year);
            assert!((made - series).abs() <= 1e-5 * series + 1e-6, "{} {}: {made} against {series}", y.year, C.method_name[m]);
        }
        // spending is the observer's output in money
        let gdp = 12.0 * annual(st.series.get("gdp_nom"), y0, y.year);
        assert!((y.spending - gdp).abs() < 1e-9 * gdp, "{}: {} spent, output {gdp}", y.year, y.spending);
        // the headline series hold the year's figure in all its months
        assert!((annual(st.series.get("acct_share_work"), y0, y.year) - y.share_work).abs() < 1e-12);
        assert_eq!(st.series.get("acct_share_land")[k * 12 + 5].to_bits(), y.share_land.to_bits());
        assert!((annual(st.series.get("acct_ownership_years"), y0, y.year) - y.ownership_years).abs() < 1e-12);
    }
    // the families' own books of the last year say what ours say they received
    let last = st.acct.years.last().unwrap();
    assert_eq!(last.to_families.len(), 6);
    for (fam, got) in st.families.iter().zip(&last.to_families) {
        let own = fam.last_year.earned();
        assert!((got.dividends + got.rents - own).abs() <= 1e-9 * own.abs().max(1.0), "the {} family: {} in the books, {own} in its own", fam.surname, got.dividends + got.rents);
        assert!((got.rents - fam.last_year.rents).abs() <= 1e-9 * fam.last_year.rents.abs().max(1.0));
    }
    assert!(last.works_value_families > 0.0 && last.works_value_families < last.works_value);
    assert!(last.land_value_families > 0.0 && last.land_value_families < last.land_value);
    let m = &last.money;
    let money = m.households + m.landlords + m.merchants + m.works + m.families + m.projects + m.fund;
    assert!((money - econ::money_total(st)).abs() < 1e-9 * money, "the purses hold all the money there is");
}

#[test]
fn a_town_and_a_works_can_be_asked_for() {
    let r = run(Rules::Port, &params(Rules::Port, 7, 6, 1764, true));
    let (st, C) = (&r.state, &r.content);
    let iron = C.gi("iron");
    // a town's books for a past year and for the last
    let old = accounts::town_books(st, C, 0, Some(1755)).expect("town 0 in 1755");
    let last = accounts::town_books(st, C, 0, None).expect("town 0 in the last year");
    assert_eq!((old.year, last.year, last.town), (1755, 1764, 0));
    assert_eq!((old.goods.len(), old.land.len(), old.needs.len()), (C.G, C.NS, C.S));
    assert!(old.months.is_none() && last.months.as_ref().is_some_and(|m| m.len() == 12 && m[0].len() == C.G));
    assert!(accounts::town_books(st, C, 0, Some(1765)).is_none() && accounts::town_books(st, C, 0, Some(1749)).is_none());
    assert!(accounts::town_books(st, C, st.T_max, None).is_none());
    let g = &last.goods[iron];
    assert!(g.made > 0.0 && g.bought > 0.0 && g.unmet >= 0.0 && g.anchor.is_none());
    assert!(g.unplaced.abs() < 1e-4 * (g.stock_open + g.made + g.received));
    // the last year's months add up to the year
    let months = last.months.as_ref().unwrap();
    let made: f64 = months.iter().map(|m| m[iron][1]).sum();
    assert!((made - g.made).abs() < 1e-4 * g.made);
    assert!((months[11][iron][3] - (g.stock - g.moved)).abs() < 1e-4 * g.stock.max(1.0), "December's stock is the year's, less what came with a district at the year's end");
    // the methods that made the iron account for all of it, and their sums add up
    let by_methods: f64 = last.methods.iter().filter(|m| m.good == iron).map(|m| m.out).sum();
    assert!((by_methods - g.made).abs() < 1e-4 * g.made);
    for m in &last.methods {
        let spent = m.inputs + m.fuel + m.cart_in + m.wages + m.rent + m.managers + m.profit;
        let revenue = m.value * (1.0 - r.params.merchant_spread) - m.cart_out;
        assert!((spent - revenue).abs() < 1e-4 * revenue.abs().max(1.0), "{}: {spent} against {revenue}", m.name);
    }
    // what the methods used, good by good, is what the town's works bought
    for good in &last.goods {
        let used: f64 = last.methods.iter().flat_map(|m| &m.uses).filter(|u| u.good == good.good).map(|u| u.used).sum();
        let burnt: f64 = last.methods.iter().flat_map(|m| &m.uses).filter(|u| u.good == good.good).map(|u| u.burnt).sum();
        assert!((used - good.works_inputs).abs() <= 1e-4 * used + 1e-6 && (burnt - good.works_fuel).abs() <= 1e-4 * burnt + 1e-6, "{}", good.name);
    }
    assert!(last.methods.iter().any(|m| m.uses.iter().any(|u| u.burnt > 0.0)), "some works burns fuel");
    for l in last.land.iter().filter(|l| l.out > 0.0) {
        let spent = l.wages + l.carting + l.inputs + l.rent;
        assert!((spent - l.value * (1.0 - r.params.merchant_spread)).abs() < 1e-4 * l.value);
    }
    let fig = |name: &str| last.figures.iter().find(|(n, _)| n == name).unwrap().1;
    assert!(fig("pop") > 0.0 && fig("wage") > 0.0 && fig("pay_land") > 0.0 && fig("spend_households") > 0.0);
    assert!(last.needs.iter().map(|n| n.households).sum::<f64>() > 0.99 * fig("spend_households"));

    // a works: every year it stood, and its sums add up
    let rec = st.acct.year(st.acct.hist.n - 1).unwrap();
    let busiest = (0..rec.works_id.len()).max_by(|&a, &b| rec.works[a * KW + wcol("inputs")].total_cmp(&rec.works[b * KW + wcol("inputs")])).unwrap();
    let f = rec.works_id[busiest] as usize;
    let wb = accounts::works_books(st, C, f).expect("the works' books");
    assert_eq!(wb.name, st.f_name[f]);
    assert!(!wb.years.is_empty() && wb.years.last().unwrap().year == 1764);
    assert!(wb.years.iter().all(|y| y.year >= st.f_born[f].max(1750)));
    for y in &wb.years {
        let left = y.revenue - y.inputs - y.fuel - y.cart_in - y.wages - y.rent - y.managers - y.profit;
        assert!(left.abs() < 1e-4 * y.revenue.abs().max(1.0), "{}: {left} of revenue {} unplaced", y.year, y.revenue);
        assert!(y.util >= 0.0 && y.util <= 1.0 + 1e-6 && y.months <= 12.0);
    }
    assert!(!wb.inputs.is_empty() && wb.inputs.iter().all(|i| i.cost > 0.0));
    assert!(accounts::works_books(st, C, st.F_count).is_none());
    // with the works' part switched off the towns' books are kept and no works'
    let mut P = params(Rules::Port, 7, 0, 1752, true);
    P.accounts.works = false;
    let r = run(Rules::Port, &P);
    assert!(r.state.acct.hist.works_id.is_empty() && r.state.acct.hist.n == 3 && !r.state.acct.year(2).unwrap().goods.is_empty());
    assert!(accounts::works_books(&r.state, &r.content, 0).is_some_and(|w| w.years.is_empty()));
}

#[test]
fn the_books_repeat_and_go_into_the_run_file_and_the_live_view() {
    let P = params(Rules::Game, 3, 6, 1762, true);
    let (a, b) = (run(Rules::Game, &P), run(Rules::Game, &P));
    assert_eq!(a.state.acct.years, b.state.acct.years);
    let f32bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    let (x, y) = (&a.state.acct.hist, &b.state.acct.hist);
    assert_eq!(f32bits(&x.goods), f32bits(&y.goods));
    assert_eq!(f32bits(&x.methods), f32bits(&y.methods));
    assert_eq!(f32bits(&x.towns), f32bits(&y.towns));
    assert_eq!(f32bits(&x.works), f32bits(&y.works));
    assert_eq!((&x.works_id, &x.trade, &x.uses, &x.works_off), (&y.works_id, &y.trade, &y.uses, &y.works_off));
    assert_eq!(x.n, 13);
    let blob = record::pack(&a.state, &a.content, &a.params, &a.events, &a.rec, "a", false);
    assert!(blob == record::pack(&b.state, &b.content, &b.params, &b.events, &b.rec, "a", false), "identical runs pack to different bytes");

    // the run file: the tables with their shapes, their columns and the island's years
    let h = header(&blob);
    let (n, T, G) = (13, a.state.T_max, a.content.G);
    let shape = |name: &str| -> Vec<u64> {
        let arr = h["arrays"].as_array().unwrap().iter().find(|x| x["name"] == name).unwrap_or_else(|| panic!("no array {name}"));
        arr["shape"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect()
    };
    assert_eq!(shape("acct_goods"), vec![n, T as u64, G as u64, KG as u64]);
    assert_eq!(shape("acct_land"), vec![n, T as u64, a.content.NS as u64, accounts::KL as u64]);
    assert_eq!(shape("acct_methods"), vec![n, T as u64, a.content.M as u64, accounts::KM as u64]);
    assert_eq!(shape("acct_towns"), vec![n, T as u64, accounts::KT as u64]);
    assert_eq!(shape("acct_needs"), vec![n, T as u64, a.content.S as u64, 2]);
    assert_eq!(shape("acct_trade_off"), vec![n + 1]);
    assert_eq!(shape("acct_trade_key")[1], 3);
    assert_eq!(shape("acct_use_off"), vec![n + 1]);
    assert_eq!(shape("acct_use_key")[0], shape("acct_use_val")[0]);
    assert_eq!(shape("acct_works_off"), vec![n + 1]);
    assert_eq!(shape("acct_works")[1], KW as u64);
    assert_eq!(shape("acct_works")[0], shape("acct_works_id")[0]);
    let books = &h["accounts"];
    assert_eq!(books["goods_fields"].as_array().unwrap().len(), KG);
    assert_eq!(books["island"].as_array().unwrap().len(), n as usize);
    assert!(books["island"][0]["share_work"].as_f64().unwrap() > 0.0);
    // under the game's rules the merchants have an anchor, and the books keep it
    let anchor = a.state.acct.year(5).unwrap().goods[col("anchor")];
    assert!(anchor > 0.0);
    // a lite pack keeps the towns' tables and drops the works'
    let lite = header(&record::pack(&a.state, &a.content, &a.params, &a.events, &a.rec, "a", true));
    let names: Vec<&str> = lite["arrays"].as_array().unwrap().iter().map(|x| x["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"acct_goods") && !names.contains(&"acct_works"));

    // the live view: each year's message carries that year's island account
    let mut s = sim::start(&P, Rules::Game.content());
    let mut cursor = live::LiveCursor::new(&s.content);
    for year in 1750..=1752 {
        sim::run_year(&mut s).unwrap();
        let y = live::live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        let acc = y.accounts.expect("the island's account");
        assert_eq!(acc.year, year);
        assert_eq!(acc, a.state.acct.years[(year - 1750) as usize]);
    }
}
