//! Playing a live game: the player's orders from the map and the panels, and
//! the families' banners and fences drawn on the island.

// see the note at the top of ui.rs
#![allow(deprecated)]

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use isostate_core::families::Group;
use isostate_core::FamilyReport;

use crate::family::{self, AssignButton, FamilyRow, FamilyView, GotoFirm, Member, MemberRow};
use crate::live::{self, Cmd, Live, Msg, SiteInfo, PLAYER};
use crate::meshkit::{hex, lerp, Builder};
use crate::replay::{self, Replay};
use crate::ui::{money, text, UiState, BTN, INK, MUTED, PAPER_2, RULE};
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
    drawn: Option<(Option<Pick>, usize, u64)>,
    marks: Option<(usize, usize)>,
}

#[derive(Clone, Copy)]
pub enum Action {
    Buy(usize),
    Found { method: usize, cell: usize },
    /// back to the present, where orders can be given
    Present,
}

#[derive(Component, Clone, Copy)]
pub struct ActionButton(pub Action);

#[derive(Resource)]
struct Marks(Handle<Mesh>);

pub fn plugin(app: &mut App) {
    app.init_resource::<Game>().add_systems(Startup, setup_marks.run_if(resource_exists::<Live>)).add_systems(
        Update,
        (pump, drive, ask_site, action_clicks, assign_clicks, member_clicks, goto_clicks, order_panel, owner_marks).chain().run_if(resource_exists::<Live>),
    );
}

// ------------------------------------------------------------------ the families, as the panels see them

fn row_of(rep: &FamilyReport, history: Vec<f32>, r: &Replay, method_group: &[Option<Group>]) -> FamilyRow {
    let mut works_by_group = [0usize; 6];
    for &f in &rep.firms {
        if let Some(g) = r.header.firms.get(f).and_then(|firm| method_group.get(firm.method).copied().flatten()) {
            works_by_group[g.index()] += 1;
        }
    }
    FamilyRow {
        name: rep.surname.clone(),
        colour: family::banner(rep.index),
        cash: rep.cash as f32,
        net_worth: rep.net_worth as f32,
        income: rep.last_year.earned() as f32,
        works: rep.firms.len(),
        land: rep.cells.len(),
        members: rep
            .members
            .iter()
            .filter(|m| m.alive)
            .map(|m| Member { index: m.index, name: m.first.clone(), age: m.age as i32, group: m.group, skill: m.skill as f32 })
            .collect(),
        holdings: rep.firms.clone(),
        bonus: rep.bonus.map(|b| b as f32),
        works_by_group,
        history,
    }
}

/// A new year's reports: every family's worth goes on its record.
pub fn year_families(view: &mut FamilyView, reports: &[FamilyReport], year: i32, r: &Replay, method_group: &[Option<Group>]) {
    let old = std::mem::take(&mut view.families);
    view.families = reports
        .iter()
        .map(|rep| {
            let mut h = old.get(rep.index).map(|r| r.history.clone()).unwrap_or_default();
            h.push(rep.net_worth as f32);
            row_of(rep, h, r, method_group)
        })
        .collect();
    view.year = year;
    view.player = reports.iter().position(|r| r.player);
    view.live = true;
}

/// The player's family after an order, between years.
fn player_family(view: &mut FamilyView, rep: &FamilyReport, r: &Replay, method_group: &[Option<Group>]) {
    if let Some(row) = view.families.get_mut(rep.index) {
        let h = std::mem::take(&mut row.history);
        *row = row_of(rep, h, r, method_group);
    }
}

// ------------------------------------------------------------------ the simulation thread

fn pump(mut live: ResMut<Live>, mut data: ResMut<Data>, mut fam: ResMut<FamilyView>, mut game: ResMut<Game>, mut tl: ResMut<Timeline>, mut ui: ResMut<UiState>) {
    while let Some(msg) = live.try_recv() {
        match msg {
            Msg::Year(y) => {
                let Data { r, g, slots, .. } = &mut *data;
                live::apply_year(r, &y);
                scatter::place_new_firms(r, g, slots);
                year_families(&mut fam, &y.families, y.year as i32, r, &live.method_group);
                live.stepping = false;
                if tl.built == Some(y.index) {
                    tl.built = None;
                }
                if game.follow {
                    game.follow = false;
                    tl.t = y.index as f32;
                }
                // the world has moved on: ask about the picked square again
                game.asked = None;
                game.site = None;
                game.version += 1;
                ui.dirty += 1;
            }
            Msg::Site(s) => {
                if game.asked == Some(s.cell) {
                    game.site = Some(s);
                    game.version += 1;
                }
            }
            Msg::Done { ok, text, family } => {
                game.busy = false;
                game.note = Some((ok, text));
                if let Some(rep) = family {
                    player_family(&mut fam, &rep, &data.r, &live.method_group);
                    ui.dirty += 1;
                }
                game.asked = None;
                game.version += 1;
            }
            Msg::Failed(e) => {
                live.stepping = false;
                game.note = Some((false, e));
                game.version += 1;
            }
        }
    }
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
        match a.0 {
            Action::Buy(c) => {
                live.send(Cmd::BuyLand(c));
                game.busy = true;
            }
            Action::Found { method, cell } => {
                live.send(Cmd::Found { method, cell });
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

fn assign_clicks(q: Query<(&Interaction, &AssignButton), Changed<Interaction>>, live: Res<Live>) {
    for (i, a) in &q {
        if *i == Interaction::Pressed {
            live.send(Cmd::Assign { member: a.member, group: a.group });
        }
    }
}

/// Clicking a member shows the posts they could take; clicking again hides them.
fn member_clicks(q: Query<(&Interaction, &MemberRow), Changed<Interaction>>, mut ui: ResMut<UiState>) {
    for (i, m) in &q {
        if *i == Interaction::Pressed {
            ui.expanded = if ui.expanded == Some(m.0) { None } else { Some(m.0) };
            ui.dirty += 1;
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
    let me = fam.player();
    let accent = me.map(|m| m.colour).unwrap_or(INK);
    let cash = me.map(|m| m.cash).unwrap_or(0.0);
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
            }
            Pick::Town(_) => {}
            Pick::Cell(c) => {
                rule(p);
                if !now {
                    p.spawn(text(&f, &format!("This is how it stood in {}. Orders are given in the present, {}.", r.y0 + k as i32, latest_year), 13.5, MUTED));
                    p.spawn(Node { margin: UiRect::top(Val::Px(6.0)), ..default() }).with_children(|row| button(row, &f, &format!("Back to {latest_year}"), Some(Action::Present), accent));
                    return;
                }
                let Some(site) = game.site.as_ref().filter(|s| s.cell == c) else {
                    p.spawn(text(&f, "Asking the land agent\u{2026}", 13.5, MUTED));
                    return;
                };
                // the land
                let held = match site.owner {
                    Some(o) if o == PLAYER => "Your family's land.".to_string(),
                    Some(o) => format!("The {} family's land.", family_name(&fam, o)),
                    None => "Held by the gentry, who would sell.".to_string(),
                };
                p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                    row.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 1.0, ..default() }).with_children(|col| {
                        col.spawn(text(&f, &held, 14.0, if site.owner == Some(PLAYER) { accent } else { INK }));
                        if site.owner != Some(PLAYER) {
                            col.spawn(text(&f, &format!("{} for the freehold, 25 years' rent.", money(site.land_price as f32)), 12.5, MUTED));
                        }
                    });
                    if site.owner != Some(PLAYER) {
                        let ok = cash as f64 >= site.land_price && !game.busy;
                        button(row, &f, "Buy", ok.then_some(Action::Buy(c)), accent);
                    }
                });
                // works
                match &site.options {
                    Err(why) => {
                        rule(p);
                        p.spawn(text(&f, &format!("Nothing can be built here: {why}."), 13.5, MUTED));
                    }
                    Ok(opts) => {
                        let mut good: Vec<(usize, &isostate_core::SiteSums)> = opts.iter().filter_map(|o| o.sums.as_ref().ok().map(|s| (o.method, s))).collect();
                        good.retain(|(m, _)| r.header.methods[*m].name != "services");
                        good.sort_by(|a, b| b.1.expected_return.partial_cmp(&a.1.expected_return).unwrap_or(std::cmp::Ordering::Equal));
                        let unknown = opts.iter().filter(|o| o.sums.is_err()).count();
                        rule(p);
                        p.spawn(text(&f, "Build here", 16.0, INK));
                        if good.is_empty() {
                            p.spawn(text(&f, "Nothing anybody knows how to build would pay here.", 13.0, MUTED));
                        }
                        for (m, s) in good.iter().take(5) {
                            let afford = cash as f64 >= s.capital_cost && !game.busy;
                            p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(10.0), padding: UiRect::vertical(Val::Px(3.0)), ..default() }).with_children(|row| {
                                row.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 1.0, ..default() }).with_children(|col| {
                                    col.spawn(text(&f, &r.header.methods[*m].label, 14.5, INK));
                                    let ret = if s.expected_return > 0.0 { format!("about {:.0}% a year", 100.0 * s.expected_return) } else { "would lose money".to_string() };
                                    let bonus = if s.bonus > 0.0 { format!(", {:.0}% cheaper with your manager", 100.0 * s.bonus) } else { String::new() };
                                    col.spawn(text(&f, &format!("{} to build, {}{}", money(s.capital_cost as f32), ret, bonus), 12.5, MUTED));
                                });
                                button(row, &f, "Build", afford.then_some(Action::Found { method: *m, cell: c }), accent);
                            });
                        }
                        if unknown > 0 {
                            p.spawn(text(&f, &format!("{unknown} other kinds of works are not known here yet."), 12.5, MUTED));
                        }
                    }
                }
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

// ------------------------------------------------------------------ banners and fences

fn setup_marks(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let h = meshes.add(crate::placeholder_mesh());
    let mat = crate::vertex_coloured(&mut materials, 0.8);
    commands.spawn((Mesh3d(h.clone()), MeshMaterial3d(mat)));
    commands.insert_resource(Marks(h));
}

fn banner_rgba(i: usize) -> [f32; 4] {
    hex(family::BANNERS[i % family::BANNERS.len()])
}

/// A pennant on a pole beside every family works, and a low fence round every family's land.
fn build_marks(r: &Replay, g: &world::Ground, slots: &scatter::Slots, k: usize) -> Builder {
    let mut b = Builder::default();
    let pole = hex(0x6b5a48);
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
            let col = lerp(banner_rgba(o as usize), hex(0xf4ecd8), 0.25);
            let post = lerp(col, hex(0x5a4a3a), 0.35);
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

fn owner_marks(mut game: ResMut<Game>, tl: Res<Timeline>, data: Res<Data>, marks: Option<Res<Marks>>, mut meshes: ResMut<Assets<Mesh>>) {
    let Some(marks) = marks else { return };
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    let key = (k, data.r.filled);
    if game.marks == Some(key) {
        return;
    }
    game.marks = Some(key);
    let b = build_marks(&data.r, &data.g, &data.slots, k);
    if let Some(mut m) = meshes.get_mut(&marks.0) {
        *m = if b.is_empty() { crate::placeholder_mesh() } else { b.into_mesh() };
    }
}
