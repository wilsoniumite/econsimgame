//! Prints what docs/probes/tr-works.md measures with the books of account: where
//! output per head comes from decade by decade and which trades and towns stop
//! adding to it, how long the works that close have lived and why they closed,
//! and what each island's worst year for the real wage was made of.
//!
//!     cargo run --release -p isostate-core --example trace_works -- [--rules port|game]
//!         [--families N] [--set JSON] [--only growth|life|worst] [SEED ...]
//!
//! `--set` takes a JSON object of parameter overrides, laid over the chosen rules
//! as a `--params` file is: `--set '{"towns":{"rent_floor":false}}'`. Seeds
//! default to 1 to 8; each runs on its own thread with the books kept, and the
//! tables are means over seeds unless they say otherwise. Under the game's rules
//! with six families the books are on already; under the port they are switched
//! on here, which changes nothing in the run (docs/rules/accounts.md).
//!
//! growth: by decade, output per head three ways (the observer's, which is
//! spending deflated by the 1750 basket; the same spending deflated by the
//! chained index; and everything made, valued at fixed prices, the island's
//! prices of 1860 to 1879), what is made for use and not worked up again, the
//! deflators, investment, and then each good's output per head, the hands and
//! output per hand of every sector and trade, and the towns by when they were
//! founded.
//!
//! life: the works that closed: how many, their lives, why the review closed
//! them (a works that stood idle and lost three years running, one that shrank
//! to nothing, one that never built, one whose debts passed its limit), by
//! trade and by the decade they were founded in, and the chance a works closes
//! in a year at each age.
//!
//! worst: each island's worst year for the real wage, taken apart: the money
//! wage and the cost of living, which goods made the cost of living rise, the
//! harvest, and the town whose wage moved the island's most.
#![allow(non_snake_case)]

use std::collections::HashMap;

use isostate_core::accounts::{GOODS_FIELDS, KG, KL, KM, KT, LAND_FIELDS, METHOD_FIELDS, TOWN_FIELDS};
use isostate_core::params::Rules;
use isostate_core::*;

fn col(fields: &[&str], name: &str) -> usize {
    fields.iter().position(|&n| n == name).unwrap_or_else(|| panic!("no column {name}"))
}

/// What one run leaves for the tables.
struct Run {
    seed: u64,
    y0: i64,
    n: usize,
    /// Annual means of the observer's series, by name.
    ann: HashMap<String, Vec<f64>>,
    // ---- from the books, by year
    /// `[year][good]`: made on the island; bought for use (households, landlords,
    /// families, farms and pits, building, railways and canals); worked up by works;
    /// the island's price, a mean over towns weighted by what each made.
    made: Vec<Vec<f64>>,
    final_use: Vec<Vec<f64>>,
    /// `[year][good]`: bought by households, landlords and families; by works being
    /// built; by railways and canals; and the merchants' stock at the year's end.
    buyers: Vec<Vec<[f64; 4]>>,
    worked_up: Vec<Vec<f64>>,
    price: Vec<Vec<f64>>,
    /// `[year][sector]`: hands and output on the land.
    land_hands: Vec<Vec<f64>>,
    land_out: Vec<Vec<f64>>,
    /// `[year][method]`: hands, output and capacity in works.
    m_hands: Vec<Vec<f64>>,
    m_out: Vec<Vec<f64>>,
    m_cap: Vec<Vec<f64>>,
    /// `[year]`: carters and navvies.
    carters: Vec<f64>,
    navvies: Vec<f64>,
    /// `[year][town]`: people at the year's end, mean wage, value made (land and works, at the town's prices).
    town_pop: Vec<Vec<f64>>,
    town_wage: Vec<Vec<f64>>,
    town_made: Vec<Vec<Vec<f64>>>,
    town_founded: Vec<i64>,
    town_name: Vec<String>,
    /// `[year]`: the island's income, works' investment, railways' and canals' outlay.
    income: Vec<f64>,
    invest: Vec<f64>,
    projects: Vec<f64>,
    harvest: Vec<f64>,
    // ---- the works
    works: Vec<Works>,
    basket: Vec<f64>,
    /// `[year][method]`: what each method has learned (`firms::learning`), and how good machines are.
    learned: Vec<Vec<f64>>,
    quality: Vec<f64>,
    learn_cap: f64,
    quality_cap: f64,
}

struct Works {
    born: i64,
    /// The year it closed, or None if it stood at the end.
    died: Option<i64>,
    method: usize,
    /// Capacity at its death (or now) over its trade's usual size.
    size: f64,
    loss_years: i64,
    sister: bool,
    family: bool,
}

fn gather(P: &Params, C: Content) -> Run {
    // stepped a year at a time, to note how far each method has learned and how good machines are
    let mut s = sim::start(P, C);
    s.check_money = true;
    let (mut learned, mut quality) = (Vec::new(), Vec::new());
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved every month");
        learned.push(firms::learning(&s.content, &s.params, &s.st.method_Q));
        quality.push(machines::quality(&s.st, &s.content, &s.params));
    }
    let r = sim::finish(s);
    let (st, C) = (&r.state, &r.content);
    let y0 = C.start_year;
    let n = r.rec.n_years;
    let ann_v = facts::annual_means(&r);
    let ann: HashMap<String, Vec<f64>> = st.series.index.iter().map(|(k, &i)| (k.clone(), ann_v[i].clone())).collect();
    let (G, NS, M, T) = (C.G, C.NS, C.M, st.T_max);
    let gc = |name: &str| col(&GOODS_FIELDS, name);
    let (g_made, g_price) = (gc("made"), gc("price"));
    let g_use: Vec<usize> = ["households", "landlords", "families", "land", "building", "projects"].iter().map(|n| gc(n)).collect();
    let g_up: Vec<usize> = ["works_inputs", "works_fuel"].iter().map(|n| gc(n)).collect();
    let (l_out, l_hands, l_value) = (col(&LAND_FIELDS, "out"), col(&LAND_FIELDS, "hands"), col(&LAND_FIELDS, "value"));
    let (m_out, m_hands, m_value, m_capc) = (col(&METHOD_FIELDS, "out"), col(&METHOD_FIELDS, "hands"), col(&METHOD_FIELDS, "value"), col(&METHOD_FIELDS, "capacity"));
    let (t_pop, t_wage, t_cart, t_navvy) = (col(&TOWN_FIELDS, "pop"), col(&TOWN_FIELDS, "wage"), col(&TOWN_FIELDS, "carters"), col(&TOWN_FIELDS, "navvies"));
    let mut run = Run {
        seed: P.seed,
        y0,
        n,
        ann,
        made: Vec::new(),
        final_use: Vec::new(),
        buyers: Vec::new(),
        worked_up: Vec::new(),
        price: Vec::new(),
        land_hands: Vec::new(),
        land_out: Vec::new(),
        m_hands: Vec::new(),
        m_out: Vec::new(),
        m_cap: Vec::new(),
        carters: Vec::new(),
        navvies: Vec::new(),
        town_pop: Vec::new(),
        town_wage: Vec::new(),
        town_made: Vec::new(),
        town_founded: st.town_founded.clone(),
        town_name: st.town_name.clone(),
        income: Vec::new(),
        invest: Vec::new(),
        projects: Vec::new(),
        harvest: r.rec.harvest.iter().map(|&v| v as f64).collect(),
        works: Vec::new(),
        basket: st.cpi_basket.clone(),
        learned,
        quality,
        learn_cap: P.learn_cap,
        quality_cap: P.machines.quality_cap,
    };
    for k in 0..n {
        let rec = st.acct.year(k).expect("the books of every year");
        let (mut made, mut fin, mut up, mut pv) = (vec![0.0; G], vec![0.0; G], vec![0.0; G], vec![0.0; G]);
        let mut who = vec![[0.0f64; 4]; G];
        let (mut lh, mut lo) = (vec![0.0; NS], vec![0.0; NS]);
        let (mut mh, mut mo, mut mc) = (vec![0.0; M], vec![0.0; M], vec![0.0; M]);
        let (mut tp, mut tw, mut tm) = (vec![0.0; T], vec![0.0; T], vec![vec![0.0; G]; T]);
        let (mut cart, mut navvy) = (0.0, 0.0);
        for t in 0..T {
            for g in 0..G {
                let v = &rec.goods[(t * G + g) * KG..][..KG];
                let q = v[g_made] as f64;
                made[g] += q;
                pv[g] += q * v[g_price] as f64;
                fin[g] += g_use.iter().map(|&c| v[c] as f64).sum::<f64>();
                up[g] += g_up.iter().map(|&c| v[c] as f64).sum::<f64>();
                who[g][0] += (v[gc("households")] + v[gc("landlords")] + v[gc("families")]) as f64;
                who[g][1] += v[gc("building")] as f64;
                who[g][2] += v[gc("projects")] as f64;
                who[g][3] += v[gc("stock")] as f64;
            }
            for s in 0..NS {
                let v = &rec.land[(t * NS + s) * KL..][..KL];
                lh[s] += v[l_hands] as f64;
                lo[s] += v[l_out] as f64;
                tm[t][C.sector_good[s]] += v[l_value] as f64;
            }
            for m in 0..M {
                let v = &rec.methods[(t * M + m) * KM..][..KM];
                mh[m] += v[m_hands] as f64;
                mo[m] += v[m_out] as f64;
                mc[m] += v[m_capc] as f64;
                tm[t][C.method_out[m]] += v[m_value] as f64;
            }
            let v = &rec.towns[t * KT..][..KT];
            tp[t] = v[t_pop] as f64;
            tw[t] = v[t_wage] as f64;
            cart += v[t_cart] as f64;
            navvy += v[t_navvy] as f64;
        }
        let nat = |g: usize| run.ann.get(&format!("price_{}", C.good_name[g])).map_or(f64::NAN, |s| s[k]);
        let price: Vec<f64> = (0..G).map(|g| if made[g] > 0.0 { pv[g] / made[g] } else { nat(g) }).collect();
        run.made.push(made);
        run.final_use.push(fin);
        run.buyers.push(who);
        run.worked_up.push(up);
        run.price.push(price);
        run.land_hands.push(lh);
        run.land_out.push(lo);
        run.m_hands.push(mh);
        run.m_out.push(mo);
        run.m_cap.push(mc);
        run.carters.push(cart);
        run.navvies.push(navvy);
        run.town_pop.push(tp);
        run.town_wage.push(tw);
        run.town_made.push(tm);
        let y = &st.acct.years[k];
        run.income.push(y.income);
        run.invest.push(y.invest);
        run.projects.push(y.projects);
    }
    let y1 = y0 + n as i64 - 1;
    for f in 0..st.F_count {
        if st.f_born[f] > y1 {
            continue;
        }
        let m = st.f_method[f];
        run.works.push(Works {
            born: st.f_born[f],
            died: if st.f_alive[f] { None } else { Some(st.f_died[f]) },
            method: m,
            size: st.f_K[f] / C.m_K0[m],
            loss_years: st.f_loss_years[f],
            sister: st.f_parent[f] >= 0,
            family: st.f_owner.get(f).is_some_and(|&o| o >= 0),
        });
    }
    run
}

fn mean(v: &[f64]) -> f64 {
    let v: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 }
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let h = v.len() / 2;
    if v.len() % 2 == 1 { v[h] } else { 0.5 * (v[h - 1] + v[h]) }
}

/// The decades of the run: (label, first year, last year); the last takes in the final year.
fn decades(y0: i64, n: usize) -> Vec<(i64, i64, i64)> {
    let y1 = y0 + n as i64 - 1;
    let mut v = Vec::new();
    let mut d = y0.div_euclid(10) * 10;
    while d <= y1 {
        let end = if d + 19 > y1 { y1 } else { d + 9 };
        v.push((d, d.max(y0), end));
        if end == y1 {
            break;
        }
        d += 10;
    }
    v
}

/// The mean over a decade's years of `f(k)`, then the mean over runs.
fn dec_mean(runs: &[Run], (_, a, b): (i64, i64, i64), f: impl Fn(&Run, usize) -> f64) -> f64 {
    mean(&runs.iter().map(|r| mean(&(a..=b).map(|y| f(r, (y - r.y0) as usize)).collect::<Vec<_>>())).collect::<Vec<_>>())
}

fn print_row(label: &str, vals: &[f64], digits: usize) {
    print!("{label:<46}");
    for v in vals {
        print!("{:>8.*}", digits, v);
    }
    println!();
}

fn growth(runs: &[Run], C: &Content) {
    let r0 = &runs[0];
    let ds = decades(r0.y0, r0.n);
    let G = C.G;
    // fixed prices: each good's island price over 1860 to 1879, weighted by what was made
    let fixed: Vec<Vec<f64>> = runs
        .iter()
        .map(|r| {
            (0..G)
                .map(|g| {
                    let ks: Vec<usize> = (1860..=1879).map(|y| (y - r.y0) as usize).filter(|&k| k < r.n).collect();
                    let q: f64 = ks.iter().map(|&k| r.made[k][g]).sum();
                    if q > 0.0 { ks.iter().map(|&k| r.made[k][g] * r.price[k][g]).sum::<f64>() / q } else { mean(&ks.iter().map(|&k| r.price[k][g]).collect::<Vec<_>>()) }
                })
                .collect()
        })
        .collect();
    let pop = |r: &Run, k: usize| r.ann["pop"][k];
    let fixed_of = |r: &Run| &fixed[runs.iter().position(|x| x.seed == r.seed).unwrap()];
    let gross = |r: &Run, k: usize| (0..G).map(|g| r.made[k][g] * fixed_of(r)[g]).sum::<f64>() / pop(r, k);
    let fin = |r: &Run, k: usize| (0..G).map(|g| r.final_use[k][g] * fixed_of(r)[g]).sum::<f64>() / pop(r, k);
    println!("\n== Output per head, by decade (means over seeds; the indices are over the 1750s)");
    print!("{:<46}", "");
    for d in &ds {
        print!("{:>8}", format!("{}s", d.0));
    }
    println!();
    let rows: Vec<(&str, Box<dyn Fn(&Run, usize) -> f64>, bool)> = vec![
        ("people, thousands", Box::new(|r: &Run, k: usize| pop(r, k) / 1000.0), false),
        ("output per head, the observer's (gdp_pc)", Box::new(|r: &Run, k: usize| r.ann["gdp_pc"][k]), true),
        ("spending per head over the chained index", Box::new(|r: &Run, k: usize| r.ann["gdp_nom"][k] / pop(r, k) / r.ann["cpi_chained"][k]), true),
        ("all that is made, per head, at fixed prices", Box::new(gross), true),
        ("made for use, per head, at fixed prices", Box::new(fin), true),
        ("the 1750 basket's cost of living (cpi)", Box::new(|r: &Run, k: usize| r.ann["cpi"][k]), true),
        ("the chained cost of living", Box::new(|r: &Run, k: usize| r.ann["cpi_chained"][k]), true),
        ("real wage, 1750 basket", Box::new(|r: &Run, k: usize| r.ann["real_wage"][k]), true),
        ("real wage, chained", Box::new(|r: &Run, k: usize| r.ann["real_wage_chained"][k]), true),
        ("money wage", Box::new(|r: &Run, k: usize| r.ann["wage"][k]), true),
        ("works' investment over income", Box::new(|r: &Run, k: usize| r.invest[k] / r.income[k]), false),
        ("railways' and canals' outlay over income", Box::new(|r: &Run, k: usize| r.projects[k] / r.income[k]), false),
    ];
    for (label, f, index) in &rows {
        let vals: Vec<f64> = ds.iter().map(|&d| dec_mean(runs, d, f)).collect();
        if *index {
            // the index is taken seed by seed, then averaged
            let v: Vec<f64> = ds.iter().map(|&d| mean(&runs.iter().map(|r| dec_mean(std::slice::from_ref(r), d, f) / dec_mean(std::slice::from_ref(r), ds[0], f)).collect::<Vec<_>>())).collect();
            print_row(label, &v, 2);
        } else {
            print_row(label, &vals, if vals.iter().all(|v| v.abs() < 1.0) { 3 } else { 0 });
        }
    }
    // the peak decade of each measure, seed by seed
    for (label, f) in [("gdp_pc", &rows[1].1), ("fixed prices, all made", &rows[3].1), ("fixed prices, for use", &rows[4].1)] {
        let peaks: Vec<String> = runs
            .iter()
            .map(|r| {
                let v: Vec<f64> = ds.iter().map(|&d| dec_mean(std::slice::from_ref(r), d, f)).collect();
                let k = (0..v.len()).max_by(|&a, &b| v[a].total_cmp(&v[b])).unwrap();
                format!("{}", ds[k].0)
            })
            .collect();
        println!("  the decade {label} peaks in, seed by seed: {}", peaks.join(" "));
    }

    // each good, per head, at fixed prices, and its share of the change
    let late: Vec<(i64, i64, i64)> = ds.iter().copied().filter(|d| d.0 >= 1840).collect();
    println!("\n== Each good made, per head, at fixed prices (means over seeds; 1870s = 1); its part in the change of all that is made per head, from the 1850s to the 1870s and from the 1870s to the 1890s (as a share of the earlier decade's total); and its price");
    print!("{:<16}", "");
    for d in &late {
        print!("{:>8}", format!("{}s", d.0));
    }
    println!("{:>12}{:>12}{:>12}{:>12}{:>12}", "share 1870s", "part 50-70", "part 70-90", "use 70-90", "price 90/70");
    let d50 = *ds.iter().find(|d| d.0 == 1850).unwrap();
    let d70 = *ds.iter().find(|d| d.0 == 1870).unwrap();
    let d90 = *ds.iter().find(|d| d.0 == 1890).unwrap();
    let total = |r: &Run, d: (i64, i64, i64), use_: bool| dec_mean(std::slice::from_ref(r), d, |r, k| if use_ { fin(r, k) } else { gross(r, k) });
    for g in 0..G {
        let per = |r: &Run, k: usize| r.made[k][g] * fixed_of(r)[g] / pop(r, k);
        let per_use = |r: &Run, k: usize| r.final_use[k][g] * fixed_of(r)[g] / pop(r, k);
        let base = |r: &Run| dec_mean(std::slice::from_ref(r), d70, per);
        let share = mean(&runs.iter().map(|r| base(r) / total(r, d70, false)).collect::<Vec<_>>());
        // a good with next to nothing made in the 1870s (charcoal) has no index worth printing
        let v: Vec<f64> = late.iter().map(|&d| if share < 0.001 { f64::NAN } else { mean(&runs.iter().map(|r| dec_mean(std::slice::from_ref(r), d, per) / base(r)).collect::<Vec<_>>()) }).collect();
        let part_early = mean(&runs.iter().map(|r| (base(r) - dec_mean(std::slice::from_ref(r), d50, per)) / total(r, d50, false)).collect::<Vec<_>>());
        let part = mean(&runs.iter().map(|r| (dec_mean(std::slice::from_ref(r), d90, per) - base(r)) / total(r, d70, false)).collect::<Vec<_>>());
        let part_use = mean(&runs.iter().map(|r| (dec_mean(std::slice::from_ref(r), d90, per_use) - dec_mean(std::slice::from_ref(r), d70, per_use)) / total(r, d70, true)).collect::<Vec<_>>());
        let pr = mean(&runs.iter().map(|r| dec_mean(std::slice::from_ref(r), d90, |r, k| r.price[k][g]) / dec_mean(std::slice::from_ref(r), d70, |r, k| r.price[k][g])).collect::<Vec<_>>());
        print!("{:<16}", C.good_name[g]);
        for x in &v {
            print!("{:>8.2}", x);
        }
        println!("{:>12.3}{:>+12.4}{:>+12.4}{:>+12.4}{:>12.2}", share, part_early, part, part_use, pr);
    }
    let early: Vec<f64> = runs.iter().map(|r| total(r, d70, false) / total(r, d50, false) - 1.0).collect();
    let all: Vec<f64> = runs.iter().map(|r| total(r, d90, false) / total(r, d70, false) - 1.0).collect();
    let use_: Vec<f64> = runs.iter().map(|r| total(r, d90, true) / total(r, d70, true) - 1.0).collect();
    println!("  all goods, 1870s over 1850s less one: {:+.4}; 1890s over 1870s: {:+.4} (seeds {}), for use {:+.4}", mean(&early), mean(&all), all.iter().map(|v| format!("{v:+.3}")).collect::<Vec<_>>().join(" "), mean(&use_));

    // how far each method has learned, against the cap, and how good machines are
    println!("\n== What each method has learned (firms::learning, the island's, over learn_cap), and the machines' quality (means over seeds, decade's last year)");
    print!("{:<22}", "");
    for d in &late {
        print!("{:>8}", format!("{}s", d.0));
    }
    println!();
    for m in 0..C.M {
        let v: Vec<f64> = late.iter().map(|&(_, _, b)| mean(&runs.iter().map(|r| r.learned[(b - r.y0) as usize][m] / r.learn_cap).collect::<Vec<_>>())).collect();
        print!("{:<22}", C.method_name[m]);
        for x in &v {
            print!("{:>8.2}", x);
        }
        println!();
    }
    let q: Vec<f64> = late.iter().map(|&(_, _, b)| mean(&runs.iter().map(|r| r.quality[(b - r.y0) as usize]).collect::<Vec<_>>())).collect();
    print_row("machines' quality", &q, 2);
    let caps: Vec<String> = runs.iter().map(|r| r.quality.iter().position(|&x| x >= 0.99 * r.quality_cap).map_or("-".into(), |k| format!("{}", r.y0 + k as i64))).collect();
    println!("  the year machines' quality came within 1 per cent of its cap, seed by seed (- never): {}", caps.join(" "));

    // hands and output per hand
    println!("\n== Hands by where they work (share of all hands) and output per hand (1870s = 1), means over seeds");
    print!("{:<22}", "");
    for d in &late {
        print!("{:>8}", format!("{}s", d.0));
    }
    print!("   |");
    for d in &late {
        print!("{:>8}", format!("{}s", d.0));
    }
    println!();
    let hands_all = |r: &Run, k: usize| r.land_hands[k].iter().sum::<f64>() + r.m_hands[k].iter().sum::<f64>() + r.carters[k] + r.navvies[k];
    let mut lines: Vec<(String, Box<dyn Fn(&Run, usize) -> f64 + '_>, Box<dyn Fn(&Run, usize) -> f64 + '_>)> = Vec::new();
    for s in 0..C.NS {
        lines.push((format!("land: {}", C.sector_name[s]), Box::new(move |r: &Run, k: usize| r.land_hands[k][s]), Box::new(move |r: &Run, k: usize| r.land_out[k][s] / r.land_hands[k][s])));
    }
    for m in 0..C.M {
        lines.push((format!("works: {}", C.method_name[m]), Box::new(move |r: &Run, k: usize| r.m_hands[k][m]), Box::new(move |r: &Run, k: usize| r.m_out[k][m] / r.m_hands[k][m])));
    }
    lines.push(("carters".into(), Box::new(|r: &Run, k: usize| r.carters[k]), Box::new(|_: &Run, _: usize| f64::NAN)));
    lines.push(("navvies".into(), Box::new(|r: &Run, k: usize| r.navvies[k]), Box::new(|_: &Run, _: usize| f64::NAN)));
    for (label, h, per) in &lines {
        let shares: Vec<f64> = late.iter().map(|&d| dec_mean(runs, d, |r, k| h(r, k) / hands_all(r, k))).collect();
        if shares.iter().all(|v| !(v.abs() > 0.002)) {
            continue;
        }
        let ph: Vec<f64> = late
            .iter()
            .map(|&d| mean(&runs.iter().map(|r| dec_mean(std::slice::from_ref(r), d, |r, k| per(r, k)) / dec_mean(std::slice::from_ref(r), d70, |r, k| per(r, k))).collect::<Vec<_>>()))
            .collect();
        print!("{:<22}", label.chars().take(22).collect::<String>());
        for v in &shares {
            print!("{:>8.3}", v);
        }
        print!("   |");
        for v in &ph {
            print!("{:>8.2}", v);
        }
        println!();
    }
    let hpp: Vec<f64> = late.iter().map(|&d| dec_mean(runs, d, |r, k| hands_all(r, k) / pop(r, k))).collect();
    print_row("  hands over people", &hpp, 3);

    // towns by when they were founded
    println!("\n== Towns by when they were founded: their share of people, and what they make per head at the towns' prices over the island's (means over seeds)");
    let eras: [(&str, i64, i64); 4] = [("there in 1750", i64::MIN, 1750), ("founded 1751-1800", 1751, 1800), ("founded 1801-1850", 1801, 1850), ("founded after 1850", 1851, i64::MAX)];
    print!("{:<22}", "");
    for d in &late {
        print!("{:>8}", format!("{}s", d.0));
    }
    print!("   |");
    for d in &late {
        print!("{:>8}", format!("{}s", d.0));
    }
    println!();
    for (label, a, b) in eras {
        let inn = |r: &Run, t: usize| r.town_founded.get(t).is_some_and(|&f| f >= a && f <= b && t < r.town_name.len());
        let share = |r: &Run, k: usize| {
            let all: f64 = r.town_pop[k].iter().sum();
            (0..r.town_pop[k].len()).filter(|&t| inn(r, t)).map(|t| r.town_pop[k][t]).sum::<f64>() / all
        };
        let rel = |r: &Run, k: usize| {
            let made = |t: usize| r.town_made[k][t].iter().sum::<f64>();
            let (mut pm, mut pp, mut am, mut ap) = (0.0, 0.0, 0.0, 0.0);
            for t in 0..r.town_pop[k].len() {
                am += made(t);
                ap += r.town_pop[k][t];
                if inn(r, t) {
                    pm += made(t);
                    pp += r.town_pop[k][t];
                }
            }
            if pp > 0.0 { (pm / pp) / (am / ap) } else { f64::NAN }
        };
        let s: Vec<f64> = late.iter().map(|&d| dec_mean(runs, d, share)).collect();
        let v: Vec<f64> = late.iter().map(|&d| dec_mean(runs, d, rel)).collect();
        print!("{:<22}", label);
        for x in &s {
            print!("{:>8.3}", x);
        }
        print!("   |");
        for x in &v {
            print!("{:>8.2}", x);
        }
        println!();
    }
    // the largest towns of the first seed and the island: value made per head at the towns' prices, by decade
    let r = &runs[0];
    let mut order: Vec<usize> = (0..r.town_name.len()).collect();
    let kl = r.n - 1;
    order.sort_by(|&a, &b| r.town_pop[kl][b].total_cmp(&r.town_pop[kl][a]));
    println!("  seed {}: the five largest towns of 1900, value made per head over 1870s' (towns' own prices)", r.seed);
    for &t in order.iter().take(5) {
        let f = |rr: &Run, k: usize| rr.town_made[k][t].iter().sum::<f64>() / rr.town_pop[k][t] / rr.ann["cpi"][k];
        let base = dec_mean(std::slice::from_ref(r), d70, f);
        let v: Vec<f64> = late.iter().map(|&d| dec_mean(std::slice::from_ref(r), d, f) / base).collect();
        print!("    {:<16} founded {:>5}, {:>7.0} people in 1900:", r.town_name[t], r.town_founded[t], r.town_pop[kl][t]);
        for x in &v {
            print!("{:>7.2}", x);
        }
        println!();
    }
}

fn life(runs: &[Run], C: &Content) {
    println!("\n== The works that closed (all runs pooled; the sweep's fact is the median of the first row)");
    let y1 = runs[0].y0 + runs[0].n as i64 - 1;
    let y0 = runs[0].y0;
    let closed: Vec<(&Run, &Works)> = runs.iter().flat_map(|r| r.works.iter().filter(|w| w.died.is_some_and(|d| d >= y0)).map(move |w| (r, w))).collect();
    let life_of = |w: &Works| (w.died.unwrap() - w.born) as f64;
    let n_runs = runs.len() as f64;
    let show = |label: &str, set: Vec<&Works>| {
        if set.is_empty() {
            return;
        }
        let lives: Vec<f64> = set.iter().map(|w| life_of(w)).collect();
        let sizes: Vec<f64> = set.iter().map(|w| w.size).collect();
        println!(
            "  {:<44} {:>7.1} a run   median life {:>5.1}   quartiles {:>4.0} {:>4.0}   median size at closing {:>6.2}",
            label,
            set.len() as f64 / n_runs,
            median(lives.clone()),
            { let mut l = lives.clone(); l.sort_by(f64::total_cmp); l[l.len() / 4] },
            { let mut l = lives; l.sort_by(f64::total_cmp); l[3 * l.len() / 4] },
            median(sizes)
        );
    };
    show("every works that closed from 1750 on", closed.iter().map(|x| x.1).collect());
    show("  of them founded before 1750 (the warm-up)", closed.iter().filter(|x| x.1.born < y0).map(|x| x.1).collect());
    show("  founded 1750 or later", closed.iter().filter(|x| x.1.born >= y0).map(|x| x.1).collect());
    // why: the review's four reasons, as the state leaves them (firms::review)
    let why = |w: &Works| -> &'static str {
        if w.size <= 0.0 {
            "never built (stale)"
        } else if w.loss_years >= 3 {
            "three idle losing years"
        } else if w.size < 0.03 {
            "shrank to nothing (tiny)"
        } else {
            "in debt past its limit (insolvent)"
        }
    };
    for reason in ["three idle losing years", "shrank to nothing (tiny)", "never built (stale)", "in debt past its limit (insolvent)"] {
        show(&format!("  closed for: {reason}"), closed.iter().filter(|x| why(x.1) == reason).map(|x| x.1).collect());
    }
    show("  sister works", closed.iter().filter(|x| x.1.sister).map(|x| x.1).collect());
    show("  family works", closed.iter().filter(|x| x.1.family).map(|x| x.1).collect());
    show("  the town's own capital", closed.iter().filter(|x| !x.1.family).map(|x| x.1).collect());
    println!("  by trade:");
    for m in 0..C.M {
        show(&format!("    {}", C.method_name[m]), closed.iter().filter(|x| x.1.method == m).map(|x| x.1).collect());
    }
    println!("  by the decade they were founded in (closed from 1750 on):");
    for d in (y0.div_euclid(10) * 10 - 10..=y1).step_by(10) {
        let set: Vec<&Works> = closed.iter().filter(|x| x.1.born >= d && x.1.born < d + 10).map(|x| x.1).collect();
        show(&format!("    founded in the {d}s"), set);
    }
    // the chance of closing in a year, by age: closures at that age over works-years at that age
    println!("  the chance a works closes in a year, by its age (works standing at any time from 1750 on):");
    let bands: [(i64, i64); 7] = [(0, 2), (3, 5), (6, 9), (10, 14), (15, 24), (25, 49), (50, 400)];
    let mut line = String::new();
    for (a, b) in bands {
        let (mut at_risk, mut closes) = (0.0, 0.0);
        for r in runs {
            for w in &r.works {
                let end = w.died.unwrap_or(y1);
                for y in w.born.max(y0)..=end {
                    let age = y - w.born;
                    if age >= a && age <= b {
                        at_risk += 1.0;
                        if w.died == Some(y) {
                            closes += 1.0;
                        }
                    }
                }
            }
        }
        line += &format!("  {a}-{b}: {:.3} ({:.0} works-years a run)", closes / at_risk, at_risk / n_runs);
    }
    println!("   {line}");
    let standing: Vec<f64> = runs.iter().map(|r| r.works.iter().filter(|w| w.died.is_none()).count() as f64).collect();
    let founded: Vec<f64> = runs.iter().map(|r| r.works.iter().filter(|w| w.born >= y0).count() as f64).collect();
    println!("  founded from 1750 on {:.0} a run, standing at the end {:.0}", mean(&founded), mean(&standing));
}

fn worst(runs: &[Run], C: &Content) {
    println!("\n== Each island's worst year for the real wage (1750 basket), from 1760 on, taken apart");
    println!("  seed  year  real wage   money wage   cost of living   harvest (year, year before)   what raised the cost of living most (share of its change)              the town that moved the wage most");
    let G = C.G;
    let mut by_good = vec![0.0; G];
    for r in runs {
        let rw = &r.ann["real_wage"];
        let from = (1760 - r.y0).max(1) as usize;
        let k = (from..r.n).min_by(|&a, &b| (rw[a] / rw[a - 1]).total_cmp(&(rw[b] / rw[b - 1]))).unwrap();
        let (w, cpi) = (&r.ann["wage"], &r.ann["cpi"]);
        // the cost of living's change, good by good: basket weight times the change of the national price
        let p = |g: usize, k: usize| r.ann[&format!("price_{}", C.good_name[g])][k];
        let before: f64 = (0..G).map(|g| r.basket[g] * p(g, k - 1)).sum();
        let mut parts: Vec<(usize, f64)> = (0..G).map(|g| (g, r.basket[g] * (p(g, k) - p(g, k - 1)) / before)).collect();
        let dc: f64 = parts.iter().map(|x| x.1).sum();
        for &(g, v) in &parts {
            by_good[g] += v / dc / runs.len() as f64;
        }
        parts.sort_by(|a, b| b.1.total_cmp(&a.1));
        let top: Vec<String> = parts.iter().take(3).map(|&(g, v)| format!("{} {:.2}", C.good_name[g], v / dc)).collect();
        // the island's wage is the towns' weighted by people; each town's part of its change
        let (tp, tw) = (&r.town_pop, &r.town_wage);
        let pop0: f64 = tp[k - 1].iter().sum();
        let pop1: f64 = tp[k].iter().sum();
        let mut tparts: Vec<(usize, f64)> = (0..tp[k].len()).map(|t| (t, tp[k][t] / pop1 * tw[k][t] - tp[k - 1][t] / pop0 * tw[k - 1][t])).collect();
        let dw: f64 = tparts.iter().map(|x| x.1).sum();
        tparts.sort_by(|a, b| a.1.total_cmp(&b.1));
        let (t, v) = tparts[0];
        let h = |k: usize| r.harvest.get(k).copied().unwrap_or(f64::NAN);
        println!(
            "  {:4}  {}   {:+6.1}%      {:+6.1}%        {:+6.1}%              {:.2}, {:.2}                  {:<70} {} ({:.0} people, founded {}): {:.2} of the change",
            r.seed,
            r.y0 + k as i64,
            100.0 * (rw[k] / rw[k - 1] - 1.0),
            100.0 * (w[k] / w[k - 1] - 1.0),
            100.0 * (cpi[k] / cpi[k - 1] - 1.0),
            h(k),
            h(k - 1),
            top.join(", "),
            r.town_name[t],
            tp[k][t],
            r.town_founded[t],
            v / dw
        );
        // the good that raised the cost of living most: what was made and who bought it, the two years before and the year itself
        let g = parts[0].0;
        let line: Vec<String> = (k.saturating_sub(2)..=k)
            .map(|j| {
                let b = r.buyers[j][g];
                format!("{}: made {:.0}, households {:.0}, building {:.0}, ways {:.0}, stock at the end {:.0}, price {:.2}", r.y0 + j as i64, r.made[j][g], b[0], b[1], b[2], b[3], p(g, j))
            })
            .collect();
        println!("          {}: {}", C.good_name[g], line.join("; "));
    }
    let mut avg: Vec<(usize, f64)> = by_good.iter().copied().enumerate().collect();
    avg.sort_by(|a, b| b.1.total_cmp(&a.1));
    println!("  the cost of living's rise in the worst years, good by good (mean share over islands): {}", avg.iter().take(6).map(|&(g, v)| format!("{} {:.2}", C.good_name[g], v)).collect::<Vec<_>>().join(", "));
    // and over every year: how much of the real wage's yearly swing each good's price carries
    println!("\n  every year from 1760: the real wage's yearly change against the cost of living's, and each good's part in the cost of living's swing (variance shares, means over islands)");
    let mut var_part = vec![0.0; G];
    let (mut sd_rw, mut sd_w, mut sd_c) = (0.0, 0.0, 0.0);
    for r in runs {
        let from = (1760 - r.y0).max(1) as usize;
        let p = |g: usize, k: usize| r.ann[&format!("price_{}", C.good_name[g])][k];
        let d = |s: &[f64]| -> Vec<f64> { (from..r.n).map(|k| (s[k] / s[k - 1]).ln()).collect() };
        let sd = |v: &[f64]| { let m = mean(v); mean(&v.iter().map(|x| (x - m) * (x - m)).collect::<Vec<_>>()).sqrt() };
        sd_rw += sd(&d(&r.ann["real_wage"])) / runs.len() as f64;
        sd_w += sd(&d(&r.ann["wage"])) / runs.len() as f64;
        let dc = d(&r.ann["cpi"]);
        sd_c += sd(&dc) / runs.len() as f64;
        let mc = mean(&dc);
        let var: f64 = dc.iter().map(|x| (x - mc) * (x - mc)).sum();
        for g in 0..G {
            let part: Vec<f64> = (from..r.n).map(|k| r.basket[g] * (p(g, k) - p(g, k - 1)) / (0..G).map(|h| r.basket[h] * p(h, k - 1)).sum::<f64>()).collect();
            let mp = mean(&part);
            let cov: f64 = part.iter().zip(&dc).map(|(a, b)| (a - mp) * (b - mc)).sum();
            var_part[g] += cov / var / runs.len() as f64;
        }
    }
    println!("  yearly swing (sd of the log change): real wage {sd_rw:.3}, money wage {sd_w:.3}, cost of living {sd_c:.3}");
    let mut vp: Vec<(usize, f64)> = var_part.iter().copied().enumerate().collect();
    vp.sort_by(|a, b| b.1.total_cmp(&a.1));
    println!("  the cost of living's swing by good: {}", vp.iter().take(8).map(|&(g, v)| format!("{} {:.2}", C.good_name[g], v)).collect::<Vec<_>>().join(", "));
}

fn main() {
    let mut rules = Rules::Game;
    let mut families = 6usize;
    let mut set: Option<String> = None;
    let mut only: Option<String> = None;
    let mut seeds: Vec<u64> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--rules" => rules = Rules::parse(&args.next().expect("--rules port|game")).expect("--rules port|game"),
            "--families" => families = args.next().and_then(|v| v.parse().ok()).expect("--families N"),
            "--set" => set = Some(args.next().expect("--set JSON")),
            "--only" => only = Some(args.next().expect("--only growth|life|worst")),
            _ => seeds.push(a.parse().unwrap_or_else(|_| panic!("{a} is not a seed"))),
        }
    }
    if seeds.is_empty() {
        seeds = (1..=8).collect();
    }
    let mut P = rules.params();
    if let Some(text) = &set {
        P = P.with_json_overrides(text).unwrap_or_else(|e| panic!("{e}"));
    }
    P.n_families = families;
    P.accounts.enabled = true;
    P.accounts.works = false;
    let C = rules.content();
    let runs: Vec<Run> = std::thread::scope(|sc| {
        let handles: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let (mut P, C) = (P.clone(), C.clone());
                P.seed = seed;
                sc.spawn(move || gather(&P, C))
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });
    println!("{} rules, {} families, seeds {:?}{}", rules.as_str(), families, seeds, set.as_ref().map_or(String::new(), |s| format!(", {s}")));
    let want = |s: &str| only.as_deref().is_none_or(|o| o == s);
    if want("growth") {
        growth(&runs, &C);
    }
    if want("life") {
        life(&runs, &C);
    }
    if want("worst") {
        worst(&runs, &C);
    }
}
