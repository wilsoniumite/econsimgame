//! Scripted players for the family game: the player's family (family 0) played
//! by simple strategies through the public API only (`pending`, `decide`,
//! `post_member`, `set_stance`, `move_seat`, `site_options`, `found_works`,
//! `buy_land`, `family_report`, and the `families::prospects` hint).
//!
//!     cargo run -p isostate-core --release --example bots [--rules port|game] [SEED ...]
//!
//! The bots play under the game's rules (`game_params`, `game_content`), as a
//! player does; `--rules port` plays them on the port of the Python instead.
//! The argument parsing is that small: `--rules X` anywhere, every other
//! argument a seed.
//!
//! Strategies, each over a whole run with six families:
//!
//! - passive: takes every default.
//! - landlord: starts in estates and every year buys the land around its seat
//!   with the best rent per price that its spare cash allows (keeping a quarter
//!   of its cash).
//! - founder: the defaults, plus every year founds the best works it is allowed
//!   to at the seat, if the market there is deep enough.
//! - thoughtful: starts in the trade with the best prospects at the seat;
//!   trains children in the trades with the best prospects where they are
//!   strongest (a gifted child to university first); posts every idle adult to
//!   run a known trade where its prospects are best; each year grows the
//!   branches whose prospects clear the hurdle rate and holds the rest; founds
//!   works at the seat like the founder; and puts spare cash into land like the
//!   landlord.
//!
//! For each run: the player's net worth in 1775, 1800, 1850 and 1900, the
//! richest rival's in 1900, the player's rank among the six families (1 is
//! richest), the share of its income the family saved in its first ten years,
//! the first year its cash (between years) would pay for the best prudent works
//! at its seat in any trade, the works the family founded and how many stand in
//! 1900, and the trades and branches it has at the end.
#![allow(non_snake_case)]

use std::time::Instant;

use isostate_core::families::{self, N_GROUPS};
use isostate_core::*;

const CHECKPOINTS: [i64; 4] = [1775, 1800, 1850, 1900];
const RESERVE: f64 = 0.25;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Bot {
    Passive,
    Landlord,
    Founder,
    Thoughtful,
}

impl Bot {
    fn name(self) -> &'static str {
        match self {
            Bot::Passive => "passive",
            Bot::Landlord => "landlord",
            Bot::Founder => "founder",
            Bot::Thoughtful => "thoughtful",
        }
    }
}

#[derive(Default)]
struct Outcome {
    net: Vec<f64>,
    rank: Vec<usize>,
    rival: f64,
    saved: f64,
    works_cash: Option<i64>,
    founded: usize,
    standing: usize,
    trades: usize,
    branches: usize,
    secs: f64,
}

/// The decisions: the first trade and training by prospects, successions by default.
fn decide_all(s: &mut Sim, bot: Bot) {
    for d in pending(&s.st, 0) {
        let pick = match bot {
            Bot::Passive | Bot::Founder => d.default,
            Bot::Landlord => d.options.iter().position(|o| o.choice == Choice::Trade { group: Group::Estates }).unwrap_or(d.default),
            Bot::Thoughtful => {
                let score = |c: &Choice| -> f64 {
                    match *c {
                        Choice::Trade { group } => families::prospects(&s.st, &s.content, &s.params, s.st.families[0].seat)[group.index()],
                        Choice::Apprentice { group, town } => families::prospects(&s.st, &s.content, &s.params, town)[group.index()],
                        Choice::University => {
                            let fam = &s.st.families[0];
                            let gifted = d.member.is_some_and(|j| fam.members[j].skill >= 0.7);
                            if gifted && !fam.members.iter().any(|m| m.alive && m.educated) { f64::INFINITY } else { f64::NEG_INFINITY }
                        }
                        Choice::Head { .. } => 0.0,
                    }
                };
                if matches!(d.options[0].choice, Choice::Head { .. }) {
                    d.default
                } else {
                    (0..d.options.len()).max_by(|&a, &b| score(&d.options[a].choice).total_cmp(&score(&d.options[b].choice))).unwrap_or(d.default)
                }
            }
        };
        let _ = decide(&mut s.st, &s.content, 0, d.id, pick);
    }
}

/// Idle adults go to run a known trade where its prospects are best.
fn post_idle(s: &mut Sim) {
    let r = family_report(&s.st, &s.content, &s.params, 0).unwrap();
    let towns: Vec<usize> = (0..s.st.T_max).filter(|&t| s.st.town_active[t]).collect();
    let pros: Vec<[f64; N_GROUPS]> = towns.iter().map(|&t| families::prospects(&s.st, &s.content, &s.params, t)).collect();
    for m in r.members.iter().filter(|m| m.alive && m.age >= 18 && m.post == Post::Unposted && m.next.is_none()) {
        let mut best: Option<(f64, Group, usize)> = None;
        for &g in &r.known {
            for (k, &t) in towns.iter().enumerate() {
                if r.branches.iter().any(|b| b.group == g && b.town == t) {
                    continue;
                }
                let v = pros[k][g.index()];
                if best.is_none_or(|b| v > b.0) {
                    best = Some((v, g, t));
                }
            }
        }
        if let Some((v, g, t)) = best {
            if v > s.params.hurdle {
                let _ = post_member(&mut s.st, 0, m.index, Posting::Manager { group: g, town: t });
            }
        }
    }
}

/// Grow where the prospects clear the hurdle rate, hold elsewhere.
fn set_stances(s: &mut Sim) {
    let r = family_report(&s.st, &s.content, &s.params, 0).unwrap();
    for b in &r.branches {
        let p = families::prospects(&s.st, &s.content, &s.params, b.town)[b.group.index()];
        let stance = if b.group == Group::Estates || p > s.params.hurdle { Stance::Grow } else { Stance::Hold };
        let _ = set_stance(&mut s.st, 0, b.group, b.town, stance);
    }
}

/// The best works the family may found at its seat, in a deep enough market.
fn found_at_seat(s: &mut Sim) -> Option<usize> {
    let seat = s.st.families[0].seat;
    let spare = s.st.families[0].cash * (1.0 - RESERVE);
    let mut best: Option<(f64, usize, usize)> = None;
    for c in (0..s.st.N).filter(|&c| s.st.catch[c] == seat as i64) {
        let Ok(opts) = site_options(&s.st, &s.content, &s.params, 0, c) else { continue };
        for o in opts {
            let Ok(x) = &o.sums else { continue };
            if o.allowed.is_ok()
                && x.market_share <= s.params.family_max_share
                && x.price_ratio <= s.params.family_max_price_ratio
                && x.capital_cost <= spare
                && x.expected_return > s.params.hurdle
                && best.is_none_or(|b| x.expected_return > b.0)
            {
                best = Some((x.expected_return, o.method, c));
            }
        }
    }
    let (_, m, c) = best?;
    found_works(&mut s.st, &s.content, &s.params, 0, m, c, s.year, &mut s.events).ok()
}

fn buy(s: &mut Sim) {
    let st = &s.st;
    let mut cells: Vec<(f64, f64, usize)> = (0..st.N)
        .filter(|&c| may_buy_land(st, 0, c).is_ok() && st.cell_owner[c] < 0 && st.cell_rent_year[c] > 0.0)
        .map(|c| (st.cell_rent_year[c] / families::land_price(st, &s.params, c), st.cell_rent_year[c], c))
        .collect();
    cells.sort_by(|a, b| b.0.total_cmp(&a.0).then(b.1.total_cmp(&a.1)));
    let mut bought = 0;
    for (_, _, c) in cells {
        if bought >= 40 {
            break;
        }
        if families::land_price(&s.st, &s.params, c) > s.st.families[0].cash * (1.0 - RESERVE) {
            continue;
        }
        if buy_land(&mut s.st, &s.content, &s.params, 0, c).is_ok() {
            bought += 1;
        }
    }
}

/// The capital cost of the works with the best expected return the family could
/// found at its seat in any trade, by the prudent rules.
fn best_works_cost(s: &Sim) -> Option<f64> {
    let seat = s.st.families[0].seat;
    let mut best: Option<(f64, f64)> = None;
    for c in (0..s.st.N).filter(|&c| s.st.catch[c] == seat as i64) {
        let Ok(opts) = site_options(&s.st, &s.content, &s.params, 0, c) else { continue };
        for o in opts {
            let Ok(x) = &o.sums else { continue };
            if x.market_share <= s.params.family_max_share
                && x.price_ratio <= s.params.family_max_price_ratio
                && x.expected_return > s.params.hurdle
                && best.is_none_or(|b| x.expected_return > b.0)
            {
                best = Some((x.expected_return, x.capital_cost));
            }
        }
    }
    best.map(|b| b.1)
}

fn play(seed: u64, bot: Bot, rules: Rules) -> Outcome {
    let t0 = Instant::now();
    let mut P = rules.params();
    P.seed = seed;
    P.n_families = 6;
    let mut s = sim::start(&P, rules.content());
    let mut out = Outcome::default();
    let mut founded: Vec<usize> = Vec::new();
    let (mut spent, mut earned) = (0.0, 0.0);
    while s.year <= s.end_year {
        decide_all(&mut s, bot);
        match bot {
            Bot::Passive => {}
            Bot::Landlord => buy(&mut s),
            Bot::Founder => founded.extend(found_at_seat(&mut s)),
            Bot::Thoughtful => {
                post_idle(&mut s);
                set_stances(&mut s);
                founded.extend(found_at_seat(&mut s));
                buy(&mut s);
            }
        }
        sim::run_year(&mut s).expect("money is conserved");
        let year = s.year - 1;
        if year < s.content.start_year + 10 {
            let ly = family_report(&s.st, &s.content, &s.params, 0).unwrap().last_year;
            spent += ly.spending;
            earned += ly.earned();
            out.saved = 1.0 - spent / earned.max(1e-9);
        }
        if out.works_cash.is_none() && best_works_cost(&s).is_some_and(|cost| s.st.families[0].cash >= cost) {
            out.works_cash = Some(year);
        }
        if CHECKPOINTS.contains(&year) {
            let nets: Vec<f64> =
                (0..s.st.families.len()).map(|k| family_report(&s.st, &s.content, &s.params, k).unwrap().net_worth).collect();
            out.net.push(nets[0]);
            out.rank.push(1 + nets.iter().filter(|&&n| n > nets[0]).count());
            out.rival = nets[1..].iter().cloned().fold(0.0, f64::max);
        }
    }
    let st = &s.st;
    eprintln!("TEMP books: goods {:.2e} income {:.2e} spending {:.2e} years {}", st.acct.years.iter().map(|y| y.balance_goods).fold(0.0, f64::max), st.acct.years.iter().map(|y| y.balance_income).fold(0.0, f64::max), st.acct.years.iter().map(|y| y.balance_spending).fold(0.0, f64::max), st.acct.years.len());
    let ours: Vec<usize> =(0..st.F_count).filter(|&f| st.f_owner[f] == 0 && st.f_reason[f].kind == "family").collect();
    out.founded = ours.len();
    out.standing = ours.iter().filter(|&&f| st.f_alive[f]).count();
    let r = family_report(st, &s.content, &s.params, 0).unwrap();
    out.trades = r.known.len();
    out.branches = r.branches.len();
    let _ = founded;
    out.secs = t0.elapsed().as_secs_f64();
    out
}

fn money(x: f64) -> String {
    if x >= 1e6 { format!("{:.1}m", x / 1e6) } else { format!("{:.0}k", x / 1e3) }
}

fn main() {
    let mut rules = Rules::Game;
    let mut seeds: Vec<u64> = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--rules" {
            rules = args.next().as_deref().and_then(Rules::parse).expect("--rules port or --rules game");
        } else {
            seeds.push(a.parse().expect("seed"));
        }
    }
    if seeds.is_empty() {
        seeds = vec![1, 2, 3, 7];
    }
    println!("rules: {}\n", rules.as_str());
    let bots = [Bot::Passive, Bot::Landlord, Bot::Founder, Bot::Thoughtful];
    println!("| seed | strategy | 1775 | 1800 | 1850 | 1900 | richest rival 1900 | rank 1775/1800/1850/1900 | saved 1750s | cash for a works | works founded, standing | trades | branches | run |");
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    for &seed in &seeds {
        for &bot in &bots {
            let o = play(seed, bot, rules);
            let ranks: Vec<String> = o.rank.iter().map(|r| r.to_string()).collect();
            println!(
                "| {seed} | {} | {} | {} | {} | {} | {} | {} | {:.0}% | {} | {}, {} | {} | {} | {:.0} s |",
                bot.name(),
                money(o.net[0]),
                money(o.net[1]),
                money(o.net[2]),
                money(o.net[3]),
                money(o.rival),
                ranks.join("/"),
                100.0 * o.saved,
                o.works_cash.map_or("never".to_string(), |y| y.to_string()),
                o.founded,
                o.standing,
                o.trades,
                o.branches,
                o.secs
            );
        }
    }
}
