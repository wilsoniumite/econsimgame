//! The sandbox's Hand tab (`docs/designs/sandbox.md`, 4.6): everything done by
//! hand, newest first, each with its month and its line, read from the
//! island's news of kind "hand". A row about a place is a button that takes
//! the view there and opens the note about it, as a works in the holdings list
//! does (`game::goto_clicks`), without counting as a click on the map, so that
//! a tool in hand is not used by it. An edit's news comes with the next month
//! or year; until then the line of its answer heads the list (`Pending`). At
//! the foot stands the ledger (4.6): the island's group "By hand" as the
//! simulation tells it (`inspect`), asked while the tab is open.
//!
//! `panel` is called by `ui::panel_rebuild` with `HandParams`, which this file
//! owns: what the tab reads is added there and does not come into ui.rs.

// see the note at the top of ui.rs
#![allow(deprecated)]

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use isostate_core::inspect::{Thing, Told};
use isostate_core::save::Order;

use crate::family::chip_button;
use crate::game::Game;
use crate::live::{Cmd, Live};
use crate::replay::{Event, Replay};
use crate::sandbox::HAND_INK;
use crate::ui::{text, UiState, INK, MUTED, PAPER_2, RULE};
use crate::{clock, world, Data, Orbit, Pick, Picked};

/// What the Hand tab reads beyond the run and the year on the clock.
#[derive(SystemParam)]
pub struct HandParams<'w> {
    /// present in a sandbox; the tab is on the bar only there
    pub sandbox: Option<Res<'w, crate::sandbox::Sandbox>>,
    /// edits done whose news has not come yet
    pub pending: Option<Res<'w, Pending>>,
    /// what the simulation has said of things, among them the island, whose
    /// group "By hand" is the tab's foot (`inspect`, the hand's ledger)
    pub told: Option<Res<'w, crate::inspector::Answers>>,
    /// what waits for the year's turn (`Live::standing`), whose edits head the tab
    pub live: Option<Res<'w, Live>>,
}

/// An edit that waits taken back from the Hand tab: the words of its row,
/// and how many answers the game had had when it was sent (`Game::answered`),
/// until its answer comes.
#[derive(Resource, Default)]
pub struct TakingBack(Option<(String, u64)>);

/// The Take back chip of a waiting edit: its place among everything that
/// waits, the family's orders included (`Cmd::Cancel` takes that place).
#[derive(Component)]
pub struct TakeBack(pub usize);

/// Edits the simulation has carried out whose news has not come yet. An
/// edit's item of news reaches the viewer with the next month or year, but
/// its row belongs at the head of the tab at once (section 4.4), so the line
/// of its answer is listed until the news has it.
#[derive(Resource, Default)]
pub struct Pending {
    /// the edit's date (as the news is dated), its line, and how many items of
    /// the hand's news there will be once its own has come
    rows: Vec<(f32, String, usize)>,
    /// an order was out as the frame's messages were about to be read
    out: bool,
    /// the last answer taken
    last: Option<String>,
}

impl Pending {
    /// An edit's line, dated `when`, listed until its news is among the
    /// `in_news` items of the hand's news there are now and one more.
    pub fn add(&mut self, when: f32, line: &str, in_news: usize) {
        let n = in_news + self.rows.len() + 1;
        self.rows.push((when, line.to_string(), n));
    }
}

/// A row of the Hand tab about a place: the square the news is about.
#[derive(Component)]
pub struct HandRow(pub usize);

pub fn plugin(app: &mut App) {
    app.init_resource::<Pending>().init_resource::<TakingBack>().add_systems(Update, hand_clicks).add_systems(
        Update,
        // (in the frame the answer comes, as the tray reads it, before a question about the
        // clicked square puts the note away)
        (
            order_out.before(crate::game::pump),
            catch_answers.after(crate::game::pump).before(crate::game::ask_site),
            ask_island.after(crate::game::pump),
            (take_back, taken_back, waiting_changed).chain().after(crate::game::pump).before(crate::game::ask_site),
        )
            .run_if(resource_exists::<crate::sandbox::Sandbox>)
            .run_if(resource_exists::<Live>),
    );
}

/// The words of a waiting edit: what `live::order_words` says of it, with no
/// "By hand" before it ("a works (brickworks) at 31, 40, in Melbourne's
/// district", "Fairfax Mill closed"). Its row has them with a capital.
pub fn waiting_words(order: &Order, r: &Replay) -> String {
    crate::live::order_words(order, r, None).trim_start_matches("By hand: ").to_string()
}

/// A line begun with a capital.
fn capital(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or_else(String::new, |first| first.to_uppercase().chain(c).collect())
}

/// The Take back chip of a waiting edit: under the tray's guards (it works
/// only in the present, one thing at a time), the edit is taken back by its
/// place among everything that waits.
#[allow(clippy::too_many_arguments)]
fn take_back(
    q: Query<(&Interaction, &TakeBack), Changed<Interaction>>,
    live: Res<Live>,
    mut game: ResMut<Game>,
    sb: Res<crate::sandbox::Sandbox>,
    tl: Res<crate::Timeline>,
    data: Res<Data>,
    mut taking: ResMut<TakingBack>,
    mut toasts: ResMut<crate::Toasts>,
    time: Res<Time>,
) {
    for (i, chip) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        let say = |toasts: &mut crate::Toasts, line: &str| toasts.say("orders_refused", line, "By hand", time.elapsed_secs());
        if (tl.t.floor() as usize) < data.latest() || data.r.filled >= data.r.ny {
            say(&mut toasts, crate::sandbox::PAST);
            continue;
        }
        if game.busy || live.turning() || sb.running() {
            say(&mut toasts, crate::sandbox::BUSY);
            continue;
        }
        let Some(order) = live.standing.waiting.get(chip.0) else { continue };
        taking.0 = Some((waiting_words(order, &data.r), game.answered));
        live.send(Cmd::Cancel(chip.0));
        game.busy = true;
        game.note = None;
    }
}

/// The answer to a Take back: "Taken back: ..." in the hand's ink, or why not.
fn taken_back(mut taking: ResMut<TakingBack>, game: Res<Game>, data: Res<Data>, live: Res<Live>, mut toasts: ResMut<crate::Toasts>, time: Res<Time>) {
    let Some((words, sent_at)) = &taking.0 else { return };
    if game.busy || game.answered <= *sent_at {
        return;
    }
    let when = clock::month_year(clock::date(data.r.y0, clock::at(data.latest(), live.months.min(11))));
    let (kind, line) = match &game.note {
        Some((false, why)) => ("orders_refused", why.clone()),
        _ => ("hand", format!("Taken back: {words}")),
    };
    toasts.say(kind, &line, &when, time.elapsed_secs());
    println!("sandbox: {when}: {line}");
    taking.0 = None;
}

/// The tab is drawn again when what waits for the year's turn changes.
fn waiting_changed(live: Res<Live>, mut ui: ResMut<UiState>, mut seen: Local<Vec<Order>>) {
    if *seen != live.standing.waiting {
        *seen = live.standing.waiting.clone();
        ui.dirty += 1;
    }
}

/// The foot's ledger is the island's group "By hand", asked of the simulation
/// while the tab is open: when it opens, and again when a month, a year or an
/// order's answer comes. The tab is drawn again when the answer changes.
#[allow(clippy::too_many_arguments)]
fn ask_island(
    tl: Res<crate::Timeline>,
    data: Res<Data>,
    live: Res<Live>,
    game: Res<Game>,
    mut answers: ResMut<crate::inspector::Answers>,
    mut ui: ResMut<UiState>,
    mut seen: Local<Option<((u64, usize, u64), std::time::Instant)>>,
    mut drawn: Local<Option<Result<Told, String>>>,
) {
    if ui.tab != Some(crate::ui::Tab::Hand) {
        // asked afresh when the tab opens again
        *seen = None;
        return;
    }
    // (while the clock runs, no more often than the inspector asks: inspector::RUNNING_EVERY)
    let key = crate::inspector::news_key(&ui, &data, &game);
    if seen.is_none_or(|s| s.0 != key) && !live.turning() && crate::inspector::at_present(&tl, &data, &live) && !crate::inspector::too_soon(&tl, seen.map(|s| s.1)) {
        *seen = Some((key, std::time::Instant::now()));
        answers.ask(&live, Thing::Island);
    }
    let now = answers.get(Thing::Island);
    if now != drawn.as_ref() {
        *drawn = now.cloned();
        ui.dirty += 1;
    }
}

/// Whether an order is out before the frame's messages are read: an answer
/// can come in the very next frame, so it is looked at on either side of them.
fn order_out(mut pending: ResMut<Pending>, game: Res<Game>) {
    pending.out = game.busy;
}

/// An edit's answer: its line is listed until its news comes, and the tab is
/// drawn again either way. An answer is the game's note when an order that
/// was out comes back done, and an edit's line begins "By hand".
fn catch_answers(mut pending: ResMut<Pending>, game: Res<Game>, data: Res<Data>, live: Res<Live>, mut ui: ResMut<UiState>) {
    let in_news = done_by_hand(&data.r).count();
    let mut changed = false;
    if pending.rows.iter().any(|row| in_news >= row.2) {
        pending.rows.retain(|row| in_news < row.2);
        changed = true;
    }
    // (an answer is taken when an order was out before the messages were read, or when its line
    // is not the last one taken, which catches one sent and answered within a single frame)
    if let (false, Some((true, line))) = (game.busy, &game.note) {
        if line.starts_with("By hand") && (pending.out || pending.last.as_ref() != Some(line)) {
            pending.last = Some(line.clone());
            // dated as the core dates it: the middle of the month the simulation has reached
            let when = clock::news_time(data.r.y0, clock::at(data.latest(), live.months.min(11))) + 0.5 / 12.0;
            pending.add(when, line, in_news);
            changed = true;
        }
        // (taken once)
        pending.out = false;
    }
    if changed {
        ui.dirty += 1;
    }
}

/// The news the hand left, newest first.
pub fn done_by_hand(r: &Replay) -> impl Iterator<Item = &Event> {
    r.header.events.iter().rev().filter(|e| e.kind == "hand")
}

/// What it says when the hand has done nothing yet.
pub const NOTHING_YET: &str = "Nothing has been done by hand yet: take a tool from the tray under the clock.";
/// And in a game that is not a sandbox (a picture asked for the tab by name).
const NO_HAND: &str = "Nothing is done by hand in the family game: a sandbox, which a plain start opens, has the tray of tools.";

/// The Hand tab's body.
pub fn panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, r: &Replay, _k: usize, hand: &HandParams) {
    let mut shown = 0;
    // at its head, the edits that wait for the year's turn, each with its Take back chip
    // (`i` is its place among everything that waits, the family's orders included)
    let waiting: Vec<(usize, &Order)> = match (&hand.live, &hand.sandbox) {
        (Some(live), Some(_)) => live.standing.waiting.iter().enumerate().filter(|(_, o)| matches!(o, Order::ByHand { .. })).collect(),
        _ => Vec::new(),
    };
    if !waiting.is_empty() {
        let (_, turn) = crate::game::turn_words(r.y0, r.filled.max(1) - 1);
        p.spawn(text(f, "Waits for the year's turn", 15.0, HAND_INK));
        for (i, order) in waiting {
            shown += 1;
            p.spawn((
                Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(8.0), padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)), margin: UiRect::vertical(Val::Px(2.0)), border: UiRect::left(Val::Px(3.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() },
                BackgroundColor(PAPER_2),
                BorderColor::all(HAND_INK),
            ))
            .with_children(|row| {
                row.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 1.0, ..default() }).with_children(|col| {
                    col.spawn(text(f, &capital(&waiting_words(order, r)), 14.0, INK));
                    col.spawn(text(f, &format!("In {turn}"), 12.0, MUTED));
                });
                // (the chip keeps its width: the words beside it wrap instead)
                row.spawn(Node { flex_shrink: 0.0, ..default() }).with_children(|c| chip_button(c, f, "Take back", false, HAND_INK, TakeBack(i)));
            });
        }
        p.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(8.0)), ..default() }, BackgroundColor(RULE)));
    }
    // edits done whose news is still to come, the newest first (they are newer than any news)
    for (when, line, _) in hand.pending.iter().flat_map(|p| p.rows.iter().rev()) {
        shown += 1;
        p.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(Val::Px(6.0), Val::Px(5.0)), border: UiRect::bottom(Val::Px(1.0)), ..default() }, BorderColor::all(RULE))).with_children(|col| {
            col.spawn(text(f, &clock::news_month(*when), 12.0, HAND_INK));
            col.spawn(text(f, line, 15.0, INK));
        });
    }
    for e in done_by_hand(r) {
        shown += 1;
        let at = e.cell.and_then(|c| usize::try_from(c).ok()).filter(|&c| c < r.n);
        let mut row = p.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(Val::Px(6.0), Val::Px(5.0)), border: UiRect::bottom(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, BorderColor::all(RULE), BackgroundColor(Color::NONE)));
        if let Some(c) = at {
            row.insert((Interaction::default(), FocusPolicy::Block, HandRow(c)));
        }
        row.with_children(|col| {
            col.spawn(text(f, &clock::news_month(e.year), 12.0, HAND_INK));
            col.spawn(text(f, &e.title, 15.0, INK));
        });
    }
    if shown == 0 {
        p.spawn(text(f, if hand.sandbox.is_some() { NOTHING_YET } else { NO_HAND }, 14.0, MUTED));
    }
    // the foot: the ledger, as the simulation tells it of the island
    let ledger = hand.told.as_ref().and_then(|t| t.get(Thing::Island)).and_then(|t| t.as_ref().ok()).filter(|_| hand.sandbox.is_some());
    if let Some(told) = ledger {
        if let Some(g) = told.groups.iter().find(|g| g.name == "By hand") {
            p.spawn((Node { height: Val::Px(1.0), margin: UiRect::top(Val::Px(10.0)), ..default() }, BackgroundColor(RULE)));
            crate::inspector::group(p, f, &format!("The ledger, as it stands in {}", told.as_of), &g.rows);
        }
    }
}

/// The town whose node is this square, if any; else the square itself.
fn pick_at(r: &Replay, cell: usize) -> Pick {
    match r.header.towns.iter().position(|t| t.node == cell) {
        Some(t) => Pick::Town(t),
        None => Pick::Cell(cell),
    }
}

/// A row about a place takes the camera there and opens the note about it.
/// `Picked::stamp` is left as it is: this is not a click on the map, and a
/// tool in hand is not used by it.
fn hand_clicks(mut q: Query<(&Interaction, &HandRow, &mut BackgroundColor), Changed<Interaction>>, data: Res<Data>, mut o: ResMut<Orbit>, mut picked: ResMut<Picked>) {
    for (i, row, mut bg) in &mut q {
        bg.0 = match i {
            Interaction::None => Color::NONE,
            _ => PAPER_2,
        };
        if *i == Interaction::Pressed {
            if let Some(&(x, z)) = data.cen.get(row.0) {
                o.target = Vec3::new(x, world::height_at(&data.g, x, z).max(0.0), z);
                o.dist = o.dist.min(9.0);
                picked.what = Some(pick_at(&data.r, row.0));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hand_tab_lists_the_hands_news_newest_first_and_finds_its_town() {
        let mut r = crate::replay::tiny(1750);
        let ev = |year: f32, kind: &str, title: &str, cell: Option<i64>| Event { year, kind: kind.into(), weight: 0.5, title: title.into(), text: String::new(), cell, family: None, kept: None, seq: 0 };
        r.header.events = vec![ev(1760.2, "hand", "first", Some(0)), ev(1760.4, "harvest", "not by hand", None), ev(1761.5, "hand", "second", None)];
        let titles: Vec<&str> = done_by_hand(&r).map(|e| e.title.as_str()).collect();
        assert_eq!(titles, ["second", "first"]);
        assert_eq!(clock::news_month(1761.5), "July 1761");
        // a square that is a town's node is the town; another is the square
        if let Some(t) = r.header.towns.first() {
            let node = t.node;
            assert!(matches!(pick_at(&r, node), Pick::Town(0)));
        }
        assert!(matches!(pick_at(&r, usize::MAX), Pick::Cell(_)));
    }
}
