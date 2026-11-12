//! Prints, for docs/probes/tr-urban.md, why the towns stop at four people in ten:
//! (a) the share living in towns and the island's labour force split by where the
//! hands work, from the books; (b) every year's candidate sites for a new town and
//! which of the founding rule's tests each fails; (c) each town's fastest decade,
//! split into births less deaths, migration and cells that changed market, with how
//! often the arrivals limit, the income ceiling and the town's room were reached.
//!
//!     cargo run --release -p isostate-core --example tr_urban -- [--seeds 1-8] [--families 6]
//!         [--set KEY=VALUE]... [--part a|b|c|all] [--every 25]
//!
//! Always the game's rules, with the books on. `--set` takes the sweep's dotted
//! names (`people.found_saving=0.04`, `migration_rate=0.1`), one value each. The
//! seeds run side by side. Pooled figures are means over the seeds; `[lo .. hi]`
//! is the least and greatest seed.
#![allow(non_snake_case)]
use isostate_core::accounts::{KL, KM, KT, TOWN_FIELDS};
use isostate_core::*;

/// One year of the island, from the books and the state.
#[derive(Clone, Default)]
struct IslandRow {
    year: i64,
    pop: f64,
    urban: f64,
    /// The towns' rule summed town by town before smoothing: pop * clamp(0.04 + 0.8 * works / (p * pop)).
    urban_raw: f64,
    labour: f64,
    works: f64,
    services: f64,
    land: Vec<f64>,
    carters: f64,
    navvies: f64,
    supply: f64,
    wage_land: f64,
    wage_works: f64,
    hands_land_paid: f64,
    hands_works_paid: f64,
    wage: f64,
    towns: usize,
    /// Town people over the room of the towns' ground.
    room: f64,
}

/// A site the founding rule looked at.
#[derive(Clone)]
struct Cand {
    ratio: f64,
    cells: usize,
    people: f64,
    donor: f64,
    rest: bool,
}

#[derive(Clone)]
struct CandYear {
    year: i64,
    cands: Vec<Cand>,
    /// The island's mean money wage of the year (by people).
    wage: f64,
    founded: bool,
    /// Cells of land at least `found_spacing` from every town.
    spaced: usize,
}

/// One town's year.
#[derive(Clone, Default)]
struct TownYear {
    year: i64,
    town: usize,
    founded: i64,
    before: f64,
    after: f64,
    rate: f64,
    inc: f64,
    crowd: f64,
    mig_in: f64,
    mig_out: f64,
    capped: bool,
    urban: f64,
    room: f64,
    y: f64,
    y_island: f64,
    wage_rel: f64,
}

struct Out {
    island: Vec<IslandRow>,
    cands: Vec<CandYear>,
    towns: Vec<TownYear>,
    founded: Vec<(i64, String, String, f64)>,
    names: Vec<String>,
}

fn run(P: &Params) -> Out {
    let C = Rules::Game.content();
    let mut sim = sim::start(P, C);
    sim.check_money = true;
    let T = sim.st.T_max;
    let pp = sim.params.people.clone();
    let (mut cands, mut towns, mut urban_rows) = (Vec::new(), Vec::new(), Vec::new());
    while sim.year <= sim.end_year {
        let year = sim.year;
        for month in 0..12 {
            sim::run_month(&mut sim, month).expect("money is conserved every month");
        }
        // (b) what the founding rule is about to see: this year's carting on last
        // year's paths, at the year's mean wage. Worked on a copy; the run is untouched.
        {
            let mut probe = sim.st.clone();
            probe.cell_flow_year = probe.cell_flow.clone();
            probe.wbar = probe.wage_acc.iter().map(|v| v / 12.0).collect();
            let gave = |i: usize| probe.pp_gave.get(i).copied().unwrap_or(-10_000);
            let list = peopling::candidates(&probe, &sim.content, &sim.params)
                .into_iter()
                .map(|c| Cand {
                    ratio: c.saved / c.wages.max(1e-9),
                    cells: c.cells,
                    people: c.people,
                    donor: c.donor_share,
                    rest: c.donors.iter().all(|&i| year - gave(i) >= pp.donor_rest),
                })
                .collect();
            let act: Vec<usize> = (0..T).filter(|&t| probe.town_active[t]).collect();
            let pop: f64 = act.iter().map(|&t| probe.pop[t]).sum();
            let wage = act.iter().map(|&t| probe.wbar[t] * probe.pop[t]).sum::<f64>() / pop.max(1.0);
            let W = probe.world.W;
            let spaced = (0..probe.N)
                .filter(|&c| {
                    probe.world.land[c]
                        && probe.world.terrain[c] < 3
                        && act.iter().all(|&i| {
                            let n = probe.town_node[i];
                            ((c / W) as f64 - (n / W) as f64).hypot((c % W) as f64 - (n % W) as f64) > pp.found_spacing
                        })
                })
                .count();
            cands.push(CandYear { year, cands: list, wage, founded: false, spaced });
        }
        // (c) each town's year: what it had, and what the step does to it
        let act: Vec<usize> = (0..T).filter(|&t| sim.st.town_active[t]).collect();
        let before = sim.st.pop.clone();
        let n_found = sim.st.pp_foundings.len();
        sim::end_year(&mut sim);
        let st = &sim.st;
        if st.pp_foundings.len() > n_found {
            cands.last_mut().unwrap().founded = true;
        }
        let n = act.len();
        let flow = &st.migration_flow;
        let pop_now: f64 = (0..T).filter(|&t| st.town_active[t]).map(|t| st.pop[t]).sum();
        let y_island = act.iter().map(|&t| st.y_ratio[t] * before[t]).sum::<f64>() / act.iter().map(|&t| before[t]).sum::<f64>().max(1.0);
        let wage_island = act.iter().map(|&t| st.wbar[t] * before[t]).sum::<f64>() / act.iter().map(|&t| before[t]).sum::<f64>().max(1.0);
        for (k, &t) in act.iter().enumerate() {
            let (mut mig_in, mut mig_out) = (0.0, 0.0);
            if flow.rows == n {
                for j in 0..n {
                    mig_in += flow[(j, k)];
                    mig_out += flow[(k, j)];
                }
            }
            let grown = before[t] * (1.0 + st.pp_growth.get(t).copied().unwrap_or(0.0));
            let ground = &st.town_ground[t];
            let room: f64 = ground.iter().map(|&c| st.town_room[c]).sum();
            towns.push(TownYear {
                year,
                town: t,
                founded: st.town_founded[t],
                before: before[t],
                after: st.pop[t],
                rate: st.pp_growth.get(t).copied().unwrap_or(0.0),
                inc: st.pp_income_term.get(t).copied().unwrap_or(0.0),
                crowd: peopling::crowding_term(&pp, st.urban_pop[t], st.pop[t]),
                mig_in,
                mig_out,
                capped: mig_in > 0.0 && mig_in >= 0.999 * pp.arrivals_max * grown,
                urban: st.urban_pop[t],
                room,
                y: st.y_ratio[t],
                y_island,
                wage_rel: st.wbar[t] / wage_island.max(1e-9),
            });
        }
        let urban: f64 = (0..T).filter(|&t| st.town_active[t]).map(|t| st.urban_pop[t]).sum();
        let raw: f64 = (0..T)
            .filter(|&t| st.town_active[t])
            .map(|t| st.pop[t] * (0.04 + 0.8 * st.ind_labor[t] / (sim.params.participation * st.pop[t]).max(1.0)).clamp(0.06, 0.95))
            .sum();
        let room: f64 = (0..T).filter(|&t| st.town_active[t]).map(|t| st.town_ground[t].iter().map(|&c| st.town_room[c]).sum::<f64>()).sum();
        urban_rows.push((year, urban, raw, pop_now, room));
    }
    let r = sim::finish(sim);
    let (st, C) = (&r.state, &r.content);
    // (a) the books, year by year
    let f = |name: &str| TOWN_FIELDS.iter().position(|n| *n == name).unwrap();
    let (fpop, fwage, fsup, fcart, fnav, fhw, fpw, fpl) =
        (f("pop"), f("wage"), f("supply"), f("carters"), f("navvies"), f("hands_works"), f("pay_works"), f("pay_land"));
    let services = C.method_name.iter().position(|m| m == "services").unwrap();
    let mut island = Vec::new();
    for (k, &(year, urban, raw, pop_now, room)) in urban_rows.iter().enumerate() {
        let Some(rec) = st.acct.year(k) else { break };
        let nT = rec.towns.len() / KT;
        let NS = rec.land.len() / (nT * KL);
        let M = rec.methods.len() / (nT * KM);
        let mut row = IslandRow { year, urban, urban_raw: raw, land: vec![0.0; NS], room, ..Default::default() };
        let mut wsum = 0.0;
        for t in 0..nT {
            let v = |i: usize| rec.towns[t * KT + i] as f64;
            if v(fpop) <= 0.0 {
                continue;
            }
            row.towns += 1;
            row.pop += v(fpop);
            wsum += v(fwage) * v(fpop);
            row.supply += v(fsup);
            row.carters += v(fcart);
            row.navvies += v(fnav);
            row.works += v(fhw);
            row.hands_works_paid += v(fpw);
            row.hands_land_paid += v(fpl);
            for s in 0..NS {
                let o = (t * NS + s) * KL;
                row.land[s] += rec.land[o + 2] as f64;
                row.wage_land += rec.land[o + 3] as f64;
            }
            row.services += rec.methods[(t * M + services) * KM + 4] as f64;
            for m in 0..M {
                row.wage_works += rec.methods[(t * M + m) * KM + 5] as f64;
            }
        }
        // the state's count after the step and the books' agree on people
        debug_assert!((row.pop - pop_now).abs() < 1e-3 * pop_now.max(1.0));
        row.wage = wsum / row.pop.max(1.0);
        row.labour = r.params.participation * row.pop;
        island.push(row);
    }
    let founded = st
        .pp_foundings
        .iter()
        .map(|fo| (fo.year, st.town_name[fo.town].clone(), fo.kind.clone(), fo.saving))
        .collect();
    Out { island, cands, towns, founded, names: st.town_name.clone() }
}

fn stats(v: &[f64]) -> (f64, f64, f64) {
    let n = v.len().max(1) as f64;
    (v.iter().sum::<f64>() / n, v.iter().copied().fold(f64::INFINITY, f64::min), v.iter().copied().fold(f64::NEG_INFINITY, f64::max))
}

fn cell(v: &[f64], d: usize) -> String {
    let (m, lo, hi) = stats(v);
    format!("{m:.d$} [{lo:.d$} .. {hi:.d$}]")
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut families, mut part, mut every) = (vec![1u64, 2, 3, 4, 5, 6, 7, 8], 6usize, String::from("all"), 25i64);
    let mut P = Rules::Game.params();
    let mut sets = Vec::new();
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
            "--part" => part = next(),
            "--every" => every = next().parse().unwrap(),
            "--set" => {
                let s = next();
                let (k, v) = s.split_once('=').expect("--set KEY=VALUE");
                let val: serde_json::Value = serde_json::from_str(v).unwrap_or(serde_json::Value::String(v.to_string()));
                P = sweep::set_field(&P, k, &val).unwrap_or_else(|e| panic!("{e}"));
                sets.push(s);
            }
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    P.n_families = families;
    P.accounts.enabled = true;
    let outs: Vec<Out> = std::thread::scope(|s| {
        let hs: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let mut P = P.clone();
                P.seed = seed;
                s.spawn(move || run(&P))
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });
    println!("game rules, {families} families, seeds {seeds:?}, set: {}", if sets.is_empty() { "nothing".to_string() } else { sets.join(" ") });
    let y0 = outs[0].island[0].year;
    let y1 = outs[0].island.last().unwrap().year;
    let marks: Vec<i64> = (y0..=y1).filter(|y| (y - y0) % every == 0 || *y == y1).collect();
    let at = |o: &Out, y: i64| o.island.iter().find(|r| r.year == y).cloned().unwrap();

    if part == "a" || part == "all" {
        println!("\n(a) where the island's hands work, as shares of the labour force (participation x people), after the year's step");
        print!("{:44}", "");
        for y in &marks {
            print!("{y:>24}");
        }
        println!();
        let lines: Vec<(&str, Box<dyn Fn(&IslandRow) -> f64>, usize)> = vec![
            ("people, thousands", Box::new(|r: &IslandRow| r.pop / 1000.0), 0),
            ("towns", Box::new(|r: &IslandRow| r.towns as f64), 1),
            ("living in towns (the towns' count)", Box::new(|r: &IslandRow| r.urban / r.pop), 3),
            ("the rule unsmoothed, town by town", Box::new(|r: &IslandRow| r.urban_raw / r.pop), 3),
            ("0.04 + 0.8 x works' share, island", Box::new(|r: &IslandRow| 0.04 + 0.8 * r.works / r.labour), 3),
            ("hands in works", Box::new(|r: &IslandRow| r.works / r.labour), 3),
            ("  of which services", Box::new(|r: &IslandRow| r.services / r.labour), 3),
            ("hands on farms, pastures, woods", Box::new(|r: &IslandRow| r.land[..3].iter().sum::<f64>() / r.labour), 3),
            ("hands in quarries and clay pits", Box::new(|r: &IslandRow| (r.land[3] + r.land[4]) / r.labour), 3),
            ("hands in iron mines and collieries", Box::new(|r: &IslandRow| (r.land[5] + r.land[6]) / r.labour), 3),
            ("carters", Box::new(|r: &IslandRow| r.carters / r.labour), 3),
            ("navvies", Box::new(|r: &IslandRow| r.navvies / r.labour), 3),
            ("not at work (labour - all above)", Box::new(|r: &IslandRow| 1.0 - (r.works + r.land.iter().sum::<f64>() + r.carters + r.navvies) / r.labour), 3),
            ("pay a hand, works over land", Box::new(|r: &IslandRow| (r.wage_works / r.works.max(1e-9)) / (r.wage_land / r.land.iter().sum::<f64>().max(1e-9))), 3),
            ("town people over the towns' room", Box::new(|r: &IslandRow| r.urban / r.room.max(1.0)), 3),
            ("if quarrymen were in works", Box::new(|r: &IslandRow| 0.04 + 0.8 * (r.works + r.land[3] + r.land[4]) / r.labour), 3),
            ("if carters were town people too", Box::new(|r: &IslandRow| 0.04 + 0.8 * (r.works + r.carters) / r.labour), 3),
        ];
        for (label, fun, d) in &lines {
            print!("{label:44}");
            for &y in &marks {
                let v: Vec<f64> = outs.iter().map(|o| fun(&at(o, y))).collect();
                print!("{:>24}", cell(&v, *d));
            }
            println!();
        }
        // twenty-year means of the share, where a single year misleads
        println!("\n  twenty-year means of the share living in towns, and of the works' hands over the labour force");
        for (a, b) in [(1750, 1769), (1790, 1809), (1840, 1859), (1860, 1879), (1880, 1900)] {
            let u: Vec<f64> = outs.iter().map(|o| stats(&o.island.iter().filter(|r| r.year >= a && r.year <= b).map(|r| r.urban / r.pop).collect::<Vec<_>>()).0).collect();
            let w: Vec<f64> = outs.iter().map(|o| stats(&o.island.iter().filter(|r| r.year >= a && r.year <= b).map(|r| r.works / r.labour).collect::<Vec<_>>()).0).collect();
            println!("  {a}-{b}: {:>24} {:>24}", cell(&u, 3), cell(&w, 3));
        }
        // how far along the Engel curve the households are: their real income, which
        // the curve reads against `engel.y_ref`
        print!("\n  real income a head (y, the year's mean, by people) at the marks:");
        for &y in &marks {
            let v: Vec<f64> = outs
                .iter()
                .map(|o| {
                    let ys: Vec<&TownYear> = o.towns.iter().filter(|r| r.year == y).collect();
                    ys.iter().map(|r| r.y * r.before).sum::<f64>() / ys.iter().map(|r| r.before).sum::<f64>().max(1.0)
                })
                .collect();
            print!("  {y}: {}", cell(&v, 2));
        }
        println!("  (engel.y_ref {})", P.engel.y_ref);
    }

    if part == "b" || part == "all" {
        println!("\n(b) the founding rule's candidate sites, year by year (at most six a year), by decade");
        let pp = &P.people;
        println!(
            "decade  founded  sites/yr  spaced cells  money wage/1750 | best saving: as is   at 1750 wages | of the sites: saving ok  cells ok  people ok  donor ok  rest ok | years a site passed all"
        );
        let w0: Vec<f64> = outs.iter().map(|o| o.cands[0].wage).collect();
        let mut d = (y0 / 10) * 10;
        while d <= y1 {
            let mut n_years = 0.0;
            let (mut founded, mut sites, mut spaced, mut windex, mut best, mut best_real, mut pass) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, Vec::new(), Vec::new(), 0.0f64);
            let mut ok = [0.0f64; 5];
            let mut n_sites = 0.0f64;
            for (o, &w1750) in outs.iter().zip(&w0) {
                for cy in o.cands.iter().filter(|c| c.year >= d && c.year < d + 10) {
                    n_years += 1.0;
                    founded += cy.founded as u8 as f64;
                    sites += cy.cands.len() as f64;
                    spaced += cy.spaced as f64;
                    windex += cy.wage / w1750;
                    if let Some(b) = cy.cands.iter().map(|c| c.ratio).reduce(f64::max) {
                        best.push(b);
                        best_real.push(b * cy.wage / w1750);
                    }
                    let mut any = false;
                    for c in &cy.cands {
                        n_sites += 1.0;
                        let t = [c.ratio >= pp.found_saving, c.cells >= pp.found_cells, c.people >= pp.found_people, c.donor <= pp.donor_max, c.rest];
                        for (k, &b) in t.iter().enumerate() {
                            ok[k] += b as u8 as f64;
                        }
                        any |= t.iter().all(|&b| b);
                    }
                    pass += any as u8 as f64;
                }
            }
            if n_years > 0.0 {
                println!(
                    "{d}s  {:7.2}  {:8.1}  {:12.0}  {:15.2} | {:>18} {:>15} | {:>19.2} {:>9.2} {:>10.2} {:>9.2} {:>8.2} | {:.2}",
                    founded / outs.len() as f64,
                    sites / n_years,
                    spaced / n_years,
                    windex / n_years,
                    format!("{:.3}", median(best.clone())),
                    format!("{:.3}", median(best_real.clone())),
                    ok[0] / n_sites.max(1.0),
                    ok[1] / n_sites.max(1.0),
                    ok[2] / n_sites.max(1.0),
                    ok[3] / n_sites.max(1.0),
                    ok[4] / n_sites.max(1.0),
                    pass / n_years
                );
            }
            d += 10;
        }
        println!("  (best saving: the median over the decade's island-years of the best site's carting saved over its district's wages; the rule asks {:.3})", pp.found_saving);
        // the sites that pass every test but the saving: what they offer as is and at
        // 1750's money wage, and in how many island-years one of them would pass the
        // saving test at 1750's money wage (so would be founded, but for the real area)
        println!("\n  sites that fail on saving alone, and the saving they offer, by decade (median): as is, at 1750's money wage; island-years with one that passes at 1750's money wage");
        let mut d = (y0 / 10) * 10;
        while d <= y1 {
            let (mut v, mut v_real, mut years, mut real_pass) = (Vec::new(), Vec::new(), 0.0f64, 0.0f64);
            for (o, &w1750) in outs.iter().zip(&w0) {
                for cy in o.cands.iter().filter(|c| c.year >= d && c.year < d + 10) {
                    years += 1.0;
                    let mut any = false;
                    for c in &cy.cands {
                        if c.ratio < pp.found_saving && c.cells >= pp.found_cells && c.people >= pp.found_people && c.donor <= pp.donor_max && c.rest {
                            v.push(c.ratio);
                            v_real.push(c.ratio * cy.wage / w1750);
                            any |= c.ratio * cy.wage / w1750 >= pp.found_saving;
                        }
                    }
                    real_pass += any as u8 as f64;
                }
            }
            if !v.is_empty() {
                println!(
                    "  {d}s: {:5} sites a decade on {} islands, saving {:.3}, at 1750's money wage {:.3}; island-years with one passing at 1750's money wage {:.2}",
                    v.len(),
                    outs.len(),
                    median(v),
                    median(v_real),
                    real_pass / years.max(1.0)
                );
            }
            d += 10;
        }
        println!("\n  the foundings: year, town, kind, saving over wages");
        for (o, seed) in outs.iter().zip(&seeds) {
            let line: Vec<String> = o.founded.iter().map(|(y, n, k, s)| format!("{y} {n} ({k}, {s:.3})")).collect();
            println!("  seed {seed}: {}", line.join("; "));
        }
        let last: Vec<f64> = outs.iter().map(|o| o.founded.iter().map(|f| f.0).max().unwrap_or(0) as f64).collect();
        println!("  the last founding: {}", cell(&last, 0));
    }

    if part == "c" || part == "all" {
        println!("\n(c) each town's fastest decade (from its tenth year; 3,000 people or more at the start), all islands pooled");
        let pp = &P.people;
        // per town: the decade [a, a + 10) with the greatest growth in people
        let mut rows: Vec<(u64, String, i64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64)> = Vec::new();
        let mut all = Vec::new();
        for (o, &seed) in outs.iter().zip(&seeds) {
            let tmax = o.towns.iter().map(|r| r.town).max().unwrap_or(0);
            for t in 0..=tmax {
                let ys: Vec<&TownYear> = o.towns.iter().filter(|r| r.town == t).collect();
                if ys.is_empty() {
                    continue;
                }
                let start = ys[0].founded.max(y0) + if ys[0].founded >= y0 { 10 } else { 0 };
                let mut best: Option<(f64, i64)> = None;
                for a in start..=(y1 - 9) {
                    let w: Vec<&&TownYear> = ys.iter().filter(|r| r.year >= a && r.year < a + 10).collect();
                    if w.len() < 10 || w[0].before < 3000.0 {
                        continue;
                    }
                    let g = (w[9].after / w[0].before).powf(0.1) - 1.0;
                    all.push(g);
                    if best.is_none_or(|(b, _)| g > b) {
                        best = Some((g, a));
                    }
                }
                let Some((g, a)) = best else { continue };
                let w: Vec<&&TownYear> = ys.iter().filter(|r| r.year >= a && r.year < a + 10).collect();
                let mean = |f: &dyn Fn(&TownYear) -> f64| w.iter().map(|r| f(r)).sum::<f64>() / w.len() as f64;
                rows.push((
                    seed,
                    o.names.get(t).cloned().unwrap_or_default(),
                    a,
                    g,
                    mean(&|r| r.rate),
                    mean(&|r| (r.mig_in - r.mig_out) / r.before),
                    mean(&|r| (r.after - r.before * (1.0 + r.rate) - (r.mig_in - r.mig_out)) / r.before),
                    mean(&|r| r.capped as u8 as f64),
                    mean(&|r| (r.inc >= 0.95 * pp.growth_top) as u8 as f64),
                    mean(&|r| r.crowd),
                    mean(&|r| r.urban / r.room.max(1.0)),
                    mean(&|r| r.y / r.y_island.max(1e-9)),
                    mean(&|r| r.wage_rel),
                ));
            }
        }
        rows.sort_by(|a, b| b.3.total_cmp(&a.3));
        println!("seed town            decade  growth/yr | births-deaths  migration  with cells | arrivals at limit  income near ceiling  crowding deaths | town people/room  income/island  wage/island");
        for r in rows.iter().take(25) {
            println!(
                "{:4} {:15} {:5}s  {:8.4} | {:13.4} {:10.4} {:11.4} | {:17.2} {:20.2} {:15.4} | {:16.2} {:14.2} {:12.2}",
                r.0, r.1, r.2, r.3, r.4, r.5, r.6, r.7, r.8, r.9, r.10, r.11, r.12
            );
        }
        let col = |k: usize| -> Vec<f64> {
            rows.iter()
                .map(|r| match k {
                    3 => r.3,
                    4 => r.4,
                    5 => r.5,
                    6 => r.6,
                    7 => r.7,
                    8 => r.8,
                    9 => r.9,
                    10 => r.10,
                    11 => r.11,
                    _ => r.12,
                })
                .collect()
        };
        println!(
            "\n  all {} towns' fastest decades, mean (median): growth {:.4} ({:.4}); births less deaths {:.4}; migration {:.4}; with cells {:.4}; years at the arrivals limit {:.2}; income near its ceiling {:.2}; crowding {:.4}; town people over room {:.2}; income over the island's {:.2}; wage over the island's {:.2}",
            rows.len(),
            stats(&col(3)).0,
            median(col(3)),
            stats(&col(4)).0,
            stats(&col(5)).0,
            stats(&col(6)).0,
            stats(&col(7)).0,
            stats(&col(8)).0,
            stats(&col(9)).0,
            stats(&col(10)).0,
            stats(&col(11)).0,
            stats(&col(12)).0
        );
        // the mean of the parts leans on a few towns whose market area jumped: the
        // medians, the share of decades led by each part, and the decades before and after 1850
        println!(
            "  the same, median: births less deaths {:.4}; migration {:.4}; with cells {:.4}",
            median(col(4)),
            median(col(5)),
            median(col(6))
        );
        let mut led = [0usize; 3];
        for r in &rows {
            let parts = [r.4, r.5, r.6];
            led[(0..3).fold(0, |b, k| if parts[k] > parts[b] { k } else { b })] += 1;
        }
        println!("  decades in which each part is the largest: births less deaths {}, migration {}, with cells {}", led[0], led[1], led[2]);
        for (label, early) in [("beginning before 1850", true), ("beginning in 1850 or after", false)] {
            let v: Vec<_> = rows.iter().filter(|r| (r.2 < 1850) == early).collect();
            let n = v.len().max(1) as f64;
            println!(
                "  {} decades {label}: growth {:.4}; births less deaths {:.4}; migration {:.4}; with cells {:.4} (means)",
                v.len(),
                v.iter().map(|r| r.3).sum::<f64>() / n,
                v.iter().map(|r| r.4).sum::<f64>() / n,
                v.iter().map(|r| r.5).sum::<f64>() / n,
                v.iter().map(|r| r.6).sum::<f64>() / n
            );
        }
        println!("  every town-decade from the tenth year, 3,000 people or more: {} of them, growth a year median {:.4}, ninetieth {:.4}", all.len(), median(all.clone()), {
            let mut v = all.clone();
            v.sort_by(f64::total_cmp);
            v.get(v.len() * 9 / 10).copied().unwrap_or(f64::NAN)
        });
        // every town-year, by half-century: how often each limit is reached
        println!("\n  every town-year (towns past their tenth year), by half-century: share of town-years at the arrivals limit, with the income term near its ceiling, with town people over room");
        for (a, b) in [(1750, 1799), (1800, 1849), (1850, 1900)] {
            let v: Vec<&TownYear> = outs.iter().flat_map(|o| o.towns.iter()).filter(|r| r.year >= a && r.year <= b && r.year >= r.founded + 10).collect();
            let n = v.len().max(1) as f64;
            println!(
                "  {a}-{b}: {:6} town-years; at the arrivals limit {:.3}; income near ceiling {:.3}; over room {:.3}; births less deaths {:.4}, net migration {:.4} a year",
                v.len(),
                v.iter().filter(|r| r.capped).count() as f64 / n,
                v.iter().filter(|r| r.inc >= 0.95 * pp.growth_top).count() as f64 / n,
                v.iter().filter(|r| r.urban > 1.0001 * r.room).count() as f64 / n,
                v.iter().map(|r| r.rate).sum::<f64>() / n,
                v.iter().map(|r| (r.mig_in - r.mig_out) / r.before).sum::<f64>() / n
            );
        }
    }
}
