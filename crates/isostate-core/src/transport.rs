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

use std::cmp::Reverse;
use std::collections::BinaryHeap;

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
    Graph { N, W, H, w, railed }
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
    Graph { N, W, H, w, railed: vec![false; N * 8] }
}

/// Shortest paths from one or more sources, each entered at a starting distance.
/// One source at 0.0 is scipy's `dijkstra(indices=s)`; several sources at their
/// offsets is the virtual source of `multi_source_field` (a node joined to each
/// source by an edge of that length; relaxing those edges first gives exactly
/// these starting distances). Returns distances (inf where unreachable) and
/// predecessors (`NO_PRED` for sources and unreachable cells).
///
/// With `target`, the search stops once the target is settled. Its distance and
/// the predecessors along its path are final at that point, so they are the same
/// as after a full search; other cells may be unfinished.
pub fn dijkstra(g: &Graph, sources: &[(usize, f64)], target: Option<usize>) -> (Vec<f64>, Vec<i64>) {
    let N = g.N;
    let mut dist = vec![f64::INFINITY; N];
    let mut pred = vec![NO_PRED; N];
    let mut done = vec![false; N];
    // Distances are never negative here, so their bit patterns sort like the values.
    let mut heap: BinaryHeap<Reverse<(u64, u32)>> = BinaryHeap::with_capacity(1024);
    for &(s, d0) in sources {
        debug_assert!(d0 >= 0.0);
        if d0 < dist[s] {
            dist[s] = d0;
            heap.push(Reverse((d0.to_bits(), s as u32)));
        }
    }
    while let Some(Reverse((dbits, u))) = heap.pop() {
        let u = u as usize;
        if done[u] || f64::from_bits(dbits) > dist[u] {
            continue;
        }
        done[u] = true;
        if target == Some(u) {
            break;
        }
        let du = dist[u];
        let (y, x) = (u / g.W, u % g.W);
        let interior = y > 0 && y + 1 < g.H && x > 0 && x + 1 < g.W;
        for k in 0..8 {
            let v = if interior {
                let (dy, dx) = NEIGH8[k];
                ((y as isize + dy) as usize) * g.W + (x as isize + dx) as usize
            } else {
                match crate::transport::neighbour(g.H, g.W, u, k) {
                    Some(v) => v,
                    None => continue,
                }
            };
            if done[v] {
                continue;
            }
            let nd = du + g.w[u * 8 + k];
            if nd < dist[v] {
                dist[v] = nd;
                pred[v] = u as i64;
                heap.push(Reverse((nd.to_bits(), v as u32)));
            }
        }
    }
    (dist, pred)
}

/// Distances and predecessor trees from every town. Towns sharing a node (the
/// inactive ones are pointed at the first active town) share one search.
pub fn town_paths(g: &Graph, town_nodes: &[usize]) -> (Mat, Vec<Vec<i64>>) {
    let T = town_nodes.len();
    let mut dist = Mat::zeros(T, g.N);
    let mut pred: Vec<Vec<i64>> = Vec::with_capacity(T);
    for i in 0..T {
        if let Some(j) = (0..i).find(|&j| town_nodes[j] == town_nodes[i]) {
            let row = dist.row(j).to_vec();
            dist.row_mut(i).copy_from_slice(&row);
            let p = pred[j].clone();
            pred.push(p);
            continue;
        }
        let (d, p) = dijkstra(g, &[(town_nodes[i], 0.0)], None);
        dist.row_mut(i).copy_from_slice(&d);
        pred.push(p);
    }
    (dist, pred)
}

/// Each land cell belongs to the town it can reach most cheaply.
pub fn catchments(dist: &Mat, active: &[bool], land: &[bool]) -> (Vec<i64>, Vec<f64>) {
    let (T, N) = (dist.rows, dist.cols);
    let mut catch = vec![-1i64; N];
    let mut dmin = vec![f64::INFINITY; N];
    for c in 0..N {
        // argmin over towns: first minimum, inactive towns count as infinite
        let mut best = 0usize;
        let mut bd = if active[0] { dist[(0, c)] } else { f64::INFINITY };
        for i in 1..T {
            let d = if active[i] { dist[(i, c)] } else { f64::INFINITY };
            if d < bd {
                bd = d;
                best = i;
            }
        }
        if land[c] && bd.is_finite() {
            catch[c] = best as i64;
            dmin[c] = bd;
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
    let (d, _) = dijkstra(g, &init, None);
    d.into_iter().map(|v| v + shift).collect()
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
