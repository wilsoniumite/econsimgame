//! Isolated State: a 3D replay viewer for runs of the island economy.
//!
//!     cargo run -p isostate-viewer --release -- runs/landlords/run.bin
//!     cargo run -p isostate-viewer --release -- runs/landlords/run.bin --year 1850 --shot out.png
//!
//! Space plays and pauses, the arrow keys step through years, drag to pan,
//! right-drag to turn, scroll to zoom.

mod boats;
mod carts;
mod cursor;
mod family;
mod fields;
mod game;
mod icons;
mod live;
mod look;
mod meshkit;
mod network;
mod plan;
mod rails;
mod replay;
mod rivers;
mod scatter;
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

use look::{Look, TheLook};
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
    wait: f32,
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
}

fn parse_args() -> Args {
    let mut a = Args {
        run: "runs/landlords/run.bin".into(),
        year: None,
        shot: None,
        size: (1600, 1000),
        at: None,
        look_file: None,
        yaw: 0.0,
        pitch: 52.0,
        dist: None,
        frames: 40,
        wait: 45.0,
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
        pick: None,
        ground_debug: false,
        class_map: false,
        shots: None,
        shots_compare: None,
        shots_note: String::new(),
        compare_shots: None,
    };
    let v: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < v.len() {
        let next = |i: usize| v.get(i + 1).cloned().unwrap_or_default();
        match v[i].as_str() {
            "--year" => { a.year = next(i).parse().ok(); i += 1; }
            "--shot" => { a.shot = Some(next(i)); i += 1; }
            "--size" => {
                let s = next(i);
                if let Some((w, h)) = s.split_once('x') { a.size = (w.parse().unwrap_or(1600), h.parse().unwrap_or(1000)); }
                i += 1;
            }
            "--at" => {
                let s = next(i);
                if let Some((x, z)) = s.split_once(',') { a.at = Some((x.parse().unwrap_or(60.0), z.parse().unwrap_or(42.0))); }
                i += 1;
            }
            "--look" => {
                // a look file; two numbers are the point to look at, which --look used to mean
                let s = next(i);
                match s.split_once(',').map(|(x, z)| (x.parse::<f32>(), z.parse::<f32>())) {
                    Some((Ok(x), Ok(z))) => a.at = Some((x, z)),
                    _ => a.look_file = Some(s),
                }
                i += 1;
            }
            "--yaw" => { a.yaw = next(i).parse().unwrap_or(0.0); i += 1; }
            "--pitch" => { a.pitch = next(i).parse().unwrap_or(52.0); i += 1; }
            "--dist" => { a.dist = next(i).parse().ok(); i += 1; }
            "--frames" => { a.frames = next(i).parse().unwrap_or(40); i += 1; }
            "--wait" => { a.wait = next(i).parse().unwrap_or(45.0); i += 1; }
            "--no-tilt" => a.tilt = false,
            "--new" => a.new = true,
            "--seed" => { a.seed = next(i).parse().unwrap_or(7); i += 1; }
            "--families" => { a.families = next(i).parse().unwrap_or(6); i += 1; }
            "--advance" => { a.advance = next(i).parse().unwrap_or(0); i += 1; }
            "--demo" => a.demo = true,
            "--pick" => {
                let s = next(i);
                if let Some((x, z)) = s.split_once(',') { a.pick = Some((x.parse().unwrap_or(0.0), z.parse().unwrap_or(0.0))); }
                i += 1;
            }
            "--ground-debug" => a.ground_debug = true,
            "--class-map" => a.class_map = true,
            "--season" => { a.season = next(i).parse().ok(); i += 1; }
            "--shots" => { a.shots = Some(next(i)); i += 1; }
            "--shots-compare" => { a.shots_compare = Some(next(i)); i += 1; }
            "--shots-note" => { a.shots_note = next(i); i += 1; }
            "--compare-shots" => { a.compare_shots = Some((next(i), next(i + 1))); i += 2; }
            "--describe" => {
                let s = next(i);
                if let Some((x, z)) = s.split_once(',') { a.describe = Some((x.parse().unwrap_or(0.0), z.parse().unwrap_or(0.0))); }
                i += 1;
            }
            "--view" => {
                // path,year,x,z,yaw,pitch,dist[,season][,tab]: a number after dist is a season, a word a panel
                let p: Vec<String> = next(i).split(',').map(String::from).collect();
                let num = |j: usize| p.get(j).and_then(|s| s.parse::<f32>().ok());
                a.views.push(ShotSpec {
                    path: p.first().cloned().unwrap_or_else(|| "shot.png".into()),
                    year: num(1).map(|y| y as i32),
                    look: match (num(2), num(3)) { (Some(x), Some(z)) => Some((x, z)), _ => None },
                    yaw: num(4).unwrap_or(0.0),
                    pitch: num(5).unwrap_or(52.0),
                    dist: num(6),
                    season: p.iter().skip(7).find_map(|s| s.parse::<f32>().ok()),
                    tab: p.iter().skip(7).find(|s| !s.is_empty() && s.parse::<f32>().is_err()).map(|s| ui::Tab::parse(s)),
                });
                i += 1;
            }
            s if !s.starts_with("--") => { a.run = s.to_string(); a.named_run = true; }
            _ => {}
        }
        i += 1;
    }
    if !a.named_run {
        a.new = true;
    }
    if a.shots.is_some() {
        // the standard shots run from the first year to the last, so play the whole game first
        a.advance = usize::MAX - 1;
    }
    a
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
    trees: Handle<Mesh>,
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

#[derive(Resource, Default)]
struct Toasts {
    shown: Vec<(usize, f32)>,
    last_k: Option<usize>,
}

#[derive(Resource)]
struct UiFont(Handle<Font>);

#[derive(Resource)]
struct Tilt(bool);

/// Materials that change with the seasons.
#[derive(Resource)]
struct SeasonMats {
    terrain: Handle<terrain::GroundMaterial>,
    trees: Handle<StandardMaterial>,
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
    let t0 = std::time::Instant::now();
    let (r, live, families) = if args.new {
        eprintln!("making island {} with {} families\u{2026}", args.seed, args.families);
        match live::start(args.seed, args.families, args.advance, args.demo) {
            Ok((r, l, f)) => (r, Some(l), Some(f)),
            Err(e) => {
                eprintln!("the game could not start: {e}");
                std::process::exit(1);
            }
        }
    } else {
        match replay::load(&args.run) {
            Ok(r) => (r, None, None),
            Err(e) => {
                eprintln!("{e}\nMake a run first, for example:\n  cargo run --release -p isostate-core -- run --seed 7 --out runs/rust7\nor start a new game with --new");
                std::process::exit(1);
            }
        }
    };
    let g = world::build_ground(&r);
    let slots = scatter::make_slots(&r, &g);
    let cen = network::centres(&r);
    eprintln!("prepared {} in {:.1}s", if args.new { "the new game" } else { args.run.as_str() }, t0.elapsed().as_secs_f32());

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
    let latest = r.filled.clamp(1, r.ny) - 1;
    let k0 = match args.year {
        Some(y) => ((y - r.y0).max(0) as usize).min(latest),
        None => latest,
    };
    if let Some((x, z)) = args.describe {
        // print what a click at this spot would show, then stop
        let hit = Vec3::new(x, world::height_at(&g, x, z), z);
        match choose(&r, &slots, &cen, hit, k0) {
            Some(p) => println!("{}", describe(&r, p, k0)),
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
        views.insert(0, ShotSpec { path, year: args.year, look: args.at, yaw: args.yaw, pitch: args.pitch, dist: args.dist, season: None, tab: None });
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
    let headless = !views.is_empty();
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
    app.insert_resource(ClearColor(look.sky.background.color()))
        .insert_resource(GlobalAmbientLight { color: look.sky.light.color(), brightness: look.sky.brightness, ..default() })
        .insert_resource(Data { r: Arc::new(r), g: Arc::new(g), slots: Arc::new(slots), cen: Arc::new(cen), live: live.is_some() })
        .init_resource::<Builds>()
        .insert_resource(Timeline { t: k0 as f32, built: None, playing: false, speed: 2.0 })
        .insert_resource(orbit)
        .insert_resource(Tilt(args.tilt && look.tilt_shift.on))
        .insert_resource(TheLook(look))
        .insert_resource(ForcedSeason(args.season))
        .insert_resource(GroundDebug(args.ground_debug))
        .init_resource::<Toasts>()
        .insert_resource(picked)
        .init_resource::<Rails>()
        .add_plugins(ui::plugin)
        .add_plugins(game::plugin)
        .add_plugins(cursor::plugin)
        .add_plugins(boats::plugin)
        .add_plugins(carts::plugin)
        .add_systems(Startup, (setup_world, setup_ui, icons::setup).chain())
        .add_systems(Update, (ui::ui_hover, time_input, advance_time, rebuild_year, move_trains, seasons, camera_input, apply_camera, labels, toasts, pick, inspect_panel).chain());
    if let Some(l) = live {
        app.insert_resource(l);
    }
    if let Some(f) = families {
        app.insert_resource(f);
    }
    if args.ground_debug {
        app.add_systems(Update, hide_all_but_ground.after(rebuild_year));
    }
    if headless {
        app.insert_resource(ShotPlanArgs { views, size: args.size, frames: args.frames, wait: args.wait, default_dist: dist, default_look: look_at });
        app.add_systems(Update, take_shot.after(toasts));
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
    b.into_mesh()
}

fn non_empty(b: meshkit::Builder) -> Mesh {
    if b.is_empty() { placeholder_mesh() } else { b.into_mesh() }
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
    let year_cells = images.add(terrain::cell_image(r, terrain::year_cells(r, k), TextureFormat::Rgba8Unorm));
    let fixed_cells = images.add(terrain::cell_image(r, terrain::fixed_cells(r, g), TextureFormat::Rgba8Unorm));
    let terrain_mat = ground_mats.add(terrain::GroundMaterial {
        base: StandardMaterial { perceptual_roughness: lk.surfaces.ground, reflectance: lk.surfaces.ground_reflectance, ..default() },
        extension: terrain::Patchwork {
            params: terrain::patchwork_params(r, g, season_phase(&tl, forced.0), debug.0),
            year: year_cells.clone(),
            fixed: fixed_cells,
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
    commands.spawn((Mesh3d(meshes.add(non_empty(network::rivers(g, lk)))), MeshMaterial3d(river_mat), bevy::light::NotShadowCaster));

    // year meshes start empty; rebuild_year fills them
    let vc = vertex_coloured(&mut materials, lk, lk.surfaces.buildings);
    let flat = vertex_coloured(&mut materials, lk, lk.surfaces.paths);
    let handles = Handles {
        year_cells,
        trees: meshes.add(placeholder_mesh()),
        houses: meshes.add(placeholder_mesh()),
        works: meshes.add(placeholder_mesh()),
        smoke: meshes.add(placeholder_mesh()),
        roads: meshes.add(placeholder_mesh()),
        rails: meshes.add(placeholder_mesh()),
        lanes: meshes.add(placeholder_mesh()),
    };
    let tree_mat = vertex_coloured(&mut materials, lk, lk.surfaces.foliage);
    commands.spawn((Mesh3d(handles.trees.clone()), MeshMaterial3d(tree_mat.clone())));
    commands.insert_resource(SeasonMats { terrain: terrain_mat, trees: tree_mat, forced: forced.0 });
    commands.spawn((Mesh3d(handles.houses.clone()), MeshMaterial3d(vc.clone())));
    commands.spawn((Mesh3d(handles.works.clone()), MeshMaterial3d(vc.clone())));
    commands.insert_resource(TrainAssets { mesh: meshes.add(trains::train_mesh(lk).into_mesh()), mat: vc.clone() });
    let smoke_mat = materials.add(StandardMaterial { base_color: Color::WHITE, perceptual_roughness: 1.0, reflectance: 0.0, diffuse_transmission: lk.surfaces.smoke_translucence, ..default() });
    commands.spawn((Mesh3d(handles.smoke.clone()), MeshMaterial3d(smoke_mat), bevy::light::NotShadowCaster));
    commands.spawn((Mesh3d(handles.roads.clone()), MeshMaterial3d(flat.clone()), bevy::light::NotShadowCaster));
    commands.spawn((Mesh3d(handles.rails.clone()), MeshMaterial3d(flat.clone()), bevy::light::NotShadowCaster));
    commands.spawn((Mesh3d(handles.lanes.clone()), MeshMaterial3d(flat), bevy::light::NotShadowCaster));
    commands.insert_resource(handles);

    // a warm low sun from the south-west, and soft sky light; the seasons
    // move it from the first frame on, so this is only where it starts
    commands.spawn((
        Sun,
        DirectionalLight { color: lk.sun.colour(1.0), illuminance: lk.sun.strength(1.0), shadow_maps_enabled: true, ..default() },
        Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, lk.sun.turn, -lk.sun.height(1.0), 0.0)),
        CascadeShadowConfigBuilder { num_cascades: lk.shadows.cascades, first_cascade_far_bound: lk.shadows.first_far, maximum_distance: lk.shadows.max_distance, ..default() }.build(),
    ));

    // the camera
    let mut cam = commands.spawn((
        Camera3d::default(),
        MainCam,
        IsDefaultUiCamera,
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
        orbit_transform(&orbit),
    ));
    if lk.post.bloom > 0.0 && !debug.0 {
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
    if debug.0 {
        // flat class colours must reach the image unchanged
        cam.insert(Tonemapping::None);
    }
    if tilt.0 {
        cam.insert(DepthOfField {
            mode: DepthOfFieldMode::Gaussian,
            focal_distance: orbit.dist,
            aperture_f_stops: lk.tilt_shift.f_stops(orbit.dist),
            ..default()
        });
    }
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
    task: Option<(usize, Task<Scenery>)>,
}

fn build_scenery(r: &Replay, g: &Ground, slots: &scatter::Slots, cen: &[(f32, f32)], k: usize, lk: &Look) -> Scenery {
    let t0 = std::time::Instant::now();
    let cells = terrain::year_cells(r, k);
    let ym = scatter::build_year(r, g, slots, cen, k, lk);
    let t1 = t0.elapsed().as_secs_f32();
    let nets = network::year_nets(r, g, cen, k, lk);
    let rails = trains::rail_paths(r, g, cen, k);
    if std::env::var_os("ISOSTATE_TIMING").is_some() {
        eprintln!("year {}: scatter {:.0} ms, networks {:.0} ms (off the main thread)", r.y0 + k as i32, t1 * 1e3, (t0.elapsed().as_secs_f32() - t1) * 1e3);
    }
    Scenery { k, cells, ym, nets, rails }
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
    old_trains: Query<Entity, With<Train>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    // a finished build goes on screen
    if let Some(sc) = builds.task.as_mut().and_then(|(_, t)| check_ready(t)) {
        builds.task = None;
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
        let (ym, nets) = (sc.ym, sc.nets);
        for (h, b) in [(&handles.trees, ym.trees), (&handles.houses, ym.houses), (&handles.works, ym.works), (&handles.smoke, ym.smoke), (&handles.roads, nets.roads), (&handles.rails, nets.rails), (&handles.lanes, ym.lanes)] {
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
        let task = AsyncComputeTaskPool::get().spawn(async move { build_scenery(&r, &g, &slots, &cen, k, &lk) });
        builds.task = Some((k, task));
    }
}

/// The scenery shows the year on the clock (screenshots wait for it).
fn scenery_ready(tl: &Timeline, data: &Data) -> bool {
    tl.built == Some((tl.t.floor() as usize).min(data.r.ny - 1))
}

/// Phase of the year: while playing the months roll by; a paused year shows high summer.
fn season_phase(tl: &Timeline, forced: Option<f32>) -> f32 {
    if let Some(f) = forced {
        return f.rem_euclid(1.0);
    }
    if tl.playing { tl.t.fract() } else { 0.55 }
}

fn seasons(
    tl: Res<Timeline>,
    sm: Res<SeasonMats>,
    look: Res<TheLook>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ground_mats: ResMut<Assets<terrain::GroundMaterial>>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), With<Sun>>,
    mut last: Local<Option<i32>>,
) {
    let ph = season_phase(&tl, sm.forced);
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
    // trees: 0 midwinter, 0.25 spring, 0.5 summer, 0.75 autumn
    let leaf = &look.foliage;
    let keys: [(f32, [f32; 3]); 5] = [(0.0, leaf.tint_winter), (0.25, leaf.tint_spring), (0.5, leaf.tint_summer), (0.75, leaf.tint_autumn), (1.0, leaf.tint_winter)];
    let i = keys.iter().position(|k| k.0 > ph).unwrap_or(4).max(1);
    let (a, b) = (keys[i - 1], keys[i]);
    let t = ((ph - a.0) / (b.0 - a.0)).clamp(0.0, 1.0);
    if let Some(mut m) = materials.get_mut(&sm.trees) {
        m.base_color = Color::srgb(a.1[0] + (b.1[0] - a.1[0]) * t, a.1[1] + (b.1[1] - a.1[1]) * t, a.1[2] + (b.1[2] - a.1[2]) * t);
    }
    // the sun rides higher and warmer in summer
    let summer = 0.5 - 0.5 * (ph * std::f32::consts::TAU).cos();
    for (mut light, mut tf) in &mut sun {
        light.illuminance = look.sun.strength(summer);
        light.color = look.sun.colour(summer);
        *tf = Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, look.sun.turn, -look.sun.height(summer), 0.0));
    }
}

fn time_input(keys: Res<ButtonInput<KeyCode>>, mut tl: ResMut<Timeline>, data: Res<Data>, mut tilt: ResMut<Tilt>) {
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
    if keys.just_pressed(KeyCode::ArrowUp) {
        tl.speed = (tl.speed * 1.5).min(20.0);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        tl.speed = (tl.speed / 1.5).max(0.3);
    }
    if keys.just_pressed(KeyCode::KeyT) {
        tilt.0 = !tilt.0;
    }
}

fn advance_time(time: Res<Time>, mut tl: ResMut<Timeline>, data: Res<Data>) {
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
        o.dist = (o.dist * (-lines * 0.12).exp()).clamp(1.5, 220.0);
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
    if keys.pressed(KeyCode::KeyR) { o.dist = (o.dist * (1.0 - 1.5 * dt)).max(1.5); }
    if keys.pressed(KeyCode::KeyF) { o.dist = (o.dist * (1.0 + 1.5 * dt)).min(220.0); }
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
                commands.entity(e).insert(DepthOfField { mode: DepthOfFieldMode::Gaussian, focal_distance: o.dist, aperture_f_stops: look.tilt_shift.f_stops(o.dist), ..default() });
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

fn fmt_pop(p: f32) -> String {
    if p >= 1e6 { format!("{:.2} million", p / 1e6) } else { format!("{:.0},{:03.0}", (p / 1000.0).floor(), p % 1000.0) }
}

fn labels(
    data: Res<Data>,
    tl: Res<Timeline>,
    o: Res<Orbit>,
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
    order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let max_shown = if o.dist > 90.0 { 9 } else if o.dist > 45.0 { 16 } else { 99 };
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
                let size = 13.0 + 7.0 * (order[rank].1 / biggest).sqrt();
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
        for (i, e) in r.header.events.iter().enumerate() {
            if e.year.floor() as i32 == year && e.weight >= 0.6 {
                st.shown.push((i, now));
            }
        }
        while st.shown.len() > 3 {
            st.shown.remove(0);
        }
        st.last_k = Some(k);
        changed = true;
    }
    let before = st.shown.len();
    st.shown.retain(|&(_, t0)| now - t0 < 7.0);
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
        for (i, _) in shown.iter().rev() {
            let e = &r.header.events[*i];
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
                q.spawn(text(f, &format!("{} \u{00b7} {}", kind_label(&e.kind), e.year.floor() as i32), 13.0, kind_colour(&e.kind)));
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
    data: Res<Data>,
    sheet: Option<Res<shots::Sheet>>,
    mut patience: Local<u32>,
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
        tl.t = (y - r.y0).clamp(0, r.ny as i32 - 1) as f32;
    }
    if let Some(tab) = v.tab {
        if ui.tab != tab {
            ui.tab = tab;
        }
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
    let px: Vec<u8> = (0..w * h)
        .into_par_iter()
        .map(|i| {
            let (x, y) = ((i % w) as f32 + 0.5, (i / w) as f32 + 0.5);
            let d = cam.rotation * Vec3::new((x / w as f32 * 2.0 - 1.0) * tan * aspect, (1.0 - y / h as f32 * 2.0) * tan, -1.0);
            match mesh_hit(g, cam.translation, d.normalize()) {
                Some(p) if p.y >= 0.03 && p.x >= 0.0 && p.z >= 0.0 && p.x < r.w as f32 && p.z < r.h as f32 => world::class_at(r, g, k, p.x, p.z).0,
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
) {
    if keys.just_pressed(KeyCode::Escape) {
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
    mut panel: Query<&mut Visibility, With<InspectPanel>>,
    mut txt: Query<&mut Text, (With<InspectText>, Without<InspectTitle>)>,
    mut title: Query<&mut Text, (With<InspectTitle>, Without<InspectText>)>,
    mut last: Local<Option<(Option<Pick>, usize)>>,
) {
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    if *last == Some((picked.what, k)) {
        return;
    }
    *last = Some((picked.what, k));
    for mut v in &mut panel {
        *v = if picked.what.is_some() { Visibility::Inherited } else { Visibility::Hidden };
    }
    if let Some(w) = picked.what {
        let s = describe(&data.r, w, k);
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
