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

mod boats;
mod carts;
mod chart;
mod cursor;
mod family;
mod fences;
mod fields;
mod game;
mod icons;
mod lab;
mod live;
mod look;
mod meshkit;
mod network;
mod overlay;
mod plan;
mod rails;
mod replay;
mod rivers;
mod saves;
mod scatter;
mod script;
mod shots;
mod terrain;
mod trains;
mod ui;
mod world;

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
use bevy::tasks::{futures::check_ready, AsyncComputeTaskPool, Task};
use std::sync::Arc;
use bevy::window::{ExitCondition, WindowResolution};
use bevy::winit::WinitPlugin;

use isostate_gen::trees::{in_leaf, LEAF_GROUPS};
use look::{Look, TheLook, ToColor};
use replay::Replay;
use ui::{kind_colour, kind_label, text, UiState, INK, MUTED, PAPER};
use world::Ground;

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
    /// seconds to wait before the first screenshot (`--wait`)
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
    /// a new game was asked for (--new or --seed): the island's autosave is not gone on with
    fresh: bool,
    /// a saved game to go on with
    load: Option<String>,
    /// where to save, instead of the island's autosave
    save: Option<String>,
    /// start with this spot clicked
    pick: Option<(f32, f32)>,
    ground_debug: bool,
    class_map: bool,
    /// render the standard shots into this directory, then stop (shots.rs)
    shots: Option<String>,
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

With no run named, a new game starts.

  --new                a new game, whatever is saved (with no run named and nothing else
                       asked for, the island's saved game is gone on with)
  --seed N             which island (7); a new game
  --load FILE          go on with this saved game
  --save FILE          keep the game in FILE instead of the island's autosave
  --families N         how many great families (6)
  --advance N          play N years before the game opens
  --demo               the family builds a works and buys land in its first year
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
  --overlay NAME       start with a map view: rent, areas, reach, price, price:GOOD, wages
                       or plain
  --describe X,Z       print what a click there would show, and stop

Pictures and checks with no window:

  --view PATH,YEAR,X,Z,YAW,PITCH,DIST[,SEASON][,WORDS]
                       a picture; give it once for each. Empty values fall back to
                       the opening view. WORDS, in any order: a panel (family, holdings,
                       markets, news or none), a map view as for --overlay, and island
                       or open:GOOD for what the Markets panel shows
  --shot PATH          a picture of the opening view
  --wait S             seconds before the first picture, for shaders to compile (45)
  --frames N           frames to let a picture settle (40)
  --ground-debug       draw the ground as flat class colours and nothing else
  --class-map          write the CPU's land class under every pixel of each view, and stop
  --script STEPS       play keys and clicks and print what happens (see src/script.rs)

The standard shots (src/shots.rs) and the lab (src/lab.rs):

  --shots DIR          play the whole game, take the standard shots into DIR, and stop
  --shots-compare DIR  with --shots: set each beside the shot of the same name in DIR
  --shots-note TEXT    with --shots: a line for the sheet, such as the commit
  --compare-shots A B  compare two sets of standard shots already taken, and stop
  --lab SHEET          contact sheets of the scenery's generators (a sheet, all, or list), and stop
  --out DIR            with --lab: where its pictures go
  --tilt               with --lab: through the miniature focus
  --loupe N            with --lab: enlarge the close picture N times (1 to 8)
  --turn D             with --lab: turn each thing D degrees about its middle

  --help               this";

fn parse_args() -> Args {
    let v: Vec<String> = std::env::args().skip(1).collect();
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

/// Two numbers with `sep` between them, as in --look 31.5,29.5 and --size 1600x1000.
fn pair<T: std::str::FromStr>(opt: &str, s: &str, sep: char) -> Result<(T, T), String> {
    match s.split_once(sep) {
        Some((a, b)) => Ok((number(opt, a)?, number(opt, b)?)),
        None => Err(format!("{opt}: {s} should be two numbers with {} between them", if sep == 'x' { "an x" } else { "a comma" })),
    }
}

/// A value that names something (a directory, a sheet): it must be there.
fn named(opt: &str, s: String, what: &str) -> Result<String, String> {
    if s.trim().is_empty() { Err(format!("{opt} needs {what} after it")) } else { Ok(s) }
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
        load: None,
        save: None,
        pick: None,
        ground_debug: false,
        class_map: false,
        shots: None,
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
            "--shot" => { a.shot = Some(next(i)?); i += 1; }
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
                    _ if s.is_empty() => return Err(format!("{opt} needs the name of a look file")),
                    _ => a.look_file = Some(s),
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
            "--load" => { a.load = Some(named(opt, next(i)?, "the name of a saved game")?); i += 1; }
            "--save" => { a.save = Some(named(opt, next(i)?, "the name of a file to keep the game in")?); i += 1; }
            "--families" => { a.families = number(opt, &next(i)?)?; i += 1; }
            "--advance" => { a.advance = number(opt, &next(i)?)?; i += 1; }
            "--demo" => a.demo = true,
            "--pick" => { a.pick = Some(pair(opt, &next(i)?, ',')?); i += 1; }
            "--ground-debug" => a.ground_debug = true,
            "--class-map" => a.class_map = true,
            "--overlay" => {
                let w = next(i)?;
                if !overlay::is_word(&w) {
                    return Err(format!("{opt}: {w} is not a map view: use rent, areas, reach, price, price:GOOD, wages or plain"));
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
            "--shots-compare" => { a.shots_compare = Some(named(opt, next(i)?, "a directory")?); i += 1; }
            "--shots-note" => { a.shots_note = next(i)?; i += 1; }
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
                let p: Vec<String> = next(i)?.split(',').map(String::from).collect();
                // an empty value falls back; one that is not a number is refused
                let num = |j: usize| -> Result<Option<f32>, String> {
                    match p.get(j).map(|s| s.trim()) {
                        None | Some("") => Ok(None),
                        Some(s) => number(opt, s).map(Some),
                    }
                };
                let words: Vec<&String> = p.iter().skip(7).filter(|s| !s.is_empty() && s.parse::<f32>().is_err()).collect();
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
    }
    Ok(a)
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
    /// the trees, a group to each: what is green all year, the leaves, and the bare limbs
    trees: Vec<[Handle<Mesh>; 3]>,
    houses: Handle<Mesh>,
    works: Handle<Mesh>,
    smoke: Handle<Mesh>,
    roads: Handle<Mesh>,
    rails: Handle<Mesh>,
    lanes: Handle<Mesh>,
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
    start: std::time::Instant,
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
    year: i32,
    /// when it went up, in seconds since the app started
    since: f32,
}

#[derive(Resource, Default)]
struct Toasts {
    shown: Vec<Toast>,
    last_k: Option<usize>,
}

/// The major news of a year, to go up as toasts.
fn year_toasts(events: &[replay::Event], year: i32, now: f32) -> Vec<Toast> {
    events
        .iter()
        .filter(|e| e.year.floor() as i32 == year && e.major())
        .map(|e| Toast { kind: e.kind.clone(), title: e.title.clone(), year, since: now })
        .collect()
}

#[derive(Resource)]
struct UiFont(Handle<Font>);

#[derive(Resource)]
struct Tilt(bool);

/// The leaves of one group of the broadleaf trees, shown while that group is
/// in leaf, and its bare limbs, shown while it is not.
#[derive(Component)]
struct Leaves(usize);

#[derive(Component)]
struct BareLimbs(usize);

/// Materials that change with the seasons.
#[derive(Resource)]
struct SeasonMats {
    terrain: Handle<terrain::GroundMaterial>,
    trees: Handle<StandardMaterial>,
    rivers: Handle<StandardMaterial>,
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
#[derive(Component)]
struct InspectPanel;

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
}

#[derive(Clone, Copy, PartialEq)]
enum Pick {
    Firm(usize),
    Town(usize),
    Cell(usize),
}

static FREDOKA: &[u8] = include_bytes!("../assets/fonts/Fredoka.ttf");

fn main() {
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
    let t0 = std::time::Instant::now();
    let (r, live, families) = if args.new {
        let opening = live::Opening {
            seed: args.seed,
            families: args.families,
            advance: args.advance,
            demo: args.demo,
            load: args.load.clone(),
            save: args.save.clone(),
            fresh: args.fresh,
            // the standard shots are pictures too; and a scripted run is a check, not the player's
            // game, so it leaves the autosave alone unless ISOSTATE_SAVES names a folder to keep one in
            shots: !args.views.is_empty()
                || args.shot.is_some()
                || args.describe.is_some()
                || args.class_map
                || args.shots.is_some()
                || (args.script.is_some() && std::env::var_os("ISOSTATE_SAVES").is_none_or(|v| v.is_empty())),
        };
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
    eprintln!("prepared {} in {:.1}s", if args.new { "the game" } else { args.run.as_str() }, t0.elapsed().as_secs_f32());

    if let Some(me) = families.as_ref().and_then(|f| f.player()) {
        let at = r.header.towns.get(me.seat).map(|t| cen[t.node]).unwrap_or((0.0, 0.0));
        eprintln!("you are the {} family of {} ({:.1},{:.1}), with {} in hand", me.name, me.seat_name, at.0, at.1, ui::money(me.cash));
        for fr in families.as_ref().map(|f| f.families.as_slice()).unwrap_or(&[]) {
            if fr.rep.index != me.rep.index {
                let at = r.header.towns.get(fr.seat).map(|t| cen[t.node]).unwrap_or((0.0, 0.0));
                eprintln!("  the {} family of {} ({:.1},{:.1})", fr.name, fr.seat_name, at.0, at.1);
            }
        }
        for &f in &me.rep.firms {
            let firm = &r.header.firms[f];
            eprintln!("  {} at {},{}", firm.name, firm.cell % r.w, firm.cell / r.w);
        }
    }
    // an overlay word that cannot be shown is refused, so a mistyped shot is not taken for the plain map
    // (a word that is no map view was refused with the options; a good is known only with the run)
    for w in args.overlay.iter().chain(args.views.iter().filter_map(|v| v.overlay.as_ref())) {
        if let Some(why) = overlay::complaint(w, &r) {
            eprintln!("--overlay: {why}");
            std::process::exit(2);
        }
    }
    let latest = r.filled.clamp(1, r.ny) - 1;
    let k0 = match args.year {
        Some(y) => ((y - r.y0).max(0) as usize).min(latest),
        None => latest,
    };
    if let Some((x, z)) = args.describe {
        // print what a click at this spot would show, then stop
        let hit = Vec3::new(x, world::height_at(&g, x, z), z);
        match choose(&r, &slots, &cen, hit, k0) {
            Some(p) => {
                println!("{}", describe(&r, p, k0));
                // with --overlay, what that overlay says of the spot, as the inspector would
                if let Some((Some(kind), good)) = args.overlay.as_deref().and_then(|w| overlay::parse(w, &r)) {
                    if let Some(line) = overlay::said(&r, k0, pick_cell(&r, p), kind, good.unwrap_or(0)) {
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
        what: args.pick.and_then(|(x, z)| choose(&r, &slots, &cen, Vec3::new(x, world::height_at(&g, x, z), z), k0)),
        ..default()
    };

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
        let standard = shots::standard_views(&r, &cen, &slots, seat, shots::island_view(&r, args.size), dir);
        // pictures of an earlier set in this directory would be taken for this run's
        shots::clear_pictures(&standard);
        views = standard.iter().map(|s| s.spec.clone()).collect();
        views.sort_by_key(|v| v.year);
        let mut what = if args.new {
            format!("Island {} with {} families, {} by {} pixels", args.seed, args.families, args.size.0, args.size.1)
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
                title: "Isolated State".into(),
                resolution: WindowResolution::new(args.size.0, args.size.1),
                ..default()
            }),
            ..default()
        }));
    }
    terrain::register(&mut app);
    let (start_overlay, start_good) = args.overlay.as_deref().and_then(|w| overlay::parse(w, &r)).unwrap_or((None, None));
    app.insert_resource(overlay::MapOverlay::new(start_overlay, start_good.unwrap_or(0)));
    app.insert_resource(ClearColor(look.sky.background.color()))
        .insert_resource(GlobalAmbientLight { color: look.sky.light.color(), brightness: look.sky.brightness, ..default() })
        .insert_resource(Data { r: Arc::new(r), g: Arc::new(g), slots: Arc::new(slots), cen: Arc::new(cen), live: live.is_some() })
        .init_resource::<Builds>()
        .insert_resource(Timeline { t: k0 as f32, built: None, playing: false, speed: 2.0 })
        .insert_resource(orbit)
        .insert_resource(Tilt(args.tilt && look.tilt_shift.on))
        .insert_resource(TheLook(look))
        .insert_resource(ForcedSeason(args.season))
        .init_resource::<SeasonClock>()
        .insert_resource(GroundDebug(args.ground_debug))
        .init_resource::<Toasts>()
        .insert_resource(picked)
        .init_resource::<Rails>()
        .add_plugins(ui::plugin)
        .add_plugins(game::plugin)
        .add_plugins(overlay::plugin)
        .add_plugins(cursor::plugin)
        .add_plugins(boats::plugin)
        .add_plugins(carts::plugin)
        .add_systems(Startup, (setup_world, setup_ui, icons::setup).chain())
        .add_systems(Update, (ui::ui_hover, time_input, advance_time, rebuild_year, move_trains, seasons, camera_input, apply_camera, labels, toasts, pick, inspect_panel).chain());
    // the seasons' clock is read by the ground, the trees, the sun and the date alike
    app.add_systems(Update, season_clock.after(advance_time).before(seasons));
    if let Some(l) = live {
        app.insert_resource(l);
    }
    if let Some(f) = families {
        app.insert_resource(f);
    }
    if args.ground_debug {
        app.add_systems(Update, hide_all_but_ground.after(rebuild_year));
    }
    if let Some(s) = &args.script {
        if let Err(e) = script::add(&mut app, s, args.size) {
            eprintln!("--script: {e}");
            std::process::exit(2);
        }
    } else if headless {
        // screenshots wait for the renderer's shaders by themselves (take_shot); the fixed wait
        // is kept for software renderers, and the standard shots do without it
        let wait = args.wait.unwrap_or(if args.shots.is_some() { 0.0 } else { 45.0 });
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
    app.run();
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

// ------------------------------------------------------------------ world setup

fn vertex_coloured(materials: &mut Assets<StandardMaterial>, lk: &Look, rough: f32) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial { base_color: Color::WHITE, perceptual_roughness: rough, reflectance: lk.surfaces.painted_reflectance, ..default() })
}

fn placeholder_mesh() -> Mesh {
    let mut b = meshkit::Builder::default();
    b.tri(Vec3::new(0.0, -50.0, 0.0), Vec3::new(0.0, -50.0, 0.01), Vec3::new(0.01, -50.0, 0.0), [0.0, 0.0, 0.0, 1.0]);
    meshkit::into_mesh(b)
}

fn non_empty(b: meshkit::Builder) -> Mesh {
    if b.is_empty() { placeholder_mesh() } else { meshkit::into_mesh(b) }
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
    mut images: ResMut<Assets<Image>>,
    mut fonts: ResMut<Assets<Font>>,
) {
    let (r, g) = (&data.r, &data.g);
    let lk: &Look = &look;
    let k = tl.t.floor() as usize;
    commands.insert_resource(UiFont(fonts.add(Font::from_bytes(FREDOKA.to_vec()))));

    // ground: one heightfield, its patchwork painted per pixel from the year's land mix
    let year_cells = images.add(terrain::parcel_image(g, terrain::year_parcels(r, g, k)));
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

    // year meshes start empty; rebuild_year fills them
    let vc = vertex_coloured(&mut materials, lk, lk.surfaces.buildings);
    let flat = vertex_coloured(&mut materials, lk, lk.surfaces.paths);
    let handles = Handles {
        year_cells,
        trees: (0..LEAF_GROUPS).map(|_| [(); 3].map(|_| meshes.add(placeholder_mesh()))).collect(),
        houses: meshes.add(placeholder_mesh()),
        works: meshes.add(placeholder_mesh()),
        smoke: meshes.add(placeholder_mesh()),
        roads: meshes.add(placeholder_mesh()),
        rails: meshes.add(placeholder_mesh()),
        lanes: meshes.add(placeholder_mesh()),
    };
    // trees: what is green all year as it is; the leaves of the broadleaves under a material
    // that turns with the seasons; their bare limbs, flat blades seen from both sides. The
    // leaves and the limbs of a group are shown turn about (`seasons`), so winter costs no rebuild.
    let evergreen_mat = vertex_coloured(&mut materials, lk, lk.surfaces.foliage);
    let tree_mat = vertex_coloured(&mut materials, lk, lk.surfaces.foliage);
    let bare_mat = materials.add(bare_limb_material(lk));
    for (group, [evergreen, leaves, bare]) in handles.trees.iter().enumerate() {
        commands.spawn((Mesh3d(evergreen.clone()), MeshMaterial3d(evergreen_mat.clone())));
        commands.spawn((Mesh3d(leaves.clone()), MeshMaterial3d(tree_mat.clone()), Leaves(group)));
        commands.spawn((Mesh3d(bare.clone()), MeshMaterial3d(bare_mat.clone()), BareLimbs(group), Visibility::Hidden));
    }
    commands.insert_resource(SeasonMats { terrain: terrain_mat, trees: tree_mat, rivers: river_mat, forced: forced.0 });
    commands.spawn((Mesh3d(handles.houses.clone()), MeshMaterial3d(vc.clone())));
    commands.spawn((Mesh3d(handles.works.clone()), MeshMaterial3d(vc.clone())));
    commands.insert_resource(TrainAssets { mesh: meshes.add(meshkit::into_mesh(trains::train_mesh(lk))), mat: vc.clone() });
    let smoke_mat = materials.add(smoke_material(lk));
    commands.spawn((Mesh3d(handles.smoke.clone()), MeshMaterial3d(smoke_mat), bevy::light::NotShadowCaster));
    commands.spawn((Mesh3d(handles.roads.clone()), MeshMaterial3d(flat.clone()), bevy::light::NotShadowCaster));
    commands.spawn((Mesh3d(handles.rails.clone()), MeshMaterial3d(flat.clone()), bevy::light::NotShadowCaster));
    commands.spawn((Mesh3d(handles.lanes.clone()), MeshMaterial3d(flat), bevy::light::NotShadowCaster));
    commands.insert_resource(handles);

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
            start: std::time::Instant::now(),
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

// ------------------------------------------------------------------ the year

#[allow(clippy::too_many_arguments)]
/// A year's scenery: the ground's land mix, the meshes and the railway paths.
struct Scenery {
    k: usize,
    /// the camera its houses were built for
    lod: scatter::Lod,
    cells: Vec<u8>,
    ym: scatter::YearMeshes,
    nets: network::Nets,
    rails: Vec<trains::TrainPath>,
}

/// The year's scenery being built off the main thread, so a new year never
/// stops the picture. One build at a time; the year on screen keeps its
/// scenery until the next is ready, and the ground changes with it.
#[derive(Resource, Default)]
struct Builds {
    /// the year being built, and which state of the run it is built from (`Shown::run`)
    task: Option<(usize, u32, Task<Scenery>)>,
}

/// What `rebuild_year` remembers of the scenery on screen.
#[derive(Default)]
struct Shown {
    /// the camera its houses were built for
    lod: Option<scatter::Lod>,
    /// how many times the run has changed under the viewer (a live game's new year)
    run: u32,
    /// the year, and the state of the run, that everything on screen was last built from
    whole: Option<(usize, u32)>,
}

fn build_scenery(r: &Replay, g: &Ground, slots: &scatter::Slots, cen: &[(f32, f32)], k: usize, lk: &Look, lod: scatter::Lod) -> Scenery {
    let t0 = std::time::Instant::now();
    let cells = terrain::year_parcels(r, g, k);
    let ym = scatter::build_year(r, g, slots, cen, k, lk, &lod);
    let t1 = t0.elapsed().as_secs_f32();
    let nets = network::year_nets(r, g, cen, k, lk);
    let rails = trains::rail_paths(r, g, cen, k);
    if std::env::var_os("ISOSTATE_TIMING").is_some() {
        eprintln!("year {}: scatter {:.0} ms, networks {:.0} ms (off the main thread); {} vertices of houses for a camera {:.1} off", r.y0 + k as i32, t1 * 1e3, (t0.elapsed().as_secs_f32() - t1) * 1e3, ym.houses.pos.len(), lod.dist);
    }
    if std::env::var_os("ISOSTATE_MESH_SUMS").is_some() {
        // a fingerprint of the year's geometry: two builds of the viewer that print the
        // same line for a year build exactly the same scenery for it, to the last bit
        let trees = ym.trees.iter().flat_map(|t| t.parts()).fold(meshkit::Builder::default(), |mut all, b| {
            all.merge(b);
            all
        });
        let sums: Vec<String> = [("trees", &trees), ("houses", &ym.houses), ("works", &ym.works), ("smoke", &ym.smoke), ("lanes", &ym.lanes), ("roads", &nets.roads), ("rails", &nets.rails)]
            .iter()
            .map(|(name, b)| format!("{name} {:016x}", mesh_sum(b)))
            .collect();
        eprintln!("year {} meshes: {}", r.y0 + k as i32, sums.join(", "));
    }
    Scenery { k, lod, cells, ym, nets, rails }
}

/// A fingerprint of a mesh buffer: every bit of every corner, normal, colour and index.
fn mesh_sum(b: &meshkit::Builder) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut eat = |v: u32| {
        for byte in v.to_le_bytes() {
            h = (h ^ byte as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for p in b.pos.iter().chain(&b.nrm) {
        p.iter().for_each(|v| eat(v.to_bits()));
    }
    for c in &b.col {
        c.iter().for_each(|v| eat(v.to_bits()));
    }
    b.idx.iter().for_each(|&i| eat(i));
    h
}

#[allow(clippy::too_many_arguments)]
fn rebuild_year(
    mut commands: Commands,
    mut tl: ResMut<Timeline>,
    data: Res<Data>,
    look: Res<TheLook>,
    handles: Res<Handles>,
    ta: Res<TrainAssets>,
    mut rails: ResMut<Rails>,
    mut builds: ResMut<Builds>,
    orbit: Res<Orbit>,
    mut shown: Local<Shown>,
    old_trains: Query<Entity, With<Train>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    if data.is_changed() {
        shown.run = shown.run.wrapping_add(1);
    }
    // the houses are built with as much as the camera can see of them: when it has moved or
    // zoomed past what the scenery on screen was built for, the year is built again
    let lod = scatter::Lod::of_camera((orbit.target.x, orbit.target.z), orbit.dist);
    if tl.built.is_some() && shown.lod.is_some_and(|was| was.stale(&lod)) {
        tl.built = None;
    }
    // a finished build goes on screen
    if let Some((run, sc)) = builds.task.as_mut().and_then(|(_, run, t)| check_ready(t).map(|sc| (*run, sc))) {
        builds.task = None;
        shown.lod = Some(sc.lod);
        // a year built again for the camera alone, from the run as it was, differs only in its
        // houses and their smoke (a garden's tree is drawn at every level): the rest stays as
        // it is on screen, trees and works are not sent to the card again (they are most of
        // the year's corners), and the trains run on from where they are
        let for_camera = shown.whole == Some((sc.k, run));
        shown.whole = Some((sc.k, run));
        let (ym, nets) = (sc.ym, sc.nets);
        if !for_camera {
            if let Some(mut img) = images.get_mut(&handles.year_cells) {
                img.data = Some(sc.cells);
            }
            // trains: one for every eight units of line, up to three, spread along it
            for e in &old_trains {
                commands.entity(e).despawn();
            }
            rails.0 = sc.rails;
            for (pi, p) in rails.0.iter().enumerate() {
                let count = (1 + (p.len / 8.0) as usize).min(3);
                for i in 0..count {
                    let s = p.len * (i as f32 + 0.5) / count as f32;
                    let (pos, yaw) = p.at(s);
                    let dir = if (pi + i) % 2 == 0 { 1.0 } else { -1.0 };
                    commands.spawn((
                        Mesh3d(ta.mesh.clone()),
                        MeshMaterial3d(ta.mat.clone()),
                        Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw + if dir < 0.0 { std::f32::consts::PI } else { 0.0 })),
                        Train { path: pi, s, dir, speed: 0.5 + 0.3 * world::hash2(pi as i32, i as i32, 3) },
                    ));
                }
            }
            for (h, b) in [(&handles.works, ym.works), (&handles.roads, nets.roads), (&handles.rails, nets.rails), (&handles.lanes, ym.lanes)] {
                if let Some(mut m) = meshes.get_mut(h) {
                    *m = non_empty(b);
                }
            }
            for (hs, t) in handles.trees.iter().zip(ym.trees) {
                for (h, b) in hs.iter().zip([t.evergreen, t.leaves, t.bare]) {
                    if let Some(mut m) = meshes.get_mut(h) {
                        *m = non_empty(b);
                    }
                }
            }
        }
        for (h, b) in [(&handles.houses, ym.houses), (&handles.smoke, ym.smoke)] {
            if let Some(mut m) = meshes.get_mut(h) {
                *m = non_empty(b);
            }
        }
        tl.built = Some(sc.k);
    }
    // the year on screen, if it is neither shown nor being built; a build for
    // another year (the clock moved on) finishes first and is shown on the way
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    if tl.built != Some(k) && builds.task.is_none() {
        let (r, g, slots, cen, lk) = (data.r.clone(), data.g.clone(), data.slots.clone(), data.cen.clone(), look.0.clone());
        let task = AsyncComputeTaskPool::get().spawn(async move { build_scenery(&r, &g, &slots, &cen, k, &lk, lod) });
        builds.task = Some((k, shown.run, task));
    }
}

/// The scenery shows the year on the clock (screenshots wait for it).
fn scenery_ready(tl: &Timeline, data: &Data) -> bool {
    tl.built == Some((tl.t.floor() as usize).min(data.r.ny - 1))
}

/// One round of the seasons never takes less than this many seconds, however
/// fast the years go by.
const SEASON_ROUND_SECS: f32 = 24.0;

/// The seasons' own clock. A year lasts two seconds at 1x and a quarter of a
/// second at 8x, and a season tied to it strobes the fields and the sun. So
/// the seasons turn in real time while the game plays: with the year when the
/// years go slowly enough, and at one round in `SEASON_ROUND_SECS` when they
/// go faster. Paused, the season holds where it is; it never jumps.
#[derive(Resource, Clone, Copy, Debug)]
struct SeasonClock {
    /// phase of the year: 0 midwinter, 0.25 spring, 0.5 midsummer, 0.75 autumn
    phase: f32,
    /// the season on screen is the date's, so the top bar may name it
    follows: bool,
}

impl Default for SeasonClock {
    /// A game opens in high summer.
    fn default() -> Self {
        SeasonClock { phase: 0.55, follows: false }
    }
}

impl SeasonClock {
    /// Move on by `dt` seconds of real time, with the year's clock at `t`
    /// years and running at `speed` years a second (or stopped).
    fn advance(&mut self, playing: bool, speed: f32, t: f32, dt: f32) {
        // a long frame must not show as a jump of the season
        let dt = dt.clamp(0.0, 0.1);
        let cap = 1.0 / SEASON_ROUND_SECS;
        self.follows = playing && speed <= cap;
        if !playing {
            return;
        }
        if self.follows {
            // ease onto the date the short way round, then stay on it
            let d = (t.fract() - self.phase + 0.5).rem_euclid(1.0) - 0.5;
            self.phase += d * (1.0 - (-4.0 * dt).exp());
        } else {
            self.phase += cap * dt;
        }
        self.phase = self.phase.rem_euclid(1.0);
    }
}

fn season_clock(time: Res<Time>, tl: Res<Timeline>, mut clock: ResMut<SeasonClock>) {
    clock.advance(tl.playing, tl.speed, tl.t, time.delta_secs());
}

/// Phase of the year for the ground, the trees, the sun and the date: the
/// seasons' clock, or the season a screenshot asked for.
fn season_phase(clock: &SeasonClock, forced: Option<f32>) -> f32 {
    match forced {
        Some(f) => f.rem_euclid(1.0),
        None => clock.phase,
    }
}

fn seasons(
    clock: Res<SeasonClock>,
    sm: Res<SeasonMats>,
    look: Res<TheLook>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ground_mats: ResMut<Assets<terrain::GroundMaterial>>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), With<Sun>>,
    mut leaves: Query<(&Leaves, &mut Visibility), Without<BareLimbs>>,
    mut bare: Query<(&BareLimbs, &mut Visibility), Without<Leaves>>,
    mut last: Local<Option<i32>>,
) {
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
    // the leaves turn: 0 midwinter, 0.25 spring, 0.5 summer, 0.75 autumn; and each group of
    // broadleaf trees is in leaf from spring to autumn and shows its bare limbs through the
    // winter. They go by the seasons' clock, as the ground and the sun do, so a wood is gold,
    // bare and green again once a round at any speed of the years, and holds while paused
    if let Some(mut m) = materials.get_mut(&sm.trees) {
        m.base_color = leaf_tint(&look, ph);
    }
    // the rivers go to steel in the depth of winter
    if let Some(mut m) = materials.get_mut(&sm.rivers) {
        let t = look.water.river_tint(ph);
        m.base_color = Color::srgb(t[0], t[1], t[2]);
    }
    let shown = |on: bool| if on { Visibility::Inherited } else { Visibility::Hidden };
    for (group, mut vis) in &mut leaves {
        vis.set_if_neq(shown(in_leaf(ph, group.0)));
    }
    for (group, mut vis) in &mut bare {
        vis.set_if_neq(shown(!in_leaf(ph, group.0)));
    }
    let (strength, colour, stands) = sun_at(&look, ph);
    for (mut light, mut tf) in &mut sun {
        light.illuminance = strength;
        light.color = colour;
        *tf = stands;
    }
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
    if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::Period) {
        tl.playing = false;
        tl.t = (tl.t.floor() + 1.0).min(last);
    }
    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::Comma) {
        tl.playing = false;
        tl.t = (tl.t.floor() - 1.0).max(0.0);
    }
    if keys.just_pressed(KeyCode::PageUp) {
        tl.t = (tl.t.floor() + 10.0).min(last);
    }
    if keys.just_pressed(KeyCode::PageDown) {
        tl.t = (tl.t.floor() - 10.0).max(0.0);
    }
    // faster and slower go from one speed button to the next
    if keys.just_pressed(KeyCode::ArrowUp) {
        tl.speed = ui::step_speed(tl.speed, true);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        tl.speed = ui::step_speed(tl.speed, false);
    }
    if keys.just_pressed(KeyCode::KeyT) {
        tilt.0 = !tilt.0;
    }
}

fn advance_time(time: Res<Time>, mut tl: ResMut<Timeline>, data: Res<Data>, game: Res<game::Game>) {
    // a card that waits for the player stops the clock, however it came to be open
    if game.modal() && tl.playing {
        tl.playing = false;
    }
    if !tl.playing {
        return;
    }
    // a live game waits at the end of the last year it has until the next arrives
    let end = (data.r.ny - 1) as f32;
    let cap = if data.r.filled < data.r.ny { data.latest() as f32 + 0.999 } else { end };
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
) {
    let dt = time.delta_secs();
    let fwd = Vec3::new(-o.yaw.sin(), 0.0, -o.yaw.cos());
    let right = Vec3::new(o.yaw.cos(), 0.0, -o.yaw.sin());
    let pan = o.dist * 0.0016;
    // a drag belongs to the map only if it started on the map
    for (i, b) in [MouseButton::Left, MouseButton::Right].into_iter().enumerate() {
        if buttons.just_pressed(b) {
            on_map[i] = !ui.hover;
        }
    }
    if buttons.pressed(MouseButton::Left) && on_map[0] {
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

// Interaction is deprecated in Bevy 0.20; see the note at the top of ui.rs
#[allow(deprecated)]
fn setup_ui(mut commands: Commands, font: Res<UiFont>, data: Res<Data>) {
    let f = &font.0;
    // news as it happens, top right under the tabs
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(14.0),
            top: Val::Px(86.0),
            width: Val::Px(340.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },
        GlobalZIndex(5),
        ToastBox,
    ));
    // the inspector, bottom right above the timeline
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(14.0),
                bottom: Val::Px(70.0),
                width: Val::Px(380.0),
                padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(12.0), Val::Px(14.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                border_radius: BorderRadius::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(PAPER),
            BoxShadow::new(Color::srgba(0.25, 0.18, 0.10, 0.18), Val::Px(0.0), Val::Px(3.0), Val::Px(0.0), Val::Px(10.0)),
            Interaction::default(),
            bevy::ui::FocusPolicy::Block,
            GlobalZIndex(10),
            Visibility::Hidden,
            InspectPanel,
        ))
        .with_children(|p| {
            p.spawn((text(f, "", 20.0, INK), InspectTitle));
            p.spawn((text(f, "", 14.0, MUTED), InspectText));
            p.spawn((Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), ..default() }, InspectActions));
        });
    // town labels
    for t in 0..data.r.header.towns.len().min(data.r.n_towns) {
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
                p.spawn((
                    Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(9.0)), ..default() },
                    BackgroundColor(Color::srgba(0.99, 0.97, 0.92, 0.82)),
                ))
                .with_children(|q| {
                    q.spawn(text(f, &data.r.header.towns[t].name, 15.0, INK));
                });
            });
    }
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

fn labels(
    data: Res<Data>,
    tl: Res<Timeline>,
    o: Res<Orbit>,
    ov: Res<overlay::MapOverlay>,
    cams: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    mut q: Query<(&TownLabel, &mut Node, &mut Visibility, &Children)>,
    mut boxes: Query<&Children, Without<TownLabel>>,
    mut fonts: Query<&mut TextFont>,
) {
    let Ok((cam, cam_t)) = cams.single() else { return };
    let r = &data.r;
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
    mut boxq: Query<(Entity, Option<&Children>, &mut Visibility), With<ToastBox>>,
) {
    // with the news open the panel says it all; the standard shots are of the island, not the news
    for (_, _, mut v) in &mut boxq {
        v.set_if_neq(if ui.tab == Some(ui::Tab::News) || sheet.is_some() { Visibility::Hidden } else { Visibility::Inherited });
    }
    let r = &data.r;
    let k = (tl.t.floor() as usize).min(r.ny - 1);
    let now = time.elapsed_secs();
    let mut changed = false;
    if st.last_k != Some(k) {
        let year = r.y0 + k as i32;
        let jump = st.last_k.map(|l| (l as i64 - k as i64).abs() > 1).unwrap_or(true);
        if jump {
            st.shown.clear();
        }
        st.shown.extend(year_toasts(&r.header.events, year, now));
        while st.shown.len() > 3 {
            st.shown.remove(0);
        }
        st.last_k = Some(k);
        changed = true;
    }
    let before = st.shown.len();
    st.shown.retain(|t| now - t.since < 7.0);
    changed |= st.shown.len() != before;
    if !changed {
        return;
    }
    let Ok((boxe, children, _)) = boxq.single() else { return };
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
                q.spawn(text(f, &format!("{} \u{00b7} {}", kind_label(&e.kind), e.year), 13.0, kind_colour(&e.kind)));
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
    if let Some((kind, good)) = v.overlay.as_deref().and_then(|w| overlay::parse(w, r)) {
        let good = good.unwrap_or(ov.good);
        if ov.kind != kind || ov.good != good {
            ov.set(kind);
            ov.good = good;
        }
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
    // the first view waits for shaders to compile on software renderers, and every view for its year's scenery
    if !scenery_ready(&tl, &data) {
        return;
    }
    let ready = plan.index > 0 || plan.start.elapsed().as_secs_f32() >= plan.wait;
    // a pipeline still being put together means something in view is not drawn yet:
    // hold the shot until it is there and has been drawn for a few frames (a busy
    // machine can take many seconds over the ground's shader), but not for ever
    if ready && waiting.0.load(std::sync::atomic::Ordering::Relaxed) > 0 {
        let since = *held.get_or_insert_with(std::time::Instant::now);
        if since.elapsed().as_secs_f32() < 120.0 {
            plan.countdown = plan.countdown.max(4);
            return;
        }
    } else if let Some(since) = held.take() {
        eprintln!("waited {:.1} s for the renderer's shaders", since.elapsed().as_secs_f32());
    }
    if plan.countdown > 0 {
        if plan.countdown > 1 || ready {
            plan.countdown -= 1;
        }
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

fn ground_hit(g: &Ground, ray: Ray3d) -> Option<Vec3> {
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
    picked.what = choose(&data.r, &data.slots, &data.cen, hit, k);
}

/// The nearest works standing this year, then the nearest town, then the land itself.
fn choose(r: &Replay, slots: &scatter::Slots, cen: &[(f32, f32)], hit: Vec3, k: usize) -> Option<Pick> {
    // the nearest works standing this year, then the nearest town, then the land itself
    let mut best: Option<(f32, usize)> = None;
    for (f, firm) in r.header.firms.iter().enumerate() {
        let (x, z, _) = slots.firm_pos[f];
        let d = ((x - hit.x).powi(2) + (z - hit.z).powi(2)).sqrt();
        if d < 0.12 && replay::firm_k(r, f, k) > 0.0 && r.header.methods[firm.method].name != "services" && best.map_or(true, |b| d < b.0) {
            best = Some((d, f));
        }
    }
    if let Some((_, f)) = best {
        return Some(Pick::Firm(f));
    }
    for t in 0..r.n_towns.min(r.header.towns.len()) {
        if r.town_active[k * r.n_towns + t] == 0 {
            continue;
        }
        let (x, z) = cen[r.header.towns[t].node];
        if ((x - hit.x).powi(2) + (z - hit.z).powi(2)).sqrt() < 0.35 {
            return Some(Pick::Town(t));
        }
    }
    if hit.x >= 0.0 && hit.z >= 0.0 && (hit.x as usize) < r.w && (hit.z as usize) < r.h {
        let c = hit.z as usize * r.w + hit.x as usize;
        if r.terrain[c] > 0 {
            return Some(Pick::Cell(c));
        }
    }
    None
}

/// The square a picked thing stands on.
fn pick_cell(r: &Replay, what: Pick) -> usize {
    match what {
        Pick::Firm(f) => r.header.firms[f].cell,
        Pick::Town(t) => r.header.towns[t].node,
        Pick::Cell(c) => c,
    }
}

const DISP_NAMES: [&str; 12] = ["sea", "farmland", "pasture", "woodland", "heath", "quarry", "clay pit", "iron mine", "colliery", "works", "town", "derelict ground"];

fn describe(r: &Replay, what: Pick, k: usize) -> String {
    let year = r.y0 + k as i32;
    match what {
        Pick::Firm(f) => {
            let firm = &r.header.firms[f];
            let m = &r.header.methods[firm.method];
            let good = r.header.goods.get(m.out).map(|g| g.label.to_lowercase()).unwrap_or_default();
            let kk = replay::firm_k(r, f, k);
            let size = kk / m.k0.max(1e-6);
            let since = if firm.born < r.y0 { "working before 1750".to_string() } else { format!("since {}", firm.born) };
            let mut s = format!("{}\n{}, {}\nMakes up to {:.0} {} a month, {:.1} times a standard works.", firm.name, m.label, since, kk, good, size);
            if let Some(rs) = &firm.reason {
                let num = |key: &str| rs.get(key).and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
                let town = rs.get("town").and_then(|v| v.as_i64()).and_then(|t| r.header.towns.get(t as usize)).map(|t| t.name.clone()).unwrap_or_else(|| "a nearby town".into());
                let kind = rs.get("kind").and_then(|v| v.as_str()).unwrap_or("new");
                let lead = match kind {
                    "old" => format!("Already at work in 1750; built new, it would return about {:.0}% a year.", num("roi") * 100.0),
                    "expansion" => format!("A second site, opened in {} when the first ran out of room; about {:.0}% a year expected.", num("year") as i64, num("roi") * 100.0),
                    _ => format!("Built in {} with money from {}. The best of {} sites priced: about {:.0}% a year on the capital.", num("year") as i64, town, num("n_cand") as i64, num("roi") * 100.0),
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
            let since = if town.founded < r.y0 { "a market town before 1750".to_string() } else { format!("founded {}", town.founded) };
            format!("{}\n{}\n{} people in its market area, {} in the town, in {}.", town.name, since, fmt_pop(r.town_pop[i]), fmt_pop(r.town_urban[i]), year)
        }
        Pick::Cell(c) => {
            let off = k * r.n + c;
            let (a, b, sh) = (r.mix_a[off] as usize, r.mix_b[off] as usize, r.mix_s[off] as f32 / 255.0);
            let kind = { let n = DISP_NAMES[a % 12]; let mut t = n.to_string(); if let Some(f) = t.get_mut(0..1) { f.make_ascii_uppercase(); } t };
            // the town whose market the square trades in
            let town = (r.catch[off] as usize).checked_sub(1).and_then(|t| r.header.towns.get(t));
            let mut s = match town {
                Some(t) => format!("{} near {}\n", kind, t.name),
                None => format!("{kind}\n"),
            };
            if a == b {
                s.push_str(&format!("All {} in {}.", DISP_NAMES[a % 12], year));
            } else {
                s.push_str(&format!("{:.0}% {} and {:.0}% {} in {}.", sh * 100.0, DISP_NAMES[a % 12], (1.0 - sh) * 100.0, DISP_NAMES[b % 12], year));
            }
            s.push_str(&format!("\n{} km east and {} km south of the north-west corner.", (c % r.w) * 2, (c / r.w) * 2));
            s
        }
    }
}

fn inspect_panel(
    picked: Res<Picked>,
    tl: Res<Timeline>,
    data: Res<Data>,
    ov: Res<overlay::MapOverlay>,
    mut panel: Query<&mut Visibility, With<InspectPanel>>,
    mut txt: Query<&mut Text, (With<InspectText>, Without<InspectTitle>)>,
    mut title: Query<&mut Text, (With<InspectTitle>, Without<InspectText>)>,
    mut last: Local<Option<(Option<Pick>, usize, Option<overlay::Overlay>, usize)>>,
) {
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    let key = (picked.what, k, ov.kind, ov.good);
    if *last == Some(key) {
        return;
    }
    *last = Some(key);
    for mut v in &mut panel {
        *v = if picked.what.is_some() { Visibility::Inherited } else { Visibility::Hidden };
    }
    if let Some(w) = picked.what {
        let mut s = describe(&data.r, w, k);
        // with a map overlay on, what it says of this square
        if let Some(line) = ov.said(&data.r, k, pick_cell(&data.r, w)) {
            s.push_str("\n\n");
            s.push_str(&line);
        }
        let (head, body) = s.split_once('\n').unwrap_or((s.as_str(), ""));
        for mut t in &mut title {
            t.0 = head.to_string();
        }
        for mut t in &mut txt {
            t.0 = body.to_string();
        }
    }
}

fn move_trains(time: Res<Time>, rails: Res<Rails>, mut q: Query<(&mut Train, &mut Transform)>) {
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

    /// Run a season clock for some seconds at 30 frames a second; the year's
    /// clock runs beside it at `speed`. Returns the clock, how far the season
    /// went in all, and the biggest move between two frames.
    fn run_seasons(mut c: SeasonClock, playing: bool, speed: f32, secs: f32) -> (SeasonClock, f32, f32) {
        let dt = 1.0 / 30.0;
        let (mut t, mut went, mut jump) = (10.0f32, 0.0f32, 0.0f32);
        for _ in 0..(secs / dt).round() as usize {
            let before = c.phase;
            if playing {
                t += speed * dt;
            }
            c.advance(playing, speed, t, dt);
            let d = ((c.phase - before + 0.5).rem_euclid(1.0) - 0.5).abs();
            went += d;
            jump = jump.max(d);
        }
        (c, went, jump)
    }

    #[test]
    fn seasons_take_their_time_at_any_speed() {
        // at 8x (four years a second) and at 1x, one round in SEASON_ROUND_SECS and no faster
        for speed in [0.5, 1.0, 2.0, 4.0, 20.0] {
            let (c, went, jump) = run_seasons(SeasonClock::default(), true, speed, SEASON_ROUND_SECS);
            assert!((went - 1.0).abs() < 0.01, "at {speed} years a second the season went {went} rounds in {SEASON_ROUND_SECS} s");
            assert!(jump <= 1.0 / SEASON_ROUND_SECS / 30.0 + 1e-5, "a jump of {jump} at {speed}");
            assert!(!c.follows);
            assert!((c.phase - 0.55).abs() < 0.01, "a whole round comes back to where it began, not {}", c.phase);
        }
    }

    #[test]
    fn seasons_hold_when_paused_and_do_not_jump_on_resuming() {
        let (c, _, _) = run_seasons(SeasonClock::default(), true, 4.0, 5.0);
        let held = c.phase;
        let (c, went, _) = run_seasons(c, false, 4.0, 60.0);
        assert_eq!(c.phase, held);
        assert_eq!(went, 0.0);
        let (_, _, jump) = run_seasons(c, true, 0.5, 1.0);
        assert!(jump <= 1.0 / SEASON_ROUND_SECS / 30.0 + 1e-5, "resuming jumped {jump}");
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
                    c.advance(true, speed, t, dt);
                    let now = in_leaf(c.phase, group);
                    shed += (before && !now) as usize;
                    bud += (!before && now) as usize;
                    let tint = lk.foliage.tint(c.phase);
                    gold |= now && tint[0] > 1.15 * tint[1];
                }
                assert_eq!((shed, bud), (1, 1), "group {group} at {speed} years a second");
                assert!(gold, "group {group} is never gold in leaf at {speed} years a second");
                // paused in the depth of winter, the trees stay bare
                let mut c = SeasonClock { phase: 0.02, follows: false };
                for _ in 0..300 {
                    c.advance(false, speed, t, dt);
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
        assert!(jump < 0.08, "eased with a jump of {jump}");
        // paused, the name goes from the top bar and the season stays
        let (p, went, _) = run_seasons(c, false, speed, 5.0);
        assert!(!p.follows);
        assert_eq!(went, 0.0);
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

    fn news(year: f32, weight: f32, title: &str) -> replay::Event {
        replay::Event { year, kind: "family".into(), weight, title: title.into(), text: String::new(), cell: None, family: None, kept: None, seq: 0 }
    }

    #[test]
    fn toasts_keep_their_words_when_the_news_is_sorted_again() {
        let mut events = vec![news(1759.9, 0.9, "An old thing"), news(1760.7, 0.7, "Coke smelting"), news(1760.8, 0.3, "A small thing"), news(1760.95, 0.6, "A works opens")];
        // the year's major news, and nothing minor or from another year
        let shown = year_toasts(&events, 1760, 5.0);
        let titles: Vec<&str> = shown.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, ["Coke smelting", "A works opens"]);
        assert!(shown.iter().all(|t| t.year == 1760 && t.since == 5.0));
        // a live game adds news dated before what is there and sorts the list, as live::apply_year does
        events.push(news(1760.1, 0.9, "News that came late"));
        events.sort_by(|a, b| a.year.total_cmp(&b.year));
        // the place a toast came from now holds something else; the toast still says what it said
        assert_eq!(events[1].title, "News that came late");
        assert_eq!(shown[0].title, "Coke smelting");
        assert_eq!(shown[1].title, "A works opens");
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
        // a saved game is a game, even beside a recorded run
        let a = args("runs/x/run.bin --load mine.json --save other.json").unwrap();
        assert_eq!((a.new, a.fresh, a.load.as_deref(), a.save.as_deref()), (true, false, Some("mine.json"), Some("other.json")));
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
        // map views (a good the run does not have is refused once the run is read)
        assert_eq!(args("--overlay rents").err().unwrap(), "--overlay: rents is not a map view: use rent, areas, reach, price, price:GOOD, wages or plain");
        assert_eq!(args("--overlay").err().unwrap(), "--overlay needs a value after it");
        // a script's steps are read with the options
        assert_eq!(args("--script bogus").err().unwrap(), "--script: bogus: not a step (see the top of script.rs)");
        assert_eq!(args("--script key:Nope").err().unwrap(), "--script: key:Nope: no key called Nope");
        // saved games
        assert_eq!(args("--load").err().unwrap(), "--load needs a value after it");
        assert_eq!(args("--new --save").err().unwrap(), "--save needs a value after it");
        assert_eq!(args("--seed seven").err().unwrap(), "--seed: seven is not a number it can take");
        assert_eq!(args("--size 800").err().unwrap(), "--size: 800 should be two numbers with an x between them");
        assert_eq!(args("--size 0x600").err().unwrap(), "--size: a picture needs a width and a height");
        // in a view an empty value falls back, and one that is not a number is refused
        assert_eq!(args("--view a.png,l850").err().unwrap(), "--view: l850 is not a number it can take");
        assert_eq!(args("--view a.png,1850,markets").err().unwrap(), "--view: markets is not a number it can take");
        assert!(args("--view a.png,,,,,,").is_ok());
        // asking for the list
        assert_eq!(args("--help").err().unwrap(), "");
        // the list names every option the reader knows
        for opt in ["--new", "--seed", "--load", "--save", "--families", "--advance", "--demo", "--year", "--size", "--at", "--look", "--yaw", "--pitch", "--dist", "--no-tilt", "--season", "--pick", "--overlay", "--describe", "--view", "--shot", "--wait", "--frames", "--ground-debug", "--class-map", "--script", "--shots", "--shots-compare", "--shots-note", "--compare-shots", "--lab", "--out", "--tilt", "--loupe", "--turn", "--help"] {
            assert!(USAGE.contains(opt), "the usage text leaves out {opt}");
        }
    }

    #[test]
    fn a_forced_season_wins() {
        let c = SeasonClock { phase: 0.3, follows: false };
        assert_eq!(season_phase(&c, None), 0.3);
        assert_eq!(season_phase(&c, Some(0.0)), 0.0);
        assert_eq!(season_phase(&c, Some(1.25)), 0.25);
    }
}
