# %% [markdown]
# # Exploring Isolated State runs
# Cells for VS Code / Jupyter (`# %%`). Everything is plain functions over the
# dicts and arrays the sim already returns, so any cell can be lifted into a module.

# %%
import json
from pathlib import Path

import numpy as np
import pandas as pd
from tqdm.auto import tqdm

from isostate import default_params, record, run

ROOT = Path(__file__).resolve().parent.parent if "__file__" in globals() else Path("..").resolve()

# %% [markdown]
# ## Load a saved run (written by `python -m isostate run ... --out runs/landlords`)

# %%
def load_bin(path):
    """Parse a run.bin into (header, arrays) without the viewer."""
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


def annual(path):
    """Annual means of every monthly series, one row per year."""
    d = pd.read_csv(Path(path) / "series_monthly.csv")
    return d.groupby("year").mean().drop(columns="month")


runs = {name: annual(ROOT / "runs" / name) for name in ["landlords", "commons"] if (ROOT / "runs" / name).exists()}
cols = ["pop", "urban_share", "real_wage", "gdp_pc", "rail_km", "out_iron", "out_cloth"]
for name, a in runs.items():
    print(name)
    print(a.loc[::25, cols].round(3))

# %% [markdown]
# ## Events by kind and decade

# %%
def events_table(path):
    ev = pd.DataFrame(json.loads((Path(path) / "events.json").read_text()))
    ev["decade"] = (ev["year"] // 10 * 10).astype(int)
    return ev


if runs:
    ev = events_table(ROOT / "runs" / "landlords")
    print(ev.pivot_table(index="decade", columns="kind", values="title", aggfunc="count", fill_value=0))

# %% [markdown]
# ## Von Thünen check: rent against carting distance in a given year

# %%
def rent_gradient(header, arrays, year):
    k = year - header["meta"]["start_year"]
    use, rent, dist = arrays["use"][k], arrays["rent"][k], arrays["dist"][k]
    land = (use >= 2) & (use <= 8)
    rent_val = np.where(rent > 0, 10 ** (rent / 255 * 6 - 3), 0.0)
    df = pd.DataFrame({"dist_km": dist[land] / 4 * 2, "rent": rent_val[land], "use": np.array(header["uses"])[use[land]]})
    df["band"] = pd.cut(df["dist_km"], [0, 10, 20, 30, 45, 60, 90, 400])
    return df.groupby("band", observed=True).agg(rent=("rent", "median"), cells=("rent", "size"),
                                                 farm=("use", lambda u: (u == "farm").mean()),
                                                 pasture=("use", lambda u: (u == "pasture").mean()))


if (ROOT / "runs" / "landlords" / "run.bin").exists():
    H, A = load_bin(ROOT / "runs" / "landlords" / "run.bin")
    print(rent_gradient(H, A, 1800).round(3))

# %% [markdown]
# ## Seed sweep: how robust is a headline?
# Each run takes about 80 seconds for the full 150 years, so this uses a shorter horizon.

# %%
def sweep(seeds, end_year=1850, **overrides):
    rows = []
    for seed in tqdm(seeds, desc="seeds"):
        res = run(default_params(seed=seed, end_year=end_year, **overrides), progress=False)
        s = res["state"]["series"]
        last = slice(-12, None)
        rows.append(dict(seed=seed,
                         pop=float(np.mean(s["pop"][last])),
                         urban_share=float(np.mean(s["urban_share"][last])),
                         real_wage=float(np.mean(s["real_wage"][last])),
                         rail_km=float(np.mean(s["rail_km"][last])),
                         first_rail=next((e["year"] for e in res["events"] if e["kind"] == "railway"), None)))
    return pd.DataFrame(rows)


# sw = sweep(range(1, 9))
# sw.describe()
