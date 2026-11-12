//! Rivers, roads and railways as ribbons over the terrain. The rivers are
//! made in rivers.rs and the railways in rails.rs; the roads are draped here.

use bevy::math::Vec3;

use crate::look::Look;
use crate::meshkit::Builder;
use crate::replay::Replay;
use crate::world::{self, Ground};

/// Cell centres with a little jitter, shared by every line on the map: where
/// a cell's roads meet, its town has its market square and its railway
/// passes. On the new land a land cell's is moved onto the nearest dry and
/// level spot of the cell (newland/middles.rs): the water there is where the
/// core put it, and the old island's middle is as often as not in it. On the
/// old island they are where they always were.
pub fn centres(r: &Replay) -> Vec<(f32, f32)> {
    if let Some(land) = &r.land {
        return land.middles(&r.terrain).to_vec();
    }
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

/// The rivers: water and wet banks, lying on each river's own level (see
/// rivers.rs). On the new land the lakes are with them, each a sheet of the
/// rivers' water at its level (newland/ground.rs), so that a lake goes to
/// steel in the depth of winter with the rivers that feed it.
pub fn rivers(g: &Ground, lk: &Look) -> Builder {
    let mut b = crate::rivers::ribbons(&g.rivers, lk);
    if let Some(laid) = &g.laid {
        laid.sheets(&mut b, *lk.water.river);
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

/// Where the railway has sidings in year `k`: two at a big town's station,
/// one at a middling town's, and one in a cell where three works or more
/// stand by the line.
fn sidings(r: &Replay, cen: &[(f32, f32)], k: usize) -> Vec<crate::rails::Sidings> {
    let railed = |c: usize| r.trans[k * r.n + c] & 4 != 0;
    let middle = |c: usize| bevy::math::Vec2::new(cen[c].0, cen[c].1);
    let mut out: Vec<crate::rails::Sidings> = Vec::new();
    for t in 0..r.n_towns.min(r.header.towns.len()) {
        let node = r.header.towns[t].node;
        let urban = r.town_urban[k * r.n_towns + t];
        if r.town_active[k * r.n_towns + t] != 0 && railed(node) && urban > 3000.0 {
            out.push((node, if urban > 12000.0 { 2 } else { 1 }, middle(node)));
        }
    }
    let mut works: std::collections::HashMap<usize, usize> = Default::default();
    for (f, firm) in r.header.firms.iter().enumerate() {
        let (first, len, off) = (r.firm_first[f].max(0) as usize, r.firm_len[f].max(0) as usize, r.firm_off[f].max(0) as usize);
        if railed(firm.cell) && k >= first && k < first + len && r.firm_k[off + k - first] > 0.0 {
            *works.entry(firm.cell).or_default() += 1;
        }
    }
    let mut districts: Vec<usize> = works.into_iter().filter(|&(c, n)| n >= 3 && !out.iter().any(|s| s.0 == c)).map(|(c, _)| c).collect();
    districts.sort_unstable();
    out.extend(districts.into_iter().map(|c| (c, 1, middle(c))));
    out
}

pub fn year_nets(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, lk: &Look) -> Nets {
    world::timed("year nets", || year_nets_inner(r, g, cen, k, lk))
}

fn year_nets_inner(r: &Replay, g: &Ground, cen: &[(f32, f32)], k: usize, lk: &Look) -> Nets {
    let mut roads = Builder::default();
    // the railways are graded curves with their own works (rails.rs)
    let rails = crate::rails::track(&crate::rails::year(r, g, cen, k), g, lk, &sidings(r, cen, k));
    let (lane, main) = (*lk.ground.lane_dust, *lk.ground.road_gravel);
    for (c, m, lv) in road_segments(r, k) {
        let pts = drape(g, &[cen[c], cen[m]], 0.01);
        roads.ribbon(&pts, if lv == 3 { 0.034 } else { 0.022 }, if lv == 3 { main } else { lane });
    }
    Nets { roads, rails }
}
