//! The houses of a town and of the country round it: a cottage on its own
//! plot, a townhouse the full width of its plot in a high street's frontage,
//! a terraced row, a villa in its garden, and the farmhouse of a farmstead.
//!
//! The town's layout (the viewer's `scatter/towns.rs`) decides which plots are
//! built and what kind of house each gets; these draw one house or one row.
//! How many storeys a house has is a step it takes at a density of its own:
//! each kind has a `storeys` function that reads the step off the ground's
//! density, and the generator is handed the answer.
//!
//! Every house is put together from the parts in `kit.rs` (ranges with walls
//! and openings, roofs fitted to them, stacks, and what stands in a yard),
//! and is handed, besides its plot, storeys, roof and seed: its `Era` (the
//! step its style takes from the year its plot was first built), whether it
//! stands in the hills (stone for wash and brick), whether a neighbour stands
//! close on either side, its `Street` (the family its street's walls and
//! tiles run in, and the street's habits), and the `Detail` it is to be
//! drawn at.
//!
//! What a house looks like follows from what it is told, so that variety
//! means something: wall and roof run in families along a street, and a
//! street has habits of building that the next street has not (hipped
//! ends, rooms in the roof, a gable to the street, a cross-wing where the
//! plots are wide); a cottage raised a storey in a later age shows the
//! later work above the old. A house that is told none of this is the house
//! it always was: `tests/houses_as_told.rs` holds it to that, corner for
//! corner, at every level of detail.

mod cottage;
mod farmhouse;

pub mod roof;
mod terrace;
mod townhouse;
mod villa;

pub use cottage::{cottage, Cottage};
pub use farmhouse::{farmhouse, Farmhouse, FARMHOUSE};
pub use crate::kit::{Detail, Era};
pub use roof::Roof;
pub use terrace::{terrace, Terrace};
pub use townhouse::{townhouse, Townhouse};
pub use villa::{villa, Villa};

use glam::Vec3;

use crate::colour::{pick, shade, Rgba};
use crate::hash::Seed;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::kit::STOREY;
use crate::site::Site;
use crate::trees::{hedge_tree, Greenery};
use crate::kit::{Kit, Open, Opening, Over, Range, Wall, FOOT};

/// The steps each kind of house takes as its ground is built up.
pub mod storeys {
    pub use super::cottage::storeys as cottage;
    pub use super::terrace::storeys as terrace;
    pub use super::townhouse::storeys as townhouse;
}

/// The width and depth of a house plot as the towns lay them out on a high
/// street and round a market place, in map units.
pub const PLOT: (f32, f32) = (0.05, 0.036);
/// The width of a plot on a back street, where the terraces are.
pub const BACK_PLOT_WIDTH: f32 = 0.046;
/// The ground behind a plot that its house may take for its yard or its
/// garden: the towns leave twice this between the backs of two rows.
pub const BACK_ROOM: f32 = 0.012;
/// How far a plot's front stands back from the edge of its street: the path to the door crosses it.
pub const SET_BACK: f32 = 0.005;

/// Whether a townhouse's ground floor is a shop: where the ground is built
/// up enough, each house at a density of its own, so that a high street
/// fills with shops from its middle as the town grows busy.
pub fn shop_at(density: f32, seed: Seed) -> bool {
    density >= 0.55 + 0.4 * seed.pick(30)
}

/// Whether a plot that would have a cottage has a villa instead: about a
/// third of those first built once the railway years' money was about.
pub fn villa_plot(built: i32, seed: Seed) -> bool {
    built >= 1830 && seed.pick(31) < 0.34
}

/// What a house is told of the street it stands in. The layout knows the
/// street, and with it variety means something: the wash or the brick of the
/// walls and the tint of the tiles run in families along a street, and a
/// street has its habits (hipped ends, rooms in the roof, where the stacks
/// stand, a gable to the street), so that two streets of one town are not
/// the same street twice.
///
/// A house told nothing of its street (`Street::ALONE`) chooses every colour
/// for itself and takes none of the habits: it is the house it was before
/// houses were told of their streets, to the last corner (the test
/// `tests/houses_as_told.rs` holds that).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Street {
    /// two numbers that the houses of one street share. None for a house
    /// that is told nothing of its street
    pub family: Option<Seed>,
}

impl Street {
    /// A house told nothing of its street.
    pub const ALONE: Street = Street { family: None };

    /// A street of a family.
    pub fn of(family: Seed) -> Street {
        Street { family: Some(family) }
    }

    /// One of `n` things (washes, bricks, tints of tile) as a number from 0
    /// to 1, for a house whose own choice would be `own`: in a street of a
    /// family, half the houses take the street's (its `slot`th number says
    /// which), a quarter the next to it, and a quarter their own, so that a
    /// street is a family and not a uniform. `r` is a number of the house's
    /// own.
    fn choice(&self, n: usize, slot: i32, own: f32, r: f32) -> f32 {
        let Some(f) = self.family else { return own };
        let base = (f.pick(slot) * n as f32) as usize % n;
        if r < 0.5 {
            (base as f32 + 0.5) / n as f32
        } else if r < 0.75 {
            ((base + 1) % n) as f32 / n as f32 + 0.5 / n as f32
        } else {
            own
        }
    }

    /// A share (of houses in brick, of houses that show their stone) as a
    /// street of a family has it: a street leans one way or the other, by
    /// up to `LEAN` either way, so that a few are nearly all of one kind
    /// and most are mixed. A share of none or of all is left alone: what
    /// cannot be had in a year is not built.
    fn lean(&self, share: f32, slot: i32) -> f32 {
        self.leaning(share, slot, Self::LEAN)
    }

    /// How far a street's share of a material leans from the island's, at the most.
    const LEAN: f32 = 0.45;

    fn leaning(&self, share: f32, slot: i32, by: f32) -> f32 {
        match self.family {
            Some(f) if share > 0.0 && share < 1.0 => (share + (f.pick(slot) - 0.5) * 2.0 * by).clamp(0.0, 1.0),
            _ => share,
        }
    }

    /// The share of a street's houses that take a form no house took
    /// before houses were told of their streets (a hipped roof, rooms in the
    /// roof, a stack in the middle of the ridge): `share` of them over all
    /// the streets, leaning from street to street further than a material
    /// does, so that a form is one street's habit and is hardly seen in the
    /// next. A house told nothing of its street takes none of them.
    fn habit(&self, share: f32, slot: i32) -> f32 {
        if self.family.is_some() { self.leaning(share, slot, 0.65) } else { 0.0 }
    }
}

/// What a house is built of, settled by when and where it was built and by a number of its own.
#[derive(Clone, Copy, Debug)]
struct Fabric {
    pub wall: Wall,
    /// a timber frame shows in its walls
    pub timber: bool,
    pub brick: bool,
    /// its chimney stacks
    pub stack: Rgba,
    /// the wall round its yard
    pub yard_wall: Rgba,
}

/// In the hills at least this share of the houses show their stone; the rest are limewashed over it.
const UPLAND_BARE: f32 = 0.6;

impl Fabric {
    /// `brick_early`, `brick_late`: how many of the lowland's houses of this
    /// kind are of brick, before the brick years and in them (the rest are
    /// washed). In the hills stone stands where brick would, and for
    /// `UPLAND_BARE` of the houses at the least.
    ///
    /// In a street of a family (`Street::family`) the material and its
    /// colour run along the street: a street leans to brick or to wash (to
    /// bare stone or to limewash in the hills, to timber frames or to
    /// plastered fronts among the oldest), and most of its houses take the
    /// street's wash, its brick or its stone.
    pub fn of(pal: &Palette, era: Era, upland: bool, brick_early: f32, brick_late: f32, seed: Seed, street: &Street) -> Fabric {
        let (frame, sill) = (*pal.walls.window_frame, *pal.stone.sill);
        let share = if era >= Era::Brick { brick_late } else { brick_early };
        if upland {
            // the hills build in their own stone, to the ground
            let stone = pick(&pal.stone.upland_farm, street.choice(pal.stone.upland_farm.len().max(1), 3, seed.v, seed.pick(42)));
            let (stack, yard_wall) = (shade(stone, 0.9), *pal.stone.garden_wall);
            // (a street may be all of bare stone; none is all limewash)
            let bare = share.max(UPLAND_BARE);
            if seed.pick(1) < street.lean(bare, 1).max(bare.min(0.35)) {
                return Fabric { wall: Wall { col: stone, plinth: None, frame, sill }, timber: false, brick: false, stack, yard_wall };
            }
            // a coat of limewash over the rubble, white or cream, down to a foot of bare stone: a
            // town of nothing but stone under stone slates is one grey from the play distance
            let col = if seed.pick(2) < 0.6 { *pal.walls.limewash } else { *pal.walls.cream };
            return Fabric { wall: Wall { col, plinth: Some(shade(stone, 0.86)), frame, sill }, timber: false, brick: false, stack, yard_wall };
        }
        if seed.pick(1) < street.lean(share, 1) {
            let col = pal.brick.terrace(street.choice(4, 2, seed.u, seed.pick(41)));
            return Fabric { wall: Wall { col, plinth: None, frame, sill }, timber: false, brick: true, stack: *pal.brick.chimney, yard_wall: *pal.brick.dark };
        }
        let col = pal.walls.town_house(street.choice(6, 3, seed.v, seed.pick(42)));
        let yard_wall = if era >= Era::Brick { *pal.brick.dark } else { *pal.stone.yard_wall };
        Fabric { wall: Wall { col, plinth: Some(*pal.stone.plinth), frame, sill }, timber: era == Era::Old && seed.pick(2) < street.lean(0.65, 4), brick: false, stack: *pal.brick.chimney, yard_wall }
    }
}

/// The windows of an era: casements in the oldest houses, sashes after.
fn window(era: Era) -> Open {
    if era == Era::Old { Open::Casement } else { Open::Sash }
}

/// A front door: painted in a colour of the house's own, and with what its era put over a door.
fn door(pal: &Palette, era: Era, seed: Seed) -> Open {
    let paint = pick(&pal.walls.door_paints, seed.pick(4));
    let over = match era {
        Era::Old => Over::Nothing,
        Era::Georgian => if seed.pick(5) < 0.55 { Over::Hood } else { Over::Fanlight },
        _ => if seed.pick(5) < 0.7 { Over::Fanlight } else { Over::Hood },
    };
    Open::Door { paint, over }
}

/// A door standing on the floor at `u` along its wall.
fn door_at(u: f32, kind: Open) -> Opening {
    Opening { u, y: FOOT, w: 0.0052, h: 0.0126, kind }
}

/// A window of storey `s` at `u` along its wall, `w` wide and `h` tall, its sill `sill` above the floor.
fn window_at(u: f32, s: usize, sill: f32, w: f32, h: f32, kind: Open) -> Opening {
    // (the sills and heights its callers give are for a storey of 0.024)
    Opening { u, y: s as f32 * STOREY + sill * 0.86, w: w * 1.16, h: h * 0.86, kind }
}

/// The timbers that show in a framed wall, seen close to: posts at its ends
/// and at `posts` along it, a sill beam, a rail at each floor and the plate
/// under the eaves, on side `side` of a range from `y0` to `y1`. The timbers
/// stand a little off the wall, so they stop short of its top: the roof
/// comes down over the wall there, and they would show through it.
fn timber_frame(k: &mut Kit, rg: &Range, side: usize, y0: f32, y1: f32, posts: &[f32]) {
    if !k.near() {
        return;
    }
    let oak = *k.pal.timber.house_frame;
    let len = rg.face(side).len;
    let (t, out) = (0.0012, 0.0006);
    let y1 = y1 - 0.0012;
    k.strip(rg, side, 0.0, t, y0, y1, out, oak);
    k.strip(rg, side, len - t, len, y0, y1, out, oak);
    for &u in posts {
        k.strip(rg, side, u - t * 0.5, u + t * 0.5, y0, y1, out, oak);
    }
    k.strip(rg, side, t, len - t, y0, y0 + t, out, oak);
    k.strip(rg, side, t, len - t, y1 - t - 0.0002, y1 - 0.0002, out, oak);
    let mut y = (y0 / STOREY).floor() * STOREY + STOREY;
    while y < y1 - 0.004 {
        k.strip(rg, side, t, len - t, y - t * 0.5, y + t * 0.5, out, oak);
        y += STOREY;
    }
}

/// The same wall from the distance the game is played from, for a house
/// told of its street: a hint of the black and white that reads from
/// there. Only the timbers that carry at that distance are drawn, and
/// stouter than they are, since a timber a hand's breadth wide is a third
/// of a pixel from up there: the posts at the wall's ends and at `posts`
/// along it, and the rail at each floor. (A house told nothing of its
/// street shows its frame only close to, as every house once did.)
fn timber_hint(k: &mut Kit, rg: &Range, side: usize, y0: f32, y1: f32, posts: &[f32]) {
    if k.near() || k.far() {
        return;
    }
    let oak = *k.pal.timber.house_frame;
    let len = rg.face(side).len;
    let (t, out) = (0.0022, 0.0005);
    let y1 = y1 - 0.0012;
    k.strip(rg, side, 0.0, t, y0, y1, out, oak);
    k.strip(rg, side, len - t, len, y0, y1, out, oak);
    for &u in posts {
        k.strip(rg, side, u - t * 0.5, u + t * 0.5, y0, y1, out, oak);
    }
    let mut y = (y0 / STOREY).floor() * STOREY + STOREY;
    while y < y1 - 0.004 {
        k.strip(rg, side, t, len - t, y - t * 0.5, y + t * 0.5, out, oak);
        y += STOREY;
    }
}

/// A house fire's smoke, rising from the top of a stack.
fn hearth_smoke(smoke: &mut MeshBuf, top: Vec3, seed: Seed, pal: &Palette) {
    crate::works::plume(smoke, top, 0.39, seed.pick(12) * 7.0, *pal.smoke.puff);
}

/// A small tree in a garden, at a place in the house's frame.
fn garden_tree(green: &mut Greenery, k: &Kit, site: &Site, x: f32, z: f32, size: f32, seed: Seed, pal: &Palette) {
    let p = k.pt(x, k.ground(x, z), z);
    hedge_tree(green, &site.moved(p, 0.0), size, seed.pick(17) * 3.0, seed.pick(18) * 6.0, pal);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hills_build_in_stone_and_limewash_some_of_it() {
        let pal = Palette::standard();
        let of_stone = |f: &Fabric| pal.stone.upland_farm.iter().any(|c| **c == f.wall.col);
        // houses that stand apart: most show their stone, the rest are white or cream on a foot of stone
        let houses: Vec<Fabric> = (0..400).map(|n| Fabric::of(&pal, Era::Georgian, true, 0.0, 0.6, Seed::of(n), &Street::ALONE)).collect();
        let bare = houses.iter().filter(|f| of_stone(f)).count();
        assert!((200..290).contains(&bare), "{bare} of 400 show their stone");
        for f in houses.iter().filter(|f| !of_stone(f)) {
            assert!(f.wall.col == *pal.walls.limewash || f.wall.col == *pal.walls.cream);
            assert!(f.wall.plinth.is_some());
        }
        // none is framed or of brick, washed or not, and their stacks are of stone
        assert!(houses.iter().all(|f| !f.timber && !f.brick && f.stack != *pal.brick.chimney));
        // a row, which in the lowland is all of brick, is all of stone
        assert!((0..400).all(|n| of_stone(&Fabric::of(&pal, Era::Brick, true, 1.0, 1.0, Seed::of(n), &Street::ALONE))));
    }

    #[test]
    fn a_street_s_houses_run_in_a_family_and_streets_differ() {
        let pal = Palette::standard();
        // of the houses of one street, how many are of the commonest wall
        let commonest = |street: &Street, era: Era, upland: bool| {
            let walls: Vec<_> = (0..200).map(|n| Fabric::of(&pal, era, upland, 0.25, 0.6, Seed::of(n), street).wall.col).collect();
            let most = walls.iter().map(|w| walls.iter().filter(|x| *x == w).count()).max().unwrap();
            (most, walls[walls.iter().position(|w| walls.iter().filter(|x| *x == w).count() == most).unwrap()])
        };
        // each for itself: no wash or brick has more than a third of the houses
        assert!(commonest(&Street::ALONE, Era::Georgian, false).0 < 70);
        let mut leading = Vec::new();
        for s in 0..40 {
            let street = Street::of(Seed::of(900 + s));
            for (era, upland) in [(Era::Georgian, false), (Era::Late, false), (Era::Late, true)] {
                let (most, wall) = commonest(&street, era, upland);
                // in a street most are of one, and never all: a street is a family, not a uniform
                assert!(most >= 60 && most <= 190, "street {s} {era:?}: {most} of 200 alike");
                if era == Era::Georgian {
                    leading.push(wall);
                }
            }
            // and the same house in the same street is the same house
            let f = |n| Fabric::of(&pal, Era::Brick, false, 0.25, 0.6, Seed::of(n), &street).wall.col;
            assert_eq!(f(7), f(7));
        }
        // the streets do not all take the same wash
        leading.sort_by(|a, b| a.partial_cmp(b).unwrap());
        leading.dedup();
        assert!(leading.len() >= 4, "{} washes lead among 40 streets", leading.len());
    }
}
