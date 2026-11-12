# dx-roads: who pays for the roads and bridges, and who is paid a railway's fares

Branch `night/o-dx-roads` (measuring code and a prototype; not for merging until Stella has read it). Asked on 12 November 2026, in Stella's words: "And who's paying for bridges and roads?" The answer today is nobody (`docs/BACKLOG.md`, note 21; pieces 15 and 16 of `docs/backlog/found-along-the-way.md`). This probe measures what the roads are and what the carting over them is worth, designs who should pay and who should be paid from the period's own institutions, prototypes the smallest whole version behind a switch, sweeps it, and says whether to build it.

Everything is on seeds 1 to 8 under the game's rules with six families, at main a6ff0ec. Probe: `crates/isostate-core/examples/probe_roads.rs` (`[--seeds 1-8] [--rules port|game] [--families N] [--land] [--set NAME=VALUE]...`), which runs the islands side by side and reads the state at each year's end. `[lo .. hi]` is the least and greatest island. Raw output: `.claude/night/reports/opus/dx-roads/` (`base-game.txt`, `base-game-land.txt`, `proto-game.txt`, the first run of the prototype with a mistake in it, `proto-game-v1-navigable-tolled.txt`, and the sweeps `game`, with the switch off, and `game-roads`, with it on). The prototype is `crates/isostate-core/src/roads.rs`, behind `roads.enabled`, which is off under both sets of rules: with it off `cargo test -p isostate-core --release` passes, the golden test among them, and a sweep gives every fact of every island as main does.

## In short

1. **Today nobody pays and nobody is paid.** A road is worn into a track, a road and a turnpike by traffic alone; a river, unless navigable, costs a road nothing to cross; and a railway's or canal's fares are carting money paid to the carters of the town that pays, so no company has an income. On eight islands the island has 5,700 km of road in 1750 and 6,600 km in 1900, a seventh to a fifth of it turnpike; its roads cross 300 to 400 rivers an island, 15 metres wide at the median and 50 at the widest. Carting over the turnpikes is 3.7 per cent of output in 1760 and 0.7 per cent in 1900; railway fares are 2.5 to 3 per cent of output from 1850, 0.2 to 0.4 of the capital laid out a year.
2. **The design, from the period:** a parish makes roads out of a rate and its own men's labour; a turnpike trust borrows, improves a road, takes tolls and pays its lenders; a bridge is costed by its river's width and paid by a parish (for the county), a trust or a landlord; a canal or railway company takes a margin of its fares and pays its subscribers. All four are one pattern: a body that raised money once, holds cells, charges for passing them, keeps them and pays its lenders.
3. **The prototype** (`roads.enabled`, off under both sets of rules) does the first three: road works are projects that buy stone and take hands; a town's trust turnpikes where tolls less upkeep pay 5 per cent; the toll is in the cost graph and paid out of the carters' town to the lenders; bridges cost stone by the metre. With it off nothing moves: 0 of 446 facts on eight islands, and the golden test passes.
4. **Switched on, it moves who pays and what is built, and little else.** Turnpikes are half what the free rule makes (809 km in 1900 against 1,480) and parish roads take their place; tolls are 0.5 per cent of output in 1760 and 0.06 in 1900; 140 bridges an island are built in a run; a few hundred men mend roads. The carting bill, what a wage buys and the towns do not move clear of chance. The railways (a tenth more line in 1900, on every island) and canals (a tenth more in 1800) gain from the worse roads.
5. **Build it in four steps:** railway and canal companies paid their fares first (the most money, needs nothing else); then the roads as prototyped with the toll charged to the load that passes; then bridges and fords on the new land; then toll houses and true bridges in the picture.

## 1. What there is

### How a road comes and goes today

`sim::update_roads` adds up, each year, the weight that passed through every cell: each town's own district carted to market along its tree of cheapest paths, and the trade between towns along theirs. A cell's road is then the number of thresholds its traffic passes (1,500, 9,000 and 40,000 units of weight a year: a track, a road, a turnpike). A road only rises; it falls one level after six years under a fifth of its level's threshold. Nothing goes into it: no money, no hands, no stone.

What a road does is make carting cheaper. In the cost graph (`transport::build_graph`) a cell costs its terrain (1 for lowland, 1.7 for hills, 3.2 for mountains) times 1, 0.7, 0.45 or 0.3 for no road, a track, a road or a turnpike, and any road takes away the charge for climbing. A navigable river is 0.3 a cell, a railway 0.07. A river that is not navigable costs nothing at all, to cross or to follow: a cart fords every stream on the island for free, and a road bridge exists only in the picture.

The carting money is `freight` (0.015) times the weight times the cost of the path. It is paid to the households of the town that pays it, and under the game's rules (`carting.enabled`) it is the wages of carters, who are taken out of that town's hands. That holds for a railway's and a canal's carriage as much as a cart's: their "fares" are the same carting money at their cheaper rates, paid to the same households as wages. Nobody keeps a margin, nobody owns a line once it is open, and nobody is paid a toll.

### Kilometres of each kind

A cell is 2 km. Mean over the eight islands, and the least and greatest:

| | track | road | turnpike | railway | canal |
|---|---|---|---|---|---|
| 1750 | 3,590 [3,266 .. 3,870] | 1,212 | 864 [734 .. 968] | | |
| 1800 | 4,059 | 1,438 | 1,156 [1,082 .. 1,230] | | 662 |
| 1850 | 3,803 | 1,616 | 1,207 [874 .. 1,632] | 1,340 | 888 |
| 1900 | 2,751 | 2,415 | 1,480 [1,312 .. 1,710] | 2,255 | 892 |

So a turnpike is a seventh of the island's roads in 1750 and a fifth in 1900, which is about England's share in the 1830s (some 22,000 miles of trust road of about 120,000). On the second island generator (`--land`) the kilometres are within a tenth of these.

Cells improved in a year, the mean of the decade to the date:

| decade to | to a track | to a road | to a turnpike | fallen back a level |
|---|---|---|---|---|
| 1760 | 8.0 | 6.4 | 5.3 | 5.2 |
| 1800 | 11.0 | 7.0 | 4.6 | 7.9 |
| 1840 | 10.6 | 14.0 | 28.0 [10.8 .. 44.8] | 12.1 |
| 1850 | 9.6 | 16.7 | 27.3 | 14.0 |
| 1900 | 6.8 | 15.5 | 11.7 | 2.4 |

The 1840s' rush of turnpikes is the roads that feed the railway: the traffic to and from a station is new, and the cells on the line itself, which lose their road traffic, are the ones falling back.

### Where roads cross water

A road crosses a river where it runs through a river cell with road on both sides of the river's line there, or where a diagonal step of road passes over a diagonal reach of river (`roads::crossings`; the test in `roads.rs` shows the rule on a hand-made valley). That is the network as the viewer joins it, every two neighbouring cells with road. A river's width is the textbook's 4.4 times the square root of its flow in metres (the land page draws rivers two and a half times that). The second generator keeps each cell's flow; the port's island keeps only the area upstream, and its flow is taken at 0.07 cubic metres a second for each cell of area, which is what the second generator's rivers carry (0.067 to 0.074 by island, about 550 mm of runoff a year).

Crossings an island, and the river's width there, all islands pooled:

| | by any road | by a road or better | by a turnpike | width: median, 90th centile, widest | rivers 20 m or wider crossed by a turnpike |
|---|---|---|---|---|---|
| 1750, port's island | 307 | 85 | 24 | 15, 31, 50 m | 6 |
| 1900, port's island | 406 | 301 | 141 | 15, 31, 50 m | 46 |
| 1750, second generator | 256 | 77 | 35 | 15, 29, 49 m | 11 |
| 1900, second generator | 340 | 212 | 85 | 15, 29, 48 m | 26 |

(The port's island was measured at 0.1 cubic metres a cell, before the 0.07 was found; its widths are about a sixth too wide.) So the island's turnpikes of 1900 cross 85 to 140 rivers, 1.5 to 2.6 km of water in all, and tracks cross three or four hundred. None of it costs anything today.

### What the carting over each kind is worth

The year's carting over the cells of each kind, as a share of the year's output (each cell's weight passed times its own cost; the handling charge and the sea are not in it):

| | no road | track | road | turnpike | navigable river | railway |
|---|---|---|---|---|---|---|
| 1760 | 0.43% | 0.74% | 1.14% | 3.73% | 1.53% | |
| 1800 | 0.23% | 0.50% | 0.68% | 2.61% | 0.70% | |
| 1850 | 0.07% | 0.21% | 0.25% | 0.87% | 0.09% | 2.54% |
| 1900 | 0.02% | 0.07% | 0.14% | 0.72% | 0.04% | 2.39% |

What each step of road saves on the step below, on its own traffic: a turnpike over a made road 1.9 per cent of output in 1760, 1.3 in 1800, 0.4 in 1850 and after; a made road over a track 0.6, 0.4 and 0.1. A toll of 0.05 a cell of turnpike (a sixth of the lowland turnpike's cost) would have come to 0.57 per cent of output in 1760, 0.38 in 1800, 0.10 in 1850 and 0.07 in 1900. England's toll income in the 1830s, about 1.5 million pounds on a national income of some 400 million, was about 0.4 per cent. The busiest half of turnpike cells carry over 82,000 units a year in 1760 and 50,000 to 73,000 later, against the threshold of 40,000.

### What a railway's and a canal's fares come to, and where they go

From the promoters' books (`YearNote::fares`: the weight over each cell of the way at its running cost; a canal's wharves included):

| | railway fares over output | over the capital laid out | canal fares over output | over the capital |
|---|---|---|---|---|
| 1790 | | | 3.2% | 0.36 |
| 1820 | | | 3.4% | 0.45 |
| 1850 | 2.5% | 0.20 | 0.3% | 0.05 |
| 1870 | 2.8% | 0.27 | 0.07% | 0.02 |
| 1900 | 2.4% | 0.39 [0.30 .. 0.49] | 0.04% | 0.02 |

(The second generator: railways 0.43 of capital in 1900, canals 0.61 in 1800.) All of it goes to the households of the town that pays the carriage, as carters' wages, and under the game's rules all of it is counted as carters' time. A railway that took 40 per cent of its fares as its own (the British companies' working expenses ran at about half their receipts) would pay its subscribers 8 per cent on their money in 1850 and 16 per cent in 1900. The subscribers today get nothing back but what is left in the till when a line opens.

## 2. The design: who pays and who is paid

Four bodies paid for the ways of the period, and each has a plain counterpart on the island. What follows is from the standard histories as I remember them (Webb on the King's highway, Albert and Bogart on the turnpike trusts, Hadfield on the canals, the railway companies' returns); the dates and sums are approximate and none was checked for this note.

**The parish keeps its roads with its own people's labour.** From the Highways Act of 1555 every parish chose a surveyor of highways each year, and every householder owed days of labour on the roads (six a year from 1691), those who held land a cart and team as well; a highway rate on the occupiers made up what labour could not. A parish mended the roads its own people used and had no reason to mend a road for the traffic passing through, which is why the main roads were bad and why the trusts came. On the island the parish is the market town's district (`catch`): the island has no smaller unit, and a district is a fair stand-in for a union of parishes. A made road is the parish's to make. It pays out of a rate on the landlords (an occupier's rate falls in the end on rents) and its labour is the town's own men, which the game can treat as wages paid out of the rate to the households who are taken from other work. Nothing new is needed from the simulation but a way to take hands for it, and the canals' navvies are the pattern.

**A turnpike trust borrows, improves a road, takes tolls at its gates and pays its lenders.** A trust was made by a private Act of Parliament at the asking of the landowners, merchants and farmers along a road; it borrowed on the security of its tolls at up to 5 per cent (the usury laws' limit), improved and kept the road, and took tolls by the beast and the wheel at its gates. Most of its income went on repairs, and it seldom paid off what it owed. There were about 1,100 trusts by the 1830s, holding a fifth of the roads. The railways killed them: tolls fell by a third or more on a road beside a line, and the trusts were wound up between the 1860s and the 1890s, their roads passing to highway boards and then to the county councils of 1888. On the island a trust is one a town, lent its money by the town's landlords and households, and it turnpikes a cell where the tolls on last year's traffic, less the cell's upkeep, would pay its lenders their hurdle on the cost. The toll is a charge on the cell in the cost graph, so carriage over a turnpike costs more by the toll, and the money is taken from whoever paid that carriage and goes to the trust: repairs first, the rest to the lenders. What the simulation lacks for this is a charge in the cost graph that is money to somebody (today every unit of path cost is carters' wages) and a way to know who paid it.

**A bridge is costed by the width of what it crosses, and paid by a trust, a town or a landlord.** The Statute of Bridges of 1531 made the county repair the bridges nobody else was bound to, out of the county rate; a trust built and kept the bridges on its road; a landlord bridged a river on his own estate; and from the middle of the eighteenth century companies built toll bridges where a town wanted one and the county would not. A stone bridge's cost goes roughly with its span. On the island a bridge is costed in stone by the width of the river, and paid by whoever first makes a road across it: the parish (standing for the county) for a made road, the trust for a turnpike. A landlord's bridge and a bridge company are a trust of one cell. What the simulation lacks is any cost to crossing water without a bridge. A cart today fords the island's widest river for nothing, so nobody would pay for a bridge but as part of a road, and a road has no reason to keep to one bank (note 20). A ford or ferry charge needs to know which steps of the cost graph cross a river. The cell grid cannot say: whether a step into a river cell crosses it depends on the step after. The second generator can say exactly, once per island: for every pair of neighbouring cells, whether the line between their centres crosses a river on the fine grid, and how wide it is there.

**A canal or railway company takes fares, pays its costs, and pays its subscribers a dividend.** Canal and railway companies were incorporated by Act, raised their capital in shares, made calls on the shareholders as the work went on, and paid dividends out of what their tolls and fares left after working costs. A canal company mostly took tolls from carriers who owned the boats, and was a toll-taker like a trust; a railway company carried for itself, and its working expenses ran at about half its receipts. On the island the promoters already raise the money (`ways::subscribe`) and keep who put in what (`Project::subs`), and the canals already make calls; what is missing is the owner after opening. A `Line` should keep its cells and its subscribers; a share (`margin`) of the fares on its cells' traffic should be taken each month from the paying town, as the turnpike's tolls are, and paid to the subscribers; the rest stays carters' wages, which stand for the crews and porters. The railway's renewals (rails worn out and laid again every fifteen or twenty years) would be the company's upkeep, and they are where steel rails were first wanted: the README notes that the steel rail hardly arrives on the island, and a company that must renew its iron rails is a buyer for them.

**One pattern under all four.** Each is a body that raised money once, holds a set of cells, may charge for passing them, pays for their upkeep and pays its lenders what is left. A parish is such a body with no tolls, financed by a rate. The prototype's `Trust` is the first of them; a canal or railway company would be the same book with a `Line`'s cells. What each needs from the simulation that it does not yet have:

| | needs |
|---|---|
| parish | hands for road work (done in the prototype, after the canals' navvies) |
| trust | a charge in the cost graph that is somebody's money (done roughly), and who paid it (roughly: the carters' town); a book for the trust (done) |
| bridge | which steps of the graph cross water, and how wide: exact from the second generator's fine rivers; a ford charge for crossing without one |
| canal and railway companies | the owner after opening: a line's cells and subscribers kept; a cell's line; the fares' margin taken from the payer; renewals as upkeep |
| all | a heading in the books of account (`accounts.rs` would show tolls and dividends only as transfers between households and landlords); a public purse for the county's bridges and for the roads after the trusts, which the 2050 design is to have (that design is not on main, and I have not read it) |

## 3. The prototype

`roads.enabled`, in `RoadParams` in `crates/isostate-core/src/roads.rs`, off under both sets of rules (it is not in `game_params`). `--set roads.enabled=true` switches it on for a sweep. Hooks, each one line and each returning at once while the switch is off: `sim::update_roads` (a cap on what traffic makes for free), `sim::yearly` (`roads::begin` and `roads::yearly`), `sim::run_month` (`roads::month`), `econ::step_month` (`roads::take_menders`, `roads::count_menders`) and `land::update_transport` (`roads::tolls`). `ways.rs` gains a fourth kind of project (`ways::ROAD`) and lends its `repay`.

- **1750's roads are inherited.** The warm-up of the 1740s makes roads for free as before: they are what earlier centuries paid for. Every turnpike standing at the first recorded year gets its town's trust, whose lenders are the town's landlords, for a notional capital at the day's prices (no money moves; it was lent long ago). Their bridges stand.
- **A track is still worn for nothing.** From then on traffic makes a track and nothing more.
- **The parish makes roads.** Each year, the tracks in a town's district whose traffic passed the port's second threshold (9,000) are made into roads, busiest first, as far as a rate of 2 per cent of the town's landlords' cash goes (or half a per cent of its households' where the landlords are in debt). A cell takes 50 stone on lowland, 85 on hills and 160 on mountains, and 8 stone a metre of river where the road is first made across one. The work is a project like a canal's: the town's merchants sell it the stone over twelve months and it pays two shillings of wages on every shilling of stone, to the town's households, whose men are taken from the farms and works while they mend (`take_menders`, at most a tenth of a town's hands, moved a quarter of the way a month). A works short of cash calls on the town for more. When its stone is all bought the cells become roads.
- **The trust turnpikes roads.** Each year the town's trust prices turnpiking each track or road in its district that has no railway and no navigable river (110 stone a cell on lowland, plus the road's 50 where it is still a track, plus any bridge), and takes those whose tolls on last year's traffic, less 15 stone a year of upkeep, pay its lenders 5 per cent on the cost, the best first, as far as a fifth of the landlords' cash and a twenty-fifth of the households' will lend. It builds as the parish does, and when the work is done it sets up its gates (with a news item the first time).
- **The toll is part of what carriage costs.** A cell under a trust costs 0.05 more to pass in the cost graph, half on each edge into it and half out, except by rail. So a cart over a lowland turnpike pays 0.35 where it paid 0.3. Each month the trust takes a twelfth of the toll on its cells' traffic of the year before from the households of the cell's market town (the carters, who were paid it as part of the carting money) and out of that town's carting bill, since a toll is nobody's time. It pays its repairs first (a standing project buying the year's upkeep stone, with wages), and what is left to its lenders in proportion to what each lent.

Every penny moves between purses `econ::money_total` counts: a works' cash is a project's, counted already; the trusts hold none. The monthly money check holds on every run with the switch on (the sweep's eight and the probe's sixteen, below).

## 4. What it moved

Two runs on seeds 1 to 8, the game's rules, six families: the probe (`proto-game.txt`, against `base-game.txt`) and the sweep (`game-roads` against `game`, the same code with the switch off, which is main to the last fact). All eight runs finished and the monthly money check held on every one. Of 446 facts 387 differ, none starred.

**Roads built.** The trusts turnpike about half what the free rule does, and the parishes make more roads in its place:

| km | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| turnpike, main | 864 | 1,156 | 1,207 | 1,480 |
| turnpike, paid for | 858 | 750 | 620 [440 .. 812] | 809 [418 .. 1,064] |
| made road, main | 1,212 | 1,438 | 1,616 | 2,415 |
| made road, paid for | 1,212 | 1,841 | 2,192 | 2,908 |

A trust turnpikes 1 or 2 cells a year before 1830 and 4 to 7 in the railway age, where the free rule made 4 to 7 and then 28; 1750's turnpikes, tolled, lose traffic to the made roads beside them and fall back. A cell pays its trust from some 50,000 units of traffic a year (the free rule's threshold is 40,000), and a town's trust can borrow only what a fifth of its landlords' cash will lend. Trusts number 7.6 an island in 1760 (the turnpikes of 1750), 12.3 in 1800 and 16.6 in 1900; the news tells of 10.6 trusts setting up their gates in a run. Bridges: roads cross 85 rivers an island in 1750 and 222 in 1900 (main 301), turnpikes 24 and 31 (main 141); so the parishes and trusts build about 140 bridges an island in a run, about one a year, at a median width of 12 metres.

**What it costs and who is paid.** The works' outlay is 0.33 per cent of output in 1760, 0.18 in 1800, 0.06 in 1850 and 0.03 in 1900; the menders are 214 people in 1760 (0.3 per cent of the island's hands; the carters are 12.5 per cent), 169 in 1800 and 94 in 1900. Tolls are 0.49 per cent of output in 1760, 0.27 in 1800, 0.08 in 1850 and 0.06 in 1900, about England's share of the 1830s at the start and well under it later (the first run, which let boats on the river be tolled at a road's gates, gave 0.76; that was a mistake and was mended). Households hold 0.183 of a year's output in cash in 1760 where they held 0.193: the tolls come out of their carting money. The landlords' cash hardly moves (0.097 against 0.098), since they both lend and are repaid; nor do the books' shares of income (work's share 0.647 against 0.654 in 1850, land's 0.172 against 0.174 in 1900).

**Carriage.** The carting bill over output is 0.078 against 0.079 in 1800, 0.054 against 0.052 in 1850 and 0.045 against 0.044 in 1900, no island moving clearly either way; carters are 8.7 per cent of hands in 1850 against 8.3 (seven islands of eight higher). A tolled turnpike (0.35 a lowland cell) is not much better than the made road (0.45) a parish puts beside it, so swapping one for the other costs carriage little.

**The railways and canals gain.** What does move, on nearly every island, is the other ways: with worse and tolled roads, a line or a cut saves more. Railway in 1900 2,489 km against 2,248 (eight islands of eight), in 1870 1,961 against 1,835; branch lines to farms 48.6 a run against 31.1. Canal in 1800 743 km against 662 (seven of eight), in 1900 958 against 892, the first opened in 1770 against 1774, and the canals carry more all century.

**What a wage buys, and the towns.** Nothing clear. The real wage is 1.63, 2.00 and 2.56 times 1750's in 1800, 1850 and 1900, against 1.63, 1.95 and 2.60, the islands split 3/5, 4/4 and 5/3. Baskets a head in 1900 8.42 against 8.76 (two up, six down), goods a head 11.3 against 11.4. The share in towns is 0.414 against 0.419 in 1900, towns 16.6 against 17.8, the last founded in 1856 against 1847. Famine years 0.8 a run against 0.4, on three islands of eight. Works in 1900, 709 against 738 (one up, six down).

So in this form the mechanism is a matter of who pays and who is paid, and of what is built where, more than of how well the island lives: it moves a quarter to half a per cent of output from households to the trusts' lenders before 1800 and much less after, takes a few hundred men off the farms, halves the turnpikes, and gives the railways and canals a tenth more traffic to win.

## 5. What it leaves out

- **Who pays a toll is roughly guessed.** The toll is taken from the households of the market town whose district holds the gate, as if the carters who passed it lived there. A load from a farther town that passes through pays its toll in the graph's cost but the money is taken from the town at the gate. The honest way is a second sum along every path: the cost graph would carry tolls as a separate layer, the path searches would add up the tolls on each path beside its cost (`dcatch` and `town_cost` beside a `dtoll` and a `town_toll`), and `econ::step_month` would pay the toll part of each carting bill to the trust whose gates it passed. That is a change to the searches every mechanism uses and was too large for a probe.
- **Tolls are on last year's traffic.** A twelfth a month, of the traffic `update_roads` measured over the year before. A road whose traffic doubles pays last year's tolls for a year.
- **No ford charge, so no reason to bridge but a road's own making.** See the design, above. With it, bridges would be built because crossing without one costs carriage, and roads would keep to one bank.
- **Nothing obliges a parish.** It makes the busiest tracks into roads as far as its rate goes and lets the rest be; in the period an indictment at quarter sessions could make a parish mend a road, and nobody here can.
- **Parish roads are not kept.** Only trusts pay upkeep. A made road decays only by the port's rule, after six quiet years.
- **One trust a town, never wound up.** A trust that loses its cells to the railway keeps its lenders and its name, takes no tolls and pays nothing. The trusts' debts are never repaid; their lenders are paid what the tolls leave after repairs, which is the trusts' history but not their law.
- **A road works takes twelve months whatever its length**, as every railway and canal line takes its fixed months.
- **The 1750 trusts' capital is notional.** Their lenders are their town's landlords for what turnpiking cost at 1750's prices; nobody paid it in the run.
- **Canal and railway companies are designed, not built**: their fares still go to the carters.
- **The books of account do not know the trusts.** Tolls and dividends show there only as money moving from households to landlords and households; a heading for each belongs in `accounts.rs` (another helper's ground today).

## 6. What it would take to draw

Today the viewer stands a bridge wherever two neighbouring cells with road are joined across the river it has drawn (`scatter.rs`, `isostate_gen::bridge::road_bridge`), and the simulation knows nothing of it (piece 3 of the findings). With roads paid for, the simulation knows three things the picture could show.

- **Bridges where roads really cross.** `RoadState::bridged` marks each cell where a bridge was built (or inherited in 1750), and `roads::crossings` says which cells a road crosses water in and how wide the water is. The viewer's own rule and this one are close cousins: both join every two neighbouring road cells; this one says a crossing is where road lies on both sides of the river's line through a cell, or a diagonal step of road passes over a diagonal reach. Drawn from the simulation, a bridge would stand only where one was paid for, a ford or a ferry where a track crosses without one, and a road down a valley would cross its river only where its cells do. On the second generator the crossing should be found on the fine rivers, where the road's line meets the river's, and the bridge's span is the river's width there, which the land page already keeps for every point of a river.
- **Toll houses and gates.** A trust's cells are `RoadState::cell_trust`. A toll house stands where a trust's road meets a road that is not the trust's, and along a long stretch every four or five cells (gates were some six to ten miles apart): a small house of the kit's cottages with a bay to the road, and a bar or a gate across it. The trust's road could be drawn as the turnpike is today and the parish's road as the made road, which is what they already are.
- **Works on the road.** A road works is a project like a canal's, so its cells could be drawn as a road being made (heaps of stone, a gang, a roller) for the months it takes, as the canals' cuts are shown while they are dug.

What the record would carry: two bits of the `trans` layer are free (64 and 128); 64 for a cell under a trust and 128 for a bridge standing. That is all the viewer needs for the first two, with `roads::crossings` run on the year's roads to give each bridge's direction and width. Two things in the core would have to change first, and are noted in the code: the record already skips road works when it marks a railway being built (`record.rs`), but the live game's list of lines building (`live.rs`, another helper's ground today) counts every project that is not a canal as a railway, so with the switch on a road works would be listed as one.

## 7. Whether to build it, and in what order

Yes, in four steps, smallest first, and each with the convention's switch, page and sweep.

1. **Railway and canal companies that are paid their fares** (piece 16; a day or two). It is the half of Stella's question with the most money in it: 2.5 to 3 per cent of output from 1850, against tolls of a tenth of that. The promoters already keep who put in what; a `Line` keeps its cells and its subscribers; each month a margin of the fares on its cells' traffic is taken from the paying town, as the prototype's tolls are, and paid as dividends. It moves money from carters' wages to the landlords and households who subscribed, frees the carters it stands for, and gives the subscribers a reason to have put their money in. With it go the calls with a limit and the line that is abandoned (piece 16's other half). Its renewals of rails are a buyer for steel.
2. **Roads paid for, as prototyped, with the toll paid by whoever's goods pass the gate** (a few days). The prototype is the smallest whole version and it holds together; what it needs before it is a mechanism is the second sum along the paths (section 5), so that a toll is charged to the load that passes and paid to the trust whose gate it passed, and a page and numbers searched on more islands. The parish's rate and the trusts' hurdle and lending are the numbers that decide how much is turnpiked, and none was searched.
3. **Bridges and fords on the new land** (a day, once the second generator is on). The crossing table from the fine rivers, a ford charge for crossing water without a bridge, and a bridge built where the carting saved pays for it, by the parish (the county), a trust or a landlord's company. This is what makes roads keep to one bank (note 20), and it waits on the new land being drawn, as piece 3 of the findings does.
4. **The drawing** (section 6): toll houses and gates on the trusts' roads and bridges where roads really cross, from two bits of the record. It comes after step 2 at the earliest, and is worth doing with piece 3 of the findings, which rebuilds the viewer's roads and bridges on the new rivers anyway.

The order could be argued the other way: step 3 is what Stella saw (a road crossing its river again and again), and steps 1 and 2 are what she asked. But step 3 cannot be done well on today's rivers, and steps 1 and 2 can be done now. Nothing here should be built before the trigger the backlog gives it (the new land merged, and the public purse of the 2050 design read), except step 1, which needs neither.

## 8. How to run it again

```bash
cargo run --release -p isostate-core --example probe_roads                          > base-game.txt
cargo run --release -p isostate-core --example probe_roads -- --land                > base-game-land.txt
cargo run --release -p isostate-core --example probe_roads -- --set roads.enabled=true   > proto-game.txt
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules game --families 6 --set roads.enabled=true --out PROTO --compare BASE
```
