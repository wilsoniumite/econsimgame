//! Isolated State: an agent-based spatial economy on a procedural island, 1750 to 1900.
//!
//! A Rust port of the Python package `isostate`. The modules mirror the Python
//! files; the state is one struct of arrays (`state::State`) that free functions
//! read and write. See `PORTING.md` for the mapping and every deviation.
//!
//! The public shape a binding can wrap:
//!
//! - [`load_content`] / [`default_content`]: the goods, needs, sectors, methods and techs.
//! - [`default_params`]: every tunable number; [`Params`] loads overrides from TOML or JSON.
//! - [`init_state`]: the island and its 1750 economy.
//! - [`step_month`] and [`step_year`]: one month of the economy, one yearly update.
//! - [`run`]: the whole history; [`sim::start`], [`sim::run_year`] and [`sim::finish`] step through it.
#![allow(non_snake_case)]

pub mod cli;
pub mod content;
pub mod econ;
pub mod firms;
pub mod land;
pub mod mat;
pub mod names;
pub mod ndimage;
pub mod observer;
pub mod params;
pub mod people;
pub mod rail;
pub mod record;
pub mod rng;
pub mod sim;
pub mod state;
pub mod tech;
pub mod transport;
pub mod util;
pub mod world;

pub use content::{Content, default_content, load_content};
pub use econ::step_month;
pub use params::{Params, RentRegime, default_params};
pub use sim::{RunResult, Sim, init_state, run, yearly as step_year};
pub use state::{Event, State};
