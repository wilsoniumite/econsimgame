//! The thing under the pointer, lit with a thin line on the ground, and the
//! thing that was clicked, lit more boldly in the accent (pick.rs says what a
//! click picks): a works by the outline of its yard, a town by the outline of
//! its ground, a pit by a ring round it, and a spot in open country by a small
//! ring, since the country of a market town has no edge yet that is not the
//! simulation's 2 km cells. With the Ground tool in hand the pointer is the
//! brush instead, a ring as wide as it paints. No square is drawn.
//!
//! What lies under the pointer is asked once a frame while it moves and the
//! answer kept while it rests; the test (the ray to the ground and `choose`)
//! is timed, and a scripted run's `print` says what it cost. A town's outline
//! is worked out off the main thread, the first time the town is lit in a
//! year, and drawn when it comes: the view never waits for it.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use bevy::prelude::*;
use bevy::tasks::{futures::check_ready, AsyncComputeTaskPool, Task};

use crate::look::{TheLook, ToColor};
use crate::meshkit::Builder;
use crate::pick::{self, Pits};
use crate::ui::UiState;
use crate::{ground_hit, world, Data, MainCam, Pick, Picked, Timeline};

/// A town's outline: its closed lines on the map.
type Lines = Arc<Vec<Vec<(f32, f32)>>>;
/// Which outline: the town, the year, and the run (a live game's run is a new one each year).
type LinesKey = (usize, usize, usize);

#[derive(Resource)]
struct Outline {
    mesh: Handle<Mesh>,
    /// what lies under the pointer, and where it rests
    hover: Option<(Pick, (f32, f32))>,
    /// the ray last asked about, and the year, so a pointer at rest is not asked about again
    asked: Option<(Vec3, Vec3, usize)>,
    drawn: Option<(Option<(Pick, (f32, f32))>, Option<Pick>, [u8; 4], Option<u8>, u64)>,
    /// the towns' outlines worked out, the newest few
    towns: HashMap<LinesKey, Lines>,
    making: Option<(LinesKey, Task<Vec<Vec<(f32, f32)>>>)>,
    /// raised by one when an outline comes
    came: u64,
}

/// For a scripted run (`hover:X,Z`): the pointer rests on this spot of the
/// map, as a hand's would, until `hover:none`.
#[derive(Resource, Default)]
pub struct ScriptPointer(pub Option<(f32, f32)>);

/// What the test under the pointer cost: the frames it was asked in, and
/// their time in microseconds, all told and the longest.
#[derive(Resource, Default)]
pub struct HoverCost {
    pub frames: u32,
    pub total_us: f64,
    pub max_us: f64,
}

impl HoverCost {
    /// In a line, for a script's `print`.
    pub fn said(&self) -> String {
        if self.frames == 0 {
            return "the pointer was not asked about".to_string();
        }
        format!("the test under the pointer took {:.0} µs a frame on the main thread over the {} frames it was asked in, {:.0} µs at most", self.total_us / self.frames as f64, self.frames, self.max_us)
    }
}

/// What lies under the pointer, in a line, for a script's `print`.
#[derive(Resource, Default)]
pub struct Hovered(pub Option<Pick>);

pub fn plugin(app: &mut App) {
    app.init_resource::<ScriptPointer>().init_resource::<HoverCost>().init_resource::<Hovered>().add_systems(Startup, setup).add_systems(Update, (hover, outlines, draw).chain().after(crate::pick));
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mesh = meshes.add(crate::placeholder_mesh());
    // unlit, so the line reads the same in sun and shade
    let mat = materials.add(StandardMaterial { base_color: Color::WHITE, unlit: true, alpha_mode: AlphaMode::Blend, ..default() });
    commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat), bevy::light::NotShadowCaster));
    commands.insert_resource(Outline { mesh, hover: None, asked: None, drawn: None, towns: HashMap::new(), making: None, came: 0 });
}

#[allow(clippy::too_many_arguments)]
fn hover(
    mut o: ResMut<Outline>,
    ui: Res<UiState>,
    data: Res<Data>,
    tl: Res<Timeline>,
    pits: Res<Pits>,
    script: Res<ScriptPointer>,
    mut cost: ResMut<HoverCost>,
    mut hovered: ResMut<Hovered>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cams: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    let Ok((cam, cam_t)) = cams.single() else { return };
    // the ray from the camera: through the pointer, or toward the script's spot
    let ray = match script.0 {
        Some((x, z)) => {
            let at = Vec3::new(x, world::height_at(&data.g, x, z).max(0.0), z);
            let from = cam_t.translation();
            Dir3::new(at - from).ok().map(|d| Ray3d::new(from, d))
        }
        None => {
            // nothing while dragging the view or over a panel
            let dragging = buttons.pressed(MouseButton::Left) || buttons.pressed(MouseButton::Right);
            windows.single().ok().and_then(|w| w.cursor_position()).filter(|_| !ui.hover && !dragging).and_then(|c| cam.viewport_to_world(cam_t, c).ok())
        }
    };
    let Some(ray) = ray else {
        (o.hover, o.asked, hovered.0) = (None, None, None);
        return;
    };
    // a pointer at rest, with the camera still, over the same year, is not asked about again
    if o.asked.is_some_and(|(from, to, year)| year == k && from.distance(ray.origin) < 1e-4 && to.distance(*ray.direction) < 1e-5) {
        return;
    }
    o.asked = Some((ray.origin, *ray.direction, k));
    let t0 = Instant::now();
    let found = ground_hit(&data.g, ray).and_then(|hit| pick::choose(&data.r, &data.g, &data.slots, &data.cen, &pits.spots, hit.x, hit.z, k).map(|p| (p, (hit.x, hit.z))));
    let us = t0.elapsed().as_secs_f64() * 1e6;
    cost.frames += 1;
    cost.total_us += us;
    cost.max_us = cost.max_us.max(us);
    hovered.0 = found.map(|f| f.0);
    o.hover = found;
}

/// A town lit or picked has its outline worked out off the main thread, once a year.
fn outlines(mut o: ResMut<Outline>, data: Res<Data>, tl: Res<Timeline>, picked: Res<Picked>) {
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    let run = Arc::as_ptr(&data.r) as usize;
    if let Some((key, task)) = o.making.as_mut() {
        if let Some(lines) = check_ready(task) {
            let key = *key;
            o.making = None;
            if o.towns.len() >= 8 {
                o.towns.clear();
            }
            o.towns.insert(key, Arc::new(lines));
            o.came += 1;
        }
        return;
    }
    let wanted = [o.hover.map(|h| h.0), picked.what].into_iter().flatten().find_map(|p| match p {
        Pick::Town(t) if !o.towns.contains_key(&(t, k, run)) => Some(t),
        _ => None,
    });
    if let Some(t) = wanted {
        let (r, g) = (data.r.clone(), data.g.clone());
        let task = AsyncComputeTaskPool::get().spawn(async move { pick::town_outline(&r, &g, k, t) });
        o.making = Some(((t, k, run), task));
    }
}

/// A closed line on the map, draped on the ground a hair above it.
fn closed(b: &mut Builder, data: &Data, pts: &[(f32, f32)], width: f32, col: [f32; 4]) {
    if pts.len() < 3 {
        return;
    }
    let mut out = Vec::new();
    for i in 0..pts.len() {
        let (a, e) = (pts[i], pts[(i + 1) % pts.len()]);
        let n = ((a.0 - e.0).hypot(a.1 - e.1) / 0.04).ceil().max(1.0) as usize;
        for s in 0..n {
            let t = s as f32 / n as f32;
            let (x, z) = (a.0 + (e.0 - a.0) * t, a.1 + (e.1 - a.1) * t);
            out.push(Vec3::new(x, world::height_at(&data.g, x, z).max(0.0) + 0.012, z));
        }
    }
    if let Some(&p) = out.first() {
        out.push(p);
    }
    b.ribbon(&out, width, col);
}

/// A ring of radius `rad` round a spot.
fn ring(b: &mut Builder, data: &Data, (x, z): (f32, f32), rad: f32, width: f32, col: [f32; 4]) {
    let n = ((rad * std::f32::consts::TAU / 0.03).ceil() as usize).clamp(16, 400);
    let pts: Vec<(f32, f32)> = (0..n).map(|i| { let a = i as f32 / n as f32 * std::f32::consts::TAU; (x + rad * a.cos(), z + rad * a.sin()) }).collect();
    closed(b, data, &pts, width, col);
}

/// The ring round a spot picked in open country, and round one under the pointer.
const SPOT: f32 = 0.09;

/// The line round a thing: its yard, its ground or a ring.
fn light(b: &mut Builder, o: &Outline, data: &Data, pits: &Pits, k: usize, what: Pick, width: f32, col: [f32; 4]) {
    let run = Arc::as_ptr(&data.r) as usize;
    match what {
        Pick::Firm(f) => closed(b, data, &pick::yard_outline(&data.r, &data.slots, f, k), width, col),
        Pick::Town(t) => {
            if let Some(lines) = o.towns.get(&(t, k, run)) {
                for l in lines.iter() {
                    closed(b, data, l, width, col);
                }
            }
        }
        Pick::Pit { cell, class } => {
            if let Some(p) = pits.spots.iter().find(|p| p.cell == cell && p.class == class) {
                ring(b, data, (p.x, p.z), pick::PIT_REACH, width, col);
            }
        }
        Pick::Place { x, z, .. } => ring(b, data, (x, z), SPOT, width, col),
    }
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut o: ResMut<Outline>,
    picked: Res<Picked>,
    data: Res<Data>,
    tl: Res<Timeline>,
    pits: Res<Pits>,
    look: Res<TheLook>,
    fam: Option<Res<crate::family::FamilyView>>,
    sandbox: Option<Res<crate::sandbox::Sandbox>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let k = (tl.t.floor() as usize).min(data.r.ny - 1);
    let sel = picked.what;
    let accent = fam.as_ref().and_then(|f| f.player()).map(|p| p.colour).unwrap_or(look.marks.outline_plain.color());
    // GROUND-BRUSH: the brush in hand, as wide as it paints (`edits::stroke` reaches `size`
    // cells from each it passes over: three, five or seven cells across)
    let brush = sandbox.as_ref().and_then(|s| s.brush()).map(|(_, size, _)| crate::ground::SIZES[size.min(crate::ground::SIZES.len() - 1)]);
    let key = (o.hover, sel, accent.to_srgba().to_u8_array(), brush, o.came + 1_000_000 * k as u64);
    if o.drawn == Some(key) {
        return;
    }
    o.drawn = Some(key);
    let mut b = Builder::default();
    let mut col = *look.marks.hover;
    col[3] = look.marks.hover_alpha;
    match (o.hover, brush) {
        (Some((_, at)), Some(size)) => ring(&mut b, &data, at, size as f32 + 0.5, 0.03, col),
        (Some((what, _)), None) if Some(what) != sel => light(&mut b, &o, &data, &pits, k, what, 0.022, col),
        _ => {}
    }
    if let Some(what) = sel {
        let s = accent.to_linear();
        light(&mut b, &o, &data, &pits, k, what, 0.04, [s.red, s.green, s.blue, look.marks.outline_alpha]);
    }
    if let Some(mut m) = meshes.get_mut(&o.mesh) {
        *m = if b.is_empty() { crate::placeholder_mesh() } else { crate::meshkit::into_mesh(b) };
    }
}
