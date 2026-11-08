"""Reference outputs of the scipy.ndimage routines the sim uses, for tests/ndimage.rs.

    python3 crates/isostate-core/parity/dump_ndimage.py crates/isostate-core/parity/fixtures/ndimage

Inputs are drawn here and saved next to the outputs, so the Rust test only reads files.
"""
import sys
from pathlib import Path

import numpy as np
from scipy import ndimage


def main(out):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    rng = np.random.default_rng(12345)
    H, W = 84, 120

    def save(name, arr):
        arr = np.asarray(arr)
        if arr.dtype == bool:
            arr = arr.astype(np.uint8)
        elif arr.dtype.kind in "iu" and arr.dtype != np.uint8:
            arr = arr.astype(np.int64)
        np.save(out / f"{name}.npy", np.ascontiguousarray(arr))

    # uniform filters: counts (like firm counts) and smooth values
    counts = np.zeros((H, W))
    idx = rng.integers(0, H * W, 400)
    np.add.at(counts.ravel(), idx, 1.0)
    vals = rng.random((H, W)) ** 2
    save("uf_counts", counts)
    save("uf_vals", vals)
    save("uf_counts_c5", ndimage.uniform_filter(counts, size=5, mode="constant"))
    save("uf_counts_c7", ndimage.uniform_filter(counts, size=7, mode="constant"))
    save("uf_vals_c5", ndimage.uniform_filter(vals, size=5, mode="constant"))
    save("uf_vals_r11", ndimage.uniform_filter(vals, size=11))

    # label, dilation and distance transform on a blobby mask
    blob = ndimage.gaussian_filter(rng.random((H, W)), 3) > 0.5
    save("mask", blob)
    lab, n = ndimage.label(blob)
    save("label", lab)
    save("label_n", np.array([n]))
    save("dilate_square", ndimage.binary_dilation(blob, structure=np.ones((3, 3))))
    save("dilate_cross", ndimage.binary_dilation(blob, iterations=1))
    sparse = rng.random((H, W)) > 0.995
    save("sparse", sparse)
    save("edt_sparse", ndimage.distance_transform_edt(~sparse))
    save("edt_blob", ndimage.distance_transform_edt(blob))

    # cubic zoom with the shapes smooth_noise uses
    for step, (gh, gw) in ((13, (10, 13)), (6, (18, 24)), (3, (32, 44)), (2, (46, 64)), (11, (11, 14))):
        grid = rng.random((gh, gw))
        save(f"zoom_in_{step}", grid)
        save(f"zoom_out_{step}", ndimage.zoom(grid, step, order=3))
    print("wrote", out)


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "crates/isostate-core/parity/fixtures/ndimage")
