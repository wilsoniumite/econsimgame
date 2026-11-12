//! Four loose ends traced with the books (docs/probes/tr-loose-a.md). Prints, by mode:
//! `merchants`, where the merchants' debts come from, year by year and good by good;
//! `wages`, a town's labour market month by month around a runaway wage; `twins`, the works
//! that are exact copies of another and where they come from; `extend`, works given more
//! capacity in a year and closed within two, and one works' last years with the closing test;
//! `market`, one town's market for one good month by month.
//!
//!     cargo run --release -p isostate-core --example loose_ends -- MODE [--seeds 1-8] [--rules game|port]
//!         [--families 6] [--end-year Y] [--set JSON]...
//!         merchants [--decades]
//!         wages --seeds S --town NAME --from YEAR --to YEAR [--works GOOD, the price to show]
//!         twins
//!         extend [--works NAME]
//!         market --seeds S --town NAME --from YEAR --to YEAR [--works GOOD, services if left out]
//!
//! The seeds run side by side under the game's rules with six families unless told otherwise.
//! Nothing here changes what a run does: each steps the island as `sim::run_year` does and
//! reads the state between the months and around the year's end.
#![allow(non_snake_case)]
use std::collections::BTreeMap;

use isostate_core::accounts::{self, GOODS_FIELDS, KG, KT, TOWN_FIELDS};
use isostate_core::*;

struct Opts {
    seeds: Vec<u64>,
    P: Params,
    rules: Rules,
    decades: bool,
    town: String,
    from: i64,
    to: i64,
    works: String,
}

fn col(fields: &[&str], name: &str) -> usize {
    fields.iter().position(|&n| n == name).unwrap_or_else(|| panic!("no column {name}"))
}

fn parse(args: &[String]) -> (String, Opts) {
    let mode = args.first().cloned().expect("a mode: merchants, wages, twins or extend");
    let mut o = Opts { seeds: (1..=8).collect(), P: game_params(), rules: Rules::Game, decades: false, town: String::new(), from: 1750, to: 1900, works: String::new() };
    let (mut families, mut end_year, mut sets) = (6usize, None, Vec::new());
    let mut i = 1;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => {
                let v = next();
                o.seeds = match v.split_once('-') {
                    Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                    None => v.split(',').map(|s| s.parse().unwrap()).collect(),
                };
            }
            "--rules" => o.rules = Rules::parse(&next()).expect("port or game"),
            "--families" => families = next().parse().unwrap(),
            "--end-year" => end_year = Some(next().parse().unwrap()),
            "--set" => sets.push(next()),
            "--town" => o.town = next(),
            "--from" => o.from = next().parse().unwrap(),
            "--to" => o.to = next().parse().unwrap(),
            "--works" => o.works = next(),
            "--decades" => {
                o.decades = true;
                i += 1;
                continue;
            }
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    o.P = o.rules.params();
    for s in &sets {
        o.P = o.P.with_json_overrides(s).expect("--set takes JSON, as {\"prices\":{\"enabled\":false}}");
    }
    o.P.n_families = families;
    o.P.end_year = end_year;
    o.P.accounts.enabled = true;
    (mode, o)
}

/// Run each seed on its own thread; `f` sees the simulation between months and years.
fn each_seed<R: Send>(o: &Opts, f: impl Fn(u64, Sim) -> R + Sync) -> Vec<(u64, R)> {
    std::thread::scope(|s| {
        let hs: Vec<_> = o
            .seeds
            .iter()
            .map(|&seed| {
                let mut P = o.P.clone();
                P.seed = seed;
                let f = &f;
                let content = o.rules.content();
                s.spawn(move || (seed, f(seed, sim::start(&P, content))))
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    })
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mode, o) = parse(&args);
    println!("{} rules, {} families, seeds {:?}, mode {mode}", o.rules.as_str(), o.P.n_families, o.seeds);
    match mode.as_str() {
        "merchants" => merchants(&o),
        "wages" => wages(&o),
        "twins" => twins(&o),
        "extend" => extend(&o),
        "market" => market(&o),
        other => panic!("unknown mode {other}"),
    }
}

/// One town's market for one good, month by month: its price and the merchants' anchor,
/// the stock, what was sold, spoiled, made and came in, and the merchants' cash.
fn market(o: &Opts) {
    let o2 = Opts { seeds: vec![o.seeds[0]], P: o.P.clone(), rules: o.rules, decades: false, town: o.town.clone(), from: o.from, to: o.to, works: o.works.clone() };
    let good = if o.works.is_empty() { "services".to_string() } else { o.works.clone() };
    each_seed(&o2, |_seed, mut sim| {
        let C = sim.content.clone();
        let P = sim.params.clone();
        let g = C.gi(&good);
        sim.st.watch_markets = true;
        println!("month     price   anchor  stock in  requests      sold   spoiled      made  came in (net)  stock out   merchants' cash   their change   cheapest source, landed");
        while sim.year <= sim.end_year.min(o.to) {
            let year = sim.year;
            for month in 0..12 {
                let t = sim.st.town_name.iter().position(|n| *n == o.town);
                let before = t.map(|t| (sim.st.inv[(t, g)], sim.st.merch_cash[t], sim.st.town_prod[(t, g)]));
                sim::run_month(&mut sim, month).expect("money is conserved");
                let st = &sim.st;
                if let (Some(t), Some((inv0, cash0, prod0)), true) = (t, before, year >= o.from) {
                    let sold = st.month_sold[(t, g)];
                    let spoiled = (inv0 - sold).max(0.0) * P.decay[g];
                    let made = st.town_prod[(t, g)] - prod0;
                    let kept = ((inv0 - sold) * (1.0 - P.decay[g])).max(0.0) + made;
                    let came = st.inv[(t, g)] - kept;
                    // the cheapest town to buy from, with the carriage and handling the trade rule charges
                    let src = (0..st.T_max)
                        .filter(|&u| u != t && st.town_active[u] && st.town_cost[(u, t)].is_finite())
                        .map(|u| (u, st.price[(u, g)] + P.freight * C.weight[g] * st.town_cost[(u, t)] + P.handling * st.price[(u, g)]))
                        .min_by(|a, b| a.1.total_cmp(&b.1));
                    let anchor = if st.price_anchor.rows > t { st.price_anchor[(t, g)] } else { f64::NAN };
                    println!(
                        "{year}-{:02} {:8.3} {:8.3} {:9.0} {:9.0} {:9.0} {:9.0} {:9.0} {:14.0} {:10.0} {:17.0} {:+14.0}   {}",
                        month + 1,
                        st.price[(t, g)],
                        anchor,
                        inv0,
                        st.month_requests[(t, g)],
                        sold,
                        spoiled,
                        made,
                        came,
                        st.inv[(t, g)],
                        st.merch_cash[t],
                        st.merch_cash[t] - cash0,
                        src.map_or(String::new(), |(u, c)| format!("{} {:.3}", st.town_name[u], c))
                    );
                }
            }
            sim::end_year(&mut sim);
        }
    });
}

// ---------------------------------------------------------------- (a) the merchants' debts

/// One year of the merchants of one island.
#[derive(Default, Clone)]
struct MerchYear {
    year: i64,
    cash: f64,
    debt: f64,
    debtors: usize,
    money: f64,
    stock_value: f64,
    net: f64,
    div: f64,
    in_year_step: f64,
    closures: f64,
    closed_in_debt: usize,
}

/// A good's part in the merchants' takings, the century added up: the four terms that
/// make up what the merchants took in less what they paid out (see the page).
#[derive(Default, Clone)]
struct GoodTerms {
    margin: f64,
    imports: f64,
    spoiled: f64,
    stock: f64,
    exports: f64,
    timing: f64,
    net_debtors: f64,
}

struct MerchRun {
    years: Vec<MerchYear>,
    goods: Vec<GoodTerms>,
    goods_debtors: Vec<GoodTerms>,
    towns_1900: Vec<(String, i64, f64, f64, Vec<(String, f64)>)>,
    residual: f64,
    net_total: f64,
}

fn merchants(o: &Opts) {
    let runs = each_seed(o, |_seed, mut sim| {
        let C = sim.content.clone();
        let (T, G) = (sim.st.T_max, C.G);
        let s = sim.params.merchant_spread;
        let (gc, tc) = (|n| col(&GOODS_FIELDS, n), |n| col(&TOWN_FIELDS, n));
        let mut years = Vec::new();
        let mut goods = vec![GoodTerms::default(); G];
        let mut goods_debtors = vec![GoodTerms::default(); G];
        let mut life: Vec<Vec<f64>> = vec![vec![0.0; G]; T]; // each town's import gain by good, over its life
        let (mut residual, mut net_total) = (0.0, 0.0);
        while sim.year <= sim.end_year {
            for month in 0..12 {
                sim::run_month(&mut sim, month).expect("money is conserved");
            }
            let before: f64 = sim.st.merch_cash.iter().sum();
            let alive: Vec<(usize, f64)> = (0..sim.st.F_count).filter(|&f| sim.st.f_alive[f]).map(|f| (f, sim.st.f_cash[f])).collect();
            sim::end_year(&mut sim);
            let st = &sim.st;
            let after: f64 = st.merch_cash.iter().sum();
            let gone: Vec<f64> = alive.iter().filter(|&&(f, _)| !st.f_alive[f]).map(|&(_, c)| c).collect();
            let k = (sim.year - 1 - C.start_year) as usize;
            let rec = st.acct.year(k).expect("the year's books");
            let mut y = MerchYear { year: sim.year - 1, cash: after, money: econ::money_total(st), in_year_step: after - before, ..Default::default() };
            y.closures = gone.iter().filter(|&&c| c < 0.0).sum();
            y.closed_in_debt = gone.iter().filter(|&&c| c < 0.0).count();
            for t in 0..T {
                let row = &rec.towns[t * KT..(t + 1) * KT];
                y.net += row[tc("merchants_net")] as f64;
                y.div += row[tc("div_merchants")] as f64;
                y.stock_value += row[tc("stock_value")] as f64;
                if st.merch_cash[t] < 0.0 {
                    y.debt -= st.merch_cash[t];
                    y.debtors += 1;
                }
            }
            // good by good, town by town: what the merchants took on each good
            let mut recv_value = vec![0.0f64; T * G];
            let mut recv_qty = vec![0.0f64; T * G];
            let mut sent_value = vec![0.0f64; T * G];
            for fl in rec.trade {
                let (a, b, g) = (fl.from as usize, fl.to as usize, fl.good as usize);
                recv_value[b * G + g] += fl.value as f64 + fl.carting as f64;
                recv_qty[b * G + g] += fl.qty as f64;
                sent_value[a * G + g] += fl.value as f64;
            }
            let lc = |n| col(&accounts::LAND_FIELDS, n);
            let mc = |n| col(&accounts::METHOD_FIELDS, n);
            let mut made_value = vec![0.0f64; T * G];
            for t in 0..T {
                for sct in 0..C.NS {
                    let r = &rec.land[(t * C.NS + sct) * accounts::KL..(t * C.NS + sct + 1) * accounts::KL];
                    made_value[t * G + C.sector_good[sct]] += r[lc("value")] as f64;
                }
                for m in 0..C.M {
                    let r = &rec.methods[(t * C.M + m) * accounts::KM..(t * C.M + m + 1) * accounts::KM];
                    made_value[t * G + C.method_out[m]] += r[mc("value")] as f64;
                }
            }
            for t in 0..T {
                let tnet = rec.towns[t * KT + tc("merchants_net")] as f64;
                if tnet == 0.0 && st.pop[t] <= 0.0 {
                    continue;
                }
                let debtor = st.merch_cash[t] < 0.0;
                let mut sum = 0.0;
                for g in 0..G {
                    let v = &rec.goods[(t * G + g) * KG..(t * G + g + 1) * KG];
                    let p = v[gc("price")] as f64;
                    let bought: f64 = (gc("households")..=gc("projects")).map(|c| v[c] as f64).sum();
                    let sent = v[gc("sent")] as f64;
                    let recv = recv_qty[t * G + g];
                    // sales at the year's mean price, less the goods' balance at that price:
                    // what the months' prices moving within the year did, and the stock that
                    // came or went with a district
                    let timing = bought * p - (made_value[t * G + g] + p * (recv - sent) - p * v[gc("spoiled")] as f64 - p * (v[gc("stock")] - v[gc("stock_open")]) as f64);
                    let tm = GoodTerms {
                        margin: s * made_value[t * G + g],
                        imports: p * recv - recv_value[t * G + g],
                        spoiled: -p * v[gc("spoiled")] as f64,
                        stock: -p * (v[gc("stock")] - v[gc("stock_open")]) as f64,
                        exports: sent_value[t * G + g] - p * sent,
                        timing,
                        net_debtors: 0.0,
                    };
                    let all = tm.margin + tm.imports + tm.spoiled + tm.stock + tm.exports + tm.timing;
                    sum += all;
                    life[t][g] += tm.imports + tm.spoiled;
                    let add = |a: &mut GoodTerms| {
                        a.margin += tm.margin;
                        a.imports += tm.imports;
                        a.spoiled += tm.spoiled;
                        a.stock += tm.stock;
                        a.exports += tm.exports;
                        a.timing += tm.timing;
                    };
                    add(&mut goods[g]);
                    if debtor {
                        add(&mut goods_debtors[g]);
                        goods_debtors[g].net_debtors += all;
                    }
                }
                residual += tnet - sum;
                net_total += tnet;
            }
            years.push(y);
        }
        let st = &sim.st;
        let mut towns: Vec<usize> = (0..T).filter(|&t| st.merch_cash[t] < 0.0).collect();
        towns.sort_by(|&a, &b| st.merch_cash[a].total_cmp(&st.merch_cash[b]));
        let towns_1900 = towns
            .iter()
            .take(6)
            .map(|&t| {
                let mut worst: Vec<(String, f64)> = (0..G).map(|g| (C.good_name[g].clone(), life[t][g])).collect();
                worst.sort_by(|a, b| a.1.total_cmp(&b.1));
                worst.truncate(3);
                (st.town_name[t].clone(), st.town_founded[t], st.merch_cash[t], st.pop[t], worst)
            })
            .collect();
        MerchRun { years, goods, goods_debtors, towns_1900, residual, net_total }
    });
    let C = default_content_for(o);
    println!("\nthe merchants, island by island (money in thousands; debt = what the merchants below nothing owe)");
    for (seed, r) in &runs {
        let first = |share: f64| r.years.iter().find(|y| y.debt > share * y.money.abs().max(1.0)).map_or("never".to_string(), |y| y.year.to_string());
        let cum = |f: fn(&MerchYear) -> f64| r.years.iter().map(f).sum::<f64>() / 1e3;
        let last = r.years.last().unwrap();
        println!(
            "seed {seed}: debt passes 1% of the money in {}, 10% in {}, 25% in {}; in {} the merchants hold {:.0} (debt {:.0} in {} towns, {:.2} of the money {:.0}), stock worth {:.0}; over the run net takings {:+.0}, dividends {:.0}, at the year's turns {:+.0} (closed works' debts {:.0}, {} works); books' net less the four terms {:+.3e} of {:.3e}",
            first(0.01),
            first(0.10),
            first(0.25),
            last.year,
            last.cash / 1e3,
            last.debt / 1e3,
            last.debtors,
            last.debt / last.money,
            last.money / 1e3,
            last.stock_value / 1e3,
            cum(|y| y.net),
            cum(|y| y.div),
            cum(|y| y.in_year_step),
            cum(|y| y.closures),
            r.years.iter().map(|y| y.closed_in_debt).sum::<usize>(),
            r.residual,
            r.net_total
        );
        for (name, founded, cash, pop, worst) in &r.towns_1900 {
            let w: Vec<String> = worst.iter().map(|(g, v)| format!("{g} {:+.0}k", v / 1e3)).collect();
            println!("    {name:>14} founded {founded}, {pop:7.0} people, merchants hold {:+.0}k; their worst goods over its life, imports and spoilage: {}", cash / 1e3, w.join(", "));
        }
    }
    if o.decades {
        println!("\ndecade by decade, mean of the islands (thousands): cash, debt, debt/money, stock value, net, dividends, at the year's turns, of which closed works' debts");
        let y0 = runs[0].1.years[0].year;
        let n = runs[0].1.years.len();
        for d in (0..n).step_by(10) {
            let span = d..(d + 10).min(n);
            let mean = |f: &dyn Fn(&MerchYear) -> f64| runs.iter().map(|(_, r)| span.clone().map(|k| f(&r.years[k])).sum::<f64>()).sum::<f64>() / runs.len() as f64 / 1e3;
            let endk = span.end - 1;
            let at_end = |f: &dyn Fn(&MerchYear) -> f64| runs.iter().map(|(_, r)| f(&r.years[endk])).sum::<f64>() / runs.len() as f64;
            println!(
                "{}s  cash {:9.0}  debt {:9.0}  {:5.3}  stock {:9.0}  net {:+9.0}  div {:9.0}  turns {:+9.0}  closures {:+9.0}",
                y0 + d as i64,
                at_end(&|y| y.cash) / 1e3,
                at_end(&|y| y.debt) / 1e3,
                at_end(&|y| y.debt / y.money),
                at_end(&|y| y.stock_value) / 1e3,
                mean(&|y| y.net),
                mean(&|y| y.div),
                mean(&|y| y.in_year_step),
                mean(&|y| y.closures)
            );
        }
    }
    println!("\ngood by good, the century and all islands added up (thousands): the spread on what the town made, imports' value here less their cost delivered, spoiled at the price, stock built up at the price, exports' value less their value here, the months' prices against the year's mean");
    println!("{:>10} {:>11} {:>11} {:>11} {:>11} {:>11} {:>11}   | in the towns whose merchants were below nothing that year: spread, imports, spoiled, stock, all", "good", "spread", "imports", "spoiled", "stock", "exports", "timing");
    let G = runs[0].1.goods.len();
    for g in 0..G {
        let sum = |f: &dyn Fn(&GoodTerms) -> f64, d: bool| runs.iter().map(|(_, r)| f(if d { &r.goods_debtors[g] } else { &r.goods[g] })).sum::<f64>() / 1e3;
        println!(
            "{:>10} {:+11.0} {:+11.0} {:+11.0} {:+11.0} {:+11.0} {:+11.0}   | {:+11.0} {:+11.0} {:+11.0} {:+11.0} {:+11.0}",
            C.good_name[g],
            sum(&|t| t.margin, false),
            sum(&|t| t.imports, false),
            sum(&|t| t.spoiled, false),
            sum(&|t| t.stock, false),
            sum(&|t| t.exports, false),
            sum(&|t| t.timing, false),
            sum(&|t| t.margin, true),
            sum(&|t| t.imports, true),
            sum(&|t| t.spoiled, true),
            sum(&|t| t.stock, true),
            sum(&|t| t.net_debtors, true)
        );
    }
}

fn default_content_for(o: &Opts) -> Content {
    o.rules.content()
}

// ---------------------------------------------------------------- (b) a runaway wage

fn wages(o: &Opts) {
    let o2 = Opts { seeds: vec![o.seeds[0]], P: o.P.clone(), rules: o.rules, decades: false, town: o.town.clone(), from: o.from, to: o.to, works: o.works.clone() };
    each_seed(&o2, |_seed, mut sim| {
        let C = sim.content.clone();
        let P = sim.params.clone();
        let T = sim.st.T_max;
        let mut prev: Option<(Vec<f64>, Vec<f64>)> = None;
        println!("month     wage  x middle   people  for hire  carters  sinkers  navvies   land hands (by sector)                         works hands   at work/for hire  floor/for hire   pexp of its main good");
        while sim.year <= sim.end_year.min(o.to) {
            let year = sim.year;
            for month in 0..12 {
                let watch = year >= o.from;
                let t = sim.st.town_name.iter().position(|n| *n == o.town);
                // the hands for hire this month, as the month will count them
                let supply = if watch && t.is_some() {
                    let mut probe = sim.st.clone();
                    let mut s: Vec<f64> = probe.pop.iter().map(|&p| P.participation * p).collect();
                    let s0 = s.clone();
                    carting::take_carters(&mut probe, &P, &mut s);
                    let s1 = s.clone();
                    mines::take_crews(&mut probe, &P, &mut s);
                    let s2 = s.clone();
                    canals::take_navvies(&mut probe, &P, &mut s);
                    let t = t.unwrap();
                    Some((s[t], s0[t] - s1[t], s1[t] - s2[t], s2[t] - s[t]))
                } else {
                    None
                };
                // what the town's employers hired last month: the floor this month's demand cannot go under
                let floor = t.map(|t| {
                    let st = &sim.st;
                    let land: f64 = (0..st.ex_Lprev.len()).filter(|&u| st.ex_town[u] == t).map(|u| st.ex_Lprev[u]).sum();
                    let works: f64 = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_town[f] == t && st.f_K[f] > 1e-9).map(|f| st.f_labor[f]).sum();
                    (1.0 - P.labor_adjust_ex) * land + (1.0 - P.labor_adjust_firm) * works
                });
                sim::run_month(&mut sim, month).expect("money is conserved");
                let st = &sim.st;
                if let (Some(t), Some((sup, carters, crews, navvies))) = (t, supply) {
                    let act: Vec<usize> = (0..T).filter(|&u| st.town_active[u] && st.pop[u] > 0.0).collect();
                    let mut ws: Vec<f64> = act.iter().map(|&u| st.wage[u]).collect();
                    ws.sort_by(f64::total_cmp);
                    let middle = ws[ws.len() / 2];
                    let mut by_sector = vec![0.0; C.NS];
                    for u in 0..st.ex_labor.len().min(st.ex_town.len()) {
                        if st.ex_town[u] == t {
                            by_sector[st.ex_sector[u]] += st.ex_labor[u];
                        }
                    }
                    let mut by_method: BTreeMap<String, f64> = BTreeMap::new();
                    for f in 0..st.F_count {
                        if st.f_alive[f] && st.f_town[f] == t && st.f_labor[f] > 0.5 {
                            *by_method.entry(C.method_name[st.f_method[f]].clone()).or_default() += st.f_labor[f];
                        }
                    }
                    let land: f64 = by_sector.iter().sum();
                    let works: f64 = by_method.values().sum();
                    let sectors: Vec<String> = (0..C.NS).filter(|&k| by_sector[k] > 0.5).map(|k| format!("{} {:.0}", C.sector_name[k], by_sector[k])).collect();
                    let methods: Vec<String> = by_method.iter().map(|(m, v)| format!("{m} {v:.0}")).collect();
                    let main = (0..C.NS).max_by(|&a, &b| by_sector[a].total_cmp(&by_sector[b])).unwrap();
                    let g = if o.works.is_empty() { C.sector_good[main] } else { C.gi(&o.works) };
                    let _ = &prev;
                    let mut elsewhere: Vec<f64> = act.iter().filter(|&&u| u != t).map(|&u| st.price[(u, g)]).collect();
                    elsewhere.sort_by(f64::total_cmp);
                    println!(
                        "{year}-{:02} {:8.2} {:8.2} {:8.0} {:9.0} {:8.0} {:8.0} {:8.0}   {:<46} {:10.0}   {:6.2}           {:6.2}          {} {:.3} (price {:.3}, the middle town's {:.3})   {}",
                        month + 1,
                        st.wage[t],
                        st.wage[t] / middle,
                        st.pop[t],
                        sup,
                        carters,
                        crews,
                        navvies,
                        sectors.join(", "),
                        works,
                        (land + works) / sup.max(1e-9),
                        floor.unwrap() / sup.max(1e-9),
                        C.good_name[g],
                        st.pexp[(t, g)],
                        st.price[(t, g)],
                        elsewhere[elsewhere.len() / 2],
                        methods.join(", ")
                    );
                }
            }
            prev = Some((sim.st.wage.clone(), sim.st.pop.clone()));
            let (pop0, price0, req0) = (sim.st.pop.clone(), sim.st.price.clone(), sim.st.requests.clone());
            sim::end_year(&mut sim);
            if let Some(t) = sim.st.town_name.iter().position(|n| *n == o.town) {
                if year >= o.from {
                    println!("  -- {year}'s turn: {} people, founded {}", sim.st.pop[t] as i64, sim.st.town_founded[t]);
                    // where newcomers came from, and what the watched good cost there and to fetch from there
                    if !o.works.is_empty() && sim.st.pop[t] > pop0[t] + 1.0 {
                        let g = C.gi(&o.works);
                        let st = &sim.st;
                        for i in (0..T).filter(|&i| i != t && st.pop[i] + 1.0 < pop0[i] && st.town_cost[(i, t)].is_finite()) {
                            let fetch = P.freight * C.weight[g] * st.town_cost[(i, t)] + P.handling * price0[(i, g)];
                            println!(
                                "     {} lost {:.0} people; there {} cost {:.3} and its buyers asked for {:.0} a month; fetched from there to {} it costs {:.3} more",
                                st.town_name[i],
                                pop0[i] - st.pop[i],
                                o.works,
                                price0[(i, g)],
                                req0[(i, g)],
                                o.town,
                                fetch
                            );
                        }
                    }
                }
            }
        }
    });
}

// ---------------------------------------------------------------- (c) twin works

fn twins(o: &Opts) {
    let runs = each_seed(o, |_seed, mut sim| {
        while sim.year <= sim.end_year {
            sim::run_year(&mut sim).expect("money is conserved");
        }
        let st = &sim.st;
        let C = &sim.content;
        // exact twins: same cell, method, year opened and year closed
        let mut groups: BTreeMap<(usize, usize, i64, i64), Vec<usize>> = BTreeMap::new();
        for f in 0..st.F_count {
            groups.entry((st.f_cell[f], st.f_method[f], st.f_born[f], st.f_died[f])).or_default().push(f);
        }
        let mut by_kind: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        let (mut works_in_twins, mut same_K, mut same_name, mut biggest, mut standing_1900) = (0usize, 0usize, 0usize, 0usize, 0usize);
        let mut example = String::new();
        for (key, fs) in &groups {
            let kind = st.f_reason[fs[0]].kind.clone();
            let e = by_kind.entry(format!("{} {}", kind, if key.2 < C.start_year { "before 1750" } else { "after" })).or_default();
            e.1 += fs.len();
            if fs.len() > 1 {
                e.0 += fs.len();
                works_in_twins += fs.len();
                biggest = biggest.max(fs.len());
                same_K += fs.iter().all(|&f| st.f_K[f] == st.f_K[fs[0]]) as usize;
                same_name += fs.iter().all(|&f| st.f_name[f] == st.f_name[fs[0]]) as usize;
                standing_1900 += (key.3 < 0) as usize;
                if example.is_empty() && fs.len() >= 3 {
                    let names: Vec<&str> = fs.iter().map(|&f| st.f_name[f].as_str()).collect();
                    example = format!("{} of {} at cell {} ({}), opened {}, closed {}: {}", fs.len(), C.method_name[key.1], key.0, st.town_name[st.f_town[fs[0]]], key.2, key.3, names.join(" / "));
                }
            }
        }
        // the seed works of 1749: how many of a town's works of one method stood on one cell
        let mut seed_cells: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for f in 0..st.F_count {
            if st.f_born[f] < C.start_year {
                *seed_cells.entry((st.f_cell[f], st.f_method[f])).or_default() += 1;
            }
        }
        let seeded: usize = seed_cells.values().sum();
        let shared: usize = seed_cells.values().filter(|&&n| n > 1).sum();
        (st.F_count, works_in_twins, groups.values().filter(|v| v.len() > 1).count(), biggest, same_K, same_name, standing_1900, by_kind, seeded, shared, example)
    });
    for (seed, (n, inw, ng, big, sk, sn, s1900, kinds, seeded, shared, ex)) in runs {
        println!("seed {seed}: {n} works in the run; {inw} are exact twins (same cell, method, year opened, year closed) in {ng} sets, at most {big} in a set; {sk} sets still alike in capacity at the end, {sn} with one name, {s1900} still standing in the last year");
        println!("    the works of 1749: {seeded}, of which {shared} share their cell with another of their kind");
        for (k, (tw, all)) in kinds {
            println!("    {k:>24}: {tw} twins of {all}");
        }
        println!("    for example {ex}");
    }
}

// ---------------------------------------------------------------- (d) extended, then closed

fn extend(o: &Opts) {
    let runs = each_seed(o, |_seed, mut sim| {
        let C = sim.content.clone();
        let P = sim.params.clone();
        let mut ordered_at: BTreeMap<usize, Vec<(i64, f64, f64)>> = BTreeMap::new(); // firm -> (year, K ordered, cost)
        let mut lines = Vec::new();
        let mut watch: Option<usize> = None;
        while sim.year <= sim.end_year {
            for month in 0..12 {
                sim::run_month(&mut sim, month).expect("money is conserved");
            }
            let year = sim.year;
            let st = &sim.st;
            if watch.is_none() && !o.works.is_empty() {
                watch = (0..st.F_count).find(|&f| st.f_name[f] == o.works);
            }
            let before: Vec<(f64, f64, bool)> = (0..st.F_count).map(|f| (st.f_order_K[f], st.f_K[f], st.f_alive[f])).collect();
            let fam_cash: Vec<f64> = st.families.iter().map(|f| f.cash).collect();
            // the closing test as the year's review will put it, read before the review
            if let Some(f) = watch.filter(|&f| st.f_alive[f]) {
                let m = st.f_method[f];
                let i = st.f_town[f];
                let unit: f64 = (0..C.G).map(|g| C.m_cap[(m, g)] * st.price[(i, g)]).sum();
                let capval = st.f_K[f].max(1e-9) * unit;
                let roi = st.f_profit_acc[f] / capval;
                let owner = st.f_owner[f];
                lines.push(format!(
                    "{year}: capacity {:.0} (+{:.0} on order), usual size {:.0}, largest allowed {:.0}; a year's profit {:.0}, return {:.3} on capacity worth {:.0}; utilisation {:.2}; cash {:.0}, a month's costs {:.0} (closes below {:.0}); years at a loss {}; owner {} with {:.0} in hand",
                    st.f_K[f],
                    st.f_order_K[f],
                    C.m_K0[m],
                    works::max_K(&C, &P, m),
                    st.f_profit_acc[f],
                    roi,
                    capval,
                    st.f_u[f],
                    st.f_cash[f],
                    st.f_cost_m[f],
                    -(6.0 * st.f_cost_m[f]).max(0.4 * capval) - 1.0,
                    st.f_loss_years[f],
                    owner,
                    if owner >= 0 { st.families[owner as usize].cash } else { 0.0 }
                ));
            }
            sim::end_year(&mut sim);
            let st = &sim.st;
            for f in 0..before.len() {
                let (ok0, k0, alive0) = before[f];
                if alive0 && st.f_alive[f] && st.f_order_K[f] > ok0 + 1e-9 && k0 > 0.0 {
                    let dK = st.f_order_K[f] - ok0;
                    let i = st.f_town[f];
                    let m = st.f_method[f];
                    let unit: f64 = (0..C.G).map(|g| C.m_cap[(m, g)] * st.price[(i, g)]).sum();
                    ordered_at.entry(f).or_default().push((year, dK, dK * unit));
                }
            }
            if let Some(f) = watch {
                let k = st.f_owner[f];
                if k >= 0 {
                    let k = k as usize;
                    lines.push(format!("    after {year}'s turn: alive {}, capacity {:.0} (+{:.0} on order), cash {:.0}; the family's cash {:.0} -> {:.0}", st.f_alive[f], st.f_K[f], st.f_order_K[f], st.f_cash[f], fam_cash[k], st.families[k].cash));
                }
            }
        }
        let st = &sim.st;
        if let Some(wb) = watch.and_then(|f| accounts::works_books(st, &C, f)) {
            lines.push(format!("{} in its books: year, capacity at the year's end, output, revenue, wages, profit, dividends, paid for capital goods, utilisation, cash at the year's end", wb.name));
            for y in &wb.years {
                lines.push(format!("    {}  {:8.0} {:9.0} {:9.0} {:8.0} {:8.0} {:9.0} {:9.0}  {:.2} {:9.0}", y.year, y.capacity, y.out, y.revenue, y.wages, y.profit, y.dividends, y.invest, y.util, y.cash));
            }
            // what a unit of the works' capacity cost in its town, year by year (the capital goods at the year's mean prices)
            let (f, gc) = (watch.unwrap(), |n| col(&GOODS_FIELDS, n));
            let (m, t) = (st.f_method[f], st.f_town[f]);
            for k in 0..st.acct.years.len() {
                let rec = st.acct.year(k).unwrap();
                let year = st.acct.years[k].year;
                if (wb.years.first().map_or(0, |y| y.year)..=wb.years.last().map_or(0, |y| y.year)).contains(&year) {
                    let parts: Vec<String> = (0..C.G)
                        .filter(|&g| C.m_cap[(m, g)] > 0.0)
                        .map(|g| format!("{} {:.2} x {:.3}", C.good_name[g], rec.goods[(t * C.G + g) * KG + gc("price")], C.m_cap[(m, g)]))
                        .collect();
                    let unit: f64 = (0..C.G).map(|g| C.m_cap[(m, g)] * rec.goods[(t * C.G + g) * KG + gc("price")] as f64).sum();
                    lines.push(format!("    {year}: a unit of capacity costs {unit:.1} at the town's mean prices ({})", parts.join(", ")));
                }
            }
        }
        // standing works given capacity in a year and closed within two years of it
        let (mut n_all, mut n_fam, mut cost_all, mut cost_fam, mut n_orders, mut cost_orders) = (0usize, 0usize, 0.0, 0.0, 0usize, 0.0);
        let mut worst: Vec<(f64, String)> = Vec::new();
        for (&f, orders) in &ordered_at {
            for &(y, _dK, cost) in orders {
                n_orders += 1;
                cost_orders += cost;
                if st.f_died[f] >= 0 && st.f_died[f] - y <= 2 && st.f_died[f] > y {
                    n_all += 1;
                    cost_all += cost;
                    if st.f_owner[f] >= 0 {
                        n_fam += 1;
                        cost_fam += cost;
                    }
                    worst.push((cost, format!("{} ({}, {}, owner {}) given {:.0} in {y}, closed {}", st.f_name[f], C.method_name[st.f_method[f]], st.town_name[st.f_town[f]], st.f_owner[f], cost, st.f_died[f])));
                }
            }
        }
        worst.sort_by(|a, b| b.0.total_cmp(&a.0));
        worst.truncate(5);
        (lines, n_orders, cost_orders, n_all, cost_all, n_fam, cost_fam, worst)
    });
    for (seed, (lines, n_orders, cost_orders, n_all, cost_all, n_fam, cost_fam, worst)) in runs {
        println!("seed {seed}: {n_orders} orders of more capacity for standing works, worth {cost_orders:.0}; {n_all} of them ({cost_all:.0}) for works that closed within two years, {n_fam} ({cost_fam:.0}) of those a family's");
        for (_, w) in worst {
            println!("    {w}");
        }
        for l in lines {
            println!("  {l}");
        }
    }
}
