# Machines that reckon: office computing, a thin slice

Not in the Python. One `[[use]]` of `data/tasks.toml`, `office_computing`,
standing behind a switch of its own, `tasks.computing`: off under the port's
rules, on under the game's. It works through the rule for machines and tasks
as it stands ([tasks.md](tasks.md)) and adds no good, no method and nothing
drawn. It is a thin slice of the run to 2050, brought forward so that the
whole run has its age of machines that reckon, however plainly, before the
full stage is built.

**The standing check: 12 of 12 inside.** Nothing can be found before 1955,
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
from the year it is found, to a cap of 100 (never reached by 2050). It is
found at the first year's turn from 1955 at which the island has made 10.5
million electrical goods since 1750: island 7 had made 10.39 million by the
end of 1958 and 10.58 million by the end of 1959, so it finds it in 1960. Its
one `scale` was refitted once: at 1 the towns' target in the year found was
0.262 on the mean of islands 1 to 8, so a unit was made `exp(ln 3000 * (0.262
- 0.25)) = 1.10` times as dear. Every number is in the file's comment.

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
  electrical goods made alone.

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
land, on 19 November 2026 (`isostate-core sweep --seeds 1-16 --rules game
--families 6 --end-year 2050 --set tasks.computing=false,true --jobs 8
--standing`). Means over islands 1 to 8, the least and greatest in brackets;
islands 9 to 16 say the same unless noted.

| islands 1 to 8 | off | on |
|---|---|---|
| found | | 1973 [1960 .. 1986]: 1960 on islands 5 and 7, 1965, 1975 twice, 1982, 1983, 1986 |
| the towns' target in the year found | | 0.250 [0.247 .. 0.254] |
| clerking done by machine, 1970 / 1990 / 2010 / 2030 / 2050 | 0 | 0.073 / 0.269 / 0.427 / 0.553 / 0.677 [0.592 .. 0.759] |
| half of clerking taken | | 2022 [2009 .. 2036] |
| clerks' hours in works, 2050 | 61,300 | 23,100 |
| tending, judging and presence hours in works, 2050 | 285,600 | 315,700 |
| minders, 2050 | 3,630 | 4,730 |
| work's share of income, gross, 1990 / 2010 / 2030 / 2050 | 0.674 / 0.665 / 0.668 / 0.666 | 0.669 / 0.659 / 0.665 / 0.664 |
| the same net of wear | 0.696 / 0.689 / 0.692 / 0.690 | 0.693 / 0.684 / 0.689 / 0.689 |
| the chained real wage, 2050 against 1750 | 13.90 | 14.68 (higher on 8 of 8) |
| the machines' book, years of the wage bill, 2050 | 0.073 | 0.108 (higher on 8 of 8, every island apart) |
| hands on the land, 2050 | 0.174 | 0.176 |
| hands in works, 2050 | 0.658 | 0.656 |
| share living in towns, 2050 | 0.564 | 0.557 (lower on 8 of 8) |
| people, 2050 | 1.178 M | 1.178 M |

The design's three asks of the century after 1900, on islands 1 to 8: the
chained real wage grows 1.14 per cent a year from 1900 to 1970 off and 1.15
on; 2050 over 1970 is 1.13 off and 1.18 on; the share of hands on the land
falls by 0.076 off and 0.074 on. All met in both arms.

Against the design's bands: the towns' first share is 0.25 (met); clerking
done by machine rises at every mark from the decade after the find, on every
island (met); dearer towns go first, `dearer_first` positive on all sixteen
islands, 0.39 to 0.81 on islands 1 to 8 (met); but half of clerking is taken
in 2022 on the mean island, where the design asks for 1990 to 2010 (only the
islands that find it in 1960 take half by 2009), and by 2050 machines hold
0.68 of it, where the design asks for eight to nine tenths. The margin's gap
two lives after the find is 1.70 to 1.80 on every island, not within a tenth
of one: the works trail a target that rises with the pace, and on a class as
wide as clerking (a factor of 3,000 across it) a small lag in the share is a
large gap at the margin, as with the hoist ([tasks.md](tasks.md)).

## What it shows

When, how fast and in which towns clerks' tasks go, in a run that has every
earlier rule. Office computing is found between 1960 and 1992 by island,
on what the island has made of electrical goods, and takes a quarter of
clerking where it is found; the share then grows by about six hundredths a
decade, as its quality grows a twentieth a year against a wage that barely
moves after 1975, and half is taken some fifty years after the find. Towns
with dearer labour take it first, on every island. The hands it frees from
clerking stay in the works and do the works' other tasks: by 2050 there are
38,000 fewer clerks' hours a month on the mean island and 30,000 more at
tending, judging and presence, 2,000 more at hauling and 1,100 more minding
the machines. The shops and trades, a quarter of whose labour is clerking,
sell nearly a fifth more (the emporium's output 2030 to 2050 is 18.5 per cent
higher, at a price 16 per cent lower), the electrical works make a fifth more
and the power stations a tenth more, for the machines' plant and current.
None of them go onto the land or into the pits: the land's share of hands
and the diggings' do not move beyond chance. Work's share of income falls by
half a point to a point between 1990 and 2040 (lower on five to seven
islands of eight, never on all eight), within the machines' bill: by the
design's sum the machines that hold clerking are paid under about an eighth
of what its hands were paid, and clerks' pay is about seven hundredths of
income (a sixth of the works' hours, the works two thirds of the hands, work
two thirds of income), so the bill can take about a point. So the move is the machines' own bill, not
investment and not hands pushed onto the land. The wage ends higher (14.68
against 13.90 times 1750's in 2050, on every island), because the same hands
make more.

## Where it misleads

Its levels are not calibrated: one island's prices of 1960 and a target of a
quarter set them, and the find spreads over thirty years because the islands
make electrical goods at very different rates, which is what puts half of
clerking a decade past the design's band on the mean island. It only takes
work: it gives none of the work the full stage's computing trade will give
(computer rooms, software, data halls), so work's share and the wage read low
against what the full stage should show; that the freed hands find other
tasks here is the works' recipes and the Engel rule doing what they did
before, not a new trade. The island has one wage a town, so nothing here can
show what becomes of clerks' pay against anyone else's. A town with no power
station has electricity only at its ceiling price and cannot run it. Its
plant wears at the life of the works' longest-lived machine (the electric
drive's twenty years, not its own eight), since a works keeps one book for
all its machines. And it is read on an island whose wage stands nearly still
after about 1975: its pace is then all that cheapens it.
