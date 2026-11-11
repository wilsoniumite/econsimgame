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
same town it is paid to, so the carters live where they are paid. The lag of
a month is because the bill is only known once the month's goods have moved.

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
| the same with the game's railways | 0.119 | 0.133 | 0.089 | 0.058 (0.055 to 0.060) |

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
1751 to 1820 and 6.55 to 6.14 in 1860 to 1900; works standing in 1900, 6,973
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
| output per head, carting off | 6.23 | 6.29 | 6.21 |
| output per head, carting on | 5.60 | 5.93 | 5.92 |
| real wage (1750 = 1), carting off | 1.31 | 1.33 | 1.32 |
| real wage, carting on | 1.23 | 1.24 | 1.23 |
| urban share, carting on | 0.210 | 0.219 | 0.219 |

With carting off the three columns differ by less than the spread between
seeds. With it on, railways are worth 5.9 per cent of output per head, and
every one of the eight seeds is higher with them (by 3.4 to 11.8 per cent
under the port's network, 1.2 to 17.8 under the game's). The second famine
item in three also goes: 4.9 a run without the game's railways, 2.4 with
them.

Two things do not follow, and I report them as they are:

- **The real wage does not rise with output.** It is 1.23 with or without
  railways. The hands that come off the road are hired at the going wage and
  make more, and the gain goes somewhere other than the wage. I did not trace
  where (rents and profits are the likely places).
- **The game's branches add nothing one can see over the port's trunks**:
  5.92 against 5.93, although they take another sixth of the carters off the
  road. With six families it is 6.04 against 6.14, inside the spread
  (two seeds of eight higher, six lower). A promoter who reckons more strictly
  (`rail_induced` 1 instead of 2.5) builds 1,680 km instead of 2,050 and the
  figure is 5.88, so it is not the marginal lines either. The trunks take
  most of what there is to save; what the branches save is about the size of
  what they cost.

## What worries me

- **All of the bill is counted as men's time.** Horses, fodder, wagons and
  tolls are not in the model, so `labor = 1` makes carters of them all, and
  an eighth of all hands on the road in 1800 is a lot. `labor` is the dial.
  Anything under 1 leaves part of the bill as the port had it, income that
  costs nobody anything.
- **Everything tuned to the early years shifts.** The economy is a twelfth
  smaller from the start, the towns are four points less urban, population
  falls short of the cap more often and famines double. PLAN's targets for
  1750 to 1800 were set without it.
- **The first railway comes later in some seeds** (between 1813 and 1849 on
  eight seeds, against 1815 to 1838): a smaller iron trade learns rails later.
- **Carters are a town's, not a road's.** They are taken from the town that
  is paid the carting, wherever along the way the cart is. Trade between two
  towns takes its carters from the town that sends.
- **A month's lag.** In a month when the bill jumps, the hands are missed a
  month late. I saw no oscillation in the wage series, and the money check
  holds every month, but I did not look for one at monthly resolution beyond
  the tests.
- **The bots were run under these rules but not compared** with a run without
  them; see the report.
