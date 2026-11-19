//! Firms: where they set up, when they grow, when they fail, and why. Python: `firms.py`.
//!
//! Entry is local and short-sighted. Each town's savers look at a handful of
//! candidate sites for every method the town knows, price the output and inputs
//! *at that site* (delivered-price fields), add the rent needed to outbid the
//! current land use, and back the best returns. Every new firm keeps the numbers
//! that justified it, so the map can say why a works stands where it does.

use std::collections::{BTreeMap, BTreeSet};

use crate::content::Content;
use crate::econ::{firm_coefficients, heat_mix};
use crate::families;
use crate::land::{USE_DERELICT, USE_NAMES};
use crate::mat::Mat;
use crate::names::{self, firm_name};
use crate::ndimage::{self, Mode};
use crate::params::Params;
use crate::rng::{Rng, choice_without_replacement};
use crate::state::{Reason, State};
use crate::util::{argmax, argsort, pairwise_sum, pairwise_sum_by, round_dp};
use crate::works;

pub fn learning(C: &Content, P: &Params, Q: &[f64]) -> Vec<f64> {
    (0..C.M).map(|m| P.learn_cap.min(C.m_A0[m] * (1.0 + Q[m] / C.m_Q0[m]).powf(C.m_b[m]))).collect()
}

/// Number of firms of each method within a 5x5 window of every cell.
pub fn same_method_counts(st: &State, C: &Content) -> Mat {
    let (H, W) = (st.world.H, st.world.W);
    let mut counts = Mat::zeros(C.M, H * W);
    for f in 0..st.f_alive.len() {
        if st.f_alive[f] {
            counts[(st.f_method[f], st.f_cell[f])] += 1.0;
        }
    }
    let mut out = Mat::zeros(C.M, H * W);
    for m in 0..C.M {
        if counts.row(m).iter().all(|&v| v == 0.0) {
            continue; // no firms of this method: the filtered counts are all zero
        }
        let filtered = ndimage::uniform_filter(counts.row(m), H, W, 5, Mode::Constant);
        for (o, v) in out.row_mut(m).iter_mut().zip(filtered) {
            *o = v * 25.0;
        }
    }
    out
}

pub fn update_productivity(st: &mut State, C: &Content, P: &Params) {
    let mut A = learning(C, P, &st.method_Q);
    // machines (docs/rules/machines.md): a new method inherits part of its predecessor's skill
    crate::machines::inherit_skill(st, C, P, &mut A);
    let counts = same_method_counts(st, C);
    for f in 0..st.f_alive.len() {
        if st.f_alive[f] {
            let (m, c, t) = (st.f_method[f], st.f_cell[f], st.f_town[f]);
            let k = st.know[(t, m)];
            let agglo = 1.0 + P.agglo_same * (counts[(m, c)] - 1.0).max(0.0).ln_1p();
            st.f_prod[f] = A[m] * (0.5 + 0.5 * k) * agglo;
        }
    }
    st.method_A = A;
    st.same_counts = counts;
}

/// ROI and the accounting behind it for one method on candidate cells.
#[derive(Clone, Debug, Default)]
pub struct SiteEval {
    pub roi: Vec<f64>,
    pub margin: Vec<f64>,
    pub pout: Vec<f64>,
    pub pin: Mat,
    pub a: Mat,
    pub h: Vec<f64>,
    pub hunit: Vec<f64>,
    pub l: Vec<f64>,
    pub rent_bid: Vec<f64>,
    pub cap_cost: f64,
    pub agglo: Vec<f64>,
    pub in_cost: Vec<f64>,
    pub heat_cost: Vec<f64>,
    pub labor_cost: Vec<f64>,
}

/// Return ROI and the accounting behind it for method m on candidate cells.
pub fn evaluate_sites(st: &State, C: &Content, P: &Params, i: usize, m: usize, cells: &[usize], K: f64, pbar: &Mat, wbar: &[f64]) -> SiteEval {
    let G = C.G;
    let A = st.method_A[m];
    let k = st.know[(i, m)];
    let n = cells.len();
    let agglo: Vec<f64> = cells.iter().map(|&c| 1.0 + P.agglo_same * st.same_counts[(m, c)].ln_1p()).collect();
    let prod: Vec<f64> = agglo.iter().map(|&g| A * (0.5 + 0.5 * k) * g).collect();
    let (a, h, mut l) = firm_coefficients(C, &vec![m; n], &vec![K; n], &prod);
    works::walk_to_work(st, P, cells, &mut l); // works: hands walk out from the town
    let mut pin = Mat::zeros(n, G);
    for (j, &c) in cells.iter().enumerate() {
        for g in 0..G {
            pin[(j, g)] = (pbar[(i, g)] + st.delta[(g, c)]).max(0.25 * pbar[(i, g)]);
        }
    }
    let fuels = &C.fuel_goods;
    let heat_f: Vec<f64> = fuels.iter().map(|&g| C.heat[g]).collect();
    let mut share = vec![0.0; fuels.len()];
    let hunit: Vec<f64> = (0..n)
        .map(|j| {
            let fp: Vec<f64> = fuels.iter().map(|&g| pin[(j, g)]).collect();
            heat_mix(&fp, &heat_f, P.heat_theta, &mut share)
        })
        .collect();
    let out = C.method_out[m];
    let mut pout: Vec<f64> = cells.iter().map(|&c| pbar[(i, out)] - P.freight * C.weight[out] * st.dcatch[c]).collect();
    // works: the merchant's spread comes off the price, as it will off the works' revenue
    let spread = works::entry_spread(P, pbar[(i, out)]);
    if spread != 0.0 {
        pout.iter_mut().for_each(|p| *p -= spread);
    }
    let in_cost: Vec<f64> = (0..n).map(|j| pairwise_sum_by(G, |g| a[(j, g)] * pin[(j, g)])).collect();
    let heat_cost: Vec<f64> = (0..n).map(|j| h[j] * hunit[j]).collect();
    let labor_cost: Vec<f64> = (0..n).map(|j| l[j] * wbar[i]).collect();
    let margin: Vec<f64> = (0..n).map(|j| pout[j] - in_cost[j] - heat_cost[j] - labor_cost[j]).collect();
    let foot = works::land_take(C, P, m, K);
    let rent_bid: Vec<f64> = cells.iter().map(|&c| 0.01f64.max(1.25 * st.land_rent_only[c] * foot)).collect();
    let cap_cost = K * pairwise_sum_by(G, |g| C.m_cap[(m, g)] * pbar[(i, g)]);
    let roi: Vec<f64> = (0..n).map(|j| site_roi(K, margin[j], rent_bid[j], cap_cost)).collect();
    SiteEval { roi, margin, pout, pin, a, h, hunit, l, rent_bid, cap_cost, agglo, in_cost, heat_cost, labor_cost }
}

/// The yearly return entry expects from a site: twelve months of output at 85 per
/// cent of capacity times the unit margin, less the rent bid, over the capacity's cost.
pub fn site_roi(K: f64, margin: f64, rent_bid: f64, cap_cost: f64) -> f64 {
    12.0 * (K * 0.85 * margin - rent_bid) / cap_cost.max(1e-9)
}

/// A cell a new works may stand on: not nine tenths built over, and neither
/// sea (use 0) nor derelict.
pub fn can_stand(st: &State, c: usize) -> bool {
    st.cell_K_share[c] < 0.9 && st.land_use[c] != 0 && st.land_use[c] != USE_DERELICT
}

pub fn candidate_cells(st: &State, C: &Content, P: &Params, i: usize, m: usize, rng: &mut Rng) -> Vec<usize> {
    let cells: Vec<usize> = st.town_cells[i].iter().copied().filter(|&c| can_stand(st, c)).collect();
    if cells.is_empty() {
        return cells;
    }
    let w: Vec<f64> = cells
        .iter()
        .map(|&c| {
            let water = if st.water[c] { 1.0 } else { 0.0 };
            (-st.dcatch[c] / 10.0).exp() * (1.0 + 1.5 * water) * (1.0 + 2.0 * st.same_counts[(m, c)] + 0.5 * st.any_counts[c])
                + 1e-12
        })
        .collect();
    let total = pairwise_sum(&w);
    let p: Vec<f64> = w.iter().map(|&x| x / total).collect();
    let n = P.candidates.min(cells.len());
    let mut chosen: BTreeSet<usize> = choice_without_replacement(rng, n, &p).into_iter().map(|j| cells[j]).collect();
    // resource-oriented candidates: where the bulkiest input is cheapest
    let a: Vec<f64> = (0..C.G).map(|g| C.m_in[(m, g)] * C.weight[g]).collect();
    if a.iter().cloned().fold(f64::NEG_INFINITY, f64::max) > 0.0 {
        let g = argmax(&a);
        let d: Vec<f64> = cells.iter().map(|&c| st.delta[(g, c)]).collect();
        for &j in argsort(&d).iter().take(8) {
            chosen.insert(cells[j]);
        }
    }
    chosen.into_iter().collect()
}

/// The year the works' table filled, if it did. A place in it is never given back,
/// so from then on nobody can found a works: `create_firm` and the checks before it
/// refuse without a word, and the run says so instead (a fact, `works_table_full`;
/// a line in the sweep's summary and at the end of `isostate-core run`).
pub fn table_full(st: &State) -> Option<i64> {
    (st.F_count > 0 && st.F_count >= st.f_alive.len()).then(|| st.f_born[st.F_count - 1])
}

/// What a run says when its works' table of `places` filled in `year`.
pub fn table_full_message(year: i64, places: usize) -> String {
    format!(
        "the works' table was full in {year}: all {} of its places were taken, and no works could be founded after",
        crate::util::fmt_thousands(places as i64)
    )
}

/// Open firm slot `F_count` and order its capacity. Returns the firm id, or None when the table is full.
#[allow(clippy::too_many_arguments)]
pub fn create_firm(
    st: &mut State,
    C: &Content,
    P: &Params,
    rng: &mut Rng,
    i: usize,
    m: usize,
    cell: usize,
    K: f64,
    rent_bid_one: f64,
    reason: Reason,
    year: i64,
    funding: f64,
    name: Option<String>,
    parent: Option<usize>,
) -> Option<usize> {
    let fid = st.F_count;
    if fid >= st.f_alive.len() {
        return None;
    }
    st.F_count += 1;
    st.f_alive[fid] = true;
    st.f_method[fid] = m;
    st.f_cell[fid] = cell;
    st.f_town[fid] = i;
    st.f_K[fid] = 0.0;
    for g in 0..C.G {
        st.f_order[(fid, g)] = K * C.m_cap[(m, g)];
    }
    st.f_order_K[fid] = K;
    st.f_cash[fid] = funding;
    st.f_rent[fid] = rent_bid_one;
    st.f_born[fid] = year;
    st.f_died[fid] = -1;
    st.f_u[fid] = 0.5;
    st.f_fill[fid] = 1.0;
    st.f_prod[fid] = 1.0;
    st.f_profit_acc[fid] = 0.0;
    st.f_loss_years[fid] = 0;
    let nm = match name {
        Some(n) => n,
        None => firm_name(rng, &C.method_name[m], &st.town_name[i]),
    };
    st.f_name.push(nm);
    st.f_reason.push(reason);
    st.f_parent[fid] = parent.map_or(-1, |p| p as i64);
    if st.firm_at[cell] < 0 {
        st.firm_at[cell] = fid as i64;
    }
    st.cell_K_share[cell] += works::land_take(C, P, m, K);
    works::opened(st, C, P, fid, K); // works: the year's room and the lease
    Some(fid)
}

/// The numbers that justified a firm, kept for the map inspector.
#[allow(clippy::too_many_arguments)]
pub fn reason_record(st: &State, i: usize, m: usize, cell: usize, ev: &SiteEval, j: usize, K: f64, year: i64, kind: &str) -> Reason {
    let _ = m;
    let G = ev.a.cols;
    let inputs = (0..G)
        .filter(|&g| ev.a[(j, g)] > 0.0)
        .map(|g| (g, round_dp(ev.a[(j, g)], 4), round_dp(ev.pin[(j, g)], 4)))
        .collect();
    Reason {
        kind: kind.to_string(),
        year,
        town: i,
        cell,
        K: round_dp(K, 4),
        roi: round_dp(ev.roi[j], 4),
        margin: round_dp(ev.margin[j], 4),
        pout: round_dp(ev.pout[j], 4),
        inputs,
        heat: [round_dp(ev.h[j], 4), round_dp(ev.hunit[j], 4)],
        labor: [round_dp(ev.l[j], 4), round_dp(ev.labor_cost[j] / ev.l[j].max(1e-12), 4)],
        rent: round_dp(ev.rent_bid[j], 4),
        displaced: USE_NAMES[st.land_use[cell] as usize].to_string(),
        agglo: round_dp(ev.agglo[j], 3),
        dist: round_dp(st.dcatch[cell], 2),
        n_cand: ev.roi.len(),
    }
}

pub fn town_pool(st: &State, P: &Params, i: usize) -> f64 {
    P.invest_frac_hh * st.hh_cash[i].max(0.0) + P.invest_frac_lord * st.lord_cash[i].max(0.0)
}

/// Money for an existing firm: from the family that owns it (`families`), or else
/// from its town's savers (`draw_pool`).
pub fn back_firm(st: &mut State, P: &Params, fid: usize, amount: f64) -> f64 {
    match families::owner_of_firm(st, fid) {
        Some(k) => families::draw_cash(st, k, amount),
        None => draw_pool(st, P, st.f_town[fid], amount),
    }
}

/// Take funding from a town's households and landlords in proportion to what they offered.
pub fn draw_pool(st: &mut State, P: &Params, i: usize, amount: f64) -> f64 {
    let hh = P.invest_frac_hh * st.hh_cash[i].max(0.0);
    let lo = P.invest_frac_lord * st.lord_cash[i].max(0.0);
    let tot = hh + lo;
    if tot <= 0.0 {
        return 0.0;
    }
    let amount = amount.min(tot);
    st.hh_cash[i] -= amount * hh / tot;
    st.lord_cash[i] -= amount * lo / tot;
    amount
}

struct Proposal {
    roi: f64,
    i: usize,
    m: usize,
    cells: Vec<usize>,
    ev: SiteEval,
    j: usize,
    K: f64,
}

/// New firms where returns are best.
#[allow(clippy::too_many_arguments)]
pub fn entry(st: &mut State, C: &Content, P: &Params, rng: &mut Rng, year: i64, pbar: &Mat, wbar: &[f64], max_new: Option<usize>) -> usize {
    let mut proposals: Vec<Proposal> = Vec::new();
    let mut n_tm = Mat::zeros(st.T_max, C.M);
    for f in 0..st.f_alive.len() {
        if st.f_alive[f] {
            n_tm[(st.f_town[f], st.f_method[f])] += 1.0;
        }
    }
    for i in 0..st.T_max {
        if !st.town_active[i] {
            continue;
        }
        let cap_n = 2.0 + st.pop[i] / 40000.0;
        for m in 0..C.M {
            if !(st.know[(i, m)] > 0.02) {
                continue;
            }
            let cells = candidate_cells(st, C, P, i, m, rng);
            if cells.is_empty() {
                continue;
            }
            let K = works::entry_K(st, C, P, i, m);
            // works: only where the year's room is not taken and the cell can hold it
            let cells = works::sites_with_room(st, C, P, i, m, K, cells);
            if cells.is_empty() {
                continue;
            }
            let ev = evaluate_sites(st, C, P, i, m, &cells, K, pbar, wbar);
            let j = argmax(&ev.roi);
            let need = P.hurdle * if n_tm[(i, m)] < cap_n { 1.0 } else { 3.0 };
            if ev.roi[j] > need {
                proposals.push(Proposal { roi: ev.roi[j], i, m, cells, ev, j, K });
            }
        }
    }
    // a stable sort on -roi, as Python's list.sort
    proposals.sort_by(|a, b| b.roi.total_cmp(&a.roi));
    let mut made = 0;
    let mut per_tm: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    let mut used_cells: BTreeSet<(usize, usize)> = BTreeSet::new();
    for pr in proposals {
        let Proposal { roi, i, m, cells, ev, j, mut K } = pr;
        // works: a works sized to its market comes one at a time and is not doubled
        let sized = works::sized_entry(P);
        let limit = if sized { 1 } else { 1 + (roi > 0.8) as usize };
        if per_tm.get(&(i, m)).copied().unwrap_or(0) >= limit {
            continue;
        }
        let cell = cells[j];
        if roi > 0.6 && !sized {
            K *= 2.0;
        }
        if used_cells.contains(&(cell, m)) || st.cell_K_share[cell] >= 0.9 || !works::cell_fits(st, C, P, cell, m, K) {
            continue;
        }
        let cost = K * pairwise_sum_by(C.G, |g| C.m_cap[(m, g)] * pbar[(i, g)]) * 1.15;
        if town_pool(st, P, i) < cost || st.F_count >= st.f_alive.len() {
            continue;
        }
        let got = draw_pool(st, P, i, cost);
        let rent_bid_one = ev.rent_bid[j];
        let reason = reason_record(st, i, m, cell, &ev, j, K, year, "new");
        let Some(fid) = create_firm(st, C, P, rng, i, m, cell, K, rent_bid_one, reason, year, got, None, None) else {
            break;
        };
        used_cells.insert((cell, m));
        *per_tm.entry((i, m)).or_insert(0) += 1;
        made += 1;
        st.new_firms_year.push(fid);
        if max_new.is_some_and(|mx| made >= mx) {
            break;
        }
    }
    made
}

/// Grow profitable firms, shrink idle ones, close the hopeless.
pub fn review(st: &mut State, C: &Content, P: &Params, rng: &mut Rng, year: i64, pbar: &Mat, wbar: &[f64]) -> Vec<usize> {
    let mut alive: Vec<usize> = (0..st.f_alive.len()).filter(|&f| st.f_alive[f]).collect();
    let mut closed = Vec::new();
    works::new_year(st, C, P, pbar, &mut alive); // works: the year's room, the best earners served first
    for &fid in &alive {
        let (i, m) = (st.f_town[fid], st.f_method[fid]);
        let K = st.f_K[fid];
        let unit_cap = pairwise_sum_by(C.G, |g| C.m_cap[(m, g)] * pbar[(i, g)]);
        let capval = K.max(1e-9) * unit_cap;
        let roi = st.f_profit_acc[fid] / capval.max(1e-9);
        let u = st.f_u[fid];
        let building = works::building(P, K, st.f_order_K[fid]);
        if st.f_cash[fid] < 0.0 && year - st.f_born[fid] <= 3 {
            let need = -st.f_cash[fid];
            st.f_cash[fid] += back_firm(st, P, fid, need);
        }
        if K > 0.0 {
            st.f_K[fid] = K * (1.0 - P.depreciation);
        }
        if roi < 0.0 && u < 0.15 && !building {
            st.f_loss_years[fid] += 1;
        } else {
            st.f_loss_years[fid] = 0;
        }
        let insolvent = st.f_cash[fid] < -(6.0 * st.f_cost_m[fid]).max(0.4 * capval) - 1.0;
        let tiny = st.f_K[fid] < 0.03 * C.m_K0[m] && !building;
        let stale = building && year - st.f_born[fid] > 6 && K <= 0.0;
        if insolvent || tiny || stale || st.f_loss_years[fid] >= 3 {
            closed.push(fid);
            continue;
        }
        if u < 0.25 && !building && K > 0.0 {
            st.f_K[fid] *= 0.85;
        }
        if roi > P.hurdle && u > 0.65 && !building && K > 0.0 {
            // works: no more than the year's room in this town and trade allows
            let dK = works::growth_allowed(st, C, P, fid, K * 0.5f64.min(0.2 * roi / P.hurdle));
            if dK <= 0.0 {
                // works: the room is taken; wait for next year
            } else if K + dK > works::max_K(C, P, m) || !works::cell_fits(st, C, P, st.f_cell[fid], m, dK) {
                // works: a sister works it already has takes the growth, if one has room
                if !works::extend_sister(st, C, P, fid, dK, unit_cap) {
                    spawn_sister(st, C, P, rng, fid, year, pbar, wbar, dK);
                }
            } else {
                let cost = dK * unit_cap;
                let short = cost * 1.1 - st.f_cash[fid];
                if short > 0.0 {
                    st.f_cash[fid] += back_firm(st, P, fid, short);
                }
                if st.f_cash[fid] > 0.5 * cost {
                    for g in 0..C.G {
                        st.f_order[(fid, g)] += dK * C.m_cap[(m, g)];
                    }
                    st.f_order_K[fid] += dK;
                    works::ordered(st, C, P, fid, dK); // works: the year's room and the lease
                }
            }
        }
    }
    for &fid in &closed {
        close_firm(st, C, P, fid, year);
    }
    for &f in &alive {
        st.f_profit_acc[f] = 0.0;
        st.f_out_acc[f] = 0.0;
    }
    closed
}

/// A works at full size grows by opening a new site next door. `dK` is the
/// growth it would have ordered (under the port a sister works is always of the
/// usual size).
#[allow(clippy::too_many_arguments)]
pub fn spawn_sister(st: &mut State, C: &Content, P: &Params, rng: &mut Rng, parent: usize, year: i64, pbar: &Mat, wbar: &[f64], dK: f64) -> Option<usize> {
    if !families::may_spawn(st, C, parent) {
        return None; // a family works opens sister works only under a growing branch
    }
    let (i, m) = (st.f_town[parent], st.f_method[parent]);
    let (H, W) = (st.world.H, st.world.W);
    let (y, x) = (st.f_cell[parent] / W, st.f_cell[parent] % W);
    let mut cand = Vec::new();
    for dy in -2isize..=2 {
        for dx in -2isize..=2 {
            let (ny, nx) = (y as isize + dy, x as isize + dx);
            if ny >= 0 && ny < H as isize && nx >= 0 && nx < W as isize && (dy != 0 || dx != 0) {
                let c = ny as usize * W + nx as usize;
                if st.cell_K_share[c] < 0.7 && st.catch[c] == i as i64 && st.land_use[c] != 0 {
                    cand.push(c);
                }
            }
        }
    }
    // works: a sister works the size of the growth step, on a cell with room for it
    let K = works::sister_K(C, P, m, dK);
    cand.retain(|&c| works::cell_fits(st, C, P, c, m, K));
    if cand.is_empty() {
        return None;
    }
    let ev = evaluate_sites(st, C, P, i, m, &cand, K, pbar, wbar);
    let j = argmax(&ev.roi);
    if ev.roi[j] < P.hurdle {
        return None;
    }
    let cost = K * pairwise_sum_by(C.G, |g| C.m_cap[(m, g)] * pbar[(i, g)]) * 1.15;
    let owner = families::owner_of_firm(st, parent);
    let mut fund = (st.f_cash[parent].max(0.0) * 0.8).min(cost);
    st.f_cash[parent] -= fund;
    if fund < cost {
        fund += back_firm(st, P, parent, cost - fund);
    }
    if fund < 0.6 * cost || st.F_count >= st.f_alive.len() {
        // Python sends the money to the town's households; a family keeps its own
        match owner {
            Some(k) => st.families[k].cash += fund,
            None => st.hh_cash[i] += fund,
        }
        return None;
    }
    let rent_bid_one = ev.rent_bid[j];
    let reason = reason_record(st, i, m, cand[j], &ev, j, K, year, "expansion");
    let nm = &st.f_name[parent];
    let base = nm.split(" No.").next().unwrap_or(nm).to_string();
    // works (docs/rules/works.md): only the works of this very name are counted;
    // the port counts every name that starts with it, as the Python does
    let count = if P.works.enabled { names::works_of_name(&st.f_name, &base) } else { st.f_name.iter().filter(|n| n.starts_with(&base)).count() };
    let name = format!("{base} No. {}", count + 1);
    let fid = create_firm(st, C, P, rng, i, m, cand[j], K, rent_bid_one, reason, year, fund, Some(name), Some(parent));
    if let Some(f) = fid {
        st.new_firms_year.push(f);
        if let Some(k) = owner {
            st.f_owner[f] = k as i64; // a family's works grow into the family's sister works
        }
    }
    fid
}

pub fn close_firm(st: &mut State, C: &Content, P: &Params, fid: usize, year: i64) {
    let i = st.f_town[fid];
    let cash = st.f_cash[fid];
    if cash >= 0.0 {
        match families::owner_of_firm(st, fid) {
            Some(k) => {
                st.families[k].cash += cash;
                st.f_paid[fid] += cash;
            }
            None => st.hh_cash[i] += cash,
        }
    } else {
        st.merch_cash[i] += cash; // creditors take the loss
    }
    let held = works::land_held(st, C, P, fid); // works: the land it gives up
    // unfinished orders: nothing more is bought
    st.f_cash[fid] = 0.0;
    st.f_order.row_mut(fid).iter_mut().for_each(|v| *v = 0.0);
    st.f_order_K[fid] = 0.0;
    st.f_alive[fid] = false;
    st.f_died[fid] = year;
    let c = st.f_cell[fid];
    let m = st.f_method[fid];
    st.cell_K_share[c] = (st.cell_K_share[c] - held).max(0.0);
    if works::leaves_derelict(C, P, m, st.f_K[fid]) && st.cell_K_share[c] < 0.05 && !st.urban[c] {
        st.derelict_timer[c] = 6;
    }
    st.closed_year.push(fid);
}
