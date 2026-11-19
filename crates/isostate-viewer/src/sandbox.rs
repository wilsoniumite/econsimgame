//! The sandbox, stage 1 (`docs/designs/sandbox.md`, sections 4.2, 4.3 and
//! 6.1): a game nobody plays a family in, changed by hand.
//!
//! A tray of tools hangs under the clock: Look, which is no tool and is how
//! the tray starts, then Money, People and Harvest. With a tool in hand a
//! second card under the first holds its choices as rows of chips and one line
//! that says what a click will now do. A click on the map with a tool in hand
//! (the clock at the present, nothing else asked of the simulation) sends the
//! edit to the simulation's thread as an order (`Cmd::ByHand`), and the line
//! the edit wrote in the island's news comes back and goes up as a toast. A
//! tool works on the town that was clicked, or on the town whose market the
//! clicked square or works trades in. Escape puts the tool down.
//!
//! Under the tools, Run on: 1 year, 10 years or 25 years, played as fast as
//! the simulation gives them and stopped at the January turn of the year
//! asked for. It is To January (`game.follow`) armed again as each year comes.

// see the note at the top of ui.rs
#![allow(deprecated)]

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use isostate_core::edits::{Edit, Purse};

use crate::family::chip_button;
use crate::game::Game;
use crate::live::{Cmd, Live};
use crate::ui::{card, text, MUTED};
use crate::{clock, Data, Pick, Picked, Timeline, Toasts, UiFont};

/// The hand's ink, the blue-black of a pen (section 4.1).
pub const HAND_INK: Color = Color::srgb(0.13, 0.24, 0.45);

/// The tools of stage 1, in the tray's order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Tool {
    #[default]
    Look,
    Money,
    People,
    Harvest,
}

impl Tool {
    const ALL: [Tool; 4] = [Tool::Look, Tool::Money, Tool::People, Tool::Harvest];

    fn word(self) -> &'static str {
        match self {
            Tool::Look => "Look",
            Tool::Money => "Money",
            Tool::People => "People",
            Tool::Harvest => "Harvest",
        }
    }

    /// The rows of choices, in the words of section 4.3: scripts click by them.
    fn rows(self) -> &'static [&'static [&'static str]] {
        match self {
            Tool::Look => &[],
            Tool::Money => &[&["Households", "Landlords", "Merchants"], &["This town", "Every town"], &["A tenth more", "Half as much again", "Double", "A tenth less", "Halved"]],
            Tool::People => &[&["A tenth more", "Half as many again", "A tenth fewer", "Halved"]],
            Tool::Harvest => &[&["This district", "The whole island"], &["A poor year", "A failure", "A bumper year"]],
        }
    }

    fn index(self) -> usize {
        Tool::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }
}

/// What each size chip multiplies by.
const MONEY_FACTORS: [f64; 5] = [1.1, 1.5, 2.0, 0.9, 0.5];
const PEOPLE_FACTORS: [f64; 4] = [1.1, 1.5, 0.9, 0.5];
const HARVEST_FACTORS: [f64; 3] = [0.9, 0.7, 1.1];

/// How a size reads in the tool's line.
const MONEY_WORDS: [&str; 5] = ["raised by a tenth", "raised by half", "doubled", "cut by a tenth", "halved"];
const PEOPLE_WORDS: [&str; 4] = ["raised by a tenth", "raised by half", "cut by a tenth", "halved"];
const HARVEST_WORDS: [&str; 3] = ["cut by a tenth", "cut by three tenths", "raised by a tenth"];

/// The years Run on offers.
const RUN_ON: [usize; 3] = [1, 10, 25];

/// A run on to a year: where it began and where it stops.
#[derive(Clone, Copy, Debug)]
struct RunOn {
    years: usize,
    /// the newest year (as an index) when it began, and the one it stops at
    from: usize,
    to: usize,
    /// the newest year when `game.follow` was last set: a step that is let go
    /// with no year come (a failed month) lets the run on go too
    armed: Option<usize>,
    /// when it began, in seconds since the app started
    began: f32,
}

/// A game changed by hand: present as a resource only in a sandbox.
#[derive(Resource, Default)]
pub struct Sandbox {
    pub tool: Tool,
    /// the chip chosen in each row of each tool's choices
    choice: [[usize; 3]; 4],
    run: Option<RunOn>,
    /// the last click seen (`Picked::stamp`)
    stamp: Option<u32>,
    /// an edit is out, and its answer goes up as a toast when it comes
    waiting: bool,
    /// raised whenever the tray is to be drawn again
    version: u64,
    drawn: Option<u64>,
}

impl Sandbox {
    /// Running on to a year.
    pub fn running(&self) -> bool {
        self.run.is_some()
    }

    fn chosen(&self, tool: Tool, row: usize) -> usize {
        self.choice[tool.index()][row]
    }

    /// The edit a click on `town` makes with the tool in hand, or why there is none.
    fn edit(&self, town: Option<usize>) -> Result<Edit, &'static str> {
        let c = self.choice[self.tool.index()];
        let at = |every: bool| if every { Ok(None) } else { town.map(Some).ok_or("Click a town, or a square or works in a town's district.") };
        match self.tool {
            Tool::Look => Err(""),
            Tool::Money => {
                let purse = [Purse::Households, Purse::Landlords, Purse::Merchants][c[0]];
                Ok(Edit::Money { purse, town: at(c[1] == 1)?, factor: MONEY_FACTORS[c[2]] })
            }
            Tool::People => Ok(Edit::People { town: at(false)?.unwrap_or(0), factor: PEOPLE_FACTORS[c[0]] }),
            Tool::Harvest => Ok(Edit::Harvest { town: at(c[0] == 1)?, factor: HARVEST_FACTORS[c[1]] }),
        }
    }

    /// One line, in muted ink, that says what a click will now do.
    fn hint(&self) -> String {
        let c = self.choice[self.tool.index()];
        match self.tool {
            Tool::Look => String::new(),
            Tool::Money => {
                let whose = ["households'", "landlords'", "merchants'"][c[0]];
                if c[1] == 1 {
                    format!("Click anywhere on the island: the {whose} money in every town is {}.", MONEY_WORDS[c[2]])
                } else {
                    format!("Click a town, or anywhere in its district: the {whose} money there is {}.", MONEY_WORDS[c[2]])
                }
            }
            Tool::People => {
                let after = if PEOPLE_FACTORS[c[0]] > 1.0 { "Those who arrive come with nothing." } else { "Those who stay inherit what the others had." };
                format!("Click a town, or anywhere in its district: its people are {}. {after}", PEOPLE_WORDS[c[0]])
            }
            Tool::Harvest => {
                if c[0] == 1 {
                    format!("Click anywhere on the island: the rest of this year's crop everywhere is {}.", HARVEST_WORDS[c[1]])
                } else {
                    format!("Click anywhere in a town's district: the rest of this year's crop there is {}.", HARVEST_WORDS[c[1]])
                }
            }
        }
    }
}

/// Where the tray is drawn.
#[derive(Component)]
struct TrayRoot;
#[derive(Component)]
struct ToolChip(Tool);
#[derive(Component)]
struct ChoiceChip {
    row: usize,
    index: usize,
}
#[derive(Component)]
struct RunOnChip(usize);

pub fn plugin(app: &mut App) {
    app.add_systems(PostStartup, setup.run_if(resource_exists::<Sandbox>)).add_systems(
        Update,
        (
            tray_clicks,
            use_tool,
            // (the answer is read in the frame it comes, before a question about the clicked
            // square, asked again after any order, puts the note of it away)
            answers.after(crate::game::pump).before(crate::game::ask_site),
            run_on.after(crate::time_input),
            draw_tray,
        )
            .chain()
            .run_if(resource_exists::<Sandbox>)
            .run_if(resource_exists::<Live>),
    );
}

fn setup(mut commands: Commands) {
    // a strip across the screen, 80 px from the top, that centres the tray and
    // takes no pointer itself: only the cards in it keep the pointer off the map
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(80.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(6.0),
            ..default()
        },
        TrayRoot,
    ));
}

/// The chips: a tool taken up or put down, a choice made, a run on begun or stopped.
#[allow(clippy::too_many_arguments)]
fn tray_clicks(
    tools: Query<(&Interaction, &ToolChip), Changed<Interaction>>,
    choices: Query<(&Interaction, &ChoiceChip), Changed<Interaction>>,
    runs: Query<(&Interaction, &RunOnChip), Changed<Interaction>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sb: ResMut<Sandbox>,
    mut tl: ResMut<Timeline>,
    data: Res<Data>,
    live: Res<Live>,
    time: Res<Time>,
) {
    for (i, t) in &tools {
        if *i == Interaction::Pressed {
            sb.tool = t.0;
            sb.version += 1;
        }
    }
    for (i, c) in &choices {
        if *i == Interaction::Pressed {
            let tool = sb.tool.index();
            sb.choice[tool][c.row] = c.index;
            sb.version += 1;
        }
    }
    for (i, r) in &runs {
        if *i != Interaction::Pressed {
            continue;
        }
        // the chip of a run on under way stops it at the next turn
        if sb.run.is_some_and(|run| run.years == r.0) {
            sb.run = None;
        } else if data.r.filled < data.r.ny {
            // the clock goes to the present and stops: the years are asked for one after another
            let latest = data.latest();
            tl.playing = false;
            tl.t = clock::at(latest, live.months.min(11));
            sb.run = Some(RunOn { years: r.0, from: latest, to: (latest + r.0).min(data.r.ny - 1), armed: None, began: time.elapsed_secs() });
        }
        sb.version += 1;
    }
    // Escape puts the tool down (and stops a run on, in `run_on`)
    if keys.just_pressed(KeyCode::Escape) && sb.tool != Tool::Look {
        sb.tool = Tool::Look;
        sb.version += 1;
    }
}

/// The town a click is on: the town clicked, or the one whose market the
/// clicked works or square trades in (as the inspector's own text reads it).
fn town_of(data: &Data, what: Option<Pick>, k: usize) -> Option<usize> {
    let r = &data.r;
    match what? {
        Pick::Town(t) => Some(t),
        Pick::Firm(f) => r.header.firms.get(f).and_then(|f| usize::try_from(f.town).ok()),
        Pick::Cell(c) => r.catch.get(k * r.n + c).and_then(|&t| (t as usize).checked_sub(1)),
    }
}

/// A click on the map with a tool in hand: the edit goes to the simulation.
#[allow(clippy::too_many_arguments)]
fn use_tool(mut sb: ResMut<Sandbox>, picked: Res<Picked>, tl: Res<Timeline>, data: Res<Data>, live: Res<Live>, mut game: ResMut<Game>, mut toasts: ResMut<Toasts>, time: Res<Time>) {
    let first = sb.stamp.is_none();
    if sb.stamp == Some(picked.stamp) {
        return;
    }
    sb.stamp = Some(picked.stamp);
    // (the click the game opened with, if any, is not a use of a tool)
    if first || sb.tool == Tool::Look {
        return;
    }
    let now = time.elapsed_secs();
    let latest = data.latest();
    let say = |toasts: &mut Toasts, line: &str| toasts.say("orders_refused", line, "By hand", now);
    // the hand works only in the present, and one thing at a time
    if (tl.t.floor() as usize) < latest || data.r.filled >= data.r.ny {
        say(&mut toasts, "The hand works only in the present: this is a year gone by, and nothing was changed.");
        return;
    }
    if game.busy || live.turning() || sb.run.is_some() {
        say(&mut toasts, "The simulation is busy (a year turning, a run on, or an edit still out): nothing was changed. Try again in a moment.");
        return;
    }
    let edit = match sb.edit(town_of(&data, picked.what, latest)) {
        Ok(e) => e,
        Err(why) => {
            say(&mut toasts, why);
            return;
        }
    };
    live.send(Cmd::ByHand(edit));
    game.busy = true;
    game.note = None;
    sb.waiting = true;
}

/// The edit's answer goes up as a toast: the line it wrote in the island's
/// news, with the figure before and after, or why it was refused.
fn answers(mut sb: ResMut<Sandbox>, game: Res<Game>, data: Res<Data>, live: Res<Live>, mut toasts: ResMut<Toasts>, time: Res<Time>) {
    if !sb.waiting || game.busy {
        return;
    }
    sb.waiting = false;
    let when = clock::month_year(clock::date(data.r.y0, clock::at(data.latest(), live.months.min(11))));
    if let Some((ok, line)) = &game.note {
        // (grey and labelled News until the tray's own colour comes, in stage 4)
        toasts.say(if *ok { "hand" } else { "orders_refused" }, line, &when, time.elapsed_secs());
        println!("sandbox: {} {when}: {line}", if *ok { "by hand," } else { "refused," });
    }
}

/// Run on: To January, armed again as each year comes, until the year asked
/// for is reached, the run's last year is played, or Space or Escape is
/// pressed (the year on its way still comes).
#[allow(clippy::too_many_arguments)]
fn run_on(mut sb: ResMut<Sandbox>, mut game: ResMut<Game>, mut tl: ResMut<Timeline>, data: Res<Data>, keys: Res<ButtonInput<KeyCode>>, mut toasts: ResMut<Toasts>, time: Res<Time>) {
    let Some(mut run) = sb.run else { return };
    let latest = data.latest();
    let stop = keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Space) || tl.playing;
    let reached = latest >= run.to || data.r.filled >= data.r.ny;
    // a step let go with no year come: the month failed, and the reason is under the date
    let dropped = game.follow.is_none() && run.armed == Some(latest);
    if stop || reached || dropped {
        if stop {
            // (Space starts the clock in `time_input`: here it only stops the run on)
            tl.playing = false;
        }
        let secs = time.elapsed_secs() - run.began;
        let years = latest.saturating_sub(run.from);
        let line = format!(
            "Ran on {years} year{} to January {} in {secs:.1} s: {:.1} years a second{}",
            if years == 1 { "" } else { "s" },
            data.r.y0 + latest as i32 + 1,
            years as f32 / secs.max(1e-3),
            if reached { "" } else { ", stopped short" }
        );
        println!("sandbox: {line}");
        toasts.say("hand", &line, "Run on", time.elapsed_secs());
        sb.run = None;
        sb.version += 1;
        return;
    }
    if game.follow.is_none() && !game.busy {
        tl.playing = false;
        game.follow = Some((latest + 1) as f32);
        run.armed = Some(latest);
        sb.run = Some(run);
    }
}

/// The tray, drawn again when a tool or a choice changes or a run on begins or ends.
fn draw_tray(mut commands: Commands, mut sb: ResMut<Sandbox>, font: Res<UiFont>, q: Query<(Entity, Option<&Children>), With<TrayRoot>>) {
    if sb.drawn == Some(sb.version) {
        return;
    }
    let Ok((root, children)) = q.single() else { return };
    sb.drawn = Some(sb.version);
    if let Some(ch) = children {
        for c in ch.iter() {
            commands.entity(c).despawn();
        }
    }
    let f = font.0.clone();
    let sb = &*sb;
    commands.entity(root).with_children(|p| {
        tray(p, &f, sb);
        if sb.tool != Tool::Look {
            choices(p, &f, sb);
        }
    });
}

fn chip_row(p: &mut ChildSpawnerCommands, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(Node { flex_wrap: FlexWrap::Wrap, align_items: AlignItems::Center, column_gap: Val::Px(6.0), row_gap: Val::Px(4.0), ..default() }).with_children(build);
}

/// The first card: the tools, and Run on.
fn tray(p: &mut ChildSpawnerCommands, f: &Handle<Font>, sb: &Sandbox) {
    p.spawn(card(Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(6.0),
        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
        max_width: Val::Px(520.0),
        border_radius: BorderRadius::all(Val::Px(12.0)),
        ..default()
    }))
    .with_children(|c| {
        chip_row(c, |row| {
            for t in Tool::ALL {
                chip_button(row, f, t.word(), sb.tool == t, HAND_INK, ToolChip(t));
            }
        });
        chip_row(c, |row| {
            row.spawn(text(f, "Run on:", 12.5, MUTED));
            for n in RUN_ON {
                let on = sb.run.is_some_and(|r| r.years == n);
                chip_button(row, f, if n == 1 { "1 year".to_string() } else { format!("{n} years") }.as_str(), on, HAND_INK, RunOnChip(n));
            }
        });
    });
}

/// The second card: the tool's choices, one chip inked in each row, and its line.
fn choices(p: &mut ChildSpawnerCommands, f: &Handle<Font>, sb: &Sandbox) {
    p.spawn(card(Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(6.0),
        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
        max_width: Val::Px(520.0),
        border_radius: BorderRadius::all(Val::Px(12.0)),
        ..default()
    }))
    .with_children(|c| {
        for (r, words) in sb.tool.rows().iter().enumerate() {
            chip_row(c, |row| {
                for (i, w) in words.iter().enumerate() {
                    chip_button(row, f, w, sb.chosen(sb.tool, r) == i, HAND_INK, ChoiceChip { row: r, index: i });
                }
            });
        }
        c.spawn(text(f, &sb.hint(), 12.5, MUTED));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tool_makes_the_edit_its_chips_say() {
        let mut sb = Sandbox { tool: Tool::Harvest, ..default() };
        // this district, a failure: a click needs a town
        sb.choice[Tool::Harvest.index()] = [0, 1, 0];
        assert_eq!(sb.edit(Some(4)), Ok(Edit::Harvest { town: Some(4), factor: 0.7 }));
        assert!(sb.edit(None).is_err());
        assert!(sb.hint().contains("cut by three tenths"), "{}", sb.hint());
        // the whole island: any click will do
        sb.choice[Tool::Harvest.index()] = [1, 2, 0];
        assert_eq!(sb.edit(None), Ok(Edit::Harvest { town: None, factor: 1.1 }));
        sb.tool = Tool::Money;
        sb.choice[Tool::Money.index()] = [1, 1, 4];
        assert_eq!(sb.edit(Some(2)), Ok(Edit::Money { purse: Purse::Landlords, town: None, factor: 0.5 }));
        sb.tool = Tool::People;
        sb.choice[Tool::People.index()] = [3, 0, 0];
        assert_eq!(sb.edit(Some(2)), Ok(Edit::People { town: 2, factor: 0.5 }));
        assert!(sb.edit(None).is_err());
        assert!(sb.hint().contains("inherit"));
        // every row's chips have a factor and a word
        assert_eq!(Tool::Money.rows()[2].len(), MONEY_FACTORS.len());
        assert_eq!(Tool::People.rows()[0].len(), PEOPLE_FACTORS.len());
        assert_eq!(Tool::Harvest.rows()[1].len(), HARVEST_FACTORS.len());
    }
}
