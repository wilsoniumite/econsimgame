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
//!
//! Stage 7a (section 6.7) adds the kind of edit that waits for the year's
//! turn (`Edit::at_turn`): a works built by hand (`BuildWorks`), paid for with
//! money booked in the ledger, and a works closed by hand (`CloseWorks`).
//! Given in the middle of a year such an edit is checked as the island stands
//! (`check`), waits with the family's works and land, and is carried out in
//! January after the books are closed. `works_offered` is the list the Works
//! tool shows on a square, judged by the same function as the edit.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::families::{self, Group, SiteOption, SiteSums};
use crate::firms::{self, SiteEval};
use crate::names;
use crate::params::Params;
use crate::rng::Rng;
use crate::sim::{self, RunResult, Sim};
use crate::state::{Event, State};
use crate::util::{fmt_pct, fmt_thousands, fmt_thousands_f0};
use crate::works;

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
/// So is the linked measure (`linked`, docs/rules/measures.md), whose series
/// are made when the record is, and whose first link is the warm-up's end.
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
    "linked",
    // (its series are made with the record; its dials may be turned)
    "pay.watch",
    // (the people's ages: their series are made with the record, and their rows at the start)
    "ages.watch",
    // (the state's purse: its treasury and its book are kept from the start; its chips may be set)
    "state.enabled",
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
    "owners.enabled",
    "tasks.enabled",
    "woods.enabled",
    "who_works.enabled",
    "pace.enabled",
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
/// Its owners and tasks (`owners.enabled`, `tasks.enabled`) joined both on
/// night/g-batch-2, where they met the Rules tab; both came through, tasks
/// tried a third time from 1905, after island 7 finds its first machine.
/// The woods' rule (`woods.enabled`, docs/rules/woods.md) joined both on
/// night/g-woods-managed and came through as they did. Who works by age
/// (`who_works.enabled`, docs/rules/who-works.md) joined both on
/// night/g-who-works: it acts only from 1901, so the trial to 1790 throws it
/// where it does nothing; thrown on after 1900 it counts from the next year's
/// end, and thrown off the hands are half the people from the next month.
/// Pace (`pace.enabled`, docs/rules/pace.md) joined both on night/g-timber-check,
/// when it was switched on: it too acts only from 1901, and thrown either way
/// after it only changes the share of the plant still to come that the works
/// ask for from the next month, which `tests/pace.rs` plays with the money
/// checked every month.
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
    "owners.enabled",
    "tasks.enabled",
    "woods.enabled",
    "who_works.enabled",
    "pace.enabled",
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
    /// A works of a method (by `Content::method_name`) built on a square, paid
    /// for with money made by hand. It waits for the year's turn.
    BuildWorks { method: String, cell: usize },
    /// A works closed, by its slot (the record's list of works, as
    /// `inspect::Thing::Works` takes it). It waits for the year's turn.
    CloseWorks { works: usize },
    /// An idea pinned (the atlas's third plate, `docs/atlas.md`; `docs/lab.md`):
    /// an idea of the tree (a method or a technique, by `Content::tech_name`)
    /// or a use of the tasks rule (by its name in `data/tasks.toml`), found on
    /// the island in `year` (`when` "in", the default), or held back until
    /// `year` and found after it as it would have been (`when` "not_before").
    /// It stands from the month it is given to the end of the run; a second
    /// pin of the same idea takes the first one's place.
    Pin {
        idea: String,
        year: i64,
        #[serde(default)]
        when: PinWhen,
    },
    /// A university founded by the treasury in the town whose district holds
    /// the square (docs/rules/state.md, "The university"). It waits for the
    /// year's turn, and takes its first students at the year's end.
    FoundUniversity { cell: usize },
    /// A town's university closed. It waits for the year's turn.
    CloseUniversity { town: usize },
    // TOWN-FLOW-HAND (the hand on a town's making, docs/designs/sandbox.md):
    /// A method (by `Content::method_name`) turned off in a town: no new works
    /// of it may stand there, founded by the island, by a family, opened next
    /// door as a sister or built by hand. The works of it standing keep it
    /// until they close. Carried out at once.
    Forbid { town: usize, method: String },
    /// A method turned off by hand in a town turned on again. At once.
    Allow { town: usize, method: String },
    /// Every works of a town that makes a good (by `Content::good_name`)
    /// closed, and every method that makes it turned off there: "Close all and
    /// forbid this kind here". It waits for the year's turn.
    CloseKind { town: usize, good: String },
    // GROUND-BRUSH (the brush on the ground, docs/designs/sandbox.md, 6.9):
    /// One stroke of the brush: a kind of ground laid down (`lay`) or taken
    /// away on every square within `size` squares of those the brush passed
    /// over, in the order it passed them. Carried out at once; what follows
    /// is the rules' own, from the next year's review of the land.
    Ground { kind: Ground, lay: bool, size: u8, squares: Vec<usize> },
}

/// What a pin asks of its idea.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PinWhen {
    /// Found in the year: held back before it, and found in it with no draw.
    #[default]
    In,
    /// Held back before the year, and after it left to the island.
    NotBefore,
}

/// An idea the island can find: one of the tree's (by its place in
/// `Content::tech_name`), or a use of the tasks rule (by its place in
/// `Content::tasks.uses`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Idea {
    Tech(usize),
    Use(usize),
}

/// A pin as the island holds it (`Ledger::pins`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pin {
    pub idea: Idea,
    pub year: i64,
    pub when: PinWhen,
}

/// What the pins say of an idea in a year (`pin_says`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pinned {
    /// No pin: the island's own rule finds it, or not.
    Free,
    /// Held back this year.
    Hold,
    /// Found this year, with no draw.
    Find,
}

/// What the pins say of `idea` in `year`: the hook `tech::discovery` and the
/// tasks rule's year's turn ask before they look for an idea. With no pin
/// given (every run with no edits) it is `Free`, and those rules are the
/// port's and the game's to the bit.
pub fn pin_says(st: &State, idea: Idea, year: i64) -> Pinned {
    match st.by_hand.pins.iter().find(|p| p.idea == idea) {
        None => Pinned::Free,
        Some(p) if year < p.year => Pinned::Hold,
        Some(p) if p.when == PinWhen::In => Pinned::Find,
        Some(_) => Pinned::Free,
    }
}

/// An idea of the tree found by its pin in `year`: what `tech::discovery`
/// does once it has chosen, in the town it weighs heaviest (the one that
/// handled most of the idea's goods last year, against their thresholds),
/// with no draw and no inventor named.
pub fn find_pinned(st: &mut State, C: &Content, t: usize, year: i64, events: &mut Vec<Event>) {
    let req: Vec<usize> = (0..C.G).filter(|&g| st.disc_X0[(t, g)] > 0.0).collect();
    let mut best: Option<(f64, usize)> = None;
    for i in (0..st.T_max).filter(|&i| st.town_active[i]) {
        let score: f64 = req.iter().map(|&g| st.town_prod_year[(i, g)] / st.disc_X0[(t, g)]).sum::<f64>() + 1e-9 * st.pop[i];
        if best.is_none_or(|(b, _)| score > b) {
            best = Some((score, i));
        }
    }
    let Some((_, i)) = best else { return };
    st.know[(i, t)] = st.know[(i, t)].max(0.25);
    st.tech_year[t] = year;
    st.tech_town[t] = i as i64;
    let (label, town) = (C.tech_label[t].clone(), st.town_name[i].clone());
    let mut ev = Event::new(
        year as f64 + 0.5,
        "discovery",
        0.9,
        format!("{label} found in {town}, in the year it was pinned to"),
        format!("{label} was pinned by hand to {year}, and is found in {town}, the town that handled most of what it is found on. A quarter of the town knows it at once, and it spreads from there along the island's trade as any idea does."),
    );
    ev.town = Some(i);
    ev.cell = Some(st.town_node[i]);
    ev.tech = Some(t);
    events.push(ev);
}

/// An idea by its name, with its label: the tree's ideas first, then the
/// tasks rule's uses.
pub fn idea_named(C: &Content, name: &str) -> Option<(Idea, String)> {
    if let Some(t) = C.tech_name.iter().position(|n| n == name) {
        return Some((Idea::Tech(t), C.tech_label[t].clone()));
    }
    let u = C.tasks.uses.iter().position(|u| u.name == name)?;
    Some((Idea::Use(u), use_label(name)))
}

/// A use's name as words: "office computing".
pub fn use_label(name: &str) -> String {
    let s = name.replace('_', " ");
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
}

impl Edit {
    /// Whether the edit is of the kind that waits for the year's turn (section
    /// 2.2 of the design): given in the middle of a year it is checked
    /// (`check`), kept with the orders that wait, and carried out in January,
    /// after the books are closed. The others are carried out at once.
    pub fn at_turn(&self) -> bool {
        matches!(self, Edit::BuildWorks { .. } | Edit::CloseWorks { .. } | Edit::FoundUniversity { .. } | Edit::CloseUniversity { .. } | Edit::CloseKind { .. })
    }
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
    /// The pins standing (`Edit::Pin`), which `pin_says` reads.
    pub pins: Vec<Pin>,
    /// TOWN-FLOW-HAND: the methods turned off by hand, as (town, method),
    /// which `forbidden` reads.
    pub forbidden: Vec<(usize, usize)>,
    /// GROUND-BRUSH: the squares whose deposit the hand has changed, as (the
    /// kind's place in `Ground::ALL`, square), for the viewer's map of what
    /// the ground holds (the standing timber is sent whole every year).
    pub ground: std::collections::BTreeSet<(u8, usize)>,
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
    let (mut cell, mut tech, mut firm) = (None, None, None);
    if edit.at_turn() {
        // the kind that waits is carried out only at a turn, and is checked first
        if months_played(sim) != 0 {
            return Err(format!("{} waits for the year's turn, and {} months of {} have been played", what_waits(edit), months_played(sim), sim.year));
        }
        check(sim, edit)?;
    }
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
        Edit::BuildWorks { method, cell: c } => {
            let (said, f) = build_works(sim, method, *c)?;
            (cell, firm) = (Some(*c), Some(f));
            said
        }
        Edit::CloseWorks { works } => {
            cell = sim.st.f_cell.get(*works).copied();
            firm = Some(*works);
            close_works(sim, *works)?
        }
        Edit::Pin { idea, year, when } => pin(sim, idea, *year, *when)?,
        Edit::FoundUniversity { cell: c } => {
            let t = university_town(sim, *c)?;
            cell = Some(sim.st.town_node[t]);
            let year = sim.year;
            let (title, text) = crate::university::found(&mut sim.st, t, year, true);
            (format!("By hand: {title}"), text, Some(t))
        }
        Edit::CloseUniversity { town: t } => {
            crate::university::may_close(&sim.st, &sim.params, *t)?;
            let year = sim.year;
            let (title, text) = crate::university::close(&mut sim.st, *t, year).expect("checked that a university is open there");
            (format!("By hand: {title}"), text, Some(*t))
        }
        // TOWN-FLOW-HAND: a method off or on in a town, and a kind closed and forbidden
        Edit::Forbid { town: t, method } | Edit::Allow { town: t, method } => {
            let (said, m) = method_here(sim, *t, method, matches!(edit, Edit::Forbid { .. }))?;
            tech = tech_of_method(&sim.content, m);
            said
        }
        Edit::CloseKind { town: t, good } => {
            let (said, m) = close_kind(sim, *t, good)?;
            tech = m.and_then(|m| tech_of_method(&sim.content, m));
            said
        }
        // GROUND-BRUSH
        Edit::Ground { kind, lay, size, squares } => {
            let (said, c) = ground(sim, *kind, *lay, *size, squares)?;
            cell = Some(c);
            said
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
    ev.firm = firm;
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

/// An idea pinned by hand (`Edit::Pin`): kept in the ledger, where the
/// rules that find ideas ask after it (`pin_says`). Nothing is found now; the
/// pin acts in its year.
fn pin(sim: &mut Sim, name: &str, year: i64, when: PinWhen) -> Result<(String, String, Option<usize>), String> {
    let C = &sim.content;
    let Some((idea, label)) = idea_named(C, name) else {
        return Err(format!("there is no idea called {name:?}: neither a method, a technique nor a use of the tasks rule"));
    };
    let st = &mut sim.st;
    let found = match idea {
        Idea::Tech(t) => {
            if C.tech_known[t] {
                return Err(format!("{label} was known before the island's history began"));
            }
            st.tech_year[t]
        }
        Idea::Use(u) => st.tasks.found.get(u).copied().unwrap_or(-1),
    };
    if found >= 0 {
        return Err(format!("{label} was found already, in {found}"));
    }
    // a tree's idea is looked for at the end of each year, a use at the turn
    // for the year after; a pin to find one must leave its year to come
    let first = match idea {
        Idea::Tech(_) => sim.year,
        Idea::Use(_) => sim.year + 1,
    };
    if when == PinWhen::In && year < first {
        return Err(format!("{label} cannot be found in {year}: the first year it can still be found in is {first}"));
    }
    let replaced = st.by_hand.pins.iter().position(|p| p.idea == idea).map(|k| st.by_hand.pins.remove(k));
    st.by_hand.pins.push(Pin { idea, year, when });
    let (title, what) = match when {
        PinWhen::In => (format!("By hand: {label} pinned to {year}"), format!("{label} was pinned by hand to {year}: the island will not find it before then, and finds it in that year whatever it has made.")),
        PinWhen::NotBefore => (format!("By hand: {label} held back until {year}"), format!("{label} was held back by hand until {year}: the island will not find it before then, and after it finds it as it would have.")),
    };
    let text = match replaced {
        Some(p) => format!("{what} The pin takes the place of the one given before, to {}.", p.year),
        None => what,
    };
    Ok((title, text, None))
}

/// Whether a name is fixed when the island is made (section 2.7): it is on
/// [`FIXED_AT_START`], or its first part is.
pub fn fixed_at_start(name: &str) -> bool {
    let first = name.split('.').next().unwrap_or_default();
    FIXED_AT_START.contains(&first) || FIXED_AT_START.contains(&name)
}

/// The rules as they would stand with `name` set to `value` by hand, or the
/// sentence that refuses it: `sweep::set_field` on the run's own parameters,
/// then `Params::check`, and the refusals of section 2.7 of the design.
fn rule_set(params: &Params, name: &str, value: &serde_json::Value) -> Result<Params, String> {
    if fixed_at_start(name) {
        return Err(format!("the rule {name} sizes the island or is read only as a run begins, and cannot be changed in the middle of one"));
    }
    if SWITCHES.contains(&name) && !SWITCHABLE.contains(&name) {
        return Err(format!("{name} is a rule's own switch, and it is not among those shown to be safe to throw in a running island (edits::SWITCHABLE)"));
    }
    if value.is_object() {
        return Err(format!("{name} cannot be set to {value}: set a rule's numbers one at a time"));
    }
    let P = crate::sweep::set_field(params, name, value).map_err(|e| e.replacen("--set ", "the rule ", 1))?;
    P.check()?;
    if P == *params {
        return Err(format!("the rule {name} is {value} already"));
    }
    Ok(P)
}

/// Why a `Rule` edit setting `name` to `value` would be refused under these
/// rules, in the sentence the edit itself refuses it with; None if it would
/// be carried out. The Rules tab draws its chips by this (`rulebook`), and the
/// edit is refused by the same function, so the two cannot disagree.
pub fn rule_refused(params: &Params, name: &str, value: &serde_json::Value) -> Option<String> {
    rule_set(params, name, value).err()
}

/// One of the rules' numbers or switches set by hand (`rule_set`).
fn rule(sim: &mut Sim, name: &str, value: &serde_json::Value) -> Result<(String, String, Option<usize>), String> {
    let P = rule_set(&sim.params, name, value)?;
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

// ---------------------------------------------------------------- the kind that waits (stage 7a)

/// What an edit that waits is, in a few words, for the sentence that says so.
fn what_waits(edit: &Edit) -> &'static str {
    match edit {
        Edit::BuildWorks { .. } => "a works built by hand",
        Edit::CloseWorks { .. } => "a works closed by hand",
        Edit::FoundUniversity { .. } => "a university founded by hand",
        Edit::CloseUniversity { .. } => "a university closed by hand",
        Edit::CloseKind { .. } => "a kind of works closed and forbidden by hand",
        _ => "this edit",
    }
}

/// Every refusal an edit of the kind that waits (`Edit::at_turn`) would meet
/// if it were carried out now, as the island stands. It only reads.
/// `carry_out` goes through it first, and `save::Game::give` asks it of such
/// an edit given in the middle of a year, on the run itself and never on a
/// copy of the island (section 2.2 of the design), before keeping the edit to
/// wait. The island moves on before the turn, so an edit checked in March can
/// still be refused in January, in the same words. An edit carried out at
/// once is checked as it is carried out, and of it this says Ok.
pub fn check(sim: &Sim, edit: &Edit) -> Result<(), String> {
    match edit {
        Edit::BuildWorks { method, cell } => build_plan(sim, method, *cell).map(|_| ()),
        Edit::CloseWorks { works } => closable(sim, *works),
        Edit::FoundUniversity { cell } => university_town(sim, *cell).map(|_| ()),
        Edit::CloseUniversity { town } => crate::university::may_close(&sim.st, &sim.params, *town),
        Edit::CloseKind { town, good } => kind_plan(sim, *town, good).map(|_| ()),
        _ => Ok(()),
    }
}

/// The town a university founded by hand on `cell` would stand in: the town
/// whose district the square trades through, if one may be founded there.
fn university_town(sim: &Sim, cell: usize) -> Result<usize, String> {
    let st = &sim.st;
    if cell >= st.N {
        return Err(format!("there is no square {cell} on the island, which has {}", st.N));
    }
    let t = st.catch[cell];
    if t < 0 {
        return Err(format!("the square at {} trades through no town", square_at(st, cell)));
    }
    crate::university::may_found(st, &sim.params, t as usize)?;
    Ok(t as usize)
}

/// A square's place, "31, 40".
fn square_at(st: &State, cell: usize) -> String {
    format!("{}, {}", cell % st.world.W, cell / st.world.W)
}

/// The town whose district a works built by hand on `cell` would trade in,
/// or the square's own sentence saying why no works can stand there, as the
/// families judge a square (`families::buildable`).
fn hand_square(st: &State, cell: usize) -> Result<usize, String> {
    if cell >= st.N {
        return Err(format!("there is no square {cell} on the island, which has {}", st.N));
    }
    families::buildable(st, cell).map_err(|why| format!("no works can stand on the square at {}: {}", square_at(st, cell), why.replacen("the cell", "it", 1)))
}

/// A works as the hand would build it: its capacity, its site's sums and
/// what it costs.
struct HandWorks {
    K: f64,
    ev: SiteEval,
    cost: f64,
}

/// What a works of method m built by hand on `cell`, a square of town i's
/// district, would be (if the town knows the method), and whether it may be
/// built, or the refusal it would meet. It is the one function behind both
/// the edit (`build_plan`) and the list the tool shows (`works_offered`), so
/// that the two cannot disagree.
///
/// Its size is what the island's own entrants build in that town under the
/// works rule (`works::entry_K`), so that the room on the square and the depth
/// of the market are judged as for any works. Its cost is what a family pays
/// (`families::found_at`): the site's sums on the one square
/// (`firms::evaluate_sites`), the plant and any machines at the town's
/// capital-goods prices, plus 15 per cent, with no family's bonus.
fn hand_works(sim: &Sim, m: usize, cell: usize, i: usize) -> (Option<HandWorks>, Result<(), String>) {
    let (st, C, P) = (&sim.st, &sim.content, &sim.params);
    if !(st.know[(i, m)] > 0.02) {
        return (None, Err(format!("{} is not known in {} yet", C.method_label[m], st.town_name[i])));
    }
    let K = works::entry_K(st, C, P, i, m);
    // TOWN-FLOW-HAND: a method turned off here by hand is refused, as it is to the island's own entrants
    if forbidden(st, i, m) {
        let ev = firms::evaluate_sites(st, C, P, i, m, &[cell], K, families::prices(st), families::wages(st));
        let cost = ev.cap_cost * 1.15;
        return (Some(HandWorks { K, ev, cost }), Err(forbidden_words(st, C, i, m)));
    }
    let ev = firms::evaluate_sites(st, C, P, i, m, &[cell], K, families::prices(st), families::wages(st));
    let cost = ev.cap_cost * 1.15;
    let allowed = if st.F_count >= st.f_alive.len() {
        Err(format!("the works' table is full: all {} of its places are taken", fmt_thousands(st.f_alive.len() as i64)))
    } else if !(cost.is_finite() && cost > 0.0) {
        Err(format!("{} cannot be priced in {} as things stand", C.method_label[m], st.town_name[i]))
    } else {
        works::room_on_cell(st, C, P, cell, m, K).map_err(|why| why.replacen("the cell", "the square", 1))
    };
    (Some(HandWorks { K, ev, cost }), allowed)
}

/// The method's index, the town and the works a `BuildWorks` would build, or
/// the first refusal it meets.
fn build_plan(sim: &Sim, method: &str, cell: usize) -> Result<(usize, usize, HandWorks), String> {
    let Some(m) = sim.content.method_name.iter().position(|n| n == method) else {
        return Err(format!("there is no method called {method:?}"));
    };
    let i = hand_square(&sim.st, cell)?;
    let (works, allowed) = hand_works(sim, m, cell, i);
    allowed?;
    Ok((m, i, works.expect("a works that may be built is of a method known in the town")))
}

/// The sums of a works by hand, laid out as the land agent's list lays out a
/// family's (`families::site_options`), with no bonus.
fn hand_sums(sim: &Sim, m: usize, i: usize, w: &HandWorks) -> SiteSums {
    let (st, C, ev) = (&sim.st, &sim.content, &w.ev);
    let (output, town_demand, town_output, market_share, price_ratio) = families::market_depth(st, C, i, m, w.K);
    SiteSums {
        town: i,
        K: w.K,
        gate_price: ev.pout[0],
        inputs: (0..C.G).filter(|&g| ev.a[(0, g)] > 0.0).map(|g| (g, ev.a[(0, g)], ev.pin[(0, g)])).collect(),
        input_cost: ev.in_cost[0],
        heat: ev.h[0],
        heat_price: ev.hunit[0],
        heat_cost: ev.heat_cost[0],
        labor: ev.l[0],
        wage: families::wages(st)[i],
        labor_cost: ev.labor_cost[0],
        margin: ev.margin[0],
        rent_bid: ev.rent_bid[0],
        capital_cost: w.cost,
        expected_return: ev.roi[0],
        bonus: 0.0,
        agglo: ev.agglo[0],
        output,
        town_demand,
        town_output,
        market_share,
        price_ratio,
    }
}

/// What the Works tool offers on a square: every method, with its sums at the
/// size and cost a works by hand would have, and `allowed` the very refusal a
/// `BuildWorks` of it would meet now (`hand_works` says both). Err with the
/// square's own sentence when no works can stand there. It only reads, and is
/// asked on the simulation's thread between months after each click.
pub fn works_offered(sim: &Sim, cell: usize) -> Result<Vec<SiteOption>, String> {
    let C = &sim.content;
    let i = hand_square(&sim.st, cell)?;
    Ok((0..C.M)
        .map(|m| {
            let (works, allowed) = hand_works(sim, m, cell, i);
            let sums = match &works {
                Some(w) => Ok(hand_sums(sim, m, i, w)),
                // (only a method the town does not know has none, and it is refused)
                None => Err(allowed.clone().err().unwrap_or_default()),
            };
            SiteOption { method: m, name: C.method_name[m].clone(), label: C.method_label[m].clone(), group: Group::of_method(C, m), sums, allowed }
        })
        .collect())
}

/// A works by hand is named as a family's would be, with the town's name
/// where the family's surname goes ("Melbourne Brickworks"), and numbered on
/// from the works of that very name: no draw.
fn hand_works_name(st: &State, C: &Content, m: usize, i: usize) -> String {
    let town = &st.town_name[i];
    let base = names::family_firm_name(&C.method_name[m], town, town);
    match names::works_of_name(&st.f_name, &base) {
        0 => base,
        n => format!("{base} No. {}", n + 1),
    }
}

/// A works built by hand at the turn: `create_firm` with its cost as its cash
/// (as a family's works has), the cost booked as money made by hand, its
/// reason kept with kind "hand". No family owns it; under the owners' rule
/// (docs/rules/owners.md) it is held as a works the town's savers found that
/// day is held, by the town's pool ratio (`owners::shares` sets it so). The
/// title, text and town of the news, and the new works' slot.
fn build_works(sim: &mut Sim, method: &str, cell: usize) -> Result<((String, String, Option<usize>), usize), String> {
    let (m, i, w) = build_plan(sim, method, cell)?;
    let year = sim.year;
    let Sim { st, content: C, params: P, .. } = sim;
    let name = hand_works_name(st, C, m, i);
    let reason = firms::reason_record(st, i, m, cell, &w.ev, 0, w.K, year, "hand");
    // the name is given, so create_firm draws nothing from this stream
    let mut unused = Rng::from_seed([0; 32]);
    let fid = firms::create_firm(st, C, P, &mut unused, i, m, cell, w.K, w.ev.rent_bid[0], reason, year, w.cost, Some(name), None)
        .expect("checked that the works' table has room");
    st.by_hand.money += w.cost;
    if P.owners.enabled {
        crate::owners::shares(st, P, fid);
    }
    st.new_firms_year.push(fid);
    let what = C.method_label[m].to_lowercase();
    let (place, good) = (st.town_name[i].clone(), C.good_label[C.method_out[m]].to_lowercase());
    // (the method's label is a trade or a product, "Brickworks", "Crucible steel", so it goes in brackets)
    let title = format!("By hand: {} ({what}) at {}, in {place}'s district, built for {}", st.f_name[fid], square_at(st, cell), fmt_thousands_f0(w.cost));
    let held = if P.owners.enabled {
        let s = crate::owners::peek(st, P, fid);
        format!("{place}'s landlords and households, {} and {}, as they would hold a works they had founded that day", fmt_pct(s[crate::owners::LANDLORDS], 0), fmt_pct(s[crate::owners::HOUSEHOLDS], 0))
    } else {
        format!("{place}'s households, as any works no family owns is")
    };
    let text = format!(
        "{} was built by hand, sized as the town's own new works are, to make up to {} {good} a month, and expecting to return {} a year at last year's prices. It cost {}, paid with money made by hand and booked as such; the works holds it as cash and buys its plant with it. No family owns it: it is held by {held}.",
        st.f_name[fid],
        fmt_thousands_f0(w.K),
        fmt_pct(w.ev.roi[0], 0),
        fmt_thousands_f0(w.cost)
    );
    Ok(((title, text, Some(i)), fid))
}

/// Whether there is a works `works` standing to be closed, or why not.
fn closable(sim: &Sim, works: usize) -> Result<(), String> {
    let st = &sim.st;
    if works >= st.F_count {
        return Err(format!("there is no works {works}: the island has had {} works, numbered from 0", fmt_thousands(st.F_count as i64)));
    }
    if !st.f_alive[works] {
        return Err(format!("{} closed already, in {}", st.f_name[works], st.f_died[works]));
    }
    Ok(())
}

/// A works closed by hand at the turn: `firms::close_firm` with the year,
/// which sends its cash where it always does, and then what the year's end
/// does after its own closings (`sim::yearly`), which is `sim::after_transport`
/// and nothing else: the works' squares, counts and towns made again from the
/// works that stand. (`st.same_counts` is made in `firms::update_productivity`
/// before the year's closings, so the year's end does not make it again after
/// them, and neither does this.) Nothing is booked by hand.
fn close_works(sim: &mut Sim, works: usize) -> Result<(String, String, Option<usize>), String> {
    closable(sim, works)?;
    let year = sim.year;
    let Sim { st, content: C, params: P, .. } = sim;
    let (i, cash, cell, born) = (st.f_town[works], st.f_cash[works], st.f_cell[works], st.f_born[works]);
    let name = st.f_name[works].clone();
    let owner = families::owner_of_firm(st, works);
    let shares = (owner.is_none() && P.owners.enabled).then(|| crate::owners::peek(st, P, works));
    firms::close_firm(st, C, P, works, year);
    sim::after_transport(st, C, P);
    let place = st.town_name[i].clone();
    let went = if cash < 0.0 {
        format!("its debt of {} fell on {place}'s merchants, as a creditor's loss", fmt_thousands_f0(-cash))
    } else {
        match (owner, shares) {
            (Some(k), _) => format!("its cash of {} went to the {} family", fmt_thousands_f0(cash), st.families[k].surname),
            (None, Some(s)) => {
                let lords = crate::owners::lords_part(&s, cash);
                format!(
                    "its cash of {} went to {place}'s landlords ({}) and households ({}) by their shares",
                    fmt_thousands_f0(cash),
                    fmt_thousands_f0(lords),
                    fmt_thousands_f0(cash - lords)
                )
            }
            (None, None) => format!("its cash of {} went to {place}'s households", fmt_thousands_f0(cash)),
        }
    };
    let what = C.method_label[st.f_method[works]].to_lowercase();
    let title = format!("By hand: {name} ({what}) at {}, in {place}'s district, closed; {went}", square_at(st, cell));
    let text = format!(
        "{name}, open since {born}, was closed by hand at the turn of {year}. It buys no more plant and gives up its ground. Nothing is booked by hand: its money goes where any closing works' money goes, and {went}."
    );
    Ok((title, text, Some(i)))
}

const MONTHS: [&str; 12] =["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// A run with no window and a list of edits: the plain loop of
/// `sim::run_with`, with each edit carried out before its month is played,
/// in the order of their dates (and of the list, for edits of the same
/// date). An edit refused, or dated outside the run, stops the run with a
/// sentence that names it. The run's money is checked every month.
pub fn play(P: &Params, content: Content, edits: &[Dated]) -> Result<Sim, String> {
    play_noting(P, content, edits, |_, _| {})
}

/// `play`, with `note` called after each month (`Some(month)`, 0 to 11) and
/// after each year's end (`None`), as the sweep notes its facts (`lab.rs`).
pub fn play_noting(P: &Params, content: Content, edits: &[Dated], mut note: impl FnMut(&Sim, Option<usize>)) -> Result<Sim, String> {
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
        if d.edit.at_turn() && month != 1 {
            let edit = serde_json::to_string(&d.edit).unwrap_or_default();
            return Err(format!("the edit before {} {year}, {edit}, waits for the year's turn: date it before January", MONTHS[month - 1]));
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
            note(&sim, Some(month));
        }
        sim::end_year(&mut sim);
        note(&sim, None);
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

// ---------------------------------------------------------------- the hand on a town's making (TOWN-FLOW-HAND)
//
// The viewer's Making panel (a town's making, `making.rs`) is also a handle:
// a click on a box of works offers to build another, close one, or close all
// and forbid the kind; a click on a faint kind offers to build one, or to find
// its idea; and each method that makes the box's good can be turned off for
// the town or on again. Building and closing one are the tray's own edits
// (`BuildWorks`, `CloseWorks`); the rest are `Forbid`, `Allow` and
// `CloseKind`, here.
//
// A method turned off is kept in the ledger (`Ledger::forbidden`), and the
// rules that found works ask `forbidden` before they weigh a method in a town:
// the island's entrants (`firms::entry`), a works opening a sister next door
// (`firms::spawn_sister`), the families (`families`), and the hand
// (`hand_works`). The list is empty in every run with no such edit, so those
// rules are the port's and the game's to the bit. A works keeps the method it
// was built with all its life, so a method turned off stops new works of it;
// the works of it standing go on until they close, by their own rules or by
// the hand.

/// Whether method m is turned off by hand in town i.
pub fn forbidden(st: &State, i: usize, m: usize) -> bool {
    !st.by_hand.forbidden.is_empty() && st.by_hand.forbidden.contains(&(i, m))
}

/// The sentence that refuses a works of a method turned off in a town.
fn forbidden_words(st: &State, C: &Content, i: usize, m: usize) -> String {
    format!("{} may not be built in {}: it was turned off there by hand", C.method_label[m], st.town_name[i])
}

/// A method's place among the ideas (`Content::tech_name`), for the news.
pub fn tech_of_method(C: &Content, m: usize) -> Option<usize> {
    (0..C.NT).find(|&t| C.tech_kind[t] == "method" && C.tech_name[t] == C.method_name[m])
}

/// The works of town t that make good g, standing now: the smallest plant
/// first, and of two alike the newer. "Close one" closes the first.
pub fn kind_works(st: &State, C: &Content, t: usize, g: usize) -> Vec<usize> {
    let mut ws: Vec<usize> = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_town[f] == t && C.method_out[st.f_method[f]] == g).collect();
    ws.sort_by(|&a, &b| st.f_K[a].total_cmp(&st.f_K[b]).then(b.cmp(&a)));
    ws
}

fn works_count(n: usize) -> String {
    if n == 1 { "1 works".to_string() } else { format!("{n} works") }
}

/// A method turned off (`off`) or on again in a town: the news's title, text
/// and town, and the method.
fn method_here(sim: &mut Sim, t: usize, method: &str, off: bool) -> Result<((String, String, Option<usize>), usize), String> {
    let C = &sim.content;
    let Some(m) = C.method_name.iter().position(|n| n == method) else {
        return Err(format!("there is no method called {method:?}"));
    };
    town_ok(sim, t)?;
    let st = &mut sim.st;
    let (label, place) = (C.method_label[m].clone(), st.town_name[t].clone());
    let standing = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_town[f] == t && st.f_method[f] == m).count();
    let k = st.by_hand.forbidden.iter().position(|&x| x == (t, m));
    if off {
        if k.is_some() {
            return Err(format!("{label} is turned off in {place} already"));
        }
        st.by_hand.forbidden.push((t, m));
        let title = format!("By hand: {label} turned off in {place}");
        let text = format!(
            "{label} was turned off in {place} by hand: no new works of it may stand there, whether the town's savers would found it, a family, or the hand itself, nor may a works of it open another next door. {}",
            match standing {
                0 => "None of it stands there now.".to_string(),
                1 => "The one works of it standing keeps the way it was built with until it closes.".to_string(),
                n => format!("The {n} works of it standing keep the way they were built with until they close."),
            }
        );
        Ok(((title, text, Some(t)), m))
    } else {
        let Some(k) = k else {
            return Err(format!("{label} is not turned off in {place}"));
        };
        st.by_hand.forbidden.remove(k);
        let title = format!("By hand: {label} turned on again in {place}");
        let text = format!("{label} was turned on again in {place} by hand: new works of it may stand there again, as the island and its families judge them.");
        Ok(((title, text, Some(t)), m))
    }
}

/// What a `CloseKind` would do as the island stands: the good, the works to
/// close (`kind_works`) and the methods that make it, or why nothing would.
fn kind_plan(sim: &Sim, t: usize, good: &str) -> Result<(usize, Vec<usize>, Vec<usize>), String> {
    let (st, C) = (&sim.st, &sim.content);
    let Some(g) = C.good_name.iter().position(|n| n == good) else {
        return Err(format!("there is no good called {good:?}"));
    };
    town_ok(sim, t)?;
    let ms: Vec<usize> = (0..C.M).filter(|&m| C.method_out[m] == g).collect();
    if ms.is_empty() {
        return Err(format!("no works on this island makes {}", C.good_label[g].to_lowercase()));
    }
    let ws = kind_works(st, C, t, g);
    if ws.is_empty() && ms.iter().all(|&m| forbidden(st, t, m)) {
        return Err(format!("no works in {} makes {}, and every way of making it is turned off there already", st.town_name[t], C.good_label[g].to_lowercase()));
    }
    Ok((g, ws, ms))
}

/// Every works of a town that makes a good closed at the turn, as a works
/// closed by hand is (`close_works`: `firms::close_firm`, which sends each
/// one's cash where it always goes, then `sim::after_transport` once), and
/// every method of the good turned off there. The news's title, text and
/// town, and the first method, for the news's idea.
fn close_kind(sim: &mut Sim, t: usize, good: &str) -> Result<((String, String, Option<usize>), Option<usize>), String> {
    let (g, ws, ms) = kind_plan(sim, t, good)?;
    let year = sim.year;
    let Sim { st, content: C, params: P, .. } = sim;
    let names: Vec<String> = ws.iter().map(|&f| st.f_name[f].clone()).collect();
    let cash: f64 = ws.iter().map(|&f| st.f_cash[f].max(0.0)).sum();
    for &f in &ws {
        firms::close_firm(st, C, P, f, year);
    }
    if !ws.is_empty() {
        sim::after_transport(st, C, P);
    }
    let newly: Vec<usize> = ms.iter().copied().filter(|&m| !forbidden(st, t, m)).collect();
    st.by_hand.forbidden.extend(newly.iter().map(|&m| (t, m)));
    let (place, gl) = (st.town_name[t].clone(), C.good_label[g].to_lowercase());
    let title = format!("By hand: the works making {gl} in {place} closed ({}), and no more may be built there", ws.len());
    let ways: Vec<String> = ms.iter().map(|&m| C.method_label[m].to_lowercase()).collect();
    let closed = if names.is_empty() {
        "None stood to be closed.".to_string()
    } else {
        format!(
            "{} closed: {}. Their cash of {} went where any closing works' money goes.",
            works_count(names.len()),
            crate::inspect::and_list(&names),
            fmt_thousands_f0(cash)
        )
    };
    let text = format!(
        "At the turn of {year} the hand closed every works in {place} that made {gl}, and turned off every way of making it there ({}): no new works of it may stand in {place} until a way is turned on again. {closed}",
        crate::inspect::and_list(&ways)
    );
    Ok(((title, text, Some(t)), ms.first().copied()))
}

/// The works "Build another here" (or "Build one here") would build in town
/// t to make good g: of every method of the good that a `BuildWorks` would
/// build in the town now, on the square of the town's district where it
/// should return most a year, the best of them. Its method's name, the square,
/// what it costs and what it should return; or the sentence that says why
/// none would be offered, in the edit's own words. As the tray does, a works
/// expected to lose money is not offered.
#[derive(Clone, Debug, PartialEq)]
pub struct Offer {
    pub method: String,
    pub label: String,
    pub cell: usize,
    pub cost: f64,
    pub ret: f64,
}

/// The `Offer` for good g in town t (see it), or why there is none. It only
/// reads.
pub fn best_build(sim: &Sim, t: usize, g: usize) -> Result<Offer, String> {
    let (st, C, P) = (&sim.st, &sim.content, &sim.params);
    town_ok(sim, t)?;
    let place = &st.town_name[t];
    let mut why: Vec<String> = Vec::new();
    let mut best: Option<Offer> = None;
    let district: Vec<usize> = (0..st.N).filter(|&c| st.catch[c] == t as i64 && families::buildable(st, c).is_ok()).collect();
    for m in (0..C.M).filter(|&m| C.method_out[m] == g) {
        if !(st.know[(t, m)] > 0.02) {
            why.push(format!("{} is not known in {place} yet", C.method_label[m]));
            continue;
        }
        if forbidden(st, t, m) {
            why.push(forbidden_words(st, C, t, m));
            continue;
        }
        let K = works::entry_K(st, C, P, t, m);
        let cells: Vec<usize> = district.iter().copied().filter(|&c| works::room_on_cell(st, C, P, c, m, K).is_ok()).collect();
        if cells.is_empty() {
            why.push(format!("no square of {place}'s district has room for {}", C.method_label[m].to_lowercase()));
            continue;
        }
        let ev = firms::evaluate_sites(st, C, P, t, m, &cells, K, families::prices(st), families::wages(st));
        let cell = cells[crate::util::argmax(&ev.roi)];
        // the edit's own judgement on that square
        match build_plan(sim, &C.method_name[m], cell) {
            Ok((_, _, w)) => {
                let o = Offer { method: C.method_name[m].clone(), label: C.method_label[m].clone(), cell, cost: w.cost, ret: w.ev.roi[0] };
                if best.as_ref().is_none_or(|b| o.ret > b.ret) {
                    best = Some(o);
                }
            }
            Err(e) => why.push(e),
        }
    }
    match best {
        Some(o) if o.ret > 0.0 => Ok(o),
        Some(o) => Err(format!(
            "none would pay: the best, {} on {}, would return {} a year at last year's prices",
            o.label.to_lowercase(),
            square_at(st, o.cell),
            fmt_pct(o.ret, 0)
        )),
        None if why.is_empty() => Err(format!("no works on this island makes {}", C.good_label[g].to_lowercase())),
        None => Err(why.join("; ")),
    }
}

// ---------------------------------------------------------------- the brush on the ground (GROUND-BRUSH)
//
// The atlas's fourth plate, the ground, is the viewer's Deposits view; this is
// its hand. A stroke of the brush is one edit, however many squares it passes
// over, and is carried out at once: the island's deposits are read once a year,
// at the review of the land (`land::update_land_use`, `mines::review`), so a
// deposit changed in March acts from that January on. Laying down raises a kind
// of ground on the squares the brush reaches, as rich as the island's own good
// deposits of it under the brush's middle and half that at its edge; taking
// away leaves none of it there. Nothing is opened or closed by the hand: the
// mines' rule finds a deposit laid down at its next review and sinks a working
// where one would pay, as on the island's own ground, and a working whose
// deposit is taken away raises nothing more from it, stands idle, and is given
// up once it has stood idle its years (docs/rules/mines.md, D). The standing
// timber is the woods' cover: trees are planted only where woods may stand,
// and taken away they leave no timber, since the hand makes no goods.

/// A kind of ground the brush lays down or takes away.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ground {
    Coal,
    IronOre,
    Clay,
    Stone,
    Timber,
}

impl Ground {
    /// In the order of the viewer's kinds of ground (`overlay::KINDS`), whose
    /// first four are the record's deposits in its order (`DEPOSIT_FIELDS`).
    pub const ALL: [Ground; 5] = [Ground::Coal, Ground::IronOre, Ground::Clay, Ground::Stone, Ground::Timber];

    /// The island's field it is (`World::field`), or "cover" for the standing timber.
    pub fn field(self) -> &'static str {
        match self {
            Ground::Coal => "coal",
            Ground::IronOre => "ore",
            Ground::Clay => "clay",
            Ground::Stone => "rock",
            Ground::Timber => "cover",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Ground::Coal => "coal",
            Ground::IronOre => "iron ore",
            Ground::Clay => "clay",
            Ground::Stone => "stone",
            Ground::Timber => "standing timber",
        }
    }

    /// The land sector that works it (`Content::sector_name`).
    fn sector(self) -> &'static str {
        match self {
            Ground::Coal => "coalmine",
            Ground::IronOre => "ironmine",
            Ground::Clay => "claypit",
            Ground::Stone => "quarry",
            Ground::Timber => "forestry",
        }
    }

    /// Its place in `ALL`.
    pub fn index(self) -> u8 {
        Ground::ALL.iter().position(|&g| g == self).unwrap_or(0) as u8
    }
}

/// The widest a brush may be: this many squares from a square it passes over.
pub const BRUSH_MOST: u8 = 4;

/// How rich a deposit is laid on an island that holds none of its kind, on
/// the fields' scale of 0 to 1: a good coalfield of the port's island.
const RICH_IF_NONE: f64 = 0.8;

/// How far a brush of `size` reaches: every square whose middle is within
/// `size` and a half squares of one it passes over (size 1 is the square and
/// the eight round it).
fn reach(size: u8) -> f64 {
    size as f64 + 0.5
}

/// What one stroke leaves on each square it changes, as (square, what it holds
/// after it), by the squares' order. `old` is the kind as the ground holds it
/// now, on the fields' scale of 0 to 1, on an island `w` squares wide.
/// Laying down, a square within reach is raised, where it holds less, to
/// `top(square)` times a share that is one under the brush's middle and falls
/// to a half at its edge, by the distance to the nearest square the brush
/// passed over; taking away, it is left with none. A square whose figure would
/// not change is left out. It only reads: the viewer paints its preview of a
/// stroke by it, so that the two cannot reach different squares.
pub fn stroke(old: &[f64], w: usize, lay: bool, size: u8, squares: &[usize], top: impl Fn(usize) -> f64) -> Vec<(usize, f64)> {
    let n = old.len();
    let (w, r) = (w.max(1), reach(size));
    let h = n / w;
    let k = r.floor() as i64;
    let mut near: std::collections::BTreeMap<usize, f64> = std::collections::BTreeMap::new();
    for &s in squares.iter().filter(|&&s| s < n) {
        let (sx, sy) = ((s % w) as i64, (s / w) as i64);
        for dy in -k..=k {
            for dx in -k..=k {
                let (x, y) = (sx + dx, sy + dy);
                let d = ((dx * dx + dy * dy) as f64).sqrt();
                if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 || d > r + 1e-9 {
                    continue;
                }
                let e = near.entry(y as usize * w + x as usize).or_insert(d);
                *e = e.min(d);
            }
        }
    }
    near.into_iter()
        .filter_map(|(c, d)| {
            let new = if lay { old[c].max(top(c) * (1.0 - 0.5 * (d / r).powi(2))) } else { 0.0 };
            (new != old[c]).then_some((c, new))
        })
        .collect()
}

/// How rich the brush lays a deposit under its middle: as rich as the
/// island's own good deposits of the kind, the mean of the richest tenth of
/// the squares of land that hold any (on an island that holds none,
/// `RICH_IF_NONE`). The island's ground as it stands, so a second stroke is
/// laid about as rich as the first.
pub fn richness(field: &[f64], land: &[bool]) -> f64 {
    let mut v: Vec<f64> = field.iter().zip(land).filter(|&(&f, &l)| l && f > 0.0).map(|(&f, _)| f).collect();
    if v.is_empty() {
        return RICH_IF_NONE;
    }
    v.sort_by(|a, b| b.total_cmp(a));
    let k = (v.len() / 10).max(1);
    v[..k].iter().sum::<f64>() / k as f64
}

/// Whether woods may stand on a square: land whose woodland and wild shares
/// pass what the land rule lets trees stay on (`land::update_land_use`, where
/// trees on less halve each year).
fn woods_may_stand(st: &State, C: &Content, c: usize) -> bool {
    let (fi, ns) = (st.forestry_sector, C.NS);
    st.world.land[c] && st.lam.rows > ns && c < st.lam.cols && st.lam[(fi, c)] + st.lam[(ns, c)] > 0.02
}

/// The field of a deposit, to change.
fn deposit_mut(world: &mut crate::world::World, kind: Ground) -> &mut Vec<f64> {
    match kind {
        Ground::Coal => &mut world.coal,
        Ground::IronOre => &mut world.ore,
        Ground::Clay => &mut world.clay,
        Ground::Stone | Ground::Timber => &mut world.rock,
    }
}

/// What a stroke of `kind` would change on the island as it stands, as
/// `stroke` says it, and how rich it lays a deposit (for the standing
/// timber nought: its top is each square's own, the stand its ground bears).
pub fn ground_plan(st: &State, C: &Content, kind: Ground, lay: bool, size: u8, squares: &[usize]) -> (Vec<(usize, f64)>, f64) {
    let w = st.world.W;
    match kind {
        Ground::Timber => {
            let top = |c: usize| if woods_may_stand(st, C, c) { st.world.forest_pot[c] } else { 0.0 };
            (stroke(&st.cover, w, lay, size, squares, top), 0.0)
        }
        _ => {
            let f = st.world.field(kind.field());
            let rich = richness(f, &st.world.land);
            (stroke(f, w, lay, size, squares, |c| if st.world.land[c] { rich } else { 0.0 }), rich)
        }
    }
}

/// The workings standing on the squares a stroke changes: those whose pit
/// head is on one, or whose lease takes one in (`mines.rs`, A).
fn workings_on(st: &State, C: &Content, kind: Ground, squares: &[(usize, f64)]) -> usize {
    let N = st.N;
    let Some(r) = C.sector_name.iter().position(|n| n == kind.sector()).and_then(|s| crate::mines::row(st, s)) else { return 0 };
    let mut heads: Vec<usize> = Vec::new();
    for &(c, _) in squares {
        let leased = st.mine_head[r * N + c];
        for h in [Some(c), (leased >= 0).then_some(leased as usize)].into_iter().flatten() {
            if st.mine_opened[r * N + h] != 0 && !heads.contains(&h) {
                heads.push(h);
            }
        }
    }
    heads.len()
}

/// A number of squares in words: "1 square", "21 squares".
fn squares_count(n: usize) -> String {
    if n == 1 { "1 square".to_string() } else { format!("{n} squares") }
}

/// A phrase begun with a capital.
fn capital_first(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
}

/// A stroke of the brush carried out (`Edit::Ground`): the news's title, text
/// and town, and the square the news is about (the middle one of the stroke).
fn ground(sim: &mut Sim, kind: Ground, lay: bool, size: u8, squares: &[usize]) -> Result<((String, String, Option<usize>), usize), String> {
    let (st, C, P) = (&sim.st, &sim.content, &sim.params);
    let n = st.N;
    if squares.is_empty() {
        return Err("a stroke of the brush passes over at least one square".to_string());
    }
    if let Some(&s) = squares.iter().find(|&&s| s >= n) {
        return Err(format!("there is no square {s} on the island, which has {n}"));
    }
    if size > BRUSH_MOST {
        return Err(format!("the brush reaches at most {BRUSH_MOST} squares from where it passes, not {size}"));
    }
    let mid = squares[squares.len() / 2];
    let near = (st.catch[mid] >= 0).then_some(st.catch[mid] as usize);
    let place = match near {
        Some(t) => format!("round {}, in {}'s district", square_at(st, mid), st.town_name[t]),
        None => format!("round {}", square_at(st, mid)),
    };
    let label = kind.label();
    let (changes, rich) = ground_plan(st, C, kind, lay, size, squares);
    if changes.is_empty() {
        return Err(match (lay, kind) {
            (true, Ground::Timber) => format!("no trees can be planted {place}: the woods there stand as thick as the ground bears, or the land is farmed, built on or worked"),
            (true, _) => format!("the ground {place} holds as much {label} as the brush lays already"),
            (false, Ground::Timber) => format!("no trees stand {place} to take away"),
            (false, _) => format!("there is no {label} {place} to take away"),
        });
    }
    let pits = if kind == Ground::Timber { 0 } else { workings_on(st, C, kind, &changes) };
    let row = C.sector_name.iter().position(|s| s == kind.sector()).and_then(|s| crate::mines::row(st, s));
    let (mines_on, idle) = (P.mines.enabled, P.mines.idle_years.max(1));
    let count = squares_count(changes.len());
    let st = &mut sim.st;
    for &(c, v) in &changes {
        if kind == Ground::Timber {
            st.cover[c] = v;
            continue;
        }
        deposit_mut(&mut st.world, kind)[c] = v;
        // what is laid down is fresh: nothing has been raised from it
        if let (true, Some(r)) = (lay, row) {
            st.mine_dug[(r, c)] = 0.0;
        }
        st.by_hand.ground.insert((kind.index(), c));
    }
    let worked = match pits {
        0 => "No working stands on it.".to_string(),
        1 => format!("The one working that raised it there raises nothing more from those squares; a working left with none stands idle, and the mines' rule gives it up once it has stood idle {idle} years."),
        p => format!("The {p} workings that raised it there raise nothing more from those squares; a working left with none stands idle, and the mines' rule gives it up once it has stood idle {idle} years."),
    };
    let follows = if mines_on {
        "No pit is opened by the hand: the island's adventurers find it at the next year's review of the land, and sink a working where one would pay, as they do on the island's own ground."
    } else {
        "No pit is opened by the hand: from the next year's review of the land it is bid for as the island's own ground of the kind is."
    };
    let (title, text) = match (lay, kind) {
        (true, Ground::Timber) => (
            format!("By hand: trees planted on {count} {place}"),
            format!("Trees were planted by hand on {count} {place}: a full stand of what the ground bears under the brush's middle and half of one at its edge, on the squares where woods may stand. Squares that are farmed, built on or worked take none. The woods are cut and grow back by their own rule from this month."),
        ),
        (false, Ground::Timber) => (
            format!("By hand: the trees on {count} {place} taken away"),
            format!("Every tree standing on {count} {place} was taken away by hand. No timber comes of them, since the hand makes no goods; the woods grow back there by their own rule, slowly."),
        ),
        (true, _) => (
            format!("By hand: {label} laid down on {count} {place}"),
            format!(
                "{} was laid down by hand on {count} {place}, as rich as the island's own best tenth of {label} ({rich:.2} on the fields' scale of 0 to 1) under the brush's middle and half that at its edge, and fresh: nothing has been raised from it. {follows}",
                capital_first(label)
            ),
        ),
        (false, _) => (format!("By hand: {label} taken away from {count} {place}"), format!("The {label} under {count} {place} was taken away by hand: none of it is left there. {worked}")),
    };
    Ok(((title, text, near), mid))
}
