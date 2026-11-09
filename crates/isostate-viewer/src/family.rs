//! The families as the interface sees them: who they are, what they own and
//! how they stand against each other. The live game fills a `FamilyView`
//! each year from the simulation's reports; a plain replay has none, and the
//! panels say so.
//!
//! Everything the player can change here is a choice from a short list: a
//! post for a member, what a branch does, which option of a decision. There
//! are no amounts to type and no sliders.

// see the note at the top of ui.rs
#![allow(deprecated)]

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use isostate_core::families::{Group, Path, Post, Posting, Stance, N_GROUPS};
use isostate_core::{BranchReport, Decision, FamilyReport, MemberReport};

use crate::replay::Replay;
use crate::ui::{money, text, BAD, BTN, GOOD, INK, MUTED, PAPER_2, RULE};

/// Banner colours, in the order families are given them.
pub const BANNERS: [u32; 7] = [0xb23a3a, 0x2f5fa8, 0x3b7d3e, 0xc9952b, 0x7a4a9a, 0x2b8a86, 0xd06a2a];

pub fn banner(i: usize) -> Color {
    let h = BANNERS[i % BANNERS.len()];
    Color::srgb_u8((h >> 16) as u8, (h >> 8) as u8, h as u8)
}

#[derive(Clone, Debug)]
pub struct FamilyRow {
    pub name: String,
    pub seat: usize,
    pub seat_name: String,
    pub colour: Color,
    pub cash: f32,
    pub net_worth: f32,
    pub income: f32,
    pub works: usize,
    /// squares of land held outright
    pub land: usize,
    /// net worth at the end of each year so far
    pub history: Vec<f32>,
    /// everything the simulation says about the family
    pub rep: FamilyReport,
}

#[derive(Resource, Default, Clone)]
pub struct FamilyView {
    pub year: i32,
    pub families: Vec<FamilyRow>,
    pub player: Option<usize>,
    /// the player can give orders (a live game, not a replay)
    pub live: bool,
    /// what new works of each trade would earn in each town, by town index
    pub prospects: Vec<[f64; N_GROUPS]>,
    /// the trade of every method's works (none for shops and trades)
    pub method_group: Vec<Option<Group>>,
}

impl FamilyView {
    pub fn player(&self) -> Option<&FamilyRow> {
        self.player.and_then(|i| self.families.get(i))
    }

    /// The best prospect of a trade in a town, if any works of it would pay there.
    pub fn prospect(&self, town: usize, g: Group) -> Option<f64> {
        self.prospects.get(town).map(|p| p[g.index()]).filter(|v| v.is_finite())
    }
}

/// A member's row in the family panel: click to see where they could be posted.
#[derive(Component, Clone, Copy)]
pub struct MemberRow(pub usize);

/// Post a member (from the family panel or a town's note).
#[derive(Component, Clone, Copy)]
pub struct PostButton {
    pub member: usize,
    pub posting: Posting,
}

/// Set what a branch does.
#[derive(Component, Clone, Copy)]
pub struct StanceButton {
    pub group: Group,
    pub town: usize,
    pub stance: Stance,
}

/// Open the card for a decision.
#[derive(Component, Clone, Copy)]
pub struct OpenDecision(pub u64);

/// A rival in the list of great families: click to see more of it.
#[derive(Component, Clone, Copy)]
pub struct RivalRow(pub usize);

/// A works in the holdings list: jump to it on the map.
#[derive(Component, Clone, Copy)]
pub struct GotoFirm(pub usize);

/// A town named in a panel: jump to it on the map.
#[derive(Component, Clone, Copy)]
pub struct GotoTown(pub usize);

// ------------------------------------------------------------------ words for things

pub fn short_group(g: Group) -> &'static str {
    match g {
        Group::Iron => "Iron",
        Group::Textiles => "Textiles",
        Group::CoalMining => "Mining",
        Group::Engineering => "Engineering",
        Group::Building => "Building",
        Group::Estates => "Estates",
    }
}

/// "iron and steel", for the middle of a sentence.
pub fn trade(g: Group) -> String {
    g.label().to_lowercase()
}

pub fn ordinal(n: usize) -> String {
    let s = match (n % 10, n % 100) {
        (1, x) if x != 11 => "st",
        (2, x) if x != 12 => "nd",
        (3, x) if x != 13 => "rd",
        _ => "th",
    };
    format!("{n}{s}")
}

/// What a member is doing, in a few words.
pub fn post_words(m: &MemberReport, fam: &FamilyReport) -> String {
    match m.post {
        Post::Head => format!("Heads the family at {}", fam.seat_name),
        Post::Manager { group } => format!("Runs {} at {}", trade(group), m.town_name),
        Post::Learning { path: Path::Apprentice { group, .. }, until } => format!("Apprenticed in {} at {} until {}", trade(group), m.town_name, until),
        Post::Learning { path: Path::University, until } => format!("At university until {until}"),
        Post::Child => format!("A child at {}", m.town_name),
        Post::Unposted => format!("No post, living at {}", m.town_name),
    }
}

/// A yearly return in words: snapshot returns above a half are scarcity
/// prices that will not last, and are shown as such.
pub fn yearly(v: f64) -> String {
    if v > 0.6 { "over 60% a year while prices last".to_string() } else { format!("about {:.0}% a year", 100.0 * v) }
}

/// ", 6% cheaper to run" for works, ", 6% better yields" for land.
pub fn bonus_words(g: Group, bonus: f64) -> String {
    if bonus <= 0.004 {
        String::new()
    } else if g == Group::Estates {
        format!(", {:.0}% better yields", 100.0 * bonus)
    } else {
        format!(", {:.0}% cheaper to run", 100.0 * bonus)
    }
}

/// A member posted to a branch that someone else runs: nothing to do there.
pub fn spare(m: &MemberReport, fam: &FamilyReport) -> bool {
    match m.post {
        Post::Manager { group } => branch_of(fam, group, m.town).is_some_and(|b| b.runner.is_some() && b.runner != Some(m.index)),
        Post::Unposted => m.alive && m.age >= 18,
        _ => false,
    }
}

/// A member who could be given a post now.
pub fn postable(m: &MemberReport) -> bool {
    m.alive && m.age >= 18 && matches!(m.post, Post::Unposted | Post::Manager { .. })
}

fn branch_of<'a>(fam: &'a FamilyReport, g: Group, t: usize) -> Option<&'a BranchReport> {
    fam.branches.iter().find(|b| b.group == g && b.town == t)
}

/// Where a member could usefully be posted: works with nobody of the family
/// over them first, then the towns where new works of a known trade would earn
/// most. Each with a line saying why.
pub fn suggestions(v: &FamilyView, r: &Replay, fam: &FamilyReport, m: &MemberReport, n: usize) -> Vec<(Posting, String, String)> {
    let mut out: Vec<(f64, Posting, String, String)> = Vec::new();
    let town_name = |t: usize| r.header.towns.get(t).map(|x| x.name.clone()).unwrap_or_else(|| format!("town {t}"));
    // absentee works, by trade and town
    let mut idle: std::collections::BTreeMap<(Group, usize), usize> = Default::default();
    for &f in &fam.absentee {
        let Some(firm) = r.header.firms.get(f) else { continue };
        if let Some(g) = v.method_group.get(firm.method).copied().flatten() {
            *idle.entry((g, firm.town.max(0) as usize)).or_default() += 1;
        }
    }
    for &g in &fam.known {
        for t in 0..v.prospects.len().min(r.header.towns.len()) {
            if t == fam.seat {
                // the head already runs every known trade at the seat
                continue;
            }
            let run_by_other = branch_of(fam, g, t).is_some_and(|b| b.runner.is_some() && b.runner != Some(m.index));
            if run_by_other || matches!(m.post, Post::Manager { group } if group == g && m.town == t) {
                continue;
            }
            let own = m.trade == Some(g);
            let idle_here = idle.get(&(g, t)).copied().unwrap_or(0);
            let p = v.prospect(t, g);
            let (score, why) = if idle_here > 0 {
                (10.0 + idle_here as f64, format!("{} of the family's works there have nobody of the family over them", idle_here))
            } else if let Some(p) = p.filter(|&p| p > 0.04) {
                let what = if g == Group::Estates { format!("land there yields {}", yearly(p)) } else { format!("new works there would earn {}", yearly(p)) };
                (p.min(0.6), what)
            } else {
                continue;
            };
            let mut why = why;
            if own {
                why.push_str(&format!("; {} was apprenticed in it", m.first));
            }
            let label = format!("{} at {}", short_group(g), town_name(t));
            out.push((score + if own { 0.03 } else { 0.0 }, Posting::Manager { group: g, town: t }, label, why));
        }
    }
    out.sort_by(|a, b| b.0.total_cmp(&a.0));
    out.into_iter().take(n).map(|(_, p, l, w)| (p, l, w)).collect()
}

// ------------------------------------------------------------------ small pieces

fn heading(p: &mut ChildSpawnerCommands, f: &Handle<Font>, s: &str) {
    p.spawn((text(f, s, 17.0, INK), Node { margin: UiRect::top(Val::Px(12.0)), ..default() }));
}

fn no_family(p: &mut ChildSpawnerCommands, f: &Handle<Font>, what: &str) {
    p.spawn((
        Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(6.0), padding: UiRect::all(Val::Px(12.0)), margin: UiRect::top(Val::Px(6.0)), border_radius: BorderRadius::all(Val::Px(12.0)), ..default() },
        BackgroundColor(PAPER_2),
    ))
    .with_children(|c| {
        c.spawn(text(f, "This is a recorded history", 16.0, INK));
        c.spawn(text(f, &format!("Nobody plays a family in a replay, so there is no {what} to show. Start a game to found a family; its works, land and people appear here."), 13.5, MUTED));
    });
}

/// Ability as five dots.
pub fn pips(p: &mut ChildSpawnerCommands, skill: f64, colour: Color) {
    p.spawn(Node { column_gap: Val::Px(3.0), flex_shrink: 0.0, ..default() }).with_children(|dots| {
        let full = (skill * 5.0).round().clamp(1.0, 5.0) as i32;
        for i in 0..5 {
            dots.spawn((Node { width: Val::Px(8.0), height: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() }, BackgroundColor(if i < full { colour } else { RULE })));
        }
    });
}

/// A rounded chip button.
pub fn chip_button<C: Component>(p: &mut ChildSpawnerCommands, f: &Handle<Font>, label: &str, on: bool, accent: Color, marker: C) {
    let mut e = p.spawn((
        Node { padding: UiRect::axes(Val::Px(9.0), Val::Px(3.0)), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() },
        BackgroundColor(if on { accent } else { BTN }),
        Interaction::default(),
        FocusPolicy::Block,
        marker,
    ));
    if !on {
        e.insert(crate::ui::Hoverable);
    }
    e.with_children(|c| {
        c.spawn(text(f, label, 12.5, if on { Color::WHITE } else { INK }));
    });
}

fn tag(p: &mut ChildSpawnerCommands, f: &Handle<Font>, label: &str, colour: Color) {
    p.spawn((Node { padding: UiRect::axes(Val::Px(6.0), Val::Px(0.0)), border_radius: BorderRadius::all(Val::Px(7.0)), flex_shrink: 0.0, ..default() }, BackgroundColor(colour.with_alpha(0.14))))
        .with_children(|t| {
            t.spawn(text(f, label, 11.5, colour));
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

// ------------------------------------------------------------------ the family tab

pub fn family_panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, fam: Option<&FamilyView>, r: &Replay, k: usize, expanded: Option<usize>, rival: Option<usize>) {
    let Some(v) = fam else { return no_family(p, f, "family") };
    let Some(me) = v.player() else { return no_family(p, f, "family") };
    let rep = &me.rep;
    let accent = me.colour;
    let mut rank: Vec<usize> = (0..v.families.len()).collect();
    rank.sort_by(|&a, &b| v.families[b].net_worth.total_cmp(&v.families[a].net_worth));
    let my_rank = rank.iter().position(|&i| Some(i) == v.player).unwrap_or(0) + 1;
    p.spawn(text(f, &format!("Worth {}, the {} fortune of the {} great families.", money(me.net_worth), ordinal(my_rank), v.families.len()), 14.0, MUTED));
    sparkline(p, f, &me.history, accent);
    let change = if me.history.len() >= 2 { me.history[me.history.len() - 1] - me.history[me.history.len() - 2] } else { 0.0 };
    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|row| {
        row.spawn(text(f, &format!("{} in hand", money(me.cash)), 14.0, INK));
        row.spawn(text(f, &format!("{}{} this year", if change >= 0.0 { "+" } else { "" }, money(change)), 14.0, if change >= 0.0 { GOOD } else { BAD }));
    });

    // the seat and the trades
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), margin: UiRect::top(Val::Px(8.0)), ..default() }).with_children(|c| {
        c.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0), flex_wrap: FlexWrap::Wrap, ..default() }).with_children(|row| {
            row.spawn(text(f, "Seat", 13.5, MUTED));
            chip_button(row, f, &rep.seat_name, true, accent, GotoTown(rep.seat));
            if let Some(next) = rep.next_seat {
                let name = r.header.towns.get(next).map(|t| t.name.as_str()).unwrap_or("?");
                row.spawn(text(f, &format!("moving to {name} at the year end"), 13.0, accent));
            }
        });
        c.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0), flex_wrap: FlexWrap::Wrap, row_gap: Val::Px(4.0), ..default() }).with_children(|row| {
            row.spawn(text(f, "Trades", 13.5, MUTED));
            for &g in &rep.known {
                tag(row, f, short_group(g), accent);
            }
            let learning: Vec<Group> = rep
                .members
                .iter()
                .filter(|m| m.alive)
                .filter_map(|m| if let Post::Learning { path: Path::Apprentice { group, .. }, .. } = m.post { Some(group) } else { None })
                .filter(|g| !rep.known.contains(g))
                .collect();
            for g in learning {
                tag(row, f, &format!("{} soon", short_group(g)), MUTED);
            }
        });
    });

    let idle: Vec<&MemberReport> = rep.members.iter().filter(|m| m.alive && spare(m, rep)).collect();
    if v.live && (!rep.pending.is_empty() || !idle.is_empty()) {
        heading(p, f, "Waiting for you");
        for d in &rep.pending {
            decision_line(p, f, d, accent);
        }
        if !idle.is_empty() {
            let names: Vec<String> = idle.iter().map(|m| m.first.clone()).collect();
            let verb = if idle.len() == 1 { "has" } else { "have" };
            p.spawn((
                Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)), border_radius: BorderRadius::all(Val::Px(10.0)), border: UiRect::left(Val::Px(3.0)), ..default() },
                BackgroundColor(PAPER_2),
                BorderColor::all(accent),
            ))
            .with_children(|c| {
                c.spawn(text(f, &format!("{} {verb} nothing to run", join_words(&names)), 14.0, INK));
                c.spawn(text(f, "Click a name below to post them where they would count.", 12.0, MUTED));
            });
        }
    }

    heading(p, f, "Last year's accounts");
    let line = |p: &mut ChildSpawnerCommands, label: &str, v: f64, sign: f64| {
        p.spawn(Node { justify_content: JustifyContent::SpaceBetween, padding: UiRect::vertical(Val::Px(1.0)), ..default() }).with_children(|row| {
            row.spawn(text(f, label, 13.5, INK));
            let s = if v.abs() < 0.5 { "\u{2013}".to_string() } else if sign > 0.0 { money(v as f32) } else { format!("\u{2212}{}", money(v as f32)) };
            row.spawn(text(f, &s, 13.5, if sign > 0.0 || v.abs() < 0.5 { INK } else { MUTED }));
        });
    };
    let ly = &rep.last_year;
    for g in Group::ALL {
        if ly.dividends[g.index()].abs() >= 0.5 {
            line(p, &format!("Dividends: {}", trade(g)), ly.dividends[g.index()], 1.0);
        }
    }
    if ly.dividends.iter().all(|d| d.abs() < 0.5) {
        line(p, "Dividends from the works", ly.other_dividends, 1.0);
    }
    line(p, "Rents from the land", ly.rents, 1.0);
    if ly.sales > 0.5 {
        line(p, "Works and land sold", ly.sales, 1.0);
    }
    line(p, "Spent on living", ly.spending, -1.0);
    line(p, "Put into works and land", ly.invested, -1.0);

    heading(p, f, "The family");
    p.spawn(text(f, "The head runs every trade at the seat. Anyone else of age can be posted to run a trade in another town; whoever runs a branch makes its works cheaper to run, the abler the more.", 12.5, MUTED));
    // the family tree: the founders, each followed by their children, indented under them
    let mut roots: Vec<&MemberReport> = rep.members.iter().filter(|m| m.parent.is_none_or(|j| j >= rep.members.len())).collect();
    roots.sort_by_key(|m| -m.age);
    for m in roots {
        tree(p, f, v, r, me, m, expanded, 0);
    }
    heading(p, f, "The great families");
    p.spawn(text(f, "Click a family to see how it is doing.", 12.0, MUTED));
    for (pos, &i) in rank.iter().enumerate() {
        let fr = &v.families[i];
        let mine = Some(i) == v.player;
        let open = rival == Some(i);
        p.spawn((
            Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), padding: UiRect::axes(Val::Px(4.0), Val::Px(3.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
            BackgroundColor(if open { PAPER_2 } else { Color::NONE }),
        ))
        .with_children(|block| {
            block
                .spawn((Node { align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() }, Interaction::default(), RivalRow(i)))
                .with_children(|row| {
                    row.spawn((text(f, &format!("{}", pos + 1), 13.0, MUTED), Node { width: Val::Px(16.0), ..default() }));
                    row.spawn((Node { width: Val::Px(12.0), height: Val::Px(14.0), border_radius: BorderRadius::new(Val::Px(2.0), Val::Px(2.0), Val::Px(6.0), Val::Px(6.0)), ..default() }, BackgroundColor(fr.colour)));
                    row.spawn(Node { flex_direction: FlexDirection::Column, flex_grow: 1.0, ..default() }).with_children(|col| {
                        col.spawn(text(f, &fr.name, 15.0, if mine { fr.colour } else { INK }));
                        col.spawn(text(f, &format!("of {}", fr.seat_name), 11.5, MUTED));
                    });
                    row.spawn((text(f, &format!("{} works", fr.works), 13.0, MUTED), Node { width: Val::Px(64.0), ..default() }));
                    row.spawn((text(f, &money(fr.net_worth), 14.0, INK), Node { width: Val::Px(64.0), justify_content: JustifyContent::End, ..default() }));
                });
            if open {
                rival_details(block, f, r, fr);
            }
        });
    }
    let _ = k;
}

/// A member and, indented under them, their children (the living as full
/// cards, the dead as a line).
#[allow(clippy::too_many_arguments)]
fn tree(p: &mut ChildSpawnerCommands, f: &Handle<Font>, v: &FamilyView, r: &Replay, me: &FamilyRow, m: &MemberReport, expanded: Option<usize>, depth: usize) {
    let rep = &me.rep;
    if m.alive {
        member_card(p, f, v, r, me, m, v.live && expanded == Some(m.index));
    } else {
        p.spawn((Node { padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)), ..default() },)).with_children(|row| {
            row.spawn(text(f, &format!("{} {} \u{2020}, died at {}", m.first, rep.surname, m.age), 12.5, MUTED));
        });
    }
    let mut kids: Vec<&MemberReport> = rep.members.iter().filter(|c| c.parent == Some(m.index)).collect();
    if kids.is_empty() || depth > 8 {
        return;
    }
    kids.sort_by_key(|c| -c.age);
    p.spawn((
        Node { flex_direction: FlexDirection::Column, margin: UiRect::left(Val::Px(10.0)), padding: UiRect::left(Val::Px(8.0)), border: UiRect::left(Val::Px(2.0)), ..default() },
        BorderColor::all(RULE),
    ))
    .with_children(|c| {
        for k in kids {
            tree(c, f, v, r, me, k, expanded, depth + 1);
        }
    });
}

fn decision_line(p: &mut ChildSpawnerCommands, f: &Handle<Font>, d: &Decision, accent: Color) {
    p.spawn((
        Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(8.0), padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)), border_radius: BorderRadius::all(Val::Px(10.0)), border: UiRect::left(Val::Px(3.0)), ..default() },
        BackgroundColor(PAPER_2),
        BorderColor::all(accent),
    ))
    .with_children(|row| {
        row.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 1.0, ..default() }).with_children(|col| {
            col.spawn(text(f, &d.prompt, 14.0, INK));
            let def = d.options.get(d.default).map(|o| o.label.as_str()).unwrap_or("");
            col.spawn(text(f, &format!("Otherwise, at the end of {}: {}", d.due, def.to_lowercase()), 12.0, MUTED));
        });
        chip_button(row, f, "Choose", true, accent, OpenDecision(d.id));
    });
}

fn member_card(p: &mut ChildSpawnerCommands, f: &Handle<Font>, v: &FamilyView, r: &Replay, me: &FamilyRow, m: &MemberReport, open: bool) {
    let rep = &me.rep;
    let accent = me.colour;
    let can_post = v.live && postable(m);
    if m.post == Post::Child {
        // children are a line: the decision about them comes at fourteen
        p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)), ..default() }).with_children(|row| {
            row.spawn(text(f, &format!("{} {}, {}", m.first, rep.surname, m.age), 13.5, INK));
            pips(row, m.skill, accent.with_alpha(0.6));
        });
        return;
    }
    p.spawn((
        Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(6.0), padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)), border: UiRect::bottom(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() },
        BorderColor::all(RULE),
        BackgroundColor(if open { PAPER_2 } else { Color::NONE }),
    ))
    .with_children(|block| {
        let mut head = block.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() });
        if can_post {
            head.insert((Interaction::default(), MemberRow(m.index)));
        }
        head.with_children(|row| {
            row.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 1.0, ..default() }).with_children(|col| {
                col.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0), ..default() }).with_children(|name| {
                    name.spawn(text(f, &format!("{} {}, {}", m.first, rep.surname, m.age), 15.0, INK));
                    if m.post == Post::Head {
                        tag(name, f, "head", accent);
                    }
                    if m.educated {
                        tag(name, f, "educated", MUTED);
                    }
                });
                let mut words = post_words(m, rep);
                // the bonus of what the member actually runs
                if let Some(b) = rep.branches.iter().filter(|b| b.runner == Some(m.index)).max_by(|a, b| a.bonus.total_cmp(&b.bonus)) {
                    words.push_str(&bonus_words(b.group, b.bonus));
                }
                let idle = spare(m, rep);
                if let (true, Post::Manager { group }) = (idle, m.post) {
                    let runner = branch_of(rep, group, m.town).and_then(|b| b.runner).and_then(|i| rep.members.get(i)).map(|x| x.first.clone()).unwrap_or_default();
                    words = format!("At {} with nothing to run: {} runs the {} there", m.town_name, runner, trade(group));
                }
                let busy = matches!(m.post, Post::Head | Post::Manager { .. }) && !idle;
                col.spawn(text(f, &words, 12.5, if busy { accent } else { MUTED }));
                if let Some((t, post)) = m.next {
                    let town = r.header.towns.get(t).map(|x| x.name.as_str()).unwrap_or("?");
                    let what = match post {
                        Post::Manager { group } => format!("to run {}", trade(group)),
                        _ => String::new(),
                    };
                    col.spawn(text(f, &format!("Moves to {town} {what} at the year end"), 12.5, accent));
                }
                if let Some(g) = m.trade {
                    if !matches!(m.post, Post::Learning { .. }) {
                        col.spawn(text(f, &format!("Trained in {}", trade(g)), 11.5, MUTED));
                    }
                }
                if can_post && !open {
                    col.spawn(text(f, if m.post == Post::Unposted { "Click to give a post." } else { "Click to post elsewhere." }, 11.5, MUTED));
                }
            });
            pips(row, m.skill, accent);
        });
        if open {
            block.spawn(text(f, "Put in charge of", 12.0, MUTED));
            let sugg = suggestions(v, r, rep, m, 6);
            if sugg.is_empty() {
                block.spawn(text(f, "Nowhere stands out just now. A town's note on the map lists every trade there.", 12.0, MUTED));
            }
            for (posting, label, why) in sugg {
                block
                    .spawn((
                        Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(Val::Px(9.0), Val::Px(5.0)), border_radius: BorderRadius::all(Val::Px(9.0)), ..default() },
                        BackgroundColor(BTN),
                        Interaction::default(),
                        FocusPolicy::Block,
                        crate::ui::Hoverable,
                        PostButton { member: m.index, posting },
                    ))
                    .with_children(|b| {
                        b.spawn(text(f, &label, 14.0, INK));
                        b.spawn(text(f, &why, 11.5, MUTED));
                    });
            }
            block.spawn(Node { column_gap: Val::Px(6.0), align_items: AlignItems::Center, ..default() }).with_children(|row| {
                if m.post != Post::Unposted {
                    chip_button(row, f, "Take off the post", false, accent, PostButton { member: m.index, posting: Posting::Unposted });
                }
                row.spawn(text(f, "or click a town on the map", 11.5, MUTED));
            });
        }
    });
}

/// No fog: what a rival family is and does, for anyone to see.
fn rival_details(p: &mut ChildSpawnerCommands, f: &Handle<Font>, r: &Replay, fr: &FamilyRow) {
    let rep = &fr.rep;
    let head = rep.head.and_then(|h| rep.members.get(h));
    let living = rep.members.iter().filter(|m| m.alive).count();
    let mut lines = Vec::new();
    if let Some(h) = head {
        lines.push(format!("Headed by {} {}, {}, from {}.", h.first, rep.surname, h.age, rep.seat_name));
    }
    let trades: Vec<String> = rep.known.iter().map(|&g| trade(g)).collect();
    lines.push(format!("Knows {}.", join_words(&trades)));
    lines.push(format!("{} of the family living; {} branches; {} squares of land.", living, rep.branches.len(), rep.cells.len()));
    let earned = rep.last_year.earned();
    lines.push(format!("Earned {} last year and put {} into works and land.", money(earned as f32), money(rep.last_year.invested as f32)));
    for l in lines {
        p.spawn(text(f, &l, 12.5, MUTED));
    }
    p.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(4.0), row_gap: Val::Px(4.0), ..default() }).with_children(|row| {
        for b in rep.branches.iter().take(8) {
            let town = r.header.towns.get(b.town).map(|t| t.name.as_str()).unwrap_or("?");
            tag(row, f, &format!("{} at {}", short_group(b.group), town), fr.colour);
        }
    });
}

/// Small numbers in words, for the start of a sentence; larger ones as figures.
pub fn number_word(n: usize) -> String {
    const W: [&str; 13] = ["no", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve"];
    W.get(n).map(|s| s.to_string()).unwrap_or_else(|| n.to_string())
}

pub fn capitalised(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

pub fn join_words(items: &[String]) -> String {
    match items.len() {
        0 => "nothing yet".to_string(),
        1 => items[0].clone(),
        n => format!("{} and {}", items[..n - 1].join(", "), items[n - 1]),
    }
}

// ------------------------------------------------------------------ the holdings tab

pub fn holdings_panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, fam: Option<&FamilyView>, r: &Replay, k: usize) {
    let Some(v) = fam else { return no_family(p, f, "property") };
    let Some(me) = v.player() else { return no_family(p, f, "property") };
    let rep = &me.rep;
    let accent = me.colour;
    p.spawn(text(f, &format!("{} works and {} squares of land, bringing in {} last year.", me.works, me.land, money(me.income)), 14.0, MUTED));

    heading(p, f, "Branches");
    p.spawn(text(f, "A branch is a trade in a town with someone of the family there to run it. Growing, it builds a works a year (for estates, buys land); winding down, it sells one.", 12.5, MUTED));
    if rep.branches.is_empty() {
        p.spawn(text(f, "None yet.", 13.5, MUTED));
    }
    let mut branches: Vec<&BranchReport> = rep.branches.iter().collect();
    branches.sort_by_key(|b| (b.town != rep.seat, b.town_name.clone(), b.group));
    for b in branches {
        let runner = b.runner.and_then(|i| rep.members.get(i)).map(|m| format!("{} {}", m.first, rep.surname)).unwrap_or_else(|| "nobody".to_string());
        p.spawn((Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)), border: UiRect::bottom(Val::Px(1.0)), ..default() }, BorderColor::all(RULE)))
            .with_children(|block| {
                block.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() }).with_children(|row| {
                    row.spawn(Node { flex_direction: FlexDirection::Column, flex_shrink: 1.0, ..default() }).with_children(|col| {
                        col.spawn(text(f, &format!("{} at {}", b.group.label(), b.town_name), 15.0, INK));
                        let size = if b.group == Group::Estates { format!("{} squares", b.cells) } else { format!("{} works, {} last year", b.works.len(), money(b.last_year_profit as f32)) };
                        let bonus = bonus_words(b.group, b.bonus);
                        col.spawn(text(f, &format!("Run by {runner}{bonus}. {size}."), 12.5, MUTED));
                    });
                });
                if v.live {
                    block.spawn(Node { column_gap: Val::Px(4.0), ..default() }).with_children(|row| {
                        for (s, label) in [(Stance::Grow, "Grow"), (Stance::Hold, "Hold"), (Stance::WindDown, "Wind down")] {
                            chip_button(row, f, label, b.stance == s, accent, StanceButton { group: b.group, town: b.town, stance: s });
                        }
                    });
                }
            });
    }

    if !rep.absentee.is_empty() {
        heading(p, f, "Works with nobody of the family");
        p.spawn(text(f, "Nobody of the family runs these, so a hired manager does, for a fifth of their profit. Post someone to their town to bring them back under the family.", 12.5, MUTED));
        for &h in &rep.absentee {
            works_row(p, f, r, h, k);
        }
    }

    heading(p, f, "Works");
    if rep.firms.is_empty() {
        p.spawn(text(f, "None yet. At the seat, click a square on the map to see what could be built there.", 14.0, MUTED));
    }
    let mut hs: Vec<(usize, f32)> = rep.firms.iter().filter(|h| !rep.absentee.contains(h)).map(|&h| (h, crate::replay::firm_k(r, h, k))).collect();
    hs.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    for &(h, _) in hs.iter().take(60) {
        works_row(p, f, r, h, k);
    }
    if me.land > 0 {
        heading(p, f, "Land");
        p.spawn(text(f, &format!("{} squares, each 2 km across. Their rents come to the family; whatever is built on them pays the family for the ground.", me.land), 13.0, MUTED));
    }
    let _ = accent;
}

fn works_row(p: &mut ChildSpawnerCommands, f: &Handle<Font>, r: &Replay, h: usize, k: usize) {
    let Some(firm) = r.header.firms.get(h) else { return };
    let kk = crate::replay::firm_k(r, h, k);
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
