"""Work out a sweep's stylised facts again, for one seed, from a run's own files.

    cargo run --release -p isostate-core -- run --seed 7 --out /tmp/s7
    cargo run --release -p isostate-core -- sweep --seeds 7 --out /tmp/sweep7
    python3 crates/isostate-core/parity/check_facts.py /tmp/s7 /tmp/sweep7/facts.json 7

The first directory holds run.bin, series_monthly.csv and events.json as
`isostate-core run` writes them (not --lite, and with the same rules, families
and end year as the sweep); the second argument is the sweep's facts.json.
Nothing here shares code with facts.rs: the definitions are written out again
from docs/rules/README.md, in plain Python with no packages, so a slip in
either shows as a difference. Exits non-zero if any fact differs.

Not checked here, because the files do not hold what they need: growth_capped_*,
works_top_1pct_capacity_share, works_utilisation and the area_* facts. Those
are worked out a second way in tests/sweep.rs.
"""
import csv
import json
import math
import struct
import sys
from collections import Counter

run_dir, facts_path, seed = sys.argv[1], sys.argv[2], int(sys.argv[3])

# ---- the monthly series
rows = list(csv.reader(open(f"{run_dir}/series_monthly.csv")))
names = rows[0]
cols = {n: [] for n in names}
for r in rows[1:]:
    for n, v in zip(names, r):
        cols[n].append(float(v) if v not in ("", "nan") else math.nan)
y0 = int(cols["year"][0])
n_years = len(rows[1:]) // 12
y1 = y0 + n_years - 1
years = range(y0, y1 + 1)


def annual(name, y):
    """The mean of a series over the months of year y that have a value."""
    v = [x for x in cols[name][(y - y0) * 12:(y - y0) * 12 + 12] if math.isfinite(x)]
    return sum(v) / len(v) if v else math.nan


# ---- run.bin: a uint32 header length, the JSON header, then the arrays
blob = open(f"{run_dir}/run.bin", "rb").read()
(hlen,) = struct.unpack_from("<I", blob, 0)
head = json.loads(blob[4:4 + hlen])
base = 4 + hlen
table = {a["name"]: a for a in head["arrays"]}


def array(name):
    a = table[name]
    raw = blob[base + a["offset"]: base + a["offset"] + a["nbytes"]]
    if a["dtype"] == "uint8":
        return raw
    return struct.unpack(f"<{len(raw) // 4}{'i' if a['dtype'] == 'int32' else 'f'}", raw)


def quantile(v, q):
    """numpy's default: linear between neighbours of a sorted list."""
    pos = q * (len(v) - 1)
    lo, hi = math.floor(pos), math.ceil(pos)
    return v[lo] + (v[hi] - v[lo]) * (pos - lo)


# the years facts are taken in: the first, every fiftieth after it, the last
marks = sorted({y0, y1} | {y for y in years if y % 50 == 0})
mine = {}

# ---- people and towns, work and wages
for y in marks:
    mine[f"pop_{y}"] = annual("pop", y)
    mine[f"urban_share_{y}"] = annual("urban_share", y)
    mine[f"towns_{y}"] = annual("towns", y)
    mine[f"farm_share_{y}"] = annual("ag_share", y)
    mine[f"industry_share_{y}"] = annual("ind_share", y)
for y in marks[1:]:
    mine[f"real_wage_{y}_vs_{y0}"] = annual("real_wage", y) / annual("real_wage", y0)
    mine[f"output_per_head_{y}_vs_{y0}"] = annual("gdp_pc", y) / annual("gdp_pc", y0)
per_head = [annual("gdp_pc", y) for y in years]
mine["output_per_head_peak_year"] = y0 + per_head.index(max(per_head))
founded = [t["founded"] for t in head["towns"] if t["founded"] >= y0]
mine["last_town_founded"] = max(founded) if founded else None

# ---- railways
km = cols["rail_km"]
first = next((t for t, v in enumerate(km) if v > 0), None)
mine["rail_first_year"] = None if first is None else y0 + first // 12
for y in sorted({y for y in marks[1:] if y >= 1850 or y == y1} | ({1870} if y0 < 1870 < y1 else set())):
    mine[f"rail_km_{y}"] = annual("rail_km", y)
december = [km[k * 12 + 11] for k in range(n_years)]
grew = [k for k in range(n_years) if december[k] > (december[k - 1] if k else 0.0)]
mine["rail_last_growth_year"] = y0 + grew[-1] if grew else None

# ---- works: standing at a year's end if born by then and not yet closed
firms, methods = head["firms"], head["methods"]
services = next((k for k, m in enumerate(methods) if m["name"] == "services"), None)


def standing(f, y):
    return f["born"] <= y and (f["died"] == -1 or f["died"] > y)


for y in marks:
    alive = [f for f in firms if standing(f, y)]
    mine[f"works_{y}"] = len(alive)
    mine[f"works_ex_services_{y}"] = sum(1 for f in alive if f["method"] != services)
mine[f"works_per_1000_people_{y1}"] = 1000 * mine[f"works_{y1}"] / mine[f"pop_{y1}"]
first_year, length, offset, capacity = array("firm_first"), array("firm_len"), array("firm_off"), array("firm_Kvals")
size = sorted(
    capacity[offset[k] + length[k] - 1] / methods[f["method"]]["K0"]
    for k, f in enumerate(firms)
    if standing(f, y1) and length[k] > 0 and first_year[k] + length[k] == n_years
)
mine[f"works_size_median_{y1}"] = quantile(size, 0.5)
mine[f"works_size_p90_{y1}"] = quantile(size, 0.9)
mine[f"works_size_largest_{y1}"] = size[-1]
life = sorted(f["died"] - f["born"] for f in firms if f["died"] >= y0)
mine["works_founded"] = sum(1 for f in firms if f["born"] >= y0)
mine["works_closed"] = len(life)
mine["works_closed_median_life"] = quantile(life, 0.5) if life else None

# ---- land: the whole cells a town, the works or dereliction has taken
use = array("use")
N = table["use"]["shape"][1]
uses = head["uses"]


def year_use(y):
    return use[(y - y0) * N:(y - y0 + 1) * N]


for name in ("urban", "industry", "derelict"):
    code = uses.index(name)
    for y in marks:
        mine[f"cells_{name}_{y}"] = sum(1 for u in year_use(y) if u == code)
urban = uses.index("urban")
churn = []
for y in range(y0, y1 - 9):
    now, later = year_use(y), year_use(y + 10)
    cells = [c for c in range(N) if now[c] == urban]
    if cells:
        churn.append(sum(1 for c in cells if later[c] != urban) / len(cells))
mine["urban_churn_10y"] = sum(churn) / len(churn) if churn else None

# ---- methods and inventions
for j, t in enumerate(head["techs"]):
    if t["known"]:
        continue
    name = t["name"]
    mine[f"discovered_{name}"] = t["year"] if t["year"] >= y0 else None
    if j >= len(methods):
        continue  # a tech nobody builds works for
    out = [annual(f"mout_{name}", y) for y in years]
    same_good = [m["name"] for m in methods if m["out"] == methods[j]["out"]]
    total = [sum(annual(f"mout_{m}", y) for m in same_good) for y in years]
    k = next((k for k, v in enumerate(out) if v > 0), None)
    mine[f"first_output_{name}"] = None if k is None else y0 + k
    k = next((k for k in range(n_years) if total[k] > 0 and out[k] >= 0.5 * total[k]), None)
    mine[f"half_of_output_{name}"] = None if k is None else y0 + k
for name, y in [("coke_furnace", 1800), ("textile_mill", 1830), ("bessemer", 1880), ("rolling_mill", 1850), ("steel_rolling", 1890)]:
    j = next((k for k, m in enumerate(methods) if m["name"] == name), None)
    if j is None or not y0 <= y <= y1:
        continue
    same_good = [m["name"] for m in methods if m["out"] == methods[j]["out"]]
    total = sum(annual(f"mout_{m}", y) for m in same_good)
    mine[f"share_{name}_{y}"] = annual(f"mout_{name}", y) / total if total > 0 else None

# ---- prices and cycles
start = max(1760, y0) if y1 >= 1762 else y0
for good in ("grain", "iron", "rails", "cloth"):
    if f"price_{good}" not in cols:
        continue
    p = [annual(f"price_{good}", y) for y in range(start, y1 + 1)]
    d = [math.log(b / a) for a, b in zip(p, p[1:])]
    if len(d) >= 2:
        m = sum(d) / len(d)
        mine[f"price_swing_{good}"] = math.sqrt(sum((x - m) ** 2 for x in d) / len(d))
farm = [annual("ag_share", y) for y in years]
bend = [abs(farm[k + 2] - 2 * farm[k + 1] + farm[k]) for k in range(n_years - 2)]
mine["farm_zigzag"] = 100 * sum(bend) / len(bend) if bend else None

# ---- news
events = json.load(open(f"{run_dir}/events.json"))
for kind, n in Counter(e["kind"] for e in events).items():
    mine[f"news_{kind}"] = n
mine["news_total"] = len(events)

# ---- against the sweep. The monthly file is written to six significant digits,
# so a fact made from it can differ from the sweep's in the sixth.
runs = [r for r in json.load(open(facts_path))["runs"] if r["seed"] == seed and r["ok"]]
if not runs:
    sys.exit(f"{facts_path} has no finished run of seed {seed}")
theirs = runs[0]["facts"]
differ = 0
for name, v in mine.items():
    t = theirs.get(name, "missing")
    if v is None or t is None or t == "missing":
        ok = v is None and t is None
    else:
        ok = abs(v - t) <= 2e-5 * max(abs(v), abs(t)) + 1e-9
    if not ok:
        differ += 1
        print(f"DIFFERS {name}: here {v}, the sweep {t}")
left = sorted(set(theirs) - set(mine))
print(f"{len(mine)} facts worked out again, {differ} differ; not checked here: {', '.join(left)}")
sys.exit(1 if differ else 0)
