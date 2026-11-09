# Plan

Three stages: a lab you can tinker with, a visual tool you can steer, then a game people step into. Detail is front-loaded on purpose; later stages will be rewritten once the earlier ones teach us what matters.

## Decisions so far (November 2026)

- **Rust all the way.** The simulation moves to a Rust crate (`crates/isostate-core`), ported module by module and checked against the Python version, which stays as the reference and the notebook lab. The 3D client is Bevy (`crates/isostate-viewer`). Desktop first.
- **Look: a cosy diorama in the spirit of Tiny Glade.** Character comes from palette, silhouette, light and small moving life, not from hand-drawn assets, so it survives an island that rebuilds itself every year.
- **Buildings feel placed, the economy stays smooth.** Under the hood the economy works in shares and densities per 2 km cell. The viewer turns those into stable things: each firm is a building on a fixed plot that grows in tiers, and every tree, house and farmstead has a fixed slot that fills in once its cell's density passes a fixed threshold. Nothing reshuffles from year to year.

## Where we are

The simulation runs in Rust (`crates/isostate-core`), checked against the Python step by step, 150 years in about 21 seconds. The Bevy viewer (`crates/isostate-viewer`) plays recorded runs and live games. A live game has six families, the player's first. The player chooses from short lists: the family's first trade, each child's training, where grown members run its trades, what each branch does, where the seat is, and, at the seat only, which works to build and which land to buy (see `crates/isostate-core/FAMILIES.md` for the rules and docs/DEVLOG.md for what exists). The look is planned in docs/ART.md. The economic weaknesses listed under stage 1 still stand, and the politics half of the game is not started.

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

- Done: replays and live play in Bevy, a terrain shader for fields, hedges and crops (crisp at any zoom, with the seasons), relief, meandering rivers, coasts and a sea, enclosure and open fields, graded railways with their engineering, towns laid out along their roads, trains, carts and boats, scaffolding on new and growing works, click-to-inspect with each works' "why here" sums, a game interface.
- Next, from docs/ART.md: visible change (fixed steps per element on a staggered ladder, scheduled inside the year, scaffolding as a state) and building the scenery per chunk in the background. Town ground also needs a kept urban layer that decays slowly, shared by the shader and the slots: about a third of urban cells switch in or out each decade in seed 7, so districts come and go.
- Next for the look: construction with a puff of dust, people and carts on the roads, weather, a title screen and saved games (the replay format can carry a game).
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
