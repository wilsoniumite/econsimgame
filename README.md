# Isolated State

An agent-based spatial economy on a procedural island, 1750 to 1900, with a map you can scrub through.

Nothing on the map is placed by hand. Households, landlords, investors and railway promoters act on the prices they can see, and the geography comes out of those choices: farm belts around market towns, ironworks moving onto the coalfields, roads where the carts go, railways where the roads are busiest. A separate observer watches the numbers and writes the news.

The name is von Thünen's: *Der isolierte Staat* (1826) is where the rings of land use around a market town were first worked out. Here they form on their own and then bend around rivers, roads and rails.

## Quick start

```bash
pip install -r requirements.txt          # numpy, scipy, numba, tqdm
python -m isostate build                 # runs both scenarios (about 3 minutes) and writes dist/isolated_state.html
python -m pytest -q                      # about half a minute
```

Open `dist/isolated_state.html` in any recent browser, phone included. It is one self-contained file: the runs are embedded in it, gzipped.

Other commands:

```bash
python -m isostate run --seed 11 --regime landlords --out runs/seed11      # one run, saved as run.bin + CSV + JSON
python -m isostate run --seed 11 --regime commons --out runs/seed11c --lite # charts-only comparison run
python -m isostate html runs/seed11 runs/seed11c --out dist/seed11.html    # first full run is the map, the rest are compared in charts
```

For poking at things interactively, `notebooks/explore.py` is a set of `# %%` cells.

## Layout

```
data/content.toml     goods, needs, sectors, production methods, inventions, discovery thresholds, all text
isostate/
  params.py           every tunable number, one plain dict
  world.py            island generation: elevation, rivers, soils, ore, coal, town sites
  transport.py        cost graph (terrain, roads, rivers, sea, rail), Dijkstra fields, traffic
  econ.py             the monthly step: demand, wages, production, prices, trade, money
  land.py             yearly land allocation by bid rent, urban growth
  firms.py            site evaluation, entry, expansion, closure, the "why here" records
  tech.py             discovery from cumulative use, diffusion along trade routes
  people.py           growth, migration, new towns
  rail.py             railway promotion and construction
  observer.py         series registry and the news detectors (never read by the sim)
  record.py           yearly snapshots and the binary export
  sim.py              init_state, yearly, run
  cli.py              the commands above
viewer/index.html     the single-file viewer; record.build_html embeds runs into it
tests/                money conservation, determinism, rent gradient, no hard walls, export round trip
notebooks/explore.py  notebook-style cells for analysis
```

## How the model works

**Monthly.** Each town is a market. Households and landlords split their budgets across six needs (food, warmth, shelter, clothing, implements, comfort) with a linear expenditure system: subsistence first, then fixed shares of what is left. Each need can be met by any good that has the property, at that good's efficiency, chosen by a logit on price per unit of service. That is the "no hard walls" rule: a little steel is bought as a better tool long before anyone rolls steel rails, so new goods always have some market. Wages clear each town's labor market by bisection. Land produces with Cobb-Douglas technology on the cell's share of land; firms use fixed-coefficient methods with learning curves. Prices move with inventories and with the flow imbalance, merchants take a spread, goods move between towns where price gaps beat the carting cost, and the money stock is checked every month against what was issued.

**Yearly.** Every cell's land is shared between farming, grazing, forestry, quarrying, clay, iron and coal in proportion to each use's bid raised to `1/theta`, adjusting part of the way each year. Bids are what the land earns at its market after carting, which is where the von Thünen rings come from. Towns grow outward by path cost. Investors in each town price candidate sites for every method they know and build where the return clears a hurdle; each firm keeps the sums that justified it. Roads upgrade where traffic is heavy. After the locomotive is invented, promoters raise money for the busiest town pairs and buy rails as they build, then add mineral lines. Population grows with real income and moves toward better wages; new towns appear where work grows up far from any market.

**Inventions.** No tech tree. Each method has thresholds in cumulative output of what it combines (coke smelting needs a lot of coal and iron handled; powered mills need machinery and cloth), expressed partly as multiples of 1750 output. Discovery is a hazard that rises steeply as the thresholds are approached, and the place it happens is drawn in proportion to how much of the ingredients each town handles. Know-how then diffuses along trade costs and productivity follows Wright's law.

**The observer.** It reads series and writes events: crossings (a new method passes half of output), doublings and orders of magnitude, cities passing round numbers, railway milestones, and steepness. Steepness is a year whose growth is far outside its trailing trend (a z-score on log growth), labelled by the quadrant of price and quantity: boom, glut, shortage or slump. Downturns run on two clocks: a live alarm when output falls, and a dated verdict in hindsight once the recovery is complete.

## The export format

`record.pack` writes one binary container per run: a little-endian `uint32` header length (including padding), a JSON header, then typed arrays aligned to 8 bytes. The header has names, events, firm records with their reasons, towns, series names and an `arrays` table of `{name, dtype, shape, offset, nbytes}`. Map layers are stored layer-major (`[year, cell]`) so years of the same layer sit together and compress well. The viewer reads it with `DecompressionStream` and typed-array views, no parsing of the bulk data. Nothing in it is specific to the web viewer; a game engine could read the same file.

## Scenarios

`rent_regime="landlords"` (default): land rent goes to landlords, who spend more of it on shelter, services and comfort, mostly in town.
`rent_regime="commons"`: the same rent is paid out to every household as an equal dividend.

The built page shows the landlords run on the map and compares the two in the charts. They are single runs on one seed; differences of a few per cent are within run-to-run noise.

## Known problems

- The share of people living in towns barely moves. Farm employment falls from about 65% to 45%, but industry's share stays near a quarter and towns do not pull people in as they did in Britain, so this is an industrious revolution more than an industrial one. The likely culprits are the food budget share staying high and farm productivity growth arriving too late.
- Farm employment cycles with a period of two or three years, visible as a zigzag in the charts: labor and land shares adjust on different clocks and overshoot each other.
- Some industrial prices (iron especially) swing much more than they should, a cobweb between capacity decisions and prices.
- The railway network is built in a rush in the 1830s and then stops growing at about 850 km once the best town pairs are linked.
- Discoveries cluster in the largest industrial town.
- One island, one seed, no foreign trade, no government, money grows at a fixed rate.
- Runs take about 80 seconds for 150 years, mostly in Python loops that could be vectorised or moved to numba.

See `PLAN.md` for where this goes next.
