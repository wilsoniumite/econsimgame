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

use crate::family::{self, short_group, trade, FamilyRow, FamilyView, GotoFirm, GotoTown, MemberRow, OpenDecision, PostButton, RivalRow, StanceButton};
use crate::live::{self, Cmd, Live, Msg, SiteInfo, PLAYER};
use crate::meshkit::{hex, lerp, Builder};
use crate::replay::{self, Replay};
use crate::ui::{money, text, UiState, BTN, INK, MUTED, PAPER_2, RULE};
use crate::look::{Look, TheLook};
use crate::{scatter, world, Data, InspectActions, Orbit, Pick, Picked, Timeline, UiFont};

#[derive(Resource, Default)]
pub struct Game {
    /// the answer about the picked cell, once it has come
    pub site: Option<SiteInfo>,
    /// the cell a question is out about
    pub asked: Option<usize>,
    /// how the last order went
    pub note: Option<(bool, String)>,
    /// an order is out
    pub busy: bool,
    /// show the next year as soon as it arrives (the player stepped forward)
    pub follow: bool,
    /// bumped whenever the order panel must be redrawn
    pub version: u64,
    /// the last order sent, until its answer comes
    last_order: Option<Action>,
    /// squares where the player's new works will stand next year
    pending: Vec<usize>,
    /// the end of the game has been shown
    ended: bool,
    drawn: Option<(Option<Pick>, usize, u64)>,
    marks: Option<(usize, usize, usize, u64)>,
    /// decisions already put in front of the player, so each opens by itself only once
    seen: BTreeSet<u64>,
    /// decisions to show, one after another
    queue: VecDeque<u64>,
    /// the decision whose card is open
    pub open: Option<u64>,
    /// the welcome card is up: decisions wait behind it
    welcoming: bool,
    card_drawn: Option<(Option<u64>, u64)>,
}

/// The card shown when the century is over.
#[derive(Component)]
struct EndCard;

/// The card that opens a new game.
#[derive(Component)]
struct Welcome;

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
struct DecideButton {
    id: u64,
    option: usize,
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
        .add_systems(Startup, setup_marks.run_if(resource_exists::<Live>))
        .add_systems(PostStartup, welcome.run_if(resource_exists::<Live>))
        .add_systems(Update, close_welcome)
        .add_systems(
            Update,
            (
                pump,
                drive,
                ask_site,
                action_clicks,
                decide_clicks,
                post_clicks,
                stance_clicks,
                panel_clicks,
                goto_clicks,
                next_year_clicks,
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
fn player_family(view: &mut FamilyView, rep: &FamilyReport) {
    if let Some(row) = view.families.get_mut(rep.index) {
        let h = std::mem::take(&mut row.history);
        *row = row_of(rep, h);
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

#[allow(clippy::too_many_arguments)]
fn pump(
    mut live: ResMut<Live>,
    mut data: ResMut<Data>,
    mut fam: ResMut<FamilyView>,
    mut game: ResMut<Game>,
    mut tl: ResMut<Timeline>,
    mut ui: ResMut<UiState>,
    shots: Option<Res<crate::ShotPlanArgs>>,
) {
    while let Some(msg) = live.try_recv() {
        match msg {
            Msg::Year(y, prospects) => {
                // copy-on-write: a scenery build may still hold the old run
                let Data { r, g, slots, .. } = &mut *data;
                let r = std::sync::Arc::make_mut(r);
                live::apply_year(r, &y);
                scatter::place_new_firms(r, g, std::sync::Arc::make_mut(slots));
                year_families(&mut fam, &y.families, y.year as i32);
                fam.prospects = prospects;
                live.stepping = false;
                if tl.built == Some(y.index) {
                    tl.built = None;
                }
                if game.follow {
                    game.follow = false;
                    tl.t = y.index as f32;
                }
                game.pending.clear();
                // the world has moved on: ask about the picked square again
                game.asked = None;
                game.site = None;
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
                    game.version += 1;
                }
            }
            Msg::Done { ok, text, family } => {
                game.busy = false;
                if let (true, Some(Action::Found { cell, .. })) = (ok, game.last_order) {
                    game.pending.push(cell);
                }
                game.last_order = None;
                if !text.is_empty() || !ok {
                    game.note = Some((ok, text));
                }
                if let Some(rep) = family {
                    player_family(&mut fam, &rep);
                    ui.dirty += 1;
                    queue_new(&mut game, &fam, &mut tl);
                }
                game.asked = None;
                game.version += 1;
            }
            Msg::Failed(e) => {
                live.stepping = false;
                game.busy = false;
                game.note = Some((false, e));
                game.version += 1;
            }
        }
    }
}

/// Screenshots skip the welcome and the decisions unless asked for.
fn cards_in_shots() -> bool {
    std::env::var_os("ISOSTATE_WELCOME").is_some()
}

/// Ask for the next year when the clock reaches the last one there is.
fn drive(mut live: ResMut<Live>, mut game: ResMut<Game>, tl: Res<Timeline>, data: Res<Data>, keys: Res<ButtonInput<KeyCode>>) {
    let r = &data.r;
    if r.filled >= r.ny || live.stepping {
        return;
    }
    let at_end = tl.t >= data.latest() as f32;
    if tl.playing && at_end {
        live.step();
    } else if !tl.playing && at_end && (keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::Period)) {
        // one more year, then stop and show it
        game.follow = true;
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
    if present(&tl, &data) && !live.stepping && game.asked != Some(c) {
        game.asked = Some(c);
        game.site = None;
        game.note = None;
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
                tl.t = data.latest() as f32;
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
        if *i == Interaction::Pressed && !game.busy {
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

fn next_year_clicks(q: Query<&Interaction, (Changed<Interaction>, With<crate::ui::NextYearButton>)>, mut live: ResMut<Live>, mut game: ResMut<Game>, mut tl: ResMut<Timeline>, data: Res<Data>) {
    for i in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        tl.playing = false;
        if !present(&tl, &data) {
            tl.t = data.latest() as f32;
        } else if data.r.filled < data.r.ny && !live.stepping {
            game.follow = true;
            live.step();
        }
    }
}

fn step_note(live: Res<Live>, data: Res<Data>, mut q: Query<&mut Text, With<crate::ui::StepNote>>) {
    let r = &data.r;
    let s = if live.stepping {
        format!("working out {}\u{2026}", r.y0 + r.filled as i32)
    } else if r.filled >= r.ny {
        "the century is over".to_string()
    } else {
        String::new()
    };
    for mut t in &mut q {
        if t.0 != s {
            t.0 = s.clone();
        }
    }
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
    font: Res<UiFont>,
    q: Query<(Entity, Option<&Children>), With<InspectActions>>,
) {
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    let key = (picked.what, k, game.version);
    if game.drawn == Some(key) && !fam.is_changed() {
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
    let latest_year = r.y0 + data.latest() as i32;
    let now = present(&tl, &data);
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
                if !now {
                    back_to_present(p, &f, r, k, latest_year, accent);
                    return;
                }
                town_orders(p, &f, &fam, me, r, t, &game);
            }
            Pick::Cell(c) => {
                rule(p);
                if !now {
                    back_to_present(p, &f, r, k, latest_year, accent);
                    return;
                }
                let Some(site) = game.site.as_ref().filter(|s| s.cell == c) else {
                    p.spawn(text(&f, "Asking the land agent\u{2026}", 13.5, MUTED));
                    return;
                };
                cell_orders(p, &f, me, r, site, &game);
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

fn back_to_present(p: &mut ChildSpawnerCommands, f: &Handle<Font>, r: &Replay, k: usize, latest_year: i32, accent: Color) {
    p.spawn(text(f, &format!("This is how it stood in {}. Orders are given in the present, {}.", r.y0 + k as i32, latest_year), 13.5, MUTED));
    p.spawn(Node { margin: UiRect::top(Val::Px(6.0)), ..default() }).with_children(|row| button(row, f, &format!("Back to {latest_year}"), Some(Action::Present), accent));
}

fn town_name(r: &Replay, t: usize) -> String {
    r.header.towns.get(t).map(|x| x.name.clone()).unwrap_or_else(|| format!("town {t}"))
}

/// A town: the seat, the family's branches and people there, and who could
/// be posted to run a trade there.
fn town_orders(p: &mut ChildSpawnerCommands, f: &Handle<Font>, fam: &FamilyView, me: &FamilyRow, r: &Replay, t: usize, game: &Game) {
    let rep = &me.rep;
    let accent = me.colour;
    let name = town_name(r, t);
    // the seat
    if t == rep.seat {
        p.spawn(text(f, &format!("The seat of the {} family.", rep.surname), 14.5, accent));
        p.spawn(text(f, "The head lives here and runs every trade the family knows here. Works are chosen and land bought directly only around the seat.", 12.5, MUTED));
        if let Some(next) = rep.next_seat {
            p.spawn(text(f, &format!("Moving to {} at the year end.", town_name(r, next)), 13.0, accent));
            p.spawn(Node { margin: UiRect::top(Val::Px(4.0)), ..default() }).with_children(|x| button(x, f, &format!("Stay at {name}"), (!game.busy).then_some(Action::MoveSeat(t)), accent));
        }
    } else if rep.next_seat == Some(t) {
        p.spawn(text(f, &format!("The {} family's seat moves here at the year end.", rep.surname), 14.0, accent));
        p.spawn(Node { margin: UiRect::top(Val::Px(4.0)), ..default() }).with_children(|x| button(x, f, &format!("Stay at {}", rep.seat_name), (!game.busy).then_some(Action::MoveSeat(rep.seat)), accent));
    } else {
        p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(10.0), padding: UiRect::vertical(Val::Px(2.0)), ..default() }).with_children(|x| {
            x.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 1.0, ..default() }).with_children(|col| {
                col.spawn(text(f, &format!("Move the seat to {name}?"), 14.0, INK));
                col.spawn(text(f, &format!("The head moves at the year end, and with the head the bonus and the family's direct choices. Now at {}.", rep.seat_name), 12.0, MUTED));
            });
            button(x, f, "Move", (!game.busy).then_some(Action::MoveSeat(t)), accent);
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
fn cell_orders(p: &mut ChildSpawnerCommands, f: &Handle<Font>, me: &FamilyRow, r: &Replay, site: &SiteInfo, game: &Game) {
    let rep = &me.rep;
    let accent = me.colour;
    let c = site.cell;
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
        if site.owner != Some(PLAYER) && site.land_allowed.is_ok() {
            let ok = me.cash as f64 >= site.land_price && !game.busy;
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
            if good.is_empty() {
                p.spawn(text(f, "Nothing in the family's trades would pay here.", 13.0, MUTED));
            }
            for (m, s) in good.iter().take(5) {
                let afford = me.cash as f64 >= s.capital_cost && !game.busy;
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
fn decision_card(mut commands: Commands, mut game: ResMut<Game>, fam: Res<FamilyView>, data: Res<Data>, font: Res<UiFont>, cards: Query<Entity, With<DecisionCard>>) {
    let key = (game.open, game.version);
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
                    c.spawn((
                        Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(Val::Px(12.0), Val::Px(7.0)), border_radius: BorderRadius::all(Val::Px(11.0)), ..default() },
                        BackgroundColor(BTN),
                        Interaction::default(),
                        FocusPolicy::Block,
                        crate::ui::Hoverable,
                        DecideButton { id: d.id, option: i },
                    ))
                    .with_children(|b| {
                        b.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() }).with_children(|head| {
                            head.spawn(text(&f, &o.label, 15.5, INK));
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
                    row.spawn(text(&f, &format!("Unanswered, the default is taken at the end of {}.", d.due), 12.5, MUTED));
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
}

fn banner_rgba(i: usize) -> [f32; 4] {
    hex(family::BANNERS[i % family::BANNERS.len()])
}

/// A pennant on a pole beside every family works, a low fence round every
/// family's land, and a tall flag on the market square of every family's seat.
fn build_marks(r: &Replay, g: &world::Ground, slots: &scatter::Slots, cen: &[(f32, f32)], seats: &[(usize, usize, bool)], k: usize, lk: &Look) -> Builder {
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
        let Some(&(x, z, yaw)) = slots.firm_pos.get(f) else { continue };
        let q = Quat::from_rotation_y(yaw);
        let off = q * Vec3::new(-0.055, 0.0, -0.045);
        let base = Vec3::new(x + off.x, world::height_at(g, x + off.x, z + off.z), z + off.z);
        let top = base + Vec3::Y * 0.13;
        b.beam(base - Vec3::Y * 0.01, top, 0.005, pole);
        // the pennant streams downwind, the same way on every pole
        let fly = Vec3::new(0.06, -0.012, 0.03);
        let col = banner_rgba(o);
        b.tri(top, top - Vec3::Y * 0.032, top + fly, col);
        b.tri(top, top + fly, top - Vec3::Y * 0.032, col);
    }
    if !r.owner.is_empty() {
        let n = r.n;
        let own = |c: usize| r.owner[k * n + c];
        for c in 0..n {
            let o = own(c);
            if o < 0 {
                continue;
            }
            let (x, z) = ((c % r.w) as i32, (c / r.w) as i32);
            let col = lerp(banner_rgba(o as usize), *lk.marks.cloth_pale, 0.25);
            let post = lerp(col, *lk.marks.post_shade, 0.35);
            let inset = 0.03;
            // edges whose far side is not this family's: (dx, dz) to the neighbour, and the edge's two ends
            let x0 = x as f32 + inset;
            let x1 = x as f32 + 1.0 - inset;
            let z0 = z as f32 + inset;
            let z1 = z as f32 + 1.0 - inset;
            for (dx, dz, a, e) in [(0, -1, (x0, z0), (x1, z0)), (0, 1, (x0, z1), (x1, z1)), (-1, 0, (x0, z0), (x0, z1)), (1, 0, (x1, z0), (x1, z1))] {
                let (nx, nz) = (x + dx, z + dz);
                let same = nx >= 0 && nz >= 0 && (nx as usize) < r.w && (nz as usize) < r.h && own(nz as usize * r.w + nx as usize) == o;
                if same {
                    continue;
                }
                let steps = 8;
                let mut prev: Option<Vec3> = None;
                for i in 0..=steps {
                    let t = i as f32 / steps as f32;
                    let (px, pz) = (a.0 + (e.0 - a.0) * t, a.1 + (e.1 - a.1) * t);
                    let h = world::height_at(g, px, pz);
                    if h < 0.03 {
                        prev = None;
                        continue;
                    }
                    let p0 = Vec3::new(px, h, pz);
                    b.block(p0, 0.008, 0.008, 0.024, 0.0, post, Some(post), 0.01);
                    let rail = p0 + Vec3::Y * 0.017;
                    if let Some(q0) = prev {
                        b.beam(q0, rail, 0.005, col);
                    }
                    prev = Some(rail);
                }
            }
        }
    }
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

fn owner_marks(mut game: ResMut<Game>, tl: Res<Timeline>, data: Res<Data>, look: Res<TheLook>, fam: Res<FamilyView>, marks: Option<Res<Marks>>, mut meshes: ResMut<Assets<Mesh>>) {
    let Some(marks) = marks else { return };
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    // the seats are known for the present only, so they are drawn there
    let seats: Vec<(usize, usize, bool)> = if present(&tl, &data) {
        fam.families.iter().flat_map(|f| std::iter::once((f.rep.index, f.seat, false)).chain(f.rep.next_seat.map(|t| (f.rep.index, t, true)))).collect()
    } else {
        Vec::new()
    };
    let seat_key = seats.iter().fold(0u64, |h, &(a, b, m)| h.wrapping_mul(31).wrapping_add((a * 1000 + b * 2 + m as usize) as u64 + 1));
    let key = (k, data.r.filled, game.pending.len(), seat_key);
    if game.marks == Some(key) {
        return;
    }
    game.marks = Some(key);
    let mut b = build_marks(&data.r, &data.g, &data.slots, &data.cen, &seats, k, &look);
    if present(&tl, &data) {
        for &c in &game.pending {
            stakes(&mut b, &look, &data.g, &data.r, c, banner_rgba(PLAYER));
        }
    }
    if let Some(mut m) = meshes.get_mut(&marks.0) {
        *m = if b.is_empty() { crate::placeholder_mesh() } else { b.into_mesh() };
    }
}

/// When the last year has been played: where the families finished.
#[allow(clippy::too_many_arguments)]
fn end_card(mut commands: Commands, mut game: ResMut<Game>, tl: Res<Timeline>, data: Res<Data>, fam: Res<FamilyView>, font: Res<UiFont>, keys: Res<ButtonInput<KeyCode>>, cards: Query<Entity, With<EndCard>>, shots: Option<Res<crate::ShotPlanArgs>>) {
    if keys.just_pressed(KeyCode::Escape) {
        for e in &cards {
            commands.entity(e).despawn();
        }
    }
    let r = &data.r;
    if game.ended || r.filled < r.ny || tl.t < (r.ny - 1) as f32 {
        return;
    }
    // screenshots skip it, as they do the welcome, unless asked for
    if shots.is_some() && !cards_in_shots() {
        return;
    }
    game.ended = true;
    let f = font.0.clone();
    let mut rank: Vec<&FamilyRow> = fam.families.iter().collect();
    rank.sort_by(|a, b| b.net_worth.partial_cmp(&a.net_worth).unwrap_or(std::cmp::Ordering::Equal));
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
/// first decision waits behind it.
fn welcome(mut commands: Commands, fam: Res<FamilyView>, data: Res<Data>, font: Res<UiFont>, mut game: ResMut<Game>, shots: Option<Res<crate::ShotPlanArgs>>) {
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
    let year = data.r.y0 + data.latest() as i32;
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
                    "Next year plays one year and stops; Space keeps the years going. H shows every key.",
                ] {
                    c.spawn(Node { column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                        row.spawn((Node { width: Val::Px(7.0), height: Val::Px(7.0), margin: UiRect::top(Val::Px(7.0)), border_radius: BorderRadius::all(Val::Px(4.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(me.colour)));
                        row.spawn(text(&f, line, 14.5, INK));
                    });
                }
                c.spawn((text(&f, "Click anywhere to begin.", 13.0, MUTED), Node { margin: UiRect::top(Val::Px(8.0)), ..default() }));
            });
        });
}

fn close_welcome(mut commands: Commands, q: Query<(Entity, &Interaction), With<Welcome>>, keys: Res<ButtonInput<KeyCode>>, game: Option<ResMut<Game>>) {
    for (e, i) in &q {
        if *i == Interaction::Pressed || keys.get_just_pressed().next().is_some() {
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
