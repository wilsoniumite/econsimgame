//! The ground of the new land: the heights, the rivers and the valley floors
//! of `world::Ground`, made from the core's and not made up
//! (docs/probes/dx-land-view.md, stage 1).
//!
//! - The heights are laid through the core's nodes at the ground's eight
//!   vertices to a cell, with none of the viewer's own coast or relief.
//! - The valley floors are the core's wet ground.
//! - The rivers are the core's lines at the core's widths, each with a bed in
//!   the heightfield no wider than the squares of the mesh its water lies in,
//!   and are drawn by today's ribbons (rivers.rs).
//! - Each lake has a sheet of water at its level, and is water to everything
//!   that asks the ground how high or how dry it is (`Laid`).
//! - The sea reads the same heights, so the tideways are water.
//!
//! Everything else (the lie of the land, roads, towns, railways) reads the
//! ground as before and takes what it finds.

use bevy::math::{Vec2, Vec3};

use super::{NewLand, WET_MARSH};
use crate::meshkit::Builder;
use crate::rivers::River;
use crate::world::VRES;

/// How many times the relief is raised when the land is drawn: a hill of
/// 1,000 m stands 1.5 cells high. It stays at three until the heights the
/// viewer's rules are written in are metres. Everything that reads a height
/// as drawn was tuned at "about three times": the field system's hill and
/// margin, the conifers, the summer snow (in terrain.wesl twice and in
/// world.rs twice), the rock, the contour lines, the frost, and every rule of
/// the railways' profile. The critic of the land-view probe measured it: at
/// four or five every one of those thresholds moves (the railways' ruling
/// gradient of 0.085 is 1 in 35 at three times and 1 in 59 at five), and at
/// five the railways' profile breaks. So it is not a setting of the look yet.
pub const RELIEF: f32 = 3.0;

/// Metres to a cell's side.
pub const CELL_M: f32 = 2000.0;

/// The smallest stream drawn, in cubic metres a second of mean flow. The
/// core keeps lines from 0.3; at 1 the viewer draws some 1,800 km of river
/// on an island, twice what the old island had, and every house plot, lane
/// and road link asks the index of water about them each year.
pub const MIN_FLOW: f32 = 1.0;

/// How far above its level a lake's sheet is drawn, in the viewer's height
/// (some 40 cm above a hill lake): enough that it never lies in the plane of
/// the ground at its lip, and little enough that a lake in the lowland,
/// where the ground beside it stands a metre over its water, is not drawn
/// wider than it is.
const LAKE_LIFT: f32 = 0.0006;

/// How far under nought a spot at a lake's very edge is said to stand to
/// whoever asks (`Laid::asked`): below the sea floor's line and every rule's
/// "dry enough".
const LAKE_UNDER: f32 = 0.01;

/// The viewer's height (1 is a cell's side) for a height in metres: the
/// shore's stretch. One function, for the ground, the rivers' levels and the
/// lakes, so that the three agree.
///
/// Today's rules read heights in the viewer's units with the old island's
/// shore in them: the sea's floor is below -0.004, a beach reaches 0.022,
/// nothing is built or grown below 0.05 (the old land began at 0.06), and the
/// sea's colour runs from clear at no depth to dark at 1.1. The new land's
/// shore is real: half of it lies under 40 m and a tideway is mostly under
/// 1.4 m deep. So the first metre or two above the tide is stretched to clear
/// 0.05, and above that the relief is raised `RELIEF` times; and under the
/// sea the depth is stretched the way the sea's shader reads it (5 m of
/// tideway is 0.17 down, the deep sea 1.2). Every threshold written for the
/// old shore then goes on meaning what it meant. Its price is a shore drawn
/// as a bank some twenty times steeper than it is; the honest answer is the
/// thresholds in metres (the design's stage 3).
pub fn units(m: f32) -> f32 {
    if m >= 0.0 {
        RELIEF * m / CELL_M + 0.055 * (1.0 - (-m / 0.7).exp())
    } else {
        let d = -m;
        -(0.02 * (1.0 - (-d / 0.5).exp()) + 0.03 * d.min(10.0) + 0.006 * (d - 10.0).max(0.0))
    }
}

/// How high a rise of `m` metres is drawn: what `units` makes of a difference
/// of heights anywhere above the first few metres over the tide, where the
/// shore's stretch has run out. A rule about how far one thing stands over or
/// under another (how deep the ground may stand over a railway before it
/// tunnels, how high a line may stand on arches) is stated in metres and read
/// through this, so that it means the same whatever `RELIEF` is; and a
/// gradient of 1 in N is `rise(CELL_M / N)` for each cell of length.
pub fn rise(m: f32) -> f32 {
    RELIEF * m / CELL_M
}

/// The height in metres that is drawn at `u`: `units` the other way.
pub fn metres(u: f32) -> f32 {
    // `units` only ever rises, so its place is found by halving
    let (mut lo, mut hi) = if u >= 0.0 { (0.0, u * CELL_M / RELIEF) } else { (u / 0.006 - 60.0, 0.0) };
    for _ in 0..48 {
        let mid = 0.5 * (lo + hi);
        if units(mid) < u {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Catmull-Rom weights for the four nodes about a spot `t` of the way from the second to the third.
fn weights(t: f32) -> [f32; 4] {
    let (t2, t3) = (t * t, t * t * t);
    [-0.5 * t3 + t2 - 0.5 * t, 1.5 * t3 - 2.5 * t2 + 1.0, -1.5 * t3 + 2.0 * t2 + 0.5 * t, 0.5 * t3 - 0.5 * t2]
}

impl NewLand {
    /// Metres above the sea at a spot in cells: a smooth surface through the
    /// nodes. A smooth surface overshoots where the ground breaks, and its
    /// overshoot downwards would dig a pit in a valley's floor between two
    /// nodes of it and drown a low shore under the hill behind. So a spot is
    /// never lower than the lowest of the nodes it lies between, and where
    /// every one of those is under the sea so is the spot: the shore runs
    /// between a dry node and a wet one and nowhere else. Upwards the surface
    /// is left to round the ridges.
    pub fn metres_at(&self, x: f32, y: f32) -> f32 {
        let (fx, fy) = (x * self.fine as f32 - 0.5, y * self.fine as f32 - 0.5);
        let (i, j) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - i, fy - j);
        let (wx, wy) = (weights(tx), weights(ty));
        let (i, j) = (i as isize, j as isize);
        let node = |a: isize, b: isize| self.height[(j + b).clamp(0, self.fh as isize - 1) as usize * self.fw + (i + a).clamp(0, self.fw as isize - 1) as usize];
        let mut sum = 0.0;
        for (b, wyb) in wy.iter().enumerate() {
            for (a, wxa) in wx.iter().enumerate() {
                sum += wxa * wyb * node(a as isize - 1, b as isize - 1);
            }
        }
        // the nodes the spot lies between: one, two or four of them
        let (ax, by) = (if tx > 1e-4 { 1 } else { 0 }, if ty > 1e-4 { 1 } else { 0 });
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for b in 0..=by {
            for a in 0..=ax {
                let h = node(a, b);
                lo = lo.min(h);
                hi = hi.max(h);
            }
        }
        let sum = sum.max(lo);
        if hi <= 0.0 { sum.min(hi) } else { sum }
    }

    /// How much a spot is of the nodes that `is` is true of, 0 to 1, read between the nodes.
    fn share_at(&self, x: f32, y: f32, is: impl Fn(usize) -> bool) -> f32 {
        let (fx, fy) = ((x * self.fine as f32 - 0.5).clamp(0.0, self.fw as f32 - 1.001), (y * self.fine as f32 - 0.5).clamp(0.0, self.fh as f32 - 1.001));
        let (i, j) = (fx.floor() as usize, fy.floor() as usize);
        let (tx, ty) = (fx - i as f32, fy - j as f32);
        let at = |i: usize, j: usize| is(j * self.fw + i) as u8 as f32;
        (at(i, j) * (1.0 - tx) + at(i + 1, j) * tx) * (1.0 - ty) + (at(i, j + 1) * (1.0 - tx) + at(i + 1, j + 1) * tx) * ty
    }

    /// The level of lake `id` (its number plus 1, as the nodes have it), in metres.
    fn lake_level(&self, id: u8) -> f32 {
        self.lakes.get(id as usize - 1).map_or(0.0, |l| l.level)
    }

    /// The level of a lake that has a node beside node `n` (on any of its
    /// eight sides), in metres: the highest, if there are two.
    pub fn lake_beside(&self, n: usize) -> Option<f32> {
        let (i, j) = ((n % self.fw) as isize, (n / self.fw) as isize);
        let mut level: Option<f32> = None;
        for dj in -1..=1 {
            for di in -1..=1 {
                let (mi, mj) = (i + di, j + dj);
                if (di, dj) == (0, 0) || mi < 0 || mj < 0 || mi as usize >= self.fw || mj as usize >= self.fh {
                    continue;
                }
                let id = self.lake[mj as usize * self.fw + mi as usize];
                if id > 0 {
                    level = Some(level.map_or(self.lake_level(id), |l| l.max(self.lake_level(id))));
                }
            }
        }
        level
    }
}

/// What the ground of the new land has besides the heights, the rivers and
/// the valley floors that every ground has (`world::Ground::laid`).
pub struct Laid {
    /// the vertex grid's size
    gw: usize,
    gh: usize,
    /// Per vertex: metres above the sea as laid through the nodes, before
    /// the rivers' beds were cut. Nothing reads it yet but the tests: it is
    /// what the thresholds will be written in when they are put into metres.
    pub metres: Vec<f32>,
    /// Per vertex: 1 on marsh, falling to 0 off it. Nothing reads it yet: the
    /// land's classes will (the design's stage 3).
    pub marsh: Vec<f32>,
    /// Per square of the vertex grid (`gh - 1` rows of `gw - 1`): 0, or the
    /// lake whose sheet of water lies over it plus 1.
    pub sheet: Vec<u8>,
    /// the height each lake's sheet is drawn at
    pub surface: Vec<f32>,
    /// per vertex: how far the water of the nearest lake is, in cells (under nought on it)
    pub lake_dist: Vec<f32>,
    /// Per cell: the record calls it water (the sea or a lake), and it holds
    /// dry ground all the same: a lake's shore, the land behind a tideway.
    pub shore_cell: Vec<bool>,
}

impl Laid {
    /// The lake whose water stands over a spot where the ground is `h` high,
    /// if one does: the height its sheet is drawn at.
    pub fn lake_over(&self, h: f32, x: f32, z: f32) -> Option<f32> {
        let i = ((x * VRES as f32).max(0.0) as usize).min(self.gw - 2);
        let j = ((z * VRES as f32).max(0.0) as usize).min(self.gh - 2);
        match self.sheet[j * (self.gw - 1) + i] {
            0 => None,
            id => Some(self.surface[id as usize - 1]).filter(|&s| h < s),
        }
    }

    /// Whether a lake's water stands over a spot where the ground is `h` high.
    #[cfg(test)]
    pub fn in_lake(&self, h: f32, x: f32, z: f32) -> bool {
        self.lake_over(h, x, z).is_some()
    }

    /// The height of a spot as it is told to whoever asks how high or how dry
    /// the ground is, given the ground's own height `h` there. On dry land it
    /// is the ground's. Under a lake's water it is the depth below the lake's
    /// surface, as a height under nought: a lake is water as the sea is, so
    /// every rule that keeps a tree, a field, a house or a works off ground
    /// that is not "dry enough" keeps it out of a lake without being told
    /// that there are lakes.
    pub fn asked(&self, h: f32, x: f32, z: f32) -> f32 {
        match self.lake_over(h, x, z) {
            Some(surface) => h - surface - LAKE_UNDER,
            None => h,
        }
    }

    /// How far a spot is from the water of the nearest lake, in cells: under
    /// nought on the water, and large where no lake is near. It is read off
    /// the vertices, so it is good to half a vertex step (some 120 m), which
    /// is as near as the sheet's own shore is known.
    pub fn lake_dist_at(&self, x: f32, z: f32) -> f32 {
        crate::world::grid_at(&self.lake_dist, self.gw, self.gh, x, z)
    }

    /// The lakes' water: a square of the vertex grid at the lake's level
    /// wherever the sheet lies, so that the shore is where the ground comes up
    /// through it. Squares side by side in a row are drawn as one.
    pub fn sheets(&self, b: &mut Builder, colour: [f32; 4]) {
        let step = 1.0 / VRES as f32;
        let sw = self.gw - 1;
        for j in 0..self.gh - 1 {
            let mut i = 0;
            while i < sw {
                let id = self.sheet[j * sw + i];
                if id == 0 {
                    i += 1;
                    continue;
                }
                let i0 = i;
                while i < sw && self.sheet[j * sw + i] == id {
                    i += 1;
                }
                let y = self.surface[id as usize - 1];
                let (x0, x1, z0, z1) = (i0 as f32 * step, i as f32 * step, j as f32 * step, (j + 1) as f32 * step);
                b.quad(Vec3::new(x0, y, z1), Vec3::new(x1, y, z1), Vec3::new(x1, y, z0), Vec3::new(x0, y, z0), colour);
            }
        }
    }
}

/// The ground made from the new land.
pub struct Made {
    /// vertex heights as drawn, row-major
    pub hgt: Vec<f32>,
    pub rivers: Vec<River>,
    /// per vertex: 1 on a valley floor (the core's flood plain and marsh), falling to 0 off it
    pub floodplain: Vec<f32>,
    pub laid: Laid,
}

/// The core's lines as the viewer's rivers: from where each first carries
/// `MIN_FLOW`, at the core's width, its water at the core's level.
pub fn rivers(l: &NewLand) -> Vec<River> {
    let mut out = Vec::new();
    for line in &l.lines {
        let Some(from) = line.flow.iter().position(|&q| q >= MIN_FLOW) else { continue };
        if line.pts.len() - from < 2 {
            continue;
        }
        let pts: Vec<Vec2> = line.pts[from..].to_vec();
        let half: Vec<f32> = line.width[from..].iter().map(|&m| 0.5 * m / CELL_M).collect();
        let mut level: Vec<f32> = line.level[from..].iter().map(|&m| units(m)).collect();
        // (the core's levels never rise downstream; the stretch keeps that, and this holds it)
        for i in 1..level.len() {
            level[i] = level[i].min(level[i - 1]);
        }
        let len: f32 = pts.windows(2).map(|p| p[0].distance(p[1])).sum();
        out.push(River { swing: vec![0.0; pts.len()], pts, half, level, mouth: line.end == super::End::Sea, coast: len });
    }
    out
}

/// A bed for each river: every vertex of the squares of the mesh that the
/// water at a point lies in is brought down to just under the water there.
/// The core cut the ground to the water at the node a line passes through
/// and nowhere between, so a ribbon laid at the river's level on a surface
/// through the nodes would be buried along a fifth of its length. The bed is
/// as narrow as the mesh can make it: one or two squares, 250 to 500 m.
fn bed(rivers: &[River], hgt: &mut [f32], gw: usize, gh: usize) {
    let res = VRES as f32;
    for rv in rivers {
        for (i, p) in rv.pts.iter().enumerate() {
            // the water and its wet bank (rivers::ribbons draws 0.011 of bank a side)
            let reach = rv.half[i] + 0.012;
            let (x0, x1) = (((p.x - reach) * res).floor().max(0.0) as usize, (((p.x + reach) * res).floor() as usize + 1).min(gw - 1));
            let (y0, y1) = (((p.y - reach) * res).floor().max(0.0) as usize, (((p.y + reach) * res).floor() as usize + 1).min(gh - 1));
            let floor = rv.level[i] + 0.003;
            for gy in y0..=y1 {
                for gx in x0..=x1 {
                    let k = gy * gw + gx;
                    if floor < hgt[k] {
                        hgt[k] = floor;
                    }
                }
            }
        }
    }
}

/// Which lake's sheet lies over each square of the vertex grid. A sheet lies
/// over the nodes of its lake, and over the half of each node beside them
/// that faces the lake, if that node stands no lower than the water: the
/// shore is then where the ground comes up through the sheet, between a node
/// of the lake and the dry node beside it. A node beside the lake that is
/// lower than its water is where the lake's river leaves it, and the sheet
/// stops short of it, or it would hang over the valley below.
fn sheets(l: &NewLand, gw: usize, gh: usize) -> Vec<u8> {
    let (sw, sh) = (gw - 1, gh - 1);
    let fine = l.fine as f32;
    let mut out = vec![0u8; sw * sh];
    for j in 0..sh {
        for i in 0..sw {
            // the square's middle, and the node it lies in
            let (x, y) = ((i as f32 + 0.5) / VRES as f32, (j as f32 + 0.5) / VRES as f32);
            let (ni, nj) = (((x * fine) as usize).min(l.fw - 1), ((y * fine) as usize).min(l.fh - 1));
            let n = nj * l.fw + ni;
            if l.lake[n] > 0 {
                out[j * sw + i] = l.lake[n];
                continue;
            }
            // the nodes this part of the node faces: across its side, across its end, across its corner
            let (sx, sy) = (if x * fine - ni as f32 >= 0.5 { 1isize } else { -1 }, if y * fine - nj as f32 >= 0.5 { 1isize } else { -1 });
            for (dx, dy) in [(sx, 0), (0, sy), (sx, sy)] {
                let (mi, mj) = (ni as isize + dx, nj as isize + dy);
                if mi < 0 || mj < 0 || mi as usize >= l.fw || mj as usize >= l.fh {
                    continue;
                }
                let id = l.lake[mj as usize * l.fw + mi as usize];
                if id > 0 && l.height[n] >= l.lake_level(id) - 0.5 {
                    out[j * sw + i] = id;
                }
            }
        }
    }
    out
}

/// How far each cell of a grid is from the nearest cell that `seed` is true
/// of, in steps: straight steps count 1 and diagonal ones the square root of 2.
fn chamfer(w: usize, h: usize, seed: &[bool]) -> Vec<f32> {
    let big = 1e9f32;
    let mut d: Vec<f32> = seed.iter().map(|&s| if s { 0.0 } else { big }).collect();
    let r2 = std::f32::consts::SQRT_2;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let mut m = d[i];
            if x > 0 {
                m = m.min(d[i - 1] + 1.0);
            }
            if y > 0 {
                m = m.min(d[i - w] + 1.0);
                if x > 0 {
                    m = m.min(d[i - w - 1] + r2);
                }
                if x + 1 < w {
                    m = m.min(d[i - w + 1] + r2);
                }
            }
            d[i] = m;
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            let mut m = d[i];
            if x + 1 < w {
                m = m.min(d[i + 1] + 1.0);
            }
            if y + 1 < h {
                m = m.min(d[i + w] + 1.0);
                if x + 1 < w {
                    m = m.min(d[i + w + 1] + r2);
                }
                if x > 0 {
                    m = m.min(d[i + w - 1] + r2);
                }
            }
            d[i] = m;
        }
    }
    d
}

/// The heights, the rivers, the valley floors and the lakes from the core's
/// land. `terrain` is the record's class of each cell (0 for water).
pub fn ground(l: &NewLand, terrain: &[u8]) -> Made {
    use rayon::prelude::*;
    let (gw, gh) = (l.w * VRES + 1, l.h * VRES + 1);
    let res = VRES as f32;
    let at = |k: usize| ((k % gw) as f32 / res, (k / gw) as f32 / res);
    let t0 = std::time::Instant::now();
    let metres: Vec<f32> = (0..gw * gh).into_par_iter().map(|k| l.metres_at(at(k).0, at(k).1)).collect();
    let floodplain: Vec<f32> = (0..gw * gh).into_par_iter().map(|k| l.share_at(at(k).0, at(k).1, |n| l.wet[n] > 0)).collect();
    let marsh: Vec<f32> = (0..gw * gh).into_par_iter().map(|k| l.share_at(at(k).0, at(k).1, |n| l.wet[n] == WET_MARSH)).collect();
    let mut hgt: Vec<f32> = metres.iter().map(|&m| units(m)).collect();
    let t_lay = t0.elapsed();

    let t1 = std::time::Instant::now();
    let rivers = rivers(l);
    bed(&rivers, &mut hgt, gw, gh);
    let t_bed = t1.elapsed();

    // the lakes: where each sheet lies and how high, and how far every vertex is from one's water
    let t2 = std::time::Instant::now();
    let sheet = sheets(l, gw, gh);
    let surface: Vec<f32> = l.lakes.iter().map(|lake| units(lake.level) + LAKE_LIFT).collect();
    let (sw, sh) = (gw - 1, gh - 1);
    let under: Vec<bool> = (0..gw * gh)
        .map(|k| {
            // a vertex is under a lake if a sheet lies over a square it is a corner of and stands above it
            let (i, j) = (k % gw, k / gw);
            let mut wet = false;
            for (di, dj) in [(0usize, 0usize), (1, 0), (0, 1), (1, 1)] {
                if i >= di && j >= dj && i - di < sw && j - dj < sh {
                    let id = sheet[(j - dj) * sw + i - di];
                    wet |= id > 0 && hgt[k] < surface[id as usize - 1];
                }
            }
            wet
        })
        .collect();
    // (a vertex on the water counts as half a step inside it)
    let lake_dist: Vec<f32> = chamfer(gw, gh, &under).iter().map(|&d| (d - 0.5) / res).collect();
    let t_lakes = t2.elapsed();

    // the cells the record calls water that hold dry ground: a tenth of the cell's vertices or more
    // stand above the building line and out of any lake
    let shore_cell: Vec<bool> = (0..l.w * l.h)
        .map(|c| {
            if terrain[c] != 0 {
                return false;
            }
            let (cx, cy) = (c % l.w, c / l.w);
            let mut dry = 0;
            for j in 0..VRES {
                for i in 0..VRES {
                    let k = (cy * VRES + j) * gw + cx * VRES + i;
                    dry += (hgt[k] >= 0.05 && !under[k]) as usize;
                }
            }
            dry * 10 >= VRES * VRES
        })
        .collect();

    if std::env::var_os("ISOSTATE_TIMING").is_some() {
        let points: usize = rivers.iter().map(|r| r.pts.len()).sum();
        let km: f32 = rivers.iter().map(|rv| rv.pts.windows(2).map(|p| p[0].distance(p[1])).sum::<f32>()).sum::<f32>() * CELL_M / 1000.0;
        eprintln!(
            "timing the new land's ground: {gw} by {gh} vertices laid through {} by {} nodes in {:.1} ms; {} rivers of {points} points and {km:.0} km bedded in {:.1} ms; {} lakes under {} squares of sheet, and the distance to them, in {:.1} ms; {} cells of water hold dry shore",
            l.fw,
            l.fh,
            t_lay.as_secs_f64() * 1e3,
            rivers.len(),
            t_bed.as_secs_f64() * 1e3,
            l.lakes.len(),
            sheet.iter().filter(|&&s| s > 0).count(),
            t_lakes.as_secs_f64() * 1e3,
            shore_cell.iter().filter(|&&s| s).count()
        );
    }
    Made { hgt, rivers, floodplain, laid: Laid { gw, gh, metres, marsh, sheet, surface, lake_dist, shore_cell } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stretch_keeps_the_old_shore_s_thresholds_and_goes_back_again() {
        // the tide is nought, the first metres clear the building line, and the hills are raised three times
        assert_eq!(units(0.0), 0.0);
        assert!(units(0.3) < 0.022 && units(0.5) > 0.022, "the beach is the first third of a metre or so: {} and {}", units(0.3), units(0.5));
        assert!(units(1.4) < 0.05 && units(1.5) > 0.05, "nothing is built on the first metre and a half: {} and {}", units(1.4), units(1.5));
        assert!((units(1000.0) - (1.5 + 0.055)).abs() < 1e-4, "{}", units(1000.0));
        assert!((units(600.0) - units(400.0) - RELIEF * 200.0 / CELL_M).abs() < 1e-5);
        // and a rise is drawn the same height wherever it stands, from a few metres over the tide
        for base in [8.0f32, 40.0, 500.0] {
            assert!((units(base + 67.0) - units(base) - rise(67.0)).abs() < 1e-4, "a rise of 67 m from {base} m");
        }
        // under the sea: a tideway of a metre is under the sea floor's line, 5 m is 0.17 down, the deep 1.2
        assert!(units(-0.2) < -0.004 && units(-1.0) < -0.04);
        assert!((units(-5.0) + 0.17).abs() < 0.001 && (units(-165.0) + 1.25).abs() < 0.001, "{} and {}", units(-5.0), units(-165.0));
        // it only ever rises, and `metres` undoes it
        let mut last = units(-200.0);
        for i in -1999..=10000 {
            let m = i as f32 * 0.1;
            let u = units(m);
            assert!(u > last, "the stretch falls at {m} m");
            last = u;
            let back = metres(u);
            assert!((back - m).abs() < 0.02 + 2e-4 * m.abs(), "{m} m is drawn at {u} and read back as {back} m");
        }
    }
}
