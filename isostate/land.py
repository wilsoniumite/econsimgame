"""Yearly spatial update: transport, catchments, delivered prices and land use.

Land goes to whoever bids the most rent for it. For land-based sectors the bid
is the Cobb-Douglas land rent a cell would earn at the town's prices and wage,
after carting costs. Industry and towns outbid farms near markets; farms push
outward; forests survive where nobody can pay for the land.
"""
import numpy as np

from . import transport as tr

USE_SEA, USE_WILD, USE_SECTOR0 = 0, 1, 2
USE_INDUSTRY, USE_URBAN, USE_DERELICT = 9, 10, 11
USE_NAMES = ["sea", "wild", "farm", "pasture", "forestry", "quarry", "claypit",
             "ironmine", "coalmine", "industry", "urban", "derelict"]


def update_transport(st, C, P):
    world = st["world"]
    port = np.zeros(st["N"], dtype=bool)
    act = st["town_active"]
    port[st["town_node"][act]] = world["coast"].ravel()[st["town_node"][act]]
    world["_cover"] = st["cover"]
    graph = tr.build_graph(world, st["road"], st["rail"], port)
    nodes = st["town_node"].copy()
    nodes[~act] = nodes[act][0]
    dist, pred = tr.town_paths(graph, nodes)
    dist[~act] = np.inf
    catch, dmin = tr.catchments(dist, act, world["land"].ravel())
    st["graph"], st["dist"], st["pred"] = graph, dist, pred
    st["catch"], st["dcatch"] = catch, dmin
    tc = dist[:, nodes]
    tc[~act, :] = np.inf
    tc[:, ~act] = np.inf
    st["town_cost"] = tc
    delta_fields(st, C, P)


def delta_fields(st, C, P):
    """Delivered price of each good at each cell relative to its town price.

    A cell next to a colliery gets coal cheaper than the town price; a cell far
    out pays carting from town. One shortest-path pass per good, from a virtual
    source joined to every producing cell.
    """
    G, N = C["G"], st["N"]
    d = st["dcatch"]
    delta = np.zeros((G, N))
    land = np.isfinite(d)
    for g in range(G):
        srcs = []
        sel = C["sector_good"][st["ex_sector"]] == g
        if sel.any():
            srcs.append(st["ex_cell"][sel])
        fsel = st["f_alive"] & (C["method_out"][st["f_method"]] == g) & (st["f_K"] > 0)
        if fsel.any():
            srcs.append(st["f_cell"][fsel])
        base = np.where(land, d, 1e6)
        if srcs:
            s = np.unique(np.concatenate(srcs))
            s = s[np.isfinite(d[s])]
            if len(s):
                E = tr.multi_source_field(st["graph"], s, -d[s])
                base = np.minimum(base, np.where(np.isfinite(E), E, 1e6))
        delta[g] = P["freight"] * C["weight"][g] * base
    st["delta"] = delta


def sector_fields(st, C):
    w = st["world"]
    fields = []
    for name in C["sector_field"]:
        if name == "forest":
            fields.append(st["cover"])
        else:
            fields.append(w[name].ravel())
    return fields


def sector_bids(st, C, P, pbar, wbar):
    """Monthly land rent each land-based sector could pay on every cell."""
    N, NS = st["N"], C["NS"]
    catch, d = st["catch"], st["dcatch"]
    ok = catch >= 0
    ct = np.where(ok, catch, 0)
    bids = np.zeros((NS, N))
    fields = sector_fields(st, C)
    w = np.maximum(wbar[ct], 1e-6)
    from .econ import sector_tfp
    for s in range(NS):
        g = C["sector_good"][s]
        a = C["sector_alpha"][s]
        mult, _, _, in_unit = sector_tfp(st, C, P, np.full(N, s), ct, pbar, st["fill"])
        A = C["sector_A"][s] * mult
        npr = pbar[ct, g] - P["freight"] * C["weight"][g] * np.where(ok, d, 1e9) - in_unit
        f = fields[s]
        good = ok & (npr > 0) & (f > 0)
        base = np.where(good, A * f * npr, 0.0)
        bids[s] = np.where(good, a * ((1 - a) / w) ** ((1 - a) / a) * base ** (1 / a), 0.0)
    return bids


def update_urban(st, C, P, year, wbar):
    """Towns grow outward from their centre along the cheapest ground."""
    N, T = st["N"], st["T_max"]
    world = st["world"]
    urban = np.zeros(N, dtype=bool)
    res = np.zeros(N)
    st["urban_rent"] = np.zeros(T)
    st["cell_urban_rent"] = np.zeros(N)
    density = P["urban_density0"] + P["urban_density_growth"] * (year - C["start_year"])
    ind = st["ind_labor"]
    jitter = world["jitter"].ravel()
    water = (world["coast"] | world["river"]).ravel()
    blocked = (st["use"] == USE_INDUSTRY) | (st["use"] == USE_SEA) | (st["use"] >= USE_SECTOR0 + 5) & (st["use"] <= USE_SECTOR0 + 6)
    for i in np.flatnonzero(st["town_active"]):
        share = 0.04 + 0.8 * ind[i] / max(P["participation"] * st["pop"][i], 1.0)
        U = st["pop"][i] * float(np.clip(share, 0.06, 0.95))
        n = max(1, int(np.ceil(U / density)))
        cells = np.flatnonzero(st["catch"] == i)
        cells = cells[~blocked[cells] | (cells == st["town_node"][i])]
        dd = st["dist"][i, cells]
        score = -dd + 0.8 * jitter[cells] + 0.4 * water[cells]
        score[cells == st["town_node"][i]] = 1e9
        k = min(n, len(cells))
        top = cells[np.argsort(-score)[:k]]
        dtop = st["dist"][i, top]
        wgt = np.exp(-dtop / max(dtop.max() + 1.0, 1.0) * 2.0)
        r = U * wgt / wgt.sum()
        urban[top] = True
        res[top] = r
        fringe = dtop.max() + 1.0
        cell_rent = r * P["urban_rent_rate"] * wbar[i] * np.maximum(0.0, 1 - dtop / fringe)
        st["cell_urban_rent"][top] = cell_rent
        st["urban_rent"][i] = cell_rent.sum()
        st["urban_pop"][i] = U
    st["urban"], st["urban_res"] = urban, res
    st["urban_density"] = density


def update_land_use(st, C, P, year, pbar, wbar, initial=False):
    """Each cell's land is shared between sectors in proportion to bid^(1/theta).

    Shares move part of the way toward that target every year, so land use
    shifts smoothly as prices change instead of flipping wholesale. The last
    share is "wild": land not worth the fixed cost of working it.
    """
    N, NS = st["N"], C["NS"]
    land = st["world"]["land"].ravel()
    bids = sector_bids(st, C, P, pbar, wbar)
    ct = np.where(st["catch"] >= 0, st["catch"], 0)
    wild_bid = P["land_fixed_cost"] * np.maximum(wbar[ct], 1e-6)

    update_urban(st, C, P, year, wbar)

    st["derelict_timer"] = np.maximum(st["derelict_timer"] - 1, 0)
    ks = st["cell_K_share"]
    taken = st["urban"] | (ks >= 0.6) | (st["derelict_timer"] > 0) | ~land | (st["catch"] < 0)

    allb = np.vstack([bids, wild_bid[None, :]])
    allb = np.where(allb > 0, allb, 0.0)
    logb = np.log(np.maximum(allb, 1e-12)) / P["land_theta"]
    logb = np.where(allb > 0, logb, -np.inf)
    mx = logb.max(axis=0)
    z = np.exp(logb - np.where(np.isfinite(mx), mx, 0.0))
    target = z / np.maximum(z.sum(axis=0), 1e-12)
    target = target * (1.0 - ks)[None, :]
    target[:, taken] = 0.0
    target[NS, taken & land & ~st["urban"] & (ks < 0.6)] = 1.0

    lam = st["lam"]
    lam_new = target if initial else lam + P["land_adjust"] * (target - lam)
    lam_new[:, taken] = target[:, taken]
    lam_new = np.where(lam_new < 0.01, 0.0, lam_new)
    tot = lam_new.sum(axis=0)
    room = np.where(taken, 1.0, 1.0 - ks)
    lam_new = np.where(tot > 0, lam_new / np.maximum(tot, 1e-12) * room[None, :], 0.0)

    # forest: cover lives on the woodland and wild share; clearing yields timber
    fi = st["forestry_sector"]
    pot = st["world"]["forest_pot"].ravel()
    wood_share = lam_new[fi] + lam_new[NS]
    old_wood = lam[fi] + lam[NS] if not initial else wood_share
    c = st["cover"]
    regrow = P["forest_regrow"] * c * (1 - c / np.maximum(pot, 1e-6)) + 0.01 * (pot - c)
    c_new = np.clip(c + regrow * (wood_share > 0), 0, pot)
    cleared = np.maximum(old_wood - wood_share, 0.0) * c_new
    if not initial and cleared.sum() > 0:
        towns = st["catch"]
        okc = (towns >= 0) & (cleared > 0)
        np.add.at(st["inv"][:, C["good_index"]["timber"]], towns[okc], cleared[okc] * P["clearing_timber"])
    c_new = np.where(wood_share > 0.02, c_new, np.where(taken, 0.0, c_new * 0.5))
    st["cover"] = c_new
    st["lam"] = lam_new

    # dominant use for the map
    dom = np.argmax(lam_new, axis=0)
    use = np.where(dom == NS, USE_WILD, USE_SECTOR0 + dom)
    use = np.where(st["derelict_timer"] > 0, USE_DERELICT, use)
    use = np.where(st["urban"], USE_URBAN, use)
    use = np.where((ks >= 0.25) & ~st["urban"], USE_INDUSTRY, use)
    use = np.where(land & (st["catch"] >= 0), use, np.where(land, USE_WILD, USE_SEA))
    st["use"] = use.astype(np.int8)

    # extractive units (cell, sector) used by the monthly loop
    sec_idx, cell_idx = np.nonzero(lam_new[:NS] > 0)
    order = np.argsort(cell_idx, kind="stable")
    sec_idx, cell_idx = sec_idx[order], cell_idx[order]
    old_key = st["ex_sector"] * N + st["ex_cell"]
    new_key = sec_idx * N + cell_idx
    carried = np.zeros(len(new_key))
    if len(old_key) and len(st.get("ex_Lprev", [])) == len(old_key):
        srt = np.argsort(old_key)
        pos = np.searchsorted(old_key[srt], new_key)
        pos = np.clip(pos, 0, len(old_key) - 1)
        hit = old_key[srt][pos] == new_key
        carried[hit] = st["ex_Lprev"][srt][pos[hit]]
    st["ex_Lprev"] = carried
    fields = sector_fields(st, C)
    fstack = np.vstack(fields)
    st["ex_cell"] = cell_idx
    st["ex_sector"] = sec_idx.astype(np.int64)
    st["ex_lam"] = lam_new[sec_idx, cell_idx]
    st["ex_town"] = st["catch"][cell_idx]
    st["ex_field"] = fstack[sec_idx, cell_idx]
    st["ex_tau"] = P["freight"] * C["weight"][C["sector_good"][sec_idx]] * st["dcatch"][cell_idx]

    # rent on every cell, for firms deciding where to locate and for the map
    rent = (lam_new[:NS] * bids).sum(axis=0)
    rent[st["urban"]] = st["cell_urban_rent"][st["urban"]]
    st["land_rent_only"] = rent.copy()
    alive = np.flatnonzero(st["f_alive"])
    np.add.at(rent, st["f_cell"][alive], st["f_rent"][alive])
    st["cell_rent"] = rent
    st["bids"] = bids
