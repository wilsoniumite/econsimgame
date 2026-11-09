"""Compare a --ground-debug shot with the CPU class map (--class-map) of the same view.

usage: python3 compare_classes.py shot.png [more.png ...]
Expects shot.classes.pgm beside each shot. Writes shot.diff.png: grey where the
two agree, red where they disagree on a class edge, yellow where they disagree
away from one (which should not happen), dark where the shot shows no class
colour (interface, smoothed edges) or the CPU map has no land.

The colours are those of debug_colour in src/terrain.wesl, by display class.
"""
import sys
import numpy as np
from PIL import Image

PAL = np.array([
    (0, 0, 255), (255, 255, 0), (0, 255, 0), (0, 128, 0), (255, 0, 255), (0, 255, 255),
    (255, 128, 0), (128, 0, 0), (128, 0, 255), (255, 0, 0), (0, 128, 255), (128, 255, 128),
], dtype=np.float32)

def read_pgm(path):
    data = open(path, "rb").read()
    parts = data.split(b"\n", 3)
    w, h = map(int, parts[1].split())
    return np.frombuffer(parts[3], dtype=np.uint8).reshape(h, w)

for shot in sys.argv[1:]:
    img = np.asarray(Image.open(shot).convert("RGB"), dtype=np.float32)
    cpu = read_pgm(shot[:-4] + ".classes.pgm")
    d = np.linalg.norm(img[:, :, None, :] - PAL[None, None, :, :], axis=3)
    gpu = d.argmin(axis=2)
    known = d.min(axis=2) < 20
    land = cpu != 255
    both = known & land
    agree = both & (gpu == cpu)
    differ = both & (gpu != cpu)
    # pixels next to a class edge in the CPU map, where a pixel's centre can fall either side
    c = cpu.astype(np.int16)
    edge = np.zeros_like(land)
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            edge |= np.roll(np.roll(c, dy, 0), dx, 1) != c
    inner_differ = differ & ~edge
    n = both.sum()
    print(f"{shot}: {n} land pixels compared, {agree.sum()} agree ({100 * agree.sum() / max(n, 1):.3f}%), "
          f"{differ.sum()} differ, of which {inner_differ.sum()} away from a class edge; "
          f"{(land & ~known).sum()} land pixels without a class colour (interface, smoothing)")
    if differ.sum():
        pairs, counts = np.unique(np.stack([cpu[differ], gpu[differ]], 1), axis=0, return_counts=True)
        top = sorted(zip(counts, map(tuple, pairs)), reverse=True)[:6]
        print("   most common (cpu, gpu) disagreements:", [(p, int(k)) for k, p in top])
    out = np.zeros(img.shape, dtype=np.uint8)
    out[agree] = (150, 150, 150)
    out[differ] = (255, 40, 40)
    out[inner_differ] = (255, 255, 0)
    Image.fromarray(out).save(shot[:-4] + ".diff.png")
