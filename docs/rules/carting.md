# Carting

Carting takes labour. Not in the Python. The switch is `carting.enabled`; the
code is `carting.rs` and five marked lines in `econ::step_month`. The
diagnosis this answers is `docs/probes/dx-rail.md`, section 2.

## What was wrong

Under the port every unit of weight moved costs money: a cell's produce on
its way to market, a works' inputs and output, the trade a town sends out.
That money is paid to the town's households as income, and nobody's time goes
into it. The carters are paid and do not exist. So a railway that cuts the
cost of carriage by three quarters moves money from one pocket to another and
leaves output per head exactly where it was, which is what dx-rail measured:
railways raise trade by a third and living standards by nothing.

## What it does

With the switch on, the month's carting bill of each town is the wages of
carters at that town's wage, and those hands are not there to be hired by
farms and works the month after:

    carters[t] = labor * bill[t] / wage[t]      (at most max_share of the town's hands)
    supply[t]  = participation * pop[t] - carters[t]

The bill is added up where the carting money is already paid out, to the
same town it is paid to, so the carters live where they are paid: a cell's
produce on its way to market, a works' inputs and output, and what a town
pays to send its trade to other towns. That last part is the port's whole
charge on trade, which is the carriage and a handling charge of 3 per cent
of the goods' value (`handling`); both were income that cost nobody
anything, and both are now somebody's time, but only the carriage falls when
a railway is built. The lag of a month is because the bill is only known
once the month's goods have moved.

Nothing else changes, and no money moves that did not move before: the
carting money still goes to households, who are now the carters. The wage
then clears a smaller supply of hands for everything else. A better road, a
railway or a canal lowers the bill, and next month the hands are back.

Carters are counted in each town's employment (they are at work), but they
belong to no sector, so the farm and industry shares of employment are shares
of a larger total.

## The numbers

`CartingParams`, in `carting.rs`. Off by default; `game()` switches it on
with these.

| | | |
|---|---|---|
| `enabled` | false | carters come out of each town's labour supply |
| `labor` | 1.0 | the share of the bill that is carters' wages; 0 measures the bill and changes nothing |
| `max_share` | 0.5 | at most this share of a town's hands can be on the road |

`--set carting.enabled=true --set carting.labor=0` is a run of the port, bit
for bit, that also keeps the bill (a test holds it to that), so the
`carting_share_of_output_Y` fact can be had without changing anything.

## What the sweeps showed

Seeds 1 to 8 to 1900, the port's rules with only this switched on, against
the port. Mean, and in brackets the least and greatest seed. Sweep folders
`sweep-carting*` and `sweep-railways-carting*` under `.claude/night/reports/`.

**How many carters.** Under the port's own railways:

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| carting bill over output | 0.085 | 0.090 | 0.052 | 0.038 (0.035 to 0.041) |
| carters over all hands | 0.119 | 0.133 | 0.089 | 0.069 (0.063 to 0.075) |
| the same with no railways at all (`rail_know=99`) | 0.119 | 0.133 | 0.124 | 0.110 (0.095 to 0.141) |
| the same with the game's railways | 0.119 | 0.133 | 0.091 | 0.057 (0.050 to 0.069) |

So an eighth of the country's hands are on the road before the railway, and
the railway gives back four in every ten of them by 1900; the branches of
`railways.md` give back a tenth more.

**The whole economy sits lower.** This is a change of level, from the first
month of the warm-up:

| | port | carting |
|---|---|---|
| output per head, 1751 to 1820 (1750 money) | 5.02 | 4.60 (minus 8.3%) |
| output per head, 1860 to 1900 | 6.29 | 5.93 (minus 5.6%) |
| goods made per head at base prices, 1750 | 2.85 | 2.61 |
| the same, 1800 | 4.35 | 3.81 |
| baskets per head, 1750 | 1.76 | 1.61 |
| the same, 1800 | 3.04 | 2.69 |
| urban share, 1750 | 0.264 | 0.223 |
| farm share of employment, 1750 | 0.650 | 0.569 |
| population, 1900 | 682,000 | 664,000 (7 seeds of 8 lower) |
| years in which most people's towns grew at the cap | 0.995 | 0.933 |
| famine items a run | 2.2 | 4.9 (7 seeds of 8 higher) |
| works closed | 4,710 | 4,236 |

With six families it is the same picture: output per head 5.13 to 4.70 in
1751 to 1820 and 6.54 to 6.14 in 1860 to 1900; works standing in 1900, 6,973
to 6,494.

Because a run's own 1750 is lower too, the facts that are ratios to 1750
(`real_wage_Y_vs_1750`, `output_per_head_Y_vs_1750`) cannot set a run with
carting beside one without. Two facts were added for that, in units that do
not depend on a run's own start: `baskets_per_head_Y`, the households' monthly
budget for goods over the cost of the subsistence basket, averaged over
people; and `goods_per_head_Y`, everything made in a month at the content's
base prices, per head (gross: iron is counted, and so are the tools made of
it).

**Railways now raise output per head.** Means of 1860 to 1900, eight seeds:

| | no railways at all | port's railways | game's railways |
|---|---|---|---|
| output per head, carting off | 6.23 | 6.29 | 6.26 |
| output per head, carting on | 5.60 | 5.93 | 5.90 |
| real wage (1750 = 1), carting off | 1.31 | 1.33 | 1.34 |
| real wage, carting on | 1.23 | 1.23 | 1.23 |
| urban share, carting on | 0.210 | 0.219 | 0.218 |

With carting off the three columns differ by less than the spread between
seeds. With it on, railways are worth 5 to 6 per cent of output per head,
and every one of the eight seeds is higher with them (by 3.4 to 11.8 per
cent under the port's network, 0.0 to 8.0 under the game's).

Out of sample, on seeds 9 to 16, it is the same picture and a smaller gain:
carting alone takes output per head from 5.09 to 4.71 in 1751 to 1820 and
from 6.30 to 5.89 in 1860 to 1900, and with carting on railways are worth 3
per cent (5.72 with none, 5.89 with the port's, 5.87 with the game's; seven
and eight seeds of eight higher). Those islands get their railways ten to
fifteen years later, so less of 1860 to 1900 has them.

Famines are odd. With carting on there are 1.8 famine items a run with no
railways at all, 4.9 with the port's railways (every seed of eight higher),
and 1.6 with the game's (2.1 with six families, against 4.8); on seeds 9 to
16 it is 1.9, 4.6 and 1.8. So the extra famines in the table above belong
to the port's cheap railway rush once carters are real, not to carting
itself, and the game's dearer, slower railways do not bring them. I did not
trace why.

Two things do not follow, and I report them as they are:

- **The real wage does not rise with output.** It is 1.23 with or without
  railways. The hands that come off the road are hired at the going wage and
  make more, and the gain goes somewhere other than the wage. I did not trace
  where (rents and profits are the likely places).
- **The game's branches add nothing one can see over the port's trunks**:
  5.90 against 5.93, although they take another sixth of the carters off the
  road. With six families it is 6.00 against 6.14, and the real wage 1.25
  against 1.32, with one seed of eight higher and seven lower: not far
  outside what chance gives, but it is the pair the game is played under.
  Before the review's repairs a promoter who reckoned more strictly
  (`rail_induced` 1 instead of 2.5) built a fifth less line for the same
  output per head, so it is not the marginal lines either. The trunks take
  most of what there is to save; what the branches save is about the size of
  what they cost.

## What worries me

- **All of the bill is counted as men's time.** Horses, fodder, wagons and
  tolls are not in the model, so `labor = 1` makes carters of them all, and
  an eighth of all hands on the road in 1800 is a lot. `labor` is the dial.
  Anything under 1 leaves part of the bill as the port had it, income that
  costs nobody anything.
- **Everything tuned to the early years shifts.** The economy is a twelfth
  smaller from the start, the towns are four points less urban and
  population falls short of the cap more often. PLAN's targets for 1750 to
  1800 were set without it.
- **A carter's work is priced in money, and the money changes.** Carters are
  the bill over the wage. The freight rate (`freight`) is the port's and
  never moves, while the money wage rises by about two thirds over the run
  (5.5 to 8.9 on eight seeds). So by 1900 the same cartload over the same
  road takes two fifths fewer hands than in 1750, railway or no railway,
  and part of the fall in carters with no railways at all (0.133 of all
  hands in 1800, 0.110 in 1900) is this and not better roads. A freight
  rate that followed the wage would mend it; that changes what carriage
  costs everyone and was left for a decision.
- **Three per cent of the value of all trade is on the bill**, as handling,
  and no road or railway lowers it. I did not measure how much of the bill
  it is.
- **The first railway comes later in some seeds** (between 1813 and 1849 on
  eight seeds, against 1815 to 1838): a smaller iron trade learns rails
  later. On seeds 9 to 16 the discovery itself moves by seven years on
  average and by more than twenty either way on single islands (1847 to
  1820 on one, 1824 to 1852 on another), so carting reshuffles when the
  railway age begins.
- **Carters are a town's, not a road's.** They are taken from the town that
  is paid the carting, wherever along the way the cart is. Trade between two
  towns takes its carters from the town that sends.
- **A month's lag.** In a month when the bill jumps, the hands are missed a
  month late. I saw no oscillation in the wage series, and the money check
  holds every month, but I did not look for one at monthly resolution beyond
  the tests.
- **The family game shifts on one seed of four.** The scripted players
  (`examples/bots.rs`) finish all sixteen runs under the game's rules, which
  now means railways and carting together. Against the port's rules their
  rank in 1900 moves by a place or two either way on seeds 1, 2 and 3; on seed 7
  the landlord and the thoughtful player fall from first to fourth. One run
  each, so I cannot say which of the two mechanisms did it or whether it is
  chance.
