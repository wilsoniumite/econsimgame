"""Movement costs, shortest paths, catchments, delivered-price fields and traffic.

The map is a graph with one node per cell and 8-neighbour edges. Edge cost is
the average cost of the two cells times step length, plus a slope penalty.
Roads, navigable rivers, sea lanes and railways make edges cheaper.
"""
import numpy as np
from numba import njit
from scipy.sparse import csr_matrix
from scipy.sparse.csgraph import dijkstra

from .world import HILLS, LOWLAND, MOUNTAINS, NEIGH8

TERRAIN_COST = np.array([0.0, 1.0, 1.7, 3.2])          # sea, lowland, hills, mountains
ROAD_MULT = np.array([1.0, 0.7, 0.45, 0.3])            # none, track, road, turnpike
RIVER_COST = 0.3
SEA_COST = 0.22
RAIL_COST = 0.07
PORT_COST = 0.6
BEACH_COST = 4.0
SLOPE_K = 8.0


def _edges(H, W):
    """Neighbour pairs (a, b) and step lengths for all 8 directions, vectorised."""
    rows, cols, step = [], [], []
    yy, xx = np.mgrid[0:H, 0:W]
    for dy, dx in NEIGH8:
        ny, nx = yy + dy, xx + dx
        ok = (ny >= 0) & (ny < H) & (nx >= 0) & (nx < W)
        rows.append((yy * W + xx)[ok])
        cols.append((ny * W + nx)[ok])
        step.append(np.full(ok.sum(), np.hypot(dy, dx)))
    return np.concatenate(rows), np.concatenate(cols), np.concatenate(step)


def build_graph(world, road, rail, port_mask):
    """Weighted adjacency for the current state of roads, rails and ports."""
    H, W = world["H"], world["W"]
    if "_edges" not in world:
        world["_edges"] = _edges(H, W)
    a, b, step = world["_edges"]
    land = world["land"].ravel()
    terr = world["terrain"].ravel()
    h = world["h"].ravel()
    forest = world.get("_cover", np.zeros(H * W))
    nav = world["navigable"].ravel()

    cell = TERRAIN_COST[terr] * ROAD_MULT[road] * (1 + 0.25 * (forest > 0.6) * (road == 0))
    cost = step * 0.5 * (cell[a] + cell[b]) + SLOPE_K * np.abs(h[a] - h[b]) * (road[a] == 0)
    both_land = land[a] & land[b]
    both_sea = ~land[a] & ~land[b]
    river = both_land & nav[a] & nav[b]
    cost = np.where(river, np.minimum(cost, RIVER_COST * step), cost)
    railed = both_land & (rail[a] > 0) & (rail[b] > 0)
    cost = np.where(railed, RAIL_COST * step, cost)
    cost = np.where(both_sea, SEA_COST * step, cost)
    mixed = ~both_land & ~both_sea
    landside = np.where(land[a], a, b)
    cost = np.where(mixed, np.where(port_mask[landside], PORT_COST, BEACH_COST), cost)
    cost = np.maximum(cost, 1e-3)
    N = H * W
    g = csr_matrix((cost, (a, b)), shape=(N, N))
    return dict(csr=g, a=a, b=b, w=cost, N=N, railed=railed)


def town_paths(graph, town_nodes):
    """Distances and predecessor trees from every town."""
    dist, pred = dijkstra(graph["csr"], directed=True, indices=town_nodes, return_predecessors=True)
    return dist, pred.astype(np.int64)


def catchments(dist, active, land):
    """Each land cell belongs to the town it can reach most cheaply."""
    d = np.where(active[:, None], dist, np.inf)
    catch = np.argmin(d, axis=0)
    dmin = d[catch, np.arange(d.shape[1])]
    catch = np.where(land & np.isfinite(dmin), catch, -1)
    dmin = np.where(catch >= 0, dmin, np.inf)
    return catch, dmin


def multi_source_field(graph, sources, offsets):
    """min over sources of (offset_s + dist(s, c)) using one virtual source node."""
    N = graph["N"]
    if len(sources) == 0:
        return np.full(N, np.inf)
    shift = offsets.min()
    rows = np.concatenate([graph["a"], np.full(len(sources), N)])
    cols = np.concatenate([graph["b"], sources])
    w = np.concatenate([graph["w"], offsets - shift + 1e-6])
    g = csr_matrix((w, (rows, cols)), shape=(N + 1, N + 1))
    d = dijkstra(g, directed=True, indices=N)
    return d[:N] + shift


@njit(cache=True)
def tree_traffic(pred_row, order, flow, traffic):
    """Push cell flows up a shortest-path tree toward its root, adding to traffic."""
    acc = flow.copy()
    for k in range(order.size):
        n = order[k]
        traffic[n] += acc[n]
        p = pred_row[n]
        if p >= 0:
            acc[p] += acc[n]
    return traffic


@njit(cache=True)
def pair_traffic(pred, town_nodes, F, traffic):
    """Add town-to-town flows along their shortest paths."""
    T = F.shape[0]
    for i in range(T):
        root = town_nodes[i]
        for j in range(T):
            f = F[i, j]
            if f <= 0.0 or i == j:
                continue
            n = town_nodes[j]
            steps = 0
            while n != root and n >= 0 and steps < 100000:
                traffic[n] += f
                n = pred[i, n]
                steps += 1
    return traffic


@njit(cache=True)
def path_cells(pred_row, src, dst):
    out = []
    n = dst
    steps = 0
    while n != src and n >= 0 and steps < 100000:
        out.append(n)
        n = pred_row[n]
        steps += 1
    if n == src:
        out.append(src)
        return np.array(out[::-1], dtype=np.int64)
    return np.zeros(0, dtype=np.int64)


def rail_build_graph(world, rail):
    """Construction cost of laying track: slopes, mountains and bridges are expensive."""
    H, W = world["H"], world["W"]
    a, b, step = world["_edges"]
    land = world["land"].ravel()
    terr = world["terrain"].ravel()
    h = world["h"].ravel()
    river = world["river"].ravel()
    cell = np.select([terr == LOWLAND, terr == HILLS, terr == MOUNTAINS], [1.0, 2.2, 6.0], 50.0)
    cell = cell + 1.5 * river
    cell = np.where(rail > 0, 0.15, cell)
    cost = step * 0.5 * (cell[a] + cell[b]) + 40.0 * np.abs(h[a] - h[b])
    cost = np.where(land[a] & land[b], cost, 1e6)
    g = csr_matrix((np.maximum(cost, 1e-3), (a, b)), shape=(H * W, H * W))
    return g
