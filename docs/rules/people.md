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

- **Income.** At `y_still`, 1.3 times bare subsistence, a household just
  replaces itself. Above it more children live and people marry younger, but
  only so far: the rate climbs toward `growth_top` and never reaches it,
  because there is a limit to how fast a people without modern medicine can
  grow however well it eats. At one and a half times subsistence, where the
  game's island stands in 1750, the income part is 0.42 per cent a year; at
  twice 0.95, at three times 1.32, at six times 1.54, at a hundred 1.60. So a
  town's fortunes show in its cradles, and a doubling of mid-century incomes
  adds a fifth of a point, not a population bomb. (These are the game's two
  numbers. On the port's island, which is richer in 1750 and much poorer in
  1900, `y_still` was 1.5 and `growth_top` 0.0165, and everything under "What
  a sweep showed" was measured with those; "In company" says why they moved.)
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

Births less deaths never fall below -6 per cent a year (`peopling::GROWTH_FLOOR`,
-0.06, a setting called `growth_floor` until stage 5 of the pass over the
rules): it is a guard, and in no town of the sweeps has the rate come near
it. And whatever the numbers
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
trades through. For the half dozen spots where that bill is heaviest (summed
over the seven by seven cells round each, the spots more than five cells
apart), each settled on the best ground within two cells of it, it works out
the district a market there would serve (every cell nearer to it than to its
present market), what the district's people would save in carting in a year,
and what its hands earn. A spot must be more than `found_spacing` cells from
every town as the crow flies, on lowland or hills (not mountains; a hill
site counts a little against it when the spot is settled), with a way to
market, and not on a cell that works hold half of. Taking the sites
in order of what they save, a town is founded on the first that passes all
of these:

- the saving is at least `found_saving` of the district's wages,
- the district holds at least `found_cells` cells and `found_people` people,
- no existing town would give up more than `donor_max` of its people, and
- none of the towns giving people has given to a founding, or been founded
  itself, in the last `donor_rest` years.

The estimate is made on last year's paths, so before anything is settled the
site is tried for real: the town's node is switched on, the catchments are
worked out exactly as the yearly step will work them out, and the district is
counted again. Only if it still passes does the town stand (its cells, its
people, the share any one town gives and the rest its givers are owed; the
saving is not reckoned again). Each such try is a full set of paths, so no
more than two sites are tried for real in a year. No town can be founded on
a handful of cells, because the cells are counted first.

At most one town is founded a year, and every town that gave people to it,
however few, is left alone for eight years, as is the new town itself. On
seed 7 the 1750s see two foundings where the port has eight or nine (people
alone on the port's island, at the numbers it was first tuned to; today's
sweep does not count foundings by decade).

Nothing in the rule names a coalfield or a railway. They show up because
that is where tonnage is: a pit ten cells from its market pays more in
carting than a parish of farms does. Each founding is recorded with what its
district was known for (coal, iron, stone, works, or farm where none stands
out: of the trades that take at least a tenth of the district's hands and a
quarter more of them than of the island's, the one that is most over its
island share; services are left out, since they follow people), whether the
site is on the coast, and whether a railway ran within a cell of it on the
day, and the news tells it that way: a colliery town, a mining town (iron),
a quarry town, a town of works, and for a farming district a harbour town on
the coast and a market town inland. Town
names come from a random stream of their own (stream 8), so nothing the port
draws is disturbed.

### People go with their cells

Every year, once the catchments are worked out again, the households of every
cell that now trades through another town go to that town, with their cash
and their landlords' cash. A town's people are taken to be spread over its
cells as its labour at work is, so a town keeps the people whose cells it
keeps; but a town always keeps its own cell, and so never gives up more than
95 per cent of its people in a year, however many of its cells go. A town
founded this year gets its people the same way, and with them
the same share of the old market's stocks and its merchants' cash. Every one
of these is a transfer from one town's purse to another's.

Under the game's rules a town that already stood gets that share of the
stocks too, with the merchants' cash and their measure of sales, whenever it
takes a district: the merchants who served the district go on serving it.
That has a switch of its own, `people.shelves_go`, and a page of its own,
[shelves.md](shelves.md). Without it an old town that took a district opened
January with the shelves of the town it had been, and most of the game's
hungry months and its emigrants came from that January.

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
| `growth_top` | 0.016 (first tuned to 0.0165 on the port's island) | births less deaths a year that income alone can approach |
| `y_still` | 1.3 (first tuned to 1.5 on the port's island) | income over subsistence at which births balance deaths |
| `ease` | 2.1 | how quickly the rate nears its ceiling |
| `town_deaths` | 0.012 | extra deaths a year per head in a great town where all live on town ground |
| `town_size` | 10,000 | townsfolk at which a town is half way to being a great one |
| `dearth_from`, `dearth_full` | 1.0, 0.6 | the worst month's food over need at which dearth begins and is complete |
| `dearth` | 0.03 | deaths a year per head in a complete dearth |
| `emigration` | 0.02 | share who leave the island in a complete dearth |
| `arrivals_max` | 0.1 | migrants a town can take in a year, as a share of its people |
| `found_saving` | 0.08 | carting saved in a year over the district's yearly wages |
| `found_cells`, `found_people` | 40, 3,000 | the least district that can keep a market |
| `found_spacing` | 8 | a new town stands more than this many cells from any other |
| `donor_max` | 0.2 | the most of its people a town gives to one founding |
| `donor_rest` | 8 | years a giving town, or a new one, is left alone |
| `shelves_go` | off; on in the game | the merchants' stock, cash and measure of sales go with a district to a town that already stood, as to a new one ([shelves.md](shelves.md)) |

`growth_top` was set so that the island grows about fivefold on the port's
income path: on seeds 1 to 8 (with `found_saving` at 0.05, before the review's
repairs, and `y_still` at 1.5), 0.016 gave 636,000 people in 1900, 0.0165
gave 667,000 and 0.017 gave 703,000. Both it and `y_still` were set again
beside the game's other rules, for the same fivefold and the same slow start
on an island that begins poorer and ends three times as rich ("In company").
To measure people alone on the port's island as it was tuned there, add
`--set people.y_still=1.5 --set people.growth_top=0.0165`.
`found_saving` decides how many towns there are, and is the one
to turn if you want more or fewer (see the sweep below). The rest are first
values that gave sensible histories and have not been searched. What goes
with a district has no numbers of its own: the weights are the trade itself.

At the edges that were tried (four seeds, six families): a town allowed
anywhere, of any size, every year (`found_saving`, `found_people`,
`found_spacing` and `donor_rest` at 0, `found_cells` 1, `donor_max` 1) fills
all 48 town slots by 1791 and ends with a real wage of 1.19, no run failing;
`growth_top` doubled to 0.033 gives 2.2 million people and a real wage of
0.63, which is Malthus and not a crash, no run failing; and the hardest
dearth the numbers allow (`dearth` and `emigration` 1, `growth_floor` -1,
while it was a setting)
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
  `peopling::found_towns` after `update_roads` (and after the canals have
  counted the year's traffic, which changes nothing it reads), notes what each cell paid to
  reach its market before the paths are worked out again, and calls
  `peopling::move_with_cells` every year where the port calls
  `people::resettle_new_towns` in founding years only.
- `econ::step_month` keeps `st.requests` (what each town's buyers asked for)
  when the switch is on, as it already does when there are families.
- The state's new fields (`pp_*`, in `State` after the port's own; the
  mechanisms merged since have added theirs after them) stay empty while the
  switch is off.
- The sweep gains `town_growth_pct_p10`, `_p50` and `_p90` (every town-year's
  rate of increase, in per cent; under the port, its clamped rule),
  `town_growth_near_old_cap` (the share of town-years within a tenth of a
  point of the port's 1.1 per cent), and with the switch on `late_towns` (the
  towns founded from 1800), `late_towns_coal`, `_iron`, `_stone`, `_works`,
  `_farm`, `late_towns_port`, `late_towns_rail`, `late_town_people_median`,
  `late_town_people_largest` and `emigrants`. With the switch on,
  `growth_capped_*` count a town as held when income has carried it to within
  a twentieth of `growth_top`, which takes an income above 5.4 times
  subsistence (6.25 at the numbers first tuned on the port's island).
- Beside the game's other rules (see "In company"): a new town takes its
  donor's price anchors (`prices::found_town`), and every price that
  `move_with_cells` sets takes its anchor with it (`prices::anchor_follows`);
  the same share of a town's carters goes with a district as of its other
  people (`st.carters`); each town's share of people who came or went with
  their cells is kept in `st.pp_moved`, and the chronicle treats a town that
  gained or lost a tenth that way as redrawn, as it does at a founding; and
  the emigration news carries its own reason, so the chronicle does not give
  it the hunger's.
- `cargo run --release -p isostate-core --example people -- 7 --families 6`
  prints one run by decade under the game's rules, every founding with its
  reason, and one town's people and cells (`--town`, `--every-year` for a
  line a year, `--off` for the rest of the game's rules with the port's
  people rules in place of these, `--candidates` for the sites the rule
  looked at, `--set people.NAME=VALUE`, `--end-year`).

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

## In company

People was merged into the game's rules last of nine, after the merchants'
prices, the Engel curve, the machines, the harvests, the chronicle, the life
cycle for works, the game's railways and carting. The integration's sweeps
are under `.claude/night/reports/core-steps/`, all with six families:
`people-before` and `people-after` (seeds 1 to 6, the game before the merge
and after), `people-s7-12-before` and `people-s7-12` (the same on six seeds
nothing was chosen on), `people-x-raw` (the merge before anything below was
changed), `people-x-off` and `people-s7-12-x-off` (the carters' change
alone), `people-x-at-once` (people without it), `people-x-still` and
`people-x-still-top` (the two numbers), and in `people-probe/` the wages
month by month, the chronicle's counts and the scripted players. No run
failed in any of them. With `--set people.enabled=false --set
carting.adjust=1` every fact of every seed is what it was before the merge
(`people-x-off-at-once`).

Four things had to be fitted together.

**The carters.** As merged, no wage ran away any more (the game before had
a town-month at 7 to 70 times the middle town's wage on every one of twelve
seeds, the port's founding rule at work under the game's rules), but a new
town's wage still went to between two and four and a half times the middle
town's in its first spring, on every island. With carting off it was 1.3
to 1.9, as on this page. The cause was older than people: carters
were hired all at once on last month's bill, and in a town with a heavy
bill that set the wage doubling and halving month about, in a fifth of all
the game's town-months (`carting.md`, "The wage that flipped"). A district
that gets a market of its own is such a town. Its cells cart a short way
now, but its merchants pay the cart onward to the towns that buy its stone
or coal (the worry below about what the founding test counts), and that
bill falls on a few thousand people where it used to be spread over its
mother town. So carters are now taken on by degrees (`carting.adjust`
0.25), and the same share of a town's carters goes with a district as of
its other people. Then, month by month on seeds 1 to 16
(`people-probe/wage-watch-game.txt`):

| | the game before | with people, carters hired at once | now |
|---|---|---|---|
| a town's greatest wage against the middle town's | 7 to 70 | 2.3 to 3.9 | 1.4 to 2.3 |
| its least | 0.23 to 0.36 | 0.29 to 0.40 | 0.33 to 0.57 |
| town-months with the wage moved by half or more | 0.13 to 0.28 | 0.17 to 0.34 | at most 0.002 |
| hungry town-years in a run, and those in towns under three years old | 7 to 14, 6 to 14 | 0 to 3, none | 0 to 4, none |

(The first two columns are seeds 1 to 12, `wage-watch-before.txt` and
`wage-watch-carters-at-once.txt`; a hungry town-year has a month with under
82 per cent of the food it needs.)

**The merchants' anchors.** Under `prices` a town's price stands off a slow
anchor by its stock, and moves toward that by at most 12 per cent a month.
The port's founding rule hands a new town its mother's anchors
(`prices::found_town`); this page's rule did not know of them, so a new
town began with the anchors its empty slot was given in 1750, and every
price the district brought with it was walked back toward those. Now
`open_town` calls `prices::found_town`, and wherever `move_with_cells` sets
a price the anchor moves in the same proportion (`prices::anchor_follows`),
so the price stands as far off its anchor as the stock says. Both rules
also set what a new town's producers expect: the merchants' rule takes the
carriage to the mother town off her expected prices, and this page's then
replaces that, good by good, wherever the district makes or buys anything.

**The chronicle.** It never sets a town against what it was before a
founding redrew its district. Here households go with their cells every
year, and a road or a railway can carry a district from one market to
another with no founding at all: on three seeds, 45 to 170 town-years in a
run saw a town gain or lose more than 3 per cent of its people that way,
and one town took in three and a half times its own people in a year. So a town that
gained or lost a tenth of its people with their cells in a year
(`st.pp_moved`) is redrawn too. And the news of people taking ship has the
kind `famine`; the chronicle gave it the hunger's reason, which speaks of
the food in the town named, and the town named is the port they sailed
from. It now carries its own.

**Two numbers.** With the reviewed numbers the island's people stood still
for twenty years: 135,000 in 1750, 135,000 in 1760, 140,000 in 1770, and
634,000 in 1900. Nothing was broken. With carters real the island of 1750
has 1.5 baskets a head where the port's has 1.76, and 1.5 was `y_still`,
the income at which births balance deaths. The rule was doing what it
says on an island that had moved under it. So `y_still` is now 1.3, which
puts the game's 1750 about as far above the line as the port's was above
1.5; and `growth_top` is 0.016 for 0.0165, as this page said it would want
once the island was richer (it ends at 8.5 baskets a head, not 3.4). On
seeds 1 to 6:

| `y_still`, `growth_top` | people in 1800, 1850, 1900 | growth a year in the 1750s, 1760s, 1770s | baskets a head in 1900 |
|---|---|---|---|
| the game before (the capped rule) | 225, 387, 669 thousand | 0.84, 1.01, 1.06 | 8.50 |
| 1.5, 0.0165 (as reviewed) | 185, 336, 634 | 0.0, 0.4, 0.6 | 8.45 |
| 1.3, 0.0165 | 206, 384, 726 | | 8.10 |
| 1.2, 0.0165 | 216, 411, 785 | | 8.02 |
| 1.35, 0.016 | 198, 360, 667 | 0.30, 0.55, 0.73 | 8.17 |
| 1.3, 0.0155 | 201, 359, 650 | 0.37, 0.59, 0.80 | 8.45 |
| **1.3, 0.016** | 203, 372, 686 | 0.38, 0.61, 0.82 | 8.46 |

The tests of `tests/people.rs` that run on the port's island (Kelrigg, the
hemmed-in sites, and also the sixty years with six families and the run
with the rules on against off) keep the pair they were built on.

**What it does in company**, against the game before, seed for seed. The
middle column is the carters' change alone, to show what is not people's.

| | before | carters by degrees | with people | seeds 7 to 12: before | with people |
|---|---|---|---|---|---|
| people in 1800 | 224,500 | 222,900 | 203,500 (every seed fewer) | 226,400 | 203,000 (every seed) |
| in 1850 | 387,200 | 384,300 | 371,900 | 390,200 | 374,600 |
| in 1900 | 669,100 | 664,100 | 686,000 (4 seeds more, 2 fewer) | 674,000 | 696,700 (5, 1) |
| a town-year's rate, per cent: tenth, middle, ninetieth | 1.10 all | 1.10 all | 0.84, 1.24, 1.48 | 1.10 all | 0.85, 1.25, 1.50 |
| towns in 1800, 1850, 1900 | 13.5, 15.3, 15.3 | 13.8, 16.0, 16.0 | 12.7, 16.3, 16.5 | 12.8, 16.0, 16.0 | 14.0, 19.5, 20.2 |
| last town founded | 1826 | 1818 | 1842 | 1825 | 1846 |
| towns founded from 1800 | | | 3.8 | | 6.2 |
| emigrants, whole run | | | 307 | | 219 |
| baskets a head in 1800 | 2.98 | 2.96 | 2.95 | 3.06 | 3.10 |
| in 1850 | 4.85 | 5.23 | 5.03 | 4.91 | 4.59 |
| in 1900 | 8.50 | 8.39 | 8.46 (4 higher, 2 lower) | 8.62 | 8.13 (1, 5) |
| goods a head in 1900 | 11.4 | 11.3 | 11.1 | 11.3 | 10.9 |
| chained real wage in 1900, 1750 = 1 | 4.45 | 4.45 | 4.45 | 4.37 | 4.07 (every seed lower) |
| urban share in 1900 | 0.415 | 0.416 | 0.414 | 0.424 | 0.419 |
| works standing in 1800, 1850, 1900 | 253, 537, 671 | 260, 516, 668 | 242, 521, 687 | 274, 529, 664 | 283, 530, 752 |
| town cells, industry cells in 1900 | 117, 219 | 116.5, 218 | 120.5, 225 | 120, 219 | 124.5, 217.5 |
| railway in 1850, 1870, 1900, km | 1,420, 2,191, 2,643 | 1,650, 2,332, 2,645 | 1,520, 2,192, 2,644 | 1,144, 2,009, 2,616 | 944, 2,007, 2,617 |
| price swings: grain, iron, rails, cloth | 0.112, 0.172, 0.179, 0.038 | 0.109, 0.171, 0.187, 0.039 | 0.108, 0.171, 0.185, 0.036 | 0.114, 0.176, 0.167, 0.037 | 0.115, 0.166, 0.166, 0.038 |
| famine years in a run | 2.2 | 1.8 | 1.3 | 1.8 | 1.3 |
| regional stories told | 44.2 | 41.2 | 37.2 | 49.7 | 44.5 |
| seconds a run, six at a time | 17.6 | | 17.9 | | |

(The seconds are the old rules and the new run back to back; the sweep
called `people-before` ran on a quieter machine and shows 13.5.)

What that says, set against what this page says of people alone:

- **The people.** The slow start is here as it is alone: 0.4 per cent a
  year in the 1750s, 0.6 in the 1760s, 0.8 in the 1770s, 1.0 to 1.1 in the
  1780s and 1.2 to 1.25 from the 1790s to the end, so 21,000 to 23,000
  fewer people than the game before in 1800 on every one of twelve seeds,
  and by 1900 2 to 3 per cent more: 5.1 times the people of 1750 (alone
  5.0). The fastest
  town-years are faster than alone (1.48 per cent at the ninetieth for
  1.32), because the island is richer.
- **Towns.** The game before founded fewer towns in the first rush than the
  port does alone (13 by 1800, not 18), so people does not thin them out
  here; it goes on founding where the game before stopped. The last town is
  founded in the 1840s, not the 1820s (alone: the 1860s, not about 1800).
  Seeds 1 to 6 end with one town more and seeds 7 to 12 with four more
  (every seed of those six). Of the towns founded from 1800, 3.8 and 6.2 a
  seed (alone 7.0), most stand among quarries and claypits (2.2 and 4.2),
  then farms (1.0 and 0.7), works (0.5 and 0.8) and coal (0.2 and 0.5); 0.7
  and 0.5 are on the coast, one in twenty-five beside a railway. The middle one has 14,000
  people in 1900 (alone 30,000), the largest in a seed 26,000 and 40,000.
- **Living standards.** Alone, the late wage with families leant 3 per cent
  lower. In company the baskets of 1900 are 8.46 for 8.50 on seeds 1 to 6
  and 8.13 for 8.62 on seeds 7 to 12: together 3 per cent lower, five seeds
  higher and seven lower. On seeds 7 to 12, where there are four more
  towns, it is 6 per cent and the chained real wage of 1900 is lower on
  every seed. More towns cost more, as the `found_saving` table says, and
  so do 3 per cent more people on the same land.
- **The railway** of 1850 is 7 per cent longer on seeds 1 to 6 and 17 per
  cent shorter on seeds 7 to 12, and the same from 1870 on. Against the
  carters' change alone it is 8 and 21 per cent shorter, which is what
  people does alone (a fifth less line in 1850, caught up by 1870).
- **Hunger and sailing.** Famine years fall from 2.0 a run to 1.3, since a
  new town no longer starves in its first year. A few hundred people a run
  take ship (alone 1,200).
- **Works.** As many as before to 1850; in 1900 2 per cent more on seeds 1
  to 6 and 13 per cent more on seeds 7 to 12 (every seed), with the towns.
- **Prices** swing as they did, and nothing sits at a floor or a ceiling.
- **The news.** The chronicle's feed is the length it was (58.9 items kept
  in a run for 58.2, on seeds 1 to 12; `people-probe/chronicle-game.txt`
  and `chronicle-before.txt`). It tells 41 regional stories a run where it
  told 47: fewer declines (11.2 for 13.0) and fewer fastest-growing towns
  (10.2 for 11.6), since a town whose district was redrawn is passed over
  for some years, and fewer districts of closures (12.2 for 14.2). Eight
  are kept, as before.
- **The scripted players** (`people-probe/bots-game.txt`, against
  `railways-probe/bots-game.txt` from before) finish all sixteen games.
  The landlord and the thoughtful player are first of six in 1900 on every
  seed, as they were. The player who takes every default is where this
  page's first worry said it would be, on two islands of four: first on
  seeds 1 and 3 (it was second and first), fourth on seeds 2 and 7 (it was
  first), having saved 16 and 15 per cent of its income in the 1750s where
  it saved 27 and 23. The founder is second, first, second and fourth (it
  was third, first, second and fourth). In money the player ends with 14 to
  86 million where it had 23 to 76. One game each.

**Today.** Everything above in this section was measured at the merge and is
kept as it was. The mines and the canals have been merged into the game's
rules since, and the books of account beside them, so the island has moved
again. The game's sweep of seeds 1 to 8 with six families as it stands
(`isostate-core sweep --seeds 1-8 --rules game --families 6`, in
`.claude/night/reports/merged-game-f6-l`) reads, beside the merge's seeds 1
to 6:

| | at the merge, seeds 1 to 6 | today, seeds 1 to 8 |
|---|---|---|
| people in 1800, 1850, 1900 | 203,500, 371,900, 686,000 | 211,600, 393,300, 731,900 |
| a town-year's rate, per cent: tenth, middle, ninetieth | 0.84, 1.24, 1.48 | 0.92, 1.27, 1.50 |
| towns in 1800, 1850, 1900 | 12.7, 16.3, 16.5 | 12.6, 16.9, 17.8 |
| last town founded | 1842 | 1847 (1814 to 1873) |
| towns founded from 1800 | 3.8 | 5.1 (2 to 7) |
| of those: quarry, farm, works, coal, iron | 2.2, 1.0, 0.5, 0.2, none | 2.4, 1.1, 0.9, 0.8, none |
| of those on the coast, and beside a railway | 0.7, one in twenty-five | 0.4, 0.8 |
| people of the middle and the largest late town in 1900 | 14,000, 26,000 | 20,300, 47,300 |
| emigrants, whole run | 307 | 133 (0 to 470) |
| baskets a head in 1800, 1850, 1900 | 2.95, 5.03, 8.46 | 3.30, 5.18, 8.76 |
| chained real wage in 1900, 1750 = 1 | 4.45 | 4.24 |
| urban share in 1900 | 0.414 | 0.419 |
| famine years in a run | 1.3 | 0.4 |
| railway in 1850, 1870, 1900, km | 1,520, 2,192, 2,644 | 1,328, 1,835, 2,248 |

The slow start is still there, a little quicker: from `annual.csv`, the mean
of the eight seeds grows 0.5 per cent a year in the 1750s, 0.7 in the 1760s,
0.9 in the 1770s, 1.1 in the 1780s and 1.22 to 1.28 from the 1790s on, and
the island ends with 5.4 times its people of 1750 where the merge had 5.1.
That is the worry about the two numbers below come true in a small way: the
island stands richer than it did, so it grows faster. Late towns are a
little more common and larger, and a sixth of them now stand beside a
railway where almost none did. Nothing here was searched again. The other
tables of this page (the merge's, and those under "What a sweep showed")
are not in today's sweep in the same form, or were measured on other seeds
or under the port, and are left as measured.

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
more than births and the arrivals limit allow. Those histories are the
port's island's at `y_still` 1.5 and `growth_top` 0.0165, and the tests keep
that pair. One more runs the whole of the game's rules with six families
on seed 4 to 1800, month by month: every town's wage within 0.2 and 3 times
the middle town's, the wage moving by half or more in under one town-month
in two hundred, and every new town opening with each price between half and
twice its merchants' anchor. `prices.rs` has a test of the anchor that
follows a price, and `tests/chronicle.rs` now expects a town to be redrawn
by a tenth of its people changing market as well as by a founding.

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
  company. It was ("In company"): 3 per cent lower again on twelve seeds,
  and 6 per cent on the six where four more towns stand.
- **Two numbers were set again on six seeds.** `y_still` 1.3 and
  `growth_top` 0.016 were chosen on seeds 1 to 6 and checked on 7 to 12,
  for a start as slow as this page describes and an island of about five
  times its people. They answer to where the game's island stands in 1750
  and in 1900, so anything that moves those (carting's `labor`, the
  machines) moves them again. If the island should rather stand still for
  its first twenty years, as it did at the reviewed numbers, 1.5 does it.
- **A town of exporters carts a great deal.** With carters real, a new
  quarry or colliery town can have three hands in ten on the road against
  one in seven on the island, because the cart from its market to its
  buyers is its merchants' to pay. It is the next worry in the flesh: the
  founding test credited the district with a saving that the island, and
  now the town's own labour market, partly pays back.
- **The chronicle tells a seventh fewer regional stories.** A town that
  gains or loses a tenth of its people with their cells is passed over for
  the years its stories look back. The line of a tenth is the chronicle's
  own least decline; a district that drifts over at 5 per cent a year for
  three years is not caught and could be told as a decline.
- **The founding test counts the producers' saving, not the island's.** It
  adds up what the district's cells would save in carting to a nearer market.
  Coal that the district does not burn itself still has to get from the new
  market to the old one, and that leg is the merchants' to pay. So the test
  is a fair measure of where a market is missed and a poor measure of what
  the island gains, and a town of exporters gains least. A better test would
  net off the onward journey of what the district sells beyond itself.
- **If incomes rise, so will the population.** That is the point of the
  rule, and it is bounded: at six times subsistence the income part is 1.54
  per cent a year against 1.32 at three (1.56 against 1.27 at the numbers
  first tuned on the port's island). But fifty years at the higher rate
  is 15 per cent more people in 1900, and they press on the same land.
  `growth_top` was tuned on the port's incomes and will want turning down a
  little if the island gets much richer. (It did, and was: 0.016 beside the
  game's other rules. See "In company".)
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
- **Emigration is told as hunger.** The news item has the kind `famine`
  (`peopling::growth`), so the sweep's `news_famine` counts sailings with the
  port's hunger items (5.3 a run against 2.0 without families, 3.1 against
  1.6 with; people alone on the port's island, at its first numbers). So does
  `famine_years`, which counts every year with an item of that kind: a year
  in which a thousandth of the island sailed after a month that fed fewer
  than all, but the port's hunger news did not fire (no town of three years'
  standing fell under its line of 82 per cent), is counted as a famine year.
  The chronicle tells the two apart; the sweep does not. Today's game sweep
  has 0.5 items of that kind a run and 0.4 famine years in all, so whatever
  share of them is sailings, it is small.
- **The test hook.** `st.pp_found_next` orders a town at a cell whatever the
  rule thinks of it. It is there for the tiny-town test and for scenarios;
  nothing in a normal run sets it.
