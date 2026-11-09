//! The families as the interface sees them: who they are, what they own and
//! how they stand against each other. The live game fills a `FamilyView`
//! each year; a plain replay has none, and the panels say so.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use crate::replay::Replay;
use crate::ui::{money, text, BAD, GOOD, INK, MUTED, RULE};

/// Banner colours, in the order families are given them.
pub const BANNERS: [u32; 7] = [0xb23a3a, 0x2f5fa8, 0x3b7d3e, 0xc9952b, 0x7a4a9a, 0x2b8a86, 0xd06a2a];

pub fn banner(i: usize) -> Color {
    let h = BANNERS[i % BANNERS.len()];
    Color::srgb_u8((h >> 16) as u8, (h >> 8) as u8, h as u8)
}

#[derive(Clone, Debug)]
pub struct Member {
    pub name: String,
    pub born: i32,
    /// what they do for the family, in words: "Head of the family", "Runs the collieries"
    pub role: String,
    /// 0..1
    pub skill: f32,
}

#[derive(Clone, Debug)]
pub struct Holding {
    pub firm: usize,
    pub capital: f32,
    pub profit: f32,
}

#[derive(Clone, Debug)]
pub struct FamilyRow {
    pub name: String,
    pub colour: Color,
    pub cash: f32,
    pub net_worth: f32,
    pub income: f32,
    pub works: usize,
    /// squares of land held outright
    pub land: usize,
    pub members: Vec<Member>,
    pub holdings: Vec<Holding>,
    /// net worth at the end of each year so far
    pub history: Vec<f32>,
}

#[derive(Resource, Default, Clone)]
pub struct FamilyView {
    pub year: i32,
    pub families: Vec<FamilyRow>,
    pub player: Option<usize>,
}

impl FamilyView {
    pub fn player(&self) -> Option<&FamilyRow> {
        self.player.and_then(|i| self.families.get(i))
    }
}

fn heading(p: &mut ChildSpawnerCommands, f: &Handle<Font>, s: &str) {
    p.spawn((text(f, s, 17.0, INK), Node { margin: UiRect::top(Val::Px(12.0)), ..default() }));
}

fn no_family(p: &mut ChildSpawnerCommands, f: &Handle<Font>, what: &str) {
    p.spawn((
        Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(6.0), padding: UiRect::all(Val::Px(12.0)), margin: UiRect::top(Val::Px(6.0)), border_radius: BorderRadius::all(Val::Px(12.0)), ..default() },
        BackgroundColor(crate::ui::PAPER_2),
    ))
    .with_children(|c| {
        c.spawn(text(f, "This is a recorded history", 16.0, INK));
        c.spawn(text(f, &format!("Nobody plays a family in a replay, so there is no {what} to show. Start a game to found a family; its works, land and people appear here."), 13.5, MUTED));
    });
}

/// A small bar chart of one family's worth over the years, in its colour.
fn sparkline(p: &mut ChildSpawnerCommands, history: &[f32], colour: Color) {
    let n = history.len().min(60);
    if n < 2 {
        return;
    }
    let tail = &history[history.len() - n..];
    let top = tail.iter().cloned().fold(1.0_f32, f32::max);
    p.spawn(Node { height: Val::Px(44.0), align_items: AlignItems::End, column_gap: Val::Px(2.0), margin: UiRect::vertical(Val::Px(6.0)), ..default() }).with_children(|row| {
        for &v in tail {
            let h = (v.max(0.0) / top * 44.0).max(2.0);
            row.spawn((Node { flex_grow: 1.0, height: Val::Px(h), border_radius: BorderRadius::top(Val::Px(2.0)), ..default() }, BackgroundColor(colour.with_alpha(0.85))));
        }
    });
}

fn ordinal(n: usize) -> String {
    let s = match (n % 10, n % 100) {
        (1, x) if x != 11 => "st",
        (2, x) if x != 12 => "nd",
        (3, x) if x != 13 => "rd",
        _ => "th",
    };
    format!("{n}{s}")
}

pub fn family_panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, fam: Option<&FamilyView>, _r: &Replay, k: usize) {
    let Some(v) = fam else { return no_family(p, f, "family") };
    let Some(me) = v.player() else { return no_family(p, f, "family") };
    let mut rank: Vec<usize> = (0..v.families.len()).collect();
    rank.sort_by(|&a, &b| v.families[b].net_worth.partial_cmp(&v.families[a].net_worth).unwrap());
    let my_rank = rank.iter().position(|&i| Some(i) == v.player).unwrap_or(0) + 1;
    p.spawn(text(f, &format!("Worth {}, the {} fortune on the island of {}.", money(me.net_worth), ordinal(my_rank), v.families.len()), 14.0, MUTED));
    sparkline(p, &me.history, me.colour);
    let change = if me.history.len() >= 2 { me.history[me.history.len() - 1] - me.history[me.history.len() - 2] } else { 0.0 };
    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|row| {
        row.spawn(text(f, &format!("{} in hand", money(me.cash)), 14.0, INK));
        row.spawn(text(f, &format!("{}{} this year", if change >= 0.0 { "+" } else { "" }, money(change)), 14.0, if change >= 0.0 { GOOD } else { BAD }));
    });

    heading(p, f, "The family");
    let year = v.year.max(_r.y0 + k as i32);
    for m in &me.members {
        p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::vertical(Val::Px(4.0)), border: UiRect::bottom(Val::Px(1.0)), ..default() })
            .insert(BorderColor::all(RULE))
            .with_children(|row| {
                row.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|col| {
                    col.spawn(text(f, &format!("{}, {}", m.name, year - m.born), 15.0, INK));
                    col.spawn(text(f, &m.role, 12.5, MUTED));
                });
                // skill as five dots
                row.spawn(Node { column_gap: Val::Px(3.0), ..default() }).with_children(|dots| {
                    let full = (m.skill * 5.0).round() as i32;
                    for i in 0..5 {
                        dots.spawn((Node { width: Val::Px(8.0), height: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }, BackgroundColor(if i < full { me.colour } else { RULE })));
                    }
                });
            });
    }

    heading(p, f, "The great families");
    for (pos, &i) in rank.iter().enumerate() {
        let fr = &v.families[i];
        let mine = Some(i) == v.player;
        p.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(8.0), padding: UiRect::vertical(Val::Px(3.0)), ..default() }).with_children(|row| {
            row.spawn((text(f, &format!("{}", pos + 1), 13.0, MUTED), Node { width: Val::Px(16.0), ..default() }));
            row.spawn((Node { width: Val::Px(12.0), height: Val::Px(14.0), border_radius: BorderRadius::new(Val::Px(2.0), Val::Px(2.0), Val::Px(6.0), Val::Px(6.0)), ..default() }, BackgroundColor(fr.colour)));
            row.spawn((text(f, &fr.name, 15.0, if mine { fr.colour } else { INK }), Node { flex_grow: 1.0, ..default() }));
            row.spawn((text(f, &format!("{} works", fr.works), 13.0, MUTED), Node { width: Val::Px(64.0), ..default() }));
            row.spawn((text(f, &money(fr.net_worth), 14.0, INK), Node { width: Val::Px(64.0), justify_content: JustifyContent::End, ..default() }));
        });
    }
}

pub fn holdings_panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, fam: Option<&FamilyView>, r: &Replay, k: usize) {
    let Some(me) = fam.and_then(|v| v.player()) else { return no_family(p, f, "property") };
    p.spawn(text(f, &format!("{} works, {} squares of land, {} a year coming in.", me.works, me.land, money(me.income)), 14.0, MUTED));
    heading(p, f, "Works");
    if me.holdings.is_empty() {
        p.spawn(text(f, "None yet. Click a square on the map to found one.", 14.0, MUTED));
    }
    let mut hs = me.holdings.clone();
    hs.sort_by(|a, b| b.capital.partial_cmp(&a.capital).unwrap());
    for h in hs.iter().take(40) {
        let Some(firm) = r.header.firms.get(h.firm) else { continue };
        let m = &r.header.methods[firm.method];
        let town = r.header.towns.get(firm.town.max(0) as usize).map(|t| t.name.as_str()).unwrap_or("the country");
        let open = crate::replay::firm_k(r, h.firm, k) > 0.0;
        p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::vertical(Val::Px(4.0)), border: UiRect::bottom(Val::Px(1.0)), ..default() })
            .insert(BorderColor::all(RULE))
            .with_children(|row| {
                row.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|col| {
                    col.spawn(text(f, &m.label, 15.0, if open { INK } else { MUTED }));
                    col.spawn(text(f, &format!("near {}{}", town, if open { "" } else { ", closed" }), 12.5, MUTED));
                });
                row.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::End, ..default() }).with_children(|col| {
                    col.spawn(text(f, &money(h.capital), 14.0, INK));
                    col.spawn(text(f, &format!("{}{} a year", if h.profit >= 0.0 { "+" } else { "" }, money(h.profit)), 12.5, if h.profit >= 0.0 { GOOD } else { BAD }));
                });
            });
    }
}
