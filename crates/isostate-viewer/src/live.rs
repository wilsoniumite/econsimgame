//! A game in progress. The simulation runs in a thread of its own; every year
//! it finishes is written into the same arrays a recorded run fills, so the
//! map, the panels and the timeline cannot tell a live game from a replay.
//!
//! The thread does nothing on its own. The viewer asks for the next year when
//! the clock reaches the last one it has, and sends the player's orders (buy
//! this land, build that works) in between, so an order given while looking at
//! 1786 takes effect in the world before 1787 is worked out.
//!
//! The thread also keeps the game saved. Every order it carries out goes in a
//! log, and the log is written to the island's autosave after every year and
//! every order (`saves`). Going on with a saved game plays its years again
//! here, before the window opens, giving each order where it was first given.

use std::sync::{mpsc, Mutex};

use bevy::prelude::*;
use isostate_core::families::{self, Group, Posting, Stance, N_GROUPS};
use isostate_core::save::{Done, Game, Order};
use isostate_core::{self as iso, FamilyReport, LiveCursor, LiveStatic, LiveYear, Prospect, SiteOption};

use crate::family::FamilyView;
use crate::replay::{self, Firm, Good, Header, Meta, Method, Replay, Town};
use crate::saves::{self, Saver};

/// The player's family is always the first.
pub const PLAYER: usize = 0;

pub enum Cmd {
    /// work out the next year
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
    /// a year, with what each trade would earn in each town (`families::prospect_works`)
    Year(Box<LiveYear>, Vec<[Prospect; N_GROUPS]>),
    Site(SiteInfo),
    /// an order was carried out (or not), with the player's family as it now stands
    Done { ok: bool, text: String, family: Option<FamilyReport> },
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
    /// a year is being worked out
    pub stepping: bool,
    /// the newest year (as an index) the clock has shown. The next is asked
    /// for a little before it is due, so for a moment there is a year worked
    /// out that the player has not seen; see `game::Now`.
    pub seen: usize,
    /// when the year being worked out was asked for
    sent: Option<std::time::Instant>,
    /// how long the last year took to arrive, in seconds
    pub took: Option<f32>,
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
    /// orders given again
    pub orders: usize,
    /// orders the simulation refused this time: its rules have changed since the save
    pub refused: usize,
    /// the island does not stand as the save says it stood
    pub drifted: bool,
    /// newer saves that could not be read, so that this is an earlier one
    pub passed_over: usize,
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

    pub fn step(&mut self) {
        if !self.stepping {
            self.stepping = true;
            self.sent = Some(std::time::Instant::now());
            self.send(Cmd::Step);
        }
    }

    /// The year asked for has come (or failed to).
    pub fn stepped(&mut self, arrived: bool) {
        self.stepping = false;
        if let (true, Some(t0)) = (arrived, self.sent.take()) {
            self.took = Some(t0.elapsed().as_secs_f32());
        }
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

    /// Take in the year the simulation has just played.
    fn year(&mut self, sim: &iso::Sim) {
        let y = iso::live_year(&mut self.cursor, &sim.st, &sim.content, &sim.params, &sim.events);
        apply_year(&mut self.r, &y);
        crate::game::year_families(&mut self.view, &y.families, y.year as i32);
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
            eprintln!("  back in {} after {:.1}s", loaded.game.last_year(), t0.elapsed().as_secs_f32());
            resumed = Some(Resumed {
                year: loaded.game.last_year() as i32,
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
        game.year()?;
        board.year(&game.sim);
        if new && i == 0 && o.demo {
            demo_orders(&mut game);
        }
    }
    // years played on after the saved ones: the card says the year the game opens in
    if let Some(back) = &mut resumed {
        back.year = game.last_year() as i32;
    }
    let Board { cursor, r, mut view } = board;
    // orders given since the last year: the family as it stands now, and the works to come
    if let Ok(rep) = iso::family_report(&game.sim.st, &game.sim.content, &game.sim.params, PLAYER) {
        crate::game::player_family(&mut view, &rep);
    }
    let ordered = game
        .orders
        .iter()
        .filter(|g| g.year == game.last_year())
        .filter_map(|g| if let Order::Found { cell, .. } = &g.order { Some(*cell) } else { None })
        .collect();
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

    let (tx_cmd, rx_cmd) = mpsc::channel::<Cmd>();
    let (tx_msg, rx_msg) = mpsc::channel::<Msg>();
    let thread = std::thread::Builder::new()
        .name("isostate-sim".into())
        .spawn(move || run_thread(game, cursor, saver, rx_cmd, tx_msg))
        .map_err(|e| format!("cannot start the simulation thread: {e}"))?;
    let live = Live {
        tx: Some(tx_cmd),
        rx: Mutex::new(rx_msg),
        thread: Some(thread),
        stepping: false,
        // a game opens on the newest year there is
        seen: r.filled.max(1) - 1,
        sent: None,
        took: None,
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

/// A command as the order the log keeps. Asking for a year or about a square is not one.
fn order_of(sim: &iso::Sim, cmd: Cmd) -> Result<Order, String> {
    match cmd {
        Cmd::Step | Cmd::Site(_) => Err("that is not an order".to_string()),
        Cmd::Found { method, cell } => Order::found(sim, method, cell),
        Cmd::BuyLand(cell) => Ok(Order::BuyLand { cell }),
        Cmd::Decide { id, option } => Order::decide(sim, id, option),
        Cmd::Post { member, posting } => Ok(Order::Post { member, posting }),
        Cmd::Stance { group, town, stance } => Ok(Order::Stance { group, town, stance }),
        Cmd::MoveSeat(town) => Ok(Order::MoveSeat { town }),
    }
}

fn run_thread(mut game: Game, mut cursor: LiveCursor, mut saver: Option<Saver>, rx: mpsc::Receiver<Cmd>, tx: mpsc::Sender<Msg>) {
    // whether the last save was written, so a failure is told once and not every year
    let mut saved = true;
    while let Ok(cmd) = rx.recv() {
        let mut year_end = false;
        let msg = match cmd {
            Cmd::Step => {
                if game.over() {
                    Msg::Failed(format!("{} was the last year.", game.sim.end_year))
                } else {
                    match game.year() {
                        Ok(()) => {
                            year_end = true;
                            let s = &game.sim;
                            let y = iso::live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
                            Msg::Year(Box::new(y), all_prospects(s))
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
            order => {
                // carried out, and kept in the log if it was
                let r = order_of(&game.sim, order).and_then(|o| game.give(o));
                let s = &game.sim;
                let text = r.map(|d| match d {
                    Done::Founded(f) => format!("{} will open in {}.", s.st.f_name[f], s.year),
                    Done::Bought(price) => format!("The land is yours, for {}.", crate::ui::money(price as f32)),
                    Done::Done => String::new(),
                });
                done(s, text)
            }
        };
        // the game is kept saved: after every year, and after every order carried out
        let changed = year_end || matches!(msg, Msg::Done { ok: true, .. });
        if tx.send(msg).is_err() {
            return;
        }
        if let (true, Some(saver)) = (changed, &mut saver) {
            let now = saver.write(&game.save(saves::now()), year_end);
            if let Err(e) = &now {
                eprintln!("the game could not be saved: {e}");
            }
            if now.is_ok() != saved {
                saved = now.is_ok();
                if tx.send(Msg::Unsaved(now.err())).is_err() {
                    return;
                }
            }
        }
    }
}

fn done(s: &iso::Sim, r: Result<String, String>) -> Msg {
    let family = iso::family_report(&s.st, &s.content, &s.params, PLAYER).ok();
    match r {
        Ok(text) => Msg::Done { ok: true, text, family },
        Err(text) => Msg::Done { ok: false, text, family },
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
        mix_a: vec![0; ny * n],
        mix_b: vec![0; ny * n],
        mix_s: vec![255; ny * n],
        town_active: Vec::new(),
        town_pop: Vec::new(),
        town_urban: Vec::new(),
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
    }
    for t in 0..nt.min(r.n_towns) {
        let i = k * r.n_towns + t;
        r.town_active[i] = y.towns.active[t] as u8;
        r.town_pop[i] = y.towns.pop[t];
        r.town_urban[i] = y.towns.urban[t];
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

    for e in &y.events {
        r.header.events.push(replay::Event {
            year: e.year as f32,
            kind: e.kind.clone(),
            weight: e.weight as f32,
            title: e.title.clone(),
            text: e.text.clone(),
            cell: e.cell.map(|c| c as i64),
            family: e.family,
        });
    }
    // in order of date; nothing may keep a place in this list across years (the toasts keep copies)
    r.header.events.sort_by(|a, b| a.year.total_cmp(&b.year));

    // the monthly series arrive as yearly means: every month of the year gets the mean
    for (i, v) in y.series.iter().enumerate() {
        let row = i * r.series_len;
        for m in 0..12 {
            r.series[row + k * 12 + m] = *v as f32;
        }
    }

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
