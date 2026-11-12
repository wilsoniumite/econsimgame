# Prices: merchants who mark prices with the stock they hold

Under the port the island's prices swing on a clock of their own. Iron, rails
and machinery go from trough to peak and back every eight to ten years, about
three times from bottom to top, and the share of people working the farms
zigzags every three years. Neither is the economy answering anything: both are
one fault in the rule that sets prices (`docs/probes/dx-cycles.md` traced
it). With `prices.enabled` the merchants set prices the way a
shopkeeper does, by looking at the shelves, and stop buying what they cannot
sell. None of it is in the Python.

Everything is off by default. With `prices.enabled = false` the old branch of
the price rule runs untouched, the merchants take all a works makes, the
anchors' table stays empty, and a run is byte for byte the port's
(`tests/golden.rs`). It is on under the game's rules; `--set
prices.enabled=true` measures it alone. The code is `prices.rs`, with a few
small hooks in `econ::step_month` and one in `people::found_towns`, each marked
`prices:`.

## What was wrong

Each month the port moves a town's price of a good by how far the merchants'
stock is from two months of sales (`price_speed` times that gap, plus a small
term for the month's flow). So the price is the running sum of past stock
gaps. And the stock is the running sum of past surpluses, which follow the
price. Two sums feeding each other make a spring, and the flow term is nearly
all the friction it has: for iron about a tenth of what would stop it ringing.
It rings at a period of two pi over the square root of (`price_speed` x e /
`price_target_months`), with e the elasticity of supply and demand together:
about seven years for iron by that sum (eight to ten measured), where neither
gives much, and three for grain, where farm hands come and go freely but half
a year late.

One turn of it, in iron (seed 7, from the probe): at a price of 16 the trade
over-orders. Output passes what buyers ask for in 1812, but the price can only
come down two or three per cent a month, so for five years the works run flat
out, the merchants pay for all of it, and seven months of iron pile up. At 5
buyers ask for a third more than is made, the pile is gone in eighteen months,
a third of orders go unfilled, and the price starts back up.

## What it does

- **The stock markup.** What the old rule moved becomes a slow *anchor*, one
  for each town and good: the merchants' idea of what the thing is worth,
  learnt from shortages and gluts that last. It moves exactly as the port's
  price did. The price they ask stands off it with their stock, `anchor x
  exp(markup x gap)`, where the gap is 1 with bare shelves, 0 at the target
  stock and falls toward -1 as the stock grows. With `markup` 0.5 that is 65
  per cent above the anchor when there is nothing to sell, 22 per cent below
  it at four months of stock, never more than 39 per cent below. The price
  moves toward that by at most 12 per cent a month, and stays between the
  port's floor and ceiling. This is the friction the spring lacked: a short
  stock now raises the price at once and in proportion, and the price comes
  back as the shelves fill, where before it could only creep and overshoot.
  The damping goes from a tenth of what stops the ringing to two thirds for
  iron and well past it for grain (`prices::mark` has the sum).
- **The anchors' start.** They are set to the prices the first time they are
  read, so the warm-up runs with them. A new town takes its mother town's
  anchors with her prices; without that it would start from a stale idea of
  what things cost.
- **What a new town expects.** Farms, pits and quarries hire on the price
  they expect (`pexp`), and the port never sets it for a new town: a slot
  that has not been a town still expects the prices of 1750. With this
  mechanism on, the new town's people go on expecting what they have been
  getting: the mother town's expected price less the carriage from the new
  town's site to her, never under the price floor. That is what a load from
  there fetched the day before, so nobody's net price moves on the day of
  the founding; after it the town's own market finds its own prices and
  what people expect follows them. Added when the machines were merged; see
  "In company with the machines" below.
- **Merchants stop buying into a glut.** In the port they pay the gate price
  for everything a works makes, whether or not anyone will buy it (the open
  point in the DEVLOG about works in thin markets). Here they take all of it
  while the town's stock of the good is under `glut_months` (4) of its sales
  and shipments, less and less beyond that, and no more at `glut_stop_months`
  (8). The works sees this as short time: its utilisation is multiplied by the
  share the merchants will take, inside its demand for hands, so it hires
  fewer, makes less, buys less and pays less, and the town's wage sees that.
  Nobody is paid for iron that is not made, and no money moves that did not
  before. A stock nobody buys from at all counts as the worst glut; no stock
  is no glut, so a new trade in a new place starts freely.

The months are counted on `st.outflow`, the smoothed sales and shipments to
other towns, so a town that makes for export is not taken for a glutted one.

## Its numbers

| `prices.` | game | |
|---|---|---|
| `enabled` | true | off under the port |
| `markup` | 0.5 | how far the price stands off its anchor; 0 is the port's rule |
| `glut_months` | 4 | merchants take everything under this many months of stock; 0: always |
| `glut_stop_months` | 8 | at this many they take only `glut_floor` |
| `glut_floor` | 0 | the least share of capacity they go on taking |

A markup of 0.3 is weaker on the zigzag (1.17 against 0.80) and no better
elsewhere; 0.8 measures like 0.5. A floor of 0.2 measures like 0. I kept the
probe's 0.5 and the plain floor.

**How far the numbers can be pushed** (a review's sweeps, seeds 1 to 4 to
1900, no families). The markup has a top as well as a bottom.
The hands that follow a price arrive half a year late, and a merchant who
marks hard on his shelves is answering stock that is already on its way, so a
strong markup makes a swing of its own:

| `markup` | 0 (brake alone) | 0.5 | 0.8 | 1 | 1.5 | 2 | 5 |
|---|---|---|---|---|---|---|---|
| farm zigzag | 5.2 | 0.76 | 0.95 | 1.8 | 2.8 | 4.0 | 10.3 |
| `price_swing_iron` | 0.147 | 0.125 | 0.124 | 0.150 | 0.223 | 0.269 | 0.566 |
| `price_swing_grain` | 0.084 | 0.026 | 0.034 | 0.040 | 0.077 | 0.126 | 0.284 |
| alarms a run | 13.8 | 2.8 | 1.2 | 3.5 | 8.2 | 15 | 28 |
| famines a run | 4 | 2.5 | 3.2 | 2.5 | 13.5 | 42 | 109 |

So 0.5 sits in the middle of a window that runs from about 0.3 to 0.8, with a
factor of two to spare before it turns on itself. A negative markup (dearer
as the shelves fill) is nonsense and wrecks the island: industry goes within
fifty years. (Two of its four runs then stop at the money check in the 1880s.
Nothing leaks: `econ::step_month` issues new money as a share of the money it
counts while the month's wages are between purses, on that island the count
is below zero, so the stock shrinks until it is negative, and `sim::run_month`
allows a drift of a millionth of the stock or of 1, whichever is larger,
which for a negative stock is a millionth of a coin in forty million. That is
the port's, not this mechanism's.) So a negative markup is refused where the
settings are read, by every road (a file, an override, the sweep's `--set`),
in one sentence that names it, and so are a negative `glut_months`, a
`glut_stop_months` under `glut_months` (equal is allowed: a brake that drops
at once), a `glut_floor` outside 0 to 1, and any of the four that is not a
number, whether the switch is on or off (`PriceSettings` in `prices.rs`; a
`nan` or `inf` in a file is refused before it gets there, as "invalid type:
null", since a file is read by way of JSON, which has no such numbers).
The brake has to stand well above `price_target_months` (2), the
stock merchants mean to hold: with `glut_months` 0.5 and `glut_stop_months`
0.6 every works is on short time for ever (utilisation 0.11, no railway, 439
thousand people in 1900), and with 0.5 and 2 the wage of 1900 is 1.08 times
1750's. A brake that starts at 0.5 and stops at 40, or one that drops from all
to nothing at 4 months, measures like the game's. (Those runs were made before
a brake at or under the floor was refused; see "The brake leans on two
numbers" under What worries me. A brake starting at 0.5 is refused from a
file; the sweep's `--set` does not check it yet, and a test can build one in
code, as `tests/prices.rs` does.)

## The wage still finds its root

`econ::step_month` finds each town's wage with Newton steps kept inside a
bisection, and both assume that the demand for hands falls as the wage rises.
The merchants' share is read from last month's stock and sales before the wage
is sought, so within the month it is a fixed number between 0 and 1 for each
works. A works' demand is the old one times that number: still falling in the
wage, and zero where the number is zero. The slope the Newton step uses
changes with it: utilisation is `take x s` with `s` the old sigmoid, so its
slope is `take x s (1 - s)`, which is `u (1 - u / take)`; with a take of 1
that is the port's line to the bit. `tests/prices.rs` holds the shortcut to
the plain bisection, every midpoint evaluated, wage for wage and bit for bit
over three years, with a brake set harsh enough to bite on over a hundred
works-months, with the game's brake, and with six families.

That test proves the root, not the slope: a wrong slope would only cost
steps. So a review checked the slope itself once, with a line of scaffolding
that is not in the code: at three wages across each town's bracket, every
month of the same three years, the slope the Newton step is given against a
central difference of the demand it belongs to. Over 4,000 evaluations in
towns with works on short time they agree to 3 parts in 100 million at worst
(in towns with none, to 4 in a billion), and the slope is never positive.

## What the sweeps showed

Seeds 1 to 8 to 1900, the port against the port with `prices.enabled=true`,
without families and with six. The first table is the sweep's scorecard
(`sweep-prices-base`, `sweep-prices`, and the same ending `-f6`, under
`.claude/night/reports/`); the rest is `examples/prices_probe.rs`, which
samples every month (`prices-probe/` in the same place; a review's second
copies of both are `sweep-prices-review*` and `prices-probe-review/`). A swing is the
standard deviation of the yearly change in the log of the annual mean. A star
means every seed with the mechanism is below (or above) every seed without.

| | no families | | six families | |
|---|---|---|---|---|
| | port | prices | port | prices |
| `price_swing_iron` | 0.219 | 0.133 * | 0.202 | 0.127 * |
| `price_swing_rails` | 0.204 | 0.182 | 0.200 | 0.149 |
| `price_swing_grain` | 0.086 | 0.027 * | 0.090 | 0.026 * |
| `price_swing_cloth` | 0.083 | 0.026 * | 0.084 | 0.023 * |
| `farm_zigzag` | 4.82 | 0.80 * | 4.99 | 0.78 * |
| `news_alarm` | 16.0 | 2.4 * | 15.0 | 2.0 * |
| `news_steep` | 45.4 | 24.1 * | 44.8 | 24.5 * |
| `real_wage_1900_vs_1750` | 1.29 | 1.42 | 1.35 | 1.42 |
| `works_1900` | 4,512 | 3,821 * | 6,973 | 6,310 |
| `works_utilisation_1900` | 0.81 | 0.92 | 0.84 | 0.92 |

**Checked again, and out of sample.** A review reran the four sweeps and the
probe on a build of its own and got these numbers again, digit for digit. It
then ran seeds 9 to 16, which nothing here was tuned on
(`sweep-prices-review-*-s9` in the same place):

| seeds 9 to 16 | no families | | six families | |
|---|---|---|---|---|
| | port | prices | port | prices |
| `price_swing_iron` | 0.211 | 0.121 * | 0.206 | 0.122 * |
| `price_swing_rails` | 0.189 | 0.152 | 0.178 | 0.141 |
| `price_swing_grain` | 0.083 | 0.027 * | 0.085 | 0.029 * |
| `price_swing_cloth` | 0.085 | 0.028 * | 0.084 | 0.028 * |
| `farm_zigzag` | 5.00 | 0.79 * | 4.69 | 0.80 * |
| `news_alarm` | 15.5 | 2.4 * | 16.0 | 1.8 * |
| `news_steep` | 43.2 | 24.0 | 44.2 | 24.1 * |
| `real_wage_1900_vs_1750` | 1.32 | 1.31 | 1.38 | 1.41 |
| `works_1900` | 4,423 | 4,016 | 6,832 | 6,174 * |
| `works_utilisation_1900` | 0.86 | 0.91 | 0.87 | 0.93 |

The cure holds as it stood: the same sizes, the same stars. The higher wage
of 1900 does not: on these seeds without families it is higher in three and
lower in five. None of the 32 runs with the mechanism on (sixteen seeds, with
and without families, each to 1900) failed the monthly money check.

By half-century (1750-99 / 1800-49 / 1850-1900):

| swing of | port | prices | port, six families | prices, six families |
|---|---|---|---|---|
| iron price | .189 / .225 / .227 | .113 / .139 / .145 | .181 / .204 / .208 | .120 / .133 / .128 |
| rails price | .169 / .215 / .222 | .144 / .206 / .189 | .167 / .215 / .208 | .139 / .167 / .152 |
| machinery price | .178 / .235 / .269 | .144 / .116 / .198 | .181 / .225 / .260 | .168 / .136 / .172 |
| cloth price | .099 / .078 / .078 | .030 / .024 / .025 | .091 / .079 / .087 | .029 / .019 / .020 |
| grain price | .065 / .080 / .104 | .025 / .020 / .033 | .074 / .081 / .105 | .025 / .019 / .032 |
| iron output | .173 / .183 / .192 | .072 / .056 / .051 | .168 / .172 / .186 | .077 / .055 / .047 |
| grain output | .082 / .119 / .170 | .019 / .014 / .031 | .097 / .116 / .159 | .020 / .016 / .029 |
| real output | .030 / .040 / .053 | .014 / .011 / .015 | .029 / .042 / .053 | .016 / .010 / .013 |
| farm zigzag | 3.3 / 5.0 / 6.1 | 0.7 / 0.7 / 1.0 | 3.8 / 5.1 / 6.0 | 0.6 / 0.7 / 1.0 |

Cloth, grain, both outputs, real output and the zigzag are clear of the spread
between seeds in every half-century (real output before 1800 with families
aside). Iron's price is clear in the first half-century (and the second with
families) and below on average in the others. **Rails and machinery prices
are not clear of the spread anywhere**: they are lower on average in every
half-century, by anything from 4 to 50 per cent, and I would not claim more.
What is left in them is a real cycle of capacity, and the railway's orders
arriving in lumps.

| a run, on average | port | prices | port, six families | prices, six families |
|---|---|---|---|---|
| alarms; verdicts; depressions among them | 16.0; 15.6; 5.5 | 2.4; 2.4; 0.4 | 15.0; 14.8; 5.1 | 2.0; 2.0; 0.4 |
| booms; slumps | 11.6; 14.3 | 4.3; 7.3 | 9.6; 15.0 | 4.4; 7.1 |
| gluts; shortages | 6.3; 24.6 | 5.6; 9.1 | 5.9; 25.2 | 6.3; 9.0 |
| years real output falls over 2 / 4 per cent | 23.0 / 12.0 | 3.0 / 0.9 | 21.5 / 11.5 | 2.5 / 0.6 |
| iron asked for and not got | 8 / 12 / 13 % | 1 / 2 / 3 % | 8 / 11 / 12 % | 1 / 2 / 2 % |
| the worst year's share of iron orders filled | 0.37 | 0.74 | 0.39 | 0.75 |
| iron stock, months of sales | 3.0 / 3.4 / 3.3 | 2.0 / 2.0 / 2.0 | 3.2 / 3.0 / 2.9 | 2.1 / 2.0 / 2.0 |
| made where over six months' stock stood: iron, rails, machinery | 10, 15, 16 % | 0.1, 0.6, 0.5 % | 7, 14, 16 % | 0.0, 0.2, 0.3 % |
| wage in baskets, 1750s / 1790s / 1840s / 1890s | 2.87 / 4.30 / 4.67 / 4.94 | 2.76 / 4.31 / 4.65 / 5.32 | 2.89 / 4.44 / 4.96 / 5.43 | 2.79 / 4.43 / 4.94 / 5.67 |
| iron output a month, 1790s / 1840s / 1890s | 1,966 / 4,506 / 8,204 | 2,032 / 4,484 / 9,028 | 2,228 / 4,962 / 8,794 | 2,219 / 4,995 / 9,543 |
| cloth output, the same | 4,609 / 32,507 / 63,022 | 4,948 / 36,390 / 71,902 | 4,951 / 45,478 / 90,344 | 5,493 / 47,473 / 98,513 |

The observer still has something to say: nearly two booms, gluts, shortages
or slumps a decade, and a downturn about every sixty years where it called one
every ten. The wage in baskets (the town wage over the cost of its subsistence
bundle, so it compares levels between rules, which the scorecard's ratio to
1750 does not) is the same until the 1840s and 5 to 8 per cent higher in the
1890s; iron and cloth are about a tenth higher by then. Those gains are inside
the spread between seeds; the scorecard's real wage of 1900 is higher in six
seeds of eight, and in seven with families.

**Does it still answer real change?** As fast as before, as far as eight seeds
can tell. Years from discovery to half of the good's output, port then prices:
coke iron 8.0 and 7.8 (with families 8.4 and 8.0), mills 11.6 and 10.3 (11.1
and 9.8), rolling mills 18.5 and 15.3 (19.0 and 19.0). Railway open ten years
after the locomotive, 296 km and 281 (387 and 406); twenty-five years after,
1,000 and 1,045 (1,057 and 1,065). All of these differences are inside the
spread.

**A shock.** At the end of 1815 every iron works loses 30 per cent of its
capacity, and the run is followed against the same seed left alone (mean over
8 seeds of the log of the ratio; years 1 to 20; no families).

| | port | prices |
|---|---|---|
| iron price | +.10 +.35 +.65 +.98 **+1.08** in year 5, then down to **-.66** in year 13 and up again to +.36 in year 19 | +.30 +.68 **+.99** in year 3, back to +.03 in year 7, **-.38** in year 11, within .26 after |
| iron capacity | back to par in year 7, overshoots by .17 | back to par in year 5 or 6, overshoots by .12 |
| iron output | lowest -.55 in year 4, par in year 6, then +.15 and -.25 as the echo passes | lowest -.63 in year 3, par in year 5, within .06 after |
| real output | lowest -.10 in year 4, par in year 7, -.05 again in year 17 | lowest -.12 in year 3, par in year 7, flat after |

So a real loss shows in the price at once and about as high; the trade
rebuilds a year or two sooner; and the echo, the glut ten years on and the
second shortage after it, is halved in the price and gone from output. The
bottom is as deep. With six families the paths are the same (price peak +.94
in year 3, undershoot -.43; the port's undershoot there is -.81).

**Which part does what** (no families). The markup alone does nearly all of
it: zigzag 0.77, alarms 2.4, iron output swing .079 / .049 / .054. The brake
alone (`markup` 0) is a partial cure for industry and none for the farms: the
iron price swings .133 / .149 / .152, but iron output still .147 / .126 /
.167, the zigzag is 5.2, there are 13.6 alarms a run, and 6 to 10 per cent of
iron orders still go unfilled. On top of the markup the brake's work is
narrow: iron made into a long stock was already gone, but rails and machinery
fall from 4.6 and 3.3 per cent of output to 0.6 and 0.5, and after the shock
the price undershoots by .38 where the markup alone gives .61. It is in
because it closes the hole the DEVLOG names, not because the scorecard needs
it.

**How often the brake bites.** Seldom. Over a run the merchants turn away 0.3
per cent of the iron works' capacity, 3.1 of the rail mills' and 1.9 of the
machine shops' (with six families 0.2, 1.2 and 1.2). The worry was the towns
that make for others: a town can ship at most `trade_cap` (0.3) of its stock
a month, so one that only ships a good never holds less than 3.3 months of
it, and the brake starts at 4. Towns that ship out more of a good than they
sell at home hold 2, 9 and 7 per cent of those three trades' capacity, and
there the merchants turn away 0.7, 3.9 and 0.6 per cent: about what they do
everywhere. The brake is not falling on the exporters.

**The scripted players** (`examples/bots.rs`, seeds 1, 2, 3 and 7, run before
any other mechanism joined the game's rules). Worth in 1900, mean of the four
seeds, port then prices: passive 20.3m and 23.3m, landlord 28.7m and 31.1m,
founder 20.3m and 22.2m, thoughtful 25.5m and 28.1m. Ranks in 1900: the
landlord is first in every seed either way; the thoughtful player is first in
all four under the port and first in three and fourth in one (seed 7) with
prices; passive 2, 1, 3, 4 both ways. With six families and every default
taken, the player's rank over eight seeds is 2.4 and 2.5, and the families
hold 56 and 60 per cent of the works, 26 and 26 of the rent, 12 and 12 of the
money. The order of the strategies does not change, and on these four seeds
each ends a little richer.

## Tried and left out: the pipeline

The probe's fourth suggestion was that someone founding or enlarging a works
should count the capacity already on order: expect the price times `(market x
1.05 / (market + on order))^eta`. It was built into `firms::entry`, `review`
and `spawn_sister` as a trial and run at eta 0.5 and 1 on top of the above,
with the shock. It did not earn its place. Iron's price swing was .113 / .139 / .145
without it, .120 / .119 / .125 at 0.5 and .108 / .129 / .144 at 1; alarms 2.4,
3.1 and 2.5; all inside the spread. After the shock capacity overshoots by
.12, .12 and .10. And it costs something: rolling mills take 15, 20 and 26
years to reach half of rails. So it is not in the code.

The works mechanism (`docs/rules/works.md`, another branch) gives each trade
in each town a year's room for new capacity, taken first by the works already
there. That does the pipeline's job from the other side, by limiting what can
be ordered in a year where the pipeline only talked founders out of it, and I
would expect it to take the .12 overshoot down where the pipeline could not.
The two touch different code (this one the price and the wage in `econ.rs`,
that one entry and growth in `firms.rs`) and should merge without argument.
What I have not measured is the pair together: see the worries.

## An open question for Stella: the flow term cannot see exports

The price rule's flow term compares what a town's own buyers asked for with
what arrived (made there or brought in). It does not count what the town
ships out. So a town that makes iron for the island reads every month as a
glut, and its price is leaned on for as long as it exports. This is in the port, and it is in
the anchor here, which moves by the port's rule.

The probe counted shipments as wanted, on top of a markup of 0.5 and a stock
cap of 4 months. The real wage rose by 11, 21 and 21 per cent in the three
half-centuries, coal output by 28 to 48 per cent, the farm share of the 1840s
fell from 0.51 to 0.47, and downturns nearly vanished (0.3 alarms a run). But
stocks piled up in thin markets (18 months of rails at the worst twentieth of
town-months), and in the one test with families (4 seeds, in a set that also
had harvests and a choice of technique) the families' share of all money went
from 10 per cent to 66. So it is a real distortion, and mending it is a
rebalancing of the whole game, not a repair. I left it alone, as asked. If it
is taken up, it wants its own switch, a look at where the extra money comes
to rest, and the bots.

## In company with the machines

When the machines (`machines.md`) were merged into the game's rules, one run
in six went badly wrong, and the cause was a line the port does not have. It
is mended here because this mechanism already carried a new town's idea of
what things are worth.

**What happened.** Seed 6, six families: a market town is founded at the end
of 1888. Its farms hire on the price of grain they expect, and nothing had
ever set that for the new town, so it was still 1750's: 0.78, on an island
where the machines had brought grain down to 0.27. On a farm the hands
wanted go up with between the cube and the fourth power of the net price.
The town had 4,300 workers and its farms asked for 150,000. The port's labour market lets the wage rise no more than
twofold in a month, and when demand is still above the people at that wage it
puts everyone asked for to work and pays them. So the town's wage doubled
month after month, its farms raised as much grain as the rest of the island
together in the first month, the wage bill came out of the landlords' rents
(the island's rent share for 1889 is -1.9), and households ended up holding
money the landlords had held. Households spend nine tenths of what they earn
where landlords spend three quarters, so the same money turns over faster in
their hands; that is how I read what followed, without having followed each
purse. The island's money did not change and its prices rose 3.3 times in
five years and stayed up for the rest of the run; the real wage fell from 2.5
to 1.6 on the way
(`core-steps/machines-after-merge` and `machines-seed6-run` under
`.claude/night/reports/`).

The port has such years in a smaller way: its "one wild year" (seed 3 in
1753: the wage doubles, the rent share is -0.7) comes with a founding too.
In the 1750s a stale price is three years stale and does little; what the
machines changed is how far prices travel from where they started, so that
a town founded late remembers a different island.

**What was changed.** `prices::found_town` now sets what the new town's people
expect ("What it does", above). Two rules were tried on 48 seeds with six
families and the whole game. A wild year here is one in which the island's
average wage rises by more than a quarter or the rent share is under -0.05:

| what a new town expects | runs of 48 with a wild year | rent share under -0.4 | the worst, and prices five years on |
|---|---|---|---|
| 1750's prices (the port) | 3 | 2 | -1.94, prices 3.3 times (seed 6, 1889) |
| the mother town's, whole | 4 | 2 | -1.80, prices 3.0 times (seed 3, 1756) |
| the mother town's less the carriage | 4 | 0 | -0.22, prices 1.4 times (seed 28, 1753) |

Her prices whole were worse than they look: towns are founded far from any
market, where something heavy is dug, and the mother town's price for coal is
the price in a town far from the pits. Seed 3's new coal town expected 1.74
for coal, with no carriage to pay any more, and put 12,000 colliers to work
out of 2,600 people. Taking the carriage off gives each good the price a
load of it from that spot fetched the day before, so the founding itself
moves nobody's net price, and the four years left are smaller ones (the wage
up by a quarter to three quarters for a year, prices a tenth to two fifths
higher five years on). Two of the four are not foundings at all; see the
worries.

Without the machines (the game's rules as they stood, sixteen seeds, six
families) the same change moves no fact clear of the spread between islands
(none of 166 starred), and the two wild years there were go (seed 11 in 1837,
rent share -0.44, and a small one on seed 2). Towns founded after 1840, in 48
runs of the whole game: five with the port's rule, nine with this one.

## In company with people

The game's rules for people (`people.md`) found towns by a rule of their
own and move households with their cells every year, so the line above is
no longer the only place a town's prices are set from outside the market.
Two things keep the anchors in step. `peopling::open_town` calls
`found_town` as the port's founding rule does, so a new town has its
mother's anchors and not those its empty slot was given in 1750. And
wherever `peopling::move_with_cells` sets a town's price to the average of
its own and what an arriving district was used to, `anchor_follows` moves
the anchor in the same proportion: the price stands as far off its anchor
as it did, which is the stock speaking, and the stock is what it was (a new
town's is its share of its mother's, as many months of sales as hers).
Without it a new town's prices were walked back at 12 per cent a month
toward the mother town's idea of what things are worth.

What a new town's producers expect is set twice, in order: `found_town`
takes the carriage to the mother town off her expected prices, and people's
average then replaces that for every good the district makes or buys.
People's is the finer rule (cell by cell, and the buyers' side too); the
line above is left standing for goods nobody there trades, and for a game
with people switched off.

## What worries me

- **A town can hire more hands than it has.** This is the port's, and the
  change above only removes one thing that sets it off. Each month a town's
  wage may rise or fall twofold at most (`econ::step_month`, the bracket of
  the bisection). If the hands wanted are still more than the people at the
  top of the bracket, all of them are put to work and paid, and the wage
  bill beyond what the land earns comes out of the landlords' rents. A new
  town, a new railway, anything that lifts the net price at the pithead or
  the field at a stroke can do it. Under any rules somewhere between one
  run in four and one in twelve has such a year, most of them small (the
  port: two of eight, four of eight with families); the two left here that
  are not foundings fall in 1825 and 1836, the railway years, and I did not
  open them. The honest cure is to ration:
  where demand exceeds the people at the highest wage the month allows,
  scale every employer in the town down to the people there are. That is a
  rule of its own with its own switch, and belongs to whoever takes up
  people and towns.
- **Steel.** Under the port the converter is built in 3 seeds of 8 (2 with
  families), never before 1871 and never to any size. With prices on it is
  built in none. The converter clears its hurdle only in the years when steel
  spikes (`docs/probes/dx-growth.md` has it clearing in 4 per cent of years),
  and the spikes were the false cycle. Nothing real is lost, but "steel
  never arrives" is now flat, and whatever makes the converter pay has to do
  it on steady prices.
- **Grain is too quiet.** A price swing of 0.027 a year is flatter than any
  farm economy; the port's 0.086 was the wrong noise, and this is none. The
  harvest mechanism is meant to put the right noise back, and the probe found
  that the same harvest shocks ring badly under the port's rule and settle
  under this one. Not measured here together.
- **Fewer works.** A seventh fewer stand in 1900 without families (every
  seed), a tenth fewer with six; they are a little larger and run fuller
  (utilisation 0.92 against 0.81). Fewer false peaks call fewer founders in.
  That is right, but anything that counts works will see it.
- **In company with the works mechanism** both hold back new capacity at a
  peak: this one by ending the peaks, that one by rationing a year's room.
  After a real loss the price here rises faster (+.30 in the first year
  against +.10), which is the signal to rebuild; if the room then rations the
  rebuilding, the shortage could last longer than either alone. The shock
  test in `prices_probe` (`--shock`) is the thing to run on the merged rules.
  It was run when the two were merged (`works.md`, "In company with the rest
  of the game"): the shortage is no longer, capacity is back a year sooner,
  but the rebuilding overshoots twice as far (18 per cent for 9) and rings
  for twenty years. The pair also shows a building cycle that neither has
  alone: mills ordering a year's room at once, and the price of timber,
  marked on a stock those orders empty, trebling every three or four years.
- **Rails and machinery** still swing by 0.15 to 0.2 a year. That is a real
  capacity cycle, and for rails the lumpiness of railway building; this
  mechanism does not touch either.
- **A stock nobody buys from stops the works dead** (floor 0). I saw no sign
  of a trade strangled by it: new methods spread as fast as before. But a
  town whose only buyer of a good goes away keeps its last stock and makes no
  more until someone asks, where the port went on making it for the
  merchants' account.
- **The brake is on works only.** Farms, pits, quarries and woods still sell
  the merchants everything they raise, whatever stands in the warehouse. The
  markup steadies grain well enough that I saw no need, and I did not count
  how much coal or timber is raised into a long stock.
- **The warm-up runs with the mechanism**, so 1750 is not the port's 1750:
  the farm share is 0.636 against 0.650 and the urban share 0.252 against
  0.264 in every seed, and the 1750s wage in baskets is 4 per cent lower.
  Ratios to 1750 flatter the mechanism a little for that reason; the wage in
  baskets does not.
- **Money buys more in 1750.** The port comes out of its warm-up on a price
  spike of its own making; with the markup it does not. In 1750 grain is 0.64
  against 0.96, iron 18 against 30, machinery 535 against 983 and the wage
  3.5 against 5.0, while the island holds the same money. What a wage buys is
  about the same, but anything fixed in money at the start is worth half as
  much again: the families' first purses, most of all. And `gdp_real` is in
  each run's own prices of 1750, so its level cannot be set beside the port's
  (it reads a quarter lower all century); its changes can.
- **The brake leans on two numbers it does not own.** It starts at 4 months
  because merchants aim at 2 (`price_target_months`) and a town that only
  ships holds 3.3 (`1 / trade_cap`). The code now holds this:
  `prices::brake_floor` is the larger of the two, and a parameters file or
  override that leaves `glut_months` at or under it is refused in a sentence
  that names the three settings (`prices::check`, called by every reader in
  `params.rs`; `tests/prices.rs`,
  `the_brake_moves_with_the_two_numbers_it_leans_on`). So a `trade_cap` of
  0.2 alone is refused under the game's rules, and goes through with
  `glut_months` 6 beside it. Two roads are not checked: a `Params` built in
  code (on purpose, so a test can try a harsher brake) and the sweep's
  `--set`, which reads each setting by its own route (`sweep::set_field`)
  and does not call `Params::check` yet.
- **Famines.** Without families there are more: 2.0 a run under the port and
  3.2 with prices over sixteen seeds, up in eleven and down in three. With six
  families there are not (1.6 and 1.7). In the one run opened (seed 13) they
  fall in the same years under both rules, 1853 to 1858, when farm machinery
  has just been found and its price takes the wage with it (the machinery
  famine of `docs/probes/dx-cycles.md`), so I read this as the same trouble
  landing on a few more towns, not a new one. It is inside the spread (the
  port has eight in seed 3) and I did not look further.
- **Sixteen seeds for the scorecard, eight for the rest.** The shock paths are
  means of eight pairs of runs that draw different random numbers after the
  shock, so a single year's figure is rough; the shape is what to read.
- **Not looked at**: the commons rent regime, years after 1900, the viewer's
  Markets panel (it draws prices, and would have room for the anchor beside
  them), the Python.

## Measuring it again

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --out DIR-base
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --set prices.enabled=true --out DIR --compare DIR-base
cargo run --release -p isostate-core --example prices_probe -- --out base.json --shock
cargo run --release -p isostate-core --example prices_probe -- --out prices.json --shock --set prices.enabled=true --compare base.json
```

and the same with `--families 6`. `--set prices.glut_months=0` gives the
markup alone, `--set prices.markup=0` the brake alone. It has worked if the
farm zigzag is under 1.2, the iron price swings under 0.16 a year and iron
output under 0.09 in every half-century, iron stock stands near 2 months with
under 4 per cent of orders unfilled, and coke iron passes half of output
within 10 years of its discovery.
