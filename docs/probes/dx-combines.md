# dx-combines: what a works holds, combined works, and ways in and out

Branch `night/dx-combines` (measuring code and a prototype, not for merging as they stand). Asked on 11 November 2026, after Stella's note that works should be drawn from what the simulation knows of them: "inventory sizes, different stages of production using different technologies, dilapidation, train stations, truck loading stations, industries combined not needing transport between them, i.e. vertical integration, but they're less flexible". Three questions: what a works holds, how the chains lie on the map and what a combined works should be, and what a railway or navigable water does for a works.

Everything is on seeds 1 to 8 under the game's rules with six families, at `rust-3d` 63f7b46. Probe: `crates/isostate-core/examples/probe_combines.rs` (`[--set JSON] [SEED ...]`), which steps a run month by month and reads the state between December and the yearly review. Its tables pool the eight runs (sums, then the ratio); `[lo .. hi]` is the least and greatest seed. Raw output: `.claude/night/reports/dx-combines/` (`base.txt`, `proto.txt`, `proto-merge.txt`, `proto-loose.txt`, and the sweeps' comparisons). Sweeps: `.claude/night/reports/sweep-dx-combines-base`, `-proto`, `-merge`, `-loose`. Two fields were added to the state for the probe alone (`month_exports`, `month_trade_cost`, kept only when `watch_markets` is set). With the prototype's switch off a sweep gives every one of the 244 facts exactly as before it was written, and `cargo test --release -p isostate-core` passes, the golden test included.

A second helper then went through this report as its critic (branch `night/dx-combines-review`): ran the measurements again, some of them with other code (`examples/probe_combines_review.rs`), swept eight more seeds, and worked the sums of a tie. The numbers stood; several readings of them did not. Section 6, "After the critique", has what was checked, what was found and the verdict. Where a sentence in sections 1 to 5 was wrong it has been corrected in place and marked "(corrected)" or "(added)"; nothing else in them was changed.

## In short

1. **A works holds nothing but its plant and its cash.** The only stocks of goods in the engine are the merchants', one per town and good: 1.1 to 2.2 months of sales, about two months of the island's output in all, steady all century. A works buys each month what it uses that month and is paid at once for all it makes. So a full yard at a works would be a guess today. What is true per works and year is its capacity, capacity on order, hands, utilisation, cash, and the town's stock of its good.
2. **The chains already stand close together, and nothing ties them.** A fifth to a quarter of the iron used is used on a cell where iron is made, another third on the next cell, the rest across the same town. (Added: that is where the user stands, not what could pass. The lesser of what each cell makes and what it uses is 10 to 18 per cent of the iron used, and that is the most one-cell ties could ever carry.) A fifth to three tenths of iron and steel crosses to another town. Carting an intermediate good to market costs 0.1 to 0.2 per cent of its price; the merchant's spread is 6 per cent and trade between towns 3.5 to 6 per cent. A works with its supplier next door is idle as often as one without (21 against 24 per cent of years), and four idle spells in five come while the supplier next door is busy.
3. **So a combined works saves the merchant's spread, not the cart.** The prototype (`combines.enabled`, off under both sets of rules) ties two works on one cell so that the good crosses the yard at cost. By the 1890s there are 58 ties a run, 17 per cent of works (added: and under 3 per cent of the works' hands), but they carry 6 per cent of the iron and 23 per cent of the tools: mostly a small using stage set down beside a large maker. Nothing is swallowed even with every limit loosened (8 per cent of iron). No fact of the sweep moves clear of the spread between seeds. Tied stages close half as often. (Corrected: this line went on to say that the tied part of a maker runs at its user's 0.71 where a free maker runs at 0.86, as the sign of "less flexible". That compares users with makers; a free user runs at 0.71 too. What the tie costs a concern is in section 6.)
4. **A railway is worth little to a works as things stand**, because the works already stand in and by the towns: carting is 0.1 per cent of the value made on the line and 0.4 per cent away from it. Nothing like a siding, station or wharf exists with a cost or a capacity; every cell of track is a station. The viewer may draw a siding wherever a works' own cell has track and a wharf wherever it is navigable, and nowhere else.
5. **After the critique: build it changed.** The record's row for a works and year can be built now, since everything in it but the yards is true today. Combined works as designed here would be more of a shelter than a less flexible firm. The tied row carries its maker's whole margin (five to ten times what the tie itself saves), so the using stage slows less than a free one when its own good falls; the two are judged as one; and a tied concern is at a loss in 13 to 17 per cent of years where a free user is in 26 to 36, and closes half as often. What it gives up is real in the rule (it never buys in the market, and its tied part sells nothing outside) but it is forgone profit that no book shows: in a year when one of its prices falls by a tenth the concern earns about a fifth less on its plant than a like mix of free works, and in other years 7 per cent more. And what it saves is the merchant's spread, a transfer: on sixteen seeds nothing on the island moves but the life of a works (a year longer). Section 6 says how to change it: books for each stage as if it stood alone, so the cost of the tie is a number; the stages kept in proportion; heat saved by the pair of methods, not the good; families' works included; and no small stage hung on a large maker.

## 1. What a works holds

### What there is

| stock | where | size | note |
|---|---|---|---|
| goods for sale | `State::inv` (town, good): the merchants' | below | the only stock of goods; prices are marked on it (`prices::mark`) and merchants stop taking a works' output past four months of it (`prices::merchants_take`) |
| a works' materials | nowhere | none | `econ::step_month` asks the market for `q_des * a` and makes what the month's fill allows (`phi`) |
| a works' unsold output | nowhere | none | all of `q` goes into `inv` and is paid for in the same month, less the spread |
| work in progress | nowhere | none | a month is one step |
| plant | `f_K`, and `f_order_K` with `f_order` (the capital goods still to buy) | 0.2 to 0.5 per cent of all capacity is on order at a year's end; under 1 per cent of works count as building | one number per works; 4 per cent wears out a year; no age, no vintage |
| land | `works_lease`, `f_rent` | | the most land the works has taken |
| cash | `f_cash` | 3.4 to 4.2 months of its costs | `firm_buffer_months` is 4 |
| machines | `machine_stock` (town, land sector and use) | | the machines mechanism is about farms, pits and quarries; a works' plant is `f_K` alone |

The books on `night/accounts` add, per works and year, 21 columns of flows: output, value, revenue, materials, fuel, carting in and out, hands, wages, rent, managers, profit, dividends, investment, utilisation, cash at the year's end and months worked. No stock, because there is none to report; the town's books have the merchants' stock at the year's opening and close, what spoiled and what was sold though not in store.

### The merchants' stocks

Months of sales and shipments held, every town's stock over every town's outflow, monthly. "Glut" is the share of sales made in towns whose stock stood over four months, where the works are on short time.

| | 1750-99 | 1800-49 | 1850-1900 | glut, by half-century |
|---|---|---|---|---|
| grain | 1.88 | 1.78 | 1.66 | 0, 0, 0 |
| coal | 1.38 | 1.23 | 1.13 | 0.01, 0, 0 |
| iron | 1.85 [1.7 .. 2.1] | 1.80 | 1.82 | 0.02, 0.05, 0.06 |
| steel | 1.77 | 1.65 | 1.59 | 0.01, 0.02, 0.01 |
| tools | 1.73 | 1.55 | 2.06 [1.4 .. 2.6] | 0.02, 0.02, 0.10 |
| cloth | 1.55 | 1.74 | 1.52 | 0.02, 0.03, 0 |
| bricks | 1.90 | 1.94 | 1.72 | 0.02, 0.03, 0.02 |
| machinery | 1.86 | 1.90 | 1.87 | 0.06, 0.07, 0.06 |
| rails | 2.17 | 1.93 | 1.90 | 0.09, 0.07, 0.06 |
| services | 1.24 | 1.22 | 1.16 | 0, 0.01, 0.01 |

The whole stock at the day's prices is 2.0 to 2.2 months of the island's output in every decade. (Charcoal after 1800 reads nine months: a dead trade's last stock with no sales under it.)

What a yard at a works would have to carry, from works not building, at each year's end:

| | 1750s | 1800s | 1850s | 1890s |
|---|---|---|---|---|
| materials short (smoothed fill under 0.9) | 0.010 | 0.086 | 0.092 | 0.074 |
| starved (under 0.5) | 0.001 | 0.004 | 0.003 | 0.003 |
| merchants not taking all it could make | 0.019 | 0.107 | 0.076 | 0.050 |
| utilisation under a half | 0.208 | 0.207 | 0.194 | 0.175 |
| mean utilisation | 0.70 | 0.75 | 0.75 | 0.75 |

A fifth of works run under half speed in any year, and for most of them neither materials nor the merchants are the reason: their margin is thin (`u` is a sigmoid of the margin over the price, centred on 3 per cent). Slumps in this engine arrive through prices. Rationing by quantity, which is what a yard would buffer, touches one works in ten.

### Should a works hold stocks of its own?

Yes, but only the least version, and only with a sweep on the price swings beside it.

**Yards.** Two numbers per works: `f_yard_out`, units of its good made and not yet sold, and `f_yard_in`, materials and fuel bought ahead, counted in units of output they are enough for (the recipe is fixed, so one number does for all inputs).

- Materials. The works asks the market for the month's need plus a share of what its yard lacks of `in_months` (first value 1) of its usual make. It makes what the yard and the month's deliveries allow. A month the market is short it works from the yard; `f_fill` then falls only when the yard is empty.
- Output. What is made goes into the yard. The merchants take from it as they do now: all of it while their own stock is under four months of sales, less beyond (`merchants_take` becomes a limit on sales, where today it is a limit on making). The works itself slows when its yard passes half of `out_months` (first value 2) of its usual make and stops adding at the full figure.
- Money. Materials are paid for when bought, as now, only earlier; output is paid for when the merchants take it. Nothing new is counted by `econ::money_total`. Spoilage (`decay`) applies in the yard as on the shelf.

Cost in the monthly step: a handful of multiplications for each of 200 to 700 works; nothing beside the wage search. In the state, two floats a works.

What it adds. A glut shows first as full yards with the hands still on, then as short time, and the yards empty before the hands come back: the sequence Stella asks to see. Unsold stock is wages paid with no revenue, so a long glut drains `f_cash` and the insolvency rule sorts the works with deep purses from the rest. And it is an inventory cycle: stock hidden from the merchants' price rule comes back onto the market when they buy again. That last is the risk. The prices mechanism was built to take a false cycle out of exactly this loop, and a second store of goods in the loop is a second lag. It must be measured as `prices.md` measured its own change (`price_swing_*`, `output_swing_*`, the farm zigzag) before it is believed.

What it does not add is carting time: a month is one step and carriage is instant, so "bought ahead against carting time" has nothing to stand on. Bought ahead against a short market does.

(Added after the critique.) Three things about the yards as specified, before anybody builds them:

- **They would be nearly still.** The merchants take all a works makes unless their stock is over four months, which is so for 5 to 11 per cent of works at a year's end, so the output yard is empty nine years in ten. The materials yard fills to its month once and is drawn on only when the market is short (7 to 10 per cent). A viewer gets a heap that is always one month high and a yard that is nearly always bare. That is honest, and it is little to draw.
- **The refill rule hoards.** "The month's need plus a share of what its yard lacks" asks the market for more exactly when the market is short, and every works does so at once. `works.md` has tools at ten to twenty times their price for years on some islands; this rule would deepen that. The least repair: no refilling in a month the good's fill is under one, and the sweep on the price swings has to include the tools famines (`tools: dearest year over the median`), not only `price_swing_*`.
- **One number for all materials does not cover fuel.** A works buys its materials in the recipe's proportion, cut by the scarcest (`phi`), so one number is right for those. But its fuel is a mix that moves with prices month by month (`heat_mix`), and the recipe's coefficients move with the works' size and skill. "Units of output the yard is enough for" then means a different heap of coal each year. Two builders would do different things here. Keep the yard in units of each good (a method has at most three materials and three fuels) and let the record carry months of throughput for the whole and the viewer split it by the recipe and the town's fuel mix of the year.

### What the record should carry per works and year

Today `run.bin` has one `f32` of built capacity per works and year (`firm_Kvals`, 60,054 works-years a run: 240 KB) and the works' table in the header.

| field | type | bytes | from | to be had |
|---|---|---|---|---|
| capacity built | f32 | 4 | `f_K` | now |
| share still on order | u8 | 1 | `f_order_K` | now |
| hands, the year's mean (added: in tenths; a works of under half a hand would read nought, and none passes 2,000) | u16 | 2 | `f_labor`, summed over the months as `town_hands_acc` is | now (the accounts keep it) |
| how hard it worked, the year's mean | u8 | 1 | output over capacity (the accounts' `util`) | now |
| state | u8 | 1 | 0 going up (nothing built, or over `built_share` on order), 1 working, 2 short time (under a half), 3 standing (under 0.15), 4 closed this year | now |
| output yard, tenths of a month | u8 | 1 | `f_yard_out` | with yards |
| materials yard, tenths of a month | u8 | 1 | `f_yard_in` | with yards |
| passed across the yard, share of output | u8 | 1 | `comb_passed` | with combined works |

Twelve bytes a works-year: 720 KB a run, 480 KB more than now, on a file of about 21 MB (the map layers are 18 MB of it). In the live view `LiveYear::firm_K` is 16 bytes a works; a row of these fields is about 32, so 22 KB a year for 680 works, sent once.

Three things asked for need no bytes:

- **Railway or navigable water on its cell**: the `trans` layer of that year (corrected: track open is the value 4 in it, mask `0x04`, and 8 is track being built; `record::snapshot`) and the static `nav` layer, at the works' cell.
- **The year and size of each enlargement**: the capacity series already shows them, since an order is delivered within the year (a quarter of what remains each month). A run has 18,837 enlargements, one works-year in 3.2, the median 16 per cent of the works' capacity (quartiles 12 and 24). So a works of 1900 is honestly a first range and a dozen or two additions.
- **Derelict**: `died` in the works' table, and the cell's `use`.

(Added.) Two things the table needs before it is built. The record keeps a works' series from the first year it has capacity (`record::pack` cuts each row to the years `K > 0`), so "going up, nothing built" would have no row to sit in: the rows must start at the year the works was founded. And the ground a works takes, which the viewer needs to fill the space it is given, is not a field but is to be had from the row: capacity built and on order, times its method's hands a unit, over `works.hands_per_cell` (`works::land_take`).

One thing asked for does not exist: **the method or vintage an enlargement came with**. A works has one method for life and one productivity for all its plant (`f_prod`: the method's learning, the town's knowledge, its neighbours). An old range is as good as a new one. To make ranges of different ages true the least version is one more number per works, the capacity-weighted mean year of its plant (an order of `dK` in year `y` moves it toward `y`; wear does not), with `f_prod` taking the method's skill as it stood in that mean year rather than today's. Old works would then be worse than new ones of the same method, a works that stops growing would fall behind, and "dilapidation" would be a number. Not built; it changes how fast learning reaches output, so it needs its own sweep.

Until yards exist the honest drawing of stock is the town's: months of sales of each good in its merchants' hands (town, good and year: 116 KB as bytes). Every works of a good in a town may show that figure; none has a figure of its own.

## 2. How the chains lie, and combined works

### The chains today

Four goods are made by one method and used as a material by another: charcoal (for bloomeries and charcoal furnaces), iron (for smithies, millwrights, engine works, crucible steel, converters, forges and rolling mills), steel (for steel rail mills) and tools (for millwrights and engine works). Machinery and bricks go into plant, not into a recipe.

Where the user stands against its nearest maker, as a share of the good used: on the same cell, on a neighbouring cell, elsewhere in the same town's market area, or with no maker in the town.

| | iron: cell / next / town / none | steel | tools |
|---|---|---|---|
| 1750s | 0.08 / 0.32 / 0.60 / 0 | | 0.22 / 0.39 / 0.38 / 0.01 |
| 1780s | 0.24 / 0.38 / 0.38 / 0 | | 0.08 / 0.48 / 0.43 / 0.02 |
| 1810s | 0.21 / 0.35 / 0.43 / 0 | 0.48 / 0.09 / 0.43 / 0 | 0.16 / 0.15 / 0.45 / 0.24 |
| 1840s | 0.21 / 0.33 / 0.46 / 0 | 0.38 / 0.07 / 0.54 / 0 | 0.36 / 0.25 / 0.39 / 0.01 |
| 1870s | 0.22 / 0.28 / 0.49 / 0 | 0.14 / 0.20 / 0.64 / 0.02 | 0.28 / 0.26 / 0.43 / 0.02 |
| 1890s | 0.19 / 0.28 / 0.53 / 0 | 0.25 / 0.13 / 0.59 / 0.02 | 0.27 / 0.29 / 0.44 / 0.01 |

Charcoal is the exception: 96 to 99 per cent of it is used across the town from where it is burnt and under 3 per cent on the same or the next cell; the burners are in the woods. From the maker's side, 29 to 43 per cent of iron after 1780 is made on a cell where some of it is used. By using method, 1850 to 1900, the share of the material used on a maker's cell: rolling mill 0.23 [0.04 .. 0.37], engine works 0.24 for iron and 0.28 for tools, converter 0.22, steel rail mill 0.23 [0.09 .. 0.53], smithy 0.19, crucible steel 0.13.

They stand together because the life cycle's walk to work and the kept town ground put every light trade in and by the town: in 1900 the island's 683 works stand on 325 cells, and 165 of those cells have two or more.

What moving and selling an intermediate good costs, as a share of its price:

| | sent to another town, share of the make | carriage and handling of what was sent | maker's carting to market | user's carting from market | merchant's spread |
|---|---|---|---|---|---|
| charcoal, 1750-99 | 0.15 | 0.076 | 0.019 | 0.001 | 0.06 |
| iron, 1750-99 | 0.18 | 0.052 | 0.001 | 0 | 0.06 |
| iron, 1800-49 | 0.23 | 0.058 | 0.002 | 0 | 0.06 |
| iron, 1850-1900 | 0.25 | 0.040 | 0.002 | 0 | 0.06 |
| steel, 1850-1900 | 0.29 | 0.035 | 0.001 | 0 | 0.06 |
| tools, 1850-1900 | 0.41 | 0.031 | 0 | 0 | 0.06 |

(A load passed on through two towns is counted twice in the first column.) Iron is worth six to eight a unit and weighs one; freight is 0.015 a unit of weight and path cost, and works stand under one unit of path cost from their market. Carting is nothing beside the price. What stands between a maker and a user in the same town is the merchant: the maker gets the price less 6 per cent, the user pays the price. Between towns there is also 3 per cent handling and the freight.

One oddity found on the way: a user on a maker's cell already pays the maker's gate price for the good (`land::delta_fields` treats every producing cell as a source), while the maker is still charged carting to market on all of it, and the merchants make up the difference. It is too small to matter here (under 0.1 per cent of the good's value), but the two ends of that sum do not agree.

Idle and closed, by where the partner stands (works not building; idle is utilisation under a half at the year's end):

| | works-years a run | mean utilisation | idle | idle while the partner ran at 0.8 or more | closed in the year |
|---|---|---|---|---|---|
| user, a maker on its cell or the next | 13,573 | 0.71 | 0.215 | 0.175 | 0.029 |
| user, a maker only elsewhere in town | 12,273 | 0.69 | 0.239 | 0.197 | 0.032 |
| user, none in town | 597 | 0.54 | 0.437 | | 0.056 |
| maker, a user on its cell or the next | 9,068 | 0.77 | 0.181 | 0.106 | 0.019 |
| maker, a user only elsewhere in town | 5,996 | 0.75 | 0.207 | 0.137 | 0.026 |
| maker, none in town | 7,379 | 0.74 | 0.174 | | 0.025 |

A user beside its maker is idle in 21.5 per cent of years, and in 17.5 of those 21.5 the maker next door is running at four fifths or better. Neighbours do not share a fate: each sells to and buys from the town, and what the works next door does reaches it only through the town's price. That is the thing a combined works changes.

### The design

**What it is.** One concern running two or more consecutive stages on one site. In the engine each stage stays a works of its own method, with its own plant, land and row in the record, and the stages are tied: `comb_feeds[u] = d` where `u` makes the good and `d` uses it. The tied part of the two runs as one works that makes `d`'s good from `u`'s materials: `d`'s recipe with the intermediate good struck out and `u`'s recipe, labour and heat added in the proportion `d` uses the good. Its capacity is the lesser of `d`'s and what `u` can feed. What `u` makes beyond that it sells as before.

**What it saves**, in the engine's own terms:

- the merchant's spread on the good, 6 per cent of its price: the whole of the saving within a town;
- handling and freight, 3.5 to 6 per cent, where the stage it replaces would have bought from another town;
- the merchants' stock of the good (1.6 to 2 months of it today) and, with yards, the two yards between the stages;
- heat where the metal goes on hot: `hot` (0.3) of the using stage's heat, for iron and steel;
- carting: 0.1 to 0.2 per cent of the good's value. Not a reason.

**What it gives up.** It takes its own stage's output whether or not the market's is cheaper: a tied `d` buys none of the good. Its stages are tied in proportion, so the tied part of `u` runs as hard as the last good sells and no harder. More plant stands on one site under one purse. The hurdle to add or found a stage is `hurdle_mult` (1.25) times the usual. A tied stage grows only in step with its other stage, and it keeps its method until the concern fails or is broken up, since a method is a works and a works is not re-equipped.

**How one comes to be.**

1. *Founded whole* (designed, not built). In `firms::entry`, for each pair of methods the town knows where one's output is the other's material, reckon the two as one on the candidate cells of both: `evaluate_sites` for the using method with the good costed at the making method's own unit cost on that cell, over the plant of both, against `hurdle * hurdle_mult`. It goes into the same list of proposals and is backed if the town's savers can find the money for both plants at once. Two works are opened and tied.
2. *A stage added to a standing works* (built: `combines::propose`). Where a trade would found a new works, it also reckons the same works on the cell of a free works that makes its material, with the material costed at what that works gets for it at its gate after the spread, and the heat saved; or on the cell of one that uses its product, with the product valued at what that works pays for it delivered. If that beats the best free site and clears the higher hurdle, the stage is built there, sized to its partner within the year's room and the cell, and tied. A free works is one that has no family over it, is in no tie, is not building and ran at `min_u` (0.65) or better.
3. *Neighbours brought under one owner* (built as a rule, off: `combines.merge`). Two free works on one cell, the maker's capacity between `merge_fit` (a half) and its inverse of the user's need, are tied by the review. For a family this is the decision to buy the works next door and run the two as one; the rule is what makes it possible, at the works' value as `families::works_value` has it.

**How it ends.** When either stage closes the other stands on alone (built). A tie between works of different owners is dissolved at the review (built). Two more are designed: the review keeps shadow books for each stage as if it stood alone (`u` selling at its gate less the spread, `d` buying in the market); if the concern has lost money two years running and one stage's shadow return is over half the hurdle, the other stage is closed; and if the two shadow returns together beat the joint return by a quarter three years running, the tie is cut and both live. That last is the escape from a tied furnace whose iron the market now makes cheaper.

**In the monthly step** (`econ::step_month`, five calls): `tie_rows` after the coefficients are worked out rewrites the two rows; `prev_hands` gives the tied part last month's hands; `one_purse` after the profit loop sweeps `u`'s cash into `d`'s; `after_month` at the end books what crossed the yard to `u`'s method (its output series, its learning, the good's cumulative output that discovery thresholds read) and `u`'s share of the hands to `u`. The wage search is untouched: the tied part is one more Leontief works whose demand for hands falls as the wage rises.

**In the review** (`firms::review`): `combines::new_year` shares the purse between the stages by the value of their plant and works out the concern's return and utilisation; `judged` hands those to the loop for both stages, so they grow, shrink, lose years and fail together; `in_step` lets `d` order no more than `step` (1.25) times what `u` can feed; `untie` in `close_firm`.

**In ownership** (`families.rs`): nothing yet. A family's works are never tied by the prototype. The design: a family may tie two of its own works on one cell, buy a neighbour to do so, and cut a tie; rivals do it where a year's saving on the spread is over a tenth of the smaller works' profit.

**Parameters** (`CombineParams`): `enabled` false; `hurdle_mult` 1.25; `hot` 0.3 and `hot_goods` iron, steel; `backward` true; `merge` false; `merge_fit` 0.5; `min_u` 0.65; `step` 1.25.

**In the record**: per works, the partner it feeds or is fed by and the years the tie began and ended (the prototype writes the tie as it stood at the end: `feeds`, `fed`, `tied` in the works' table); per works and year, the share of its output passed across the yard.

**Facts that would show it works**: for each intermediate good, the share of its make passed across a yard at each mark; works and hands in ties; ties made and dissolved; the sizes of concerns (the hands of both stages); carters' share of hands; utilisation and closures of tied and free works in slack years and brisk ones.

**What it should resemble.** Coke ironworks on the coalfields that put forges and rolling mills beside their furnaces from the 1780s, until most of the bar iron of such a district came from works that smelted their own pig; converters set down at blast furnaces so the metal goes on hot, with their rail mills; a colliery with coke ovens and furnaces; mills that spun and wove under one roof in the first half of the century and, in the largest cotton district, came apart again into spinners and weavers in the second. And beside all of them the trades that stayed apart: edge-tool makers and cutlers buying bar and steel, foundries and engine builders buying pig from merchants, market towns living on the trade between them. A share of each chain, different by chain, rising and in places falling.

**What it must not do**: take every chain. Three things hold it back in the design: a tie needs the two stages on one cell, which is where a fifth to a quarter of the iron is used today; the higher hurdle; and the break-up rule. If the tree's deeper chains arrive, the pairs to watch are lime with builders, gasworks and soda; sawn timber with builders; bricks with bricklayers; chemicals with glass and paper; steel with tool factories and engineering works; gas and paper with shops. The tree's design left out pig iron, yarn and coke as goods because each would be a thin market with one buyer; a tie is the place such a good could live without a market, and that is worth a second look once ties exist.

### The prototype's result

`--set combines.enabled=true`, eight seeds, against the same seeds without it. All money conserved in every run. What is built is points 2 and 3 above with the two built endings; a works is in one tie at most; a tied user that outgrows its maker idles the excess; family works are left out.

Ties, by decade (means over the years of the decade and the eight runs):

| | ties standing | new stages added | share of works in a tie | iron passed across a yard, share of the make | tools passed |
|---|---|---|---|---|---|
| 1750s | 12.8 | 3.8 | 0.12 | 0.030 [0.01 .. 0.07] | 0 |
| 1780s | 16.2 | 7.4 | 0.14 | 0.150 [0.12 .. 0.24] | 0.001 |
| 1810s | 16.5 | 6.9 | 0.11 | 0.066 [0.03 .. 0.13] | 0.002 |
| 1840s | 28.9 | 9.8 | 0.12 | 0.055 [0.03 .. 0.12] | 0.042 [0 .. 0.22] |
| 1870s | 45.5 | 11.8 | 0.15 | 0.061 [0.03 .. 0.12] | 0.168 [0.04 .. 0.30] |
| 1890s | 58.2 | 13.4 | 0.17 | 0.062 [0.02 .. 0.12] | 0.227 [0.11 .. 0.36] |

Charcoal: 2 to 3 per cent passed before 1780. Steel: under one part in a thousand. (Most of the dozen ties of the 1750s were made in the warm-up decade, which runs under the same rules.)

Which stages, 1850 to 1900 (ties standing; the hands of both works together; the share of the maker's capacity its tied user can take):

| making stage | using stage | ties | hands | share of the maker taken |
|---|---|---|---|---|
| smithy | engine works | 16.4 | 1,105 | 1.12 |
| coke furnace | smithy | 7.9 | 337 | 0.03 |
| coke furnace | engine works | 6.1 | 443 | 0.41 |
| coke furnace | converter | 4.3 | 379 | 1.17 |
| coke furnace | rolling mill | 4.2 | 156 | 0.25 |
| converter | steel rail mill | 3.5 | 289 | 0.01 |
| coke furnace | crucible steel | 2.4 | 125 | 0.94 |

Two kinds. The smith's shop of an engine works, the converter at a furnace and the crucible works at a furnace are whole: each stage takes all the other makes. The rest are a small user beside a large maker (a rail mill taking a hundredth of a converter's steel): the new stage is sized by the year's room in its own trade, which is small where the trade is new, and then grows at the room's pace. Before 1800 the same pattern holds with charcoal furnaces, and those ties die with charcoal iron.

With `merge` on as well: 70 ties in the 1890s, 20 per cent of works, 7 per cent of the iron and 28 per cent of the tools. With every limit loosened (`merge` on, `merge_fit` 0.05, `hurdle_mult` 1): 77 ties, 22 per cent of works, 8 per cent of the iron (20 in the 1780s), 27 per cent of the tools. It does not run away. The ceilings are the one-cell rule, one tie a works, and the family works left out.

What moved in the sweep. None of the 244 facts is starred in any of the three comparisons. Those that lean the same way in all three (mean, with seeds up/down against the baseline):

| | without | prototype | with merge | loosened |
|---|---|---|---|---|
| baskets per head, 1900 | 7.82 | 8.23 (7/1) | 8.17 (7/1) | 8.21 (7/1) |
| real wage 1900 over 1750, chained | 4.13 | 4.29 (7/1) | 4.23 (6/2) | 4.21 (5/3) |
| real wage 1900 over 1750, fixed basket | 2.59 | 2.59 (4/4) | 2.55 (4/4) | 2.60 (5/3) |
| works closed in a run | 1,477 | 1,408 (2/6) | 1,416 (1/7) | 1,411 (2/6) |
| median life of a works that closed | 10.8 | 11.8 (6/1) | 11.8 (5/0) | 11.6 (5/2) |
| median size of a works, 1900 | 5.72 | 5.43 (1/7) | 5.42 (1/7) | 5.15 (1/7) |
| works in 1900 | 701 | 711 (4/4) | 725 (5/3) | 733 (6/2) |
| carters' share of hands, 1900 | 0.058 | 0.060 (6/2) | 0.059 (6/2) | 0.060 (4/4) |
| iron's price swing | 0.172 | 0.170 (3/5) | 0.172 (2/6) | 0.169 (3/5) |

No carting hands are saved, as the measurements said there would not be. Fewer works close and they last a year longer. Works are a little smaller at the median because the added stages are small. Households end the century some 5 per cent better off by the basket measure on seven islands of eight in all three variants and no better by the fixed-basket wage; I have not traced why (below). (Corrected: it is chance. On seeds 9 to 16 baskets per head in 1900 are 8.23 with the prototype and 8.24 without, five seeds up and three down; and with no heat saved, `combines.hot=0`, seeds 1 to 8 still read 8.19 against 7.82. Seeds 1 to 8 without the prototype happen to stand low. Of this table only the longer life of a works holds on both sets of seeds: 11.0 to 12.5 years on seeds 9 to 16, six up and one down, as here.)

Through slack and brisk markets (a slack market: the town's stock of what the concern sells stands over three months at the year's end):

| | brisk: works-years a run | mean utilisation | closed | slack: works-years | mean utilisation | closed |
|---|---|---|---|---|---|---|
| tied using stage | 3,378 | 0.71 | 0.017 | 712 | 0.58 | 0.019 |
| tied making stage, with its concern | 3,388 | 0.71 | 0.009 | 717 | 0.58 | 0.005 |
| free user of an intermediate good | 18,310 | 0.71 | 0.030 | 3,062 | 0.59 | 0.034 |
| free maker of one | 4,383 | 0.86 | 0.013 | 550 | 0.73 | 0.029 |

This is where "less flexible" shows. A free maker runs at 0.86 in a brisk market for its good; the tied part of a maker runs at 0.71, its user's rate, whatever iron is fetching. In return the concern closes about half as often as a free user (1.7 to 1.9 per cent a year against 3.0 to 3.4): it is judged on the joint return, with one purse, and it started beside a works that was running well.

(Corrected after the critique.) The second row of this table is the first row again: the probe prints for a tied maker its user's utilisation (`st.f_u[sells]`), so the two agree by construction, and a free user runs at 0.71 and 0.59 as well. The table shows that a tied user runs as hard as a free one and closes half as often, and nothing about flexibility. Nor is "the tied part of a maker" most of a maker: seven tenths of the tied makers' capacity is beyond what their users take and is sold as before. Section 6 has a table that does bear on the question.

One thing to watch: the steel rail mill's share of rails in 1890 is 0.22 without the prototype and 0.07 with it (seeds 1/6). Rolling mills tied to furnaces could be what holds the iron rail in place, which would be the design's "older technology for longer" arriving unasked. But the other two variants do not lean that way (0.18 on seeds 4/3 and 0.15 on 3/4), and the method is thinly built in every run, so it is more likely chance. (Added: on seeds 9 to 16 it goes the other way, 0.24 with the prototype against 0.16 without, four up and four down. Chance.)

The prototype costs five seconds a run in the sweep (22 against 17). I believe it is all in `propose`, which looks through every works for every town and trade each year; a list of free works by town would remove it. Not profiled. (Corrected: one second. Run back to back twice on a machine with nothing else of mine on it, the sweep takes 23 and 22 seconds without the prototype and 24 and 23 with it. The five seconds were the machine's other work.)

Where it got hard, and stopped: founding whole (the sums are easy, raising money for two plants in one proposal and naming the pair are not hard either, but it doubles the entry code and there was not the time); the shadow books for breaking up; a tied user buying its shortfall in the market, which needs one works to be two rows in the month's tables; chains of three; anything for families.

## 3. Ways in and out

### What the line and the water do for a works today

Both are cheaper edges in one cost graph (`transport::build_graph`): 0.07 a cell between two cells with track, 0.3 between two navigable river cells, against 0.3 for a turnpike on lowland and 1.0 for open lowland. A works meets that graph in three places, all through its cell's cost to market (`dcatch`) and the delivered-price fields built on the same graph (`land::delta_fields`): the carting of its output (`freight * weight * dcatch`), the delivered price of its materials (`delta`), and, with the life cycle, its labour (`works.walk`: 2 per cent more hands a unit of output for each unit of path cost). Entry weighs candidate cells by `exp(-dcatch / 10)` and two and a half times for water (`firms::candidate_cells`).

Nothing is a siding, a station or a wharf. Every cell of track takes on and sets down at no charge. On the coast only a town's own cell is a port (`PORT_COST` 0.6 against 4.0 for a beach). There is no capacity anywhere: a line carries any weight at the same rate.

| | on a cell with track | next to one | further off |
|---|---|---|---|
| share of works' hands, 1830s | 0.19 | 0.28 | 0.53 |
| 1850s | 0.38 | 0.47 | 0.15 |
| 1890s | 0.37 | 0.54 | 0.10 |
| share of the land, 1850s | 0.13 | 0.32 | 0.55 |
| 1890s | 0.22 | 0.54 | 0.24 |
| works founded in the 1850s | 0.48 | 0.29 | 0.22 |
| works founded in the 1890s | 0.42 | 0.48 | 0.11 |
| works' cost to market (hands' mean), 1850s | 0.22 | 0.57 | 0.88 |
| the land's own cost to market, 1850s | 0.81 | 1.56 | 3.78 |
| works' carting over the value they make | 0.001 | 0.002 to 0.003 | 0.004 |

Works are on the line because the line runs to the towns, where the works already were: their mean cost to market is under one unit wherever they stand. Being on the line rather than off it is worth 0.3 per cent of the value made in carting and, through the walk, about 1.3 per cent of the wage bill (0.02 times 0.66 of path cost). Small beside a margin.

(Added after the critique.) That is the last mile only: the works' own cell to its own town's market. What the line does for a works beyond that reaches it through its town's prices and is not in this table. The second table of section 2 has one piece of it: the carriage and handling of iron sent between towns is 5.8 per cent of its value in 1800 to 1849 and 4.0 per cent in 1850 to 1900, of which 3 points are handling in both, so the freight on a quarter of the iron fell from about 2.8 per cent of its value to 1.0. What the line took off the price of coal and ore at a furnace's town was not measured, and for a furnace that moves 8.3 units of weight a unit of iron it is likely the larger part. So read "worth little to a works" as "worth little to which cell of its town a works stands on".

Works founded on the line or next to it, before and after it came (each founding counted as its share of that year's foundings on the island, per thousand cell-years of line):

| years from the line's opening | -30 to -20 | -20 to -10 | -10 to 0 | 0 to 10 | 10 to 20 | 20 to 30 |
|---|---|---|---|---|---|---|
| on the line or next to it (7,593 foundings) | 0.69 | 0.74 | 0.72 | 1.02 | 0.81 | 0.86 |
| two cells off or more (368) | 0.042 | 0.031 | 0.047 | 0.032 | 0.028 | 0.048 |

Founding near a line is two fifths higher in the ten years after it opens and a sixth higher after that. The line is promoted where carting is heaviest, so some of this is the growth that brought the line; there is no clean comparison, since by 1900 almost no works is founded more than a cell from where track runs.

Water: works' hands on a navigable river cell are 13.6 per cent in 1750 to 1799, 10.3 in 1800 to 1849 and 3.8 after, on 1.9 per cent of the land; on other river or coast 18.7, 12.5 and 6.0 on 9.9 per cent. The early towns are on the water and the works with them; the share falls as the towns grow away from the bank.

### What to propose

Only this earns its keep now:

- **Nothing in the engine for works.** A siding with a cost would be an outlay to save part of 0.3 per cent of output. It cannot change a decision.
- **What the viewer may draw meanwhile.** A siding at a works whose own cell has track in that year's `trans` layer: true, because the works on that cell does load at the rail rate from its door. No siding at a works next to the line: the graph carts its goods a cell by road first. A wharf at a works whose cell is navigable (`nav`). A quay only on a town's own coastal cell. How busy to draw them: the weight the works moves a month, its output times the weight of what it makes and uses, which the record's capacity, utilisation and method give.

Worth building later, in this order:

1. **A halt charge**, when the tree's bulky works arrive (lime, cement, the continuous kiln, sawmills, the pulp mill: usual sizes in the hundreds and thousands, weights of 1.6 to 2.2). Goods get on and off the rail only at a halt, with a charge like `PORT_COST`; halts are the towns, the head of every branch, and sidings. A siding is then a works' own purchase (a cell's worth of rails, timber and stone at the railway's prices) made when a year's saving clears the hurdle, and a fact in the record. A coke furnace moves 8.3 units of weight for each unit of iron (its ore, its coal and the iron); at a charge of half a unit of path cost a siding would save it 0.06 a unit, about 1 per cent of the iron's price, where all its carting today is 0.1 to 0.4 per cent. That is enough to tip a site. It would also end the oddity dx-rail noted, that every cell of track is a station, which matters more for the farms and quarries than for works (I did not measure how the carting bill splits between them).
2. **Wharves** fall out of the same charge on navigable water and on canals, and log driving (the tree's stage 2) needs the boom at its foot to be a place.
3. **Capacity** (a line or a wharf that fills): not until something queues. Nothing in the engine waits for anything today.

## 4. What I am unsure of

- **Why households come out 5 per cent ahead.** The spread is a transfer: merchants' income is paid out to households, and no hands or goods go into it. A tie that saves the spread moves money from the merchants' dividend to the concern's and should change little for the island. Yet baskets per head in 1900 are up on seven seeds of eight in all three variants. It may be the heat saved, more investment drawn by the higher returns of tied stages, fewer closures, or the luck of a different run on each island; the fixed-basket wage does not move. I did not take it apart, and would not build on it. (Settled by the critique: chance. It is not there on seeds 9 to 16, and it is there on seeds 1 to 8 with no heat saved.)
- **Whether the saving is believable at all.** In this engine the merchant costs 6 per cent and does nothing a combined works then has to do for itself: no stock to hold, no credit, no selling. The honest version of the saving waits for yards (the concern carries the stock the merchant carried) and for merchants who cost something.
- **The sizing of an added stage.** It is held to the year's room of its own trade, which is why most ties are a small user on a large maker and why 17 per cent of works carry 6 per cent of the iron. Founding whole, in proportion, is the part of the design the prototype does not test, and it is the part that would look like an integrated ironworks.
- **One cell.** A works may take four fifths of a cell and a cell holds 2,000 hands, so two stages on one cell are small by construction: no works has over 2,000 hands and the largest concerns here have a few hundred. A third of the iron is used on the cell next to its maker. A tie across neighbouring cells, with the yard's tramway as a cost, is probably needed before a combined works is the largest thing on the map.
- **Idle is measured at December** from a smoothed utilisation (a tenth of each month's), under a half. It lags a slump by some months.
- **The railway's effect on founding** has no control group, as said.
- **The share sent between towns** counts a load once for each town it leaves.
- **Family works** are never tied. They were about three in ten of all works when the life cycle was measured (`docs/probes/dx-firms.md`); I did not count them here. Every share above is of a field with those owners left out. (Counted by the critique: from 1780 on a family owns 19 to 24 per cent of all works, 22 to 28 per cent of the works that make or use an intermediate good, and 41 to 49 per cent of those works' plant by value. So the prototype's ties are drawn from little more than half the chains' capital.)
- **Nothing tests the switch** beyond the sweeps running clean and the facts with it off being unchanged. `tests/` has no test of a tie.
- **The deeper tree** (`night/tree`) was not loaded; its pairs are named from the design, not measured.

## 5. How to run it again

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules game --families 6 --out BASE
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules game --families 6 --set combines.enabled=true --out PROTO --compare BASE
cargo run --release -p isostate-core --example probe_combines                                        # the tables of sections 1 to 3
cargo run --release -p isostate-core --example probe_combines -- --set '{"combines":{"enabled":true}}'   # and the ties
cargo run --release -p isostate-core --example probe_combines_review                                 # section 6: what could cross a yard, the pairs, the sums
cargo run --release -p isostate-core --example probe_combines_review -- --set '{"combines":{"enabled":true}}'   # and how tied concerns fare
```

The critique's output is in `.claude/night/reports/dx-combines-review/` (`review-base.txt`, `review-proto.txt`, the first probe's two outputs again, and the sweeps' comparisons); its sweeps are `sweep-dx-combines-review-base`, `-proto`, `-nohot`, `-base-s9` and `-proto-s9` beside the others.

## 6. After the critique

By a second helper, on `night/dx-combines-review`, the same day. The task was to take nothing on trust: run again what the design rests on, attack the design as an economist and as an engineer, and say whether to build it. Everything is on seeds 1 to 8 under the game's rules with six families unless it says otherwise. The new program is `crates/isostate-core/examples/probe_combines_review.rs`; it reads the state after December and again after the yearly step, like the first, but shares no code with it beyond the engine.

### What was run again

| | the report | run again | how |
|---|---|---|---|
| iron used on a cell where iron is made, 1780s / 1840s / 1890s | 0.24 / 0.21 / 0.19 | 0.24 / 0.21 / 0.19 | other code |
| iron used on such a cell or the next | 0.62 / 0.54 / 0.47 | 0.62 / 0.54 / 0.47 | other code |
| steel and tools on a maker's cell, 1840s and 1890s | 0.38, 0.25 and 0.36, 0.27 | the same | other code |
| a maker's carting to market over the value made, iron, by half-century | 0.001 / 0.002 / 0.002 | 0.0014 / 0.0022 / 0.0016 | other code |
| a user's carting from market | 0 | 0.0002 to 0.0004 | other code |
| iron sent to another town, and its carriage and handling | 0.18 / 0.23 / 0.25, and 0.052 / 0.058 / 0.040 | the same | the same two fields of the state |
| the merchant's spread | 0.06 | 0.06 | it is a parameter (`merchant_spread`); there is nothing to measure |
| the first probe's output, without the prototype and with it | `base.txt`, `proto.txt` | the same to the byte | run again and compared |
| the sweep: baskets per head 1900, works closed, median life | 7.82 to 8.23 (7/1), 1,477 to 1,408, 10.8 to 11.8 | the same, every figure | both sweeps run again |
| facts starred | none of 244 | none of 244 | |
| the switch off, against the baseline taken before the prototype was written | every fact the same | 0 of 244 differ | swept against `sweep-dx-combines-base` |
| seconds for the sweep, eight runs at a time, without and with | 17 and 22 | 23 and 24, then 22 and 23 | back to back, twice |
| a run's record | about 21 MB | 21.3 MB (seed 3) | one run written |
| the tests | pass | pass, the golden test among them | `cargo test --release -p isostate-core` |

The measurements stand. Money was conserved in every run here too: no sweep had a failed run and no probe run stopped.

### What the numbers do not say (each corrected above, in place)

1. **Where a user stands is not what could pass.** "A fifth to a quarter of the iron is used on a cell where iron is made" counts all of a large steel works' iron if a bloomery shares its cell. What could cross a yard is the lesser of what a cell makes and what it uses:

   | share of the good used | iron: on a maker's cell | iron: could cross a yard on one cell [least and greatest seed] | iron: could cross to the cell or its neighbours (an upper bound) | iron: the town makes it | steel: one cell | tools: one cell |
   |---|---|---|---|---|---|---|
   | 1780s | 0.24 | 0.15 [0.05 .. 0.24] | 0.53 | 0.86 | | 0.08 |
   | 1810s | 0.21 | 0.15 [0.03 .. 0.28] | 0.47 | 0.82 | 0.48 | 0.15 |
   | 1840s | 0.21 | 0.13 [0.03 .. 0.19] | 0.43 | 0.80 | 0.36 | 0.32 |
   | 1870s | 0.22 | 0.12 [0.05 .. 0.21] | 0.36 | 0.82 | 0.14 | 0.24 |
   | 1890s | 0.19 | 0.10 [0.04 .. 0.15] | 0.32 | 0.82 | 0.25 | 0.22 |

   So ties on one cell could carry 10 to 18 per cent of the iron at the very most, and the prototype's 6 per cent is a third to a half of what was there to take. Ties across neighbouring cells could carry a third to a half. A town makes four fifths of the iron it uses.
2. **The ties are small.** Seventeen per cent of works are in a tie in the 1890s and 2.7 per cent of the works' hands (2 to 4 per cent in every decade): about sixty hands a concern. Seven tenths of a tied maker's capacity is more than its user takes and is sold as before, and a tied user has 11 to 24 per cent of its capacity that its maker cannot feed, which stands idle for good (`in_step` lets the user grow to `step`, one and a quarter times what the maker feeds, and the monthly step takes the lesser).
3. **The slump table compared users with makers** (it printed the user's utilisation for the tied maker). It says nothing about flexibility.
4. **The households' 5 per cent is chance, and so is the steel rail.** Seeds 9 to 16, with the prototype against without: baskets per head in 1900 8.23 against 8.24 (5/3), the chained real wage 4.33 against 4.33, steel rails in 1890 0.24 against 0.16 (4/4), works closed 1,272 against 1,394 (3/5), the median life of a closed works 12.5 against 11.0 (6/1). With no heat saved, seeds 1 to 8 still read 8.19 against 7.82. Across both sets of seeds the facts that lean the same way by four seeds or more are the life of a works, the chronicle's stories about districts (34 a run for 39, and 34 for 40; a round of closures is one kind) and a few thousand fewer people in 1900 (0.4 and 0.9 per cent, 2/6 both times). That is all the prototype does to the island.
5. **It costs a second a run, not five.**
6. **Families own nearly half of what could be tied**: 41 to 49 per cent of the chains' plant by value from 1780, 22 to 28 per cent of their works by count.

### What was measured besides

Pairs of works standing on one cell, one making what the other uses, without the prototype (a furnace with three users is three pairs):

| 1850 to 1900 | pairs standing | in proportion (the maker makes a half to twice what the user needs) | no family owns either | both |
|---|---|---|---|---|
| smithy and engine works | 40.6 | 0.42 | 0.84 | 0.38 |
| coke furnace and engine works | 33.5 | 0.34 | 0.80 | 0.30 |
| coke furnace and converter | 19.0 | 0.37 | 0.61 | 0.26 |
| coke furnace and smithy | 15.0 | 0.05 | 0.80 | 0.03 |
| coke furnace and crucible steel | 7.7 | 0.51 | 0.79 | 0.45 |
| converter and steel rail mill | 5.9 | 0.01 | 0.66 | 0 |
| coke furnace and rolling mill | 5.3 | 0.18 | 0.61 | 0.14 |

About 128 pairs stand on a cell together and 35 of them are in proportion and free of a family: that, and not the 58 ties, is the stock of concerns that would look like one works. In 1800 to 1849 it is 86 pairs and about 17.

The sums of a tie, from the runs. For each town and year, a new works of the using method on the cell of the town's largest maker, at the year's mean prices and wage, as `firms::evaluate_sites` reckons it and `combines::propose` adds to it. Means over the decade's town-years on eight seeds; shares are of the using stage's price.

| | price | iron in the price | town-years a run where a free works clears the hurdle | its margin in those | the tie adds (spread and heat) | its return, free and tied | the maker's margin a unit of the user's output |
|---|---|---|---|---|---|---|---|
| **1800s**: iron at 7.3, the coke furnace's margin 0.45 of that, its return at its size 0.11 | | | of 131 | | | | |
| engine works | 162 | 0.19 | 44 | 0.39 | 0.013 | 0.70, 0.72 | 0.08 |
| crucible steel | 27.8 | 0.31 | 28 | 0.19 | 0.027 | 0.27, 0.31 | 0.14 |
| rolling mill | 29.3 | 0.43 | 10 | 0.30 | 0.017 | 0.22, 0.23 | 0.18 |
| smithy | 12.1 | 0.36 | 4 | 0.40 | 0.011 | 6.6, 6.7 | 0.16 |
| **1870s**: iron at 5.6, the furnace's margin 0.39, its return 0.09 | | | of 185 | | | | |
| engine works | 60.9 | 0.19 | 88 | 0.43 | 0.011 | 0.37, 0.38 | 0.07 |
| smithy | 22.0 | 0.17 | 71 | 0.34 | 0.009 | 4.2, 4.3 | 0.06 |
| converter | 10.8 | 0.45 | 63 (of 163) | 0.32 | 0.022 | 0.28, 0.30 | 0.17 |
| rolling mill | 9.7 | 0.60 | 25 | 0.39 | 0.017 | 0.19, 0.20 | 0.22 |
| crucible steel | 11.9 | 0.43 | 3 | 0.11 | 0.026 | 0.20, 0.25 | 0.16 |
| steel rail mill (on steel) | 9.5 | 0.44 | 14 (of 146) | 0.25 | 0.013 | 0.33, 0.35 | 0.07 |

In the 1870s the higher hurdle (0.15 for 0.12) refuses the tie in 5 town-years of the converter's 63, 5 of the rolling mill's 25 and 6 of the engine works' 88, and the tie alone clears the hurdle in one town-year each for the smithy and crucible steel.

With the prototype on, the year's profit over the value of the plant (pooled), for works that no family owns and that are not building:

| | works-years a run | return | utilisation | at a loss |
|---|---|---|---|---|
| tied concern, a year the last good's price fell a tenth or more | 939 | 0.116 | 0.63 | 0.15 |
| tied concern, a year the middle good's fell (and the last good's did not) | 644 | 0.100 | 0.73 | 0.17 |
| tied concern, other years | 2,501 | 0.156 | 0.69 | 0.13 |
| free user of an intermediate good, its product fell | 3,346 | 0.178 | 0.60 | 0.36 |
| free user, its material fell | 2,578 | 0.179 | 0.73 | 0.26 |
| free user, other years | 9,440 | 0.171 | 0.69 | 0.30 |
| free maker of one, its product fell | 703 | 0.087 | 0.78 | 0.21 |
| free maker, other years | 2,163 | 0.129 | 0.79 | 0.21 |

A tied concern's plant is 61 per cent the maker's by value. A mix of free works in that proportion earns 0.145 in an ordinary year, and the concern 0.156: 7 per cent more. In a year the last good falls the mix earns 0.148 and the concern 0.116; in a year the middle good falls, 0.123 and 0.100. So the concern earns about a fifth less than free works when a price falls, and is at a loss half as often as a free user in every kind of year. I did not hold the trades equal (the tied users are engine works and converters more than the free ones are), so the fifth is a pointer and no more.

How ties end, a run, by the making stage: with a coke furnace, 20 by the user closing, 8 by both closing in one year, 3 by the furnace closing; with a charcoal furnace, 6 by the user, 5 by the furnace, 3 by both (these are the ties that "die with charcoal iron": in three of fifteen the user goes down with its furnace); with a smithy, 7, 2 and 4; with a converter, 8 by the user and none otherwise. At the review a tied maker orders 5.8 per cent more capacity a year and a tied user 5.2, a free maker 6.8 and a free user 6.5. Tied users had their materials short at a year's end in 8.5 per cent of years and free users in 10.6: the supply a tie assures is a small thing, because the tied row still buys its ore and coal in the market.

### The design, as an economist

**Would it take every chain, or none?** Neither, and not for a reason of economics. The table of sums says a tie adds 1 to 3 per cent of the last good's price to a margin of 11 to 43 per cent, which is 0.01 to 0.07 on a return of 0.2 to 6. Against a hurdle a quarter higher that is too little for a stage near the hurdle (a tenth to a fifth of the converter's and the rolling mill's chances are refused) and always enough for a stage well clear of it, provided the maker's cell is as good a site as any other. A works that would be founded is founded tied if a free maker with room happens to stand where it would have gone, and free otherwise. What sets the share of a chain made in ties is therefore where the walk to work and the kept town ground have already put the works, and the ration (one tie a works, no family's works, a maker running at 0.65, the cell's room, the year's room of the user's trade). Loosening `hurdle_mult` and `merge_fit` moved 58 ties to 77 and 6 per cent of the iron to 8 for that reason. There is no price at which integration spreads or retreats, so it cannot "rise and in places fall" as the design hopes: nothing in it answers to the price of iron, the cost of carriage or the merchant's terms.

**Is "less flexible" in the rule, or only said?** It is in the rule, and missing from every book.

- *It takes its own iron whatever the market's costs.* True (`tie_rows` strikes the good from the user's recipe). But the tied row then carries the maker's whole margin: 0.06 to 0.22 of the last good's price, five to ten times what the tie saves. A coke furnace earns 35 to 49 per cent on the price of iron. So the tied user runs on a margin far fatter than a free user's and slows less when its own good falls (utilisation from 0.69 to 0.63 in such a year, where a free user's goes from 0.69 to 0.60): it goes on turning iron into rails that do not pay, because iron and rails together still do. That is a real rigidity, and it reads in the concern's own books as strength.
- *A slump in the last good idles the whole.* Only through the merchants' brake (`take` on the user's good now brakes the tied part of the maker), and only for the three tenths of the maker that is tied. The rest sells as before.
- *One purse, one judgment, more capital at risk.* The review gives both stages the joint return (`judged`), so a using stage that loses money on its own never counts a losing year while its furnace earns. Tied users close at 1.7 per cent a year where free ones close at 3.4. The life cycle's weeding of the small fringe (`works.md`) is switched off for a tied stage. That is the opposite of risk.
- *It carries older technology for longer.* Nothing re-equips any works in this engine, tied or free, so a tie adds nothing here except through the first point: a tied smithy cannot buy the coke furnaces' cheap iron. In three ties of fifteen the user closed in the same year as its charcoal furnace.

What it gives up is forgone profit: iron it could have sold dearer, rails it should not have rolled. That shows in the table above as a return a fifth lower in years a price falls, if the table can be trusted that far. No book in the engine or the record shows it, so neither the review nor a player could act on it. The design's shadow books are the missing piece, and they were the part put off.

**What it saves is a transfer.** The spread is paid to the merchants, whose income goes to households, as a works' dividends do. No hands and no goods are saved (carting is 0.1 to 0.2 per cent; the heat saved makes no difference to any fact). Sixteen seeds confirm it: the island is the same island. This is not an argument against building it. A family that owns a furnace and a forge would gain 6 per cent of the value of the iron between them, and that is a decision worth offering the player; and it gives the viewer a true thing to draw. But nobody should expect combined works to change growth, wages or the carters' share, and the sweep's facts for it should be about the concerns themselves.

**The heat is given to the wrong pairs.** `hot_goods` is by good, so a smithy and an engine works beside a furnace save three tenths of their heat on iron that "goes on hot". They forge and machine cold metal. Fourteen of the 26 ties with a coke furnace in 1850 to 1900 are of that kind.

### The design, as an engineer

**Can money leak or be made?** No, and the reason is worth knowing: the prototype has no price inside the firm at all. The tied row is an ordinary row of `econ::step_month` with a rewritten recipe; every payment it makes or receives is at a market price, to or from a purse `money_total` counts; `one_purse` moves cash between two such purses. Passed goods are added to the maker's output, its method's learning and the good's cumulative output without ever entering `State::inv`, and the materials for them are bought in the right proportion, so no goods are made from nothing either. The price of that safety is the list below: with no transfer price, neither stage's own books mean anything.

**What assumes one works is one method, buying and selling in its market:**

| where | what it assumes | what a tie does to it |
|---|---|---|
| `works.rs:371` (`works::new_year`) | a works' return is its own profit over its own plant; the year's room is served in that order | the tied user books the whole concern's margin on its own plant and is served first in its trade; the tied maker last |
| `works.rs:297` to `304` (`town_trade`, `year_budget`), `econ.rs:968` | a town's trade in a good is what its merchants sold and shipped | passed goods are in no market, so the room for the maker's trade shrinks as more is passed; a maker that is wholly tied has room for one smallest works a year |
| `prices.rs:167` to `189` (`take_at`, `merchants_take`) | the same `outflow` sets when the merchants stop taking | the residual market in a mostly passed good is thin, and brakes sooner |
| `firms.rs:394`, `405`, `410` (the review) | return, losing years and insolvency by works | return and utilisation are joint (`judged`); insolvency is not: `one_purse` leaves the maker with no monthly cost (`combines.rs:219`), `new_year` then shares the purse by plant (`combines.rs:280`), and the two stages face different tests at line 410, so one can be closed as insolvent while the other stands |
| `firms.rs:356` to `371` (entry) | a proposal that is funded is the works that was reckoned | a works reckoned with a tie is built even if its partner was taken by an earlier proposal that year: 8 of the 133 a run are founded for a tie they never get, sized to a partner and not to their market |
| `firms.rs:330` to `333` | the best free site is proposed when it clears the hurdle | where a tied site is proposed the free one is dropped; if the tied one is then refused (the cell, the pool) the trade founds nothing in that town that year |
| `land.rs:84` (`delta_fields`) | every cell with a works making a good is a source of it at the gate price | a wholly tied maker's cell is still a source for its neighbours |
| `families.rs:319` (`works_value`), `600` (a branch buys a free works) | a works is bought and valued alone | a family can buy one stage of a tie; the tie is dissolved at the next review (`combines.rs:275`) and nothing is paid for it |
| `families.rs:898` to `902` (`market_depth`) | what a town makes against what its buyers ask for | passed goods are in `town_prod_year` and not in `requests`: the player's site sums see a town better supplied than its market is |
| `chronicle.rs:286`, `940` to `978`, `sim.rs:477` | a closure is one works of one trade, notable by its own size | a concern's end is two stories, each judged small |
| `facts.rs:355` to `371` | size and count by works | no fact sees a concern; the median size fell because stages were added |
| `record.rs:487`, `live.rs:327` | one row a works, one method a row | holds, by design; but the tie is written as it stood at the end only, so a viewer cannot tell when a tie that ended began or ended |
| `accounts.rs` on `night/accounts` | a works' 21 columns add up; a town's goods balance (made, bought, sent, stock) closes | the maker has output with no revenue for it and the user carries the maker's materials and wages; whether the balance closes depends on whether "made" is read from the rows or from `month_stats`, which now differ by what was passed |

None of these breaks a run. Several bend the result: the first two decide how fast tied works grow, and the fourth and fifth are plain faults of the prototype.

**What is underspecified**, to the point that two builders would build different things:

1. *"At cost."* There is no cost in the prototype. For the record and the books a price is needed: the maker's materials, fuel and wages a unit that month, with or without its rent and a charge for its plant?
2. *The maker's surplus and the user's shortfall.* May the maker sell what its user does not take (the prototype: yes, seven tenths of it)? May the user buy what its maker cannot make (the prototype: no, it idles; the design does not say)? "Tied in proportion" means neither, and then most of the prototype's ties are not allowed.
3. *Founded whole*: on whose candidate cells, at what sizes, against which year's room (the user's, the maker's or both), and what "the savers can find the money for both at once" does to the order proposals are served in.
4. *The shadow books*: at the quantities the concern ran, or at those each stage would have chosen alone? The second is the honest one and needs the utilisation rule run twice.
5. *Closing one stage*: "its shadow return is over half the hurdle" and "beat the joint return by a quarter three years running" are numbers with no measurement behind them.
6. *Families*: what a tie costs to make (nothing?), whom buying the neighbour pays (a free works' backers are the town's savers), whether a rival's rule is the player's rule.
7. *Chains of three.* A works is in one tie. A furnace, a converter and a rail mill, which is the works the design says it should resemble, cannot be one concern.
8. *The hot pairs*: by good or by pair of methods.
9. *The yards between the stages*, once works hold stocks: one yard or two, and whether the saving of the merchants' stock is then real.

### Is the record's row enough, and affordable?

Affordable: 12 bytes a works-year is 720 KB on a record of 21.3 MB, 3.4 per cent, and 22 KB a year live. Enough to lay a works out by its plant: capacity and what was ordered each year give the ranges and when they came, the method gives the kit, hands and utilisation give how busy it is, state and `died` give dereliction, the cell's track and water give the siding and the wharf. Not enough to lay it out by what it holds, because it holds nothing, and the yards as specified would be still nine years in ten (section 1). Changes to the row: hands in tenths; rows from the year of founding; for a tie, the partner and the years it began and ended in the works' table (a works may be tied more than once, so a short list), and per year the share passed and the concern's gain or loss against its shadow books, a signed byte in hundredths of the plant's value. That last is the one new number a viewer or a ledger could show as what being combined is worth to this works.

One thing the viewer may honestly draw today, with no change to the engine: on a cell where a maker and a user stand together the user already pays the maker's gate price and nothing for carting (`land::delta_fields`). The lesser of what the cell makes and what it uses of the good (the "could cross a yard" column above, to be had from capacity, utilisation and method) passes over the wall in the engine's own sums. A gate in the wall between them is true now.

### Verdict: build it changed

In this order.

1. **The record's row, now.** Capacity, share on order, hands, utilisation and state are all true today. With the two changes above. This is what the viewer's helper is waiting for and it needs no decision about anything else.
2. **Not the yards as specified.** Either leave stock where it is (the merchants', drawn in the town) or build yards with the three repairs in section 1 and a sweep that watches the tools famines.
3. **Combined works, but as a concern with books, not as a pair with a shelter.** Keep from the prototype: each stage stays a works of its own method with its own row; the tied part runs as one row; passed goods are booked to the maker; the switch, off under both sets of rules. Change:
   - *Shadow books first.* Every month, for each stage, what it would have earned alone at the quantities it ran (the maker selling what it passed at its gate price less the spread, the user buying it at the delivered price). The concern's gain is the joint profit less the two. It goes in the record. The review counts a stage's losing years on its shadow return, so a stage that lives only on its neighbour's margin closes as a free one would, and it cuts the tie when the gain has been negative three years running. This is what makes "less flexible" a number, and it removes the shelter.
   - *In proportion.* `step` 1.0, so a user cannot outgrow its maker and stand a fifth idle; and a tie is made only where the maker makes between `merge_fit` and its inverse of what the user needs (0.5, as `merge` has it). That drops the small stage hung on a large maker, which is seven tenths of what the prototype tied and is a customer on the same cell, already served by the gate price.
   - *How one comes to be*: neighbours in proportion brought under one owner (the prototype's `merge`, switched on), a user building its own making stage sized to itself (the prototype's backward case, which is where its whole integrations came from), and then founding whole. A family's works included: the player's two works on a cell can be tied and untied by the player, and a rival's by the rule, where the year's gain by the shadow books is over a tenth of the smaller works' profit, as the design has it.
   - *Heat by pair*: furnace to converter, furnace to rolling mill, converter or crucible to steel rail mill. Nothing for a smithy, an engine works, a millwright or crucible steel.
   - *The year's room and the merchants' brake count what is passed* as part of the town's trade in the good (add it to what `works::town_trade` reads), or a tied maker cannot grow.
   - *A concern is a list of stages*, not a pair of pointers, so that furnace, converter and rail mill can be one. It can be built for two without ruling out three.
   - *The two faults*: a proposal whose partner is gone is dropped, not built; and the purse is one purse for the insolvency test.
4. **Then, and only if the picture wants it, ties across neighbouring cells** with a tramway's cost: a third to a half of the iron could cross there, against a tenth to a sixth on one cell.

What to expect of it, so that nobody is disappointed: 35 pairs in proportion and free today, about 41 if families' works are counted, carrying a tenth of the iron or so on one cell. The island's wage, output and carting will not move. The facts to watch are the concerns' own: how many, their hands, the share of each good passed, the gain by the shadow books and how it falls when a price falls, stages closed and ties cut.

**Sidings and wharves**: agreed as written. Nothing in the engine now; the viewer draws a siding where the works' own cell has track that year (the value 4 in `trans`) and a wharf where it is navigable.

### What I am unsure of

- **The fifth.** That a concern earns a fifth less than free works when a price falls rests on a mix by value of plant across trades I did not hold equal, and on years picked by one year's price against the year before. The direction fits the rule; the size may not survive a like-for-like count.
- **Whether shadow books at the quantities run are enough.** A stage's shadow loss at the concern's quantities overstates what it would have lost alone, since alone it would have slowed. If stages are closed on that, some would be closed that could have lived. The honest books run the utilisation rule a second time for each stage; I have not costed that.
- **The sums use the town's largest maker's cell** and a new works of the size entry would give it before any of the year's room is taken. A real proposal is reckoned on sampled cells after the review has taken room, so the town-years where "a free works clears the hurdle" are more than the works founded.
- **Seeds 9 to 16 were swept, not probed.** The tables of ties, pairs and returns are seeds 1 to 8.
- **The merged tree.** With thirty-odd goods (`night/tree`) the pairs multiply, and the tree's design leaves out pig iron, yarn and coke as goods because each would be a thin market. A concern is the natural home for a good with no market, and that may be a better reason to build concerns than the merchant's spread is. I did not load the tree.
- **I changed no engine code.** The two faults of the prototype are described, not mended, so that its tables can still be had again to the byte.
