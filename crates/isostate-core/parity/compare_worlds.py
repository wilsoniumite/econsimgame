"""Island statistics of Python and Rust runs, from the static layers of run.bin.

    python3 crates/isostate-core/parity/compare_worlds.py --seeds 1 2 3 4 7

The two implementations draw different random numbers, so the islands differ;
this checks that they are drawn from the same distribution (land fraction,
relief, rivers, coast, resources, towns).
"""
import argparse
import json
from pathlib import Path

import numpy as np


def load(path):
    blob = Path(path).read_bytes()
    hlen = int(np.frombuffer(blob[:4], np.uint32)[0])
    h = json.loads(blob[4:4 + hlen])
    base = 4 + hlen
    arrays = {}
    for a in h["arrays"]:
        dt = np.dtype(a["dtype"])
        arrays[a["name"]] = np.frombuffer(blob, dtype=dt, count=a["nbytes"] // dt.itemsize,
                                          offset=base + a["offset"]).reshape(a["shape"])
    return h, arrays


def stats(path):
    h, A = load(path)
    terr = A["terrain"].ravel()
    land = terr > 0
    out = dict(
        land=land.mean(),
        hills=(terr == 2).sum() / land.sum(),
        mountains=(terr == 3).sum() / land.sum(),
        river=(A["river"].ravel() > 0).sum(),
        nav=(A["nav"].ravel() > 0).sum(),
        fert=A["fertility"].ravel()[land].mean() / 255,
        coal_cells=(A["coal"].ravel() > 0.2 * 255).sum(),
        ore_cells=(A["ore"].ravel() > 0.2 * 255).sum(),
        forest=A["forest_pot"].ravel()[land].mean() / 255,
        towns_1750=int(A["town_active"][0].sum()),
        towns_1900=int(A["town_active"][-1].sum()),
        dist_1800=float(np.mean(A["dist"][50].ravel()[land]) / 4 * 2),
    )
    # coast: land cells next to sea (8-neighbour)
    t = terr.reshape(A["terrain"].shape)
    sea = t == 0
    pad = np.pad(sea, 1, constant_values=True)
    near = np.zeros_like(sea)
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            near |= pad[1 + dy:1 + dy + t.shape[0], 1 + dx:1 + dx + t.shape[1]]
    out["coast"] = int((near & ~sea).sum())
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--py", default="/tmp/py_s{seed}/run.bin")
    ap.add_argument("--rs", default="/tmp/rs_s{seed}/run.bin")
    ap.add_argument("--seeds", type=int, nargs="+", default=[1, 2, 3, 4, 7])
    a = ap.parse_args()
    rows = {"py": [], "rs": []}
    for seed in a.seeds:
        for impl, pat in (("py", a.py), ("rs", a.rs)):
            p = pat.format(seed=seed)
            if Path(p).exists():
                rows[impl].append((seed, stats(p)))
    keys = list(rows["py"][0][1])
    print("| seed | impl | " + " | ".join(keys) + " |")
    print("|---|---|" + "---|" * len(keys))
    for impl in ("py", "rs"):
        for seed, s in rows[impl]:
            print(f"| {seed} | {impl} | " + " | ".join(f"{s[k]:.3g}" for k in keys) + " |")
    for impl in ("py", "rs"):
        m = {k: np.mean([s[k] for _, s in rows[impl]]) for k in keys}
        print(f"| mean | {impl} | " + " | ".join(f"{m[k]:.3g}" for k in keys) + " |")


if __name__ == "__main__":
    main()
