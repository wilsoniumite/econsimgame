# Plan

Three stages: a lab you can tinker with, a visual tool you can steer, then a game people step into. Detail is front-loaded on purpose; later stages will be rewritten once the earlier ones teach us what matters.

## Decisions so far (November 2026)

- **Rust all the way.** The simulation moves to a Rust crate (`crates/isostate-core`), ported module by module and checked against the Python version, which stays as the reference and the notebook lab. The 3D client is Bevy (`crates/isostate-viewer`). Desktop first.
- **Look: a cosy diorama in the spirit of Tiny Glade.** Character comes from palette, silhouette, light and small moving life, not from hand-drawn assets, so it survives an island that rebuilds itself every year.
- **Buildings feel placed, the economy stays smooth.** Under the hood the economy works in shares and densities per 2 km cell. The viewer turns those into stable things: each firm's works is a compound in a yard of its own that grows by adding parts, in steps you can see, and every tree, house and farmstead has a fixed slot that fills in once its ground passes a fixed threshold. Nothing reshuffles from year to year.

## Where we are

As of 12 November 2026. The repository's main branch is the Rust game, with the Python model kept beside it as the reference it was ported from.

The simulation (`crates/isostate-core`) runs under two sets of rules (docs/rules/README.md). The port is the Python's run, checked against it step by step and held byte for byte by a golden test; 150 years take 15 to 20 seconds. The game's rules are the port plus eleven mechanisms the Python never had, each with a switch and a page of its own: merchants' prices, an Engel curve, machines, harvests, a chronicle, a life cycle for works, railways that keep growing, carting that takes labour, people who answer to how they live, town ground that is kept, and mines that take little of the surface. A sweep runs many seeds in threads and boils each run down to its stylised facts.

The Bevy viewer (`crates/isostate-viewer`) plays recorded runs and live games, and the scenery's generators have a crate of their own (`crates/isostate-gen`) with a lab. A live game has six families, the player's first, and saves itself. The player chooses from short lists: the family's first trade, each child's training, where grown members run its trades, what each branch does, where the seat is, and, at the seat only, which works to build and which land to buy (see `crates/isostate-core/FAMILIES.md` for the rules and docs/DEVLOG.md for what exists). The look is planned in docs/ART.md.

Of the economic weaknesses listed under stage 1, the game's rules answer most and have brought some of their own; a note under each part of that stage says where it stands, and README.md, "Where it stands", has the list as it is now. The port keeps every one of them, on purpose. The core loop of the game is not fun yet and waits for thinking of its own, and the politics half is not started. What is in flight and what comes next, and why in that order, is in docs/BACKLOG.md.

## Stage 1: the lab (next, dense)

**1.1 Make runs cheap enough to sweep.**
- Profile the monthly step; move wage bisection, trade and land allocation into numba or fully vectorised numpy. Target: 150 years in under 15 seconds.
- A batch runner: `python -m isostate sweep --seeds 1-32 --set land_theta=2,3,4` writing one Parquet table of annual series per run, with tqdm over runs.
- Seed robustness as a standing check: every headline claim in the README should hold in most seeds, not just seed 7.

*Where this stands (12 November 2026):* done in Rust and not in the Python. The port in `crates/isostate-core` runs 150 years in 15 to 20 seconds, by what else the machine is doing, so the target of under 15 is not yet met. The batch runner is `isostate-core sweep` (docs/rules/README.md): seeds against `--set` values in threads, eight seeds in about a minute, written as CSV and JSON and not Parquet. The README's numbers are now means over eight seeds.

**1.2 Fix the economics we already know are off.**
- Structural change: let the food share fall with income (Engel), make farm productivity respond earlier to tools and husbandry, and let towns bid labor away with an urban wage premium. Success test: urban share rises from about 0.25 to 0.5 over the run in most seeds.
- Price volatility: add capacity expectations (firms see the pipeline of capacity under construction) and a slower investment response; check iron and rails no longer cycle with a period of a few years.
- Railways: let promoters see the traffic a line would induce, not just today's flows, and allow branch lines from any railed town, so the network keeps growing after the first rush instead of stopping at about 850 km.
- Damp the two-to-three-year cycle in farm employment (labor and land shares overshoot each other).
- Foreign trade as a world price for a few goods at the ports, so the island can specialise.

*Where this stands (12 November 2026):* the list above describes the Python model, and the port still does all of it on purpose. Each item but the last now has a mechanism under the game's rules, with a page under docs/rules/. The figures are means over seeds 1 to 8 with six families, from sweeps run that day.

- Structural change (`engel`, `machines`, `carting`, `people`): the share of people in towns goes from 0.22 to 0.42, on every seed between 0.41 and 0.44, so the test of 0.5 is not met. What a wage buys in 1900 is 2.61 times 1750's, where the port's is 1.35.
- Price volatility (`prices`): the swings in iron and the cycle in farm employment turned out to be one oscillator in the price rule, cured by letting merchants mark prices on the stock they hold. Capacity pipelines and slower investment were tried and did not help (docs/probes/dx-cycles.md). Iron swings 0.17 where the port's swings 0.20, and the farm zigzag is under half. A building cycle has appeared in its place (docs/rules/works.md).
- Railways (`railways`): branch lines from any junction, each priced on the carting it saves. The network is 1,240 km in 1850, 1,850 in 1870 and 2,260 in 1900, and is still growing in 1899 or 1900 on every seed. Under the port it stops at 1,020 to 1,070 km; the 850 km above was the Python's seed 7.
- Foreign trade: not started. It is in the third stage of the production tree's design (docs/designs/production-tree.md).

**1.3 Calibration harness.**
- A small table of stylised facts with tolerances (population growth, real wage path, coke iron share by 1800, mill cloth share by 1830, rail km by 1870, urban share by 1900).
- A loss over those, and a simple search (random or CMA-ES) over a handful of parameters, run on many seeds.
- Keep parameters in `params.py` and content in `content.toml`; calibration writes a new params file, never edits code.

*Where this stands (12 November 2026):* the table of facts exists, some 320 for a run under the game's rules (`crates/isostate-core/src/facts.rs`), with a seed-for-seed comparison between two sweeps. The tolerances, the loss and the search do not. In the Rust the numbers live in `params.rs` and in each mechanism's own struct, and `--params FILE` or `--set` lays values over them without editing code.

**1.4 Observer quality.**
- Store event provenance (which series, which test, which threshold) so a reader can always ask "why is this news".
- Rank events per decade so the major feed reads like a history, around 2 to 4 items a decade.
- Detect regional stories (a coalfield boom, a town in decline) from cell and town data, not just national series.

*Where this stands (11 November 2026):* all three are in under the game's rules, as the chronicle (docs/rules/chronicle.md). Nobody has yet watched it in the running game.

**1.5 Tests to add.**
- Land shares sum to one; no negative stocks; every firm's reason record matches its site.
- Golden-run regression: annual series for seed 7 within tolerance of a stored copy, updated deliberately.

*Where this stands (11 November 2026):* the golden test is in (`crates/isostate-core/tests/golden.rs`), stricter than asked: seed 7's monthly series and news are held to the byte under the port's rules, with and without six families. `tests/run.rs` checks on a short run that no stock is negative and that no cell's land shares pass one, and `tests/works.rs` that no cell holds more works than it has ground for. That every firm's reason record matches its site is checked only in the parity tests, which need the Python's dumps.

## Stage 2: the visual tool (sparser)

- Done: replays and live play in Bevy, a terrain shader for fields, hedges and crops (crisp at any zoom, with the seasons), relief, meandering rivers, coasts and a sea, enclosure and open fields, graded railways with their engineering, towns laid out along their roads, trains, carts and boats, scaffolding on new and growing works, click-to-inspect with each works' "why here" sums, a game interface.
- Done since (docs/DEVLOG.md, 10 to 11 November): every colour, the light and the lens in one look file; the generators in a crate of their own, with a lab of contact sheets and a set of standard shots to compare before and after; works drawn as compounds in their own yards; houses built from a kit at three levels of detail; land whose uses meet along the terrain, and a far view; saved games; map views and charts.
- Next, from docs/ART.md: visible change (fixed steps per element on a staggered ladder, scheduled inside the year, scaffolding as a state) and building the scenery per chunk with a cache (a year is already built off the main thread). The kept layer of town ground that this item used to ask for is in the simulation under the game's rules (docs/rules/towns.md): a cell that joins a town stays, and under half of one per cent of town cells change in ten years. The viewer does not draw from it yet.
- Next for the look: construction with a puff of dust, people and carts on the roads, weather, a title screen. Saved games are in, as a log of orders played again from 1750 and not in the replay format.
- Interventions: pause, change a parameter or place a thing (a canal, a tariff, a land tax), resume, and compare the branch against the baseline on the same seed.
- A diff view on the map: where two branches differ, in rent, use and population.
- Agent drill-down: follow one firm or town over time, with its decisions and their reasons in a timeline.

## Stage 3: the game (sparse)

- The player's seat, decided: a family running a conglomerate among rival families. Companies mostly run themselves; the player chooses trades, places and people. The government is always run by one family; holding office drains the family's people and attention, so power changes hands without ending anyone's game. Historical politics (a limited franchise, rotten boroughs, reform acts, the Corn Laws) comes next, after the company half is fun on its own.
- Balance from the scripted players (DEVLOG, 2026-11-09): honest returns in thin markets, consent or a premium for buying a rival's land, a start that puts the player mid-table, and spending as a lever.
- Open after families v2 (FAMILIES.md): land beats works (decide on land buying: more years' purchase, a cap on direct purchases, or prices that rise as one buyer takes a town's land); families own over half the works' capital by 1900; industrial rivals stay small; AI families never hold, wind down or move their seat.
- Next for the family game: research and patents (an educated member posted to research a trade; a patent on a method the family brings in, licensed for royalties or kept to the family for its term), and a bonus for running both ends of a supply chain in one town, so vertical integration is a choice. Then politics.
- Keep the sim honest: the player acts through the same prices and bids as everyone else, never by editing outcomes.
- Make tangibility the design brief: milestones, inventions with names and places, booms and slumps called live and dated later, and a map that visibly answers "why is that there".
