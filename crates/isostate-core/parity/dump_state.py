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
  firms.review              tag "review"
  people.found_towns        tag "found_towns"   (only in a year a town is founded)
  rail.plan                 tag "rail_plan"
  rail.plan_spurs           tag "rail_spurs"
  firms.entry               tag "entry"         (with --stochastic)
  tech.discovery            tag "discovery"     (with --stochastic)
  rail.progress             tag "rail_progress" (with --whole-year; the month's call)
  observer.record_month     tag "record_month"  (with --whole-year; the month's series
                                                 values go to record_month_after/series.json)
  sim.yearly                tag "yearly"        (with --whole-year; the whole yearly step)
  sim.init_state            tag "init"          (with --init; only init_after, the state it
                                                 returns, with the draws of every stream)
  sim.seed_firms            tag "seed_firms"    (with --init; the firms stream's draws)

Every numpy array in the state dict goes to `<out>/<tag>_<when>/<name>.npy`
(booleans as uint8), world arrays as `world_<name>.npy`, rail projects as
`project<k>_<name>.npy`, month statistics as `ms_<name>.npy`, and scalars, lists
and sets into `scalars.json`. Function arguments other than the state go to
`arg_<name>.npy` / `args.json`. The parameters are in `<out>/params.json`.

The steps that take a random stream (review, found_towns, entry, discovery and
the rail planners) are run with a recording copy of that stream: every number
numpy draws, including the ones `Generator.choice` draws internally, goes in
order to `<tag>_after/draws.npy` (integers as floats). For `yearly` each stream
is recorded, to `yearly_after/draws_<stream>.npy`. The events a call adds go to
`<tag>_after/events.json`. The Rust tests replay those draws
(`rng::Rng::replaying`), which checks the stochastic steps draw for draw.

Usage (from the repository root):

    python3 crates/isostate-core/parity/dump_state.py --seed 7 --year 1752 --month 3 \
        --found-towns --stochastic --init --out crates/isostate-core/parity/fixtures/s7_1752
    python3 crates/isostate-core/parity/dump_state.py --seed 7 --year 1843 --month 11 \
        --stochastic --whole-year --out crates/isostate-core/parity/fixtures/s7_1843
    python3 crates/isostate-core/parity/dump_state.py --seed 7 --year 1847 --month 5 \
        --stochastic --whole-year --out crates/isostate-core/parity/fixtures/s7_1847

The monthly capture is month `--month` (0-based) of year `--year`; the yearly
captures are the calls made inside `yearly(year)` at the end of that year (for
`--month 11` the yearly captures come right after the monthly one). Each fixture
directory is 0.5 to 0.8 GB; they are not committed (see .gitignore).
"""
import argparse
import json
import os
import sys
from pathlib import Path

import numpy as np

# numpy's default argsort breaks exact ties in an order that depends on the CPU
# (the AVX-512, AVX2 and baseline builds differ), so Python runs are not
# reproducible from one machine to another where ties occur (for example the
# eight cheapest sites in firms.candidate_cells). The fixtures pin every
# np.argsort without an explicit kind to the stable order, ties by index, which
# is what the Rust port uses (util::argsort).
_numpy_argsort = np.argsort


def _stable_argsort(a, axis=-1, kind=None, order=None, **kwargs):
    return _numpy_argsort(a, axis=axis, kind=kind or "stable", order=order, **kwargs)


np.argsort = _stable_argsort

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
# keep the Python package directory untouched: no bytecode, numba cache elsewhere
sys.dont_write_bytecode = True
os.environ.setdefault("NUMBA_CACHE_DIR", "/tmp/numba_cache")

from isostate import econ, firms, land, observer, people, rail, sim, tech  # noqa: E402
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


class RecordingGenerator(np.random.Generator):
    """A numpy Generator that logs every number it hands out, in order.

    `choice` calls `self.random` / `self.integers` internally, so its draws are
    logged too."""

    def __init__(self, bit_generator):
        super().__init__(bit_generator)
        self.draws = []

    def random(self, size=None, dtype=np.float64, out=None):
        v = super().random(size, dtype, out)
        self.draws.extend(np.asarray(v, dtype=float).ravel().tolist())
        return v

    def integers(self, low, high=None, size=None, dtype=np.int64, endpoint=False):
        v = super().integers(low, high, size, dtype, endpoint)
        self.draws.extend(np.asarray(v, dtype=float).ravel().tolist())
        return v

    def uniform(self, low=0.0, high=1.0, size=None):
        # numpy computes low + (high - low) * next_double, one double per value
        if size is None:
            size = np.broadcast(low, high).shape or None
        return low + (np.asarray(high) - low) * self.random(size)


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
    ap.add_argument("--found-towns", action="store_true", help="also capture people.found_towns (the year must found a town)")
    ap.add_argument("--stochastic", action="store_true", help="also capture firms.entry and tech.discovery")
    ap.add_argument("--whole-year", action="store_true", help="also capture rail.progress and the whole of sim.yearly")
    ap.add_argument("--init", action="store_true", help="also capture sim.init_state")
    a = ap.parse_args(argv)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    P = default_params(seed=a.seed, rent_regime=a.regime, end_year=a.year)
    (out / "params.json").write_text(json.dumps(_jsonable(P), indent=1))

    y0 = 1750
    target_month = P["warmup_years"] * 12 + (a.year - y0) * 12 + a.month
    yearly_tags = {"land", "transport", "productivity", "diffusion", "growth", "migration", "review"}
    if a.year >= 1800:
        yearly_tags |= {"rail_plan", "rail_spurs"}
    if a.found_towns:
        yearly_tags |= {"found_towns"}
    if a.stochastic:
        yearly_tags |= {"entry", "discovery"}
    pending = {"econ"} | yearly_tags
    if a.whole_year:
        pending |= {"rail_progress", "record_month", "yearly"}
    if a.init:
        pending |= {"init"}
    state = dict(month_calls=0, progress_calls=0, year=None, warm=None)
    # econ.step_month runs in the warm-up years too; rail.progress only after them
    targets = dict(month_calls=target_month, progress_calls=(a.year - y0) * 12 + a.month)

    def finish(tag):
        pending.discard(tag)
        if not pending:
            raise Done()

    def recording(rng):
        rec = RecordingGenerator(type(rng.bit_generator)())
        rec.bit_generator.state = rng.bit_generator.state
        return rec

    def handed_back(rng, rec):
        """Give the original stream the state the recording copy ended in."""
        rng.bit_generator.state = rec.bit_generator.state
        if isinstance(rng, RecordingGenerator):  # a step inside a recorded yearly call
            rng.draws.extend(rec.draws)

    def save_events(path, events, n0):
        new = [_jsonable(e) for e in events[n0:]]
        (path / "events.json").write_text(json.dumps(new, indent=1))

    def report(tag, st):
        print(f"captured {tag}: {len(st['ex_cell'])} extractive units, "
              f"{int(st['f_alive'].sum())} firms alive, {int(st['town_active'].sum())} towns, "
              f"{len(st['projects'])} rail projects", flush=True)

    def wrap(module, fname, tag, arg_names=(), rng_at=None, events_at=None, counter=None):
        """Capture one call: the `counter`-th call overall, or the call inside the
        target year's `yearly`. Arguments after the state (and the stream, if any)
        are saved under `arg_names`."""
        orig = getattr(module, fname)

        def wrapper(*args, **kwargs):
            if counter is not None:
                k = state[counter]
                state[counter] += 1
                hit = k == targets[counter] and tag in pending
            else:
                hit = state["year"] == a.year and not state["warm"] and tag in pending
            if not hit:
                return orig(*args, **kwargs)
            st = args[0]
            first = 3 if rng_at is None else rng_at + 1
            extra = dict(zip(arg_names, args[first:]))
            extra.update(kwargs)
            if fname == "update_land_use":
                extra.setdefault("initial", False)
            args = list(args)
            rec = None
            if rng_at is not None:
                rng = args[rng_at]
                rec = args[rng_at] = recording(rng)
            events = args[events_at] if events_at is not None else None
            n_events = len(events) if events is not None else 0
            dump_state(st, out / f"{tag}_before", extra)
            res = orig(*args, **kwargs)
            after = out / f"{tag}_after"
            dump_state(st, after, extra)
            if rec is not None:
                handed_back(rng, rec)
                np.save(after / "draws.npy", np.array(rec.draws, dtype=np.float64))
            if events is not None:
                save_events(after, events, n_events)
            report(tag, st)
            finish(tag)
            return res

        setattr(module, fname, wrapper)

    orig_record = observer.record_month

    def record_wrapper(st, C, P_, t):
        if not (t == targets["progress_calls"] and "record_month" in pending):
            return orig_record(st, C, P_, t)
        dump_state(st, out / "record_month_before", dict(t=t, n_months=len(st["series"]["pop"])))
        res = orig_record(st, C, P_, t)
        after = out / "record_month_after"
        after.mkdir(parents=True, exist_ok=True)
        row = {k: (float(v[t]) if np.isfinite(v[t]) else None) for k, v in st["series"].items()}
        (after / "series.json").write_text(json.dumps(row, indent=1))
        report("record_month", st)
        finish("record_month")
        return res

    observer.record_month = record_wrapper

    orig_init, orig_make_rngs = sim.init_state, sim.make_rngs

    def init_wrapper(P_, C=None):
        if "init" not in pending:
            return orig_init(P_, C)
        recs = {}

        def recording_make_rngs(seed):
            recs.update({k: recording(g) for k, g in orig_make_rngs(seed).items()})
            return recs

        orig_seed = sim.seed_firms

        def seed_wrapper(st, C, P_, rng, pbar, wbar):
            extra = dict(pbar=pbar, wbar=wbar)
            dump_state(st, out / "seed_firms_before", extra)
            n0 = len(rng["firms"].draws)
            res = orig_seed(st, C, P_, rng, pbar, wbar)
            after = out / "seed_firms_after"
            dump_state(st, after, extra)
            np.save(after / "draws.npy", np.array(rng["firms"].draws[n0:], dtype=np.float64))
            report("seed_firms", st)
            return res

        sim.make_rngs, sim.seed_firms = recording_make_rngs, seed_wrapper
        try:
            st, C_, rng = orig_init(P_, C)
        finally:
            sim.make_rngs, sim.seed_firms = orig_make_rngs, orig_seed
        after = out / "init_after"
        dump_state(st, after)
        plain = {}
        for k, g in recs.items():
            np.save(after / f"draws_{k}.npy", np.array(g.draws, dtype=np.float64))
            plain[k] = np.random.Generator(type(g.bit_generator)())
            plain[k].bit_generator.state = g.bit_generator.state
        report("init", st)
        finish("init")
        return st, C_, plain

    sim.init_state = init_wrapper
    orig_yearly = sim.yearly

    def yearly_wrapper(st, C, P_, rng, year, events, warm=False):
        state["year"], state["warm"] = year, warm
        try:
            if not (year == a.year and not warm and "yearly" in pending):
                return orig_yearly(st, C, P_, rng, year, events, warm=warm)
            recs = {k: recording(g) for k, g in rng.items()}
            n_events = len(events)
            dump_state(st, out / "yearly_before", dict(year=year))
            res = orig_yearly(st, C, P_, recs, year, events, warm=warm)
            after = out / "yearly_after"
            dump_state(st, after, dict(year=year))
            for k, g in rng.items():
                handed_back(g, recs[k])
                np.save(after / f"draws_{k}.npy", np.array(recs[k].draws, dtype=np.float64))
            save_events(after, events, n_events)
            report("yearly", st)
        finally:
            state["year"], state["warm"] = None, None
        finish("yearly")
        return res

    sim.yearly = yearly_wrapper
    wrap(econ, "step_month", "econ", counter="month_calls")
    wrap(rail, "progress", "rail_progress", arg_names=("year_float",), events_at=4, counter="progress_calls")
    wrap(land, "update_land_use", "land", arg_names=("year", "pbar", "wbar", "initial"))
    wrap(land, "update_transport", "transport")
    wrap(firms, "update_productivity", "productivity")
    wrap(tech, "diffusion", "diffusion")
    wrap(people, "growth", "growth", arg_names=("year",))
    wrap(people, "migration", "migration")
    wrap(firms, "review", "review", arg_names=("year", "pbar", "wbar"), rng_at=3)
    wrap(people, "found_towns", "found_towns", arg_names=("year",), rng_at=3, events_at=5)
    wrap(rail, "plan", "rail_plan", arg_names=("year", "pbar"), rng_at=3, events_at=6)
    wrap(rail, "plan_spurs", "rail_spurs", arg_names=("year", "pbar"), rng_at=3, events_at=6)
    wrap(firms, "entry", "entry", arg_names=("year", "pbar", "wbar"), rng_at=3)
    wrap(tech, "discovery", "discovery", arg_names=("year",), rng_at=3, events_at=5)
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
