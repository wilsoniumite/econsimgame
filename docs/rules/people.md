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

That crash was the far end of something the port does in most runs. When
cells change market, only the cells change. The new market's prices know
nothing of them, and for a new town the prices its producers expect are still
those the empty slot was given in 1750. So a pit that hardly paid at its old
market may be offered a buyer's price at the pit mouth, want ten times its
hands, and bid the town's wage up to double every month. With six families
the port has a town-month with a wage over four times the middle town's in
14 seeds of 16 (the worst in each run 6 to 111 times), and almost every town
it founds has a month of near-starvation in its first year, because its
households' incomes were left behind and its market has to run out of grain
before any is brought in.

## What it does

Five things, the first four each in place of one of the port's.

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

Births less deaths never fall below `growth_floor`. And whatever the numbers
are set to, deaths and sailings together never take more than half a town in
a year: with `dearth` and `emigration` both at 1 a town's people went below
nothing and the money check failed, and a town with nobody in it is not
something the markets can price.

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

### What a district is used to goes with it

People and cash are not all that a district brings to a market. Three more
things go with it, so that the day after the move nobody is paid or charged
a price that no trade has made:

- **Last month's income.** Households spend out of what they earned the
  month before. It goes with them (and the landlords' with theirs), or a new
  town buys next to no bread in its first month.
- **The wage.** A town's wage starts the year as the average of what its old
  hands and its newcomers were paid, by head. A new town has no old hands
  and starts at its newcomers' wage.
- **Prices.** Each market's price for each good, and the price expected
  there, becomes an average of three, weighted by a month's trade: its own
  price, by its own turnover (a new market has none); for what the arriving
  land yields, the price at which its producers are no better or worse off
  than they were (their old market's price less what they now save in
  carting), by the produce they bring; and for what the arriving people buy,
  the price at which their old market can still supply them (its price plus
  the cart from there and the merchant's handling), by what they used to ask
  for. So a district that sells coal opens its market near the price its
  pits were already getting, a district that must buy its grain opens near
  the price grain can be fetched at, and a field that goes over to a large
  market hardly moves that market's price at all. After that the market's
  own rule moves prices as it always does.

A new town also takes its donor's expected prices on the day it is founded,
which the port leaves as they stood in 1750.

None of this moves money. It sets where prices and the wage start; where
they go is the market's business.

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
| `found_saving` | 0.08 | carting saved in a year over the district's yearly wages |
| `found_cells`, `found_people` | 40, 3,000 | the least district that can keep a market |
| `found_spacing` | 8 | cells between a new town and any other |
| `donor_max` | 0.2 | the most of its people a town gives to one founding |
| `donor_rest` | 8 | years a giving town, or a new one, is left alone |

`growth_top` was set so that the island grows about fivefold on the port's
income path: on seeds 1 to 8 (with `found_saving` at 0.05, before the review's
repairs), 0.016 gave 636,000 people in 1900, 0.0165 gave 667,000 and 0.017
gave 703,000. `found_saving` decides how many towns there are, and is the one
to turn if you want more or fewer (see the sweep below). The rest are first
values that gave sensible histories and have not been searched. What goes
with a district has no numbers of its own: the weights are the trade itself.

At the edges that were tried (four seeds, six families): a town allowed
anywhere, of any size, every year (`found_saving`, `found_people`,
`found_spacing` and `donor_rest` at 0, `found_cells` 1, `donor_max` 1) fills
all 48 town slots by 1791 and ends with a real wage of 1.19, no run failing;
`growth_top` doubled to 0.033 gives 2.2 million people and a real wage of
0.63, which is Malthus and not a crash, no run failing; and the hardest
dearth the numbers allow (`dearth` and `emigration` 1, `growth_floor` -1)
leaves 330,000 people in 1900, no run failing. One edge does break:
`growth_top` 0.05 with `y_still` 0.5 and no limit on arrivals puts six to
eight million people on the island, and three runs of four then fail the
money check between 1887 and 1899, with single purses of thirty thousand
million and a total gone negative. I did not trace it. My guess is the
port's own issue of money, which is reckoned in the middle of a month from
a total that leaves out incomes on their way, and goes negative when a
month's incomes outgrow the stock; nothing near the game's numbers comes
within sight of it.

## Where it touches the rest

- `people::growth` and `people::migration` each call in with one line when
  the switch is on.
- `sim::yearly` skips the port's `people::found_towns`, calls
  `peopling::found_towns` after `update_roads`, notes what each cell paid to
  reach its market before the paths are worked out again, and calls
  `peopling::move_with_cells` every year where the port calls
  `people::resettle_new_towns` in founding years only.
- `econ::step_month` keeps `st.requests` (what each town's buyers asked for)
  when the switch is on, as it already does when there are families.
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
  people and cells (`--town`, `--every-year` for a line a year, `--off` for
  the port's rules, `--candidates` for the sites the rule looked at).

## What a sweep showed

Seeds 1 to 16 under the port's rules with `--set people.enabled=true`, set
seed for seed against the port alone, once without families and once with
six. No run failed in either. Each number is the mean across seeds, with the
least and greatest seed in brackets.

| | no families: port | with people | six families: port | with people |
|---|---|---|---|---|
| people in 1800 | 229,505 [226,626 .. 231,445] | 199,627 [185,903 .. 215,239] | 229,527 [226,666 .. 231,185] | 200,991 [186,827 .. 217,123] |
| people in 1850 | 396,144 [390,493 .. 399,757] | 363,499 [329,249 .. 386,995] | 396,041 [388,872 .. 398,733] | 364,942 [331,970 .. 390,485] |
| people in 1900 | 682,805 [672,074 .. 690,802] | 655,966 [529,736 .. 720,757] | 684,367 [671,992 .. 689,032] | 669,513 [620,202 .. 693,951] |
| a town-year's rate, per cent: tenth, middle, ninetieth | 1.10, 1.10, 1.10 | 0.73, 1.22, 1.32 | 1.10, 1.10, 1.10 | 0.77, 1.22, 1.32 |
| town-years within 0.1 of the port's 1.1 per cent | 0.986 | 0.247 [0.115 .. 0.523] | 0.987 | 0.263 [0.110 .. 0.517] |
| towns in 1800 | 18.0 [16 .. 22] | 11.3 [7 .. 14] | 18.6 [16 .. 22] | 11.4 [8 .. 14] |
| towns in 1850 | 18.6 [17 .. 22] | 14.7 [7 .. 20] | 19.2 [16 .. 22] | 16.3 [10 .. 21] |
| towns in 1900 | 18.8 [17 .. 22] | 17.1 [9 .. 22] | 19.3 [16 .. 22] | 18.4 [12 .. 22] |
| last town founded | 1798 [1766 .. 1851] | 1869 [1854 .. 1887] | 1802 [1775 .. 1873] | 1860 [1835 .. 1879] |
| towns founded from 1800 | | 5.8 [2 .. 9] | | 7.0 [4 .. 9] |
| emigrants, whole run | | 3,335 [0 .. 19,416] | | 1,217 [12 .. 4,995] |
| real wage in 1800, 1750 = 1 | 1.34 | 1.37 | 1.31 | 1.41 |
| real wage in 1850 | 1.30 | 1.28 | 1.36 | 1.35 |
| real wage in 1900 | 1.31 [0.93 .. 1.48] | 1.29 [1.10 .. 1.42] | 1.36 [1.22 .. 1.49] | 1.28 [0.84 .. 1.46] |
| real wage, mean of 1880 to 1900 | 1.32 [1.15 .. 1.45] | 1.31 [1.08 .. 1.56] | 1.37 [1.29 .. 1.44] | 1.33 [1.25 .. 1.46] |
| output per head in 1800 | 1.41 | 1.43 | 1.40 | 1.46 |
| output per head in 1850 | 1.48 | 1.46 | 1.55 | 1.58 |
| output per head in 1900 | 1.47 [1.04 .. 1.64] | 1.46 [1.25 .. 1.59] | 1.51 [1.38 .. 1.61] | 1.46 [0.93 .. 1.62] |
| railway in 1850, km | 797 | 770 | 1,048 | 830 |
| railway in 1900, km | 1,018 | 1,081 | 1,065 | 1,130 |

What that says:

- **The population is no longer a clock.** The island still grows about
  fivefold (4.9 times on average without families, 5.0 with), but a seed now
  ends anywhere from 3.9 to 5.3 times its start where the port's seeds all
  ended within 1.4 per cent of each other, and a quarter of town-years sit
  near the old 1.1 per cent where 99 in 100 did. Growth is slow while the
  island is poor (on seed 7 with six families 0.6 per cent a year in the
  1750s, 0.7 in the next two decades, 0.9 in the 1780s) and about 1.2 per
  cent from the 1790s, so there are 30,000 fewer people in 1800 in every seed
  and 2 to 4 per cent fewer than the port by 1900.
- **A hungry decade now shows in the count.** The low seed is seed 9 without
  families: the real wage is 0.95 in 1850, 18,000 people sail in the 1850s,
  the island has 10,000 fewer people in 1860 than in 1850, and the wage is
  1.7 by 1860. Under the port the same island's wage sags to 1.2 and 1.1 in
  those years and its people grow through it at 1.1 per cent.
- **Towns are founded all century, and no more of them.** Eleven stand in
  1800 where the port has eighteen, and by 1900 there are about as many as
  the port ends with; the last is founded in the 1860s where the port stops
  around 1800. Six or seven a seed are founded from 1800 on (2 to 9). The
  number varies more between islands than the port's does: one island without
  families ends with nine towns.
- **Where the late towns stand.** Without families, of 5.8 late towns a seed
  2.8 are in quarry and claypit districts, 1.7 among works, 1.1 in farming
  country and 0.2 on a coalfield; 0.8 are on the coast. With six families:
  3.0 quarry, 2.5 works, 1.1 farm, 0.4 coal, 0.8 on the coast. No iron
  district got a town. The coalfields mostly get theirs before 1800 (on seed
  7 with families: 1766, 1767 and 1783, and one more in 1828). Almost none is
  founded beside a railway (one town in the 32 runs): a district on the line
  already carts cheaply, so it is not where a market is missed. The railway
  comes to the new towns afterwards instead. The median late town has 33,000
  people in 1900 without families (17,000 to 53,000 across seeds) and 30,000
  with; the largest in a seed averages 55,000 and 50,000.
- **The home town keeps its people.** Seed 7 with six families: Melbourne has
  26,314 people at the end of 1750, 18,715 in 1760 and 19,306 in 1780. Under
  the port it has 21,570, 13,508 and 14,243. What it loses in the 1750s now
  goes by ordinary migration (it is the poorest of the six towns in 1750),
  two or three in a hundred a year, not to seven foundings. Later foundings
  take a district at a time: Coldbridge in 1833 takes 112 of its 726 cells
  and about 1,500 of its 27,800 people.
- **No wage runs away, and new towns eat.** In these 32 runs no town's wage
  is ever more than 1.9 times the middle town's or less than 0.28 of it, in
  any month. Under the port with six families the worst month in a run is 6
  to 111 times in 14 seeds of 16. A town under three years old has a hungry
  year (a month with less than 82 per cent of the food it needs) 0 to 2 times
  a seed, against 10 to 17 times under the port.
- **Wages: higher in 1800, a little lower at the end with families.** With
  30,000 fewer people the real wage and output per head of 1800 are higher
  in 12 to 14 seeds of 16, by 2 to 7 per cent. The single year 1900 is
  a noisy thing to compare (a seed's real wage moves by 0.2 from one year to
  the next late in the century), so the table also gives the mean of 1880 to
  1900: without families 1.31 against 1.32 (6 seeds higher, 10 lower), with
  six families 1.33 against 1.37 (5 higher, 11 lower). Output per head over
  the same years is 1.50 against 1.50 and 1.52 against 1.53. So with families
  the late wage leans about 3 per cent lower, inside the spread between
  seeds, and without them there is nothing to see.
- **The railway comes later with families, and goes on longer.** With six
  families there is a fifth less line in 1850 (830 km against 1,048; on seeds
  9 to 16 less in every seed, 589 against 1,039), because there are fewer
  towns to join and 8 per cent fewer people. By 1870 it has caught up, the
  last year the network grows moves from 1853 to 1869 (13 seeds of 16), and
  there is 6 per cent more line in 1900. Without families the line of 1850
  is the same and 1900 has 6 per cent more.
- **Emigration is small and comes in the hungry years**: 1,200 people a seed
  with families (12 to 5,000), 3,300 without (0 to 19,400, seed 9's 1850s).
- **Fewer works early.** There are 11 to 17 per cent fewer works in 1800 (14
  and 16 seeds of 16) and 4 to 7 per cent fewer in 1900, with 13 per cent
  fewer people in 1800 and a third fewer towns.
- **Starred** (every seed above or below every seed of the port): the people
  of 1800 (and of 1850 without families), the growth facts, the towns of 1800
  and, without families, the year of the last founding.
- **Seeds 9 to 16**, which nothing was tuned on, say the same as seeds 1 to 8
  for people, growth, wages and the last founding. They have fewer towns
  (15.1 in 1900 against 19.0 without families, 17.0 against 19.8 with) and
  they are where the railway of 1850 is short.

`found_saving` and the number of towns, six families, seeds 1 to 16 (run
before the last of the review's repairs, so the middle row differs a little
from the table above):

| `found_saving` | towns 1800, 1850, 1900 | founded from 1800 | last | real wage 1900 against the port's 1.36 | railway 1900 |
|---|---|---|---|---|---|
| 0.05 | 12.4, 18.8, 24.5 | 12.2 | 1886 | 1.22 (4 seeds higher, 12 lower) | 1,331 km |
| 0.065 | 12.1, 17.9, 21.7 | 9.6 | 1878 | 1.30 (5 higher, 11 lower) | 1,295 km |
| 0.08 (the game's) | 11.6, 15.9, 18.1 | 6.5 | 1868 | 1.31 (6 higher, 10 lower) | 1,167 km |

At 0.05 there are five more towns than the port has, founded into the 1880s,
a quarter more railway, and with six families a 1900 wage a tenth lower
(output per head 1.41 against 1.51, 3 seeds higher and 13 lower). Without
families the three settings cannot be told apart (1.29, 1.29 and 1.31
against the port's 1.31). The game keeps 0.08; if more towns are wanted for
the look of the map, 0.065 buys about three a seed for little.

The sweeps are in `.claude/night/reports/`: `sweep-people-review-base` and
`sweep-people-review` (no families), `sweep-people-review-base-f6` and
`sweep-people-review-f6`, the `found_saving` runs in
`sweep-people-review-saving` and `sweep-people-review-saving-f6`, and the
edges in `sweep-people-review-x-anytown`, `-x-double`, `-x-boom` and
`-x-dearth`. `cargo run --release -p isostate-core --example wage_watch`
counts the wages that run away and the hungry years.

The scripted players (`examples/bots.rs`, seeds 1, 2, 3 and 7, four
strategies each) all play a whole game under these rules without a failure.
Set beside the same games under the port (`--rules port`), in millions in
1900 on seeds 1, 2, 3 and 7:

| player | port | with people | place among the six, with people |
|---|---|---|---|
| passive | 24.1, 22.4, 17.4, 17.2 | 15.2, 8.1, 9.5, 9.2 | 4th in every game |
| founder | 24.6, 23.0, 16.7, 16.8 | 16.9, 6.6, 8.8, 10.4 | 4th in every game |
| landlord | 37.9, 28.3, 23.9, 24.5 | 38.5, 35.2, 29.3, 27.7 | 1st in every game |
| thoughtful | 35.0, 21.9, 21.7, 23.5 | 31.9, 28.0, 24.3, 29.4 | 1st in every game |

The two players who buy land or choose their trades do as well as under the
port or better. The two who keep the default trade end with half as much and
save 15 to 17 per cent of their income in the 1750s where they saved 24 to
30 (see the worries).

## Tests

`tests/people.rs`: the rate's shape (zero at `y_still`, rising, flattening,
under its ceiling, a doubling of income worth less than a third of a point);
crowding and dearth; the probe's district of four cells and 363 people
refused; sixty years with six families run twice, with money conserved every
month, the two runs the same bit for bit, every founding on a district of
the size the rule asks for and the right distance from its neighbours; a
switched-off run that reads none of the numbers; Kelrigg on seed 7, whose
market area trebles the year after it is founded and takes in a coalfield,
with every town's wage for the year within a factor of three of the middle
town's (15.6 times with the price rule taken out); and six towns planted by
hand on a handful of cells hard by another and watched to 1840. Two of the
six stay hamlets of 2 to 22 cells for the rest of the run. Their wage stays
between a twenty-fifth and four times the island's (in the run, 0.08 to
1.1), and while a planted town's area is no larger its people grow by no
more than births and the arrivals limit allow.

## What worries me

- **The default family is poorer.** The table of scripted players above: a
  family that keeps its first trade and takes every default ends fourth of
  six where under the port it ended first to fourth. On seed 7 its worth in
  1775 is 0.7 million with 15 per cent of income saved in the 1750s; under
  the port 1.1 million and 24 per cent; with the review's three repairs (the
  price rule, the wage, last month's income) all taken out 1.2 million and
  22 per cent, with any one taken out much as with all in; and under the
  first helper's version, which also left a new town's expected prices
  stale, 2.0 million and 30 per cent. So a family's first works were living
  in part on the booms that mis-set prices made at every founding, and they
  now meet dearer labour too (a real wage 7 per cent higher in 1800, 13 per
  cent fewer people). That is one seed and one strategy's worth of taking
  things out, not a diagnosis. Whoever balances the families should look at it
  before anything else here: which trade a new family is given, and what it
  earns in the 1750s.
- **The late wage with families.** About 3 per cent lower over 1880 to 1900,
  as above, inside the spread. More towns cost more (the `found_saving`
  table), which fits what the founding test leaves out (next item). Other
  mechanisms are meant to lift late-century incomes; this one leans a little
  the other way with families, and it should be measured again in their
  company.
- **The founding test counts the producers' saving, not the island's.** It
  adds up what the district's cells would save in carting to a nearer market.
  Coal that the district does not burn itself still has to get from the new
  market to the old one, and that leg is the merchants' to pay. So the test
  is a fair measure of where a market is missed and a poor measure of what
  the island gains, and a town of exporters gains least. A better test would
  net off the onward journey of what the district sells beyond itself.
- **If incomes rise, so will the population.** That is the point of the
  rule, and it is bounded: at six times subsistence the income part is 1.56
  per cent a year against 1.27 at three. But fifty years at the higher rate
  is 15 per cent more people in 1900, and they press on the same land.
  `growth_top` was tuned on the port's incomes and will want turning down a
  little if the island gets much richer.
- **What the probe's crash was.** It was not reproduced on its own branch.
  What was found instead is the thing it was the far end of: prices and
  expectations that did not go with cells (see Why). With that mended a town
  planted on two cells is poor and quiet. The port still does it, and any
  mechanism that moves catchments under the port's rules (a railway, a canal)
  will meet it there.
- **Prices are set by rule on one day a year.** When a district changes
  market the market's prices, expected prices and wage are set to averages,
  not found by trade. It is the same kind of thing as the port copying a
  donor's prices to a new town, done with more care, and it moves no money.
  But it is a rule, and it leans on the port's own carting and handling
  charges staying what they are in `econ::step_month`.
- **A new town still pays more for what it must buy in.** Its prices for
  those goods open at what it costs to fetch them from the old market, so
  its people are a little poorer than they were as that market's outlying
  customers. That is how the model's trade works (a household pays the town
  price and no carting), not something this mechanism adds, but it is where
  it shows.
- **Markets side by side are unstable.** A town planted beside a large one
  can take most of its cells the next year, because every cell goes to
  whichever market is nearer by a hair. Spacing keeps the founding rule well
  away from this, and people, prices and wages going with their cells is the
  right bookkeeping when it happens, but a railway can still move a district
  from one market to another in a year, and a town's count of people will
  jump when it does.
- **Few colliery towns after 1800 and no railway towns.** The rule founds a
  town where carting is dear. Late in the century that is quarry country and
  outlying works, not the line. A rule for stations or junctions would have
  to stand on something else (what a market on the line is worth to
  travellers and shippers, say), and neither the first helper nor the
  reviewer wanted to invent one that the prices do not already speak for.
- **`found_saving` is a judgement.** See the table above.
- **Births and deaths are one number.** A game that wants to show a cholera
  year or a baby boom will want them apart. The state keeps only each town's
  rate and its income part (`pp_growth`, `pp_income_term`).
- **Emigrants' cash goes to merchants.** It keeps the money on the island,
  which the conservation check needs, and a passage is a fair thing for it
  to buy, but it is a windfall to the port's merchants with no service made
  for it. It is small: a few thousand people a century.
- **Emigration is told as hunger.** The news item has the kind `famine`, so
  the sweep's `news_famine` counts sailings with the port's hunger items
  (5.3 a run against 2.0 without families, 3.1 against 1.6 with).
- **The test hook.** `st.pp_found_next` orders a town at a cell whatever the
  rule thinks of it. It is there for the tiny-town test and for scenarios;
  nothing in a normal run sets it.
