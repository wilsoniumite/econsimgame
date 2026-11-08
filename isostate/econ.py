"""The monthly economy: wages, production, consumption, markets, trade, money.

Every payment is a transfer between two accounts (households, landlords,
merchants, firms, the land fund, rail projects), so total money only changes
through explicit issuance. `money_total` checks that invariant.
"""
import numpy as np


def sigmoid(x):
    return 1.0 / (1.0 + np.exp(-np.clip(x, -50, 50)))


def service_mix(price, props, theta):
    """Logit mix of goods for each household service.

    Returns share (T, G, S): fraction of a service bought from each good, and
    unit (T, S): average price of one unit of service.
    """
    with np.errstate(divide="ignore", invalid="ignore"):
        s = price[:, :, None] / np.where(props > 0, props, np.nan)[None]
    s = np.where(np.isfinite(s), s, np.inf)
    smin = s.min(axis=1, keepdims=True)
    z = np.where(np.isfinite(s), np.exp(-(s / smin - 1.0) / theta), 0.0)
    share = z / z.sum(axis=1, keepdims=True)
    unit = (share * np.where(np.isfinite(s), s, 0.0)).sum(axis=1)
    return share, unit


def heat_mix(fuel_price, heatval, theta):
    s = fuel_price / heatval[None, :]
    smin = s.min(axis=1, keepdims=True)
    z = np.exp(-(s / smin - 1.0) / theta)
    share = z / z.sum(axis=1, keepdims=True)
    return share, (share * s).sum(axis=1)


def les(budget_pc, unit, gamma, beta):
    """Linear expenditure system over services with a subsistence floor."""
    sub = (gamma[None, :] * unit).sum(axis=1)
    extra = np.maximum(budget_pc - sub, 0.0)
    scale = np.where(budget_pc < sub, budget_pc / np.maximum(sub, 1e-12), 1.0)
    q = gamma[None, :] * scale[:, None] + beta[None, :] * extra[:, None] / np.maximum(unit, 1e-12)
    return q, sub


def goods_from_services(q, share, props):
    inv_prop = np.where(props > 0, 1.0 / np.where(props > 0, props, 1.0), 0.0)
    return np.einsum("ts,tgs,gs->tg", q, share, inv_prop)


def firm_coefficients(C, method, K, prod):
    """Per-unit inputs, heat and labor; small firms and new methods are worse."""
    K0 = C["m_K0"][method]
    r = np.minimum(np.sqrt(K0 / np.maximum(K, 1e-9)), 10.0)
    a = C["m_in"][method] * ((1 + 0.1 * r) / prod ** 0.35)[:, None]
    h = C["m_heat"][method] * (1 + 0.2 * r) / prod ** 0.5
    l = C["m_labor"][method] * (1 + 0.8 * r) / prod
    return a, h, l


def sector_tfp(st, C, P, sec, town, price, fill):
    """Productivity multiplier and per-unit machine inputs for land sectors.

    Tools make every land worker better (households' implements), improved
    husbandry lifts farms and pastures, steam pumps and iron wagonways lift
    mines, and farm machinery arrives late. Machine inputs are bought per unit
    of output; the boost scales with how much of them the market could supply.
    """
    gm, gr = C["good_index"]["machinery"], C["good_index"]["rails"]
    name = np.array(C["sector_name"])[sec]
    mult = st["tools_mult"][town].copy()
    k = st["know"]
    ag = (name == "farm") | (name == "pasture")
    mult = np.where(ag, mult * (1 + P["husbandry_boost"] * k[town, st["t_husb"]]), mult)
    mine = C["sector_mine"][sec]
    kp, kw, kf = k[town, st["t_pump"]], k[town, st["t_wagon"]], k[town, st["t_fmach"]]
    use_pump = mine & (kp > 0.05)
    use_wagon = mine & (kw > 0.05)
    use_fm = (name == "farm") & (kf > 0.05)
    mult = mult * (1.0 + use_pump * P["pump_boost"] * kp * fill[town, gm]
                   + use_wagon * P["wagon_boost"] * kw * fill[town, gr]
                   + use_fm * P["farm_mach_boost"] * kf * fill[town, gm])
    mach = use_pump * P["pump_input"] + use_fm * P["farm_mach_input"]
    rails = use_wagon * P["wagon_input"]
    unit_cost = mach * price[town, gm] + rails * price[town, gr]
    return mult, mach, rails, unit_cost


def pay_rent(st, P, town, amount):
    """Land rent goes to the landlords of a town, or to the common fund."""
    if P["rent_regime"] == "commons":
        st["fund_cash"] += amount.sum()
    else:
        st["lord_income"] += amount
    st["m_rent"] += amount.sum()


def money_total(st):
    alive = st["f_alive"] | (st["f_cash"] != 0)
    return (st["hh_cash"].sum() + st["lord_cash"].sum() + st["merch_cash"].sum()
            + st["f_cash"][alive].sum() + st["fund_cash"] + sum(pr["cash"] for pr in st["projects"])
            + st["hh_income"].sum() + st["lord_income"].sum())


def step_month(st, C, P):
    T, G = st["T_max"], C["G"]
    gi = C["good_index"]
    p, inv, pop = st["price"], st["inv"], st["pop"]
    act = st["town_active"]
    st["m_rent"] = 0.0

    # ---------------- households: what they want to buy
    share, unit = service_mix(p, C["props"], P["logit_theta"])
    hh_cash = np.maximum(st["hh_cash"], 0.0)
    budget = np.minimum(hh_cash, P["cons_income"] * np.maximum(st["hh_income_prev"], 0.0) + P["cons_wealth"] * hh_cash)
    urban_pay = np.minimum(st["urban_rent"], 0.5 * budget)
    goods_budget = budget - urban_pay
    pc = np.where(pop > 0, goods_budget / np.maximum(pop, 1.0), 0.0)
    q_hh, subcost = les(pc, unit, C["subsistence"], C["shares"])
    d_hh = goods_from_services(q_hh, share, C["props"]) * pop[:, None]

    lord_pop = P["lord_share"] * pop
    lord_cash = np.maximum(st["lord_cash"], 0.0)
    lbudget = np.minimum(lord_cash, P["lord_cons_income"] * np.maximum(st["lord_income_prev"], 0.0) + P["lord_cons_wealth"] * lord_cash)
    lpc = np.where(lord_pop > 0, lbudget / np.maximum(lord_pop, 1e-9), 0.0)
    q_l, _ = les(lpc, unit, C["subsistence"], C["lord_shares"])
    d_lord = goods_from_services(q_l, share, C["props"]) * lord_pop[:, None]

    # ---------------- land-based production: cell enterprises hire at the town wage
    ex, sec, et = st["ex_cell"], st["ex_sector"], st["ex_town"]
    g_s = C["sector_good"][sec]
    alpha = C["sector_alpha"][sec]
    f = np.where(sec == st["forestry_sector"], st["cover"][ex], st["ex_field"])
    gm, gr = gi["machinery"], gi["rails"]
    pe = st["pexp"]
    mult, mach_unit, rail_unit, in_unit = sector_tfp(st, C, P, sec, et, pe, st["fill"])
    A = C["sector_A"][sec] * mult
    npr = pe[et, g_s] - st["ex_tau"] - in_unit
    Cc = np.where((npr > 0) & (f > 0), (1 - alpha) * A * f * npr, 0.0)
    lam = st["ex_lam"]

    # ---------------- firms: unit economics at their own location
    fi = np.flatnonzero(st["f_alive"] & (st["f_K"] > 1e-9))
    ft, fm, fc = st["f_town"][fi], st["f_method"][fi], st["f_cell"][fi]
    K = st["f_K"][fi]
    a, h, l = firm_coefficients(C, fm, K, st["f_prod"][fi])
    out = C["method_out"][fm]
    pin = np.maximum(p[ft] + st["delta"][:, fc].T, 0.25 * p[ft])
    fuels = C["fuel_goods"]
    hshare, hunit = heat_mix(pin[:, fuels], C["heat"][fuels], P["heat_theta"])
    pout = p[ft, out] - P["freight"] * C["weight"][out] * st["dcatch"][fc]
    nonlabor = (a * pin).sum(axis=1) + h * hunit
    phi_prev = st["f_fill"][fi]

    # ---------------- labour market: one wage per town clears supply and demand
    supply = P["participation"] * pop
    # Short-run labour demand moves only part of the way to the long-run optimum,
    # so a price blip does not empty the fields in a month.
    prev_w = st["wage"]
    has = prev_w > 0
    safe_w = np.where(has, prev_w, 1.0)
    lo = np.where(has, np.log(np.maximum(safe_w / 2.0, 0.02)), np.log(0.02))
    hi = np.where(has, np.log(np.minimum(safe_w * 2.0, 500.0)), np.log(500.0))
    inv_alpha = 1.0 / alpha
    kx, kf = P["labor_adjust_ex"], P["labor_adjust_firm"]
    Lprev_ex = st["ex_Lprev"]
    Lprev_f = st["f_labor"][fi]

    def demand(w):
        Lx = Lprev_ex + kx * (lam * (Cc / w[et]) ** inv_alpha - Lprev_ex)
        m = pout - nonlabor - l * w[ft]
        u = sigmoid((m / np.maximum(pout, 1e-9) - P["util_target_margin"]) / P["util_band"]) * (pout > 0)
        Lf = Lprev_f + kf * (K * u * l * phi_prev - Lprev_f)
        return np.maximum(Lx, 0.0), np.maximum(Lf, 0.0), u

    for _ in range(40):
        w = np.exp(0.5 * (lo + hi))
        Lx, Lf, _ = demand(w)
        D = np.bincount(et, weights=Lx, minlength=T) + np.bincount(ft, weights=Lf, minlength=T)
        too_much = D > supply
        mid = 0.5 * (lo + hi)
        lo = np.where(too_much, mid, lo)
        hi = np.where(too_much, hi, mid)
    w = np.exp(0.5 * (lo + hi))
    w = np.where(act, w, 0.0)
    Lex, Lf, u = demand(np.maximum(w, 1e-9))
    st["ex_Lprev"] = Lex
    Y = A * f * lam ** alpha * Lex ** (1 - alpha)
    q_des = np.where(l > 0, Lf / np.maximum(l * np.maximum(phi_prev, 0.05), 1e-12), 0.0)
    q_des = np.minimum(q_des, K)

    # ---------------- requests to each town market
    R = d_hh + d_lord
    req_f = q_des[:, None] * a
    heat_need = q_des * h
    req_f[:, fuels] += heat_need[:, None] * hshare / C["heat"][fuels][None, :]
    for g in range(G):
        R[:, g] += np.bincount(ft, weights=req_f[:, g], minlength=T)
    mine_mach = Y * mach_unit
    mine_rail = Y * rail_unit
    R[:, gm] += np.bincount(et, weights=mine_mach, minlength=T)
    R[:, gr] += np.bincount(et, weights=mine_rail, minlength=T)
    oi = np.flatnonzero(st["f_alive"] & (st["f_order_K"] > 0))
    ot = st["f_town"][oi]
    req_inv = st["f_order"][oi] * 0.25
    for g in range(G):
        R[:, g] += np.bincount(ot, weights=req_inv[:, g], minlength=T)
    proj_req = []
    for pr in st["projects"]:
        rq = np.minimum(pr["order"], pr["need"] / 10.0)
        proj_req.append(rq)
        R[pr["town"]] += rq

    fill = np.where(R > 1e-12, np.minimum(1.0, inv / np.maximum(R, 1e-12)), 1.0)

    # ---------------- execute: firms produce what their scarcest input allows
    fill_t = fill[ft]
    fill_in = np.where(a > 0, fill_t, 1.0).min(axis=1)
    fill_heat = np.where(h > 0, (hshare * fill_t[:, fuels]).sum(axis=1), 1.0)
    phi = np.minimum(fill_in, fill_heat)
    q = q_des * phi
    use_f = q[:, None] * a
    use_f[:, fuels] += (q * h)[:, None] * hshare / C["heat"][fuels][None, :]
    labor_f = Lf * np.where(phi_prev > 0, np.minimum(phi / np.maximum(phi_prev, 0.05), 1.0), 0.0)
    labor_f = np.minimum(labor_f, q * l * 1.5 + 1e-9)
    st["f_fill"][fi] = 0.7 * phi_prev + 0.3 * phi
    st["f_u"][fi] = 0.9 * st["f_u"][fi] + 0.1 * u * phi

    hh_got = d_hh * fill
    lord_got = d_lord * fill
    mach_got = mine_mach * fill[et, gm]
    rail_got = mine_rail * fill[et, gr]
    inv_got = req_inv * fill[ot]
    sold = hh_got + lord_got
    for g in range(G):
        sold[:, g] += np.bincount(ft, weights=use_f[:, g], minlength=T)
        sold[:, g] += np.bincount(ot, weights=inv_got[:, g], minlength=T)
    sold[:, gm] += np.bincount(et, weights=mach_got, minlength=T)
    sold[:, gr] += np.bincount(et, weights=rail_got, minlength=T)
    proj_got = []
    for pr, rq in zip(st["projects"], proj_req):
        got = rq * fill[pr["town"]]
        proj_got.append(got)
        sold[pr["town"]] += got

    # ---------------- money flows
    merch = np.zeros(T)
    hh_inc = np.zeros(T)
    sales_value = np.zeros(T)

    # firms
    pay_in = (use_f * pin).sum(axis=1)
    cart_in = (use_f * np.maximum(pin - p[ft], 0.0)).sum(axis=1)
    spread = P["merchant_spread"]
    rev = q * (pout - spread * p[ft, out])
    cart_out = q * (p[ft, out] - pout)
    wages_f = labor_f * w[ft]
    rent_f = st["f_rent"][fi]
    st["f_cash"][fi] += rev - pay_in - wages_f - rent_f
    merch += np.bincount(ft, weights=pay_in - cart_in - rev - cart_out, minlength=T)
    st["m_merch_margin"] = (q * spread * p[ft, out]).sum()
    sales_value += np.bincount(ft, weights=pay_in, minlength=T)
    hh_inc += np.bincount(ft, weights=cart_in + cart_out + wages_f, minlength=T)
    st["f_profit_acc"][fi] += rev - pay_in - wages_f - rent_f
    st["f_out_acc"][fi] += q
    st["f_cost_m"][fi] = pay_in + wages_f + rent_f
    pay_rent(st, P, None, np.bincount(ft, weights=rent_f, minlength=T))

    # land-based enterprises
    value = p[et, g_s] * Y
    in_paid = mach_got * p[et, gm] + rail_got * p[et, gr]
    wage_ex = w[et] * Lex
    cart_ex = st["ex_tau"] * Y
    rent_ex = value * (1 - spread) - cart_ex - wage_ex - in_paid
    merch += np.bincount(et, weights=in_paid - value * (1 - spread), minlength=T)
    sales_value += np.bincount(et, weights=in_paid, minlength=T)
    hh_inc += np.bincount(et, weights=wage_ex + cart_ex, minlength=T)
    pay_rent(st, P, None, np.bincount(et, weights=rent_ex, minlength=T))

    # households and landlords buy goods; households pay urban rent
    hh_spend = (hh_got * p).sum(axis=1)
    lord_spend = (lord_got * p).sum(axis=1)
    st["hh_cash"] -= hh_spend + urban_pay
    st["lord_cash"] -= lord_spend
    merch += hh_spend + lord_spend
    sales_value += hh_spend + lord_spend
    pay_rent(st, P, None, urban_pay)

    # investment deliveries: firms pay for capital goods, capacity arrives pro rata
    if len(oi):
        inv_pay = (inv_got * p[ot]).sum(axis=1)
        remaining_value = (st["f_order"][oi] * p[ot]).sum(axis=1)
        frac = np.where(remaining_value > 1e-12, inv_pay / np.maximum(remaining_value, 1e-12), 1.0)
        addK = st["f_order_K"][oi] * np.clip(frac, 0, 1)
        st["f_K"][oi] += addK
        st["f_order_K"][oi] -= addK
        st["f_order"][oi] -= inv_got
        st["f_order"][oi] = np.maximum(st["f_order"][oi], 0.0)
        done = (st["f_order"][oi].sum(axis=1) < 1e-6) | (st["f_order_K"][oi] < 1e-9)
        st["f_order_K"][oi[done]] = 0.0
        st["f_order"][oi[done]] = 0.0
        st["f_cash"][oi] -= inv_pay
        merch += np.bincount(ot, weights=inv_pay, minlength=T)
        sales_value += np.bincount(ot, weights=inv_pay, minlength=T)
        st["m_invest"] = inv_pay.sum()
    else:
        st["m_invest"] = 0.0

    proj_spend = 0.0
    for pr, got in zip(st["projects"], proj_got):
        pay = (got * p[pr["town"]]).sum()
        wages = 0.3 * pay
        pr["cash"] -= pay + wages
        pr["order"] = np.maximum(pr["order"] - got, 0.0)
        merch[pr["town"]] += pay
        sales_value[pr["town"]] += pay
        hh_inc[pr["town"]] += wages
        proj_spend += pay + wages
    st["m_invest"] += proj_spend

    # ---------------- inventories: sales out, production in
    prod_TG = np.zeros((T, G))
    for g in np.unique(g_s):
        sel = g_s == g
        prod_TG[:, g] += np.bincount(et[sel], weights=Y[sel], minlength=T)
    for g in np.unique(out):
        sel = out == g
        prod_TG[:, g] += np.bincount(ft[sel], weights=q[sel], minlength=T)
    inv -= sold
    inv *= 1.0 - np.asarray(P["decay"])[None, :]
    inv += prod_TG
    np.maximum(inv, 0.0, out=inv)

    # forests are depleted by harvesting
    fsel = sec == st["forestry_sector"]
    if fsel.any():
        cells = ex[fsel]
        st["cover"][cells] = np.maximum(st["cover"][cells] - Y[fsel] / (P["forest_capacity"] * np.maximum(lam[fsel], 0.05)), 0.0)

    # ---------------- trade between towns
    exports = np.zeros((T, G))
    imports = np.zeros((T, G))
    A_idx = np.flatnonzero(act)
    if len(A_idx) > 1:
        pa, ia = p[A_idx], inv[A_idx]
        c = st["town_cost"][np.ix_(A_idx, A_idx)]
        c = np.where(np.isfinite(c), c, 1e9)
        tau = P["freight"] * C["weight"][None, None, :] * c[:, :, None] + P["handling"] * pa[:, None, :]
        gap = pa[None, :, :] - pa[:, None, :] - tau
        frac = np.clip(gap / np.maximum(pa[None, :, :], 1e-9), 0.0, 0.5) * P["trade_k"]
        idx = np.arange(len(A_idx))
        frac[idx, idx, :] = 0.0
        want = frac * ia[:, None, :]
        tot = want.sum(axis=1)
        cap = P["trade_cap"] * ia
        scale = np.where(tot > cap, cap / np.maximum(tot, 1e-12), 1.0)
        qt = want * scale[:, None, :]
        ex_q = qt.sum(axis=1)
        im_q = qt.sum(axis=0)
        inv[A_idx] += im_q - ex_q
        exports[A_idx] = ex_q
        imports[A_idx] = im_q
        pay = (qt * pa[:, None, :]).sum(axis=2)
        tcost = (qt * tau).sum(axis=2)
        merch[A_idx] += pay.sum(axis=1) - pay.sum(axis=0) - tcost.sum(axis=0)
        hh_inc[A_idx] += tcost.sum(axis=1)
        wflow = (qt * C["weight"][None, None, :]).sum(axis=2)
        st["trade_F"][np.ix_(A_idx, A_idx)] += wflow
        st["m_trade_value"] = pay.sum()
    else:
        st["m_trade_value"] = 0.0
    np.maximum(inv, 0.0, out=inv)

    # ---------------- prices respond to inventories (level) and to this month's
    # imbalance between wanted and arriving goods (flow), which damps cycles
    outflow = sold + exports
    st["outflow"] = 0.8 * st["outflow"] + 0.2 * outflow
    target = P["price_target_months"] * st["outflow"]
    gap = (target - inv) / np.maximum(np.maximum(target, inv), 1e-9)
    arriving = prod_TG + imports
    imbalance = (R - arriving) / np.maximum(R + arriving, 1e-9)
    imbalance = np.where(R + arriving > 1e-9, imbalance, 0.0)
    newp = p * np.exp(np.clip(P["price_speed"] * gap + P["price_flow"] * imbalance, -0.12, 0.12))
    lo_p = P["price_floor"] * C["base_price"][None, :]
    hi_p = P["price_ceiling"] * C["base_price"][None, :]
    p[:] = np.where(act[:, None], np.clip(newp, lo_p, hi_p), p)
    st["pexp"] = (1 - P["expect_rate"]) * st["pexp"] + P["expect_rate"] * p

    # ---------------- dividends, common fund, issuance
    st["merch_cash"] += merch
    fdiv = np.zeros(T)
    if len(fi):
        excess = st["f_cash"][fi] - P["firm_buffer_months"] * st["f_cost_m"][fi]
        div = P["firm_div_rate"] * np.maximum(excess, 0.0)
        st["f_cash"][fi] -= div
        fdiv = np.bincount(ft, weights=div, minlength=T)
    mexcess = st["merch_cash"] - P["merch_buffer_months"] * sales_value
    mdiv = np.where(act, P["merch_div_rate"] * np.maximum(mexcess, 0.0), 0.0)
    st["merch_cash"] -= mdiv
    popshare = pop / max(pop.sum(), 1.0)
    fund_div = 0.9 * st["fund_cash"]
    st["fund_cash"] -= fund_div
    m_tot = money_total(st)
    issue = P["money_growth"] / 12.0 * m_tot
    st["issued"] += issue
    hh_inc += fdiv + mdiv + (fund_div + issue) * popshare

    st["hh_income"] += hh_inc
    st["hh_cash"] += st["hh_income"]
    st["lord_cash"] += st["lord_income"]
    st["hh_income_prev"] = st["hh_income"].copy()
    st["lord_income_prev"] = st["lord_income"].copy()
    st["hh_income"][:] = 0.0
    st["lord_income"][:] = 0.0

    # ---------------- bookkeeping for the year and for observers
    serv_deliv = np.einsum("tg,gs->ts", hh_got, C["props"]) / np.maximum(pop, 1.0)[:, None]
    st["imp_pc"] = 0.9 * st["imp_pc"] + 0.1 * serv_deliv[:, 4]
    st["tools_mult"] = 1.0 + P["tools_boost"] * (1.0 - np.exp(-st["imp_pc"] / P["tools_scale"]))
    st["fill"] = fill
    st["wage"] = w
    st["subcost"] = subcost
    st["y_acc"] += np.where(subcost > 0, pc / np.maximum(subcost, 1e-12), 0.0)
    st["food_ratio"] = serv_deliv[:, 0] / C["subsistence"][0]
    st["food_min_acc"] = np.minimum(st["food_min_acc"], np.where(act, st["food_ratio"], 9.0))
    st["price_acc"] += p
    st["wage_acc"] += w
    st["town_prod"] += prod_TG
    st["cumprod"] += prod_TG.sum(axis=0)
    mo = np.bincount(fm, weights=q, minlength=C["M"])
    st["method_Q"] += mo
    st["ex_labor"] = Lex
    st["ex_out"] = Y
    st["f_labor"][:] = 0.0
    st["f_labor"][fi] = labor_f
    flow = np.zeros(st["N"])
    np.add.at(flow, ex, Y * C["weight"][g_s])
    np.add.at(flow, fc, (use_f * C["weight"][None, :]).sum(axis=1) + q * C["weight"][out])
    st["cell_flow"] += flow

    st["month_stats"] = dict(
        prod=prod_TG.sum(axis=0), method_out=mo, hh_got=hh_got, lord_got=lord_got, share=share,
        hh_spend=hh_spend.sum(), lord_spend=lord_spend.sum(), urban_rent=urban_pay.sum(),
        employment=np.bincount(et, weights=Lex, minlength=T) + np.bincount(ft, weights=labor_f, minlength=T),
        ex_labor_by_sector=np.bincount(sec, weights=Lex, minlength=C["NS"]),
        firm_labor_by_method=np.bincount(fm, weights=labor_f, minlength=C["M"]),
        heat_use=((q * h)[:, None] * hshare).sum(axis=0) if len(fi) else np.zeros(len(fuels)),
        supply=supply,
    )
