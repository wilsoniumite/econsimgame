# tr-urban: why the towns stop at four people in ten

Branch `night/o-trace-urban`, one of the traces with the books of account (piece 12 of `docs/backlog/found-along-the-way.md`). Asked on 12 November 2026: the share of people living in towns reaches 0.42 by 1900 under the game's rules and the plan asks for a half (R36). Three questions: (a) what holds the share there; (b) why the founding of towns stops in the 1840s; (c) what limits a single town's growth in the decades it grows fastest.

Everything is under the game's rules with six families, at `main` a6ff0ec, on seeds 1 to 8 and again on seeds 9 to 16 where it says so. The measuring program is `crates/isostate-core/examples/tr_urban.rs`; it plays each island month by month with the books on and prints the three parts. Its output, the sweeps and the notes are under `.claude/night/reports/opus/trace-urban/` (`base.txt` and `base-s9.txt` for the islands as they are, `c-*.txt` for part (c) with a number changed, `sweeps/` for the sweeps, each compared with `merged-game-f6-l`, today's main). Nothing under `crates/isostate-core/src` or `data/` was changed. The three mends that were tried were built, swept and taken out again; their differences are kept beside the sweeps (`trial-*.diff`). The page was then checked on `night/o-trace-urban-check`: the example run again on seeds 1 to 8 gives the same output to the byte, it was run on seeds 17 and 18, which the trace had not used, and it was given a few more lines (real income against the Engel reference, the saving of the sites that fail on saving alone at 1750's money wage, the medians of a fast decade's parts); those runs and one more sweep are under `check/`.

## In short

1. **The share living in towns is a count of the works' hands, not of where anybody lives.** Every year each town's town people are set to `people x (0.04 + 0.8 x works' hands / labour force)`, the port's rule, kept by kept town ground (`towns::update_urban`; under the port `land::update_urban`), and then smoothed half and half with last year's. So the island's share is, but for the smoothing, 0.04 plus eight tenths of the share of its hands that work in works, to within a thousandth at every mark on sixteen islands. 0.42 in 1900 is 48 per cent of the hands in works; a half needs 57.5 per cent. (`dx-towns.md` found the same identity under the port.)
2. **So what holds the share is where the other 52 per cent of hands work**: on farms, pastures and in woods (24 per cent in 1900), in quarries and clay pits (15), in iron mines and collieries (5) and on the road (6). Of the five suspects in the question, four do nothing or next to nothing when switched: there is no gap between what a wage buys in town and in the country (a town's district is one labour market and works and land pay the same wage, to 1 or 2 per cent), the ground a town can take never binds, how fast people move does not move it, and more towns do not raise it (28 towns for 18 gives 0.413 for 0.419). The fifth, food, is the strongest lever there is: with no Engel curve the share in 1900 is 0.28; with one twice as steep, 0.43. Carriage counts only through the carters, whom the rule leaves in the country.
3. **The largest single thing holding it is the quarrymen.** Households buy their shelter as raw stone, and one hand in seven is in a quarry or a clay pit by 1850. Moved into works they would make the share 0.545 in 1900 by the same arithmetic. The production tree, which puts builders between the quarry and the house, does most of that on its own branch: quarry hands 0.025 for 0.141 and the share 0.465 for 0.420, on sixteen islands of sixteen. Taking stone out of shelter by one number on main gives 0.466, higher on eight islands of eight, and costs 7 per cent of the people.
4. **Founding stops because the test is in money and the money wage trebles.** A town is founded where a market would save `found_saving` (0.08) of its district's wages in carting. The carting is priced at `freight`, a fixed sum of money, and the wages in money: by the 1880s the island's money wage is 2.7 times 1750's, so the same cart saved is a third of the share of wages it was. The best site's saving falls from 0.21 of wages in the 1750s to 0.05 in the 1880s; measured against wages at 1750's money wage it is 0.20 and 0.14, and never under 0.08. The sites that could be founded, those that pass every other test, are nearer the line: at 1750's money wage they offer 0.07 to 0.08 from the 1860s, so in 1750's money founding goes on, but only just. Loosening the other gate, the donor test, does not keep it going. With the test put in 1750's money, the last town is founded in 1878 on average, later than now on all eight islands, and 24 towns stand in 1900 for 18. If it had gone on: more and smaller towns, a tenth more railway, a sixth more works, a few per cent more people, and a share living in towns that does not rise.
5. **A town grows fastest by drawing migrants, or, for a few towns, by a jump in its market area; neither housing nor the cap on arrivals holds it.** In each town's fastest decade (4.3 per cent a year on average) the typical town grows by migrants and births: over the 141 towns the median parts are 1.5 points a year of migration, 1.3 of births less deaths and 0.2 of people on cells that came to its market, and migration is the largest part in 72 of the decades. A minority (41) grow mostly by cells that changed market, some by 20 to 30 points a year, and they carry the mean of that part to 2.2 points, close to half of the mean growth. Before 1850 the jumps lead; from 1850 migration does (2.4 points a year for 1.1 with cells). Housing never binds (on sixteen islands no town past its tenth year ever had more town people than its ground has room for, and with less room a town only takes ground faster), and the limit on arrivals binds in one year in twenty-five of those decades; loosening it changes nothing. Births less deaths sit within a twentieth of their ceiling in nine town-years in ten after 1850, which is what `growth_capped_*` and `town_growth_pct_p90` (1.50) show, but every town is held there alike. Migrants follow income a head; with people three times as ready to move the fastest decades reach 4.8 per cent a year and the arrivals limit begins to bind. So of the three the question names, neither the cap nor housing sets a fast town apart: what does is the pull of its income a head on the people of the towns round it (work, in so far as work is what makes that income), and `migration_rate` is how hard that pulls.

## (a) What holds the share at 0.42

### The count

`towns::update_urban` (towns.rs, line 320 at a6ff0ec), for every town, every year:

```rust
let share = 0.04 + 0.8 * st.ind_labor[i] / (P.participation * st.pop[i]).max(1.0);
let raw = st.pop[i] * share.clamp(0.06, 0.95);
let U = if st.urban_pop[i] > 0.0 { tp.smooth * st.urban_pop[i] + (1.0 - tp.smooth) * raw } else { raw };
```

`ind_labor` is the year's mean of the town's works' hands (`towns::year_hands`), `participation` is 0.5, and the observer's `urban_share` is the sum of `U` over the island's people. The town ground then grows until it has room for `U` (`towns.md`). Nothing about the ground, the rent, or anybody's decision to move enters the count: a migrant who comes to a town adds to its people, and to its town people only in so far as the town's works' share of hands is what it is. Summed over towns, the island's share before smoothing is `0.04 + 0.8 x` the island's works' hands over its labour force, but for the clamps, which make no difference at any mark (the town-by-town sum and the island's sum agree to a thousandth).

### Where the hands work

From the books (`TOWN_FIELDS` `hands_works`, `carters`, `navvies`; `LAND_FIELDS` `hands` by sector; `METHOD_FIELDS` for services), as shares of the labour force, means over seeds 1 to 8 with the least and greatest seed; after each year's step (`cargo run --release -p isostate-core --example tr_urban -- --seeds 1-8`, part (a)):

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| living in towns, the towns' count | 0.219 | 0.268 [0.246 .. 0.310] | 0.347 [0.311 .. 0.410] | 0.419 [0.401 .. 0.431] |
| 0.04 + 0.8 x the works' share, island | 0.218 | 0.274 | 0.356 | 0.424 [0.400 .. 0.438] |
| hands in works | 0.223 | 0.293 | 0.394 | 0.480 [0.450 .. 0.497] |
| of which shops and trades (`services`) | 0.125 | 0.186 | 0.264 | 0.359 |
| on farms, pastures and in woods | 0.558 | 0.420 | 0.310 | 0.242 [0.228 .. 0.264] |
| in quarries and clay pits | 0.080 | 0.114 | 0.151 | 0.152 [0.141 .. 0.167] |
| in iron mines and collieries | 0.010 | 0.044 | 0.050 | 0.049 |
| carting | 0.125 | 0.117 | 0.082 | 0.064 |
| digging canals, or not at work | 0.003 | 0.012 | 0.013 | 0.013 |
| a works hand's pay over a land hand's | 0.99 | 1.03 | 1.02 | 1.01 |
| town people over the room of the towns' ground | 0.94 | 0.80 | 0.88 | 0.86 |

The count trails the rule by half a point because of the smoothing. Seeds 9 to 16 give the same table to a point or two (`base-s9.txt`: 0.422 in 1900, works 0.486, farms 0.232, quarries and clay pits 0.152, carters 0.067), and so do seeds 17 and 18 (`check/s17-18.txt`: 0.416, works 0.480, quarries and clay pits 0.141, carters 0.075). As twenty-year means the share is 0.215 (1750-69), 0.267 (1790-1809), 0.347 (1840-59), 0.406 (1860-79) and 0.420 (1880-1900) on seeds 1 to 8, and 0.216, 0.255, 0.353, 0.413 and 0.422 on seeds 9 to 16: it climbs until about 1875 and then stops, with the works' share, which stops at 0.48.

Two sums from the same table, arithmetic and not experiments: with the quarrymen and clay diggers in works the rule would give 0.545 in 1900 (0.551 on seeds 9 to 16), and with the carters counted as town people 0.475.

### Switching each suspect

Sweeps of eight islands against today's main (`isostate-core sweep --seeds 1-8 --rules game --families 6 --set KEY=V1,V2 --compare merged-game-f6-l`). The share is the sweep's `urban_share_Y`, the year's mean; "up/down" is seeds higher and lower than main.

| what was changed | the suspect | share 1850 | share 1900 | up/down | what else moved |
|---|---|---|---|---|---|
| nothing (main) | | 0.344 | 0.419 | | 17.8 towns, last founded 1847 |
| `urban_rent_rate` 0 (0.12) | what town life costs over the country's | 0.345 | 0.424 | 7/1 | people +1.7 per cent (8/0) |
| `urban_rent_rate` 0.24 | | 0.340 | 0.411 | 0/8 | people -2.0 per cent (0/8) |
| `towns.d_old_growth` 3 (6) | the ground a town can take | 0.340 | 0.418 | 3/5 | town cells 438 for 328 (starred) |
| `towns.d_old_growth` 12 | | 0.347 | 0.421 | 3/5 | town cells 210 (starred) |
| `migration_rate` 0 (0.05) | how fast people move | 0.366 | 0.409 | 0/8 | |
| `migration_rate` 0.15 | | 0.349 | 0.417 | 3/5 | |
| `people.found_saving` 0.04 (0.08) | too few towns | 0.334 | 0.418 | 2/6 | 23.8 towns (starred) |
| `people.found_saving` 0.027 | | 0.341 | 0.413 | 1/7 | 28.1 towns (starred) |
| `freight` 0.0075 (0.015) | carriage | 0.381 | 0.420 | 7/1 | carters 0.039 for 0.064 (starred), 12.2 towns |
| `freight` 0.03 | | 0.325 | 0.405 | 0/8 | carters 0.093 (starred), 23.1 towns |
| `engel.strength` 0 (3) | food | 0.254 | 0.278 | 0/8, starred | farm land 1,687 cells for 1,443 (starred), real wage 2.30 for 2.60 |
| `engel.strength` 6 | | 0.365 | 0.434 | 8/0 | real wage 2.77 |
| `people.growth_top` 0.02 (0.016), in the example | | 0.348 | 0.392 | | 1.17 million people for 0.74 |

What this says, suspect by suspect:

- **The gap in what a wage buys between town and country.** There is none to close. A town's district is one market for labour with one wage, paid alike by its farms, pits and works: a works hand earns 0.99 to 1.03 of a land hand's, by the books, at every mark. Town people pay a house rent (`urban_rent_rate`, 12 per cent of the wage at the centre, less outward) that country people do not, and that is all. Taking it away or doubling it moves the share by half a point to a point, and through the number of people, not through where they live, since nobody chooses where to live.
- **The ground a town can take.** It never binds: the towns' people stand at 0.73 to 0.94 of their ground's room, and when they outgrow it the town takes its best free neighbour. A third more ground or a third less leaves the share where it was.
- **Food and its carriage.** Food is the strongest lever in the table. As households grow richer the Engel curve moves their spending from bread to comfort, which the shops and trades (`services`, a works) supply: without it 0.28, at twice its strength 0.43. It already does most of what it can; the curve at strength 3 has spent seven eighths of its turn by twice the reference income, and the island is at four times it by 1900 (real income a head 8.8 against the reference 2, by the example; 1.6 in 1750 and 5.2 in 1850). Carriage counts only through the carters, a tenth of the hands early and one in sixteen late, whom the count leaves in the country. Halving the freight frees two and a half points of the labour force (carters 0.039 for 0.064 in 1900) and raises the share by nearly four points in 1850, but by 1900 the share is where it was (0.420 for 0.419, with one island at 0.343): the freed hands are not in the works by then. Doubling it takes a point and a half off, on all eight islands.
- **How fast people move.** Migration moves people between towns. The count is a sum over towns of a rule that is the same in every town, so moving people from one to another changes it only as far as they change the works' share of hands, which is a point.
- **Too few towns.** More towns bring more works (854 for 738 at `found_saving` 0.04, 905 at 0.027) but not a larger share of the hands in works, and at 28 towns the share is 0.413.

### The quarrymen, and the rule itself

Two things in the table of hands are odd for an island of 1900, and the count rests on both.

**One hand in seven is in a quarry or a clay pit.** Households buy their shelter as raw stone (`data/content.toml`: stone serves shelter at 0.5), so every house built or kept is quarrymen's work and no builder's, and the quarrymen live in the country by the rule. `README.md` and `docs/rules/mines.md` ("Why quarrying takes so many hands") already have this; here it is the largest single holder of the share. The production tree's review, on its own branch and not mine, ran the game's rules with and without builders on sixteen islands (`.claude/night/reports/sweep-treerev-m-f6.txt`): quarry hands in 1900 0.025 for 0.141, and the share of people in towns 0.293 for 0.217 in 1750, 0.419 for 0.348 in 1850 and 0.465 for 0.420 in 1900, higher on every island at every mark.

**Tried on main and not kept: stone that hardly serves for shelter.** With stone's use for shelter at 0.05 for 0.5 (`trial-stone.diff`, one number in `data/content.toml`), so that households build in brick and timber, eight islands: quarry hands in 1900 0.005 for 0.139 and clay pit hands 0.077 for 0.019 (both starred), carters 0.043 for 0.064 (stone is the heaviest thing carted, at 2.2 a unit), woodland 1,281 cells for 1,063; and the share living in towns 0.235, 0.279, 0.375 and 0.466 in 1750, 1800, 1850 and 1900, higher on every island in 1750 and 1900. That is the tree's 0.465 again by another road, so the quarrymen are a cause and not a correlation. It is not a mend: shelter is dearer in brick, baskets a head in 1900 are 7 per cent fewer (seven islands of eight) and there are 7.5 per cent fewer people.

**The count's own numbers were never set for the game.** 0.04 and 0.8 are the Python's. They say that a twentieth of the people live in towns whatever they do, and four fifths of the works' households. Under the port, with a quarter of the hands in works, that gave a quarter in towns; under the game's rules the works' share has doubled and the count has followed it, as it was made to.

**Tried and not kept: every works' household in town.** With 1.0 in place of 0.8 (`trial-count.diff`), eight islands: the share is 0.263, 0.318, 0.424 and 0.513 in 1750, 1800, 1850 and 1900, higher on every island at every mark. But the count is not only a count. The people rule's crowding deaths read it (the share of a district's people who are townsfolk, times how great the town is, `peopling::crowding_term`), and the island ends with 666,000 people for 732,000, fewer on all eight. Works are 691 for 738 and the real wage 2.67 for 2.60, inside the spread. So changing the definition is not free: `people.town_deaths` (0.012) was set beside the old count and would want setting again with a new one.

## (b) Why founding stops

### The test, site by site

`peopling::found_towns` looks every year at the six spots where the carting bill is heaviest, at least 8 cells from every town, and founds a town on the best that passes five tests: the district's carting saved in a year is at least `found_saving` (0.08) of its wages; it has at least 40 cells and 3,000 people; no town gives up more than a fifth of its people; and no town giving people has given in the last eight years. The best site that passes is then tried for real, with the market areas worked out again, and stands only if its real district passes the same tests (at most two tries a year); the example does not make that second trial. The example asks the same function (`peopling::candidates`) about the same sites on a copy of the island just before each year's step, at the year's mean wage, and counts which tests each site passes. By decade, seeds 1 to 8 (`base.txt`, part (b)):

| | towns founded on an island | the island's money wage over 1750's | the best site's saving over wages | the same at 1750's money wage | sites passing the saving test | passing cells, people, donor |
|---|---|---|---|---|---|---|
| 1750s | 2.1 | 1.00 | 0.205 | 0.203 | 0.91 | 1.00, 0.54, 0.81 |
| 1780s | 0.9 | 1.41 | 0.161 | 0.228 | 0.55 | 0.96, 0.58, 0.74 |
| 1800s | 0.5 | 1.49 | 0.128 | 0.194 | 0.41 | 0.94, 0.65, 0.69 |
| 1820s | 1.4 | 1.61 | 0.131 | 0.210 | 0.31 | 0.90, 0.70, 0.65 |
| 1840s | 0.5 | 1.81 | 0.073 | 0.134 | 0.08 | 0.81, 0.74, 0.62 |
| 1860s | 0.4 | 2.17 | 0.061 | 0.129 | 0.04 | 0.75, 0.74, 0.65 |
| 1880s | 0 | 2.70 | 0.051 | 0.138 | 0.01 | 0.78, 0.76, 0.65 |
| 1890s | 0 | 3.01 | 0.047 | 0.140 | 0 | 0.81, 0.81, 0.62 |

(The best saving is the median over the decade's island-years. Seeds 9 to 16, `base-s9.txt`: 0.230, 0.135, 0.141, 0.102, 0.062, 0.060, 0.054 as is; 0.224, 0.187, 0.217, 0.158, 0.121, 0.146, 0.156 at 1750's money wage; last founding 1845.)

The other tests change little over the century, or grow easier: from the 1840s three sites in four have the cells and the people, and three in five would take no more than a fifth of a town, while the rest owed to towns that gave people lately, which three sites in ten passed in the 1750s, is passed by nearly all from the 1850s. There is always room: some 2,650 cells or more stand 8 cells from every town in every decade, and six sites are looked at every year. What changes is the saving, and almost all of its fall is the denominator. `saved = freight x tonnage x path cost saved`, where `freight` is 0.015 of money a unit of weight and path cost, fixed; `wages = 12 x hands x the town's mean wage`, in money that grows. By the 1880s the same tonnage carried the same way saves a third of the share of wages it did in 1750. Most of the rise in the money wage is real (what a wage buys is 2.6 times 1750's in 1900), so this is not a matter of money growing: people grow richer and the cart's hire, fixed in money, does not. Measured against wages at 1750's money wage, the best site saves about 0.2 of them until the 1820s, falls once to about 0.13 in the 1830s and 1840s, as the railways come, and stays there to 1900: well over the 0.08 the rule asks. But that is the best of the six sites, and the best mostly fails another test. The sites that fail on the saving alone (they pass the cells, the people, the donor and the rest) offer less: at 1750's money wage, 0.07 to 0.09 of their wages from the 1780s to the 1820s, 0.05 in the 1840s and 1850s, and 0.07 to 0.08 from the 1860s, right at the threshold, against 0.03 as they are. In about half of the island-years from the 1870s one of them would pass at 1750's money wage, against none as they are (`check/base-new.txt`, the example's lines on sites that fail on saving alone). So with the test in 1750's money founding would go on, but only just, and that is why the trial below still stops, in the late 1870s, once it has used the best of those sites.

This is the same thing `docs/rules/carting.md` left for a decision (R14, C91): carting is priced in money while the money wage rises, so a cartload takes two fifths fewer hands by 1900 with no better road. The founding test inherits it.

### Switched

- `freight` doubled: 23.1 towns in 1900, the last founded in 1880 on average, later on all eight islands; halved: 12.2 towns, the last in 1815 (sweeps/freight.txt).
- The rival, the donor test (no town to give up more than a fifth of its people), which on seeds 17 and 18 only a fifth to a half of the sites pass for most of the century: `people.donor_max` 0.35 and 0.5 for 0.2 give more towns early (18.4 and 18.9 in 1850 for 16.9) and the last founded earlier, in 1839 and 1838, on six and seven islands of eight (`check/donor.txt`). Loosening it spends the sites sooner; it does not keep founding going.
- **The mend tried**: the district's wages counted at 1750's money wage, so that the test compares the carting saved with what the same work would have cost in 1750 (`peopling::candidates`, one factor: January 1750's island wage from the observer's `wage` series over this year's mean wage by people; `trial-deflate.diff`, not committed). Last town 1878 for 1847 (later on all eight islands), 24.0 towns in 1900 (starred), 9.9 founded from 1800 for 5.1 (starred).

### If founding had gone on

| | main | `found_saving` 0.04 | `found_saving` 0.027 | test at 1750's wage (trial) | `freight` doubled |
|---|---|---|---|---|---|
| towns in 1850 and 1900 | 16.9, 17.8 | 22.1, 23.8 | 23.0, 28.1 | 20.8, 24.0 | 20.5, 23.1 |
| the last founded | 1847 [1814 .. 1873] | 1868 | 1882 | 1878 | 1880 |
| founded from 1800 | 5.1 | 9.1 | 13.0 | 9.9 | 9.6 |
| people in the middle one of those in 1900 | 20,318 | 14,808 | 13,438 | 16,712 | 16,572 |
| share living in towns, 1900 | 0.419 | 0.418 | 0.413 | 0.413 | 0.405 |
| works in 1900 | 738 | 854 | 905 | 852 | 823 |
| railway in 1900, km | 2,248 | 2,371 | 2,482 | 2,455 | 2,545 |
| people in 1900 | 731,872 | +1.8 per cent | +2.5 per cent | +2.2 per cent | -1.9 per cent |
| baskets a head in 1900 | 8.76 | 8.54 | 8.53 | 8.67 | 8.29 |
| real wage in 1900, 1750 = 1 | 2.60 | 2.55 | 2.53 | 2.62 | 2.74 |

An island that went on founding would have half as many towns again, smaller ones, more works (each town is a market with its own year's room for new capacity in every trade, `works.md`), a tenth more railway (each town is a station to be joined), a few per cent more people, and households a little poorer or no richer. The share living in towns does not rise; it falls a little, because each new market comes with its own country.

## (c) What limits a town in its fastest decade

For each town on each island the example takes the decade of its fastest growth in people (from its tenth year, with at least 3,000 people at the start) and splits each year's change into births less deaths (`pp_growth`), migration (the year's flows in and out, `migration_flow`), and the rest, which is people who came or went with cells that changed market (`peopling::move_with_cells`). Beside it, how often the limit on arrivals bound (in-migrants at `arrivals_max` of the town's people), how often the income term stood within a twentieth of `growth_top`, and town people over the room of the town's ground. Means over the 141 towns of seeds 1 to 8, and the same with one number changed (`--part c --set KEY=V`):

| | growth a year (median) | births less deaths | migration | with cells | years at the arrivals limit | income term near its ceiling |
|---|---|---|---|---|---|---|
| main, seeds 1 to 8 | 0.0434 (0.0384) | 0.0131 | 0.0146 | 0.0222 | 0.04 | 0.28 |
| main, seeds 9 to 16 | 0.0393 (0.0366) | 0.0128 | 0.0114 | 0.0199 | 0.04 | 0.28 |
| `people.arrivals_max` 0.3 (0.1) | 0.0426 (0.0402) | 0.0132 | 0.0144 | 0.0208 | 0.01 | 0.27 |
| `towns.d_old_growth` 3 (6): less room | 0.0459 (0.0405) | 0.0130 | 0.0134 | 0.0272 | 0.05 | 0.20 |
| `people.growth_top` 0.02 (0.016) | 0.0474 (0.0433) | 0.0167 | 0.0197 | 0.0139 | 0.06 | 0.36 |
| `migration_rate` 0.15 (0.05) | 0.0484 (0.0416) | 0.0130 | 0.0235 | 0.0199 | 0.22 | 0.30 |

And over every town-year from a town's tenth year, 1850 to 1900: at the arrivals limit 0.021 of them, the income term near its ceiling 0.894, over the room of its ground none (seeds 9 to 16: 0.024, 0.919, none). The fastest-growing towns have an income a head 1.01 times the island's and a wage 0.92 of it.

- **Housing does not limit a town.** No town-year on sixteen islands (from a town's tenth year) had more town people than the room of its ground, as `towns.md` found on eighteen, and a town given less room in each cell grows as fast: it takes ground faster (4.6 per cent a year in its fastest decade for 4.3). The rule has no way for room to hold people back; it only decides how many cells they cover.
- **The cap on arrivals hardly binds**, one year in twenty-five of the fastest decades, and loosening it threefold leaves them as they were.
- **The ceiling on births less deaths binds, for everybody at once.** From the 1850s nine town-years in ten have incomes high enough to sit within a twentieth of `growth_top`, which is what the sweep's `growth_capped_people` (0.31 over the whole run) and `town_growth_pct_p90` (1.50 per cent) say. Raised to 0.02, every town's births add 0.36 of a point and the fastest decades grow by as much, but so does the island (1.17 million people in 1900), which is poorer and less urban for it (0.392). It is a ceiling all towns share, so it sets how fast a fast town grows at the least, and not why one town outgrows another.
- **What sets a town apart is its pull on migrants, and for a few towns a jump in market area.** The means put close to half the growth of the fastest decades in cells changing market (2.2 points of the 5.0 that the three parts add to; the decade's compound rate, 4.3, is less than the mean of its years because the cells come in jumps): a road, canal or railway that brings a district nearer to this town than to its old market, or a neighbour's district redrawn, takes its households with it. But that mean is carried by a few towns. The top of the list is of this kind (Stoneford on seed 8 from 1869, 16.6 per cent a year, with cells bringing 27.6 points a year and migrants 1.0; Uldale on seed 1 from 1847, 30.8 points with cells), while the median town's fastest decade has 0.2 points a year with cells, 1.5 of migrants and 1.3 of births less deaths. Migration is the largest part in 72 of the 141 decades, cells in 41 and births less deaths in 28; before 1850 cells lead (2.6 points a year for 1.1 of migrants, 104 decades), from 1850 migrants do (2.4 for 1.1, 37 decades). Seeds 17 and 18 show the same (migration first in 16 of 34 decades, cells in 14). Migrants follow income a head (`people::migration`: each year `migration_rate` of a town's people, times what the richer towns offer them, move toward those towns, each weighed by how much richer it is and how cheap the journey; at most `migration_rate` of them). The fastest-growing towns are no richer than the island as a whole (income a head 1.01 of the island's, the wage 0.92); what draws people is being richer than the towns within reach. With people three times as ready to move, the fastest decades reach 4.8 per cent a year, migration is 2.4 points of it, and the arrivals limit binds in a fifth of their years.

So the answer to "the cap on growth, housing, or work" is: not housing, hardly the cap on arrivals, the ceiling on births for every town alike, and, for what sets one town above another, the pull of its income a head on the towns round it, through `migration_rate`; for a few towns, mostly before 1850, a road or railway that hands them a neighbour's district.

## How sure I am, and what would change my mind

- **(a) is certain as far as the count goes**: it is a line of code, and the books show it holding to within a thousandth on sixteen islands. What is a judgement is which of the other hands "should" be in works. The quarrymen are the clearest case and the tree's sweep is sixteen islands of sixteen, but it is another branch with a dozen other changes, and its higher share comes partly from 7 per cent fewer people. The trial on main that took stone out of shelter by one number gives the same 0.466 with the same 7 per cent fewer people, so I am sure of the direction and the size; what a designer would actually build (builders, as the tree has them) is the tree's to show.
- **The switch sweeps are eight islands each.** The effects that matter here are either large and on every island (Engel, the freight's carters, the towns' count) or nothing; the ones of a point (rent, migration) lean the same way on seven or eight islands of eight but are small enough that I would not build on their sign.
- **(b): the money wage is the cause.** Three different ways of making the saving larger relative to the wage (a lower threshold, dearer freight, the test in 1750's money) each keep founding going into the late 1860s to 1880s on average, and later than now on six to eight islands of eight, and the share of sites that pass every other test does not move over the century. What would change my mind: a run in which the saving at 1750's wage fell under 0.08 by itself. For the best site it does not on any of sixteen islands; for the sites that pass every other test it sits at about 0.08 from the 1860s, so a few per cent either way in the deflator or in tonnage decides whether founding at 1750's money goes on to 1900 or stops in the 1870s, and the date the trial gives (1878) should be read as "the late 1870s or after", not as a figure. The rival gate, the donor test, was loosened and founding stopped sooner, not later. One caution: the example asks the founding rule a moment before the year's step does (before the year's births and migration, at the year's mean wage), so its site lists are the step's to within a year's growth; the foundings it reports are the run's own.
- **(c): the mean and the median tell different stories.** The mean of each part over the fastest decades puts close to half in cells changing market; the median puts almost none there, because a few towns gain 20 to 30 points a year in a jump. Read from the mean alone, the pull of migrants looks weak; the medians and the count of which part leads each decade say the opposite, on seeds 1 to 8 and on seeds 17 and 18.
- **(c): the split of growth is by subtraction.** "With cells" is what is left after births less deaths and the migration flows, so anything else that moves people in the step lands there. Nothing else does under the game's rules (emigration is inside births less deaths), and the towns whose decades are mostly "cells" are those whose market area grew. What it does not show is why a market area grows; that wants the year's catchments, which the example does not read.
- **A single year of the share misleads less than the wage does**, but the twenty-year means are given above, and they agree with the marks.

## What it means for the pass over all the rules

- **The target of a half is a target on the works' share of hands, through a rule nobody has set for the game.** Before any number is searched for it, the designer should decide what "living in towns" is to mean. Three ways to reach a half: (1) put the quarrymen where they belong, which the production tree does (0.465 on its own branch; stone taken out of shelter on main, 0.466); (2) change the count, `0.04 + 0.8 x` in `towns::update_urban`, which is the port's and was never set for the game (with 1.0 for 0.8 it is 0.51, but the crowding deaths that read the count take 9 per cent of the people, so `people.town_deaths` goes with it); (3) count the carters, who live where the carts are loaded, as town people (0.475 by arithmetic). Nothing else in the question's list moves it by more than a point, except the Engel curve, which is already near the end of its turn.
- **`people.found_saving` is weighed in money against a freight fixed in money**, so it stops founding when wages rise, whatever the roads do. Whoever settles piece 15 (carting that costs, R14 and C91) settles this with it: if freight follows the wage, or the saving is counted in hands, founding goes on into the 1870s at 0.08. Then `found_saving` wants setting again for the number of towns wanted: at 0.08 measured in 1750's money there are 24 towns in 1900 on eight islands.
- **More towns are not more town people.** If the map should have more towns, that is for the map's sake; it costs a little of the wage (baskets a head 1 to 3 per cent lower) and adds railway and works.
- **For a town's growth, the number is `migration_rate`** (0.05) and the income gap it multiplies; `arrivals_max` (0.1) rarely binds and could stay. `growth_top` is the people rule doing what its page says, and raising it is Malthus, not towns.

## How to run it again

```bash
cargo run --release -p isostate-core --example tr_urban -- --seeds 1-8                     # the three parts, as on main
cargo run --release -p isostate-core --example tr_urban -- --seeds 9-16
cargo run --release -p isostate-core --example tr_urban -- --seeds 17,18                   # the check's two islands
cargo run --release -p isostate-core --example tr_urban -- --seeds 1-8 --part c --set people.arrivals_max=0.3
cargo run --release -p isostate-core --example tr_urban -- --seeds 1-8 --part c --set migration_rate=0.15
cargo run --release -p isostate-core --example tr_urban -- --seeds 1-8 --part c --set towns.d_old_growth=3
cargo run --release -p isostate-core --example tr_urban -- --seeds 1-8 --set people.growth_top=0.02
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules game --families 6 --set urban_rent_rate=0,0.24 --out DIR/rent --compare MAIN
```

and the same sweep with `--set towns.d_old_growth=3,12`, `migration_rate=0,0.15`, `people.found_saving=0.04,0.027`, `freight=0.0075,0.03`, `engel.strength=0,6` and (the check's) `people.donor_max=0.35,0.5`, where `MAIN` is a sweep of main with no `--set`. The three trials are the diffs beside the sweeps, each applied to a6ff0ec and swept with no `--set`. The example plays each island by the month and copies it once a year to ask the founding rule, so it is slower than a sweep; on a machine shared with a dozen other jobs it took some minutes for eight islands.
