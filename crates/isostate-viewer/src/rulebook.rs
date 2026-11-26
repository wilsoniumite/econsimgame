//! The sandbox's Rules tab (`docs/designs/sandbox.md`, 4.6 and 6.6): the
//! island's rules, to read and to change by hand.
//!
//! The rules come from the game's thread (`Cmd::Rules`), which walks every
//! name the sweep's `--set` can set (`isostate_core::rulebook`) and answers
//! between months with each name's value in this game, its value under the
//! game's rules and what a click may set it to (`Msg::Rules`); `game::pump`
//! hands the answer to `Known`. They are asked for when the tab opens and
//! after any order's answer, never every frame, and the tab is drawn again
//! only when the answer differs from the one drawn.
//!
//! At the top, under "Most asked", the pinned numbers. Then each rule by name,
//! in the order of the parameters, with On and Off where its switch may be
//! thrown in a running island, or its state in muted words where it may not;
//! a click on a rule's name opens it to its numbers, one rule open at a time.
//! A number offers None, Half, As set, Half again and Double (0, 0.5, 1, 1.5
//! and 2 times the game's figure), the chip at its value inked; a value that is
//! none of them shows as one more chip, muted, that does nothing; an offer an
//! edit would refuse is not drawn. A name that differs from the game's has a
//! dot in the hand's ink and the game's figure beside it.
//!
//! A chip pressed sends `Edit::Rule` as the tray sends its edits, under the
//! tray's guards and in its words, and the answer goes up as a toast as the
//! tray's does. The tab is filled here, through `RulesParams`, without coming
//! into ui.rs.

// see the note at the top of ui.rs
#![allow(deprecated)]

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use isostate_core::edits::Edit;
use isostate_core::rulebook::{Entry, Kind, PINNED};
use serde_json::Value;

use crate::family::chip_button;
use crate::game::Game;
use crate::live::{Cmd, Live, RulesNow};
use crate::sandbox::{BUSY, HAND_INK, PAST};
use crate::ui::{text, UiState, BTN, INK, MUTED, RULE};
use crate::{clock, Data, Timeline, Toasts};

/// What the Rules tab reads (`ui::panel_rebuild` takes it and hands it here).
#[derive(SystemParam)]
pub struct RulesParams<'w> {
    /// present in a sandbox; the tab is on the bar only there
    pub sandbox: Option<Res<'w, crate::sandbox::Sandbox>>,
    /// the rules as the game's thread last told them
    pub known: Option<Res<'w, Known>>,
    /// what waits and what could be taken back (`Standing::last`): a rule set by the last edit by
    /// hand has Take it back on its row (take_back.rs)
    pub live: Option<Res<'w, Live>>,
}

/// The name a rule was set by the last edit by hand, while it can be taken
/// back, and the month the island would go back to.
fn set_last(rules: &RulesParams) -> Option<(String, String)> {
    let s = &rules.live.as_ref()?.standing;
    match crate::take_back::last_edit(s)? {
        Edit::Rule { name, .. } => Some((name.clone(), crate::take_back::date_of(s.back?))),
        _ => None,
    }
}

/// Take it back, under the row of the name the last edit by hand set.
fn back_row(p: &mut ChildSpawnerCommands, f: &Handle<Font>, name: &str, undo: Option<&(String, String)>) {
    let Some((_, date)) = undo.filter(|u| u.0 == name) else { return };
    p.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(8.0), flex_wrap: FlexWrap::Wrap, padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)), ..default() }).with_children(|row| {
        chip_button(row, f, crate::take_back::CARD_CHIP, false, HAND_INK, crate::take_back::TakeBackChip);
        row.spawn(text(f, &format!("puts the island back to {date}, before it was set by hand"), 11.5, MUTED));
    });
}

/// The rules as the game's thread last told them, the ideas not yet found
/// with them, and which rule the tab has open.
#[derive(Resource, Default)]
pub struct Known {
    now: Option<RulesNow>,
    /// raised by one whenever the answer kept changes
    pub version: u64,
    /// the rule opened to its numbers, by its name ("money", "general")
    open: Option<String>,
    /// an edit from this tab is out, and its answer goes up as a toast
    waiting: bool,
    /// a question is out and its answer has not come (a script's `settle` waits for it)
    out: bool,
}

impl Known {
    /// An edit taken back (back.rs): the rules are asked for again.
    pub fn forget(&mut self) {
        (self.now, self.waiting, self.out) = (None, false, false);
        self.version += 1;
    }

    /// Keep an answer (`game::pump`). The version is raised only when it differs.
    pub fn keep(&mut self, now: RulesNow) {
        self.out = false;
        if self.now.as_ref() != Some(&now) {
            self.now = Some(now);
            self.version += 1;
        }
    }

    /// Every name the sweep can set, as the thread last told them; none before the first answer.
    pub fn entries(&self) -> &[Entry] {
        self.now.as_ref().map_or(&[], |n| &n.entries)
    }

    /// The name and label of each method and technique not yet found, as of the last answer.
    pub fn unfound(&self) -> &[(String, String)] {
        self.now.as_ref().map_or(&[], |n| &n.unfound)
    }

    /// Ask the game's thread for the rules (the tray's Idea asks for its list
    /// here, so that a script's `settle` waits for the answer as it does for the tab's).
    pub fn ask(&mut self, live: &Live) {
        live.send(Cmd::Rules);
        self.out = true;
    }

    pub fn entry(&self, name: &str) -> Option<&Entry> {
        self.entries().iter().find(|e| e.name == name)
    }

    /// A question is out and its answer has not come.
    pub fn out(&self) -> bool {
        self.out
    }

    /// The rule open to its numbers.
    pub fn open(&self) -> Option<&str> {
        self.open.as_deref()
    }

    /// Open a rule to its numbers (and close the one that was open).
    pub fn open_rule(&mut self, rule: &str) {
        self.open = Some(rule.to_string());
    }
}

/// A rule's name: a click opens it to its numbers, or closes it.
#[derive(Component)]
pub struct RuleHead(String);

/// A chip that sets a name to an offer's value.
#[derive(Component)]
pub struct RuleChip {
    pub name: String,
    pub word: String,
    value: Value,
    /// false for the chip at the value the name has: pressing it changes nothing
    acts: bool,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Known>().add_systems(
        Update,
        (
            ask.after(crate::game::pump),
            clicks,
            // (the answer is read in the frame it comes, as the tray reads its own, before a
            // question about the clicked square puts the note away)
            answers.after(crate::game::pump).before(crate::game::ask_site),
        )
            .run_if(resource_exists::<crate::sandbox::Sandbox>)
            .run_if(resource_exists::<Live>),
    );
}

/// The rules are asked for when the tab opens and after any order's answer
/// (`Game::answered`), and the tab is drawn again when the answer kept changes.
/// The Markets panel's Work half asks and draws again in the same way, for the
/// chips of the reckoning of pay that stand over its charts (`dial_chips`).
fn ask(live: Res<Live>, game: Res<Game>, mut known: ResMut<Known>, mut ui: ResMut<UiState>, mut asked: Local<Option<u64>>, mut drawn: Local<u64>) {
    let work = ui.tab == Some(crate::ui::Tab::Markets) && ui.markets == crate::ui::MarketsShow::Work;
    if ui.tab != Some(crate::ui::Tab::Rules) && !work {
        // asked afresh when the tab opens again
        *asked = None;
        return;
    }
    if *asked != Some(game.answered) {
        *asked = Some(game.answered);
        known.ask(&live);
    }
    if *drawn != known.version {
        *drawn = known.version;
        ui.dirty += 1;
    }
}

/// A rule's name pressed opens it (or closes it); a chip pressed sends its edit.
#[allow(clippy::too_many_arguments)]
fn clicks(
    heads: Query<(&Interaction, &RuleHead), Changed<Interaction>>,
    chips: Query<(&Interaction, &RuleChip), Changed<Interaction>>,
    mut known: ResMut<Known>,
    mut ui: ResMut<UiState>,
    sb: Res<crate::sandbox::Sandbox>,
    tl: Res<Timeline>,
    data: Res<Data>,
    live: Res<Live>,
    mut game: ResMut<Game>,
    mut toasts: ResMut<Toasts>,
    time: Res<Time>,
) {
    for (i, head) in &heads {
        if *i == Interaction::Pressed {
            known.open = if known.open.as_deref() == Some(head.0.as_str()) { None } else { Some(head.0.clone()) };
            ui.dirty += 1;
        }
    }
    for (i, chip) in &chips {
        if *i != Interaction::Pressed || !chip.acts {
            continue;
        }
        let now = time.elapsed_secs();
        let say = |toasts: &mut Toasts, line: &str| toasts.say("orders_refused", line, "By hand", now);
        // the tray's guards, in its words: the present only, and one thing at a time
        if (tl.t.floor() as usize) < data.latest() || data.r.filled >= data.r.ny {
            say(&mut toasts, PAST);
            continue;
        }
        if game.busy || live.turning() || sb.running() {
            say(&mut toasts, BUSY);
            continue;
        }
        live.send(Cmd::ByHand(Edit::Rule { name: chip.name.clone(), value: chip.value.clone() }));
        game.busy = true;
        game.note = None;
        known.waiting = true;
    }
}

/// The edit's answer goes up as a toast, as the tray's does: the line it wrote
/// in the island's news, or why it was refused. (The rules are asked for
/// again by `ask`, which sees the answer counted.)
fn answers(mut known: ResMut<Known>, game: Res<Game>, data: Res<Data>, live: Res<Live>, mut toasts: ResMut<Toasts>, time: Res<Time>) {
    if !known.waiting || game.busy {
        return;
    }
    known.waiting = false;
    let when = clock::month_year(clock::date(data.r.y0, clock::at(data.latest(), live.months.min(11))));
    if let Some((ok, line)) = &game.note {
        toasts.say(if *ok { "hand" } else { "orders_refused" }, line, &when, time.elapsed_secs());
        println!("rules: {} {when}: {line}", if *ok { "by hand," } else { "refused," });
    }
}

/// A value as the tab writes it: a number as it stands ("0.0175", "32"),
/// a switch as on or off, a word as it is, a list with commas.
pub fn figure(v: &Value) -> String {
    match v {
        Value::Bool(b) => if *b { "on" } else { "off" }.to_string(),
        Value::Number(n) => match n.as_f64() {
            Some(x) if n.is_f64() => {
                let s = format!("{x}");
                // (a figure the float writes long, 0.026249999999999996, is cut to what it says)
                if s.len() > 9 { format!("{}", (x * 1e6).round() / 1e6) } else { s }
            }
            _ => n.to_string(),
        },
        Value::String(s) => s.clone(),
        Value::Array(a) => a.iter().map(figure).collect::<Vec<_>>().join(", "),
        Value::Null => "none".to_string(),
        Value::Object(_) => "a group".to_string(),
    }
}

/// Two values the same, numbers by their figure (a rule's 1 and 1.0 are one value).
pub(crate) fn same(a: &Value, b: &Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

/// The name differs from the game's, and could be changed (a fixed name is the
/// island's making, not a rule set by hand).
fn differs(e: &Entry) -> bool {
    e.fixed.is_none() && !same(&e.value, &e.game)
}

/// The rules by name, in the order of the parameters: each with its own switch,
/// if it has one, and its other names.
fn by_rule(entries: &[Entry]) -> Vec<(&str, Option<&Entry>, Vec<&Entry>)> {
    let mut out: Vec<(&str, Option<&Entry>, Vec<&Entry>)> = Vec::new();
    for e in entries {
        let at = match out.iter().position(|r| r.0 == e.rule) {
            Some(i) => i,
            None => {
                out.push((&e.rule, None, Vec::new()));
                out.len() - 1
            }
        };
        if matches!(e.kind, Kind::Switch { .. }) && out[at].1.is_none() {
            out[at].1 = Some(e);
        } else {
            out[at].2.push(e);
        }
    }
    out
}

/// What a rule is called on the tab.
fn rule_label(rule: &str) -> String {
    match rule {
        "general" => "general numbers".to_string(),
        r => r.to_string(),
    }
}

/// The words a switch that cannot be thrown stands in: "on: cannot be thrown in a running island".
fn switch_state(e: &Entry) -> String {
    let why = match (&e.fixed, e.kind) {
        (Some(fixed), _) => fixed.clone(),
        (None, Kind::Switch { throwable: false }) => "cannot be thrown in a running island".to_string(),
        _ => String::new(),
    };
    if why.is_empty() { figure(&e.value) } else { format!("{}: {why}", figure(&e.value)) }
}

fn dot(p: &mut ChildSpawnerCommands) {
    p.spawn((Node { width: Val::Px(7.0), height: Val::Px(7.0), border_radius: BorderRadius::all(Val::Px(4.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(HAND_INK)));
}

/// A muted chip that does nothing: a value that is none of the offers.
fn muted_chip(p: &mut ChildSpawnerCommands, f: &Handle<Font>, label: &str) {
    p.spawn((Node { padding: UiRect::axes(Val::Px(9.0), Val::Px(3.0)), border_radius: BorderRadius::all(Val::Px(10.0)), border: UiRect::all(Val::Px(1.0)), ..default() }, BorderColor::all(RULE), BackgroundColor(BTN.with_alpha(0.4))))
        .with_children(|c| {
            c.spawn(text(f, label, 12.5, MUTED));
        });
}

/// The chips of a name: its offers that an edit would not refuse, the one at
/// its value inked, and its value as one more muted chip when it is none of
/// them. False when it has nothing to draw.
fn chips(p: &mut ChildSpawnerCommands, f: &Handle<Font>, e: &Entry) -> bool {
    let at_value = |v: &Value| same(v, &e.value);
    let shown: Vec<_> = e.offers.iter().filter(|o| o.refused.is_none() || at_value(&o.value)).collect();
    if shown.is_empty() {
        return false;
    }
    let none_at_value = !shown.iter().any(|o| at_value(&o.value));
    p.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(4.0), row_gap: Val::Px(4.0), ..default() }).with_children(|row| {
        for o in &shown {
            let on = at_value(&o.value);
            chip_button(row, f, &o.word, on, HAND_INK, RuleChip { name: e.name.clone(), word: o.word.clone(), value: o.value.clone(), acts: !on });
        }
        if none_at_value {
            muted_chip(row, f, &figure(&e.value));
        }
    });
    true
}

/// One name's row: the name, a dot and the game's figure where it differs,
/// then its chips; or its value in muted words where it offers nothing.
fn name_row(p: &mut ChildSpawnerCommands, f: &Handle<Font>, e: &Entry, label: &str, undo: Option<&(String, String)>) {
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)), ..default() }).with_children(|col| {
        let fixed = e.fixed.is_some();
        col.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(6.0), flex_wrap: FlexWrap::Wrap, ..default() }).with_children(|line| {
            line.spawn(text(f, label, 13.0, if fixed { MUTED } else { INK }));
            if differs(e) {
                dot(line);
                line.spawn(text(f, &format!("game: {}", figure(&e.game)), 12.0, MUTED));
            }
        });
        let drew = !fixed && chips(col, f, e);
        if !drew {
            let said = match &e.fixed {
                Some(why) => format!("{}: {why}", figure(&e.value)),
                None => figure(&e.value),
            };
            col.spawn(text(f, &said, 12.0, MUTED));
        }
        back_row(col, f, &e.name, undo);
    });
}

/// A name said in plain words where the tab has words for it: the two dials of
/// the reckoning of pay by class (docs/rules/pay.md), which are an argument's
/// two sides more than settings.
pub fn plain(name: &str) -> Option<&'static str> {
    match name {
        "pay.bend" => Some("How far demand for a class bends to its pay"),
        "pay.follow_years" => Some("How many years hands take to follow the work"),
        "state.school_years" => Some("Years of school the treasury pays for every child, who is out of work while at school"),
        "state.pension_age" => Some("The age the pension is paid from, when the old leave work"),
        "state.pension_share" => Some("The pension, as a share of the town's wage: those it is paid to leave work"),
        "state.university_share" => Some("The share of each cohort the university takes after school, out of work while there"),
        "who_works.enabled" => Some("Who works follows age: school, the university and the pension take hands out of work"),
        _ => None,
    }
}

/// The two dials of the reckoning of pay, as the tab draws them, for the Work
/// half of the Markets panel to stand over its charts of pay: only in a
/// sandbox, once the rules have been told.
pub fn dial_chips(p: &mut ChildSpawnerCommands, f: &Handle<Font>, rules: &RulesParams) {
    if rules.sandbox.is_none() {
        return;
    }
    let Some(known) = rules.known.as_ref().filter(|k| !k.entries().is_empty()) else { return };
    let undo = set_last(rules);
    for name in ["pay.bend", "pay.follow_years"] {
        if let Some(e) = known.entry(name) {
            name_row(p, f, e, plain(name).unwrap_or(name), undo.as_ref());
        }
    }
}

/// The words over the tab.
pub const ABOUT: &str = "The island's rules as they stand. A chip sets a number from this month on, as an edit by hand; a dot marks one that differs from the game's.";
/// Before the first answer.
pub const ASKING: &str = "Asking the simulation for its rules...";
/// In a game that is not a sandbox (a picture asked for the tab by name).
const NO_HAND: &str = "The rules are changed by hand only in a sandbox, which a plain start opens.";

fn heading(p: &mut ChildSpawnerCommands, f: &Handle<Font>, s: &str) {
    p.spawn((Node { margin: UiRect::top(Val::Px(8.0)), padding: UiRect::bottom(Val::Px(3.0)), border: UiRect::bottom(Val::Px(1.0)), ..default() }, BorderColor::all(RULE))).with_children(|h| {
        h.spawn(text(f, s, 13.5, INK));
    });
}

/// The Rules tab's body.
pub fn panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, rules: &RulesParams) {
    if rules.sandbox.is_none() {
        p.spawn(text(f, NO_HAND, 14.0, MUTED));
        return;
    }
    let Some(known) = rules.known.as_ref().filter(|k| !k.entries().is_empty()) else {
        p.spawn(text(f, ASKING, 14.0, MUTED));
        return;
    };
    p.spawn(text(f, ABOUT, 12.5, MUTED));
    let undo = set_last(rules);
    heading(p, f, "Most asked");
    for name in PINNED {
        if let Some(e) = known.entry(name) {
            name_row(p, f, e, plain(name).unwrap_or(name), undo.as_ref());
        }
    }
    heading(p, f, "Every rule");
    for (rule, switch, names) in by_rule(known.entries()) {
        let open = known.open() == Some(rule);
        let changed = switch.is_some_and(differs) || names.iter().any(|e| differs(e));
        p.spawn(Node { flex_direction: FlexDirection::Column, border: UiRect::bottom(Val::Px(1.0)), padding: UiRect::vertical(Val::Px(2.0)), ..default() }).insert(BorderColor::all(RULE)).with_children(|block| {
            block.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(8.0), row_gap: Val::Px(3.0), ..default() }).with_children(|line| {
                // the name, which opens the rule
                line.spawn((
                    Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(6.0), padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
                    BackgroundColor(if open { BTN } else { Color::NONE }),
                    Interaction::default(),
                    FocusPolicy::Block,
                    crate::ui::Hoverable,
                    RuleHead(rule.to_string()),
                ))
                .with_children(|n| {
                    n.spawn(text(f, &rule_label(rule), 14.0, INK));
                    if changed {
                        dot(n);
                    }
                    n.spawn(text(f, &format!("{} {}", names.len(), if names.len() == 1 { "number" } else { "numbers" }), 11.5, MUTED));
                });
                // its own switch: On and Off where it may be thrown, its state in words where not
                if let Some(s) = switch {
                    let throwable = s.fixed.is_none() && s.offers.iter().any(|o| o.refused.is_none());
                    if !throwable || !chips(line, f, s) {
                        line.spawn(text(f, &switch_state(s), 12.0, MUTED));
                    }
                    if differs(s) {
                        line.spawn(text(f, &format!("game: {}", figure(&s.game)), 12.0, MUTED));
                    }
                }
            });
            if let Some(s) = switch {
                back_row(block, f, &s.name, undo.as_ref());
            }
            if open {
                for e in &names {
                    let short = e.name.strip_prefix(&format!("{rule}.")).unwrap_or(&e.name);
                    name_row(block, f, e, plain(&e.name).unwrap_or(short), undo.as_ref());
                }
            }
        });
    }
}

/// The tab in one line for a scripted run's `print`: how many names it knows,
/// the rule open, and the names that differ from the game's with their values.
pub fn said(world: &World) -> Option<String> {
    let known = world.get_resource::<Known>()?;
    if known.entries().is_empty() {
        return Some("the rules: none told yet".to_string());
    }
    let differ: Vec<String> = known.entries().iter().filter(|e| differs(e)).map(|e| format!("{} {} (game: {})", e.name, figure(&e.value), figure(&e.game))).collect();
    Some(format!(
        "the rules: {} names told, {} ideas not yet found; open: {}; differing from the game's: {}",
        known.entries().len(),
        known.unfound().len(),
        known.open().unwrap_or("none"),
        if differ.is_empty() { "none".to_string() } else { differ.join(", ") }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use isostate_core::params::game_params;
    use isostate_core::rulebook::rulebook;

    #[test]
    fn the_rules_are_grouped_by_name_with_their_switches_and_written_plainly() {
        let book = rulebook(&game_params());
        let rules = by_rule(&book);
        assert_eq!(rules.first().map(|r| r.0), Some("general"));
        let money = rules.iter().find(|r| r.0 == "money").unwrap();
        assert_eq!(money.1.map(|e| e.name.as_str()), Some("money.true_total"));
        assert_eq!(money.2.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), ["money.growth"]);
        // every rule with a switch has it as its switch, none twice
        let switches: usize = rules.iter().filter(|r| r.1.is_some()).count();
        assert_eq!(switches, isostate_core::edits::SWITCHES.len());
        assert!(rules.iter().all(|r| !r.2.iter().any(|e| matches!(e.kind, Kind::Switch { .. }))));
        // the game's own rules differ from nothing
        assert!(!book.iter().any(differs));
        assert_eq!(figure(&serde_json::json!(0.0175)), "0.0175");
        assert_eq!(figure(&serde_json::json!(0.0175 * 1.5)), "0.02625");
        assert_eq!(figure(&serde_json::json!(32)), "32");
        assert_eq!(figure(&serde_json::json!(true)), "on");
        assert_eq!(figure(&serde_json::json!([1500.0, 9000.0])), "1500, 9000");
        assert!(same(&serde_json::json!(1), &serde_json::json!(1.0)));
        let land = book.iter().find(|e| e.name == "land.enabled").unwrap();
        // (on under the game's rules since 19 November 2026: docs/rules/land.md, "The switch")
        assert_eq!(switch_state(land), "on: fixed when the island is made");
        let ration = book.iter().find(|e| e.name == "hands.ration").unwrap();
        assert_eq!(switch_state(ration), "on");
    }
}
