"""Railway companies: lines get built where existing traffic would save most.

A line is proposed between two towns once railway know-how reaches them. Its
promoters estimate the yearly carting cost it would save on current traffic
(times an induced-traffic factor), compare that with the cost of rails, timber
and stone along the cheapest buildable route, and raise money locally if the
return clears the hurdle. Rails are then bought month by month, so a burst of
projects shows up as a rail and iron boom.
"""
import numpy as np
from scipy import ndimage
from scipy.sparse.csgraph import dijkstra

from . import transport as tr


def plan(st, C, P, rng, year, pbar, events):
    tr_idx = st["t_rail"]
    act = np.flatnonzero(st["town_active"])
    k = st["know"][:, tr_idx]
    if len(act) < 2 or k[act].max() < P["rail_know"] or len(st["projects"]) >= P["rail_max_projects"]:
        return
    gi = C["good_index"]
    F = st["trade_F_year"]
    pairs = []
    for a_ in act:
        for b_ in act:
            if a_ < b_ and max(k[a_], k[b_]) >= P["rail_know"] and min(k[a_], k[b_]) > 0.05:
                if (a_, b_) in st["rail_links"] or any((pr["a"], pr["b"]) == (a_, b_) for pr in st["projects"]):
                    continue
                if st["rail_cooldown"].get((a_, b_), -99) > year:
                    continue
                flow = F[a_, b_] + F[b_, a_]
                if flow > 0:
                    pairs.append((flow, a_, b_))
    if not pairs:
        return
    pairs.sort(reverse=True)
    rg = tr.rail_build_graph(st["world"], st["rail"])
    for flow, a_, b_ in pairs[:30]:
        if len(st["projects"]) >= P["rail_max_projects"]:
            break
        na, nb = st["town_node"][a_], st["town_node"][b_]
        dist, pred = dijkstra(rg, directed=True, indices=na, return_predecessors=True)
        if not np.isfinite(dist[nb]) or dist[nb] > 1e5:
            continue
        path = tr.path_cells(pred.astype(np.int64), na, nb)
        if len(path) < 2:
            continue
        new_cells = path[st["rail"][path] == 0]
        n_new = len(new_cells)
        if n_new == 0:
            st["rail_links"].add((a_, b_))
            continue
        H, W = st["world"]["H"], st["world"]["W"]
        ys, xs = np.divmod(path, W)
        length = np.hypot(np.diff(ys), np.diff(xs)).sum()
        cost_now = st["town_cost"][a_, b_]
        cost_rail = tr.RAIL_COST * length + 1.0
        saving_per_w = P["freight"] * max(cost_now - cost_rail, 0.0)
        benefit = P["rail_induced"] * flow * saving_per_w
        need = np.zeros(C["G"])
        need[gi["rails"]] = P["rail_rails_per_cell"] * n_new
        need[gi["timber"]] = P["rail_timber_per_cell"] * n_new
        need[gi["stone"]] = P["rail_stone_per_cell"] * n_new
        buy_town = a_ if pbar[a_, gi["rails"]] <= pbar[b_, gi["rails"]] else b_
        cost = (need * pbar[buy_town]).sum() * 1.3
        roi = benefit / max(cost, 1e-9)
        if roi < P["rail_hurdle"]:
            st["rail_cooldown"][(a_, b_)] = year + 4
            continue
        # promoters in both towns subscribe
        raised = 0.0
        for t in (a_, b_):
            for key, frac in (("lord_cash", 0.35), ("hh_cash", 0.08)):
                take = min(max(st[key][t], 0.0) * frac, cost - raised)
                if take > 0:
                    st[key][t] -= take
                    raised += take
        if raised < 0.5 * cost:
            for t in (a_, b_):
                st["hh_cash"][t] += raised / 2.0
            st["rail_cooldown"][(a_, b_)] = year + 2
            continue
        pr = dict(a=int(a_), b=int(b_), town=int(buy_town), cells=path, new=new_cells, need=need,
                  order=need.copy(), cash=raised, start=year, roi=roi, cost=cost, length_km=2.0 * length)
        st["projects"].append(pr)
        events.append(dict(
            year=year + 0.95, kind="rail_plan", clock="live", weight=0.45,
            title=f"{st['town_name'][a_]} and {st['town_name'][b_]} Railway promoted",
            text=f"Promoters raise money for {2.0 * length:.0f} km of line. They expect the traffic now carted between the towns to repay {roi:.0%} a year.",
            town=int(a_), cell=int(na),
        ))


def plan_spurs(st, C, P, rng, year, pbar, events):
    """Mineral and factory lines: from a town out to a busy cluster of pits or works."""
    tr_idx = st["t_rail"]
    if len(st["projects"]) >= P["rail_max_projects"]:
        return
    H, W = st["world"]["H"], st["world"]["W"]
    gi = C["good_index"]
    flow = ndimage.uniform_filter(st["cell_flow_year"].reshape(H, W), size=5, mode="constant").ravel() * 25.0
    railed = (st["rail"] > 0).reshape(H, W)
    if not railed.any():
        return
    d_rail = ndimage.distance_transform_edt(~railed).ravel()
    rg = None
    for c in np.argsort(-flow)[:25]:
        if len(st["projects"]) >= P["rail_max_projects"]:
            break
        i = st["catch"][c]
        if i < 0 or d_rail[c] < 4 or flow[c] < P["spur_min_flow"] or st["know"][i, tr_idx] < P["rail_know"]:
            continue
        if any(abs(c - d) % W <= 3 and abs(c // W - d // W) <= 3 for d in st["spur_done"]):
            continue
        if st["rail"][st["town_node"][i]] == 0:
            continue
        if rg is None:
            rg = tr.rail_build_graph(st["world"], st["rail"])
        src = st["town_node"][i]
        dist, pred = dijkstra(rg, directed=True, indices=src, return_predecessors=True)
        if not np.isfinite(dist[c]) or dist[c] > 1e5:
            continue
        path = tr.path_cells(pred.astype(np.int64), src, c)
        new_cells = path[st["rail"][path] == 0]
        if len(new_cells) < 2:
            st["spur_done"].add(int(c))
            continue
        ys, xs = np.divmod(path, W)
        length = np.hypot(np.diff(ys), np.diff(xs)).sum()
        saving = P["freight"] * max(st["dcatch"][c] - (tr.RAIL_COST * length + 0.5), 0.0)
        benefit = P["rail_induced"] * flow[c] * saving
        need = np.zeros(C["G"])
        n_new = len(new_cells)
        need[gi["rails"]] = P["rail_rails_per_cell"] * n_new
        need[gi["timber"]] = P["rail_timber_per_cell"] * n_new
        need[gi["stone"]] = P["rail_stone_per_cell"] * n_new
        cost = (need * pbar[i]).sum() * 1.3
        roi = benefit / max(cost, 1e-9)
        st["spur_done"].add(int(c))
        if roi < P["rail_hurdle"]:
            continue
        raised = 0.0
        for key, frac in (("lord_cash", 0.3), ("hh_cash", 0.06)):
            take = min(max(st[key][i], 0.0) * frac, cost - raised)
            if take > 0:
                st[key][i] -= take
                raised += take
        if raised < 0.5 * cost:
            st["hh_cash"][i] += raised
            continue
        st["projects"].append(dict(a=int(i), b=int(i), kind="spur", target=int(c), town=int(i), cells=path,
                                   new=new_cells, need=need, order=need.copy(), cash=raised, start=year, roi=roi,
                                   cost=cost, length_km=2.0 * length))


def progress(st, C, P, year_float, events):
    """Open finished lines; promoters top up projects that run short of money."""
    keep = []
    for pr in st["projects"]:
        if pr["cash"] < 0:
            for t in (pr["a"], pr["b"]):
                top = min(max(st["lord_cash"][t], 0.0) * 0.1, -pr["cash"])
                st["lord_cash"][t] -= top
                pr["cash"] += top
        done = pr["order"].sum() <= 0.01 * pr["need"].sum()
        if done:
            st["rail"][pr["cells"]] = 1
            st["rail_links"].add((pr["a"], pr["b"]))
            st["hh_cash"][pr["town"]] += pr["cash"]
            pr["cash"] = 0.0
            a_, b_ = pr["a"], pr["b"]
            if pr.get("kind") == "spur":
                st["rail_opened"].append((year_float, a_, -1, pr["length_km"]))
                what = {7: "iron mines", 8: "collieries", 9: "works"}.get(int(st["use"][pr["target"]]), "pits and works")
                events.append(dict(
                    year=year_float, kind="railway", clock="live", weight=0.35,
                    title=f"Mineral line opens near {st['town_name'][a_]}",
                    text=f"{pr['length_km']:.0f} km of branch line now carries the output of the {what} into {st['town_name'][a_]}.",
                    town=int(a_), cell=int(pr["target"]),
                ))
                continue
            first = len(st["rail_opened"]) == 0
            st["rail_opened"].append((year_float, a_, b_, pr["length_km"]))
            events.append(dict(
                year=year_float, kind="railway", clock="live", weight=1.0 if first else 0.45,
                title=("The first railway opens: " if first else "Railway opens: ") + f"{st['town_name'][a_]} to {st['town_name'][b_]}",
                text=f"{pr['length_km']:.0f} km of track, begun in {pr['start']}. Freight between the towns now moves at a fraction of the carting cost.",
                town=int(a_), cell=int(st["town_node"][a_]), cell2=int(st["town_node"][b_]),
            ))
        else:
            keep.append(pr)
    st["projects"] = keep
