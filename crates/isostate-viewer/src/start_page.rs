//! The start page (her note of 26 November 2026: "it needs a start page, to
//! go alongside what you usually use command like arguments for"). A start
//! with nothing on the command line opens a page of choices: the saved games
//! to go on with, a new island, and a few more things folded away. Every
//! choice on it is an argument. The page turns them into a command line,
//! shows that line at its foot, and runs exactly it: it starts the program
//! again with those arguments and closes, as New game on the opening card
//! does (`saves::new_game_args`). So the page cannot make a game the
//! arguments could not, and the arguments' own path is not touched.
//!
//! Why a small program of its own and not the game's first state: the game
//! makes its island, its ground and its slots in `main` before its App is
//! made, and winit cannot open a second window loop in one process, so a page
//! in the same process would mean moving all of that into systems. Starting
//! the program again costs the half second a window takes to open.
//!
//! When it shows (`wanted`): with no argument at all and no `ISOSTATE_`
//! variable but `ISOSTATE_SAVES` (the measuring tools are run with no
//! arguments and a variable); or when `--start-page` asks for it (the help
//! card's Start page does), or `--page-shot PATH` asks for a picture of it.
//! Any other argument skips it, and the game opens as it always did.
//!
//! Nothing is simulated before it shows: it reads the saved games' first
//! lines (the whole files, a few hundred kilobytes each), the choices it
//! remembers, and the devlog for What's new, on a thread of its own.

// see the note at the top of ui.rs
#![allow(deprecated)]

use std::path::PathBuf;

use bevy::app::ScheduleRunnerPlugin;
use bevy::camera::RenderTarget;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use bevy::window::{ExitCondition, WindowResolution};
use bevy::winit::WinitPlugin;

use crate::saves::{self, Listed, Place};
use crate::ui::{self, BTN, INK, MUTED, RULE};
use crate::UiFont;

// ------------------------------------------------------------------ when it shows

/// How the page was asked for.
#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    /// with a window, for somebody to choose; `from_game` when a game started it
    /// (`--start-page`), so that the game it starts waits for that one to let go
    /// of its saved game
    Window { from_game: bool },
    /// a picture of the page with nobody at it (`--page-shot`)
    Picture(PictureAsk),
}

/// `--page-shot PATH[,more][,family][,begin]`: a picture of the page, with More
/// open, with the family game chosen, and then Begin pressed, the game it
/// starts taking a picture of its opening view beside the page's
/// (`PATH` with `-island` before its extension).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PictureAsk {
    pub path: String,
    pub more: bool,
    pub family: bool,
    pub begin: bool,
    /// a name whose tooltip is opened and locked for the picture (`tip:Land`)
    pub tip: Option<String>,
    /// the start chosen, by the core's word (`start:landfall`)
    pub start: Option<String>,
    /// Landfall's settlers and ideas by sea chosen off (`settlers:off`, `ideas-by-sea:off`)
    pub no_settlers: bool,
    pub no_ideas: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Asked {
    pub mode: Mode,
    /// `--size` beside the page's options: the page's own window or picture
    pub size: Option<(u32, u32)>,
}

/// Whether this start opens the page. `argv` is the command line after the
/// program's name, `set` the names of the environment variables set (to
/// something). An empty line with no variable of the house's but
/// `ISOSTATE_SAVES` opens it, and so do its own two options; anything else is
/// a start the page never meets. Its options beside a game's are refused.
pub fn wanted(argv: &[String], set: &[String]) -> Result<Option<Asked>, String> {
    if argv.is_empty() {
        let tool = set.iter().any(|v| v.starts_with("ISOSTATE_") && v != "ISOSTATE_SAVES");
        return Ok((!tool).then_some(Asked { mode: Mode::Window { from_game: false }, size: None }));
    }
    if !argv.iter().any(|a| a == "--start-page" || a == "--page-shot") {
        return Ok(None);
    }
    let mut asked = Asked { mode: Mode::Window { from_game: true }, size: None };
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--start-page" => {}
            "--page-shot" => {
                let spec = argv.get(i + 1).filter(|s| !s.starts_with("--") && !s.trim().is_empty()).ok_or("--page-shot needs the name of a picture after it")?;
                let mut parts = spec.split(',');
                let mut pic = PictureAsk { path: parts.next().unwrap_or_default().to_string(), ..default() };
                for w in parts {
                    match w.trim() {
                        "more" => pic.more = true,
                        "family" => pic.family = true,
                        "begin" => pic.begin = true,
                        "settlers:off" => pic.no_settlers = true,
                        "ideas-by-sea:off" => pic.no_ideas = true,
                        w if w.starts_with("tip:") => pic.tip = Some(w["tip:".len()..].to_string()),
                        w if w.starts_with("start:") && starts().iter().any(|s| s.word == &w["start:".len()..]) => pic.start = Some(w["start:".len()..].to_string()),
                        w => return Err(format!("--page-shot: {w} is not a word it takes: more, family, begin, settlers:off, ideas-by-sea:off, tip:NAME or start:WORD ({})", isostate_core::landfall::STARTS.join(", "))),
                    }
                }
                asked.mode = Mode::Picture(pic);
                i += 1;
            }
            "--size" => {
                let s = argv.get(i + 1).ok_or("--size needs a value after it")?;
                asked.size = Some(crate::pair("--size", s, 'x')?);
                i += 1;
            }
            a => return Err(format!("--start-page and --page-shot open the start page, and {a} is an option of a game: the page makes its own (give the game's options without --start-page to skip the page)")),
        }
        i += 1;
    }
    Ok(Some(asked))
}

/// `wanted` for this process.
pub fn asked() -> Result<Option<Asked>, String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let set: Vec<String> = std::env::vars_os().filter(|(_, v)| !v.is_empty()).map(|(k, _)| k.to_string_lossy().into_owned()).collect();
    wanted(&argv, &set)
}

// ------------------------------------------------------------------ the choices

/// The way an island begins, as the simulation offers them.
#[derive(Clone, Debug, PartialEq)]
pub struct StartKind {
    /// its word in the core (`settled`)
    pub word: &'static str,
    /// its name on the page
    pub name: &'static str,
    /// a line under it, of twelve words at most
    pub line: &'static str,
    /// the viewer's argument that asks for it; None for the settled island, every game's start
    pub arg: Option<&'static str>,
}

/// The starts there are, as the simulation lists them
/// (`isostate_core::landfall::STARTS`): the settled island, and landfall, one
/// town in a corner (docs/rules/landfall.md, new on 26 November 2026). A start
/// the page has no words or argument for is not offered (a test says so).
pub fn starts() -> Vec<StartKind> {
    isostate_core::landfall::STARTS
        .iter()
        .filter_map(|w| match *w {
            "settled" => Some(StartKind { word: "settled", name: "Settled island", line: "Towns, farms and families already there in 1750.", arg: None }),
            "landfall" => Some(StartKind { word: "landfall", name: "Landfall", line: "One town of 5,000 at a harbour near a corner. New.", arg: Some("--landfall") }),
            _ => None,
        })
        .collect()
}

/// The last years a new game can be played to, on the page.
pub const LAST_YEARS: [i64; 7] = [1900, 1925, 1950, 1975, 2000, 2025, 2050];
/// The years played before a game opens, on the page: it opens in January of
/// 1751 and these after.
pub const ADVANCES: [usize; 8] = [0, 9, 24, 49, 99, 149, 199, 249];
/// The window's sizes on the page.
pub const SIZES: [(u32, u32); 7] = [(1280, 720), (1280, 800), (1600, 900), (1600, 1000), (1920, 1080), (1920, 1200), (2560, 1440)];
/// The seasons that can be held, as `--season` takes them; the first follows the clock.
pub const SEASONS: [(&str, Option<f32>); 5] = [("Clock", None), ("Spring", Some(0.25)), ("Summer", Some(0.5)), ("Autumn", Some(0.75)), ("Winter", Some(0.0))];
/// The island a game is on when nobody says (`--seed`), and its families.
pub const SEED: u64 = 7;
pub const FAMILIES: usize = 6;
pub const SIZE: (u32, u32) = (1600, 1000);
/// The largest island number the page steps to; any number is an island.
pub const MOST_SEED: u64 = 9999;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Sandbox,
    Family,
}

/// Every choice on the page, as the command line has it.
#[derive(Clone, Debug, PartialEq)]
pub struct Choices {
    pub kind: Kind,
    /// which of `starts()`
    pub start: usize,
    pub seed: u64,
    pub families: usize,
    pub to: i64,
    pub advance: usize,
    pub old_land: bool,
    /// Landfall's settlers by ship and ideas by sea (docs/rules/landfall.md,
    /// "The start's switches"): on as the start has them, offered under
    /// Landfall alone, and said on the command line only when off
    pub settlers: bool,
    pub ideas: bool,
    /// which of `SEASONS`
    pub season: usize,
    pub size: (u32, u32),
    /// a map view's word (`overlay::Overlay::word`), or None for the plain map
    pub view: Option<String>,
}

impl Default for Choices {
    fn default() -> Self {
        Choices { kind: Kind::Sandbox, start: 0, seed: SEED, families: FAMILIES, to: isostate_core::save::LATEST_YEAR, advance: 0, old_land: false, settlers: true, ideas: true, season: 0, size: SIZE, view: None }
    }
}

/// One choice, as a row of the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Field {
    Kind,
    Start,
    Abroad,
    Seed,
    Families,
    To,
    Advance,
    Land,
    Season,
    Size,
    View,
}

impl Field {
    /// The new island's rows, and More's. Abroad shows under Landfall alone (`shown`).
    pub const NEW: [Field; 7] = [Field::Kind, Field::Start, Field::Abroad, Field::Seed, Field::Families, Field::To, Field::Advance];
    pub const MORE: [Field; 4] = [Field::Land, Field::Season, Field::Size, Field::View];

    pub fn label(self) -> &'static str {
        match self {
            Field::Kind => "Play",
            Field::Start => "Start",
            Field::Abroad => "Abroad",
            Field::Seed => "Island",
            Field::Families => "Families",
            Field::To => "Last year",
            Field::Advance => "Opens in",
            Field::Land => "Land",
            Field::Season => "Season",
            Field::Size => "Window",
            Field::View => "Map view",
        }
    }

    /// What its name's tooltip says: what the choice is, then what it decides
    /// (docs/designs/words.md: three sentences at most, 18 words each).
    pub fn says(self) -> &'static str {
        match self {
            Field::Kind => "A sandbox is the island to watch and to change with tools. The family game is one family among the great families. Both are saved as you play.",
            Field::Start => "How the island begins in 1750. The settled island has its towns, farms and great families from the first year. Landfall is one town of 5,000 at a harbour, with settlers coming by ship.",
            Field::Abroad => "Settlers come by ship to the ports while income per head is above abroad. Ports learn technology from abroad, as from a town across the sea. Without either, the colony is alone.",
            Field::Seed => "The number that makes the island, its land and its first towns. The same number makes the same island every time.",
            Field::Families => "The great families hold land and firms across the island. In the family game you play one of them.",
            Field::To => "The game ends after this year. A game saved goes on to the last year it began with.",
            Field::Advance => "The years before it are played before the window opens. A family game that opens late is saved apart, as a trial.",
            Field::Land => "The new land has rivers that run downhill, lakes and relief every 500 m. The old island is the land of every game before 19 November 2026.",
            Field::Season => "Holds one season all year, for its look. Clock lets the seasons turn with the months.",
            Field::Size => "The size of the game's window, in pixels. The window can be made larger once the game is open.",
            Field::View => "The map view the game opens on. M steps through the views in the game.",
        }
    }

    /// Whether its options are laid side by side (else it steps, with - and +).
    pub fn laid_out(self) -> bool {
        matches!(self, Field::Kind | Field::Start | Field::Abroad | Field::Land | Field::Season)
    }

    /// Whether its row is on the page with these choices: Landfall's two
    /// switches only when Landfall is chosen (on the settled island both are
    /// off, and the page offers nothing to change there).
    pub fn shown(self, c: &Choices) -> bool {
        self != Field::Abroad || c.landfall()
    }
}

/// The map views, as their chips name them, and their words (`overlay.rs`).
fn views() -> Vec<(Option<String>, String)> {
    std::iter::once((None, "Plain".to_string())).chain(crate::overlay::Overlay::ALL.iter().map(|o| (Some(o.word().to_string()), o.chip().to_string()))).collect()
}

/// Where `x` is in `list`, or the nearest below it.
fn at_or_below<T: PartialOrd>(list: &[T], x: &T) -> usize {
    list.iter().rposition(|v| v <= x).unwrap_or(0)
}

impl Choices {
    /// Whether the start chosen is landfall.
    pub fn landfall(&self) -> bool {
        starts().get(self.start).is_some_and(|s| s.word == "landfall")
    }

    /// The opening years the page offers under the last year chosen: a game
    /// must have a year left to play when it opens.
    pub fn advances(&self) -> Vec<usize> {
        ADVANCES.iter().copied().filter(|&a| 1751 + a as i64 <= self.to).collect()
    }

    /// Which of a field's chips are lit: the one chosen, or for chips that
    /// are each on or off (`Field::toggles`), every one that is on.
    pub fn lit(&self, f: Field) -> Vec<bool> {
        match f {
            Field::Abroad => vec![self.settlers, self.ideas],
            _ => {
                let (names, at) = self.options(f);
                (0..names.len()).map(|i| i == at).collect()
            }
        }
    }

    /// The names of a laid-out field's options, and which is chosen.
    pub fn options(&self, f: Field) -> (Vec<String>, usize) {
        match f {
            Field::Kind => (vec!["Sandbox".into(), "Family game".into()], (self.kind == Kind::Family) as usize),
            Field::Start => (starts().iter().map(|s| s.name.to_string()).collect(), self.start),
            // (each chip on or off: `lit`)
            Field::Abroad => (vec!["Settlers".into(), "Technology".into()], 0),
            Field::Land => (vec!["New land".into(), "Old island".into()], self.old_land as usize),
            Field::Season => (SEASONS.iter().map(|s| s.0.to_string()).collect(), self.season),
            _ => (vec![self.value(f)], 0),
        }
    }

    /// What a stepped field shows.
    pub fn value(&self, f: Field) -> String {
        match f {
            Field::Seed => self.seed.to_string(),
            Field::Families => self.families.to_string(),
            Field::To => self.to.to_string(),
            Field::Advance => (1751 + self.advance).to_string(),
            Field::Size => format!("{} \u{00d7} {}", self.size.0, self.size.1),
            Field::View => views().into_iter().find(|(w, _)| *w == self.view).map_or("Plain".into(), |(_, name)| name),
            _ => self.options(f).0.get(self.options(f).1).cloned().unwrap_or_default(),
        }
    }

    /// Choose a laid-out field's option `i`.
    pub fn set(&mut self, f: Field, i: usize) {
        match f {
            Field::Kind => self.kind = if i == 1 { Kind::Family } else { Kind::Sandbox },
            Field::Start => self.start = i.min(starts().len().saturating_sub(1)),
            // a chip turned the other way
            Field::Abroad if i == 0 => self.settlers = !self.settlers,
            Field::Abroad => self.ideas = !self.ideas,
            Field::Land => self.old_land = i == 1,
            Field::Season => self.season = i.min(SEASONS.len() - 1),
            _ => {}
        }
    }

    /// One step along a field (left, right, - or +); a laid-out field goes round.
    pub fn step(&mut self, f: Field, by: i32) {
        let go = |i: usize, n: usize| (i as i64 + by as i64).clamp(0, n as i64 - 1) as usize;
        match f {
            Field::Seed => self.seed = (self.seed as i64 + by as i64).clamp(1, MOST_SEED as i64) as u64,
            Field::Families => self.families = (self.families as i64 + by as i64).clamp(1, isostate_core::save::MAX_FAMILIES as i64) as usize,
            Field::To => {
                self.to = LAST_YEARS[go(at_or_below(&LAST_YEARS, &self.to), LAST_YEARS.len())];
                // a game that would open after its last year opens as late as it can
                let can = self.advances();
                if !can.contains(&self.advance) {
                    self.advance = can.last().copied().unwrap_or(0);
                }
            }
            Field::Advance => {
                let can = self.advances();
                self.advance = can[go(at_or_below(&can, &self.advance), can.len())];
            }
            Field::Size => self.size = SIZES[go(SIZES.iter().position(|s| *s == self.size).unwrap_or(3), SIZES.len())],
            Field::View => {
                let all = views();
                let at = all.iter().position(|(w, _)| *w == self.view).unwrap_or(0);
                self.view = all[go(at, all.len())].0.clone();
            }
            // round the four: both on, no settlers, no technology, neither
            Field::Abroad => {
                const ROUND: [(bool, bool); 4] = [(true, true), (false, true), (true, false), (false, false)];
                let at = ROUND.iter().position(|&x| x == (self.settlers, self.ideas)).unwrap_or(0) as i64;
                (self.settlers, self.ideas) = ROUND[(at + by as i64).rem_euclid(4) as usize];
            }
            _ => {
                let (names, at) = self.options(f);
                let n = names.len().max(1);
                self.set(f, ((at as i64 + by as i64).rem_euclid(n as i64)) as usize);
            }
        }
    }

    /// The line under a field: a term and a number where it can be, a short sentence where not.
    pub fn line(&self, f: Field) -> String {
        match f {
            Field::Kind => match self.kind {
                Kind::Sandbox => "Watch the island, and change it with tools.".into(),
                Kind::Family => format!("Play one family among {}.", self.families),
            },
            Field::Start => starts().get(self.start).map_or(String::new(), |s| s.line.to_string()),
            Field::Abroad => match (self.settlers, self.ideas) {
                (true, true) => "Ships bring settlers and technology.".into(),
                (false, true) => "Ships bring technology. No settlers.".into(),
                (true, false) => "Ships bring settlers. No technology from abroad.".into(),
                (false, false) => "No ships. The colony is alone.".into(),
            },
            Field::Seed => "Each number is another island.".into(),
            Field::Families => "Great families, holding land and firms.".into(),
            Field::To => format!("{} years from 1750.", self.to - 1750 + 1),
            Field::Advance => {
                if self.advance == 0 {
                    "1750 played. Nothing to wait for.".into()
                } else {
                    let trial = if self.kind == Kind::Family { " Saved apart, as a trial." } else { "" };
                    format!("{} years played first: {}.{trial}", self.advance + 1, play_words(self.advance + 1))
                }
            }
            Field::Land => if self.old_land { "The island before 19 November 2026.".into() } else { "Every new game's land.".into() },
            Field::Season => if self.season == 0 { "The seasons follow the clock.".into() } else { "Held all year.".into() },
            Field::Size => "The game's window, in pixels.".into(),
            Field::View => if self.view.is_none() { "Opens on the plain map.".into() } else { "Opens on this map view. M steps through them.".into() },
        }
    }

    /// The arguments of the window: what More holds that a game gone on with takes too.
    pub fn window_args(&self) -> Vec<String> {
        let mut v = Vec::new();
        if let Some(s) = SEASONS[self.season.min(SEASONS.len() - 1)].1 {
            v.extend(["--season".to_string(), s.to_string()]);
        }
        if self.size != SIZE {
            v.extend(["--size".to_string(), format!("{}x{}", self.size.0, self.size.1)]);
        }
        if let Some(w) = &self.view {
            v.extend(["--overlay".to_string(), w.clone()]);
        }
        v
    }

    /// The arguments of a new game with these choices: what the page runs, and
    /// shows at its foot. Only what differs from a game's defaults is said.
    pub fn args(&self) -> Vec<String> {
        let mut v = vec![if self.kind == Kind::Family { "--family" } else { "--sandbox" }.to_string(), "--new".to_string()];
        if let Some(a) = starts().get(self.start).and_then(|s| s.arg) {
            v.push(a.to_string());
        }
        // (landfall's own switches, said only when off: on the settled island both are off already)
        if self.landfall() && !self.settlers {
            v.extend(["--settlers".to_string(), "off".to_string()]);
        }
        if self.landfall() && !self.ideas {
            v.extend(["--ideas-by-sea".to_string(), "off".to_string()]);
        }
        if self.seed != SEED {
            v.extend(["--seed".to_string(), self.seed.to_string()]);
        }
        if self.families != FAMILIES {
            v.extend(["--families".to_string(), self.families.to_string()]);
        }
        if self.to != isostate_core::save::LATEST_YEAR {
            v.extend(["--to".to_string(), self.to.to_string()]);
        }
        if self.advance > 0 {
            v.extend(["--advance".to_string(), self.advance.to_string()]);
        }
        if self.old_land {
            v.push("--old-land".to_string());
        }
        v.extend(self.window_args());
        v
    }

    /// As the page keeps them between starts, a line a choice.
    pub fn to_text(&self) -> String {
        let start = starts().get(self.start).map_or("settled", |s| s.word);
        format!(
            "The start page's last choices. Delete this file to forget them.\nkind: {}\nstart: {start}\nsettlers: {}\nideas_by_sea: {}\nseed: {}\nfamilies: {}\nto: {}\nadvance: {}\nold_land: {}\nseason: {}\nsize: {}x{}\nview: {}\n",
            if self.kind == Kind::Family { "family" } else { "sandbox" },
            if self.settlers { "on" } else { "off" },
            if self.ideas { "on" } else { "off" },
            self.seed,
            self.families,
            self.to,
            self.advance,
            self.old_land,
            SEASONS[self.season.min(SEASONS.len() - 1)].0,
            self.size.0,
            self.size.1,
            self.view.as_deref().unwrap_or("plain")
        )
    }

    /// What the page kept, read back: a line that cannot be read, or a value no
    /// longer offered, leaves that choice as it is by default.
    pub fn from_text(text: &str) -> Choices {
        let mut c = Choices::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once(": ") else { continue };
            let value = value.trim();
            match key {
                "kind" => c.kind = if value == "family" { Kind::Family } else { Kind::Sandbox },
                "start" => c.start = starts().iter().position(|s| s.word == value).unwrap_or(0),
                "seed" => c.seed = value.parse().ok().filter(|s| (1..=MOST_SEED).contains(s)).unwrap_or(SEED),
                "families" => c.families = value.parse().ok().filter(|&n| isostate_core::save::families_allowed(n).is_ok()).unwrap_or(FAMILIES),
                "to" => c.to = value.parse().ok().filter(|y| LAST_YEARS.contains(y)).unwrap_or(c.to),
                "advance" => c.advance = value.parse().ok().filter(|a| ADVANCES.contains(a)).unwrap_or(0),
                "old_land" => c.old_land = value == "true",
                "settlers" => c.settlers = value != "off",
                "ideas_by_sea" => c.ideas = value != "off",
                "season" => c.season = SEASONS.iter().position(|s| s.0 == value).unwrap_or(0),
                "size" => c.size = value.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?))).filter(|s| SIZES.contains(s)).unwrap_or(SIZE),
                "view" => c.view = views().into_iter().find(|(w, _)| w.as_deref() == Some(value)).and_then(|(w, _)| w),
                _ => {}
            }
        }
        if !c.advances().contains(&c.advance) {
            c.advance = 0;
        }
        c
    }
}

/// The file the page keeps its last choices in, beside the saved games.
pub fn kept_at() -> PathBuf {
    saves::folder().join("start-page.txt")
}

/// How long playing `years` from 1750 takes before the window opens, in
/// words: measured with this build's simulation on island 7 (see the
/// viewer's README, "The start page").
pub fn play_words(years: usize) -> String {
    let secs = play_secs(years);
    if secs < 90.0 {
        format!("about {} s", (secs / 5.0).round().max(1.0) * 5.0)
    } else {
        format!("about {} min", (secs / 60.0).round())
    }
}

/// Seconds to play `years` from 1750 before a game opens. Fitted to
/// `--advance` on island 7 (crates/isostate-viewer/README.md, "The start
/// page"): the years to 1900 are quick, those after slower.
pub fn play_secs(years: usize) -> f32 {
    let early = years.min(150) as f32;
    let late = years.saturating_sub(150) as f32;
    early * EARLY_YEAR + late * LATE_YEAR
}

/// Seconds a year takes to play before the window opens, before 1900 and
/// after: `--advance 149` took 56.3 s and `--advance 249` 147.3 s on island 7
/// (the dev build, 26 November 2026, a machine busy with other builds).
const EARLY_YEAR: f32 = 0.37;
const LATE_YEAR: f32 = 0.9;

// ------------------------------------------------------------------ the page's state

/// What can hold the keyboard's attention, in the order Tab goes through it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Save(usize),
    Field(Field),
    Draw,
    Begin,
    More,
    Copy,
    WhatsNew,
    Backlog,
    Notes,
    Quit,
}

#[derive(Resource)]
pub struct Page {
    pub choices: Choices,
    pub saves: Vec<Listed>,
    pub focus: Focus,
    pub more: bool,
    /// a line from the page itself: what it could not do
    pub said: Option<String>,
    pub mode: Mode,
    /// the arguments to start the program with, once chosen
    go: Option<Vec<String>>,
    /// when the program started, for saying how long the page took
    t0: std::time::Instant,
}

impl Page {
    /// Everything the keyboard reaches, in order.
    pub fn order(&self) -> Vec<Focus> {
        let mut v: Vec<Focus> = (0..self.saves.len()).map(Focus::Save).collect();
        v.extend(Field::NEW.iter().filter(|f| f.shown(&self.choices)).map(|&f| Focus::Field(f)));
        v.extend([Focus::Draw, Focus::Begin, Focus::More]);
        if self.more {
            v.extend(Field::MORE.iter().map(|&f| Focus::Field(f)));
        }
        v.extend([Focus::Copy, Focus::WhatsNew, Focus::Backlog, Focus::Notes, Focus::Quit]);
        v
    }

    fn move_focus(&mut self, by: i32) {
        let order = self.order();
        let at = order.iter().position(|f| *f == self.focus).unwrap_or(0) as i64;
        self.focus = order[(at + by as i64).rem_euclid(order.len() as i64) as usize];
    }

    /// The arguments of what has the focus: a saved game's, or the new island's.
    pub fn args(&self) -> Vec<String> {
        match self.focus {
            Focus::Save(i) => self.saves.get(i).map(|s| [s.args(), self.choices.window_args()].concat()).unwrap_or_default(),
            _ => self.choices.args(),
        }
    }
}

/// The command line that does what `args` does, ready to copy: the program's
/// own name, and each argument quoted where a shell would split it.
pub fn command_line(program: &str, args: &[String]) -> String {
    let quoted = |a: &String| if a.is_empty() || a.contains([' ', '\t', '"', '&', '(', ')']) { format!("\"{a}\"") } else { a.clone() };
    std::iter::once(program.to_string()).chain(args.iter().map(quoted)).collect::<Vec<_>>().join(" ")
}

/// The program's name as a player would type it.
fn program() -> String {
    std::env::current_exe().ok().and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned())).unwrap_or_else(|| "isolated-state".into())
}

/// "2 h ago", "3 days ago", or a date: when a saved game was last written.
pub fn ago(when: u64, now: u64) -> String {
    let secs = now.saturating_sub(when);
    match secs {
        0..60 => "just now".into(),
        60..3_600 => format!("{} min ago", secs / 60),
        3_600..86_400 => format!("{} h ago", secs / 3_600),
        86_400..172_800 => "yesterday".into(),
        172_800..1_209_600 => format!("{} days ago", secs / 86_400),
        _ => crate::note::day_and_time(when).0,
    }
}

/// A saved game's date as the clock would show it: "March 1784", or the last
/// year alone when it is over.
fn save_date(s: &Listed) -> String {
    if s.year >= s.to {
        return s.year.to_string();
    }
    crate::clock::month_year(crate::clock::Date { year: (s.year + 1) as i32, month: s.month.min(11), part: 0.0 })
}

// ------------------------------------------------------------------ the program

/// The page's colours: plain ground behind the cards, and the hand's ink for
/// what is chosen.
const GROUND: Color = Color::srgb(0.66, 0.71, 0.56);
const CHOSEN: Color = crate::sandbox::HAND_INK;
const FOCUS: Color = Color::srgb(0.76, 0.40, 0.17);

#[derive(Component)]
struct Root;
/// Something the keyboard can reach; the pointer over it gives it the focus.
#[derive(Component)]
struct Focusable(Focus);
/// What a press does.
#[derive(Component, Clone, Copy)]
enum Press {
    Step(Field, i32),
    Pick(Field, usize),
    GoOn(usize),
    Draw,
    Begin,
    More,
    Copy,
    Notes,
    Quit,
}
/// Where a field's options or value are laid, made again when they change.
#[derive(Component)]
struct FieldBox(Field);
#[derive(Component)]
struct FieldLine(Field);
/// A field's whole row, hidden while the field is not `shown`.
#[derive(Component)]
struct FieldRow(Field);
#[derive(Component)]
struct MoreCard;
#[derive(Component)]
struct CommandText;
#[derive(Component)]
struct SaidText;
#[derive(Component)]
struct NewCount;
/// The word on More's button: More, or Less while it is open.
#[derive(Component)]
struct MoreWord;
/// The parts of the page whose heights decide how tall the list of saved games may be.
#[derive(Component, Clone, Copy, PartialEq)]
enum Fits {
    Title,
    GoOn,
    Foot,
}

/// The saved games' card is as tall as the window leaves it, under More when
/// More is open, so that the foot stays on the screen and the list scrolls.
/// (A flex layout would do it, if a height shrunk by the flex could be shared
/// out again: in Bevy's it cannot, so the room is worked out here.)
fn fit(page: Res<Page>, cams: Query<&Camera>, parts: Query<(&Fits, &ComputedNode)>, more: Query<&ComputedNode, With<MoreCard>>, mut go_on: Query<(&Fits, &mut Node)>) {
    let Some(view) = cams.iter().find_map(|c| c.logical_viewport_size()) else { return };
    let height = |which: Fits| parts.iter().find(|(f, _)| **f == which).map_or(0.0, |(_, n)| n.size().y * n.inverse_scale_factor);
    // the root's padding (20 above and below) and its three gaps of 14 between title, cards, room and foot
    let mut room = view.y - 40.0 - 42.0 - height(Fits::Title) - height(Fits::Foot);
    if page.more {
        room -= more.iter().map(|n| n.size().y * n.inverse_scale_factor).fold(0.0, f32::max) + 14.0;
    }
    let want = Val::Px(room.max(120.0));
    for (f, mut n) in &mut go_on {
        if *f == Fits::GoOn && n.max_height != want {
            n.max_height = want;
        }
    }
}

/// What the tooltips of Command and Go on say.
pub const COMMAND_SAYS: &str = "This command line starts the same game as the page. Copy puts it on the clipboard. Any option on a command line skips the page.";
pub const GO_ON_SAYS: &str = "The games saved in this machine's folder of saved games. A game saves itself as you play. A click plays its years again and goes on.";

/// A tooltip behind a name on the page, from its own words, with links to the
/// glossary's terms where its words name them.
/// A name that is itself a term of the glossary (Abroad) does not link to itself.
fn tip(name: &str, says: &str) -> crate::tips::Tip {
    let g = crate::words::glossary();
    crate::tips::Tip::new(name, crate::tips::TipBody { term: name.to_string(), says: g.pieces(says, g.key_of(name)), ..default() })
}

/// Run the page, and return the program's exit code.
pub fn run(asked: Asked, t0: std::time::Instant) -> i32 {
    let picture = match &asked.mode {
        Mode::Picture(p) => Some(p.clone()),
        Mode::Window { .. } => None,
    };
    // what it remembers: not in a picture, which shows the page as it first opens
    let mut choices = match &picture {
        Some(_) => Choices::default(),
        None => std::fs::read_to_string(kept_at()).map(|t| Choices::from_text(&t)).unwrap_or_default(),
    };
    if picture.as_ref().is_some_and(|p| p.family) {
        choices.kind = Kind::Family;
    }
    if let Some(i) = picture.as_ref().and_then(|p| p.start.as_ref()).and_then(|w| starts().iter().position(|s| &s.word == w)) {
        choices.start = i;
    }
    if let Some(p) = &picture {
        choices.settlers &= !p.no_settlers;
        choices.ideas &= !p.no_ideas;
    }
    let saves = saves::listed_in(&saves::folder());
    let size = asked.size.unwrap_or(choices.size);
    // Enter goes on with the newest saved game, as a plain start did before the page; a
    // picture of a choice made shows the new island's command line
    let chose = picture.as_ref().is_some_and(|p| p.more || p.family || p.start.is_some() || p.no_settlers || p.no_ideas);
    let focus = if saves.is_empty() || chose { Focus::Begin } else { Focus::Save(0) };
    eprintln!("the start page: {} saved game{} in {}", saves.len(), if saves.len() == 1 { "" } else { "s" }, saves::folder().display());
    let page = Page { choices, saves, focus, more: picture.as_ref().is_some_and(|p| p.more), said: None, mode: asked.mode.clone(), go: None, t0 };

    let mut app = App::new();
    if picture.is_some() {
        app.add_plugins(DefaultPlugins.set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, ..default() }).disable::<WinitPlugin>())
            .add_plugins(ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 30.0)));
        let waiting = crate::PipelinesWaiting::default();
        app.insert_resource(waiting.clone());
        if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render.insert_resource(waiting).add_systems(bevy::render::Render, crate::count_waiting_pipelines.in_set(bevy::render::RenderSystems::Cleanup));
        }
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: format!("Isolated State {}", crate::VERSION), resolution: WindowResolution::new(size.0, size.1), ..default() }),
            ..default()
        }));
    }
    app.insert_resource(ClearColor(GROUND))
        .insert_resource(page)
        .insert_resource(PageSize(size))
        // What's new and the backlog are the game's own cards, opened from the page's foot
        .insert_resource(crate::note::About { island: "the start page".into(), kept_in: "nothing is kept from the page".into(), folder: Ok(saves::notes()) })
        .add_systems(Startup, setup)
        .add_plugins(crate::whats_new::plugin)
        .add_plugins(crate::backlog_seen::plugin)
        // the tooltips that lock and nest, behind each choice's name (tips.rs); Esc closes them first
        .add_plugins(crate::tips::plugin)
        // (after the pointer's own pass over the buttons, so that a button Enter presses stays pressed for the cards to see)
        .add_systems(PreUpdate, keys.after(bevy::input::InputSystems).after(bevy::ui::UiSystems::Focus).after(crate::tips::escape))
        .add_systems(Update, (ui::button_colours, ui::row_colours, ui::wheel_lists, ui::scroll_thumbs, pointer, presses, draw, fit, news_count, up_after, go).chain());
    if let Some(p) = picture {
        app.insert_resource(Shooting { ask: p, frames: 0, image: None, asked_at: None, held: None });
        app.add_systems(Update, shoot.after(go));
    }
    // (a picture that was not written, or a game that did not take its own, ends the program with 1)
    match app.run() {
        AppExit::Success => 0,
        AppExit::Error(code) => code.get() as i32,
    }
}

#[derive(Resource)]
struct PageSize((u32, u32));

fn setup(mut commands: Commands, mut fonts: ResMut<Assets<Font>>, mut images: ResMut<Assets<Image>>, page: Res<Page>, size: Res<PageSize>, shooting: Option<ResMut<Shooting>>) {
    let font = fonts.add(Font::from_bytes(crate::FREDOKA.to_vec()));
    commands.insert_resource(UiFont(font.clone()));
    // (the tooltips keep themselves inside the main camera's view)
    let mut cam = commands.spawn((Camera2d, IsDefaultUiCamera, crate::MainCam));
    if let Some(mut shooting) = shooting {
        let mut img = Image::new_target_texture(size.0 .0, size.0 .1, TextureFormat::Rgba8UnormSrgb, None);
        img.texture_descriptor.usage |= TextureUsages::COPY_SRC;
        let handle = images.add(img);
        cam.insert(RenderTarget::Image(handle.clone().into()));
        shooting.image = Some(handle);
    }
    spawn_page(&mut commands, &font, &page);
}

/// The page, laid once; its fields' values are laid again as they change (`draw`).
fn spawn_page(commands: &mut Commands, f: &Handle<Font>, page: &Page) {
    let now = saves::now();
    commands
        .spawn((
            Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::axes(Val::Px(24.0), Val::Px(20.0)), row_gap: Val::Px(14.0), ..default() },
            Root,
        ))
        .with_children(|root| {
            // the name of the game
            root.spawn((Node { align_items: AlignItems::Baseline, column_gap: Val::Px(12.0), ..default() }, Fits::Title)).with_children(|t| {
                t.spawn(ui::text(f, "Isolated State", 38.0, INK));
                t.spawn(ui::text(f, &format!("version {}", crate::VERSION), 15.0, INK.with_alpha(0.7)));
            });
            // (each card as tall as what it holds; the saved games scroll when there are more than fit)
            root.spawn(Node { column_gap: Val::Px(18.0), flex_shrink: 1.0, min_height: Val::Px(0.0), align_items: AlignItems::FlexStart, ..default() }).with_children(|row| {
                // the saved games, and under them More, folded away: open, it takes the room the
                // list gives up, and the list scrolls, so that the foot stays in a window 720 high
                row.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(14.0), max_height: Val::Percent(100.0), min_height: Val::Px(0.0), ..default() }).with_children(|col| {
                    go_on_card(col, f, page, now);
                    col.spawn((
                        ui::card(Node { display: Display::None, flex_direction: FlexDirection::Column, width: Val::Px(520.0), padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)), row_gap: Val::Px(0.0), border_radius: BorderRadius::all(Val::Px(16.0)), flex_shrink: 0.0, ..default() }),
                        MoreCard,
                    ))
                    .with_children(|c| {
                        for f2 in Field::MORE {
                            field_row(c, f, f2);
                        }
                    });
                });
                new_card(row, f);
            });
            // the foot stands at the bottom of the window
            root.spawn(Node { flex_grow: 1.0, ..default() });
            foot(root, f);
        });
}

fn heading(p: &mut ChildSpawnerCommands, f: &Handle<Font>, title: &str, aside: &str, says: Option<&str>) {
    p.spawn(Node { align_items: AlignItems::Baseline, column_gap: Val::Px(10.0), margin: UiRect::bottom(Val::Px(4.0)), ..default() }).with_children(|h| {
        let mut t = h.spawn(ui::text(f, title, 22.0, INK));
        if let Some(says) = says {
            t.insert(tip(title, says));
        }
        h.spawn(ui::text(f, aside, 14.0, MUTED));
    });
}

fn go_on_card(row: &mut ChildSpawnerCommands, f: &Handle<Font>, page: &Page, now: u64) {
    row.spawn((
        ui::card(Node { flex_direction: FlexDirection::Column, width: Val::Px(520.0), padding: UiRect::axes(Val::Px(22.0), Val::Px(18.0)), row_gap: Val::Px(4.0), border_radius: BorderRadius::all(Val::Px(18.0)), min_height: Val::Px(0.0), ..default() }),
        Fits::GoOn,
        RelativeCursorPosition::default(),
    ))
    .with_children(|c| {
        let card = c.target_entity();
        let n = page.saves.len();
        heading(c, f, "Go on", &match n { 0 => "no saved games".to_string(), 1 => "1 saved game".to_string(), n => format!("{n} saved games, newest first") }, Some(GO_ON_SAYS));
        if n == 0 {
            c.spawn(ui::text(f, "A game saves itself as you play.", 15.0, MUTED));
            c.spawn(ui::text(f, "Begin a new island, and it will be here.", 15.0, MUTED));
            return;
        }
        ui::scroll_list(c, card, Node { flex_grow: 1.0, flex_shrink: 1.0, ..default() }, Node { flex_grow: 1.0, row_gap: Val::Px(3.0), padding: UiRect::right(Val::Px(ui::THUMB_ROOM)), ..default() }, (), |list| {
            for (i, s) in page.saves.iter().enumerate() {
                save_row(list, f, i, s, now);
            }
        });
    });
}

fn save_row(p: &mut ChildSpawnerCommands, f: &Handle<Font>, i: usize, s: &Listed, now: u64) {
    let what = match s.place {
        Place::Sandbox => "Sandbox".to_string(),
        Place::Family => format!("The {} family", s.family),
        Place::Trial => format!("The {} family, a trial", s.family),
    };
    let mut details = vec![save_date(s), format!("{} families", s.families), format!("to {}", s.to)];
    if s.landfall {
        details.push("landfall".into());
    }
    details.extend(saves::switch_words(s.switches).into_iter().map(String::from));
    if s.old_land {
        details.push("old island".into());
    }
    p.spawn((
        Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::axes(Val::Px(10.0), Val::Px(7.0)), border: UiRect::all(Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(10.0)), flex_shrink: 0.0, ..default() },
        BackgroundColor(Color::NONE),
        BorderColor::all(Color::NONE),
        Interaction::default(),
        FocusPolicy::Block,
        ui::RowHover { open: false },
        Focusable(Focus::Save(i)),
        Press::GoOn(i),
    ))
    .with_children(|row| {
        row.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(1.0), ..default() }).with_children(|l| {
            l.spawn(Node { column_gap: Val::Px(8.0), align_items: AlignItems::Baseline, ..default() }).with_children(|t| {
                t.spawn(ui::text(f, &format!("Island {}", s.seed), 17.0, INK));
                t.spawn(ui::text(f, &what, 15.0, if s.place == Place::Sandbox { CHOSEN } else { INK }));
            });
            l.spawn(ui::text(f, &details.join(" \u{00b7} "), 13.5, MUTED));
        });
        row.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::End, row_gap: Val::Px(2.0), ..default() }).with_children(|r| {
            r.spawn(ui::text(f, &ago(s.when, now), 13.5, MUTED));
            r.spawn((Node { padding: UiRect::axes(Val::Px(10.0), Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(9.0)), ..default() }, BackgroundColor(CHOSEN))).with_children(|b| {
                b.spawn(ui::text(f, "Go on", 13.5, Color::WHITE));
            });
        });
    });
}

fn new_card(row: &mut ChildSpawnerCommands, f: &Handle<Font>) {
    row.spawn(ui::card(Node { flex_direction: FlexDirection::Column, width: Val::Px(540.0), padding: UiRect::axes(Val::Px(22.0), Val::Px(18.0)), row_gap: Val::Px(2.0), border_radius: BorderRadius::all(Val::Px(18.0)), flex_shrink: 0.0, ..default() })).with_children(|c| {
        heading(c, f, "A new island", "1750 on", None);
        for field in Field::NEW {
            field_row(c, f, field);
        }
        c.spawn(Node { margin: UiRect::top(Val::Px(10.0)), column_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }).with_children(|b| {
            b.spawn((
                Node { padding: UiRect::axes(Val::Px(26.0), Val::Px(9.0)), border: UiRect::all(Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(13.0)), ..default() },
                BackgroundColor(CHOSEN),
                BorderColor::all(Color::NONE),
                Interaction::default(),
                FocusPolicy::Block,
                Focusable(Focus::Begin),
                Press::Begin,
            ))
            .with_children(|t| {
                t.spawn(ui::text(f, "Begin", 18.0, Color::WHITE));
            });
            b.spawn(ui::text(f, "Enter starts what is lit.", 13.5, MUTED));
            b.spawn(Node { flex_grow: 1.0, ..default() });
            // More, folded away under the cards: the land, a season held, the window, the map view
            b.spawn((ui_button(), BackgroundColor(BTN), BorderColor::all(Color::NONE), Interaction::default(), FocusPolicy::Block, ui::Hoverable, Focusable(Focus::More), Press::More)).with_children(|m| {
                m.spawn((ui::text(f, "More", 14.0, INK), MoreWord));
            });
        });
    });
}

/// A field: its label, its options or its value with - and +, and its line.
fn field_row(p: &mut ChildSpawnerCommands, f: &Handle<Font>, field: Field) {
    p.spawn((
        Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)), border: UiRect::all(Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(10.0)), row_gap: Val::Px(1.0), flex_shrink: 0.0, ..default() },
        BorderColor::all(Color::NONE),
        Interaction::default(),
        Focusable(Focus::Field(field)),
        FieldRow(field),
    ))
    .with_children(|r| {
        r.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(10.0), ..default() }).with_children(|line| {
            line.spawn((Node { width: Val::Px(84.0), flex_shrink: 0.0, ..default() },)).with_children(|l| {
                l.spawn((ui::text(f, field.label(), 14.5, MUTED), tip(field.label(), field.says())));
            });
            line.spawn((Node { align_items: AlignItems::Center, column_gap: Val::Px(5.0), ..default() }, FieldBox(field)));
        });
        r.spawn((ui::text(f, "", 13.0, MUTED), Node { margin: UiRect::left(Val::Px(94.0)), ..default() }, FieldLine(field)));
    });
}

/// A field's options as chips, or its value between - and +.
fn lay_field(p: &mut ChildSpawnerCommands, f: &Handle<Font>, field: Field, c: &Choices) {
    let chip = |p: &mut ChildSpawnerCommands, label: &str, on: bool, press: Press| {
        p.spawn((
            Node { padding: UiRect::axes(Val::Px(11.0), Val::Px(4.0)), border_radius: BorderRadius::all(Val::Px(9.0)), ..default() },
            BackgroundColor(if on { CHOSEN } else { BTN }),
            Interaction::default(),
            FocusPolicy::Block,
            press,
        ))
        .with_children(|b| {
            b.spawn(ui::text(f, label, 14.5, if on { Color::WHITE } else { INK }));
        });
    };
    if field.laid_out() {
        let (names, lit) = (c.options(field).0, c.lit(field));
        for (i, name) in names.iter().enumerate() {
            chip(p, name, lit.get(i).copied().unwrap_or(false), Press::Pick(field, i));
        }
        if field == Field::Start && names.len() == 1 {
            p.spawn(ui::text(f, "more starts to come", 13.0, MUTED));
        }
    } else {
        let step = |p: &mut ChildSpawnerCommands, label: &str, by: i32| {
            p.spawn((Node { width: Val::Px(28.0), justify_content: JustifyContent::Center, padding: UiRect::vertical(Val::Px(3.0)), border_radius: BorderRadius::all(Val::Px(9.0)), ..default() }, BackgroundColor(BTN), Interaction::default(), FocusPolicy::Block, ui::Hoverable, Press::Step(field, by)))
                .with_children(|b| {
                    b.spawn(ui::text(f, label, 15.0, INK));
                });
        };
        step(p, "\u{2212}", -1);
        p.spawn(Node { min_width: Val::Px(if field == Field::Size { 118.0 } else { 64.0 }), justify_content: JustifyContent::Center, ..default() }).with_children(|v| {
            v.spawn(ui::text(f, &c.value(field), 17.0, INK));
        });
        step(p, "+", 1);
        if field == Field::Seed {
            p.spawn((Node { padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)), border: UiRect::all(Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(9.0)), margin: UiRect::left(Val::Px(6.0)), ..default() }, BackgroundColor(BTN), BorderColor::all(Color::NONE), Interaction::default(), FocusPolicy::Block, ui::Hoverable, Focusable(Focus::Draw), Press::Draw))
                .with_children(|b| {
                    b.spawn(ui::text(f, "Draw another", 14.0, INK));
                });
        }
    }
}

fn foot(root: &mut ChildSpawnerCommands, f: &Handle<Font>) {
    root.spawn((ui::card(Node { flex_direction: FlexDirection::Column, width: Val::Px(1078.0), max_width: Val::Percent(100.0), padding: UiRect::axes(Val::Px(22.0), Val::Px(12.0)), row_gap: Val::Px(8.0), border_radius: BorderRadius::all(Val::Px(16.0)), flex_shrink: 0.0, ..default() }), Fits::Foot)).with_children(|c| {
        // the command line that does the same
        c.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(10.0), ..default() }).with_children(|row| {
            row.spawn(Node { width: Val::Px(84.0), flex_shrink: 0.0, ..default() }).with_children(|l| {
                l.spawn((ui::text(f, "Command", 14.5, MUTED), tip("Command", COMMAND_SAYS)));
            });
            row.spawn((Node { flex_grow: 1.0, flex_shrink: 1.0, padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgb(0.95, 0.93, 0.87)))).with_children(|t| {
                t.spawn((ui::text(f, "", 14.0, INK), CommandText));
            });
            foot_button(row, f, "Copy", Focus::Copy, Press::Copy);
        });
        c.spawn((ui::text(f, "", 13.5, INK), SaidText, Node { display: Display::None, ..default() }));
        c.spawn((Node { height: Val::Px(1.0), ..default() }, BackgroundColor(RULE)));
        c.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() }).with_children(|row| {
            row.spawn((Node { flex_grow: 1.0, ..default() },)).with_children(|v| {
                v.spawn(ui::text(f, &format!("Isolated State {} \u{00b7} commit {}", crate::VERSION, crate::note::COMMIT), 13.5, MUTED));
            });
            // What's new, with its count once the devlog is read
            row.spawn((ui_button(), BackgroundColor(BTN), BorderColor::all(Color::NONE), Interaction::default(), FocusPolicy::Block, ui::Hoverable, Focusable(Focus::WhatsNew), crate::whats_new::OpenWhatsNew)).with_children(|b| {
                b.spawn(ui::text(f, "What's new", 14.0, INK));
                b.spawn((Node { padding: UiRect::axes(Val::Px(7.0), Val::Px(0.0)), border_radius: BorderRadius::all(Val::Px(8.0)), display: Display::None, ..default() }, BackgroundColor(crate::whats_new::NEW_INK), NewCount)).with_children(|n| {
                    n.spawn(ui::text(f, "", 12.5, Color::WHITE));
                });
            });
            row.spawn((ui_button(), BackgroundColor(BTN), BorderColor::all(Color::NONE), Interaction::default(), FocusPolicy::Block, ui::Hoverable, Focusable(Focus::Backlog), crate::backlog_seen::OpenBacklog)).with_children(|b| {
                b.spawn(ui::text(f, "Backlog", 14.0, INK));
            });
            foot_button(row, f, "Notes folder", Focus::Notes, Press::Notes);
            foot_button(row, f, "Quit", Focus::Quit, Press::Quit);
        });
    });
}

fn ui_button() -> Node {
    Node { padding: UiRect::axes(Val::Px(11.0), Val::Px(4.0)), border: UiRect::all(Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(10.0)), column_gap: Val::Px(7.0), align_items: AlignItems::Center, flex_shrink: 0.0, ..default() }
}

fn foot_button(p: &mut ChildSpawnerCommands, f: &Handle<Font>, label: &str, focus: Focus, press: Press) {
    p.spawn((ui_button(), BackgroundColor(BTN), BorderColor::all(Color::NONE), Interaction::default(), FocusPolicy::Block, ui::Hoverable, Focusable(focus), press)).with_children(|b| {
        b.spawn(ui::text(f, label, 14.0, INK));
    });
}

// ------------------------------------------------------------------ what the page does

/// The pointer over something gives it the focus, so that the command line
/// at the foot says what a click there would start.
fn pointer(mut page: ResMut<Page>, q: Query<(&Interaction, &Focusable), Changed<Interaction>>) {
    for (i, f) in &q {
        if *i == Interaction::Hovered && page.focus != f.0 {
            page.focus = f.0;
        }
    }
}

/// A click on a chip, a step, a saved game or a button.
fn presses(mut page: ResMut<Page>, q: Query<(&Interaction, &Press), Changed<Interaction>>) {
    for (i, press) in &q {
        if *i == Interaction::Pressed {
            act(&mut page, *press);
        }
    }
}

fn act(page: &mut Page, press: Press) {
    match press {
        Press::Step(f, by) => {
            page.choices.step(f, by);
            page.focus = Focus::Field(f);
        }
        Press::Pick(f, i) => {
            page.choices.set(f, i);
            page.focus = Focus::Field(f);
        }
        Press::GoOn(i) => {
            page.focus = Focus::Save(i);
            page.go = Some(page.args());
        }
        Press::Draw => {
            // another island: any number is one, so a number from the clock will do
            let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.subsec_nanos() as u64);
            let mut seed = 1 + (nanos / 7 + page.choices.seed * 7_919) % 999;
            if seed == page.choices.seed {
                seed = seed % 999 + 1;
            }
            page.choices.seed = seed;
            page.focus = Focus::Draw;
        }
        Press::Begin => {
            page.focus = Focus::Begin;
            page.go = Some(page.choices.args());
        }
        Press::More => {
            page.more = !page.more;
            page.focus = Focus::More;
        }
        Press::Copy => {
            let line = command_line(&program(), &page.args());
            copy(&line);
            page.said = Some("Copied.".into());
        }
        Press::Notes => {
            let dir = saves::notes();
            let _ = std::fs::create_dir_all(&dir);
            if let Err(e) = open_folder(&dir) {
                page.said = Some(format!("The folder could not be opened: {}", saves::plainly(&e)));
            }
        }
        Press::Quit => page.go = Some(Vec::new()),
    }
}

/// The keys: Up, Down and Tab move through the page; Left and Right change
/// what has the focus; Enter (or Space) starts what is lit or presses a
/// button; Escape closes More, or quits. A card open over the page (What's
/// new, the backlog) has the keys to itself, and Escape closes it.
fn keys(keys: Res<ButtonInput<KeyCode>>, mut page: ResMut<Page>, wn: Res<crate::whats_new::WhatsNew>, backlog: Res<crate::backlog_seen::BacklogCard>, mut buttons: Query<(&Focusable, &mut Interaction)>) {
    if wn.open || backlog.open || page.go.is_some() {
        return;
    }
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if keys.just_pressed(KeyCode::ArrowDown) || (keys.just_pressed(KeyCode::Tab) && !shift) {
        page.move_focus(1);
    }
    if keys.just_pressed(KeyCode::ArrowUp) || (keys.just_pressed(KeyCode::Tab) && shift) {
        page.move_focus(-1);
    }
    for (key, by) in [(KeyCode::ArrowLeft, -1), (KeyCode::ArrowRight, 1)] {
        if keys.just_pressed(key) {
            match page.focus {
                Focus::Field(f) => page.choices.step(f, by),
                Focus::More if (by > 0) != page.more => act(&mut page, Press::More),
                _ => {}
            }
        }
    }
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space]) {
        match page.focus {
            Focus::Save(i) => act(&mut page, Press::GoOn(i)),
            // a field of the new island, or of More: Enter begins with the choices as they stand
            Focus::Field(_) | Focus::Begin => act(&mut page, Press::Begin),
            Focus::Draw => act(&mut page, Press::Draw),
            Focus::More => act(&mut page, Press::More),
            Focus::Copy => act(&mut page, Press::Copy),
            Focus::Notes => act(&mut page, Press::Notes),
            Focus::Quit => act(&mut page, Press::Quit),
            // the game's own cards open as their buttons are pressed
            f @ (Focus::WhatsNew | Focus::Backlog) => {
                for (b, mut i) in &mut buttons {
                    if b.0 == f {
                        *i = Interaction::Pressed;
                    }
                }
            }
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        if page.more {
            page.more = false;
            if matches!(page.focus, Focus::Field(f) if Field::MORE.contains(&f)) {
                page.focus = Focus::More;
            }
        } else {
            page.go = Some(Vec::new());
        }
    }
}

/// Lay again what changed: every field's options and line, More, the focus's
/// ring and the command line.
#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    page: Res<Page>,
    font: Res<UiFont>,
    boxes: Query<(Entity, &FieldBox)>,
    mut lines: Query<(&mut Text, &FieldLine), (Without<CommandText>, Without<SaidText>, Without<MoreWord>)>,
    mut command: Query<&mut Text, (With<CommandText>, Without<SaidText>, Without<MoreWord>)>,
    mut more_word: Query<&mut Text, (With<MoreWord>, Without<SaidText>)>,
    mut said: Query<(&mut Text, &mut Node), (With<SaidText>, Without<CommandText>)>,
    mut more: Query<&mut Node, (With<MoreCard>, Without<SaidText>, Without<FieldRow>)>,
    mut rows: Query<(&mut Node, &FieldRow), (Without<SaidText>, Without<MoreCard>)>,
    mut rings: Query<(&Focusable, &mut BorderColor)>,
) {
    if !page.is_changed() {
        return;
    }
    let f = &font.0;
    for (e, b) in &boxes {
        commands.entity(e).despawn_children();
        let (field, c) = (b.0, page.choices.clone());
        commands.entity(e).with_children(|p| lay_field(p, f, field, &c));
    }
    for (mut t, l) in &mut lines {
        t.0 = page.choices.line(l.0);
    }
    for mut t in &mut command {
        t.0 = command_line(&program(), &page.args());
    }
    for (mut t, mut n) in &mut said {
        t.0 = page.said.clone().unwrap_or_default();
        n.display = if page.said.is_some() { Display::Flex } else { Display::None };
    }
    for mut n in &mut more {
        n.display = if page.more { Display::Flex } else { Display::None };
    }
    for (mut n, r) in &mut rows {
        let display = if r.0.shown(&page.choices) { Display::Flex } else { Display::None };
        if n.display != display {
            n.display = display;
        }
    }
    for mut t in &mut more_word {
        t.0 = if page.more { "Less" } else { "More" }.to_string();
    }
    for (fo, mut b) in &mut rings {
        *b = BorderColor::all(if fo.0 == page.focus { FOCUS } else { Color::NONE });
    }
}

/// The count of what is new on What's new's button, once the devlog is read;
/// the game's chip for it, which stands at the top right of a game, is not shown here.
fn news_count(wn: Res<crate::whats_new::WhatsNew>, mut chip: Query<&mut Visibility, With<crate::whats_new::StartChip>>, mut count: Query<(&mut Node, &Children), With<NewCount>>, mut texts: Query<&mut Text>, mut done: Local<bool>) {
    for mut v in &mut chip {
        v.set_if_neq(Visibility::Hidden);
    }
    if *done || !wn.ready {
        return;
    }
    *done = true;
    for (mut n, children) in &mut count {
        n.display = if wn.new > 0 { Display::Flex } else { Display::None };
        for c in children.iter() {
            if let Ok(mut t) = texts.get_mut(c) {
                t.0 = wn.new.to_string();
            }
        }
    }
}

/// Say once how long the page took to be laid out from the program's start.
fn up_after(page: Res<Page>, root: Query<&ComputedNode, With<Root>>, mut said: Local<bool>) {
    if *said || !root.iter().any(|n| n.size().x > 0.0) {
        return;
    }
    *said = true;
    eprintln!("the start page is up {:.2} s after the program started", page.t0.elapsed().as_secs_f32());
}

/// Start the program again with what was chosen, and close; or only close.
fn go(mut page: ResMut<Page>, mut exit: MessageWriter<AppExit>, shooting: Option<Res<Shooting>>) {
    // a picture's page is pressed by its own system, when the picture is taken
    if shooting.is_some() {
        return;
    }
    let Some(args) = page.go.clone() else { return };
    if args.is_empty() {
        eprintln!("the start page is closed");
        exit.write(AppExit::Success);
        return;
    }
    let Mode::Window { from_game } = page.mode else { return };
    eprintln!("the start page starts: {}", command_line(&program(), &args));
    // the choices are kept for next time, off the main thread (a fresh folder is made for them)
    let (path, text) = (kept_at(), page.choices.to_text());
    let keeping = std::thread::spawn(move || {
        let _ = path.parent().map(std::fs::create_dir_all);
        let _ = std::fs::write(&path, text);
    });
    match launch(&args, from_game) {
        Ok(_) => {
            let _ = keeping.join();
            exit.write(AppExit::Success);
        }
        Err(e) => {
            page.go = None;
            page.said = Some(format!("The game could not be started: {}", saves::plainly(&e)));
        }
    }
}

/// The program, started again with `args`. A copy started from a game's
/// Start page (`from_game`) is told that it was restarted, so that a game
/// gone on with waits for the one closing to let go of it.
pub fn launch(args: &[String], from_game: bool) -> std::io::Result<std::process::Child> {
    let mut c = std::process::Command::new(std::env::current_exe()?);
    c.args(args);
    if from_game {
        c.env("ISOSTATE_RESTARTED", "1");
    } else {
        c.env_remove("ISOSTATE_RESTARTED");
    }
    c.spawn()
}

/// Put a line on the system's clipboard, on a thread of its own.
fn copy(line: &str) {
    let line = line.to_string();
    std::thread::spawn(move || {
        use std::io::Write;
        let tries: &[(&str, &[&str])] = if cfg!(windows) {
            &[("powershell", &["-NoProfile", "-NonInteractive", "-Command", "[Console]::In.ReadToEnd() | Set-Clipboard"])]
        } else if cfg!(target_os = "macos") {
            &[("pbcopy", &[])]
        } else {
            &[("wl-copy", &[]), ("xclip", &["-selection", "clipboard"])]
        };
        for (cmd, args) in tries {
            let mut c = std::process::Command::new(cmd);
            c.args(*args).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                // no console window flashing up over the page
                c.creation_flags(0x0800_0000);
            }
            if let Ok(mut child) = c.spawn() {
                if let Some(mut input) = child.stdin.take() {
                    let _ = input.write_all(line.as_bytes());
                }
                if child.wait().is_ok_and(|s| s.success()) {
                    return;
                }
            }
        }
        eprintln!("the start page could not put the command line on the clipboard");
    });
}

/// Open a folder in the system's own window for files.
fn open_folder(dir: &std::path::Path) -> std::io::Result<()> {
    let cmd = if cfg!(windows) { "explorer" } else if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    std::process::Command::new(cmd).arg(dir).spawn().map(|_| ())
}

// ------------------------------------------------------------------ a picture of the page

#[derive(Resource)]
struct Shooting {
    ask: PictureAsk,
    frames: u32,
    image: Option<Handle<Image>>,
    asked_at: Option<u32>,
    held: Option<std::time::Instant>,
}

/// With nobody at the page: wait for it to be laid out, its font drawn, its
/// shaders made and What's new read, take its picture, and with `begin` press
/// Begin, starting the game it would start with a picture of its own.
fn shoot(mut commands: Commands, mut s: ResMut<Shooting>, mut page: ResMut<Page>, wn: Res<crate::whats_new::WhatsNew>, waiting: Res<crate::PipelinesWaiting>, mut tips: ResMut<crate::tips::Tips>, mut exit: MessageWriter<AppExit>) {
    s.frames += 1;
    // the pointer rests on a name, as a script's does (tips.rs), long enough for its tooltip to lock
    if tips.aim.is_none() {
        tips.aim = s.ask.tip.clone();
    }
    let Some(image) = s.image.clone() else { return };
    match s.asked_at {
        None => {
            let held = *s.held.get_or_insert_with(std::time::Instant::now);
            let pending = waiting.0.load(std::sync::atomic::Ordering::Relaxed);
            let ready = (wn.ready || held.elapsed().as_secs_f32() > 10.0) && pending == 0 && (s.ask.tip.is_none() || held.elapsed().as_secs_f32() > 2.5);
            if s.frames < 20 || (!ready && held.elapsed().as_secs_f32() < 45.0) {
                return;
            }
            if let Some(parent) = std::path::Path::new(&s.ask.path).parent().filter(|p| !p.as_os_str().is_empty()) {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::remove_file(&s.ask.path);
            commands.spawn(Screenshot::image(image)).observe(save_to_disk(s.ask.path.clone()));
            s.asked_at = Some(s.frames);
        }
        Some(at) => {
            let there = std::path::Path::new(&s.ask.path).exists();
            if !there && s.frames < at + 300 {
                return;
            }
            if !there {
                eprintln!("the picture of the start page was not written: {}", s.ask.path);
                exit.write(AppExit::from_code(1));
                return;
            }
            eprintln!("the start page's picture is in {} ({:.1} s after the program started)", s.ask.path, page.t0.elapsed().as_secs_f32());
            if s.ask.begin {
                // Begin, as a click would press it, with a picture of what opens beside the page's
                act(&mut page, Press::Begin);
                let mut args = page.go.take().unwrap_or_default();
                let path = std::path::Path::new(&s.ask.path);
                let island = path.with_file_name(format!("{}-island.png", path.file_stem().map(|x| x.to_string_lossy().into_owned()).unwrap_or_default()));
                eprintln!("the start page starts: {} (and --shot {} for the picture)", command_line(&program(), &args), island.display());
                args.extend(["--shot".to_string(), island.to_string_lossy().into_owned()]);
                match launch(&args, false).and_then(|mut c| c.wait()) {
                    Ok(status) if status.success() && island.exists() => eprintln!("the game it started took its picture: {}", island.display()),
                    Ok(status) => {
                        eprintln!("the game it started ended with {status} and {} picture", if island.exists() { "a" } else { "no" });
                        exit.write(AppExit::from_code(1));
                        return;
                    }
                    Err(e) => {
                        eprintln!("the game could not be started: {e}");
                        exit.write(AppExit::from_code(1));
                        return;
                    }
                }
            }
            exit.write(AppExit::Success);
        }
    }
}

// ------------------------------------------------------------------ a game's way back

/// The help card's button that closes the game and opens the start page.
#[derive(Component)]
pub struct OpenStartPage;

/// In a game: the help card's Start page. The game is saved as it closes
/// (`live::Live`'s drop waits for its thread and its save).
pub fn game_plugin(app: &mut App) {
    app.add_systems(Update, back_to_page);
}

fn back_to_page(q: Query<&Interaction, (Changed<Interaction>, With<OpenStartPage>)>, mut exit: MessageWriter<AppExit>) {
    if !q.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    if std::env::args().any(|a| a == "--script") {
        // a scripted run is one program from start to end: say what would be started, and stop
        println!("script: Start page would start the program again with: --start-page");
        exit.write(AppExit::Success);
        return;
    }
    let mut c = match std::env::current_exe() {
        Ok(exe) => std::process::Command::new(exe),
        Err(e) => {
            eprintln!("the start page could not be opened: {e}");
            return;
        }
    };
    c.arg("--start-page").env_remove("ISOSTATE_RESTARTED");
    match c.spawn() {
        Ok(_) => {
            eprintln!("closing the game for the start page");
            exit.write(AppExit::Success);
        }
        Err(e) => eprintln!("the start page could not be opened: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn only_an_empty_line_or_the_pages_own_options_open_the_page() {
        let none: Vec<String> = Vec::new();
        let window = Some(Asked { mode: Mode::Window { from_game: false }, size: None });
        assert_eq!(wanted(&none, &none), Ok(window.clone()));
        // the saves' folder named is still a player's start; any other variable of the house is a tool's
        assert_eq!(wanted(&none, &["ISOSTATE_SAVES".into(), "PATH".into()]), Ok(window));
        for tool in ["ISOSTATE_PLAN", "ISOSTATE_BENCH", "ISOSTATE_RESTARTED", "ISOSTATE_FRAMES", "ISOSTATE_LAND_REPORT"] {
            assert_eq!(wanted(&none, &[tool.into()]), Ok(None), "{tool}");
        }
        // a run named, and every start the house's scripts and pictures make, never meets the page
        for start in [
            "runs/rust7/run.bin",
            "runs/x/run.bin --year 1850 --shot out.png",
            "--family",
            "--sandbox",
            "--new",
            "--seed 11 --families 4",
            "--size 1600x1000",
            "--no-tilt",
            "--advance 5 --sandbox --shot trial.png",
            "--shots out/std",
            "--lab all --out out/lab",
            "--script click:Begin",
            "--describe 31,29",
            "--load a.json",
            "--help",
            "--version",
        ] {
            assert_eq!(wanted(&line(start), &[]), Ok(None), "{start}");
        }
        // asked for by name
        assert_eq!(wanted(&line("--start-page"), &[]), Ok(Some(Asked { mode: Mode::Window { from_game: true }, size: None })));
        assert_eq!(
            wanted(&line("--page-shot out/p.png,more,begin --size 1280x720"), &[]),
            Ok(Some(Asked { mode: Mode::Picture(PictureAsk { path: "out/p.png".into(), more: true, family: false, begin: true, tip: None, start: None, no_settlers: false, no_ideas: false }), size: Some((1280, 720)) }))
        );
        assert!(wanted(&line("--start-page --family"), &[]).unwrap_err().contains("--family is an option of a game"));
        assert!(wanted(&line("--page-shot"), &[]).is_err());
        assert!(wanted(&line("--page-shot p.png,blue"), &[]).unwrap_err().contains("blue"));
    }

    /// One row a choice: the choice made on the page, the arguments it gives,
    /// and what the command line's own reader makes of them, which must be
    /// the game the choice says.
    #[test]
    fn every_choice_gives_the_options_its_argument_gives() {
        type Check = fn(&crate::Args) -> bool;
        let rows: Vec<(&str, Box<dyn Fn(&mut Choices)>, &str, Check)> = vec![
            ("as it opens: a new sandbox on island 7", Box::new(|_| {}), "--sandbox --new", |a| a.sandbox && a.new && a.fresh && a.seed == 7 && a.families == 6 && a.to.is_none() && a.advance == 0),
            ("Play: Family game", Box::new(|c| c.set(Field::Kind, 1)), "--family --new", |a| !a.sandbox && a.family && a.fresh),
            ("Start: Settled island", Box::new(|c| c.set(Field::Start, 0)), "--sandbox --new", |a| a.sandbox && !a.old_land),
            ("Island: 8", Box::new(|c| c.step(Field::Seed, 1)), "--sandbox --new --seed 8", |a| a.seed == 8),
            ("Island: drawn, 412", Box::new(|c| c.seed = 412), "--sandbox --new --seed 412", |a| a.seed == 412),
            ("Families: 5", Box::new(|c| c.step(Field::Families, -1)), "--sandbox --new --families 5", |a| a.families == 5),
            ("Last year: 2025", Box::new(|c| c.step(Field::To, -1)), "--sandbox --new --to 2025", |a| a.to == Some(2025)),
            ("Last year: 1900", Box::new(|c| c.to = 1900), "--sandbox --new --to 1900", |a| a.to == Some(1900)),
            ("Opens in: 1760", Box::new(|c| c.step(Field::Advance, 1)), "--sandbox --new --advance 9", |a| a.advance == 9),
            ("Opens in: 1900, family", Box::new(|c| { c.set(Field::Kind, 1); c.advance = 149; }), "--family --new --advance 149", |a| a.advance == 149 && !a.sandbox),
            ("Land: Old island", Box::new(|c| c.set(Field::Land, 1)), "--sandbox --new --old-land", |a| a.old_land && a.fresh),
            ("Season: Summer", Box::new(|c| c.set(Field::Season, 2)), "--sandbox --new --season 0.5", |a| a.season == Some(0.5)),
            ("Season: Winter", Box::new(|c| c.set(Field::Season, 4)), "--sandbox --new --season 0", |a| a.season == Some(0.0)),
            ("Window: 1920 x 1080", Box::new(|c| { c.step(Field::Size, 1); }), "--sandbox --new --size 1920x1080", |a| a.size == (1920, 1080)),
            ("Map view: Rent", Box::new(|c| c.step(Field::View, 1)), "--sandbox --new --overlay rent", |a| a.overlay.as_deref() == Some("rent")),
            ("Start: Landfall", Box::new(|c| c.set(Field::Start, 1)), "--sandbox --new --landfall", |a| a.landfall && a.sandbox && a.fresh),
            ("Start: Landfall, family", Box::new(|c| { c.set(Field::Kind, 1); c.set(Field::Start, 1); }), "--family --new --landfall", |a| a.landfall && !a.sandbox),
            ("Landfall, Abroad: Settlers off", Box::new(|c| { c.set(Field::Start, 1); c.set(Field::Abroad, 0); }), "--sandbox --new --landfall --settlers off", |a| a.landfall && a.switches == isostate_core::save::Switches { settlers: Some(false), ideas_by_sea: None }),
            ("Landfall, Abroad: Technology off", Box::new(|c| { c.set(Field::Start, 1); c.set(Field::Abroad, 1); }), "--sandbox --new --landfall --ideas-by-sea off", |a| a.landfall && a.switches == isostate_core::save::Switches { settlers: None, ideas_by_sea: Some(false) }),
            ("Landfall alone, family", Box::new(|c| { c.set(Field::Kind, 1); c.set(Field::Start, 1); c.step(Field::Abroad, -1); }), "--family --new --landfall --settlers off --ideas-by-sea off", |a| a.landfall && !a.sandbox && a.switches == isostate_core::save::Switches { settlers: Some(false), ideas_by_sea: Some(false) }),
            ("Landfall, Settlers off and on again", Box::new(|c| { c.set(Field::Start, 1); c.set(Field::Abroad, 0); c.set(Field::Abroad, 0); }), "--sandbox --new --landfall", |a| a.landfall && a.switches.plain()),
            // (the settled island has neither: a choice left off under Landfall says nothing there)
            ("Settled island after Landfall alone", Box::new(|c| { c.set(Field::Start, 1); c.set(Field::Abroad, 0); c.set(Field::Abroad, 1); c.set(Field::Start, 0); }), "--sandbox --new", |a| !a.landfall && a.switches.plain()),
        ];
        for (name, choose, want, check) in rows {
            let mut c = Choices::default();
            choose(&mut c);
            let args = c.args();
            assert_eq!(args.join(" "), want, "{name}");
            let parsed = crate::parse(&args).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(check(&parsed), "{name}: {want} does not open the game the choice says");
            // and the game it opens is a new one with somebody at it, kept as any such start keeps it
            let o = crate::opening(&parsed);
            assert!(!o.plain() && !o.unattended, "{name}");
        }
        // every start the simulation offers is on the page, with an argument the command line knows
        assert_eq!(starts().iter().map(|s| s.word).collect::<Vec<_>>(), isostate_core::landfall::STARTS);
        for s in starts() {
            assert!(s.arg.is_none() || crate::USAGE.contains(s.arg.unwrap()), "{}", s.word);
        }
    }

    #[test]
    fn a_saved_game_is_gone_on_with_by_name_and_kept_where_it_was() {
        let dir = saves::test_dir("start-page-saves");
        let listed = |place: Place, path: &str| Listed { path: PathBuf::from(path), place, seed: 11, families: 4, year: 1800, month: 3, to: 2050, family: "Fairfax".into(), old_land: false, landfall: false, switches: Default::default(), when: 0 };
        let fam = crate::parse(&listed(Place::Family, "s/island-11-families-4.json").args()).unwrap();
        assert!(!fam.sandbox && fam.family && fam.load.as_deref() == Some("s/island-11-families-4.json"));
        assert_eq!(crate::opening(&fam).keeping(), crate::live::Keeping::Autosave);
        let sb = crate::parse(&listed(Place::Sandbox, "s/sandbox/island-11-families-4.json").args()).unwrap();
        assert!(sb.sandbox && sb.load.is_some());
        assert_eq!(crate::opening(&sb).keeping(), crate::live::Keeping::Session);
        let trial = crate::parse(&listed(Place::Trial, "s/trials/island-11-families-4.json").args()).unwrap();
        assert_eq!(crate::opening(&trial).keeping(), crate::live::Keeping::Named("s/trials/island-11-families-4.json".into()));
        // the window's choices go with it
        let mut page = Page { choices: Choices::default(), saves: vec![listed(Place::Sandbox, "a b/x.json")], focus: Focus::Save(0), more: false, said: None, mode: Mode::Window { from_game: false }, go: None, t0: std::time::Instant::now() };
        page.choices.step(Field::Size, -1);
        assert_eq!(page.args(), ["--sandbox", "--load", "a b/x.json", "--size", "1600x900"]);
        assert_eq!(command_line("isolated-state", &page.args()), "isolated-state --sandbox --load \"a b/x.json\" --size 1600x900");
        // the listing: each island's own game, session and trial, newest first, and nothing set aside
        std::fs::create_dir_all(dir.join("sandbox")).unwrap();
        std::fs::create_dir_all(dir.join("trials")).unwrap();
        let game = isostate_core::save::Game::under(isostate_core::params::Rules::Game, 11, 4, 0);
        let text = game.save(0).to_text();
        for name in ["island-11-families-4.json", "island-11-families-4.1.json", "island-11-families-4.previous.json", "sandbox/island-11-families-4.json", "trials/island-11-families-4.json", "notes.json"] {
            std::fs::write(dir.join(name), &text).unwrap();
        }
        let found = saves::listed_in(&dir);
        let places: Vec<Place> = found.iter().map(|l| l.place).collect();
        assert_eq!(found.len(), 3, "{places:?}");
        assert!(places.contains(&Place::Family) && places.contains(&Place::Sandbox) && places.contains(&Place::Trial));
        assert!(found.iter().all(|l| l.seed == 11 && l.families == 4 && l.year == 1749));
        assert_eq!(saves::named_for("island-7-families-6.json"), Some((7, 6)));
        assert_eq!(saves::named_for("island-7-families-6.previous.json"), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_choices_are_kept_and_read_back() {
        let mut c = Choices::default();
        c.set(Field::Kind, 1);
        c.seed = 412;
        c.step(Field::Families, 2);
        c.to = 1950;
        c.advance = 49;
        c.set(Field::Land, 1);
        c.set(Field::Season, 3);
        c.step(Field::Size, -2);
        c.step(Field::View, 3);
        c.set(Field::Start, 1);
        c.set(Field::Abroad, 0);
        assert_eq!(Choices::from_text(&c.to_text()), c);
        c.set(Field::Abroad, 1);
        assert_eq!(Choices::from_text(&c.to_text()), c);
        // a file kept before the two switches were on the page has both on, as landfall has them
        let before = "kind: sandbox\nstart: landfall\nseed: 7\n";
        assert!(Choices::from_text(before).settlers && Choices::from_text(before).ideas);
        // what cannot be read is left as it is by default
        assert_eq!(Choices::from_text("seed: x\nfamilies: 99\nto: 1913\nsize: 13x7\nview: rents\nnonsense"), Choices::default());
        // a game that would open after its last year opens as late as it can
        let mut c = Choices { advance: 249, ..Choices::default() };
        c.to = 2000;
        c.step(Field::To, -1);
        assert_eq!((c.to, c.advance), (1975, 199));
        assert!(c.advances().iter().all(|a| 1751 + *a as i64 <= c.to));
    }

    /// Every choice and button is reached by the keys alone: Down goes through
    /// all of them and round, and Left or Right changes every field.
    #[test]
    fn every_choice_is_reached_without_the_mouse() {
        let listed = Listed { path: PathBuf::from("x.json"), place: Place::Sandbox, seed: 7, families: 6, year: 1790, month: 0, to: 2050, family: String::new(), old_land: false, landfall: false, switches: Default::default(), when: 0 };
        let mut page = Page { choices: Choices::default(), saves: vec![listed.clone(), listed], focus: Focus::Save(0), more: false, said: None, mode: Mode::Window { from_game: false }, go: None, t0: std::time::Instant::now() };
        // on the settled island, and under Landfall with its two switches
        for (more, start) in [(false, 0), (true, 0), (false, 1), (true, 1)] {
            page.more = more;
            page.choices = Choices { start, ..Choices::default() };
            assert_eq!(page.order().contains(&Focus::Field(Field::Abroad)), start == 1);
            let order = page.order();
            let mut seen = Vec::new();
            page.focus = order[0];
            for _ in 0..order.len() {
                seen.push(page.focus);
                page.move_focus(1);
            }
            assert_eq!(seen, order);
            assert_eq!(page.focus, order[0], "round again");
            page.move_focus(-1);
            assert_eq!(page.focus, *order.last().unwrap());
            for want in [Focus::Save(1), Focus::Draw, Focus::Begin, Focus::More, Focus::Copy, Focus::WhatsNew, Focus::Backlog, Focus::Notes, Focus::Quit] {
                assert!(order.contains(&want), "{want:?}");
            }
            let fields: Vec<Field> = Field::NEW.iter().chain(if more { &Field::MORE[..] } else { &[] }).copied().filter(|f| f.shown(&page.choices)).collect();
            for f in fields {
                assert!(order.contains(&Focus::Field(f)), "{f:?}");
                let before = page.choices.clone();
                page.choices.step(f, 1);
                if page.choices == before {
                    page.choices.step(f, -1);
                }
                // (a start with one way to begin has nothing to change to)
                assert!(page.choices != before || (f == Field::Start && starts().len() == 1), "{f:?} does not change with the keys");
                // (Start stepped away from Landfall would hide the rows after it)
                page.choices.start = start;
            }
        }
    }

    #[test]
    fn the_pages_lines_are_short() {
        let mut c = Choices::default();
        for kind in [0, 1] {
            c.set(Field::Kind, kind);
            for (adv, on) in [(0, true), (9, false), (149, true)] {
                c.advance = adv;
                c.settlers = on;
                c.ideas = on;
                for f in Field::NEW.iter().chain(&Field::MORE) {
                    let l = c.line(*f);
                    for sentence in l.split(['.', ':']).filter(|s| !s.trim().is_empty()) {
                        assert!(sentence.split_whitespace().count() <= 12, "{f:?}: {sentence}");
                    }
                    assert!(crate::words::kept_out_in(&l).is_empty(), "{f:?}: {l} holds {:?}", crate::words::kept_out_in(&l));
                    for (_, name) in c.options(*f).0.iter().enumerate() {
                        assert_eq!(crate::words::label_fault(name), None, "{f:?}");
                    }
                }
            }
        }
        for s in starts() {
            assert!(s.line.split_whitespace().count() <= 12, "{}", s.line);
            assert_eq!(crate::words::label_fault(s.name), None);
        }
        // the tooltips keep the glossary's rules (docs/designs/words.md): three sentences, 18 words each
        let says: Vec<(&str, &str)> = Field::NEW.iter().chain(&Field::MORE).map(|f| (f.label(), f.says())).chain([("Command", COMMAND_SAYS), ("Go on", GO_ON_SAYS)]).collect();
        for (name, text) in says {
            assert_eq!(crate::words::label_fault(name), None);
            let s = crate::words::sentences(text);
            assert!((1..=3).contains(&s.len()), "{name}: {} sentences", s.len());
            for sen in &s {
                assert!(sen.split_whitespace().count() <= 18 && sen.ends_with('.') && !sen.contains([':', ';', '(', ')']), "{name}: {sen}");
            }
            assert!(crate::words::kept_out_in(text).is_empty(), "{name}: {:?}", crate::words::kept_out_in(text));
        }
        assert_eq!(ago(1_000, 1_000 + 7_300), "2 h ago");
        assert_eq!(ago(0, 3 * 86_400 + 5), "3 days ago");
    }
}
