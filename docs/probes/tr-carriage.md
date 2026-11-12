# tr-carriage: carriage, stocks and famine, traced with the books

Branch `night/o-trace-carriage`. One of four traces with the books of account (piece 12 of `docs/backlog/found-along-the-way.md`): things measured and never explained, which the books (`docs/rules/accounts.md`) make cheap to follow. Four questions:

- (a) Where the gain from railways goes. With them output per head is up about 5 per cent and what a wage buys is flat (R21, C86; `docs/rules/carting.md`, `docs/rules/railways.md`). Who gets the saving in carting?
- (b) How much of the carting bill is the 3 per cent handling charge that no road, canal or railway lowers (R15, C87), by decade, and so how much of the bill a better way can ever save.
- (c) How much coal and timber is raised into stocks that sit for years, since the brake on a glut is on works alone (R81, C10), and what it costs the island in hands.
- (d) Why famine years fell from 2.2 a run to 0.4 when canals came in (README, "Where it stands"; `docs/rules/canals.md`).

Everything is under the game's rules with six families, at `main` 295a379 (the code is the same at e388c8c), on seeds 1 to 16 where a question turns on a count and 1 to 8 where it does not. No rule was changed; nothing under `crates/isostate-core/src` differs from `main`. The measuring is one example, `crates/isostate-core/examples/trace_carriage.rs`, which plays each island month by month so that it can read each town's food before the year's end clears it, keeps the books, and prints twenty-year means (decades for the carting bill, half-centuries for hunger) for every setting given with `--set`, with each setting set beside the first seed for seed. `--out DIR` writes the rows behind the tables. Three cures were tried in the worktree and not committed; each is described where it belongs. The numbers, the commands and every run's output are in `.claude/night/reports/opus/trace-carriage/` (outside the repository).

## In short

1. **The railway's gain reaches the wage now.** Under the game's rules on sixteen islands, railways against none, 1880 to 1899: output a head 8.3 per cent higher and the real wage 8.3 per cent higher, both on every island. The cost of living is 11 per cent lower and the money wage 3 per cent lower: the gain comes through prices. Of the extra income, work gets about seven parts in ten, which is its share of income with or without railways, and the owners of land and houses a third, twice theirs; works' owners and merchants get nothing (on eight other islands, run in review, the works' owners get a sixteenth). The freed carters (a third of them, 3.6 points of all hands) go to the land and the works, and none stand idle. The "real wage flat" of `carting.md` is true of the port with carting, where it was measured (0.8 per cent, merchants and works' owners taking four parts in five of the gain), and not of the game.
2. **The handling charge is a tenth to a seventh of the carting bill once the railways run**, 3 per cent of it in the 1750s, 6 per cent in 1800, 12 per cent in the 1840s and 15 per cent in the 1890s; a fifth of the carting on trade before the railway and a third after. It is half a per cent to seven tenths of a per cent of income and about one hand in a hundred. Taking it away altogether frees four tenths of a point of the hands and moves no fact outside the spread between islands. What a better way could still save is the rest of the bill, and half of that, at the end of the century, is the farms' and woods' carting to their own market, which no line between towns touches.
3. **Coal is not raised into long stocks, and timber hardly.** The merchants hold 1.1 to 1.2 months of coal from 1780 on; under a two-hundredth of it is in towns holding a year's worth; 0.14 to 0.21 per cent of what is dug spoils, and the hands that dug it are a ten-thousandth of all hands. Timber's stock grows from 2.1 months in 1800 to 3.6 in the 1880s and 1890s, a sixteenth of it in towns holding a year's, and 1.2 per cent of it spoils: two hands in ten thousand. Part of the long timber is woodland cleared at the year's end, which nobody chose to raise. A brake on the woods and pits like the works' was tried and changes nothing outside the spread, because there is next to nothing for it to brake.
4. **The famines the canals took away were the railway's, and they are a month's empty shelf in a town that has just been handed a district larger than itself.** Of 30 hungry town-years on sixteen islands with and without canals, 23 come in the year after the town at least doubled its people when the catchments were redrawn at the year's end, and more than half of the hungry months are Januaries. `peopling::move_with_cells` sends the people, their cash and last month's income with their cells, but the merchants' stock and their record of sales only to a market that is new; an old town that doubles keeps the shelves of the town it was. Without canals the first railways do this 45 times in the 1830s and 1840s (sixteen runs together); with canals the catchments were redrawn earlier and by less, and it happens 14 times in those decades. With neither canals nor railways it happens 8 times in the whole century and the famines fall to 0.19 a run; with canals and no railway they are 0.19 too, so the canals make no difference to famine but through the railway. Sending the stock with the people (a trial, not committed) ends every hunger that followed a doubling, with canals and without, here and on sixteen other islands run in review. Cheaper coal and fewer people are none of it. Grain carried more cheaply between towns is not what the canals do either: making it dear again on an island with canals brings no hunger back. (On an island without canals, carrying grain a quarter cheaper on its own takes off some hunger, seven islands of sixteen lower and four higher, which is inside the spread.)

## (a) Where the railway's gain goes

### How it was measured

```bash
cargo run --release -p isostate-core --example trace_carriage -- --seeds 1-16 --set rail_know=0.25,99 --threads 12 --out DIR
```

`rail_know` is how well a town must know the railway before promoters build; 0.25 is the game's, and at 99 no line is ever built. Everything else, the canals included, is as the game has it. The example adds up the island's income account (`IslandYear`) by kind and puts each kind a head a month in 1750 money, deflated by the observer's cost of living (`cpi`, the 1750 basket at the island's prices), so that the parts add up to output a head (`gdp_pc`), which they do to within two parts in a thousand (the observer averages the month's ratio, the books the year's sums). Twenty-year means; the parts of the gain are the differences of those means.

Then the same pair on eight islands with one mechanism after another switched off (`--set works.enabled=false --set rail_know=0.25,99`, and so for `prices`, `machines`, `people`, `engel`), with the port's own promoters in place of the game's (`railways.enabled=false`), and under the port with carting alone, where `carting.md` measured it (`--rules port --families 0 --set carting.enabled=true --set rail_know=0.25,99`).

### What it shows

1880 to 1899, railways against none, the mean of sixteen islands and the islands higher and lower:

| | with | without | | |
|---|---|---|---|---|
| output a head | 8.38 | 7.73 | +8.3% | 16/0 |
| real wage (1750 = 1) | 2.73 | 2.52 | +8.3% | 16/0 |
| money wage | 12.20 | 12.62 | -3.3% | 4/12 |
| cost of living | 1.053 | 1.180 | -10.7% | 2/14 |
| people, thousands | 640 | 644 | -0.6% | 1/15 |
| carters, of all hands | 0.070 | 0.107 | | 0/16 |
| on the land, of all hands | 0.440 | 0.421 | | 16/0 |
| in works, of all hands | 0.489 | 0.472 | | 16/0 |
| for hire and not at work | 0.001 | 0.001 | | |
| work's share of income | 0.697 | 0.697 | | 7/9 |

and the income a head a month, in 1750 money:

| | with | without | difference | part of the gain |
|---|---|---|---|---|
| all of it | 8.363 | 7.721 | +0.643 | |
| pay other than carting | 5.429 | 4.831 | +0.597 | 93% |
| carting | 0.401 | 0.554 | -0.152 | -24% |
| works' owners | 0.729 | 0.731 | -0.003 | 0% |
| merchants | 0.372 | 0.382 | -0.010 | -1% |
| rent of farms, woods, pits and mines | 0.912 | 0.799 | +0.114 | 18% |
| rent of works' sites | 0.064 | 0.062 | +0.002 | 0% |
| rent of town houses | 0.456 | 0.361 | +0.094 | 15% |

So the carters' pay that the railway saves (0.15 a head) is a quarter of the gain, and the gain is four times the saving: the hands that leave the road make more on the land and in the works than their carting was worth. Work nets 69 per cent of the gain, which is its share of income with or without railways (0.697 both), and the owners of land and houses 33 per cent, twice their share of income (about 0.17); the works' owners and the merchants, whose share is the remaining eighth, get none of it. Buyers get it through prices: the cost of living falls a tenth and the money wage a thirtieth. Nobody loses it to idleness. In 1860 to 1879 the shape is the same (output a head and the real wage both up 10 per cent).

The real wage is flat where `carting.md` said, and only there. With each mechanism switched off in turn, or with the port's rules, eight islands each, 1880 to 1899:

| | output a head | real wage | money wage | cost of living | to work | to land and houses | to works' owners | to merchants |
|---|---|---|---|---|---|---|---|---|
| the game, sixteen islands | +8.3% 16/0 | +8.3% 16/0 | -3.3% | -10.7% | 69% | 33% | 0% | -1% |
| the game, the eight islands of the rows below | +7.9% 8/0 | +7.5% 8/0 | -6.0% | -12.6% | 66% | 32% | 1% | 1% |
| the port with carting, no families | +6.9% 8/0 | +0.8% 6/2 | -8.0% | -8.9% | 8% | 13% | 38% | 40% |
| the game, the port's promoters | +5.3% 7/1 | +4.5% 7/1 | -2.1% | -6.3% | 60% | 33% | -1% | 7% |
| the game less the works' life cycle | +7.1% 8/0 | +5.1% 8/0 | -10.1% | -14.6% | 45% | 30% | 24% | 1% |
| the game less merchants' prices | +10.4% 8/0 | +10.6% 7/1 | +1.9% | -8.7% | 71% | 30% | 3% | -4% |
| the game less machines | +4.4% 8/0 | +3.8% 6/2 | -4.1% | -7.4% | 59% | 38% | 7% | -4% |
| the game less people | +6.6% 8/0 | +5.9% 8/0 | -10.6% | -15.5% | 63% | 36% | -6% | 7% |
| the game less the Engel curve | +12.1% 8/0 | +11.4% 8/0 | -0.5% | -10.9% | 63% | 30% | 2% | 6% |

("To work" is pay and carting together; the parts can pass 100 per cent because some are below nothing. The rows of eight islands are seeds 1 to 8, and the second row is the game on those same eight, from the run that switched the merchants' prices, so that each variant can be read against its own islands.)

Under the port the money wage falls as far as prices do and the gain goes to the merchants and the works' owners, whose dividends are paid to the households of their towns, so the people get it, but as dividends and not as a wage. Under the game no single mechanism taken away brings that back. Taking away the works' life cycle sends a quarter of the gain to the works' owners (works that cannot grow where they stand keep the margin the cheaper carriage gives them, where under the life cycle the year's room for new capacity competes it away); that is the largest single move. The network is not it: the port's trunks under the game's other rules give merchants 7 per cent, not 40. What makes the port's merchants take two fifths I did not find; it would take the opposite sweep, the port with carting and each of the game's mechanisms added one at a time.

### How sure, and what would change my mind

Sure of the direction and of the split: sixteen islands of sixteen for output and the real wage, and the split between work and land within a few points in every variant but the two named. On eight islands that were not used here (seeds 17 to 24, run in review) the shape holds: output a head +8.3 per cent and the real wage +7.8 per cent on all eight, the cost of living 12.5 per cent lower, work 66 per cent of the gain and land and houses 33; the one difference is that the works' owners get a sixteenth of it there (their profit higher on all eight) and the merchants lose a twentieth. The size (8 per cent) depends on how far the network reaches, which differs between islands by a factor of three in the 1840s and 1850s (540 to 1,790 km in the mean of those twenty years). Two things are not measured. The population is a little smaller with railways on fifteen islands of sixteen (by 0.6 per cent), which I did not trace; a gain spread over fewer people is a little larger a head. And the cost of living is a fixed basket of 1750, which leaves out that a railway's cheaper goods are bought in larger quantities; the households' own prices (what they bought at the year's prices over the same at base prices) fall by the same tenth, so it does not change the answer here.

### What it means for the pass over the rules

- R21 and C86 can close as asked: under the game's rules the gain does reach the wage. `carting.md`'s "What the sweeps showed" now says at its head that it is the port with carting alone; its bullet "The real wage does not rise with output", which says it did not trace where the gain goes, could point here: under the port merchants and works' owners take four parts in five, and under the game the wage gets it.
- Landlords get a third of a railway's gain and rents of town houses rise by a quarter: the railway is a rent-raiser in towns. Whoever designs railway companies and fares (C86's other half, `railways.md`) should know that today a third of a line's value goes to the rents of land and of town houses, two thirds to work, and none of it to whoever paid for the line, since nobody collects its fares (`railways.md`).
- If the works' life cycle is ever retuned, watch this: it is what keeps cheaper carriage from becoming works' profit.

## (b) How much of the carting bill is handling

### How it was measured

From the same sixteen pairs. The carting bill is the towns' three kinds of carting paid to carters (`cart_land`, the produce's way to its market; `cart_works`, the works' carting in and out; `cart_trade`, what a town sends to other towns). The handling charge is paid only on trade between towns: `econ::step_month` charges `freight * weight * path cost + handling * price at the sending town` a unit, and the books' trade rows hold the units, their worth at the sending town and the carting paid on them, month by month, so the handling is exactly 0.03 times the worth (the example checks that the trade rows' carting adds up to the towns' `cart_trade`: 1.0000 in every decade). Carting labour (`carting.md`) makes carters of the whole bill, handling included, so a town's carters times the handling's share of the bill is the hands it takes.

Then the charge was set to nothing: `--seeds 1-8 --set handling=0.03,0`.

### What it shows

By decade, sixteen islands, with the railway and (in brackets) without:

| | 1750s | 1800s | 1830s | 1840s | 1850s | 1870s | 1890s |
|---|---|---|---|---|---|---|---|
| carting bill, of income | 0.091 | 0.078 | 0.062 (0.073) | 0.054 (0.076) | 0.052 (0.079) | 0.052 (0.078) | 0.046 (0.070) |
| handling, of the bill | 0.027 | 0.063 | 0.104 (0.074) | 0.123 (0.075) | 0.129 (0.081) | 0.128 (0.088) | 0.149 (0.105) |
| handling, of trade's carting | 0.21 | 0.19 | 0.24 (0.19) | 0.27 (0.19) | 0.29 (0.20) | 0.28 (0.21) | 0.32 (0.24) |
| the bill: land's produce to market | 0.85 | 0.63 | 0.54 (0.57) | 0.52 (0.56) | 0.51 (0.55) | 0.52 (0.54) | 0.49 (0.52) |
| the bill: works' carting | 0.03 | 0.03 | 0.03 | 0.03 (0.04) | 0.03 (0.04) | 0.03 (0.04) | 0.04 |
| the bill: trade between towns | 0.13 | 0.34 | 0.43 (0.40) | 0.45 (0.41) | 0.46 (0.41) | 0.45 (0.42) | 0.47 (0.44) |
| carters, of all hands | 0.131 | 0.116 | 0.096 (0.111) | 0.084 (0.115) | 0.080 (0.122) | 0.078 (0.117) | 0.068 (0.103) |
| of them carrying the handling | 0.004 | 0.007 | 0.010 (0.008) | 0.010 (0.009) | 0.010 (0.010) | 0.010 (0.010) | 0.010 (0.011) |

Handling is a growing share because trade between towns grows (from 8 per cent of income in the 1750s to 22 per cent in the 1840s) and the freight on it falls. As a share of income it is nearly constant, half a per cent in 1800 and seven tenths in the 1890s, with or without railways, since it is 3 per cent of a trade that is a fifth of income either way.

With no handling at all, 1880 to 1899, eight islands: carters are 6.45 per cent of hands for 6.86 (seven islands of eight fewer), output a head 8.17 for 8.28 (two of eight higher), the real wage 2.66 for 2.68 (four and four), and the merchants' takings 10 per cent higher (seven of eight), since the charge was theirs to pay. The four tenths of a point of hands do not show as output: with the charge gone more trade moves, and part of what the handling took goes to carting the extra.

### How sure

The shares are arithmetic on the books and as sure as the books (`accounts.md`: they balance to rounding). The run without handling is eight islands and its output figure is inside the spread.

### What it means for the pass over the rules

- R15 and C87 can close: a better way can save at most about 85 per cent of the bill at the end of the century, and the 15 per cent it cannot is a hand in a hundred.
- The larger fact for a designer is the other half: the farms' and woods' carting to their own market is half the bill to the end, railway or none, and only roads, a nearer market or a branch to the district lowers it. That, not the handling, is the carting that no line between towns touches.
- If a merchant's handling were ever to be labour of its own (warehousemen, not carters), this is its size.

## (c) Coal and timber held long

### How it was measured

From the same runs. For each town and year the books give the merchants' stock at the year's end and the year's sales and shipments of each good (the eight kinds of buyer and `sent`), and what spoiled. The example adds up, island-wide: the stock in months of sales and shipments; the share of the stock in towns holding more than twelve months of their own; the stock beyond four months (where the works' brake begins) in months of the island's sales; what spoiled over what was made; the hands the spoiled goods took to raise (spoiled times the land sector's hands over its output); and for timber, the woodland cleared at the year's end (`cleared`), which is timber nobody chose to fell. Grain beside them for comparison.

Then a brake on the woods and the pits, tried and not committed: in `econ::step_month` the labour wanted by every forestry and colliery unit was scaled by `prices::take_at` on its town's stock and sales, the same share the merchants take of a works' output (it is the diff `trial-mends.diff` in the reports folder). Eight islands.

### What it shows

Sixteen islands, the game as it is:

| | 1780-99 | 1800-19 | 1840-59 | 1860-79 | 1880-99 |
|---|---|---|---|---|---|
| coal: stock, months of sales | 1.25 | 1.16 | 1.14 | 1.11 | 1.08 |
| coal: share in towns holding over a year | 0.004 | 0.003 | 0.000 | 0.000 | 0.000 |
| coal: stock beyond four months, months of sales | 0.009 | 0.004 | 0.001 | 0.001 | 0.001 |
| coal: spoiled, of what was dug | 0.0021 | 0.0018 | 0.0018 | 0.0016 | 0.0014 |
| coal: hands that dug what spoiled, of all hands | 0.0001 | 0.0001 | 0.0001 | 0.0001 | 0.0001 |
| timber: stock, months of sales | 2.14 | 2.11 | 2.62 | 3.22 | 3.60 |
| timber: share in towns holding over a year | 0.004 | 0.002 | 0.004 | 0.030 | 0.063 |
| timber: stock beyond four months, months of sales | 0.011 | 0.006 | 0.136 | 0.458 | 0.777 |
| timber: spoiled, of what was made | 0.0048 | 0.0043 | 0.0056 | 0.0081 | 0.0117 |
| timber: hands that felled what spoiled, of all hands | 0.0003 | 0.0002 | 0.0001 | 0.0001 | 0.0002 |
| timber: cleared woodland, of what was made | 0.025 | 0.024 | 0.055 | 0.077 | 0.084 |
| timber: of the cleared, share landing in towns that end the year over four months | 0.04 | 0.04 | 0.12 | 0.24 | 0.28 |
| grain: stock, months of sales | 1.83 | 1.71 | 1.83 | 1.74 | 1.60 |
| grain: spoiled, of what was made | 0.028 | 0.025 | 0.030 | 0.027 | 0.023 |

Coal is not raised into stocks that sit: the island holds about five weeks of it, less than of grain, and what spoils is a fifth of a per cent. Before 1780 there is a little more (1.9 months in 1760-79, a sixth of a month beyond four), the pits of a young island running ahead of the works. Timber is held longer late in the century, when the woods' hands have fallen from a fifteenth of all hands to a seventy-fifth and the price has nearly doubled. A good part of it is in towns whose ground is being cleared (the towns' and the works' ground, the workings' leases), and the clearing lands in the books' year-end stock the moment it is felled, so a part of "beyond four months" is a December's clearing waiting for the year's buyers. Even so, a ninetieth of the timber made spoils, and the hands it cost are two in ten thousand.

The brake on woods and pits, eight islands against the same eight, 1880 to 1899: timber beyond four months 0.69 months for 0.73 (five islands of eight lower, three higher), coal 0.000 either way, timber spoiled 1.06 per cent for 1.13, output a head 8.14 for 8.28 (three of eight higher), the real wage 2.60 for 2.68 (three of eight). Nothing outside the spread, in any half-century.

### How sure

Sure that coal is not a problem: every island, every period after 1780. For timber the late stock is real and small; the share of it that is the year-end clearing is an estimate (a quarter to three in ten of the cleared lands in the long-stock towns, which is about a third of the stock beyond four months), since the books do not keep each December's clearing apart from the stock it joins.

### What it means for the pass over the rules

- R81 and C10 can close: the brake on works alone costs the island nothing measurable in coal or timber. The likely reason, which I did not test apart, is that the land slows down without it: a pit or a wood hires by what its produce fetches at the town's price, and a long stock marks that price down (`prices::mark`), so the land sheds hands of its own accord.
- If timber's late stock matters (for the viewer's woodyards, or the building cycle of `works.md`, where timber trebles in the years the mills order), the thing to look at is the clearing, which comes at the year's end in one lump and in the towns that are growing. Spreading it over the year, or letting it go to the buyers that clear it, would be the change.

## (d) Why canals took the famines away

### How it was measured

The sweep's `famine_years` counts years with any news item of the kind `famine`. Two things make one: `people::growth`'s hunger (a town founded more than two years before whose worst month's food fell under 0.82 of need), and the sailings of `peopling` ("N people leave the island", after a hungry year). The example tells them apart and also records, for every town and year, its worst month's food and the share of its people that came or went with redrawn districts at the year's end before (`State::pp_moved`, which the chronicle reads too); for each hungry town it keeps the months of that year and the one before.

```bash
cargo run --release -p isostate-core --example trace_carriage -- --seeds 1-16 --set canals.enabled=true,false --threads 16 --out DIR
cargo run --release -p isostate-core --example trace_carriage -- --seeds 1-16 --set canals.enabled=false --set rail_know=99 --threads 16 --out DIR
```

Then the suspected causes one at a time. Grain moved to where it is short: a trial (not committed) multiplied the carriage of grain between towns by 0.75 with no canals (about what canals do to it) and by 1.35 with them (about undoing it). Cheaper coal freeing income for food: households' spending on warmth and on food from the books. Fewer people: the population. The sweep's luck: sixteen islands, a continuous measure (the worst town's worst month) beside the count, and the hunger traced town by town. And the cause found: a trial (not committed) that sends the merchants' stock and sales record with the people to every market.

### What it shows

Canals on and off, sixteen islands, 1801 to 1850, with the islands on which the run with canals is higher and lower than the same island without:

| | canals | none | |
|---|---|---|---|
| hunger years in a run (all of the century) | 0.38 | 1.38 | 3/9 |
| of which in 1801-1850 | 0.25 | 1.31 | 1/9 |
| sailings | 0.06 | 0.06 | |
| town-months under 0.82 of food, a run | 0.44 | 1.69 | 2/9 |
| the worst town's worst month's food, mean of the years | 1.574 | 1.486 | 15/1 |
| grain sent between towns, of grain made | 0.098 | 0.071 | 16/0 |
| grain's carting between towns, a unit | 0.096 | 0.126 | 0/16 |
| grain price | 0.672 | 0.719 | 3/13 |
| households' spending on warmth, of all their spending | 0.102 | 0.103 | 9/7 |
| on food | 0.311 | 0.313 | 4/12 |
| people, thousands | 293 | 292 | 14/2 |

Every famine year in these runs is a hunger year (the two sailings come in years that had hunger too). The island with canals has more people, not fewer, and spends the same share on warmth and on food, so neither fewer mouths nor cheaper coal is the story. The worst town's month is better with canals on fifteen islands of sixteen, a measure that does not rest on a few events, so it is not the sweep's luck either.

Grain carried between towns is a quarter cheaper with canals and a third more of it moves. But the trials say it is not the cause. With no canals and grain's carriage at three quarters, hunger years in 1801 to 1850 are 0.75 a run for 1.31 (seven islands lower, four higher, five the same), about half of what the canals do, and the worst town's month does not move (1.490 for 1.486). With canals and grain's carriage at 1.35, back near what it was without them, the hunger does not come back: 0.06 a run for 0.25.

What the hungry towns have in common is in the months. Of 38 hungry town-months, 20 are Januaries, and the town's food runs in a sawtooth that starts at the year's turn and dies away by summer. Uldale on island 1 in 1848, the merchants' grain stock at each month's end and the food households got against need:

| | Nov | Dec | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug |
|---|---|---|---|---|---|---|---|---|---|---|
| people | 3,257 | 3,257 | 13,637 | 13,637 | 13,637 | 13,637 | 13,637 | 13,637 | 13,637 | 13,637 |
| food, of need | 1.67 | 1.67 | 0.69 | 1.25 | 0.77 | 1.43 | 0.88 | 1.70 | 1.01 | 1.70 |
| grain in store, a head | | 2.95 | 1.30 | 0.79 | 1.47 | 0.90 | 1.74 | 1.03 | 3.01 | 2.37 |

At the year's end the catchments were redrawn and Uldale, a town of three thousand, became a market of thirteen thousand. Its new people came with their cash and last month's earnings and bought bread in January from shelves stocked for three thousand, out of a trade sized to three thousand. Twenty-three of the thirty hungry town-years look like this; the other seven, all on islands without canals, are towns that took less (three after a redraw of a fifth to all of their people, four after smaller ones or none), most of them hungry later in the year, which I did not trace. Over all of the towns that count, every year, sixteen islands with canals and without:

| the town's people at the year's end before, came or went with redrawn districts | town-years, canals / none | hungry, canals / none |
|---|---|---|
| under a fifth | 32,540 / 33,431 | 0 / 4 |
| a fifth to a half | 212 / 180 | 0 / 2 |
| a half to all of them again (the town less than doubles) | 51 / 47 | 0 / 1 |
| the town doubles or trebles | 29 / 37 | 2 / 4 |
| the town more than trebles | 10 / 27 | 4 / 13 |

A town that more than trebles goes hungry two times in five either way. What the canals change is how many towns do it, and when: 39 doublings over sixteen runs with canals, spread from the 1770s to the 1840s; 64 without, 45 of them in the 1830s and 1840s, when the first railways open; and with neither canals nor railways, 8 (and 0.19 hunger years a run). A railway that comes to a canal-less island moves whole districts at once to the small towns on the line; a canal comes earlier and moves fewer cells, and when the railway follows, the districts have already been drawn to the canal towns.

The check of this (run when the page was reviewed) is canals with no railway ever, on the same sixteen islands:

```bash
./target/release/examples/trace_carriage --seeds 1-16 --set canals.enabled=true,false --set rail_know=99 --out DIR
```

| no railway | canals | none |
|---|---|---|
| hunger years in a run | 0.19 | 0.19 |
| hungry town-years, sixteen runs | 3 | 3 |
| towns that at least doubled | 31 | 8 |
| of them, the town more than trebles | 5 | 1 |
| hungry after a doubling | 1 | 1 |

Without the railway the canals make no difference to the famines at all, though they redraw the catchments four times as often as nothing does: their redraws come early (most of them from the 1760s to the 1820s), are mostly under a trebling, and leave one town of 31 hungry. So the railway adds 8 doublings to an island with canals (39 against 31) and 56 to one without (64 against 8), and that is where the canals' famines went.

Why a doubling empties the shelves is one condition in `peopling::move_with_cells`. When cells change market the people go with them, and so do their cash, last month's income, the carters among them and (by `prices_go_with_trade`) the price; but the merchants' stock, their cash and their smoothed sales (`inv`, `merch_cash`, `outflow`) go only when the receiving market is new (`if st.town_new[j]`). An old town that takes a district keeps the stock and the sales record of the town it was, so its merchants mark their price, order their trade and ration their shelves for the smaller town for a month or two.

The trial of the cure sent the stock, the merchants' cash and the sales record with the people to every market, not only a new one (`ISOSTATE_TRY_SHELVES_GO`, the diff `trial-shelves.diff`), sixteen islands with canals and without:

| | canals, as they are | canals, shelves go | none, as they are | none, shelves go |
|---|---|---|---|---|
| hunger years in a run | 0.38 | 0 | 1.38 | 0.25 |
| hungry town-years, sixteen runs | 6 | 0 | 24 | 5 |
| of them after the town at least doubled | 6 | 0 | 17 | 0 |
| towns that at least doubled, sixteen runs | 39 | 38 | 64 | 34 |
| the worst town's worst month's food, 1801-50 | 1.574 | 1.593 (11/5) | 1.486 | 1.538 (13/3) |
| output a head, 1801-50 | 5.66 | 5.63 (7/9) | 5.61 | 5.63 (8/8) |
| real wage, 1801-50 | 1.761 | 1.752 (8/8) | 1.765 | 1.776 (7/9) |

(The islands higher and lower are each trial against the same setting without it.) With the stock going with the people no town that doubles goes hungry, with canals or without, and what is left of the canals' difference in famines is 0.25 a run against none, from five town-years that were not doublings. Nothing else moves outside the spread. One thing I cannot explain: without canals the trial has 34 doublings where the island as it is has 64. A town's prices and sales after a redraw feed the next year's land bids and so the next year's catchments, and the histories part after the first redraw, so the count is not a fixed property of the island; but I did not follow why it should fall by half.

The same trial on the sixteen islands that were not used here (seeds 17 to 32, run in review with the same diff, and the code put back after): hunger years 0.12 a run with canals and 0.12 without, for 0.69 and 1.56 as the islands are; 2 hungry town-years each way, for 12 and 27; no town hungry after any of its 32 and 55 doublings, where as the islands are 8 of 39 and 26 of 61 were. The canals' difference in famine is gone altogether there, and the four hungry town-years with canals that were not doublings go too. The halving of the doublings does not come back on these islands (55 for 61 without canals, 32 for 39 with), so it was a property of the first sixteen histories and not of the trial. Output a head and the real wage stay inside the spread (1801 to 1850, 6 and 7 islands of sixteen higher), and so does the worst town's worst month, which on these islands is a little lower with the trial where there are canals (1.573 for 1.587, six of sixteen higher) and higher without (1.547 for 1.507, eleven).

### How sure, and what would change my mind

Sure of the cause: 23 of 30 hungry town-years follow a doubling, 20 of those 23 go hungry in the January straight after (one in February, two later in the year), and sending the shelves with the people takes every one of those hungers away in thirty-two runs (sixteen islands, with canals and without), and again in thirty-two more on other islands in review. Sure that grain's carriage, coal and the number of people are not what the canals do: each was measured, and the first was switched both ways (cheaper grain carriage alone may take off a little hunger on an island without canals, but making it dear again on one with them puts none back). Less sure of the canals' part in it, which rests on counts of doublings by decade (39 against 64, 14 against 45 in the 1830s and 1840s) and on the run with neither canals nor railways; I did not follow a single district from the canal to the railway to show it was drawn to the canal town first. The run with canals and no railway, made in review, makes it surer: with no railway the canals leave the famines where they were (0.19 a run either way), so what they take away is the railway's. The count of famine years is small (six hungry town-years in sixteen runs with canals, 24 without) and each run's history differs after the first canal, so the ratio between them is rough; the continuous measure and the table by size of redraw are what to read. What would change my mind: islands where the doublings come as often with canals as without and the famines still fall.

On sixteen islands that were not used here (seeds 17 to 32, run in review, `--seeds 17-32 --set canals.enabled=true,false`) the story holds and is a little less complete. Without canals: 1.56 hunger years a run, 27 hungry town-years, 26 of them after the town at least doubled (21 of those in the January straight after), and 24 of the 35 towns that more than trebled went hungry; 61 doublings, 44 of them in the 1830s and 1840s. With canals: 0.69 a run, 12 hungry town-years, 8 after a doubling; 39 doublings, 15 in those decades. So 34 of 39 hungry town-years follow a doubling, but with canals a third of the hunger that is left is something else, which was not traced.

### What it means for the pass over the rules

- The famine count of the README and the sweep is mostly a measure of how violently the catchments are redrawn, and the canals' "fall from 2.2 to 0.4" is the railway's redrawing made gentler, not food reaching the hungry by water. It should not be read as a fact about canals.
- `peopling::move_with_cells` is the rule to look at: the merchants' stock, cash and sales record should go with the cells to any market, as the people do. The trial is a one-line change to a condition and keeps money and goods exactly (it moves them between purses and shelves); it moves the history of every island after its first redraw, so it belongs to the pass and not to a trace, and it would need its sweep and its page.
- `famine_years` mixes hunger and sailings; it would be clearer as two facts. And the hunger rule ignores a town in its first two years (`people::growth`, `town_founded < year - 2`), while the towns that go hungry are old towns that became, overnight, new ones; whether that grace should follow a redraw is a question for the same pass.

## How to run it again

The example prints every table above but the gain decomposition and the redraw table, which were read from its `--out` files with a short script (in the notes).

```bash
cargo build --release -p isostate-core --example trace_carriage
./target/release/examples/trace_carriage --seeds 1-16 --set rail_know=0.25,99 --out DIR        # (a), (b), (c)
./target/release/examples/trace_carriage --seeds 1-8 --set works.enabled=false --set rail_know=0.25,99 --out DIR   # and prices, machines, people, engel, railways
./target/release/examples/trace_carriage --seeds 1-8 --rules port --families 0 --set carting.enabled=true --set rail_know=0.25,99 --out DIR
./target/release/examples/trace_carriage --seeds 1-8 --set handling=0.03,0 --out DIR             # (b)
./target/release/examples/trace_carriage --seeds 1-16 --set canals.enabled=true,false --out DIR  # (c), (d)
./target/release/examples/trace_carriage --seeds 1-16 --set canals.enabled=false --set rail_know=99 --out DIR
./target/release/examples/trace_carriage --seeds 1-16 --set canals.enabled=true,false --set rail_know=99 --out DIR   # (d), the review's check
```

A run of sixteen islands with two settings takes six to fifteen minutes with twelve threads, by the machine's load. The trials of cures are in the notes as diffs and were never committed.
