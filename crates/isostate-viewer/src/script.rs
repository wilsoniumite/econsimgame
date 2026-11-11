//! A scripted run, for checking keys and clicks with nobody at the keyboard:
//!
//!     isostate-viewer --advance 30 --script key:M,key:M,chip:wages,key:3,row:coal --view out.png
//!
//! Each step is pressed as the player would press it: a key goes into the
//! keyboard's state for one frame, and a click sets a button to pressed, so the
//! same systems answer as in play. After every step the viewer prints what the
//! map and the Markets panel now show. Screenshots wait for the script, so a
//! `--view` with no words of its own shows where the steps led.
//!
//! Steps: `key:M` (any letter or digit, `Shift+M` with Shift held), `chip:rent`
//! (a chip on the legend card: rent, areas, reach, price, wages, plain, or
//! `pill` for the Map views button of the plain map), `good:next` and
//! `good:back` (the legend's steps through the goods), `row:coal` (a good's row
//! in the Markets panel), `half:island` and `half:goods` (its two halves).

// see the note at the top of ui.rs
#![allow(deprecated)]

use std::collections::VecDeque;

use bevy::input::InputSystems;
use bevy::prelude::*;
use bevy::ui::UiSystems;

use crate::overlay::{GoodStep, MapOverlay, Overlay, OverlayChip};
use crate::ui::{GoodRow, MarketsChip, MarketsShow, UiState};
use crate::Data;

/// Frames between steps, so panels that are rebuilt after one are there for the next.
const PAUSE: u32 = 8;

#[derive(Resource)]
pub struct Script {
    steps: VecDeque<String>,
    wait: u32,
    /// keys to let go of on the next frame
    held: Vec<KeyCode>,
    /// the step whose outcome is still to be printed
    said: Option<String>,
}

impl Script {
    pub fn new(steps: &str) -> Script {
        Script { steps: steps.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(), wait: 30, held: Vec::new(), said: None }
    }

    /// Every step has been taken and reported.
    pub fn done(&self) -> bool {
        self.steps.is_empty() && self.said.is_none() && self.held.is_empty()
    }
}

pub fn plugin(app: &mut App) {
    // after the frame's real input is read and the pointer's buttons are settled, before anything acts on them
    app.add_systems(PreUpdate, run.after(InputSystems).after(UiSystems::Focus));
}

fn key_code(name: &str) -> Option<KeyCode> {
    Some(match name.to_ascii_uppercase().as_str() {
        "M" => KeyCode::KeyM,
        "G" => KeyCode::KeyG,
        "H" => KeyCode::KeyH,
        "T" => KeyCode::KeyT,
        "1" => KeyCode::Digit1,
        "2" => KeyCode::Digit2,
        "3" => KeyCode::Digit3,
        "4" => KeyCode::Digit4,
        "SPACE" => KeyCode::Space,
        "ESC" => KeyCode::Escape,
        _ => return None,
    })
}

#[allow(clippy::too_many_arguments)]
fn run(
    mut sc: ResMut<Script>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    data: Res<Data>,
    ov: Res<MapOverlay>,
    ui: Res<UiState>,
    shots: Option<Res<crate::ShotPlanArgs>>,
    mut exit: MessageWriter<AppExit>,
    mut chips: Query<(&mut Interaction, &OverlayChip), (Without<GoodStep>, Without<GoodRow>, Without<MarketsChip>)>,
    mut steps: Query<(&mut Interaction, &GoodStep), (Without<OverlayChip>, Without<GoodRow>, Without<MarketsChip>)>,
    mut rows: Query<(&mut Interaction, &GoodRow), (Without<OverlayChip>, Without<GoodStep>, Without<MarketsChip>)>,
    mut halves: Query<(&mut Interaction, &MarketsChip), (Without<OverlayChip>, Without<GoodStep>, Without<GoodRow>)>,
) {
    for k in std::mem::take(&mut sc.held) {
        keys.release(k);
    }
    if sc.wait > 0 {
        sc.wait -= 1;
        return;
    }
    let r = &data.r;
    let good = |i: usize| r.header.goods.get(i).map_or("?", |g| g.name.as_str());
    if let Some(step) = sc.said.take() {
        // what the last step led to, now that every system has had its say
        let map = ov.kind.map_or("plain".to_string(), |k| format!("{k:?}").to_lowercase());
        let half = if ui.markets == MarketsShow::Island { "island" } else { "goods" };
        let panel = ui.tab.map_or("none".to_string(), |t| format!("{t:?}").to_lowercase());
        println!("{step:<12} map: {map:<6} price of: {:<9} panel: {panel:<8} markets half: {half:<6} open good: {}", good(ov.good), ui.good.map_or("none", good));
    }
    let Some(step) = sc.steps.pop_front() else {
        if shots.is_none() {
            exit.write(AppExit::Success);
        }
        return;
    };
    let (what, arg) = step.split_once(':').unwrap_or((step.as_str(), ""));
    let arg_l = arg.to_ascii_lowercase();
    let mut hit = false;
    match what {
        "key" => {
            let (shift, name) = arg.strip_prefix("Shift+").map_or((false, arg), |n| (true, n));
            if let Some(k) = key_code(name) {
                if shift {
                    keys.press(KeyCode::ShiftLeft);
                    sc.held.push(KeyCode::ShiftLeft);
                }
                keys.press(k);
                sc.held.push(k);
                hit = true;
            }
        }
        "chip" => {
            let want: Option<Option<Overlay>> = match arg_l.as_str() {
                "pill" | "plain" => Some(None),
                _ => crate::overlay::parse(&arg_l, r).map(|p| p.0),
            };
            for (mut i, c) in &mut chips {
                if Some(c.0) == want {
                    *i = Interaction::Pressed;
                    hit = true;
                }
            }
        }
        "good" => {
            let want = if arg_l == "back" { -1 } else { 1 };
            for (mut i, s) in &mut steps {
                if s.0 == want {
                    *i = Interaction::Pressed;
                    hit = true;
                }
            }
        }
        "row" => {
            let want = r.header.goods.iter().position(|g| g.name.eq_ignore_ascii_case(arg));
            for (mut i, g) in &mut rows {
                if Some(g.0) == want {
                    *i = Interaction::Pressed;
                    hit = true;
                }
            }
        }
        "half" => {
            let want = if arg_l == "island" { MarketsShow::Island } else { MarketsShow::Prices };
            for (mut i, c) in &mut halves {
                if c.0 == want {
                    *i = Interaction::Pressed;
                    hit = true;
                }
            }
        }
        _ => {}
    }
    sc.said = Some(if hit { step.clone() } else { format!("{step} (nothing to press)") });
    sc.wait = PAUSE;
}
