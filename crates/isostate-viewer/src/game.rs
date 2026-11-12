//! Playing a live game: the player's orders from the map and the panels, the
//! decisions that wait for the player, and the families' banners, fences and
//! seats drawn on the island.

// see the note at the top of ui.rs
#![allow(deprecated)]

use std::collections::{BTreeSet, VecDeque};

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use isostate_core::families::{Group, Post, Posting};
use isostate_core::{Choice, Decision, FamilyReport};

use crate::clock;
use crate::family::{self, short_group, trade, CancelButton, FamilyRow, FamilyView, GotoFirm, GotoTown, MemberRow, OpenDecision, PostButton, RivalRow, StanceButton};
use crate::live::{self, Cmd, Live, Msg, SiteInfo, Standing, PLAYER};
use crate::look::{Look, TheLook};
use crate::meshkit::{hex, lerp, Builder};
use crate::replay::{self, Replay};
use crate::ui::{money, text, UiState, BTN, INK, MUTED, PAPER_2, RULE};
use crate::{scatter, world, Data, InspectActions, Orbit, Pick, Picked, Timeline, UiFont};

#[derive(Resource, Default)]
pub struct Game {
    /// the answer about the picked cell, once it has come
    pub site: Option<SiteInfo>,
    /// the world has moved on since that answer: it stays on screen, but
    /// nothing can be ordered from it until the new answer comes
    site_stale: bool,
    /// the cell a question is out about
    pub asked: Option<usize>,
    /// how the last order went
    pub note: Option<(bool, String)>,
    /// an order is out
    pub busy: bool,
    /// the player stepped forward: where the clock is to stand once the
    /// months up to there have come (the start of a month, or the year's turn)
    pub follow: Option<f32>,
    /// bumped whenever the order panel must be redrawn
    pub version: u64,
    /// the last order sent, until its answer comes
    last_order: Option<Action>,
    /// squares where the player's new works are to stand: ordered and
    /// waiting for the year's turn, or begun at the last one
    pending: Vec<usize>,
    /// months that have come and are not yet in the run's tables (`months_into_run`)
    held: Vec<Box<isostate_core::LiveMonth>>,
    /// the end of the game has been shown
    ended: bool,
    /// the card that ends the game is up
    end_open: bool,
    drawn: Option<(Option<Pick>, usize, u64, Now, (bool, bool))>,
    marks: Option<(usize, usize, u64, u64)>,
    /// decisions already put in front of the player, so each opens by itself only once
    seen: BTreeSet<u64>,
    /// decisions to show, one after another
    queue: VecDeque<u64>,
    /// the decision whose card is open
    pub open: Option<u64>,
    /// the welcome card is up: decisions wait behind it
    welcoming: bool,
    card_drawn: Option<(Option<u64>, u64, bool)>,
}

impl Game {
    /// A card that wants an answer is up: the welcome, a decision or the end
    /// of the game. The clock stands still under it.
    pub fn modal(&self) -> bool {
        self.welcoming || self.open.is_some() || self.end_open
    }

    /// For the scripted runs: which cards are up and how the last order went.
    pub fn summary(&self) -> String {
        let card = match (self.welcoming, self.open) {
            (true, _) => "the welcome card is up".to_string(),
            (false, Some(id)) => format!("decision {id} is open"),
            (false, None) if self.end_open => "the end card is up".to_string(),
            (false, None) => "no card is open".to_string(),
        };
        let note = match &self.note {
            Some((ok, text)) => format!("; the last order {}: {text}", if *ok { "went through" } else { "was refused" }),
            None => String::new(),
        };
        format!("{card}{}{}{}{note}", if self.ended && !self.end_open { ", the end card has been shown" } else { "" }, if self.busy { ", an order is out" } else { "" }, if self.follow.is_some() { ", stepping on" } else { "" })
    }
}

/// Whether a card that wants an answer was up when this frame began. A key
/// that closes or answers the card is the card's alone: read here, not from
/// `Game`, it stays so whichever system runs first, so Space on the welcome
/// card cannot also start the clock.
#[derive(Resource, Default)]
pub struct CardOpen(pub bool);

fn note_cards(game: Res<Game>, mut open: ResMut<CardOpen>) {
    open.0 = game.modal();
}

/// The card shown when the century is over.
#[derive(Component)]
struct EndCard;

/// The card that opens a new game.
#[derive(Component)]
struct Welcome;

/// On the opening card of a game gone on with: carry on, or start again.
#[derive(Component)]
struct ContinueButton;
#[derive(Component)]
struct NewGameButton;
/// The small print under them, which also says if a new game could not be started.
#[derive(Component)]
struct WelcomeHint;

/// The card for a decision.
#[derive(Component)]
struct DecisionCard;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    Buy(usize),
    Found { method: usize, cell: usize },
    /// back to the present, where orders can be given
    Present,
    /// move the seat to a town (the present seat cancels a move)
    MoveSeat(usize),
}

#[derive(Component, Clone, Copy)]
pub struct ActionButton(pub Action);

/// One option of a decision.
#[derive(Component, Clone, Copy)]
pub struct DecideButton {
    pub id: u64,
    pub option: usize,
}

/// Close the decision card and decide later.
#[derive(Component)]
struct LaterButton;

/// Shows how many decisions wait, and opens them.
#[derive(Component)]
pub struct DecisionBadge;

#[derive(Resource)]
struct Marks(Handle<Mesh>);

pub fn plugin(app: &mut App) {
    app.init_resource::<Game>()
        .init_resource::<CardOpen>()
        .add_systems(First, note_cards)
        .add_systems(Startup, setup_marks.run_if(resource_exists::<Live>))
        .add_systems(PostStartup, welcome.run_if(resource_exists::<Live>))
        .add_systems(Update, (press_unattended, new_game_clicks, close_welcome).chain())
        .add_systems(
            Update,
            (
                pump,
                settle,
                drive,
                ask_site,
                action_clicks,
                decide_clicks,
                post_clicks,
                stance_clicks,
                cancel_clicks,
                panel_clicks,
                goto_clicks,
                // (before the keys step the clock, so a key that brings the clock to the present does not also play on from it)
                next_clicks.before(crate::time_input),
                step_note,
                decision_badge,
                decision_card,
                order_panel,
                owner_marks,
                end_card,
            )
                .chain()
                .run_if(resource_exists::<Live>),
        );
}

// ------------------------------------------------------------------ the families, as the panels see them

fn row_of(rep: &FamilyReport, history: Vec<f32>) -> FamilyRow {
    FamilyRow {
        name: rep.surname.clone(),
        seat: rep.seat,
        seat_name: rep.seat_name.clone(),
        colour: family::banner(rep.index),
        cash: rep.cash as f32,
        net_worth: rep.net_worth as f32,
        income: rep.last_year.earned() as f32,
        works: rep.firms.len(),
        land: rep.cells.len(),
        history,
        rep: rep.clone(),
    }
}

/// A new year's reports: every family's worth goes on its record.
pub fn year_families(view: &mut FamilyView, reports: &[FamilyReport], year: i32) {
    let old = std::mem::take(&mut view.families);
    view.families = reports
        .iter()
        .map(|rep| {
            let mut h = old.get(rep.index).map(|r| r.history.clone()).unwrap_or_default();
            h.push(rep.net_worth as f32);
            row_of(rep, h)
        })
        .collect();
    view.year = year;
    view.player = reports.iter().position(|r| r.player);
    view.live = true;
}

/// The player's family after an order, between years.
pub fn player_family(view: &mut FamilyView, rep: &FamilyReport) {
    if let Some(row) = view.families.get_mut(rep.index) {
        let h = std::mem::take(&mut row.history);
        *row = row_of(rep, h);
    }
}

/// A month's end: what each family has in hand. The rest of what is known of
/// a family is the year's.
pub fn month_cash(view: &mut FamilyView, cash: &[f64]) {
    for (row, &c) in view.families.iter_mut().zip(cash) {
        row.cash = c as f32;
        row.rep.cash = c;
    }
}

// ------------------------------------------------------------------ the simulation thread

/// Put any new decisions in the queue; with something to decide, the clock stops.
fn queue_new(game: &mut Game, fam: &FamilyView, tl: &mut Timeline) {
    let Some(me) = fam.player() else { return };
    let mut fresh = false;
    for d in &me.rep.pending {
        if game.seen.insert(d.id) {
            game.queue.push_back(d.id);
            fresh = true;
        }
    }
    if fresh {
        tl.playing = false;
    }
    // drop answered (or defaulted) decisions from the queue
    let live: BTreeSet<u64> = me.rep.pending.iter().map(|d| d.id).collect();
    game.queue.retain(|id| live.contains(id));
    if game.open.is_some_and(|id| !live.contains(&id)) {
        game.open = None;
    }
    if game.open.is_none() && !game.welcoming {
        game.open = game.queue.pop_front();
    }
}

/// Write the months that have come into the run's tables: their series and their news.
fn months_into_run(game: &mut Game, data: &mut Data, ui: &mut UiState) {
    if game.held.is_empty() {
        return;
    }
    let r = std::sync::Arc::make_mut(&mut data.r);
    for m in game.held.drain(..) {
        live::apply_month(r, &m);
    }
    // a Markets panel that goes by the month has new figures to show
    if ui.tab == Some(crate::ui::Tab::Markets) && ui.by_month {
        ui.dirty += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn pump(
    mut live: ResMut<Live>,
    mut data: ResMut<Data>,
    mut fam: ResMut<FamilyView>,
    mut game: ResMut<Game>,
    mut tl: ResMut<Timeline>,
    mut ui: ResMut<UiState>,
    mut months: ResMut<live::Months>,
    mut toasts: ResMut<crate::Toasts>,
    time: Res<Time>,
    shots: Option<Res<crate::ShotPlanArgs>>,
) {
    let _frames = crate::frames::span(crate::frames::Sys::Pump);
    // months that waited for a scenery build to let go of the run
    if !game.held.is_empty() && std::sync::Arc::strong_count(&data.r) == 1 {
        months_into_run(&mut game, &mut data, &mut ui);
    }
    while let Some(msg) = live.try_recv() {
        match msg {
            Msg::Month(m) => {
                months.add(&m);
                month_cash(&mut fam, &m.cash);
                // the month's series and news go into the run, which a scenery build may still
                // hold: the whole run would be copied for it, so the month waits until it has let go
                game.held.push(m);
                if std::sync::Arc::strong_count(&data.r) == 1 {
                    months_into_run(&mut game, &mut data, &mut ui);
                }
                live.month_came();
                // the player stepped a month on: the clock goes to where it was sent
                // (a step to the year's turn waits for the year)
                if let Some(to) = game.follow.filter(|&to| to <= clock::at(data.latest(), live.months.min(11)) + 1e-3) {
                    game.follow = None;
                    tl.t = to;
                }
                ui.months += 1;
                // a month stepped on with the clock stopped: the panels that say what the family has
                // in hand are drawn again (while the clock runs they wait for the year, or for it to stop)
                if !tl.playing {
                    game.version += 1;
                    ui.dirty += 1;
                }
            }
            Msg::Year(end) => {
                let y = &end.year;
                // the year's months first, whoever holds the run
                months_into_run(&mut game, &mut data, &mut ui);
                // copy-on-write: a scenery build may still hold the old run
                let Data { r, g, slots, .. } = &mut *data;
                // (task D) what a new year costs the frame it arrives in, with ISOSTATE_TIMING
                let (t0, held) = (std::time::Instant::now(), std::sync::Arc::strong_count(r) > 1);
                let r = std::sync::Arc::make_mut(r);
                let t1 = t0.elapsed().as_secs_f32();
                live::apply_year(r, y);
                let t2 = t0.elapsed().as_secs_f32();
                scatter::place_new_firms(r, g, std::sync::Arc::make_mut(slots));
                if std::env::var_os("ISOSTATE_TIMING").is_some() {
                    eprintln!(
                        "year {} arrives: {:.0} ms on the main thread ({:.0} taking the run{}, {:.0} writing the year in, {:.0} standing its new works)",
                        y.year,
                        t0.elapsed().as_secs_f32() * 1e3,
                        t1 * 1e3,
                        if held { ", copied because a scenery build holds it" } else { "" },
                        (t2 - t1) * 1e3,
                        (t0.elapsed().as_secs_f32() - t2) * 1e3
                    );
                }
                year_families(&mut fam, &y.families, y.year as i32);
                // the turn's orders have been carried out since the year's picture was taken
                if let Some(rep) = &end.family {
                    player_family(&mut fam, rep);
                }
                fam.prospects = end.prospects.clone();
                live.year_came();
                if tl.built == Some(y.index) {
                    tl.built = None;
                }
                if game.follow.take().is_some() {
                    tl.t = y.index as f32;
                }
                game.pending = end.standing.pegs.clone();
                live.standing = end.standing.clone();
                // what became of the orders that waited for this year's end: word to the family was
                // carried out as the year ended and works and land as the next began, so the line
                // over each says the turn, which is both
                for (ok, line) in &end.carried {
                    toasts.say(if *ok { "orders" } else { "orders_refused" }, line, "the year's turn", time.elapsed_secs());
                }
                // the world has moved on: ask about the picked square again
                game.asked = None;
                game.site_stale = true;
                game.note = end.carried.iter().rev().find(|c| !c.0).map(|c| (false, c.1.clone()));
                game.version += 1;
                ui.dirty += 1;
                if shots.is_none() || cards_in_shots() {
                    queue_new(&mut game, &fam, &mut tl);
                    if game.open.is_some() {
                        // show the year the decision belongs to
                        tl.t = tl.t.max(y.index as f32);
                    }
                }
            }
            Msg::Site(s) => {
                if game.asked == Some(s.cell) {
                    game.site = Some(s);
                    game.site_stale = false;
                    game.version += 1;
                }
            }
            Msg::Done { ok, text, family, standing } => {
                game.busy = false;
                game.pending = standing.pegs.clone();
                live.standing = standing;
                game.last_order = None;
                if !text.is_empty() || !ok {
                    game.note = Some((ok, text));
                }
                if let Some(rep) = family {
                    player_family(&mut fam, &rep);
                    ui.dirty += 1;
                    queue_new(&mut game, &fam, &mut tl);
                }
                // the order changed the square: ask again before offering more
                game.asked = None;
                game.site_stale = true;
                game.version += 1;
            }
            Msg::Failed(e) => {
                live.failed();
                game.follow = None;
                game.busy = false;
                game.note = Some((false, e));
                game.version += 1;
            }
            Msg::Unsaved(why) => live.unsaved = why,
        }
    }
}

/// Screenshots skip the welcome and the decisions unless asked for.
fn cards_in_shots() -> bool {
    std::env::var_os("ISOSTATE_WELCOME").is_some()
}

/// Where the year on the clock stands against the years worked out. The next
/// year is asked for a little before the clock needs it, so that the years
/// roll without a stop; until the clock gets there, the newest year the
/// player has been shown is still the present as they see it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Now {
    /// a year gone by, which the player has gone back to look at
    Past,
    /// the newest year shown, with the next worked out and about to show:
    /// the simulation has moved on, so an order would land a year late
    Turning,
    /// the newest year there is: orders land here
    Present,
}

/// `k` is the year on the clock, `latest` the newest worked out and `seen`
/// the newest the clock has shown, all as indices.
pub fn now_of(k: usize, latest: usize, seen: usize) -> Now {
    if k >= latest {
        Now::Present
    } else if k >= seen {
        Now::Turning
    } else {
        Now::Past
    }
}

fn now(tl: &Timeline, data: &Data, live: &Live) -> Now {
    now_of((tl.t.floor() as usize).min(data.r.ny - 1), data.latest(), live.seen)
}

/// Stopped on the newest year shown while the next is already worked out,
/// the clock moves on to it: the simulation cannot go back, so the present is
/// there, and that is where orders can be given. (The player who steps back a
/// year on purpose is in the past and stays there.) And a clock that has just
/// stopped has the months played so far saved, and the panels drawn again
/// with what the family has in hand now (its cash came month by month while
/// the clock ran, and only the top bar followed it).
fn settle(mut live: ResMut<Live>, mut tl: ResMut<Timeline>, data: Res<Data>, mut game: ResMut<Game>, mut ui: ResMut<UiState>, mut was_playing: Local<bool>) {
    if !tl.playing && now(&tl, &data, &live) == Now::Turning {
        tl.t = data.latest() as f32;
    }
    let k = (tl.t.floor() as usize).min(data.latest());
    if k > live.seen {
        live.seen = k;
    }
    if *was_playing && !tl.playing {
        live.send(Cmd::Keep);
        game.version += 1;
        ui.dirty += 1;
    }
    *was_playing = tl.playing;
}

/// Ask for the months of the year in progress in time for the clock to roll
/// through them: each as late as will still have it ready by the time the
/// clock comes to its end, and December, which brings the year's end with it,
/// that much earlier. So at a year's turn the simulation stands there, and an
/// order is carried out at once, until January is nearly over.
fn drive(mut live: ResMut<Live>, mut game: ResMut<Game>, mut tl: ResMut<Timeline>, data: Res<Data>, cards: Res<CardOpen>) {
    let r = &data.r;
    if r.filled >= r.ny {
        game.follow = None;
        return;
    }
    // nothing is worked out under a card that waits for the player
    if cards.0 || game.modal() {
        return;
    }
    let latest = data.latest();
    let want = if let Some(to) = game.follow {
        // the player stepped on: the months up to there, and no further
        let need = (((to - latest as f32) * 12.0).round() as usize).clamp(1, 12);
        if need < 12 && live.months >= need {
            // they are there already (the clock had been taken back within the year)
            game.follow = None;
            tl.t = to;
        }
        need
    } else if tl.playing {
        clock::months_due(tl.t - latest as f32, clock::lead_years(live.month_took, clock::MONTH_GUESS, tl.speed), clock::lead_years(live.turn_took, clock::TURN_GUESS, tl.speed))
    } else {
        0
    };
    while live.asked < want {
        live.step();
    }
}

fn present(tl: &Timeline, data: &Data) -> bool {
    tl.t.floor() as usize >= data.latest()
}

fn ask_site(picked: Res<Picked>, tl: Res<Timeline>, data: Res<Data>, live: Res<Live>, mut game: ResMut<Game>) {
    let Some(Pick::Cell(c)) = picked.what else {
        if game.asked.is_some() {
            game.asked = None;
            game.site = None;
        }
        return;
    };
    // (while the year turns the answer would be of a world about to change: it is asked for after)
    if present(&tl, &data) && !live.turning() && game.asked != Some(c) {
        game.asked = Some(c);
        // the last answer about this square stays up until the new one comes,
        // and so does word of how the last order about it went
        if game.site.as_ref().is_none_or(|s| s.cell != c) {
            game.site = None;
            game.note = None;
        }
        game.version += 1;
        live.send(Cmd::Site(c));
    }
}

// ------------------------------------------------------------------ orders

fn action_clicks(q: Query<(&Interaction, &ActionButton), Changed<Interaction>>, live: Res<Live>, mut game: ResMut<Game>, mut tl: ResMut<Timeline>, data: Res<Data>) {
    for (i, a) in &q {
        if *i != Interaction::Pressed || game.busy {
            continue;
        }
        // an order is given in the present: carried out at once at a year's turn, and kept
        // for the year's end after it. While the year turns it waits a moment to be given
        if a.0 != Action::Present && !(present(&tl, &data) && !live.turning()) {
            continue;
        }
        game.last_order = Some(a.0);
        match a.0 {
            Action::Buy(c) => {
                live.send(Cmd::BuyLand(c));
                game.busy = true;
            }
            Action::Found { method, cell } => {
                live.send(Cmd::Found { method, cell });
                game.busy = true;
            }
            Action::MoveSeat(t) => {
                live.send(Cmd::MoveSeat(t));
                game.busy = true;
            }
            Action::Present => {
                tl.playing = false;
                // the month the simulation has reached
                tl.t = clock::at(data.latest(), live.months.min(11));
            }
        }
        game.note = None;
        game.version += 1;
    }
}

fn decide_clicks(
    q: Query<(&Interaction, &DecideButton), Changed<Interaction>>,
    later: Query<&Interaction, (Changed<Interaction>, With<LaterButton>)>,
    open: Query<(&Interaction, &OpenDecision), Changed<Interaction>>,
    badge: Query<&Interaction, (Changed<Interaction>, With<DecisionBadge>)>,
    keys: Res<ButtonInput<KeyCode>>,
    live: Res<Live>,
    fam: Res<FamilyView>,
    mut game: ResMut<Game>,
) {
    for (i, d) in &q {
        // a year's end on its way takes the default before an answer sent now
        // could arrive, and the answer would be refused: the card says so and waits
        if *i == Interaction::Pressed && !game.busy && !live.turning() {
            live.send(Cmd::Decide { id: d.id, option: d.option });
            game.busy = true;
            game.open = None;
        }
    }
    let close = later.iter().any(|i| *i == Interaction::Pressed) || (game.open.is_some() && keys.just_pressed(KeyCode::Escape));
    if close {
        // everything waiting can wait: the badge brings it back
        game.open = None;
        game.queue.clear();
    }
    for (i, o) in &open {
        if *i == Interaction::Pressed {
            game.open = Some(o.0);
        }
    }
    if badge.iter().any(|i| *i == Interaction::Pressed) {
        game.open = fam.player().and_then(|me| me.rep.pending.first()).map(|d| d.id);
    }
}

fn post_clicks(q: Query<(&Interaction, &PostButton), Changed<Interaction>>, live: Res<Live>, mut game: ResMut<Game>, mut ui: ResMut<UiState>) {
    for (i, p) in &q {
        if *i == Interaction::Pressed && !game.busy {
            live.send(Cmd::Post { member: p.member, posting: p.posting });
            game.busy = true;
            game.note = None;
            game.version += 1;
            ui.expanded = None;
        }
    }
}

fn stance_clicks(q: Query<(&Interaction, &StanceButton), Changed<Interaction>>, live: Res<Live>, mut game: ResMut<Game>) {
    for (i, s) in &q {
        if *i == Interaction::Pressed && !game.busy {
            live.send(Cmd::Stance { group: s.group, town: s.town, stance: s.stance });
            game.busy = true;
            // as the other orders do: the last order's note goes, and the order panel is drawn again
            game.note = None;
            game.version += 1;
        }
    }
}

/// An order that waits for the year's end is taken back.
fn cancel_clicks(q: Query<(&Interaction, &CancelButton), Changed<Interaction>>, live: Res<Live>, mut game: ResMut<Game>) {
    for (i, c) in &q {
        // (with the year's end on its way the order would be carried out before this came, and there would be none to take back)
        if *i == Interaction::Pressed && !game.busy && !live.turning() {
            live.send(Cmd::Cancel(c.0));
            game.busy = true;
            game.note = None;
            game.version += 1;
        }
    }
}

/// Clicking a member shows where they could be posted, clicking a rival shows
/// more of it; clicking again hides it. A town named in a panel takes the
/// camera there.
#[allow(clippy::too_many_arguments)]
fn panel_clicks(
    members: Query<(&Interaction, &MemberRow), Changed<Interaction>>,
    rivals: Query<(&Interaction, &RivalRow), Changed<Interaction>>,
    towns: Query<(&Interaction, &GotoTown), Changed<Interaction>>,
    data: Res<Data>,
    mut ui: ResMut<UiState>,
    mut o: ResMut<Orbit>,
    mut picked: ResMut<Picked>,
) {
    for (i, m) in &members {
        if *i == Interaction::Pressed {
            ui.expanded = if ui.expanded == Some(m.0) { None } else { Some(m.0) };
            ui.dirty += 1;
        }
    }
    for (i, r) in &rivals {
        if *i == Interaction::Pressed {
            ui.rival = if ui.rival == Some(r.0) { None } else { Some(r.0) };
            ui.dirty += 1;
        }
    }
    for (i, t) in &towns {
        if *i == Interaction::Pressed {
            if let Some(town) = data.r.header.towns.get(t.0) {
                let (x, z) = data.cen[town.node];
                o.target = Vec3::new(x, world::height_at(&data.g, x, z).max(0.0), z);
                o.dist = o.dist.min(9.0);
                picked.what = Some(Pick::Town(t.0));
            }
        }
    }
}

/// Next month and To January: the clock stops, and steps on. In the past it
/// steps through what has been played; at the present the months are asked of
/// the simulation (`drive`) and the clock follows when they have come.
fn next_clicks(
    months: Query<&Interaction, (Changed<Interaction>, With<crate::ui::NextMonthButton>)>,
    years: Query<&Interaction, (Changed<Interaction>, With<crate::ui::YearEndButton>)>,
    keys: Res<ButtonInput<KeyCode>>,
    cards: Res<CardOpen>,
    mut game: ResMut<Game>,
    mut tl: ResMut<Timeline>,
    data: Res<Data>,
) {
    // the keys: Right (or the full stop) a month on, Page Up to the year's turn; in the
    // past they step through the years as they always did (`time_input`)
    let at_present = present(&tl, &data);
    let key = |k: KeyCode| !cards.0 && at_present && keys.just_pressed(k);
    let month = months.iter().any(|i| *i == Interaction::Pressed) || key(KeyCode::ArrowRight) || key(KeyCode::Period);
    let year = years.iter().any(|i| *i == Interaction::Pressed) || key(KeyCode::PageUp);
    if !month && !year {
        return;
    }
    tl.playing = false;
    let latest = data.latest();
    let to = if year { tl.t.floor() + 1.0 } else { clock::next_month(tl.t) };
    if !at_present {
        // the months of a year gone by are all there
        tl.t = to.min(latest as f32);
        game.follow = None;
    } else if data.r.filled < data.r.ny {
        game.follow = Some(to.min((latest + 1) as f32));
    }
}

/// The line under the date: what the clock is doing, and when an order given
/// now would be carried out.
fn step_note(live: Res<Live>, data: Res<Data>, tl: Res<Timeline>, mut q: Query<&mut Text, With<crate::ui::StepNote>>) {
    let r = &data.r;
    let when = now(&tl, &data, &live);
    let s = if live.unsaved.is_some() {
        // the terminal has the reason; here it must not pass unseen
        "the game could not be saved".to_string()
    } else if r.filled >= r.ny && when != Now::Past {
        "the last year is over".to_string()
    } else if tl.playing {
        // (the pace and nothing else while the clock runs: at a year a second the year is
        // turning for half of every second, and a line that said so would flicker)
        clock::speed_at(tl.speed).map_or(String::new(), |s| s.pace.to_string())
    } else if live.turning() {
        "the year is turning\u{2026}".to_string()
    } else if when == Now::Past {
        "looking back".to_string()
    } else if live.at_turn() {
        "the year's turn".to_string()
    } else {
        format!("orders wait for {}", turn_words(r.y0, data.latest()).1)
    };
    for mut t in &mut q {
        if t.0 != s {
            t.0 = s.clone();
        }
    }
}

/// When the year in progress ends, two ways: "the end of 1784", for word to
/// the family, and "January 1785", for works and land.
pub fn turn_words(y0: i32, latest: usize) -> (String, String) {
    let year = y0 + latest as i32 + 1;
    (format!("the end of {year}"), format!("January {}", year + 1))
}

/// A works in the holdings list takes the camera to it.
fn goto_clicks(mut q: Query<(&Interaction, &GotoFirm, &mut BackgroundColor), Changed<Interaction>>, data: Res<Data>, mut o: ResMut<Orbit>, mut picked: ResMut<Picked>) {
    for (i, gf, mut bg) in &mut q {
        bg.0 = match i {
            Interaction::None => Color::NONE,
            _ => PAPER_2,
        };
        if *i == Interaction::Pressed {
            if let Some(&(x, z, _)) = data.slots.firm_pos.get(gf.0) {
                o.target = Vec3::new(x, world::height_at(&data.g, x, z).max(0.0), z);
                o.dist = o.dist.min(6.0);
                picked.what = Some(Pick::Firm(gf.0));
            }
        }
    }
}

fn button(p: &mut ChildSpawnerCommands, f: &Handle<Font>, label: &str, action: Option<Action>, accent: Color) {
    let mut e = p.spawn((
        Node { padding: UiRect::axes(Val::Px(11.0), Val::Px(4.0)), border_radius: BorderRadius::all(Val::Px(9.0)), flex_shrink: 0.0, ..default() },
        BackgroundColor(if action.is_some() { accent } else { BTN }),
        Interaction::default(),
        FocusPolicy::Block,
    ));
    if let Some(a) = action {
        e.insert(ActionButton(a));
    }
    e.with_children(|b| {
        b.spawn(text(f, label, 13.0, if action.is_some() { Color::WHITE } else { MUTED }));
    });
}

fn family_name(fam: &FamilyView, i: usize) -> String {
    fam.families.get(i).map(|f| f.name.clone()).unwrap_or_else(|| format!("family {i}"))
}

/// What the player can do about the thing in the inspector.
#[allow(clippy::too_many_arguments)]
fn order_panel(
    mut commands: Commands,
    mut game: ResMut<Game>,
    picked: Res<Picked>,
    tl: Res<Timeline>,
    data: Res<Data>,
    fam: Res<FamilyView>,
    live: Res<Live>,
    font: Res<UiFont>,
    q: Query<(Entity, Option<&Children>), With<InspectActions>>,
) {
    let _frames = crate::frames::span(crate::frames::Sys::Orders);
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    let when = now(&tl, &data, &live);
    // (while the year turns the buttons go unlit, since a press would not be taken: the panel is drawn again for that)
    let key = (picked.what, k, game.version, when, (live.at_turn(), live.turning()));
    // (a family's cash comes every month; the panel is drawn again for it only when the year's report does)
    if game.drawn == Some(key) {
        return;
    }
    game.drawn = Some(key);
    let Ok((e, children)) = q.single() else { return };
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    let Some(what) = picked.what else { return };
    let f = font.0.clone();
    let r = &data.r;
    let Some(me) = fam.player() else { return };
    let accent = me.colour;
    // where the game stands: the month the simulation has reached, or the last year once all are played
    let present = if r.filled < r.ny { clock::month_year(clock::date(r.y0, clock::at(data.latest(), live.months.min(11)))) } else { format!("{}", r.y0 + data.latest() as i32) };
    // orders can be given in the present, but for the moment a year is turning
    let can = when == Now::Present && !live.turning() && !game.busy;
    // in the middle of a year an order waits for the year's end: say when that is
    let waits = (!live.at_turn()).then(|| turn_words(r.y0, data.latest()));
    let stands = &live.standing;
    commands.entity(e).with_children(|p| {
        let rule = |p: &mut ChildSpawnerCommands| {
            p.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(6.0)), ..default() }, BackgroundColor(RULE)));
        };
        match what {
            Pick::Firm(fi) => {
                let Some(firm) = r.header.firms.get(fi) else { return };
                rule(p);
                let who = match firm.owner {
                    Some(o) if o == PLAYER => "One of your family's works.".to_string(),
                    Some(o) => format!("Owned by the {} family.", family_name(&fam, o)),
                    None => "Owned by its partners, local people with a little capital.".to_string(),
                };
                p.spawn(text(&f, &who, 14.0, if firm.owner == Some(PLAYER) { accent } else { INK }));
                if firm.owner == Some(PLAYER) && me.rep.absentee.contains(&fi) {
                    p.spawn(text(&f, "Nobody of the family runs it, so a hired manager takes a fifth of its profit.", 12.5, MUTED));
                }
            }
            Pick::Town(t) => {
                rule(p);
                if when == Now::Past {
                    back_to_present(p, &f, r, k, &present, accent);
                    return;
                }
                town_orders(p, &f, &fam, me, r, t, can, waits.as_ref());
            }
            Pick::Cell(c) => {
                rule(p);
                if when == Now::Past {
                    back_to_present(p, &f, r, k, &present, accent);
                    return;
                }
                let Some(site) = game.site.as_ref().filter(|s| s.cell == c) else {
                    // while the year turns the land agent is asked once it has
                    let wait = if when == Now::Present && !live.turning() { "Asking the land agent\u{2026}" } else { "The year is turning\u{2026}" };
                    p.spawn(text(&f, wait, 13.5, MUTED));
                    return;
                };
                cell_orders(p, &f, me, r, site, can && !game.site_stale, waits.as_ref(), stands);
            }
        }
        if let Some((ok, note)) = &game.note {
            p.spawn((
                Node { margin: UiRect::top(Val::Px(8.0)), padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() },
                BackgroundColor(if *ok { Color::srgb(0.90, 0.95, 0.88) } else { Color::srgb(0.98, 0.90, 0.86) }),
            ))
            .with_children(|n| {
                n.spawn(text(&f, note, 13.5, INK));
            });
        } else if game.busy {
            p.spawn(text(&f, "Sending word\u{2026}", 13.0, MUTED));
        }
    });
}

fn back_to_present(p: &mut ChildSpawnerCommands, f: &Handle<Font>, r: &Replay, k: usize, present: &str, accent: Color) {
    p.spawn(text(f, &format!("This is how it stood in {}. Orders are given in the present, {}.", r.y0 + k as i32, present), 13.5, MUTED));
    p.spawn(Node { margin: UiRect::top(Val::Px(6.0)), ..default() }).with_children(|row| button(row, f, &format!("Back to {present}"), Some(Action::Present), accent));
}

fn town_name(r: &Replay, t: usize) -> String {
    r.header.towns.get(t).map(|x| x.name.clone()).unwrap_or_else(|| format!("town {t}"))
}

/// A town: the seat, the family's branches and people there, and who could
/// be posted to run a trade there.
fn town_orders(p: &mut ChildSpawnerCommands, f: &Handle<Font>, fam: &FamilyView, me: &FamilyRow, r: &Replay, t: usize, can: bool, waits: Option<&(String, String)>) {
    let rep = &me.rep;
    let accent = me.colour;
    let name = town_name(r, t);
    // word to the family given in the middle of a year is carried out at its end
    if let Some((end, _)) = waits {
        p.spawn(text(f, &format!("What you tell the family here takes effect at {end}."), 12.5, MUTED));
    }
    // the seat
    if t == rep.seat {
        p.spawn(text(f, &format!("The seat of the {} family.", rep.surname), 14.5, accent));
        p.spawn(text(f, "The head lives here and runs every trade the family knows here. Works are chosen and land bought directly only around the seat.", 12.5, MUTED));
        if let Some(next) = rep.next_seat {
            p.spawn(text(f, &format!("Moving to {} at the year end.", town_name(r, next)), 13.0, accent));
            p.spawn(Node { margin: UiRect::top(Val::Px(4.0)), ..default() }).with_children(|x| button(x, f, &format!("Stay at {name}"), can.then_some(Action::MoveSeat(t)), accent));
        }
    } else if rep.next_seat == Some(t) {
        p.spawn(text(f, &format!("The {} family's seat moves here at the year end.", rep.surname), 14.0, accent));
        p.spawn(Node { margin: UiRect::top(Val::Px(4.0)), ..default() }).with_children(|x| button(x, f, &format!("Stay at {}", rep.seat_name), can.then_some(Action::MoveSeat(rep.seat)), accent));
    } else {
        p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(10.0), padding: UiRect::vertical(Val::Px(2.0)), ..default() }).with_children(|x| {
            x.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 1.0, ..default() }).with_children(|col| {
                col.spawn(text(f, &format!("Move the seat to {name}?"), 14.0, INK));
                col.spawn(text(f, &format!("The head moves at the year end, and with the head the bonus and the family's direct choices. Now at {}.", rep.seat_name), 12.0, MUTED));
            });
            button(x, f, "Move", can.then_some(Action::MoveSeat(t)), accent);
        });
    }

    // the family here
    let here: Vec<&isostate_core::BranchReport> = rep.branches.iter().filter(|b| b.town == t).collect();
    let people: Vec<&isostate_core::MemberReport> = rep.members.iter().filter(|m| m.alive && m.town == t).collect();
    if !here.is_empty() || !people.is_empty() {
        p.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(6.0)), ..default() }, BackgroundColor(RULE)));
        for b in &here {
            let runner = b.runner.and_then(|i| rep.members.get(i)).map(|m| m.first.clone()).unwrap_or_else(|| "nobody".into());
            let size = if b.group == Group::Estates { format!("{} squares", b.cells) } else { format!("{} works", b.works.len()) };
            p.spawn(text(f, &format!("{} branch, run by {}: {}, {}.", b.group.label(), runner, size, stance_word(b.stance)), 13.0, INK));
        }
        let names: Vec<String> = people.iter().map(|m| format!("{} ({})", m.first, m.age)).collect();
        if !names.is_empty() {
            p.spawn(text(f, &format!("Living here: {}.", family::join_words(&names)), 12.5, MUTED));
        }
    }

    // run a trade here
    if t != rep.seat {
        let free: Vec<&isostate_core::MemberReport> = rep.members.iter().filter(|m| family::postable(m) && !(matches!(m.post, Post::Manager { .. }) && m.town == t)).collect();
        let open: Vec<Group> = rep.known.iter().copied().filter(|&g| !here.iter().any(|b| b.group == g && b.runner.is_some())).collect();
        if !open.is_empty() {
            p.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(6.0)), ..default() }, BackgroundColor(RULE)));
            p.spawn(text(f, "Post someone to run a trade here", 14.0, INK));
            if free.is_empty() {
                p.spawn(text(f, "Nobody of the family is free: everyone of age is the head, training, or already posted here.", 12.0, MUTED));
            }
            for g in open {
                let pr = fam.prospect(t, g).map(|p| family::earnings(p, g, &name)).unwrap_or_else(|| "nothing new would pay here".into());
                p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), padding: UiRect::vertical(Val::Px(3.0)), ..default() }).with_children(|col| {
                    col.spawn(text(f, &format!("{} \u{00b7} {}", short_group(g), pr), 13.0, INK));
                    if !free.is_empty() {
                        col.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(4.0), row_gap: Val::Px(4.0), ..default() }).with_children(|chips| {
                            for m in &free {
                                let own = if m.trade == Some(g) { "\u{2022} " } else { "" };
                                family::chip_button(chips, f, &format!("{}{}", own, m.first), false, accent, PostButton { member: m.index, posting: Posting::Manager { group: g, town: t } });
                            }
                        });
                    }
                });
            }
            p.spawn(text(f, "A dot marks someone apprenticed in that trade.", 11.5, MUTED));
        }
    }

    // the rivals here
    let others: Vec<String> = fam
        .families
        .iter()
        .filter(|fr| fr.rep.index != rep.index)
        .filter_map(|fr| {
            let seat = fr.rep.seat == t;
            let trades: Vec<String> = fr.rep.branches.iter().filter(|b| b.town == t).map(|b| trade(b.group)).collect();
            match (seat, trades.is_empty()) {
                (true, true) => Some(format!("the {}s' seat", fr.name)),
                (true, false) => Some(format!("the {}s' seat, with {}", fr.name, family::join_words(&trades))),
                (false, false) => Some(format!("the {}s in {}", fr.name, family::join_words(&trades))),
                _ => None,
            }
        })
        .collect();
    if !others.is_empty() {
        p.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(6.0)), ..default() }, BackgroundColor(RULE)));
        p.spawn(text(f, &format!("Also here: {}.", family::join_words(&others)), 12.5, MUTED));
    }
}

fn stance_word(s: isostate_core::Stance) -> &'static str {
    match s {
        isostate_core::Stance::Grow => "growing",
        isostate_core::Stance::Hold => "holding",
        isostate_core::Stance::WindDown => "winding down",
    }
}

/// A square of land: who holds it, buying it, and the works that could be built on it.
#[allow(clippy::too_many_arguments)]
fn cell_orders(p: &mut ChildSpawnerCommands, f: &Handle<Font>, me: &FamilyRow, r: &Replay, site: &SiteInfo, can: bool, waits: Option<&(String, String)>, stands: &Standing) {
    let rep = &me.rep;
    let accent = me.colour;
    let c = site.cell;
    // what the family can spend: its cash, less what the orders that wait will take
    let in_hand = me.cash as f64 - stands.promised;
    let land_waits = stands.waiting.iter().any(|o| matches!(o, isostate_core::save::Order::BuyLand { cell } if *cell == c));
    let works_waits = stands.waiting.iter().any(|o| matches!(o, isostate_core::save::Order::Found { cell, .. } if *cell == c));
    // works and land ordered in the middle of a year are begun and bought at its turn
    if let Some((_, turn)) = waits {
        let spoken = if stands.promised >= 0.5 { format!(" {} of the family's cash is already promised to orders that wait.", money(stands.promised as f32)) } else { String::new() };
        p.spawn(text(f, &format!("An order given now is carried out at the year's turn, in {turn}.{spoken}"), 12.5, MUTED));
        p.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(6.0)), ..default() }, BackgroundColor(RULE)));
    }
    let at_seat = site.town == Some(rep.seat);
    let rule = |p: &mut ChildSpawnerCommands| {
        p.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(6.0)), ..default() }, BackgroundColor(RULE)));
    };
    // the land
    let held = match site.owner {
        Some(o) if o == PLAYER => "Your family's land.".to_string(),
        Some(o) => format!("The {} family's land.", site.owner_name.clone().unwrap_or_else(|| format!("{o}"))),
        None => "Held by the gentry, who would sell.".to_string(),
    };
    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
        row.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 1.0, ..default() }).with_children(|col| {
            col.spawn(text(f, &held, 14.0, if site.owner == Some(PLAYER) { accent } else { INK }));
            if site.owner != Some(PLAYER) {
                col.spawn(text(f, &format!("{} for the freehold, 25 years' rent.", money(site.land_price as f32)), 12.5, MUTED));
                if let Err(why) = &site.land_allowed {
                    col.spawn(text(f, &format!("{}.", family::capitalised(why)), 12.0, MUTED));
                }
            }
        });
        if land_waits {
            row.spawn(text(f, "Ordered", 13.0, accent));
        } else if site.owner != Some(PLAYER) && site.land_allowed.is_ok() {
            let ok = in_hand >= site.land_price && can;
            button(row, f, "Buy", ok.then_some(Action::Buy(c)), accent);
        }
    });
    // works
    match &site.options {
        Err(why) => {
            rule(p);
            p.spawn(text(f, &format!("Nothing can be built here: {why}."), 13.5, MUTED));
        }
        Ok(opts) => {
            rule(p);
            if !at_seat {
                p.spawn(text(f, "Building here", 16.0, INK));
                p.spawn(text(f, &format!("Works are chosen directly only at the seat, {}. Elsewhere, post someone to run a trade in the town: a growing branch builds for itself.", rep.seat_name), 12.5, MUTED));
                return;
            }
            let thin = |s: &isostate_core::SiteSums| s.market_share > 0.25 || s.price_ratio > 3.0;
            let mut good: Vec<(usize, &isostate_core::SiteSums)> = opts.iter().filter(|o| o.allowed.is_ok()).filter_map(|o| o.sums.as_ref().ok().map(|s| (o.method, s))).collect();
            // deep markets first: a works that would flood its town's market, or sells
            // at a scarcity price, earns its promised return only until it does
            good.sort_by(|a, b| thin(a.1).cmp(&thin(b.1)).then(b.1.expected_return.total_cmp(&a.1.expected_return)));
            p.spawn(text(f, "Build here", 16.0, INK));
            if works_waits {
                p.spawn(text(f, "A works is ordered here, and waits for the year's turn.", 13.0, accent));
            }
            if good.is_empty() {
                p.spawn(text(f, "Nothing in the family's trades would pay here.", 13.0, MUTED));
            }
            for (m, s) in good.iter().take(5) {
                let afford = in_hand >= s.capital_cost && can && !works_waits;
                p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(10.0), padding: UiRect::vertical(Val::Px(3.0)), ..default() }).with_children(|row| {
                    row.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 1.0, row_gap: Val::Px(1.0), ..default() }).with_children(|col| {
                        col.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0), ..default() }).with_children(|head| {
                            head.spawn(text(f, &r.header.methods[*m].label, 14.5, INK));
                            if thin(s) {
                                head.spawn((Node { padding: UiRect::axes(Val::Px(6.0), Val::Px(0.0)), border_radius: BorderRadius::all(Val::Px(7.0)), ..default() }, BackgroundColor(Color::srgb(0.97, 0.89, 0.74))))
                                    .with_children(|tag| {
                                        tag.spawn(text(f, "thin market", 11.5, Color::srgb(0.55, 0.36, 0.12)));
                                    });
                            }
                        });
                        let ret = if s.expected_return > 0.0 { format!("about {:.0}% a year at today's prices", 100.0 * s.expected_return) } else { "would lose money at today's prices".to_string() };
                        let bonus = if s.bonus > 0.0 { format!(", {:.0}% cheaper run by the family", 100.0 * s.bonus) } else { String::new() };
                        col.spawn(text(f, &format!("{} to build, {}{}", money(s.capital_cost as f32), ret, bonus), 12.5, MUTED));
                        let town = r.header.towns.get(s.town).map(|t| t.name.as_str()).unwrap_or("its town");
                        let share = if s.market_share >= 0.995 { format!("more than {town} buys") } else { format!("{:.0}% of what {town} buys", 100.0 * s.market_share.max(0.0)) };
                        let price = if s.price_ratio > 1.5 { format!(", selling at {:.1} times the usual price", s.price_ratio) } else { String::new() };
                        col.spawn(text(f, &format!("Would make {share}{price}."), 12.5, if thin(s) { Color::srgb(0.55, 0.36, 0.12) } else { MUTED }));
                    });
                    button(row, f, "Build", afford.then_some(Action::Found { method: *m, cell: c }), accent);
                });
            }
            // what the family could build here if it knew the trade
            let mut unknown: Vec<Group> = opts.iter().filter(|o| o.sums.is_ok() && o.allowed.is_err()).filter_map(|o| o.group).filter(|g| !rep.known.contains(g)).collect();
            unknown.sort();
            unknown.dedup();
            if !unknown.is_empty() {
                let names: Vec<String> = unknown.iter().map(|&g| trade(g)).collect();
                p.spawn(text(f, &format!("Works in {} could be built here too, once the family knows the trade.", family::join_words(&names)), 12.0, MUTED));
            }
        }
    }
}

// ------------------------------------------------------------------ decisions

fn decision_badge(fam: Res<FamilyView>, mut q: Query<(&mut Node, &Children), With<DecisionBadge>>, mut texts: Query<&mut Text>) {
    let n = fam.player().map(|m| m.rep.pending.len()).unwrap_or(0);
    for (mut node, ch) in &mut q {
        let want = if n > 0 { Display::Flex } else { Display::None };
        if node.display != want {
            node.display = want;
        }
        let s = if n == 1 { "A decision is waiting".to_string() } else { format!("{} decisions are waiting", family::capitalised(&family::number_word(n))) };
        for c in ch.iter() {
            if let Ok(mut t) = texts.get_mut(c) {
                if t.0 != s {
                    t.0 = s.clone();
                }
            }
        }
    }
}

/// What choosing an option would mean, in a line.
fn option_note(d: &Decision, choice: Choice, fam: &FamilyView, me: &FamilyRow, r: &Replay) -> String {
    let rep = &me.rep;
    let earn = |t: usize, g: Group| -> String {
        let at = town_name(r, t);
        match fam.prospect(t, g) {
            Some(p) => family::earnings(p, g, &at),
            None => format!("nothing in it would pay at {at} just now"),
        }
    };
    // a training chosen between years runs to the end of the year its decision falls due, plus four
    let finish = d.due + isostate_core::families::TRAINING_YEARS;
    let who = d.member.and_then(|j| rep.members.get(j)).map(|m| m.first.clone()).unwrap_or_else(|| "they".into());
    match choice {
        Choice::Trade { group } => earn(rep.seat, group),
        Choice::Apprentice { group, town } => {
            let knows = if rep.known.contains(&group) { "a trade the family knows".to_string() } else { format!("the family learns {} when {who} finishes, in {finish}", trade(group)) };
            format!("{knows}; {}", earn(town, group))
        }
        Choice::University => format!("{who} comes back in {finish} much abler, and educated"),
        Choice::Head { member } => match rep.members.get(member) {
            Some(m) => family::post_words(m, rep),
            None => String::new(),
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn decision_card(mut commands: Commands, mut game: ResMut<Game>, fam: Res<FamilyView>, data: Res<Data>, live: Res<Live>, font: Res<UiFont>, cards: Query<Entity, With<DecisionCard>>) {
    // with the year's end on its way (the card opened from the badge while the
    // months ran) an answer cannot land in time: the options wait, unlit
    let turning = live.turning();
    let key = (game.open, game.version, turning);
    if game.card_drawn == Some(key) && !fam.is_changed() {
        return;
    }
    game.card_drawn = Some(key);
    for e in &cards {
        commands.entity(e).despawn();
    }
    let Some(id) = game.open else { return };
    let Some(me) = fam.player() else { return };
    let Some(d) = me.rep.pending.iter().find(|d| d.id == id) else {
        game.open = None;
        return;
    };
    let f = font.0.clone();
    let r = &data.r;
    let rep = &me.rep;
    let accent = me.colour;
    let member = d.member.and_then(|j| rep.members.get(j));
    let first = d.options.first().map(|o| o.choice);
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, left: Val::Px(0.0), right: Val::Px(0.0), top: Val::Px(0.0), bottom: Val::Px(0.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgba(0.18, 0.14, 0.10, 0.16)),
            GlobalZIndex(45),
            Interaction::default(),
            FocusPolicy::Block,
            DecisionCard,
        ))
        .with_children(|wrap| {
            wrap.spawn((
                Node { flex_direction: FlexDirection::Column, width: Val::Px(500.0), padding: UiRect::axes(Val::Px(28.0), Val::Px(22.0)), row_gap: Val::Px(6.0), border_radius: BorderRadius::all(Val::Px(20.0)), ..default() },
                BackgroundColor(crate::ui::PAPER),
                BoxShadow::new(Color::srgba(0.25, 0.18, 0.10, 0.28), Val::Px(0.0), Val::Px(6.0), Val::Px(0.0), Val::Px(22.0)),
                Interaction::default(),
                FocusPolicy::Block,
            ))
            .with_children(|c| {
                c.spawn(text(&f, &format!("{}  \u{00b7}  for the {} family", d.due - 1, rep.surname), 13.0, MUTED));
                c.spawn(text(&f, &d.prompt, 23.0, INK));
                // who and what it is about
                match (first, member) {
                    (Some(Choice::Trade { .. }), _) => {
                        c.spawn(text(&f, &format!("The family starts at {} with {} in hand and land around the town. Its first trade is the one it can build in from the start; apprentices can learn others later.", rep.seat_name, money(me.cash)), 14.0, INK));
                    }
                    (Some(Choice::Head { .. }), _) => {
                        c.spawn(text(&f, "The head lives at the seat and runs every trade there; the abler the head, the cheaper the family's works there are to run.", 14.0, INK));
                    }
                    (_, Some(m)) => {
                        c.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                            row.spawn(text(&f, &format!("{} is {}, at {}.", m.first, m.age, m.town_name), 14.0, INK));
                            family::pips(row, m.skill, accent);
                        });
                        c.spawn(text(&f, "Training takes four years. An apprentice learns a trade for the family and runs it best; university makes anyone abler.", 13.0, MUTED));
                    }
                    _ => {}
                }
                c.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(6.0)), ..default() }, BackgroundColor(RULE)));
                for (i, o) in d.options.iter().enumerate() {
                    let note = option_note(d, o.choice, &fam, me, r);
                    let mut option = c.spawn((
                        Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(Val::Px(12.0), Val::Px(7.0)), border_radius: BorderRadius::all(Val::Px(11.0)), ..default() },
                        BackgroundColor(BTN),
                        Interaction::default(),
                        FocusPolicy::Block,
                    ));
                    if !turning {
                        option.insert((crate::ui::Hoverable, DecideButton { id: d.id, option: i }));
                    }
                    option.with_children(|b| {
                        b.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() }).with_children(|head| {
                            head.spawn(text(&f, &o.label, 15.5, if turning { MUTED } else { INK }));
                            if i == d.default {
                                head.spawn((Node { padding: UiRect::axes(Val::Px(6.0), Val::Px(0.0)), border_radius: BorderRadius::all(Val::Px(7.0)), ..default() }, BackgroundColor(accent.with_alpha(0.15))))
                                    .with_children(|t| {
                                        t.spawn(text(&f, "default", 11.5, accent));
                                    });
                            }
                        });
                        if !note.is_empty() {
                            b.spawn(text(&f, &note, 12.5, MUTED));
                        }
                    });
                }
                c.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(12.0), margin: UiRect::top(Val::Px(8.0)), ..default() }).with_children(|row| {
                    let foot = if turning {
                        "The year is turning\u{2026}".to_string()
                    } else if live.at_turn() {
                        format!("Unanswered, the default is taken at the end of {}.", d.due)
                    } else {
                        // in the middle of a year an answer waits for its end, where the default would be taken
                        format!("An answer now takes effect at {}. Unanswered, the default is taken at the end of {}.", turn_words(r.y0, data.latest()).0, d.due)
                    };
                    row.spawn(text(&f, &foot, 12.5, MUTED));
                    row.spawn((
                        Node { padding: UiRect::axes(Val::Px(11.0), Val::Px(4.0)), border_radius: BorderRadius::all(Val::Px(9.0)), flex_shrink: 0.0, ..default() },
                        BackgroundColor(BTN),
                        Interaction::default(),
                        FocusPolicy::Block,
                        crate::ui::Hoverable,
                        LaterButton,
                    ))
                    .with_children(|b| {
                        b.spawn(text(&f, "Decide later", 13.0, INK));
                    });
                });
            });
        });
}

// ------------------------------------------------------------------ banners and fences

fn setup_marks(mut commands: Commands, look: Res<TheLook>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let h = meshes.add(crate::placeholder_mesh());
    let mat = crate::vertex_coloured(&mut materials, &look, look.surfaces.boats_and_marks);
    commands.spawn((Mesh3d(h.clone()), MeshMaterial3d(mat)));
    commands.insert_resource(Marks(h));
    commands.insert_resource(crate::fences::Lines::default());
}

fn banner_rgba(i: usize) -> [f32; 4] {
    hex(family::BANNERS[i % family::BANNERS.len()])
}

/// A pennant on the highest gable of every family works, a low fence round every
/// family's open land, and a tall flag on the market square of every family's seat.
#[allow(clippy::too_many_arguments)]
fn build_marks(r: &Replay, g: &world::Ground, slots: &scatter::Slots, cen: &[(f32, f32)], seats: &[(usize, usize, bool)], k: usize, lk: &Look, flags: &mut scatter::Flags, lines: &mut crate::fences::Lines) -> Builder {
    let mut b = Builder::default();
    let pole = *lk.marks.pole;
    for (i, &(fam, town, moving)) in seats.iter().enumerate() {
        let Some((at, radius, yaw)) = scatter::towns::square_of(r, cen, k, town) else { continue };
        // families sharing a seat stand their flags side by side along the square's west edge
        let along = seats[..i].iter().filter(|s| s.1 == town).count() as f32;
        let q = Quat::from_rotation_y(yaw);
        let o = q * Vec3::new(-radius * 0.62, 0.0, -0.035 + 0.045 * along);
        let (x, z) = (at.x + o.x, at.y + o.z);
        let base = Vec3::new(x, world::height_at(g, x, z).max(0.0), z);
        // a seat the family is moving to has a shorter pole and a paler flag until the year end
        let top = base + Vec3::Y * if moving { 0.13 } else { 0.21 };
        b.beam(base - Vec3::Y * 0.01, top + Vec3::Y * 0.01, 0.0055, pole);
        b.block(top + Vec3::Y * 0.01, 0.012, 0.012, 0.012, 0.0, *lk.marks.finial, Some(*lk.marks.finial), 0.0);
        let col = if moving { lerp(banner_rgba(fam), *lk.marks.cloth_pale, 0.5) } else { banner_rgba(fam) };
        // the flag flies with the same wind as the pennants
        let fly = Vec3::new(0.068, -0.006, 0.034);
        let (a, d) = (top, top - Vec3::Y * 0.048);
        for (p0, p1, p2) in [(a, d, d + fly), (a, d + fly, a + fly)] {
            b.tri(p0, p1, p2, col);
            b.tri(p0, p2, p1, col);
        }
    }
    for (f, firm) in r.header.firms.iter().enumerate() {
        let Some(o) = firm.owner else { continue };
        if replay::firm_k(r, f, k) <= 0.0 {
            continue;
        }
        // one flag on the works' highest gable, on a staff half the building's height: big
        // enough to tell the family by from close up, a fleck of its colour from play distance
        let Some(peak) = scatter::firm_flag(r, g, slots, f, k, lk, flags) else { continue };
        let tall = (peak.y - world::height_at(g, peak.x, peak.z)).max(0.0);
        let staff = (tall * 0.5).clamp(0.022, 0.04);
        let top = peak + Vec3::Y * staff;
        b.beam(peak - Vec3::Y * 0.004, top, staff * 0.1, pole);
        // the pennant streams downwind, the same way on every staff (and with the smoke)
        let fly = isostate_gen::works::WIND * (staff * 1.1) - Vec3::Y * (staff * 0.2);
        let col = banner_rgba(o);
        b.tri(top, top - Vec3::Y * (staff * 0.6), top + fly, col);
        b.tri(top, top + fly, top - Vec3::Y * (staff * 0.6), col);
    }
    // fences round the families' open land, along its hedge lines (fences.rs)
    world::timed("family fences", || crate::fences::family_fences(&mut b, r, g, k, lk, lines, &banner_rgba));
    b
}

/// Surveyors' pegs and a sign where a works was ordered this year.
fn stakes(b: &mut Builder, lk: &Look, g: &world::Ground, r: &Replay, cell: usize, colour: [f32; 4]) {
    let (cx, cz) = ((cell % r.w) as f32 + 0.5, (cell / r.w) as f32 + 0.5);
    let at = |x: f32, z: f32| Vec3::new(x, world::height_at(g, x, z).max(0.0), z);
    let peg = *lk.marks.peg;
    let c = [at(cx - 0.12, cz - 0.09), at(cx + 0.12, cz - 0.09), at(cx + 0.12, cz + 0.09), at(cx - 0.12, cz + 0.09)];
    for p in c {
        b.block(p, 0.01, 0.01, 0.03, 0.0, peg, Some(peg), 0.01);
    }
    for i in 0..4 {
        b.beam(c[i] + Vec3::Y * 0.022, c[(i + 1) % 4] + Vec3::Y * 0.022, 0.003, colour);
    }
    // a little board on a post, in the family colour
    let s = at(cx - 0.14, cz + 0.12);
    b.beam(s, s + Vec3::Y * 0.07, 0.006, *lk.marks.pole);
    b.block(s + Vec3::Y * 0.05, 0.05, 0.006, 0.03, 0.3, colour, Some(colour), 0.0);
}

#[allow(clippy::too_many_arguments)]
fn owner_marks(mut game: ResMut<Game>, tl: Res<Timeline>, data: Res<Data>, look: Res<TheLook>, fam: Res<FamilyView>, live: Res<Live>, marks: Option<Res<Marks>>, lines: Option<ResMut<crate::fences::Lines>>, mut meshes: ResMut<Assets<Mesh>>, mut flags: Local<scatter::Flags>) {
    let _frames = crate::frames::span(crate::frames::Sys::Marks);
    let (Some(marks), Some(mut lines)) = (marks, lines) else { return };
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    // the seats are known for the present only, so they are drawn there (and
    // while the year turns, so the flags do not blink once a year)
    let front = now(&tl, &data, &live) != Now::Past;
    let seats: Vec<(usize, usize, bool)> = if front {
        fam.families.iter().flat_map(|f| std::iter::once((f.rep.index, f.seat, false)).chain(f.rep.next_seat.map(|t| (f.rep.index, t, true)))).collect()
    } else {
        Vec::new()
    };
    let seat_key = seats.iter().fold(0u64, |h, &(a, b, m)| h.wrapping_mul(31).wrapping_add((a * 1000 + b * 2 + m as usize) as u64 + 1));
    let pegs = game.pending.iter().fold(game.pending.len() as u64, |h, &c| h.wrapping_mul(31).wrapping_add(c as u64 + 1));
    let key = (k, data.r.filled, pegs, seat_key);
    if game.marks == Some(key) {
        return;
    }
    game.marks = Some(key);
    let mut b = world::timed("year marks", || build_marks(&data.r, &data.g, &data.slots, &data.cen, &seats, k, &look, &mut flags, &mut lines));
    if front {
        for &c in &game.pending {
            stakes(&mut b, &look, &data.g, &data.r, c, banner_rgba(PLAYER));
        }
    }
    if let Some(mut m) = meshes.get_mut(&marks.0) {
        *m = if b.is_empty() { crate::placeholder_mesh() } else { crate::meshkit::into_mesh(b) };
    }
}

/// When the last year has been played: where the families finished.
#[allow(clippy::too_many_arguments)]
fn end_card(mut commands: Commands, mut game: ResMut<Game>, tl: Res<Timeline>, data: Res<Data>, fam: Res<FamilyView>, font: Res<UiFont>, keys: Res<ButtonInput<KeyCode>>, cards: Query<Entity, With<EndCard>>, shots: Option<Res<crate::ShotPlanArgs>>) {
    if keys.just_pressed(KeyCode::Escape) {
        for e in &cards {
            commands.entity(e).despawn();
        }
        game.end_open = false;
    }
    let r = &data.r;
    // a finished game gone on with: the opening card first, then this one
    if game.ended || game.welcoming || r.filled < r.ny || tl.t < (r.ny - 1) as f32 {
        return;
    }
    // screenshots skip it, as they do the welcome, unless asked for
    if shots.is_some() && !cards_in_shots() {
        return;
    }
    game.ended = true;
    game.end_open = true;
    let f = font.0.clone();
    let mut rank: Vec<&FamilyRow> = fam.families.iter().collect();
    rank.sort_by(|a, b| crate::ui::biggest_first(a.net_worth, b.net_worth));
    let me = fam.player();
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, left: Val::Px(0.0), right: Val::Px(0.0), top: Val::Px(0.0), bottom: Val::Px(0.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            GlobalZIndex(40),
            EndCard,
        ))
        .with_children(|wrap| {
            wrap.spawn((
                Node { flex_direction: FlexDirection::Column, width: Val::Px(440.0), padding: UiRect::axes(Val::Px(28.0), Val::Px(22.0)), row_gap: Val::Px(6.0), border_radius: BorderRadius::all(Val::Px(18.0)), ..default() },
                BackgroundColor(crate::ui::PAPER),
                BoxShadow::new(Color::srgba(0.25, 0.18, 0.10, 0.25), Val::Px(0.0), Val::Px(4.0), Val::Px(0.0), Val::Px(16.0)),
                Interaction::default(),
                FocusPolicy::Block,
            ))
            .with_children(|c| {
                c.spawn(text(&f, &format!("{}", r.y0 + r.ny as i32 - 1), 40.0, INK));
                c.spawn(text(&f, "The century is over.", 18.0, INK));
                if let Some(me) = me {
                    let place = rank.iter().position(|x| x.name == me.name).unwrap_or(0) + 1;
                    c.spawn(text(&f, &format!("The {} family finished {} of {}, worth {}.", me.name, place, rank.len(), money(me.net_worth)), 15.0, MUTED));
                }
                c.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(8.0)), ..default() }, BackgroundColor(RULE)));
                for (i, row) in rank.iter().enumerate() {
                    c.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|line| {
                        line.spawn(text(&f, &format!("{}.  {}", i + 1, row.name), 16.0, row.colour));
                        line.spawn(text(&f, &money(row.net_worth), 16.0, INK));
                    });
                }
                c.spawn((text(&f, "The timeline still takes you back through the years. Esc closes this.", 13.0, MUTED), Node { margin: UiRect::top(Val::Px(10.0)), ..default() }));
            });
        });
}

/// A new game opens on a card saying who the player is and how to begin; the
/// first decision waits behind it. A game gone on with from a save opens on
/// the same card saying how the family stands, with Continue and New game.
fn welcome(mut commands: Commands, fam: Res<FamilyView>, data: Res<Data>, live: Res<Live>, font: Res<UiFont>, mut game: ResMut<Game>, shots: Option<Res<crate::ShotPlanArgs>>) {
    // works ordered since the last year (in a game gone on with) keep their pegs
    game.pending = live.ordered.clone();
    // screenshots skip it unless asked for
    if shots.is_some() && !cards_in_shots() {
        return;
    }
    let Some(me) = fam.player() else { return };
    // the decisions already waiting open once the card is closed
    for d in &me.rep.pending {
        if game.seen.insert(d.id) {
            game.queue.push_back(d.id);
        }
    }
    // for screenshots of the first decision: no welcome
    if shots.is_some() && std::env::var("ISOSTATE_WELCOME").is_ok_and(|v| v == "decision") {
        game.open = game.queue.pop_front();
        return;
    }
    game.welcoming = true;
    let f = font.0.clone();
    // where the clock stands: the turn after the first year, or where a saved game left off
    // (a game that is over stands on its last year, which has no January after it)
    let over = data.r.filled >= data.r.ny;
    let year = if over { format!("{}", data.r.y0 + data.latest() as i32) } else { clock::month_year(clock::date(data.r.y0, clock::at(data.latest(), live.months.min(11)))) };
    let rivals = fam.families.len().saturating_sub(1);
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, left: Val::Px(0.0), right: Val::Px(0.0), top: Val::Px(0.0), bottom: Val::Px(0.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgba(0.18, 0.14, 0.10, 0.18)),
            GlobalZIndex(50),
            Interaction::default(),
            FocusPolicy::Block,
            Welcome,
        ))
        .with_children(|wrap| {
            wrap.spawn((
                Node { flex_direction: FlexDirection::Column, width: Val::Px(490.0), padding: UiRect::axes(Val::Px(30.0), Val::Px(24.0)), row_gap: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(20.0)), ..default() },
                BackgroundColor(crate::ui::PAPER),
                BoxShadow::new(Color::srgba(0.25, 0.18, 0.10, 0.28), Val::Px(0.0), Val::Px(6.0), Val::Px(0.0), Val::Px(22.0)),
            ))
            .with_children(|c| {
                c.spawn(text(&f, &format!("{year}"), 15.0, MUTED));
                c.spawn(text(&f, &format!("The {} family of {}", me.name, me.seat_name), 26.0, me.colour));
                let living = me.rep.members.iter().filter(|m| m.alive).count();
                if let Some(back) = &live.resumed {
                    welcome_back(c, &f, me, living, back, over, data.r.y0);
                    return;
                }
                c.spawn(text(
                    &f,
                    &format!(
                        "{} of the family, {} in hand and land around {}. {} other great families already hold land and works across the island. The century is yours to make something of.",
                        family::capitalised(&family::number_word(living)),
                        money(me.cash),
                        me.seat_name,
                        family::capitalised(&family::number_word(rivals))
                    ),
                    15.0,
                    INK,
                ));
                c.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(6.0)), ..default() }, BackgroundColor(RULE)));
                for line in [
                    "First choose the family's trade. At the seat you build works in it yourself.",
                    "Post grown members to run your trades in other towns: their branches grow on their own.",
                    "Children are trained at fourteen. An apprentice teaches the family a new trade.",
                    "Space starts and stops the clock. Next month plays a month; To January runs on to the year's turn.",
                    "What you order at the turn happens at once; later in the year it waits for the year's end. H shows every key.",
                ] {
                    c.spawn(Node { column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                        row.spawn((Node { width: Val::Px(7.0), height: Val::Px(7.0), margin: UiRect::top(Val::Px(7.0)), border_radius: BorderRadius::all(Val::Px(4.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(me.colour)));
                        row.spawn(text(&f, line, 14.5, INK));
                    });
                }
                // only said of a game that is being saved
                let begin = if live.kept { "Click anywhere to begin. The game saves itself as you play." } else { "Click anywhere to begin." };
                c.spawn((text(&f, begin, 13.0, MUTED), Node { margin: UiRect::top(Val::Px(8.0)), ..default() }));
            });
        });
}

/// A key that only goes with others, or belongs to the desktop. Alt on its
/// way to Alt-Tab, or the key that takes a picture of the screen, is not the
/// player beginning the game.
fn goes_with_others(k: &KeyCode) -> bool {
    use KeyCode::*;
    matches!(k, AltLeft | AltRight | ControlLeft | ControlRight | ShiftLeft | ShiftRight | SuperLeft | SuperRight | PrintScreen | CapsLock | NumLock | ScrollLock)
}

/// What a save that could not be played again as it was is owed in words:
/// that it is an earlier one than the last, how many orders were refused,
/// and that the island stands differently.
pub fn resumed_note(back: &live::Resumed) -> Option<String> {
    let all = back.orders + back.refused;
    let earlier = (back.passed_over > 0).then(|| format!("The last save could not be read, so this is an earlier one, from {}.", back.year));
    let rules = match (back.refused, back.drifted) {
        (0, false) => None,
        (0, true) => Some("The island's rules have changed since this game was saved, so the years have played out a little differently.".to_string()),
        (n, _) => {
            let which = match (n, all) {
                (_, 1) => "Your one order".to_string(),
                (n, all) if n == all => format!("All {} of your orders", family::number_word(all)),
                (n, all) => format!("{} of your {} orders", family::capitalised(&family::number_word(n)), family::number_word(all)),
            };
            Some(format!("The island's rules have changed since this game was saved. {which} could not be given again, so some things stand differently."))
        }
    };
    match (earlier, rules) {
        (Some(a), Some(b)) => Some(format!("{a} {b}")),
        (a, b) => a.or(b),
    }
}

/// The rest of the opening card for a game gone on with: how the family
/// stands, what could not be brought back, and the two ways on.
fn welcome_back(c: &mut ChildSpawnerCommands, f: &Handle<Font>, me: &FamilyRow, living: usize, back: &live::Resumed, over: bool, first_year: i32) {
    c.spawn(text(
        f,
        &format!("Welcome back. {} of the family, with {} in hand and worth {} in all.", family::capitalised(&family::number_word(living)), money(me.cash), money(me.net_worth)),
        15.0,
        INK,
    ));
    if let Some(note) = resumed_note(back) {
        c.spawn((
            Node { margin: UiRect::top(Val::Px(4.0)), padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() },
            BackgroundColor(Color::srgb(0.97, 0.89, 0.74)),
        ))
        .with_children(|n| {
            n.spawn(text(f, &note, 13.5, Color::srgb(0.45, 0.29, 0.10)));
        });
    }
    c.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(6.0)), ..default() }, BackgroundColor(RULE)));
    c.spawn(Node { column_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }).with_children(|row| {
        row.spawn((
            Node { padding: UiRect::axes(Val::Px(16.0), Val::Px(7.0)), border_radius: BorderRadius::all(Val::Px(11.0)), ..default() },
            BackgroundColor(me.colour),
            Interaction::default(),
            FocusPolicy::Block,
            ContinueButton,
        ))
        .with_children(|b| {
            b.spawn(text(f, &format!("Continue from {}", back.date(over)), 15.5, Color::WHITE));
        });
        row.spawn((
            Node { padding: UiRect::axes(Val::Px(16.0), Val::Px(7.0)), border_radius: BorderRadius::all(Val::Px(11.0)), ..default() },
            BackgroundColor(BTN),
            Interaction::default(),
            FocusPolicy::Block,
            crate::ui::Hoverable,
            NewGameButton,
        ))
        .with_children(|b| {
            b.spawn(text(f, "New game", 15.5, INK));
        });
    });
    c.spawn((
        text(f, &format!("A new game begins again in {first_year} and leaves this one saved beside it."), 13.0, MUTED),
        Node { margin: UiRect::top(Val::Px(6.0)), ..default() },
        WelcomeHint,
    ));
}

/// Nobody at the pointer: ISOSTATE_WELCOME=continue or =new presses that
/// button of the opening card, once, so that what the buttons do can be
/// checked from the terminal (in a screenshot run, or with a window).
fn press_unattended(
    mut pressed: Local<bool>,
    mut go_on: Query<&mut Interaction, (With<ContinueButton>, Without<NewGameButton>)>,
    mut new_game: Query<&mut Interaction, (With<NewGameButton>, Without<ContinueButton>)>,
) {
    if *pressed {
        return;
    }
    let button = match std::env::var("ISOSTATE_WELCOME").as_deref() {
        Ok("continue") => go_on.single_mut(),
        Ok("new") => new_game.single_mut(),
        _ => return,
    };
    if let Ok(mut i) = button {
        *i = Interaction::Pressed;
        *pressed = true;
        eprintln!("pressing the card's button, as ISOSTATE_WELCOME asks");
    }
}

/// Start the program again on a new game, and leave this one. The game being
/// left is already saved; the new one sets it aside when it is first saved.
fn new_game_clicks(
    q: Query<&Interaction, (Changed<Interaction>, With<NewGameButton>)>,
    data: Res<Data>,
    // there is none in a replay, which has no such button either
    fam: Option<Res<FamilyView>>,
    mut hint: Query<&mut Text, With<WelcomeHint>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(fam) = fam else { return };
    if !q.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    // on the island on the table, with as many families: a save that was loaded may be of another than the arguments name
    let args: Vec<String> = std::env::args().skip(1).collect();
    let scripted = args.iter().any(|a| a == "--script");
    let args = crate::saves::new_game_args(&args, data.r.header.meta.seed, fam.families.len());
    if scripted {
        // a scripted run is one program from start to end: say what would be started, and stop
        println!("script: New game would start the program again with: {}", args.join(" "));
        exit.write(AppExit::Success);
        return;
    }
    let started = std::env::current_exe().and_then(|exe| std::process::Command::new(exe).args(args).spawn());
    match started {
        Ok(_) => {
            eprintln!("starting again on a new game");
            exit.write(AppExit::Success);
        }
        Err(e) => {
            eprintln!("a new game could not be started: {e}");
            for mut t in &mut hint {
                t.0 = "A new game could not be started from here. Close the game and start it again with --new.".to_string();
            }
        }
    }
}

fn close_welcome(
    mut commands: Commands,
    q: Query<(Entity, &Interaction), With<Welcome>>,
    go_on: Query<&Interaction, With<ContinueButton>>,
    new_game: Query<&Interaction, With<NewGameButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    game: Option<ResMut<Game>>,
) {
    // New game leaves the card up until the program starts again
    if new_game.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    let go_on = go_on.iter().any(|i| *i == Interaction::Pressed);
    for (e, i) in &q {
        if *i == Interaction::Pressed || go_on || keys.get_just_pressed().any(|k| !goes_with_others(k)) {
            commands.entity(e).despawn();
            if let Some(mut game) = game {
                game.welcoming = false;
                if game.open.is_none() {
                    game.open = game.queue.pop_front();
                }
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_present_is_the_newest_year_and_the_past_is_what_was_left_behind() {
        // the clock on the newest year there is
        assert_eq!(now_of(36, 36, 36), Now::Present);
        // the next year has been fetched ahead and the clock has not got there
        assert_eq!(now_of(36, 37, 36), Now::Turning);
        // the clock got there
        assert_eq!(now_of(37, 37, 36), Now::Present);
        // stepped back a year on purpose, or scrubbed far back and playing up
        // to the front again: the past until the clock is on the newest year
        assert_eq!(now_of(36, 37, 37), Now::Past);
        assert_eq!(now_of(3, 37, 37), Now::Past);
    }

    #[test]
    fn a_key_held_with_others_does_not_close_the_welcome() {
        for k in [KeyCode::AltLeft, KeyCode::ShiftRight, KeyCode::ControlLeft, KeyCode::SuperLeft, KeyCode::PrintScreen] {
            assert!(goes_with_others(&k), "{k:?}");
        }
        for k in [KeyCode::Space, KeyCode::Enter, KeyCode::Escape, KeyCode::KeyA, KeyCode::ArrowRight] {
            assert!(!goes_with_others(&k), "{k:?}");
        }
    }

    /// The clock and the simulation thread in miniature, at 30 frames a
    /// second: `month_secs` to work out a month and `turn_secs` more for the
    /// year's end. Returns the frames the clock stood waiting at a month's
    /// end, those it stood waiting at the year's end, and the share of frames
    /// in which the simulation stood at a year's turn (so an order given then
    /// would be carried out at once).
    fn play(speed: f32, month_secs: f32, turn_secs: f32, years: usize) -> (u32, u32, f32) {
        let dt = 1.0 / 30.0;
        let (mut t, mut latest, mut months, mut asked) = (0.0f32, 0usize, 0usize, 0usize);
        let (mut month_took, mut turn_took) = (None, None);
        // when each piece of work asked for will be done
        let mut due: std::collections::VecDeque<f32> = Default::default();
        let mut busy_until = 0.0f32;
        let (mut waited_month, mut waited_turn, mut at_turn, mut frames) = (0u32, 0u32, 0u32, 0u32);
        let mut now = 0.0f32;
        while latest < years {
            now += dt;
            // what was asked for arrives
            while due.front().is_some_and(|&d| now >= d) {
                due.pop_front();
                months += 1;
                if months == 12 {
                    (latest, months, asked) = (latest + 1, 0, 0);
                    turn_took = Some(month_secs + turn_secs);
                } else {
                    month_took = Some(month_secs);
                }
            }
            // the clock runs, and waits inside the first month still to come
            let cap = clock::frontier(latest, months);
            if t + dt * speed > cap {
                if months == 11 { waited_turn += 1 } else { waited_month += 1 }
            }
            t = (t + dt * speed).min(cap);
            let want = clock::months_due(t - latest as f32, clock::lead_years(month_took, clock::MONTH_GUESS, speed), clock::lead_years(turn_took, clock::TURN_GUESS, speed));
            while asked < want {
                asked += 1;
                busy_until = busy_until.max(now) + month_secs + if asked == 12 { turn_secs } else { 0.0 };
                due.push_back(busy_until);
            }
            frames += 1;
            // (for the moment between a year arriving and the clock reaching its turn, orders wait)
            at_turn += (asked == 0 && t >= latest as f32) as u32;
        }
        (waited_month, waited_turn, at_turn as f32 / frames as f32)
    }

    #[test]
    fn the_months_roll_without_a_stop_at_every_speed() {
        // a month takes 15 ms to work out and the year's end a quarter of a second more
        for s in clock::SPEEDS {
            let years = if s.rate < 0.2 { 4 } else { 20 };
            let (waited_month, waited_turn, at_turn) = play(s.rate, 0.015, 0.25, years);
            if s.rate <= 1.0 {
                // (before a month or a year has been timed the clock goes by a guess of how long they take)
                assert!(waited_month <= 2, "{}: the clock stood {waited_month} frames at a month's end", s.name);
                assert!(waited_turn <= 2 * years as u32, "{}: the clock stood {waited_turn} frames at the years' ends", s.name);
            }
            // at the slow speeds the simulation stands at the turn for most of January
            if s.rate <= 1.0 / 12.0 {
                assert!(at_turn > 0.07 && at_turn < 1.0 / 12.0 + 0.01, "{}: at the turn for {at_turn} of the time", s.name);
            }
        }
    }

    #[test]
    fn when_an_order_given_in_the_middle_of_a_year_will_happen() {

        // the newest year's end is 1783's: the year in progress is 1784
        assert_eq!(turn_words(1750, 33), ("the end of 1784".to_string(), "January 1785".to_string()));
    }
}
