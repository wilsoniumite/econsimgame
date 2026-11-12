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

`tests/money.rs`, five tests, each of which runs the port's count beside the
rule and fails without it:

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
- the money grows every month: the issue is never below nothing.

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

With the switch off the game is today's: on seeds 1 to 8, 0 of 565 facts
differ from main's own sweep, and the golden test holds the port.

**Its own measure.** The money grows 1.764 per cent a year on every island,
in every decade, with nothing left out of the count (`money_trace`): 2.6758
million in January 1750 and 37.518 million in December 1900 on all sixteen,
where today's islands end between 36.1 and 38.4 million and grow between
1.744 and 1.784 per cent a year. Every month passes the money check, and the
worst month's own drift is under a millionth of a millionth of the money.

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
  16. It is island 3. Under the rule that island finds crucible steel in
  1784, takes the stalled road of the design's section 4.8 (the converter is
  never found, and what a wage buys at the end is 1.99 where today's island
  3 has 2.51), and its towns hold 0.361 of the people for 0.404. Without
  island 3 the share is 0.4214, and the three histories of today give 0.4214
  to 0.4223. The stall is the lottery of the dates of invention, which stage
  7 is for.
- *The wage's worst year* is a little milder, 14.5 per cent down for 15.3,
  on 10 islands of 16, which chance gives often.

The scorecard's loss falls from 151 to 42, which is not the island coming
nearer its aims: the scorecard takes its tolerances from the spread between
islands, and island 3 widens it (`scorecard.md`, "What worries me").

**The money check's failures.** The machines' extreme setting
(`quality_b` 2, `quality_cap` 50, `quality_Q0` 0), which stops islands 3 and
7 today when their money goes below nothing, runs to 1900 on both with every
month passing, the money 37.518 million in December 1900 as on any island.
The negative markup that also failed is now refused by the parameters' own
check, so it cannot be run either way. Under the commons
(`--set rent_regime=commons`), read only for failures as the design asks,
none of sixteen islands fails.

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
