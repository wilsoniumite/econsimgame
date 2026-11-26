//! The backlog seen from the game (her question of 19 November, "Can I see
//! the backlog somewhere?"). A button on the help card (H) opens a card that
//! reads docs/BACKLOG.md where it can be found beside the game: what is in
//! flight, the headings of the order of work and what waits on her; and under
//! them her own notes of the day (note.rs), the last of which she can drop.
//! It only reads: the file is the managing session's, and a dropped note is
//! moved to `notes/dropped/`, not deleted.

// see the note at the top of ui.rs
#![allow(deprecated)]

use std::path::{Path, PathBuf};

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::ui::{self, INK, MUTED, RULE};
use crate::UiFont;

/// What the card shows of the backlog.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Seen {
    /// each piece in flight, with its state
    pub in_flight: Vec<String>,
    /// the headings of the order of work
    pub order: Vec<String>,
    /// what waits on her, by its lead words
    pub waiting: Vec<String>,
}

/// Markdown made plain: links to their words, and no backticks or stars.
/// The What's new card (whats_new.rs) reads the devlog through it too.
pub fn plain(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(open) = rest.find('[') {
        let Some(mid) = rest[open..].find("](") else { break };
        let Some(close) = rest[open + mid..].find(')') else { break };
        out.push_str(&rest[..open]);
        out.push_str(&rest[open + 1..open + mid]);
        rest = &rest[open + mid + close + 1..];
    }
    out.push_str(rest);
    out.replace(['`', '*'], "").trim().to_string()
}

/// The lines of the section under `## NAME`, up to the next `## `.
fn section<'a>(md: &'a str, name: &str) -> impl Iterator<Item = &'a str> {
    let mut lines = md.lines();
    let found = lines.by_ref().any(|l| l.trim_end() == format!("## {name}") || l.starts_with(&format!("## {name} ")));
    lines.take_while(|l| !l.starts_with("## ")).filter(move |_| found)
}

/// Read what the card shows from the backlog's text.
pub fn read(md: &str) -> Seen {
    let in_flight = section(md, "In flight")
        .filter(|l| l.starts_with('|'))
        .map(|l| l.trim().trim_matches('|').split('|').map(|c| c.trim()).collect::<Vec<_>>())
        // the table's head and the rule under it
        .filter(|cells| cells.len() >= 2 && !cells[0].starts_with("---") && cells[0] != "Piece")
        .map(|cells| format!("{}: {}", plain(cells[0]), plain(cells[cells.len() - 1])))
        .collect();
    let order = section(md, "The order of work").filter_map(|l| l.strip_prefix("### ")).map(plain).collect();
    let waiting = section(md, "Waiting on Stella")
        .filter_map(|l| l.strip_prefix("- **"))
        .filter_map(|l| l.split_once("**").map(|(lead, _)| plain(lead).trim_end_matches([':', '.', ',']).to_string()))
        .collect();
    Seen { in_flight, order, waiting }
}

/// Where docs/BACKLOG.md is, if it can be found beside the game.
pub fn find() -> Option<PathBuf> {
    find_doc("BACKLOG.md")
}

/// Where a tracked page of docs/ is, if it can be found beside the game: in
/// the folder the game was started from or one above it, or beside the
/// program. The backlog's card and the What's new card (docs/DEVLOG.md) both
/// look for their pages so.
pub fn find_doc(name: &str) -> Option<PathBuf> {
    let starts = [std::env::current_dir().ok(), std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf))];
    for start in starts.into_iter().flatten() {
        for dir in start.ancestors().take(8) {
            let p = dir.join("docs").join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// Her notes of the day in `folder`: (number, words, the clock it was typed at).
pub fn notes_of_the_day(folder: &Path, day: &str) -> Vec<(usize, String, String)> {
    crate::note::of_the_day(folder, day)
        .into_iter()
        .map(|(n, path)| {
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            let words = text.lines().next().and_then(|l| l.strip_prefix("words: ")).unwrap_or("").to_string();
            let clock = text.lines().find_map(|l| l.strip_prefix("clock: ")).unwrap_or("").to_string();
            (n, words, clock)
        })
        .collect()
}

/// Set the last note of the day aside, its text and its picture, in
/// `dropped/` inside the notes' folder; say which, or why not.
pub fn drop_last(folder: &Path, day: &str) -> Result<String, String> {
    let Some((n, text)) = crate::note::of_the_day(folder, day).pop() else { return Err("there is no note of today to drop".into()) };
    let aside = folder.join("dropped");
    std::fs::create_dir_all(&aside).map_err(|e| crate::saves::plainly(&e))?;
    for path in [text.clone(), text.with_extension("png")] {
        if path.exists() {
            let to = aside.join(path.file_name().unwrap_or_default());
            std::fs::rename(&path, &to).map_err(|e| crate::saves::plainly(&e))?;
        }
    }
    Ok(format!("the note {day}-{n:02} is set aside in {}", aside.display()))
}

#[derive(Resource, Default)]
pub struct BacklogCard {
    pub open: bool,
    /// raised when the card is to be read and drawn again
    version: u64,
    /// what the drop last said
    said: Option<String>,
}

/// The help card's button that opens the card.
#[derive(Component)]
pub struct OpenBacklog;
#[derive(Component)]
struct CardRoot;
#[derive(Component)]
struct CardBody;
#[derive(Component)]
struct CloseCard;
#[derive(Component)]
struct DropLast;

pub fn plugin(app: &mut App) {
    app.init_resource::<BacklogCard>().add_systems(PostStartup, spawn_card).add_systems(Update, (clicks, draw).chain());
}

fn spawn_card(mut commands: Commands, font: Res<UiFont>) {
    let f = &font.0;
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, left: Val::Px(0.0), right: Val::Px(0.0), top: Val::Px(90.0), bottom: Val::Px(70.0), justify_content: JustifyContent::Center, ..default() },
            GlobalZIndex(31),
            Visibility::Hidden,
            CardRoot,
        ))
        .with_children(|wrap| {
            wrap.spawn((ui::card(Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), max_height: Val::Percent(100.0), padding: UiRect::axes(Val::Px(24.0), Val::Px(18.0)), row_gap: Val::Px(6.0), border_radius: BorderRadius::all(Val::Px(18.0)), ..default() }), RelativeCursorPosition::default()))
                .insert(GlobalZIndex(31))
                .with_children(|c| {
                    let card = c.target_entity();
                    c.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() }).with_children(|row| {
                        row.spawn(ui::text(f, "The backlog, and your notes of today", 22.0, INK));
                        ui::small_button(row, f, "Close", CloseCard);
                    });
                    ui::scroll_list(c, card, Node { flex_grow: 1.0, flex_shrink: 1.0, ..default() }, Node { flex_grow: 1.0, row_gap: Val::Px(4.0), padding: UiRect::right(Val::Px(ui::THUMB_ROOM)), ..default() }, CardBody, |_| {});
                });
        });
}

fn clicks(mut card: ResMut<BacklogCard>, open: Query<&Interaction, (Changed<Interaction>, With<OpenBacklog>)>, close: Query<&Interaction, (Changed<Interaction>, With<CloseCard>)>, drop: Query<&Interaction, (Changed<Interaction>, With<DropLast>)>, mut help: Query<&mut Visibility, With<ui::HelpCard>>, keys: Res<ButtonInput<KeyCode>>, about: Option<Res<crate::note::About>>) {
    if open.iter().any(|i| *i == Interaction::Pressed) {
        card.open = true;
        card.said = None;
        card.version += 1;
        for mut v in &mut help {
            *v = Visibility::Hidden;
        }
    }
    if card.open && (close.iter().any(|i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Escape)) {
        card.open = false;
        card.version += 1;
    }
    if card.open && drop.iter().any(|i| *i == Interaction::Pressed) {
        let (day, _) = crate::note::day_and_time(crate::saves::now());
        let said = match about.as_ref().map(|a| a.folder.clone()) {
            Some(Ok(folder)) => drop_last(&folder, &day).unwrap_or_else(|why| format!("Nothing dropped: {why}")),
            Some(Err(why)) => format!("Nothing dropped: {why}"),
            None => "Nothing dropped: the game did not say where notes go".to_string(),
        };
        println!("backlog: {said}");
        card.said = Some(said);
        card.version += 1;
    }
}

fn heading(p: &mut ChildSpawnerCommands, f: &Handle<Font>, s: &str) {
    p.spawn((ui::text(f, s, 16.0, INK), Node { margin: UiRect::top(Val::Px(10.0)), ..default() }));
    p.spawn((Node { height: Val::Px(1.0), margin: UiRect::bottom(Val::Px(2.0)), ..default() }, BackgroundColor(RULE)));
}

fn line(p: &mut ChildSpawnerCommands, f: &Handle<Font>, s: &str, colour: Color) {
    p.spawn(ui::text(f, s, 13.5, colour));
}

fn draw(mut commands: Commands, card: Res<BacklogCard>, font: Res<UiFont>, about: Option<Res<crate::note::About>>, mut root: Query<&mut Visibility, With<CardRoot>>, body: Query<(Entity, Option<&Children>), With<CardBody>>, mut drawn: Local<Option<u64>>) {
    if *drawn == Some(card.version) {
        return;
    }
    *drawn = Some(card.version);
    for mut v in &mut root {
        v.set_if_neq(if card.open { Visibility::Inherited } else { Visibility::Hidden });
    }
    if !card.open {
        return;
    }
    let Ok((body, children)) = body.single() else { return };
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    let f = &font.0;
    let found = find();
    let seen = found.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).map(|md| read(&md));
    let (day, _) = crate::note::day_and_time(crate::saves::now());
    let folder = about.as_ref().and_then(|a| a.folder.clone().ok());
    let notes = folder.as_ref().map(|d| notes_of_the_day(d, &day)).unwrap_or_default();
    let said = card.said.clone();
    commands.entity(body).with_children(|p| {
        match (&found, &seen) {
            (Some(path), Some(seen)) => {
                line(p, f, &format!("Read from {} as it stands now.", path.display()), MUTED);
                heading(p, f, "In flight");
                for s in &seen.in_flight {
                    line(p, f, &format!("\u{2022} {s}"), INK);
                }
                heading(p, f, "The order of work");
                for s in &seen.order {
                    line(p, f, s, INK);
                }
                heading(p, f, "Waiting on you");
                for s in &seen.waiting {
                    line(p, f, &format!("\u{2022} {s}"), INK);
                }
            }
            _ => line(p, f, "docs/BACKLOG.md is not beside the game here (it is looked for from the folder the game was started in, and beside the program), so only your notes are shown.", MUTED),
        }
        heading(p, f, &format!("Your notes of today, {day} (Greenwich time)"));
        if notes.is_empty() {
            line(p, f, "None yet. N, anywhere in the game, keeps one in your words with a picture of the screen.", MUTED);
        }
        for (n, words, clock) in &notes {
            p.spawn(Node { column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
                row.spawn(ui::text(f, &format!("{n}."), 13.5, crate::note::NOTE_INK));
                row.spawn((ui::text(f, words, 13.5, INK), Node { flex_shrink: 1.0, ..default() }));
                row.spawn(ui::text(f, clock, 12.5, MUTED));
            });
        }
        if let Some(folder) = &folder {
            line(p, f, &format!("Kept in {}", folder.display()), MUTED);
        }
        if !notes.is_empty() {
            p.spawn(Node { margin: UiRect::top(Val::Px(6.0)), ..default() }).with_children(|row| ui::small_button(row, f, "Drop the last note", DropLast));
        }
        if let Some(s) = said {
            line(p, f, &s, crate::note::NOTE_INK);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD: &str = "# Backlog\n\n## In flight\n\nWhat is out.\n\n| Piece | Order of work | Branch | State |\n|---|---|---|---|\n| The lorry ([rules/lorry.md](rules/lorry.md)) | 1, the ways | `night/g-lorry` | in the gate |\n| **Town streets** | 3 | `night/g-town-streets` | landed |\n\n## The order of work\n\nIntro.\n\n### 1. The ways (note 20)\n\ntext\n\n### 2. The sandbox's atlas\n\n## Waiting on Stella\n\n- **Seven things that need a person at the window:** the sandbox.\n- **The look's three questions,** answered.\n- not a lead\n\n## Closed\n\n### Not an order heading\n";

    #[test]
    fn the_card_reads_in_flight_the_order_and_what_waits_on_her() {
        let seen = read(MD);
        assert_eq!(seen.in_flight, ["The lorry (rules/lorry.md): in the gate", "Town streets: landed"]);
        assert_eq!(seen.order, ["1. The ways (note 20)", "2. The sandbox's atlas"]);
        assert_eq!(seen.waiting, ["Seven things that need a person at the window", "The look's three questions"]);
        // a backlog without those sections shows nothing of them, and no section's lines leak into another
        assert_eq!(read("# Backlog\n\n## Closed\n\n### 1. x\n- **y** z\n"), Seen::default());
    }

    #[test]
    fn the_last_note_of_the_day_is_set_aside_not_deleted() {
        let dir = crate::saves::test_dir("backlog-drop");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for n in 1..=2 {
            std::fs::write(dir.join(format!("2026-11-26-0{n}.txt")), format!("words: note {n}\nclock: March 1791 (the map is of 1790)\n")).unwrap();
        }
        std::fs::write(dir.join("2026-11-26-02.png"), b"png").unwrap();
        assert_eq!(notes_of_the_day(&dir, "2026-11-26"), [(1, "note 1".to_string(), "March 1791 (the map is of 1790)".to_string()), (2, "note 2".to_string(), "March 1791 (the map is of 1790)".to_string())]);
        assert!(drop_last(&dir, "2026-11-26").unwrap().starts_with("the note 2026-11-26-02 is set aside"));
        assert!(dir.join("dropped").join("2026-11-26-02.txt").exists() && dir.join("dropped").join("2026-11-26-02.png").exists());
        assert_eq!(notes_of_the_day(&dir, "2026-11-26").len(), 1);
        drop_last(&dir, "2026-11-26").unwrap();
        assert!(drop_last(&dir, "2026-11-26").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
