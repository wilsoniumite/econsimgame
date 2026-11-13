# Isolated State

An agent-based spatial economy on a procedural island, 1750 to 1900, and a game played on it: you are one family among six.

Nothing on the map is placed by hand. Households, landlords, investors and railway promoters act on the prices they can see, and the geography comes out of those choices: farm belts around market towns, ironworks moving onto the coalfields, roads where the carts go, railways where the roads are busiest. A separate observer watches the numbers and writes the news.

The name is von Thünen's: *Der isolierte Staat* (1826) is where the rings of land use around a market town were first worked out. Here they form on their own and then bend around rivers, roads and rails.

![Melbourne, the player's home town on island 7, in the summer of 1860: a mill and furnaces by the river, the parish church, streets of houses, a stone bridge, and a train on the railway through the town](docs/shots/melbourne-1860.jpg)

The simulation and the game are written in Rust, in three crates. The model began as a Python package, which is kept here as the reference the Rust was ported from. This is early work. The island runs, and a game can be played from 1750 to 1900 and saved, but the part where you decide what your family does is not much fun yet. [Where it stands](#where-it-stands), at the foot, says what is wrong with it today.

## Quick start

You need a recent Rust toolchain (the workspace is edition 2024; everything here was built with Rust 1.97). On Linux you also need `pkg-config`, `libwayland-dev` and `libxkbcommon-dev`. Windows and macOS need nothing extra.

```bash
cargo run -p isostate-viewer
```

The first build compiles Bevy and takes some minutes. Then the terminal says which island is being made and where the game is kept, and a few seconds later a window opens over your home town with a card. On island 7, the one you get unless you ask for another, you are the Fairfax family of Melbourne: seven people, £18k in hand and 104 squares of land, with five other great families already at work across the island.

- Click to begin. The first decision is waiting: the family's trade. Every choice in the game is from a short list that says what each option would mean. There are no sliders.
- The top bar shows the month and the year, "January 1751" to begin with, over a thin bar that fills as the month goes by. **Next month** plays one month and stops, and **To January** runs on to the year's turn. **Space** starts and stops the clock, and Up and Down change its speed, from Slow to Fastest. A game starts at Steady, a month a second, and at that the 150 years take half an hour of running; at Fastest they take 75 seconds.
- What you order at a year's turn happens at once. Later in the year it waits for the year's end, and the Family panel lists what waits.
- Drag to pan, right-drag to turn and tilt, and use the wheel to zoom. **H** shows every key.
- Click a square near your seat to see what could be built there, with its cost and the return it expects at today's prices. Click a town, a works or a field to ask about it.
- Children are trained at fourteen, and an apprentice teaches the family a new trade. Grown members can be posted to run your trades in other towns, where their branches build on their own.
- **1** to **4** open the Family, Holdings, Markets and News panels. **M** washes a map view over the ground: rent, market areas, the way to market, prices, wages, when each square of a town joined it, or who lives there.
- The game saves itself after every year and every order. Start it again and it offers to continue. `--new` starts afresh, and `--seed 11 --families 4` gives another island with fewer rivals.
- In 1900 a card says where you stand among the families.

![A game in 1790: the Family panel with the Fairfax family's worth over forty years, last year's accounts and a member with nothing to run, and the note on Melbourne, the family's seat, listing its branches there](docs/shots/game-family-1790.jpg)

That command builds in Cargo's dev profile, in which this workspace still optimises Bevy and the simulation (see `Cargo.toml`). Adding `--release` gives a fully optimised build and takes longer to compile. The keys, the clock and its speeds, the panels, the map views, saved games and every option are in [crates/isostate-viewer/README.md](crates/isostate-viewer/README.md).

## Other things to run

Everything below writes under `runs/`, which git ignores.

**The simulation from the command line**, under either set of rules (the next section says what they are). A run to 1900 takes 15 to 20 seconds.

```bash
cargo run -p isostate-core --release -- run --seed 7 --out runs/port7                             # the port of the Python model
cargo run -p isostate-core --release -- run --seed 7 --rules game --families 6 --out runs/game7  # what a game is played under
cargo run -p isostate-core --release -- run --seed 11 --regime commons --lite --out runs/seed11c # the commons scenario, without the map layers
```

Each writes `run.bin` (the whole run, in the format described below), `series_monthly.csv`, `events.json` (the news) and `params.json`. There are also `--end-year Y`, `--params FILE` and `--content FILE`; `cargo run -p isostate-core --release -- --help` prints the list.

**A recorded run in the viewer.** Give the viewer a `run.bin` and it plays it back instead of starting a game:

```bash
cargo run -p isostate-viewer -- runs/game7/run.bin
```

Space plays and pauses, Left and Right step a year, and a click asks about a works, a town or a square, with the sums that put each works where it is. There is no family to play in a recording. `runs/port7/run.bin` shows the island under the port's rules, which is a different history of the same land.

**A sweep of seeds.** `sweep` runs many islands side by side on their own threads and boils each run down to its stylised facts, some 450 of them under the game's rules: people, wages, who earns what, towns, railways, canals, works, land and the diggings, inventions, price swings and the news. With `--compare` it sets the result beside an earlier sweep, seed for seed.

```bash
cargo run -p isostate-core --release -- sweep --seeds 1-8 --rules port --families 6 --out runs/sweep-port
cargo run -p isostate-core --release -- sweep --seeds 1-8 --rules game --families 6 --out runs/sweep-game --compare runs/sweep-port
```

Each takes about a minute for eight islands, all eight running at once. It prints a table of every fact's mean across the seeds with the least and greatest beside it, and then the comparison, of which this is a little:

```text
now: game rules, 6 families (8 runs); then: port rules, 6 families (8 runs) in runs/sweep-port
seed for seed, on seeds 1 2 3 4 5 6 7 8
of the 257 facts both have, the seconds aside, 253 differ and 144 of those are starred
...
                                          now     then            change  up/down
  urban_share_1900                      0.419    0.255            +0.165      8/0  *
  real_wage_1900_vs_1750                 2.60     1.35             +1.25      8/0  *
  rail_km_1900                          2,248    1,072  +1,176 (+109.7%)      8/0  *
  works_1900                            738.2    6,973   -6,234 (-89.4%)      0/8  *
  price_swing_iron                      0.172    0.202   -0.030 (-14.9%)      1/7
```

`up/down` counts the islands on which the fact is greater now and those on which it is less. A star means every island now stands clear of every island then. It also writes `annual.csv`, `facts.csv`, `facts.json`, `summary.txt` and a `README.txt` saying how they were made. `--set key=v1,v2` sweeps any parameter by name. [docs/rules/README.md](docs/rules/README.md) explains the table and each fact.

**Pictures with no window.** The viewer can play a game and write pictures without opening a window. The picture at the top of this page was made with the first of these on 11 November (and then saved as a JPEG); the clock's buttons and the towns have changed since, so today it draws the same corner of Melbourne as the game now has it. The second plays a whole game and writes twelve standard views of it (the island and the home town in 1750, 1800, 1850 and 1900, the town in winter, a street, the busiest works district) with an `index.html` to look at them in.

```bash
cargo run -p isostate-viewer -- --advance 110 --season 0.5 --view runs/melbourne-1860.png,,28.8,28.5,25,32,2.6,none
cargo run -p isostate-viewer -- --shots runs/shots
```

**The generators' lab.** Every house, works, church, bridge and tree is drawn by a plain function in `crates/isostate-gen`. The lab lays a contact sheet of them on bare ground in the game's own light, with no game, in two or three seconds:

```bash
cargo run -p isostate-viewer -- --lab list
cargo run -p isostate-viewer -- --lab cottages-eras --out runs/lab
```

**The tests.**

```bash
cargo test -p isostate-core --release    # 210 tests, four to six minutes once built; the golden test is among them
cargo test -p isostate-gen               # 127 tests, a few seconds
```

## Two sets of rules

The simulation runs under one of two sets of rules, and [docs/rules/README.md](docs/rules/README.md) is the page that explains them.

**The port** is a line-by-line port of the Python package. Every step was checked against the Python: deterministic steps agree to about 1e-14, and given Python's random numbers the stochastic steps make the same draws and reach the same decisions ([crates/isostate-core/PORTING.md](crates/isostate-core/PORTING.md)). It must stay exactly that, so nothing new is ever switched on under it. `crates/isostate-core/tests/golden.rs` runs seed 7 to 1900, once without families and once with six, and holds the monthly series and the news to stored hashes, byte for byte. It is what `isostate-core run` and `sweep` use unless told otherwise.

**The game's rules** are the port plus every mechanism the Python never had, each with a switch of its own, a page under `docs/rules/` and a sweep before and after. They are what a live game is played under, and `--rules game` picks them on the command line. Fifteen mechanisms are in, the last three from the pass over all the rules (`docs/designs/rules-pass.md`), each with its switch:

| Mechanism | What it does | Page |
|---|---|---|
| prices | Merchants mark a price by the stock they hold and stop buying into a glut, which ends two false cycles, in iron and on the farms. | [prices.md](docs/rules/prices.md) |
| engel | As a town grows richer its households spend less of each further shilling on food and warmth and more on comfort. | [engel.md](docs/rules/engel.md) |
| machines | Farms and quarries buy machinery where it pays, machines go on improving with the steel made, and converters and steel rails are made to pay. | [machines.md](docs/rules/machines.md) |
| harvest | Good years and bad on the farms, island-wide and by district, with the news of them. | [harvest.md](docs/rules/harvest.md) |
| chronicle | News that reads like a history: why each item is news, the few a decade worth keeping, and stories of towns and districts. | [chronicle.md](docs/rules/chronicle.md) |
| works | A life cycle for works. They grow where they stand and take land by their hands, so there are hundreds of works of every size, not thousands of small ones. | [works.md](docs/rules/works.md) |
| railways | Branch lines to towns and districts, each priced on the carting it saves, at several times the port's cost. | [railways.md](docs/rules/railways.md) |
| carting | Carting takes hands out of a town's labour supply, so cheaper carriage frees people for other work. | [carting.md](docs/rules/carting.md) |
| people | Births and deaths that answer to income, crowding and dearth, and towns that go on being founded where a market would save the most carting. | [people.md](docs/rules/people.md) |
| towns | A town keeps the ground it builds on and grows outward from it along its roads. | [towns.md](docs/rules/towns.md) |
| mines | A quarry, pit or mine is a pit head with the lease of the deposit round it. It pays a royalty and takes little of the surface, so the diggings hold a few cells of the island and not a fifth of it. | [mines.md](docs/rules/mines.md) |
| canals | A canal age before the railways. A canal is dug where the carting it would save pays for it, boats carry what goes far, and the traffic leaves for the railway when one comes. | [canals.md](docs/rules/canals.md) |
| hands (`hands.ration`) | A town cannot hire hands it does not have: when even the highest wage the month allows leaves more wanted than the town can spare, every employer there is cut by the same share, which ends the runaway wages of the mining towns. | [hands.md](docs/rules/hands.md) |
| money (`money.true_total`) | Each month's new money is reckoned on all the money there is as the month begins, so it grows 1.764 per cent a year on every island, where the port's count, taken while the month's incomes were between purses, gave 1.74 to 1.78 for a setting of 2. | [money.md](docs/rules/money.md) |
| shelves (`people.shelves_go`) | When a district changes town, its merchants' stock, cash and measure of sales go with it, as its people do, so a town that takes a district does not open January with the shelves of the town it was. | [shelves.md](docs/rules/shelves.md) |

Four more switches are not mechanisms of the economy, or not yet. The books of account (`accounts`, on under the game's rules) keep a year's books for every town and good and for every works, and an income account for the island; they watch and never touch the economy ([accounts.md](docs/rules/accounts.md)). Under the commons scenario a common fund in debt pays no dividend until its rents repay it, and charges nobody (`commons.carry_debt`, on under the game's rules, [commons.md](docs/rules/commons.md)). A second island generator, with rivers that run downhill, lakes and relief (`land`), is merged and switched off under the game's rules: the viewer draws part of it, and a game can be played on it as a preview (`--new-land`; [land.md](docs/rules/land.md), and the viewer's README, "The new land"). And roads, turnpikes and bridges that somebody pays for are a probe's prototype, off under both sets of rules (`roads.enabled`, [dx-roads.md](docs/probes/dx-roads.md)).

Families are older than this convention. They are off under both sets of rules until a run says how many there are: the viewer asks for six, and `--families 6` does it on the command line ([crates/isostate-core/FAMILIES.md](crates/isostate-core/FAMILIES.md)).

What the two give, as means over islands 1 to 8 with six families. The port's column is from its sweep above as run on 12 November 2026 (its last row from the books' own example, `cargo run --release -p isostate-core --example accounts -- --rules port`, since the port's sweep keeps no books); the sweep was run again on `main` at 973396e on 13 November and gave every figure above the same, since none of the newer switches is on under the port. The game's column is from the sweep of `main` at 38425b2 on 13 November 2026, with the pass's stages 1 to 4 in, which gave the island a new history:

| | the port | the game's rules |
|---|---|---|
| People in 1900 | 683 thousand | 734 thousand |
| Share of people living in towns, 1750 and 1900 | 0.26 and 0.26 | 0.22 and 0.42 |
| What a wage buys in 1800, 1850 and 1900 (1750 = 1) | 1.31, 1.35, 1.35 | 1.62, 1.92, 2.64 |
| Works standing in 1900 | 6,973 | 759 |
| The middling works of 1900, in its trade's usual size | 0.46 | 6.1 |
| Cells of town ground, 1750 and 1900 | 69 and 77 | 64 and 327 |
| Cells of works ground in 1900 | 981 | 201 |
| Land under quarries, pits and mines in 1900, in cells' worth | 1,188 | 9 |
| Railway in 1850, 1870 and 1900 | 1,057, 1,071 and 1,072 km | 1,058, 1,814 and 2,303 km |
| Canal in 1800 and 1900 | none | 682 and 950 km |
| The last town founded | 1802 | 1850 |
| Converter steel's share of steel in 1880 | none | 0.84 |
| News items in a run | 1,485 | 1,017 |
| Work's share of the island's income, 1750 and 1900 (from the books, `docs/rules/accounts.md`) | 0.69 and 0.60 | 0.71 and 0.71 |

## How the model works

This is the model the two sets of rules share. Where the game's rules do something else, a note in brackets says so.

**Monthly.** Each town is a market. Households and landlords split their budgets across six needs (food, warmth, shelter, clothing, implements, comfort) with a linear expenditure system: subsistence first, then fixed shares of what is left. (The game's shares shift with income.) Each need can be met by any good that has the property, at that good's efficiency, chosen by a logit on price per unit of service. That is the "no hard walls" rule: a little steel is bought as a better tool long before anyone rolls steel rails, so new goods always have some market. Wages clear each town's labour market by bisection. Land produces with Cobb-Douglas technology on the cell's share of land; firms use fixed-coefficient methods with learning curves. Prices move with inventories and with the flow imbalance, merchants take a spread, goods move between towns where price gaps beat the carting cost, and the money stock is checked every month against what was issued. (In the game merchants set prices from the stock they hold, the harvest varies, and the men who do the carting are not there for other work.)

**Yearly.** Every cell's land is shared between farming, grazing, forestry, quarrying, clay, iron and coal in proportion to each use's bid raised to `1/theta`, adjusting part of the way each year. Bids are what the land earns at its market after carting, which is where the von Thünen rings come from. (In the game the four kinds of digging do not bid for the surface. Each quarry, pit or mine is a working: a pit head with the lease of the deposit within 4 km of it, paying a royalty to whoever owns the land it works under.) Towns grow outward by path cost. (In the game a town keeps its ground and takes its best free neighbour when it is full.) Investors in each town price candidate sites for every method they know and build where the return clears a hurdle; each firm keeps the sums that justified it. (In the game a works that earns its keep grows where it stands, and each trade in each town has a year's room for new capacity.) Roads upgrade where traffic is heavy. (In the game a canal age comes first: once enough coal and stone are dug, promoters dig canals where the carting saved pays for them, with navvies taken from the towns' hands.) After the locomotive is invented, promoters raise money for the busiest town pairs and buy rails as they build, then add mineral lines. (In the game they go on to branch lines, each priced on the carting it would save.) Population grows with real income and moves toward better wages; new towns appear where work grows up far from any market. (Under the port the growth sits at its cap in nearly every year; in the game it answers to income, crowding and dearth.)

**Inventions.** No tech tree. Each method has thresholds in cumulative output of what it combines (coke smelting needs a lot of coal and iron handled; powered mills need machinery and cloth), expressed partly as multiples of 1750 output. Discovery is a hazard that rises steeply as the thresholds are approached, and the place it happens is drawn in proportion to how much of the ingredients each town handles. Know-how then diffuses along trade costs and productivity follows Wright's law.

**The observer.** It reads series and writes events: crossings (a new method passes half of output), doublings and orders of magnitude, cities passing round numbers, railway milestones, and steepness. Steepness is a year whose growth is far outside its trailing trend (a z-score on log growth), labelled by the quadrant of price and quantity: boom, glut, shortage or slump. Downturns run on two clocks: a live alarm when output falls, and a dated verdict in hindsight once the recovery is complete. The simulation never reads what the observer writes. (In the game the chronicle marks the few items of each decade that a history would keep, and tells the stories of towns and districts.)

**Families.** With families switched on, six families own works and land and grow through their people: children trained at fourteen, grown members posted to run a trade in a town, branches that build on their own. They act through the same prices and bids as everyone else, and every flow of money is a transfer the monthly check counts. One of the six is the player's; the others play by the same rules. None of this is in the Python.

![Island 7 in the summer of 1900, seen whole: woods as dark masses, the snow on the mountain, rivers running to the coast, and the names of the larger towns](docs/shots/island-1900-far-view.jpg)

The island is 120 by 84 cells of 2 km. The content of the economy is one file, `data/content.toml`: sixteen goods, six needs, seven uses of the land, sixteen production methods, five further inventions (steam pumping, wagonways, railways, improved husbandry, farm machinery) and all the text. The game's rules add to it in code (`game_content`), never in that file, which is the Python's.

## The export format

`record.pack` in the Python and `record::pack` in the Rust write the same binary container, one per run: a little-endian `uint32` header length (including padding), a JSON header, then typed arrays aligned to 8 bytes. The header has names, events, firm records with their reasons, towns, series names and an `arrays` table of `{name, dtype, shape, offset, nbytes}`. Map layers are stored layer-major (`[year, cell]`) so years of the same layer sit together and compress well. The web viewer reads it with `DecompressionStream` and typed-array views, with no parsing of the bulk data; the 3D viewer reads the same file in `crates/isostate-viewer/src/replay.rs`. A run under the game's rules adds three layers for the towns (the year each cell joined its town, its residents, and the share of each cell under works) and a table of the workings of each year (where each pit head stands, its hands, what it raised and its spoil heap).

A saved game is something else: the seed, the rules and a log of the player's orders, played again from 1750 when it is loaded (`crates/isostate-core/src/save.rs`).

## Scenarios

`rent_regime="landlords"` (default): land rent goes to landlords, who spend more of it on shelter, services and comfort, mostly in town.
`rent_regime="commons"`: the same rent is paid out to every household as an equal dividend.

On the command line it is `--regime commons`, in the Python and the Rust alike. A sweep runs both on many islands at once:

```bash
cargo run -p isostate-core --release -- sweep --seeds 1-8 --rules game --families 6 --set rent_regime=landlords,commons --out runs/sweep-regimes
```

As run on 12 November 2026, all sixteen runs finish. The commons islands end the century with more people (833 thousand against 732 thousand), a lower wage (2.46 times 1750's against 2.60, though a commons household also has its dividend, which the wage does not count), half the railway (1,148 km against 2,248) and two thirds of the canal (600 km against 892). Famine years are the same, 0.4 a run. Nobody has yet traced why, so take these as what the model does and not as findings.

One rule of the commons differs under the game's rules. When the common fund is in debt (a mine's bad year can put it there) it pays no dividend until its rents have repaid the debt. In the Python and the port it charges the debt to every household by head, and on island 3 that emptied every purse and stopped the run in 1762 (`docs/rules/mines.md`, "Under the commons").

## Layout

```
Cargo.toml              the Rust workspace: the three crates under crates/
crates/isostate-core/   the simulation, in Rust: a library and the isostate-core command
  src/                  one module per Python module (world, transport, econ, land, firms, tech, people,
                        rail, observer, record, sim, cli), then what the Python never had: families and
                        the live API (families.rs, live.rs), saved games (save.rs), the sweep and its
                        facts (sweep.rs, facts.rs), and a module for each mechanism of the game's rules
  tests/                the golden test, the step-by-step parity tests, and tests for each mechanism
  examples/             scripted players (bots.rs), censuses of works, towns and mines, timing, probes
  parity/               Python scripts that dump the Python's state for the parity tests
  README.md             what the crate is, how to run it, and where each part is explained
  FAMILIES.md           how families work, the live API, and what the scripted players found
  PORTING.md            the port: the mapping, every deviation, the parity tables, oddities in the Python
crates/isostate-gen/    the generators: plain functions that turn what the simulation knows into
                        houses, works, churches, bridges and trees (no Bevy; its README says how)
crates/isostate-viewer/ the game and the 3D viewer, in Bevy 0.20 (its README has the keys, the options
                        and how the picture is made)
  src/                  the ground and sea shaders, rivers, fields, railways, the towns' layout,
                        the interface, map views, charts, saved games, the standard shots and the lab
  assets/look.toml      every colour of the scenery, the light and the lens, in one file
data/content.toml       goods, needs, sectors, production methods, inventions, discovery thresholds, all text
docs/                   the devlog, the backlog, the art plan, and the rules, designs and probes (below)
docs/shots/             the pictures the documents show
PLAN.md                 the plan, in three stages

isostate/               the original Python model, kept as the reference
  params.py             every tunable number, one plain dict
  content.py, names.py  reading data/content.toml; names for towns, works and people
  world.py              island generation: elevation, rivers, soils, ore, coal, town sites
  transport.py          cost graph (terrain, roads, rivers, sea, rail), Dijkstra fields, traffic
  econ.py               the monthly step: demand, wages, production, prices, trade, money
  land.py               yearly land allocation by bid rent, urban growth
  firms.py              site evaluation, entry, expansion, closure, the "why here" records
  tech.py               discovery from cumulative use, diffusion along trade routes
  people.py             growth, migration, new towns
  rail.py               railway promotion and construction
  observer.py           series registry and the news detectors (never read by the sim)
  record.py             yearly snapshots and the binary export
  sim.py                init_state, yearly, run
  cli.py                the Python's commands
viewer/index.html       the single-file web viewer; record.build_html embeds runs into it
dist/                   where the Python writes the web viewer's pages (git ignores them)
tests/test_sim.py       the Python's tests: money conservation, determinism, rent gradient, no hard walls,
                        export round trip
notebooks/explore.py    notebook-style cells for analysis
requirements.txt, pyproject.toml   the Python package's dependencies
```

## The Python original

The model was first written in Python, and the Rust core is a port of it. The Python stays in the repository, unchanged, as the reference: the port's rules are held to it, and the parity tests compare the two step by step. It is also still the only thing that builds the web viewer, a single self-contained page with the runs embedded in it.

```bash
pip install -r requirements.txt          # numpy, scipy, numba, tqdm, and pytest and pandas for the tests and the notebook
python -m isostate build                 # runs both scenarios and writes dist/isolated_state.html
python -m pytest -q
```

Open `dist/isolated_state.html` in any recent browser, phone included. Other commands:

```bash
python -m isostate run --seed 11 --regime landlords --out runs/seed11      # one run, saved as run.bin + CSV + JSON
python -m isostate run --seed 11 --regime commons --out runs/seed11c --lite # charts-only comparison run
python -m isostate html runs/seed11 runs/seed11c --out dist/seed11.html    # first full run is the map, the rest are compared in charts
```

`python -m isostate html` reads a `run.bin` written by the Rust as well. For poking at things interactively, `notebooks/explore.py` is a set of `# %%` cells. A Python run takes 80 to 120 seconds, by the machine, where the Rust takes about 20 (PORTING.md, "Timing"), and a seed gives a different island in Python than in Rust, because the random streams differ; across 16 seeds the two agree within chance.

## The documents

- [PLAN.md](PLAN.md): the plan, in three stages: a lab, a visual tool, a game.
- [docs/DEVLOG.md](docs/DEVLOG.md): what happened, newest first. Its newest entry is the best account of what exists.
- [docs/BACKLOG.md](docs/BACKLOG.md): what is waiting, what is being done and why in that order. The pages under `docs/backlog/` hold what helpers and reviewers found along the way and what waits on a decision.
- [docs/rules/](docs/rules/README.md): the port and the game's rules, the golden test, the sweep and its facts, and a page for each mechanism with its numbers, what the sweeps showed and what worries us.
- [docs/designs/](docs/designs/README.md): designs worked out and argued over but not yet built, or only in part. Today, a deeper production tree; the pass over all the game's rules, with its scorecard of twenty aims and eleven stages, of which four are built; and a review of the code's architecture, which finds no large refactor due and sets out nine small changes in order.
- [docs/probes/](docs/probes/README.md): measurements of the simulation and the viewer, each made on eight or more islands and kept as written, of four kinds. Probes diagnose something and design a cure: the game's first mechanisms were built from five of them, and the later ones are of combined works, where the frames stall, what it takes to draw the second island generator's land, a saved game that holds the island, what the record carries for the viewer, an inspector that shows a market's trades as a flow, towns at wharves, junctions and pit heads, a harvest that comes in a month, and roads and bridges that somebody pays for. Nine traces with the books of account follow things the sweeps had measured and nobody had explained: rent, money, carriage and famine, growth that stops, the steel rail, the building cycle, the towns' share, and two sets of four loose ends (the merchants' debts and runaway wages among them). Six tables of sensitivity say what every number of twelve mechanisms moves, and a baseline of the scorecard gives the floor of chance it is read against. And one reads a century of the news as a player meets it.
- [docs/ART.md](docs/ART.md): the plan for the look: generators and not asset files, and change you can see happen.
- [crates/isostate-viewer/README.md](crates/isostate-viewer/README.md): playing, the keys, every option, saved games, and how the picture is made.
- [crates/isostate-gen/README.md](crates/isostate-gen/README.md): what a generator is, the kits the houses and works are built from, and how to add one.
- [crates/isostate-core/FAMILIES.md](crates/isostate-core/FAMILIES.md): families, the live API, and what scripted players found.
- [crates/isostate-core/PORTING.md](crates/isostate-core/PORTING.md): how the Rust was ported from the Python and checked against it.

## Where it stands

The game's numbers below are means over islands 1 to 8 with six families, from the sweep of `main` at 38425b2 on 13 November 2026, with the first four stages of the pass over the rules in, unless a page is named; the port's are from 12 November, and the same when run again on 13 November.

**What exists.** The simulation in Rust under both sets of rules, with a sweep to measure a change on many islands and a scorecard of twenty aims to read it against. A game that can be played through the century and saved: decisions, a seat, branches, works built and land bought, four panels, seven map views, charts, the news. A picture drawn entirely by code, with no model or image files, from the whole island down to a street. On 12 November came canals, books of account, a clock that tells the month, towns drawn from the ground they keep, works built from the houses' kit at one scale of detail, winter on what is built, saved games that a test start cannot lose, and a second island generator with real rivers and lakes. On the night of 12 to 13 November came:

- the new island in the picture, as a preview: a game started with `--new-land` is played on the land made by water, with the ground, rivers, lakes and sea drawn from the core's own heights at three times the relief, each cell's meeting place on dry, level ground, and railways that find their way through their cells by the ground. The land stays switched off under the game's rules, and its roads, river banks, quays and towns on slopes are not drawn yet (the viewer's README, "The new land");
- works on walled plots of their own, laid out from what they do and hold, and standing cold and then derelict for some years after they close (`crates/isostate-gen/README.md`, "Works on their plots");
- carts that keep to the road as drawn and cross a bridge on its deck;
- the towns' second pass, in which houses are told their street and their age and vary by them, and a house told nothing new is the house it was;
- the news's plain faults mended (`docs/rules/chronicle.md`);
- the first four stages of the pass over all the rules (`docs/designs/rules-pass.md`): the scorecard's twenty aims; a town that cannot hire hands it does not have; money issued on its true total; and the merchants' shelves that go with a district when it changes town. The last three each gave the island a new history;
- a golden test under the game's rules, beside the port's (`docs/rules/README.md`, "The game's golden test").

The work in hand is listed in the backlog: the pass's fifth stage and two mends on the merchants' side, then the deeper production tree with its panels, and then the pass's later stages. The drawing of mines and canals waits on one table of a year's meshes, which is begun.

**The game.**

- The core loop is not fun yet. It is the designer's to think about, and may be adjusted where it stands in the way of other work. Scripted players show the shape of it: buying land beats building works on all four islands they were tried on, a family that takes every default ends fourth of six on three islands of four, and a player who founds a works every year loses most of them (FAMILIES.md, `docs/rules/works.md`, `docs/rules/towns.md`).
- A saved game is its orders played again, so it does not survive a change to the simulation (each stage of the pass over the rules is such a change), and loading a whole century takes about 23 seconds before the window opens. A picture of the island's state written beside the orders and read back goes on as the same game to the last digit, so a save that loads at once is measured and designed (`docs/probes/dx-save-state.md`), not built.
- A game on the new land is a preview: it is never kept in the island's autosave unless `--save FILE` names a file, and nobody has yet played it at a window.
- The clock tells the month since 12 November, and a game starts at a month a second, but the game is still played by the year: an order given in the middle of a year waits for its end, and decisions come only at a year's turn, so their cards always sit over a winter island.
- There is no title screen and no sound.
- The politics half of the game is not started.

**The economy under the game's rules.**

- Growth comes early and then stops: output per head peaks in 1884 on average (1869 to 1898). The cause is traced. The farms, quarries, clay pits and mines all take their gains from one curve, the machines' quality, and it comes within a hundredth of its ceiling of 4 (`machines.quality_cap`) about 1870. Lift the ceiling and growth goes on past the 1870s on every island of twenty-four; lower it and the stall comes sooner. What falls after the peak is the observer's figure, which deflates by the basket of 1750; counted in quantities, output per head stops rising and does not fall (`docs/probes/tr-works.md`, `docs/probes/sens-machines.md`, both measured on the build of 12 November). The pass over the rules lifts the ceiling to 12 at its stage 8, after the production tree.
- The share of people in towns reaches 0.42 by 1900 (0.416), where the plan asks for a half; on the scorecard it is the third largest part of the loss, after the first railway's date and the tools' dear years. It is a count, not where anybody lives: under both sets of rules it is 0.04 plus eight tenths of the share of the hands that work in works, so 0.416 is 47 per cent of the hands in works, and a half would need 57.5. What holds it is where the rest work: the quarrymen most (with stone taken out of shelter the share is 0.47), then the carters and the Engel curve. Town ground, migration and the number of towns do not hold it. The founding of towns stops about the middle of the century (the last is founded in 1850 on the mean island, and between 1798 and 1876 by island), because a site is weighed by the carting it would save, at a freight fixed in money, against money wages that treble over the century (`docs/probes/tr-urban.md`). The pass makes the freight follow the wage at its stage 6, after the tree.
- There is a building cycle nobody chose: the mills and, a year behind them, the brickworks order a whole year's room for new capacity at once and then next to nothing for two or three years, in step across the island; timber's price swings 0.20 a year with a period of about four years and its dearest month of the century is two and a half times the usual. The worst year for what a wage buys is 15 per cent down (`docs/probes/tr-cycles.md`).
- Tools run short when a wave of engine works is built (20 tools a unit of plant) on a trade households had already left for iron and steel: the dearest year is 5.2 times the median on sixteen islands, most often in the 1880s and 1890s, and the smithies can grow only at their year's room while earning eight times the hurdle (`docs/probes/tr-cycles.md`).
- Famine is nearly gone: 0.1 famine years in a run, one year on one island of eight. Until 13 November there were 0.4, and three in four were a month's empty shelf, most often in January, in a small town that a redrawn district had just doubled or trebled: its people came with their cells and the merchants' stock and their record of sales did not (`docs/probes/tr-carriage.md`). The pass's third stage sends the merchants' stock, cash and measure of sales with a district to whichever town takes it (`people.shelves_go`, `docs/rules/shelves.md`): on sixteen islands, with the second stage in, famine years fall from 5 to 1 and emigrants from 74 a run to 8, and no town goes short after taking a district. The hunger left is a town that gives its farmland to a neighbour and keeps a share of a grain store its own fields no longer fill (`docs/probes/tr-loose-b.md`, section 4).
- The steel rail hardly arrives: a twentieth of the rails of 1890 (18 per cent on the island that makes most), and on one island of eight it is never made. The cause is traced: the steel rail mill starts at half the rolling mill's skill (`machines.inherit`) and never rolls enough rails to learn past it, so its hands cost twice as much a rail; given the rolling mill's skill alone it takes nine tenths of the rails by 1880. The mines' cheaper ore made it worse, since it cheapens the iron rail more than the steel mill's costs (`docs/probes/tr-steel.md`). The pass gives the steel rail mill the rolling mill's whole skill at its stage 8, and says that with the tree in the steel rail will still fall short by 1890.
- The first line opens anywhere from 1804 to 1852, so in 1850 one island has no railway and another 1,741 km. The date is the draw's: the first line opens about eight years after the locomotive is found, and the yearly chance of an invention is shallow on both sides of its threshold, so no setting of the railways moves the date by more than two years (`docs/probes/tr-steel.md`, `docs/probes/sens-ways.md`). The pass makes the chance steeper and quicker at its stage 7. Nobody collects a railway's fares, and nobody pays for a road or a bridge: parishes and turnpike trusts that do are prototyped behind `roads.enabled`, which is off under both sets of rules, and switched on they move who pays and what is built but not the carting bill or the wage beyond chance (`docs/probes/dx-roads.md`).
- Quarrymen are about one hand in seven by 1900 (0.135 of all hands at work in December 1900, carters among them, in the sweep's `hands_quarry_1900`) and the clay pits 0.022 more, and the island takes some twenty units of stone a head a year, which is far too much if a unit is about a ton. Households buy shelter as raw stone, because the masons, bricklayers and carpenters who belong between the quarry and the house are designed and not yet built (`docs/rules/mines.md`, "Why quarrying takes so many hands"). The sweep's `mine_share` is not that count: it is what is left of the hands when the land's and the works' are taken out, so under the game's rules it holds the carters and the navvies too, and of its 0.215 in 1750 only 0.090 is quarries, pits and mines (`docs/rules/engel.md`, "Facts added to the sweep"). Since the pass's first stage the sweep also gives that true share, `diggings_share_Y`: 0.090 in 1750 and 0.205 in 1900.
- A town no longer hires hands it does not have. Until 13 November a mine could: after a price jumped at a year's turn it hired two and a half to six times its town's hands, and the town's wage doubled month after month to its ceiling, up to 121 times the middle town's (`docs/probes/tr-loose-a.md`, `docs/rules/mines.md`). Since the pass's second stage (`hands.ration`, `docs/rules/hands.md`) every employer in such a town is cut by the same share to the hands there are: on the runs where the wage ran away it now reaches 1.3 to 2.1 times the middle town's, and on 24 islands no town-month has more at work than the town has people, under the landlords or the commons.
- Six tables of sensitivity have moved every number of twelve mechanisms to three quarters and four thirds of its value (`docs/probes/sens-*.md`). A few numbers matter a great deal (the machines' ceiling and `inherit`, Engel's `y_ref`, the people's `growth_top` and `y_still`, the mines' `reach`, the works' `year_room` for how rough the century is, carting's `labor` for its first fifty years) and many do nothing at these sizes; the pass's fifth stage, being built, makes sixteen of those constants. They also found that by 1900 a change too small to matter moves an island as far as the islands differ from one another, so eight islands is a small sample and a single year's wage misleads: read twenty-year means, and a setting's two values against each other, before believing a row (`docs/probes/sens-mines.md`, `docs/rules/towns.md`). A sweep gives the twenty-year means as facts, and the scorecard reads it against the pass's twenty aims, each with its tolerance written beside it (`docs/rules/scorecard.md`), over a floor of chance measured from histories that differ by nothing else (`docs/probes/scorecard-baseline.md`). On today's eight islands nine aims of twenty are met; the loss is 43.5, about a fifth each from the first railway's date, the tools' dear years and the towns' share. There is no search.
- No foreign trade and no government. Money grows at a fixed rate. Since the pass's fourth stage each month's issue is reckoned on all the money there is as the month begins (`money.true_total`, `docs/rules/money.md`), at 0.0175 a year, so money grows 1.764 per cent a year on every island, the rate the islands had when the port's count, taken while the month's incomes were between purses, gave 1.74 to 1.78 for a setting of 2; prices moved no further than chance. A run whose money goes below nothing stops and says so. Under the game's rules the merchants owed about half the island's money by 1900 on the build of 12 November. It is traced: the price rule marks up services, which spoil, and a town that brings its services in gets months of them at once and loses what rots; with no markup on services the debt is 1.4 per cent of the money and not 45 (`docs/probes/tr-loose-a.md`). That mend is being built.
- Run past 1900, the island goes Malthusian, because there is no demographic transition and no technology after steel. On island 7 under the game's rules with six families and `--end-year 2050`, as run on 12 November, the people grow from 740 thousand in 1900 to 4.3 million, and what a wage buys falls from 2.65 times 1750's to two thirds of what it bought in 1750.

**The port's rules.** These are the Python model's own weaknesses. The port keeps them on purpose, and the game's rules are the answer to most. On eight islands without families (`docs/rules/README.md`, "What the port's baseline says"):

- Towns do not pull people in: the share living in towns is 0.26 in 1750 and 0.24 in 1900.
- What a wage buys rises by a third before 1800 and then stops.
- The railway is built in a rush and stops at about 1,020 km.
- Works stay small and many: 4,500 by 1900, and 7,000 with six families.
- Iron and rail prices swing two and a half times as hard as grain and cloth, and farm employment zigzags with a period of two or three years.
- The converter is found and hardly ever built, so cheap steel never arrives. No town is founded after about 1800.
- People multiply at the growth cap in nearly every year, whatever their standard of living.
- Quarries, pits and mines take a fifth of the land: 1,190 cells' worth by 1900, nearly as much as the farms.
- PORTING.md lists ten things in the Python that look like bugs. The port does what the Python does in nine of them; in the other, ties in a sort, it breaks them by index where the Python's order depends on the machine. Under the game's rules one of the ten, the money issued on a total taken while the month's income is between purses, was the slow growth of money described above, and the game's rules now issue on the true total; of the other nine none matters there (`docs/probes/tr-money.md`).

**The picture and the interface.**

- A town is drawn from the ground it keeps, one house to twelve residents, and its frontages join before they rise. Since the towns' second pass a house is told its street, the age it was raised in and the age it got a new front, and a street's houses share a family of walls, tiles and habits; a house told nothing new is the house it was, which two tests hold. Houses that go over 150 years on island 7 fell from 5,042 to 4,015. Still wrong: harbour towns draw fewer houses than their people ask for (Granness 1,119 of 1,505 in 1900), 1,132 townhouses go back to cottages over the century when a neighbour goes, the home town of 1900 at play distance is ribbons of houses among closes and not a mass, and the closes of hill towns are the lowland's green (a hill tone is in the look file, switched off) (`docs/DEVLOG.md`).
- A quarry, pit or mine is drawn as one small pit on its cell, a stand-in. Pit heads, spoil heaps to their size, and idle or abandoned workings are not drawn yet (`docs/rules/mines.md`, "What the viewer is given"), and a canal is not drawn at all.
- The roads and bridges are the viewer's own invention, so a road down a valley crosses its river again and again. Since 13 November a cart keeps to the road as drawn and crosses a bridge on its deck; where a bridge's ramp ends above its road the carter turns back (the viewer's README, "Carts on the roads").
- The railways climb like railways since 12 November, and some of it is still wrong: on island 7 in 1900 four churches have the line hard against their yard, eighteen curves are tighter than a train should take (a branch that joins from the side comes round in a hook), roads drape over the banks with no bridge or crossing, and sidings are plain ribbons (`docs/DEVLOG.md`).
- A works stands on a walled plot of its own, laid out from what it does and holds: a gate, a counting house, a store for each input, stock yards, a tip, a siding or a quay where it reaches the line or the water, and a range for each stage of its growth. A works that closes stands cold for two years and derelict to its sixth before it is cleared, by the viewer's rule, since the record keeps only the year it closed. Still wrong: in a live game a works that outgrows its plot is planned afresh and its buildings shift; in a recorded run a works of 1750 keeps clear of a railway laid a century later, so any change to the railways moves it; and nobody has judged whether small works of different kinds can now be told apart from the distance the game is played at.
- On the new land the ground, the rivers, the lakes and the railways are drawn from the core's own land, and much is not: roads still run straight between the cells' meeting places (one that would lie under a lake or along tidewater is left out), and river banks, shores, marsh, quays and boats, towns on slopes and crags are not drawn; heights are held at three times the relief until they are in metres. Its railways have half the viaducts they had when a line passed each cell beside its middle, but on island 7 in 1900 still more viaducts, tunnels and inclines than the old island (113, 24 and 56 against 95, 18 and 28), and one branch crosses a corner of Carford's lake (the viewer's README, "The new land"; `docs/probes/dx-land-view.md`).
- A year's scenery is still rebuilt whole, off the main thread, but since 12 November it is handed to the graphics card one mesh at a time under a budget, and a year is shown whole or skipped whole. With no window the longest frame at a year's turn fell from 460 to 480 ms to 35 to 52 in 1751, and from 350 to 470 to about 90 to 110 in 1850, where what is left is the frame the year arrives in; at the fastest speed no frame passes 100 ms where one did at nearly every year, and a year is shown about one time in two and a half (`docs/probes/dx-smooth.md`, section 12). Nobody has yet watched it in a window, and the later stages (the year's arrival off the main thread, pieces of bounded size, sending less) are designed and not built.
- A recorded run of a game replays without its owners, so with no pennants or fences.

**The code and its tests.**

- The golden hashes, the port's and the game's, were taken on Windows with the MSVC toolchain. Another system's maths library may round a last digit the other way, and the tests have not been tried on one.
- The port's golden test watches the economy and not the map. Since 13 November the game's golden test (`tests/golden_game.rs`) runs seed 7 to 1900 under the game's rules with six families, twice side by side, and holds four hashes: the economy's series, the books' series, the news, and `run.bin` with its date blanked, so a change to what the record carries for the viewer under the game's rules now fails it, and says which of the four moved. A stage meant to move the game takes the hashes again on the merged tree and says so (`docs/rules/README.md`, "The game's golden test"). A test of the record's layout itself, with a row for each works, is the first stage of the next layout designed in `docs/probes/dx-record.md`, and is not built.
- A test whose case is one island's history under the game's rules breaks whenever a later rule changes that history, as happened four times on 13 November. Those tests now play their case with the newer rule off and say so in their comments.
- The parity tests need fixtures dumped from the Python, about 3 GB, which are not in the repository. Without them they pass and check nothing, and the golden test stands alone.

[docs/BACKLOG.md](docs/BACKLOG.md) has the order of work, and [docs/backlog/found-along-the-way.md](docs/backlog/found-along-the-way.md) the long list these were drawn from. [PLAN.md](PLAN.md) says where it is all meant to go.
