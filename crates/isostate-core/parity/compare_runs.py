"""Side-by-side table of Python and Rust runs (statistical parity).

    python3 crates/isostate-core/parity/compare_runs.py --py /tmp/py_s{seed} --rs /tmp/rs_s{seed} --seeds 1 2 3 4

Each directory is the output of `run` (series_monthly.csv, events.json, run.bin).
Annual means of the monthly series at 1800, 1850 and 1900, discovery years and
the first railway, then the mean over seeds of each implementation.
The two implementations draw different random numbers, so single runs differ;
the comparison is of levels and shapes.
"""
import argparse
import json
from pathlib import Path

import numpy as np

COLS = ["pop", "real_wage", "gdp_pc", "urban_share", "rail_km", "out_iron", "out_cloth"]
YEARS = [1800, 1850, 1900]
TECHS = ["coke_furnace", "textile_mill", "engine_works", "crucible_steel", "bessemer",
         "rolling_mill", "steel_rolling", "railway", "improved_husbandry", "farm_machinery"]


def annual(d):
    rows = np.genfromtxt(Path(d) / "series_monthly.csv", delimiter=",", names=True, dtype=float)
    years = rows["year"].astype(int)
    out = {}
    for c in COLS:
        out[c] = {y: float(np.nanmean(rows[c][years == y])) for y in YEARS}
    return out


def header(d):
    blob = (Path(d) / "run.bin").read_bytes()
    hlen = int(np.frombuffer(blob[:4], np.uint32)[0])
    return json.loads(blob[4:4 + hlen])


def facts(d):
    h = header(d)
    disc = {t["name"]: (t["year"] if t["year"] >= 1750 else None) for t in h["techs"]}
    first_rail = next((e["year"] for e in h["events"] if e["kind"] == "railway" and e["title"].startswith("The first")), None)
    return disc, first_rail, len(h["towns"]), len(h["firms"])


def fmt(v):
    if v is None or (isinstance(v, float) and not np.isfinite(v)):
        return "-"
    if abs(v) >= 1000:
        return f"{v:,.0f}"
    if abs(v) >= 10:
        return f"{v:.0f}"
    return f"{v:.3f}" if abs(v) < 1 else f"{v:.2f}"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--py", default="/tmp/py_s{seed}")
    ap.add_argument("--rs", default="/tmp/rs_s{seed}")
    ap.add_argument("--seeds", type=int, nargs="+", default=[1, 2, 3, 4])
    a = ap.parse_args()
    rows = []
    for seed in a.seeds:
        for impl, pat in (("py", a.py), ("rs", a.rs)):
            d = pat.format(seed=seed)
            if not Path(d, "series_monthly.csv").exists():
                continue
            rows.append((seed, impl, annual(d), facts(d)))

    for y in YEARS:
        print(f"\n### {y}\n")
        print("| seed | impl | " + " | ".join(COLS) + " |")
        print("|---|---|" + "---|" * len(COLS))
        for seed, impl, ann, _ in rows:
            print(f"| {seed} | {impl} | " + " | ".join(fmt(ann[c][y]) for c in COLS) + " |")
        for impl in ("py", "rs"):
            sel = [r for r in rows if r[1] == impl]
            if sel:
                means = [np.mean([r[2][c][y] for r in sel]) for c in COLS]
                print(f"| mean | {impl} | " + " | ".join(fmt(m) for m in means) + " |")

    print("\n### Discoveries and the first railway (year; - = not by 1900)\n")
    print("| seed | impl | " + " | ".join(TECHS) + " | first railway | towns | firms ever |")
    print("|---|---|" + "---|" * (len(TECHS) + 3))
    for seed, impl, _, (disc, rail, towns, firms) in rows:
        print(f"| {seed} | {impl} | " + " | ".join(str(disc.get(t) or "-") for t in TECHS)
              + f" | {rail:.1f} | {towns} | {firms} |" if rail else
              f"| {seed} | {impl} | " + " | ".join(str(disc.get(t) or "-") for t in TECHS) + f" | - | {towns} | {firms} |")
    for impl in ("py", "rs"):
        sel = [r for r in rows if r[1] == impl]
        if not sel:
            continue
        cells = []
        for t in TECHS:
            ys = [r[3][0].get(t) for r in sel if r[3][0].get(t)]
            cells.append(f"{np.mean(ys):.0f} ({len(ys)}/{len(sel)})" if ys else "-")
        rails = [r[3][1] for r in sel if r[3][1]]
        print(f"| mean | {impl} | " + " | ".join(cells)
              + f" | {np.mean(rails):.0f} | {np.mean([r[3][2] for r in sel]):.1f} | {np.mean([r[3][3] for r in sel]):.0f} |")


if __name__ == "__main__":
    main()
