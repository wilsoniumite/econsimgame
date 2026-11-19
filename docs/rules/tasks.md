# Hands and machines in the works and at the pits

Not in the Python. `crates/isostate-core/src/tasks.rs` and
`data/tasks.toml`, switched by `tasks.enabled`, and on the land by
`tasks.land`: both off under the port's rules, on under the game's. It is
the fifth stage of the run to 2050, in two halves: the works first, then the
land ("On the land", below); its first real machine came with the sixth.
**The works' machine is the electric drive** ("The electric drive", below),
the first machine of the run to 2050's design: made of machinery and
electrical goods and run on electricity, laid with stage 6 of the tree's
overlay (the game's own), and holding nothing in a town without a station.
**With the same content after 1900 two machines of the land's twentieth
century come** ("The tractor and the cutter", below): the tractor hauls on
the farms, the pastures and in the woods, and the cutter at the quarries,
clay pits and mines, where it takes the hoist's place as the drive does in
the works. **The hoist is a stand-in**, made of machinery and burning coal:
without the content after 1900 (`tree.stage = 1`) it hauls in the works and
at the quarries, clay pits and mines, as it did before the drive and the
cutter, and with it it hauls nowhere. The carters are still out of reach.
Nothing can be found before 1901, so
the played century is untouched: every fact of the standing check is the
same to the digit with either switch on and off. Two more settings are for
sweeps and are unset under both rules: a ceiling on what the ground may
yield, and a control that holds the machine still.

## The standing check

**The drive is found from 1901, not 1890.** Its first values had it found
from 1890, once the island had made 15,000 electrical goods, which seven of
the eight islands did before 1900; and the standing check had two of its
twelve facts outside the spread of seeds:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set tree.stage=1,6 --standing --out DIR
```

```text
standing check: 10 of 12 inside (0 moved, 2 OUTSIDE, 0 without a value): tree.stage=6 against tree.stage=1
  urban_share_1900                       0.465  [0.457 .. 0.489]             0.455  OUTSIDE
  income_share_land_1900                 0.165  [0.157 .. 0.176]             0.154  OUTSIDE
```

So its `found_after` is 1901, as the hoist's is, and its `scale` was
refitted on the islands that find it then. With that the same check is
12 of 12 inside, and the stage-6 arm is the island of the tree's stage 6
before the drive, since nothing can be found before 1901.

The checks below were run when the hoist was the one machine, and hold for
the switches as they are:

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
taken cheapest first, each holding what lies above the one before it (so
far each place has one: the drive in the works, the hoist at the pits).

**The works.** A new works means to hold its town's target: new plant is
today's technique. A standing works moves the share it means to hold toward
it by `1 / (12 * adopt_years)` of the gap a month (`adopt_years` 8), times
how well last month's order for its machines was filled, and lets it fall no
faster than its machines wear out. Where the machines it has in place hold
some part of a class (below, "A machine works once it stands"):

- its hands a unit are the port's times the sum over classes of `share *
  (1 - held + minders of the machines held)`;
- the machines' run goods join its inputs a unit, so it buys them each
  month; where they run short its machines idle in that measure and its
  hands do their part, and nothing else is held back ("A machine short of
  its run good", below);
- the machines' plant, sized to its capacity at the share it means to hold,
  is ordered through a table of its own (the works' order for capacity is wiped when no capacity is on
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

## A machine works once it stands

Each works keeps two shares of each class, not one. The share it **means**
to hold (`f_auto`) moves toward its town's target as above, and its order
for plant is sized to it: a works orders the machines it means to have. The
share it **holds** (`f_held`) is what the machines it has in place can do
this month, worked out at the start of the month from its plant in place.
For each part of a class a machine would hold, the machines standing are
the least, over the goods its plant is made of, of the plant in place over
the plant one machine takes (shared out from the cheapest part up); the
machines a part takes are reckoned at the works' capacity and its hands a
unit by hand, as the order is; and a part the standing machines cannot fill
is held only as far as they reach. The share held is never above the share
meant.

The month reads the share held: the cut in hands a unit, the machines' run
goods and their minders, the year's record of what machines did, and the
margin's gap. The gauges were already reading what stands, since the
machines' book is the plant delivered and their wear is taken on it. **A
works founded after a machine is found** means to hold its town's target
and holds nothing until its plant comes: its first months are worked by
hand at its recipe. Its plant is still drawn from the pool with its
capacity when it is founded, so it is paid for and on order from its first
month, and its hands fall as the machines are delivered.

**The land likewise.** Under `tasks.land`, `land_auto` is what a town's
sector means to hold, and `land_held` what its plant in place can do, the
machines reckoned on the sector's work of last month; the mixture of hands
and machine units in the month's hiring reads the share held, and the order
for plant the share meant. The land's bids and a prospective pit's sums
(`land::sector_bids`, `mines::review`) still read the share meant, as a
works' site reads its town's target.

There is **no switch**. This mends a rule that is on only under
`tasks.enabled`, and while no machine has been found both shares are
nothing, so every run to 1900 is as it was to the byte: with
`tasks.enabled=false` islands 1 to 4 give the 840 facts of the binary
before it with none differing; the standing check is 12 of 12 inside with
the rule on and off, and of the 918 facts seed by seed only the rule's own
differ; and the game's golden hashes did not move.

The facts: `auto_haul_works_Y` and `auto_haul_pits_Y` are the share held;
`aim_haul_works_Y` and `aim_haul_pits_Y` the share meant (each weighted by
its work, as the share held is); and `waiting_haul_works_Y` and
`waiting_haul_pits_Y` the difference, the hauling meant for machines that
have not yet come.

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --out DIR
```

As run on 19 November 2026, means over islands 1 to 8, "before" the binary
whose machines worked from the month they were meant:

| | before | now |
|---|---|---|
| the works' hauling done by machines in place, 1910, 1920, 1950, 1980, 2050 | .148, .262, .441, .574, .537 | .143, .260, .439, .570, .535 |
| the works' hauling meant for machines, the same years | | .148, .263, .441, .573, .537 |
| waiting on machines not yet delivered, the same years | | .0051, .0032, .0022, .0026, .0022 |
| the pits' hauling done by machines in place, 1910, 1950, 1980, 2050 | .141, .454, .596, .563 | .136, .451, .591, .560 |
| waiting at the pits, the same years | | .0059, .0028, .0032, .0028 |
| the margin's gap forty years after the find | 1.27 [1.21 .. 1.31] | 1.28 [1.22 .. 1.33] |
| hands in works, 1950, 1980, 2050 | .504, .469, .287 | .510, .473, .288 |
| hands on the land, 1950, 1980, 2050 | .290, .335, .465 | .285, .326, .465 |
| hands in the pits and quarries, 1950, 1980, 2050 | .124, .114, .143 | .124, .117, .141 |
| the towns' share, 1950, 1980, 2050 | .442, .413, .270 | .442, .414, .271 |
| work's share, gross, 1950, 1980, 2020, 2050 | .650, .632, .559, .577 | .654, .628, .549, .578 |
| work's share, net of wear, the same years | .675, .668, .595, .601 | .677, .665, .586, .602 |

**Little waits, because machinery is easy to get.** The hoist is made of
machinery alone, a good the island makes in many works, and its order is
served a quarter of what is missing a month, while wear opens the gap by a
two-hundred-and-fortieth a month: so after the first years some 0.2 to 0.3
per cent of the hauling class waits, about 0.4 per cent of what the
machines do. The first years are different. Measured on the run before
(the same run, with the plant in place reckoned beside it but not used), in
the year of the find half of what the machines were said to do was done by
machines not yet delivered, and in the second year a fifth to a quarter on
every island (0.006 to 0.008 of the class). With this rule those years are
worked by hand. A machine made of goods that are new, or scarce, will wait
far longer, which is why this mending comes before one. The electric drive
is that machine, four tenths of its plant by value in electrical goods,
which few works make: it waits a little longer, not far longer (0.005 of
the works' hauling in 1920 against the hoist's 0.003, 0.004 against 0.002
in 1940, and as little as the hoist from 1970; "The electric drive").

**The margin's gap** forty years after the find is a hundredth wider read
at what stands (1.28 against 1.27): with 0.002 of the class waiting, a gap
read at the share held is `(100 / 0.3)^0.002`, 1.01 times one read at the
share meant. The gap of a quarter is the works trailing a target that rises
with the machine's quality, as "The margin" says below; machines on order
were none of it.

## The electric drive

Motors on the line shafts and the cranes: the first machine of the run to
2050's design, and no longer a stand-in.

| | |
|---|---|
| class | haul, from 0 to 1 of it, in the works |
| plant | 68.5 machinery and 4.92 electrical goods a machine (six tenths and four tenths by value); life 20 years |
| run | 1.01 electricity a month |
| minders | 0.104 of a hand a month (a tenth of its cost when found) |
| quality | grows 3 per cent a year from the year found, to a cap of 8 |
| found | at the first year's turn from 1901 at which the island has made 15,000 electrical goods since 1750 |
| scale | 0.77 |
| replaces | the hoist, in the works |

**Laid only where its goods are.** A use that names a good the content
lacks is not laid, as the tree's overlay drops an input of a later stage. So
the drive is in the table only with `tree.stage` at 6 or more (the game's
own is 6), and with the tree at stage 1 the table is the hoist alone, in the
works and at the pits, and the run is the run before the drive to the byte:

```bash
isostate-core sweep --seeds 1-4 --rules game --families 6 --set tree.stage=1 --out DIR --compare BASE
```

against the same run of the binary before the drive gives "of the 888 facts
both have, the seconds aside, 0 differ". The six facts that changed their
names (below, "The numbers") are not among them.

**It takes the works from the hoist.** A use may say `replaces = "NAME"`:
where it is laid, the use of that name no longer reaches the places this
one is `on`. The drive is `on` the works, so with it the hoist hauls only at
the quarries, clay pits and mines.

**Sizing.** From island 7 in 1894, the year it found the drive when it was
found from 1890, in town 7, the only town on the island with a station
working that year (in every other town electricity stood at its ceiling,
73.3 on the island's mean): a wage of 8.41 a month, machinery 3.25,
electrical goods 30.1 and electricity 2.59. When found a unit costs 1.04 of
a month's wage, of which six tenths is the return a works must see and the
wear on its plant (44 months' wages of plant), a tenth its minders and the
rest its electricity. At that cost a hand is worth 0.96 units, which takes
`ln(0.96 / 0.3) / ln(100 / 0.3)` = 0.20 of hauling. The threshold, 15,000
electrical goods, was set so that the islands as they stood found it
between 1890 and 1899 but for island 4, whose electrical works came about
1920; it is now held to 1901 ("The standing check"), and binds only on
island 4, which finds the drive in 1922.

**The refit** cannot read the towns' target in the year found, as the
hoist's did: on half the islands no town has a station that year (in 1900
islands 2, 3, 4 and 6 have none, and island 8 one), and there the target is
nothing whatever the scale. So it reads what the towns that can run the
drive give it (`use_share_run_electric_drive`): from the first year in which
those towns do three quarters of the works' hauling, the mean of the five
years after, each taken back to a quality of one. At a scale of 1, found
from 1890, that was 0.161 (0.146 to 0.172): settled stations sell
electricity at 3 to 4.5 a unit, dearer than town 7's first; so 0.80. Found
from 1901 it read 0.194 at 0.80, so 0.77. As run now it is 0.200 (0.137 to
0.226), and on the three islands with stations in the year found (1, 5 and
7) the towns' target in that year is 0.226, 0.207 and 0.211.

**No station, no drive.** Electricity goes no further than a town's own
wires. Where a town has no station it stands at its ceiling price, and the
town's target gives the drive nothing. Where a station closes, or its
stations make less than the town wants, the works there still have their
drives, and the drives idle in the measure the electricity is short, as the
next section says. A test holds island 7 for the ten years after the find:
a works with drives in a town with no electricity runs them on none of it,
and the month after hires its hands for all its work by hand.

### A machine short of its run good

**The rule.** A machine whose run good is short idles, in the measure it is
short, and nothing else does. In the month's execute step a works makes what
the scarcest input of its own recipe allows, as the port has it; the
machines' run goods are no longer among those inputs. Where they are filled
`fr` (the least fill of the machines' run goods in the town that month), the
machines do `fr` of their part, the hands do the rest of it at the labour it
took before the machines, and the run goods are paid for what was used, `fr`
of what was asked. The hands are hired before the month's fill is known, so a
works hires for last month's: its hands a unit are the port's times `factor +
share * (1 - last month's fill)`, where `factor` is its hands a unit with
every machine running and `share` the part of its hours by hand its machines
do. Where this month's fill is no worse than last month's it makes all its
recipe allows; where it is worse it makes as much as the hands it hired can,
`(factor + share * (1 - last)) / (factor + share * (1 - fr))` of it. A town
whose stations have all shut runs its works by hand, its drives standing,
from the month after (`Running` in `tasks.rs`, and the execute step of
`econ::step_month`).

**Why.** Before, the drive's electricity was an input of the works' whole
output: a works with drives in a town short of power made what that
electricity allowed, and nothing more. Power stations hold drives (three
tenths of their hours are hauling), and so do the builders whose work is a
station's plant, so a town whose stations fell short of what its works
wanted stopped every works in it, the stations among them, and each month's
shortage made the next one's. The electricity made fell about a hundredfold a
year (island 7, town 6: 37,100 in 1945, 28,700, 5,290, 103 and 0.00004 in
1949), the town's wage fell to a fifth, and the island's chained wage halved
for a decade, the price index pricing the whole island at the stricken
town's prices. The rule that was meant to catch it, `idle`, idled a town's
drives only when the town had no electricity at all in stock, and a fall by
a factor never reaches nothing while a station stands. The trace that found
this is `.claude/night/reports/decline/second-fall.md` (not in the
repository); on the eight islands it counted 26 such collapses.

**Two forms were tried.** The trace's own, the land's rule for the works:
output times `1 - share * (1 - fr)`, the machines' part simply not done. And
the one above, where the hands do it. They measured alike (islands 1 to 16,
1970 and 2050: 9.00 and 9.76 against 8.97 and 9.59 with the idling rule
kept; no collapse in either), and the one above is what this page always
said ("its part is done by hand"), costs a dozen lines more and one number a
works (`f_run_fill`), and does the idling rule's work too, so it is the one
kept.

**The idling rule is gone.** With the soft run `idle` had nothing left to
do. Without it, with either form, nothing moved beyond the islands' own
spread (islands 1 to 16, the trace's form: 1970 9.06 against 9.00, 2050
9.73 against 9.76; no collapse and no town left without work), so it went.
A town with no electricity now holds its drives and runs them on nothing,
its hands doing their part; their plant wears out and is not replaced, the
town's target giving the drive nothing.

**Before 1901 nothing here acts**: no works holds a machine, so every works
is made as the port makes it, to the byte (the played century did move on
the same branch, by the merchants' crumb: prices.md, "A crumb is no glut").

```bash
isostate-core sweep --seeds 1-16 --rules game --families 6 --end-year 2050 --standing --compare MAIN --out DIR
```

As run on 19 November 2026 against main as it was (5bfd60d), with the crumb;
"no drive" is the same branch with the drive left out of the table (a
temporary switch, not in the tree). Means, [least .. greatest] island:

| islands 1 to 8 | main | this rule | no drive |
|---|---|---|---|
| chained wage 1970 (1750 = 1) | 7.88 [4.95 .. 10.04] | 8.83 [7.61 .. 9.45] | 8.83 [7.50 .. 9.49] |
| chained wage 2050 | 7.21 [4.17 .. 10.06] | 9.66 [8.63 .. 10.36] | 9.68 [8.53 .. 10.80] |
| 2050 over 1970, island by island | 0.54 .. 1.25, five below 1 | 1.04 .. 1.19 | |
| towns whose electricity fell a hundredfold with a station standing | 21 | 0 | |
| spells of more than ten years in which a town had no station and no work | 19 | 0 | |
| trades standing still in a town for more than ten years, islands 1 to 16 | 191 | 1 | |

On islands 9 to 16, which nobody had played with the cure: 1970 9.12 and
2050 9.62 (main 8.19 and 6.72), no collapse, no town frozen; island 9 is the
one with 2050 below 1970 (0.93), and it is so with the drive left out too
(its wage tops out about 1990 and falls on every arm). Collapses are counted
with the decline check's `episodes.py` from a town probe; a town without
work is one with no station, builders standing and under a hundredth of its
best year's building work.

### What the drive does, to 2050

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set tree.stage=1,6 --end-year 2050 --out DIR
```

As run on 19 November 2026. "Hoist" is `tree.stage=1`, the hoist in the
works and the pits, the island before the tree's stage 6; "drive" is
`tree.stage=6`, the game's own. Means over islands 1 to 8, [least ..
greatest] where it helps.

| | hoist | drive |
|---|---|---|
| found | 1903 [1901 .. 1907] | 1904: 1901 on seven islands, 1922 on island 4 |
| the works' hauling done by machines in place, 1910, 1920, 1930, 1940 | .143, .260, .331, .388 | .105, .250, .340, .431 |
| the same, 1950, 1960, 1970, 1980 | .439, .488, .538, .570 | .505, .566, .623, .657 |
| the same, 2000, 2020, 2050 | .570, .557, .535 | .669, .659, .647 |
| meant for machines, 1910, 1950, 2000, 2050 | .148, .441, .573, .537 | .111, .508, .671, .649 |
| waiting on machines not delivered, 1910, 1920, 1950, 2000 | .005, .003, .002, .002 | .005, .005, .003, .002 |
| the year half the works' hauling is done by machine | 1963 [1955 .. 1972] | 1949 [1944 .. 1968] |
| the same at the pits (the hoist in both) | 1960 [1953 .. 1967] | 1958 [1953 .. 1963] |
| the margin's gap forty years after the find | 1.28 [1.22 .. 1.33] | 1.40 [1.32 .. 1.53] |
| the margin's gap, 1920, 1950, 1980, 2050 | 1.45, 1.26, 1.12, 1.05 | 1.78, 1.36, 1.13, 1.05 |
| of the electricity made, the share the drives run on, 1910, 1920, 1950, 1980 | | .093, .136, .131, .134 |
| the same, 2000, 2020, 2050 | | .112, .085, .061 |
| electricity's share of the works' heat, 1930, 1950, 2000, 2050 | | .015, .040, .058, .089 |
| the machines' book in years of the wage bill, 1950, 2000, 2050 | .053, .043, .021 | .070, .059, .033 |
| hands in works, 1920, 1950, 1980, 2000, 2050 | .532, .510, .473, .443, .288 | .526, .522, .498, .467, .310 |
| hands on the land, the same years | .265, .285, .325, .355, .465 | .271, .285, .327, .348, .462 |
| hands in the pits and quarries, the same years | .123, .124, .117, .114, .141 | .122, .114, .094, .099, .125 |
| the towns' share, the same years | .460, .442, .414, .397, .271 | .448, .448, .423, .402, .287 |
| work's share of income, gross, 1920, 1950, 1980, 2000, 2020, 2050 | .668, .654, .628, .586, .549, .578 | .651, .645, .622, .583, .559, .575 |
| work's share, net of wear, the same years | .688, .677, .665, .638, .586, .602 | .673, .670, .657, .633, .598, .600 |
| what a wage buys (chained), 1950, 2000, 2050 (1750 = 1) | 7.43, 5.77, 3.45 | 7.66, 5.82, 3.57 |

**Slower at first, then further.** By 1910 the drive holds less of the
works' hauling than the hoist did (0.105 against 0.143): on islands 2 and 6
the stations come only about 1907 to 1910, and the works there hold almost
nothing until then (0.040 and 0.006 in 1910). From 1930 it holds more, and
half the works' hauling is done by machine in 1949 on the mean island
(1944 to 1949 on seven islands, 1968 on island 4), against 1963 with the
hoist: inside the design's band for the island's hauling (half between 1925
and 1950, with the pits; the pits' half still comes with the hoist, in
1958). It settles near two thirds from 1980, where the hoist settled near
0.57. The hoist was a fair test at a poor price: its coal is carried and
burned, where the drive's electricity is made in the town for less each
year as the stations settle.

**The margin's gap is wider.** Forty years on the works trail the town's
target by 1.40 (1.32 to 1.53), against the hoist's 1.28: with stations
coming late and new ones opening dear, the target rises in steps the works
follow at a ninety-sixth of the gap a month. It closes once the quality
stops at its cap: 1.13 in 1980 and 1.05 in 2050, as with the hoist.

**Dearer towns first.** On each island, across the towns whose works haul
in both years, the rank correlation between a town's wage in the year the
drive is found and the share of its works' hauling held ten years on
(`dearer_first_electric_drive`): 0.78, 0.91, 0.53, 0.57, 0.75, 0.83, 0.59 and
0.43, mean 0.67. Positive on every island. Beside it the same with "a
station stood in the town in the year found" in place of the wage
(`local_run_first_electric_drive`): 0.39, 0.53 and 0.39 on islands 3, 5 and
7, and nothing on the others, where in the year found every town or none
had a station. Where there is a choice, the wage says more than the station
about where the drive goes first.

**A tenth of the electricity, and little of the heat.** The drives run on
a tenth to a seventh of the electricity made from 1910 to 1980 (0.13 on the
mean island), falling to 0.06 by 2050 as households buy more of it for fuel
and light. The works' heat hardly takes it: 0.04 in 1950 and 0.09 in 2050.

**Hands.** The works keep more hands than with the hoist (0.522 against
0.510 in 1950, 0.467 against 0.443 in 2000), the pits fewer (0.094 against
0.117 in 1980), the land about the same, and the towns' share is higher from
1950 by about a point. Work's share of income is a little lower to 1980
(gross 0.645 against 0.654 in 1950) and level after; what a wage buys is a
little higher (7.66 against 7.43 in 1950).

**The required return.**

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 1960 --set hurdle=0.09,0.12,0.15 --out DIR
```

Run on 19 November 2026 with the drive: the share of the works' hauling
done by machine in 1950 is 0.543, 0.505 and 0.473 at a hurdle of 0.09, 0.12
and 0.15, and half of it is done by machine in 1944, 1949 and 1955 on the
mean island. `hurdle` is every works' required return, so it changes the
whole played century and with it the year an island finds the drive (island
4 in 1901 at 0.09 and 1908 at 0.15, against 1922 at 0.12; islands 1, 6 and 8
in 1904 to 1911 at 0.15). On the islands that find it in the same year in
both arms the steps are +0.026 from 0.12 to 0.09 (islands 1, 2, 3, 5, 6 and
8: +0.027 to +0.034 but +0.021 and +0.017) and -0.031 from 0.12 to 0.15
(islands 2, 3, 5 and 7: -0.049, -0.005, -0.029 and -0.039), against the
design's step of about 0.017 and the hoist's +0.025 and -0.037. The unit's
cost alone (six tenths of it return and wear, as the hoist's: 0.894 and
1.106 times as much) moves the target by +0.019 and -0.017 of the class; the
rest is the island's history moving with the hurdle, as with the hoist.

## The hoist

The stand-in, made of machinery and burning coal, which proved the rule
before there was anything after 1900 in the content. Where the content
after 1900 is laid (the tree at stage 6, the game's own) it hauls nowhere:
the drive takes the works from it and the cutter the quarries, clay pits,
iron mines and collieries, each with the key `replaces`. With the tree at
stage 1 it hauls in the works and, under `tasks.land`, at the pits, as below. Its entry in `data/tasks.toml` is as it was but for its
comment.

| | |
|---|---|
| class | haul, from 0 to 1 of it, in works and (under `tasks.land`) at the quarries, clay pits, iron mines and collieries, with the tree at stage 1 only |
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

## The tractor and the cutter

Two machines of the land's twentieth century, laid only with the content
after 1900 (`tree.stage` 6): a use whose plant, run goods or threshold name a
good the content lacks is left out of the table, so at `tree.stage = 1` the
table is the hoist's alone (the electric drive is left out too).

| | the tractor | the cutter and conveyor |
|---|---|---|
| class | haul, from 0 to 1 | haul, from 0 to 1 |
| on | farms, pastures, woods | quarries, clay pits, iron mines, collieries |
| plant | 54.2 machinery and 41.7 steel (six tenths and four tenths by value); life 10 years | 87.7 machinery and 11.0 electrical goods (six tenths and four tenths by value); life 15 years |
| run | 0.36 oil a month | 1.10 electricity a month |
| minders | 0.117 of a hand (0.15 of its cost when found) | 0.104 of a hand (a tenth of its cost) |
| quality | 4 per cent a year to a cap of 6 | 3 per cent a year to a cap of 6 |
| found | at the first year's turn from 1915 at which the island has made 300 units of oil | from 1920, at 600,000 units of electrical goods made |
| scale | 0.87 | 1.0 |
| takes the place of | | the hoist, in the pits (`replaces = "hoist"`) |

The tractor is made of machinery and steel because motor vehicles are a later
stage's good; when they come its plant moves to them. It is found on oil
made, not on a date, and the island's oil is made from coal ([tree.md](tree.md),
"Oil, nitrogen and the land"): nobody else buys it, so the first oil works'
first months of output are what finds the tractor, and an island that lights
its towns late (island 1, its first station in 1933) finds oil from coal late
and mechanises its fields late (1951).

**`replaces`.** A use may carry `replaces = "NAME"`: where it is laid (its
goods exist), the use it names no longer reaches the places this one is `on`,
in the works or on a sector, and keeps the rest. The cutter replaces the hoist
on the four pits, and the electric drive in the works; the hoist's own entry
is unchanged. The key is read in
`tasks::table_from`; a name that is no use of the file is an error.

**Sized as the hoist was**, from island 7's wage and prices in the year each
was found there (`cargo run --release -p isostate-core --example land --
--seeds 7 --end-year 1945 --from 1918`, which prints, for each machine of the
land and each year, a unit's return and wear, its run goods and its minders,
each over the wage). Island 7 finds the tractor in 1915 and the cutter in
1923. The tractor was read in 1918, once the first oil works had brought
oil down from its ceiling to 5.6 (wage 10.6, machinery 3.0, steel 2.4): a
unit costs 0.78 of a month's wage, 0.468 of it return and wear on 25.5
months' wages of plant (`(0.12 + 1/10) / 12` a month), 0.117 its minders and
0.195 its oil, so a hand is worth 1.28 units and the tractor takes
`ln(1.28 / 0.3) / ln(100 / 0.3)` = 0.25 of hauling. The cutter in 1923 (wage
10.8, machinery 3.0, electrical goods 14.9, electricity 3.1): 1.04 of a month's
wage, 0.624 of it return and wear on 40 months' wages of plant, 0.104 its
minders and 0.312 its electricity: 0.20 of hauling.

`scale` was refitted once, from the sweep to 2050 at 1.0, in which the
towns' share for the tractor in the year it was found, each town weighted by
the work of the sectors it reaches, was 0.226 on the mean island, where 0.25
was meant: `exp(ln(100 / 0.3) * (0.226 - 0.25))` = 0.87. The cutter's was
0.201 for 0.20 and was left. On the refitted build: the tractor 0.256 (0.211
to 0.302), the cutter 0.200 (0.187 to 0.212). (`use_share_found_NAME` is the
works' target for a use of the works, as before, and the land's for a use of
the land alone.)

### What the eight islands showed

These sweeps were made on the land's own branch, cut from stage 6's first
part (electricity) before the electric drive: in them the hoist still hauls
in the works, and the drive is not there. They were not run again on the tree
with both; "The electric drive" above has the drive's own sweep, without the
tractor and the cutter.

Islands 1 to 8, six families, to 2050:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set tree.stage=1 --out S1
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --out S6
```

Stage 1 is the island without the content after 1900 (the hoist in the works
and the pits, nothing on the fields); "6 before" is stage 6 as it was before
this part (electricity alone, the build it was cut from, at its own
settings); "6" is this part. Means of the eight islands:

| | | 1900 | 1920 | 1930 | 1950 | 1970 | 2000 | 2050 |
|---|---|---|---|---|---|---|---|---|
| chained real wage, 1750 = 1 | 1 | 6.03 | 6.90 | 7.10 | 7.43 | 7.33 | 5.77 | 3.45 |
| | 6 before | 5.87 | 7.02 | 7.56 | 8.00 | 8.02 | 6.67 | 3.97 |
| | 6 | 6.27 | 7.26 | 7.62 | 8.36 | 8.79 | 7.27 | 5.21 |
| hands on the land | 1 | 0.247 | 0.265 | 0.271 | 0.285 | 0.318 | 0.355 | 0.465 |
| | 6 before | 0.249 | 0.263 | 0.269 | 0.276 | 0.303 | 0.360 | 0.439 |
| | 6 | 0.245 | 0.254 | 0.249 | 0.252 | 0.239 | 0.272 | 0.340 |
| hands in the pits | 1 | 0.129 | 0.123 | 0.124 | 0.124 | 0.115 | 0.114 | 0.141 |
| | 6 | 0.124 | 0.124 | 0.127 | 0.121 | 0.122 | 0.114 | 0.125 |
| hands in works | 1 | 0.542 | 0.532 | 0.523 | 0.510 | 0.484 | 0.443 | 0.288 |
| | 6 | 0.547 | 0.543 | 0.544 | 0.548 | 0.559 | 0.530 | 0.443 |
| towns' share | 1 | 0.465 | 0.460 | 0.456 | 0.442 | 0.426 | 0.397 | 0.271 |
| | 6 | 0.465 | 0.466 | 0.469 | 0.477 | 0.483 | 0.471 | 0.391 |
| people | 1 | 673,962 | 814,630 | 889,022 | 1,054,043 | 1,253,099 | 1,659,405 | 2,781,755 |
| | 6 | 663,610 | 799,661 | 870,033 | 1,014,182 | 1,157,004 | 1,367,041 | 1,940,362 |
| farms' hauling by machine | 6 | | 0.126 | 0.177 | 0.309 | 0.472 | 0.569 | 0.567 |
| pits' hauling by machine | 1 | | 0.258 | 0.336 | 0.451 | 0.556 | 0.595 | 0.560 |
| | 6 | | 0.004 | 0.117 | 0.302 | 0.397 | 0.524 | 0.533 |
| works' hauling by machine | 6 | | 0.262 | 0.330 | 0.444 | 0.550 | 0.597 | 0.573 |
| farms' output a hand, 1900 = 1 | 6 | 1 | 0.94 | 1.05 | 1.29 | 1.62 | 1.68 | 1.39 |
| work's share of income (books) | 6 | 0.670 | 0.677 | 0.665 | 0.664 | 0.644 | 0.592 | 0.586 |
| land's share of income (books) | 6 | 0.161 | 0.164 | 0.164 | 0.166 | 0.160 | 0.159 | 0.168 |

The tractor is found in 1930 on the mean island (1915 to 1951) and the cutter
in 1929 (1920 to 1956). The land's machines lift the island: the chained wage
is a tenth higher in 1970 than before them and a third higher in 2050, hands
on the land are fewer from 1930 and a fifth fewer by 1970 (0.239 against
0.303), the towns hold their share to 1970 and lose less after, and there
are 1.94 million people in 2050 for 2.49. **But they do not lift it enough.**
The chained wage grows 0.48 per cent a year from 1900 to 1970 (6.27 to 8.79),
not one, and falls after 1970 on every arm, to 5.21 by 2050; hands on the
land fall by less than a point to 1970 and are 0.340 in 2050, more than in
1900. The design's bands for 1900 to 1970, as a sweep reads them:

| band | the design | this part | |
|---|---|---|---|
| hauling half done by machine, works | 1925 to 1950 | about 1961 (0.495 in 1960, 0.550 in 1970) | fail (the works' machine is the hoist until the electric drive) |
| hauling half done by machine, pits | 1925 to 1950 | about 1993 (0.456 in 1980, 0.524 in 2000) | fail |
| hands on the land | from about 0.27 toward 0.12 by 1975 | 0.245 in 1900, 0.239 in 1970, 0.253 in 1980 | fail |
| the chained real wage | 1 per cent a year or better, 1900 to 1970 | 0.48 per cent | fail |
| work's share of income | 0.58 to 0.72 throughout | 0.644 to 0.677 | pass |
| land's share of income | 0.08 to 0.13 by 1950 | 0.166 | fail |
| the towns' share | must not fall | 0.465 in 1900, 0.483 in 1970 (0.391 in 2050) | pass to 1970 |

The pits are slower than at stage 1: the cutter takes them from the hoist in
1929 where the hoist had them from 1903, and at stage 6 they hold nothing by
machine for those twenty-six years.

### The dial, and what holds the wage

`tasks.dial` multiplies every machine's quality. Nobody had turned it; here
it is at 1, 3, 10, 30 and 100 on the same eight islands to 2050 (`--set
tasks.dial=3,10,30,100`):

| dial | wage 1950 | wage 1970 | wage 2050 | growth 1900 to 1970 | land 1970 | land 2050 | works 1970 | farms' hauling 1970 | farms' output a hand 1970 | people 2050 |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 8.36 | 8.79 | 5.21 | 0.48 % | 0.239 | 0.340 | 0.559 | 0.47 | 1.62 | 1.94 M |
| 3 | 8.76 | 9.03 | 5.51 | 0.52 % | 0.243 | 0.334 | 0.557 | 0.68 | 1.76 | 1.86 M |
| 10 | 9.59 | 9.35 | 5.95 | 0.57 % | 0.274 | 0.330 | 0.545 | 0.90 | 1.99 | 1.79 M |
| 30 | 10.3 | 9.49 | 6.37 | 0.59 % | 0.260 | 0.333 | 0.559 | 0.98 | 2.16 | 1.77 M |
| 100 | 10.9 | 9.57 | 6.58 | 0.60 % | 0.269 | 0.332 | 0.552 | 0.99 | 2.22 | 1.77 M |

(The wage is the chained real wage, 1750 = 1, 6.27 in 1900 on every arm;
land and works are shares of all hands.) The dial does what it says to the
hauling: at 30 the works, the pits and the fields have nearly all of it by
machine by 1970. It lifts the wage of 1950 by nearly a third at a
hundredfold, and then the wage falls from 1960 on every arm. Hands on the
land do not fall at all, and work's share of income (books) is 0.617 to
0.644 in 1970 on every arm, and higher at the higher settings by 2050 (0.634
at a hundredfold for 0.586). Hauling is all that is
in reach on the land: the farm's tending (four tenths of its work) and
judging wait for the combine of a later stage, and a land unit spends the
same share of its revenue on work however cheap its work becomes, so where
the hauling is cheaper the farm takes more work, not fewer hands.

Two cheap levers inside the design's own ranges, each measured against this
part (dial 1):

| | wage 1900 | wage 1970 | wage 2050 | growth 1900 to 1970 | land 1900 | land 1970 | land 2050 | people 2050 |
|---|---|---|---|---|---|---|---|---|
| this part | 6.27 | 8.79 | 5.21 | 0.48 % | 0.245 | 0.239 | 0.340 | 1.94 M |
| `machines.quality_cap` 12 (the pass over the rules' stage 8) | 7.41 | 12.7 | 11.2 | 0.77 % | 0.222 | 0.206 | 0.296 | 0.94 M |
| `transition.z_half` 4.0 (the design's first value; the game has 4.5) | 5.91 | 9.35 | 7.01 | 0.66 % | 0.245 | 0.228 | 0.294 | 1.30 M |
| both | 6.88 | 13.0 | 12.0 | 0.91 % | 0.226 | 0.197 | 0.259 | 0.84 M |

The machines' ceiling of 4 on farm machinery, pumps and the quarries' engines
is what holds the wage most: lifted to 12, the farms' output a hand is 2.7
times 1900's by 1970 and the wage grows three fifths faster. But it lifts the
played century too (the wage of 1900 is 7.41, outside the spread of the
island as it stands, 5.10 to 6.94), so it is the pass over the rules' change
to make, not this part's. Fewer births lift the wage after 1970 most of all
(7.01 in 2050 for 5.21) and bring people into the design's 1.0 to 1.5
million; it lowers 1900's people by 5 per cent, which is why the game's line
was set at 4.5. Neither alone gives 1 per cent a year; both together give
0.91 per cent, with people stalling at 0.84 million, below the design's
peak. On every arm the wage of 2050 is below 1970's and the land holds more
of the hands in 2050 than in 1900: what the land can be given in this part
stops improving by the 1970s (the tractor's quality reaches its cap of 6 in
the 1970s, the cutter's about 1990, nitrogen and breeding are fixed), people
go on growing, and the farm's tending waits for the combine.

## What a sweep showed, with the hoist in the works

What the hoist did in the works, before the drive; with the tree laid to
stage 1 the island is still this one. (The facts named by the machine were
`hoist_found_year` and the like then; they are `use_found_year_hoist`,
`dearer_first_hoist` and so on now, "The numbers".)

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set tasks.enabled=false,true --out DIR
```

The "rule off" column as run on 19 November 2026 on the owners' branch,
with the long run's anchors in (a run with the switch off has not moved
since, to the digit). The "rule on" column, before the land was in reach,
is the run with `tasks.land=false`, run again the same day once machines
worked only when they stood
(`--set tasks.land=false,true`, the first arm). Means over islands 1 to 8,
[least .. greatest] where it helps.

| | rule off | rule on |
|---|---|---|
| the hoist found | | 1903 [1901 .. 1907] |
| the towns' target in the year found | | 0.200 [0.172 .. 0.221] |
| share of the works' hauling done by machine, 1910, 1920, 1930, 1940 | | 0.143, 0.254, 0.320, 0.374 |
| the same, 1950, 1960, 1970, 1980 | | 0.424, 0.471, 0.517, 0.544 |
| the same, 2000, 2020, 2050 | | 0.543, 0.535, 0.514 |
| the year half of it is done by machine | | 1967 [1959 .. 1976] |
| the margin's gap, 1920, 1950, 1980, 2000, 2050 | | 1.41, 1.25, 1.12, 1.09, 1.05 |
| the margin's gap forty years after the find | | 1.26 [1.21 .. 1.30] |
| the machines' book in years of the wage bill, 1950, 2000, 2050 | | 0.049, 0.040, 0.020 |
| hands on the land, 1920, 1950, 1980 | 0.263, 0.271, 0.308 | 0.265, 0.287, 0.339 |
| the same, 2000, 2020, 2050 | 0.345, 0.360, 0.453 | 0.352, 0.383, 0.471 |
| hands in works, 1950, 2000, 2050 | 0.519, 0.455, 0.308 | 0.496, 0.436, 0.275 |
| hands in the diggings, 1950, 2000, 2050 | 0.131, 0.121, 0.140 | 0.139, 0.128, 0.152 |
| the towns' share, 1950, 2000, 2050 | 0.448, 0.396, 0.287 | 0.433, 0.383, 0.261 |
| work's share of income, gross, 1950, 2000, 2050 | 0.659, 0.609, 0.572 | 0.658, 0.593, 0.575 |
| work's share, net of wear, 1950, 2000, 2050 | 0.680, 0.657, 0.596 | 0.681, 0.648, 0.599 |

**Dearer towns first.** On each island, across the towns whose works haul
in both years, the rank correlation between a town's wage in the year the
hoist is found and the share of its works' hauling held ten years on
(`dearer_first_hoist`): 0.63, 0.67, 0.26, 0.32, 0.76, 0.60, 0.66 and 0.68,
mean 0.57. Positive on every island.

**The margin.** The wage over the machine's cost at the last task each
works holds is 1 when the works stand at their town's target. It falls from
1.94 in 1910 to 1.25 by 1950 and is **not within a tenth of one forty years
after the find** (1.21 to 1.30 on every island). The works trail a target
that rises with the hoist's quality, 3 per cent a year, by about
`adopt_years` years of its rise (some 0.04 of the class, which is a factor
of `(100 / 0.3)^0.04` = 1.26 in the gap), and part of each month's step
waits on the machine order's fill. Once the quality stops at its cap, about
1973, the gap closes: 1.12 in 1980, 1.09 in 2000, 1.05 in 2050. The rule
comes to rest near where it should; at these settings it cannot keep up
with a machine that is still improving. (Machines on order are not part of
the quarter: see "A machine works once it stands".)

**Hands go back to the fields.** With the works mechanising and the land
not, the share of hands on the land is higher with the rule at every mark
(0.287 against 0.271 in 1950, 0.471 against 0.453 in 2050), the works'
share lower and the towns' share lower by 0.013 to 0.026. The second half
of the stage, the machines on the land, exists to undo this.

**The required return.**

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 1960 --set hurdle=0.09,0.12,0.15 --out DIR
```

Run again on 19 November 2026 with the land in reach and machines working
only once they stand: the share of the works' hauling done by machine in
1950 is 0.449, 0.439 and 0.383 at a hurdle of 0.09, 0.12 and 0.15.
`hurdle` is every works' required return, so it changes the whole played
century and with it the year each island finds the hoist (1913 on island 1
at 0.09, 1919 on island 3 at 0.15). On the islands that find it in the same
year in both arms the steps are +0.025 from 0.12 to 0.09 (islands 2, 5 and
8) and -0.037 from 0.12 to 0.15 (islands 5 and 6), where the unit's cost
alone (0.894 and 1.106 times as much) moves the target by +0.019 and -0.017
of the class. The uneven step is not machines waiting: 0.002 of the class
waits in 1950 in every arm, and the binary before this mending gave +0.021
and -0.036 on the same islands. (The first half, with the land out of
reach, measured +0.016 and -0.036.)

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
  the share it means its machines to hold (`land_auto`) moves toward it by
  the same rule as a works' share: a ninety-sixth of the gap a month, times
  the fill of the sector's last order for plant, and down no faster than
  wear. It starts from nothing in the year the machine is found. What the
  plant in place can do (`land_held`) is what the month's hiring reads ("A
  machine works once it stands").
- **The cost of an hour's work.** Where machines hold part of a sector's
  work, an hour of it costs `cL = hands * w + cost`: `hands` is the work left
  to hands, `sum over classes of share * (1 - held)`, and the minders of the
  machine units held; `cost` is those units' return and wear on their plant,
  `(hurdle + 1/life) / 12` of its worth, and their run goods, at the prices
  the unit expects. The unit hires `lam * (Cc / cL)^(1/alpha)` of work, which
  takes `hands` hands an hour, and makes its crop from the work. The wage
  search keeps its shape (one sum a town and sector, read at `cL` for `w`).
  The land's bids (`land::sector_bids`) and a prospective pit head's sums
  (`mines::review`) read `cL` at the share the sector means to hold.
- **The machines' goods.** Each sector asks each month for its machines'
  run goods with the day's trade, and for a quarter of what its plant lacks
  for the share it means to hold, beside the works' plant orders; each unit pays for them by its part of the
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

Run again on 19 November 2026 once machines worked only when they stood.
Means over islands 1 to 8; "none" is the first half's sweep with
`tasks.enabled=false` (a run with the switch off has not moved since),
"works" is `tasks.land=false`, "land" is `tasks.land=true`.

| | none | works | land |
|---|---|---|---|
| hands on the land (farms, pastures, woods), 1950, 1980, 2000, 2050 | .271, .308, .345, .453 | .287, .339, .352, .471 | .285, .326, .355, .465 |
| hands in the pits and quarries, 1950, 1980, 2000, 2050 | .131, .129, .121, .140 | .139, .134, .128, .152 | .124, .117, .114, .141 |
| hands in works, 1950, 1980, 2000, 2050 | .519, .485, .455, .308 | .496, .446, .436, .275 | .510, .473, .444, .288 |
| the towns' share, 1950, 1980, 2000, 2050 | .448, .424, .396, .287 | .433, .405, .383, .261 | .442, .414, .397, .271 |
| the works' hauling done by machine, 1950, 1980, 2050 | | .424, .544, .514 | .439, .570, .535 |
| the pits' hauling done by machine, 1920, 1950, 1980, 2050 | | | .258, .451, .591, .560 |
| work's share, gross, 1950, 1980, 2000, 2050 | .659, .652, .609, .572 | .658, .624, .593, .575 | .654, .628, .586, .578 |
| work's share, net of wear, 1950, 1980, 2000, 2050 | .680, .680, .657, .596 | .681, .660, .648, .599 | .677, .665, .638, .602 |
| what a wage buys (chained), 1950, 2000, 2050 (1750 = 1) | 6.88, 5.84, 3.29 | 7.21, 5.57, 3.24 | 7.43, 5.77, 3.45 |
| the land machines' return over the land's rent, 1950, 2000, 2050 | | | .026, .028, .021 |

**The pits let hands go; the fields take some of them.** At the pits the
hoist takes hauling as fast as in the works (0.451 of it in 1950, 0.591 in
1980), and the share of hands in the pits falls below even the island with
no machine at all from 1950 to 2000. The fields are out of reach, and the
share of hands on them is much as with the works alone: a little lower in
1950, 1980 and 2050, a little higher in 2000. Counted together, of the
hands the first half sent to the land and the pits this half takes back
three quarters in 1950 (0.017 of 0.023), five sixths in 1980 and a half in
2050; it gives the towns back three fifths of the share the first half took
in 1950 and all of it in 2000. The towns' share is higher with the land on
than without at every mark shown, and below the island with no machines
but in 2000, where it is level.

**Work's share hardly moves.** Gross and net, it is within a point of the
works-only run at every mark (gross 0.586 against 0.593 in 2000; net
0.638 against 0.648). With one machine that hauls, a cap on its quality of
8 and the fields out of reach, nothing in this stage could move it much;
the stage is a test of the plumbing, and of the direction hands go. **What
a wage buys** is higher with the land on at every mark (7.43 against 7.21
in 1950, 3.45 against 3.24 in 2050; not traced).

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

Run again on 19 November 2026 once machines worked only when they stood:

| | unset | 2 |
|---|---|---|
| the land's output at the ceiling, by value, 1960, 2000, 2020, 2050 | | .0002, .009, .044, .143 |
| the land's share of income, 1960, 2000, 2020, 2050 | .173, .163, .158, .183 | .174, .160, .174, .307 |
| the wage over the price of the land's goods (the gauges' fork), 1960, 2000, 2020, 2050 | 6.68, 5.61, 4.72, 3.84 | 6.68, 5.59, 4.45, 2.61 |
| work's share, gross, 2000, 2020, 2050 | .586, .549, .578 | .595, .537, .431 |
| hands on the land, 2000, 2050 | .355, .465 | .351, .345 |
| what a wage buys (chained), 2000, 2050 | 5.77, 3.45 | 5.74, 2.55 |

The island's yields grow slowly after 1950 (no machine reaches the fields,
and nothing new makes ground yield more), so twice the best year before
1950 is far off for decades: a
five-thousandth of the land's output stands at it in 1960 and a hundredth in
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

Run again on 19 November 2026 once machines worked only when they stood:

| | growing | held from 1950 |
|---|---|---|
| the works' hauling done by machine, 1950, 1960, 1970, 1980 | .439, .488, .539, .570 | .439, .462, .460, .451 |
| the pits' hauling done by machine, 1950, 1980 | .451, .591 | .451, .472 |
| work's share, net of wear, 1950, 1960, 1970, 1980 | .677, .675, .672, .665 | .677, .674, .670, .665 |
| work's share, gross, 1950, 1980 | .654, .628 | .654, .631 |
| investment's share of spending, 1950, 1960, 1970, 1980 | .041, .043, .050, .061 | .041, .042, .042, .056 |

From 1950 to 1980 the machines' share of hauling rises by 0.13 in the works
and 0.14 at the pits in one arm and slips a little in the other, and work's
share net of wear falls by 0.012 in both. The frozen arm's share falls as
far: whatever lowered it from 1950 was not the machine. Investment's share
of spending rose by two points in the growing arm and one and a half in the
frozen one, which on the long run's own measure (a point of investment
takes about a point off work's measured share) is about the size of the
fall; so it was investment, and the island going on, not the margin.

## The numbers

`TasksParams`, in `tasks.rs`: `enabled`, false by default and true under
`game()`; `land`, likewise; `adopt_years`, 8; `dial`, 1, which multiplies
every machine's quality; `yield_cap` and `freeze_from`, unset under both
(for sweeps). A sandbox's Rules tab offers the dial among its most asked, in
multiples from a tenth to a hundredfold (`rulebook::WIDE_OFFERS`), and the
Markets panel's Work half draws the part of each class machines do. `land`, `yield_cap` and `freeze_from` are left out of a run's
`params.json` while off or unset, so a run with them so writes the first
half's file. The classes, the spreads, the shares and the machines are content,
in `data/tasks.toml`, and only the game's content has them
(`content::game_content`, laid again by name when the tree's overlay
rebuilds the content). A machine (`[[use]]`) naming a good the content lacks
is not laid; `replaces = "NAME"` takes the places it is `on` from the use
named (a name that is no other use of the file is an error).

The facts, at the sweep's marks and in 1890: `auto_haul_works_Y` (the share of the
works' hauling done by machines in place, each class weighted by its work
in each works), `aim_haul_works_Y` (the share meant) and
`waiting_haul_works_Y` (meant less done), `hours_CLASS_Y` for the five classes (hands at their tasks in works,
worker-months a month) and `hours_minders_Y`, `waterline_haul_Y` (the wage
over the cheapest unit's cost, work-weighted), `margin_gap_haul_Y`,
`machines_years_Y` (their book over the year's wage bill);
`haul_half_year_works` (the first year the works' hauling done by machines
in place passes a half); `GOOD_to_machines_Y` for each good a machine runs
on, in 1890 and from 1900 (`electricity_to_machines_Y`,
`coal_to_machines_Y`: of what the island made of it in the year, the share
the machines for tasks ran on, in the works and on the land). For each
machine, by its name: `use_found_year_NAME`, `use_share_found_NAME` (the
towns' target in the year found), `use_share_run_NAME` and
`use_share_run_year_NAME` (what the towns that can run it give it, as the
drive's refit reads it, and the year that reading starts from); and, for a
machine in the works, `use_auto_after10_NAME`,
`use_margin_gap_after40_NAME`, `dearer_first_NAME` and
`local_run_first_NAME` (the same rank correlation with "a works making each
of its run goods stood in the town in the year found" in place of the
wage). Until the drive these were `NAME_found_year`, `NAME_half_year` (now
`haul_half_year_works`, by class), `NAME_share_found`, `NAME_auto_after10`,
`NAME_margin_gap_after40` and `NAME_dearer_first`.
Under `tasks.land`: `auto_haul_pits_Y` (the share of the quarries', clay
pits' and mines' hauling done by machines in place, each weighted by its
work), `aim_haul_pits_Y`, `waiting_haul_pits_Y`, `haul_half_year_pits`,
`land_rent_machines_Y` and `income_share_ground_Y` (above); while
`yield_cap` is set, `yield_capped_share_Y` (the value of the land's planned
output that stands at the ceiling, over all of it); and `auto_haul_farm_Y`,
`auto_haul_pasture_Y` and `auto_haul_forestry_Y` (each sector's hauling done
by machines in place, over its hauling). With oil in the content, from 1900:
`nitrogen_share_Y`, `breeding_share_Y` and `grain_a_hand_Y`
([machines.md](machines.md), "Nitrogen and breeding"), `oil_made_Y` and
`oil_over_coal_heat_Y` ([tree.md](tree.md), "Oil, nitrogen and the land").
The shares of hands on
the land, in the pits and quarries and in the works are the sweep's own
`farm_share_Y`, `diggings_share_Y` and `industry_share_Y`.

`cargo run --release -p isostate-core --example tasks -- --seeds 1-8
--end-year 2050` prints a line a year from 1895 for each island: the
works' machine's quality (the drive's, or the hoist's with the tree at
stage 1), the share of hauling held in works and at the pits, the
share of each meant, the works' target, the waterline,
the gap, the book, the hands on the land and in works, the towns' share,
the wage, the prices of machinery and coal, and the share of the
electricity made that machines ran on. `--json '{"tree": {"stage": 1}}'`
plays the island with the hoist in the works. `cargo run --release -p
isostate-core --example land -- --seeds 7 --end-year 1960` prints, a line a
year, each machine of the land's cost over the wage in its parts and its
share, nitrogen's and breeding's shares, the farms' output a hand, the
prices of oil, coal, electricity and the rest where they are sold, and the
oil and nitrogen works standing.

## What worries me

- **The stations come late on half the islands.** The drive is found in
  1901 on seven islands, but on islands 2 and 6 the first stations come only
  about 1907 to 1910, and island 4 makes electrical goods so late that it
  finds the drive in 1922. A gate on electrical goods alone cannot put the
  find after the first station on every island; the drive simply holds
  nothing until one stands, and the refit reads the towns that can run it.
- **The works and the land idle their machines in two ways.** A works'
  hands do its idle machines' part ("A machine short of its run good"); a
  land unit short of its machines' run goods (the tractor's oil, the
  cutter's electricity) loses that part of its crop and hires no more hands
  for it (`Y *= (1 - machine_work * (1 - fr))^(1 - alpha)`). Since the
  idling rule went, a town with no electricity at all holds its cutters too,
  and they do nothing. The two should be one rule; the land's is the next
  piece of the run to 2050.
- **A works with idle drives still reckons their power at its price.** Its
  margin counts the machines' run goods a unit at the town's price, short or
  not, so a works in a town without a station reckons its drives' electricity
  at the ceiling price until they wear out. Measured, it moved nothing (the
  idling rule's removal), but it leans against the works there.
- **The hoist is a stand-in.** It is made of the island's own machinery and
  burns its own coal, and with the content after 1900 it hauls nowhere. With
  the hoist in the works, machinery was a tenth cheaper with the rule (3.19
  against 3.53 in 1940), perhaps because its orders took the makers further
  down their learning curves (not traced).
- **The drive and the land's machines were measured apart.** The drive's
  sweeps had no tractor or cutter, and the land's had the hoist in the
  works; the island with both has not been swept against the design's bands.
- **The fields are in reach, and the island still grows poorer after
  1970.** With the tractor, nitrogen and breeding the wage grows half as fast
  as the design asks from 1900 to 1970 and falls after it, and hands go back
  to the fields after 1980 ("What the eight islands showed", above).
- **Oil waits on the tractor.** Nobody buys oil made from coal until a
  tractor runs on it, so the tractor's threshold is the first oil works'
  first months: a threshold much above 300 units leaves most islands with
  no tractor at all.
- **The pits go without a machine from the hoist's year to the cutter's**
  under the content after 1900, which the design did not mean: its cutter is
  aimed at 1925 and the hoist was never in its pits.
- **The land's machines are bought as inputs.** A sector's plant is bought
  out of its revenue and counted with its inputs, as the port's farm
  machinery is, not as investment; the books balance, but the gauges' plant
  behind a year's income does not count it. Its return is reckoned at the
  month's prices on the plant in place, not on what it cost.
- **A town founded after the find starts its land from nothing**, where
  its works mean to hold the target from the start.
- **The ceiling's best years are of a unit's planned crop** (an average
  year's), by sector and cell, per share of the cell the use held; how that
  behaves on a cell whose use changed a great deal before 1950 is not looked
  at.
- **The machines stop improving at a quality of 8**, about 1973, and the
  wage falls after 2000 on an island with little new to make, so the works'
  share of hauling stalls near two thirds with the drive (0.57 with the
  hoist) and eases back a little by 2050.
- **What stands is reckoned at full capacity.** A works' machines are
  sized to its capacity, not to what it makes this month, so a works whose
  capacity grows holds a smaller share until more plant comes, and one
  working below capacity holds no more for it. A sector's are reckoned on
  last month's work, so its share held moves a little with the seasons.
- **One machine for each class in each place is all the code has been tried
  with.** There are four for hauling now, but each in places of its own
  (the drive the works, the tractor the fields, the cutter the pits, and the
  hoist all of them only without the content after 1900). With
  several in one place, the town's uses are taken cheapest first, and a works holding
  more than its town would has the dearest of them hold the rest. Where two
  machines are made of the same goods the plant in place is given to the
  cheapest part first, and a part not filled leaves every part above it in
  that class unheld (the share is a top, not a set of tasks).
- The plant a unit of capacity takes is reckoned at the sites' mean hands a
  unit in `evaluate_sites`, so every site of one method in one town is
  charged the same for its machines.
