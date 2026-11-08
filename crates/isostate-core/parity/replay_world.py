"""Push the Rust world generator's noise fields through Python's own downstream code.

    cargo run -p isostate-core --release --example world_dump -- 1
    python3 crates/isostate-core/parity/replay_world.py

Python's code, fed the Rust noise and spine points, must give the Rust terrain
cell for cell: that isolates any difference between the generators to the
random numbers themselves.
"""
import os
import sys
sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parents[3]))
sys.dont_write_bytecode = True
import numpy as np
D = os.environ.get("ISOSTATE_WORLD_DIR", "/tmp/worldcmp")
from scipy import ndimage
from isostate import world as wm
H, W = 84, 120
ld = lambda n: np.fromfile(f"{D}/rs_{n}.f64", dtype="<f8")
base, detail = ld("base").reshape(H, W), ld("detail").reshape(H, W)
wx, wy = ld("wx").reshape(H, W) - 0.5, ld("wy").reshape(H, W) - 0.5
p = ld("p")
p0, p1, p2 = (p[0], p[1]), (p[2], p[3]), (p[4], p[5])
yy, xx = np.mgrid[0:H, 0:W]
fx, fy = xx / (W - 1), yy / (H - 1)
falloff = np.sqrt(((fx + 0.22 * wx - 0.5) / 0.5) ** 2 + ((fy + 0.22 * wy - 0.52) / 0.47) ** 2)
island = 1 - np.clip(falloff, 0, 1.4) ** 2.3
ridge_d = wm.bezier_distance(H, W, p0, p1, p2)
ridge = np.exp(-(ridge_d / (0.07 * W)) ** 2) * (0.6 + 0.6 * base)
elev = 0.5 * base + 0.14 * detail + 0.4 * ridge + 0.55 * island
elev[[0, 1, -2, -1], :] = elev.min()
elev[:, [0, 1, -2, -1]] = elev.min()
sea_level = np.quantile(elev, 1 - 0.58)
land = elev > sea_level
lab, n = ndimage.label(land)
if n > 1:
    sizes = ndimage.sum(land, lab, range(1, n + 1)); land = lab == 1 + int(np.argmax(sizes))
sea_lab, n = ndimage.label(~land)
if n > 1:
    bl = set(np.unique(np.concatenate([sea_lab[0], sea_lab[-1], sea_lab[:, 0], sea_lab[:, -1]])))
    land = land | (~land & ~np.isin(sea_lab, list(bl)))
h = np.where(land, (elev - sea_level) / (elev[land].max() - sea_level), 0.0)
h = np.clip(h, 0, 1) ** 1.35
terr = np.zeros((H, W)); terr[land & (h < 0.24)] = 1; terr[land & (h >= 0.24) & (h < 0.5)] = 2; terr[land & (h >= 0.5)] = 3
rt, rh = ld("terrain").reshape(H, W), ld("h").reshape(H, W)
print("python-from-rust-noise hills %.3f; rust hills %.3f; terrain cells equal %d/%d; max |h diff| %.2e" % (
    (terr == 2).sum() / land.sum(), (rt == 2).sum() / (rt > 0).sum(), (terr == rt).sum(), H * W, np.abs(h - rh).max()))
