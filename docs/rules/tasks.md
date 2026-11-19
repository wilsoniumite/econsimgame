# Hands and machines in the works and at the pits

Not in the Python. `crates/isostate-core/src/tasks.rs` and
`data/tasks.toml`, switched by `tasks.enabled`, and on the land by
`tasks.land`: both off under the port's rules, on under the game's. It is
the fifth stage of the run to 2050, in two halves: the works first, then the
land ("On the land", below). **The one machine in it, the hoist, is a
stand-in**: made of machinery and burning coal, so that the rule can be
proved on the island as it stands, with nothing after 1900 in the content;
the electric drive takes its place when that content lands. It hauls in the
works and at the quarries, clay pits and mines. **The fields are still out
of reach**: no machine names the farms, the pastures or the woods (the
tractor is a later stage's), nor the carters, so they keep the machines of
[machines.md](machines.md) and nothing else, and some of the hands the works
and the pits let go find work on them. Nothing can be found before 1901, so
the played century is untouched: every fact of the standing check is the
same to the digit with either switch on and off. Two more settings are for
sweeps and are unset under both rules: a ceiling on what the ground may
yield, and a control that holds the machine still.

## The standing check

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set tasks.enabled=false,true --standing --out DIR
```

```text
standing check: 12 of 12 inside (0 moved, 0 OUTSIDE, 0 without a value): tasks.enabled=true against tasks.enabled=false

                                  tasks.enabled=false                        tasks.enabled=true
the twelve
  pop_1800                                    202,320  [191,936 .. 213,770]             202,320  inside
  pop_1850                                    364,624  [347,461 .. 387,834]             364,624  inside
  pop_1900                                    673,962  [637,660 .. 710,550]             673,962  inside
  real_wage_chained_1850_vs_1750                 3.00  [2.51 .. 3.46]                      3.00  inside
  real_wage_chained_1900_vs_1750                 6.03  [5.10 .. 6.94]                      6.03  inside
  towns_1900                                     19.4  [13 .. 23]                          19.4  inside
  urban_share_1900                              0.465  [0.457 .. 0.489]                   0.465  inside
  rail_km_1900                                  2,482  [2,100 .. 2,710]                   2,482  inside
  rail_first_year                                1832  [1821 .. 1849]                      1832  inside
  income_share_work_1850                        0.660  [0.606 .. 0.705]                   0.660  inside
  income_share_work_1900                        0.670  [0.637 .. 0.700]                   0.670  inside
  income_share_land_1900                        0.165  [0.157 .. 0.176]                   0.165  inside
also
  last_town_founded                              1892  [1878 .. 1899]                      1892  inside
```

Set seed by seed, of the 896 facts of a run to 1900 the only ones that
differ between the two arms are the rule's own, which a run with the rule
off does not have.

The land's switch passes the same check:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set tasks.land=false,true --standing --out DIR
```

```text
standing check: 12 of 12 inside (0 moved, 0 OUTSIDE, 0 without a value): tasks.land=true against tasks.land=false
```

and of the 908 facts, seed by seed, the only ones that differ are the land's
own (`land_rent_machines_Y` and `income_share_ground_Y`, which a run with the
switch off does not have).

## What it does

**Classes of task.** Every hour of a works' labour is of one of five
classes: *haul* (lifting, carrying, turning a shaft, hewing), *tend*
(feeding and minding a process, fitting, driving), *clerk* (records,
reckoning, routine dealing), *judge* (design, selling, managing) and
*presence*, work that only a person may do, which no machine may take. Each
method's labour is split between them by kind (`data/tasks.toml`):

| kind | methods | haul | tend | clerk | judge | presence |
|---|---|---|---|---|---|---|
| furnaces, forges and rolling mills | bloomery, the three blast furnaces, crucible steel, the converter, the forge and both rolling mills | .45 | .40 | .03 | .12 | 0 |
| mills and factories | the textile mill and every works of the tree that makes a good in a plant | .20 | .60 | .05 | .15 | 0 |
| engine and machine works | millwright, engine works, machine shop, engineering works | .25 | .50 | .05 | .20 | 0 |
| hand trades | smithy, hand loom, saw pit, charcoal burning | .30 | .45 | 0 | .25 | 0 |
| builders | masons, carpenters, bricklayers, contract builders | .40 | .40 | .02 | .18 | 0 |
| shops and trades | services, the emporium | .15 | .25 | .25 | .25 | .10 |

The land's seven sectors and the carters have their shares in the file too
(below, "On the land"); the carters' wait, since no machine reaches them.

**The schedule.** Within a class the tasks run from the easiest for a
machine to the hardest. At task `z` (0 to 1) a hand is worth `worth(z)`
machine units, spread evenly in the logarithm between two knots: hauling
0.3 to 100, tending 1 to 1,000, clerking 1 to 3,000, judging 10 to 30,000.
The file writes them as knots, `[[0, 0.3], [1, 100]]`, and the code reads
any number of them, so a bent schedule can be loaded later. The machine
units that hold the tasks from `a` to `b` for an hour of the class are the
schedule summed over that part: for two knots `lo` and `hi`,
`lo * ((hi/lo)^b - (hi/lo)^a) / ln(hi/lo)`.

**What a unit costs.** Each month, in each town, for each machine the
island has found: `c = ((hurdle + 1/life) / 12 * plant + run + minders *
wage) / (quality * dial)`, the plant and the run at the town's prices. That
is the return a works must see (`hurdle`, 0.12) and the wear on the
machine's plant, its fuel, and the hands who mind it, over the units one
machine does.

**The town's target** is the share of the class where the town's wage over
`c` falls on the schedule, kept between the machine's `from` and `reach`:
every task at which a hand is worth fewer units than the wage buys is
cheaper done by machine. Where a class has more than one machine they are
taken cheapest first, each holding what lies above the one before it (only
one exists in this half).

**The works.** A new works is built at its town's target: new plant is
today's technique. A standing works moves toward it by `1 / (12 *
adopt_years)` of the gap a month (`adopt_years` 8), times how well last
month's order for its machines was filled, and falls no faster than its
machines wear out. Where a works' machines hold some part of a class:

- its hands a unit are the port's times the sum over classes of `share *
  (1 - held + minders of the machines held)`;
- the machines' run goods join its inputs a unit, so it buys them each
  month and is held back as any works is when they run short;
- the machines' plant, sized to its capacity, is ordered through a table of
  its own (the works' order for capacity is wiped when no capacity is on
  order, so it cannot hold them): a quarter of what is missing a month,
  served after the day's trade as the plant orders are, paid from its cash
  and by its backers when it is short (`firms::back_firm`), and kept back
  from its dividend while it is on order (`works::kept_for_order`). The
  machines wear by a twelfth of their life a month, and their book with them.

A run where no works holds any part of any class (every run to 1900) passes
none of this, and is the same run to the byte.

**Sites and reviews.** `firms::evaluate_sites`, `firms::entry` and
`firms::review` see the works at its town's target: the machines' plant for
a unit of capacity is part of what that capacity costs, and the run goods
and the minders come off the margin. Without that every trade in a
mechanised town would show a return it does not earn.

**The books and the gauges.** The machines' plant is counted by the books
as building, as a works' capacity is, so both balances hold. The gauges'
floor reads the class of presence from the table (it had a table of its own,
which named only the services; the emporium now holds its tenth too, which
lifted `gauge_floor_1900` from 0.023 to 0.065 and moved nothing else), the
machines' book is counted in the plant behind a year's income
(`gauge_plant_years`), and their wear in the wear (`gauge_work_net`).

## The hoist

| | |
|---|---|
| class | haul, from 0 to 1 of it, in works and (under `tasks.land`) at the quarries, clay pits, iron mines and collieries |
| plant | 82 machinery a machine; life 20 years |
| run | 5.2 coal a month |
| minders | 0.104 of a hand a month (a tenth of its cost when found) |
| quality | grows 3 per cent a year from the year found, to a cap of 8 |
| found | at the first year's turn from 1901 at which the island has made 4.1 million units of machinery since 1750 |
| scale | 1.14 |

It was sized from island 7's wage and prices of 1900 (a wage of 8.18 a
month, machinery 4.40, coal 0.494): when found a unit costs 1.04 of a
month's wage, of which six tenths is return and wear on its plant (44
months' wages of plant), a tenth its minders and the rest its coal. At that
cost a hand is worth 0.96 units, which takes `ln(0.96 / 0.3) / ln(100 /
0.3)` = 0.20 of hauling. The threshold was set so that island 7 finds it in
1903. `scale` multiplies everything one machine takes; it was refitted once,
from a sweep at 1.0, in which the towns' target in the year found was 0.223
on the mean island, so that the mean is a fifth.

## What a sweep showed

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set tasks.enabled=false,true --out DIR
```

As run on 19 November 2026 on the owners' branch, with the long run's
anchors in, before the land was in reach (the "rule on" column is the run
with `tasks.land=false` today, to the digit). Means over islands 1 to 8,
[least .. greatest] where it helps.

| | rule off | rule on |
|---|---|---|
| the hoist found | | 1903 [1901 .. 1907] |
| the towns' target in the year found | | 0.200 [0.173 .. 0.221] |
| share of the works' hauling done by machine, 1910, 1920, 1930, 1940 | | 0.148, 0.256, 0.322, 0.375 |
| the same, 1950, 1960, 1970, 1980 | | 0.426, 0.473, 0.518, 0.545 |
| the same, 2000, 2020, 2050 | | 0.546, 0.536, 0.515 |
| the year half of it is done by machine | | 1967 [1958 .. 1977] |
| the margin's gap, 1920, 1950, 1980, 2000, 2050 | | 1.39, 1.23, 1.11, 1.06, 1.02 |
| the margin's gap forty years after the find | | 1.25 [1.21 .. 1.28] |
| the machines' book in years of the wage bill, 1950, 2000, 2050 | | 0.051, 0.040, 0.019 |
| hands on the land, 1920, 1950, 1980 | 0.263, 0.271, 0.308 | 0.266, 0.290, 0.320 |
| the same, 2000, 2020, 2050 | 0.345, 0.360, 0.453 | 0.342, 0.385, 0.471 |
| hands in works, 1950, 2000, 2050 | 0.519, 0.455, 0.308 | 0.490, 0.440, 0.275 |
| hands in the diggings, 1950, 2000, 2050 | 0.131, 0.121, 0.140 | 0.139, 0.133, 0.152 |
| the towns' share, 1950, 2000, 2050 | 0.448, 0.396, 0.287 | 0.432, 0.379, 0.260 |
| work's share of income, gross, 1950, 2000, 2050 | 0.659, 0.609, 0.572 | 0.646, 0.598, 0.575 |
| work's share, net of wear, 1950, 2000, 2050 | 0.680, 0.657, 0.596 | 0.671, 0.651, 0.598 |

**Dearer towns first.** On each island, across the towns whose works haul
in both years, the rank correlation between a town's wage in the year the
hoist is found and the share of its works' hauling held ten years on
(`hoist_dearer_first`): 0.58, 0.69, 0.29, 0.32, 0.74, 0.61, 0.68 and 0.65,
mean 0.57. Positive on every island.

**The margin.** The wage over the machine's cost at the last task each
works holds is 1 when the works stand at their town's target. It falls from
1.88 in 1910 to 1.23 by 1950 and is **not within a tenth of one forty years
after the find** (1.21 to 1.28 on every island). The works trail a target
that rises with the hoist's quality, 3 per cent a year, by about
`adopt_years` years of its rise (some 0.04 of the class, which is a factor
of `(100 / 0.3)^0.04` = 1.26 in the gap), and part of each month's step
waits on the machine order's fill. Once the quality stops at its cap, about
1973, the gap closes: 1.11 in 1980, 1.06 in 2000, 1.02 in 2050. The rule
comes to rest where it should; at these settings it cannot keep up with a
machine that is still improving.

**Hands go back to the fields.** With the works mechanising and the land
not, the share of hands on the land is higher with the rule at every mark
but 2000 (0.290 against 0.271 in 1950, 0.471 against 0.453 in 2050), the
works' share lower and the towns' share lower by 0.016 to 0.027. The second
half of the stage, the machines on the land, exists to undo this.

**The required return.**

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 1960 --set hurdle=0.09,0.12,0.15 --out DIR
```

The share of the works' hauling done by machine in 1950 is 0.431, 0.426 and
0.369 at a hurdle of 0.09, 0.12 and 0.15. `hurdle` is every works' required
return, so it changes the whole played century and with it the year each
island finds the hoist (1913 on island 1 at 0.09, 1919 on island 3 at
0.15). On the islands that find it in 1901 in both arms the steps are
+0.016 from 0.12 to 0.09 (islands 2, 5 and 8) and -0.036 from 0.12 to 0.15
(islands 5, 6 and 8), where the unit's cost alone (0.894 and 1.106 times as
much) moves the target by +0.019 and -0.017 of the class.

## On the land

`tasks.land`, true under `game()` and false by default. A land unit (a
farm's, a pasture's or a wood's share of a cell, or a working at a pit head)
hires work as the port's recipe says, `lam * (Cc / w)^(1/alpha)`, and pays
`1 - alpha` of what its crop is worth to it for that work. Under this switch
it pays that share to a mixture of hands and machine units, wherever a
machine names its sector:

- **The classes.** Each sector's work is split between the classes like a
  method's: the farm .45 hauling, .40 tending, .15 judging; the pasture .30,
  .50, .20; the woods, the quarries, the clay pits and the mines .60, .30,
  .10; the carters .70, .20, .10 (`data/tasks.toml`).
- **What machines hold.** In each town each sector has a target for each
  class, worked out as the works' is from the machines that reach it, and
  the share its machines hold (`land_auto`) moves toward it by the same rule
  as a works' share: a ninety-sixth of the gap a month, times the fill of
  the sector's last order for plant, and down no faster than wear. It starts
  from nothing in the year the machine is found.
- **The cost of an hour's work.** Where machines hold part of a sector's
  work, an hour of it costs `cL = hands * w + cost`: `hands` is the work left
  to hands, `sum over classes of share * (1 - held)`, and the minders of the
  machine units held; `cost` is those units' return and wear on their plant,
  `(hurdle + 1/life) / 12` of its worth, and their run goods, at the prices
  the unit expects. The unit hires `lam * (Cc / cL)^(1/alpha)` of work, which
  takes `hands` hands an hour, and makes its crop from the work. The wage
  search keeps its shape (one sum a town and sector, read at `cL` for `w`),
  and the land's bids (`land::sector_bids`) and a prospective pit head's
  sums (`mines::review`) read the same `cL`.
- **The machines' goods.** Each sector asks each month for its machines'
  run goods with the day's trade, and for a quarter of what its plant lacks
  beside the works' plant orders; each unit pays for them by its part of the
  sector's work, out of its revenue, so they come off the rent, and the books
  count them among the land's inputs as they count the port's farm machinery.
  The return on the plant is never paid to anyone: it stays in the rent, so
  whoever holds the land holds its machines. Where the run goods are short
  the machines stand for that part of their work and the crop is the smaller.
- **The rent's split.** The gauges split the farms' and pits' rent in two:
  the return on the land's machines, `hurdle` a year on their plant's worth,
  and the ground's part, the rest (`rent_machines` and `rent_ground` in the
  gauges' year; the facts `land_rent_machines_Y`, the return over that rent,
  and `income_share_ground_Y`, the ground's part over income).

The hauling boosts of [machines.md](machines.md) (pumps, wagonways, farm
machinery and the quarries' machinery) stay as they are. A run in which no
machine holds any part of any land sector (every run to 1900) passes none of
this and is the same run to the byte: with `tasks.land=false` the eight
islands to 1960 give the first half's 1,523 facts with none differing.

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set tasks.land=false,true --out DIR
```

As run on 19 November 2026 (the first half's branch with this half on top).
Means over islands 1 to 8; "none" is the first half's sweep with
`tasks.enabled=false`, "works" is `tasks.land=false`, "land" is
`tasks.land=true`.

| | none | works | land |
|---|---|---|---|
| hands on the land (farms, pastures, woods), 1950, 1980, 2000, 2050 | .271, .308, .345, .453 | .290, .320, .342, .471 | .290, .335, .355, .465 |
| hands in the pits and quarries, 1950, 1980, 2000, 2050 | .131, .129, .121, .140 | .140, .136, .133, .152 | .124, .114, .117, .143 |
| hands in works, 1950, 1980, 2000, 2050 | .519, .485, .455, .308 | .490, .460, .440, .275 | .504, .469, .441, .287 |
| the towns' share, 1950, 1980, 2000, 2050 | .448, .424, .396, .287 | .432, .400, .379, .260 | .442, .413, .394, .270 |
| the works' hauling done by machine, 1950, 1980, 2050 | | .426, .545, .515 | .441, .574, .537 |
| the pits' hauling done by machine, 1920, 1950, 1980, 2050 | | | .261, .454, .596, .563 |
| work's share, gross, 1950, 1980, 2000, 2050 | .659, .652, .609, .572 | .646, .623, .598, .575 | .650, .632, .587, .577 |
| work's share, net of wear, 1950, 1980, 2000, 2050 | .680, .680, .657, .596 | .671, .659, .651, .598 | .675, .668, .639, .601 |
| what a wage buys (chained), 1950, 2000, 2050 (1750 = 1) | 6.88, 5.84, 3.29 | 7.13, 5.53, 3.23 | 7.37, 5.72, 3.45 |
| the land machines' return over the land's rent, 1950, 2000, 2050 | | | .026, .029, .021 |

**The pits let hands go; the fields take some of them.** At the pits the
hoist takes hauling as fast as in the works (0.454 of it in 1950, 0.596 in
1980), and the share of hands in the pits falls below even the island with
no machine at all from 1950 to 2000. The fields are out of reach, and the
share of hands on them is no lower than with the works alone: the same in
1950, higher in 1980 and 2000. Counted together, of the hands the first
half sent to the land and the pits this half takes back three fifths in
1950 (0.016 of 0.027), a third in 1980 and a half in 2050; it gives the
towns back two thirds of the share the first half took in 1950 and nearly
all of it in 2000. The towns' share is higher with the land on than without
at every mark from 1920, though still below the island with no machines.

**Work's share hardly moves.** Gross and net, it is within a point of the
works-only run at most marks. With one machine that hauls, a cap on its
quality of 8 and the fields out of reach, nothing in this stage could move
it much; the stage is a test of the plumbing, and of the direction hands
go. **What a wage buys** is higher with the land on at every mark (7.37
against 7.13 in 1950, 3.45 against 3.23 in 2050; not traced).

## The ceiling on yield

`tasks.yield_cap`, unset under both rules: a setting for sweeps (the
question whether land limits what the island can make is the designer's,
and the played game has no ceiling, as the land has never had one). While
it is set, a unit's output may not pass `yield_cap` times what the same
ground yielded in its best year before 1950, for each share of the cell it
holds. Ground with no year before 1950 (a field ploughed or a pit sunk
since) is held to its sector's mean best. Where the ceiling binds, the unit
hires only the work it needs, and the good's price rises until what is
wanted fits; which units stand at the ceiling is taken from last month
(whether the work a unit wanted at last month's wage would have taken it
there), so in the wage search a town's sum is still two numbers a sector.
Before 1950 nothing acts and the run is the run without it (a test holds
island 7 to 1949 to the digit).

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set tasks.yield_cap=null,2 --out DIR
```

| | unset | 2 |
|---|---|---|
| the land's output at the ceiling, by value, 1960, 2000, 2020, 2050 | | .001, .013, .043, .142 |
| the land's share of income, 1960, 2000, 2020, 2050 | .171, .163, .161, .184 | .171, .157, .171, .307 |
| the wage over the price of the land's goods (the gauges' fork), 1960, 2000, 2020, 2050 | 6.68, 5.58, 4.73, 3.83 | 6.67, 5.54, 4.44, 2.61 |
| work's share, gross, 2000, 2020, 2050 | .587, .559, .577 | .581, .534, .434 |
| hands on the land, 2000, 2050 | .355, .465 | .336, .346 |
| what a wage buys (chained), 2000, 2050 | 5.72, 3.45 | 5.65, 2.57 |

The island's yields grow slowly after 1950 (no machine reaches the fields,
and nothing new makes ground yield more), so twice the best year before
1950 is far off for decades: a
thousandth of the land's output stands at it in 1960 and a hundredth in
2000. It bites as the people double after 2000: by 2050 a seventh of the
land's output is at the ceiling, the land takes 0.31 of income where it
took 0.18, work 0.43 where it took 0.58, and the wage buys two thirds of the
land's goods it bought without the ceiling. That is the repository's world
on the island's own terms, arriving late because the island's yields were
never pushed before 1950.

## The control

`tasks.freeze_from`, unset under both rules: every machine's quality is
held still from that year, so that the machines in place stay and spread no
further than the prices of 1950 would take them.

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 1980 --set tasks.freeze_from=null,1950 --out DIR
```

| | growing | held from 1950 |
|---|---|---|
| the works' hauling done by machine, 1950, 1960, 1970, 1980 | .441, .492, .540, .574 | .441, .464, .461, .456 |
| the pits' hauling done by machine, 1950, 1980 | .454, .596 | .454, .479 |
| work's share, net of wear, 1950, 1960, 1970, 1980 | .675, .672, .666, .668 | .675, .672, .669, .664 |
| work's share, gross, 1950, 1980 | .650, .632 | .650, .629 |
| investment's share of spending, 1950, 1960, 1970, 1980 | .048, .044, .064, .057 | .048, .039, .060, .058 |

From 1950 to 1980 the machines' share of hauling rises by 0.13 in the works
and 0.14 at the pits in one arm and stays put in the other, and work's share
net of wear falls by 0.006 in the first and 0.010 in the second. The frozen
arm's share does not come back: whatever lowered it from 1950 was not the
machine. Investment's share of spending rose by about a point in both arms,
which on the long run's own measure (a point of investment takes about a
point off work's measured share) is about the size of the fall; so it was
investment, and the island going on, not the margin.

## The numbers

`TasksParams`, in `tasks.rs`: `enabled`, false by default and true under
`game()`; `land`, likewise; `adopt_years`, 8; `dial`, 1, which multiplies
every machine's quality; `yield_cap` and `freeze_from`, unset under both
(for sweeps). A sandbox's Rules tab offers the dial among its most asked, in
multiples from a tenth to a hundredfold (`rulebook::WIDE_OFFERS`), and the
Markets panel's Work half draws the part of each class machines do. `land`, `yield_cap` and `freeze_from` are left out of a run's
`params.json` while off or unset, so a run with them so writes the first
half's file. The classes, the spreads, the shares and the machine are content,
in `data/tasks.toml`, and only the game's content has them
(`content::game_content`, laid again by name when the tree's overlay
rebuilds the content).

The facts, at the sweep's marks: `auto_haul_works_Y` (the share of the
works' hauling done by machine, each class weighted by its work in each
works), `hours_CLASS_Y` for the five classes (hands at their tasks in works,
worker-months a month) and `hours_minders_Y`, `waterline_haul_Y` (the wage
over the cheapest unit's cost, work-weighted), `margin_gap_haul_Y`,
`machines_years_Y` (their book over the year's wage bill); and for each
machine `NAME_found_year`, `NAME_half_year`, `NAME_share_found`,
`NAME_auto_after10`, `NAME_margin_gap_after40` and `NAME_dearer_first`.
Under `tasks.land`: `auto_haul_pits_Y` (the share of the quarries', clay
pits' and mines' hauling done by machine, each weighted by its work),
`land_rent_machines_Y` and `income_share_ground_Y` (above); while
`yield_cap` is set, `yield_capped_share_Y` (the value of the land's planned
output that stands at the ceiling, over all of it). The shares of hands on
the land, in the pits and quarries and in the works are the sweep's own
`farm_share_Y`, `diggings_share_Y` and `industry_share_Y`.

`cargo run --release -p isostate-core --example tasks -- --seeds 1-8
--end-year 2050` prints a line a year from 1895 for each island: the
machine's quality, the share of hauling held in works and at the pits and
the works' target, the waterline,
the gap, the book, the hands on the land and in works, the towns' share,
the wage and the prices of machinery and coal.

## What worries me

- **The hoist is a stand-in.** It is made of the island's own machinery and
  burns its own coal. On the mean island coal's price is hardly moved by it
  (0.870 against 0.865 in 1940), and machinery is a tenth cheaper with the
  rule (3.19 against 3.53 in 1940), perhaps because the hoists' orders take
  its makers further down their learning curves (not traced). The electric drive will run on power from stations that
  are themselves works with classes of task.
- **The fields are out of reach.** The pits let hands go, but the farms,
  pastures and woods take as many hands as with the works alone, or more,
  and the towns' share is still below the island with no machines. Read
  nothing about the island's twentieth century until a tractor exists.
- **The land's machines are bought as inputs.** A sector's plant is bought
  out of its revenue and counted with its inputs, as the port's farm
  machinery is, not as investment; the books balance, but the gauges' plant
  behind a year's income does not count it. Its return is reckoned at the
  month's prices on the plant in place, not on what it cost.
- **A sector holds machines it has not been delivered**, as a works does,
  and a town founded after the find starts its land from nothing where its
  works start at the target.
- **The ceiling's best years are of a unit's planned crop** (an average
  year's), by sector and cell, per share of the cell the use held; how that
  behaves on a cell whose use changed a great deal before 1950 is not looked
  at.
- **The hoist stops improving at a quality of 8**, about 1973, and the wage
  falls after 2000 on an island with nothing new to make, so the works'
  share of hauling stalls at 0.55 and eases back to 0.52 by 2050.
- **A works may hold machines it has not been delivered.** Its share rises
  each month by a step times the fill of its last order, and its hands fall
  at once; the plant follows a quarter of what is missing a month. Where
  machinery is short for years the hands fall ahead of the plant.
- **One machine for each class is all the code has been tried with.** With
  several, the town's uses are taken cheapest first, and a works holding
  more than its town would has the dearest of them hold the rest.
- The plant a unit of capacity takes is reckoned at the sites' mean hands a
  unit in `evaluate_sites`, so every site of one method in one town is
  charged the same for its machines.
