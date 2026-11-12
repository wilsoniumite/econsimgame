# The game's production tree, 1750 to 1900: the one design

Night of 11 November 2026. Three designers answered the brief from three sides: history first (`content-design-history.md`), map first (`content-design-map.md`), engine first (`content-design-engine.md`). This is the design to build, revised after its critique (`content-design-critique.md`): section 16 says what the critique changed and what is still uncertain, and section 17 is for whoever builds it. It says what is taken from each design, how their disagreements are settled, and what Stella's later words add: the building trades, logging, water, canals, and a tree that has to carry on to 2050. The overlay itself is `game-content-draft.toml`, beside this file.

Nothing in the repository was changed and cargo was not run. Three kinds of evidence are kept apart throughout:

- **Run under the game's rules tonight.** The methods this design puts on the sixteen base goods (edge-tool works, tool factory, hot blast, machine-tool works, engineering works, continuous kiln, shops and offices), with its changes to base methods, were run with a copy of the binary built from `night/core` this afternoon, every mechanism of the game on: sixteen seeds, eight used to set the thresholds and eight held out, against the game's own content on the same seeds (section 11). That binary cannot load a seventeenth good, so nothing with a new good was run.
- **Reckoned at the game's prices.** Everything on a new good: unit sums with the engine's own formulas (`firm_coefficients`, `heat_mix`, `service_mix`, the linear expenditure system with the Engel rule, the hurdle on plant, the discovery hazard), with population, the wage and the sixteen base goods' prices and outputs taken year by year from those runs: a wage of 4.0 rising to 14.7 while coal, stone and bricks fall to 0.9, 0.9 and 1.5. This is the critic's reckoning (`.claude/night/tmp/critique/`), rerun on the revised overlay by the scripts in `.claude/night/tmp/revise/`, which also print the overlay and every table here, so the three agree. The first draft reckoned at the port baseline's prices (a wage of 4.6 to 8.1, coal at 1.5 to 1.9), and most of what the critique found follows from that. It is still a reckoning: it has no towns, it prices a good at the cost of making it, and it holds the base goods' prices at the run's.
- **Asserted.** The engine changes: specified against the code, not built.

## 1. In one page

**What is added.** 14 goods, making 30, and ships as a thirty-first if fleets are ever built. 2 needs, making 8. 2 land sectors, making 9. 40 methods, making 56 (53 without the three shipyards), and six base methods touched. 10 inventions that are not works, making 15.

| stage | engine change it needs | goods it adds | what arrives |
|---|---|---|---|
| 1 | loading an overlay, with decay and opening stock per good; a theta for each need; `group` on a method | building work, sawn timber, lime, beer, pottery, town gas, soap and chemicals, glass, paper (25) | the building trades; sawmills; breweries, potteries, gasworks; household goods as a need with four living trades and their alkali, glass and paper generations; the later generations of every base chain (edge tools, hot blast, machine tools, kilns, tool factories, engineering works, shops and offices) |
| 2 | sites on methods (water power with a capacity, wind, a harbour) from the new island's fields; land uses for more than seven sectors; log driving | flour, fish, dairy and garden produce (28) | wind and water mills, water-powered cloth and saw mills, the garden ring, fishing; the move from streams to coal; the pulp mill at the log boom |
| 3 | foreign trade at the ports | raw cotton, cotton cloth (30) | the cotton chain; grain, wool and timber from abroad; exports |
| 4 | improvements as data (the machines rule made general) | none | liming, drainage, manures, deep mining with pit props, haulage, fishing smacks and trawlers |
| 5 | fleets (and railway upkeep) | ships (31) | shipyards in timber, iron and steel; sleepers and rails bought every year |
| with their own mechanisms | canals (being built, with their own clock); reservoirs and floods | none | canals, embanking, town waterworks |

The stages are numbered in the order they are to be built, so the loader needs only a ceiling: stage N is every entry with `stage <= N`. (The first draft had six stages to be built in the order 1, 2, 3, 5, 4, 6, which a ceiling cannot express; its stages 1 and 2 are now one, because the first alone gave every spare shilling of "household goods" to potters.)

**The decisions that matter most.**

1. **Shelter is bought as building work.** Masons, bricklayers and timber-frame builders in every town turn stone, brick, timber and lime into houses, and households buy the houses. For the game, raw stone, brick and timber are re-rated as shelter (stone 0.3 where the port has 0.5): a household can still heap up its own wall where builders are short, at a real loss. Without the re-rating builders lose shelter back to heaped stone and kiln brick as wages rise (their share fell to 23 per cent in 1870 in the critic's reckoning); with it they hold 91 to 100 per cent all century. Households and the new trades then want 11,000 units of stone a month in 1770 where the game's own run digs 37,000, and 66,000 in 1890 against 727,000 (plant and railways come on top of both). The hands go into a town trade: builders are 7.5 per cent of all labour in 1770, 12 in 1850 and 10 in 1890. This is stage 1.
2. **The port's fast first wave is kept, and every successor starts as good as its recipe.** The three designers found the same lock from three sides: a method that has learned to the cap of 4 cannot be displaced by one that starts clumsy. History slowed the old curves, engine flattened them, map left them alone and gave successors `A0 = 1` with recipes sized against a learned rival. The runs settle it for map's way: slowing the first curves cost the first wave in both designers' own runs; map's way cost nothing early. Under the game's rules tonight, on sixteen seeds, the seven such methods on base goods are each found within a few years of the year aimed at and built, and the 1900 real wage is 3.4 times 1750's against 2.7 with the game's own content (5.4 against 4.3 on the chained index).
3. **One iron good; steel stays at 3.0; and a tools chain all the same.** No pig iron, no yarn, no salt, no coke: each was one more market with one buyer. Two of the three designs cut steel as an implement from 3.0 to about 1.6, so that households would buy tools and not bar steel. Under the game's rules that breaks the island: no steel is ever sold, the converter is never found, the land's machines never improve, and the 1900 real wage is 1.8 where the game's own content gives 2.7 (section 11). Households' appetite for bar steel is what starts the steel trade, and the machines rule runs on the steel poured. So steel is left alone, and the smithy still gets its two successors: without them tools become a dear, thin input of the machine-tool works (91 a unit in 1890), and with them tools end the century at 3.
4. **Two needs, and gas goes into an old one.** "Drink and groceries" and "household goods" are added; warmth is shown as "fuel and light" and takes gas, rated at 4.5 so that it is a fifth of the need from 1850 (at the first draft's 3.5 it was 3 per cent). The falling share of food is left to the Engel rule the engine now has: the overlay gives its richer set of shares, and nothing is recalibrated.
5. **Water is a field and a set of built things, not a good; sawn timber is a good.** Logging gets a new good with sawmills at the foot of the log drives. Water gets sites whose power is the new island's own, in kilowatts, with a capacity; reservoirs, floods and town waterworks.

**What to do first when it loads.** Refit the thresholds of everything on a new good (section 17). Those on base goods were set under the game's rules and land within a few years of their aim; those on new goods are fitted to the reckoning and have never been run.

## 2. What has changed since the three designs were written

The designs were written against the port. The game now runs under rules the port does not have (`docs/rules/` on `rust-3d` and `night/core`), and each changes what the content should say.

| in the engine now | what it means for the content |
|---|---|
| **engel**: marginal shares lean toward `shares_rich` as income passes twice subsistence | No design needs to raise food subsistence (map) or cut food's share at every income (engine, history). The overlay writes `shares_rich` for eight needs and keeps `shares` near the port's. New consumer goods get a market that grows with income. |
| **machines**: farm and quarry machinery bought where it pays; machines improve with all the steel poured; a learned method inherits half its predecessor's skill; the converter and steel rails made to pay | "Improvements bought where they pay" (engine's first change), the machinery famine and the converter are done. What is left is to make the rule's table of uses come from content (stage 4). Its growth rests on the steel the island pours, most of it bought by households as an implement, which is why steel's efficiency is left at 3.0 (section 11). The inheritance applies only to a method with `A0` below 1 that follows a learned method of the same good: one pair in the overlay is in that case (the port's powered mill once the water-powered woollen mill comes before it: section 8.1), harmlessly, and it goes on serving the converter and the steel rail mill. |
| **works**: a works grows where it stands; some 650 works in 1900, the median six or seven times its usual size; a new works is a tenth of its town's trade; a trade may add 15 per cent of its trade a year and never less than one smallest works | Thin markets hurt less than the designs feared, and a count of "works at K0" means little. But a trade nobody is in yet grows by one smallest works (0.4 K0) a year, so the usual sizes of the 1750 methods of new goods are set large (a fifth to a third of a town's need), not small. The first invention on each new good is the other way about: small, so that its first works fits the island's trade (section 8.1). |
| **carting** takes labour; **railways** branch | A shorter haul now frees hands, so weights matter more, and canals will pay in labour as well as money. |
| **people**: births, deaths and a penalty for town living | Town waterworks have something to act on (section 6.3). |
| **prices**, **harvest** | Floods have a home: a bad year on the flood plain. |
| **mines** (being built): a working has a deposit, a crew and a pit head, and goes deeper | Pit props become a real purchase (stage 4). |
| **canals** (being built); **a new island** with watersheds, lakes, discharge and water power (being built) | The canals mechanism finds canals on its own clock, so the overlay carries no canal invention (section 9). The generator on `night/land` has `power` (kW a cell), `discharge`, `floodplain`, `marsh`, lakes, shores, estuary heads and one prevailing wind. The sites of stage 2 read water power as it stands, in kilowatts; wind, fish and harbour are derived from what it has; it has no salt, and the design no longer asks for any (section 10). |

And Stella has said four things since. The game runs on to about 2050 (section 13, and the naming of needs and goods). She asked by name for logging, for water and for canals before railways (sections 6.2 to 6.4). Much of what the designs proposed as engine changes is built, so the list of engine changes is shorter and in a different order (section 10). And the building trades belong in the first stage (section 6.1).

## 3. Where the three disagreed, and what is settled

| question | history | map | engine | settled | why |
|---|---|---|---|---|---|
| How does a late method get in past a learned one? | re-state the old methods with slow curves; successors at `A0` 0.9 | leave the port's curves; successors at `A0` 1, low recipes, light plant | flatten the old curves; successors at 0.9 to 1 | map's | Under the port's rules, engine's flat curves gave a 1790 wage of 0.96 where the port has 1.24, on two seeds of three, and map's run C (slowed curves) lost the first sixty years. Under the game's rules tonight, map's way finds and builds every successor on sixteen seeds and costs nothing before 1800 (section 11). |
| Pig iron as a good, with finery and puddling? | yes | no | no | no | It puts a second thin market under every forge in 1750 and makes iron a sixth dearer, for a story (pig plentiful, bar dear) that nobody can see on the map. History names dropping it as its own fallback. |
| Yarn | yes | no | no | no | Its author calls it close to "only another input". Spinning and weaving are one mill here. |
| Steel as an implement | 1.4 | 3.0 | 1.6 | 3.0 | Run under the game's rules, eight seeds each. At 1.6 no steel is sold all century, the converter is never built and the 1900 wage is 1.81; at 2.0 the converter is built in three seeds of eight (2.33); at 2.4 in all eight, with a lower and wider wage (3.22, the worst seed 2.65); at 3.0 the wage is 3.38 (the worst seed 3.08). Under the port's rules 1.6 worked, because there a works is paid for output nobody buys. |
| Edge-tool works and tool factory | yes | no | yes | yes | Run under the game's rules: they take tools over in order (half by 1814, then half by 1864), tools output in 1899 is 21,000 a month against 1,300, and tools cost 2.7 against 31. Without them the machine-tool works' pinch of tools comes from a dying trade, tools cost 91 in 1890, and machinery ends the century at 28 where it should be 9. |
| Needs | 7 (wares; beer is food; gas is warmth) | 8 (drink, wares), food subsistence 1.3 | 9 (table, wares, light), grain at 0.8 | 8: table, wares; gas in warmth | The Engel rule does what map's and engine's recalibrations were for. Beer needs a need of its own or food's falling share starves it. A "light" need would be gas alone; warmth relabelled "fuel and light" is the need a twentieth century can use. Grain stays at 1.0 and steel at 3.0; raw stone, brick and timber are re-rated as shelter, for the game only (section 4). |
| The logit's width | softer everywhere (0.4) | 2.0 for wares only | a width for most needs | map's, and clothing at 1.0 once there is cotton | Household goods are unlike each other from the first day; wool and cotton become so when cotton lands (section 4). Everything else stays as tuned. |
| Salt | cut | a good | a site field | neither | Its buyers are one works and a pinch for fish, and the new island has no rock to put it in. The alkali works stand on coal and limestone. |
| Paper | cut | yes | yes | yes | A deep household market, the shops' first input, and the place the logging chain ends (pulp). |
| More food | meat (on pasture) | produce (a garden sector) | neither | produce | A ring of gardens and dairies round every town is the first of von Thünen's rings and reads from any height. Meat and dairy are folded into it. |
| Tea and sugar | a thirty-first good, perhaps | no | yes | left out | No works on the island: it fails the second aim. The "table" need is named so that it can take them later. |
| Roller mill | yes | yes | cut | kept | At the game's wages the old mills' hands and upkeep are a fifth of the price of flour by 1870, and the roller mill saves them: it is built on cost, and drawn to the quay once grain is landed. |
| Open hearth | yes | yes | cut | cut | It passes only on free scrap (history) or returns 9 per cent (map). The engine cannot tell good steel from cheap steel. |
| Third generations | few | many | few | tank furnace, ammonia-soda, pulp; not steam pottery, ring mill, regenerative gasworks, brine works | Each one kept ends a visible trade or moves a works; each one cut only lowered a cost. |
| Machinery as a running input | a trace in plant | a trace in plant | measured: a crash | plant only | Engine's run: cloth output down 85 per cent in a machinery shortage. |
| Ships | yes, thin | yes, thin | yes, thin | thirty-first, conditional | Tens a month, and no buyer without two engine changes. All three say so. |
| The world's prices | paths in world units, floating rate | base price times a level, a band, a money account | world money, a floating exchange, market depth | engine's form, with history's cotton famine | It is the one specified down to the month's arithmetic, and depth is what stops the island becoming one mill. |
| Site rule | must stand there | floor and reference | a weight on a clamped field | for water, must stand there, with a capacity in kilowatts; for wind, engine's weight | Both designs relied on a cell holding only so many works to make the streams fill. Under the life cycle land is counted by hands, a mill has few, and a stream would never fill: the field has to carry a capacity, and the generator's own unit is the honest one. |
| Usual size of the 1750 crafts | small (brewhouse 40) | large (300) | middling (150) | large | See the works rule in section 2. |
| First engine change after the plumbing | Engel, then trade | sites | improvements | sites | Engel and the land's machines are built. Sites are what Stella asked for (water, logging) and what the map lacks. |

## 4. Needs

| need | shown as | subsistence a head | share at subsistence and a bit | share far above it | landlords | theta | goods and their efficiency (new or changed in bold) |
|---|---|---|---|---|---|---|---|
| food | Food | 1.0 | 0.25 (0.28) | 0.05 | 0.03 | 0.25 | grain 1.0, **flour 1.5**, **fish 1.1**, **produce 1.4** |
| warmth | Fuel and light | 0.25 | 0.13 (0.14) | 0.06 | 0.07 | 0.25 | timber 1.0, charcoal 1.8, coal 2.4, **gas 4.5** |
| shelter | Shelter | 0.06 | 0.20 (0.22) | 0.20 | 0.28 | 0.25 | **building 3.0**, **timber 0.4** (0.6), **stone 0.3** (0.5), **bricks 0.6** (1.0), **sawn timber 0.6**, **lime 0.5** |
| clothing | Clothing | 0.03 | 0.17 (0.19) | 0.17 | 0.19 | 0.25; **1.0 from stage 3** | wool 0.35, cloth 1.0, **cotton cloth 0.75** |
| implements | Implements | 0.002 | 0.07 | 0.07 | 0.07 | 0.25 | iron 0.35, tools 1.0, steel 3.0, machinery 3.0 (all as now) |
| comfort | Comfort | 0 | 0.09 (0.10) | 0.28 | 0.24 | 0.25 | services 1.0 |
| table | Drink and groceries | 0 | 0.06 | 0.07 | 0.04 | 0.25 | **beer 1.0**, **grain 0.15** (home brewing) |
| wares | Household goods | 0 | 0.03 | 0.10 | 0.08 | **2.0** | **pottery 1.0**, **glass 2.5**, **paper 1.3**, **soap and chemicals 1.3**, **sawn timber 0.35**, **timber 0.1** (treen) |

The port's shares and efficiencies are in brackets. The choices:

- **Nothing made shares a need with services.** Hands never get cheaper, so anything made that sat in "comfort" would empty the shops. All three designers found this.
- **The same holds for builders, so raw shelter is re-rated.** Half the cost of building work is wages, and it shares shelter with stone and brick, which the game's machines and the overlay's own kiln make cheaper every decade. At the port's efficiencies a unit of shelter in 1870 costs 2.36 as heaped stone and 2.75 from a builder. So for the game the raw goods serve shelter worse: stone 0.3, bricks 0.6, timber 0.4, sawn timber 0.6, lime 0.5. This is the one place the overlay re-rates base goods; the port's file is untouched.
- **Building at 3.0.** A unit of building work holds 1.1 of stone and 0.35 of timber, which heaped up at home would now shelter 0.47. The mason's skill is worth the difference, and it is what takes the weight off the quarries.
- **Gas at 4.5.** A unit is made from 1.2 of coal, which burnt in the grate would give 2.9: light is the point. Gas's price follows coal's, so its share is steep in this number: at the game's prices 3.5 gives 3 per cent of fuel and light, 4.5 gives 20 in 1850 and 21 in 1890, 5.0 gives a third. It wants a sweep of its own (section 17).
- **Comfort's richer share is 0.28, not the Engel rule's 0.36.** The difference goes to the two new needs. Shops and trades are the town's largest employer and this takes eight points from them at high incomes; the builders, who are a town trade this design creates, give more than that back. It is the number in this table most worth a sweep.
- **Implements are left at 0.07 at every income**, as the Engel page asks: households' tools are what make land workers better.
- **Household goods at theta 2.** Pottery, glass, paper and soap are not substitutes. At 0.25 the first to get a factory takes nine tenths of the need (history's and map's first drafts both did this). At 2.0 a good twice as dear per unit of service keeps about the same share of the money. Efficiencies are set so that the four cost the same per unit of service at base prices. All four load with stage 1.
- **Clothing at theta 1.0 once there is cotton.** At 0.25 cotton cloth ends wool: no wool is worn after 1870 and woollen cloth falls to a thirtieth of the run's. At 1.0, with cotton cloth at 0.75, cotton has 47 per cent of clothing in 1850 and 75 in 1890, and woollen cloth keeps a quarter. Before cotton the base's 0.25 stands, as tuned: the overlay carries a `[[services.change]]` at stage 3.
- **Fish 1.1 and produce 1.4**, with each sector's `A` to be tuned until it serves about an eighth of food. (The first draft had 1.3 and 1.8 with price aims that gave them half of food between them and halved the grain trade.)
- **Drink is beer.** With home brewing as the only ceiling on its price, beer is a fixed 6 to 7 per cent of spending above subsistence whatever it costs: 480,000 units a month by 1899 and nearly a quarter of the grain the island buys. That is known and accepted; tea and sugar are the second good the need is named for (section 12).
- **Names for a longer century.** "Table" will take tea, sugar, tobacco and packaged food. "Wares" will take furniture, appliances and plastics. "Fuel and light" will take electricity, oil and gas from the ground. "Implements" already holds machinery at 3.0 (sewing machines, bicycles) and will hold the motor car until travel is a need of its own. "Comfort" is where clerks, teachers, and whatever computing does for a household belong.

Money spent on each need and who serves it, in the reckoning at the game's prices (households and landlords together; stage 3 content, so cotton is in):

**1770**: food 45%, fuel and light 10%, shelter 12%, clothing 21%, implements 4%, comfort 4%, drink and groceries 3%, household goods 1%
- food: flour 42%, grain 33%, produce 13%, fish 12%
- fuel and light: coal 100%
- shelter: building 99%
- clothing: wool 42%, cloth 36%, cottons 22%
- implements: tools 100%
- comfort: services 100%
- drink and groceries: beer 100%
- household goods: chemicals 20%, pottery 19%, glass 19%, paper 17%, lumber 14%, timber 11%
- hands in the new trades: building 5,587, cottons 3,666, lumber 1,141, beer 874, flour 773, paper 285, pottery 212, glass 197, chemicals 176, lime 138; 18% of 74,323 at work

**1850**: food 27%, fuel and light 10%, shelter 18%, clothing 16%, implements 6%, comfort 14%, drink and groceries 5%, household goods 5%
- food: grain 37%, flour 33%, produce 16%, fish 15%
- fuel and light: coal 80%, gas 20%
- shelter: building 96%, stone 4%
- clothing: cottons 47%, cloth 44%, wool 9%
- implements: steel 92%, tools 5%, machinery 2%
- comfort: services 100%
- drink and groceries: beer 100%
- household goods: paper 27%, pottery 24%, chemicals 24%, glass 19%, lumber 6%
- hands in the new trades: building 21,530, cottons 2,307, flour 2,064, beer 1,840, lumber 1,352, gas 1,336, glass 1,270, chemicals 1,008, pottery 1,006, paper 1,002, lime 664; 20% of 179,119 at work

**1890**: food 13%, fuel and light 8%, shelter 19%, clothing 17%, implements 6%, comfort 22%, drink and groceries 6%, household goods 8%
- food: flour 40%, grain 33%, produce 14%, fish 13%
- fuel and light: coal 79%, gas 21%
- shelter: building 100%
- clothing: cottons 75%, cloth 25%
- implements: steel 85%, tools 10%, machinery 5%
- comfort: services 100%
- drink and groceries: beer 100%
- household goods: chemicals 28%, glass 27%, paper 23%, pottery 19%, lumber 3%
- hands in the new trades: building 29,401, cottons 6,682, beer 6,246, pottery 3,343, paper 3,137, glass 2,982, gas 2,921, lumber 2,850, flour 2,082, chemicals 2,050, lime 1,595; 21% of 297,467 at work

## 5. The goods

Every new good: who makes it, who buys it and about how much, where its works stand and what they look like. Quantities are the reckoning's at the game's prices (households, landlords and the works that use the good; new plant, the land's own purchases and exports are left out).

| good | stage | made by (year aimed at) | bought by | a month in 1770 / 1850 / 1899 | where its works stand | from above |
|---|---|---|---|---|---|---|
| `building` Building work | 1 | masons and carpenters, timber-frame builders, bricklayers, contract builders (1862) | households and landlords, as shelter (nearly all of it once raw stone and brick are re-rated) | 14,991 / 58,192 / 467,019 | in every town; the walling is whatever lies nearest: stone by the quarries, brick on the clay, timber frame under the woods | a builder's yard (stone or brick stacks, baulks, a lime pit) and scaffolding on the town's new houses; after 1862 whole streets building at once |
| `lumber` Sawn timber | 1 | saw pit, steam sawmill (1825), water sawmill | builders (two thirds); new works' plant; mines for props and railways for sleepers (stages 4 and 5); furniture; shipwrights | 3,847 / 19,354 / 83,977 | saw pits at the wood's edge and in every town that builds; water sawmills at the first fall below the woods; steam sawmills where logs arrive (river mouths, timber quays, railway yards) | a trestle over a pit; a saw shed on a leat with a log pond held by a boom; a long steam shed with deals stacked in ranks and a sawdust heap |
| `lime` Lime and cement | 1 | lime kiln, cement works (1850) | builders (mortar and plaster); gasworks; glass and soda works; farms once liming is bought (stage 4: about 2,500 a month more in 1770) | 1,103 / 5,303 / 58,043 | under the quarries on the side nearest coal; cement works on limestone by a quay, a canal or a siding | a squat stone drum set into a bank, white spoil below; later a row of bottle kilns and a grinding shed under grey dust |
| `beer` Beer | 1 | brewhouse, common brewery (1785) | households: the whole of drink until tea and sugar | 5,583 / 85,659 / 478,414 | every market town (it is too heavy to cart far); great breweries only in the largest towns and where a canal or railway widens their reach | a malt kiln with a cowl beside an inn yard; later a brewing tower, a vat house and maltings, drays in the yard |
| `pottery` Pottery | 1 | country pottery, pottery manufactory (1770) | households (household goods); drain tiles are bricks, not pottery | 271 / 6,423 / 40,988 | on clay near a town; then where clay lies beside coal, clustered | one bottle oven and a drying shed by a flooded clay hole; later a courtyard of four to eight ovens and shard heaps |
| `gas` Town gas | 1 | gasworks (1815) | the households of its own town (fuel and light); shops, offices and warehouses | none / 20,956 / 81,598 | the low edge of every town of four thousand people or more, on the side the coal arrives | a retort house with a row of stub chimneys and round gasholders that rise and fall with the stock, one more for each tier |
| `chemicals` Soap and chemicals | 1 | soap boilers and ash burners, alkali works (1815), ammonia-soda works (1872) | households (soap, candles, soda); glass, paper and cotton works; farms once manures are bought (stage 4) | 250 / 6,289 / 63,289 | ash burners in the woods, boiling houses downwind of the towns; alkali works where limestone and coal meet on water; ammonia-soda works on the limestone nearest coal and a railway | pots under an open shed and cordwood stacks; then lead-chamber sheds, the tallest chimney on the map and a grey waste heap; then iron towers and white lagoons |
| `glass` Glass | 1 | glass cone, sheet glass works (1835), tank glass furnace (1874) | households (bottles, table glass, panes); contract builders; cotton mills' plant | 105 / 2,021 / 33,547 | where coal and sand meet (six of heat to one of glass); later coalfield towns on a railway | one brick cone eighty feet high; later long louvred sheds; then great halls with a pair of regenerator chimneys |
| `paper` Paper | 1 | paper mill, machine paper mill (1815), wood-pulp mill (1885) | households; shops, offices and warehouses | 182 / 5,419 / 42,022 | clean streams near towns (the rags come from the town); then the town edge; then the log boom at a river mouth | a stone mill with a wheel and slatted drying lofts; a long machine house and a chimney; a log pond and a digester tower |
| `flour` Flour | 2 | windmill, water corn mill, steam roller mill (1878) | households: a third to two fifths of food, two thirds once roller mills grind | 55,242 / 147,890 / 797,543 | windmills on open corn ground with no fall, watermills at falls within carting of a market; roller mills at the biggest market, and the quay once grain is landed | a tower with four turning sails; a gabled mill astride a leat with a pond above; a six-storey block with silos and a chimney |
| `fish` Fish | 2 | fishing grounds (land) | households | 22,512 / 89,622 / 129,492 | coast cells, densest where a harbour town or a railway is near | boats drawn up on a strand, net racks, a smokehouse, a huddle of cottages; smacks offshore, funnels after 1880 |
| `produce` Dairy, meat and garden produce | 2 | dairy and gardens (land) | households | 18,690 / 74,172 / 108,560 | a ring two to five cells wide round every town, stretching along the railways after 1840 | narrow bright strip plots, orchard dots, cattle in small hedged closes; glasshouses late |
| `cotton` Raw cotton | 3 | landed at the ports | cotton mills only: the one good with a single kind of buyer, kept because it is why the mills stand where they do | 2,757 / 30,454 / 170,029 | the quays of the port towns | bales on the quay and in bonded warehouses, carts and barges inland |
| `cottons` Cotton cloth | 3 | fustian weaving, water-frame cotton mill (1771), steam cotton mill (1805) | households (half of clothing by 1850, three quarters by 1890); the world (as much again, not in these figures) | 2,341 / 42,061 / 275,702 | weavers' cottages behind the ports; water-frame mills on the nearest fast water to a cotton port; steam mills between the port and the coal | cottages with a long row of top-floor windows; a many-windowed stone block with a wheelhouse and weir; red-brick mills with chimneys, mill ponds and saw-tooth weaving sheds |
| `ships` Ships | 5 | shipwright's yard, iron shipyard (1845), steel shipyard (1882) | port merchants for sea carriage and the fishery (stage 5); nobody without fleets | about 10 / 40 / 200 (the engine design's reckoning) | a sheltered shore with timber upriver; then the harbour nearest iron and coal | a hull in frame on a slip beside a timber pond; later plated hulls in red lead, sheer-legs and gantries |

The numbers of each good:

| good | weight | base price | colour | decay a month | opening stock a head | serves |
|---|---|---|---|---|---|---|
| `building` | 12 | 2.2 | `#b08d6a` | 0.1 | 0.09 | shelter 3 |
| `lumber` | 1.2 | 2.2 | `#b98d54` | 0.004 | 0.02 | shelter 0.6, wares 0.35 |
| `lime` | 1.5 | 2 | `#e2ddd0` | 0.01 | 0.03 | shelter 0.5 |
| `beer` | 2 | 2 | `#b8742a` | 0.06 | 0.07 | table 1 |
| `pottery` | 1 | 3.4 | `#c9b08a` | 0.003 | 0.01 | wares 1 |
| `gas` | 20 | 2.4 | `#e6c94f` | 0.15 | 0 | warmth 4.5 |
| `chemicals` | 0.9 | 4.5 | `#c9c23a` | 0.003 | 0.006 | wares 1.3 |
| `glass` | 1 | 9 | `#9fd0c8` | 0.003 | 0.003 | wares 2.5 |
| `paper` | 0.5 | 4.6 | `#f2ecd9` | 0.004 | 0.004 | wares 1.3 |
| `flour` | 0.9 | 1.4 | `#efe6cf` | 0.02 | 0.5 | food 1.5 |
| `fish` | 1.5 | 1.3 | `#7fa6b8` | 0.12 | 0.05 | food 1.1 |
| `produce` | 4 | 1.6 | `#9fbf5f` | 0.15 | 0.06 | food 1.4 |
| `cotton` | 0.5 | 2.2 | `#f4f1e6` | 0.003 | 0 | no need: works only |
| `cottons` | 0.25 | 6 | `#5f8fc4` | 0.004 | 0 | clothing 0.75 |
| `ships` | 2 | 160 | `#39607a` | 0.002 | 3e-05 | no need: works only |

Weights carry the map. Building work at 12 is never carted: it is made in the town that buys it. Gas at 20 stands within two or three cells of its market. Produce at 4 rings the town. Beer at 2 is brewed where it is drunk until a canal or a railway widens a great brewery's reach. Raw cotton at 0.5 travels, so cotton mills are drawn by coal and hands within reach of a port, not to the quay. Decay: building work loses a tenth of any stock a month and gas 0.15 (the first draft had 0.3 and 0.5, which under the game's price rule wasted one builder in seven and a fifth of the gas).

What the design does to goods that already exist. The first row of each pair is what households and the new trades want in the reckoning; the second is what the sixteen-seed run without them made, for every use (plant, railways and the base works as well as households), so the two are not the same measure and are to be read for direction:

| good | | 1770 | 1810 | 1850 | 1870 | 1890 | 1899 |
|---|---|---|---|---|---|---|---|
| stone | wanted by households and the new trades | 11,314 | 23,526 | 62,765 | 116,880 | 66,308 | 79,185 |
| | the run without them, all uses | 37,354 | 76,839 | 265,868 | 523,501 | 727,379 | 706,068 |
| bricks | wanted by households and the new trades | 1,565 | 4,258 | 12,448 | 50,198 | 201,617 | 251,443 |
| | the run without them, all uses | 5,467 | 24,835 | 77,419 | 179,464 | 572,510 | 775,563 |
| timber | wanted by households and the new trades | 13,135 | 24,715 | 40,778 | 52,073 | 76,194 | 87,432 |
| | the run without them, all uses | 27,909 | 35,231 | 25,568 | 19,966 | 14,001 | 14,422 |
| grain | wanted by households and the new trades | 137,870 | 271,425 | 494,065 | 844,998 | 1,308,831 | 1,524,585 |
| | the run without them, all uses | 216,570 | 428,326 | 744,891 | 1,318,664 | 1,920,955 | 2,090,102 |
| coal | wanted by households and the new trades | 39,637 | 90,239 | 213,870 | 355,762 | 593,507 | 648,977 |
| | the run without them, all uses | 43,768 | 127,543 | 321,977 | 495,056 | 768,569 | 850,094 |
| wool | wanted by households and the new trades | 9,893 | 21,740 | 16,355 | 7,435 | 1,164 | 440 |
| | the run without them, all uses | 21,688 | 45,415 | 58,250 | 80,166 | 104,815 | 114,203 |
| cloth | wanted by households and the new trades | 2,970 | 7,566 | 30,414 | 44,770 | 54,999 | 55,270 |
| | the run without them, all uses | 3,217 | 17,206 | 81,968 | 114,881 | 154,578 | 170,371 |
| clay | wanted by households and the new trades | 2,039 | 4,560 | 14,174 | 24,279 | 42,188 | 51,382 |
| | the run without them, all uses | 7,889 | 32,104 | 94,193 | 210,777 | 586,278 | 743,721 |

Stone is the point of the exercise: households and the builders want under a third of what the run digs until 1850 and a tenth of it by 1890, when the contract builders have turned the towns to brick. Bricks stop being bought raw by households at once (a town of 20,000 buys none where it bought 780 a month), so from 1750 to 1850 the brick trade is the bricklayers and other works' plant, and thin; after the kiln and the contract builders it is 250,000 a month. Grain is down by a third all century, because flour stretches it and fish and produce take a quarter of food. Wool is worn less and woven less once cotton lands. Clay goes with the bricks.

**Timber is the one that rises, and it is the thing to watch.** The building trades, the sawmills and the ash burners want 13,000 units a month in 1770 where the run cuts 28,000 for everything, which is a fall; but they want 41,000 in 1850 and 87,000 in 1899, where the run, with charcoal dead, cuts 14,000. The reckoning holds timber at the run's price (2.3 to 2.8). In the engine timber will be much dearer than that from about 1840, sawn timber and building with it, and the forests will be cut back hard. That is the history (the island's own woods could not build its towns), it is what log driving and plantations are for, and from stage 3 the answer is the same as it was: timber from abroad.

Running prices in the reckoning (island money; a blank is before the good is made). The rows marked "the run" are base goods and the wage in the sweep under the game's rules that the reckoning stands on:

| | 1750 | 1770 | 1790 | 1810 | 1830 | 1850 | 1870 | 1890 | 1899 |
|---|---|---|---|---|---|---|---|---|---|
| building | 4.43 | 5.20 | 6.50 | 7.17 | 8.25 | 7.68 | 7.04 | 3.18 | 2.89 |
| lumber | 3.81 | 4.31 | 5.39 | 6.06 | 7.13 | 4.71 | 4.21 | 3.40 | 3.36 |
| lime | 4.87 | 4.02 | 4.83 | 5.13 | 5.60 | 4.69 | 2.89 | 2.20 | 2.14 |
| beer | 2.52 | 2.65 | 2.61 | 1.70 | 1.75 | 1.37 | 0.95 | 0.79 | 0.80 |
| pottery | 8.96 | 7.69 | 5.86 | 5.28 | 5.27 | 3.99 | 3.54 | 3.21 | 3.15 |
| gas |  |  |  |  | 3.73 | 2.76 | 2.41 | 2.14 | 2.18 |
| chemicals | 9.19 | 9.29 | 11.61 | 12.93 | 7.84 | 5.44 | 4.47 | 2.35 | 2.06 |
| glass | 23.84 | 20.37 | 24.99 | 26.83 | 27.73 | 14.01 | 9.51 | 4.70 | 4.24 |
| paper | 10.07 | 12.48 | 14.97 | 15.45 | 6.53 | 4.38 | 3.86 | 3.36 | 3.23 |
| flour | 1.04 | 1.37 | 1.18 | 1.21 | 1.40 | 1.21 | 0.77 | 0.53 | 0.48 |
| fish | 0.98 | 1.31 | 1.07 | 1.08 | 1.25 | 1.06 | 0.57 | 0.50 | 0.53 |
| produce | 1.24 | 1.65 | 1.35 | 1.36 | 1.57 | 1.34 | 0.72 | 0.63 | 0.67 |
| cotton | 4.40 | 4.16 | 3.92 | 3.67 | 3.40 | 3.13 | 3.31 | 2.84 | 2.62 |
| cottons | 12.49 | 14.25 | 7.72 | 7.09 | 5.24 | 3.93 | 3.63 | 2.87 | 2.63 |
| coal (the run) | 3.39 | 1.42 | 1.58 | 1.48 | 1.53 | 1.10 | 0.96 | 0.85 | 0.94 |
| stone (the run) | 1.06 | 1.24 | 1.51 | 1.65 | 1.80 | 1.40 | 1.18 | 0.92 | 0.93 |
| bricks (the run) | 2.99 | 3.32 | 3.88 | 3.81 | 4.20 | 3.39 | 2.69 | 1.66 | 1.51 |
| timber (the run) | 1.49 | 1.56 | 1.96 | 2.24 | 2.82 | 2.75 | 2.68 | 2.25 | 2.29 |
| grain (the run) | 0.73 | 0.97 | 0.79 | 0.80 | 0.92 | 0.79 | 0.42 | 0.37 | 0.40 |
| iron (the run) | 21.32 | 26.05 | 15.68 | 8.36 | 8.34 | 5.03 | 4.04 | 3.18 | 2.56 |
| machinery (the run) | 1647.35 | 216.68 | 209.55 | 156.48 | 121.73 | 35.44 | 18.50 | 10.46 | 9.35 |
| wage (the run) | 4.1 | 5.3 | 6.7 | 7.5 | 8.6 | 9.7 | 11.9 | 14.1 | 14.7 |
| goods budget a head | 2.27 | 2.94 | 3.81 | 4.42 | 5.05 | 5.70 | 6.66 | 7.63 | 7.96 |

Which method makes each new good (share of its output):

| good | method | 1770 | 1790 | 1810 | 1830 | 1850 | 1870 | 1890 | 1899 |
|---|---|---|---|---|---|---|---|---|---|
| building | Masons and carpenters | 49% | 49% | 48% | 50% | 37% | 18% |  |  |
| building | Timber-frame builders | 34% | 32% | 29% | 26% | 36% | 16% |  |  |
| building | Bricklayers | 16% | 18% | 23% | 25% | 27% | 16% |  |  |
| building | Contract builders |  |  |  |  |  | 50% | 100% | 100% |
| lumber | Saw pit | 24% | 23% | 23% | 16% | 1% |  |  |  |
| lumber | Steam sawmill |  |  |  | 30% | 94% | 98% | 100% | 100% |
| lumber | Water sawmill | 76% | 77% | 77% | 54% | 5% | 2% |  |  |
| lime | Lime kiln | 100% | 100% | 100% | 100% | 100% | 5% | 1% | 1% |
| lime | Cement works |  |  |  |  |  | 95% | 99% | 99% |
| beer | Brewhouse | 100% | 70% | 3% | 1% |  |  |  |  |
| beer | Common brewery |  | 30% | 97% | 99% | 100% | 100% | 100% | 100% |
| pottery | Country pottery | 100% | 4% | 1% |  |  |  |  |  |
| pottery | Pottery manufactory |  | 96% | 99% | 100% | 100% | 100% | 100% | 100% |
| gas | Gasworks |  |  |  | 100% | 100% | 100% | 100% | 100% |
| chemicals | Soap boilers and ash burners | 100% | 100% | 100% | 1% |  |  |  |  |
| chemicals | Alkali works |  |  |  | 99% | 100% | 100% | 4% | 1% |
| chemicals | Ammonia-soda works |  |  |  |  |  |  | 96% | 99% |
| glass | Glass cone | 100% | 100% | 100% | 100% | 1% |  |  |  |
| glass | Sheet glass works |  |  |  |  | 99% | 100% | 4% | 2% |
| glass | Tank glass furnace |  |  |  |  |  |  | 96% | 98% |
| paper | Paper mill | 100% | 100% | 100% |  |  |  |  |  |
| paper | Machine paper mill |  |  |  | 100% | 100% | 100% | 100% | 100% |
| paper | Wood-pulp mill |  |  |  |  |  |  |  |  |
| flour | Windmill | 47% | 47% | 47% | 47% | 46% | 43% | 6% | 2% |
| flour | Water corn mill | 53% | 53% | 53% | 53% | 54% | 57% | 9% | 4% |
| flour | Steam roller mill |  |  |  |  |  |  | 85% | 94% |
| cottons | Fustian weaving | 100% |  |  |  |  |  |  |  |
| cottons | Water-frame cotton mill |  | 100% | 100% | 11% | 4% | 1% |  |  |
| cottons | Steam cotton mill |  |  |  | 89% | 96% | 99% | 100% | 100% |

## 6. The chains

### 6.1 Building (stage 1)

```
stone, timber, lime ──> building work           masons and carpenters (1750)
sawn timber, timber, clay ──> building work     timber-frame builders (1750), where the woods are near and the rock is not
bricks, lime, sawn timber ──> building work     bricklayers (1750), on the clay
bricks, lime, sawn timber, glass ──> building   contract builders (1862), with the kiln's bricks
stone + coal ──> lime                           lime kiln (1750) ──> cement works (1850)
clay + coal ──> bricks                          brickworks (base) ──> continuous kiln (1856)
```

Why it is first. In the baseline the shelter need is a seventh of all spending and goes straight to the land: stone is the cheapest shelter per unit, so the money of a growing island is spent on a fixed rock field with falling returns, and quarrying takes a sixth of all labour by 1900. With builders between the household and the quarry the same money buys labour in town and a third of the stone.

What it gives the map. Three ways to build, each a known method, chosen town by town on delivered prices: stone towns near the quarries, brick towns on the clay, timber-framed towns under the woods. The viewer can draw each town's houses in the material its builders use, which is honest decoration and answers the complaint that houses have less to them than works. Scaffolding belongs on whatever the town's builders are putting up. After 1862 the contract builder, who needs cheap brick, turns the growing edges of the large towns to terraces, and the quarries that fed them go quiet. Lime kilns stand under the quarries on the side nearest coal; cement works, a few on the island, on limestone by a quay or a siding.

What it needs from the engine: the loader, and `group` so that a family can own a builder's yard. With no seed plan the builders are founded during the warm-up decade like any new trade, a works a year in each town and then 15 per cent of the trade a year, so a town's builders reach its demand in about eight of the ten years; until then households build for themselves, at a loss. With the seed plan 1750 opens full.

After 1900: steel frames and concrete, council building, then system building. Construction stays a large employer of hands to the end, which the labour story needs: it is the trade machines are slowest to take.

### 6.2 Timber and logging

```
woodland ──> timber (felled)        forestry (base land sector, on the tree cover the engine already grows and clears)
timber ──> sawn timber              saw pit (1750) ──> water sawmill (1750, at a fall) ──> steam sawmill (1825)
sawn timber ──> builders, plant, furniture; pit props (deep mining); sleepers (railway upkeep); ships
timber ──> paper                    wood-pulp mill (1885)
timber ──> charcoal ──> iron        (base; ends with coke)
```

The place of each. Felling is where the trees are. The saw pit stands at the wood's edge and in every town that builds. The water sawmill takes the first good fall below the woods. The steam sawmill stands where logs arrive in bulk: a river mouth, a timber quay, a railway yard. The pulp mill stands at the boom, where a log drive ends, and nowhere else: at the island's average timber price it fails its entry test and at the price of logs delivered by water it passes.

What it needs from the engine, in order:

1. Stage 1, nothing: sawn timber is a good, the saw pit and the steam sawmill are works.
2. Stage 2, **log driving**. Timber, and only timber, moves down any river with enough discharge at a twentieth of the cost of a road, one way. The yearly delivered-price pass is already run per good, so this is a cost table for one good in one direction. It is what makes "why is the sawmill there" a river's answer, and the viewer may then draw logs in the rivers, because the simulation knows they are there.
3. Stage 2, the water sawmill's site, with its capacity: the saw pit lives on beside the water mills because the streams near the woods fill.
4. Stage 4, **pit props**: deep mining buys 0.015 of sawn timber for each unit raised. With the mines rule (workings that go deeper) this is a standing market that grows with coal: about 4,000 units a month by 1850 on the baseline's coal.
5. Stage 5, **sleepers**: railways buy sawn timber when they build (today they buy raw timber: one name in `rail.rs`) and every year after.
6. Not asked for now: plantations. The engine regrows cover on its own; replanting as a paying use of poor land is a twentieth-century story. It may have to come sooner: the builders and sawmills want six times the timber the island cuts by 1899 (section 5).

After 1900: pulp becomes the main use (newsprint, packaging), plywood and board follow, pit props die with the pits, and softwood plantations cover the uplands that sheep leave.

### 6.3 Water

Water is not a good in this design. Nothing could carry it between towns, every household has some from a well, and a good whose market cannot be missed is bookkeeping. It is a field of the island and a set of things built on it, and each use is a place.

| use | from | what it does | needs |
|---|---|---|---|
| **Power at a fall** | 1750 | Corn mills, cloth mills, saw mills, paper mills and edge-tool works take `site = { field = "power", kw }`: they may stand only where the new island's water power is, and each draws its kilowatts from the cell (a usual size of corn mill 6, of saw mill 8, of woollen or cotton mill 25). The mill pond is in the plant (stone, sawn timber). | stage 2: the generator's `power`, the site rule, and the **capacity**: the works on a cell share its power, so good streams fill and later mills go upstream or wait for steam |
| **Wind** | 1750 | The windmill takes `site = wind`: the mill of corn country with no fall. | stage 2: a wind field derived from height and the lee of the prevailing wind |
| **A harbour** | 1750 | Shipyards may stand only there, and the fishing is richer near one. | stage 2: derived from the generator's estuary heads and lake shores |
| **Canal water** | 1765 | A canal that crosses a watershed needs a reservoir at its summit: a dam on an upland stream, drawn as a lake. | the canals mechanism |
| **Floods, banks and drains** | 1790 | Flood-plain cells are the best corn land and lose a harvest in a wet year (the harvest rule's district draw, on the cells the new island marks). Embanking is an improvement the farms there buy (sawn timber, bricks): a fifth more output and no flood years. Where the new island has a dry side, irrigation is the same rule with the river's discharge as its limit. | the flood plain from the generator; improvements as data |
| **Town water** | 1848 | A town past the size its own streams can serve commissions a reservoir in the hills and an iron main. It buys bricks, cement and iron, drowns a valley, and halves the town's death penalty in the people rule. | reservoirs; one hook in `peopling.rs` |

The smallest honest version of a reservoir is one rule: a dam is a built thing on a river cell that turns the cells upstream below its crest into water for good, steadies the flow below it (more usable power, no floods), and serves one canal or one town. Whoever needs it builds it: canal promoters, or the town. That is also where the politics half will find municipal gas and water.

Power as a carrier of its own (a "power" good met by water, wind or steam) was weighed by all three designers and refused by all three, and that stands: a water method and a steam method with a site between them give the same picture, and the observer can report the crossing.

After 1900 the same fields are the hydro-electric sites, the reservoirs are the cities' water, and the flood plains are where the suburbs should not have been built.

### 6.4 Canals

The mechanism is being built by another hand tonight, on the promoters the railways already use. The content's part:

- The invention is the mechanism's own (`discover_coal` and `discover_stone` in its parameters), not the overlay's: see the end of section 9.
- **What a canal should buy**: stone, bricks, sawn timber and lime for each cell, more for a lock, and navvies' wages. (The mechanism today buys stone, bricks, raw timber and puddling clay; sawn timber and lime are two names to change once stage 1 is loaded.) It is the first boom in building materials, sixty years before the railway's.
- **What it does to this tree**: coal is the cheapest heavy thing on the island and most of these works burn it. Potteries (1.6 of heat to a unit), lime kilns (1.3), glass (4 to 6), alkali (3), breweries and gasworks all stand where coal is cheap, so a canal draws a line of them along its banks and lets inland towns have them before 1830. Beer's weight of 2 means a great brewery's reach is what a canal makes it.
- The canal's water is section 6.3.

After 1900 canals carry little; they are the diorama's quiet water and, in places, the line a motorway follows.

### 6.5 Iron, steel and engineering

```
ore + charcoal ──> iron        bloomery, charcoal furnace (base)
ore + coal ──> iron            coke furnace (base, 1765) ──> hot blast (1830)
iron ──> steel                 crucible (base, 1760) ──> converter (base, as the machines rule has it, 1856)
iron ──> tools                 smithy (base) ──> edge-tool works, with a little steel (1778) ──> tool factory, all steel (1864)
iron, tools ──> machinery      millwright, engine works (base) ──> machine-tool works (1838) ──> engineering works, of steel (1876)
iron, steel ──> rails          forge, rolling mill, steel rail mill (base)
```

Seven of these are the port's and are not touched beyond three usual sizes and the converter's and steel rail mill's thresholds. The five that are new were run under the game's rules (section 11). Hot blast makes half of the iron by 1849 and takes its price from 8 to 2. The machine-tool works makes half of the machinery by 1850 and takes its price from 72 to 9; it is the hinge of the middle of the century, since every later works buys its plant with it. The tool factory and the engineering works are the first things built of the converter's steel.

### 6.6 Cloth

```
wool ──> cloth                 handloom (base) ──> water-powered woollen mill (1775, stage 2) ──> steam-powered mill (base; moved to 1798)
raw cotton ──> cotton cloth    fustian weaving (1750) ──> water-frame mill (1771) ──> steam cotton mill (1805)   (stage 3)
```

Water comes before steam in both. The water mills string out along the hill valleys nearest the sheep, or nearest a cotton port; when the streams are full the steam mill takes the growth and stands where coal and hands are. Cotton needs foreign trade and waits for it; nothing else waits for cotton.

### 6.7 Food and drink

```
grain ──> flour                windmill, water corn mill (1750) ──> steam roller mill (1878)          (stage 2)
grain ──> beer                 brewhouse (1750) ──> common brewery (1785)                              (stage 1)
garden land ──> produce        a land sector, ringed round the towns                                   (stage 2)
coast ──> fish                 a land sector on coastal cells; smacks (1750), steam trawlers (1882)    (stages 2 and 4)
```

Flour is grain made to go further (1.12 of grain gives 1.5 of food); with fish and produce it lowers the grain the island buys by a third. It is in the tree for the mills: they are the first users of water and wind sites and the furniture of the 1750 map.

### 6.8 Household goods, gas and shops

```
clay + coal ──> pottery        country pottery (1750) ──> pottery manufactory (1770)                  (stage 1)
wood ash ──> soap and soda     soap boilers and ash burners (1750) ──> alkali works (1815) ──> ammonia-soda works (1872)
sand, soda, coal ──> glass     glass cone (1750) ──> sheet glass (1835) ──> tank furnace (1874)
rags ──> paper                 paper mill (1750) ──> machine paper (1815) ──> wood pulp (1885)
coal ──> town gas              gasworks (1815)                                                         (stage 1)
hands, gas, paper ──> services shops and trades (base) ──> shops, offices and warehouses (1860)       (stage 1)
```

Each of the four household goods has a trade that moves: pottery onto the coal, alkali from the woods to the coal and then, burning a third as much, out along the limestone, glass onto the coalfield, paper from the streams to the town edge and then to the log boom. The shops' second generation is the only method that makes services with fewer hands; it buys gas and paper, and it should be drawn.

## 7. The four waves

Dates are the years aimed at; in the engine one seed in five finds a thing more than about twelve years from its aim, and a method that enters a little above the going price takes three to eight years more to be built.

**1750, the organic economy.** Every town has its builders and takes the look of what lies near it: stone, brick or timber frame. There is a brewhouse in every market town, a potter on the nearest clay, saw pits at the wood's edge, lime kilns under the rock. Soap is boiled downwind of the towns from ash burnt in the woods; a glass cone or two stands where coal is cheapest; paper mills take the clean streams. From stage 2 the corn country is dotted with windmills, every fall near a town turns a mill, gardens ring the towns and boats work the coast. Everything is small and stands at its material or its market, and wood is under all of it: fuel, frames, charcoal, ash. What to watch: timber. Charcoal iron, ash burning and building all draw on the same woods, and that is what makes coal worth the trouble.

**1760 to 1830: coal, iron, cotton, canals, steam.** Things learn to burn coal, and move to it. Crucible steel (1760) and coke iron (1765) as now. Canals (the mechanism's own clock, the 1760s) carry coal to the towns that lack it, and the heat-hungry trades line the cuts. Pottery manufactories (1770) gather where clay lies beside coal and stay gathered. Water-frame cotton mills (1771) and water-powered woollen mills (1775) fill the hill valleys; edge-tool works (1778) take streams near the steel and keep tools cheap as the smithies give way. The common brewery (1785) makes the largest towns' breweries landmarks. Then steam frees the mill from the stream: the steam-powered mill (1798) and the steam cotton mill (1805) stand where coal and hands are, and the valleys keep what they have. Alkali works (1815) end the ash burners; the paper machine (1815) leaves the streams; gasworks (1815) rise at the edge of every town of a size; deep mining (1815) lifts the pits for the first time since the pumps; steam saws (1825) end the saw pit and then the water sawmill. Why it happens: heat. Every one of these is a way of spending coal to save hands or wood.

**1830 to 1870: railways, engineering, iron ships.** Hot blast (1830) and the railway (as now). Sheet glass (1835) puts windows in ordinary houses. The machine-tool works (1838) is the hinge: in the runs machinery falls from 124 in 1830 to 21 in 1870, and that brings on the land's machines, deeper pits and every later works. Tile drainage (1843) and artificial manures (1850) make farms customers of the brickworks and the alkali works; farm machinery (1850) is bought where it pays. Town waterworks (1848) dam the upland valleys. Cement (1850). The converter (1856) and the continuous kiln (1856): cheap steel and cheap brick. Shops, offices and warehouses (1860) in the gas-lit towns. Contract builders (1862) run up brick terraces, building work falls from 7 a unit to 3 in twenty years, and the quarries shrink. Tool factories (1864), steel rails (1866), mechanical haulage (1868). If fleets are built, iron shipyards (1845) settle at the harbour nearest the iron. Why it happens: cheap iron and cheap machines, each making the next cheaper, and the railway taking weight out of the map so that trades once tied to their market gather.

**1870 to 1900: steel, chemicals, gas, food from abroad.** Ammonia-soda works (1872) halve the price of soda by 1890: soap in every house and manure on every farm. Tank furnaces (1874) do the same for glass. Engineering works (1876) build machinery of steel with a third of the hands. Roller mills (1878) end the wind and water mills within fifteen years, and flour goes from a fifth of food in 1870 to three fifths by 1899. Steam trawlers (1882) and the fish train. Pulp mills (1885) at the log booms. Gas holds a fifth of fuel and light and passes a quarter by 1899. Half a million units of building work a month go up in brick, eight times 1850's. With foreign trade the world's grain falls by three fifths and ocean steamers (1866) cut the freight: grain lands at the ports below what the heavier land can grow it for, arable near the ports goes to grass and gardens, timber comes in for the builders, and the island pays in cotton cloth, rails and machinery until the world learns to make its own. Why the late century does not stall: eleven methods and three improvements arrive after 1855, each still climbing its curve in 1900; the machines rule already has the land's productivity rising to the end; and the two largest changes are in things every household buys every month, its house and its bread. Two of the eleven are weak and the design says so: the steel rail mill is found and hardly built until railways wear out (stage 5), and the pulp mill is built only where logs arrive cheap.

## 8. Methods

All numbers; the overlay has the same with headlines and stories. "Found after" is cumulative output since 1750: "xN" is N years of 1750's own output, a plain number is units. Plain numbers are used for every new good, because "xN" means "never" for a good that is not made in the warm-up, and, after the critique, for every coal threshold after 1800: across the sixteen seeds an "xN" in coal is met over 25 to 32 years, the same threshold in plain units over 15 to 26, and one in iron over 11 to 13. So each late invention's clock is its own good, with iron, steel or machinery beside it as a guard set a little early, and coal only as a second guard.

**Stage 1**

| method | makes | inputs a unit | heat | labour | plant a unit of monthly capacity | K0 | A0 / Q0 / b | found after | aimed at | site |
|---|---|---|---|---|---|---|---|---|---|---|
| `masons` Masons and carpenters | building | stone 1.1, timber 0.35, lime 0.1 | 0 | 0.25 | timber 3, tools 0.05 | 300 | known in 1750 |  |  |  |
| `carpenters` Timber-frame builders | building | lumber 0.5, timber 0.3, clay 0.3 | 0 | 0.22 | timber 3, tools 0.05 | 200 | known in 1750 |  |  |  |
| `bricklayers` Bricklayers | building | bricks 0.6, lime 0.12, lumber 0.2 | 0 | 0.24 | timber 3, tools 0.05 | 300 | known in 1750 |  |  |  |
| `contract_builders` Contract builders | building | bricks 0.65, lime 0.1, lumber 0.18, glass 0.02 | 0 | 0.1 | bricks 2, lumber 2, tools 0.1 | 800 | 1 / 2,000,000 / 0.2 | building 41,000,000, machinery 410,000 | 1862 |  |
| `saw_pit` Saw pit | lumber | timber 1.4 | 0 | 0.28 | timber 2, tools 0.05 | 60 | known in 1750 |  |  |  |
| `steam_sawmill` Steam sawmill | lumber | timber 1.25 | 0.3 | 0.05 | bricks 4, timber 3, machinery 0.03 | 600 | 0.85 / 300,000 / 0.2 | lumber 5,000,000, machinery 55,000 | 1825 |  |
| `lime_kiln` Lime kiln | lime | stone 1.6 | 1.3 | 0.08 | stone 4, bricks 1.5 | 100 | known in 1750 |  |  |  |
| `cement_works` Cement works | lime | stone 0.8, clay 0.2 | 0.9 | 0.02 | bricks 8, iron 0.5, machinery 0.03 | 400 | 0.6 / 1,000,000 / 0.2 | lime 2,800,000, machinery 200,000 | 1850 |  |
| `brewhouse` Brewhouse | beer | grain 1.2 | 0.4 | 0.1 | timber 3, bricks 2, stone 1 | 300 | known in 1750 |  |  |  |
| `steam_brewery` Common brewery | beer | grain 0.92 | 0.25 | 0.02 | bricks 4, lumber 2, iron 0.3, machinery 0.02 | 1500 | 0.6 / 300,000 / 0.2 | beer 2,100,000, coal 13,000,000 | 1785 |  |
| `potbank` Country pottery | pottery | clay 1.3 | 2.5 | 0.5 | bricks 6, clay 2, timber 3 | 40 | known in 1750 |  |  |  |
| `pottery_works` Pottery manufactory | pottery | clay 1.2 | 1.6 | 0.14 | bricks 12, tools 0.05 | 100 | 0.65 / 80,000 / 0.25 | pottery 39,000, coal x35 | 1770 |  |
| `gasworks` Gasworks | gas | coal 1.2, lime 0.04 | 0 | 0.06 | bricks 10, iron 2, lumber 1 | 150 | 0.85 / 150,000 / 0.15 | iron 1,200,000, coal 47,000,000 | 1815 |  |
| `edge_tool_works` Edge-tool works | tools | steel 0.08, iron 0.25 | 0.6 | 0.45 | stone 15, timber 8, tools 2 | 60 | 0.9 / 30,000 / 0.15 | steel 400, tools x20 | 1778 | power, 6 kW |
| `tool_factory` Tool factory | tools | steel 0.22 | 0.5 | 0.06 | bricks 20, machinery 0.4 | 600 | 1 / 300,000 / 0.15 | steel 4,000,000, machinery 350,000 | 1864 |  |
| `hot_blast` Hot-blast furnace | iron | iron_ore 1.1, coal 0.8 | 0 | 0.05 | stone 30, bricks 45, machinery 0.25 | 150 | 1 / 400,000 / 0.3 | iron 1,720,000, coal 65,000,000, machinery 70,000 | 1830 |  |
| `machine_shop` Machine-tool works | machinery | iron 1.4, steel 0.25, tools 0.15 | 1 | 0.6 | bricks 60, machinery 0.6, tools 10 | 40 | 1 / 20,000 / 0.2 | machinery 110,000, steel 1,300,000 | 1838 |  |
| `engineering_works` Engineering works | machinery | steel 0.6, tools 0.08 | 0.8 | 0.1 | bricks 60, machinery 1.5, tools 10 | 150 | 1 / 200,000 / 0.15 | steel 7,500,000, machinery 1,000,000 | 1876 |  |
| `continuous_kiln` Continuous kiln | bricks | clay 1.2 | 0.2 | 0.03 | bricks 8, stone 3, machinery 0.02 | 2000 | 1 / 2,000,000 / 0.15 | bricks x300, machinery 300,000 | 1856 |  |
| `emporium` Shops, offices and warehouses | services | gas 0.04, paper 0.02 | 0 | 0.5 | bricks 8, iron 0.2 | 1000 | 1 / 5,000,000 / 0.1 | gas 1,000,000, services 32,000,000 | 1860 |  |
| `soap_boilers` Soap boilers and ash burners | chemicals | timber 2.5 | 1 | 0.45 | timber 4, bricks 2 | 40 | known in 1750 |  |  |  |
| `alkali_works` Alkali works | chemicals | stone 0.8 | 3 | 0.15 | bricks 25, stone 8, iron 1.5 | 150 | 0.7 / 60,000 / 0.25 | chemicals 340,000, iron 1,000,000 | 1815 |  |
| `ammonia_soda` Ammonia-soda works | chemicals | lime 0.25 | 1 | 0.04 | bricks 30, iron 3, machinery 0.15 | 2000 | 1 / 800,000 / 0.25 | chemicals 4,200,000, steel 4,400,000 | 1872 |  |
| `glasshouse` Glass cone | glass | stone 1, chemicals 0.3 | 6 | 1.2 | bricks 20, stone 6 | 30 | known in 1750 |  |  |  |
| `sheet_glass` Sheet glass works | glass | stone 1, chemicals 0.25, lime 0.1 | 4 | 0.35 | bricks 30, iron 1, machinery 0.05 | 150 | 0.6 / 40,000 / 0.3 | glass 250,000, chemicals 630,000, iron 2,000,000 | 1835 |  |
| `tank_furnace` Tank glass furnace | glass | stone 0.9, chemicals 0.2, lime 0.12 | 1.5 | 0.09 | bricks 30, steel 0.3, machinery 0.15 | 1500 | 1 / 800,000 / 0.2 | glass 1,400,000, steel 4,400,000 | 1874 |  |
| `paper_mill` Paper mill | paper | cloth 0.2 | 0.5 | 1 | timber 10, stone 8 | 30 | known in 1750 |  |  | power, 8 kW |
| `machine_paper` Machine paper mill | paper | cloth 0.2, chemicals 0.05 | 1 | 0.2 | bricks 15, machinery 0.2 | 150 | 0.6 / 30,000 / 0.3 | paper 230,000, machinery 30,000 | 1815 |  |

**Stage 2**

| method | makes | inputs a unit | heat | labour | plant a unit of monthly capacity | K0 | A0 / Q0 / b | found after | aimed at | site |
|---|---|---|---|---|---|---|---|---|---|---|
| `pulp_mill` Wood-pulp mill | paper | timber 0.5, chemicals 0.08 | 0.6 | 0.04 | bricks 15, machinery 0.15, steel 0.05 | 2000 | 1 / 800,000 / 0.25 | paper 6,100,000, chemicals 5,800,000, machinery 1,700,000 | 1885 |  |
| `windmill` Windmill | flour | grain 1.12 | 0 | 0.01 | lumber 1, stone 0.4, tools 0.01 | 800 | known in 1750 |  |  | wind |
| `watermill` Water corn mill | flour | grain 1.1 | 0 | 0.008 | lumber 1, stone 0.9, tools 0.01 | 1200 | known in 1750 |  |  | power, 6 kW |
| `roller_mill` Steam roller mill | flour | grain 0.96 | 0.08 | 0.002 | bricks 1, machinery 0.012, steel 0.01 | 3000 | 0.8 / 3,000,000 / 0.2 | flour 150,000,000, steel 5,500,000, machinery 1,200,000 | 1878 |  |
| `water_wool_mill` Water-powered woollen mill | cloth | wool 1 | 0 | 0.28 | stone 20, lumber 12, machinery 0.2 | 100 | 0.9 / 30,000 / 0.15 | cloth x22, machinery 3,000 | 1775 | power, 25 kW |
| `water_sawmill` Water sawmill | lumber | timber 1.35 | 0 | 0.16 | timber 8, stone 3, iron 0.2 | 200 | known in 1750 |  |  | power, 8 kW |

**Stage 3**

| method | makes | inputs a unit | heat | labour | plant a unit of monthly capacity | K0 | A0 / Q0 / b | found after | aimed at | site |
|---|---|---|---|---|---|---|---|---|---|---|
| `fustian_weaving` Fustian weaving | cottons | cotton 1.1 | 0 | 1 | timber 12.5 | 60 | known in 1750 |  |  |  |
| `water_frame_mill` Water-frame cotton mill | cottons | cotton 1 | 0 | 0.28 | stone 20, lumber 12, machinery 0.2 | 120 | 0.9 / 30,000 / 0.15 | cottons 380,000, machinery 2,200 | 1771 | power, 25 kW |
| `steam_cotton_mill` Steam cotton mill | cottons | cotton 0.85, chemicals 0.02 | 0.4 | 0.06 | bricks 12, iron 0.5, glass 0.1, machinery 0.3 | 1000 | 1 / 300,000 / 0.2 | cottons 2,900,000, machinery 17,000, iron 700,000 | 1805 |  |

**Stage 5**

| method | makes | inputs a unit | heat | labour | plant a unit of monthly capacity | K0 | A0 / Q0 / b | found after | aimed at | site |
|---|---|---|---|---|---|---|---|---|---|---|
| `shipyard` Shipwright's yard | ships | lumber 30, iron 1.5, cloth 2, tools 0.5 | 0 | 16 | lumber 40, stone 10, tools 6 | 0.5 | known in 1750 |  |  | harbour, must stand there |
| `iron_shipyard` Iron shipyard | ships | iron 7, steel 0.5, lumber 3, machinery 0.2 | 3 | 6 | bricks 60, stone 30, tools 15, machinery 3 | 10 | 0.65 / 5,000 / 0.2 | ships 15,000, iron 2,700,000, machinery 120,000 | 1845 | harbour, must stand there |
| `steel_shipyard` Steel shipyard | ships | steel 3, lumber 1.5, machinery 0.25 | 2 | 1.2 | bricks 60, steel 3, machinery 3, tools 10 | 30 | 1 / 20,000 / 0.2 | ships 40,000, steel 6,900,000 | 1882 | harbour, must stand there |

Six base methods are touched (the overlay says why at each): the usual sizes of the smithy (10), crucible steel (10) and the engine works (8), which were in every run; the converter's and the steel rail mill's plant and steel as the machines rule has them, written out, with thresholds moved for the machinery and steel this content makes; and, from stage 2, the port's powered mill, relabelled "steam-powered" and found some fifteen years later so that water has its generation.

### 8.1 The rules the numbers follow

- **A method known in 1750** earns about a fifth on its recipe at base prices and a wage of 2.5, as the port's known methods do (smithy 25 per cent, handloom 19, brickworks 26). Two are below on purpose: bricklayers (brick is dear until the kiln) and timber-frame builders. One is above: the water sawmill (33 per cent), whose streams fill.
- **The first invention on a good whose old method never learns** starts clumsy (`A0` 0.6 to 0.9) and learns slowly (`b` 0.15 to 0.3 on a large `Q0`), so that it about doubles in fifty years and is still climbing in 1900. In the town that finds it, it costs a little more than the going price; once know-how has spread it costs a tenth to a quarter less. This is the brief's "beats the old one only after some learning", with the learning being the town's. It is also small: a usual size of 100 to 150 (400 for cement), so that its first works, at 0.4 of that, is under a sixth of the island's trade in the year it is found. Under the life cycle it grows where it stands.
- **A successor to a method that has learned** starts at `A0 = 1` with a recipe already below what the old works use at their plateau, and light plant. It does not rely on the machines rule's inheritance. (That rule does reach one pair in the overlay: the port's powered mill, at `A0` 0.5, starts from half the water-powered woollen mill's skill once the water mill comes first, and the other way about in a seed where steam is found first. It is harmless and should be in the fit.)
- **Plant** is 1 to 5 months of output for a craft and 5 to 15 for heavy works at base prices, and nothing asks for more than a trace of machinery before 1800, when machinery costs 150.
- **No recipe has more than four inputs**, and no thin good is a running input of a large trade unless an older method without it stands beside it (gas and paper in the new shops, glass in contract building) or the amount is a pinch against a deep stock (lime in the masons' mortar).

### 8.2 Methods known in 1750, at base prices

Wage 2.5, heat 0.376 a unit. The margin on the recipe as written is the fair test. The last column is the engine's own sum at the usual size (labour times 1.8, materials times 1.1, the merchant's 6 per cent): no known method breaks even there at base prices, old or new (the port's smithy is at -11 per cent and its handloom at -27), because base prices are not the island's prices: tools open at 18 and cloth at 9. The brief's "about break even at efficient scale at base prices" cannot be passed by any content, the port's included.

| method | makes | sells at | inputs | heat | labour | cost | margin on the recipe | plant, months of output | margin at the usual size, with the engine's penalties and the merchant's spread |
|---|---|---|---|---|---|---|---|---|---|
| Masons and carpenters | building | 2.20 | 1.14 | 0.00 | 0.62 | 1.77 | 20% | 1.3 | -14% |
| Timber-frame builders | building | 2.20 | 1.46 | 0.00 | 0.55 | 2.01 | 9% | 1.3 | -24% |
| Bricklayers | building | 2.20 | 1.76 | 0.00 | 0.60 | 2.36 | -7% | 1.3 | -43% |
| Saw pit | lumber | 2.20 | 1.12 | 0.00 | 0.70 | 1.82 | 17% | 0.9 | -19% |
| Lime kiln | lime | 2.00 | 0.96 | 0.49 | 0.20 | 1.65 | 18% | 2.5 | -6% |
| Brewhouse | beer | 2.00 | 1.20 | 0.15 | 0.25 | 1.60 | 20% | 3.3 | -4% |
| Country pottery | pottery | 3.40 | 0.52 | 0.94 | 1.25 | 2.71 | 20% | 4.1 | -22% |
| Soap boilers and ash burners | chemicals | 4.50 | 2.00 | 0.38 | 1.12 | 3.50 | 22% | 1.5 | -10% |
| Glass cone | glass | 9.00 | 1.95 | 2.26 | 3.00 | 7.21 | 20% | 4.4 | -20% |
| Paper mill | paper | 4.60 | 1.00 | 0.19 | 2.50 | 3.69 | 20% | 2.8 | -33% |
| Windmill | flour | 1.40 | 1.12 | 0.00 | 0.03 | 1.15 | 18% | 1.8 | 3% |
| Water corn mill | flour | 1.40 | 1.10 | 0.00 | 0.02 | 1.12 | 20% | 2.0 | 5% |
| Water sawmill | lumber | 2.20 | 1.08 | 0.00 | 0.40 | 1.48 | 33% | 4.3 | 7% |
| Fustian weaving | cottons | 6.00 | 2.42 | 0.00 | 2.50 | 4.92 | 18% | 1.7 | -25% |
| Shipwright's yard | ships | 160.00 | 89.00 | 0.00 | 40.00 | 129.00 | 19% | 0.9 | -12% |

### 8.3 The entry test

What a new works of each invention would have to charge in the year it is found (0.4 of its usual size, with the engine's penalties for size and for know-how, and the hurdle on its plant), against what its good then sells for, at the game's prices. For the seven methods that were run the year is the runs' mean and the price is the run's own; for the rest the year is the reckoning's median and the price is the cost of the methods then making the good.

| method | aimed at | found | its good then sells at | a new works needs: when found | know-how 0.6 | know-how 1 | as a share of the going price | its smallest works, of the island's trade |
|---|---|---|---|---|---|---|---|---|
| Pottery manufactory | 1770 | 1771 | 7.85 | 8.76 | 7.40 | 6.39 | 1.12, 0.94, 0.81 | 15% |
| Water-frame cotton mill | 1771 | 1771 | 14.40 | 14.19 | 12.26 | 10.83 | 0.99, 0.85, 0.75 | 2% |
| Water-powered woollen mill | 1775 | 1775 | 13.94 | 14.60 | 12.59 | 11.09 | 1.05, 0.90, 0.80 | 1% |
| Edge-tool works | 1778 | 1783 (run) | 18.99 | 31.41 | 27.22 | 24.05 | 1.65, 1.43, 1.27 | 2% |
| Common brewery | 1785 | 1784 | 2.86 | 3.16 | 2.81 | 2.54 | 1.10, 0.98, 0.89 | 7% |
| Steam cotton mill | 1805 | 1804 | 7.21 | 8.40 | 7.57 | 6.93 | 1.17, 1.05, 0.96 | 4% |
| Gasworks | 1815 | 1813 | a new good | 5.66 | 4.98 | 4.47 | | 4% |
| Alkali works | 1815 | 1814 | 13.65 | 14.22 | 12.19 | 10.67 | 1.04, 0.89, 0.78 | 7% |
| Machine paper mill | 1815 | 1814 | 15.67 | 16.72 | 14.05 | 12.08 | 1.07, 0.90, 0.77 | 10% |
| Steam sawmill | 1825 | 1824 | 6.84 | 7.51 | 6.66 | 5.99 | 1.10, 0.97, 0.88 | 3% |
| Hot-blast furnace | 1830 | 1831 (run) | 7.63 | 8.15 | 7.51 | 7.01 | 1.07, 0.98, 0.92 | 1% |
| Sheet glass works | 1835 | 1832 | 26.93 | 33.25 | 27.81 | 23.80 | 1.23, 1.03, 0.88 | 10% |
| Machine-tool works | 1838 | 1839 (run) | 63.68 | 51.87 | 45.03 | 39.88 | 0.81, 0.71, 0.63 | 3% |
| Iron shipyard | 1845 | at 1845 prices | 454 (the older yard) | 486 | 400 | 337 | 1.07, 0.88, 0.74 | no market without fleets |
| Cement works | 1850 | 1849 | 4.75 | 4.92 | 4.34 | 3.89 | 1.04, 0.91, 0.82 | 3% |
| Continuous kiln | 1856 | 1856 (run) | 3.25 | 3.32 | 2.91 | 2.60 | 1.02, 0.90, 0.80 | 1% |
| Shops, offices and warehouses | 1860 | 1861 (run) | 13.09 | 21.11 | 16.60 | 13.38 | 1.61, 1.27, 1.02 | 1% |
| Contract builders | 1862 | 1861 | 7.81 | 9.17 | 7.87 | 6.89 | 1.17, 1.01, 0.88 | 0% |
| Tool factory | 1864 | 1857 (run) | 9.92 | 9.48 | 8.43 | 7.63 | 0.96, 0.85, 0.77 | 6% |
| Ammonia-soda works | 1872 | 1872 | 4.51 | 4.87 | 4.29 | 3.86 | 1.08, 0.95, 0.86 | 5% |
| Tank glass furnace | 1874 | 1873 | 9.39 | 9.88 | 8.52 | 7.51 | 1.05, 0.91, 0.80 | 7% |
| Engineering works | 1876 | 1880 (run) | 13.11 | 17.12 | 15.21 | 13.75 | 1.31, 1.16, 1.05 | 1% |
| Steam roller mill | 1878 | 1877 | 0.74 | 0.80 | 0.72 | 0.65 | 1.09, 0.98, 0.89 | 1% |
| Steel shipyard | 1882 | at 1882 prices | 114 (the older yard) | 115 | 97 | 84 | 1.01, 0.85, 0.74 | no market without fleets |
| Wood-pulp mill | 1885 | 1884 | 3.50 | 5.05 | 4.36 | 3.85 | 1.44, 1.25, 1.10 | 3% |

Read it as the brief asks. In the first of the three ratios (the year it is found, in the town that finds it) every method on a new good is level or dearer, by up to a quarter; in the last (know-how spread) every one is cheaper, by 4 to 25 per cent, but the pulp mill, which waits for a cheap log (section 6.2). The thinnest pass is the steam cotton mill's (0.96), which is why its first works comes nine years after it is found. The gasworks has no older method to beat, only coal in the grate. For the seven methods on base goods the runs of section 11 are the test, not this table: the edge-tool works cannot enter at 1783's steel price and is built as crucible steel cheapens (half of all tools by 1814); the shops and the engineering works enter as their know-how spreads; and one method, the machine-tool works, undersells the going price on the day it is found (0.81). It is left as run. Machinery's price in that year is a shortage price (333 in 1830 with the game's own content), the works still takes eleven years from discovery to half of the machinery because plant is built a works at a time, and it is the hinge every later works waits on; if it should be slowed, `A0` 0.85 is the number.

### 8.4 Where each works stands and what it looks like from above

"Stands" is what the engine will do with the recipe: a works goes where its output's carting and its inputs' delivered prices leave the best return. Shapes are in the viewer's vocabulary.

| works | where it stands, and why | from above |
|---|---|---|
| Fishing grounds (land) | coast cells, densest where a harbour town or a railway is near | boats drawn up on a strand, net racks, a smokehouse, a huddle of cottages; smacks offshore, funnels after 1880 |
| Dairy and gardens (land) | a ring two to five cells wide round every town, stretching along the railways after 1840 | narrow bright strip plots, orchard dots, cattle in small hedged closes; glasshouses late |
| Masons and carpenters | in every town, nearest the quarry road | a yard of dressed stone and baulks, a lime pit, a hoist; scaffolding on whatever the town is building |
| Timber-frame builders | towns with woods behind them and no stone near | a framing yard: oak frames laid out on the ground, a saw pit, bundles of laths |
| Bricklayers | towns on the clay, far from any quarry | brick stacks under thatch hurdles, a mortar mill turned by a horse |
| Contract builders | the growing edge of large towns within reach of a brickfield | a builder's yard with a steam saw and a joiners' shop, and beyond it whole streets in scaffolding at once |
| Saw pit | at the edge of the woods and in every builder's town | a trestle over a pit, a log on it, a top sawyer and a bottom sawyer; planks stacked to season |
| Steam sawmill | where logs arrive: river mouths below the woods, timber quays, railway yards | a long open-sided shed, a boiler chimney, a log pond or log stacks on one side and sawn deals in ranks on the other, a sawdust heap |
| Lime kiln | at the foot of the quarries, on the side nearest coal | a squat stone drum built into a bank with an arched draw hole, white spoil below, smoke from the top |
| Cement works | limestone nearest coal, on a quay, a canal or a siding | a row of bottle kilns and a long grinding shed, grey dust over everything |
| Brewhouse | in every market town: beer is too heavy to cart far | a malt kiln with a cowl beside an inn yard, casks under a lean-to |
| Common brewery | the largest towns only, and towns a canal or railway has brought within reach | a tall brewing tower with a louvred roof, a vat house, maltings with pyramid kiln roofs, drays in the yard |
| Country pottery | on clay near a town | one bottle oven beside a drying shed, a flooded clay hole |
| Pottery manufactory | where clay lies beside coal; the works cluster and stay clustered | a courtyard works with four to eight bottle ovens of unequal height, shard heaps, a canal wharf if there is one |
| Gasworks | the low edge of every town of four thousand people or more, on the side the coal arrives | a retort house with a row of stub chimneys and round gasholders that rise and fall with the stock; one more holder for each tier of capacity |
| Edge-tool works | towns of smiths near the steel, on a stream once sites are read | a low range of grinding hulls along a dam, small wheels, a tilt-hammer shed |
| Tool factory | steel towns | saw-toothed sheds and one hammer shop with a row of stubby stacks |
| Hot-blast furnace | on the coal measures, where the ironstone follows the coal | taller iron-banded stacks with domed stoves beside them, an incline to the furnace top, a cast house, slag banks |
| Machine-tool works | large towns with iron and coal within reach: it sells to every other works | long saw-tooth north-light roofs, a yard crane, castings stacked outside |
| Engineering works | steel towns on a railway | parallel erecting bays, a foundry cupola, engines on wagons in a siding |
| Continuous kiln | on clay beside a railway or canal, near the towns that are building | an oval ring of arches round one very tall chimney, a flooded clay pit, a tramway, brick stacks |
| Shops, offices and warehouses | the middle of towns large enough to have gas | a four-storey block with a glazed ground floor, or a street of shops roofed in iron and glass; drawn as part of the town centre |
| Soap boilers and ash burners | ash burners in the woods, boiling houses downwind of the towns | iron pots under an open shed, ash pits and cordwood stacks; in town a boiling house with a wide low chimney |
| Alkali works | where limestone and coal meet on water: three of heat to one of soda keeps it at the pithead | long lead-chamber sheds on legs, the tallest chimney on the map, grey waste heaps and dead hedges |
| Ammonia-soda works | on the limestone nearest coal and a railway, with its own lime kilns: it burns a third of the coal, so it can leave the pithead | a cluster of iron towers and pipe bridges, a row of lime kilns, white settling lagoons |
| Glass cone | where coal and sand meet: six of heat to one of glass | one brick cone eighty feet high, three times the height of its sheds; the tallest thing in 1750 after the church |
| Sheet glass works | coalfield towns within reach of the alkali works | long sheds with louvred roofs and a row of short chimneys, crates of sheet in the yard |
| Tank glass furnace | coalfield towns on a railway | two great gabled halls and a pair of tall regenerator chimneys |
| Paper mill | clean streams near towns: the rags come from the town | a stone mill with a wheel and slatted drying lofts |
| Machine paper mill | town edge, downstream | a long single-storey machine house, a boiler chimney, a rag store |
| Wood-pulp mill | where logs land: a river mouth below the forests, or a timber port | a log pond with a boom across it, a digester tower, a chimney, bales of white pulp |
| Windmill | open ground among the grain where no stream has a fall; low rises and the coast | a tapered tower with four sails turning and a boat-shaped cap |
| Water corn mill | a stream with a fall within carting of a market | a gabled mill astride a leat, a wheel on the gable end, a mill pond above |
| Steam roller mill | the biggest market with the cheapest grain: the quay once grain is imported | a six-storey brick block, a row of round silos, one chimney, a ship or a train alongside |
| Water-powered woollen mill | hill streams within reach of the wool; a cell holds only so many, so they string out along the valley | a narrow four-storey stone mill, a wheelhouse, a weir and a pond, a row of cottages, tenter frames on the hillside |
| Water sawmill | the first good fall below the woods, where the logs come down | a saw shed on a leat with a wheel, a log pond held by a boom, deals stacked to dry |
| Fustian weaving | port towns and the villages behind them | weavers' cottages with a long row of top-floor windows |
| Water-frame cotton mill | the nearest fast water to a cotton port | a plain many-windowed block in a narrow valley, a wheelhouse, a weir and leat, a row of workers' cottages |
| Steam cotton mill | between the cotton port and the coal, in the towns | a red-brick block with an engine house, a square chimney and a mill pond, a saw-tooth weaving shed beside it, terraces round it |
| Shipwright's yard | a sheltered shore with woods upriver | a slip with a hull in frame, a saw pit, a timber pond |
| Iron shipyard | the harbour nearest iron and coal | longer slips with plated hulls in red lead, sheer-legs, a platers' shed, an engine-shop chimney |
| Steel shipyard | the same harbours, larger | gantries over three berths, a fitting-out basin, a hammerhead crane |

Honest decoration follows from numbers the simulation keeps: a wheel on a works whose method has a water site, sails on one with a wind site, a chimney on one that burns heat, a gasholder for each tier of a gasworks, scaffolding in proportion to the builders' output, logs in a river that carries a drive, houses in the material of the town's builders.

## 9. Other inventions

| invention | aimed at | found after | what it does | stage, and what it needs |
|---|---|---|---|---|
| Liming | known in 1750 |  | farms, pasture, gardens: a tenth more output, for 0.012 of lime a unit | 4 |
| Fishing smacks | known in 1750 |  | the fishery: half as much again, for 0.02 of sawn timber, 0.002 of cloth a unit | 4 |
| Embanking and draining | 1790 | grain x47 | farms, gardens: a fifth more, and no flood years, for 0.004 of sawn timber, 0.004 of bricks a unit | the generator's `floodplain`: it lifts only cells that flood, and takes their flood years away |
| Deep mining | 1815 | coal 50,000,000, machinery 30,000 | iron mines, collieries: half as much again, for 0.0003 of machinery, 0.015 of sawn timber a unit | 4 |
| Tile drainage | 1843 | grain x175, machinery 130,000 | farms, gardens: 15 per cent more, for 0.012 of bricks a unit | 4 |
| Town waterworks | 1848 | iron 3,000,000, machinery 140,000 | a town past the size its streams can serve builds a reservoir and a main, and its death penalty is halved | reservoirs, and a hook in the people rule's town deaths |
| Artificial manures | 1850 | chemicals 1,500,000, grain x200 | farms, pasture, gardens: a quarter more, for 0.004 of chemicals a unit | 4 |
| Ocean steamers | 1866 | machinery 550,000, steel 3,100,000, coal 170,000,000 | ocean freight falls from 0.25 to 0.10 | 3; foreign trade, whose ocean freight it cuts from 0.25 to 0.10 |
| Mechanical haulage | 1868 | steel 4,600,000, coal 190,000,000 | iron mines, collieries: half as much again, for 0.0003 of machinery, 0.002 of steel a unit | 4 |
| Steam trawling | 1882 | fish 120,000,000, steel 7,300,000, coal 270,000,000 | the fishery: double, for 0.08 of coal, 0.0002 of machinery, 0.002 of steel a unit | 4 |

Liming alone would take about 2,500 units of lime a month in 1770, twice what the builders take; pit props about 4,000 units of sawn timber a month by 1850. The port's steam pumping, wagonways, railways, improved husbandry and farm machinery stay as they are (farm machinery is found about 1843 in the runs, bought where it pays). The quarries' machines are already in the machines rule and need no invention.

Canals are not in this table or in the overlay. The mechanism on `night/canals` finds them on its own clock (`discover_coal` and `discover_stone` in its parameters, 3,000,000 units each, by the same hazard), and a second invention in the content would be a second clock that disagrees. One thing for its builder: the stone threshold runs on stone dug, which the building trades cut by half or more, so with this overlay canals come five to ten years later than tuned. Halve `discover_stone`, or count coal alone.

## 10. Engine changes, in the order to build them

Each with what it unlocks and the smallest honest version of it. "Honest" means the map never shows what the simulation does not know: where a smaller version would put a watermill on a dry hill, the content waits. The stage numbers are the overlay's.

**Stage 1: an overlay loader; a theta for each need; `group` on a method.**
- *Unlocks*: 9 goods, 28 methods, the two needs, the building trades, household goods, gas, the whole late century on the base goods.
- *Smallest version*: (a) `content_from_str` takes the base text and an overlay text and merges by name as the overlay's header says, keeping entries with `stage <= N`; (b) `decay` and `stock0` become fields of a good, defaulting for the sixteen to the two tables they are in today (`params.rs`, and the `guess` array in `sim.rs`, which also seeds the opening trade that a year's room is reckoned on); (c) `service_mix` takes a width for each need from `[services] theta` in place of the one `P.logit_theta`, and the loader applies `[[services.change]]` blocks; (d) `group` on a method maps it onto one of the families' six trades: `Group::of_method` (`families.rs:80`) is a match on sixteen names today and returns none for any other, so without this no family, the player's included, can own a builder's yard, a brewery or a gasworks; (e) `util::pairwise_sum_by` keeps its terms on the stack only up to sixteen (`util.rs:41`) and every sum over goods in the monthly step goes through it: raise the buffer to 64 before timing anything; (f) the observer's lists of who serves shelter and warmth are by name (`observer.rs:26-31`): take them from the needs. `game_content()` loads the overlay after `machines_content`, or drops `machines_content`, whose numbers the overlay now carries.
- *Check before trusting it*: that nothing else indexes sixteen goods or six needs (`observer.rs:22-57`, `record.rs`, and the viewer's panels). The port and the golden test are untouched, since the port never loads the overlay.
- *Worth doing with it, not needed*: `traded = false` on a good, for services, building work and gas: it takes three goods out of the monthly trade pass and the yearly delivered-price pass, and stops the engine making carters and road traffic out of building work moved from the yard to the market. And the seed plan: `seed_firms` taking its plan from the methods marked `seed = true`, so that 1750 opens with every trade in place. Without it the 1750 trades are founded in the warm-up decade, a works a year in each town and then 15 per cent of the trade a year, and a town's builders reach its demand in about eight of the ten years.
- *Set with it*: nothing. No parameter of the game's rules needs to move (section 11).
- *Without (c)*: do not load the household goods: at 0.25 one of the four takes the need.

**Stage 2: sites, the new island's fields, more land uses, log driving.**
- *Unlocks*: 3 goods (flour, fish, produce), 2 sectors, 6 methods, and a site for two methods of stage 1 (the paper mill and the edge-tool works); the map's reasons; most of what Stella asked for under water and logging.
- *Smallest version*: (a) fields. The generator on `night/land` has `power` (kW a cell: flow times fall, summed over its streams), `discharge`, `floodplain`, `marsh`, lakes and shores, estuary heads, and one prevailing wind for the island. Water power is `power` as it stands. Three fields are derived, because it has none of them: `wind`, 0 to 1, from a cell's height above its neighbours and the lee of the one wind vector; `harbour`, 1 at an estuary head or on a lake shore; `fish`, on coast and lake-shore cells, weighted by the discharge of the nearest river mouth. There is no salt, and the design no longer asks for it. (b) the site rule. `site = { field = "power", kw = N }`: the works may stand only on a cell with power above a floor; the eight best such cells of the town's catchment join its candidates; and the works on a cell together draw no more than a share of its power, each drawing `kw` for every usual size of its capacity. That share (`site_share`, one number, the part of a stream's fall that weirs and leats can take; start at 0.15) is to be set so that the streams within a day's carting of the woods and the sheep are full by about 1820. `site = { field = "wind", weight = 1 }`: productivity is multiplied by `clamp(field / 0.5, 0.3, 1.6) ^ weight`. Weight 0 (the harbour) means only "must stand there". (c) land-use codes derived from the number of sectors, and a sector flag `ag` in place of the names "farm" and "pasture" (`land.rs:17-24`, `sim.rs:405`, `econ.rs:150`, `harvest.rs:137`, `rail.rs:276`); `yearly` counts sectors 0 to 2 as agriculture, and gardens and fisheries must join them or the farm share of hands falls by an accident of numbering; (d) log driving, as in section 6.2.
- *Why a capacity in kilowatts, and not the first draft's "eight usual sizes a cell"*: a corn mill's usual size and a woollen mill's are not the same wheel, and the critique showed the count did not bind: six good cells held all the sawn timber the island wanted until 1870, so the saw pit died in 1750 and the steam sawmill was never built.
- *Also worth doing here*: add to the candidates the eight cells where heat is cheapest, for a method that burns more weight than it carries in (today a glass cone finds the coal by luck; map's finding).
- *Without it*: stage 1 stands. Do not load stage 2's methods without it: a watermill would stand anywhere. Log driving can follow the sites; until it does the pulp mill is found and not built.

**Stage 3: foreign trade at the ports.**
- *Unlocks*: 2 goods, 3 methods, ocean steamers; cheap food late; timber for the builders; exports, which are the one way a made good employs more hands as it gets cheaper.
- *Smallest version*: the engine designer's, without tariffs and without ships: for each port town and each good in `[world]`, import when the town's price is above the world's price times the exchange factor plus freight, export when below, by the rule towns already trade by; a foreign account that `money_total` counts; an exchange factor that moves a little each month toward balance; a depth for each export; imports counted as handled for discovery.
- *Why before improvements*: the machines rule already gives the land its rising productivity, which was the larger half of what "improvements" were for. Trade gives the first wave its cotton and the last its bread and its timber, and none of them exists otherwise.
- *Without it*: 28 goods; clothing is wool; the last wave rests on home methods. Everything else stands.

**Stage 4: improvements as data.**
- *Unlocks*: liming, drainage, manures, deep mining, haulage, smacks and trawlers; farms, mines and fisheries as customers of lime, bricks, chemicals, sawn timber and steel.
- *Smallest version*: `machines::uses` builds its list from the inventions that carry `sectors`, `inputs` and `boost`, with the adoption rule it already has (none where the gain equals the cost, all where it is seven times the cost); the quality of machines multiplies only uses that buy machinery; pumps and wagonways stay bought the port's way, as now. `machines::SLOTS` is 2 a sector today and the farm would have five (liming, drainage, embanking, manures, machinery): the stock table is `T x NS x SLOTS`, so this changes saved state, not only `uses`.
- *Without it*: lime keeps the builders, chemicals keep households and glass, sawn timber keeps the builders. The design holds; the farm has only the steps it has today.

**Stage 5: fleets and upkeep.**
- *Unlocks*: ships and three shipyards; a rail, sleeper and locomotive market that outlives the building rush, which is also what the steel rail mill waits for.
- *Smallest version*: (a) port merchants buy ships in proportion to the weight they move by sea, and a month's sea trade is scaled by how well they were supplied; (b) each way (railway, later canal) buys each year a few per cent of what it was built of, in rails, sawn timber and machinery, from the nearest town.
- *Without it*: leave the ships good out. Nothing else depends on it.

**Water and canals**, on their own tracks: canals (built on `night/canals`, with their own clock; see the end of section 9); reservoirs, floods and town water as in section 6.3.

**Weighed and not asked for.** By-products (the gasworks' coke is netted in its coal). Power as a carrier. A smaller materials penalty for small and new works (history's two constants): the successors here pass without it, and it would move the port. A steeper discovery hazard (map: the power 4 in `tech.rs:35` to 8): untried, and now the most useful thing not asked for, since the hazard alone spreads a discovery over twenty-four years between the first seed in ten and the last. The rate itself was tried, and does not narrow the scatter (section 11).

What content needs which change, in one table:

| content | needs | works without it? |
|---|---|---|
| building work, sawn timber, lime, beer, pottery, gas and their 13 methods; the 7 methods on base goods | the loader | |
| soap and chemicals, glass, paper (8 methods); the two needs | theta per need | no: at 0.25 one of the four household goods takes the need |
| family ownership of the new works | `group` | yes, but no family can run them |
| 1750 opening with the new trades in place | the seed plan | yes: they fill in over the warm-up |
| flour (3 methods), water-powered woollen mill, water sawmill; the sites of the paper mill and the edge-tool works | site fields, with a capacity for water | no for the first five; the last two stand without a site until then |
| fish, produce | the derived fish field; land uses past seven; `ag` | no |
| pulp mill | log driving, or imported timber | no: it is not built at the island's timber price |
| cotton, cotton cloth (3 methods), ocean steamers; clothing at theta 1.0 | foreign trade | no |
| liming, drainage, manures, deep mining, haulage, smacks, trawlers | improvements as data | no: headlines only, so they wait |
| ships (3 methods) | fleets | no |
| canals; embanking; town waterworks | their own mechanisms | no |

## 11. What was run under the game's rules, and the thresholds

**The binary.** A copy of `isostate-core.exe` as built in the `night/core` worktree at 14:10 today: the game's rules with prices, Engel, machines, harvest, works, railways, carting and people all on. It takes `--content`, and it cannot take a seventeenth good. So what could be run is the part of stage 1 that lives on the sixteen base goods: the seven new methods (the new shops without their gas and paper), the three larger usual sizes, and the converter's and steel rail mill's thresholds. The control is the game's own content. Sixteen seeds: 1 to 8, on which the thresholds were set by hand in two passes, and 9 to 16, which were run once at the end. `.claude/night/tmp/synth2/final16.py` builds the content from the overlay's own numbers and prints these tables.

Sixteen seeds, mean with the least and greatest seed. Real wages and output per head are the year's mean over 1750's.

| | the game's own content | with the overlay's base-goods part |
|---|---|---|
| real wage 1800, 1750 = 1 (the model's basket) | 1.57 [1.44 .. 1.92] | 1.58 [1.43 .. 1.81] |
| 1850 | 1.67 [1.25 .. 2.00] | 1.99 [1.58 .. 2.81] |
| 1900 | 2.71 [2.15 .. 3.17] | 3.36 [3.05 .. 4.01] |
| real wage 1850, chained index | 2.32 [1.85 .. 2.86] | 2.62 [1.88 .. 3.74] |
| 1900, chained index | 4.28 [3.80 .. 4.75] | 5.44 [4.72 .. 6.48] |
| output per head 1900 | 2.74 [2.15 .. 3.15] | 3.37 [3.06 .. 3.82] |
| share of people in towns 1900 | 0.42 [0.39 .. 0.44] | 0.40 [0.38 .. 0.42] |
| farm share of hands 1900 | 0.25 [0.22 .. 0.29] | 0.25 [0.24 .. 0.26] |
| works' share of hands 1900 | 0.48 [0.45 .. 0.50] | 0.46 [0.44 .. 0.48] |
| mines', quarries' and pits' share of hands 1900 | 0.27 [0.25 .. 0.30] | 0.29 [0.28 .. 0.31] |
| works standing 1900 | 692 [567 .. 876] | 634 [478 .. 786] |

When each invention is found and when it takes over, with the overlay. "Never built" counts seeds in which it made nothing by 1900.

| method | aimed at | found (mean, least .. greatest) | in seeds | makes half of its good's output by | never built in |
|---|---|---|---|---|---|
| Edge-tool works | 1778 | 1783, 1765 .. 1826 | 16 of 16 | 1814 (15 of 16) | 0 |
| Tool factory | 1864 | 1857, 1836 .. 1879 | 16 of 16 | 1864 (16 of 16) | 0 |
| Hot-blast furnace | 1830 | 1831, 1821 .. 1856 | 16 of 16 | 1849 (16 of 16) | 0 |
| Machine-tool works | 1838 | 1839, 1825 .. 1872 | 16 of 16 | 1850 (16 of 16) | 0 |
| Engineering works | 1876 | 1880, 1860 .. 1895 | 16 of 16 | 1888 (8 of 16) | 2 |
| Continuous kiln | 1856 | 1856, 1836 .. 1874 | 16 of 16 | 1869 (16 of 16) | 0 |
| Shops, offices and warehouses | 1860 | 1861, 1825 .. 1898 | 16 of 16 | 1877 (14 of 16) | 1 |
| Air-blown converter | 1856 | 1858, 1842 .. 1900 | 16 of 16 | 1865 (15 of 16) | 1 |
| Steel rail mill | 1866 | 1871, 1843 .. 1894 | 15 of 16 | never | 3 |
| farm machinery |  | 1843, 1827 .. 1863 | 16 of 16 | never |  |
| railway |  | 1826, 1809 .. 1849 | 16 of 16 | never |  |

Prices and output, mean of the sixteen seeds.

| monthly, mean of sixteen seeds | 1790 | 1830 | 1850 | 1870 | 1899 |
|---|---|---|---|---|---|
| price of iron, the game's own | 14.0 | 9.7 | 8.7 | 7.7 | 7.9 |
| price of iron, with the overlay | 12.7 | 7.1 | 5.4 | 3.6 | 2.2 |
| steel, the game's own | 72.3 | 22.9 | 20.7 | 16.9 | 11.3 |
| steel, with the overlay | 95.6 | 22.4 | 19.6 | 13.1 | 5.4 |
| tools, the game's own | 15.2 | 38.1 | 51.3 | 31.2 | 31.5 |
| tools, with the overlay | 14.9 | 12.9 | 10.7 | 6.8 | 2.7 |
| machinery, the game's own | 222 | 333 | 121 | 67.2 | 72.0 |
| machinery, with the overlay | 197 | 124 | 48.0 | 21.2 | 8.6 |
| bricks, the game's own | 3.7 | 4.1 | 4.1 | 3.3 | 3.7 |
| bricks, with the overlay | 3.7 | 4.0 | 3.4 | 2.5 | 1.4 |
| services, the game's own | 11.3 | 12.0 | 12.9 | 13.5 | 17.1 |
| services, with the overlay | 11.9 | 12.1 | 12.5 | 12.1 | 10.4 |
| steel poured, the game's own | 154 | 4,269 | 6,970 | 14,908 | 46,040 |
| steel poured, with the overlay | 447 | 4,286 | 6,602 | 17,566 | 73,357 |
| tools made, the game's own | 2,528 | 599 | 470 | 691 | 1,251 |
| tools made, with the overlay | 1,884 | 956 | 2,313 | 7,996 | 20,247 |
| machinery made, the game's own | 46.3 | 247 | 671 | 1,620 | 2,831 |
| machinery made, with the overlay | 47.3 | 344 | 1,228 | 4,927 | 9,984 |
| bricks made, the game's own | 13,168 | 41,661 | 68,821 | 119,764 | 183,055 |
| bricks made, with the overlay | 12,902 | 43,792 | 76,742 | 187,117 | 746,745 |
| stone dug, the game's own | 55,608 | 109,905 | 212,005 | 479,077 | 1,046,444 |
| stone dug, with the overlay | 55,194 | 125,489 | 266,292 | 543,691 | 710,961 |

What this says:

- **Every wave arrives, in order, on every seed.** The seven methods are each found in all sixteen seeds, on average within seven years of the year aimed at, and all but the engineering works take half of their good's output within ten to thirty years. The engineering works was too weak as first written (never built in eight seeds of sixteen); with the recipe now in the overlay it is built in fourteen and makes two thirds of the machinery by 1899.
- **The century keeps unfolding.** The real wage is level with the control to 1800, a fifth higher in 1850 and a quarter higher in 1900, and the worst seed with the overlay (3.05) is close to the best without (3.17). Iron ends at 2.2 against 7.9, machinery at 9 against 72, bricks at 1.4 against 3.7, and services, which nothing had ever made cheaper, at 10 against 17.
- **It does nothing for the pits.** Mines, quarries and clay pits hold 29 per cent of all hands in 1900 with the overlay and 27 without; the control digs a million units of stone a month by 1899 and the overlay 700,000 (kiln bricks take some of shelter). The towns' share is 0.40 against 0.42. This is the part of Stella's fourth point that the base-goods methods cannot touch and the building trades are for; it is also the part that could not be run.
- **The steel rail mill is found and not built**, here and almost so in the control (2 per cent of rails in 1890 with the game's own content, where the machines page measured 86 per cent before the other mechanisms were merged). Rails rolled from cheap iron undersell it. It needs railways that wear out (stage 6); nothing in the content will fix it.

**Steel as an implement was settled by runs**: the overlay's sixteen-good part on seeds 1 to 8, with that one number changed.

| steel's efficiency as an implement | steel poured a month, 1790 / 1850 / 1899 | converter built in | 1900 real wage |
|---|---|---|---|
| 3.0 (as now) | 400 / 6,900 / 75,000 | 8 seeds of 8 | 3.38 [3.08 .. 3.60] |
| 2.4 | 250 / 3,700 / 49,000 | 8 | 3.22 [2.65 .. 3.61] |
| 2.0 | 26 / 1,600 / 8,600 | 3 | 2.33 [1.75 .. 3.00] |
| 1.6 | 0.3 / 0.05 / 0.06 | 0 | 1.81 [1.72 .. 1.97] |

At 2.4 the converter is still built on every seed, but the engineering works in only half of them, and the weakest seed is half a point lower. At 1.6 a unit of crucible steel costs over a hundred when it is first made and does a household less good than five tools at twenty, so none is bought, so none is made, so the crucible works never learn (their curve is used up in fifty units), and every later thing that waits on steel poured waits for ever: the edge-tool works, the machine-tool works, the converter, and the quality of every machine on the land. Under the port's rules the same content worked, because there a works is paid for output nobody buys and the steel piled up; the game's merchants stop buying into a glut. Two designs and the earlier runs all chose 1.6, and all were right about the port.

**Without the tools chain the machine-tool works starves.** The same content less the edge-tool works and tool factory (seeds 1 to 8): tools cost 91 in 1890 where they should cost 3, machinery 39 where it should cost 10, and the 1900 wage is 3.06 [1.72 .. 3.51] against 3.38 [3.08 .. 3.60] with them. The smithies die as households turn to steel, and the machine-tool works' 0.15 of tools a unit then comes from a trade of a few hundred units a month.

**The discovery rate was tried and left alone.** With `discovery_rate` at 0.5 in place of 0.18 and the same thresholds, everything is found about thirteen years sooner and the spread across seeds is no narrower (the converter: 29 years between the first seed and the last at 0.18, 27 at 0.5; the machine-tool works 21 and 22). The earlier runs under the port's rules had found it narrower. The thresholds here are for 0.18.

**The thresholds.**

- *On base goods, for the seven methods and the converter*: set on seeds 1 to 8 under the game's rules. A discovery comes about five years after its last threshold is met when that threshold is in something that grows fast (machinery, steel), and several years before when it grows slowly (services), because the hazard is already 4 per cent a year at three quarters of the way.
- *On base goods, beside a threshold on a new good* (iron, steel and machinery; coal only as a second guard, in plain units; grain as "xN", since stage 2 lowers 1750's grain and the whole path with it): 0.85 of the same runs' cumulative output at the date the own-good threshold is set for, so that they guard and do not bind. The inventions with no new good of their own to count (gasworks, deep mining, mechanical haulage, ocean steamers, town waterworks) take iron, steel or machinery as their clock, at the runs' output. Hot blast's coal and iron, which ran as x500 and x270, are written as the median seed's plain units. They will all move when the new goods change what the island makes.
- *On new goods*: fitted to the reckoning at the game's prices (`fit.py`): the good's cumulative output zero to five years before the year aimed at, moved until the engine's hazard puts the median discovery within three years of the aim. Section 16 has the table. Late is the dangerous side: a threshold never reached is the port's converter. No brick, stone or timber threshold is used for anything on a new good, because the building trades change those three paths most.

**The fitter** for real runs is described in section 17.1.

## 12. What was left out, and why

- **Pig iron, puddling, yarn, coke, salt as a good or as a site field, hides and leather, flax, candles as a good of their own.** Each has one buyer or adds a step nobody sees.
- **Tea and sugar.** No works on the island. If trade feels thin on the import side it is the first thing to add, as a thirty-second good at `table = 2.2`.
- **Open hearth, steam pottery, ring mill, regenerative gasworks, brine works, salt pans, a third brewery.** A cost falls and nothing moves.
- **Meat as its own sector.** Folded into produce.
- **A "light" need.** Gas would be alone in it, and the next century needs one household energy need, not two.
- **Machinery as a running input or as upkeep.** Measured to turn a shortage into a crash; upkeep that never gates output is sound and can come with fleets.
- **Machine tools and locomotives as goods.** They are methods that make machinery.
- **Recalibrating food** (subsistence 1.3, or a share of 0.10 at every income). The Engel rule is in.
- **Water as a good; power as a good.** Section 6.3.

## 13. After 1900

The tree is the first half of a longer one. Nothing in it should have to be undone; this is where each chain is expected to go, and what it will feed.

| chain | 1900 to 1950 | 1950 to 2050 | what it feeds in the labour story |
|---|---|---|---|
| Fuel and light | electricity from coal and from the water-power sites; gas for cooking | oil, gas from the ground, nuclear, wind on the old windmill ground, sun | one need with many carriers; the works that make them need ever fewer hands |
| Coal | peaks, then pits close seam by seam (the mines rule's depletion) | gone but for power stations, then gone | the first great trade to lose its hands to machines and then its market |
| Iron and steel | open hearth and alloy steels, by quality if the engine learns to tell it | basic oxygen, mini-mills on scrap; fewer, larger works | output up, hands down by nine tenths |
| Engineering | motor cars (a good in implements, then a need of its own, travel), electrical machinery | electronics, then computing as a capital good bought by every trade | the good that takes over tasks: this is where "machines take the work" enters as content |
| Tools | power tools, appliances (into household goods) | | |
| Cloth | rayon and synthetics from the chemical works | imported; the mills close | a trade that leaves the island whole |
| Building | steel frame, concrete, council estates | system building, then little change | the slowest trade to mechanise: a floor under labour's share |
| Timber | pulp and newsprint, plywood, plantations | board, packaging | |
| Soap and chemicals | dyes, fertiliser from the air, explosives | plastics (into household goods), pharmaceuticals, oil refining | few hands from the start |
| Glass, pottery, paper | bottles by machine, sanitary ware, packaging | float glass; paper falls with screens | |
| Flour, beer, produce, fish | canning, refrigeration, branded food (into "table") | supermarkets; restaurants as a service | food's share keeps falling; the hands move to serving it |
| Shops, offices and warehouses | department stores, typists and telephones | self-service, then computing in the office, then models that do the clerk's work | the largest employer, and the one the last wave is about |
| Water | hydro dams on the water-power sites, cities' reservoirs | flood defence, irrigation as the climate dries | |
| Canals, railways | lorries take the short haul | motorways on the canals' lines | carters were the first hands the engine counted and freed |
| Ships | steel, then welded; the yards peak about 1950 | containers; the yards close | |
| Land | tractors, then the land sheds nearly all its hands | | rent, and who owns the land, stay when the hands have gone |

Two things in this design are there for that half. The needs are few and wide, so that a new carrier or a new good enters by price as the old ones did. And the improvements rule is the form in which automation should arrive: a thing a trade buys where it pays, which raises output per hand and whose maker's own hands are few. Who owns the works that make it is then the question the game is for.

## 14. Risks, and what I am least sure of

Section 16 lists what the critique settled and what it left open; section 17 says what to look at in the first sweep. In order of how much hangs on each:

1. **Nothing with a new good has run.** The base-goods part ran under tonight's rules on sixteen seeds. Everything else is reckoned, now at the game's prices and not the port's, with the wage and the base goods' prices held at what a run without the new trades gave. The new trades want 18 to 21 per cent of all hands (builders half of that), so wages will be bid up early, and the reckoning cannot say by how much.
2. **Timber.** The building trades and the sawmills want three times the timber the run cuts by 1870 and six times by 1899 (section 5). Timber, sawn timber and building work will all be dearer than reckoned, the forests will be cut back, and until stage 3 lands timber at the ports nothing relieves it.
3. **Gas hangs on one number.** At 4.5 it is a fifth of fuel and light; at 3.5 it is 3 per cent and at 5.0 a third. The decay and the merchants' stocks act on it like a tenth on its cost, which the reckoning leaves out.
4. **Thresholds on new goods** are the reckoning's output at about the year aimed at, with guards in iron, steel and machinery from the runs. They decide when half the tree arrives. Plain numbers also scale with the island: a generator that makes a larger or richer island brings every one of them forward.
5. **Steel at 3.0 is a wart the game leans on.** Households buy bar steel for nine tenths of their implements, and that is what pours the steel that improves the land's machines. It is left because the runs say it must be. The honest repair is in the engine (let the machines' quality count machinery made, or let a new works sell its first output on a promise as the port does), after which steel can be re-rated and the tools trade can be the size it was.
6. **The building trades move the island's largest flows.** Stone is halved from 1750 and falls to a quarter; a tenth of all hands become town builders; brick is not bought by households at all, so the brick trade is thin and jumpy from 1750 to 1850 (a pottery manufactory's plant is a month of the island's bricks). The urban share should rise by several points, which the Engel and people pages did not have when they were tuned.
7. **Cotton against wool, and the world's prices.** Raw cotton is priced at twice the `[world]` path, a guess; at clothing's new width cotton already has a fifth of clothing in 1770, as hand-woven fustian, which may be too much; and woollen cloth keeps a quarter in 1890 only if a run agrees. The world's prices were guessed twice by their author and were wrong the first time. Sweep the two together.
8. **Flour between 1850 and the roller mill.** The old mills' hands cost more each decade against grain that halves after 1850: flour is a third of food in 1850, a fifth in 1870, and two fifths again by 1890 once roller mills grind. If the dip is deeper in a run the mills will close before their successor comes. And a corn mill's costs are still nine tenths grain: in a dear year its margin goes and it stops, households eat grain instead, and only a run can say whether that is graceful.
9. **The sector `A` of fishing and gardens** is a first guess, with the aim (an eighth of food each) written beside it. Stage 2 takes a third of the grain trade from 1750, which re-bases the farm sector the harvest and people pages were tuned on.
10. **The stream's capacity** is a new rule with one unmeasured number, and the kilowatts of each mill are historical guesses against a field nobody has yet summed.
11. **Comfort's richer share** (0.28 for 0.36) trades shop hands for made goods at high incomes.
12. **Cost of running.** Thirty goods make the monthly trade loop and the yearly delivered-price pass about 1.9 times the work; 56 methods make the yearly entry pass about three and a half times longer; the new trades add works in every town. The critic's guess for the whole tree is 2 to 2.5 times today's 12 ms a simulated month, once the sixteen-term buffer is fixed. `traded = false` gives some of it back.
13. **Not designed here**: the families' lists for the new trades, the viewer's new silhouettes and ground textures, the observer's news for new goods.

## 15. Files

- `.claude/night/reports/game-content-draft.toml`: the overlay, revised. It parses, every name in it resolves, and no entry asks for a good of a later stage; two stage 1 methods carry a site that is read from stage 2.
- `.claude/night/reports/content-design-critique.md`: the critique this revision answers, with its scripts in `.claude/night/tmp/critique/`.
- `.claude/night/tmp/revise/`: the revision. `design.py` and `extra.py` hold every number; `fitted.py` the thresholds, written by `fit.py`; `gen.py` prints the overlay and the tables; `gp.py` is the reckoning at the game's prices for those tables; `model.py`, `reckon.py`, `sums.py`, `demand.py`, `chains.py` and `atfound.py` are the critic's scripts with the new stage numbers and the overlay's path, and `out/check_*.txt` is what they print on the revised overlay; `mkreport.py` builds this file; `patch_r1.py` to `patch_r4.py` and `exp*.py` are the log of what was changed and tried.
- `.claude/night/tmp/synth2/`: the first draft's calculator and its sixteen-seed runs under the game's rules (`runs/F_fit`, `F_hold`, `G_fit`, `G_hold`, and the experiments: `S24`, `S20`, `S16` for steel re-rated, `NOTOOLS`, `T5` for the discovery rate, `Fb` for the engineering works), with `core.exe`, a copy of the `night/core` build.
- `.claude/night/handover/synthesis.md`: the log of the earlier runs under the port's rules.
- The three designs: `content-design-history.md`, `content-design-map.md`, `content-design-engine.md` and `content-design-engine.overlay.toml`.

## 16. After the critique

The critique (`content-design-critique.md`) redid the first draft's reckoning at the game's prices and read the code for what stage 1 needs. Its arithmetic was checked and stands. This is each of its findings, what was done, and what its own scripts say of the revised overlay (`.claude/night/tmp/revise/out/check_*.txt`; sections 4, 5 and 8 carry the tables).

| | the critique found | what was done | the rerun |
|---|---|---|---|
| 1 | Gas is 3 per cent of fuel and light, not 19 to 37; the ammonia-soda works and the tank furnace, which waited on gas made, are never found, and the shops come twelve years late; the first gasworks is 73 per cent of the island's gas trade | Gas rated 4.5 for 3.5; gasworks' usual size 150 for 500; gas's decay 0.15 for 0.5; gas taken out of the two thresholds, and the shops' gas threshold cut to a loose 1,000,000 | Gas is 20 per cent of fuel and light in 1850 and 21 in 1890; a first gasworks is 4 per cent of the trade; ammonia-soda found 1872, tank furnace 1873, shops 1864 in the reckoning (1861 in the runs, on services alone) |
| 2 | Builders lose shelter to heaped stone and kiln brick: 72 per cent in 1770, 23 in 1870; stone is cut three tenths, not "more than half" | Raw shelter re-rated in the overlay (stone 0.3, bricks 0.6, timber 0.4, sawn timber 0.6, lime 0.5); bricklayers' bricks 0.6 for 0.7; building work's decay 0.1 for 0.3 | Builders make 91 to 100 per cent of shelter (the low point is 1870, before the contract builders have the trade) and are 7.5, 12 and 10 per cent of hands in 1770, 1850 and 1890; households and the new trades want 30 per cent of the run's stone in 1770, 24 in 1850 and 9 in 1890; contract builders found 1861, first works 1866 |
| 3 | Thresholds on new goods late or never: roller mill never met, contract builders met 1883, steam cotton thirteen years late, water frame eleven, pottery nine; "xN" in coal spreads 25 to 32 years across seeds, machine paper's cloth 33 | Every threshold on a new good refitted to the reckoning at the game's prices (`fit.py`); coal after 1800 in plain units and only as a guard, with iron, steel or machinery beside it; cloth out of machine paper; hot blast's two "xN" written as the median seed's units | Every refitted invention is found within three years of its aim at the median (the table below); the shops four years late in the reckoning and one in the runs. The hazard still spreads each over about twelve years either side |
| 4 | Stage 3 halves the grain trade, and the tuning aims contradict the design's own food table | Fish 1.1 for 1.3, produce 1.4 for 1.8; the aim is now a share (an eighth of food each) | Food in 1770 is flour 42 per cent, grain 33, produce 13, fish 12; grain bought is 64 to 73 per cent of the run's, down a third |
| 5 | First inventions too big for their market: pottery works 50 per cent of the island's trade, machine paper 22, sheet glass 16 | Usual size 150 for the alkali works, sheet glass and machine paper; 100 for the pottery manufactory; 400 for cement | The largest first works is the pottery manufactory's, 15 per cent of the island's trade; sheet glass and machine paper 10, the rest under 8 |
| 6 | Stage 1 alone gives household goods to pottery: 12,500 potters by 1899, then five sixths of the trade lost when stage 2 loads | The old stages 1 and 2 are one: soap, glass and paper load with pottery, and theta for each need is a stage 1 change | Pottery is 19 to 24 per cent of household goods at every date |
| 7 | Cotton ends wool by 1870 to 1890 | Clothing's width 1.0 from the trade stage (the critique proposed 0.5, which its own reckoning showed still left woollen cloth at 5 per cent); cotton cloth 0.75 for 0.85 | Cotton has 47 per cent of clothing in 1850 and 75 in 1890; woollen cloth keeps 25 per cent, 55,000 units a month, a third of the run's; no wool is worn raw after 1890 |
| 8 | The water sawmill has a 44 per cent margin in 1750, takes nine tenths of sawn timber on the first day, and the steam sawmill is never built | Water sawmill timber 1.35 and labour 0.16; steam sawmill `A0` 0.85; the stream's capacity in kilowatts drawn, not usual sizes counted | The saw pit keeps a quarter of the trade to 1820; the steam sawmill is found 1824, built 1828 and has 94 per cent by 1850 |
| 9 | Corn mills are 97 per cent grain cost under harvests that move grain 8 to 12 per cent | Mill labour doubled and plant quadrupled to two months of flour; flour's base price 1.4. Not the critique's fallback of six times the labour: at the game's wages that prices flour out of food by 1850 | Recipe margins 18 and 20 per cent. Still nine tenths grain at the game's prices; only a run can test it |
| 10 | Stage 1 quietly needs: `stage` as a set; `group`; a larger buffer in `pairwise_sum_by`; one canal clock; less waste from decay | Stages renumbered in build order, so a ceiling serves; `group` and the buffer are stage 1 changes (section 10); the overlay's `canal` invention is deleted; decay as above | `chains.py`: no deadlock, no good without a maker but raw cotton, no reference to a later stage but two sites |
| 11 | The engine changes are not the smallest honest versions: the seed plan is not needed; the generator has no wind, fish, harbour or salt | The seed plan is optional; salt is dropped and the alkali works stand on coal and limestone; wind, fish and harbour are derived from what the generator has; `SLOTS` is named as a change to saved state | |
| 12 | Against the brief: four inventions undersell on the day they are found; no known method breaks even at its usual size at base prices; a wrong statement about the machines rule's inheritance; mills with two weeks of plant | Iron shipyard: iron 7, labour 6, `A0` 0.65. Hot blast and the kiln are level at the game's prices (1.07 and 1.02 of the going price). The machine-tool works (0.81) is left as run, and section 8.3 says why. The break-even test cannot be passed by any content; section 8.2 shows both margins. The statement is corrected (section 8.1). Mill plant is two months | Iron shipyard 1.07, 0.88, 0.74 of the older yard's price |
| 13 | Cost: 2 to 2.5 times, unmeasured | Nothing in the content; the buffer and `traded = false` are in section 10 | |

Two things were changed that the critique did not ask for. The roller mill was re-cut (grain 0.96, labour 0.002, `A0` 0.8, `b` 0.2): once the old mills have hands, a roller mill written at `A0` 1 undersold them by 15 per cent on the day it was found. And the ammonia-soda works and the tank furnace are aimed at 1872 and 1874, not 1875 and 1878, because a works that enters above the going price is built about five years after it is found, and the last wave has no years to spare.

When each invention is found, in the reckoning at the game's prices with the revised thresholds. For the methods on base goods the reckoning's median is beside the sixteen-seed runs' mean, as a check on the estimator:

| invention | aimed at | last threshold met | found (median) | one seed in ten before, and after | first works |
|---|---|---|---|---|---|
| Engine works |  | 1782 | 1784 | 1775, 1793 | (run) |
| Crucible steel |  | 1764 | 1768 | 1761, 1776 | (run) |
| Pottery manufactory | 1770 | 1768 | 1771 | 1764, 1779 | 1775 |
| Water-frame cotton mill | 1771 | 1768 | 1771 | 1764, 1780 | 1771 |
| Water-powered woollen mill | 1775 | 1771 | 1775 | 1767, 1784 | (run) |
| Edge-tool works | 1778 | 1769 | 1774 | 1769, 1782 | run: found 1783 |
| Common brewery | 1785 | 1783 | 1784 | 1774, 1794 | 1788 |
| Embanking and draining | 1790 | 1788 | 1787 | 1775, 1799 |  |
| Steam-powered mill | 1798 | 1795 | 1796 | 1785, 1807 | (run) |
| Steam cotton mill | 1805 | 1804 | 1804 | 1792, 1815 | 1813 |
| Gasworks | 1815 | 1815 | 1813 | 1799, 1826 | 1813 |
| Alkali works | 1815 | 1816 | 1814 | 1799, 1826 | 1816 |
| Machine paper mill | 1815 | 1814 | 1814 | 1802, 1825 | 1816 |
| Deep mining | 1815 | 1812 | 1813 | 1802, 1824 |  |
| Steam sawmill | 1825 | 1826 | 1824 | 1813, 1838 | 1828 |
| Hot-blast furnace | 1830 | 1827 | 1829 | 1817, 1839 | run: found 1831 |
| Sheet glass works | 1835 | 1835 | 1832 | 1815, 1845 | 1838 |
| Machine-tool works | 1838 | 1836 | 1837 | 1826, 1847 | run: found 1839 |
| Tile drainage | 1843 | 1840 | 1840 | 1829, 1853 |  |
| Iron shipyard | 1845 | never | never | , after 1899 | not built |
| Town waterworks | 1848 | 1844 | 1843 | 1830, 1855 |  |
| Cement works | 1850 | 1849 | 1849 | 1837, 1861 | 1851 |
| Artificial manures | 1850 | 1850 | 1850 | 1837, 1862 |  |
| Air-blown converter | 1856 | 1854 | 1853 | 1841, 1865 | run: found 1858 |
| Continuous kiln | 1856 | 1853 | 1855 | 1845, 1864 | run: found 1856 |
| Shops, offices and warehouses | 1860 | 1868 | 1864 | 1844, 1878 | run: found 1861 |
| Contract builders | 1862 | 1860 | 1861 | 1850, 1873 | 1866 |
| Tool factory | 1864 | 1865 | 1863 | 1848, 1875 | run: found 1857 |
| Steel rail mill | 1866 | 1870 | 1869 | 1853, 1881 | run: found 1871 |
| Ocean steamers | 1866 | 1862 | 1865 | 1855, 1874 |  |
| Mechanical haulage | 1868 | 1868 | 1866 | 1850, 1878 |  |
| Ammonia-soda works | 1872 | 1872 | 1872 | 1859, 1883 | 1877 |
| Tank glass furnace | 1874 | 1871 | 1873 | 1861, 1882 | 1877 |
| Engineering works | 1876 | 1880 | 1880 | 1866, 1890 | run: found 1880 |
| Steam roller mill | 1878 | 1878 | 1877 | 1867, 1891 | 1881 |
| Steel shipyard | 1882 | never | never | , after 1899 | not built |
| Steam trawling | 1882 | 1883 | 1879 | 1864, 1893 |  |
| Wood-pulp mill | 1885 | 1883 | 1884 | 1872, 1894 | not built |

The shipyards are never found because nobody buys ships until fleets are built. The pulp mill is found and not built at the island's timber price, by design.

**What remains uncertain.** Everything in section 14, and in particular what no reckoning can reach: what the new trades' hands do to the wage; what their timber does to the woods; where anything stands, which is half of what the design is for; whether the corn mills ride out a dear year; whether wool cheapens as it is abandoned. The numbers here are the ones that make the critic's reckoning hold. The first real sweep will move them, and section 17 says which first.

## 17. For whoever builds this

### 17.1 The order of work

1. **Stage 1, the plumbing.** The loader with a stage ceiling; `decay` and `stock0` on goods; theta for each need and `[[services.change]]`; `group`; the buffer in `pairwise_sum_by`; the observer's lists. Load the overlay at stage 0 first (nothing kept) and check the golden test and a game run are unchanged to the last digit; then at stage 1.
2. **Stage 1, the first sweep** (eight seeds, the game's rules). Look at 1750 before anything else: the builders' share of shelter, the price of lime and the builders' fill (the opening stock of lime doubles as the opening guess of its trade, and the masons' output is cut to the lime there is), the wage against the run without the overlay. Then the checks of 17.3.
3. **The fitter.** A short script, to be written as a tool. Each invention in the overlay carries `target`. Run `isostate-core sweep --rules game --content ... --seeds 1-8`; from `annual.csv` take each good's cumulative output about five years before each target, averaged over seeds (twelve times the sum of the yearly means); write the own-good thresholds from that and the guards at 0.85 of theirs; repeat two or three times with the change damped by half, because moving one discovery moves the rest. `facts.csv` gives `discovered_NAME` to score it. `.claude/night/tmp/revise/fit.py` is the same loop on the reckoning, and its `SPEC` says which threshold of each invention is the clock and which are guards.
4. **Stage 2.** The derived fields and the site rule first, on the new island; then land uses and `ag`; then log driving. Set `site_share` and the two sector `A`s by sweep, refit, and re-check the 1750 level of food against the people rule: flour, fish and produce make the same harvest feed more.
5. **Stage 3.** Foreign trade, with the world's prices swept beside clothing's width and raw cotton's landed price.
6. **Stage 4**, then **stage 5**. Reservoirs, floods and town water when the water mechanism is built; canals are already on their own track.

### 17.2 The numbers most likely to need tuning

In the order to try them.

| number | now | try | the sign that it is wrong |
|---|---|---|---|
| gas as fuel and light | 4.5 | 4 to 5.5 | gas under 10 per cent of fuel and light in 1860, or over 35; fewer than a third of towns over 8,000 people with a gasworks by 1850 |
| raw shelter (stone, bricks, timber) | 0.3, 0.6, 0.4 | stone 0.25 to 0.4 | builders under 85 per cent of shelter in any decade; or the 1750s short of shelter because the builders are not yet founded (then the seed plan, not the number) |
| thresholds on new goods | fitted to the reckoning | the fitter | any invention found more than ten years from its aim at the mean of eight seeds |
| wage and timber | not numbers of the overlay | | a 1760 wage more than a fifth above the run without the overlay; timber above three times that run's price before 1850. For the first, cut the builders' labour (0.25) a little; for the second, the contract builders' sawn timber (0.18) and the masons' timber (0.35) |
| water sawmill and `site_share` | timber 1.35, labour 0.16; 0.15 | | the saw pit gone before 1800 (the streams do not fill: lower the share); or no water sawmill ever built |
| corn mills' labour | 0.01 and 0.008 | 0.006 to 0.012 | flour under a fifth of food in any decade before the roller mill; or mills working at a quarter in a dear year while flour is short |
| fish and produce `A` | 20 and 17 | | each outside 8 to 18 per cent of food by service in 1770 |
| clothing's width; cotton cloth; raw cotton's landed price | 1.0; 0.75; the world's path | 0.5 to 1.0; 0.65 to 0.85 | cotton over a quarter of clothing in 1770 or over 90 per cent in 1890; woollen cloth under a tenth of clothing in 1899 |
| comfort's richer share | 0.28 | 0.28 to 0.34 | shops' and trades' hands falling as a share after 1850 |
| the machine-tool works' `A0` | 1 | 0.85 | machinery's price halving within five years of the discovery |
| the first inventions' usual sizes | 100 to 150 | | a first works closed within three years of opening in more than a quarter of seeds |
| beer's share of spending | 0.06, 0.07 | 0.04, 0.05 | beer taking more than a quarter of the grain crop |

### 17.3 How to tell from a sweep that the tree is working

Eight seeds or more, means across seeds. The years are the reckoning's and the runs'; a mean within ten years is working, and one seed in eight outside that is the hazard, not a fault.

*Which methods should be built by when* (`discovered_NAME` in `facts.csv`, `mout_NAME` in `annual.csv`):

| method | found about | first works by | half of its good's output by |
|---|---|---|---|
| Pottery manufactory | 1771 | 1780 | 1790 |
| Edge-tool works (run) | 1783 | | 1814 |
| Common brewery | 1784 | 1792 | 1800 |
| Gasworks | 1813 | 1818 | a works in most towns over 8,000 by 1850 |
| Alkali works; machine paper mill | 1814 | 1820 | 1830 |
| Steam sawmill | 1824 | 1832 | 1840 |
| Hot-blast furnace (run) | 1831 | | 1849 |
| Sheet glass works | 1832 | 1842 | 1850 |
| Machine-tool works (run) | 1839 | | 1850 |
| Cement works | 1849 | 1855 | 1865 |
| Continuous kiln (run) | 1856 | | 1869 |
| Converter (run) | 1858 | | 1865 |
| Tool factory (run) | 1857 | | 1864 |
| Shops, offices and warehouses (run) | 1861 | | 1877 |
| Contract builders | 1861 | 1868 | 1875 |
| Ammonia-soda works; tank glass furnace | 1872, 1873 | 1880 | 1888 |
| Steam roller mill | 1877 | 1885 | 1892 |
| Engineering works (run) | 1880 | | two thirds of machinery by 1899 |
| from stage 2: water-powered woollen mill; steam-powered mill | 1775; 1796 | | the saw pit and the water sawmill together still half of sawn timber in 1830 |
| from stage 3: water-frame mill; steam cotton mill | 1771; 1804 | 1775; 1817 | steam: 1830 |
| Steel rail mill; wood-pulp mill | 1871; 1884 | | not expected to be built: the first waits for stage 5, the second for a log drive or a timber quay |

*Which shares should cross when:*

- **Shelter.** Builders make nine tenths of shelter by 1760 and not under 85 per cent after. Stone dug is at most 60 per cent of the run without the overlay in 1800, and falls in absolute terms after about 1875, to under 300,000 units a month in 1899 where the game's own content digs a million. Bricks made pass stone dug in the 1880s.
- **Hands.** Builders are 7 to 12 per cent of all hands from 1760. Mines, quarries and pits, 27 to 29 per cent of hands in 1900 today, are under 20. Works' share of hands is over a half by 1900 (0.46 today) and the towns' share of people over 0.45 (0.40 to 0.42 today). All the new trades together are 15 to 22 per cent of hands.
- **Fuel and light.** Gas passes a tenth within twenty years of the first gasworks and is 15 to 25 per cent from 1850.
- **Food.** Food's share of household spending falls from about 45 per cent in 1770 to 27 in 1850 and under 15 in 1890 (the Engel rule; the overlay only gives it somewhere to go). From stage 2: flour a third to two fifths of food to 1850, over half by 1899; fish and produce 10 to 18 per cent each; grain bought two thirds of the run's.
- **Household goods.** No one of pottery, glass, paper and soap under 15 per cent of the need or over 35 at any date after 1800; each with at least three works on the island from 1770.
- **Clothing**, from stage 3. Cotton cloth passes woollen cloth between 1840 and 1865 and is not over 85 per cent in 1899.
- **The century keeps unfolding.** The real wage in 1900 is not below the 3.4 times 1750's that the base-goods part gave alone; no twenty-year stretch after 1790 without a rise; at least three inventions found in every decade from 1770 to 1890; iron under 3, machinery under 12, bricks under 2 and building work under 4 a unit in 1899.

*Signs of the known failures:* a good with one works or none on the island for ten years running (a thin market: look at its first invention's usual size); an invention found and making nothing fifteen years later, the steel rail mill and the pulp mill apart (the port's converter again: look at its entry test in section 8.3 at that year's prices); a first works closed within three years (too big for its market); the price of bricks doubling for a year whenever a large works is built (the thin brick trade before the kiln); mills idle in a dear year; timber's price running away.
