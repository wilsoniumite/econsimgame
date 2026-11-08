"""Firms: where they set up, when they grow, when they fail, and why.

Entry is local and short-sighted. Each town's savers look at a handful of
candidate sites for every method the town knows, price the output and inputs
*at that site* (delivered-price fields), add the rent needed to outbid the
current land use, and back the best returns. Every new firm keeps the numbers
that justified it, so the map can say why a works stands where it does.
"""
import numpy as np
from scipy import ndimage

from .econ import firm_coefficients, heat_mix
from .land import USE_DERELICT, USE_INDUSTRY, USE_NAMES, USE_SECTOR0
from .names import firm_name


def learning(C, P, Q):
    return np.minimum(P["learn_cap"], C["m_A0"] * (1.0 + Q / C["m_Q0"]) ** C["m_b"])


def same_method_counts(st, C):
    """Number of firms of each method within a 5x5 window of every cell."""
    H, W = st["world"]["H"], st["world"]["W"]
    counts = np.zeros((C["M"], H * W))
    alive = np.flatnonzero(st["f_alive"])
    np.add.at(counts, (st["f_method"][alive], st["f_cell"][alive]), 1.0)
    out = np.empty_like(counts)
    for m in range(C["M"]):
        out[m] = ndimage.uniform_filter(counts[m].reshape(H, W), size=5, mode="constant").ravel() * 25.0
    return out


def update_productivity(st, C, P):
    A = learning(C, P, st["method_Q"])
    st["method_A"] = A
    counts = same_method_counts(st, C)
    st["same_counts"] = counts
    alive = np.flatnonzero(st["f_alive"])
    m, c, t = st["f_method"][alive], st["f_cell"][alive], st["f_town"][alive]
    k = st["know"][t, m]
    agglo = 1.0 + P["agglo_same"] * np.log1p(np.maximum(counts[m, c] - 1.0, 0.0))
    st["f_prod"][alive] = A[m] * (0.5 + 0.5 * k) * agglo


def evaluate_sites(st, C, P, i, m, cells, K, pbar, wbar):
    """Return ROI and the accounting behind it for method m on candidate cells."""
    A = st["method_A"][m]
    k = st["know"][i, m]
    agglo = 1.0 + P["agglo_same"] * np.log1p(st["same_counts"][m, cells])
    prod = A * (0.5 + 0.5 * k) * agglo
    n = len(cells)
    meth = np.full(n, m)
    a, h, l = firm_coefficients(C, meth, np.full(n, K), prod)
    pin = np.maximum(pbar[i][None, :] + st["delta"][:, cells].T, 0.25 * pbar[i][None, :])
    fuels = C["fuel_goods"]
    _, hunit = heat_mix(pin[:, fuels], C["heat"][fuels], P["heat_theta"])
    out = C["method_out"][m]
    pout = pbar[i, out] - P["freight"] * C["weight"][out] * st["dcatch"][cells]
    in_cost = (a * pin).sum(axis=1)
    heat_cost = h * hunit
    labor_cost = l * wbar[i]
    margin = pout - in_cost - heat_cost - labor_cost
    foot = K / (P["footprint_K_mult"] * C["m_K0"][m])
    rent_bid = np.maximum(0.01, 1.25 * st["land_rent_only"][cells] * foot)
    profit_m = K * 0.85 * margin - rent_bid
    cap_cost = K * (C["m_cap"][m] * pbar[i]).sum()
    roi = 12.0 * profit_m / max(cap_cost, 1e-9)
    return dict(roi=roi, margin=margin, pout=pout, pin=pin, a=a, h=h, hunit=hunit, l=l,
                rent_bid=rent_bid, cap_cost=cap_cost, agglo=agglo, in_cost=in_cost,
                heat_cost=heat_cost, labor_cost=labor_cost)


def candidate_cells(st, C, P, i, m, rng):
    cells = st["town_cells"][i]
    if len(cells) == 0:
        return cells
    use = st["use"][cells]
    ok = (st["cell_K_share"][cells] < 0.9) & (use != 0) & (use != USE_DERELICT)
    cells = cells[ok]
    if len(cells) == 0:
        return cells
    d = st["dcatch"][cells]
    water = st["water"][cells]
    near_same = st["same_counts"][m, cells]
    near_any = st["any_counts"][cells]
    w = np.exp(-d / 10.0) * (1 + 1.5 * water) * (1 + 2.0 * near_same + 0.5 * near_any) + 1e-12
    n = min(P["candidates"], len(cells))
    pick = rng.choice(len(cells), size=n, replace=False, p=w / w.sum())
    chosen = [cells[pick]]
    # resource-oriented candidates: where the bulkiest input is cheapest
    a = C["m_in"][m] * C["weight"]
    if a.max() > 0:
        g = int(np.argmax(a))
        order = np.argsort(st["delta"][g, cells])[:8]
        chosen.append(cells[order])
    return np.unique(np.concatenate(chosen))


def create_firm(st, C, P, rng, i, m, cell, K, ev, year, funding, name=None, parent=None):
    fid = st["F_count"]
    if fid >= len(st["f_alive"]):
        return -1
    st["F_count"] += 1
    st["f_alive"][fid] = True
    st["f_method"][fid] = m
    st["f_cell"][fid] = cell
    st["f_town"][fid] = i
    st["f_K"][fid] = 0.0
    st["f_order"][fid] = K * C["m_cap"][m]
    st["f_order_K"][fid] = K
    st["f_cash"][fid] = funding
    st["f_rent"][fid] = ev["rent_bid_one"]
    st["f_born"][fid] = year
    st["f_died"][fid] = -1
    st["f_u"][fid] = 0.5
    st["f_fill"][fid] = 1.0
    st["f_prod"][fid] = 1.0
    st["f_profit_acc"][fid] = 0.0
    st["f_loss_years"][fid] = 0
    nm = name or firm_name(rng, C["method_name"][m], st["town_name"][i])
    st["f_name"].append(nm)
    st["f_reason"].append(ev["reason"])
    st["f_parent"][fid] = -1 if parent is None else parent
    if st["firm_at"][cell] < 0:
        st["firm_at"][cell] = fid
    st["cell_K_share"][cell] += K / (P["footprint_K_mult"] * C["m_K0"][m])
    return fid


def reason_record(C, st, i, m, cell, ev, j, K, year, kind):
    """The numbers that justified a firm, kept for the map inspector."""
    inputs = []
    for g in np.flatnonzero(ev["a"][j] > 0):
        inputs.append([int(g), round(float(ev["a"][j][g]), 4), round(float(ev["pin"][j][g]), 4)])
    return dict(
        kind=kind, year=int(year), town=int(i), cell=int(cell), K=round(float(K), 4),
        roi=round(float(ev["roi"][j]), 4), margin=round(float(ev["margin"][j]), 4),
        pout=round(float(ev["pout"][j]), 4), inputs=inputs,
        heat=[round(float(ev["h"][j]), 4), round(float(ev["hunit"][j]), 4)],
        labor=[round(float(ev["l"][j]), 4), round(float(ev["labor_cost"][j] / max(ev["l"][j], 1e-12)), 4)],
        rent=round(float(ev["rent_bid"][j]), 4), displaced=USE_NAMES[int(st["use"][cell])],
        agglo=round(float(ev["agglo"][j]), 3), dist=round(float(st["dcatch"][cell]), 2),
        n_cand=int(len(ev["roi"])),
    )


def town_pool(st, P, i):
    return P["invest_frac_hh"] * max(st["hh_cash"][i], 0.0) + P["invest_frac_lord"] * max(st["lord_cash"][i], 0.0)


def draw_pool(st, P, i, amount):
    """Take funding from a town's households and landlords in proportion to what they offered."""
    hh = P["invest_frac_hh"] * max(st["hh_cash"][i], 0.0)
    lo = P["invest_frac_lord"] * max(st["lord_cash"][i], 0.0)
    tot = hh + lo
    if tot <= 0:
        return 0.0
    amount = min(amount, tot)
    st["hh_cash"][i] -= amount * hh / tot
    st["lord_cash"][i] -= amount * lo / tot
    return amount


def entry(st, C, P, rng, year, pbar, wbar, max_new=None):
    """New firms where returns are best."""
    proposals = []
    know_m = st["know"][:, :C["M"]]
    alive = np.flatnonzero(st["f_alive"])
    n_tm = np.zeros((st["T_max"], C["M"]))
    np.add.at(n_tm, (st["f_town"][alive], st["f_method"][alive]), 1.0)
    for i in np.flatnonzero(st["town_active"]):
        cap_n = 2 + st["pop"][i] / 40000.0
        for m in np.flatnonzero(know_m[i] > 0.02):
            cells = candidate_cells(st, C, P, i, m, rng)
            if len(cells) == 0:
                continue
            K = P["entry_K_frac"] * C["m_K0"][m]
            ev = evaluate_sites(st, C, P, i, m, cells, K, pbar, wbar)
            j = int(np.argmax(ev["roi"]))
            need = P["hurdle"] * (1.0 if n_tm[i, m] < cap_n else 3.0)
            if ev["roi"][j] > need:
                proposals.append((ev["roi"][j], i, m, cells, ev, j, K))
    proposals.sort(key=lambda x: -x[0])
    made = 0
    per_tm = {}
    used_cells = set()
    for roi, i, m, cells, ev, j, K in proposals:
        limit = 1 + int(roi > 0.8)
        if per_tm.get((i, m), 0) >= limit:
            continue
        cell = int(cells[j])
        if (cell, m) in used_cells or st["cell_K_share"][cell] >= 0.9:
            continue
        if roi > 0.6:
            K = K * 2.0
        cost = K * (C["m_cap"][m] * pbar[i]).sum() * 1.15
        if town_pool(st, P, i) < cost or st["F_count"] >= len(st["f_alive"]):
            continue
        got = draw_pool(st, P, i, cost)
        ev["rent_bid_one"] = float(ev["rent_bid"][j])
        ev["reason"] = reason_record(C, st, i, m, cell, ev, j, K, year, "new")
        fid = create_firm(st, C, P, rng, i, m, cell, K, ev, year, got)
        if fid < 0:
            break
        used_cells.add((cell, m))
        per_tm[(i, m)] = per_tm.get((i, m), 0) + 1
        made += 1
        st["new_firms_year"].append(fid)
        if max_new is not None and made >= max_new:
            break
    return made


def review(st, C, P, rng, year, pbar, wbar):
    """Grow profitable firms, shrink idle ones, close the hopeless."""
    alive = np.flatnonzero(st["f_alive"])
    closed = []
    for fid in alive:
        i, m = st["f_town"][fid], st["f_method"][fid]
        K = st["f_K"][fid]
        capval = max(K, 1e-9) * (C["m_cap"][m] * pbar[i]).sum()
        roi = st["f_profit_acc"][fid] / max(capval, 1e-9)
        u = st["f_u"][fid]
        building = st["f_order_K"][fid] > 0
        if st["f_cash"][fid] < 0 and year - st["f_born"][fid] <= 3:
            st["f_cash"][fid] += draw_pool(st, P, i, -st["f_cash"][fid])
        if K > 0:
            st["f_K"][fid] = K * (1 - P["depreciation"])
        if roi < 0 and u < 0.15 and not building:
            st["f_loss_years"][fid] += 1
        else:
            st["f_loss_years"][fid] = 0
        insolvent = st["f_cash"][fid] < -max(6.0 * st["f_cost_m"][fid], 0.4 * capval) - 1.0
        tiny = (st["f_K"][fid] < 0.03 * C["m_K0"][m]) and not building
        stale = building and year - st["f_born"][fid] > 6 and K <= 0
        if insolvent or tiny or stale or st["f_loss_years"][fid] >= 3:
            closed.append(fid)
            continue
        if u < 0.25 and not building and K > 0:
            st["f_K"][fid] *= 0.85
        if roi > P["hurdle"] and u > 0.65 and not building and K > 0:
            dK = K * min(0.5, 0.2 * roi / P["hurdle"])
            if K + dK > P["footprint_K_mult"] * C["m_K0"][m]:
                spawn_sister(st, C, P, rng, fid, year, pbar, wbar)
            else:
                cost = dK * (C["m_cap"][m] * pbar[i]).sum()
                short = cost * 1.1 - st["f_cash"][fid]
                if short > 0:
                    st["f_cash"][fid] += draw_pool(st, P, i, short)
                if st["f_cash"][fid] > 0.5 * cost:
                    st["f_order"][fid] += dK * C["m_cap"][m]
                    st["f_order_K"][fid] += dK
    for fid in closed:
        close_firm(st, C, P, fid, year)
    st["f_profit_acc"][alive] = 0.0
    st["f_out_acc"][alive] = 0.0
    return closed


def spawn_sister(st, C, P, rng, parent, year, pbar, wbar):
    """A works at full size grows by opening a new site next door."""
    i, m = st["f_town"][parent], st["f_method"][parent]
    W = st["world"]["W"]
    y, x = divmod(int(st["f_cell"][parent]), W)
    H = st["world"]["H"]
    cand = []
    for dy in range(-2, 3):
        for dx in range(-2, 3):
            ny, nx = y + dy, x + dx
            if 0 <= ny < H and 0 <= nx < W and (dy or dx):
                c = ny * W + nx
                if st["cell_K_share"][c] < 0.7 and st["catch"][c] == i and st["use"][c] != 0:
                    cand.append(c)
    if not cand:
        return -1
    cells = np.array(cand)
    K = C["m_K0"][m]
    ev = evaluate_sites(st, C, P, i, m, cells, K, pbar, wbar)
    j = int(np.argmax(ev["roi"]))
    if ev["roi"][j] < P["hurdle"]:
        return -1
    cost = K * (C["m_cap"][m] * pbar[i]).sum() * 1.15
    fund = min(max(st["f_cash"][parent], 0.0) * 0.8, cost)
    st["f_cash"][parent] -= fund
    if fund < cost:
        fund += draw_pool(st, P, i, cost - fund)
    if fund < 0.6 * cost or st["F_count"] >= len(st["f_alive"]):
        st["hh_cash"][i] += fund
        return -1
    ev["rent_bid_one"] = float(ev["rent_bid"][j])
    ev["reason"] = reason_record(C, st, i, m, int(cells[j]), ev, j, K, year, "expansion")
    nm = st["f_name"][parent]
    base = nm.split(" No.")[0]
    count = sum(1 for n in st["f_name"] if n.startswith(base))
    fid = create_firm(st, C, P, rng, i, m, int(cells[j]), K, ev, year, fund, name=f"{base} No. {count + 1}", parent=parent)
    if fid >= 0:
        st["new_firms_year"].append(fid)
    return fid


def close_firm(st, C, P, fid, year):
    i = st["f_town"][fid]
    cash = st["f_cash"][fid]
    if cash >= 0:
        st["hh_cash"][i] += cash
    else:
        st["merch_cash"][i] += cash  # creditors take the loss
    # unfinished orders: nothing more is bought
    st["f_cash"][fid] = 0.0
    st["f_order"][fid] = 0.0
    st["f_order_K"][fid] = 0.0
    st["f_alive"][fid] = False
    st["f_died"][fid] = year
    c = st["f_cell"][fid]
    m = st["f_method"][fid]
    st["cell_K_share"][c] = max(st["cell_K_share"][c] - st["f_K"][fid] / (P["footprint_K_mult"] * C["m_K0"][m]), 0.0)
    if st["f_K"][fid] >= C["m_K0"][m] and st["cell_K_share"][c] < 0.05 and not st["urban"][c]:
        st["derelict_timer"][c] = 6
    st["closed_year"].append(fid)
