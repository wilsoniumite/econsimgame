//! Little trains running up and down the railway lines.

use bevy::math::Vec3;

use crate::look::Look;
use crate::meshkit::Builder;
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
pub fn road_routes(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> Vec<TrainPath> {
    let adj = graph(r, k, |v| v & 3 == 3);
    let mut out = Vec::new();
    for start in 0..r.n {
        if adj[start].is_empty() || world::hash2(start as i32, 5, 44) > 0.03 {
            continue;
        }
        let mut cells = vec![start];
        let (mut prev, mut cur) = (usize::MAX, start);
        for step in 0..12 {
            let options: Vec<usize> = adj[cur].iter().copied().filter(|&m| m != prev && !cells.contains(&m)).collect();
            if options.is_empty() {
                break;
            }
            let pick = (world::hash2(start as i32, step, 45) * options.len() as f32) as usize;
            prev = cur;
            cur = options[pick.min(options.len() - 1)];
            cells.push(cur);
        }
        if let Some(p) = along(&cells, g, cen, 0.012) {
            if p.len > 1.5 {
                out.push(p);
            }
        }
    }
    out
}

/// The cells whose transport byte passes `on`, linked to their neighbours (a
/// diagonal only where no pair of straight links joins the two).
fn graph(r: &Replay, k: usize, on: impl Fn(u8) -> bool) -> Vec<Vec<usize>> {
    let (w, h, n) = (r.w, r.h, r.n);
    let tr = &r.trans[k * n..(k + 1) * n];
    let rail = |x: isize, y: isize| x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && on(tr[y as usize * w + x as usize]);
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for c in 0..n {
        if !on(tr[c]) {
            continue;
        }
        let (x, y) = ((c % w) as isize, (c / w) as isize);
        for (dx, dy) in [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)] {
            let (nx, ny) = (x + dx, y + dy);
            if !rail(nx, ny) {
                continue;
            }
            if dx != 0 && dy != 0 && (rail(x + dx, y) || rail(x, y + dy)) {
                continue;
            }
            let m = ny as usize * w + nx as usize;
            adj[c].push(m);
            adj[m].push(c);
        }
    }
    adj
}

/// A path through the centres of a list of cells, draped `lift` above the ground.
fn along(cells: &[usize], g: &Ground, cen: &[(f32, f32)], lift: f32) -> Option<TrainPath> {
    if cells.len() < 2 {
        return None;
    }
    let mut pts = Vec::new();
    for wnd in cells.windows(2) {
        let (a, b) = (cen[wnd[0]], cen[wnd[1]]);
        let steps = 6;
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let (x, z) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            pts.push(Vec3::new(x, world::height_at(g, x, z).max(0.0) + lift, z));
        }
    }
    let last = cen[*cells.last()?];
    pts.push(Vec3::new(last.0, world::height_at(g, last.0, last.1).max(0.0) + lift, last.1));
    let mut cum = vec![0.0];
    for i in 1..pts.len() {
        let d = pts[i].distance(pts[i - 1]);
        cum.push(cum[i - 1] + d);
    }
    let len = *cum.last()?;
    Some(TrainPath { pts, cum, len })
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
