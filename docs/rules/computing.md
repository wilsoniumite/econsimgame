# Machines that reckon: office computing, a thin slice

Not in the Python. One `[[use]]` of `data/tasks.toml`, `office_computing`,
standing behind a switch of its own, `tasks.computing`: off under the port's
rules, on under the game's. It works through the rule for machines and tasks
as it stands ([tasks.md](tasks.md)) and adds no good, no method and nothing
drawn. It is a thin slice of the run to 2050, brought forward so that the
whole run has its age of machines that reckon, however plainly, before the
full stage is built. Its second half, models, which take judging tasks from
2022 behind a switch of their own (`tasks.models`), is the last part of this
page ("Models: machines that judge"), with what the two halves show together.

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

# Models: machines that judge

The second half of the thin slice. One more `[[use]]` of `data/tasks.toml`,
`models`, standing behind a switch of its own, `tasks.models`: off under the
port's rules (and left out of their parameters' file), on under the game's.
Like office computing it adds no good, no method and nothing drawn.

**The standing check: 12 of 12 inside**, with models on and off, and with
both halves on against both off. Nothing can be found before 2022.

## What it is

**A machine for judging tasks.** Judging is design, selling, managing and
knowing when to sow: from an eighth to a third of every works' labour, a
quarter of the shops' and trades', and a tenth to a fifth of the land's
([tasks.md](tasks.md), "What it does"). Models may hold the whole class, from
the easiest task to the hardest, in the works and on all seven land sectors
(the farms, the pastures, the woods, the quarries, the clay pits and the
mines); the carters are passed over, as for every machine. Judging's tasks run
from those at which a hand is worth 10 machine units to those at which a hand
is worth 30,000, evenly in the logarithm between.

**Rented, with nothing behind the rent but goods.** A model owns no plant. A
use whose `plant` is empty is rented, and the rule treats it so: it has
nothing to wait for, so a works or a land sector holds what it means to hold
from the first month (a works founded after the find holds its town's
target in its first month at work); it orders nothing and adds nothing to the
works' book or the land's plant; and a works may drop it as fast as it likes
(the rule that a works' share falls no faster than its machines wear out
passes a rented use by, so a works whose town's target falls drops to it in
the month). Its whole cost is goods bought each month: seven tenths
electricity and three tenths electrical goods, by value. That rent is paid to
the power stations and the electrical works and comes to rest as their wages
and their owners' return.

**Its numbers.** Sized from island 7 in 2022, the year it finds it, under the
game's rules with six families and office computing on (a wage of 44.49 a
month, electricity 8.56 and electrical goods 26.58, the year's means over the
island). When found a unit costs 0.020 of a month's wage, 0.890: 0.0727
electricity and 0.0100 electrical goods a month. At that cost a hand is worth
50 units, so models take `ln(50 / 10) / ln 3000 = 0.20` of judging. Its quality
grows eight per cent a year from the year found; its cap of 1000 is never in
effect (1.08 a year for the 28 years to 2050 is 8.6). Its `life` is not read.
Its one `scale` was refitted once: at 1 the towns' target for judging in the
year found was 0.195 on the mean of islands 1 to 8, so a unit was made
`exp(ln 3000 * (0.195 - 0.20)) = 0.96` times as dear. Every sum is in the
file's comment.

**When it is found.** From 2022, the year the design gives, on an island that
has made at least 11.6 electrical goods a head since 1750 (`discover_per_head
= true`, as for office computing): half of what island 7 had made a head when
the year came, 23.28 by the end of 2021. Every island of sixteen had made 17.4
a head or more by then, so all sixteen find it in 2022.

**Judging only.** The design gives models what is left of clerking as well.
By its own numbers that holds nothing before 2050, so it is left out. On
island 7 in 2025 a unit of office computing costs 0.0054 of a month's wage and
a unit of model 0.0154, nearly three times as much; in 2050, 0.0015 and
0.0021. At eight per cent a year against five the gap closes only in the
2060s, and cheapest first (`work_out_parts`) models would hold none of
clerking.

**The switch, and the command.** `switch = "models"` in the file; while
`tasks.models` is off the use is never found and has no cost, part or order,
which `tests/tasks.rs` holds: island 7 to 2030 with the switch off is, to the
last digit, the run with models taken out of the table in code. Thrown off by
hand, every works and land sector drops its judging in that month. The facts
of the rule's loop come for it (`use_found_year_models`,
`use_share_found_models`, `use_auto_after10_models`, `dearer_first_models`;
`use_margin_gap_two_lives_models` is none, its life being unread), with two
new facts for every use: `target_CLASS_works_Y`, the towns' target in the
works, which the standing works move toward, and, for a use that reaches the
land, `use_auto_land_after10_NAME`, its class on the sectors it reaches ten
years after the find (none for models in a run to 2050, which reaches only 28
years past the find). The command that shows it:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set tasks.computing=false,true --set tasks.models=false,true --end-year 2050 --jobs 8 --standing --out DIR
```

## What it leaves to the full stage

- **The model works and the computing they make.** In the design a model's
  capital is the model works and its power, owned works with years of output
  in plant, so that the rent comes to rest with their owners. Here the rent
  goes to the stations and the electrical works, which are not model works.
- **The rest of clerking**, which models take from office computing only once
  they are the cheaper unit (above).
- **Robots in works and robots that walk.** Tending, the largest class in the
  works, has no machine after the drive in either half (the combine tends on
  the farms and pastures only), because neither the line nor robots are built.
- **The dial turned along a curve**, from a year and over years, for sweeps.
- **A statute that opens presence** to machines.

## How the full stage deepens it

Without throwing it away. The use keeps its name, its class, its switch and
its place in the file. Its run moves to computing alone, made by the model
works; it is sized again by the same rule (a unit costs 0.020 of a month's
wage when found, rented, all of it run goods) and its `scale` is refitted
once. A second entry gives it what is left of clerking from where office
computing stops. Until those are built the sandbox's Rules tab can turn
`tasks.dial` by hand once the viewer's charts are on main, which is the
nearest the island has to the design's question of how far the dial has
turned by 2050.

## What the islands showed

Islands 1 to 16 to 2050 under the game's rules with six families, the new
land, on 20 November 2026, the command above with `--seeds 1-16`. Means over
islands 1 to 8 with the least and greatest in brackets, office computing on in
both columns; islands 9 to 16 say the same unless noted.

| islands 1 to 8 | models off | models on |
|---|---|---|
| found | | 2022 on every island of sixteen |
| the towns' target for judging in the year found | | 0.200 [0.186 .. 0.208] (9 to 16: 0.206) |
| judging done by machine, 2030 / 2040 / 2050 | 0 | 0.162 / 0.291 / 0.399 [0.388 .. 0.408] |
| in the works | | 0.165 / 0.293 / 0.401 |
| on the land | | 0.156 / 0.284 / 0.393 |
| the towns' target for judging in the works | | 0.276 / 0.377 / 0.476 |
| clerking done by machine, 2050 | 0.748 | 0.755 (higher on 8 of 8) |
| judges' hours in works, 2050 | 106,000 | 69,800 |
| tending, presence, hauling and minding hours in works, 2050 | 253,000 | 275,900 |
| work's share of income, gross, 2030 / 2040 / 2050 | 0.663 / 0.652 / 0.663 | 0.661 / 0.655 / 0.656 (lower on 7 of 8 in 2050) |
| the same net of wear | 0.687 / 0.677 / 0.688 | 0.686 / 0.681 / 0.681 |
| the works' owners' share of income, 2050 | 0.123 | 0.134 |
| the chained real wage, 2040 / 2050, against 1750 | 14.30 / 14.82 | 15.08 / 15.37 (higher on 8 of 8) |
| of the island's electricity, what machines for tasks run on, 2050 | 0.414 | 0.564 |
| of its electrical goods, 2050 | 0 | 0.152 |
| hands on the land, 2050 | 0.171 | 0.183 (higher on 6 of 8) |
| hands in works, 2050 | 0.657 | 0.645 (lower on 8 of 8) |

Against the bands: the towns' first share is 0.20 (met); every island finds
models within ten years of every other (all in 2022: met); judging done by
machine is above nothing from the mark after the find and rises at every mark
to 2050, in the works and on the land, on every island of sixteen (met);
dearer towns take it first, `dearer_first_models` positive on all sixteen
(0.20 to 0.81). The design's "about half of judging by 2050" is not reached:
0.40 on the mean island, every island between 0.39 and 0.41. The towns' target
is near half by 2050 (0.476), and the works trail it by a tenth in 2030 and
three quarters of a tenth in 2050, the lag of their eight years of adopting
behind a target that rises with the pace. That shortfall is what the full
stage is for: computing whose price falls, and the model works that make it.

The design's three asks of the century after 1900, on the mean of islands 1 to
8 (the chained real wage's growth a year from 1900 to 1970; its 2050 over its
1970; the share of hands on the land in 2050 less in 1900):

| | 1900 to 1970 | 2050 over 1970 | land |
|---|---|---|---|
| neither half | 1.14 % | 1.129 | -0.076 |
| office computing alone | 1.15 % | 1.191 | -0.079 |
| models alone | 1.14 % | 1.171 | -0.070 |
| both | 1.15 % | 1.236 | -0.067 |

Islands 9 to 16: 1.13 and 1.14 per cent a year; 1.147, 1.188, 1.188 and 1.264;
-0.073, -0.070, -0.077 and -0.068. All met in every arm.

## What the two halves show together

On one island, the classes go to machines in turn: hauling from 1901 (the
drive, then the tractor and the cutter), two thirds of it by 1990 and no more
after; clerking from 1960 (office computing), 0.38 by 1990 and three quarters
by 2050; judging from 2022 (models), two fifths by 2050. Tending stays near
an eighth throughout, the combine's share of the farms', in both arms. Set
against the island with neither half, work's share of income, gross, is
lower by a point from 1990 (0.665 against 0.674 in 1990, 0.656 against 0.666
in 2050 on islands 1 to 8; on islands 9 to 16, 0.660 against 0.679 in 2050,
lower on all eight), and the same net of wear (0.681 against 0.690 in 2050).
The machines' part went to the owners of works: their share of income is
0.134 against 0.114 in 2050, higher on all eight islands, for office
computing's plant and for the stations and electrical works that models'
rent is paid to; the split of the works' book between families, landlords and
households hardly moves. The hands freed (41,400 fewer clerks' hours a month
and 22,300 fewer judges' in 2050) stay mostly in the works at their other
tasks (34,700 more tending, 7,500 more presence, 5,100 more hauling, 1,800 more
minding), and with models about a hundredth of the island's hands goes back
onto the land on islands 1 to 8 (0.183 against 0.171 in 2050, higher on six of
eight; on islands 9 to 16, 0.177 against 0.175). The wage ends a tenth higher (15.37 against 13.90
times 1750's in 2050, higher on all eight islands), because the same hands make
more and the machines that hold judging run on goods the island makes.

## Where the two halves mislead

Everything said above for office computing holds for both. Models' rent goes
to makers who are not model works, so the owners who gain are the stations'
and the electrical works', and the design's model works are nowhere. Models
run on electricity and electrical goods at prices that did not move to meet
them: by 2050 machines for tasks take more than half the island's electricity
and a sixth of its electrical goods, where the full stage has a computing
trade that grows to meet its own demand. Tending, the largest class in the
works, has no machine after the drive in either half, because neither the line
nor robots exist yet, so the order in which the classes go is not the
history's: here clerking and judging go to machines while the works' tending
stays by hand, and the freed hands find work in it. The island has one wage a
town, so nothing here speaks of the gap between pay for judging and pay for
anything else. And the levels are not calibrated: one island's prices in 2022
and a first share of a fifth set them, and the pace of eight per cent a year
is the design's, not tuned to reach half by 2050.
