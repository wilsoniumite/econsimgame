//! A frame recorder, a way to drive a run with nobody at the window, and a
//! frame-time strip on screen. They answer one question: does the view wait?
//! It is the one measure of a frame the viewer has: a scripted run's `sample`
//! and `print` read their frame times from it (script.rs), and so does
//! `ISOSTATE_TIMING`.
//!
//!     ISOSTATE_FRAMES=out/run        record every frame; at the end write out/run.csv (a line a
//!                                    frame) and out/run.txt (the summary, which is also printed)
//!     ISOSTATE_FRAMES_PLAN="STEPS"   drive the run by itself (below), then stop
//!     ISOSTATE_FRAMES_PRESENT=novsync|vsync
//!                                    the window's present mode, for telling the frame's cost
//!                                    from the wait for the screen
//!     ISOSTATE_FRAMES_FAIL="freeze<150,frozen<10"
//!                                    thresholds on every segment whose name does not start with
//!                                    "warm" or "calm": the run exits 3 if one is broken (`Limits`),
//!                                    or 4 if one is and a "calm" segment, in which nothing is asked
//!                                    of the game, shows the machine was busy with something else
//!     ISOSTATE_FRAMES_TRY=instances  trees as things of their own, to price that way of drawing them
//!     ISOSTATE_FRAMES_CHUNKS=1750-1760
//!                                    no window: how much of a year's scenery differs from the year before
//!     ISOSTATE_FRAMES_STRIP=1|off    the strip up from the start, or not in the app at all
//!     F3                             the frame-time strip, in any run (no variable needed)
//!
//! (`ISOSTATE_HANDOVER=whole` is handover.rs's: a year's scenery to the
//! renderer in one frame, as it went before, to set beside the hand-over.)
//!
//! What a frame's line holds. On the main thread: the time from one frame's
//! start to the next (`frame_ms`), the main schedules (`main_ms`, and its parts
//! before, in and after `Update`), the extract (`extract_ms`: the main world
//! held still while the renderer copies what it needs) and the wait for the
//! render thread to finish the frame before (`wait_ms`). In the heavy systems:
//! one column each (`Sys`). On the render thread, for the same frame: its whole
//! schedule (`render_ms`) and its parts, of which `r_assets` is where changed
//! meshes and textures are packed and sent to the card and `r_acquire` is where
//! a window waits for the screen. And what happened in it: bytes of mesh
//! changed, how many of them were the year's scenery handed over (`handed_mb`),
//! how many of its meshes still wait (`queue`) and at what rate they are let
//! go (`rate_mb_s`), entities made and gone, and marks (`Y` a year arrived, `B`
//! a scenery build began, `W` a whole year's scenery was built and began to go
//! on screen, `C` the same for the houses and works alone, for the camera, `H`
//! a mesh of the year's scenery was handed to the renderer, `K` the clock's
//! year changed, `S` the season stepped, `L` a group of trees came into leaf or
//! shed, `P` a panel was opened or closed, `R` a panel's content changed, `O`
//! the map view changed, `D` a decision opened). W and C are `Shown::swaps`,
//! which `rebuild_year` counts; H is `HandOver::sent`.
//!
//! `work_ms` is the frame's cost with the screen left out: what the main thread
//! did (schedules and extract) and then what the render thread did for the same
//! frame, less its wait for the screen. A quiet frame's is about twice what the
//! frame takes, since the two threads overlap; a stalled frame's is how long the
//! picture stood. With no window (a scripted run draws into a picture thirty
//! times a second) it is the number to read. With a window `present_ms` is what
//! the player saw: the time on the render thread from the picture before this
//! frame's being handed to the screen to this frame's (the stamp after the
//! `Render` set, in which the engine presents). `frame_ms`, the main thread's
//! own frame-to-frame time, shows the same stall a frame late and split over
//! two frames: the one whose schedules and extract were long, and the next,
//! which waits for the render thread to finish with the first. The summary
//! adds long frames that follow one another up as freezes, so the two agree.
//!
//! Unset, what is left in the app: one system in `Last` that looks for F3
//! (the strip's card, three nodes of the interface and a texture of 46 kB,
//! is made the first time F3 is pressed, so that the standard pictures are
//! the same to the pixel with the recorder in the app as without it), and one
//! load of a flag in each of the hooked systems. Set, six small
//! schedules run between the engine's own on the main thread and nine stamps
//! in the render schedule, each taking a lock nobody else wants. A scripted
//! run (and `ISOSTATE_TIMING`) keeps the main thread's half of it as a meter:
//! the six schedules and the extract's timer, one frame remembered, nothing
//! written.
//!
//! The plan's steps, separated by semicolons:
//!
//!     seg:NAME            what follows is summarised under this name
//!     wait:SECONDS        let the game run
//!     settle              wait until no year is being worked out and the scenery is current,
//!                         and has been for four frames (the renderer takes a mesh in after it)
//!     key:NAME            press a key for a frame (Space closes the opening card)
//!     play, pause         the clock
//!     speed:fast          a speed by its button's name (slow, steady, brisk, fast, fastest),
//!                         or years a second as a number
//!     look:X,Z,DIST       point the camera
//!     pan:X,Z,DIST,SECS   move the camera there smoothly, as a drag and the wheel would
//!     year:Y              put the clock on a year
//!     scrub:Y0,Y1,SECS    drag the timeline from one year to another
//!     tab:NAME            a side panel: family, holdings, markets, news or none
//!     markets:WORD        what the Markets panel shows: island, or open:GOOD for a good's chart
//!     overlay:NAME        a map view: rent, areas, reach, price, price:coal, wages or plain
//!     queue:N             wait until a year is on its way to the renderer and N of its meshes
//!                         or fewer still wait (with ISOSTATE_HANDOVER=step:1 there is a second
//!                         between one mesh and the next: time for a picture of a year half arrived)
//!     shot:PATH           save a picture of what is on screen, and say what was still waiting
//!     quit                write the files and stop (the end of the plan does the same)
//!
//! While a plan runs, a decision that opens is given its default and the clock
//! started again, as a scripted `run:` does.

// see the note at the top of ui.rs
#![allow(deprecated)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::Mutex;
use std::time::Instant;

use bevy::app::MainScheduleOrder;
use bevy::asset::RenderAssetUsages;
use bevy::ecs::message::MessageCursor;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::render::pipelined_rendering::RenderExtractApp;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::{Render, RenderApp, RenderSystems};

use crate::handover::HandOver;
use crate::{Builds, Data, Orbit, SeasonClock, Shown, Timeline};

// ------------------------------------------------------------------ spans in the heavy systems

/// The systems that do real work when something changes, each with a column of its own.
#[derive(Clone, Copy)]
pub enum Sys {
    /// game::pump: a year's message written into the run
    Pump,
    /// rebuild_year: a finished build put on screen, the next one begun
    Scenery,
    /// game::owner_marks: flags, pennants and fences
    Marks,
    /// ui::panel_rebuild: the side panel and its charts
    Panel,
    /// game::order_panel and game::decision_card
    Orders,
    /// overlay::paint and overlay::legend_card: the map views
    MapView,
    /// seasons: the ground's and the trees' materials, the sun
    Seasons,
    /// move_trains, carts and boats: placing them anew and moving them
    Traffic,
    /// labels, toasts, the timeline's ticks
    Hud,
    /// handover::feed: the year's meshes let go to the renderer
    HandOver,
}

const SYS: [&str; 10] = ["pump", "scenery", "flags", "panel", "orders", "mapview", "seasons", "traffic", "hud", "handover"];

static ON: AtomicBool = AtomicBool::new(false);
static SPANS: [AtomicU64; SYS.len()] = [const { AtomicU64::new(0) }; SYS.len()];

/// Times the system it is the first line of, when the recorder is on.
pub struct Span(Option<(usize, Instant)>);

pub fn span(s: Sys) -> Span {
    Span(if ON.load(Relaxed) { Some((s as usize, Instant::now())) } else { None })
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some((i, t0)) = self.0 {
            SPANS[i].fetch_add(t0.elapsed().as_nanos() as u64, Relaxed);
        }
    }
}

// ------------------------------------------------------------------ what the render thread and the extract report

/// The parts of the render thread's frame, between the stamps set round its sets.
const RENDER_PARTS: [&str; 8] = ["r_cmds", "r_assets", "r_meshes", "r_views", "r_acquire", "r_queue", "r_draw", "r_cleanup"];

struct RenderSide {
    /// the main frame the render world last took in
    frame: u64,
    stamps: [Option<Instant>; RENDER_PARTS.len() + 1],
    /// for each main frame drawn: its number, the milliseconds of each part, and when its
    /// picture was handed to the screen (the stamp after the `Render` set, in which a
    /// window's picture is presented: `bevy_render/src/renderer/mod.rs`)
    done: Vec<(u64, [f32; RENDER_PARTS.len()], Option<Instant>)>,
}

/// The stamp after the `Render` set: the picture has been drawn and, with a window, presented.
const PRESENTED: usize = 7;

static RENDER: Mutex<RenderSide> = Mutex::new(RenderSide { frame: 0, stamps: [None; RENDER_PARTS.len() + 1], done: Vec::new() });
/// The main frame in hand, for the extract to hand to the render thread.
static FRAME: AtomicU64 = AtomicU64::new(0);
/// Nanoseconds in the last extract itself, and in the wait and the extract together.
static EXTRACT_NS: AtomicU64 = AtomicU64::new(0);
static HANDOVER_NS: AtomicU64 = AtomicU64::new(0);

fn render_stamp(i: usize) -> impl FnMut() {
    move || {
        let Ok(mut r) = RENDER.lock() else { return };
        r.stamps[i] = Some(Instant::now());
        if i == RENDER_PARTS.len() {
            let mut ms = [0.0; RENDER_PARTS.len()];
            for (j, m) in ms.iter_mut().enumerate() {
                if let (Some(a), Some(b)) = (r.stamps[j], r.stamps[j + 1]) {
                    *m = b.saturating_duration_since(a).as_secs_f32() * 1e3;
                }
            }
            let (frame, shown) = (r.frame, r.stamps[PRESENTED]);
            r.done.push((frame, ms, shown));
        }
    }
}

// ------------------------------------------------------------------ the record

const MAIN_PARTS: [&str; 5] = ["first_ms", "pre_ms", "update_ms", "post_ms", "last_ms"];

#[derive(Clone, Default)]
struct Frame {
    /// seconds since the recorder started
    at: f32,
    seg: usize,
    frame_ms: f32,
    main_ms: f32,
    parts: [f32; MAIN_PARTS.len()],
    extract_ms: f32,
    wait_ms: f32,
    spans: [f32; SYS.len()],
    render: Option<[f32; RENDER_PARTS.len()]>,
    /// when the render thread handed this frame's picture to the screen
    shown_at: Option<Instant>,
    /// from the picture before this one to this one, on the render thread: what the screen got
    present_ms: f32,
    mesh_mb: f32,
    meshes: u32,
    /// megabytes of the year's scenery handed to the renderer this frame, how many of
    /// its meshes still wait, and the rate they are let go at, in megabytes a second
    handed_mb: f32,
    queue: u32,
    rate: f32,
    images: u32,
    entities: u32,
    spawned: i32,
    year: i32,
    newest: i32,
    stepping: bool,
    building: bool,
    cam: bool,
    dist: f32,
    /// how long the scenery put on screen this frame took to build, off the main thread
    build_ms: f32,
    /// how long the year that arrived this frame took to come from the simulation
    sim_ms: f32,
    marks: String,
}

impl Frame {
    fn render_ms(&self) -> f32 {
        self.render.map_or(0.0, |r| r.iter().sum())
    }

    /// The frame's cost with the wait for the screen left out: its schedules,
    /// its extract and the render thread's work on it, which follow one another.
    fn work_ms(&self) -> f32 {
        let render = self.render.map_or(0.0, |r| r.iter().sum::<f32>() - r[4]);
        self.main_ms + self.extract_ms + render
    }
}

/// What the marks are read against: the game as the last frame left it.
#[derive(Default, PartialEq, Clone)]
struct Seen {
    filled: usize,
    built: Option<usize>,
    building: bool,
    year: usize,
    season: i32,
    leaf: [bool; isostate_gen::trees::LEAF_GROUPS],
    tab: Option<crate::ui::Tab>,
    dirty: u64,
    map: (Option<crate::overlay::Overlay>, usize),
    decision: Option<u64>,
    cam: (Vec3, f32, f32, f32),
    entities: u32,
    /// whole years and camera's builds begun to go on screen (`Shown::swaps`)
    swaps: (u32, u32),
    /// meshes and bytes of the year's scenery handed over so far (`HandOver::sent`)
    sent: (u64, u64),
}

#[derive(Resource)]
struct Recorder {
    /// where the files go; none for a meter, which remembers one frame and writes nothing
    out: Option<String>,
    t0: Instant,
    begun: Instant,
    stamps: [Instant; MAIN_PARTS.len() - 1],
    cur: Option<Frame>,
    frames: Vec<Frame>,
    segs: Vec<String>,
    seen: Option<Seen>,
    meshes: MessageCursor<AssetEvent<Mesh>>,
    images: MessageCursor<AssetEvent<Image>>,
    windowed: bool,
    about: String,
    written: bool,
    /// when the scenery build in hand began
    build_began: Option<Instant>,
    /// the longest frame since `take_longest` last asked, in ms
    longest: f32,
    /// scenery is going on screen: the longest frame since it began to, and how many
    /// frames are still to be watched once the last of it has gone
    swap: Option<(f32, u8)>,
    /// the longest frame while the last scenery went on screen and the renderer took it in
    swap_longest: Option<f32>,
}

/// What the last whole frame cost the main thread, for a scripted run's `sample`.
pub struct Work {
    /// the main schedules, from before `First` to after `Last`, in ms
    pub ms: f32,
    /// of them: before `Update` (input, the interface's pointer), in `Update` (the game's
    /// own systems), and after it (laying out the interface, transforms, what is in view)
    pub parts: [f32; 3],
    /// the extract after them: the main world held still while the renderer takes what changed
    pub extract_ms: f32,
}

/// The last whole frame's work, if the recorder is on (it is in a scripted run).
pub fn last_work(world: &World) -> Option<Work> {
    let f = world.get_resource::<Recorder>()?.frames.last()?;
    Some(Work { ms: f.main_ms, parts: [f.parts[0] + f.parts[1], f.parts[2], f.parts[3] + f.parts[4]], extract_ms: f.extract_ms })
}

/// The longest frame, start to start on the main thread, since this was last asked.
pub fn take_longest(world: &mut World) -> Option<f32> {
    world.get_resource_mut::<Recorder>().map(|mut rec| std::mem::take(&mut rec.longest))
}

/// The longest frame while the last scenery went on screen: from the frame its
/// build was queued in to the second after the last of its meshes was handed
/// over (the renderer takes a mesh at the end of the frame it is handed over
/// in, works on it beside the next, and holds up the one after).
pub fn swap_longest(world: &World) -> Option<f32> {
    let rec = world.get_resource::<Recorder>()?;
    rec.swap.map(|s| s.0).or(rec.swap_longest)
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct Stamp(u8);

/// The recorder is asked for (`ISOSTATE_FRAMES`).
fn asked() -> Option<String> {
    std::env::var("ISOSTATE_FRAMES").ok().filter(|s| !s.is_empty())
}

/// `ISOSTATE_FRAMES_CHUNKS=1750-1760,1890-1900`: with no window, build each
/// of those years' scenery and say how much of it differs from the year
/// before when the island is cut into squares of 8, 4 and 2 cells: what a
/// viewer that sent the card only the squares a year changed would send.
fn chunk_report(data: &Data, look: &crate::look::Look, spec: &str) {
    use std::collections::HashMap;
    const SIZES: [f32; 3] = [8.0, 4.0, 2.0];
    const KINDS: [&str; 4] = ["trees", "houses", "works", "flat and smoke"];
    type Squares = HashMap<(i32, i32), (u64, f32)>;
    let r = &data.r;
    let home = r.header.towns.first().map_or((r.w as f32 * 0.5, r.h as f32 * 0.5), |t| data.cen[t.node]);
    let lod = crate::scatter::Lod::of_camera(home, crate::shots::PLAY_DIST);
    // each square's fingerprint and megabytes, for each kind of scenery and each size of square
    let cut = |k: usize| -> Vec<([Squares; 3], std::collections::HashSet<u64>)> {
        let ym = crate::scatter::build_year(r, &data.g, &data.slots, &data.cen, k, look, &lod);
        let nets = crate::network::year_nets(r, &data.g, &data.cen, k, look);
        let trees: Vec<&crate::meshkit::Builder> = ym.trees.iter().flat_map(|t| t.parts()).collect();
        let kinds: [Vec<&crate::meshkit::Builder>; 4] = [trees, vec![&ym.houses], vec![&ym.works], vec![&ym.lanes, &ym.smoke, &nets.roads, &nets.rails]];
        kinds
            .iter()
            .map(|bufs| {
                let mut out: [Squares; 3] = Default::default();
                // every corner of the kind, to count those that are not where one was the year before
                let mut corners = std::collections::HashSet::new();
                for b in bufs {
                    if b.pos.is_empty() {
                        continue;
                    }
                    // a corner with its share of the indices, as the card holds them
                    let mb = (40.0 + 4.0 * b.idx.len() as f32 / b.pos.len() as f32) / 1e6;
                    for i in 0..b.pos.len() {
                        let (p, n, c) = (b.pos[i], b.nrm[i], b.col[i]);
                        let mut h = 0u64;
                        for v in p.iter().chain(&n).chain(c.iter()) {
                            h = (h ^ v.to_bits() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
                        }
                        corners.insert(h);
                        for (s, size) in SIZES.iter().enumerate() {
                            let e = out[s].entry(((p[0] / size).floor() as i32, (p[2] / size).floor() as i32)).or_insert((0, 0.0));
                            e.0 = (e.0.rotate_left(5) ^ h).wrapping_mul(0x0000_0100_0000_01b3);
                            e.1 += mb;
                        }
                    }
                }
                (out, corners)
            })
            .collect()
    };
    println!("chunks: island {} by {} cells; houses built for the play camera over the first town", r.w, r.h);
    println!("chunks: year, then for each kind of scenery its megabytes, how many differ from the year before in squares of 8, 4 and 2 cells (and how many squares of that size hold any), and how many of its corners are new");
    for range in spec.split(',').filter(|s| !s.is_empty()) {
        let (a, b) = range.split_once('-').unwrap_or((range, range));
        let (Ok(a), Ok(b)) = (a.trim().parse::<i32>(), b.trim().parse::<i32>()) else {
            eprintln!("ISOSTATE_FRAMES_CHUNKS: {range} should be YEAR-YEAR");
            continue;
        };
        let k_of = |y: i32| ((y - r.y0).max(0) as usize).min(data.latest());
        let mut before = cut(k_of(a));
        let mut sums = [[0.0f32; 4]; 4];
        let mut years = 0;
        for y in a + 1..=b {
            let now = cut(k_of(y));
            let mut line = format!("chunks: {y}");
            for (kind, name) in KINDS.iter().enumerate() {
                let total: f32 = now[kind].0[0].values().map(|v| v.1).sum();
                sums[kind][0] += total;
                line.push_str(&format!("  {name} {total:.0} MB:"));
                for s in 0..3 {
                    let changed: f32 = now[kind].0[s].iter().filter(|(at, v)| before[kind].0[s].get(*at).is_none_or(|was| was.0 != v.0)).map(|(_, v)| v.1).sum();
                    let squares = now[kind].0[s].iter().filter(|(at, v)| before[kind].0[s].get(*at).is_none_or(|was| was.0 != v.0)).count();
                    sums[kind][s + 1] += changed;
                    line.push_str(&format!(" {changed:.0} ({squares}/{})", now[kind].0[s].len()));
                }
                // the corners that are not where one was the year before: what truly changed
                let moved = now[kind].1.iter().filter(|c| !before[kind].1.contains(*c)).count();
                line.push_str(&format!(", {:.1}% of corners new", 100.0 * moved as f32 / now[kind].1.len().max(1) as f32));
            }
            println!("{line}");
            before = now;
            years += 1;
        }
        if years > 0 {
            let mut line = format!("chunks: mean of {} to {b}:", a + 1);
            for (kind, name) in KINDS.iter().enumerate() {
                let m = sums[kind].map(|v| v / years as f32);
                line.push_str(&format!("  {name} {:.0} MB, of which changed {:.0} / {:.0} / {:.0}", m[0], m[1], m[2], m[3]));
            }
            println!("{line}");
        }
    }
}

/// Add the strip's key, and with `ISOSTATE_FRAMES` the recorder, with
/// `ISOSTATE_FRAMES_PLAN` the plan that drives the run. `meter` asks for the
/// main thread's half of the recorder where no file is asked for: a scripted
/// run's `sample` and `print`, and `ISOSTATE_TIMING`, read their frames from it.
pub fn add(app: &mut App, meter: bool) {
    if let Ok(spec) = std::env::var("ISOSTATE_FRAMES_CHUNKS") {
        chunk_report(app.world().resource::<Data>(), &app.world().resource::<crate::look::TheLook>().0, &spec);
        std::process::exit(0);
    }
    // (`ISOSTATE_FRAMES_STRIP=off` leaves the strip out altogether: what the recorder costs
    // when it is off is measured against that)
    if std::env::var("ISOSTATE_FRAMES_STRIP").as_deref() != Ok("off") {
        app.init_resource::<Strip>().add_systems(Last, strip_draw);
    }
    let plan = std::env::var("ISOSTATE_FRAMES_PLAN").ok().filter(|s| !s.is_empty());
    if let Some(steps) = &plan {
        match parse(steps) {
            Ok(steps) => {
                app.insert_resource(Plan { steps, at: 0, begun: false, until: 0.0, from: None, keys: Vec::new(), clicked: None, resume: false, playing: false, tally: ("warm-up".into(), Instant::now(), 0), settled: 0 })
                    .add_systems(PreUpdate, drive.after(bevy::ui::UiSystems::Focus).after(bevy::input::InputSystems));
                // nobody is at the window: it must keep drawing whether or not it has the focus
                app.insert_resource(bevy::winit::WinitSettings::continuous());
            }
            Err(e) => {
                eprintln!("ISOSTATE_FRAMES_PLAN: {e}");
                std::process::exit(2);
            }
        }
    }
    // a cure priced without touching the scenery's own code
    let tries = std::env::var("ISOSTATE_FRAMES_TRY").unwrap_or_default();
    for t in tries.split(',').filter(|t| !t.is_empty()) {
        match t {
            "instances" => {
                app.add_systems(Update, try_instances);
            }
            // (the probe's other tries are built now, or gone: see docs/probes/dx-smooth.md)
            "nobounds" | "renderonly" => eprintln!("ISOSTATE_FRAMES_TRY: {t} is how the game is now, and is no longer a try"),
            "slices" => eprintln!("ISOSTATE_FRAMES_TRY: the squares are not in this build; the year's meshes go one a frame under a budget (handover.rs), and the prototype that cut them is in frames.rs as of 9dae4ab"),
            _ => eprintln!("ISOSTATE_FRAMES_TRY: {t} is not a try"),
        }
    }
    let mut present = String::from("none (no window: the picture is drawn off screen, thirty frames a second)");
    let mut q = app.world_mut().query_filtered::<&mut Window, With<bevy::window::PrimaryWindow>>();
    let windowed = if let Some(mut w) = q.iter_mut(app.world_mut()).next() {
        match std::env::var("ISOSTATE_FRAMES_PRESENT").as_deref() {
            Ok("novsync") => w.present_mode = bevy::window::PresentMode::AutoNoVsync,
            Ok("vsync") => w.present_mode = bevy::window::PresentMode::AutoVsync,
            _ => {}
        }
        present = format!("{:?}", w.present_mode);
        true
    } else {
        false
    };
    let out = asked();
    if out.is_none() && !meter {
        return;
    }
    ON.store(true, Relaxed);
    let pools = format!(
        "{} hardware threads; Bevy's pools: compute {}, async compute {}, io {}; rayon {}",
        std::thread::available_parallelism().map_or(0, |n| n.get()),
        bevy::tasks::ComputeTaskPool::get().thread_num(),
        bevy::tasks::AsyncComputeTaskPool::get().thread_num(),
        bevy::tasks::IoTaskPool::get().thread_num(),
        rayon::current_num_threads()
    );
    let pipelined = app.get_sub_app(RenderExtractApp).is_some();
    let about = format!(
        "{} build; rendering {}; present mode {present}; {pools}",
        if cfg!(debug_assertions) { "dev" } else { "release" },
        if pipelined { "pipelined (a render thread)" } else { "on the main thread" }
    );
    if out.is_some() {
        eprintln!("frames: {about}");
    }
    let recording = out.is_some();
    let now = Instant::now();
    let world = app.world_mut();
    let (meshes, images) = (world.resource::<Messages<AssetEvent<Mesh>>>().get_cursor(), world.resource::<Messages<AssetEvent<Image>>>().get_cursor());
    world.insert_resource(Recorder { out, t0: now, begun: now, stamps: [now; MAIN_PARTS.len() - 1], cur: None, frames: Vec::new(), segs: vec!["warm-up".into()], seen: None, meshes, images, windowed, about, written: false, build_began: None, longest: 0.0, swap: None, swap_longest: None });

    // the main thread: a stamp before First, after PreUpdate, Update and PostUpdate, and after Last
    let mut order = app.world_mut().resource_mut::<MainScheduleOrder>();
    order.insert_before(First, Stamp(0));
    order.insert_after(First, Stamp(1));
    order.insert_after(PreUpdate, Stamp(2));
    order.insert_after(Update, Stamp(3));
    order.insert_after(PostUpdate, Stamp(4));
    order.insert_after(Last, Stamp(5));
    app.add_systems(Stamp(0), begin);
    for i in 1..5u8 {
        app.add_systems(Stamp(i), move |mut rec: ResMut<Recorder>| rec.stamps[i as usize - 1] = Instant::now());
    }
    app.add_systems(Stamp(5), end);

    // the extract, and the wait for the render thread before it
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        let mut inner = render.take_extract();
        render.set_extract(move |main, sub| {
            let t0 = Instant::now();
            if let Some(f) = inner.as_mut() {
                f(main, sub);
            }
            EXTRACT_NS.store(t0.elapsed().as_nanos() as u64, Relaxed);
            if let Ok(mut r) = RENDER.lock() {
                r.frame = FRAME.load(Relaxed);
            }
        });
        // the render thread: a stamp between the sets of its schedule (not for a meter, which
        // reads the main thread only and keeps nothing a render thread's frames could pile up in)
        use RenderSystems as At;
        if recording {
            render.add_systems(
            Render,
            (
                render_stamp(0).before(At::ExtractCommands),
                render_stamp(1).after(At::ExtractCommands).before(At::PrepareAssets),
                render_stamp(2).after(At::PrepareAssets).before(At::PrepareMeshes),
                render_stamp(3).after(At::PrepareMeshes).before(At::CreateViews),
                render_stamp(4).after(At::Specialize).before(At::PrepareViews),
                render_stamp(5).after(At::PrepareViews).before(At::Queue),
                render_stamp(6).after(At::Prepare).before(At::Render),
                render_stamp(7).after(At::Render).before(At::Cleanup),
                render_stamp(8).after(At::PostCleanup),
            ),
            );
        }
    }
    if let Some(handover) = app.get_sub_app_mut(RenderExtractApp) {
        let mut inner = handover.take_extract();
        handover.set_extract(move |main, sub| {
            let t0 = Instant::now();
            if let Some(f) = inner.as_mut() {
                f(main, sub);
            }
            HANDOVER_NS.store(t0.elapsed().as_nanos() as u64, Relaxed);
        });
    }
}

/// `ISOSTATE_FRAMES_TRY=instances`: what trees would cost as things of their
/// own. A tree is one of a few shapes, turned, scaled and set down
/// (`isostate_gen::trees::tree`), so a wood could be a few meshes drawn many
/// times, which the engine batches, and a year would then send the card a
/// line for each tree that came or went instead of every corner of every
/// tree. This puts `ISOSTATE_FRAMES_INSTANCES` (100,000 if unset) of them on
/// the tree slots, in eight meshes and one material, beside the scenery as
/// it is, so that a quiet frame with them can be set against one without.
/// With `ISOSTATE_FRAMES_INSTANCES_CHURN=N`, N of them are taken away and N
/// put back every second, as a year's felling and planting would.
fn try_instances(mut commands: Commands, data: Res<Data>, look: Res<crate::look::TheLook>, ta: Option<Res<crate::TrainAssets>>, time: Res<Time<Real>>, mut meshes: ResMut<Assets<Mesh>>, mut made: Local<Option<(Vec<Handle<Mesh>>, std::collections::VecDeque<Entity>, usize, f32)>>) {
    use isostate_gen::trees::{tree, Greenery, Species, Tree};
    let Some(ta) = ta else { return };
    let slots = &data.slots.trees;
    let thing = |i: usize, kinds: &[Handle<Mesh>]| {
        let s = &slots[i % slots.len()];
        let y = crate::world::height_at(&data.g, s.x, s.z);
        (
            Mesh3d(kinds[i % kinds.len()].clone()),
            MeshMaterial3d(ta.mat.clone()),
            Transform::from_xyz(s.x, y, s.z).with_rotation(Quat::from_rotation_y(s.yaw)).with_scale(Vec3::splat(0.85 + 0.4 * s.v)),
        )
    };
    let Some((kinds, things, next, due)) = made.as_mut() else {
        let n: usize = std::env::var("ISOSTATE_FRAMES_INSTANCES").ok().and_then(|s| s.parse().ok()).unwrap_or(100_000);
        let mut corners = 0;
        let kinds: Vec<Handle<Mesh>> = (0..8)
            .map(|i| {
                let mut g = Greenery::default();
                let species = if i % 2 == 0 { Species::Conifer } else { Species::Broadleaf };
                tree(&mut g, &isostate_gen::Site::level(Vec3::ZERO, 0.0), &Tree { species, size: 1.0, tint: i as f32 / 8.0, turn: 0.0 }, &look.0);
                let mut b = g.evergreen;
                b.merge(&g.leaves);
                corners = b.pos.len();
                let mut m = crate::meshkit::into_mesh(b);
                m.asset_usage = RenderAssetUsages::RENDER_WORLD;
                meshes.add(m)
            })
            .collect();
        // spread over all the slots, so they stand all over the island as the trees do
        let stride = (slots.len() / n.max(1)).max(1);
        let things: std::collections::VecDeque<Entity> = (0..n).map(|i| commands.spawn(thing(i * stride, &kinds)).id()).collect();
        eprintln!("frames: try instances: {n} trees as things of their own, in 8 meshes of about {corners} corners and one material, on {} slots", slots.len());
        *made = Some((kinds, things, n * stride, time.elapsed_secs() + 1.0));
        return;
    };
    let churn: usize = std::env::var("ISOSTATE_FRAMES_INSTANCES_CHURN").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    if churn == 0 || time.elapsed_secs() < *due || things.is_empty() {
        return;
    }
    *due += 1.0;
    for _ in 0..churn.min(things.len()) {
        let Some(gone) = things.pop_front() else { break };
        commands.entity(gone).despawn();
        *next += 7;
        things.push_back(commands.spawn(thing(*next, kinds)).id());
    }
}

/// Each whole year that went to the renderer inside these frames, from the
/// frame its build was queued in (W) to the frame its last mesh was handed
/// over in: the megabytes handed over, and the seconds it took.
fn whole_years(fs: &[&Frame]) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    let mut going: Option<(f32, f32)> = None;
    for f in fs {
        if f.marks.contains('W') {
            going = Some((0.0, f.at));
        } else if f.marks.contains('C') {
            going = None;
        }
        if let Some((mb, began)) = going.as_mut() {
            *mb += f.handed_mb;
            if f.queue == 0 {
                out.push((*mb, f.at - *began));
                going = None;
            }
        }
    }
    out
}

fn ms(a: Instant, b: Instant) -> f32 {
    b.saturating_duration_since(a).as_secs_f32() * 1e3
}

/// Before `First`: the frame before is complete (its extract has run), and this one begins.
fn begin(mut rec: ResMut<Recorder>) {
    let now = Instant::now();
    if let Some(mut f) = rec.cur.take() {
        f.frame_ms = ms(rec.begun, now);
        let (extract, handover) = (EXTRACT_NS.swap(0, Relaxed) as f32 / 1e6, HANDOVER_NS.swap(0, Relaxed) as f32 / 1e6);
        f.extract_ms = extract;
        f.wait_ms = (handover - extract).max(0.0);
        rec.longest = rec.longest.max(f.frame_ms);
        // scenery going on screen is watched from the frame its build was queued in until
        // the last of it has gone, and for the two frames after that one
        if f.marks.contains('W') || f.marks.contains('C') {
            rec.swap = Some((0.0, 3));
        }
        if let Some((longest, left)) = rec.swap.as_mut() {
            *longest = longest.max(f.frame_ms);
            if f.queue == 0 {
                *left -= 1;
            }
            if *left == 0 {
                rec.swap_longest = rec.swap.take().map(|s| s.0);
                if crate::timing() {
                    eprintln!("  the longest frame while the renderer took it in was {:.0} ms", rec.swap_longest.unwrap_or(0.0));
                }
            }
        }
        if rec.out.is_none() {
            // a meter remembers the last frame only
            rec.frames.clear();
        }
        rec.frames.push(f);
    }
    rec.begun = now;
    FRAME.store(rec.frames.len() as u64, Relaxed);
}

/// After `Last`: what the frame's schedules took, and what happened in them.
fn end(world: &mut World) {
    let now = Instant::now();
    world.resource_scope(|world, mut rec: Mut<Recorder>| {
        let mut f = Frame { at: ms(rec.t0, rec.begun) / 1e3, seg: rec.segs.len() - 1, main_ms: ms(rec.begun, now), ..default() };
        let s = rec.stamps;
        f.parts = [ms(rec.begun, s[0]), ms(s[0], s[1]), ms(s[1], s[2]), ms(s[2], s[3]), ms(s[3], now)];
        for (i, a) in SPANS.iter().enumerate() {
            f.spans[i] = a.swap(0, Relaxed) as f32 / 1e6;
        }
        // the meshes and pictures changed this frame, which the renderer copies and sends to the card
        let meshes = world.resource::<Assets<Mesh>>();
        let mut ids = Vec::new();
        for e in rec.meshes.read(world.resource::<Messages<AssetEvent<Mesh>>>()) {
            if let AssetEvent::Modified { id } | AssetEvent::Added { id } = e {
                if !ids.contains(id) {
                    ids.push(*id);
                }
            }
        }
        for id in ids {
            let Some(m) = meshes.get(id) else { continue };
            f.meshes += 1;
            f.mesh_mb += (m.get_vertex_buffer_size() + m.get_index_buffer_bytes().map_or(0, |b| b.len())) as f32 / 1e6;
        }
        f.images = rec.images.read(world.resource::<Messages<AssetEvent<Image>>>()).filter(|e| matches!(e, AssetEvent::Modified { .. })).count() as u32;

        let (tl, data, orbit) = (world.resource::<Timeline>(), world.resource::<Data>(), world.resource::<Orbit>());
        let phase = crate::season_phase(world.resource::<SeasonClock>(), world.get_resource::<crate::SeasonMats>().and_then(|m| m.forced));
        let ui = world.resource::<crate::ui::UiState>();
        let ov = world.resource::<crate::overlay::MapOverlay>();
        let now_seen = Seen {
            filled: data.r.filled,
            built: tl.built,
            building: world.resource::<Builds>().task.is_some(),
            year: (tl.t.floor() as usize).min(data.r.ny - 1),
            season: (phase * 192.0).floor() as i32,
            leaf: std::array::from_fn(|g| isostate_gen::trees::in_leaf(phase, g)),
            tab: ui.tab,
            dirty: ui.dirty,
            map: (ov.kind, ov.good),
            decision: world.resource::<crate::game::Game>().open,
            cam: (orbit.target, orbit.yaw, orbit.pitch, orbit.dist),
            entities: world.entities().count_spawned(),
            swaps: world.resource::<Shown>().swaps,
            sent: world.resource::<HandOver>().sent,
        };
        let was = rec.seen.replace(now_seen.clone()).unwrap_or_else(|| now_seen.clone());
        let n = &now_seen;
        // a build was queued for the renderer this frame: a whole year, or the camera's
        let (whole, houses) = (n.swaps.0 != was.swaps.0, n.swaps.1 != was.swaps.1);
        let hand = world.resource::<HandOver>();
        f.handed_mb = (n.sent.1 - was.sent.1) as f32 / 1e6;
        f.queue = hand.waiting().0 as u32;
        f.rate = hand.budget.rate / 1e6;
        for (mark, on) in [
            ('Y', n.filled != was.filled),
            ('B', n.building && !was.building),
            ('W', whole),
            ('C', houses),
            ('H', n.sent.0 != was.sent.0),
            ('K', n.year != was.year),
            ('S', n.season != was.season),
            ('L', n.leaf != was.leaf),
            ('P', n.tab != was.tab),
            ('R', n.dirty != was.dirty),
            ('O', n.map != was.map),
            ('D', n.decision.is_some() && n.decision != was.decision),
        ] {
            if on {
                f.marks.push(mark);
            }
        }
        if whole || houses {
            f.build_ms = rec.build_began.take().map_or(0.0, |t| ms(t, now));
        }
        // (no build begins in the frame the last one is queued in: the queue is not empty)
        if n.building && !was.building {
            rec.build_began = Some(now);
        }
        if n.filled != was.filled {
            f.sim_ms = world.get_resource::<crate::live::Live>().and_then(|l| l.turn_took).unwrap_or(0.0) * 1e3;
        }
        f.entities = n.entities;
        f.spawned = n.entities as i32 - was.entities as i32;
        f.year = data.r.y0 + n.year as i32;
        f.newest = data.r.y0 + data.latest() as i32;
        f.stepping = world.get_resource::<crate::live::Live>().is_some_and(|l| l.stepping());
        f.building = n.building;
        f.cam = n.cam != was.cam;
        f.dist = orbit.dist;
        rec.cur = Some(f);
        // the window was closed, or the script ended: say what was seen
        let leaving = !world.resource::<Messages<AppExit>>().is_empty();
        if leaving && !rec.written && rec.out.is_some() {
            let verdict = write(&mut rec);
            if verdict != 0 {
                // the app is on its way out; the exit code says a limit was broken (3), or that
                // one was but the machine was busy, so the run says nothing (4)
                std::process::exit(verdict);
            }
        }
    });
}

// ------------------------------------------------------------------ the summary

/// A frame this long or longer is part of a freeze: two frames of a 60 Hz
/// screen missed, or four of a 144 Hz one.
const FREEZE_MS: f32 = 34.0;

/// In a segment named "calm..." nothing is asked of the game, and 95 frames
/// in a hundred should cost less than this: about twice a quiet frame's work
/// with no window, and three frames of a 120 Hz screen with one.
const CALM_MS: f32 = 25.0;

fn centile(sorted: &[f32], q: f32) -> f32 {
    if sorted.is_empty() { 0.0 } else { sorted[((sorted.len() - 1) as f32 * q).round() as usize] }
}

/// The middle, the 95th and 99th in a hundred and the longest, and how many were over each line.
fn spread(mut v: Vec<f32>) -> ([f32; 4], [usize; 4]) {
    v.sort_by(|a, b| a.total_cmp(b));
    let over = [16.7, 33.4, 50.0, 100.0].map(|line| v.iter().filter(|&&x| x > line).count());
    ([centile(&v, 0.5), centile(&v, 0.95), centile(&v, 0.99), v.last().copied().unwrap_or(0.0)], over)
}

fn spread_line(what: &str, v: Vec<f32>) -> String {
    let n = v.len();
    let (c, over) = spread(v);
    format!("  {what:<28} {n:>5} frames  middle {:>6.1}  95th {:>6.1}  99th {:>6.1}  longest {:>7.1} ms   over 16.7: {:>4}  over 33: {:>3}  over 50: {:>3}  over 100: {:>3}", c[0], c[1], c[2], c[3], over[0], over[1], over[2], over[3])
}

/// Everything in a frame that took half a millisecond or more, largest first.
fn held_by(f: &Frame) -> String {
    let mut parts: Vec<(String, f32)> = Vec::new();
    for (name, v) in MAIN_PARTS.iter().zip(f.parts) {
        parts.push((format!("main {}", name.trim_end_matches("_ms")), v));
    }
    parts.push(("extract".into(), f.extract_ms));
    parts.push(("wait for the render thread".into(), f.wait_ms));
    for (name, v) in SYS.iter().zip(f.spans) {
        parts.push((format!("[{name}]"), v));
    }
    if let Some(r) = f.render {
        for (name, v) in RENDER_PARTS.iter().zip(r) {
            parts.push((format!("render {}", name.trim_start_matches("r_")), v));
        }
    }
    parts.retain(|p| p.1 >= 0.5);
    parts.sort_by(|a, b| b.1.total_cmp(&a.1));
    parts.iter().map(|(n, v)| format!("{n} {v:.1}")).collect::<Vec<_>>().join(", ")
}

/// Thresholds a run must keep (`ISOSTATE_FRAMES_FAIL`): `p50`, `p95`, `p99`
/// and `max` in milliseconds, `over17`, `over33`, `over50` and `over100` in
/// frames, `freeze` (the longest freeze, in milliseconds) and `frozen` (the
/// share of the time in freezes, in hundredths), each with `<` or `<=` and a
/// number. They are read against
/// `present_ms` with a window and `work_ms` without, in every segment whose
/// name does not start with "warm".
struct Limits(Vec<(String, bool, f32)>);

impl Limits {
    fn from_env() -> Limits {
        let s = std::env::var("ISOSTATE_FRAMES_FAIL").unwrap_or_default();
        Limits(
            s.split(',')
                .filter(|p| !p.trim().is_empty())
                .filter_map(|p| {
                    let (name, rest) = p.trim().split_once('<')?;
                    let (strict, num) = match rest.strip_prefix('=') {
                        Some(n) => (false, n),
                        None => (true, rest),
                    };
                    Some((name.trim().to_string(), strict, num.trim().parse().ok()?))
                })
                .collect(),
        )
    }

    /// What a segment's frame times break, if anything.
    fn broken(&self, v: Vec<f32>) -> Vec<String> {
        let (c, over) = spread(v);
        let mut out = Vec::new();
        for (name, strict, line) in &self.0 {
            let got = match name.as_str() {
                "p50" => c[0],
                "p95" => c[1],
                "p99" => c[2],
                "max" => c[3],
                "over17" => over[0] as f32,
                "over33" => over[1] as f32,
                "over50" => over[2] as f32,
                "over100" => over[3] as f32,
                _ => continue,
            };
            if got > *line || (*strict && got == *line) {
                out.push(format!("{name} is {got:.1}, not {} {line}", if *strict { "under" } else { "at most" }));
            }
        }
        out
    }

    /// What a segment's freezes break: `freeze` is the longest, in
    /// milliseconds, and `frozen` the share of the time spent in them, in
    /// hundredths. These two are the ones to hold a run to: one long frame
    /// on a busy machine moves `max`, and hardly moves these.
    fn broken_freezes(&self, longest: f32, share: f32) -> Vec<String> {
        let mut out = Vec::new();
        for (name, strict, line) in &self.0 {
            let got = match name.as_str() {
                "freeze" => longest,
                "frozen" => share,
                _ => continue,
            };
            if got > *line || (*strict && got == *line) {
                out.push(format!("{name} is {got:.1}, not {} {line}", if *strict { "under" } else { "at most" }));
            }
        }
        out
    }
}

/// Write the CSV and the summary. The exit code the run should leave with:
/// 0, or 3 if a limit was broken, or 4 if one was but a "calm" segment shows
/// the machine was busy with something else.
fn write(rec: &mut Recorder) -> i32 {
    rec.written = true;
    // each frame's render thread record, which came in after the frame was closed
    if let Ok(mut r) = RENDER.lock() {
        for (frame, parts, shown) in r.done.drain(..) {
            if let Some(f) = rec.frames.get_mut(frame as usize) {
                f.render = Some(parts);
                f.shown_at = shown;
            }
        }
    }
    // picture to picture: from one frame's hand-over to the screen to the next's, which is
    // what the screen got (the main thread's frame-to-frame time splits a stall over two frames)
    let mut last: Option<Instant> = None;
    for f in rec.frames.iter_mut() {
        if let Some(t) = f.shown_at {
            f.present_ms = last.map_or(0.0, |l| ms(l, t));
            last = Some(t);
        }
    }
    let mut csv = String::from("frame,at_s,segment,frame_ms,present_ms,work_ms,main_ms");
    for name in MAIN_PARTS.iter().chain(&["extract_ms", "wait_ms"]).chain(SYS.iter()).chain(&["render_ms"]).chain(RENDER_PARTS.iter()) {
        csv.push(',');
        csv.push_str(name);
    }
    csv.push_str(",mesh_mb,meshes,handed_mb,queue,rate_mb_s,images,entities,spawned,year,newest,stepping,building,camera_moved,dist,build_ms,sim_ms,marks\n");
    for (i, f) in rec.frames.iter().enumerate() {
        csv.push_str(&format!("{i},{:.3},{},{:.2},{:.2},{:.2},{:.2}", f.at, rec.segs[f.seg], f.frame_ms, f.present_ms, f.work_ms(), f.main_ms));
        for v in f.parts.iter().chain(&[f.extract_ms, f.wait_ms]).chain(&f.spans).chain(&[f.render_ms()]).chain(f.render.unwrap_or_default().iter()) {
            csv.push_str(&format!(",{v:.2}"));
        }
        csv.push_str(&format!(",{:.2},{},{:.2},{},{:.0},{},{},{},{},{},{},{},{},{:.1},{:.0},{:.0},{}\n", f.mesh_mb, f.meshes, f.handed_mb, f.queue, f.rate, f.images, f.entities, f.spawned, f.year, f.newest, f.stepping as u8, f.building as u8, f.cam as u8, f.dist, f.build_ms, f.sim_ms, f.marks));
    }
    let limits = Limits::from_env();
    let mut broken = Vec::new();
    let mut busy = Vec::new();
    let mut txt = format!("frames: {}\n", rec.about);
    let which = if rec.windowed { "present_ms (with a window: from one picture handed to the screen to the next, on the render thread)" } else { "work_ms (no window: the frame's cost on either thread)" };
    txt.push_str(&format!("frames: stalls are read from {which}\n"));
    for (s, name) in rec.segs.iter().enumerate() {
        let fs: Vec<&Frame> = rec.frames.iter().filter(|f| f.seg == s).collect();
        if fs.is_empty() {
            continue;
        }
        // with a window, what the screen got: the time from the picture before to this one
        // (the main thread's frame-to-frame time, for a frame the render thread did not report)
        let stall = |f: &Frame| match (rec.windowed, f.shown_at) {
            (true, Some(_)) if f.present_ms > 0.0 => f.present_ms,
            (true, _) => f.frame_ms,
            (false, _) => f.work_ms(),
        };
        let secs = fs.last().map_or(0.0, |f| f.at) - fs[0].at;
        txt.push_str(&format!("\n[{name}] {:.1} s, {} frames, years {} to {}\n", secs, fs.len(), fs[0].year, fs.last().map_or(0, |f| f.year)));
        txt.push_str(&spread_line("frame to frame", fs.iter().map(|f| f.frame_ms).collect()));
        txt.push('\n');
        txt.push_str(&spread_line("picture to picture (render)", fs.iter().filter(|f| f.shown_at.is_some()).map(|f| f.present_ms).collect()));
        txt.push('\n');
        txt.push_str(&spread_line("work (screen left out)", fs.iter().map(|f| f.work_ms()).collect()));
        txt.push('\n');
        txt.push_str(&spread_line("main schedules", fs.iter().map(|f| f.main_ms).collect()));
        txt.push('\n');
        txt.push_str(&spread_line("extract", fs.iter().map(|f| f.extract_ms).collect()));
        txt.push('\n');
        txt.push_str(&spread_line("render thread", fs.iter().filter(|f| f.render.is_some()).map(|f| f.render_ms()).collect()));
        txt.push('\n');
        txt.push_str(&spread_line("  of it, assets to the card", fs.iter().filter_map(|f| f.render.map(|r| r[1])).collect()));
        txt.push('\n');
        txt.push_str(&spread_line("  of it, waiting for a frame", fs.iter().filter_map(|f| f.render.map(|r| r[4])).collect()));
        txt.push('\n');
        // the frames something came in during, which are the ones that can hitch
        for (mark, what) in [('Y', "a year arriving"), ('W', "a whole year queued"), ('C', "houses and works for the camera"), ('H', "a mesh handed over"), ('K', "the clock's year changing"), ('L', "trees shedding or budding"), ('P', "a panel opened"), ('R', "a panel's content"), ('O', "the map view changing")] {
            let hit: Vec<f32> = fs.iter().filter(|f| f.marks.contains(mark)).map(|f| stall(f)).collect();
            if !hit.is_empty() {
                txt.push_str(&spread_line(&format!("frames with {mark}: {what}"), hit));
                txt.push('\n');
            }
        }
        // the frame after a mesh is handed over is where the renderer takes it in
        let after: Vec<f32> = fs.windows(2).filter(|w| w[0].marks.contains('H')).map(|w| stall(w[1])).collect();
        if !after.is_empty() {
            txt.push_str(&spread_line("the frame after H", after));
            txt.push('\n');
        }
        let mb: f32 = fs.iter().map(|f| f.mesh_mb).sum();
        let swaps = fs.iter().filter(|f| f.marks.contains('W')).count();
        let made: i32 = fs.iter().map(|f| f.spawned.max(0)).sum();
        txt.push_str(&format!("  meshes changed: {mb:.0} MB in all ({} whole swaps, {} for the camera); entities made: {made}\n", swaps, fs.iter().filter(|f| f.marks.contains('C')).count()));
        // each whole year that went to the renderer from its first mesh to its last inside
        // the segment: how many megabytes of it arrived, and how long it took to
        let years = whole_years(&fs);
        if !years.is_empty() {
            let (mb, _) = spread(years.iter().map(|y| y.0).collect());
            let (secs, _) = spread(years.iter().map(|y| y.1 * 1e3).collect());
            let least = years.iter().map(|y| y.0).fold(f32::MAX, f32::min);
            txt.push_str(&format!(
                "  whole years handed over: {}, each of {:.0} MB in the middle (least {least:.0}, most {:.0}), on screen {:.0} ms after it was built in the middle (longest {:.0}); {:.0} MB of the year's scenery handed over in all\n",
                years.len(),
                mb[0],
                mb[3],
                secs[0],
                secs[3],
                fs.iter().map(|f| f.handed_mb).sum::<f32>()
            ));
        }
        // off the main thread: how long a year's scenery took to build, and a year to come from the simulation
        for (what, v) in [("scenery builds", fs.iter().map(|f| f.build_ms).filter(|&v| v > 0.0).collect::<Vec<f32>>()), ("years from the simulation", fs.iter().map(|f| f.sim_ms).filter(|&v| v > 0.0).collect())] {
            if !v.is_empty() {
                let n = v.len();
                let (c, _) = spread(v);
                txt.push_str(&format!("  {what}: {n}, middle {:.0} ms, longest {:.0} ms (off the main thread)\n", c[0], c[3]));
            }
        }
        let mean = |get: &dyn Fn(&Frame) -> f32| fs.iter().map(|f| get(f)).sum::<f32>() / fs.len() as f32;
        let spans: Vec<String> = SYS.iter().enumerate().map(|(i, n)| format!("{n} {:.2}", mean(&|f| f.spans[i]))).collect();
        txt.push_str(&format!("  mean ms a frame: main {:.2} (first {:.2}, pre {:.2}, update {:.2}, post {:.2}, last {:.2}), extract {:.2}, wait {:.2}; systems: {}\n", mean(&|f| f.main_ms), mean(&|f| f.parts[0]), mean(&|f| f.parts[1]), mean(&|f| f.parts[2]), mean(&|f| f.parts[3]), mean(&|f| f.parts[4]), mean(&|f| f.extract_ms), mean(&|f| f.wait_ms), spans.join(", ")));
        // freezes: frames over the line one after another, which the eye sees as one stop
        let mut freezes: Vec<f32> = Vec::new();
        let mut run = 0.0;
        for f in &fs {
            if stall(f) > FREEZE_MS {
                run += stall(f);
            } else if run > 0.0 {
                freezes.push(run);
                run = 0.0;
            }
        }
        if run > 0.0 {
            freezes.push(run);
        }
        let frozen: f32 = freezes.iter().sum();
        let n = freezes.len();
        let (c, _) = spread(freezes);
        let share = if secs > 0.0 { frozen / 10.0 / secs } else { 0.0 };
        if name.starts_with("calm") {
            // a segment in which nothing is asked of the game is the run's own check of the
            // machine: a freeze there, or frames well over a quiet frame's cost, is other programs
            let (c, _) = spread(fs.iter().map(|f| stall(f)).collect());
            if n > 0 || c[1] > CALM_MS {
                busy.push(format!("[{name}] 95th {:.1} ms, {n} freezes", c[1]));
            }
        } else if !name.starts_with("warm") {
            for why in limits.broken_freezes(c[3], share) {
                broken.push(format!("[{name}] {why}"));
            }
        }
        if n == 0 {
            txt.push_str(&format!("  freezes (frames over {FREEZE_MS} ms, one after another): none\n"));
        } else {
            txt.push_str(&format!("  freezes (frames over {FREEZE_MS} ms, one after another): {n}, middle {:.0} ms, longest {:.0} ms; {:.1} s in all, {share:.0}% of the time\n", c[0], c[3], frozen / 1e3));
        }
        // the three longest stalls and what was in them
        let mut worst: Vec<usize> = (0..fs.len()).collect();
        worst.sort_by(|&a, &b| stall(fs[b]).total_cmp(&stall(fs[a])));
        for &i in worst.iter().take(3) {
            let f = fs[i];
            // (a frame the render thread did not report is read from the main thread's time,
            // which a stall reaches a frame late: then the frame before says what it was)
            let before = if i > 0 && rec.windowed && f.shown_at.is_none() { format!(" (the frame before: [{}] {:.0} MB of mesh)", fs[i - 1].marks, fs[i - 1].mesh_mb) } else { String::new() };
            txt.push_str(&format!("  stall {:>6.1} ms at {:.2} s in {} [{}] {:.0} MB of mesh, {:+} entities{before}: {}\n", stall(f), f.at, f.year, f.marks, f.mesh_mb, f.spawned, held_by(f)));
        }
        if !name.starts_with("warm") && !name.starts_with("calm") {
            for why in limits.broken(fs.iter().map(|f| stall(f)).collect()) {
                broken.push(format!("[{name}] {why}"));
            }
        }
    }
    for b in &broken {
        txt.push_str(&format!("\nframes: BROKEN {b}"));
    }
    for b in &busy {
        txt.push_str(&format!("\nframes: BUSY {b}: the machine was not quiet while nothing was asked of the game"));
    }
    if !broken.is_empty() && !busy.is_empty() {
        txt.push_str("\nframes: the limits cannot be judged on a busy machine: run it again");
    }
    if !limits.0.is_empty() && broken.is_empty() {
        txt.push_str("\nframes: every limit was kept");
    }
    eprintln!("{txt}");
    let out = rec.out.clone().unwrap_or_default();
    let path = std::path::Path::new(&out);
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        let _ = std::fs::create_dir_all(parent);
    }
    for (ext, body) in [("csv", &csv), ("txt", &txt)] {
        let file = format!("{out}.{ext}");
        match std::fs::write(&file, body) {
            Ok(()) => eprintln!("frames: wrote {file}"),
            Err(e) => eprintln!("frames: cannot write {file}: {e}"),
        }
    }
    match (broken.is_empty(), busy.is_empty()) {
        (true, _) => 0,
        (false, true) => 3,
        (false, false) => 4,
    }
}

// ------------------------------------------------------------------ the plan

#[derive(Clone, Debug, PartialEq)]
enum Step {
    Seg(String),
    Wait(f32),
    Settle,
    Key(KeyCode),
    Play(bool),
    Speed(f32),
    Look(f32, f32, f32),
    Pan(f32, f32, f32, f32),
    Year(i32),
    Scrub(i32, i32, f32),
    Tab(Option<crate::ui::Tab>),
    Markets(String),
    Overlay(String),
    Queue(usize),
    Shot(String),
    Quit,
}

#[derive(Resource)]
struct Plan {
    steps: Vec<Step>,
    at: usize,
    begun: bool,
    until: f32,
    /// where the camera was when a pan began
    from: Option<(Vec3, f32, f32)>,
    keys: Vec<KeyCode>,
    clicked: Option<Entity>,
    resume: bool,
    /// the plan has the clock running
    playing: bool,
    /// the segment in hand, when it began and how many frames it has had: with the
    /// recorder off this is all that is counted, so that a run with it on can be set
    /// beside one without and the recorder's own cost read off
    tally: (String, Instant, u32),
    /// how many frames in a row a `settle` has found the game quiet
    settled: u32,
}

/// A `settle` is over when the game has been quiet for more frames than this
/// one after another: the render thread is a frame behind the main thread and
/// takes a swapped year in during the frame after the swap, and the main
/// thread waits for it in the frame after that.
const SETTLE_FRAMES: u32 = 3;

impl Plan {
    /// With the recorder off, say how many frames the segment in hand had, and begin the next.
    fn tell(&mut self, recording: bool, next: &str) {
        let (name, began, frames) = &self.tally;
        if !recording && *frames > 0 {
            let secs = began.elapsed().as_secs_f32();
            eprintln!("frames: [{name}] {frames} frames in {secs:.2} s, {:.2} ms a frame (the recorder is off: nothing else is measured)", secs * 1e3 / *frames as f32);
        }
        self.tally = (next.into(), Instant::now(), 0);
    }
}

fn parse(s: &str) -> Result<Vec<Step>, String> {
    let mut steps = Vec::new();
    for part in s.split(';').map(str::trim).filter(|p| !p.is_empty()) {
        let (word, arg) = part.split_once(':').unwrap_or((part, ""));
        let nums = || arg.split(',').map(|v| v.trim().parse::<f32>()).collect::<Result<Vec<f32>, _>>().map_err(|_| format!("{part}: numbers separated by commas"));
        steps.push(match word {
            "seg" if !arg.is_empty() => Step::Seg(arg.to_string()),
            "wait" => Step::Wait(arg.parse().map_err(|_| format!("{part}: the seconds must be a number"))?),
            "settle" => Step::Settle,
            "key" => Step::Key(match arg.to_ascii_lowercase().as_str() {
                "space" => KeyCode::Space,
                "escape" | "esc" => KeyCode::Escape,
                "left" => KeyCode::ArrowLeft,
                "right" => KeyCode::ArrowRight,
                "m" => KeyCode::KeyM,
                "t" => KeyCode::KeyT,
                "f3" => KeyCode::F3,
                _ => return Err(format!("{part}: no key called {arg}")),
            }),
            "play" => Step::Play(true),
            "pause" => Step::Play(false),
            "speed" => {
                let by_name = crate::clock::SPEEDS.iter().find(|s| s.name.eq_ignore_ascii_case(arg)).map(|s| s.rate);
                Step::Speed(by_name.or_else(|| arg.parse().ok()).ok_or_else(|| format!("{part}: a speed button's name or years a second"))?)
            }
            "look" => match nums()?.as_slice() {
                [x, z, d] => Step::Look(*x, *z, *d),
                _ => return Err(format!("{part}: look takes x,z,distance")),
            },
            "pan" => match nums()?.as_slice() {
                [x, z, d, s] => Step::Pan(*x, *z, *d, *s),
                _ => return Err(format!("{part}: pan takes x,z,distance,seconds")),
            },
            "year" => Step::Year(arg.parse().map_err(|_| format!("{part}: a year"))?),
            "scrub" => match nums()?.as_slice() {
                [a, b, s] => Step::Scrub(*a as i32, *b as i32, *s),
                _ => return Err(format!("{part}: scrub takes year,year,seconds")),
            },
            "tab" => Step::Tab(crate::ui::Tab::parse(arg)),
            "markets" if crate::ui::markets_word(arg) => Step::Markets(arg.to_string()),
            "overlay" if crate::overlay::is_word(arg) => Step::Overlay(arg.to_string()),
            "queue" => Step::Queue(arg.parse().map_err(|_| format!("{part}: how many meshes may still wait"))?),
            "shot" if !arg.is_empty() => Step::Shot(arg.to_string()),
            "quit" => Step::Quit,
            _ => return Err(format!("{part}: not a step (see the top of frames.rs)")),
        });
    }
    if steps.is_empty() {
        return Err("the plan has no steps".into());
    }
    Ok(steps)
}

fn drive(world: &mut World) {
    let Some(mut plan) = world.remove_resource::<Plan>() else { return };
    step(world, &mut plan);
    world.insert_resource(plan);
}

/// Give an open decision its default and start the clock again once its card
/// has gone, as script.rs does for `run:`. True if something was pressed.
fn keep_playing(world: &mut World, plan: &mut Plan) -> bool {
    let game = world.resource::<crate::game::Game>();
    let (open, busy) = (game.open, game.busy);
    if let Some(id) = open {
        if busy {
            return false;
        }
        let default = world.get_resource::<crate::family::FamilyView>().and_then(|f| f.player()).and_then(|p| p.rep.pending.iter().find(|d| d.id == id)).map_or(0, |d| d.default);
        let mut q = world.query::<(Entity, &crate::game::DecideButton)>();
        let hit = q.iter(world).find(|(_, d)| d.id == id && d.option == default).map(|(e, _)| e);
        if let Some(e) = hit {
            if let Some(mut i) = world.get_mut::<Interaction>(e) {
                *i = Interaction::Pressed;
                plan.clicked = Some(e);
                plan.resume = true;
                return true;
            }
        }
        return false;
    }
    if plan.resume && !busy {
        plan.resume = false;
        // only a clock the plan has started is started again
        if plan.playing && !world.resource::<Timeline>().playing {
            world.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
            plan.keys.push(KeyCode::Space);
            return true;
        }
    }
    false
}

fn step(world: &mut World, plan: &mut Plan) {
    plan.tally.2 += 1;
    let recording = world.get_resource::<Recorder>().is_some_and(|r| r.out.is_some());
    if !plan.keys.is_empty() {
        let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
        for k in plan.keys.drain(..) {
            keys.release(k);
        }
        return;
    }
    if let Some(e) = plan.clicked.take() {
        if let Some(mut i) = world.get_mut::<Interaction>(e) {
            *i = Interaction::None;
        }
        return;
    }
    if world.contains_resource::<crate::live::Live>() && keep_playing(world, plan) {
        return;
    }
    let now = world.resource::<Time<Real>>().elapsed_secs();
    loop {
        let Some(st) = plan.steps.get(plan.at).cloned() else {
            plan.tell(recording, "");
            world.write_message(AppExit::Success);
            return;
        };
        let mut done = true;
        let mut frame = false;
        match st {
            Step::Seg(name) => {
                plan.tell(recording, &name);
                if let Some(mut rec) = world.get_resource_mut::<Recorder>() {
                    rec.segs.push(name);
                }
                frame = true;
            }
            Step::Wait(s) => {
                if !plan.begun {
                    plan.begun = true;
                    plan.until = now + s;
                }
                done = now >= plan.until;
            }
            Step::Settle => {
                if !plan.begun {
                    plan.begun = true;
                    plan.until = now + 180.0;
                }
                let live = world.get_resource::<crate::live::Live>();
                let quiet = !live.is_some_and(|l| l.stepping())
                    && !world.resource::<crate::game::Game>().busy
                    && crate::scenery_ready(world.resource::<Timeline>(), world.resource::<Data>())
                    && world.resource::<Builds>().task.is_none()
                    && world.resource::<HandOver>().is_empty();
                // the frame the scenery goes on screen in is not the end of it: the render
                // thread takes it in during the frames that follow, and the segment after a
                // `settle` must not be charged with that
                plan.settled = if quiet { plan.settled + 1 } else { 0 };
                done = plan.settled > SETTLE_FRAMES || now >= plan.until;
                if done {
                    plan.settled = 0;
                }
            }
            Step::Key(k) => {
                world.resource_mut::<ButtonInput<KeyCode>>().press(k);
                plan.keys.push(k);
                frame = true;
            }
            Step::Play(on) => {
                world.resource_mut::<Timeline>().playing = on;
                // a decision that stops the clock is answered, and the clock started again
                plan.playing = on;
                plan.resume = on;
            }
            Step::Speed(v) => world.resource_mut::<Timeline>().speed = v,
            Step::Look(x, z, d) => {
                let y = crate::world::height_at(&world.resource::<Data>().g, x, z).max(0.0);
                let mut o = world.resource_mut::<Orbit>();
                o.target = Vec3::new(x, y, z);
                o.dist = d;
                frame = true;
            }
            Step::Pan(x, z, d, secs) => {
                if !plan.begun {
                    plan.begun = true;
                    plan.until = now + secs;
                    let o = world.resource::<Orbit>();
                    plan.from = Some((o.target, o.dist, now));
                }
                let (from, dist, t0) = plan.from.unwrap_or((Vec3::ZERO, d, now));
                let u = ((now - t0) / secs.max(1e-3)).clamp(0.0, 1.0);
                let mut o = world.resource_mut::<Orbit>();
                // the height follows the ground by itself (camera_input)
                o.target.x = from.x + (x - from.x) * u;
                o.target.z = from.z + (z - from.z) * u;
                o.dist = dist * (d / dist.max(1e-3)).powf(u);
                done = u >= 1.0;
            }
            Step::Year(y) => {
                let data = world.resource::<Data>();
                let k = (y - data.r.y0).clamp(0, data.latest() as i32) as f32;
                let mut tl = world.resource_mut::<Timeline>();
                tl.playing = false;
                tl.t = k;
                frame = true;
            }
            Step::Scrub(a, b, secs) => {
                if !plan.begun {
                    plan.begun = true;
                    plan.until = now + secs;
                }
                let u = (1.0 - (plan.until - now) / secs.max(1e-3)).clamp(0.0, 1.0);
                let data = world.resource::<Data>();
                let y = a as f32 + (b - a) as f32 * u;
                let k = (y - data.r.y0 as f32).round().clamp(0.0, data.latest() as f32);
                let mut tl = world.resource_mut::<Timeline>();
                tl.playing = false;
                tl.t = k;
                done = u >= 1.0;
            }
            Step::Tab(t) => {
                world.resource_mut::<crate::ui::UiState>().tab = t;
                frame = true;
            }
            Step::Markets(word) => {
                world.resource_scope(|world, mut ui: Mut<crate::ui::UiState>| ui.show_markets(&word, &world.resource::<Data>().r));
                frame = true;
            }
            Step::Overlay(word) => {
                let parsed = crate::overlay::parse(&word, &world.resource::<Data>().r);
                if let Some((kind, good)) = parsed {
                    let mut ov = world.resource_mut::<crate::overlay::MapOverlay>();
                    ov.set(kind);
                    if let Some(g) = good {
                        ov.good = g;
                    }
                }
                frame = true;
            }
            Step::Queue(n) => {
                if !plan.begun {
                    plan.begun = true;
                    plan.until = now + 120.0;
                }
                let hand = world.resource::<HandOver>();
                done = (hand.year().is_some() && hand.waiting().0 <= n) || now >= plan.until;
                // (the mesh that has just gone is in the picture of the frame after this one)
                frame = done;
            }
            Step::Shot(path) => {
                use bevy::camera::RenderTarget;
                use bevy::render::view::screenshot::{save_to_disk, Screenshot};
                if let Some(parent) = std::path::Path::new(&path).parent().filter(|p| !p.as_os_str().is_empty()) {
                    let _ = std::fs::create_dir_all(parent);
                }
                // with no window the camera draws into a picture (a scripted run gives it one)
                let mut q = world.query_filtered::<&RenderTarget, With<crate::MainCam>>();
                let shot = match q.iter(world).next() {
                    Some(RenderTarget::Image(target)) => Screenshot::image(target.handle.clone()),
                    _ => Screenshot::primary_window(),
                };
                let (hand, y0) = (world.resource::<HandOver>(), world.resource::<Data>().r.y0);
                let (n, bytes) = hand.waiting();
                let of = hand.year().map_or("no year on its way".to_string(), |k| format!("{} on its way", y0 + k as i32));
                let built = world.resource::<Timeline>().built.map_or("none".to_string(), |k| (y0 + k as i32).to_string());
                eprintln!("frames: shot {path}: {of}, {n} of its meshes ({:.0} MB) still waiting ({}); the last year whole on screen is {built}", bytes as f32 / 1e6, hand.waiting_names(world.resource::<crate::Handles>()));
                world.spawn(shot).observe(save_to_disk(path));
                frame = true;
            }
            Step::Quit => {
                plan.tell(recording, "");
                world.write_message(AppExit::Success);
                return;
            }
        }
        if !done {
            return;
        }
        plan.begun = false;
        plan.at += 1;
        if frame {
            return;
        }
    }
}

// ------------------------------------------------------------------ the strip on screen

const STRIP_W: usize = 240;
const STRIP_H: usize = 48;
/// The strip's full height, in milliseconds.
const STRIP_MS: f32 = 50.0;

/// The frame-time strip: the last 240 frames as bars, a line at 16.7 ms, a
/// dark tick under the frames a year arrived in and a pale one under those a
/// mesh of the year's scenery was handed to the renderer in. F3 shows and hides it.
#[derive(Resource, Default)]
struct Strip {
    on: bool,
    /// the card, once it has been asked for: nothing of it is made before then
    card: bool,
    image: Handle<Image>,
    /// each frame's milliseconds, whether a year arrived in it, and whether a mesh was handed over
    bars: std::collections::VecDeque<(f32, bool, bool)>,
    filled: usize,
    sent: u64,
    last: Option<Instant>,
}

#[derive(Component)]
struct StripCard;
#[derive(Component)]
struct StripText;

/// The strip's card, made the first time it is asked for. It is not made
/// before then, hidden: a card in the interface, even a hidden one with no
/// words, moved a few pixels of every standard picture by a shade (its text's
/// size, it seems, changes how the interface's letters are packed), and the
/// standard pictures must be the same with the recorder in the app as without.
fn strip_card(commands: &mut Commands, images: &mut Assets<Image>, font: &crate::UiFont) -> Handle<Image> {
    let size = Extent3d { width: STRIP_W as u32, height: STRIP_H as u32, depth_or_array_layers: 1 };
    let image = images.add(Image::new(size, TextureDimension::D2, vec![0; STRIP_W * STRIP_H * 4], TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default()));
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, left: Val::Px(14.0), bottom: Val::Px(70.0), flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), padding: UiRect::all(Val::Px(8.0)), border_radius: BorderRadius::all(Val::Px(10.0)), ..default() },
            BackgroundColor(crate::ui::PAPER),
            GlobalZIndex(20),
            StripCard,
        ))
        .with_children(|p| {
            p.spawn((ImageNode::new(image.clone()), Node { width: Val::Px(STRIP_W as f32), height: Val::Px(STRIP_H as f32), ..default() }));
            p.spawn((crate::ui::text(&font.0, "", 12.0, crate::ui::INK), StripText));
        });
    image
}

#[allow(clippy::too_many_arguments)]
fn strip_draw(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    data: Res<Data>,
    hand: Res<HandOver>,
    font: Res<crate::UiFont>,
    mut strip: ResMut<Strip>,
    mut images: ResMut<Assets<Image>>,
    mut card: Query<&mut Visibility, With<StripCard>>,
    mut words: Query<&mut Text, With<StripText>>,
    mut looked: Local<bool>,
) {
    // F3, or ISOSTATE_FRAMES_STRIP=1 in the first frame
    let asked = !std::mem::replace(&mut *looked, true) && std::env::var_os("ISOSTATE_FRAMES_STRIP").is_some();
    if keys.just_pressed(KeyCode::F3) || asked {
        strip.on = !strip.on;
        strip.bars.clear();
        strip.last = None;
        if !strip.card {
            strip.card = true;
            strip.image = strip_card(&mut commands, &mut images, &font);
        }
        for mut v in &mut card {
            *v = if strip.on { Visibility::Inherited } else { Visibility::Hidden };
        }
    }
    if !strip.on {
        return;
    }
    // by the clock on the wall: the engine's own delta is held to a quarter of a second
    let now = Instant::now();
    let took = strip.last.replace(now).map(|t| ms(t, now));
    let year = data.r.filled != strip.filled;
    strip.filled = data.r.filled;
    let handed = hand.sent.0 != strip.sent;
    strip.sent = hand.sent.0;
    let Some(took) = took else { return };
    strip.bars.push_back((took, year, handed));
    while strip.bars.len() > STRIP_W {
        strip.bars.pop_front();
    }
    let mut px = vec![0u8; STRIP_W * STRIP_H * 4];
    let mut put = |x: usize, y: usize, c: [u8; 4]| px[(y * STRIP_W + x) * 4..][..4].copy_from_slice(&c);
    let line = STRIP_H - 1 - (16.7 / STRIP_MS * (STRIP_H - 4) as f32) as usize;
    for (i, &(v, year, handed)) in strip.bars.iter().enumerate() {
        let x = STRIP_W - strip.bars.len() + i;
        let h = ((v / STRIP_MS).min(1.0) * (STRIP_H - 4) as f32).ceil() as usize;
        let c = if v > 33.4 { [178, 62, 52, 255] } else if v > 16.7 { [204, 140, 48, 255] } else { [88, 128, 84, 255] };
        for y in 0..h.max(1) {
            put(x, STRIP_H - 4 - y, c);
        }
        if year || handed {
            for y in STRIP_H - 3..STRIP_H {
                put(x, y, if year { [70, 60, 50, 255] } else { [150, 140, 125, 255] });
            }
        }
    }
    for x in (0..STRIP_W).step_by(3) {
        put(x, line - 3, [70, 60, 50, 110]);
    }
    if let Some(mut img) = images.get_mut(&strip.image) {
        img.data = Some(px);
    }
    let mut v: Vec<f32> = strip.bars.iter().map(|b| b.0).collect();
    v.sort_by(|a, b| a.total_cmp(b));
    let s = format!("{:.1} ms a frame \u{00b7} longest of {} {:.0} ms \u{00b7} over 33: {}", centile(&v, 0.5), v.len(), v.last().copied().unwrap_or(0.0), v.iter().filter(|&&x| x > 33.4).count());
    for mut t in &mut words {
        if t.0 != s {
            t.0 = s.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plan_is_read_and_one_that_is_not_is_refused() {
        let steps = parse("seg:still; wait:5;settle;key:Space;play;pause;speed:Fastest;speed:0.25;look:60,46,30;pan:20,20,3,8;year:1850;scrub:1750,1900,10;tab:markets;tab:none;overlay:price:coal;quit").unwrap();
        assert_eq!(steps.len(), 16);
        assert_eq!(steps[0], Step::Seg("still".into()));
        assert_eq!((&steps[6], &steps[7]), (&Step::Speed(2.0), &Step::Speed(0.25)));
        assert_eq!(steps[9], Step::Pan(20.0, 20.0, 3.0, 8.0));
        assert_eq!(steps[11], Step::Scrub(1750, 1900, 10.0));
        assert_eq!((&steps[12], &steps[13]), (&Step::Tab(Some(crate::ui::Tab::Markets)), &Step::Tab(None)));
        assert_eq!(parse("").err().unwrap(), "the plan has no steps");
        assert_eq!(parse("dance").err().unwrap(), "dance: not a step (see the top of frames.rs)");
        assert_eq!(parse("overlay:rents").err().unwrap(), "overlay:rents: not a step (see the top of frames.rs)");
        assert!(parse("pan:1,2,3").is_err());
        assert_eq!(parse("queue:11;shot:out/half.png").unwrap(), vec![Step::Queue(11), Step::Shot("out/half.png".into())]);
        assert!(parse("queue:some").is_err() && parse("shot").is_err());
    }

    #[test]
    fn the_spread_counts_the_frames_over_each_line() {
        let v: Vec<f32> = (0..100).map(|i| if i == 99 { 120.0 } else if i >= 95 { 40.0 } else { 7.0 }).collect();
        let (c, over) = spread(v.clone());
        assert_eq!((c[0], c[3]), (7.0, 120.0));
        assert_eq!(over, [5, 5, 1, 1]);
        // limits: under means under, and at most lets the line itself through
        let limits = Limits(vec![("p50".into(), true, 8.0), ("max".into(), true, 100.0), ("over33".into(), false, 5.0), ("over50".into(), true, 1.0)]);
        let broken = limits.broken(v);
        assert_eq!(broken.len(), 2, "{broken:?}");
        assert!(broken[0].starts_with("max is 120.0") && broken[1].starts_with("over50 is 1.0"));
    }

    #[test]
    fn work_leaves_out_the_wait_for_the_screen() {
        let mut f = Frame { main_ms: 3.0, extract_ms: 1.0, ..default() };
        assert_eq!(f.work_ms(), 4.0);
        // a render thread that spent 14 of its 20 ms waiting for the screen did 6 ms of work
        f.render = Some([0.5, 2.0, 0.5, 0.5, 14.0, 1.0, 1.0, 0.5]);
        assert_eq!((f.render_ms(), f.work_ms()), (20.0, 10.0));
    }
}
