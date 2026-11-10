//! Carts on the main roads: a few covered carts plodding between the towns,
//! more where the roads are longest. Purely for life on the map.

use bevy::prelude::*;

use crate::trains::{self, TrainPath};
use crate::{world, Data, Timeline};

#[derive(Component)]
struct Cart {
    path: usize,
    s: f32,
    dir: f32,
    speed: f32,
}

#[derive(Resource)]
struct CartAssets {
    mesh: Handle<Mesh>,
    mat: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
struct Roads(Vec<TrainPath>);

pub fn plugin(app: &mut App) {
    app.init_resource::<Roads>().add_systems(Startup, setup).add_systems(Update, (place, drive).chain());
}

fn setup(mut commands: Commands, look: Res<crate::look::TheLook>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mesh = meshes.add(trains::cart_mesh(&look).into_mesh());
    let mat = crate::vertex_coloured(&mut materials, &look, look.surfaces.buildings);
    commands.insert_resource(CartAssets { mesh, mat });
}

fn place(mut commands: Commands, tl: Res<Timeline>, data: Res<Data>, assets: Option<Res<CartAssets>>, mut roads: ResMut<Roads>, old: Query<Entity, With<Cart>>, mut last: Local<Option<usize>>) {
    let Some(assets) = assets else { return };
    let r = &data.r;
    let k = (tl.t.floor() as usize).min(r.ny - 1);
    if *last == Some(k) {
        return;
    }
    *last = Some(k);
    for e in &old {
        commands.entity(e).despawn();
    }
    roads.0 = trains::road_routes(r, &data.g, &data.cen, k);
    for (pi, p) in roads.0.iter().enumerate() {
        // one or two carts on each carter's round
        let count = 1 + (world::hash2(pi as i32, 7, 43) < 0.35) as usize;
        for i in 0..count {
            let u = world::hash2(pi as i32, i as i32, 41);
            let s = p.len * (i as f32 + u) / count as f32;
            let dir = if u < 0.5 { 1.0 } else { -1.0 };
            let (pos, yaw) = p.at(s);
            commands.spawn((
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(assets.mat.clone()),
                Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw)),
                bevy::light::NotShadowCaster,
                Cart { path: pi, s, dir, speed: 0.10 + 0.06 * world::hash2(pi as i32, i as i32, 42) },
            ));
        }
    }
}

fn drive(time: Res<Time>, roads: Res<Roads>, mut q: Query<(&mut Cart, &mut Transform)>) {
    let dt = time.delta_secs().min(0.1);
    for (mut c, mut tf) in &mut q {
        let Some(p) = roads.0.get(c.path) else { continue };
        c.s += c.dir * c.speed * dt;
        if c.s > p.len {
            c.s = p.len;
            c.dir = -1.0;
        } else if c.s < 0.0 {
            c.s = 0.0;
            c.dir = 1.0;
        }
        let (pos, yaw) = p.at(c.s);
        // carts going opposite ways pass on opposite sides of the road
        let heading = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
        let side = Vec3::new(heading.z, 0.0, -heading.x) * (0.007 * c.dir);
        tf.translation = pos + side;
        tf.rotation = Quat::from_rotation_y(yaw + if c.dir < 0.0 { std::f32::consts::PI } else { 0.0 });
    }
}
