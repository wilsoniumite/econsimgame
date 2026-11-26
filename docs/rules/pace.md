# Pace: works built over a builder's year

**Off under both sets of rules: a cure on trial.** It was built to end the timber-and-brick cycle that the trace found ([tr-timber.md](../probes/tr-timber.md)), and is left off for a checker to try before it is switched on in `game_params()`. With it off nothing moves: the port's golden test and the game's hold their hashes. With it on, islands 1 to 16 to 2050: the standing check twelve of twelve inside, every fact of 1900 and before to the last digit; the three growth asks met (the linked wage 1.27 per cent a year from 1900 to 1970 against 1.25, 2050 over 1970 1.14 against 1.09, the land's share of hands 0.117 lower in 2050 than in 1900 against 0.112).

The code is `pace.rs`, with one hook in `econ::step_month` marked "pace". Not in the Python.

## What was wrong

A works that orders plant (to be founded, or to grow at the year's review) asks the merchants every month for a quarter of what is still to come, so most of its timber and bricks are asked for in the three or four months after the review. Before 1900 the households bought two thirds of the timber and half the bricks and gave way when the works came to buy. After 1900 they leave both: by 1950 the works being built take all the bricks, and the cloth mills, whose plant is 10 timber a unit, order nine tenths of the timber that goes into plant. One year's orders can be more than the woods cut in a year, asked for in a season, and nobody else in the market gives way. Timber's price swings fivefold every four or five years and bricks' with it (the trace, sections 1 to 3).

## What it does

From January of `from` (1901), a works asks each month for `1 / months` of the plant still to come (`months` 12): the plant comes in over a building year, as a builder's season brings it, and the merchants see the order book a month at a time. Nothing else changes: what each works orders and when, the year's room, the review and its hurdle, the merchants' prices, the woods. The capacity still arrives in proportion to what has been paid for, so a works with a mill half delivered has half a mill. A works counts as building while more than a twentieth of its capacity is still on order (`works.built_share`), which now lasts longer, so it orders again a little later.

## Its numbers

`PaceParams`, in `pace.rs`. Off by default and in `game_params()`; `PaceParams::on()` is the third column.

| | default | game | on | |
|---|---|---|---|---|
| `enabled` | false | false | true | works ask for their plant over `months` from `from` |
| `months` | 12 | 12 | 12 | each month a works asks for one part in this many of what is still to come (the port's is a quarter, as if 4) |
| `from` | 1901 | 1901 | 1901 | the first year whose months are paced; the played century is untouched |

`--set pace.enabled=true` tries it in a sweep, and `--set pace.months=6,12,24` a pace.

## What the sweeps showed

Islands 1 to 16, six families, to 2050 (`isostate-core sweep --seeds 1-16 --rules game --families 6 --end-year 2050 --set pace.enabled=false,true --standing`). The swing is the standard deviation of the yearly change in the log of the price over the wage, 1910 to 2050.

| | off | on |
|---|---|---|
| timber's swing | 0.532 | 0.398, lower on 16 of 16 |
| bricks' swing | 0.493 | 0.276, lower on 16 of 16 |
| sawn timber's swing | 0.597 | 0.403, lower on 16 of 16 |
| timber in months of the wage, 2030-2050 | 1.35 | 0.85 |
| the real wage's worst year | -0.457 | -0.334 |
| real income in the dearest fifth of years, against the years about them | -0.4 per cent (worst -24.4) | +0.4 per cent (worst -9.1) |
| the linked wage, 2030-2050 | 15.56 | 16.23 |
| building work a head, 2030-2050 | 4.08 | 4.19 |
| people in 2050 | 1,162,245 | 1,153,271 |
| alarms and verdicts in the news | 13.4, 12.8 | 9.8, 9.3 |

## What worries me

- **It does not end timber's cycle.** Timber's swing falls by a quarter and bricks' by nearly half. The rest is the woodmen, who hire on a price they expect months late and move a seventh of the way a month: with the land's hands and expectations quick as well, timber's swing is under half on eight islands (the trace, section 3). That wants a woodman's own pace in the labour market, which this rule does not touch.
- **The machines' plant is not paced.** The orders for the machines that take tasks (`tasks::orders`) still ask for a quarter a month. They are electrical goods and machinery, not timber or bricks, and were left as they are.
- **Eight islands for the arms, sixteen for the rule.** The rule's own numbers are on sixteen; the arms that say why are on eight.
