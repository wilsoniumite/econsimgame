//! Town ground that is kept (not in the Python; `docs/rules/towns.md`).
//!
//! Under the port a town has no ground of its own. Every year `land::update_urban`
//! works out how many cells the town's people need at that year's density and picks
//! that many afresh, the cheapest to reach first. The density is made to rise about
//! as fast as the people, so a town of 1900 stands on as many cells as it did in
//! 1750, and because the count steps up and down with every wobble in the works'
//! hands, cells at the edge are town one year and farm the next.
//!
//! With `towns.enabled` a town keeps what it builds:
//!
//! - each town has a list of its cells in the order they joined, the centre first;
//! - a cell has room for `d_new` people the year it joins and fills in with age
//!   toward a ceiling (`d_old`, rising by `d_old_growth` a year), so the centre is
//!   the oldest ground and the densest;
//! - when the town's people outgrow its room it takes the best free cell next to
//!   its own ground, by the port's score (near by road, by water), so it grows in
//!   ribbons along roads and shores;
//! - works standing on a cell the town takes stay where they are, and the cell
//!   has less room for people (`works_room`);
//! - it gives up its newest cell, one a year, only after `decline_years` in a row
//!   with under `empty` of the most people it has had;
//! - the towns of the first year stand on old ground: their cells are dated
//!   `back_years` and more before the start, the centre oldest;
//! - the town's people are a smoothed count (`smooth`), taken from the year's
//!   mean of works' hands and not December's (`mean_hands`).
//!
//! Two repairs ride with it. A town founded at the end of a year is reckoned for
//! that year at its donor's mean wage and prices (`donor_wage`): the port leaves
//! its wage at nothing, and the land round it is then priced as if labour were
//! free. And a town cell never pays less than its ground would earn under the
//! plough (`rent_floor`).
//!
//! Nothing here moves money. The households' rent bill is still the sum over the
//! town's cells and is still paid as `econ::step_month` pays it; a works on town
//! ground bids for its yard against the housing it displaces. Switched off, every
//! function below returns at once and the state's tables stay empty.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::land::{USE_SEA, USE_SECTOR0};
use crate::mat::Mat;
use crate::params::Params;
use crate::state::State;
use crate::util::bool_f;

/// The numbers of kept town ground.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TownsParams {
    /// Towns keep their ground.
    pub enabled: bool,
    /// People a cell has room for in the year it joins a town.
    pub d_new: f64,
    /// People an old cell has room for in the first year: the ceiling a cell fills in toward...
    pub d_old: f64,
    /// ...and how much the ceiling rises each year after.
    pub d_old_growth: f64,
    /// Years a cell takes to fill in: after this many it is 63 per cent of the way
    /// from `d_new` to the ceiling.
    pub tau: f64,
    /// A cell wholly under works has room for this share of the people it would
    /// otherwise hold (and in proportion between). 1: works take no room from people.
    pub works_room: f64,
    /// The town's people are smoothed: this share of last year's count, the rest this year's.
    pub smooth: f64,
    /// A town gives up its newest cell after `decline_years` in a row with fewer
    /// people than this share of the most it has had...
    pub empty: f64,
    /// ...one cell a year for as long as it stays so.
    pub decline_years: i64,
    /// The newest ground of a first town was built this many years before the start...
    pub back_years: i64,
    /// ...and its centre this many years before that, the cells between in order.
    pub back_span: i64,
    /// The year's mean of works' hands, not December's, says how many of a town's
    /// people are town people.
    pub mean_hands: bool,
    /// A town founded at the year's end is reckoned for that year at its donor's
    /// mean wage and prices (the port leaves the wage at nothing).
    pub donor_wage: bool,
    /// Ground is never priced under what it would earn without them: a town cell
    /// pays at least its rent under the plough, and on a cell the works have
    /// taken a works bids against what the cell would earn as open land.
    pub rent_floor: bool,
}

impl Default for TownsParams {
    /// Off: what the port runs with. The numbers are the game's, so that
    /// `--set towns.enabled=true` alone measures the mechanism as it is played.
    fn default() -> TownsParams {
        TownsParams {
            enabled: false,
            d_new: 300.0,
            // 530 puts the first towns on as many cells as the port gives them
            // in 1750; 5 a year is for the game as it is played, where the
            // town people grow ninefold by 1900 and not fivefold (towns.md)
            d_old: 530.0,
            d_old_growth: 5.0,
            tau: 30.0,
            works_room: 0.5,
            smooth: 0.5,
            empty: 0.6,
            decline_years: 15,
            back_years: 60,
            back_span: 40,
            mean_hands: true,
            donor_wage: true,
            rent_floor: true,
        }
    }
}

impl TownsParams {
    /// The game's towns.
    pub fn game() -> TownsParams {
        TownsParams { enabled: true, ..TownsParams::default() }
    }
}

/// Whether this state keeps town ground (its tables are sized on the first yearly update).
pub fn kept(st: &State) -> bool {
    st.N > 0 && st.town_built.len() == st.N
}

/// The people an old cell has room for in `year`: the ceiling young cells fill in toward.
pub fn ceiling(C: &Content, P: &Params, year: i64) -> f64 {
    P.towns.d_old + P.towns.d_old_growth * (year - C.start_year).max(0) as f64
}

/// The people a clear cell built in `built` has room for in `year`.
pub fn room(C: &Content, P: &Params, year: i64, built: i64) -> f64 {
    let tp = &P.towns;
    let age = (year - built).max(0) as f64;
    tp.d_new + (ceiling(C, P, year) - tp.d_new).max(0.0) * (1.0 - (-age / tp.tau.max(1.0)).exp())
}

/// The share of a cell's room that is left to people when works hold `ks` of it.
pub fn room_beside_works(P: &Params, ks: f64) -> f64 {
    1.0 - (1.0 - P.towns.works_room.clamp(0.0, 1.0)) * ks.clamp(0.0, 1.0)
}

/// What a whole clear cell would pay if it were shared out among `uses` (rows of
/// `bids`) in proportion to bid^(1/theta), as `land::update_land_use` shares it,
/// the wild share paying nothing.
fn shared_rent(P: &Params, bids: &Mat, wild_bid: f64, c: usize, uses: &[usize]) -> f64 {
    let log = |b: f64| if b > 0.0 { b.max(1e-12).ln() / P.land_theta } else { f64::NEG_INFINITY };
    let mut mx = log(wild_bid);
    for &s in uses {
        mx = mx.max(log(bids[(s, c)]));
    }
    if !mx.is_finite() {
        return 0.0;
    }
    let mut zsum = (log(wild_bid) - mx).exp();
    let mut rent = 0.0;
    for &s in uses {
        let z = (log(bids[(s, c)]) - mx).exp();
        zsum += z;
        rent += z * bids[(s, c)];
    }
    rent / zsum.max(1e-12)
}

/// What a whole clear cell would pay as open land, every use bidding for it,
/// pits and quarries too: the rent of the open cell beside it.
pub fn open_rent(P: &Params, bids: &Mat, wild_bid: f64, c: usize) -> f64 {
    let all: Vec<usize> = (0..bids.rows).collect();
    shared_rent(P, bids, wild_bid, c, &all)
}

/// The land uses of the plough: farm, pasture and woodland.
pub fn plough_uses(C: &Content) -> Vec<usize> {
    (0..C.NS).filter(|&s| matches!(C.sector_name[s].as_str(), "farm" | "pasture" | "forestry")).collect()
}

/// What a whole clear cell would pay under the plough (`plough_uses`), whatever
/// lies beneath it: the floor under a town cell's rent. The coal or clay under a
/// town is not what its houses displaced, and a pit's bid swings with the price
/// of what it digs; held to that, a town on a coalfield paid half its wages in
/// rent in a dear year, and a hamlet founded on one several times its wages.
pub fn plough_rent(P: &Params, bids: &Mat, wild_bid: f64, c: usize, plough: &[usize]) -> f64 {
    shared_rent(P, bids, wild_bid, c, plough)
}

// ---------------------------------------------------------------- the month

/// `econ::step_month`: works' hands this month are added up by town, for the
/// year's mean.
pub fn month(st: &mut State, P: &Params) {
    if !(P.towns.enabled && P.towns.mean_hands) {
        return;
    }
    if st.town_hands_acc.len() != st.T_max {
        st.town_hands_acc = vec![0.0; st.T_max];
    }
    for f in 0..st.F_count {
        if st.f_alive[f] {
            st.town_hands_acc[st.f_town[f]] += st.f_labor[f];
        }
    }
}

// ---------------------------------------------------------------- the year

/// `sim::yearly`, after December's hands are counted: the count that says how
/// many of a town's people are town people becomes the mean of the year's twelve
/// months. December alone moves a town's count by 6 per cent a year up or down
/// (`dx-towns.md`), which is most of what made town ground come and go.
pub fn year_hands(st: &mut State, P: &Params) {
    if !(P.towns.enabled && P.towns.mean_hands) || st.town_hands_acc.len() != st.T_max {
        return;
    }
    st.ind_labor = st.town_hands_acc.iter().map(|v| v / 12.0).collect();
    st.town_hands_acc.iter_mut().for_each(|v| *v = 0.0);
}

/// `sim::yearly`, once the year's towns are founded and before the market areas
/// are worked out again: a town founded this year has no
/// year behind it, so its mean wage is nothing and its mean prices are the first
/// year's list. Left so, `land::sector_bids` prices the land round it for a year
/// as if labour were free (rents of 1e9 and more) and entry reckons its works
/// with unpaid hands. It is given its donor's year instead: the town it was
/// carved out of, whose prices and wage it starts with.
pub fn new_town_year(st: &mut State, P: &Params, founded: &[usize], pbar: &mut Mat, wbar: &mut [f64]) {
    if !(P.towns.enabled && P.towns.donor_wage) {
        return;
    }
    for &n in founded {
        // the catchments are still last year's here, as they were when found_towns named the donor
        let donor = st.catch[st.town_node[n]];
        if donor < 0 || donor as usize == n {
            continue;
        }
        let d = donor as usize;
        wbar[n] = wbar[d];
        st.wbar[n] = wbar[d];
        for g in 0..pbar.cols {
            pbar[(n, g)] = pbar[(d, g)];
            st.pbar[(n, g)] = pbar[(d, g)];
        }
    }
}

/// The yearly update of town ground, in place of `land::update_urban`.
/// `initial` is the first allocation, before the warm-up.
pub fn update_urban(st: &mut State, C: &Content, P: &Params, year: i64, wbar: &[f64], bids: &Mat, initial: bool) {
    let (N, T) = (st.N, st.T_max);
    let (W, H) = (st.world.W, st.world.H);
    let tp = &P.towns;
    if !kept(st) {
        st.town_built = vec![0; N];
        st.town_room = vec![0.0; N];
        st.town_ground = vec![Vec::new(); T];
        st.town_low_years = vec![0; T];
        st.town_peak = vec![0.0; T];
        st.town_ground_rent = vec![0.0; N];
    }
    st.urban_rent = vec![0.0; T];
    st.cell_urban_rent = vec![0.0; N];
    st.town_ground_rent.iter_mut().for_each(|v| *v = 0.0);
    // before the first recorded year the towns settle: they fit their people, and
    // the first towns' ground is old ground
    let settling = initial || year < C.start_year;
    // ground no town takes: the sea, and iron and coal mines (as in the port)
    let blocked: Vec<bool> = (0..N)
        .map(|c| {
            let u = st.land_use[c];
            !st.world.land[c] || u == USE_SEA || (u >= USE_SECTOR0 + 5 && u <= USE_SECTOR0 + 6)
        })
        .collect();
    let ks = st.cell_K_share.clone();
    let plough = plough_uses(C);

    // a cell whose market town changed goes with its new town and keeps its
    // year; one left with no market is given up
    let mut owned = vec![false; N];
    let mut moved: Vec<(usize, usize)> = Vec::new();
    for i in 0..T {
        let cells = std::mem::take(&mut st.town_ground[i]);
        let mut keep = Vec::with_capacity(cells.len());
        for c in cells {
            let j = st.catch[c];
            if j == i as i64 && st.town_active[i] {
                keep.push(c);
                owned[c] = true;
            } else if j >= 0 && st.town_active[j as usize] {
                moved.push((j as usize, c));
                owned[c] = true;
            } else {
                st.town_built[c] = 0;
            }
        }
        st.town_ground[i] = keep;
    }
    for (j, c) in moved {
        st.town_ground[j].push(c);
    }

    let neighbours = |c: usize| {
        let (y, x) = ((c / W) as i64, (c % W) as i64);
        (-1i64..=1)
            .flat_map(move |dy| (-1i64..=1).map(move |dx| (y + dy, x + dx)))
            .filter(move |&(ny, nx)| ny >= 0 && nx >= 0 && (ny as usize) < H && (nx as usize) < W && !(ny == y && nx == x))
            .map(move |(ny, nx)| ny as usize * W + nx as usize)
    };
    let mut seen = vec![false; N];
    for i in 0..T {
        if !st.town_active[i] {
            continue;
        }
        // the port's count of town people, smoothed
        let share = 0.04 + 0.8 * st.ind_labor[i] / (P.participation * st.pop[i]).max(1.0);
        let raw = st.pop[i] * share.clamp(0.06, 0.95);
        let U = if st.urban_pop[i] > 0.0 { tp.smooth * st.urban_pop[i] + (1.0 - tp.smooth) * raw } else { raw };
        let node = st.town_node[i];
        let old = settling && st.town_founded[i] < C.start_year;
        // a cell taken now is new ground, or old ground while the first towns settle
        let built_now = if old { C.start_year - tp.back_years } else { year };
        let mut list = std::mem::take(&mut st.town_ground[i]);
        // the centre is the town's first cell
        match list.iter().position(|&c| c == node) {
            Some(k) => {
                let c = list.remove(k);
                list.insert(0, c);
            }
            None => {
                list.insert(0, node);
                owned[node] = true;
                st.town_built[node] = built_now;
            }
        }
        let room_of = |built: &[i64], c: usize| room(C, P, year, built[c]) * room_beside_works(P, ks[c]);
        let mut total: f64 = list.iter().map(|&c| room_of(&st.town_built, c)).sum();
        // the free cells next to the town's ground, within its market area. A pit
        // is grown round: the ground beyond it counts as next to the town.
        seen.iter_mut().for_each(|v| *v = false);
        for &c in &list {
            seen[c] = true;
        }
        let mut stack: Vec<usize> = list.clone();
        let mut frontier: Vec<usize> = Vec::new();
        let score = |c: usize| -st.dist[(i, c)] + 0.8 * st.world.jitter[c] + 0.4 * bool_f(st.world.coast[c] || st.world.river[c]);
        loop {
            while let Some(c) = stack.pop() {
                for nc in neighbours(c) {
                    if seen[nc] || st.catch[nc] != i as i64 {
                        continue;
                    }
                    seen[nc] = true;
                    if owned[nc] {
                        continue;
                    }
                    if !blocked[nc] {
                        frontier.push(nc);
                    } else if st.world.land[nc] {
                        stack.push(nc);
                    }
                }
            }
            if U <= total || frontier.is_empty() {
                break;
            }
            // the town's people have outgrown its room: it takes its best free neighbour
            let mut best = 0;
            for k in 1..frontier.len() {
                if score(frontier[k]) > score(frontier[best]) {
                    best = k;
                }
            }
            let c = frontier.swap_remove(best);
            st.town_built[c] = built_now;
            owned[c] = true;
            list.push(c);
            stack.push(c);
            total += room_of(&st.town_built, c);
        }
        if settling {
            // not yet kept: ground the town's people do not need is let go
            while list.len() > 1 {
                let c = *list.last().unwrap();
                let r = room_of(&st.town_built, c);
                if total - r < U {
                    break;
                }
                list.pop();
                total -= r;
                st.town_built[c] = 0;
                owned[c] = false;
            }
            st.town_low_years[i] = 0;
            st.town_peak[i] = U;
        } else {
            // a long, deep decline gives up the newest cell, one a year. The
            // decline is in people, against the most the town has had: against
            // its room, a town that only stood still would be found wanting as
            // its old ground went on filling in.
            st.town_peak[i] = st.town_peak[i].max(U);
            if U < tp.empty * st.town_peak[i] {
                st.town_low_years[i] += 1;
            } else {
                st.town_low_years[i] = 0;
            }
            if st.town_low_years[i] >= tp.decline_years && list.len() > 1 {
                // the ground that is left held that much less of the town at its fullest
                st.town_peak[i] *= (list.len() - 1) as f64 / list.len() as f64;
                let c = list.pop().unwrap();
                st.town_built[c] = 0;
                owned[c] = false;
            }
        }
        if old {
            // the first towns' ground is dated from the centre out
            let n = list.len();
            for (k, &c) in list.iter().enumerate() {
                let inward = if n > 1 { 1.0 - k as f64 / (n - 1) as f64 } else { 1.0 };
                st.town_built[c] = C.start_year - tp.back_years - (tp.back_span as f64 * inward).round() as i64;
            }
        }
        // people are shared out by room; rent as the port's rule has it, highest at
        // the centre and falling to nothing a cell past the edge
        let clear: Vec<f64> = list.iter().map(|&c| room(C, P, year, st.town_built[c])).collect();
        let rooms: Vec<f64> = list.iter().zip(&clear).map(|(&c, &r)| r * room_beside_works(P, ks[c])).collect();
        let per_room = U / rooms.iter().sum::<f64>().max(1.0);
        let dmax = list.iter().map(|&c| st.dist[(i, c)]).fold(0.0, f64::max);
        let fringe = dmax + 1.0;
        let wild_bid = P.land_fixed_cost * wbar[i].max(1e-6);
        let mut bill = 0.0;
        for (j, &c) in list.iter().enumerate() {
            let per_head = P.urban_rent_rate * wbar[i] * (1.0 - st.dist[(i, c)] / fringe).max(0.0);
            // what the cell would pay if houses stood on all of it
            let houses = clear[j] * per_room * per_head;
            let floor = if tp.rent_floor && wbar[i] > 0.0 { plough_rent(P, bids, wild_bid, c, &plough) } else { 0.0 };
            st.town_room[c] = rooms[j];
            // a works' yard displaces the houses or the plough, whichever pays more
            st.town_ground_rent[c] = houses.max(floor);
            // the residents pay for the part the works leave them. A full cell
            // never pays under the plough's rent; in a town that has emptied the
            // floor falls with the people, or the few who stayed would pay the
            // farm rent of ground nobody lives on (more than their wages, in a
            // town at a tenth of its room)
            st.cell_urban_rent[c] = houses.max(floor * per_room.min(1.0)) * rooms[j] / clear[j].max(1e-9);
            bill += st.cell_urban_rent[c];
        }
        st.urban_rent[i] = bill;
        st.urban_pop[i] = U;
        st.town_ground[i] = list;
    }

    let mut urban = vec![false; N];
    let mut res = vec![0.0; N];
    for i in 0..T {
        let list = &st.town_ground[i];
        if list.is_empty() {
            continue;
        }
        let per_room = st.urban_pop[i] / list.iter().map(|&c| st.town_room[c]).sum::<f64>().max(1.0);
        for &c in list {
            urban[c] = true;
            res[c] = st.town_room[c] * per_room;
        }
    }
    for c in 0..N {
        if !urban[c] {
            st.town_built[c] = 0;
            st.town_room[c] = 0.0;
        }
    }
    st.urban = urban;
    st.urban_res = res;
    st.urban_density = ceiling(C, P, year);
}

/// The end of `land::update_land_use`: what a cell's worth of ground costs a
/// works (`State::land_rent_only`, which a works' rent bid is reckoned from).
///
/// On town ground the port prices it at what the cell's residents pay, so a
/// cell half under works already is half price. Here it is what the whole cell
/// would pay under houses, the housing the works would displace.
///
/// Outside the towns, a cell the works have taken three fifths of is shared out
/// to no other use (`land::update_land_use`), so nothing is left there to set a
/// price and the rest of its ground comes at the port's floor of a hundredth.
/// With `rent_floor` it costs what the cell would earn as open land.
pub fn price_ground(st: &mut State, P: &Params, wbar: &[f64]) {
    if !P.towns.enabled || !kept(st) {
        return;
    }
    for c in 0..st.N {
        if st.urban[c] {
            st.land_rent_only[c] = st.town_ground_rent[c];
        } else if P.towns.rent_floor && st.cell_K_share[c] >= 0.6 && st.world.land[c] && st.catch[c] >= 0 {
            let w = wbar[st.catch[c] as usize];
            if w > 0.0 {
                st.land_rent_only[c] = open_rent(P, &st.bids, P.land_fixed_cost * w, c);
            }
        }
    }
}
