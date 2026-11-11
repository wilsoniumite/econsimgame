//! The generators of Isolated State: the functions that turn what the
//! simulation knows (a plot built up to a density, a works of a kind and a
//! size, a farm among arable fields) into buildings and trees.
//!
//! A generator is a plain function of plain inputs. It is handed the buffers
//! it writes, the site it stands on, a small struct of parameters with the
//! slot's seed in it, and the palette; it appends triangles and does nothing
//! else. There is no Bevy here, so the crate builds in a few seconds and its
//! tests run without a window. The viewer (`isostate-viewer`) calls these to
//! draw the island, and its `--lab` mode lays them out as contact sheets.
//!
//! README.md beside this crate's Cargo.toml has the rules and how to add a
//! generator and its sheet.

pub mod bridge;
pub mod colour;
pub mod farm;
pub mod fence;
pub mod hash;
pub mod houses;
mod kit;
pub mod mesh;
pub mod palette;
pub mod parts;
pub mod registry;
pub mod scale;
pub mod scenes;
pub mod sheets;
pub mod site;
pub mod town;
pub mod trees;
pub mod works;

pub use colour::{hex, lerp, pick, shade, Colour, Rgba};
pub use hash::{hash2, Seed};
pub use mesh::MeshBuf;
pub use palette::Palette;
pub use site::{Model, Site};
pub use trees::Greenery;
