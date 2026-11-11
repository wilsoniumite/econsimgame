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
//! - A. the deposit is the fixed factor. A working has the whole of its cell's
//!   deposit, so its output is `A * f * L^(1 - alpha)`: the same function as
//!   before with the cell's deposit in place of a share of its surface. What is
//!   left of the sale after the merchant, the carting, the machines and the
//!   wages is the royalty, paid to whoever owns the cell exactly as rent is.
//! - B. a working keeps a standing crew (`crew` hands at the town's wage) that
//!   raises nothing: the banksman, the pumps, the roads kept open. So a pit has
//!   a least size, and a deposit too poor or too far from a market to carry a
//!   crew is left in the ground.
//! - C. a working is opened where the royalty its deposit would have paid over
//!   the last few years' prices, before the crew, is `1 + open_margin` crews'
//!   wages or more, and abandoned when that falls below one crew's wages.
//! - D. it goes deeper. Each unit raised lowers the richness the hands meet:
//!   `f / (1 + dug / (reserve * f))`, so the near deposits are worked first and
//!   the workings move on as they tire.
//! - E. the surface it takes is by its hands (`hands_per_cell` fill a cell, as
//!   the works' land is counted), taken out of the cell before the farms,
//!   pastures and woods share the rest.
//!
//! Every payment here is one the monthly loop already makes: the crew's wages go
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
    /// B. Hands every working keeps whatever it raises. 0: no standing crew, and
    /// every deposit that pays anything at all is worked.
    pub crew: f64,
    /// C. A working is opened where the royalty promised, before the crew, is at
    /// least `1 + open_margin` crews' wages; it is abandoned below one.
    pub open_margin: f64,
    /// C. How much of this year's promise goes into what a deposit is believed to
    /// pay (the rest is what was believed last year). 1: this year's prices alone.
    pub memory: f64,
    /// D. The output, per unit of richness, at which a working's richness has
    /// halved. 0: a working never tires.
    pub reserve: f64,
    /// E. Hands that fill a cell with pit heads, yards and spoil.
    pub hands_per_cell: f64,
    /// E. The most of a cell its workings' surface may take.
    pub cell_limit: f64,
}

impl Default for MineParams {
    /// Off: what the port runs with. The numbers are the game's, so that
    /// `--set mines.enabled=true` alone measures the mechanism as it is played.
    fn default() -> MineParams {
        MineParams { enabled: false, crew: 12.0, open_margin: 1.0, memory: 0.3, reserve: 1.5e6, hands_per_cell: 2000.0, cell_limit: 0.5 }
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
    st.mine_dug = Mat::zeros(R, N);
    st.mine_promise = Mat::zeros(R, N);
    st.mine_surface = Mat::zeros(R, N);
    st.mine_hands = Mat::zeros(R, N);
    st.mine_out = Mat::zeros(R, N);
    st.mine_royalty = Mat::zeros(R, N);
    st.mine_acc = Mat::zeros(3 * R, N);
    st.mine_months = 0;
}

// ---------------------------------------------------------------- D. depth

/// The richness the hands of a working meet, where the deposit's is f and
/// `dug` has been raised from it so far. A rich deposit has more in it, so it
/// tires more slowly; a cell nobody has worked gives f.
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
/// hands, what it raised and what it paid its landowner.
fn close_books(st: &mut State) {
    let (R, N) = (st.mine_sectors.len(), st.N);
    let months = st.mine_months.max(1) as f64;
    for r in 0..R {
        for c in 0..N {
            let out = st.mine_acc[(R + r, c)];
            st.mine_hands[(r, c)] = st.mine_acc[(r, c)] / months;
            st.mine_out[(r, c)] = out;
            st.mine_royalty[(r, c)] = st.mine_acc[(2 * R + r, c)];
            st.mine_dug[(r, c)] += out;
        }
    }
    st.mine_acc.fill(0.0);
    st.mine_months = 0;
}

/// The year's review of the workings, on the year's mean prices and wages:
/// which deposits are opened and which abandoned (C), and the surface each
/// working takes (E). Returns that surface cell by cell, for the land rule to
/// leave out of what the farms, pastures and woods share; all zeros while the
/// mechanism is off.
///
/// `taken` are the cells no land use may have (town ground, a cell the works
/// have filled, dereliction, land with no market): a working there is given up.
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
    let (R, T) = (st.mine_sectors.len(), st.T_max);
    close_books(st);
    for r in 0..R {
        let s = st.mine_sectors[r];
        let (g, a) = (C.sector_good[s], C.sector_alpha[s]);
        let field = st.world.field(&C.sector_field[s]).to_vec();
        // the sector's productivity and what its machines cost a unit depend only on the town
        let mut per_town: Vec<Option<(f64, f64)>> = vec![None; T];
        for c in 0..N {
            let k = r * N + c;
            let open = st.mine_opened[k] != 0;
            if st.catch[c] < 0 || taken[c] || !(field[c] > 0.0) {
                if open {
                    st.mine_opened[k] = 0;
                    st.mine_left[k] = year as i32;
                }
                st.mine_promise[(r, c)] = 0.0;
                continue;
            }
            let t = st.catch[c] as usize;
            let w = wbar[t];
            if !(w > 0.0) {
                // a town founded this year has no year's wage yet (FAMILIES.md,
                // "Surprises"): its workings stay as they are until it has one
                continue;
            }
            let (A, in_unit) = match per_town[t] {
                Some(v) => v,
                None => {
                    let (mult, _, _, in_unit) = sector_tfp(st, C, P, s, t, pbar, &st.fill);
                    let v = (C.sector_A[s] * mult, in_unit);
                    per_town[t] = Some(v);
                    v
                }
            };
            // the royalty the deposit would pay at the hands that suit the year's
            // prices, before the crew, counted in the town's wages
            let f = worn(P, field[c], st.mine_dug[(r, c)]);
            let npr = pbar[(t, g)] - P.freight * C.weight[g] * st.dcatch[c] - in_unit;
            let promise = if npr > 0.0 {
                let L = ((1.0 - a) * A * f * npr / w).powf(1.0 / a);
                let Y = A * f * L.powf(1.0 - a);
                ((Y * (npr - P.merchant_spread * pbar[(t, g)]) - w * L) / w).max(0.0)
            } else {
                0.0
            };
            let was = st.mine_promise[(r, c)];
            let believed = if initial { promise } else { was + m.memory * (promise - was) };
            st.mine_promise[(r, c)] = believed;
            if open && believed < m.crew {
                st.mine_opened[k] = 0;
                st.mine_left[k] = year as i32;
            } else if !open && promise > 0.0 && believed.min(promise) >= (1.0 + m.open_margin) * m.crew {
                // not into a falling market: this year's prices must promise it too
                st.mine_opened[k] = year as i32;
            }
        }
    }
    // E. the surface: by last year's hands, and a new working's by its crew
    for c in 0..N {
        let mut tot = 0.0;
        for r in 0..R {
            let v = if st.mine_opened[r * N + c] != 0 && m.hands_per_cell > 0.0 {
                st.mine_hands[(r, c)].max(m.crew) / m.hands_per_cell
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
/// surface is `lam`: for a working, the whole of the cell's deposit if it is
/// open and nothing if not.
pub fn unit_share(st: &State, P: &Params, s: usize, c: usize, lam: f64) -> f64 {
    if !P.mines.enabled {
        return lam;
    }
    match row(st, s) {
        Some(r) => (st.mine_opened[r * st.N + c] != 0) as u8 as f64,
        None => lam,
    }
}

/// The richness the unit of sector s on cell c works this year: the deposit's,
/// less what its working has gone through (D).
pub fn unit_field(st: &State, P: &Params, s: usize, c: usize, f: f64) -> f64 {
    if !P.mines.enabled {
        return f;
    }
    match row(st, s) {
        Some(r) => worn(P, f, st.mine_dug[(r, c)]),
        None => f,
    }
}

/// The map's rent counts what each cell's workings paid their landowner a month
/// over the last year. (The rent a works reckons with when it looks for a site,
/// `land_rent_only`, does not: a yard on the surface displaces no seam.)
pub fn add_royalties(st: &State, P: &Params, rent: &mut [f64]) {
    if !P.mines.enabled {
        return;
    }
    for r in 0..st.mine_sectors.len() {
        for (c, x) in rent.iter_mut().enumerate() {
            if st.mine_opened[r * st.N + c] != 0 && !st.urban[c] {
                *x += (st.mine_royalty[(r, c)] / 12.0).max(0.0);
            }
        }
    }
}

/// A town does not build over an iron mine or a colliery at work (under the
/// port, over a cell that is mostly one).
pub fn blocks_town(st: &State, C: &Content, P: &Params, c: usize) -> bool {
    P.mines.enabled && st.mine_sectors.iter().enumerate().any(|(r, &s)| C.sector_mine[s] && st.mine_opened[r * st.N + c] != 0)
}

// ---------------------------------------------------------------- the month (econ::step_month)

/// B. The standing crews are not for hire: they come out of each town's labour
/// supply before the wage is sought. (Should a town's workings ever ask for more
/// than a quarter of its hands, the crews are thinned to that: a safety valve
/// for a town founded on top of a coalfield, not a rule anyone should meet.)
pub fn take_crews(st: &mut State, P: &Params, supply: &mut [f64]) {
    if !P.mines.enabled || st.mine_sectors.is_empty() {
        return;
    }
    let T = supply.len();
    let mut n = vec![0.0; T];
    for u in 0..st.ex_cell.len() {
        if row(st, st.ex_sector[u]).is_some() {
            n[st.ex_town[u]] += 1.0;
        }
    }
    let mut each = vec![0.0; T];
    for t in 0..T {
        if n[t] > 0.0 && st.town_active[t] {
            each[t] = P.mines.crew.min(0.25 * supply[t] / n[t]).max(0.0);
            supply[t] -= each[t] * n[t];
        }
    }
    st.mine_crew_each = each;
}

/// B. The crews are paid at the town's wage out of what their working sells:
/// added to each working's wage bill, so the royalty is what is left after them.
pub fn pay_crews(st: &State, P: &Params, w: &[f64], wage_ex: &mut [f64]) {
    if !P.mines.enabled || st.mine_crew_each.is_empty() {
        return;
    }
    for (u, x) in wage_ex.iter_mut().enumerate() {
        if row(st, st.ex_sector[u]).is_some() {
            let t = st.ex_town[u];
            *x += st.mine_crew_each[t] * w[t];
        }
    }
}

/// The month's books: hands (the crew among them), output and royalty of every
/// working, added up until the year's review.
pub fn note_month(st: &mut State, P: &Params, Y: &[f64], Lex: &[f64], rent_ex: &[f64]) {
    if !P.mines.enabled || st.mine_crew_each.is_empty() {
        return;
    }
    let R = st.mine_sectors.len();
    for u in 0..st.ex_cell.len() {
        if let Some(r) = row(st, st.ex_sector[u]) {
            let c = st.ex_cell[u];
            st.mine_acc[(r, c)] += Lex[u] + st.mine_crew_each[st.ex_town[u]];
            st.mine_acc[(R + r, c)] += Y[u];
            st.mine_acc[(2 * R + r, c)] += rent_ex[u];
        }
    }
    st.mine_months += 1;
}

/// The crews are at work at their workings: counted in the hands each unit has
/// (`st.ex_labor`), and so in the cell's, the sector's and the town's.
pub fn add_crews(st: &mut State, P: &Params) {
    if !P.mines.enabled || st.mine_crew_each.is_empty() {
        return;
    }
    for u in 0..st.ex_cell.len() {
        if row(st, st.ex_sector[u]).is_some() {
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
    pub cell: usize,
    /// The content's sector (`sectors` in the header).
    pub sector: usize,
    /// The year it was opened; 0 for one that lies abandoned.
    pub opened: i32,
    /// The year it was abandoned, if it is (or 0); for a working at work, the
    /// year the one before it on this cell was given up.
    pub left: i32,
    /// Mean hands over the year, the standing crew among them.
    pub hands: f32,
    /// What it raised over the year.
    pub out: f32,
    /// Everything raised from this cell's deposit so far: the size of the spoil heap.
    pub dug: f32,
    /// The share of its cell its pit head, yards and spoil take.
    pub surface: f32,
    /// What it paid its landowner over the year.
    pub royalty: f32,
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
                dug: st.mine_dug[(r, c)] as f32,
                surface: st.mine_surface[(r, c)] as f32,
                royalty: st.mine_royalty[(r, c)] as f32,
            });
        }
    }
    out
}

/// The year's workings into the record (nothing while the mechanism is off).
pub fn snapshot(rec: &mut Record, st: &State, P: &Params, k: usize) {
    if !P.mines.enabled {
        return;
    }
    if rec.workings.len() <= k {
        rec.workings.resize(k + 1, Vec::new());
    }
    rec.workings[k] = workings(st);
}

// ---------------------------------------------------------------- for the census and the sweep

/// What sector s's workings paid their landowners a month over the last
/// complete year. With the mechanism off nobody keeps it, and this is what the
/// units would pay at December's hands and this month's prices: the sale, less
/// the merchant, the carting and the wages (the machines' cost is left out).
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
/// working takes of its cell, and what it pays its landowners a month.
#[derive(Clone, Debug, Default)]
pub struct YearNote {
    pub workings: Vec<f64>,
    pub surface: Vec<f64>,
    pub most: Vec<f64>,
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
        note.royalty.push(royalty_month(st, C, P, s));
    }
    note
}
