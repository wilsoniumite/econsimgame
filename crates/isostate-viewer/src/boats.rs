//! Sailing boats off every harbour town: a few small craft tacking round a
//! loop of open water, more for a bigger town. Purely for life on the map.

use bevy::prelude::*;

use crate::meshkit::{hex, Builder};
use crate::{world, Data, Timeline};

#[derive(Component)]
struct Boat {
    centre: Vec2,
    radius: Vec2,
    angle: f32,
    speed: f32,
    seed: f32,
}

#[derive(Resource)]
struct BoatAssets {
    mesh: Handle<Mesh>,
    mat: Handle<StandardMaterial>,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup).add_systems(Update, (place, sail).chain());
}

fn boat_mesh() -> Builder {
    let mut b = Builder::default();
    let hull = hex(0x6b4a35);
    let deck = hex(0xb08a5e);
    // a hull tapering to a bow along +x
    let (l, w, h) = (0.075, 0.026, 0.012);
    let p = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
    let top = [p(-l * 0.5, h, -w * 0.5), p(l * 0.3, h, -w * 0.5), p(l * 0.5, h, 0.0), p(l * 0.3, h, w * 0.5), p(-l * 0.5, h, w * 0.5)];
    let bot = [p(-l * 0.42, -0.004, -w * 0.3), p(l * 0.25, -0.004, -w * 0.3), p(l * 0.45, -0.002, 0.0), p(l * 0.25, -0.004, w * 0.3), p(-l * 0.42, -0.004, w * 0.3)];
    for i in 0..5 {
        let j = (i + 1) % 5;
        b.quad(bot[i], bot[j], top[j], top[i], hull);
    }
    b.tri(top[0], top[2], top[1], deck);
    b.tri(top[0], top[3], top[2], deck);
    b.tri(top[0], top[4], top[3], deck);
    // mast and two sails, cream and a little weathered
    let mast = p(0.004, h, 0.0);
    b.beam(mast, mast + Vec3::Y * 0.085, 0.003, hex(0x5a4030));
    let sail = hex(0xf3ead6);
    let jib = hex(0xe8dcc0);
    let (s0, s1, s2) = (mast + Vec3::new(-0.002, 0.008, 0.0), mast + Vec3::new(-0.002, 0.082, 0.0), mast + Vec3::new(-0.034, 0.01, 0.006));
    b.tri(s0, s1, s2, sail);
    b.tri(s0, s2, s1, sail);
    let (j0, j1, j2) = (mast + Vec3::new(0.004, 0.075, 0.0), p(l * 0.48, h + 0.002, 0.0), mast + Vec3::new(0.006, 0.006, 0.004));
    b.tri(j0, j1, j2, jib);
    b.tri(j0, j2, j1, jib);
    b
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mesh = meshes.add(boat_mesh().into_mesh());
    let mat = crate::vertex_coloured(&mut materials, 0.8);
    commands.insert_resource(BoatAssets { mesh, mat });
}

/// Put boats off the harbours that are towns this year.
fn place(mut commands: Commands, tl: Res<Timeline>, data: Res<Data>, assets: Option<Res<BoatAssets>>, old: Query<Entity, With<Boat>>, mut last: Local<Option<usize>>) {
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
    let g = &data.g;
    let sea_at = |x: f32, z: f32| world::height_at(g, x, z) < -0.03;
    for t in 0..r.n_towns.min(r.header.towns.len()) {
        if r.town_active[k * r.n_towns + t] == 0 {
            continue;
        }
        let node = r.header.towns[t].node;
        let (cx, cy) = ((node % r.w) as isize, (node / r.w) as isize);
        // a harbour is a town on the shore
        let shore = [(1isize, 0isize), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)].iter().any(|&(dx, dy)| {
            let (nx, ny) = (cx + dx, cy + dy);
            nx >= 0 && ny >= 0 && (nx as usize) < r.w && (ny as usize) < r.h && r.terrain[ny as usize * r.w + nx as usize] == 0
        });
        if !shore {
            continue;
        }
        let (x, z) = data.cen[node];
        // sail toward the most open water
        let dir = (0..16)
            .map(|i| {
                let a = i as f32 / 16.0 * std::f32::consts::TAU;
                let d = Vec2::new(a.cos(), a.sin());
                let open = (2..9).filter(|&s| sea_at(x + d.x * s as f32 * 0.5, z + d.y * s as f32 * 0.5)).count();
                (open, d)
            })
            .max_by_key(|(open, _)| *open)
            .map(|(_, d)| d)
            .unwrap_or(Vec2::X);
        let urban = r.town_urban[k * r.n_towns + t];
        let count = (1.0 + (urban / 6000.0).sqrt()).floor().clamp(1.0, 4.0) as usize;
        for i in 0..count {
            let seed = world::hash2(t as i32, i as i32, 31);
            // a loop of open water out beyond the harbour mouth
            let mut placed = None;
            for out in [0.9f32, 1.3, 1.8, 2.5] {
                let centre = Vec2::new(x, z) + dir * (out + 0.4 * seed);
                let radius = Vec2::new(0.3 + 0.35 * seed, 0.2 + 0.25 * world::hash2(t as i32, i as i32, 32));
                let clear = (0..16).all(|j| {
                    let a = j as f32 / 16.0 * std::f32::consts::TAU;
                    sea_at(centre.x + radius.x * a.cos(), centre.y + radius.y * a.sin())
                });
                if clear {
                    placed = Some((centre, radius));
                    break;
                }
            }
            let Some((centre, radius)) = placed else { continue };
            commands.spawn((
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(assets.mat.clone()),
                Transform::from_translation(Vec3::new(centre.x, 0.0, centre.y)),
                bevy::light::NotShadowCaster,
                Boat { centre, radius, angle: seed * std::f32::consts::TAU, speed: 0.10 + 0.06 * seed, seed },
            ));
        }
    }
}

fn sail(time: Res<Time>, mut q: Query<(&mut Boat, &mut Transform)>) {
    let dt = time.delta_secs().min(0.1);
    let now = time.elapsed_secs();
    for (mut b, mut tf) in &mut q {
        b.angle += dt * b.speed;
        let (s, c) = b.angle.sin_cos();
        let pos = Vec2::new(b.centre.x + b.radius.x * c, b.centre.y + b.radius.y * s);
        // heading along the loop
        let vel = Vec2::new(-b.radius.x * s, b.radius.y * c);
        let yaw = (-vel.y).atan2(vel.x);
        let bob = 0.004 * (now * 1.7 + b.seed * 9.0).sin();
        let roll = 0.05 * (now * 1.3 + b.seed * 5.0).sin() + 0.08;
        tf.translation = Vec3::new(pos.x, bob, pos.y);
        tf.rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(roll);
    }
}
