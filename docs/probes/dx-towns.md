# dx-towns: town ground, churn, the urban share, and a kept urban layer


Worktree `C:/Users/wilso/Documents/GitHub/econsimgame/.claude/worktrees/wf_b38216ce-d6f-9`, branch `night/dx-towns`, commit `7677f96` (measuring only, not for merging). Line numbers are those of unmodified `rust-3d`. Seeds 3, 7, 11, 21 unless "8 seeds" (adds 1, 5, 13, 17). All runs without families unless stated; a check with six families on seeds 3 and 7 gave the same picture.

Probes: `crates/isostate-core/examples/probe_towns.rs` (one CSV row a year per seed; `probe_towns OUT_DIR SEEDS "k=v;k=v"`), `probe_fabric.rs`, `probe_cell.rs`; helpers in `probe_tools/` (`run_variants.py`, `cmp.py`, `decades.py`, `summ.py`); outputs in `probe_out/` (uncommitted). Every switch is a field of `Params::hack` (`params.rs`), off by default. With all off, the probe CSVs are identical to those from the unpatched code and `cargo test --release -p isostate-core` passes.

## 1. What is wrong, in five findings

1. **The urban share is an accounting identity, not a stock of people.** `land::update_urban` (land.rs:171-172) sets U = pop x clamp(0.04 + 0.8 x firm labour / labour force, 0.06, 0.95) and `observer::record_month` (observer.rs:178-179) reports sum(U)/pop. Check: 1900 firm labour share 0.246 gives 0.237; measured 0.235. Nothing in the economy reads U except the urban rent bill (econ.rs:292). "Shops and trades" are a firm method (`services`), so they are already counted.
2. **The cell count is flat because density rises 5.2 times while U rises 4.6 times** (land.rs:161, 173). With density frozen at 500 the same rule would give 73 cells in 1750 and 331 in 1900 (offline counterfactual, 4-seed mean).
3. **Churn is arithmetic noise, not industry.** Cells are re-picked every year from n = ceil(U/density). A town's U moves 6.3% a year on average (seed 7, mean absolute log change 1761-1900) because `ind_labor` is the December snapshot of `f_labor` (sim.rs:407-408), and an average town has only 4 urban cells, so n steps up and down.
4. **The picture is dominated by works, not towns.** Industry cells (footprint share >= 0.25, land.rs:325) go from 220 to 872 while urban cells stay near 70. A works fills a 4 km2 cell at K = 4 x K0 (`footprint_K_mult`, firms.rs:210, sim.rs:244) whatever its size in people: in seed 7 in 1900, 1,347 crucible steel shops cover 600 cell-equivalents with 8,368 workers (14 a cell). Urban ground holds 3,700 people a cell at the centre in the same year.
5. **The home town does shrink.** Game configuration (seed 7, six families), Melbourne: 15 urban cells in 1750 (pop 21,570), 5 in 1780 (pop 14,243: about seven new towns are founded on the island in the 1750s and take people and catchment), 6 in 1900 (pop 48,683), while industry cells in its market area go 13, 14, 56 (1850), 86 (1900).

I could not reproduce "Kirkford, cell 75,47": the Rust seed 7 is a different island from the Python one (cell 75,47 is never urban; no town of that name).

## 2. The churn (question 1)

Port, 4 seeds, mean per seed:

| | 1750s | 1760-1900 |
|---|---|---|
| urban cells flipping in or out, per cell per year | 0.135 | 0.069 |
| decade churn, (left + entered) / cells at start | 0.78 | 0.21 (0.53 in the 1830s, 0.34 in the 1840s) |
| cells leaving per decade / cells at start | 0.43 | 0.10 |

Over a run: 162 to 173 cells are urban at some time, 71 to 79 at the end, 15 or 16 every year; 55 to 68 cells have more than one spell.

Causes, 1760-1900, summed over the four seeds (1,187 entries, 1,144 exits):

| cause | entries | exits |
|---|---|---|
| the town's n went up or down (U/density) | 812 (68%) | 793 (69%) |
| rank swaps at constant n (path costs change: road upgrades and decay, railways; worst 1830s-40s) | 350 (29%) | 351 (31%) |
| catchment changed | 25 (2%) | 0 |
| of these: entering cell had been blocked by industry the year before | 21 | |
| of these: leaving cell became `industry` | | 18 |

Offline counterfactuals on the same runs (flips 1760-1900, seed 7 / 11): actual 487 / 682; no blocking by industry or mines 469 / 654; U smoothed (EMA 0.2) 249 / 372; share frozen at its 1750 value 164 / 256. So industry blocking causes almost no churn; the moving firm-labour share causes about two thirds. Exact score ties are not a cause (the jitter field breaks them). The 1750s are a separate episode: 6 towns become about 13 (`people::found_towns`, people.rs:117, one a year) and `resettle_new_towns` moves the people.

What blocking does do is shape: by 1900, 74 blocked cells lie nearer a town centre than its outermost urban cell (2 in 1750).

Rents and sites:

- A cell leaving town ground: rent (`land_rent_only`) 761 to 139 in one year (1850 mean), 1,355 to 299 (1900). A cell entering: 107 to 790 (1850), 265 to 1,476 (1900). A works' rent is fixed for life at founding (firms.rs:114, 192), so it depends on which side of the flip the cell was that year.
- The households' bill is steady: `urban_rent` is 0.094 to 0.107 of U x wage, 2.9 to 3.8% of household budgets.
- Mean urban cell rent is 7 to 11 times the adjoining farm rent (1,923 against 236 in 1900), so works avoid town ground: urban cells carry 3 to 4.5% of the candidate weight (firms.rs:135-142), new works on urban cells are 1.1 a year of 43 in the early 1750s, 0.2 of 59 around 1800 and none from 1825, and the share of firm labour standing on urban cells falls 0.27, 0.18, 0.07, 0.04 (1750, 1800, 1850, 1900).

## 3. Why the share stays flat (question 3)

Labour shares (4-seed means, 5-year averages):

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| farm | 0.384 | 0.335 | 0.367 | 0.310 |
| pasture + forestry | 0.246 | 0.224 | 0.147 | 0.138 |
| quarry | 0.082 | 0.120 | 0.151 | 0.181 |
| clay, iron ore, coal | 0.034 | 0.071 | 0.099 | 0.125 |
| shops and trades (`services`) | 0.136 | 0.162 | 0.162 | 0.172 |
| all other works | 0.118 | 0.087 | 0.074 | 0.075 |
| real wage (1750 = 1) | 1.09 | 1.34 | 1.39 | 1.35 |
| budget / subsistence cost (`y_ratio`) | 2.05 | 3.00 | 3.24 | 3.67 |

Household spending shares 1750 to 1900: grain 0.39 to 0.32, stone 0.075 to 0.138, bricks 0.020 to 0.043, coal 0.12 to 0.13, cloth 0.07 to 0.15, wool 0.12 to 0.05, services 0.048 to 0.071, urban rent 0.035 to 0.032. Grain relative to the cost of living: 0.98, 0.86, 0.94, 0.67.

Reading: farms do release labour (18 points), but it goes to quarries (+10), coal and clay (+9) and shops (+4); manufacturing loses 4 because new methods and learning cut labour per unit while the linear expenditure system holds budget shares fixed. Shelter (marginal share 0.22) is bought as raw stone, so by 1900 quarrymen are 18% of all labour. The marginal food share of 0.28 is a floor under farm labour. Real income stops growing after 1800 (`y_ratio` 3.0 to 3.3 from 1800 to 1875), so nothing shifts later.

Tests (4 seeds, 1750 and 1900 five-year means):

| variant | urban 1750 | urban 1900 | works' labour 1900 | farm+pasture+wood | mines+quarries | real wage 1900 |
|---|---|---|---|---|---|---|
| port | 0.251 | 0.235 | 0.246 | 0.448 | 0.305 | 1.35 |
| S1 shares food 0.20, comfort 0.18 | 0.293 | 0.295 | 0.323 | 0.378 | 0.299 | 1.33 |
| S2 food 0.12, comfort 0.28 | 0.333 | 0.363 | 0.408 | 0.309 | 0.282 | 1.31 |
| S3 food 0.12, shelter 0.30, comfort 0.20 | 0.305 | 0.316 | 0.350 | 0.310 | 0.340 | 1.36 |
| T2 above 2.0 x subsistence spend [0.05, 0.08, 0.15, 0.15, 0.05, 0.52] | 0.271 | 0.397 | 0.445 | 0.316 | 0.239 | 1.42 |
| T4 above 2.3 x: [0.04, 0.06, 0.13, 0.13, 0.04, 0.60] | 0.257 | 0.390 | 0.439 | 0.325 | 0.237 | 1.44 |
| H1 husbandry_boost 1.2, tools_boost 1.0 only | 0.255 | 0.257 | 0.273 | 0.414 | 0.314 | 1.62 |
| T4 + H1 | 0.278 | 0.453 (0.41-0.50 by seed) | 0.522 | 0.254 | 0.224 | 1.75 |
| T3 (tier 1.8, comfort 0.57) + H1 | 0.326 | 0.501 | 0.580 | 0.218 | 0.202 | 1.89 |
| P1 works pay wage x 1.2 | 0.228 | 0.210 | 0.213 | 0.474 | 0.312 | 1.32 |
| P2 works pay wage x 1.5 | 0.198 | 0.190 | 0.186 | 0.497 | 0.317 | 1.30 |

T4 over 8 seeds: 0.254 to 0.395.

Conclusions. (a) Changing the fixed shares only shifts the level, 1750 included. (b) An Engel curve in two pieces (`econ::les2` in the worktree) leaves 1750 alone and raises the share with income: 0.39 by 1900 on today's income path, most of it by 1800 because that is when income grows. (c) More shelter spending feeds quarries, not towns. (d) Farm productivity alone raises the wage, not the share. (e) A wage premium lowers the share: each town has one labour market and one wage, so a premium is a tax on works. (f) Counting services changes nothing. Honest arrival at one half needs the Engel tier plus real income of about 5.5 times subsistence by 1900 (T3+H1), so it depends on whoever fixes the post-1800 stall. Not tested but indicated: shelter through a building-trades method (town work) instead of raw stone, and exports of manufactures.

## 4. The kept urban layer (question 2)

Prototype: `land::update_urban_kept` in the worktree (switched at the top of `update_urban`), state in `state.rs` (`urban_built`, `urban_cap`, `town_urban_cells`, `town_low_years`, `urban_pop_s`).

Rule, per town, each year:

1. U as now, then smoothed: 0.5 x last year's + 0.5 x this year's.
2. Each town keeps an ordered list of cells, the centre first. A cell holds cap = (d_new + (d_old(year) - d_new) x (1 - exp(-age/tau))) x (1 - 0.5 x works' footprint share), with d_new 300, d_old 700 + 1.0 a year, tau 40 years, age = years since it joined. The centre is the oldest and so the densest; density rises slowly.
3. While U exceeds the sum of caps, the town takes its best free neighbour (8-neighbourhood of its own cells), by the port's score (minus path cost, plus jitter and the water bonus), so towns grow in ribbons along roads. Sea and mines are refused; a cell with works may be taken and stays town ground (land.rs:322-327 already keeps `urban` above `industry`).
4. A town gives up its newest cell, one a year, only after 15 years in a row below 60% of capacity.
5. A cell whose market town changes passes to the new town with its year built. Residents are shared by capacity; rents keep the port's formula.
6. The 1750 towns' first cells are back-dated (built 1690 and earlier, centre oldest).

Two companions tested with it: works' land take by hands and plant (`firms::footprint`: K x labour per unit / 1,500 + (K/K0) x 0.03 of a cell, instead of K/(4 K0)), and town trades (services, smithy, hand_loom, millwright, engine_works, crucible_steel, forge_rails) drawing candidate sites only from town ground and the cells next to it (`firms::candidate_cells`).

Before and after, 8 seeds (five-year means; churn rows are 1760-1900):

| | port | kept only (K2) | kept + land take + town trades (R) | R + Engel T4 (RT4) |
|---|---|---|---|---|
| urban cells 1750 / 1800 / 1850 / 1900 | 69 / 55 / 59 / 72 | 96 / 124 / 146 / 219 | 94 / 134 / 179 / 295 | 95 / 171 / 274 / 493 |
| 1900 over 1750 | 1.04 | 2.3 | 3.1 | 5.2 |
| cells leaving per decade / cells at start | 0.101 | 0.031 | 0.013 | 0.005 |
| (left + entered) / cells at start, per decade | 0.213 | 0.109 | 0.103 | 0.123 |
| cells with more than one spell | 55-68 | 7-21 | 3-7 | 0-13 |
| urban share 1750 / 1900 | 0.251 / 0.237 | 0.249 / 0.237 | 0.252 / 0.244 | 0.254 / 0.386 |
| industry cells 1750 / 1900 | 219 / 857 | 219 / 819 | 21 / 240 | 21 / 216 |
| share of firm labour on town ground 1750 / 1900 | 0.28 / 0.05 | 0.32 / 0.21 | 0.79 / 0.47 | 0.79 / 0.60 |
| urban rent bill / (U x wage), 1900 | 0.105 | 0.093 | 0.092 | 0.091 |
| mean urban cell rent / fringe farm rent, 1900 | 1,895 / 231 | 544 / 235 | 401 / 218 | 415 / 182 |
| people on densest / thinnest town cell, 1900 | 3,688 / 933 | 1,095 / 188 | 820 / 162 | 801 / 158 |
| real wage 1900 (seed spread about +-0.07) | 1.35 | 1.32 | 1.39 | 1.45 |
| output per head 1900 | 6.45 | 6.25 | 6.55 | 6.35 |
| farm land (sum of farm shares, cells) 1900 | 1,276 | 1,254 | 1,344 | 1,235 |
| works alive 1900; most works on one cell | 4,329; 13 | 4,325; 13 | 5,590; 44 | 4,987; 41 |

With six families (seeds 3, 7), 1750 to 1900: port 69 to 77 cells; R 92 to 316; R + Engel T2 100 to 483; urban share in 1900 0.26 / 0.26 / 0.41. Seed 7 without families, Melbourne in 1900: 5 town cells in the port, 18 under R.

What it does:

- Farmland near towns: town cells are `taken` in `update_land_use` (land.rs:229-231), so kept-only loses 1.7% of farm land by 1900; grain's relative price and the real wage do not move beyond seed noise. With the smaller works' land take the island has 5% more farm land than the port.
- Rents: the bill falls about 12% (the port's gradient term averages lower over a wider town); `urban_rent_rate` 0.12 to 0.137 would restore it. Mean town cell rent falls from 8 times to under 2 times the fringe farm rent, so outer town cells pay less than the farms they displaced: floor each cell's urban rent at its best farm bid (not tested).
- Where works go: with kept only, works still sit on cheap ground outside and the town absorbs them later (0.21 of firm labour on town ground). With the land take and town trades, 0.47 to 0.60. The remaining gap is works on cells next to the town that the town has not yet grown over.
- The 1750 map has about a third more town cells than now (94 against 69), because cells never leave and 1750s towns start at d_new. Raise d_old to about 850 to match the port in 1750 if wanted.

Viewer data. `record::layers` (record.rs:175-219) writes for town ground: `use` = 10; `inten` = residents / the year's national density, clamped to 0.05-1 (record.rs:183-185; the centre saturates from 1750); `mix_a/b/s` = urban alone (record.rs:80-83, so works on a town cell are invisible in the mix); and `town_urban` per town. To add (per cell, per year, in `LAYERS` record.rs:19 and `LiveLayers` in live.rs): `built` (u8, year joined minus 1645, 0 = not town ground), `res` (u8, residents / 8, absolute), `works` (u8, footprint share x 255 on every cell; today it is recorded only on industry cells, through `inten`). The viewer reads arrays by name and already tolerates missing ones (replay.rs:180), so they can be optional. No viewer change made.

## 5. Recommendations, ranked

1. **Kept urban layer** (`urban_kept`, with `urban_d_new` 300, `urban_d_old` 700, `urban_d_old_growth` 1.0, `urban_tau` 40, `urban_empty` 0.6, `urban_decline_years` 15, `urban_smooth` 0.5). Touches `land::update_urban` (new branch), `State`, `record::layers`, `live::LiveLayers`. Port the prototype as is. Measured: 2.3 times the cells by 1900 alone, exits a third of the port's (an eighth with 2). Risks: families' land rules treat urban cells as not farmland (families.rs:633-640), so more ground near a seat is unbuyable (not checked); 1750 looks a third larger. Worked if: `decades.py` shows left/n_start under 0.035 a decade, cells 1900/1750 at least 2.3, fewer than 25 cells with two spells, money check passes, port CSVs unchanged with the switch off.
2. **Works' land take by hands and plant, and town trades on town ground** (`works_per_cell` 1,500, `works_plant` 0.03, `town_trades`; a `town = true` flag per method in content). Touches `firms::create_firm` (210), `close_firm` (497), `evaluate_sites` (113), `sim::after_transport` (244), `firms::candidate_cells` (126); leave the size cap in `review` (387) on `footprint_K_mult`. Measured: industry cells 857 to 240 in 1900, town cells 3.1 times 1750, firm labour on town ground 0.05 to 0.47. Risks: up to 44 works on one cell (13 now), which the viewer's plots may not hold (`works_plant` 0.06 gives 32 and 362 industry cells); 29% more works alive; family balance not rerun. This is what removes the grey plain; without it the town stays the smaller thing on screen.
3. **Engel tier** (`engel_y` 2.3 and `shares_high` [0.04, 0.06, 0.13, 0.13, 0.04, 0.60]; `econ::les2`, called in `step_month` at econ.rs:296 for households only). Measured: urban share 0.25 to 0.39-0.40, real wage +7%, town cells 5.2 times with 1 and 2. Risk: grain output is 30% lower in 1900 (people eat 2.0 instead of 2.8 times subsistence), services become 37% of labour. One half needs the income stall fixed as well.
4. **Do not add an urban wage premium, and do not change the counting rule**: the first lowers the share, the second is already in.
5. Follow-ups, untested: U from the year's mean firm labour instead of December's (sim.rs:407); urban rent floored at the farm bid; shelter through building trades; one new town a year is already the cap, but seven in the 1750s empties the home town, which may want a later start.

New parameters must be skipped in `params.json` when at their defaults, as the family ones are (params.rs:140-189); the game switches them on where it sets `n_families` (crates/isostate-viewer/src/live.rs:103).

## 6. Not checked

The viewer with any of this; game balance (bots) under the new rules; the Python reference; whether individual outer town cells (not the mean) fall below farm rent; sensitivity to `urban_tau`, `urban_smooth` and the 0.5 x footprint capacity cut; seeds beyond the eight; more than one run per seed and variant (differences in the real wage under about 5% are inside seed noise).
