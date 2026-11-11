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
//! - [`game_params`] / [`game_content`]: the rules a live game is played under. The
//!   defaults above are the port of the Python and stay exactly that; every mechanism
//!   the Python never had is off under them and on under the game's rules ([`Rules`]
//!   names the two; `docs/rules/README.md` has the convention).
//! - [`init_state`]: the island and its 1750 economy.
//! - [`step_month`] and [`step_year`]: one month of the economy, one yearly update.
//! - [`run`]: the whole history; [`sim::start`], [`sim::run_year`] and [`sim::finish`] step through it.
//! - [`families`]: the player's and rival families (off unless `n_families` > 0), and
//!   [`live`]: a running simulation's data, year by year, for the viewer (see FAMILIES.md).
//! - [`sweep`]: many seeds against many settings at once, and [`facts`]: the short list of
//!   stylised facts each run is reduced to (`isostate-core sweep`).
#![allow(non_snake_case)]

pub mod cli;
pub mod content;
pub mod econ;
pub mod engel;
pub mod facts;
pub mod families;
pub mod firms;
pub mod harvest;
pub mod land;
pub mod live;
pub mod machines;
pub mod mat;
pub mod names;
pub mod ndimage;
pub mod observer;
pub mod params;
pub mod people;
pub mod prices;
pub mod rail;
pub mod record;
pub mod rng;
pub mod save;
pub mod sim;
pub mod state;
pub mod sweep;
pub mod tech;
pub mod transport;
pub mod util;
pub mod world;

pub use content::{Content, default_content, game_content, load_content};
pub use families::{
    BranchReport, Choice, Decision, DecisionOption, FamilyReport, Group, MemberReport, Path, Post, Posting, Prospect, SiteOption, SiteSums, Stance, buy_land,
    decide, family_report, found_works, may_buy_land, move_seat, pending, post_member, set_stance, site_options,
};
pub use live::{LiveCursor, LiveStatic, LiveYear, live_static, live_year};
pub use econ::step_month;
pub use params::{Params, RentRegime, Rules, default_params, game_params};
pub use sim::{RunResult, Sim, init_state, run, yearly as step_year};
pub use state::{Event, State};
