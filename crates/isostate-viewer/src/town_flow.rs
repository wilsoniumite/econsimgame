//! A town's making as a panel, in a sandbox: boxes for its works and the
//! land round it, its households and the world outside the town, and ribbons
//! between them as wide as the value of the goods that went, coloured by good
//! (the designer's notes of 19 and 20 November 2026 in `docs/BACKLOG.md`:
//! "Can I see a production flowchart?", "per town, with scaled flows and
//! boxes").
//!
//! A town's card has a chip, Making; it opens the panel over the left of the
//! map. The panel follows the town picked (another town's card opened shows
//! that town's making) and closes with its own Close, with Escape, or when
//! nothing is picked. What it shows is the core's answer to
//! `inspect::Thing::Making` (`isostate_core::making`), asked of the game's
//! thread as every card is (`inspector::Answers`), and only while the panel
//! is open: once when it opens and again when a year comes or an order is
//! answered, since the books it reads close once a year. The thread makes the
//! answer between months in about half a millisecond for a town of forty
//! thousand people. The view never waits: the panel is laid out and painted
//! only when the answer differs from what is drawn or the window changes size,
//! and painted again only when the pointer moves onto another box or ribbon.
//!
//! The drawing is plain arithmetic on pixels (`plan`, `paint`), so it can be
//! tested without a window: the columns the core gave, each box as tall as the
//! value through it and as wide as its hands (a square root), the boxes of a
//! column ordered so that ribbons cross little (a few passes of each box to
//! the mean height of the boxes it trades with), and each ribbon leaving its
//! box's right side and entering the next's left in the order of where it
//! goes, so that a box's ribbons lie side by side. Each good has the colour the
//! data gives it (`data/content.toml`, `data/game.toml`), which the atlas's
//! page uses too; a pale one is darkened a little so that it shows on paper.
//! Kinds of works the town knows the idea of and has none of are faint boxes
//! with a dashed rule, under the others in the column the tree gives them.
//!
//! Under the pointer a box says what it is (its works, hands, what they made
//! and what it sold for, each method's recipe, and the methods known here and
//! used by none) and a ribbon says its good, its amount, its price here and
//! what it cost to carry; the words are the core's (`Node::said`,
//! `Ribbon::said`). A script points with `point:NAME` (script.rs).

// see the note at the top of ui.rs
#![allow(deprecated)]

use std::time::Instant;

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use isostate_core::inspect::Thing;
use isostate_core::making::{Kind, Making};

use crate::game::{self, Game};
use crate::inspector::{at_present, thing_of, Answers};
use crate::live::Live;
use crate::sandbox::Sandbox;
use crate::ui::{text, INK, MUTED, PAPER, RULE};
use crate::{Data, Picked, Timeline, UiFont};

/// Where the chip that opens the panel stands in a town's card (main.rs).
#[derive(Component)]
pub struct MakingBlock;

#[derive(Component)]
struct OpenMaking;

#[derive(Component)]
struct CloseMaking;

/// The panel.
#[derive(Component)]
struct FlowPanel;

/// The chart's picture.
#[derive(Component)]
struct FlowChart;

/// The words under the pointer.
#[derive(Component)]
struct FlowTip;

/// The panel's state: the town shown, the layout drawn and what is under
/// the pointer.
#[derive(Resource, Default)]
pub struct Flow {
    /// the town whose making is shown, while the panel is open
    town: Option<usize>,
    plan: Option<Plan>,
    image: Option<Handle<Image>>,
    hit: Option<Hit>,
    /// a script's pointer: a box by the start of its name, or a ribbon as
    /// GOOD>BOX (the good's name, and the start of the name of the box it
    /// goes to), in place of the mouse
    pub point: Option<String>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Flow>().add_systems(Startup, spawn_panel).add_systems(
        Update,
        (card_chip, open_close, ask, draw, hover).chain().after(game::pump).run_if(resource_exists::<Sandbox>).run_if(resource_exists::<Live>),
    );
}

// ---------------------------------------------------------------- sizes

/// Room above each box for its name.
const LABEL: f32 = 17.0;
/// Room between a box and the name of the next under it.
const GAP: f32 = 5.0;
/// Margins of the chart.
const TOP: f32 = 22.0;
const BOTTOM: f32 = 8.0;
const SIDE: f32 = 6.0;
/// The widest a box may be, and the narrowest; a box with no hands of its own.
const BOX_MAX: f32 = 34.0;
const BOX_MIN: f32 = 5.0;
const BOX_PLAIN: f32 = 10.0;
/// A faint kind's height.
const FAINT_H: f32 = 9.0;
/// The panel's place on the window: clear of the bar across the top, the
/// timeline under it and the card on the right (main.rs's `INSPECT_W`).
const PANEL_LEFT: f32 = 14.0;
const PANEL_TOP: f32 = 86.0;
const PANEL_RIGHT: f32 = 14.0 + 380.0 + 12.0;
const PANEL_BOTTOM: f32 = 70.0;
const PAD: f32 = 16.0;
/// The panel's own lines above and below the chart: the title, the line on
/// what the chart is, and the key of the goods.
const HEAD: f32 = 74.0;
const FOOT: f32 = 30.0;

/// The chart's size for a window of this size, in pixels of the interface.
fn chart_size(win: Vec2) -> Vec2 {
    Vec2::new((win.x - PANEL_LEFT - PANEL_RIGHT - 2.0 * PAD).max(420.0), (win.y - PANEL_TOP - PANEL_BOTTOM - 2.0 * PAD - HEAD - FOOT).max(240.0))
}

// ---------------------------------------------------------------- the layout

/// Where a box stands on the chart.
#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Place {
    fn centre(&self) -> f32 {
        self.y + 0.5 * self.h
    }
    fn holds(&self, p: Vec2, slack: f32) -> bool {
        p.x >= self.x - slack && p.x <= self.x + self.w + slack && p.y >= self.y - slack && p.y <= self.y + self.h + slack
    }
}

/// A ribbon as drawn: from the right side of one box at `y0` to the left
/// side of another at `y1`, `t` pixels thick.
#[derive(Clone, Debug, PartialEq)]
pub struct Band {
    pub ribbon: usize,
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub t: f32,
    pub colour: [f32; 3],
    pub back: bool,
}

impl Band {
    /// The ribbon's top at `x`: an easy S from one end to the other.
    fn top(&self, x: f32) -> f32 {
        let s = ((x - self.x0) / (self.x1 - self.x0)).clamp(0.0, 1.0);
        let s = s * s * (3.0 - 2.0 * s);
        self.y0 + (self.y1 - self.y0) * s
    }
}

/// The chart laid out.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub w: f32,
    pub h: f32,
    pub making: Making,
    /// one per box of the making, in its order
    pub boxes: Vec<Place>,
    /// one per faint kind; None where its column had no room
    pub faint: Vec<Option<Place>>,
    pub bands: Vec<Band>,
    /// pixels a unit of value
    pub scale: f32,
}

/// What the pointer is on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    Box(usize),
    Faint(usize),
    Ribbon(usize),
}

/// A good's colour as the data writes it ("#d6b85a"), 0 to 255, a pale one
/// darkened so that it shows on the chart's paper.
pub fn good_colour(hex: &str) -> [f32; 3] {
    let h = hex.trim_start_matches('#');
    let c = |i: usize| h.get(i..i + 2).and_then(|s| u8::from_str_radix(s, 16).ok()).unwrap_or(128) as f32;
    let mut rgb = [c(0), c(2), c(4)];
    let light = 0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2];
    if light > 190.0 {
        let k = 190.0 / light;
        for v in rgb.iter_mut() {
            *v *= k;
        }
    }
    rgb
}

/// The colour of a box, by what it stands for.
fn box_colour(k: Kind) -> [f32; 3] {
    match k {
        Kind::Land | Kind::OtherLand => [104.0, 128.0, 78.0],
        Kind::Works | Kind::OtherWorks => [86.0, 72.0, 60.0],
        Kind::Households => [176.0, 104.0, 70.0],
        Kind::FromTowns | Kind::ToTowns => [88.0, 112.0, 140.0],
        Kind::LandBuys | Kind::Building => [140.0, 128.0, 112.0],
        Kind::FromStore | Kind::ToStore => [190.0, 182.0, 168.0],
    }
}

/// Lay a making out on a chart `w` by `h` pixels.
pub fn plan(m: &Making, w: f32, h: f32) -> Plan {
    let n = m.nodes.len();
    let cols = m.cols.max(2);
    // the columns' places, the widest box room at each
    let step = (w - 2.0 * SIDE - BOX_MAX) / (cols - 1) as f32;
    let col_x = |c: usize| SIDE + c as f32 * step;
    let most_hands = m.nodes.iter().filter(|x| x.kind.makes()).map(|x| x.hands).fold(0.0f64, f64::max).max(1.0);
    let width = |i: usize| {
        let x = &m.nodes[i];
        if x.kind.makes() { (BOX_MAX * (x.hands / most_hands).sqrt() as f32).clamp(BOX_MIN, BOX_MAX) } else { BOX_PLAIN }
    };
    let in_col: Vec<Vec<usize>> = (0..cols).map(|c| (0..n).filter(|&i| m.nodes[i].col == c).collect()).collect();
    let faint_in: Vec<usize> = (0..cols).map(|c| m.faint.iter().filter(|f| f.col == c).count()).collect();
    // one scale for all: the fullest column fills the height
    let mut scale = f32::INFINITY;
    for c in 0..cols {
        let total: f64 = in_col[c].iter().map(|&i| m.nodes[i].through()).sum();
        if total <= 0.0 {
            continue;
        }
        let room = h - TOP - BOTTOM - in_col[c].len() as f32 * (LABEL + GAP + 2.0) - faint_in[c].min(3) as f32 * (FAINT_H + LABEL + GAP);
        scale = scale.min(room.max(20.0) / total as f32);
    }
    if !scale.is_finite() {
        scale = 0.0;
    }
    let height = |i: usize| (m.nodes[i].through() as f32 * scale).max(2.0);
    let mut boxes: Vec<Place> = (0..n).map(|i| Place { x: col_x(m.nodes[i].col) + 0.5 * (BOX_MAX - width(i)), y: 0.0, w: width(i), h: height(i) }).collect();
    // the sinks stand at the right edge, the sources at the left
    for i in 0..n {
        if m.nodes[i].col == cols - 1 {
            boxes[i].x = w - SIDE - boxes[i].w;
        } else if m.nodes[i].col == 0 {
            boxes[i].x = SIDE;
        }
    }
    // stack each column in an order, centred in the height
    let mut order = in_col.clone();
    let stack = |order: &Vec<Vec<usize>>, boxes: &mut Vec<Place>| {
        for col in order {
            let used: f32 = col.iter().map(|&i| boxes[i].h + LABEL + GAP).sum();
            let mut y = TOP + (0.5 * (h - TOP - BOTTOM - used)).max(0.0);
            for &i in col {
                y += LABEL;
                boxes[i].y = y;
                y += boxes[i].h + GAP;
            }
        }
    };
    stack(&order, &mut boxes);
    // a few passes: each box to the mean height of the boxes it trades with
    // on the side the pass comes from, weighted by value
    let mut links: Vec<(usize, usize, f64)> = Vec::new();
    for r in m.ribbons.iter().filter(|r| r.from != r.to) {
        links.push((r.from, r.to, r.value));
    }
    for pass in 0..8 {
        let forward = pass % 2 == 0;
        let cs: Vec<usize> = if forward { (1..cols).collect() } else { (0..cols - 1).rev().collect() };
        for c in cs {
            let mut keyed: Vec<(f32, usize)> = order[c]
                .iter()
                .map(|&i| {
                    let (mut s, mut v) = (0.0f64, 0.0f64);
                    for &(a, b, val) in &links {
                        let other = if b == i && forward && m.nodes[a].col < c {
                            a
                        } else if a == i && !forward && m.nodes[b].col > c {
                            b
                        } else {
                            continue;
                        };
                        s += val * boxes[other].centre() as f64;
                        v += val;
                    }
                    (if v > 0.0 { (s / v) as f32 } else { boxes[i].centre() }, i)
                })
                .collect();
            keyed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            order[c] = keyed.into_iter().map(|k| k.1).collect();
        }
        stack(&order, &mut boxes);
    }
    // the faint kinds under the column's boxes, as many as there is room for
    let mut faint: Vec<Option<Place>> = vec![None; m.faint.len()];
    for c in 0..cols {
        let mut y = in_col[c].iter().map(|&i| boxes[i].y + boxes[i].h).fold(TOP, f32::max) + GAP;
        for (k, _) in m.faint.iter().enumerate().filter(|(_, f)| f.col == c) {
            if y + LABEL + FAINT_H > h - BOTTOM {
                break;
            }
            y += LABEL;
            faint[k] = Some(Place { x: col_x(c) + 0.5 * (BOX_MAX - BOX_PLAIN * 2.0), y, w: BOX_PLAIN * 2.0, h: FAINT_H });
            y += FAINT_H + GAP;
        }
    }
    // the ribbons: out of each box's right side in the order of where they
    // go, into each box's left side in the order of where they came from
    let mut out_at = vec![0.0f32; n];
    let mut in_at = vec![0.0f32; n];
    let mut idx: Vec<usize> = (0..m.ribbons.len()).filter(|&k| m.ribbons[k].from != m.ribbons[k].to).collect();
    idx.sort_by(|&a, &b| {
        let (ra, rb) = (&m.ribbons[a], &m.ribbons[b]);
        ra.from.cmp(&rb.from).then(boxes[ra.to].centre().total_cmp(&boxes[rb.to].centre())).then(ra.good.cmp(&rb.good))
    });
    let mut y0s = vec![0.0f32; m.ribbons.len()];
    for &k in &idx {
        let r = &m.ribbons[k];
        y0s[k] = boxes[r.from].y + out_at[r.from];
        out_at[r.from] += r.value as f32 * scale;
    }
    idx.sort_by(|&a, &b| {
        let (ra, rb) = (&m.ribbons[a], &m.ribbons[b]);
        ra.to.cmp(&rb.to).then(boxes[ra.from].centre().total_cmp(&boxes[rb.from].centre())).then(ra.good.cmp(&rb.good))
    });
    let mut bands = Vec::new();
    for &k in &idx {
        let r = &m.ribbons[k];
        let y1 = boxes[r.to].y + in_at[r.to];
        in_at[r.to] += r.value as f32 * scale;
        let (a, b) = (&boxes[r.from], &boxes[r.to]);
        bands.push(Band { ribbon: k, x0: a.x + a.w, y0: y0s[k], x1: b.x, y1, t: r.value as f32 * scale, colour: good_colour(m.good_colour.get(r.good).map_or("#888888", |s| s.as_str())), back: r.back });
    }
    // the widest under the narrow, so that a thin ribbon is not hidden
    bands.sort_by(|a, b| b.t.total_cmp(&a.t).then(a.ribbon.cmp(&b.ribbon)));
    Plan { w, h, making: m.clone(), boxes, faint, bands, scale }
}

/// What is at a point of the chart: a box (with a little slack), a faint
/// kind, or the thinnest ribbon there.
pub fn hit(p: &Plan, at: Vec2) -> Option<Hit> {
    if let Some(i) = (0..p.boxes.len()).find(|&i| p.boxes[i].holds(at, 3.0)) {
        return Some(Hit::Box(i));
    }
    if let Some(k) = (0..p.faint.len()).find(|&k| p.faint[k].as_ref().is_some_and(|f| f.holds(at, 3.0))) {
        return Some(Hit::Faint(k));
    }
    p.bands
        .iter()
        .filter(|b| b.t >= 0.3 && at.x >= b.x0.min(b.x1) && at.x <= b.x0.max(b.x1))
        .filter(|b| {
            let top = b.top(at.x);
            at.y >= top - 1.5 && at.y <= top + b.t.max(1.0) + 1.5
        })
        .min_by(|a, b| a.t.total_cmp(&b.t))
        .map(|b| Hit::Ribbon(b.ribbon))
}

/// A script's pointer: a box whose name starts with `words`, or a ribbon
/// `GOOD>BOX` of that good into a box whose name starts so (the largest such).
pub fn find(p: &Plan, words: &str) -> Option<Hit> {
    let w = words.trim().to_lowercase();
    let starts = |s: &str, with: &str| s.to_lowercase().starts_with(with);
    if let Some((good, to)) = w.split_once('>') {
        let (good, to) = (good.trim(), to.trim());
        return p
            .bands
            .iter()
            .filter(|b| {
                let r = &p.making.ribbons[b.ribbon];
                let gname = p.making.good_label.get(r.good).map_or("", |s| s.as_str());
                starts(gname, good) && starts(&p.making.nodes[r.to].label, to)
            })
            .max_by(|a, b| a.t.total_cmp(&b.t))
            .map(|b| Hit::Ribbon(b.ribbon));
    }
    if let Some(i) = p.making.nodes.iter().position(|x| starts(&x.label, &w)) {
        return Some(Hit::Box(i));
    }
    p.making.faint.iter().position(|f| starts(&f.label, &w)).map(Hit::Faint)
}

/// Where the pointer stands over a thing, for a script's pointer.
fn spot(p: &Plan, h: Hit) -> Vec2 {
    match h {
        Hit::Box(i) => Vec2::new(p.boxes[i].x + p.boxes[i].w, p.boxes[i].centre()),
        Hit::Faint(k) => p.faint[k].as_ref().map_or(Vec2::ZERO, |f| Vec2::new(f.x + f.w, f.centre())),
        Hit::Ribbon(k) => p.bands.iter().find(|b| b.ribbon == k).map_or(Vec2::ZERO, |b| {
            let x = b.x0 + 0.5 * (b.x1 - b.x0);
            Vec2::new(x, b.top(x) + 0.5 * b.t)
        }),
    }
}

// ---------------------------------------------------------------- painting

const GROUND: [f32; 3] = [252.0, 247.0, 235.0];

struct Canvas {
    w: usize,
    h: usize,
    px: Vec<[f32; 3]>,
}

impl Canvas {
    fn over(&mut self, x: usize, y: usize, col: [f32; 3], a: f32) {
        if x >= self.w || y >= self.h {
            return;
        }
        let d = &mut self.px[y * self.w + x];
        for i in 0..3 {
            d[i] += (col[i] - d[i]) * a;
        }
    }

    /// A rectangle, its edges soft by the share of each pixel it covers.
    fn rect(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, col: [f32; 3], a: f32) {
        let (xa, xb) = (x0.floor().max(0.0) as usize, (x1.ceil() as usize).min(self.w));
        let (ya, yb) = (y0.floor().max(0.0) as usize, (y1.ceil() as usize).min(self.h));
        for y in ya..yb {
            let cy = ((y as f32 + 1.0).min(y1) - (y as f32).max(y0)).clamp(0.0, 1.0);
            for x in xa..xb {
                let cx = ((x as f32 + 1.0).min(x1) - (x as f32).max(x0)).clamp(0.0, 1.0);
                self.over(x, y, col, a * cx * cy);
            }
        }
    }

    /// A ribbon, column by column of pixels.
    fn band(&mut self, b: &Band, a: f32) {
        let t = b.t.max(0.8);
        let (xa, xb) = (b.x0.min(b.x1).floor().max(0.0) as usize, (b.x0.max(b.x1).ceil() as usize).min(self.w));
        for x in xa..xb {
            let top = b.top(x as f32 + 0.5);
            let bot = top + t;
            for y in top.floor().max(0.0) as usize..(bot.ceil() as usize).min(self.h) {
                let c = ((y as f32 + 1.0).min(bot) - (y as f32).max(top)).clamp(0.0, 1.0);
                self.over(x, y, b.colour, a * c);
            }
        }
    }

    /// A dashed rule round a rectangle.
    fn dashed(&mut self, p: &Place, col: [f32; 3], a: f32) {
        let (x0, y0, x1, y1) = (p.x.round() as usize, p.y.round() as usize, (p.x + p.w).round() as usize, (p.y + p.h).round() as usize);
        for x in x0..=x1 {
            if (x / 3) % 2 == 0 {
                self.over(x, y0, col, a);
                self.over(x, y1, col, a);
            }
        }
        for y in y0..=y1 {
            if (y / 3) % 2 == 0 {
                self.over(x0, y, col, a);
                self.over(x1, y, col, a);
            }
        }
    }
}

/// The chart painted, sRGB with alpha, four bytes a pixel; what is under the
/// pointer stands out and the rest steps back.
pub fn paint(p: &Plan, lit: Option<Hit>) -> (usize, usize, Vec<u8>) {
    let (w, h) = (p.w.round().max(1.0) as usize, p.h.round().max(1.0) as usize);
    let mut c = Canvas { w, h, px: vec![GROUND; w * h] };
    let m = &p.making;
    let lit_ribbon = |k: usize| match lit {
        None => None,
        Some(Hit::Ribbon(r)) => Some(r == k),
        Some(Hit::Box(i)) => Some(m.ribbons[k].from == i || m.ribbons[k].to == i),
        Some(Hit::Faint(_)) => Some(false),
    };
    for b in &p.bands {
        let a = match lit_ribbon(b.ribbon) {
            None => if b.back { 0.30 } else { 0.62 },
            Some(true) => 0.88,
            Some(false) => 0.16,
        };
        c.band(b, a);
    }
    for (i, b) in p.boxes.iter().enumerate() {
        let a = if matches!(lit, Some(Hit::Box(j)) if j == i) { 1.0 } else { 0.92 };
        c.rect(b.x, b.y, b.x + b.w, b.y + b.h, box_colour(m.nodes[i].kind), a);
    }
    for (k, f) in p.faint.iter().enumerate() {
        if let Some(f) = f {
            let a = if lit == Some(Hit::Faint(k)) { 0.9 } else { 0.5 };
            c.rect(f.x, f.y, f.x + f.w, f.y + f.h, [236.0, 229.0, 214.0], a);
            c.dashed(f, [120.0, 107.0, 92.0], a);
        }
    }
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
        BackgroundColor(PAPER),
        BoxShadow::new(Color::srgba(0.25, 0.18, 0.10, 0.18), Val::Px(0.0), Val::Px(3.0), Val::Px(0.0), Val::Px(10.0)),
        Interaction::default(),
        FocusPolicy::Block,
        GlobalZIndex(9),
        Visibility::Hidden,
        FlowPanel,
    ));
}

/// The town picked, while the clock is at the present.
fn town_picked(picked: &Picked) -> Option<usize> {
    match picked.what.map(thing_of) {
        Some(Thing::Town(t)) => Some(t),
        _ => None,
    }
}

/// The chip in a town's card, drawn again only when the town picked changes.
fn card_chip(mut commands: Commands, font: Res<UiFont>, picked: Res<Picked>, tl: Res<Timeline>, data: Res<Data>, live: Res<Live>, q: Query<(Entity, Option<&Children>), With<MakingBlock>>, mut drawn: Local<Option<Option<usize>>>) {
    let town = town_picked(&picked).filter(|_| at_present(&tl, &data, &live));
    if *drawn == Some(town) {
        return;
    }
    let Ok((e, children)) = q.single() else { return };
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    if town.is_some() {
        let f = font.0.clone();
        commands.entity(e).with_children(|p| {
            p.spawn(Node { margin: UiRect::top(Val::Px(6.0)), column_gap: Val::Px(8.0), align_items: AlignItems::Center, ..default() }).with_children(|row| {
                crate::family::chip_button(row, &f, "Making", false, INK, OpenMaking);
                row.spawn(text(&f, "what it makes, from what, for whom", 11.5, MUTED));
            });
        });
    }
    *drawn = Some(town);
}

/// Open from the card's chip; follow the town picked; close with Close,
/// Escape, or when no town is picked.
fn open_close(open: Query<&Interaction, (Changed<Interaction>, With<OpenMaking>)>, close: Query<&Interaction, (Changed<Interaction>, With<CloseMaking>)>, keys: Res<ButtonInput<KeyCode>>, picked: Res<Picked>, mut flow: ResMut<Flow>) {
    let town = town_picked(&picked);
    if open.iter().any(|i| *i == Interaction::Pressed) {
        flow.town = town;
    } else if flow.town.is_some() && town.is_some() && flow.town != town {
        flow.town = town;
    }
    if close.iter().any(|i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Escape) || town.is_none() {
        flow.town = None;
    }
}

/// Ask for the town's making when the panel opens, and again when a year
/// comes or an order is answered; not while the year turns, and only with
/// the clock at the present.
fn ask(flow: Res<Flow>, tl: Res<Timeline>, data: Res<Data>, live: Res<Live>, game: Res<Game>, mut answers: ResMut<Answers>, mut seen: Local<Option<(usize, usize, u64)>>) {
    let Some(t) = flow.town else {
        *seen = None;
        return;
    };
    let key = (t, data.r.filled, game.answered);
    if *seen == Some(key) || live.turning() || !at_present(&tl, &data, &live) {
        return;
    }
    *seen = Some(key);
    answers.ask(&live, Thing::Making(t));
}

/// What the panel shows, to tell whether it must be drawn again.
#[derive(Clone, PartialEq, Debug)]
enum Shown {
    Shut,
    Asking(usize),
    Refused(String),
    Chart(Box<Making>, (u32, u32), bool),
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    font: Res<UiFont>,
    answers: Res<Answers>,
    tl: Res<Timeline>,
    data: Res<Data>,
    live: Res<Live>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut flow: ResMut<Flow>,
    mut images: ResMut<Assets<Image>>,
    mut q: Query<(Entity, &mut Visibility, Option<&Children>), With<FlowPanel>>,
    mut drawn: Local<Option<Shown>>,
) {
    let win = windows.single().map(|w| Vec2::new(w.width(), w.height())).unwrap_or(Vec2::new(1600.0, 1000.0));
    let now = match flow.town {
        None => Shown::Shut,
        Some(t) => match answers.get(Thing::Making(t)) {
            None => Shown::Asking(t),
            Some(Err(why)) => Shown::Refused(why.clone()),
            Some(Ok(told)) => match &told.making {
                Some(m) if m.town == t => Shown::Chart(Box::new(m.clone()), (win.x as u32, win.y as u32), at_present(&tl, &data, &live)),
                _ => Shown::Asking(t),
            },
        },
    };
    if drawn.as_ref() == Some(&now) {
        return;
    }
    let Ok((e, mut vis, children)) = q.single_mut() else { return };
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    flow.hit = None;
    flow.plan = None;
    flow.image = None;
    *vis = if now == Shown::Shut { Visibility::Hidden } else { Visibility::Inherited };
    let f = font.0.clone();
    match &now {
        Shown::Shut => {}
        Shown::Asking(_) => {
            commands.entity(e).with_children(|p| {
                head(p, &f, "The making", None);
                p.spawn(text(&f, "Asking the simulation...", 13.0, MUTED));
            });
        }
        Shown::Refused(why) => {
            commands.entity(e).with_children(|p| {
                head(p, &f, "The making", None);
                p.spawn(text(&f, why, 13.0, MUTED));
            });
        }
        Shown::Chart(m, _, present) => {
            let t0 = Instant::now();
            let size = chart_size(win);
            let pl = plan(m, size.x, size.y);
            let t1 = Instant::now();
            let (w, h, px) = paint(&pl, None);
            let t2 = Instant::now();
            let handle = images.add(image(w, h, px));
            println!(
                "making: {} in {}: {} boxes, {} ribbons ({} back), {} faint kinds, {} columns; laid out in {:.1} ms, painted in {:.1} ms",
                m.name,
                m.year,
                m.nodes.len(),
                m.ribbons.iter().filter(|r| r.from != r.to).count(),
                m.ribbons.iter().filter(|r| r.back).count(),
                m.faint.len(),
                m.cols,
                1e3 * (t1 - t0).as_secs_f64(),
                1e3 * (t2 - t1).as_secs_f64()
            );
            commands.entity(e).with_children(|p| {
                head(p, &f, &format!("How {} made its living in {}", m.name, m.year), Some(*present));
                p.spawn(text(&f, &m.said, 11.5, MUTED));
                p.spawn((
                    Node { width: Val::Px(size.x), height: Val::Px(size.y), margin: UiRect::top(Val::Px(4.0)), flex_shrink: 0.0, ..default() },
                    ImageNode { image: handle.clone(), ..default() },
                    Interaction::default(),
                    RelativeCursorPosition::default(),
                    FlowChart,
                ))
                .with_children(|c| labels(c, &f, &pl));
                key(p, &f, m);
            });
            flow.plan = Some(pl);
            flow.image = Some(handle);
        }
    }
    *drawn = Some(now);
}

/// The panel's title, and its Close.
fn head(p: &mut ChildSpawnerCommands, f: &Handle<Font>, title: &str, present: Option<bool>) {
    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
        row.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|col| {
            col.spawn(text(f, title, 20.0, INK));
            if present == Some(false) {
                col.spawn(text(f, "The clock is looking back: the chart is of the last year closed, and follows the present.", 11.5, MUTED));
            }
        });
        crate::family::chip_button(row, f, "Close", false, INK, CloseMaking);
    });
}

/// A name shortened to about `room` pixels of the interface's small type.
fn fit(s: &str, room: f32) -> String {
    let most = ((room / 6.1).floor() as usize).max(4);
    if s.chars().count() <= most {
        return s.to_string();
    }
    let cut: String = s.chars().take(most - 1).collect();
    format!("{}…", cut.trim_end())
}

/// Each box's name above it, with its hands or its worth; a faint kind's in
/// grey. The names of the last column end at its right edge.
fn labels(c: &mut ChildSpawnerCommands, f: &Handle<Font>, p: &Plan) {
    let cols = p.making.cols.max(2);
    let step = (p.w - 2.0 * SIDE - BOX_MAX) / (cols - 1) as f32;
    let room = (step - 8.0).max(60.0);
    let halo = PAPER.with_alpha(0.82);
    let mut put = |b: &Place, words: String, ink: Color, right: bool| {
        let mut node = Node { position_type: PositionType::Absolute, top: Val::Px(b.y - LABEL + 1.0), padding: UiRect::horizontal(Val::Px(2.0)), ..default() };
        if right {
            node.right = Val::Px(p.w - (b.x + b.w) - 2.0);
        } else {
            node.left = Val::Px(b.x - 2.0);
        }
        c.spawn((node, BackgroundColor(halo), FocusPolicy::Pass)).with_children(|t| {
            t.spawn((text(f, &words, 11.0, ink), TextLayout::default().with_no_wrap()));
        });
    };
    for (i, b) in p.boxes.iter().enumerate() {
        let x = &p.making.nodes[i];
        let figure = if x.kind.makes() {
            format!("{} hands", isostate_core::util::fmt_thousands_f0(x.hands))
        } else if x.kind == Kind::Households {
            format!("{} people", isostate_core::util::fmt_thousands_f0(x.hands))
        } else {
            isostate_core::util::fmt_thousands_f0(x.through())
        };
        let words = format!("{}  {figure}", x.label);
        put(b, fit(&words, room), INK, x.col == cols - 1);
    }
    for (k, fp) in p.faint.iter().enumerate() {
        if let Some(b) = fp {
            put(b, fit(&format!("{} (none here)", p.making.faint[k].label), room), MUTED.with_alpha(0.75), false);
        }
    }
    // the words under the pointer, hidden until it rests on something
    c.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            max_width: Val::Px(400.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(7.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(1.0, 0.99, 0.96)),
        BorderColor::all(RULE),
        FocusPolicy::Pass,
        Visibility::Hidden,
        GlobalZIndex(11),
        FlowTip,
    ));
}

/// The goods drawn, the largest first, each with its colour.
fn key(p: &mut ChildSpawnerCommands, f: &Handle<Font>, m: &Making) {
    let mut by: Vec<(f64, usize)> = Vec::new();
    for r in m.ribbons.iter().filter(|r| r.from != r.to) {
        match by.iter_mut().find(|x| x.1 == r.good) {
            Some(x) => x.0 += r.value,
            None => by.push((r.value, r.good)),
        }
    }
    by.sort_by(|a, b| b.0.total_cmp(&a.0));
    p.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(10.0), row_gap: Val::Px(2.0), margin: UiRect::top(Val::Px(4.0)), ..default() }).with_children(|row| {
        for &(_, g) in by.iter().take(16) {
            let c = good_colour(m.good_colour.get(g).map_or("#888888", |s| s.as_str()));
            row.spawn(Node { column_gap: Val::Px(4.0), align_items: AlignItems::Center, ..default() }).with_children(|k| {
                k.spawn((Node { width: Val::Px(10.0), height: Val::Px(10.0), border_radius: BorderRadius::all(Val::Px(5.0)), ..default() }, BackgroundColor(Color::srgb_u8(c[0] as u8, c[1] as u8, c[2] as u8))));
                k.spawn(text(f, m.good_label.get(g).map_or("", |s| s.as_str()), 11.5, INK));
            });
        }
        if by.len() > 16 {
            row.spawn(text(f, &format!("and {} more goods", by.len() - 16), 11.5, MUTED));
        }
    });
}

/// What is under the pointer (or a script's pointer): its words beside it,
/// and the chart painted again with it standing out, only when it changes.
#[allow(clippy::too_many_arguments)]
fn hover(
    mut commands: Commands,
    font: Res<UiFont>,
    mut flow: ResMut<Flow>,
    chart: Query<(&RelativeCursorPosition, &ComputedNode), With<FlowChart>>,
    mut tip: Query<(Entity, &mut Node, &mut Visibility, Option<&Children>), With<FlowTip>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(pl) = flow.plan.as_ref() else { return };
    let (at, hit_now) = match flow.point.as_deref() {
        Some(words) => match find(pl, words) {
            Some(h) => (Some(spot(pl, h)), Some(h)),
            None => (None, None),
        },
        None => {
            let at = chart.single().ok().and_then(|(rel, node)| {
                let n = rel.normalized?;
                let size = node.size() * node.inverse_scale_factor();
                Some((n + Vec2::splat(0.5)) * size)
            });
            (at, at.and_then(|a| hit(pl, a)))
        }
    };
    if hit_now == flow.hit {
        return;
    }
    let t0 = Instant::now();
    let (_, _, px) = paint(pl, hit_now);
    if let Some(mut img) = flow.image.as_ref().and_then(|h| images.get_mut(h)) {
        img.data = Some(px);
    }
    let painted = t0.elapsed().as_secs_f64() * 1e3;
    let Ok((e, mut node, mut vis, children)) = tip.single_mut() else { return };
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    let lines: Vec<String> = match hit_now {
        Some(Hit::Box(i)) => pl.making.nodes[i].said.clone(),
        Some(Hit::Faint(k)) => std::iter::once(format!("{} (none here)", pl.making.faint[k].label)).chain(pl.making.faint[k].said.iter().cloned()).collect(),
        Some(Hit::Ribbon(k)) => pl.making.ribbons[k].said.clone(),
        None => Vec::new(),
    };
    if let (Some(at), false) = (at, lines.is_empty()) {
        // beside the pointer, on the side with room
        let right = at.x < 0.55 * pl.w;
        node.left = if right { Val::Px(at.x + 14.0) } else { Val::Auto };
        node.right = if right { Val::Auto } else { Val::Px(pl.w - at.x + 14.0) };
        node.top = Val::Px((at.y - 20.0).clamp(0.0, (pl.h - 160.0).max(0.0)));
        *vis = Visibility::Inherited;
        let f = font.0.clone();
        commands.entity(e).with_children(|t| {
            for (n, l) in lines.iter().enumerate() {
                t.spawn(text(&f, l, if n == 0 { 13.0 } else { 11.5 }, if n == 0 { INK } else { MUTED }));
            }
        });
        println!("making: under the pointer ({painted:.1} ms to paint): {}", lines.join(" | "));
    } else {
        *vis = Visibility::Hidden;
    }
    flow.hit = hit_now;
}

#[cfg(test)]
mod tests {
    use super::*;
    use isostate_core::making::{GoodFlow, Node as MNode, Ribbon};

    fn node(kind: Kind, label: &str, col: usize, hands: f64, vin: f64, vout: f64) -> MNode {
        MNode { kind, label: label.into(), col, hands, works: 0.0, value_in: vin, value_out: vout, good: None, said: vec![label.into()] }
    }

    fn ribbon(from: usize, to: usize, good: usize, value: f64) -> Ribbon {
        Ribbon { from, to, good, qty: value, value, back: false, said: vec!["a ribbon".into()] }
    }

    /// A small town: farms and a pit, a mill and a smithy, households and the outside.
    fn making() -> Making {
        let nodes = vec![
            node(Kind::FromTowns, "From other towns", 0, 0.0, 0.0, 20.0),
            node(Kind::Land, "Farmland", 0, 500.0, 0.0, 100.0),
            node(Kind::Land, "Colliery", 0, 100.0, 0.0, 40.0),
            node(Kind::Works, "Mill", 1, 50.0, 30.0, 60.0),
            node(Kind::Works, "Smithy", 2, 20.0, 30.0, 40.0),
            node(Kind::Households, "Households", 3, 3000.0, 170.0, 0.0),
            node(Kind::ToTowns, "To other towns", 3, 0.0, 30.0, 0.0),
        ];
        let ribbons = vec![
            ribbon(1, 3, 0, 30.0),
            ribbon(1, 5, 0, 70.0),
            ribbon(2, 4, 1, 20.0),
            ribbon(2, 5, 1, 20.0),
            ribbon(0, 4, 2, 10.0),
            ribbon(0, 5, 3, 10.0),
            ribbon(3, 5, 4, 50.0),
            ribbon(3, 6, 4, 10.0),
            ribbon(4, 6, 5, 20.0),
            ribbon(4, 5, 5, 20.0),
        ];
        Making {
            town: 0,
            name: "Testham".into(),
            year: 1800,
            cols: 4,
            nodes,
            ribbons,
            faint: vec![isostate_core::making::Faint { good: 6, label: "Glassworks".into(), col: 2, said: vec!["glass".into()] }],
            goods: vec![GoodFlow::default()],
            good_label: ["Grain", "Coal", "Iron", "Wool", "Cloth", "Tools", "Glass"].iter().map(|s| s.to_string()).collect(),
            good_colour: ["#d6b85a", "#2e2b28", "#5b6670", "#e8e1cf", "#a8476b", "#6e5b45", "#9fd0c8"].iter().map(|s| s.to_string()).collect(),
            hands: 670.0,
            said: String::new(),
        }
    }

    #[test]
    fn a_plan_stacks_boxes_apart_and_each_boxs_ribbons_side_by_side_within_it() {
        let m = making();
        let p = plan(&m, 900.0, 500.0);
        // every box inside the chart, none over another in its column, room for its name above
        for (i, b) in p.boxes.iter().enumerate() {
            assert!(b.x >= 0.0 && b.x + b.w <= p.w + 0.01 && b.y >= LABEL && b.y + b.h <= p.h + 0.01, "{}: {b:?}", m.nodes[i].label);
            for (j, o) in p.boxes.iter().enumerate() {
                if i != j && m.nodes[i].col == m.nodes[j].col {
                    assert!(b.y + b.h + LABEL <= o.y + 0.01 || o.y + o.h + LABEL <= b.y + 0.01, "{} and {} overlap", m.nodes[i].label, m.nodes[j].label);
                }
            }
            // a box is as tall as the value through it, on the plan's one scale
            assert!((b.h - (m.nodes[i].through() as f32 * p.scale).max(2.0)).abs() < 1e-3);
        }
        // a box's ribbons out lie within its right side, and its ribbons in within its left
        for (i, b) in p.boxes.iter().enumerate() {
            let outs: Vec<&Band> = p.bands.iter().filter(|x| m.ribbons[x.ribbon].from == i).collect();
            let ins: Vec<&Band> = p.bands.iter().filter(|x| m.ribbons[x.ribbon].to == i).collect();
            for x in &outs {
                assert!(x.y0 >= b.y - 1e-3 && x.y0 + x.t <= b.y + b.h + 1e-3 && (x.x0 - (b.x + b.w)).abs() < 1e-3);
            }
            for x in &ins {
                assert!(x.y1 >= b.y - 1e-3 && x.y1 + x.t <= b.y + b.h + 1e-3 && (x.x1 - b.x).abs() < 1e-3);
            }
            let sum_out: f32 = outs.iter().map(|x| x.t).sum();
            assert!((sum_out - m.nodes[i].value_out as f32 * p.scale).abs() < 1e-2);
        }
        // the wider the hands, the wider the box: the farms are the widest
        let widest = (0..p.boxes.len()).max_by(|&a, &b| p.boxes[a].w.total_cmp(&p.boxes[b].w)).unwrap();
        assert_eq!(m.nodes[widest].label, "Farmland");
        // and the faint kind found room in its column
        assert!(p.faint[0].is_some());
    }

    #[test]
    fn the_pointer_finds_a_box_a_ribbon_and_a_faint_kind_and_a_script_names_them() {
        let m = making();
        let p = plan(&m, 900.0, 500.0);
        for i in 0..m.nodes.len() {
            let b = &p.boxes[i];
            assert_eq!(hit(&p, Vec2::new(b.x + 0.5 * b.w, b.centre())), Some(Hit::Box(i)));
            assert_eq!(find(&p, &m.nodes[i].label[..4]), Some(Hit::Box(i)), "{}", m.nodes[i].label);
        }
        // the middle of each ribbon wide enough to point at is that ribbon, or one drawn over it that
        // is thinner, or a box it passes behind
        for b in p.bands.iter().filter(|b| b.t >= 2.0) {
            let x = b.x0 + 0.5 * (b.x1 - b.x0);
            match hit(&p, Vec2::new(x, b.top(x) + 0.5 * b.t)) {
                Some(Hit::Ribbon(k)) => assert!(k == b.ribbon || p.bands.iter().find(|o| o.ribbon == k).unwrap().t <= b.t),
                Some(Hit::Box(i)) => assert!(p.boxes[i].x > b.x0 && p.boxes[i].x < b.x1, "ribbon {} found box {i} not on its way", b.ribbon),
                other => panic!("ribbon {} found {other:?}", b.ribbon),
            }
        }
        assert_eq!(find(&p, "grain>house"), Some(Hit::Ribbon(1)));
        assert_eq!(find(&p, "glass"), Some(Hit::Faint(0)));
        assert_eq!(find(&p, "nothing of the kind"), None);
        assert_eq!(hit(&p, Vec2::new(p.w - 1.0, 1.0)), None);
    }

    #[test]
    fn a_painting_lights_what_is_pointed_at_and_a_pale_good_is_darkened() {
        let m = making();
        let p = plan(&m, 600.0, 400.0);
        let (w, h, plain) = paint(&p, None);
        assert_eq!((w, h, plain.len()), (600, 400, 600 * 400 * 4));
        let (_, _, lit) = paint(&p, Some(Hit::Box(3)));
        assert_ne!(plain, lit);
        // wool's pale colour is made darker, grain's is kept
        assert!(good_colour("#e8e1cf")[0] < 232.0);
        assert_eq!(good_colour("#d6b85a"), [214.0, 184.0, 90.0]);
    }
}
