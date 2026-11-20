//! A town's ideas on its card, in a sandbox: the first of the views that are
//! also a handle (the designer's note of 20 November 2026 in
//! `docs/BACKLOG.md`: "opening up a towns known ideas timeline should allow me
//! to click and discover things").
//!
//! Under the inspector's block, in a section of its own, a town's card shows
//! every idea of the data the game plays, each in one of three states, as the
//! game's thread tells them with the card (`inspect::Told::ideas`, made by
//! `inspect::town_ideas`): those the town knows, with the year it came to know
//! each; those known elsewhere on the island and not here, greyed; and those
//! found nowhere yet, fainter. First a strip from the run's first year to the
//! year in progress with a tick at each year the town learnt something, then
//! the ideas in groups as the tree groups them (the trades of the works, the
//! ways of working, the machines for tasks), each a chip.
//!
//! A click on a chip only chooses it: the line over the groups then says what
//! the simulation says of it (what it opened and how many works of it stand
//! here now; where it is known; or what it waits on). Only an idea found
//! nowhere yet has a second chip under that line, Discover it here, and only
//! that chip changes the island: it sends the hand's edit for an idea
//! (`sandbox::discover_edit`, the very edit the Idea tool makes) under the
//! tray's guards, and its answer goes up as the Idea tool's does. So the card
//! opened and read, and any chip chosen, issues no edit.
//!
//! An idea found by hand is carried out at once, between two months, as the
//! Idea tool's is: it is in the Hand tab's list and on the timeline at once,
//! and a kept session keeps it, since it is one more order in the log. It
//! does not wait for the year's turn, and it cannot be taken back, any more
//! than the Idea tool's can: the design takes back an edit carried out only at
//! its stage 8, which is not built.
//!
//! The view never waits: the ideas come with the card's answer, which the
//! thread makes between months (a few tenths of a millisecond more for a town
//! with sixty ideas), and the section is drawn again only when what it shows
//! differs from what is drawn, never every frame.

// see the note at the top of ui.rs
#![allow(deprecated)]

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use isostate_core::edits::Edit;
use isostate_core::inspect::{Ideas, Stands, Thing, TownIdea};

use crate::game::Game;
use crate::inspector::{at_present, thing_of, Answers};
use crate::live::Live;
use crate::sandbox::{self, Sandbox, Tool, HAND_INK};
use crate::ui::{text, BTN, INK, MUTED, RULE};
use crate::{Data, Picked, Timeline, Toasts, UiFont};

/// Where the section stands in the card: after the inspector's block.
#[derive(Component)]
pub struct IdeasBlock;

/// An idea's chip: its name in the data.
#[derive(Component)]
struct IdeaChip(String);

/// The chip that finds the idea chosen in the town by hand.
#[derive(Component)]
struct DiscoverHere;

/// The idea chosen on a town's card, by the town and the idea's name.
#[derive(Resource, Default)]
pub struct Chosen {
    pick: Option<(usize, String)>,
    version: u64,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Chosen>().add_systems(
        Update,
        // (after the answers of the frame are kept, as the inspector's own block is drawn)
        (chips, discover, draw).chain().after(crate::game::pump).run_if(resource_exists::<Sandbox>).run_if(resource_exists::<Live>),
    );
}

/// What a press on a chip of the section does: a chip only chooses an idea
/// (or lets it go, pressed again), and makes no edit; Discover it here makes
/// the hand's edit for the idea chosen, if it may be found here, and nothing
/// else ever does.
#[derive(Clone, Debug, PartialEq)]
enum Press {
    Choose(String),
    Discover,
}

fn edit_of(press: &Press, ideas: &Ideas, chosen: Option<&str>) -> Option<Edit> {
    match press {
        Press::Choose(_) => None,
        Press::Discover => {
            let idea = ideas.all().find(|i| Some(i.name.as_str()) == chosen)?;
            idea.discover.then(|| sandbox::discover_edit(idea.name.clone(), ideas.town))
        }
    }
}

/// The town picked, and its ideas as last told, while the clock is at the present.
fn shown<'a>(picked: &Picked, answers: &'a Answers, tl: &Timeline, data: &Data, live: &Live) -> Option<&'a Ideas> {
    let thing = picked.what.map(thing_of)?;
    if !matches!(thing, Thing::Town(_)) || !at_present(tl, data, live) {
        return None;
    }
    answers.get(thing)?.as_ref().ok()?.ideas.as_ref()
}

fn chips(q: Query<(&Interaction, &IdeaChip), Changed<Interaction>>, picked: Res<Picked>, answers: Res<Answers>, tl: Res<Timeline>, data: Res<Data>, live: Res<Live>, mut chosen: ResMut<Chosen>) {
    let Some(ideas) = shown(&picked, &answers, &tl, &data, &live) else { return };
    for (i, c) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        debug_assert!(edit_of(&Press::Choose(c.0.clone()), ideas, None).is_none());
        let again = chosen.pick.as_ref().is_some_and(|(t, n)| *t == ideas.town && *n == c.0);
        chosen.pick = if again { None } else { Some((ideas.town, c.0.clone())) };
        chosen.version += 1;
        if let Some(idea) = ideas.all().find(|x| x.name == c.0).filter(|_| !again) {
            println!("ideas: {}", idea.said);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn discover(
    q: Query<&Interaction, (Changed<Interaction>, With<DiscoverHere>)>,
    picked: Res<Picked>,
    answers: Res<Answers>,
    chosen: Res<Chosen>,
    mut sb: ResMut<Sandbox>,
    tl: Res<Timeline>,
    data: Res<Data>,
    live: Res<Live>,
    mut game: ResMut<Game>,
    mut toasts: ResMut<Toasts>,
    time: Res<Time>,
) {
    if !q.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    let Some(ideas) = shown(&picked, &answers, &tl, &data, &live) else { return };
    let name = chosen.pick.as_ref().filter(|(t, _)| *t == ideas.town).map(|(_, n)| n.as_str());
    let Some(edit) = edit_of(&Press::Discover, ideas, name) else { return };
    if let Err(why) = sandbox::send_as(Tool::Idea, edit, &mut sb, &tl, &data, &live, &mut game) {
        toasts.say("orders_refused", why, "By hand", time.elapsed_secs());
    }
}

/// What the section shows, to tell whether it must be drawn again: the
/// strip's ends, each chip, and the line of the idea chosen.
#[derive(Clone, PartialEq, Debug)]
struct Look {
    from: i64,
    to: i64,
    groups: Vec<(String, Vec<(String, String, Stands, Option<i64>, bool)>)>,
    /// the idea chosen: its name, its line and whether it may be found here
    chosen: Option<(String, String, bool)>,
}

impl Look {
    fn of(ideas: &Ideas, chosen: Option<&str>) -> Look {
        Look {
            from: ideas.from,
            to: ideas.to,
            groups: ideas
                .groups
                .iter()
                .map(|g| (g.name.clone(), g.ideas.iter().map(|i| (i.name.clone(), chip_words(i), i.stands, if i.stands == Stands::Here { i.year } else { None }, i.how.starts_with("found here"))).collect()))
                .collect(),
            chosen: chosen.and_then(|n| ideas.all().find(|i| i.name == n)).map(|i| (i.name.clone(), i.said.clone(), i.discover)),
        }
    }
}

/// The words on an idea's chip: the year it was learnt and its label for one
/// known here; with the number of towns that know it for one known elsewhere.
fn chip_words(i: &TownIdea) -> String {
    match (i.stands, i.year) {
        (Stands::Here, Some(y)) => format!("{y} {}", i.label),
        (Stands::Elsewhere, _) if i.towns > 0 => format!("{} ({})", i.label, if i.towns == 1 { "1 town".to_string() } else { format!("{} towns", i.towns) }),
        _ => i.label.clone(),
    }
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    font: Res<UiFont>,
    picked: Res<Picked>,
    answers: Res<Answers>,
    tl: Res<Timeline>,
    data: Res<Data>,
    live: Res<Live>,
    chosen: Res<Chosen>,
    q: Query<(Entity, Option<&Children>), With<IdeasBlock>>,
    mut drawn: Local<Option<Option<Look>>>,
) {
    let ideas = shown(&picked, &answers, &tl, &data, &live);
    let name = ideas.and_then(|x| chosen.pick.as_ref().filter(|(t, _)| *t == x.town)).map(|(_, n)| n.as_str());
    let look = ideas.map(|x| Look::of(x, name));
    if drawn.as_ref() == Some(&look) {
        return;
    }
    let Ok((e, children)) = q.single() else { return };
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    if let Some(l) = &look {
        let f = font.0.clone();
        commands.entity(e).with_children(|p| section(p, &f, l));
    }
    *drawn = Some(look);
}

/// The strip's width, inside the card.
const STRIP_W: f32 = 330.0;

fn section(p: &mut ChildSpawnerCommands, f: &Handle<Font>, l: &Look) {
    p.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(6.0)), ..default() }, BackgroundColor(RULE)));
    p.spawn(text(f, &format!("Its ideas, {} to {}", l.from, l.to), 15.5, INK));
    let count = |s: Stands| l.groups.iter().flat_map(|g| g.1.iter()).filter(|c| c.2 == s).count();
    p.spawn(text(f, &format!("{} known here, {} known elsewhere on the island (greyed), {} found nowhere yet (faint).", count(Stands::Here), count(Stands::Elsewhere), count(Stands::Nowhere)), 12.0, MUTED));
    strip(p, f, l);
    // the line of the idea chosen, and the one chip that changes the island
    match &l.chosen {
        Some((_, said, may)) => {
            p.spawn((text(f, said, 12.5, INK), Node { margin: UiRect::top(Val::Px(4.0)), ..default() }));
            if *may {
                p.spawn(Node { margin: UiRect::top(Val::Px(4.0)), ..default() }).with_children(|row| crate::family::chip_button(row, f, "Discover it here", true, HAND_INK, DiscoverHere));
            }
        }
        None => {
            p.spawn((text(f, "Click an idea to see what it opened, where it is known, or what it waits on; one found nowhere yet can then be discovered here by hand.", 11.5, MUTED), Node { margin: UiRect::top(Val::Px(4.0)), ..default() }));
        }
    }
    let chosen = l.chosen.as_ref().map(|c| c.0.as_str());
    for (name, ideas) in &l.groups {
        p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), margin: UiRect::top(Val::Px(6.0)), ..default() }).with_children(|col| {
            col.spawn(text(f, name, 13.0, INK));
            col.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(4.0), row_gap: Val::Px(3.0), ..default() }).with_children(|row| {
                for (idea, words, stands, _, _) in ideas {
                    chip(row, f, idea, words, *stands, chosen == Some(idea.as_str()));
                }
            });
        });
    }
}

/// A chip: ink on the buttons' paper for one known here, muted on a paler
/// paper for one known elsewhere, faint and unfilled with a thin rule round it
/// for one found nowhere; white on the hand's ink while chosen.
fn chip(p: &mut ChildSpawnerCommands, f: &Handle<Font>, name: &str, words: &str, stands: Stands, on: bool) {
    let (fill, ink, edge) = match (on, stands) {
        (true, _) => (HAND_INK, Color::WHITE, HAND_INK),
        (false, Stands::Here) => (BTN, INK, BTN),
        (false, Stands::Elsewhere) => (Color::srgb(0.93, 0.92, 0.89), MUTED, Color::srgb(0.93, 0.92, 0.89)),
        (false, Stands::Nowhere) => (Color::NONE, MUTED.with_alpha(0.6), RULE),
    };
    p.spawn((
        Node { padding: UiRect::axes(Val::Px(6.0), Val::Px(1.0)), border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() },
        BackgroundColor(fill),
        BorderColor::all(edge),
        Interaction::default(),
        FocusPolicy::Block,
        IdeaChip(name.to_string()),
    ))
    .with_children(|c| {
        c.spawn(text(f, words, 11.5, ink));
    });
}

/// The strip from the run's first year to the year in progress: a rule with
/// a tick at each year the town came to know an idea (taller, in the hand's
/// ink, where it was found here), and the years at the ends and each
/// half-century between.
fn strip(p: &mut ChildSpawnerCommands, f: &Handle<Font>, l: &Look) {
    let span = (l.to - l.from).max(1) as f32;
    let x = |y: i64| ((y - l.from) as f32 / span).clamp(0.0, 1.0) * STRIP_W;
    p.spawn(Node { width: Val::Px(STRIP_W), height: Val::Px(34.0), margin: UiRect::top(Val::Px(4.0)), ..default() }).with_children(|s| {
        s.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(14.0), width: Val::Px(STRIP_W), height: Val::Px(1.0), ..default() }, BackgroundColor(MUTED)));
        for (_, ideas) in &l.groups {
            for (_, _, stands, year, found) in ideas {
                let (Stands::Here, Some(y)) = (stands, year) else { continue };
                let (h, colour) = if *found { (14.0, HAND_INK) } else { (8.0, INK.with_alpha(0.7)) };
                s.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(x(*y) - 0.75), top: Val::Px(14.0 - h), width: Val::Px(1.5), height: Val::Px(h), ..default() }, BackgroundColor(colour)));
            }
        }
        let mut marks = vec![l.from];
        let mut y = (l.from / 50 + 1) * 50;
        while y < l.to - 12 {
            marks.push(y);
            y += 50;
        }
        marks.push(l.to);
        for y in marks {
            let left = (x(y) - 14.0).clamp(0.0, STRIP_W - 28.0);
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(left), top: Val::Px(18.0), width: Val::Px(28.0), justify_content: JustifyContent::Center, ..default() },)).with_children(|t| {
                t.spawn(text(f, &y.to_string(), 10.5, MUTED));
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use isostate_core::inspect::IdeaGroup;

    fn idea(name: &str, stands: Stands, year: Option<i64>, discover: bool) -> TownIdea {
        TownIdea { name: name.into(), label: name.replace('_', " "), kind: "method", stands, year, how: String::new(), share: 0.0, towns: 0, said: format!("{}: ...", name.replace('_', " ")), discover }
    }

    fn ideas() -> Ideas {
        Ideas {
            town: 3,
            from: 1750,
            to: 1821,
            groups: vec![IdeaGroup {
                name: "Iron and steel".into(),
                ideas: vec![idea("coke_furnace", Stands::Here, Some(1784), false), idea("bessemer", Stands::Elsewhere, Some(1810), false), idea("steel_rail", Stands::Nowhere, None, true)],
            }],
        }
    }

    #[test]
    fn a_chip_chosen_makes_no_edit_and_discover_makes_the_idea_tools_edit() {
        let ideas = ideas();
        // choosing any chip, of any state, makes no edit
        for i in ideas.all() {
            assert_eq!(edit_of(&Press::Choose(i.name.clone()), &ideas, Some(&i.name)), None);
        }
        // Discover it here makes nothing with no idea chosen, or one that may not be found here
        assert_eq!(edit_of(&Press::Discover, &ideas, None), None);
        assert_eq!(edit_of(&Press::Discover, &ideas, Some("coke_furnace")), None);
        assert_eq!(edit_of(&Press::Discover, &ideas, Some("bessemer")), None);
        // and for one found nowhere, the very edit the Idea tool makes on a click in the same town
        let card = edit_of(&Press::Discover, &ideas, Some("steel_rail")).expect("an idea found nowhere may be found here");
        assert_eq!(card, sandbox::discover_edit("steel_rail".into(), 3));
        assert_eq!(card, sandbox::idea_tool_edit("steel_rail", 3));
    }

    #[test]
    fn a_chip_says_its_year_or_its_towns() {
        let mut i = idea("coke_furnace", Stands::Here, Some(1784), false);
        i.label = "Coke blast furnace".into();
        assert_eq!(chip_words(&i), "1784 Coke blast furnace");
        let mut e = idea("railway", Stands::Elsewhere, Some(1823), false);
        (e.label, e.towns) = ("Railways".into(), 5);
        assert_eq!(chip_words(&e), "Railways (5 towns)");
        assert_eq!(chip_words(&idea("steel_rail", Stands::Nowhere, None, true)), "steel rail");
        // what the section shows knows the idea chosen by its name
        let look = Look::of(&ideas(), Some("steel_rail"));
        assert_eq!(look.chosen, Some(("steel_rail".to_string(), "steel rail: ...".to_string(), true)));
        assert_eq!(Look::of(&ideas(), Some("no_such_idea")).chosen, None);
    }
}
