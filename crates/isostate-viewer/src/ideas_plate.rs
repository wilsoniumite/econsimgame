//! IDEAS-PLATE: the island's ideas as a timeline, in a sandbox, and the hand
//! on it: drag an idea to a year (the atlas's third plate in the game; the
//! designer's notes of 19 and 20 November 2026 in `docs/BACKLOG.md`: "Can I
//! see a tech timeline? Move things around in it?").
//!
//! The Idea tool's card in the tray has a chip, **The island's ideas as a
//! timeline**, which opens the panel over the left of the map, where the
//! Making panel stands; Close or Escape shuts it. One row for each idea of the
//! data the game plays (the tree's methods and techniques, those known in 1750
//! among them, and the uses of the tasks rule), in groups as a town's card
//! groups them, each group folded or opened by a click on its name; the years
//! 1750 to the run's last across. What it shows is the core's answer to
//! `inspect::Thing::Ideas` (`isostate_core::island_ideas`), asked of the game's
//! thread as every card is (`inspector::Answers`), only while the panel is open:
//! when it opens and again when a year comes or an order is answered.
//!
//! On a row: a dark red tick at history's year; for an idea found, a dot at the
//! year of its first find (a diamond in the hand's ink if the hand or a pin
//! found it) and a band for its spread from town to town, with a fine tick for
//! each town at the year it came to know it, to the year the last town
//! standing did; for one not yet found, its name greyed, a dashed line from
//! now, a pale bar to the year it is likely by where the rule's own chance
//! gives one, and a ring where its mark stands to be dragged; a pin or a hold
//! by hand ruled in the hand's ink to its year. Under the pointer the panel's
//! foot says what the idea is, what it opens, where and when it was found,
//! which towns know it and what it waits on.
//!
//! The hand: a press on the row of an idea not yet found takes its mark, and
//! the mark follows the pointer along the row; let go on a year, and the foot
//! offers what may be done there (`island_ideas::drop_choices`): found now (the
//! Idea tool's own edit, in the town a pin would choose), found in that year,
//! or held back until it. Each chip is an ordinary edit by hand, sent under the
//! tray's guards as the Idea tool's (`sandbox::send_as`), so it is kept with
//! the session, listed on the Hand tab and taken back as any edit is. A press
//! on a find, or on an idea known in 1750, says in a sentence why nothing can
//! be done. A script drags with `ideas:drag:NAME@YEAR` (script.rs).
//!
//! The view never waits: the thread makes the answer between months; the
//! layout is plain arithmetic and the chart is painted off the main thread, as
//! the Making panel's is; the names are drawn again only when their words
//! change (a find, a pin, a group folded), not every year.

// see the note at the top of ui.rs
#![allow(deprecated)]

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::tasks::{futures::check_ready, AsyncComputeTaskPool, Task};
use bevy::ui::{FocusPolicy, RelativeCursorPosition, ScrollPosition};
use isostate_core::edits::PinWhen;
use isostate_core::inspect::Thing;
use isostate_core::island_ideas::{drop_choices, IdeaRow, IslandIdeas, Offer};

use crate::game::{self, Game};
use crate::inspector::{at_present, Answers};
use crate::live::Live;
use crate::sandbox::{Sandbox, Tool, HAND_INK};
use crate::ui::{text, INK, MUTED, PAPER, RULE};
use crate::{Data, Timeline, UiFont};

/// The chip that opens the panel (the Idea tool's card, sandbox.rs).
#[derive(Component)]
pub struct OpenPlate;

/// The chip's words, which a script's `click:` finds it by.
pub const OPEN_CHIP: &str = "The island's ideas as a timeline";

#[derive(Component)]
struct ClosePlate;

#[derive(Component)]
struct PlatePanel;

/// The chart's picture, inside the list that scrolls.
#[derive(Component)]
struct PlateChart;

/// The list that scrolls.
#[derive(Component)]
struct PlateList;

/// The foot: the words under the pointer, or what a drop offers.
#[derive(Component)]
struct PlateFoot;

#[derive(Component, Clone)]
struct FoldChip(String);

/// The words of the head and the axis that follow the year, set in place each
/// year without drawing the panel again.
#[derive(Component, Clone, Copy, PartialEq)]
enum Yearly {
    Title,
    Summary,
    Now,
}

#[derive(Component, Clone)]
struct OfferChip(Offer);

#[derive(Component)]
struct CancelDrop;

/// A mark being dragged: its line and the year under the pointer.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Dragging {
    line: usize,
    year: i64,
}

/// What a drop offers: the row's label, the year, and the chips.
#[derive(Clone, Debug, PartialEq)]
struct Dropped {
    line: Option<usize>,
    label: String,
    year: i64,
    offers: Vec<Offer>,
}

/// The panel's state.
#[derive(Resource, Default)]
pub struct Plate {
    open: bool,
    folded: HashSet<String>,
    plan: Option<Arc<Plan>>,
    generation: u64,
    image: Option<Handle<Image>>,
    /// the line under the pointer, and a mark being dragged
    hover: Option<usize>,
    drag: Option<Dragging>,
    dropped: Option<Dropped>,
    /// raised whenever a group is folded or opened
    folds: u64,
    /// what the picture shows, once painted: the plan's generation and what is lit
    painted: Option<(u64, Lit)>,
    painting: Option<(u64, Lit, Task<Vec<u8>>, Instant)>,
    /// a script's step for the panel (`ideas:...`, script.rs)
    pub script: Option<String>,
    /// a script's pointer, on a row by its name, in place of the mouse
    point: Option<usize>,
    /// raised whenever the foot is to be drawn again
    foot: u64,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Plate>().add_systems(Startup, spawn_panel).add_systems(
        Update,
        (open_close, scripted, ask, draw, folds, pointer, offers, draw_foot, painter).chain().after(game::pump).run_if(resource_exists::<Sandbox>).run_if(resource_exists::<Live>),
    );
}

// ---------------------------------------------------------------- sizes

/// A line's height: a group's name or an idea's row.
pub const LINE_H: f32 = 18.0;
/// The room for the names on the left.
pub const LABEL_W: f32 = 250.0;
/// The chart's margins at each end.
const SIDE: f32 = 10.0;
/// The panel's place: where the Making panel stands (town_flow.rs).
const PANEL_LEFT: f32 = 14.0;
const PANEL_TOP: f32 = 86.0;
const PANEL_RIGHT: f32 = 14.0 + 380.0 + 12.0;
const PANEL_BOTTOM: f32 = 70.0;
const PAD: f32 = 16.0;
/// The foot's height.
const FOOT_H: f32 = 128.0;

/// The chart's width for a window of this size.
fn chart_w(win: Vec2) -> f32 {
    (win.x - PANEL_LEFT - PANEL_RIGHT - 2.0 * PAD - LABEL_W - 14.0).max(360.0)
}

// ---------------------------------------------------------------- the layout

/// A line of the panel.
#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    Group { name: String, n: usize, folded: bool },
    Row { g: usize, r: usize },
}

/// The panel laid out.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub ideas: IslandIdeas,
    pub lines: Vec<Line>,
    pub w: f32,
    pub h: f32,
    pub from: i64,
    pub to: i64,
}

impl Plan {
    /// A year's place across the chart.
    pub fn x(&self, year: f64) -> f32 {
        let span = (self.to - self.from).max(1) as f64;
        SIDE + ((year - self.from as f64) / span) as f32 * (self.w - 2.0 * SIDE)
    }

    /// The year at a place across the chart.
    pub fn year_at(&self, x: f32) -> i64 {
        let span = (self.to - self.from).max(1) as f32;
        (self.from as f32 + (x - SIDE) / (self.w - 2.0 * SIDE).max(1.0) * span).round() as i64
    }

    /// The row on a line, if the line is a row.
    pub fn row(&self, line: usize) -> Option<&IdeaRow> {
        match self.lines.get(line)? {
            Line::Row { g, r } => self.ideas.groups.get(*g)?.rows.get(*r),
            Line::Group { .. } => None,
        }
    }

    /// The line of a row whose name or label starts with `words` (a script's).
    pub fn find(&self, words: &str) -> Option<usize> {
        let w = words.trim().to_lowercase();
        (0..self.lines.len()).find(|&i| self.row(i).is_some_and(|r| r.name == w || r.label.to_lowercase().starts_with(&w)))
    }
}

/// Lay the island's ideas out on a chart `w` pixels wide, with the groups
/// named in `folded` folded to their names.
pub fn plan(ideas: &IslandIdeas, folded: &HashSet<String>, w: f32) -> Plan {
    let mut lines = Vec::new();
    for (g, grp) in ideas.groups.iter().enumerate() {
        let f = folded.contains(&grp.name);
        lines.push(Line::Group { name: grp.name.clone(), n: grp.rows.len(), folded: f });
        if !f {
            lines.extend((0..grp.rows.len()).map(|r| Line::Row { g, r }));
        }
    }
    let h = lines.len() as f32 * LINE_H;
    Plan { ideas: ideas.clone(), lines, w, h, from: ideas.from, to: ideas.end.max(ideas.from + 1) }
}

/// A row's name as the panel writes it, and whether it is greyed, inked or in
/// the hand's ink.
fn row_words(r: &IdeaRow) -> (String, u8) {
    match (r.pin, r.found) {
        (Some((y, PinWhen::In)), None) => (format!("{}  (pinned to {y})", r.label), 2),
        (Some((y, PinWhen::NotBefore)), None) => (format!("{}  (held until {y})", r.label), 2),
        (_, Some(_)) if r.at_start => (format!("{}  (known in {})", r.label, r.found.unwrap_or_default()), 3),
        (_, Some(y)) => (format!("{}  {y}", r.label), if r.by_hand { 2 } else { 0 }),
        (_, None) => (r.label.clone(), 1),
    }
}

/// What the names say, to tell whether they must be drawn again.
fn names_key(p: &Plan) -> Vec<(String, u8)> {
    (0..p.lines.len())
        .map(|i| match &p.lines[i] {
            Line::Group { name, n, folded } => (format!("{} {name}, {n}", if *folded { "+" } else { "–" }), 9),
            Line::Row { .. } => p.row(i).map(row_words).unwrap_or_default(),
        })
        .collect()
}

/// Under the pointer, or a script's pointer: the line, and the year.
pub fn hit(p: &Plan, at: Vec2) -> Option<(usize, i64)> {
    if at.x < 0.0 || at.y < 0.0 || at.x > p.w || at.y >= p.h {
        return None;
    }
    let line = (at.y / LINE_H).floor() as usize;
    (line < p.lines.len()).then(|| (line, p.year_at(at.x).clamp(p.from, p.to)))
}

// ---------------------------------------------------------------- painting

const GROUND: [f32; 3] = [252.0, 247.0, 235.0];
const GRID: [f32; 3] = [222.0, 212.0, 194.0];
const HEADER: [f32; 3] = [241.0, 233.0, 216.0];
const LIT_ROW: [f32; 3] = [246.0, 237.0, 219.0];
const BASE: [f32; 3] = [196.0, 186.0, 168.0];
const DARK: [f32; 3] = [58.0, 48.0, 40.0];
const FAINT: [f32; 3] = [150.0, 140.0, 125.0];
const SPREAD: [f32; 3] = [205.0, 170.0, 110.0];
const TOWN: [f32; 3] = [150.0, 112.0, 60.0];
const HISTORY: [f32; 3] = [160.0, 40.0, 30.0];
const LIKELY: [f32; 3] = [214.0, 205.0, 186.0];
const HAND: [f32; 3] = [0.13 * 255.0, 0.24 * 255.0, 0.45 * 255.0];

struct Canvas {
    w: usize,
    h: usize,
    px: Vec<[f32; 3]>,
}

impl Canvas {
    fn over(&mut self, x: usize, y: usize, col: [f32; 3], a: f32) {
        if x >= self.w || y >= self.h || a <= 0.0 {
            return;
        }
        let d = &mut self.px[y * self.w + x];
        for i in 0..3 {
            d[i] += (col[i] - d[i]) * a.min(1.0);
        }
    }

    /// A rectangle, its edges soft by the share of each pixel it covers.
    fn rect(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, col: [f32; 3], a: f32) {
        let (xa, xb) = (x0.floor().max(0.0) as usize, (x1.ceil().max(0.0) as usize).min(self.w));
        let (ya, yb) = (y0.floor().max(0.0) as usize, (y1.ceil().max(0.0) as usize).min(self.h));
        for y in ya..yb {
            let cy = ((y as f32 + 1.0).min(y1) - (y as f32).max(y0)).clamp(0.0, 1.0);
            for x in xa..xb {
                let cx = ((x as f32 + 1.0).min(x1) - (x as f32).max(x0)).clamp(0.0, 1.0);
                self.over(x, y, col, a * cx * cy);
            }
        }
    }

    /// A shape round (cx, cy) of radius r, by its distance function: `d(dx, dy)`
    /// is how far a point is outside it, in pixels (negative inside).
    fn shape(&mut self, cx: f32, cy: f32, r: f32, col: [f32; 3], a: f32, d: impl Fn(f32, f32) -> f32) {
        let (xa, xb) = ((cx - r - 1.0).floor().max(0.0) as usize, ((cx + r + 1.0).ceil().max(0.0) as usize).min(self.w));
        let (ya, yb) = ((cy - r - 1.0).floor().max(0.0) as usize, ((cy + r + 1.0).ceil().max(0.0) as usize).min(self.h));
        for y in ya..yb {
            for x in xa..xb {
                let k = (0.5 - d(x as f32 + 0.5 - cx, y as f32 + 0.5 - cy)).clamp(0.0, 1.0);
                self.over(x, y, col, a * k);
            }
        }
    }

    fn dot(&mut self, cx: f32, cy: f32, r: f32, col: [f32; 3], a: f32) {
        self.shape(cx, cy, r, col, a, |dx, dy| (dx * dx + dy * dy).sqrt() - r);
    }

    fn ring(&mut self, cx: f32, cy: f32, r: f32, col: [f32; 3], a: f32) {
        self.shape(cx, cy, r, col, a, |dx, dy| ((dx * dx + dy * dy).sqrt() - r).abs() - 0.7);
    }

    fn diamond(&mut self, cx: f32, cy: f32, r: f32, col: [f32; 3], a: f32) {
        self.shape(cx, cy, r, col, a, |dx, dy| (dx.abs() + dy.abs() - r) * std::f32::consts::FRAC_1_SQRT_2);
    }

    fn open_diamond(&mut self, cx: f32, cy: f32, r: f32, col: [f32; 3], a: f32) {
        self.shape(cx, cy, r, col, a, |dx, dy| ((dx.abs() + dy.abs() - r) * std::f32::consts::FRAC_1_SQRT_2).abs() - 0.7);
    }

    /// A dashed line across at height y.
    fn dashes(&mut self, x0: f32, x1: f32, y: f32, col: [f32; 3], a: f32) {
        let mut x = x0;
        while x < x1 {
            self.rect(x, y - 0.5, (x + 3.0).min(x1), y + 0.5, col, a);
            x += 6.0;
        }
    }
}

/// What the picture shows lit: the line under the pointer, and a mark being
/// dragged (its line and year).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Lit {
    pub hover: Option<usize>,
    pub drag: Option<(usize, i64)>,
}

/// The chart painted, sRGB with alpha, four bytes a pixel.
pub fn paint(p: &Plan, lit: Lit) -> (usize, usize, Vec<u8>) {
    let (w, h) = (p.w.round().max(1.0) as usize, p.h.round().max(1.0) as usize);
    let mut c = Canvas { w, h, px: vec![GROUND; w * h] };
    let now = p.ideas.now.min(p.to);
    // the decades, every fiftieth year stronger
    let mut y = p.from - p.from.rem_euclid(10);
    while y <= p.to {
        if y >= p.from {
            let x = p.x(y as f64);
            c.rect(x - 0.5, 0.0, x + 0.5, p.h, GRID, if y % 50 == 0 { 0.95 } else { 0.35 });
        }
        y += 10;
    }
    for (i, line) in p.lines.iter().enumerate() {
        let top = i as f32 * LINE_H;
        let mid = top + 0.5 * LINE_H;
        let Line::Row { .. } = line else {
            c.rect(0.0, top + 2.0, p.w, top + LINE_H - 1.0, HEADER, 1.0);
            continue;
        };
        let Some(r) = p.row(i) else { continue };
        if lit.hover == Some(i) {
            c.rect(0.0, top, p.w, top + LINE_H, LIT_ROW, 1.0);
        }
        let (x0, xn, x1) = (p.x(p.from as f64), p.x(now as f64), p.x(p.to as f64));
        if let Some(t) = r.target {
            let x = p.x(t as f64);
            c.rect(x - 0.8, top + 3.0, x + 0.8, top + LINE_H - 3.0, HISTORY, 0.9);
        }
        match r.found {
            Some(_) if r.at_start => {
                c.rect(x0, mid - 0.5, xn, mid + 0.5, BASE, 0.7);
                c.rect(x0 - 3.0, mid - 3.0, x0 + 3.0, mid + 3.0, FAINT, 0.9);
            }
            Some(y) => {
                c.rect(x0, mid - 0.5, p.x(y as f64), mid + 0.5, BASE, 0.5);
                // the spread from town to town: a band to the year the last town standing came to
                // know it (or to now, while some do not), and a fine tick for each town's year
                let to = r.spread_to.unwrap_or(now).max(y);
                if r.kind != "use" {
                    c.rect(p.x(y as f64), mid - 3.5, p.x(to as f64).max(p.x(y as f64) + 2.0), mid + 3.5, SPREAD, 0.55);
                    for (_, ty) in &r.learnt {
                        if let Some(ty) = ty {
                            let x = p.x(*ty as f64);
                            c.rect(x - 0.5, mid - 5.5, x + 0.5, mid + 5.5, TOWN, 0.55);
                        }
                    }
                } else {
                    c.rect(p.x(y as f64), mid - 0.5, xn, mid + 0.5, DARK, 0.5);
                }
                let x = p.x(y as f64);
                if r.by_hand {
                    c.diamond(x, mid, 5.0, HAND, 1.0);
                } else {
                    c.dot(x, mid, 4.0, DARK, 1.0);
                }
            }
            None => {
                c.rect(x0, mid - 0.5, xn, mid + 0.5, BASE, 0.5);
                c.dashes(xn, x1, mid, FAINT, 0.6);
                if let Some(l) = r.likely.filter(|_| r.pin.is_none()) {
                    c.rect(xn, mid - 2.0, p.x(l as f64).max(xn + 2.0), mid + 2.0, LIKELY, 0.9);
                }
                match r.pin {
                    // ruled in the hand's ink to its year: a pin to find it, a diamond; a hold, a bar and a stop
                    Some((py, PinWhen::In)) => {
                        let x = p.x(py as f64);
                        c.rect(xn, mid - 0.75, x, mid + 0.75, HAND, 0.9);
                        c.diamond(x, mid, 5.0, HAND, 1.0);
                    }
                    Some((py, PinWhen::NotBefore)) => {
                        let x = p.x(py as f64);
                        c.rect(xn, mid - 1.5, x, mid + 1.5, HAND, 0.75);
                        c.rect(x - 1.0, mid - 6.0, x + 1.0, mid + 6.0, HAND, 1.0);
                    }
                    None => {
                        if r.drag.is_ok() {
                            c.ring(p.x(r.handle as f64), mid, 4.0, FAINT, 1.0);
                        }
                    }
                }
            }
        }
        if let Some((_, y)) = lit.drag.filter(|d| d.0 == i) {
            let x = p.x(y as f64);
            let mut yy = top;
            while yy < top + LINE_H {
                c.rect(x - 0.5, yy, x + 0.5, yy + 2.0, HAND, 0.8);
                yy += 4.0;
            }
            c.open_diamond(x, mid, 6.0, HAND, 1.0);
        }
    }
    // the present, in the hand's ink: the hand works only here
    let xn = p.x(now as f64);
    c.rect(xn - 0.75, 0.0, xn + 0.75, p.h, HAND, 0.35);
    let mut out = Vec::with_capacity(w * h * 4);
    for v in &c.px {
        out.extend_from_slice(&[v[0].round().clamp(0.0, 255.0) as u8, v[1].round().clamp(0.0, 255.0) as u8, v[2].round().clamp(0.0, 255.0) as u8, 255]);
    }
    (w, h, out)
}

fn image(w: usize, h: usize, px: Vec<u8>) -> Image {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let mut img = Image::new(Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: 1 }, TextureDimension::D2, px, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD);
    img.sampler = bevy::image::ImageSampler::linear();
    img
}

// ---------------------------------------------------------------- the systems

fn spawn_panel(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(PANEL_LEFT),
            top: Val::Px(PANEL_TOP),
            right: Val::Px(PANEL_RIGHT),
            bottom: Val::Px(PANEL_BOTTOM),
            padding: UiRect::all(Val::Px(PAD)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            border_radius: BorderRadius::all(Val::Px(16.0)),
            ..default()
        },
        BackgroundColor(PAPER.with_alpha(1.0)),
        BoxShadow::new(Color::srgba(0.25, 0.18, 0.10, 0.18), Val::Px(0.0), Val::Px(3.0), Val::Px(0.0), Val::Px(10.0)),
        Interaction::default(),
        RelativeCursorPosition::default(),
        FocusPolicy::Block,
        GlobalZIndex(12),
        Visibility::Hidden,
        PlatePanel,
    ));
}

/// Open from the Idea tool's chip; shut with Close or Escape.
fn open_close(open: Query<&Interaction, (Changed<Interaction>, With<OpenPlate>)>, close: Query<&Interaction, (Changed<Interaction>, With<ClosePlate>)>, keys: Res<ButtonInput<KeyCode>>, mut plate: ResMut<Plate>) {
    if open.iter().any(|i| *i == Interaction::Pressed) {
        plate.open = true;
    }
    if close.iter().any(|i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Escape) {
        if plate.dropped.is_some() && !close.iter().any(|i| *i == Interaction::Pressed) {
            // (Escape first puts a drop's offers away)
            plate.dropped = None;
            plate.foot += 1;
        } else {
            plate.open = false;
            (plate.drag, plate.dropped) = (None, None);
        }
    }
}

/// A script's step for the panel: `open`, `shut`, `fold:GROUP`, `unfold:GROUP`,
/// `point:NAME` (`point:none`), `show:NAME` (the list scrolled to its row) and
/// `drag:NAME@YEAR` (its mark dropped on the year, as a drag would).
fn scripted(mut plate: ResMut<Plate>, mut list: Query<&mut ScrollPosition, With<PlateList>>) {
    let Some(step) = plate.script.take() else { return };
    let (what, arg) = step.split_once(':').map_or((step.as_str(), ""), |(a, b)| (a, b));
    let (what, arg) = (what.trim().to_string(), arg.trim().to_string());
    match what.as_str() {
        "open" => plate.open = true,
        "shut" => plate.open = false,
        "fold" | "unfold" => {
            let Some(pl) = plate.plan.clone() else {
                println!("ideas: the panel is not drawn yet");
                return;
            };
            let name = pl.ideas.groups.iter().map(|g| g.name.clone()).find(|n| n.to_lowercase().starts_with(&arg.to_lowercase()) || arg == "all");
            let names: Vec<String> = if arg == "all" { pl.ideas.groups.iter().map(|g| g.name.clone()).collect() } else { name.into_iter().collect() };
            plate.folds += 1;
            for n in names {
                if what == "fold" {
                    plate.folded.insert(n);
                } else {
                    plate.folded.remove(&n);
                }
            }
        }
        "point" | "show" | "drag" => {
            let Some(pl) = plate.plan.clone() else {
                println!("ideas: the panel is not drawn yet");
                return;
            };
            if arg.eq_ignore_ascii_case("none") {
                plate.point = None;
                return;
            }
            let (name, year) = match arg.rsplit_once('@') {
                Some((n, y)) => (n.to_string(), y.trim().parse::<i64>().ok()),
                None => (arg.clone(), None),
            };
            let Some(line) = pl.find(&name) else {
                println!("ideas: no row for {name} (folded, or no such idea)");
                return;
            };
            if what == "show" {
                if let Ok(mut s) = list.single_mut() {
                    s.0.y = (line as f32 * LINE_H - 3.0 * LINE_H).max(0.0);
                }
                return;
            }
            plate.point = Some(line);
            if what == "drag" {
                let row = pl.row(line).expect("a row");
                let year = year.unwrap_or(pl.ideas.now);
                plate.drag = None;
                drop_on(&mut plate, row, year.clamp(pl.ideas.now, pl.to), pl.ideas.end);
            }
        }
        other => println!("ideas: no such step {other:?}"),
    }
    plate.foot += 1;
}

/// A row's mark let go on a year: what may be done there, or why nothing.
fn drop_on(plate: &mut Plate, row: &IdeaRow, year: i64, end: i64) {
    match drop_choices(row, year, end) {
        Ok(offers) => {
            println!("ideas: {} dropped on {year}: {}", row.label, offers.iter().map(|o| o.label.as_str()).collect::<Vec<_>>().join(", "));
            plate.dropped = Some(Dropped { line: plate.point, label: row.label.clone(), year, offers });
        }
        Err(why) => {
            println!("ideas: {} dropped on {year}: {why}", row.label);
            plate.dropped = Some(Dropped { line: plate.point, label: row.label.clone(), year, offers: Vec::new() });
            // (the sentence stands in the foot)
            plate.refusal(why);
        }
    }
    plate.foot += 1;
}

impl Plate {
    fn refusal(&mut self, why: String) {
        if let Some(d) = self.dropped.as_mut() {
            d.offers.clear();
            d.label = format!("{}: {why}", d.label);
        }
    }
}

/// Ask for the island's ideas when the panel opens, and again when a year
/// comes or an order is answered; not while the year turns, and only with
/// the clock at the present.
fn ask(plate: Res<Plate>, tl: Res<Timeline>, data: Res<Data>, live: Res<Live>, game: Res<Game>, mut answers: ResMut<Answers>, mut seen: Local<Option<(usize, u64)>>) {
    if !plate.open {
        *seen = None;
        return;
    }
    let key = (data.r.filled, game.answered);
    if *seen == Some(key) || live.turning() || !at_present(&tl, &data, &live) {
        return;
    }
    *seen = Some(key);
    answers.ask(&live, Thing::Ideas);
}

/// The panel drawn again: wholly when it opens or shuts, its names change or
/// the window changes size; otherwise only its picture, painted afresh.
#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    font: Res<UiFont>,
    answers: Res<Answers>,
    tl: Res<Timeline>,
    data: Res<Data>,
    live: Res<Live>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut plate: ResMut<Plate>,
    mut images: ResMut<Assets<Image>>,
    mut q: Query<(Entity, &mut Visibility, Option<&Children>), With<PlatePanel>>,
    chart: Query<Entity, With<PlateChart>>,
    mut yearly: Query<(&Yearly, &mut Text, &mut Node), Without<PlatePanel>>,
    mut seen: Local<Option<(u64, bool, u32, u64, bool)>>,
    mut drawn: Local<Option<(Vec<(String, u8)>, u32, bool)>>,
    mut shown: Local<Option<IslandIdeas>>,
) {
    let win = windows.single().map(|w| Vec2::new(w.width(), w.height())).unwrap_or(Vec2::new(1600.0, 1000.0));
    let present = at_present(&tl, &data, &live);
    let came = (answers.version, plate.open, win.x as u32, plate.folds, present);
    if *seen == Some(came) {
        return;
    }
    *seen = Some(came);
    let Ok((e, mut vis, children)) = q.single_mut() else { return };
    let told = answers.get(Thing::Ideas).and_then(|t| t.as_ref().ok()).and_then(|t| t.timeline.clone());
    if !plate.open {
        if *vis != Visibility::Hidden {
            *vis = Visibility::Hidden;
        }
        *drawn = None;
        *shown = None;
        (plate.plan, plate.image, plate.painted, plate.painting, plate.hover) = (None, None, None, None, None);
        return;
    }
    *vis = Visibility::Inherited;
    let f = font.0.clone();
    let Some(ideas) = told else {
        if drawn.is_some() || children.is_none() {
            if let Some(ch) = children {
                for c in ch.iter() {
                    commands.entity(c).despawn();
                }
            }
            commands.entity(e).with_children(|p| {
                head(p, &f, None, true);
                p.spawn(text(&f, "Asking the simulation...", 13.0, MUTED));
            });
            *drawn = None;
        }
        return;
    };
    let t0 = Instant::now();
    let w = chart_w(win).round();
    let pl = plan(&ideas, &plate.folded, w);
    let key = (names_key(&pl), win.x as u32, present);
    let whole = drawn.as_ref() != Some(&key) || chart.is_empty();
    let size = (pl.w.round().max(1.0) as usize, pl.h.round().max(1.0) as usize);
    let same_size = plate.plan.as_ref().is_some_and(|old| (old.w.round() as usize, old.h.round() as usize) == size);
    if whole {
        if let Some(ch) = children {
            for c in ch.iter() {
                commands.entity(c).despawn();
            }
        }
        let handle = images.add(image(size.0, size.1, [GROUND[0] as u8, GROUND[1] as u8, GROUND[2] as u8, 255].repeat(size.0 * size.1)));
        plate.image = Some(handle.clone());
        plate.painted = None;
        let panel = e;
        commands.entity(e).with_children(|p| {
            head(p, &f, Some(&ideas), present);
            axis(p, &f, &pl);
            crate::ui::scroll_list(
                p,
                panel,
                Node { flex_grow: 1.0, flex_shrink: 1.0, min_height: Val::Px(60.0), ..default() },
                Node { flex_grow: 1.0, padding: UiRect::right(Val::Px(10.0)), ..default() },
                PlateList,
                |rows| {
                    rows.spawn(Node { width: Val::Px(LABEL_W + pl.w), height: Val::Px(pl.h), flex_shrink: 0.0, ..default() }).with_children(|c| {
                        names(c, &f, &pl);
                        c.spawn((
                            Node { position_type: PositionType::Absolute, left: Val::Px(LABEL_W), top: Val::Px(0.0), width: Val::Px(pl.w), height: Val::Px(pl.h), ..default() },
                            ImageNode { image: handle.clone(), ..default() },
                            Interaction::default(),
                            RelativeCursorPosition::default(),
                            PlateChart,
                        ));
                    });
                },
            );
            p.spawn((Node { height: Val::Px(FOOT_H), flex_shrink: 0.0, flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), padding: UiRect::top(Val::Px(6.0)), border: UiRect::top(Val::Px(1.0)), ..default() }, BorderColor::all(RULE), PlateFoot));
        });
        *drawn = Some(key);
    } else if !same_size {
        if let Some(mut h) = plate.image.as_ref().and_then(|h| images.get_mut(h)) {
            *h = image(size.0, size.1, [GROUND[0] as u8, GROUND[1] as u8, GROUND[2] as u8, 255].repeat(size.0 * size.1));
        }
        plate.painted = None;
    }
    if !whole {
        // the head and the present's place follow the year in place
        for (y, mut t, mut node) in &mut yearly {
            match y {
                Yearly::Title => t.0 = title_words(&ideas),
                Yearly::Summary => t.0 = summary_words(&ideas),
                Yearly::Now => {
                    t.0 = format!("now, {}", ideas.now);
                    node.left = Val::Px(LABEL_W + pl.x(ideas.now.min(pl.to) as f64) + 3.0);
                }
            }
        }
    }
    let found_now = shown.as_ref().map_or(0, |s| s.rows().filter(|r| r.found.is_some()).count());
    println!(
        "ideas: {} rows in {} groups in {}, {} found ({} new); laid out in {:.2} ms{}",
        ideas.rows().count(),
        ideas.groups.len(),
        ideas.now,
        ideas.rows().filter(|r| r.found.is_some()).count(),
        ideas.rows().filter(|r| r.found.is_some()).count().saturating_sub(found_now),
        1e3 * t0.elapsed().as_secs_f64(),
        if whole { ", the names drawn again" } else { "" }
    );
    *shown = Some(ideas);
    plate.plan = Some(Arc::new(pl));
    plate.generation += 1;
    plate.foot += 1;
    plate.hover = None;
    if let (Some(d), Some(pl)) = (plate.drag, plate.plan.clone()) {
        if pl.row(d.line).is_none_or(|r| r.drag.is_err()) {
            plate.drag = None;
        }
    }
}

fn title_words(i: &IslandIdeas) -> String {
    format!("The island's ideas, {} to {}, as they stand in {}", i.from, i.end, i.now)
}

fn summary_words(i: &IslandIdeas) -> String {
    let found = i.rows().filter(|r| r.found.is_some() && !r.at_start).count();
    let start = i.rows().filter(|r| r.at_start).count();
    let waiting = i.rows().filter(|r| r.found.is_none()).count();
    format!(
        "{} ideas: {start} known in {}, {found} found since, {waiting} not yet found. A dot is a find, its band the spread from town to town; the red tick is our own history's year; a diamond and a rule in the hand's ink are the hand's. Drag a greyed idea's mark along its row to a year.",
        i.rows().count(),
        i.from
    )
}

/// The panel's title, the line on what it shows, and its Close.
fn head(p: &mut ChildSpawnerCommands, f: &Handle<Font>, ideas: Option<&IslandIdeas>, present: bool) {
    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
        row.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|col| {
            match ideas {
                Some(i) => {
                    col.spawn((Node::default(), text(f, &title_words(i), 20.0, INK), Yearly::Title));
                    col.spawn((Node::default(), text(f, &summary_words(i), 11.5, MUTED), Yearly::Summary));
                }
                None => {
                    col.spawn(text(f, "The island's ideas", 20.0, INK));
                }
            }
            if !present {
                col.spawn(text(f, "The clock is looking back: the panel is of the present, and the hand works only there.", 11.5, MUTED));
            }
        });
        crate::family::chip_button(row, f, "Close", false, INK, ClosePlate);
    });
}

/// The years across, over the chart.
fn axis(p: &mut ChildSpawnerCommands, f: &Handle<Font>, pl: &Plan) {
    p.spawn(Node { height: Val::Px(28.0), flex_shrink: 0.0, ..default() }).with_children(|a| {
        let mut y = pl.from - pl.from.rem_euclid(50);
        while y <= pl.to {
            if y >= pl.from {
                let x = LABEL_W + pl.x(y as f64);
                a.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(x - 14.0), top: Val::Px(0.0), ..default() }, text(f, &y.to_string(), 11.0, MUTED)));
            }
            y += 50;
        }
        let xn = LABEL_W + pl.x(pl.ideas.now.min(pl.to) as f64);
        // (under the years, so that it never stands on one of them)
        a.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(xn + 3.0), top: Val::Px(13.0), ..default() }, text(f, &format!("now, {}", pl.ideas.now), 11.0, HAND_INK), Yearly::Now));
    });
}

/// A name shortened to about `room` pixels of the interface's small type.
fn fit(s: &str, room: f32) -> String {
    let most = ((room / 6.0).floor() as usize).max(4);
    if s.chars().count() <= most {
        return s.to_string();
    }
    let cut: String = s.chars().take(most - 1).collect();
    format!("{}…", cut.trim_end())
}

/// Each line's name on the left: a group's as a chip that folds it, a row's
/// in ink, greyed, or in the hand's ink.
fn names(c: &mut ChildSpawnerCommands, f: &Handle<Font>, p: &Plan) {
    for (i, (words, how)) in names_key(p).into_iter().enumerate() {
        let top = i as f32 * LINE_H;
        if let Some(Line::Group { name, .. }) = p.lines.get(i) {
            c.spawn((
                Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(top + 1.0), height: Val::Px(LINE_H - 2.0), padding: UiRect::horizontal(Val::Px(4.0)), align_items: AlignItems::Center, ..default() },
                Interaction::default(),
                FocusPolicy::Block,
                crate::ui::Hoverable,
                FoldChip(name.clone()),
            ))
            .with_children(|t| {
                t.spawn((text(f, &words, 12.0, INK), TextLayout::default().with_no_wrap()));
            });
            continue;
        }
        let ink = match how {
            0 => INK,
            1 => MUTED.with_alpha(0.8),
            2 => HAND_INK,
            _ => MUTED.with_alpha(0.6),
        };
        c.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(14.0), top: Val::Px(top + 2.0), ..default() }, FocusPolicy::Pass)).with_children(|t| {
            t.spawn((text(f, &fit(&words, LABEL_W - 20.0), 11.5, ink), TextLayout::default().with_no_wrap()));
        });
    }
}

/// A click on a group's name folds or opens it.
fn folds(q: Query<(&Interaction, &FoldChip), Changed<Interaction>>, mut plate: ResMut<Plate>) {
    for (i, chip) in &q {
        if *i == Interaction::Pressed {
            if !plate.folded.remove(&chip.0) {
                plate.folded.insert(chip.0.clone());
            }
            plate.folds += 1;
        }
    }
}

/// The pointer on the chart: the line under it lit and its words in the foot;
/// a press on a row takes its mark (or says why nothing can be done), the mark
/// follows the pointer along the row, and let go it offers what may be done.
#[allow(clippy::too_many_arguments)]
fn pointer(
    mut plate: ResMut<Plate>,
    buttons: Res<ButtonInput<MouseButton>>,
    chart: Query<(&RelativeCursorPosition, &ComputedNode), With<PlateChart>>,
    panel: Query<&RelativeCursorPosition, With<PlatePanel>>,
    mut toasts: ResMut<crate::Toasts>,
    time: Res<Time>,
) {
    let Some(pl) = plate.plan.clone() else { return };
    let over_panel = panel.single().is_ok_and(|r| r.cursor_over());
    let at = chart.single().ok().and_then(|(rel, node)| {
        let n = rel.normalized?;
        let size = node.size() * node.inverse_scale_factor();
        Some((n + Vec2::splat(0.5)) * size)
    });
    let under = at.filter(|_| over_panel).and_then(|a| hit(&pl, a));
    // a drag goes on wherever the pointer goes, along its own row
    if let Some(mut d) = plate.drag {
        if let Some(a) = at {
            let y = pl.year_at(a.x).clamp(pl.ideas.now, pl.to);
            if y != d.year {
                d.year = y;
                plate.drag = Some(d);
                plate.foot += 1;
            }
        }
        if buttons.just_released(MouseButton::Left) {
            plate.drag = None;
            if let Some(row) = pl.row(d.line) {
                let row = row.clone();
                plate.point = Some(d.line);
                drop_on(&mut plate, &row, d.year, pl.ideas.end);
                plate.point = None;
            }
        }
        return;
    }
    let line = under.map(|u| u.0).or(plate.point);
    if line != plate.hover {
        plate.hover = line;
        plate.foot += 1;
    }
    if buttons.just_pressed(MouseButton::Left) {
        if let Some((line, year)) = under {
            if let Some(row) = pl.row(line) {
                plate.point = None;
                match &row.drag {
                    Ok(_) => {
                        plate.dropped = None;
                        plate.drag = Some(Dragging { line, year: year.clamp(pl.ideas.now, pl.to) });
                        plate.foot += 1;
                    }
                    Err(why) => {
                        toasts.say("orders_refused", why, "By hand", time.elapsed_secs());
                        println!("ideas: {why}");
                    }
                }
            }
        }
    }
}

/// A chip of a drop pressed: its edit sent under the tray's guards, as the
/// Idea tool's; Cancel puts the offers away.
#[allow(clippy::too_many_arguments)]
fn offers(
    mut plate: ResMut<Plate>,
    chips: Query<(&Interaction, &OfferChip), Changed<Interaction>>,
    cancel: Query<&Interaction, (Changed<Interaction>, With<CancelDrop>)>,
    mut sb: ResMut<Sandbox>,
    tl: Res<Timeline>,
    data: Res<Data>,
    live: Res<Live>,
    mut game: ResMut<Game>,
    mut toasts: ResMut<crate::Toasts>,
    time: Res<Time>,
) {
    if cancel.iter().any(|i| *i == Interaction::Pressed) {
        plate.dropped = None;
        plate.foot += 1;
    }
    for (i, chip) in &chips {
        if *i != Interaction::Pressed {
            continue;
        }
        match crate::sandbox::send_as(Tool::Idea, chip.0.edit.clone(), &mut sb, &tl, &data, &live, &mut game) {
            Ok(()) => {
                println!("ideas: by hand from the timeline: {}", serde_json::to_string(&chip.0.edit).unwrap_or_default());
                plate.dropped = None;
                plate.foot += 1;
            }
            Err(why) => {
                toasts.say("orders_refused", why, "By hand", time.elapsed_secs());
                println!("ideas: refused: {why}");
            }
        }
    }
}

/// The foot: what a drop offers, or where a mark being dragged would go, or
/// the words of the row under the pointer.
fn draw_foot(mut commands: Commands, font: Res<UiFont>, plate: Res<Plate>, q: Query<(Entity, Option<&Children>), With<PlateFoot>>, mut drawn: Local<Option<(u64, u64)>>) {
    let Ok((e, children)) = q.single() else { return };
    let now = (plate.generation, plate.foot);
    if *drawn == Some(now) {
        return;
    }
    *drawn = Some(now);
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    let Some(pl) = plate.plan.clone() else { return };
    let f = font.0.clone();
    commands.entity(e).with_children(|foot| {
        if let Some(d) = &plate.dropped {
            if d.offers.is_empty() {
                foot.spawn(text(&f, &d.label, 12.5, INK));
                foot.spawn(Node::default()).with_children(|r| crate::family::chip_button(r, &f, "Cancel", false, INK, CancelDrop));
                return;
            }
            foot.spawn(text(&f, &format!("By hand: {}, to {}", d.label, d.year), 14.0, HAND_INK));
            for o in &d.offers {
                foot.spawn(Node { column_gap: Val::Px(8.0), align_items: AlignItems::Center, ..default() }).with_children(|r| {
                    crate::family::chip_button(r, &f, &o.label, false, HAND_INK, OfferChip(o.clone()));
                    r.spawn(text(&f, &o.line, 11.5, INK));
                });
            }
            foot.spawn(Node::default()).with_children(|r| crate::family::chip_button(r, &f, "Cancel", false, INK, CancelDrop));
            return;
        }
        if let Some(d) = plate.drag {
            let label = pl.row(d.line).map_or(String::new(), |r| r.label.clone());
            foot.spawn(text(&f, &format!("{label}, to {}", d.year), 14.0, HAND_INK));
            foot.spawn(text(&f, "Let go on a year: the foot then offers what may be done there, and nothing is sent until a chip is pressed.", 11.5, MUTED));
            return;
        }
        match plate.hover.and_then(|l| pl.row(l)) {
            Some(r) => {
                for (n, l) in r.said.iter().enumerate() {
                    foot.spawn(text(&f, l, if n == 0 { 13.0 } else { 11.5 }, if n == 0 { INK } else { MUTED }));
                }
            }
            None => {
                foot.spawn(text(&f, "Rest the pointer on a row: what the idea is, what it opens, where and when it was found, which towns know it, and what it waits on.", 11.5, MUTED));
            }
        }
    });
    if let Some(r) = plate.hover.and_then(|l| pl.row(l)) {
        println!("ideas: under the pointer: {}", r.said.join(" | "));
    }
}

/// The chart is painted off the main thread, so that the view never waits
/// for it, and laid into its image when it comes, if the plan it was made
/// from is still the one drawn.
fn painter(mut plate: ResMut<Plate>, mut images: ResMut<Assets<Image>>) {
    if let Some((generation, lit, task, t0)) = plate.painting.as_mut() {
        let Some(px) = check_ready(task) else { return };
        let (generation, lit, ms) = (*generation, *lit, t0.elapsed().as_secs_f64() * 1e3);
        plate.painting = None;
        if generation != plate.generation {
            return;
        }
        if let Some(mut img) = plate.image.as_ref().and_then(|h| images.get_mut(h)) {
            img.data = Some(px);
        }
        plate.painted = Some((generation, lit));
        println!("ideas: painted in {ms:.1} ms, off the main thread");
    }
    let Some(pl) = plate.plan.clone() else { return };
    // (a mark dropped stands where it was let go while its offers are out)
    let ghost = plate.drag.map(|d| (d.line, d.year)).or(plate.dropped.as_ref().and_then(|d| d.line.map(|l| (l, d.year))));
    let lit = Lit { hover: plate.hover, drag: ghost };
    if plate.painted == Some((plate.generation, lit)) || plate.painting.is_some() {
        return;
    }
    let task = AsyncComputeTaskPool::get().spawn(async move { paint(&pl, lit).2 });
    plate.painting = Some((plate.generation, lit, task, Instant::now()));
}
