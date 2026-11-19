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
//! The rule itself is the core's (`isostate_core::crossings`, "where a
//! cell's roads meet"): the core reads the same meeting places to say where
//! a way between two cells crosses water, and what that costs (the core's
//! docs/rules/crossings.md). Here the viewer's copy of the land is handed to
//! it, so the two find the same spots to the last bit.

use isostate_core::crossings::{self as core, Fine};

use super::NewLand;

pub use isostate_core::crossings::{jittered, Footing};

impl Fine for NewLand {
    fn cells(&self) -> (usize, usize) {
        (self.w, self.h)
    }
    fn grid(&self) -> (usize, usize, usize) {
        (self.fine, self.fw, self.fh)
    }
    fn height(&self) -> &[f32] {
        &self.height
    }
    fn lake(&self, n: usize) -> u8 {
        self.lake[n]
    }
    fn wet(&self, n: usize) -> u8 {
        self.wet[n]
    }
    fn lake_level(&self, id: u8) -> f32 {
        self.lakes.get(id as usize - 1).map_or(0.0, |l| l.level)
    }
    fn rivers(&self) -> usize {
        self.lines.len()
    }
    fn points(&self, r: usize) -> usize {
        self.lines[r].pts.len()
    }
    fn point(&self, r: usize, i: usize) -> (f32, f32) {
        let p = self.lines[r].pts[i];
        (p.x, p.y)
    }
    fn width(&self, r: usize) -> &[f32] {
        &self.lines[r].width
    }
    fn level(&self, r: usize) -> &[f32] {
        &self.lines[r].level
    }
    fn flow(&self, r: usize) -> &[f32] {
        &self.lines[r].flow
    }
}

/// The spots near a river's water, on a grid finer than the cells.
pub struct Rivers(core::Marks);

impl Rivers {
    /// Mark every spot near the water of a river the viewer draws.
    pub fn of(l: &NewLand) -> Rivers {
        Rivers(core::Marks::of(l, true))
    }
}

/// What a spot is like to stand a market place on.
pub fn footing(l: &NewLand, rivers: &Rivers, x: f32, y: f32) -> Footing {
    core::footing(l, &rivers.0, x, y)
}

/// The meeting place of every cell: for a cell of water (`terrain` 0) the old
/// island's, and for a land cell the nearest spot to the old island's that
/// is as good as the cell has.
pub fn place(l: &NewLand, terrain: &[u8]) -> Vec<(f32, f32)> {
    use rayon::prelude::*;
    let marks = core::Marks::of(l, true);
    (0..l.w * l.h).into_par_iter().map(|c| core::meeting_place(l, &marks, terrain, c)).collect()
}
