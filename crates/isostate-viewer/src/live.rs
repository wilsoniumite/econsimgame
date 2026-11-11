//! A game in progress. The simulation runs in a thread of its own; every year
//! it finishes is written into the same arrays a recorded run fills, so the
//! map, the panels and the timeline cannot tell a live game from a replay.
//!
//! The thread does nothing on its own. The viewer asks for the next month as
//! the clock comes to its end, and gets back the little that a month changes
//! (the series, the towns' prices and wages, the families' cash, a railway
//! opened); after December comes the year's end, and with it the year as
//! before: the map, the works, the families and the news. The player's orders
//! (buy this land, build that works) go in between. One given at a year's
//! turn, before January is asked for, is carried out at once; one given later
//! in the year waits for the year's end (`isostate_core::save` says which end
//! of it), and the thread says so in its answer.
//!
//! The thread also keeps the game saved. Every order it carries out goes in a
//! log, and the log is written to the island's autosave after every year and
//! every order, and when the clock stops in the middle of a year (`saves`).
//! Going on with a saved game plays its months again here, before the window
//! opens, carrying each order out where it was first carried out.

use std::sync::{mpsc, Mutex};

use bevy::prelude::*;
use isostate_core::families::{self, Group, Posting, Stance, N_GROUPS};
use isostate_core::save::{Done, Game, Order, Outcome};
use isostate_core::{self as iso, FamilyReport, LiveCursor, LiveMonth, LiveStatic, LiveYear, Prospect, SiteOption};

use crate::family::FamilyView;
use crate::replay::{self, Firm, Good, Header, Meta, Method, Replay, Town};
use crate::saves::{self, Saver};

/// The player's family is always the first.
pub const PLAYER: usize = 0;

pub enum Cmd {
    /// work out the next month, and after December the year's end with it
    Step,
    /// what could be done with this cell
    Site(usize),
    Found { method: usize, cell: usize },
    BuyLand(usize),
    /// answer a decision with one of its options
    Decide { id: u64, option: usize },
    /// post an adult to run a trade in a town, or take them off it
    Post { member: usize, posting: Posting },
    /// what a branch does each year
    Stance { group: Group, town: usize, stance: Stance },
    /// move the seat (and the head) at the year end; the present seat cancels a move
    MoveSeat(usize),
    /// take back one of the orders that wait for the year's end, by its place among them
    Cancel(usize),
    /// the clock has stopped: write the save now if months have gone by since the last
    Keep,
}

/// What waits for the year's end, as the interface shows it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Standing {
    /// the orders given since the year's turn, in the order they were given
    pub waiting: Vec<Order>,
    /// how much of the family's cash they would take if carried out today
    pub promised: f64,
    /// squares where a works is to stand and does not yet: ordered and
    /// waiting, or begun at the last turn
    pub pegs: Vec<usize>,
}

/// A year's end as the thread sends it.
pub struct YearEnd {
    pub year: LiveYear,
    /// what each trade would earn in each town (`families::prospect_works`)
    pub prospects: Vec<[Prospect; N_GROUPS]>,
    /// what became of each order that waited for this year's end: whether it
    /// was carried out, and a line to tell the player
    pub carried: Vec<(bool, String)>,
    /// the player's family once the turn's orders were carried out, if any were
    pub family: Option<FamilyReport>,
    pub standing: Standing,
}

pub struct SiteInfo {
    pub cell: usize,
    /// the town whose reach the cell is in
    pub town: Option<usize>,
    pub land_price: f64,
    pub owner: Option<usize>,
    pub owner_name: Option<String>,
    /// whether the player may buy it (cash aside), or why not
    pub land_allowed: Result<(), String>,
    pub options: Result<Vec<SiteOption>, String>,
}

pub enum Msg {
    /// a month (December's comes just before its year)
    Month(Box<LiveMonth>),
    Year(Box<YearEnd>),
    Site(SiteInfo),
    /// an order was carried out, or waits, or was refused; with the player's
    /// family as it stands (and as it has been told to stand), and what waits
    Done { ok: bool, text: String, family: Option<FamilyReport>, standing: Standing },
    Failed(String),
    /// the game could not be saved, and why; None once it can be again
    Unsaved(Option<String>),
}

#[derive(Resource)]
pub struct Live {
    /// None only while the game is being left
    tx: Option<mpsc::Sender<Cmd>>,
    rx: Mutex<mpsc::Receiver<Msg>>,
    thread: Option<std::thread::JoinHandle<()>>,
    /// months of the year in progress (the one after the newest year's end
    /// there is) that have been asked for, and that have come
    pub asked: usize,
    pub months: usize,
    /// the newest year (as an index) the clock has shown. The next is asked
    /// for a little before it is due, so for a moment there is a year worked
    /// out that the player has not seen; see `game::Now`.
    pub seen: usize,
    /// when each month still on its way was asked for
    sent: std::collections::VecDeque<std::time::Instant>,
    /// when the last month or year came, and when December's work began
    came: Option<std::time::Instant>,
    turn_began: Option<std::time::Instant>,
    /// how long the last month took to arrive, and the last year's end with
    /// its December, in seconds
    pub month_took: Option<f32>,
    pub turn_took: Option<f32>,
    /// what waits for the year's end
    pub standing: Standing,
    /// the thread has stopped and that has been reported
    gone: std::sync::atomic::AtomicBool,
    /// the game was gone on with from a save, and how that went
    pub resumed: Option<Resumed>,
    /// squares where works ordered since the last year will stand
    pub ordered: Vec<usize>,
    /// why the last save could not be written, if it could not
    pub unsaved: Option<String>,
    /// the game is being saved as it is played (a run that only takes pictures is not)
    pub kept: bool,
}

/// How going on with a saved game went.
#[derive(Clone, Debug, PartialEq)]
pub struct Resumed {
    /// the year the game is gone on with from: the one the save had reached,
    /// or later if more were played before the window opened (`--advance`)
    pub year: i32,
    /// months of the year after it that had been played
    pub month: usize,
    /// orders given again
    pub orders: usize,
    /// orders the simulation refused this time: its rules have changed since the save
    pub refused: usize,
    /// the island does not stand as the save says it stood
    pub drifted: bool,
    /// newer saves that could not be read, so that this is an earlier one
    pub passed_over: usize,
}

impl Resumed {
    /// Where the game is gone on with from, as the clock will show it:
    /// "January 1787" after 1786, "June 1787" for one left in the middle of that year.
    pub fn date(&self) -> String {
        crate::clock::month_year(crate::clock::Date { year: self.year + 1, month: self.month.min(11), part: 0.0 })
    }
}

impl Live {
    pub fn send(&self, c: Cmd) {
        // the thread only stops when the app does
        if let Some(tx) = &self.tx {
            let _ = tx.send(c);
        }
    }

    pub fn try_recv(&self) -> Option<Msg> {
        match self.rx.lock().ok()?.try_recv() {
            Ok(m) => Some(m),
            Err(mpsc::TryRecvError::Empty) => None,
            // the thread is gone (it panicked): say so once, as a failure
            Err(mpsc::TryRecvError::Disconnected) if !self.gone.swap(true, std::sync::atomic::Ordering::Relaxed) => {
                Some(Msg::Failed("The simulation stopped unexpectedly; the game cannot go on. The terminal may say why.".into()))
            }
            Err(_) => None,
        }
    }

    /// A month, or the year's end, is being worked out.
    pub fn stepping(&self) -> bool {
        self.asked > self.months || self.months >= 12
    }

    /// The simulation stands at a year's turn and nothing has been asked of
    /// it since: an order given now is carried out at once.
    pub fn at_turn(&self) -> bool {
        self.asked == 0
    }

    /// December has been asked for, and the year's end comes with it: until
    /// it has come, an order or an answer given now would arrive after it.
    pub fn turning(&self) -> bool {
        self.asked >= 12
    }

    /// Ask for the next month, if the year has one left to ask for.
    pub fn step(&mut self) {
        if self.asked < 12 {
            self.asked += 1;
            self.sent.push_back(std::time::Instant::now());
            self.send(Cmd::Step);
        }
    }

    /// A month has come. December's is not the end of its work: the year follows.
    pub fn month_came(&mut self) {
        let now = std::time::Instant::now();
        // the work began when it was asked for, or when the month before it was done
        let began = self.sent.pop_front().map(|t0| self.came.map_or(t0, |c| c.max(t0)));
        self.months = (self.months + 1).min(12);
        if self.months < 12 {
            if let Some(b) = began {
                self.month_took = Some((now - b).as_secs_f32());
            }
        } else {
            self.turn_began = began;
        }
        self.came = Some(now);
    }

    /// The year's end has come: the next year's months are still to ask for.
    pub fn year_came(&mut self) {
        let now = std::time::Instant::now();
        if let Some(b) = self.turn_began.take() {
            self.turn_took = Some((now - b).as_secs_f32());
        }
        self.came = Some(now);
        (self.asked, self.months) = (0, 0);
        self.sent.clear();
    }

    /// What was asked for failed to come.
    pub fn failed(&mut self) {
        self.asked = self.months;
        self.sent.clear();
        self.turn_began = None;
    }
}

impl Drop for Live {
    /// Leaving the game: the thread finishes what it was asked, and the save
    /// of it, before the program ends.
    fn drop(&mut self) {
        self.tx = None;
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// How a game opens.
#[derive(Clone, Debug, Default)]
pub struct Opening {
    pub seed: u64,
    pub families: usize,
    /// years to play before the window opens, after the first (or after the saved ones)
    pub advance: usize,
    /// the player's family builds a works and buys a little land after the
    /// first year, to have something to show in screenshots; such a game is
    /// only saved where `save` says
    pub demo: bool,
    /// a saved game to go on with (`--load`)
    pub load: Option<String>,
    /// where to save, instead of the island's autosave (`--save`)
    pub save: Option<String>,
    /// a new game was asked for (`--new` or `--seed`), whatever is saved
    pub fresh: bool,
    /// only pictures are being taken: nothing is saved unless a place is named
    pub shots: bool,
}

impl Opening {
    /// Nothing was asked for but the game: the island's autosave, if there is
    /// one, is gone on with. Screenshots, and the switches that ask for a game
    /// of a certain shape or kept in a certain place, get a new one.
    pub fn plain(&self) -> bool {
        !self.fresh && !self.shots && !self.demo && self.advance == 0 && self.load.is_none() && self.save.is_none()
    }
}

/// The island as the viewer holds it, filled in year by year.
struct Board {
    cursor: LiveCursor,
    r: Replay,
    view: FamilyView,
}

impl Board {
    fn new(sim: &iso::Sim) -> Board {
        let stat = iso::live_static(&sim.st, &sim.content);
        let method_group: Vec<Option<Group>> = stat.methods.iter().map(|m| m.group).collect();
        Board { cursor: LiveCursor::new(&sim.content), r: shell(&stat, sim.params.seed), view: FamilyView { method_group, ..default() } }
    }

    /// Take in the year the simulation has just played, with its twelve months.
    fn year(&mut self, sim: &iso::Sim) {
        self.months(sim, 12);
        let y = iso::live_year(&mut self.cursor, &sim.st, &sim.content, &sim.params, &sim.events);
        apply_year(&mut self.r, &y);
        crate::game::year_families(&mut self.view, &y.families, y.year as i32);
    }

    /// Take in the last `n` months the simulation has played: their series.
    /// (What the towns paid month by month is not kept by the simulation, so
    /// a game gone on with has it only from the month it is gone on with.)
    fn months(&mut self, sim: &iso::Sim, n: usize) {
        for t in sim.t.saturating_sub(n)..sim.t {
            apply_series(&mut self.r, t, &iso::live::month_series(&sim.st, t));
        }
    }
}

/// What waits for the year's end in a game, for the interface.
fn standing(game: &Game, begun: &[usize]) -> Standing {
    let s = &game.sim;
    let promised = match (game.foreseen(), iso::family_report(&s.st, &s.content, &s.params, PLAYER)) {
        (Some(seen), Ok(now)) => (now.cash - seen.cash).max(0.0),
        _ => 0.0,
    };
    let waiting_works = game.waiting.iter().filter_map(|o| if let Order::Found { cell, .. } = o { Some(*cell) } else { None });
    Standing { waiting: game.waiting.clone(), promised, pegs: begun.iter().copied().chain(waiting_works).collect() }
}

/// The player's family for the panels: as it stands, and in what the family
/// has been told (who is posted where, what a branch does, where the seat
/// goes, what is decided) as it will stand once the orders that wait are
/// carried out. What it owns and has in hand is only ever what it has now.
fn family_now(game: &Game) -> Option<FamilyReport> {
    let s = &game.sim;
    let now = iso::family_report(&s.st, &s.content, &s.params, PLAYER).ok()?;
    Some(match game.foreseen() {
        Some(seen) => FamilyReport {
            cash: now.cash,
            works_value: now.works_value,
            land_value: now.land_value,
            net_worth: now.net_worth,
            firms: now.firms,
            absentee: now.absentee,
            cells: now.cells,
            ..seen
        },
        None => now,
    })
}

/// The squares where works were begun at the last year's turn: by an order
/// carried out there, which the year's own picture does not show yet.
fn begun_at_turn(game: &Game) -> Vec<usize> {
    game.orders
        .iter()
        .filter(|g| g.year == game.last_year() && g.month == 0)
        .filter_map(|g| if let Order::Found { cell, .. } = &g.order { Some(*cell) } else { None })
        .collect()
}

/// What an order is, in a few words, for the list of those that wait.
pub fn order_words(order: &Order, r: &Replay, rep: Option<&FamilyReport>) -> String {
    let town = |t: usize| r.header.towns.get(t).map_or_else(|| format!("town {t}"), |x| x.name.clone());
    let near = |c: usize| {
        let t = r.filled.checked_sub(1).and_then(|k| r.catch.get(k * r.n + c)).and_then(|&t| (t as usize).checked_sub(1));
        t.map_or_else(|| "the island".to_string(), town)
    };
    match order {
        Order::Found { method, cell } => {
            let label = r.header.methods.iter().find(|m| &m.name == method).map_or(method.as_str(), |m| m.label.as_str());
            format!("Build by {}: {}", near(*cell), label.to_lowercase())
        }
        Order::BuyLand { cell } => format!("Buy a square of land by {}", near(*cell)),
        Order::Decide { choice, .. } => match choice {
            iso::Choice::Trade { group } => format!("Take up {}", crate::family::trade(*group)),
            iso::Choice::Apprentice { group, town: t } => format!("An apprenticeship in {} at {}", crate::family::trade(*group), town(*t)),
            iso::Choice::University => "University".to_string(),
            iso::Choice::Head { member } => match rep.and_then(|f| f.members.get(*member)) {
                Some(m) => format!("{} to head the family", m.first),
                None => "A new head of the family".to_string(),
            },
        },
        Order::Post { member, posting } => {
            let who = rep.and_then(|f| f.members.get(*member)).map_or_else(|| "Someone".to_string(), |m| m.first.clone());
            match posting {
                Posting::Manager { group, town: t } => format!("{who} to run {} at {}", crate::family::trade(*group), town(*t)),
                Posting::Unposted => format!("{who} off the post"),
            }
        }
        Order::Stance { group, town: t, stance } => {
            let does = match stance {
                Stance::Grow => "to grow",
                Stance::Hold => "to hold",
                Stance::WindDown => "to wind down",
            };
            format!("{} at {} {does}", crate::family::short_group(*group), town(*t))
        }
        Order::MoveSeat { town: t } => format!("Move the seat to {}", town(*t)),
    }
}

/// When an order given now, in the middle of a year, will happen: said to
/// the player in the thread's answer.
fn waits_words(game: &Game, order: &Order) -> String {
    // the year in progress, whose end the order waits for
    let year = game.sim.year;
    match order {
        Order::Found { .. } => format!("Ordered. Building begins at the year's turn, in January {}, if the money is there then.", year + 1),
        Order::BuyLand { .. } => format!("Ordered. The land is bought at the year's turn, in January {}, at its price then.", year + 1),
        Order::Decide { .. } => format!("Decided. It takes effect at the end of {year}, where the default would have."),
        Order::Post { .. } => format!("Told. It takes effect at the end of {year}."),
        Order::Stance { .. } => format!("Told. The branch goes by it from the end of {year}."),
        Order::MoveSeat { .. } => format!("Told. The seat moves at the end of {year}."),
    }
}

/// What became of an order that waited, in a line for the player.
fn carried_words(s: &iso::Sim, o: &Outcome, what: &str) -> (bool, String) {
    match &o.result {
        Ok(Done::Founded(f)) => (true, format!("{} is begun, and will open in {}", s.st.f_name[*f], s.year)),
        Ok(Done::Bought(price)) => (true, format!("{what}: bought for {}", crate::ui::money(*price as f32))),
        Ok(_) => (true, format!("{what}: done")),
        Err(why) => (false, format!("{what}: not done, because {why}")),
    }
}

/// Start a game: the island is made and its first year simulated here (and
/// `advance` more, to start later), then the simulation moves to its own
/// thread. With a save to go on from (one named, or the island's autosave when
/// nothing else was asked for) its years are played again first, with the
/// orders in it. A new game is played under the game's rules (`game_params`,
/// `game_content`), not the port's; a saved one under the rules it names.
pub fn start(o: &Opening) -> Result<(Replay, Live, FamilyView), String> {
    let t0 = std::time::Instant::now();
    // said before any year is played, not after
    if o.load.as_ref().is_some_and(|file| file.is_empty()) {
        return Err("--load needs the name of a saved game".to_string());
    }
    if let Some(file) = &o.save {
        saves::may_keep(std::path::Path::new(file))?;
    }
    let mut passed_over = 0;
    let from = match &o.load {
        Some(file) => Some((saves::read(std::path::Path::new(file))?, std::path::PathBuf::from(file))),
        None if o.plain() => {
            // a game has at least one family, and its autosave is named for the families it has
            saves::read_latest(&saves::autosave(o.seed, o.families.max(1))).map(|(save, file, passed)| {
                for e in &passed {
                    eprintln!("the latest save could not be read ({e}); going back to the one before it");
                }
                passed_over = passed.len();
                (save, file)
            })
        }
        None => None,
    };

    let mut board: Option<Board> = None;
    let mut resumed = None;
    let mut game = match &from {
        Some((save, file)) => {
            let years = (save.year - save.played_under().content().start_year + 1).max(0);
            eprintln!(
                "going on with the {} family's game on island {} ({} families), saved {} in {}: {}",
                save.family,
                save.seed,
                save.families,
                save.saved,
                save.year,
                file.display()
            );
            if !save.same_rules() {
                eprintln!("  it was saved under other rules (version {}), so the years may play out differently", save.version);
            }
            let s = |n: usize| if n == 1 { "" } else { "s" };
            eprintln!("  playing {} year{} again, with {} order{}\u{2026}", years, s(years as usize), save.orders.len(), s(save.orders.len()));
            let loaded = iso::save::load(save, |sim| {
                board.get_or_insert_with(|| Board::new(sim)).year(sim);
                if (sim.year - 1) % 10 == 0 {
                    eprintln!("  {}", sim.year - 1);
                }
            })
            // a plain start would meet the same save every time: say the way round it
            .map_err(|e| format!("{} cannot be played again ({e}); --new starts a new game and sets it aside", file.display()))?;
            for (given, why) in &loaded.refused {
                eprintln!("  an order of {} could not be given again ({why}): {}", given.year, serde_json::to_string(&given.order).unwrap_or_default());
            }
            if loaded.drifted {
                eprintln!("  the island does not stand quite as it was saved: the simulation has changed since");
            }
            let reached = match loaded.game.month { 0 => format!("at the end of {}", loaded.game.last_year()), m => format!("{m} months into {}", loaded.game.last_year() + 1) };
            eprintln!("  back {reached} after {:.1}s", t0.elapsed().as_secs_f32());
            resumed = Some(Resumed {
                year: loaded.game.last_year() as i32,
                month: loaded.game.month,
                orders: loaded.game.orders.len(),
                refused: loaded.refused.len(),
                drifted: loaded.drifted,
                passed_over,
            });
            loaded.game
        }
        None => {
            eprintln!("making island {} with {} families\u{2026}", o.seed, o.families);
            Game::new(o.seed, o.families, saves::now())
        }
    };
    let mut board = board.unwrap_or_else(|| Board::new(&game.sim));
    // a new game opens with its first year played
    let new = game.sim.year == game.sim.content.start_year;
    for i in 0..o.advance + new as usize {
        if game.over() {
            break;
        }
        // (from a save made in the middle of a year, the first of these is the rest of that year)
        game.year()?;
        board.year(&game.sim);
        // what waited for the year's end, in a save made in the middle of a year
        for done in game.turn() {
            if let Err(why) = &done.result {
                eprintln!("  an order that waited for the end of {} could not be carried out ({why}): {}", game.last_year(), serde_json::to_string(&done.order).unwrap_or_default());
            }
        }
        if new && i == 0 && o.demo {
            demo_orders(&mut game);
        }
    }
    // a game saved in the middle of a year is gone on with there: the months played of it
    board.months(&game.sim, game.month);
    // years played on after the saved ones: the card says where the game opens
    if let Some(back) = &mut resumed {
        (back.year, back.month) = (game.last_year() as i32, game.month);
    }
    let Board { cursor, r, mut view } = board;
    // orders given since the last year: the family as it stands now, and the works to come
    if let Some(rep) = family_now(&game) {
        crate::game::player_family(&mut view, &rep);
    }
    let begun = begun_at_turn(&game);
    let stands = standing(&game, &begun);
    let ordered = stands.pegs.clone();
    view.prospects = all_prospects(&game.sim);

    // where the game is kept: a place that was named, or the island's autosave
    // (which a run that only takes pictures leaves alone, and so does the
    // demo's game, made to be looked at: it would set the player's own aside)
    let mut saver = match &o.save {
        Some(file) => Some(Saver::new(file.into())),
        None if o.shots || (o.demo && o.load.is_none()) => None,
        None => Some(Saver::new(saves::autosave(game.sim.params.seed, game.sim.params.n_families))),
    };
    match (&mut saver, &o.save) {
        (Some(s), Some(_)) => {
            // a place that was named is written at once, to say now if it cannot be
            s.write(&game.save(saves::now()), true)?;
            eprintln!("saved in {}: {}", game.last_year(), s.path.display());
        }
        // the autosave is first written when something happens in the game
        (Some(s), None) => eprintln!("the game is kept in {}", s.path.display()),
        (None, _) => {}
    }

    let kept = saver.is_some();
    let months = game.month;

    let (tx_cmd, rx_cmd) = mpsc::channel::<Cmd>();
    let (tx_msg, rx_msg) = mpsc::channel::<Msg>();
    let thread = std::thread::Builder::new()
        .name("isostate-sim".into())
        .spawn(move || run_thread(game, cursor, saver, begun, rx_cmd, tx_msg))
        .map_err(|e| format!("cannot start the simulation thread: {e}"))?;
    let live = Live {
        tx: Some(tx_cmd),
        rx: Mutex::new(rx_msg),
        thread: Some(thread),
        asked: months,
        months,
        // a game opens on the newest year there is
        seen: r.filled.max(1) - 1,
        sent: Default::default(),
        came: None,
        turn_began: None,
        month_took: None,
        turn_took: None,
        standing: stands,
        gone: std::sync::atomic::AtomicBool::new(false),
        resumed,
        ordered,
        unsaved: None,
        kept,
    };
    Ok((r, live, view))
}

/// For screenshots: the best-paying works near home that costs at most half
/// the family's cash (taking up its trade first, if the first trade is still
/// to choose), and the cheapest few squares around it. They are given as a
/// player's orders are, after the first year, so a save holds them.
fn demo_orders(game: &mut Game) {
    let s = &game.sim;
    let Ok(rep) = iso::family_report(&s.st, &s.content, &s.params, PLAYER) else { return };
    let home = s.st.town_node[rep.seat];
    let (w, h) = (s.st.world.W as i64, s.st.world.H as i64);
    let (hx, hy) = ((home % s.st.world.W) as i64, (home / s.st.world.W) as i64);
    let near: Vec<usize> = (-2..=2)
        .flat_map(|dy| (-2..=2).map(move |dx| (hx + dx, hy + dy)))
        .filter(|&(x, y)| x >= 0 && y >= 0 && x < w && y < h && (x, y) != (hx, hy))
        .map(|(x, y)| (y * w + x) as usize)
        .collect();
    // while the family's first trade is still to choose, a works in any trade
    // on offer counts: the demo takes up the trade and then builds
    let offered: Vec<(u64, Group)> = families::pending(&s.st, PLAYER)
        .iter()
        .flat_map(|d| d.options.iter().filter_map(move |o| if let iso::Choice::Trade { group } = o.choice { Some((d.id, group)) } else { None }))
        .collect();
    let mut best: Option<(f64, usize, usize, Option<Group>)> = None;
    for &c in &near {
        let Ok(opts) = iso::site_options(&s.st, &s.content, &s.params, PLAYER, c) else { continue };
        let at_seat = s.st.catch[c] == rep.seat as i64;
        for o in opts {
            let may = o.allowed.is_ok() || (at_seat && o.group.is_some_and(|g| offered.iter().any(|&(_, x)| x == g)));
            if let (Ok(sums), true) = (o.sums, may) {
                if sums.capital_cost <= rep.cash * 0.5 && best.is_none_or(|b| sums.expected_return > b.0) {
                    best = Some((sums.expected_return, o.method, c, o.group));
                }
            }
        }
    }
    if best.is_none() {
        let mut cheapest = f64::MAX;
        let mut errs = std::collections::BTreeMap::<String, usize>::new();
        for &c in &near {
            match iso::site_options(&s.st, &s.content, &s.params, PLAYER, c) {
                Ok(opts) => {
                    for o in opts {
                        match o.sums {
                            Ok(sums) => cheapest = cheapest.min(sums.capital_cost),
                            Err(e) => *errs.entry(e.split(" is not").next().unwrap_or("").to_string()).or_default() += 1,
                        }
                    }
                }
                Err(e) => *errs.entry(e).or_default() += 1,
            }
        }
        eprintln!("demo: nothing affordable near home; cash {:.0}, cheapest works {:.0}, {} cells; {:?}", rep.cash, cheapest, near.len(), errs);
    }
    if let Some((_, m, c, group)) = best {
        if let Some(&(id, g)) = offered.iter().find(|&&(_, g)| Some(g) == group && !rep.known.contains(&g)) {
            if let Err(e) = game.give(Order::Decide { id, choice: iso::Choice::Trade { group: g } }) {
                eprintln!("demo: could not take up the trade: {e}");
            }
        }
        if let Err(e) = Order::found(&game.sim, m, c).and_then(|o| game.give(o)) {
            eprintln!("demo: could not build: {e}");
        }
        let s = &game.sim;
        let (cx, cy) = ((c % s.st.world.W) as i64, (c / s.st.world.W) as i64);
        let mut around: Vec<(f64, usize)> = (-1..=1)
            .flat_map(|dy| (-1..=1).map(move |dx| (cx + dx, cy + dy)))
            .filter(|&(x, y)| x >= 0 && y >= 0 && x < w && y < h)
            .map(|(x, y)| (y * w + x) as usize)
            .filter(|&c| s.st.world.land[c])
            .map(|c| (families::land_price(&s.st, &s.params, c), c))
            .collect();
        around.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (_, c) in around.into_iter().take(4) {
            let _ = game.give(Order::BuyLand { cell: c });
        }
    }
}

/// What each trade would earn on new works in every town, as the posting
/// suggestions and the choice of a first trade show it.
fn all_prospects(s: &iso::Sim) -> Vec<[Prospect; N_GROUPS]> {
    let t0 = std::time::Instant::now();
    let none = [Prospect { roi: f64::NEG_INFINITY, cost: 0.0 }; N_GROUPS];
    let out: Vec<[Prospect; N_GROUPS]> = (0..s.st.T_max)
        .map(|t| if s.st.town_active[t] { families::prospect_works(&s.st, &s.content, &s.params, t) } else { none })
        .collect();
    if std::env::var_os("ISOSTATE_TIMING").is_some() {
        eprintln!("prospects for {} towns in {:.0} ms", out.len(), t0.elapsed().as_secs_f32() * 1e3);
    }
    out
}

/// A command as the order the log keeps. Asking for a month or about a square is not one.
fn order_of(sim: &iso::Sim, cmd: Cmd) -> Result<Order, String> {
    match cmd {
        Cmd::Step | Cmd::Site(_) | Cmd::Cancel(_) | Cmd::Keep => Err("that is not an order".to_string()),
        Cmd::Found { method, cell } => Order::found(sim, method, cell),
        Cmd::BuyLand(cell) => Ok(Order::BuyLand { cell }),
        Cmd::Decide { id, option } => Order::decide(sim, id, option),
        Cmd::Post { member, posting } => Ok(Order::Post { member, posting }),
        Cmd::Stance { group, town, stance } => Ok(Order::Stance { group, town, stance }),
        Cmd::MoveSeat(town) => Ok(Order::MoveSeat { town }),
    }
}

/// Write the game's save, and tell the interface once when saving starts to
/// fail and once when it works again. False if the interface has gone.
fn write_save(game: &Game, saver: &mut Option<Saver>, saved: &mut bool, year_end: bool, tx: &mpsc::Sender<Msg>) -> bool {
    let Some(saver) = saver else { return true };
    let now = saver.write(&game.save(saves::now()), year_end);
    if let Err(e) = &now {
        eprintln!("the game could not be saved: {e}");
    }
    if now.is_ok() != *saved {
        *saved = now.is_ok();
        return tx.send(Msg::Unsaved(now.err())).is_ok();
    }
    true
}

fn run_thread(mut game: Game, mut cursor: LiveCursor, mut saver: Option<Saver>, mut begun: Vec<usize>, rx: mpsc::Receiver<Cmd>, tx: mpsc::Sender<Msg>) {
    // whether the last save was written, so a failure is told once and not every year
    let mut saved = true;
    // months have been played since the last save
    let mut unkept = false;
    while let Ok(cmd) = rx.recv() {
        let mut year_end = false;
        let msg = match cmd {
            Cmd::Step => {
                if game.over() {
                    Msg::Failed(format!("{} was the last year.", game.sim.end_year))
                } else {
                    match game.month() {
                        Ok(turned) => {
                            unkept = true;
                            let s = &game.sim;
                            let month = iso::live_month(&mut cursor, &s.st, &s.content, &s.params, &s.events, s.t - 1);
                            if !turned {
                                Msg::Month(Box::new(month))
                            } else {
                                // December first, then its year as it ended, before the turn's orders
                                if tx.send(Msg::Month(Box::new(month))).is_err() {
                                    return;
                                }
                                year_end = true;
                                let year = iso::live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
                                let done = game.turn();
                                begun = done.iter().filter(|o| o.result.is_ok()).filter_map(|o| if let Order::Found { cell, .. } = &o.order { Some(*cell) } else { None }).collect();
                                let s = &game.sim;
                                let carried = done.iter().map(|o| carried_words(s, o, &order_line(&game, &o.order))).collect();
                                let family = if done.is_empty() { None } else { family_now(&game) };
                                Msg::Year(Box::new(YearEnd { year, prospects: all_prospects(s), carried, family, standing: standing(&game, &begun) }))
                            }
                        }
                        Err(e) => Msg::Failed(e),
                    }
                }
            }
            Cmd::Site(cell) => {
                let s = &game.sim;
                let owner = families::owner_of_cell(&s.st, cell);
                Msg::Site(SiteInfo {
                    cell,
                    town: s.st.catch.get(cell).and_then(|&t| (t >= 0).then_some(t as usize)),
                    land_price: families::land_price(&s.st, &s.params, cell),
                    owner,
                    owner_name: owner.and_then(|o| s.st.families.get(o)).map(|f| f.surname.clone()),
                    land_allowed: families::may_buy_land(&s.st, PLAYER, cell),
                    options: iso::site_options(&s.st, &s.content, &s.params, PLAYER, cell),
                })
            }
            Cmd::Keep => {
                // the clock has stopped in the middle of a year: the months since the last save are kept
                if std::mem::take(&mut unkept) && !write_save(&game, &mut saver, &mut saved, false, &tx) {
                    return;
                }
                continue;
            }
            Cmd::Cancel(i) => {
                let r = game.cancel(i).map(|_| String::new());
                done(&game, &begun, r)
            }
            order => {
                // carried out, and kept in the log if it was; or kept to wait for the year's end
                let r = order_of(&game.sim, order).and_then(|o| game.give(o.clone()).map(|d| (o, d)));
                if let Ok((Order::Found { cell, .. }, Done::Founded(_))) = &r {
                    begun.push(*cell);
                }
                let s = &game.sim;
                let text = r.map(|(o, d)| match d {
                    Done::Founded(f) => format!("{} will open in {}.", s.st.f_name[f], s.year),
                    Done::Bought(price) => format!("The land is yours, for {}.", crate::ui::money(price as f32)),
                    Done::Done => String::new(),
                    Done::Waits => waits_words(&game, &o),
                });
                done(&game, &begun, text)
            }
        };
        // the game is kept saved: after every year, and after every order carried out or kept to wait
        let changed = year_end || matches!(msg, Msg::Done { ok: true, .. });
        if tx.send(msg).is_err() {
            return;
        }
        if changed {
            unkept = false;
            if !write_save(&game, &mut saver, &mut saved, year_end, &tx) {
                return;
            }
        }
    }
    // the game is being left: the months played since the last save go with it
    if unkept {
        write_save(&game, &mut saver, &mut saved, false, &tx);
    }
}

/// An order in a few words, with nothing but the simulation to go by (the
/// thread has no map): for the lines that say what became of it.
fn order_line(game: &Game, order: &Order) -> String {
    let s = &game.sim;
    let town = |t: usize| s.st.town_name.get(t).cloned().unwrap_or_else(|| format!("town {t}"));
    let near = |c: usize| s.st.catch.get(c).and_then(|&t| (t >= 0).then(|| town(t as usize))).unwrap_or_else(|| "the island".to_string());
    let member = |j: usize| s.st.families.get(PLAYER).and_then(|f| f.members.get(j)).map_or_else(|| "Someone".to_string(), |m| m.first.clone());
    match order {
        Order::Found { method, cell } => {
            let label = s.content.method_name.iter().position(|n| n == method).map_or(method.clone(), |m| s.content.method_label[m].to_lowercase());
            format!("The works by {} ({label})", near(*cell))
        }
        Order::BuyLand { cell } => format!("The land by {}", near(*cell)),
        Order::Decide { choice, .. } => match choice {
            iso::Choice::Trade { group } => format!("Taking up {}", crate::family::trade(*group)),
            iso::Choice::Apprentice { group, town: t } => format!("The apprenticeship in {} at {}", crate::family::trade(*group), town(*t)),
            iso::Choice::University => "University".to_string(),
            iso::Choice::Head { member: j } => format!("{} as head of the family", member(*j)),
        },
        Order::Post { member: j, posting } => match posting {
            Posting::Manager { group, town: t } => format!("{} to run {} at {}", member(*j), crate::family::trade(*group), town(*t)),
            Posting::Unposted => format!("{} off the post", member(*j)),
        },
        Order::Stance { group, town: t, .. } => format!("The word to {} at {}", crate::family::short_group(*group).to_lowercase(), town(*t)),
        Order::MoveSeat { town: t } => format!("The seat's move to {}", town(*t)),
    }
}

fn done(game: &Game, begun: &[usize], r: Result<String, String>) -> Msg {
    let family = family_now(game);
    let standing = standing(game, begun);
    match r {
        Ok(text) => Msg::Done { ok: true, text, family, standing },
        Err(text) => Msg::Done { ok: false, text, family, standing },
    }
}

/// A replay with the island and nothing else, every year still to come.
fn shell(stat: &LiveStatic, seed: u64) -> Replay {
    let (w, h) = (stat.W, stat.H);
    let n = w * h;
    let ny = (stat.end_year - stat.start_year + 1).max(1) as usize;
    let header = Header {
        meta: Meta { label: "a new game".into(), lite: false, seed, w, h, start_year: stat.start_year as i32, n_years: ny },
        goods: stat.goods.iter().map(|g| Good { name: g.name.clone(), label: g.label.clone() }).collect(),
        methods: stat.methods.iter().map(|m| Method { name: m.name.clone(), label: m.label.clone(), out: m.out, k0: m.K0 as f32 }).collect(),
        towns: Vec::new(),
        firms: Vec::new(),
        events: Vec::new(),
        disp: stat.disp.clone(),
        uses: stat.uses.clone(),
        arrays: Vec::new(),
        series: Default::default(),
    };
    let series_len = ny * 12;
    Replay {
        header,
        w,
        h,
        n,
        ny,
        y0: stat.start_year as i32,
        n_towns: 0,
        elev: stat.elev.clone(),
        terrain: stat.terrain.clone(),
        river: stat.river.clone(),
        down: stat.down.clone(),
        use_: vec![0; ny * n],
        inten: vec![0; ny * n],
        trans: vec![0; ny * n],
        catch: vec![0; ny * n],
        rent: vec![0; ny * n],
        dist: vec![0; ny * n],
        mix_a: vec![0; ny * n],
        mix_b: vec![0; ny * n],
        mix_s: vec![255; ny * n],
        town_active: Vec::new(),
        town_pop: Vec::new(),
        town_urban: Vec::new(),
        town_wage: Vec::new(),
        town_price: Vec::new(),
        firm_first: Vec::new(),
        firm_len: Vec::new(),
        firm_off: Vec::new(),
        firm_k: Vec::new(),
        series_names: stat.series.iter().map(|s| s.name.clone()).collect(),
        series: vec![f32::NAN; stat.series.len() * series_len],
        series_len,
        filled: 0,
        owner: Vec::new(),
    }
}

/// Add news to the run's list, each item with its place in the simulation's list.
fn add_news(r: &mut Replay, events: &[iso::Event]) {
    let sent = r.header.events.len();
    for (j, e) in events.iter().enumerate() {
        r.header.events.push(replay::Event {
            year: e.year as f32,
            kind: e.kind.clone(),
            weight: e.weight as f32,
            title: e.title.clone(),
            text: e.text.clone(),
            cell: e.cell.map(|c| c as i64),
            family: e.family,
            kept: e.kept,
            seq: sent + j,
        });
    }
}

/// Write one month of every monthly series into the replay's table: `t` is
/// the month's place in the run, 0 for the first January.
pub fn apply_series(r: &mut Replay, t: usize, values: &[f64]) {
    if t >= r.series_len {
        return;
    }
    for (i, v) in values.iter().enumerate() {
        if let Some(x) = r.series.get_mut(i * r.series_len + t) {
            *x = *v as f32;
        }
    }
}

/// Write one simulated month into the replay: its series, and its news.
pub fn apply_month(r: &mut Replay, m: &LiveMonth) {
    apply_series(r, m.index, &m.series);
    if !m.events.is_empty() {
        add_news(r, &m.events);
        r.header.events.sort_by(|a, b| a.year.total_cmp(&b.year));
    }
}

/// What the towns paid and charged month by month, for the months a live game
/// has played while the viewer watched. The run's own tables hold a year's
/// averages, which is all a recorded run has; with these the price and wage
/// maps can show a month.
#[derive(Resource, Default)]
pub struct Months {
    /// towns and goods in each month's row
    nt: usize,
    ng: usize,
    /// the place in the run of the first month held
    first: usize,
    wage: Vec<f32>,
    price: Vec<f32>,
}

impl Months {
    pub fn add(&mut self, m: &LiveMonth) {
        let (nt, ng) = (m.wage.len(), if m.wage.is_empty() { 0 } else { m.price.len() / m.wage.len() });
        // the first month, or one that does not follow the last held: start again from it
        if self.wage.is_empty() || (nt, ng) != (self.nt, self.ng) || m.index != self.first + self.wage.len() / nt.max(1) {
            *self = Months { nt, ng, first: m.index, wage: Vec::new(), price: Vec::new() };
        }
        self.wage.extend_from_slice(&m.wage);
        self.price.extend_from_slice(&m.price);
    }

    /// The row of month `t` (its place in the run), if it is held.
    fn row(&self, t: usize) -> Option<usize> {
        let i = t.checked_sub(self.first)?;
        (self.nt > 0 && i < self.wage.len() / self.nt).then_some(i)
    }

    /// The newest month held that is no later than `t`, and no earlier than
    /// the January of its year.
    pub fn at_or_before(&self, t: usize) -> Option<usize> {
        (t - t % 12..=t).rev().find(|&m| self.row(m).is_some())
    }

    pub fn wage(&self, t: usize, town: usize) -> Option<f32> {
        let i = self.row(t)?;
        self.wage.get(i * self.nt + town).copied()
    }

    pub fn price(&self, t: usize, town: usize, good: usize) -> Option<f32> {
        let i = self.row(t)?;
        (town < self.nt && good < self.ng).then(|| self.price[(i * self.nt + town) * self.ng + good])
    }
}

/// Write one simulated year into the replay's arrays.
pub fn apply_year(r: &mut Replay, y: &LiveYear) {
    let k = y.index;
    if k >= r.ny {
        return;
    }
    let (n, ny) = (r.n, r.ny);
    let l = &y.layers;
    for (dst, src) in [
        (&mut r.use_, &l.land_use),
        (&mut r.inten, &l.inten),
        (&mut r.trans, &l.trans),
        (&mut r.catch, &l.catch),
        (&mut r.rent, &l.rent),
        (&mut r.dist, &l.dist),
        (&mut r.mix_a, &l.mix_a),
        (&mut r.mix_b, &l.mix_b),
        (&mut r.mix_s, &l.mix_s),
    ] {
        dst[k * n..(k + 1) * n].copy_from_slice(&src[..n]);
    }

    // towns: the slots are fixed for the run, so the arrays are made on the first year
    let nt = y.towns.active.len();
    if r.n_towns == 0 {
        r.n_towns = nt;
        r.town_active = vec![0; ny * nt];
        r.town_pop = vec![0.0; ny * nt];
        r.town_urban = vec![0.0; ny * nt];
        r.town_wage = vec![0.0; ny * nt];
        r.town_price = vec![0.0; ny * nt * r.header.goods.len()];
    }
    let ng = r.header.goods.len();
    for t in 0..nt.min(r.n_towns) {
        let i = k * r.n_towns + t;
        r.town_active[i] = y.towns.active[t] as u8;
        r.town_pop[i] = y.towns.pop[t];
        r.town_urban[i] = y.towns.urban[t];
        r.town_wage[i] = y.towns.wage[t];
        r.town_price[i * ng..(i + 1) * ng].copy_from_slice(&y.towns.price[t * ng..(t + 1) * ng]);
    }
    for t in &y.new_towns {
        debug_assert_eq!(t.id, r.header.towns.len());
        r.header.towns.push(Town { name: t.name.clone(), node: t.node, founded: t.founded as i32 });
    }

    // firms: every firm gets a whole row of capacities, filled in as the years come
    for f in &y.new_firms {
        debug_assert_eq!(f.id, r.header.firms.len());
        let id = r.header.firms.len();
        r.header.firms.push(Firm {
            name: f.name.clone(),
            method: f.method,
            cell: f.cell,
            town: f.town as i32,
            born: f.born as i32,
            // still at work, as a recorded run writes it
            died: -1,
            reason: serde_json::to_value(&f.reason).ok(),
            owner: f.owner,
        });
        r.firm_first.push(0);
        r.firm_len.push(ny as i32);
        r.firm_off.push((id * ny) as i32);
        r.firm_k.extend(std::iter::repeat_n(0.0, ny));
    }
    for &(f, kk) in &y.firm_K {
        if f < r.header.firms.len() {
            r.firm_k[f * ny + k] = kk;
        }
    }
    for &f in &y.closed_firms {
        if let Some(firm) = r.header.firms.get_mut(f) {
            firm.died = y.year as i32;
        }
    }

    // chronicle: the list is sorted below, so each item remembers its place in
    // the simulation's list, by which marks that have changed since are found
    for &(seq, kept) in &y.rekept {
        if let Some(e) = r.header.events.iter_mut().find(|e| e.seq == seq) {
            e.kept = Some(kept);
        }
    }
    add_news(r, &y.events);
    // in order of date; nothing may keep a place in this list across years (the toasts keep copies)
    r.header.events.sort_by(|a, b| a.year.total_cmp(&b.year));

    // (the monthly series came month by month: `apply_series`)

    if !y.cell_owner.is_empty() {
        if r.owner.is_empty() {
            r.owner = vec![-1; ny * n];
        }
        for (c, &o) in y.cell_owner.iter().take(n).enumerate() {
            r.owner[k * n + c] = o.clamp(-1, 127) as i8;
        }
    }
    r.filled = r.filled.max(k + 1);
}

#[cfg(test)]
#[path = "live_tests.rs"]
mod tests;
