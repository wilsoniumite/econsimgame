//! The brush on the ground (GROUND-BRUSH; `docs/designs/sandbox.md`, "Built:
//! the brush on the ground"; the core's `edits::Ground`). The tray's Ground
//! tool (the land's icon) has three rows: the kind (Coal, Iron ore, Clay,
//! Stone, Timber), the size (Small, Middling, Large: three, five and seven
//! squares across) and the way (Lay down, Take away). Taken up, it shows the
//! Deposits view of the kind in hand, and puts back the view there was when
//! it is put down.
//!
//! With it in hand a press on the map, or a drag, paints: each square the
//! pointer passes over joins the stroke (the squares between two frames'
//! squares too, so a quick drag leaves no gaps), and the view shows at once
//! what the stroke would leave there, worked out by the core's own function
//! (`edits::stroke`, through `overlay::preview_stroke`) on the ground as the
//! viewer holds it. The left button paints instead of moving the camera; the
//! right still turns it and the keys still move it. When the button is let
//! go the whole stroke goes to the simulation as one order
//! (`Edit::Ground`), under the tray's guards, and its answer goes up as a
//! toast as any tool's does. The preview stays over the view until the year
//! after it comes with the island's own figures (`Holds::now`); a stroke the
//! simulation refuses takes its preview away again.
//!
//! A script paints with `stroke:X,Z,X,Z,...`: the brush is drawn from point
//! to point a square a frame, as a hand would drag it, and let go at the end.

use std::collections::VecDeque;

use bevy::prelude::*;
use isostate_core::edits::{Edit, Ground};

use crate::game::Game;
use crate::live::Live;
use crate::overlay::{MapOverlay, Overlay, Preview};
use crate::sandbox::{Sandbox, Tool};
use crate::ui::UiState;
use crate::{Data, MainCam, Timeline, Toasts};

/// The words of the kinds' chips, in the order of `edits::Ground::ALL` and of
/// the Deposits view's first five kinds (`overlay::KINDS`).
pub const KIND_WORDS: [&str; 5] = ["Coal", "Iron ore", "Clay", "Stone", "Timber"];
/// The words of the sizes' chips, and how far each reaches from a square the
/// brush passes over (`edits::stroke`): three, five and seven squares across.
pub const SIZE_WORDS: [&str; 3] = ["Small", "Middling", "Large"];
pub const SIZES: [u8; 3] = [1, 2, 3];
const ACROSS: [&str; 3] = ["three", "five", "seven"];

/// The tool's line: what a press or a drag will now do.
pub fn hint(kind: usize, size: usize, lay: bool) -> String {
    let g = Ground::ALL[kind.min(Ground::ALL.len() - 1)];
    let wide = ACROSS[size.min(ACROSS.len() - 1)];
    match (g, lay) {
        (Ground::Timber, true) => format!("Press or drag on the map: trees are planted under the brush, {wide} squares across, wherever woods may stand. The woods are cut and grow by their own rule."),
        (Ground::Timber, false) => format!("Press or drag on the map: the trees under the brush, {wide} squares across, are taken away, and no timber comes of them."),
        (_, true) => format!(
            "Press or drag on the map: {} is laid down under the brush, {wide} squares across, as rich as the island's own best. No pit is opened by it: the island finds it at the year's review, and works it if that pays.",
            g.label()
        ),
        (_, false) => format!(
            "Press or drag on the map: the {} under the brush, {wide} squares across, is taken away. A working on it raises nothing more there, and the mines' rule gives it up once it has stood idle.",
            g.label()
        ),
    }
}

/// What a stroke did, in words that follow "before" on the tray's line for
/// taking it back: "coal was laid down round 31, 40, in Melbourne's district".
pub fn before_words(kind: Ground, lay: bool, squares: &[usize], at: &dyn Fn(usize) -> String, near: &dyn Fn(usize) -> Option<String>) -> String {
    let place = match squares.get(squares.len() / 2) {
        Some(&c) => match near(c) {
            Some(t) => format!("round {}, in {t}'s district", at(c)),
            None => format!("round {}", at(c)),
        },
        None => "on the island".to_string(),
    };
    match (kind, lay) {
        (Ground::Timber, true) => format!("trees were planted {place}"),
        (Ground::Timber, false) => format!("the trees {place} were taken away"),
        (_, true) => format!("{} was laid down {place}", kind.label()),
        (_, false) => format!("the {} {place} was taken away", kind.label()),
    }
}

/// The squares on a straight line from square `a` to square `b`, both
/// included, on an island `w` squares wide: each one a step from the last,
/// sideways, up or down, or across a corner.
pub fn line(a: usize, b: usize, w: usize) -> Vec<usize> {
    let w = w.max(1);
    let (x0, y0, x1, y1) = ((a % w) as i64, (a / w) as i64, (b % w) as i64, (b / w) as i64);
    let n = (x1 - x0).abs().max((y1 - y0).abs());
    let mut out: Vec<usize> = Vec::with_capacity(n as usize + 1);
    for i in 0..=n {
        let t = if n == 0 { 0.0 } else { i as f64 / n as f64 };
        let (x, y) = ((x0 as f64 + t * (x1 - x0) as f64).round() as usize, (y0 as f64 + t * (y1 - y0) as f64).round() as usize);
        let c = y * w + x;
        if out.last() != Some(&c) {
            out.push(c);
        }
    }
    out
}

/// A stroke under way.
struct Stroke {
    kind: usize,
    size: u8,
    lay: bool,
    /// the squares the brush has passed over, in order
    squares: Vec<usize>,
    /// how rich it lays, once its first square is painted
    rich: Option<f64>,
    /// the preview as it stood before the stroke, put back if it is refused
    before: Preview,
    /// how long its previews took to work out, in ms, all told and the longest
    cost: (f32, f32),
}

/// The brush: the stroke under way, and what a script asks of it.
#[derive(Resource, Default)]
pub struct Brush {
    stroke: Option<Stroke>,
    /// a stroke sent and not yet answered: the preview before it
    out: Option<Preview>,
    /// a script's stroke: the squares still to come, one a frame
    script: VecDeque<usize>,
    scripting: bool,
    /// the tool is in hand (camera_input lets the left button paint)
    held: bool,
    /// the view before the tool was taken up, and the kind it last showed
    view: Option<(Option<Overlay>, usize)>,
    shown: Option<usize>,
}

impl Brush {
    /// The brush is in hand: a drag with the left button paints.
    pub fn in_hand(&self) -> bool {
        self.held
    }

    /// A script's stroke: the brush is drawn over these squares, one a frame.
    pub fn script(&mut self, squares: Vec<usize>) {
        self.script = squares.into();
        self.scripting = true;
    }

    /// A script's stroke is under way.
    pub fn scripting(&self) -> bool {
        self.scripting
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Brush>().add_systems(
        Update,
        (show_kind, paint.after(crate::game::pump).before(crate::overlay::paint), answered.after(crate::game::pump))
            .chain()
            .run_if(resource_exists::<Sandbox>)
            .run_if(resource_exists::<Live>),
    );
}

/// The Deposits view of the kind in hand while the tool is up; the view there
/// was before, when it is put down.
fn show_kind(sb: Res<Sandbox>, mut ov: ResMut<MapOverlay>, mut brush: ResMut<Brush>) {
    match sb.brush() {
        Some((kind, _, _)) => {
            brush.held = true;
            if brush.view.is_none() {
                brush.view = Some((ov.kind, ov.deposit));
            }
            if brush.shown != Some(kind) {
                ov.set(Some(Overlay::Deposits));
                ov.deposit = kind;
                brush.shown = Some(kind);
            }
        }
        None => {
            brush.held = false;
            brush.stroke = None;
            brush.shown = None;
            if let Some((kind, deposit)) = brush.view.take() {
                if ov.kind == Some(Overlay::Deposits) {
                    ov.set(kind);
                    ov.deposit = deposit;
                }
            }
        }
    }
}

/// A press, a drag and a let go with the brush in hand (or a script's stroke).
#[allow(clippy::too_many_arguments)]
fn paint(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cams: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    ui: Res<UiState>,
    mut sb: ResMut<Sandbox>,
    data: Res<Data>,
    tl: Res<Timeline>,
    live: Res<Live>,
    mut game: ResMut<Game>,
    mut ov: ResMut<MapOverlay>,
    mut toasts: ResMut<Toasts>,
    time: Res<Time>,
    mut brush: ResMut<Brush>,
) {
    let Some((kind, size, lay)) = sb.brush() else {
        if brush.scripting {
            println!("script: no stroke: the Ground tool is not in hand (click:Ground first)");
            (brush.scripting, brush.script) = (false, VecDeque::new());
        }
        return;
    };
    let now = time.elapsed_secs();
    let (mut begin, mut next, mut release) = (false, None, false);
    if brush.scripting {
        match brush.script.pop_front() {
            Some(c) => {
                begin = brush.stroke.is_none();
                next = Some(c);
            }
            None => release = true,
        }
    } else {
        begin = buttons.just_pressed(MouseButton::Left) && !ui.hover;
        if buttons.pressed(MouseButton::Left) && (begin || brush.stroke.is_some()) {
            next = square_under_pointer(&windows, &cams, &data);
        }
        release = buttons.just_released(MouseButton::Left);
    }
    if begin {
        match crate::sandbox::may_act(&sb, &tl, &data, &live, &game) {
            Ok(()) => brush.stroke = Some(Stroke { kind, size: SIZES[size.min(SIZES.len() - 1)], lay, squares: Vec::new(), rich: None, before: ov.preview.clone(), cost: (0.0, 0.0) }),
            Err(why) => {
                toasts.say("orders_refused", why, "By hand", now);
                if brush.scripting {
                    println!("script: no stroke: {why}");
                    (brush.scripting, brush.script) = (false, VecDeque::new());
                }
                return;
            }
        }
    }
    if let Some(c) = next.filter(|_| brush.stroke.is_some()) {
        add(&mut brush, &mut ov, &data, c);
    }
    if release {
        brush.scripting = false;
        let Some(s) = brush.stroke.take() else { return };
        if s.squares.is_empty() {
            return;
        }
        let edit = Edit::Ground { kind: Ground::ALL[s.kind], lay: s.lay, size: s.size, squares: s.squares.clone() };
        println!(
            "ground: a stroke of {} squares sent ({}, {}); its previews took {:.1} ms on the main thread, {:.2} ms at most in a frame",
            s.squares.len(),
            KIND_WORDS[s.kind].to_lowercase(),
            if s.lay { "laid down" } else { "taken away" },
            s.cost.0,
            s.cost.1
        );
        match crate::sandbox::send_as(Tool::Ground, edit, &mut sb, &tl, &data, &live, &mut game) {
            Ok(()) => brush.out = Some(s.before),
            Err(why) => {
                toasts.say("orders_refused", why, "By hand", now);
                ov.set_preview(s.before);
            }
        }
    }
}

/// The square of land under the pointer, if any.
fn square_under_pointer(windows: &Query<&Window, With<bevy::window::PrimaryWindow>>, cams: &Query<(&Camera, &GlobalTransform), With<MainCam>>, data: &Data) -> Option<usize> {
    let cursor = windows.single().ok()?.cursor_position()?;
    let (cam, cam_t) = cams.single().ok()?;
    let ray = cam.viewport_to_world(cam_t, cursor).ok()?;
    let hit = crate::ground_hit(&data.g, ray)?;
    let r = &data.r;
    (hit.x >= 0.0 && hit.z >= 0.0 && (hit.x as usize) < r.w && (hit.z as usize) < r.h).then(|| hit.z as usize * r.w + hit.x as usize)
}

/// A square joins the stroke, with those on the way to it from the last, and
/// the preview shows what they would hold.
fn add(brush: &mut Brush, ov: &mut MapOverlay, data: &Data, c: usize) {
    let r = &data.r;
    let Some(s) = brush.stroke.as_mut() else { return };
    let new: Vec<usize> = match s.squares.last() {
        Some(&last) if last == c => return,
        Some(&last) => line(last, c, r.w).into_iter().skip(1).collect(),
        None => vec![c],
    };
    s.squares.extend(new.iter().copied());
    let t0 = std::time::Instant::now();
    let k = data.latest();
    let mut p = ov.preview.clone();
    if p.from != k {
        p = Preview { from: k, cells: Vec::new() };
    }
    if let Some((changed, rich)) = crate::overlay::preview_stroke(r, k, s.kind, &p, s.lay, s.size, &new, s.rich) {
        s.rich.get_or_insert(rich);
        for (cell, v) in changed {
            match p.cells.iter_mut().find(|x| x.0 == s.kind && x.1 == cell) {
                Some(x) => x.2 = v,
                None => p.cells.push((s.kind, cell, v)),
            }
        }
    }
    ov.set_preview(p);
    let ms = t0.elapsed().as_secs_f32() * 1e3;
    s.cost = (s.cost.0 + ms, s.cost.1.max(ms));
}

/// A stroke's answer: refused, its preview goes.
fn answered(game: Res<Game>, mut brush: ResMut<Brush>, mut ov: ResMut<MapOverlay>) {
    if brush.out.is_none() || game.busy {
        return;
    }
    let before = brush.out.take().unwrap_or_default();
    if matches!(game.note, Some((false, _))) {
        ov.set_preview(before);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_of_squares_leaves_no_gaps_and_the_words_say_what_the_brush_does() {
        // on an island ten squares wide, from 1, 1 to 5, 3
        let l = line(11, 35, 10);
        assert_eq!((l.first(), l.last(), l.len()), (Some(&11), Some(&35), 5));
        for p in l.windows(2) {
            let (a, b) = (p[0] as i64, p[1] as i64);
            assert!(((a % 10) - (b % 10)).abs() <= 1 && ((a / 10) - (b / 10)).abs() <= 1, "{l:?}");
        }
        assert_eq!(line(7, 7, 10), vec![7]);
        assert!(hint(0, 1, true).contains("coal is laid down") && hint(0, 1, true).contains("five squares across"));
        assert!(hint(4, 0, false).contains("no timber"));
        let at = |c: usize| format!("{}, {}", c % 10, c / 10);
        let near = |_: usize| Some("Melbourne".to_string());
        assert_eq!(before_words(Ground::Coal, true, &[11, 12, 13], &at, &near), "coal was laid down round 2, 1, in Melbourne's district");
        assert_eq!(before_words(Ground::Stone, false, &[11], &at, &|_| None), "the stone round 1, 1 was taken away");
        assert_eq!(KIND_WORDS.len(), Ground::ALL.len());
        assert_eq!(SIZES.len(), SIZE_WORDS.len());
    }
}
