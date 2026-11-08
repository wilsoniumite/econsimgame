//! Movement costs, shortest paths, catchments, delivered-price fields and traffic.
//! Python: `transport.py`.
//!
//! The map is a graph with one node per cell and 8-neighbour edges. Edge cost is
//! the average cost of the two cells times step length, plus a slope penalty.
//! Roads, navigable rivers, sea lanes and railways make edges cheaper.
//!
//! scipy's sparse-matrix Dijkstra becomes a binary-heap Dijkstra over the grid:
//! edges are stored per cell in the order of `NEIGH8`, which is the column order
//! scipy's CSR matrix uses.

use crate::mat::Mat;
use crate::world::{HILLS, LOWLAND, MOUNTAINS, NEIGH8, World};

pub const TERRAIN_COST: [f64; 4] = [0.0, 1.0, 1.7, 3.2]; // sea, lowland, hills, mountains
pub const ROAD_MULT: [f64; 4] = [1.0, 0.7, 0.45, 0.3]; // none, track, road, turnpike
pub const RIVER_COST: f64 = 0.3;
pub const SEA_COST: f64 = 0.22;
pub const RAIL_COST: f64 = 0.07;
pub const PORT_COST: f64 = 0.6;
pub const BEACH_COST: f64 = 4.0;
pub const SLOPE_K: f64 = 8.0;

/// scipy's predecessor value for sources and unreachable nodes.
pub const NO_PRED: i64 = -9999;

/// Weighted adjacency for the current state of roads, rails and ports.
#[derive(Clone, Debug, Default)]
pub struct Graph {
    pub N: usize,
    pub W: usize,
    pub H: usize,
    /// `w[c * 8 + k]`: cost of the edge from cell c to its neighbour in direction
    /// `NEIGH8[k]`; infinite where the neighbour is off the map.
    pub w: Vec<f64>,
    /// `railed[c * 8 + k]`: both ends of the edge are land with track.
    pub railed: Vec<bool>,
    /// The cheapest and dearest finite edge, which decide how `dijkstra` queues nodes.
    pub min_w: f64,
    pub max_w: f64,
}

impl Graph {
    pub fn new(N: usize, W: usize, H: usize, w: Vec<f64>, railed: Vec<bool>) -> Graph {
        let finite = w.iter().copied().filter(|x| x.is_finite());
        let min_w = finite.clone().fold(f64::INFINITY, f64::min);
        let max_w = finite.fold(0.0, f64::max);
        Graph { N, W, H, w, railed, min_w, max_w }
    }
}

/// The neighbour of cell c in direction k, if it is on the map.
pub fn neighbour(H: usize, W: usize, c: usize, k: usize) -> Option<usize> {
    let (dy, dx) = NEIGH8[k];
    let (y, x) = ((c / W) as isize + dy, (c % W) as isize + dx);
    if y >= 0 && y < H as isize && x >= 0 && x < W as isize { Some(y as usize * W + x as usize) } else { None }
}

/// Step length of direction k: 1 or sqrt(2), as `np.hypot(dy, dx)`.
pub fn step_len(k: usize) -> f64 {
    let (dy, dx) = NEIGH8[k];
    (dy as f64).hypot(dx as f64)
}

/// Python: `build_graph(world, road, rail, port_mask)`, with the forest cover passed
/// in rather than stashed in the world dict (`world["_cover"]`).
pub fn build_graph(world: &World, road: &[u8], rail: &[u8], port_mask: &[bool], cover: &[f64]) -> Graph {
    let (H, W) = (world.H, world.W);
    let N = H * W;
    let cell: Vec<f64> = (0..N)
        .map(|c| {
            let woods = if cover[c] > 0.6 && road[c] == 0 { 0.25 } else { 0.0 };
            TERRAIN_COST[world.terrain[c] as usize] * ROAD_MULT[road[c] as usize] * (1.0 + woods)
        })
        .collect();
    let mut w = vec![f64::INFINITY; N * 8];
    let mut railed = vec![false; N * 8];
    for a in 0..N {
        for k in 0..8 {
            let Some(b) = neighbour(H, W, a, k) else { continue };
            let step = step_len(k);
            let slope = if road[a] == 0 { 1.0 } else { 0.0 };
            let mut cost = step * 0.5 * (cell[a] + cell[b]) + SLOPE_K * (world.h[a] - world.h[b]).abs() * slope;
            let both_land = world.land[a] && world.land[b];
            let both_sea = !world.land[a] && !world.land[b];
            let river = both_land && world.navigable[a] && world.navigable[b];
            if river {
                cost = cost.min(RIVER_COST * step);
            }
            let on_rail = both_land && rail[a] > 0 && rail[b] > 0;
            if on_rail {
                cost = RAIL_COST * step;
            }
            if both_sea {
                cost = SEA_COST * step;
            }
            if !both_land && !both_sea {
                let landside = if world.land[a] { a } else { b };
                cost = if port_mask[landside] { PORT_COST } else { BEACH_COST };
            }
            w[a * 8 + k] = cost.max(1e-3);
            railed[a * 8 + k] = on_rail;
        }
    }
    Graph::new(N, W, H, w, railed)
}

/// Construction cost of laying track: slopes, mountains and bridges are expensive.
pub fn rail_build_graph(world: &World, rail: &[u8]) -> Graph {
    let (H, W) = (world.H, world.W);
    let N = H * W;
    let cell: Vec<f64> = (0..N)
        .map(|c| {
            let base = match world.terrain[c] {
                LOWLAND => 1.0,
                HILLS => 2.2,
                MOUNTAINS => 6.0,
                _ => 50.0,
            };
            let v = base + 1.5 * (world.river[c] as u8 as f64);
            if rail[c] > 0 { 0.15 } else { v }
        })
        .collect();
    let mut w = vec![f64::INFINITY; N * 8];
    for a in 0..N {
        for k in 0..8 {
            let Some(b) = neighbour(H, W, a, k) else { continue };
            let step = step_len(k);
            let mut cost = step * 0.5 * (cell[a] + cell[b]) + 40.0 * (world.h[a] - world.h[b]).abs();
            if !(world.land[a] && world.land[b]) {
                cost = 1e6;
            }
            w[a * 8 + k] = cost.max(1e-3);
        }
    }
    Graph::new(N, W, H, w, vec![false; N * 8])
}

/// The result of one shortest-path search.
#[derive(Clone, Debug, Default)]
pub struct Paths {
    /// Path cost to every cell (inf where unreachable).
    pub dist: Vec<f64>,
    /// Predecessor on the path (`NO_PRED` for sources and unreachable cells).
    pub pred: Vec<i64>,
    /// The reached cells in the order the search settled them. Every cell comes
    /// after its predecessor, so walking this backwards visits children first.
    pub order: Vec<u32>,
}

/// Shortest paths from one or more sources, each entered at a starting distance:
/// scipy's `dijkstra(indices=s, return_predecessors=True)`, tie for tie.
///
/// scipy keeps a `std::priority_queue` of (-distance, cell) pairs, so it settles
/// cells by increasing distance and, at equal distance, the higher cell index
/// first; a cell's predecessor is the first settled neighbour that reaches it at
/// its final distance (relaxation needs a strict improvement). A binary heap with
/// the same ordering settles cells in exactly the same order, so the predecessor
/// trees match scipy's even where two paths cost exactly the same. That matters:
/// the first roads, the yearly traffic and the railway routes follow these trees.
///
/// With `target`, the search stops once the target is settled. Its distance and
/// the predecessors along its path are final at that point, so they are the same
/// as after a full search; other cells may be unfinished.
pub fn dijkstra(g: &Graph, sources: &[(usize, f64)], target: Option<usize>) -> Paths {
    dijkstra_heap(g, sources, target)
}

/// Path costs only, from one or more sources entered at starting distances. Several
/// sources at their offsets is the virtual source of `multi_source_field` (a node
/// joined to each source by an edge of that length; relaxing those edges first
/// gives exactly these starting distances).
///
/// Distances do not depend on how ties are broken, so this uses Dial's bucket
/// queue (see `dijkstra_buckets`), about twice as fast as the heap, unless the
/// edge weights span too wide a range for buckets. Both give exactly the
/// distances scipy gives.
pub fn distances(g: &Graph, sources: &[(usize, f64)]) -> Vec<f64> {
    if g.min_w > 0.0 && g.max_w / g.min_w < 65_536.0 {
        dijkstra_buckets(g, sources, None).dist
    } else {
        dijkstra_heap(g, sources, None).dist
    }
}

/// The 8 neighbours of u that are on the map, with the index of each edge.
#[inline]
fn for_each_edge(g: &Graph, u: usize, mut f: impl FnMut(usize, usize)) {
    let (y, x) = (u / g.W, u % g.W);
    if y > 0 && y + 1 < g.H && x > 0 && x + 1 < g.W {
        for (k, (dy, dx)) in NEIGH8.iter().enumerate() {
            f(k, ((y as isize + dy) as usize) * g.W + (x as isize + dx) as usize);
        }
    } else {
        for k in 0..8 {
            if let Some(v) = neighbour(g.H, g.W, u, k) {
                f(k, v);
            }
        }
    }
}

/// Dijkstra with Dial's bucket queue. Bucket b holds tentative distances in
/// [b * width, (b + 1) * width), with the width a hair under the cheapest edge:
/// relaxing a node always lands in a later bucket, so no node in a bucket can
/// shorten another one in the same bucket, and a whole bucket is final once the
/// buckets before it are done. Distances lie within one dearest edge of the
/// bucket being settled, so a ring of buckets suffices. Sources enter when the
/// sweep reaches their bucket, which is what the virtual source does.
fn dijkstra_buckets(g: &Graph, sources: &[(usize, f64)], target: Option<usize>) -> Paths {
    let N = g.N;
    let mut dist = vec![f64::INFINITY; N];
    let mut pred = vec![NO_PRED; N];
    let mut done = vec![false; N];
    let mut order: Vec<u32> = Vec::with_capacity(N);
    // a step of one cheapest edge moves at least 1 + 1e-9 buckets, far more than rounding
    let per_width = 1.0 / (g.min_w * (1.0 - 1e-9));
    let bucket = |d: f64| (d * per_width) as usize;
    let ring = ((g.max_w * per_width) as usize + 2).next_power_of_two();
    let mask = ring - 1;
    let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); ring];
    let mut srcs: Vec<(usize, f64)> = sources.to_vec();
    srcs.sort_unstable_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
    let mut next_src = 0;
    let mut queued = 0usize;
    let mut cur = match srcs.first() {
        Some(&(_, d0)) => bucket(d0),
        None => return Paths { dist, pred, order },
    };
    loop {
        // sources whose starting distance falls in this bucket
        while next_src < srcs.len() && bucket(srcs[next_src].1) <= cur {
            let (s, d0) = srcs[next_src];
            debug_assert!(d0 >= 0.0);
            next_src += 1;
            if d0 < dist[s] {
                dist[s] = d0;
                buckets[cur & mask].push(s as u32);
                queued += 1;
            }
        }
        if queued == 0 {
            match srcs.get(next_src) {
                Some(&(_, d0)) => {
                    cur = bucket(d0);
                    continue;
                }
                None => break,
            }
        }
        // the order within a bucket cannot change any distance (see above)
        let mut items = std::mem::take(&mut buckets[cur & mask]);
        queued -= items.len();
        for &u in &items {
            let u = u as usize;
            if done[u] {
                continue; // queued again at a shorter distance and settled then
            }
            done[u] = true;
            order.push(u as u32);
            if target == Some(u) {
                return Paths { dist, pred, order };
            }
            let du = dist[u];
            let wu = &g.w[u * 8..u * 8 + 8];
            for_each_edge(g, u, |k, v| {
                if done[v] {
                    return;
                }
                let nd = du + wu[k];
                if nd < dist[v] {
                    dist[v] = nd;
                    pred[v] = u as i64;
                    buckets[bucket(nd) & mask].push(v as u32);
                    queued += 1;
                }
            });
        }
        items.clear();
        buckets[cur & mask] = items; // keep the allocation
        cur += 1;
    }
    Paths { dist, pred, order }
}

/// Dijkstra with a binary heap (lazy deletion), for graphs whose edge weights span
/// too wide a range for buckets (the rail construction graph prices sea at 1e6).
fn dijkstra_heap(g: &Graph, sources: &[(usize, f64)], target: Option<usize>) -> Paths {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let N = g.N;
    let mut dist = vec![f64::INFINITY; N];
    let mut pred = vec![NO_PRED; N];
    let mut done = vec![false; N];
    let mut order: Vec<u32> = Vec::with_capacity(N);
    // A max-heap of (Reverse(distance), cell): the shortest distance comes out
    // first and, among equal distances, the highest cell index, as in scipy.
    // Distances are never negative, so their bit patterns sort like the values.
    let mut heap: BinaryHeap<(Reverse<u64>, u32)> = BinaryHeap::new();
    for &(s, d0) in sources {
        debug_assert!(d0 >= 0.0);
        if d0 < dist[s] {
            dist[s] = d0;
            heap.push((Reverse(d0.to_bits()), s as u32));
        }
    }
    while let Some((Reverse(dbits), u)) = heap.pop() {
        let u = u as usize;
        if done[u] || f64::from_bits(dbits) > dist[u] {
            continue;
        }
        done[u] = true;
        order.push(u as u32);
        if target == Some(u) {
            break;
        }
        let du = dist[u];
        for_each_edge(g, u, |k, v| {
            if done[v] {
                return;
            }
            let nd = du + g.w[u * 8 + k];
            if nd < dist[v] {
                dist[v] = nd;
                pred[v] = u as i64;
                heap.push((Reverse(nd.to_bits()), v as u32));
            }
        });
    }
    Paths { dist, pred, order }
}

/// Distances, predecessor trees and settle orders from every active town.
///
/// Python searches from every town slot, pointing the inactive ones at the first
/// active town, and then sets their distances to infinity; nothing reads their
/// trees. Here inactive rows are infinite and their trees left empty.
pub fn town_paths(g: &Graph, town_nodes: &[usize], active: &[bool]) -> (Mat, Vec<Vec<i64>>, Vec<Vec<u32>>) {
    let T = town_nodes.len();
    let mut dist = Mat::filled(T, g.N, f64::INFINITY);
    let mut pred: Vec<Vec<i64>> = vec![Vec::new(); T];
    let mut order: Vec<Vec<u32>> = vec![Vec::new(); T];
    for i in 0..T {
        if !active[i] {
            continue;
        }
        let p = dijkstra(g, &[(town_nodes[i], 0.0)], None);
        dist.row_mut(i).copy_from_slice(&p.dist);
        pred[i] = p.pred;
        order[i] = p.order;
    }
    (dist, pred, order)
}

/// Each land cell belongs to the town it can reach most cheaply (the first such
/// town on a tie, as `np.argmin`).
pub fn catchments(dist: &Mat, active: &[bool], land: &[bool]) -> (Vec<i64>, Vec<f64>) {
    let (T, N) = (dist.rows, dist.cols);
    let mut best = vec![0usize; N];
    let mut bd = vec![f64::INFINITY; N];
    for i in 0..T {
        if !active[i] {
            continue; // inactive towns count as infinitely far
        }
        for (c, &d) in dist.row(i).iter().enumerate() {
            if d < bd[c] {
                bd[c] = d;
                best[c] = i;
            }
        }
    }
    let mut catch = vec![-1i64; N];
    let mut dmin = vec![f64::INFINITY; N];
    for c in 0..N {
        if land[c] && bd[c].is_finite() {
            catch[c] = best[c] as i64;
            dmin[c] = bd[c];
        }
    }
    (catch, dmin)
}

/// min over sources of (offset_s + dist(s, c)) using one virtual source node.
pub fn multi_source_field(g: &Graph, sources: &[usize], offsets: &[f64]) -> Vec<f64> {
    if sources.is_empty() {
        return vec![f64::INFINITY; g.N];
    }
    let shift = offsets.iter().cloned().fold(f64::INFINITY, f64::min);
    let init: Vec<(usize, f64)> = sources.iter().zip(offsets).map(|(&s, &o)| (s, o - shift + 1e-6)).collect();
    distances(g, &init).into_iter().map(|v| v + shift).collect()
}

/// Push cell flows up a shortest-path tree toward its root, adding to traffic.
pub fn tree_traffic(pred_row: &[i64], order: &[usize], flow: &[f64], traffic: &mut [f64]) {
    let mut acc = flow.to_vec();
    for &n in order {
        traffic[n] += acc[n];
        let p = pred_row[n];
        if p >= 0 {
            acc[p as usize] += acc[n];
        }
    }
}

/// Add town-to-town flows along their shortest paths.
pub fn pair_traffic(pred: &[Vec<i64>], town_nodes: &[usize], F: &Mat, traffic: &mut [f64]) {
    let T = F.rows;
    for i in 0..T {
        let root = town_nodes[i] as i64;
        for j in 0..T {
            let f = F[(i, j)];
            if f <= 0.0 || i == j {
                continue;
            }
            let mut n = town_nodes[j] as i64;
            let mut steps = 0;
            while n != root && n >= 0 && steps < 100_000 {
                traffic[n as usize] += f;
                n = pred[i][n as usize];
                steps += 1;
            }
        }
    }
}

/// The cells of the path src..=dst along a predecessor tree, or empty if dst does not lead to src.
pub fn path_cells(pred_row: &[i64], src: usize, dst: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut n = dst as i64;
    let mut steps = 0;
    while n != src as i64 && n >= 0 && steps < 100_000 {
        out.push(n as usize);
        n = pred_row[n as usize];
        steps += 1;
    }
    if n == src as i64 {
        out.push(src);
        out.reverse();
        return out;
    }
    Vec::new()
}
