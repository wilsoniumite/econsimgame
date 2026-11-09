//! Rivers, roads and railways as ribbons draped over the terrain.

use bevy::math::Vec3;

use crate::meshkit::{hex, Builder};
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

fn quad_curve(a: (f32, f32), c: (f32, f32), b: (f32, f32), n: usize) -> Vec<(f32, f32)> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            (u * u * a.0 + 2.0 * u * t * c.0 + t * t * b.0, u * u * a.1 + 2.0 * u * t * c.1 + t * t * b.1)
        })
        .collect()
}

pub fn rivers(r: &Replay, g: &Ground, cen: &[(f32, f32)]) -> Builder {
    let n = r.n;
    let mut best = vec![0u8; n];
    let mut upm = vec![usize::MAX; n];
    for c in 0..n {
        let d = r.down[c];
        if r.river[c] == 0 || d < 0 || d as usize == c {
            continue;
        }
        let d = d as usize;
        if r.river[c] > best[d] {
            best[d] = r.river[c];
            upm[d] = c;
        }
    }
    let mid = |a: usize, b: usize| ((cen[a].0 + cen[b].0) * 0.5, (cen[a].1 + cen[b].1) * 0.5);
    let water = hex(0x5ea9d3);
    let mut b = Builder::default();
    for c in 0..n {
        if r.river[c] == 0 || r.terrain[c] == 0 {
            continue;
        }
        let d = r.down[c];
        let has_d = d >= 0 && d as usize != c;
        let s = if upm[c] != usize::MAX { mid(upm[c], c) } else { cen[c] };
        let e = if has_d {
            let d = d as usize;
            if r.terrain[d] > 0 { mid(c, d) } else { (cen[d].0 * 0.7 + cen[c].0 * 0.3, cen[d].1 * 0.7 + cen[c].1 * 0.3) }
        } else {
            cen[c]
        };
        let class = ((r.river[c] as f32 - 105.0) / 28.0).floor().clamp(0.0, 4.0) as usize;
        let width = [0.03, 0.045, 0.065, 0.085, 0.11][class];
        let mut pts = quad_curve(s, cen[c], e, 6);
        if has_d {
            let d = d as usize;
            if r.terrain[d] > 0 && r.river[d] > 0 && upm[d] != c {
                pts.push(cen[d]); // a tributary joins the main stream
            }
        }
        b.ribbon(&drape(g, &pts, 0.012), width, water);
    }
    b
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

pub fn year_nets(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize) -> Nets {
    let (w, h, n) = (r.w, r.h, r.n);
    let tr = &r.trans[k * n..(k + 1) * n];
    let bit = |x: isize, y: isize, m: u8| -> bool {
        x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && tr[y as usize * w + x as usize] & m != 0
    };
    let mut roads = Builder::default();
    let mut rails = Builder::default();
    let (lane, main) = (hex(0xcdb88d), hex(0xd9c596));
    let (ballast, iron, build) = (hex(0x8d877f), hex(0x3d3633), hex(0xa3967f));
    for c in 0..n {
        let v = tr[c];
        if v == 0 {
            continue;
        }
        let (x, y) = ((c % w) as isize, (c / w) as isize);
        for (dx, dy) in [(1isize, 0isize), (0, 1), (1, 1), (-1, 1)] {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || nx >= w as isize || ny >= h as isize {
                continue;
            }
            let m = ny as usize * w + nx as usize;
            let u = tr[m];
            if u == 0 {
                continue;
            }
            let diag = dx != 0 && dy != 0;
            let skip = |mask: u8| diag && (bit(x + dx, y, mask) || bit(x, y + dy, mask));
            let seg = [cen[c], cen[m]];
            if v & 4 != 0 && u & 4 != 0 && !skip(4) {
                let pts = drape(g, &seg, 0.016);
                rails.ribbon(&pts, 0.04, ballast);
                let top: Vec<Vec3> = pts.iter().map(|p| *p + Vec3::Y * 0.004).collect();
                rails.ribbon(&top, 0.016, iron);
            } else if ((v & 8 != 0) && (u & 12 != 0)) || ((u & 8 != 0) && (v & 12 != 0)) {
                if !skip(12) {
                    let pts = drape(g, &seg, 0.014);
                    // works in progress: a dashed line of earthworks
                    for (i, pair) in pts.windows(2).enumerate() {
                        if i % 2 == 0 {
                            rails.ribbon(pair, 0.03, build);
                        }
                    }
                }
            }
        }
    }
    for (c, m, lv) in road_segments(r, k) {
        let pts = drape(g, &[cen[c], cen[m]], 0.01);
        roads.ribbon(&pts, if lv == 3 { 0.034 } else { 0.022 }, if lv == 3 { main } else { lane });
    }
    Nets { roads, rails }
}
