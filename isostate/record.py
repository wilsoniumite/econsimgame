"""Record a run as yearly snapshots and pack it for the viewer.

The export is one binary container: a JSON header (names, events, firm
histories, array layout) followed by raw typed arrays, gzipped together and
embedded in the viewer HTML as base64. Map layers are stored layer-major so
consecutive years of the same layer sit next to each other and compress well.
"""
import base64
import gzip
import json
import time
from pathlib import Path

import numpy as np

from .land import USE_INDUSTRY, USE_NAMES, USE_SECTOR0, USE_URBAN, USE_WILD
from .observer import SHARE_GROUPS

LAYERS = ["use", "inten", "rent", "trans", "catch", "dist", "mix_a", "mix_b", "mix_s"]

# Display classes for the map. Each cell is drawn as a patchwork of its two
# largest classes, so mixed land shows as mixed land instead of one colour.
DISP = ["sea", "farm", "pasture", "wood", "heath", "quarry", "claypit", "ironmine", "coalmine", "industry", "urban", "derelict"]
_SECTOR_DISP = {"farm": 1, "pasture": 2, "forestry": 3, "quarry": 5, "claypit": 6, "ironmine": 7, "coalmine": 8}
_EXTRACTIVE = {"quarry", "claypit", "ironmine", "coalmine"}


def display_mix(st, C):
    """Top two display classes per cell and the share of the first.

    Quarries and mines take land in the model, but most of it is rough ground
    around the workings, so only part of their share is drawn as diggings.
    Wild land shows as wood or heath depending on its tree cover.
    """
    N, NS = st["N"], C["NS"]
    lam = st["lam"]
    ks = np.clip(st["cell_K_share"], 0.0, 1.0)
    tot = lam.sum(axis=0)
    scale = np.where(tot > 0, (1.0 - ks) / np.maximum(tot, 1e-12), 0.0)
    D = np.zeros((len(DISP), N))
    for s, name in enumerate(C["sector_name"]):
        share = lam[s] * scale
        if name in _EXTRACTIVE:
            D[_SECTOR_DISP[name]] += 0.45 * share
            D[4] += 0.55 * share
        else:
            D[_SECTOR_DISP[name]] += share
    pot = np.maximum(st["world"]["forest_pot"].ravel(), 1e-6)
    woody = np.clip((st["cover"] / pot - 0.3) / 0.4, 0.0, 1.0)
    wild = lam[NS] * scale + np.where(tot > 0, 0.0, 1.0 - ks)
    D[3] += wild * woody
    D[4] += wild * (1 - woody)
    D[9] += ks
    land = st["world"]["land"].ravel()
    urban = st["urban"].astype(bool)
    der = st["derelict_timer"] > 0
    D[:, urban] = 0.0
    D[10, urban] = 1.0
    D[:, der & ~urban] = 0.0
    D[11, der & ~urban] = 1.0
    D[:, ~land] = 0.0
    D[0, ~land] = 1.0
    order = np.argsort(-D, axis=0)
    a, b = order[0], order[1]
    da, db = D[a, np.arange(N)], D[b, np.arange(N)]
    sh = np.where(db > 0.02, da / np.maximum(da + db, 1e-12), 1.0)
    b = np.where(db > 0.02, b, a)
    return a.astype(np.uint8), b.astype(np.uint8), np.round(sh * 255).astype(np.uint8)


def init(st, C, P, n_years):
    N, T = st["N"], st["T_max"]
    rec = dict(n_years=n_years)
    for name in LAYERS:
        rec[name] = np.zeros((n_years, N), dtype=np.uint8)
    rec["town_active"] = np.zeros((n_years, T), dtype=np.uint8)
    for name in ("town_pop", "town_urban", "town_wage", "town_y"):
        rec[name] = np.zeros((n_years, T), dtype=np.float32)
    rec["town_price"] = np.zeros((n_years, T, C["G"]), dtype=np.float32)
    rec["town_know"] = np.zeros((n_years, T, C["NT"]), dtype=np.uint8)
    rec["firm_K"] = []
    rec["projects"] = []
    return rec


def snapshot(rec, st, C, P, k):
    use = st["use"].astype(np.int64)
    inten = np.zeros(st["N"])
    inten[:] = np.clip(st["cell_labor"] / 45.0, 0, 1)
    ind = use == USE_INDUSTRY
    inten[ind] = np.clip(st["cell_K_share"][ind], 0.05, 1)
    urb = use == USE_URBAN
    inten[urb] = np.clip(st["urban_res"][urb] / max(st["urban_density"], 1.0), 0.05, 1)
    wild = use == USE_WILD
    inten[wild] = st["cover"][wild]
    rec["use"][k] = use.astype(np.uint8)
    rec["mix_a"][k], rec["mix_b"][k], rec["mix_s"][k] = display_mix(st, C)
    rec["inten"][k] = np.round(inten * 255).astype(np.uint8)
    lr = np.log10(np.maximum(st["cell_rent"], 1e-3))
    rec["rent"][k] = np.round(np.clip((lr + 3) / 6.0, 0, 1) * 255).astype(np.uint8) * (st["cell_rent"] > 1e-3)
    trans = np.minimum(st["road"], 3) + 4 * (st["rail"] > 0)
    for pr in st["projects"]:
        trans[pr["new"]] |= 8
    rec["trans"][k] = trans.astype(np.uint8)
    rec["catch"][k] = (st["catch"] + 1).astype(np.uint8)
    rec["dist"][k] = np.round(np.clip(np.where(np.isfinite(st["dcatch"]), st["dcatch"], 63.75) * 4, 0, 255)).astype(np.uint8)
    act = st["town_active"]
    rec["town_active"][k] = act
    rec["town_pop"][k] = st["pop"]
    rec["town_urban"][k] = st["urban_pop"] * act
    rec["town_wage"][k] = st["wbar"]
    rec["town_y"][k] = st["y_ratio"]
    rec["town_price"][k] = st["pbar"]
    rec["town_know"][k] = np.round(np.clip(st["know"], 0, 1) * 255).astype(np.uint8)
    alive = np.flatnonzero(st["f_alive"])
    rec["firm_K"].append((alive.copy(), st["f_K"][alive].astype(np.float32)))
    rec["projects"].append([dict(a=pr["a"], b=pr["b"], done=float(1 - pr["order"].sum() / max(pr["need"].sum(), 1e-9)))
                            for pr in st["projects"]])


def _clean(o):
    if isinstance(o, dict):
        return {k: _clean(v) for k, v in o.items()}
    if isinstance(o, (list, tuple)):
        return [_clean(v) for v in o]
    if isinstance(o, (np.integer,)):
        return int(o)
    if isinstance(o, (np.floating,)):
        v = float(o)
        return None if not np.isfinite(v) else round(v, 5)
    if isinstance(o, float):
        return None if not np.isfinite(o) else round(o, 5)
    if isinstance(o, np.ndarray):
        return _clean(o.tolist())
    return o


def pack(result, label="", lite=False):
    """Binary container for one run: JSON header + aligned typed arrays.

    lite=True keeps series, events and town data but drops map layers and
    firm histories; used for a comparison scenario shown in charts only.
    """
    st, C, P, rec = result["state"], result["content"], result["params"], result["rec"]
    world = st["world"]
    y0 = C["start_year"]
    n_years = rec["n_years"]
    F = st["F_count"]
    arrays = []

    def add(name, arr):
        arrays.append((name, np.ascontiguousarray(arr)))

    if lite:
        F = 0
    add("elev", np.round(world["h"] * 255).astype(np.uint8))
    add("terrain", world["terrain"].astype(np.uint8))
    racc = np.where(world["river"], 1 + np.clip(np.log2(np.maximum(world["acc"], 1.0)) * 20, 0, 253), 0).astype(np.uint8)
    add("river", racc)
    add("nav", world["navigable"].astype(np.uint8))
    add("down", world["down"].astype(np.int32))
    for f in ("fertility", "ore", "coal", "rock", "clay", "pasture", "forest_pot"):
        add(f, np.round(np.clip(world[f], 0, 1) * 255).astype(np.uint8))
    if not lite:
        for name in LAYERS:
            add(name, rec[name])
    add("town_active", rec["town_active"])
    for name in ("town_pop", "town_urban", "town_wage", "town_y"):
        add(name, rec[name])
    add("town_price", rec["town_price"])
    add("town_know", rec["town_know"])
    fk = np.zeros((F, n_years), dtype=np.float32)
    for k, (ids, K) in enumerate(rec["firm_K"] if not lite else []):
        fk[ids, k] = K
    # store each firm's capacity only over the years it existed
    first = np.full(F, n_years, dtype=np.int32)
    last = np.full(F, -1, dtype=np.int32)
    nz = fk > 0
    anyk = nz.any(axis=1)
    first[anyk] = nz[anyk].argmax(axis=1)
    last[anyk] = n_years - 1 - nz[anyk][:, ::-1].argmax(axis=1)
    lens = np.where(anyk, last - first + 1, 0).astype(np.int32)
    offs = np.concatenate([[0], np.cumsum(lens)[:-1]]).astype(np.int32)
    vals = np.concatenate([fk[f, first[f]:last[f] + 1] for f in range(F) if lens[f] > 0]) if lens.sum() else np.zeros(0, np.float32)
    add("firm_first", first)
    add("firm_len", lens)
    add("firm_off", offs)
    add("firm_Kvals", vals.astype(np.float32))
    names = list(st["series"].keys())
    ser = np.stack([st["series"][n] for n in names]).astype(np.float32)
    add("series", ser)

    towns = [dict(name=st["town_name"][i], node=int(st["town_node"][i]), founded=int(st["town_founded"][i]))
             for i in range(len(st["town_name"]))]
    firms = [dict(name=st["f_name"][f], method=int(st["f_method"][f]), cell=int(st["f_cell"][f]),
                  town=int(st["f_town"][f]), born=int(st["f_born"][f]),
                  died=int(st["f_died"][f]) if not st["f_alive"][f] else -1,
                  parent=int(st["f_parent"][f]), reason=st["f_reason"][f]) for f in range(F)]
    header = dict(
        meta=dict(label=label, lite=lite, seed=P["seed"], W=int(world["W"]), H=int(world["H"]), start_year=y0,
                  end_year=C["end_year"], n_years=n_years, rent_regime=P["rent_regime"],
                  generated=time.strftime("%Y-%m-%d"), freight=P["freight"], km_per_cell=2.0),
        goods=[dict(name=n, label=C["good_label"][g], color=C["good_color"][g], weight=float(C["weight"][g]))
               for g, n in enumerate(C["good_name"])],
        methods=[dict(name=n, label=C["method_label"][m], out=int(C["method_out"][m]),
                      inputs=[[int(g), float(C["m_in"][m][g])] for g in np.flatnonzero(C["m_in"][m])],
                      heat=float(C["m_heat"][m]), labor=float(C["m_labor"][m]), K0=float(C["m_K0"][m]))
                 for m, n in enumerate(C["method_name"])],
        techs=[dict(name=n, label=C["tech_label"][t], kind=C["tech_kind"][t], known=bool(C["tech_known"][t]),
                    year=int(st["tech_year"][t]), town=int(st["tech_town"][t]))
               for t, n in enumerate(C["tech_name"])],
        sectors=[dict(name=n, label=C["sector_label"][s], good=int(C["sector_good"][s])) for s, n in enumerate(C["sector_name"])],
        uses=USE_NAMES,
        disp=DISP,
        towns=towns,
        firms=firms,
        events=result["events"],
        rail_opened=[dict(year=y, a=a, b=b, km=km) for y, a, b, km in st["rail_opened"]],
        projects=rec["projects"],
        series=dict(names=names, meta=st["series_meta"], start_year=y0),
        share_groups={k: dict(kind=v[0], members=v[1]) for k, v in SHARE_GROUPS.items()},
        arrays=[],
    )
    offset = 0
    for name, arr in arrays:
        nbytes = arr.nbytes
        header["arrays"].append(dict(name=name, dtype=str(arr.dtype), shape=list(arr.shape), offset=offset, nbytes=nbytes))
        offset += nbytes + (-nbytes) % 8
    hjson = json.dumps(_clean(header), separators=(",", ":")).encode()
    pad = (-(4 + len(hjson))) % 8
    parts = [np.uint32(len(hjson) + pad).tobytes(), hjson, b" " * pad]
    for name, arr in arrays:
        b = arr.tobytes()
        parts.append(b)
        parts.append(b"\0" * ((-len(b)) % 8))
    return b"".join(parts)


def build_html(blobs, template=Path(__file__).resolve().parent.parent / "viewer" / "index.html", out=None):
    """Embed one or more packed runs into the viewer template."""
    html = Path(template).read_text()
    tags = []
    for key, blob in blobs:
        b64 = base64.b64encode(gzip.compress(blob, compresslevel=9)).decode()
        tags.append(f'<script type="application/octet-stream" class="run-data" data-key="{key}">{b64}</script>')
    html = html.replace("<!--RUN_DATA-->", "\n".join(tags))
    if out:
        Path(out).write_text(html)
    return html
