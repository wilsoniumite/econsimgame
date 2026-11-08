"""World generation statistics of the Python generator over seeds (see examples/world_stats.rs).

    python3 crates/isostate-core/parity/world_stats.py 100
"""
import sys
sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parents[3]))
sys.dont_write_bytecode = True
import numpy as np
from isostate import world as wm
from isostate.sim import make_rngs
n = int(sys.argv[1]) if len(sys.argv) > 1 else 30
hills, mounts, fert, bm, bs, dm, ds = [], [], [], [], [], [], []
for seed in range(1, n + 1):
    rng = make_rngs(seed)["world"]
    base = wm.smooth_noise(rng, 84, 120, scale=26, octaves=5, persistence=0.55)
    detail = wm.smooth_noise(rng, 84, 120, scale=8, octaves=3, persistence=0.5)
    bm.append(base.mean()); bs.append(base.std()); dm.append(detail.mean()); ds.append(detail.std())
    w = wm.generate_world(make_rngs(seed)["world"])
    land = w["land"]
    hills.append((w["terrain"] == 2).sum() / land.sum()); mounts.append((w["terrain"] == 3).sum() / land.sum())
    fert.append(w["fertility"][land].mean())
print(f"python {n} seeds: hills mean {np.mean(hills):.3f} median {np.median(hills):.3f} | mountains {np.mean(mounts):.3f} | fert {np.mean(fert):.3f} | base mean {np.mean(bm):.3f} std {np.mean(bs):.3f} | detail mean {np.mean(dm):.3f} std {np.mean(ds):.3f}")
