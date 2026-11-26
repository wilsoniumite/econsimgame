# Pace: works whose plant comes in over a year

**On under the game's rules from 1901, since its check of 26 November 2026; off under the port's.** It damps the timber-and-brick cycle that the trace found ([tr-timber.md](../probes/tr-timber.md)): timber's price, and bricks' after 1950, swinging threefold to fivefold every few years in the century after 1900. The trace built it and left it off; the check tried the trace's claim on sixteen islands it had not used, found the cause and the effect of the cure standing and the trace's account of how the cure works wrong, and switched it on (the trace's page, "The check"). With it on, islands 17 to 32 to 2050, on the tree of 26 November: the standing check twelve of twelve inside, every fact of 1900 and before to the last digit; the three growth asks met (the linked wage 1.31 per cent a year from 1900 to 1970 against 1.30 with it off, 2050 over 1970 1.20 against 1.17, the land's share of hands 0.125 lower in 2050 than in 1900 against 0.121).

The code is `pace.rs`, with one hook in `econ::step_month` marked "pace". Not in the Python.

## What was wrong

A works that orders plant (to be founded, or to grow at the year's review) asks the merchants every month for a quarter of what is still to come, so most of its timber and bricks are asked for in the three or four months after the review, and the mean wait for a unit of plant is four months. Before 1900 the households bought two thirds of the timber and half the bricks. After 1900 the works being built take nearly all of both: by 1950 all the bricks, and the cloth mills, whose plant is 10 timber a unit, order nine tenths of the timber that goes into plant. One year's orders can be more than the woods cut in a year, asked for in a season. Timber's price swings threefold to fivefold every four or five years, and bricks' with it (the trace, sections 1 to 3). Plant built without timber, sawn timber or bricks, a counterfactual and not a cure, takes four fifths off both swings, on the trace's eight islands and on the check's sixteen.

## What it does

From January of `from` (1901), a works asks each month for `1 / months` of the plant still to come (`months` 12). Half of an order is in by the eighth month, a third is still to come after a year and a twentieth after nearly three; the mean wait is a year, where the port's is four months. So each year's orders reach the merchants spread over the next two or three years, the waves of a four-to-five-year cycle overlap, and no season sees a whole year's orders at once. Nothing else changes: what each works orders and when, the year's room, the review and its hurdle, the merchants' prices, the woods. The capacity still arrives in proportion to what has been paid for, so a works with a mill half delivered has half a mill. A works counts as building while more than a twentieth of its capacity is still on order (`works.built_share`), which now lasts longer, so it orders again later; the check showed that this longer wait is not what cures (below).

**It is the year's mean wait that does it, not a building season.** The trace described the rule as a builder's season: the plant brought in over a building year. The check tried exactly that, a twelfth of the order as placed each month, all in within a year, and it did nothing for timber (its swing 0.98 of what it was) and less than half as much for bricks. Plant brought in evenly over two years, whose mean wait is a year as this rule's is, did what this rule does (timber 0.78, bricks 0.48). A works whose plant takes a year on the mean to come in is near what surveys of the time to build a works have found; a works whose plant is all asked for in a season is what made the cycle.

## Its numbers

`PaceParams`, in `pace.rs`. Off under `default_params()`; `PaceParams::on()` in `game_params()`.

| | port | game | |
|---|---|---|---|
| `enabled` | false | true | works ask for their plant at a twelfth of what is still to come from `from` |
| `months` | 12 | 12 | each month a works asks for one part in this many of what is still to come (the port's is a quarter, as if 4) |
| `from` | 1901 | 1901 | the first year whose months are paced; the played century is untouched |

`--set pace.enabled=false` plays the game without it, and `--set pace.months=6,12,24` other paces. It is on the list of switches that may be thrown in a running island (`edits::SWITCHABLE`), and so it is a switch with its two chips in the Rules tab and one of the rules the core's table (`isostate-core ablate`) turns off in turn.

## What the sweeps showed

Islands 17 to 32, six families, to 2050, on the tree of 26 November with main merged in (`isostate-core sweep --seeds 17-32 --rules game --families 6 --end-year 2050 --set pace.enabled=false,true --standing --growth`). The swing is the standard deviation of the yearly change in the log, of a price over the wage or of a quantity, 1910 to 2050, the mean of the islands.

| | off | on | |
|---|---|---|---|
| timber's price | 0.532 | 0.380 (0.71) | lower on 16 of 16 |
| bricks' price | 0.474 | 0.259 (0.55) | lower on 16 of 16 |
| sawn timber's price | 0.577 | 0.378 (0.65) | lower on 16 of 16 |
| building work's price | 0.115 | 0.055 (0.48) | lower on 16 of 16 |
| timber cut, bricks made, building made | 0.566, 0.458, 0.127 | 0.410, 0.235, 0.061 | each lower on 16 of 16 |
| the money wage; the linked real wage | 0.029; 0.041 | 0.018; 0.025 | |
| the investment share (its yearly change) | 0.028 | 0.011 | |
| the other 26 goods' prices, mean | 0.127 | 0.094 | none swings more |
| timber in months of the wage, 2030-2050 | 0.69 | 0.52 | |
| the linked wage, 2030-2050 | 16.75 | 17.05 | |
| real income in the dearest fifth of years 1952-2048, against the years within two | +0.7 per cent (worst -17.2) | +0.7 per cent (worst -11.2) | |
| the linked wage in those years | +0.5 per cent (worst -16.9) | +1.6 per cent (worst -7.1) | |
| the real wage's worst year (`real_wage_worst_year`) | -0.364 | -0.275 | |
| alarms and verdicts in the news (`news_alarm`, `news_verdict`) | 15.5, 15.1 | 11.5, 11.1 | |
| shortages and steep moves in the news (`news_dearth`, `news_steep`) | 17.7, 64.4 | 15.6, 57.2 | |
| people in 2050 | 1,201,000 | 1,201,000 | |

The cycle is not hidden in the price: the quantities swing less with it, the building most of all, and it does not move to another good, to rents or to the wage, all of which swing less. The trace's own sixteen islands (1 to 16) gave the same: timber 0.75, bricks 0.56, lower on all sixteen.

## What worries me

- **It does not end timber's cycle.** Timber's swing falls by about three tenths and bricks' by nearly half. The rest is the woodmen, who hire on a price they expect months late and move a seventh of the way a month: on island 21 with the rule on, timber climbs three and a half times over 2048 and 2049 while the woodmen come in from 1,100 to 10,800, too late. With the land's hands and expectations quick as well, timber's swing is under half on eight islands (the trace, section 3). That wants a woodman's own pace in the labour market, which this rule does not touch.
- **The settling of pay over five years must stay.** Pay by class (docs/rules/pay.md) is reckoned on five years of work because the works' hiring lurches with their building. With the rule on the lurch is a third smaller, but a clerk's pay reckoned on one year's work would still move 14 per cent a year (20 per cent with it off).
- **A slower pace calms more and costs more.** At `months` 24 timber's swing is 0.45 of what it was and bricks' 0.37, but the linked wage of 2030-2050 is 2 per cent lower and building a head 3 per cent lower. Twelve is kept.
- **The machines' plant is not paced.** The orders for the machines that take tasks (`tasks::orders`) still ask for a quarter a month. They are electrical goods and machinery, not timber or bricks, and were left as they are.
