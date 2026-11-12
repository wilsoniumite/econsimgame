//! Little trains running up and down the railway lines.

use bevy::math::Vec3;

use crate::look::Look;
use crate::meshkit::Builder;
use crate::network;
use crate::replay::Replay;
use crate::world::{self, Ground};

pub struct TrainPath {
    pub pts: Vec<Vec3>,
    pub cum: Vec<f32>,
    pub len: f32,
}

impl TrainPath {
    /// Position and heading (radians about Y) at distance s along the line.
    pub fn at(&self, s: f32) -> (Vec3, f32) {
        let s = s.clamp(0.0, self.len);
        let i = self.cum.partition_point(|&c| c < s).clamp(1, self.pts.len() - 1);
        let (a, b) = (self.pts[i - 1], self.pts[i]);
        let seg = (self.cum[i] - self.cum[i - 1]).max(1e-6);
        let t = ((s - self.cum[i - 1]) / seg).clamp(0.0, 1.0);
        let d = b - a;
        (a + d * t, (-d.z).atan2(d.x))
    }
}

/// Railway lines for year k, along the graded track of rails.rs, so the trains
/// run on the embankments and viaducts and vanish into the tunnels.
pub fn rail_paths(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> Vec<TrainPath> {
    world::timed("year trains", || crate::rails::train_paths(&crate::rails::year(r, g, cen, k)))
}

/// Routes along the main roads for year k: carters' rounds of a dozen cells,
/// starting from a few cells picked by hash and turning by hash at junctions.
/// The rounds keep to the main roads as they are drawn (`network::road_segments`),
/// and each runs along its road's surface (`network::driven`): the ribbon,
/// and over a river the bridge. Two kinds of link are left out, where the
/// map shows no way a cart could go and a carter turns back: one whose road
/// goes under a river's water with nothing to carry it, and one that ends in
/// a cell whose middle, where the roads meet, has a bridge's roadway standing
/// above the road there (a river running through the middle of the cell, so
/// that the other roads come out from under the bridge, and a cart going on
/// from the bridge would drop through it).
pub fn road_routes(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> Vec<TrainPath> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); r.n];
    let mut links: Vec<Vec<(usize, usize, u8)>> = vec![Vec::new(); r.n];
    for (c, m, lv) in network::road_segments(r, k) {
        if lv == 3 {
            adj[c].push(m);
            adj[m].push(c);
        }
        links[c].push((c, m, lv));
        links[m].push((c, m, lv));
    }
    let mut covered: std::collections::HashMap<usize, bool> = Default::default();
    let mut under_bridge = |cell: usize| -> bool { *covered.entry(cell).or_insert_with(|| middle_under_bridge(g, cen, &links[cell], cell)) };
    // each link's surface as drawn, from the lower-numbered cell (the way it is drawn), worked
    // out when a round first comes to it; none where a cart cannot go
    let waters = network::Waters::new(r, g);
    let mut ways: std::collections::HashMap<(usize, usize), Option<Vec<Vec3>>> = Default::default();
    let mut way = |u: usize, v: usize| -> bool {
        let (lo, hi) = (u.min(v), u.max(v));
        if !ways.contains_key(&(lo, hi)) {
            let open = !under_bridge(lo) && !under_bridge(hi);
            let pts = open.then(|| network::driven(g, cen[lo], cen[hi], true)).filter(|pts| !waters.drowns(g, pts));
            ways.insert((lo, hi), pts);
        }
        ways[&(lo, hi)].is_some()
    };
    let mut rounds = Vec::new();
    for start in 0..r.n {
        if adj[start].is_empty() || world::hash2(start as i32, 5, 44) > 0.03 {
            continue;
        }
        let mut cells = vec![start];
        let (mut prev, mut cur) = (usize::MAX, start);
        for step in 0..12 {
            let options: Vec<usize> = adj[cur].iter().copied().filter(|&m| m != prev && !cells.contains(&m) && way(cur, m)).collect();
            if options.is_empty() {
                break;
            }
            let pick = (world::hash2(start as i32, step, 45) * options.len() as f32) as usize;
            prev = cur;
            cur = options[pick.min(options.len() - 1)];
            cells.push(cur);
        }
        rounds.push(cells);
    }
    let mut out = Vec::new();
    for cells in rounds {
        let mut pts: Vec<Vec3> = Vec::new();
        for w in cells.windows(2) {
            let Some(Some(link)) = ways.get(&(w[0].min(w[1]), w[0].max(w[1]))) else { continue };
            let mut link = link.clone();
            if w[0] > w[1] {
                link.reverse();
            }
            // the cart's wheels stand a little above the surface
            for p in &mut link {
                p.y += crate::carts::LIFT;
            }
            // the link begins where the last ended, at the middle of the cell between them
            if pts.last().is_some_and(|q| q.distance(link[0]) < 1e-6) {
                link.remove(0);
            }
            pts.extend(link);
        }
        if let Some(p) = TrainPath::new(pts) {
            if p.len > 1.5 {
                out.push(p);
            }
        }
    }
    out
}

/// Whether, at a cell's middle, the roadway of one of the bridges of the
/// roads that meet there (`links`, as from `network::road_segments`) stands
/// above the road's ribbon, by more than a cart's lift: the foot of a ramp
/// lies a little under the ribbon, and does not count.
pub fn middle_under_bridge(g: &Ground, cen: &[(f32, f32)], links: &[(usize, usize, u8)], cell: usize) -> bool {
    let middle = |c: usize| bevy::math::Vec2::new(cen[c].0, cen[c].1);
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    let p = middle(cell);
    let ribbon = height(p.x, p.y).max(0.0) + network::ROAD_LIFT;
    links.iter().any(|&(c, m, lv)| {
        network::road_bridges(g, middle(c), middle(m), lv == 3, |_| isostate_gen::houses::Detail::Play)
            .iter()
            .any(|b| b.holds(p) && b.roadway_at(b.t_of(p), &height).is_some_and(|h| h > ribbon + crate::carts::LIFT))
    })
}

impl TrainPath {
    /// A path along a line of points (none if it has fewer than two).
    pub fn new(pts: Vec<Vec3>) -> Option<TrainPath> {
        if pts.len() < 2 {
            return None;
        }
        let mut cum = vec![0.0];
        for i in 1..pts.len() {
            let d = pts[i].distance(pts[i - 1]);
            cum.push(cum[i - 1] + d);
        }
        let len = *cum.last()?;
        Some(TrainPath { pts, cum, len })
    }
}

/// A locomotive and three wagons, pointing along +x, centred on the engine.
pub fn train_mesh(lk: &Look) -> Builder {
    let mut b = Builder::default();
    let green = *lk.vehicles.engine;
    let black = *lk.vehicles.engine_black;
    b.block(Vec3::new(0.008, 0.0, 0.0), 0.05, 0.02, 0.02, 0.0, green, Some(green), 0.0);
    b.block(Vec3::new(-0.014, 0.0, 0.0), 0.016, 0.022, 0.034, 0.0, green, Some(black), 0.0);
    b.frustum(Vec3::new(0.026, 0.02, 0.0), 0.0045, 0.0055, 0.016, 6, black, Some(black));
    b.ball(Vec3::new(0.036, 0.048, 0.0), 0.009, 0.007, 6, 3, *lk.smoke.steam, 0.3);
    b.ball(Vec3::new(0.05, 0.056, 0.004), 0.007, 0.006, 6, 3, *lk.smoke.puff, 1.1);
    for (i, c) in lk.vehicles.wagons.iter().enumerate() {
        let x = -0.05 - 0.036 * i as f32;
        b.block(Vec3::new(x, 0.0, 0.0), 0.03, 0.019, 0.018, 0.0, **c, Some(**c), 0.0);
    }
    b
}

/// A covered cart and its horse, pointing along +x.
pub fn cart_mesh(lk: &Look) -> Builder {
    let mut b = Builder::default();
    let wood = *lk.vehicles.cart_wood;
    let canvas = *lk.vehicles.cart_canvas;
    let horse = *lk.vehicles.horse;
    let dark = *lk.vehicles.cart_wheels;
    // the cart: a bed, a canvas tilt and two wheels' worth of dark below
    b.block(Vec3::new(-0.012, 0.006, 0.0), 0.03, 0.016, 0.006, 0.0, wood, Some(wood), 0.0);
    b.ball(Vec3::new(-0.012, 0.014, 0.0), 0.016, 0.009, 6, 3, canvas, 0.2);
    b.block(Vec3::new(-0.012, 0.0, 0.0), 0.012, 0.019, 0.007, 0.0, dark, None, 0.0);
    // shafts and the horse
    b.beam(Vec3::new(0.003, 0.008, 0.005), Vec3::new(0.02, 0.01, 0.004), 0.002, wood);
    b.beam(Vec3::new(0.003, 0.008, -0.005), Vec3::new(0.02, 0.01, -0.004), 0.002, wood);
    b.block(Vec3::new(0.022, 0.008, 0.0), 0.018, 0.007, 0.008, 0.0, horse, Some(horse), 0.0);
    b.block(Vec3::new(0.033, 0.013, 0.0), 0.007, 0.005, 0.008, 0.0, horse, Some(horse), 0.0);
    for (dx, dz) in [(0.016, 0.0025), (0.016, -0.0025), (0.028, 0.0025), (0.028, -0.0025)] {
        b.beam(Vec3::new(dx, 0.0, dz), Vec3::new(dx, 0.009, dz), 0.002, dark);
    }
    b
}
