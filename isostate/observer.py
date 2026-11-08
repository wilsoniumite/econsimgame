"""The observer: turns smooth series into announced events.

The simulation never reads anything written here. Monthly series go into a
registry; once a year the detectors look for
  - crossings: a new way of doing something overtakes the old one,
  - milestones: doublings, orders of magnitude, cities passing round numbers,
  - steepness: a year whose growth is far outside the trailing trend, labelled
    by how price and quantity moved together (demand boom, glut, shortage, slump),
  - downturns on two clocks: a live alarm when output falls, and a historian's
    verdict once the recovery is complete and the episode can be dated.
"""
import numpy as np

SHARE_GROUPS = {
    # name: (kind, members, labels)
    "iron": ("method", ["bloomery", "charcoal_furnace", "coke_furnace"]),
    "cloth": ("method", ["hand_loom", "textile_mill"]),
    "steel": ("method", ["crucible_steel", "bessemer"]),
    "rails": ("method", ["forge_rails", "rolling_mill", "steel_rolling"]),
    "machinery": ("method", ["millwright", "engine_works"]),
    "warmth": ("service", ["timber", "charcoal", "coal"]),
    "heat": ("heat", ["timber", "charcoal", "coal"]),
    "clothing": ("service", ["wool", "cloth"]),
    "shelter": ("service", ["timber", "stone", "bricks"]),
    "implements": ("service", ["iron", "steel", "tools"]),
}

CROSSINGS = [
    ("iron", "coke_furnace", "Coke iron outweighs charcoal iron",
     "Coke furnaces now make {share:.0%} of the country's iron. Charcoal is dearer every year as the woods retreat; coal is not."),
    ("cloth", "textile_mill", "Mills now make most of the cloth",
     "Powered mills turn out {share:.0%} of all cloth. Handloom weavers are paid less each year for the same work."),
    ("clothing", "cloth", "Bought cloth overtakes homespun",
     "Households now get {share:.0%} of their clothing from bought cloth rather than wool spun at home."),
    ("steel", "bessemer", "Converter steel overtakes crucible steel",
     "Air-blown converters make {share:.0%} of the steel. What cost a week per pot now takes twenty minutes per charge."),
    ("rails", "rolling_mill", "Rolling mills overtake the rail forges",
     "Rolled rails are {share:.0%} of output, and the forges that hammered them out by hand are closing."),
    ("rails", "steel_rolling", "Most new rails are steel",
     "Steel rails are {share:.0%} of output. They cost a little more and last far longer."),
    ("machinery", "engine_works", "Engine works outbuild the millwrights",
     "Engine works now build {share:.0%} of all machinery. Millwrights go back to mending."),
    ("warmth", "coal", "Coal now heats more homes than wood",
     "Coal supplies {share:.0%} of household warmth. A coal fire is cheaper than a wood fire almost everywhere the roads reach."),
    ("heat", "coal", "Coal fires most of industry",
     "{share:.0%} of industrial heat now comes from coal."),
    ("shelter", "bricks", "Brick becomes the building material",
     "Bricks now provide {share:.0%} of new building and repair."),
]

MILESTONE_CITY = [10_000, 25_000, 50_000, 100_000, 200_000, 400_000]


def init(st, C, P, n_months):
    reg = {}
    meta = {}

    def add(name, label, group, unit=""):
        reg[name] = np.full(n_months, np.nan, dtype=np.float64)
        meta[name] = dict(label=label, group=group, unit=unit)

    add("pop", "Population", "people", "people")
    add("urban_share", "Living in towns", "people", "share")
    add("ag_share", "Working the land", "people", "share")
    add("ind_share", "Working in industry", "people", "share")
    add("gdp_nom", "Output (money)", "growth", "money/month")
    add("gdp_real", "Real output", "growth", "1750 money/month")
    add("gdp_pc", "Real output per head", "growth", "1750 money")
    add("cpi", "Cost of living", "prices", "1750 = 1")
    add("wage", "Average wage", "people", "money/month")
    add("real_wage", "Real wage", "people", "1750 = 1")
    add("rent_share", "Land rent share of output", "distribution", "share")
    add("invest_share", "Investment share of output", "growth", "share")
    add("lord_share_spend", "Landlords' share of spending", "distribution", "share")
    add("money", "Money stock", "money", "money")
    add("rail_km", "Railway", "transport", "km")
    add("road_km", "Roads", "transport", "km")
    add("towns", "Towns", "people", "count")
    add("firms", "Firms", "growth", "count")
    for g, name in enumerate(C["good_name"]):
        add(f"price_{name}", C["good_label"][g], "prices", "money/unit")
        add(f"out_{name}", C["good_label"][g], "output", "units/month")
    for m, name in enumerate(C["method_name"]):
        add(f"mout_{name}", C["method_label"][m], "methods", "units/month")
    for gname, (kind, members) in SHARE_GROUPS.items():
        for mem in members:
            add(f"share_{gname}_{mem}", label_of(C, kind, mem), f"share_{gname}", "share")
    st["series"] = reg
    st["series_meta"] = meta
    st["obs"] = dict(cross_state={}, milestones=set(), last_alarm={}, peak=None, in_slump=False,
                     slump_start=None, trough=None, first_output=set(), annual={})


def label_of(C, kind, mem):
    if kind == "method":
        return C["method_label"][C["method_name"].index(mem)]
    return C["good_label"][C["good_index"][mem]]


def national_prices(st):
    act = st["town_active"]
    w = st["pop"] * act
    return (st["price"] * w[:, None]).sum(axis=0) / max(w.sum(), 1.0)


def record_month(st, C, P, t):
    s = st["series"]
    ms = st["month_stats"]
    act = st["town_active"]
    pop = st["pop"] * act
    P_nat = national_prices(st)
    s["pop"][t] = pop.sum()
    s["urban_share"][t] = (st["urban_pop"] * act).sum() / max(pop.sum(), 1.0)
    emp = max(ms["employment"].sum(), 1e-9)
    ag = ms["ex_labor_by_sector"][:3].sum()
    s["ag_share"][t] = ag / emp
    s["ind_share"][t] = ms["firm_labor_by_method"].sum() / emp
    gdp = ms["hh_spend"] + ms["lord_spend"] + ms["urban_rent"] + st["m_invest"]
    s["gdp_nom"][t] = gdp
    basket = st["cpi_basket"]
    cpi = (basket * P_nat).sum() / max((basket * st["cpi_base_prices"]).sum(), 1e-9)
    s["cpi"][t] = cpi
    s["gdp_real"][t] = gdp / max(cpi, 1e-9)
    s["gdp_pc"][t] = s["gdp_real"][t] / max(pop.sum(), 1.0)
    wage = (st["wage"] * pop).sum() / max(pop.sum(), 1.0)
    s["wage"][t] = wage
    s["real_wage"][t] = wage / max(cpi, 1e-9) / st["wage_base"]
    s["rent_share"][t] = st["m_rent"] / max(gdp, 1e-9)
    s["invest_share"][t] = st["m_invest"] / max(gdp, 1e-9)
    s["lord_share_spend"][t] = ms["lord_spend"] / max(ms["hh_spend"] + ms["lord_spend"], 1e-9)
    s["money"][t] = st["money_now"]
    s["rail_km"][t] = st["rail"].sum() * 2.0
    s["road_km"][t] = (st["road"] > 1).sum() * 2.0
    s["towns"][t] = act.sum()
    s["firms"][t] = st["f_alive"].sum()
    for g, name in enumerate(C["good_name"]):
        s[f"price_{name}"][t] = P_nat[g]
        s[f"out_{name}"][t] = ms["prod"][g]
    for m, name in enumerate(C["method_name"]):
        s[f"mout_{name}"][t] = ms["method_out"][m]
    # shares
    gi = C["good_index"]
    hh = ms["hh_got"].sum(axis=0) + ms["lord_got"].sum(axis=0)
    for gname, (kind, members) in SHARE_GROUPS.items():
        if kind == "method":
            idx = [C["method_name"].index(x) for x in members]
            v = ms["method_out"][idx]
        elif kind == "service":
            sidx = C["service_name"].index(gname)
            v = np.array([hh[gi[x]] * C["props"][gi[x], sidx] for x in members])
        else:  # heat
            fuels = list(C["fuel_goods"])
            v = np.array([ms["heat_use"][fuels.index(gi[x])] for x in members])
        tot = v.sum()
        for mem, val in zip(members, v):
            s[f"share_{gname}_{mem}"][t] = val / tot if tot > 1e-9 else np.nan


def annual(series, year0, y):
    """Mean of a monthly series over calendar year y."""
    i = (y - year0) * 12
    v = series[i:i + 12]
    v = v[np.isfinite(v)]
    return v.mean() if len(v) else np.nan


def yearly(st, C, P, year, events):
    s = st["series"]
    y0 = C["start_year"]
    ob = st["obs"]
    k = year - y0

    # ---- crossings, with hysteresis and a three-month confirmation
    for group, mem, title, text in CROSSINGS:
        ser = s[f"share_{group}_{mem}"]
        i = k * 12
        recent = ser[i + 9:i + 12]
        recent = recent[np.isfinite(recent)]
        if len(recent) == 0:
            continue
        v = recent.mean()
        key = (group, mem)
        if key not in ob["cross_state"]:
            ob["cross_state"][key] = "above" if v > P["crossing_hi"] else "below"
            if v > P["crossing_hi"]:
                ob["milestones"].add(key)
            continue
        state = ob["cross_state"][key]
        if state == "below" and v > P["crossing_hi"]:
            ob["cross_state"][key] = "above"
            first = key not in ob["milestones"]
            ob["milestones"].add(key)
            events.append(dict(year=year + 0.8, kind="crossing", clock="live", weight=0.85 if first else 0.4,
                               title=title if first else title + " again", text=text.format(share=v),
                               series=f"share_{group}_{mem}"))
        elif state == "above" and v < P["crossing_lo"]:
            ob["cross_state"][key] = "below"

    # structural crossings
    ag, ind, urb = (annual(s[n], y0, year) for n in ("ag_share", "ind_share", "urban_share"))
    if np.isfinite(ind) and ind > ag and "ind>ag" not in ob["milestones"] and k > 0:
        ob["milestones"].add("ind>ag")
        events.append(dict(year=year + 0.8, kind="crossing", clock="live", weight=0.95,
                           title="More hands in workshops and mills than on the land",
                           text=f"Industry employs {ind:.0%} of workers; farms, pastures and woods {ag:.0%}.",
                           series="ind_share"))
    for thr, label in ((0.25, "A quarter of the people now live in towns"), (0.4, "Two in five people now live in towns"),
                       (0.5, "Most people now live in towns")):
        key = f"urban{thr}"
        if np.isfinite(urb) and urb > thr and key not in ob["milestones"]:
            ob["milestones"].add(key)
            if k == 0:
                continue
            events.append(dict(year=year + 0.8, kind="milestone", clock="live", weight=0.7, title=label,
                               text=f"{urb:.0%} of the population lives in towns.", series="urban_share"))

    # ---- doublings and orders of magnitude
    base = ob["annual"].setdefault("base", {})
    for name, label, kind in (("gdp_real", "Real output", 2), ("pop", "Population", 2), ("gdp_pc", "Output per head", 2),
                              ("real_wage", "The real wage", 2),
                              ("out_iron", "Iron output", 10), ("out_coal", "Coal output", 10),
                              ("out_cloth", "Cloth output", 10)):
        v = annual(s[name], y0, year)
        if not np.isfinite(v) or v <= 0:
            continue
        if name not in base:
            base[name] = (year, v)
            continue
        when, v0 = base[name]
        ratio = v / v0
        lvl = int(np.floor(np.log(ratio) / np.log(kind) + 1e-9)) if ratio >= kind else 0
        key = (name, lvl)
        if lvl >= 1 and key not in ob["milestones"]:
            ob["milestones"].add(key)
            mult = kind ** lvl
            verb = "doubles" if mult == 2 else f"is up {mult}x"
            events.append(dict(year=year + 0.7, kind="milestone", clock="live",
                               weight=0.75 if name in ("gdp_real", "gdp_pc", "real_wage") else 0.5,
                               title=f"{label} {verb} since {when}",
                               text=f"{label} is now {ratio:.1f} times its {when} level.", series=name))

    # ---- the railway network passing round numbers
    km = annual(s["rail_km"], y0, year)
    for thr in (250, 500, 1000, 2000):
        key = ("rail", thr)
        if np.isfinite(km) and km >= thr and key not in ob["milestones"]:
            ob["milestones"].add(key)
            events.append(dict(year=year + 0.8, kind="milestone", clock="live", weight=0.65,
                               title=f"The railway network passes {thr:,} km",
                               text=f"There are now {km:,.0f} km of railway in the country.", series="rail_km"))

    # ---- cities passing round numbers
    for i in np.flatnonzero(st["town_active"]):
        u = st["urban_pop"][i]
        for thr in MILESTONE_CITY:
            key = ("city", thr)
            if u >= thr and key not in ob["milestones"]:
                ob["milestones"].add(key)
                if k == 0:
                    continue
                events.append(dict(year=year + 0.75, kind="milestone", clock="live", weight=0.55 + 0.05 * MILESTONE_CITY.index(thr),
                                   title=f"{st['town_name'][i]} passes {thr:,} people",
                                   text=f"{st['town_name'][i]} is the first town in the country with more than {thr:,} inhabitants.",
                                   town=int(i), cell=int(st["town_node"][i])))

    # ---- steepness: this year's growth against the trailing decade
    steep = []
    if k >= 6:
        for gname in ("grain", "coal", "iron", "cloth", "machinery", "rails", "steel", "timber", "charcoal"):
            g = C["good_index"][gname]
            q = np.array([annual(s[f"out_{gname}"], y0, yy) for yy in range(max(y0, year - 11), year + 1)])
            p = np.array([annual(s[f"price_{gname}"], y0, yy) for yy in range(max(y0, year - 11), year + 1)])
            if not np.all(np.isfinite(q)) or q.min() <= 1e-6:
                continue
            dq = np.diff(np.log(q))
            dp = np.diff(np.log(p))
            hist, now = dq[:-1], dq[-1]
            z = (now - hist.mean()) / max(hist.std(), 0.04)
            last = ob["last_alarm"].get(gname, -99)
            if year - last < 4 or abs(z) < 2.5:
                continue
            if abs(now) < 0.12:
                continue
            share = annual(s[f"out_{gname}"], y0, year) * annual(s[f"price_{gname}"], y0, year) / max(annual(s["gdp_nom"], y0, year), 1e-9)
            up_q, up_p = now > 0, dp[-1] > 0
            label = C["good_label"][g]
            if up_q and up_p:
                title, why = f"{label} boom", "demand is outrunning supply"
            elif up_q and not up_p:
                title, why = f"{label} glut", "new capacity is flooding the market"
            elif not up_q and up_p:
                title, why = f"{label} shortage", "supply has failed while buyers still want it"
            else:
                title, why = f"{label} slump", "buyers have stopped buying"
            ob["last_alarm"][gname] = year
            steep.append((title.split()[-1], label, dict(year=year + 0.85, kind="steep", clock="live",
                               weight=float(np.clip(0.35 + 2.0 * share + 0.05 * abs(z), 0.35, 0.8)),
                               title=title,
                               text=f"{label} output {'rose' if up_q else 'fell'} {abs(np.expm1(now)):.0%} this year against a trend of {np.expm1(hist.mean()):+.0%} a year, while prices {'rose' if up_p else 'fell'} {abs(np.expm1(dp[-1])):.0%}: {why}.",
                               series=f"out_{gname}")))
    by_kind = {}
    for kind_, label, ev in steep:
        by_kind.setdefault(kind_, []).append((label, ev))
    for kind_, items in by_kind.items():
        if len(items) >= 3:
            labels = [lb for lb, _ in items]
            listing = ", ".join(labels[:-1]) + " and " + labels[-1]
            what = {"shortage": "Shortages", "glut": "Gluts", "boom": "Booms", "slump": "Slumps"}[kind_]
            why = {"shortage": "Output fell while buyers kept bidding.", "glut": "New capacity is flooding markets and prices are falling.",
                   "boom": "Demand is running ahead of supply and prices are rising.", "slump": "Buyers have pulled back; output and prices are both falling."}[kind_]
            events.append(dict(year=year + 0.85, kind="steep", clock="live", weight=0.7,
                               title=f"{what} in {len(items)} industries",
                               text=f"{listing} all moved far outside their trend this year. {why}",
                               series=items[0][1]["series"]))
        else:
            events.extend(ev for _, ev in items)

    # ---- downturns, on two clocks
    gdp = annual(s["gdp_real"], y0, year)
    prev = annual(s["gdp_real"], y0, year - 1) if k >= 1 else np.nan
    if np.isfinite(gdp) and np.isfinite(prev):
        if ob["peak"] is None or (not ob["in_slump"] and gdp > ob["peak"][1]):
            ob["peak"] = (year, gdp)
        fall = gdp / prev - 1
        if not ob["in_slump"] and fall < -0.02:
            ob["in_slump"] = True
            ob["slump_start"] = ob["peak"]
            ob["trough"] = (year, gdp)
            events.append(dict(year=year + 0.9, kind="alarm", clock="live", weight=round(min(0.8, 0.45 + 4.0 * -fall), 3),
                               title=f"Output falls {-fall:.1%}", text=f"Real output fell {-fall:.1%} this year. Nobody yet knows how long it will last.",
                               series="gdp_real"))
        elif ob["in_slump"]:
            if gdp < ob["trough"][1]:
                ob["trough"] = (year, gdp)
            if gdp > ob["slump_start"][1]:
                (py, pv), (ty, tv) = ob["slump_start"], ob["trough"]
                depth = 1 - tv / pv
                events.append(dict(year=year + 0.95, kind="verdict", clock="verdict", weight=0.75 if depth > 0.04 else 0.5,
                                   title=f"The {'depression' if depth > 0.08 else 'slump'} of {py}–{ty}",
                                   text=f"Output fell {depth:.1%} from its {py} peak to the {ty} trough and took until {year} to recover. Historians will call this a {'depression' if depth > 0.08 else 'recession'}.",
                                   span=[py + 0.5, ty + 0.5], series="gdp_real"))
                ob["in_slump"] = False
                ob["peak"] = (year, gdp)

    # ---- first production by newly discovered methods
    for fid in st["new_firms_year"]:
        m = st["f_method"][fid]
        if C["tech_known"][m] or m in ob["first_output"]:
            continue
        ob["first_output"].add(m)
        i = st["f_town"][fid]
        events.append(dict(year=year + 0.9, kind="first", clock="live", weight=0.5,
                           title=f"{st['f_name'][fid]} founded",
                           text=f"The first {C['method_label'][m].lower()} business in the country sets up near {st['town_name'][i]}.",
                           town=int(i), cell=int(st["f_cell"][fid]), firm=int(fid)))
