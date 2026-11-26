//! What's new (her note of 26 November: she is always behind on what the game
//! already has). A card opened from the help card (H), beside the backlog's,
//! lists the entries of docs/DEVLOG.md, the newest first: the day, the bold
//! lead as its title, where the entry says the thing is found in the game, and
//! its first sentence or two; a click opens the whole entry. The game keeps the
//! newest entry it has shown in a small file beside the notes, and a chip on
//! the screen a game starts from says how many entries are new since.
//!
//! The devlog is read as the backlog's card reads its page: at run time, where
//! it is found beside the game (`backlog_seen::find_doc`), and made plain by the
//! same `backlog_seen::plain`. Unlike the backlog's card it is read and parsed
//! once, on a thread of its own as the game starts, and the card's rows are
//! laid a few a frame while it is shut, so that opening it only shows it.

// see the note at the top of ui.rs
#![allow(deprecated)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

use crate::backlog_seen::{find_doc, plain};
use crate::ui::{self, INK, MUTED, RULE};
use crate::UiFont;

/// The file, in the saved games' folder beside `notes/`, that keeps the newest
/// entry the card has shown.
pub const SEEN: &str = "whats-new.txt";

/// The words an entry may use to say where its thing is found in the game,
/// as the last sentence of its paragraph: "In the game: H, then What's new."
pub const IN_THE_GAME: &str = "In the game:";

/// The ground of the chip that marks an entry new, and of the count on the
/// chip at the start.
pub const NEW_INK: Color = Color::srgb(0.76, 0.40, 0.17);

/// One entry of the devlog, made plain.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// the day it is filed under, as its heading says it: "2026-11-26", or
    /// "2026-11-10 to 11 (overnight)"
    pub when: String,
    /// its first day, "2026-11-26", which is what is compared
    pub day: String,
    /// the bold lead, without its full stop
    pub title: String,
    /// the sentence that says where the thing is found in the game, if the
    /// entry says: its "In the game:" line, or else its first sentence that
    /// names a key, a chip, a tab, a panel, a card, a tray, a button or a map view
    pub found: Option<String>,
    /// its first sentence or two (leaving out the one in `found`)
    pub gist: String,
    /// the whole entry, a paragraph or a point of a list a string
    pub whole: Vec<String>,
}

/// The sentences of a plain paragraph: it is cut after a full stop, a question
/// or an exclamation (and any closing quote or bracket) where a space and a
/// capital, a figure, a quote or a bracket follow, or a name out of the code
/// (a word with `_` or `/` in it, or an option: "run_tests.py takes ...").
pub fn sentences(s: &str) -> Vec<String> {
    let code = |w: &[char]| {
        let w: String = w.iter().take_while(|ch| **ch != ' ').collect();
        w.contains('_') || w.contains('/') || w.starts_with("--")
    };
    let c: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let (mut start, mut i) = (0, 0);
    while i < c.len() {
        if matches!(c[i], '.' | '?' | '!') {
            let mut j = i + 1;
            while j < c.len() && matches!(c[j], '"' | '\'' | ')' | '\u{201d}' | '\u{2019}') {
                j += 1;
            }
            let next = c.get(j + 1).copied();
            if c.get(j) == Some(&' ') && next.is_some_and(|n| n.is_uppercase() || n.is_ascii_digit() || matches!(n, '"' | '(' | '\u{201c}') || code(&c[j + 1..])) {
                out.push(c[start..j].iter().collect::<String>().trim().to_string());
                start = j + 1;
                i = j;
            }
        }
        i += 1;
    }
    let tail: String = c[start.min(c.len())..].iter().collect::<String>().trim().to_string();
    if !tail.is_empty() {
        out.push(tail);
    }
    out
}

/// Whether a sentence says where something is in the game. The devlog's
/// older days use "card", "panel" and "key" of other things (the rules pass's
/// scorecard, a graphics card, a hashable key), so the words are taken only
/// where they read as the game's own: a chip, a tray, a button, the legend or
/// a map view anywhere; a tab or a panel with its name before it (the Rules
/// tab, the Markets panel, the side panel); a card that is something's (a
/// town's card, the Idea tool's card) or the help or opening card; and a key,
/// a capital letter alone that opens, shows or turns something ("N opens a
/// line"), is pressed, or stands in brackets ("(H)"), or one held with Shift
/// or Ctrl. A sentence that is a bracket is passed over.
pub fn names_a_place(s: &str) -> bool {
    if s.starts_with('(') {
        return false;
    }
    const ANYWHERE: [&str; 7] = ["chip", "chips", "tray", "button", "buttons", "legend", "legend's"];
    const NOT_NAMES: [&str; 12] = ["The", "A", "An", "Its", "Each", "Every", "This", "That", "Their", "In", "On", "Your"];
    const KEY_VERBS: [&str; 4] = ["opens", "shows", "turns", "steps"];
    let words: Vec<&str> = s.split_whitespace().map(|w| w.trim_matches(|ch: char| !(ch.is_alphanumeric() || ch == '+' || ch == '\''))).filter(|w| !w.is_empty()).collect();
    let lower = s.to_lowercase();
    if lower.contains("map view") {
        return true;
    }
    for (i, w) in words.iter().enumerate() {
        let lw = w.to_lowercase();
        let before = if i > 0 { words[i - 1] } else { "" };
        let after = words.get(i + 1).copied().unwrap_or("");
        let named = before.chars().next().is_some_and(char::is_uppercase) && !NOT_NAMES.contains(&before);
        let place = match lw.trim_end_matches("'s") {
            p if ANYWHERE.contains(&p) => true,
            "tab" | "tabs" | "panel" | "panels" => named || before == "side",
            // (not "Stage 1's card", the rules pass's scorecard)
            "card" | "cards" => before.ends_with("'s") && !before.starts_with(|ch: char| ch.is_ascii_digit()) || matches!(before, "help" | "opening"),
            _ => false,
        };
        let letter = w.len() == 1 && w.chars().all(|ch| ch.is_ascii_uppercase()) && *w != "A" && *w != "I";
        let key = letter && (KEY_VERBS.contains(&after) || matches!(before, "press" | "Press") || s.contains(&format!("({w})")));
        if place || key || w.starts_with("Shift+") || w.starts_with("Ctrl+") {
            return true;
        }
    }
    false
}

/// A dated heading's day: "2026-11-10 to 11 (overnight): a night of helpers"
/// gives ("2026-11-10 to 11 (overnight)", "2026-11-10", "a night of helpers").
fn dated(heading: &str) -> Option<(String, String, String)> {
    let (when, words) = heading.split_once(": ").unwrap_or((heading, ""));
    let day = when.get(..10)?;
    let b = day.as_bytes();
    let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
    (digits(0..4) && b[4] == b'-' && digits(5..7) && b[7] == b'-' && digits(8..10)).then(|| (when.trim().to_string(), day.to_string(), words.trim().to_string()))
}

/// The paragraphs of a section's text: runs of lines with no blank between.
fn paragraphs(body: &[&str]) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for l in body {
        if l.trim().is_empty() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(l.to_string());
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// A paragraph made plain: a list is a point a string, prose one string.
fn plain_paragraph(lines: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for l in lines {
        match l.trim_start().strip_prefix("- ") {
            Some(point) => out.push(format!("\u{2022} {}", plain(point))),
            None => match out.last_mut() {
                // a line that carries on the one before
                Some(last) => {
                    last.push(' ');
                    last.push_str(&plain(l));
                }
                None => out.push(plain(l)),
            },
        }
    }
    out.retain(|s| !s.is_empty());
    out
}

/// An entry from its title and its paragraphs, made plain.
fn entry(when: &str, day: &str, title: String, whole: Vec<String>) -> Entry {
    let prose: Vec<String> = whole.iter().map(|p| p.trim_start_matches('\u{2022}').trim().to_string()).collect();
    // "In the game:" first; else the first sentence that names a place in the game
    let said = prose.iter().find_map(|p| p.find(IN_THE_GAME).map(|i| p[i + IN_THE_GAME.len()..].trim().to_string()));
    let all: Vec<String> = prose.iter().flat_map(|p| sentences(p)).collect();
    let found_at = if said.is_some() { None } else { all.iter().position(|s| names_a_place(s)) };
    let found = said.clone().or_else(|| found_at.map(|i| all[i].clone()));
    // the first sentence, and the second if the first is short; leaving out the one that says
    // where, the "In the game:" line, and a bracket that only names a page before the first
    let mut gist = String::new();
    let mut taken = 0;
    for (i, s) in all.iter().enumerate() {
        let page = taken == 0 && s.starts_with('(') && (s.ends_with(").") || s.ends_with(')'));
        if Some(i) == found_at || s.starts_with(IN_THE_GAME) || page {
            continue;
        }
        if !gist.is_empty() {
            gist.push(' ');
        }
        gist.push_str(s);
        taken += 1;
        if taken == 2 || gist.chars().count() >= 100 {
            break;
        }
    }
    // an entry that is all bracket or all place says that much
    if gist.is_empty() {
        gist = all.first().cloned().unwrap_or_default();
    }
    Entry { when: when.to_string(), day: day.to_string(), title, found, gist, whole }
}

/// Read the devlog's entries, the newest first. A dated section's entries are
/// its paragraphs that begin with a bold lead, each with what follows it up to
/// the next; since a day's entries are added at its end, they are listed from
/// its end. What a section says before its first bold lead (or all of it, if
/// it has none) is an entry of its own under the heading's words, the oldest
/// of its day. Anything outside a dated section is passed over.
pub fn read(md: &str) -> Vec<Entry> {
    let mut out = Vec::new();
    let lines: Vec<&str> = md.lines().collect();
    let heads: Vec<usize> = (0..lines.len()).filter(|&i| lines[i].starts_with("## ")).collect();
    for (n, &h) in heads.iter().enumerate() {
        let Some((when, day, words)) = dated(lines[h][3..].trim()) else { continue };
        let end = heads.get(n + 1).copied().unwrap_or(lines.len());
        let mut intro: Vec<String> = Vec::new();
        let mut led: Vec<(String, Vec<String>)> = Vec::new();
        for p in paragraphs(&lines[h + 1..end]) {
            let first = &p[0];
            let lead = first.strip_prefix("**").and_then(|rest| rest.find("**").map(|close| (rest[..close].to_string(), rest[close + 2..].to_string())));
            match lead {
                Some((lead, rest)) => {
                    let title = plain(&lead).trim_end_matches(['.', ':', ',']).trim().to_string();
                    let mut first_lines = vec![rest.trim().to_string()];
                    first_lines.extend(p[1..].iter().cloned());
                    led.push((title, plain_paragraph(&first_lines)));
                }
                None => match led.last_mut() {
                    Some((_, whole)) => whole.extend(plain_paragraph(&p)),
                    None => intro.extend(plain_paragraph(&p)),
                },
            }
        }
        for (title, whole) in led.into_iter().rev() {
            out.push(entry(&when, &day, title, whole));
        }
        if !intro.is_empty() {
            let mut title = if words.is_empty() { "What happened".to_string() } else { plain(&words) };
            if let Some(f) = title.get(..1) {
                title = f.to_uppercase() + &title[1..];
            }
            out.push(entry(&when, &day, title, intro));
        }
    }
    out
}

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// A heading's day in words: "2026-11-26" is "26 November 2026", and
/// "2026-11-10 to 11 (overnight)" is "10 to 11 November 2026 (overnight)".
pub fn date_words(when: &str) -> String {
    let Some(day) = when.get(..10) else { return when.to_string() };
    let (Ok(y), Ok(m), Ok(d)) = (day[..4].parse::<u32>(), day[5..7].parse::<usize>(), day[8..10].parse::<u32>()) else { return when.to_string() };
    let Some(month) = MONTHS.get(m.wrapping_sub(1)) else { return when.to_string() };
    let rest = when[10..].trim();
    if let Some(to) = rest.strip_prefix("to ") {
        let (d2, after) = to.split_once(' ').unwrap_or((to, ""));
        if d2.chars().all(|ch| ch.is_ascii_digit()) {
            return format!("{d} to {d2} {month} {y} {after}").trim_end().to_string();
        }
    }
    format!("{d} {month} {y} {rest}").trim_end().to_string()
}

/// The newest entry shown, as the file beside the notes keeps it.
#[derive(Clone, Debug, PartialEq)]
pub struct Seen {
    pub day: String,
    pub title: String,
}

impl Seen {
    pub fn of(e: &Entry) -> Seen {
        Seen { day: e.day.clone(), title: e.title.clone() }
    }

    pub fn text(&self) -> String {
        format!("The newest entry of docs/DEVLOG.md the game's What's new card has shown.\nday: {}\ntitle: {}\n", self.day, self.title)
    }

    pub fn from_text(text: &str) -> Option<Seen> {
        let day = text.lines().find_map(|l| l.strip_prefix("day: "))?.trim().to_string();
        let title = text.lines().find_map(|l| l.strip_prefix("title: "))?.trim().to_string();
        Some(Seen { day, title })
    }
}

/// How many entries, from the top, are new: those above the one last shown;
/// if it is no longer there, those of a later day; and if none was ever
/// shown, those of the last seven days (`today` and the six before, from
/// `week_start`).
pub fn new_count(entries: &[Entry], seen: Option<&Seen>, week_start: &str) -> usize {
    match seen {
        Some(s) => match entries.iter().position(|e| e.day == s.day && e.title == s.title) {
            Some(i) => i,
            None => entries.iter().take_while(|e| e.day.as_str() > s.day.as_str()).count(),
        },
        None => entries.iter().take_while(|e| e.day.as_str() >= week_start).count(),
    }
}

/// What the reading thread hands over.
struct Ready {
    entries: Vec<Entry>,
    new: usize,
    /// where the devlog was read from, or why it was not
    from: Result<PathBuf, String>,
}

#[derive(Resource)]
struct Reading(Arc<Mutex<Option<Ready>>>);

/// The card's state and what it shows.
#[derive(Resource, Default)]
pub struct WhatsNew {
    pub open: bool,
    /// the devlog has been read (or found missing)
    pub ready: bool,
    pub entries: Vec<Entry>,
    /// how many from the top are new
    pub new: usize,
    pub from: Option<Result<PathBuf, String>>,
    /// the card has been opened in this game, so the chip has gone
    pub looked: bool,
    /// where the newest entry shown is kept; None where nothing may be kept
    seen_at: Option<PathBuf>,
    /// how many entries have rows on the card so far
    laid: usize,
    /// raised when the card is to be shown or hidden
    version: u64,
}

/// The help card's button, and the chip at the start, that open the card.
#[derive(Component)]
pub struct OpenWhatsNew;
#[derive(Component)]
struct CardRoot;
#[derive(Component)]
struct CardBody;
#[derive(Component)]
struct CardHead;
#[derive(Component)]
struct CloseCard;
/// An entry's row; a click opens it whole, or shuts it again.
#[derive(Component)]
struct EntryRow(usize);
/// Where an entry's gist or its whole text stands.
#[derive(Component)]
struct EntryText(usize);
/// The chip on the screen a game starts from.
#[derive(Component)]
struct StartChip;
#[derive(Component)]
struct StartCount;

/// Rows laid on the card a frame while it fills.
const ROWS_A_FRAME: usize = 8;

pub fn plugin(app: &mut App) {
    app.init_resource::<WhatsNew>().add_systems(Startup, begin).add_systems(PostStartup, spawn_card).add_systems(Update, (arrive, clicks, open_entries, lay_rows, draw).chain());
}

/// Start reading the devlog on a thread of its own.
fn begin(mut commands: Commands, about: Option<Res<crate::note::About>>, mut wn: ResMut<WhatsNew>) {
    // beside the notes: in the saved games' folder, and only where a note may be kept
    let seen_at = about.as_ref().and_then(|a| a.folder.as_ref().ok()).map(|notes| notes.parent().unwrap_or(notes).join(SEEN));
    wn.seen_at = seen_at.clone();
    let slot = Arc::new(Mutex::new(None));
    commands.insert_resource(Reading(slot.clone()));
    let now = crate::saves::now();
    let today = crate::note::day_and_time(now).0;
    let week_start = crate::note::day_and_time(now.saturating_sub(6 * 86_400)).0;
    std::thread::spawn(move || {
        let t = std::time::Instant::now();
        let from = find_doc("DEVLOG.md").ok_or_else(|| "docs/DEVLOG.md is not beside the game here (it is looked for from the folder the game was started in, and beside the program)".to_string());
        let entries = match &from {
            Ok(p) => std::fs::read_to_string(p).map(|md| read(&md)).unwrap_or_default(),
            Err(_) => Vec::new(),
        };
        let seen = seen_at.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| Seen::from_text(&t));
        let new = new_count(&entries, seen.as_ref(), &week_start);
        let took = t.elapsed().as_secs_f64() * 1000.0;
        let since = match &seen {
            Some(s) => format!("since \"{}\" of {}", s.title, s.day),
            None => format!("of the seven days to {today}, as nothing has been shown before"),
        };
        match &from {
            Ok(p) => println!("what's new: {} entries read from {} in {took:.1} ms on a thread of their own; {new} new {since}", entries.len(), p.display()),
            Err(why) => println!("what's new: {why}"),
        }
        if let Ok(mut s) = slot.lock() {
            *s = Some(Ready { entries, new, from });
        }
    });
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
                        row.spawn(ui::text(f, "What's new", 22.0, INK));
                        ui::small_button(row, f, "Close", CloseCard);
                    });
                    c.spawn((ui::text(f, "Reading docs/DEVLOG.md...", 13.0, MUTED), CardHead));
                    ui::scroll_list(c, card, Node { flex_grow: 1.0, flex_shrink: 1.0, ..default() }, Node { flex_grow: 1.0, row_gap: Val::Px(2.0), padding: UiRect::right(Val::Px(ui::THUMB_ROOM)), ..default() }, CardBody, |_| {});
                });
        });
    // the chip on the screen a game starts from, at the top right under the tabs, hidden until there is something new
    commands
        .spawn((
            ui::card(Node {
                position_type: PositionType::Absolute,
                right: Val::Px(14.0),
                top: Val::Px(86.0),
                padding: UiRect::new(Val::Px(12.0), Val::Px(8.0), Val::Px(6.0), Val::Px(6.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                border_radius: BorderRadius::all(Val::Px(14.0)),
                ..default()
            }),
            OpenWhatsNew,
            StartChip,
            Visibility::Hidden,
        ))
        .with_children(|chip| {
            chip.spawn(ui::text(f, "What's new", 14.0, INK));
            chip.spawn((Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(9.0)), ..default() }, BackgroundColor(NEW_INK)))
                .with_children(|n| {
                    n.spawn((ui::text(f, "", 13.0, Color::WHITE), StartCount));
                });
        });
}

/// Take what the thread read, once it has.
fn arrive(reading: Option<Res<Reading>>, mut wn: ResMut<WhatsNew>, mut head: Query<&mut Text, With<CardHead>>, mut count: Query<&mut Text, (With<StartCount>, Without<CardHead>)>, mut chip: Query<&mut Visibility, With<StartChip>>) {
    if wn.ready {
        return;
    }
    let Some(ready) = reading.and_then(|r| r.0.lock().ok().and_then(|mut s| s.take())) else { return };
    wn.ready = true;
    wn.new = ready.new;
    let said = match &ready.from {
        Ok(_) => {
            let new = match ready.new {
                0 => "Nothing is new since you last looked.".to_string(),
                1 => "One is new since you last looked, marked so.".to_string(),
                n => format!("{n} are new since you last looked, marked so."),
            };
            format!("{} entries of docs/DEVLOG.md as it stood when the game began, the newest first. {new} A click on an entry opens it whole.", ready.entries.len())
        }
        Err(why) => format!("{why}, so there is nothing to show."),
    };
    for mut t in &mut head {
        t.0 = said.clone();
    }
    for mut t in &mut count {
        t.0 = ready.new.to_string();
    }
    if ready.new > 0 && !wn.looked {
        for mut v in &mut chip {
            *v = Visibility::Inherited;
        }
    }
    wn.entries = ready.entries;
    wn.from = Some(ready.from);
}

fn clicks(mut wn: ResMut<WhatsNew>, open: Query<&Interaction, (Changed<Interaction>, With<OpenWhatsNew>)>, close: Query<&Interaction, (Changed<Interaction>, With<CloseCard>)>, mut help: Query<&mut Visibility, With<ui::HelpCard>>, mut chip: Query<&mut Visibility, (With<StartChip>, Without<ui::HelpCard>)>, keys: Res<ButtonInput<KeyCode>>) {
    if open.iter().any(|i| *i == Interaction::Pressed) {
        wn.open = true;
        wn.version += 1;
        for mut v in &mut help {
            *v = Visibility::Hidden;
        }
        if !wn.looked && wn.ready {
            wn.looked = true;
            for mut v in &mut chip {
                *v = Visibility::Hidden;
            }
            // keep the newest entry shown, off the main thread: the marks stay for this game
            if let (Some(path), Some(newest)) = (wn.seen_at.clone(), wn.entries.first()) {
                let text = Seen::of(newest).text();
                std::thread::spawn(move || {
                    let _ = path.parent().map(std::fs::create_dir_all);
                    if let Err(e) = std::fs::write(&path, text) {
                        println!("what's new: could not keep the newest entry shown in {}: {}", path.display(), crate::saves::plainly(&e));
                    }
                });
            }
        }
    }
    if wn.open && (close.iter().any(|i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Escape)) {
        wn.open = false;
        wn.version += 1;
    }
}

/// A click on an entry opens it whole; a second shuts it.
fn open_entries(mut commands: Commands, wn: Res<WhatsNew>, font: Res<UiFont>, mut rows: Query<(&Interaction, &EntryRow, &mut ui::RowHover), Changed<Interaction>>, texts: Query<(Entity, &EntryText)>) {
    for (i, row, mut hover) in &mut rows {
        if *i != Interaction::Pressed {
            continue;
        }
        let Some(e) = wn.entries.get(row.0) else { continue };
        hover.open = !hover.open;
        let Some((at, _)) = texts.iter().find(|(_, t)| t.0 == row.0) else { continue };
        commands.entity(at).despawn_children();
        let f = &font.0;
        let open = hover.open;
        commands.entity(at).with_children(|p| {
            if open {
                for para in &e.whole {
                    p.spawn(ui::text(f, para, 13.5, INK));
                }
            } else {
                p.spawn(ui::text(f, &e.gist, 13.5, MUTED));
            }
        });
    }
}

/// Lay the card's rows a few a frame, while it is shut or open alike.
fn lay_rows(mut commands: Commands, mut wn: ResMut<WhatsNew>, font: Res<UiFont>, body: Query<Entity, With<CardBody>>) {
    if !wn.ready || wn.laid >= wn.entries.len() {
        return;
    }
    let Ok(body) = body.single() else { return };
    let f = &font.0;
    let (from, to) = (wn.laid, (wn.laid + ROWS_A_FRAME).min(wn.entries.len()));
    let new = wn.new;
    let entries = &wn.entries;
    commands.entity(body).with_children(|p| {
        for i in from..to {
            let e = &entries[i];
            if i == 0 || entries[i - 1].when != e.when {
                p.spawn((ui::text(f, &date_words(&e.when), 16.0, INK), Node { margin: UiRect::top(Val::Px(if i == 0 { 2.0 } else { 12.0 })), ..default() }));
                p.spawn((Node { height: Val::Px(1.0), margin: UiRect::bottom(Val::Px(2.0)), ..default() }, BackgroundColor(RULE)));
            }
            p.spawn((
                Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(2.0), padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() },
                BackgroundColor(Color::NONE),
                Interaction::default(),
                FocusPolicy::Block,
                ui::RowHover { open: false },
                EntryRow(i),
            ))
            .with_children(|row| {
                row.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() }).with_children(|line| {
                    if i < new {
                        line.spawn((Node { padding: UiRect::axes(Val::Px(6.0), Val::Px(0.0)), border_radius: BorderRadius::all(Val::Px(7.0)), ..default() }, BackgroundColor(NEW_INK))).with_children(|n| {
                            n.spawn(ui::text(f, "new", 12.0, Color::WHITE));
                        });
                    }
                    line.spawn((ui::text(f, &e.title, 15.0, INK), Node { flex_shrink: 1.0, ..default() }));
                });
                if let Some(found) = &e.found {
                    row.spawn(ui::text(f, &format!("In the game: {found}"), 13.5, crate::note::NOTE_INK));
                }
                row.spawn((Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), ..default() }, EntryText(i))).with_children(|t| {
                    t.spawn(ui::text(f, &e.gist, 13.5, MUTED));
                });
            });
        }
    });
    wn.laid = to;
    if to == wn.entries.len() {
        println!("what's new: the card's {to} rows are laid, {ROWS_A_FRAME} a frame, before it is opened");
    }
}

fn draw(wn: Res<WhatsNew>, mut root: Query<&mut Visibility, With<CardRoot>>, mut drawn: Local<Option<u64>>) {
    if *drawn == Some(wn.version) {
        return;
    }
    *drawn = Some(wn.version);
    for mut v in &mut root {
        v.set_if_neq(if wn.open { Visibility::Inherited } else { Visibility::Hidden });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD: &str = "# Devlog\n\nWorking notes.\n\n## 2026-11-26: what landed\n\n**The first.** It came first. N opens a line over the game. More of it.\n\n**The `second`: a [page](rules/x.md).** It came second, and a sentence that is long enough to be the whole gist of it without a second one following it here. Then a second sentence.\nIn the game: H, then What's new.\n\n## 2026-11-10 to 11 (overnight): a night of helpers\n\nStella went to bed. The rest.\n\n**Landed: the port** (`docs/rules/README.md`). The simulation runs under two rules.\n\n- `look.toml` holds every colour.\n- `--shots DIR` writes twelve views.\n\nAfter the list.\n\n## 2026-11-08: Rust core and 3D viewer\n\n- `crates/isostate-core`: full port.\n- the viewer.\n";

    #[test]
    fn the_devlog_is_read_newest_first_with_its_titles_and_gists() {
        let es = read(MD);
        let titles: Vec<&str> = es.iter().map(|e| e.title.as_str()).collect();
        // the day's last entry first; an intro is its day's oldest; a section with no bold lead is one entry
        assert_eq!(titles, ["The second: a page", "The first", "Landed: the port", "A night of helpers", "Rust core and 3D viewer"]);
        assert_eq!(es[0].found.as_deref(), Some("H, then What's new."));
        assert_eq!(es[0].gist, "It came second, and a sentence that is long enough to be the whole gist of it without a second one following it here.");
        // a sentence that names a key is where it is found, and is not said twice
        assert_eq!(es[1].found.as_deref(), Some("N opens a line over the game."));
        assert_eq!(es[1].gist, "It came first. More of it.");
        // what follows a bold lead belongs to it, a list a point a string
        assert_eq!(es[2].whole, ["(docs/rules/README.md). The simulation runs under two rules.", "\u{2022} look.toml holds every colour.", "\u{2022} --shots DIR writes twelve views.", "After the list."]);
        assert_eq!(es[2].found, None);
        // the bracket naming its page is left out of the gist
        assert_eq!(es[2].gist, "The simulation runs under two rules. look.toml holds every colour.");
        assert_eq!((es[2].when.as_str(), es[2].day.as_str()), ("2026-11-10 to 11 (overnight)", "2026-11-10"));
        assert_eq!(es[3].gist, "Stella went to bed. The rest.");
        assert_eq!(es[4].gist, "crates/isostate-core: full port. the viewer.");
        assert_eq!(date_words(&es[2].when), "10 to 11 November 2026 (overnight)");
        assert_eq!(date_words("2026-11-09 (evening)"), "9 November 2026 (evening)");
        assert_eq!(date_words("2026-11-26"), "26 November 2026");
    }

    #[test]
    fn sentences_are_cut_at_their_ends_only() {
        assert_eq!(sentences("Made roads fall from 2,886 km in 1900 to 1,325. Rail carries 0.91 of it (rules/lorry.md.) \"So it is.\" 1 in 35.3 is a gradient."), ["Made roads fall from 2,886 km in 1900 to 1,325.", "Rail carries 0.91 of it (rules/lorry.md.)", "\"So it is.\"", "1 in 35.3 is a gradient."]);
        assert_eq!(sentences("e.g. this is one. "), ["e.g. this is one."]);
        assert_eq!(sentences("It builds in seven seconds. run_tests.py takes --quick."), ["It builds in seven seconds.", "run_tests.py takes --quick."]);
        assert!(names_a_place("A town's card in a sandbox has a Making chip that opens a flow chart."));
        assert!(names_a_place("A Traffic map view draws each way as wide as what it carried."));
        assert!(names_a_place("N opens a line over the game for a note."));
        assert!(names_a_place("The town's card names it; the Works tool founds or closes one by hand."));
        assert!(names_a_place("A chip in the Rules tab turns it on."));
        assert!(names_a_place("The help card (H) ends with a button."));
        // the older days' other cards, panels and keys
        assert!(!names_a_place("A road nobody uses for six years falls back."));
        assert!(!names_a_place("I think the island is a cosy place."));
        assert!(!names_a_place("H moves nothing beyond the null pair."));
        assert!(!names_a_place("G closes a few works sooner, and Stage 1's card was checked."));
        assert!(!names_a_place("Read on seven histories, nothing on the card moves."));
        assert!(!names_a_place("Thing is a hashable key, and the tree's panels wait."));
        assert!(!names_a_place("It renders 0.45 M on the graphics card."));
        assert!(!names_a_place("(repairs, map views and charts)."));
    }

    #[test]
    fn what_is_new_is_what_stands_above_the_entry_last_shown() {
        let es = read(MD);
        let seen = Seen::of(&es[2]);
        assert_eq!(Seen::from_text(&seen.text()), Some(seen.clone()));
        assert_eq!(new_count(&es, Some(&seen), "2026-11-20"), 2);
        // the newest shown: nothing is new
        assert_eq!(new_count(&es, Some(&Seen::of(&es[0])), "2026-11-20"), 0);
        // an entry shown that is no longer there: those of a later day
        assert_eq!(new_count(&es, Some(&Seen { day: "2026-11-10".into(), title: "Gone".into() }), "2026-11-20"), 2);
        // the first time: the last seven days
        assert_eq!(new_count(&es, None, "2026-11-20"), 2);
        assert_eq!(new_count(&es, None, "2026-11-04"), 5);
    }

    /// The real devlog, as it stands in the checkout: every dated section gives
    /// entries, none of them empty or with markdown left in, the newest first.
    #[test]
    fn the_real_devlog_reads_whole() {
        let md = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/DEVLOG.md")).expect("docs/DEVLOG.md");
        let es = read(&md);
        let sections = md.lines().filter(|l| l.starts_with("## ")).count();
        let led = md.lines().filter(|l| l.starts_with("**")).count();
        // every bold lead is an entry, and each section with none, or with words before its first, one more
        assert!(es.len() >= led && es.len() <= led + sections, "{} entries from {led} bold leads in {sections} sections", es.len());
        assert!(es.len() >= 190, "{} entries", es.len());
        let newest = md.lines().find_map(|l| l.strip_prefix("## ")).and_then(|h| h.get(..10)).unwrap();
        assert_eq!(es[0].day, newest);
        assert!(es[0].day.as_str() >= "2026-11-26");
        let mut days: Vec<&str> = es.iter().map(|e| e.day.as_str()).collect();
        assert!(days.windows(2).all(|w| w[0] >= w[1]), "the newest first");
        days.dedup();
        assert_eq!(days.len(), md.lines().filter(|l| l.starts_with("## ")).filter_map(|l| l.get(3..13)).collect::<std::collections::BTreeSet<_>>().len());
        for e in &es {
            let what = format!("{} ({})", e.title, e.when);
            assert!(!e.title.is_empty() && e.title.chars().count() < 200, "{what}: its title");
            assert!(!e.gist.is_empty() && !e.whole.is_empty(), "{what}: its gist");
            for s in std::iter::once(&e.title).chain(std::iter::once(&e.gist)).chain(&e.whole).chain(&e.found) {
                assert!(!s.contains("**") && !s.contains('`') && !s.contains("]("), "{what}: markdown left in {s}");
            }
        }
        println!("{} entries, the newest of {}; {} say where they are found in the game", es.len(), es[0].day, es.iter().filter(|e| e.found.is_some()).count());    }
}
