# Prices: merchants who mark prices with the stock they hold

Under the port the island's prices swing on a clock of their own. Iron, rails
and machinery go from trough to peak and back every eight to ten years, about
three times from bottom to top, and the share of people working the farms
zigzags every three years. Neither is the economy answering anything: both are
one fault in the rule that sets prices (`.claude/night/reports/dx-cycles.md`
traced it). With `prices.enabled` the merchants set prices the way a
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

## What the sweeps showed

Seeds 1 to 8 to 1900, the port against the port with `prices.enabled=true`,
without families and with six. The first table is the sweep's scorecard
(`sweep-prices-base`, `sweep-prices`, and the same ending `-f6`, under
`.claude/night/reports/`); the rest is `examples/prices_probe.rs`, which
samples every month (`prices-probe/` in the same place). A swing is the
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

## What worries me

- **Steel.** Under the port the converter is built in 3 seeds of 8 (2 with
  families), never before 1871 and never to any size. With prices on it is
  built in none. The converter clears its hurdle only in the years when steel
  spikes (the growth probe has it clearing in 4 per cent of years), and the
  spikes were the false cycle. Nothing real is lost, but "steel never
  arrives" is now flat, and whatever makes the converter pay has to do it on
  steady prices.
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
- **Rails and machinery** still swing by 0.15 to 0.2 a year. That is a real
  capacity cycle, and for rails the lumpiness of railway building; this
  mechanism does not touch either.
- **A stock nobody buys from stops the works dead** (floor 0). I saw no sign
  of a trade strangled by it: new methods spread as fast as before. But a
  town whose only buyer of a good goes away keeps its last stock and makes no
  more until someone asks, where the port went on making it for the
  merchants' account.
- **The warm-up runs with the mechanism**, so 1750 is not the port's 1750:
  the farm share is 0.636 against 0.650 and the urban share 0.252 against
  0.264 in every seed, and the 1750s wage in baskets is 4 per cent lower.
  Ratios to 1750 flatter the mechanism a little for that reason; the wage in
  baskets does not.
- **Eight seeds.** The shock paths are means of eight pairs of runs that draw
  different random numbers after the shock, so a single year's figure is
  rough; the shape is what to read.
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
