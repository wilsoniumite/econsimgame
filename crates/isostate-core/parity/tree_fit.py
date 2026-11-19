"""Fit the discovery thresholds of the production tree to a sweep.

    python crates/isostate-core/parity/tree_fit.py SWEEP_DIR [--overlay data/game.toml] [--damp 0.7] [--only A,B] [--write]

docs/rules/tree.md, "The fitter". Every invention of the overlay carries
`target`, the year it is aimed at, and `discover`, what the island must have
made since 1750 before it can be found. The thresholds are plain numbers of
units, so they are right for one island and one set of rules: when either
changes (the mines made ore a third cheaper and every invention on iron and
steel came ten years early), they have to be fitted again.

SWEEP_DIR is a sweep under the game's rules with the overlay as it stands
(`isostate-core sweep --rules game --families 6 --seeds 1-8 --out SWEEP_DIR`).
From its `annual.csv` this takes each seed's cumulative output of every good,
year by year, and for each invention works out the year it would be found on
average under the engine's own hazard (`tech::discovery`: each year
`discovery_rate * p^4 / (1 + p^4)`, with p the least ratio of what has been made
to what is asked). It prints that beside the year the sweep itself found it,
then the thresholds that would bring the average to the year aimed at.

The new thresholds keep the shape `SPEC` gives each invention: one good is its
clock (what the island has made of it in some year Y, averaged over seeds), the
others are guards at 0.85 of what the island has made of them by the same year,
and a threshold marked "keep" is left as written. Y is the one number solved
for. Moving one discovery moves the output the others are counted in, so fit,
sweep and fit again: `--damp` moves each threshold only that share of the way
(in logs) so that two or three passes settle. `--write` rewrites the
`discover = { ... }` lines of the overlay in place; without it nothing is
changed. `--only` fits and writes the inventions named and leaves every other
threshold as it is written (a later stage's inventions are fitted so, without
touching the island before them).

The hazard spreads a discovery over about twelve years either side of its
mean, so with eight seeds a mean within four or five years of the aim is as
good as the fit can be told to be.
"""
import csv
import math
import re
import sys
import tomllib

DISCOVERY_RATE = 0.18  # params.rs
START, END = 1750, 1900

# invention: [(good, share of what the island has made by year Y, or "keep")]. The first is the clock.
SPEC = {
    "bessemer": [("steel", 1.0), ("machinery", 0.85), ("iron", "keep")],
    "steel_rolling": [("steel", 1.0), ("rails", 0.85), ("machinery", 0.85)],
    "contract_builders": [("building", 1.0), ("machinery", 0.85)],
    "steam_sawmill": [("lumber", 1.0), ("machinery", 0.85)],
    "cement_works": [("lime", 1.0), ("machinery", 0.85)],
    "steam_brewery": [("beer", 1.0), ("coal", 0.85)],
    "pottery_works": [("pottery", 1.0), ("coal", "keep")],
    "gasworks": [("iron", 1.0), ("coal", 0.85)],
    "edge_tool_works": [("steel", "keep"), ("tools", "keep")],
    "tool_factory": [("steel", 1.0), ("machinery", 0.85)],
    "hot_blast": [("iron", 1.0), ("coal", 0.85), ("machinery", 0.85)],
    "machine_shop": [("machinery", 1.0), ("steel", 0.85)],
    "engineering_works": [("steel", 1.0), ("machinery", 0.85)],
    "continuous_kiln": [("machinery", 1.0), ("bricks", "keep")],
    "emporium": [("services", 1.0), ("gas", "keep")],
    "alkali_works": [("chemicals", 1.0), ("iron", 0.85)],
    "ammonia_soda": [("chemicals", 1.0), ("steel", 0.85)],
    "sheet_glass": [("glass", 1.0), ("chemicals", 0.8), ("iron", 0.85)],
    "tank_furnace": [("glass", 1.0), ("steel", 0.85)],
    "machine_paper": [("paper", 1.0), ("machinery", 0.85)],
    # stage 6, the content after 1900 (docs/rules/tree.md, "After 1900: electricity")
    "electrical_works": [("steel", 1.0), ("machinery", 0.85)],
    "coal_power_station": [("electricals", 1.0), ("coal", 0.85)],
}


def load(sweep):
    """seed -> good -> list over years of cumulative output at the end of the year."""
    rows = {}
    with open(sweep + "/annual.csv", newline="") as f:
        for r in csv.DictReader(f):
            rows.setdefault(int(r["seed"]), {})[int(r["year"])] = r
    cum, first = {}, {}
    for seed, years in rows.items():
        goods = [c[4:] for c in years[START] if c.startswith("out_")]
        cum[seed], first[seed] = {}, {}
        for g in goods:
            total, path = 0.0, []
            for y in range(START, END + 1):
                if y in years and years[y].get("out_" + g, "") != "":
                    total += 12.0 * float(years[y]["out_" + g])
                path.append(total)
            cum[seed][g] = path
            first[seed][g] = 12.0 * float(years[START].get("out_" + g) or 0.0)
    return cum, first


def found_in(sweep):
    """invention -> mean year found in the sweep, and in how many runs."""
    out = {}
    with open(sweep + "/facts.csv", newline="") as f:
        rows = list(csv.DictReader(f))
    for c in rows[0]:
        if c.startswith("discovered_"):
            v = [float(r[c]) for r in rows if r[c] not in ("", None)]
            out[c[len("discovered_"):]] = (sum(v) / len(v) if v else float("nan"), len(v), len(rows))
    return out


def resolve(x, first):
    """A threshold as the engine reads it: units, or "xN" years of the first year's output."""
    return float(x[1:]) * first if isinstance(x, str) else float(x)


def expected(thresholds, cum, first):
    """The mean over seeds of the year found on average; a run that has not found it by the end counts as five years after."""
    total = 0.0
    for seed in cum:
        need = [(g, resolve(x, first[seed].get(g, 0.0))) for g, x in thresholds.items() if g in cum[seed]]
        alive, e = 1.0, 0.0
        for k in range(END - START + 1):
            p = min((cum[seed][g][k] / x if x > 0 else float("inf")) for g, x in need) if need else 0.0
            p4 = max(p, 0.0) ** 4
            hz = DISCOVERY_RATE * p4 / (1.0 + p4)
            e += alive * hz * (START + k)
            alive *= 1.0 - hz
        total += e + alive * (END + 5)
    return total / len(cum)


def made_by(cum, g, year):
    """What the island has made of g by the end of a (fractional) year, averaged over seeds."""
    year = min(max(year, START), END)
    k, frac = int(year - START), year - START - int(year - START)
    v = []
    for seed in cum:
        path = cum[seed].get(g)
        if path is None:
            continue
        a = path[k]
        b = path[min(k + 1, END - START)]
        v.append(a + frac * (b - a))
    return sum(v) / len(v) if v else 0.0


def shaped(name, now, cum, year):
    out = {}
    for g, k in SPEC[name]:
        if g not in now:
            continue
        out[g] = now[g] if k == "keep" else k * made_by(cum, g, year)
    for g in now:  # a threshold the spec does not name is kept
        out.setdefault(g, now[g])
    return out


def sig(x):
    """Two significant figures: the fit is not told more finely than that."""
    if isinstance(x, str) or x <= 0:
        return x
    e = math.floor(math.log10(x)) - 1
    return round(x / 10 ** e) * 10 ** e


def fit(name, now, target, cum, first):
    lo, hi = float(START), float(END)
    # an aim that the kept thresholds alone put out of reach: leave it as it is
    if expected(shaped(name, now, cum, lo), cum, first) >= target:
        return dict(now)
    for _ in range(40):
        mid = 0.5 * (lo + hi)
        if expected(shaped(name, now, cum, mid), cum, first) < target:
            lo = mid
        else:
            hi = mid
    return shaped(name, now, cum, 0.5 * (lo + hi))


def written(d):
    def one(v):
        return '"%s"' % v if isinstance(v, str) else "%.1f" % v
    return "{ " + ", ".join("%s = %s" % (g, one(v)) for g, v in d.items()) + " }"


def main():
    args = sys.argv[1:]
    if not args or args[0].startswith("--"):
        sys.exit(__doc__)
    sweep, overlay, damp, write, only = args[0], "data/game.toml", 0.7, False, None
    k = 1
    while k < len(args):
        if args[k] == "--overlay":
            overlay, k = args[k + 1], k + 2
        elif args[k] == "--damp":
            damp, k = float(args[k + 1]), k + 2
        elif args[k] == "--only":
            only, k = set(args[k + 1].split(",")), k + 2
        elif args[k] == "--write":
            write, k = True, k + 1
        else:
            sys.exit("unknown argument " + args[k])
    with open(overlay, "rb") as f:
        over = tomllib.load(f)
    cum, first = load(sweep)
    found = found_in(sweep)
    text = open(overlay, encoding="utf-8", newline="").read()
    print("%-20s %5s %12s %9s   %s" % ("invention", "aim", "sweep found", "reckoned", "thresholds now -> fitted"))
    for m in over.get("method", []) + over.get("tech", []):
        name, target, now = m["name"], m.get("target"), m.get("discover")
        if not target or not now or name not in SPEC or (only is not None and name not in only):
            continue
        reckoned = expected(now, cum, first)
        new = fit(name, now, target, cum, first)
        out = {}
        for g, v in now.items():
            if isinstance(v, str) or isinstance(new[g], str):
                out[g] = v
            else:
                out[g] = sig(math.exp(math.log(v) + damp * (math.log(max(new[g], 1e-9)) - math.log(v))))
        f = found.get(name, (float("nan"), 0, 0))
        print("%-20s %5d %8.0f (%d/%d) %7.0f   %s" % (name, target, f[0], f[1], f[2], reckoned, "  ".join(
            "%s %s -> %s" % (g, ("%.3g" % v if not isinstance(v, str) else v), ("%.3g" % out[g] if not isinstance(out[g], str) else out[g])) for g, v in now.items())))
        if write and out != now:
            # the entry's own `discover` line: the first after its `name = "..."`
            at = text.index('name = "%s"' % name)
            m2 = re.compile(r"^discover = \{[^}]*\}", re.M).search(text, at)
            nxt = text.find("[[", at)
            if m2 is None or (nxt != -1 and m2.start() > nxt):
                sys.exit("no discover line for " + name)
            text = text[:m2.start()] + "discover = " + written(out) + text[m2.end():]
    if write:
        open(overlay, "w", encoding="utf-8", newline="").write(text)
        print("\nwritten to " + overlay + ": sweep again and fit again until the means settle")


if __name__ == "__main__":
    main()
