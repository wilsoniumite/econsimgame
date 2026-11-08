"""Replay the raw random draws of one Rust smooth_noise call through Python's smooth_noise.

    cargo run -p isostate-core --release --example noise_dump -- 26 5 0.55
    python3 crates/isostate-core/parity/replay_noise.py 26 5 0.55
"""
import os
import sys
sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parents[3]))
sys.dont_write_bytecode = True
import numpy as np
D = os.environ.get("ISOSTATE_WORLD_DIR", "/tmp/worldcmp")
from isostate import world as wm
scale, octaves, pers = float(sys.argv[1]), int(sys.argv[2]), float(sys.argv[3])
draws = list(np.fromfile(D + "/noise_draws.f64", dtype="<f8"))
class Fake:
    def random(self, shape):
        n = int(np.prod(shape)); out = np.array(draws[:n]); del draws[:n]; return out.reshape(shape)
    def integers(self, lo, hi):
        v = int(draws.pop(0)); assert lo <= v < hi; return v
out = wm.smooth_noise(Fake(), 84, 120, scale, octaves, pers)
rs = np.fromfile(D + "/noise_out.f64", dtype="<f8").reshape(84, 120)
print(f"scale {scale} octaves {octaves}: max |python - rust| = {np.abs(out - rs).max():.2e}; leftover draws {len(draws)}")
