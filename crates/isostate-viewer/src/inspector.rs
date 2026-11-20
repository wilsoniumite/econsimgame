//! The sandbox's inspector (`docs/designs/sandbox.md`, 4.5): under the record's
//! lines about the thing clicked, what the simulation knows of it as it stands.
//! The question goes to the game's thread (`Cmd::Ask`), which answers between
//! months with `inspect::tell` (`Msg::Told`), and the answer is drawn without
//! being understood: a heading with its month, then each group's rows as a
//! name and a figure. A town has two small lines under its rows, its wage and
//! grain's price month by month, with a tick in the hand's ink at every month
//! the hand touched the town or the whole island; and under it all stands the
//! line of the last edit, when the last order answered was one.
//!
//! The question is asked when something is picked with the clock at the
//! present, and again when a month or a year arrives while the card is open
//! and when an order's answer comes: once for each frame that brings them,
//! never every frame. The block is drawn again only when what it shows
//! differs from what is drawn. An answer is a few rows of text and the
//! question costs the thread a few hundredths of a millisecond, so the view
//! does not wait for it (4.7). None of this runs in the family game, whose
//! inspector is as it was.

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use isostate_core::inspect::{Thing, Told};

use crate::game::{self, Game, Now};
use crate::live::{Cmd, Live, Months};
use crate::replay::Replay;
use crate::sandbox::HAND_INK;
use crate::ui::{text, UiState, INK, MUTED, RULE};
use crate::{chart, Data, Pick, Picked, Timeline, UiFont};

/// Where the inspector's block stands in the card: after the node the
/// family's orders are drawn in, which a sandbox leaves empty.
#[derive(Component)]
pub struct InspectorBlock;

/// The newest answer the simulation gave about each thing asked about.
#[derive(Resource, Default)]
pub struct Answers {
    told: HashMap<Thing, Result<Told, String>>,
    /// when each question still to be answered was asked, oldest first
    asked: HashMap<Thing, VecDeque<Instant>>,
    /// how long the newest answer about each thing took to come, in milliseconds
    took: HashMap<Thing, f32>,
    /// raised by one with every answer kept
    pub version: u64,
    /// print each new answer once, for a scripted run
    print: bool,
}

impl Answers {
    /// For a scripted run: each new answer is printed once.
    pub fn printing() -> Answers {
        Answers { print: true, ..default() }
    }

    /// Ask the game's thread about a thing.
    pub fn ask(&mut self, live: &Live, thing: Thing) {
        self.asked.entry(thing).or_default().push_back(Instant::now());
        live.send(Cmd::Ask(thing));
    }

    /// Keep an answer that has come (`game::pump`).
    pub fn keep(&mut self, thing: Thing, told: Result<Told, String>) {
        if let Some(t0) = self.asked.get_mut(&thing).and_then(|q| q.pop_front()) {
            self.took.insert(thing, t0.elapsed().as_secs_f32() * 1e3);
        }
        if self.print && self.told.get(&thing) != Some(&told) {
            let took = self.took.get(&thing).map_or(String::new(), |ms| format!(" (it came {ms:.1} ms after its question)"));
            println!("inspector: {}{took}", said(&told));
        }
        self.told.insert(thing, told);
        self.version += 1;
    }

    pub fn get(&self, thing: Thing) -> Option<&Result<Told, String>> {
        self.told.get(&thing)
    }
}

/// An answer in one line: "TITLE, AS_OF | GROUP: name figure; name figure | ...".
pub fn said(told: &Result<Told, String>) -> String {
    match told {
        Ok(t) => {
            let mut s = format!("{}, {}", t.title, t.as_of);
            for g in &t.groups {
                let rows: Vec<String> = g.rows.iter().map(|(n, f)| format!("{n} {f}")).collect();
                s.push_str(&format!(" | {}: {}", g.name, rows.join("; ")));
            }
            // a town's ideas (`town_ideas`), counted by where they stand
            if let Some(i) = &t.ideas {
                use isostate_core::inspect::Stands;
                s.push_str(&format!(" | Its ideas: {} known here, {} elsewhere, {} nowhere", i.count(Stands::Here), i.count(Stands::Elsewhere), i.count(Stands::Nowhere)));
            }
            s
        }
        Err(why) => format!("refused: {why}"),
    }
}

/// The question for a thing picked on the map. A works is asked for by its
/// slot, which is its place in the record's list of works (`inspect`).
pub(crate) fn thing_of(p: Pick) -> Thing {
    match p {
        Pick::Town(t) => Thing::Town(t),
        Pick::Firm(f) => Thing::Works(f),
        Pick::Cell(c) => Thing::Cell(c),
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Answers>().add_systems(
        Update,
        // in the frame the answers come, and before a question about the clicked square puts
        // the note of the last order away
        (ask, draw).chain().after(game::pump).before(game::ask_site).run_if(resource_exists::<crate::sandbox::Sandbox>).run_if(resource_exists::<Live>),
    );
}

/// The clock shows the present: the newest year, or the one before it while
/// the next is about to show. A year gone by is the past.
pub(crate) fn at_present(tl: &Timeline, data: &Data, live: &Live) -> bool {
    game::now_of((tl.t.floor() as usize).min(data.r.ny - 1), data.latest(), live.seen) != Now::Past
}

/// What brings a new question: a month come, a year come, an order answered.
pub(crate) fn news_key(ui: &UiState, data: &Data, game: &Game) -> (u64, usize, u64) {
    (ui.months, data.r.filled, game.answered)
}

/// While the clock runs, how often at most the same thing is asked about
/// again and its card drawn again, in seconds. At Fastest a month comes
/// nearly every frame, and a town's card drawn afresh each time cost the
/// frames 3 ms (in the middle of them all) that the view does not have; twice
/// a second is as fast as the rows can be read. With the clock stopped every
/// month, year and answer is asked about at once.
pub(crate) const RUNNING_EVERY: f32 = 0.5;

/// The clock runs and the same thing was asked about or drawn less than
/// `RUNNING_EVERY` ago.
pub(crate) fn too_soon(tl: &Timeline, last: Option<Instant>) -> bool {
    tl.playing && last.is_some_and(|t| t.elapsed().as_secs_f32() < RUNNING_EVERY)
}

/// Ask about the thing picked when it is picked and whenever something has
/// come since the last question; only with the clock at the present, and not
/// while the year turns (the answer would be of the year about to end: it is
/// asked once the year has come). While the clock runs, the same thing no
/// more often than `RUNNING_EVERY`.
#[allow(clippy::too_many_arguments)]
fn ask(picked: Res<Picked>, tl: Res<Timeline>, data: Res<Data>, live: Res<Live>, game: Res<Game>, ui: Res<UiState>, mut answers: ResMut<Answers>, mut seen: Local<Option<((Option<Thing>, (u64, usize, u64)), Instant)>>) {
    let key = (picked.what.map(thing_of), news_key(&ui, &data, &game));
    let last = seen.as_ref().filter(|s| s.0 .0 == key.0).map(|s| s.1);
    if seen.as_ref().is_some_and(|s| s.0 == key) || live.turning() || !at_present(&tl, &data, &live) || too_soon(&tl, last) {
        return;
    }
    *seen = Some((key, Instant::now()));
    if let Some(thing) = key.0 {
        answers.ask(&live, thing);
    }
}

/// How many months the two small lines go back.
const MONTHS_SHOWN: usize = 60;

/// A town's two small lines: from the January before the first month held,
/// so that the chart's rules stand on Januaries.
#[derive(Clone, PartialEq, Debug)]
struct Lines {
    /// the place in the run of the first figure (a January)
    start: usize,
    /// months held, and the figures (NaN before the first held)
    held: usize,
    wage: Vec<f32>,
    price: Vec<f32>,
    /// the figures whose month the hand touched
    ticks: Vec<usize>,
}

#[derive(Clone, PartialEq, Debug)]
enum Shown {
    Nothing,
    /// the clock is on a year gone by
    Past,
    /// asked, and nothing has come yet
    Asking,
    Told { told: Result<Told, String>, lines: Option<Lines> },
}

#[derive(Default)]
struct Drawn {
    /// what the last look was made from, so that a frame with nothing new looks no further
    key: Option<(Option<Thing>, u64, bool, u64, usize, u64)>,
    shown: Option<(Shown, Option<String>)>,
    /// when it was last drawn
    at: Option<Instant>,
    /// the orders answered when last looked, and the line of the last edit if the last answer was one
    answered: u64,
    edit: Option<String>,
}

/// The hand's news that touched the town whose market place is `node`, or the
/// whole island: the months they fall in, as places in the run.
fn hand_months(r: &Replay, node: usize) -> Vec<usize> {
    r.header
        .events
        .iter()
        .filter(|e| e.kind == "hand" && e.cell.is_none_or(|c| c == node as i64))
        .filter_map(|e| {
            let m = ((e.year - r.y0 as f32) * 12.0).floor();
            (m >= 0.0).then_some(m as usize)
        })
        .collect()
}

fn lines(r: &Replay, months: &Months, t: usize) -> Option<Lines> {
    let grain = r.header.goods.iter().position(|g| g.name == "grain")?;
    let tail = months.town_tail(t, grain, MONTHS_SHOWN);
    let first = tail.first()?.0;
    let start = first - first % 12;
    let pad = first - start;
    let mut wage = vec![f32::NAN; pad];
    let mut price = vec![f32::NAN; pad];
    for &(_, w, p) in &tail {
        wage.push(w);
        price.push(p);
    }
    let node = r.header.towns.get(t)?.node;
    let mut ticks: Vec<usize> = hand_months(r, node).into_iter().filter(|&m| m >= first && m < start + wage.len()).map(|m| m - start).collect();
    ticks.sort_unstable();
    ticks.dedup();
    Some(Lines { start, held: tail.len(), wage, price, ticks })
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    font: Res<UiFont>,
    picked: Res<Picked>,
    tl: Res<Timeline>,
    data: Res<Data>,
    live: Res<Live>,
    game: Res<Game>,
    ui: Res<UiState>,
    answers: Res<Answers>,
    months: Res<Months>,
    mut images: ResMut<Assets<Image>>,
    q: Query<(Entity, Option<&Children>), With<InspectorBlock>>,
    mut drawn: Local<Drawn>,
) {
    // the last order answered, and whether it was an edit
    if game.answered != drawn.answered {
        drawn.answered = game.answered;
        drawn.edit = match &game.note {
            Some((true, line)) if line.starts_with("By hand") => Some(line.clone()),
            _ => None,
        };
    }
    let thing = picked.what.map(thing_of);
    let present = at_present(&tl, &data, &live);
    let key = (thing, answers.version, present, ui.months, data.r.header.events.len(), drawn.answered);
    // (while the clock runs, the same thing's card is drawn again no more often than RUNNING_EVERY;
    // what came meanwhile is drawn when that time is up)
    if drawn.key == Some(key) || (drawn.key.is_some_and(|k| k.0 == thing) && too_soon(&tl, drawn.at)) {
        return;
    }
    drawn.key = Some(key);
    let shown = match thing {
        None => Shown::Nothing,
        Some(_) if !present => Shown::Past,
        Some(th) => match answers.get(th) {
            None => Shown::Asking,
            Some(told) => Shown::Told { told: told.clone(), lines: if let Thing::Town(t) = th { lines(&data.r, &months, t) } else { None } },
        },
    };
    let now = (shown, drawn.edit.clone());
    if drawn.shown.as_ref() == Some(&now) {
        return;
    }
    let Ok((e, children)) = q.single() else { return };
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    let f = font.0.clone();
    commands.entity(e).with_children(|p| block(p, &f, &mut images, data.r.y0, &now.0, now.1.as_deref()));
    drawn.shown = Some(now);
    drawn.at = Some(Instant::now());
}

fn rule(p: &mut ChildSpawnerCommands) {
    p.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(6.0)), ..default() }, BackgroundColor(RULE)));
}

/// What a past year shows in place of the block.
pub const PAST: &str = "What the simulation knows is of the present: come back to it to see.";

fn block(p: &mut ChildSpawnerCommands, f: &Handle<Font>, images: &mut Assets<Image>, y0: i32, shown: &Shown, edit: Option<&str>) {
    match shown {
        Shown::Nothing => return,
        Shown::Past => {
            rule(p);
            p.spawn(text(f, PAST, 13.0, MUTED));
        }
        Shown::Asking => {
            rule(p);
            p.spawn(text(f, "Asking the simulation\u{2026}", 13.0, MUTED));
        }
        Shown::Told { told: Err(why), .. } => {
            rule(p);
            p.spawn(text(f, why, 13.0, MUTED));
        }
        Shown::Told { told: Ok(told), lines } => {
            rule(p);
            p.spawn(text(f, &format!("As it stands, {}", told.as_of), 15.5, INK));
            for g in &told.groups {
                group(p, f, &g.name, &g.rows);
            }
            if let Some(l) = lines {
                small_lines(p, f, images, y0, l);
            }
        }
    }
    if let Some(line) = edit {
        p.spawn((text(f, line, 12.5, MUTED), Node { margin: UiRect::top(Val::Px(8.0)), ..default() }));
    }
}

/// A group of rows: its name in ink, then each row's name (muted) and figure
/// (ink) in two columns. The Hand tab's foot draws the island's ledger so.
pub fn group(p: &mut ChildSpawnerCommands, f: &Handle<Font>, name: &str, rows: &[(String, String)]) {
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(2.0), margin: UiRect::top(Val::Px(6.0)), ..default() }).with_children(|col| {
        col.spawn(text(f, name, 13.5, INK));
        for (n, v) in rows {
            col.spawn(Node { column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                row.spawn((text(f, n, 12.5, MUTED), Node { width: Val::Px(NAME_W), flex_shrink: 0.0, ..default() }));
                row.spawn((text(f, v, 12.5, INK), Node { flex_grow: 1.0, flex_shrink: 1.0, flex_basis: Val::Px(0.0), ..default() }));
            });
        }
    });
}

/// The width of a row's name.
const NAME_W: f32 = 140.0;
/// The width of the small lines, their figures beside them.
const LINES_W: f32 = 340.0;

/// A figure of the small lines: a wage or a price, to two places from one,
/// three below it, as the core writes them.
fn figure(v: f32) -> String {
    if v.abs() >= 100.0 {
        format!("{v:.0}")
    } else if v.abs() >= 1.0 {
        format!("{v:.2}")
    } else {
        format!("{v:.3}")
    }
}

fn small_lines(p: &mut ChildSpawnerCommands, f: &Handle<Font>, images: &mut Assets<Image>, y0: i32, l: &Lines) {
    if l.held < 2 {
        return;
    }
    let head = if l.held >= MONTHS_SHOWN { format!("The last {} months", l.held) } else { format!("The last {} months, all the view holds", l.held) };
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), margin: UiRect::top(Val::Px(8.0)), ..default() }).with_children(|col| {
        col.spawn(text(f, &head, 13.5, INK));
        let ink = HAND_INK.to_srgba();
        let tick = [(ink.red * 255.0) as u8, (ink.green * 255.0) as u8, (ink.blue * 255.0) as u8];
        for (name, vals) in [("the wage, a month", &l.wage), ("the price of grain, a unit", &l.price)] {
            col.spawn(text(f, name, 12.5, MUTED));
            let c = chart::Chart {
                width: LINES_W,
                height: 34.0,
                line: vals,
                second: None,
                accent: tick,
                mark: vals.len().saturating_sub(1),
                years: vals.len(),
                first_year: y0 + (l.start / 12) as i32,
                months: true,
            };
            let mut pl = chart::plot(&c);
            chart::ticks(&mut pl, &c, &l.ticks, tick);
            chart::spawn_painted(col, f, images, &c, pl, &figure, &figure);
        }
        if !l.ticks.is_empty() {
            col.spawn(text(f, "A tick in the hand's ink marks a month the hand touched this town or the whole island.", 11.5, MUTED));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use isostate_core::inspect::Group;

    #[test]
    fn an_answer_is_said_in_one_line_and_a_works_is_asked_for_by_its_slot() {
        let told = Told { title: "Melbourne".into(), as_of: "March 1791".into(), groups: vec![Group { name: "People".into(), rows: vec![("people in the town".into(), "9,690".into()), ("hands in works".into(), "1,200".into())] }], ideas: None };
        assert_eq!(said(&Ok(told)), "Melbourne, March 1791 | People: people in the town 9,690; hands in works 1,200");
        assert_eq!(said(&Err("The square at 3, 4 is sea.".into())), "refused: The square at 3, 4 is sea.");
        assert_eq!(thing_of(Pick::Firm(12)), Thing::Works(12));
        assert_eq!(thing_of(Pick::Town(3)), Thing::Town(3));
        assert_eq!(thing_of(Pick::Cell(40)), Thing::Cell(40));
    }

    #[test]
    fn the_small_lines_start_on_a_january_and_tick_the_hands_months() {
        let mut r = crate::replay::tiny(1750);
        let node = r.header.towns.first().map(|t| t.node).unwrap_or(0);
        let ev = |year: f32, kind: &str, cell: Option<i64>| crate::replay::Event { year, kind: kind.into(), weight: 0.5, title: String::new(), text: String::new(), cell, family: None, kept: None, seq: 0 };
        // the hand on the whole island in March 1752, on this town in May 1752, elsewhere in June; a harvest that is not the hand's
        r.header.events = vec![ev(1752.2, "hand", None), ev(1752.37, "hand", Some(node as i64)), ev(1752.45, "hand", Some(node as i64 + 1)), ev(1752.6, "harvest", None)];
        assert_eq!(hand_months(&r, node), vec![26, 28]);
    }
}
