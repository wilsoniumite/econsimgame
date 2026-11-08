//! Little trains running up and down the railway lines.

use bevy::math::Vec3;

use crate::meshkit::{hex, Builder};
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

/// Railway lines for year k, as chains of cells between junctions and ends.
pub fn rail_paths(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> Vec<TrainPath> {
    let (w, h, n) = (r.w, r.h, r.n);
    let tr = &r.trans[k * n..(k + 1) * n];
    let rail = |x: isize, y: isize| x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && tr[y as usize * w + x as usize] & 4 != 0;
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for c in 0..n {
        if tr[c] & 4 == 0 {
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
    let mut used = std::collections::HashSet::new();
    let mut paths = Vec::new();
    for start in 0..n {
        if adj[start].is_empty() || adj[start].len() == 2 {
            continue;
        }
        for &first in &adj[start] {
            if used.contains(&(start.min(first), start.max(first))) {
                continue;
            }
            let mut cells = vec![start];
            let (mut prev, mut cur) = (start, first);
            loop {
                used.insert((prev.min(cur), prev.max(cur)));
                cells.push(cur);
                if adj[cur].len() != 2 {
                    break;
                }
                let next = if adj[cur][0] == prev { adj[cur][1] } else { adj[cur][0] };
                if used.contains(&(cur.min(next), cur.max(next))) {
                    break;
                }
                prev = cur;
                cur = next;
            }
            if cells.len() < 3 {
                continue;
            }
            let mut pts = Vec::new();
            for wnd in cells.windows(2) {
                let (a, b) = (cen[wnd[0]], cen[wnd[1]]);
                let steps = 6;
                for i in 0..steps {
                    let t = i as f32 / steps as f32;
                    let (x, z) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                    pts.push(Vec3::new(x, world::height_at(g, x, z).max(0.0) + 0.02, z));
                }
            }
            let last = cen[*cells.last().unwrap()];
            pts.push(Vec3::new(last.0, world::height_at(g, last.0, last.1).max(0.0) + 0.02, last.1));
            let mut cum = vec![0.0];
            for i in 1..pts.len() {
                let d = pts[i].distance(pts[i - 1]);
                cum.push(cum[i - 1] + d);
            }
            let len = *cum.last().unwrap();
            if len > 2.0 {
                paths.push(TrainPath { pts, cum, len });
            }
        }
    }
    paths
}

/// A locomotive and three wagons, pointing along +x, centred on the engine.
pub fn train_mesh() -> Builder {
    let mut b = Builder::default();
    let green = hex(0x2f6b55);
    let black = hex(0x2b2725);
    let wagons = [hex(0xb5523f), hex(0x8a6a4a), hex(0x6f7a8a)];
    b.block(Vec3::new(0.008, 0.0, 0.0), 0.05, 0.02, 0.02, 0.0, green, Some(green), 0.0);
    b.block(Vec3::new(-0.014, 0.0, 0.0), 0.016, 0.022, 0.034, 0.0, green, Some(black), 0.0);
    b.frustum(Vec3::new(0.026, 0.02, 0.0), 0.0045, 0.0055, 0.016, 6, black, Some(black));
    b.ball(Vec3::new(0.036, 0.048, 0.0), 0.009, 0.007, 6, 3, hex(0xf2f0ec), 0.3);
    b.ball(Vec3::new(0.05, 0.056, 0.004), 0.007, 0.006, 6, 3, hex(0xeeece8), 1.1);
    for (i, c) in wagons.iter().enumerate() {
        let x = -0.05 - 0.036 * i as f32;
        b.block(Vec3::new(x, 0.0, 0.0), 0.03, 0.019, 0.018, 0.0, *c, Some(*c), 0.0);
    }
    b
}
