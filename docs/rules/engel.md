# An Engel curve, and an honest price index

`P.engel`, in `crates/isostate-core/src/engel.rs`. Off under the port, on under
the game's rules. Not in the Python.

## What was wrong

The port's households split whatever they have above subsistence by fixed
shares: 28 per cent on food, 14 on warmth, 22 on shelter, 19 on clothing, 7 on
implements and 10 on comfort, however well off they are. Real income per head
doubles between 1750 and 1900 (from 1.8 to 3.6 subsistence bundles), and the
households of 1900 answer by eating 1.8 times the grain a head. The farms
therefore keep their hands, the works' share of labour stays at a quarter for
150 years, and since a town's size follows the share of its people who work in
works, the towns' share of people stays at a quarter too (0.26 in 1750, 0.24 in
1900). Two probes found the same cause from two sides (`dx-towns`, `dx-growth`).

Nobody who doubles their income doubles their bread. Engel's law is the oldest
regularity in household budgets: the richer the household, the smaller the part
of a further shilling that goes on food.

The second thing wrong was the yardstick. The cost of living was measured on
the basket of 1750, heavy in firewood and homespun wool, for 150 years in which
people moved to coal and mill cloth because those grew cheap. A fixed basket
charges them for the firewood they no longer buy.

## What it does

**One rule, continuous in income.** Each month a town's households have a goods
budget a head, and the cost of the subsistence bundle at the town's prices. The
ratio is the town's real income `y` (the model already kept it: it is what
population growth reads). Subsistence is bought first, as before. What changes
is how each further shilling is split:

```text
share of a further shilling on need s  =  shares[s] + weight(y) x (shares_rich[s] - shares[s])

weight(y)  =  0                              up to y_ref
           =  1 - (y_ref / y)^strength        past it
```

Up to the reference income the split is the content's, so a town at or below it
buys exactly what the port's town buys, to the last bit. Past it the split leans
toward a second, richer set, slowly at first and never quite all the way. With
the game's numbers (`y_ref` 2, `strength` 3) a further shilling is halfway
between the two sets at an income of 2.5, seven eighths of the way at 4, and 96
per cent of the way at 6.

What a household spends on a need is its subsistence plus the sum of those
shillings one by one, the integral of the marginal share from subsistence up to
its income. The integral has a closed form (`engel::richer_part`), so there is
no table and no step: of an income of `y` bundles, `richer_part(y)` bundles'
worth is spent by the richer shares and the rest above subsistence by the
content's.

Three things follow from building it on the marginal share, and the tests hold
the code to each of them:

- **The budget is spent exactly.** Both sets of shares add up to the same total
  (content.rs refuses a richer set that does not), so at every income the needs
  cost what the household has, to rounding.
- **More income never buys less of anything.** Every marginal share lies between
  the two sets, so none is negative. Food is still a normal good; it is only a
  smaller part of the next shilling. (A rule that applied the leaner share to the
  whole budget above subsistence, as one of the probes did, makes food inferior
  just past the reference when the curve is steep.)
- **Demand still falls as a price rises.** A dearer need makes a household
  poorer in bundles, which leans its shillings back toward the content's shares,
  and for food that pulls the other way. It never wins: the pull is at most
  `(shares - shares_rich) x weight`, which is less than `shares`, the fall the
  port's rule already had. The test walks every need's price from a fifth to
  five times its level at ten incomes and six strengths.

**Who it touches.** Households only. Landlords keep `lord_shares`, and families
spend through their home town's landlords as before. The households' orders go
to the same town markets at the same prices, are filled from the same stocks and
paid for from the same purses, so no money is made or lost: the monthly check
on `econ::money_total` passes untouched. The wage bisection is not involved at
all: it clears each town's labour market on the demand of land and works at the
month's prices, and household demand enters only through next month's prices and
stocks, as it always did.

**In `step_month`** this is one call in place of `les` and one call that notes
what was spent on each need for the observer. With the switch off neither is
reached.

## The richer set, and where content says it

The richer set is content, like the shares it is laid beside. A content file can
give it in the services table:

```toml
[services]
names       = ["food", "warmth", "shelter", "clothing", "implements", "comfort"]
shares      = [0.28, 0.14, 0.22, 0.19, 0.07, 0.10]   # marginal shares at subsistence-and-a-bit
shares_rich = [0.05, 0.06, 0.24, 0.22, 0.07, 0.36]   # marginal shares far above it
```

It must have one share for each need, none negative, adding up to what `shares`
adds up to; anything else is refused when the file is read. `data/content.toml`
is the Python's file and has no such line, so for it, and for any file that has
none, the mechanism supplies its own set by the needs' names
(`engel::RICH_SHARES`, the line above). A need the table does not know keeps the
content's share, and the named ones are scaled to make room for it, so a new
need added without a richer share does not break the sums; but whoever adds
needs or consumer goods should write `shares_rich` out.

Why these numbers. Against the content's shares, food falls from 0.28 to 0.05
and warmth from 0.14 to 0.06: a household at five times subsistence does not
want much more bread or a much hotter fire. Shelter and clothing rise a little
(0.22 to 0.24, 0.19 to 0.22). Implements stay at 0.07, because households' tools
are what makes land workers better (`tools_mult`) and that channel should not
move for a reason that has nothing to do with it. Comfort, which is shops,
trades and service, takes what is freed: 0.10 to 0.36. The set is close to what
the landlords already spend (0.04, 0.08, 0.30, 0.22, 0.08, 0.28): a household
that grows rich comes to spend like the gentry, with less on the house.

## The chained index

When the mechanism is on, and only then (a port run's list of series does not
change), the observer keeps nine more series after the port's:

| series | what it is |
|---|---|
| `cpi_chained` | the cost of living on a chained Fisher index, 1750 = 1 |
| `real_wage_chained` | the average wage over `cpi_chained`, on the same 1750 footing as `real_wage` |
| `real_income` | households' goods budget over the cost of their subsistence, all towns together: the curve's x-axis |
| `budget_food` ... `budget_comfort` | the share of households' spending on goods that went on each need: the curve's y-axis |

The chain has one link a year. Each link is a Fisher index (the geometric mean
of the index on last year's basket and the index on this year's) between two
years' mean national prices, weighted by what households and landlords together
actually bought in each. So a year is always compared with the year before on
the baskets of those two years, and firewood stops counting when people stop
buying it. A month is read against the last complete year, so its value is known
at once; when December is in, the year's own means close the link. The first
link starts where the fixed basket does, at the last month of the warm-up
(the fixed basket's own prices and purchases, landlords' included), so both
indices stand at 1 on the same base. They are not the same number in January
1750, since the chained one already weighs that month's purchases as well as
the base's, but they start together: on today's eight islands (the sweep
named under "In today's game", below) the means of 1750 are 0.976 and 0.976,
and on no island further apart than a thousandth.

The twelve readings of a year do not average exactly to the link the year
closes on, because one month's purchases are not the year's. Measured on seed 3
over the 150 years (the port with the curve, before the other rules joined),
the two differ by 0.06 per cent in a typical year and by
0.35 per cent in the worst, so a year's mean of `cpi_chained` (which is what the
sweep's facts use) can be taken for the chain. `tests/engel.rs` works the chain
out a second time, month by month, from the prices and purchases the observer
reads.

The old `cpi`, `real_wage`, `gdp_real` and `gdp_pc` are untouched and still on
the 1750 basket; the news still reads the old ones.

## Its numbers

| | | |
|---|---|---|
| `engel.enabled` | off (port), on (game) | households' marginal shares move with real income, and the nine series are kept |
| `engel.y_ref` | 2.0 | real income up to which the content's shares hold. A value under 1 is taken as 1: subsistence is always bought by the subsistence rule |
| `engel.strength` | 3.0 | how fast the shares turn: at twice `y_ref` they are `1 - 0.5^strength` of the way. 0 is a flat curve: the port's households, with the new series recorded |
| `shares_rich` (content) | 0.05, 0.06, 0.24, 0.22, 0.07, 0.36 | the richer set |

`y_ref` is 2 because that is where the island started when it was chosen: real
income is 1.83 in 1750 on eight seeds (1.78 to 1.90) under the port, so 1750 is
the port's 1750. At 1.5 the towns of the early 1750s are already two points
larger (0.27 for 0.25) and comfort's share of budgets half as big again; at 2.5
the curve does not begin until the 1780s and the towns of 1900 are two points
smaller (0.275 for 0.298). (All of this on the port with the curve.)

Under today's game rules the island starts poorer, not richer: 1.56 bundles a
head in 1750 (1.51 to 1.60; `real_income_1750` in today's sweep). So 1750 is
still the port's rule in every town, and the curve begins to turn only when an
island's income first passes 2, which on today's eight islands is between 1761
and 1782. Nothing has been retuned for it; see the worry about `y_ref` below.

`--set engel.enabled=true` measures the mechanism alone, and
`--set engel.enabled=true --set engel.strength=0` gives the port with the new
series, which is the fair baseline for the chained wage and the budget shares.
(Checked: on seeds 1 to 8, with and without families, every one of the port's 92
series comes out of a flat-curve run the same in every year as out of a port
run.)

## What the sweeps showed

Everything from here to "In company with prices" was measured on the port with
the curve alone, before any other mechanism joined the game's rules; the
numbers are the record of why the curve was built as it is, and they cannot be
read from today's sweep. Where the game stands now is under "In today's game".

Eight seeds (1 to 8) to 1900 under the port's rules, with and without six
families. The first two tables are the sweep's own facts: each number is one
year's mean over its twelve months, averaged over the seeds, and a star marks a
fact whose ranges across seeds do not overlap between the two columns of its
pair. "Port" is the baseline; for the chained wage, real income and the budget
shares, which a port run does not record, it is the flat curve
(`engel.strength=0`), which is the same run with the series kept.

| 1750 / 1800 / 1850 / 1900 | port | curve on | port, six families | curve on, six families |
|---|---|---|---|---|
| towns: share of people | 0.264 / 0.232 / 0.231 / 0.237 | 0.264 / 0.269* / 0.282 / 0.297* | 0.264 / 0.239 / 0.257 / 0.255 | 0.264 / 0.274 / 0.309 / 0.318* |
| land (farms, pasture, woods): share of labour | 0.650 / 0.563 / 0.494 / 0.464 | 0.647 / 0.514 / 0.436 / 0.393 | 0.653 / 0.547 / 0.454 / 0.452 | 0.649 / 0.503 / 0.432 / 0.343* |
| mines (quarries, pits, mines): share of labour | 0.089 / 0.197 / 0.267 / 0.288 | 0.091 / 0.197 / 0.255 / 0.287 | 0.088 / 0.205 / 0.272 / 0.275 | 0.090 / 0.201 / 0.234 / 0.306 |
| works: share of labour | 0.261 / 0.240 / 0.239 / 0.248 | 0.262 / 0.289* / 0.310* / 0.320 | 0.259 / 0.248 / 0.274 / 0.272 | 0.261 / 0.296* / 0.334 / 0.351* |
| real income (bundles a head) | 1.83 / 3.04 / 3.43 / 3.63 | 1.83 / 3.02 / 3.46 / 3.56 | 1.84 / 2.98 / 3.49 / 3.59 | 1.84 / 3.06 / 3.50 / 3.73 |
| grain output (1750 = 1) | - / 2.39 / 4.52 / 8.93 | - / 2.07 / 3.60 / 7.25 | - / 2.28 / 4.38 / 9.12 | - / 2.05 / 4.03 / 6.47* |
| real wage, 1750 basket (1750 = 1) | - / 1.33 / 1.34 / 1.29 | - / 1.32 / 1.36 / 1.30 | - / 1.31 / 1.35 / 1.35 | - / 1.34 / 1.42 / 1.34 |
| real wage, chained (1750 = 1) | - / 1.46 / 1.56 / 1.63 | - / 1.43 / 1.60 / 1.66 | - / 1.42 / 1.58 / 1.74 | - / 1.48 / 1.69 / 1.83 |

Households' budget shares by need, in 1750 and then in 1900:

| | port | curve on | port, six families | curve on, six families |
|---|---|---|---|---|
| food | 0.422, 0.327 | 0.421, 0.271 | 0.420, 0.337 | 0.419, 0.258 |
| warmth | 0.144, 0.146 | 0.143, 0.122 | 0.144, 0.140 | 0.143, 0.119 |
| shelter | 0.147, 0.202 | 0.147, 0.210 | 0.147, 0.208 | 0.147, 0.221 |
| clothing | 0.193, 0.199 | 0.193, 0.205 | 0.194, 0.188 | 0.194, 0.201 |
| implements | 0.049, 0.054 | 0.049, 0.055 | 0.049, 0.054 | 0.049, 0.055 |
| comfort | 0.045, 0.072 | 0.047, 0.136 | 0.046, 0.073 | 0.048, 0.147 |

Reading it:

- **1750 is hardly moved.** No share of 1750 changes by more than 0.003. It is
  not nothing: the national income of 1750 is 1.83 bundles, under the
  reference, but the best-off towns are already a little past it.
- **The works gain what the land loses, and the towns follow.** By 1900 the
  works have 32 per cent of the hands where they had 25 (35 for 27 with
  families), the land 39 where it had 46 (34 for 45), and the towns hold 30 per
  cent of the people where they held 24 (32 for 26). It begins when incomes
  begin to rise, in the 1780s, and goes on to the end; the port's line is flat.
  Set seed against seed, the towns' share is higher on all eight islands in
  1800, 1850 and 1900, with families and without; the works' share on all
  eight but for one island in one year (1900, no families). The mines' share
  does not move.
- **People eat less of their rise.** Grain output is a fifth lower in 1900 (7.2
  times its 1750 level for 8.9; with families 6.5 for 9.1). That is 1.4 times
  the grain a head of 1750 where the port's households ate 1.8 times. Food
  falls to 27 per cent of budgets where it fell to 33, and comfort rises to 14
  per cent where it rose to 7.
- **The real wage is not hurt, and not helped.** On either index every
  difference is well inside the spread between seeds (in 1900 the chained wage
  runs from 1.36 to 1.86 across seeds with the curve and from 1.12 to 1.82
  without). This is a change in what is bought, not in how much can be made.
- **The index matters more than the curve.** On the port's own run the real wage
  of 1900 is 1.29 times 1750's on the 1750 basket and 1.63 times on the chained
  index (1.35 and 1.74 with families): the fixed basket hides half of the
  century and a half's gain. The cost of living in 1900 is 1.27 on the 1750
  basket and 1.01 chained. `dx-growth` found 1.36 against 1.71 on sixteen seeds
  and five-year means; this is the same gap.

### Eight more islands

The game's numbers were chosen on seeds 1 to 8, so a reviewer ran seeds 9 to 16,
which nobody had looked at, with the same settings. 1900, the port and then the
curve, with the tally seed for seed (islands where the curve's number is
greater, then less):

| | no families | | six families | |
|---|---|---|---|---|
| towns: share of people | 0.240, 0.300 | 8/0 | 0.257, 0.323 | 8/0* |
| works: share of labour | 0.250, 0.328 | 8/0* | 0.276, 0.359 | 8/0* |
| land: share of labour | 0.457, 0.391 | 1/7 | 0.433, 0.338 | 1/7 |
| mines: share of labour | 0.294, 0.281 | 2/6 | 0.291, 0.304 | 6/2 |
| food: share of budgets | 0.327, 0.267 | 0/8 | 0.331, 0.251 | 0/8* |
| comfort: share of budgets | 0.073, 0.139 | 8/0* | 0.074, 0.149 | 8/0* |
| grain output (1750 = 1) | 9.28, 7.39 | 0/8 | 9.03, 6.53 | 1/7 |
| real wage, 1750 basket | 1.32, 1.30 | 4/4 | 1.38, 1.37 | 4/4 |
| real wage, chained | 1.66, 1.66 | 3/5 | 1.78, 1.87 | 8/0 |
| grain's price swing | 0.083, 0.104 | 8/0 | 0.085, 0.107 | 8/0* |
| famine notices a run | 1.8, 3.6 | 8/0 | 0.8, 3.2 | 6/0 |

It is the same picture to the second decimal: six points more of town, eight
more of works, a fifth less grain (more than a quarter less with families), the
wage unmoved. On
all sixteen islands together the towns' share of 1900 is 0.298 for 0.238
without families (higher on every island) and 0.321 for 0.256 with them (every
island, and the ranges do not overlap). The chained wage's 8/0 with families is
one row among many; on seeds 1 to 8 it was five up and three down, and I would
not claim it.

What the second eight islands add is that the price of the cure is as steady
as the cure. Grain's price swings a fifth to a quarter harder on 31 islands of
32 (the two sweeps of sixteen, with and without families), and famine notices
about double: 3.9 a run for 2.0 without families (more on fourteen islands,
fewer on one) and 3.6 for 1.6 with them (more on eleven, fewer on three). The
people of 1900 are a quarter to a half of one per cent fewer for it.

### Across strengths

Five-year means here (1900 is 1898 to 1900, 1750 is 1750 to 1752), because one
year is a noisy thing: in a single year a seed in a slump can sit ten points
off. All for 1900; grain and wages are against 1750.

No families:

| `engel.strength` | 0 | 1 | 2 | 3 | 5 | 8 |
|---|---|---|---|---|---|---|
| towns: share of people | 0.238 | 0.259 | 0.299 | 0.298 | 0.327 | 0.339 |
| its least and greatest seed | 0.228, 0.247 | 0.239, 0.280 | 0.278, 0.322 | 0.247, 0.319 | 0.282, 0.358 | 0.325, 0.364 |
| works: share of labour | 0.249 | 0.276 | 0.325 | 0.324 | 0.358 | 0.375 |
| land: share of labour | 0.453 | 0.427 | 0.388 | 0.386 | 0.360 | 0.348 |
| mines: share of labour | 0.30 | 0.30 | 0.29 | 0.29 | 0.28 | 0.28 |
| food: share of budgets | 0.328 | 0.307 | 0.269 | 0.269 | 0.245 | 0.234 |
| comfort: share of budgets | 0.074 | 0.100 | 0.137 | 0.139 | 0.165 | 0.175 |
| grain output | 8.67 | 7.71 | 7.42 | 6.87 | 6.47 | 6.05 |
| real wage, 1750 basket | 1.31 | 1.28 | 1.36 | 1.28 | 1.33 | 1.32 |
| real wage, chained | 1.64 | 1.61 | 1.75 | 1.64 | 1.71 | 1.71 |
| grain's price swing (whole run) | 0.086 | 0.090 | 0.100 | 0.102 | 0.106 | 0.115 |
| famine notices (whole run) | 2.2 | 2.8 | 3.6 | 4.1 | 5.0 | 5.6 |

Six families:

| `engel.strength` | 0 | 1 | 3 | 8 |
|---|---|---|---|---|
| towns: share of people | 0.256 | 0.287 | 0.316 | 0.334 |
| its least and greatest seed | 0.245, 0.268 | 0.263, 0.302 | 0.264, 0.345 | 0.295, 0.360 |
| works: share of labour | 0.272 | 0.309 | 0.346 | 0.376 |
| land: share of labour | 0.426 | 0.406 | 0.368 | 0.337 |
| mines: share of labour | 0.30 | 0.29 | 0.29 | 0.29 |
| food: share of budgets | 0.334 | 0.303 | 0.268 | 0.237 |
| comfort: share of budgets | 0.074 | 0.105 | 0.143 | 0.177 |
| grain output | 8.41 | 7.77 | 6.98 | 6.34 |
| real wage, 1750 basket | 1.34 | 1.32 | 1.34 | 1.38 |
| real wage, chained | 1.74 | 1.74 | 1.80 | 1.87 |
| grain's price swing (whole run) | 0.090 | 0.096 | 0.111 | 0.115 |
| famine notices (whole run) | 2.5 | 1.6 | 3.9 | 3.5 |

The shares answer in order all the way up; the wages do not answer at all
beyond the noise. Grain's price swings harder with every step of strength, and
without families the famine notices rise in step with it (with families the
count is too jumpy on eight seeds to put in order). Strengths 2 and 3 cannot be
told apart on eight seeds. A
strength of 8 is nearly the two-piece curve of the `dx-towns` probe (a further
shilling is nine tenths of the way to the richer set at an income of 2.6) and
buys three or four more points of town than 3 does. The game has 3 because the
brief was a smooth curve, and because households that already spend like the
rich at three bundles leave the nineteenth century nothing to do; 5 or 8 are
there for whoever wants more.

### How far the towns get

**On the port's income path, to three in ten.** (This section is the port
with the curve; under today's game rules incomes do keep rising, to 8.8 bundles
a head in 1900, and the towns reach 0.42: "In today's game", below.) Real
income stops at about 3.6 bundles a head from 1850 on, and at that income the
curve has the towns at 0.30
in 1900 where the port has 0.24 (0.32 for 0.26 with six families). That is a
quarter of the way from the port to "most people live in towns".

**If incomes kept rising, to just under four in ten, and there it stops.** Two
richer islands, made only with parameters the port already has (better husbandry
and tools, which lift incomes from the 1780s on; no families; five-year means
for 1900):

| | real income 1900 | towns, flat curve | towns, curve | works, curve | land, curve | grain a head, flat and curve (1750 = 1) |
|---|---|---|---|---|---|---|
| the port's island | 3.6 | 0.238 | 0.298 | 0.324 | 0.386 | 1.75, 1.39 |
| `husbandry_boost=1.2`, `tools_boost=1.0` | 5.3 | 0.251 | 0.374 | 0.421 | 0.300 | 2.40, 1.63 |
| `husbandry_boost=1.8`, `tools_boost=1.5` | 6.5 to 7.3 | 0.260 | 0.381 | 0.430 | 0.294 | 3.20, 1.67 |

Without the curve a richer island is no more urban at all (0.25, 0.26): its
people eat 2.4 and 3.2 times the grain of 1750. With it the towns reach 0.37 at
five bundles and 0.38 at seven. By five bundles nearly every further shilling is
already split by the richer set, so the ceiling is the richer set's own: with
comfort at 0.36 of a shilling, a little under four people in ten. Most people
living in towns needs the works to hold 57 per cent of the hands (a town's size
is 0.04 plus 0.8 times the works' share of its labour), and this set tops out at
43.

**What would take it further is what the rich buy, not how fast they turn to
it.** In this content comfort is the only need met entirely by town work.
Shelter is bought mostly as quarried stone, clothing is mill cloth that needs
few hands, and the mines keep three hands in ten throughout. The `dx-towns`
probe's set, with comfort at 0.60 and less on shelter and clothing (0.04, 0.06,
0.13, 0.13, 0.04, 0.60), gives 0.34 on the port's island at strength 3 (0.30 to
0.38 by seed) and 0.38 with `husbandry_boost=1.2` alone (income 4.2); that probe
reported 0.45 for it with better husbandry and tools and a two-piece curve, and
0.50 with the tier set lower. So one half wants three things together: this
curve, incomes that keep rising past 1850, and consumer goods and building that
are made by hands in town. The last is content, which is why the richer set
lives there.

### What else moved

- **Grain's price swings a fifth harder** (`price_swing_grain` 0.102 for 0.086,
  up on seven islands of eight; 0.111 for 0.090 with families, up on all
  eight), **the farm zigzag is a sixth larger** (5.7 for 4.8; 5.7 for 5.0; up
  on seven of eight both times), **and there are more famine notices** (4.1 a
  run for 2.2, up on six islands and down on one; 3.9 for 2.5 with families,
  up on five and down on three, which is not much of a tally on its own; the
  second eight islands, above, settle it). None is starred on seeds 1 to 8,
  but all three move the same way on nearly every island in every sweep, and
  there is a reason: with less of each further shilling going on food, the
  demand for grain bends less when its price moves, so the same shortfall
  takes a bigger price rise to clear, and households that eat 1.4 times the
  grain of 1750 instead of 1.8 are nearer the floor when it comes. (A famine
  notice is a year in which a town's households, in their worst month, get
  less than 0.82 of their subsistence food: one notice for each such town
  founded more than two years before, or a single one for the year when three
  or more are hit (`people::growth`). Buying 1.8 times subsistence, they can
  be short by more than half before that happens; buying 1.4 times, by four
  tenths.) Food
  markets do behave so, but it feeds the cobweb that `dx-cycles` describes.
- The farms' area is 5 per cent less in 1900 (1,216 cells' worth for 1,282,
  less on every island; 7 per cent less with families) and the pastures' and
  woods' area is not moved. Towns cover a fifth more cells (88 for 72; 94 for
  77 with families).
- The railway is invented a few years sooner on seeds 1 to 8 (1814 for 1822
  without families, 1817 for 1820 with) and a few years later on seeds 9 to 16
  (1834 for 1832, 1829 for 1823): noise.
- Output per head in 1900, which is deflated by the 1750 basket, is a little
  lower (1.44 times 1750's for 1.47 on sixteen islands, 1.48 for 1.51 with
  families; lower on 23 islands of 32), inside the spread.
- Works, their sizes and lives, the railway's length, land rent's share of
  output and the news counts apart from famines: no change beyond noise.
- No run failed: 156 runs to 1900 with the mechanism switched on, and 96 more
  in the review (a flat curve and the game's on seeds 1 to 16, with and
  without families, and the odd settings below), every one passing the monthly
  money check.

### Odd settings

Tried to see that nothing breaks or runs away, on seeds 3 and 7 (two islands,
so read the direction and not the digits; six families but for the last
case, where the port has the towns at 0.25 in 1900 and one to four famine
notices a run):

- **A curve that is nearly a step** (`strength` 60 or 500) at `y_ref` 20 is
  the port to within noise (not to the bit: some town is past twenty bundles a
  head at some point). At 1.5 it has the towns at 0.40 in 1900 and 18 to 20
  famine notices a run. At 1 (or 0, which is taken as 1) the towns are 0.37 in
  1750 and 0.43 in 1900, there are 50 to 70 famine notices and a fifth fewer
  people. Households that give up bread for comfort the moment they can afford
  subsistence live on the edge of it. This is the far end of the trade the
  strengths table shows, and the reason not to take `y_ref` much below where
  the island starts.
- **A curve that is nearly flat** (`strength` 0.01) is the port to within
  noise at every `y_ref`.
- **A richer set that is all comfort** (`shares_rich` 0, 0, 0, 0, 0, 1 in a
  content file) puts the towns at 0.43 in 1900 on the port's incomes. **One that
  is all bread**, an Engel curve run backwards (no families), shrinks them to
  0.23 and takes food to half of budgets. Both run to 1900 with money
  conserved.

### The scripted players

`examples/bots.rs` plays the player's family by four scripts on seeds 1, 2, 3
and 7; here under the game's rules (curve on) and under the port's. The
player's rank among the six families in 1900, by seed, game and then port:
passive 2, 1, 2, 4 and 2, 1, 3, 4; landlord 1, 1, 1, 1 and 1, 1, 1, 1; founder
2, 1, 1, 4 and 2, 1, 2, 4; thoughtful 2, 1, 2, 1 and 1, 1, 1, 1. The landlord
script finishes first on every seed either way, and no script moves by more than
one place. The landlord's net worth in 1900 is about a tenth lower with the
curve (25.9m for 28.7m on average), as one would expect when less grain is
wanted, and the thoughtful player's too (22.3m for 25.5m); these are four single
runs a side and I would not lean on them. (They were run before any other
mechanism joined the game's rules, so "the game's rules" in this paragraph is
the port with the curve.)

### In company with prices

Everything above was measured on the port with the curve alone. When the curve
was merged into the game's rules the merchants' prices (`prices.md`) were
already there, so the pair was run together: seeds 1 to 6 and then 7 to 12,
six families, to 1900, the game's rules without the curve and with it
(`core-steps/engel-before`, `engel-after` and `engel-s7-12` under
`.claude/night/reports/`). No run failed. Seeds 1 to 6, with the tally seed
for seed, and beside it what the curve did alone on seeds 1 to 8:

| 1900, six families | prices | prices and the curve | tally | alone (port, then curve) |
|---|---|---|---|---|
| towns: share of people | 0.259 | 0.320 | 6/0* | 0.255, 0.318 |
| works: share of labour | 0.277 | 0.354 | 6/0* | 0.272, 0.351 |
| land: share of labour | 0.430 | 0.364 | 0/6* | 0.452, 0.343 |
| mines: share of labour | 0.293 | 0.283 | | 0.275, 0.306 |
| grain output (1750 = 1) | 9.18 | 7.48 | 0/6* | 9.12, 6.47 |
| food, comfort: share of budgets | | 0.266, 0.145 | | 0.258, 0.147 |
| real wage, 1750 basket (1750 = 1) | 1.42 | 1.45 | 5/1 | 1.35, 1.34 |
| real wage, chained (1750 = 1) | | 1.77 | | 1.74, 1.83 |
| output per head (1750 = 1) | 1.51 | 1.50 | 2/4 | |
| grain's price swing | 0.027 | 0.032 | 6/0 | 0.090, 0.111 |
| farm zigzag | 0.76 | 0.84 | 4/2 | 4.99, 5.65 |
| famine notices a run | 1.5 | 3.3 | 5/0 | 2.5, 3.9 |
| alarms a run | 2.0 | 2.2 | 2/4 | |
| town cells | 79.7 | 94.8 | 6/0* | 77, 94 |

Seeds 7 to 12 say the same, each on all six islands unless a tally is given:
towns 0.327 for 0.260, works 0.362 for 0.278, grain 7.51 for 9.14, the wage
1.43 for 1.40 (5/1), grain's swing 0.031 for 0.029 (4/2), the zigzag 0.92 for
0.81, famine notices 3.3 for 2.2 (5/1).

So in company the curve does what it does alone: six points more of town,
eight more of works, a fifth less grain, the wage where it was. The land gives
up seven points where alone it gave eleven; the mines' share, which is jumpy
from year to year, makes the difference, and I would not read anything into
it. People, works, their sizes and lives, and the iron, rail and cloth
prices' swings are not moved beyond the spread between seeds. Two things
moved on one set of islands and not on the other, and I take both for noise
until more seeds say otherwise: engine works reach half of their good's
output ten years later on seeds 1 to 6 (later on five islands) and two years
sooner on seeds 7 to 12 (sooner on four); and the railway of 1900 is 6 per
cent shorter on all of seeds 7 to 12 (1,078 km for 1,145) and 4 per cent
longer on seeds 1 to 6 (three islands up, three down).

**The worry about the cobweb is much smaller than it was.** Alone, the curve
made grain's price swing a fifth harder on top of the port's false cycle
(0.111 for 0.090) and the zigzag a sixth larger. With the merchants' markup
the fifth is still there on seeds 1 to 6, but it is a fifth of very little:
0.032 for 0.027. The zigzag is a tenth larger on ten islands of the twelve,
0.84 and 0.92 for 0.76 and 0.81, and 1.04 on the worst island: still under
the 1.2 the prices page asks for, with less room than there was. **The famines are not smaller**: notices still about double.
They come from households that eat 1.5 times the grain of 1750 where they ate
1.8 times, and so stand nearer the floor in a bad year, and steady prices do
nothing for that. It is the thing to look at again when harvests arrive.

**The island starts a little nearer the reference.** Under the merchants'
prices the households of 1750 have 1.89 bundles a head (1.94 on seeds 7 to 12,
1.98 on the best-off island) where the port has 1.84, and `y_ref` is 2. So a
few more towns are already past it in 1750: the towns' share of 1750 is 0.002
higher, on eleven of the twelve islands. That is too little to retune for,
and `y_ref` is left at 2; but one more mechanism that lifts the incomes of
1750 would put the whole island past the reference at the start, and then
`y_ref` wants moving with it.

With the curve switched off (`--set engel.enabled=false` under the game's
rules) the merged code gives every fact of every seed exactly as the code
before the merge gave it, so the curve leaks nothing into the others when it
is off in their company (`core-steps/engel-off`).

Not done here: the scripted players and the prices probe's shock were not run
again on the pair.

## In today's game

Since the pair above, the machines, the harvests, the life cycle for works, the
railways, carting, the people, the towns, the mines, the canals and the books
of account have all joined the game's rules, so none of the tables above is
what a game now plays. This is what the curve's own facts read today: the
sweep of the merged game, seeds 1 to 8 with six families, at the code of
`340f263` (`isostate-core sweep --seeds 1-8 --rules game --families 6`, its
`summary.txt` and `annual.csv` under `.claude/night/reports/merged-game-f6-l/`).
Each number is
one year's mean over its twelve months, averaged over the eight seeds; no
flat-curve run was made beside it, so it says where the game stands and not
what the curve does in it.

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| real income (bundles a head) | 1.56 | 3.30 | 5.18 | 8.75 |
| food: share of budgets | 0.410 | 0.322 | 0.259 | 0.150 |
| warmth | 0.199 | 0.107 | 0.092 | 0.083 |
| shelter | 0.133 | 0.197 | 0.208 | 0.224 |
| clothing | 0.180 | 0.199 | 0.192 | 0.219 |
| implements | 0.041 | 0.052 | 0.058 | 0.063 |
| comfort | 0.036 | 0.122 | 0.191 | 0.261 |
| real wage, 1750 basket (1750 = 1) | | 1.63 | 1.95 | 2.60 |
| real wage, chained (1750 = 1) | | 1.90 | 2.88 | 4.24 |
| towns: share of people | 0.218 | 0.266 | 0.344 | 0.419 |
| works: share of labour | 0.224 | 0.296 | 0.400 | 0.486 |
| grain output (1750 = 1) | | 2.18 | 4.37 | 11.76 |

Three things in it bear on this page. The island of 1750 is poorer than the
one the curve was tuned on (1.56 bundles a head against 1.83), and further
below `y_ref`; the life cycle for works took it from 1.87 to 1.68 and the
merge of the railways, carting and the people to 1.49 (the merged sweeps
`merged-game-f6-c`, `-d` and `-e`
in the same folder), and it has been 1.56 since. Incomes then rise far past
anything the port reached, so by 1900 nearly every further shilling is split
by the richer set and food is down to 15 per cent of budgets. And the gap
between the two indices is wider than the port's: the cost of living in 1900
is 1.20 on the 1750 basket and 0.73 chained, so the chained real wage is 4.24
where the basket's is 2.60.

## Facts added to the sweep

Under "Work and wages": `mine_share_Y` and `grain_output_Y_vs_1750`, for every
run; and for a run with the mechanism's series, `real_wage_chained_Y_vs_1750`,
`real_income_Y` and `budget_NEED_Y`.

`mine_share_Y` is what is left when the land's share and the works' share are
taken from the whole (`facts.rs`): under the port, the quarries, pits and
mines. Under the game's rules the carters and the navvies are counted among
the hands too (`carting.md`, `canals.md`) and fall into the same remainder, so
there it is not the mines' share: of today's 0.216 in 1750, about 0.125 is
carters and 0.090 the quarries, pits and mines (`carters_share_of_hands_1750`
and the `hands_*_1750` facts of the sweep above).

These are not in the table of facts in this folder's `README.md` yet, and
`crates/isostate-core/parity/check_facts.py` does not work them out a second
time (it lists them as not checked).

## What worries me

- **On its own it is a modest cure.** Six points of town by 1900, and a ceiling
  just under four in ten however rich the island gets, on the port's other
  rules. The section above says why; it should not be read as "the towns are
  fixed". (With the rest of today's game the towns reach 0.42 in 1900; how
  much of that is the curve's has not been measured on today's rules.)
- **The richer set is a judgement, not a measurement.** It follows the brief
  (less food and warmth, more comfort, a little more clothing and shelter) and
  sits near the landlords' shares. Comfort is carrying everything a richer
  household buys that is not one of the other five needs, which is a lot to ask
  of shops and trades. Whoever adds consumer goods should write `shares_rich`
  for them and look at this page's tables again.
- **Grain is a jumpier market with the curve on, and famines about double**,
  as measured above on sixteen islands with and without families. It is the
  honest other side of the cure (people who eat less of their rise have less
  to give up in a bad year) and it grows with the strength of the curve, so it
  is the thing to watch when choosing between 3 and something steeper. If the
  cycles work wants a calm grain price, it will find this pulling the other
  way.
- **`y_ref` is tied to where the island starts.** If another mechanism lifts
  the incomes of 1750 above 2 bundles, the curve is already turning in 1750 and
  1750 is no longer the port's. On the richer islands above (real income 2.25
  and 2.55 in the early 1750s) the towns of 1750 are one and two points larger.
  What has happened since is the other way: today's game starts at 1.56, so
  the curve waits until the 1760s or 1770s to begin ("In today's game"). The
  strengths and the reference were chosen on an island that started at 1.83,
  and have not been looked at again on this one.
- **The news and the viewer still read the 1750 basket.** "The real wage
  doubles" is announced on the old index, and `gdp_real` and output per head are
  deflated by it. With the curve on, the basket of 1750 is less and less what
  anyone buys. Moving the news and the charts to the chained series is a small
  change, but it changes what the player is told, so it wants a decision.
- **A month's chained value is read against last year's basket and that month's
  purchases**, and the year's link is closed on the year's means. The mean of
  the twelve months is therefore not exactly the closed link, though the gap is
  small (measured above: 0.35 per cent in the worst year of one seed). Both
  indices also price a month's purchases at the prices the month ends on, not
  the ones they were bought at, as the port's `cpi` always has.
- **National prices are town prices weighted by people**, as for the fixed
  basket, not what was paid per unit bought. A good bought mostly in cheap towns
  is priced a little too high in both indices.
- **Landlords and families have no curve.** Their shares are fixed however rich
  they grow. For the gentry that is probably right; for a family that goes from
  a farm to an empire it may not be.
- **The seed firms of 1750 are still sized by the port's rule** (`sim::seed_firms`
  asks what a town with a goods budget of 2.1 a head in money would buy, with
  the content's shares; at the content's base prices that is about 1.6
  bundles). At `y_ref` 2 the difference is nothing, since that is below the
  reference, but a much lower `y_ref` would start the island with too few
  shops.
- **Sixteen seeds for the game's numbers, eight for everything else.** A star
  says the two ranges do not overlap, which with eight seeds is strong for one
  fact and weak across 136 of them. The shares of labour and of budgets move
  far outside the spread on both sets of islands, and grain's price swing and
  the famine count move the same way on nearly all of them; the wages and the
  railway's date do not move. The strengths, the reference income and the
  richer islands were measured on seeds 1 to 8 only.
