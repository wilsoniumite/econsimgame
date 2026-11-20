# Pay by class of task, reckoned and not played

Added on 20 November 2026. Not in the Python. The rule `pay` (`src/pay.rs`) only watches: nothing in the
economy reads it. On islands 1 to 8 run to 2050 at four settings of its two dials, every one of the 3,712
facts the island had before gives what it gave before, and the game's golden test holds the economy and the
news in both eras.

The island pays one wage a town, so it cannot say who loses when machines take tasks. This reckons it from
the hands the island plays: what each class of task would be paid if each cleared a market of its own, with
the two numbers the argument turns on as dials in the Rules tab. It is pay, not living, and it is a
reckoning, not a run: the island goes on paying one wage, and nobody's choices answer to these figures.

## What is reckoned

**Four classes** of task, as [tasks.md](tasks.md) has them: hauling; tending, with the work only a person may
do and every machine's minders; clerking; judging. A class's hands are those of the works, the land and the
carters together (below). **Two kinds** beside them: the plain (hauling and tending) and the schooled
(clerking and judging), reckoned as two markets of their own, not added up from the four.

**The reckoning**, each year from 1900:

1. **The demand at equal pay.** What the island hires of a class at the one wage, as a share of all its
   hands, is the demand for the class at equal pay. It is taken over the year and the four before it (from
   1900): the works' hiring of a class lurches every other year with their building, and a clerk's share of
   the hands moves 7 per cent a year on the mean island from 1951 to 2050.
2. **Demand bends.** At pay `p` over the mean, the demand is that share times `p` to the power of minus
   `pay.bend`.
3. **Hands follow the work.** In 1900 each class has the hands its work takes. Each year after, its hands
   move one part in `pay.follow_years` of the way to where the work is.
4. **Pay clears.** A class's pay is where its demand meets its hands, `(share of work / share of hands) ^ (1
   / bend)`, scaled so that the hands' mean pay is the one wage. So every figure is a class's pay over the
   mean, and the hands' mean is 1 in every year.
5. **Real pay** is a class's pay over the mean times the linked real wage (`real_wage_linked`,
   [measures.md](measures.md)); in a run that does not keep it, the chained one.

It keeps no state of its own. At each year's end from 1900 the whole reckoning is played again from the
year's record of the tasks (`TasksYear`), cheaply (some 150 years of four sums), and every year's figures
since 1900 are written again. So a dial turned in a running island reads the whole century at the new dial,
and the facts at a run's end are the reckoning at its last dials.

**Where it comes from.** The analysis of 20 November 2026 (the second step back, "who loses when the
machines come?") reckoned this by hand from a sweep, to ask what a rule of pay by class would show before
building it. The bend of 0.3 is what it measured of the island's own machines: turning every machine's
quality up and down (`tasks.dial` at 0.8 and 1.25 from 1900) moves each class's share of the hands by about
0.3 times the change in the log of its price against the machines, where a machine stands at the margin.
Ten and thirty years for hands to follow bound what the island cannot measure, since it never moves a hand
from one class to another.

## The two dials

| name | game | what it is |
|---|---|---|
| `pay.watch` | on | the switch; fixed when the island is made, since its series are made with the record |
| `pay.bend` | 0.3 | how far demand for a class bends to its pay |
| `pay.follow_years` | 10 | how many years hands take to follow the work |

Both are pinned at the head of the sandbox's Rules tab, in those plain words, at figures of their own: the
bend at 0.1, 0.3, 1 and 2, the years at 5, 10, 20 and 30 (`rulebook::FIGURES`). In a sandbox the same chips
stand over the charts of pay in the Markets panel's Work half. A bend under 0.05 or fewer than one year are
held there in the reckoning (`LEAST_BEND`, `FEWEST_YEARS`), so that no setting runs a pay out of the numbers a
computer holds. The demand's five years are a constant (`SETTLE_YEARS`).

## The hands by class

The record of the tasks keeps, each year, the hands at each class in the works (`TasksYear::hands`, and the
minders), and since this page the land's and the carters':

- **On the land** (`TasksYear::land_hands`, `land_minders`): where the land's machines are reckoned, a
  class's hands are its work less what the machines in place did, every town and sector together; before
  the first machine on the land, every hand on a land unit does its whole work, so its hands are at its
  sector's shares of the classes (`data/tasks.toml`). That is counted month by month from 1750; no stand-in
  is left. What else the land's units hired beside the classes' hands (the machines' minders, the mines'
  sinkers) is `land_minders`, counted with tending.
- **The carters** (`TasksYear::carters`): the hands carting, at the carters' own shares of the classes in
  `data/tasks.toml` (0.70 hauling, 0.20 tending, 0.10 judging). The brief asked for them as haulers; the
  table is the game's own reading of a carter's work, and it is how every hand on the land is split.
- The canals' navvies are left out.

Facts: `hours_land_CLASS_Y`, `hours_land_minders_Y` and `hours_carters_Y` at the sweep's marks, beside the
works' `hours_CLASS_Y` ([tasks.md](tasks.md)). On islands 1 to 8 the four classes' shares of all hands,
works, land and carters, are:

| year | haul | tend | clerk | judge | hands | of them on the land | carting |
|---|---|---|---|---|---|---|---|
| 1900 | 0.355 | 0.392 | 0.079 | 0.174 | 336,242 | 0.375 | 0.091 |
| 1950 | 0.221 | 0.401 | 0.147 | 0.231 | 471,997 | 0.268 | 0.082 |
| 1970 | 0.196 | 0.409 | 0.150 | 0.245 | 484,866 | 0.267 | 0.086 |
| 2000 | 0.184 | 0.419 | 0.130 | 0.267 | 498,901 | 0.259 | 0.089 |
| 2050 | 0.200 | 0.489 | 0.091 | 0.220 | 517,826 | 0.271 | 0.087 |

## Where to read it

- **Series** (a year's figure in its twelve months; nothing before 1900): `pay_hands_CLASS`,
  `pay_CLASS` (over the mean), `pay_real_CLASS` (over the mean times the wage, 1750 = 1) and
  `pay_schooled_over_plain`. They are counted with the books' own (`accounts::own_series`): a saved game's
  fingerprint leaves them out, and the game's golden test hashes them with the books.
- **Facts**, each the mean of the nine years round its year (the last nine, 2042 to 2050, for 2050), as the
  linked wage's are: `pay_CLASS_Y` and `pay_schooled_over_plain_Y` at 1950, 1970, 2000 and 2050;
  `pay_real_CLASS_2050_vs_1970`, and `pay_real_mean_2050_vs_1970`, the wage's own, for setting beside them.
- **On screen**: the Markets panel's Work half opens on "Pay by class of task, reckoned and not played", three
  charts from 1900 to the year on the clock (each class's pay over the mean; each class's real pay against
  1900; the schooled over the plain), with the dials in words over them. The viewer runs the same reckoning
  (`pay::reckon`) on the hands it holds whenever the half is drawn, so a chip moved redraws the charts with
  nothing replayed.

## What it shows

Islands 1 to 8, six families, to 2050; the mean, and the least and greatest island where it matters:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set pay.bend=0.3,2 --set pay.follow_years=10,30 --out DIR
```

| | bend 0.3, 10 years | bend 2, 30 years |
|---|---|---|
| hauling's pay over the mean, 1950 | 0.69 [0.65 .. 0.74] | 0.88 |
| clerking's, 1950 | 1.46 [1.26 .. 1.93] | 1.17 |
| hauling's, 2050 | 1.09 | 1.01 |
| tending's, 2050 | 1.14 | 1.04 |
| clerking's, 2050 | 0.76 [0.70 .. 0.80] | 0.92 |
| judging's, 2050 | 0.78 | 0.95 |
| a hauler's real pay, 2050 over 1970 | 1.65 | 1.37 |
| a tender's | 1.34 | 1.24 |
| a clerk's | 0.86 [0.81 .. 0.93] | 1.01 [0.96 .. 1.09] |
| a judge's | 0.84 [0.77 .. 0.94] | 1.08 |
| the one wage's (linked) | 1.20 | 1.20 |
| the schooled over the plain, 1950 / 1970 / 2000 / 2050 | 1.51 / 1.16 / 1.02 / 0.69 | 1.16 / 1.11 / 1.05 / 0.91 |

At the island's own bend the clerk and the judge lose: from 1970 to 2050 the one wage rises a fifth, and
their real pay falls to 0.86 and 0.84 of 1970's. The schooled, paid half as much again as the plain in 1950,
are level with them about 2000 and fall to seven tenths of them by 2050. At a bend of 2 with hands slow to follow, nobody's real pay falls and the
schooled only fall behind. The analysis of 20 November had the clerk at 0.72 and 0.98 and the schooled at
0.61 and 0.88 in 2050; it took single years, the chained wage, and from 1940 the land counted without the
carters, whose hauling (six hundredths of the hands) dropped out of its count between 1900 and 1950.

## What it is not

- **It is pay, not living.** A town still has one purse: its households share one income however their hands
  are paid, so nobody here lives worse for being a clerk. Who lives worse needs a purse for each kind.
- **The tenders' gain rests on a machine that is not there.** No machine tends in a works (the electric
  drive hauls, computing clerks, models judge), so tending's share of the hands grows to half by 2050 and its
  pay with it. A machine that tends (numerical control, robots) would change who loses after 1970.
- **The bend is the machines' margin alone.** 0.3 is how far the island's hiring of a class bends where a
  machine stands at the margin. Inside a works the classes go in fixed shares, so the mix of goods and of
  methods, which would bend demand more in a played rule, adds nothing here.
- **Hands do not choose.** They follow the work at a fixed pace, with no schooling, age or pay to choose by,
  so `pay.follow_years` is a bare number.
- **A year's figure is one year's.** At a bend of 0.3 a class's pay is its settled share of the work over its
  hands to the power 3.3, so even on five years' work the charts move from year to year; the facts are
  nine-year means.

## What the played rule will add

A rule of pay by class of task, behind its own switch: each town keeps each class's pay over the wage its
month's search finds, and the hands of each class; once a year each pay moves toward clearing its class, and
the hands move toward the work. Everything that reads the one wage today chooses a class's pay instead: a
works' cost of a hand (its method's mix of the classes' pays), the sites and reviews of works, the land's cost
of work, the machines' cost and their waterline (each class at its own pay, minders as tenders), the carters,
navvies and miners (haulers), and new towns, which copy their donor's. Budgets, births, migration and house
rent read the town's one purse and the mean wage, which the scaling keeps where they are, until a purse for
each kind follows. It comes after ages with schooling, through which the young who enter work choose a class
by its pay and may enter clerking or judging only with schooling, so that the years of following are no
longer a bare number. About 600 lines; the order is in the backlog (the notes of 20 November).

## What worries me

- **The yearly figures move.** The settled demand still moves a class's pay a few per cent a year, and more
  in the years an island's offices or works change fast (clerks on island 7 from 1915 to 1935). The facts are
  nine-year means; one year on a chart should not be read alone.
- **Before 1900 nothing is reckoned**, and 1900 is equal pay by construction. What the nineteenth century
  did to who does what is not in these figures, though the hands by class are counted from 1750
  (`hours_land_*`, `hours_*`).
- **The carters' shares are the content's guess**, never measured.
