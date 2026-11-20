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
//! From stage 4 a press on the timeline ahead of the present, and a script's
//! `goto:YEAR`, run on in the same way to the January of that year (`run_to`),
//! and the tools' chips carry an icon before the word (`tool_chip`).
//!
//! From stage 6 two tools more. Idea (the lamp) offers the methods and
//! techniques not yet found on the island, asked of the game's thread with
//! the rules (`Cmd::Rules`, kept in `rulebook::Known`) when the tool is taken
//! up, after each of its edits and as each year comes; a click finds the one
//! chosen in the town clicked (`Edit::Discover`). Town (the market cross)
//! orders a town on the square clicked, founded at the year's end
//! (`Edit::FoundTown`), and is put down once the order is taken, since one
//! founding waits for each year's end.
//!
//! From stage 7 (7b) Works (the works' icon), with one row: Build a works and
//! Close a works. With Build a click on a square makes no edit: it takes the
//! square (`Sandbox::site`), and the choices card lists what could stand on
//! it as the game's thread tells it with its answer about the square
//! (`SiteInfo::hand`, `edits::works_offered`): the methods allowed by their
//! labels, the best expected return first, the cost and return of the one
//! chosen, and Build it here, which sends `Edit::BuildWorks`. In the middle
//! of a year it waits for the turn, with a stake on its square and a row with
//! Take back in the Hand tab. With Close a click on a works sends
//! `Edit::CloseWorks`, and the tool is put down after each use. The same row
//! has Found a university and Close a university (the core's
//! docs/rules/state.md, "The university"): a click on a square founds one in
//! the town whose district it is (`Edit::FoundUniversity`), and a click in a
//! town's district closes that town's (`Edit::CloseUniversity`); both wait for
//! the year's turn, and the tool is put down after each.

// see the note at the top of ui.rs
#![allow(deprecated)]

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use isostate_core::edits::{Edit, Purse};

use bevy::ui::FocusPolicy;

use crate::family::chip_button;
use crate::game::Game;
use crate::icons::{icon, Icons};
use crate::live::{Cmd, Live};
use crate::rulebook::Known;
use crate::ui::{card, text, BTN, INK, MUTED};
use crate::{clock, Data, Pick, Picked, Timeline, Toasts, UiFont};

/// The hand's ink, the blue-black of a pen (section 4.1).
pub const HAND_INK: Color = Color::srgb(0.13, 0.24, 0.45);

/// The hand's two refusals before anything is sent, the tray's and the Rules
/// tab's alike: it works only in the present, and one thing at a time.
pub const PAST: &str = "The hand works only in the present: this is a year gone by, and nothing was changed.";
pub const BUSY: &str = "The simulation is busy (a year turning, a run on, or an edit still out): nothing was changed. Try again in a moment.";

/// The tools, in the tray's order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Tool {
    #[default]
    Look,
    Money,
    People,
    Harvest,
    Idea,
    Town,
    Works,
}

impl Tool {
    const ALL: [Tool; 7] = [Tool::Look, Tool::Money, Tool::People, Tool::Harvest, Tool::Idea, Tool::Town, Tool::Works];

    fn word(self) -> &'static str {
        match self {
            Tool::Look => "Look",
            Tool::Money => "Money",
            Tool::People => "People",
            Tool::Harvest => "Harvest",
            Tool::Idea => "Idea",
            Tool::Town => "Town",
            Tool::Works => "Works",
        }
    }

    /// The rows of choices, in the words of section 4.3: scripts click by them.
    /// (Idea's one row is the ideas not yet found, as the game's thread tells them.)
    fn rows(self) -> &'static [&'static [&'static str]] {
        match self {
            Tool::Look | Tool::Idea => &[],
            Tool::Money => &[&["Households", "Landlords", "Merchants"], &["This town", "Every town"], &["A tenth more", "Half as much again", "Double", "A tenth less", "Halved"]],
            Tool::People => &[&["A tenth more", "Half as many again", "A tenth fewer", "Halved"]],
            Tool::Harvest => &[&["This district", "The whole island"], &["A poor year", "A failure", "A bumper year"]],
            Tool::Town => &[&["Found a town here"]],
            Tool::Works => &[&["Build a works", "Close a works", "Found a university", "Close a university"]],
        }
    }

    fn index(self) -> usize {
        Tool::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }

    /// The icon before the word, and whether it is drawn in its own colours
    /// (the coin keeps its gold) rather than in the chip's ink.
    fn icon(self, icons: &Icons) -> (&Handle<Image>, bool) {
        match self {
            Tool::Look => (&icons.eye, false),
            Tool::Money => (&icons.coin, true),
            Tool::People => (&icons.people, false),
            Tool::Harvest => (&icons.sheaf, false),
            Tool::Idea => (&icons.lamp, false),
            Tool::Town => (&icons.cross, false),
            Tool::Works => (&icons.works, false),
        }
    }
}

/// With Build a works in hand, what could stand on the square taken, as the
/// game's thread told it (`SiteInfo::hand`, `edits::works_offered`).
#[derive(Clone, Debug, PartialEq)]
struct Offers {
    /// "31, 40, in Melbourne's district"
    place: String,
    /// the methods a works by hand may be built of here, the best expected return first
    list: Vec<Offer>,
    /// how many more the square's town knows, refused for want of room or of a market
    more: usize,
}

/// A works that may be built on the square: its method's name and label,
/// what it costs and what it should return a year (a fraction).
#[derive(Clone, Debug, PartialEq)]
struct Offer {
    name: String,
    label: String,
    cost: f64,
    ret: f64,
}

impl Offers {
    /// The list as the thread's answer gives it, or its sentence when no works
    /// can stand on the square. `place` says where the square is.
    fn of(hand: &Result<Vec<isostate_core::SiteOption>, String>, place: String) -> Result<Offers, String> {
        let options = hand.as_ref().map_err(|why| capital(why))?;
        // those that may be built and should pay: a works expected to lose money at last year's
        // prices may still be built by hand (the edit refuses it no more than the island's own
        // entrants are refused it), but it is not offered, and is counted with the refused
        let mut list: Vec<Offer> = options
            .iter()
            .filter(|o| o.allowed.is_ok())
            .filter_map(|o| o.sums.as_ref().ok().map(|s| Offer { name: o.name.clone(), label: o.label.clone(), cost: s.capital_cost, ret: s.expected_return }))
            .filter(|o| o.ret > 0.0)
            .collect();
        list.sort_by(|a, b| b.ret.total_cmp(&a.ret));
        // (a method the town does not know has no sums, and is not counted)
        let more = options.iter().filter(|o| o.sums.is_ok()).count() - list.len();
        Ok(Offers { place, list, more })
    }
}

/// A sentence begun with a capital.
fn capital(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or_else(String::new, |first| first.to_uppercase().chain(c).collect())
}

/// A sum of money as the core writes it, "12,400".
fn sum(x: f64) -> String {
    isostate_core::util::fmt_thousands_f0(x)
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
    /// the Run on chip that began it (its years), which stays inked while it
    /// lasts; none when the timeline or a script began it, or moved its year
    chip: Option<usize>,
    /// the newest year (as an index) when it began, and the one it stops at:
    /// the clock's whole number at the January it is to reach
    from: usize,
    to: usize,
    /// the clock when it began, for its line
    since: f32,
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
    choice: [[usize; 3]; 7],
    run: Option<RunOn>,
    /// the last click seen (`Picked::stamp`)
    stamp: Option<u32>,
    /// an edit is out, made with this tool, and its answer goes up as a toast when it comes
    sent: Option<Tool>,
    /// the ideas not yet found (name and label), as the game's thread last told
    /// them; none while they are asked for afresh (the tool just taken up, or an
    /// idea just found by hand)
    ideas: Option<Vec<(String, String)>>,
    /// the idea chosen, by its name (`Content::tech_name`)
    idea: Option<String>,
    /// the newest year when the ideas were last asked for; none when they are to be asked again
    ideas_at: Option<usize>,
    /// the ideas are asked for, and the answer has not come
    asking: bool,
    /// with Build a works in hand: the square taken (`Sandbox::click`), and
    /// what could stand on it once the game's thread has said (none while it
    /// is asked about, an Err the sentence that no works can stand there)
    site: Option<usize>,
    offers: Option<Result<Offers, String>>,
    /// the method chosen among the offers, by its name (`Content::method_name`)
    method: Option<String>,
    /// the January the works would be built in, while the year is in progress
    /// (none at a turn, where it is built at once)
    turn: Option<i32>,
    /// raised whenever the tray is to be drawn again
    version: u64,
    drawn: Option<u64>,
}

/// Where the island stands when a run on is asked for.
#[derive(Clone, Copy, Debug)]
pub struct Present {
    /// the newest year worked out, as an index (`Data::latest`)
    pub latest: usize,
    /// the months of the year after it that have come
    pub months: usize,
    /// the run's last year, as an index
    pub last: usize,
}

impl Present {
    pub fn of(data: &Data, live: &Live) -> Present {
        Present { latest: data.latest(), months: live.months.min(11), last: data.r.ny.max(1) - 1 }
    }

    /// The clock at the present.
    pub fn clock(self) -> f32 {
        clock::at(self.latest, self.months)
    }
}

impl Sandbox {
    /// Running on to a year.
    pub fn running(&self) -> bool {
        self.run.is_some()
    }

    /// The clock's whole number at the January a run on under way is to reach.
    pub fn target(&self) -> Option<usize> {
        self.run.map(|r| r.to)
    }

    /// Run on to the January whose clock is `to` (a year's end, as the clock
    /// counts: `clock::january`), as Run on's chips, a press on the timeline
    /// ahead of the present and a script's `goto:YEAR` all ask. The year is cut
    /// to the run's last; one at or behind the present starts nothing (false).
    /// A run on that has not begun puts the clock at the present and stops
    /// it; one under way has its year moved and its chip (`chip`, a chip's
    /// years, or none) changed, and goes on from where it was.
    pub fn run_to(&mut self, tl: &mut Timeline, at: Present, to: usize, chip: Option<usize>, now: f32) -> bool {
        let to = to.min(at.last);
        if to <= at.latest {
            return false;
        }
        match &mut self.run {
            Some(run) => {
                if (run.to, run.chip) == (to, chip) {
                    return true;
                }
                (run.to, run.chip) = (to, chip);
            }
            None => {
                // the clock goes to the present and stops: the years are asked for one after another
                tl.playing = false;
                tl.t = at.clock();
                self.run = Some(RunOn { chip, from: at.latest, to, since: tl.t, armed: None, began: now });
            }
        }
        self.version += 1;
        true
    }

    fn chosen(&self, tool: Tool, row: usize) -> usize {
        self.choice[tool.index()][row]
    }

    /// The edit a click on `what` makes with the tool in hand, or why there is
    /// none; `town` is the town the click is on (`town_of`).
    fn edit(&self, town: Option<usize>, what: Option<Pick>) -> Result<Edit, &'static str> {
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
            Tool::Idea => {
                if self.ideas.is_none() {
                    return Err("The ideas not yet found are still being asked for: nothing was changed. Try again in a moment.");
                }
                let Some(tech) = self.idea.clone() else { return Err("Choose an idea first: the chips are the methods and techniques not yet found.") };
                Ok(discover_edit(tech, at(false)?.unwrap_or(0)))
            }
            Tool::Town => match what {
                Some(Pick::Cell(cell)) => Ok(Edit::FoundTown { cell }),
                Some(Pick::Town(_)) => Err("Click open land, not a town."),
                Some(Pick::Firm(_)) => Err("Click open land, not a works."),
                None => Err("Click a square of land in a town's district."),
            },
            // (a square is taken by `click`, and the works is built from its chip, `build`)
            Tool::Works => Err("Click a square of land, then choose a works and Build it here."),
        }
    }

    /// A click on `what` with the tool in hand: the edit it makes (Some), or
    /// none when it only takes a square, with Build a works in hand, whose list
    /// is then asked for (`game::ask_site` asks about any square clicked); or
    /// why nothing is done. Close a works is put down once used.
    fn click(&mut self, town: Option<usize>, what: Option<Pick>) -> Result<Option<Edit>, &'static str> {
        if self.tool != Tool::Works {
            return self.edit(town, what).map(Some);
        }
        let chosen = self.chosen(Tool::Works, 0);
        if chosen >= 2 {
            // a university founded or closed: the tool is put down after each use
            let edit = match (chosen, what, town) {
                (2, Some(Pick::Cell(cell)), _) => Edit::FoundUniversity { cell },
                (2, _, _) => return Err("Click a square of land in the town's district."),
                (_, Some(Pick::Firm(_)) | Some(Pick::Cell(_)) | Some(Pick::Town(_)), Some(t)) => Edit::CloseUniversity { town: t },
                _ => return Err("Click in the district of the town whose university is to close."),
            };
            self.tool = Tool::Look;
            self.version += 1;
            return Ok(Some(edit));
        }
        let close = chosen == 1;
        match (close, what) {
            (false, Some(Pick::Cell(cell))) => {
                if self.site != Some(cell) {
                    (self.site, self.offers) = (Some(cell), None);
                }
                self.version += 1;
                Ok(None)
            }
            (false, other) => {
                (self.site, self.offers) = (None, None);
                self.version += 1;
                Err(match other {
                    Some(Pick::Town(_)) => "Click a square of land, not a town.",
                    Some(Pick::Firm(_)) => "Click a square of land, not a works.",
                    _ => "Click a square of land.",
                })
            }
            (true, Some(Pick::Firm(works))) => {
                // works that are to be closed are put down after each use (section 4.3)
                self.tool = Tool::Look;
                self.version += 1;
                Ok(Some(Edit::CloseWorks { works }))
            }
            (true, _) => Err("Click a works."),
        }
    }

    /// The edit Build it here makes: the method chosen, on the square taken.
    fn build(&self) -> Result<Edit, &'static str> {
        let Some(cell) = self.site else { return Err("Click a square of land first.") };
        let Some(Ok(offers)) = &self.offers else { return Err("What could stand on the square is still being asked for: nothing was changed. Try again in a moment.") };
        match &self.method {
            Some(m) if offers.list.iter().any(|o| &o.name == m) => Ok(Edit::BuildWorks { method: m.clone(), cell }),
            _ => Err("Choose a works first: the chips are those that may stand on the square."),
        }
    }

    /// Keep what the game's thread said could stand on the square taken: the
    /// method chosen stays while it is offered, else the best is chosen, and
    /// the tray is drawn again when anything differs. True when it did.
    fn keep_offers(&mut self, offers: Result<Offers, String>) -> bool {
        if self.offers.as_ref() == Some(&offers) {
            return false;
        }
        if let Ok(o) = &offers {
            if !self.method.as_ref().is_some_and(|m| o.list.iter().any(|x| &x.name == m)) {
                self.method = o.list.first().map(|x| x.name.clone());
            }
        }
        self.offers = Some(offers);
        self.version += 1;
        true
    }

    /// The line for the works chosen: what it costs and should return, and when it is built.
    fn offer_line(&self) -> Option<String> {
        let Some(Ok(offers)) = &self.offers else { return None };
        let o = offers.list.iter().find(|o| Some(&o.name) == self.method.as_ref())?;
        let when = match self.turn {
            Some(year) => format!("It is built at the year's turn, in January {year}."),
            None => "It is built at the year's turn.".to_string(),
        };
        Some(format!("{} here: it costs {}, paid by hand, and should return {} a year. {when}", o.label, sum(o.cost), isostate_core::util::fmt_pct(o.ret, 0)))
    }

    /// The label of the idea chosen, while it is among those not yet found.
    fn idea_label(&self) -> Option<&str> {
        let name = self.idea.as_deref()?;
        self.ideas.as_ref()?.iter().find(|(n, _)| n == name).map(|(_, label)| label.as_str())
    }

    /// Keep the ideas not yet found, as the game's thread told them: the idea
    /// chosen is let go when it is no longer among them (found by hand or by the
    /// island), and the tray is drawn again when the list differs.
    fn keep_ideas(&mut self, list: Vec<(String, String)>) {
        if self.idea.as_ref().is_some_and(|i| !list.iter().any(|(n, _)| n == i)) {
            self.idea = None;
        }
        if self.ideas.as_ref() != Some(&list) {
            self.ideas = Some(list);
            self.version += 1;
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
            Tool::Idea => match (self.idea_label(), &self.ideas) {
                (Some(label), _) => format!("Click a town, or anywhere in its district: {label} is found there, and spreads from it as any idea does."),
                (None, Some(list)) if list.is_empty() => "Every method and technique has been found on the island: there is nothing left to find by hand.".to_string(),
                _ => "Choose an idea, then click a town, or anywhere in its district: it is found there, and spreads from it as any idea does.".to_string(),
            },
            Tool::Town => "Click a square of land in a town's district: a town is founded on it at the year's end, if the founding rule allows it there.".to_string(),
            Tool::Works if c[0] == 1 => "Click a works: it is closed at the year's turn, and its cash goes to its owners. The tool is put down after.".to_string(),
            Tool::Works if c[0] == 2 => "Click a square of land: a university is founded at the year's turn in the town whose district it is, paid for by the treasury. The tool is put down after.".to_string(),
            Tool::Works if c[0] == 3 => "Click in a town's district: its university is closed at the year's turn. The tool is put down after.".to_string(),
            Tool::Works => "Click a square of land: the works that could stand on it are listed here, each with what it costs and should return.".to_string(),
        }
    }
}

/// The widest the tray's cards may be (section 4.3): wider than this, the chips wrap.
const TRAY_WIDEST: f32 = 520.0;

/// Where the tray is drawn.
#[derive(Component)]
struct TrayRoot;
/// The tray's first card, the tools and Run on.
#[derive(Component)]
struct TrayCard;
#[derive(Component)]
struct ToolChip(Tool);
#[derive(Component)]
struct ChoiceChip {
    row: usize,
    index: usize,
}
#[derive(Component)]
struct RunOnChip(usize);
/// An idea's chip: its name (`Content::tech_name`); the chip carries its label.
#[derive(Component)]
struct IdeaChip(String);
/// A method's chip in the Works tool's list: its name; the chip carries its label.
#[derive(Component)]
struct MethodChip(String);
/// The chip that builds the works chosen on the square taken.
#[derive(Component)]
struct BuildHere;

pub fn plugin(app: &mut App) {
    app.add_systems(PostStartup, setup.run_if(resource_exists::<Sandbox>)).add_systems(
        Update,
        (
            tray_clicks,
            use_tool,
            build_here,
            works_list.after(crate::game::pump),
            // (the answer is read in the frame it comes, before a question about the clicked
            // square, asked again after any order, puts the note of it away)
            answers.after(crate::game::pump).before(crate::game::ask_site),
            ideas,
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
    idea_chips: Query<(&Interaction, &IdeaChip), Changed<Interaction>>,
    method_chips: Query<(&Interaction, &MethodChip), Changed<Interaction>>,
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
            if t.0 == Tool::Idea && sb.tool != Tool::Idea {
                // taken up: the ideas are asked for afresh, and the row waits for them
                (sb.ideas, sb.ideas_at) = (None, None);
            }
            if t.0 == Tool::Works && sb.tool != Tool::Works {
                // taken up: no square is taken yet
                (sb.site, sb.offers) = (None, None);
            }
            sb.tool = t.0;
            sb.version += 1;
        }
    }
    for (i, c) in &method_chips {
        if *i == Interaction::Pressed {
            sb.method = Some(c.0.clone());
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
    for (i, c) in &idea_chips {
        if *i == Interaction::Pressed {
            sb.idea = Some(c.0.clone());
            sb.version += 1;
        }
    }
    for (i, r) in &runs {
        if *i != Interaction::Pressed {
            continue;
        }
        // the chip of a run on under way stops it at the next turn
        if sb.run.is_some_and(|run| run.chip == Some(r.0)) {
            sb.run = None;
            sb.version += 1;
        } else {
            let at = Present::of(&data, &live);
            sb.run_to(&mut tl, at, at.latest + r.0, Some(r.0), time.elapsed_secs());
        }
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
    let say = |toasts: &mut Toasts, line: &str| toasts.say("orders_refused", line, "By hand", now);
    if let Err(why) = may_act(&sb, &tl, &data, &live, &game) {
        say(&mut toasts, why);
        return;
    }
    let tool = sb.tool;
    let edit = match sb.click(town_of(&data, picked.what, data.latest()), picked.what) {
        Ok(Some(e)) => e,
        // a square taken, with Build a works in hand: its list is asked for
        Ok(None) => return,
        Err(why) => {
            say(&mut toasts, why);
            return;
        }
    };
    live.send(Cmd::ByHand(edit));
    game.busy = true;
    game.note = None;
    sb.sent = Some(tool);
}

/// The hand's edit for an idea, found in a town: the Idea tool's, and a
/// town's card's (`town_ideas`), made by this one function so that the two
/// cannot differ.
pub(crate) fn discover_edit(tech: String, town: usize) -> Edit {
    Edit::Discover { tech, town }
}

/// The edit the Idea tool makes with `name` chosen, on a click on town `town`:
/// for holding a town's card to the same edit (`town_ideas`).
#[cfg(test)]
pub(crate) fn idea_tool_edit(name: &str, town: usize) -> Edit {
    let mut sb = Sandbox { tool: Tool::Idea, ..default() };
    sb.keep_ideas(vec![(name.to_string(), name.replace('_', " "))]);
    sb.idea = Some(name.to_string());
    sb.edit(Some(town), Some(Pick::Town(town))).expect("the Idea tool makes an edit with an idea chosen and a town clicked")
}

/// Send an edit made somewhere other than the map, under the tray's guards,
/// as if the tool `tool` had made it: its answer goes up as that tool's does
/// (`answers`). Or the guard's refusal, and nothing is sent.
pub(crate) fn send_as(tool: Tool, edit: Edit, sb: &mut Sandbox, tl: &Timeline, data: &Data, live: &Live, game: &mut Game) -> Result<(), &'static str> {
    may_act(sb, tl, data, live, game)?;
    live.send(Cmd::ByHand(edit));
    game.busy = true;
    game.note = None;
    sb.sent = Some(tool);
    Ok(())
}

/// The hand works only in the present, and one thing at a time: the tray's
/// two refusals before anything is sent, or Ok.
fn may_act(sb: &Sandbox, tl: &Timeline, data: &Data, live: &Live, game: &Game) -> Result<(), &'static str> {
    if (tl.t.floor() as usize) < data.latest() || data.r.filled >= data.r.ny {
        return Err(PAST);
    }
    if game.busy || live.turning() || sb.run.is_some() {
        return Err(BUSY);
    }
    Ok(())
}

/// Build it here: the works chosen is built on the square taken, under the
/// tray's guards; the tool stays in hand.
#[allow(clippy::too_many_arguments)]
fn build_here(q: Query<&Interaction, (Changed<Interaction>, With<BuildHere>)>, mut sb: ResMut<Sandbox>, tl: Res<Timeline>, data: Res<Data>, live: Res<Live>, mut game: ResMut<Game>, mut toasts: ResMut<Toasts>, time: Res<Time>) {
    if !q.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    match may_act(&sb, &tl, &data, &live, &game).and_then(|()| sb.build()) {
        Ok(edit) => {
            live.send(Cmd::ByHand(edit));
            game.busy = true;
            game.note = None;
            sb.sent = Some(Tool::Works);
        }
        Err(why) => toasts.say("orders_refused", why, "By hand", time.elapsed_secs()),
    }
}

/// With Build a works in hand and a square taken, the list of what could
/// stand there is taken from the game's answer about the square
/// (`game.site`, which `game::ask_site` asks for after every click and again
/// after every order and every year), kept, and printed once as it comes:
/// `sandbox: on 31, 40: brickworks 622 42%, ...`. The January the works
/// would be built in is kept too, so the tray says it as the months go by.
fn works_list(mut sb: ResMut<Sandbox>, game: Res<Game>, data: Res<Data>, live: Res<Live>) {
    if sb.tool != Tool::Works {
        return;
    }
    // (the year in progress is the one after the newest worked out, and its turn the January after that)
    let turn = (live.months > 0).then(|| data.r.y0 + data.latest() as i32 + 2);
    if sb.turn != turn {
        sb.turn = turn;
        sb.version += 1;
    }
    let Some(cell) = sb.site else { return };
    let Some(site) = game.site.as_ref().filter(|s| s.cell == cell) else { return };
    let r = &data.r;
    let at = format!("{}, {}", cell % r.w.max(1), cell / r.w.max(1));
    let place = match site.town.and_then(|t| r.header.towns.get(t)) {
        Some(t) => format!("{at}, in {}'s district", t.name),
        None => at.clone(),
    };
    let offers = Offers::of(&site.hand, place);
    if sb.keep_offers(offers.clone()) {
        match offers {
            Ok(o) => {
                let each: Vec<String> = o.list.iter().map(|x| format!("{} {} {}", x.label.to_lowercase(), sum(x.cost), isostate_core::util::fmt_pct(x.ret, 0))).collect();
                println!("sandbox: on {at}: {}{}", if each.is_empty() { "nothing offered".to_string() } else { each.join(", ") }, if o.more > 0 { format!("; {} more known, with no room, no market or no profit", o.more) } else { String::new() });
            }
            Err(why) => println!("sandbox: on {at}: {why}"),
        }
    }
}

/// The edit's answer goes up as a toast: the line it wrote in the island's
/// news, with the figure before and after, or why it was refused. An idea's
/// answer has the ideas asked for again; a town's puts the tool down.
fn answers(mut sb: ResMut<Sandbox>, game: Res<Game>, data: Res<Data>, live: Res<Live>, mut toasts: ResMut<Toasts>, time: Res<Time>) {
    let Some(tool) = sb.sent else { return };
    if game.busy {
        return;
    }
    sb.sent = None;
    if tool == Tool::Idea {
        (sb.ideas, sb.ideas_at) = (None, None);
        sb.version += 1;
    }
    let when = clock::month_year(clock::date(data.r.y0, clock::at(data.latest(), live.months.min(11))));
    if let Some((ok, line)) = &game.note {
        let mut when = when;
        if *ok && tool == Tool::Town {
            // the order is taken: one founding waits for each year's end, so the tool is put down
            if let Some(january) = stands_from(line) {
                // (said beside the date: the line itself stays the news's title, which the toasts
                // match so that the news of it does not go up a second time)
                when = format!("{when}: the town stands from January {january}");
            }
            if sb.tool == Tool::Town {
                sb.tool = Tool::Look;
                sb.version += 1;
            }
        }
        // (in the hand's ink and labelled "By hand", from stage 4)
        toasts.say(if *ok { "hand" } else { "orders_refused" }, line, &when, time.elapsed_secs());
        println!("sandbox: {} {when}: {line}", if *ok { "by hand," } else { "refused," });
    }
}

/// The January a town ordered by hand stands from: the year after the one its
/// order names ("... at the end of 1791").
fn stands_from(line: &str) -> Option<i32> {
    line.rsplit_once("at the end of ").and_then(|(_, y)| y.trim().parse::<i32>().ok()).map(|y| y + 1)
}

/// With Idea in hand, the ideas not yet found are asked of the game's thread
/// when the tool is taken up, after each of its edits and as each year comes
/// (never during a run on, nor while the year turns); the answer is kept, and
/// printed once as it comes: `sandbox: not yet found: label, label, ...`.
fn ideas(mut sb: ResMut<Sandbox>, known: Option<ResMut<Known>>, live: Res<Live>, data: Res<Data>, game: Res<Game>) {
    let Some(mut known) = known else { return };
    if sb.asking {
        if known.out() {
            return;
        }
        sb.asking = false;
        let list = known.unfound().to_vec();
        let labels: Vec<&str> = list.iter().map(|(_, label)| label.as_str()).collect();
        println!("sandbox: not yet found: {}", if labels.is_empty() { "nothing".to_string() } else { labels.join(", ") });
        sb.keep_ideas(list);
        return;
    }
    let latest = data.latest();
    if sb.tool == Tool::Idea && sb.ideas_at != Some(latest) && sb.run.is_none() && !game.busy && !live.turning() {
        sb.ideas_at = Some(latest);
        sb.asking = true;
        known.ask(&live);
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
            "Ran on {years} year{} from {} to January {} in {secs:.1} s: {:.1} years a second{}",
            if years == 1 { "" } else { "s" },
            clock::month_year(clock::date(data.r.y0, run.since)),
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
fn draw_tray(mut commands: Commands, mut sb: ResMut<Sandbox>, font: Res<UiFont>, icons: Res<Icons>, q: Query<(Entity, Option<&Children>), With<TrayRoot>>) {
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
        tray(p, &f, &icons, sb);
        if sb.tool != Tool::Look {
            choices(p, &f, sb);
        }
    });
}

/// How many chips `width` pixels wide go in one row of a tray card at its widest.
fn chips_a_row(width: f32) -> usize {
    // the card's padding is 12 px a side and the chips stand 6 px apart
    ((TRAY_WIDEST - 24.0 + 6.0) / (width + 6.0)).floor().max(0.0) as usize
}

/// The tray as it is laid out, for a script's `print`: the first card's width,
/// each tool's chip, and how many chips of their mean width go in two rows.
pub fn tray_measure(world: &mut World) -> Option<String> {
    let mut q = world.query_filtered::<&ComputedNode, With<TrayCard>>();
    let card = q.iter(world).next().map(|c| c.size().x * c.inverse_scale_factor)?;
    let mut q = world.query::<(&ComputedNode, &ToolChip)>();
    let mut chips: Vec<(usize, &str, f32)> = q.iter(world).map(|(c, t)| (t.0.index(), t.0.word(), c.size().x * c.inverse_scale_factor)).collect();
    chips.sort_by_key(|c| c.0);
    let mean = chips.iter().map(|c| c.2).sum::<f32>() / chips.len().max(1) as f32;
    let each: Vec<String> = chips.iter().map(|c| format!("{} {:.0}", c.1, c.2)).collect();
    Some(format!(
        "the tray's card is {card:.0} px wide (at most {TRAY_WIDEST:.0}); the tools' chips {} px; chips of their mean width ({mean:.0} px) go {} to a row, {} in two rows",
        each.join(", "),
        chips_a_row(mean),
        2 * chips_a_row(mean)
    ))
}

fn chip_row(p: &mut ChildSpawnerCommands, build: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(Node { flex_wrap: FlexWrap::Wrap, align_items: AlignItems::Center, column_gap: Val::Px(6.0), row_gap: Val::Px(4.0), ..default() }).with_children(build);
}

/// A tool's chip: its icon before its word, built as `family::chip_button`
/// builds a chip, white on the hand's ink while the tool is in hand. The word
/// is what a script clicks by.
fn tool_chip(p: &mut ChildSpawnerCommands, f: &Handle<Font>, icons: &Icons, tool: Tool, on: bool) {
    let mut e = p.spawn((
        Node { padding: UiRect::new(Val::Px(7.0), Val::Px(9.0), Val::Px(3.0), Val::Px(3.0)), column_gap: Val::Px(4.0), align_items: AlignItems::Center, border_radius: BorderRadius::all(Val::Px(10.0)), ..default() },
        BackgroundColor(if on { HAND_INK } else { BTN }),
        Interaction::default(),
        FocusPolicy::Block,
        ToolChip(tool),
    ));
    if !on {
        e.insert(crate::ui::Hoverable);
    }
    let (ic, own) = tool.icon(icons);
    let ink = if on { Color::WHITE } else { INK };
    e.with_children(|c| {
        c.spawn(icon(ic, 16.0, if own { Color::WHITE } else { ink }));
        c.spawn(text(f, tool.word(), 12.5, ink));
    });
}

/// The first card: the tools, and Run on.
fn tray(p: &mut ChildSpawnerCommands, f: &Handle<Font>, icons: &Icons, sb: &Sandbox) {
    p.spawn((
        card(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
            max_width: Val::Px(TRAY_WIDEST),
            border_radius: BorderRadius::all(Val::Px(12.0)),
            ..default()
        }),
        TrayCard,
    ))
    .with_children(|c| {
        chip_row(c, |row| {
            for t in Tool::ALL {
                tool_chip(row, f, icons, t, sb.tool == t);
            }
        });
        chip_row(c, |row| {
            row.spawn(text(f, "Run on:", 12.5, MUTED));
            for n in RUN_ON {
                let on = sb.run.is_some_and(|r| r.chip == Some(n));
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
        max_width: Val::Px(TRAY_WIDEST),
        border_radius: BorderRadius::all(Val::Px(12.0)),
        ..default()
    }))
    .with_children(|c| {
        if sb.tool == Tool::Idea {
            // one row of the ideas not yet found, by their labels, wrapping; "asking..." until they come
            chip_row(c, |row| match &sb.ideas {
                None => {
                    row.spawn(text(f, "asking...", 12.5, MUTED));
                }
                Some(list) => {
                    for (name, label) in list {
                        chip_button(row, f, label, sb.idea.as_deref() == Some(name.as_str()), HAND_INK, IdeaChip(name.clone()));
                    }
                }
            });
        }
        for (r, words) in sb.tool.rows().iter().enumerate() {
            chip_row(c, |row| {
                for (i, w) in words.iter().enumerate() {
                    chip_button(row, f, w, sb.chosen(sb.tool, r) == i, HAND_INK, ChoiceChip { row: r, index: i });
                }
            });
        }
        if sb.tool == Tool::Works && sb.chosen(Tool::Works, 0) == 0 && sb.site.is_some() {
            works_offered(c, f, sb);
        } else {
            c.spawn(text(f, &sb.hint(), 12.5, MUTED));
        }
    });
}

/// With Build a works in hand and a square taken: where it is, the works that
/// may stand there by their labels (the best expected return first, one
/// chosen), how many more are known and refused, the line for the one chosen,
/// and Build it here. "asking..." until the game's thread has said.
fn works_offered(c: &mut ChildSpawnerCommands, f: &Handle<Font>, sb: &Sandbox) {
    match &sb.offers {
        None => {
            chip_row(c, |row| {
                row.spawn(text(f, "asking...", 12.5, MUTED));
            });
        }
        Some(Err(why)) => {
            c.spawn(text(f, why, 12.5, INK));
        }
        Some(Ok(o)) => {
            c.spawn(text(f, &format!("On the square at {}:", o.place), 12.5, MUTED));
            if o.list.is_empty() {
                c.spawn(text(f, "No works the town knows has room and a market here, and should pay.", 12.5, INK));
            } else {
                chip_row(c, |row| {
                    for x in &o.list {
                        chip_button(row, f, &x.label, sb.method.as_ref() == Some(&x.name), HAND_INK, MethodChip(x.name.clone()));
                    }
                });
            }
            match o.more {
                0 => {}
                1 => {
                    c.spawn(text(f, "1 more is known here, with no room, no market or no profit for it.", 12.5, MUTED));
                }
                n => {
                    c.spawn(text(f, &format!("{n} more are known here, with no room, no market or no profit for them."), 12.5, MUTED));
                }
            }
            if let Some(line) = sb.offer_line() {
                c.spawn(text(f, &line, 12.5, INK));
                chip_row(c, |row| chip_button(row, f, "Build it here", false, HAND_INK, BuildHere));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tool_makes_the_edit_its_chips_say() {
        let mut sb = Sandbox { tool: Tool::Harvest, ..default() };
        // this district, a failure: a click needs a town
        sb.choice[Tool::Harvest.index()] = [0, 1, 0];
        assert_eq!(sb.edit(Some(4), Some(Pick::Cell(900))), Ok(Edit::Harvest { town: Some(4), factor: 0.7 }));
        assert!(sb.edit(None, None).is_err());
        assert!(sb.hint().contains("cut by three tenths"), "{}", sb.hint());
        // the whole island: any click will do
        sb.choice[Tool::Harvest.index()] = [1, 2, 0];
        assert_eq!(sb.edit(None, None), Ok(Edit::Harvest { town: None, factor: 1.1 }));
        sb.tool = Tool::Money;
        sb.choice[Tool::Money.index()] = [1, 1, 4];
        assert_eq!(sb.edit(Some(2), Some(Pick::Town(2))), Ok(Edit::Money { purse: Purse::Landlords, town: None, factor: 0.5 }));
        sb.tool = Tool::People;
        sb.choice[Tool::People.index()] = [3, 0, 0];
        assert_eq!(sb.edit(Some(2), Some(Pick::Firm(7))), Ok(Edit::People { town: 2, factor: 0.5 }));
        assert!(sb.edit(None, None).is_err());
        assert!(sb.hint().contains("inherit"));
        // an idea: nothing while the list is asked for, nor before one is chosen
        sb.tool = Tool::Idea;
        assert!(sb.edit(Some(3), Some(Pick::Town(3))).is_err());
        sb.keep_ideas(vec![("steel_rail".into(), "the steel rail".into()), ("dynamo".into(), "the dynamo".into())]);
        assert!(sb.edit(Some(3), Some(Pick::Town(3))).unwrap_err().starts_with("Choose an idea"));
        assert!(sb.hint().starts_with("Choose an idea"), "{}", sb.hint());
        // the steel rail chosen: a click on a town, or a square or works in its district, finds it there
        sb.idea = Some("steel_rail".into());
        assert_eq!(sb.edit(Some(3), Some(Pick::Cell(4000))), Ok(Edit::Discover { tech: "steel_rail".into(), town: 3 }));
        assert!(sb.edit(None, None).is_err());
        assert_eq!(sb.hint(), "Click a town, or anywhere in its district: the steel rail is found there, and spreads from it as any idea does.");
        // found, it leaves the list, and the choice is let go
        let v = sb.version;
        sb.keep_ideas(vec![("dynamo".into(), "the dynamo".into())]);
        assert_eq!((sb.idea.clone(), sb.version), (None, v + 1));
        sb.keep_ideas(vec![]);
        assert!(sb.hint().contains("nothing left to find"), "{}", sb.hint());
        // a town: a square makes the edit; a town and a works do not, and say so before anything is sent
        sb.tool = Tool::Town;
        assert_eq!(sb.edit(Some(1), Some(Pick::Cell(5123))), Ok(Edit::FoundTown { cell: 5123 }));
        assert_eq!(sb.edit(Some(1), Some(Pick::Town(1))), Err("Click open land, not a town."));
        assert_eq!(sb.edit(Some(1), Some(Pick::Firm(12))), Err("Click open land, not a works."));
        assert!(sb.edit(None, None).is_err());
        assert_eq!(Tool::Town.rows(), &[&["Found a town here"]]);
        assert_eq!(stands_from("By hand: a town to be founded at 31, 40, in Melbourne's district, at the end of 1791"), Some(1792));
        assert_eq!(stands_from("a town is already to be founded this year, at 31, 40"), None);
        // works (stage 7b). Build a works: a square takes the site and makes no edit, and the
        // list asked about it is "asking..." until it comes
        sb.tool = Tool::Works;
        sb.choice[Tool::Works.index()] = [0, 0, 0];
        assert_eq!(sb.click(Some(1), Some(Pick::Cell(2950))), Ok(None));
        assert_eq!((sb.site, sb.offers.clone()), (Some(2950), None));
        assert!(sb.build().is_err(), "nothing to build until the list has come");
        // the list comes: the allowed methods that should pay, best return first and chosen at
        // the start; the refused ones the town knows, and one that should lose money, are
        // counted; one it does not know is not
        let option = |name: &str, cost: f64, ret: f64, allowed: Result<(), String>, known: bool| isostate_core::SiteOption {
            method: 0,
            name: name.into(),
            label: format!("{}{}", name[..1].to_uppercase(), &name[1..]),
            group: None,
            sums: if known { Ok(sums(cost, ret)) } else { Err("not known".into()) },
            allowed,
        };
        let hand = Ok(vec![
            option("brickworks", 622.0, 0.42, Ok(()), true),
            option("brewery", 16999.0, 0.19, Ok(()), true),
            option("ironworks", 9000.0, 0.30, Err("no room".into()), true),
            option("smithy", 1472.0, -0.39, Ok(()), true),
            option("dynamo", 1.0, 0.9, Err("not known in Melbourne yet".into()), false),
        ]);
        assert!(sb.keep_offers(Offers::of(&hand, "31, 40, in Melbourne's district".into())));
        assert!(!sb.keep_offers(Offers::of(&hand, "31, 40, in Melbourne's district".into())), "the same list is not drawn again");
        let o = sb.offers.clone().unwrap().unwrap();
        assert_eq!((o.list.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(), o.more), (vec!["brickworks", "brewery"], 2));
        assert_eq!(sb.method.as_deref(), Some("brickworks"));
        sb.turn = Some(1792);
        assert_eq!(sb.offer_line().unwrap(), "Brickworks here: it costs 622, paid by hand, and should return 42% a year. It is built at the year's turn, in January 1792.");
        // Build it here makes BuildWorks with the method chosen, on the square taken
        assert_eq!(sb.build(), Ok(Edit::BuildWorks { method: "brickworks".into(), cell: 2950 }));
        sb.method = Some("brewery".into());
        assert_eq!(sb.build(), Ok(Edit::BuildWorks { method: "brewery".into(), cell: 2950 }));
        // a method no longer offered is not built
        sb.method = Some("ironworks".into());
        assert!(sb.build().is_err());
        // no works can stand on the square: its sentence
        sb.keep_offers(Offers::of(&Err("no works can stand on the square at 0, 0: it is sea".into()), "0, 0".into()));
        assert_eq!(sb.offers, Some(Err("No works can stand on the square at 0, 0: it is sea".into())));
        // a town and a works are refused with Build, before anything is sent, and the square is let go
        assert_eq!(sb.click(Some(1), Some(Pick::Town(1))), Err("Click a square of land, not a town."));
        assert_eq!(sb.click(Some(1), Some(Pick::Firm(12))), Err("Click a square of land, not a works."));
        assert_eq!((sb.site, sb.tool), (None, Tool::Works));
        // Close a works: a square is refused, a works is closed, and the tool is put down
        sb.choice[Tool::Works.index()] = [1, 0, 0];
        assert!(sb.hint().starts_with("Click a works"), "{}", sb.hint());
        assert_eq!(sb.click(Some(1), Some(Pick::Cell(2950))), Err("Click a works."));
        assert_eq!(sb.click(Some(1), Some(Pick::Town(1))), Err("Click a works."));
        assert_eq!(sb.tool, Tool::Works);
        assert_eq!(sb.click(Some(1), Some(Pick::Firm(12))), Ok(Some(Edit::CloseWorks { works: 12 })));
        assert_eq!(sb.tool, Tool::Look);
        assert_eq!(Tool::Works.rows(), &[&["Build a works", "Close a works", "Found a university", "Close a university"]]);
        // a university founded on a square and closed in a district, the tool put down after each
        sb.tool = Tool::Works;
        sb.choice[Tool::Works.index()] = [2, 0, 0];
        assert_eq!(sb.click(Some(1), Some(Pick::Town(1))), Err("Click a square of land in the town's district."));
        assert_eq!(sb.click(Some(1), Some(Pick::Cell(2950))), Ok(Some(Edit::FoundUniversity { cell: 2950 })));
        assert_eq!(sb.tool, Tool::Look);
        sb.tool = Tool::Works;
        sb.choice[Tool::Works.index()] = [3, 0, 0];
        assert_eq!(sb.click(None, Some(Pick::Cell(2950))), Err("Click in the district of the town whose university is to close."));
        assert_eq!(sb.click(Some(4), Some(Pick::Town(4))), Ok(Some(Edit::CloseUniversity { town: 4 })));
        assert_eq!(sb.tool, Tool::Look);
        // the words scripts click by
        assert_eq!(Tool::ALL.map(Tool::word), ["Look", "Money", "People", "Harvest", "Idea", "Town", "Works"]);
        // every row's chips have a factor and a word
        assert_eq!(Tool::Money.rows()[2].len(), MONEY_FACTORS.len());
        assert_eq!(Tool::People.rows()[0].len(), PEOPLE_FACTORS.len());
        assert_eq!(Tool::Harvest.rows()[1].len(), HARVEST_FACTORS.len());
    }

    /// A site's sums with nothing in them but a cost and an expected return.
    fn sums(capital_cost: f64, expected_return: f64) -> isostate_core::SiteSums {
        isostate_core::SiteSums {
            town: 0, K: 1.0, gate_price: 0.0, inputs: Vec::new(), input_cost: 0.0, heat: 0.0, heat_price: 0.0, heat_cost: 0.0, labor: 0.0, wage: 0.0, labor_cost: 0.0,
            margin: 0.0, rent_bid: 0.0, capital_cost, expected_return, bonus: 0.0, agglo: 0.0, output: 0.0, town_demand: 0.0, town_output: 0.0, market_share: 0.0, price_ratio: 0.0,
        }
    }

    fn stopped_clock() -> Timeline {
        Timeline { t: 37.5, built: None, playing: true, speed: 1.0 }
    }

    #[test]
    fn a_run_on_from_a_chip_from_the_timeline_or_from_a_script_goes_to_the_same_january() {
        // an island begun in 1750, at January 1791 (the clock at 40), playing on to 2050
        let at = Present { latest: 40, months: 0, last: 299 };
        // the chip of ten years
        let (mut by_chip, mut tl) = (Sandbox::default(), stopped_clock());
        assert!(by_chip.run_to(&mut tl, at, at.latest + 10, Some(10), 1.0));
        // the clock went to the present and stopped
        assert_eq!((tl.t, tl.playing), (40.0, false));
        // the timeline pressed where January 1801 stands on it
        let x = 50.0 / 299.0 - 0.5;
        let (mut by_press, mut tl) = (Sandbox::default(), stopped_clock());
        assert!(by_press.run_to(&mut tl, at, crate::ui::timeline_year(x, 299), None, 1.0));
        // and goto:1801
        let (mut by_script, mut tl) = (Sandbox::default(), stopped_clock());
        assert!(by_script.run_to(&mut tl, at, clock::january(1750, 1801), None, 1.0));
        assert_eq!((by_chip.target(), by_press.target(), by_script.target()), (Some(50), Some(50), Some(50)));
        // the chip stays inked while its run on lasts; the others ink none
        assert_eq!((by_chip.run.unwrap().chip, by_press.run.unwrap().chip), (Some(10), None));
        // a press ahead while the run on is under way moves its year and lets go of the chip,
        // and the run on goes on from where it was, the clock where the years have brought it
        let mut tl = Timeline { t: 43.0, built: None, playing: false, speed: 1.0 };
        let v = by_chip.version;
        assert!(by_chip.run_to(&mut tl, Present { latest: 43, ..at }, 70, None, 2.0));
        let run = by_chip.run.unwrap();
        assert_eq!((run.to, run.chip, run.from, run.began, tl.t), (70, None, 40, 1.0, 43.0));
        assert_eq!(by_chip.version, v + 1);
        // the same press held for another frame changes nothing, and the tray is not drawn again
        assert!(by_chip.run_to(&mut tl, Present { latest: 43, ..at }, 70, None, 2.1));
        assert_eq!(by_chip.version, v + 1);
    }

    #[test]
    fn a_run_on_to_the_present_or_behind_it_starts_nothing_and_one_past_the_end_stops_there() {
        // in April 1791: January 1791 is behind the clock, and so is 1785
        let at = Present { latest: 40, months: 3, last: 299 };
        for to in [40, 34, 0] {
            let (mut sb, mut tl) = (Sandbox::default(), stopped_clock());
            assert!(!sb.run_to(&mut tl, at, to, None, 1.0), "{to}");
            assert!(sb.target().is_none() && sb.version == 0);
            // and the clock is left alone
            assert_eq!((tl.t, tl.playing), (37.5, true));
        }
        // January 1792 is ahead
        let (mut sb, mut tl) = (Sandbox::default(), stopped_clock());
        assert!(sb.run_to(&mut tl, at, 41, None, 1.0));
        assert!((tl.t - clock::at(40, 3)).abs() < 1e-6, "the clock goes to April 1791, the present: {}", tl.t);
        // twenty-five years from 2041 stop at the run's last year
        let near_the_end = Present { latest: 290, months: 0, last: 299 };
        let (mut sb, mut tl) = (Sandbox::default(), stopped_clock());
        assert!(sb.run_to(&mut tl, near_the_end, 290 + 25, Some(25), 1.0));
        assert_eq!(sb.target(), Some(299));
        // and with the last year played there is nothing to run on to
        let over = Present { latest: 299, months: 0, last: 299 };
        let (mut sb, mut tl) = (Sandbox::default(), stopped_clock());
        assert!(!sb.run_to(&mut tl, over, 310, Some(10), 1.0));
    }

    #[test]
    fn the_tray_holds_a_dozen_tools_in_two_rows() {
        // the tools' chips with their icons were laid out 64 to 81 px wide, 74 on the mean
        // (a scripted run's print): six of the mean to a row of the widest card, five of the widest
        assert_eq!((chips_a_row(74.0), chips_a_row(81.0)), (6, 5));
        assert_eq!(chips_a_row(500.0), 0);
    }
}
