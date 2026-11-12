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
use crate::{Data, Timeline};

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

#[derive(Resource, Default)]
struct Roads(Vec<TrainPath>);

pub fn plugin(app: &mut App) {
    app.init_resource::<Roads>().add_systems(Startup, setup).add_systems(Update, (place, drive).chain());
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

#[allow(clippy::too_many_arguments)]
fn place(mut commands: Commands, tl: Res<Timeline>, data: Res<Data>, assets: Option<Res<CartAssets>>, mut roads: ResMut<Roads>, old: Query<Entity, With<Cart>>, mut last: Local<Option<usize>>, mut memo: Local<Option<trains::RoadMemo>>) {
    let _frames = crate::frames::span(crate::frames::Sys::Traffic);
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
    // what is the same in every year is kept from one to the next (`trains::RoadMemo`)
    roads.0 = world::timed("year carts", || {
        let memo = memo.get_or_insert_with(|| trains::RoadMemo::new(r, &data.g));
        trains::road_routes(r, &data.g, &data.cen, k, memo)
    });
    // a cart asked for at a spot, for a picture: the first of the round nearest it, held still there
    let spots = pinned_spots();
    let pins: Vec<(usize, f32)> = spots.iter().flatten().filter_map(|&at| nearest_on(&roads.0, at)).collect();
    for (pi, p) in roads.0.iter().enumerate() {
        // one or two carts on each carter's round
        let count = 1 + (world::hash2(pi as i32, 7, 43) < 0.35) as usize;
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
        let Some(p) = roads.0.get(c.path) else { continue };
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
    let height_along = network::height_along;
    let segs = network::road_segments(r, k);
    let v2 = |p: (f32, f32)| Vec2::new(p.0, p.1);
    // every link: what carries it over water, and where water lies under it with nothing to carry it
    let mut main: Vec<(Vec2, Vec2, Vec<Vec3>, Vec<isostate_gen::bridge::RoadBridge>)> = Vec::new();
    for &(a, b, lv) in &segs {
        let (pa, pb) = (v2(cen[a]), v2(cen[b]));
        let bridges = network::road_bridges(g, pa, pb, lv == 3, |_| isostate_gen::houses::Detail::Play);
        let runs = network::causeway_runs(g, pa, pb);
        let ribbon = network::road_line(g, cen[a], cen[b]);
        c.links += 1;
        c.bridges += bridges.len();
        c.causeways += runs.len();
        let len = pa.distance(pb);
        let n = ((len / 0.005).ceil() as usize).max(1);
        let (mut wet_here, mut drowned_here) = (false, false);
        let mut stretch: Option<(Vec2, bool, f32, f32)> = None;
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let p = pa + (pb - pa) * t;
            let carried = runs.iter().any(|&(t0, t1)| (t0..=t1).contains(&t)) || bridges.iter().any(|br| (-br.ramp()..=1.0 + br.ramp()).contains(&br.t_of(p)));
            let water = if carried { None } else { waters.at(g, p) };
            if let Some((top, sea)) = water {
                let step = len / n as f32;
                if sea { c.wet_sea += step } else { c.wet_river += step }
                let under = if height_along(&ribbon, t) < top { step } else { 0.0 };
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
            main.push((pa, pb, network::driven(g, cen[a], cen[b], true), bridges));
        }
    }
    // the main roads' meeting places that lie under a bridge's roadway, which carts keep away from
    let mut at_cell: Vec<Vec<(usize, usize, u8)>> = vec![Vec::new(); r.n];
    for &(a, b, lv) in &segs {
        at_cell[a].push((a, b, lv));
        at_cell[b].push((a, b, lv));
    }
    let mut found = Default::default();
    c.closed = (0..r.n).filter(|&cell| at_cell[cell].iter().any(|l| l.2 == 3) && trains::middle_under_bridge(g, cen, &at_cell[cell], cell, &mut found)).map(|cell| v2(cen[cell])).collect();
    // every cart's path, both ways, every 0.005 cells
    let routes = trains::road_routes(r, g, cen, k, &mut trains::RoadMemo::new(r, g));
    c.rounds = routes.len();
    let mut crossed = vec![false; main.iter().map(|m| m.3.len()).sum()];
    // the main road nearest a point: the link whose middle line it is nearest, how far, and how far along
    // (looked for among the links that pass within 0.1 of the point's cell, which hold the
    // nearest wherever it is that near, as it always is for a cart; among all of them elsewhere)
    let cell = |q: Vec2| (q.y.floor().clamp(0.0, r.h as f32 - 1.0) as usize) * r.w + q.x.floor().clamp(0.0, r.w as f32 - 1.0) as usize;
    let mut near_cell: Vec<Vec<usize>> = vec![Vec::new(); r.n];
    for (li, (a, b, _, _)) in main.iter().enumerate() {
        let (lo, hi) = (a.min(*b) - Vec2::splat(0.1), a.max(*b) + Vec2::splat(0.1));
        for y in lo.y.floor().max(0.0) as usize..=(hi.y.floor().max(0.0) as usize).min(r.h - 1) {
            for x in lo.x.floor().max(0.0) as usize..=(hi.x.floor().max(0.0) as usize).min(r.w - 1) {
                near_cell[y * r.w + x].push(li);
            }
        }
    }
    let all: Vec<usize> = (0..main.len()).collect();
    let nearest = |q: Vec2| -> Option<(f32, usize, f32)> {
        let mut near: Option<(f32, usize, f32)> = None;
        let here = &near_cell[cell(q)];
        for &li in if here.is_empty() { &all } else { here } {
            let (a, b) = (&main[li].0, &main[li].1);
            let ab = *b - *a;
            let t = ((q - *a).dot(ab) / ab.length_squared().max(1e-9)).clamp(0.0, 1.0);
            let d = q.distance(*a + ab * t);
            if near.is_none_or(|n| d < n.0) {
                near = Some((d, li, t));
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
            let Some((_, li, t)) = nearest(Vec2::new(pos.x, pos.z)) else { continue };
            let surface = height_along(&main[li].2, t);
            if (pos.y - LIFT - surface).abs() > c.off_surface.0 {
                c.off_surface = ((pos.y - LIFT - surface).abs(), Vec2::new(pos.x, pos.z));
            }
            for dir in [1.0, -1.0] {
                let at = pos + side(yaw, dir);
                let q = Vec2::new(at.x, at.z);
                // the ribbon nearest where the cart stands, to one side of the middle line
                let Some((d, _, _)) = nearest(q) else { continue };
                let off = d - network::road_width(3) * 0.5;
                if off > c.off_ribbon.0 {
                    c.off_ribbon = (off, q);
                }
                // the cart as drawn: its wheels and hooves on the surface under each of them
                let (drawn, rot) = pose(p, p.len * i as f32 / n as f32, dir);
                let mut highest = f32::MIN;
                let on_bridge = main[li].3.iter().any(|br| br.holds(q));
                for (x, z) in FEET {
                    let f = drawn + rot * Vec3::new(x, 0.0, z);
                    let Some((_, fl, ft)) = nearest(Vec2::new(f.x, f.z)) else { continue };
                    let gap = f.y - height_along(&main[fl].2, ft);
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
                let first = main[..li].iter().map(|m| m.3.len()).sum::<usize>();
                for (bi, br) in main[li].3.iter().enumerate() {
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
    c.cart_bridges = main.iter().flat_map(|m| m.3.iter()).zip(&crossed).filter(|(_, x)| **x).map(|(br, _)| *br).collect();
    c
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
        let c = check(r, g, cen, k);
        let at = |p: Vec2| format!("{:.2},{:.2}", p.x, p.y);
        eprintln!(
            "roads {}: {} links ({} main), {} bridges ({} on main roads), {} causeways; over water with nothing under them: {} links ({} main), {:.2} cells over a river's water and {:.2} over the sea, the ribbon under the water's surface for {:.2} of it, on {} links ({} main)",
            r.y0 + k as i32, c.links, c.main_links, c.bridges, c.main_bridges, c.causeways, c.wet_links, c.wet_main_links, c.wet_river, c.wet_sea, c.drowned, c.drowned_links, c.drowned_main_links
        );
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
        // island 7 with six families, played to 1900 as the standard pictures play it (about half a minute)
        let o = crate::live::Opening { seed: 7, families: 6, advance: 150, fresh: true, shots: true, ..Default::default() };
        let (r, _live, _view) = crate::live::start(&o).expect("the game starts");
        let g = world::build_ground(&r);
        let cen = network::centres(&r);
        let mut crossed = 0;
        for year in [1800, 1850, 1900] {
            let k = (year - r.y0) as usize;
            assert!(k < r.filled, "{year} is not played");
            let c = check(&r, &g, &cen, k);
            assert!(c.rounds >= 10, "{year}: only {} carters' rounds", c.rounds);
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
            // (on island 7 it is at most 0.003 into it, and 0.007 in the air where a ramp meets the road)
            assert!(c.feet_under_on_bridge.0 < 0.005, "{year}: a cart on a bridge has a wheel or hoof {} into the roadway at {}", c.feet_under_on_bridge.0, c.feet_under_on_bridge.1);
            assert!(c.feet_over_on_bridge.0 < 0.01, "{year}: a cart on a bridge has a wheel or hoof {} in the air at {}", c.feet_over_on_bridge.0, c.feet_over_on_bridge.1);
            crossed += c.cart_bridges.len();
        }
        assert!(crossed >= 10, "the carts cross only {crossed} bridges in the three years");
    }
}
