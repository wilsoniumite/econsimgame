//! The square under the pointer, outlined on the ground, and the square that
//! was clicked, outlined more boldly, so the 2 km grid the economy works in
//! is always clear when it matters and invisible when it does not.

use bevy::prelude::*;

use crate::look::{TheLook, ToColor};
use crate::meshkit::Builder;
use crate::ui::UiState;
use crate::{ground_hit, world, Data, MainCam, Pick, Picked};

#[derive(Resource)]
struct Outline {
    mesh: Handle<Mesh>,
    hover: Option<usize>,
    drawn: Option<(Option<usize>, Option<usize>, [u8; 4])>,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup).add_systems(Update, (hover, draw).chain());
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mesh = meshes.add(crate::placeholder_mesh());
    // unlit, so the line reads the same in sun and shade
    let mat = materials.add(StandardMaterial { base_color: Color::WHITE, unlit: true, alpha_mode: AlphaMode::Blend, ..default() });
    commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat), bevy::light::NotShadowCaster));
    commands.insert_resource(Outline { mesh, hover: None, drawn: None });
}

fn hover(
    mut o: ResMut<Outline>,
    ui: Res<UiState>,
    data: Res<Data>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cams: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let mut cell = None;
    // nothing while dragging the view or over a panel
    let dragging = buttons.pressed(MouseButton::Left) || buttons.pressed(MouseButton::Right);
    if let (Ok(window), Ok((cam, cam_t))) = (windows.single(), cams.single()) {
        if let Some(cursor) = window.cursor_position().filter(|_| !ui.hover && !dragging) {
            if let Ok(ray) = cam.viewport_to_world(cam_t, cursor) {
                if let Some(hit) = ground_hit(&data.g, ray) {
                    let r = &data.r;
                    if hit.x >= 0.0 && hit.z >= 0.0 && (hit.x as usize) < r.w && (hit.z as usize) < r.h {
                        let c = hit.z as usize * r.w + hit.x as usize;
                        if r.terrain[c] > 0 {
                            cell = Some(c);
                        }
                    }
                }
            }
        }
    }
    if o.hover != cell {
        o.hover = cell;
    }
}

/// A line round a square, draped on the ground and inset a little.
fn square(b: &mut Builder, data: &Data, c: usize, inset: f32, width: f32, col: [f32; 4]) {
    let r = &data.r;
    let (x0, z0) = ((c % r.w) as f32 + inset, (c / r.w) as f32 + inset);
    let (x1, z1) = (x0 + 1.0 - 2.0 * inset, z0 + 1.0 - 2.0 * inset);
    // start mid-edge so the loop closes on a straight
    let corners = [((x0 + x1) * 0.5, z0), (x1, z0), (x1, z1), (x0, z1), (x0, z0), ((x0 + x1) * 0.5, z0)];
    let mut pts = Vec::new();
    for w in corners.windows(2) {
        let (a, e) = (w[0], w[1]);
        let n = ((((e.0 - a.0).powi(2) + (e.1 - a.1).powi(2)).sqrt() / 0.05).ceil() as usize).max(1);
        for i in 0..n {
            let t = i as f32 / n as f32;
            let (x, z) = (a.0 + (e.0 - a.0) * t, a.1 + (e.1 - a.1) * t);
            pts.push(Vec3::new(x, world::height_at(&data.g, x, z).max(0.0) + 0.012, z));
        }
    }
    if let Some(&p) = pts.first() {
        pts.push(p);
    }
    b.ribbon(&pts, width, col);
}

fn draw(mut o: ResMut<Outline>, picked: Res<Picked>, data: Res<Data>, look: Res<TheLook>, fam: Option<Res<crate::family::FamilyView>>, mut meshes: ResMut<Assets<Mesh>>) {
    let sel = match picked.what {
        Some(Pick::Cell(c)) => Some(c),
        _ => None,
    };
    let accent = fam.as_ref().and_then(|f| f.player()).map(|p| p.colour).unwrap_or(look.marks.outline_plain.color());
    let key = (o.hover, sel, accent.to_srgba().to_u8_array());
    if o.drawn == Some(key) {
        return;
    }
    o.drawn = Some(key);
    let mut b = Builder::default();
    if let Some(c) = o.hover.filter(|&h| Some(h) != sel) {
        let mut col = *look.marks.hover;
        col[3] = look.marks.hover_alpha;
        square(&mut b, &data, c, 0.02, 0.022, col);
    }
    if let Some(c) = sel {
        let s = accent.to_linear();
        square(&mut b, &data, c, 0.02, 0.04, [s.red, s.green, s.blue, look.marks.outline_alpha]);
    }
    if let Some(mut m) = meshes.get_mut(&o.mesh) {
        *m = if b.is_empty() { crate::placeholder_mesh() } else { crate::meshkit::into_mesh(b) };
    }
}
