//! What a house is roofed with. The covering is settled when the house goes
//! up, by where it stands and what could be had that year: thatch on the
//! older lowland cottages, clay tile in the lowland, stone slates in the
//! hills, and blue slate from the quarries once it can be carried, which is
//! soonest where the railway has come. A roof then weathers in steps, a
//! quiet tint toward moss and lichen, each at ages of its own.

use crate::colour::{lerp, Rgba};
use crate::hash::Seed;
use crate::palette::Palette;

/// What covers a roof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cover {
    Thatch,
    /// thick sandstone slates, as the hills have them
    StoneSlate,
    Tile,
    /// thin blue slate brought from far off
    Slate,
}

/// A house's roof: what it is covered with and how weathered it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Roof {
    pub cover: Cover,
    /// from 0, new, to 3, a century old
    pub weathering: u8,
}

/// Blue slate begins to reach the towns in this year, and by `SLATE_COMMON` most new roofs are of it.
pub const SLATE_FIRST: i32 = 1815;
pub const SLATE_COMMON: i32 = 1860;
/// New cottages in the lowland are thatched less and less up to this year, and not after.
pub const THATCH_LAST: i32 = 1800;

/// The year a house that was standing when the story opens, in `start`, was
/// built: some time in the sixty years before, by a number of its own.
pub fn built_before(start: i32, seed: Seed) -> i32 {
    start - (60.0 * (seed.u * 5.3 + seed.v * 3.1).fract()) as i32
}

impl Roof {
    /// The year the roof a house has in `year` was laid. A roof lasts from
    /// seventy to a hundred and ten years, by a number of the house's own,
    /// and is then laid again in what can be had that year: so the blue
    /// slate comes to the old middle of a town as well as to its new edges,
    /// a roof at a time.
    pub fn laid(built: i32, year: i32, seed: Seed) -> i32 {
        let life = 70 + (40.0 * (seed.u * 3.7 + seed.v * 11.3).fract()) as i32;
        built + (year - built).max(0) / life * life
    }

    /// The roof of a house built in the year `built`, as it is in `year`.
    /// `upland` is whether it stands in the hills; `by_rail` whether the
    /// railway had reached its ground when the roof was laid (see `laid`).
    /// `seed` is the house's own, so the same house has the same roof.
    pub fn of(built: i32, year: i32, upland: bool, by_rail: bool, seed: Seed) -> Roof {
        let laid = Roof::laid(built, year, seed);
        let pick = (seed.u * 13.37 + seed.v * 7.11).fract();
        let slate = if laid < SLATE_FIRST {
            0.0
        } else {
            (0.7 * ((laid - SLATE_FIRST) as f32 / (SLATE_COMMON - SLATE_FIRST) as f32).min(1.0) + if by_rail { 0.25 } else { 0.0 }).max(0.04)
        };
        // only a house's first roof is thatch
        let thatch = if upland || laid > built { 0.0 } else { (0.45 * (THATCH_LAST - built) as f32 / 50.0).clamp(0.0, 0.45) };
        let cover = if pick < slate {
            Cover::Slate
        } else if pick > 1.0 - thatch {
            Cover::Thatch
        } else if upland {
            Cover::StoneSlate
        } else {
            Cover::Tile
        };
        // a step every forty years, begun at an age of the house's own
        let age = (year - laid).max(0) as f32;
        Roof { cover, weathering: (((age + 30.0 * seed.v) / 40.0) as u8).min(3) }
    }

    /// The same roof on a house that has been raised or rebuilt: thatch does
    /// not go back on, and the new roof is of tile.
    pub fn rebuilt(self) -> Roof {
        if self.cover == Cover::Thatch { Roof { cover: Cover::Tile, weathering: self.weathering.min(1) } } else { self }
    }

    /// The colour of the roof: its covering's, in a tint of the house's own, weathered.
    pub fn colour(&self, pal: &Palette, seed: Seed) -> Rgba {
        let tint = (seed.u * 29.3 + seed.v * 17.9).fract();
        let w = self.weathering.min(3) as usize;
        match self.cover {
            Cover::Thatch => lerp(*pal.thatch.cottage, *pal.thatch.weathered, [0.0, 0.35, 0.7, 1.0][w]),
            Cover::StoneSlate => lerp(*pal.slate.stone, *pal.roofs.moss, [0.0, 0.1, 0.2, 0.3][w]),
            Cover::Tile => {
                let tile = [pal.roofs.tile_red, pal.roofs.tile_orange, pal.roofs.tile_red, pal.roofs.tile_dark][(tint * 4.0) as usize % 4];
                lerp(*tile, *pal.roofs.moss, [0.0, 0.1, 0.2, 0.3][w])
            }
            Cover::Slate => {
                let slate = if tint < 0.5 { pal.slate.house } else { pal.slate.town };
                lerp(*slate, *pal.roofs.moss, [0.0, 0.07, 0.14, 0.22][w])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn share(cover: Cover, built: i32, upland: bool, by_rail: bool) -> f32 {
        (0..400).filter(|&n| Roof::of(built, built, upland, by_rail, Seed::of(n)).cover == cover).count() as f32 / 400.0
    }

    #[test]
    fn thatch_and_tile_early_and_slate_later() {
        // the lowland: thatch and tile at first, tile alone by 1800, and slate spreading from 1815
        assert!((0.35..0.55).contains(&share(Cover::Thatch, 1750, false, false)));
        assert!((0.45..0.65).contains(&share(Cover::Tile, 1750, false, false)));
        assert_eq!(share(Cover::Thatch, 1800, false, false), 0.0);
        assert_eq!(share(Cover::Slate, 1810, false, false), 0.0);
        let (s30, s60, s90) = (share(Cover::Slate, 1830, false, false), share(Cover::Slate, 1860, false, false), share(Cover::Slate, 1890, false, false));
        assert!(s30 > 0.1 && s30 < s60 && (0.6..0.8).contains(&s60) && s90 == s60, "{s30} {s60} {s90}");
        // the railway brings it sooner
        assert!(share(Cover::Slate, 1830, false, true) > s30 + 0.15);
        // the hills have stone slates, never thatch or tile
        assert_eq!(share(Cover::StoneSlate, 1750, true, false), 1.0);
        assert_eq!(share(Cover::Tile, 1870, true, false) + share(Cover::Thatch, 1870, true, false), 0.0);
    }

    #[test]
    fn a_house_keeps_its_roof_and_it_weathers_in_steps() {
        let mut relaid = 0;
        for n in 0..60 {
            let seed = Seed::of(n);
            let new = Roof::of(1790, 1790, false, false, seed);
            let mut last = new;
            assert_eq!(last.weathering, 0);
            for year in 1790..=1900 {
                let r = Roof::of(1790, year, false, false, seed);
                if Roof::laid(1790, year, seed) == 1790 {
                    // the roof it was built with, a step older now and then
                    assert!(year < 1790 + 110 && r.cover == new.cover);
                    assert!(r.weathering == last.weathering || r.weathering == last.weathering + 1);
                    assert!(year < 1855 || r.weathering >= 1);
                } else {
                    // laid again, once, in tile or slate, and new again that year
                    assert!(year >= 1790 + 70 && r.cover != Cover::Thatch);
                    if Roof::laid(1790, year - 1, seed) == 1790 {
                        assert_eq!(r.weathering, 0);
                        relaid += 1;
                    }
                }
                last = r;
            }
        }
        assert!(relaid > 30, "{relaid} of 60 roofs were laid again within 110 years");
        // two houses of an age need not be at the same step
        assert!((0..60).any(|n| Roof::of(1790, 1820, false, false, Seed::of(n)).weathering != Roof::of(1790, 1820, false, false, Seed::of(n + 1)).weathering));
    }

    #[test]
    fn weathering_is_a_tint_not_a_colour_of_its_own() {
        let pal = Palette::standard();
        let far = |a: Rgba, b: Rgba| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        for n in 0..40 {
            let seed = Seed::of(n);
            for cover in [Cover::StoneSlate, Cover::Tile, Cover::Slate] {
                let (new, old) = (Roof { cover, weathering: 0 }.colour(&pal, seed), Roof { cover, weathering: 3 }.colour(&pal, seed));
                // an old roof is still nearer its own covering than the moss it leans toward
                assert!(new != old && far(old, new) < far(old, *pal.roofs.moss), "{cover:?} seed {n}");
                // and never greener than it is red
                if cover == Cover::Tile {
                    assert!(old[0] > old[1] * 1.5);
                }
            }
        }
        // a rebuilt house is not thatched again
        let thatched = Roof { cover: Cover::Thatch, weathering: 3 };
        assert_eq!(thatched.rebuilt().cover, Cover::Tile);
        assert_eq!(Roof { cover: Cover::Slate, weathering: 2 }.rebuilt(), Roof { cover: Cover::Slate, weathering: 2 });
    }
}
