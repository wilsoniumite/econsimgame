//! The state's purse, at the foot of the island half of the Markets panel
//! (docs/rules/state.md): the rate on pay, and what the schools and the
//! pensions cost over the island's income, year by year from 1900, with the
//! treasury in months of its spending in the year on the clock.
//!
//! Read from the core's series (`state_rate`, `state_schools_share`,
//! `state_pensions_share`, `state_months_held`), which a game under the game's
//! rules since 20 November 2026 keeps; a run without them, or a clock before
//! 1900, draws nothing here.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use crate::chart::{self, Lines, INK_U8};
use crate::replay::{self, Replay};
use crate::ui::{text, CHEAPER, GOOD, INK, MUTED};

const CHART_W: f32 = 342.0;
/// The year the chart begins: no act of the book can be met before it.
pub const FROM: i32 = 1900;

fn u8s(c: Color) -> [u8; 3] {
    let a = c.to_srgba().to_u8_array();
    [a[0], a[1], a[2]]
}

/// The four series from 1900, and the year on the clock counted from 1900:
/// None in a run without them or before 1900.
pub struct Purse {
    pub rate: Vec<f32>,
    pub schools: Vec<f32>,
    pub pensions: Vec<f32>,
    pub held: Vec<f32>,
    pub mark: usize,
}

pub fn read(r: &Replay, k: usize) -> Option<Purse> {
    let from = (FROM - r.y0).max(0) as usize;
    if k < from {
        return None;
    }
    let cut = |name: &str| -> Vec<f32> { replay::yearly(r, name).into_iter().skip(from).collect() };
    let rate = cut("state_rate");
    if rate.is_empty() {
        return None;
    }
    Some(Purse { rate, schools: cut("state_schools_share"), pensions: cut("state_pensions_share"), held: cut("state_months_held"), mark: k - from })
}

/// The figure for a share: per cent to one place.
fn pct(v: f32) -> String {
    format!("{:.1}%", v * 100.0)
}

/// What the print of a script says of the purse in the year on the clock
/// (`purse: YEAR | rate .. | schools .. | pensions .. | held .. months`), or None.
pub fn said(r: &Replay, k: usize) -> Option<String> {
    let p = read(r, k)?;
    let at = |v: &[f32]| v.get(p.mark).copied().unwrap_or(f32::NAN);
    Some(format!(
        "purse: {} | rate on pay {:.4} | schools {:.4} | pensions {:.4} of income | held {:.2} months",
        FROM + p.mark as i32,
        at(&p.rate),
        at(&p.schools),
        at(&p.pensions),
        at(&p.held)
    ))
}

/// The chart, under the island's others, at the year `k` on the clock.
pub fn spawn(p: &mut ChildSpawnerCommands, f: &Handle<Font>, images: &mut Assets<Image>, r: &Replay, k: usize) {
    let Some(pu) = read(r, k) else { return };
    let at = |v: &[f32]| v.get(pu.mark).copied().unwrap_or(f32::NAN);
    let (c_rate, c_school, c_pens) = (INK_U8, u8s(CHEAPER), u8s(GOOD));
    let ink = |c: [u8; 3]| Color::srgb_u8(c[0], c[1], c[2]);
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), margin: UiRect::top(Val::Px(12.0)), ..default() }).with_children(|col| {
        col.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Baseline, padding: UiRect::right(Val::Px(4.0)), ..default() }).with_children(|row| {
            row.spawn(text(f, "The state's purse", 15.0, INK));
            let held = at(&pu.held);
            let said = if held > 0.0 { format!("{held:.1} months in hand") } else { "nothing paid yet".to_string() };
            row.spawn(text(f, &said, 13.0, MUTED));
        });
        col.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(12.0), padding: UiRect::left(Val::Px(4.0)), ..default() }).with_children(|keys| {
            for (words, v, c) in [("rate on pay", at(&pu.rate), c_rate), ("schools", at(&pu.schools), c_school), ("pensions", at(&pu.pensions), c_pens)] {
                keys.spawn(text(f, &format!("{words} {}", if v.is_finite() { pct(v) } else { "-".into() }), 13.0, ink(c)));
            }
        });
        let lines: Vec<(&[f32], [u8; 3])> = vec![(&pu.rate[..], c_rate), (&pu.schools[..], c_school), (&pu.pensions[..], c_pens)];
        let c = Lines { width: CHART_W, height: 70.0, lines: &lines, mark: pu.mark, years: pu.rate.len(), first_year: FROM, from_zero: true };
        chart::spawn_lines(col, f, images, &c, &pct);
        col.spawn(text(f, "Schools and pensions over the island's income, and the rate on pay that pays for them, set each year so that the treasury keeps two months of its spending (docs/rules/state.md).", 12.5, MUTED));
    });
}
