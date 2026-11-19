//! The sandbox's Rules tab (`docs/designs/sandbox.md`, 4.6): the island's
//! rules, to read and to change by hand. Stage 4 puts the tab on the bar with a
//! line that says what it will be; stage 6 fills it, here, and adds what it
//! reads to `RulesParams` without coming into ui.rs.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::ui::{text, MUTED};

/// What the Rules tab reads (`ui::panel_rebuild` takes it and hands it here).
#[derive(SystemParam)]
pub struct RulesParams<'w> {
    /// the game's thread, which stage 6 asks for the rules as they stand
    #[allow(dead_code)]
    pub live: Option<Res<'w, crate::live::Live>>,
}

/// The one line the tab holds until stage 6.
pub const NOT_YET: &str = "The island's rules, to read and to change by hand, will be here.";

/// The Rules tab's body.
pub fn panel(p: &mut ChildSpawnerCommands, f: &Handle<Font>, _rules: &RulesParams) {
    p.spawn(text(f, NOT_YET, 14.0, MUTED));
}
