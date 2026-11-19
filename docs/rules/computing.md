# Machines that reckon: office computing, a thin slice

Not in the Python. One `[[use]]` of `data/tasks.toml`, `office_computing`,
standing behind a switch of its own, `tasks.computing`: off under the port's
rules, on under the game's. It works through the rule for machines and tasks
as it stands ([tasks.md](tasks.md)) and adds no good, no method and nothing
drawn. It is a thin slice of the run to 2050, brought forward so that the
whole run has its age of machines that reckon, however plainly, before the
full stage is built.

**The standing check: 12 of 12 inside.** Nothing can be found before 1960,
so the played century cannot move, and it does not:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set tasks.computing=false,true --standing --out DIR
```

```text
standing check: 12 of 12 inside (0 moved, 0 OUTSIDE, 0 without a value): tasks.computing=true against tasks.computing=false
```

Every one of the twelve is the same to the digit in both arms, and so is
every other fact of the eight runs to 1900 (1,177 facts compared, none
differs). Run to 2050 on islands 1 to 16, the switch off gives the island
without the slice: set against the build before it, "of the 3015 facts both
have, the seconds aside, 0 differ". The design's three asks of the century
after 1900 are met with the switch on and off (below, "What the islands
showed").

## What the slice is

**One machine for clerks' tasks.** Clerking is records, reckoning and
routine dealing, a class of every works' labour ([tasks.md](tasks.md), "What
it does"): a quarter of the shops' and trades' (the services and the
emporium), and three to five hundredths of the mills', furnaces' and engine
works'. The hand trades have none, and the land has none. Office computing
may hold the whole class, from the easiest task (`from = 0`) to the hardest
(`reach = 1`), in the works alone. Clerks' tasks run from those at which a
hand is worth one machine unit to those at which a hand is worth 3,000, evenly
in the logarithm between.

**Its numbers.** Sized from island 7 in 1960, the year it finds it, under the
game's rules with six families (a wage of 16.69 a month, electrical goods
10.74 and electricity 4.86, the year's means over the island). When found a
unit costs 0.135 of a month's wage:

| part | of the unit's cost | what it is |
|---|---|---|
| return and wear on its plant | 0.6 | `(hurdle + 1/8) / 12` a month on plant worth 3.97 months' wages: 6.17 electrical goods a machine |
| its minders | 0.1 | 0.0135 of a hand a machine |
| what it runs on | 0.3 | 0.139 electricity a month |

At that cost a hand is worth 7.4 units, so it takes `ln 7.4 / ln 3000 = 0.25`
of clerking. Its life is 8 years, and its quality grows a twentieth a year
from the year it is found, to a cap of 100 (never reached by 2050). Its
one `scale` was refitted once: at 1 the towns' target in the year found was
0.262 on the mean of islands 1 to 8, so a unit was made `exp(ln 3000 * (0.262
- 0.25)) = 1.10` times as dear. Every number is in the file's comment.

**When it is found.** From 1960, the year the machine is to be had, on an
island that has made at least 5.4 electrical goods a head since 1750. The
floor is read per head (`discover_per_head = true`, a key any use may carry:
what the island has made of each good named in `discover`, over its people at
the year's end), so that a small island is not made late by its size. It is
half of what island 7 had made a head when the year came (10.90 by the end of
1959), so island 7 finds it in 1960 by the date. The floor is low on purpose.
The islands are near one size (0.93 to 1.15 million people in 1959), but they
came to electrical goods 35 years apart (island 5 first made them in 1868,
island 14 in 1904), so a threshold on their own making that island 7 just
reached in 1960 found it from 1960 to 1992 when it was counted in all, and
from 1960 to 1996 when counted a head. At 5.4 a head it is found in 1960 on
twelve islands of sixteen, in 1965 on island 14 and in 1967 on islands 1, 3
and 11: an island with a small electrical trade waits a few years for it, and
no island waits thirty. Every other use reads its threshold in all, as
before.

**The switch.** A `[[use]]` may name a switch of the rule it stands behind
(`switch = "computing"`; the file refuses a name that is not one of
`tasks::USE_SWITCHES`). While `tasks.computing` is off the use is never
found, has no cost, no part, no order and no plant, and the run is the run
without it, which `tests/tasks.rs` holds: island 7 to 1975 with the switch on
and the use taken out of the table in code is the run with the switch off, to
the last digit. Thrown off by hand after the find, the use is priced nowhere
and the works fall back as their machines wear. The switch is left out of the
parameters' file while it is off, so a run with it off writes the file it
wrote before. The ablation (`isostate-core ablate`, [../core.md](../core.md))
asks it with the other switches.

**The facts it shows.** For every class a machine may enter (haul, tend,
clerk, judge) at the sweep's marks: `auto_CLASS_works_Y` and
`aim_CLASS_works_Y` (the share of the works' work in the class their machines
in place do, and mean to), `waterline_CLASS_Y`, `margin_gap_CLASS_Y`,
`auto_CLASS_land_Y` (the land's sectors together), `machine_share_CLASS_Y`
(the works and the land together: both are counted in worker-months a month
of work as by hand, so they add), and `CLASS_half_year_works`. For the use,
from the rule's loop: `use_found_year_office_computing`,
`use_share_found_office_computing` (the towns' target in the year found),
`use_auto_after10_`, `use_margin_gap_two_lives_` (sixteen years on),
`dearer_first_` (the rank correlation across towns of the wage in the year
found and the share of clerking their works hold ten years on). The command
that shows them:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set tasks.computing=false,true --end-year 2050 --jobs 8 --out DIR
```

## What it leaves to the full stage

- **A computing good and its makers.** The design runs this machine on
  computing, made in computer rooms and later data halls, whose price falls a
  hundredfold over three generations of recipes. Here there is no such good:
  the running cost goes on electricity, and the cost falls only by the pace,
  five per cent a year in quality.
- **The work they give.** Computer rooms, software, the data halls, and the
  hands that make and mind them: the work the computing trade gives back.
- **Offices.** Clerking here is a part of each works' labour; the full stage
  has offices as trades of their own.
- **The office machines below it.** The design's typewriters and calculators
  hold the easiest tenth of clerking from 1900. They are not built, so office
  computing starts from the easiest task.
- **A school gate on its find.** The design finds computer rooms only on an
  island whose adults have had nine years of school. Here it is found on
  the date and a floor of electrical goods made a head.

## How the full stage deepens it

Without throwing it away. The use keeps its name, its class, its switch and
its place in the file. Its running cost moves from electricity alone to
computing and electricity, its plant from electrical goods to electrical
goods and computing, and it is sized again by the same rule: a unit costs
0.135 of a month's wage when found, six tenths of it return and wear, a tenth
minders; then its `scale` is refitted once, as now. The office machines come
in below it from the bottom of the class, and since a unit of computing is
the cheaper by 1960, the stacking of `work_out_parts` lets it take the
office machines' tasks from them as it gets cheaper: the computer took the
typewriter's tasks. The facts by class need nothing new.

## What the islands showed

Islands 1 to 16 to 2050 under the game's rules with six families, the new
land, on 20 November 2026, with the find as above (`isostate-core sweep
--seeds 1-16 --rules game --families 6 --end-year 2050 --set
tasks.computing=false,true --jobs 8 --standing`). Means over islands 1 to 8,
the least and greatest in brackets; islands 9 to 16 say the same unless
noted.

| islands 1 to 8 | off | on |
|---|---|---|
| found | | 1961.8 [1960 .. 1967]: 1960 on six islands, 1967 on islands 1 and 3 |
| the towns' target in the year found | | 0.249 [0.242 .. 0.255] |
| clerking done by machine, 1970 / 1990 / 2010 / 2030 / 2050 | 0 | 0.191 / 0.375 / 0.500 / 0.625 / 0.748 [0.709 .. 0.762] |
| half of clerking taken | | 2010.5 [2008 .. 2017] |
| clerks' hours in works, 2050 | 61,300 | 18,600 |
| tending, judging and presence hours in works, 2050 | 285,600 | 321,800 |
| hauling hours in works, 2050 | 29,800 | 32,400 |
| minders, 2050 | 3,630 | 4,810 |
| work's share of income, gross, 1990 / 2010 / 2030 / 2050 | 0.674 / 0.665 / 0.668 / 0.666 | 0.665 / 0.660 / 0.663 / 0.663 |
| the same net of wear | 0.696 / 0.689 / 0.692 / 0.690 | 0.689 / 0.685 / 0.687 / 0.688 |
| the chained real wage, 2050 against 1750 | 13.90 | 14.82 (higher on 8 of 8) |
| the machines' book, years of the wage bill, 2050 | 0.073 | 0.109 (higher on 8 of 8, every island apart) |
| hands on the land, 2050 | 0.174 | 0.171 |
| hands in works, 2050 | 0.658 | 0.657 |
| share living in towns, 2050 | 0.564 | 0.559 (lower on 8 of 8) |
| people, 2050 | 1.178 M | 1.178 M |

On islands 9 to 16 it is found in 1961.5 [1960 .. 1967], half of clerking is
taken in 2009.6 [2008 .. 2015], and machines hold 0.751 [0.716 .. 0.762] of
it in 2050.

The design's three asks of the century after 1900, on islands 1 to 8: the
chained real wage grows 1.14 per cent a year from 1900 to 1970 off and 1.15
on; 2050 over 1970 is 1.13 off and 1.19 on; the share of hands on the land
falls by 0.076 off and 0.079 on. Islands 9 to 16: 1.13 and 1.14 per cent a
year, 1.15 and 1.19, a fall of 0.073 and 0.070. All met in both arms.

Against the bands: the towns' first share is 0.25 (met); every island finds
it within seven years of island 7 (met); clerking done by machine is above
nothing from the decade after the find and rises at every mark to 2050, on
every island (met); dearer towns go first, `dearer_first` positive on all
sixteen islands, 0.34 to 0.81 on islands 1 to 8 (met); half of clerking is
taken in 2010.5 on the mean of islands 1 to 8, on twelve islands of sixteen
in 2008 or 2009 and on the other four between 2013 and 2017, so the mean
island sits at the far edge of the design's 1990 to 2010; and by 2050 machines
hold 0.75 of clerking, where the design asks for eight to nine tenths.

Both of those last two are what the full stage's computing good is for. Here
a machine's cost falls only by its quality, five per cent a year; in the full
stage the price of computing, which it runs on, falls a hundredfold besides,
and that is what should bring half into the 1990s and the end of the run to
eight or nine tenths. The making of electrical goods also limits how fast the
machines can be put in place: two lives (sixteen years) after the find the
works' gap at the margin is 1.69 to 1.78 on every island, not within a tenth
of one, as the works trail a target that rises with the pace, and the
machines' plant takes about twice the share of the island's electrical goods
that it took without them. On a class as wide as clerking (a factor of 3,000
across it) a small lag in the share is a large gap at the margin, as with the
hoist ([tasks.md](tasks.md)).

## What it shows

When, how fast and in which towns clerks' tasks go, in a run that has every
earlier rule. Office computing is found in 1960 on most islands, a few years
later on those with a small electrical trade, and takes a quarter of clerking
where it is found; the share then grows by about six hundredths a decade, as
its quality grows a twentieth a year against a wage that barely moves after
1975, and half is taken some fifty years after the find. Towns with dearer
labour take it first, on every island. The hands it frees from clerking stay
in the works and do the works' other tasks: by 2050 there are 42,700 fewer
clerks' hours a month on the mean island and 36,200 more at tending, judging
and presence, 2,600 more at hauling and 1,200 more minding the machines. The
first measurement (finds spread from 1960 to 1986) showed where the work came
from: the shops and trades, a quarter of whose labour is clerking, sold nearly
a fifth more at a sixth lower price, and the electrical works and the power
stations made more for the machines' plant and current. None of the hands go
onto the land or into the pits: the land's share of hands and the diggings'
do not move beyond chance. Work's share of income falls by a third of a point
to a point from 1990 (lower on five to seven islands of eight, never on all
eight), within the machines' bill: by the design's sum the machines that hold
clerking are paid under about an eighth of what its hands were paid, and
clerks' pay is about seven hundredths of income (a sixth of the works' hours,
the works two thirds of the hands, work two thirds of income), so the bill can
take about a point. So the move is the machines' own bill, not investment and
not hands pushed onto the land. The wage ends higher (14.82 against 13.90
times 1750's in 2050, on every island), because the same hands make more.

## Where it misleads

Its levels are not calibrated: one island's prices of 1960 and a target of a
quarter set them, and the floor that times its find is half of one island's
making. It only takes work: it gives none of the work the full stage's
computing trade will give (computer rooms, software, data halls), so work's
share and the wage read low against what the full stage should show; that the
freed hands find other tasks here is the works' recipes and the Engel rule
doing what they did before, not a new trade. With no falling price of
computing behind it, half of clerking comes at the edge of the design's band
and 2050 ends a tenth below it. The island has one wage a town, so nothing
here can show what becomes of clerks' pay against anyone else's. A town with
no power station has electricity only at its ceiling price and cannot run it.
Its plant wears at the life of the works' longest-lived machine (the electric
drive's twenty years, not its own eight), since a works keeps one book for
all its machines. And it is read on an island whose wage stands nearly still
after about 1975: its pace is then all that cheapens it.
