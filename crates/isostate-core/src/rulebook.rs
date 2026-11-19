//! The rules as a list, for the sandbox's Rules tab (not in the Python; see
//! `docs/designs/sandbox.md`, sections 4.6 and 6.6). It only reads.
//!
//! [`rulebook`] walks every name the sweep's `--set` can set, in the order of
//! `Params`, and says of each its value in the run, its value under the game's
//! rules, what kind of thing it is, and what a click on the tab may set it to.
//! A rule that stands at its default is not written out by the parameters'
//! serialiser (`skip_serializing_if = "is_default"`), so walking the written
//! form alone would miss every rule that is off: the fields left out are filled
//! from their own structs ([`skipped`]), so that a rule that is off is listed
//! with its numbers. The order of `Params` is read from the reader's own list of
//! its fields, so a number added to `Params` tomorrow is in the list with no
//! line written here; a rule added tomorrow needs one line in [`skipped`], and
//! the test `tests/rulebook.rs` says so if it is missing.
//!
//! What an offer would do is asked of one function, `edits::rule_refused`,
//! which is what an edit by hand (`Edit::Rule`) is refused by too: the tab and
//! the edit cannot disagree.

use std::collections::HashMap;

use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::edits::{self, SWITCHABLE, SWITCHES};
use crate::params::{game_params, Params};
use crate::sim::Sim;

/// The numbers that matter most, pinned at the head of the tab (section 4.6).
pub const PINNED: [&str; 11] = [
    "money.growth",
    "rent_regime",
    "hurdle",
    "freight",
    "discovery_rate",
    "diffusion_rate",
    "growth_cap",
    "harvest.island_sd",
    "harvest.foreseen",
    "people.found_saving",
    "rail_hurdle",
];

/// The word a name fixed when the island is made carries in place of offers.
pub const FIXED: &str = "fixed when the island is made";

/// A number's offers: the word on the chip and the share of the game's value
/// it sets.
pub const NUMBER_OFFERS: [(&str, f64); 5] = [("None", 0.0), ("Half", 0.5), ("As set", 1.0), ("Half again", 1.5), ("Double", 2.0)];

/// One name the sweep can set.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// the dotted name `--set` takes (`money.growth`, `hurdle`)
    pub name: String,
    /// the first part of the name, or "general" for the numbers at the top of `Params`
    pub rule: String,
    /// its value in the run
    pub value: Value,
    /// its value under the game's rules (`game_params`)
    pub game: Value,
    pub kind: Kind,
    /// what a click may set it to, each with why it would be refused, if it would
    pub offers: Vec<Offer>,
    /// `FIXED` for a name on `edits::FIXED_AT_START`, which offers nothing
    pub fixed: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// a rule's own switch (`edits::SWITCHES`), throwable in a running island
    /// when it is on `edits::SWITCHABLE`
    Switch { throwable: bool },
    /// another switch: a rule's lesser one (`works.keep_order`)
    Flag,
    /// a number
    Number,
    /// one of a few words: `rent_regime`, landlords or commons
    Choice,
    /// a list, a word or nothing: shown, and offers nothing
    Other,
}

/// What a click on a chip would set a name to.
#[derive(Clone, Debug, PartialEq)]
pub struct Offer {
    /// the chip's word: On, Off; Landlords, Commons; None, Half, As set, Half again, Double
    pub word: String,
    pub value: Value,
    /// why `Edit::Rule` would refuse it, in its own sentence (a value the rule has
    /// already is refused too: an edit that changes nothing is not made)
    pub refused: Option<String>,
}

/// The names of `Params`' fields, in the order they are declared, as its
/// reader lists them when it meets a field it does not know.
fn top_names() -> Vec<String> {
    match serde_json::from_value::<Params>(json!({ "\u{1}": 0 })) {
        Ok(_) => Vec::new(),
        Err(e) => {
            let said = e.to_string();
            let listed = said.split_once("expected").map_or("", |(_, tail)| tail);
            listed.split('`').skip(1).step_by(2).map(str::to_string).collect()
        }
    }
}

fn to_json<T: Serialize>(x: &T) -> Option<Value> {
    serde_json::to_value(x).ok()
}

/// A field the parameters' serialiser leaves out while it stands at its
/// default, read from the field itself. None for a name that is never left out
/// (or one added to `Params` with a `skip_serializing_if` and not added here,
/// which `tests/rulebook.rs` catches).
fn skipped(P: &Params, name: &str) -> Option<Value> {
    match name {
        "n_families" => to_json(&P.n_families),
        "family_bonus" => to_json(&P.family_bonus),
        "family_cons_income" => to_json(&P.family_cons_income),
        "family_cons_wealth" => to_json(&P.family_cons_wealth),
        "years_purchase" => to_json(&P.years_purchase),
        "land_rent_floor" => to_json(&P.land_rent_floor),
        "family_start_share" => to_json(&P.family_start_share),
        "family_start_cash" => to_json(&P.family_start_cash),
        "family_reserve_months" => to_json(&P.family_reserve_months),
        "family_max_share" => to_json(&P.family_max_share),
        "family_max_price_ratio" => to_json(&P.family_max_price_ratio),
        "family_seat_bonus" => to_json(&P.family_seat_bonus),
        "family_birth_rate" => to_json(&P.family_birth_rate),
        "absentee_cut" => to_json(&P.absentee_cut),
        "university_gain" => to_json(&P.university_gain),
        "end_year" => to_json(&P.end_year),
        "prices" => to_json(&P.prices),
        "engel" => to_json(&P.engel),
        "machines" => to_json(&P.machines),
        "harvest" => to_json(&P.harvest),
        "chronicle" => to_json(&P.chronicle),
        "works" => to_json(&P.works),
        "railways" => to_json(&P.railways),
        "carting" => to_json(&P.carting),
        "people" => to_json(&P.people),
        "towns" => to_json(&P.towns),
        "mines" => to_json(&P.mines),
        "canals" => to_json(&P.canals),
        "land" => to_json(&P.land),
        "tree" => to_json(&P.tree),
        "accounts" => to_json(&P.accounts),
        "roads" => to_json(&P.roads),
        "commons" => to_json(&P.commons),
        "hands" => to_json(&P.hands),
        "money" => to_json(&P.money),
        "longrun" => to_json(&P.longrun),
        "gauges" => to_json(&P.gauges),
        "transition" => to_json(&P.transition),
        _ => None,
    }
}

/// Every field of the parameters, in their order, the ones the serialiser
/// leaves out among them. A field that cannot be found is `null`.
fn whole(P: &Params) -> Map<String, Value> {
    let written = serde_json::to_value(P).ok().and_then(|v| v.as_object().cloned()).unwrap_or_default();
    let mut out = Map::new();
    for name in top_names() {
        let v = written.get(&name).cloned().or_else(|| skipped(P, &name)).unwrap_or(Value::Null);
        out.insert(name, v);
    }
    out
}

/// Each dotted name and its value, in order: a rule's struct is opened to its
/// numbers (and so on down, for a rule that holds a group of its own).
fn flat(P: &Params) -> Vec<(String, Value)> {
    fn walk(name: String, v: Value, out: &mut Vec<(String, Value)>) {
        match v {
            Value::Object(m) => {
                for (k, x) in m {
                    walk(format!("{name}.{k}"), x, out);
                }
            }
            v => out.push((name, v)),
        }
    }
    let mut out = Vec::new();
    for (k, v) in whole(P) {
        walk(k, v, &mut out);
    }
    out
}

fn kind_of(name: &str, value: &Value) -> Kind {
    if SWITCHES.contains(&name) {
        Kind::Switch { throwable: SWITCHABLE.contains(&name) }
    } else if value.is_boolean() {
        Kind::Flag
    } else if name == "rent_regime" {
        Kind::Choice
    } else if value.is_number() {
        Kind::Number
    } else {
        Kind::Other
    }
}

/// What a click may set a name to under these rules, each with its refusal.
fn offers_of(run: &Params, name: &str, kind: Kind, game: &Value) -> Vec<Offer> {
    let offer = |word: &str, value: Value| Offer { word: word.to_string(), refused: edits::rule_refused(run, name, &value), value };
    match kind {
        Kind::Switch { .. } | Kind::Flag => vec![offer("On", json!(true)), offer("Off", json!(false))],
        Kind::Choice => vec![offer("Landlords", json!("landlords")), offer("Commons", json!("commons"))],
        Kind::Number => {
            // 0, a half, as set, half as much again and double the game's figure, rounded for
            // a whole number; a figure two words give is offered once (As set keeps its own)
            let Some(g) = game.as_f64().filter(|g| *g != 0.0) else { return Vec::new() };
            let whole = game.is_u64() || game.is_i64();
            let at = |share: f64| -> (f64, Value) {
                let x = share * g + 0.0;
                if !whole {
                    (x, json!(x))
                } else if game.is_u64() && x >= 0.0 {
                    (x.round(), json!(x.round() as u64))
                } else {
                    (x.round(), json!(x.round() as i64))
                }
            };
            let as_set = at(1.0).0;
            let mut seen: Vec<f64> = Vec::new();
            let mut out = Vec::new();
            for (word, share) in NUMBER_OFFERS {
                let (x, value) = at(share);
                if seen.contains(&x) || (x == as_set && word != "As set") {
                    continue;
                }
                seen.push(x);
                out.push(offer(word, value));
            }
            out
        }
        Kind::Other => Vec::new(),
    }
}

/// Every name the sweep's `--set` can set, in the order of `Params`, with its
/// value in `run`, its value under the game's rules and what a click may set
/// it to. It runs on the simulation's thread between months.
pub fn rulebook(run: &Params) -> Vec<Entry> {
    let game: HashMap<String, Value> = flat(&game_params()).into_iter().collect();
    flat(run)
        .into_iter()
        .map(|(name, value)| {
            let game = game.get(&name).cloned().unwrap_or(Value::Null);
            let rule = match name.split_once('.') {
                Some((r, _)) => r.to_string(),
                None => "general".to_string(),
            };
            let kind = kind_of(&name, &value);
            let fixed = edits::fixed_at_start(&name).then(|| FIXED.to_string());
            let offers = if fixed.is_some() { Vec::new() } else { offers_of(run, &name, kind, &game) };
            Entry { name, rule, value, game, kind, offers, fixed }
        })
        .collect()
}

/// The name and label of each method and technique not yet found on the
/// island: neither known before its history began nor found since (what
/// `Edit::Discover` would not refuse for being found already).
pub fn unfound(sim: &Sim) -> Vec<(String, String)> {
    let (C, st) = (&sim.content, &sim.st);
    (0..C.tech_name.len())
        .filter(|&t| !C.tech_known.get(t).copied().unwrap_or(false) && st.tech_year.get(t).is_none_or(|&y| y < 0))
        .map(|t| (C.tech_name[t].clone(), C.tech_label.get(t).cloned().unwrap_or_default()))
        .collect()
}
