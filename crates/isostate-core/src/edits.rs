//! The hand: edits made to a running island from outside. Not in the Python;
//! see `docs/designs/sandbox.md`, sections 2 and 6.1.
//!
//! The sandbox lets whoever is at the window change what the simulation holds
//! (the money in a town's purses, its people, the rest of a year's crop) and
//! watch what follows. An edit is one more kind of order in a game's log
//! (`save::Order::ByHand`), dated by year and month and carried out between two
//! months, so a saved session played again gives the same island to the last
//! digit. An edit draws no random number of its own.
//!
//! The books stay whole. Money made or unmade by hand is booked in the state's
//! ledger (`State::by_hand`), which the monthly money check in `sim::run_month`
//! adds to what the island began with and what was issued. People made or
//! unmade are counted there too. The weather made worse or better is booked
//! nowhere, because nothing is made by hand: the crop is simply smaller.
//!
//! Nothing here is reached except through an edit, and a run with no edits has
//! a ledger of nothing, which the money check adds as 0.0: the same bits.
//!
//! Stage 3 of the design (section 6.3) adds three edits that change what the
//! island knows and how it is ruled rather than what it holds: an idea found
//! by hand (`Discover`), a rule's number or switch set (`Rule`), and a town
//! ordered on a square (`FoundTown`). It also adds a run with no window that
//! carries out a list of dated edits (`run_with`), which the core's `run`
//! command reads with `--edits FILE`.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::params::Params;
use crate::sim::{self, RunResult, Sim};
use crate::state::Event;
use crate::util::{fmt_pct, fmt_thousands_f0};

/// The weight of an edit's item of news: above the island's ordinary news, so
/// that a history of the decade keeps what the hand did.
const NEWS_WEIGHT: f64 = 0.6;

/// How far one edit may multiply money or people, and the harvest.
const MONEY_BOUNDS: (f64, f64) = (0.1, 10.0);
const PEOPLE_BOUNDS: (f64, f64) = (0.1, 10.0);
const HARVEST_BOUNDS: (f64, f64) = (0.2, 2.0);

/// The rules' names a `Rule` edit may not set (section 2.7 of the design): those
/// that size the run or are read only at its start. A name is refused when it
/// is one of these or its first part is, so `land` and `accounts` are refused
/// whole. The production tree's stage, its file and its seeded trades are laid
/// over the content as the run begins (`tree::content`), and are refused too.
pub const FIXED_AT_START: &[&str] = &[
    "tree.stage",
    "tree.file",
    "tree.seed_trades",
    "seed",
    "W",
    "H",
    "n_towns",
    "pop0",
    "T_max",
    "F_max",
    "warmup_years",
    "decay",
    "n_families",
    "end_year",
    "land",
    "accounts",
    "family_start_share",
    "family_start_cash",
];

/// Every rule's own switch, by the dotted name `--set` takes: `enabled` where
/// the rule has one, and the one switch of the three rules that have none
/// (`commons.carry_debt`, `hands.ration`, `money.true_total`). A `Rule` edit
/// may throw one of these only if it is also on [`SWITCHABLE`]. The rules'
/// lesser switches (`works.keep_order`, `prices.no_spoil_markup` and the like)
/// are set as any of their numbers is.
pub const SWITCHES: &[&str] = &[
    "prices.enabled",
    "engel.enabled",
    "machines.enabled",
    "harvest.enabled",
    "chronicle.enabled",
    "works.enabled",
    "railways.enabled",
    "carting.enabled",
    "people.enabled",
    "towns.enabled",
    "mines.enabled",
    "canals.enabled",
    "land.enabled",
    "accounts.enabled",
    "roads.enabled",
    "longrun.enabled",
    "transition.enabled",
    "gauges.enabled",
    "commons.carry_debt",
    "hands.ration",
    "money.true_total",
];

/// The switches that may be thrown in the middle of a run, found by trying
/// each (`tests/edits.rs`, `which_switches_can_be_thrown_in_a_running_island`):
/// on island 7 under the game's rules with six families, played to 1790, each
/// was thrown the other way on a copy at a year's turn, played five years,
/// thrown back and played five more, and every one came through with no
/// panic, no failed money check and no figure that is not a number.
/// `land.enabled` and `accounts.enabled` were left out untried: the island's
/// land is made once, and the books open once (both are refused whole by
/// [`FIXED_AT_START`]). Section 6.3 of the design says what "came through"
/// does not promise. The run to 2050's three switches (longrun, the
/// transition, the gauges) joined both lists when they met the sandbox's
/// stage 3 on night/g-batch-1, and were tried there as above: all came through.
pub const SWITCHABLE: &[&str] = &[
    "prices.enabled",
    "engel.enabled",
    "machines.enabled",
    "harvest.enabled",
    "chronicle.enabled",
    "works.enabled",
    "railways.enabled",
    "carting.enabled",
    "people.enabled",
    "towns.enabled",
    "mines.enabled",
    "canals.enabled",
    "roads.enabled",
    "longrun.enabled",
    "transition.enabled",
    "gauges.enabled",
    "commons.carry_debt",
    "hands.ration",
    "money.true_total",
];

/// One edit by hand. Stage 1 of the design has three; stage 3 adds three more.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "what", rename_all = "snake_case")]
pub enum Edit {
    /// A purse multiplied, in one town or (`town` None) in every town.
    Money {
        purse: Purse,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        town: Option<usize>,
        factor: f64,
    },
    /// A town's people multiplied.
    People { town: usize, factor: f64 },
    /// The rest of the year's crop multiplied, round one town (its market area)
    /// or (`town` None) on the whole island.
    Harvest {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        town: Option<usize>,
        factor: f64,
    },
    /// An idea (a method or a technique, by `Content::tech_name`) found in a
    /// town, as `tech::discovery` finds one once it has chosen, with no draw.
    Discover { tech: String, town: usize },
    /// One of the rules' numbers or switches set, by the dotted name `--set`
    /// takes (`money.growth`, `hurdle`, `rent_regime`, `hands.ration`).
    Rule { name: String, value: serde_json::Value },
    /// A town founded on a square at the year's end, by the hook the founding
    /// rule already has (`State::pp_found_next`).
    FoundTown { cell: usize },
}

/// An edit and when it is carried out in a run with no window: before the
/// month `before` (year, and month 1 to 12) is played. `[1800, 1]` is at the
/// turn of 1799 into 1800, before January 1800.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dated {
    pub before: (i64, usize),
    pub edit: Edit,
}

/// Whose money.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purse {
    Households,
    Landlords,
    Merchants,
}

impl Purse {
    fn whose(self) -> &'static str {
        match self {
            Purse::Households => "households'",
            Purse::Landlords => "landlords'",
            Purse::Merchants => "merchants'",
        }
    }
}

/// What the hand has made and unmade so far: money (added to the right side
/// of the money check), people, and how many edits were carried out.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ledger {
    pub money: f64,
    pub people: f64,
    pub count: usize,
}

/// A factor the hand may use, or a sentence saying why not.
fn factor_ok(factor: f64, (lo, hi): (f64, f64), what: &str) -> Result<(), String> {
    if !factor.is_finite() {
        return Err(format!("{what} cannot be multiplied by {factor}"));
    }
    if factor < lo || factor > hi {
        return Err(format!("{what} may be multiplied by {lo} to {hi} at a time, not by {factor}"));
    }
    Ok(())
}

/// A town that stands, or a sentence saying it does not.
fn town_ok(sim: &Sim, town: usize) -> Result<(), String> {
    if town < sim.st.T_max && sim.st.town_active[town] {
        Ok(())
    } else {
        Err(format!("there is no town {town} standing on the island"))
    }
}

/// How many months of the year in progress have been played (0 to 11): the
/// month an edit is dated in.
fn months_played(sim: &Sim) -> usize {
    let begun = 12 * (sim.year - sim.content.start_year).max(0) as usize;
    sim.t.saturating_sub(begun).min(11)
}

/// Carry an edit out on the run itself. A refused edit changes nothing. Ok is
/// the line that says what was done, which is also the title of the item of
/// news (kind "hand") the edit leaves in the island's history.
pub fn carry_out(sim: &mut Sim, edit: &Edit) -> Result<String, String> {
    let (mut cell, mut tech) = (None, None);
    let (title, text, town) = match edit {
        Edit::Money { purse, town, factor } => money(sim, *purse, *town, *factor)?,
        Edit::People { town, factor } => people(sim, *town, *factor)?,
        Edit::Harvest { town, factor } => harvest(sim, *town, *factor)?,
        Edit::Discover { tech: name, town } => {
            let (said, t) = discover(sim, name, *town)?;
            tech = Some(t);
            said
        }
        Edit::Rule { name, value } => rule(sim, name, value)?,
        Edit::FoundTown { cell: c } => {
            cell = Some(*c);
            found_town(sim, *c)?
        }
    };
    let when = sim.year as f64 + (months_played(sim) as f64 + 0.5) / 12.0;
    let mut ev = Event::new(when, "hand", NEWS_WEIGHT, title.clone(), text);
    if let Some(t) = town {
        ev.town = Some(t);
        ev.cell = Some(sim.st.town_node[t]);
    }
    if cell.is_some() {
        ev.cell = cell;
    }
    ev.tech = tech;
    sim.events.push(ev);
    sim.st.by_hand.count += 1;
    Ok(title)
}

/// A purse multiplied: the title, the text and the town of the news.
fn money(sim: &mut Sim, purse: Purse, town: Option<usize>, factor: f64) -> Result<(String, String, Option<usize>), String> {
    let what = format!("the {} money", purse.whose());
    factor_ok(factor, MONEY_BOUNDS, &what)?;
    if let Some(t) = town {
        town_ok(sim, t)?;
    }
    let st = &mut sim.st;
    let towns: Vec<usize> = match town {
        Some(t) => vec![t],
        None => (0..st.T_max).filter(|&t| st.town_active[t]).collect(),
    };
    let cash = match purse {
        Purse::Households => &mut st.hh_cash,
        Purse::Landlords => &mut st.lord_cash,
        Purse::Merchants => &mut st.merch_cash,
    };
    // a purse at or below nothing is left alone, so a levy cannot take the
    // island's money below zero
    let (mut before, mut after) = (0.0, 0.0);
    for &t in &towns {
        if cash[t] > 0.0 {
            let old = cash[t];
            cash[t] = old * factor;
            before += old;
            after += cash[t];
        }
    }
    let place = match town {
        Some(t) => format!("in {}", st.town_name[t]),
        None => "in every town".to_string(),
    };
    if before <= 0.0 {
        // nothing was changed: every purse asked for was at or below nothing
        return Err(format!("no {} purse {place} holds any money to multiply", purse.whose()));
    }
    st.by_hand.money += after - before;
    let title = format!("By hand: {} money {place}, {} to {}", purse.whose(), fmt_thousands_f0(before), fmt_thousands_f0(after));
    let text = format!(
        "The {} money {place} was multiplied by {factor} by hand. The difference, {}, is booked as money made by hand, and the island's money check counts it.",
        purse.whose(),
        fmt_thousands_f0(after - before)
    );
    Ok((title, text, town))
}

/// A town's people multiplied.
fn people(sim: &mut Sim, town: usize, factor: f64) -> Result<(String, String, Option<usize>), String> {
    let name = sim.st.town_name.get(town).cloned().unwrap_or_default();
    factor_ok(factor, PEOPLE_BOUNDS, &format!("{name}'s people"))?;
    town_ok(sim, town)?;
    let st = &mut sim.st;
    let before = st.pop[town];
    st.pop[town] = before * factor;
    st.by_hand.people += st.pop[town] - before;
    let title = format!("By hand: {name}'s people, {} to {}", fmt_thousands_f0(before), fmt_thousands_f0(st.pop[town]));
    let text = if factor >= 1.0 {
        format!("{name}'s people were multiplied by {factor} by hand. Those who arrive come with nothing, so the money a head falls.")
    } else {
        format!("{name}'s people were multiplied by {factor} by hand. Those who are left inherit, so the money a head rises.")
    };
    Ok((title, text, Some(town)))
}

/// The rest of the year's crop multiplied.
fn harvest(sim: &mut Sim, town: Option<usize>, factor: f64) -> Result<(String, String, Option<usize>), String> {
    factor_ok(factor, HARVEST_BOUNDS, "the harvest")?;
    if let Some(t) = town {
        town_ok(sim, t)?;
    }
    let before = match town {
        Some(t) => sim.st.harvest_town_now.get(t).copied().unwrap_or(1.0),
        None => sim.st.harvest_now,
    };
    let Sim { st, content: C, params: P, year, .. } = sim;
    crate::harvest::by_hand(st, C, P, town, factor)?;
    let after = match town {
        Some(t) => st.harvest_town_now.get(t).copied().unwrap_or(1.0),
        None => st.harvest_now,
    };
    let place = match town {
        Some(t) => format!("round {}", st.town_name[t]),
        None => "on the whole island".to_string(),
    };
    let title = format!("By hand: the rest of {year}'s harvest {place}, {} to {} of an ordinary year", fmt_pct(before, 0), fmt_pct(after, 0));
    let text = format!(
        "What is still to be brought in of {year}'s crop {place} was multiplied by {factor} by hand. Nothing is booked for it: the crop is simply {}.",
        if factor < 1.0 { "smaller" } else { "larger" }
    );
    Ok((title, text, town))
}

/// An idea found by hand: what `tech::discovery` does once it has chosen the
/// town, with no draw and no inventor named. The title, text and town of the
/// news, and the idea's index.
fn discover(sim: &mut Sim, name: &str, town: usize) -> Result<((String, String, Option<usize>), usize), String> {
    let C = &sim.content;
    let Some(t) = C.tech_name.iter().position(|n| n == name) else {
        return Err(format!("there is no method or technique called {name:?}"));
    };
    town_ok(sim, town)?;
    let label = C.tech_label[t].clone();
    let st = &mut sim.st;
    if C.tech_known[t] {
        return Err(format!("{label} was known before the island's history began"));
    }
    if st.tech_year[t] >= 0 {
        let where_ = st.town_name.get(st.tech_town[t].max(0) as usize).cloned().unwrap_or_default();
        return Err(format!("{label} was found already, in {where_} in {}", st.tech_year[t]));
    }
    st.know[(town, t)] = st.know[(town, t)].max(0.25);
    st.tech_year[t] = sim.year;
    st.tech_town[t] = town as i64;
    let place = st.town_name[town].clone();
    let title = format!("By hand: {label} found in {place} in {}", sim.year);
    let text = format!(
        "{label} was found in {place} by hand, with no inventor named. A quarter of the town knows it at once, and it spreads from there along the island's trade as any idea does."
    );
    Ok(((title, text, Some(town)), t))
}

/// One of the rules' numbers or switches set by hand: `sweep::set_field` on
/// the run's own parameters, then `Params::check`, and the refusals of
/// section 2.7 of the design.
fn rule(sim: &mut Sim, name: &str, value: &serde_json::Value) -> Result<(String, String, Option<usize>), String> {
    let first = name.split('.').next().unwrap_or_default();
    if FIXED_AT_START.contains(&first) || FIXED_AT_START.contains(&name) {
        return Err(format!("the rule {name} sizes the island or is read only as a run begins, and cannot be changed in the middle of one"));
    }
    if SWITCHES.contains(&name) && !SWITCHABLE.contains(&name) {
        return Err(format!("{name} is a rule's own switch, and it is not among those shown to be safe to throw in a running island (edits::SWITCHABLE)"));
    }
    if value.is_object() {
        return Err(format!("{name} cannot be set to {value}: set a rule's numbers one at a time"));
    }
    let P = crate::sweep::set_field(&sim.params, name, value).map_err(|e| e.replacen("--set ", "the rule ", 1))?;
    P.check()?;
    if P == sim.params {
        return Err(format!("the rule {name} is {value} already"));
    }
    // the value before, as the rules write it (a rule that stands at its
    // default is not written out, so its numbers are not there to read), and
    // the value as it was given
    let at = |doc: &serde_json::Value| name.split('.').try_fold(doc, |at, part| at.get(part)).map(|v| v.to_string());
    let old = serde_json::to_value(&sim.params).ok().as_ref().and_then(at).unwrap_or_else(|| "its default".to_string());
    let new = value.to_string();
    sim.params = P;
    let title = format!("By hand: the rule {name}, {old} to {new}");
    let text = format!(
        "The rule {name} was set by hand from {old} to {new}, and the island is run under it from this month on. A saved session keeps the change in its log, and plays it again in the same month."
    );
    Ok((title, text, None))
}

/// A town ordered on a square, founded at the year's end where the founding
/// rule founds its own (`peopling::found_towns`), whatever its district.
fn found_town(sim: &mut Sim, cell: usize) -> Result<(String, String, Option<usize>), String> {
    let st = &sim.st;
    if !sim.params.people.enabled {
        return Err("a town can be founded by hand only under the game's rule of people and towns (people.enabled), and it is off".to_string());
    }
    if cell >= st.N {
        return Err(format!("there is no square {cell} on the island, which has {}", st.N));
    }
    let (x, y) = (cell % st.world.W, cell / st.world.W);
    if !st.world.land[cell] {
        return Err(format!("the square at {x}, {y} is sea"));
    }
    if st.catch[cell] < 0 {
        return Err(format!("the square at {x}, {y} is in no town's market area"));
    }
    let active: Vec<usize> = (0..st.T_max).filter(|&t| st.town_active[t]).collect();
    if let Some(&t) = active.iter().find(|&&t| st.town_node[t] == cell) {
        return Err(format!("the square at {x}, {y} is {}'s market place already", st.town_name[t]));
    }
    if active.len() >= st.T_max {
        return Err(format!("the island has {} towns, as many as it can hold", st.T_max));
    }
    if let Some(c) = st.pp_found_next {
        return Err(format!("a town is already to be founded this year, at {}, {}", c % st.world.W, c / st.world.W));
    }
    let donor = st.catch[cell] as usize;
    let place = st.town_name[donor].clone();
    sim.st.pp_found_next = Some(cell);
    let title = format!("By hand: a town to be founded at {x}, {y}, in {place}'s district, at the end of {}", sim.year);
    let text = format!(
        "A town was ordered by hand on the square at {x}, {y}, which trades through {place}. It is founded at the end of {} whatever its district, with {place}'s prices and knowledge, and the people of the squares that then trade through it go to it.",
        sim.year
    );
    Ok((title, text, Some(donor)))
}

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// A run with no window and a list of edits: the plain loop of
/// `sim::run_with`, with each edit carried out before its month is played,
/// in the order of their dates (and of the list, for edits of the same
/// date). An edit refused, or dated outside the run, stops the run with a
/// sentence that names it. The run's money is checked every month.
pub fn play(P: &Params, content: Content, edits: &[Dated]) -> Result<Sim, String> {
    let mut list: Vec<&Dated> = edits.iter().collect();
    list.sort_by_key(|d| d.before);
    let mut sim = sim::start(P, content);
    sim.check_money = true;
    let first = (sim.year, 1);
    for d in &list {
        let (year, month) = d.before;
        if !(1..=12).contains(&month) {
            return Err(format!("an edit is dated before month {month} of {year}, and the months are 1 to 12"));
        }
        if d.before < first || year > sim.end_year {
            return Err(format!("an edit is dated before {} {year}, outside the run's years, {} to {}", MONTHS[month - 1], sim.year, sim.end_year));
        }
    }
    let mut next = 0;
    while sim.year <= sim.end_year {
        for month in 0..12 {
            while next < list.len() && list[next].before <= (sim.year, month + 1) {
                let d = list[next];
                next += 1;
                carry_out(&mut sim, &d.edit).map_err(|why| {
                    let edit = serde_json::to_string(&d.edit).unwrap_or_default();
                    format!("the edit before {} {}, {edit}, was refused: {why}", MONTHS[month], sim.year)
                })?;
            }
            sim::run_month(&mut sim, month)?;
        }
        sim::end_year(&mut sim);
    }
    Ok(sim)
}

/// `play`, finished as `sim::run_with` finishes a run. The parameters the
/// result carries are those the run began with: a rule set by hand is in the
/// run's news, and the list of edits says when.
pub fn run_with(P: &Params, content: Content, edits: &[Dated]) -> Result<RunResult, String> {
    let sim = play(P, content, edits)?;
    let mut result = sim::finish(sim);
    result.params = P.clone();
    Ok(result)
}
