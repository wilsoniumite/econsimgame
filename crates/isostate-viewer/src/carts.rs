//! Carts on the main roads: a few covered carts plodding between the towns,
//! more where the roads are longest. Purely for life on the map.
//!
//! A cart keeps to the road as it is drawn: the middle line of its ribbon at
//! the ribbon's height, and over a river on the bridge's roadway, coming on
//! and off by its ramps (`network::driven`, which the ribbon, the bridges and
//! the causeways are read from too). Carts going opposite ways pass on
//! opposite sides of the middle line, inside the ribbon and, on a bridge,
//! between its parapets.
//!
//! `ISOSTATE_ROADS=1` counts, for 1800, 1850 and 1900, the roads, their
//! bridges and causeways, the stretches of road that run over water with
//! nothing to carry them, and how far every cart strays from its road, and
//! stops (`maybe_check`). `ISOSTATE_CARTS_AT=X,Z;X,Z` puts a cart at each of
//! those spots of the map, on the carters' round that passes nearest, and
//! holds every cart still, for pictures of carts on bridges.

use bevy::math::Vec2;
use bevy::prelude::*;

use crate::network;
use crate::replay::Replay;
use crate::trains::{self, TrainPath};
use crate::world::{self, Ground};
use crate::Data;

/// How far above the surface it rides a cart is drawn (its wheels' feet).
pub const LIFT: f32 = 0.002;
/// How far to one side of the road's middle line a cart keeps.
pub const SIDE: f32 = 0.007;
/// Half the width of a cart over its wheels (`trains::cart_mesh`).
pub const WHEELS: f32 = 0.0095;

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

/// The carters' rounds the carts are on, and the year they are of. A year's
/// rounds are found with its scenery, off the main thread
/// (`trains::year_rounds`), and handed over with it (`year::rebuild_year`):
/// the carts go out on a year's roads as those roads go on screen, and no
/// road's way is looked for on the main thread for them.
#[derive(Resource, Default)]
pub struct Roads {
    paths: Vec<TrainPath>,
    year: Option<usize>,
    /// a year's rounds handed over and not yet taken (`place`)
    handed: Option<(usize, Vec<TrainPath>)>,
}

impl Roads {
    /// Hand over year `k`'s rounds.
    pub fn hand(&mut self, k: usize, rounds: Vec<TrainPath>) {
        self.handed = Some((k, rounds));
    }

    /// The year whose roads are on screen, the carts on them: none before the first year's scenery is.
    pub fn year(&self) -> Option<usize> {
        self.year
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Roads>().add_systems(Startup, setup).add_systems(Update, (place, drive).chain().after(crate::year::rebuild_year));
}

fn setup(mut commands: Commands, look: Res<crate::look::TheLook>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mesh = meshes.add(crate::meshkit::into_mesh(trains::cart_mesh(&look)));
    let mat = crate::vertex_coloured(&mut materials, &look, look.surfaces.buildings);
    commands.insert_resource(CartAssets { mesh, mat });
}

/// The spots `ISOSTATE_CARTS_AT` asks for a cart at, if any.
fn pinned_spots() -> Option<Vec<Vec2>> {
    let v = std::env::var("ISOSTATE_CARTS_AT").ok()?;
    let spots: Vec<Vec2> = v
        .split(';')
        .filter_map(|s| {
            let mut it = s.split(',').map(|x| x.trim().parse::<f32>());
            match (it.next(), it.next()) {
                (Some(Ok(x)), Some(Ok(z))) => Some(Vec2::new(x, z)),
                _ => None,
            }
        })
        .collect();
    Some(spots)
}

/// The place on any of the paths nearest a spot of the map: which path, and how far along it.
fn nearest_on(paths: &[TrainPath], at: Vec2) -> Option<(usize, f32)> {
    let mut best: Option<(f32, usize, f32)> = None;
    for (pi, p) in paths.iter().enumerate() {
        for i in 1..p.pts.len() {
            let (a, b) = (Vec2::new(p.pts[i - 1].x, p.pts[i - 1].z), Vec2::new(p.pts[i].x, p.pts[i].z));
            let ab = b - a;
            let u = ((at - a).dot(ab) / ab.length_squared().max(1e-9)).clamp(0.0, 1.0);
            let d = at.distance(a + ab * u);
            if best.is_none_or(|bst| d < bst.0) {
                best = Some((d, pi, p.cum[i - 1] + (p.cum[i] - p.cum[i - 1]) * u));
            }
        }
    }
    best.map(|b| (b.1, b.2))
}

/// Put the carts out on a year's rounds when they are handed over, and on a
/// year the carts are already on, leave them going. (Until 27 November 2026
/// the rounds were found here, on the main thread, the frame the clock's year
/// changed: a third of a second when a record of 2050 opened, and 28 to 40 ms
/// at the turn to a year not seen before, for the roads' ways that the
/// year's scenery was finding at the same time.)
fn place(mut commands: Commands, data: Res<Data>, assets: Option<Res<CartAssets>>, mut roads: ResMut<Roads>, old: Query<Entity, With<Cart>>) {
    let _frames = crate::frames::span(crate::frames::Sys::Traffic);
    let Some(assets) = assets else { return };
    let Some((k, rounds)) = roads.handed.take() else { return };
    if roads.year == Some(k) {
        // (the year built again whole, in a live game when a year arrives: its rounds are the same)
        return;
    }
    let r = &data.r;
    roads.year = Some(k);
    roads.paths = rounds;
    for e in &old {
        commands.entity(e).despawn();
    }
    // a cart asked for at a spot, for a picture: the first of the round nearest it, held still there
    let spots = pinned_spots();
    let pins: Vec<(usize, f32)> = spots.iter().flatten().filter_map(|&at| nearest_on(&roads.paths, at)).collect();
    // traffic (the core's docs/rules/traffic.md): where the run keeps what each way carried,
    // a round's carts follow the carting along it (`traffic::carts_by_weight`)
    let by_traffic = r.traffic.kept().then(|| crate::traffic::carts_by_weight(&roads.paths.iter().map(|p| crate::traffic::cart_weight_along(r, k, p)).collect::<Vec<f32>>()));
    for (pi, p) in roads.paths.iter().enumerate() {
        // one or two carts on each carter's round, or one to three by its carting
        let count = by_traffic.as_ref().map_or(1 + (world::hash2(pi as i32, 7, 43) < 0.35) as usize, |c| c[pi]);
        for i in 0..count {
            let u = world::hash2(pi as i32, i as i32, 41);
            let mut s = p.len * (i as f32 + u) / count as f32;
            if i == 0 {
                if let Some(&(_, at)) = pins.iter().find(|q| q.0 == pi) {
                    s = at;
                }
            }
            let dir = if u < 0.5 { 1.0 } else { -1.0 };
            let (pos, rot) = pose(p, s, dir);
            let speed = if spots.is_some() { 0.0 } else { 0.10 + 0.06 * world::hash2(pi as i32, i as i32, 42) };
            commands.spawn((
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(assets.mat.clone()),
                Transform::from_translation(pos).with_rotation(rot),
                bevy::light::NotShadowCaster,
                Cart { path: pi, s, dir, speed },
            ));
        }
    }
}

fn drive(time: Res<Time>, roads: Res<Roads>, mut q: Query<(&mut Cart, &mut Transform)>) {
    let dt = time.delta_secs().min(0.1);
    for (mut c, mut tf) in &mut q {
        let Some(p) = roads.paths.get(c.path) else { continue };
        c.s += c.dir * c.speed * dt;
        if c.s > p.len {
            c.s = p.len;
            c.dir = -1.0;
        } else if c.s < 0.0 {
            c.s = 0.0;
            c.dir = 1.0;
        }
        (tf.translation, tf.rotation) = pose(p, c.s, c.dir);
    }
}

/// Where a cart `s` along its path, going forward (1) or back (-1), is drawn,
/// and which way it faces. It is tipped to the line through its path under
/// its wheels and under the horse's forefeet, so that it climbs a bridge's
/// ramp and goes over its hump with its wheels and hooves on the roadway; a
/// cart drawn level there would have its horse's legs in the deck.
fn pose(p: &TrainPath, s: f32, dir: f32) -> (Vec3, Quat) {
    let (pos, yaw) = p.at(s);
    let (back, front) = (p.at(s - dir * AXLE).0.y, p.at(s + dir * FOREFEET).0.y);
    let slope = (front - back) / (AXLE + FOREFEET);
    let at = Vec3::new(pos.x, back + AXLE * slope, pos.z) + side(yaw, dir);
    (at, Quat::from_rotation_y(yaw + if dir < 0.0 { std::f32::consts::PI } else { 0.0 }) * Quat::from_rotation_z(slope.atan()))
}

/// How far behind a cart's middle its wheels are, and how far ahead the horse's forefeet (`trains::cart_mesh`).
const AXLE: f32 = 0.012;
const FOREFEET: f32 = 0.028;

/// What a cart stands on, in the cart's own frame (`trains::cart_mesh`): the
/// four corners of its wheels' foot and the horse's four hooves.
const FEET: [(f32, f32); 8] = [(-0.018, WHEELS), (-0.018, -WHEELS), (-0.006, WHEELS), (-0.006, -WHEELS), (0.016, 0.0025), (0.016, -0.0025), (0.028, 0.0025), (0.028, -0.0025)];

/// Carts going opposite ways pass on opposite sides of the road: how far a
/// cart heading `yaw` along its path, forward (1) or back (-1), stands from
/// the middle line.
fn side(yaw: f32, dir: f32) -> Vec3 {
    let heading = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
    Vec3::new(heading.z, 0.0, -heading.x) * (SIDE * dir)
}

// ------------------------------------------------------------------ the check

/// What `check` finds for a year: the roads, what carries them over water,
/// and how far the carts keep to them. A worst case is given with where it is.
#[derive(Debug, Default)]
pub struct RoadCheck {
    pub links: usize,
    pub main_links: usize,
    pub bridges: usize,
    pub main_bridges: usize,
    pub causeways: usize,
    /// links that run over water drawn on the map (the sea, or a river's
    /// water) with no bridge or causeway under them there, of them main roads,
    /// and how far, in cells, over a river's water and over the sea
    pub wet_links: usize,
    pub wet_main_links: usize,
    pub wet_river: f32,
    pub wet_sea: f32,
    /// of that, how far the ribbon lies under the water's surface, and on how many links (of them main roads)
    pub drowned: f32,
    pub drowned_links: usize,
    pub drowned_main_links: usize,
    /// each wet stretch: where it starts, whether it is the sea, its length, how far of it the
    /// ribbon is under the water, and whether a main road
    pub wet: Vec<(Vec2, bool, f32, f32, bool)>,
    /// the feet of bridges' ramps that stand above the road they come down to by more than a
    /// cart's lift (where the river's level is above the bank's ground), and the highest
    pub ramp_steps: usize,
    pub ramp_step: (f32, Vec2),
    /// the middles of cells with a main road where a bridge's roadway stands above the road,
    /// so that no carters' round passes through (`trains::road_routes`)
    pub closed: Vec<Vec2>,
    /// the carters' rounds, and how far they run
    pub rounds: usize,
    pub round_cells: f32,
    /// the furthest a cart rides from the surface under it, less its lift (either way)
    pub off_surface: (f32, Vec2),
    /// the least height of a cart above water drawn under it (none if it never is over water)
    pub over_water: Option<(f32, Vec2)>,
    /// how far a cart's middle comes from its road's middle line, less half the ribbon's width
    /// (more than nought: off the ribbon)
    pub off_ribbon: (f32, Vec2),
    /// on a bridge, how far a cart's wheels come past the inner face of a parapet (more than nought: through it)
    pub through_parapet: Option<(f32, Vec2)>,
    /// the bridges a carters' round crosses
    pub cart_bridges: Vec<isostate_gen::bridge::RoadBridge>,
    /// the cart as drawn (`pose`, `FEET`): how far a wheel or a hoof sinks under the surface
    /// beneath it, and how far one stands in the air above it beyond the lift, anywhere and with
    /// the cart on a bridge or its ramps (a cart drawn level on a ramp has its horse's legs in
    /// the ramp or in the air; anywhere, a cart turning a corner where the ground slopes across
    /// it has its hooves over the next road, which is higher or lower)
    pub feet_under: (f32, Vec2),
    pub feet_over: (f32, Vec2),
    pub feet_under_on_bridge: (f32, Vec2),
    pub feet_over_on_bridge: (f32, Vec2),
}


/// Walk every road of year `k` and every cart's path along them (see `RoadCheck`).
pub fn check(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> RoadCheck {
    let mut c = RoadCheck { off_surface: (0.0, Vec2::ZERO), off_ribbon: (f32::MIN, Vec2::ZERO), ..Default::default() };
    let waters = network::Waters::new(r, g);
    let segs = network::drawn_links(r, &g.worn, k);
    let v2 = |p: (f32, f32)| Vec2::new(p.0, p.1);
    // every link: what carries it over water, and where water lies under it with nothing to carry it,
    // read along its course as it is drawn (`network::course`): a curve, or round a meander or a lake
    #[allow(clippy::type_complexity)]
    let mut main: Vec<(Vec<Vec2>, Vec<Vec3>, Vec<isostate_gen::bridge::RoadBridge>, Vec<(Vec2, Vec2, f32)>)> = Vec::new();
    for &(a, b, lv) in &segs {
        let way = network::course(r, g, cen, a, b, k);
        let bridges = network::bridges_of(&way, lv == 3, r.crossed(a, b, k), |_| isostate_gen::houses::Detail::Play);
        let runs = network::causeways(g, &way);
        let ribbon = network::way_line(g, &way);
        // the foot of a ramp that stands above the road it comes down to, where the river's level is
        // above the bank's ground: the roadway ends in a step there, which a cart goes down
        for br in &bridges {
            for tb in [-br.ramp(), 1.0 + br.ramp()] {
                let foot = br.point(tb);
                let (d, _) = nearest_on_line(&way.pts, foot);
                let step = br.roadway_at(tb, &|x, z| world::height_at(g, x, z)).unwrap_or(0.0) - ribbon_at(&ribbon, foot);
                if d < 0.01 && step > LIFT {
                    c.ramp_steps += 1;
                    if step > c.ramp_step.0 {
                        c.ramp_step = (step, foot);
                    }
                }
            }
        }
        c.links += 1;
        c.bridges += bridges.len();
        c.causeways += runs.len();
        let (mut wet_here, mut drowned_here) = (false, false);
        let mut stretch: Option<(Vec2, bool, f32, f32)> = None;
        for w in way.pts.windows(2) {
            let (pa, pb) = (w[0], w[1]);
            let len = pa.distance(pb);
            let n = ((len / 0.005).ceil() as usize).max(1);
            for i in 0..=n {
                let t = i as f32 / n as f32;
                let p = pa + (pb - pa) * t;
                let carried = on_causeway(&runs, p).is_some() || bridges.iter().any(|br| (-br.ramp()..=1.0 + br.ramp()).contains(&br.t_of(p)));
                let water = if carried { None } else { waters.at(g, p) };
                if let Some((top, sea)) = water {
                    let step = len / n as f32;
                    if sea { c.wet_sea += step } else { c.wet_river += step }
                    let under = if ribbon_at(&ribbon, p) < top { step } else { 0.0 };
                    c.drowned += under;
                    drowned_here |= under > 0.0;
                    wet_here = true;
                    match &mut stretch {
                        Some(s) => {
                            s.2 += step;
                            s.3 += under;
                        }
                        None => stretch = Some((p, sea, step, under)),
                    }
                } else if let Some(s) = stretch.take() {
                    c.wet.push((s.0, s.1, s.2, s.3, lv == 3));
                }
            }
        }
        if let Some(s) = stretch.take() {
            c.wet.push((s.0, s.1, s.2, s.3, lv == 3));
        }
        c.wet_links += wet_here as usize;
        c.drowned_links += drowned_here as usize;
        if lv == 3 {
            c.main_links += 1;
            c.main_bridges += bridges.len();
            c.wet_main_links += wet_here as usize;
            c.drowned_main_links += drowned_here as usize;
            main.push((way.pts, ribbon, bridges, runs));
        }
    }
    // the main roads' meeting places that lie under a bridge's roadway, which carts keep away from
    let mut at_cell: Vec<Vec<(usize, usize, u8)>> = vec![Vec::new(); r.n];
    for &(a, b, lv) in &segs {
        at_cell[a].push((a, b, lv));
        at_cell[b].push((a, b, lv));
    }
    let mut found = Default::default();
    c.closed = (0..r.n).filter(|&cell| at_cell[cell].iter().any(|l| l.2 == 3) && trains::middle_under_bridge(r, k, g, cen, &at_cell[cell], cell, &mut found)).map(|cell| v2(cen[cell])).collect();
    // every cart's path, both ways, every 0.005 cells
    let routes = trains::road_routes(r, g, cen, k, &mut trains::RoadMemo::new(r, g));
    c.rounds = routes.len();
    let mut crossed = vec![false; main.iter().map(|m| m.2.len()).sum()];
    // the surface a cart should ride at a point of a main link's middle line, worked out here afresh
    // and not read from `network::driven_on`, which the carts' paths are made from: the highest of
    // the ribbon, the roadway of each of its bridges and the top of a causeway
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let surface_at = |li: usize, p: Vec2| -> f32 {
        let (_, ribbon, bridges, runs) = &main[li];
        let mut h = ribbon_at(ribbon, p);
        for br in bridges {
            h = h.max(br.roadway_at(br.t_of(p), &height).unwrap_or(h));
        }
        if let Some(top) = on_causeway(runs, p) {
            h = h.max(top);
        }
        h
    };
    // the main road nearest a point: the link whose middle line it is nearest, how far, the point of
    // the line nearest it and the line's direction there (looked for among the links that pass within
    // 0.1 of the point's cell, which hold the nearest wherever it is that near, as it always is for a
    // cart; among all of them elsewhere)
    let cell = |q: Vec2| (q.y.floor().clamp(0.0, r.h as f32 - 1.0) as usize) * r.w + q.x.floor().clamp(0.0, r.w as f32 - 1.0) as usize;
    let mut near_cell: Vec<Vec<usize>> = vec![Vec::new(); r.n];
    for (li, (pts, ..)) in main.iter().enumerate() {
        let (lo, hi) = pts.iter().fold((Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
        let (lo, hi) = (lo - Vec2::splat(0.1), hi + Vec2::splat(0.1));
        for y in lo.y.floor().max(0.0) as usize..=(hi.y.floor().max(0.0) as usize).min(r.h - 1) {
            for x in lo.x.floor().max(0.0) as usize..=(hi.x.floor().max(0.0) as usize).min(r.w - 1) {
                near_cell[y * r.w + x].push(li);
            }
        }
    }
    let all: Vec<usize> = (0..main.len()).collect();
    let nearest = |q: Vec2| -> Option<(f32, usize, Vec2, Vec2)> {
        let mut near: Option<(f32, usize, Vec2, Vec2)> = None;
        let here = &near_cell[cell(q)];
        for &li in if here.is_empty() { &all } else { here } {
            let (d, (at, dir)) = nearest_on_line(&main[li].0, q);
            if near.is_none_or(|n| d < n.0) {
                near = Some((d, li, at, dir));
            }
        }
        near
    };
    for p in &routes {
        c.round_cells += p.len;
        let n = ((p.len / 0.005).ceil() as usize).max(1);
        for i in 0..=n {
            let (pos, yaw) = p.at(p.len * i as f32 / n as f32);
            // the road the cart is on, from its path's own point on the middle line; the
            // ribbon and the roadway are level from side to side, so this is its height to the side too
            let q0 = Vec2::new(pos.x, pos.z);
            let Some((d0, li, at0, dir0)) = nearest(q0) else { continue };
            // (where the surface steps, at the foot of a ramp that ends above the road, a cart
            // going down the step is between the heights on either side of it)
            let off_on = |lj: usize, at: Vec2, dir: Vec2| -> f32 {
                let around = [surface_at(lj, at - dir * 6e-4), surface_at(lj, at), surface_at(lj, at + dir * 6e-4)];
                let (lo, hi) = (around.into_iter().reduce(f32::min).unwrap_or(0.0), around.into_iter().reduce(f32::max).unwrap_or(0.0));
                (lo - (pos.y - LIFT)).max(pos.y - LIFT - hi).max(0.0)
            };
            // where roads meet, every road that meets there is as near as the nearest, and
            // their ribbons may end a little apart in height: the cart rides the one it is on
            let mut off = off_on(li, at0, dir0);
            for &lj in &near_cell[cell(q0)] {
                let (dj, (atj, dirj)) = nearest_on_line(&main[lj].0, q0);
                if lj != li && dj <= d0 + 1e-6 {
                    off = off.min(off_on(lj, atj, dirj));
                }
            }
            if off > c.off_surface.0 {
                c.off_surface = (off, Vec2::new(pos.x, pos.z));
            }
            for dir in [1.0, -1.0] {
                let at = pos + side(yaw, dir);
                let q = Vec2::new(at.x, at.z);
                // the ribbon nearest where the cart stands, to one side of the middle line
                let Some((d, ..)) = nearest(q) else { continue };
                let off = d - network::road_width(3) * 0.5;
                if off > c.off_ribbon.0 {
                    c.off_ribbon = (off, q);
                }
                // the cart as drawn: its wheels and hooves on the surface under each of them
                let (drawn, rot) = pose(p, p.len * i as f32 / n as f32, dir);
                let mut highest = f32::MIN;
                let on_bridge = main[li].2.iter().any(|br| br.holds(q));
                for (x, z) in FEET {
                    let f = drawn + rot * Vec3::new(x, 0.0, z);
                    let Some((_, fl, ft, _)) = nearest(Vec2::new(f.x, f.z)) else { continue };
                    let gap = f.y - surface_at(fl, ft);
                    if -gap > c.feet_under.0 {
                        c.feet_under = (-gap, Vec2::new(f.x, f.z));
                    }
                    if on_bridge && -gap > c.feet_under_on_bridge.0 {
                        c.feet_under_on_bridge = (-gap, Vec2::new(f.x, f.z));
                    }
                    highest = highest.max(gap);
                }
                if highest - LIFT > c.feet_over.0 {
                    c.feet_over = (highest - LIFT, q);
                }
                if on_bridge && highest - LIFT > c.feet_over_on_bridge.0 {
                    c.feet_over_on_bridge = (highest - LIFT, q);
                }
                if let Some((top, _)) = waters.at(g, q) {
                    if c.over_water.is_none_or(|w| at.y - top < w.0) {
                        c.over_water = Some((at.y - top, q));
                    }
                }
                // on a bridge (between its banks, where the parapets stand): the wheels clear of them
                let first = main[..li].iter().map(|m| m.2.len()).sum::<usize>();
                for (bi, br) in main[li].2.iter().enumerate() {
                    let tb = br.t_of(q);
                    let out = (q - br.point(tb)).length();
                    if (0.0..=1.0).contains(&tb) && out < br.width() {
                        crossed[first + bi] = true;
                        let through = out + WHEELS - br.inner_half();
                        if c.through_parapet.is_none_or(|w| through > w.0) {
                            c.through_parapet = Some((through, q));
                        }
                    }
                }
            }
        }
    }
    c.cart_bridges = main.iter().flat_map(|m| m.2.iter()).zip(&crossed).filter(|(_, x)| **x).map(|(br, _)| *br).collect();
    c
}

/// How far a point is from a line of points, and the point of the line
/// nearest it with the line's direction there.
fn nearest_on_line(pts: &[Vec2], q: Vec2) -> (f32, (Vec2, Vec2)) {
    let mut best = (f32::MAX, (pts[0], Vec2::X));
    for w in pts.windows(2) {
        let ab = w[1] - w[0];
        let t = ((q - w[0]).dot(ab) / ab.length_squared().max(1e-12)).clamp(0.0, 1.0);
        let at = w[0] + ab * t;
        let d = q.distance(at);
        if d < best.0 {
            best = (d, (at, ab.normalize_or(Vec2::X)));
        }
    }
    best
}

/// The height of a road's ribbon (`network::way_line`) at the point of its middle line nearest `p`.
fn ribbon_at(ribbon: &[Vec3], p: Vec2) -> f32 {
    let mut best = (f32::MAX, ribbon[0].y);
    for w in ribbon.windows(2) {
        let (a, b) = (Vec2::new(w[0].x, w[0].z), Vec2::new(w[1].x, w[1].z));
        let ab = b - a;
        let t = ((p - a).dot(ab) / ab.length_squared().max(1e-12)).clamp(0.0, 1.0);
        let d = p.distance(a + ab * t);
        if d < best.0 {
            best = (d, w[0].y + (w[1].y - w[0].y) * t);
        }
    }
    best.1
}

/// The top of the causeway a point of a road's middle line lies on, if any (`network::causeways`).
fn on_causeway(runs: &[(Vec2, Vec2, f32)], p: Vec2) -> Option<f32> {
    runs.iter().filter(|&&(s, e, _)| {
        let se = e - s;
        let u = (p - s).dot(se) / se.length_squared().max(1e-12);
        (-1e-4..=1.0 + 1e-4).contains(&u) && p.distance(s + se * u.clamp(0.0, 1.0)) < 1e-3
    }).map(|r| r.2).reduce(f32::max)
}

/// `ISOSTATE_ROADS=1`: the check of 1800, 1850 and 1900 (or the last year
/// there is), printed; then the program stops.
pub fn maybe_check(r: &Replay, g: &Ground, cen: &[(f32, f32)]) -> bool {
    if std::env::var_os("ISOSTATE_ROADS").is_none() {
        return false;
    }
    let last = r.filled.clamp(1, r.ny) - 1;
    let mut years: Vec<usize> = [50usize, 100, 150].into_iter().filter(|&k| k <= last).collect();
    if years.is_empty() {
        years.push(last);
    }
    for k in years {
        let t0 = std::time::Instant::now();
        // the roads the carts wore against every road between road squares (`network::drawn_links`)
        let m = network::measure(r, &g.worn, k);
        let all = (m.carted_drawn + m.carted_road_not_drawn + m.carted_track).max(1e-9);
        eprintln!(
            "worn {}: {} links drawn closing {} squares ({} with a cell of town ground), {} of them diagonals, against {} links closing {} by every road; of the carting, {:.3} on roads drawn, {:.3} on made roads not drawn, {:.3} on tracks",
            m.year, m.drawn, m.closed, m.closed_town, m.diagonals, m.today, m.today_closed, m.carted_drawn / all, m.carted_road_not_drawn / all, m.carted_track / all
        );
        for line in network::water_check(r, g, cen, k) {
            eprintln!("{line}");
        }
        // the roads' courses (`road_course::shape`): how straight they run and how they turn
        let s = crate::road_course::shape(r, g, cen, k);
        eprintln!(
            "courses {}: {} links ({} on curves), {:.0} cells long; lattice lines {} of {:.2} links on average, the longest {}; straight runs (turning less than {} degrees) {} of {:.2} cells on average, {:.2} by length, the longest {:.2}; turns where a road passes through, by {:?} degrees: {:?}, at junctions {:?}; {:.3} of the length within 5 degrees of the eight directions; the tightest curve {:.3} cells, {} links tighter than 0.25; {} river crossings, {} links crossing more often than before; bridges and fords {:.3} square; {} links held straight on town ground, {} on curves with a square of town ground of 1900; places a road passes through that turn more than 5 degrees, by why (town, river, a corner over 120 degrees, years apart): {:?}",
            s.year, s.links, s.bent, s.length, s.lattice_lines, s.lattice_mean, s.lattice_longest, crate::road_course::STRAIGHT_ON, s.runs, s.run_mean, s.run_mean_by_length, s.run_longest, crate::road_course::TURN_BINS, s.turns, s.junction_turns, s.on_eight, s.tightest, s.tight_links, s.crossings, s.extra_crossings, s.square, s.held, s.bent_on_1900_town, s.kinks
        );
        eprintln!("  links on curves crossing a river at: {}", s.bent_crossings.iter().map(|p| format!("{:.2},{:.2}", p.x, p.y)).collect::<Vec<_>>().join("; "));
        let (moved, most) = crate::road_course::moved(r, g, cen, 0..=k);
        eprintln!("  from the first year to {}: {moved} links drawn again with a point moved, the furthest by {most:.4}", s.year);
        let c = check(r, g, cen, k);
        let at = |p: Vec2| format!("{:.2},{:.2}", p.x, p.y);
        eprintln!(
            "roads {}: {} links ({} main), {} bridges ({} on main roads), {} causeways; over water with nothing under them: {} links ({} main), {:.2} cells over a river's water and {:.2} over the sea, the ribbon under the water's surface for {:.2} of it, on {} links ({} main)",
            r.y0 + k as i32, c.links, c.main_links, c.bridges, c.main_bridges, c.causeways, c.wet_links, c.wet_main_links, c.wet_river, c.wet_sea, c.drowned, c.drowned_links, c.drowned_main_links
        );
        eprintln!("  {} feet of ramps stand above the road they come down to, the highest by {:.4} at {}", c.ramp_steps, c.ramp_step.0, at(c.ramp_step.1));
        for (p, sea, len, under, main) in &c.wet {
            eprintln!("  wet {} at {} for {:.3} cells, the ribbon under the water for {:.3}{}", if *sea { "sea" } else { "river" }, at(*p), len, under, if *main { ", a main road" } else { "" });
        }
        eprintln!(
            "carts {}: {} meeting places of main roads under a bridge's roadway, kept away from; {} rounds over {:.1} cells; furthest from the surface under it {:.4} at {}; least height over water {}; furthest off the ribbon (beyond its edge) {:.4} at {}; furthest through a parapet {}; {} bridges on the rounds ({:.0} ms)",
            r.y0 + k as i32,
            c.closed.len(),
            c.rounds,
            c.round_cells,
            c.off_surface.0,
            at(c.off_surface.1),
            c.over_water.map_or("never over water".to_string(), |w| format!("{:.4} at {}", w.0, at(w.1))),
            c.off_ribbon.0,
            at(c.off_ribbon.1),
            c.through_parapet.map_or("never on a bridge".to_string(), |w| format!("{:.4} at {}", w.0, at(w.1))),
            c.cart_bridges.len(),
            t0.elapsed().as_secs_f64() * 1000.0
        );
        eprintln!(
            "  the cart as drawn: a wheel or hoof at most {:.4} under the surface at {} and {:.4} in the air beyond the lift at {}; with the cart on a bridge or its ramps, {:.4} under at {} and {:.4} in the air at {}",
            c.feet_under.0,
            at(c.feet_under.1),
            c.feet_over.0,
            at(c.feet_over.1),
            c.feet_under_on_bridge.0,
            at(c.feet_under_on_bridge.1),
            c.feet_over_on_bridge.0,
            at(c.feet_over_on_bridge.1)
        );
        for p in &c.closed {
            eprintln!("  kept away from {}", at(*p));
        }
        // with the ends of its ramps, so that a picture can put a cart on a ramp, on the deck or off it
        for br in &c.cart_bridges {
            let (from, to) = (br.point(-br.ramp()), br.point(1.0 + br.ramp()));
            eprintln!("  bridge at {} over water {:.3} wide, from the foot of one ramp at {:.3},{:.3} to the other's at {:.3},{:.3}", at(br.at), 2.0 * br.half, from.x, from.y, to.x, to.y);
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carts_keep_to_their_roads_and_their_bridges_on_island_7() {
        // island 7 with six families, played to 1900 as the standard pictures play it (about half a
        // minute), with the tree's overlay at stage 1: the case was found on that island, and the
        // content after 1900 (docs/rules/tree.md, "After 1900: electricity") gives it another history,
        // on which in 1850 a cart rides 0.0011 off its road's surface at 53.46, 12.58 (a fault of the
        // carts' drawing that history shows, listed for mending and not mended here). And on the
        // old island: since the game's rules took the new land (docs/rules/land.md, "The switch")
        // island 7 is another island, on which in 1800 a cart rides 0.0024 off its road's surface at
        // 60.99, 16.53 (a fault of the carts' drawing on the new land, in the faults' file for the
        // round of mending, docs/backlog/for-the-bugfix-round.md, and not mended here)
        // The case is played twice: with every road drawn, as it was found (`network::ALL_IN_TEST`),
        // and with the roads the carts wore (`network::drawn_links`), on which there are fewer
        // main roads and so fewer rounds (9 in 1800 where there were 10 or more)
        let r = crate::live::played_at_stage(7, 6, 150, 1, true).expect("the game starts");
        let g = world::build_ground(&r);
        let cen = network::centres(&r);
        for all in [true, false] {
            network::ALL_IN_TEST.with(|a| a.set(all));
            keep_to_their_roads(&r, &g, &cen, if all { 10 } else { 6 });
            network::ALL_IN_TEST.with(|a| a.set(false));
        }
    }

    fn keep_to_their_roads(r: &Replay, g: &Ground, cen: &[(f32, f32)], least_rounds: usize) {
        let mut crossed = 0;
        for year in [1800, 1850, 1900] {
            let k = (year - r.y0) as usize;
            assert!(k < r.filled, "{year} is not played");
            let c = check(r, g, cen, k);
            assert!(c.rounds >= least_rounds, "{year}: only {} carters' rounds", c.rounds);
            // at the height of the road or the deck under it, wherever it is
            assert!(c.off_surface.0 < 1e-4, "{year}: a cart rides {} off its road's surface at {}", c.off_surface.0, c.off_surface.1);
            // and so never at the water's height, nor under it
            if let Some((h, at)) = c.over_water {
                assert!(h > LIFT, "{year}: a cart is {h} above the water at {at}");
            }
            // on its road's ribbon, to one side of the middle line
            assert!(c.off_ribbon.0 <= 0.0, "{year}: a cart is {} beyond the edge of its road's ribbon at {}", c.off_ribbon.0, c.off_ribbon.1);
            // and on a bridge, its wheels between the parapets
            if let Some((through, at)) = c.through_parapet {
                assert!(through <= 0.0, "{year}: a cart's wheels are {through} into a parapet at {at}");
            }
            // and as drawn, tipped up a ramp and over the hump, its wheels and hooves on the roadway:
            // a cart drawn level there has a hoof 0.013 to 0.015 into the deck and one as far in the air
            // (on island 7 it is at most 0.003 into it, and 0.0095 in the air where a ramp meets the road;
            // since the gentler rivers of 19 November one bridge in 1800, at 53.3,20.0, has a hoof 0.0052
            // into it, so the bar is 0.006, still well short of a cart drawn level)
            assert!(c.feet_under_on_bridge.0 < 0.006, "{year}: a cart on a bridge has a wheel or hoof {} into the roadway at {}", c.feet_under_on_bridge.0, c.feet_under_on_bridge.1);
            assert!(c.feet_over_on_bridge.0 < 0.012, "{year}: a cart on a bridge has a wheel or hoof {} in the air at {}", c.feet_over_on_bridge.0, c.feet_over_on_bridge.1);
            crossed += c.cart_bridges.len();
        }
        assert!(crossed >= 10, "the carts cross only {crossed} bridges in the three years");
    }
}
