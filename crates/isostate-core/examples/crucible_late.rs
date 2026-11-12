//! Prints, for docs/probes/tr-loose-b.md (b), why crucible steel found late stalls:
//! for each island the year crucible steel and the coke furnace were found, the
//! prices of iron, tools and steel then, and whether the island's crucible steel
//! grew (a year's output in 1800 to 1830); and, for the islands asked for with
//! `--detail`, year by year from the find, the trade's books, what households
//! paid for a unit of "implements" from steel against tools and iron, the first
//! crucible works' own books, and the sums entry makes for a new one.
//!
//!     cargo run --release -p isostate-core --example crucible_late -- [--seeds 1-16] [--families 6]
//!         [--set JSON] [--method NAME.FIELD=V ...] [--disc TECH.GOOD=X ...] [--detail 3,4] [--years 30]
//!
//! The game's rules. `--set` lays a JSON object over the game's parameters;
//! `--method` sets one number of one method (FIELD is `labor`, `K0`, `A0`, `Q0`
//! or `b`); `--disc` sets one discovery threshold as the content file writes it
//! (`--disc crucible_steel.coal=x30`). Both may be given more than once and
//! change the content in this program only. Nothing here draws a random number
//! of the run's or changes its state: the runs are the game's.
#![allow(non_snake_case)]
use std::fmt::Write as _;

use isostate_core::accounts::{KG, KM};
use isostate_core::*;

struct Opts {
    seeds: Vec<u64>,
    families: usize,
    set: Option<String>,
    methods: Vec<String>,
    disc: Vec<String>,
    detail: Vec<u64>,
    years: i64,
}

fn content(o: &Opts) -> Content {
    let mut c = Rules::Game.content();
    for s in &o.methods {
        let (key, v) = s.split_once('=').expect("NAME.FIELD=V");
        let (name, field) = key.split_once('.').expect("NAME.FIELD=V");
        let m = c.method_name.iter().position(|n| n == name).unwrap_or_else(|| panic!("no method {name}"));
        let v: f64 = v.parse().expect("a number");
        match field {
            "labor" => c.m_labor[m] = v,
            "K0" => c.m_K0[m] = v,
            "A0" => c.m_A0[m] = v,
            "Q0" => c.m_Q0[m] = v,
            "b" => c.m_b[m] = v,
            other => panic!("no field {other}: labor, K0, A0, Q0 or b"),
        }
    }
    for s in &o.disc {
        let (key, v) = s.split_once('=').expect("TECH.GOOD=X");
        let (name, good) = key.split_once('.').expect("TECH.GOOD=X");
        let t = c.tech_name.iter().position(|n| n == name).unwrap_or_else(|| panic!("no invention {name}"));
        let g = c.gi(good);
        // as in the content file: a plain number is cumulative output, "x30" thirty years of 1750's
        c.tech_disc[(t, g)] = match v.strip_prefix('x') {
            Some(x) => -x.parse::<f64>().expect("a number"),
            None => v.parse().expect("a number"),
        };
    }
    c
}

/// What one island's crucible steel did.
#[derive(Default)]
struct Island {
    seed: u64,
    found: Option<i64>,
    town: String,
    coke: Option<i64>,
    converter: Option<i64>,
    /// At the find: island means of iron's, tools' and steel's prices and the wage.
    at_find: [f64; 4],
    /// Ten years after the find: the same.
    after10: [f64; 4],
    /// Crucible steel made a year, 1800 to 1830.
    out_1800_30: f64,
    /// Steel's price a unit of implements over the cheapest good's, in the town
    /// that found it, in the find year and the next five (mean).
    ratio5: f64,
    /// The first crucible works: founded, its capacity then and ten years on, its
    /// output over the first ten years, its profit over them, when it closed.
    first: Option<(i64, f64, f64, f64, f64, i64)>,
    /// Crucible works founded in the first twenty years after the find.
    founded20: usize,
    detail: String,
}

fn island_mean(st: &State, act: &[usize], g: usize) -> f64 {
    act.iter().map(|&i| st.pbar[(i, g)]).sum::<f64>() / act.len().max(1) as f64
}

fn trace(seed: u64, P: &Params, C: Content, detail: bool, span: i64) -> Island {
    let mut P = P.clone();
    P.seed = seed;
    let mut s = sim::start(&P, C);
    let C = s.content.clone();
    let mi = |n: &str| C.method_name.iter().position(|x| x == n).unwrap();
    let ti = |n: &str| C.tech_name.iter().position(|x| x == n).unwrap();
    let crucible = mi("crucible_steel");
    let (t_cru, t_coke, t_conv) = (ti("crucible_steel"), ti("coke_furnace"), ti("bessemer"));
    let (iron, tools, steel) = (C.gi("iron"), C.gi("tools"), C.gi("steel"));
    let implements = C.service_name.iter().position(|n| n == "implements").unwrap();
    let S = C.service_name.len();
    let mut out = Island { seed, ..Default::default() };
    let mut d = String::new();
    let mut ratios = Vec::new();
    let mut sum_out = 0.0;
    let mut n_out: f64 = 0.0;
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved every month");
        let year = s.year - 1;
        let st = &s.st;
        let P = &s.params;
        let act: Vec<usize> = (0..st.T_max).filter(|&i| st.town_active[i]).collect();
        let prices = |st: &State| [island_mean(st, &act, iron), island_mean(st, &act, tools), island_mean(st, &act, steel), act.iter().map(|&i| st.wbar[i]).sum::<f64>() / act.len().max(1) as f64];
        if out.coke.is_none() && st.tech_year[t_coke] >= 0 {
            out.coke = Some(st.tech_year[t_coke]);
        }
        if out.converter.is_none() && st.tech_year[t_conv] >= 0 {
            out.converter = Some(st.tech_year[t_conv]);
        }
        let Some(rec) = st.acct.year(st.acct.years.len() - 1) else { panic!("the books are kept") };
        // the island's crucible steel this year
        let (mut c_out, mut c_works, mut c_cap, mut c_profit, mut c_wages) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for i in 0..st.T_max {
            let b = (i * C.M + crucible) * KM;
            c_out += rec.methods[b] as f64;
            c_works += rec.methods[b + 2] as f64;
            c_cap += rec.methods[b + 3] as f64;
            c_wages += rec.methods[b + 5] as f64;
            c_profit += rec.methods[b + 12] as f64;
        }
        if (1800..=1830).contains(&year) {
            sum_out += c_out;
            n_out += 1.0;
        }
        if out.found.is_none() && st.tech_year[t_cru] >= 0 {
            out.found = Some(st.tech_year[t_cru]);
            let town = st.tech_town[t_cru] as usize;
            out.town = st.town_name[town].clone();
            out.at_find = prices(st);
        }
        let Some(fy) = out.found else { continue };
        let town = st.tech_town[t_cru] as usize;
        if year == fy + 10 {
            out.after10 = prices(st);
        }
        // a unit of implements from each good, in the finding town: price over the good's worth for the need
        let per = |g: usize| st.pbar[(town, g)] / C.props[(g, implements)];
        let cheapest = [iron, tools, steel, C.gi("machinery")].iter().map(|&g| per(g)).fold(f64::INFINITY, f64::min);
        let ratio = per(steel) / cheapest;
        if year <= fy + 5 {
            ratios.push(ratio);
        }
        if !detail || year > fy + span {
            continue;
        }
        if year == fy {
            writeln!(d, "seed {seed}: crucible steel found {fy} at {} (coke furnace {})", out.town, out.coke.map_or("not yet".into(), |y| y.to_string())).unwrap();
            writeln!(d, "  the island's crucible steel, prices (island means), a unit of implements in the finding town, what households and landlords bought, and the first works:").unwrap();
            writeln!(
                d,
                "  year works   cap    out   skill  p_iron p_tools p_steel  wage | iron/u tools/u steel/u steel's share | hh+lords steel  tools   iron | profit wages/out  | first: cap   out  util  got/u  wage/u  mat/u   profit | new one here: K    r   pout  lab/u  in/u  margin  roi"
            )
            .unwrap();
        }
        // what households and landlords bought, island-wide, and steel's share of implements spending
        let mut bought = [0.0; 3];
        let mut imp_spend = 0.0;
        for &i in &act {
            for (k, &g) in [steel, tools, iron].iter().enumerate() {
                let r = &rec.goods[(i * C.G + g) * KG..(i * C.G + g + 1) * KG];
                bought[k] += (r[2] + r[3] + r[4]) as f64;
            }
            imp_spend += (rec.needs[(i * S + implements) * 2] + rec.needs[(i * S + implements) * 2 + 1]) as f64;
        }
        let steel_spend: f64 = act.iter().map(|&i| {
            let r = &rec.goods[(i * C.G + steel) * KG..(i * C.G + steel + 1) * KG];
            (r[2] + r[3] + r[4]) as f64 * r[18] as f64
        }).sum();
        // the first crucible works
        let first = (0..st.F_count).filter(|&f| st.f_method[f] == crucible).min_by_key(|&f| (st.f_born[f], f));
        let mut fw = String::from("  -");
        if let Some(f) = first {
            if let Some(wb) = accounts::works_books(st, &C, f) {
                if let Some(y) = wb.years.iter().find(|y| y.year == year) {
                    let o = y.out.max(1e-9);
                    fw = format!("{:6.1} {:6.1} {:5.2} {:6.1} {:6.1} {:6.1} {:8.0}", y.capacity, y.out, y.util, y.revenue / o, y.wages / o, (y.inputs + y.fuel + y.cart_in) / o, y.profit);
                } else if st.f_died[f] >= 0 && st.f_died[f] <= year {
                    fw = format!("  closed {}", st.f_died[f]);
                }
            }
        }
        // the sums entry makes for a new crucible works in the finding town, at the size entry gives it
        let mut sw = String::from("  -");
        if st.know[(town, crucible)] > 0.02 && st.town_active[town] {
            let base = P.entry_K_frac * C.m_K0[crucible];
            let K = (P.works.entry_share * works::town_trade(st, &C, town, crucible)).min(works::max_K(&C, P, crucible)).max(base);
            let cells: Vec<usize> = st.town_cells[town].iter().copied().filter(|&c| st.cell_K_share[c] < 0.9 && st.land_use[c] != 0 && st.land_use[c] != land::USE_DERELICT).collect();
            if !cells.is_empty() {
                let ev = firms::evaluate_sites(st, &C, P, town, crucible, &cells, K, &st.pbar, &st.wbar);
                let j = util::argmax(&ev.roi);
                sw = format!("{:6.2} {:4.2} {:6.1} {:6.1} {:5.1} {:7.1} {:5.2}", K, (C.m_K0[crucible] / K).sqrt().min(10.0), ev.pout[j], ev.labor_cost[j], ev.in_cost[j] + ev.heat_cost[j], ev.margin[j], ev.roi[j]);
            }
        }
        let p = prices(st);
        writeln!(
            d,
            "  {year} {:5.0} {:6.1} {:7.0} {:5.2} {:6.1} {:6.1} {:7.1} {:5.1} | {:6.1} {:6.1} {:7.1} {:12.4} | {:9.0} {:8.0} {:6.0} | {:7.0} {:6.1} |{fw} |{sw}",
            c_works, c_cap, c_out, st.method_A[crucible], p[0], p[1], p[2], p[3], per(iron), per(tools), per(steel), steel_spend / imp_spend.max(1e-9), bought[0], bought[1], bought[2], c_profit, c_wages / c_out.max(1e-9)
        )
        .unwrap();
    }
    out.out_1800_30 = sum_out / n_out.max(1.0);
    out.ratio5 = ratios.iter().sum::<f64>() / ratios.len().max(1) as f64;
    let st = &s.st;
    let firms: Vec<usize> = (0..st.F_count).filter(|&f| st.f_method[f] == crucible).collect();
    if let (Some(fy), Some(&f)) = (out.found, firms.iter().min_by_key(|&&f| (st.f_born[f], f))) {
        out.founded20 = firms.iter().filter(|&&g| st.f_born[g] <= fy + 20).count();
        if let Some(wb) = accounts::works_books(st, &C, f) {
            let born = st.f_born[f];
            let cap0 = wb.years.first().map_or(0.0, |y| y.capacity);
            let cap10 = wb.years.iter().find(|y| y.year == born + 10).map_or(0.0, |y| y.capacity);
            let out10: f64 = wb.years.iter().filter(|y| y.year < born + 10).map(|y| y.out).sum();
            let prof10: f64 = wb.years.iter().filter(|y| y.year < born + 10).map(|y| y.profit).sum();
            out.first = Some((born, cap0, cap10, out10, prof10, st.f_died[f]));
        }
    }
    out.detail = d;
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut o = Opts { seeds: (1..=16).collect(), families: 6, set: None, methods: Vec::new(), disc: Vec::new(), detail: Vec::new(), years: 30 };
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => o.seeds = sweep::parse_seeds(&next()).unwrap(),
            "--families" => o.families = next().parse().unwrap(),
            "--set" => o.set = Some(next()),
            "--method" => o.methods.push(next()),
            "--disc" => o.disc.push(next()),
            "--detail" => o.detail = next().split(',').map(|x| x.parse().unwrap()).collect(),
            "--years" => o.years = next().parse().unwrap(),
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let mut P = Rules::Game.params();
    P.n_families = o.families;
    P.accounts.enabled = true;
    if let Some(j) = &o.set {
        P = P.with_json_overrides(j).expect("a JSON object of parameters");
    }
    let C = content(&o);
    let islands: Vec<Island> = std::thread::scope(|s| {
        let hs: Vec<_> = o
            .seeds
            .iter()
            .map(|&seed| {
                let (P, C) = (P.clone(), C.clone());
                let det = o.detail.contains(&seed);
                let span = o.years;
                s.spawn(move || trace(seed, &P, C, det, span))
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });
    println!(
        "the game's rules, {} families, seeds {:?}{}{}{}",
        o.families,
        o.seeds,
        o.set.as_ref().map_or(String::new(), |s| format!(", with {s}")),
        if o.methods.is_empty() { String::new() } else { format!(", methods {:?}", o.methods) },
        if o.disc.is_empty() { String::new() } else { format!(", thresholds {:?}", o.disc) },
    );
    println!();
    println!("seed  crucible found (town)      coke  converter | at the find: iron tools steel  wage | ten years on: iron tools steel  wage | steel/u over cheapest, first 6 yrs | made a year 1800-30 | first works: founded cap  cap+10  out 10 yrs  profit 10 yrs  closed | founded in 20 yrs");
    for x in &islands {
        let f = x.first.map_or("-".to_string(), |(b, c0, c10, o10, p10, died)| format!("{b} {c0:5.1} {c10:6.1} {o10:9.0} {p10:12.0}  {}", if died >= 0 { died.to_string() } else { "-".into() }));
        println!(
            "{:4}  {:>5} ({:<16})  {:>5}  {:>5}  | {:5.1} {:5.1} {:5.1} {:5.1} | {:5.1} {:5.1} {:5.1} {:5.1} | {:6.2} | {:10.0} | {f} | {}",
            x.seed,
            x.found.map_or("-".into(), |y| y.to_string()),
            x.town,
            x.coke.map_or("-".into(), |y| y.to_string()),
            x.converter.map_or("-".into(), |y| y.to_string()),
            x.at_find[0], x.at_find[1], x.at_find[2], x.at_find[3],
            x.after10[0], x.after10[1], x.after10[2], x.after10[3],
            x.ratio5,
            x.out_1800_30,
            x.founded20
        );
    }
    let stalled: Vec<&Island> = islands.iter().filter(|x| x.out_1800_30 < 10_000.0).collect();
    println!();
    println!("stalled (under 10,000 a year in 1800-30): {} of {}: seeds {:?}", stalled.len(), islands.len(), stalled.iter().map(|x| x.seed).collect::<Vec<_>>());
    let late: Vec<&Island> = islands.iter().filter(|x| x.found.is_some_and(|y| y > 1778)).collect();
    println!("found after 1778: {} ({} of them stalled)", late.len(), late.iter().filter(|x| x.out_1800_30 < 10_000.0).count());
    for x in &islands {
        if !x.detail.is_empty() {
            println!();
            print!("{}", x.detail);
        }
    }
}
