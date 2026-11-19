//! Prints, for docs/probes/tr-loose-b.md (d), every town-year short of food
//! (under 0.82 of its need in its worst month, `people::growth`'s line), sorted
//! into five kinds, and for all but the first what the town's own grain did: its
//! district's harvest that year and the year before, the grain it made, sent and
//! received, its farm hands, and its store, price and wage month by month. It
//! ends with the count of each kind, and how often a town more than two years
//! old goes short after a district harvest under 0.95.
//!
//!     cargo run --release -p isostate-core --example spring_hunger -- [--seeds 1-32] [--families 6] [--set JSON]
//!
//! The kinds, in the order they are tried:
//!   jump     the town's people grew by half or more at the year's end and it ran
//!            short in January to March (the shelves of the town it was);
//!   new      a town founded at one of the last two year's ends, which
//!            `people::growth` does not count;
//!   lost     the town's people fell by a fifth or more at the year's end: it gave
//!            a district to another town;
//!   harvest  its district's harvest that year was under 0.95;
//!   other    the rest.
//!
//! The game's rules. `--set` lays a JSON object over the game's parameters, e.g.
//! `--set '{"harvest":{"enabled":false}}'`. Nothing here draws a random number
//! of the run's or changes its state.
#![allow(non_snake_case)]
use std::fmt::Write as _;

use isostate_core::accounts::{KG, KL};
use isostate_core::*;

const KINDS: [&str; 5] = ["jump", "new", "lost", "harvest", "other"];
const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// One town's month.
#[derive(Clone, Copy, Default)]
struct M {
    pop: f64,
    food: f64,
    /// Food in store at the month's end, in months of the town's need (a head a month of subsistence).
    store: f64,
    price: f64,
    wage: f64,
}

/// One town's year from the books.
#[derive(Clone, Copy, Default)]
struct Y {
    made: f64,
    sent: f64,
    received: f64,
    farm_hands: f64,
    harvest: f64,
}

#[derive(Default)]
struct Out {
    text: String,
    /// Every town-year short of food: (kind, worst month).
    short: Vec<(usize, usize)>,
    /// Town-years of towns more than two years old with no jump or loss at the
    /// year's end, by their district's harvest that year (under 0.95, and over),
    /// and of them those short of food.
    by_harvest: [[usize; 2]; 2],
}

fn trace(seed: u64, P: &Params) -> Out {
    let mut P = P.clone();
    P.seed = seed;
    let mut s = sim::start(&P, Rules::Game.content());
    let C = s.content.clone();
    let T = s.st.T_max;
    let food: Vec<usize> = (0..C.G).filter(|&g| C.props[(g, 0)] > 0.0).collect();
    let grain = C.gi("grain");
    let farm = C.sector_name.iter().position(|n| n == "farm").unwrap();
    let need = C.subsistence[0];
    let mut months: Vec<Vec<M>> = Vec::new(); // [year * 12 + month][town]
    let mut years: Vec<Vec<Y>> = Vec::new();
    let mut out = Out::default();
    let y0 = s.year;
    while s.year <= s.end_year {
        for month in 0..12 {
            sim::run_month(&mut s, month).expect("money is conserved");
            let st = &s.st;
            months.push(
                (0..T)
                    .map(|t| {
                        let stock: f64 = food.iter().map(|&g| st.inv[(t, g)]).sum();
                        M { pop: st.pop[t], food: if st.town_active[t] { st.food_ratio[t] } else { 9.0 }, store: stock / (need * st.pop[t]).max(1.0), price: st.price[(t, grain)], wage: st.wage[t] }
                    })
                    .collect(),
            );
        }
        // the harvest of the year now running, before the year's end draws the next
        let harvest_now: Vec<f64> = (0..T).map(|t| s.st.harvest_town_now.get(t).copied().unwrap_or(1.0)).collect();
        sim::end_year(&mut s);
        let st = &s.st;
        let Some(rec) = st.acct.year(st.acct.years.len() - 1) else { panic!("the books are kept") };
        years.push(
            (0..T)
                .map(|t| {
                    let g = &rec.goods[(t * C.G + grain) * KG..(t * C.G + grain + 1) * KG];
                    let l = &rec.land[(t * C.NS + farm) * KL..(t * C.NS + farm + 1) * KL];
                    Y { made: g[0] as f64, sent: g[10] as f64, received: g[11] as f64, farm_hands: l[2] as f64, harvest: harvest_now[t] }
                })
                .collect(),
        );
    }
    let st = &s.st;
    for k in 0..years.len() {
        let year = y0 + k as i64;
        for t in 0..T {
            if !st.town_name.get(t).is_some_and(|n| !n.is_empty()) {
                continue;
            }
            let ms: Vec<M> = (0..12).map(|m| months[k * 12 + m][t]).collect();
            if ms.iter().all(|m| m.food >= 9.0) {
                continue;
            }
            let founded = st.town_founded[t];
            let y = years[k][t];
            let yb = if k > 0 { years[k - 1][t] } else { Y::default() };
            let worst = ms.iter().map(|m| m.food).fold(f64::INFINITY, f64::min);
            let wm = ms.iter().enumerate().min_by(|a, b| a.1.food.total_cmp(&b.1.food)).unwrap().0;
            let pop_dec = if k > 0 { months[k * 12 - 1][t].pop } else { ms[0].pop };
            let jump = ms[0].pop / pop_dec.max(1.0);
            let kind = if jump >= 1.5 && wm <= 2 {
                0
            } else if founded >= year - 2 {
                1
            } else if jump <= 0.8 {
                2
            } else if y.harvest < 0.95 {
                3
            } else {
                4
            };
            if founded < year - 2 && jump > 0.8 && jump < 1.5 {
                let b = if y.harvest < 0.95 { 0 } else { 1 };
                out.by_harvest[b][0] += 1;
                if worst < 0.82 {
                    out.by_harvest[b][1] += 1;
                }
            }
            if !(worst < 0.82) {
                continue;
            }
            out.short.push((kind, wm));
            writeln!(
                out.text,
                "  seed {seed} {year} {} (founded {founded}): {}; worst {} {:.2}; people {:.0} in December before, {:.0} in January ({:.1} times)",
                st.town_name[t], KINDS[kind], MONTHS[wm], worst, pop_dec, ms[0].pop, jump
            )
            .unwrap();
            if kind == 0 {
                continue;
            }
            writeln!(
                out.text,
                "      district harvest {:.2} (the year before {:.2}); grain made {:.0} ({:.0} the year before, {:+.0}%), farm hands {:.0} ({:.0}), sent {:.0}, received {:.0}",
                y.harvest,
                yb.harvest,
                y.made,
                yb.made,
                100.0 * (y.made / yb.made.max(1.0) - 1.0),
                y.farm_hands,
                yb.farm_hands,
                y.sent,
                y.received
            )
            .unwrap();
            let row = |f: &dyn Fn(&M) -> f64| ms.iter().map(|m| format!("{:5.2}", f(m))).collect::<Vec<_>>().join(" ");
            writeln!(out.text, "      food got  {}", row(&|m| m.food)).unwrap();
            writeln!(out.text, "      in store  {}  (months of need, at each month's end)", row(&|m| m.store)).unwrap();
            writeln!(out.text, "      price     {}", row(&|m| m.price)).unwrap();
            writeln!(out.text, "      wage      {}", row(&|m| m.wage)).unwrap();
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut families, mut set): (Vec<u64>, usize, Option<String>) = ((1..=32).collect(), 6, None);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => seeds = sweep::parse_seeds(&next()).unwrap(),
            "--families" => families = next().parse().unwrap(),
            "--set" => set = Some(next()),
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let mut P = Rules::Game.params();
    P.n_families = families;
    P.accounts.enabled = true;
    if let Some(j) = &set {
        P = P.with_json_overrides(j).expect("a JSON object of parameters");
    }
    let outs: Vec<Out> = std::thread::scope(|sc| {
        let hs: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let P = P.clone();
                sc.spawn(move || trace(seed, &P))
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });
    println!("the game's rules, {families} families, seeds {}-{}{}", seeds[0], seeds[seeds.len() - 1], set.as_ref().map_or(String::new(), |s| format!(", with {s}")));
    println!();
    for o in &outs {
        print!("{}", o.text);
    }
    let all: Vec<(usize, usize)> = outs.iter().flat_map(|o| o.short.iter().copied()).collect();
    println!();
    println!("town-years short of food: {}", all.len());
    for (k, name) in KINDS.iter().enumerate() {
        let n = all.iter().filter(|x| x.0 == k).count();
        let late = all.iter().filter(|x| x.0 == k && x.1 >= 3).count();
        println!("  {name:<8} {n:3}  ({late} of them in April or later)");
    }
    let mut bh = [[0usize; 2]; 2];
    for o in &outs {
        for b in 0..2 {
            bh[b][0] += o.by_harvest[b][0];
            bh[b][1] += o.by_harvest[b][1];
        }
    }
    println!();
    println!("towns more than two years old with no jump or loss at the year's end: town-years, and short of food");
    for (b, label) in ["district harvest under 0.95", "0.95 and over"].iter().enumerate() {
        println!("  {label:<28} {:7}  {:4}  ({:.5} a town-year)", bh[b][0], bh[b][1], bh[b][1] as f64 / bh[b][0].max(1) as f64);
    }
}
