//! Prints what docs/probes/tr-rent.md reads: (a) every spell in which a town's
//! landlords end a year below nothing, with what filled and emptied their purse;
//! (b) how much of each town's house rent families own and whether a town's
//! remainder could go below nothing; (c) how far `harvest::average_rent` is from
//! the rent of an average year; (d) the towns short of food, with their food's
//! books. Written for the trace of landlords, rent and empty shelves.
//!
//!     cargo run --release -p isostate-core --example trace_rent -- [--seeds 1-8] [--families 6]
//!         [--part a,b,c,d] [--set JSON] [--from 1790 --to 1820] [--out DIR] [--port]
//!
//! The seeds run side by side under the game's rules (`--port`: the port's).
//! `--set` overrides parameters as `run --params` does, e.g.
//! `--set '{"mines":{"enabled":false}}'`. `--from`/`--to` bound part (d)'s
//! years. `--out DIR` also writes every town's year for part (a) as
//! `debt-seedN.csv`, and part (c)'s years as `rent-seedN.csv`.
//!
//! Part (c) plays every year three times: as it was, again from the same
//! January with the hands hired for an average year (`harvest.foreseen` 0) and
//! the same weather, and again with no weather at all. The first gives the
//! figure the families price land on, the second what that figure would be had
//! nothing been foreseen, the third the rent an average year would really have
//! paid. It takes three times as long.
#![allow(non_snake_case)]
use isostate_core::accounts::{self, KL, KT, KG, LAND_FIELDS, TOWN_FIELDS, GOODS_FIELDS};
use isostate_core::*;
use std::fmt::Write as _;

fn col(names: &[&str], name: &str) -> usize {
    names.iter().position(|n| *n == name).unwrap_or_else(|| panic!("no column {name}"))
}

#[derive(Clone, Default)]
struct DebtRow {
    town: usize,
    year: i64,
    cash: f64,
    cash_min: f64,
    wage: f64,
    wage_max: f64,
    income: f64,
    month_net: f64,
    spend: f64,
    yearend: f64,
    subs: f64,
    sales: f64,
    rent_sector: Vec<f64>,
    rent_sites: f64,
    rent_town: f64,
    rent_fam: f64,
    pop: f64,
}

#[derive(Clone, Default)]
struct UrbanCheck {
    years: usize,
    worst_sum_off: f64,
    worst_owned: (f64, String),
    town_years_owned: usize,
    town_years: usize,
    mismatched: usize,
    mean_owned: f64,
}

#[derive(Clone, Default)]
struct RentYear {
    year: i64,
    harvest: f64,
    paid: f64,
    est: f64,
    est0: f64,
    avg: f64,
    abs_est: f64,
    abs_est0: f64,
    abs_paid: f64,
    bought_value: f64,
    bought_err_hire: f64,
    bought_err_all: f64,
    bought_cells: usize,
}

struct Out {
    seed: u64,
    text: String,
    debt: Vec<DebtRow>,
    urban: UrbanCheck,
    rent: Vec<RentYear>,
    names: Vec<String>,
    founded: Vec<i64>,
    sectors: Vec<String>,
}

fn run(seed: u64, P: &Params, rules: Rules, parts: &str, from: i64, to: i64) -> Out {
    let mut P = P.clone();
    P.seed = seed;
    P.accounts.enabled = true;
    let mut sim = sim::start(&P, rules.content());
    let C = sim.content.clone();
    let T = sim.st.T_max;
    let (NS, G) = (C.NS, C.G);
    let t_lords = col(&TOWN_FIELDS, "cash_landlords");
    let t_spend = col(&TOWN_FIELDS, "spend_landlords");
    let (t_sites, t_town, t_fam, t_pop) =
        (col(&TOWN_FIELDS, "rent_sites"), col(&TOWN_FIELDS, "rent_town"), col(&TOWN_FIELDS, "rent_families"), col(&TOWN_FIELDS, "pop"));
    let l_rent = col(&LAND_FIELDS, "rent");
    let food: Vec<usize> = (0..G).filter(|&g| C.props[(g, 0)] > 0.0).collect();
    let mut text = String::new();
    let mut debt = Vec::new();
    let mut urban = UrbanCheck::default();
    let mut rent = Vec::new();
    // part (d): every town's month: people, food in store, food's price, food got over need, households' cash a head
    let mut months: Vec<Vec<[f64; 5]>> = Vec::new();
    while sim.year <= sim.end_year {
        let year = sim.year;
        let k = (year - C.start_year) as usize;
        let cash_jan = sim.st.lord_cash.clone();
        let mut cash_min = cash_jan.clone();
        let mut wage_max = vec![0.0f64; T];
        let mut income = vec![0.0; T];
        let mut ratio_months = vec![[9.0f64; 12]; T];
        // part (c): the same year with the hands hired for an average year, and with no weather
        let twins = if parts.contains('c') && families::active(&sim.st) && P.harvest.enabled {
            let mut hire = sim.clone();
            hire.params.harvest.foreseen = 0.0;
            let mut none = sim.clone();
            none.st.harvest_mult.data.iter_mut().for_each(|v| *v = 1.0);
            for month in 0..12 {
                sim::run_month(&mut hire, month).expect("money is conserved");
                sim::run_month(&mut none, month).expect("money is conserved");
            }
            Some((hire, none))
        } else {
            None
        };
        for month in 0..12 {
            let before = sim.st.lord_cash.clone();
            sim::run_month(&mut sim, month).expect("money is conserved");
            let st = &sim.st;
            for t in 0..T {
                // the month's rents, as they were credited (`econ::step_month` keeps them in lord_income_prev)
                let _ = before[t];
                income[t] += st.lord_income_prev[t];
                cash_min[t] = cash_min[t].min(st.lord_cash[t]);
                wage_max[t] = wage_max[t].max(st.wage[t]);
                ratio_months[t][month] = if st.town_active[t] { st.food_ratio[t] } else { 9.0 };
            }
            if parts.contains('d') {
                months.push(
                    (0..T)
                        .map(|t| {
                            let stock: f64 = food.iter().map(|&g| st.inv[(t, g)]).sum();
                            [st.pop[t], stock, st.price[(t, food[0])], st.food_ratio[t], st.hh_cash[t] / st.pop[t].max(1.0)]
                        })
                        .collect(),
                );
            }
        }
        let cash_dec = sim.st.lord_cash.clone();
        let food_min: Vec<f64> = sim.st.food_min_acc.clone();
        // part (c): the figure the families will price land on, cell by cell, before the year closes
        let (mut est_c, mut est0_c, mut avg_c) = (Vec::new(), Vec::new(), Vec::new());
        if let Some((hire, none)) = &twins {
            let st = &sim.st;
            let gap = |s: &State, c: usize| s.harvest_rent_gap.get(c).copied().unwrap_or(0.0);
            est_c = (0..st.N).map(|c| st.cell_rent_paid[c] + gap(st, c)).collect();
            est0_c = (0..st.N).map(|c| hire.st.cell_rent_paid[c] + gap(&hire.st, c)).collect();
            avg_c = (0..st.N).map(|c| none.st.cell_rent_paid[c]).collect();
        }
        let projects_before: Vec<(usize, usize, usize, i64)> = sim.st.projects.iter().map(|p| (p.a, p.b, p.target, p.start)).collect();
        let owners_before = sim.st.cell_owner.clone();
        sim::end_year(&mut sim);
        let st = &sim.st;
        // (a) the landlords' purse, year by year
        if parts.contains('a') {
            let mut subs = vec![0.0; T];
            for p in &st.projects {
                if projects_before.contains(&(p.a, p.b, p.target, p.start)) {
                    continue;
                }
                for &(lords, t, put) in &p.subs {
                    if lords {
                        subs[t] += put;
                    }
                }
            }
            let mut sales = vec![0.0; T];
            for c in 0..st.N {
                if owners_before[c] < 0 && st.cell_owner[c] >= 0 && st.catch[c] >= 0 && P.rent_regime == RentRegime::Landlords {
                    sales[st.catch[c] as usize] += families::land_price(st, &P, c);
                }
            }
            if let Some(b) = st.acct.year(k) {
                for t in 0..T {
                    if !st.town_active[t] && b.towns[t * KT + t_pop] == 0.0 {
                        continue;
                    }
                    let tw = &b.towns[t * KT..(t + 1) * KT];
                    debt.push(DebtRow {
                        town: t,
                        year,
                        cash: st.lord_cash[t],
                        cash_min: cash_min[t].min(st.lord_cash[t]),
                        wage: tw[col(&TOWN_FIELDS, "wage")] as f64,
                        wage_max: wage_max[t],
                        income: income[t],
                        month_net: cash_dec[t] - cash_jan[t] - income[t],
                        spend: tw[t_spend] as f64,
                        yearend: st.lord_cash[t] - cash_dec[t],
                        subs: subs[t],
                        sales: sales[t],
                        rent_sector: (0..NS).map(|s| b.land[(t * NS + s) * KL + l_rent] as f64).collect(),
                        rent_sites: tw[t_sites] as f64,
                        rent_town: tw[t_town] as f64,
                        rent_fam: tw[t_fam] as f64,
                        pop: tw[t_pop] as f64,
                    });
                    let _ = tw[t_lords];
                }
            }
        }
        // (b) families' part of each town's house rent, as the coming year will share it
        if parts.contains('b') && families::active(st) {
            urban.years += 1;
            let mut share = vec![0.0; T];
            let mut owned = vec![0.0; T];
            for c in 0..st.N {
                let t = st.catch[c];
                if !st.urban[c] || t < 0 || st.urban_rent[t as usize] <= 0.0 {
                    continue;
                }
                let t = t as usize;
                let s = st.cell_urban_rent[c] / st.urban_rent[t];
                share[t] += s;
                if st.cell_owner[c] >= 0 {
                    owned[t] += s;
                }
            }
            for t in 0..T {
                if !st.town_active[t] || st.urban_rent[t] <= 0.0 {
                    continue;
                }
                // the town's own ground against the cells the month's sharing will count for it
                let listed: f64 = st.town_ground.get(t).map(|l| l.iter().map(|&c| st.cell_urban_rent[c]).sum::<f64>()).unwrap_or(0.0);
                if P.towns.enabled && (listed / st.urban_rent[t] - 1.0).abs() > 1e-9 {
                    urban.mismatched += 1;
                }
                if st.town_ground.get(t).is_some_and(|l| l.iter().any(|&c| st.catch[c] != t as i64 || !st.urban[c])) {
                    urban.mismatched += 1;
                }
                urban.worst_sum_off = urban.worst_sum_off.max((share[t] - 1.0).abs());
                urban.town_years += 1;
                urban.mean_owned += owned[t];
                if owned[t] > 0.0 {
                    urban.town_years_owned += 1;
                }
                if owned[t] > urban.worst_owned.0 {
                    urban.worst_owned = (owned[t], format!("{} in {}", st.town_name[t], year + 1));
                }
            }
        }
        // (c) the three figures, added up over the island's land, and the land the families bought on them
        if let Some(_) = twins {
            let mut r = RentYear { year, harvest: st.harvest_year, ..RentYear::default() };
            for c in 0..st.N {
                if st.urban[c] || st.catch[c] < 0 || !st.world.land[c] {
                    continue;
                }
                let paid = st.cell_rent_year[c];
                r.paid += paid;
                r.est += est_c[c];
                r.est0 += est0_c[c];
                r.avg += avg_c[c];
                r.abs_est += (est_c[c] - avg_c[c]).abs();
                r.abs_est0 += (est0_c[c] - avg_c[c]).abs();
                r.abs_paid += (paid - avg_c[c]).abs();
                if owners_before[c] != st.cell_owner[c] && st.cell_owner[c] >= 0 {
                    // bought at the year's end, at 25 years of est (or the floor)
                    let price = families::land_price(st, &P, c);
                    r.bought_value += price;
                    if price > P.years_purchase * est_c[c].min(est0_c[c]).min(avg_c[c]) + 1e-9 || est_c[c] * P.years_purchase >= price - 1e-6 {
                        r.bought_err_hire += P.years_purchase * (est_c[c] - est0_c[c]);
                        r.bought_err_all += P.years_purchase * (est_c[c] - avg_c[c]);
                    }
                    r.bought_cells += 1;
                }
            }
            rent.push(r);
        }
        // (d) towns short of food, with the books of their food
        if parts.contains('d') && year >= from && year <= to {
            let b = st.acct.year(k);
            for t in 0..T {
                if !st.town_active[t] || food_min[t] >= 0.82 {
                    continue;
                }
                let _ = writeln!(
                    text,
                    "  {year} {} (founded {}, {:.0} people): food at the worst month {:.2}; by month {}",
                    st.town_name[t],
                    st.town_founded[t],
                    st.pop[t],
                    food_min[t],
                    ratio_months[t].iter().map(|v| format!("{v:.2}")).collect::<Vec<_>>().join(" ")
                );
                // the months round the worst: the year before's last three and this year's first five
                let worst = (0..12).min_by(|&a, &c| ratio_months[t][a].total_cmp(&ratio_months[t][c])).unwrap_or(0);
                let at = months.len() - 12 + worst;
                for m in at.saturating_sub(3)..(at + 3).min(months.len()) {
                    let r = months[m][t];
                    let _ = writeln!(
                        text,
                        "      {}-{:02}: people {:7.0}, food in store {:8.0} ({:.2} months of need), price {:.3}, food got {:.2} of need, households' cash a head {:.2}",
                        C.start_year + (m / 12) as i64,
                        m % 12 + 1,
                        r[0],
                        r[1],
                        r[1] / (r[0] * C.subsistence[0]).max(1e-9),
                        r[2],
                        r[3],
                        r[4]
                    );
                }
                if k > 0 {
                    if let Some(p) = st.acct.year(k - 1) {
                        for &g in &food {
                            let row = &p.goods[(t * G + g) * KG..(t * G + g + 1) * KG];
                            let v = |n: &str| row[col(&GOODS_FIELDS, n)] as f64;
                            let _ = writeln!(text, "      the year before, {}: made {:.0}, came or went with districts at its end {:+.0}, in store at its end {:.0}", C.good_name[g], v("made"), v("moved"), v("stock"));
                        }
                    }
                }
                let Some(b) = &b else { continue };
                for &g in &food {
                    let row = &b.goods[(t * G + g) * KG..(t * G + g + 1) * KG];
                    let v = |n: &str| row[col(&GOODS_FIELDS, n)] as f64;
                    let bought = ["households", "landlords", "families", "works_inputs", "works_fuel", "land", "building", "projects"].iter().map(|n| v(n)).sum::<f64>();
                    if v("made") + v("received") + v("asked") == 0.0 {
                        continue;
                    }
                    let _ = writeln!(
                        text,
                        "      {:10} made {:9.0} received {:8.0} sent {:8.0} bought {:9.0} asked {:9.0} unmet {:8.0} stock {:7.0} -> {:7.0} price {:.2} ({:.2} to {:.2})",
                        C.good_name[g],
                        v("made"),
                        v("received"),
                        v("sent"),
                        bought,
                        v("asked"),
                        v("asked") - bought,
                        v("stock_open"),
                        v("stock"),
                        v("price"),
                        v("price_low"),
                        v("price_high")
                    );
                }
            }
        }
    }
    Out {
        seed,
        text,
        debt,
        urban,
        rent,
        names: sim.st.town_name.clone(),
        founded: sim.st.town_founded.clone(),
        sectors: C.sector_name.clone(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut families, mut parts, mut set, mut from, mut to, mut out, mut rules) =
        (vec![1u64, 2, 3, 4, 5, 6, 7, 8], 6usize, String::from("a,b,d"), None::<String>, 1750i64, 1900i64, None::<String>, Rules::Game);
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
            "--families" => families = next().parse().unwrap(),
            "--part" => parts = next(),
            "--set" => set = Some(next()),
            "--from" => from = next().parse().unwrap(),
            "--to" => to = next().parse().unwrap(),
            "--out" => out = Some(next()),
            "--port" => {
                rules = Rules::Port;
                i += 1;
                continue;
            }
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let mut P = rules.params();
    if let Some(s) = &set {
        P = P.with_json_overrides(s).expect("--set takes a JSON object of parameters");
    }
    P.n_families = families;
    let outs: Vec<Out> = std::thread::scope(|s| {
        let hs: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let P = P.clone();
                let parts = parts.clone();
                s.spawn(move || run(seed, &P, rules, &parts, from, to))
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });
    println!("{} rules, {} families, seeds {:?}{}", rules.as_str(), families, seeds, set.as_deref().map(|s| format!(", {s}")).unwrap_or_default());

    if parts.contains('a') {
        println!("\n(a) landlords below nothing at a year's end: each spell, town by town");
        println!("    rent = the landlords' rents of the year (land by sector, sites, houses, less the families' part)");
        let mut spells: Vec<(u64, &str, usize, f64, f64)> = Vec::new();
        let mut debt_years = 0;
        for o in &outs {
            if let Some(dir) = &out {
                let mut csv = String::from("town,name,founded,year,pop,wage,wage_max,cash,cash_min,income,month_net,spend,yearend,subs,sales,rent_sites,rent_town,rent_fam");
                for s in &o.sectors {
                    let _ = write!(csv, ",rent_{s}");
                }
                csv.push('\n');
                for r in &o.debt {
                    let _ = write!(
                        csv,
                        "{},{},{},{},{:.0},{:.3},{:.3},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1}",
                        r.town, o.names[r.town], o.founded[r.town], r.year, r.pop, r.wage, r.wage_max, r.cash, r.cash_min, r.income, r.month_net, r.spend, r.yearend, r.subs, r.sales, r.rent_sites, r.rent_town, r.rent_fam
                    );
                    for v in &r.rent_sector {
                        let _ = write!(csv, ",{v:.1}");
                    }
                    csv.push('\n');
                }
                std::fs::write(format!("{dir}/debt-seed{}.csv", o.seed), csv).expect("the --out folder exists");
            }
            // the island's middle wage each year, to measure a town's against
            let mut by_year: std::collections::BTreeMap<i64, Vec<f64>> = std::collections::BTreeMap::new();
            for r in o.debt.iter().filter(|r| r.pop > 0.0 && r.wage > 0.0) {
                by_year.entry(r.year).or_default().push(r.wage);
            }
            let median = |y: i64| -> f64 {
                let mut v = by_year.get(&y).cloned().unwrap_or_default();
                v.sort_by(f64::total_cmp);
                if v.is_empty() { f64::NAN } else { v[v.len() / 2] }
            };
            let towns: std::collections::BTreeSet<usize> = o.debt.iter().filter(|r| r.cash < 0.0).map(|r| r.town).collect();
            let min_months = o.debt.iter().filter(|r| r.cash_min < 0.0).count();
            println!("  seed {}: {} towns' landlords end a year below nothing; {} town-years dip below within the year", o.seed, towns.len(), min_months);
            for &t in &towns {
                let rows: Vec<&DebtRow> = o.debt.iter().filter(|r| r.town == t).collect();
                // spells: runs of year ends below nothing
                let mut k = 0;
                while k < rows.len() {
                    if rows[k].cash >= 0.0 {
                        k += 1;
                        continue;
                    }
                    let start = k;
                    while k < rows.len() && rows[k].cash < 0.0 {
                        k += 1;
                    }
                    let spell = &rows[start..k];
                    let first = spell[0];
                    let deepest = spell.iter().map(|r| r.cash).fold(0.0, f64::min);
                    // what they live on while they pay it off: the rents of the spell's later years
                    let later: Vec<&&DebtRow> = spell.iter().skip(1).collect();
                    let n = later.len().max(1) as f64;
                    let rent_later = later.iter().map(|r| r.income).sum::<f64>() / n;
                    let spend_later = later.iter().map(|r| r.spend).sum::<f64>() / n;
                    // and before: the three years before it went under
                    let pre: Vec<&&DebtRow> = rows[start.saturating_sub(3)..start].iter().collect();
                    let rent_pre = if pre.is_empty() { f64::NAN } else { pre.iter().map(|r| r.income).sum::<f64>() / pre.len() as f64 };
                    let spend_pre = if pre.is_empty() { f64::NAN } else { pre.iter().map(|r| r.spend).sum::<f64>() / pre.len() as f64 };
                    let how = if start == 0 && first.income == 0.0 {
                        "came with the town's founding"
                    } else if first.income < 0.0 {
                        "rents below nothing"
                    } else {
                        "rents above nothing"
                    };
                    spells.push((o.seed, how, spell.len(), -deepest, first.wage_max / median(first.year)));
                    debt_years += spell.len();
                    let over = if k < rows.len() { format!("paid off in {}", rows[k].year) } else { "still owed in the last year".to_string() };
                    println!(
                        "    {} (founded {}): {} to {} ({} years), deepest {:.0}; {}; {}",
                        o.names[t],
                        o.founded[t],
                        first.year,
                        spell[spell.len() - 1].year,
                        spell.len(),
                        deepest,
                        how,
                        over
                    );
                    let sec: Vec<String> = first.rent_sector.iter().zip(&o.sectors).filter(|(v, _)| v.abs() > 0.5).map(|(v, s)| format!("{s} {v:.0}")).collect();
                    println!(
                        "      {}: rents {:.0} [{}; sites {:.0}; houses {:.0}; to families {:.0}]; wage {:.2} in the mean, {:.2} at the most, the island's middle town {:.2}; spent {:.0}; projects and other in the months {:.0}; at the year's end {:+.0} (subscribed {:.0}, land sold {:.0})",
                        first.year,
                        first.income,
                        sec.join(", "),
                        first.rent_sites,
                        first.rent_town,
                        first.rent_fam,
                        first.wage,
                        first.wage_max,
                        median(first.year),
                        first.spend,
                        first.month_net + first.spend,
                        first.yearend,
                        first.subs,
                        first.sales
                    );
                    println!(
                        "      before: rents {:.0} a year, spent {:.0}; in the later years of the spell: rents {:.0} a year, spent {:.0}; the debt at its deepest is {:.1} years of the rents before",
                        rent_pre, spend_pre, rent_later, spend_later, -deepest / rent_pre
                    );
                }
            }
        }
        println!("\n  spells by how they began: count, town-years, the longest, the deepest, and the most a town's wage stood over the island's middle in the year it began");
        for how in ["rents below nothing", "came with the town's founding", "rents above nothing"] {
            let v: Vec<&(u64, &str, usize, f64, f64)> = spells.iter().filter(|s| s.1 == how).collect();
            if v.is_empty() {
                continue;
            }
            let mut w: Vec<f64> = v.iter().map(|s| s.4).filter(|x| x.is_finite()).collect();
            w.sort_by(f64::total_cmp);
            println!(
                "    {:32} {:4} spells {:5} town-years, longest {:3}, deepest {:10.0}, wage over the middle at most {:.0} times (the median spell {:.1})",
                how,
                v.len(),
                v.iter().map(|s| s.2).sum::<usize>(),
                v.iter().map(|s| s.2).max().unwrap_or(0),
                v.iter().map(|s| s.3).fold(0.0, f64::max),
                w.last().copied().unwrap_or(f64::NAN),
                if w.is_empty() { f64::NAN } else { w[w.len() / 2] }
            );
        }
        let spells_all = spells.len();
        println!("  in all: {spells_all} spells, {debt_years} town-years below nothing");
    }

    if parts.contains('b') {
        println!("\n(b) families' part of a town's house rent, at every year's end");
        for o in &outs {
            let u = &o.urban;
            println!(
                "  seed {}: {} town-years; shares off one by at most {:.1e}; ground and sharing disagree in {} town-years; families own some of {} town-years' house rent, {:.3} of it in the mean, at most {:.3} ({})",
                o.seed,
                u.town_years,
                u.worst_sum_off,
                u.mismatched,
                u.town_years_owned,
                u.mean_owned / u.town_years.max(1) as f64,
                u.worst_owned.0,
                u.worst_owned.1
            );
        }
    }

    if parts.contains('c') {
        println!("\n(c) average_rent against the rent of an average year, the island's land outside the towns");
        println!("    est: rent paid plus the crop the weather took (what families price land on); est0: the same with the hands hired for an average year;");
        println!("    avg: the year played again with no weather. Errors are sums over cells of |x - avg| over the sum of avg.");
        println!("  seed  harvest       years   paid-avg  est-avg  est0-avg  |paid|  |est|  |est0|   land bought   its error, hiring   all");
        for o in &outs {
            if let Some(dir) = &out {
                let mut csv = String::from("year,harvest,paid,est,est0,avg,abs_paid,abs_est,abs_est0,bought_value,bought_err_hire,bought_err_all,bought_cells\n");
                for r in &o.rent {
                    let _ = writeln!(
                        csv,
                        "{},{:.4},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{}",
                        r.year, r.harvest, r.paid, r.est, r.est0, r.avg, r.abs_paid, r.abs_est, r.abs_est0, r.bought_value, r.bought_err_hire, r.bought_err_all, r.bought_cells
                    );
                }
                std::fs::write(format!("{dir}/rent-seed{}.csv", o.seed), csv).expect("the --out folder exists");
            }
            for (label, lo, hi) in [("bad (<0.97)", 0.0, 0.97), ("ordinary", 0.97, 1.03), ("good (>1.03)", 1.03, 9.0)] {
                let ys: Vec<&RentYear> = o.rent.iter().filter(|r| r.harvest >= lo && r.harvest < hi).collect();
                if ys.is_empty() {
                    continue;
                }
                let s = |f: fn(&RentYear) -> f64| ys.iter().map(|r| f(r)).sum::<f64>();
                let avg = s(|r| r.avg).max(1.0);
                println!(
                    "  {:4}  {:12} {:5}   {:+.4}   {:+.4}   {:+.4}   {:.4}  {:.4}  {:.4}   {:12.0}   {:+10.0}  {:+10.0}",
                    o.seed,
                    label,
                    ys.len(),
                    (s(|r| r.paid) - avg) / avg,
                    (s(|r| r.est) - avg) / avg,
                    (s(|r| r.est0) - avg) / avg,
                    s(|r| r.abs_paid) / avg,
                    s(|r| r.abs_est) / avg,
                    s(|r| r.abs_est0) / avg,
                    s(|r| r.bought_value),
                    s(|r| r.bought_err_hire),
                    s(|r| r.bought_err_all)
                );
            }
        }
    }

    if parts.contains('d') {
        println!("\n(d) towns short of food (under 0.82 of their need at the worst month), {from} to {to}");
        for o in &outs {
            println!(" seed {}", o.seed);
            print!("{}", o.text);
        }
    }
    let _ = accounts::KM;
}
