# The production tree, checked: what is wrong with `content-design.md` before anyone builds it

Night of 11 November 2026. Read-only: nothing in the repository was changed and cargo was not run. The scripts are in `.claude/night/tmp/critique/` (Python 3.13, `tomllib`, no numpy); their tables are at the end of this file, unedited.

## What was checked, and how far to trust it

| check | script | what it rests on |
|---|---|---|
| Unit sums at base prices, every method, at K0 and 0.4 K0 | `sums.py` | the engine's own formulas (`firm_coefficients`, `heat_mix`, `site_roi`, the merchant's spread that entry now counts). Exact. |
| Demand at base prices, a town of 20,000 at 3 a head | `demand.py` | `service_mix`, `les`, `engel::les`, landlords beside households, running inputs of the 1750 methods. Exact for those prices. |
| The new goods at the **game's** prices, 1750 to 1899 | `reckon.py`, `tweaks.py`, `atfound.py` | A reckoning, not a run. Population, wage, the sixteen base prices and outputs, households' real income and the landlords' share come year by year from the designer's own sweep under the game's rules (`tmp/synth2/runs/F_fit`, eight seeds). New goods are priced at the cost of the methods making them; shares move a tenth of the trade a year toward a logit on cost (width 0.2, standing for differences between towns); demand is the engine's. Plant, exports and the land's own purchases are left out. Fish and produce are priced at the design's tuning aims; raw cotton at twice the `[world]` path (a guess). |
| Discovery years | `reckon.py` | the engine's hazard, as a median with a 10 to 90 per cent range. **It reproduces the designer's sixteen-seed runs**: edge tools 1774 (run 1783), tool factory 1863 (1857), hot blast 1829 (1831), machine tools 1837 (1839), engineering works 1880 (1880), kiln 1855 (1856), converter 1853 (1858), steel rails 1869 (1871), farm machinery 1843 (1843), railway 1826 (1826). So what it says about thresholds on new goods deserves the same trust as the output figures behind them. |
| Seed-to-seed spread of "xN" thresholds | `seeds.py` | the sixteen runs of `F_fit` and `F_hold`, seed by seed. Measured. |
| Chains and deadlocks | `chains.py` | the overlay's own tables. Exact. |
| Engine demands, cost | reading the code on `rust-3d`, `night/land`, `night/canals` | not measured: no build was made. |

The designer's reckoning was done at the **port** baseline's prices (wage 4.6 rising to 8.1, coal 1.5 to 1.8, stone 1.2 to 2.2, bricks 3.5 to 6). Under the game's rules the island is a different place: the wage goes from 4.0 to 14.7 while coal falls to 0.9, stone to 0.9, bricks to 1.5 and machinery to 9. The design names this as its first risk. Most of what follows is that risk worked through with numbers: **everything in the tree that is mostly labour comes out dearer than reckoned, and everything that competes with a cheap dug or fired good comes out weaker.**

What holds up: the arithmetic of every unit sum in the overlay's comments is right; there is no deadlock and no good without a maker or a buyer (raw cotton and ships aside, as the design says); the waves do arrive in order; the thresholds on base goods are sound at the median. The problems are below, worst first.

## 1. Gas is a 3 per cent trade, and the three methods that wait on it come late or never

**What happens.** The design gives gas 3.5 as fuel and light and makes it from 1.2 of coal. That coal, burnt in the grate, is 2.9 of warmth, so before a hand is paid or a retort built gas costs 86 per cent of what coal costs per unit of warmth, and it can never be the cheaper. At the port's prices (coal 1.5, wage 6.5) the reckoning gave gas 19 per cent of fuel and light in 1850 and 37 in 1890. At the game's (coal 1.1 falling to 0.86, wage 9.7 rising to 14) gas costs 1.9 times coal per unit of warmth and the logit gives it **3 per cent in 1850, 2 in 1890, 4 in 1899**: 3,500 units a month in 1850 where the design has 18,700, and 6 million made by 1899.

**What it breaks.**
- `ammonia_soda` waits for 9,000,000 of gas and `tank_furnace` for 10,000,000. Progress in 1899 is 0.65 and 0.58: **neither is ever found.** The last wave loses its chemicals and its glass.
- `emporium`, the one method that makes services with fewer hands, waits for 3,000,000: met in 1877, found about 1872 (1857 to 1887), twelve years after its aim. And it burns gas as a running input, so it can only stand in a town with a gasworks.
- The first gasworks (0.4 K0 = 200) is 73 per cent of the whole island's gas market in the year it is found. Gas cannot be carted (weight 20, half lost a month), so only two or three towns can ever hold one.

**Fix.** Rate gas at **4.5** (light is the point, and the design's own comment says so): 23 per cent of fuel and light in 1830, 20 in 1850, 21 in 1890, 28 in 1899; 31 million made by 1899; shops found 1864, ammonia-soda 1873, tank furnace 1879. The share is steep in this number (3.5 gives 3 per cent, 5.0 gives 33, 6.0 gives 54), so it wants a sweep of its own. With it: gasworks `K0` 500 to **150**; and take gas out of the two thresholds whatever the rating, since a clock should not hang on the thinnest good in the tree: `ammonia_soda` = { chemicals 2,500,000, steel 5,000,000 } (found 1869), `tank_furnace` = { glass 700,000, steel 5,800,000 } (found 1873).

## 2. Builders lose shelter back to heaped stone and kiln brick for a generation

**What happens.** The design's own rule for comfort is "nothing made shares a need with services: hands never get cheaper". Building work is the same thing in another need: half its cost is wages, and it shares shelter with stone and brick, which the game's machines and the overlay's own continuous kiln make cheaper every decade. At the game's prices:

| | 1770 | 1810 | 1850 | 1870 | 1890 | 1899 |
|---|---|---|---|---|---|---|
| builders' share of shelter, as drafted | 72% | 68% | 47% | **23%** | 64% | 74% |
| the same, with stone's price following what is dug | 71% | 66% | 43% | **21%** | 53% | 65% |
| the design's reckoning | 88% | | 84% | | 100% | |
| stone dug, as a share of the run without builders | 71% | 71% | 70% | 75% | 51% | 42% |
| the same, with stone's price following | 82% | 82% | 81% | 84% | 72% | 65% |
| builders as a share of all hands | 5.2% | 6.3% | 5.6% | 3.7% | 6.2% | 7.0% |

(Stone for plant and railways is taken as 15 per cent of the run's in every row.) In 1870 a unit of shelter costs 2.36 as heaped stone, 2.69 as raw brick and 2.75 from a builder (a mason's work alone 2.63). The masons' trade halves between 1850 and 1870 while the island grows by a third, and it is saved only by the contract builders, whose threshold (35,000,000 of building) is met in 1883 to 1886 at these volumes: found about 1866 to 1870 at the median, but not before 1887 in one seed in ten. Quarrying is cut by about three tenths until then, not "by more than half at once". And the fall is self-limiting: the rock field is fixed, so the less stone is dug the cheaper it gets (a supply elasticity of 3), and cheap stone wins shelter back. Stella's fourth point is half met.

**Fix.** Re-rate raw shelter **in the overlay** (the port's file is untouched, and the design's "no base good is re-rated" was a rule about steel): stone 0.5 to **0.3**, bricks 1.0 to **0.6**, timber 0.6 to **0.4**, sawn timber 0.9 to **0.6**, lime 0.8 to **0.5**. A household can still build for itself where builders are short, at a real loss. Then:

| | 1770 | 1810 | 1850 | 1870 | 1890 | 1899 |
|---|---|---|---|---|---|---|
| builders' share of shelter | 99% | 99% | 95% | 95% | 100% | 100% |
| with stone's price following | 98% | 98% | 88% | 83% | 98% | 99% |
| stone dug, against the run's | 52% | 51% | 39% | 32% | 24% | 25% |
| with stone's price following | 61% | 61% | 56% | 55% | 34% | 32% |
| builders as a share of all hands | 7.7% | 10.1% | 11.9% | 11.1% | 9.5% | 9.6% |

That is the design's own claim (a twelfth of hands, a tenth by 1850, stone down by half and then by two thirds) and the contract builders are found in 1859 on the threshold as written. Without the re-rating, cut their threshold to building 12,000,000 (found 1859).

A smaller thing in the same place: with building in, households stop buying raw brick. At base prices a town of 20,000 buys 40 bricks a month where it bought 783 (down 95 per cent). The brick trade from 1750 to 1850 is then mostly other works' plant (a pottery manufactory's 1,920 bricks is a month of the whole island's output), so expect brick prices to jump at every new works. The bricklayers at -15 per cent "on purpose" make this worse; bricks 0.7 to 0.6 a unit in their recipe would keep a brick trade alive on the clay.

## 3. Thresholds on new goods: what is late and what never comes

Median year found, at the game's prices, full content (stage 5). The base-goods rows are in the table at the end and agree with the runs.

| invention | aim | threshold met | found (10%..90%) | first works | why |
|---|---|---|---|---|---|
| ammonia-soda | 1875 | never | never | | gas 9M (problem 1) |
| tank furnace | 1878 | never | never | | gas 10M (problem 1) |
| roller mill | 1878 | never | 1889 (1867..never) | 1893 | flour 200M: flour is 15 to 30 per cent of food here, not over half (problem 4) |
| contract builders | 1862 | 1883 | 1866 (1849..1887) | 1871 | building 35M reckoned on twice the volume |
| shops, offices | 1860 | 1877 | 1872 (1857..1887) | (run) | gas 3M |
| steam cotton mill | 1805 | 1817 | 1818 (1808..1829) | 1829 | cottons 4M; cotton cloth starts slower |
| water-frame mill | 1771 | 1780 | 1782 (1773..1792) | 1782 | cottons 250,000 |
| pottery manufactory | 1770 | 1777 | 1779 (1769..1789) | 1783 | pottery 80,000 once four goods share the need (1771 at stage 1) |
| sheet glass | 1835 | 1843 | 1842 (1828..1854) | 1848 | coal x620 |
| pulp mill | 1885 | 1878 | 1880 | not built | by design until logs are driven; its entry price is 1.22 of the going price |
| steam sawmill | 1825 | 1821 | 1823 | not built from stage 3 | the water sawmill undersells it for ever (problem 8) |

On time (within five years): cement 1849, common brewery 1790, gasworks 1812, alkali 1817, machine paper 1818, water-powered woollen mill 1775, deep mining 1813, tile drainage 1840, manures 1854, haulage 1861, trawling 1877, ocean steamers 1864.

So the four waves do arrive in order, and the first three are full. **The last is the thin one**: of the "eleven methods and three improvements after 1855", two are never found, three are found and not built (pulp mill, steel rail mill, and the steam sawmill once water sawmills exist), the roller mill comes in 1893 if at all, and the shops and the contract builders are ten to twenty years late in the slower seeds. What the last wave really has is the converter, the kiln, the tool factory, the engineering works, haulage and trawlers.

**Fix**, beyond problems 1 and 2: the roller mill's 200,000,000 of flour is right once fish and produce are re-rated (problem 4: found 1877); as drafted it would have to be 100,000,000. `steam_cotton_mill` cottons 4,000,000 to **1,500,000**; `water_frame_mill` cottons 250,000 to **120,000**; `pottery_works` pottery 80,000 to **40,000** from stage 2. With every number proposed in this file the reckoning finds all of them within nine years of their aim (table at the end); run the fitter the design describes all the same, since these are starting points. One stays marginal: the ammonia-soda works is found but its smallest works needs 4.18 against a going price of 4.38 in 1875, and once the alkali works are smaller and more numerous it is not built before 1900. Its 0.4 of lime is a third of its cost; lime **0.25** makes it safe.

**"xN" on coal is a noisy clock.** Twelve inventions take a threshold in coal as a multiple of 1750's output. Across the sixteen seeds of the designer's own runs, the year a late coal threshold is met spreads by a generation before the hazard adds its ten years: gasworks and alkali (x350) 1804 to 1829, hot blast (x500) 1814 to 1842, sheet glass (x620) 1821 to 1850, haulage (x1400) 1850 to 1882, trawling (x1980) 1864 to 1895. Machine paper's cloth x110 is worst, 1799 to 1832, because cloth takes off only when the powered mill is found. The same thresholds as plain numbers are tighter by about ten years each (gasworks 1806 to 1821, hot blast 1817 to 1833), and thresholds in iron are tighter still (11 to 13 years). **Fix**: write late thresholds in plain units, put a second threshold in iron, steel or machinery beside every late coal one, and drop cloth from machine paper (paper 300,000 and machinery 30,000 are enough).

## 4. Stage 3 halves the grain trade, and its own numbers disagree

The overlay's tuning aims say fish should sell at 1.3 times grain and produce at 1.7. At those prices a unit of food costs 1.00 as fish, 0.94 as produce, 0.87 as flour and 1.00 as grain, and the logit splits food evenly: flour 30 per cent, produce 32, fish 19, grain 19. The design's own table (flour 48, grain 30, produce 13, fish 10) needs fish at about 1.55 times grain and produce at about 2.05.

At the written aims: grain bought falls by 48 per cent at base prices (a town of 20,000: 29,900 to 15,400 a month) and by 54 per cent at the game's (83,600 against 183,000 in 1750), where the design says "about a seventh" for flour and shows 30 per cent. Fish and produce reach 300,000 and 360,000 units a month by 1899; produce at weight 4 would then be a larger carting load than grain. And flour never reaches the roller mill's threshold.

It is an opening state, not a crash in mid-game, but it re-bases the farm sector the Engel, harvest and people pages were tuned on (the harvest's dearth line is a grain price; `yearly` counts sectors 0 to 2 as agriculture, so gardens and fisheries drop out of the farm share).

**Fix.** The efficiencies: fish 1.3 to **1.1**, produce 1.8 to **1.4**. With the tuning aims as written that gives the design's own table (1770: flour 44 per cent, grain 28, produce 15, fish 13), grain down by a third instead of a half, and the roller mill found in 1877 on its threshold as written. Say in the overlay that the aim is a share of food (about an eighth each), since that is what the sector `A` should be tuned to, not a price ratio.

Flour itself: 1.15 of grain becomes 1.5 of food for 0.005 of a hand, a third more food from the same harvest for nothing, from 1750. It is in the tree for the mills, and the mills are worth it, but it is a free lunch the port's calibration never had. At 1.3 flour is three tenths of food and falling, which is too little to carry a roller mill; so it probably has to stay, and the 1750 level of food should be re-checked against the people rule when stage 3 loads.

## 5. Works sized for a market that is not there yet

At the game's 1750 prices a household's budget is 2.27 a head, not the 3 at base prices the sizes were set against, and the wares are half the reckoning: glass 67 units a month on the whole island in 1750 (the design: 155 in 1770), pottery 174 (401), paper 146 (263), soap 175 (363). The 1750 crafts are sized small enough for that (4 to 7 smallest works a town). The first inventions are not:

| invention | found | island market that year | its smallest works (0.4 K0) | as a share |
|---|---|---|---|---|
| gasworks | 1812 | 274 | 200 | 73% |
| pottery manufactory (stage 2 on) | 1779 | 321 | 160 | 50% |
| machine paper mill | 1818 | 727 | 160 | 22% |
| sheet glass works | 1842 | 770 | 120 | 16% |
| cement works | 1849 | 2,766 | 400 | 14% |
| alkali works | 1817 | 913 | 120 | 13% |

Entry promises a return at 85 per cent of capacity. A works that is half the island's market sells into a glut, the merchants stop taking at eight months' stock, and it is closed after three losing years: the port's "works dying young", now in the trades the design adds. Pottery, glass and paper are light and dear and will trade their way out in part; gas cannot. **Fix**: `pottery_works` K0 400 to **150**, `gasworks` 500 to **150**, `machine_paper` 400 to **150**, `sheet_glass` 300 to **150**, `alkali_works` 300 to **150**, `cement_works` 1000 to **400**. They grow where they stand under the life cycle, so nothing is lost.

## 6. Stage 1 alone gives every spare shilling of "household goods" to potters

Until stage 2 the wares need holds pottery and wooden ware, but keeps its shares (0.03, and 0.10 at high incomes). With one real good in it the need is a fixed slice of spending: pottery reaches 210,000 units a month and 12,500 potters by 1899 (3.7 per cent of all hands; the design's figure with all four goods is 22,400 units). When stage 2 loads the pottery trade loses five sixths of its market (a town of 20,000: 628 a month to 104), which breaks every save and every tuned number from stage 1.

**Fix.** Ship the three stage-2 goods with stage 1: `theta` for each need is a few lines in `service_mix`, and the seed plan is not needed (problem 10). If they must wait, give stage 1 its own shares: wares 0.01 and 0.03, the rest to comfort.

"Drink and groceries" has the same shape for good: it is beer, with home brewing (grain at 0.15) as the only ceiling on its price, about twice beer's cost. Beer reaches 450,000 units a month and a fifth of the grain crop by 1899. That is acceptable as long as it is known: the need is a fixed 6 to 7 per cent of spending above subsistence whatever beer costs.

## 7. Cotton ends wool, not "takes three quarters of it"

With raw cotton at twice the `[world]` path, cotton cloth is 47 per cent of clothing in 1810, 67 in 1850, 93 in 1870 and all of it by 1890; woollen cloth falls from 170,000 units a month (the run) to 5,000 and no wool is worn. The design's risk 8 has it stopping at 77 per cent. The difference is the game's wool price, which rises to 10 as farms take the pastures. Wool would cheapen as it is abandoned, so the truth is between, but at a logit width of 0.25 a good 2.4 times dearer per unit of clothing keeps under 1 per cent.

**Fix.** Clothing `theta` **0.5** from stage 5 (the design's own fallback was 0.4), and cotton cloth 0.85 to **0.75**. Even so the reckoning has cotton at 90 per cent by 1890 and woollen cloth at 5 to 10: wool is saved only if its price falls as the pastures empty, which a run will show and a reckoning cannot. Sweep it with the world's prices, which nobody has run.

## 8. The water sawmill is the best business of 1750, and the steam sawmill is never built

Recipe margins of the 1750 methods run from 9 to 26 per cent; the water sawmill's is 44. At the usual size and base prices it is the only 1750 method in the whole content, old or new, that clears the hurdle (a return of 0.61 a year; the saw pit's is -2.16). From stage 3 it has 90 per cent of sawn timber in 1750, the saw pit is a remnant from the first day, and the steam sawmill of 1825 needs 1.21 times the going price to enter, so it is found and never built: the port's converter again, in the trade Stella asked for by name. The stream's capacity (eight usual sizes a cell) does not bind: the island wants under 9,000 units a month until 1870, which six good cells hold.

**Fix.** Water sawmill: timber 1.3 to **1.35**, labour 0.08 to **0.16** (a margin of 33 per cent, and still half the saw pit's hands). Steam sawmill: `A0` 0.7 to **0.85**. In the reckoning the saw pit then keeps a quarter of the trade to 1820 and the steam sawmill is built in 1828 and has nine tenths by 1850. And tie the mill's capacity to discharge more tightly than `8 x field`, so that streams near the woods fill by about 1820.

## 9. Mills on a knife edge

A corn mill's costs are 97 per cent grain (1.27 of 1.30 at the usual size), its plant is half a month's output, and its labour is 0.005. The engine runs a works by its margin as a share of price: at +7 per cent it works at three quarters, at -1 per cent at a quarter. Under the game's harvests grain moves 8 to 12 per cent from year to year. A dear year takes every mill on the island from profit to loss at once, the mills stop, and flour, which is a third to two thirds of food, is short in exactly the year bread is. The brewhouse (grain is 60 per cent of its price) has half the problem. Nothing in the present content has a volatile land good as nine tenths of its cost.

This cannot be reckoned, only run. **Watch for it in the first run with stage 3** (the mills' `f_u` in a dearth year). If it shows: mill labour 0.005 to 0.03 and plant to 3 months of output, which makes the miller a visible trade (2,000 hands, not 250) and thickens the margin.

## 10. What stage 1 quietly needs from the engine

The design says "the loader only". Reading the code, six more things:

1. **`stage` has to be a set, not a ceiling.** The file says "keep the entries with stage <= N", and section 10 says to build 1, 2, 3, 5, 4, 6. At "stage 5" the stage-4 improvements load without their rule: liming and fishing smacks are `known = true` and the others are found and print headlines for things that do nothing. Give the loader a list of stages.
2. **`group` belongs in stage 1.** `Group::of_method` returns none for a name it does not know, so no family, the player's included, can own a builder's yard, brewery, pottery or gasworks until `group` is read, and families found a large part of the island's works (half of those standing in 1900 under the port). The design defers it to stage 2 as "families cannot run them". It is a string-to-enum map.
3. **`util::pairwise_sum_by` allocates on the heap above sixteen terms** (`if n <= 16 { stack buffer } else { Vec }`). Every sum over goods in the monthly step (twelve call sites in `econ.rs`, five in `firms.rs`) takes that branch from the seventeenth good. Raise the buffer to 64 before timing anything.
4. **The opening stock doubles as the opening guess of trade** (`sim.rs:145-151`), and stage 1 has no seed plan. For lime that is 1,200 units in a town whose few first masons use 50 to 100 a month, so lime opens in glut, its price sags for a year or two of the warm-up, the first kilns come late, and meanwhile a mason's output is cut to the lime there is (`phi` is the least fill of any input). Ten years of warm-up should absorb it; look at 1750's lime price and the builders' fill in the first run, and if it is not settled give goods two numbers (a stock and a trade guess). The builders themselves do not need a seed plan: at one works a year in each of three trades, then 15 per cent of the trade a year, a town's builders reach its demand in about eight of the ten warm-up years by my arithmetic.
5. **Canals have their own clock already.** On `night/canals` the mechanism finds canals from `discover_coal` and `discover_stone` (3,000,000 each) in its own parameters, by the same hazard. The overlay's `canal` invention (coal x18, about 2.6 million; stone x13, about 5.0 million) would be a second clock that disagrees. Keep one. And note that its stone threshold runs on stone dug, which the building trades cut by a third to a half: five to ten years later than tuned.
6. **Perishables waste a fixed share of output.** With `price_target_months` at 2 and a decay of d, the price rule settles where production exceeds sales: by 17 per cent for building work (0.3), 21 for gas (0.5, as services today), 11 for produce, 9 for fish, 5 for beer. One builder in seven works on houses nobody gets. Building's decay at **0.1** halves that and still stops stockpiling.

Smaller: building work at weight 12 and gas at 20 are carted in the engine all the same (from the yard to the market, and between towns when the gap passes 1.3), which makes road traffic and carters out of nothing; it is under 1 per cent of hands, and the design's `traded = false` flag would end it honestly. The observer's lists of who serves shelter and warmth (`observer.rs:26-31`) are by name and will not show building or gas.

## 11. Are the engine changes the smallest honest versions?

- **Stage 2, the seed plan**: not needed for the trades (above). Drop it from the critical path; `theta` and `group` are the real stage-2 changes and both are small enough for stage 1.
- **Stage 3, "five fields from the generator"**: the generator on `night/land` has one of them. `Landform` gives `power` (kW a cell), `discharge`, `floodplain`, `marsh`, `rain`, lakes and shores, and the prevailing wind as a single vector for the island. There is no wind field by cell, no fish, no harbour and no salt, and its rock kinds (basement, cap, soft, band, vale, coal measures, alluvium) have no evaporite to put salt in. Smallest honest version: water power from `power`; harbour from the generator's estuary heads and lake shores; fish from coast cells weighted by the outlet's discharge; wind derived from height and the lee of the one vector; **salt dropped** (the alkali works stand on coal and limestone, which is where the design puts them until "the field is read" anyway). Floods and embanking can use `floodplain` as it is.
- **The stream's capacity** is a new rule with one unmeasured number, and problem 8 says it will not bind where it should.
- **Log driving** (one cost table for one good, one way) is the smallest version and is what makes the pulp mill and the steam sawmill buildable. It is worth more than its place in the list suggests.
- **Improvements as data**: `machines::SLOTS` is 2 a sector and the farm would have liming, drainage, embanking, manures and machinery. The design says "more than two slots"; the stock table is `T x NS x SLOTS`, so this is a real change to saved state, not only to `uses`.
- **Foreign trade**: as specified it is the smallest version I can see.

## 12. Against the brief

- **"Beats the old one only after some learning."** By the design's own entry table four inventions undersell the incumbent on the day they are found, in the town that finds them: hot blast (0.84 of the going price), engineering works (0.83), continuous kiln (0.89), iron shipyard (0.62, and 0.46 once know-how spreads). The first three were run and take ten to twenty years to reach half their trade because plant is built a works at a time, so the life cycle does the pacing the recipe does not. The iron shipyard is too strong by any reading: iron 5 to 7 a ship and labour 4 to 6.
- **"Each known method about breaks even at efficient scale at base prices."** None does, old or new: with the engine's size penalties and the spread, the smithy is at -11 per cent, the handloom -27, masons -14, the brewhouse -4. Base prices are not the island's prices (tools open at 18, cloth at 9), so this test cannot be passed by any content and the recipe margin is the fair comparison. On that the new crafts sit in the port's band (17 to 22 per cent against 17 to 26), with three outliers: bricklayers -15 and timber-frame builders 9 (on purpose), water sawmill 44 (problem 8).
- **A wrong statement about the machines rule.** The design says no overlay method is heir to a learned predecessor. One is: the port's powered mill (`A0` 0.5), once it follows the water-powered woollen mill (`A0` 0.9, `b` 0.15), starts from half the water mill's skill; and in seeds where the steam mill is found first the water mill inherits from it. Harmless, but it moves the steam mill's first years and should be in the fit.
- **Windmills and watermills have plant worth two weeks of output** (0.6 and 0.8 a unit), against the design's own rule of 1 to 5 months for a craft.

## 13. Cost

30 goods (31 with ships), 53 methods (56), 13 other inventions, 8 needs, 9 sectors. Not measured. From the loops: the monthly trade pass and the yearly delivered-price pass scale with goods (1.9 times); `service_mix` with goods times needs (2.5 times, small); entry with methods (3.3 times, yearly); the monthly works loop with works times goods, and the new trades add works in every town (builders, brewers, mills, kilns), so perhaps 3 times. The designer's sweep ran eight seeds in 22 seconds, eight at a time, about 12 ms a simulated month. A fair guess for the full tree is 2 to 2.5 times that once the sixteen-term buffer is fixed, and worse if it is not. `traded = false` for services, building work and gas takes three goods out of the trade pass.

## 14. Proposed numbers, in one place

| what | drafted | proposed | buys |
|---|---|---|---|
| gas, as fuel and light | 3.5 | 4.5 (sweep 4 to 5) | a fifth of fuel and light, not 3 per cent |
| `gasworks` K0 | 500 | 150 | a gasworks in a town of ten thousand |
| `ammonia_soda` discover | chemicals 4M, gas 9M, steel 5M | chemicals 2.5M, steel 5M | found 1869, not never |
| `tank_furnace` discover | glass 1.7M, steel 5.8M, gas 10M | glass 700,000, steel 5.8M | found 1873, not never |
| shelter: stone, bricks, timber, sawn timber, lime (overlay) | 0.5, 1.0, 0.6, 0.9, 0.8 | 0.3, 0.6, 0.4, 0.6, 0.5 | builders hold shelter all century; stone down by half, then two thirds |
| `contract_builders` discover (only without the re-rating) | building 35M | building 12M | found 1859 |
| `bricklayers` bricks | 0.7 | 0.6 | a brick trade before the kiln |
| `building` decay | 0.3 | 0.1 | one builder in fourteen wasted, not one in seven |
| wares shares at stage 1 (if stage 2 waits) | 0.03 / 0.10 | 0.01 / 0.03, the rest to comfort | no pottery monster |
| `pottery_works`, `machine_paper`, `sheet_glass`, `alkali_works` K0 | 400, 400, 300, 300 | 150 each | a first works the island can absorb |
| `cement_works` K0 | 1000 | 400 | |
| `pottery_works` discover (stage 2 on) | pottery 80,000 | 40,000 | 1771, not 1779 |
| `machine_paper` discover | paper 300,000, machinery 30,000, cloth x110 | drop cloth | a 33-year spread gone |
| fish, produce as food | 1.3, 1.8 | 1.1, 1.4 | the design's own food table; grain down by a third, not a half; the roller mill found 1877 |
| `ammonia_soda` lime | 0.4 | 0.25 | built, not only found |
| `water_sawmill` timber, labour | 1.3, 0.08 | 1.35, 0.16 | a saw pit that lasts, a steam sawmill that is built |
| `steam_sawmill` A0 | 0.7 | 0.85 | |
| clothing theta (stage 5); cotton cloth | 0.25; 0.85 | 0.5; 0.75 | cotton takes the century to win, not forty years |
| `steam_cotton_mill`, `water_frame_mill` discover (cottons) | 4M, 250,000 | 1.5M, 120,000 | 1805 and 1771 |
| `iron_shipyard` iron, labour | 5, 4 | 7, 6 | does not halve the price of ships in a year |
| coal "xN" thresholds after 1800 | xN | plain units, with iron or machinery beside them | ten years less spread |
| `canal` invention in the overlay | coal x18, stone x13 | delete: the mechanism has its own | one clock |

## 15. What I could not check

Nothing here was run in the engine. The reckoning holds the base goods' prices at the run's, so it misses what the new trades do to coal, timber and the wage (builders want 8 to 12 per cent of hands after the re-rating, which will lift wages early). It prices a good at cost, where the engine's prices overshoot both ways. It has no towns, so it cannot say where anything stands, which is half of what the design is for. Raw cotton's price and the fish and garden sectors' yields are guesses in the design and stay guesses here. The proposed numbers are the ones that fix the reckoning; the first real sweep will move them.

---

# The scripts' tables

All from `.claude/night/tmp/critique/`. `model.py` holds the engine's formulas and the overlay merge; each table names the script that prints it.

## A. Unit sums at base prices (`sums.py`)

Every method of the base file and the overlay, at a wage of 2.5 and heat at 0.376 a unit.

```text
heat a unit at base prices: 0.376

recipe = margin on the recipe as written, no size penalty, no merchant's spread (the design's own column)
K0 columns: at the usual size (labour x1.8, materials x1.1, heat x1.2), merchant's spread 6% off the price, know-how 1, A = A0
0.4K0: the smallest works, know-how 1.  day1: the smallest works in the town that finds it (know-how 0.25, so A0 x 0.625); known methods the same as 0.4K0
ROI = 12 x 0.85 x margin / plant, a year; the hurdle is 0.12
method             st makes      price | recipe |   mats   heat labour  margin  share   plant    ROI |  0.4K0    ROI |   day1    ROI
charcoal_burning    0 charcoal    2.40 |     2% |   1.76   0.00   1.35   -0.85   -36%    24.0  -0.36 |   -54%  -0.55 |   -54%  -0.55
bloomery            0 iron        6.00 |   -83% |   8.80   0.00   5.40   -8.56  -143%    48.0  -1.82 |  -174%  -2.21 |  -174%  -2.21
charcoal_furnace    0 iron        6.00 |   -23% |   7.04   0.00   1.80   -3.20   -53%   120.0  -0.27 |   -67%  -0.34 |   -67%  -0.34
coke_furnace        0 iron        6.00 |    24% |   4.98   0.00   2.86   -2.20   -37%   147.0  -0.15 |   -53%  -0.22 |  -105%  -0.44
services            0 services    3.00 |    17% |   0.00   0.00   4.50   -1.68   -56%     7.2  -2.38 |   -95%  -4.03 |   -95%  -4.03
smithy              0 tools       8.00 |    25% |   3.63   0.27   4.50   -0.88   -11%    24.0  -0.37 |   -28%  -0.96 |   -28%  -0.96
hand_loom           0 cloth       5.00 |    19% |   1.98   0.00   4.05   -1.33   -27%    10.0  -1.36 |   -50%  -2.53 |   -50%  -2.53
textile_mill        0 cloth       5.00 |    57% |   2.31   0.22   1.35    0.81    16%    74.0   0.11 |     6%   0.04 |   -24%  -0.17
brickworks          0 bricks      1.80 |    26% |   0.66   0.27   0.90   -0.14    -8%    14.0  -0.10 |   -24%  -0.32 |   -24%  -0.32
millwright          0 machinery  70.00 |    19% |  23.76   0.45  63.00  -21.41   -31%    56.0  -3.90 |   -56%  -7.10 |   -56%  -7.10
engine_works        0 machinery  70.00 |    47% |  32.14   1.43  67.50  -35.27   -50%   466.0  -0.77 |   -78%  -1.19 |  -160%  -2.45
crucible_steel      0 steel      30.00 |    37% |  11.10   1.92  36.00  -20.82   -69%    60.0  -3.54 |  -103%  -5.25 |  -202% -10.32
bessemer            0 steel      30.00 |    72% |   9.67   0.38   4.50   13.64    45%   123.0   1.13 |    40%   0.99 |    22%   0.55
forge_rails         0 rails      18.00 |    20% |   8.25   0.68  11.25   -3.26   -18%    24.0  -1.38 |   -37%  -2.83 |   -37%  -2.83
rolling_mill        0 rails      18.00 |    51% |   9.25   0.51   7.20   -0.04    -0%   250.0  -0.00 |   -14%  -0.10 |   -54%  -0.40
steel_rolling       0 rails      18.00 |    31% |  13.81   0.35   5.25   -2.49   -14%    96.3  -0.26 |   -26%  -0.49 |   -63%  -1.19
masons              1 building    2.20 |    20% |   1.25   0.00   1.12   -0.31   -14%     2.8  -1.13 |   -30%  -2.43 |   -30%  -2.43
carpenters          1 building    2.20 |     9% |   1.61   0.00   0.99   -0.53   -24%     2.8  -1.92 |   -39%  -3.16 |   -39%  -3.16
bricklayers         1 building    2.20 |   -15% |   2.13   0.00   1.08   -1.15   -52%     2.8  -4.17 |   -70%  -5.60 |   -70%  -5.60
contract_builders   1 building    2.20 |     0% |   2.14   0.00   0.45   -0.52   -24%     8.8  -0.61 |   -34%  -0.87 |   -68%  -1.73
saw_pit             1 lumber      2.20 |    17% |   1.23   0.00   1.26   -0.42   -19%     2.0  -2.16 |   -37%  -4.15 |   -37%  -4.15
steam_sawmill       1 lumber      2.20 |    44% |   1.25   0.16   0.32    0.34    15%    11.7   0.29 |     8%   0.15 |   -16%  -0.31
lime_kiln           1 lime        2.00 |    18% |   1.06   0.59   0.36   -0.12    -6%     5.1  -0.25 |   -16%  -0.66 |   -16%  -0.66
cement_works        1 lime        2.00 |    53% |   0.74   0.52   0.15    0.47    23%    19.5   0.25 |    17%   0.18 |    -3%  -0.03
brewhouse           1 beer        2.00 |    20% |   1.32   0.18   0.45   -0.07    -4%     6.6  -0.11 |   -14%  -0.42 |   -14%  -0.42
steam_brewery       1 beer        2.00 |    47% |   1.21   0.15   0.15    0.37    19%    14.8   0.26 |    13%   0.18 |    -6%  -0.09
potbank             1 pottery     3.40 |    20% |   0.57   1.13   2.25   -0.76   -22%    14.0  -0.55 |   -43%  -1.08 |   -43%  -1.08
pottery_works       1 pottery     3.40 |    58% |   0.61   0.90   0.97    0.72    21%    22.0   0.33 |    10%   0.16 |   -22%  -0.35
gasworks            1 gas         2.40 |    45% |   1.35   0.00   0.32    0.59    24%    32.2   0.19 |    18%   0.14 |    -2%  -0.02
edge_tool_works     1 tools       8.00 |    34% |   4.45   0.29   2.25    0.53     7%    31.4   0.17 |    -4%  -0.10 |   -37%  -0.95
tool_factory        1 tools       8.00 |    13% |   7.26   0.23   0.27   -0.24    -3%    64.0  -0.04 |    -9%  -0.11 |   -29%  -0.37
hot_blast           1 iron        6.00 |    71% |   1.76   0.00   0.23    3.65    61%   116.5   0.32 |    58%   0.31 |    50%   0.26
machine_shop        1 machinery  70.00 |    73% |  18.81   0.45   2.70   43.84    63%   230.0   1.94 |    60%   1.87 |    52%   1.61
engineering_works   1 machinery  70.00 |    73% |  20.50   0.36   0.45   44.48    64%   293.0   1.55 |    62%   1.51 |    56%   1.36
continuous_kiln     1 bricks      1.80 |    65% |   0.53   0.09   0.14    0.94    52%    17.6   0.54 |    48%   0.50 |    36%   0.37
emporium            1 services    3.00 |    52% |   0.21   0.00   2.25    0.36    12%    15.6   0.24 |    -8%  -0.15 |   -66%  -1.29
soap_boilers        2 chemicals   4.50 |    22% |   2.20   0.45   2.03   -0.45   -10%     6.8  -0.67 |   -25%  -1.69 |   -25%  -1.69
alkali_works        2 chemicals   4.50 |    56% |   0.60   1.62   0.96    1.05    23%    58.8   0.18 |    14%   0.11 |   -16%  -0.12
ammonia_soda        2 chemicals   4.50 |    72% |   0.88   0.45   0.18    2.72    60%    82.5   0.34 |    57%   0.32 |    48%   0.27
glasshouse          2 glass       9.00 |    20% |   2.15   2.71   5.40   -1.80   -20%    39.6  -0.46 |   -40%  -0.92 |   -40%  -0.92
sheet_glass         2 glass       9.00 |    52% |   2.53   2.33   2.62    0.97    11%    63.5   0.16 |    -1%  -0.01 |   -36%  -0.51
tank_furnace        2 glass       9.00 |    73% |   1.85   0.68   0.41    5.53    61%    73.5   0.77 |    58%   0.73 |    49%   0.61
paper_mill          2 paper       4.60 |    20% |   1.10   0.23   4.50   -1.50   -33%    12.8  -1.20 |   -60%  -2.19 |   -60%  -2.19
machine_paper       2 paper       4.60 |    54% |   1.61   0.58   1.50    0.63    14%    41.0   0.16 |     2%   0.02 |   -33%  -0.37
pulp_mill           3 paper       4.60 |    75% |   0.92   0.27   0.18    2.95    64%    39.0   0.77 |    61%   0.74 |    53%   0.64
windmill            3 flour       1.30 |    11% |   1.26   0.00   0.02   -0.07    -5%     0.6  -1.07 |   -11%  -2.27 |   -11%  -2.27
watermill           3 flour       1.30 |    12% |   1.24   0.00   0.02   -0.04    -3%     0.8  -0.47 |    -8%  -1.32 |    -8%  -1.32
roller_mill         3 flour       1.30 |    24% |   1.04   0.04   0.01    0.13    10%     1.4   0.95 |     6%   0.52 |   -11%  -0.98
water_wool_mill     3 cloth       5.00 |    56% |   1.71   0.00   1.40    1.59    32%    52.4   0.31 |    23%   0.22 |    -5%  -0.05
water_sawmill       3 lumber      2.20 |    44% |   1.14   0.00   0.36    0.56    26%     9.4   0.61 |    19%   0.45 |    19%   0.45
fustian_weaving     5 cottons     6.00 |    18% |   2.66   0.00   4.50   -1.52   -25%    10.0  -1.55 |   -47%  -2.88 |   -47%  -2.88
water_frame_mill    5 cottons     6.00 |    52% |   2.51   0.00   1.40    1.73    29%    52.4   0.34 |    21%   0.24 |    -5%  -0.06
steam_cotton_mill   5 cottons     6.00 |    62% |   2.16   0.18   0.27    3.03    51%    46.5   0.67 |    47%   0.62 |    36%   0.48
shipyard            6 ships     160.00 |    19% |  97.90   0.00  72.00  -19.50   -12%   142.0  -1.40 |   -27%  -3.11 |   -27%  -3.11
iron_shipyard       6 ships     160.00 |    52% |  74.87   1.43  20.00   54.10    34%   456.0   1.21 |    28%   1.00 |    10%   0.34
steel_shipyard      6 ships     160.00 |    28% | 121.88   0.90   5.40   22.22    14%   488.0   0.46 |     9%   0.30 |    -8%  -0.27

== known in 1750, by return at the usual size (base prices)
  bricklayers        building  recipe  -15%  K0 margin   -52%  ROI  -4.17   0.4K0 ROI  -5.60   NEW
  millwright         machinery recipe   19%  K0 margin   -31%  ROI  -3.90   0.4K0 ROI  -7.10   base
  services           services  recipe   17%  K0 margin   -56%  ROI  -2.38   0.4K0 ROI  -4.03   base
  saw_pit            lumber    recipe   17%  K0 margin   -19%  ROI  -2.16   0.4K0 ROI  -4.15   NEW
  carpenters         building  recipe    9%  K0 margin   -24%  ROI  -1.92   0.4K0 ROI  -3.16   NEW
  bloomery           iron      recipe  -83%  K0 margin  -143%  ROI  -1.82   0.4K0 ROI  -2.21   base
  fustian_weaving    cottons   recipe   18%  K0 margin   -25%  ROI  -1.55   0.4K0 ROI  -2.88   NEW
  shipyard           ships     recipe   19%  K0 margin   -12%  ROI  -1.40   0.4K0 ROI  -3.11   NEW
  forge_rails        rails     recipe   20%  K0 margin   -18%  ROI  -1.38   0.4K0 ROI  -2.83   base
  hand_loom          cloth     recipe   19%  K0 margin   -27%  ROI  -1.36   0.4K0 ROI  -2.53   base
  paper_mill         paper     recipe   20%  K0 margin   -33%  ROI  -1.20   0.4K0 ROI  -2.19   NEW
  masons             building  recipe   20%  K0 margin   -14%  ROI  -1.13   0.4K0 ROI  -2.43   NEW
  windmill           flour     recipe   11%  K0 margin    -5%  ROI  -1.07   0.4K0 ROI  -2.27   NEW
  soap_boilers       chemicals recipe   22%  K0 margin   -10%  ROI  -0.67   0.4K0 ROI  -1.69   NEW
  potbank            pottery   recipe   20%  K0 margin   -22%  ROI  -0.55   0.4K0 ROI  -1.08   NEW
  watermill          flour     recipe   12%  K0 margin    -3%  ROI  -0.47   0.4K0 ROI  -1.32   NEW
  glasshouse         glass     recipe   20%  K0 margin   -20%  ROI  -0.46   0.4K0 ROI  -0.92   NEW
  smithy             tools     recipe   25%  K0 margin   -11%  ROI  -0.37   0.4K0 ROI  -0.96   base
  charcoal_burning   charcoal  recipe    2%  K0 margin   -36%  ROI  -0.36   0.4K0 ROI  -0.55   base
  charcoal_furnace   iron      recipe  -23%  K0 margin   -53%  ROI  -0.27   0.4K0 ROI  -0.34   base
  lime_kiln          lime      recipe   18%  K0 margin    -6%  ROI  -0.25   0.4K0 ROI  -0.66   NEW
  brewhouse          beer      recipe   20%  K0 margin    -4%  ROI  -0.11   0.4K0 ROI  -0.42   NEW
  brickworks         bricks    recipe   26%  K0 margin    -8%  ROI  -0.10   0.4K0 ROI  -0.32   base
  water_sawmill      lumber    recipe   44%  K0 margin    26%  ROI   0.61   0.4K0 ROI   0.45   NEW

== inventions: against the best known/earlier method of the same good, at base prices
   cost+hurdle = what a unit must sell for (spread included) to clear the 12% hurdle
method             makes       price |     day1   know 1    A=cap | older methods' need at K0 (A0), and at 2xK0 learned
coke_furnace       iron         6.00 |    14.54    11.25     5.07 | bloomery 15.71 / 14.71; charcoal_furnace 10.91 / 10.46
textile_mill       cloth        5.00 |     7.20     5.59     2.39 | hand_loom 6.54 / 5.92; water_wool_mill 3.97 / 2.82
engine_works       machinery   70.00 |   194.94   133.85    31.84 | millwright 93.48 / 84.06
crucible_steel     steel       30.00 |    95.34    63.61    19.68 | 
bessemer           steel       30.00 |    24.52    18.85     8.36 | crucible_steel 52.90 / 19.68
rolling_mill       rails       18.00 |    31.52    23.73    10.99 | forge_rails 21.77 / 19.94
steel_rolling      rails       18.00 |    31.20    24.11    12.36 | forge_rails 21.77 / 19.94; rolling_mill 21.18 / 10.99
contract_builders  building     2.20 |     3.90     3.11     1.98 | masons 2.57 / 2.37; carpenters 2.80 / 2.61; bricklayers 3.45 / 3.24
steam_sawmill      lumber       2.20 |     2.72     2.16     1.37 | saw_pit 2.68 / 2.47; water_sawmill 1.72 / 1.64
cement_works       lime         2.00 |     2.31     1.88     1.24 | lime_kiln 2.19 / 2.08
steam_brewery      beer         2.00 |     2.32     1.91     1.30 | brewhouse 2.16 / 2.05
pottery_works      pottery      3.40 |     4.49     3.31     1.62 | potbank 4.38 / 3.99
gasworks           gas          2.40 |     2.87     2.34     1.70 | 
edge_tool_works    tools        8.00 |    11.51     8.72     5.51 | smithy 9.24 / 6.61
tool_factory       tools        8.00 |    11.30     9.56     7.23 | smithy 9.24 / 6.61; edge_tool_works 7.83 / 5.51
hot_blast          iron         6.00 |     4.26     3.73     2.73 | bloomery 15.71 / 14.71; charcoal_furnace 10.91 / 10.46; coke_furnace 10.18 / 5.07
machine_shop       machinery   70.00 |    34.16    28.09    19.12 | millwright 93.48 / 84.06; engine_works 113.35 / 31.84
engineering_works  machinery   70.00 |    32.24    27.66    21.44 | millwright 93.48 / 84.06; engine_works 113.35 / 31.84; machine_shop 26.24 / 19.12
continuous_kiln    bricks       1.80 |     1.34     1.10     0.80 | brickworks 2.12 / 1.49
emporium           services     3.00 |     5.29     3.44     1.79 | services 4.88 / 3.13
alkali_works       chemicals    4.50 |     5.98     4.59     2.51 | soap_boilers 5.06 / 4.69
ammonia_soda       chemicals    4.50 |     3.25     2.79     2.02 | soap_boilers 5.06 / 4.69; alkali_works 4.12 / 2.51
sheet_glass        glass        9.00 |    13.20     9.87     4.59 | glasshouse 11.41 / 10.46
tank_furnace       glass        9.00 |     5.23     4.32     3.01 | glasshouse 11.41 / 10.46; sheet_glass 8.76 / 4.59
machine_paper      paper        4.60 |     6.71     5.01     2.37 | paper_mill 6.36 / 5.69
pulp_mill          paper        4.60 |     2.49     2.08     1.40 | paper_mill 6.36 / 5.69; machine_paper 4.44 / 2.37
roller_mill        flour        1.30 |     1.47     1.24     0.93 | windmill 1.38 / 1.34; watermill 1.35 / 1.31
water_wool_mill    cloth        5.00 |     5.91     4.45     2.82 | hand_loom 6.54 / 5.92
water_frame_mill   cottons      6.00 |     6.97     5.34     3.49 | fustian_weaving 7.74 / 7.05
steam_cotton_mill  cottons      6.00 |     4.27     3.57     2.51 | fustian_weaving 7.74 / 7.05; water_frame_mill 4.82 / 3.49
iron_shipyard      ships      160.00 |   149.50   118.00    73.99 | shipyard 182.52 / 169.78
steel_shipyard     ships      160.00 |   179.92   150.90   104.84 | shipyard 182.52 / 169.78; iron_shipyard 108.15 / 73.99
```

## B. Demand at base prices, a town of 20,000 (`demand.py`)

Goods budget 3 a head a month; the stages are 0 (the game's own content), 1, 2, 3 and 5.

```text
A town of 20000 people, goods budget 3.0 a head a month, base prices (wage 2.5). Units a month.

-- cost of subsistence a head, and the share of household spending on each need
stage 0: subsistence 1.33, real income 2.26; food 49%, warmth 11%, shelter 15%, clothing 15%, implements 4%, comfort 6%
stage 1: subsistence 1.30, real income 2.32; food 47%, warmth 10%, shelter 13%, clothing 14%, implements 5%, comfort 6%, table 3%, wares 2%
stage 2: subsistence 1.30, real income 2.32; food 47%, warmth 10%, shelter 13%, clothing 14%, implements 5%, comfort 6%, table 3%, wares 2%
stage 3: subsistence 1.22, real income 2.47; food 45%, warmth 11%, shelter 13%, clothing 15%, implements 5%, comfort 6%, table 4%, wares 2%
stage 5: subsistence 1.22, real income 2.46; food 45%, warmth 11%, shelter 13%, clothing 15%, implements 5%, comfort 6%, table 4%, wares 2%

-- who serves each need (share of the service), by stage
  food       grain        100%   100%   100%    18%    18%
  food       flour           -      -      -    34%    34%
  food       fish            -      -      -    18%    18%
  food       produce         -      -      -    30%    30%
  warmth     timber         1%     1%     1%     1%     1%
  warmth     coal          99%    99%    99%    99%    99%
  shelter    timber        36%     3%     3%     3%     3%
  shelter    stone         56%     7%     7%     7%     7%
  shelter    bricks         8%     0%     0%     0%     0%
  shelter    building        -    89%    89%    89%    89%
  clothing   wool          66%    66%    66%    66%    63%
  clothing   cloth         34%    34%    34%    34%    32%
  clothing   cottons         -      -      -      -     5%
  implements iron           1%     1%     1%     1%     1%
  implements steel         27%    27%    27%    27%    27%
  implements tools         73%    73%    73%    73%    73%
  comfort    services     100%   100%   100%   100%   100%
  table      beer            -   100%   100%   100%   100%
  wares      timber          -     0%    10%    10%    10%
  wares      lumber          -     3%    13%    13%    13%
  wares      pottery         -    96%    20%    20%    20%
  wares      chemicals       -      -    19%    19%    19%
  wares      glass           -      -    19%    19%    19%
  wares      paper           -      -    19%    19%    19%

-- the town's monthly market for each good: households and landlords + running inputs of works = total
good          stage 0              stage 1              stage 2              stage 3              stage 5          
grain        29886+     0=  29886   28838+  1742=  30580   28838+  1742=  30580    5379+ 10061=  15440    5379+ 10057=  15435
wool          5137+  1219=   6356    4706+  1116=   5822    4706+  1139=   5845    4818+  1167=   5986    4495+  1091=   5586
timber        6396+  1001=   7397    1076+  2796=   3872    1579+  3255=   4833    1638+  3342=   4980    1635+  3337=   4972
stone        11569+     0=  11569    2099+  6262=   8361    2099+  6307=   8406    2155+  6478=   8633    2153+  6470=   8623
clay             0+  1292=   1292       0+   964=    964       0+   215=    215       0+   226=    226       0+   226=    226
iron_ore         0+   453=    453       0+   441=    441       0+   441=    441       0+   454=    454       0+   453=    453
coal          8328+   331=   8659    7873+  1471=   9344    7873+  1005=   8878    7989+  1047=   9036    7984+  1045=   9030
charcoal         0+   453=    453       0+   441=    441       0+   441=    441       0+   454=    454       0+   453=    454
iron            10+   196=    206       9+   191=    200       9+   191=    200      10+   197=    206      10+   197=    206
steel           40+     0=     40      39+     0=     39      39+     0=     39      40+     0=     40      40+     0=     40
tools          324+     0=    324     316+     0=    316     316+     0=    316     325+     0=    325     325+     0=    325
cloth          923+     0=    923     846+     0=    846     846+    17=    863     866+    18=    884     808+    18=    826
bricks         783+     0=    783      40+     0=     40      40+     0=     40      41+     0=     41      41+     0=     41
machinery        0+     0=      0       0+     0=      0       0+     0=      0       0+     0=      0       0+     0=      0
rails            0+     0=      0       0+     0=      0       0+     0=      0       0+     0=      0       0+     0=      0
services      2567+     0=   2567    2281+     0=   2281    2281+     0=   2281    2418+     0=   2418    2411+     0=   2411
building                       -    4460+     0=   4460    4460+     0=   4460    4580+     0=   4580    4574+     0=   4574
lumber                         -      62+     0=     62     196+     0=    196     209+     0=    209     208+     0=    208
lime                           -       1+   491=    492       1+   491=    492       1+   504=    505       1+   503=    504
beer                           -    1320+     0=   1320    1320+     0=   1320    1374+     0=   1374    1371+     0=   1371
pottery                        -     628+     0=    628     104+     0=    104     111+     0=    111     111+     0=    111
gas                            -                     -                     -                     -                     -
chemicals                      -                     -      80+    13=     93      85+    14=     99      84+    14=     99
glass                          -                     -      41+     0=     41      43+     0=     43      43+     0=     43
paper                          -                     -      79+     0=     79      84+     0=     84      83+     0=     83
flour                          -                     -                     -    6635+     0=   6635    6634+     0=   6634
fish                           -                     -                     -    4137+     0=   4137    4137+     0=   4137
produce                        -                     -                     -    4990+     0=   4990    4990+     0=   4990
cotton                         -                     -                     -                     -                     -
cottons                        -                     -                     -                     -     139+     0=    139

-- against the smallest works (0.4 K0) of each method, at the fullest stage that has the good (stage 5)
method             makes       0.4 K0   market works fit  
charcoal_burning   charcoal       8.0      454      56.7  
bloomery           iron           1.2      206     171.8  
charcoal_furnace   iron           8.0      206      25.8  
coke_furnace       iron          16.0      206      12.9    (an invention: its market is larger by the time it is found)
services           services      80.0     2411      30.1  
smithy             tools          4.0      325      81.2  
hand_loom          cloth         32.0      826      25.8  
textile_mill       cloth         60.0      826      13.8    (an invention: its market is larger by the time it is found)
brickworks         bricks        24.0       41       1.7  thin: under three smallest works
millwright         machinery      0.2        0       0.3  THIN: one smallest works is more than the town's whole market
engine_works       machinery      3.2        0       0.0  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
crucible_steel     steel          4.0       40      10.0    (an invention: its market is larger by the time it is found)
bessemer           steel         60.0       40       0.7  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
forge_rails        rails          0.8        0       0.0  THIN: one smallest works is more than the town's whole market
rolling_mill       rails         16.0        0       0.0  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
steel_rolling      rails         32.0        0       0.0  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
masons             building     120.0     4574      38.1  
carpenters         building      80.0     4574      57.2  
bricklayers        building     120.0     4574      38.1  
contract_builders  building     320.0     4574      14.3    (an invention: its market is larger by the time it is found)
saw_pit            lumber        24.0      208       8.7  
steam_sawmill      lumber       240.0      208       0.9  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
lime_kiln          lime          40.0      504      12.6  
cement_works       lime         400.0      504       1.3  thin: under three smallest works  (an invention: its market is larger by the time it is found)
brewhouse          beer         120.0     1371      11.4  
steam_brewery      beer         600.0     1371       2.3  thin: under three smallest works  (an invention: its market is larger by the time it is found)
potbank            pottery       16.0      111       6.9  
pottery_works      pottery      160.0      111       0.7  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
edge_tool_works    tools         24.0      325      13.5    (an invention: its market is larger by the time it is found)
tool_factory       tools        240.0      325       1.4  thin: under three smallest works  (an invention: its market is larger by the time it is found)
hot_blast          iron          60.0      206       3.4    (an invention: its market is larger by the time it is found)
machine_shop       machinery     16.0        0       0.0  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
engineering_works  machinery     60.0        0       0.0  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
continuous_kiln    bricks       800.0       41       0.1  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
emporium           services     400.0     2411       6.0    (an invention: its market is larger by the time it is found)
soap_boilers       chemicals     16.0       99       6.2  
alkali_works       chemicals    120.0       99       0.8  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
ammonia_soda       chemicals    800.0       99       0.1  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
glasshouse         glass         12.0       43       3.6  
sheet_glass        glass        120.0       43       0.4  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
tank_furnace       glass        600.0       43       0.1  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
paper_mill         paper         12.0       83       6.9  
machine_paper      paper        160.0       83       0.5  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
pulp_mill          paper        800.0       83       0.1  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)
windmill           flour        320.0     6634      20.7  
watermill          flour        480.0     6634      13.8  
roller_mill        flour       1200.0     6634       5.5    (an invention: its market is larger by the time it is found)
water_wool_mill    cloth         40.0      826      20.7    (an invention: its market is larger by the time it is found)
water_sawmill      lumber        80.0      208       2.6  thin: under three smallest works
fustian_weaving    cottons       24.0      139       5.8  
water_frame_mill   cottons       48.0      139       2.9  thin: under three smallest works  (an invention: its market is larger by the time it is found)
steam_cotton_mill  cottons      400.0      139       0.3  THIN: one smallest works is more than the town's whole market  (an invention: its market is larger by the time it is found)

-- what the overlay does to base goods' markets in this town, stage 0 -> 1 -> 2 -> 3 -> 5 (total units a month)
grain          29886     30580     30580     15440     15435    -48%
wool            6356      5822      5845      5986      5586    -12%
timber          7397      3872      4833      4980      4972    -33%
stone          11569      8361      8406      8633      8623    -25%
clay            1292       964       215       226       226    -83%
iron_ore         453       441       441       454       453    +0%
coal            8659      9344      8878      9036      9030    +4%
charcoal         453       441       441       454       454    +0%
iron             206       200       200       206       206    +0%
steel             40        39        39        40        40    +0%
tools            324       316       316       325       325    +0%
cloth            923       846       863       884       826    -11%
bricks           783        40        40        41        41    -95%
machinery          0         0         0         0         0    +0%
rails              0         0         0         0         0 
services        2567      2281      2281      2418      2411    -6%
```

## C. The reckoning at the game's prices, as drafted, full content (`reckon.py 5`)

Raw cotton at twice the `[world]` path; fish and produce at 1.3 and 1.6 times grain. Rows marked (run) are base goods taken from the designer's sweep.

```text
==============================================================================================================
STAGE 5  
budget a head a month: 1750:2.27 1770:2.94 1790:3.81 1810:4.42 1830:5.05 1850:5.70 1870:6.66 1890:7.63 1899:7.96
wage:                  1750:4.05 1770:5.28 1790:6.72 1810:7.48 1830:8.59 1850:9.67 1870:11.89 1890:14.14 1899:14.71

-- price of each new good (cost of the methods making it, weighted)
               1750     1770     1790     1810     1830     1850     1870     1890     1899
building       4.30     4.96     6.22     6.92     8.10     8.00     8.42     4.25     3.82
lumber         3.12     3.41     4.23     4.78     5.89     5.85     6.01     5.58     5.69
lime           4.87     4.02     4.83     5.13     5.60     4.69     2.78     2.14     2.13
beer           2.52     2.65     2.71     1.70     1.72     1.40     0.99     0.85     0.86
pottery        8.96     7.69     7.18     5.39     5.30     3.97     3.50     3.17     3.13
gas            0.00     0.00     0.00     0.00     3.99     3.06     2.73     2.48     2.53
chemicals      9.19     9.29    11.61    12.93     8.23     5.45     4.42     3.49     3.39
glass         23.84    20.37    24.99    26.83    27.87    25.49     9.93     7.17     6.70
paper         10.07    12.48    14.97    15.45     7.28     4.39     3.81     3.36     3.27
flour          0.99     1.31     1.10     1.11     1.29     1.12     0.66     0.61     0.55
fish           0.95     1.26     1.03     1.04     1.20     1.02     0.55     0.48     0.51
produce        1.16     1.55     1.27     1.28     1.48     1.26     0.67     0.59     0.63
cotton         4.40     4.16     3.92     3.67     3.40     3.13     3.31     2.84     2.62
cottons       12.49    14.25    10.71     6.97     6.63     4.04     3.55     2.78     2.54

-- units a month, whole island (households, landlords, and running inputs of works)
building       9672    12071    16467    21473    25293    25904    21103   135639   196900
lumber         2283     4342     6078     7821     8603     9005     8623    30318    39417
lime            764      707     1021     1465     2127     3016    28242    29026    28830
beer           3656     6007    11176    29864    43590    86519   208786   380332   444487
pottery         174      285      591     1395     2385     7263    16889    31851    39109
gas               0        0        0        0     3058     3523     5026    10347    17984
chemicals       175      263      465      812     1970     7099    17722    36096    44709
glass            67      111      194      340      550     1157     6234    15979    20553
paper           146      192      336      587     1784     6100    14546    29470    37095
flour         35495    41488    56795    73677    92285   120101   147879   172389   341029
fish          26796    30933    46363    61769    77515   108732   206124   300709   289133
produce       31907    36833    55207    73551    92300   129472   245441   358068   344284
cotton          316     1069     3832    11822    10793    47017   106444   210251   262530
cottons         269      908     3479    13415    12684    63849   160209   343207   440612
-- base goods wanted by households and by the works of the new goods (the run's own output beneath)
stone         17056    17030    25873    36443    58470   154213   329825   312692   260640
 (run)        32701    37354    56865    76839   123636   265868   523501   727379   706068
bricks         1114     1614     3095     7291    12049    30619    81518   282389   393855
 (run)         3309     5467    11219    24835    44051    77419   179464   572510   775563
timber        33337    18299    25490    31409    28375    29121    27010    51312    63239
 (run)        38726    27909    34368    35231    32292    25568    19966    14001    14422
grain         83441    98676   145609   200008   253087   362417   619091   895519  1079039
 (run)       183002   216570   327288   428326   532858   744891  1318664  1920955  2090102
coal          11878    41235    60845    90085   122946   234521   405818   653472   719864
 (run)        12330    43768    82678   127543   185446   321977   495056   768569   850094
cloth          2872     2759     2496     6147    30271    27637    13084     5082     5296
 (run)         2988     3217     3611    17206    50545    81968   114881   154578   170371
wool          13476    15128    25557    18964     2938       55        0        0        0
 (run)        18620    21688    39650    45415    44239    58250    80166   104815   114203
steel             0        0      345     2438     4212     6913     8927    46396    69068
 (run)            0       16      400     2471     4208     6910    16176    55022    74676
tools           970     1257     1600      753      326     1133     5948    16543    16696
 (run)          939     1251     1772      948      679     2294     7940    19525    23179
services       4133     4812     7036    12559    18281    30888    58418   101806   129384
 (run)         5533     6404     9028    16527    25050    43010    84518   150643   192254
clay           1120     2274     3316     4757     5872    10857    25661    37437    43275
 (run)         4880     7889    15434    32104    54808    94193   210777   586278   743721

-- who serves each need (share of the need's service), and the need's share of household spending
food        spend:      39%      42%      33%      30%      30%      25%      15%      13%      13%
   grain               19%      19%      20%      20%      20%      20%      22%      23%      20%
   flour               30%      30%      28%      27%      27%      26%      18%      15%      27%
   fish                19%      19%      20%      20%      20%      20%      22%      23%      20%
   produce             32%      32%      33%      33%      33%      33%      37%      38%      33%
warmth      spend:      21%      11%      12%      11%      11%       9%       8%       8%       8%
   timber              44%       0%       0%       0%       0%       0%       0%       0%       0%
   coal                55%     100%     100%     100%      96%      97%      98%      98%      97%
   charcoal             1%       0%       0%       0%       0%       0%       0%       0%       0%
   gas                  0%       0%       0%       0%       4%       3%       2%       2%       3%
shelter     spend:      11%      13%      16%      17%      18%      19%      20%      20%      19%
   timber               4%       8%       8%       6%       4%       3%       1%       0%       0%
   stone               12%      11%      12%      13%      18%      37%      45%      17%      10%
   bricks               1%       1%       2%       5%       8%      15%      26%      27%      29%
   building            82%      79%      77%      74%      70%      45%      21%      55%      61%
   lime                 0%       0%       0%       0%       0%       0%       7%       2%       1%
clothing    spend:      18%      20%      19%      18%      16%      16%      17%      17%      17%
   wool                61%      60%      62%      28%       2%       0%       0%       0%       0%
   cloth               36%      31%      17%      25%      72%      33%       7%       0%       0%
   cottons              3%       9%      21%      47%      26%      67%      93%     100%     100%
implements  spend:       4%       5%       5%       5%       5%       6%       6%       6%       6%
   steel                0%       0%      39%      91%      97%      92%      62%      85%      89%
   tools              100%     100%      61%       9%       3%       5%      14%      10%       7%
   machinery            0%       0%       0%       0%       0%       2%      24%       5%       4%
comfort     spend:       3%       4%       7%      10%      11%      15%      20%      22%      23%
   services           100%     100%     100%     100%     100%     100%     100%     100%     100%
table       spend:       2%       3%       4%       4%       5%       5%       6%       6%       6%
   grain                2%       0%       2%       0%       0%       0%       0%       0%       0%
   beer                98%     100%      98%     100%     100%     100%     100%     100%     100%
wares       spend:       1%       1%       3%       3%       4%       5%       7%       8%       8%
   timber              11%      11%       9%       6%       3%       1%       0%       1%       0%
   lumber              17%      16%      16%      13%       9%       4%       2%       2%       2%
   pottery             17%      19%      22%      29%      26%      27%      24%      23%      23%
   chemicals           20%      20%      19%      19%      23%      27%      24%      25%      25%
   glass               16%      18%      18%      17%      15%      11%      22%      24%      25%
   paper               19%      16%      16%      16%      25%      30%      26%      26%      26%

-- method shares of each new good's output
   building   masons                  52%      41%      41%      41%      45%      48%      48%       1%       0%
   building   carpenters              38%      48%      46%      42%      36%      30%      23%       0%       0%
   building   bricklayers             10%      11%      13%      17%      19%      22%      29%       0%       0%
   building   contract_builders        0%       0%       0%       0%       0%       0%       0%      99%     100%
   lumber     saw_pit                 10%       7%       6%       6%       7%       5%       3%       1%       1%
   lumber     steam_sawmill            0%       0%       0%       0%       0%       0%       0%       0%       0%
   lumber     water_sawmill           90%      93%      94%      94%      93%      95%      97%      99%      99%
   lime       lime_kiln              100%     100%     100%     100%     100%     100%       4%       1%       1%
   lime       cement_works             0%       0%       0%       0%       0%       0%      96%      99%      99%
   beer       brewhouse              100%     100%     100%       3%       1%       0%       0%       0%       0%
   beer       steam_brewery            0%       0%       0%      97%      99%     100%     100%     100%     100%
   pottery    potbank                100%     100%      20%       1%       0%       0%       0%       0%       0%
   pottery    pottery_works            0%       0%      80%      99%     100%     100%     100%     100%     100%
   gas        gasworks                 0%       0%       0%       0%     100%     100%     100%     100%     100%
   chemicals  soap_boilers           100%     100%     100%     100%       1%       0%       0%       0%       0%
   chemicals  alkali_works             0%       0%       0%       0%      99%     100%     100%     100%     100%
   chemicals  ammonia_soda             0%       0%       0%       0%       0%       0%       0%       0%       0%
   glass      glasshouse             100%     100%     100%     100%     100%      70%       0%       0%       0%
   glass      sheet_glass              0%       0%       0%       0%       0%      30%     100%     100%     100%
   glass      tank_furnace             0%       0%       0%       0%       0%       0%       0%       0%       0%
   paper      paper_mill             100%     100%     100%     100%       0%       0%       0%       0%       0%
   paper      machine_paper            0%       0%       0%       0%     100%     100%     100%     100%     100%
   paper      pulp_mill                0%       0%       0%       0%       0%       0%       0%       0%       0%
   flour      windmill                48%      48%      47%      47%      47%      47%      46%      44%      13%
   flour      watermill               52%      52%      53%      53%      53%      53%      54%      56%      17%
   flour      roller_mill              0%       0%       0%       0%       0%       0%       0%       0%      70%
   cottons    fustian_weaving        100%     100%      10%       0%       0%       0%       0%       0%       0%
   cottons    water_frame_mill         0%       0%      90%     100%      80%       5%       1%       0%       0%
   cottons    steam_cotton_mill        0%       0%       0%       0%      20%      95%      99%     100%     100%

-- hands in the new trades (labour of the works making each new good), and all hands at work
building       3642     4430     6054     7926     9388     9669     7942    14828    17953
lumber          356      639      883     1140     1258     1252     1175     3973     5137
lime             96       89      128      183      266      378     1250      954      897
beer            572      941     1750     1058     1142     1866     3787     5908     6507
pottery         136      223      279      340      461     1130     2088     3195     3620
gas               0        0        0        0      284      279      362      690     1150
chemicals       123      186      328      572      519     1143     2107     3361     3805
glass           126      208      365      639     1033     1970     2322     3865     4328
paper           229      301      526      920      657     1136     1871     2841     3210
flour           249      291      398      516      646      840     1032     1200     1379
fish              0        0        0        0        0        0        0        0        0
produce           0        0        0        0        0        0        0        0        0
cotton            0        0        0        0        0        0        0        0        0
cottons         421     1421     2348     3381     2655     3735     6343    10444    12357
all hands     67500    74323    86935   110058   140256   179119   230562   297467   333298

-- discovery (median year by the engine's hazard; 'met' = year the last threshold is reached)
invention               aim |   met found   10%..90%   | entered | the threshold that binds, and progress in 1899
coke_furnace                |  1772  1774  1766..1784  |   (run) | coal 92.91
textile_mill           1798 |  1795  1796  1785..1807  |   (run) | cloth 56.96
engine_works                |  1782  1784  1775..1793  |   (run) | iron 112.29
crucible_steel              |  1764  1768  1761..1776  |   (run) | iron 187.15
bessemer               1856 |  1854  1853  1841..1865  |   (run) | steel 7.22
rolling_mill                |  1797  1798  1786..1808  |   (run) | rails 20.95
steel_rolling          1866 |  1870  1869  1853..1881  |   (run) | rails 2.25
contract_builders      1862 |  1883  1866  1849..1887  |    1871 | building 1.80
steam_sawmill          1825 |  1821  1823  1812..1833  | not built | lumber 9.05
cement_works           1850 |  1853  1849  1837..1863  |    1851 | lime 8.88
steam_brewery          1785 |  1789  1790  1777..1800  |    1793 | coal 32.52
pottery_works          1770 |  1777  1779  1769..1789  |    1783 | coal 92.91
gasworks               1815 |  1813  1812  1798..1824  |    1812 | coal 9.29
edge_tool_works        1778 |  1769  1774  1769..1782  |   (run) | tools 36.52
tool_factory           1864 |  1865  1863  1848..1875  |   (run) | steel 5.06
hot_blast              1830 |  1827  1829  1817..1839  |   (run) | coal 6.50
machine_shop           1838 |  1836  1837  1826..1847  |   (run) | steel 15.56
engineering_works      1876 |  1880  1880  1866..1890  |   (run) | steel 2.70
continuous_kiln        1856 |  1853  1855  1845..1864  |   (run) | machinery 11.20
emporium               1860 |  1877  1872  1857..1887  |   (run) | gas 1.94
alkali_works           1815 |  1820  1817  1802..1830  |    1818 | coal 9.29
ammonia_soda           1875 | never never  1892..-     | not built | gas 0.65
sheet_glass            1835 |  1843  1842  1828..1854  |    1848 | coal 5.24
tank_furnace           1878 | never never  1896..-     | not built | gas 0.58
machine_paper          1815 |  1821  1818  1803..1831  |    1820 | cloth 25.89
pulp_mill              1885 |  1878  1880  1870..1890  | not built | machinery 2.24
roller_mill            1878 | never  1889  1867..-     |    1893 | flour 0.89
water_wool_mill        1775 |  1771  1775  1767..1784  |   (run) | cloth 129.46
water_frame_mill       1771 |  1780  1782  1773..1792  |    1782 | cottons 562.12
steam_cotton_mill      1805 |  1817  1818  1808..1829  |    1829 | coal 13.55
railway                     |  1825  1826  1815..1837  | not built | rails 8.38
improved_husbandry          |  1774  1776  1766..1786  | not built | tools 24.34
farm_machinery              |  1842  1843  1831..1853  | not built | grain 5.54
deep_mining            1815 |  1812  1813  1802..1825  | not built | coal 9.56
tile_drainage          1843 |  1840  1840  1829..1853  | not built | grain 3.48
artificial_manures     1850 |  1853  1854  1842..1865  | not built | grain 3.05
mine_haulage           1868 |  1865  1861  1845..1875  | not built | coal 2.32
steam_trawling         1882 |  1879  1877  1862..1890  | not built | coal 1.64
ocean_steamers         1866 |  1863  1864  1854..1876  | not built | coal 2.44
```

## D. The same at stage 1 only (`reckon.py 1`)

What the first stage does alone: pottery has the household goods need to itself.

```text
==============================================================================================================
STAGE 1  
budget a head a month: 1750:2.27 1770:2.94 1790:3.81 1810:4.42 1830:5.05 1850:5.70 1870:6.66 1890:7.63 1899:7.96
wage:                  1750:4.05 1770:5.28 1790:6.72 1810:7.48 1830:8.59 1850:9.67 1870:11.89 1890:14.14 1899:14.71

-- price of each new good (cost of the methods making it, weighted)
               1750     1770     1790     1810     1830     1850     1870     1890     1899
building       4.52     5.30     6.63     7.31     8.35     7.89     8.25     3.71     3.24
lumber         4.32     5.01     6.32     7.12     7.56     5.30     4.96     4.14     4.02
lime           4.87     4.02     4.83     5.13     5.60     4.69     2.79     2.15     2.15
beer           2.52     2.65     2.71     1.78     1.78     1.40     0.97     0.82     0.82
pottery        8.96     7.69     4.72     4.00     4.01     3.00     2.65     2.39     2.37
gas            0.00     0.00     0.00     0.00     4.03     3.06     2.72     2.46     2.51

-- units a month, whole island (households, landlords, and running inputs of works)
building       8527    10177    13991    18369    23196    26732    22502   196379   297974
lumber         1933     1938     2364     2901     3906     6893     5518    32598    46364
lime            674      831     1197     1673     2332     2877    26150    25269    29337
beer           3513     5812    10981    27671    41564    85785   210474   388137   454928
pottery         714     1629     5897    11890    18400    45053    98874   177582   210015
gas               0        0        0        0     2898     3548     5218    10854    19053
-- base goods wanted by households and by the works of the new goods (the run's own output beneath)
stone         17516    20878    31456    42907    62136   143211   301537   195695   128666
 (run)        32701    37354    56865    76839   123636   265868   523501   727379   706068
bricks         1149     1779     3433     7965    12529    29717    79690   265767   349777
 (run)         3309     5467    11219    24835    44051    77419   179464   572510   775563
timber        32170    14111    18244    21008    19966    23348    18962    40477    54924
 (run)        38726    27909    34368    35231    32292    25568    19966    14001    14422
grain        180496   210542   313488   423682   533973   759119  1377035  2028125  2256650
 (run)       183002   216570   327288   428326   532858   744891  1318664  1920955  2090102
coal          11890    41593    63667    94625   126190   231619   391887   622140   673032
 (run)        12330    43768    82678   127543   185446   321977   495056   768569   850094
cloth          2908     2989     3105    10872    41785    72751    98077   129037   142460
 (run)         2988     3217     3611    17206    50545    81968   114881   154578   170371
wool          13795    16636    32744    33844     4109      452      100       35       30
 (run)        18620    21688    39650    45415    44239    58250    80166   104815   114203
steel             0        0      341     2396     4171     6845     8821    45646    67746
 (run)            0       16      400     2471     4208     6910    16176    55022    74676
tools           947     1229     1582      740      323     1122     5877    16276    16377
 (run)          939     1251     1772      948      679     2294     7940    19525    23179
services       4047     4720     6826    11965    17776    30019    56513    97486   122922
 (run)         5533     6404     9028    16527    25050    43010    84518   150643   192254
clay           1768     3189     8629    14439    20489    46061    96121   153775   177141
 (run)         4880     7889    15434    32104    54808    94193   210777   586278   743721

-- who serves each need (share of the need's service), and the need's share of household spending
food        spend:      41%      44%      35%      32%      31%      26%      16%      13%      14%
   grain              100%     100%     100%     100%     100%     100%     100%     100%     100%
warmth      spend:      21%      11%      12%      11%      11%      10%       9%       8%       8%
   timber              44%       0%       0%       0%       0%       0%       0%       0%       0%
   coal                55%     100%     100%     100%      96%      97%      98%      98%      96%
   charcoal             1%       0%       0%       0%       0%       0%       0%       0%       0%
   gas                  0%       0%       0%       0%       4%       3%       2%       2%       4%
shelter     spend:      11%      13%      16%      17%      18%      19%      20%      19%      19%
   timber               6%      11%      10%       8%       4%       2%       1%       0%       0%
   stone               15%      14%      16%      17%      20%      36%      44%      10%       5%
   bricks               2%       2%       3%       7%       9%      15%      25%      18%      16%
   building            77%      72%      70%      68%      67%      46%      23%      71%      79%
   lime                 0%       0%       0%       0%       0%       0%       6%       1%       0%
clothing    spend:      17%      20%      19%      18%      16%      16%      18%      18%      18%
   wool                62%      66%      79%      52%       3%       0%       0%       0%       0%
   cloth               38%      34%      21%      48%      97%     100%     100%     100%     100%
implements  spend:       4%       5%       5%       5%       5%       6%       6%       6%       6%
   steel                0%       0%      39%      91%      97%      92%      62%      85%      89%
   tools              100%     100%      61%       9%       3%       5%      14%      10%       7%
   machinery            0%       0%       0%       0%       0%       2%      24%       5%       4%
comfort     spend:       3%       4%       7%       9%      11%      14%      19%      21%      21%
   services           100%     100%     100%     100%     100%     100%     100%     100%     100%
table       spend:       2%       3%       4%       4%       4%       5%       6%       6%       6%
   grain                2%       0%       2%       0%       0%       0%       0%       0%       0%
   beer                98%     100%      98%     100%     100%     100%     100%     100%     100%
wares       spend:       1%       1%       2%       3%       4%       5%       7%       8%       8%
   timber               5%       2%       0%       0%       0%       0%       0%       0%       0%
   lumber              17%       3%       0%       0%       0%       0%       0%       0%       0%
   pottery             77%      95%     100%     100%     100%     100%     100%     100%     100%

-- method shares of each new good's output
   building   masons                  58%      60%      61%      60%      56%      44%      42%       0%       0%
   building   carpenters              32%      28%      26%      23%      25%      34%      29%       0%       0%
   building   bricklayers             10%      12%      13%      17%      19%      22%      29%       0%       0%
   building   contract_builders        0%       0%       0%       0%       0%       0%       0%     100%     100%
   lumber     saw_pit                100%     100%     100%     100%      40%       3%       1%       0%       0%
   lumber     steam_sawmill            0%       0%       0%       0%      60%      97%      99%     100%     100%
   lime       lime_kiln              100%     100%     100%     100%     100%     100%       4%       1%       1%
   lime       cement_works             0%       0%       0%       0%       0%       0%      96%      99%      99%
   beer       brewhouse              100%     100%     100%       4%       1%       0%       0%       0%       0%
   beer       steam_brewery            0%       0%       0%      96%      99%     100%     100%     100%     100%
   pottery    potbank                100%     100%       1%       0%       0%       0%       0%       0%       0%
   pottery    pottery_works            0%       0%      99%     100%     100%     100%     100%     100%     100%
   gas        gasworks                 0%       0%       0%       0%     100%     100%     100%     100%     100%

-- hands in the new trades (labour of the works making each new good), and all hands at work
building       3211     3831     5276     6945     8754     9933     8404    20384    25379
lumber          848      850     1036     1272     1100      605      402     1987     2554
lime             84      104      150      210      292      360     1172      844      929
beer            550      910     1719     1050     1131     1869     3834     6032     6654
pottery         559     1275     1256     1673     2084     4247     7627    11354    12483
gas               0        0        0        0      271      283      376      722     1214
all hands     67500    74323    86935   110058   140256   179119   230562   297467   333298

-- discovery (median year by the engine's hazard; 'met' = year the last threshold is reached)
invention               aim |   met found   10%..90%   | entered | the threshold that binds, and progress in 1899
coke_furnace                |  1772  1774  1766..1784  |   (run) | coal 92.91
textile_mill                |  1787  1785  1773..1798  |   (run) | cloth 71.21
engine_works                |  1782  1784  1775..1793  |   (run) | iron 112.29
crucible_steel              |  1764  1768  1761..1776  |   (run) | iron 187.15
bessemer               1856 |  1854  1853  1841..1865  |   (run) | steel 7.22
rolling_mill                |  1797  1798  1786..1808  |   (run) | rails 20.95
steel_rolling          1866 |  1870  1869  1853..1881  |   (run) | rails 2.25
contract_builders      1862 |  1885  1870  1849..1890  |    1873 | building 2.09
steam_sawmill          1825 |  1821  1823  1812..1835  |    1825 | lumber 6.64
cement_works           1850 |  1849  1849  1837..1861  |    1851 | lime 8.35
steam_brewery          1785 |  1790  1790  1778..1801  |    1794 | coal 32.52
pottery_works          1770 |  1768  1771  1764..1779  |    1775 | coal 92.91
gasworks               1815 |  1813  1812  1798..1824  |    1812 | coal 9.29
edge_tool_works        1778 |  1769  1774  1769..1782  |   (run) | tools 36.52
tool_factory           1864 |  1865  1863  1848..1875  |   (run) | steel 5.06
hot_blast              1830 |  1827  1829  1817..1839  |   (run) | coal 6.50
machine_shop           1838 |  1836  1837  1826..1847  |   (run) | steel 15.56
engineering_works      1876 |  1880  1880  1866..1890  |   (run) | steel 2.70
continuous_kiln        1856 |  1853  1855  1845..1864  |   (run) | machinery 11.20
emporium               1860 |  1877  1873  1858..1887  |   (run) | gas 2.00
railway                     |  1825  1826  1815..1837  | not built | rails 8.38
improved_husbandry          |  1774  1776  1766..1786  | not built | tools 24.34
farm_machinery              |  1842  1843  1831..1853  | not built | grain 5.54
```

## E. Variants: what each fix does (`tweaks.py`)

Stage 2 content.

```text
years shown: [1770, 1810, 1850, 1870, 1890, 1899]
as drafted                                                
    builders' share of shelter    72%  68%  47%  23%  64%  74%    builders' share of all hands  5.2%  6.3%  5.6%  3.7%  6.2%  7.0%
    stone dug, against the run's  71%  71%  70%  75%  51%  42%    (households, builders, kilns, and 15%% of the run's taken as plant)
    gas's share of fuel and light   0%   0%   3%   2%   2%   4%   gas made by 1899: 6.0M
    found: contract builders 1870, shops 1873, ammonia-soda never, tank furnace never, steam sawmill 1823, cement 1849
as drafted, stone's price following what is dug           
    builders' share of shelter    71%  66%  43%  21%  53%  65%    builders' share of all hands  5.1%  6.3%  5.3%  3.5%  5.3%  6.2%
    stone dug, against the run's  82%  82%  81%  84%  72%  65%    (households, builders, kilns, and 15%% of the run's taken as plant)
    gas's share of fuel and light   0%   0%   3%   2%   2%   4%   gas made by 1899: 6.1M
    found: contract builders 1871, shops 1872, ammonia-soda never, tank furnace never, steam sawmill 1823, cement 1849
gas at 5.0 as fuel and light (3.5 drafted)                
    builders' share of shelter    72%  68%  47%  23%  65%  75%    builders' share of all hands  5.2%  6.3%  5.6%  3.7%  6.3%  7.2%
    stone dug, against the run's  71%  71%  71%  75%  51%  43%    (households, builders, kilns, and 15%% of the run's taken as plant)
    gas's share of fuel and light   0%   0%  33%  33%  34%  42%   gas made by 1899: 44.8M
    found: contract builders 1870, shops 1864, ammonia-soda 1873, tank furnace 1879, steam sawmill 1823, cement 1848
gas at 6.0                                                
    builders' share of shelter    72%  68%  47%  23%  65%  75%    builders' share of all hands  5.2%  6.3%  5.6%  3.7%  6.3%  7.2%
    stone dug, against the run's  71%  71%  71%  75%  51%  43%    (households, builders, kilns, and 15%% of the run's taken as plant)
    gas's share of fuel and light   0%   0%  54%  54%  56%  64%   gas made by 1899: 64.8M
    found: contract builders 1870, shops 1864, ammonia-soda 1873, tank furnace 1879, steam sawmill 1823, cement 1848
raw shelter re-rated in the overlay: stone 0.3, bricks 0.6, timber 0.4, sawn 0.6, lime 0.5
    builders' share of shelter    99%  99%  95%  95% 100% 100%    builders' share of all hands  7.7% 10.1% 11.9% 11.1%  9.5%  9.6%
    stone dug, against the run's  52%  51%  39%  32%  24%  25%    (households, builders, kilns, and 15%% of the run's taken as plant)
    gas's share of fuel and light   0%   0%   3%   2%   2%   4%   gas made by 1899: 6.0M
    found: contract builders 1859, shops 1872, ammonia-soda never, tank furnace never, steam sawmill 1823, cement 1848
the same, stone's price following what is dug             
    builders' share of shelter    98%  98%  88%  83%  98%  99%    builders' share of all hands  8.0% 10.6% 11.1% 10.1%  9.2%  9.5%
    stone dug, against the run's  61%  61%  56%  55%  34%  32%    (households, builders, kilns, and 15%% of the run's taken as plant)
    gas's share of fuel and light   0%   0%   3%   2%   2%   4%   gas made by 1899: 6.3M
    found: contract builders 1859, shops 1871, ammonia-soda never, tank furnace never, steam sawmill 1823, cement 1848
thresholds refitted: contract builders building 12M; shops gas 1.2M; ammonia-soda and tank furnace without gas
    builders' share of shelter    72%  68%  47%  43%  79%  83%    builders' share of all hands  5.2%  6.3%  5.6%  4.8%  7.2%  7.6%
    stone dug, against the run's  71%  71%  70%  65%  41%  37%    (households, builders, kilns, and 15%% of the run's taken as plant)
    gas's share of fuel and light   0%   0%   3%   2%   2%   4%   gas made by 1899: 6.0M
    found: contract builders 1859, shops 1864, ammonia-soda 1869, tank furnace 1873, steam sawmill 1823, cement 1849
all three together (shops' gas threshold back at 3M)      
    builders' share of shelter    99%  99%  95%  95% 100% 100%    builders' share of all hands  7.7% 10.1% 11.9% 11.1%  9.6%  9.7%
    stone dug, against the run's  52%  51%  40%  33%  25%  27%    (households, builders, kilns, and 15%% of the run's taken as plant)
    gas's share of fuel and light   0%   0%  33%  32%  34%  42%   gas made by 1899: 44.6M
    found: contract builders 1859, shops 1864, ammonia-soda 1869, tank furnace 1873, steam sawmill 1823, cement 1848
all three together, stone's price following what is dug   
    builders' share of shelter    98%  98%  88%  83%  98%  99%    builders' share of all hands  8.0% 10.6% 11.1% 10.1%  9.4%  9.7%
    stone dug, against the run's  61%  61%  57%  55%  34%  33%    (households, builders, kilns, and 15%% of the run's taken as plant)
    gas's share of fuel and light   0%   0%  34%  33%  34%  42%   gas made by 1899: 45.4M
    found: contract builders 1859, shops 1864, ammonia-soda 1869, tank furnace 1873, steam sawmill 1823, cement 1848
```

## F. Each invention in the year it is found (`atfound.py`)

Entry test and the size of its smallest works against the market, as drafted.

```text
gas at 4.5: share of fuel and light ['1830 23%', '1850 20%', '1870 20%', '1890 21%', '1899 28%'] made by 1899 31.2M shops 1864 ammonia 1873 tank 1879

invention          makes       aim found |    going | need:0.25      0.6      1.0 |   ratio                 |   island  0.4 K0  share  towns
contract_builders  building   1862  1866 |     8.34 |     9.68     8.30     7.26 |    1.16    0.99    0.87 |    22924     320     1%  20
steam_sawmill      lumber     1825  1823 |     5.37 |     8.22     7.25     6.51 |    1.53    1.35    1.21 |     8509     240     3%  17
cement_works       lime       1850  1849 |     4.75 |     4.92     4.34     3.89 |    1.04    0.91    0.82 |     2766     400    14%  19
steam_brewery      beer       1785  1790 |     2.71 |     2.92     2.58     2.32 |    1.08    0.95    0.86 |    11176     600     5%  12
pottery_works      pottery    1770  1779 |     8.72 |     9.72     8.21     7.09 |    1.12    0.94    0.81 |      321     160    50%  11   <- one smallest works is over a fifth of the island's whole market
gasworks           gas        1815  1812 |     4.92 |     5.78     5.09     4.56 |    1.17    1.03    0.93 |      274     200    73%  16   <- one smallest works is over a fifth of the island's whole market
alkali_works       chemicals  1815  1817 |    13.96 |    14.59    12.49    10.92 |    1.05    0.89    0.78 |      913     120    13%  16
ammonia_soda       chemicals  1875 never
sheet_glass        glass      1835  1842 |    28.13 |    34.45    28.64    24.38 |    1.22    1.02    0.87 |      770     120    16%  18
tank_furnace       glass      1878 never
machine_paper      paper      1815  1818 |    15.69 |    16.49    13.80    11.83 |    1.05    0.88    0.75 |      727     160    22%  16   <- one smallest works is over a fifth of the island's whole market
pulp_mill          paper      1885  1880 |     3.55 |     5.59     4.88     4.34 |    1.57    1.37    1.22 |    21863     800     4%  22
roller_mill        flour      1878  1889 |     0.62 |     0.66     0.60     0.54 |    1.08    0.97    0.88 |   171248    1200     1%  23
water_frame_mill   cottons    1771  1782 |    15.65 |    15.24    13.11    11.53 |    0.97    0.84    0.74 |      869      48     6%  11
steam_cotton_mill  cottons    1805  1818 |     6.83 |     8.28     7.44     6.79 |    1.21    1.09    0.99 |    13849     400     3%  16

1750 methods of new goods: the island's market in 1750 and 1770 against the smallest works, and works at the usual size it would hold
  masons           building  K0    300 | island 1750     5041 ( 16.8 usual sizes,  2.8 a town)  1770     4940 ( 16.5,  1.7 a town)
  carpenters       building  K0    200 | island 1750     3700 ( 18.5 usual sizes,  3.1 a town)  1770     5805 ( 29.0,  2.9 a town)
  bricklayers      building  K0    300 | island 1750      930 (  3.1 usual sizes,  0.5 a town)  1770     1326 (  4.4,  0.4 a town)
  saw_pit          lumber    K0     60 | island 1750      224 (  3.7 usual sizes,  0.6 a town)  1770      296 (  4.9,  0.5 a town)
  lime_kiln        lime      K0    100 | island 1750      764 (  7.6 usual sizes,  1.3 a town)  1770      707 (  7.1,  0.7 a town)
  brewhouse        beer      K0    300 | island 1750     3656 ( 12.2 usual sizes,  2.0 a town)  1770     6007 ( 20.0,  2.0 a town)
  potbank          pottery   K0     40 | island 1750      174 (  4.4 usual sizes,  0.7 a town)  1770      285 (  7.1,  0.7 a town)
  soap_boilers     chemicals K0     40 | island 1750      175 (  4.4 usual sizes,  0.7 a town)  1770      263 (  6.6,  0.7 a town)
  glasshouse       glass     K0     30 | island 1750       67 (  2.2 usual sizes,  0.4 a town)  1770      111 (  3.7,  0.4 a town)
  paper_mill       paper     K0     30 | island 1750      146 (  4.9 usual sizes,  0.8 a town)  1770      192 (  6.4,  0.6 a town)
  windmill         flour     K0    800 | island 1750    16893 ( 21.1 usual sizes,  3.5 a town)  1770    19715 ( 24.6,  2.5 a town)
  watermill        flour     K0   1200 | island 1750    18602 ( 15.5 usual sizes,  2.6 a town)  1770    21773 ( 18.1,  1.8 a town)
  water_sawmill    lumber    K0    200 | island 1750     2059 ( 10.3 usual sizes,  1.7 a town)  1770     4046 ( 20.2,  2.0 a town)
  fustian_weaving  cottons   K0     60 | island 1750      269 (  4.5 usual sizes,  0.7 a town)  1770      908 ( 15.1,  1.5 a town)
```

## G. Seed-to-seed spread of 'xN' thresholds (`seeds.py`)

The designer's sixteen runs under the game's rules.

```text
1750 monthly output by seed (the 'x1' rate is twelve times this; the engine uses the warm-up decade's mean, which is close):
  coal    least    11024  median    11937  greatest    16540   greatest/least 1.5
  iron    least      443  median      531  greatest      572   greatest/least 1.3
  grain   least   168365  median   181732  greatest   197130   greatest/least 1.2
  cloth   least     2600  median     3057  greatest     3128   greatest/least 1.2
  tools   least      829  median      938  greatest     1035   greatest/least 1.2
  bricks  least     2969  median     3469  greatest     4496   greatest/least 1.5
  stone   least    23752  median    32232  greatest    36072   greatest/least 1.5

year the threshold is met, by seed: least, median, greatest (spread)   [the discovery itself then falls about 0 to 10 years either side]
  canal                    coal   x18     1760  1761  1764   (4 years)
  canal                    stone  x13     1761  1762  1763   (2 years)
  pottery_works            coal   x35     1765  1767  1771   (6 years)
  steam_brewery            coal   x100    1779  1782  1793   (14 years)
  steam_cotton_mill        coal   x240    1795  1800  1816   (21 years)
  deep_mining              coal   x340    1803  1810  1828   (25 years)
  gasworks / alkali_works  coal   x350    1804  1811  1829   (25 years)
  gasworks                 iron   x185    1806  1813  1819   (13 years)
  hot_blast                coal   x500    1814  1823  1842   (28 years)
  hot_blast                iron   x270    1818  1824  1830   (12 years)
  sheet_glass              coal   x620    1821  1831  1850   (29 years)
  ocean_steamers           coal   x1330   1848  1860  1880   (32 years)
  mine_haulage             coal   x1400   1850  1862  1882   (32 years)
  steam_trawling           coal   x1980   1864  1875  1895   (31 years)
  waterworks               iron   x490    1836  1844  1849   (13 years)
  iron_shipyard            iron   x450    1833  1841  1846   (13 years)
  machine_paper            cloth  x110    1799  1815  1832   (33 years)
  water_wool_mill          cloth  x22     1769  1771  1772   (3 years)
  textile_mill (override)  cloth  x50     1788  1795  1806   (18 years)
  embanking                grain  x47     1785  1788  1791   (6 years)
  tile_drainage            grain  x175    1835  1840  1844   (9 years)
  artificial_manures       grain  x200    1841  1847  1851   (10 years)
  continuous_kiln          bricks x300    1817  1822  1831   (14 years)
  edge_tool_works          tools  x20     1765  1766  1772   (7 years)

the same goods with a threshold in plain units (the median seed's xN turned into a number):
  canal                    coal        2578392  1759  1761  1764   (5 years)
  canal                    stone       5028254  1760  1762  1765   (5 years)
  pottery_works            coal        5013540  1764  1768  1771   (7 years)
  steam_brewery            coal       14324400  1777  1781  1787   (10 years)
  steam_cotton_mill        coal       34378560  1796  1800  1809   (13 years)
  deep_mining              coal       48702960  1805  1809  1820   (15 years)
  gasworks / alkali_works  coal       50135400  1806  1810  1821   (15 years)
  gasworks                 iron        1177834  1808  1813  1819   (11 years)
  hot_blast                coal       71622000  1817  1822  1833   (16 years)
  hot_blast                iron        1719001  1820  1825  1831   (11 years)
  sheet_glass              coal       88811280  1824  1830  1841   (17 years)
  ocean_steamers           coal      190514520  1851  1859  1875   (24 years)
  mine_haulage             coal      200541600  1853  1861  1878   (25 years)
  steam_trawling           coal      283623120  1866  1875  1892   (26 years)
  waterworks               iron        3119669  1839  1844  1850   (11 years)
  iron_shipyard            iron        2865002  1836  1841  1847   (11 years)
  machine_paper            cloth       4035623  1799  1815  1833   (34 years)
  water_wool_mill          cloth        807125  1769  1771  1775   (6 years)
  textile_mill (override)  cloth       1834374  1788  1797  1807   (19 years)
  embanking                grain     102496848  1785  1788  1791   (6 years)
  tile_drainage            grain     381637200  1836  1838  1844   (8 years)
  artificial_manures       grain     436156800  1843  1846  1851   (8 years)
  continuous_kiln          bricks     12487392  1815  1823  1834   (19 years)
  edge_tool_works          tools        225129  1765  1767  1772   (7 years)
```

## H. Chains (`chains.py`)

Makers, buyers, later-stage references, deadlocks.

```text
-- references to a good of a later stage than the entry (the loader drops them until that stage)
   contract_builders  stage 1  inputs   glass     (stage 2)
   edge_tool_works    stage 1  site water_power is read from stage 3
   emporium           stage 1  inputs   paper     (stage 2)
   alkali_works       stage 2  site salt is read from stage 3
   ammonia_soda       stage 2  site salt is read from stage 3
   paper_mill         stage 2  site water_power is read from stage 3

-- every good: makers in 1750, later makers, and every buyer (need, running input, plant, improvement)
building   stage 1 | 1750: masons, carpenters, bricklayers | later: contract_builders (1862)
                    | needs: shelter 3 | inputs of: - | plant of: - | improvements: - | gates: contract_builders
lumber     stage 1 | 1750: saw_pit, water_sawmill | later: steam_sawmill (1825)
                    | needs: shelter 0.9, wares 0.35 | inputs of: carpenters, bricklayers, contract_builders, shipyard, iron_shipyard, steel_shipyard | plant of: contract_builders, steam_brewery, gasworks, windmill, watermill, water_wool_mill, water_frame_mill, shipyard | improvements: deep_mining, fishing_smacks, embanking | gates: steam_sawmill
lime       stage 1 | 1750: lime_kiln | later: cement_works (1850)
                    | needs: shelter 0.8 | inputs of: masons, bricklayers, contract_builders, gasworks, ammonia_soda, sheet_glass, tank_furnace | plant of: - | improvements: liming | gates: cement_works
beer       stage 1 | 1750: brewhouse | later: steam_brewery (1785)
                    | needs: table 1 | inputs of: - | plant of: - | improvements: - | gates: steam_brewery
pottery    stage 1 | 1750: potbank | later: pottery_works (1770)
                    | needs: wares 1 | inputs of: - | plant of: - | improvements: - | gates: pottery_works
gas        stage 1 | 1750: NOBODY | later: gasworks (1815)
                    | needs: warmth 3.5 | inputs of: emporium | plant of: - | improvements: - | gates: emporium, ammonia_soda, tank_furnace
chemicals  stage 2 | 1750: soap_boilers | later: alkali_works (1815), ammonia_soda (1875)
                    | needs: wares 1.3 | inputs of: glasshouse, sheet_glass, tank_furnace, machine_paper, pulp_mill, steam_cotton_mill | plant of: - | improvements: artificial_manures | gates: alkali_works, ammonia_soda, sheet_glass, pulp_mill, artificial_manures
glass      stage 2 | 1750: glasshouse | later: sheet_glass (1835), tank_furnace (1878)
                    | needs: wares 2.5 | inputs of: contract_builders | plant of: steam_cotton_mill | improvements: - | gates: sheet_glass, tank_furnace
paper      stage 2 | 1750: paper_mill | later: machine_paper (1815), pulp_mill (1885)
                    | needs: wares 1.3 | inputs of: emporium | plant of: - | improvements: - | gates: machine_paper, pulp_mill
flour      stage 3 | 1750: windmill, watermill | later: roller_mill (1878)
                    | needs: food 1.5 | inputs of: - | plant of: - | improvements: - | gates: roller_mill
fish       stage 3 | 1750: land:fishery | later: -
                    | needs: food 1.3 | inputs of: - | plant of: - | improvements: - | gates: steam_trawling
produce    stage 3 | 1750: land:garden | later: -
                    | needs: food 1.8 | inputs of: - | plant of: - | improvements: - | gates: -
cotton     stage 5 | 1750: NOBODY | later: -
                    | needs: none | inputs of: fustian_weaving, water_frame_mill, steam_cotton_mill | plant of: - | improvements: - | gates: -
cottons    stage 5 | 1750: fustian_weaving | later: water_frame_mill (1771), steam_cotton_mill (1805)
                    | needs: clothing 0.85 | inputs of: - | plant of: - | improvements: - | gates: water_frame_mill, steam_cotton_mill
ships      stage 6 | 1750: shipyard | later: iron_shipyard (1845), steel_shipyard (1882)
                    | needs: none | inputs of: - | plant of: - | improvements: - | gates: iron_shipyard, steel_shipyard

-- an invention that needs (as input, plant or threshold) a good nobody can make by the year it is aimed at
   fustian_weaving    aimed 1750: inputs   cotton    first made never (no maker in the tree: needs trade)
   water_frame_mill   aimed 1771: inputs   cotton    first made never (no maker in the tree: needs trade)
   steam_cotton_mill  aimed 1805: inputs   cotton    first made never (no maker in the tree: needs trade)

-- plant that includes the works' own output or a good first made by the same works
   edge_tool_works    plant holds tools 2 a unit; earlier makers: smithy
   machine_shop       plant holds machinery 0.6 a unit; earlier makers: millwright, engine_works
   engineering_works  plant holds machinery 1.5 a unit; earlier makers: millwright, engine_works, machine_shop
   continuous_kiln    plant holds bricks 8 a unit; earlier makers: brickworks

-- thresholds in a good whose only makers are the invention's own successors or itself

-- counts
   stage 0: 16 goods, 6 needs, 7 sectors, 16 methods, 5 other inventions
   stage 1: 22 goods, 8 needs, 7 sectors, 36 methods, 5 other inventions
   stage 2: 25 goods, 8 needs, 7 sectors, 44 methods, 5 other inventions
   stage 3: 28 goods, 8 needs, 9 sectors, 50 methods, 5 other inventions
   stage 5: 30 goods, 8 needs, 9 sectors, 53 methods, 13 other inventions
   stage 6: 31 goods, 8 needs, 9 sectors, 56 methods, 13 other inventions
```

## I. With every proposed number (`verify.py`)

Full content (stage 5) with the numbers of section 14.

```text
as drafted: flour made by 1868 112M, 1873 121M, 1899 178M; cottons by 1766 95944, 1771 147667, 1800 1.45M, 1805 2.16M; pottery by 1765 41220 1770 57755
fish 1.1, produce 1.4, flour 1.5
   1770 food: grain 28%, flour 44%, fish 13%, produce 15%; grain wanted 137257 against the run's 216570 (63%)
   1850 food: grain 31%, flour 38%, fish 14%, produce 17%; grain wanted 492083 against the run's 744891 (66%)
   1890 food: grain 26%, flour 50%, fish 11%, produce 13%; grain wanted 1291154 against the run's 1920955 (67%)
   flour made by 1873 175M, by 1899 318M; roller mill found 1877
fish 1.1, produce 1.4, flour 1.3
   1770 food: grain 35%, flour 29%, fish 17%, produce 20%; grain wanted 129315 against the run's 216570 (60%)
   1850 food: grain 36%, flour 25%, fish 18%, produce 21%; grain wanted 472681 against the run's 744891 (63%)
   1890 food: grain 42%, flour 15%, fish 20%, produce 24%; grain wanted 1165728 against the run's 1920955 (61%)
   flour made by 1873 129M, by 1899 197M; roller mill found 1886

with every proposed number (section 14), stage 5 content:
invention              aim found     10%..90%  entered
textile_mill          1798  1796  1785..1807     (run)
engine_works                1784  1775..1793     (run)
crucible_steel              1768  1761..1776     (run)
bessemer              1856  1853  1841..1865     (run)
steel_rolling         1866  1869  1853..1881     (run)
contract_builders     1862  1859  1849..1869      1865
steam_sawmill         1825  1823  1812..1833      1828
cement_works          1850  1848  1837..1858      1850
steam_brewery         1785  1791  1778..1801      1795
pottery_works         1770  1771  1764..1779      1775
gasworks              1815  1812  1798..1824      1812
edge_tool_works       1778  1774  1769..1782     (run)
tool_factory          1864  1863  1848..1875     (run)
hot_blast             1830  1829  1817..1839     (run)
machine_shop          1838  1837  1826..1847     (run)
engineering_works     1876  1880  1866..1890     (run)
continuous_kiln       1856  1855  1845..1864     (run)
emporium              1860  1864  1844..1878     (run)
alkali_works          1815  1818  1803..1830      1819
ammonia_soda          1875  1869  1852..1881      1873
sheet_glass           1835  1842  1828..1854      1848
tank_furnace          1878  1873  1857..1884      1874
machine_paper         1815  1819  1803..1831      1821
pulp_mill             1885  1880  1870..1890  not built
roller_mill           1878  1877  1866..1891      1882
water_wool_mill       1775  1775  1767..1784     (run)
water_frame_mill      1771  1771  1764..1779      1771
steam_cotton_mill     1805  1803  1792..1814      1813
deep_mining           1815  1813  1802..1825          
tile_drainage         1843  1840  1829..1853          
artificial_manures    1850  1854  1841..1865          
mine_haulage          1868  1861  1845..1875          
steam_trawling        1882  1877  1862..1890          
ocean_steamers        1866  1864  1854..1876          
years [1770, 1810, 1850, 1870, 1890, 1899]
   shelter   building    99%   99%   96%   94%  100%  100%
   shelter   stone        0%    1%    4%    4%    0%    0%
   shelter   bricks       0%    0%    1%    2%    0%    0%
   warmth    gas          0%    0%   21%   20%   21%   28%
   warmth    coal       100%  100%   79%   80%   79%   72%
   clothing  wool        51%   34%    2%    0%    0%    0%
   clothing  cloth       36%   33%   45%   30%    9%    5%
   clothing  cottons     13%   33%   53%   70%   91%   95%
   food      grain       29%   30%   30%   34%   26%   19%
   food      flour       43%   40%   39%   30%   50%   65%
   food      fish        13%   14%   14%   17%   11%    7%
   food      produce     15%   16%   17%   19%   13%    9%
   lumber by saw_pit          24%   23%    1%    0%    0%    0%
   lumber by water_sawmill    76%   77%    5%    1%    0%    0%
   lumber by steam_sawmill     0%    0%   94%   98%  100%  100%
   hands in all the new trades, share of all hands: 15.4% 18.5% 20.1% 21.3% 22.0% 22.9%
```

