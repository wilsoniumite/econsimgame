# New money issued on the true total

Not in the Python. `crates/isostate-core/src/money.rs`, switched by
`money.true_total`: off under the port's rules, on under the game's. It is
stage 4 of the pass over all the rules (`docs/designs/rules-pass.md`,
section 4.3), and the fault it mends was measured in
`docs/probes/tr-money.md`, section 1.

## What it does

Each month the island issues new money to its households, by head. The port
issues `money_growth / 12` (2 per cent a year) times the money it counts, and
counts it at a moment when the households' whole income for the month (their
pay, their carting and both kinds of dividend) is held in a local vector in
`econ::step_month` and counted nowhere. That is 11.5 to 14.7 per cent of the
money, so the money grows 1.74 to 1.78 per cent a year where the setting says
2. The share left out is not a constant: it rises with what the merchants owe,
which is why each island's money ends the century somewhere different (36.4
to 38.5 million on seeds 1 to 8), and a month whose incomes are larger than
the money issues less than nothing, which is the road by which every
setting that still fails the money check fails.

With `money.true_total` on, the month's issue is `money.growth / 12` times the
money as it stood when the month began (`econ::money_total`, taken at the top
of `econ::step_month`; money only moves between purses inside a month, so that
is all of it). The port's `money_growth` is not read. The rule acts in the
warm-up as well, so the money of January 1750 is the purses the island starts
with, grown at the rule's rate for ten years.

## Its number

`money.growth` 0.0175 a year, issued a twelfth each month, which is 1.764 per
cent a year. That is the rate today's islands really had under the port's
count, so the island every other mechanism was tuned on keeps the money it
had; it now has it on every island alike and in every month. The 2 per cent
the old setting names was tried in the trace: the money of 1900 is 56
million, prices and money wages stand a third higher, nothing real is gained,
and carriage grows cheaper in real terms because the freight rate is a fixed
number of coins (`tr-money.md`, section 1). So the mend is to make the rate
what the setting says, and then to say the rate that was being used.

## What the tests hold

`tests/money.rs`, six tests. Each fails when the rule is taken away (tried
by leaving the switch on and counting the port's way, at 0.0175): the first
then still passes, being about the switch itself, and the other five fail.

- the switch is off under the port and on in the game, and with it off the
  rate is the port's number to the last bit and a port run's `params.json`
  does not mention it;
- the level: in every recorded month the money is the money the island began
  with times (1 + 0.0175 / 12) to the power of the months since the warm-up
  began, to a thousand-millionth, with no families and with six;
- the rate: 1.764 per cent a year to four figures, and the same money month
  for month on two islands;
- the level once more, as the critic asked (`rules-pass.md`, 10.9): an island
  that begins with every purse doubled has exactly twice the money in every
  month after, whatever its prices and wages do. Whether anything real moves
  with the level is section 7, task 1 of the design, which can only be read
  once the freight follows the wage (stage 6); this test holds the money's
  side of it;
- the issue is never below nothing, on the case that went below it: island 7
  under the machines' extreme setting (`quality_b` 2, `quality_cap` 50,
  `quality_Q0` 0), whose money grows every month to 1846 under the rule,
  where the port's count takes it below nothing in November 1845 and the
  run stops (in 1766 before stage 2's rationed hands, which slow that road
  and do not close it);
- what the player does makes no money: the family takes up a trade, and at
  the turn of each of its first three years founds the dearest works it can
  pay for at its seat and buys land, and the money of every month is still
  the rule's level to a thousand-millionth.

## What the sweeps showed

Measured on 12 November 2026, under the game's rules with six families,
read as section 3 of the design asks: sixteen islands, against today's game
and against a null pair (two runs of today with a number moved by a
millionth, `works.walk=0.0200001` and `carting.adjust=0.2500001`), so that
the rule is read against three histories of today and not one.

```bash
isostate-core sweep --seeds 1-16 --rules game --families 6 --set money.true_total=false --out today
isostate-core sweep --seeds 1-16 --rules game --families 6 --set money.true_total=false --set works.walk=0.0200001 --out null-walk
isostate-core sweep --seeds 1-16 --rules game --families 6 --set money.true_total=false --set carting.adjust=0.2500001 --out null-adjust
isostate-core sweep --seeds 1-16 --rules game --families 6 --out money
cargo run --release -p isostate-core --example money_trace -- --seeds 1-16 --rules game --families 6
```

With the switch off the game is today's: on seeds 1 to 8, 0 of the 563
facts the sweep compares differ from main's own sweep, and the golden test
holds the port.

**Its own measure.** The money grows 1.764 per cent a year on every island,
in every decade, with nothing left out of the count (`money_trace`): 2.6758
million in January 1750 and 37.518 million in December 1900 on all sixteen,
where today's islands end December 1900 between 36.4 and 38.7 million and
grow between 1.744 and 1.784 per cent a year (`money_trace` with
`--set '{"money":{"true_total":false}}'`). The same holds on eight islands
the measure was not made on, seeds 17 to 24: 1.764 per cent in every decade
of every island and 37.518 million at the end, where today's grow 1.753 to
1.791 per cent. Every month passes the money check, and the worst month's
own drift is under a millionth of a millionth of the money.

0.0175 is a little under today's middle island: today's money grows faster
than 1.764 per cent a year on twelve islands of sixteen (the mean is 1.771),
so by 1900 the rule's money is about a hundredth below today's on the mean
(0.988 in the 1890s, below). That is the design's number, and the null
pair says nothing real follows from a hundredth of the price level.

**Prices.** The money is 0.999 of today's in the 1750s, 0.983 in the 1860s
and 0.988 in the 1890s: the old count issued a little faster early in the
century and a little slower late, as the design said it would. The price
index by decade is 0.975 to 1.038 of today's on the mean of the islands, and
the money wage 0.982 to 1.013; the null pair alone moves them 0.960 to 1.076
and 0.978 to 1.044. So prices move less than one history of today differs
from another.

**The card.** Taking each of the four sweeps in turn and reading it against
the other three, the number of facts whose mean lies beyond the other three
by more than twice their spread is 27, 61 and 44 for the three histories of
today and 57 for the rule; of those, the ones that also move the same way on
13 islands of 16 or more are 3, 10 and 4, and 1 for the rule (the royalties'
share of output in 1800). By that test the rule is one more history of
today. Two facts on the card stand out and each has its reason:

- *The towns' share, 1881 to 1900*, 0.418 for 0.421, lower on 9 islands of
  16. It is island 3. That island finds crucible steel late, in 1784, in
  every history of today as under the rule, and its converter comes late
  too (1886 today, 1892 and 1872 in the null pair); under the rule it is
  never found, so the island takes the stalled road of the design's section
  4.8 (what a wage buys at the end is 1.99 where today's island 3 has 2.51
  and the walk's null 2.31), and its towns hold 0.361 of the people for 0.404. Without
  island 3 the share is 0.4214, and the three histories of today give 0.4214
  to 0.4223. The stall is the lottery of the dates of invention, which stage
  7 is for.
- *The wage's worst year* is a little milder, 14.5 per cent down for 15.3,
  on 10 islands of 16, which chance gives often.

Read a second way, island by island (for each fact the sixteen differences
from today and their paired t), the rule moves no fact by a t over 3, where
the walk's null moves 8 facts and the adjust's 4, and the largest t a null
reaches is 4.5 against the rule's 2.7. The rule's three largest movements are
the books' worst goods balance (5e-16 smaller: rounding), the railway's cost
in years of output (0.19 for 0.21, lower on 12 islands of 16) and the canals
in use in 1780. The railway's is island 9 almost alone: under the rule it
finds the railway in 1837 for 1818 and has 372 km of it in 1850 for 1,382,
as the adjust's null does to island 10 (found in 1837 for 1815) and the
walk's to island 7 (1830 for 1816). That is the lottery of the dates of
invention again, and not the money.

The scorecard as it then stood gave a loss of 42 for today's 151, which was
not the island coming nearer its aims: that scorecard took its tolerances
from the spread between islands, and island 3 widened it. The card of stage
1, with its tolerances held still, is read below.

**Again, on top of stage 2, with the card.** Stage 1 (the card) and stage 2
(hands that exist) landed while this stage was being checked, so the stage
was read again on main as they leave it, the same four sweeps of sixteen
islands. Today's money now grows 1.714 to 1.790 per cent a year; under the
rule 1.764 on every island as before. Every aim of the card stays inside the
three histories of today or just beyond them, the aims that could not be read
before among them: the towns' gain at the end 0.013 (today and the nulls
0.010 to 0.013), growth at the end 1.04 (1.03 to 1.05), the five worst years
12.2 per cent down (12.0 to 13.0), timber's swing 0.202 (0.192 to 0.207),
the tools' dear years 6.6 (6.9 to 8.1), no wild town-month anywhere, and the
carters between towns 0.014 in 1750 and 0.034 in 1900 as today. The loss is
29.2, where today's is 32.0 and the null pair's 34.5 and 38.8. Island by
island, six facts move by a paired t over 3 where the nulls move none and
two; six is what chance alone gives about 600 facts on sixteen islands, and
none of the six moves the same way on islands 17 to 32 (the carters between
towns of 1800, the largest, t -4.0 on the first sixteen and -1.1 on the
next, and the walk's null has it at -2.5 too). On all 32 islands one fact of
613 has a t over 3.

**The money check's failures.** The machines' extreme setting
(`quality_b` 2, `quality_cap` 50, `quality_Q0` 0), which stops islands 3
and 7 under the port's count when their money goes below nothing (in 1785
and 1766 before stage 2, in October 1893 and November 1845 with it), runs
to 1900 on both under the rule with every month passing, the money 37.518
million in December 1900 as on any island, before stage 2 and after.
The negative markup that also failed is now refused by the parameters' own
check, so it cannot be run either way. Under the commons
(`--set rent_regime=commons`), read only for failures as the design asks,
none of sixteen islands fails, nor with no families (`--families 0`); in
both the money grows 1.764 per cent a year on every island, and the
families' purses, taken from the savers when they start, change nothing in
it.

## What worries me

- **Every history moves.** The rule sends each island down another road from
  the first month of the warm-up, as any change does; a test that pins one
  island's history (the one sailing of sixteen islands, `tests/people.rs`) had
  to find its case again, and will after every stage of the pass.
- **Money is not neutral here, and this rule does not make it so.** At the
  same average rate the slope the trace found (more new money, cheaper
  carriage, fewer carters and towns) has nothing to work on, but it is still
  there, through the freight rate fixed in coins. Stage 6 takes that road
  away, and section 7, task 1 of the design is the test that it is gone: it
  should be read on the money's level as well as on its rate, which the
  test of twice the money holds on the money's side only.
- **The merchants still owe four tenths of the money by 1900**, now under a
  count that does not mind (`money_trace`: purses below nothing are 0.40 of
  the money in the 1890s, nearly all the merchants'). Why the price rule lets
  them is task 2 of the design.
- **The check's allowance** is still a millionth of the net money while the
  rounding grows with the purses' gross; the machines' extreme setting now
  passes with own drifts of up to 1.3e-11 of the money, and purses of 1e18
  would still cry wolf. That belongs to the guards, not to this rule.
