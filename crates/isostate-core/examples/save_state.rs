//! A probe for docs/probes/dx-save-state.md: what the island's state is, what it
//! weighs, what changes and when, and whether a rough picture of it written to
//! bytes and read back goes on as the same game to the last digit.
//!
//!     cargo run --release -p isostate-core --example save_state -- [--seeds 7,3] [--out DIR]
//!         [--families 6] [--round-trip 1849] [--ablate] [--no-change]
//!
//! For each island it plays a game under the game's rules from 1750 to 1900 a
//! month at a time, as `save::Game` does, with no orders. It prints:
//!
//! - the size of every part of the state in memory (what a deep copy of it
//!   allocates) and written raw, in 1750 (before the first month), at the turns
//!   into 1800 and 1850, and at the end of 1900;
//! - how often each part changes: in the months of eight sample years (1751,
//!   1752, 1800, 1801, 1850, 1851, 1899, 1900), at each year's end, and whether
//!   it ever changes after 1750;
//! - how long the century took to play (what loading a save costs today), and
//!   how long the rough writer and reader take;
//! - at the turn into the year after `--round-trip` and six months into it: the
//!   state written to bytes and read back, then both played on ten years and
//!   compared by `save::check` and byte for byte;
//! - with `--ablate`, at the same turn, each part left out of the picture in
//!   turn (put back at its default), the game played on two years, and whether
//!   it is still the same game;
//! - whether the tables worked out from others (the transport, the counts of
//!   works, the island itself) come out the same when worked out again.
//!
//! With `--out` the raw picture of each island at the 1850 turn and at the end
//! is written there (`state-7-1850.bin` and so on), to be packed by `gzip` and
//! `xz` from the shell.
//!
//! The writer here is rough: every number as its eight (or four, or one) bytes,
//! little-endian, every list with its length before it. It reaches every field
//! an example can reach. Three things it cannot: the families' private
//! fields, most of the books of account, and the inside of a random stream.
//! The families and the books are carried across by a copy in memory and only
//! weighed; a random stream is written as how many numbers it has given, found
//! by drawing from a fresh stream of the same seed until the two agree, and read
//! back by drawing that many again.
#![allow(non_snake_case)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::Instant;

use isostate_core::canals::{CanalState, CanalYear, Link};
use isostate_core::chronicle::{ChronicleState, Why};
use isostate_core::engel::EngelState;
use isostate_core::landcells::Landform;
use isostate_core::mat::Mat;
use isostate_core::mines::Working;
use isostate_core::observer::{ObserverState, Series, SeriesMeta};
use isostate_core::peopling::Founding;
use isostate_core::record::{ProjectSnap, Record};
use isostate_core::rng::{self, Rng, Rngs};
use isostate_core::save::{self, Game};
use isostate_core::state::{MonthStats, Project, RailOpened, Reason};
use isostate_core::transport::Graph;
use isostate_core::ways::{Line, WayBook, YearNote};
use isostate_core::world::World;
use isostate_core::*;

// ---------------------------------------------------------------- weighing what is allocated

/// Counts the bytes allocated and not yet freed, so that a deep copy of a part
/// says what the part holds on the heap.
struct Counting;

static LIVE: AtomicIsize = AtomicIsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        LIVE.fetch_add(l.size() as isize, Ordering::Relaxed);
        unsafe { System.alloc(l) }
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        LIVE.fetch_add(l.size() as isize, Ordering::Relaxed);
        unsafe { System.alloc_zeroed(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size() as isize, Ordering::Relaxed);
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        LIVE.fetch_add(new as isize - l.size() as isize, Ordering::Relaxed);
        unsafe { System.realloc(p, l, new) }
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;

/// What a deep copy of `x` allocates, plus the part itself where it sits.
fn weigh<T: Clone>(x: &T) -> usize {
    let before = LIVE.load(Ordering::Relaxed);
    let copy = x.clone();
    let after = LIVE.load(Ordering::Relaxed);
    drop(copy);
    (after - before).max(0) as usize + std::mem::size_of::<T>()
}

// ---------------------------------------------------------------- the rough writer and reader

trait Bin: Sized {
    fn put(&self, w: &mut Vec<u8>);
    fn get(r: &mut &[u8]) -> Self;
}

fn take<'a>(r: &mut &'a [u8], n: usize) -> &'a [u8] {
    let (a, b) = r.split_at(n);
    *r = b;
    a
}

macro_rules! bin_num {
    ($($t:ty),*) => {$(
        impl Bin for $t {
            fn put(&self, w: &mut Vec<u8>) {
                w.extend_from_slice(&self.to_le_bytes());
            }
            fn get(r: &mut &[u8]) -> Self {
                <$t>::from_le_bytes(take(r, std::mem::size_of::<$t>()).try_into().unwrap())
            }
        }
    )*};
}
bin_num!(f64, f32, i64, i32, u64, u32, u8);

impl Bin for usize {
    fn put(&self, w: &mut Vec<u8>) {
        (*self as u64).put(w);
    }
    fn get(r: &mut &[u8]) -> Self {
        u64::get(r) as usize
    }
}

impl Bin for bool {
    fn put(&self, w: &mut Vec<u8>) {
        w.push(*self as u8);
    }
    fn get(r: &mut &[u8]) -> Self {
        take(r, 1)[0] != 0
    }
}

impl Bin for String {
    fn put(&self, w: &mut Vec<u8>) {
        self.len().put(w);
        w.extend_from_slice(self.as_bytes());
    }
    fn get(r: &mut &[u8]) -> Self {
        let n = usize::get(r);
        String::from_utf8(take(r, n).to_vec()).unwrap()
    }
}

impl<T: Bin> Bin for Vec<T> {
    fn put(&self, w: &mut Vec<u8>) {
        self.len().put(w);
        for x in self {
            x.put(w);
        }
    }
    fn get(r: &mut &[u8]) -> Self {
        let n = usize::get(r);
        (0..n).map(|_| T::get(r)).collect()
    }
}

impl<T: Bin> Bin for Option<T> {
    fn put(&self, w: &mut Vec<u8>) {
        match self {
            None => w.push(0),
            Some(x) => {
                w.push(1);
                x.put(w);
            }
        }
    }
    fn get(r: &mut &[u8]) -> Self {
        if take(r, 1)[0] == 0 { None } else { Some(T::get(r)) }
    }
}

impl<T: Bin, const K: usize> Bin for [T; K] {
    fn put(&self, w: &mut Vec<u8>) {
        for x in self {
            x.put(w);
        }
    }
    fn get(r: &mut &[u8]) -> Self {
        std::array::from_fn(|_| T::get(r))
    }
}

impl<A: Bin, B: Bin> Bin for (A, B) {
    fn put(&self, w: &mut Vec<u8>) {
        self.0.put(w);
        self.1.put(w);
    }
    fn get(r: &mut &[u8]) -> Self {
        let a = A::get(r);
        (a, B::get(r))
    }
}

impl<A: Bin, B: Bin, C: Bin> Bin for (A, B, C) {
    fn put(&self, w: &mut Vec<u8>) {
        self.0.put(w);
        self.1.put(w);
        self.2.put(w);
    }
    fn get(r: &mut &[u8]) -> Self {
        let a = A::get(r);
        let b = B::get(r);
        (a, b, C::get(r))
    }
}

impl<K: Bin + Ord, V: Bin> Bin for BTreeMap<K, V> {
    fn put(&self, w: &mut Vec<u8>) {
        self.len().put(w);
        for (k, v) in self {
            k.put(w);
            v.put(w);
        }
    }
    fn get(r: &mut &[u8]) -> Self {
        let n = usize::get(r);
        (0..n).map(|_| <(K, V)>::get(r)).collect()
    }
}

impl<K: Bin + Ord> Bin for BTreeSet<K> {
    fn put(&self, w: &mut Vec<u8>) {
        self.len().put(w);
        for k in self {
            k.put(w);
        }
    }
    fn get(r: &mut &[u8]) -> Self {
        let n = usize::get(r);
        (0..n).map(|_| K::get(r)).collect()
    }
}

/// The new land's island, which is off under the game's rules: never written.
impl Bin for Box<Landform> {
    fn put(&self, _w: &mut Vec<u8>) {
        panic!("this rough writer does not write the new land's island");
    }
    fn get(_r: &mut &[u8]) -> Self {
        panic!("this rough writer does not read the new land's island");
    }
}

/// A struct, field by field. The reader is a struct literal, so a field left
/// out of the list is an error at compile time: the list is the whole struct.
macro_rules! bin_struct {
    ($t:ident { $($f:ident),* $(,)? }) => {
        impl Bin for $t {
            fn put(&self, w: &mut Vec<u8>) {
                $( self.$f.put(w); )*
            }
            fn get(r: &mut &[u8]) -> Self {
                $t { $( $f: Bin::get(r), )* }
            }
        }
    };
}

bin_struct!(Mat { rows, cols, data });
bin_struct!(Reason { kind, year, town, cell, K, roi, margin, pout, inputs, heat, labor, rent, displaced, agglo, dist, n_cand });
bin_struct!(Project { a, b, spur, target, town, cells, new, need, order, cash, start, roi, cost, length_km, way, months, serves, paid_in, subs, wages, navvy_pay });
bin_struct!(RailOpened { year, a, b, km });
bin_struct!(Why { test, series, value, threshold, baseline, note });
bin_struct!(Event { year, kind, clock, weight, title, text, span, series, town, cell, cell2, tech, firm, family, why, kept });
bin_struct!(MonthStats { prod, method_out, hh_got, lord_got, share, hh_spend, lord_spend, urban_rent, employment, ex_labor_by_sector, firm_labor_by_method, heat_use, supply });
bin_struct!(World { W, H, land, h, terrain, down, acc, river, navigable, coast, fertility, pasture, forest_pot, rock, clay, ore, coal, jitter, town_sites, forest0, landform });
bin_struct!(Graph { N, W, H, w, railed, min_w, max_w, boat, wharf });
bin_struct!(SeriesMeta { label, group, unit });
bin_struct!(ObserverState { cross_state, milestones, last_alarm, peak, in_slump, slump_start, trough, first_output, base });
bin_struct!(EngelState { need_spend, real_income, chain, base_p, base_q, acc_p, acc_q, months });
bin_struct!(ChronicleState {
    kinds, reached, branch_opened, branch_extended, city_passed, pop, urban, coal, wage, moved, prices, peak, decline_told, leaders, challenger, lead_years, held,
    held_since, ousted, fell, told, redrawn
});
bin_struct!(Line { start, opened, serves, a, b, target, new_cells, km, cost, roi, spent });
bin_struct!(YearNote { year, km, fares, capital, building });
bin_struct!(WayBook { links, pair_cooldown, spot_cooldown, lines, unreported, years });
bin_struct!(Link { a, b, begun, opened });
bin_struct!(CanalYear { year, boarded, ton_cells, trade, land, works, busy_cells, cells, navvies });
bin_struct!(CanalState { book, links, traffic, pair_cells, trade_acc, years, navvies, navvy_acc, know, found, rng });
bin_struct!(Founding { town, year, kind, port, rail, cells, people, saving });
bin_struct!(Working { cell, sector, opened, left, hands, out, heap, surface, royalty, cells });
bin_struct!(ProjectSnap { a, b, done });
bin_struct!(Record { n_years, N, T, layers, town_active, town_pop, town_urban, town_wage, town_y, town_price, town_know, firm_K, projects, town_harvest, harvest, town_layers, workings });

/// The series: names, labels and every month so far. The index by name is a
/// `HashMap`, whose order is not the same twice; it is made again from the names.
impl Bin for Series {
    fn put(&self, w: &mut Vec<u8>) {
        self.names.put(w);
        self.meta.put(w);
        self.data.put(w);
    }
    fn get(r: &mut &[u8]) -> Self {
        let names: Vec<String> = Bin::get(r);
        let meta = Bin::get(r);
        let data = Bin::get(r);
        let index: HashMap<String, usize> = names.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();
        Series { names, meta, data, index }
    }
}

// ---------------------------------------------------------------- random streams

thread_local! {
    /// The island's seed, and which of its streams the next `Rng` written or read is.
    static SEED: Cell<u64> = const { Cell::new(0) };
    static STREAM: Cell<u8> = const { Cell::new(0) };
    /// Time spent finding where streams stand, which a real writer would not spend.
    static SEARCH: Cell<f64> = const { Cell::new(0.0) };
}

fn set_stream(k: u8) {
    STREAM.with(|s| s.set(k));
}

/// Stream `k` of the island as it stands before its first draw: 0 to 7 are
/// `make_rngs`' (world, names, firms, tech, people, rail, misc, harvest), 8 the
/// people rules' and 9 the canals'.
fn fresh(seed: u64, k: u8) -> Rng {
    let r = rng::make_rngs(seed);
    match k {
        0 => r.world,
        1 => r.names,
        2 => r.firms,
        3 => r.tech,
        4 => r.people,
        5 => r.rail,
        6 => r.misc,
        7 => r.harvest,
        8 => rng::people_stream(seed),
        9 => rng::canal_stream(seed),
        _ => panic!("no stream {k}"),
    }
}

/// How many numbers a stream has given since it was made: a fresh copy is
/// drawn from until its next three numbers are the stream's next three.
fn position(cur: &Rng, mut cand: Rng) -> u64 {
    let mut c = cur.clone();
    let target = [rng::random(&mut c).to_bits(), rng::random(&mut c).to_bits(), rng::random(&mut c).to_bits()];
    let mut window = [0u64; 3];
    for k in 0..u64::MAX {
        window = [window[1], window[2], rng::random(&mut cand).to_bits()];
        if k >= 2 && window == target {
            return k - 2;
        }
    }
    unreachable!()
}

impl Bin for Rng {
    fn put(&self, w: &mut Vec<u8>) {
        let t0 = Instant::now();
        let k = STREAM.with(|s| s.get());
        let n = position(self, fresh(SEED.with(|s| s.get()), k));
        SEARCH.with(|s| s.set(s.get() + t0.elapsed().as_secs_f64()));
        k.put(w);
        n.put(w);
    }
    fn get(r: &mut &[u8]) -> Self {
        let k = u8::get(r);
        let n = u64::get(r);
        let mut g = fresh(SEED.with(|s| s.get()), k);
        for _ in 0..n {
            rng::random(&mut g);
        }
        g
    }
}

fn put_rngs(r: &Rngs, w: &mut Vec<u8>) {
    for (k, g) in [&r.world, &r.names, &r.firms, &r.tech, &r.people, &r.rail, &r.misc, &r.harvest].into_iter().enumerate() {
        set_stream(k as u8);
        g.put(w);
    }
}

fn get_rngs(r: &mut &[u8]) -> Rngs {
    let mut next = || Rng::get(r);
    Rngs { world: next(), names: next(), firms: next(), tech: next(), people: next(), rail: next(), misc: next(), harvest: next() }
}

/// How many numbers each of the eight streams of `make_rngs` has given.
fn positions(r: &Rngs, seed: u64) -> Vec<u64> {
    [&r.world, &r.names, &r.firms, &r.tech, &r.people, &r.rail, &r.misc, &r.harvest]
        .into_iter()
        .enumerate()
        .map(|(k, g)| position(g, fresh(seed, k as u8)))
        .collect()
}

// ---------------------------------------------------------------- the state, part by part

/// One part of the state: its name, the heading it is under in state.rs, its
/// size in memory and written, and a fingerprint of what it holds.
struct Part {
    name: &'static str,
    group: &'static str,
    mem: usize,
    raw: usize,
    hash: u64,
}

fn fnv(bytes: &[u8]) -> u64 {
    // FNV-1a over eight bytes at a time: a fingerprint, not a check against tampering
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut chunks = bytes.chunks_exact(8);
    for c in &mut chunks {
        h = (h ^ u64::from_le_bytes(c.try_into().unwrap())).wrapping_mul(0x0000_0100_0000_01b3);
    }
    for &b in chunks.remainder() {
        h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn part<T: Bin + Clone>(name: &'static str, group: &'static str, x: &T, mem: bool) -> Part {
    let mut w = Vec::new();
    x.put(&mut w);
    Part { name, group, mem: if mem { weigh(x) } else { 0 }, raw: w.len(), hash: fnv(&w) }
}

/// A part the writer cannot reach into: weighed, and fingerprinted by how it prints.
fn part_debug<T: std::fmt::Debug + Clone>(name: &'static str, group: &'static str, x: &T, mem: usize, raw: usize) -> Part {
    Part { name, group, mem, raw, hash: fnv(format!("{x:?}").as_bytes()) }
}

macro_rules! state_parts {
    ($($f:ident : $g:literal),* $(,)?) => {
        /// Every part the writer reaches, in the order it writes them; then the two
        /// streams in the state; the families and the books are not written.
        fn put_state(st: &State, w: &mut Vec<u8>) {
            $( st.$f.put(w); )*
            set_stream(8);
            st.pp_rng.put(w);
            set_stream(9);
            st.cn.put(w);
        }

        /// The state read back. The families and the books are left empty, for
        /// the caller to carry across.
        fn get_state(r: &mut &[u8]) -> State {
            State {
                $( $f: Bin::get(r), )*
                pp_rng: { set_stream(8); Bin::get(r) },
                cn: { set_stream(9); Bin::get(r) },
                families: Vec::new(),
                acct: Default::default(),
            }
        }

        fn parts(st: &State, seed: u64, mem: bool) -> Vec<Part> {
            SEED.with(|s| s.set(seed));
            let mut out = vec![ $( part(stringify!($f), $g, &st.$f, mem), )* ];
            set_stream(8);
            out.push(part("pp_rng", "people", &st.pp_rng, mem));
            set_stream(9);
            out.push(part("cn", "canals", &st.cn, mem));
            // the families: their public part as the game's JSON, for a size
            let fam_raw = serde_json::to_vec(&st.families).map(|v| v.len()).unwrap_or(0);
            out.push(part_debug("families", "families", &st.families, if mem { weigh(&st.families) } else { 0 }, fam_raw));
            // the books: their history sits behind an `Arc`, which a copy shares, so it is weighed apart
            let books = if mem { weigh(&st.acct) + weigh(&*st.acct.hist) } else { 0 };
            out.push(part_debug("acct", "accounts", &st.acct, books, books));
            out
        }

        const NAMES: &[&str] = &[ $( stringify!($f), )* ];

        /// Put one part back at its default, as if it had been left out of the picture.
        fn ablate(st: &mut State, name: &str) {
            match name {
                $( stringify!($f) => st.$f = Default::default(), )*
                "pp_rng" => st.pp_rng = None,
                "cn" => st.cn = Default::default(),
                _ => panic!("no part {name}"),
            }
        }
    };
}

state_parts! {
    world: "world", N: "world", T_max: "world",
    town_active: "towns", town_node: "towns", town_founded: "towns", town_new: "towns", town_name: "towns", used_names: "towns", pop: "towns",
    land_use: "cells", cover: "cells", road: "cells", rail: "cells", road_quiet: "cells", firm_at: "cells", cell_K_share: "cells", land_rent_only: "cells",
    derelict_timer: "cells", cell_rent: "cells", cell_flow: "cells", traffic: "cells", urban: "cells", urban_res: "cells", water: "cells",
    F_count: "firms", f_alive: "firms", f_method: "firms", f_cell: "firms", f_town: "firms", f_born: "firms", f_died: "firms", f_loss_years: "firms",
    f_parent: "firms", f_K: "firms", f_order_K: "firms", f_cash: "firms", f_rent: "firms", f_u: "firms", f_fill: "firms", f_prod: "firms",
    f_profit_acc: "firms", f_out_acc: "firms", f_cost_m: "firms", f_labor: "firms", f_order: "firms", f_name: "firms", f_reason: "firms",
    new_firms_year: "firms", closed_year: "firms",
    hh_cash: "money", lord_cash: "money", merch_cash: "money", hh_income: "money", lord_income: "money", hh_income_prev: "money", lord_income_prev: "money",
    fund_cash: "money", issued: "money", price: "money", pexp: "money", lam: "money", inv: "money", outflow: "money", fill: "money", wage: "money",
    tools_mult: "money", imp_pc: "money", urban_rent: "money", urban_pop: "money", ag_labor: "money", ind_labor: "money", y_ratio: "money",
    know: "knowledge", tech_year: "knowledge", tech_town: "knowledge", t_pump: "knowledge", t_wagon: "knowledge", t_rail: "knowledge", t_husb: "knowledge",
    t_fmach: "knowledge", forestry_sector: "knowledge", method_Q: "knowledge", method_A: "knowledge",
    y_acc: "accumulators", price_acc: "accumulators", wage_acc: "accumulators", town_prod: "accumulators", town_prod_year: "accumulators",
    cumprod: "accumulators", trade_F: "accumulators", trade_F_year: "accumulators", food_min_acc: "accumulators", projects: "accumulators",
    rail_links: "accumulators", rail_cooldown: "accumulators", spur_done: "accumulators", cell_flow_year: "accumulators", rail_opened: "accumulators",
    m_rent: "accumulators", m_invest: "accumulators", m_merch_margin: "accumulators", m_trade_value: "accumulators",
    ex_cell: "extractive", ex_sector: "extractive", ex_town: "extractive", ex_lam: "extractive", ex_field: "extractive", ex_tau: "extractive",
    ex_Lprev: "extractive", ex_labor: "extractive", ex_out: "extractive",
    graph: "transport", dist: "transport", pred: "transport", settle: "transport", catch: "transport", dcatch: "transport", town_cost: "transport",
    delta: "transport",
    town_cells: "after_transport", any_counts: "after_transport", cell_firms: "after_transport", same_counts: "after_transport",
    bids: "land_use", cell_urban_rent: "land_use", urban_density: "land_use",
    month_stats: "month", subcost: "month", food_ratio: "month",
    pbar: "yearly", wbar: "yearly", prev_catch: "yearly", cell_labor: "yearly", migration_flow: "yearly",
    cumprod_1750: "baselines", disc_X0: "baselines", cpi_basket: "baselines", cpi_base_prices: "baselines", wage_base: "baselines",
    money_start: "baselines", money_now: "baselines",
    series: "observer", obs: "observer",
    f_owner: "families", f_paid: "families", cell_owner: "families", cell_rent_paid: "families", cell_rent_year: "families", requests: "families",
    price_anchor: "prices", watch_markets: "prices", month_requests: "prices", month_sold: "prices", plain_bisection: "prices",
    engel: "engel", machine_stock: "machines",
    harvest_mult: "harvest", harvest_now: "harvest", harvest_town_now: "harvest", harvest_year: "harvest", harvest_town: "harvest",
    harvest_past: "harvest", harvest_dearth_year: "harvest", harvest_rent_gap: "harvest", harvest_rent_gap_year: "harvest",
    chron: "chronicle", works_added: "works", works_lease: "works", rw: "railways",
    cart_bill: "carting", carters: "carting", cart_months: "carting", cart_years: "carting",
    pp_growth: "people", pp_income_term: "people", pp_emigrants: "people", pp_foundings: "people", pp_gave: "people", pp_dcatch_was: "people",
    pp_moved: "people", pp_found_next: "people",
    town_built: "towns_kept", town_room: "towns_kept", town_ground: "towns_kept", town_peak: "towns_kept", town_low_years: "towns_kept",
    town_ground_rent: "towns_kept", town_hands_acc: "towns_kept",
    mine_sectors: "mines", mine_opened: "mines", mine_left: "mines", mine_idle: "mines", mine_head: "mines", mine_heap: "mines",
    mine_districts: "mines", mine_dug: "mines", mine_share: "mines", mine_promise: "mines", mine_rich: "mines", mine_surface: "mines",
    mine_hands: "mines", mine_out: "mines", mine_royalty: "mines", mine_acc: "mines", mine_months: "mines", mine_crew_each: "mines", mine_year: "mines",
    canal: "canals",
}

/// What else the run keeps besides the state: the streams, the news, the
/// yearly record the viewer's history and `run.bin` are made from, and the clock.
fn sim_parts(sim: &Sim, mem: bool) -> Vec<Part> {
    let mut w = Vec::new();
    SEED.with(|s| s.set(sim.params.seed));
    put_rngs(&sim.rngs, &mut w);
    let rngs = Part { name: "rngs", group: "sim", mem: if mem { weigh(&sim.rngs) } else { 0 }, raw: w.len(), hash: fnv(&w) };
    vec![
        rngs,
        part("events", "sim", &sim.events, mem),
        part("rec", "sim", &sim.rec, mem),
        part("clock", "sim", &(sim.year, sim.end_year, sim.t), mem),
        Part { name: "content", group: "rebuilt", mem: if mem { weigh(&sim.content) } else { 0 }, raw: 0, hash: 0 },
        Part { name: "params", group: "rebuilt", mem: if mem { weigh(&sim.params) } else { 0 }, raw: 0, hash: 0 },
    ]
}

/// The run written whole: the state, the streams, the news, the record, the clock.
fn put_sim(sim: &Sim) -> Vec<u8> {
    SEED.with(|s| s.set(sim.params.seed));
    let mut w = Vec::with_capacity(64 << 20);
    put_state(&sim.st, &mut w);
    put_rngs(&sim.rngs, &mut w);
    sim.events.put(&mut w);
    sim.rec.put(&mut w);
    (sim.year, sim.end_year, sim.t).put(&mut w);
    w
}

/// The run read back, with the families and the books carried across from
/// `carry` (which an example cannot write), and the content and parameters
/// made again from the rules, as `sim::start_from` makes them.
fn get_sim(bytes: &[u8], rules: Rules, seed: u64, families: usize, carry: &State) -> Sim {
    SEED.with(|s| s.set(seed));
    let mut r = bytes;
    let mut st = get_state(&mut r);
    st.families = carry.families.clone();
    st.acct = carry.acct.clone();
    let rngs = get_rngs(&mut r);
    let events = Bin::get(&mut r);
    let rec = Bin::get(&mut r);
    let (year, end_year, t): (i64, i64, usize) = Bin::get(&mut r);
    assert!(r.is_empty(), "{} bytes left over", r.len());
    let params = save::params(rules, seed, families);
    let mut content = rules.content();
    content.end_year = params.end_year.unwrap_or(content.end_year);
    Sim { st, content, params, rngs, events, rec, year, end_year, t, check_money: true }
}

/// Everything about a run that a continued game must have the same: the
/// check a save keeps, and the state, news and record byte for byte (with the
/// families and the books by how they print).
fn fingerprint(sim: &Sim) -> (String, u64, u64, u64) {
    let bytes = put_sim(sim);
    // (the books' `seconds` is the time they took on this machine, and no part of the island)
    let mut books = sim.st.acct.clone();
    books.seconds = 0.0;
    (save::check(sim), fnv(&bytes), fnv(format!("{:?}", sim.st.families).as_bytes()), fnv(format!("{books:?}").as_bytes()))
}

/// The news so far, as a fingerprint.
fn news(sim: &Sim) -> u64 {
    let mut w = Vec::new();
    sim.events.put(&mut w);
    fnv(&w)
}

// ---------------------------------------------------------------- what changes when

/// For each part: changes seen in ordinary months (not the one that closes a
/// year) of the sample years, and at the year's end, and in the turns.
#[derive(Default, Clone)]
struct Changes {
    months: usize,
    at_year_end: usize,
    turns: usize,
    ever: bool,
}

fn hashes(sim: &Sim) -> Vec<(&'static str, u64)> {
    let mut out: Vec<(&'static str, u64)> = parts(&sim.st, sim.params.seed, false).iter().map(|p| (p.name, p.hash)).collect();
    out.extend(sim_parts(sim, false).iter().filter(|p| p.group == "sim").map(|p| (p.name, p.hash)));
    out
}

fn sample_year(y: i64) -> bool {
    matches!(y, 1751 | 1752 | 1800 | 1801 | 1850 | 1851 | 1899 | 1900)
}

// ---------------------------------------------------------------- reports

fn kb(b: usize) -> String {
    if b >= 10 << 20 {
        format!("{:.1} MB", b as f64 / (1 << 20) as f64)
    } else if b >= 1 << 20 {
        format!("{:.2} MB", b as f64 / (1 << 20) as f64)
    } else {
        format!("{:.1} kB", b as f64 / 1024.0)
    }
}

fn size_table(seed: u64, label: &str, sim: &Sim) -> Vec<Part> {
    let mut ps = parts(&sim.st, seed, true);
    ps.extend(sim_parts(sim, true));
    let (mem, raw): (usize, usize) = ps.iter().fold((0, 0), |(m, r), p| (m + p.mem, r + p.raw));
    println!("\n== island {seed}, {label}: {} in memory, {} written raw (the content and parameters, made again from the rules, not written)", kb(mem), kb(raw));
    // by heading
    let mut groups: Vec<(&str, usize, usize, usize)> = Vec::new();
    for p in &ps {
        match groups.iter_mut().find(|g| g.0 == p.group) {
            Some(g) => {
                g.1 += p.mem;
                g.2 += p.raw;
                g.3 += 1;
            }
            None => groups.push((p.group, p.mem, p.raw, 1)),
        }
    }
    groups.sort_by(|a, b| b.1.cmp(&a.1));
    println!("   {:<16} {:>6} {:>11} {:>11}", "heading", "parts", "memory", "raw");
    for g in &groups {
        println!("   {:<16} {:>6} {:>11} {:>11}", g.0, g.3, kb(g.1), kb(g.2));
    }
    let mut big: Vec<&Part> = ps.iter().collect();
    big.sort_by(|a, b| b.mem.cmp(&a.mem));
    println!("   the twenty largest parts:");
    for p in big.iter().take(20) {
        println!("   {:<22} {:<16} {:>11} {:>11}", p.name, p.group, kb(p.mem), kb(p.raw));
    }
    // what is held for slots not used yet: the firms' slots past F_count, the
    // series' months past t, the record's years past those played
    let st = &sim.st;
    let fmax = st.f_alive.len().max(1);
    let firm_raw: usize = ps.iter().filter(|p| p.group == "firms" && p.name != "F_count" && p.name != "new_firms_year" && p.name != "closed_year").map(|p| p.raw).sum::<usize>()
        + ps.iter().filter(|p| matches!(p.name, "f_owner" | "f_paid" | "works_lease")).map(|p| p.raw).sum::<usize>();
    let firm_unused = firm_raw as f64 * (1.0 - st.F_count as f64 / fmax as f64);
    let months = st.series.data.first().map(|d| d.len()).unwrap_or(0);
    let series_unused = st.series.data.len() * months.saturating_sub(sim.t) * 8;
    let years = (sim.year - sim.content.start_year).max(0) as usize;
    let rec_raw = ps.iter().find(|p| p.name == "rec").map(|p| p.raw).unwrap_or(0);
    let rec_unused = if sim.rec.n_years > 0 { (rec_raw as f64 * (1.0 - years as f64 / sim.rec.n_years as f64)) as usize } else { 0 };
    println!(
        "   held for later: firm slots {} of {} used ({} of the firms' {} raw is empty slots); series {} of {} months ({} unused); record {} of {} years (about {} unused)",
        st.F_count,
        fmax,
        kb(firm_unused as usize),
        kb(firm_raw),
        sim.t,
        months,
        kb(series_unused),
        years,
        sim.rec.n_years,
        kb(rec_unused)
    );
    println!("   so trimmed to what is used, the raw picture would be about {}", kb(raw - firm_unused as usize - series_unused - rec_unused));
    // the books split by their public fields: the history of complete years
    // (behind the `Arc`), the island's yearly accounts, the last year's months,
    // and what is left, which is the open year's private tables
    let a = &st.acct;
    let (hist, years, last) = (weigh(&*a.hist), weigh(&a.years), weigh(&a.last_months) + weigh(&a.last_works_in));
    let open = weigh(a).saturating_sub(years + last);
    println!(
        "   the books: {} in the history of complete years (`hist`), {} in the yearly accounts (`years`), {} in the last year's months (`last_months`, `last_works_in`), {} in the open year's tables",
        kb(hist),
        kb(years),
        kb(last),
        kb(open)
    );
    ps
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut families, mut out, mut trip_year, mut do_ablate, mut do_change) = (vec![7u64, 3], 6usize, None::<String>, 1849i64, false, true);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => seeds = next().split(',').map(|s| s.parse().unwrap()).collect(),
            "--families" => families = next().parse().unwrap(),
            "--out" => out = Some(next()),
            "--round-trip" => trip_year = next().parse().unwrap(),
            "--ablate" => {
                do_ablate = true;
                i += 1;
                continue;
            }
            "--no-change" => {
                do_change = false;
                i += 1;
                continue;
            }
            other => panic!("unknown option {other}"),
        }
        i += 2;
    }
    let rules = Rules::Game;
    for &seed in &seeds {
        SEED.with(|s| s.set(seed));
        let t0 = Instant::now();
        let mut game = Game::under(rules, seed, families, 0);
        let start_secs = t0.elapsed().as_secs_f64();
        println!("\n######## island {seed}, {families} families, the game's rules: made and warmed up in {start_secs:.2} s");
        let t1 = Instant::now();
        let world_again = world::generate_world(&mut rng::make_rngs(seed).world, game.sim.params.W, game.sim.params.H, game.sim.params.n_towns, 0.58);
        println!("the island alone made again from its seed in {:.3} s; the same as the run's to the last bit: {}", t1.elapsed().as_secs_f64(), {
            let (mut a, mut b) = (Vec::new(), Vec::new());
            world_again.put(&mut a);
            game.sim.st.world.put(&mut b);
            a == b
        });
        let first = size_table(seed, "1750 (before the first month)", &game.sim);
        let start_hash: Vec<(&str, u64)> = hashes(&game.sim);
        let mut changes: Vec<Changes> = vec![Changes::default(); start_hash.len()];
        let mut last_turn = start_hash.clone();
        let mut sampled_years = 0;
        let mut play_secs = 0.0;
        let mut transport_same = (0, 0, 0.0f64);
        let mut sizes: Vec<(String, Vec<Part>)> = vec![("1750".into(), first)];
        loop {
            let y = game.sim.year;
            let sample = do_change && sample_year(y);
            let mut prev = if sample { hashes(&game.sim) } else { Vec::new() };
            if sample {
                sampled_years += 1;
            }
            for m in 0..12 {
                let t = Instant::now();
                game.month().expect("the month plays");
                play_secs += t.elapsed().as_secs_f64();
                if sample {
                    let now = hashes(&game.sim);
                    for (k, (a, b)) in prev.iter().zip(&now).enumerate() {
                        if a.1 != b.1 {
                            if m < 11 {
                                changes[k].months += 1;
                            } else {
                                changes[k].at_year_end += 1;
                            }
                        }
                    }
                    // are the transport tables what the cells as they stand would give?
                    if m == 5 || m == 11 {
                        let mut again = game.sim.st.clone();
                        let tt = Instant::now();
                        land::update_transport(&mut again, &game.sim.content, &game.sim.params);
                        transport_same.2 = transport_same.2.max(tt.elapsed().as_secs_f64());
                        let same = [
                            part("graph", "", &again.graph, false).hash == part("graph", "", &game.sim.st.graph, false).hash,
                            part("dist", "", &again.dist, false).hash == part("dist", "", &game.sim.st.dist, false).hash,
                            part("pred", "", &again.pred, false).hash == part("pred", "", &game.sim.st.pred, false).hash,
                            part("catch", "", &again.catch, false).hash == part("catch", "", &game.sim.st.catch, false).hash,
                        ];
                        println!("   {y} {}: the transport worked out again from the cells as they stand: graph {}, dist {}, pred {}, catch {}",
                            if m == 5 { "after June" } else { "at the turn" }, same[0], same[1], same[2], same[3]);
                        if same.iter().all(|&s| s) {
                            transport_same.0 += 1;
                        }
                        transport_same.1 += 1;
                    }
                    prev = now;
                }
            }
            game.turn();
            let now = if do_change { hashes(&game.sim) } else { Vec::new() };
            for (k, (a, b)) in last_turn.iter().zip(&now).enumerate() {
                if a.1 != b.1 {
                    changes[k].turns += 1;
                }
                if start_hash[k].1 != b.1 {
                    changes[k].ever = true;
                }
            }
            if do_change {
                last_turn = now;
            }
            let last = game.last_year();
            if last == 1799 || last == 1849 {
                sizes.push(((last + 1).to_string(), size_table(seed, &format!("the turn into {}", last + 1), &game.sim)));
            }
            if last == trip_year {
                round_trip(&game, rules, seed, families, do_ablate);
            }
            if last == 1849 || game.over() {
                if let Some(dir) = &out {
                    let bytes = put_sim(&game.sim);
                    let path = format!("{dir}/state-{seed}-{}.bin", last + 1);
                    std::fs::write(&path, &bytes).unwrap();
                    println!("   wrote {} ({})", path, kb(bytes.len()));
                }
            }
            if game.over() {
                break;
            }
        }
        sizes.push(("1900 end".into(), size_table(seed, "the end of 1900", &game.sim)));
        println!("\nthe century played a month at a time in {play_secs:.1} s (this is what loading a save of 1900 costs today, after the {start_secs:.1} s start)");
        // streams
        let pos = positions(&game.sim.rngs, seed);
        println!("numbers drawn from each stream by 1900 (world, names, firms, tech, people, rail, misc, harvest): {pos:?}");
        // writing and reading the whole run
        SEARCH.with(|s| s.set(0.0));
        let tw = Instant::now();
        let bytes = put_sim(&game.sim);
        let write = tw.elapsed().as_secs_f64();
        let search = SEARCH.with(|s| s.get());
        let tr = Instant::now();
        let back = get_sim(&bytes, rules, seed, families, &game.sim.st);
        let read = tr.elapsed().as_secs_f64();
        let same = fingerprint(&back) == fingerprint(&game.sim);
        println!(
            "the run at the end of 1900 written in {:.3} s ({:.3} s of it finding where the streams stand, which a counter would save) and read back in {:.3} s: {}; the same as it was: {same}",
            write,
            search,
            read,
            kb(bytes.len())
        );
        // the parts' size side by side
        println!("\nisland {seed}: each part's size in memory over the century (kB), and how often it changed");
        if do_change {
            println!("   ordinary months sampled: {}; year ends sampled: {}; turns: 151", sampled_years * 11, sampled_years);
        }
        println!(
            "   {:<22} {:<16} {}  {:>7} {:>7} {:>6} {:>5}  class",
            "part",
            "heading",
            sizes.iter().map(|(l, _)| format!("{l:>9}")).collect::<Vec<_>>().join(" "),
            "months",
            "yrend",
            "turns",
            "ever"
        );
        for (k, (name, _)) in start_hash.iter().enumerate() {
            let c = &changes[k];
            let class = if !do_change {
                "-"
            } else if !c.ever && c.turns == 0 && c.months == 0 && c.at_year_end == 0 {
                "never"
            } else if c.months * 10 >= sampled_years * 11 * 9 {
                "every month"
            } else if c.months > 0 {
                "some months"
            } else if c.turns * 10 >= 151 * 9 {
                "every year's end"
            } else {
                "some years' ends"
            };
            let group = sizes[0].1.iter().find(|p| p.name == *name).map(|p| p.group).unwrap_or("sim");
            let row: Vec<String> = sizes.iter().map(|(_, ps)| format!("{:>9.1}", ps.iter().find(|p| p.name == *name).map(|p| p.mem).unwrap_or(0) as f64 / 1024.0)).collect();
            println!("   {:<22} {:<16} {}  {:>7} {:>7} {:>6} {:>5}  {class}", name, group, row.join(" "), c.months, c.at_year_end, c.turns, c.ever);
        }
        if transport_same.1 > 0 {
            println!(
                "\nthe transport worked out again: the same as the run's at {} of {} points sampled; one working out takes up to {:.3} s",
                transport_same.0, transport_same.1, transport_same.2
            );
        }
    }
}

/// Write the run to bytes at a turn and six months later, read it back, play
/// both on ten years, and say whether they are still the same game.
fn round_trip(game: &Game, rules: Rules, seed: u64, families: usize, do_ablate: bool) {
    let y = game.last_year();
    println!("\n-- round trip at the turn into {}", y + 1);
    for months in [0usize, 6] {
        let mut orig = game.sim.clone();
        for m in 0..months {
            sim::run_month(&mut orig, m).unwrap();
        }
        let t = Instant::now();
        let bytes = put_sim(&orig);
        let w = t.elapsed().as_secs_f64();
        let t = Instant::now();
        let mut back = get_sim(&bytes, rules, seed, families, &orig.st);
        let r = t.elapsed().as_secs_f64();
        let same_now = fingerprint(&back) == fingerprint(&orig);
        // play both on: the rest of the year, then nine more
        let go = |s: &mut Sim| {
            for m in months..12 {
                sim::run_month(s, m).unwrap();
            }
            sim::end_year(s);
            for _ in 0..9 {
                sim::run_year(s).unwrap();
            }
        };
        go(&mut orig);
        go(&mut back);
        let (a, b) = (fingerprint(&orig), fingerprint(&back));
        println!(
            "   {} months in: written in {w:.3} s, read in {r:.3} s ({}); the same on reading: {same_now}; ten years on: check {} / {} ({}), state and record {}, families {}, books {}",
            months,
            kb(bytes.len()),
            a.0,
            b.0,
            if a.0 == b.0 { "the same" } else { "DIFFERENT" },
            if a.1 == b.1 { "the same" } else { "DIFFERENT" },
            if a.2 == b.2 { "the same" } else { "DIFFERENT" },
            if a.3 == b.3 { "the same" } else { "DIFFERENT" },
        );
    }
    // a rebuilt transport at the turn, played on
    {
        let mut orig = game.sim.clone();
        let mut back = game.sim.clone();
        let t = Instant::now();
        land::update_transport(&mut back.st, &back.content, &back.params);
        let secs = t.elapsed().as_secs_f64();
        for s in [&mut orig, &mut back] {
            for _ in 0..10 {
                sim::run_year(s).unwrap();
            }
        }
        println!(
            "   the transport worked out again at the turn ({secs:.3} s) and played on ten years: {}",
            if fingerprint(&orig) == fingerprint(&back) { "the same game" } else { "a DIFFERENT game" }
        );
    }
    if !do_ablate {
        return;
    }
    println!("\n-- each part left out of a picture taken at the turn into {}, and two years played on", y + 1);
    let base = {
        let mut s = game.sim.clone();
        for _ in 0..2 {
            sim::run_year(&mut s).unwrap();
        }
        (save::check(&s), news(&s))
    };
    std::panic::set_hook(Box::new(|_| {}));
    let mut needed = Vec::new();
    let mut news_only = Vec::new();
    let mut spare = Vec::new();
    let mut panics = Vec::new();
    for &name in NAMES.iter().chain(["pp_rng", "cn"].iter()) {
        let mut s = game.sim.clone();
        ablate(&mut s.st, name);
        let played = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for _ in 0..2 {
                sim::run_year(&mut s)?;
            }
            Ok::<(String, u64), String>((save::check(&s), news(&s)))
        }));
        match played {
            Ok(Ok(c)) if c == base => spare.push(name),
            Ok(Ok(c)) if c.0 == base.0 => news_only.push(name),
            Ok(Ok(_)) => needed.push(name),
            Ok(Err(e)) => panics.push(format!("{name} ({e})")),
            Err(_) => panics.push(name.to_string()),
        }
    }
    let _ = std::panic::take_hook();
    println!("   the same game and the same news without it ({}): {}", spare.len(), spare.join(" "));
    println!("   the same economy, other news ({}): {}", news_only.len(), news_only.join(" "));
    println!("   a different game without it ({}): {}", needed.len(), needed.join(" "));
    println!("   the run stops or fails without it ({}): {}", panics.len(), panics.join(" "));
}
