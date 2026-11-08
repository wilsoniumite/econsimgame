"""Population: growth from living standards, migration toward better wages,
and new towns where work appears far from any market."""
import numpy as np
from scipy import ndimage

from .names import town_name
from .world import site_context


def growth(st, C, P, year, events):
    act = st["town_active"]
    y = st["y_acc"] / 12.0
    st["y_ratio"] = np.where(act, y, 0.0)
    g = np.clip(P["growth_slope"] * np.log(np.maximum(y, 1e-3)) + P["growth_base"], -0.04, P["growth_cap"])
    famine = act & (st["food_min_acc"] < 0.82)
    g = np.where(famine, g - 0.02, g)
    settled = famine & (st["town_founded"] < year - 2)
    hit = np.flatnonzero(settled)
    if len(hit) >= 3:
        worst = hit[np.argmin(st["food_min_acc"][hit])]
        events.append(dict(
            year=year + 0.6, kind="famine", clock="live", weight=0.7,
            title=f"Hunger in {len(hit)} towns",
            text=f"Bread runs short across the country. In {st['town_name'][worst]}, the worst hit, households get {st['food_min_acc'][worst]:.0%} of the food they need at the worst point of the year.",
            town=int(worst), cell=int(st["town_node"][worst]),
        ))
    else:
        for i in hit:
            events.append(dict(
                year=year + 0.6, kind="famine", clock="live", weight=0.5,
                title=f"Hunger in {st['town_name'][i]}",
                text=f"Bread runs short in {st['town_name'][i]}: households get {st['food_min_acc'][i]:.0%} of the food they need at the worst point of the year.",
                town=int(i), cell=int(st["town_node"][i]),
            ))
    st["pop"] = np.where(act, st["pop"] * (1 + g), 0.0)
    st["y_acc"][:] = 0.0
    st["food_min_acc"][:] = 9.0
    return g


def migration(st, C, P):
    act = np.flatnonzero(st["town_active"])
    if len(act) < 2:
        return
    y = np.maximum(st["y_ratio"][act], 1e-3)
    c = st["town_cost"][np.ix_(act, act)]
    gain = np.maximum(y[None, :] / y[:, None] - 1.0, 0.0)
    wgt = gain * np.where(np.isfinite(c), np.exp(-c / P["migration_dist"]), 0.0)
    np.fill_diagonal(wgt, 0.0)
    tot = wgt.sum(axis=1)
    movers = P["migration_rate"] * st["pop"][act] * np.minimum(tot, 1.0)
    share = np.where(tot[:, None] > 0, wgt / np.maximum(tot[:, None], 1e-12), 0.0)
    flow = movers[:, None] * share
    cash_pc = np.maximum(st["hh_cash"][act], 0.0) / np.maximum(st["pop"][act], 1.0)
    st["pop"][act] += flow.sum(axis=0) - flow.sum(axis=1)
    money = flow * cash_pc[:, None]
    st["hh_cash"][act] += money.sum(axis=0) - money.sum(axis=1)
    st["migration_flow"] = flow


def found_towns(st, C, P, rng, year, events):
    """Mining and industrial settlements far from existing markets become towns."""
    world = st["world"]
    H, W = world["H"], world["W"]
    act = np.flatnonzero(st["town_active"])
    if len(act) >= st["T_max"]:
        return []
    work = np.zeros(st["N"])
    np.add.at(work, st["ex_cell"], st["ex_labor"])
    alive = np.flatnonzero(st["f_alive"])
    np.add.at(work, st["f_cell"][alive], st["f_labor"][alive])
    near = ndimage.uniform_filter(work.reshape(H, W), size=7, mode="constant").ravel() * 49.0
    land = world["land"].ravel() & (world["terrain"].ravel() < 3)
    yy, xx = np.divmod(np.arange(st["N"]), W)
    far = st["dcatch"] > P["found_distance"]
    for i in act:
        ty, tx = divmod(int(st["town_node"][i]), W)
        far &= np.hypot(yy - ty, xx - tx) > 8
    score = np.where(land & far & np.isfinite(st["dcatch"]), near, 0.0)
    founded = []
    c = int(np.argmax(score))
    if score[c] > P["found_activity"]:
        # settle on the best nearby site: flat ground, water, close to the work
        cy, cx = divmod(c, W)
        best, bc = -np.inf, c
        for dy in range(-2, 3):
            for dx in range(-2, 3):
                ny, nx = cy + dy, cx + dx
                if 0 <= ny < H and 0 <= nx < W and land[ny * W + nx] and st["cell_K_share"][ny * W + nx] < 0.5:
                    s = near[ny * W + nx] / max(score[c], 1.0) + 0.3 * world["river"][ny, nx] + 0.3 * world["coast"][ny, nx] - 0.2 * (world["terrain"][ny, nx] - 1)
                    if s > best:
                        best, bc = s, ny * W + nx
        n = len(act)
        if n < st["T_max"]:
            ctx = site_context(world, bc)
            name = town_name(rng, ctx, st["used_names"])
            st["town_active"][n] = True
            st["town_node"][n] = bc
            st["town_name"].append(name)
            st["town_founded"][n] = year
            st["town_new"][n] = True
            donor = int(st["catch"][bc])
            st["price"][n] = st["price"][donor]
            st["wage"][n] = st["wage"][donor]
            st["know"][n] = st["know"][donor]
            st["tools_mult"][n] = st["tools_mult"][donor]
            st["imp_pc"][n] = st["imp_pc"][donor]
            st["fill"][n] = st["fill"][donor]
            what = {"mine": "a mining settlement", "coast": "a harbour town", "river": "a river town",
                    "hills": "a hill town", "plain": "a market town"}[ctx]
            events.append(dict(
                year=year + 0.9, kind="town", clock="live", weight=0.6,
                title=f"{name} founded",
                text=f"Enough people now work around this spot, far from any market, that {name} grows up as {what}.",
                town=n, cell=int(bc),
            ))
            founded.append(n)
    return founded


def resettle_new_towns(st, C, P):
    """After catchments change, new towns take the people working in their area."""
    new = np.flatnonzero(st["town_new"])
    if len(new) == 0:
        return
    work = np.zeros(st["N"])
    np.add.at(work, st["ex_cell"], st["ex_labor"])
    alive = np.flatnonzero(st["f_alive"])
    np.add.at(work, st["f_cell"][alive], st["f_labor"][alive])
    old_catch = st["prev_catch"]
    for n in new:
        cells = np.flatnonzero(st["catch"] == n)
        for i in np.unique(old_catch[cells]):
            if i < 0 or i == n:
                continue
            mine = cells[old_catch[cells] == i]
            tot_i = work[old_catch == i].sum()
            frac = work[mine].sum() / max(tot_i, 1e-9) if tot_i > 0 else 0.0
            frac = float(np.clip(frac, 0.02, 0.5))
            for key in ("pop", "hh_cash", "lord_cash"):
                moved = st[key][i] * frac
                st[key][i] -= moved
                st[key][n] += moved
            mv = st["merch_cash"][i] * frac
            st["merch_cash"][i] -= mv
            st["merch_cash"][n] += mv
            iv = st["inv"][i] * frac
            st["inv"][i] -= iv
            st["inv"][n] += iv
            st["outflow"][n] += st["outflow"][i] * frac
        # firms in the new catchment now trade through the new town
    alive = np.flatnonzero(st["f_alive"])
    st["f_town"][alive] = np.where(st["catch"][st["f_cell"][alive]] >= 0, st["catch"][st["f_cell"][alive]], st["f_town"][alive])
    st["town_new"][:] = False
