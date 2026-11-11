//! A scripted run, for checking what a still cannot show: keys, clicks and
//! timing. `--script "key:Space;wait:2;print"` runs the game with no window,
//! plays the steps in order as a player's presses and clicks, prints what the
//! clock, the cards and the order panel did, and stops. It takes its own
//! pictures (`shot:PATH`), and leaves the saved game alone unless
//! `ISOSTATE_SAVES` or `--save` gives it a place of its own.
//!
//! Steps, separated by semicolons:
//!
//!     key:NAME        press a key for one frame (Space, Left, Right, Up, Down,
//!                     PageUp, PageDown, Escape, a letter or a digit; Shift+M
//!                     with Shift held)
//!     click:LABEL     click the button that says LABEL (the smallest thing on
//!                     screen holding that text; a card open over it stops the click)
//!     decide:N        click option N of the open decision, counting from 0
//!     wait:SECONDS    let the game run
//!     frames:N        let N frames pass
//!     settle          wait until no year is being worked out, no order is out
//!                     and the scenery shows the year on the clock
//!     sample:SECONDS  let the game run and count, frame by frame, where the
//!                     clock stood and how far the season moved
//!     run:SECONDS     the same, keeping the years going: a decision that opens
//!                     is given its default and the clock is started again
//!     until:PART      let the game run until the clock is this far through a
//!                     year (until:0.95 is just before the year turns)
//!     print[:NOTE]    print the clock, the cards and the order panel, and what
//!                     scenery is on screen and what putting it there cost
//!     scenery[:NOTE]  build the year on the clock afresh and say whether what
//!                     is on screen is that; stops the run if it is not
//!     tab:NAME        open a side panel (family, holdings, markets, news, none)
//!     pick:X,Z        click the map at this spot
//!     look:X,Z,DIST   point the camera
//!     scroll:PIXELS   scroll the side panel down this far
//!     shot:PATH       save a picture
//!
//! and for the map views and the Markets panel:
//!
//!     chip:NAME       click a chip of the legend card: rent, areas, reach,
//!                     price, wages, plain, or pill for the Map views button
//!                     that stands there while the map is plain
//!     good:next       the legend's steps through the goods whose price is
//!     good:back       mapped
//!     row:GOOD        click a good's row in the Markets panel (row:coal)
//!     half:island     click a chip of the Markets panel: the island's charts,
//!     half:goods      or the goods
//!     map[:NOTE]      let the panels catch up, then print in one line what
//!                     the map and the Markets panel show
//!
//! The lines it prints start with "script:" and go to standard output.

// see the note at the top of ui.rs
#![allow(deprecated)]

use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::ui::{FocusPolicy, ScrollPosition, UiSystems};

use crate::family::FamilyView;
use crate::game::{ActionButton, DecideButton, Game};
use crate::live::Live;
use crate::overlay::{GoodStep, MapOverlay, Overlay, OverlayChip};
use crate::ui::{self, GoodRow, MarketsChip, MarketsShow, UiState};
use crate::{world, Data, InspectActions, MainCam, Orbit, Picked, Timeline};

#[derive(Clone, Debug, PartialEq)]
enum Step {
    /// the key, and whether Shift is held with it
    Key(KeyCode, bool),
    Click(String),
    Decide(usize),
    Wait(f32),
    Frames(u32),
    Settle,
    /// seconds, and whether to answer decisions and keep the clock going
    Sample(f32, bool),
    /// how far through a year the clock must be
    Until(f32),
    Print(String),
    Scenery(String),
    Tab(Option<ui::Tab>),
    Pick(f32, f32),
    Look(f32, f32, f32),
    Scroll(f32),
    Shot(String),
    /// a chip of the legend card: an overlay, or none for Plain map and the pill
    Chip(String),
    /// the legend's step through the goods: 1 on, -1 back
    Good(i32),
    /// a good's row in the Markets panel, by name
    Row(String),
    Half(MarketsShow),
    Map(String),
}

/// Frames a `map` step lets pass, so that a panel rebuilt after a press is there to be read.
const CATCH_UP: u32 = 8;

/// What a `sample` step counted.
#[derive(Default)]
struct Tally {
    frames: u32,
    /// frames with the clock running
    playing: u32,
    /// of those, frames in which an order given now would land in the year on screen
    can_act: u32,
    /// frames with the next year being worked out
    stepping: u32,
    /// how far round the year the season went, forwards and backwards
    season_fwd: f32,
    season_back: f32,
    last_phase: Option<f32>,
    /// the biggest move of the season between two frames
    season_jump: f32,
    dates: Vec<String>,
    k0: usize,
    /// decisions answered for the player
    answered: u32,
    /// start the clock again once the decision's card has gone
    resume: bool,
}

#[derive(Resource)]
struct Script {
    steps: Vec<Step>,
    at: usize,
    /// the step in hand has started and is waiting
    begun: bool,
    until: f32,
    frames: u32,
    /// the keys held this frame, to be let go of in the next
    keys: Vec<KeyCode>,
    clicked: Option<Entity>,
    tally: Tally,
    image: Handle<Image>,
    /// frames left before stopping, once the steps are done
    closing: Option<u32>,
}

fn key_code(name: &str) -> Option<KeyCode> {
    let n = name.to_ascii_lowercase();
    Some(match n.as_str() {
        "space" => KeyCode::Space,
        "left" => KeyCode::ArrowLeft,
        "right" => KeyCode::ArrowRight,
        "up" => KeyCode::ArrowUp,
        "down" => KeyCode::ArrowDown,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "escape" | "esc" => KeyCode::Escape,
        "period" => KeyCode::Period,
        "comma" => KeyCode::Comma,
        "1" => KeyCode::Digit1,
        "2" => KeyCode::Digit2,
        "3" => KeyCode::Digit3,
        "4" => KeyCode::Digit4,
        "a" => KeyCode::KeyA,
        "d" => KeyCode::KeyD,
        "e" => KeyCode::KeyE,
        "f" => KeyCode::KeyF,
        "g" => KeyCode::KeyG,
        "h" => KeyCode::KeyH,
        "m" => KeyCode::KeyM,
        "q" => KeyCode::KeyQ,
        "r" => KeyCode::KeyR,
        "s" => KeyCode::KeyS,
        "t" => KeyCode::KeyT,
        "w" => KeyCode::KeyW,
        "x" => KeyCode::KeyX,
        _ => return None,
    })
}

fn parse(s: &str) -> Result<Vec<Step>, String> {
    let mut steps = Vec::new();
    for part in s.split(';').map(str::trim).filter(|p| !p.is_empty()) {
        let (word, arg) = part.split_once(':').unwrap_or((part, ""));
        let num = |what: &str| arg.parse::<f32>().map_err(|_| format!("{part}: {what} must be a number"));
        let nums = || arg.split(',').map(|v| v.trim().parse::<f32>()).collect::<Result<Vec<f32>, _>>().map_err(|_| format!("{part}: numbers separated by commas"));
        steps.push(match word {
            "key" => {
                let (shift, name) = match arg.split_once('+') {
                    Some((with, name)) if with.eq_ignore_ascii_case("shift") => (true, name),
                    _ => (false, arg),
                };
                Step::Key(key_code(name).ok_or_else(|| format!("{part}: no key called {name}"))?, shift)
            }
            "click" if !arg.is_empty() => Step::Click(arg.to_string()),
            "decide" => Step::Decide(num("the option")? as usize),
            "wait" => Step::Wait(num("the seconds")?),
            "frames" => Step::Frames(num("the frames")? as u32),
            "settle" => Step::Settle,
            "sample" => Step::Sample(num("the seconds")?, false),
            "run" => Step::Sample(num("the seconds")?, true),
            "until" => match num("how far through the year")? {
                part if (0.0..1.0).contains(&part) => Step::Until(part),
                _ => return Err(format!("{part}: a part of a year runs from 0 to just under 1")),
            },
            "print" => Step::Print(arg.to_string()),
            "scenery" => Step::Scenery(arg.to_string()),
            "tab" => Step::Tab(ui::Tab::parse(arg)),
            "pick" => match nums()?.as_slice() {
                [x, z] => Step::Pick(*x, *z),
                _ => return Err(format!("{part}: pick takes x,z")),
            },
            "look" => match nums()?.as_slice() {
                [x, z, d] => Step::Look(*x, *z, *d),
                _ => return Err(format!("{part}: look takes x,z,distance")),
            },
            "scroll" => Step::Scroll(num("the pixels")?),
            "shot" if !arg.is_empty() => Step::Shot(arg.to_string()),
            "chip" if arg.eq_ignore_ascii_case("pill") || crate::overlay::is_word(arg) && !arg.contains(':') => Step::Chip(arg.to_ascii_lowercase()),
            "chip" => return Err(format!("{part}: the chips are rent, areas, reach, price, wages, plain and pill")),
            "good" => match arg.to_ascii_lowercase().as_str() {
                "next" => Step::Good(1),
                "back" => Step::Good(-1),
                _ => return Err(format!("{part}: good:next or good:back")),
            },
            "row" if !arg.is_empty() => Step::Row(arg.to_string()),
            "half" => match arg.to_ascii_lowercase().as_str() {
                "island" => Step::Half(MarketsShow::Island),
                "goods" => Step::Half(MarketsShow::Prices),
                _ => return Err(format!("{part}: half:island or half:goods")),
            },
            "map" => Step::Map(arg.to_string()),
            _ => return Err(format!("{part}: not a step (see the top of script.rs)")),
        });
    }
    if steps.is_empty() {
        return Err("the script has no steps".into());
    }
    Ok(steps)
}

/// What is wrong with a script, if anything: said with the options, before an island is made.
pub fn check(script: &str) -> Result<(), String> {
    parse(script).map(|_| ())
}

/// Add the scripted run to the app, or say what is wrong with the script.
pub fn add(app: &mut App, script: &str, size: (u32, u32)) -> Result<(), String> {
    let steps = parse(script)?;
    app.insert_resource(Script { steps, at: 0, begun: false, until: 0.0, frames: 0, keys: Vec::new(), clicked: None, tally: Tally::default(), image: Handle::default(), closing: None })
        .add_systems(PostStartup, move |mut commands: Commands, mut images: ResMut<Assets<Image>>, mut sc: ResMut<Script>, cams: Query<Entity, With<MainCam>>| {
            // with no window, the camera draws into a picture the shot step can save
            let mut img = Image::new_target_texture(size.0, size.1, TextureFormat::Rgba8UnormSrgb, None);
            img.texture_descriptor.usage |= TextureUsages::COPY_SRC;
            sc.image = images.add(img);
            for cam in &cams {
                commands.entity(cam).insert(RenderTarget::Image(sc.image.clone().into()));
            }
        })
        // after the interface has read the (absent) pointer and keyboard, so a press made here lasts the frame
        .add_systems(PreUpdate, run.after(UiSystems::Focus).after(bevy::input::InputSystems));
    Ok(())
}

fn plain(s: &str) -> String {
    s.replace('\u{00d7}', "x").trim().to_lowercase()
}

/// Every text under an entity, in order.
fn texts(world: &World, e: Entity, out: &mut Vec<String>) {
    if let Some(t) = world.get::<Text>(e) {
        out.push(t.0.clone());
    }
    if let Some(ch) = world.get::<Children>(e) {
        for c in ch.iter() {
            texts(world, c, out);
        }
    }
}

fn root_z(world: &World, mut e: Entity) -> i32 {
    while let Some(p) = world.get::<ChildOf>(e) {
        e = p.parent();
    }
    world.get::<GlobalZIndex>(e).map_or(0, |z| z.0)
}

fn shown(world: &World, e: Entity) -> bool {
    world.get::<InheritedVisibility>(e).is_none_or(|v| v.get()) && world.get::<Node>(e).is_none_or(|n| n.display != Display::None)
}

/// The layer of the topmost card that covers the whole screen and takes the
/// pointer, as the welcome and the decisions do: nothing under it can be clicked.
fn cover_z(world: &mut World) -> i32 {
    let mut q = world.query::<(Entity, &Node, &FocusPolicy, &GlobalZIndex)>();
    let zero = Val::Px(0.0);
    q.iter(world)
        .filter(|(e, n, f, _)| **f == FocusPolicy::Block && n.left == zero && n.right == zero && n.top == zero && n.bottom == zero && shown(world, *e))
        .map(|(_, _, _, z)| z.0)
        .max()
        .unwrap_or(i32::MIN)
}

/// The smallest clickable thing on screen that holds this text.
fn find_button(world: &mut World, label: &str) -> Result<Entity, String> {
    let want = plain(label);
    let mut q = world.query_filtered::<Entity, With<Interaction>>();
    let all: Vec<Entity> = q.iter(world).collect();
    let mut best: Option<(usize, Entity)> = None;
    for e in all {
        if !shown(world, e) {
            continue;
        }
        let mut t = Vec::new();
        texts(world, e, &mut t);
        if t.iter().any(|s| plain(s) == want) && best.is_none_or(|b| t.len() < b.0) {
            best = Some((t.len(), e));
        }
    }
    let Some((_, e)) = best else { return Err(format!("nothing on screen says \"{label}\"")) };
    if root_z(world, e) < cover_z(world) {
        return Err(format!("\"{label}\" is under an open card and cannot be clicked"));
    }
    Ok(e)
}

fn press(world: &mut World, sc: &mut Script, e: Entity) {
    if let Some(mut i) = world.get_mut::<Interaction>(e) {
        *i = Interaction::Pressed;
        sc.clicked = Some(e);
    }
}

fn clock_year(world: &World) -> (usize, usize) {
    let (tl, data) = (world.resource::<Timeline>(), world.resource::<Data>());
    ((tl.t.floor() as usize).min(data.r.ny - 1), data.latest())
}

fn season(world: &World) -> f32 {
    let forced = world.get_resource::<crate::SeasonMats>().and_then(|m| m.forced);
    crate::season_phase(world.resource::<crate::SeasonClock>(), forced)
}

fn date_text(world: &mut World) -> String {
    let mut q = world.query_filtered::<&Text, With<ui::DateText>>();
    q.iter(world).next().map(|t| t.0.clone()).unwrap_or_default()
}

fn stepping(world: &World) -> bool {
    world.get_resource::<Live>().is_some_and(|l| l.stepping)
}

fn print_state(world: &mut World, note: &str) {
    let date = date_text(world);
    let (k, latest) = clock_year(world);
    let (tl, data) = (world.resource::<Timeline>(), world.resource::<Data>());
    let y0 = data.r.y0;
    let clock = format!("clock {:.2} ({}), {} at {} years a second", tl.t, y0 + k as i32, if tl.playing { "playing" } else { "paused" }, tl.speed);
    let sim = format!("newest year {}{}", y0 + latest as i32, if stepping(world) { ", working out the next" } else { "" });
    let cards = world.resource::<Game>().summary();
    let fam = world.get_resource::<FamilyView>().and_then(|f| f.player()).map(|p| format!("cash {}, {} works, {} squares", ui::money(p.cash), p.works, p.land)).unwrap_or_default();
    let ph = season(world);
    println!("script: {}{}; {}; date \"{}\"; season {:.3}", if note.is_empty() { String::new() } else { format!("[{note}] ") }, clock, sim, date, ph);
    println!("script:   {cards}; {fam}");
    println!("script:   {}", crate::scenery_line(world));
    // the order panel under the inspector: what it says and what can be pressed
    let mut q = world.query_filtered::<Entity, With<InspectActions>>();
    let panel: Vec<Entity> = q.iter(world).collect();
    let mut said = Vec::new();
    for e in panel {
        texts(world, e, &mut said);
    }
    let mut q = world.query::<(Entity, &ActionButton)>();
    let buttons: Vec<Entity> = q.iter(world).map(|(e, _)| e).collect();
    let mut labels = Vec::new();
    for e in buttons {
        texts(world, e, &mut labels);
    }
    if !said.is_empty() || world.resource::<Picked>().what.is_some() {
        let lines: Vec<String> = said.iter().take(8).map(|s| s.chars().take(70).collect()).collect();
        println!("script:   order panel says: {}", if lines.is_empty() { "nothing".to_string() } else { lines.join(" | ") });
        println!("script:   order buttons: {}", if labels.is_empty() { "none".to_string() } else { labels.join(", ") });
    }
    // the side panel: how many rows, how far scrolled, and its last few lines
    let mut q = world.query_filtered::<(Entity, &ScrollPosition, &ComputedNode), With<ui::PanelBody>>();
    let body: Vec<(Entity, f32, f32)> = q.iter(world).map(|(e, s, c)| (e, s.0.y, ((c.content_size().y - c.size().y) * c.inverse_scale_factor).max(0.0))).collect();
    if let (Some(tab), Some(&(e, y, max))) = (world.resource::<UiState>().tab, body.first()) {
        let rows = world.get::<Children>(e).map_or(0, |c| c.len());
        let mut t = Vec::new();
        texts(world, e, &mut t);
        let tail: Vec<String> = t.iter().rev().take(3).rev().map(|s| s.chars().take(60).collect()).collect();
        println!("script:   {tab:?} panel: {rows} rows, scrolled {y:.0} of {max:.0}; first line \"{}\"; ends \"{}\"", t.first().cloned().unwrap_or_default(), tail.join(" | "));
    }
}

/// What the map and the Markets panel show, in one line.
fn print_map(world: &World, note: &str) {
    let (ov, ui, data) = (world.resource::<MapOverlay>(), world.resource::<UiState>(), world.resource::<Data>());
    let good = |i: usize| data.r.header.goods.get(i).map_or("?", |g| g.name.as_str());
    let map = ov.kind.map_or("plain".to_string(), |k| format!("{k:?}").to_lowercase());
    let half = if ui.markets == MarketsShow::Island { "island" } else { "goods" };
    let panel = ui.tab.map_or("none".to_string(), |t| format!("{t:?}").to_lowercase());
    println!(
        "script: {}map: {map:<6} price of: {:<9} panel: {panel:<8} markets half: {half:<6} open good: {}",
        if note.is_empty() { String::new() } else { format!("[{note}] ") },
        good(ov.good),
        ui.good.map_or("none", good)
    );
}

/// The thing of one kind that the pointer could reach, chosen by `want`: on
/// screen, and not under an open card.
fn find_marked<C: Component>(world: &mut World, what: &str, want: impl Fn(&C) -> bool) -> Result<Entity, String> {
    let mut q = world.query::<(Entity, &C)>();
    let all: Vec<Entity> = q.iter(world).filter(|(_, c)| want(c)).map(|(e, _)| e).collect();
    let Some(e) = all.into_iter().find(|&e| shown(world, e)) else { return Err(format!("{what}: nothing to press")) };
    if root_z(world, e) < cover_z(world) {
        return Err(format!("{what} is under an open card and cannot be clicked"));
    }
    Ok(e)
}

fn sample(world: &mut World, t: &mut Tally) {
    let (k, latest) = clock_year(world);
    if t.frames == 0 {
        t.k0 = k;
    }
    t.frames += 1;
    let step = stepping(world);
    if world.resource::<Timeline>().playing {
        t.playing += 1;
        t.stepping += step as u32;
        t.can_act += (k >= latest && !step) as u32;
    }
    let ph = season(world);
    if let Some(last) = t.last_phase {
        // the short way round
        let d = (ph - last + 0.5).rem_euclid(1.0) - 0.5;
        if d >= 0.0 { t.season_fwd += d } else { t.season_back -= d }
        t.season_jump = t.season_jump.max(d.abs());
    }
    t.last_phase = Some(ph);
    let date = date_text(world);
    if t.dates.last() != Some(&date) {
        t.dates.push(date);
    }
}

/// For a `run` step: give an open decision its default, and start the clock
/// again once its card has gone. True if something was pressed this frame.
fn keep_playing(world: &mut World, sc: &mut Script) -> bool {
    let game = world.resource::<Game>();
    let (open, busy) = (game.open, game.busy);
    if let Some(id) = open {
        if busy {
            return false;
        }
        let default = world.get_resource::<FamilyView>().and_then(|f| f.player()).and_then(|p| p.rep.pending.iter().find(|d| d.id == id)).map_or(0, |d| d.default);
        let mut q = world.query::<(Entity, &DecideButton)>();
        let hit = q.iter(world).find(|(_, d)| d.id == id && d.option == default).map(|(e, _)| e);
        if let Some(e) = hit {
            press(world, sc, e);
            sc.tally.answered += 1;
            sc.tally.resume = true;
            return true;
        }
        return false;
    }
    if sc.tally.resume && !busy {
        sc.tally.resume = false;
        if !world.resource::<Timeline>().playing {
            world.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
            sc.keys.push(KeyCode::Space);
            return true;
        }
    }
    false
}

fn print_tally(world: &World, t: &Tally, secs: f32) {
    let (k, _) = clock_year(world);
    let pc = |n: u32| 100.0 * n as f32 / t.playing.max(1) as f32;
    println!(
        "script: sampled {secs} s, {} frames, {} years, {} decisions answered; the clock ran in {} frames: in {:.0}% of them an order would land in the year on screen, in {:.0}% the next year was being worked out",
        t.frames,
        k.saturating_sub(t.k0),
        t.answered,
        t.playing,
        pc(t.can_act),
        pc(t.stepping)
    );
    println!(
        "script:   the season went {:.2} of a year forwards and {:.2} back, {:.3} at most between two frames; the date read {} different things: {}",
        t.season_fwd,
        t.season_back,
        t.season_jump,
        t.dates.len(),
        t.dates.iter().take(6).cloned().collect::<Vec<_>>().join(", ") + if t.dates.len() > 6 { ", \u{2026}" } else { "" }
    );
}

fn fail(why: &str) -> ! {
    eprintln!("script: stopped: {why}");
    std::process::exit(1);
}

fn run(world: &mut World) {
    let Some(mut sc) = world.remove_resource::<Script>() else { return };
    step(world, &mut sc);
    world.insert_resource(sc);
}

fn step(world: &mut World, sc: &mut Script) {
    // let go of what the last frame pressed
    if !sc.keys.is_empty() {
        let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
        for k in sc.keys.drain(..) {
            keys.release(k);
        }
        return;
    }
    if let Some(e) = sc.clicked.take() {
        if let Some(mut i) = world.get_mut::<Interaction>(e) {
            *i = Interaction::None;
        }
        return;
    }
    if let Some(left) = sc.closing {
        if left == 0 {
            world.write_message(AppExit::Success);
        }
        sc.closing = Some(left.saturating_sub(1));
        return;
    }
    let now = world.resource::<Time>().elapsed_secs();
    loop {
        let Some(st) = sc.steps.get(sc.at).cloned() else {
            // a few frames for the last picture to reach the disk
            sc.closing = Some(8);
            return;
        };
        // a step that takes time comes back here every frame until it is done
        let mut done = true;
        let mut frame = false;
        match st {
            Step::Key(k, shift) => {
                let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
                if shift {
                    keys.press(KeyCode::ShiftLeft);
                    sc.keys.push(KeyCode::ShiftLeft);
                }
                keys.press(k);
                sc.keys.push(k);
                frame = true;
            }
            Step::Click(label) => {
                match find_button(world, &label) {
                    Ok(e) => press(world, sc, e),
                    Err(why) => println!("script: no click: {why}"),
                }
                frame = true;
            }
            Step::Decide(n) => {
                let open = world.resource::<Game>().open;
                let mut q = world.query::<(Entity, &DecideButton)>();
                let hit = q.iter(world).find(|(_, d)| Some(d.id) == open && d.option == n).map(|(e, _)| e);
                match hit {
                    Some(e) => press(world, sc, e),
                    // none open, or its options are unlit while the year turns
                    None => println!("script: no click: no option {n} of an open decision can be clicked"),
                }
                frame = true;
            }
            Step::Wait(s) => {
                if !sc.begun {
                    sc.begun = true;
                    sc.until = now + s;
                }
                done = now >= sc.until;
            }
            Step::Frames(n) => {
                if !sc.begun {
                    sc.begun = true;
                    sc.frames = n;
                }
                done = sc.frames == 0;
                sc.frames = sc.frames.saturating_sub(1);
            }
            Step::Settle => {
                if !sc.begun {
                    sc.begun = true;
                    sc.until = now + 120.0;
                }
                let quiet = !stepping(world) && !world.resource::<Game>().busy && crate::scenery_ready(world.resource::<Timeline>(), world.resource::<Data>());
                if !quiet && now >= sc.until {
                    fail("the game did not settle in two minutes");
                }
                done = quiet;
            }
            Step::Sample(s, keep_going) => {
                if !sc.begun {
                    sc.begun = true;
                    sc.until = now + s;
                    sc.tally = Tally::default();
                }
                if keep_going && keep_playing(world, sc) {
                    return;
                }
                sample(world, &mut sc.tally);
                done = now >= sc.until;
                if done {
                    print_tally(world, &sc.tally, s);
                }
            }
            Step::Until(part) => {
                if !sc.begun {
                    sc.begun = true;
                    sc.until = now + 60.0;
                }
                // a decision that opens is given its default, as `run` does, to keep the years going
                if keep_playing(world, sc) {
                    return;
                }
                let tl = world.resource::<Timeline>();
                done = tl.playing && tl.t.fract() >= part;
                if !done && now >= sc.until {
                    fail(&format!("until:{part}: the clock is {} and did not come to that part of a year in a minute", if tl.playing { "running" } else { "stopped" }));
                }
            }
            Step::Print(note) => print_state(world, &note),
            Step::Scenery(note) => {
                let note = if note.is_empty() { String::new() } else { format!("[{note}] ") };
                match crate::check_scenery(world) {
                    Ok(said) => println!("script: {note}scenery: {said}"),
                    Err(why) => fail(&format!("{note}scenery: {why}")),
                }
            }
            Step::Tab(t) => {
                world.resource_mut::<UiState>().tab = t;
                frame = true;
            }
            Step::Pick(x, z) => {
                let (k, _) = clock_year(world);
                let data = world.resource::<Data>();
                let what = crate::choose(&data.r, &data.slots, &data.cen, Vec3::new(x, world::height_at(&data.g, x, z), z), k);
                world.resource_mut::<Picked>().what = what;
                frame = true;
            }
            Step::Look(x, z, d) => {
                let y = world::height_at(&world.resource::<Data>().g, x, z).max(0.0);
                let mut o = world.resource_mut::<Orbit>();
                o.target = Vec3::new(x, y, z);
                o.dist = d;
                frame = true;
            }
            Step::Scroll(px) => {
                // as the wheel would, a positive turn of the wheel being up
                let mut q = world.query_filtered::<(&mut ScrollPosition, &ComputedNode), With<ui::PanelBody>>();
                for (mut s, c) in q.iter_mut(world) {
                    ui::scroll_panel(&mut s, c, -px);
                }
                frame = true;
            }
            Step::Chip(name) => {
                // Plain map and the pill are the same chip: the one that asks for no overlay
                let want: Option<Overlay> = crate::overlay::parse(&name, &world.resource::<Data>().r).and_then(|p| p.0);
                match find_marked::<OverlayChip>(world, &format!("chip:{name}"), |c| c.0 == want) {
                    Ok(e) => press(world, sc, e),
                    Err(why) => println!("script: no click: {why}"),
                }
                frame = true;
            }
            Step::Good(turn) => {
                match find_marked::<GoodStep>(world, if turn < 0 { "good:back" } else { "good:next" }, |s| s.0 == turn) {
                    Ok(e) => press(world, sc, e),
                    Err(why) => println!("script: no click: {why}"),
                }
                frame = true;
            }
            Step::Row(name) => {
                let want = world.resource::<Data>().r.header.goods.iter().position(|g| g.name.eq_ignore_ascii_case(&name) || g.label.eq_ignore_ascii_case(&name));
                match find_marked::<GoodRow>(world, &format!("row:{name}"), |g| Some(g.0) == want) {
                    Ok(e) => press(world, sc, e),
                    Err(why) => println!("script: no click: {why}"),
                }
                frame = true;
            }
            Step::Half(half) => {
                match find_marked::<MarketsChip>(world, if half == MarketsShow::Island { "half:island" } else { "half:goods" }, |c| c.0 == half) {
                    Ok(e) => press(world, sc, e),
                    Err(why) => println!("script: no click: {why}"),
                }
                frame = true;
            }
            Step::Map(note) => {
                if !sc.begun {
                    sc.begun = true;
                    sc.frames = CATCH_UP;
                }
                done = sc.frames == 0;
                sc.frames = sc.frames.saturating_sub(1);
                if done {
                    print_map(world, &note);
                    // whether the trees stand aside for a map view, as they are set this frame
                    let (shown, all) = crate::trees_showing(world);
                    println!("script:   trees: {shown} of {all} meshes showing");
                }
            }
            Step::Shot(path) => {
                if !sc.begun {
                    sc.begun = true;
                    sc.frames = 8;
                    // a picture whose directory is not there would be made and then not written
                    if let Some(parent) = std::path::Path::new(&path).parent().filter(|p| !p.as_os_str().is_empty()) {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            fail(&format!("cannot make {}: {e}", parent.display()));
                        }
                    }
                    world.spawn(Screenshot::image(sc.image.clone())).observe(save_to_disk(path));
                }
                done = sc.frames == 0;
                sc.frames = sc.frames.saturating_sub(1);
            }
        }
        if !done {
            return;
        }
        sc.begun = false;
        sc.at += 1;
        if frame {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_steps_of_both_kinds_are_read() {
        let steps = parse("key:Space; wait:2;click:Next year;decide:1;frames:3;settle;sample:5;run:10;print:a note;tab:markets;pick:31.5,27.5;look:60,46,130;scroll:36;shot:out/a.png").unwrap();
        assert_eq!(steps.len(), 14);
        assert_eq!(steps[0], Step::Key(KeyCode::Space, false));
        assert_eq!(steps[2], Step::Click("Next year".into()));
        assert_eq!((&steps[6], &steps[7]), (&Step::Sample(5.0, false), &Step::Sample(10.0, true)));
        assert_eq!(steps[10], Step::Pick(31.5, 27.5));
        // the map views and the Markets panel
        let steps = parse("key:M;key:Shift+M;key:shift+g;chip:rent;chip:Plain;chip:pill;good:next;good:back;row:coal;half:island;half:goods;map;map:after").unwrap();
        assert_eq!(steps[..3], [Step::Key(KeyCode::KeyM, false), Step::Key(KeyCode::KeyM, true), Step::Key(KeyCode::KeyG, true)]);
        assert_eq!(steps[3..6], [Step::Chip("rent".into()), Step::Chip("plain".into()), Step::Chip("pill".into())]);
        assert_eq!(steps[6..9], [Step::Good(1), Step::Good(-1), Step::Row("coal".into())]);
        assert_eq!(steps[9..], [Step::Half(MarketsShow::Island), Step::Half(MarketsShow::Prices), Step::Map(String::new()), Step::Map("after".into())]);
    }

    #[test]
    fn a_step_that_is_not_one_is_refused() {
        assert_eq!(parse("").err().unwrap(), "the script has no steps");
        assert_eq!(parse("bogus").err().unwrap(), "bogus: not a step (see the top of script.rs)");
        assert_eq!(parse("key:Nope").err().unwrap(), "key:Nope: no key called Nope");
        assert_eq!(parse("key:Shift+Nope").err().unwrap(), "key:Shift+Nope: no key called Nope");
        assert_eq!(parse("wait:long").err().unwrap(), "wait:long: the seconds must be a number");
        assert_eq!(parse("chip:rents").err().unwrap(), "chip:rents: the chips are rent, areas, reach, price, wages, plain and pill");
        assert_eq!(parse("chip:price:coal").err().unwrap(), "chip:price:coal: the chips are rent, areas, reach, price, wages, plain and pill");
        assert_eq!(parse("good:up").err().unwrap(), "good:up: good:next or good:back");
        assert_eq!(parse("half:all").err().unwrap(), "half:all: half:island or half:goods");
        // steps are parted by semicolons: commas belong to pick and look
        assert!(parse("key:M,key:M").is_err());
    }
}
