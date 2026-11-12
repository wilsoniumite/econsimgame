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

    needed[t]  = labor * bill[t] / wage[t]      (at most max_share of the town's hands)
    carters[t] = carters[t] + adjust * (needed[t] - carters[t])
    supply[t]  = participation * pop[t] - carters[t]

Carters are taken on and laid off by degrees, a quarter of the way each
month to the number last month's bill would pay, as the farms' and the
works' hands are hired part of the way to what their employers want
(`labor_adjust_ex` 0.15, `labor_adjust_firm` 0.35). The first version hired
them all at once, and that set towns with a heavy bill swinging ("The wage
that flipped", below). The cap holds after the step too: a town never has
more than `max_share` of its hands carting.

The carters come out of the supply first. Under the game's rules two more
groups follow them, each from the hands still left: the hands sinking new
workings (`mines.md`) and the navvies digging a canal (`canals.md`). So the
supply the wage clears is smaller again by those.

The bill is added up where the carting money is already paid out, to the
same town it is paid to, so the carters live where they are paid: a cell's
produce on its way to market, a works' inputs and output, and the charge on
the trade a town sends to other towns, which the buying town's merchants
pay to the sending town's households. That charge is the port's whole
charge on trade: the carriage and a handling charge of 3 per cent of the
goods' value at the sending town's price (`handling`); both were income that
cost nobody anything, and both are now somebody's time, but only the
carriage falls when a railway is built. The lag of a month is because the bill is only known
once the month's goods have moved.

Nothing else changes, and no money moves that did not move before: the
carting money still goes to households, who are now the carters. The wage
then clears a smaller supply of hands for everything else. A better road, a
railway or a canal lowers the bill, and from the next month the hands come
back, a quarter of the way each month.

Carters are counted in each town's employment (they are at work), but they
belong to no sector, so the farm and industry shares of employment are shares
of a larger total.

Carters are people, too. Under the game's rules, when cells and their people
go to another town at a year's end, the same share of the town's carters goes
with them (`peopling::move_with_cells`; `people.md`), so the new town's
carters move on from those and do not start from none.

## The numbers

`CartingParams`, in `carting.rs`. Off by default; `game()` switches it on
with these.

| | | |
|---|---|---|
| `enabled` | false | carters come out of each town's labour supply |
| `labor` | 1.0 | the share of the bill that is carters' wages; 0 measures the bill and changes nothing |
| `max_share` | 0.5 | at most this share of a town's hands can be on the road |
| `adjust` | 0.25 | how far a town's carters move in a month toward the number last month's bill would pay; 1 is all the way at once, as the first version did |

`--set carting.enabled=true --set carting.labor=0` is a run of the port, bit
for bit, that also keeps the bill (a test holds it to that), so the
`carting_share_of_output_Y` fact can be had without changing anything.

## What the sweeps showed

Seeds 1 to 8 to 1900, the port's rules with only this switched on, against
the port. Mean, and in brackets the least and greatest seed. Sweep folders
`sweep-carting*` and `sweep-railways-carting*` under `.claude/night/reports/`.
These are the port with carting alone, with carters hired at once (before
`adjust`); a sweep of today's game cannot give them and they were not
measured again. The game as it stands is under "In company".

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

## In company

Carting was merged into the game's rules with the game's railways
(`railways.md`), after the merchants' prices, the Engel curve, the machines,
the harvests, the chronicle and the life cycle for works. The integration's
sweeps are under `.claude/night/reports/core-steps/`: `railways-before` and
`railways-after` (the game's rules with six families on seeds 1 to 6, before
the merge and after), `railways-x-2x2` and `railways-x-2x2-s7-12` (seeds 1
to 6 and 7 to 12 with `carting.enabled` and `railways.enabled` each off and
on), `railways-x-less-NAME` (carting off and on with one other mechanism
switched off) and `railways-x-labor05`. No run failed; with both switches
off every fact of every seed is what it was before the merge.

**How many carters.** Twelve seeds, six families:

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| carters over all hands, the port's railways | 0.140 | 0.142 | 0.098 | 0.082 |
| the same with the game's railways | 0.140 | 0.142 | 0.086 | 0.066 (0.049 to 0.082) |
| carting bill over output, the game's railways | 0.100 | 0.096 | 0.054 | 0.045 |

A seventh of all hands before the railway (an eighth alone), and by 1900
the game's branches leave a fifth fewer on the road than the port's trunks.

**The level, and a slower middle of the century.** Carting on against
carting off, the port's railways in both, seed for seed on twelve seeds;
then the game as it now stands (carting and the game's railways) against
the game before the merge:

| | carting off | carting on | higher/lower | the game now |
|---|---|---|---|---|
| baskets per head 1750 | 1.66 | 1.50 (minus 10%) | 0/12 | 1.50 |
| 1800 | 3.26 | 3.03 (minus 7%) | 1/11 | 3.02 |
| 1850 | 6.54 | 4.87 (minus 26%) | 0/12 | 4.88 |
| 1900 | 8.86 | 8.27 (minus 7%) | 2/10 | 8.56 (minus 3%) |
| goods per head 1750 | 2.78 | 2.47 (minus 11%) | 0/12 | 2.47 |
| 1800 | 4.81 | 4.45 (minus 7%) | 1/11 | 4.45 |
| 1850 | 8.81 | 7.22 (minus 18%) | 0/12 | 7.18 |
| 1900 | 11.78 | 10.94 (minus 7%) | 0/12 | 11.34 (minus 4%) |
| urban share 1750, 1900 | 0.249, 0.450 | 0.210, 0.412 | 0/12 | 0.210, 0.420 |
| people in 1900 | 682,000 | 671,000 | 0/12 | 672,000 |
| years in which most people's towns grew at the cap | 0.997 | 0.962 | 0/11 | 0.962 |
| towns in 1900 | 16.8 | 15.7 | 1/6 | 15.7 |
| works standing in 1900 | 718 | 676 | 1/11 | 667 |
| famine years in a run | 1.7 | 3.3 | 10/0 | 2.0 |

At both ends of the run this is what carting does alone: an island a tenth
poorer at the start and a fifteenth at the end, four points less urban, a
little short of the cap, and the port's railways bringing famines that the
game's do not. With the game's railways the end is 3 or 4 per cent under
the game before, not 7: the branches give back half of what the carters
cost (`railways.md`, "In company").

The middle is not what it does alone. The game before the merge has a steep
rise from the 1830s to the 1860s, when farm machinery arrives; with carters
real that rise comes about ten years later and is still going in the 1890s.
The households' budget over the cost of subsistence, by decade, twelve seeds
(the game before, then the game now):

| | 1800s | 1810s | 1820s | 1830s | 1840s | 1850s | 1860s | 1870s | 1880s | 1890s |
|---|---|---|---|---|---|---|---|---|---|---|
| before | 3.42 | 3.63 | 3.91 | 4.36 | 5.57 | 7.29 | 8.53 | 8.96 | 9.05 | 8.82 |
| now | 3.16 | 3.33 | 3.49 | 3.60 | 4.12 | 5.51 | 7.06 | 8.10 | 8.39 | 8.46 |
| now over before | 0.92 | 0.92 | 0.89 | 0.83 | 0.74 | 0.76 | 0.83 | 0.90 | 0.93 | 0.96 |

Machinery tells it: 341 units a month in the 1820s for 569 and 483 in the
1830s for 789, two fifths less, then 1,311 for 1,634 in the 1850s and 2,732
for 2,775 in the 1890s. The reading of 1850 falls in the steepest part of
the rise, which is why it looks so much worse than a tenth.

Taking the other mechanisms away one at a time (seeds 1 to 6, the baskets of
1850 with carting over without) says where it comes from: minus 34 per cent
with everything on; minus 9 without the life cycle for works, which is the
level shift again; minus 14 without the merchants' prices; minus 2 without
the machines, where there is no rise to delay (3.6 baskets either way);
minus 38 and minus 29 without the Engel curve and without the harvests,
which have no part in it. So it is the machines' rise, delayed where works
grow at the pace of the year's room (`works.md`, D): a tenth less trade is a
tenth less room every year, the engine works come later and the thresholds
that farm machinery waits on are reached later. On seeds 1 to 6 the first
engine works make machinery twelve years later with carting (five seeds of
six) and farm machinery is found fifteen years later (five of six); without
the life cycle neither date moves. It is not the same on every island: on
seeds 7 to 12 neither date moves with everything on, and the baskets of
1850 are 16 per cent down, not 34. Nothing is broken in it (no failed run,
no price at a floor or a ceiling, the money check holds), and each
mechanism is doing what its page says, but together they move the island's
steepest decades by about ten years. Whether the game wants that is a
question of pacing for Stella, so `labor` stays at 1 as it was reviewed.
It is the dial: at 0.5 (`railways-x-labor05`, seeds 1 to 6) carters are 7.6 per
cent of hands in 1800 and 3.3 in 1900, and the baskets of 1750, 1850 and
1900 are 1.54, 6.08 and 8.61, where 1 gives 1.48, 4.86 and 8.50 and no
carting 1.65, 7.30 and 8.94: half the gap of 1850 closes.

Because a run with carting starts lower, the ratios to a run's own 1750 hide
all of this (at the merge the real wage of 1900 was 2.77 times 1750's, and
2.86 before it). Baskets and goods per head are the facts to read.

All of this section is the game as it stood at that merge, with carters
hired at once. The people rules (which brought `adjust`), kept town ground,
the mines, the canals and the books of account have come in since, so none
of it can be read from a sweep of today's game, and it was not measured
again. Where the same fact exists today it reads as follows: the merge's
numbers from the tables above (the game's railways, twelve seeds), beside the
game as it is played now (seeds 1 to 8, six families,
`isostate-core sweep --seeds 1-8 --rules game --families 6`, 12 November
2026; the mean over seeds):

| six families | at the merge | the game now |
|---|---|---|
| carters over all hands 1750 / 1800 / 1850 / 1900 | 0.140 / 0.142 / 0.086 / 0.066 | 0.125 / 0.118 / 0.083 / 0.064 |
| carting bill over output, same years | 0.100 / 0.096 / 0.054 / 0.045 | 0.091 / 0.079 / 0.052 / 0.044 |
| baskets per head, same years | 1.50 / 3.02 / 4.88 / 8.56 | 1.52 / 3.30 / 5.18 / 8.76 |
| goods per head, same years | 2.47 / 4.45 / 7.18 / 11.34 | 2.59 / 4.83 / 7.92 / 11.4 |
| urban share 1750, 1900 | 0.210, 0.420 | 0.218, 0.419 |
| people in 1900 | 672,000 | 732,000 |
| towns in 1900 | 15.7 | 17.8 |
| works standing in 1900 | 667 | 738 |
| famine years in a run | 2.0 | 0.4 |
| real wage of 1900 over 1750's | 2.77 | 2.60 |

So it is an eighth of all hands on the road before the railway now, not a
seventh, and in 1800 the bill and the carters are both a sixth lighter
(0.079 for 0.096, 0.118 for 0.142); by 1900 the two stand where they stood. The slow middle of the century is still there: the
baskets of 1850 are 5.18, between seeds 3.69 and 7.33, where at the merge
with carting off they were 6.54 (the level table above). Whether the delay is
still ten years has not been measured again.

**The scripted players** (`examples/bots.rs` on seeds 1, 2, 3 and 7, the
whole game; `core-steps/railways-probe/bots-game.txt`, against
`works-probe/bots-game.txt` from before the merge) finish all sixteen games.
The player's rank in 1900 is what it was or a place better in fourteen of
them (the landlord is first on every seed, as before); on seed 7 the passive
player rises from fourth to first and the founder falls from third to
fourth. In
money everyone ends richer, the player more than the rivals: 23 to 76
million for 19 to 48, the richest rival 20 to 71 for 17 to 47. One game
each, so this says the family game still plays, not which way it leans.

## The wage that flipped

Found when the game's rules for people were merged (`people.md`, "In
company"), and mended there. It was in carting from the start; the yearly
facts cannot show it.

As first written, a month's carters were last month's whole bill at last
month's wage, hired at once. The farms and the works hire by degrees
(`labor_adjust_ex`, `labor_adjust_firm`), so in the short run what they want
hardly bends. In a town with a heavy bill that sets up a swing: many carters
this month leave few hands for the rest, the wage is bid up to double (the
most it may move in a month), so the same bill buys half the carters next
month, hands are plentiful, the wage halves, and so on month about. One
town caught in it (Redgate, seed 4, founded 1758) had a wage of 4.4 and 8.7
in alternate months through its first year and 3.7 and 7.4 through its
second, with a half and a fifth of its hands on the road by turns.

How much of it there was, as the share of town-months (February to
December) in which a town's wage moved by half or more from the month
before, six families, seeds 1 to 12:

| | carters hired at once | by degrees (`adjust` 0.25) | no carting |
|---|---|---|---|
| the port with carting alone | 0.064 (0.017 to 0.118) | 0.002 | 0.001 to 0.002 |
| the game before people | 0.197 (0.132 to 0.284) | 0.003 (0.002 to 0.005) | |
| the game with people | 0.235 (0.166 to 0.336) | 0.000 (at most 0.002 on sixteen seeds) | |

So in the game as it stood a fifth of all town-months had the wage double
or halve. People made it plain because a district that gets a market of its
own is an exporter, and the cart onward to the towns that buy from it is
paid to its own households and so is on its own bill, which puts a third
to a half of its wage bill on the road; a new town's wage went to between
two and four and a half times the middle town's in its first spring, on
every one of twelve islands.

With `adjust` at 0.25 a town's carters go a quarter of the way each month
to the number last month's bill would pay, the swing dies in a month or two
where it starts at all, and what is left is the port's own (a new town's
first months under the port's founding rule). Tried on seeds 2 and 4 with
everything on: at 0.5 the wage still moves by half or more in 0.002 and
0.008 of town-months, at 0.35 in under 0.001, at 0.25 in four or five
town-months of a whole run, at 0.15 in one. 0.25 stands between the farms'
0.15 and the works' 0.35. `--set carting.adjust=1` is the first version, bit
for bit.

It hardly moves the yearly facts, which is why nobody saw it. The port with
carting alone, seeds 1 to 8 without families, at once and by degrees: 664
and 665 thousand people in 1900, 3.41 and 3.47 baskets a head, 4.99 and 5.00
of goods, 6.9 and 7.1 per cent of hands carting, 4.5 and 4.0 famine years
(`core-steps/people-x-carting-alone`); the tables above were made with
carters hired at once and still stand. In the game without people
(`people-x-off`, `people-s7-12-x-off`, seed for seed against the game
before) the baskets of 1900 go from 8.50 to 8.39 on seeds 1 to 6 and from
8.62 to 8.69 on seeds 7 to 12, the goods from 11.4 to 11.3 and from 11.3 to
11.6, and the railway of 1850 is 16 and 5 per cent longer (ten seeds of
twelve); nothing is starred but a little less wild land in 1800.
`tests/carting.rs` holds the flipping down (over 3 per cent of town-months
at once on seed 4, under half a per cent by degrees: the port with carting
alone, to 1789).

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
  under the port with carting alone (5.5 to 8.9 on eight seeds). So there,
  by 1900, the same cartload over the same road takes two fifths fewer hands
  than in 1750, railway or no railway, and part of the fall in carters with
  no railways at all (0.133 of all hands in 1800, 0.110 in 1900) is this and
  not better roads. In the game as it is played now the wage rises much
  more: from 4.3 in 1750 to 13.5 in 1900, about three times (2.5 to 4.0
  times by seed; the people-weighted wage, `wage` in the `annual.csv` of
  `isostate-core sweep --seeds 1-8 --rules game --families 6`, 12 November
  2026). There the same cartload takes about two thirds fewer hands in 1900
  than in 1750 before any road, canal or railway is counted. A freight rate
  that followed the wage would mend it; that changes what carriage costs
  everyone and was left for a decision.
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
  month late. The first version said here that no oscillation had been seen
  in the wage series, without looking month by month; there was one, in a
  fifth of the game's town-months ("The wage that flipped"). Carters taken on
  by degrees end it, but the lag is still there, and `adjust` is a damper
  set by trying four values on two islands and checking the choice on
  sixteen. The cleaner cure would hire carters in the month's own labour
  market, as one more demand that falls as the wage rises; that goes into
  the wage's root-finder and was more than an integration should change.
- **A town of exporters carts a great deal.** A new quarry or colliery town
  can have three hands in ten on the road, twice the island's share,
  because the cart from its market to the towns that buy its stone is on
  its own bill (see "Carters are a town's, not a road's", above). That is
  what the model's trade says, not a fault of the damper, but it is where
  carting bites hardest.
- **The family game shifts on one seed of four.** The scripted players
  (`examples/bots.rs`) finish all sixteen runs under the game's rules, which
  now means railways and carting together. Against the port's rules their
  rank in 1900 moves by a place or two either way on seeds 1, 2 and 3; on seed 7
  the landlord falls from first to third and the thoughtful player from
  first to fourth (after the review's repairs; before them both fell to
  fourth). One run each, so I cannot say which of the two mechanisms did it
  or whether it is chance.
