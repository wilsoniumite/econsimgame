//! The lab: contact sheets of the generators, in the game's light, through
//! the game's camera, with no game.
//!
//!     isostate-viewer --lab list
//!     isostate-viewer --lab cottages --out out/lab
//!     isostate-viewer --lab all --out out/lab --season 0
//!
//! A sheet is a grid of cells, each one call of a generator with a label
//! under it saying what went in (the sheets are in the generator crate,
//! `isostate_gen::sheets`). The lab stands them on plain ground of the colour
//! the island has under such things, ruled faintly for scale, and writes a
//! picture from each of the camera's standard distances: `NAME-close.png` as
//! near as the game goes, `NAME-play.png` as a new game opens, `NAME-far.png`
//! about half way out to the whole island. A sheet too long for one close
//! picture goes over several (`NAME-close-2.png`). An `index.html` beside
//! them shows every picture in the directory.
//!
//! From play distance a house is seven pixels wide, so the play and far
//! pictures show the middle of the true picture enlarged a whole number of
//! times, each pixel a square: what is there to see is exactly what the game
//! draws, only bigger. `--loupe N` enlarges the close picture the same way.
//!
//! The sun, the sky light, the shadows, the ambient occlusion, the haze, the
//! tonemapping and the lens are the look's, set up by the same functions the
//! game uses. The tilt-shift is off unless `--tilt` is given, as it blurs
//! what the sheet is for looking at. `--season X` is the game's: 0 midwinter,
//! 0.5 midsummer; it moves the sun, turns the leaves and, in winter, the grass.
//!
//! Each thing is turned 30 degrees to its right about its own middle, so
//! that a gable shows as well as a front; `--turn DEG` turns it otherwise
//! (`--turn 0` is square on). A row too wide for a close picture is broken
//! into lines, and a single thing too big for one (a scene) is seen from as
//! near as shows the whole of it, which the picture's heading then says.

mod layout;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::app::ScheduleRunnerPlugin;
use bevy::camera::RenderTarget;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::image::ImageSampler;
use bevy::pbr::DistanceFog;
use bevy::post_process::dof::DepthOfField;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use isostate_gen::sheets::{Cell, Land, Sheet, SHEETS};
use isostate_gen::Model;

use crate::look::{Colour, Look, TheLook, ToColor};
use crate::meshkit::{self, shade, Builder};
use crate::ui::{text, INK, MUTED, PAPER};
use crate::{shots, terrain, Orbit, PipelinesWaiting, Sun, UiFont};
use layout::{Page, Shot};

/// What the command line asked of the lab.
pub struct LabArgs {
    /// a sheet's name, "all" or "list"
    pub sheet: String,
    pub out: Option<String>,
    pub size: (u32, u32),
    /// phase of the year: 0 midwinter, 0.5 midsummer
    pub season: Option<f32>,
    pub tilt: bool,
    /// how many times to enlarge the close picture
    pub loupe: u32,
    /// how far each thing is turned about its own middle, in degrees (the lab's own, if not given)
    pub turn: Option<f32>,
}

/// How far the lab turns each thing about its own middle unless told otherwise,
/// in degrees. Seen square on from the game's tilt a house is a rectangle of
/// roof over a strip of wall; turned a little it shows a gable end too, as
/// nearly every house on the island does.
pub(crate) const TURN: f32 = -30.0;

/// The season a paused game shows: high summer.
const HIGH_SUMMER: f32 = 0.55;

/// One picture to take.
struct Job {
    sheet: usize,
    shot: Shot,
    page: usize,
    of: usize,
    path: PathBuf,
}

#[derive(Resource)]
struct Lab {
    sheets: Vec<(&'static Sheet, Vec<Vec<Cell>>, Vec<Vec<Page>>)>,
    jobs: Vec<Job>,
    index: usize,
    phase: Phase,
    size: (u32, u32),
    season: f32,
    tilt: bool,
    turn: f32,
    out: PathBuf,
    started: std::time::Instant,
}

#[derive(Clone, Copy)]
enum Phase {
    /// put the next picture's things in front of the camera
    Arrange,
    /// let the renderer draw them for this many frames more
    Settle(u32),
    /// the picture has been asked for: wait for its file
    Written(u32),
}

/// What the lab keeps from one picture to the next.
#[derive(Resource)]
struct Kit {
    scene: Handle<Image>,
    picture: Handle<Image>,
    ui_camera: Entity,
    solid: Handle<StandardMaterial>,
    green: Handle<StandardMaterial>,
    smoke: Handle<StandardMaterial>,
    flat: Handle<StandardMaterial>,
    ground: Handle<StandardMaterial>,
}

/// Everything that belongs to one picture and goes when the next is arranged.
#[derive(Component)]
struct OfThePicture;

#[derive(Component)]
struct SceneCam;

/// Run the lab and stop: the exit code for the process.
pub fn run(args: LabArgs, look: Arc<Look>) -> i32 {
    if args.sheet == "list" {
        let wide = SHEETS.iter().map(|s| s.name.len()).max().unwrap_or(0);
        for s in SHEETS {
            println!("{:<wide$}  {}", s.name, s.about);
        }
        return 0;
    }
    let chosen: Vec<&'static Sheet> = if args.sheet == "all" {
        SHEETS.iter().collect()
    } else {
        match isostate_gen::sheets::find(&args.sheet) {
            Some(s) => vec![s],
            None => {
                let names: Vec<&str> = SHEETS.iter().map(|s| s.name).collect();
                eprintln!("there is no sheet called \"{}\". The sheets are: {}; or \"all\" for every one.", args.sheet, names.join(", "));
                return 1;
            }
        }
    };
    let Some(out) = args.out.as_ref().map(PathBuf::from) else {
        eprintln!("say where the pictures go: --lab {} --out DIR", args.sheet);
        return 1;
    };
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("cannot make {}: {e}", out.display());
        return 1;
    }

    let turn = args.turn.unwrap_or(TURN);
    // lay every sheet out for every distance before anything is drawn
    let mut sheets = Vec::new();
    let mut jobs = Vec::new();
    for (i, sheet) in chosen.into_iter().enumerate() {
        let cells = cells_of(sheet, &look.palette, turn);
        // pictures of this sheet from an earlier run would be mistaken for this run's
        clear_pictures(&out, sheet.name);
        let mut by_shot = Vec::new();
        for shot in Shot::ALL {
            let pages = layout::pages(&cells, shot, args.size, args.loupe);
            for (p, page) in pages.iter().enumerate() {
                let name = if pages.len() > 1 && p > 0 { format!("{}-{}-{}.png", sheet.name, shot.name(), p + 1) } else { format!("{}-{}.png", sheet.name, shot.name()) };
                if page.cut_off {
                    eprintln!("{name}: a row of this sheet is too big for the picture and is cut off");
                }
                jobs.push(Job { sheet: i, shot, page: p, of: pages.len(), path: out.join(name) });
            }
            by_shot.push(pages);
        }
        sheets.push((sheet, cells, by_shot));
    }

    let season = args.season.unwrap_or(HIGH_SUMMER).rem_euclid(1.0);
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, ..default() }).disable::<WinitPlugin>())
        .add_plugins(ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 60.0)))
        .insert_resource(ClearColor(look.sky.background.color()))
        .insert_resource(GlobalAmbientLight { color: look.sky.light.color(), brightness: look.sky.brightness, ..default() })
        .insert_resource(TheLook(look))
        .insert_resource(Lab { sheets, jobs, index: 0, phase: Phase::Arrange, size: args.size, season, tilt: args.tilt, turn, out, started: std::time::Instant::now() })
        .add_systems(Startup, setup)
        .add_systems(Update, step);
    let waiting = PipelinesWaiting::default();
    app.insert_resource(waiting.clone());
    if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
        render.insert_resource(waiting).add_systems(bevy::render::Render, crate::count_waiting_pipelines.in_set(bevy::render::RenderSystems::Cleanup));
    }
    app.run();
    0
}

/// A sheet's cells, each thing turned `turn` degrees about its own middle.
pub(crate) fn cells_of(sheet: &Sheet, palette: &isostate_gen::Palette, turn: f32) -> Vec<Vec<Cell>> {
    let mut cells = (sheet.build)(palette);
    for cell in cells.iter_mut().flatten() {
        cell.model.turn(turn.to_radians());
    }
    cells
}

/// Remove a sheet's pictures from a directory.
fn clear_pictures(dir: &Path, sheet: &str) {
    for path in pictures_of(dir, sheet).into_iter().flat_map(|(_, files)| files) {
        let _ = std::fs::remove_file(dir.join(path));
    }
}

/// A sheet's pictures in a directory, by distance, each distance's pages in order.
fn pictures_of(dir: &Path, sheet: &str) -> Vec<(Shot, Vec<String>)> {
    let mut out = Vec::new();
    for shot in Shot::ALL {
        let first = format!("{sheet}-{}.png", shot.name());
        let mut files = Vec::new();
        if dir.join(&first).exists() {
            files.push(first);
            for p in 2.. {
                let next = format!("{sheet}-{}-{p}.png", shot.name());
                if !dir.join(&next).exists() {
                    break;
                }
                files.push(next);
            }
        }
        out.push((shot, files));
    }
    out
}

fn setup(mut commands: Commands, look: Res<TheLook>, lab: Res<Lab>, mut images: ResMut<Assets<Image>>, mut materials: ResMut<Assets<StandardMaterial>>, mut fonts: ResMut<Assets<Font>>) {
    let lk: &Look = &look;
    commands.insert_resource(UiFont(fonts.add(Font::from_bytes(crate::FREDOKA.to_vec()))));

    // the game's sun and the game's camera, which draws into a picture of its own
    crate::spawn_sun(&mut commands, lk);
    let mut scene = Image::new_target_texture(lab.size.0, lab.size.1, TextureFormat::Rgba8UnormSrgb, None);
    // enlarged, each of its pixels is a square
    scene.sampler = ImageSampler::nearest();
    let scene = images.add(scene);
    let orbit = Orbit { target: Vec3::ZERO, yaw: 0.0, pitch: shots::PLAY_PITCH.to_radians(), dist: shots::PLAY_DIST };
    let mut cam = crate::spawn_view_camera(&mut commands, lk, &orbit, lab.tilt, false);
    cam.insert((SceneCam, Camera { order: -1, ..default() }, RenderTarget::Image(scene.clone().into())));

    // the finished picture: the camera's picture (or its middle, enlarged) with the labels over it.
    // The interface needs a camera to be drawn by; this one looks away from everything.
    let mut picture = Image::new_target_texture(lab.size.0, lab.size.1, TextureFormat::Rgba8UnormSrgb, None);
    picture.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let picture = images.add(picture);
    let ui_camera = commands
        .spawn((
            Camera3d::default(),
            Camera { order: 0, clear_color: ClearColorConfig::Custom(PAPER.with_alpha(1.0)), ..default() },
            Msaa::Off,
            Tonemapping::None,
            RenderTarget::Image(picture.clone().into()),
            Transform::from_xyz(0.0, -9000.0, 0.0).looking_to(Vec3::NEG_Y, Vec3::Z),
            IsDefaultUiCamera,
        ))
        .id();

    commands.insert_resource(Kit {
        scene,
        picture,
        ui_camera,
        solid: crate::vertex_coloured(&mut materials, lk, lk.surfaces.buildings),
        green: crate::vertex_coloured(&mut materials, lk, lk.surfaces.foliage),
        smoke: materials.add(crate::smoke_material(lk)),
        flat: crate::vertex_coloured(&mut materials, lk, lk.surfaces.paths),
        ground: materials.add(StandardMaterial { perceptual_roughness: lk.surfaces.ground, reflectance: lk.surfaces.ground_reflectance, ..default() }),
    });
}

/// The colour of the lab's ground under a sheet: what the island's ground
/// shader paints under such things, plain, and its winter colour in winter.
fn ground_colour(lk: &Look, land: Land, season: f32) -> Colour {
    let winter = (season * std::f32::consts::TAU).cos() > 0.5;
    match (land, winter) {
        (Land::Pasture, false) => lk.land.pasture,
        (Land::Pasture, true) => lk.land.grass_winter,
        (Land::Town, _) => lk.land.town_earth,
        (Land::Yard, _) => lk.land.yard,
        (Land::Wood, false) => lk.land.wood,
        (Land::Wood, true) => lk.land.wood_winter,
        (Land::Heath, false) => lk.land.heath,
        (Land::Heath, true) => lk.land.heath_brown,
    }
}

fn season_word(season: f32) -> &'static str {
    if (season - 0.5).abs() < 0.13 {
        "summer"
    } else if (season - 0.25).abs() <= 0.12 {
        "spring"
    } else if (season - 0.75).abs() <= 0.12 {
        "autumn"
    } else {
        "winter"
    }
}

/// The spacing of the ground's ruling: the finest of these that is at least
/// forty pixels in the picture, and what to call it (a map unit is 2 km).
fn ruling(px_per_unit: f32) -> (f32, &'static str) {
    let steps = [(0.05, "100 m"), (0.125, "250 m"), (0.25, "500 m"), (0.5, "1 km"), (1.0, "2 km, one square of the map"), (2.5, "5 km")];
    steps.into_iter().find(|(s, _)| s * px_per_unit >= 40.0).unwrap_or(steps[steps.len() - 1])
}

#[allow(clippy::too_many_arguments)]
fn step(
    mut commands: Commands,
    mut lab: ResMut<Lab>,
    kit: Res<Kit>,
    look: Res<TheLook>,
    font: Res<UiFont>,
    waiting: Res<PipelinesWaiting>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    old: Query<Entity, With<OfThePicture>>,
    mut cams: Query<(&mut Transform, &mut DistanceFog, Option<&mut DepthOfField>), With<SceneCam>>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), (With<Sun>, Without<SceneCam>)>,
    mut exit: MessageWriter<AppExit>,
    mut held: Local<Option<std::time::Instant>>,
) {
    let lk: &Look = &look;
    match lab.phase {
        Phase::Arrange => {
            for e in &old {
                commands.entity(e).despawn();
            }
            if lab.index >= lab.jobs.len() {
                write_index(&lab.out);
                eprintln!("{} pictures in {:.1} s", lab.jobs.len(), lab.started.elapsed().as_secs_f32());
                exit.write(AppExit::Success);
                return;
            }
            let job = &lab.jobs[lab.index];
            let (sheet, cells, by_shot) = &lab.sheets[job.sheet];
            let shot_index = Shot::ALL.iter().position(|s| *s == job.shot).unwrap_or(0);
            let page = &by_shot[shot_index][job.page];
            let view = page.view;

            // the season's light and leaves, and the ground's colour under this sheet
            let (strength, colour, stands) = crate::sun_at(lk, lab.season);
            for (mut light, mut tf) in &mut sun {
                light.illuminance = strength;
                light.color = colour;
                *tf = stands;
            }
            if let Some(mut m) = materials.get_mut(&kit.green) {
                m.base_color = crate::leaf_tint(lk, lab.season);
            }
            let ground = ground_colour(lk, sheet.land, lab.season);
            if let Some(mut m) = materials.get_mut(&kit.ground) {
                m.base_color = ground.color();
            }

            // the camera, at the real distance
            let orbit = Orbit { target: view.target, yaw: 0.0, pitch: shots::PLAY_PITCH.to_radians(), dist: view.dist };
            for (mut tf, mut fog, dof) in &mut cams {
                *tf = crate::orbit_transform(&orbit);
                fog.falloff = crate::haze_falloff(lk, view.dist);
                if let Some(mut d) = dof {
                    *d = crate::tilt_shift(lk, view.dist);
                }
            }

            // the ground, wide enough to fill the picture, and its ruling
            let reach = view.dist * 8.0;
            commands.spawn((Mesh3d(meshes.add(terrain::plane(view.target.x - reach, view.target.z - reach, view.target.x + reach, view.target.z + reach, 0.0))), MeshMaterial3d(kit.ground.clone()), OfThePicture));
            let (rule, rule_words) = ruling(view.px_per_unit());
            let ruled = if view.k > 1.0 { ruled_squares(&view, rule, shade(*ground, 0.94)) } else { ruled_lines(&view, rule, shade(*ground, 0.88)) };
            commands.spawn((Mesh3d(meshes.add(meshkit::into_mesh(ruled))), MeshMaterial3d(kit.flat.clone()), bevy::light::NotShadowCaster, OfThePicture));

            // every cell's model where the layout stands it
            let mut all = Model::default();
            for pl in &page.placed {
                all.merge_at(&cells[pl.row][pl.col].model, pl.origin);
            }
            let Model { solid, smoke, green, flat } = all;
            for (b, mat, shadow) in [(solid, &kit.solid, true), (green, &kit.green, true), (smoke, &kit.smoke, false), (flat, &kit.flat, false)] {
                if b.is_empty() {
                    continue;
                }
                let mut e = commands.spawn((Mesh3d(meshes.add(meshkit::into_mesh(b))), MeshMaterial3d(mat.clone()), OfThePicture));
                if !shadow {
                    e.insert(bevy::light::NotShadowCaster);
                }
            }

            // the finished picture: the camera's, its middle enlarged, with the labels and the heading over it
            let (w, h) = (lab.size.0 as f32, lab.size.1 as f32);
            let crop = Rect::from_center_size(Vec2::new(w, h) * 0.5, Vec2::new(w, h) / view.k);
            let f = &font.0;
            let heading = heading(sheet, job, &view, lab.season, lab.tilt, lab.turn, rule_words);
            commands
                .spawn((Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Px(w), height: Val::Px(h), ..default() }, UiTargetCamera(kit.ui_camera), OfThePicture))
                .with_children(|root| {
                    root.spawn((
                        Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Px(w), height: Val::Px(h), ..default() },
                        ImageNode { image: kit.scene.clone(), rect: Some(crop), image_mode: NodeImageMode::Stretch, ..default() },
                    ));
                    for pl in &page.placed {
                        let at = view.project(pl.label_at);
                        // the cell's place in the picture, which the label sits in the middle of
                        let half = (view.project(pl.label_at + Vec3::X * pl.slot * 0.5).x - at.x).abs();
                        root.spawn(Node { position_type: PositionType::Absolute, left: Val::Px(at.x - half), top: Val::Px(at.y), width: Val::Px(half * 2.0), justify_content: JustifyContent::Center, ..default() }).with_children(|slot| {
                            slot.spawn((
                                Node { padding: UiRect::axes(Val::Px(7.0), Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(8.0)), ..default() },
                                BackgroundColor(Color::srgba(0.99, 0.97, 0.92, 0.88)),
                            ))
                            .with_children(|pill| {
                                pill.spawn((text(f, &cells[pl.row][pl.col].label, layout::LABEL_TEXT, INK), TextLayout { justify: Justify::Center, ..default() }));
                            });
                        });
                    }
                    root.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(14.0),
                            top: Val::Px(12.0),
                            max_width: Val::Px(w - 28.0),
                            padding: UiRect::new(Val::Px(14.0), Val::Px(16.0), Val::Px(6.0), Val::Px(8.0)),
                            flex_direction: FlexDirection::Column,
                            border_radius: BorderRadius::all(Val::Px(14.0)),
                            ..default()
                        },
                        BackgroundColor(PAPER),
                        BoxShadow::new(Color::srgba(0.25, 0.18, 0.10, 0.18), Val::Px(0.0), Val::Px(3.0), Val::Px(0.0), Val::Px(10.0)),
                    ))
                    .with_children(|card| {
                        card.spawn(text(f, &heading.0, 20.0, INK));
                        card.spawn(text(f, sheet.about, 13.0, MUTED));
                        card.spawn(text(f, &heading.1, 13.0, INK));
                    });
                });

            let _ = std::fs::remove_file(&job.path);
            lab.phase = Phase::Settle(10);
        }
        Phase::Settle(left) => {
            // a pipeline still being put together means something is not drawn yet: hold
            // the picture until it is there and has been drawn for a few frames, but not for ever
            if waiting.0.load(std::sync::atomic::Ordering::Relaxed) > 0 && held.get_or_insert_with(std::time::Instant::now).elapsed().as_secs_f32() < 120.0 {
                lab.phase = Phase::Settle(left.max(6));
                return;
            }
            if let Some(since) = held.take() {
                if since.elapsed().as_secs_f32() > 1.0 {
                    eprintln!("waited {:.1} s for the renderer's shaders", since.elapsed().as_secs_f32());
                }
            }
            if left > 0 {
                lab.phase = Phase::Settle(left - 1);
                return;
            }
            let path = lab.jobs[lab.index].path.clone();
            commands.spawn(Screenshot::image(kit.picture.clone())).observe(save_to_disk(path));
            lab.phase = Phase::Written(0);
        }
        Phase::Written(frames) => {
            // the file is written a few frames after it is asked for
            let path = &lab.jobs[lab.index].path;
            if (frames >= 3 && path.exists()) || frames > 600 {
                if !path.exists() {
                    eprintln!("{} was not written", path.display());
                }
                lab.index += 1;
                lab.phase = Phase::Arrange;
            } else {
                lab.phase = Phase::Written(frames + 1);
            }
        }
    }
}

/// The two lines of a picture's heading: the sheet's name with which picture
/// of it this is, and how it was taken.
fn heading(sheet: &Sheet, job: &Job, view: &layout::View, season: f32, tilt: bool, turn: f32, rule: &str) -> (String, String) {
    let mut title = format!("{} \u{00b7} {}", sheet.name, match job.shot {
        Shot::Close => "close up",
        Shot::Play => "at play distance",
        Shot::Far => "from far off",
    });
    if job.of > 1 {
        let _ = write!(title, " \u{00b7} page {} of {}", job.page + 1, job.of);
    }
    let pixels = if view.k > 1.0 { format!("the middle of the game's picture, each pixel drawn {} times its size", view.k as u32) } else { "pixel for pixel as the game draws it".to_string() };
    let ruled = if view.k > 1.0 { "chequered in squares of" } else { "ruled every" };
    // a thing too big for the closest picture is seen from as near as shows the whole of it
    let from = if view.dist > job.shot.dist() { "further off than the game's nearest, to show the whole of it" } else { job.shot.words() };
    let dist = (view.dist * 100.0).round() / 100.0;
    let mut how = format!("The camera {dist} units from the ground, {from}: {pixels}. The ground is {ruled} {rule}. {}{}.", season_word(season)[..1].to_uppercase(), &season_word(season)[1..]);
    if turn != 0.0 {
        let _ = write!(how, " Each thing is turned {} degrees to its {}.", turn.abs(), if turn > 0.0 { "left" } else { "right" });
    }
    if tilt {
        how.push_str(" Tilt-shift on.");
    }
    (title, how)
}

/// Faint lines on the ground every `step`, a little over a true pixel wide,
/// across what the picture shows.
fn ruled_lines(view: &layout::View, step: f32, colour: [f32; 4]) -> Builder {
    let mut b = Builder::default();
    // what the finished picture shows of the ground, with room to spare for the tilt
    let s = view.px_per_unit();
    let (half_x, half_z) = (view.size.0 / s * 0.75, view.size.1 / s * 1.6);
    let width = 1.3 * view.k / s;
    let y = 0.0012;
    let (x0, x1) = (((view.target.x - half_x) / step).floor() as i32, ((view.target.x + half_x) / step).ceil() as i32);
    let (z0, z1) = (((view.target.z - half_z) / step).floor() as i32, ((view.target.z + half_z) / step).ceil() as i32);
    for i in x0..=x1 {
        let x = i as f32 * step;
        b.ribbon(&[Vec3::new(x, y, z0 as f32 * step), Vec3::new(x, y, z1 as f32 * step)], width, colour);
    }
    for j in z0..=z1 {
        let z = j as f32 * step;
        b.ribbon(&[Vec3::new(x0 as f32 * step, y, z), Vec3::new(x1 as f32 * step, y, z)], width, colour);
    }
    b
}

/// The ground chequered faintly in squares of `step`, for an enlarged
/// picture: a line a pixel wide breaks up when its pixels are drawn as
/// squares, where the edge of a faint square only steps.
fn ruled_squares(view: &layout::View, step: f32, colour: [f32; 4]) -> Builder {
    let mut b = Builder::default();
    let s = view.px_per_unit();
    let (half_x, half_z) = (view.size.0 / s * 0.75, view.size.1 / s * 1.6);
    let y = 0.0012;
    let (x0, x1) = (((view.target.x - half_x) / step).floor() as i32, ((view.target.x + half_x) / step).ceil() as i32);
    let (z0, z1) = (((view.target.z - half_z) / step).floor() as i32, ((view.target.z + half_z) / step).ceil() as i32);
    for j in z0..z1 {
        for i in x0..x1 {
            if (i + j).rem_euclid(2) == 0 {
                let (x, z) = (i as f32 * step, j as f32 * step);
                b.quad(Vec3::new(x, y, z), Vec3::new(x, y, z + step), Vec3::new(x + step, y, z + step), Vec3::new(x + step, y, z), colour);
            }
        }
    }
    b
}

// ------------------------------------------------------------------ the index

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

const STYLE: &str = r#"
:root { --paper: #f6f0e2; --card: #fdf9ee; --ink: #332b24; --muted: #786b5c; --rule: #e3d9c4; }
* { box-sizing: border-box; }
body { margin: 0; padding: 28px 32px 60px; background: var(--paper); color: var(--ink);
  font: 15px/1.45 "Fredoka", ui-rounded, "Segoe UI", system-ui, sans-serif; }
h1 { font-size: 26px; font-weight: 500; margin: 0 0 2px; }
p.about { margin: 0; color: var(--muted); max-width: 70em; }
nav { margin: 14px 0 0; display: flex; flex-wrap: wrap; gap: 6px 8px; }
nav a { padding: 2px 10px; background: var(--card); border-radius: 10px; text-decoration: none; color: inherit;
  box-shadow: 0 1px 4px rgba(64, 46, 26, 0.12); }
section { margin-top: 26px; padding: 14px 14px 12px; background: var(--card); border-radius: 16px;
  box-shadow: 0 3px 10px rgba(64, 46, 26, 0.14); }
h2 { font-size: 19px; font-weight: 500; margin: 0 4px; }
section p { margin: 0 4px 10px; color: var(--muted); }
.shots { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
figure { margin: 0; }
figcaption { color: var(--muted); font-size: 13px; padding: 4px 2px 0; text-align: center; }
img { display: block; width: 100%; height: auto; border-radius: 9px; }
@media (max-width: 1100px) { .shots { grid-template-columns: 1fr; } }
"#;

/// Write `index.html`: every picture of every sheet that is in the directory,
/// whichever run made it, a sheet to a row.
fn write_index(dir: &Path) {
    let mut h = String::new();
    let _ = write!(h, "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>Isolated State: the lab</title>\n<style>{STYLE}</style>\n</head>\n<body>\n");
    let _ = writeln!(h, "<h1>The lab: contact sheets of the generators</h1>");
    let _ = writeln!(h, "<p class=\"about\">Each sheet is a grid of generator calls on plain ground, in the game's light, through the game's camera at three distances: close (as near as the game goes), play (as a new game opens) and far (about half way out to the whole island). The play and far pictures show the middle of the true picture enlarged, each pixel a square. Click a picture to see it whole.</p>");
    let present: Vec<(&Sheet, Vec<(Shot, Vec<String>)>)> = SHEETS.iter().map(|s| (s, pictures_of(dir, s.name))).filter(|(_, p)| p.iter().any(|(_, f)| !f.is_empty())).collect();
    let _ = writeln!(h, "<nav>");
    for (s, _) in &present {
        let _ = writeln!(h, "<a href=\"#{n}\">{n}</a>", n = esc(s.name));
    }
    let _ = writeln!(h, "</nav>");
    for (s, by_shot) in &present {
        let _ = writeln!(h, "<section id=\"{n}\">\n<h2>{n}</h2>\n<p>{}</p>\n<div class=\"shots\">", esc(s.about), n = esc(s.name));
        // a row for each page: close, play, far; the later pages have only a close picture
        let pages = by_shot.iter().map(|(_, f)| f.len()).max().unwrap_or(0);
        for p in 0..pages {
            for (shot, files) in by_shot {
                match files.get(p) {
                    Some(file) => {
                        let page = if files.len() > 1 { format!(", page {} of {}", p + 1, files.len()) } else { String::new() };
                        let _ = writeln!(h, "<figure><a href=\"{f}\"><img src=\"{f}\" loading=\"lazy\" alt=\"{} {}\"></a><figcaption>{}{page}</figcaption></figure>", esc(s.name), shot.name(), shot.name(), f = esc(file));
                    }
                    None => {
                        let _ = writeln!(h, "<figure></figure>");
                    }
                }
            }
        }
        let _ = writeln!(h, "</div>\n</section>");
    }
    let _ = writeln!(h, "</body>\n</html>");
    let path = dir.join("index.html");
    match std::fs::write(&path, h) {
        Ok(()) => eprintln!("wrote {}", path.display()),
        Err(e) => eprintln!("cannot write {}: {e}", path.display()),
    }
}
