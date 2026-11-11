//! Mines and pits that take little of the surface (not in the Python;
//! `docs/rules/mines.md`).
//!
//! Under the port a quarry, a clay pit, an iron mine and a colliery are land
//! uses like a farm: each bids for a share of its cell's surface, its output is
//! Cobb-Douglas in that share, and by 1900 the four of them hold a fifth of the
//! island by area. A real working is a pit head, a yard and a spoil heap over
//! seams that run for miles underground. What is scarce is the deposit and the
//! right to work it, not the acres on top.
//!
//! With `mines.enabled`:
//!
//! - A. the deposit is the fixed factor. A working stands on one cell (its pit
//!   head) and has the lease of the deposit within `reach` of it: every cell
//!   that no working had before it, for as long as it stands, and nobody else
//!   may sink a pit head on those cells. Each cell of deposit is worked as the port
//!   works it, `A * f * L^(1 - alpha)`, with the whole cell's deposit where a
//!   share of its surface stood, and richness lost to the haul underground
//!   (`haul` a cell). The hands share themselves among the cells so that the
//!   last hand raises the same everywhere, which sums exactly to one working
//!   with `A * F * L^(1 - alpha)`, `F = (sum of (f * haul)^(1/alpha))^alpha`.
//! - B. what is left of the sale after the merchant, the carting, the machines
//!   and the wages is the royalty. It is paid as rent is, to the owners of the
//!   cells whose deposit is worked, each by what its cell gives.
//! - C. a working has to be sunk: `sink` hands for a year at the town's wage,
//!   who raise nothing, paid out of the royalty (the landowner's outlay). So
//!   nobody opens a pit head for a few tons, and the workings stand apart.
//! - D. a working is opened where the royalty it would have paid over the last
//!   few years' prices would repay the sinking in `payback` years: the best
//!   prospects first, and only so many in a year as the town's market has
//!   room for. Once sunk it costs nothing to keep, so a slump lays its hands
//!   off and leaves it standing; it is abandoned when it has stood idle for
//!   `idle_years`, and its neighbours take over what they can reach.
//! - E. it goes deeper. Each unit raised from a cell lowers the richness the
//!   hands meet there, `f / (1 + dug / (reserve * f))`, so the near deposits
//!   are worked first and the workings move on as they tire.
//! - F. the surface it takes is by its hands (`hands_per_cell` fill a cell, as
//!   the works' land is counted), taken out of the pit head's cell before the
//!   farms, pastures and woods share the rest.
//!
//! Every payment here is one the monthly loop already makes: the sinkers' wages go
//! to the town's households with the other wages, and the royalty goes through
//! `econ::pay_cell_rents` with the land's rent. Switched off, every function
//! below returns at once and the state's tables stay empty.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::econ::sector_tfp;
use crate::mat::Mat;
use crate::params::Params;
use crate::record::Record;
use crate::state::State;

/// The numbers of the mines.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MineParams {
    /// Quarries, pits and mines work a deposit and take little surface.
    pub enabled: bool,
    /// A. How far from its pit head a working reaches, in cells (a cell is 2 km).
    pub reach: f64,
    /// A. Richness lost to the haul underground: a cell d cells from the pit
    /// head counts for `1 / (1 + haul * d)` of what is in it.
    pub haul: f64,
    /// C. Hands it takes for a year to sink a working. 0: a pit head costs nothing.
    pub sink: f64,
    /// D. A working is opened where the royalty promised would repay its
    /// sinking in this many years (so: `sink / payback` wages a month or more).
    pub payback: f64,
    /// D. A working is abandoned when it has stood idle (under one hand, on a
    /// year's average) this many years running.
    pub idle_years: u32,
    /// D. How much of this year's promise goes into what a site is believed to
    /// pay (the rest is what was believed last year). 1: this year's prices alone.
    pub memory: f64,
    /// D. The new workings a town's adventurers open in a year add at most this
    /// share of what its market takes of the good in a month (one may always
    /// open). 0: every prospect is opened at once.
    pub year_room: f64,
    /// E. The output, per unit of richness, at which a cell's richness has
    /// halved. 0: a deposit never tires.
    pub reserve: f64,
    /// F. Hands that fill a cell with pit heads, yards and spoil.
    pub hands_per_cell: f64,
    /// F. The most of a cell its workings' surface may take.
    pub cell_limit: f64,
}

impl Default for MineParams {
    /// Off: what the port runs with. The numbers are the game's, so that
    /// `--set mines.enabled=true` alone measures the mechanism as it is played.
    fn default() -> MineParams {
        MineParams {
            enabled: false,
            reach: 2.0,
            haul: 0.05,
            sink: 40.0,
            payback: 4.0,
            idle_years: 10,
            memory: 0.3,
            year_room: 0.15,
            reserve: 6.0e6,
            hands_per_cell: 5000.0,
            cell_limit: 0.5,
        }
    }
}

impl MineParams {
    /// The game's mines.
    pub fn game() -> MineParams {
        MineParams { enabled: true, ..MineParams::default() }
    }
}

/// A sector that works a deposit: the four the map draws as diggings.
pub fn is_extractive(name: &str) -> bool {
    matches!(name, "quarry" | "claypit" | "ironmine" | "coalmine")
}

/// The row of the mines' tables that holds sector s, if it is extractive (and the
/// mechanism is on: the tables are empty otherwise).
pub fn row(st: &State, s: usize) -> Option<usize> {
    st.mine_sectors.iter().position(|&x| x == s)
}

/// The tables, sized once the run knows its cells. Left empty while the mechanism is off.
fn init(st: &mut State, C: &Content) {
    let N = st.N;
    st.mine_sectors = (0..C.NS).filter(|&s| is_extractive(&C.sector_name[s])).collect();
    let R = st.mine_sectors.len();
    st.mine_opened = vec![0; R * N];
    st.mine_left = vec![0; R * N];
    st.mine_head = vec![-1; R * N];
    st.mine_heap = vec![0.0; R * N];
    st.mine_idle = vec![0; R * N];
    for m in [
        &mut st.mine_dug,
        &mut st.mine_share,
        &mut st.mine_promise,
        &mut st.mine_rich,
        &mut st.mine_surface,
        &mut st.mine_hands,
        &mut st.mine_out,
        &mut st.mine_royalty,
    ] {
        *m = Mat::zeros(R, N);
    }
    st.mine_acc = Mat::zeros(3 * R, N);
    st.mine_months = 0;
}

/// The rounds in which a run's first workings are placed (`review`): each
/// takes the best sites left that stand two reaches apart.
const FIRST_ROUNDS: usize = 3;

// ---------------------------------------------------------------- E. depth

/// The richness the hands meet in a cell whose deposit's is f, when `dug` has
/// been raised from it so far. A rich deposit has more in it, so it tires more
/// slowly; a cell nobody has worked gives f.
fn worn(P: &Params, f: f64, dug: f64) -> f64 {
    let reserve = P.mines.reserve;
    if reserve > 0.0 && f > 0.0 && dug > 0.0 { f / (1.0 + dug / (reserve * f)) } else { f }
}

// ---------------------------------------------------------------- the year (land::update_land_use)

/// The land rule's bids with the extractive sectors' taken out: a working bids
/// for no surface, it is given what its hands need (`review`).
pub fn surface_bids(C: &Content, P: &Params, mut bids: Mat) -> Mat {
    if P.mines.enabled {
        for s in (0..C.NS).filter(|&s| is_extractive(&C.sector_name[s])) {
            bids.row_mut(s).iter_mut().for_each(|v| *v = 0.0);
        }
    }
    bids
}

/// The books of the months since the last review: what each working had in
/// hands, what it raised and what it paid in royalty; and what was raised goes
/// against the cells it came from.
fn close_books(st: &mut State) {
    let (R, N) = (st.mine_sectors.len(), st.N);
    let months = st.mine_months.max(1) as f64;
    for r in 0..R {
        for c in 0..N {
            st.mine_hands[(r, c)] = st.mine_acc[(r, c)] / months;
            st.mine_out[(r, c)] = st.mine_acc[(R + r, c)];
            st.mine_royalty[(r, c)] = st.mine_acc[(2 * R + r, c)];
            st.mine_heap[r * N + c] += st.mine_out[(r, c)];
        }
        for c in 0..N {
            let h = st.mine_head[r * N + c];
            if h >= 0 {
                st.mine_dug[(r, c)] += st.mine_out[(r, h as usize)] * st.mine_share[(r, c)];
            }
        }
    }
    st.mine_acc.fill(0.0);
    st.mine_months = 0;
}

/// The cells within `reach` of a cell, as (rows down, columns across, distance),
/// the cell itself first and the rest nearest first.
fn offsets(reach: f64) -> Vec<(i64, i64, f64)> {
    let n = reach.max(0.0).floor() as i64;
    let mut out = Vec::new();
    for dy in -n..=n {
        for dx in -n..=n {
            let d = ((dy * dy + dx * dx) as f64).sqrt();
            if d <= reach + 1e-9 {
                out.push((dy, dx, d));
            }
        }
    }
    out.sort_by(|a, b| a.2.total_cmp(&b.2).then(a.0.cmp(&b.0)).then(a.1.cmp(&b.1)));
    out
}

/// One sector's sums for the year: what a pit head on each cell would pay for
/// deposit, and the deposit there is.
struct Year {
    /// Money a month a pit head on the cell pays per unit of `fx` it works; 0
    /// where no pit head may stand or the price at the cell does not cover the machines.
    k: Vec<f64>,
    /// The price at the cell net of carting and machines, the town's price and its wage.
    npr: Vec<f64>,
    p: Vec<f64>,
    w: Vec<f64>,
    /// Each cell's richness, worn, to the power 1 / alpha.
    fx: Vec<f64>,
    /// The haul's factor for each offset, to the same power.
    hx: Vec<f64>,
    offs: Vec<(i64, i64, f64)>,
    alpha: f64,
    spread: f64,
    H: usize,
    W: usize,
}

impl Year {
    /// What a working on h with `x` of deposit (the sum of its cells' `fx * hx`)
    /// would pay in royalty, in its town's wages a month, and what it would
    /// raise: at the hands that suit the year's prices.
    fn promise(&self, h: usize, x: f64) -> (f64, f64) {
        if !(self.k[h] > 0.0 && x > 0.0) {
            return (0.0, 0.0);
        }
        // the bid k x is alpha of the sale net of carting and machines; the
        // merchant's spread comes off what is left
        let bid = self.k[h] * x;
        let out = bid / (self.alpha * self.npr[h]);
        (((bid - self.spread * self.p[h] * out) / self.w[h]).max(0.0), out)
    }

    /// The cell at offset j from c, if it is on the map.
    fn at(&self, c: usize, j: usize) -> Option<usize> {
        let (y, x) = ((c / self.W) as i64 + self.offs[j].0, (c % self.W) as i64 + self.offs[j].1);
        (y >= 0 && x >= 0 && y < self.H as i64 && x < self.W as i64).then(|| y as usize * self.W + x as usize)
    }

    /// The offset that takes cell `from` to cell `to`, if `to` is within reach.
    fn offset(&self, from: usize, to: usize) -> Option<usize> {
        let (dy, dx) = ((to / self.W) as i64 - (from / self.W) as i64, (to % self.W) as i64 - (from % self.W) as i64);
        self.offs.iter().position(|o| o.0 == dy && o.1 == dx)
    }

    /// The deposit a pit head on h would be given: every cell within its reach
    /// that no working has, as the sum of `fx * hx`.
    fn free(&self, h: usize, head: &[i32]) -> f64 {
        let mut x = 0.0;
        for j in 0..self.offs.len() {
            if let Some(c) = self.at(h, j) {
                if self.fx[c] > 0.0 && head[c] < 0 {
                    x += self.fx[c] * self.hx[j];
                }
            }
        }
        x
    }

    /// The leases for the year. A working keeps the cells it has while it
    /// stands; a cell nobody has goes to the working within reach that pays most
    /// for it (one whose year did not pay still takes what nobody else wants:
    /// whether it raises anything is each month's question). Returns each
    /// working's deposit, the sum of its cells' `fx * hx`.
    fn lease(&self, open: &[bool], head: &mut [i32]) -> Vec<f64> {
        let N = self.H * self.W;
        let mut x = vec![0.0; N];
        for c in 0..N {
            if !(self.fx[c] > 0.0) {
                head[c] = -1;
                continue;
            }
            // a lease lapses with its working, or if the working can no longer reach
            let mut via = if head[c] >= 0 && open[head[c] as usize] { self.offset(c, head[c] as usize) } else { None };
            if via.is_none() {
                head[c] = -1;
                let mut best = 0.0;
                for j in 0..self.offs.len() {
                    if let Some(h) = self.at(c, j) {
                        let pays = self.k[h].max(f64::MIN_POSITIVE) * self.hx[j];
                        if open[h] && pays > best {
                            (best, via) = (pays, Some(j));
                            head[c] = h as i32;
                        }
                    }
                }
            }
            if let Some(j) = via {
                x[head[c] as usize] += self.fx[c] * self.hx[j];
            }
        }
        x
    }
}

/// The year's review of the workings, on the year's mean prices and wages:
/// which are abandoned and which opened (D), which cells' deposit each works
/// (A), and the surface each takes (F). Returns that surface cell by cell, for
/// the land rule to leave out of what the farms, pastures and woods share; all
/// zeros while the mechanism is off.
///
/// `taken` are the cells no land use may have (town ground, a cell the works
/// have filled, dereliction, land with no market): no pit head stands there,
/// though the deposit under them may be worked from beside them.
#[allow(clippy::too_many_arguments)]
pub fn review(st: &mut State, C: &Content, P: &Params, year: i64, pbar: &Mat, wbar: &[f64], taken: &[bool], initial: bool) -> Vec<f64> {
    let N = st.N;
    let mut ms = vec![0.0; N];
    if !P.mines.enabled {
        return ms;
    }
    if st.mine_sectors.is_empty() {
        init(st, C);
    }
    let m = &P.mines;
    // the royalty, in wages a month, that makes a site a prospect
    let worth = if m.payback > 0.0 { m.sink / m.payback } else { 0.0 };
    let (R, T) = (st.mine_sectors.len(), st.T_max);
    let (H, W) = (st.world.H, st.world.W);
    close_books(st);
    let offs = offsets(m.reach);
    // a town founded this year has no year's wage or prices yet (FAMILIES.md,
    // "Surprises"): its workings are reckoned at this month's, or the country's
    let known: Vec<f64> = (0..T).filter(|&t| st.town_active[t] && wbar[t] > 0.0).map(|t| wbar[t]).collect();
    let w_mean = if known.is_empty() { 0.0 } else { known.iter().sum::<f64>() / known.len() as f64 };
    // a working opened by this review is sunk over the twelve months that follow
    st.mine_year = if initial { 0 } else { year };
    st.mine_districts.clear();
    for r in 0..R {
        let s = st.mine_sectors[r];
        let (g, a) = (C.sector_good[s], C.sector_alpha[s]);
        let field = st.world.field(&C.sector_field[s]);
        let fx: Vec<f64> = (0..N)
            .map(|c| if st.world.land[c] && field[c] > 0.0 { worn(P, field[c], st.mine_dug[(r, c)]).powf(1.0 / a) } else { 0.0 })
            .collect();
        let hx: Vec<f64> = offs.iter().map(|o| (1.0 / (1.0 + m.haul * o.2)).powf(1.0 / a)).collect();
        // what a pit head would pay: the port's bid for land (`land::sector_bids`),
        // per unit of deposit instead of per share of the surface
        let mut y = Year {
            k: vec![0.0; N],
            npr: vec![0.0; N],
            p: vec![0.0; N],
            w: vec![0.0; N],
            fx,
            hx,
            offs: offs.clone(),
            alpha: a,
            spread: P.merchant_spread,
            H,
            W,
        };
        let mut per_town: Vec<Option<(f64, f64, f64, f64)>> = vec![None; T];
        for h in 0..N {
            if st.catch[h] < 0 || taken[h] || !st.world.land[h] {
                continue;
            }
            let t = st.catch[h] as usize;
            let (A, in_unit, p, w) = match per_town[t] {
                Some(v) => v,
                None => {
                    let (mult, _, _, in_unit) = sector_tfp(st, C, P, s, t, pbar, &st.fill);
                    let w = if wbar[t] > 0.0 { wbar[t] } else if st.wage[t] > 0.0 { st.wage[t] } else { w_mean };
                    let p = if pbar[(t, g)] > 0.0 { pbar[(t, g)] } else { st.price[(t, g)] };
                    let v = (C.sector_A[s] * mult, in_unit, p, w);
                    per_town[t] = Some(v);
                    v
                }
            };
            let npr = p - P.freight * C.weight[g] * st.dcatch[h] - in_unit;
            if npr > 0.0 && w > 0.0 {
                y.k[h] = a * ((1.0 - a) / w).powf((1.0 - a) / a) * (A * npr).powf(1.0 / a);
                y.npr[h] = npr;
                y.p[h] = p;
                y.w[h] = w;
            }
        }

        // the workings there are: one whose site is lost (the town has grown over
        // it, or it has no market) is given up, and so is one that has stood
        // idle too long; the deposit they had is free again
        let mut open: Vec<bool> = (0..N).map(|h| st.mine_opened[r * N + h] != 0).collect();
        let mut shut = vec![false; N];
        for h in 0..N {
            if open[h] {
                let k = r * N + h;
                st.mine_idle[k] = if st.mine_hands[(r, h)] < 1.0 { st.mine_idle[k].saturating_add(1) } else { 0 };
                if st.catch[h] < 0 || taken[h] || st.mine_idle[k] as u32 >= m.idle_years.max(1) {
                    (open[h], shut[h]) = (false, true);
                }
            }
        }
        let mut head: Vec<i32> = st.mine_head[r * N..(r + 1) * N].to_vec();
        let x = y.lease(&open, &mut head);
        for h in 0..N {
            if open[h] {
                let was = st.mine_promise[(r, h)];
                st.mine_promise[(r, h)] = was + m.memory * (y.promise(h, x[h]).0 - was);
            }
        }
        // the prospects: what a new pit head would pay from the deposit nobody
        // has within its reach, believed over the last few years and promised by
        // this one too, since nobody sinks into a falling market. It may not
        // stand on deposit another working has. The best are opened first, none
        // within two reaches of another opened this year (they would want the
        // same cells), and in each town only until the new workings would add
        // `year_room` of what its market takes of the good in a month (works.md,
        // D, is the same thought); one may always open. A run starts with the
        // best sites that would pay anything at all, whatever they would pay:
        // its first prices are guesses, and the workings nobody then needs stand
        // idle at no cost.
        let far = (2.0 * m.reach) * (2.0 * m.reach);
        let mut added = vec![0.0; T];
        let mut new_heads: Vec<usize> = Vec::new();
        for round in 0..(if initial { FIRST_ROUNDS } else { 1 }) {
            let mut prospects: Vec<(f64, usize, f64)> = Vec::new();
            for h in 0..N {
                if open[h] || shut[h] {
                    continue;
                }
                let (promise, out) = if y.k[h] > 0.0 && head[h] < 0 { y.promise(h, y.free(h, &head)) } else { (0.0, 0.0) };
                if round == 0 {
                    let was = st.mine_promise[(r, h)];
                    st.mine_promise[(r, h)] = if initial { promise } else { was + m.memory * (promise - was) };
                }
                let sure = if initial { promise } else { st.mine_promise[(r, h)].min(promise) };
                if promise > 0.0 && (initial || sure >= worth) {
                    prospects.push((sure, h, out));
                }
            }
            prospects.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
            let before = new_heads.len();
            for &(_, h, out) in &prospects {
                let t = st.catch[h] as usize;
                let room = if initial || !(m.year_room > 0.0) { f64::INFINITY } else { m.year_room * st.outflow[(t, g)] };
                if added[t] > 0.0 && added[t] + out > room {
                    continue;
                }
                let (hy, hxx) = ((h / W) as f64, (h % W) as f64);
                let near = new_heads[before..].iter().any(|&o| {
                    let (dy, dx) = ((o / W) as f64 - hy, (o % W) as f64 - hxx);
                    dy * dy + dx * dx <= far
                });
                if near {
                    continue;
                }
                open[h] = true;
                added[t] += out;
                new_heads.push(h);
            }
            if new_heads.len() == before {
                break;
            }
            y.lease(&open, &mut head);
        }

        // the year's workings and the deposit each has
        let x = y.lease(&open, &mut head);
        for h in 0..N {
            let k = r * N + h;
            if shut[h] {
                st.mine_opened[k] = 0;
                st.mine_left[k] = year as i32;
                st.mine_idle[k] = 0;
            } else if open[h] && st.mine_opened[k] == 0 {
                // the workings a run starts with are old ones
                st.mine_opened[k] = if initial { (year - 1 - P.warmup_years) as i32 } else { year as i32 };
            }
            st.mine_rich[(r, h)] = if open[h] && x[h] > 0.0 { x[h].powf(a) } else { 0.0 };
        }
        for c in 0..N {
            let h = head[c];
            st.mine_head[r * N + c] = h;
            st.mine_share[(r, c)] = match (h >= 0).then(|| y.offset(c, h as usize)).flatten() {
                Some(j) => y.fx[c] * y.hx[j] / x[h as usize],
                None => 0.0,
            };
            if h >= 0 {
                st.mine_districts.entry(r * N + h as usize).or_default().push((c, st.mine_share[(r, c)]));
            }
        }
    }
    // F. the surface: by last year's hands, and a new working's by its sinkers
    for c in 0..N {
        let mut tot = 0.0;
        for r in 0..R {
            let v = if st.mine_opened[r * N + c] != 0 && m.hands_per_cell > 0.0 {
                st.mine_hands[(r, c)].max(if st.mine_opened[r * N + c] as i64 == st.mine_year { m.sink } else { 0.0 }) / m.hands_per_cell
            } else {
                0.0
            };
            st.mine_surface[(r, c)] = v;
            tot += v;
        }
        let room = m.cell_limit.min(1.0 - st.cell_K_share[c]).max(0.0);
        if tot > room {
            for r in 0..R {
                st.mine_surface[(r, c)] *= room / tot;
            }
            tot = room;
        }
        ms[c] = tot;
    }
    // the land rule moves last year's shares toward this year's bids; the
    // workings' surface is not its to move, and is written back when it has done
    for r in 0..R {
        let s = st.mine_sectors[r];
        if st.lam.rows > s {
            st.lam.row_mut(s).iter_mut().for_each(|v| *v = 0.0);
        }
    }
    ms
}

/// The workings' surface, written into the land shares the rule has just
/// worked out for everything else: a cell's shares go on summing to what the
/// works have left of it.
pub fn write_surface(st: &State, P: &Params, lam: &mut Mat) {
    if !P.mines.enabled {
        return;
    }
    for (r, &s) in st.mine_sectors.iter().enumerate() {
        lam.row_mut(s).copy_from_slice(st.mine_surface.row(r));
    }
}

/// The fixed factor of the unit sector s has on cell c, where its share of the
/// surface is `lam`: for an extractive sector, one working if a pit head
/// stands there and nothing if not (what it works is in `unit_field`).
pub fn unit_share(st: &State, P: &Params, s: usize, c: usize, lam: f64) -> f64 {
    if !P.mines.enabled {
        return lam;
    }
    match row(st, s) {
        Some(r) => (st.mine_opened[r * st.N + c] != 0 && st.mine_rich[(r, c)] > 0.0) as u8 as f64,
        None => lam,
    }
}

/// The richness the unit of sector s on cell c works this year: for a working,
/// the F of everything within its reach (A), worn (E).
pub fn unit_field(st: &State, P: &Params, s: usize, c: usize, f: f64) -> f64 {
    if !P.mines.enabled {
        return f;
    }
    match row(st, s) {
        Some(r) => st.mine_rich[(r, c)],
        None => f,
    }
}

/// The map's rent counts what each cell's deposit was paid a month over the
/// last year. (The rent a works reckons with when it looks for a site,
/// `land_rent_only`, does not: a yard on the surface displaces no seam.)
pub fn add_royalties(st: &State, P: &Params, rent: &mut [f64]) {
    if !P.mines.enabled {
        return;
    }
    let N = st.N;
    for r in 0..st.mine_sectors.len() {
        for (c, x) in rent.iter_mut().enumerate() {
            let h = st.mine_head[r * N + c];
            if h >= 0 && !st.urban[c] {
                *x += (st.mine_royalty[(r, h as usize)] / 12.0 * st.mine_share[(r, c)]).max(0.0);
            }
        }
    }
}

/// A town does not build over the pit head of an iron mine or a colliery at
/// work (under the port, over a cell that is mostly one).
pub fn blocks_town(st: &State, C: &Content, P: &Params, c: usize) -> bool {
    P.mines.enabled && st.mine_sectors.iter().enumerate().any(|(r, &s)| C.sector_mine[s] && st.mine_opened[r * st.N + c] != 0)
}

// ---------------------------------------------------------------- the month (econ::step_month)

/// Whether unit u is a working being sunk: one the last review opened.
fn sinking(st: &State, u: usize) -> bool {
    match row(st, st.ex_sector[u]) {
        Some(r) => st.mine_opened[r * st.N + st.ex_cell[u]] as i64 == st.mine_year,
        None => false,
    }
}

/// C. The sinkers are not for hire: they come out of each town's labour supply
/// before the wage is sought. (Should a town's new workings ever ask for more
/// than a tenth of its hands, the sinkers are thinned to that: a safety valve
/// for a village with a coalfield, not a rule anyone should meet.)
pub fn take_crews(st: &mut State, P: &Params, supply: &mut [f64]) {
    if !P.mines.enabled || st.mine_sectors.is_empty() {
        return;
    }
    let T = supply.len();
    let mut n = vec![0.0; T];
    for u in 0..st.ex_cell.len() {
        if sinking(st, u) {
            n[st.ex_town[u]] += 1.0;
        }
    }
    let mut each = vec![0.0; T];
    for t in 0..T {
        if n[t] > 0.0 && st.town_active[t] {
            each[t] = P.mines.sink.min(0.1 * supply[t] / n[t]).max(0.0);
            supply[t] -= each[t] * n[t];
        }
    }
    st.mine_crew_each = each;
}

/// C. The sinkers are paid at the town's wage: added to their working's wage
/// bill, so the royalty is what is left after them (less than nothing, in a
/// working's first year: the landowners' outlay).
pub fn pay_crews(st: &State, P: &Params, w: &[f64], wage_ex: &mut [f64]) {
    if !P.mines.enabled || st.mine_crew_each.is_empty() {
        return;
    }
    for (u, x) in wage_ex.iter_mut().enumerate() {
        if sinking(st, u) {
            let t = st.ex_town[u];
            *x += st.mine_crew_each[t] * w[t];
        }
    }
}

/// B. Who a working's royalty is paid to: the cells whose deposit it works,
/// each by its share, in place of the pit head's cell alone. The lists
/// `econ::pay_cell_rents` is given, with every working's line split so; None
/// while the mechanism is off.
pub fn royalty_cells(st: &State, P: &Params, town: &[usize], cell: &[usize], rent: &[f64]) -> Option<(Vec<usize>, Vec<usize>, Vec<f64>)> {
    if !P.mines.enabled || st.mine_sectors.is_empty() {
        return None;
    }
    let N = st.N;
    let (mut t, mut c, mut v) = (Vec::with_capacity(rent.len()), Vec::with_capacity(rent.len()), Vec::with_capacity(rent.len()));
    for u in 0..rent.len() {
        let district = row(st, st.ex_sector[u]).and_then(|r| st.mine_districts.get(&(r * N + cell[u])));
        match district {
            Some(cells) if !cells.is_empty() => {
                // the last takes what the others leave, so the parts sum to the whole
                let mut left = rent[u];
                for (j, &(dc, share)) in cells.iter().enumerate() {
                    let part = if j + 1 == cells.len() { left } else { rent[u] * share };
                    left -= part;
                    t.push(town[u]);
                    c.push(dc);
                    v.push(part);
                }
            }
            _ => {
                t.push(town[u]);
                c.push(cell[u]);
                v.push(rent[u]);
            }
        }
    }
    Some((t, c, v))
}

/// The month's books: hands (the sinkers among them), output and royalty of
/// every working, added up until the year's review.
pub fn note_month(st: &mut State, P: &Params, Y: &[f64], Lex: &[f64], rent_ex: &[f64]) {
    if !P.mines.enabled || st.mine_crew_each.is_empty() {
        return;
    }
    let R = st.mine_sectors.len();
    for u in 0..st.ex_cell.len() {
        if let Some(r) = row(st, st.ex_sector[u]) {
            let c = st.ex_cell[u];
            st.mine_acc[(r, c)] += Lex[u] + if sinking(st, u) { st.mine_crew_each[st.ex_town[u]] } else { 0.0 };
            st.mine_acc[(R + r, c)] += Y[u];
            st.mine_acc[(2 * R + r, c)] += rent_ex[u];
        }
    }
    st.mine_months += 1;
}

/// The sinkers are at work at their workings: counted in the hands each unit
/// has (`st.ex_labor`), and so in the cell's, the sector's and the town's.
pub fn add_crews(st: &mut State, P: &Params) {
    if !P.mines.enabled || st.mine_crew_each.is_empty() {
        return;
    }
    for u in 0..st.ex_cell.len() {
        if sinking(st, u) {
            st.ex_labor[u] += st.mine_crew_each[st.ex_town[u]];
        }
    }
}

// ---------------------------------------------------------------- what the viewer is given

/// The map draws a land use's share of a cell as (diggings, rough ground):
/// under the port a mine's share is mostly rough ground round the workings;
/// here it is the pit head, the yard and the spoil, and nothing else.
pub fn display_split(st: &State) -> (f64, f64) {
    if st.mine_sectors.is_empty() { (0.45, 0.55) } else { (1.0, 0.0) }
}

/// One working as the viewer is given it, for a year.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Working {
    /// The cell its pit head stands on.
    pub cell: usize,
    /// The content's sector (`sectors` in the header).
    pub sector: usize,
    /// The year it was opened; 0 for one that lies abandoned.
    pub opened: i32,
    /// The year it was abandoned, if it is (or 0); for a working at work, the
    /// year the one before it on this cell was given up.
    pub left: i32,
    /// Mean hands over the year (in its first, the sinkers among them).
    pub hands: f32,
    /// What it raised over the year.
    pub out: f32,
    /// Everything raised at this pit head so far: the size of the spoil heap.
    pub heap: f32,
    /// The share of its cell its pit head, yards and spoil take.
    pub surface: f32,
    /// What it paid in royalty over the year.
    pub royalty: f32,
    /// The cells whose deposit it works, its own among them.
    pub cells: u32,
}

/// Every working there is or has been, by sector and then by cell: those at
/// work, and those abandoned whose spoil still lies there.
pub fn workings(st: &State) -> Vec<Working> {
    let N = st.N;
    let mut out = Vec::new();
    for (r, &s) in st.mine_sectors.iter().enumerate() {
        for c in 0..N {
            let k = r * N + c;
            if st.mine_opened[k] == 0 && st.mine_left[k] == 0 {
                continue;
            }
            let open = st.mine_opened[k] != 0;
            out.push(Working {
                cell: c,
                sector: s,
                opened: st.mine_opened[k],
                left: st.mine_left[k],
                hands: if open { st.mine_hands[(r, c)] as f32 } else { 0.0 },
                out: st.mine_out[(r, c)] as f32,
                heap: st.mine_heap[k] as f32,
                surface: st.mine_surface[(r, c)] as f32,
                royalty: st.mine_royalty[(r, c)] as f32,
                cells: st.mine_districts.get(&k).map_or(0, |d| d.len()) as u32,
            });
        }
    }
    out
}

/// The year's workings into the record (nothing while the mechanism is off).
pub fn snapshot(rec: &mut Record, st: &State, P: &Params, k: usize) {
    if !P.mines.enabled || st.mine_sectors.is_empty() {
        return;
    }
    if rec.workings.len() <= k {
        rec.workings.resize(k + 1, Vec::new());
    }
    rec.workings[k] = workings(st);
}

// ---------------------------------------------------------------- for the census and the sweep

/// What sector s's workings paid in royalty a month over the last complete
/// year. With the mechanism off nobody keeps it, and this is what the units
/// would pay at December's hands and this month's prices: the sale, less the
/// merchant, the carting and the wages (the machines' cost is left out).
pub fn royalty_month(st: &State, C: &Content, P: &Params, s: usize) -> f64 {
    if let Some(r) = row(st, s) {
        return st.mine_royalty.row(r).iter().sum::<f64>() / 12.0;
    }
    let (g, a) = (C.sector_good[s], C.sector_alpha[s]);
    let mut total = 0.0;
    for u in (0..st.ex_cell.len()).filter(|&u| st.ex_sector[u] == s) {
        let (t, L, lam) = (st.ex_town[u], st.ex_Lprev[u], st.ex_lam[u]);
        if L > 0.0 && lam > 0.0 {
            let (mult, _, _, _) = sector_tfp(st, C, P, s, t, &st.price, &st.fill);
            let Y = C.sector_A[s] * mult * st.ex_field[u] * lam * (L / lam).powf(1.0 - a);
            total += Y * (st.price[(t, g)] * (1.0 - P.merchant_spread) - st.ex_tau[u]) - st.wage[t] * L;
        }
    }
    total
}

/// What the sweep notes of the workings after each year, with the mechanism
/// on or off: for every extractive sector, the cells where it has a hand or
/// more at work, the surface it takes (in cells), the largest share any one
/// working takes of its cell, its share of all hands at work, and what it
/// pays in royalty a month.
#[derive(Clone, Debug, Default)]
pub struct YearNote {
    pub workings: Vec<f64>,
    pub surface: Vec<f64>,
    pub most: Vec<f64>,
    pub hands: Vec<f64>,
    pub royalty: Vec<f64>,
}

pub fn gather(st: &State, C: &Content, P: &Params) -> YearNote {
    let N = st.N;
    let mut note = YearNote::default();
    for s in (0..C.NS).filter(|&s| is_extractive(&C.sector_name[s])) {
        let mut hands = vec![0.0; N];
        for u in (0..st.ex_cell.len()).filter(|&u| st.ex_sector[u] == s) {
            hands[st.ex_cell[u]] = st.ex_Lprev[u];
        }
        let cells: Vec<usize> = (0..N).filter(|&c| hands[c] >= 1.0).collect();
        note.workings.push(cells.len() as f64);
        note.surface.push((0..N).map(|c| st.lam[(s, c)]).sum());
        note.most.push(cells.iter().map(|&c| st.lam[(s, c)]).fold(0.0, f64::max));
        let employed: f64 = st.month_stats.employment.iter().sum();
        note.hands.push(st.month_stats.ex_labor_by_sector.get(s).copied().unwrap_or(0.0) / employed.max(1e-9));
        note.royalty.push(royalty_month(st, C, P, s));
    }
    note
}

/// Facts for the sweep's table, under "Land", in the years the other facts are
/// taken in. For each extractive sector: `workings_SECTOR_Y`, the cells where it
/// has a hand or more at work; `hands_SECTOR_Y`, its share of all hands (in
/// December); `out_GOOD_per_head_Y`, what is raised a month per head; and
/// `price_GOOD_in_wages_Y`, the good's price over the wage, which can be set
/// beside another run's whatever its money is worth. For the four together:
/// `workings_Y`; `workings_share_of_cell_Y`, the land they take over their
/// number (a cell is 2 km square), and `workings_most_of_cell_Y`, the most any
/// one takes; and `royalties_share_of_output_Y`, what they pay their landowners
/// over everything made (with the mechanism off, what the units would pay at
/// December's hands: `royalty_month`).
pub fn facts(r: &crate::sim::RunResult, g: &crate::facts::Gathered) -> Vec<(String, crate::facts::Kind, f64)> {
    use crate::facts::Kind;
    let (st, C, rec) = (&r.state, &r.content, &r.rec);
    let y0 = C.start_year;
    let y1 = y0 + rec.n_years as i64 - 1;
    let series = |name: &str| st.series.index.get(name).map_or(&[][..], |&k| &st.series.data[k][..]);
    let at = |name: &str, y: i64| crate::observer::annual(series(name), y0, y);
    let sectors: Vec<usize> = (0..C.NS).filter(|&s| is_extractive(&C.sector_name[s])).collect();
    let mut out: Vec<(String, Kind, f64)> = Vec::new();
    for y in crate::facts::marks(y0, y1) {
        let Some(note) = g.mines.get((y - y0) as usize) else { continue };
        let all: f64 = note.workings.iter().sum();
        out.push((format!("workings_{y}"), Kind::Count, all));
        for (j, &s) in sectors.iter().enumerate() {
            out.push((format!("workings_{}_{y}", C.sector_name[s]), Kind::Count, note.workings[j]));
        }
        out.push((format!("workings_share_of_cell_{y}"), Kind::Share, if all > 0.0 { note.surface.iter().sum::<f64>() / all } else { f64::NAN }));
        out.push((format!("workings_most_of_cell_{y}"), Kind::Share, note.most.iter().cloned().fold(0.0, f64::max)));
        for (j, &s) in sectors.iter().enumerate() {
            out.push((format!("hands_{}_{y}", C.sector_name[s]), Kind::Share, note.hands[j]));
        }
        for &s in &sectors {
            let good = &C.good_name[C.sector_good[s]];
            out.push((format!("out_{good}_per_head_{y}"), Kind::Real, at(&format!("out_{good}"), y) / at("pop", y)));
            out.push((format!("price_{good}_in_wages_{y}"), Kind::Real, at(&format!("price_{good}"), y) / at("wage", y)));
        }
        out.push((format!("royalties_share_of_output_{y}"), Kind::Share, note.royalty.iter().sum::<f64>() / at("gdp_nom", y)));
    }
    out
}
