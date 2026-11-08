"""Procedural map: terrain, coast, rivers, resources and the first towns.

All fields are 2D arrays of shape (H, W); cell index c = y * W + x.
"""
import heapq

import numpy as np
from scipy import ndimage

NEIGH8 = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)]

SEA, LOWLAND, HILLS, MOUNTAINS = 0, 1, 2, 3


def smooth_noise(rng, H, W, scale, octaves=5, persistence=0.5):
    """Fractal value noise in [0, 1]: random grids upsampled with cubic splines."""
    out = np.zeros((H, W))
    amp, total = 1.0, 0.0
    for o in range(octaves):
        step = max(2, int(round(scale / 2 ** o)))
        gh, gw = H // step + 4, W // step + 4
        grid = rng.random((gh, gw))
        z = ndimage.zoom(grid, step, order=3)
        oy, ox = rng.integers(0, step), rng.integers(0, step)
        out += amp * z[oy:oy + H, ox:ox + W]
        total += amp
        amp *= persistence
    out /= total
    return (out - out.min()) / (out.max() - out.min() + 1e-12)


def blob(H, W, cy, cx, sy, sx, angle=0.0):
    yy, xx = np.mgrid[0:H, 0:W]
    dy, dx = yy - cy, xx - cx
    ca, sa = np.cos(angle), np.sin(angle)
    u = (dx * ca + dy * sa) / sx
    v = (-dx * sa + dy * ca) / sy
    return np.exp(-(u ** 2 + v ** 2))


def bezier_distance(H, W, p0, p1, p2):
    """Distance (in cells) from every cell to a quadratic Bezier curve."""
    t = np.linspace(0, 1, 300)[:, None]
    pts = (1 - t) ** 2 * np.array(p0) + 2 * (1 - t) * t * np.array(p1) + t ** 2 * np.array(p2)
    yy, xx = np.mgrid[0:H, 0:W]
    d = np.full((H, W), np.inf)
    for py, px in pts:
        d = np.minimum(d, np.hypot(yy - py, xx - px))
    return d


def fill_pits(elev, land):
    """Priority-flood so every land cell drains to the sea."""
    H, W = elev.shape
    filled = elev.copy()
    done = ~land.copy()
    heap = []
    for y in range(H):
        for x in range(W):
            if land[y, x]:
                edge = y in (0, H - 1) or x in (0, W - 1)
                if not edge:
                    for dy, dx in NEIGH8:
                        if not land[y + dy, x + dx]:
                            edge = True
                            break
                if edge:
                    heapq.heappush(heap, (filled[y, x], y, x))
                    done[y, x] = True
    while heap:
        e, y, x = heapq.heappop(heap)
        for dy, dx in NEIGH8:
            ny, nx = y + dy, x + dx
            if 0 <= ny < H and 0 <= nx < W and not done[ny, nx]:
                done[ny, nx] = True
                filled[ny, nx] = max(filled[ny, nx], e + 1e-5)
                heapq.heappush(heap, (filled[ny, nx], ny, nx))
    return filled


def flow_directions(filled, land):
    """Downstream neighbour index for each land cell (-1 for outlets/sea)."""
    H, W = filled.shape
    down = np.full(H * W, -1, dtype=np.int64)
    for y in range(H):
        for x in range(W):
            if not land[y, x]:
                continue
            best, bi = filled[y, x], -1
            for dy, dx in NEIGH8:
                ny, nx = y + dy, x + dx
                if 0 <= ny < H and 0 <= nx < W:
                    e = filled[ny, nx] if land[ny, nx] else -1.0
                    if e < best:
                        best, bi = e, ny * W + nx
            down[y * W + x] = bi
    return down


def flow_accumulation(filled, land, down):
    H, W = filled.shape
    acc = np.where(land.ravel(), 1.0, 0.0)
    order = np.argsort(-np.where(land, filled, -np.inf).ravel())
    for c in order:
        if not land.ravel()[c]:
            continue
        d = down[c]
        if d >= 0:
            acc[d] += acc[c]
    return acc.reshape(H, W)


def generate_world(rng, W=120, H=84, n_towns=6, land_fraction=0.58):
    yy, xx = np.mgrid[0:H, 0:W]
    fx, fy = xx / (W - 1), yy / (H - 1)

    base = smooth_noise(rng, H, W, scale=26, octaves=5, persistence=0.55)
    detail = smooth_noise(rng, H, W, scale=8, octaves=3, persistence=0.5)
    # Domain-warped falloff gives bays and peninsulas instead of a round blob.
    wx = smooth_noise(rng, H, W, scale=22, octaves=3) - 0.5
    wy = smooth_noise(rng, H, W, scale=22, octaves=3) - 0.5
    falloff = np.sqrt(((fx + 0.22 * wx - 0.5) / 0.5) ** 2 + ((fy + 0.22 * wy - 0.52) / 0.47) ** 2)
    island = 1 - np.clip(falloff, 0, 1.4) ** 2.3

    # A curved spine of hills, like a Pennine chain, with randomised control points.
    p0 = (H * rng.uniform(0.12, 0.25), W * rng.uniform(0.25, 0.4))
    p1 = (H * rng.uniform(0.4, 0.6), W * rng.uniform(0.45, 0.7))
    p2 = (H * rng.uniform(0.75, 0.9), W * rng.uniform(0.35, 0.6))
    ridge_d = bezier_distance(H, W, p0, p1, p2)
    ridge = np.exp(-(ridge_d / (0.07 * W)) ** 2) * (0.6 + 0.6 * base)

    elev = 0.5 * base + 0.14 * detail + 0.4 * ridge + 0.55 * island
    elev[[0, 1, -2, -1], :] = elev.min()
    elev[:, [0, 1, -2, -1]] = elev.min()
    sea_level = np.quantile(elev, 1 - land_fraction)
    land = elev > sea_level
    # drop tiny islands and lakes for a cleaner map
    lab, n = ndimage.label(land)
    if n > 1:
        sizes = ndimage.sum(land, lab, range(1, n + 1))
        keep = 1 + int(np.argmax(sizes))
        land = lab == keep
    sea_lab, n = ndimage.label(~land)
    if n > 1:
        border_labels = set(np.unique(np.concatenate([sea_lab[0], sea_lab[-1], sea_lab[:, 0], sea_lab[:, -1]])))
        lakes = ~land & ~np.isin(sea_lab, list(border_labels))
        land = land | lakes

    h = np.where(land, (elev - sea_level) / (elev[land].max() - sea_level), 0.0)
    h = np.clip(h, 0, 1) ** 1.35

    terrain = np.full((H, W), SEA, dtype=np.uint8)
    terrain[land & (h < 0.24)] = LOWLAND
    terrain[land & (h >= 0.24) & (h < 0.5)] = HILLS
    terrain[land & (h >= 0.5)] = MOUNTAINS

    filled = fill_pits(h, land)
    down = flow_directions(filled, land)
    acc = flow_accumulation(filled, land, down)
    river = land & (acc >= 45)
    navigable = land & (acc >= 140)

    coast = land & ndimage.binary_dilation(~land, structure=np.ones((3, 3)))
    near_river = ndimage.distance_transform_edt(~river)

    n1 = smooth_noise(rng, H, W, scale=14, octaves=4)
    n2 = smooth_noise(rng, H, W, scale=10, octaves=4)
    n3 = smooth_noise(rng, H, W, scale=12, octaves=4)
    by_terrain = lambda lo, hi, mo: np.select([terrain == LOWLAND, terrain == HILLS, terrain == MOUNTAINS], [lo, hi, mo], 0.0)

    fertility = by_terrain(0.78, 0.38, 0.06) * (0.6 + 0.55 * n1) + 0.18 * np.exp(-near_river / 2.0) * land
    fertility = np.clip(fertility, 0, 1) * land
    pasture = np.clip(by_terrain(0.5, 0.75, 0.35) * (0.7 + 0.45 * n2), 0, 1) * land
    forest_pot = np.clip(by_terrain(0.72, 0.9, 0.45) * (0.65 + 0.5 * n3), 0, 1) * land
    rock = np.clip(by_terrain(0.08, 0.5, 0.85) * (0.6 + 0.6 * n2), 0, 1) * land
    clay = np.clip(np.where(terrain == LOWLAND, 0.25 + 0.55 * np.exp(-near_river / 2.5), 0.08) * (0.6 + 0.6 * n3), 0, 1) * land

    # Ore bodies in the hills, coalfields on the flanks of the spine.
    ore = np.zeros((H, W))
    hill_cells = np.argwhere((terrain >= HILLS) & land)
    for _ in range(7):
        cy, cx = hill_cells[rng.integers(len(hill_cells))]
        ore = np.maximum(ore, rng.uniform(0.55, 1.0) * blob(H, W, cy, cx, rng.uniform(1.5, 3), rng.uniform(1.5, 3.5), rng.uniform(0, np.pi)))
    ore = np.where(ore > 0.15, ore, 0.0) * land

    coal = np.zeros((H, W))
    flank = np.argwhere(land & (ridge_d > 0.06 * W) & (ridge_d < 0.16 * W) & (terrain <= HILLS))
    for _ in range(4):
        cy, cx = flank[rng.integers(len(flank))]
        coal = np.maximum(coal, rng.uniform(0.7, 1.0) * blob(H, W, cy, cx, rng.uniform(3, 4.5), rng.uniform(6, 9), rng.uniform(0, np.pi)))
    coal = np.where(coal > 0.18, coal, 0.0) * land
    ore = np.maximum(ore, 0.4 * coal)  # ironstone bands in the coal measures

    world = dict(
        W=W, H=H, land=land, h=h, terrain=terrain, down=down, acc=acc, river=river,
        navigable=navigable, coast=coast, fertility=fertility, pasture=pasture,
        forest_pot=forest_pot, rock=rock, clay=clay, ore=ore, coal=coal,
        jitter=smooth_noise(rng, H, W, scale=4, octaves=2),
    )
    world["town_sites"] = pick_town_sites(world, rng, n_towns)
    # Woods are already thinned around old settlements in 1750.
    yy, xx = np.mgrid[0:H, 0:W]
    cleared = np.zeros((H, W))
    for c in world["town_sites"]:
        ty, tx = divmod(c, W)
        cleared = np.maximum(cleared, np.exp(-np.hypot(yy - ty, xx - tx) / 9.0))
    world["forest0"] = forest_pot * (1 - 0.85 * cleared)
    return world


def pick_town_sites(world, rng, n):
    land, terrain = world["land"], world["terrain"]
    H, W = land.shape
    fert_sum = ndimage.uniform_filter(world["fertility"], size=11)
    near_nav = ndimage.binary_dilation(world["navigable"], iterations=1)
    score = fert_sum + 0.35 * near_nav + 0.25 * world["coast"] + 0.08 * rng.random((H, W))
    score[~land | (terrain == 3)] = -np.inf
    score[[0, 1, 2, -3, -2, -1], :] = -np.inf
    score[:, [0, 1, 2, -3, -2, -1]] = -np.inf
    yy, xx = np.mgrid[0:H, 0:W]
    sites = []
    for _ in range(n):
        c = int(np.argmax(score))
        if not np.isfinite(score.flat[c]):
            break
        sites.append(c)
        ty, tx = divmod(c, W)
        score[np.hypot(yy - ty, xx - tx) < 17] = -np.inf
    return sites


def site_context(world, c):
    y, x = divmod(c, world["W"])
    if world["coast"][y, x]:
        return "coast"
    if world["coal"][y, x] > 0.2 or world["ore"][y, x] > 0.2:
        return "mine"
    ys, xs = slice(max(0, y - 1), y + 2), slice(max(0, x - 1), x + 2)
    if world["river"][ys, xs].any():
        return "river"
    if world["terrain"][y, x] >= HILLS:
        return "hills"
    return "plain"
