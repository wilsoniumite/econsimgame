//! A note in her own words, kept where she stands (N). Whoever is at the
//! window presses N in a live game, a sandbox or a recorded run, and one line
//! opens over the game to type in: Enter keeps it, Escape drops it. The clock
//! stops while the line is open and goes on as it was after. What is kept is
//! her words exactly as typed, with where she stood: the island and its
//! rules, the year and month on the clock, where the camera was (enough to
//! stand there again: `--at`, `--yaw`, `--pitch`, `--dist`), the town or
//! square chosen, the panel and the map view open, the session the game is
//! kept in, the build's commit and the time; and a picture of the screen as it
//! stood before the line opened. A toast says it was kept and how many there
//! are today.
//!
//! Each note is one small text file and one picture, named by the day and its
//! number in the day (`2026-11-26-03.txt` and `.png`), in a folder of their
//! own beside the sessions (`saves::notes`). The managing session reads them
//! there and files the words in docs/BACKLOG.md.
//!
//! The line is the least that works: letters as the keyboard gives them
//! (Shift and accents included), Backspace, and a paste with Ctrl+V or
//! Shift+Insert, which asks the system for its clipboard on a thread of its
//! own. It has no caret to move, no selection and no undo; a newline pasted
//! becomes a space, since a note is one line.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bevy::camera::RenderTarget;
use bevy::ecs::system::SystemParam;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};

use crate::ui::{self, MarketsShow, Tab, UiState, INK, MUTED};
use crate::{clock, Data, MainCam, Orbit, Pick, Picked, Timeline, Toasts, UiFont};

/// The key that opens the line.
pub const KEY: KeyCode = KeyCode::KeyN;

/// Frames the line waits for its picture before it opens without one.
const PICTURE_PATIENCE: u32 = 90;

/// What is known of the game from its start and does not change while it is
/// played: set once, in `main`.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct About {
    /// "Island 7 with 6 families, under the game's rules, on the new land"
    pub island: String,
    /// where the game is kept, in words: "kept in" the session, the autosave
    /// or a file, "read from" a recorded run's file, or "not kept"
    pub kept_in: String,
    /// the folder notes go to; None where a note may not be kept, with why
    pub folder: Result<PathBuf, String>,
}

impl Default for About {
    fn default() -> Self {
        About { island: "an island".into(), kept_in: "kept nowhere said".into(), folder: Err("the game did not say where notes go".into()) }
    }
}

/// The build's commit, as `build.rs` found it.
pub const COMMIT: &str = env!("ISOSTATE_COMMIT");

/// Where she stood when she pressed N: everything of the note but her words
/// and the time it was kept.
#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    /// "a sandbox", "the family game", "a recorded run"
    pub game: String,
    /// the date on the clock, "March 1791", and the year the map is of
    pub date: String,
    pub map_year: i32,
    /// the point the camera looks at, in map cells, its turn and tilt in
    /// degrees, and how far back it stands, in map cells
    pub at: (f32, f32),
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    /// the town, works or square chosen, in words; None if nothing
    pub chosen: Option<String>,
    /// the side panel open, in words; None if none
    pub panel: Option<String>,
    /// the map view, as `--overlay` takes it ("rent", "price:coal"); None for the plain map
    pub map: Option<String>,
    /// the town whose Making panel is open
    pub making: Option<String>,
}

/// A note as it is written to its file.
#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub words: String,
    pub place: Place,
    pub about: About,
    /// its name in the folder, without the ending: "2026-11-26-03"
    pub name: String,
    /// its number in the day
    pub number: usize,
    /// when it was kept, "2026-11-26 05:41 UTC"
    pub when: String,
    /// whether a picture is kept with it
    pub picture: bool,
}

impl Note {
    /// The note's text file: her words first, as typed, then a line a thing.
    pub fn text(&self) -> String {
        let p = &self.place;
        let none = |o: &Option<String>| o.clone().unwrap_or_else(|| "none".to_string());
        let camera = format!("--at {:.3},{:.3} --yaw {:.2} --pitch {:.2} --dist {:.3}", p.at.0, p.at.1, p.yaw, p.pitch, p.dist);
        let overlay = p.map.as_ref().map(|m| format!(" --overlay {m}")).unwrap_or_default();
        let view_word = p.map.as_ref().map(|m| format!(",{m}")).unwrap_or_default();
        let mut s = String::new();
        s.push_str(&format!("words: {}\n", self.words));
        s.push_str(&format!("kept: {}, the {} note of the day\n", self.when, ordinal(self.number)));
        s.push_str(&format!("island: {}\n", self.about.island));
        s.push_str(&format!("game: {}, {}\n", p.game, self.about.kept_in));
        s.push_str(&format!("clock: {} (the map is of {})\n", p.date, p.map_year));
        s.push_str(&format!("camera: looking at {:.3}, {:.3} in map cells, turned {:.2} degrees, tilted {:.2} above the horizon, {:.3} cells back\n", p.at.0, p.at.1, p.yaw, p.pitch, p.dist));
        s.push_str(&format!("chosen: {}\n", none(&p.chosen)));
        s.push_str(&format!("panel: {}\n", none(&p.panel)));
        s.push_str(&format!("map view: {}\n", p.map.clone().unwrap_or_else(|| "plain".to_string())));
        s.push_str(&format!("making panel: {}\n", none(&p.making)));
        s.push_str(&format!("build: {COMMIT}\n"));
        s.push_str(&format!("picture: {}\n", if self.picture { format!("{}.png", self.name) } else { "none (the renderer gave none in time)".to_string() }));
        s.push_str(&format!("stand here again: {camera}{overlay}\n"));
        s.push_str(&format!("as a picture: --view {}-again.png,{},{:.3},{:.3},{:.2},{:.2},{:.3}{view_word}\n", self.name, p.map_year, p.at.0, p.at.1, p.yaw, p.pitch, p.dist));
        s
    }
}

/// Read the words and the camera back from a note's text, as a check that
/// what was kept is enough: (words, at, yaw, pitch, dist).
#[cfg(test)]
pub fn read_back(text: &str) -> Option<(String, (f32, f32), f32, f32, f32)> {
    let words = text.lines().next()?.strip_prefix("words: ")?.to_string();
    let line = text.lines().find_map(|l| l.strip_prefix("stand here again: "))?;
    let mut it = line.split_whitespace();
    let (mut at, mut yaw, mut pitch, mut dist) = (None, None, None, None);
    while let Some(opt) = it.next() {
        let v = it.next();
        match opt {
            "--at" => at = v.and_then(|v| v.split_once(',')).and_then(|(x, z)| Some((x.parse().ok()?, z.parse().ok()?))),
            "--yaw" => yaw = v.and_then(|v| v.parse().ok()),
            "--pitch" => pitch = v.and_then(|v| v.parse().ok()),
            "--dist" => dist = v.and_then(|v| v.parse().ok()),
            _ => {}
        }
    }
    Some((words, at?, yaw?, pitch?, dist?))
}

fn ordinal(n: usize) -> String {
    const WORDS: [&str; 10] = ["first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth"];
    match WORDS.get(n.wrapping_sub(1)) {
        Some(w) => w.to_string(),
        None => {
            let end = match (n % 10, n % 100) {
                (1, x) if x != 11 => "st",
                (2, x) if x != 12 => "nd",
                (3, x) if x != 13 => "rd",
                _ => "th",
            };
            format!("{n}{end}")
        }
    }
}

/// The day and the time of day of a moment, counted in seconds since 1970,
/// in Greenwich time: ("2026-11-26", "05:41").
pub fn day_and_time(secs: u64) -> (String, String) {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // the civil calendar from a count of days (H. Hinnant's days_from_civil, turned round)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    (format!("{y:04}-{m:02}-{d:02}"), format!("{:02}:{:02}", rem / 3_600, rem % 3_600 / 60))
}

/// The notes of `day` in `folder`, by number, oldest first: (number, path of the text).
pub fn of_the_day(folder: &Path, day: &str) -> Vec<(usize, PathBuf)> {
    let mut found: Vec<(usize, PathBuf)> = std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let n = name.strip_prefix(day)?.strip_prefix('-')?.strip_suffix(".txt")?.parse::<usize>().ok()?;
            Some((n, e.path()))
        })
        .collect();
    found.sort();
    found
}

/// The name the next note of `day` takes, and its number: one past the highest of the day.
pub fn next_name(folder: &Path, day: &str) -> (String, usize) {
    let n = of_the_day(folder, day).last().map_or(0, |(n, _)| *n) + 1;
    (format!("{day}-{n:02}"), n)
}

/// Write a note's text, and say what was written. The picture is written apart, off the main thread.
pub fn write(folder: &Path, note: &Note) -> Result<PathBuf, String> {
    std::fs::create_dir_all(folder).map_err(|e| format!("cannot make {}: {}", folder.display(), crate::saves::plainly(&e)))?;
    let path = folder.join(format!("{}.txt", note.name));
    std::fs::write(&path, note.text()).map_err(|e| format!("cannot write {}: {}", path.display(), crate::saves::plainly(&e)))?;
    Ok(path)
}

/// Pictures still being written, which a scripted run waits for before it stops.
static WRITING: AtomicUsize = AtomicUsize::new(0);

/// Whether a note's picture is still being written.
pub fn writing() -> bool {
    WRITING.load(Ordering::Relaxed) > 0
}

fn write_picture(image: Image, path: PathBuf) {
    WRITING.fetch_add(1, Ordering::Relaxed);
    std::thread::spawn(move || {
        let said = match image.try_into_dynamic() {
            Ok(d) => d.to_rgb8().save(&path).map_err(|e| e.to_string()),
            Err(e) => Err(format!("{e:?}")),
        };
        if let Err(e) = said {
            eprintln!("note: the picture {} could not be written: {e}", path.display());
        }
        WRITING.fetch_sub(1, Ordering::Relaxed);
    });
}

// ------------------------------------------------------------------ the line

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Stage {
    #[default]
    Shut,
    /// N was pressed and the picture is on its way: the line is not drawn yet
    Taking(u32),
    Open,
}

#[derive(Resource, Default)]
pub struct NoteLine {
    stage: Stage,
    /// what has been typed
    pub words: String,
    place: Option<Place>,
    shot: Arc<Mutex<Option<Image>>>,
    /// the clock was running when the line opened
    was_playing: bool,
    ctrl: bool,
    shift: bool,
    /// a paste asked of the system's clipboard, and what it gave once it has come
    paste: Option<Arc<Mutex<Option<String>>>>,
    /// raised whenever what the line shows changes
    version: u64,
    /// the last note kept in this window: its text's path
    pub last: Option<PathBuf>,
    /// what the card says once Enter is pressed, when, and the words it was said of
    told: Option<(String, f32, String)>,
}

impl NoteLine {
    /// For a script's print: whether it is open, and what is typed.
    pub fn said(&self) -> String {
        match self.stage {
            Stage::Shut => "shut".to_string(),
            Stage::Taking(_) => format!("opening, its picture on its way, typed \"{}\"", self.words),
            Stage::Open => format!("open, typed \"{}\"", self.words),
        }
    }
}

#[derive(Component)]
struct LineRoot;
#[derive(Component)]
struct LineWords;
#[derive(Component)]
struct LineDate;
#[derive(Component)]
struct LineFoot;

/// How long the card stays up after Enter, saying the note is kept, in seconds.
const TOLD_FOR: f32 = 4.0;
/// What the card says under the line while it is open.
const FOOT: &str = "Enter keeps it, with a picture of the screen and where you stand. Escape drops it. Ctrl+V pastes.";

/// What a note says of where she stands.
#[derive(SystemParam)]
pub struct Where<'w, 's> {
    data: Res<'w, Data>,
    orbit: Res<'w, Orbit>,
    picked: Res<'w, Picked>,
    ui: Res<'w, UiState>,
    overlay: Res<'w, crate::overlay::MapOverlay>,
    flow: Option<Res<'w, crate::town_flow::Flow>>,
    sandbox: Option<Res<'w, crate::sandbox::Sandbox>>,
    live: Option<Res<'w, crate::live::Live>>,
    /// the date as the bar across the top shows it
    date: Query<'w, 's, &'static Text, With<ui::DateText>>,
}

impl Where<'_, '_> {
    fn place(&self, tl: &Timeline) -> Place {
        let r = &self.data.r;
        let game = if self.sandbox.is_some() {
            "a sandbox"
        } else if self.live.is_some() {
            "the family game"
        } else {
            "a recorded run"
        };
        let latest = self.data.latest();
        let k = tl.built.unwrap_or(latest).min(latest);
        // the date as she sees it on the bar (a month, or a year alone on a turn gone by)
        let date = self.date.iter().next().map(|t| t.0.trim().to_string()).filter(|d| !d.is_empty()).unwrap_or_else(|| clock::month_year(clock::date(r.y0, tl.t)));
        let chosen = self.picked.what.map(|p| match p {
            Pick::Town(t) => format!("the town of {}", r.header.towns.get(t).map_or("?", |t| t.name.as_str())),
            Pick::Firm(f) => match r.header.firms.get(f) {
                Some(firm) => format!("the works {}, about {} km east and {} km south of the north-west corner", firm.name, (firm.cell % r.w) * 2 + 1, (firm.cell / r.w) * 2 + 1),
                None => "a works".to_string(),
            },
            Pick::Pit { cell, class } => format!("the {} about {} km east and {} km south of the north-west corner", crate::DISP_NAMES[class as usize % 12], (cell % r.w) * 2 + 1, (cell / r.w) * 2 + 1),
            Pick::Place { x, z, .. } => format!("the spot at {x:.2}, {z:.2} on the map, {:.1} km east and {:.1} km south of the north-west corner", x * 2.0, z * 2.0),
        });
        let panel = self.ui.tab.map(|t| {
            let name = match t {
                Tab::Family if self.sandbox.is_some() => "Families",
                Tab::Family => "Family",
                Tab::Holdings => "Holdings",
                Tab::Markets => "Markets",
                Tab::News => "News",
                Tab::Hand => "Hand",
                Tab::Rules => "Rules",
            };
            match t {
                Tab::Markets => format!(
                    "{name}, {}{}",
                    match self.ui.markets {
                        MarketsShow::Prices => "the goods",
                        MarketsShow::Island => "the island",
                        MarketsShow::Work => "work",
                    },
                    self.ui.good.and_then(|g| r.header.goods.get(g)).map(|g| format!(", {} open", g.label)).unwrap_or_default()
                ),
                _ => name.to_string(),
            }
        });
        let map = self.overlay.word(r);
        let making = self.flow.as_ref().and_then(|f| f.town()).and_then(|t| r.header.towns.get(t)).map(|t| t.name.clone());
        let o = &self.orbit;
        Place {
            game: game.to_string(),
            date,
            map_year: r.y0 + k as i32,
            at: (o.target.x, o.target.z),
            yaw: o.yaw.to_degrees(),
            pitch: o.pitch.to_degrees(),
            dist: o.dist,
            chosen,
            panel,
            map,
            making,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<NoteLine>()
        .add_systems(PostStartup, setup_line)
        // after the keyboard has been read (and a script has pressed its keys), before anything else looks at it
        .add_systems(PreUpdate, keys.after(bevy::input::InputSystems).after(bevy::ui::UiSystems::Focus))
        .add_systems(Update, draw_line);
}

fn setup_line(mut commands: Commands, font: Res<UiFont>) {
    let f = &font.0;
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, left: Val::Px(0.0), right: Val::Px(0.0), bottom: Val::Px(84.0), justify_content: JustifyContent::Center, ..default() },
            GlobalZIndex(40),
            Visibility::Hidden,
            LineRoot,
        ))
        .with_children(|wrap| {
            wrap.spawn(ui::card(Node { flex_direction: FlexDirection::Column, width: Val::Px(660.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(14.0)), row_gap: Val::Px(6.0), border: UiRect::left(Val::Px(5.0)), border_radius: BorderRadius::all(Val::Px(14.0)), ..default() }))
                .insert((GlobalZIndex(40), BorderColor::all(NOTE_INK)))
                .with_children(|c| {
                    c.spawn((ui::text(f, "A note, in your words", 13.5, NOTE_INK), LineDate));
                    c.spawn((ui::text(f, "|", 19.0, INK), LineWords));
                    c.spawn((ui::text(f, FOOT, 12.5, MUTED), LineFoot));
                });
        });
}

/// The ink of a note: a deep green, apart from the hand's blue-black and the news's colours.
pub const NOTE_INK: Color = Color::srgb(0.20, 0.42, 0.36);

/// The line while it is open; and for a few seconds after Enter, the same
/// card saying the note is kept, since a toast can stand under a panel or the
/// tray, or be hidden by the News panel.
#[allow(clippy::type_complexity)]
fn draw_line(
    line: Res<NoteLine>,
    time: Res<Time>,
    mut root: Query<&mut Visibility, With<LineRoot>>,
    mut texts: ParamSet<(Query<&mut Text, With<LineDate>>, Query<&mut Text, With<LineWords>>, Query<&mut Text, With<LineFoot>>)>,
    mut drawn: Local<Option<(u64, bool)>>,
) {
    let told = line.told.as_ref().filter(|(_, at, _)| time.elapsed_secs() - at < TOLD_FOR);
    let state = (line.version, told.is_some());
    if *drawn == Some(state) {
        return;
    }
    *drawn = Some(state);
    let open = line.stage == Stage::Open;
    let want = if open || told.is_some() { Visibility::Inherited } else { Visibility::Hidden };
    for mut v in &mut root {
        v.set_if_neq(want);
    }
    let date = line.place.as_ref().map_or(String::new(), |p| format!(" \u{00b7} {}", p.date));
    let (head, words, foot) = match (open, told) {
        (false, Some((said, _, words))) => (format!("Your note{date}"), words.clone(), said.clone()),
        _ => (format!("A note, in your words{date}"), format!("{}|", line.words), FOOT.to_string()),
    };
    for mut t in &mut texts.p0() {
        t.0 = head.clone();
    }
    for mut t in &mut texts.p1() {
        t.0 = words.clone();
    }
    for mut t in &mut texts.p2() {
        t.0 = foot.clone();
    }
}

/// The keys while the line is open, and N to open it.
#[allow(clippy::too_many_arguments)]
pub fn keys(
    mut commands: Commands,
    mut line: ResMut<NoteLine>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut typed: MessageReader<KeyboardInput>,
    mut tl: ResMut<Timeline>,
    cards: Res<crate::game::CardOpen>,
    at: Where,
    about: Option<Res<About>>,
    cams: Query<&RenderTarget, With<MainCam>>,
    mut toasts: ResMut<Toasts>,
    time: Res<Time>,
) {
    let line = &mut *line;
    if line.stage == Stage::Shut {
        // what was typed while the line was shut is the game's, N's own letter among it
        typed.clear();
        if cards.0 || !keys.just_pressed(KEY) || keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) || keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight) {
            return;
        }
        line.place = Some(at.place(&tl));
        line.words.clear();
        line.told = None;
        line.was_playing = tl.playing;
        tl.playing = false;
        // the picture of the screen as it stands, before the line is drawn over it
        let slot = Arc::new(Mutex::new(None));
        line.shot = slot.clone();
        let shot = match cams.iter().next() {
            Some(RenderTarget::Image(target)) => Screenshot::image(target.handle.clone()),
            _ => Screenshot::primary_window(),
        };
        commands.spawn(shot).observe(move |got: On<ScreenshotCaptured>| {
            if let Ok(mut s) = slot.lock() {
                *s = Some(got.image.clone());
            }
        });
        line.stage = Stage::Taking(0);
        line.version += 1;
        line.ctrl = false;
        line.shift = false;
        keys.reset_all();
        return;
    }
    // the clock stays stopped while the line is open (a click on a speed would start it)
    tl.playing = false;
    if let Stage::Taking(n) = line.stage {
        let come = line.shot.lock().is_ok_and(|s| s.is_some());
        line.stage = if come || n >= PICTURE_PATIENCE { Stage::Open } else { Stage::Taking(n + 1) };
        if line.stage == Stage::Open {
            line.version += 1;
        }
    }
    let (mut enter, mut escape, mut back) = (false, false, 0usize);
    let (mut enter_msg, mut escape_msg, mut back_msg) = (false, false, false);
    let mut paste = false;
    for ev in typed.read() {
        let down = ev.state == ButtonState::Pressed;
        match ev.key_code {
            KeyCode::ControlLeft | KeyCode::ControlRight => line.ctrl = down,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => line.shift = down,
            _ => {}
        }
        if !down {
            continue;
        }
        match &ev.logical_key {
            Key::Enter => (enter, enter_msg) = (true, true),
            Key::Escape => (escape, escape_msg) = (true, true),
            Key::Backspace => {
                back += 1;
                back_msg = true;
            }
            Key::Insert if line.shift => paste = true,
            Key::Character(c) if line.ctrl && c.eq_ignore_ascii_case("v") => paste = true,
            _ if line.ctrl => {}
            _ => {
                if let Some(t) = &ev.text {
                    line.words.extend(t.chars().filter(|c| !c.is_control()));
                    line.version += 1;
                }
            }
        }
    }
    // a script presses keys without the keyboard's messages
    enter |= !enter_msg && keys.just_pressed(KeyCode::Enter);
    escape |= !escape_msg && keys.just_pressed(KeyCode::Escape);
    if !back_msg && keys.just_pressed(KeyCode::Backspace) {
        back += 1;
    }
    for _ in 0..back {
        line.words.pop();
        line.version += 1;
    }
    if paste {
        line.paste = Some(ask_clipboard());
    }
    let pasted = line.paste.as_ref().and_then(|slot| slot.lock().ok().and_then(|mut s| s.take()));
    if let Some(text) = pasted {
        line.words.push_str(&one_line(&text));
        line.version += 1;
        line.paste = None;
    }
    keys.reset_all();
    if !(enter || escape) {
        return;
    }
    let now = time.elapsed_secs();
    let when = line.place.as_ref().map_or(String::new(), |p| p.date.clone());
    if enter && !line.words.trim().is_empty() {
        let about = about.map(|a| a.clone()).unwrap_or_default();
        let picture = line.shot.lock().ok().and_then(|mut s| s.take());
        match about.folder.clone() {
            Ok(folder) => {
                let (day, time_of_day) = day_and_time(crate::saves::now());
                let (name, number) = next_name(&folder, &day);
                let note = Note { words: line.words.clone(), place: line.place.clone().unwrap_or_else(|| at.place(&tl)), about, name: name.clone(), number, when: format!("{day} {time_of_day} UTC"), picture: picture.is_some() };
                match write(&folder, &note) {
                    Ok(path) => {
                        if let Some(img) = picture {
                            write_picture(img, folder.join(format!("{name}.png")));
                        }
                        println!("note: kept {} ({} words)", path.display(), note.words.split_whitespace().count());
                        line.last = Some(path);
                        let said = format!("Kept, the {} note today{}.", ordinal(number), if note.picture { ", with a picture of the screen" } else { "" });
                        toasts.say("note", &said, &when, now);
                        line.told = Some((said, now, line.words.clone()));
                    }
                    Err(why) => {
                        eprintln!("note: not kept: {why}");
                        let said = format!("The note could not be kept: {why}");
                        toasts.say("orders_refused", &said, &when, now);
                        line.told = Some((said, now, line.words.clone()));
                    }
                }
            }
            Err(why) => {
                println!("note: not kept: {why}");
                let said = format!("The note is not kept: {why}");
                toasts.say("orders_refused", &said, &when, now);
                line.told = Some((said, now, line.words.clone()));
            }
        }
    } else if enter {
        toasts.say("note", "Nothing was typed, so nothing was kept.", &when, now);
    } else {
        println!("note: dropped");
    }
    line.stage = Stage::Shut;
    line.words.clear();
    line.paste = None;
    line.version += 1;
    tl.playing = line.was_playing;
}

/// What is pasted, on one line.
pub fn one_line(text: &str) -> String {
    let mut s = String::new();
    for part in text.split(['\r', '\n']).filter(|p| !p.is_empty()) {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(part);
    }
    s.chars().filter(|c| !c.is_control() || *c == '\t').map(|c| if c == '\t' { ' ' } else { c }).collect()
}

/// Ask the system for the text on its clipboard, on a thread of its own, so
/// that the view does not wait for it.
fn ask_clipboard() -> Arc<Mutex<Option<String>>> {
    let slot = Arc::new(Mutex::new(None));
    let tx = slot.clone();
    std::thread::spawn(move || {
        let tries: &[(&str, &[&str])] = if cfg!(windows) {
            &[("powershell", &["-NoProfile", "-NonInteractive", "-Command", "Get-Clipboard -Raw"])]
        } else if cfg!(target_os = "macos") {
            &[("pbpaste", &[])]
        } else {
            &[("wl-paste", &["-n"]), ("xclip", &["-o", "-selection", "clipboard"])]
        };
        for (cmd, args) in tries {
            let mut c = std::process::Command::new(cmd);
            c.args(*args).stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null());
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                // no console window flashing up over the game
                c.creation_flags(0x0800_0000);
            }
            if let Ok(out) = c.output() {
                if out.status.success() {
                    if let Ok(mut s) = tx.lock() {
                        *s = Some(String::from_utf8_lossy(&out.stdout).into_owned());
                    }
                    return;
                }
            }
        }
        // (nothing to paste: the slot stays empty, and the line as it was)
    });
    slot
}

/// A script's `type:TEXT` step: each letter as the keyboard would give it,
/// pressed and let go, through the same messages a player's typing makes.
pub fn type_text(world: &mut World, text: &str) {
    for c in text.chars() {
        for state in [ButtonState::Pressed, ButtonState::Released] {
            world.write_message(KeyboardInput {
                key_code: KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified),
                logical_key: Key::Character(c.to_string().into()),
                state,
                text: (state == ButtonState::Pressed).then(|| c.to_string().into()),
                repeat: false,
                window: Entity::PLACEHOLDER,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_place() -> Place {
        Place {
            game: "a sandbox".into(),
            date: "March 1791".into(),
            map_year: 1790,
            at: (31.5, 29.25),
            yaw: 25.0,
            pitch: 52.0,
            dist: 14.0,
            chosen: Some("the town of Glenwick".into()),
            panel: Some("Markets, the island".into()),
            map: Some("price:coal".into()),
            making: Some("Glenwick".into()),
        }
    }

    fn a_note(words: &str) -> Note {
        let about = About { island: "Island 7 with 6 families, under the game's rules, on the new land".into(), kept_in: "kept in sandbox/island-7-families-6.json".into(), folder: Ok(PathBuf::from("notes")) };
        Note { words: words.into(), place: a_place(), about, name: "2026-11-26-03".into(), number: 3, when: "2026-11-26 05:41 UTC".into(), picture: true }
    }

    #[test]
    fn a_notes_file_holds_her_words_as_typed_and_every_field() {
        // accents, capitals, punctuation and a colon of her own are hers, untouched
        let words = "Dunheath's towers look like toys: café-height, NOT 12 storeys!  (two spaces)";
        let text = a_note(words).text();
        assert_eq!(text.lines().next(), Some(format!("words: {words}").as_str()));
        for field in ["kept: 2026-11-26 05:41 UTC, the third note of the day", "island: Island 7 with 6 families", "game: a sandbox, kept in sandbox/island-7-families-6.json", "clock: March 1791 (the map is of 1790)", "camera: looking at 31.500, 29.250", "chosen: the town of Glenwick", "panel: Markets, the island", "map view: price:coal", "making panel: Glenwick", "build: ", "picture: 2026-11-26-03.png", "stand here again: --at 31.500,29.250 --yaw 25.00 --pitch 52.00 --dist 14.000 --overlay price:coal", "as a picture: --view 2026-11-26-03-again.png,1790,31.500,29.250,25.00,52.00,14.000,price:coal"] {
            assert!(text.lines().any(|l| l.starts_with(field)), "no line starting {field:?} in\n{text}");
        }
        // and nothing chosen reads as none, the plain map as plain
        let mut bare = a_note("x");
        (bare.place.chosen, bare.place.panel, bare.place.map, bare.place.making, bare.picture) = (None, None, None, None, false);
        let text = bare.text();
        for field in ["chosen: none", "panel: none", "map view: plain", "making panel: none", "picture: none", "stand here again: --at 31.500,29.250 --yaw 25.00 --pitch 52.00 --dist 14.000\n"] {
            assert!(text.contains(field), "no {field:?} in\n{text}");
        }
    }

    #[test]
    fn the_camera_kept_reads_back_to_the_same_place() {
        let note = a_note("a word");
        let (words, at, yaw, pitch, dist) = read_back(&note.text()).unwrap();
        assert_eq!(words, "a word");
        let p = &note.place;
        assert!((at.0 - p.at.0).abs() < 1e-3 && (at.1 - p.at.1).abs() < 1e-3);
        assert!((yaw - p.yaw).abs() < 1e-2 && (pitch - p.pitch).abs() < 1e-2 && (dist - p.dist).abs() < 1e-3);
    }

    #[test]
    fn notes_are_named_by_the_day_and_numbered_in_it() {
        let dir = crate::saves::test_dir("notes-named");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(next_name(&dir, "2026-11-26"), ("2026-11-26-01".to_string(), 1));
        let mut note = a_note("first");
        (note.name, note.number) = next_name(&dir, "2026-11-26");
        write(&dir, &note).unwrap();
        assert_eq!(next_name(&dir, "2026-11-26"), ("2026-11-26-02".to_string(), 2));
        // another day's notes and other files do not count
        std::fs::write(dir.join("2026-11-25-07.txt"), "words: older\n").unwrap();
        std::fs::write(dir.join("2026-11-26-01.png"), b"").unwrap();
        std::fs::write(dir.join("2026-11-26-x.txt"), "").unwrap();
        assert_eq!(next_name(&dir, "2026-11-26").1, 2);
        assert_eq!(of_the_day(&dir, "2026-11-26").len(), 1);
        assert_eq!(std::fs::read_to_string(dir.join("2026-11-26-01.txt")).unwrap().lines().next(), Some("words: first"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_day_and_time_are_greenwich_time() {
        assert_eq!(day_and_time(0), ("1970-01-01".to_string(), "00:00".to_string()));
        // 26 November 2026, 05:41:30 UTC
        assert_eq!(day_and_time(1_795_671_690), ("2026-11-26".to_string(), "05:41".to_string()));
        assert_eq!(day_and_time(951_782_400), ("2000-02-29".to_string(), "00:00".to_string()));
    }

    #[test]
    fn a_paste_is_one_line_and_ordinals_read_as_words() {
        assert_eq!(one_line("one\r\ntwo\nthree\tfour"), "one two three four");
        assert_eq!(one_line("\n"), "");
        assert_eq!((ordinal(1), ordinal(3), ordinal(10), ordinal(11), ordinal(22), ordinal(113)), ("first".into(), "third".into(), "tenth".into(), "11th".into(), "22nd".into(), "113th".into()));
    }
}
