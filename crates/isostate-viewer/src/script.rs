//! A scripted run, for checking what a still cannot show: keys, clicks and
//! timing. `--script "key:Space;wait:2;print"` runs the game with no window,
//! plays the steps in order as a player's presses and clicks, prints what the
//! clock, the cards and the order panel did, and stops.
//!
//! Steps, separated by semicolons:
//!
//!     key:NAME        press a key for one frame (Space, Left, Right, Up, Down,
//!                     PageUp, PageDown, Escape, a letter or a digit)
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
//!     print[:NOTE]    print the clock, the cards and the order panel
//!     tab:NAME        open a side panel (family, holdings, markets, news, none)
//!     pick:X,Z        click the map at this spot
//!     look:X,Z,DIST   point the camera
//!     scroll:PIXELS   scroll the side panel down this far
//!     shot:PATH       save a picture
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
use crate::ui::{self, UiState};
use crate::{world, Data, InspectActions, MainCam, Orbit, Picked, Timeline};

#[derive(Clone, Debug, PartialEq)]
enum Step {
    Key(KeyCode),
    Click(String),
    Decide(usize),
    Wait(f32),
    Frames(u32),
    Settle,
    /// seconds, and whether to answer decisions and keep the clock going
    Sample(f32, bool),
    Print(String),
    Tab(Option<ui::Tab>),
    Pick(f32, f32),
    Look(f32, f32, f32),
    Scroll(f32),
    Shot(String),
}

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
    key: Option<KeyCode>,
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
        "h" => KeyCode::KeyH,
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
            "key" => Step::Key(key_code(arg).ok_or_else(|| format!("{part}: no key called {arg}"))?),
            "click" if !arg.is_empty() => Step::Click(arg.to_string()),
            "decide" => Step::Decide(num("the option")? as usize),
            "wait" => Step::Wait(num("the seconds")?),
            "frames" => Step::Frames(num("the frames")? as u32),
            "settle" => Step::Settle,
            "sample" => Step::Sample(num("the seconds")?, false),
            "run" => Step::Sample(num("the seconds")?, true),
            "print" => Step::Print(arg.to_string()),
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
            _ => return Err(format!("{part}: not a step (see the top of script.rs)")),
        });
    }
    if steps.is_empty() {
        return Err("the script has no steps".into());
    }
    Ok(steps)
}

/// Add the scripted run to the app, or say what is wrong with the script.
pub fn add(app: &mut App, script: &str, size: (u32, u32)) -> Result<(), String> {
    let steps = parse(script)?;
    app.insert_resource(Script { steps, at: 0, begun: false, until: 0.0, frames: 0, key: None, clicked: None, tally: Tally::default(), image: Handle::default(), closing: None })
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
    crate::season_phase(world.resource::<Timeline>(), forced)
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
            sc.key = Some(KeyCode::Space);
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
    if let Some(k) = sc.key.take() {
        world.resource_mut::<ButtonInput<KeyCode>>().release(k);
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
            Step::Key(k) => {
                world.resource_mut::<ButtonInput<KeyCode>>().press(k);
                sc.key = Some(k);
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
                    None => fail(&format!("no option {n} of an open decision to click")),
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
            Step::Print(note) => print_state(world, &note),
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
                let mut q = world.query_filtered::<&mut ScrollPosition, With<ui::PanelBody>>();
                for mut s in q.iter_mut(world) {
                    s.0.y += px;
                }
                frame = true;
            }
            Step::Shot(path) => {
                if !sc.begun {
                    sc.begun = true;
                    sc.frames = 8;
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
