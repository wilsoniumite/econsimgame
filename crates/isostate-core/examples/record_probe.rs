//! What the record carries for the viewer, and what it would weigh to carry
//! more (`docs/probes/dx-record.md`). Measuring code for a probe: it changes
//! nothing in the simulation or the record.
//!
//!     cargo run -p isostate-core --release --example record_probe -- [--out DIR]
//!         [--end-year Y] [--hash] [SEED ...]
//!
//! Seeds default to 7 and 3, each on its own thread, under the game's rules with
//! six families, played a month at a time as a live game is. For each year it
//! prices what the record and the live hand-over leave out, as each would be
//! encoded (bytes, the time to make the bytes from the island, the time to turn
//! them back into what the viewer would hold), and counts what today's bytes clip:
//!
//! - `mix4`: the four largest land classes of a cell and three shares (seven
//!   bytes a cell, where today's three keep two classes); `mix12`, all twelve shares;
//! - `rent16`, `dist16`, `res16`, `built16`: rent, the way to market, residents
//!   and the year a cell joined its town in two bytes each, where one clips;
//! - `traffic16` and `boats16`: the weight carted and boated past each cell;
//! - `owner8` (a byte a cell), `owner_rows` (three bytes for each square a
//!   family owns) and `owner_moves` (three bytes for each square that changed hands);
//! - `works12`: a row for each works and year, from its founding and for six
//!   years after it closes (capacity, the share on order, hands, how hard it
//!   runs, its state and three bytes kept for yards and combined works);
//!   `works_month`, the same works' hands, work and state month by month (eight bytes);
//! - `machines`: the share of each sector in each town worked by machines, and their quality;
//! - `harvest_cells`: this year's yield of the farms and pastures of each cell;
//! - `town_y`: each town's income ratio, which the live year does not carry.
//!
//! And what is carried today: the time and size of `live_year` and `live_month`,
//! of `record::layers`, of `record::pack` at the end, and of reading the packed
//! run back (its header parsed, every array copied out). With `--out` each
//! candidate's whole run is written there as a file of its own, with the run's
//! `run.bin` and a table by year, for measuring what they come to compressed.
//!
//! `--hash` adds two runs of seed 7 to 1900 under the port's rules (no families,
//! and six), as the golden test makes them, and prints an FNV-1a hash of every
//! array of the packed record and of its header without the date: what a test
//! that holds the record would hold, and what it would cost.
#![allow(non_snake_case)]

use std::hint::black_box;
use std::time::Instant;

use isostate_core::params::Rules;
use isostate_core::*;
use isostate_core::{accounts, live, machines, mines, record, sim, works};

/// One thing priced year by year.
#[derive(Clone, Default)]
struct Cand {
    name: &'static str,
    /// by year: bytes, nanoseconds to write, nanoseconds to read
    bytes: Vec<usize>,
    write: Vec<u64>,
    read: Vec<u64>,
    /// the whole run, year after year, for compressing
    all: Vec<u8>,
}

impl Cand {
    fn new(name: &'static str) -> Cand {
        Cand { name, ..Default::default() }
    }

    /// Time `enc` making the year's bytes and `dec` reading them back.
    fn year(&mut self, keep: bool, enc: impl FnOnce() -> Vec<u8>, dec: impl FnOnce(&[u8]) -> f64) {
        let t0 = Instant::now();
        let bytes = black_box(enc());
        let t1 = Instant::now();
        black_box(dec(&bytes));
        let t2 = Instant::now();
        self.bytes.push(bytes.len());
        self.write.push((t1 - t0).as_nanos() as u64);
        self.read.push((t2 - t1).as_nanos() as u64);
        if keep {
            self.all.extend_from_slice(&bytes);
        }
    }

    /// The same for a cost that comes twelve times a year: the year's sums.
    fn add_month(&mut self, k: usize, keep: bool, enc: impl FnOnce() -> Vec<u8>, dec: impl FnOnce(&[u8]) -> f64) {
        if self.bytes.len() <= k {
            self.bytes.resize(k + 1, 0);
            self.write.resize(k + 1, 0);
            self.read.resize(k + 1, 0);
        }
        let t0 = Instant::now();
        let bytes = black_box(enc());
        let t1 = Instant::now();
        black_box(dec(&bytes));
        let t2 = Instant::now();
        self.bytes[k] += bytes.len();
        self.write[k] += (t1 - t0).as_nanos() as u64;
        self.read[k] += (t2 - t1).as_nanos() as u64;
        if keep {
            self.all.extend_from_slice(&bytes);
        }
    }
}

/// What today's bytes lose or clip, by year.
#[derive(Clone, Default)]
struct Clip {
    year: i64,
    land: usize,
    /// mean share of a land cell's ground outside its two largest classes, and outside its four
    lost2: f64,
    lost4: f64,
    /// land cells whose third class holds a tenth of the cell or more
    third10: usize,
    rent_cells: usize,
    rent_over: usize,
    rent_max: f64,
    dist_over: usize,
    dist_none: usize,
    dist_max: f64,
    res_over: usize,
    res_max: f64,
    built_over: usize,
    traffic_cells: usize,
    boat_cells: usize,
    owned: usize,
    moved: usize,
    works_rows: usize,
    works_kept: usize,
    works_alive: usize,
    states: [usize; 6],
    news_with_cell: usize,
    news: usize,
    /// today's hand-over: live_year's bytes (the families' reports as JSON) and its time
    live_year_bytes: usize,
    live_year_ns: u64,
    live_month_bytes: usize,
    live_month_ns: u64,
    layers_ns: u64,
    firm_k_bytes: usize,
}

fn display_class(name: &str) -> Option<(usize, bool)> {
    match name {
        "farm" => Some((1, false)),
        "pasture" => Some((2, false)),
        "forestry" => Some((3, false)),
        "quarry" => Some((5, true)),
        "claypit" => Some((6, true)),
        "ironmine" => Some((7, true)),
        "coalmine" => Some((8, true)),
        _ => None,
    }
}

/// Every display class's share of every cell, as `record::display_mix` works
/// them out before it keeps the largest two (copied here, since it keeps them inside).
fn shares(st: &State, C: &Content) -> Vec<[f64; 12]> {
    let (N, NS) = (st.N, C.NS);
    let (dig, rough) = mines::display_split(st);
    let mut out = vec![[0.0f64; 12]; N];
    for c in 0..N {
        let D = &mut out[c];
        let ks = st.cell_K_share[c].clamp(0.0, 1.0);
        let mut tot = 0.0;
        for r in 0..=NS {
            tot += st.lam[(r, c)];
        }
        let scale = if tot > 0.0 { (1.0 - ks) / tot.max(1e-12) } else { 0.0 };
        for (s, name) in C.sector_name.iter().enumerate() {
            let share = st.lam[(s, c)] * scale;
            match display_class(name) {
                Some((k, true)) => {
                    D[k] += dig * share;
                    D[4] += rough * share;
                }
                Some((k, false)) => D[k] += share,
                None => D[4] += share,
            }
        }
        let pot = st.world.forest_pot[c].max(1e-6);
        let woody = ((st.cover[c] / pot - 0.3) / 0.4).clamp(0.0, 1.0);
        let wild = st.lam[(NS, c)] * scale + if tot > 0.0 { 0.0 } else { 1.0 - ks };
        D[3] += wild * woody;
        D[4] += wild * (1.0 - woody);
        D[9] += ks;
        if st.urban[c] {
            *D = [0.0; 12];
            D[10] = 1.0;
        }
        if st.derelict_timer[c] > 0 && !st.urban[c] {
            *D = [0.0; 12];
            D[11] = 1.0;
        }
        if !st.world.land[c] {
            *D = [0.0; 12];
            D[0] = 1.0;
        }
    }
    out
}

/// The classes of one cell from the largest down, ties to the lower class.
fn ranked(D: &[f64; 12]) -> [usize; 12] {
    let mut k: [usize; 12] = std::array::from_fn(|i| i);
    k.sort_by(|&a, &b| D[b].total_cmp(&D[a]).then(a.cmp(&b)));
    k
}

/// A positive number on a log scale in two bytes: 0 for none, else 1 to 65535
/// for 10^lo to 10^(lo + span).
fn log16(x: f64, lo: f64, span: f64) -> u16 {
    if x > 0.0 && x.is_finite() { (((x.log10() - lo) / span).clamp(0.0, 1.0) * 65534.0).round() as u16 + 1 } else { 0 }
}

fn unlog16(v: u16, lo: f64, span: f64) -> f32 {
    if v == 0 { 0.0 } else { 10f64.powf(lo + (v - 1) as f64 / 65534.0 * span) as f32 }
}

fn u16s(v: impl Iterator<Item = u16>) -> Vec<u8> {
    v.flat_map(|x| x.to_le_bytes()).collect()
}

fn read_u16s(b: &[u8]) -> impl Iterator<Item = u16> + '_ {
    b.chunks_exact(2).map(|p| u16::from_le_bytes([p[0], p[1]]))
}

/// A works' state for the row: 0 going up, 1 working, 2 short time (under half
/// its capacity), 3 standing (under 0.15), 4 closed this year, 5 closed before
/// and still standing in the picture.
fn works_state(P: &Params, K: f64, order: f64, util: f64) -> u8 {
    if works::building(P, K, order) {
        0
    } else if util < 0.15 {
        3
    } else if util < 0.5 {
        2
    } else {
        1
    }
}

/// Years a closed works keeps a row, so the picture can show it standing empty.
const KEEP_CLOSED: i64 = 6;

struct Run {
    seed: u64,
    end: i64,
    cands: Vec<Cand>,
    clip: Vec<Clip>,
    /// the packed run: bytes, the time to pack it and to read it back, and its arrays' sizes
    run_bin: Vec<u8>,
    pack_ms: f64,
    read_ms: f64,
    header_bytes: usize,
    arrays: Vec<(String, String, usize)>,
    live_static_bytes: usize,
    sim_ms: f64,
}

#[allow(clippy::too_many_lines)]
fn one(seed: u64, end_year: Option<i64>, keep: bool) -> Run {
    let mut P = Rules::Game.params();
    P.seed = seed;
    P.n_families = 6;
    P.end_year = end_year;
    let mut s = sim::start(&P, Rules::Game.content());
    let C = s.content.clone();
    let (N, T, G, NS) = (s.st.N, s.st.T_max, C.G, C.NS);
    let names = [
        "mix4", "mix12", "rent16", "dist16", "res16", "built16", "traffic16", "boats16", "owner8", "owner_rows", "owner_moves", "works12",
        "works_month", "machines", "harvest_cells", "town_y",
    ];
    let mut cands: Vec<Cand> = names.iter().map(|n| Cand::new(n)).collect();
    let ix = |name: &str| names.iter().position(|n| *n == name).unwrap();
    let mut clip: Vec<Clip> = Vec::new();
    let stat = live::live_static(&s.st, &s.content);
    let live_static_bytes = serde_json::to_string(&stat).map_or(0, |t| t.len());
    let mut cursor = LiveCursor::new(&C);
    let mut prev_owner: Vec<i64> = s.st.cell_owner.clone();
    let hands_col = accounts::WORKS_FIELDS.iter().position(|&f| f == "hands").unwrap();
    let util_col = accounts::WORKS_FIELDS.iter().position(|&f| f == "util").unwrap();
    let farm_rows: Vec<usize> = C.sector_name.iter().enumerate().filter(|(_, n)| *n == "farm" || *n == "pasture").map(|(s, _)| s).collect();
    let t_sim = Instant::now();
    let mut sim_ns = 0u128;
    while s.year <= s.end_year {
        let k = (s.year - C.start_year) as usize;
        let mut cl = Clip { year: s.year, ..Default::default() };
        for m in 0..12 {
            let t0 = Instant::now();
            sim::run_month(&mut s, m).expect("money is conserved");
            if m == 11 {
                sim::end_year(&mut s);
            }
            sim_ns += t0.elapsed().as_nanos();
            if m < 11 {
                let t0 = Instant::now();
                let lm = live::live_month(&mut cursor, &s.st, &s.content, &s.params, &s.events, s.t - 1);
                cl.live_month_ns += t0.elapsed().as_nanos() as u64;
                cl.live_month_bytes += lm.series.len() * 8 + lm.wage.len() * 4 + lm.price.len() * 4 + lm.cash.len() * 8 + serde_json::to_string(&lm.events).map_or(0, |t| t.len());
            }
            // the works month by month: id, hands in tenths, how hard it runs, its state
            let st = &s.st;
            cands[ix("works_month")].add_month(
                k,
                keep,
                || {
                    let mut b = Vec::new();
                    for f in (0..st.F_count).filter(|&f| st.f_alive[f]) {
                        b.extend_from_slice(&(f as u32).to_le_bytes());
                        b.extend_from_slice(&((st.f_labor[f] * 10.0).round().clamp(0.0, 65535.0) as u16).to_le_bytes());
                        b.push((st.f_u[f].clamp(0.0, 1.0) * 255.0).round() as u8);
                        b.push(works_state(&s.params, st.f_K[f], st.f_order_K[f], st.f_u[f]));
                    }
                    b
                },
                |b| b.chunks_exact(8).map(|r| u16::from_le_bytes([r[4], r[5]]) as f64 * 0.1 + r[6] as f64 / 255.0).sum(),
            );
        }
        let (st, P) = (&s.st, &s.params);

        // today's hand-over, as the game makes it at a year's end
        let t0 = Instant::now();
        let ly = live::live_year(&mut cursor, st, &C, P, &s.events);
        cl.live_year_ns = t0.elapsed().as_nanos() as u64;
        let l = &ly.layers;
        cl.live_year_bytes = [&l.land_use, &l.inten, &l.rent, &l.trans, &l.catch, &l.dist, &l.mix_a, &l.mix_b, &l.mix_s].iter().map(|v| v.len()).sum::<usize>()
            + [&l.built, &l.res, &l.works].iter().map(|v| v.as_ref().map_or(0, |v| v.len())).sum::<usize>()
            + ly.towns.active.len() + (ly.towns.pop.len() + ly.towns.urban.len() + ly.towns.wage.len() + ly.towns.price.len()) * 4 + ly.towns.know.len()
            + ly.firm_K.len() * 16
            + serde_json::to_string(&ly.new_firms).map_or(0, |t| t.len())
            + ly.closed_firms.len() * 8
            + serde_json::to_string(&ly.events).map_or(0, |t| t.len())
            + ly.series.len() * 8
            + serde_json::to_string(&ly.families).map_or(0, |t| t.len())
            + ly.cell_owner.len() * 2
            + ly.harvest_town.len() * 4
            + ly.workings.len() * 40
            + serde_json::to_string(&ly.canals).map_or(0, |t| t.len())
            + serde_json::to_string(&ly.accounts).map_or(0, |t| t.len());
        cl.firm_k_bytes = ly.firm_K.len() * 4;
        let t0 = Instant::now();
        black_box(record::layers(st, &C));
        black_box(record::town_layers(st));
        cl.layers_ns = t0.elapsed().as_nanos() as u64;
        cl.news = s.events.len();
        cl.news_with_cell = s.events.iter().filter(|e| e.cell.is_some()).count();

        // the land: four classes and three shares, and all twelve
        let D = shares(st, &C);
        let land: Vec<usize> = (0..N).filter(|&c| st.world.land[c]).collect();
        cl.land = land.len();
        for &c in &land {
            let r = ranked(&D[c]);
            let tot: f64 = D[c].iter().sum::<f64>().max(1e-12);
            let top = |n: usize| r[..n].iter().map(|&i| D[c][i]).sum::<f64>() / tot;
            cl.lost2 += 1.0 - top(2);
            cl.lost4 += 1.0 - top(4);
            if D[c][r[2]] / tot >= 0.1 {
                cl.third10 += 1;
            }
        }
        cl.lost2 /= cl.land.max(1) as f64;
        cl.lost4 /= cl.land.max(1) as f64;
        cands[ix("mix4")].year(
            keep,
            || {
                let mut b = vec![0u8; 7 * N];
                for c in 0..N {
                    let r = ranked(&D[c]);
                    let tot: f64 = D[c].iter().sum::<f64>().max(1e-12);
                    for j in 0..4 {
                        b[j * N + c] = r[j] as u8;
                    }
                    for j in 0..3 {
                        b[(4 + j) * N + c] = (D[c][r[j]] / tot * 255.0).round() as u8;
                    }
                }
                b
            },
            |b| {
                let mut out = vec![[(0u8, 0f32); 4]; N];
                for (c, o) in out.iter_mut().enumerate() {
                    let s: [f32; 3] = std::array::from_fn(|j| b[(4 + j) * N + c] as f32 / 255.0);
                    let last = (1.0 - s.iter().sum::<f32>()).max(0.0);
                    *o = [(b[c], s[0]), (b[N + c], s[1]), (b[2 * N + c], s[2]), (b[3 * N + c], last)];
                }
                out.iter().map(|o| o[1].1 as f64).sum()
            },
        );
        cands[ix("mix12")].year(
            keep,
            || {
                let mut b = vec![0u8; 12 * N];
                for c in 0..N {
                    let tot: f64 = D[c].iter().sum::<f64>().max(1e-12);
                    for j in 0..12 {
                        b[j * N + c] = (D[c][j] / tot * 255.0).round() as u8;
                    }
                }
                b
            },
            |b| b.iter().map(|&v| v as f32 / 255.0).sum::<f32>() as f64,
        );

        // the bytes that clip today, in two bytes each
        for c in 0..N {
            let r = st.cell_rent[c];
            if r > 1e-3 {
                cl.rent_cells += 1;
                cl.rent_max = cl.rent_max.max(r);
                if r >= 1000.0 {
                    cl.rent_over += 1;
                }
            }
            if st.world.land[c] {
                let d = st.dcatch[c];
                if !d.is_finite() {
                    cl.dist_none += 1;
                } else {
                    cl.dist_max = cl.dist_max.max(d);
                    if d * 4.0 >= 254.5 {
                        cl.dist_over += 1;
                    }
                }
            }
            if st.urban_res[c] > 0.0 {
                cl.res_max = cl.res_max.max(st.urban_res[c]);
                if st.urban_res[c] / record::RES_UNIT >= 255.5 {
                    cl.res_over += 1;
                }
            }
            if st.urban[c] && st.town_built.get(c).is_some_and(|&y| y - record::BUILT_YEAR0 > 255) {
                cl.built_over += 1;
            }
        }
        cands[ix("rent16")].year(keep, || u16s(st.cell_rent.iter().map(|&r| log16(r, -3.0, 10.0))), |b| read_u16s(b).map(|v| unlog16(v, -3.0, 10.0) as f64).sum());
        cands[ix("dist16")].year(
            keep,
            || u16s(st.dcatch.iter().map(|&d| if d.is_finite() { (d * 100.0).round().clamp(0.0, 65534.0) as u16 } else { 65535 })),
            |b| read_u16s(b).map(|v| if v == 65535 { f64::INFINITY } else { v as f64 * 0.01 }).filter(|d| d.is_finite()).sum(),
        );
        cands[ix("res16")].year(keep, || u16s(st.urban_res.iter().map(|&r| r.round().clamp(0.0, 65535.0) as u16)), |b| read_u16s(b).map(|v| v as f64).sum());
        cands[ix("built16")].year(
            keep,
            || u16s((0..N).map(|c| if st.urban[c] { st.town_built.get(c).map_or(0, |&y| y.clamp(1, 65535) as u16) } else { 0 })),
            |b| read_u16s(b).filter(|&v| v > 0).count() as f64,
        );

        // what moves: carts and boats past each cell over the year
        cl.traffic_cells = st.traffic.iter().filter(|&&t| t > 0.0).count();
        cl.boat_cells = st.cn.traffic.iter().filter(|&&t| t > 0.0).count();
        cands[ix("traffic16")].year(keep, || u16s(st.traffic.iter().map(|&t| log16(t, -3.0, 10.0))), |b| read_u16s(b).map(|v| unlog16(v, -3.0, 10.0) as f64).sum());
        cands[ix("boats16")].year(
            keep,
            || if st.cn.traffic.is_empty() { Vec::new() } else { u16s(st.cn.traffic.iter().map(|&t| log16(t, -3.0, 10.0))) },
            |b| read_u16s(b).map(|v| unlog16(v, -3.0, 10.0) as f64).sum(),
        );

        // who owns the land
        cl.owned = st.cell_owner.iter().filter(|&&o| o >= 0).count();
        cl.moved = (0..N).filter(|&c| st.cell_owner.get(c) != prev_owner.get(c)).count();
        cands[ix("owner8")].year(keep, || st.cell_owner.iter().map(|&o| o.clamp(-1, 127) as i8 as u8).collect(), |b| b.iter().filter(|&&v| v as i8 >= 0).count() as f64);
        let rows = |pick: &dyn Fn(usize) -> bool| -> Vec<u8> {
            let mut b = Vec::new();
            for c in (0..N).filter(|&c| pick(c)) {
                b.extend_from_slice(&(c as u16).to_le_bytes());
                b.push(st.cell_owner[c].clamp(-1, 127) as i8 as u8);
            }
            b
        };
        let read_rows = |b: &[u8]| -> f64 {
            let mut v = vec![-1i8; N];
            for r in b.chunks_exact(3) {
                v[u16::from_le_bytes([r[0], r[1]]) as usize] = r[2] as i8;
            }
            v.iter().filter(|&&o| o >= 0).count() as f64
        };
        cands[ix("owner_rows")].year(keep, || rows(&|c| st.cell_owner[c] >= 0), read_rows);
        cands[ix("owner_moves")].year(keep, || rows(&|c| st.cell_owner.get(c) != prev_owner.get(c)), read_rows);
        prev_owner = st.cell_owner.clone();

        // the works' row: from the books where the run keeps them, else from the state
        let books = st.acct.year(st.acct.hist.n.wrapping_sub(1)).filter(|_| st.acct.on && st.acct.hist.n == k + 1);
        let from_books = |f: usize| -> Option<(f64, f64)> {
            let b = books.as_ref()?;
            let j = b.works_id.binary_search(&(f as u32)).ok()?;
            let r = &b.works[j * accounts::KW..(j + 1) * accounts::KW];
            Some((r[hands_col] as f64, r[util_col] as f64))
        };
        let mut states = [0usize; 6];
        let rows_of: Vec<(usize, u8)> = (0..st.F_count)
            .filter_map(|f| {
                if st.f_alive[f] {
                    let util = from_books(f).map_or(st.f_u[f], |b| b.1);
                    Some((f, works_state(P, st.f_K[f], st.f_order_K[f], util)))
                } else if st.f_died[f] == s.year - 1 {
                    Some((f, 4))
                } else if st.f_died[f] < s.year - 1 && st.f_died[f] >= s.year - 1 - KEEP_CLOSED {
                    Some((f, 5))
                } else {
                    None
                }
            })
            .collect();
        for &(_, x) in &rows_of {
            states[x as usize] += 1;
        }
        cl.states = states;
        cl.works_alive = (0..st.F_count).filter(|&f| st.f_alive[f]).count();
        cl.works_rows = rows_of.len();
        cl.works_kept = states[4] + states[5];
        cands[ix("works12")].year(
            keep,
            || {
                let mut b = Vec::with_capacity(rows_of.len() * 12);
                for &(f, state) in &rows_of {
                    let (hands, util) = from_books(f).unwrap_or((st.f_labor[f], st.f_u[f]));
                    let (K, order) = (st.f_K[f], st.f_order_K[f]);
                    b.extend_from_slice(&(K as f32).to_le_bytes());
                    b.push((order / (K + order).max(1e-12) * 255.0).round() as u8);
                    b.extend_from_slice(&((hands * 10.0).round().clamp(0.0, 65535.0) as u16).to_le_bytes());
                    b.push((util.clamp(0.0, 1.0) * 255.0).round() as u8);
                    b.push(state);
                    b.extend_from_slice(&[0, 0, 0]);
                }
                b
            },
            |b| b.chunks_exact(12).map(|r| f32::from_le_bytes([r[0], r[1], r[2], r[3]]) as f64 + u16::from_le_bytes([r[5], r[6]]) as f64 * 0.1).sum(),
        );

        // machines by town and sector, and their quality
        cands[ix("machines")].year(
            keep,
            || {
                let mut b: Vec<u8> = st.machine_stock.data.iter().map(|&v| if v < 0.0 { 255 } else { (v.clamp(0.0, 1.0) * 254.0).round() as u8 }).collect();
                b.extend_from_slice(&(machines::quality(st, &C, P) as f32).to_le_bytes());
                b
            },
            |b| b.iter().filter(|&&v| v < 255).map(|&v| v as f64 / 254.0).sum(),
        );
        // the year's harvest on the farms and pastures of each cell, in 128ths of an average year
        cands[ix("harvest_cells")].year(
            keep,
            || {
                if st.harvest_mult.rows < NS {
                    return Vec::new();
                }
                farm_rows.iter().flat_map(|&r| st.harvest_mult.row(r).iter().map(|&v| (v * 128.0).round().clamp(0.0, 255.0) as u8)).collect()
            },
            |b| b.iter().map(|&v| v as f64 / 128.0).sum(),
        );
        cands[ix("town_y")].year(keep, || st.y_ratio.iter().flat_map(|&y| (y as f32).to_le_bytes()).collect(), |b| b.chunks_exact(4).map(|p| f32::from_le_bytes([p[0], p[1], p[2], p[3]]) as f64).sum());
        let _ = (T, G);
        clip.push(cl);
    }
    let sim_ms = sim_ns as f64 / 1e6;
    let _ = t_sim;
    let end = s.year - 1;
    let r = sim::finish(s);
    let t0 = Instant::now();
    let run_bin = record::pack(&r.state, &r.content, &r.params, &r.events, &r.rec, "record probe", false);
    let pack_ms = t0.elapsed().as_secs_f64() * 1e3;
    let t0 = Instant::now();
    let (header_bytes, arrays) = read_back(&run_bin);
    let read_ms = t0.elapsed().as_secs_f64() * 1e3;
    Run { seed, end, cands, clip, run_bin, pack_ms, read_ms, header_bytes, arrays, live_static_bytes, sim_ms }
}

/// Read a packed run as the viewer does, in outline: the header parsed whole,
/// every array copied out into numbers of its type. Gives the header's size and
/// each array's name, type and size.
fn read_back(blob: &[u8]) -> (usize, Vec<(String, String, usize)>) {
    let hlen = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    let header: serde_json::Value = serde_json::from_slice(&blob[4..4 + hlen]).expect("the header reads");
    let base = 4 + hlen;
    let mut out = Vec::new();
    for a in header["arrays"].as_array().expect("an array table") {
        let (off, n) = (a["offset"].as_u64().unwrap() as usize, a["nbytes"].as_u64().unwrap() as usize);
        let bytes = &blob[base + off..base + off + n];
        let dtype = a["dtype"].as_str().unwrap_or("").to_string();
        match dtype.as_str() {
            "float32" => {
                black_box(bytes.chunks_exact(4).map(|p| f32::from_le_bytes([p[0], p[1], p[2], p[3]])).collect::<Vec<f32>>());
            }
            "int32" => {
                black_box(bytes.chunks_exact(4).map(|p| i32::from_le_bytes([p[0], p[1], p[2], p[3]])).collect::<Vec<i32>>());
            }
            _ => {
                black_box(bytes.to_vec());
            }
        }
        out.push((a["name"].as_str().unwrap_or("").to_string(), dtype, n));
    }
    (hlen, out)
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The port's record of seed 7, as the golden test's runs would pack it: a hash
/// of every array and of the header without its date.
fn hash_port(families: usize) {
    let mut P = default_params();
    P.seed = 7;
    P.n_families = families;
    let t0 = Instant::now();
    let r = sim::run_with(&P, default_content(), false, true).expect("money is conserved");
    let run_s = t0.elapsed().as_secs_f64();
    let t0 = Instant::now();
    let blob = record::pack(&r.state, &r.content, &r.params, &r.events, &r.rec, "golden", false);
    let pack_ms = t0.elapsed().as_secs_f64() * 1e3;
    let t0 = Instant::now();
    let hlen = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    let mut header: serde_json::Value = serde_json::from_slice(&blob[4..4 + hlen]).unwrap();
    header["meta"].as_object_mut().unwrap().remove("generated");
    let table = header["arrays"].clone();
    let mut lines = Vec::new();
    for a in table.as_array().unwrap() {
        let (off, n) = (a["offset"].as_u64().unwrap() as usize, a["nbytes"].as_u64().unwrap() as usize);
        lines.push(format!("{:<16} {:>9} 0x{:016x}", a["name"].as_str().unwrap(), n, fnv1a(&blob[4 + hlen + off..4 + hlen + off + n])));
    }
    header.as_object_mut().unwrap().remove("arrays");
    let htext = serde_json::to_string(&header).unwrap();
    lines.push(format!("{:<16} {:>9} 0x{:016x}", "header", htext.len(), fnv1a(htext.as_bytes())));
    let hash_ms = t0.elapsed().as_secs_f64() * 1e3;
    println!(
        "\nthe port's record, seed 7 to 1900, {families} families: {} bytes, {} arrays; the run {run_s:.1} s, packed in {pack_ms:.0} ms, hashed in {hash_ms:.0} ms",
        blob.len(),
        lines.len() - 1
    );
    for l in lines {
        println!("  {l}");
    }
}

fn kb(b: f64) -> String {
    if b >= 1e6 { format!("{:.2} MB", b / 1e6) } else { format!("{:.1} kB", b / 1e3) }
}

fn print_run(r: &Run, out: Option<&std::path::Path>) {
    let y0 = r.clip.first().map_or(1750, |c| c.year);
    let ny = r.clip.len();
    println!("\n==== island {} to {}, game rules, six families: the simulation {:.1} s (played month by month)", r.seed, r.end, r.sim_ms / 1e3);
    println!("run.bin {} ({} header, {} arrays); packed in {:.0} ms, read back in {:.0} ms; LiveStatic as JSON {}", kb(r.run_bin.len() as f64), kb(r.header_bytes as f64), r.arrays.len(), r.pack_ms, r.read_ms, kb(r.live_static_bytes as f64));
    // the run file by part
    let group = |name: &str| -> &'static str {
        if record::LAYERS.contains(&name) {
            "nine map layers"
        } else if record::TOWN_LAYERS.contains(&name) {
            "three town layers"
        } else if name.starts_with("town_") {
            "town tables"
        } else if name.starts_with("firm_") {
            "works' capacities"
        } else if name.starts_with("mine_") {
            "workings"
        } else if name.starts_with("acct_works") {
            "books: works"
        } else if name.starts_with("acct_") {
            "books: towns, goods, flows"
        } else if name == "series" {
            "monthly series"
        } else if name == "harvest" {
            "harvest"
        } else if ["elev", "terrain", "river", "nav", "down"].contains(&name) || record::STATIC_FIELDS.contains(&name) {
            "the island, once"
        } else {
            "the new land, once"
        }
    };
    let mut parts: Vec<(&str, usize)> = Vec::new();
    for (name, _, n) in &r.arrays {
        let g = group(name);
        match parts.iter_mut().find(|p| p.0 == g) {
            Some(p) => p.1 += n,
            None => parts.push((g, *n)),
        }
    }
    parts.push(("header (JSON)", r.header_bytes));
    for (g, n) in &parts {
        println!("  {:<28} {:>10}  {:>5.1}%", g, kb(*n as f64), *n as f64 * 100.0 / r.run_bin.len() as f64);
    }
    let dtypes: std::collections::BTreeSet<&str> = r.arrays.iter().map(|a| a.1.as_str()).collect();
    println!("  array types written: {:?}", dtypes);

    // what today's bytes clip, by year
    let marks: Vec<usize> = (0..ny).filter(|&k| (y0 + k as i64) % 25 == 0 || k + 1 == ny).collect();
    println!("\nwhat today's bytes lose or clip (land cells {}):", r.clip[0].land);
    println!("  year  lost2  lost4 third>=10%  rent>0 rent>=1000  rent_max  dist>=63.75 no_way dist_max  res>2044 res_max built>1900  traffic boats owned moved");
    for &k in &marks {
        let c = &r.clip[k];
        println!(
            "  {} {:>6.3} {:>6.3} {:>10} {:>7} {:>10} {:>9.0} {:>12} {:>6} {:>8.1} {:>9} {:>7.0} {:>10} {:>8} {:>5} {:>5} {:>5}",
            c.year, c.lost2, c.lost4, c.third10, c.rent_cells, c.rent_over, c.rent_max, c.dist_over, c.dist_none, c.dist_max, c.res_over, c.res_max, c.built_over, c.traffic_cells, c.boat_cells, c.owned, c.moved
        );
    }
    println!("\nthe works' rows: rows, works standing, kept after closing, and by state (going up, working, short time, standing, closed this year, closed before):");
    for &k in &marks {
        let c = &r.clip[k];
        println!("  {} {:>6} {:>6} {:>6}   {:?}", c.year, c.works_rows, c.works_alive, c.works_kept, c.states);
    }
    let rows: usize = r.clip.iter().map(|c| c.works_rows).sum();
    let alive: usize = r.clip.iter().map(|c| c.works_alive).sum();
    println!("  in all {rows} rows over the run, {alive} of them works standing ({} kept after closing)", rows - alive);
    let news: usize = r.clip.last().map_or(0, |c| c.news);
    println!("  news items {news}, of them with a cell {}", r.clip.last().map_or(0, |c| c.news_with_cell));

    // today's hand-over
    let mean = |f: &dyn Fn(&Clip) -> f64| r.clip.iter().map(f).sum::<f64>() / ny as f64;
    println!("\ntoday's hand-over, a year's mean (and in {}, {}, {}):", marks[0] as i64 + y0, y0 + marks[marks.len() / 2] as i64, r.end);
    let at = |k: usize, f: &dyn Fn(&Clip) -> f64| f(&r.clip[k]);
    for (what, f) in [
        ("live_year: bytes", &(|c: &Clip| c.live_year_bytes as f64) as &dyn Fn(&Clip) -> f64),
        ("live_year: ms", &|c: &Clip| c.live_year_ns as f64 / 1e6),
        ("live_month x11: bytes", &|c: &Clip| c.live_month_bytes as f64),
        ("live_month x11: ms", &|c: &Clip| c.live_month_ns as f64 / 1e6),
        ("record::layers + town_layers: ms", &|c: &Clip| c.layers_ns as f64 / 1e6),
        ("firm_K as recorded: bytes", &|c: &Clip| c.firm_k_bytes as f64),
    ] {
        println!("  {:<34} {:>12.2}   {:>12.2} {:>12.2} {:>12.2}", what, mean(f), at(marks[0], f), at(marks[marks.len() / 2], f), at(ny - 1, f));
    }

    // the candidates
    println!("\nwhat each would weigh: bytes in {}, {}, {}, the whole run, and the year's mean time to write and to read (microseconds)", y0, y0 + 50, r.end);
    println!("  {:<14} {:>10} {:>10} {:>10} {:>11} {:>9} {:>9}", "", y0, y0 + 50, r.end, "run", "write", "read");
    for c in &r.cands {
        let b = |k: usize| c.bytes.get(k).copied().unwrap_or(0);
        let total: usize = c.bytes.iter().sum();
        let w = c.write.iter().sum::<u64>() as f64 / ny as f64 / 1e3;
        let rd = c.read.iter().sum::<u64>() as f64 / ny as f64 / 1e3;
        println!("  {:<14} {:>10} {:>10} {:>10} {:>11} {:>9.0} {:>9.0}", c.name, b(0), b(50.min(ny - 1)), b(ny - 1), kb(total as f64), w, rd);
    }

    if let Some(dir) = out {
        let dir = dir.join(format!("seed{}", r.seed));
        std::fs::create_dir_all(&dir).expect("the folder can be made");
        std::fs::write(dir.join("run.bin"), &r.run_bin).expect("run.bin written");
        for c in &r.cands {
            std::fs::write(dir.join(format!("{}.bin", c.name)), &c.all).expect("a candidate written");
        }
        let mut csv = String::from("year,lost2,lost4,third10,rent_cells,rent_over,rent_max,dist_over,dist_none,dist_max,res_over,res_max,built_over,traffic_cells,boat_cells,owned,moved,works_rows,works_alive,works_kept,live_year_bytes,live_year_ms,live_month_bytes,live_month_ms,layers_ms");
        for c in &r.cands {
            csv.push_str(&format!(",{0}_bytes,{0}_write_us,{0}_read_us", c.name));
        }
        csv.push('\n');
        for (k, c) in r.clip.iter().enumerate() {
            csv.push_str(&format!(
                "{},{:.5},{:.5},{},{},{},{:.1},{},{},{:.2},{},{:.0},{},{},{},{},{},{},{},{},{},{:.3},{},{:.3},{:.3}",
                c.year, c.lost2, c.lost4, c.third10, c.rent_cells, c.rent_over, c.rent_max, c.dist_over, c.dist_none, c.dist_max, c.res_over, c.res_max, c.built_over,
                c.traffic_cells, c.boat_cells, c.owned, c.moved, c.works_rows, c.works_alive, c.works_kept, c.live_year_bytes, c.live_year_ns as f64 / 1e6,
                c.live_month_bytes, c.live_month_ns as f64 / 1e6, c.layers_ns as f64 / 1e6
            ));
            for x in &r.cands {
                csv.push_str(&format!(",{},{:.1},{:.1}", x.bytes.get(k).copied().unwrap_or(0), x.write.get(k).copied().unwrap_or(0) as f64 / 1e3, x.read.get(k).copied().unwrap_or(0) as f64 / 1e3));
            }
            csv.push('\n');
        }
        std::fs::write(dir.join("years.csv"), csv).expect("years.csv written");
        let mut arrays = String::from("name,dtype,bytes\n");
        for (n, d, b) in &r.arrays {
            arrays.push_str(&format!("{n},{d},{b}\n"));
        }
        std::fs::write(dir.join("arrays.csv"), arrays).expect("arrays.csv written");
        println!("\nwritten to {}", dir.display());
    }
}

fn main() {
    let mut seeds: Vec<u64> = Vec::new();
    let mut end_year: Option<i64> = None;
    let mut out: Option<std::path::PathBuf> = None;
    let mut hash = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--end-year" => end_year = args.next().and_then(|v| v.parse().ok()),
            "--out" => out = args.next().map(Into::into),
            "--hash" => hash = true,
            s => seeds.push(s.parse().expect("a seed is a number")),
        }
    }
    if seeds.is_empty() {
        seeds = vec![7, 3];
    }
    let keep = out.is_some();
    let runs: Vec<Run> = std::thread::scope(|sc| {
        let hs: Vec<_> = seeds.iter().map(|&seed| sc.spawn(move || one(seed, end_year, keep))).collect();
        hs.into_iter().map(|h| h.join().expect("a run")).collect()
    });
    for r in &runs {
        print_run(r, out.as_deref());
    }
    if hash {
        hash_port(0);
        hash_port(6);
    }
}
