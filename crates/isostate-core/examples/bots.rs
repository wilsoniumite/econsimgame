//! Scripted players: the player's family (family 0) played by simple strategies
//! through the public API only (`site_options`, `found_works`, `buy_land`,
//! `assign_member`, `family_report`), to see whether there is a game.
//!
//!     cargo run -p isostate-core --release --example bots [SEED ...]
//!
//! Strategies, each over a whole run with six families:
//!
//! - passive: does nothing.
//! - greedy: every year builds the works with the best expected return it can
//!   afford, up to `MAX_BUILDS`, keeping `RESERVE` of its cash back, searching
//!   the cells within `RADIUS` squares of every town.
//! - landlord: every year buys the land with the best rent per price (the
//!   anonymous gentry's land only), keeping the same reserve.
//! - manager: greedy, and every year puts its adults in charge of the groups it
//!   has most works in, the most skilled first.
//! - prudent: manager, but builds only where the works' output is at most a
//!   quarter of the town's demand and the price at most three times the base
//!   price (`market_share`, `price_ratio`), and puts what is left over into land.
//!
//! For each run: the player's net worth in 1775, 1800, 1850 and 1900 and its
//! rank among the families (1 is richest), the works it founded and how many
//! still stand, and the yearly return those works promised (`expected_return`)
//! against what they paid (dividends and closing cash, plus what is left,
//! less what they cost, per year of life).
#![allow(non_snake_case)]

use std::collections::BTreeSet;
use std::time::Instant;

use isostate_core::families::{Group, N_GROUPS};
use isostate_core::*;

const RADIUS: isize = 2;
const MAX_BUILDS: usize = 4;
const RESERVE: f64 = 0.25;
const CHECKPOINTS: [i64; 4] = [1775, 1800, 1850, 1900];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Bot {
    Passive,
    Greedy,
    Landlord,
    Manager,
    Prudent,
}

impl Bot {
    fn name(self) -> &'static str {
        match self {
            Bot::Passive => "passive",
            Bot::Greedy => "greedy",
            Bot::Landlord => "landlord",
            Bot::Manager => "manager",
            Bot::Prudent => "prudent",
        }
    }
}

/// One works the bot founded.
struct Works {
    fid: usize,
    year: i64,
    expected: f64,
    cost: f64,
    /// The town's price of the output over its base price, when founded.
    price_ratio: f64,
    /// (K, utilisation, cash, loss years, building) at the last year end it was alive.
    last: (f64, f64, f64, i64, bool),
    died: Option<i64>,
}

#[derive(Default)]
struct Outcome {
    net: Vec<f64>,
    rank: Vec<usize>,
    /// The richest rival's net worth in 1900.
    rival: f64,
    founded: usize,
    standing: usize,
    expected: f64,
    realised: f64,
    cells: usize,
    search_s: f64,
    deaths: Vec<(i64, String)>,
    /// Per works: (method, promised, realised, survived, price ratio at founding).
    per_works: Vec<(usize, f64, f64, bool, f64)>,
}

/// Land cells within `RADIUS` squares of any town.
fn near_towns(st: &State) -> Vec<usize> {
    let (H, W) = (st.world.H as isize, st.world.W as isize);
    let mut cells = BTreeSet::new();
    for t in 0..st.T_max {
        if !st.town_active[t] {
            continue;
        }
        let (y, x) = ((st.town_node[t] / st.world.W) as isize, (st.town_node[t] % st.world.W) as isize);
        for dy in -RADIUS..=RADIUS {
            for dx in -RADIUS..=RADIUS {
                let (ny, nx) = (y + dy, x + dx);
                if ny >= 0 && nx >= 0 && ny < H && nx < W {
                    let c = (ny * W + nx) as usize;
                    if st.world.land[c] {
                        cells.insert(c);
                    }
                }
            }
        }
    }
    cells.into_iter().collect()
}

fn build(s: &mut Sim, works: &mut Vec<Works>, search_s: &mut f64, prudent: bool) {
    let t0 = Instant::now();
    let mut options: Vec<(f64, usize, usize, f64)> = Vec::new();
    for c in near_towns(&s.st) {
        let Ok(opts) = site_options(&s.st, &s.content, &s.params, 0, c) else { continue };
        for o in opts {
            if let Ok(x) = o.sums {
                if prudent && (x.market_share > 0.25 || x.price_ratio > 3.0) {
                    continue;
                }
                if x.expected_return > s.params.hurdle {
                    options.push((x.expected_return, o.method, c, x.capital_cost));
                }
            }
        }
    }
    *search_s += t0.elapsed().as_secs_f64();
    options.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut built = 0;
    let mut used = BTreeSet::new();
    for (expected, m, c, cost) in options {
        let spare = s.st.families[0].cash * (1.0 - RESERVE);
        if built >= MAX_BUILDS {
            break;
        }
        if cost > spare || used.contains(&c) {
            continue;
        }
        if let Ok(fid) = found_works(&mut s.st, &s.content, &s.params, 0, m, c, s.year, &mut s.events) {
            used.insert(c);
            built += 1;
            let (t, g) = (s.st.f_town[fid], s.content.method_out[m]);
            let price_ratio = s.st.pbar[(t, g)] / s.content.base_price[g];
            works.push(Works { fid, year: s.year, expected, cost, price_ratio, last: (0.0, 0.0, 0.0, 0, true), died: None });
        }
    }
}

fn buy(s: &mut Sim) {
    let st = &s.st;
    let mut cells: Vec<(f64, f64, usize)> = (0..st.N)
        .filter(|&c| st.world.land[c] && st.catch[c] >= 0 && st.cell_owner[c] < 0 && st.cell_rent_year[c] > 0.0)
        .map(|c| (st.cell_rent_year[c] / families::land_price(st, &s.params, c), st.cell_rent_year[c], c))
        .collect();
    cells.sort_by(|a, b| b.0.total_cmp(&a.0).then(b.1.total_cmp(&a.1)));
    let mut bought = 0;
    for (_, _, c) in cells {
        if bought >= 40 {
            break;
        }
        let spare = s.st.families[0].cash * (1.0 - RESERVE);
        if families::land_price(&s.st, &s.params, c) > spare {
            continue;
        }
        if buy_land(&mut s.st, &s.content, &s.params, 0, c).is_ok() {
            bought += 1;
        }
    }
}

fn manage(s: &mut Sim) {
    let st = &s.st;
    let mut count = [0usize; N_GROUPS];
    for f in 0..st.F_count {
        if st.f_alive[f] && st.f_owner[f] == 0 {
            if let Some(g) = Group::of_method(&s.content, st.f_method[f]) {
                count[g.index()] += 1;
            }
        }
    }
    let mut groups: Vec<Group> = Group::ALL.into_iter().filter(|g| count[g.index()] > 0).collect();
    groups.sort_by(|a, b| count[b.index()].cmp(&count[a.index()]));
    let fam = &st.families[0];
    let mut adults: Vec<usize> = (0..fam.members.len()).filter(|&j| fam.members[j].alive && fam.members[j].age >= 18).collect();
    adults.sort_by(|&a, &b| fam.members[b].skill.total_cmp(&fam.members[a].skill));
    for (&j, &g) in adults.iter().zip(&groups) {
        let _ = assign_member(&mut s.st, 0, j, Some(g));
    }
}

fn works_value(s: &Sim, f: usize) -> f64 {
    let (st, C) = (&s.st, &s.content);
    let unit: f64 = (0..C.G).map(|g| C.m_cap[(st.f_method[f], g)] * st.pbar[(st.f_town[f], g)]).sum();
    (st.f_K[f] + st.f_order_K[f]) * unit
}

fn play(seed: u64, bot: Bot) -> Outcome {
    let mut P = default_params();
    P.seed = seed;
    P.n_families = 6;
    let mut s = sim::start(&P, default_content());
    let mut works: Vec<Works> = Vec::new();
    let mut out = Outcome::default();
    while s.year <= s.end_year {
        match bot {
            Bot::Passive => {}
            Bot::Greedy => build(&mut s, &mut works, &mut out.search_s, false),
            Bot::Landlord => buy(&mut s),
            Bot::Manager => {
                manage(&mut s);
                build(&mut s, &mut works, &mut out.search_s, false);
            }
            Bot::Prudent => {
                manage(&mut s);
                build(&mut s, &mut works, &mut out.search_s, true);
                buy(&mut s);
            }
        }
        sim::run_year(&mut s).expect("money is conserved");
        let year = s.year - 1;
        for w in works.iter_mut().filter(|w| w.died.is_none()) {
            let f = w.fid;
            if s.st.f_alive[f] {
                w.last = (s.st.f_K[f], s.st.f_u[f], s.st.f_cash[f], s.st.f_loss_years[f], s.st.f_order_K[f] > 0.0);
            } else {
                w.died = Some(year);
            }
        }
        if CHECKPOINTS.contains(&year) {
            let nets: Vec<f64> =
                (0..s.st.families.len()).map(|k| family_report(&s.st, &s.content, &s.params, k).unwrap().net_worth).collect();
            out.net.push(nets[0]);
            out.rank.push(1 + nets.iter().filter(|&&n| n > nets[0]).count());
            out.rival = nets[1..].iter().cloned().fold(0.0, f64::max);
        }
    }
    out.founded = works.len();
    out.standing = works.iter().filter(|w| w.died.is_none()).count();
    out.cells = s.st.cell_owner.iter().filter(|&&o| o == 0).count();
    let (mut e, mut r) = (0.0, 0.0);
    for w in &works {
        let end = w.died.unwrap_or(s.end_year);
        let years = (end - w.year + 1).max(1) as f64;
        let left = if w.died.is_none() { works_value(&s, w.fid) } else { 0.0 };
        e += w.expected;
        let realised = (s.st.f_paid[w.fid] + left - w.cost) / w.cost / years;
        r += realised;
        out.per_works.push((s.st.f_method[w.fid], w.expected, realised, w.died.is_none(), w.price_ratio));
        if let Some(d) = w.died {
            let m = s.st.f_method[w.fid];
            let (K, u, cash, loss, building) = w.last;
            let K0 = s.content.m_K0[m];
            let why = if building {
                "never built"
            } else if K < 0.06 * K0 {
                "shrank away"
            } else if loss >= 2 {
                "years of losses"
            } else if cash < 0.0 {
                "insolvent"
            } else {
                "other"
            };
            out.deaths.push((d - w.year, format!("{} {why} (u {u:.2})", s.content.method_name[m])));
        }
    }
    if !works.is_empty() {
        out.expected = e / works.len() as f64;
        out.realised = r / works.len() as f64;
    }
    out
}

fn main() {
    let seeds: Vec<u64> = {
        let v: Vec<u64> = std::env::args().skip(1).map(|a| a.parse().expect("seed")).collect();
        if v.is_empty() { vec![1, 2, 3, 7] } else { v }
    };
    let bots = [Bot::Passive, Bot::Greedy, Bot::Landlord, Bot::Manager, Bot::Prudent];
    println!("| seed | strategy | net worth 1775 | 1800 | 1850 | 1900 | richest rival 1900 | rank 1775/1800/1850/1900 | works founded | standing | promised return | realised | cells | search |");
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    let mut deaths: Vec<(i64, String)> = Vec::new();
    let mut per_works: Vec<(usize, f64, f64, bool, f64)> = Vec::new();
    let C = default_content();
    for &seed in &seeds {
        for &bot in &bots {
            let t = Instant::now();
            let o = play(seed, bot);
            let ranks: Vec<String> = o.rank.iter().map(|r| r.to_string()).collect();
            println!(
                "| {seed} | {} | {:.0} | {:.0} | {:.0} | {:.0} | {:.0} | {} | {} | {} | {:.0}% | {:.0}% | {} | {:.1} s of {:.0} s |",
                bot.name(),
                o.net[0],
                o.net[1],
                o.net[2],
                o.net[3],
                o.rival,
                ranks.join("/"),
                o.founded,
                o.standing,
                100.0 * o.expected,
                100.0 * o.realised,
                o.cells,
                o.search_s,
                t.elapsed().as_secs_f64()
            );
            deaths.extend(o.deaths);
            per_works.extend(o.per_works);
        }
    }
    if !per_works.is_empty() {
        println!("\nThe builders' works by method: promised and realised yearly return (medians), share still standing in 1900, output price over base price when founded (median)");
        println!("| method | works | promised | realised | standing | price / base |");
        println!("|---|---|---|---|---|---|");
        let median = |mut v: Vec<f64>| -> f64 {
            v.sort_by(|a, b| a.total_cmp(b));
            if v.is_empty() { f64::NAN } else { v[v.len() / 2] }
        };
        for m in 0..C.M {
            let w: Vec<&(usize, f64, f64, bool, f64)> = per_works.iter().filter(|x| x.0 == m).collect();
            if w.is_empty() {
                continue;
            }
            println!(
                "| {} | {} | {:.0}% | {:.0}% | {:.0}% | {:.2} |",
                C.method_name[m],
                w.len(),
                100.0 * median(w.iter().map(|x| x.1).collect()),
                100.0 * median(w.iter().map(|x| x.2).collect()),
                100.0 * w.iter().filter(|x| x.3).count() as f64 / w.len() as f64,
                median(w.iter().map(|x| x.4).collect())
            );
        }
    }
    if !deaths.is_empty() {
        let mut ages: Vec<i64> = deaths.iter().map(|d| d.0).collect();
        ages.sort();
        println!("\n{} of the bots' works closed; median age {} years. Why (last year-end state):", deaths.len(), ages[ages.len() / 2]);
        let mut kinds: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for (_, why) in &deaths {
            let key = why.split(" (").next().unwrap().to_string();
            *kinds.entry(key).or_insert(0) += 1;
        }
        let mut kinds: Vec<_> = kinds.into_iter().collect();
        kinds.sort_by(|a, b| b.1.cmp(&a.1));
        for (k, n) in kinds.iter().take(12) {
            println!("  {n:>4}  {k}");
        }
    }
}
