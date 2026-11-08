"""Set up a world and run it. `run(params)` is the whole history in one call."""
import numpy as np
from tqdm.auto import tqdm

from . import econ, firms, land, observer, people, rail, record, tech
from . import transport as tr
from .content import load_content, tech_index
from .names import town_name
from .world import generate_world, site_context

RNG_STREAMS = ["world", "names", "firms", "tech", "people", "rail", "misc"]


def make_rngs(seed):
    ss = np.random.SeedSequence(seed)
    return {k: np.random.Generator(np.random.PCG64(s)) for k, s in zip(RNG_STREAMS, ss.spawn(len(RNG_STREAMS)))}


def init_state(P, C=None):
    C = C or load_content()
    rng = make_rngs(P["seed"])
    world = generate_world(rng["world"], P["W"], P["H"], P["n_towns"])
    W, H = world["W"], world["H"]
    N, T, G, Fm = W * H, P["T_max"], C["G"], P["F_max"]
    st = dict(world=world, N=N, T_max=T)

    # towns
    st["town_active"] = np.zeros(T, dtype=bool)
    st["town_node"] = np.zeros(T, dtype=np.int64)
    st["town_founded"] = np.full(T, -1, dtype=np.int64)
    st["town_new"] = np.zeros(T, dtype=bool)
    st["town_name"] = []
    st["used_names"] = set()
    for k, c in enumerate(world["town_sites"]):
        st["town_active"][k] = True
        st["town_node"][k] = c
        st["town_name"].append(town_name(rng["names"], site_context(world, c), st["used_names"]))
        st["town_founded"][k] = C["start_year"] - 1
    yy, xx = np.mgrid[0:H, 0:W]
    wts = []
    for c in world["town_sites"]:
        ty, tx = divmod(c, W)
        wts.append((world["fertility"] * (np.hypot(yy - ty, xx - tx) < 12)).sum())
    wts = np.array(wts)
    st["pop"] = np.zeros(T)
    st["pop"][:len(wts)] = P["pop0"] * wts / wts.sum()

    # cells
    land_mask = world["land"].ravel()
    st["use"] = np.where(land_mask, land.USE_WILD, land.USE_SEA).astype(np.int8)
    st["cover"] = world["forest0"].ravel().copy()
    st["road"] = np.zeros(N, dtype=np.int64)
    st["rail"] = np.zeros(N, dtype=np.int64)
    st["road_quiet"] = np.zeros(N, dtype=np.int64)
    st["firm_at"] = np.full(N, -1, dtype=np.int64)
    st["cell_K_share"] = np.zeros(N)
    st["land_rent_only"] = np.zeros(N)
    st["derelict_timer"] = np.zeros(N, dtype=np.int64)
    st["cell_rent"] = np.zeros(N)
    st["cell_flow"] = np.zeros(N)
    st["traffic"] = np.zeros(N)
    st["urban"] = np.zeros(N, dtype=bool)
    st["urban_res"] = np.zeros(N)
    st["water"] = (world["coast"] | world["river"]).ravel()

    # firms
    st["F_count"] = 0
    st["f_alive"] = np.zeros(Fm, dtype=bool)
    for key, dt in (("f_method", np.int64), ("f_cell", np.int64), ("f_town", np.int64), ("f_born", np.int64),
                    ("f_died", np.int64), ("f_loss_years", np.int64), ("f_parent", np.int64)):
        st[key] = np.zeros(Fm, dtype=dt)
    for key in ("f_K", "f_order_K", "f_cash", "f_rent", "f_u", "f_fill", "f_prod", "f_profit_acc",
                "f_out_acc", "f_cost_m", "f_labor"):
        st[key] = np.zeros(Fm)
    st["f_order"] = np.zeros((Fm, G))
    st["f_name"], st["f_reason"] = [], []
    st["new_firms_year"], st["closed_year"] = [], []

    # money and markets
    pop = st["pop"]
    st["hh_cash"] = 6.0 * pop
    st["lord_cash"] = 6.0 * pop
    st["merch_cash"] = 2.0 * pop
    for key in ("hh_income", "lord_income", "hh_income_prev", "lord_income_prev"):
        st[key] = np.zeros(T)
    st["hh_income_prev"] = 1.8 * pop
    st["lord_income_prev"] = 0.4 * pop
    st["fund_cash"] = 0.0
    st["issued"] = 0.0
    st["price"] = np.tile(C["base_price"], (T, 1))
    st["pexp"] = st["price"].copy()
    st["lam"] = np.zeros((C["NS"] + 1, N))
    st["ex_lam"] = np.zeros(0)
    guess = np.array([1.3, 0.06, 0.45, 0.06, 0.03, 0.02, 0.03, 0.04, 0.01, 0.0, 0.006, 0.02, 0.02, 0.0, 0.0, 0.05])
    st["inv"] = 2.0 * pop[:, None] * guess[None, :]
    st["outflow"] = pop[:, None] * guess[None, :]
    st["fill"] = np.ones((T, G))
    st["wage"] = np.where(st["town_active"], 2.5, 0.0)
    st["tools_mult"] = np.full(T, 1.2)
    st["imp_pc"] = np.full(T, 0.006)
    st["urban_rent"] = np.zeros(T)
    st["urban_pop"] = 0.1 * pop
    st["ag_labor"] = 0.7 * P["participation"] * pop
    st["ind_labor"] = 0.08 * P["participation"] * pop
    st["y_ratio"] = np.ones(T)

    # knowledge
    st["know"] = np.tile(C["tech_known"].astype(float), (T, 1))
    st["tech_year"] = np.where(C["tech_known"], C["start_year"] - 100, -1)
    st["tech_town"] = np.full(C["NT"], -1)
    st["t_pump"] = tech_index(C, "steam_pumping")
    st["t_wagon"] = tech_index(C, "wagonway")
    st["t_rail"] = tech_index(C, "railway")
    st["t_husb"] = tech_index(C, "improved_husbandry")
    st["t_fmach"] = tech_index(C, "farm_machinery")
    st["forestry_sector"] = C["sector_name"].index("forestry")
    st["method_Q"] = C["m_Q0"] * 0.0
    st["method_A"] = np.ones(C["M"])

    # accumulators
    for key, shape in (("y_acc", T), ("price_acc", (T, G)), ("wage_acc", T), ("town_prod", (T, G)),
                       ("town_prod_year", (T, G)), ("cumprod", G), ("trade_F", (T, T)), ("trade_F_year", (T, T))):
        st[key] = np.zeros(shape)
    st["food_min_acc"] = np.full(T, 9.0)
    st["projects"] = []
    st["rail_links"] = set()
    st["rail_cooldown"] = {}
    st["spur_done"] = set()
    st["cell_flow_year"] = np.zeros(N)
    st["rail_opened"] = []
    st["m_rent"] = st["m_invest"] = 0.0
    st["ex_cell"] = np.zeros(0, dtype=np.int64)
    st["ex_sector"] = np.zeros(0, dtype=np.int64)
    st["ex_town"] = np.zeros(0, dtype=np.int64)
    st["ex_labor"] = np.zeros(0)
    st["ex_Lprev"] = np.zeros(0)

    # first roads: tracks between each town and its two nearest neighbours
    land.update_transport(st, C, P)
    act = np.flatnonzero(st["town_active"])
    for i in act:
        costs = st["town_cost"][i, act]
        for j in act[np.argsort(costs)[1:3]]:
            path = tr.path_cells(st["pred"][i], st["town_node"][i], st["town_node"][j])
            st["road"][path[land_mask[path]]] = np.maximum(st["road"][path[land_mask[path]]], 1)
    land.update_transport(st, C, P)
    after_transport(st, C, P)

    pbar = st["price"].copy()
    wbar = st["wage"].copy()
    firms.update_productivity(st, C, P)
    land.update_land_use(st, C, P, C["start_year"], pbar, wbar, initial=True)
    land.delta_fields(st, C, P)
    seed_firms(st, C, P, rng, pbar, wbar)
    return st, C, rng


def after_transport(st, C, P):
    """Bookkeeping that depends on catchments."""
    alive = np.flatnonzero(st["f_alive"])
    ct = st["catch"][st["f_cell"][alive]]
    st["f_town"][alive] = np.where(ct >= 0, ct, st["f_town"][alive])
    st["town_cells"] = [np.flatnonzero(st["catch"] == i) for i in range(st["T_max"])]
    H, W = st["world"]["H"], st["world"]["W"]
    from scipy import ndimage
    cnt = np.zeros(st["N"])
    np.add.at(cnt, st["f_cell"][alive], 1.0)
    st["any_counts"] = ndimage.uniform_filter(cnt.reshape(H, W), size=5, mode="constant").ravel() * 25.0
    foot = st["f_K"][alive] / (P["footprint_K_mult"] * C["m_K0"][st["f_method"][alive]])
    ks = np.zeros(st["N"])
    np.add.at(ks, st["f_cell"][alive], foot)
    st["cell_K_share"] = np.minimum(ks, 1.0)
    main = np.full(st["N"], -1, dtype=np.int64)
    best = np.zeros(st["N"])
    order = alive[np.argsort(foot)]
    main[st["f_cell"][order]] = order
    st["firm_at"] = main
    st["cell_firms"] = cnt


def seed_firms(st, C, P, rng, pbar, wbar):
    """The artisans of 1750: enough smiths, weavers, colliers and furnaces for local demand."""
    gi = C["good_index"]
    share, unit = econ.service_mix(pbar, C["props"], P["logit_theta"])
    q, _ = econ.les(np.full(st["T_max"], 2.1), unit, C["subsistence"], C["shares"])
    d = econ.goods_from_services(q, share, C["props"]) * st["pop"][:, None]
    need = d.copy()
    need[:, gi["iron"]] += 0.55 * need[:, gi["tools"]]
    need[:, gi["charcoal"]] += 2.3 * need[:, gi["iron"]]
    need[:, gi["bricks"]] += 0.03 * st["pop"]
    plan = ["tools", "cloth", "iron", "charcoal", "bricks", "services"]
    for i in np.flatnonzero(st["town_active"]):
        for gname in plan:
            g = gi[gname]
            target = need[i, g] * 1.25
            meths = [m for m in range(C["M"]) if C["method_out"][m] == g and C["tech_known"][m]]
            best = None
            for m in meths:
                K = min(target, 2.0 * C["m_K0"][m])
                cells = firms.candidate_cells(st, C, P, i, m, rng["firms"])
                if len(cells) == 0:
                    continue
                ev = firms.evaluate_sites(st, C, P, i, m, cells, K, pbar, wbar)
                j = int(np.argmax(ev["roi"]))
                if best is None or ev["roi"][j] > best[0]:
                    best = (ev["roi"][j], m, cells, ev, j)
            if best is None:
                continue
            _, m, cells, ev, j = best
            n_firms = int(np.clip(np.ceil(target / (2.0 * C["m_K0"][m])), 1, 6))
            K_each = target / n_firms
            for _ in range(n_firms):
                cells = firms.candidate_cells(st, C, P, i, m, rng["firms"])
                if len(cells) == 0:
                    break
                ev = firms.evaluate_sites(st, C, P, i, m, cells, K_each, pbar, wbar)
                j = int(np.argmax(ev["roi"]))
                ev["rent_bid_one"] = float(ev["rent_bid"][j])
                ev["reason"] = firms.reason_record(C, st, i, m, int(cells[j]), ev, j, K_each, C["start_year"] - 1, "old")
                cost_m = K_each * pbar[i, C["method_out"][m]]
                fid = firms.create_firm(st, C, P, rng["firms"], i, m, int(cells[j]), K_each, ev, C["start_year"] - 1, 6.0 * cost_m)
                st["f_K"][fid] = K_each
                st["f_order"][fid] = 0.0
                st["f_order_K"][fid] = 0.0
                st["f_u"][fid] = 0.8
            st["same_counts"] = firms.same_method_counts(st, C)
    after_transport(st, C, P)
    land.delta_fields(st, C, P)


def update_roads(st, C, P):
    """Traffic wears tracks; busy tracks become roads, roads become turnpikes."""
    N = st["N"]
    traffic = np.zeros(N)
    land_mask = st["world"]["land"].ravel()
    for i in np.flatnonzero(st["town_active"]):
        fl = np.where(st["catch"] == i, st["cell_flow"], 0.0)
        if fl.sum() <= 0:
            continue
        reach = np.flatnonzero(np.isfinite(st["dist"][i]))
        order = reach[np.argsort(-st["dist"][i][reach])]
        traffic = tr.tree_traffic(st["pred"][i], order, fl, traffic)
    nodes = st["town_node"].copy()
    F = np.where(np.isfinite(st["town_cost"]), st["trade_F_year"], 0.0)
    traffic = tr.pair_traffic(st["pred"], nodes, F, traffic)
    traffic *= land_mask
    t1, t2, t3 = P["road_thresholds"]
    lvl = (traffic >= t1).astype(int) + (traffic >= t2) + (traffic >= t3)
    up = lvl > st["road"]
    st["road"] = np.where(up, lvl, st["road"])
    thr = np.array([0.0, t1, t2, t3])
    quiet = (st["road"] > 0) & (traffic < 0.2 * thr[st["road"]])
    st["road_quiet"] = np.where(quiet, st["road_quiet"] + 1, 0)
    down = st["road_quiet"] >= 6
    st["road"] = np.where(down, st["road"] - 1, st["road"])
    st["road_quiet"] = np.where(down, 0, st["road_quiet"])
    st["traffic"] = traffic
    st["cell_flow"][:] = 0.0


def yearly(st, C, P, rng, year, events, warm=False):
    pbar = st["price_acc"] / 12.0
    wbar = st["wage_acc"] / 12.0
    st["pbar"], st["wbar"] = pbar, wbar
    st["price_acc"][:] = 0.0
    st["wage_acc"][:] = 0.0
    st["trade_F_year"] = st["trade_F"].copy()
    st["trade_F"][:] = 0.0
    st["town_prod_year"] = st["town_prod"].copy()
    st["town_prod"][:] = 0.0
    is_ag = np.isin(st["ex_sector"], [0, 1, 2])
    st["ag_labor"] = np.bincount(st["ex_town"], weights=st["ex_labor"] * is_ag, minlength=st["T_max"])
    st["ind_labor"] = np.bincount(st["f_town"], weights=st["f_labor"] * st["f_alive"], minlength=st["T_max"])
    st["cell_labor"] = np.zeros(st["N"])
    np.add.at(st["cell_labor"], st["ex_cell"], st["ex_labor"])

    founded = []
    if not warm:
        people.growth(st, C, P, year, events)
        people.migration(st, C, P)
        tech.discovery(st, C, P, rng["tech"], year, events)
        founded = people.found_towns(st, C, P, rng["people"], year, events)
    else:
        st["y_ratio"] = st["y_acc"] / 12.0
        st["y_acc"][:] = 0.0
        st["food_min_acc"][:] = 9.0
    tech.diffusion(st, C, P)
    st["cell_flow_year"] = st["cell_flow"].copy()
    update_roads(st, C, P)
    st["prev_catch"] = st["catch"].copy()
    land.update_transport(st, C, P)
    if founded:
        people.resettle_new_towns(st, C, P)
    after_transport(st, C, P)
    firms.update_productivity(st, C, P)
    land.update_land_use(st, C, P, year, pbar, wbar)
    land.delta_fields(st, C, P)
    st["new_firms_year"], st["closed_year"] = [], []
    closed = firms.review(st, C, P, rng["firms"], year, pbar, wbar)
    firms.entry(st, C, P, rng["firms"], year, pbar, wbar)
    after_transport(st, C, P)
    if not warm:
        rail.plan(st, C, P, rng["rail"], year, pbar, events)
        rail.plan_spurs(st, C, P, rng["rail"], year, pbar, events)
        notable_closures(st, C, P, closed, year, events)


def notable_closures(st, C, P, closed, year, events):
    big = [f for f in closed if st["f_K"][f] > 1.5 * C["m_K0"][st["f_method"][f]]]
    for fid in big[:2]:
        i = st["f_town"][fid]
        events.append(dict(year=year + 0.9, kind="closure", clock="live", weight=0.4,
                           title=f"{st['f_name'][fid]} closes",
                           text=f"The {C['method_label'][st['f_method'][fid]].lower()} near {st['town_name'][i]}, open since {st['f_born'][fid]}, can no longer cover its costs.",
                           town=int(i), cell=int(st["f_cell"][fid]), firm=int(fid)))


def run(P, progress=True, check_money=True):
    st, C, rng = init_state(P)
    y0 = C["start_year"]
    y1 = P.get("end_year", C["end_year"])
    C["end_year"] = y1
    events = []
    # warm-up: the 1740s settle into an equilibrium nobody records
    for wy in range(P["warmup_years"]):
        for _ in range(12):
            econ.step_month(st, C, P)
        yearly(st, C, P, rng, y0 - P["warmup_years"] + wy, [], warm=True)
    st["cumprod_1750"] = st["cumprod"].copy()
    rate = st["cumprod"] / max(P["warmup_years"], 1)
    disc = C["tech_disc"].copy()
    rel = disc < 0
    disc[rel] = -disc[rel] * np.broadcast_to(rate, disc.shape)[rel]
    disc[rel & (disc <= 0)] = 1e18
    st["disc_X0"] = disc
    st["cumprod"][:] = 0.0
    ms = st["month_stats"]
    st["cpi_basket"] = ms["hh_got"].sum(axis=0) + ms["lord_got"].sum(axis=0)
    st["cpi_base_prices"] = observer.national_prices(st)
    w = st["wage"] * st["pop"] * st["town_active"]
    st["wage_base"] = w.sum() / max((st["pop"] * st["town_active"]).sum(), 1.0)
    n_months = (y1 - y0 + 1) * 12
    observer.init(st, C, P, n_months)
    rec = record.init(st, C, P, y1 - y0 + 1)
    st["money_start"] = econ.money_total(st)
    st["issued"] = 0.0
    t = 0
    years = range(y0, y1 + 1)
    for year in tqdm(years, disable=not progress, desc="years"):
        for month in range(12):
            econ.step_month(st, C, P)
            rail.progress(st, C, P, year + (month + 0.5) / 12.0, events)
            st["money_now"] = econ.money_total(st)
            if check_money:
                drift = st["money_now"] - (st["money_start"] + st["issued"])
                assert abs(drift) < 1e-6 * max(st["money_now"], 1.0), f"money not conserved: drift {drift:.6g} in {year}-{month}"
            observer.record_month(st, C, P, t)
            t += 1
        yearly(st, C, P, rng, year, events)
        observer.yearly(st, C, P, year, events)
        record.snapshot(rec, st, C, P, year - y0)
    events.sort(key=lambda e: e["year"])
    return dict(state=st, content=C, params=P, events=events, rec=rec)
