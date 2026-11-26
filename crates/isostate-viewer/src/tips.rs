//! Tooltips, and tooltips in tooltips (docs/designs/words.md, "Tooltips").
//!
//! Any piece of the interface can carry one: give its node a `Tip` with the
//! tooltip's body (`TipBody`), or make it a text of pieces (`spawn_pieces`)
//! whose links are spans. A tooltip shows when the pointer has rested on its
//! source for `OPEN_AFTER`, and goes when the pointer leaves the source. When
//! the pointer has rested `LOCK_AFTER` in all, it locks: a ring in its top
//! right corner fills on the way and shows a dot once locked, and the border
//! darkens. A middle click on a source, or a click on a term, opens its
//! tooltip locked at once. A locked tooltip stays while the pointer is on it,
//! on its source or on any tooltip opened from it, and closes `CLOSE_AFTER`
//! after the pointer has left all of them. Esc closes the whole stack before
//! anything else sees the key, and a click outside the stack closes it too.
//!
//! A link inside a tooltip is a source like any other, with the same times,
//! and there is no limit of depth: the open tooltips are a stack, each opened
//! from a source in the one before it (or in the card under them all). A
//! tooltip never covers the source it was opened from and is kept inside the
//! window. While a stack is open the map takes no click and no hover
//! (`ui::ui_hover`).
//!
//! It never waits. A tooltip is built in the frame it opens, from the
//! glossary (words.rs) and from what the view already holds: the record, and
//! the figures its source was given when its card was drawn. Nothing here
//! asks the simulation. What a tooltip shows is what it was given when it
//! opened: it does not change while the clock runs.
//!
//! Which span of a text the pointer is on is read from the text's own layout
//! (`TextLayoutInfo::run_geometry`, whose runs carry the index of their span),
//! so a paragraph is one text with its links as spans, laid out and wrapped by
//! Bevy as any text is.

// see the note at the top of ui.rs
#![allow(deprecated)]

use std::sync::Arc;
use std::time::Instant;

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::text::{ComputedTextBlock, TextLayoutInfo, Underline, UnderlineColor};
use bevy::ui::{CalculatedClip, ComputedNode, FocusPolicy, UiGlobalTransform};

use crate::ui::{INK, MUTED, RULE, UiState};
use crate::words::{self, Glossary, Link, Piece};
use crate::{Data, MainCam, Orbit, Pick, Picked, Timeline, UiFont};

// ------------------------------------------------------------------ the named values

/// How long the pointer rests on a source before its tooltip shows, in seconds.
pub const OPEN_AFTER: f32 = 0.3;
/// How long in all before the tooltip locks; the ring fills between the two.
pub const LOCK_AFTER: f32 = 0.9;
/// How long a locked tooltip stays once the pointer has left it, its source and those opened from it.
pub const CLOSE_AFTER: f32 = 0.25;
/// The width of a tooltip of a term, and of the small one of a thing, in logical pixels.
pub const WIDTH: f32 = 300.0;
pub const SMALL_WIDTH: f32 = 220.0;
/// The room between a tooltip and its source, and between a tooltip and the window's edge.
pub const GAP: f32 = 6.0;
pub const EDGE: f32 = 8.0;
/// The three kinds of link (words.md, "Links"): a term, a thing on the island, Wikipedia.
pub const TERM_BLUE: Color = Color::srgb(0x2c as f32 / 255.0, 0x5f as f32 / 255.0, 0x8a as f32 / 255.0);
pub const THING_BROWN: Color = Color::srgb(0x9a as f32 / 255.0, 0x5a as f32 / 255.0, 0x1e as f32 / 255.0);
pub const WIKI_PLUM: Color = Color::srgb(0x7a as f32 / 255.0, 0x4f as f32 / 255.0, 0x8f as f32 / 255.0);
/// A tooltip's paper (the cards' paper, solid), its border open and locked, and its shadow.
pub const TIP_PAPER: Color = Color::srgb(0.99, 0.975, 0.94);
pub const BORDER: Color = Color::srgb(0.84, 0.79, 0.70);
pub const BORDER_LOCKED: Color = INK;
/// The border's width, open and locked alike, so that locking moves nothing.
pub const BORDER_W: f32 = 2.0;
/// A term's dotted line: each dot's length and the step from one to the next, in logical pixels.
pub const DOT: f32 = 1.6;
pub const DOT_STEP: f32 = 3.2;
/// The ring's dots before and after they fill.
pub const RING_EMPTY: Color = Color::srgb(0.86, 0.81, 0.72);
pub const RING_FULL: Color = Color::srgb(0.36, 0.31, 0.26);
/// The ground of a row of a card while the pointer rests on it or its tooltip is open.
pub const SOURCE_LIT: Color = Color::srgb(0.955, 0.925, 0.86);
/// The type: the term in small capitals, its value, the sentences and rows.
pub const TERM_SIZE: f32 = 11.0;
pub const VALUE_SIZE: f32 = 18.0;
pub const TEXT_SIZE: f32 = 13.5;
pub const ROW_SIZE: f32 = 12.5;

/// The dots round the ring.
const RING_DOTS: usize = 12;
/// The tooltips stand above every card (the cards are at 10 to 30).
const Z: i32 = 60;

// ------------------------------------------------------------------ what a tooltip holds

/// A tooltip's body, in the order words.md gives the parts: the term (shown
/// in small capitals), its value now, its sentences, its rows and a line of
/// links.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TipBody {
    pub term: String,
    pub value: Option<String>,
    pub says: Vec<Piece>,
    pub rows: Vec<(Vec<Piece>, Vec<Piece>)>,
    pub links: Vec<(Link, String)>,
    /// a thing's tooltip: narrower
    pub small: bool,
}

impl TipBody {
    /// The tooltip of a glossary term, with its value now if it has one, and
    /// rows if its number is made of parts. A key the glossary lacks gives a
    /// tooltip of the key alone.
    pub fn term(g: &Glossary, key: &str, value: Option<String>, rows: Vec<(Vec<Piece>, Vec<Piece>)>) -> TipBody {
        let Some(t) = g.get(key) else {
            return TipBody { term: key.to_string(), value, rows, ..default() };
        };
        let mut links: Vec<(Link, String)> = t.see.iter().filter_map(|s| g.get(s).map(|x| (Link::Term(s.clone()), x.name.clone()))).collect();
        links.extend(t.wiki.iter().map(|w| (Link::Wiki(w.clone()), w.clone())));
        TipBody { term: t.name.clone(), value, says: g.pieces(&t.says, Some(key)), rows, links, small: false }
    }

    /// Every word it shows, for a script's print and for the tests.
    pub fn words(&self) -> String {
        let mut s = self.term.to_uppercase();
        if let Some(v) = &self.value {
            s.push_str(&format!(" | {v}"));
        }
        if !self.says.is_empty() {
            s.push_str(&format!(" | {}", self.says.iter().map(|p| p.text()).collect::<String>()));
        }
        for (a, b) in &self.rows {
            s.push_str(&format!(" | {}: {}", a.iter().map(|p| p.text()).collect::<String>(), b.iter().map(|p| p.text()).collect::<String>()));
        }
        if !self.links.is_empty() {
            s.push_str(&format!(" | see {}", self.links.iter().map(|l| l.1.as_str()).collect::<Vec<_>>().join(", ")));
        }
        s
    }
}

/// A row of a tooltip: a label and a value, each plain words.
pub fn row(label: impl Into<String>, value: impl Into<String>) -> (Vec<Piece>, Vec<Piece>) {
    (words::plain(label), words::plain(value))
}

/// A piece of the interface that carries a tooltip, as a whole node. `label`
/// is what a script names it by (`tip:LABEL`).
#[derive(Component, Clone)]
pub struct Tip {
    pub body: Arc<TipBody>,
    pub label: String,
    /// what tells it from every other source on its card, so that a card
    /// drawn again with new figures keeps the pointer's rest and an open
    /// tooltip on the same row
    pub id: String,
}

impl Tip {
    #[allow(dead_code)]
    pub fn new(label: impl Into<String>, body: TipBody) -> Tip {
        let label = label.into();
        Tip { body: Arc::new(body), id: label.clone(), label }
    }
}

/// A text whose spans may be links: the link of each section of the text,
/// by its index in the text's layout (0 is the text itself, which is empty;
/// its spans follow in order).
#[derive(Component, Clone, Debug)]
pub struct Linked(pub Vec<Option<Link>>);

/// A source that stands in an open tooltip, and which one (0 is the first).
#[derive(Component, Clone, Copy)]
pub struct InTip(pub usize);

/// The node of an open tooltip.
#[derive(Component)]
pub struct TipCard;

#[derive(Component)]
struct RingDot(usize);
#[derive(Component)]
struct LockDot;

/// Gives a card's row its lit ground while the pointer rests on it or its tooltip is open.
#[derive(Component)]
pub struct LitRow;

/// The colour of a link.
pub fn link_colour(l: &Link) -> Color {
    match l {
        Link::Term(_) | Link::Method(_) => TERM_BLUE,
        Link::Town(_) | Link::Good(_) => THING_BROWN,
        Link::Wiki(_) => WIKI_PLUM,
    }
}

/// A text of pieces: plain words in `colour`, links in theirs, a term's and a
/// thing's underlined, a Wikipedia link with its arrow after it. In a tooltip
/// (`level`) its links are sources of the tooltips above it.
pub fn spawn_pieces(p: &mut ChildSpawnerCommands, font: &Handle<Font>, pieces: &[Piece], size: f32, colour: Color, level: Option<usize>, node: Node) -> Entity {
    let mut links = vec![None];
    for pc in pieces {
        links.push(match pc {
            Piece::Link(l, _) => Some(l.clone()),
            Piece::Text(_) => None,
        });
    }
    let tf = TextFont { font: font.clone().into(), font_size: FontSize::Px(size), ..default() };
    // the text stands in a box of its own, so that a term's dotted line can be laid under it once
    // the text is laid out (`dot_lines`)
    let mut boxed = p.spawn(Node { flex_direction: FlexDirection::Column, ..node });
    let mut id = Entity::PLACEHOLDER;
    boxed.with_children(|b| {
    let mut e = b.spawn((Text::new(""), tf.clone(), TextColor(colour), Node::default(), Linked(links)));
    if let Some(l) = level {
        e.insert(InTip(l));
    }
    e.with_children(|t| {
        for pc in pieces {
            match pc {
                Piece::Text(s) => {
                    t.spawn((TextSpan::new(s.clone()), tf.clone(), TextColor(colour)));
                }
                Piece::Link(l @ Link::Wiki(_), s) => {
                    // (its arrow is an icon after the text, drawn by `spawn_card`: the type has no ↗)
                    t.spawn((TextSpan::new(s.clone()), tf.clone(), TextColor(link_colour(l))));
                }
                // a term's line is dotted (`dot_lines`), a thing's solid
                Piece::Link(l @ (Link::Term(_) | Link::Method(_)), s) => {
                    t.spawn((TextSpan::new(s.clone()), tf.clone(), TextColor(link_colour(l))));
                }
                Piece::Link(l, s) => {
                    t.spawn((TextSpan::new(s.clone()), tf.clone(), TextColor(link_colour(l)), Underline, UnderlineColor(link_colour(l).with_alpha(0.75))));
                }
            }
        }
    });
    id = e.id();
    });
    id
}

/// A text whose terms' dotted lines are laid.
#[derive(Component)]
struct Dotted;

/// Lay a term's dotted line under each run of it, once its text is laid out:
/// a row of short nodes in the box the text stands in, from the text's own
/// layout. Each text is done once.
/// How many dots have been laid, for a script's print: they are laid once for each text, never again.
pub static DOTS_LAID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn dot_lines(mut commands: Commands, q: Query<(Entity, &Linked, &TextLayoutInfo, &ComputedNode, &ChildOf), Without<Dotted>>) {
    for (e, linked, layout, node, parent) in &q {
        let terms: Vec<(Rect, f32, Color)> = layout
            .run_geometry
            .iter()
            .filter_map(|run| match linked.0.get(run.section_index as usize) {
                Some(Some(l @ (Link::Term(_) | Link::Method(_)))) => Some((run.bounds, run.underline_y + run.underline_thickness, link_colour(l))),
                _ => None,
            })
            .collect();
        let none = linked.0.iter().all(|l| !matches!(l, Some(Link::Term(_) | Link::Method(_))));
        if !none && (layout.run_geometry.is_empty() || node.size.x <= 0.0) {
            continue; // not laid out yet
        }
        commands.entity(e).insert(Dotted);
        let inv = node.inverse_scale_factor;
        commands.entity(parent.parent()).with_children(|b| {
            for (bounds, y, colour) in terms {
                let mut x = bounds.min.x * inv;
                while x + DOT <= bounds.max.x * inv + 0.5 {
                    DOTS_LAID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    b.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(x), top: Val::Px(y * inv), width: Val::Px(DOT), height: Val::Px(1.0), ..default() }, BackgroundColor(colour.with_alpha(0.85))));
                    x += DOT_STEP;
                }
            }
        });
    }
}

// ------------------------------------------------------------------ the stack

/// A source: a node with a `Tip`, or one span of a `Linked` text. Two are
/// the same source when their ids are: a card drawn again is the same card.
#[derive(Clone, Copy, Debug)]
pub struct Src {
    e: Entity,
    section: u32,
    id: u64,
}

impl PartialEq for Src {
    fn eq(&self, o: &Src) -> bool {
        self.id == o.id
    }
}

const WHOLE: u32 = u32::MAX;

fn id_of(level: i32, kind: &str, words: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (level, kind, words).hash(&mut h);
    h.finish()
}

struct Open {
    source: Src,
    card: Entity,
    locked: bool,
    /// when the pointer left it, its source and those above it, while it is locked
    away: Option<f32>,
    /// the source's rectangle in logical pixels, when it opened
    rect: Rect,
    placed: bool,
    /// its words, for a script's print
    words: String,
}

/// The stack of open tooltips, and the pointer's rest.
#[derive(Resource, Default)]
pub struct Tips {
    stack: Vec<Open>,
    /// the source the pointer rests on, and since when
    rest: Option<(Src, f32)>,
    /// a script's pointer (script.rs, `tip:`): rests on the source it names, in place of the mouse
    pub aim: Option<String>,
    /// a script's middle click, taken in the next frame
    pub middle: bool,
    /// what the test under the pointer cost each frame, in microseconds, by how many were open
    cost: [Vec<f32>; 4],
}

impl Tips {
    /// Whether a tooltip is open.
    pub fn open(&self) -> bool {
        !self.stack.is_empty()
    }

    /// The stack in words, for a script's print: each tooltip's words, whether it is locked.
    pub fn said(&self, now: f32) -> Vec<String> {
        let rest = self.rest.map(|r| now - r.1);
        self.stack
            .iter()
            .enumerate()
            .map(|(i, o)| {
                let state = if o.locked { "locked".to_string() } else { format!("open, the pointer resting {:.2} s", rest.unwrap_or(0.0)) };
                format!("{} ({state}): {}", i + 1, o.words)
            })
            .collect()
    }

    /// What the test under the pointer cost a frame: with none open, one, two, three or more.
    pub fn cost_said(&self) -> String {
        let one = |v: &Vec<f32>| {
            if v.is_empty() {
                return "no frames".to_string();
            }
            let mut s = v.clone();
            s.sort_by(f32::total_cmp);
            format!("{:.0} us middle, {:.0} us longest over {} frames", s[s.len() / 2], s[s.len() - 1], s.len())
        };
        format!("none open {}; one {}; two {}; three or more {}; {} dots laid in all", one(&self.cost[0]), one(&self.cost[1]), one(&self.cost[2]), one(&self.cost[3]), DOTS_LAID.load(std::sync::atomic::Ordering::Relaxed))
    }

    fn close_from(&mut self, commands: &mut Commands, i: usize) {
        for o in self.stack.drain(i..) {
            commands.entity(o.card).despawn();
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Tips>()
        .add_systems(PreUpdate, escape.after(bevy::input::InputSystems).before(crate::note::keys))
        .add_systems(Update, (update, lit_rows, dot_lines).chain().after(crate::ui::ui_hover));
}

/// Esc closes the whole stack, and nothing else sees the key in that frame.
pub fn escape(mut keys: ResMut<ButtonInput<KeyCode>>, mut tips: ResMut<Tips>, mut commands: Commands) {
    if tips.open() && keys.just_pressed(KeyCode::Escape) {
        tips.close_from(&mut commands, 0);
        tips.rest = None;
        keys.clear_just_pressed(KeyCode::Escape);
    }
}

/// A source under the pointer, or that a script names.
struct Cand {
    src: Src,
    /// the tooltip it stands in, or -1 for the cards under them all
    level: i32,
    /// its rectangle on the screen, in physical pixels
    rect: Rect,
    clip: Option<CalculatedClip>,
    link: Option<Link>,
    tip: Option<Arc<TipBody>>,
    label: String,
    /// logical pixels to a physical one
    inv: f32,
}

fn node_rect(node: &ComputedNode, gt: &UiGlobalTransform) -> Rect {
    Rect::from_center_size(gt.translation, node.size)
}

/// Everything that can be hovered, with its rectangle.
#[allow(clippy::type_complexity)]
fn candidates(
    tip_q: &Query<(Entity, &Tip, &ComputedNode, &UiGlobalTransform, &InheritedVisibility, Option<&CalculatedClip>, Option<&InTip>)>,
    linked_q: &Query<(Entity, &Linked, &TextLayoutInfo, &ComputedTextBlock, &ComputedNode, &UiGlobalTransform, &InheritedVisibility, Option<&CalculatedClip>, Option<&InTip>)>,
    spans: &Query<&TextSpan>,
) -> Vec<Cand> {
    let mut out = Vec::new();
    for (e, tip, node, gt, vis, clip, intip) in tip_q {
        if !vis.get() || node.is_empty() {
            continue;
        }
        let level = intip.map_or(-1, |t| t.0 as i32);
        out.push(Cand { src: Src { e, section: WHOLE, id: id_of(level, "tip", &tip.id) }, level, rect: node_rect(node, gt), clip: clip.cloned(), link: None, tip: Some(tip.body.clone()), label: tip.label.clone(), inv: node.inverse_scale_factor });
    }
    for (e, linked, layout, block, node, gt, vis, clip, intip) in linked_q {
        if !vis.get() || node.is_empty() || linked.0.iter().all(Option::is_none) {
            continue;
        }
        let origin = node.content_box().min;
        for run in &layout.run_geometry {
            let Some(Some(link)) = linked.0.get(run.section_index as usize) else { continue };
            let level = intip.map_or(-1, |t| t.0 as i32);
            let a = gt.transform_point2(origin + run.bounds.min);
            let b = gt.transform_point2(origin + run.bounds.max);
            let label = block.entities().get(run.section_index as usize).and_then(|s| spans.get(s.entity).ok()).map_or(String::new(), |s| s.0.clone());
            let src = Src { e, section: run.section_index, id: id_of(level, "span", &label) };
            out.push(Cand { src, level, rect: Rect::from_corners(a, b), clip: clip.cloned(), link: Some(link.clone()), tip: None, label, inv: node.inverse_scale_factor });
        }
    }
    out
}

/// What a source opens: its own body, or one built now for the link it is.
fn body_of(c: &Cand, g: &Glossary, data: &Data, k: usize, fam: Option<&crate::family::FamilyView>) -> Option<TipBody> {
    if let Some(t) = &c.tip {
        return Some((**t).clone());
    }
    match c.link.as_ref()? {
        Link::Term(key) => Some(TipBody::term(g, key, None, Vec::new())),
        Link::Method(m) => crate::firm_card::method_tip(&data.r, *m, fam),
        Link::Town(t) => crate::firm_card::town_tip(&data.r, *t, k),
        Link::Good(gd) => crate::firm_card::good_tip(&data.r, *gd, k),
        Link::Wiki(_) => None,
    }
}

/// Open a tooltip's card, hidden until it is placed.
fn spawn_card(commands: &mut Commands, font: &Handle<Font>, out: Option<&Handle<Image>>, body: &TipBody, level: usize) -> Entity {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(if body.small { SMALL_WIDTH } else { WIDTH }),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                padding: UiRect::new(Val::Px(14.0), Val::Px(14.0), Val::Px(10.0), Val::Px(12.0)),
                border: UiRect::all(Val::Px(BORDER_W)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(TIP_PAPER),
            BorderColor::all(BORDER),
            BoxShadow::new(Color::srgba(0.25, 0.18, 0.10, 0.22), Val::Px(0.0), Val::Px(3.0), Val::Px(0.0), Val::Px(10.0)),
            GlobalZIndex(Z + level as i32),
            Visibility::Hidden,
            Interaction::default(),
            FocusPolicy::Block,
            TipCard,
        ))
        .with_children(|p| {
            // the term in small capitals, the ring at the right
            p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() }).with_children(|h| {
                let tf = TextFont { font: font.clone().into(), font_size: FontSize::Px(TERM_SIZE), ..default() };
                h.spawn((Text::new(body.term.to_uppercase()), tf, TextColor(MUTED)));
                h.spawn(Node { width: Val::Px(14.0), height: Val::Px(14.0), flex_shrink: 0.0, ..default() }).with_children(|r| {
                    for i in 0..RING_DOTS {
                        let a = i as f32 / RING_DOTS as f32 * std::f32::consts::TAU;
                        let (x, y) = (7.0 + 5.6 * a.sin() - 1.3, 7.0 - 5.6 * a.cos() - 1.3);
                        r.spawn((
                            Node { position_type: PositionType::Absolute, left: Val::Px(x), top: Val::Px(y), width: Val::Px(2.6), height: Val::Px(2.6), border_radius: BorderRadius::all(Val::Px(1.3)), ..default() },
                            BackgroundColor(RING_EMPTY),
                            RingDot(i),
                        ));
                    }
                    r.spawn((
                        Node { position_type: PositionType::Absolute, left: Val::Px(4.5), top: Val::Px(4.5), width: Val::Px(5.0), height: Val::Px(5.0), border_radius: BorderRadius::all(Val::Px(2.5)), ..default() },
                        BackgroundColor(RING_FULL),
                        Visibility::Hidden,
                        LockDot,
                    ));
                });
            });
            if let Some(v) = &body.value {
                let tf = TextFont { font: font.clone().into(), font_size: FontSize::Px(VALUE_SIZE), ..default() };
                p.spawn((Text::new(v.clone()), tf, TextColor(INK)));
            }
            if !body.says.is_empty() {
                spawn_pieces(p, font, &body.says, TEXT_SIZE, INK, Some(level), Node::default());
            }
            if !body.rows.is_empty() {
                p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(2.0), margin: UiRect::top(Val::Px(2.0)), ..default() }).with_children(|rows| {
                    for (a, b) in &body.rows {
                        rows.spawn(Node { justify_content: JustifyContent::SpaceBetween, column_gap: Val::Px(12.0), ..default() }).with_children(|r| {
                            spawn_pieces(r, font, a, ROW_SIZE, MUTED, Some(level), Node { flex_shrink: 0.0, ..default() });
                            spawn_pieces(r, font, b, ROW_SIZE, INK, Some(level), Node { flex_shrink: 1.0, ..default() });
                        });
                    }
                });
            }
            if !body.links.is_empty() {
                p.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(2.0)), ..default() }, BackgroundColor(RULE)));
                p.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(12.0), row_gap: Val::Px(2.0), ..default() }).with_children(|l| {
                    for (link, name) in &body.links {
                        l.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(2.0), ..default() }).with_children(|w| {
                            spawn_pieces(w, font, &[Piece::Link(link.clone(), name.clone())], ROW_SIZE, INK, Some(level), Node::default());
                            if let (Link::Wiki(_), Some(h)) = (link, out) {
                                w.spawn(crate::icons::icon(h, 9.0, WIKI_PLUM));
                            }
                        });
                    }
                });
            }
        })
        .id()
}

/// Where a tooltip of size `size` stands by a source at `src`, in a view of
/// `view`, all in logical pixels: beside the source on the side with more
/// room, else under or over it; never over the source, and inside the view.
pub fn place(src: Rect, size: Vec2, view: Vec2) -> Vec2 {
    let fits_x = |x: f32| x >= EDGE && x + size.x <= view.x - EDGE;
    let fits_y = |y: f32| y >= EDGE && y + size.y <= view.y - EDGE;
    let clamp_x = |x: f32| x.clamp(EDGE, (view.x - EDGE - size.x).max(EDGE));
    let clamp_y = |y: f32| y.clamp(EDGE, (view.y - EDGE - size.y).max(EDGE));
    let left = src.min.x - GAP - size.x;
    let right = src.max.x + GAP;
    let (first, second) = if src.center().x > view.x / 2.0 { (left, right) } else { (right, left) };
    for x in [first, second] {
        if fits_x(x) {
            return Vec2::new(x, clamp_y(src.min.y - 10.0));
        }
    }
    let below = src.max.y + GAP;
    if fits_y(below) {
        return Vec2::new(clamp_x(src.min.x), below);
    }
    let above = src.min.y - GAP - size.y;
    if fits_y(above) {
        return Vec2::new(clamp_x(src.min.x), above);
    }
    // nowhere it fits whole: at the side with more room, inside the view
    Vec2::new(clamp_x(first), clamp_y(src.min.y))
}

/// Where a tooltip opened from a word of another stands: wholly beside its
/// parent, left or right as there is room, its top level with the word's
/// line; else wholly under or over that line; and only if neither fits, where
/// `place` puts it, never over the word itself.
pub fn place_nested(src: Rect, parent: Rect, size: Vec2, view: Vec2) -> Vec2 {
    let fits_x = |x: f32| x >= EDGE && x + size.x <= view.x - EDGE;
    let fits_y = |y: f32| y >= EDGE && y + size.y <= view.y - EDGE;
    let clamp_x = |x: f32| x.clamp(EDGE, (view.x - EDGE - size.x).max(EDGE));
    let clamp_y = |y: f32| y.clamp(EDGE, (view.y - EDGE - size.y).max(EDGE));
    let left = parent.min.x - GAP - size.x;
    let right = parent.max.x + GAP;
    let sides = if parent.min.x > view.x - parent.max.x { [left, right] } else { [right, left] };
    for x in sides {
        if fits_x(x) {
            return Vec2::new(x, clamp_y(src.min.y - 10.0));
        }
    }
    if fits_y(src.max.y + GAP) {
        return Vec2::new(clamp_x(src.min.x), src.max.y + GAP);
    }
    if fits_y(src.min.y - GAP - size.y) {
        return Vec2::new(clamp_x(src.min.x), src.min.y - GAP - size.y);
    }
    place(src, size, view)
}

/// Open a link a click lands on: a town or a good is gone to, a Wikipedia article opened in the browser.
fn follow(link: &Link, data: &Data, picked: &mut Picked, orbit: &mut Orbit, ui: &mut UiState, ov: &mut crate::overlay::MapOverlay, window: bool) {
    match link {
        Link::Town(t) => {
            if let Some((x, z)) = data.r.header.towns.get(*t).and_then(|town| data.cen.get(town.node)).copied() {
                orbit.target = Vec3::new(x, crate::world::height_at(&data.g, x, z).max(0.0), z);
                orbit.dist = orbit.dist.min(8.0);
            }
            picked.what = Some(Pick::Town(*t));
            picked.at = None;
            picked.stamp = picked.stamp.wrapping_add(1);
        }
        Link::Good(g) => {
            ui.tab = Some(crate::ui::Tab::Markets);
            ui.markets = crate::ui::MarketsShow::Prices;
            ui.good = Some(*g);
            ui.dirty += 1;
            ov.good = *g;
        }
        Link::Wiki(title) => {
            // a picture or a script has no window, and never opens a browser
            if window {
                open_wiki(title);
            }
        }
        Link::Term(_) | Link::Method(_) => {}
    }
}

/// The address of a Wikipedia article, from its title as the glossary keeps it.
pub fn wiki_url(title: &str) -> String {
    let mut s = String::from("https://en.wikipedia.org/wiki/");
    for b in title.replace(' ', "_").bytes() {
        if b.is_ascii_alphanumeric() || b"_-.()'".contains(&b) {
            s.push(b as char);
        } else {
            s.push_str(&format!("%{b:02X}"));
        }
    }
    s
}

fn open_wiki(title: &str) {
    let url = wiki_url(title);
    // the system's own way to open an address, on a thread of its own so that the view does not wait
    std::thread::spawn(move || {
        let r = if cfg!(target_os = "windows") {
            std::process::Command::new("cmd").args(["/C", "start", "", &url]).spawn()
        } else if cfg!(target_os = "macos") {
            std::process::Command::new("open").arg(&url).spawn()
        } else {
            std::process::Command::new("xdg-open").arg(&url).spawn()
        };
        if let Err(e) = r {
            eprintln!("could not open {url}: {e}");
        }
    });
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update(
    mut commands: Commands,
    mut tips: ResMut<Tips>,
    time: Res<Time<Real>>,
    (font, data, tl, fam, icons): (Res<UiFont>, Res<Data>, Res<Timeline>, Option<Res<crate::family::FamilyView>>, Option<Res<crate::icons::Icons>>),
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cams: Query<&Camera, With<MainCam>>,
    buttons: Res<ButtonInput<MouseButton>>,
    tip_q: Query<(Entity, &Tip, &ComputedNode, &UiGlobalTransform, &InheritedVisibility, Option<&CalculatedClip>, Option<&InTip>)>,
    linked_q: Query<(Entity, &Linked, &TextLayoutInfo, &ComputedTextBlock, &ComputedNode, &UiGlobalTransform, &InheritedVisibility, Option<&CalculatedClip>, Option<&InTip>)>,
    spans: Query<&TextSpan>,
    mut cards: Query<(Entity, &TipCard, &ComputedNode, &UiGlobalTransform, &mut Node, &mut Visibility, &mut BorderColor), Without<LockDot>>,
    mut dots: Query<(&RingDot, &ChildOf, &mut BackgroundColor), Without<TipCard>>,
    mut lock_dots: Query<(&ChildOf, &mut Visibility), (With<LockDot>, Without<TipCard>)>,
    parents: Query<&ChildOf>,
    (mut picked, mut orbit, mut ui, mut ov): (ResMut<Picked>, ResMut<Orbit>, ResMut<UiState>, ResMut<crate::overlay::MapOverlay>),
) {
    let t0 = Instant::now();
    let now = time.elapsed_secs();
    let window = windows.single().ok();
    let cands = candidates(&tip_q, &linked_q, &spans);
    // the pointer, in physical pixels: a script's aim, or the mouse
    let pointer = match &tips.aim {
        Some(label) => {
            let l = label.to_lowercase();
            // the topmost, and of those the first on the screen
            let named: Vec<&Cand> = cands.iter().filter(|c| c.label.trim().to_lowercase() == l).collect();
            let top = named.iter().map(|c| c.level).max();
            // (a link before the row it stands in, as the pointer finds it)
            named.into_iter().filter(|c| Some(c.level) == top).min_by(|a, b| (a.src.section == WHOLE).cmp(&(b.src.section == WHOLE)).then(a.rect.min.y.total_cmp(&b.rect.min.y))).map(|c| c.rect.center())
        }
        None => window.and_then(|w| w.physical_cursor_position()),
    };
    let middle = std::mem::take(&mut tips.middle) || buttons.just_pressed(MouseButton::Middle);
    let left = buttons.just_pressed(MouseButton::Left);
    // the source under the pointer: in the topmost tooltip, a span before the node it lies in
    let hit = pointer.and_then(|p| cands.iter().filter(|c| c.rect.contains(p) && c.clip.as_ref().is_none_or(|cl| cl.contains_point(p))).max_by_key(|c| (c.level, c.src.section != WHOLE)));
    // which open tooltips the pointer is over
    let over: Vec<bool> = tips.stack.iter().map(|o| pointer.is_some_and(|p| cards.get(o.card).is_ok_and(|(_, _, n, gt, ..)| node_rect(n, gt).contains(p)))).collect();
    let hit_src = hit.map(|c| c.src);
    // the rest
    match (tips.rest, hit_src) {
        (Some((s, _)), Some(h)) if s == h => {}
        (_, Some(h)) => tips.rest = Some((h, now)),
        (_, None) => tips.rest = None,
    }
    let rested = tips.rest.map_or(0.0, |r| now - r.1);
    // a click outside the stack closes it (a click on a link still follows the link)
    if left && tips.open() && !over.iter().any(|&o| o) {
        tips.close_from(&mut commands, 0);
    }
    // close from the top: an open tooltip goes when the pointer leaves its source; a locked one
    // a moment after the pointer has left it, its source and every tooltip above it
    let mut i = tips.stack.len();
    while i > 0 {
        i -= 1;
        let on_source = hit_src == Some(tips.stack[i].source);
        let kept = on_source || over[i..over.len().min(tips.stack.len())].iter().any(|&o| o);
        let o = &mut tips.stack[i];
        if !o.locked {
            if !on_source {
                tips.close_from(&mut commands, i);
            }
        } else if kept {
            o.away = None;
        } else {
            let away = *o.away.get_or_insert(now);
            if now - away >= CLOSE_AFTER {
                tips.close_from(&mut commands, i);
            }
        }
    }
    if let Some(c) = hit {
        let slot = (c.level + 1) as usize;
        // a tooltip opens from the cards, or from a tooltip that is locked
        let parent_ok = c.level < 0 || tips.stack.get(c.level as usize).is_some_and(|o| o.locked);
        let at_once = middle || (left && matches!(c.link, Some(Link::Term(_)) | Some(Link::Method(_))));
        let is_open = tips.stack.get(slot).is_some_and(|o| o.source == c.src);
        if parent_ok && !is_open && (rested >= OPEN_AFTER || at_once) {
            let k = (tl.t.floor() as usize).min(data.r.ny - 1);
            if let Some(body) = body_of(c, words::glossary(), &data, k, fam.as_deref()) {
                let from = slot.min(tips.stack.len());
                tips.close_from(&mut commands, from);
                let card = spawn_card(&mut commands, &font.0, icons.as_ref().map(|i| &i.out), &body, slot);
                let rect = Rect { min: c.rect.min * c.inv, max: c.rect.max * c.inv };
                tips.stack.push(Open { source: c.src, card, locked: false, away: None, rect, placed: false, words: body.words() });
            }
        }
        if let Some(o) = tips.stack.get_mut(slot).filter(|o| o.source == c.src) {
            if !o.locked && (rested >= LOCK_AFTER || at_once) {
                o.locked = true;
            }
        }
        if left {
            if let Some(l) = &c.link {
                if matches!(l, Link::Town(_) | Link::Good(_)) {
                    tips.close_from(&mut commands, 0);
                }
                follow(l, &data, &mut picked, &mut orbit, &mut ui, &mut ov, window.is_some());
            }
        }
    }
    // each open tooltip: placed once its size is known, its ring and its border
    let view = cams.iter().find_map(|c| c.logical_viewport_size()).unwrap_or(Vec2::new(1600.0, 1000.0));
    let n = tips.stack.len();
    // each tooltip's own rectangle, in logical pixels, for the one opened from it
    let rects: Vec<Option<Rect>> = tips.stack.iter().map(|o| cards.get(o.card).ok().filter(|(_, _, nd, ..)| nd.size.x > 0.0).map(|(_, _, nd, gt, ..)| { let r = node_rect(nd, gt); Rect { min: r.min * nd.inverse_scale_factor, max: r.max * nd.inverse_scale_factor } })).collect();
    for (idx, o) in tips.stack.iter_mut().enumerate() {
        let Ok((e, _, node, _, mut nd, mut vis, mut border)) = cards.get_mut(o.card) else { continue };
        if !o.placed && node.size.x > 0.0 {
            let size = node.size * node.inverse_scale_factor;
            let at = match idx.checked_sub(1).and_then(|p| rects.get(p).copied().flatten()) {
                Some(parent) => place_nested(o.rect, parent, size, view),
                None => place(o.rect, size, view),
            };
            nd.left = Val::Px(at.x);
            nd.top = Val::Px(at.y);
            *vis = Visibility::Inherited;
            o.placed = true;
        }
        border.set_if_neq(BorderColor::all(if o.locked { BORDER_LOCKED } else { BORDER }));
        // the ring fills from OPEN_AFTER to LOCK_AFTER, and is full with a dot once locked
        let fill = if o.locked { 1.0 } else if idx + 1 == n { ((rested - OPEN_AFTER) / (LOCK_AFTER - OPEN_AFTER)).clamp(0.0, 1.0) } else { 0.0 };
        let lit = (fill * RING_DOTS as f32).round() as usize;
        for (d, parent, mut bg) in &mut dots {
            if card_of(parent.parent(), &parents) == Some(e) {
                bg.set_if_neq(BackgroundColor(if d.0 < lit { RING_FULL } else { RING_EMPTY }));
            }
        }
        for (parent, mut v) in &mut lock_dots {
            if card_of(parent.parent(), &parents) == Some(e) {
                v.set_if_neq(if o.locked { Visibility::Inherited } else { Visibility::Hidden });
            }
        }
    }
    let us = t0.elapsed().as_secs_f32() * 1e6;
    let slot = tips.stack.len().min(3);
    let cost = &mut tips.cost[slot];
    if cost.len() < 100_000 {
        cost.push(us);
    }
}

/// The tooltip's card a ring's node is in: the ring's parent is in the header, whose parent is the card.
fn card_of(ring: Entity, parents: &Query<&ChildOf>) -> Option<Entity> {
    let header = parents.get(ring).ok()?.parent();
    Some(parents.get(header).ok()?.parent())
}

/// A card's rows light under the pointer and while their tooltip is open.
fn lit_rows(tips: Res<Tips>, mut q: Query<(Entity, &mut BackgroundColor), With<LitRow>>) {
    let open: Vec<Entity> = tips.stack.iter().map(|o| o.source.e).chain(tips.rest.map(|r| r.0.e)).collect();
    for (e, mut bg) in &mut q {
        bg.set_if_neq(BackgroundColor(if open.contains(&e) { SOURCE_LIT } else { Color::NONE }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tooltip_never_covers_its_source_and_stays_in_the_view() {
        let view = Vec2::new(1600.0, 1000.0);
        let size = Vec2::new(300.0, 180.0);
        for src in [
            // a row of the card at the right, a word at the left, one at the bottom right, one at the top
            Rect::new(1240.0, 300.0, 1560.0, 322.0),
            Rect::new(20.0, 500.0, 90.0, 516.0),
            Rect::new(1450.0, 960.0, 1540.0, 976.0),
            Rect::new(700.0, 10.0, 760.0, 26.0),
            // a source as wide as the view: the tooltip goes under or over it
            Rect::new(0.0, 400.0, 1600.0, 420.0),
        ] {
            let at = place(src, size, view);
            let tip = Rect::from_corners(at, at + size);
            assert!(tip.intersect(src).is_empty(), "{src:?}: the tooltip at {at:?} covers its source");
            assert!(tip.min.x >= EDGE - 0.01 && tip.min.y >= EDGE - 0.01 && tip.max.x <= view.x - EDGE + 0.01 && tip.max.y <= view.y - EDGE + 0.01, "{src:?}: {tip:?} leaves the view");
        }
    }

    #[test]
    fn a_nested_tooltip_stands_beside_its_parent_and_never_over_its_word() {
        let view = Vec2::new(1600.0, 1000.0);
        let size = Vec2::new(300.0, 180.0);
        // a parent to the left of the card: the child goes to its left, level with the word's line
        let parent = Rect::new(900.0, 500.0, 1200.0, 700.0);
        let word = Rect::new(1100.0, 580.0, 1160.0, 596.0);
        let at = place_nested(word, parent, size, view);
        let child = Rect::from_corners(at, at + size);
        assert!(child.intersect(parent).is_empty(), "{child:?} covers its parent");
        assert!((at.y - (word.min.y - 10.0)).abs() < 0.01);
        // with no room beside it, under the word's line and not over the word
        let parent = Rect::new(200.0, 300.0, 1400.0, 500.0);
        let word = Rect::new(600.0, 340.0, 660.0, 356.0);
        let at = place_nested(word, parent, size, view);
        let child = Rect::from_corners(at, at + size);
        assert!(child.intersect(word).is_empty() && at.y >= word.max.y, "{child:?}");
    }

    #[test]
    fn a_terms_tooltip_has_its_parts_in_order_and_links_to_what_it_sees() {
        let g = words::glossary();
        let b = TipBody::term(g, "ground_rent", Some("\u{a3}2.39/mo".into()), vec![row("Bid in 1799", "\u{a3}2.39/mo")]);
        assert_eq!(b.term, "Ground rent");
        assert!(b.says.iter().any(|p| matches!(p, Piece::Link(Link::Term(k), s) if k == "bid_rent" && s == "bid rent")), "{:?}", b.says);
        assert_eq!(b.links.first(), Some(&(Link::Term("bid_rent".into()), "Bid rent".into())));
        assert!(b.links.contains(&(Link::Wiki("Economic rent".into()), "Economic rent".into())));
        assert!(b.words().starts_with("GROUND RENT | \u{a3}2.39/mo | What a firm pays each month"), "{}", b.words());
        assert_eq!(wiki_url("Johann Heinrich von Th\u{fc}nen"), "https://en.wikipedia.org/wiki/Johann_Heinrich_von_Th%C3%BCnen");
    }
}
