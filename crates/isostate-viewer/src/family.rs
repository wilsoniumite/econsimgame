//! The families as the interface sees them: who they are, what they own and
//! how they stand against each other. The live game fills a `FamilyView`
//! each year; a plain replay has none, and the panels say so.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use isostate_core::families::Group;

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
    /// index in the family, for orders
    pub index: usize,
    pub name: String,
    pub age: i32,
    /// the industry group they manage
    pub group: Option<Group>,
    /// 0..1
    pub skill: f32,
}

/// A button that puts a member in charge of a group (or of nothing).
#[derive(Component, Clone, Copy)]
pub struct AssignButton {
    pub member: usize,
    pub group: Option<Group>,
}

/// A member's row: click to choose what they manage.
#[derive(Component, Clone, Copy)]
pub struct MemberRow(pub usize);

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
    /// the works it owns (firm ids)
    pub holdings: Vec<usize>,
    /// what its managers save on every unit, by group
    pub bonus: [f32; 6],
    /// how many of its works are in each group
    pub works_by_group: [usize; 6],
    /// last year's accounts: dividends, rents, land sold, spent on living, put into works and land
    pub dividends: f32,
    pub rents: f32,
    pub sales: f32,
    pub spending: f32,
    pub invested: f32,
    /// net worth at the end of each year so far
    pub history: Vec<f32>,
}

#[derive(Resource, Default, Clone)]
pub struct FamilyView {
    pub year: i32,
    pub families: Vec<FamilyRow>,
    pub player: Option<usize>,
    /// the player can give orders (a live game, not a replay)
    pub live: bool,
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

/// A small bar chart of one family's worth over the years, in its colour,
/// the latest year on the right.
fn sparkline(p: &mut ChildSpawnerCommands, f: &Handle<Font>, history: &[f32], colour: Color) {
    let n = history.len().min(50);
    if n < 2 {
        return;
    }
    let tail = &history[history.len() - n..];
    let top = tail.iter().cloned().fold(1.0_f32, f32::max);
    p.spawn(Node { flex_direction: FlexDirection::Column, margin: UiRect::vertical(Val::Px(6.0)), ..default() }).with_children(|col| {
        col.spawn(Node { height: Val::Px(40.0), align_items: AlignItems::End, justify_content: JustifyContent::End, column_gap: Val::Px(2.0), border: UiRect::bottom(Val::Px(1.0)), ..default() })
            .insert(BorderColor::all(RULE))
            .with_children(|row| {
                for (i, &v) in tail.iter().enumerate() {
                    let h = (v.max(0.0) / top * 40.0).max(1.0);
                    let last = i + 1 == tail.len();
                    row.spawn((Node { width: Val::Px(4.0), height: Val::Px(h), border_radius: BorderRadius::top(Val::Px(2.0)), ..default() }, BackgroundColor(colour.with_alpha(if last { 1.0 } else { 0.55 }))));
                }
            });
        col.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|row| {
            row.spawn(text(f, &format!("{} years ago", n - 1), 11.5, MUTED));
            row.spawn(text(f, "now", 11.5, MUTED));
        });
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

pub fn family_panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, fam: Option<&FamilyView>, _r: &Replay, k: usize, expanded: Option<usize>) {
    let Some(v) = fam else { return no_family(p, f, "family") };
    let Some(me) = v.player() else { return no_family(p, f, "family") };
    let mut rank: Vec<usize> = (0..v.families.len()).collect();
    rank.sort_by(|&a, &b| v.families[b].net_worth.partial_cmp(&v.families[a].net_worth).unwrap());
    let my_rank = rank.iter().position(|&i| Some(i) == v.player).unwrap_or(0) + 1;
    p.spawn(text(f, &format!("Worth {}, the {} fortune on the island of {}.", money(me.net_worth), ordinal(my_rank), v.families.len()), 14.0, MUTED));
    sparkline(p, f, &me.history, me.colour);
    let change = if me.history.len() >= 2 { me.history[me.history.len() - 1] - me.history[me.history.len() - 2] } else { 0.0 };
    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|row| {
        row.spawn(text(f, &format!("{} in hand", money(me.cash)), 14.0, INK));
        row.spawn(text(f, &format!("{}{} this year", if change >= 0.0 { "+" } else { "" }, money(change)), 14.0, if change >= 0.0 { GOOD } else { BAD }));
    });

    heading(p, f, "Last year's accounts");
    let line = |p: &mut ChildSpawnerCommands, label: &str, v: f32, sign: f32| {
        p.spawn(Node { justify_content: JustifyContent::SpaceBetween, padding: UiRect::vertical(Val::Px(1.0)), ..default() }).with_children(|row| {
            row.spawn(text(f, label, 13.5, INK));
            let s = if v.abs() < 0.5 { "\u{2013}".to_string() } else if sign > 0.0 { money(v) } else { format!("\u{2212}{}", money(v)) };
            row.spawn(text(f, &s, 13.5, if sign > 0.0 || v.abs() < 0.5 { INK } else { MUTED }));
        });
    };
    line(p, "Dividends from the works", me.dividends, 1.0);
    line(p, "Rents from the land", me.rents, 1.0);
    if me.sales > 0.5 {
        line(p, "Land sold", me.sales, 1.0);
    }
    line(p, "Spent on living", me.spending, -1.0);
    line(p, "Put into works and land", me.invested, -1.0);

    heading(p, f, "The family");
    p.spawn(text(f, "A member who manages an industry makes the family's works in it a little cheaper to run. The abler the manager, the bigger the saving.", 12.5, MUTED));
    let _ = (_r, k);
    for m in &me.members {
        let open = expanded == Some(m.index) && m.age >= 18 && v.live;
        p.spawn((
            Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(6.0), padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)), border: UiRect::bottom(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
            BorderColor::all(RULE),
            BackgroundColor(if open { crate::ui::PAPER_2 } else { Color::NONE }),
        ))
        .with_children(|block| {
            let mut head = block.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() });
            if m.age >= 18 && v.live {
                head.insert((Interaction::default(), MemberRow(m.index)));
            }
            head.with_children(|row| {
                row.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|col| {
                    col.spawn(text(f, &format!("{} {}, {}", m.name, me.name, m.age), 15.0, INK));
                    let role = match m.group {
                        Some(g) => format!("Manages {}, {:.0}% cheaper to run", g.label().to_lowercase(), 100.0 * me.bonus[g.index()]),
                        None if m.age < 18 => "Too young to manage anything".to_string(),
                        None if v.live => "Has no post. Click to give one.".to_string(),
                        None => "Has no post".to_string(),
                    };
                    col.spawn(text(f, &role, 12.5, if m.group.is_some() { me.colour } else { MUTED }));
                });
                // ability as five dots
                row.spawn(Node { column_gap: Val::Px(3.0), ..default() }).with_children(|dots| {
                    let full = (m.skill * 5.0).round() as i32;
                    for i in 0..5 {
                        dots.spawn((Node { width: Val::Px(8.0), height: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }, BackgroundColor(if i < full { me.colour } else { RULE })));
                    }
                });
            });
            if open {
                block.spawn(text(f, "Put in charge of", 12.0, MUTED));
                block.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(4.0), row_gap: Val::Px(4.0), ..default() }).with_children(|chips| {
                    for g in Group::ALL.iter().copied().map(Some).chain([None]) {
                        let label = match g {
                            Some(g) => match me.works_by_group[g.index()] {
                                0 => short_group(g).to_string(),
                                n => format!("{} \u{00b7} {}", short_group(g), n),
                            },
                            None => "Nothing".to_string(),
                        };
                        let on = m.group == g;
                        let mut e = chips.spawn((
                            Node { padding: UiRect::axes(Val::Px(9.0), Val::Px(3.0)), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() },
                            BackgroundColor(if on { me.colour } else { crate::ui::BTN }),
                            Interaction::default(),
                            bevy::ui::FocusPolicy::Block,
                            AssignButton { member: m.index, group: g },
                        ));
                        if !on {
                            e.insert(crate::ui::Hoverable);
                        }
                        e.with_children(|c| {
                            c.spawn(text(f, &label, 12.5, if on { Color::WHITE } else { INK }));
                        });
                    }
                });
                block.spawn(text(f, "The number is how many of the family's works are in that trade.", 11.5, MUTED));
            }
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

fn short_group(g: Group) -> &'static str {
    match g {
        Group::Iron => "Iron",
        Group::Textiles => "Textiles",
        Group::CoalMining => "Mining",
        Group::Engineering => "Engineering",
        Group::Building => "Building",
        Group::Estates => "Estates",
    }
}

/// A works in the holdings list: jump to it on the map.
#[derive(Component, Clone, Copy)]
pub struct GotoFirm(pub usize);

pub fn holdings_panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, fam: Option<&FamilyView>, r: &Replay, k: usize) {
    let Some(me) = fam.and_then(|v| v.player()) else { return no_family(p, f, "property") };
    p.spawn(text(f, &format!("{} works and {} squares of land, bringing in {} last year.", me.works, me.land, money(me.income)), 14.0, MUTED));
    heading(p, f, "Works");
    if me.holdings.is_empty() {
        p.spawn(text(f, "None yet. Click a square on the map to see what could be built there.", 14.0, MUTED));
    }
    let mut hs: Vec<(usize, f32)> = me.holdings.iter().map(|&h| (h, crate::replay::firm_k(r, h, k))).collect();
    hs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
    for &(h, kk) in hs.iter().take(60) {
        let Some(firm) = r.header.firms.get(h) else { continue };
        let m = &r.header.methods[firm.method];
        let town = r.header.towns.get(firm.town.max(0) as usize).map(|t| t.name.as_str()).unwrap_or("the country");
        let open = kk > 0.0;
        p.spawn((
            Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)), border: UiRect::bottom(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
            BorderColor::all(RULE),
            BackgroundColor(Color::NONE),
            Interaction::default(),
            GotoFirm(h),
        ))
        .with_children(|row| {
            row.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|col| {
                col.spawn(text(f, &firm.name, 15.0, if open { INK } else { MUTED }));
                let since = if firm.born > r.y0 { format!(", since {}", firm.born) } else { String::new() };
                col.spawn(text(f, &format!("{} at {}{}", m.label, town, since), 12.5, MUTED));
            });
            let size = if open { format!("size {:.1}", kk / m.k0.max(1e-6)) } else if firm.born > r.y0 + k as i32 { "building".to_string() } else { "closed".to_string() };
            row.spawn(text(f, &size, 13.0, if open { INK } else { MUTED }));
        });
    }
    if me.land > 0 {
        heading(p, f, "Land");
        p.spawn(text(f, &format!("{} squares, each 2 km across. Their rents come to the family; whatever is built on them pays the family for the ground.", me.land), 13.0, MUTED));
    }
}
