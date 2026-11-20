//! The island's people by age, at the foot of the island half of the Markets
//! panel (docs/rules/ages.md): the pyramid in the year on the clock, and the
//! shares under 15 and over 64 year by year. Both are reckoned beneath the
//! played people, not played, and say so.
//!
//! Read from the core's series (`ages_g00` to `ages_g16`, `ages_under15`,
//! `ages_over64`), which every game and every run recorded under the game's
//! rules since 20 November 2026 holds; a run without them draws nothing here.
//! Their figures are a whole year's, written when the year closes, so in the
//! months of a year still open the pyramid is the last year's.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use isostate_core::ages::GROUPS;

use crate::chart::{self, Lines};
use crate::replay::{self, Replay};
use crate::ui::{text, CHEAPER, DEARER, GOOD, INK, MUTED};

const CHART_W: f32 = 342.0;
/// The pyramid's own room: its plot, and the ages written to its left.
const PYRAMID_W: f32 = 290.0;
const PYRAMID_H: f32 = 136.0;
const LABEL_W: f32 = 48.0;
/// Image pixels per pixel of the interface, as the charts have.
const SS: f32 = 2.0;
const GROUND: [u8; 3] = [246, 238, 222];
const GRID: [u8; 3] = [229, 219, 198];

fn u8s(c: Color) -> [u8; 3] {
    let a = c.to_srgba().to_u8_array();
    [a[0], a[1], a[2]]
}

/// The colour of a group's bar: the children warm, the working ages cool, the old green.
fn colour(a: usize) -> [u8; 3] {
    if a < isostate_core::ages::WORKING.start {
        u8s(DEARER)
    } else if a < isostate_core::ages::OLD.start {
        u8s(CHEAPER)
    } else {
        u8s(GOOD)
    }
}

/// The island's seventeen groups in the last year at or before `k` that has
/// them all, with that year's place: None in a run without them.
pub fn pyramid(r: &Replay, k: usize) -> Option<(usize, [f32; GROUPS])> {
    let rows: Vec<Vec<f32>> = (0..GROUPS).map(|a| replay::yearly(r, &isostate_core::ages::group_name(a))).collect();
    let n = rows.iter().map(|v| v.len()).min().unwrap_or(0);
    (0..n.min(k + 1)).rev().find(|&y| rows.iter().all(|v| v[y].is_finite())).map(|y| (y, std::array::from_fn(|a| rows[a][y])))
}

/// The pyramid painted: each group a bar about a middle line, as wide as its
/// share of the people, the youngest at the foot; a faint line at every
/// twentieth year of age. Four bytes a pixel, `w` by `h`.
pub fn paint(groups: &[f32; GROUPS], w: usize, h: usize) -> Vec<u8> {
    let mut px = vec![0u8; w * h * 4];
    let mut put = |x: usize, y: usize, c: [u8; 3]| {
        let i = 4 * (y * w + x);
        px[i..i + 3].copy_from_slice(&c);
        px[i + 3] = 255;
    };
    for y in 0..h {
        for x in 0..w {
            put(x, y, GROUND);
        }
    }
    let all: f32 = groups.iter().sum();
    if !(all > 0.0) {
        return px;
    }
    // a share of a fifth reaches the edge: a young people's broadest group is under that
    let widest = groups.iter().fold(0.0f32, |m, v| m.max(*v / all)).max(0.12);
    let half = w as f32 / 2.0 - 2.0;
    let bh = h as f32 / GROUPS as f32;
    for a in 0..GROUPS {
        let top = h as f32 - (a + 1) as f32 * bh;
        let (y0, y1) = ((top + 1.0).round() as usize, ((top + bh - 1.0).round() as usize).min(h));
        let half_w = (groups[a] / all / widest * half).max(0.0);
        let (x0, x1) = (((w as f32 / 2.0) - half_w).round().max(0.0) as usize, ((w as f32 / 2.0) + half_w).round().min(w as f32) as usize);
        for y in y0..y1 {
            for x in x0..x1 {
                put(x, y, colour(a));
            }
        }
        // the grid: every twentieth year of age, under the bars' gaps
        if a % 4 == 0 && y0 > 0 {
            for x in 0..w {
                if x < x0 || x >= x1 {
                    put(x, y0 - 1, GRID);
                }
            }
        }
    }
    px
}

/// The figure for a share: per cent to one place.
fn pct(v: f32) -> String {
    format!("{:.1}%", v * 100.0)
}

/// What the print of a script says of the ages at the year `k` on the clock
/// (`ages: YEAR | under 15 .. | 15 to 64 .. | over 64 ..`), or None.
pub fn said(r: &Replay, k: usize) -> Option<String> {
    let (y, g) = pyramid(r, k)?;
    let all: f32 = g.iter().sum();
    let sh = |rg: std::ops::Range<usize>| g[rg].iter().sum::<f32>() / all;
    Some(format!(
        "ages: {} | people {:.0} | under 15 {:.4} | 15 to 64 {:.4} | over 64 {:.4}",
        r.y0 + y as i32,
        all,
        sh(isostate_core::ages::YOUNG),
        sh(isostate_core::ages::WORKING),
        sh(isostate_core::ages::OLD)
    ))
}

/// The two charts, under the island's others, at the year `k` on the clock.
pub fn spawn(p: &mut ChildSpawnerCommands, f: &Handle<Font>, images: &mut Assets<Image>, r: &Replay, k: usize) {
    let Some((y, groups)) = pyramid(r, k) else { return };
    let all: f32 = groups.iter().sum();
    let share = |rg: std::ops::Range<usize>| groups[rg].iter().sum::<f32>() / all;
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), margin: UiRect::top(Val::Px(12.0)), ..default() }).with_children(|col| {
        col.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Baseline, padding: UiRect::right(Val::Px(4.0)), ..default() }).with_children(|row| {
            row.spawn(text(f, &format!("People by age in {}", r.y0 + y as i32), 15.0, INK));
            row.spawn(text(f, "reckoned", 13.0, MUTED));
        });
        col.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(12.0), padding: UiRect::left(Val::Px(4.0)), ..default() }).with_children(|keys| {
            for (words, rg) in [("under 15", isostate_core::ages::YOUNG), ("15 to 64", isostate_core::ages::WORKING), ("over 64", isostate_core::ages::OLD)] {
                let c = colour(rg.start);
                keys.spawn(text(f, &format!("{words} {}", pct(share(rg))), 13.0, Color::srgb_u8(c[0], c[1], c[2])));
            }
        });
        let (w, h) = ((PYRAMID_W * SS) as usize, (PYRAMID_H * SS) as usize);
        let pl = chart::Plot { w, h, px: paint(&groups, w, h), range: None, range2: None };
        let handle = images.add(chart::image(&pl));
        col.spawn(Node { column_gap: Val::Px(6.0), ..default() }).with_children(|row| {
            row.spawn(Node { width: Val::Px(LABEL_W - 6.0), height: Val::Px(PYRAMID_H), flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::End, flex_shrink: 0.0, ..default() }).with_children(|ages| {
                for words in ["80+", "40", "0"] {
                    ages.spawn((text(f, words, 11.5, MUTED), TextLayout::default().with_no_wrap()));
                }
            });
            row.spawn((ImageNode { image: handle, ..default() }, Node { width: Val::Px(PYRAMID_W), height: Val::Px(PYRAMID_H), flex_shrink: 0.0, ..default() }));
        });
        // the shares under 15 and over 64, year by year
        let (young, old) = (replay::yearly(r, "ages_under15"), replay::yearly(r, "ages_over64"));
        if young.iter().any(|v| v.is_finite()) {
            let (cy, co) = (colour(0), colour(GROUPS - 1));
            col.spawn(Node { margin: UiRect::top(Val::Px(8.0)), ..default() }).with_children(|row| {
                row.spawn(text(f, "Under 15 and over 64, reckoned", 15.0, INK));
            });
            let lines: Vec<(&[f32], [u8; 3])> = vec![(&young[..], cy), (&old[..], co)];
            let c = Lines { width: CHART_W, height: 70.0, lines: &lines, mark: k, years: young.len().max(old.len()), first_year: r.y0, from_zero: true };
            chart::spawn_lines(col, f, images, &c, &pct);
        }
        col.spawn(text(f, "Reckoned beneath the people the island plays: births are what its totals need once the deaths by age are counted (docs/rules/ages.md).", 12.5, MUTED));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_broadest_group_is_the_widest_bar_and_the_old_sit_on_top() {
        let mut g = [10.0f32; GROUPS];
        g[0] = 20.0;
        g[GROUPS - 1] = 2.0;
        let (w, h) = (100, 170);
        let px = paint(&g, w, h);
        let filled = |row: usize| (0..w).filter(|&x| px[4 * (row * w + x)..4 * (row * w + x) + 3] != GROUND && px[4 * (row * w + x)..4 * (row * w + x) + 3] != GRID).count();
        // the foot (0 to 4) is the widest, the top (80 and over) the narrowest
        let (foot, mid, top) = (filled(h - 5), filled(h / 2), filled(5));
        assert!(foot > mid && mid > top && top > 0, "{foot} {mid} {top}");
        assert_eq!(&px[4 * ((h - 5) * w + w / 2)..4 * ((h - 5) * w + w / 2) + 3], &colour(0));
        assert_eq!(&px[4 * (5 * w + w / 2)..4 * (5 * w + w / 2) + 3], &colour(GROUPS - 1));
    }
}
