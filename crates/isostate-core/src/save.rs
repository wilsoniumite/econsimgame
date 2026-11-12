//! Saved games, as a log of orders. Not in the Python.
//!
//! The state of a run cannot be written out as it stands (`State`, the random
//! streams and the observer have no serialised form), but a run is decided by
//! its seed, its rules and the orders the player gave between years. So a save
//! holds those and nothing else, and loading plays the years again, giving each
//! order at the point it was first given.
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

use crate::families::{self, Choice, Group, Posting, Stance};
use crate::params::{Params, Rules};
use crate::sim::{self, Sim};

/// The layout of the file. A reader refuses a later one.
pub const FORMAT: u32 = 1;

/// The player's family is always the first.
pub const PLAYER: usize = 0;

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
}

/// What an order that was carried out did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Done {
    /// The new works' firm id.
    Founded(usize),
    /// The price paid for the land.
    Bought(f64),
    Done,
}

/// Carry out an order for the player's family. A refused order changes nothing.
pub fn carry_out(sim: &mut Sim, order: &Order) -> Result<Done, String> {
    let Sim { st, content: C, params: P, events, year, .. } = sim;
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
                Some(o) => families::decide(st, C, PLAYER, *id, o).map(|()| Done::Done),
                None => Err(format!("decision {id} no longer offers that choice")),
            }
        }
        Order::Post { member, posting } => families::post_member(st, PLAYER, *member, *posting).map(|()| Done::Done),
        Order::Stance { group, town, stance } => families::set_stance(st, PLAYER, *group, *town, *stance).map(|()| Done::Done),
        Order::MoveSeat { town } => families::move_seat(st, PLAYER, *town).map(|()| Done::Done),
    }
}

/// An order and the year it was given in: the last year played at the time,
/// so it took effect before the next one was worked out.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Given {
    pub year: i64,
    #[serde(flatten)]
    pub order: Order,
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
    let mut content = rules.content();
    content.good_index.clear();
    h.bytes(format!("{content:?}").as_bytes());
    RulesId { name: rules.as_str().to_string(), fingerprint: h.hex() }
}

/// The parameters of a game on island `seed` with `families` families.
pub fn params(rules: Rules, seed: u64, families: usize) -> Params {
    let mut P = rules.params();
    P.seed = seed;
    P.n_families = families.max(1);
    P
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
}

impl Game {
    /// A new game under the game's rules, before its first year.
    pub fn new(seed: u64, families: usize, started: u64) -> Game {
        Game::under(Rules::Game, seed, families, started)
    }

    /// A new game under these rules.
    pub fn under(rules: Rules, seed: u64, families: usize, started: u64) -> Game {
        Game { sim: sim::start(&params(rules, seed, families), rules.content()), rules, orders: Vec::new(), started }
    }

    /// The last year played (the year before the first, if none has been).
    pub fn last_year(&self) -> i64 {
        self.sim.year - 1
    }

    /// Whether every year has been played.
    pub fn over(&self) -> bool {
        self.sim.year > self.sim.end_year
    }

    /// Play the next year.
    pub fn year(&mut self) -> Result<(), String> {
        sim::run_year(&mut self.sim)
    }

    /// Carry out an order and write it in the log. A refused order is not kept.
    pub fn give(&mut self, order: Order) -> Result<Done, String> {
        let done = carry_out(&mut self.sim, &order)?;
        self.orders.push(Given { year: self.last_year(), order });
        Ok(done)
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
            family: me.as_ref().map(|r| r.surname.clone()).unwrap_or_default(),
            seat: me.as_ref().map(|r| r.seat_name.clone()).unwrap_or_default(),
            worth: me.as_ref().map(|r| r.net_worth.round() as i64).unwrap_or(0),
            check: check(&self.sim),
            orders: self.orders.clone(),
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
    pub orders: Vec<Given>,
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
        serde_json::from_value(value).map_err(|e| format!("not a saved game: {e}"))
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

/// Play a saved game again: the years up to the one it was saved in, with
/// every order given where it was first given. `each_year` is called after
/// each year, before the orders given in it (as a viewer gets a year before
/// the player answers it).
pub fn load(save: &Save, mut each_year: impl FnMut(&Sim)) -> Result<Loaded, String> {
    if save.format > FORMAT {
        return Err(format!("saved by a later version of the game (layout {}; this one reads up to {FORMAT})", save.format));
    }
    let mut game = Game::under(save.played_under(), save.seed, save.families, save.started);
    let mut refused = Vec::new();
    let mut next = 0;
    let until = save.year.min(game.sim.end_year);
    loop {
        // the orders given while the last year played was on the table
        while next < save.orders.len() && save.orders[next].year < game.sim.year {
            let given = &save.orders[next];
            next += 1;
            match carry_out(&mut game.sim, &given.order) {
                Ok(_) => game.orders.push(Given { year: game.last_year(), order: given.order.clone() }),
                Err(why) => refused.push((given.clone(), why)),
            }
        }
        if game.sim.year > until {
            break;
        }
        game.year()?;
        each_year(&game.sim);
    }
    // orders dated after the year the save reached cannot have been given
    for given in &save.orders[next..] {
        refused.push((given.clone(), format!("it is dated {}, after the save's last year", given.year)));
    }
    let drifted = !save.check.is_empty() && check(&game.sim) != save.check;
    Ok(Loaded { game, refused, drifted })
}

/// A fingerprint of a run so far: every month of every series, every family's
/// cash and seat, and who owns each cell and each works. Two runs with the
/// same fingerprint have had the same history, to the last digit.
pub fn check(sim: &Sim) -> String {
    let st = &sim.st;
    let mut h = Fnv::new();
    h.word(sim.year as u64);
    for series in &st.series.data {
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
