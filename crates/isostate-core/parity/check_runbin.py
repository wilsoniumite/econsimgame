"""Check that a Rust-produced run.bin has the layout of a Python-produced one.

    python3 crates/isostate-core/parity/check_runbin.py /tmp/py_s7/run.bin /tmp/rs_s7/run.bin

Parses both with the `load_bin` logic of notebooks/explore.py and compares array
names, dtypes and shapes (the year and firm counts may differ between runs), the
header's keys at every level the viewer reads, and the 8-byte alignment.
Exits non-zero on any mismatch.
"""
import json
import sys
from pathlib import Path

import numpy as np


def load_bin(path):
    """Parse a run.bin into (header, arrays) without the viewer (notebooks/explore.py)."""
    blob = Path(path).read_bytes()
    hlen = int(np.frombuffer(blob[:4], np.uint32)[0])
    header = json.loads(blob[4:4 + hlen])
    base = 4 + hlen
    arrays = {}
    for a in header["arrays"]:
        dt = np.dtype(a["dtype"])
        arrays[a["name"]] = np.frombuffer(blob, dtype=dt, count=a["nbytes"] // dt.itemsize,
                                          offset=base + a["offset"]).reshape(a["shape"])
    return header, arrays


def keyset(items):
    out = set()
    for it in items:
        out |= set(it)
    return out


def main(py_path, rs_path):
    problems = []
    hp, ap = load_bin(py_path)
    hr, ar = load_bin(rs_path)
    blob = Path(rs_path).read_bytes()
    hlen = int(np.frombuffer(blob[:4], np.uint32)[0])
    if (4 + hlen) % 8:
        problems.append(f"header end {4 + hlen} is not 8-byte aligned")
    for a in hr["arrays"]:
        if a["offset"] % 8:
            problems.append(f"{a['name']}: offset {a['offset']} not aligned")

    names_p = [a["name"] for a in hp["arrays"]]
    names_r = [a["name"] for a in hr["arrays"]]
    if names_p != names_r:
        problems.append(f"array names differ:\n  python {names_p}\n  rust   {names_r}")
    ny_p, ny_r = hp["meta"]["n_years"], hr["meta"]["n_years"]
    nf_p, nf_r = len(hp["firms"]), len(hr["firms"])
    print(f"python: {ny_p} years, {nf_p} firms; rust: {ny_r} years, {nf_r} firms")
    for name in names_p:
        if name not in ar:
            continue
        p, r = ap[name], ar[name]
        if p.dtype != r.dtype:
            problems.append(f"{name}: dtype {p.dtype} vs {r.dtype}")
        # shapes may differ only where a dimension is the number of years, months, firms or K values
        sp = [d if d not in (ny_p, ny_p * 12, nf_p) else "Y" for d in p.shape]
        sr = [d if d not in (ny_r, ny_r * 12, nf_r) else "Y" for d in r.shape]
        if name == "firm_Kvals":
            sp, sr = ["n"], ["n"]
        if sp != sr:
            problems.append(f"{name}: shape {p.shape} vs {r.shape}")
        print(f"  {name:12s} {str(r.dtype):8s} python {str(p.shape):18s} rust {str(r.shape):18s} "
              f"range [{r.min() if r.size else '-'}, {r.max() if r.size else '-'}]")

    if list(hp) != list(hr):
        problems.append(f"header keys differ: {list(hp)} vs {list(hr)}")
    for key in ("meta",):
        if list(hp[key]) != list(hr[key]):
            problems.append(f"{key} keys differ: {list(hp[key])} vs {list(hr[key])}")
    for key in ("goods", "methods", "techs", "sectors", "towns", "firms", "rail_opened"):
        kp, kr = keyset(hp[key]), keyset(hr[key])
        if hr[key] and kp != kr:
            problems.append(f"{key} record keys differ: {sorted(kp)} vs {sorted(kr)}")
    kp, kr = keyset(f["reason"] for f in hp["firms"]), keyset(f["reason"] for f in hr["firms"])
    if hr["firms"] and kp != kr:
        problems.append(f"firm reason keys differ: {sorted(kp)} vs {sorted(kr)}")
    allowed = keyset(hp["events"])
    extra = keyset(hr["events"]) - allowed
    if extra:
        problems.append(f"event keys Python never writes: {sorted(extra)}")
    for key in ("uses", "disp", "share_groups"):
        if hp[key] != hr[key]:
            problems.append(f"{key} differs")
    if hp["series"]["names"] != hr["series"]["names"] or hp["series"]["meta"] != hr["series"]["meta"]:
        problems.append("series names or meta differ")
    if [g["name"] for g in hp["goods"]] != [g["name"] for g in hr["goods"]]:
        problems.append("goods differ")
    if hp["methods"] != hr["methods"]:
        problems.append("methods differ")
    print(f"events: python {len(hp['events'])}, rust {len(hr['events'])}; "
          f"kinds rust {sorted({e['kind'] for e in hr['events']})}")
    if problems:
        print("PROBLEMS:")
        for p in problems:
            print(" -", p)
        return 1
    print("layout matches")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2]))
