//! A scripted run, for checking what a still cannot show: keys, clicks and
//! timing. `--script "key:Space;wait:2;print"` runs the game with no window,
//! plays the steps in order as a player's presses and clicks, prints what the
//! clock, the cards and the order panel did, and stops. It takes its own
//! pictures (`shot:PATH`). Nobody is at a scripted game, so it is not kept and
//! writes over no saved game: `--save FILE` keeps it in a file, and `--keep`
//! in the island's autosave, which is for checks of the saving itself and
//! wants `ISOSTATE_SAVES` to name the folder. Where `ISOSTATE_SAVES` names a
//! folder a script goes on with the autosave it finds there, as a player's
//! start would, and shows the card for it.
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
//!     settle          wait until nothing is being worked out, no order is out
//!                     and the scenery shows the year on the clock
//!     sample:SECONDS  let the game run and count, frame by frame, where the
//!                     clock stood, how far the season moved and how long the
//!                     frames' work took (those a month or a year came in apart)
//!     run:SECONDS     the same, keeping the years going: a decision that opens
//!                     is given its default and the clock is started again
//!     until:PART      let the game run until the clock is this far through a
//!                     year (until:0.95 is just before the year turns)
//!     goto:YEAR       the clock to January of YEAR: at or behind the present
//!                     as dragging the knob there does; ahead of it, in a
//!                     sandbox, a run on to that January (settle waits for
//!                     it), and in the family game nothing, with a line saying why
//!     print[:NOTE]    print the clock and its date, the cards (the opening card
//!                     in its own words), the orders that wait for the year's
//!                     end, the order panel, what scenery is on screen and
//!                     what putting it there cost, the side panel's title and
//!                     text, every text of the note about what was clicked,
//!                     and in a sandbox how wide the tray is laid out
//!     scenery[:NOTE]  build the year on the clock afresh and say whether what
//!                     is on screen is that; stops the run if it is not
//!     tab:NAME        open a side panel (family, holdings, markets, news, none;
//!                     a sandbox's hand and rules, and families for its family tab)
//!     pick:X,Z        click the map at this spot
//!     look:X,Z,DIST   point the camera
//!     scroll:PIXELS   scroll the side panel down this far
//!     card:PIXELS     scroll the card of what was clicked down this far (a
//!                     town's ideas stand under its rows)
//!     shot:PATH       save a picture
//!     rule:NAME=WORD  in a sandbox, press the chip WORD on the Rules tab's row
//!                     NAME (rule:money.growth=None), opening the tab and the
//!                     rule first if it must: many rows share None and Double
//!
//! and for the map views and the Markets panel:
//!
//!     chip:NAME       click a chip of the legend card: rent, areas, reach,
//!                     price, made, wages, growth, residents, deposits, plain,
//!                     or pill for the Map views button that stands there
//!                     while the map is plain
//!     good:next       the legend's steps through the goods whose price or
//!     good:back       making is mapped, or the kinds of ground
//!     range:year      the legend's chip for how the colours' ends are taken:
//!     range:30        this year, thirty years gliding, the whole run
//!     range:all
//!     row:GOOD        click a good's row in the Markets panel (row:coal)
//!     half:island     click a chip of the Markets panel: the island's charts,
//!     half:goods      the goods, or the island's work and owners (print on
//!     half:work       that half gives its figures, a line for each fifty
//!                     years and one for the year on the clock)
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
    /// the clock to January of this year
    Goto(i32),
    Print(String),
    Scenery(String),
    Tab(Option<ui::Tab>),
    Pick(f32, f32),
    Look(f32, f32, f32),
    Scroll(f32),
    /// scroll the card of what was clicked down this far
    Card(f32),
    Shot(String),
    /// a chip of the legend card: an overlay, or none for Plain map and the pill
    Chip(String),
    /// the legend's step through the goods: 1 on, -1 back
    Good(i32),
    /// the legend's chip for how the colours' ends are taken
    Range(crate::ranges::Range),
    /// a good's row in the Markets panel, by name
    Row(String),
    Half(MarketsShow),
    Map(String),
    /// the Rules tab's chip: a name and the word on its chip
    Rule(String, String),
}

/// Frames a `map` step lets pass, so that a panel rebuilt after a press is there to be read.
const CATCH_UP: u32 = 8;

/// What a `sample` step counted.
#[derive(Default)]
struct Tally {
    frames: u32,
    /// frames with the clock running
    playing: u32,
    /// of those, frames in which the simulation stood at a year's turn, so an
    /// order given then would be carried out at once
    can_act: u32,
    /// frames with a month or a year's end being worked out
    stepping: u32,
    /// of those, frames in which the clock stood still waiting for it
    waited: u32,
    last_t: f32,
    /// where the clock stood when the counting began
    t0: f32,
    /// how far round the year the season went, forwards and backwards
    season_fwd: f32,
    season_back: f32,
    last_phase: Option<f32>,
    /// the biggest move of the season between two frames
    season_jump: f32,
    dates: Vec<String>,
    k0: usize,
    /// how long the main world's work took in each frame, in milliseconds, with
    /// what came in during it: a year from the simulation, a year's scenery, a month
    work: Vec<(f32, bool, bool, bool, bool)>,
    /// how long the extract after each of those frames took, in milliseconds: the main
    /// world held still while the renderer takes what changed
    extract: Vec<f32>,
    /// the years worked out, the scenery's year and the months that had come
    /// when the last frame was counted
    last_filled: usize,
    last_built: Option<usize>,
    last_months: u64,
    /// the step of the season the last frame drew (the ground, the trees and the sun are touched when it changes)
    last_season: i32,
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
        "5" => KeyCode::Digit5,
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
            "goto" => Step::Goto(arg.trim().parse::<i32>().map_err(|_| format!("{part}: goto takes a year, as goto:1800"))?),
            "print" => Step::Print(arg.to_string()),
            "scenery" => Step::Scenery(arg.to_string()),
            // a panel that is none of these is refused, and not taken for closing the panel
            "tab" if ui::Tab::parse(arg).is_some() || arg.eq_ignore_ascii_case("none") => Step::Tab(ui::Tab::parse(arg)),
            "tab" => return Err(format!("{part}: the panels are family, holdings, markets, news, hand, rules, families and none")),
            "pick" => match nums()?.as_slice() {
                [x, z] => Step::Pick(*x, *z),
                _ => return Err(format!("{part}: pick takes x,z")),
            },
            "look" => match nums()?.as_slice() {
                [x, z, d] => Step::Look(*x, *z, *d),
                _ => return Err(format!("{part}: look takes x,z,distance")),
            },
            "scroll" => Step::Scroll(num("the pixels")?),
            "card" => Step::Card(num("the pixels")?),
            "shot" if !arg.is_empty() => Step::Shot(arg.to_string()),
            "chip" if arg.eq_ignore_ascii_case("pill") || crate::overlay::is_word(arg) && !arg.contains(':') && !arg.contains('+') => Step::Chip(arg.to_ascii_lowercase()),
            "chip" => return Err(format!("{part}: the chips are rent, areas, reach, price, made, wages, growth, residents, deposits, traffic, plain and pill")),
            "range" => Step::Range(crate::ranges::Range::parse(arg).ok_or_else(|| format!("{part}: range:year, range:30 or range:all"))?),
            "good" => match arg.to_ascii_lowercase().as_str() {
                "next" => Step::Good(1),
                "back" => Step::Good(-1),
                _ => return Err(format!("{part}: good:next or good:back")),
            },
            "row" if !arg.is_empty() => Step::Row(arg.to_string()),
            "half" => match arg.to_ascii_lowercase().as_str() {
                "island" => Step::Half(MarketsShow::Island),
                "goods" => Step::Half(MarketsShow::Prices),
                "work" => Step::Half(MarketsShow::Work),
                _ => return Err(format!("{part}: half:island, half:goods or half:work")),
            },
            "map" => Step::Map(arg.to_string()),
            "rule" => match arg.split_once('=').map(|(n, w)| (n.trim(), w.trim())) {
                Some((name, word)) if !name.is_empty() && !word.is_empty() => Step::Rule(name.to_string(), word.to_string()),
                _ => return Err(format!("{part}: rule takes a name and a chip's word, as rule:money.growth=None")),
            },
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
    // a script that checks what is on screen has each year's build take its meshes' fingerprints
    if steps.iter().any(|s| matches!(s, Step::Scenery(_))) {
        crate::PRINT_YEAR_MESHES.store(true, std::sync::atomic::Ordering::Relaxed);
    }
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
    // (what the frames cost is the frame recorder's to say, which a scripted run keeps on
    // as a meter: frames.rs, `last_work`)
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
    world.get_resource::<Live>().is_some_and(|l| l.stepping())
}

/// The line under the date.
fn note_text(world: &mut World) -> String {
    let mut q = world.query_filtered::<&Text, With<ui::StepNote>>();
    q.iter(world).next().map(|t| t.0.clone()).unwrap_or_default()
}

fn print_state(world: &mut World, note: &str) {
    let date = date_text(world);
    let under = note_text(world);
    let (k, latest) = clock_year(world);
    let (tl, data) = (world.resource::<Timeline>(), world.resource::<Data>());
    let y0 = data.r.y0;
    let d = crate::clock::date(y0, tl.t);
    let speed = crate::clock::speed_at(tl.speed).map_or(format!("{} years a second", tl.speed), |s| format!("{} ({})", s.name, s.pace));
    let clock = format!("clock {:.3} ({} {}, {:.0}% through; the map is of {}), {} at {}", tl.t, crate::clock::MONTHS[d.month], d.year, d.part * 100.0, y0 + k as i32, if tl.playing { "playing" } else { "paused" }, speed);
    let live = world.get_resource::<Live>();
    let sim = format!(
        "newest year {}, {} of the next year's months worked out{}",
        y0 + latest as i32,
        live.map_or(0, |l| l.months),
        match live {
            Some(l) if l.turning() => ", the year's end on its way",
            Some(l) if l.stepping() => ", a month on its way",
            _ => "",
        }
    );
    let cards = world.resource::<Game>().summary();
    let fam = world.get_resource::<FamilyView>().and_then(|f| f.player()).map(|p| format!("cash {}, {} works, {} squares", ui::money(p.cash), p.works, p.land)).unwrap_or_default();
    let ph = season(world);
    println!("script: {}{}; {}; date \"{}\"; season {:.3}", if note.is_empty() { String::new() } else { format!("[{note}] ") }, clock, sim, date, ph);
    println!("script:   {cards}; {fam}");
    // what waits for the year's end, as the Family panel lists it
    if let Some(l) = live {
        let rep = world.get_resource::<FamilyView>().and_then(|f| f.player()).map(|p| &p.rep);
        let lines: Vec<String> = l.standing.waiting.iter().map(|o| crate::live::order_words(o, &data.r, rep)).collect();
        println!(
            "script:   under the date: \"{under}\"; waiting for the year's end: {}; {} promised; pegs on {} squares",
            if lines.is_empty() { "nothing".to_string() } else { lines.join(" | ") },
            ui::money(l.standing.promised as f32),
            l.standing.pegs.len()
        );
    }
    println!("script:   {}", crate::scenery_line(world));
    // the opening card in its own words: what a game gone on with is told of its save
    let mut q = world.query_filtered::<Entity, With<crate::game::Welcome>>();
    let card: Vec<Entity> = q.iter(world).collect();
    let mut says = Vec::new();
    for e in card {
        texts(world, e, &mut says);
    }
    if !says.is_empty() {
        println!("script:   the welcome card says: {}", says.join(" | "));
    }
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
        // and what it says, so that a log can be read where a picture would otherwise be needed
        let mut q = world.query_filtered::<&Text, With<ui::PanelTitle>>();
        let title = q.iter(world).next().map(|t| t.0.clone()).unwrap_or_default();
        println!("script:   the panel says: {title}: {}", said_in_one_line(&t, PANEL_SAID));
        // the Work half's figures, from the very vectors its charts draw
        if tab == ui::Tab::Markets && world.resource::<UiState>().markets == MarketsShow::Work {
            for line in crate::work_charts::said(&world.resource::<Data>().r, k) {
                println!("script:   {line}");
            }
            // and the reckoning of pay by class, at the dials the rules last told
            let dials = crate::work_charts::Dials::of(world.get_resource::<crate::rulebook::Known>());
            for line in crate::work_charts::said_pay(&world.resource::<Data>().r, k, dials) {
                println!("script:   {line}");
            }
        }
        // the island half's people by age, reckoned (docs/rules/ages.md), as its chart draws them
        if tab == ui::Tab::Markets && world.resource::<UiState>().markets == MarketsShow::Island {
            if let Some(line) = crate::ages_chart::said(&world.resource::<Data>().r, k) {
                println!("script:   {line}");
            }
            // and who works by age (docs/rules/who-works.md)
            if let Some(line) = crate::ages_chart::said_work(&world.resource::<Data>().r, k) {
                println!("script:   {line}");
            }
            // and the state's purse (docs/rules/state.md), as its chart draws it
            if let Some(line) = crate::purse_chart::said(&world.resource::<Data>().r, k) {
                println!("script:   {line}");
            }
        }
    }
    // the note about what was clicked, every text of it, while it shows
    let mut q = world.query_filtered::<Entity, With<crate::InspectPanel>>();
    let cards: Vec<Entity> = q.iter(world).filter(|&e| world.get::<InheritedVisibility>(e).is_some_and(|v| v.get())).collect();
    let mut note = Vec::new();
    for e in cards {
        texts(world, e, &mut note);
    }
    if note.iter().any(|s| !s.trim().is_empty()) {
        println!("script:   the inspector says: {}", said_in_one_line(&note, usize::MAX));
    }
    // a sandbox's tray as it is laid out, and the hand's ticks on the timeline
    if world.contains_resource::<crate::sandbox::Sandbox>() {
        if let Some(line) = crate::sandbox::tray_measure(world) {
            println!("script:   {line}");
        }
        let ticks = crate::ui::hand_ticks(world);
        println!("script:   the timeline has {ticks} tick{} of the hand", if ticks == 1 { "" } else { "s" });
        if world.resource::<UiState>().tab == Some(ui::Tab::Rules) {
            if let Some(line) = crate::rulebook::said(world) {
                println!("script:   {line}");
            }
        }
        // the island's money from the record, each December of the last dozen years and the
        // newest month there is, so that a rule of money set by hand can be read in a log
        let r = &world.resource::<Data>().r;
        let newest = (0..r.series_len).rev().find(|&t| crate::replay::month(r, "money", t).is_some());
        if let Some(t) = newest {
            let decembers: Vec<String> = (t.saturating_sub(12 * 12)..=t)
                .filter(|m| m % 12 == 11 || *m == t)
                .filter_map(|m| crate::replay::month(r, "money", m).map(|v| format!("{} {} {:.6e}", &crate::clock::MONTHS[m % 12][..3], r.y0 + (m / 12) as i32, v)))
                .collect();
            println!("script:   the island's money (the record's series): {}", decembers.join(", "));
        }
    }
}

/// How much of the side panel's text `print` gives, in characters.
const PANEL_SAID: usize = 600;

/// Texts joined by " | " on one line (a text's own line breaks become " / "),
/// the empty ones left out, cut at `most` characters with an ellipsis.
fn said_in_one_line(texts: &[String], most: usize) -> String {
    let joined = texts.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| s.replace('\n', " / ")).collect::<Vec<_>>().join(" | ");
    if joined.chars().count() > most {
        format!("{}\u{2026}", joined.chars().take(most).collect::<String>())
    } else {
        joined
    }
}

/// The `rule:NAME=WORD` step: the chip to press, None while the tab, the rules
/// or the rule's numbers are still to come (the tab and the rule are opened
/// here when they must be), or why nothing will be pressed.
fn rule_chip(world: &mut World, name: &str, word: &str) -> Result<Option<Entity>, String> {
    if !world.contains_resource::<crate::sandbox::Sandbox>() {
        return Err("the Rules tab is a sandbox's".to_string());
    }
    if world.resource::<UiState>().tab != Some(ui::Tab::Rules) {
        world.resource_mut::<UiState>().tab = Some(ui::Tab::Rules);
        return Ok(None);
    }
    let known = world.resource::<crate::rulebook::Known>();
    if known.entries().is_empty() {
        return Ok(None);
    }
    let Some(e) = known.entry(name) else { return Err(format!("there is no rule called {name}")) };
    let Some(o) = e.offers.iter().find(|o| o.word.eq_ignore_ascii_case(word)) else {
        let words: Vec<&str> = e.offers.iter().map(|o| o.word.as_str()).collect();
        return Err(if words.is_empty() { format!("{name} offers nothing{}", e.fixed.as_ref().map_or(String::new(), |f| format!(": {f}"))) } else { format!("{name} offers {}", words.join(", ")) });
    };
    if crate::rulebook::same(&o.value, &e.value) {
        return Err(format!("{name} is at {} already", o.word));
    }
    if let Some(why) = &o.refused {
        return Err(format!("not drawn, since an edit would be refused: {why}"));
    }
    let (rule, open) = (e.rule.clone(), known.open().map(str::to_string));
    match find_marked::<crate::rulebook::RuleChip>(world, &format!("rule:{name}={word}"), |c| c.name == name && c.word.eq_ignore_ascii_case(word)) {
        Ok(chip) => Ok(Some(chip)),
        Err(_) => {
            // its rule is not open: open it, and the chip comes when the tab is drawn again
            if open.as_deref() != Some(rule.as_str()) {
                world.resource_mut::<crate::rulebook::Known>().open_rule(&rule);
                world.resource_mut::<UiState>().dirty += 1;
            }
            Ok(None)
        }
    }
}

/// What the map and the Markets panel show, in one line.
fn print_map(world: &World, note: &str) {
    let (ov, ui, data) = (world.resource::<MapOverlay>(), world.resource::<UiState>(), world.resource::<Data>());
    let good = |i: usize| data.r.header.goods.get(i).map_or("?", |g| g.name.as_str());
    let map = ov.kind.map_or("plain".to_string(), |k| format!("{k:?}").to_lowercase());
    let half = match ui.markets {
        MarketsShow::Island => "island",
        MarketsShow::Prices => "goods",
        MarketsShow::Work => "work",
    };
    let panel = ui.tab.map_or("none".to_string(), |t| format!("{t:?}").to_lowercase());
    // a view that colours by a number adds its range, its ends and its round figures
    let k = (world.resource::<Timeline>().t.floor() as usize).min(data.r.ny - 1);
    let range = if ov.kind.is_some_and(|k| k.scaled()) { format!(" {}", ov.range_said(&data.r, k)) } else { String::new() };
    println!(
        "script: {}map: {map:<6} price of: {:<9} panel: {panel:<8} markets half: {half:<6} open good: {}{range}",
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
    // the frame before this one: how long its work took, and whether a year or its scenery came in during it
    let (filled, built, months) = (world.resource::<Data>().r.filled, world.resource::<Timeline>().built, world.resource::<UiState>().months);
    let now_t = world.resource::<Timeline>().t;
    let season_step = (season(world) * 192.0).floor() as i32;
    if t.frames == 0 {
        t.k0 = k;
        t.t0 = now_t;
    } else {
        // from the frame recorder, the one measure of a frame there is: the main world's
        // schedules, from the start of `First` to the end of `Last` (the work a hitch would
        // show in), and the extract that follows them
        let work = crate::frames::last_work(world);
        let (ms, parts) = work.as_ref().map_or((0.0, [0.0; 3]), |w| (w.ms, w.parts));
        t.extract.push(work.map_or(0.0, |w| w.extract_ms));
        t.work.push((ms, filled != t.last_filled, built != t.last_built && built.is_some(), months != t.last_months, season_step != t.last_season));
        // a frame long enough to see is told as it happens: where the clock stood, what came in, and where the time went
        if ms > 100.0 {
            println!(
                "script:   a frame's work took {ms:.0} ms with the clock at {now_t:.3}: {:.0} ms before the game's systems, {:.0} ms in them, {:.0} ms after (laying out the interface, transforms, what is in view); a year came: {}, scenery went up: {}, a month came: {}",
                parts[0],
                parts[1],
                parts[2],
                filled != t.last_filled,
                built != t.last_built && built.is_some(),
                months != t.last_months
            );
        }
    }
    (t.last_filled, t.last_built, t.last_months, t.last_season) = (filled, built, months, season_step);
    t.frames += 1;
    let step = stepping(world);
    if world.resource::<Timeline>().playing {
        t.playing += 1;
        t.stepping += step as u32;
        t.waited += (step && t.frames > 1 && now_t == t.last_t) as u32;
        t.can_act += (k >= latest && world.get_resource::<Live>().is_some_and(|l| l.at_turn())) as u32;
    }
    t.last_t = now_t;
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
        "script: sampled {secs} s, {} frames, {} years' turns, {:.1} months on the clock, {} decisions answered; the clock ran in {} frames: in {:.0}% of them the simulation stood at a year's turn (an order would be carried out at once), in {:.0}% a month or the year's end was being worked out, in {:.1}% the clock stood waiting for it",
        t.frames,
        k.saturating_sub(t.k0),
        (world.resource::<Timeline>().t - t.t0) * 12.0,
        t.answered,
        t.playing,
        pc(t.can_act),
        pc(t.stepping),
        pc(t.waited)
    );
    println!(
        "script:   the season went {:.2} of a year forwards and {:.2} back, {:.3} at most between two frames; the date read {} different things: {}",
        t.season_fwd,
        t.season_back,
        t.season_jump,
        t.dates.len(),
        t.dates.iter().take(14).cloned().collect::<Vec<_>>().join(", ") + if t.dates.len() > 14 { ", \u{2026}" } else { "" }
    );
    print_work(t);
}

/// The middle, the 99th in a hundred and the longest of some frame times, in milliseconds.
fn spread(ms: &mut [f32]) -> Option<(f32, f32, f32)> {
    if ms.is_empty() {
        return None;
    }
    ms.sort_by(|a, b| a.total_cmp(b));
    let at = |q: f32| ms[((ms.len() - 1) as f32 * q).round() as usize];
    Some((at(0.5), at(0.99), ms[ms.len() - 1]))
}

/// What a `sample` step says of the frames' work: of them all, and of those a
/// month or a year arrived in or a year's scenery went up in, which are the
/// ones that can hitch.
fn print_work(t: &Tally) {
    let line = |what: &str, mut ms: Vec<f32>| match spread(&mut ms) {
        Some((mid, p99, max)) => println!("script:   frame work, {what}: {} frames, middle {mid:.1} ms, 99th in a hundred {p99:.1} ms, longest {max:.1} ms", ms.len()),
        None => println!("script:   frame work, {what}: no frames"),
    };
    line("all", t.work.iter().map(|w| w.0).collect());
    line("nothing coming in and the season as it was", t.work.iter().filter(|w| !w.1 && !w.2 && !w.3 && !w.4).map(|w| w.0).collect());
    line("a step of the season and nothing else", t.work.iter().filter(|w| !w.1 && !w.2 && !w.3 && w.4).map(|w| w.0).collect());
    line("a month arriving", t.work.iter().filter(|w| w.3 && !w.1).map(|w| w.0).collect());
    line("a year arriving", t.work.iter().filter(|w| w.1).map(|w| w.0).collect());
    line("a year's scenery going up", t.work.iter().filter(|w| w.2).map(|w| w.0).collect());
    // (the frame work above is the main world's schedules; the renderer's taking of what
    // changed comes after them, with both worlds held still)
    if let Some((mid, p99, max)) = spread(&mut t.extract.clone()) {
        println!("script:   the extract after them: middle {mid:.1} ms, 99th in a hundred {p99:.1} ms, longest {max:.1} ms");
    }
}

/// The `goto:YEAR` step: the clock to January of `year`. At or behind the
/// present it goes there as the knob dragged there would; ahead of it a
/// sandbox runs on to it, as a press on the timeline there does, and the
/// family game, whose years ahead come only by playing them, does nothing.
fn goto(world: &mut World, year: i32) {
    let data = world.resource::<Data>();
    let (y0, latest) = (data.r.y0, data.latest());
    let k = crate::clock::january(y0, year);
    let to = |k: usize| crate::clock::month_year(crate::clock::date(y0, k as f32));
    if k <= latest {
        let mut tl = world.resource_mut::<Timeline>();
        tl.playing = false;
        tl.t = k as f32;
        println!("script: goto:{year}: the clock to {}", to(k));
        return;
    }
    let present = world.get_resource::<Live>().map(|live| crate::sandbox::Present::of(world.resource::<Data>(), live));
    match present {
        Some(at) if world.contains_resource::<crate::sandbox::Sandbox>() => {
            let now = world.resource::<Time>().elapsed_secs();
            let going = world.resource_scope(|world, mut sb: Mut<crate::sandbox::Sandbox>| sb.run_to(&mut world.resource_mut::<Timeline>(), at, k, None, now));
            if going {
                let cut = if k > at.last { format!(" (January {year} is past the run's last year, {}, and the run on stops at its end)", y0 + at.last as i32) } else { String::new() };
                println!("script: goto:{year}: running on to {}{cut}", to(k.min(at.last)));
            } else {
                println!("script: goto:{year}: nothing to run on to: the run's last year has been played");
            }
        }
        Some(_) => println!(
            "script: goto:{year}: nothing done: January {year} is ahead of the present ({}), and in the family game the years ahead come only by playing them (the clock, Next month, To January)",
            to(latest)
        ),
        None => println!("script: goto:{year}: nothing done: January {year} is past the end of this recorded run"),
    }
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
                let game = world.resource::<Game>();
                // (a sandbox's run on lets go of the clock for a frame between one year and the next)
                let running = world.get_resource::<crate::sandbox::Sandbox>().is_some_and(|s| s.running());
                // (and the Rules tab's question answered)
                let asking = world.get_resource::<crate::rulebook::Known>().is_some_and(|k| k.out());
                let quiet = !stepping(world) && !game.busy && game.follow.is_none() && !running && !asking && game.marks_built() && crate::scenery_ready(world.resource::<Timeline>(), world.resource::<Data>());
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
            Step::Goto(year) => {
                goto(world, year);
                frame = true;
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
                let mut picked = world.resource_mut::<Picked>();
                picked.what = what;
                picked.stamp = picked.stamp.wrapping_add(1);
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
            Step::Card(px) => {
                let mut q = world.query_filtered::<(&mut ScrollPosition, &ComputedNode), With<crate::InspectBody>>();
                for (mut s, c) in q.iter_mut(world) {
                    ui::scroll_panel(&mut s, c, -px);
                }
                frame = true;
            }
            Step::Chip(name) => {
                // Plain map and the pill are the same chip: the one that asks for no overlay
                let want: Option<Overlay> = crate::overlay::parse(&name, &world.resource::<Data>().r).and_then(|p| p.kind);
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
            Step::Range(range) => {
                match find_marked::<crate::overlay::RangeChip>(world, &format!("range:{}", range.word()), |c| c.0 == range) {
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
                let what = match half {
                    MarketsShow::Island => "half:island",
                    MarketsShow::Prices => "half:goods",
                    MarketsShow::Work => "half:work",
                };
                match find_marked::<MarketsChip>(world, what, |c| c.0 == half) {
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
            Step::Rule(name, word) => {
                if !sc.begun {
                    sc.begun = true;
                    sc.until = now + 10.0;
                }
                match rule_chip(world, &name, &word) {
                    Ok(Some(e)) => {
                        press(world, sc, e);
                        frame = true;
                    }
                    // the tab, the rules or the rule's numbers are still to come
                    Ok(None) => {
                        done = now >= sc.until;
                        if done {
                            println!("script: no click: rule:{name}={word}: the chip did not come in ten seconds");
                        }
                    }
                    Err(why) => {
                        println!("script: no click: rule:{name}={word}: {why}");
                        frame = true;
                    }
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
        let steps = parse("key:M;key:Shift+M;key:shift+g;chip:rent;chip:Plain;chip:pill;good:next;good:back;row:coal;half:island;half:goods;map;map:after;half:Work").unwrap();
        assert_eq!(steps.last(), Some(&Step::Half(MarketsShow::Work)));
        let steps = &steps[..steps.len() - 1];
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
        assert_eq!(parse("chip:rents").err().unwrap(), "chip:rents: the chips are rent, areas, reach, price, made, wages, growth, residents, deposits, traffic, plain and pill");
        assert_eq!(parse("chip:price:coal").err().unwrap(), "chip:price:coal: the chips are rent, areas, reach, price, made, wages, growth, residents, deposits, traffic, plain and pill");
        assert_eq!(parse("range:ever").err().unwrap(), "range:ever: range:year, range:30 or range:all");
        assert_eq!(parse("good:up").err().unwrap(), "good:up: good:next or good:back");
        assert_eq!(parse("half:all").err().unwrap(), "half:all: half:island, half:goods or half:work");
        // a panel that is no panel was once taken for closing the panel
        assert_eq!(parse("tab:famly").err().unwrap(), "tab:famly: the panels are family, holdings, markets, news, hand, rules, families and none");
        assert_eq!(parse("tab").err().unwrap(), "tab: the panels are family, holdings, markets, news, hand, rules, families and none");
        assert_eq!(parse("tab:none;tab:None;tab:Holdings").unwrap(), [Step::Tab(None), Step::Tab(None), Step::Tab(Some(ui::Tab::Holdings))]);
        // what print says of a panel: on one line, the empty texts left out, cut where asked
        let said: Vec<String> = ["March 1791", "", "By hand: the people\nof Garside halved", " "].map(String::from).to_vec();
        assert_eq!(said_in_one_line(&said, 600), "March 1791 | By hand: the people / of Garside halved");
        assert_eq!(said_in_one_line(&said, 5), "March\u{2026}");
        // a sandbox's tabs, and its name for the family's (key 5 is its fifth tab)
        assert_eq!(parse("tab:hand;tab:Rules;tab:families;key:5").unwrap(), [Step::Tab(Some(ui::Tab::Hand)), Step::Tab(Some(ui::Tab::Rules)), Step::Tab(Some(ui::Tab::Family)), Step::Key(KeyCode::Digit5, false)]);
        // steps are parted by semicolons: commas belong to pick and look
        assert!(parse("key:M,key:M").is_err());
    }

    #[test]
    fn goto_takes_a_year_and_nothing_else() {
        assert_eq!(parse("goto:1800;goto: 1795;settle").unwrap(), [Step::Goto(1800), Step::Goto(1795), Step::Settle]);
        for bad in ["goto", "goto:", "goto:soon", "goto:1800.5", "goto:1800,1"] {
            assert_eq!(parse(bad).err().unwrap(), format!("{bad}: goto takes a year, as goto:1800"));
        }
        assert!(check("wait:6;goto:1800;settle").is_ok());
        assert!(check("goto:next").is_err());
    }

    #[test]
    fn rule_takes_a_name_and_a_chips_word() {
        assert_eq!(parse("rule:money.growth=None;rule: hurdle = Half again").unwrap(), [Step::Rule("money.growth".into(), "None".into()), Step::Rule("hurdle".into(), "Half again".into())]);
        for bad in ["rule", "rule:", "rule:money.growth", "rule:=None", "rule:money.growth="] {
            assert_eq!(parse(bad).err().unwrap(), format!("{bad}: rule takes a name and a chip's word, as rule:money.growth=None"));
        }
        assert!(check("tab:rules;rule:money.growth=None;settle").is_ok());
    }
}
