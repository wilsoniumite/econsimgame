//! Rivers, roads and railways as ribbons over the terrain. The rivers are
//! made in rivers.rs and the railways in rails.rs; the roads are draped here.
//!
//! A road between two cells is drawn as one straight line in plan, between
//! the two cells' middles (`centres`), and everything that goes with it is
//! read from here: the ribbon on the ground (`road_line`), the bridges where
//! it crosses a river (`road_bridges`), the causeway where it runs over
//! drowned ground (`causeway_runs`), and the surface a cart rides along it,
//! which is whichever of those is on top (`driven`).

use bevy::math::{Vec2, Vec3};
use isostate_gen::bridge::RoadBridge;
use isostate_gen::houses::Detail;

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

/// How far above the ground a road's ribbon lies (never below the sea's surface).
pub const ROAD_LIFT: f32 = 0.01;
/// The top of a causeway, over drowned ground.
pub const CAUSEWAY_TOP: f32 = 0.016;

/// How wide a road's ribbon is: a main road (level 3) or a lesser one.
pub fn road_width(lv: u8) -> f32 {
    if lv == 3 { 0.034 } else { 0.022 }
}

/// The middle line of the ribbon of the road from `a` to `b` (two cells'
/// middles), as it is drawn: straight in plan, and at the ground's height
/// plus `ROAD_LIFT` every 0.08 cells or less, straight from one point to the next.
pub fn road_line(g: &Ground, a: (f32, f32), b: (f32, f32)) -> Vec<Vec3> {
    drape(g, &[a, b], ROAD_LIFT)
}

/// The bridges on the road from `a` to `b`: one wherever it crosses a
/// river's middle line, as long as the river is wide where the road crosses
/// it, each built at the level of detail `detail` gives for its place.
pub fn road_bridges(g: &Ground, a: Vec2, b: Vec2, main: bool, detail: impl Fn(Vec2) -> Detail) -> Vec<RoadBridge> {
    let along = (b - a).normalize_or(Vec2::X);
    crate::rivers::crossings(&g.rivers, &g.water, a, b)
        .into_iter()
        .map(|(at, river, half, level)| {
            // a road that crosses aslant has further to go from bank to bank (up to three times as far)
            let half = half / along.perp_dot(river).abs().max(0.34);
            RoadBridge { at, along, half, level, main, detail: detail(at) }
        })
        .collect()
}

/// Where the road from `a` to `c` runs over drowned ground (the mouth of a
/// river, a creek) and a causeway carries it at `CAUSEWAY_TOP`: each stretch
/// as how far along the road it starts and ends (0 at `a`, 1 at `c`), from
/// one dry end to the other and a little past each.
pub fn causeway_runs(g: &Ground, a: Vec2, c: Vec2) -> Vec<(f32, f32)> {
    let len = a.distance(c);
    let n = ((len / 0.02).ceil() as usize).max(2);
    let wet = |t: f32| {
        let p = a + (c - a) * t;
        world::height_at(g, p.x, p.y) < 0.004
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        if !wet(i as f32 / n as f32) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < n && wet(j as f32 / n as f32) {
            j += 1;
        }
        out.push((((i as f32 - 1.0) / n as f32).max(0.0), ((j as f32 + 0.5) / n as f32).min(1.0)));
        i = j;
    }
    out
}

/// The road from `a` to `b` as a cart drives it: its ribbon's middle line,
/// at the height of whatever is on top there, the ribbon itself, a bridge's
/// roadway (up its ramp, over the hump and down again) or a causeway. Each
/// is straight from one of its points to the next, so the surface is too,
/// and the points returned are every place where one of them bends or one
/// rises above another. The bridges are read at the play distance's detail,
/// whose roadway is the same as the near one's; from far off a bridge's
/// roadway is drawn in fewer lengths, and the cart rides a shade above it.
pub fn driven(g: &Ground, a: (f32, f32), b: (f32, f32), main: bool) -> Vec<Vec3> {
    let ribbon = road_line(g, a, b);
    let (pa, pb) = (Vec2::new(a.0, a.1), Vec2::new(b.0, b.1));
    let len = pa.distance(pb);
    if len < 1e-6 {
        return ribbon;
    }
    let dir = (pb - pa) / len;
    let t_of = |p: Vec2| (p - pa).dot(dir) / len;
    let mut layers: Vec<Vec<(f32, f32)>> = vec![ribbon.iter().map(|p| (t_of(Vec2::new(p.x, p.z)), p.y)).collect()];
    let height = |x: f32, z: f32| world::height_at(g, x, z);
    for br in road_bridges(g, pa, pb, main, |_| Detail::Play) {
        layers.push(br.roadway(&height).into_iter().map(|(t, h)| (t_of(br.point(t)), h)).collect());
    }
    for (t0, t1) in causeway_runs(g, pa, pb) {
        layers.push(vec![(t0, CAUSEWAY_TOP), (t1, CAUSEWAY_TOP)]);
    }
    envelope(&layers).into_iter().map(|(t, h)| {
        let p = pa + dir * (len * t);
        Vec3::new(p.x, h, p.y)
    }).collect()
}

/// The top of several surfaces along a road, each given as heights at points
/// along it (0 to 1) and straight between them, and nothing outside its first
/// and last: the height of the highest at every point where one of them
/// bends, begins or ends, or two of them cross, from 0 to 1. Where one begins
/// or ends standing above the rest (the foot of a bridge's ramp where the
/// river's level stands above the bank), the top steps there, within `STEP`
/// of the way along, rather than sloping across what lies below.
fn envelope(layers: &[Vec<(f32, f32)>]) -> Vec<(f32, f32)> {
    const EPS: f32 = 1e-6;
    const STEP: f32 = 1e-4;
    let at = |l: &[(f32, f32)], t: f32| -> Option<f32> {
        let (first, last) = (l.first()?, l.last()?);
        if t < first.0 - EPS || t > last.0 + EPS {
            return None;
        }
        if l.len() == 1 {
            return Some(first.1);
        }
        let i = l.partition_point(|p| p.0 < t).clamp(1, l.len() - 1);
        let ((t0, h0), (t1, h1)) = (l[i - 1], l[i]);
        Some(if t1 - t0 > EPS { h0 + (h1 - h0) * ((t - t0) / (t1 - t0)).clamp(0.0, 1.0) } else { h1 })
    };
    // (and just outside where each begins and ends, for the step)
    let edges = layers.iter().filter_map(|l| Some([l.first()?.0 - STEP, l.last()?.0 + STEP])).flatten();
    let mut ts: Vec<f32> = layers.iter().flatten().map(|p| p.0).chain(edges).filter(|t| (0.0..=1.0).contains(t)).collect();
    ts.extend([0.0, 1.0]);
    ts.sort_by(f32::total_cmp);
    ts.dedup_by(|x, y| (*x - *y).abs() < EPS);
    // where two surfaces cross between two of those points, the top bends there too
    let mut more = Vec::new();
    for w in ts.windows(2) {
        let (t0, t1) = (w[0], w[1]);
        let ends: Vec<(f32, f32)> = layers.iter().filter_map(|l| Some((at(l, t0)?, at(l, t1)?))).collect();
        for i in 0..ends.len() {
            for j in i + 1..ends.len() {
                let (d0, d1) = (ends[i].0 - ends[j].0, ends[i].1 - ends[j].1);
                if d0 * d1 < 0.0 {
                    more.push(t0 + (t1 - t0) * d0 / (d0 - d1));
                }
            }
        }
    }
    ts.extend(more);
    ts.sort_by(f32::total_cmp);
    ts.dedup_by(|x, y| (*x - *y).abs() < EPS);
    ts.into_iter().filter_map(|t| Some((t, layers.iter().filter_map(|l| at(l, t)).reduce(f32::max)?))).collect()
}

/// The height of a line of points at `t` of the way from its first to its
/// last, measured in plan along the straight line between them (as for a
/// road's line from `driven`).
pub fn height_along(pts: &[Vec3], t: f32) -> f32 {
    let (a, b) = (Vec2::new(pts[0].x, pts[0].z), Vec2::new(pts[pts.len() - 1].x, pts[pts.len() - 1].z));
    let (dir, len) = ((b - a).normalize_or(Vec2::X), a.distance(b).max(1e-6));
    let ts: Vec<f32> = pts.iter().map(|p| (Vec2::new(p.x, p.z) - a).dot(dir) / len).collect();
    let i = ts.partition_point(|&s| s < t).clamp(1, pts.len() - 1);
    let (t0, t1) = (ts[i - 1], ts[i]);
    let u = if t1 - t0 > 1e-6 { ((t - t0) / (t1 - t0)).clamp(0.0, 1.0) } else { 1.0 };
    pts[i - 1].y + (pts[i].y - pts[i - 1].y) * u
}

/// The water that shows on the map, for asking where a road meets it: the
/// sea, and the rivers' water as `rivers::ribbons` draws it, bucketed by
/// cell (each length of a river whose two ends both lie above the sea, with
/// its half width and its surface at either end).
pub struct Waters {
    w: usize,
    h: usize,
    cells: std::collections::HashMap<usize, Vec<(Vec2, Vec2, f32, f32, f32, f32)>>,
}

impl Waters {
    pub fn new(r: &Replay, g: &Ground) -> Waters {
        let mut cells: std::collections::HashMap<usize, Vec<(Vec2, Vec2, f32, f32, f32, f32)>> = Default::default();
        for rv in &g.rivers {
            for i in 1..rv.pts.len() {
                // drawn where the level stands above the sea, its surface 0.009 over the level
                if rv.level[i - 1] <= 0.004 || rv.level[i] <= 0.004 {
                    continue;
                }
                let seg = (rv.pts[i - 1], rv.pts[i], rv.half[i - 1], rv.half[i], rv.level[i - 1].max(0.0) + 0.009, rv.level[i].max(0.0) + 0.009);
                let reach = rv.half[i - 1].max(rv.half[i]) + 0.02;
                let (lo, hi) = (rv.pts[i - 1].min(rv.pts[i]) - Vec2::splat(reach), rv.pts[i - 1].max(rv.pts[i]) + Vec2::splat(reach));
                for y in (lo.y.floor().max(0.0) as usize)..=(hi.y.floor().max(0.0) as usize).min(r.h - 1) {
                    for x in (lo.x.floor().max(0.0) as usize)..=(hi.x.floor().max(0.0) as usize).min(r.w - 1) {
                        cells.entry(y * r.w + x).or_default().push(seg);
                    }
                }
            }
        }
        Waters { w: r.w, h: r.h, cells }
    }

    /// The water that shows at a spot, if any: its surface's height, and
    /// whether it is the sea. A river's water is drawn flat across its width,
    /// and where the bank stands higher than it the ground hides it, so it
    /// counts only where it lies above the ground.
    pub fn at(&self, g: &Ground, p: Vec2) -> Option<(f32, bool)> {
        let ground = world::height_at(g, p.x, p.y);
        if ground < 0.0 {
            return Some((0.0, true));
        }
        let (x, y) = (p.x.floor().clamp(0.0, self.w as f32 - 1.0) as usize, p.y.floor().clamp(0.0, self.h as f32 - 1.0) as usize);
        let mut top: Option<f32> = None;
        for &(a, b, ha, hb, la, lb) in self.cells.get(&(y * self.w + x)).into_iter().flatten() {
            let ab = b - a;
            let u = ((p - a).dot(ab) / ab.length_squared().max(1e-9)).clamp(0.0, 1.0);
            if p.distance(a + ab * u) < ha + (hb - ha) * u {
                let level = la + (lb - la) * u;
                top = Some(top.map_or(level, |t: f32| t.max(level)));
            }
        }
        top.filter(|&t| t > ground).map(|t| (t, false))
    }

    /// Whether any river's water lies near the straight road from `a` to `b`
    /// (in the cells under it: those its two ends lie in and, on a diagonal,
    /// the two beside them). Only a river can drown a road, as the ribbon lies
    /// at least `ROAD_LIFT` above the sea, and most roads have none near.
    pub fn near(&self, a: Vec2, b: Vec2) -> bool {
        let (lo, hi) = (a.min(b), b.max(a));
        (lo.y.floor().max(0.0) as usize..=(hi.y.floor().max(0.0) as usize).min(self.h - 1))
            .any(|y| (lo.x.floor().max(0.0) as usize..=(hi.x.floor().max(0.0) as usize).min(self.w - 1)).any(|x| self.cells.contains_key(&(y * self.w + x))))
    }

    /// Whether a road's line from `driven` goes under the water anywhere:
    /// where it runs along a river's channel inside its water, with no bridge
    /// or causeway to carry it, the ribbon lies on the channel's bed. Looked
    /// at every 0.01 cells.
    pub fn drowns(&self, g: &Ground, pts: &[Vec3]) -> bool {
        let (a, b) = (Vec2::new(pts[0].x, pts[0].z), Vec2::new(pts[pts.len() - 1].x, pts[pts.len() - 1].z));
        if !self.near(a, b) {
            return false;
        }
        let n = ((a.distance(b) / 0.01).ceil() as usize).max(1);
        (0..=n).any(|i| {
            let t = i as f32 / n as f32;
            self.at(g, a + (b - a) * t).is_some_and(|(top, _)| height_along(pts, t) < top)
        })
    }
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
        roads.ribbon(&road_line(g, cen[c], cen[m]), road_width(lv), if lv == 3 { main } else { lane });
    }
    Nets { roads, rails }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_top_of_a_road_steps_where_a_ramp_ends_above_it_and_bends_where_two_cross() {
        // a road falling from 0.1 to 0 along its length; a ramp from 0.2 to 0.4 that begins 0.01
        // above it and rises to 0.12; a deck from 0.4 to 0.6 at 0.05, under the road at first
        let road = vec![(0.0, 0.1), (1.0, 0.0)];
        let ramp = vec![(0.2, 0.09), (0.4, 0.12)];
        let deck = vec![(0.4, 0.05), (0.6, 0.05)];
        let top = envelope(&[road, ramp, deck]);
        let at = |t: f32| {
            let i = top.partition_point(|p| p.0 < t).clamp(1, top.len() - 1);
            let ((t0, h0), (t1, h1)) = (top[i - 1], top[i]);
            h0 + (h1 - h0) * ((t - t0) / (t1 - t0)).clamp(0.0, 1.0)
        };
        // the road, and up onto the ramp at once where it begins, not sloping up to it
        assert!((at(0.1) - 0.09).abs() < 1e-5 && (at(0.199) - 0.0801).abs() < 1e-4, "{top:?}");
        assert!((at(0.2) - 0.09).abs() < 1e-5 && (at(0.3) - 0.105).abs() < 1e-5, "{top:?}");
        // down to the road at once where the ramp ends, the deck being under the road there
        assert!((at(0.401) - 0.0599).abs() < 1e-4 && (at(0.45) - 0.055).abs() < 1e-5, "{top:?}");
        // the road goes under the deck at 0.5, and the top bends there
        assert!(top.iter().any(|p| (p.0 - 0.5).abs() < 1e-5), "{top:?}");
        assert!((at(0.55) - 0.05).abs() < 1e-5 && (at(0.7) - 0.03).abs() < 1e-5, "{top:?}");
    }
}
