# dx-new-towns: towns at wharves, junctions and pit heads, and districts that change market

*Where the code is: this page's verdict is not to build the island's test as it stands, so only the page is on `main`. The prototype behind `newtowns.enabled`, its test and its measuring example are kept on a branch of their own (`night/o-dx-new-towns-check`).*

Branch `night/o-dx-new-towns` (measuring code and a prototype, not for merging). Asked on 12 November 2026 as piece 19 of `docs/backlog/found-along-the-way.md`: new towns, and districts that change market (R70 to R74, R98, R105, C106, C127), with what the reviewers of the canals and of the new land carried on to it (an island with canals founds about one town fewer, since nothing lets a wharf or a junction count for a town; quarry towns are founded where they cannot eat). Four things were asked: measure where and when towns are founded, how often a founding is tried and fails and why, and what every district would save in carting at another market; design rules for a town at a wharf, at a junction of lines and at a pit head, and for a district that changes market with no new town; prototype the smallest whole version behind a switch; and say whether to build it.

It builds on `docs/probes/tr-urban.md` (on main since this probe was written), which found why the founding of towns stops: the test weighs a carting saving priced at a freight fixed in money against wages that treble in money, so by the 1880s the same cart saved is a third of the share of wages it was. That is not measured again here; it is quoted where it bears.

Everything is under the game's rules with six families, at `main` 11dbe7d. The machine was shared with a great many other jobs while this was measured, so no timing here means much. The measuring program is `crates/isostate-core/examples/new_towns.rs`. The prototype is `crates/isostate-core/src/newtowns.rs`, behind `newtowns.enabled`, which is off under both sets of rules; with it off a sweep of seeds 1 to 8 gives every one of the 446 facts as main does, and the core's tests pass, the golden test among them. Output and notes are under `.claude/night/reports/opus/dx-new-towns/` (`base-s1-16.txt` for sixteen islands as they are, `land-s1-8.txt` with the new land on, `bulk.txt` and `bulk/` for the prototype's sweep, `proto-*.txt` for the prototype measured the same way as the base, and `build.md`, the notes, with every command).

## In short

1. **Where and when.** On sixteen islands 11.2 towns a seed are founded: 4.0 in 1750-74, 2.6 in the next quarter-century, then 2.3, 1.7, 0.5 and 0.06. Of the 4.6 a seed founded from 1800, half are quarry towns (2.4), then works (0.9), farms (0.75) and coal (0.5). 0.94 a seed stand at a wharf, 0.25 at a junction of lines, 0.5 by a railway. Wharves and junctions are a fifth of the places a town could stand by 1860 and 1900, and a fifth and a twentieth of the late foundings.
2. **Tries and failures.** The rule looks at six sites every year, and in no year on sixteen islands has it fewer. In the 1750s a site fails mostly because a town nearby has just given people to a founding (half of all sites) or would give too many (a fifth). From the 1800s it fails on the saving, 37 sites in 60 a decade, then 54 in the 1840s and 59 in the 1890s. The exact try on the real catchment fails twice in 16 runs: the estimate is good. Sites at wharves and junctions fail on the saving 88 and 98 times in 100, because carriage along a canal or a line is already cheap: their producers' saving is 0.037 and 0.030 of the district's wages at the median, against 0.078 for a site on none of these.
3. **What the island saves.** The founding test counts what the district's producers save in carting to a nearer market. Counted for the island instead, a new market also has to carry back to the old town everything the district sold to the old town's people and works, and fetch what its own households buy that it does not make. Done so, every town founded and all but nine of the 14,493 sites looked at in sixteen runs cost the island more carting than they save (the nine are one coal district at a railway junction, section 3): a late quarry town's producers save 0.104 of its wages and the island pays 0.295 to carry between the new market and the old. Goods that went on from the old market to a third town hardly count (0.015 of wages, and the wrong way).
4. **Districts that sell elsewhere.** Hub and spoke costs little. If every cell sold the goods its market sends on straight to where they go, the island would save 0.2 to 0.5 per cent of its wages in carting; a cell whose saving would pass a twentieth of its own wages is one in two or three hundred. A cell that traded through another market altogether would save carting nowhere, on any of sixteen islands at any of five marks.
5. **But the island's test is wrong about the island over time.** The prototype founds where the island's saving is positive. At the model's one freight rate that is nowhere, and an island with its six towns of 1750 and no others has more carters, not fewer (0.077 of its hands in 1900 against 0.064), a third fewer works and 6 per cent fewer people. With the legs between markets taken at three tenths of a farmer's cart (`newtowns.bulk` 0.3, a stand-in for merchants' wagons and boats, which the model does not have), towns are founded until 1884, 26.5 stand in 1900 for 17.8, and half the late ones stand by a railway, five a seed at junctions; but they are there because a district on a line saves little and costs little, so the sum clears a bar of nothing, not for any goods that change trains. Carters are 0.074 of hands, baskets a head 1.8 per cent fewer, the share living in towns 0.413 for 0.419. The carters' share of hands is least near today's count of towns, under every setting tried.
6. **Whether to build it: no, not as prototyped.** The island's sum is the right question and the wrong instrument: it is taken on the day, before land, works and the railway rearrange round the new market, and it can only be made to found towns by a number (`bulk`) that the economy does not use. What the prototype and the measurements say to build instead is in section 6: a margin before a district changes market, villages at pit heads, and wharves and stations as places with a charge, before any rule founds a town at them.

## 1. Where and when towns are founded

`cargo run --release -p isostate-core --example new_towns -- --seeds 1-16`, part 1. Each founding is listed in `base-s1-16.txt` with its year, its district's kind, the place it stands on, its cells and people, the test's terms and its people in 1900.

| | 1750-74 | 1775-99 | 1800-24 | 1825-49 | 1850-74 | 1875-1900 |
|---|---|---|---|---|---|---|
| towns founded, a seed | 4.00 | 2.62 | 2.31 | 1.69 | 0.50 | 0.06 |

In 1900 17.2 towns stand (12 to 20). Of the 73 founded from 1800, a seed: quarry and claypit districts 2.38, works 0.88, farms 0.75, coal 0.50, iron 0.06.

Where they stand, against how common each kind of place is among the cells a town could stand on (land, lowland or hills, with a way to market, more than `found_spacing` from every town):

| | late foundings, a seed | of the late foundings | of the places a town could stand, 1800 | 1860 | 1899 |
|---|---|---|---|---|---|
| a wharf (a canal on the cell or next to it) | 0.94 | 0.21 | 0.16 | 0.22 | 0.22 |
| a junction (track on the cell and on three of its neighbours, or line and canal within a cell) | 0.25 | 0.05 | 0 | 0.16 | 0.21 |
| within a cell of a pit head | 3.69 | 0.81 | 0.27 | 0.38 | 0.42 |
| on a pit head's own cell | 1.00 | 0.22 | | | |
| by a railway | 0.50 | 0.11 | | | |
| on the coast | 0.50 | 0.11 | | | |

So a late town is twice as likely as a place chosen at random to stand by a pit head (the rule looks where the tonnage is), as likely to stand at a wharf, and a quarter as likely to stand at a junction. The four junction towns of sixteen runs (seeds 2, 5, 8 and 12, 1856 to 1886) are all quarry towns on a railway.

**Do they eat?** In the first three whole years of a late town its worst month gave 1.68 of the food it needed at the median, and two of 73 had a month under 0.82, the people page's line for a hungry year. In its first year 0.57 of its hands are on farms, pastures and woods (quarry towns 0.48). With the new land switched on (`--set land.enabled=true`, seeds 1 to 8, `land-s1-8.txt`) a late quarry town has 0.40 of its hands on the land, the land's reviewer's point, but its worst month is still 1.53 of need and none of 41 late towns had a hungry year. They are founded where they cannot feed themselves, and they buy their food in. What they pay for it is R72: their prices for what they must fetch open at the old market's plus the carriage.

## 2. How often a founding is tried and fails, and why

The people rule (`peopling::found_towns`) takes the six spots where the carting bill is heaviest, settles each on the best ground within two cells, and tries them in order of what the district's producers would save. A site fails on the saving (under `found_saving`, 0.08 of the district's wages), on its size (40 cells and 3,000 people), on a donor that would give more than a fifth of its people, or on a donor's rest (eight years after it gave to a founding). The first site that passes is tried on the real catchment, which can fail too. With `newtowns.log` set the rule keeps every site and its fate (`st.nt_tries`); nothing else changes.

Sites a seed in each decade, of the sixty it looks at (part 2):

| | founded | the saving | too small | a donor too large | a donor resting | passed, then failed on the real catchment | after a founding that year |
|---|---|---|---|---|---|---|---|
| 1750s | 2.1 | 4.5 | 3.4 | 12.0 | 30.5 | 0 | 7.5 |
| 1780s | 1.0 | 24.4 | 6.0 | 11.4 | 13.9 | 0 | 3.3 |
| 1800s | 0.6 | 36.7 | 8.9 | 8.5 | 3.4 | 0 | 1.9 |
| 1820s | 1.1 | 39.1 | 5.2 | 8.1 | 3.2 | 0.06 | 3.3 |
| 1840s | 0.3 | 54.0 | 2.5 | 0.3 | 1.9 | 0 | 0.9 |
| 1860s | 0.2 | 56.7 | 1.8 | 0 | 0.6 | 0 | 0.8 |
| 1890s | 0 | 59.3 | 0.7 | 0 | 0 | 0 | 0 |

- **Early, the donors hold founding back**, not the saving: in the 1750s half of all sites wait for a donor's eight years of rest, and a fifth would take too much of a town. The rest is what spaces the first rush (2.1 a decade in the 1750s where the port has seven or more).
- **From 1800 it is the saving**, as tr-urban found, for the reason it found. Not on every island: on seeds 17 and 18 (`--seeds 17-18`, the review's two islands) a donor that would give too much of its people turns away more sites than the saving until the 1820s (32.5 sites a seed against 14.5 in the 1800s, 24.5 against 27.5 in the 1820s), and the saving takes over only from the 1830s.
- **The real try almost never fails** (twice in sixteen runs). The estimate on last year's paths is as good as the exact catchment, and the limit of two tries a year has never been reached.
- **Wharves and junctions fail on the saving.** Of 276 wharf sites a seed (over the run) 243 fail the saving; of 180 junction sites, 178. Their producers' saving is 0.037 and 0.030 of wages at the median (sites by a railway, 0.030; sites on none of these, 0.078): a district on a canal or a line already carts cheaply to its market, so a market of its own saves it little. This is the canals' reviewer's "one town fewer", seen from the sites.

## 3. What a district would save at another market

### The island's side of a founding

The test counts what each cell of the district would save carting its tonnage to the new market rather than the old (`freight` times tonnage times the path cost saved). That is what the producers save. The island's sum differs in three places, each worked out by weight from what moved in the year (`newtowns::island_terms`):

- **Onward.** What the old market sent on to a third town now goes from the new market straight there. Per ton, the leg from the old market to the third town is saved and the leg from the site to it is paid. For a site on the way to where the goods go, this is a saving; behind the old town, a cost.
- **Back to the old town.** What the district sold that the old town's people and works used has to be carried from the new market to the old.
- **Bought in.** What the district's households bought at the old market that the district did not make is now fetched to the new one. (Their own goods they keep: their households' share of what the old market used, by weight, stays at the new market.)

The last two are the cost of a new market. Before the founding a household's goods reached it free from its market, as the model has it (a household pays the town price and no carting); after, the merchants of the new market pay to bring them.

Medians over wages, for the 73 towns founded from 1800 on sixteen islands, on the day:

| district | towns | producers' saving | onward | back and bought in (of which bought in) | the island's |
|---|---|---|---|---|---|
| quarry | 38 | 0.104 | -0.015 | 0.295 (0.110) | -0.203 |
| coal | 8 | 0.092 | -0.012 | 0.247 (0.077) | -0.140 |
| works | 14 | 0.087 | -0.004 | 0.223 (0.072) | -0.131 |
| farm | 12 | 0.093 | -0.020 | 0.316 (0.136) | -0.216 |

And over every site looked at, by half-century: the producers' saving 0.101, 0.057 and 0.031 of wages, the island's -0.208, -0.121 and -0.074. Nine of the 14,493 sites in sixteen runs had an island's saving above nothing, and all nine are one place: a coal district at a railway junction on seed 13, looked at in nine years from 1836 to 1848, above nothing by at most 0.005 of its wages, and passed over each time on the producers' saving. (The table of section 2 by place prints that share as 0.00; the example now lists every such site. On seeds 17 and 18, which this page did not otherwise use, it is 15 sites of 1,812, coal districts of seed 18 at junctions and by a pit head, from 1835 to 1865, by at most 0.021, passed over on the saving or a donor's share.) So the island's sum is not against every founding: where it is for one, it is a colliery district on a line, the one kind of junction town the designs below look for, but by a hair, and never by enough to matter. Onward goods hardly matter: the median site is behind its old market, not on the way to anywhere, and what a market sends on is a small part of what it handles.

This is R70 and C106 measured: the founding test credits the district with a saving that the island pays back, and more than pays back.

### Selling where the goods go, or changing market

For every cell at five marks (part 3), by weight: (i) what its goods that its market sends on would save if the cell sold them straight to the town they go to; (ii) whether the whole cell, its people and its goods, would cart less trading through another market.

| | carting bill, cells to market, over the island's wages | towns to towns | (i) onward goods sold where they go | cells where (i) is over a twentieth of their wages | (ii) cells where a whole change saves |
|---|---|---|---|---|---|
| 1760 | 0.124 | 0.023 | 0.0035 | 0.3 per cent | none |
| 1800 | 0.092 | 0.037 | 0.0049 | 0.3 per cent | none |
| 1830 | 0.071 | 0.037 | 0.0042 | 0.5 per cent | none |
| 1860 | 0.049 | 0.028 | 0.0028 | 0.3 per cent | none |
| 1899 | 0.038 | 0.022 | 0.0023 | 0.04 per cent | none |

(Means over sixteen islands; the spread is in `base-s1-16.txt`.) The detour through the market costs the island a tenth to a seventh of what it pays for carting between towns, and a two-hundredth of its wages at most. (All of it by weight, every good alike, as in the island's side above.) A cell at the edge of two markets' areas already trades through the nearer; sending its goods the other way would carry what the old market's people use further, and its people's purchases would have to be fetched. Only the goods its market sends on could ever favour the other market, and they are a small part of the tonnage, so the last column's "none" is as much the arithmetic of nearest-path areas as a finding about the islands. So by carting alone nothing calls for a district to change market that the nearest-path rule does not already move. What does move districts is a new road, canal or line that makes another market the nearer, and then people, cash, prices and wage go with the cells (`peopling::move_with_cells`). tr-urban found that close to half the growth of a town's fastest decade comes that way.

## 4. Designs

What each would need that the simulation does not have is the point of each.

### A town at a wharf

Today every cell of a canal is a wharf: goods go on and off the water anywhere, at no charge, so a district beside a canal carts cheaply to whatever market the canal reaches, and a market of its own saves it little (section 2). Historically a canal town grew where goods changed from cart to boat and back: at the basin, where a road met the water, where the boats were loaded and the goods stored.

The rule: a wharf is a place, not a cell of water. Goods reach the water only at wharves, and each ton loaded pays a charge for loading and storing. A district's carts converge on its wharf; a market at the wharf saves the district the cart to the old town and the island the loading twice, once onto the water toward the old town and once off it there. A town is founded at a wharf where that saving, counted for the island, passes the test.

What it needs: wharves as places on the canal with a loading charge (the canals page has none; piece 16 already has "a river navigation is not told from a canal"); a carriage between markets cheaper per ton than a farmer's cart, or the island's sum is always against a new market (section 5); and traffic by wharf, which is in no layer today (piece 9).

### A town at a junction of lines

Every cell of track is a station (R90, `docs/rules/railways.md`), so a junction is only a cell where lines cross, and nothing changes trains there. The rule would be the wharf's: goods join and leave the line only at stations, a ton changing trains or changing from line to canal pays a charge, and a town is founded at a junction where the goods changing there would save that charge and the leg to the old market. It needs stations as places (piece 6, and the several days of piece 16), a charge for changing, and traffic by station. The four junction towns of sixteen runs are quarry towns that happen to stand on a line; none of them is there because of the junction.

### A town at a pit head

The rule already finds pit heads: four late towns in five stand within a cell of one, against two places in five. Two things are wrong with those towns. Their test counts a saving the island does not get: a quarry town's stone mostly goes back past the old town, and its food comes the same way. And with the new land many stand where little grows (four in ten of their hands on the land), which does no harm to their tables today but makes for a town that is a market only in name.

What a pit head historically got was a village: the pit's people housed beside it, trading through the old market town. The model has no such place. Every person belongs to a town, and every town is a market. The rule: a pit head with more than so many hands gets a village of its people on the ground beside it, counted as the old town's people, drawn as houses and not as a town, with no market, wage or prices of its own; it becomes a town only if the founding test passes for it. That needs a second kind of place in the state (people on ground, under a town), the town ground's rules for it (`towns.md`), and the viewer to draw it; the houses' generators exist. It would also answer R105 (a harbour town on a spit with no room) in part: a village needs no market area.

### A district that changes market with no new town

Section 3 says carting gives no reason for a district to change market other than becoming nearer to another, which is what happens now, and that selling the goods straight where they go is worth a few thousandths of the island's wages. So no rule for a better market is wanted. What is wanted is the stability R73 asks for: markets side by side swap districts on a hair's difference in path cost, and a town's people jump when they do. The rule: a cell changes market only when the new market is nearer by a margin (say a tenth of its path cost) or has been nearer for two years running. It needs nothing the simulation does not have: `land::update_transport` keeps last year's catchment already (`st.prev_catch`). It would change every catchment that a road, canal or line moves, and so every fact; it wants its own sweep.

## 5. The prototype

The smallest whole version of the four designs that the simulation can hold today is the founding test itself: count the island's saving, not the producers', and look at wharves, junctions and pit heads besides the six busiest spots. `newtowns.rs`, behind `newtowns.enabled`:

- the test is `saved + bulk * (onward - back - bought in) >= saving * wages`, with the terms of section 3 and `saving` 0 (the island must gain something);
- besides the people rule's six spots, the two busiest wharves, junctions and pit heads (`per_kind` 2) are looked at each year, each on its own cell, and the sites are taken in order of the island's sum;
- `bulk` scales the legs between markets (onward, back and bought in). At 1 it is the model as it is. Below 1 it stands for merchants' wagons and boats carrying a ton between markets more cheaply than a farmer's cart takes it to market, which the model does not have: `freight` is one rate for both. It is in the test only; the economy still charges what it charges.
- the rest of the people rule (size, donors, the real try, how people go with their cells) is as it was.

Off, nothing reads it, and `newtowns.log` alone changes nothing (the log is a list in the state; the measuring program sets it).

### What it moved

`isostate-core sweep --seeds 1-8 --rules game --families 6 --set newtowns.enabled=true --set newtowns.bulk=1,0.5,0.3,0.15 --compare merged-game-f6-l`; 32 runs, none failed (each checks that money is conserved every month). Means over seeds 1 to 8; in brackets, islands up and down where it matters.

| | main | bulk 1 | 0.5 | 0.3 | 0.15 |
|---|---|---|---|---|---|
| towns in 1800, 1850, 1900 | 12.6, 16.9, 17.8 | 6, 6, 6 | 7.5, 9.5, 11 | 13.4, 21.2, 26.5 | 16.4, 27, 37.8 |
| the last founded | 1847 | none | 1877 | 1884 | 1897 |
| founded from 1800 | 5.1 | 0 | 3.8 | 13.2 | 21.8 |
| of those by a railway | 0.8 | 0 | 2.8 | 6.6 | 12.5 |
| of those quarry, farm, coal | 2.4, 1.1, 0.8 | | 1.8, 0.6, 0.9 | 5.9, 4.9, 1.8 | 8.6, 10.2, 1.6 |
| people of the middle late town, 1900 | 20,318 | | 32,983 | 16,975 | 10,595 |
| share living in towns, 1850 and 1900 | 0.344, 0.419 | 0.353, 0.408 (0/8) | 0.347, 0.411 | 0.341, 0.413 | 0.324, 0.407 |
| carters' share of hands, 1850 and 1900 | 0.083, 0.064 | 0.086, 0.077 (7/1) | 0.081, 0.067 | 0.091, 0.074 | 0.093, 0.074 (8/0) |
| carting's share of output, 1800, 1850, 1900 | 0.079, 0.052, 0.044 | 0.089, 0.058, 0.058 | 0.085, 0.054, 0.048 | 0.080, 0.057, 0.050 | 0.080, 0.057, 0.049 |
| what a wage buys, 1900 (1750 = 1) | 2.60 | 2.84 (7/1) | 2.81 | 2.58 | 2.49 |
| baskets a head, 1850 and 1900 | 5.18, 8.76 | 5.36, 8.68 | 5.24, 8.66 | 5.17, 8.60 | 4.69, 8.31 |
| people in 1900 | 731,900 | 688,800 (0/8) | 705,500 | 745,300 | 765,600 (8/0) |
| works in 1900 | 738 | 484 (0/8) | 597 | 873 | 1,040 (8/0) |
| railway and canal in 1900, km | 2,248, 892 | 2,169, 462 | 2,358, 583 | 2,356, 914 | 2,536, 1,022 |
| famine years in a run | 0.4 | 0.1 | 0.4 | 0.8 | 0.9 |

- **At the model's own rate the island's test founds nothing**, not even the rush of the 1750s, and the island keeps its six towns. It then has more carters (0.077 of its hands in 1900 for 0.064) and a larger carting bill (0.058 of output for 0.044), a third fewer works, half the canal and 6 per cent fewer people. What a wage buys in 1900 is higher (2.84), but baskets a head are not (8.68 for 8.76): fewer people on the same land, poorer in the goods the towns make. So a test that says on the day that every founding costs the island carting is contradicted by the island: without the foundings it carts more. The day's sum leaves out what a market does after it stands, the rings of land use that form round it, the works that come to it, the canal and railway that are dug to it.
- **With the legs between markets at half the cart's rate**, a third fewer towns than now, founded later and larger, three in four of the late ones by a railway.
- **At three tenths** there are half as many towns again as now, founded into the 1880s, and half the late ones stand by a railway: the railway towns the people page says no rule makes (R74). But they are quarry and farm towns that a line passes, not junction towns, carting takes more hands than now, and baskets a head are 1.8 per cent fewer. This is tr-urban's island that goes on founding, reached by another road.
- **At fifteen hundredths** the island is crowded with 38 towns of 10,600 people at the middle, a tenth more railway, 5 per cent fewer baskets a head and every island with more carters than now.
- **The share living in towns does not rise** under any setting (0.407 to 0.413 for 0.419), as tr-urban found of more towns by any road.
- **Carting is least near today's number of towns.** Carters' share of hands in 1900: 0.077 with six towns, 0.064 with eighteen, 0.067 with eleven, 0.074 with 26 or 38. The producers' test, crude as it is, lands near the least carting by no design of its own.

### Where its towns stand

The measuring program with the prototype on, seeds 1 to 8 (`--set newtowns.enabled=true --set newtowns.bulk=0.3`, `proto-bulk03.txt`; and 0.5, `proto-bulk05.txt`), set beside the sixteen islands of section 1 (a seed, from 1800):

| | main | bulk 0.5 | bulk 0.3 |
|---|---|---|---|
| founded from 1800 | 4.6 | 3.8 | 13.3 |
| at a wharf | 0.94 | 1.25 | 5.0 |
| at a junction | 0.25 | 2.0 | 5.0 |
| within a cell of a pit head (on its cell) | 3.7 (1.0) | 3.4 (1.1) | 10.9 (2.5) |
| by a railway | 0.5 | 2.75 | 6.6 |
| quarry, farm, coal | 2.4, 0.75, 0.5 | 1.75, 0.6, 0.9 | 5.9, 4.9, 1.75 |
| a hungry month in the first three years | 2 of 73 | none of 30 | none of 106 |

So the prototype does found towns at junctions, a fifth to a half of its late towns, from the 1840s to the 1890s. Looked at one by one (seed 1 at 0.3: thirteen junction towns from 1841 to 1900, seven of them farming districts), they are not there for the goods that change trains. Their onward term is nothing (0.000 to -0.007 of wages). A district on a line carts cheaply both ways, so its producers save little (0.012 to 0.053 of wages) and the carriage back to the old town costs little too (0.03 to 0.12, before `bulk`), and a sum of two small numbers passes a bar of nothing easily. They are the farm towns of a line, strung along it as closely as the spacing of eight cells allows. That is not a bad picture of a railway age, but it is made by the bar and `bulk`, not by anything a junction does. At the model's own rate, a handful of sites in these runs had an island's saving above nothing (none at 0.3 on seeds 1 to 8; at 0.5, eleven, all coal districts, nine of them one district at a junction on seed 1 from 1883 to 1897; at 0.3 on seeds 9 to 16, 58, all coal districts at junctions on seeds 10, 11 and 13), by at most 0.018 of wages, every one passed over on a donor's share; none of the prototype's junction towns is among them.

### Checked on sixteen islands

The review ran the measuring program with the prototype on at 0.3 on all sixteen islands (`--seeds 1-16 --set newtowns.enabled=true --set newtowns.bulk=0.3`), and at the model's rate (`--set newtowns.enabled=true`), looking for what could go wrong:

- **Rushes.** The rule founds at most one town a year, and so it does: no seed has two in a year, and the most in any five years is three. 25.2 towns stand in 1900 (15 to 36), 12.25 a seed founded from 1800, the last of them in 1893 or later on seven islands of sixteen and before 1860 on three.
- **Towns that cannot eat.** None of 196 late towns had a month under 0.82 of its food in its first three years (worst month 1.81 of need at the median). Over the whole run the towns founded have 0.8 town-years a seed with a hungry month, against 0.6 under main's rule; the six towns of 1750 none.
- **Districts that change market back and forth.** Under main's rule a seed sees 7,809 cells change market over the run, and 879 of those changes go back to the market the cell left within ten years. With the prototype at 0.3, 9,569 and 850; at the model's rate, with six towns, 3,227 and 678. More towns move more cells, but not back and forth more.
- **Money.** Every run of the program and of the sweep checks every month that money only changed by what was issued; none failed.

### What it leaves out

- It is a sum taken on the day, on last year's flows. What the island does in the years after a founding (land, works, ways) is not in it, and that is what decides the carting.
- It weighs everything by weight: a ton of coal and a ton of cloth alike, at last year's tonnage and trade between towns. The books carry trade good by good and could replace that.
- `bulk` stands for something the economy does not do. Until merchants' carriage between markets is cheaper than a farmer's cart in `econ::step_month` itself (piece 15, carting that costs), a test that assumes it is a test of another island.
- Nothing is a wharf or a station: a wharf is a cell by a canal, a junction a cell of track with track round it. A pit head is any working within a cell, which takes in two places in five where a town could stand.
- The new town's first year is still its donor's whole (R98), and its people still pay the old market's prices and the carriage for what they buy in (R72). Neither was touched.
- Seeds 1 to 8 only for the sweep; the measurements are on sixteen, and the prototype at 0.3 was measured on sixteen by the review.

## 6. Whether to build it

Not as prototyped. Three reasons, in order:

1. **The island's sum on the day points the wrong way over time.** It says every founding costs the island carting; the island without foundings carts more. A rule built on it would need a number (`bulk`) that the economy does not use, and the number decides everything: half as many towns again at 0.3, a third fewer at 0.5.
2. **Wharves and junctions do not make towns because they are not places.** A canal is a wharf all along and a line a station all along, so a market by them saves the district little and nothing converges on them. The prototype's junction towns are farm towns strung along a line, there because both sides of the sum are small; no founding test can find a junction town for the junction's sake at a place the economy does not have.
3. **More towns do not bring more people into towns**, and they cost a little of what a wage buys, here as in tr-urban.

What to build instead, in this order, each small:

- **A margin before a district changes market** (section 4): the cure for R73, a day's work and a sweep, and the one design here that needs nothing new.
- **Villages at pit heads**, when the towns' second pass and the houses can take them: the pit's people on the ground beside it, under the old market. It gives the map what a pit head looked like without a market that is a market only in name.
- **Wharves and stations as places with a charge**, with piece 16 (a railway company) and piece 6 (stations), and then, and only then, a founding test that counts the charge saved. Before that, the island's sum would have no reason to prefer a wharf.
- **Leave the founding test as it is until piece 15 settles carting**, which settles tr-urban's money wage with it. The test in 1750's money (tr-urban's trial, 24 towns, the last in 1878) is the smallest change that keeps founding going, if the map wants towns founded late.

## 7. How to run it again

```bash
cargo run --release -p isostate-core --example new_towns -- --seeds 1-16                                  # the measurements, as main stands
cargo run --release -p isostate-core --example new_towns -- --seeds 1-8 --set land.enabled=true           # with the new land
cargo run --release -p isostate-core --example new_towns -- --seeds 1-8 --set newtowns.enabled=true --set newtowns.bulk=0.3
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules game --families 6 --set newtowns.enabled=true --set newtowns.bulk=1,0.5,0.3,0.15 --out DIR --compare MAIN
```

where `MAIN` is a sweep of main with no `--set`. The example always runs the game's rules with six families and `newtowns.log` on; it plays each island by the month and reads the state between December and the yearly step at five marks, so it takes about as long as a sweep of the same seeds.
