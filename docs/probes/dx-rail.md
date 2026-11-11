# dx-rail: why the railway network stops, what it does, and what keeps it growing

Branch `night/dx-rail` in worktree `.claude/worktrees/wf_b38216ce-d6f-11` (measuring code, not for merging). Probe: `crates/isostate-core/examples/probe_rail.rs` (`--out DIR --tag T --json '{overrides}' SEEDS`). Results and scripts: `tools/night-dx-rail/` (`out/TAG_sSEED_yearly.csv`, `_cands.csv` with every candidate looked at, `_summary.txt`). Seeds 3, 7, 11, 21 unless stated; 1, 5, 9, 13, 17, 42 where stated. A run takes 13 s with or without the prototype. With default parameters the hacked build reproduces the recorded baseline exactly (rail km, output per head and real wage of seeds 3, 7, 11 in 1850 and 1900). Line numbers are those of unmodified `rust-3d`.

"Return" means benefit over cost as `rail.rs` computes it, which already includes the `rail_induced` factor of 2.5.

## In short

1. The network stops because the promoters run out of candidates, not because lines fail their sums. `rail::plan` only looks at pairs of towns, and by 1850 every town is joined. `rail::plan_spurs` only looks at the 25 busiest cells of the map, which after the rush are all on the line.
2. Unbuilt lines to quarries, works, collieries and farm districts would return 190 to 700% a year on today's traffic. Induced traffic in the sums is not what is missing.
3. Railways cut town-to-town cost by 78% and raise trade by a third, but output per head is unchanged, because carting is paid to households as income and uses nobody's time.
4. A prototype (branch lines priced on the carting they save, railways five times dearer, promoters who see the order book) gives 700 km in 1850, 1,410 in 1870 and 2,100 in 1900 on ten seeds, rising every decade.

## 1. Why it stops (unmodified code)

| seed | first project | trunks promoted | spurs | laid out | median / lowest return | km 1850 | km 1900 | towns with track 1900 |
|---|---|---|---|---|---|---|---|---|
| 3 | 1827 | 33 | 7 | 608,000 | 0.73 / 0.11 | 946 | 976 | 17 of 18 |
| 7 | 1835 | 35 | 1 | 546,000 | 0.96 / 0.07 | 968 | 986 | 18 of 19 |
| 11 | 1826 | 31 | 1 | 948,000 | 0.57 / 0.06 | 870 | 874 | 17 of 17 |
| 21 | 1828 | 46 | 1 | 738,000 | 0.48 / 0.07 | 980 | 980 | 21 of 21 |

Ten seeds: 942 km in 1860 (776 to 1,072), 991 in 1900 (812 to 1,264).

- **Trunks.** In the 1850s, 81 to 90% of town pairs are in `rail_links`, most put there at `rail.rs:90` because the cheapest route already has track on every cell. The other 19 to 29 pairs are looked at every four years and refused every time: 234 to 356 looks per seed in 1850 to 1900, median 1 new cell on a path of 36 to 64, saving exactly zero in 95 to 100% of looks (the pair is already joined by rail, so cutting a corner saves nothing). Nothing real is under consideration.
- **Mineral lines.** `plan_spurs` takes the 25 cells with most traffic (`rail.rs:175`) and then drops any within 4 cells of track (`rail.rs:180`). From 1850 all 25 are on the line every year in seeds 7, 11 and 21, and in all but a few years of the 1870s in seed 3 (its 4 late spurs). The busiest 5x5 block at least 4 cells from track carries 0.5 to 1.2 million weight a year, 25 to 60 times `spur_min_flow`. A spot once looked at is also barred for good (`spur_done`, `rail.rs:214`).
- **Towns.** All are joined by 1850 except Welford (seed 3, 15,000 people) and Highfell (seed 7, 5,400), each one cell from the track: their cost to other towns is already low, so the pair rule sees no saving. No town is founded after 1810 in seeds 3, 7, 11 (one in 1845 in seed 21), so no new destination appears.
- **Left without rail in 1900.** 60 to 64% of cell traffic starts more than one cell from track. Quarries: 4 to 12% of quarry cells are within a cell of track (they carry 12 to 18% of all cell traffic). Collieries 37 to 56% of cells, farmland 23 to 38%, works 26 to 38%.
- **What lines to them would be worth.** I ran the prototype's sums on the unmodified run without building (the 8 districts with the largest carting bill, every year). All clear the hurdle. Median line 6 to 11 cells, cost 5,000 to 15,000, saving 10,000 to 56,000 a year on today's carting, a return of 190 to 700% a year before the 2.5 factor. Of the candidates of 1860 and 1880, 67% are quarry districts, 23% works, 6% collieries. Welford's one-cell link costs 270 to 1,250 and saves 3,700 to 10,500 a year.
- **Who paid.** Every promoted line raised 97 to 100% of its cost at once from the landlords (35% of cash) and households (8%) of its two towns, the lower-numbered town first (`rail.rs:108`). Money was never short. The whole network costs 2 to 4% of one year's output (Britain's first 10,000 km cost about 40%). Fares on the traffic over track (0.07 a cell) come to 50 to 200% of that capital every year.
- Projects promoted together pay twice for cells they share (new cells are counted against laid track only): seed 3 paid for 1,304 km and has 976.

## 2. What railways do

Means of 1860 to 1900 over the four seeds. "No rail" is `rail_know = 99`. "Prototype" is section 3's settings.

| | no rail | unmodified | prototype |
|---|---|---|---|
| town-to-town path cost (population-weighted) | 20.1 | 4.4 | 4.1 |
| cell-to-market path cost (traffic-weighted) | 3.52 | 2.43 | 1.62 |
| carting bill, share of output | 7.7% | 4.4% | 3.3% |
| trade between towns, million weight a year | 6.4 | 8.6 | 9.3 |
| mean haul, km | 62 | 71 | 71 |
| grain traded, million units | 2.0 | 3.1 | 3.4 |
| stone traded, million units | 0.53 | 0.91 | 1.04 |
| coal traded, million units | 1.56 | 1.56 | 1.52 |
| price spread between towns (CV): grain | 0.166 | 0.097 | 0.106 |
| stone / timber / coal | 0.16 / 0.11 / 0.44 | 0.12 / 0.08 / 0.50 | 0.16 / 0.09 / 0.48 |
| output per head | 6.66 | 6.65 | 6.56 |
| real wage | 1.29 | 1.27 | 1.27 |
| cell traffic within a cell of track, 1900 | none | 36 to 40% | 70 to 80% |
| output per head with `carting_labor = 1` | 5.98 | 6.55 | 6.32 |

- Grain and stone travel more and further and their prices converge. Coal does not.
- **Output per head does not move** (minus 0.2% on four seeds, plus 1.3% on six more; one year's noise is about 5%). The reason is in `econ.rs`: carting money is paid to households as income (`econ.rs:666`, `696` to `699`, `879`) and no labour is used, so cheaper carriage frees nothing. Carting was 10% of output in 1800 to 1825.
- **Rents.** No consistent premium. On the same cells with and without rail, land within 1.5 cells of the 1900 track rents at 0.83 to 1.02 times the no-rail run in 1875 and 0.87 to 1.25 times in 1900.
- **Works.** The share of works' capital within 1.5 cells of the 1900 trunk track is 31 to 47% in 1825 and 33 to 46% in 1900 (30 to 46% without rail). Trunks join towns, where the works already are.
- **Would seeing induced traffic build more?** No. A term for the trade a line would start (`rail_induced_trade = 0.2`, the monthly trade rule at the new cost, rule of half) changed km in 1900 by minus 26 to plus 52. Dropping the 2.5 factor altogether still gives 1,812 to 2,066 km.

## 3. The prototype

All switches are new parameters, off by default.

- `rail_branches`: `rail::plan_branches` (new, called after `plan_spurs` at `sim.rs:441`). Candidates each year: every town without track, and up to 8 districts where the 5x5 carting bill (`cell_flow_year * dcatch`) is largest, at least `rail_branch_gap` (3) cells from track and 6 from each other. Each is routed on `rail_build_graph` to the nearest cell of the network, so lines branch from any junction. Refused districts cool off for 5 years.
- `rail::line_sums(st, C, P, new_cells, towns)`: copies `st.graph`, lays the track (`graph_with_track`), and runs one search from all towns and one per served town. The saving is `freight` times the fall in every cell's cost to market times its traffic, plus the fall in the served towns' cost to every other town times their trade. `rail_network_sums` makes `plan` use the same sums for trunks.
- `rail_cost_mult` (5): materials per cell. `rail_build_months` (24, was the constant 10 at `econ.rs:568`). `rail_order_book`: promoters price rails at today's price times (1 + rails on order / a year's rail output).
- Caps: `rail_max_projects` 12 and `rail_branch_per_year` 6.

Rail km, mean of four seeds (lowest to highest in 1900):

| | 1840 | 1850 | 1860 | 1870 | 1880 | 1890 | 1900 |
|---|---|---|---|---|---|---|---|
| unmodified | 730 | 941 | 942 | 942 | 950 | 954 | 954 (874 to 986) |
| spur fix only (`rail_spur_scan`) | 746 | 1,123 | 1,428 | 1,698 | 1,898 | 2,094 | 2,228 (2,120 to 2,336) |
| branches at default cost, caps 12 and 6 | 1,108 | 1,706 | 2,112 | 2,442 | 2,746 | 2,866 | 2,880 (2,832 to 2,958) |
| **prototype** | 244 | 712 | 1,110 | 1,468 | 1,758 | 1,947 | 2,140 (2,028 to 2,334) |
| prototype with 6 families | 206 | 780 | 1,145 | 1,518 | 1,802 | 2,036 | 2,242 (2,120 to 2,324) |
| prototype, ten seeds | 296 | 699 | 1,023 | 1,411 | 1,671 | 1,903 | 2,098 (1,778 to 2,334) |

- At default cost the branches are paced only by the caps: nothing is ever refused and the map fills by 1880. At five times the cost the sums and the money bind: 220 to 280 trunk looks in the 1830s fail for want of subscriptions, district lines run at 17 to 39 a decade from the 1860s, and refusals on return rise from none in the 1850s to 15 to 63 in the 1890s. All towns have track by 1860.
- What gets a line: 40% farm districts, 24% quarries, 16% works, 13% pasture, 5% collieries.
- The order book matters. Without it rails peak at 8 to 24 times their 1800 to 1825 price and rail output collapses after the rush (seed 3: 26,600 a year in 1845, 74 in 1850). With it the peak is 1.8 to 8.6 times (unmodified: 1.7 to 5.4) and rail output over 1840 to 1900 is 1.5 times the unmodified run's.
- Fares on the traffic over track are 7 to 14% of the capital laid out, a year, so a company taking them could pay a believable dividend.
- Sensitivity (km in 1900): cost times 3: 2,376 to 2,686. Default caps: 1,392 to 1,536. Without `rail_capital_market` (a share of other towns' landlords' cash for branches): no change, so it is not in the settings. Spur fix at five times the cost: 1,020 to 1,798.
- Output per head 6.54 against 6.58 (1840 to 1900), real wage 1.29 against 1.30. No money drift on any run.

## 4. Recommendations, ranked

**1. Branch lines priced on network sums.** Port `plan_branches`, `line_sums`, `graph_with_track` and `State::branch_cooldown` from the branch; parameters `rail_branches`, `rail_branch_gap`, `rail_branch_per_year`, `rail_network_sums`. Game rules: both switches on, `rail_branch_per_year = 6`, `rail_max_projects = 12`. Risks: (a) news: 166 to 198 railway events a run against 32 to 47, so group branch openings by year or lower their weight; (b) `progress` calls every such line a "mineral line" for "pits and works", wrong for the 53% that serve farms and pasture; (c) the double paying above, which also hits branches (seed 3 paid for 2,922 km and has 2,334): count cells of projects under way as laid; (d) every track cell acts as a station. Test: km rises every decade from 1840 to 1900, all towns have track by 1860, and at least 70% of cell traffic is within a cell of track in 1900.

**2. Price railways properly, in the game's rules only** (set in `crates/isostate-viewer/src/live.rs:100`): `rail_cost_mult = 5`, `rail_build_months = 24`, `rail_order_book = true` (new `materials_cost` in `rail.rs`, the divisor at `econ.rs:568`). This is what turns the cap-paced fill into an S-curve with a rush limited by money and iron. Risk: the first railway comes later (none by 1830, 712 km in 1850 against 941), which moves PLAN's "rail km by 1870" fact to about 1,470. Test: the rails price peak stays under 9 times its 1800 to 1825 mean.

**3. Carting takes labour** (`carting_labor`, 0 by default). `econ::step_month` sums the month's carting by town into `State::cart_bill`, and next month's labour supply (`econ.rs:402`) falls by `carting_labor * cart_bill / wage`. With 1.0, railways raise output per head in 1860 to 1900 by 9.6% in the unmodified network (seeds 3, 7, 11, 21: plus 7.8, 4.2, 5.5, 21%) and 5.7% in the prototype, and the urban share by 1.6 points. Cost: output per head in 1751 to 1820 is 6.3% lower, so anything tuned to the early years shifts. Carters are not counted in the employment series. This is the change that makes railways (and canals and roads) matter to the standard of living.

**4. If only one line may change:** `rail_spur_scan`, which lets `plan_spurs` go down the list until it has seen 25 cells far enough from track. Alone it gives 2,228 km by 1900, and track within a cell of 67 to 70% of quarry cells (seeds 3 and 7; 4% before). It is weaker than 1: nothing is ever refused, towns one cell off stay off, and at realistic cost it is uneven.

**5. Railway companies (design only).** Pay a share of the fares on track to the subscribers as dividends, taken from the carters' income so money is conserved. Families could then hold railway shares, and promoters' hurdle could follow realised dividends, which is where manias and slumps would come from. The prototype has no manias: projects sit at the cap for decades.

**6. Hold: railway towns.** `rail_towns` lets a spot on the line 12 cells from any town, with twice `found_activity` at work, become a town. It adds 7 to 11 towns per seed between 1840 and 1900, but output per head falls 1.6% and the real wage 2.3% (thin markets). Without those two limits the map fills to `T_max` and two seeds stop on a money-conservation error I did not trace.

Not worth doing now: cut-off lines. Perfectly direct lines between all railed pairs would save at most 93,000 to 230,000 a year (16 to 27% of the trade carting bill), about a fifth of what branches save.

## 5. What a canal age needs from this code

- **Cost graph.** `transport::build_graph` (`transport.rs:67`) needs a canal layer with `cost = cost.min(CANAL_COST * step)` before the rail override. Navigable rivers cost 0.3 a cell, the same as a turnpike on lowland, and roads reach turnpike by themselves, so a canal must cost about 0.12 to 0.15 to matter. The graph has one cost for all goods; bulk goods gain most only through `weight`. An edge charge between canal and other cells, like `PORT_COST`, would give wharves.
- **Building.** `rail_build_graph` (`transport.rs:109`) generalises to a terrain table, a slope term (40 per unit of height for rail; several times that for locks) and a river term (a penalty for rail, a discount for canals).
- **Planner.** `plan`, `plan_branches`, `line_sums`, `materials` and `progress` generalise if the layer written, the running cost (`RAIL_COST`), the materials, the technology index (`st.t_rail`) and the hurdle come from a mode value carried by `Project`. `line_sums` already prices any set of cheaper edges. Records and the viewer need a transport bit for canals (`record.rs:191`).

## 6. Side findings

- **Land rent in a town's founding year.** `land::sector_bids` clamps the new town's year-mean wage, which is 0, to 1e-6 (`land.rs:136`), so rents in its catchment reach 1e9 to 1e21 for that year, and since the exponent differs by sector the bids between uses are distorted too (seed 13, 1750: 271 cells above 1e5; I did not measure the effect on land use). It happened at every founding in seed 13, the only seed I ran it on; `examples/probe_rent.rs` shows it. Families price land on this rent.
- The game has no parameter set of its own beyond `n_families`; these recommendations need one.

## 7. Not checked

