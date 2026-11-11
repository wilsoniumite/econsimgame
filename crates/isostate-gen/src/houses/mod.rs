//! The houses of a town: a cottage on its own plot, a townhouse the full
//! width of its plot in a high street's frontage, and a terraced row.
//!
//! The town's layout (the viewer's `scatter/towns.rs`) decides which plots are
//! built and what kind of house each gets; these draw one house or one row.
//! How many storeys a house has is a step it takes at a density of its own:
//! each kind has a `storeys` function that reads the step off the ground's
//! density, and the generator is handed the answer.

mod cottage;
pub mod roof;
mod terrace;
mod townhouse;

pub use cottage::{cottage, Cottage};
pub use roof::Roof;
pub use terrace::{terrace, Terrace};
pub use townhouse::{townhouse, Townhouse};

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
