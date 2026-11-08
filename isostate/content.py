"""Load content (goods, services, sectors, methods, techs) into flat arrays.

Everything downstream works with integer indices and numpy arrays, so this is
the only place that knows about names.
"""
import tomllib
from pathlib import Path

import numpy as np

CONTENT_PATH = Path(__file__).resolve().parent.parent / "data" / "content.toml"


def load_content(path=CONTENT_PATH):
    raw = tomllib.loads(Path(path).read_text())
    goods = raw["meta"]["goods"]
    gname = [g["name"] for g in goods]
    gi = {n: i for i, n in enumerate(gname)}
    G = len(goods)

    svc = raw["services"]
    sname = svc["names"]
    S = len(sname)
    props = np.zeros((G, S))
    for g, d in svc["props"].items():
        for s, v in d.items():
            props[gi[g], sname.index(s)] = v

    heat = np.zeros(G)
    for g, v in raw["heat"].items():
        heat[gi[g]] = v

    sectors = raw["sector"]
    methods = raw["method"]
    techs_extra = raw.get("tech", [])
    M = len(methods)

    m_in = np.zeros((M, G))
    m_cap = np.zeros((M, G))
    for m, d in enumerate(methods):
        for g, v in d.get("inputs", {}).items():
            m_in[m, gi[g]] = v
        for g, v in d.get("capital", {}).items():
            m_cap[m, gi[g]] = v

    # Knowledge items: every firm method, then the extra techs.
    tech_list = [dict(d, kind="method") for d in methods] + [dict(d, kind="tech") for d in techs_extra]
    NT = len(tech_list)
    t_known = np.array([bool(d.get("known", False)) for d in tech_list])
    # cumulative production thresholds (counted from the start year), 0 = not needed;
    # negative values mean "N years of start-year output" and are resolved after warm-up
    t_disc = np.zeros((NT, G))
    for t, d in enumerate(tech_list):
        for g, v in d.get("discover", {}).items():
            t_disc[t, gi[g]] = -float(v[1:]) if isinstance(v, str) and v.startswith("x") else float(v)

    return dict(
        start_year=raw["meta"]["start_year"],
        end_year=raw["meta"]["end_year"],
        G=G,
        good_name=gname,
        good_label=[g["label"] for g in goods],
        good_color=[g["color"] for g in goods],
        good_index=gi,
        weight=np.array([g["weight"] for g in goods], dtype=float),
        base_price=np.array([g["price"] for g in goods], dtype=float),
        S=S,
        service_name=sname,
        service_label=svc["labels"],
        subsistence=np.array(svc["subsistence"], dtype=float),
        shares=np.array(svc["shares"], dtype=float),
        lord_shares=np.array(svc.get("lord_shares", svc["shares"]), dtype=float),
        props=props,
        heat=heat,
        fuel_goods=np.flatnonzero(heat > 0),
        NS=len(sectors),
        sector_name=[s["name"] for s in sectors],
        sector_label=[s["label"] for s in sectors],
        sector_good=np.array([gi[s["good"]] for s in sectors]),
        sector_alpha=np.array([s["alpha"] for s in sectors], dtype=float),
        sector_A=np.array([s["A"] for s in sectors], dtype=float),
        sector_field=[s["field"] for s in sectors],
        sector_mine=np.array([bool(s.get("mine", False)) for s in sectors]),
        M=M,
        method_name=[d["name"] for d in methods],
        method_label=[d["label"] for d in methods],
        method_out=np.array([gi[d["output"]] for d in methods]),
        m_in=m_in,
        m_heat=np.array([d.get("heat", 0.0) for d in methods], dtype=float),
        m_labor=np.array([d["labor"] for d in methods], dtype=float),
        m_cap=m_cap,
        m_K0=np.array([d["K0"] for d in methods], dtype=float),
        m_A0=np.array([d.get("A0", 1.0) for d in methods], dtype=float),
        m_Q0=np.array([d.get("Q0", 1e12) for d in methods], dtype=float),
        m_b=np.array([d.get("b", 0.0) for d in methods], dtype=float),
        NT=NT,
        tech_name=[d["name"] for d in tech_list],
        tech_label=[d["label"] for d in tech_list],
        tech_kind=[d["kind"] for d in tech_list],
        tech_known=t_known,
        tech_disc=t_disc,
        tech_headline=[d.get("headline", "") for d in tech_list],
        tech_story=[d.get("story", "") for d in tech_list],
    )


def tech_index(content, name):
    return content["tech_name"].index(name)


def method_index(content, name):
    return content["method_name"].index(name)
