//! Yearly spatial update: transport, catchments, delivered prices and land use.
//! Python: `land.py`.
//!
//! Land goes to whoever bids the most rent for it. For land-based sectors the bid
//! is the Cobb-Douglas land rent a cell would earn at the town's prices and wage,
//! after carting costs. Industry and towns outbid farms near markets; farms push
//! outward; forests survive where nobody can pay for the land.

use crate::content::Content;
use crate::econ::sector_tfp;
use crate::mat::Mat;
use crate::params::Params;
use crate::state::State;
use crate::transport as tr;
use crate::util::{argsort, bool_f, pairwise_sum};

pub const USE_SEA: u8 = 0;
pub const USE_WILD: u8 = 1;
pub const USE_SECTOR0: u8 = 2;
pub const USE_INDUSTRY: u8 = 9;
pub const USE_URBAN: u8 = 10;
pub const USE_DERELICT: u8 = 11;
pub const USE_NAMES: [&str; 12] =
    ["sea", "wild", "farm", "pasture", "forestry", "quarry", "claypit", "ironmine", "coalmine", "industry", "urban", "derelict"];

/// Rebuild the cost graph, the shortest paths from every town, catchments and
/// town-to-town costs.
///
/// Python ends this function with `delta_fields`, but every caller (`init_state`
/// and `yearly`) recomputes the delta fields before anything reads them, so that
/// pass (one Dijkstra per good) is left to the callers here.
pub fn update_transport(st: &mut State, _C: &Content, _P: &Params) {
    let N = st.N;
    let T = st.T_max;
    let mut port = vec![false; N];
    for i in 0..T {
        if st.town_active[i] {
            let n = st.town_node[i];
            port[n] = st.world.coast[n];
            // land: a town on a lake's shore has a harbour on the lake
            if let Some(lf) = &st.world.landform {
                port[n] = port[n] || lf.shore[n];
            }
        }
    }
    let mut graph = tr::build_graph(&st.world, &st.road, &st.rail, &port, &st.cover);
    // canals (docs/rules/canals.md): the canals open so far, as a layer of water over the cells
    crate::canals::float(st, _P, &mut graph);
    // roads (docs/probes/dx-roads.md): a trust's road costs its toll to pass
    crate::roads::tolls(st, _P, &mut graph);
    let first = (0..T).find(|&i| st.town_active[i]).expect("no active town");
    let nodes: Vec<usize> = (0..T).map(|i| if st.town_active[i] { st.town_node[i] } else { st.town_node[first] }).collect();
    let (dist, pred, settle) = tr::town_paths(&graph, &nodes, &st.town_active);
    let (catch, dmin) = tr::catchments(&dist, &st.town_active, &st.world.land);
    let mut tc = Mat::filled(T, T, f64::INFINITY);
    for i in 0..T {
        for j in 0..T {
            if st.town_active[i] && st.town_active[j] {
                tc[(i, j)] = dist[(i, nodes[j])];
            }
        }
    }
    st.graph = graph;
    st.dist = dist;
    st.pred = pred;
    st.settle = settle;
    st.catch = catch;
    st.dcatch = dmin;
    st.town_cost = tc;
}

/// Delivered price of each good at each cell relative to its town price.
///
/// A cell next to a colliery gets coal cheaper than the town price; a cell far
/// out pays carting from town. One shortest-path pass per good, from a virtual
/// source joined to every producing cell.
pub fn delta_fields(st: &mut State, C: &Content, P: &Params) {
    let (G, N) = (C.G, st.N);
    let d = &st.dcatch;
    let mut delta = Mat::zeros(G, N);
    let mut is_src = vec![false; N];
    for g in 0..G {
        is_src.iter_mut().for_each(|x| *x = false);
        let mut any = false;
        for (u, &s) in st.ex_sector.iter().enumerate() {
            if C.sector_good[s] == g {
                is_src[st.ex_cell[u]] = true;
                any = true;
            }
        }
        for f in 0..st.f_alive.len() {
            if st.f_alive[f] && C.method_out[st.f_method[f]] == g && st.f_K[f] > 0.0 {
                is_src[st.f_cell[f]] = true;
                any = true;
            }
        }
        let mut base: Vec<f64> = d.iter().map(|&x| if x.is_finite() { x } else { 1e6 }).collect();
        if any {
            // np.unique: ascending cells; only cells with a market
            let s: Vec<usize> = (0..N).filter(|&c| is_src[c] && d[c].is_finite()).collect();
            if !s.is_empty() {
                let offsets: Vec<f64> = s.iter().map(|&c| -d[c]).collect();
                let E = tr::multi_source_field(&st.graph, &s, &offsets);
                for c in 0..N {
                    let e = if E[c].is_finite() { E[c] } else { 1e6 };
                    base[c] = base[c].min(e);
                }
            }
        }
        let k = P.freight * C.weight[g];
        for c in 0..N {
            delta[(g, c)] = k * base[c];
        }
    }
    st.delta = delta;
}

/// Each sector's resource field: the world's, or the current tree cover for forestry.
pub fn sector_fields(st: &State, C: &Content) -> Vec<Vec<f64>> {
    C.sector_field.iter().map(|name| if name == "forest" { st.cover.clone() } else { st.world.field(name).to_vec() }).collect()
}

/// Monthly land rent each land-based sector could pay on every cell.
pub fn sector_bids(st: &State, C: &Content, P: &Params, pbar: &Mat, wbar: &[f64]) -> Mat {
    let (N, NS, T) = (st.N, C.NS, st.T_max);
    let catch = &st.catch;
    let d = &st.dcatch;
    let ct: Vec<usize> = catch.iter().map(|&c| if c >= 0 { c as usize } else { 0 }).collect();
    let mut bids = Mat::zeros(NS, N);
    let fields = sector_fields(st, C);
    for s in 0..NS {
        let g = C.sector_good[s];
        let a = C.sector_alpha[s];
        // sector_tfp and the wage factor depend only on the town
        let mut per_town: Vec<Option<(f64, f64, f64)>> = vec![None; T];
        let f = &fields[s];
        for c in 0..N {
            let ok = catch[c] >= 0;
            let t = ct[c];
            let (A, in_unit, wage_term) = match per_town[t] {
                Some(v) => v,
                None => {
                    let (mult, _, _, in_unit) = sector_tfp(st, C, P, s, t, pbar, &st.fill);
                    let w = wbar[t].max(1e-6);
                    let v = (C.sector_A[s] * mult, in_unit, a * ((1.0 - a) / w).powf((1.0 - a) / a));
                    per_town[t] = Some(v);
                    v
                }
            };
            let dd = if ok { d[c] } else { 1e9 };
            let npr = pbar[(t, g)] - P.freight * C.weight[g] * dd - in_unit;
            let good = ok && npr > 0.0 && f[c] > 0.0;
            if good {
                let base = A * f[c] * npr;
                bids[(s, c)] = wage_term * base.powf(1.0 / a);
            }
        }
    }
    bids
}

/// Towns grow outward from their centre along the cheapest ground.
pub fn update_urban(st: &mut State, C: &Content, P: &Params, year: i64, wbar: &[f64]) {
    let (N, T) = (st.N, st.T_max);
    let mut urban = vec![false; N];
    let mut res = vec![0.0; N];
    st.urban_rent = vec![0.0; T];
    st.cell_urban_rent = vec![0.0; N];
    let density = P.urban_density0 + P.urban_density_growth * (year - C.start_year) as f64;
    let blocked: Vec<bool> = st
        .land_use
        .iter()
        .map(|&u| u == USE_INDUSTRY || u == USE_SEA || (u >= USE_SECTOR0 + 5 && u <= USE_SECTOR0 + 6))
        .enumerate()
        // mines: nor over a pit head at work, which no longer shows as the cell's main use
        .map(|(c, b)| b || crate::mines::blocks_town(st, P, c))
        .collect();
    for i in 0..T {
        if !st.town_active[i] {
            continue;
        }
        let share = 0.04 + 0.8 * st.ind_labor[i] / (P.participation * st.pop[i]).max(1.0);
        let U = st.pop[i] * share.clamp(0.06, 0.95);
        let n = 1usize.max((U / density).ceil() as usize);
        let node = st.town_node[i];
        let cells: Vec<usize> =
            (0..N).filter(|&c| st.catch[c] == i as i64 && (!blocked[c] || c == node)).collect();
        let score: Vec<f64> = cells
            .iter()
            .map(|&c| {
                if c == node {
                    1e9
                } else {
                    -st.dist[(i, c)] + 0.8 * st.world.jitter[c] + 0.4 * bool_f(st.world.coast[c] || st.world.river[c])
                }
            })
            .collect();
        let k = n.min(cells.len());
        let neg: Vec<f64> = score.iter().map(|&s| -s).collect();
        let top: Vec<usize> = argsort(&neg)[..k].iter().map(|&j| cells[j]).collect();
        let dtop: Vec<f64> = top.iter().map(|&c| st.dist[(i, c)]).collect();
        let dmax = dtop.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let wgt: Vec<f64> = dtop.iter().map(|&d| (-d / (dmax + 1.0).max(1.0) * 2.0).exp()).collect();
        let wsum = pairwise_sum(&wgt);
        let fringe = dmax + 1.0;
        let mut cell_rent = vec![0.0; k];
        for (j, &c) in top.iter().enumerate() {
            let r = U * wgt[j] / wsum;
            urban[c] = true;
            res[c] = r;
            cell_rent[j] = r * P.urban_rent_rate * wbar[i] * (1.0 - dtop[j] / fringe).max(0.0);
            st.cell_urban_rent[c] = cell_rent[j];
        }
        st.urban_rent[i] = pairwise_sum(&cell_rent);
        st.urban_pop[i] = U;
    }
    st.urban = urban;
    st.urban_res = res;
    st.urban_density = density;
}

/// Each cell's land is shared between sectors in proportion to bid^(1/theta).
///
/// Shares move part of the way toward that target every year, so land use
/// shifts smoothly as prices change instead of flipping wholesale. The last
/// share is "wild": land not worth the fixed cost of working it.
pub fn update_land_use(st: &mut State, C: &Content, P: &Params, year: i64, pbar: &Mat, wbar: &[f64], initial: bool) {
    let (N, NS) = (st.N, C.NS);
    // mines (docs/rules/mines.md): a working bids for no surface
    let bids = crate::mines::surface_bids(C, P, sector_bids(st, C, P, pbar, wbar));
    let ct: Vec<usize> = st.catch.iter().map(|&c| if c >= 0 { c as usize } else { 0 }).collect();
    let wild_bid: Vec<f64> = ct.iter().map(|&t| P.land_fixed_cost * wbar[t].max(1e-6)).collect();

    if P.towns.enabled {
        // towns: a town keeps its ground and grows from it, in place of the yearly re-pick
        crate::towns::update_urban(st, C, P, year, wbar, &bids, initial);
    } else {
        update_urban(st, C, P, year, wbar);
    }

    for t in st.derelict_timer.iter_mut() {
        *t = (*t - 1).max(0);
    }
    let land = st.world.land.clone();
    let ks = st.cell_K_share.clone();
    let taken: Vec<bool> = (0..N)
        .map(|c| st.urban[c] || ks[c] >= 0.6 || st.derelict_timer[c] > 0 || !land[c] || st.catch[c] < 0)
        .collect();
    // mines: the year's review of the workings, and the surface they take of each
    // cell (zeros while the mechanism is off), left out of what the rest share
    let ms = crate::mines::review(st, C, P, year, pbar, wbar, &taken, initial);

    let mut target = Mat::zeros(NS + 1, N);
    let mut logb = vec![0.0; NS + 1];
    for c in 0..N {
        let mut mx = f64::NEG_INFINITY;
        for r in 0..=NS {
            let b = if r < NS { bids[(r, c)] } else { wild_bid[c] };
            let b = if b > 0.0 { b } else { 0.0 };
            logb[r] = if b > 0.0 { b.max(1e-12).ln() / P.land_theta } else { f64::NEG_INFINITY };
            mx = mx.max(logb[r]);
        }
        let shift = if mx.is_finite() { mx } else { 0.0 };
        let mut zsum = 0.0;
        for r in 0..=NS {
            let z = (logb[r] - shift).exp();
            target[(r, c)] = z;
            zsum += z;
        }
        for r in 0..=NS {
            target[(r, c)] = target[(r, c)] / zsum.max(1e-12) * (1.0 - ks[c] - ms[c]);
        }
        if taken[c] {
            for r in 0..=NS {
                target[(r, c)] = 0.0;
            }
            if land[c] && !st.urban[c] && ks[c] < 0.6 {
                target[(NS, c)] = 1.0;
            }
        }
    }

    let lam = st.lam.clone();
    let mut lam_new = Mat::zeros(NS + 1, N);
    for c in 0..N {
        for r in 0..=NS {
            let v = if initial || taken[c] { target[(r, c)] } else { lam[(r, c)] + P.land_adjust * (target[(r, c)] - lam[(r, c)]) };
            lam_new[(r, c)] = if v < 0.01 { 0.0 } else { v };
        }
        let mut tot = 0.0;
        for r in 0..=NS {
            tot += lam_new[(r, c)];
        }
        // (mines: a pit head may stand on a cell left derelict, whose waste is then the rest)
        let room = if taken[c] { 1.0 - ms[c] } else { 1.0 - ks[c] - ms[c] };
        for r in 0..=NS {
            lam_new[(r, c)] = if tot > 0.0 { lam_new[(r, c)] / tot.max(1e-12) * room } else { 0.0 };
        }
    }
    // mines: the workings' surface goes into the shares, which sum as before
    crate::mines::write_surface(st, P, &mut lam_new);

    // forest: cover lives on the woodland and wild share; clearing yields timber
    let fi = st.forestry_sector;
    let timber = C.gi("timber");
    let mut c_new = vec![0.0; N];
    let mut cleared = vec![0.0; N];
    let mut wood_share = vec![0.0; N];
    for c in 0..N {
        let pot = st.world.forest_pot[c];
        wood_share[c] = lam_new[(fi, c)] + lam_new[(NS, c)];
        let old_wood = if initial { wood_share[c] } else { lam[(fi, c)] + lam[(NS, c)] };
        let cv = st.cover[c];
        let regrow = P.forest_regrow * cv * (1.0 - cv / pot.max(1e-6)) + 0.01 * (pot - cv);
        c_new[c] = (cv + regrow * bool_f(wood_share[c] > 0.0)).max(0.0).min(pot);
        cleared[c] = (old_wood - wood_share[c]).max(0.0) * c_new[c];
    }
    if !initial && cleared.iter().any(|&x| x > 0.0) {
        for c in 0..N {
            if st.catch[c] >= 0 && cleared[c] > 0.0 {
                st.inv[(st.catch[c] as usize, timber)] += cleared[c] * P.clearing_timber;
                // accounts: timber that no woodman cut goes into the town's books as made
                crate::accounts::note_cleared(st, st.catch[c] as usize, timber, cleared[c] * P.clearing_timber);
            }
        }
    }
    for c in 0..N {
        if !(wood_share[c] > 0.02) {
            c_new[c] = if taken[c] { 0.0 } else { c_new[c] * 0.5 };
        }
    }
    st.cover = c_new;

    // dominant use for the map
    let mut land_use = vec![0u8; N];
    for c in 0..N {
        let mut dom = 0;
        for r in 1..=NS {
            if lam_new[(r, c)] > lam_new[(dom, c)] {
                dom = r;
            }
        }
        let mut u = if dom == NS { USE_WILD } else { USE_SECTOR0 + dom as u8 };
        if st.derelict_timer[c] > 0 {
            u = USE_DERELICT;
        }
        if st.urban[c] {
            u = USE_URBAN;
        }
        if ks[c] >= 0.25 && !st.urban[c] {
            u = USE_INDUSTRY;
        }
        if !(land[c] && st.catch[c] >= 0) {
            u = if land[c] { USE_WILD } else { USE_SEA };
        }
        land_use[c] = u;
    }
    st.land_use = land_use;
    st.lam = lam_new;

    // extractive units (cell, sector) used by the monthly loop, ordered by cell then sector
    let mut old_lprev = vec![f64::NAN; NS * N];
    if !st.ex_cell.is_empty() && st.ex_Lprev.len() == st.ex_cell.len() {
        for u in 0..st.ex_cell.len() {
            old_lprev[st.ex_sector[u] * N + st.ex_cell[u]] = st.ex_Lprev[u];
        }
    }
    let fields = sector_fields(st, C);
    let (mut ex_cell, mut ex_sector, mut ex_lam, mut ex_town, mut ex_field, mut ex_tau, mut carried) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for c in 0..N {
        for s in 0..NS {
            // mines: a working's fixed factor is its cell's deposit, less what it has gone through
            let l = crate::mines::unit_share(st, P, s, c, st.lam[(s, c)]);
            if l > 0.0 {
                ex_cell.push(c);
                ex_sector.push(s);
                ex_lam.push(l);
                ex_town.push(st.catch[c] as usize);
                ex_field.push(crate::mines::unit_field(st, P, s, c, fields[s][c]));
                ex_tau.push(P.freight * C.weight[C.sector_good[s]] * st.dcatch[c]);
                let prev = old_lprev[s * N + c];
                carried.push(if prev.is_nan() { 0.0 } else { prev });
            }
        }
    }
    st.ex_Lprev = carried;
    st.ex_cell = ex_cell;
    st.ex_sector = ex_sector;
    st.ex_lam = ex_lam;
    st.ex_town = ex_town;
    st.ex_field = ex_field;
    st.ex_tau = ex_tau;

    // rent on every cell, for firms deciding where to locate and for the map
    let mut rent = vec![0.0; N];
    for c in 0..N {
        let mut r = 0.0;
        for s in 0..NS {
            r += st.lam[(s, c)] * bids[(s, c)];
        }
        rent[c] = if st.urban[c] { st.cell_urban_rent[c] } else { r };
    }
    st.land_rent_only = rent.clone();
    crate::mines::add_royalties(st, P, &mut rent); // mines: the map's rent counts the royalties
    for f in 0..st.f_alive.len() {
        if st.f_alive[f] {
            rent[st.f_cell[f]] += st.f_rent[f];
        }
    }
    st.cell_rent = rent;
    st.bids = bids;
    crate::towns::price_ground(st, P, wbar); // towns: what ground costs a works, on town ground and where works hold the cell
}
