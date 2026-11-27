//! Saved games, as a log of orders. Not in the Python.
//!
//! The state of a run cannot be written out as it stands (`State`, the random
//! streams and the observer have no serialised form), but a run is decided by
//! its seed, its rules and the orders the player gave. So a save holds those
//! and nothing else, and loading plays the months again, carrying each order
//! out at the point it was first carried out.
//!
//! A game is played a month at a time (`Game::month`), and the player may give
//! an order in any month, but nothing the player orders touches the island
//! between a year's first month and its last. An order given at the year's
//! turn, before the first month is played, is carried out at once, as it
//! always was. One given later waits (`Game::waiting`): what the family is told
//! about its decisions, its people's posts, its branches and its seat is
//! carried out when December has been played, just before the year's books are
//! closed, which is when the core gives such things their effect; works to
//! build and land to buy are carried out at the turn, after the books are
//! closed, exactly where they were when a game was played a year at a time.
//!
//! ```no_run
//! use isostate_core::save::{self, Game, Order};
//! let mut g = Game::new(7, 6, 0);
//! g.year().unwrap();
//! let _ = g.give(Order::MoveSeat { town: 2 });
//! let text = g.save(0).to_text();
//! let again = save::load(&save::Save::from_text(&text).unwrap(), |_| {}).unwrap();
//! assert!(!again.drifted && again.refused.is_empty());
//! ```
//!
//! A save is only as good as the rules it was made under: when the simulation
//! has changed since, the years play out differently. Orders it now refuses
//! are skipped and counted (`Loaded::refused`), and `Loaded::drifted` says
//! whether the island stands as the save says it stood.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::families::{self, Choice, Group, Posting, Stance};
use crate::params::{Params, Rules};
use crate::sim::{self, Sim};
use crate::state::{Event, State};

/// The layout of the file. A reader refuses a later one. Layout 2 added the
/// months played of the year in progress, the orders waiting for the year's
/// end, and the month an order was carried out in; a file in layout 1 has none
/// of them and reads as a game saved at a year's turn. Layout 3 (19 November
/// 2026, when the game's rules took the new land) writes in every save which
/// land the game is on (`Save::land`); a file in layout 1 or 2 without the
/// line was made on the port's island and is played there again.
pub const FORMAT: u32 = 3;

/// The player's family is always the first.
pub const PLAYER: usize = 0;

/// The most families a game may have. Every family then has a surname of its
/// own (`names::FAMILY_SURNAMES`; past that they are told apart by a numeral),
/// and since each begins with three hundredths of the island's rent, thirty
/// begin with nine tenths of it and no more could be given their share.
pub const MAX_FAMILIES: usize = crate::names::FAMILY_SURNAMES.len();

/// Whether a game may have this many families: at least the player's, and
/// no more than `MAX_FAMILIES`. Said as a clause, for a sentence that goes on
/// to say where the number came from.
pub fn families_allowed(n: usize) -> Result<(), String> {
    if (1..=MAX_FAMILIES).contains(&n) {
        Ok(())
    } else {
        Err(format!("a game has from 1 to {MAX_FAMILIES} families, not {n}"))
    }
}

/// The last year a game may be played to (docs/rules/longrun.md, "A game past
/// 1900"). A game's last year is chosen when it is begun and kept in its save.
pub const LATEST_YEAR: i64 = 2050;

/// The last year of the island's content under these rules: 1900. A game that
/// says nothing else ends there, as every game did before the run to 2050.
pub fn content_last_year(rules: Rules) -> i64 {
    rules.content().end_year
}

/// Whether a game under these rules may be played to the year `to`: from the
/// content's last year to `LATEST_YEAR`, and past the content's last year only
/// under rules with the long run on (`longrun.enabled`), whose bounds on prices
/// and the wage ride on the island's own cost of living and so can last.
/// Said as a clause, as `families_allowed` is.
pub fn last_year_allowed(rules: Rules, to: i64) -> Result<(), String> {
    let first = content_last_year(rules);
    if to < first || to > LATEST_YEAR {
        return Err(format!("a game is played to a year from {first} to {LATEST_YEAR}, not {to}"));
    }
    if to > first && !rules.params().longrun.enabled {
        return Err(format!(
            "a game under the {} rules ends in {first}, because the long run (docs/rules/longrun.md) is off under them, and so it cannot be played to {to}",
            rules.as_str()
        ));
    }
    Ok(())
}

/// Something the player told the family to do. Everything that changes the
/// simulation from outside is one of these.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Order {
    /// Build a works on a cell. The method goes by its name, so a save
    /// survives methods being added to the content.
    Found { method: String, cell: usize },
    BuyLand { cell: usize },
    /// Answer a decision. What was chosen is kept, not its place in the list,
    /// so a list that has changed since cannot turn it into another answer.
    Decide { id: u64, choice: Choice },
    Post { member: usize, posting: Posting },
    Stance { group: Group, town: usize, stance: Stance },
    MoveSeat { town: usize },
    /// An edit by hand (docs/designs/sandbox.md): not word to the family but a
    /// change to the island, carried out at once in any month and logged with
    /// the month it was made in; a works built or closed by hand waits for the
    /// year's turn and is logged at month 0. The edit is a field of its own, so a line of
    /// the log reads `{"year":1820,"month":3,"kind":"by_hand","edit":{"what":"people","town":4,"factor":0.5}}`.
    ByHand { edit: crate::edits::Edit },
}

impl Order {
    /// The order that answers decision `id` with its option `option`, as the
    /// lists stand now.
    pub fn decide(sim: &Sim, id: u64, option: usize) -> Result<Order, String> {
        let pending = families::pending(&sim.st, PLAYER);
        let Some(d) = pending.iter().find(|d| d.id == id) else {
            return Err(format!("there is no decision {id} waiting"));
        };
        let Some(o) = d.options.get(option) else {
            return Err(format!("decision {id} has {} options", d.options.len()));
        };
        Ok(Order::Decide { id, choice: o.choice })
    }

    /// The order to build method `method` (an index into the content) on `cell`.
    pub fn found(sim: &Sim, method: usize, cell: usize) -> Result<Order, String> {
        match sim.content.method_name.get(method) {
            Some(name) => Ok(Order::Found { method: name.clone(), cell }),
            None => Err(format!("there is no method {method}")),
        }
    }

    /// Whether the order is word to the family about its own affairs (a
    /// decision answered, a posting, what a branch does, where the seat is),
    /// which the core gives its effect at the year's end. Given in the middle
    /// of a year, such an order is carried out when December has been played,
    /// before the books are closed. The others (a works to build, land to buy)
    /// change the island at once, and wait for the turn itself. An edit by
    /// hand is named here so that it is never taken for word to the family:
    /// most are carried out at once (`Game::give`), and those that wait
    /// (`Edit::at_turn`, a works built or closed) wait for the turn itself.
    pub fn for_year_end(&self) -> bool {
        !matches!(self, Order::Found { .. } | Order::BuyLand { .. } | Order::ByHand { .. })
    }
}

/// What an order that was carried out did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Done {
    /// The new works' firm id.
    Founded(usize),
    /// The price paid for the land.
    Bought(f64),
    Done,
    /// Given in the middle of a year: it waits for the year's end, and could
    /// be carried out as things stand today.
    Waits,
}

/// Carry out an order for the player's family. A refused order changes nothing.
pub fn carry_out(sim: &mut Sim, order: &Order) -> Result<Done, String> {
    // an edit by hand is carried out on the run itself; its line is the title
    // of the news it leaves (`edits::carry_out`)
    if let Order::ByHand { edit } = order {
        return crate::edits::carry_out(sim, edit).map(|_| Done::Done);
    }
    let Sim { st, content: C, params: P, events, year, .. } = sim;
    carry_out_on(st, C, P, *year, events, order)
}

/// `carry_out` on a state that need not be the run's own: an order that waits
/// is first tried on a copy. `year` is the next year to be played.
fn carry_out_on(st: &mut State, C: &Content, P: &Params, year: i64, events: &mut Vec<Event>, order: &Order) -> Result<Done, String> {
    let year = &year;
    match order {
        Order::Found { method, cell } => {
            let Some(m) = C.method_name.iter().position(|n| n == method) else {
                return Err(format!("there is no method called {method}"));
            };
            families::found_works(st, C, P, PLAYER, m, *cell, *year, events).map(Done::Founded)
        }
        Order::BuyLand { cell } => families::buy_land(st, C, P, PLAYER, *cell).map(Done::Bought),
        Order::Decide { id, choice } => {
            let option = match st.families.get(PLAYER).and_then(|f| f.decisions.iter().find(|d| d.id == *id)) {
                Some(d) => d.options.iter().position(|o| o.choice == *choice),
                None => return Err(format!("there is no decision {id} waiting")),
            };
            match option {
                Some(o) => families::decide(st, C, P, PLAYER, *id, o, events).map(|()| Done::Done),
                None => Err(format!("decision {id} no longer offers that choice")),
            }
        }
        Order::Post { member, posting } => families::post_member(st, PLAYER, *member, *posting).map(|()| Done::Done),
        Order::Stance { group, town, stance } => families::set_stance(st, PLAYER, *group, *town, *stance).map(|()| Done::Done),
        Order::MoveSeat { town } => families::move_seat(st, PLAYER, *town).map(|()| Done::Done),
        // the copy of the island never sees an edit: it is carried out on the run itself
        Order::ByHand { .. } => Err("an edit by hand is carried out on the island itself, never tried on a copy".to_string()),
    }
}

/// An order and when it was carried out: the last year played at the time,
/// and how many months of the year after it had been played. That is none for
/// an order carried out at a year's turn, before the next year was worked out
/// (every order of a game played a year at a time), and twelve for word to the
/// family that was given in the middle of a year and carried out at its end,
/// before its books were closed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Given {
    pub year: i64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub month: usize,
    #[serde(flatten)]
    pub order: Order,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

/// What became of an order that waited for the year's end.
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub order: Order,
    pub result: Result<Done, String>,
}

/// Which rules a game is played under, as a save names them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RulesId {
    /// "game" or "port" (`params::Rules`).
    pub name: String,
    /// A fingerprint of every number of the rules (the parameters and the
    /// content), so a mechanism added or retuned since the save shows. It does
    /// not see changes to the code; `Save::check` does.
    pub fingerprint: String,
}

/// The rules as a save names them, with their numbers as they are today.
pub fn rules_id(rules: Rules) -> RulesId {
    let mut h = Fnv::new();
    h.bytes(serde_json::to_string(&rules.params()).unwrap_or_default().as_bytes());
    // the content has no serialised form, but it prints; its one table with
    // no order of its own (the goods by name) is left out
    // tree: with the overlay the rules lay over it, so a change to data/game.toml shows
    let mut content = crate::tree::content(&rules.params(), rules.content()).unwrap_or_else(|_| rules.content());
    content.good_index.clear();
    h.bytes(format!("{content:?}").as_bytes());
    RulesId { name: rules.as_str().to_string(), fingerprint: h.hex() }
}

/// The parameters of a game on island `seed` with `families` families.
pub fn params(rules: Rules, seed: u64, families: usize) -> Params {
    let mut P = rules.params();
    P.seed = seed;
    P.n_families = families.max(1);
    // longrun (docs/rules/longrun.md): the stand-in for growth is for sweeps,
    // and a game is never played under it
    P.longrun.standin_growth = false;
    P
}

/// The parameters of a game played to the year `to` (`last_year_allowed`).
/// The last year is set in `end_year` only when it is not the content's, so a
/// game to 1900 begun this way has the very parameters `params` gives.
pub fn params_to(rules: Rules, seed: u64, families: usize, to: i64) -> Result<Params, String> {
    last_year_allowed(rules, to)?;
    let mut P = params(rules, seed, families);
    if to != content_last_year(rules) {
        P.end_year = Some(to);
    }
    Ok(P)
}

/// A game being played: the simulation, and every order carried out so far.
#[derive(Clone, Debug)]
pub struct Game {
    pub sim: Sim,
    /// The rules it is played under: the game's, unless a save says the port's.
    pub rules: Rules,
    pub orders: Vec<Given>,
    /// When the game began, in seconds since 1970. It tells one game on an
    /// island from another, so a new game does not write over an old one unseen.
    pub started: u64,
    /// Months of the year in progress (`sim.year`) that have been played, 0 to
    /// 11: none at a year's turn.
    pub month: usize,
    /// Orders given since the year's first month was played, in the order
    /// they were given. They wait for the year's end (see the top of this file).
    pub waiting: Vec<Order>,
    /// What became of the orders carried out at the last year's end, until
    /// `turn` hands it over.
    outcomes: Vec<Outcome>,
    /// The start's two switches, where the game was begun with either set
    /// otherwise than its start sets it (`Game::begin`); kept in its save.
    pub begun: Switches,
}

/// The start's own two switches (docs/rules/landfall.md, "The start's
/// switches"): settlers by ship (`immigration.enabled`) and ideas by sea
/// (`abroad.ideas`), each on at landfall and off on the settled island. A game
/// may be begun with either the other way; None is the start's own. A switch
/// thrown in the middle of a game is an edit like any other (`Edit::Rule`),
/// kept in the game's log: these say only how the game began.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Switches {
    pub settlers: Option<bool>,
    pub ideas_by_sea: Option<bool>,
}

impl Switches {
    /// Both as the start has them.
    pub fn plain(&self) -> bool {
        self.settlers.is_none() && self.ideas_by_sea.is_none()
    }

    /// `P`, a start already made, with these switches set, and the switches
    /// as they then differ from the start's own: one asked for as the start
    /// already has it is the start's own, None. Only the switch is thrown, as
    /// the Rules tab throws it; the rule's numbers are the start's.
    pub fn set(self, P: &mut Params) -> Switches {
        let settlers = self.settlers.filter(|&on| on != P.immigration.enabled);
        if let Some(on) = settlers {
            P.immigration.enabled = on;
        }
        let ideas_by_sea = self.ideas_by_sea.filter(|&on| on != P.abroad.ideas);
        if let Some(on) = ideas_by_sea {
            P.abroad.ideas = on;
        }
        Switches { settlers, ideas_by_sea }
    }
}

impl Game {
    /// A new game under the game's rules, before its first year.
    pub fn new(seed: u64, families: usize, started: u64) -> Game {
        Game::under(Rules::Game, seed, families, started)
    }

    /// A new game under these rules.
    pub fn under(rules: Rules, seed: u64, families: usize, started: u64) -> Game {
        Game { sim: sim::start(&params(rules, seed, families), rules.content()), rules, orders: Vec::new(), started, month: 0, waiting: Vec::new(), outcomes: Vec::new(), begun: Switches::default() }
    }

    /// A new game under these rules on the second island generator's land
    /// (docs/rules/land.md), whatever the rules say of it: the rules as they
    /// stand, with the land switched on and its numbers the game's. The game's
    /// rules take the land themselves since 19 November 2026, so under them
    /// this is `under`; under the port's it puts a game on the new land. A
    /// save says which land its game is on (`Save::land`) and is played again
    /// on the same land.
    pub fn on_new_land(rules: Rules, seed: u64, families: usize, started: u64) -> Game {
        let mut P = params(rules, seed, families);
        P.land = crate::landform::LandParams::game();
        Game { sim: sim::start(&P, rules.content()), rules, orders: Vec::new(), started, month: 0, waiting: Vec::new(), outcomes: Vec::new(), begun: Switches::default() }
    }

    /// A new game under these rules on the port's island, whatever the rules
    /// say of the land: the rules as they stand, with the land switched off.
    /// It is the way back to the island every game was played on before 19
    /// November 2026 (the viewer's `--old-land`), and it is where a save that
    /// does not say its land is played again.
    pub fn on_old_land(rules: Rules, seed: u64, families: usize, started: u64) -> Game {
        let mut P = params(rules, seed, families);
        P.land = crate::landform::LandParams::default();
        Game { sim: sim::start(&P, rules.content()), rules, orders: Vec::new(), started, month: 0, waiting: Vec::new(), outcomes: Vec::new(), begun: Switches::default() }
    }

    /// A new game under these rules, played to the year `to` (1900 to 2050;
    /// past 1900 only under rules with the long run on: `last_year_allowed`).
    /// Its save keeps the year (`Save::to`), and loading plays it to the same.
    pub fn under_to(rules: Rules, seed: u64, families: usize, started: u64, to: i64) -> Result<Game, String> {
        let P = params_to(rules, seed, families, to)?;
        Ok(Game { sim: sim::start(&P, rules.content()), rules, orders: Vec::new(), started, month: 0, waiting: Vec::new(), outcomes: Vec::new(), begun: Switches::default() })
    }

    /// `on_new_land`, played to the year `to` (as `under_to`).
    pub fn on_new_land_to(rules: Rules, seed: u64, families: usize, started: u64, to: i64) -> Result<Game, String> {
        let mut P = params_to(rules, seed, families, to)?;
        P.land = crate::landform::LandParams::game();
        Ok(Game { sim: sim::start(&P, rules.content()), rules, orders: Vec::new(), started, month: 0, waiting: Vec::new(), outcomes: Vec::new(), begun: Switches::default() })
    }

    /// A new game begun at landfall, one town in a corner with the rules that
    /// come with it (docs/rules/landfall.md), on the land `land` says, played to
    /// the year `to` (as `under_to`). The viewer's `--landfall`.
    pub fn landfall_to(rules: Rules, seed: u64, families: usize, started: u64, to: i64, land: bool) -> Result<Game, String> {
        let mut P = params_to(rules, seed, families, to)?;
        P.land = if land { crate::landform::LandParams::game() } else { crate::landform::LandParams::default() };
        crate::landfall::landfall(&mut P);
        Ok(Game { sim: sim::start(&P, rules.content()), rules, orders: Vec::new(), started, month: 0, waiting: Vec::new(), outcomes: Vec::new(), begun: Switches::default() })
    }

    /// A new game played to the year `to`, on the new land or the old
    /// (`land`), from landfall or the settled island (`landfall`), with the
    /// start's two switches as `switches` asks (`Switches::set`). With both
    /// the start's own it is `on_new_land_to`, `on_old_land_to` or
    /// `landfall_to`, to the bit; it is how a save is played again, and the
    /// viewer begins every new game with it.
    #[allow(clippy::too_many_arguments)]
    pub fn begin(rules: Rules, seed: u64, families: usize, started: u64, to: i64, land: bool, landfall: bool, switches: Switches) -> Result<Game, String> {
        let mut P = params_to(rules, seed, families, to)?;
        P.land = if land { crate::landform::LandParams::game() } else { crate::landform::LandParams::default() };
        if landfall {
            crate::landfall::landfall(&mut P);
        }
        let begun = switches.set(&mut P);
        Ok(Game { sim: sim::start(&P, rules.content()), rules, orders: Vec::new(), started, month: 0, waiting: Vec::new(), outcomes: Vec::new(), begun })
    }

    /// Whether the game began at landfall (docs/rules/landfall.md).
    pub fn at_landfall(&self) -> bool {
        self.sim.params.start.landfall
    }

    /// `on_old_land`, played to the year `to` (as `under_to`).
    pub fn on_old_land_to(rules: Rules, seed: u64, families: usize, started: u64, to: i64) -> Result<Game, String> {
        let mut P = params_to(rules, seed, families, to)?;
        P.land = crate::landform::LandParams::default();
        Ok(Game { sim: sim::start(&P, rules.content()), rules, orders: Vec::new(), started, month: 0, waiting: Vec::new(), outcomes: Vec::new(), begun: Switches::default() })
    }

    /// The last year the game is played to: the content's (1900) unless it was
    /// begun with another (`under_to`).
    pub fn to(&self) -> i64 {
        self.sim.end_year
    }

    /// Whether the game is played on the second island generator's land.
    pub fn on_land(&self) -> bool {
        self.sim.params.land.enabled
    }

    /// The last year played (the year before the first, if none has been).
    pub fn last_year(&self) -> i64 {
        self.sim.year - 1
    }

    /// Whether every year has been played.
    pub fn over(&self) -> bool {
        self.sim.year > self.sim.end_year
    }

    /// Whether the game stands at a year's turn: no month of the next year
    /// has been played, so an order given now is carried out at once.
    pub fn at_turn(&self) -> bool {
        self.month == 0
    }

    /// Play the next month. After December the year ends: word to the family
    /// that waited is carried out, the books are closed, and this says true.
    /// The works and land that waited are carried out by `turn`, which a
    /// caller that wants to see the year as it ended calls afterwards (the
    /// next `month` or `give` does it otherwise).
    pub fn month(&mut self) -> Result<bool, String> {
        self.settle();
        self.play_month()?;
        if self.month < 12 {
            return Ok(false);
        }
        let (now, later): (Vec<Order>, Vec<Order>) = std::mem::take(&mut self.waiting).into_iter().partition(|o| o.for_year_end());
        for order in now {
            self.carry(order);
        }
        self.waiting = later;
        self.close_year();
        Ok(true)
    }

    /// One month of the simulation and nothing else.
    fn play_month(&mut self) -> Result<(), String> {
        sim::run_month(&mut self.sim, self.month)?;
        self.month += 1;
        Ok(())
    }

    /// The year's end, when its twelve months have been played.
    fn close_year(&mut self) {
        sim::end_year(&mut self.sim);
        self.month = 0;
    }

    /// Carry out an order that waited, and write it in the log if it was.
    fn carry(&mut self, order: Order) {
        let result = carry_out(&mut self.sim, &order);
        if result.is_ok() {
            self.orders.push(Given { year: self.last_year(), month: self.month, order: order.clone() });
        }
        self.outcomes.push(Outcome { order, result });
    }

    /// At a year's turn, whatever still waits is carried out.
    fn settle(&mut self) {
        if self.month == 0 {
            for order in std::mem::take(&mut self.waiting) {
                self.carry(order);
            }
        }
    }

    /// At a year's turn: carry out the works and land that waited for it, and
    /// say what became of every order that waited for this year's end, in the
    /// order they were carried out.
    pub fn turn(&mut self) -> Vec<Outcome> {
        self.settle();
        std::mem::take(&mut self.outcomes)
    }

    /// Play on to the next year's turn (the whole year, from a turn). As
    /// `month` does, it leaves the works and land that waited to `turn`.
    pub fn year(&mut self) -> Result<(), String> {
        while !self.month()? {}
        Ok(())
    }

    /// The island as it would stand if the waiting orders were carried out
    /// today, one after another; those that could not be are passed over.
    /// An edit by hand that waits is passed over too (`carry_out_on` never
    /// carries one out on a copy), so a family's order is tried as it would
    /// have been without it.
    fn with_waiting(&self) -> State {
        let Sim { content: C, params: P, year, .. } = &self.sim;
        let mut st = self.sim.st.clone();
        for order in &self.waiting {
            let _ = carry_out_on(&mut st, C, P, year + 1, &mut Vec::new(), order);
        }
        st
    }

    /// The player's family as it would stand if the waiting orders were
    /// carried out today (None when nothing waits): what the family has been
    /// told, for an interface to show before it has happened. It copies the
    /// island to find out, so it is for asking after an order, not every month.
    pub fn foreseen(&self) -> Option<families::FamilyReport> {
        if self.waiting.is_empty() {
            return None;
        }
        families::family_report(&self.with_waiting(), &self.sim.content, &self.sim.params, PLAYER).ok()
    }

    /// Give an order. At a year's turn it is carried out at once and written
    /// in the log. In the middle of a year it is tried on a copy of the island
    /// with the orders already waiting, and if it could be carried out today
    /// it waits with them for the year's end (`Done::Waits`). A refused order
    /// is not kept, and changes nothing. An edit by hand (`Order::ByHand`) is
    /// carried out at once in any month, after whatever waited for a turn it
    /// is given at, and logged with the months played so far, so that `load`
    /// carries it out again before the same month. An edit of the kind that
    /// waits (`Edit::at_turn`: a works built or closed) is carried out at once
    /// only at a turn: given in the middle of a year it is checked on the run
    /// itself (`edits::check`, which only reads; the island is never copied
    /// for an edit) and waits, to be carried out at the next turn after the
    /// orders given before it, and logged at month 0.
    pub fn give(&mut self, order: Order) -> Result<Done, String> {
        let at_once = match &order {
            Order::ByHand { edit } => !edit.at_turn(),
            _ => false,
        };
        if self.month == 0 || at_once {
            self.settle();
            let done = carry_out(&mut self.sim, &order)?;
            self.orders.push(Given { year: self.last_year(), month: self.month, order });
            return Ok(done);
        }
        if let Order::ByHand { edit } = &order {
            crate::edits::check(&self.sim, edit)?;
            self.waiting.push(order);
            return Ok(Done::Waits);
        }
        let mut st = self.with_waiting();
        carry_out_on(&mut st, &self.sim.content, &self.sim.params, self.sim.year + 1, &mut Vec::new(), &order)?;
        self.waiting.push(order);
        Ok(Done::Waits)
    }

    /// Take back the `i`-th of the orders that wait.
    pub fn cancel(&mut self, i: usize) -> Result<Order, String> {
        if i >= self.waiting.len() {
            return Err(format!("there is no waiting order {i}"));
        }
        Ok(self.waiting.remove(i))
    }

    /// The game as a save, written at `now` (seconds since 1970).
    pub fn save(&self, now: u64) -> Save {
        let me = families::family_report(&self.sim.st, &self.sim.content, &self.sim.params, PLAYER).ok();
        Save {
            format: FORMAT,
            version: env!("CARGO_PKG_VERSION").to_string(),
            rules: rules_id(self.rules),
            seed: self.sim.params.seed,
            families: self.sim.params.n_families,
            started: self.started,
            saved: utc(now),
            year: self.last_year(),
            month: self.month,
            family: me.as_ref().map(|r| r.surname.clone()).unwrap_or_default(),
            seat: me.as_ref().map(|r| r.seat_name.clone()).unwrap_or_default(),
            worth: me.as_ref().map(|r| r.net_worth.round() as i64).unwrap_or(0),
            check: check(&self.sim),
            land: self.on_land(),
            landfall: self.sim.params.start.landfall,
            settlers: self.begun.settlers,
            ideas_by_sea: self.begun.ideas_by_sea,
            to: (self.to() != content_last_year(self.rules)).then_some(self.to()),
            orders: self.orders.clone(),
            waiting: self.waiting.clone(),
        }
    }
}

/// A saved game, as it is written to a file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Save {
    pub format: u32,
    /// The version of this crate that wrote it.
    pub version: String,
    pub rules: RulesId,
    pub seed: u64,
    pub families: usize,
    pub started: u64,
    /// When it was written, for whoever opens the file.
    #[serde(default)]
    pub saved: String,
    /// The last year played.
    pub year: i64,
    /// Months of the year after it that had been played, 0 to 11 (layout 2).
    #[serde(default)]
    pub month: usize,
    /// The player's family, its seat and its net worth then. Only for people
    /// (a card that offers to go on, someone reading the file): loading works
    /// them out again.
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub seat: String,
    #[serde(default)]
    pub worth: i64,
    /// A fingerprint of the island as it stood (`check`).
    #[serde(default)]
    pub check: String,
    /// Whether the game is on the second island generator's land
    /// (docs/rules/land.md), which the game's rules take since 19 November
    /// 2026, or on the port's island; it is played again on the same. Written
    /// in every save from layout 3, `"land": true` or `"land": false`. A save
    /// without the line is in layout 1 or 2, made before the switch, and so on
    /// the port's island: before the switch only a game begun on the new land
    /// (`Game::on_new_land`, the viewer's `--new-land`) wrote the line.
    #[serde(default)]
    pub land: bool,
    /// Whether the game began at landfall, one town in a corner, and not on the
    /// settled island (docs/rules/landfall.md); it is played again from the
    /// same start. Left out of the file for the settled island, so a save of
    /// it is written as it always was.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub landfall: bool,
    /// Settlers by ship (`immigration.enabled`) as the game began, where it
    /// began otherwise than its start has them (`Switches`): `false` for a
    /// colony begun with no ships. Left out of the file otherwise, so every
    /// save made before it, and every save of a start as it comes, is written
    /// and played as it always was. A switch thrown later is in the orders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlers: Option<bool>,
    /// Ideas by sea (`abroad.ideas`) as the game began, as `settlers` is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ideas_by_sea: Option<bool>,
    /// The last year the game is played to, when it is not the content's
    /// (`Game::under_to`; docs/rules/longrun.md, "A game past 1900"). Left
    /// out of the file otherwise, so a game to 1900 is written as it always
    /// was, and a save that does not say is a game to 1900.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<i64>,
    pub orders: Vec<Given>,
    /// Orders given since the year's turn, waiting for the year's end (layout 2).
    #[serde(default)]
    pub waiting: Vec<Order>,
}

impl Save {
    /// JSON with a line for each field and a line for each order.
    pub fn to_text(&self) -> String {
        let value = serde_json::to_value(self).expect("a save is plain data");
        let fields = value.as_object().expect("a save is a JSON object");
        let mut out = String::from("{\n");
        for (i, (key, v)) in fields.iter().enumerate() {
            out.push_str(&format!("  {}: ", serde_json::Value::String(key.clone())));
            match v.as_array() {
                Some(orders) if key == "orders" && !orders.is_empty() => {
                    out.push('[');
                    for (j, o) in orders.iter().enumerate() {
                        out.push_str(if j == 0 { "\n    " } else { ",\n    " });
                        out.push_str(&o.to_string());
                    }
                    out.push_str("\n  ]");
                }
                _ => out.push_str(&v.to_string()),
            }
            out.push_str(if i + 1 < fields.len() { ",\n" } else { "\n" });
        }
        out.push_str("}\n");
        out
    }

    pub fn from_text(text: &str) -> Result<Save, String> {
        // the layout is asked for first, so a later one is refused by name and not by a missing field
        let value: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("not a saved game: {e}"))?;
        match value.get("format").and_then(|f| f.as_u64()) {
            None => return Err("not a saved game: it does not say which layout it is in".to_string()),
            Some(f) if f > FORMAT as u64 => {
                return Err(format!("saved by a later version of the game (layout {f}; this one reads up to {FORMAT})"));
            }
            Some(_) => {}
        }
        let save: Save = serde_json::from_value(value).map_err(|e| format!("not a saved game: {e}"))?;
        save.sound()?;
        Ok(save)
    }

    /// Whether the file says only what a game could have written: a number of
    /// families a game may have, a year among the island's years, a month
    /// that is a month. A file that says otherwise was not written by the
    /// game, or was changed afterwards, and it is refused before anything is
    /// played: a save doctored to the year 5000 was once taken at its word.
    /// (An order that names a square off the map, a town that is not there or
    /// a method nobody knows is another matter: the simulation refuses it
    /// when it comes to it, and the rest of the game is played.)
    pub fn sound(&self) -> Result<(), String> {
        if self.format == 0 {
            return Err("not a saved game the game wrote: it says it is in layout 0, and the first layout was 1".to_string());
        }
        if let Err(why) = families_allowed(self.families) {
            return Err(format!("not a saved game the game wrote: it names {} families, and {why}", self.families));
        }
        let C = self.played_under().content();
        // the year it is played to is its own (a game past 1900), or the content's
        if let Some(to) = self.to {
            if let Err(why) = last_year_allowed(self.played_under(), to) {
                return Err(format!("not a saved game the game wrote: it says it is played to {to}, and {why}"));
            }
        }
        let last = self.to.unwrap_or(C.end_year);
        // (a game saved before its first year was played says the year before the first)
        if self.year < C.start_year - 1 || self.year > last {
            return Err(format!(
                "not a saved game the game wrote: it says the game had reached the year {}, and the island's years are {} to {}",
                self.year, C.start_year, last
            ));
        }
        if self.month > 11 {
            return Err(format!("not a saved game the game wrote: it says {} months of a year had been played, and a save is made before the twelfth", self.month));
        }
        // (twelve is word to the family carried out after December, before the books were closed)
        if let Some(given) = self.orders.iter().find(|g| g.month > 12) {
            return Err(format!("not a saved game the game wrote: it has an order dated {} months into {}, and a year has twelve", given.month, given.year + 1));
        }
        Ok(())
    }

    /// The last year the game is played to: its own (`to`), or the content's.
    pub fn last_year_of_game(&self) -> i64 {
        self.to.unwrap_or_else(|| content_last_year(self.played_under()))
    }

    /// The rules it was played under. A name this version does not know is
    /// taken for the game's, which every game a player starts is under.
    pub fn played_under(&self) -> Rules {
        Rules::parse(&self.rules.name).unwrap_or(Rules::Game)
    }

    /// Whether it was written under the rules of today, by this version.
    pub fn same_rules(&self) -> bool {
        self.rules == rules_id(self.played_under()) && self.version == env!("CARGO_PKG_VERSION")
    }
}

/// A game played again from a save.
#[derive(Clone, Debug)]
pub struct Loaded {
    /// The game, with the orders that were carried out again as its log.
    pub game: Game,
    /// Orders the simulation refused this time, and why. They are left out of
    /// the game's log.
    pub refused: Vec<(Given, String)>,
    /// The island does not stand as the save says it stood: the simulation has
    /// changed since, or the save was made on a machine whose arithmetic
    /// differs in the last digit.
    pub drifted: bool,
}

/// Play a saved game again: the months up to the one it was saved in, with
/// every order carried out where it was first carried out, and the orders
/// that were waiting put back to wait. `each_year` is called after each year,
/// before the orders given at its turn (as a viewer gets a year before the
/// player answers it).
pub fn load(save: &Save, mut each_year: impl FnMut(&Sim)) -> Result<Loaded, String> {
    if save.format > FORMAT {
        return Err(format!("saved by a later version of the game (layout {}; this one reads up to {FORMAT})", save.format));
    }
    // a save made in memory has not been through `from_text`
    save.sound()?;
    // played again on the land it says, whatever today's rules take (a save without the
    // line was made on the port's island before the game's rules took the new land), and
    // played to the year it was begun to (a save that does not say is a game to 1900)
    let to = save.last_year_of_game();
    // and from the start it says, with the start's switches as it began
    let switches = Switches { settlers: save.settlers, ideas_by_sea: save.ideas_by_sea };
    let mut game = Game::begin(save.played_under(), save.seed, save.families, save.started, to, save.land, save.landfall, switches)?;
    let mut refused = Vec::new();
    let mut next = 0;
    // where the save had got to: a year's turn, or some months into the year after it
    let until = match save.year {
        y if y >= game.sim.end_year => (game.sim.end_year, 0),
        y => (y, save.month.min(11)),
    };
    loop {
        // the orders carried out at this point of the calendar
        while next < save.orders.len() && (save.orders[next].year, save.orders[next].month) <= (game.last_year(), game.month) {
            let given = &save.orders[next];
            next += 1;
            match carry_out(&mut game.sim, &given.order) {
                Ok(_) => game.orders.push(Given { year: game.last_year(), month: game.month, order: given.order.clone() }),
                Err(why) => refused.push((given.clone(), why)),
            }
        }
        if game.month == 12 {
            game.close_year();
            each_year(&game.sim);
        } else if (game.last_year(), game.month) >= until {
            break;
        } else {
            game.play_month()?;
        }
    }
    // orders dated after the point the save reached cannot have been given
    for given in &save.orders[next..] {
        refused.push((given.clone(), format!("it is dated {}, after the save's last year", given.year)));
    }
    let drifted = !save.check.is_empty() && check(&game.sim) != save.check;
    // what was waiting waits again: for the year's end, in a save made in the
    // middle of a year; in one made at a year's turn before `Game::turn` was
    // called, the works and land that the turn carries out, which the next
    // `month`, `give` or `turn` does as it would have. Only a save whose game
    // was over before it got there has orders that can never be carried out
    if game.month > 0 || (game.last_year(), game.month) == (save.year, save.month) {
        game.waiting = save.waiting.clone();
    } else {
        for order in &save.waiting {
            refused.push((Given { year: save.year, month: save.month, order: order.clone() }, "it was waiting for a year's end the save never reached".to_string()));
        }
    }
    Ok(Loaded { game, refused, drifted })
}

/// A fingerprint of a run so far: every month of every series, every family's
/// cash and seat, and who owns each cell and each works. Two runs with the
/// same fingerprint have had the same history, to the last digit. (The books'
/// own series are left out, `accounts::own_series`: they are worked out from
/// the history and are no part of it, so a game saved without books does not
/// look like another island when it is played again with them.)
pub fn check(sim: &Sim) -> String {
    let st = &sim.st;
    let mut h = Fnv::new();
    h.word(sim.year as u64);
    for (name, series) in st.series.names.iter().zip(&st.series.data) {
        if crate::accounts::own_series(name) {
            continue;
        }
        for v in &series[..sim.t.min(series.len())] {
            // every NaN counts as the same one
            h.word(if v.is_nan() { u64::MAX } else { v.to_bits() });
        }
    }
    for f in &st.families {
        h.word(f.cash.to_bits());
        h.word(f.seat as u64);
        h.word(f.members.len() as u64);
        h.word(f.decisions.len() as u64);
    }
    for &o in &st.cell_owner {
        h.word(o as u64);
    }
    for &o in &st.f_owner[..st.F_count.min(st.f_owner.len())] {
        h.word(o as u64);
    }
    h.hex()
}

/// FNV-1a, 64 bits: small, and the same on every machine.
struct Fnv(u64);

impl Fnv {
    fn new() -> Fnv {
        Fnv(0xcbf2_9ce4_8422_2325)
    }

    fn bytes(&mut self, b: &[u8]) {
        for &x in b {
            self.0 = (self.0 ^ x as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    fn word(&mut self, w: u64) {
        self.bytes(&w.to_le_bytes());
    }

    fn hex(&self) -> String {
        format!("{:016x}", self.0)
    }
}

/// Seconds since 1970 as a date and time of day, UTC ("2026-11-11 03:12 UTC").
pub fn utc(secs: u64) -> String {
    let (days, rest) = ((secs / 86_400) as i64, secs % 86_400);
    // days to a civil date (Howard Hinnant's algorithm)
    let z = days + 719_468;
    let (era, doe) = (z.div_euclid(146_097), z.rem_euclid(146_097));
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02} UTC", rest / 3_600, rest % 3_600 / 60)
}
