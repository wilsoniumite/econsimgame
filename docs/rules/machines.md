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
takes, at today's price in that town. Farms differ (the size of a field, the
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
machines are standing. Every month it moves one part in `life_months` towards
the share that keeps its machines up and was supplied that month. What a
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
that starts clumsy and climbs a curve) of making a good the island already
makes by an earlier learned method starts from `inherit` of that method's
skill, if that is more than its own (and never from more than the whole of
it). On this island that is the converter after crucible steel and the steel
rail mill after the rolling mill. It is a floor, not a gift: once the new
method's own curve passes it, the method is on its own.

**6. Content** (`machines_content`, applied in `game_content`):

- The converter's and the steel rail mill's plant cost `MACHINES_PLANT_SCALE`
  (0.3) of the port's. The port's converter needs about a thousand in money
  per unit of monthly capacity, five times a rolling mill; at the hurdle rate
  that is 12 a unit of steel in capital alone where crucible steel carries 2.
- A steel rail outlasts iron ones several times over. There is one good,
  "rails", counted in iron rails, so a ton of steel rail is entered as the
  three iron rails it stands in for (`MACHINES_STEEL_RAIL_WORTH`): the mill
  takes a third of the steel per rail.
- Discovery thresholds (`MACHINES_THRESHOLDS`): converter at 2.8 million of
  steel and 90,000 of machinery; steel rail mill at 3.3 million of steel,
  350,000 of rails and 150,000 of machinery.

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
Real wages are five-year means over 1750-52, on the model's own index (the
1750 basket, what the sweep's `real_wage` is) and on a chained index of what
was actually bought.

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

The works mechanism (`night/works`) changes the first line: a works has
finished building when under a twentieth of its capacity is still on order.
See the first worry below.

## In company with the works mechanism

The two were tried together in review: `night/works` (at 96f7dbc) laid over
this branch in a scratch copy, nothing committed, and
`examples/machines.rs` run on seeds 1 to 8 without families. This was done
with `adopt_span` still at 4; five-year means on the model's index.

| works, and | wage 1900 | lowest 1855-85 | 1890s over 1850 | machinery peak over usual | years above 3 times | converter's share 1880 | steel rails' share 1890 | steel a month 1900 |
|---|---|---|---|---|---|---|---|---|
| nothing of machines | 1.52 [1.45 .. 1.60] | 1.23 [0.91 .. 1.45] | 1.01 [0.86 .. 1.07] | 12.4 [3.4 .. 21.9] | 11 [1 .. 16] | 0 | 0 | |
| machines, port's content, `inherit` 0 | 2.59 [2.43 .. 2.79] | 1.95 [1.43 .. 2.34] | 1.42 [1.08 .. 1.87] | 3.7 [1.2 .. 7.9] | 2 [0 .. 7] | 0 | 0 | |
| machines, the game's content, `inherit` 0 | 2.58 [2.42 .. 2.70] | 2.00 [1.66 .. 2.31] | 1.42 [1.09 .. 1.73] | 3.5 [1.3 .. 8.0] | 2 [0 .. 7] | 0 | 0 | 20,600 |
| machines, port's content, `inherit` 0.5 | 2.48 [2.35 .. 2.58] | 1.90 [1.43 .. 2.22] | 1.36 [1.04 .. 1.80] | 4.7 [1.4 .. 8.2] | 4 [0 .. 9] | 0.78 | 0.01 | 30,100 |
| the whole game | 2.22 [1.43 .. 2.63] | 1.88 [1.52 .. 2.13] | 1.23 [0.87 .. 1.51] | 5.0 [1.9 .. 9.4] | 8 [0 .. 20] | 0.91 | 0.18 | 44,200 |

What that says:

- **The machines do their work beside the works mechanism.** It keeps the
  port's machinery famine on its own (peak 12 times usual); with the machines
  the famine goes and the 1900 wage is 2.6, higher than either alone.
- **The converter's numbers do not carry over.** Under the works rules the
  cheap plant alone no longer gets a converter built, inheritance alone gets
  it four fifths of the steel by 1880, and the two together pour twice the
  steel and cost a seventh of the 1900 wage: in two seeds of eight the 1890s
  are below 1850 (seed 4 ends at 1.43, with machinery at 200 to 320 from
  1891 on, ten times its price of the 1880s, and a third of its farms'
  machines given up). The steel rail hardly arrives at all (0.18 of rails in
  1890 against 0.86 here). The first helper saw this coming from a stand-in
  for the building rule, which made it look much worse than it is (a 1900
  wage of 1.54 [0.80 .. 2.08]); their reading of one seed was that converter
  works which may grow by half a year, in a method whose skill doubles in a
  few years, overshoot the steel market and drag iron and the toolmakers
  after it.
- **Quality is at the cap in every seed by 1900** (by 1875 to 1885 in the
  three seeds looked at), because twice the steel is poured. After that
  nothing improves, and that is when the two bad seeds turn down.

So when the two are merged, the converter's three numbers (`inherit`,
`MACHINES_PLANT_SCALE`, `MACHINES_STEEL_RAIL_WORTH`) and the quality curve
(`quality_Q0`, `quality_b`) need tuning again together, on many seeds, with
the works rules on. Until then the safe setting is `machines.inherit = 0`,
which under the works rules leaves the converter unbuilt and everything else
as good as it gets in that table.

## What worries me

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
  implements as steel in 1840. A steel that goes into the machines themselves
  would be a change to the engine works' recipe, which is content for another
  night.
- **A steel rail mill in 1815.** In one run of thirty-two (seed 6 with
  families) the mill was found by a draw of about one in two thousand and ran
  on crucible steel for twenty-five years before any converter. The discovery
  rule allows such flukes for every method; this one reads oddly.
- **The model's own index understates all of this.** It prices the 1750
  basket, heavy in firewood and wool. On the chained index the game's 1900
  wage is 2.82, not 2.08.
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
  take `boost / (1 + adopt_span)` of the price of what they grow, a quarter
  to a half at the game's numbers. With `adopt_pumps` on and quality near the cap
  a pump's boost is 8 and that share could pass the whole price; the switch
  is off, and wants this looked at before it is turned on.
- **One wild year in the 1750s, somewhere.** In one seed of sixteen the real
  wage doubles for a single year and falls back (the port: seed 3 in 1753;
  the game: seed 7 in 1755, and not seed 3). It is the port's own oddity,
  moved to another island by the different history, not something the
  machines do; it is worth its own look.
