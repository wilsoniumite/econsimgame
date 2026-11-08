# Plan

Three stages: a lab you can tinker with, a visual tool you can steer, then a game people step into. Detail is front-loaded on purpose; later stages will be rewritten once the earlier ones teach us what matters.

## Where we are

A first full demo. The sim runs 1750 to 1900 on one procedural island in about 80 seconds, conserves money exactly, and exports a replay that a single HTML page turns into a scrubbable map, news feed, charts and a per-firm "why here" inspector. Two scenarios (landlords and shared rent) can be compared in the charts. The main weakness is economic, not technical: structural change is too weak (see README, Known problems).

## Stage 1: the lab (next, dense)

**1.1 Make runs cheap enough to sweep.**
- Profile the monthly step; move wage bisection, trade and land allocation into numba or fully vectorised numpy. Target: 150 years in under 15 seconds.
- A batch runner: `python -m isostate sweep --seeds 1-32 --set land_theta=2,3,4` writing one Parquet table of annual series per run, with tqdm over runs.
- Seed robustness as a standing check: every headline claim in the README should hold in most seeds, not just seed 7.

**1.2 Fix the economics we already know are off.**
- Structural change: let the food share fall with income (Engel), make farm productivity respond earlier to tools and husbandry, and let towns bid labor away with an urban wage premium. Success test: urban share rises from about 0.25 to 0.5 over the run in most seeds.
- Price volatility: add capacity expectations (firms see the pipeline of capacity under construction) and a slower investment response; check iron and rails no longer cycle with a period of a few years.
- Railways: let promoters see the traffic a line would induce, not just today's flows, and allow branch lines from any railed town, so the network keeps growing after the first rush instead of stopping at about 850 km.
- Damp the two-to-three-year cycle in farm employment (labor and land shares overshoot each other).
- Foreign trade as a world price for a few goods at the ports, so the island can specialise.

**1.3 Calibration harness.**
- A small table of stylised facts with tolerances (population growth, real wage path, coke iron share by 1800, mill cloth share by 1830, rail km by 1870, urban share by 1900).
- A loss over those, and a simple search (random or CMA-ES) over a handful of parameters, run on many seeds.
- Keep parameters in `params.py` and content in `content.toml`; calibration writes a new params file, never edits code.

**1.4 Observer quality.**
- Store event provenance (which series, which test, which threshold) so a reader can always ask "why is this news".
- Rank events per decade so the major feed reads like a history, around 2 to 4 items a decade.
- Detect regional stories (a coalfield boom, a town in decline) from cell and town data, not just national series.

**1.5 Tests to add.**
- Land shares sum to one; no negative stocks; every firm's reason record matches its site.
- Golden-run regression: annual series for seed 7 within tolerance of a stored copy, updated deliberately.

## Stage 2: the visual tool (sparser)

- Run the sim live rather than as a replay. Options: Pyodide for the current code (simplest, slow), or port the monthly step to Rust or Zig compiled to WebAssembly once the model stops changing every week. The replay format stays as the save-game and sharing format.
- Interventions: pause, change a parameter or place a thing (a canal, a tariff, a land tax), resume, and compare the branch against the baseline on the same seed.
- A diff view on the map: where two branches differ, in rent, use and population.
- Agent drill-down: follow one firm or town over time, with its decisions and their reasons in a timeline.

## Stage 3: the game (sparse)

- Decide the player's seat: a government (Democracy 4 style levers on top of the economy), one firm among many, or a town. Prototype each as a thin layer over the stage 2 tool before committing.
- Keep the sim honest: the player acts through the same prices and bids as everyone else, never by editing outcomes.
- Make tangibility the design brief: milestones, inventions with names and places, booms and slumps called live and dated later, and a map that visibly answers "why is that there".
