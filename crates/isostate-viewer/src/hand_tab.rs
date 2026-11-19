//! The sandbox's Hand tab (`docs/designs/sandbox.md`, 4.6): everything done by
//! hand, newest first, each with its month and its line, read from the
//! island's news of kind "hand". A row about a place is a button that takes
//! the view there and opens the note about it, as a works in the holdings list
//! does (`game::goto_clicks`), without counting as a click on the map, so that
//! a tool in hand is not used by it.
//!
//! `panel` is called by `ui::panel_rebuild` with `HandParams`, which this file
//! owns: a later stage (the ledger at the foot, stage 5) adds what it needs to
//! read there and does not come into ui.rs.

// see the note at the top of ui.rs
#![allow(deprecated)]

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::replay::{Event, Replay};
use crate::sandbox::HAND_INK;
use crate::ui::{text, INK, MUTED, PAPER_2, RULE};
use crate::{clock, world, Data, Orbit, Pick, Picked};

/// What the Hand tab reads beyond the run and the year on the clock.
#[derive(SystemParam)]
pub struct HandParams<'w> {
    /// present in a sandbox; the tab is on the bar only there
    pub sandbox: Option<Res<'w, crate::sandbox::Sandbox>>,
}

/// A row of the Hand tab about a place: the square the news is about.
#[derive(Component)]
pub struct HandRow(pub usize);

pub fn plugin(app: &mut App) {
    app.add_systems(Update, hand_clicks);
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
