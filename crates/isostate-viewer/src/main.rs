//! Isolated State: a 3D replay viewer for runs of the island economy.
//!
//!     cargo run -p isostate-viewer --release -- runs/landlords/run.bin
//!     cargo run -p isostate-viewer --release -- runs/landlords/run.bin --year 1850 --shot out.png
//!
//! Space plays and pauses, the arrow keys step through years, drag to pan,
//! right-drag to turn, scroll to zoom.

mod meshkit;
mod network;
mod replay;
mod scatter;
mod terrain;
mod world;

use bevy::app::ScheduleRunnerPlugin;
use bevy::camera::{Hdr, RenderTarget};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::light::CascadeShadowConfigBuilder;
use bevy::pbr::{DistanceFog, FogFalloff, ScreenSpaceAmbientOcclusion};
use bevy::post_process::bloom::Bloom;
use bevy::post_process::dof::{DepthOfField, DepthOfFieldMode};
use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::{ExitCondition, WindowResolution};
use bevy::winit::WinitPlugin;

use replay::Replay;
use world::Ground;

// ------------------------------------------------------------------ command line

struct Args {
    run: String,
    year: Option<i32>,
    shot: Option<String>,
    size: (u32, u32),
    look: Option<(f32, f32)>,
    yaw: f32,
    pitch: f32,
    dist: Option<f32>,
    frames: u32,
    wait: f32,
    tilt: bool,
    views: Vec<ShotSpec>,
    season: Option<f32>,
    describe: Option<(f32, f32)>,
}

#[derive(Clone, Debug)]
struct ShotSpec {
    path: String,
    year: Option<i32>,
    look: Option<(f32, f32)>,
    yaw: f32,
    pitch: f32,
    dist: Option<f32>,
}

fn parse_args() -> Args {
    let mut a = Args {
        run: "runs/landlords/run.bin".into(),
        year: None,
        shot: None,
        size: (1600, 1000),
        look: None,
        yaw: 0.0,
        pitch: 52.0,
        dist: None,
        frames: 40,
        wait: 45.0,
        tilt: true,
        views: Vec::new(),
        season: None,
        describe: None,
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
            "--look" => {
                let s = next(i);
                if let Some((x, z)) = s.split_once(',') { a.look = Some((x.parse().unwrap_or(60.0), z.parse().unwrap_or(42.0))); }
                i += 1;
            }
            "--yaw" => { a.yaw = next(i).parse().unwrap_or(0.0); i += 1; }
            "--pitch" => { a.pitch = next(i).parse().unwrap_or(52.0); i += 1; }
            "--dist" => { a.dist = next(i).parse().ok(); i += 1; }
            "--frames" => { a.frames = next(i).parse().unwrap_or(40); i += 1; }
            "--wait" => { a.wait = next(i).parse().unwrap_or(45.0); i += 1; }
            "--no-tilt" => a.tilt = false,
            "--season" => { a.season = next(i).parse().ok(); i += 1; }
            "--describe" => {
                let s = next(i);
                if let Some((x, z)) = s.split_once(',') { a.describe = Some((x.parse().unwrap_or(0.0), z.parse().unwrap_or(0.0))); }
                i += 1;
            }
            "--view" => {
                // path,year,x,z,yaw,pitch,dist
                let p: Vec<String> = next(i).split(',').map(String::from).collect();
                let num = |j: usize| p.get(j).and_then(|s| s.parse::<f32>().ok());
                a.views.push(ShotSpec {
                    path: p.first().cloned().unwrap_or_else(|| "shot.png".into()),
                    year: num(1).map(|y| y as i32),
                    look: match (num(2), num(3)) { (Some(x), Some(z)) => Some((x, z)), _ => None },
                    yaw: num(4).unwrap_or(0.0),
                    pitch: num(5).unwrap_or(52.0),
                    dist: num(6),
                });
                i += 1;
            }
            s if !s.starts_with("--") => a.run = s.to_string(),
            _ => {}
        }
        i += 1;
    }
    a
}

// ------------------------------------------------------------------ resources and markers

#[derive(Resource)]
struct Data {
    r: Replay,
    g: Ground,
    ts: terrain::TexStatic,
    slots: scatter::Slots,
    cen: Vec<(f32, f32)>,
}

#[derive(Resource)]
struct Handles {
    terrain_img: Handle<Image>,
    trees: Handle<Mesh>,
    houses: Handle<Mesh>,
    works: Handle<Mesh>,
    smoke: Handle<Mesh>,
    roads: Handle<Mesh>,
    rails: Handle<Mesh>,
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
    terrain: Handle<StandardMaterial>,
    trees: Handle<StandardMaterial>,
    /// fixed phase of the year for stills (0 = midwinter, 0.5 = midsummer)
    forced: Option<f32>,
}

#[derive(Component)]
struct Sun;

#[derive(Component)]
struct MainCam;
#[derive(Component)]
struct YearText;
#[derive(Component)]
struct InfoText;
#[derive(Component)]
struct ToastBox;
#[derive(Component)]
struct TownLabel(usize);
#[derive(Component)]
struct InspectPanel;
#[derive(Component)]
struct InspectText;

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
    let t0 = std::time::Instant::now();
    let r = match replay::load(&args.run) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}\nMake a run first, for example:\n  python -m isostate run --seed 7 --regime landlords --out runs/landlords");
            std::process::exit(1);
        }
    };
    let g = world::build_ground(&r);
    let ts = terrain::tex_static(&r, &g);
    let slots = scatter::make_slots(&r, &g);
    let cen = network::centres(&r);
    eprintln!("prepared {} in {:.1}s", args.run, t0.elapsed().as_secs_f32());

    let k0 = match args.year {
        Some(y) => (y - r.y0).clamp(0, r.ny as i32 - 1) as usize,
        None => r.ny - 1,
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
    let look = args.look.unwrap_or(((x0 + x1) * 0.5, (z0 + z1) * 0.5 + 4.0));
    let dist = args.dist.unwrap_or((x1 - x0).max((z1 - z0) * 1.5) * 1.05);
    let orbit = Orbit {
        target: Vec3::new(look.0, world::height_at(&g, look.0, look.1).max(0.0), look.1),
        yaw: args.yaw.to_radians(),
        pitch: args.pitch.to_radians(),
        dist,
    };

    let mut app = App::new();
    let mut views = args.views.clone();
    if let Some(path) = args.shot.clone() {
        views.insert(0, ShotSpec { path, year: args.year, look: args.look, yaw: args.yaw, pitch: args.pitch, dist: args.dist });
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
    app.insert_resource(ClearColor(Color::srgb(0.70, 0.83, 0.92)))
        .insert_resource(GlobalAmbientLight { color: Color::srgb(0.78, 0.86, 1.0), brightness: 650.0, ..default() })
        .insert_resource(Data { r, g, ts, slots, cen })
        .insert_resource(Timeline { t: k0 as f32, built: None, playing: false, speed: 2.5 })
        .insert_resource(orbit)
        .insert_resource(Tilt(args.tilt))
        .insert_resource(ForcedSeason(args.season))
        .init_resource::<Toasts>()
        .init_resource::<Picked>()
        .add_systems(Startup, (setup_world, setup_ui).chain())
        .add_systems(Update, (time_input, advance_time, rebuild_year, seasons, camera_input, apply_camera, labels, info_text, toasts, pick, inspect_panel).chain());
    if headless {
        app.insert_resource(ShotPlanArgs { views, size: args.size, frames: args.frames, wait: args.wait, default_dist: dist, default_look: look });
        app.add_systems(Update, take_shot.after(toasts));
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

fn vertex_coloured(materials: &mut Assets<StandardMaterial>, rough: f32) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial { base_color: Color::WHITE, perceptual_roughness: rough, reflectance: 0.25, ..default() })
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
    tl: Res<Timeline>,
    orbit: Res<Orbit>,
    tilt: Res<Tilt>,
    shot: Option<Res<ShotPlanArgs>>,
    forced: Res<ForcedSeason>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut fonts: ResMut<Assets<Font>>,
) {
    let (r, g) = (&data.r, &data.g);
    let k = tl.t.floor() as usize;
    commands.insert_resource(UiFont(fonts.add(Font::from_bytes(FREDOKA.to_vec()))));

    // ground: one heightfield with the year's patchwork painted on it
    let mut rgba = vec![0u8; data.ts.tw * data.ts.th * 4];
    terrain::paint(r, &data.ts, k, &mut rgba);
    let terrain_img = images.add(terrain::make_image(data.ts.tw, data.ts.th, rgba));
    let terrain_mat = materials.add(StandardMaterial {
        base_color_texture: Some(terrain_img.clone()),
        perceptual_roughness: 0.92,
        reflectance: 0.18,
        ..default()
    });
    commands.spawn((Mesh3d(meshes.add(terrain::terrain_mesh(g))), MeshMaterial3d(terrain_mat.clone())));

    // the sea: a clear sheet of water over the painted sea floor, and a deep floor beyond the map
    let (w, h) = (r.w as f32, r.h as f32);
    commands.spawn((
        Mesh3d(meshes.add(terrain::plane(-4.0 * w, -4.0 * h, 5.0 * w, 5.0 * h, -1.15))),
        MeshMaterial3d(materials.add(StandardMaterial { base_color: Color::srgb(0.16, 0.36, 0.48), perceptual_roughness: 1.0, ..default() })),
    ));
    commands.spawn((
        Mesh3d(meshes.add(terrain::plane(-4.0 * w, -4.0 * h, 5.0 * w, 5.0 * h, 0.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.28, 0.60, 0.74, 0.58),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.12,
            reflectance: 0.45,
            ..default()
        })),
        bevy::light::NotShadowCaster,
    ));

    // rivers do not change
    let river_mat = materials.add(StandardMaterial { base_color: Color::WHITE, perceptual_roughness: 0.25, reflectance: 0.4, ..default() });
    commands.spawn((Mesh3d(meshes.add(non_empty(network::rivers(r, g, &data.cen)))), MeshMaterial3d(river_mat), bevy::light::NotShadowCaster));

    // year meshes start empty; rebuild_year fills them
    let vc = vertex_coloured(&mut materials, 0.85);
    let flat = vertex_coloured(&mut materials, 0.95);
    let handles = Handles {
        terrain_img,
        trees: meshes.add(placeholder_mesh()),
        houses: meshes.add(placeholder_mesh()),
        works: meshes.add(placeholder_mesh()),
        smoke: meshes.add(placeholder_mesh()),
        roads: meshes.add(placeholder_mesh()),
        rails: meshes.add(placeholder_mesh()),
    };
    let tree_mat = vertex_coloured(&mut materials, 0.9);
    commands.spawn((Mesh3d(handles.trees.clone()), MeshMaterial3d(tree_mat.clone())));
    commands.insert_resource(SeasonMats { terrain: terrain_mat, trees: tree_mat, forced: forced.0 });
    commands.spawn((Mesh3d(handles.houses.clone()), MeshMaterial3d(vc.clone())));
    commands.spawn((Mesh3d(handles.works.clone()), MeshMaterial3d(vc.clone())));
    let smoke_mat = materials.add(StandardMaterial { base_color: Color::WHITE, perceptual_roughness: 1.0, reflectance: 0.0, diffuse_transmission: 0.3, ..default() });
    commands.spawn((Mesh3d(handles.smoke.clone()), MeshMaterial3d(smoke_mat), bevy::light::NotShadowCaster));
    commands.spawn((Mesh3d(handles.roads.clone()), MeshMaterial3d(flat.clone()), bevy::light::NotShadowCaster));
    commands.spawn((Mesh3d(handles.rails.clone()), MeshMaterial3d(flat), bevy::light::NotShadowCaster));
    commands.insert_resource(handles);

    // a warm low sun from the south-west, and soft sky light
    commands.spawn((
        Sun,
        DirectionalLight { color: Color::srgb(1.0, 0.94, 0.84), illuminance: 9000.0, shadow_maps_enabled: true, ..default() },
        Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, -0.75, -0.72, 0.0)),
        CascadeShadowConfigBuilder { num_cascades: 3, first_cascade_far_bound: 12.0, maximum_distance: 220.0, ..default() }.build(),
    ));

    // the camera
    let mut cam = commands.spawn((
        Camera3d::default(),
        MainCam,
        IsDefaultUiCamera,
        Hdr,
        Msaa::Off,
        Projection::Perspective(PerspectiveProjection { fov: 30f32.to_radians(), near: 0.05, far: 2000.0, ..default() }),
        Tonemapping::TonyMcMapface,
        Bloom { intensity: 0.06, ..Bloom::NATURAL },
        ScreenSpaceAmbientOcclusion::default(),
        bevy::anti_alias::smaa::Smaa::default(),
        DistanceFog {
            color: Color::srgba(0.72, 0.84, 0.93, 1.0),
            directional_light_color: Color::srgba(1.0, 0.93, 0.8, 0.4),
            directional_light_exponent: 24.0,
            falloff: FogFalloff::Linear { start: orbit.dist * 1.2, end: orbit.dist * 4.0 },
        },
        orbit_transform(&orbit),
    ));
    if tilt.0 {
        cam.insert(DepthOfField {
            mode: DepthOfFieldMode::Gaussian,
            focal_distance: orbit.dist,
            aperture_f_stops: tilt_f_stops(orbit.dist),
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
        });
    }
}

/// Miniature look: shallow focus when far away, gentler up close.
fn tilt_f_stops(dist: f32) -> f32 {
    (30.0 / dist).clamp(0.8, 3.5)
}

fn orbit_transform(o: &Orbit) -> Transform {
    let dir = Vec3::new(o.yaw.sin() * o.pitch.cos(), o.pitch.sin(), o.yaw.cos() * o.pitch.cos());
    Transform::from_translation(o.target + dir * o.dist).looking_at(o.target, Vec3::Y)
}

// ------------------------------------------------------------------ the year

fn rebuild_year(
    mut tl: ResMut<Timeline>,
    data: Res<Data>,
    handles: Res<Handles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    if tl.built == Some(k) {
        return;
    }
    tl.built = Some(k);
    let (r, g) = (&data.r, &data.g);
    let mut rgba = vec![0u8; data.ts.tw * data.ts.th * 4];
    terrain::paint(r, &data.ts, k, &mut rgba);
    if let Some(mut img) = images.get_mut(&handles.terrain_img) {
        terrain::update_image(&mut img, data.ts.tw, data.ts.th, rgba);
    }
    let ym = scatter::build_year(r, g, &data.slots, &data.cen, k);
    let nets = network::year_nets(r, g, &data.cen, k);
    for (h, b) in [(&handles.trees, ym.trees), (&handles.houses, ym.houses), (&handles.works, ym.works), (&handles.smoke, ym.smoke), (&handles.roads, nets.roads), (&handles.rails, nets.rails)] {
        if let Some(mut m) = meshes.get_mut(h) {
            *m = non_empty(b);
        }
    }
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), With<Sun>>,
    mut last: Local<Option<i32>>,
) {
    let ph = season_phase(&tl, sm.forced);
    // only touch the materials when the season visibly moves
    let step = (ph * 48.0).floor() as i32;
    if *last == Some(step) {
        return;
    }
    *last = Some(step);
    // 0 midwinter, 0.25 spring, 0.5 summer, 0.75 autumn
    let keys: [(f32, [f32; 3], [f32; 3]); 5] = [
        (0.0, [1.04, 1.06, 1.12], [0.78, 0.74, 0.70]),
        (0.25, [0.96, 1.04, 0.94], [0.92, 1.12, 0.86]),
        (0.5, [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]),
        (0.75, [1.06, 0.97, 0.86], [1.35, 0.88, 0.52]),
        (1.0, [1.04, 1.06, 1.12], [0.78, 0.74, 0.70]),
    ];
    let i = keys.iter().position(|k| k.0 > ph).unwrap_or(4).max(1);
    let (a, b) = (keys[i - 1], keys[i]);
    let t = ((ph - a.0) / (b.0 - a.0)).clamp(0.0, 1.0);
    let l = |x: [f32; 3], y: [f32; 3]| Color::srgb(x[0] + (y[0] - x[0]) * t, x[1] + (y[1] - x[1]) * t, x[2] + (y[2] - x[2]) * t);
    if let Some(mut m) = materials.get_mut(&sm.terrain) {
        m.base_color = l(a.1, b.1);
    }
    if let Some(mut m) = materials.get_mut(&sm.trees) {
        m.base_color = l(a.2, b.2);
    }
    // the sun rides higher and warmer in summer
    let summer = 0.5 - 0.5 * (ph * std::f32::consts::TAU).cos();
    for (mut light, mut tf) in &mut sun {
        light.illuminance = 7000.0 + 3000.0 * summer;
        light.color = Color::srgb(1.0, 0.90 + 0.05 * summer, 0.78 + 0.07 * summer);
        *tf = Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, -0.75, -0.5 - 0.35 * summer, 0.0));
    }
}

fn time_input(keys: Res<ButtonInput<KeyCode>>, mut tl: ResMut<Timeline>, data: Res<Data>, mut tilt: ResMut<Tilt>) {
    let last = (data.r.ny - 1) as f32;
    if keys.just_pressed(KeyCode::Space) {
        if !tl.playing && tl.t >= last {
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
    let last = (data.r.ny - 1) as f32;
    tl.t = (tl.t + time.delta_secs() * tl.speed).min(last);
    if tl.t >= last {
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
) {
    let dt = time.delta_secs();
    let fwd = Vec3::new(-o.yaw.sin(), 0.0, -o.yaw.cos());
    let right = Vec3::new(o.yaw.cos(), 0.0, -o.yaw.sin());
    let pan = o.dist * 0.0016;
    if buttons.pressed(MouseButton::Left) {
        let d = motion.delta;
        o.target -= right * d.x * pan;
        o.target += fwd * d.y * pan;
    }
    if buttons.pressed(MouseButton::Right) {
        let d = motion.delta;
        o.yaw -= d.x * 0.005;
        o.pitch = (o.pitch + d.y * 0.004).clamp(0.22, 1.5);
    }
    let lines = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / 40.0,
    };
    if lines != 0.0 {
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
    mut commands: Commands,
    mut cams: Query<(Entity, &mut Transform, &mut DistanceFog, Option<&mut DepthOfField>), With<MainCam>>,
) {
    for (e, mut t, mut fog, dof) in &mut cams {
        *t = orbit_transform(&o);
        fog.falloff = FogFalloff::Linear { start: o.dist * 1.2, end: o.dist * 4.0 };
        match (tilt.0, dof) {
            (true, Some(mut d)) => {
                d.focal_distance = o.dist;
                d.aperture_f_stops = tilt_f_stops(o.dist);
            }
            (true, None) => {
                commands.entity(e).insert(DepthOfField { mode: DepthOfFieldMode::Gaussian, focal_distance: o.dist, aperture_f_stops: tilt_f_stops(o.dist), ..default() });
            }
            (false, Some(_)) => {
                commands.entity(e).remove::<DepthOfField>();
            }
            _ => {}
        }
    }
}

// ------------------------------------------------------------------ interface

const INK: Color = Color::srgb(0.20, 0.18, 0.16);
const PAPER: Color = Color::srgba(0.99, 0.97, 0.92, 0.88);

fn text(font: &Handle<Font>, s: &str, size: f32, color: Color) -> (Text, TextFont, TextColor) {
    (Text::new(s), TextFont { font: font.clone().into(), font_size: FontSize::Px(size), ..default() }, TextColor(color))
}

fn setup_ui(mut commands: Commands, font: Res<UiFont>, data: Res<Data>) {
    let f = &font.0;
    // title card, top left
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(18.0),
                top: Val::Px(16.0),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(Val::Px(14.0)),
                ..default()
            },
            BackgroundColor(PAPER),
        ))
        .with_children(|p| {
            p.spawn(text(f, "Isolated State", 20.0, Color::srgb(0.42, 0.36, 0.30)));
            p.spawn((text(f, "1900", 54.0, INK), YearText));
            p.spawn((text(f, "", 16.0, Color::srgb(0.35, 0.32, 0.28)), InfoText));
        });
    // help, bottom left
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(18.0),
                bottom: Val::Px(14.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.99, 0.97, 0.92, 0.7)),
        ))
        .with_children(|p| {
            p.spawn(text(f, "Space play   Left/Right year   Up/Down speed   drag pan   right-drag turn   scroll zoom   T tilt-shift", 14.0, Color::srgb(0.35, 0.32, 0.28)));
        });
    // news, top right
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(18.0),
            top: Val::Px(16.0),
            width: Val::Px(340.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },
        ToastBox,
    ));
    // the inspector, bottom right
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(18.0),
                bottom: Val::Px(16.0),
                width: Val::Px(380.0),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(Val::Px(14.0)),
                ..default()
            },
            BackgroundColor(PAPER),
            Visibility::Hidden,
            InspectPanel,
        ))
        .with_children(|p| {
            p.spawn((text(f, "", 15.0, INK), InspectText));
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

fn info_text(tl: Res<Timeline>, data: Res<Data>, mut year: Query<&mut Text, (With<YearText>, Without<InfoText>)>, mut info: Query<&mut Text, (With<InfoText>, Without<YearText>)>) {
    let r = &data.r;
    let k = (tl.t.floor() as usize).min(r.ny - 1);
    for mut t in &mut year {
        t.0 = format!("{}", r.y0 + k as i32);
    }
    let pop = replay::annual(r, "pop", k).unwrap_or(0.0);
    let rail = replay::annual(r, "rail_km", k).unwrap_or(0.0);
    let towns = (0..r.n_towns).filter(|&t| r.town_active[k * r.n_towns + t] > 0).count();
    let mut s = format!("{} people \u{00b7} {} towns", fmt_pop(pop), towns);
    if rail >= 1.0 {
        s.push_str(&format!(" \u{00b7} {:.0} km of railway", rail));
    }
    if tl.playing {
        s.push_str(&format!("   \u{25b6} {:.1} years a second", tl.speed));
    }
    for mut t in &mut info {
        t.0 = s.clone();
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

fn kind_colour(kind: &str) -> Color {
    match kind {
        "discovery" | "first" => Color::srgb(0.75, 0.52, 0.12),
        "railway" | "rail_plan" => Color::srgb(0.25, 0.27, 0.32),
        "milestone" => Color::srgb(0.28, 0.55, 0.33),
        "crossing" => Color::srgb(0.16, 0.48, 0.53),
        "steep" => Color::srgb(0.48, 0.36, 0.62),
        "alarm" | "famine" => Color::srgb(0.70, 0.25, 0.22),
        "town" => Color::srgb(0.55, 0.40, 0.23),
        _ => Color::srgb(0.45, 0.45, 0.45),
    }
}

fn kind_label(kind: &str) -> &'static str {
    match kind {
        "discovery" => "Invention",
        "first" => "First output",
        "railway" => "Railway",
        "rail_plan" => "Railway scheme",
        "milestone" => "Milestone",
        "crossing" => "Turning point",
        "steep" => "Markets",
        "alarm" => "Hard times",
        "verdict" => "In hindsight",
        "famine" => "Hunger",
        "town" => "New town",
        "closure" => "Closure",
        _ => "News",
    }
}

fn toasts(
    mut commands: Commands,
    mut st: ResMut<Toasts>,
    tl: Res<Timeline>,
    time: Res<Time>,
    data: Res<Data>,
    font: Res<UiFont>,
    boxq: Query<(Entity, Option<&Children>), With<ToastBox>>,
) {
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
    let Ok((boxe, children)) = boxq.single() else { return };
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
    data: Res<Data>,
) {
    let Some(mut plan) = plan else { return };
    if plan.index >= plan.views.len() {
        exit.write(AppExit::Success);
        return;
    }
    let v = plan.views[plan.index].clone();
    // point the camera and the clock at this view
    let r = &data.r;
    if let Some(y) = v.year {
        tl.t = (y - r.y0).clamp(0, r.ny as i32 - 1) as f32;
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
    // the first view waits for shaders to compile on software renderers
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
    mut picked: ResMut<Picked>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        picked.what = None;
    }
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    if buttons.just_pressed(MouseButton::Left) {
        picked.press = Some(cursor);
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
            let title = { let n = DISP_NAMES[a % 12]; let mut t = n.to_string(); if let Some(f) = t.get_mut(0..1) { f.make_ascii_uppercase(); } t };
            let mut s = format!("{}, {} km east and {} km south of the north-west corner\n", title, (c % r.w) * 2, (c / r.w) * 2);
            if a == b {
                s.push_str(&format!("All {} in {}.", DISP_NAMES[a % 12], year));
            } else {
                s.push_str(&format!("{:.0}% {} and {:.0}% {} in {}.", sh * 100.0, DISP_NAMES[a % 12], (1.0 - sh) * 100.0, DISP_NAMES[b % 12], year));
            }
            let t = r.catch[off] as i64 - 1;
            if t >= 0 {
                if let Some(town) = r.header.towns.get(t as usize) {
                    s.push_str(&format!("\nSells its produce in {}.", town.name));
                }
            }
            s
        }
    }
}

fn inspect_panel(
    picked: Res<Picked>,
    tl: Res<Timeline>,
    data: Res<Data>,
    mut panel: Query<&mut Visibility, With<InspectPanel>>,
    mut txt: Query<&mut Text, With<InspectText>>,
    mut last: Local<Option<(Option<Pick>, usize)>>,
) {
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    if *last == Some((picked.what, k)) {
        return;
    }
    *last = Some((picked.what, k));
    for mut v in &mut panel {
        *v = if picked.what.is_some() { Visibility::Visible } else { Visibility::Hidden };
    }
    if let Some(w) = picked.what {
        let s = describe(&data.r, w, k);
        for mut t in &mut txt {
            t.0 = s.clone();
        }
    }
}
