//! A game in progress. The simulation runs in a thread of its own; every year
//! it finishes is written into the same arrays a recorded run fills, so the
//! map, the panels and the timeline cannot tell a live game from a replay.
//!
//! The thread does nothing on its own. The viewer asks for the next year when
//! the clock reaches the last one it has, and sends the player's orders (buy
//! this land, build that works) in between, so an order given while looking at
//! 1786 takes effect in the world before 1787 is worked out.

use std::sync::{mpsc, Mutex};

use bevy::prelude::*;
use isostate_core::families::{self, Group};
use isostate_core::{self as iso, FamilyReport, LiveCursor, LiveStatic, LiveYear, SiteOption};

use crate::family::FamilyView;
use crate::replay::{self, Firm, Good, Header, Meta, Method, Replay, Town};

/// The player's family is always the first.
pub const PLAYER: usize = 0;

pub enum Cmd {
    /// work out the next year
    Step,
    /// what could be done with this cell
    Site(usize),
    Found { method: usize, cell: usize },
    BuyLand(usize),
    Assign { member: usize, group: Option<Group> },
}

pub struct SiteInfo {
    pub cell: usize,
    /// the year the answer holds for (the next one to be simulated)
    pub year: i64,
    pub land_price: f64,
    pub owner: Option<usize>,
    pub options: Result<Vec<SiteOption>, String>,
}

pub enum Msg {
    Year(Box<LiveYear>),
    Site(SiteInfo),
    /// an order was carried out (or not), with the player's family as it now stands
    Done { ok: bool, text: String, family: Option<FamilyReport> },
    Failed(String),
}

#[derive(Resource)]
pub struct Live {
    tx: mpsc::Sender<Cmd>,
    rx: Mutex<mpsc::Receiver<Msg>>,
    /// a year is being worked out
    pub stepping: bool,
    /// the industry group of every method, for the family panel
    pub method_group: Vec<Option<Group>>,
    pub group_labels: Vec<String>,
}

impl Live {
    pub fn send(&self, c: Cmd) {
        // the thread only stops when the app does
        let _ = self.tx.send(c);
    }

    pub fn try_recv(&self) -> Option<Msg> {
        self.rx.lock().ok()?.try_recv().ok()
    }

    pub fn step(&mut self) {
        if !self.stepping {
            self.stepping = true;
            self.send(Cmd::Step);
        }
    }
}

/// Start a new game: the island is made and its first year simulated here
/// (and `advance` more, to start later), then the simulation moves to its own
/// thread. With `demo` the player's family builds a works and buys a little
/// land in the first year, to have something to show in screenshots.
pub fn start(seed: u64, n_families: usize, advance: usize, demo: bool) -> Result<(Replay, Live, FamilyView), String> {
    let mut p = iso::default_params();
    p.seed = seed;
    p.n_families = n_families.max(1);
    let mut s = iso::sim::start(&p, iso::default_content());
    let stat = iso::live_static(&s.st, &s.content);
    let mut cursor = LiveCursor::new(&s.content);
    let mut r = shell(&stat, seed);
    let mut view = FamilyView::default();
    let method_group: Vec<Option<Group>> = stat.methods.iter().map(|m| m.group).collect();
    for i in 0..=advance {
        if s.year > s.end_year {
            break;
        }
        iso::sim::run_year(&mut s)?;
        if i == 0 && demo {
            demo_orders(&mut s);
        }
        let y = iso::live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        apply_year(&mut r, &y);
        crate::game::year_families(&mut view, &y.families, y.year as i32, &r, &method_group);
    }

    let (tx_cmd, rx_cmd) = mpsc::channel::<Cmd>();
    let (tx_msg, rx_msg) = mpsc::channel::<Msg>();
    std::thread::Builder::new()
        .name("isostate-sim".into())
        .spawn(move || run_thread(s, cursor, rx_cmd, tx_msg))
        .map_err(|e| format!("cannot start the simulation thread: {e}"))?;
    let live = Live {
        tx: tx_cmd,
        rx: Mutex::new(rx_msg),
        stepping: false,
        method_group,
        group_labels: stat.groups.iter().map(|g| g.label.clone()).collect(),
    };
    Ok((r, live, view))
}

/// For screenshots: the best-paying works near home that costs at most half
/// the family's cash, and the cheapest few squares around it.
fn demo_orders(s: &mut iso::Sim) {
    let Ok(rep) = iso::family_report(&s.st, &s.content, &s.params, PLAYER) else { return };
    let home = s.st.town_node[rep.home];
    let (w, h) = (s.st.world.W as i64, s.st.world.H as i64);
    let (hx, hy) = ((home % s.st.world.W) as i64, (home / s.st.world.W) as i64);
    let near: Vec<usize> = (-2..=2)
        .flat_map(|dy| (-2..=2).map(move |dx| (hx + dx, hy + dy)))
        .filter(|&(x, y)| x >= 0 && y >= 0 && x < w && y < h && (x, y) != (hx, hy))
        .map(|(x, y)| (y * w + x) as usize)
        .collect();
    let mut best: Option<(f64, usize, usize)> = None;
    for &c in &near {
        let Ok(opts) = iso::site_options(&s.st, &s.content, &s.params, PLAYER, c) else { continue };
        for o in opts {
            if let Ok(sums) = o.sums {
                if o.name != "services" && sums.capital_cost <= rep.cash * 0.5 && best.is_none_or(|b| sums.expected_return > b.0) {
                    best = Some((sums.expected_return, o.method, c));
                }
            }
        }
    }
    let year = s.year;
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
    if let Some((_, m, c)) = best {
        if let Err(e) = iso::found_works(&mut s.st, &s.content, &s.params, PLAYER, m, c, year, &mut s.events) {
            eprintln!("demo: could not build: {e}");
        }
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
            let _ = iso::buy_land(&mut s.st, &s.content, &s.params, PLAYER, c);
        }
    }
}

fn run_thread(mut s: iso::Sim, mut cursor: LiveCursor, rx: mpsc::Receiver<Cmd>, tx: mpsc::Sender<Msg>) {
    while let Ok(cmd) = rx.recv() {
        let msg = match cmd {
            Cmd::Step => {
                if s.year > s.end_year {
                    Msg::Failed(format!("{} was the last year.", s.end_year))
                } else {
                    match iso::sim::run_year(&mut s) {
                        Ok(()) => Msg::Year(Box::new(iso::live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events))),
                        Err(e) => Msg::Failed(e),
                    }
                }
            }
            Cmd::Site(cell) => Msg::Site(SiteInfo {
                cell,
                year: s.year,
                land_price: families::land_price(&s.st, &s.params, cell),
                owner: families::owner_of_cell(&s.st, cell),
                options: iso::site_options(&s.st, &s.content, &s.params, PLAYER, cell),
            }),
            Cmd::Found { method, cell } => {
                let year = s.year;
                let r = iso::found_works(&mut s.st, &s.content, &s.params, PLAYER, method, cell, year, &mut s.events);
                done(&s, r.map(|f| format!("{} will open in {}.", s.st.f_name[f], year)))
            }
            Cmd::BuyLand(cell) => {
                let r = iso::buy_land(&mut s.st, &s.content, &s.params, PLAYER, cell);
                done(&s, r.map(|price| format!("The land is yours, for {}.", crate::ui::money(price as f32))))
            }
            Cmd::Assign { member, group } => {
                let r = iso::assign_member(&mut s.st, PLAYER, member, group);
                done(&s, r.map(|()| "Done.".to_string()))
            }
        };
        if tx.send(msg).is_err() {
            return;
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
            died: 0,
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
    r.header.events.sort_by(|a, b| a.year.partial_cmp(&b.year).unwrap_or(std::cmp::Ordering::Equal));

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
