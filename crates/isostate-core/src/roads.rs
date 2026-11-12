//! Roads and bridges that somebody pays for. Not in the Python; a probe
//! (`docs/probes/dx-roads.md`), behind `roads.enabled`, which is off under both
//! sets of rules.
//!
//! Under the port a cell's road is worn by traffic: it becomes a track, a road
//! and a turnpike as the weight carted over it passes three thresholds, and
//! nothing goes into it, neither money nor hands nor stone (`sim::update_roads`).
//! A river costs a road nothing.
//!
//! With the switch on, a track is still worn by the carts, but the next two
//! steps are paid for:
//!
//! - **A made road is the parish's.** Where the traffic over a track passes the
//!   port's second threshold, the town whose market area holds it may make it a
//!   road. The parish pays out of a highway rate on its landlords (and on its
//!   households where the landlords are in debt), a year at a time and only as
//!   far as the year's rate goes, busiest cells first. The work is a project
//!   like a railway's: stone bought at the town's market and wages paid to the
//!   town's men, who are hands taken from its labour (`take_menders`).
//! - **A turnpike is a trust's.** Each year the town's turnpike trust (one a
//!   town, formed the first time it pays) looks at the roads and tracks in its
//!   market area with no railway, prices turnpiking them, and borrows from the
//!   town's landlords and households where the tolls on last year's traffic,
//!   less the road's upkeep, would pay its lenders `hurdle` a year. It builds
//!   with stone and hands, and when the work is done sets up its gates.
//! - **A toll is part of what carriage costs.** A cell of a trust's road costs
//!   `toll` more to pass in the cost graph (`tolls`), so the carting money paid
//!   for a load over it is higher by the toll; each month the trust takes the
//!   tolls on its cells' traffic from the households of the town whose carters
//!   took them (and out of that town's carting bill, since a toll is nobody's
//!   time), pays its road's upkeep (stone and hands, a standing project), and
//!   pays what is left to its lenders.
//! - **A bridge is costed by the width of what it crosses.** Where a road is
//!   first made over a river (`crossings`), the maker pays `bridge_stone` a metre
//!   of river on top: the parish for a made road, the trust for a turnpike.
//!
//! Every flow of money is between purses `econ::money_total` counts: the
//! projects' cash is counted there already, and the trusts keep none.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::params::Params;
use crate::state::{Event, Project, State};
use crate::transport as tr;
use crate::world::{NEIGH8, World};

/// A textbook river is about 4.4 times the square root of its flow wide, in
/// metres (`docs/rules/land.md` draws them two and a half times that).
pub const WIDTH_PER_ROOT_FLOW: f64 = 4.4;

/// The port's island keeps no flow, only the area upstream of each cell (`acc`,
/// in cells of 4 square km). Mean flow per cell of area, cubic metres a second:
/// the median of `discharge / acc` over the river cells of the second island
/// generator on islands 1 to 8 (`examples/probe_roads.rs --land`): 0.067 to
/// 0.074 by island, about 550 mm of runoff a year.
pub const PORT_FLOW_PER_CELL: f64 = 0.07;

/// `Project::serves` of a road works: the level its cells are raised to, or a
/// trust's standing repairs.
pub const MAKE_ROAD: u8 = 2;
pub const MAKE_TURNPIKE: u8 = 3;
pub const REPAIRS: u8 = 9;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RoadParams {
    /// Made roads and turnpikes are paid for; tracks are still worn for nothing.
    pub enabled: bool,
    /// What a cell of a trust's road costs to pass on top of the road itself, in
    /// the cost graph's units (a turnpike on lowland is 0.3).
    pub toll: f64,
    /// Stone to make a cell of track into a road, and a road into a turnpike, on
    /// lowland; hills and mountains take their carting cost's multiple (1.7, 3.2).
    pub road_stone: f64,
    pub turnpike_stone: f64,
    /// Stone a metre of river for a bridge, where a road is first made across one.
    pub bridge_stone: f64,
    /// Stone a year to keep a cell of turnpike, on lowland.
    pub upkeep_stone: f64,
    /// Wages paid on every shilling of stone: road making is mostly labour.
    pub wages: f64,
    /// Months a year's works take if the market fills every order.
    pub months: f64,
    /// A parish may lay out this share of its landlords' cash in a year on its
    /// roads (a quarter of it of its households' where the landlords are in debt).
    pub rate: f64,
    /// What a trust's lenders want a year on what they lend, from the tolls on
    /// last year's traffic less the road's upkeep.
    pub hurdle: f64,
    /// Of their cash, what a town's landlords and households will lend a trust
    /// in a year (the railways' subscribers put up 0.35 and 0.08).
    pub lend_lords: f64,
    pub lend_households: f64,
    /// The menders are hands: at most this share of a town's, moved this far of
    /// the way each month to what the works' wages pay for.
    pub mender_max_share: f64,
    pub mender_adjust: f64,
}

impl Default for RoadParams {
    /// Off: what the port runs with, and the game too while this is a probe.
    fn default() -> RoadParams {
        RoadParams {
            enabled: false,
            toll: 0.05,
            road_stone: 50.0,
            turnpike_stone: 110.0,
            bridge_stone: 8.0,
            upkeep_stone: 15.0,
            wages: 2.0,
            months: 12.0,
            rate: 0.02,
            hurdle: 0.05,
            lend_lords: 0.2,
            lend_households: 0.04,
            mender_max_share: 0.1,
            mender_adjust: 0.25,
        }
    }
}

impl RoadParams {
    /// The probe's roads, switched on (`--set roads.enabled=true` does the same).
    pub fn probe() -> RoadParams {
        RoadParams { enabled: true, ..RoadParams::default() }
    }
}

/// A town's turnpike trust.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Trust {
    pub town: usize,
    /// The year it first set up its gates.
    pub formed: i64,
    /// Who lent it what: (landlords or households, town, amount), merged.
    pub lenders: Vec<(bool, usize, f64)>,
    /// All that was laid out on its road, and all the tolls it has taken.
    pub capital: f64,
    pub tolls: f64,
    /// What it has paid its lenders, and spent on repairs.
    pub paid: f64,
    pub repairs: f64,
}

/// A year of the roads, for the probe's tables.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RoadYear {
    pub year: i64,
    /// Laid out on making roads and turnpikes and on repairs, money.
    pub spent: f64,
    /// Tolls taken, and of them paid to lenders.
    pub tolls: f64,
    pub dividends: f64,
    /// Raised by the parishes' rates, and lent to trusts.
    pub rates: f64,
    pub lent: f64,
    /// Trusts with gates up, cells under them, and the year's mean of menders (hands).
    pub trusts: usize,
    pub trust_cells: usize,
    pub menders: f64,
    /// Cells made into roads and turnpikes this year, and bridges built with them.
    pub made: usize,
    pub turnpiked: usize,
    pub bridges: usize,
    pub bridge_m: f64,
}

/// Everything the roads keep. Empty while `roads.enabled` is off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RoadState {
    /// The town whose trust takes the tolls on each cell, or -1.
    pub cell_trust: Vec<i32>,
    /// A bridge stands in the cell already (built with an earlier road).
    pub bridged: Vec<bool>,
    /// By town.
    pub trusts: Vec<Option<Trust>>,
    /// Menders at work this month, by town.
    pub menders: Vec<f64>,
    /// The year now running, and every year closed.
    pub now: RoadYear,
    pub years: Vec<RoadYear>,
}

/// Width in metres of the river through cell `c`, or 0 where there is none.
pub fn river_width(world: &World, c: usize) -> f64 {
    if !world.river[c] {
        return 0.0;
    }
    let flow = match &world.landform {
        Some(lf) => lf.discharge[c] as f64,
        None => PORT_FLOW_PER_CELL * world.acc[c],
    };
    WIDTH_PER_ROOT_FLOW * flow.max(0.0).sqrt()
}

fn at(W: usize, c: usize) -> (f64, f64) {
    ((c % W) as f64, (c / W) as f64)
}

/// Where the roads of `level` or better cross a river, as (the cell that would
/// carry the bridge, the river's width there in metres).
///
/// A river runs from cell centre to cell centre down `World::down`. A road is
/// every two neighbouring cells with road joined, as the viewer draws it. A
/// road crosses a river in two ways: through a river cell, with road on both
/// sides of the river's line there (the line from the main stream coming in to
/// the cell it runs on to); or diagonally between two cells of road, where the
/// other diagonal of the same square is a reach of river (charged to the upper
/// of the two road cells).
pub fn crossings(world: &World, road: &[u8], level: u8) -> Vec<(usize, f64)> {
    let (W, H) = (world.W, world.H);
    let N = W * H;
    let on = |c: usize| world.land[c] && road[c] >= level;
    // the main stream into each river cell: the upstream neighbour with most area
    let mut up = vec![usize::MAX; N];
    for c in 0..N {
        let d = world.down[c];
        if d >= 0 && world.river[c] {
            let d = d as usize;
            if up[d] == usize::MAX || world.acc[c] > world.acc[up[d]] {
                up[d] = c;
            }
        }
    }
    let mut out = Vec::new();
    for b in 0..N {
        if !on(b) || !world.river[b] {
            continue;
        }
        let d = world.down[b];
        let (from, to) = (if up[b] != usize::MAX { up[b] } else { b }, if d >= 0 { d as usize } else { b });
        if from == to {
            continue;
        }
        let (fx, fy) = at(W, from);
        let (tx, ty) = at(W, to);
        let (bx, by) = at(W, b);
        let (dx, dy) = (tx - fx, ty - fy);
        let (mut left, mut right) = (false, false);
        for k in 0..8 {
            let Some(n) = tr::neighbour(H, W, b, k) else { continue };
            if n == from || n == to || !on(n) {
                continue;
            }
            let (nx, ny) = at(W, n);
            let side = dx * (ny - by) - dy * (nx - bx);
            if side > 0.0 {
                left = true;
            } else if side < 0.0 {
                right = true;
            }
        }
        if left && right {
            out.push((b, river_width(world, b)));
        }
    }
    // diagonal steps of road across a diagonal reach of river
    for a in 0..N {
        if !on(a) {
            continue;
        }
        let (y, x) = (a / W, a % W);
        for (k, &(dy, dx)) in NEIGH8.iter().enumerate() {
            if dy != 1 || dx == 0 {
                continue; // each square once: the two diagonals going down the map
            }
            let Some(b) = tr::neighbour(H, W, a, k) else { continue };
            if !on(b) {
                continue;
            }
            let c1 = y * W + (x as isize + dx) as usize; // the other two corners
            let c2 = (y + 1) * W + x;
            let reach = |p: usize, q: usize| world.river[p] && world.down[p] == q as i64;
            if reach(c1, c2) || reach(c2, c1) {
                out.push((a, river_width(world, c1).max(river_width(world, c2))));
            }
        }
    }
    out
}

fn terrain(world: &World, c: usize) -> f64 {
    tr::TERRAIN_COST[world.terrain[c] as usize].max(1.0)
}

/// The widest river each cell's road would bridge (0 for none), for roads as they stand.
fn crossing_widths(st: &State) -> Vec<f64> {
    let mut w = vec![0.0; st.N];
    for (c, width) in crossings(&st.world, &st.road, 1) {
        w[c] = f64::max(w[c], width);
    }
    w
}

#[allow(clippy::too_many_arguments)]
fn works(town: usize, serves: u8, cells: Vec<usize>, stone: f64, gs: usize, G: usize, cash: f64, subs: Vec<(bool, usize, f64)>, year: i64, R: &RoadParams) -> Project {
    let mut need = vec![0.0; G];
    need[gs] = stone;
    Project {
        a: town,
        b: town,
        target: cells.first().copied().unwrap_or(0),
        town,
        length_km: 2.0 * cells.len() as f64,
        cells: cells.clone(),
        new: cells,
        order: need.clone(),
        need,
        cash,
        start: year,
        cost: cash,
        way: crate::ways::ROAD,
        months: R.months,
        serves,
        paid_in: cash,
        subs,
        wages: R.wages,
        ..Project::default()
    }
}

/// The first year after the warm-up: the roads of 1750 are inherited. Their
/// bridges stand, and every turnpike has a trust whose money was lent long
/// ago by its town's landlords (a claim on the tolls; no money moves now).
pub fn begin(st: &mut State, C: &Content, P: &Params, year: i64) {
    if !P.roads.enabled || !st.rd.cell_trust.is_empty() {
        return;
    }
    let R = &P.roads;
    let (N, T) = (st.N, st.T_max);
    st.rd.cell_trust = vec![-1; N];
    st.rd.trusts = vec![None; T];
    let w = crossing_widths(st);
    st.rd.bridged = (0..N).map(|c| st.road[c] >= 2 && w[c] > 0.0).collect();
    let gs = C.gi("stone");
    for c in 0..N {
        // (a navigable river's traffic is boats', which pay no toll at a road's gate)
        if !st.world.land[c] || st.road[c] < 3 || st.rail[c] > 0 || st.world.navigable[c] || st.catch[c] < 0 {
            continue;
        }
        let t = st.catch[c] as usize;
        let cost = (R.road_stone + R.turnpike_stone) * terrain(&st.world, c) * st.price[(t, gs)] * (1.0 + R.wages);
        st.rd.cell_trust[c] = t as i32;
        let trust = st.rd.trusts[t].get_or_insert_with(|| Trust { town: t, formed: year, ..Trust::default() });
        trust.capital += cost;
        match trust.lenders.first_mut() {
            Some(l) => l.2 += cost,
            None => trust.lenders.push((true, t, cost)),
        }
    }
    st.rd.now.year = year;
}

/// The year's road works, after `sim::update_roads` has measured the year's
/// traffic: the year's record is closed, cells that fell back leave their
/// trust, each parish makes what its rate will pay for, and each trust borrows
/// for whatever the tolls would repay.
pub fn yearly(st: &mut State, C: &Content, P: &Params, year: i64, warm: bool) {
    if !P.roads.enabled || warm || st.rd.cell_trust.is_empty() {
        return;
    }
    let R = &P.roads;
    let (N, T, G) = (st.N, st.T_max, C.G);
    let gs = C.gi("stone");
    for c in 0..N {
        if st.rd.cell_trust[c] >= 0 && (st.road[c] < 3 || st.rail[c] > 0) {
            st.rd.cell_trust[c] = -1;
        }
    }
    let mut now = std::mem::take(&mut st.rd.now);
    now.year = year;
    now.trusts = st.rd.trusts.iter().filter(|t| t.is_some()).count();
    now.trust_cells = st.rd.cell_trust.iter().filter(|&&t| t >= 0).count();
    now.menders /= 12.0;
    st.rd.years.push(now);
    st.rd.now = RoadYear { year: year + 1, ..RoadYear::default() };

    let mut building = vec![false; N];
    let mut busy = vec![[false; 2]; T];
    for pr in st.projects.iter().filter(|pr| pr.way == crate::ways::ROAD) {
        for &c in &pr.new {
            building[c] = true;
        }
        match pr.serves {
            MAKE_ROAD => busy[pr.town][0] = true,
            MAKE_TURNPIKE => busy[pr.town][1] = true,
            _ => {}
        }
    }
    let width = crossing_widths(st);
    let mut by_town: Vec<Vec<usize>> = vec![Vec::new(); T];
    for c in 0..N {
        if st.world.land[c] && st.catch[c] >= 0 {
            by_town[st.catch[c] as usize].push(c);
        }
    }
    let t2 = P.road_thresholds[1];
    let mut new_works: Vec<Project> = Vec::new();
    for t in 0..T {
        if !st.town_active[t] {
            continue;
        }
        let p = st.pbar[(t, gs)] * (1.0 + R.wages);
        if !(p > 0.0) {
            continue;
        }
        // the parish: tracks busy enough for a made road, busiest first, as far as the rate goes
        if !busy[t][0] {
            let mut cand: Vec<usize> =
                by_town[t].iter().copied().filter(|&c| st.road[c] == 1 && st.rail[c] == 0 && !building[c] && st.traffic[c] >= t2).collect();
            cand.sort_by(|&a, &b| st.traffic[b].total_cmp(&st.traffic[a]).then(a.cmp(&b)));
            let lords = st.lord_cash[t] > 0.0;
            let budget = if lords { R.rate * st.lord_cash[t] } else { 0.25 * R.rate * st.hh_cash[t].max(0.0) };
            let (mut stone, mut cells) = (0.0, Vec::new());
            for c in cand {
                let s = R.road_stone * terrain(&st.world, c) + if st.rd.bridged[c] { 0.0 } else { R.bridge_stone * width[c] };
                if (stone + s) * p <= budget {
                    stone += s;
                    cells.push(c);
                }
            }
            if !cells.is_empty() {
                let cash = stone * p;
                if lords {
                    st.lord_cash[t] -= cash;
                } else {
                    st.hh_cash[t] -= cash;
                }
                st.rd.now.rates += cash;
                for &c in &cells {
                    building[c] = true;
                }
                new_works.push(works(t, MAKE_ROAD, cells, stone, gs, G, cash, vec![(lords, t, cash)], year, R));
            }
        }
        // the trust: roads and tracks whose tolls, less upkeep, would pay its lenders
        if !busy[t][1] {
            let mut cand: Vec<(usize, f64, f64)> = Vec::new();
            for &c in &by_town[t] {
                if st.road[c] < 1 || st.road[c] >= 3 || st.rail[c] > 0 || st.world.navigable[c] || building[c] {
                    continue;
                }
                let k = terrain(&st.world, c);
                let mut s = R.turnpike_stone * k + if st.road[c] < 2 { R.road_stone * k } else { 0.0 };
                if !st.rd.bridged[c] {
                    s += R.bridge_stone * width[c];
                }
                let net = R.toll * P.freight * st.traffic[c] - R.upkeep_stone * k * p;
                if net >= R.hurdle * s * p {
                    cand.push((c, s, net / (s * p)));
                }
            }
            if !cand.is_empty() {
                cand.sort_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
                let lords_can = R.lend_lords * st.lord_cash[t].max(0.0);
                let hh_can = R.lend_households * st.hh_cash[t].max(0.0);
                let (mut stone, mut cells) = (0.0, Vec::new());
                for (c, s, _) in cand {
                    if (stone + s) * p <= lords_can + hh_can {
                        stone += s;
                        cells.push(c);
                    }
                }
                if !cells.is_empty() {
                    let cash = stone * p;
                    let from_lords = cash.min(lords_can);
                    let from_hh = cash - from_lords;
                    st.lord_cash[t] -= from_lords;
                    st.hh_cash[t] -= from_hh;
                    st.rd.now.lent += cash;
                    let subs: Vec<(bool, usize, f64)> = [(true, t, from_lords), (false, t, from_hh)].into_iter().filter(|s| s.2 > 0.0).collect();
                    new_works.push(works(t, MAKE_TURNPIKE, cells, stone, gs, G, cash, subs, year, R));
                }
            }
        }
    }
    // each trust's repairs for the coming year, paid out of its tolls as they come in
    let mut upkeep = vec![0.0; T];
    for c in 0..N {
        if st.rd.cell_trust[c] >= 0 {
            upkeep[st.rd.cell_trust[c] as usize] += R.upkeep_stone * terrain(&st.world, c);
        }
    }
    for t in 0..T {
        if st.rd.trusts[t].is_none() {
            continue;
        }
        match st.projects.iter_mut().find(|pr| pr.way == crate::ways::ROAD && pr.serves == REPAIRS && pr.town == t) {
            Some(pr) => {
                pr.need[gs] = upkeep[t];
                pr.order[gs] = upkeep[t];
            }
            None => new_works.push(works(t, REPAIRS, Vec::new(), upkeep[t], gs, G, 0.0, Vec::new(), year, R)),
        }
    }
    st.projects.extend(new_works);
}

/// Merge (landlords or households, town, amount) into a list of lenders.
fn lend(list: &mut Vec<(bool, usize, f64)>, l: (bool, usize, f64)) {
    match list.iter_mut().find(|x| x.0 == l.0 && x.1 == l.1) {
        Some(x) => x.2 += l.2,
        None => list.push(l),
    }
}

/// The month's end for the roads (after `econ::step_month`): tolls taken and
/// shared out, works short of cash call on their town, and finished works open.
pub fn month(st: &mut State, C: &Content, P: &Params, year_float: f64, events: &mut Vec<Event>) {
    if !P.roads.enabled || st.rd.cell_trust.is_empty() {
        return;
    }
    let R = &P.roads;
    let (N, T) = (st.N, st.T_max);
    let gs = C.gi("stone");
    for pr in st.projects.iter().filter(|pr| pr.way == crate::ways::ROAD) {
        if pr.wages > 0.0 {
            st.rd.now.spent += pr.navvy_pay * (1.0 + 1.0 / pr.wages);
        }
    }
    st.rd.now.menders += st.rd.menders.iter().sum::<f64>();

    // tolls on last year's traffic, a twelfth a month, from the carters' town
    let mut take = vec![0.0; T];
    for c in 0..N {
        let owner = st.rd.cell_trust[c];
        if owner < 0 || st.catch[c] < 0 {
            continue;
        }
        let payer = st.catch[c] as usize;
        let got = (R.toll * P.freight * st.traffic[c] / 12.0).min(st.hh_cash[payer].max(0.0));
        if got <= 0.0 {
            continue;
        }
        st.hh_cash[payer] -= got;
        // a toll is nobody's time: it is not carters' wages
        if P.carting.enabled && st.cart_bill.len() == T {
            st.cart_bill[payer] = (st.cart_bill[payer] - got).max(0.0);
        }
        take[owner as usize] += got;
    }
    for t in 0..T {
        if take[t] <= 0.0 {
            continue;
        }
        st.rd.now.tolls += take[t];
        let mut left = take[t];
        if let Some(pr) = st.projects.iter_mut().find(|pr| pr.way == crate::ways::ROAD && pr.serves == REPAIRS && pr.town == t) {
            let owed = pr.order[gs] * st.price[(t, gs)] * (1.0 + pr.wages) - pr.cash;
            let put = owed.max(0.0).min(left);
            pr.cash += put;
            pr.paid_in += put;
            left -= put;
        }
        let Some(trust) = st.rd.trusts[t].as_mut() else {
            st.hh_cash[t] += left;
            continue;
        };
        trust.tolls += take[t];
        trust.repairs += take[t] - left;
        trust.paid += left;
        let lenders = trust.lenders.clone();
        if lenders.iter().map(|l| l.2).sum::<f64>() > 0.0 {
            crate::ways::repay(st, &lenders, left);
        } else {
            st.hh_cash[t] += left;
        }
        st.rd.now.dividends += left;
    }

    // works short of cash call on their town; finished works open
    let projects = std::mem::take(&mut st.projects);
    let mut keep = Vec::with_capacity(projects.len());
    let mut width: Option<Vec<f64>> = None;
    for mut pr in projects {
        if pr.way != crate::ways::ROAD || pr.serves == REPAIRS {
            keep.push(pr);
            continue;
        }
        let t = pr.town;
        let rq = pr.order[gs].min(pr.need[gs] / pr.months.max(1.0));
        let short = rq * st.price[(t, gs)] * (1.0 + pr.wages) - pr.cash;
        if short > 0.0 {
            let (lords, frac) = if st.lord_cash[t] > 0.0 { (true, 0.1) } else { (false, 0.025) };
            let cash = if lords { &mut st.lord_cash } else { &mut st.hh_cash };
            let call = (cash[t].max(0.0) * frac).min(short);
            if call > 0.0 {
                cash[t] -= call;
                pr.cash += call;
                pr.paid_in += call;
                lend(&mut pr.subs, (lords, t, call));
                if pr.serves == MAKE_ROAD {
                    st.rd.now.rates += call;
                } else {
                    st.rd.now.lent += call;
                }
            }
        }
        if pr.order.iter().sum::<f64>() > 0.01 * pr.need.iter().sum::<f64>() {
            keep.push(pr);
            continue;
        }
        let level = pr.serves;
        let w = width.get_or_insert_with(|| crossing_widths(st));
        for &c in &pr.new {
            st.road[c] = st.road[c].max(level);
            if w[c] > 0.0 && !st.rd.bridged[c] {
                st.rd.bridged[c] = true;
                st.rd.now.bridges += 1;
                st.rd.now.bridge_m += w[c];
            }
            if level == MAKE_TURNPIKE && st.rail[c] == 0 && !st.world.navigable[c] {
                st.rd.cell_trust[c] = t as i32;
            }
        }
        if level == MAKE_ROAD {
            st.rd.now.made += pr.new.len();
        } else {
            st.rd.now.turnpiked += pr.new.len();
        }
        let spent = pr.paid_in - pr.cash;
        let subs = std::mem::take(&mut pr.subs);
        crate::ways::repay(st, &subs, pr.cash);
        pr.cash = 0.0;
        if level == MAKE_TURNPIKE {
            let first = st.rd.trusts[t].is_none();
            let trust = st.rd.trusts[t].get_or_insert_with(|| Trust { town: t, formed: year_float.floor() as i64, ..Trust::default() });
            trust.capital += spent;
            for s in subs {
                lend(&mut trust.lenders, s);
            }
            if first {
                let mut ev = Event::new(
                    year_float,
                    "turnpike",
                    0.3,
                    format!("The {} turnpike trust sets up its gates", st.town_name[t]),
                    format!(
                        "{:.0} km of the roads round {} are turnpiked with money lent by its landlords and households. Carriers now pay a toll at the gates, and the tolls keep the road and pay the lenders.",
                        pr.length_km, st.town_name[t]
                    ),
                );
                ev.town = Some(t);
                ev.cell = Some(pr.target);
                events.push(ev);
            }
        }
    }
    st.projects = keep;
}

/// The menders and road-makers are hands: last month's wages on the road works
/// of each town, at its wage, are that many people not there to be hired by
/// farms and works this month, moved part of the way each month as the canals'
/// navvies are.
pub fn take_menders(st: &mut State, P: &Params, supply: &mut [f64]) {
    if !P.roads.enabled || st.rd.cell_trust.is_empty() {
        return;
    }
    let T = supply.len();
    let mut want = vec![0.0; T];
    for pr in st.projects.iter().filter(|pr| pr.way == crate::ways::ROAD) {
        if st.wage[pr.town] > 0.0 {
            want[pr.town] += pr.navvy_pay / st.wage[pr.town];
        }
    }
    let k = P.roads.mender_adjust.clamp(0.0, 1.0);
    let was = if st.rd.menders.len() == T { std::mem::take(&mut st.rd.menders) } else { vec![0.0; T] };
    let mut hands = vec![0.0; T];
    for t in 0..T {
        let most = (P.roads.mender_max_share * supply[t]).max(0.0);
        hands[t] = (was[t] + k * (want[t].min(most) - was[t])).clamp(0.0, most);
        if hands[t] < 0.01 {
            hands[t] = 0.0;
        }
        supply[t] -= hands[t];
    }
    st.rd.menders = hands;
}

/// Menders are at work: count them in each town's employment.
pub fn count_menders(st: &State, P: &Params, employment: &mut [f64]) {
    if !P.roads.enabled {
        return;
    }
    for (e, n) in employment.iter_mut().zip(&st.rd.menders) {
        *e += n;
    }
}

/// The tolls in the cost graph: a cell of a trust's road costs `toll` more to
/// pass (half on each edge into it and half on each edge out), except by rail.
pub fn tolls(st: &State, P: &Params, g: &mut tr::Graph) {
    if !P.roads.enabled || P.roads.toll <= 0.0 || !st.rd.cell_trust.iter().any(|&t| t >= 0) {
        return;
    }
    let (H, W) = (st.world.H, st.world.W);
    let gate = |c: usize| if st.rd.cell_trust[c] >= 0 { P.roads.toll } else { 0.0 };
    for a in 0..st.N {
        for k in 0..8 {
            let Some(b) = tr::neighbour(H, W, a, k) else { continue };
            if g.railed[a * 8 + k] || !(st.world.land[a] && st.world.land[b]) {
                continue;
            }
            let add = gate(a) + gate(b);
            if add > 0.0 {
                g.w[a * 8 + k] += tr::step_len(k) * 0.5 * add;
            }
        }
    }
    let finite = g.w.iter().copied().filter(|x| x.is_finite());
    g.min_w = finite.clone().fold(f64::INFINITY, f64::min);
    g.max_w = finite.fold(0.0, f64::max);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A land of 7 by 7 cells with a river running down column 3 from the top
    /// row to the bottom, 100 cells of area upstream everywhere.
    fn valley() -> World {
        let (W, H) = (7, 7);
        let N = W * H;
        let mut w = World { W, H, land: vec![true; N], acc: vec![100.0; N], river: vec![false; N], down: vec![-1; N], ..World::default() };
        for y in 0..H {
            let c = y * W + 3;
            w.river[c] = true;
            w.down[c] = if y + 1 < H { (c + W) as i64 } else { -1 };
        }
        w
    }

    #[test]
    fn a_road_across_the_river_crosses_and_one_beside_it_does_not() {
        let w = valley();
        let mut road = vec![0u8; 49];
        // a road along row 3, across the river
        for x in 0..7 {
            road[3 * 7 + x] = 2;
        }
        let x = crossings(&w, &road, 1);
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].0, 3 * 7 + 3);
        // 4.4 times the root of the flow of 100 cells of area
        assert!((x[0].1 - WIDTH_PER_ROOT_FLOW * (100.0 * PORT_FLOW_PER_CELL).sqrt()).abs() < 1e-9);
        // a road down the valley in the river's own cells and beside them crosses nothing
        let mut road = vec![0u8; 49];
        for y in 0..7 {
            road[y * 7 + 3] = 1;
            road[y * 7 + 2] = 1;
        }
        assert!(crossings(&w, &road, 1).is_empty());
        // a diagonal step of road over a diagonal reach of river
        let mut w = valley();
        for y in 0..7 {
            w.river[y * 7 + 3] = false;
            w.down[y * 7 + 3] = -1;
        }
        w.river[2 * 7 + 4] = true;
        w.down[2 * 7 + 4] = (3 * 7 + 3) as i64;
        let mut road = vec![0u8; 49];
        road[2 * 7 + 3] = 1;
        road[3 * 7 + 4] = 1;
        let x = crossings(&w, &road, 1);
        assert_eq!(x.iter().map(|c| c.0).collect::<Vec<_>>(), vec![2 * 7 + 3]);
        // and a road below the level asked for is not looked at
        assert!(crossings(&w, &road, 2).is_empty());
    }
}
