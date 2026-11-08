"""Dump Python simulation state around single deterministic steps, for the Rust parity tests.

The Python package is the oracle. This script runs it unchanged and, by wrapping
module functions, saves the full state just before and just after one call of:

  econ.step_month           tag "econ"       (the monthly economy)
  land.update_land_use      tag "land"       (yearly land allocation and urban growth)
  land.update_transport     tag "transport"  (cost graph, Dijkstra fields, catchments, delta fields)
  firms.update_productivity tag "productivity"
  tech.diffusion            tag "diffusion"
  people.growth             tag "growth"
  people.migration          tag "migration"

Every numpy array in the state dict goes to `<out>/<tag>_<when>/<name>.npy`
(booleans as uint8), world arrays as `world_<name>.npy`, rail projects as
`project<k>_<name>.npy`, month statistics as `ms_<name>.npy`, and scalars, lists
and sets into `scalars.json`. Function arguments other than the state go to
`arg_<name>.npy` / `args.json`. The parameters are in `<out>/params.json`.

Usage (from the repository root):

    python3 crates/isostate-core/parity/dump_state.py --seed 7 --year 1752 --month 3 \
        --out crates/isostate-core/parity/fixtures/s7_1752
    python3 crates/isostate-core/parity/dump_state.py --seed 7 --year 1836 --month 5 \
        --out crates/isostate-core/parity/fixtures/s7_1836

The monthly capture is month `--month` (0-based) of year `--year`; the yearly
captures are the calls made inside `yearly(year)` at the end of that year (for
`--month 11` the yearly captures come right after the monthly one). Each fixture
directory is about 150 MB; they are not committed (see .gitignore).
"""
import argparse
import json
import os
import sys
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
# keep the Python package directory untouched: no bytecode, numba cache elsewhere
sys.dont_write_bytecode = True
os.environ.setdefault("NUMBA_CACHE_DIR", "/tmp/numba_cache")

from isostate import econ, firms, land, people, sim, tech  # noqa: E402
from isostate.params import default_params  # noqa: E402


class Done(Exception):
    pass


def _scalar(v):
    if isinstance(v, (bool, np.bool_)):
        return bool(v)
    if isinstance(v, (int, np.integer)):
        return int(v)
    if isinstance(v, (float, np.floating)):
        v = float(v)
        if not np.isfinite(v):
            return "nan" if np.isnan(v) else ("inf" if v > 0 else "-inf")
        return v
    return v


def _save_array(path, name, arr):
    arr = np.asarray(arr)
    if arr.dtype == bool:
        arr = arr.astype(np.uint8)
    elif arr.dtype.kind in "iu" and arr.dtype != np.uint8:
        arr = arr.astype(np.int64)
    elif arr.dtype.kind == "f":
        arr = arr.astype(np.float64)
    np.save(path / f"{name}.npy", np.ascontiguousarray(arr))


def _jsonable(v):
    if isinstance(v, dict):
        return {str(k): _jsonable(x) for k, x in v.items()}
    if isinstance(v, (list, tuple)):
        return [_jsonable(x) for x in v]
    if isinstance(v, set):
        return sorted(_jsonable(x) for x in v)
    if isinstance(v, np.ndarray):
        return [_jsonable(x) for x in v.tolist()]
    return _scalar(v)


def dump_state(st, path, args=None):
    path.mkdir(parents=True, exist_ok=True)
    scalars = {}
    for key, v in st.items():
        if key in ("series", "series_meta", "obs", "town_cells"):
            continue
        if key == "world":
            for wk, wv in v.items():
                if wk.startswith("_"):
                    continue
                if isinstance(wv, np.ndarray):
                    _save_array(path, f"world_{wk}", wv)
                else:
                    scalars[f"world_{wk}"] = _jsonable(wv)
        elif key == "graph":
            for gk in ("a", "b", "w", "railed"):
                _save_array(path, f"graph_{gk}", v[gk])
        elif key == "projects":
            meta = []
            for k, pr in enumerate(v):
                m = {}
                for pk, pv in pr.items():
                    if isinstance(pv, np.ndarray):
                        _save_array(path, f"project{k}_{pk}", pv)
                    else:
                        m[pk] = _scalar(pv)
                meta.append(m)
            scalars["projects"] = meta
        elif key == "month_stats":
            for mk, mv in v.items():
                if isinstance(mv, np.ndarray):
                    _save_array(path, f"ms_{mk}", mv)
                else:
                    scalars[f"ms_{mk}"] = _scalar(mv)
        elif key == "rail_cooldown":
            scalars[key] = [[int(a), int(b), int(y)] for (a, b), y in v.items()]
        elif isinstance(v, np.ndarray):
            _save_array(path, key, v)
        elif isinstance(v, (list, tuple, set, dict)):
            try:
                scalars[key] = _jsonable(v)
            except TypeError:
                pass
        else:
            scalars[key] = _scalar(v)
    if args:
        plain = {}
        for k, v in args.items():
            if isinstance(v, np.ndarray):
                _save_array(path, f"arg_{k}", v)
            else:
                plain[k] = _scalar(v)
        (path / "args.json").write_text(json.dumps(plain, indent=1))
    (path / "scalars.json").write_text(json.dumps(scalars, indent=1))


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--regime", default="landlords")
    ap.add_argument("--year", type=int, default=1752)
    ap.add_argument("--month", type=int, default=3)
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    P = default_params(seed=a.seed, rent_regime=a.regime, end_year=a.year)
    (out / "params.json").write_text(json.dumps(_jsonable(P), indent=1))

    y0 = 1750
    target_month = P["warmup_years"] * 12 + (a.year - y0) * 12 + a.month
    yearly_tags = {"land", "transport", "productivity", "diffusion", "growth", "migration"}
    pending = {"econ"} | yearly_tags
    state = dict(month_calls=0, year=None, warm=None)

    def finish(tag):
        pending.discard(tag)
        if not pending:
            raise Done()

    def wrap(module, fname, tag, arg_names=()):
        orig = getattr(module, fname)

        def wrapper(*args, **kwargs):
            if tag == "econ":
                k = state["month_calls"]
                state["month_calls"] += 1
                hit = k == target_month
            else:
                hit = state["year"] == a.year and not state["warm"] and tag in pending
            if not hit:
                return orig(*args, **kwargs)
            st = args[0]
            extra = {}
            for name, val in zip(arg_names, args[3:]):
                extra[name] = val
            for name, val in kwargs.items():
                extra[name] = val
            if fname == "update_land_use":
                extra.setdefault("initial", False)
            dump_state(st, out / f"{tag}_before", extra)
            res = orig(*args, **kwargs)
            dump_state(st, out / f"{tag}_after", extra)
            print(f"captured {tag}: {len(st['ex_cell'])} extractive units, "
                  f"{int(st['f_alive'].sum())} firms alive, {int(st['town_active'].sum())} towns, "
                  f"{len(st['projects'])} rail projects", flush=True)
            finish(tag)
            return res

        setattr(module, fname, wrapper)

    orig_yearly = sim.yearly

    def yearly_wrapper(st, C, P_, rng, year, events, warm=False):
        state["year"], state["warm"] = year, warm
        try:
            return orig_yearly(st, C, P_, rng, year, events, warm=warm)
        finally:
            state["year"], state["warm"] = None, None

    sim.yearly = yearly_wrapper
    wrap(econ, "step_month", "econ")
    wrap(land, "update_land_use", "land", arg_names=("year", "pbar", "wbar", "initial"))
    wrap(land, "update_transport", "transport")
    wrap(firms, "update_productivity", "productivity")
    wrap(tech, "diffusion", "diffusion")
    wrap(people, "growth", "growth", arg_names=("year",))
    wrap(people, "migration", "migration")
    try:
        sim.run(P, progress=False)
    except Done:
        pass
    if pending:
        print("not captured:", sorted(pending), file=sys.stderr)
        return 1
    print("wrote", out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
