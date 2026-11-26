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
//! A click on a chip chooses it: the line over the groups then says what the
//! simulation says of it (what it opened and how many works of it stand here
//! now; where it is known; or what it waits on). A click on an idea found
//! nowhere yet (a faint chip) also discovers it here: it sends the hand's edit
//! for an idea (`sandbox::discover_edit`, the very edit the Idea tool makes)
//! under the tray's guards, and its answer goes up as the Idea tool's does.
//! Until 25 November 2026 that took a second click, on Discover it here under
//! the line, because nothing could be taken back; now that an edit can be
//! (take_back.rs), the card's line after a discovery keeps what the idea
//! waited on before the hand ("Before the hand: ...") and offers Take it
//! back while the discovery is the last edit by hand, which is the tray's own
//! order. So the card opened and read, and any chip of an idea known here or
//! elsewhere chosen, issues no edit.
//!
//! An idea found by hand is carried out at once, between two months, as the
//! Idea tool's is: it is in the Hand tab's list and on the timeline at once,
//! and a kept session keeps it, since it is one more order in the log.
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

/// The idea chosen on a town's card, by the town and the idea's name.
#[derive(Resource, Default)]
pub struct Chosen {
    pick: Option<(usize, String)>,
    /// an idea discovered from the card: the town, the idea, and its line as
    /// it read before the hand (what it waited on)
    before: Option<(usize, String, String)>,
    version: u64,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Chosen>().add_systems(
        Update,
        // (after the answers of the frame are kept, as the inspector's own block is drawn)
        (chips, draw).chain().after(crate::game::pump).run_if(resource_exists::<Sandbox>).run_if(resource_exists::<Live>),
    );
}

/// The edit a click on an idea's chip makes: the hand's edit for an idea
/// found nowhere yet, found in this town; none for any other.
fn edit_of(ideas: &Ideas, name: &str) -> Option<Edit> {
    let idea = ideas.all().find(|i| i.name == name)?;
    idea.discover.then(|| sandbox::discover_edit(idea.name.clone(), ideas.town))
}

/// The town picked, and its ideas as last told, while the clock is at the present.
fn shown<'a>(picked: &Picked, answers: &'a Answers, tl: &Timeline, data: &Data, live: &Live) -> Option<&'a Ideas> {
    let thing = picked.what.map(thing_of)?;
    if !matches!(thing, Thing::Town(_)) || !at_present(tl, data, live) {
        return None;
    }
    answers.get(thing)?.as_ref().ok()?.ideas.as_ref()
}

/// A chip pressed: it is chosen (or let go, pressed again), and an idea found
/// nowhere yet is discovered here by hand, under the tray's guards.
#[allow(clippy::too_many_arguments)]
fn chips(
    q: Query<(&Interaction, &IdeaChip), Changed<Interaction>>,
    picked: Res<Picked>,
    answers: Res<Answers>,
    mut chosen: ResMut<Chosen>,
    mut sb: ResMut<Sandbox>,
    tl: Res<Timeline>,
    data: Res<Data>,
    live: Res<Live>,
    mut game: ResMut<Game>,
    mut toasts: ResMut<Toasts>,
    time: Res<Time>,
) {
    let Some(ideas) = shown(&picked, &answers, &tl, &data, &live) else { return };
    for (i, c) in &q {
        if *i != Interaction::Pressed {
            continue;
        }
        let Some(idea) = ideas.all().find(|x| x.name == c.0) else { continue };
        if let Some(edit) = edit_of(ideas, &c.0) {
            // found nowhere yet: discovered here, and its line as it stood kept for the card
            chosen.pick = Some((ideas.town, c.0.clone()));
            chosen.version += 1;
            match sandbox::send_as(Tool::Idea, edit, &mut sb, &tl, &data, &live, &mut game) {
                Ok(()) => {
                    chosen.before = Some((ideas.town, c.0.clone(), card_words(&idea.said).to_string()));
                    println!("ideas: discovered here by hand; before, {}", idea.said);
                }
                Err(why) => toasts.say("orders_refused", why, "By hand", time.elapsed_secs()),
            }
            continue;
        }
        let again = chosen.pick.as_ref().is_some_and(|(t, n)| *t == ideas.town && *n == c.0);
        chosen.pick = if again { None } else { Some((ideas.town, c.0.clone())) };
        chosen.version += 1;
        if !again {
            println!("ideas: {}", idea.said);
        }
    }
}

/// What the section shows, to tell whether it must be drawn again: the
/// strip's ends, each chip, and the line of the idea chosen.
#[derive(Clone, PartialEq, Debug)]
struct Look {
    from: i64,
    to: i64,
    /// each group's name, and each of its ideas: its name, its chip's words,
    /// where it stands, and for one known here the year it was learnt (none
    /// for one known before the history began) and how it was found here
    groups: Vec<(String, Vec<(String, String, Stands, Option<i64>, Found)>)>,
    /// the idea chosen: its name and its line
    chosen: Option<(String, String)>,
    /// while the idea chosen was found here by the last edit by hand: its line
    /// as it read before the hand, where the card was the hand, and the month
    /// the island would go back to
    back: Option<(Option<String>, String)>,
}

impl Look {
    fn of(ideas: &Ideas, chosen: Option<&str>, back: Option<(Option<String>, String)>) -> Look {
        Look {
            from: ideas.from,
            to: ideas.to,
            groups: ideas
                .groups
                .iter()
                .map(|g| (g.name.clone(), g.ideas.iter().map(|i| (i.name.clone(), chip_words(i), i.stands, learnt(i), Found::of(i))).collect()))
                .collect(),
            chosen: chosen.and_then(|n| ideas.all().find(|i| i.name == n)).map(|i| (i.name.clone(), card_words(&i.said).to_string())),
            back,
        }
    }
}

/// An idea's line as the card shows it: the core's line for an idea found
/// nowhere ends by naming the second click the card asked for until 25
/// November 2026 ("Discover it here finds it in Glenwick now, by hand."),
/// which a click on the chip now is, and that sentence is left off.
fn card_words(said: &str) -> &str {
    said.split(" Discover it here").next().unwrap_or(said)
}

/// While the idea chosen in the town was found there by the last edit by
/// hand, and that edit can be taken back: its line before the hand (if the
/// card found it) and the month the island would go back to.
fn taking_back(standing: &crate::live::Standing, chosen: &Chosen, town: usize, name: &str) -> Option<(Option<String>, String)> {
    match crate::take_back::last_edit(standing)? {
        Edit::Discover { tech, town: t } if *t == town && tech == name => {
            let before = chosen.before.as_ref().filter(|(bt, bn, _)| *bt == town && bn == name).map(|b| b.2.clone());
            Some((before, crate::take_back::date_of(standing.back?)))
        }
        _ => None,
    }
}

/// How an idea known here came to be: found here by hand, found here by the
/// island's own rule (a draw, or a pin), or learnt.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Found {
    ByHand,
    Here,
    Learnt,
}

impl Found {
    fn of(i: &TownIdea) -> Found {
        match i.how.as_str() {
            h if h.starts_with("found here") && h.ends_with("by hand") => Found::ByHand,
            h if h.starts_with("found here") => Found::Here,
            _ => Found::Learnt,
        }
    }
}

/// The year an idea known here was learnt, for the strip: none for one known
/// elsewhere or nowhere, or known before the history began.
fn learnt(i: &TownIdea) -> Option<i64> {
    i.year.filter(|_| i.stands == Stands::Here && !i.how.starts_with("known before"))
}

/// The words on an idea's chip: the year it was learnt and its label for one
/// known here (the label alone for one known before the history began, of
/// which there are a score); with the number of towns that know it for one
/// known elsewhere.
fn chip_words(i: &TownIdea) -> String {
    match (i.stands, i.year) {
        (Stands::Here, Some(y)) if !i.how.starts_with("known before") => format!("{y} {}", i.label),
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
    let look = ideas.map(|x| Look::of(x, name, name.and_then(|n| taking_back(&live.standing, &chosen, x.town, n))));
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
    p.spawn(text(f, "A short tick for each idea learnt from the other towns, a tall one for each found here, in the hand's ink if by hand.", 11.0, MUTED));
    // the line of the idea chosen; after a discovery here by hand, what it waited on before and the
    // chip that takes it back
    match &l.chosen {
        Some((_, said, ..)) => {
            p.spawn((text(f, said, 12.5, INK), Node { margin: UiRect::top(Val::Px(4.0)), ..default() }));
            if let Some((before, date)) = &l.back {
                if let Some(before) = before {
                    p.spawn((text(f, &format!("Before the hand: {before}"), 11.5, MUTED), Node { margin: UiRect::top(Val::Px(3.0)), ..default() }));
                }
                p.spawn(Node { margin: UiRect::top(Val::Px(4.0)), align_items: AlignItems::Center, column_gap: Val::Px(8.0), flex_wrap: FlexWrap::Wrap, ..default() }).with_children(|row| {
                    crate::family::chip_button(row, f, crate::take_back::CARD_CHIP, true, HAND_INK, crate::take_back::TakeBackChip);
                    row.spawn(text(f, &format!("puts the island back to {date}, before it was found here"), 11.5, MUTED));
                });
            }
        }
        None => {
            p.spawn((text(f, "Click an idea to see what it opened, where it is known, or what it waits on. A click on one found nowhere yet (faint) discovers it here by hand, and the card then offers to take it back.", 11.5, MUTED), Node { margin: UiRect::top(Val::Px(4.0)), ..default() }));
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
/// a tick at each year the town came to know an idea (short and muted where
/// it was learnt from the other towns, tall and in ink where it was found
/// here, tall and in the hand's ink where it was found here by hand), and the
/// years at the ends and each half-century between.
fn strip(p: &mut ChildSpawnerCommands, f: &Handle<Font>, l: &Look) {
    let span = (l.to - l.from).max(1) as f32;
    let x = |y: i64| ((y - l.from) as f32 / span).clamp(0.0, 1.0) * STRIP_W;
    p.spawn(Node { width: Val::Px(STRIP_W), height: Val::Px(38.0), margin: UiRect::top(Val::Px(4.0)), ..default() }).with_children(|s| {
        s.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(18.0), width: Val::Px(STRIP_W), height: Val::Px(1.0), ..default() }, BackgroundColor(MUTED)));
        for (_, ideas) in &l.groups {
            for (_, _, _, year, found) in ideas {
                let Some(y) = year else { continue };
                let (h, colour) = match found {
                    Found::ByHand => (18.0, HAND_INK),
                    Found::Here => (16.0, INK),
                    Found::Learnt => (9.0, MUTED),
                };
                s.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(x(*y) - 1.0), top: Val::Px(18.0 - h), width: Val::Px(2.0), height: Val::Px(h), ..default() }, BackgroundColor(colour)));
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
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(left), top: Val::Px(22.0), width: Val::Px(28.0), justify_content: JustifyContent::Center, ..default() },)).with_children(|t| {
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
    fn a_click_on_an_idea_found_nowhere_makes_the_idea_tools_edit_and_on_any_other_none() {
        let ideas = ideas();
        // an idea known here or elsewhere, or none at all, makes no edit
        assert_eq!(edit_of(&ideas, "coke_furnace"), None);
        assert_eq!(edit_of(&ideas, "bessemer"), None);
        assert_eq!(edit_of(&ideas, "no_such_idea"), None);
        // and one found nowhere, the very edit the Idea tool makes on a click in the same town
        let card = edit_of(&ideas, "steel_rail").expect("an idea found nowhere may be found here");
        assert_eq!(card, sandbox::discover_edit("steel_rail".into(), 3));
        assert_eq!(card, sandbox::idea_tool_edit("steel_rail", 3));
    }

    #[test]
    fn the_card_offers_to_take_back_only_its_own_discovery_while_it_is_the_last_edit() {
        use crate::live::{Last, Standing};
        let mut chosen = Chosen::default();
        let mut s = Standing::default();
        assert_eq!(taking_back(&s, &chosen, 3, "steel_rail"), None, "nothing to take back");
        let last = |edit: Edit| Some(Last { edit, said: "By hand: steel rail found in Glenwick in 1821".into(), before: "steel rail was found in Glenwick by hand".into() });
        s.last = last(sandbox::discover_edit("steel_rail".into(), 3));
        assert_eq!(taking_back(&s, &chosen, 3, "steel_rail"), None, "no copy to go back to");
        s.back = Some((1820, 0));
        // found by the Idea tool: the chip, with no line from before
        assert_eq!(taking_back(&s, &chosen, 3, "steel_rail"), Some((None, "January 1821".to_string())));
        // found from the card: the line it read before the hand as well
        chosen.before = Some((3, "steel_rail".into(), "steel rail: found nowhere yet. It waits on ...".into()));
        assert_eq!(taking_back(&s, &chosen, 3, "steel_rail"), Some((Some("steel rail: found nowhere yet. It waits on ...".to_string()), "January 1821".to_string())));
        // another town's card, another idea, or a later edit of another kind: none
        assert_eq!(taking_back(&s, &chosen, 4, "steel_rail"), None);
        assert_eq!(taking_back(&s, &chosen, 3, "bessemer"), None);
        s.last = last(Edit::People { town: 3, factor: 0.5 });
        assert_eq!(taking_back(&s, &chosen, 3, "steel_rail"), None);
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
        let look = Look::of(&ideas(), Some("steel_rail"), None);
        assert_eq!(look.chosen, Some(("steel_rail".to_string(), "steel rail: ...".to_string())));
        assert_eq!(Look::of(&ideas(), Some("no_such_idea"), None).chosen, None);
        // the core's line for an idea found nowhere names the second click the card no longer asks
        assert_eq!(card_words("Hot-blast furnace: found nowhere yet. It waits on coal. Discover it here finds it in Glenwick now, by hand."), "Hot-blast furnace: found nowhere yet. It waits on coal.");
        assert_eq!(card_words("Railways: known here since 1824."), "Railways: known here since 1824.");
    }
}
