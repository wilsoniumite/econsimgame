//! The railway's handling, and a road worn by its own traffic. Not in the
//! Python (`docs/rules/haulage.md`).
//!
//! Two mends of the ways after 1900, so that the lorry can take the short
//! hauls, each behind a switch of its own:
//!
//! - **The station** (`haulage.handling`): from the cost graph made at the close
//!   of 1901 the railway is a layer of its own over the cells, as a canal is
//!   (`transport::dijkstra_layers`). A load is on the ground or on a train, and
//!   pays `station` to be put on a train and again to be taken off: the work of
//!   a goods yard, lifting it from a cart to a wagon and making the wagon up
//!   into a train, which is the work of the quay the port's graph has always
//!   charged a load put on a ship (`transport::PORT_COST`). A cart on the road
//!   beside the line, or crossing it, pays nothing for the line. The charge is
//!   in the cost graph's units, so it is carting: money a load's carting bill
//!   pays, and carters' hands (`carting.rs`).
//! - **Wear** (`haulage.wear`): from 1901 a road is worn by what goes over it
//!   by cart (or lorry), not by the trains, boats and ships that pass the same
//!   squares (`sim::update_roads`). The weight passing each square, which the
//!   railway's fares are reckoned on, still counts every mode.
//!
//! Off, nothing here acts: the graph is built as the port builds it, and the
//! roads are worn as they always were.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::canals::Water;
use crate::params::Params;
use crate::state::State;
use crate::traffic::{self, CART, RAIL};
use crate::transport as tr;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HaulageParams {
    /// From `FROM` the railway is a layer of its own in the cost graph, and a
    /// load pays `station` to be put on a train and again to be taken off.
    pub handling: bool,
    /// From `FROM` a road is worn by the weight that goes over it by road alone.
    pub wear: bool,
}

impl Default for HaulageParams {
    /// Off: what the port runs with.
    fn default() -> HaulageParams {
        HaulageParams { handling: false, wear: false }
    }
}

impl HaulageParams {
    /// The game's.
    pub fn game() -> HaulageParams {
        HaulageParams { handling: false, wear: false }
    }
}

/// The first year at whose close the cost graph has the station and the roads
/// are worn by their own traffic.
pub const FROM: i64 = 1901;

/// What it costs, in the cost graph's units, to put a load on a train, and
/// again to take it off: the wharf's charge for putting it on a boat
/// (`canals.wharf`, 0.4), by the same rule, since lifting a load from a cart
/// to a wagon is the same work as lifting it to a boat. (The harbour's 0.6,
/// `transport::PORT_COST`, is a ship's longer loading.)
pub fn station(P: &Params) -> f64 {
    P.canals.wharf
}

/// `State::hl`: the year the cost graph was last made at the close of, and how
/// each cell was reached in each town's tree (`transport::Paths::by`), kept
/// only while the railway is a layer of its own.
#[derive(Clone, Debug, Default)]
pub struct HaulageState {
    pub year: i64,
    pub by: Vec<Vec<u8>>,
}

/// The year whose close the coming cost graph is made at (`sim::yearly`).
pub fn year(st: &mut State, year: i64) {
    st.hl.year = year;
}

/// Whether the cost graph being made has the station.
pub fn on(st: &State, P: &Params) -> bool {
    P.haulage.handling && st.hl.year >= FROM
}

/// The year's cost graph (`land::update_transport`): the port's, or from 1901
/// under `haulage.handling` the one whose railway is a layer of its own.
pub fn build_graph(st: &State, P: &Params, port: &[bool], road_mult: &[f64; 4]) -> tr::Graph {
    tr::build_graph_trains(&st.world, &st.road, &st.rail, port, &st.cover, road_mult, on(st, P).then(|| station(P)))
}

/// Whether, in town `i`'s tree, cell `n` was reached off a train boarded at its predecessor.
pub fn aboard(st: &State, i: usize, n: usize) -> bool {
    st.hl.by.get(i).and_then(|b| b.get(n)) == Some(&tr::BY_TRAIN)
}

/// The cheapest ways by train from one cell with track to every other: for
/// each cell, the squares ridden and the cell before (`u32::MAX`: not reached).
fn rail_tree(g: &tr::Graph, from: usize) -> Vec<(f64, u32)> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let N = g.N;
    let mut cost = vec![f64::INFINITY; N];
    let mut tree = vec![(0.0, u32::MAX); N];
    let mut heap: BinaryHeap<(Reverse<u64>, u32)> = BinaryHeap::new();
    cost[from] = 0.0;
    tree[from] = (0.0, from as u32);
    heap.push((Reverse(0f64.to_bits()), from as u32));
    while let Some((Reverse(bits), u)) = heap.pop() {
        let (d, u) = (f64::from_bits(bits), u as usize);
        if d > cost[u] {
            continue;
        }
        for k in 0..8 {
            let Some(v) = tr::neighbour(g.H, g.W, u, k) else { continue };
            if !g.train[v] {
                continue;
            }
            let nd = d + tr::RAIL_COST * tr::step_len(k);
            if nd < cost[v] {
                cost[v] = nd;
                tree[v] = (tree[u].0 + tr::step_len(k), u as u32);
                heap.push((Reverse(nd.to_bits()), v as u32));
            }
        }
    }
    tree
}

/// Reads the trees of `land::update_transport` for the stretches that go by
/// train, as `canals::Water` does for the boats: where a load rides a train,
/// a cell's predecessor is where it was put on, which may be far off.
pub(crate) struct Rails<'a> {
    g: &'a tr::Graph,
    trees: BTreeMap<usize, Vec<(f64, u32)>>,
}

impl<'a> Rails<'a> {
    pub(crate) fn new(st: &'a State) -> Rails<'a> {
        Rails { g: &st.graph, trees: BTreeMap::new() }
    }

    fn tree(&mut self, p: usize) -> &Vec<(f64, u32)> {
        let g = self.g;
        self.trees.entry(p).or_insert_with(|| rail_tree(g, p))
    }

    /// Squares ridden by train between `p` and `n`.
    pub(crate) fn length(&mut self, p: usize, n: usize) -> f64 {
        self.tree(p)[n].0
    }

    /// The cells with track from `n` to `p`, both ends in, in the order a
    /// train passes them going from `n` to `p`.
    pub(crate) fn route(&mut self, p: usize, n: usize) -> Vec<usize> {
        let tree = self.tree(p);
        let mut out = Vec::new();
        let mut c = n;
        while tree[c].1 != u32::MAX {
            out.push(c);
            if c == p || out.len() > 100_000 {
                break;
            }
            c = tree[c].1 as usize;
        }
        out
    }
}

/// One step of a town's tree, from a cell to its predecessor.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Step {
    /// How it went (`traffic::MODES`).
    pub mode: u8,
    /// Squares it went.
    pub squares: f64,
    /// A stretch by train or by canal, between cells that need not be neighbours.
    pub jump: bool,
}

/// The step from `n` to its predecessor `p` in town `i`'s tree.
pub(crate) fn step(st: &State, water: &mut Water, rails: &mut Rails, i: usize, p: usize, n: usize) -> Step {
    if aboard(st, i, n) {
        return Step { mode: RAIL, squares: rails.length(p, n), jump: true };
    }
    if st.graph.has_boats() && water.afloat(i, p, n) {
        return Step { mode: traffic::BOAT, squares: water.length(p, n), jump: true };
    }
    match traffic::direction(st.world.W, p, n) {
        Some(k) => Step { mode: traffic::step_mode(st, p, k, n), squares: tr::step_len(k), jump: false },
        None => Step { mode: CART, squares: 0.0, jump: false },
    }
}

/// The year's wear (`sim::update_roads`, on the weight its walk of the trees
/// has just laid on each square, `traffic`). With the railway a layer of its
/// own, a stretch by train is one step of a tree from where it was boarded to
/// where it was left, so the squares between were passed over: the weight is
/// laid on them here, as the port's walk lays it on every square a load
/// passes, so that `traffic` is what it always was and the railway's fares are
/// reckoned on it. Under `haulage.wear`, from 1901, the weight each square is
/// worn by is returned: what left it toward its market by road (and, at a
/// town's own square, what arrived there by road), the trade between towns the
/// same. None when the roads are worn by `traffic` as they always were.
pub fn wear(st: &State, P: &Params, year: i64, traffic: &mut [f64]) -> Option<Vec<f64>> {
    let trains = st.graph.has_train() && !st.hl.by.is_empty();
    let worn = P.haulage.wear && year >= FROM;
    if !trains && !worn {
        return None;
    }
    let (N, T) = (st.N, st.T_max);
    let mut wear = vec![0.0; N];
    let mut water = Water::new(st);
    let mut rails = Rails::new(st);
    let mut lay = |rails: &mut Rails, i: usize, p: usize, n: usize, f: f64, traffic: &mut [f64], wear: &mut [f64], root: bool| {
        if f <= 0.0 {
            return;
        }
        let s = step(st, &mut water, rails, i, p, n);
        if s.jump && s.mode == RAIL {
            let way = rails.route(p, n);
            for &c in way.iter().skip(1).take(way.len().saturating_sub(2)) {
                traffic[c] += f;
            }
        }
        if s.mode == CART {
            wear[n] += f;
            if root {
                wear[p] += f;
            }
        }
    };
    let mut acc = vec![0.0; N];
    for i in 0..T {
        if !st.town_active[i] || st.pred[i].len() != N {
            continue;
        }
        let mut any = false;
        for c in 0..N {
            acc[c] = if st.catch[c] == i as i64 { st.cell_flow[c] } else { 0.0 };
            any |= acc[c] > 0.0;
        }
        if !any {
            continue;
        }
        let root = st.town_node[i];
        wear[root] += acc[root];
        for &n in st.settle[i].iter().rev() {
            let n = n as usize;
            let p = st.pred[i][n];
            if p >= 0 {
                let p = p as usize;
                lay(&mut rails, i, p, n, acc[n], traffic, &mut wear, p == root);
                acc[p] += acc[n];
            }
        }
    }
    // the trade between towns, as `transport::pair_traffic` lays it
    if st.trade_F_year.rows == T && st.town_cost.rows == T {
        for i in 0..T {
            if st.pred[i].len() != N {
                continue;
            }
            let root = st.town_node[i] as i64;
            for j in 0..T {
                let f = st.trade_F_year[(i, j)];
                if f <= 0.0 || i == j || !st.town_cost[(i, j)].is_finite() {
                    continue;
                }
                let mut n = st.town_node[j] as i64;
                let mut steps = 0;
                while n != root && n >= 0 && steps < 100_000 {
                    let p = st.pred[i][n as usize];
                    if p >= 0 {
                        lay(&mut rails, i, p as usize, n as usize, f, traffic, &mut wear, false);
                    }
                    n = p;
                    steps += 1;
                }
            }
        }
    }
    worn.then_some(wear)
}
