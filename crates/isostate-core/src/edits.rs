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

use serde::{Deserialize, Serialize};

use crate::sim::Sim;
use crate::state::Event;
use crate::util::{fmt_pct, fmt_thousands_f0};

/// The weight of an edit's item of news: above the island's ordinary news, so
/// that a history of the decade keeps what the hand did.
const NEWS_WEIGHT: f64 = 0.6;

/// How far one edit may multiply money or people, and the harvest.
const MONEY_BOUNDS: (f64, f64) = (0.1, 10.0);
const PEOPLE_BOUNDS: (f64, f64) = (0.1, 10.0);
const HARVEST_BOUNDS: (f64, f64) = (0.2, 2.0);

/// One edit by hand. Stage 1 of the design has three.
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
    let (title, text, town) = match edit {
        Edit::Money { purse, town, factor } => money(sim, *purse, *town, *factor)?,
        Edit::People { town, factor } => people(sim, *town, *factor)?,
        Edit::Harvest { town, factor } => harvest(sim, *town, *factor)?,
    };
    let when = sim.year as f64 + (months_played(sim) as f64 + 0.5) / 12.0;
    let mut ev = Event::new(when, "hand", NEWS_WEIGHT, title.clone(), text);
    if let Some(t) = town {
        ev.town = Some(t);
        ev.cell = Some(sim.st.town_node[t]);
    }
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
