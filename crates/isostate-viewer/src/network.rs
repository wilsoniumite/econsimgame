//! Rivers, roads and railways as ribbons over the terrain. The rivers are
//! made in rivers.rs and the railways in rails.rs; the roads are draped here.

use bevy::math::Vec3;

use crate::look::Look;
use crate::meshkit::Builder;
use crate::replay::Replay;
use crate::world::{self, Ground};

/// Cell centres with a little jitter, shared by every line on the map.
pub fn centres(r: &Replay) -> Vec<(f32, f32)> {
    (0..r.n)
        .map(|c| {
            let (x, y) = ((c % r.w) as f32, (c / r.w) as f32);
            (
                x + 0.5 + 0.36 * (world::hash2(c as i32, 11, 1) - 0.5),
                y + 0.5 + 0.36 * (world::hash2(c as i32, 13, 2) - 0.5),
            )
        })
        .collect()
}

fn drape(g: &Ground, pts2: &[(f32, f32)], lift: f32) -> Vec<Vec3> {
    // resample so the ribbon follows the ground closely
    let mut out = Vec::new();
    for w in pts2.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let n = ((len / 0.08).ceil() as usize).max(1);
        for i in 0..n {
            let t = i as f32 / n as f32;
            let (x, z) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            out.push(Vec3::new(x, world::height_at(g, x, z).max(0.0) + lift, z));
        }
    }
    if let Some(&(x, z)) = pts2.last() {
        out.push(Vec3::new(x, world::height_at(g, x, z).max(0.0) + lift, z));
    }
    out
}

/// The rivers: water and wet banks, lying on each river's own level (see rivers.rs).
pub fn rivers(g: &Ground, lk: &Look) -> Builder {
    crate::rivers::ribbons(&g.rivers, lk)
}

pub struct Nets {
    pub roads: Builder,
    pub rails: Builder,
}

/// The roads drawn in year `k`: (cell, neighbouring cell, level 2 or 3). Each
/// link appears once; a diagonal is left out where a pair of straight links
/// already joins the two cells.
pub fn road_segments(r: &Replay, k: usize) -> Vec<(usize, usize, u8)> {
    let (w, h, n) = (r.w, r.h, r.n);
    let tr = &r.trans[k * n..(k + 1) * n];
    let bit = |x: isize, y: isize| -> bool { x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && tr[y as usize * w + x as usize] & 2 != 0 };
    let mut out = Vec::new();
    for c in 0..n {
        let v = tr[c];
        if v & 3 < 2 {
            continue;
        }
        let (x, y) = ((c % w) as isize, (c / w) as isize);
        for (dx, dy) in [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)] {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || nx >= w as isize || ny >= h as isize {
                continue;
            }
            let m = ny as usize * w + nx as usize;
            let lv = (v & 3).min(tr[m] & 3);
            let diag = dx != 0 && dy != 0;
            if lv >= 2 && !(diag && (bit(x + dx, y) || bit(x, y + dy))) {
                out.push((c, m, lv));
            }
        }
    }
    out
}

pub fn year_nets(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, lk: &Look) -> Nets {
    world::timed("year nets", || year_nets_inner(r, g, cen, k, lk))
}

fn year_nets_inner(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, lk: &Look) -> Nets {
    let mut roads = Builder::default();
    // the railways are graded curves with their own works (rails.rs)
    let rails = crate::rails::track(&crate::rails::year(r, g, cen, k), g, lk);
    let (lane, main) = (*lk.ground.lane_dust, *lk.ground.road_gravel);
    for (c, m, lv) in road_segments(r, k) {
        let pts = drape(g, &[cen[c], cen[m]], 0.01);
        roads.ribbon(&pts, if lv == 3 { 0.034 } else { 0.022 }, if lv == 3 { main } else { lane });
    }
    Nets { roads, rails }
}
