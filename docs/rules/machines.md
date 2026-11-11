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
share in proportion between. The machinery asked for is scaled by that share
and by know-how. So when machinery is dear fewer farms buy, and the price
stops where the market clears instead of running to the ceiling.

**2. A machine stands for years.** Each town keeps, for each use, the share of
output worked by machines now standing. Every month it moves one part in
`life_months` towards the share that kept its machines up that month and was
supplied. What a machine adds comes from the machines standing; what is bought
comes from who keeps theirs up. A month the engine works cannot deliver does
not flood the pits; years of it do. Settlers bring their engines with them: a
new town starts from what its people keep up.

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
skill, if that is more than its own. On this island that is the converter
after crucible steel and the steel rail mill after the rolling mill.

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

## The numbers

| `machines.` | value | |
|---|---|---|
| `enabled` | false; true in the game | |
| `adopt_span` | 4 | a machine is taken by all only where it earns its cost five times over |
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
| real wage 1800 | 1.32 [1.22 .. 1.43] | 1.32 [1.24 .. 1.38] | 1.33 [1.07 .. 1.43] | 1.35 [1.28 .. 1.43] |
| real wage 1850 | 1.31 [1.15 .. 1.57] | 1.59 [1.45 .. 1.77] | 1.34 [1.14 .. 1.60] | 1.73 [1.50 .. 2.08] |
| real wage 1900 | 1.31 [1.17 .. 1.42] | 2.13 [1.91 .. 2.31] | 1.34 [1.22 .. 1.41] | 2.18 [1.97 .. 2.32] |
| lowest five-year mean, 1855-85 | 1.09 [0.86 .. 1.40] | 1.64 [1.43 .. 1.81] | 1.27 [1.10 .. 1.45] | 1.78 [1.51 .. 1.98] |
| 1890s over 1850 | 1.01 [0.89 .. 1.19] | 1.33 [1.21 .. 1.45] | 1.01 [0.87 .. 1.21] | 1.26 [1.10 .. 1.39] |
| the same, chained index | 1.10 [0.95 .. 1.31] | 1.50 [1.36 .. 1.66] | 1.12 [0.93 .. 1.38] | 1.45 [1.28 .. 1.61] |
| machinery's peak price after 1850, over its usual price (median of 1836-50) | 14.9 [0.9 .. 31.3] | 2.4 [1.3 .. 3.9] | 8.4 [0.9 .. 18.7] | 2.6 [1.3 .. 6.2] |
| years after 1850 with machinery above three times usual | 14 [0 .. 26] | 0 [0 .. 2] | 9 [0 .. 19] | 2 [0 .. 11] |
| farm output per worker-month 1800 / 1850 / 1900 | 13.1 / 14.2 / 17.9 | 13.1 / 16.6 / 39.6 | 13.2 / 16.2 / 18.7 | 13.6 / 20.8 / 42.3 |
| quarry | 7.6 / 6.9 / 6.1 | 7.7 / 11.5 / 18.9 | 7.7 / 6.9 / 6.1 | 7.7 / 12.3 / 19.7 |
| colliery | 19.8 / 16.3 / 13.5 | 21.7 / 27.7 / 41.9 | 19.5 / 15.8 / 13.7 | 22.6 / 30.1 / 44.5 |
| converter first built | 1892 in 6 of 16 | 1854 [1827 .. 1875] | 1873 in 4 of 16 | 1850 [1839 .. 1866] |
| converter's share of steel, 1880 | 0 | 0.77 [0.22 .. 1.00] | 0.01 | 0.96 [0.46 .. 1.00] |
| steel rail mill first built | 1882 in 3 of 16 | 1861 [1842 .. 1885] | 1893 in 1 of 16 | 1856 [1815 .. 1879] |
| steel rails' share of rails, 1890 | 0 | 0.83 [0.45 .. 1.00] | 0 | 0.94 [0.66 .. 1.00] |

No run failed the monthly money check. In the sweep's own scorecard (eight
seeds, `--rules game` against the port) the facts that move clear of the
spread between seeds are the 1900 real wage (2.14 against 1.29) and output
per head (2.66 against 1.46), the urban and industry shares of 1900 (0.27 and
0.29 against 0.24 and 0.25), the two steel shares, half the alarms in the
news, and a sixth less woodland. Output per head now peaks in 1896 on average
instead of 1867. The first railway (1830 against 1828), the railway's length
and the 1800 wage do not move outside the spread. Farm machinery is found
about ten years sooner (1832 against 1843): its threshold is in machinery
made, and the quarries' buying adds to that.

What each part does alone (eight seeds, model's index):

| | lowest 1855-85 | 1890s over 1850 | machinery peak over usual |
|---|---|---|---|
| port | 1.10 [0.86 .. 1.40] | 0.97 [0.89 .. 1.07] | 11.3 [0.9 .. 24.7] |
| bought where they pay, standing eight years; no quality, no inheritance | 1.28 [1.12 .. 1.41] | 0.97 [0.87 .. 1.07] | 4.0 [1.8 .. 5.5] |
| the whole of `machines`, port's content | 1.66 [1.43 .. 1.86] | 1.29 [1.16 .. 1.45] | 2.3 [1.2 .. 3.5] |
| the game (with the content) | 1.66 [1.43 .. 1.81] | 1.31 [1.21 .. 1.39] | 2.5 [1.5 .. 3.7] |

So the first rule ends the famine and adds no growth; quality gives the
growth; the steel content gives the converter and the steel rail their story
and a little more growth through the steel poured (19,500 a month in 1900
against 11,000 without the content and 7,000 in the port).

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
  no longer flat (1.20 times on the model's index, from 0.99), and machinery
  demand builds over fifty years instead of arriving in ten.
- **`adopt_span` 4, not 1.** At 1 (all farms buy once the machine earns twice
  its cost) machinery's price still peaked at 7 times usual and stayed above
  3 times for 23 years, because a farm machine earns six or seven times its
  cost at usual prices and the engine works answer slowly (below). At 4 the
  peak was 4.8 and 13 years. (Both on the first design of quality; on all
  steel the peak is 2.4.)
- **`quality_b` 0.6.** On all steel with `quality_Q0` a million: 0.5 gives
  1890s over 1850 of 1.28 [1.18 .. 1.42] and never reaches the cap; 0.6 gives
  1.33 [1.21 .. 1.45] and reaches the cap of 4 in the late 1890s in about half
  the seeds; 0.7 reaches it in every seed by 1900 with no more growth to show.
  With six families 0.5 left one seed at 1.04.

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

## What worries me

- **In company with the works mechanism the converter may wreck the late
  century.** I could not merge `night/works`, so I stood in for its building
  rule with a temporary switch (never committed) and ran the game on eight
  seeds. The port alone becomes calmer under that rule (machinery's peak 4.0
  times usual from 11). The game does not: machinery's peak price is 45 times
  usual [6 .. 84], the lowest five-year real wage of 1855-85 is 1.06
  [0.68 .. 1.46] and the 1900 wage 1.54 [0.80 .. 2.08]. The chained index
  still grows (3.2 in 1900), so this is a boom and bust, not a stall. The
  cause is the converter, not the machines: with quality on, inheritance off
  and the port's content, the same rule is calm (peak 3.6 [2.3 .. 4.9],
  lowest wage 1.74 [1.59 .. 1.88], at `quality_b` 0.5); wherever converters
  are built it is not. My reading, from one seed traced year by year (seed 3):
  works that may grow by half every year, in a method whose skill doubles in
  a few years and whose plant is cheap, overshoot the steel market; steel
  falls to 6 or 7 from 25, iron swings between 2 and 16, tool output swings
  a hundredfold, and when the toolmakers have closed there are no tools for
  the engine works (tools at 290 from 10 and machinery output at a tenth,
  1898-1900). If the two are merged, measure first. The calm fallback is
  `machines.inherit = 0` with `machines_content` left out of `game_content`;
  it keeps points 1 to 4 and loses the converter.
- **The cap is reached.** With six families quality is at 4 in nearly every
  seed by 1900, so the last years grow less (1890s over 1850 is 1.26 with
  families, 1.33 without). The cap is `learn_cap`'s twin and I left it.
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
  wage is 2.89, not 2.13.
- **Farms are never all mechanised** (0.87 of farm output in 1900): with
  `adopt_span` 4 the last farms wait until the machine earns five times its
  cost. That is the price of a calm machinery market while the engine works
  answer slowly; with a faster works sector the span could come down.
