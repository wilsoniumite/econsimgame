# People

A population that answers to how people live, towns founded all century, and
households that go with their cells when a market's area changes. The code is
`crates/isostate-core/src/peopling.rs`; the switch is `people.enabled`, off in
the port and on in the game. None of it is in the Python.

## Why

Under the port the island's people are a clock. A town grows by
`growth_slope * ln(income) + growth_base`, held at `growth_cap`, and income has
been above the level where the cap binds since the 1770s, so more than 99 town-
years in 100 grow at exactly 1.1 per cent and every seed ends within one per
cent of 682,000 people. A good decade and a bad one, a rich town and a poor
one, leave the same number of children.

The port also founds its towns in a rush. Seven or more appear in the 1750s,
each taking up to half of a neighbour's people on the day, and after about
1800 none is founded at all: a site has to be more than `found_distance` in
path cost from its market, and once the roads are made no cell is. The
coalfields and the railway never get a town of their own. And when a town is
founded its people are moved once, by a share with a floor of two per cent,
and never again, so a town whose market area later shrinks to a few cells
keeps people with no work near them. The growth probe met the end of that
road: a town with four cells and 363 people, migrants pouring in, the wage
running from 0.02 to 500 until the money check failed.

## What it does

Four things, each in place of one of the port's.

### Births and deaths

Each town's people change once a year by

```text
rate = growth_top * (1 - (y_still / y)^ease)        income
     - town_deaths * (townsfolk / people) * townsfolk / (townsfolk + town_size)   the town itself
     - dearth * depth                                a hungry year
     - emigration * depth                            those who sail, if a port can be reached
```

where `y` is the year's income per head over the cost of subsistence (the same
number the port's rule reads), `townsfolk` are the people living on town
ground, and `depth` runs from 0 when the worst month fed everyone
(`dearth_from`) to 1 when it fed six in ten (`dearth_full`).

There are no separate purses for the born and the dead; the rate is births
less deaths. What each part stands for:

- **Income.** At `y_still`, one and a half times bare subsistence, a household
  just replaces itself. Above it more children live and people marry younger,
  but only so far: the rate climbs toward `growth_top` and never reaches it,
  because there is a limit to how fast a people without modern medicine can
  grow however well it eats. At twice subsistence the income part is 0.75 per
  cent a year, at three times 1.27, at six times 1.56, at a hundred 1.65. So a
  town's fortunes show in its cradles, and a doubling of late-century incomes
  adds a third of a point, not a population bomb.
- **The town itself.** Crowded streets kill. The cost rises with the share of
  the district's people who live on town ground and with how great the town
  is: nothing in a village, four tenths of a point where half of 40,000
  people are townsfolk, never more than `town_deaths`.
- **Dearth.** A year whose worst month left people short takes up to three
  per cent of them. The port's famine news still fires at its own threshold.
- **Emigration.** In the same years up to two per cent take ship, from the
  nearest town on the coast that can be reached. They leave the count for
  good. Their share of the households' cash does not leave the island: it is
  paid to that port's merchants as the passage (`hh_cash` of the town they
  leave to `merch_cash` of the port), so `econ::money_total` is unchanged.
  When a thousandth of the island goes in one year the news says so.

Births less deaths never fall below `growth_floor`.

### Migration between towns

The port's rule stays: people move toward towns with higher income per head,
less willingly the dearer the journey, and the journey is priced along the
same paths as freight, so a railway between two towns already brings them
closer for migrants. One limit is added. A town takes in at most
`arrivals_max` of its own people in a year, and the flows into it are scaled
down together to fit. A small place with one good year can no longer double.

### New towns

Once a year, after the year's roads are known, the rule asks where a market
is most missed. Every cell pays to cart its year's tonnage to the town it
trades through. For the half dozen spots where that bill is heaviest, at
least `found_spacing` cells from every town, it works out the district a
market there would serve (every cell nearer to it than to its present
market), what the district's people would save in carting in a year, and
what they earn. A town is founded on the site that saves the most if

- the saving is at least `found_saving` of the district's wages,
- the district holds at least `found_cells` cells and `found_people` people,
- no existing town would give up more than `donor_max` of its people, and
- none of the towns giving people has given to a founding, or been founded
  itself, in the last `donor_rest` years.

The estimate is made on last year's paths, so before anything is settled the
site is tried for real: the town's node is switched on, the catchments are
worked out exactly as the yearly step will work them out, and the district is
counted again. Only if it still passes does the town stand. No town can be
founded on a handful of cells, because the cells are counted first.

At most one town is founded a year, and a town that has just given a sixth of
its people is left alone for eight years. On seed 7 the 1750s see two
foundings where the port has eight or nine.

Nothing in the rule names a coalfield or a railway. They show up because
that is where tonnage is: a pit ten cells from its market pays more in
carting than a parish of farms does. Each founding is recorded with what its
district was known for (the trade that takes a larger share of its hands
than it does on the island: coal, iron, stone, works, or farm where none
stands out), whether the site is on the coast, and whether a railway ran
within a cell of it on the day, and the news tells it that way: a colliery
town, a quarry town, a town of works, a harbour town, a market town. Town
names come from a random stream of their own (stream 8), so nothing the port
draws is disturbed.

### People go with their cells

Every year, once the catchments are worked out again, the households of every
cell that now trades through another town go to that town, with their cash
and their landlords' cash. A town's people are taken to be spread over its
cells as its labour at work is, so a town keeps the people whose cells it
keeps. A town founded this year gets its people the same way, and with them
the same share of the old market's stocks and its merchants' cash. Every one
of these is a transfer from one town's purse to another's.

This is what makes a small market area harmless. A town left with twenty
cells has twenty cells' worth of people, not the crowd it had when it was
large, and the arrivals limit keeps newcomers from pouring into it.

## The numbers

| name | value | what it is |
|---|---|---|
| `enabled` | off; on in the game | the switch |
| `growth_top` | 0.0165 | births less deaths a year that income alone can approach |
| `y_still` | 1.5 | income over subsistence at which births balance deaths |
| `ease` | 2.1 | how quickly the rate nears its ceiling |
| `town_deaths` | 0.012 | extra deaths a year per head in a great town where all live on town ground |
| `town_size` | 10,000 | townsfolk at which a town is half way to being a great one |
| `dearth_from`, `dearth_full` | 1.0, 0.6 | the worst month's food over need at which dearth begins and is complete |
| `dearth` | 0.03 | deaths a year per head in a complete dearth |
| `emigration` | 0.02 | share who leave the island in a complete dearth |
| `growth_floor` | -0.06 | births less deaths never fall below this |
| `arrivals_max` | 0.1 | migrants a town can take in a year, as a share of its people |
| `found_saving` | 0.05 | carting saved in a year over the district's yearly wages |
| `found_cells`, `found_people` | 40, 3,000 | the least district that can keep a market |
| `found_spacing` | 8 | cells between a new town and any other |
| `donor_max` | 0.2 | the most of its people a town gives to one founding |
| `donor_rest` | 8 | years a giving town, or a new one, is left alone |

`growth_top` was set so that the island grows about fivefold on the port's
income path: on seeds 1 to 8 (with `found_saving` at 0.05), 0.016 gave
636,000 people in 1900, 0.0165 gave 667,000 and 0.017 gave 703,000.
`found_saving` decides how many towns there
are, and is the one to turn if you want more or fewer (see the sweep below).
The rest are first values that gave sensible histories and have not been
searched.

## Where it touches the rest

- `people::growth` and `people::migration` each call in with one line when
  the switch is on.
- `sim::yearly` skips the port's `people::found_towns`, calls
  `peopling::found_towns` after `update_roads`, and calls
  `peopling::move_with_cells` every year where the port calls
  `people::resettle_new_towns` in founding years only.
- The state's new fields (`pp_*`, at the end of `State`) stay empty while the
  switch is off.
- The sweep gains `town_growth_pct_p10`, `_p50` and `_p90` (every town-year's
  rate of increase, in per cent; under the port, its clamped rule),
  `town_growth_near_old_cap` (the share of town-years within a tenth of a
  point of the port's 1.1 per cent), and with the switch on `late_towns` (the
  towns founded from 1800), `late_towns_coal`, `_iron`, `_stone`, `_works`,
  `_farm`, `late_towns_port`, `late_towns_rail`, `late_town_people_median`,
  `late_town_people_largest` and `emigrants`. With the switch on,
  `growth_capped_*` count a town as held when income has carried it to within
  a twentieth of `growth_top`, which takes an income above six times
  subsistence.
- `cargo run --release -p isostate-core --example people -- 7 --families 6`
  prints one run by decade, every founding with its reason, and one town's
  people (`--town`, `--off` for the port's rules, `--candidates` for the sites
  the rule looked at).

## What a sweep showed

Seeds 1 to 16 under the port's rules with `--set people.enabled=true`, set
seed for seed against the port alone, once without families and once with
six. No run failed in either. Each number is the mean across seeds, with the
least and greatest seed in brackets.

| | no families: port | with people | six families: port | with people |
|---|---|---|---|---|
| people in 1800 | 229,505 [226,626 .. 231,445] | 198,575 [179,858 .. 214,633] | 229,527 [226,666 .. 231,185] | 200,356 [185,509 .. 216,861] |
| people in 1850 | 396,144 [390,493 .. 399,757] | 360,204 [324,354 .. 383,617] | 396,041 [388,872 .. 398,733] | 363,717 [323,604 .. 384,563] |
| people in 1900 | 682,805 [672,074 .. 690,802] | 660,286 [595,251 .. 690,206] | 684,367 [671,992 .. 689,032] | 669,539 [556,814 .. 706,479] |
| a town-year's rate, per cent: tenth, middle, ninetieth | 1.10, 1.10, 1.10 | 0.74, 1.23, 1.33 | 1.10, 1.10, 1.10 | 0.79, 1.23, 1.33 |
| town-years within 0.1 of the port's 1.1 per cent | 0.986 | 0.252 [0.088 .. 0.534] | 0.987 | 0.224 [0.070 .. 0.362] |
| towns in 1800 | 18.0 [16 .. 22] | 11.2 [8 .. 14] | 18.6 [16 .. 22] | 11.7 [8 .. 16] |
| towns in 1850 | 18.6 [17 .. 22] | 15.4 [10 .. 22] | 19.2 [16 .. 22] | 16.2 [8 .. 23] |
| towns in 1900 | 18.8 [17 .. 22] | 17.3 [13 .. 23] | 19.3 [16 .. 22] | 19.1 [11 .. 24] |
| last town founded | 1798 [1766 .. 1851] | 1870 [1848 .. 1899] | 1802 [1775 .. 1873] | 1871 [1851 .. 1897] |
| real wage in 1800, 1750 = 1 | 1.34 | 1.40 | 1.31 | 1.42 |
| real wage in 1850 | 1.30 | 1.30 | 1.36 | 1.31 |
| real wage in 1900 | 1.31 [0.93 .. 1.48] | 1.21 [0.89 .. 1.53] | 1.36 [1.22 .. 1.49] | 1.29 [1.05 .. 1.48] |
| output per head in 1800 | 1.41 | 1.45 | 1.40 | 1.49 |
| output per head in 1850 | 1.48 | 1.49 | 1.55 | 1.56 |
| output per head in 1900 | 1.47 [1.04 .. 1.64] | 1.42 [1.11 .. 1.63] | 1.51 [1.38 .. 1.61] | 1.45 [1.17 .. 1.64] |
| railway in 1900, km | 1,018 | 1,071 | 1,065 | 1,182 |

What that says:

- **The population is no longer a clock.** The island still grows about
  fivefold (4.9 times on average), but a seed now ends anywhere from 4.1 to
  5.2 times its start where the port's seeds all ended within 1.4 per cent of
  each other, and a quarter of town-years sit near the old 1.1 per cent where
  99 in 100 did. Growth is slow while the island is poor (on seed 7 with six
  families about half a per cent a year in the 1750s, 0.7 to 0.9 in the next
  two decades) and about 1.2 per cent from the 1780s, so there are 30,000
  fewer people in 1800 in every seed and 2 or 3 per cent fewer than the port
  by 1900.
- **Towns are founded all century, and no more of them.** Eleven or twelve
  stand in 1800 where the port has eighteen, and by 1900 there are as many as
  the port ends with; the last is founded around 1870 where the port stops
  around 1800. Six or seven a seed are founded from 1800 on (2 to 11).
- **Where the late towns stand.** Without families, of 6.1 late towns a seed
  2.6 are in quarry and claypit districts, 1.8 among works, 1.6 in farming
  country and 0.2 on a coalfield; 0.9 are on the coast. With six families:
  2.3 quarry, 2.9 works, 1.7 farm, 0.4 coal, 0.9 on the coast. No iron
  district got a town. The coalfields mostly get theirs before 1800 (on seed
  7: 1758, 1787, 1790, and two more in 1816 and 1833). Almost none is founded
  beside a railway (one town in the 32 runs): a district on the line already
  carts cheaply, so it is not where a market is missed. The railway comes to
  the new towns afterwards instead. The median late town has 34,000 people in
  1900 without families (18,000 to 56,000 across seeds) and 29,000 with; the
  largest in a seed averages 52,000.
- **The home town keeps its people.** Seed 7 with six families: Melbourne has
  26,333 people at the end of 1750, 20,709 in 1760 and 20,456 in 1780. Under
  the port it has 21,570, 13,508 and 14,243. It still gives about a fifth of
  its people to new towns in the 1750s, and then holds.
- **Wages: higher in 1800, lower in 1900.** With 30,000 fewer people the real
  wage and output per head of 1800 are higher in 14 or 15 seeds of 16, by 3
  to 8 per cent. In 1900 the real wage is lower in 11 and 12 seeds of 16, by
  0.10 and 0.07 on the index (8 and 5 per cent), and output per head by 0.05
  and 0.06 (9 and 12 seeds). The spread between seeds in 1900 is 0.9 to 1.5, so neither is
  clear of it, but the lean is the same in every sweep I ran and I take it as
  real: about 6 per cent of the 1900 wage. Faster growth after 1850 (1.25 per
  cent against 1.1) presses on the land when the port's economy has stopped
  growing, and the growth probe found the same price for late towns.
- **A little more railway, a little later.** The last year the network grows
  moves from 1867 to 1878 without families and from 1853 to 1875 with them
  (13 and 14 seeds of 16). There is 11 per cent more line in 1900 with
  families (13 seeds of 16) and 5 per cent without, which is inside the
  spread.
- **Emigration is small**: about 3,000 people a seed over the century and a
  half (0 to 8,600), in the hungry years.
- **Starred** (every seed above or below every seed of the port): the people
  of 1800 and 1850, the growth facts, and towns in 1800 without families.

`found_saving` and the number of towns, six families, seeds 1 to 16:

| `found_saving` | towns 1800, 1850, 1900 | founded from 1800 | last | real wage 1900 against the port's 1.36 | railway 1900 |
|---|---|---|---|---|---|
| 0.05 | 12.5, 18.4, 24.2 | 11.7 | 1882 | 1.21 (no seed higher) | 1,343 km |
| 0.08 (the game's) | 11.7, 16.2, 19.1 | 7.4 | 1871 | 1.29 (4 seeds higher, 12 lower) | 1,182 km |
| 0.12 | 10.5, 13.4, 14.6 | 4.1 | 1850 | 1.32 (4 higher, 12 lower) | 1,000 km |

At 0.05 there are five more towns than the port has, founded into the 1880s,
and the railway goes on being built into the 1890s (a quarter more line). I
settled on 0.08 because with six families the 1900 wage fell in every seed
at 0.05. Without families the same comparison went the other way (1.25 at
0.05 against 1.21 at 0.08), so how much each town costs is not something
sixteen seeds can tell; if more towns are wanted for the look of the map, 0.05
is a fair choice too.

The sweeps are in `.claude/night/reports/`: `sweep-people-base` and
`sweep-people` (no families), `sweep-people-base-f6` and `sweep-people-f6`,
and the 0.05 and 0.12 runs in `sweep-people-saving05`,
`sweep-people-f6-saving05` and `sweep-people-f6-fewtowns`.

The scripted players (`examples/bots.rs`, seeds 1, 2, 3 and 7, four
strategies each) all play a whole game under these rules without a failure.

## Tests

`tests/people.rs`: the rate's shape (zero at `y_still`, rising, flattening,
under its ceiling, a doubling of income worth less than a third of a point);
crowding and dearth; the probe's district of four cells and 363 people
refused; sixty years with six families run twice, with money conserved every
month, the two runs the same bit for bit, every founding on a district of
the size the rule asks for and the right distance from its neighbours; a
switched-off run that reads none of the numbers; and a town planted by hand
on a handful of cells hard by another, twice, watched to 1806: its wage stays
between a quarter and four times the island's (in the run, 0.59 to 1.19), and
while its area is no larger its people grow by no more than births and the
arrivals limit allow.

## What worries me

- **The 1900 wage.** About 6 per cent lower, as above, and I cannot say from
  these runs how much is the population's timing and how much is the towns.
  Other mechanisms are meant to lift late-century incomes; this one leans
  the other way, and it should be measured again in their company.
- **If incomes rise, so will the population.** That is the point of the
  rule, and it is bounded: at six times subsistence the income part is 1.56
  per cent a year against 1.27 at three. But fifty years at the higher rate
  is 15 per cent more people in 1900, and they press on the same land.
  `growth_top` was tuned on the port's incomes and will want turning down a
  little if the island gets much richer.
- **The crash was not reproduced, only fenced off.** The probe's failure was
  on another branch with another founding hack, on seed 10 in 1893. Here the
  rule cannot found such a town, and a town planted by hand on two cells
  stays sound (in the test the arrivals limit binds for three years running
  on the town left with 28 cells, at ten per cent a year). A town planted the same way under the
  port's rules sat with nobody in it, a wage of 0.02 and cash piling up per
  head, which is where the probe's crash began; that was seen by hand before
  the scratch hook was removed and is not a test.
- **Markets side by side are unstable.** The same test showed that a town
  planted beside a large one can take a thousand of its cells the next year,
  and 22,000 people with them, because every cell goes to whichever market
  is nearer by a hair. Spacing keeps the founding rule well away from this,
  and people going with their cells is the right bookkeeping when it
  happens, but a railway can still move a district from one market to
  another in a year, and a town's count of people will jump when it does
  (Melbourne loses and regains several thousand around 1810 and 1860 on
  seed 7).
- **Few colliery towns after 1800 and no railway towns.** The rule founds a
  town where carting is dear. Late in the century that is quarry country and
  outlying works, not the line. A rule for stations or junctions would have
  to stand on something else (what a market on the line is worth to
  travellers and shippers, say), and I did not want to invent one that the
  prices do not already speak for.
- **`found_saving` is a judgement.** See the table above.
- **Births and deaths are one number.** A game that wants to show a cholera
  year or a baby boom will want them apart. The state keeps only each town's
  rate and its income part (`pp_growth`, `pp_income_term`).
- **A new town starts as the port's does**, with its donor's prices, wage
  and knowledge, and nothing set for the fields the port also leaves alone
  (`pexp`, `urban_rent`). I kept to the port's way there.
- **Emigrants' cash goes to merchants.** It keeps the money on the island,
  which the conservation check needs, and a passage is a fair thing for it
  to buy, but it is a windfall to the port's merchants with no service made
  for it. It is small: a few thousand people a century.
- **The test hook.** `st.pp_found_next` orders a town at a cell whatever the
  rule thinks of it. It is there for the tiny-town test and for scenarios;
  nothing in a normal run sets it.
