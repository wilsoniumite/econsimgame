//! Isolated State: a 3D replay viewer for runs of the island economy.
//!
//!     cargo run -p isostate-viewer --release -- runs/landlords/run.bin
//!     cargo run -p isostate-viewer --release -- runs/landlords/run.bin --year 1850 --shot out.png
//!
//! Space plays and pauses, the arrow keys step through years, drag to pan,
//! right-drag to turn, scroll to zoom.
//!
//! With `--lab SHEET --out DIR` there is no game: the viewer writes contact
//! sheets of the scenery's generators and stops (lab.rs).

mod ages_chart;
mod back;
// the backlog seen from the help card, with her notes of the day
mod backlog_seen;
mod purse_chart;
mod boats;
mod built;
mod carts;
mod chart;
mod clock;
mod course;
mod cursor;
mod family;
mod fences;
mod fields;
mod frames;
mod game;
mod ground;
mod hand_tab;
mod handover;
mod icons;
// IDEAS-PLATE: the island's ideas as a timeline (ideas_plate.rs)
mod ideas_plate;
mod firm_card;
mod inspector;
mod lab;
mod live;
mod look;
mod meshkit;
mod network;
mod newland;
// a note in her words, kept where she stands (N)
mod note;
mod overlay;
mod pick;
mod plan;
mod profile;
mod ranges;
mod rails;
mod replay;
mod rivers;
mod road_course;
mod road_ground;
mod rulebook;
mod sandbox;
mod saves;
mod scatter;
mod script;
mod shots;
// the page a start with nothing on the command line opens (start_page.rs)
mod start_page;
mod take_back;
mod terrain;
mod tips;
mod town_ideas;
// MAKING: a town's making as a panel (town_flow.rs)
mod town_flow;
mod traffic;
mod trains;
mod ui;
// what's new, read from docs/DEVLOG.md (H, then What's new)
mod whats_new;
mod work_charts;
mod words;
mod world;
mod year;

use bevy::app::ScheduleRunnerPlugin;
use bevy::camera::{Exposure, Hdr, RenderTarget};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::light::CascadeShadowConfigBuilder;
use bevy::pbr::{DistanceFog, FogFalloff, ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel};
use bevy::post_process::bloom::Bloom;
use bevy::post_process::dof::{DepthOfField, DepthOfFieldMode};
use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use std::sync::Arc;
use bevy::window::{ExitCondition, WindowResolution};
use bevy::winit::WinitPlugin;

use isostate_gen::trees::in_leaf;
use look::{Look, TheLook, ToColor};
use replay::Replay;
use ui::{kind_colour, kind_label, text, UiState, INK, MUTED, PAPER};
use world::Ground;
// (the year's rebuild is in `year.rs`; the rest of the viewer reaches these as `crate::...`)
use year::{check_scenery, rebuild_year, scenery_line, scenery_ready, Builds, Shown, PRINT_YEAR_MESHES};
#[cfg(test)]
use year::mesh_sum;

// ------------------------------------------------------------------ command line

struct Args {
    run: String,
    year: Option<i32>,
    shot: Option<String>,
    size: (u32, u32),
    /// the point the camera looks at, in map cells (`--at x,z`)
    at: Option<(f32, f32)>,
    /// a look file laid over the standard look (`--look FILE`, look.rs)
    look_file: Option<String>,
    yaw: f32,
    pitch: f32,
    dist: Option<f32>,
    frames: u32,
    /// the longest a picture waits for the renderer's shaders, in seconds (`--wait`)
    wait: Option<f32>,
    tilt: bool,
    views: Vec<ShotSpec>,
    season: Option<f32>,
    describe: Option<(f32, f32)>,
    /// a new live game instead of a recorded run (the default when no run is named)
    new: bool,
    named_run: bool,
    seed: u64,
    families: usize,
    /// years to play before the window opens
    advance: usize,
    /// give the player a works and some land at the start (for screenshots)
    demo: bool,
    /// a new game was asked for (--new, --seed or --old-land): the island's autosave is not gone on with
    fresh: bool,
    /// the new game is on the port's island, the one every game was played on
    /// before the game's rules took the new land (`--old-land`)
    old_land: bool,
    /// the new game begins at landfall, one town in a corner (`--landfall`;
    /// docs/rules/landfall.md)
    landfall: bool,
    /// the new game's settlers by ship and ideas by sea, where asked for
    /// (`--settlers on|off`, `--ideas-by-sea on|off`; docs/rules/landfall.md,
    /// "The start's switches"); None is the start's own
    switches: isostate_core::save::Switches,
    /// a saved game to go on with
    load: Option<String>,
    /// where to save, instead of the island's autosave
    save: Option<String>,
    /// keep the game in the island's autosave though the start is not plain play
    keep: bool,
    /// a sandbox: nobody plays a family, and the island is changed by hand (sandbox.rs). Asked
    /// for with `--sandbox`; at the end of `parse` it is what opens (`sandbox_wanted`)
    sandbox: bool,
    /// the family game was asked for (`--family`): a start with somebody at the window is
    /// otherwise a sandbox
    family: bool,
    /// the last year of a new game (`--to`); None: `isostate_core::save::LATEST_YEAR`
    to: Option<i64>,
    /// start with this spot clicked
    pick: Option<(f32, f32)>,
    ground_debug: bool,
    class_map: bool,
    /// render the standard shots into this directory, then stop (shots.rs)
    shots: Option<String>,
    /// (task C, the land) the shots are the far views of the land (`shots::land_views`), not the standard twelve
    land_shots: bool,
    /// show each standard shot beside the one of the same name in this directory
    shots_compare: Option<String>,
    /// a line for the sheet of standard shots, such as the commit they were made from
    shots_note: String,
    /// compare two sets of standard shots already made, then stop
    compare_shots: Option<(String, String)>,
    /// write a contact sheet of generators (or "all", or "list" to name them), then stop (lab.rs)
    lab: Option<String>,
    /// the directory the lab's pictures go in
    out: Option<String>,
    /// the lab's pictures through the tilt-shift lens, which is otherwise off there
    lab_tilt: bool,
    /// how many times to enlarge the lab's close picture
    loupe: u32,
    /// how far the lab turns each thing about its own middle, in degrees
    lab_turn: Option<f32>,
    /// the map overlay to start with: rent, areas, reach, price (or price:coal), wages
    overlay: Option<String>,
    /// steps to play as a player would, printing what happens (see script.rs)
    script: Option<String>,
}

#[derive(Clone, Debug)]
struct ShotSpec {
    path: String,
    year: Option<i32>,
    look: Option<(f32, f32)>,
    yaw: f32,
    pitch: f32,
    dist: Option<f32>,
    /// phase of the year for this shot, overriding --season
    season: Option<f32>,
    /// which side panel to show: Some(None) closes it
    tab: Option<Option<ui::Tab>>,
    /// the map overlay for this shot, as `overlay::parse` reads it
    overlay: Option<String>,
    /// what the Markets panel shows, as `ui::markets_word` knows it: "island" or "open:GOOD"
    markets: Option<String>,
}

const USAGE: &str = "Isolated State: the game, and a viewer for recorded runs.

  isostate-viewer [RUN.bin] [options]

With nothing at all on the command line, the start page opens: the saved games to go on with,
a new island, and the command line each choice runs. Any option skips it.

With no run named, the game is a sandbox: nobody plays a family, and the island is watched
and changed by hand. With nothing else asked for, the island's session is gone on with where
it was left, behind a short card (Go on, or Begin again). --family plays the family game instead.

  --start-page         open the start page (with --size WxH, its window's size)

  --family             the family game: one family among six, its orders, its cards, the
                       ranking at the end. With nothing else asked for, the island's saved
                       family game is gone on with
  --new                a new game, whatever is saved (with nothing else asked for, the island's
                       session, or with --family its saved game, is gone on with)
  --seed N             which island (7); a new game
  --old-land           a new game on the old island, the one every game was played on before
                       the game's rules took the new land (19 November 2026). A saved game
                       says its land itself and is played on it again
  --landfall           a new game begun at landfall: one town of 5,000 settlers in a corner of
                       the island, its people answering to the free land, and settlers coming
                       by sea (docs/rules/landfall.md). A saved game says its start itself
  --settlers on|off    a new game with settlers by ship or without (on at landfall, off on
                       the settled island). A saved game says how it began
  --ideas-by-sea on|off
                       a new game whose ports learn from abroad, or not (on at landfall, off
                       on the settled island). In a game both are switches of the Rules tab
  --new-land           does nothing: every new game is on the new land. Kept so that commands
                       written before the switch still run
  --load FILE          go on with this saved game
  --save FILE          keep the game in FILE instead of the island's autosave
  --keep               keep the game in the island's autosave (a sandbox in its session) though
                       the start is not plain play (--advance, --demo, --script); with nobody
                       at the game it needs ISOSTATE_SAVES to name the folder
  --sandbox            a sandbox where the family game would open: pictures, a script and
                       --demo play the family game unless this is given. In a sandbox a tray
                       of tools under the clock changes the island by hand (money, people,
                       the harvest) or runs it on a year, ten or twenty-five. A sandbox is
                       kept in its island's session (sandbox/island-7-families-6.json in the
                       saved games' folder), never in the family's saved game; a plain start
                       goes on with it, and any other start makes the island anew and sets
                       the session before it aside (.previous.json)
  --families N         how many great families (6; from 1 to 30)
  --to YEAR            the last year a new game is played to (2050; from 1900 to 2050). Kept in
                       its save: a game gone on with is played to its own. Past 1900 the island
                       is hungry until the content after 1900 is written
  --advance N          play N years before the game opens (in the family game a trial: it is
                       kept among the trials, apart from the island's own saved game)
  --demo               the family builds a works and buys land in its first year (a trial too)
  --year Y             open on this year
  --size WxH           the size of the window or the pictures (1600x1000)
  --at X,Z             where the camera looks, in map cells
  --look FILE          a look file laid over the standard look (two numbers are taken as --at)
  --yaw D              the camera's turn, in degrees
  --pitch D            the camera's tilt, in degrees (52)
  --dist N             the camera's distance
  --no-tilt            no miniature focus
  --season F           hold the season: 0 midwinter, 0.25 spring, 0.5 summer, 0.75 autumn
  --pick X,Z           start with this spot clicked
  --overlay NAME       start with a map view: rent, areas, reach, price, price:GOOD, wages, growth, residents
                       or plain
  --describe X,Z       print what a click there would show, and stop

Pictures and checks with no window:

  --view PATH,YEAR,X,Z,YAW,PITCH,DIST[,SEASON][,WORDS]
                       a picture; give it once for each. Empty values fall back to
                       the opening view. WORDS, in any order: a panel (family, holdings,
                       markets, news, hand, rules, families or none), a map view as for
                       --overlay, and island
                       or open:GOOD for what the Markets panel shows (island:months and
                       open:GOOD:months for its charts month by month)
  --shot PATH          a picture of the opening view
  --wait S             the longest a picture waits for the renderer's shaders (45; 120 with --shots)
  --frames N           frames to let a picture settle (40)
  --ground-debug       draw the ground as flat class colours and nothing else
  --class-map          write the CPU's land class under every pixel of each view, and stop
  --script STEPS       play keys and clicks and print what happens (see src/script.rs)
  --page-shot PATH[,more][,family][,start:WORD][,settlers:off][,ideas-by-sea:off][,tip:NAME][,begin]
                       a picture of the start page, with More open, with the family game
                       chosen, with a start chosen and landfall's switches off, and then
                       Begin pressed: the game it starts takes a picture of its own
                       (PATH-island.png)

The standard shots (src/shots.rs) and the lab (src/lab.rs):

  --shots DIR          play the whole game, take the standard shots into DIR, and stop
  --land-shots DIR     the same for the far views of the land: five cameras (the whole island,
                       mid distance over lowland wood, a hill margin, a town's surroundings,
                       the snow line) in 1800 and 1900, in summer, in October and in midwinter
  --shots-compare DIR  with --shots: set each beside the shot of the same name in DIR
  --shots-note TEXT    with --shots: a line for the sheet, such as the commit
  --compare-shots A B  compare two sets of standard shots already taken, and stop
  --lab SHEET          contact sheets of the scenery's generators (a sheet, all, or list), and stop
  --out DIR            with --lab: where its pictures go
  --tilt               with --lab: through the miniature focus
  --loupe N            with --lab: enlarge the close picture N times (1 to 8)
  --turn D             with --lab: turn each thing D degrees about its middle

  --version            the game's version and the commit it was built from
  --help               this";

/// The game's version, the workspace's (Cargo.toml), as the window's title and
/// --version give it. A marked version is a tag `v` and this (docs/RELEASING.md).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// What --version prints: the version and the commit the program was built from.
fn version_line() -> String {
    format!("Isolated State {VERSION} (commit {})", note::COMMIT)
}

fn parse_args() -> Args {
    let v: Vec<String> = std::env::args().skip(1).collect();
    // asked for anywhere on the line, as --help is, and nothing else is done
    if v.iter().any(|s| s == "--version" || s == "-V") {
        println!("{}", version_line());
        std::process::exit(0);
    }
    match parse(&v) {
        Ok(a) => a,
        // asked for
        Err(e) if e.is_empty() => {
            println!("{USAGE}");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("{e}\n\n{USAGE}");
            std::process::exit(2);
        }
    }
}

/// A value on the command line as a number. One that cannot be read is
/// refused, like an option that is not one, rather than replaced by the
/// default without a word: --advance 3O would otherwise open on 1750.
fn number<T: std::str::FromStr>(opt: &str, s: &str) -> Result<T, String> {
    let s = s.trim();
    // "nan" and "inf" read as numbers, and would hang a wait or blank a season
    let endless = s.parse::<f32>().is_ok_and(|x| !x.is_finite());
    s.parse().ok().filter(|_| !endless).ok_or_else(|| format!("{opt}: {s} is not a number it can take"))
}

/// A switch on the command line: on or off.
fn on_off(opt: &str, s: &str) -> Result<bool, String> {
    match s.trim() {
        "on" => Ok(true),
        "off" => Ok(false),
        s => Err(format!("{opt} takes on or off, not {s}")),
    }
}

/// Two numbers with `sep` between them, as in --look 31.5,29.5 and --size 1600x1000.
fn pair<T: std::str::FromStr>(opt: &str, s: &str, sep: char) -> Result<(T, T), String> {
    match s.split_once(sep) {
        Some((a, b)) => Ok((number(opt, a)?, number(opt, b)?)),
        None => Err(format!("{opt}: {s} should be two numbers with {} between them", if sep == 'x' { "an x" } else { "a comma" })),
    }
}

/// A value that names something (a file, a directory, a sheet): it must be
/// there, and it is not the next option. `--load --size 800x600` would
/// otherwise take --size for the file to load.
fn named(opt: &str, s: String, what: &str) -> Result<String, String> {
    if s.trim().is_empty() {
        Err(format!("{opt} needs {what} after it"))
    } else if s.starts_with("--") {
        Err(format!("{opt} needs {what} after it, and {s} is another option: put the name between them (a name that begins with dashes is written ./{s})"))
    } else {
        Ok(s)
    }
}

/// Read the command line. An option that is not one is refused, with the
/// list of those there are, rather than dropped: a mistyped --sesaon would
/// otherwise run as if it had not been given. An empty error asks for the list.
/// A value that cannot be read is refused in the same way.
fn parse(v: &[String]) -> Result<Args, String> {
    let mut a = Args {
        run: "runs/landlords/run.bin".into(),
        year: None,
        shot: None,
        size: (1600, 1000),
        at: None,
        look_file: None,
        yaw: 0.0,
        pitch: shots::PLAY_PITCH,
        dist: None,
        frames: 40,
        wait: None,
        tilt: true,
        views: Vec::new(),
        season: None,
        describe: None,
        new: false,
        named_run: false,
        seed: 7,
        families: 6,
        advance: 0,
        demo: false,
        fresh: false,
        old_land: false,
        landfall: false,
        switches: isostate_core::save::Switches::default(),
        load: None,
        save: None,
        keep: false,
        sandbox: false,
        family: false,
        to: None,
        pick: None,
        ground_debug: false,
        class_map: false,
        shots: None,
        land_shots: false,
        shots_compare: None,
        shots_note: String::new(),
        compare_shots: None,
        lab: None,
        out: None,
        lab_tilt: false,
        loupe: 1,
        lab_turn: None,
        overlay: None,
        script: None,
    };
    let mut i = 0;
    while i < v.len() {
        let next = |i: usize| v.get(i + 1).cloned().ok_or_else(|| format!("{} needs a value after it", v[i]));
        let opt = v[i].as_str();
        match opt {
            "--year" => { a.year = Some(number(opt, &next(i)?)?); i += 1; }
            "--shot" => { a.shot = Some(named(opt, next(i)?, "the name of a picture")?); i += 1; }
            "--size" => {
                a.size = pair(opt, &next(i)?, 'x')?;
                if a.size.0 == 0 || a.size.1 == 0 {
                    return Err(format!("{opt}: a picture needs a width and a height"));
                }
                i += 1;
            }
            "--at" => { a.at = Some(pair(opt, &next(i)?, ',')?); i += 1; }
            "--look" => {
                // a look file; two numbers are the point to look at, which --look used to mean
                let s = next(i)?;
                match s.split_once(',') {
                    Some((x, z)) if x.trim().parse::<f32>().is_ok() || z.trim().parse::<f32>().is_ok() => a.at = Some(pair(opt, &s, ',')?),
                    _ => a.look_file = Some(named(opt, s, "the name of a look file")?),
                }
                i += 1;
            }
            "--yaw" => { a.yaw = number(opt, &next(i)?)?; i += 1; }
            "--pitch" => { a.pitch = number(opt, &next(i)?)?; i += 1; }
            "--dist" => { a.dist = Some(number(opt, &next(i)?)?); i += 1; }
            "--frames" => { a.frames = number(opt, &next(i)?)?; i += 1; }
            "--wait" => { a.wait = Some(number(opt, &next(i)?)?); i += 1; }
            "--no-tilt" => a.tilt = false,
            "--new" => { a.new = true; a.fresh = true; }
            "--seed" => { a.seed = number(opt, &next(i)?)?; a.fresh = true; i += 1; }
            "--old-land" => { a.old_land = true; a.new = true; a.fresh = true; }
            "--landfall" => { a.landfall = true; a.new = true; a.fresh = true; }
            "--settlers" => { a.switches.settlers = Some(on_off(opt, &next(i)?)?); a.new = true; a.fresh = true; i += 1; }
            "--ideas-by-sea" => { a.switches.ideas_by_sea = Some(on_off(opt, &next(i)?)?); a.new = true; a.fresh = true; i += 1; }
            // (the new land is every new game's since the switch: the option is kept so that old commands run)
            "--new-land" => {}
            "--load" => { a.load = Some(named(opt, next(i)?, "the name of a saved game")?); i += 1; }
            "--save" => { a.save = Some(named(opt, next(i)?, "the name of a file to keep the game in")?); i += 1; }
            "--keep" => a.keep = true,
            "--sandbox" => a.sandbox = true,
            "--family" => a.family = true,
            "--families" => {
                a.families = number(opt, &next(i)?)?;
                // refused here, before an island is made for a number no game can have
                isostate_core::save::families_allowed(a.families).map_err(|why| format!("{opt}: {why}; give a number in that range"))?;
                i += 1;
            }
            "--advance" => { a.advance = number(opt, &next(i)?)?; i += 1; }
            "--to" => {
                let to = number(opt, &next(i)?)?;
                // refused here, before an island is made, as --families is (a new game is under the game's rules)
                isostate_core::save::last_year_allowed(isostate_core::params::Rules::Game, to).map_err(|why| format!("{opt}: {why}; give a year in that range"))?;
                a.to = Some(to);
                i += 1;
            }
            "--demo" => a.demo = true,
            "--pick" => { a.pick = Some(pair(opt, &next(i)?, ',')?); i += 1; }
            "--ground-debug" => a.ground_debug = true,
            "--class-map" => a.class_map = true,
            "--overlay" => {
                let w = next(i)?;
                if !overlay::is_word(&w) {
                    return Err(format!("{opt}: {w} is not a map view: {}", overlay::VIEWS_WORDS));
                }
                a.overlay = Some(w);
                i += 1;
            }
            "--script" => {
                let s = next(i)?;
                script::check(&s).map_err(|e| format!("{opt}: {e}"))?;
                a.script = Some(s);
                i += 1;
            }
            "--season" => { a.season = Some(number(opt, &next(i)?)?); i += 1; }
            "--shots" => { a.shots = Some(named(opt, next(i)?, "a directory")?); i += 1; }
            // (task C, the land) the same machinery as --shots, with the far views of the land
            "--land-shots" => { a.shots = Some(named(opt, next(i)?, "a directory")?); a.land_shots = true; i += 1; }
            "--shots-compare" => { a.shots_compare = Some(named(opt, next(i)?, "a directory")?); i += 1; }
            "--shots-note" => {
                // a line of text and not a name, so its refusal does not offer ./ in front of it
                let s = next(i)?;
                if s.starts_with("--") {
                    return Err(format!("{opt} needs a line of text after it, and {s} is another option: put the line between them"));
                }
                a.shots_note = named(opt, s, "a line of text")?;
                i += 1;
            }
            "--compare-shots" => {
                match (v.get(i + 1), v.get(i + 2)) {
                    (Some(x), Some(y)) => a.compare_shots = Some((named(opt, x.clone(), "two directories")?, named(opt, y.clone(), "two directories")?)),
                    _ => return Err(format!("{opt} needs two directories after it")),
                }
                i += 2;
            }
            "--lab" => { a.lab = Some(named(opt, next(i)?, "a sheet (or all, or list)")?); i += 1; }
            "--out" => { a.out = Some(named(opt, next(i)?, "a directory")?); i += 1; }
            "--tilt" => a.lab_tilt = true,
            "--loupe" => { a.loupe = number::<u32>(opt, &next(i)?)?.clamp(1, 8); i += 1; }
            "--turn" => { a.lab_turn = Some(number(opt, &next(i)?)?); i += 1; }
            "--describe" => { a.describe = Some(pair(opt, &next(i)?, ',')?); i += 1; }
            "--view" => {
                // path,year,x,z,yaw,pitch,dist[,season][,words]: a number after dist is a season; a word
                // is a panel, a map overlay, or what the Markets panel shows
                let p: Vec<String> = named(opt, next(i)?, "a picture's name and its view")?.split(',').map(String::from).collect();
                // an empty value falls back; one that is not a number is refused
                let num = |j: usize| -> Result<Option<f32>, String> {
                    match p.get(j).map(|s| s.trim()) {
                        None | Some("") => Ok(None),
                        Some(s) => number(opt, s).map(Some),
                    }
                };
                let words: Vec<&String> = p.iter().skip(7).filter(|s| !s.is_empty() && s.parse::<f32>().is_err()).collect();
                // a word that is none of these is refused, and not read as no panel: a mistyped map
                // view would otherwise be taken of the plain map
                let known = |s: &str| overlay::is_word(s) || ui::markets_word(s) || ui::Tab::parse(s).is_some() || s.eq_ignore_ascii_case("none");
                if let Some(w) = words.iter().find(|s| !known(s)) {
                    return Err(format!("{opt}: {w} is not a word a view takes: use a panel (family, holdings, markets, news, hand, rules, families or none), a map view as for --overlay, or island or open:GOOD for the Markets panel"));
                }
                a.views.push(ShotSpec {
                    path: p.first().cloned().unwrap_or_else(|| "shot.png".into()),
                    year: num(1)?.map(|y| y as i32),
                    look: match (num(2)?, num(3)?) { (Some(x), Some(z)) => Some((x, z)), _ => None },
                    yaw: num(4)?.unwrap_or(0.0),
                    pitch: num(5)?.unwrap_or(shots::PLAY_PITCH),
                    dist: num(6)?,
                    season: p.iter().skip(7).find_map(|s| s.parse::<f32>().ok()),
                    tab: words.iter().find(|s| !overlay::is_word(s) && !ui::markets_word(s)).map(|s| ui::Tab::parse(s)),
                    overlay: words.iter().find(|s| overlay::is_word(s)).map(|s| s.to_string()),
                    markets: words.iter().find(|s| ui::markets_word(s)).map(|s| s.to_string()),
                });
                i += 1;
            }
            "--help" | "-h" => return Err(String::new()),
            s if !s.starts_with('-') => { a.run = s.to_string(); a.named_run = true; }
            s => return Err(format!("{s} is not an option")),
        }
        i += 1;
    }
    if !a.named_run || a.load.is_some() {
        a.new = true;
    }
    if a.family && a.sandbox {
        return Err("--family plays the family game and --sandbox opens a game nobody plays a family in: give one of them, not both".into());
    }
    if a.sandbox && !a.new {
        eprintln!("--sandbox changes a game by hand, and {} is a recorded history with no simulation behind it: it is opened as it was recorded", a.run);
    }
    if a.family && !a.new {
        eprintln!("--family plays a game, and {} is a recorded history with no simulation behind it: it is opened as it was recorded", a.run);
    }
    // a saved game says itself which land it is played on; --old-land is for a game that is begun
    if let (true, Some(file)) = (a.old_land, &a.load) {
        return Err(format!("--old-land starts a new game, and --load goes on with one: a saved game is played again on the land it says without being told, so leave --old-land out to go on with {file}"));
    }
    if let (true, Some(file)) = (a.landfall, &a.load) {
        return Err(format!("--landfall starts a new game, and --load goes on with one: a saved game is played again from the start it says without being told, so leave --landfall out to go on with {file}"));
    }
    if let (false, Some(file)) = (a.switches.plain(), &a.load) {
        return Err(format!("--settlers and --ideas-by-sea begin a new game, and --load goes on with one: a saved game is played again as it began without being told, and in a game the two are switches of the Rules tab, so leave them out to go on with {file}"));
    }
    // options that only mean something beside another say so, rather than being passed over in silence
    if a.lab.is_none() && (a.out.is_some() || a.lab_tilt || a.loupe != 1 || a.lab_turn.is_some()) {
        eprintln!("--out, --tilt, --turn and --loupe go with --lab and do nothing without it");
    }
    if a.script.is_some() && (a.shots.is_some() || a.shot.is_some() || !a.views.is_empty()) {
        // a scripted run takes its own pictures, where its steps say
        eprintln!("--script takes pictures with its shot:PATH step: --view, --shot and --shots are passed over beside it");
        a.shots = None;
    }
    if a.shots.is_none() && (a.shots_compare.is_some() || !a.shots_note.is_empty()) {
        eprintln!("--shots-compare and --shots-note go with --shots and do nothing without it (two sets already shot are compared with --compare-shots A B)");
    }
    if a.shots.is_some() {
        // the standard shots run from the first year to the last, so play the whole game first
        a.advance = usize::MAX - 1;
        // and that game ends in 1900, so that the standard pictures and their time are as they were
        if a.to.is_some_and(|y| y != isostate_core::save::content_last_year(isostate_core::params::Rules::Game)) {
            eprintln!("--shots plays a game to 1900 for the standard pictures: --to is passed over beside it");
        }
        a.to = Some(isostate_core::save::content_last_year(isostate_core::params::Rules::Game));
    }
    // a last year is chosen when a game is begun: a save keeps its own, and a recorded run is what it is
    if a.to.is_some() && a.shots.is_none() && (a.load.is_some() || !a.new) {
        eprintln!("--to is for a new game: {} keeps the last year it has", a.load.as_deref().unwrap_or(&a.run));
    }
    // what was asked for is settled: from here `a.sandbox` is what opens
    a.sandbox = sandbox_wanted(&a);
    Ok(a)
}

/// Which kind of game opens (docs/designs/sandbox.md, section 6.4). A live
/// game with somebody at the window is a sandbox unless `--family` asks for
/// the family game. One that nobody is at (it takes pictures, describes a
/// spot, writes a class map or plays a script) and one that plays the demo's
/// orders, which are the family's, are the family game unless `--sandbox`
/// asks for a sandbox: the standard pictures and the scripted checks stand as
/// they were. A recorded run is neither.
fn sandbox_wanted(a: &Args) -> bool {
    if !a.new {
        return false;
    }
    if a.sandbox || a.family {
        return a.sandbox;
    }
    let nobody_at_it = a.shots.is_some() || a.shot.is_some() || !a.views.is_empty() || a.describe.is_some() || a.class_map || a.script.is_some();
    !nobody_at_it && !a.demo
}

/// The game the command line asks for, as `live::start` is to begin it or go
/// on with it. Whether a sandbox opens is `a.sandbox`, settled in `parse`; the
/// Opening then says the rest (`plain`, `keeping`): a plain start goes on with
/// the island's sandbox session (`saves::session`), any other sandbox start
/// makes the island anew and keeps it there, and `--family` alone goes on with
/// the island's autosave.
fn opening(a: &Args) -> live::Opening {
    // the standard shots are pictures too
    let pictures = !a.views.is_empty() || a.shot.is_some() || a.describe.is_some() || a.class_map || a.shots.is_some();
    live::Opening {
        seed: a.seed,
        families: a.families,
        advance: a.advance,
        demo: a.demo,
        load: a.load.clone(),
        save: a.save.clone(),
        fresh: a.fresh,
        // a scripted run is a check, not the player's game: it goes on with an autosave only
        // when ISOSTATE_SAVES names the folder to look in
        shots: pictures || (a.script.is_some() && std::env::var_os("ISOSTATE_SAVES").is_none_or(|v| v.is_empty())),
        // and, like a run that takes pictures, it writes no game unless --save names the place
        unattended: pictures || a.script.is_some(),
        restarted: std::env::var_os("ISOSTATE_RESTARTED").is_some_and(|v| !v.is_empty()),
        keep: a.keep,
        old_land: a.old_land,
        landfall: a.landfall,
        switches: a.switches,
        sandbox: a.sandbox,
        // every new game runs to 2050 unless --to says otherwise (and --shots plays to 1900)
        to: Some(a.to.unwrap_or(isostate_core::save::LATEST_YEAR)),
    }
}

// ------------------------------------------------------------------ resources and markers

/// The run and what is made from it once. Shared, so a year's scenery can be
/// built on another thread from the same data; a live game changes the run
/// and the plots between years, copying them only if a build still holds them.
#[derive(Resource)]
struct Data {
    r: Arc<Replay>,
    g: Arc<Ground>,
    slots: Arc<scatter::Slots>,
    cen: Arc<Vec<(f32, f32)>>,
    /// the years are being simulated as we watch, not read from a file
    live: bool,
    /// the year that came last on its way into the run off the main thread, with its works'
    /// ground (`game::pump`): until it is in, no scenery is built and no next year is read
    arriving: Option<bevy::tasks::Task<game::Arrived>>,
}

impl Data {
    /// The last year there is anything to show for.
    fn latest(&self) -> usize {
        self.r.filled.clamp(1, self.r.ny) - 1
    }
}

#[derive(Resource)]
struct Handles {
    /// the year's land mix per cell, read by the ground shader
    year_cells: Handle<Image>,
    /// (task C, the land) the year's wood and heath as smooth fields, read by the ground shader
    year_field: Handle<Image>,
    /// a year's meshes, one to each of the table's list (`year::list`)
    meshes: Vec<Handle<Mesh>>,
    /// the entities that draw them, and their pieces (`handover::feed`)
    drawers: handover::Drawers,
}

#[derive(Resource)]
struct Timeline {
    t: f32,
    built: Option<usize>,
    playing: bool,
    speed: f32,
}

#[derive(Resource)]
struct Orbit {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    dist: f32,
}

#[derive(Resource)]
struct ShotPlan {
    views: Vec<ShotSpec>,
    index: usize,
    image: Handle<Image>,
    frames: u32,
    countdown: u32,
    capturing: u32,
    /// the longest a picture waits for the renderer's shaders, in seconds (`--wait`)
    wait: f32,
    default_dist: f32,
    default_look: (f32, f32),
    default_season: Option<f32>,
}

/// A piece of news shown as it happens. It carries its own words: the list
/// of news is sorted again whenever a live game adds to it, so a place in
/// that list does not stay the same item.
#[derive(Clone, Debug, PartialEq)]
struct Toast {
    kind: String,
    title: String,
    /// what stands over it beside its kind: the month it is dated in
    when: String,
    /// when it went up, in seconds since the app started
    since: f32,
}

#[derive(Resource, Default)]
struct Toasts {
    shown: Vec<Toast>,
    /// where the clock stood, as the news is dated, when the news was last looked through
    last: Option<f32>,
    /// how many items of news there were then: those that have come since are new
    known: usize,
    /// something was put up from elsewhere, and the box is to be drawn again
    fresh: bool,
}

impl Toasts {
    /// Put up a line that is not news of the island: what became of the player's orders.
    fn say(&mut self, kind: &str, title: &str, when: &str, now: f32) {
        self.shown.push(Toast { kind: kind.to_string(), title: title.to_string(), when: when.to_string(), since: now });
        self.fresh = true;
    }
}

/// The major news to go up as toasts when the clock has moved from `from` to
/// `to` (both as the news is dated, in years): what is dated in between, each
/// item in its month; and what has only now come from the simulation (`known`
/// is how many items there were before) though its date is already past, as
/// most of a year's news does at the year's end, if it is no more than a year old.
fn news_toasts(events: &[replay::Event], from: f32, to: f32, known: usize, now: f32) -> Vec<Toast> {
    events
        .iter()
        .filter(|e| e.major() && e.year <= to && (e.year > from || (e.seq >= known && e.year > to - 1.0)))
        .map(|e| Toast { kind: e.kind.clone(), title: e.title.clone(), when: clock::news_month(e.year), since: now })
        .collect()
}

#[derive(Resource)]
struct UiFont(Handle<Font>);

#[derive(Resource)]
struct Tilt(bool);

/// The leaves of one group of the broadleaf trees, shown while that group is
/// in leaf, and its bare limbs, shown while it is not.
// (each is Clone, so that an entity that draws a piece of a year's mesh is made as a twin of the
// first: `handover::feed`)
#[derive(Component, Clone)]
struct Leaves(usize);

#[derive(Component, Clone)]
struct BareLimbs(usize);

/// What is green all year of the trees and bushes: conifers, gorse, yews.
#[derive(Component, Clone)]
struct Evergreen;

/// Materials that change with the seasons.
#[derive(Resource)]
struct SeasonMats {
    terrain: Handle<terrain::GroundMaterial>,
    trees: Handle<StandardMaterial>,
    rivers: Handle<StandardMaterial>,
    /// what houses and works are drawn in, and the yards and ponds that lie flat round
    /// them: winter lies on both (`built.rs`)
    built: [Handle<built::BuiltMaterial>; 2],
    smoke: Handle<StandardMaterial>,
    /// fixed phase of the year for stills (0 = midwinter, 0.5 = midsummer)
    forced: Option<f32>,
}

/// Draw the ground as flat class colours with nothing on it (`--ground-debug`).
#[derive(Resource)]
struct GroundDebug(bool);

#[derive(Component)]
struct GroundMesh;

#[derive(Component)]
struct Sun;

#[derive(Component)]
struct MainCam;
#[derive(Component)]
struct ToastBox;
#[derive(Component)]
struct TownLabel(usize);
/// The name on a town's label, which a click picks the town by.
#[derive(Component)]
struct LabelPill(usize);
#[derive(Component)]
struct InspectPanel;
/// The lines of the inspector under its title, which scroll when they are more than its room.
#[derive(Component)]
pub struct InspectBody;

#[derive(Resource)]
struct TrainAssets {
    mesh: Handle<Mesh>,
    mat: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
struct Rails(Vec<trains::TrainPath>);

#[derive(Component)]
struct Train {
    path: usize,
    s: f32,
    dir: f32,
    speed: f32,
}
#[derive(Component)]
struct InspectText;
#[derive(Component)]
struct InspectTitle;
/// Where the game puts the orders the player can give about the picked thing.
#[derive(Component)]
struct InspectActions;

/// What the player clicked on last.
#[derive(Resource, Default)]
struct Picked {
    what: Option<Pick>,
    press: Option<Vec2>,
    moved: f32,
    /// raised by one at every click that sets `what`, so that a tool in hand
    /// (sandbox.rs) can tell a click from a pick that stays as it was
    stamp: u32,
    /// where on the ground the click that set `what` fell, if it was a click
    /// on the map: a tool that takes a point (sandbox.rs) takes it there,
    /// whatever thing stands on it
    at: Option<(f32, f32)>,
}

/// What a click picks (pick.rs): a thing, never a square.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Pick {
    Firm(usize),
    Town(usize),
    /// a quarry, a clay pit or a mine at work, by its pit head's cell and its kind (`replay::Working`)
    Pit { cell: usize, class: u8 },
    /// a spot in open country, which stands for the country of its market town; `cell` is the
    /// simulation's 2 km cell under it, which is what an edit or a question to the simulation names
    Place { x: f32, z: f32, cell: usize },
}

static FREDOKA: &[u8] = include_bytes!("../assets/fonts/Fredoka.ttf");

fn main() {
    let t_start = std::time::Instant::now();
    // nothing on the command line: the start page, which starts the program again with what is chosen
    match start_page::asked() {
        Ok(Some(asked)) => std::process::exit(start_page::run(asked, t_start)),
        Ok(None) => {}
        Err(e) => {
            eprintln!("{e}\n\n{USAGE}");
            std::process::exit(2);
        }
    }
    let args = parse_args();
    if let Some((a, b)) = &args.compare_shots {
        // no island and no window: two sets of pictures, compared
        let same = shots::compare_sets(a, b);
        std::process::exit(if same { 0 } else { 1 });
    }
    if let Ok(list) = std::env::var("ISOSTATE_COMPARE_CLASSES") {
        // no island and no window: --ground-debug shots against their --class-map
        let mut same = true;
        for shot in list.split(';').filter(|s| !s.is_empty()) {
            match shots::compare_classes(std::path::Path::new(shot)) {
                Ok((line, ok)) => {
                    eprintln!("{line}");
                    same &= ok;
                }
                Err(e) => {
                    eprintln!("{e}");
                    same = false;
                }
            }
        }
        std::process::exit(if same { 0 } else { 1 });
    }
    // the look first, so a slip in a look file is told at once
    let look = match &args.look_file {
        Some(path) => match Look::with_file(path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
        None => Look::standard(),
    };
    let look = Arc::new(look);
    if let Some(sheet) = args.lab.clone() {
        // no island and no game: contact sheets of the generators
        let code = lab::run(lab::LabArgs { sheet, out: args.out.clone(), size: args.size, season: args.season, tilt: args.lab_tilt, loupe: args.loupe, turn: args.lab_turn }, look);
        std::process::exit(code);
    }
    // a map view whose good a game will not have is refused before its island is made: the
    // goods are the rules' own (those of a saved game, or the game's for a new one). A recorded
    // run's goods are known once it is read, which is quick, and are checked then.
    if args.new {
        let rules = args.load.as_ref().and_then(|p| isostate_core::save::Save::from_text(&std::fs::read_to_string(p).ok()?).ok()).map_or(isostate_core::params::Rules::Game, |s| s.played_under());
        // (with the production tree laid as the rules lay it: town gas and beer are the tree's goods)
        let c = isostate_core::tree::content(&rules.params(), rules.content()).unwrap_or_else(|_| rules.content());
        let goods: Vec<replay::Good> = c.good_name.iter().zip(&c.good_label).map(|(name, label)| replay::Good { name: name.clone(), label: label.clone(), kind: None, weight: None, price: None, begins: None }).collect();
        refuse_map_views(&args, |w| overlay::complaint_among(w, &goods));
    }
    let t0 = std::time::Instant::now();
    let (r, live, families) = if args.new {
        let opening = opening(&args);
        // a check may write an autosave, but never among the player's own saved games
        if let Some(why) = opening.keep_refused(std::env::var_os("ISOSTATE_SAVES").is_some_and(|v| !v.is_empty())) {
            eprintln!("the game could not start: {why}");
            std::process::exit(1);
        }
        match live::start(&opening) {
            Ok((r, l, f)) => (r, Some(l), Some(f)),
            Err(e) => {
                eprintln!("the game could not start: {e}");
                std::process::exit(1);
            }
        }
    } else {
        if args.save.is_some() {
            eprintln!("--save is for a game: {} is a recorded run, and nothing is saved", args.run);
        }
        match replay::load(&args.run) {
            Ok(r) => (r, None, None),
            Err(e) => {
                eprintln!("{e}");
                // how to make a run is the answer to a file that is not there, not to one that is damaged
                if !std::path::Path::new(&args.run).exists() {
                    eprintln!("Make a run first, for example:\n  cargo run --release -p isostate-core -- run --seed 7 --out runs/rust7\nor start a new game with --new");
                }
                std::process::exit(1);
            }
        }
    };
    let g = world::build_ground(&r);
    let slots = scatter::make_slots(&r, &g);
    let cen = network::centres(&r);
    // a measuring tool asked for by an environment variable (newland.rs, plan.rs, rails.rs) does its work, and that is all
    if newland::maybe_report(&r, &g, &cen) || rivers::maybe_report(&r, &g.rivers) || scatter::measure(&r, &g, &slots, &cen) {
        std::process::exit(0);
    }
    eprintln!("prepared {} in {:.1}s", if args.new { "the game" } else { args.run.as_str() }, t0.elapsed().as_secs_f32());

    // (a game whose last year is played has none left to take any time)
    let left = r.ny.saturating_sub(r.filled);
    if live.is_some() && left > 0 {
        // how long the game's years take at each of the clock's speeds, if nothing stops it
        let speeds: Vec<String> = clock::SPEEDS.iter().map(|s| format!("{} ({}; {})", s.name, s.pace, clock::length_words(clock::crossing(left, s.rate)))).collect();
        eprintln!("the clock's speeds, with how long the {left} years to {} would take: {}", r.y0 + r.ny as i32 - 1, speeds.join(", "));
    }
    if let Some(me) = families.as_ref().and_then(|f| f.player()) {
        let at = r.header.towns.get(me.seat).map(|t| cen[t.node]).unwrap_or((0.0, 0.0));
        eprintln!("you are the {} family of {} ({:.1},{:.1}), with {} in hand", me.name, me.seat_name, at.0, at.1, ui::money(me.cash));
        for fr in families.as_ref().map(|f| f.families.as_slice()).unwrap_or(&[]) {
            if fr.rep.index != me.rep.index {
                let at = r.header.towns.get(fr.seat).map(|t| cen[t.node]).unwrap_or((0.0, 0.0));
                eprintln!("  the {} family of {} ({:.1},{:.1})", fr.name, fr.seat_name, at.0, at.1);
            }
        }
        for firm in me.rep.firms.iter().filter_map(|&f| r.header.firms.get(f)) {
            eprintln!("  {} at {},{}", firm.name, firm.cell % r.w, firm.cell / r.w);
        }
    }
    // a recorded run's map views, now its goods are known (a game's were checked before its island
    // was made, and are checked again against the goods it has)
    refuse_map_views(&args, |w| overlay::complaint(w, &r));
    let latest = r.filled.clamp(1, r.ny) - 1;
    let k0 = match args.year {
        Some(y) => ((y - r.y0).max(0) as usize).min(latest),
        None => latest,
    };
    if let Some((x, z)) = args.describe {
        // print what a click at this spot would show, then stop
        match pick::choose(&r, &g, &slots, &cen, &pick::near_of(&r, &slots, k0), x, z, k0) {
            Some(p) => {
                println!("{}", describe(&r, &cen, p, k0, families.as_ref().map_or(&[][..], |f| f.method_group.as_slice())));
                // with --overlay, what that overlay says of the spot, as the inspector would
                if let Some(overlay::Choice { kind: Some(kind), which, .. }) = args.overlay.as_deref().and_then(|w| overlay::parse(w, &r)) {
                    if let Some(line) = overlay::said(&r, k0, pick_cell(&r, p), kind, which.unwrap_or(0), None) {
                        println!("\n{line}");
                    }
                }
            }
            None => println!("nothing there"),
        }
        return;
    }
    // the island's land, for the opening view
    let (mut x0, mut z0, mut x1, mut z1) = (f32::MAX, f32::MAX, 0f32, 0f32);
    for c in 0..r.n {
        if r.terrain[c] > 0 {
            let (x, z) = ((c % r.w) as f32, (c / r.w) as f32);
            x0 = x0.min(x);
            z0 = z0.min(z);
            x1 = x1.max(x + 1.0);
            z1 = z1.max(z + 1.0);
        }
    }
    // a game opens over the family's home town; a replay over the whole island
    let home = families.as_ref().and_then(|f| f.player()).and_then(|p| r.header.towns.get(p.seat)).map(|t| cen[t.node]);
    let island = (((x0 + x1) * 0.5, (z0 + z1) * 0.5 + 4.0), (x1 - x0).max((z1 - z0) * 1.5) * 1.05);
    let look_at = args.at.or(home.map(|(x, z)| (x, z + 1.0))).unwrap_or(island.0);
    let dist = args.dist.unwrap_or(if home.is_some() { shots::PLAY_DIST } else { island.1 });
    let orbit = Orbit {
        target: Vec3::new(look_at.0, world::height_at(&g, look_at.0, look_at.1).max(0.0), look_at.1),
        yaw: args.yaw.to_radians(),
        pitch: args.pitch.to_radians(),
        dist,
    };

    let picked = Picked {
        what: args.pick.and_then(|(x, z)| pick::choose(&r, &g, &slots, &cen, &pick::near_of(&r, &slots, k0), x, z, k0)),
        at: args.pick,
        ..default()
    };

    // a game left in the middle of a year is gone on with there, unless a year was asked for
    let months = live.as_ref().map_or(0, |l| l.months);
    let t0 = if args.year.is_none() && k0 == latest { clock::at(k0, months.min(11)) } else { k0 as f32 };
    // a game starts at a month a second; a recorded history, which is watched and not played, at a year a second
    let speed0 = if live.is_some() { clock::START_SPEED } else { clock::SPEEDS[3].rate };
    let mut app = App::new();
    let mut views = args.views.clone();
    if let Some(path) = args.shot.clone() {
        views.insert(0, ShotSpec { path, year: args.year, look: args.at, yaw: args.yaw, pitch: args.pitch, dist: args.dist, season: None, tab: None, overlay: None, markets: None });
    }
    // a picture (or a class map) whose directory is not there would be made and then not written
    for v in &views {
        if let Some(parent) = std::path::Path::new(&v.path).parent().filter(|p| !p.as_os_str().is_empty()) {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("cannot make {}: {e}", parent.display());
                std::process::exit(1);
            }
        }
    }
    if args.class_map {
        // the CPU's answer for each view, to compare with a --ground-debug shot
        for v in &views {
            write_class_map(&r, &g, v, args.size, look_at, dist);
        }
        return;
    }
    // the standard shots: the views in the order of their years, so the years are stepped through once
    let mut sheet = None;
    if let Some(dir) = &args.shots {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("cannot make {dir}: {e}");
            std::process::exit(1);
        }
        let seat = families.as_ref().and_then(|f| f.player()).map(|p| p.seat);
        let standard = if args.land_shots {
            shots::land_views(&r, &g, &cen, seat, shots::island_view(&r, args.size), dir)
        } else {
            shots::standard_views(&r, &cen, &slots, seat, shots::island_view(&r, args.size), dir)
        };
        // pictures of an earlier set in this directory would be taken for this run's
        shots::clear_pictures(&standard);
        views = standard.iter().map(|s| s.spec.clone()).collect();
        views.sort_by_key(|v| v.year);
        let mut what = if args.new {
            let switches: String = saves::switch_words(args.switches).iter().map(|w| format!(", {w}")).collect();
            format!("Island {} with {}{}{}{switches}, {} by {} pixels", args.seed, families_words(args.families), if args.old_land { ", on the old island" } else { "" }, if args.landfall { ", from landfall" } else { "" }, args.size.0, args.size.1)
        } else {
            format!("{}, {} by {} pixels", args.run, args.size.0, args.size.1)
        };
        if let Some(path) = &args.look_file {
            what.push_str(&format!(", in the look of {path}"));
        }
        sheet = Some(shots::Sheet {
            dir: dir.clone(),
            compare: args.shots_compare.clone(),
            shots: standard.into_iter().map(|s| (s.name, s.caption)).collect(),
            about: shots::About { what, note: args.shots_note.clone() },
        });
    }
    let headless = !views.is_empty() || args.script.is_some();
    // what a note typed at the window says of the game (note.rs)
    let about = note_about(&args, &r, families.as_ref(), live.is_some(), headless);
    // a game opens in the season of its date; pictures, a recorded run, and a game opened where
    // the bar shows a year and no month (on a year gone by, or with its last year played) in high summer
    let dated0 = live.is_some() && views.is_empty() && k0 == latest && r.filled < r.ny;
    let season0 = if dated0 { SeasonClock { phase: t0.fract(), follows: true, coarse: false } } else { SeasonClock::default() };
    if headless {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, ..default() })
                .disable::<WinitPlugin>(),
        )
        .add_plugins(ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 30.0)));
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("Isolated State {VERSION}"),
                resolution: WindowResolution::new(args.size.0, args.size.1),
                // the narrowest window the panels are laid out for: under 1280 the bar across the top has
                // no room for its tabs, and under 720 the side panel and a decision's options grow short
                resize_constraints: bevy::window::WindowResizeConstraints { min_width: MIN_WINDOW.0, min_height: MIN_WINDOW.1, ..default() },
                ..default()
            }),
            ..default()
        }));
    }
    terrain::register(&mut app);
    built::register(&mut app);
    let mut start_overlay = overlay::MapOverlay::new(None, 0);
    if let Some(c) = args.overlay.as_deref().and_then(|w| overlay::parse(w, &r)) {
        start_overlay.choose(&c);
    }
    app.insert_resource(start_overlay);
    app.insert_resource(ClearColor(look.sky.background.color()))
        .insert_resource(GlobalAmbientLight { color: look.sky.light.color(), brightness: look.sky.brightness, ..default() })
        .insert_resource(Data { r: Arc::new(r), g: Arc::new(g), slots: Arc::new(slots), cen: Arc::new(cen), live: live.is_some(), arriving: None })
        .init_resource::<Builds>()
        .init_resource::<Shown>()
        .insert_resource(Timeline { t: t0, built: None, playing: false, speed: speed0 })
        .insert_resource(orbit)
        .insert_resource(Tilt(args.tilt && look.tilt_shift.on))
        .insert_resource(TheLook(look))
        .insert_resource(ForcedSeason(args.season))
        .insert_resource(season0)
        .init_resource::<live::Months>()
        .insert_resource(GroundDebug(args.ground_debug))
        .init_resource::<Toasts>()
        .insert_resource(picked)
        .init_resource::<Rails>()
        .add_plugins(ui::plugin)
        .add_plugins(game::plugin)
        .add_plugins(sandbox::plugin)
        .add_plugins(hand_tab::plugin)
        .add_plugins(inspector::plugin)
        .add_plugins(tips::plugin)
        .add_plugins(firm_card::plugin)
        .add_plugins(town_ideas::plugin)
        .add_plugins(town_flow::plugin)
        .add_plugins(ideas_plate::plugin)
        .add_plugins(rulebook::plugin)
        .add_plugins(take_back::plugin)
        .add_plugins(ground::plugin)
        .add_plugins(overlay::plugin)
        .add_plugins(traffic::plugin)
        .add_plugins(cursor::plugin)
        .add_plugins(pick::plugin)
        .add_plugins(boats::plugin)
        .add_plugins(carts::plugin)
        .add_plugins(note::plugin)
        .add_plugins(backlog_seen::plugin)
        .add_plugins(whats_new::plugin)
        .add_plugins(start_page::game_plugin)
        .insert_resource(about)
        .add_systems(Startup, (setup_world, setup_ui, icons::setup).chain());
    year_systems(&mut app);
    // the seasons' clock is read by the ground, the trees, the sun and the date alike
    app.add_systems(Update, season_clock.after(advance_time).before(seasons));
    // which of the trees' meshes show is settled in one place, by the season and the map view together
    app.add_systems(Update, show_trees.after(seasons));
    // an edit taken back: what came after it is forgotten in the frame its answer is read, before the
    // year's scenery is looked at and before the square or the panels are asked about again
    app.add_systems(Update, back::taken_back.after(game::pump).before(rebuild_year).before(game::ask_site).run_if(resource_exists::<live::Live>));
    if let Some(l) = live {
        app.insert_resource(l);
        // a game changed by hand: the tray of tools, and the family game's cards kept away
        if args.sandbox {
            app.insert_resource(sandbox::Sandbox::default());
        }
    }
    if let Some(f) = families {
        app.insert_resource(f);
    }
    if args.ground_debug {
        app.add_systems(Update, hide_all_but_ground.after(rebuild_year));
    }
    // a year's meshes go to the renderer under a budget (handover.rs); where the run is there
    // to take stills the whole year goes at once, as it always did, so the pictures are the same
    let stills = args.script.is_none() && headless;
    app.insert_resource(handover::HandOver::new(if stills { handover::Pace::Whole } else { handover::Pace::from_env() }));
    if let Some(s) = &args.script {
        if let Err(e) = script::add(&mut app, s, args.size) {
            eprintln!("--script: {e}");
            std::process::exit(2);
        }
        // a sandbox's inspector prints each new answer the simulation gives it
        app.insert_resource(inspector::Answers::printing());
    } else if headless {
        // a picture waits for what it needs, its year's scenery and the renderer's shaders, and then
        // some frames for it to settle (take_shot); for the shaders no longer than this, which a busy
        // machine taking the standard shots has been given more of (`--wait` says how long)
        let wait = args.wait.unwrap_or(if args.shots.is_some() { 120.0 } else { 45.0 });
        app.insert_resource(ShotPlanArgs { views, size: args.size, frames: args.frames, wait, default_dist: dist, default_look: look_at });
        app.add_systems(Update, take_shot.after(toasts));
        let waiting = PipelinesWaiting::default();
        app.insert_resource(waiting.clone());
        if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render.insert_resource(waiting).add_systems(bevy::render::Render, count_waiting_pipelines.in_set(bevy::render::RenderSystems::Cleanup));
        }
    }
    if let Some(sheet) = sheet {
        // the standard shots must come out the same every run: nothing moves while they are taken
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(std::time::Duration::ZERO));
        app.insert_resource(sheet);
    }
    // the frame recorder (frames.rs): asked for by ISOSTATE_FRAMES, and kept as a meter with
    // nothing written where a scripted run or ISOSTATE_TIMING will ask what the frames cost
    frames::add(&mut app, args.script.is_some() || timing());
    app.run();
}

/// What a note typed at the window says of the game it was typed in, and
/// where notes go (note.rs). A run nobody is at keeps a note only where
/// `ISOSTATE_SAVES` names the folder, as `--keep` does, so a check never
/// writes among the notes of whoever plays.
fn note_about(args: &Args, r: &Replay, families: Option<&family::FamilyView>, live: bool, headless: bool) -> note::About {
    // (a live game's land is the run's own; a recording's is in its header)
    let land = if r.land.is_some() || r.header.land.is_some() { "on the new land" } else { "on the old island" };
    let (island, kept_in) = if live {
        let rules = args.load.as_ref().and_then(|p| isostate_core::save::Save::from_text(&std::fs::read_to_string(p).ok()?).ok()).map_or(isostate_core::params::Rules::Game, |s| s.played_under());
        let rules = match rules {
            isostate_core::params::Rules::Game => "the game's rules",
            isostate_core::params::Rules::Port => "the port's rules",
        };
        let n = families.map_or(args.families, |f| f.families.len());
        let kept = match opening(args).keeping() {
            live::Keeping::Named(p) => format!("kept in {}", p.display()),
            live::Keeping::Autosave => format!("kept in {}", saves::autosave(r.header.meta.seed, n).display()),
            live::Keeping::Trial(_) => format!("kept in {}", saves::trial(r.header.meta.seed, n).display()),
            live::Keeping::Session => format!("kept in {}", saves::session(r.header.meta.seed, n).display()),
            live::Keeping::Nowhere => "not kept, since nobody is at it (a script or pictures)".to_string(),
        };
        (format!("Island {} with {}, under {rules}, {land}", r.header.meta.seed, families_words(n)), kept)
    } else {
        (format!("Island {} as recorded, {land}", r.header.meta.seed), format!("read from {}", args.run))
    };
    let named = std::env::var_os("ISOSTATE_SAVES").is_some_and(|v| !v.is_empty());
    let folder = if headless && !named { Err("nobody is at this game, so a note is kept only where ISOSTATE_SAVES names the folder".to_string()) } else { Ok(saves::notes()) };
    note::About { island, kept_in, folder }
}

/// `ISOSTATE_TIMING` is set: say on standard error what the heavy steps cost.
fn timing() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("ISOSTATE_TIMING").is_some())
}

/// Stop on a map view that cannot be shown, saying which option it came from,
/// so that a mistyped picture is not taken of the plain map. A word that is no
/// map view at all was refused with the options; whether a good is one the run
/// has is for `complaint` to say, given the run's goods. The good a view's
/// Markets panel opens on (open:GOOD) is asked about as its price.
fn refuse_map_views(args: &Args, complaint: impl Fn(&str) -> Option<String>) {
    let opened = |w: &String| {
        let lower = w.to_ascii_lowercase();
        let end = if lower.ends_with(":months") { w.len() - ":months".len() } else { w.len() };
        lower[..end].starts_with("open:").then(|| format!("price:{}", &w["open:".len()..end]))
    };
    let words = args.overlay.iter().map(|w| ("--overlay", w.clone()))
        .chain(args.views.iter().filter_map(|v| v.overlay.as_ref().map(|w| ("--view", w.clone()))))
        .chain(args.views.iter().filter_map(|v| v.markets.as_ref().and_then(opened).map(|w| ("--view", w))));
    for (opt, w) in words {
        if let Some(why) = complaint(&w) {
            eprintln!("{opt}: {why}");
            std::process::exit(2);
        }
    }
}

#[derive(Resource)]
struct ForcedSeason(Option<f32>);

/// How many shader pipelines the renderer has asked for and not yet got.
/// While one is still being put together, whatever it draws (the ground, say)
/// is simply left out of the picture, so screenshots wait for none to be.
#[derive(Resource, Clone, Default)]
struct PipelinesWaiting(Arc<std::sync::atomic::AtomicUsize>);

/// Runs in the renderer's own world at the end of every frame it draws.
fn count_waiting_pipelines(cache: Option<Res<bevy::render::render_resource::PipelineCache>>, waiting: Res<PipelinesWaiting>) {
    if let Some(cache) = cache {
        waiting.0.store(cache.waiting_pipelines().count(), std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Resource)]
struct ShotPlanArgs {
    views: Vec<ShotSpec>,
    size: (u32, u32),
    frames: u32,
    wait: f32,
    default_dist: f32,
    default_look: (f32, f32),
}

/// The game's own systems in `Update`, in the order they run. A year comes
/// from the simulation in `game::pump`, which writes it into the run, and goes
/// on screen through `rebuild_year`, which queues its build, and
/// `handover::feed`, which lets the queue go a mesh at a time. The pump runs
/// first, so that the frame a year arrives in is the frame `rebuild_year`
/// notices it in and holds the hand-over back (`HandOver::hold`), and so that
/// a year cannot be written in between the feed's last mesh and the next look.
/// That order is set where the pump is added (`game::plugin`); before, it held
/// only by way of `game::next_clicks`, which comes after the pump in its chain
/// and runs before `time_input`. A test holds it.
fn year_systems(app: &mut App) {
    app.add_systems(Update, (ui::ui_hover, time_input, advance_time, rebuild_year, handover::feed, move_trains, seasons, camera_input, apply_camera, labels, toasts, pick, label_clicks, inspect_panel).chain());
}

// ------------------------------------------------------------------ world setup

fn vertex_coloured(materials: &mut Assets<StandardMaterial>, lk: &Look, rough: f32) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial { base_color: Color::WHITE, perceptual_roughness: rough, reflectance: lk.surfaces.painted_reflectance, ..default() })
}

fn placeholder_mesh() -> Mesh {
    let mut b = meshkit::Builder::default();
    b.tri(Vec3::new(0.0, -50.0, 0.0), Vec3::new(0.0, -50.0, 0.01), Vec3::new(0.01, -50.0, 0.0), [0.0, 0.0, 0.0, 1.0]);
    meshkit::into_mesh(b)
}

/// A year's mesh of one kind, for the renderer, which alone keeps it.
fn non_empty(b: meshkit::Builder) -> Mesh {
    let mut m = if b.is_empty() { placeholder_mesh() } else { meshkit::into_mesh(b) };
    m.asset_usage = bevy::asset::RenderAssetUsages::RENDER_WORLD;
    m
}

#[allow(clippy::too_many_arguments)]
fn setup_world(
    mut commands: Commands,
    data: Res<Data>,
    look: Res<TheLook>,
    tl: Res<Timeline>,
    orbit: Res<Orbit>,
    tilt: Res<Tilt>,
    shot: Option<Res<ShotPlanArgs>>,
    forced: Res<ForcedSeason>,
    debug: Res<GroundDebug>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ground_mats: ResMut<Assets<terrain::GroundMaterial>>,
    mut built_mats: ResMut<Assets<built::BuiltMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut fonts: ResMut<Assets<Font>>,
) {
    let (r, g) = (&data.r, &data.g);
    let lk: &Look = &look;
    let k = tl.t.floor() as usize;
    commands.insert_resource(UiFont(fonts.add(Font::from_bytes(FREDOKA.to_vec()))));

    // ground: one heightfield, its patchwork painted per pixel from the year's land mix
    // (task C, the land) the year's land is worked out once, for its parcels and for its wood and heath
    let land = world::year_land(r, g, k);
    let year_cells = images.add(terrain::parcel_image(g, terrain::year_parcels(r, g, k, &land)));
    let year_field = images.add(terrain::field_image(g, land.field));
    let fixed_cells = images.add(terrain::parcel_image(g, terrain::fixed_parcels(r, g)));
    let terrain_mat = ground_mats.add(terrain::GroundMaterial {
        base: StandardMaterial { perceptual_roughness: lk.surfaces.ground, reflectance: lk.surfaces.ground_reflectance, ..default() },
        extension: terrain::Patchwork {
            params: terrain::patchwork_params(r, g, season_phase(&SeasonClock::default(), forced.0), debug.0),
            year: year_cells.clone(),
            fixed: fixed_cells,
            paint: lk.land.uniform(),
            overlay: images.add(overlay::blank(r)),
            rivers: images.add(world::timed("far rivers", || terrain::river_far(g))),
            field: year_field.clone(),
        },
    });
    for chunk in terrain::terrain_chunks(g) {
        commands.spawn((Mesh3d(meshes.add(chunk)), MeshMaterial3d(terrain_mat.clone()), GroundMesh));
    }

    // the sea: a deep floor beyond the map, under the sea's surface
    let (w, h) = (r.w as f32, r.h as f32);
    commands.spawn((
        Mesh3d(meshes.add(terrain::plane(-4.0 * w, -4.0 * h, 5.0 * w, 5.0 * h, -1.15))),
        MeshMaterial3d(materials.add(StandardMaterial { base_color: lk.water.sea_floor.color(), perceptual_roughness: 1.0, ..default() })),
    ));
    // (the sea's surface is terrain.rs's, with its own shader)

    // rivers do not change
    let river_mat = materials.add(StandardMaterial { base_color: Color::WHITE, perceptual_roughness: lk.surfaces.river, reflectance: lk.surfaces.river_reflectance, ..default() });
    commands.spawn((Mesh3d(meshes.add(non_empty(network::rivers(g, lk)))), MeshMaterial3d(river_mat.clone()), bevy::light::NotShadowCaster));

    // year meshes start empty; rebuild_year fills them (`year::handles` makes their handles and
    // `year::spawn` their entities, from the table of a year's meshes)
    let vc = vertex_coloured(&mut materials, lk, lk.surfaces.buildings);
    let meshes_of_year = year::handles(&mut meshes, year::MESHES);
    // trees: what is green all year as it is; the leaves of the broadleaves under a material
    // that turns with the seasons; their bare limbs, flat blades seen from both sides. The
    // leaves and the limbs of a group are shown turn about (`seasons`), so winter costs no rebuild.
    let evergreen_mat = vertex_coloured(&mut materials, lk, lk.surfaces.foliage);
    let tree_mat = vertex_coloured(&mut materials, lk, lk.surfaces.foliage);
    let bare_mat = materials.add(bare_limb_material(lk));
    // houses and works (churches, farm buildings and bridges among them) are drawn in a
    // material that winter lies on; the trains keep the plain one, an engine being warm
    let phase = season_phase(&SeasonClock::default(), forced.0);
    let built_mat = built_mats.add(built::material(lk, phase, lk.surfaces.buildings));
    // and so is what lies flat on the ground (the yards of the works, their ponds, the farm
    // lanes, the roads and the railway's track): frost on the yards, ice on a mill pond.
    // The roads and the track are marked as trodden clear, so winter leaves them as easy
    // to follow as summer does; they are in this material all the same, because where a
    // lane meets a road the two lie at one height, and which of two materials is drawn
    // over the other there is not the same from one run of the program to the next
    let yards_mat = built_mats.add(built::material(lk, phase, lk.surfaces.paths));
    let smoke_mat = materials.add(smoke_material(lk));
    // the year's entities go in three runs, with the season's materials and the train's queued
    // between them where they always were: the trees (made first), then the houses and the
    // works (made second and third), then the rest
    let mats = year::Materials { built: built_mat.clone(), flat: yards_mat.clone(), smoke: smoke_mat.clone(), evergreen: evergreen_mat, leaves: tree_mat.clone(), bare: bare_mat };
    let mut drawers = handover::Drawers::default();
    year::spawn(&mut commands, &meshes_of_year, year::MESHES, &mats, ..1, &mut drawers);
    commands.insert_resource(SeasonMats { terrain: terrain_mat, trees: tree_mat, rivers: river_mat, built: [built_mat.clone(), yards_mat.clone()], smoke: smoke_mat.clone(), forced: forced.0 });
    year::spawn(&mut commands, &meshes_of_year, year::MESHES, &mats, 1..3, &mut drawers);
    commands.insert_resource(TrainAssets { mesh: meshes.add(meshkit::into_mesh(trains::train_mesh(lk))), mat: vc.clone() });
    year::spawn(&mut commands, &meshes_of_year, year::MESHES, &mats, 3.., &mut drawers);
    commands.insert_resource(Handles { year_cells, year_field, meshes: meshes_of_year, drawers });

    // a warm low sun from the south-west, and soft sky light; the seasons
    // move it from the first frame on, so this is only where it starts
    spawn_sun(&mut commands, lk);

    // the camera
    let mut cam = spawn_view_camera(&mut commands, lk, &orbit, tilt.0, debug.0);
    cam.insert((MainCam, IsDefaultUiCamera));
    if let Some(shot) = shot {
        let mut img = Image::new_target_texture(shot.size.0, shot.size.1, TextureFormat::Rgba8UnormSrgb, None);
        img.texture_descriptor.usage |= TextureUsages::COPY_SRC;
        let handle = images.add(img);
        cam.insert(RenderTarget::Image(handle.clone().into()));
        commands.insert_resource(ShotPlan {
            views: shot.views.clone(),
            index: 0,
            image: handle,
            frames: shot.frames,
            countdown: shot.frames,
            capturing: 0,
            wait: shot.wait,
            default_dist: shot.default_dist,
            default_look: shot.default_look,
            default_season: forced.0,
        });
    }
}

/// The sun, as the look has it at midsummer; `seasons` (or the lab) moves it from there.
fn spawn_sun(commands: &mut Commands, lk: &Look) {
    commands.spawn((
        Sun,
        DirectionalLight { color: lk.sun.colour(1.0), illuminance: lk.sun.strength(1.0), shadow_maps_enabled: true, ..default() },
        Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, lk.sun.turn, -lk.sun.height(1.0), 0.0)),
        CascadeShadowConfigBuilder { num_cascades: lk.shadows.cascades, first_cascade_far_bound: lk.shadows.first_far, maximum_distance: lk.shadows.max_distance, ..default() }.build(),
    ));
}

/// The sun at a phase of the year (0 midwinter, 0.5 midsummer): its strength, its colour and where it stands.
fn sun_at(lk: &Look, phase: f32) -> (f32, Color, Transform) {
    // the sun rides higher and warmer in summer
    let summer = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
    (lk.sun.strength(summer), lk.sun.colour(summer), Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, lk.sun.turn, -lk.sun.height(summer), 0.0)))
}

/// The material of the bare limbs of the broadleaf trees: each limb is one
/// flat blade, so it is drawn from both sides, and lets half the light
/// through, so that the side turned from the sun is not a black spike.
fn bare_limb_material(lk: &Look) -> StandardMaterial {
    StandardMaterial { base_color: Color::WHITE, perceptual_roughness: lk.surfaces.foliage, reflectance: lk.surfaces.painted_reflectance, diffuse_transmission: lk.surfaces.bare_limb_translucence, double_sided: true, cull_mode: None, ..default() }
}

/// The material of smoke: white, matt, letting some light through, and thin:
/// a plume's corners carry how solid it is there (its colour's alpha), so it
/// fades to nothing downwind.
fn smoke_material(lk: &Look) -> StandardMaterial {
    StandardMaterial { base_color: Color::WHITE, perceptual_roughness: 1.0, reflectance: 0.0, diffuse_transmission: lk.surfaces.smoke_translucence, alpha_mode: AlphaMode::Blend, ..default() }
}

/// The camera the island is seen through, with everything the look gives it:
/// the lens, the tonemapping and exposure, the haze, bloom, ambient occlusion
/// and (if asked for) the tilt-shift. The game and the lab both spawn theirs
/// here, so the lab's picture is taken through the game's own camera.
fn spawn_view_camera<'a>(commands: &'a mut Commands, lk: &Look, orbit: &Orbit, tilt: bool, ground_debug: bool) -> EntityCommands<'a> {
    let mut cam = commands.spawn((
        Camera3d::default(),
        Hdr,
        Msaa::Off,
        Projection::Perspective(PerspectiveProjection { fov: FOV_DEG.to_radians(), near: 0.05, far: 2000.0, ..default() }),
        tonemapping(lk.post.tonemapping),
        Exposure { ev100: lk.post.exposure },
        bevy::anti_alias::smaa::Smaa::default(),
        DistanceFog {
            color: lk.haze.colour.color(),
            directional_light_color: lk.haze.sun_glow.color().with_alpha(lk.haze.sun_glow_strength),
            directional_light_exponent: lk.haze.sun_glow_tightness,
            falloff: haze_falloff(lk, orbit.dist),
        },
        orbit_transform(orbit),
    ));
    if lk.post.bloom > 0.0 && !ground_debug {
        cam.insert(Bloom { intensity: lk.post.bloom, ..Bloom::NATURAL });
    }
    if lk.ambient_occlusion.on {
        let ao = &lk.ambient_occlusion;
        cam.insert(ScreenSpaceAmbientOcclusion {
            quality_level: match ao.quality {
                look::Quality::Low => ScreenSpaceAmbientOcclusionQualityLevel::Low,
                look::Quality::Medium => ScreenSpaceAmbientOcclusionQualityLevel::Medium,
                look::Quality::High => ScreenSpaceAmbientOcclusionQualityLevel::High,
                look::Quality::Ultra => ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
            },
            radius: ao.radius,
            constant_object_thickness: ao.thickness,
        });
    }
    if ground_debug {
        // flat class colours must reach the image unchanged
        cam.insert(Tonemapping::None);
    }
    if tilt {
        cam.insert(tilt_shift(lk, orbit.dist));
    }
    cam
}

/// The miniature look at a camera distance: only what the camera looks at is sharp.
fn tilt_shift(lk: &Look, dist: f32) -> DepthOfField {
    DepthOfField { mode: DepthOfFieldMode::Gaussian, focal_distance: dist, aperture_f_stops: lk.tilt_shift.f_stops(dist), ..default() }
}

/// Vertical field of view of the camera, in degrees.
const FOV_DEG: f32 = 30.0;

/// The look file's name for a tonemapper, as Bevy's.
fn tonemapping(t: look::Tonemap) -> Tonemapping {
    match t {
        look::Tonemap::None => Tonemapping::None,
        look::Tonemap::Linear => Tonemapping::Linear,
        look::Tonemap::Reinhard => Tonemapping::Reinhard,
        look::Tonemap::ReinhardLuminance => Tonemapping::ReinhardLuminance,
        look::Tonemap::AcesFitted => Tonemapping::AcesFitted,
        look::Tonemap::Agx => Tonemapping::AgX,
        look::Tonemap::SomewhatBoringDisplayTransform => Tonemapping::SomewhatBoringDisplayTransform,
        look::Tonemap::TonyMcMapface => Tonemapping::TonyMcMapface,
        look::Tonemap::BlenderFilmic => Tonemapping::BlenderFilmic,
        look::Tonemap::KhronosPbrNeutral => Tonemapping::KhronosPbrNeutral,
    }
}

/// The haze begins and ends at multiples of the camera's distance, so it
/// frames what is looked at from any height.
fn haze_falloff(lk: &Look, dist: f32) -> FogFalloff {
    FogFalloff::Linear { start: dist * lk.haze.start, end: dist * lk.haze.end }
}

fn orbit_transform(o: &Orbit) -> Transform {
    let dir = Vec3::new(o.yaw.sin() * o.pitch.cos(), o.pitch.sin(), o.yaw.cos() * o.pitch.cos());
    Transform::from_translation(o.target + dir * o.dist).looking_at(o.target, Vec3::Y)
}

/// One round of the seasons' own clock takes this many seconds, however fast
/// the years go by.
const SEASON_ROUND_SECS: f32 = 24.0;

/// The fastest the season moves to catch up with the date, in years a second.
const SEASON_GLIDE: f32 = 0.35;

/// The seasons' clock. Where the clock shows a month, and a year takes long
/// enough to follow (`clock::FOLLOW_YEAR_SECS`: the two slowest speeds, and a
/// clock that is stopped), the season is the date's: the ground, the trees,
/// the sun and the date agree. A year that goes by faster would strobe the
/// fields and the sun, so then the seasons turn in real time on a calm round
/// of their own, one in `SEASON_ROUND_SECS`. Where the clock shows a year and
/// no month (a year gone by, a still), a stopped clock holds the season where
/// it is. The season never jumps: it eases onto the date the short way round.
#[derive(Resource, Clone, Copy, Debug)]
struct SeasonClock {
    /// phase of the year: 0 midwinter, 0.25 spring, 0.5 midsummer, 0.75 autumn
    phase: f32,
    /// the season on screen is the date's
    follows: bool,
    /// the date's year goes by in less than a round of the seasons' own
    /// clock: the season is then shown in steps twice as long, so that the
    /// ground, the trees and the sun are touched no oftener than on that round
    coarse: bool,
}

impl Default for SeasonClock {
    /// High summer: where a picture or a recorded run opens.
    fn default() -> Self {
        SeasonClock { phase: 0.55, follows: false, coarse: false }
    }
}

impl SeasonClock {
    /// Move on by `dt` seconds of real time, with the year's clock at `t`
    /// years and running at `speed` years a second (or stopped). `dated` says
    /// the clock is showing a month.
    fn advance(&mut self, playing: bool, speed: f32, t: f32, dt: f32, dated: bool) {
        // a long frame must not show as a jump of the season
        let dt = dt.clamp(0.0, 0.1);
        self.follows = dated && (!playing || speed <= 1.0 / clock::FOLLOW_YEAR_SECS + 1e-4);
        self.coarse = self.follows && playing && speed > 1.0 / SEASON_ROUND_SECS + 1e-4;
        if self.follows {
            // ease onto the date the short way round, then stay on it; from far off
            // (a clock stopped after running fast) at a glide, half a year in a second and a half
            let d = (t.fract() - self.phase + 0.5).rem_euclid(1.0) - 0.5;
            let most = SEASON_GLIDE * dt;
            self.phase += (d * (1.0 - (-4.0 * dt).exp())).clamp(-most, most);
        } else if playing {
            self.phase += dt / SEASON_ROUND_SECS;
        }
        self.phase = self.phase.rem_euclid(1.0);
    }
}

fn season_clock(time: Res<Time>, tl: Res<Timeline>, data: Res<Data>, live: Option<Res<live::Live>>, stills: Option<Res<ShotPlanArgs>>, mut clock: ResMut<SeasonClock>) {
    let dated = shown_date(&tl, &data, live.as_deref(), stills.is_some()).is_some();
    clock.advance(tl.playing, tl.speed, tl.t, time.delta_secs(), dated);
}

/// The date the top bar shows, where it shows a month: wherever the clock
/// stands inside a year, and at the turn of the year a game is in. On the
/// turn of a year gone by (and at the end of the last), and in every still,
/// the bar shows that year alone, which is the year the map is of.
fn shown_date(tl: &Timeline, data: &Data, live: Option<&live::Live>, stills: bool) -> Option<clock::Date> {
    if stills {
        return None;
    }
    let r = &data.r;
    let in_progress = live.is_some() && tl.t.floor() as usize >= data.latest() && r.filled < r.ny;
    (!clock::on_turn(tl.t) || in_progress).then(|| clock::date(r.y0, tl.t))
}

/// Phase of the year for the ground, the trees, the sun and the date: the
/// seasons' clock, or the season a screenshot asked for.
fn season_phase(clock: &SeasonClock, forced: Option<f32>) -> f32 {
    match forced {
        Some(f) => f.rem_euclid(1.0),
        // 96 steps to the year, each given from a little inside it so that
        // `seasons`, which counts 192, sees every other one of its own
        None if clock.coarse => ((clock.phase * 96.0).floor() + 0.25) / 96.0,
        None => clock.phase,
    }
}

fn seasons(
    clock: Res<SeasonClock>,
    sm: Res<SeasonMats>,
    look: Res<TheLook>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ground_mats: ResMut<Assets<terrain::GroundMaterial>>,
    mut built_mats: ResMut<Assets<built::BuiltMaterial>>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), With<Sun>>,
    mut last: Local<Option<i32>>,
) {
    let _frames = frames::span(frames::Sys::Seasons);
    let ph = season_phase(&clock, sm.forced);
    // only touch the materials when the season visibly moves, about twice a week
    let step = (ph * 192.0).floor() as i32;
    if *last == Some(step) {
        return;
    }
    *last = Some(step);
    // the ground shader draws the crops, grass, heather and frost of the season
    if let Some(mut m) = ground_mats.get_mut(&sm.terrain) {
        m.extension.params.season = ph;
    }
    // snow lies on what is built in the depth of winter, frost on the ground round it, and
    // its gardens are bare: the buildings' material is told the season and does the rest
    // per pixel (built.wesl), so winter rebuilds no mesh. Outside the winter (seven months
    // in twelve) its numbers do not change and it is left alone
    let winter = built::params(&look, ph);
    for h in &sm.built {
        if built_mats.get(h).is_some_and(|m| m.extension.params.amounts != winter.amounts) {
            if let Some(mut m) = built_mats.get_mut(h) {
                m.extension.params = winter;
            }
        }
    }
    // and smoke stands a little fuller in the cold
    let full = built::smoke_fullness(&look, ph);
    if materials.get(&sm.smoke).is_some_and(|m| m.base_color.alpha() != full) {
        if let Some(mut m) = materials.get_mut(&sm.smoke) {
            m.base_color = Color::WHITE.with_alpha(full);
        }
    }
    // the leaves turn: 0 midwinter, 0.25 spring, 0.5 summer, 0.75 autumn; and each group of
    // broadleaf trees is in leaf from spring to autumn and shows its bare limbs through the
    // winter (`show_trees`). They go by the seasons' clock, as the ground and the sun do, so
    // a wood is gold, bare and green again once a round at any speed of the years, and holds
    // while paused
    if let Some(mut m) = materials.get_mut(&sm.trees) {
        m.base_color = leaf_tint(&look, ph);
    }
    // the rivers go to steel in the depth of winter
    if let Some(mut m) = materials.get_mut(&sm.rivers) {
        let t = look.water.river_tint(ph);
        m.base_color = Color::srgb(t[0], t[1], t[2]);
    }
    let (strength, colour, stands) = sun_at(&look, ph);
    for (mut light, mut tf) in &mut sun {
        light.illuminance = strength;
        light.color = colour;
        *tf = stands;
    }
}

/// One of the three meshes a group of trees is drawn as.
#[derive(Clone, Copy, Debug, PartialEq)]
enum TreePart {
    Evergreen,
    Leaves(usize),
    BareLimbs(usize),
}

/// Whether a part of the trees shows at a phase of the year. Under a map view
/// (`aside`) none does: the woods would hide the wash under their leaves, and
/// bare limbs and conifers would speckle it. Otherwise a group of broadleaves
/// shows its leaves while it is in leaf and its bare limbs while it is not,
/// and what is evergreen always shows.
fn tree_part_shown(part: TreePart, phase: f32, aside: bool) -> bool {
    !aside
        && match part {
            TreePart::Evergreen => true,
            TreePart::Leaves(group) => in_leaf(phase, group),
            TreePart::BareLimbs(group) => !in_leaf(phase, group),
        }
}

/// Show and hide the trees' meshes. This is the one place that does: the
/// season and the map view both bear on it, and when each of them set the
/// trees' visibility for itself, the season put back the leaves a map view
/// had hidden (and the conifers and bare limbs were never hidden at all).
/// It runs every frame over a dozen entities and writes only what changes.
#[allow(clippy::type_complexity)]
fn show_trees(
    clock: Res<SeasonClock>,
    sm: Option<Res<SeasonMats>>,
    ov: Res<overlay::MapOverlay>,
    debug: Res<GroundDebug>,
    mut trees: Query<(Option<&Leaves>, Option<&BareLimbs>, &mut Visibility), Or<(With<Leaves>, With<BareLimbs>, With<Evergreen>)>>,
) {
    let ph = season_phase(&clock, sm.and_then(|sm| sm.forced));
    // `--ground-debug` shows the ground alone
    let aside = ov.kind.is_some() || debug.0;
    for (leaves, bare, mut vis) in &mut trees {
        let part = match (leaves, bare) {
            (Some(l), _) => TreePart::Leaves(l.0),
            (None, Some(b)) => TreePart::BareLimbs(b.0),
            (None, None) => TreePart::Evergreen,
        };
        vis.set_if_neq(if tree_part_shown(part, ph, aside) { Visibility::Inherited } else { Visibility::Hidden });
    }
}

/// How many of the trees' meshes are set to show, of how many (for `--script`'s `map`).
fn trees_showing(world: &mut World) -> (usize, usize) {
    let mut q = world.query_filtered::<&Visibility, Or<(With<Leaves>, With<BareLimbs>, With<Evergreen>)>>();
    q.iter(world).fold((0, 0), |(shown, all), v| (shown + (*v != Visibility::Hidden) as usize, all + 1))
}

/// What the leaves are multiplied by at a phase of the year.
fn leaf_tint(lk: &Look, phase: f32) -> Color {
    let t = lk.foliage.tint(phase);
    Color::srgb(t[0], t[1], t[2])
}

fn time_input(keys: Res<ButtonInput<KeyCode>>, mut tl: ResMut<Timeline>, data: Res<Data>, mut tilt: ResMut<Tilt>, cards: Res<game::CardOpen>) {
    // with a decision, the welcome or the end card up, the keys are the card's
    if cards.0 {
        return;
    }
    let last = data.latest() as f32;
    if keys.just_pressed(KeyCode::Space) {
        if !tl.playing && tl.t >= (data.r.ny - 1) as f32 && !data.live {
            tl.t = 0.0;
        }
        tl.playing = !tl.playing;
    }
    // in a game, forwards from the year in progress is the simulation's to give:
    // Right plays a month and Page Up plays on to the year's turn (game::next_clicks)
    let in_progress = data.live && tl.t.floor() >= last;
    if (keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::Period)) && !in_progress {
        tl.playing = false;
        tl.t = (tl.t.floor() + 1.0).min(last);
    }
    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::Comma) {
        // back to the turn the year began at, and from a turn to the one before
        tl.playing = false;
        tl.t = if clock::on_turn(tl.t) { (tl.t.floor() - 1.0).max(0.0) } else { tl.t.floor() };
    }
    if keys.just_pressed(KeyCode::PageUp) && !in_progress {
        tl.t = (tl.t.floor() + 10.0).min(last);
    }
    if keys.just_pressed(KeyCode::PageDown) {
        tl.t = (tl.t.floor() - 10.0).max(0.0);
    }
    // faster and slower go from one speed button to the next
    if keys.just_pressed(KeyCode::ArrowUp) {
        tl.speed = clock::step_speed(tl.speed, true);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        tl.speed = clock::step_speed(tl.speed, false);
    }
    if keys.just_pressed(KeyCode::KeyT) {
        tilt.0 = !tilt.0;
    }
}

fn advance_time(time: Res<Time>, mut tl: ResMut<Timeline>, data: Res<Data>, game: Res<game::Game>, live: Option<Res<live::Live>>) {
    // a card that waits for the player stops the clock, however it came to be open
    if game.modal() && tl.playing {
        tl.playing = false;
    }
    if !tl.playing {
        return;
    }
    // a live game runs into the first month still to come and waits inside it
    // until that month arrives, and at December's end until the year does
    let end = (data.r.ny - 1) as f32;
    let cap = if data.r.filled < data.r.ny { clock::frontier(data.latest(), live.map_or(11, |l| l.months)) } else { end };
    tl.t = (tl.t + time.delta_secs() * tl.speed).min(cap);
    if tl.t >= end {
        tl.playing = false;
    }
}

// ------------------------------------------------------------------ camera

fn camera_input(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    mut o: ResMut<Orbit>,
    data: Res<Data>,
    ui: Res<UiState>,
    mut on_map: Local<[bool; 2]>,
    brush: Option<Res<ground::Brush>>,
) {
    let dt = time.delta_secs();
    // (with the brush in hand a drag with the left button paints, ground.rs)
    let painting = brush.is_some_and(|b| b.in_hand());
    let fwd = Vec3::new(-o.yaw.sin(), 0.0, -o.yaw.cos());
    let right = Vec3::new(o.yaw.cos(), 0.0, -o.yaw.sin());
    let pan = o.dist * 0.0016;
    // a drag belongs to the map only if it started on the map
    for (i, b) in [MouseButton::Left, MouseButton::Right].into_iter().enumerate() {
        if buttons.just_pressed(b) {
            on_map[i] = !ui.hover;
        }
    }
    if buttons.pressed(MouseButton::Left) && on_map[0] && !painting {
        let d = motion.delta;
        o.target -= right * d.x * pan;
        o.target += fwd * d.y * pan;
    }
    if buttons.pressed(MouseButton::Right) && on_map[1] {
        let d = motion.delta;
        o.yaw -= d.x * 0.005;
        o.pitch = (o.pitch + d.y * 0.004).clamp(0.22, 1.5);
    }
    let lines = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / 40.0,
    };
    if lines != 0.0 && !ui.hover {
        o.dist = (o.dist * (-lines * 0.12).exp()).clamp(shots::CLOSE_DIST, shots::FURTHEST_DIST);
    }
    let kp = o.dist * 0.8 * dt;
    let mut mv = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { mv += fwd; }
    if keys.pressed(KeyCode::KeyS) { mv -= fwd; }
    if keys.pressed(KeyCode::KeyD) { mv += right; }
    if keys.pressed(KeyCode::KeyA) { mv -= right; }
    o.target += mv * kp;
    if keys.pressed(KeyCode::KeyQ) { o.yaw += 1.2 * dt; }
    if keys.pressed(KeyCode::KeyE) { o.yaw -= 1.2 * dt; }
    if keys.pressed(KeyCode::KeyR) { o.dist = (o.dist * (1.0 - 1.5 * dt)).max(shots::CLOSE_DIST); }
    if keys.pressed(KeyCode::KeyF) { o.dist = (o.dist * (1.0 + 1.5 * dt)).min(shots::FURTHEST_DIST); }
    let (w, h) = (data.r.w as f32, data.r.h as f32);
    o.target.x = o.target.x.clamp(-10.0, w + 10.0);
    o.target.z = o.target.z.clamp(-10.0, h + 10.0);
    let ground = world::height_at(&data.g, o.target.x, o.target.z).max(0.0);
    o.target.y += (ground - o.target.y) * (1.0 - (-8.0 * dt.max(1.0 / 30.0)).exp());
}

fn apply_camera(
    o: Res<Orbit>,
    tilt: Res<Tilt>,
    look: Res<TheLook>,
    mut commands: Commands,
    mut cams: Query<(Entity, &mut Transform, &mut DistanceFog, Option<&mut DepthOfField>), With<MainCam>>,
) {
    for (e, mut t, mut fog, dof) in &mut cams {
        *t = orbit_transform(&o);
        fog.falloff = haze_falloff(&look, o.dist);
        match (tilt.0, dof) {
            (true, Some(mut d)) => {
                d.focal_distance = o.dist;
                d.aperture_f_stops = look.tilt_shift.f_stops(o.dist);
            }
            (true, None) => {
                commands.entity(e).insert(tilt_shift(&look, o.dist));
            }
            (false, Some(_)) => {
                commands.entity(e).remove::<DepthOfField>();
            }
            _ => {}
        }
    }
}

// ------------------------------------------------------------------ interface

/// The smallest window the game opens, in logical pixels (pictures with no window may be any size).
const MIN_WINDOW: (f32, f32) = (1280.0, 720.0);

/// How far the news as it happens and the note on what was clicked stand from the window's right edge.
const TOAST_RIGHT: f32 = 14.0;
/// The width of the note on what was clicked.
const INSPECT_W: f32 = 380.0;

// Interaction is deprecated in Bevy 0.20; see the note at the top of ui.rs
#[allow(deprecated)]
fn setup_ui(mut commands: Commands, font: Res<UiFont>, data: Res<Data>) {
    let f = &font.0;
    // news as it happens, top right under the tabs
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(TOAST_RIGHT),
            top: Val::Px(86.0),
            width: Val::Px(340.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },
        GlobalZIndex(5),
        ToastBox,
    ));
    // the inspector, bottom right above the timeline. It stands on the timeline's card and grows
    // upward with what it has to say, as far as the bar across the top and no further: its
    // room is the column between the two, and past that its lines scroll under its title
    // (a square at the seat can list every works the family could build there)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(TOAST_RIGHT),
                top: Val::Px(86.0),
                bottom: Val::Px(70.0),
                width: Val::Px(INSPECT_W),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::End,
                ..default()
            },
            GlobalZIndex(10),
            Visibility::Hidden,
            InspectPanel,
        ))
        .with_children(|room| {
            room.spawn((
                Node {
                    max_height: Val::Percent(100.0),
                    padding: UiRect::new(Val::Px(16.0), Val::Px(16.0 - ui::THUMB_ROOM), Val::Px(12.0), Val::Px(14.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    border_radius: BorderRadius::all(Val::Px(16.0)),
                    ..default()
                },
                BackgroundColor(PAPER),
                BoxShadow::new(Color::srgba(0.25, 0.18, 0.10, 0.18), Val::Px(0.0), Val::Px(3.0), Val::Px(0.0), Val::Px(10.0)),
                Interaction::default(),
                bevy::ui::FocusPolicy::Block,
                bevy::ui::RelativeCursorPosition::default(),
            ))
            .with_children(|p| {
                let card = p.target_entity();
                p.spawn((text(f, "", 20.0, INK), InspectTitle));
                ui::scroll_list(p, card, Node::default(), Node { row_gap: Val::Px(4.0), padding: UiRect::right(Val::Px(ui::THUMB_ROOM)), ..default() }, InspectBody, |p| {
                    p.spawn((text(f, "", 14.0, MUTED), InspectText));
                    // a firm's card, as terms and numbers (firm_card.rs)
                    p.spawn((Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(1.0), ..default() }, firm_card::FirmCard));
                    p.spawn((Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), ..default() }, InspectActions));
                    // in a sandbox, what the simulation knows of the thing (inspector.rs)
                    p.spawn((Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(2.0), ..default() }, inspector::InspectorBlock));
                    // MAKING: a town's chip that opens its making (town_flow.rs)
                    p.spawn((Node { flex_direction: FlexDirection::Column, ..default() }, town_flow::MakingBlock));
                    // and a town's ideas, in a section of their own (town_ideas.rs)
                    p.spawn((Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(2.0), ..default() }, town_ideas::IdeasBlock));
                });
            });
        });
    // town labels (a town founded later in a live game gets its own in `labels`)
    for t in 0..data.r.header.towns.len().min(data.r.n_towns) {
        town_label(&mut commands, f, t, &data.r.header.towns[t].name);
    }
}

/// A town's label, hidden until `labels` places it.
fn town_label(commands: &mut Commands, f: &Handle<Font>, t: usize, name: &str) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(220.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Visibility::Hidden,
            TownLabel(t),
        ))
        .with_children(|p| {
            // (a click on the name picks the town: `label_clicks`)
            #[allow(deprecated)]
            p.spawn((
                Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(9.0)), ..default() },
                BackgroundColor(Color::srgba(0.99, 0.97, 0.92, 0.82)),
                Interaction::default(),
                LabelPill(t),
            ))
            .with_children(|q| {
                q.spawn(text(f, name, 15.0, INK));
            });
        });
}

/// "1 family", "6 families": what the terminal and the sheet of pictures say of a game.
fn families_words(n: usize) -> String {
    if n == 1 { "1 family".to_string() } else { format!("{n} families") }
}

/// A number of people: whole, with a comma for the thousands, in millions from a million up.
fn fmt_pop(p: f32) -> String {
    // round first, so that 12,999.6 is 13,000 and not 12,1000
    let n = if p.is_finite() { p.round().max(0.0) as u64 } else { 0 };
    if n >= 1_000_000 {
        format!("{:.2} million", n as f64 / 1e6)
    } else if n >= 1000 {
        format!("{},{:03}", n / 1000, n % 1000)
    } else {
        format!("{n}")
    }
}

#[allow(clippy::too_many_arguments)]
fn labels(
    mut commands: Commands,
    font: Res<UiFont>,
    data: Res<Data>,
    tl: Res<Timeline>,
    o: Res<Orbit>,
    ov: Res<overlay::MapOverlay>,
    cams: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    mut q: Query<(&TownLabel, &mut Node, &mut Visibility, &Children)>,
    mut boxes: Query<&Children, Without<TownLabel>>,
    mut fonts: Query<&mut TextFont>,
) {
    let _frames = frames::span(frames::Sys::Hud);
    let r = &data.r;
    // a town founded in a live game since the labels were made (by the rule or by hand) gets one
    let have = q.iter().count();
    for t in have..r.header.towns.len().min(r.n_towns) {
        town_label(&mut commands, &font.0, t, &r.header.towns[t].name);
    }
    let Ok((cam, cam_t)) = cams.single() else { return };
    let k = (tl.t.floor() as usize).min(r.ny - 1);
    // rank towns by size so only the bigger ones are named when zoomed out
    let mut order: Vec<(usize, f32)> = (0..r.n_towns).filter(|&t| r.town_active[k * r.n_towns + t] > 0).map(|t| (t, r.town_urban[k * r.n_towns + t])).collect();
    order.sort_by(|a, b| ui::biggest_first(a.1, b.1));
    // a map view that goes by market towns names every one of them
    let max_shown = if ov.names_towns() { 99 } else if o.dist > 90.0 { 9 } else if o.dist > 45.0 { 16 } else { 99 };
    let biggest = order.first().map(|x| x.1).unwrap_or(1.0).max(1.0);
    for (lab, mut node, mut vis, children) in &mut q {
        let t = lab.0;
        let rank = order.iter().position(|x| x.0 == t);
        let Some(rank) = rank else {
            *vis = Visibility::Hidden;
            continue;
        };
        if rank >= max_shown {
            *vis = Visibility::Hidden;
            continue;
        }
        let (x, z) = data.cen[r.header.towns[t].node];
        let p = Vec3::new(x, world::height_at(&data.g, x, z).max(0.0) + 0.35, z);
        match cam.world_to_viewport(cam_t, p) {
            Ok(v) => {
                node.left = Val::Px(v.x - 110.0);
                node.top = Val::Px(v.y - 34.0);
                *vis = Visibility::Visible;
                let size = 13.0 + 7.0 * (order[rank].1 / biggest).max(0.0).sqrt();
                for c in children.iter() {
                    if let Ok(inner) = boxes.get_mut(c) {
                        for tc in inner.iter() {
                            if let Ok(mut tf) = fonts.get_mut(tc) {
                                tf.font_size = FontSize::Px(size.round());
                            }
                        }
                    }
                }
            }
            Err(_) => *vis = Visibility::Hidden,
        }
    }
}

fn toasts(
    mut commands: Commands,
    mut st: ResMut<Toasts>,
    tl: Res<Timeline>,
    time: Res<Time>,
    data: Res<Data>,
    font: Res<UiFont>,
    ui: Res<UiState>,
    sheet: Option<Res<shots::Sheet>>,
    picked: Res<Picked>,
    mut boxq: Query<(Entity, Option<&Children>, &mut Visibility, &mut Node), With<ToastBox>>,
) {
    let _frames = frames::span(frames::Sys::Hud);
    // with the news open the panel says it all; the standard shots are of the island, not the news
    for (_, _, mut v, mut node) in &mut boxq {
        v.set_if_neq(if ui.tab == Some(ui::Tab::News) || sheet.is_some() { Visibility::Hidden } else { Visibility::Inherited });
        // the note on what was clicked stands at the right and can grow to the top (a long Build list):
        // while it is open the news stands to the left of it rather than under it
        let right = Val::Px(if picked.what.is_some() { TOAST_RIGHT + INSPECT_W + 12.0 } else { TOAST_RIGHT });
        if node.right != right {
            node.right = right;
        }
    }
    let r = &data.r;
    let now = time.elapsed_secs();
    let mut changed = std::mem::take(&mut st.fresh);
    // where the clock stands as the news is dated; the clock's whole numbers are years' ends
    let to = clock::news_time(r.y0, tl.t.min((r.ny - 1) as f32));
    let known = r.header.events.len();
    if st.last != Some(to) || st.known != known {
        // a clock that has gone back, or on by more than a year, starts afresh with the year up to where it stands
        let from = match st.last {
            Some(l) if to >= l - 1e-4 && to - l <= 1.0 + 1e-3 => l,
            _ => {
                st.shown.clear();
                changed = true;
                to - 1.0
            }
        };
        let mut fresh = news_toasts(&r.header.events, from, to, st.known, now);
        // an edit by hand put its line up when it was answered, before its news came
        fresh.retain(|t| !st.shown.iter().any(|s| s.title == t.title));
        changed |= !fresh.is_empty();
        st.shown.extend(fresh);
        (st.last, st.known) = (Some(to), known);
    }
    while st.shown.len() > 3 {
        st.shown.remove(0);
    }
    let before = st.shown.len();
    st.shown.retain(|t| now - t.since < 7.0);
    changed |= st.shown.len() != before;
    if !changed {
        return;
    }
    let Ok((boxe, children, _, _)) = boxq.single() else { return };
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    let f = &font.0;
    let shown = st.shown.clone();
    commands.entity(boxe).with_children(|p| {
        for e in shown.iter().rev() {
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::new(Val::Px(14.0), Val::Px(12.0), Val::Px(8.0), Val::Px(10.0)),
                    border: UiRect::left(Val::Px(5.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(PAPER),
                BoxShadow::new(Color::srgba(0.25, 0.18, 0.10, 0.16), Val::Px(0.0), Val::Px(2.0), Val::Px(0.0), Val::Px(8.0)),
                BorderColor::all(kind_colour(&e.kind)),
            ))
            .with_children(|q| {
                q.spawn(text(f, &format!("{} \u{00b7} {}", kind_label(&e.kind), e.when), 13.0, kind_colour(&e.kind)));
                q.spawn(text(f, &e.title, 18.0, INK));
            });
        }
    });
}

// ------------------------------------------------------------------ headless screenshots

fn take_shot(
    mut commands: Commands,
    plan: Option<ResMut<ShotPlan>>,
    mut exit: MessageWriter<AppExit>,
    mut tl: ResMut<Timeline>,
    mut o: ResMut<Orbit>,
    mut ui: ResMut<UiState>,
    mut sm: ResMut<SeasonMats>,
    mut ov: ResMut<overlay::MapOverlay>,
    data: Res<Data>,
    sheet: Option<Res<shots::Sheet>>,
    waiting: Res<PipelinesWaiting>,
    mut patience: Local<u32>,
    mut held: Local<Option<std::time::Instant>>,
    mut gave_up: Local<Option<usize>>,
) {
    let Some(mut plan) = plan else { return };
    if plan.index >= plan.views.len() {
        if let Some(sheet) = sheet {
            // the pictures are written a few frames after they are asked for: give the last ones time
            let missing = plan.views.iter().filter(|v| !std::path::Path::new(&v.path).exists()).count();
            if missing > 0 && *patience < 300 {
                *patience += 1;
                return;
            }
            if missing > 0 {
                eprintln!("{missing} of the standard shots were not written");
            }
            shots::print_diffs(&shots::write_sheet(&sheet));
        }
        exit.write(AppExit::Success);
        return;
    }
    let v = plan.views[plan.index].clone();
    // point the camera, the clock, the calendar and the side panel at this view
    let r = &data.r;
    if let Some(y) = v.year {
        // a game has only the years played so far: a later one would draw an empty island
        tl.t = (y - r.y0).clamp(0, data.latest() as i32) as f32;
    }
    if let Some(tab) = v.tab {
        if ui.tab != tab {
            ui.tab = tab;
        }
    }
    if let Some(c) = v.overlay.as_deref().and_then(|w| overlay::parse(w, r)) {
        ov.choose(&c);
    }
    if let Some(w) = v.markets.as_deref() {
        ui.show_markets(w, r);
    }
    let season = v.season.or(plan.default_season);
    if sm.forced != season {
        sm.forced = season;
    }
    let look = v.look.unwrap_or(plan.default_look);
    o.target = Vec3::new(look.0, world::height_at(&data.g, look.0, look.1).max(0.0), look.1);
    o.yaw = v.yaw.to_radians();
    o.pitch = v.pitch.to_radians();
    o.dist = v.dist.unwrap_or(plan.default_dist);
    if plan.capturing > 0 {
        plan.capturing -= 1;
        if plan.capturing == 0 {
            plan.index += 1;
            plan.countdown = (plan.frames / 3).max(8);
        }
        return;
    }
    // what a picture needs: first its year's scenery on screen
    if !scenery_ready(&tl, &data) {
        return;
    }
    // then every shader it is drawn with: a pipeline still being put together means
    // something in view is not drawn yet, so the shot is held until it is there and has
    // been drawn for a few frames (a busy machine can take many seconds over the ground's
    // shader, and the first frames of a run are black), but no longer than `--wait`
    let pending = waiting.0.load(std::sync::atomic::Ordering::Relaxed);
    if pending > 0 && *gave_up != Some(plan.index) {
        let since = *held.get_or_insert_with(std::time::Instant::now);
        if since.elapsed().as_secs_f32() < plan.wait {
            plan.countdown = plan.countdown.max(4);
            return;
        }
        eprintln!("{pending} of the renderer's shaders were still being put together after {:.0} s, so {} is taken without what they draw (--wait S gives them longer)", plan.wait, v.path);
        *held = None;
        *gave_up = Some(plan.index);
    } else if let Some(since) = held.take() {
        eprintln!("waited {:.1} s for the renderer's shaders", since.elapsed().as_secs_f32());
    }
    // and some frames for it all to settle
    if plan.countdown > 0 {
        plan.countdown -= 1;
        return;
    }
    commands.spawn(Screenshot::image(plan.image.clone())).observe(save_to_disk(v.path.clone()));
    plan.capturing = 6;
}

// ------------------------------------------------------------------ checking the ground shader

/// With `--ground-debug`, show nothing but the ground.
fn hide_all_but_ground(mut q: Query<&mut Visibility, (With<Mesh3d>, Without<GroundMesh>)>) {
    for mut v in &mut q {
        *v = Visibility::Hidden;
    }
}

/// Where a ray from `o` along unit `d` first meets the terrain mesh.
fn mesh_hit(g: &Ground, o: Vec3, d: Vec3) -> Option<Vec3> {
    // nothing stands above 2.6, so start where the ray comes down to that height
    let mut t = if d.y < 0.0 { ((o.y - 2.6) / -d.y).max(0.0) } else { 0.0 };
    let mut prev = o + d * t;
    for _ in 0..40_000 {
        t += 0.004 + t * 0.001;
        let p = o + d * t;
        if p.y <= terrain::mesh_height(g, p.x, p.z) {
            let (mut a, mut b) = (prev, p);
            for _ in 0..24 {
                let m = (a + b) * 0.5;
                if m.y <= terrain::mesh_height(g, m.x, m.z) { b = m } else { a = m }
            }
            return Some(b);
        }
        prev = p;
        if t > 1000.0 {
            break;
        }
    }
    None
}

/// Write the class that `world::class_at` gives for the ground under every
/// pixel of a view, one byte per pixel (255 for sea, beach and sky), as a PGM
/// next to the view's path. A `--ground-debug` shot of the same view shows what
/// the shader draws there; compare the two to check that they agree.
fn write_class_map(r: &Replay, g: &Ground, v: &ShotSpec, size: (u32, u32), default_look: (f32, f32), default_dist: f32) {
    use rayon::prelude::*;
    let k = v.year.map_or(r.ny - 1, |y| (y - r.y0).clamp(0, r.ny as i32 - 1) as usize);
    let (lx, lz) = v.look.unwrap_or(default_look);
    let o = Orbit {
        target: Vec3::new(lx, world::height_at(g, lx, lz).max(0.0), lz),
        yaw: v.yaw.to_radians(),
        pitch: v.pitch.to_radians(),
        dist: v.dist.unwrap_or(default_dist),
    };
    let cam = orbit_transform(&o);
    let (w, h) = (size.0 as usize, size.1 as usize);
    let tan = (FOV_DEG.to_radians() * 0.5).tan();
    let aspect = w as f32 / h as f32;
    let land = world::year_land(r, g, k);
    let px: Vec<u8> = (0..w * h)
        .into_par_iter()
        .map(|i| {
            let (x, y) = ((i % w) as f32 + 0.5, (i / w) as f32 + 0.5);
            let d = cam.rotation * Vec3::new((x / w as f32 * 2.0 - 1.0) * tan * aspect, (1.0 - y / h as f32 * 2.0) * tan, -1.0);
            match mesh_hit(g, cam.translation, d.normalize()) {
                Some(p) if p.y >= 0.03 && p.x >= 0.0 && p.z >= 0.0 && p.x < r.w as f32 && p.z < r.h as f32 => world::class_at(g, &land, p.x, p.z),
                _ => 255,
            }
        })
        .collect();
    let path = format!("{}.classes.pgm", v.path.trim_end_matches(".png"));
    let mut out = format!("P5\n{w} {h}\n255\n").into_bytes();
    out.extend_from_slice(&px);
    match std::fs::write(&path, out) {
        Ok(()) => eprintln!("wrote {path}"),
        Err(e) => eprintln!("cannot write {path}: {e}"),
    }
}

// ------------------------------------------------------------------ click to inspect

pub(crate) fn ground_hit(g: &Ground, ray: Ray3d) -> Option<Vec3> {
    let dir = *ray.direction;
    let mut t = 0.0f32;
    let mut prev = ray.origin;
    for _ in 0..4000 {
        let step = 0.02 + t * 0.004;
        t += step;
        let p = ray.origin + dir * t;
        let h = world::height_at(g, p.x, p.z).max(0.0);
        if p.y <= h {
            // refine between prev and p
            let (mut a, mut b) = (prev, p);
            for _ in 0..12 {
                let m = (a + b) * 0.5;
                if m.y <= world::height_at(g, m.x, m.z).max(0.0) { b = m } else { a = m }
            }
            return Some(b);
        }
        prev = p;
        if t > 600.0 {
            break;
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn pick(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cams: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    data: Res<Data>,
    tl: Res<Timeline>,
    ui: Res<UiState>,
    mut picked: ResMut<Picked>,
    cards: Res<game::CardOpen>,
    pits: Res<pick::Pits>,
) {
    // Esc on an open card puts that off and leaves the note alone
    if keys.just_pressed(KeyCode::Escape) && !cards.0 {
        picked.what = None;
    }
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    if buttons.just_pressed(MouseButton::Left) {
        picked.press = if ui.hover { None } else { Some(cursor) };
        picked.moved = 0.0;
    }
    if buttons.pressed(MouseButton::Left) {
        picked.moved += motion.delta.length();
    }
    if !buttons.just_released(MouseButton::Left) || picked.press.is_none() || picked.moved > 5.0 {
        return;
    }
    picked.press = None;
    let Ok((cam, cam_t)) = cams.single() else { return };
    let Ok(ray) = cam.viewport_to_world(cam_t, cursor) else { return };
    let Some(hit) = ground_hit(&data.g, ray) else { return };
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    picked.what = pick::choose(&data.r, &data.g, &data.slots, &data.cen, &pits.near, hit.x, hit.z, k);
    picked.at = Some((hit.x, hit.z));
    picked.stamp = picked.stamp.wrapping_add(1);
}

/// A town's name on the map picks the town, as a click on its ground does.
#[allow(deprecated)]
fn label_clicks(q: Query<(&Interaction, &LabelPill), Changed<Interaction>>, mut picked: ResMut<Picked>, cards: Res<game::CardOpen>) {
    for (i, l) in &q {
        if *i == Interaction::Pressed && !cards.0 {
            picked.what = Some(Pick::Town(l.0));
            picked.at = None;
            picked.stamp = picked.stamp.wrapping_add(1);
        }
    }
}

/// The simulation's cell a picked thing stands on, for a map view's sentence about it.
fn pick_cell(r: &Replay, what: Pick) -> usize {
    match what {
        Pick::Firm(f) => r.header.firms[f].cell,
        Pick::Town(t) => r.header.towns[t].node,
        Pick::Pit { cell, .. } | Pick::Place { cell, .. } => cell,
    }
}

/// How many sites a works' founders priced before they built: "The best of 14 sites priced", or the one.
fn best_of(sites: i64) -> String {
    if sites == 1 { "The one site priced".to_string() } else { format!("The best of {sites} sites priced") }
}

const DISP_NAMES: [&str; 12] = ["sea", "farmland", "pasture", "woodland", "heath", "quarry", "clay pit", "iron mine", "colliery", "works", "town", "derelict ground"];

/// What a click on a works, a town, a pit or open country shows. `trades` is
/// the family trade of every method, where the game at hand says (a played
/// game's `FamilyView::method_group`; empty for a recorded run).
fn describe(r: &Replay, cen: &[(f32, f32)], what: Pick, k: usize, trades: &[Option<isostate_core::families::Group>]) -> String {
    let year = r.y0 + k as i32;
    match what {
        Pick::Firm(f) => {
            let firm = &r.header.firms[f];
            let m = &r.header.methods[firm.method];
            let good = r.header.goods.get(m.out).map(|g| g.label.to_lowercase()).unwrap_or_default();
            let kk = replay::firm_k(r, f, k);
            let size = kk / m.k0.max(1e-6);
            let since = if firm.born < r.y0 { format!("working before {}", r.y0) } else { format!("since {}", firm.born) };
            // the family trade it belongs to, which its name does not always say (a brewery is the estates')
            let trade = trades.get(firm.method).copied().flatten().map_or(String::new(), |g| format!(" ({})", family::trade(g)));
            let mut s = format!("{}\n{}{}, {}\nMakes up to {:.0} {} a month, {:.1} times a standard works.", firm.name, m.label, trade, since, kk, good, size);
            if let Some(rs) = &firm.reason {
                let num = |key: &str| rs.get(key).and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
                let town = rs.get("town").and_then(|v| v.as_i64()).and_then(|t| r.header.towns.get(t as usize)).map(|t| t.name.clone()).unwrap_or_else(|| "a nearby town".into());
                let kind = rs.get("kind").and_then(|v| v.as_str()).unwrap_or("new");
                let lead = match kind {
                    "old" => format!("Already at work in {}; built new, it would return about {:.0}% a year.", r.y0, num("roi") * 100.0),
                    "expansion" => format!("A second site, opened in {} when the first ran out of room; about {:.0}% a year expected.", num("year") as i64, num("roi") * 100.0),
                    _ => format!("Built in {} with money from {}. {}: about {:.0}% a year on the capital.", num("year") as i64, town, best_of(num("n_cand") as i64), num("roi") * 100.0),
                };
                s.push_str(&format!("\n\nWhy here?  {}\nPer unit of {}: sells at {:.2}", lead, good, num("pout")));
                if let Some(ins) = rs.get("inputs").and_then(|v| v.as_array()) {
                    for q in ins {
                        let g = q.get(0).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let a = q.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let p = q.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let label = r.header.goods.get(g).map(|g| g.label.to_lowercase()).unwrap_or_default();
                        s.push_str(&format!(", {:.2} {} at {:.2}", a, label, p));
                    }
                }
                // the fuel its kilns, furnaces and coppers burn, which for a lime kiln, a pottery or
                // a glass cone is the largest of its costs: without it the sums do not come to what is left
                if let Some(h) = rs.get("heat").and_then(|v| v.as_array()) {
                    let (a, p) = (h.first().and_then(|v| v.as_f64()).unwrap_or(0.0), h.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0));
                    if a > 0.0 {
                        s.push_str(&format!(", {:.2} of heat at {:.2}", a, p));
                    }
                }
                if let Some(l) = rs.get("labor").and_then(|v| v.as_array()) {
                    let (a, w) = (l.first().and_then(|v| v.as_f64()).unwrap_or(0.0), l.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0));
                    if a > 0.0 {
                        s.push_str(&format!(", {:.2} months of work at {:.2}", a, w));
                    }
                }
                s.push_str(&format!(", leaving {:.2}.", num("margin")));
                let displaced = rs.get("displaced").and_then(|v| v.as_str()).unwrap_or("the land");
                s.push_str(&format!("\nBid {:.2} a month for the ground, outbidding {}.", num("rent"), displaced));
            }
            s
        }
        Pick::Town(t) => {
            let town = &r.header.towns[t];
            let i = k * r.n_towns + t;
            let since = if town.founded < r.y0 { format!("a market town before {}", r.y0) } else { format!("founded {}", town.founded) };
            let mut s = format!("{}\n{}\n{} people in its market area, {} in the town, in {}.", town.name, since, fmt_pop(r.town_pop[i]), fmt_pop(r.town_urban[i]), year);
            // traffic (the core's docs/rules/traffic.md): its five largest partners, from the books
            if let Some(lines) = traffic::trade_lines(r, k, t) {
                s.push_str("\n\n");
                s.push_str(&lines);
            }
            s
        }
        Pick::Pit { cell, class } => pick::describe_pit(r, cen, cell, class, k),
        Pick::Place { x, z, cell } => pick::describe_place(r, cen, x, z, cell, k),
    }
}

fn inspect_panel(
    picked: Res<Picked>,
    tl: Res<Timeline>,
    data: Res<Data>,
    ov: Res<overlay::MapOverlay>,
    months: Res<live::Months>,
    fam: Option<Res<family::FamilyView>>,
    mut panel: Query<&mut Visibility, With<InspectPanel>>,
    mut txt: Query<(&mut Text, &mut Node), (With<InspectText>, Without<InspectTitle>)>,
    mut title: Query<&mut Text, (With<InspectTitle>, Without<InspectText>)>,
    mut body: Query<&mut bevy::ui::ScrollPosition, With<InspectBody>>,
    mut last: Local<Option<(Option<Pick>, usize, Option<overlay::Overlay>, usize, Option<usize>, Option<overlay::trend::Span>)>>,
) {
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    let key = (picked.what, k, ov.kind, ov.kind.map_or(0, |kind| ov.which(kind)), ov.month(), ov.trend_shown());
    if *last == Some(key) {
        return;
    }
    // another thing clicked is read from its top
    if last.map(|l| l.0) != Some(picked.what) {
        for mut s in &mut body {
            s.0.y = 0.0;
        }
    }
    *last = Some(key);
    for mut v in &mut panel {
        *v = if picked.what.is_some() { Visibility::Inherited } else { Visibility::Hidden };
    }
    // a firm's card is its name here, and terms and numbers under it (firm_card.rs), the map view's line among them
    if let Some(Pick::Firm(f)) = picked.what {
        for mut t in &mut title {
            t.0 = data.r.header.firms.get(f).map_or(String::new(), |x| x.name.clone());
        }
        for (mut t, mut n) in &mut txt {
            t.0.clear();
            n.display = Display::None;
        }
        return;
    }
    if let Some(w) = picked.what {
        let mut s = describe(&data.r, &data.cen, w, k, fam.as_ref().map_or(&[][..], |f| f.method_group.as_slice()));
        // with a map overlay on, what it says of the cell the thing stands on
        if let Some(line) = ov.said(&data.r, &months, k, pick_cell(&data.r, w)) {
            s.push_str("\n\n");
            s.push_str(&line);
        }
        let (head, body) = s.split_once('\n').unwrap_or((s.as_str(), ""));
        for mut t in &mut title {
            t.0 = head.to_string();
        }
        for (mut t, mut n) in &mut txt {
            t.0 = body.to_string();
            n.display = Display::Flex;
        }
    }
}

fn move_trains(time: Res<Time>, rails: Res<Rails>, mut q: Query<(&mut Train, &mut Transform)>) {
    let _frames = frames::span(frames::Sys::Traffic);
    let dt = time.delta_secs().min(0.1);
    for (mut t, mut tf) in &mut q {
        let Some(p) = rails.0.get(t.path) else { continue };
        t.s += t.dir * t.speed * dt;
        if t.s > p.len {
            t.s = p.len;
            t.dir = -1.0;
        } else if t.s < 0.0 {
            t.s = 0.0;
            t.dir = 1.0;
        }
        let (pos, yaw) = p.at(t.s);
        tf.translation = pos;
        tf.rotation = Quat::from_rotation_y(yaw + if t.dir < 0.0 { std::f32::consts::PI } else { 0.0 });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use isostate_gen::trees::LEAF_GROUPS;

    #[test]
    fn a_year_is_written_in_before_its_scenery_is_looked_at_and_handed_over() {
        // the game's systems as the game adds them, in a schedule set out but never run
        let mut app = App::new();
        game::plugin(&mut app);
        year_systems(&mut app);
        app.world_mut().schedule_scope(Update, |world, s| {
            s.initialize(world).expect("the schedule is set out");
            let g = s.graph();
            let key = |set: bevy::ecs::schedule::InternedSystemSet| *g.systems_in_set(set).unwrap().first().unwrap();
            let year = [key(game::pump.into_system_set().intern()), key(rebuild_year.into_system_set().intern()), key(handover::feed.into_system_set().intern())];
            // each pair writes something the other writes (the clock's built year, the queue), and
            // the scheduler lists every such pair it was not told the order of: none of these is listed
            for (a, b, _) in g.conflicting_systems().iter() {
                assert!(!(year.contains(a) && year.contains(b)), "the pump, rebuild_year and the feed are not all ordered against one another");
            }
            // and the order is the pump, then rebuild_year, then the feed
            let order: Vec<_> = s.systems().unwrap().map(|(k, _)| k).collect();
            let at = year.map(|k| order.iter().position(|&o| o == k).unwrap());
            assert!(at[0] < at[1] && at[1] < at[2], "{at:?}");
        });
    }

    /// Run a season clock for some seconds at 30 frames a second; the year's
    /// clock runs beside it at `speed`. Returns the clock, how far the season
    /// went in all, and the biggest move between two frames.
    fn run_seasons(c: SeasonClock, playing: bool, speed: f32, secs: f32) -> (SeasonClock, f32, f32) {
        run_dated(c, playing, speed, secs, true)
    }

    /// The same, saying whether the clock shows a month (it does while it runs).
    fn run_dated(mut c: SeasonClock, playing: bool, speed: f32, secs: f32, dated: bool) -> (SeasonClock, f32, f32) {
        let dt = 1.0 / 30.0;
        let (mut t, mut went, mut jump) = (10.0f32, 0.0f32, 0.0f32);
        for _ in 0..(secs / dt).round() as usize {
            let before = c.phase;
            if playing {
                t += speed * dt;
            }
            c.advance(playing, speed, t, dt, dated);
            let d = ((c.phase - before + 0.5).rem_euclid(1.0) - 0.5).abs();
            went += d;
            jump = jump.max(d);
        }
        (c, went, jump)
    }

    #[test]
    fn seasons_take_their_time_at_any_speed() {
        // from a season a second up, one round in SEASON_ROUND_SECS and no faster
        for speed in [0.25, 0.5, 1.0, 2.0, 4.0, 20.0] {
            let (c, went, jump) = run_seasons(SeasonClock::default(), true, speed, SEASON_ROUND_SECS);
            assert!((went - 1.0).abs() < 0.01, "at {speed} years a second the season went {went} rounds in {SEASON_ROUND_SECS} s");
            assert!(jump <= 1.0 / SEASON_ROUND_SECS / 30.0 + 1e-5, "a jump of {jump} at {speed}");
            assert!(!c.follows);
            assert!((c.phase - 0.55).abs() < 0.01, "a whole round comes back to where it began, not {}", c.phase);
        }
    }

    #[test]
    fn seasons_hold_on_a_year_gone_by_and_do_not_jump_on_resuming() {
        let (c, _, _) = run_seasons(SeasonClock::default(), true, 4.0, 5.0);
        let held = c.phase;
        // stopped where the bar shows a year and no month: the season stays where it is
        let (c, went, _) = run_dated(c, false, 4.0, 60.0, false);
        assert_eq!(c.phase, held);
        assert_eq!(went, 0.0);
        assert!(!c.follows);
        let (_, _, jump) = run_seasons(c, true, 0.5, 1.0);
        assert!(jump <= 1.0 / SEASON_ROUND_SECS / 30.0 + 1e-5, "resuming jumped {jump}");
    }

    #[test]
    fn a_stopped_clock_that_shows_a_month_shows_its_season() {
        // run fast, so the seasons are on their own round, then stop in the middle of a year
        let (c, _, _) = run_seasons(SeasonClock::default(), true, 2.0, 7.3);
        assert!(!c.follows);
        // (run_dated's year clock begins at 10.0 and does not move while stopped: midwinter)
        let (c, _, jump) = run_dated(c, false, 2.0, 3.0, true);
        assert!(c.follows);
        let off = ((c.phase + 0.5).rem_euclid(1.0) - 0.5).abs();
        assert!(off < 0.005, "the season is {} off the date three seconds after stopping", off);
        assert!(jump < 0.013, "eased with a jump of {jump}");
    }

    #[test]
    fn the_two_slowest_speeds_keep_the_seasons_on_the_date() {
        for s in clock::SPEEDS {
            let (c, _, _) = run_seasons(SeasonClock { phase: 0.0, follows: true, coarse: false }, true, s.rate, 6.0);
            // the year's clock began at 10.0 and has run for six seconds
            let date = (s.rate * 6.0).fract();
            let off = ((c.phase - date + 0.5).rem_euclid(1.0) - 0.5).abs();
            if 1.0 / s.rate >= clock::FOLLOW_YEAR_SECS - 1e-3 {
                assert!(c.follows, "{}", s.name);
                assert!(off < 0.03, "{}: the season is {off} of a year off the date", s.name);
                // what is drawn is within a step of it, and at a month a second the steps are the longer ones
                let drawn = season_phase(&c, None);
                assert!((drawn - c.phase).abs() <= 1.0 / 96.0, "{}", s.name);
                assert_eq!(c.coarse, 1.0 / s.rate < SEASON_ROUND_SECS - 1e-3, "{}", s.name);
                if c.coarse {
                    // every such step is a whole number of the 192 the ground and the sun count
                    for i in 0..96 {
                        let ph = season_phase(&SeasonClock { phase: (i as f32 + 0.6) / 96.0, follows: true, coarse: true }, None);
                        assert_eq!((ph * 192.0).floor() as i32, 2 * i);
                    }
                }
            } else {
                assert!(!c.follows, "{}", s.name);
                assert!((c.phase - 6.0 / SEASON_ROUND_SECS).abs() < 0.01, "{}: the seasons' own round", s.name);
            }
        }
    }

    #[test]
    fn a_wood_sheds_and_buds_once_a_round_at_any_speed_and_holds_when_paused() {
        // the trees go by the seasons' clock: each group of broadleaves loses its leaves once and
        // gets them back once in a round, at 1x and at 8x alike, and the leaves are gold before they fall
        let dt = 1.0 / 30.0;
        let lk = Look::standard();
        for speed in [0.5, 4.0] {
            for group in 0..LEAF_GROUPS {
                let mut c = SeasonClock::default();
                let (mut t, mut shed, mut bud, mut gold) = (10.0f32, 0, 0, false);
                assert!(in_leaf(c.phase, group), "a game opens in summer leaf");
                for _ in 0..(SEASON_ROUND_SECS / dt).round() as usize {
                    let before = in_leaf(c.phase, group);
                    t += speed * dt;
                    c.advance(true, speed, t, dt, true);
                    let now = in_leaf(c.phase, group);
                    shed += (before && !now) as usize;
                    bud += (!before && now) as usize;
                    let tint = lk.foliage.tint(c.phase);
                    gold |= now && tint[0] > 1.15 * tint[1];
                }
                assert_eq!((shed, bud), (1, 1), "group {group} at {speed} years a second");
                assert!(gold, "group {group} is never gold in leaf at {speed} years a second");
                // paused in the depth of winter, the trees stay bare
                let mut c = SeasonClock { phase: 0.02, follows: false, coarse: false };
                // (on a year gone by, where the bar shows no month)
                for _ in 0..300 {
                    c.advance(false, speed, t, dt, false);
                }
                assert!(!in_leaf(c.phase, group));
            }
        }
    }

    #[test]
    fn seasons_follow_the_year_when_the_years_go_slowly() {
        // a year in 60 seconds: the season is the date's
        let speed = 1.0 / 60.0;
        let (c, _, jump) = run_seasons(SeasonClock::default(), true, speed, 30.0);
        assert!(c.follows);
        // the year's clock began at 10.0 and has run half a year, to midsummer
        assert!((c.phase - 0.5).abs() < 0.01, "the season is {} at midsummer", c.phase);
        // easing from high summer round to midwinter took no visible jump
        assert!(jump < 0.013, "eased with a jump of {jump}");
        // stopped there (the year's clock at 10.5), the season stays on the date
        let mut p = c;
        for _ in 0..150 {
            p.advance(false, speed, 10.5, 1.0 / 30.0, true);
        }
        assert!(p.follows);
        assert!((p.phase - c.phase).abs() < 0.01, "the season moved from {} to {} while the clock was stopped on its date", c.phase, p.phase);
    }

    #[test]
    fn people_are_counted_with_a_comma_for_thousands() {
        assert_eq!(fmt_pop(0.0), "0");
        assert_eq!(fmt_pop(950.0), "950");
        assert_eq!(fmt_pop(999.6), "1,000");
        assert_eq!(fmt_pop(1_004.0), "1,004");
        assert_eq!(fmt_pop(12_999.6), "13,000");
        assert_eq!(fmt_pop(254_038.2), "254,038");
        assert_eq!(fmt_pop(999_999.7), "1.00 million");
        assert_eq!(fmt_pop(1_251_000.0), "1.25 million");
        assert_eq!(fmt_pop(f32::NAN), "0");
    }

    #[test]
    fn one_family_is_not_families() {
        assert_eq!((families_words(1), families_words(6), families_words(30)), ("1 family".to_string(), "6 families".to_string(), "30 families".to_string()));
    }

    fn news(year: f32, weight: f32, title: &str) -> replay::Event {
        replay::Event { year, kind: "family".into(), weight, title: title.into(), text: String::new(), cell: None, family: None, kept: None, seq: 0 }
    }

    #[test]
    fn toasts_keep_their_words_when_the_news_is_sorted_again() {
        let mut events = vec![news(1759.9, 0.9, "An old thing"), news(1760.7, 0.7, "Coke smelting"), news(1760.8, 0.3, "A small thing"), news(1760.95, 0.6, "A works opens")];
        // the year's major news, and nothing minor or from another year
        let shown = news_toasts(&events, 1760.0, 1761.0, events.len(), 5.0);
        let titles: Vec<&str> = shown.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, ["Coke smelting", "A works opens"]);
        assert!(shown.iter().all(|t| t.since == 5.0));
        assert_eq!((shown[0].when.as_str(), shown[1].when.as_str()), ("September 1760", "December 1760"));
        // a live game adds news dated before what is there and sorts the list, as live::apply_year does
        events.push(news(1760.1, 0.9, "News that came late"));
        events.sort_by(|a, b| a.year.total_cmp(&b.year));
        // the place a toast came from now holds something else; the toast still says what it said
        assert_eq!(events[1].title, "News that came late");
        assert_eq!(shown[0].title, "Coke smelting");
        assert_eq!(shown[1].title, "A works opens");
    }

    #[test]
    fn news_goes_up_in_its_month_or_when_it_comes_if_that_is_later() {
        let mut events = vec![news(1784.21, 0.9, "A line opens"), news(1784.55, 0.9, "A poor harvest"), news(1784.9, 0.9, "A town founded")];
        for (i, e) in events.iter_mut().enumerate() {
            e.seq = i;
        }
        let titles = |from: f32, to: f32, known: usize| news_toasts(&events, from, to, known, 0.0).iter().map(|t| t.title.clone()).collect::<Vec<_>>();
        // a year gone by, played again: each item as the clock passes its date
        assert_eq!(titles(1784.0, 1784.17, 3), Vec::<String>::new());
        assert_eq!(titles(1784.17, 1784.25, 3), ["A line opens"]);
        assert_eq!(titles(1784.25, 1784.5, 3), Vec::<String>::new());
        assert_eq!(titles(1784.5, 1784.58, 3), ["A poor harvest"]);
        // a live game: the line opened in March and was told then; the harvest and the town
        // are only known at the year's end, and go up when they come, dated as they are
        assert_eq!(titles(1784.17, 1784.25, 0), ["A line opens"]);
        let late = news_toasts(&events, 1784.99, 1784.999, 1, 0.0);
        assert_eq!(late.iter().map(|t| t.when.as_str()).collect::<Vec<_>>(), ["July 1784", "November 1784"]);
        // news that has come while the clock is still short of its date waits for the clock
        assert_eq!(titles(1784.4, 1784.5, 1), Vec::<String>::new());
        assert_eq!(titles(1784.5, 1784.6, 3), ["A poor harvest"]);
        assert_eq!(titles(1784.7, 1784.95, 3), ["A town founded"]);
        // and nothing goes up twice: what was known and is past stays down
        assert_eq!(titles(1784.95, 1784.999, 3), Vec::<String>::new());
    }

    fn args(line: &str) -> Result<Args, String> {
        let v: Vec<String> = line.split_whitespace().map(String::from).collect();
        parse(&v)
    }

    #[test]
    fn every_option_is_still_read() {
        let a = args(
            "runs/x/run.bin --year 1846 --shot out.png --size 800x600 --at 31.5,29.5 --look looks/dusk.toml --yaw 25 --pitch 38 --dist 10 --frames 12 --wait 3 \
             --no-tilt --seed 11 --families 4 --advance 20 --demo --pick 3,4.5 --ground-debug --class-map --season 0.25 --describe 7,8 \
             --view a.png,1900,77.5,47.5,25,38,10,0.75,markets --view b.png,,,,,,,none --view c.png --script print              --lab works-kinds --out out/lab --tilt --loupe 3 --turn 45",
        )
        .unwrap();
        assert_eq!((a.run.as_str(), a.named_run, a.new), ("runs/x/run.bin", true, false));
        assert_eq!((a.year, a.shot.as_deref(), a.size), (Some(1846), Some("out.png"), (800, 600)));
        assert_eq!((a.at, a.look_file.as_deref(), a.yaw, a.pitch, a.dist), (Some((31.5, 29.5)), Some("looks/dusk.toml"), 25.0, 38.0, Some(10.0)));
        assert_eq!((a.frames, a.wait, a.tilt), (12, Some(3.0), false));
        assert_eq!((a.lab.as_deref(), a.out.as_deref(), a.lab_tilt, a.loupe, a.lab_turn), (Some("works-kinds"), Some("out/lab"), true, 3, Some(45.0)));
        assert_eq!((a.seed, a.families, a.advance, a.demo), (11, 4, 20, true));
        assert_eq!((a.pick, a.describe), (Some((3.0, 4.5)), Some((7.0, 8.0))));
        assert_eq!((a.ground_debug, a.class_map, a.season, a.script.as_deref()), (true, true, Some(0.25), Some("print")));
        assert_eq!(a.views.len(), 3);
        let v = &a.views[0];
        assert_eq!((v.path.as_str(), v.year, v.look, v.yaw, v.pitch, v.dist), ("a.png", Some(1900), Some((77.5, 47.5)), 25.0, 38.0, Some(10.0)));
        assert_eq!((v.season, v.tab), (Some(0.75), Some(Some(ui::Tab::Markets))));
        // empty values fall back; "none" closes the panel
        let v = &a.views[1];
        assert_eq!((v.year, v.look, v.dist, v.season, v.tab), (None, None, None, None, Some(None)));
        assert_eq!((a.views[2].path.as_str(), a.views[2].tab), ("c.png", None));
        // a view's words: a panel, a map view and what the Markets panel shows, in any order
        let a = args("--overlay price:coal --view a.png,1850,,,,,,markets,price:coal,open:coal --view b.png,1850,,,,,,wages,0.0,island").unwrap();
        assert_eq!(a.overlay.as_deref(), Some("price:coal"));
        let v = &a.views[0];
        assert_eq!((v.tab, v.overlay.as_deref(), v.markets.as_deref()), (Some(Some(ui::Tab::Markets)), Some("price:coal"), Some("open:coal")));
        let v = &a.views[1];
        assert_eq!((v.tab, v.overlay.as_deref(), v.markets.as_deref(), v.season), (None, Some("wages"), Some("island"), Some(0.0)));
        // the standard shots play the whole game first
        let a = args("--advance 20 --shots out/shots --shots-compare out/before --shots-note abc123").unwrap();
        assert_eq!((a.shots.as_deref(), a.shots_compare.as_deref(), a.shots_note.as_str(), a.advance), (Some("out/shots"), Some("out/before"), "abc123", usize::MAX - 1));
        // a scripted run takes its own pictures
        let a = args("--advance 20 --shots out/shots --script print").unwrap();
        assert_eq!((a.shots, a.advance), (None, 20));
        // two numbers after --look are the point to look at, which --look used to mean
        let a = args("--compare-shots out/a out/b --look 31.5,29.5").unwrap();
        assert_eq!((a.compare_shots, a.at, a.look_file), (Some(("out/a".to_string(), "out/b".to_string())), Some((31.5, 29.5)), None));
    }

    #[test]
    fn no_run_named_means_a_new_game() {
        let a = args("").unwrap();
        assert!(a.new && !a.named_run);
        assert_eq!((a.seed, a.families, a.advance, a.size, a.pitch, a.frames, a.wait, a.tilt), (7, 6, 0, (1600, 1000), shots::PLAY_PITCH, 40, None, true));
        assert!(args("--new --seed 3").unwrap().new);
        // a run and --new together: a new game, as before
        assert!(args("runs/x/run.bin --new").unwrap().new);
        // nothing asked for but the game: the island's saved game may be gone on with
        assert!(!a.fresh && a.load.is_none() && a.save.is_none());
        // --new and --seed ask for a new game whatever is saved
        assert!(args("--new").unwrap().fresh && args("--seed 3").unwrap().fresh);
        // --old-land is a new game too, on the old island, even beside a recorded run; and not a saved one
        let a = args("--old-land --seed 3").unwrap();
        assert!(a.new && a.fresh && a.old_land && a.seed == 3);
        assert!(args("runs/x/run.bin --old-land").unwrap().new && !args("--new").unwrap().old_land);
        assert!(args("--old-land --load mine.json").err().unwrap().starts_with("--old-land starts a new game, and --load goes on with one"));
        // the start's two switches ask for a new game, on or off, and not a saved one
        let a = args("--landfall --settlers off --ideas-by-sea on").unwrap();
        assert!(a.new && a.fresh && a.landfall);
        assert_eq!(a.switches, isostate_core::save::Switches { settlers: Some(false), ideas_by_sea: Some(true) });
        assert_eq!(args("--ideas-by-sea off").unwrap().switches, isostate_core::save::Switches { settlers: None, ideas_by_sea: Some(false) });
        assert!(args("--landfall").unwrap().switches.plain() && args("--new").unwrap().switches.plain());
        assert_eq!(args("--settlers no").err().unwrap(), "--settlers takes on or off, not no");
        assert_eq!(args("--settlers").err().unwrap(), "--settlers needs a value after it");
        assert!(args("--settlers off --load mine.json").err().unwrap().starts_with("--settlers and --ideas-by-sea begin a new game, and --load goes on with one"));
        // --new-land does nothing since every new game is on the new land: a command written before still runs
        let a = args("--new-land").unwrap();
        assert!(!a.fresh && !a.old_land && a.load.is_none());
        assert!(args("--new-land --load mine.json").is_ok());
        // a saved game is a game, even beside a recorded run
        let a = args("runs/x/run.bin --load mine.json --save other.json").unwrap();
        assert_eq!((a.new, a.fresh, a.load.as_deref(), a.save.as_deref()), (true, false, Some("mine.json"), Some("other.json")));
    }

    #[test]
    fn a_plain_start_is_a_sandbox_and_the_family_game_is_asked_for() {
        use live::Keeping;
        // what opens, whether it is a plain start (which goes on with the island's session or autosave), and where it is kept
        let opens = |line: &str| {
            let a = args(line).unwrap();
            let o = opening(&a);
            (a.new, a.sandbox, o.plain(), o.keeping())
        };
        // nobody asked for anything: a sandbox, gone on with from the island's session and kept there
        assert_eq!(opens(""), (true, true, true, Keeping::Session));
        // any other sandbox start makes the island anew, and keeps it in the session all the same
        for line in ["--seed 7", "--advance 40", "--new", "--sandbox --new"] {
            assert_eq!(opens(line), (true, true, false, Keeping::Session), "{line}");
        }
        // the family game: a plain start, going on with the island's autosave as a plain start did before
        assert_eq!(opens("--family"), (true, false, true, Keeping::Autosave));
        // nobody at it: the family game unless a sandbox is asked for, and kept nowhere either way
        // (a script goes on with an autosave where ISOSTATE_SAVES names a folder, which other tests in this
        // program set, so whether it is a plain start is not asked here)
        let script = |line: &str| {
            let (new, sandbox, _, kept) = opens(line);
            (new, sandbox, kept)
        };
        assert_eq!(script("--script print"), (true, false, Keeping::Nowhere));
        assert_eq!(script("--script print --sandbox"), (true, true, Keeping::Nowhere));
        // unless --keep asks for the session (refused where ISOSTATE_SAVES names no folder: `keep_refused`)
        assert_eq!(script("--script print --sandbox --keep"), (true, true, Keeping::Session));
        assert_eq!(script("--script print --keep"), (true, false, Keeping::Autosave));
        assert!(!opens("--script print --sandbox --advance 40").2, "years played first are never a plain start");
        for line in ["--shots out/shots", "--land-shots out/land", "--shot a.png", "--view a.png,1800", "--describe 30,30", "--class-map --view a.png"] {
            assert_eq!(opens(line), (true, false, false, Keeping::Nowhere), "{line}");
        }
        assert_eq!(opens("--describe 30,30 --sandbox"), (true, true, false, Keeping::Nowhere));
        // the demo is the family's orders
        assert_eq!(opens("--demo"), (true, false, false, Keeping::Trial("--demo")));
        assert_eq!(opens("--demo --sandbox"), (true, true, false, Keeping::Session));
        // a saved game opens in whichever kind of game was asked for, and is kept as that kind is
        assert_eq!(opens("--load mine.json"), (true, true, false, Keeping::Session));
        assert_eq!(opens("--load mine.json --family"), (true, false, false, Keeping::Autosave));
        // years played first: a sandbox still, as --advance asks nothing of who plays; the family game's is a trial
        assert_eq!(opens("--family --advance 40"), (true, false, false, Keeping::Trial("--advance")));
        // a place that was named is the place, either way
        assert_eq!(opens("--save s.json"), (true, true, false, Keeping::Named("s.json".into())));
        // both at once is refused in a sentence
        assert_eq!(args("--family --sandbox").err().unwrap(), "--family plays the family game and --sandbox opens a game nobody plays a family in: give one of them, not both");
        // a recorded run is neither
        let a = args("RUN.bin").unwrap();
        assert!(!a.new && !a.sandbox && !a.family);
        assert!(!args("RUN.bin --sandbox").unwrap().sandbox);
        // and a new game runs to 2050 whichever it is
        assert_eq!((opening(&args("").unwrap()).to, opening(&args("--family").unwrap()).to), (Some(2050), Some(2050)));
    }

    #[test]
    fn an_option_that_is_not_one_is_refused() {
        assert_eq!(args("--sesaon 0.5").err().unwrap(), "--sesaon is not an option");
        assert_eq!(args("--advance 30 --veiw x.png").err().unwrap(), "--veiw is not an option");
        assert_eq!(args("-x").err().unwrap(), "-x is not an option");
        // an option with nothing after it
        assert_eq!(args("--advance").err().unwrap(), "--advance needs a value after it");
        // a negative number after an option is its value, not an option
        assert_eq!(args("--yaw -30").unwrap().yaw, -30.0);
        // a value that cannot be read is refused, not replaced by the default
        assert_eq!(args("--advance 3O").err().unwrap(), "--advance: 3O is not a number it can take");
        assert_eq!(args("--advance -5").err().unwrap(), "--advance: -5 is not a number it can take");
        assert_eq!(args("--season summer").err().unwrap(), "--season: summer is not a number it can take");
        assert_eq!(args("--wait nan").err().unwrap(), "--wait: nan is not a number it can take");
        assert_eq!(args("--year 1846.5").err().unwrap(), "--year: 1846.5 is not a number it can take");
        assert_eq!(args("--describe 12").err().unwrap(), "--describe: 12 should be two numbers with a comma between them");
        assert_eq!(args("--at 31.5,here").err().unwrap(), "--at: here is not a number it can take");
        assert_eq!(args("--at 31.5").err().unwrap(), "--at: 31.5 should be two numbers with a comma between them");
        // --look takes a look file, or two numbers as --at does; half a point is refused
        assert_eq!(args("--look 31.5,here").err().unwrap(), "--look: here is not a number it can take");
        assert_eq!(args("--look").err().unwrap(), "--look needs a value after it");
        // the options of the standard shots and the lab
        assert_eq!(args("--loupe big").err().unwrap(), "--loupe: big is not a number it can take");
        assert_eq!(args("--loupe 20").unwrap().loupe, 8);
        assert_eq!(args("--turn left").err().unwrap(), "--turn: left is not a number it can take");
        assert_eq!(args("--shots").err().unwrap(), "--shots needs a value after it");
        assert_eq!(args("--compare-shots out/a").err().unwrap(), "--compare-shots needs two directories after it");
        assert_eq!(args("--lab").err().unwrap(), "--lab needs a value after it");
        // map views (a good the run does not have is refused before a game's island is made, or once a recorded run is read: tests/options.rs)
        assert_eq!(args("--overlay rents").err().unwrap(), format!("--overlay: rents is not a map view: {}", overlay::VIEWS_WORDS));
        assert!(args("--overlay rent+ever").is_err() && args("--overlay made:coal+30").is_ok() && args("--overlay deposits+all").is_ok());
        assert_eq!(args("--overlay").err().unwrap(), "--overlay needs a value after it");
        // a script's steps are read with the options
        assert_eq!(args("--script bogus").err().unwrap(), "--script: bogus: not a step (see the top of script.rs)");
        assert_eq!(args("--script key:Nope").err().unwrap(), "--script: key:Nope: no key called Nope");
        // saved games
        assert_eq!(args("--load").err().unwrap(), "--load needs a value after it");
        assert_eq!(args("--new --save").err().unwrap(), "--save needs a value after it");
        // the option after --load or --save is not the file's name (--size was once taken for it)
        assert_eq!(
            args("--load --size 800x600").err().unwrap(),
            "--load needs the name of a saved game after it, and --size is another option: put the name between them (a name that begins with dashes is written ./--size)"
        );
        assert!(args("--new --save --advance 3").err().unwrap().starts_with("--save needs the name of a file to keep the game in after it, and --advance is another option"));
        assert!(args("--shots --wait 3").err().unwrap().starts_with("--shots needs a directory after it, and --wait is another option"));
        // nor after --shot, --look, --view or --shots-note: --shot --new once took --new for the picture's name
        assert_eq!(
            args("--shot --new").err().unwrap(),
            "--shot needs the name of a picture after it, and --new is another option: put the name between them (a name that begins with dashes is written ./--new)"
        );
        assert!(args("--look --advance 3").err().unwrap().starts_with("--look needs the name of a look file after it, and --advance is another option"));
        assert!(args("--view --shot x.png").err().unwrap().starts_with("--view needs a picture's name and its view after it, and --shot is another option"));
        // (a note is a line of text, not a name: no ./ is offered for it)
        assert_eq!(
            args("--shots out --shots-note --shots-compare before").err().unwrap(),
            "--shots-note needs a line of text after it, and --shots-compare is another option: put the line between them"
        );
        assert_eq!(args("--shots out --shots-note").err().unwrap(), "--shots-note needs a value after it");
        let empty: Vec<String> = ["--shots", "out", "--shots-note", ""].map(String::from).to_vec();
        assert_eq!(parse(&empty).err().unwrap(), "--shots-note needs a line of text after it");
        assert_eq!(args("--shot ./--new.png").unwrap().shot.as_deref(), Some("./--new.png"));
        assert_eq!(args("--load ./--odd.json").unwrap().load.as_deref(), Some("./--odd.json"));
        assert_eq!(args("--load -odd.json").unwrap().load.as_deref(), Some("-odd.json"));
        // a number of families no game can have is refused with the options, before an island is made
        assert_eq!(args("--families 0").err().unwrap(), "--families: a game has from 1 to 30 families, not 0; give a number in that range");
        assert_eq!(args("--families 31").err().unwrap(), "--families: a game has from 1 to 30 families, not 31; give a number in that range");
        assert_eq!(args("--families 4000000000").err().unwrap(), "--families: a game has from 1 to 30 families, not 4000000000; give a number in that range");
        assert_eq!((args("--families 1").unwrap().families, args("--families 30").unwrap().families), (1, 30));
        // and so is a last year no game can be played to; with none asked for, a new game runs to 2050
        // (the default is the Opening's), and the standard pictures always play a game to 1900
        assert_eq!(args("--to 2051").err().unwrap(), "--to: a game is played to a year from 1900 to 2050, not 2051; give a year in that range");
        assert_eq!(args("--to 1899").err().unwrap(), "--to: a game is played to a year from 1900 to 2050, not 1899; give a year in that range");
        assert_eq!(args("--to").err().unwrap(), "--to needs a value after it");
        assert_eq!((args("--to 1950").unwrap().to, args("--new").unwrap().to), (Some(1950), None));
        assert_eq!((args("--shots out --to 2050").unwrap().to, args("--shots out").unwrap().to), (Some(1900), Some(1900)));
        assert_eq!(args("--seed seven").err().unwrap(), "--seed: seven is not a number it can take");
        assert_eq!(args("--size 800").err().unwrap(), "--size: 800 should be two numbers with an x between them");
        assert_eq!(args("--size 0x600").err().unwrap(), "--size: a picture needs a width and a height");
        // in a view an empty value falls back, and one that is not a number is refused
        assert_eq!(args("--view a.png,l850").err().unwrap(), "--view: l850 is not a number it can take");
        assert_eq!(args("--view a.png,1850,markets").err().unwrap(), "--view: markets is not a number it can take");
        assert!(args("--view a.png,,,,,,").is_ok());
        // a word in a view that is no panel, no map view and nothing the Markets panel shows is
        // refused: "rents" was once read as no panel, and the picture taken of the plain map
        assert_eq!(
            args("--view a.png,1850,,,,,,rents").err().unwrap(),
            "--view: rents is not a word a view takes: use a panel (family, holdings, markets, news, hand, rules, families or none), a map view as for --overlay, or island or open:GOOD for the Markets panel"
        );
        assert!(args("--view a.png,1850,,,,,,wages,famly").err().unwrap().starts_with("--view: famly is not a word a view takes"));
        assert!(args("--view a.png,1850,,,,,,open:months").err().unwrap().starts_with("--view: open:months is not a word a view takes"));
        for words in ["none", "None", "Family,price:coal", "news,island:months", "open:Coal:months", "holdings,plain,0.5"] {
            assert!(args(&format!("--view a.png,1850,,,,,,{words}")).is_ok(), "{words}");
        }
        // asking for the list
        assert_eq!(args("--help").err().unwrap(), "");
        // the list names every option the reader knows
        for opt in ["--new", "--seed", "--old-land", "--landfall", "--settlers", "--ideas-by-sea", "--load", "--save", "--keep", "--sandbox", "--family", "--families", "--to", "--advance", "--demo", "--year", "--size", "--at", "--look", "--yaw", "--pitch", "--dist", "--no-tilt", "--season", "--pick", "--overlay", "--describe", "--view", "--shot", "--wait", "--frames", "--ground-debug", "--class-map", "--script", "--shots", "--land-shots", "--shots-compare", "--shots-note", "--compare-shots", "--lab", "--out", "--tilt", "--loupe", "--turn", "--version", "--help", "--start-page", "--page-shot"] {
            assert!(USAGE.contains(opt), "the usage text leaves out {opt}");
        }
    }

    #[test]
    fn the_version_is_the_workspaces_with_its_commit() {
        // the release command (tools/release.py) reads the version from Cargo.toml and checks
        // that the program it built says the same, with the commit it was built from
        let line = version_line();
        assert!(line.starts_with(&format!("Isolated State {} (commit ", env!("CARGO_PKG_VERSION"))), "{line}");
        assert!(!note::COMMIT.is_empty() && line.contains(note::COMMIT), "{line}");
        assert_eq!(VERSION.split('.').count(), 3, "a version is three numbers: {VERSION}");
    }

    #[test]
    fn the_trees_stand_aside_for_a_map_view_whatever_the_season_does() {
        // the three meshes of each group of trees, as setup_world spawns them, and the two
        // things that bear on which of them show
        let mut app = App::new();
        app.insert_resource(SeasonClock { phase: 0.5, follows: false, coarse: false })
            .insert_resource(SeasonMats { terrain: Handle::default(), trees: Handle::default(), rivers: Handle::default(), built: [Handle::default(), Handle::default()], smoke: Handle::default(), forced: None })
            .insert_resource(overlay::MapOverlay::new(None, 0))
            .insert_resource(GroundDebug(false))
            .add_systems(Update, show_trees);
        for group in 0..LEAF_GROUPS {
            app.world_mut().spawn((Evergreen, Visibility::default()));
            app.world_mut().spawn((Leaves(group), Visibility::default()));
            app.world_mut().spawn((BareLimbs(group), Visibility::Hidden));
        }
        let showing = |app: &mut App| trees_showing(app.world_mut());
        let all = 3 * LEAF_GROUPS;
        // high summer on the plain map: what is evergreen and every group's leaves
        app.update();
        assert_eq!(showing(&mut app), (2 * LEAF_GROUPS, all));
        // a map view goes on: every mesh of the trees is hidden, the conifers and the bare limbs too
        app.world_mut().resource_mut::<overlay::MapOverlay>().set(Some(overlay::Overlay::Rent));
        app.update();
        assert_eq!(showing(&mut app), (0, all), "trees show under a map view");
        // the season moves on under the view, right round the year: they stay hidden (the
        // seasons used to put each group's leaves or limbs back as it turned)
        for step in 0..=48 {
            app.world_mut().resource_mut::<SeasonClock>().phase = (0.5 + step as f32 / 48.0).rem_euclid(1.0);
            app.update();
            assert_eq!(showing(&mut app), (0, all), "trees came back under a map view when the season reached {}", step as f32 / 48.0);
        }
        // a season held for a picture does not bring them back either
        app.world_mut().resource_mut::<SeasonMats>().forced = Some(0.0);
        app.update();
        assert_eq!(showing(&mut app), (0, all));
        // the plain map again, in the midwinter the picture holds: evergreens and bare limbs
        app.world_mut().resource_mut::<overlay::MapOverlay>().set(None);
        app.update();
        assert_eq!(showing(&mut app), (2 * LEAF_GROUPS, all));
        let mut leaves = app.world_mut().query::<(&Leaves, &Visibility)>();
        assert!(leaves.iter(app.world()).all(|(_, v)| *v == Visibility::Hidden), "no tree is in leaf at midwinter");
        // and `--ground-debug` shows the ground alone
        app.world_mut().resource_mut::<GroundDebug>().0 = true;
        app.update();
        assert_eq!(showing(&mut app), (0, all));
    }

    #[test]
    fn each_part_of_a_tree_shows_in_its_season() {
        for group in 0..LEAF_GROUPS {
            for i in 0..96 {
                let ph = i as f32 / 96.0;
                // one or the other of a group's leaves and limbs, never both and never neither
                assert_ne!(tree_part_shown(TreePart::Leaves(group), ph, false), tree_part_shown(TreePart::BareLimbs(group), ph, false));
                assert!(tree_part_shown(TreePart::Evergreen, ph, false));
                for part in [TreePart::Evergreen, TreePart::Leaves(group), TreePart::BareLimbs(group)] {
                    assert!(!tree_part_shown(part, ph, true));
                }
            }
            assert!(tree_part_shown(TreePart::Leaves(group), 0.5, false) && tree_part_shown(TreePart::BareLimbs(group), 0.0, false));
        }
    }

    #[test]
    fn a_town_older_than_the_run_is_dated_by_the_run_s_first_year() {
        // Ashby was founded in 1700; a run that begins in 1800 says so, and not 1750
        let r = replay::tiny(1800);
        let said = describe(&r, &[], Pick::Town(0), 0, &[]);
        assert!(said.starts_with("Ashby\na market town before 1800\n"), "{said}");
        assert!(describe(&replay::tiny(1750), &[], Pick::Town(0), 0, &[]).contains("a market town before 1750"));
    }

    #[test]
    fn a_forced_season_wins() {
        let c = SeasonClock { phase: 0.3, follows: false, coarse: false };
        assert_eq!(season_phase(&c, None), 0.3);
        assert_eq!(season_phase(&c, Some(0.0)), 0.0);
        assert_eq!(season_phase(&c, Some(1.25)), 0.25);
    }
}
