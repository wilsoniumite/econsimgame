"""Record a whole Python run for the Rust replay test (tests/parity.rs, full_run_*).

    python3 crates/isostate-core/parity/record_run.py --seed 7 --end-year 1800 \
        --out crates/isostate-core/parity/fixtures/run_s7_1800

Every number each random stream draws goes, in order, to `draws_<stream>.npy`
(integers as floats); the world to `world_<name>.npy`; every monthly series to
`series.npy` (one row per series, names in `series.json`); the events to
`events.json`; the parameters to `params.json`. As in dump_state.py, np.argsort
is pinned to the stable order, so the run does not depend on the CPU at ties.
The Rust test starts from this world (its own generator matches scipy's cubic
zoom only to about 1e-12, enough to break exact ties differently), replays the
draws and must reproduce the series.
"""
import argparse
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True  # before importing dump_state: no __pycache__ here

import numpy as np  # noqa: E402

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dump_state as ds  # noqa: E402  (pins np.argsort, sets up the import path)
from isostate import sim  # noqa: E402
from isostate.params import default_params  # noqa: E402


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--regime", default="landlords")
    ap.add_argument("--end-year", type=int, default=1800)
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    P = default_params(seed=a.seed, rent_regime=a.regime, end_year=a.end_year)
    (out / "params.json").write_text(json.dumps(ds._jsonable(P), indent=1))

    recs = {}
    orig_make_rngs = sim.make_rngs

    def recording_make_rngs(seed):
        for k, g in orig_make_rngs(seed).items():
            r = ds.RecordingGenerator(type(g.bit_generator)())
            r.bit_generator.state = g.bit_generator.state
            recs[k] = r
        return recs

    sim.make_rngs = recording_make_rngs
    res = sim.run(P, progress=False)
    st = res["state"]
    for k, r in recs.items():
        np.save(out / f"draws_{k}.npy", np.array(r.draws, dtype=np.float64))
    for wk, wv in st["world"].items():
        if isinstance(wv, np.ndarray) and not wk.startswith("_"):
            ds._save_array(out, f"world_{wk}", wv)
    scalars = {f"world_{wk}": ds._jsonable(wv) for wk, wv in st["world"].items()
               if not isinstance(wv, np.ndarray) and not wk.startswith("_")}
    (out / "scalars.json").write_text(json.dumps(scalars, indent=1))
    names = list(st["series"].keys())
    np.save(out / "series.npy", np.array([st["series"][k] for k in names], dtype=np.float64))
    (out / "series.json").write_text(json.dumps(names))
    (out / "events.json").write_text(json.dumps([ds._jsonable(e) for e in res["events"]], indent=1))
    print("wrote", out, {k: len(r.draws) for k, r in recs.items()})
    return 0


if __name__ == "__main__":
    sys.exit(main())
