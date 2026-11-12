# Machines

Machines that are bought where they pay, that keep getting better, and steel
that matters. None of it is in the Python. Off under the port
(`machines.enabled = false`), on under the game's rules; the code is
`crates/isostate-core/src/machines.rs`, the content changes are
`machines_content` in `content.rs`, and the numbers below come from
`examples/machines.rs`.

## What was wrong

The port gives the land three kinds of machine: pumps and wagonways in the
mines from the start, and farm machinery late in the century. Each is bought
per unit of output whatever it costs, each adds a fixed amount for ever, and
the quarries and clay pits, a fifth of all hands, get none. Three things
followed (`dx-growth.md`, `dx-cycles.md` section 7):

- **History stopped about 1800.** Husbandry had spread by 1790 and tools were
  near their ceiling by 1840. After that nothing raised what a worker on the
  land produces, and three quarters of labour works the land. Output per
  worker fell in the quarries and collieries all century as five times the
  people worked the same ground.
- **Farm machinery arrived as a famine.** The year a town learned of it, every
  farm there bought it in full at any price. Machinery's price rose up to
  thirtyfold and stayed high for twenty years, the pumps went short, and the
  real wage fell by up to a quarter.
- **The converter was found and never built.** Its plant cost five times what
  its steel could carry, it started from nothing while crucible steel stood at
  the top of its learning curve, and a steel rail sold as an iron one.

## What it does

**1. A machine is bought where it pays.** For a farm, the machine's gain on a
unit of grain is the grain's price times the share of that grain the machine
is owed, `boost / (1 + all boosts)`; its cost is the machinery that unit
takes. Both are reckoned at the prices the town's farms hire on, the prices
they expect for the month (`pexp`), not the month's market price; when the
year's land is let the same sum is done on the year's average prices. Farms
differ (the size of a field, the
lie of the land), so they do not all cross from "does not pay" to "pays" at
once: none take the machine where it only earns its cost, all of those who
know how take it where it earns its cost `1 + adopt_span` times over, and a
share in proportion between. A farm that takes the machine buys for its own
output, which is `1 + boost` times that of a farm without one. So a town's
farms ask for the machinery that fully equipped farms need, for the share of
farms that buy: those who know how, times the share of them it pays. When
machinery is dear fewer farms buy, and the price stops where the market
clears instead of running to the ceiling.

**2. A machine stands for years.** Each town keeps, for each use, the share of
its farms (or pits, each counted by what it makes with no machine) whose
machines are standing. At the end of every month it moves one part in
`life_months` towards the share that keeps its machines up (at the prices then
expected for the next month) and was supplied that month (the share of the
town's demand for machinery, or for rails, that its market filled). What a
machine adds comes from the machines standing; what is bought comes from who
keeps theirs up. Counted in machines the two are one account: if an equipped
farm's machines are `input * life_months` for each unit it grows a month,
then replacing what wears out and adding what is new comes to `input` a unit
for every farm that keeps them up, which is what point 1 buys. A month the
engine works cannot deliver does not flood the pits; years of it do. Settlers bring their engines with them: a new town
starts from what its people keep up. This reaches the mines too: their pumps
and wagonways are bought as the port buys them, but what they add now follows
eight years of deliveries, not last month's.

**3. Machines get better with the steel poured.** Quality is
`(1 + Q / quality_Q0) ^ quality_b`, where Q is all the steel the island has
made, by any method; it stops at `quality_cap`. This is the law the methods'
own learning curves follow, applied to the engine builders: no dates, no
jumps. Quality multiplies what pumps, wagonways and farm machines add.

**4. Better machines reach the quarries and clay pits.** A machine of quality
q adds `quarry_reach * (q - 1)` to a quarry's or a pit's output, for
`quarry_input` of machinery per unit, where the town knows the pump (the steam
engine is what a crane and a pug mill run on). The engines of 1750 add nothing
there; by 1850 they are worth having in most quarries. Bought by the rule of
point 1.

**5. A new method starts from what the trade knows.** A learned method (one
that starts clumsy and climbs a curve) of making a good for which an earlier
learned method was found before it (in the same year, the one the content
lists first) has `inherit` of that method's skill, if that is more than its
own (and never more than the whole of it). The rule asks only that the earlier
method was found, not that anyone still works it. On this island that is the
converter after crucible steel and the steel rail mill after the rolling mill.
It is a floor, not a gift, and it is worked out again every time the methods'
skills are (`firms::update_productivity`), from the earlier method's skill as
it then stands: once the new method's own curve passes it, the method is on
its own.

**6. Content** (`machines_content`, applied in `game_content`):

- The converter's and the steel rail mill's plant cost `MACHINES_PLANT_SCALE`
  (0.3) of the port's. The port's converter needs 150 bricks and 2 machinery
  for each unit of monthly capacity: 410 at the content's base prices, which
  is 1.6 times a rolling mill (100 bricks and 1 machinery, 250) and nearly
  seven times a crucible steel works (30 bricks and 15 clay, 60). This page
  used to say about a thousand, five times a rolling mill. The thousand must
  be at the dearer prices of the first helper's runs (it was not recorded
  which); at the hurdle rate (12 per cent a year, on 85 per cent of capacity)
  it is 12 a unit of steel in capital alone where crucible steel carries 2,
  which is the converter against the crucible: against the rolling mill it is
  never more than twice, at any prices. Cut to
  three tenths, the converter's plant is 123 at base prices and the steel rail
  mill's 96 (from 321); the engine works' (466), the rolling mill's and the
  coke furnace's (147) are all dearer.
- A steel rail outlasts iron ones several times over. There is one good,
  "rails", counted in iron rails, so a ton of steel rail is entered as the
  three iron rails it stands in for (`MACHINES_STEEL_RAIL_WORTH`): the mill
  takes a third of the steel per rail.
- Discovery thresholds (`MACHINES_THRESHOLDS`): converter at 2.8 million of
  steel and 90,000 of machinery (the port's had 30,000 and 170,000); steel
  rail mill at 3.3 million of steel, 350,000 of rails and 150,000 of
  machinery (2.5 million, 400,000 and 300,000). The converter's third
  threshold, 220 years of the iron the island made a year in its warm-up, is
  the port's and is left as it is; discovery waits on the least advanced of
  the three (`tech::discovery`).

The mines' pumps and wagonways are still bought as the port buys them, on
purpose (see "Decisions").

To run the game's rules without the content changes, give the sweep or a run
the port's content file: `--rules game --content data/content.toml`
(`--content port` in `examples/machines.rs`).

## The numbers

| `machines.` | value | |
|---|---|---|
| `enabled` | false; true in the game | |
| `adopt_span` | 6 | a machine is taken by all only where it earns its cost seven times over |
| `adopt_pumps`, `adopt_wagonways` | false | the mines buy as the port does |
| `life_months` | 96 | a machine stands eight years |
| `quality_Q0`, `quality_b`, `quality_cap` | 1,000,000; 0.6; 4 | quality 2 at 2.2 million of steel, 4 at 9 million |
| `quarry_reach` | 1 | a quarry's machine adds q - 1 |
| `quarry_input` | 0.001 | machinery per unit of stone or clay (a farm's and a pump's is 0.0004) |
| `inherit` | 0.5 | half the predecessor's skill |

## What the sweeps showed

Sixteen seeds to 1900, mean and range over seeds, from
`cargo run --release -p isostate-core --example machines -- --seeds 1-16 --rules port|game [--families 6]`.
"Game" in this section and under "Decisions" is the game's rules as they were
then, the port with the machines and their content and nothing else; these
numbers are the record of how the machines were built and cannot be read
from today's sweep, which is under "In today's game" at the end.
Real wages are five-year means (1800 is 1798 to 1802, 1900 is 1896 to 1900)
over the mean of 1750 to 1752, on the model's own index (the 1750 basket,
what the sweep's `real_wage` is) and on a chained index of what was actually
bought, which the example links year by year itself.

| | port | game | port, six families | game, six families |
|---|---|---|---|---|
| real wage 1800 | 1.32 [1.22 .. 1.43] | 1.32 [1.24 .. 1.38] | 1.33 [1.07 .. 1.43] | 1.34 [1.28 .. 1.43] |
| real wage 1850 | 1.31 [1.15 .. 1.57] | 1.53 [1.40 .. 1.72] | 1.34 [1.14 .. 1.60] | 1.66 [1.46 .. 1.90] |
| real wage 1900 | 1.31 [1.17 .. 1.42] | 2.08 [1.89 .. 2.21] | 1.34 [1.22 .. 1.41] | 2.16 [1.92 .. 2.32] |
| lowest five-year mean, 1855-85 | 1.09 [0.86 .. 1.40] | 1.57 [1.40 .. 1.82] | 1.27 [1.10 .. 1.45] | 1.74 [1.42 .. 2.02] |
| 1890s over 1850 | 1.01 [0.89 .. 1.19] | 1.36 [1.28 .. 1.41] | 1.01 [0.87 .. 1.21] | 1.31 [1.17 .. 1.45] |
| the same, chained index | 1.10 [0.95 .. 1.31] | 1.54 [1.41 .. 1.61] | 1.12 [0.93 .. 1.38] | 1.51 [1.38 .. 1.68] |
| machinery's peak price after 1850, over its usual price (median of 1836-50) | 14.9 [0.9 .. 31.3] | 2.5 [1.3 .. 4.2] | 8.4 [0.9 .. 18.7] | 2.3 [0.8 .. 5.2] |
| years after 1850 with machinery above three times usual | 14 [0 .. 26] | 1 [0 .. 4] | 9 [0 .. 19] | 2 [0 .. 13] |
| farm output per worker-month 1800 / 1850 / 1900 | 13.1 / 14.2 / 17.9 | 13.1 / 15.4 / 37.8 | 13.2 / 16.2 / 18.7 | 13.6 / 19.8 / 40.4 |
| quarry | 7.6 / 6.9 / 6.1 | 7.7 / 10.6 / 17.9 | 7.7 / 6.9 / 6.1 | 7.7 / 11.2 / 18.7 |
| colliery | 19.8 / 16.3 / 13.5 | 21.7 / 28.2 / 41.8 | 19.5 / 15.8 / 13.7 | 22.5 / 30.0 / 45.6 |
| converter first built | 1892 in 6 of 16 | 1855 [1824 .. 1881] | 1873 in 4 of 16 | 1850 [1839 .. 1867] |
| converter's share of steel, 1880 | 0 | 0.70 [0.00 .. 1.00] | 0.01 | 0.97 [0.55 .. 1.00] |
| steel rail mill first built | 1882 in 3 of 16 | 1862 [1842 .. 1882] | 1893 in 1 of 16 | 1856 [1815 .. 1874] |
| steel rails' share of rails, 1890 | 0 | 0.86 [0.20 .. 1.00] | 0 | 0.96 [0.69 .. 1.00] |

No run failed the monthly money check. The port's columns are the first
helper's; the port has not moved since (the golden test, and seed 3's files
against the night's baseline), so they were not run again. The game's columns
are the review's, after the two changes described under "Decisions".

In the sweep's own scorecard (`--rules game` against the port, seeds 1 to 8
and again 9 to 16, which nothing was tuned on) the real wage is 1.32, 1.58
and 2.10 times its 1750 level in 1800, 1850 and 1900 on the first eight seeds
and 1.32, 1.51 and 2.07 on the second eight, against 1.33, 1.34, 1.29 and
1.34, 1.27, 1.32 for the port. The facts that move clear of the spread
between seeds in both sets are the 1900 real wage and output per head (2.60
against 1.46), the urban and industry shares of 1900 (0.27 and 0.29 against
0.24 and 0.25), steel rails' share, half the alarms in the news, a sixth
less woodland with more pasture and wild land, and the median life of a works
that closed (15 years against 12). Output per head now peaks in 1896 on
average instead of 1867. The first railway (1830 against 1828), the railway's
length and the 1800 wage do not move outside the spread. Farm machinery is
found about ten years sooner (1832 against 1843): its threshold is in
machinery made, and the quarries' buying adds to that.

One fact moves the wrong way. The works are smaller at the top: the ninth
tenth of works is 2.3 times its method's usual size against 2.9 (every seed
of sixteen), and 1.6 against 2.5 with six families. Not traced. There are
twice as many engine works in 1900 (1,140 against 570, and larger: 3.6 units
each against 2.3), so the count is of a different mix of trades; whether the
big works of other trades are also fewer is for the works mechanism's sweep
to say once the two are together.

What each part does alone (eight seeds, model's index):

| | lowest 1855-85 | 1890s over 1850 | machinery peak over usual |
|---|---|---|---|
| port | 1.10 [0.86 .. 1.40] | 0.97 [0.89 .. 1.07] | 11.3 [0.9 .. 24.7] |
| bought where they pay, standing eight years; no quality, no inheritance | 1.27 [1.12 .. 1.39] | 0.98 [0.88 .. 1.08] | 3.7 [1.5 .. 5.6] |
| the whole of `machines`, port's content | 1.59 [1.37 .. 1.84] | 1.32 [1.21 .. 1.40] | 2.2 [1.0 .. 3.6] |
| the game (with the content) | 1.61 [1.41 .. 1.82] | 1.36 [1.28 .. 1.41] | 2.2 [1.3 .. 3.5] |

So the first rule ends the famine and adds no growth; quality gives the
growth; the steel content gives the converter and the steel rail their story
and a little more growth through the steel poured (18,700 a month in 1900
against 10,900 without the content and 7,000 in the port).

Settings far outside anything sensible were tried on seeds 3 and 7 under the
game's rules, to see what breaks: `adopt_span` 0, 0.01 and 100 with
`life_months` 0 and 2,400; `quality_b` 0 and 2 with `quality_cap` 0 and 50
and `quality_Q0` 0 and a million million; `quarry_reach` 0 and 20 with
`quarry_input` 0 and 1 and `inherit` 0 and 5. Of 44 runs, 42 finished and
kept their money. The two that did not are both seeds at `quality_b` 2,
`quality_cap` 50, `quality_Q0` 0, where every machine becomes fifty times
better in the month the first steel is poured: the monthly money check fails
that year by a few millionths. It fails the same way with the first
version's purchases, so it is the port's month that cannot settle a
fifty-fold jump in what every mine raises, not the machines' accounts; no
setting near the game's moves quality by more than a hundredth in a month.
On the first version, `quality_b` 2 with a cap of 50 and the game's
`quality_Q0` gave a 1900 wage of 2.9: steep, not runaway. And `quality_Q0` 0
with the game's cap of 4 and machines that stand a month gave 2.5 in 1850
and 2.1 in 1900: where the cap is reached early the gain is taken back, as
five times the people work the same land with nothing getting better, which
is the port's story after 1800.

## Decisions

- **Pumps and wagonways stay as the port buys them.** Measured both ways (on
  an earlier setting of the other numbers, eight seeds). With the rule on
  both, the 1800 real wage was 1.17 [1.11 .. 1.21] against 1.32
  [1.28 .. 1.39] and the steel rail mill was never found: the collieries lay
  their first wagonways at a loss, and those rails are what teach the island
  to roll rails and build railways. The rule on pumps alone changed nothing
  beyond the spread between seeds (1800 wage 1.30 against 1.32, peak price
  4.5 against 4.8). Both switches are there to measure again.
- **Quality counts all steel, not the converter's.** Counting only converter
  steel was the first design. Discovery is a yearly draw, and in one seed of
  sixteen the converter came in 1889: quality stayed at 1.05 and the wage of
  the 1890s was below 1850's. On all steel the worst seed of sixteen grows by
  a fifth. It also spreads the gain: quality is 2 by 1850, so 1800 to 1850 is
  no longer flat (1.16 times on the model's index, from 0.99), and machinery
  demand builds over fifty years instead of arriving in ten.
- **A farm that buys, buys for its own output** (changed in review). The
  first version asked for `input` times the share of farms that buy times the
  whole sector's output, as if the farms with machines grew only what the
  average farm grows. A farm with a machine grows `1 + boost` times what one
  without does, and its gain was reckoned on that, so its machinery has to be
  too: with three farms in ten equipped and a machine that adds 160 per cent
  the town was asking for 57 per cent of what its own sum says those farms
  need. The two agree where no farm or every farm has the machine. Sixteen
  seeds, before and after, at `adopt_span` 4: wage 1850 1.59 and 1.52, 1900
  2.13 and 2.07, 1890s over 1850 1.33 and 1.35, machinery's peak 2.4
  [1.3 .. 3.9] and 3.0 [1.3 .. 5.3], years above three times usual 0 [0 .. 2]
  and 2 [0 .. 9] (with families 2 [0 .. 11] and 4 [0 .. 20]). The growth is
  the same; the farms' true demand presses the engine works harder.
- **`adopt_span` 6.** At 1 (all farms buy once the machine earns twice its
  cost) machinery's price still peaked at 7 times usual and stayed above 3
  times for 23 years, because a farm machine earns six or seven times its
  cost at usual prices and the engine works answer slowly (below); at 4 the
  peak was 4.8 and 13 years (both on the first design of quality). The first
  version settled on 4. With the farms buying for their own output, 6 gives
  back the calm 4 had: peak 2.5 [1.3 .. 4.2] against 3.0 [1.3 .. 5.3], years
  above three times usual 1 [0 .. 4] against 2 [0 .. 9], and with families
  2.3 [0.8 .. 5.2] and 2 [0 .. 13] against 2.8 [1.2 .. 6.3] and 4 [0 .. 20].
  It costs nothing that sixteen seeds can show: the 1900 wage is 2.08 against
  2.07 (2.16 against 2.12 with families), and 0.83 of farms have the machine
  in 1900 against 0.86.
- **`quality_b` 0.6.** On all steel with `quality_Q0` a million: 0.5 gives
  1890s over 1850 of 1.28 [1.18 .. 1.42] and never reaches the cap; 0.6 gives
  1.33 [1.21 .. 1.45] and reaches the cap of 4 in the late 1890s in about half
  the seeds; 0.7 reaches it in every seed by 1900 with no more growth to show.
  With six families 0.5 left one seed at 1.04. (Measured by the first helper,
  before the two changes above; at the final numbers 0.6 gives 1.36
  [1.28 .. 1.41], and 1.31 [1.17 .. 1.45] with families. The other exponents
  were not run again.)

## Why machinery capacity answers so slowly

Both probes saw machinery's output take 28 years to rise sixfold at twenty
times the usual price. The cause is in two lines of the port:

- A works with anything on order counts as building (`firms::review`), and a
  building works may not grow. An order asks each month for a quarter of what
  is still outstanding (`econ::step_month`, `f_order * 0.25`), so it shrinks
  geometrically and is only struck off below a billionth of a unit: about 70
  months even when every delivery is filled. A works can therefore grow once
  in six or seven years, by half at most.
- So capacity comes from new works, and entry allows one a year per town and
  method, two when the return is above 0.8, each of 0.4 or 0.8 of the usual
  size. An engine works' usual size is 3 units a month.

Seed 7 of the port, 1868 to 1880: machinery's price between 9 and 25 times
usual throughout; the number of engine works rises from 287 to 670, 32 a
year, which is two per town; capacity per works stays between 1.4 and 1.6;
and 97 to 100 per cent of them count as building in every one of those years.

The works mechanism (`night/works`, now in the game's rules) changes the first
line: a works has finished building when under a twentieth of its capacity is
still on order. It changes the second too: a new works is sized to its town's
market and comes one at a time, never two a year in one town and method, nor
doubled (`works.md`).
See the first worry below.

## In company with the works mechanism

The two were tried together in review: `night/works` (at 96f7dbc) laid over
this branch in a scratch copy, nothing committed, and
`examples/machines.rs` run on seeds 1 to 8. Five-year means on the model's
index; "peak" is machinery's highest yearly price after 1850 over its usual
price, "years" the years it spends above three times usual.

Without families, to see which part does what (these five at `adopt_span` 4):

| works, and | wage 1900 | lowest 1855-85 | 1890s over 1850 | peak | years | converter's share 1880 | steel rails' share 1890 | steel a month 1900 |
|---|---|---|---|---|---|---|---|---|
| nothing of machines | 1.52 [1.45 .. 1.60] | 1.23 [0.91 .. 1.45] | 1.01 [0.86 .. 1.07] | 12.4 [3.4 .. 21.9] | 11 [1 .. 16] | 0 | 0 | |
| machines, port's content, `inherit` 0 | 2.59 [2.43 .. 2.79] | 1.95 [1.43 .. 2.34] | 1.42 [1.08 .. 1.87] | 3.7 [1.2 .. 7.9] | 2 [0 .. 7] | 0 | 0 | |
| machines, the game's content, `inherit` 0 | 2.58 [2.42 .. 2.70] | 2.00 [1.66 .. 2.31] | 1.42 [1.09 .. 1.73] | 3.5 [1.3 .. 8.0] | 2 [0 .. 7] | 0 | 0 | 20,600 |
| machines, port's content, `inherit` 0.5 | 2.48 [2.35 .. 2.58] | 1.90 [1.43 .. 2.22] | 1.36 [1.04 .. 1.80] | 4.7 [1.4 .. 8.2] | 4 [0 .. 9] | 0.78 | 0.01 | 30,100 |
| the whole game | 2.22 [1.43 .. 2.63] | 1.88 [1.52 .. 2.13] | 1.23 [0.87 .. 1.51] | 5.0 [1.9 .. 9.4] | 8 [0 .. 20] | 0.91 | 0.18 | 44,200 |

At the final numbers (`adopt_span` 6), without families and then with six,
which is how a game is played:

| works, and | wage 1900 | lowest 1855-85 | 1890s over 1850 | the same, chained | peak | years | converter's share 1880 | steel rails' share 1890 |
|---|---|---|---|---|---|---|---|---|
| the whole game, no families | 2.27 [2.02 .. 2.61] | 1.82 [1.51 .. 2.08] | 1.24 [1.07 .. 1.32] | 1.54 [1.30 .. 1.65] | 4.5 [2.4 .. 7.4] | 7 [0 .. 18] | 0.93 | 0.24 |
| the whole game, six families | 2.05 [1.68 .. 2.39] | 1.92 [1.32 .. 2.18] | 1.04 [0.93 .. 1.27] | 1.33 [1.10 .. 1.72] | 7.9 [2.3 .. 21.1] | 6 [0 .. 12] | 1.00 | 0.67 |
| six families, `inherit` 0 | 2.50 [2.13 .. 2.65] | 2.18 [1.75 .. 2.47] | 1.15 [0.94 .. 1.41] | 1.32 [1.13 .. 1.68] | 3.1 [1.9 .. 4.1] | 2 [0 .. 5] | 0.01 | 0.12 |
| six families, `inherit` 0.5 on the port's content | 2.31 [1.98 .. 2.47] | 2.15 [1.93 .. 2.37] | 1.06 [0.82 .. 1.45] | 1.29 [1.02 .. 1.78] | 3.8 [2.2 .. 5.3] | 3 [0 .. 6] | 0.94 | 0 |
| six families, `quality_Q0` 3 million | 1.90 [1.49 .. 2.16] | 1.62 [1.48 .. 1.81] | 1.20 [0.88 .. 1.32] | 1.51 [1.28 .. 1.60] | 5.4 [2.4 .. 7.7] | 7 [0 .. 15] | 0.85 | 0.80 |

No run of these failed the money check. What they say:

- **The machines do their work beside the works mechanism.** It keeps the
  port's machinery famine on its own (peak 12 times usual); with the machines
  the famine goes and the 1900 wage is 2.5 to 2.6, higher than either alone.
- **The converter's numbers do not carry over.** Under the works rules the
  cheap plant alone no longer gets a converter built, inheritance alone gets
  it four fifths of the steel by 1880, and the two together pour twice the
  steel (44,000 a month and more against 19,000 here) and cost the 1900 wage
  a seventh without families and a fifth with them (2.05 against 2.50), with
  machinery up to twenty times its usual price in the worst seed. At
  `adopt_span` 4 two seeds of eight ended the 1890s below 1850 (seed 4: 1.43
  in 1900, machinery at 200 to 320 from 1891 on, ten times its price of the
  1880s, and a third of its farms' machines given up); at 6 none does without
  families. The first helper saw this coming from a stand-in for the building
  rule, which made it look worse than it is (a 1900 wage of 1.54
  [0.80 .. 2.08]); their reading of one seed was that converter works which
  may grow by half a year, in a method whose skill doubles in a few years,
  overshoot the steel market and drag iron and the toolmakers after it. I
  did not trace it further.
- **The quality curve arrives too soon there.** The works rules pour more
  steel from the start, so quality is 2.2 to 2.5 in 1850 (2.0 to 2.1 here) and at the cap
  of 4 in every seed and every row by 1900, by 1875 to 1885 in the three
  seeds looked at. With six families that leaves the 1890s where 1850 was on
  the model's index (1.04 to 1.15) whatever the converter does, though the
  1900 wage is as high as 2.5: the growth has moved into the first half of
  the century.
  A slower curve (`quality_Q0` 3 million: quality 1.6 in 1850, 3.0 in 1900)
  spreads it out again but, with the converter on, lowers the whole path.
- **The steel rail does not carry over either**: 0.18 to 0.24 of rails in
  1890 without families and 0.67 with, against 0.86 and 0.96 here.

So when the two are merged, the converter's three numbers (`inherit`,
`MACHINES_PLANT_SCALE`, `MACHINES_STEEL_RAIL_WORTH`) and the quality curve
(`quality_Q0`, `quality_b`, `quality_cap`) need tuning again together, on
many seeds, with the works rules and six families on. Until that is done the
safe setting is `machines.inherit = 0`: under the works rules it leaves the
converter unbuilt, and it is the best row of both tables in everything else.

**Since merged.** The life cycle for works is now in the game's rules beside
the machines, the merchants' prices, the curve and the harvests, at its
reviewed numbers (`works.md`, "In company with the rest of the game"; seeds
1 to 12, six families). The wage did not suffer as the trial above feared:
1.56, 2.51 and 2.80 in 1800, 1850 and 1900 for 1.42, 1.84 and 2.45 without
the life cycle, higher on every island. The rest is as foretold. The
converter makes all the steel by 1880 but is found between 1816 and 1830 in
three seeds of six; steel rails are a quarter of rails in 1890 where they
were all of them, iron being half the price it was; and growth has moved
forward, the wage hardly rising after the 1850s. `inherit = 0` gives 2.98 in
1900 for 2.80 on seeds 1 to 6 and no converter; it was left at 0.5, since a
game without a converter is the poorer for it and the wage is not in
danger. The tuning asked for above is still to do, and the steel rail has
gone further the same way since: in today's game it is two thousandths of
the rails of 1890 ("In today's game", below).

## In company with the merchants' prices and the curve

Everything above was measured on the port with the machines alone. When they
were merged into the game's rules the merchants' prices (`prices.md`) and the
Engel curve (`engel.md`) were already there, so the three were run together:
seeds 1 to 6 and then 7 to 12, six families, to 1900, the game's rules before
the machines and with them (`core-steps/machines-before`, `machines-after`,
`machines-s7-12-before` and `machines-s7-12` under `.claude/night/reports/`),
and this page's own example on sixteen seeds with six families and with none
(`core-steps/machines-example-f6` and `-f0`). No run failed. Seeds 1 to 6, the
tally seed for seed, and beside it what the machines did alone:

| six families | prices and curve | with the machines | tally | alone (port, then machines) |
|---|---|---|---|---|
| real wage 1800, 1750 basket (1750 = 1) | 1.40 | 1.42 | 4/2 | 1.33, 1.34 |
| real wage 1850 | 1.49 | 1.95 | 6/0* | 1.34, 1.66 |
| real wage 1900 | 1.45 | 2.48 | 6/0* | 1.34, 2.16 |
| real wage 1900, chained | 1.77 | 3.36 | 6/0* | |
| output per head 1900 (1750 = 1) | 1.50 | 2.60 | 6/0* | 1.46, 2.60 (no families) |
| the year output per head peaks | 1858 | 1888 | 6/0* | 1867, 1896 (no families) |
| towns: share of people, 1900 | 0.320 | 0.432 | 6/0* | 0.24, 0.27 (no families) |
| works: share of labour, 1900 | 0.354 | 0.491 | 6/0* | 0.25, 0.29 (no families) |
| land: share of labour, 1900 | 0.364 | 0.259 | 0/6* | |
| grain output 1900 (1750 = 1) | 7.48 | 10.40 | 6/0* | |
| food, comfort: share of budgets, 1900 | 0.266, 0.145 | 0.150, 0.256 | 0/6*, 6/0* | |
| works standing, 1900 | 6,366 | 6,805 | 5/1 | |
| the ninth tenth of works, size | 2.69 | 1.95 | 0/6* | 2.5, 1.6 |
| median life of a works that closed | 14.8 | 17.5 | 6/0* | 12, 15 (no families) |
| town cells, 1900 | 94.8 | 123.8 | 6/0* | |
| industry cells, 1850 and 1900 | 575, 945 | 668, 959 | 6/0, 5/1 | |
| railway km, 1850 and 1900 | 1,045, 1,077 | 981, 1,003 | 1/5, 2/4 | |
| converter's share of steel, 1880 | 0 | 0.85 [0.13 .. 1.00] | 6/0* | 0.01, 0.97 |
| steel rails' share of rails, 1890 | 0 | 0.90 [0.49 .. 1.00] | 6/0* | 0, 0.96 |
| farm machinery found | 1840 | 1827 | 0/5 | 1843, 1832 (no families) |
| price swings: grain, iron, rails, cloth | 0.032, 0.128, 0.150, 0.024 | 0.026, 0.121, 0.141, 0.018 | cloth 0/6* | |
| farm zigzag | 0.84 | 0.56 | 0/6* | |
| famine notices, alarms a run | 3.3, 2.2 | 1.5, 0.8 | 0/5, 1/3 | |
| seconds a run, back to back | 17 | 18 to 20 | | |

Seeds 7 to 12 say the same: the wage 1.45, 1.74 and 2.44 for 1.47, 1.42 and
1.43, output per head in 1900 2.54 for 1.49, towns 0.433 for 0.327, works'
labour 0.495 for 0.362, 6,892 works for 6,278 with the ninth tenth at 1.86 for
2.67, town cells 125 for 97, the converter 0.99 of steel in 1880 and steel
rails 0.99 of rails in 1890, the zigzag 0.64 for 0.92, famine notices 2.2 for
3.3. The railway of 1900 is the same length there (1,076 km for 1,078) and 7
per cent shorter on seeds 1 to 6 (two islands up, four down), so I read
nothing into it; on seeds 7 to 12 the line of 1850 is a third longer (941 km
for 700, every island), because the first line is laid nine years sooner.

And the example's own numbers on sixteen seeds, beside the columns of "What
the sweeps showed" (five-year means, the model's index unless said):

| | alone, six families | in company, six families | in company, no families |
|---|---|---|---|
| real wage 1800 / 1850 / 1900 | 1.34 / 1.66 / 2.16 | 1.46 / 1.87 / 2.51 | 1.42 / 1.75 / 2.47 |
| lowest five-year mean, 1855-85 | 1.74 [1.42 .. 2.02] | 2.04 [1.65 .. 2.35] | 1.86 [1.58 .. 2.26] |
| 1890s over 1850 | 1.31 [1.17 .. 1.45] | 1.35 [1.15 .. 1.53] | 1.41 [1.18 .. 1.60] |
| the same, chained index | 1.51 [1.38 .. 1.68] | 1.53 [1.33 .. 1.67] | 1.57 [1.38 .. 1.72] |
| machinery's peak after 1850 over usual | 2.3 [0.8 .. 5.2] | 1.4 [0.8 .. 2.7] | 1.6 [0.7 .. 2.7] |
| years above three times usual | 2 [0 .. 13] | 0 in every seed | 0 in every seed |
| farm output per worker-month 1800 / 1850 / 1900 | 13.6 / 19.8 / 40.4 | 14.1 / 23.5 / 51.7 | 13.5 / 19.2 / 50.4 |
| quarry | 7.7 / 11.2 / 18.7 | 7.9 / 11.9 / 21.5 | 7.7 / 11.2 / 21.4 |
| colliery | 22.5 / 30.0 / 45.6 | 24.3 / 34.8 / 52.0 | 24.1 / 33.1 / 52.9 |
| converter first built | 1850 [1839 .. 1867] | 1847 [1826 .. 1876] | 1856 [1839 .. 1871] |
| converter's share of steel, 1880 | 0.97 [0.55 .. 1.00] | 0.94 [0.12 .. 1.00] | 0.85 [0.32 .. 1.00] |
| steel rails' share of rails, 1890 | 0.96 [0.69 .. 1.00] | 0.96 [0.49 .. 1.00] | 0.75 [0.48 .. 1.00] |
| quality of machines 1850, 1900 | 2.0 to 2.1, 3.98 | 2.20, 3.99 | 2.03, 3.95 |

What they say:

- **In company the machines do what they do alone, and a little more.** The
  wage of 1900 is a whole 1750 wage higher on every island (2.48 for 1.45),
  where alone it was four fifths higher (2.16 for 1.34). The 1890s stand a
  third above 1850, as alone. The converter and the steel rail arrive as they
  did, a little less surely in the slowest seed (the converter is found by a
  yearly draw, and in one seed of sixteen it comes in 1875).
- **The machinery famine is gone outright.** Alone, machinery's price still
  peaked at 2.3 times usual and spent up to thirteen years above three times
  usual in the worst seed. With the merchants marking prices on their stock it
  peaks at 1.4 times and spends no year above three times in any of thirty-two
  runs. The two mend different halves of the same trouble: the machines stop
  the farms from all buying at once, the merchants stop the price from running
  on after the shelves have filled.
- **The curve turns the machines' bread into towns.** Alone the machines
  raised the towns' share of people by three points; here by eleven (0.43 for
  0.32), and the works' share of labour by fourteen. A household with more
  than twice the income (8.3 bundles a head for 3.8) spends 15 per cent of it
  on food where it spent 27, and what it buys instead is made in works. The land keeps a quarter of all hands in 1900 and
  grows ten times the grain of 1750. This is the first set of rules under
  which the island urbanises (a quarter of people in towns in 1750, 0.43 in
  1900), and it takes the pair: the curve alone gave six points and the
  machines alone three.
- **Fewer famines and alarms, a calmer farm.** Famine notices halve on seeds
  1 to 6 (fewer on five islands, the same on one) and fall a third on seeds 7
  to 12 (fewer on three, more on two); the farm zigzag falls a third on all
  twelve. The curve had left households nearer the floor in a bad year, and
  cheaper grain lifts them off it.
- **The works at the top are smaller here too**: the ninth tenth is 1.95
  times its method's usual size for 2.69, as alone. Still not traced, still
  for the works mechanism's sweep.
- **One island ends with derelict ground**: 45 cells of it in 1900 on one seed
  of six (34 on another seed before the mend below), none on the rest and
  none before the machines. Not opened; probably the old steel and rail works
  closing together when the converter wins, which the works mechanism's life
  cycle will change anyway.
- **With the machines switched off** (`--set machines.enabled=false --content
  data/content.toml` under the game's rules) the merged code gives every fact
  of every seed exactly as the code before the merge gave it
  (`core-steps/machines-off`): they leak nothing into the others.

**One thing broke in company, and it was not the machines' doing.** On seed 6
a town was founded at the end of 1888. The port never tells a new town what
to expect: its farms and pits hire on `pexp`, and a slot that has never been
a town still holds the prices of 1750. In the port that is three or thirty
years stale and passes unseen. Here grain was three times cheaper than in
1750, so the new town's farms expected three times the price, asked for
thirty-five times the hands the town had, and got them: nothing in the port
rations work to the people there are. The town's wage doubled month after
month, the landlords' purses paid it (the rent share of 1889 is -1.9),
and with that money in households' hands the island's prices rose 3.3 times
in five years and stayed there; the real wage fell from 2.5 to 1.6 on the way
(`core-steps/machines-after-merge`, and seed 6 month by month in
`core-steps/machines-seed6-run`). The mend is in the prices mechanism, where
a new town already took its mother town's anchors: it now also takes what her
people expect, less the carriage from its site to her, which is what a load
from there fetched the day before (`prices::found_town`; `prices.md` has the
account and the count on 48 seeds). The table above is after the mend. Before
it, on seeds 1 to 6, the wage read 1.43, 1.94 and 2.43 and the rest within a
hair of the table, seed 6's last decade aside.

Not done here: the scripted players and the prices probe's shock were not run
on the three together.

## In today's game

Every table above was measured on rules that are no longer the game's: since
the three were run together the harvests, the life cycle for works, the
railways, carting, the people, the towns, the mines, the canals and the books
of account have joined them. This is where the machines' own measures stand
today, read from the sweep of the merged game, seeds 1 to 8 with six
families, at the code of `340f263` (`isostate-core sweep --seeds 1-8 --rules
game --families 6`, under `.claude/night/reports/merged-game-f6-l/`). The
facts are the sweep's own; the rest were worked out from its `annual.csv` by
the definitions of `examples/machines.rs` (five-year means over 1750 to 1752,
machinery's highest yearly price after 1850 over the median of 1836 to 1850),
without running the example. Two differ from the example's: the chained wage
is the observer's `real_wage_chained`, not the example's own chain, and
quality is reckoned from the yearly output of steel, which the sweep keeps,
not read from the machines.

| six families, seeds 1 to 8 | today |
|---|---|
| real wage 1800 / 1850 / 1900, 1750 basket | 1.63 / 2.01 / 2.58 |
| lowest five-year mean, 1855-85 | 2.22 [1.84 .. 2.51] |
| 1890s over 1850 | 1.32 [1.03 .. 1.57] |
| real wage 1800 / 1850 / 1900, chained | 1.91 / 2.91 / 4.25 |
| the same, 1890s over 1850 | 1.48 [1.23 .. 1.76] |
| machinery's peak after 1850 over usual | 2.1 [1.0 .. 2.5] |
| years above three times usual | 0 in every seed |
| quality of machines 1850, 1900 (reckoned) | 2.47 [1.02 .. 2.93], 3.93 [3.47 .. 4.00] |
| quality reaches the cap of 4 | 1864 to 1883 on seven seeds; never on seed 3 |
| farm machinery found | 1837 [1827 .. 1850] |
| converter found, first built | 1846 [1822 .. 1886], 1848 [1825 .. 1888] |
| converter's share of steel, 1880 | 0.86 [0.00 .. 1.00] |
| steel rail mill found | 1845 [1816 .. 1891] |
| steel rail mill first built | 1831 to 1860 on seven seeds; never on seed 3 |
| steel rails' share of rails, 1890 and 1900 | 0.002 [0 .. 0.012], 0.007 [0 .. 0.041] |
| steel a month, 1898-1900 | 55,300 [31,100 .. 67,400] |

Output per worker on the farms, in the quarries and in the collieries, and the
share of farms worked by machine, are not in the sweep and were not measured
again; the numbers for them above are of their own rules.

What it says. The machinery famine stays gone: machinery's dearest year after
1850 is at most two and a half times its usual price, and never three. The
wage of the 1890s still stands a third above 1850's, and the chained index
puts the century's gain at 4.25 where the 1750 basket says 2.58. The
converter comes as it did in company, making all the steel by 1880 on six
islands of eight; on one (seed 3, which pours the least steel) it is not
built until 1888, quality is still 1.02 in 1850 and never reaches the cap.
The steel rail is the change: the mill is found and built on seven islands of
eight but rolls almost nothing, where with the machines alone it rolled
nearly all the rails of 1890, and a quarter of them once the life cycle for
works had joined. The worry above about the
converter's numbers and the quality curve wanting tuning together, with the
works rules on, now covers the steel rail as well.

## What worries me

- **A town can still hire more hands than it has.** The mend above stops the
  founding that set it off, not the thing itself, which is the port's: when a
  town's demand for hands is above its people even at twice last month's wage,
  everyone asked for is put to work and paid. Somewhere between one run in
  four and one in twelve has such a "wild year" under any rules, most of them
  small (`prices.md`, "In company with the machines" and its first worry).
  The machines raise the stakes, because the land's output and prices now move
  far from where they were, and anything that still remembers an old price
  can ask for a great many hands. Under the commons scenario a wild year
  can be the end of an island: the loss goes to the common fund, and the
  port's fund charges it to every household by head. The game's rules are
  mended for that (`mines.md`, "Under the commons"); the machines alone over
  the port are not, and seed 3 of sixteen stops there in 1837, ruined since
  the wild year of 1753.
- **A new town's first year of land use is still read off 1750.** The yearly
  land step bids for a new town's cells with that town's average prices and
  wage over the year before it existed (`pbar`, `wbar` in `sim::yearly`): the
  base prices, and a wage of nothing. Land moves a little each year, so it is
  one year's nudge and I saw no harm from it; it is the same kind of stale
  number as the one mended and wants the same look.
- **The converter in company with the works mechanism**: the section above.
- **The cap is reached.** With six families quality is at 4 in nearly every
  seed by 1900 (3.98 on average, 3.81 without families), so the last years
  grow less. Past the cap the land's old arithmetic returns: more people on
  the same ground, and nothing getting better. The cap is `learn_cap`'s twin
  and was left; a curve that flattens without stopping would be more honest
  than a ceiling, and wants its own calibration.
- **A steel rail's worth is taken at the mill, so it cheapens new track
  too.** A steel rail outlasts iron ones, which is worth something to a
  railway that must relay its line; a mile of new line needs the same weight
  of rail either way. With one good called rails there is nowhere else to put
  it, so a steel mill's rail is three rails for building as well as for
  relaying.
- **What the pumps add now lags their deliveries by years**, in the game and
  from 1750: part of "machines stand for years" and meant, but it does mean
  the mines of a game differ a little from the port's from the first year
  (the land under mines in 1750 is 3 per cent larger in every seed).
- **Steel still has no use of its own.** It matters here because the engine
  builders learn from it. Households already take nine tenths of their
  implements as steel in 1840 (0.87 on today's eight islands, from
  `share_implements_steel` in the sweep's `annual.csv`, but 0.03 on seed 3,
  which pours the least steel). A steel that goes into the machines themselves
  would be a change to the engine works' recipe, which is content for another
  night.
- **A steel rail mill in 1815.** In one run of thirty-two (seed 6 with
  families) the mill was found by a draw of about one in two thousand and ran
  on crucible steel for twenty-five years before any converter. The discovery
  rule allows such flukes for every method; this one reads oddly.
- **The model's own index understates all of this.** It prices the 1750
  basket, heavy in firewood and wool. On the chained index the game's 1900
  wage was 2.82, not 2.08, when the machines stood alone; in today's game it
  is 4.24, not 2.60 (the sweep's `real_wage_chained_1900_vs_1750` and
  `real_wage_1900_vs_1750`).
- **Farms are never all mechanised** (0.83 of farms in 1900): with
  `adopt_span` 6 the last farms wait until the machine earns seven times its
  cost. That is the price of a calm machinery market while the engine works
  answer slowly; with a faster works sector the span could come down.
- **A farm equips itself only as fast as it will later replace.** A farm that
  takes up the machine buys at the steady monthly rate from the first month,
  and what stands on its land builds up over `life_months`; one that gives it
  up stops buying and its machines wear out as slowly. The account is exact,
  but nobody buys a machine outright. That would be nearer the truth and
  would make demand for machinery lumpier, which is the thing this mechanism
  set out to calm. While a town's machines are still building, its farms pay
  for more than is standing: at the worst (none standing yet) machinery can
  take `boost / (1 + adopt_span)` of the price of what they grow, from a
  ninth with the machines of 1750 (a boost of 0.8) to nearly a half at the
  cap (3.2) at the game's numbers. With `adopt_pumps` on and quality near the cap
  a pump's boost is 8 and that share could pass the whole price; the switch
  is off, and wants this looked at before it is turned on.
- **One wild year in the 1750s, somewhere.** In one seed of sixteen the real
  wage doubles for a single year and falls back (the port: seed 3 in 1753;
  the game: seed 7 in 1755, and not seed 3). It is the port's own oddity,
  moved to another island by the different history, not something the
  machines do; it is worth its own look. (It got one when the machines were
  merged: a town hiring more hands than it has. See the first worry.)
