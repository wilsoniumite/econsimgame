//! Where each cell's roads meet on the new land (`network::centres`).
//!
//! On the old island a cell's meeting place is its middle with a little
//! jitter, and the viewer makes its rivers run beside it. On the new land the
//! water is where the core put it: a fifth of the first towns had their
//! market square on tidewater or in a lake, and another sixth on marsh. So
//! each land cell's meeting place is moved onto the nearest dry and level
//! spot of its own cell: off tidewater, lakes and marsh, back from a river,
//! and off the steepest ground where the cell has gentler. Everything that
//! hangs on the middles (the market square, the church's quarter, the high
//! streets, the roads, the lanes, the railway's passing point) follows.
//!
//! Which bank of a river a place takes, and keeping to one bank down a reach,
//! wait for the roads' stage. A cell that is all water or all marsh keeps
//! the best there is of it.

use bevy::math::Vec2;

use super::ground::{CELL_M, MIN_FLOW};
use super::{NewLand, WET_MARSH};
use crate::world::hash2;

/// Spots tried along each side of a cell.
const TRY: usize = 13;
/// How far from the cell's edge the spots tried keep, in cells: two cells'
/// meeting places are then never closer than twice this, and a road between
/// them has some length.
const INSET: f32 = 0.14;
/// How high a place must stand to be built on, in metres: over the first
/// metre and a half above the tide, which the shore's stretch draws below
/// the line nothing is built under (`ground::units`: 0.05 is 1.43 m).
const DRY_M: f32 = 1.8;
/// The height the tide's beach reaches, in metres (0.022 as drawn).
const BEACH_M: f32 = 0.4;
/// How far round a place the ground must be dry and firm, in cells: a small
/// town's market square (`scatter/towns.rs`, 0.055) and a little more.
const ROUND: f32 = 0.07;
/// How far back from a river's water a place keeps, in cells (some 160 m).
const BACK: f32 = 0.08;
/// A slope that is level enough for anybody, as the core's heights have it (1 in 20).
const LEVEL: f32 = 0.05;
/// Cells of the grid the rivers are marked on, to a map cell's side.
const MARK: usize = 16;

/// The old island's meeting place for cell `c`: its middle with a little jitter.
pub fn jittered(c: usize, w: usize) -> (f32, f32) {
    let (x, y) = ((c % w) as f32, (c / w) as f32);
    (x + 0.5 + 0.36 * (hash2(c as i32, 11, 1) - 0.5), y + 0.5 + 0.36 * (hash2(c as i32, 13, 2) - 0.5))
}

/// What is wrong with a spot as a meeting place, the worst first, and how
/// steep it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Footing {
    /// the spot is under tidewater or a lake
    pub water: bool,
    /// the spot is dry and stands under `DRY_M`, where nothing is built
    pub low: bool,
    /// water close round the spot (within `ROUND`)
    pub shore: bool,
    /// marsh at the spot or close round it, or ground too low to build on close round it
    pub marsh: bool,
    /// a river's water within `BACK`
    pub river: bool,
    /// the ground's slope there, as the core's heights have it
    pub slope: f32,
}

impl Footing {
    /// How bad, in kind: 0 is dry, firm ground back from the river.
    pub fn tier(&self) -> u8 {
        if self.water {
            5
        } else if self.low {
            4
        } else if self.shore {
            3
        } else if self.marsh {
            2
        } else if self.river {
            1
        } else {
            0
        }
    }

    /// The same in a word or two, for a report.
    pub fn words(&self) -> &'static str {
        ["dry ground", "a river's bank", "marsh", "the water's edge", "ground too low to build on", "water"][self.tier() as usize]
    }
}

/// How far from a river's line its bed can lower the ground, in cells: the
/// bed takes the squares of the mesh the water lies in (`ground::bed`), and a
/// spot's ground is read between the corners of its own square.
const BED: f32 = 0.27;

/// The spots near a river's water, on a grid of `MARK` to a cell.
pub struct Rivers {
    w: usize,
    h: usize,
    /// within `BACK` of the water
    near: Vec<bool>,
    /// within `BED` of a reach whose water stands under `DRY_M`: its bed
    /// brings the ground about it down to its water, below the line nothing
    /// is built under
    low: Vec<bool>,
}

impl Rivers {
    /// Mark every spot near the water of a river the viewer draws.
    pub fn of(l: &NewLand) -> Rivers {
        let (w, h) = (l.w * MARK, l.h * MARK);
        let (mut near, mut low) = (vec![false; w * h], vec![false; w * h]);
        let m = MARK as f32;
        let mark = |marks: &mut [bool], p: Vec2, reach: f32| {
            let (x0, x1) = (((p.x - reach) * m).floor().max(0.0) as usize, (((p.x + reach) * m).floor().max(0.0) as usize).min(w - 1));
            let (y0, y1) = (((p.y - reach) * m).floor().max(0.0) as usize, (((p.y + reach) * m).floor().max(0.0) as usize).min(h - 1));
            for y in y0..=y1 {
                for x in x0..=x1 {
                    // the middle of the grid's cell, within reach and half the cell's own size
                    let q = Vec2::new((x as f32 + 0.5) / m, (y as f32 + 0.5) / m);
                    if q.distance(p) <= reach + 0.5 / m {
                        marks[y * w + x] = true;
                    }
                }
            }
        };
        for line in &l.lines {
            for (i, &p) in line.pts.iter().enumerate() {
                if line.flow[i] < MIN_FLOW {
                    continue;
                }
                let half = 0.5 * line.width[i] / CELL_M;
                mark(&mut near, p, half + BACK);
                if line.level[i] < DRY_M {
                    mark(&mut low, p, half + BED);
                }
            }
        }
        Rivers { w, h, near, low }
    }

    fn at(&self, x: f32, y: f32) -> usize {
        ((y * MARK as f32).max(0.0) as usize).min(self.h - 1) * self.w + ((x * MARK as f32).max(0.0) as usize).min(self.w - 1)
    }
}

/// What a spot is like to stand a market place on.
pub fn footing(l: &NewLand, rivers: &Rivers, x: f32, y: f32) -> Footing {
    // the spot and eight spots round it
    let mut round = vec![(x, y)];
    for k in 0..8 {
        let a = k as f32 * std::f32::consts::FRAC_PI_4;
        round.push((x + ROUND * a.cos(), y + ROUND * a.sin()));
    }
    let (mut water, mut low, mut shore, mut marsh) = (false, false, false, false);
    for (i, &(px, py)) in round.iter().enumerate() {
        let n = l.node_at(px, py);
        let m = l.metres_at(px, py);
        // under the tide (or on the beach it leaves), in a lake, or under the water of a lake beside it
        let wet = l.lake[n] > 0 || m < BEACH_M || l.lake_beside(n).is_some_and(|level| m < level + 0.5);
        // too low to build on: by its own height, or brought down by the bed of a river near the tide
        let sunk = m < DRY_M || rivers.low[rivers.at(px, py)];
        if i == 0 {
            (water, low) = (wet, !wet && sunk);
        } else {
            shore |= wet;
            marsh |= sunk;
        }
        marsh |= l.wet[n] == WET_MARSH;
    }
    let e = 0.06;
    let fall = (l.metres_at(x + e, y) - l.metres_at(x - e, y)).hypot(l.metres_at(x, y + e) - l.metres_at(x, y - e)) / (2.0 * e * CELL_M);
    Footing { water, low, shore, marsh, river: rivers.near[rivers.at(x, y)], slope: fall }
}

/// The meeting place of every cell: for a cell of water (`terrain` 0) the old
/// island's, and for a land cell the nearest spot to the old island's that
/// is as good as the cell has.
pub fn place(l: &NewLand, terrain: &[u8]) -> Vec<(f32, f32)> {
    use rayon::prelude::*;
    let rivers = Rivers::of(l);
    (0..l.w * l.h)
        .into_par_iter()
        .map(|c| {
            let first = jittered(c, l.w);
            if terrain.get(c).is_none_or(|&t| t == 0) {
                return first;
            }
            // most of the lowland: where the old island would put it is dry, firm and level, and it stays
            let there = footing(l, &rivers, first.0, first.1);
            if there.tier() == 0 && there.slope <= LEVEL {
                return first;
            }
            // else that spot and a grid of spots over the cell
            let (cx, cy) = ((c % l.w) as f32, (c / l.w) as f32);
            let mut spots = vec![first];
            for j in 0..TRY {
                for i in 0..TRY {
                    let at = |k: usize| INSET + (1.0 - 2.0 * INSET) * k as f32 / (TRY - 1) as f32;
                    spots.push((cx + at(i), cy + at(j)));
                }
            }
            let found: Vec<Footing> = spots.iter().map(|&(x, y)| footing(l, &rivers, x, y)).collect();
            // the best kind of ground the cell has, and among the spots of that kind the gentlest
            let best = found.iter().map(|f| f.tier()).min().unwrap_or(0);
            let gentlest = found.iter().filter(|f| f.tier() == best).map(|f| f.slope).fold(f32::MAX, f32::min);
            // level enough: level, or not much steeper than the gentlest the cell has
            let level = LEVEL.max(1.5 * gentlest + 0.01);
            let d2 = |p: (f32, f32)| (p.0 - first.0).powi(2) + (p.1 - first.1).powi(2);
            let mut chosen = 0;
            let mut least = f32::MAX;
            for (i, f) in found.iter().enumerate() {
                if f.tier() == best && f.slope <= level && d2(spots[i]) < least {
                    (chosen, least) = (i, d2(spots[i]));
                }
            }
            spots[chosen]
        })
        .collect()
}
