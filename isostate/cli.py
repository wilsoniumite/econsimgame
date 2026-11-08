"""Command line: run scenarios, save them, and build the viewer.

    python -m isostate run --seed 7 --regime landlords --out runs/landlords
    python -m isostate html runs/landlords runs/commons --out dist/isolated_state.html
    python -m isostate build --seed 7 --out dist/isolated_state.html   # both steps, two scenarios
"""
import argparse
import csv
import json
from pathlib import Path

import numpy as np

from . import record
from .params import default_params
from .sim import run

LABELS = {
    "landlords": "Landlords keep the rent",
    "commons": "Rent paid out as a dividend",
}


def run_scenario(seed, regime, out, end_year=None, progress=True, lite=False):
    overrides = dict(seed=seed, rent_regime=regime)
    if end_year:
        overrides["end_year"] = end_year
    P = default_params(**overrides)
    result = run(P, progress=progress)
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    blob = record.pack(result, label=LABELS.get(regime, regime), lite=lite)
    (out / "run.bin").write_bytes(blob)
    st = result["state"]
    names = list(st["series"].keys())
    y0 = result["content"]["start_year"]
    with open(out / "series_monthly.csv", "w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["year", "month"] + names)
        n = len(st["series"][names[0]])
        for t in range(n):
            w.writerow([y0 + t // 12, t % 12 + 1] + [f"{st['series'][k][t]:.6g}" for k in names])
    (out / "events.json").write_text(json.dumps(record._clean(result["events"]), indent=1))
    (out / "params.json").write_text(json.dumps(record._clean(P), indent=1))
    return result


def build_html(dirs, out, keys=None):
    blobs = []
    for i, d in enumerate(dirs):
        key = keys[i] if keys else Path(d).name
        blobs.append((key, Path(d, "run.bin").read_bytes()))
    record.build_html(blobs, out=out)
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(prog="isostate")
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("--seed", type=int, default=7)
    r.add_argument("--regime", default="landlords", choices=list(LABELS))
    r.add_argument("--out", required=True)
    r.add_argument("--end-year", type=int, default=None)
    r.add_argument("--lite", action="store_true")
    h = sub.add_parser("html")
    h.add_argument("dirs", nargs="+")
    h.add_argument("--out", default="dist/isolated_state.html")
    b = sub.add_parser("build")
    b.add_argument("--seed", type=int, default=7)
    b.add_argument("--out", default="dist/isolated_state.html")
    a = ap.parse_args(argv)
    if a.cmd == "run":
        run_scenario(a.seed, a.regime, a.out, a.end_year, lite=a.lite)
    elif a.cmd == "html":
        build_html(a.dirs, a.out)
    else:
        run_scenario(a.seed, "landlords", "runs/landlords")
        run_scenario(a.seed, "commons", "runs/commons", lite=True)
        build_html(["runs/landlords", "runs/commons"], a.out)
    print("done")


if __name__ == "__main__":
    main()
