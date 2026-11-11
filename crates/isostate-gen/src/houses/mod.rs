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
//! close on either side, and the `Detail` it is to be drawn at.

mod cottage;
mod farmhouse;
mod kit;
pub mod roof;
mod terrace;
mod townhouse;
mod villa;

pub use cottage::{cottage, Cottage};
pub use farmhouse::{farmhouse, Farmhouse, FARMHOUSE};
pub use kit::{Detail, Era};
pub use roof::Roof;
pub use terrace::{terrace, Terrace};
pub use townhouse::{townhouse, Townhouse};
pub use villa::{villa, Villa};

use glam::Vec3;

use crate::colour::{pick, shade, Rgba};
use crate::hash::Seed;
use crate::mesh::MeshBuf;
use crate::palette::Palette;
use crate::houses::kit::STOREY;
use crate::site::Site;
use crate::trees::{hedge_tree, Greenery};
use kit::{Kit, Open, Opening, Over, Range, Wall, FOOT};

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

impl Fabric {
    /// `brick_share`: how many of the lowland's houses of the brick eras are of brick (the rest are washed).
    pub fn of(pal: &Palette, era: Era, upland: bool, brick_early: f32, brick_late: f32, seed: Seed) -> Fabric {
        let (frame, sill) = (*pal.walls.window_frame, *pal.stone.sill);
        if upland {
            // the hills build in their own stone, to the ground
            let col = pick(&pal.stone.upland_farm, seed.v);
            return Fabric { wall: Wall { col, plinth: None, frame, sill }, timber: false, brick: false, stack: shade(col, 0.9), yard_wall: *pal.stone.garden_wall };
        }
        let share = if era >= Era::Brick { brick_late } else { brick_early };
        if seed.pick(1) < share {
            let col = pal.brick.terrace(seed.u);
            return Fabric { wall: Wall { col, plinth: None, frame, sill }, timber: false, brick: true, stack: *pal.brick.chimney, yard_wall: *pal.brick.dark };
        }
        let col = pal.walls.town_house(seed.v);
        let yard_wall = if era >= Era::Brick { *pal.brick.dark } else { *pal.stone.yard_wall };
        Fabric { wall: Wall { col, plinth: Some(*pal.stone.plinth), frame, sill }, timber: era == Era::Old && seed.pick(2) < 0.65, brick: false, stack: *pal.brick.chimney, yard_wall }
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
/// under the eaves, on side `side` of a range from `y0` to `y1`.
fn timber_frame(k: &mut Kit, rg: &Range, side: usize, y0: f32, y1: f32, posts: &[f32]) {
    if !k.near() {
        return;
    }
    let oak = *k.pal.timber.house_frame;
    let len = rg.face(side).len;
    let (t, out) = (0.0012, 0.0006);
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

/// A house fire's smoke, rising from the top of a stack.
fn hearth_smoke(smoke: &mut MeshBuf, top: Vec3, seed: Seed, pal: &Palette) {
    crate::works::plume(smoke, top, 0.39, seed.pick(12) * 7.0, *pal.smoke.puff);
}

/// A small tree in a garden, at a place in the house's frame.
fn garden_tree(green: &mut Greenery, k: &Kit, site: &Site, x: f32, z: f32, size: f32, seed: Seed, pal: &Palette) {
    let p = k.pt(x, k.ground(x, z), z);
    hedge_tree(green, &site.moved(p, 0.0), size, seed.pick(17) * 3.0, seed.pick(18) * 6.0, pal);
}
