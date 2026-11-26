# Machines that tend: the line, the automatic machine and the robot

Not in the Python. Four `[[use]]` of `data/tasks.toml` and four ideas of the
tree in `data/game.toml`, behind one switch, `tasks.tending`. It is **off under
the port's rules and off under the game's rules too**: it was built to be
switched on if it lifted the wage of 2050 over 1970's to 1.20 on islands 1 to 8,
and it lifts it from 1.07 to 1.13, while the hands it frees go to the carts and
the fields and the towns' share of people falls by seven points ("Why it stands
off", below). A hand can turn it on in a sandbox, on the Rules tab's row
`tasks.tending`, and a sweep can with `--set tasks.tending=true`. It works through
the rule for machines and tasks as it stands ([tasks.md](tasks.md)) and adds no
good, no method and nothing drawn.

**With the lorry** ([lorry.md](lorry.md), on since 26 November 2026) tried again on islands 1 to 8 with
the lorry on: 2050 over 1970 reaches 1.20 (1.14 with the lorry alone; with pace since, 1.16 and 1.18),
but the towns' share in 2050 is 0.493 against 0.550 with both off (with pace, 0.488 against 0.556), so
tending still stands off; that page says what it waits on.

**Where the hands go** (traced on 26 November 2026, below): the freed tenders go to the land, the pits
and the carts because one wage clears each town for its works and its land, so hands follow pay bills,
and the land's and the carts' pay bills are a fixed share of income once their own machines reach their
caps. A second switch, `tasks.tending_wide`, lets the robot and the line that runs itself reach the land,
the pits and the carts too; it narrows the towns' fall from eight points to four and lifts 2050 over 1970
to 1.28, but the towns still fall, so both stand off.

**Tried again with care** ([care.md](care.md), on since 26 November 2026; "Tried again with care", below):
the slumps that doubled with tending were traced to the power stations, whose own machines ran on current
bought a month late from a market they were starving. That is mended (`tasks.own_run`, on under the game's
rules), and the extra slumps halve. But the mend shows that part of what tending seemed to give, a better
wage and fuller towns, leaned on that shortage: with the stations' machines running, the line that runs
itself takes the whole of tending after 2030, the wage of 2050 falls back to 1970's and the towns to 0.50 to
0.52 against main's 0.556. So tending stays off, with its wider reach.

**Tending** is feeding and minding a process, fitting and driving: the largest
class of the works' hours, 0.37 of them in 1900 and 0.44 in 2050 on main
(islands 1 to 8), and the one no machine took in the works before this rule.
The drive took hauling from 1901, the office machines, office computing and
models took clerking and judging, and the hands they freed went to tending: by
2050 tenders were two fifths of all hands, the works' and the land's together
([pay.md](pay.md) counts half, with the machines' minders and presence). In a real country the
moving line, the automatic machine and the robot multiplied what one tender
makes. This rule is that history.

## The standing check

Nothing can be sought before 1908, so the played century cannot move:

```bash
isostate-core sweep --seeds 1-16 --rules game --families 6 --end-year 2050 --set tasks.tending=false,true --standing --out DIR
```

```text
standing check: 12 of 12 inside (0 moved, 0 OUTSIDE, 0 without a value): tasks.tending=true against tasks.tending=false
```

and the two arms are the same island to the end of 1907, which
`tests/tending.rs` holds on island 7. With the switch off the four ideas are
never sought and draw nothing, and the run is the island without the rule: set
against main before it, a sweep of islands 1 to 16 to 2050 with the switch off
differs in none of the 4,060 facts both have.

## What it is

Four machines for tending tasks, in the works alone, each found once the island
has found its idea of the tree and the machines it needs:

| use | its idea, history's year | needs | reach | first share | a unit costs, of a month's wage, when found | plant, months of wages | made of; run on | life | pace (cap) |
|---|---|---|---|---|---|---|---|---|---|
| the moving line | `moving_line`, 1913 | the electric drive | 0.35 | 0.20 | 0.25 | 9.65 | machinery, electrical goods; electricity | 15 | 0.03 (4) |
| the automatic machine | `automatic_machine`, 1958 | | 0.70 | 0.40 | 0.063 | 2.61 | machinery, electrical goods; electricity | 12 | 0.04 (10) |
| the robot | `industrial_robot`, 1982 | office computing | 0.90 | 0.55 | 0.022 | 0.92 | machinery, electrical goods; electricity | 12 | 0.05 (30) |
| the line that runs itself | `lights_out_line`, 2035 | models | 1.00 | 0.92 | 0.0017 | 0.06 | machinery, electrical goods; electricity | 8 | 0.08 (1000) |

**Each takes from the easiest task.** Every one has `from = 0`, as office
computing has: the cheapest unit holds the class from the bottom and each
dearer one what lies above it, so when a later machine is found it holds a
little more than the one before had reached (its first share, the towns'
target in the year found, is sized "a little more"), and once its pace has
made it the cheaper it takes the older machine's tasks: the robot took the
line's as the computer took the typewriter's. A machine whose `from` stood
above nothing would leave the tasks below it unheld whenever it was the
cheapest, which is why none does.

**Their numbers.** Each was sized from island 7's prices about the year it is
found, on main's sweep to 2050 of 25 November 2026, as a unit's cost a share of
a month's wage: six or seven tenths of it the return a works must see and the
wear on its plant, a tenth or a twentieth its minders, the rest electricity.
The line's numbers are the design's (a quarter of a wage, nine to ten months'
wages of plant, a first share of a fifth); the robot's are near the design's
(a fiftieth of a wage where the design has a thirtieth, from the easiest task
where the design has it from 0.45); the automatic machine and the line that
runs itself are this rule's own, set between and after. Each `scale` was
refitted once on islands 1 to 8, so that the towns' target in the year found
is the first share in the table: the towns gave the line 0.166 at a scale of
one, the automatic machine 0.437, the robot 0.611 and the line that runs
itself 0.967, so the line was made 0.79 times as dear and the others 1.30,
1.52 and 1.39 times. The last refit is rough: its share stands against the
reach of the whole class. Every sum is in the file's comment.

**Made by somebody.** Their plant is machinery and electrical goods, bought
from the island's engine works and electrical works through the works'
orders for machines ([tasks.md](tasks.md), "A machine works once it
stands"), and their running is electricity bought from its stations, so the
machine makers and the stations gain the work.

**Adoption** is the office machines' pattern, unchanged: a works means to hold
its town's target and moves toward it by an eighth of the gap a year, as fast
as its order for plant is filled, and lets it fall no faster than its machines
wear. A town's target is where its wage over a unit's cost falls on the
schedule of tending (a hand is worth one to a thousand machine units across
the class), so dearer towns take more.

## How each is found

**By the tree's own rules.** Each machine waits on an idea of the tree
(`idea` in the file), an invention of stage 6 in `data/game.toml`, found as
every invention is: a hazard each year that rises as the island's making of
the goods it names passes their threshold ([tree.md](tree.md)), in the town
that made most of them, and spread between towns along the roads. The use is
found at the turn of the year after its idea, once the uses it `needs` are
found too. The thresholds were set by hand on what islands 1 to 8 had made by
each idea's floor year (`found_after`), so that on most islands the clock has
run out by the floor and the hazard, near its most, finds the idea some five
years on:

| idea | made since 1750 | floor | found, islands 1 to 8 (9 to 16) |
|---|---|---|---|
| the moving line | 6 million machinery and 1 million electricity | 1908 | 1918 [1910 .. 1935] (1918) |
| the automatic machine | 50 million machinery and 5 million electrical goods | 1952 | 1962 [1952 .. 1972] (1958) |
| the industrial robot | 14 million electrical goods | 1978 | 1984 [1979 .. 1993] (1983) |
| the line that runs itself | 30 million electrical goods | 2030 | 2033 [2030 .. 2043] (2033) |

Two islands of the first eight are late to the line, island 1 (1928) and
island 8 (1935), because they made electricity late (none before 1915). The
use waits a year on its idea, and the robot also on office computing, the
line that runs itself on models.

**Draws of their own.** An idea a machine waits on draws its year's numbers
from a stream made afresh from the seed, the idea's name and the year
(`rng::idea_stream`), not from the tree's stream. So nothing need be saved,
and every other idea of the tree draws the numbers it drew before: with the
switch on the island differs from the island without it by the machines
alone. With the switch off the four ideas are not sought at all
(`tasks::idea_sought`), and the run is main's to the last digit.

**Two keys a use may now carry.** `idea` (a `[[tech]]` of the content by name;
a use whose idea the content lacks is not laid, as a good of a later stage) and
`needs` (uses by name that must have been found before it). Every earlier use
carries neither and is found as before.

## What the islands showed

Islands 1 to 16 to 2050 under the game's rules with six families, the switch
off and on (the command above; 32 runs, none failed, 217 s). Means over
islands 1 to 8, the least and greatest in brackets, and how many islands of
eight are higher with the switch on; islands 9 to 16 say the same unless noted.

| islands 1 to 8 | off | on |
|---|---|---|
| tending done by machine in the works, 1930 / 1950 / 1970 / 1990 / 2010 / 2030 / 2050 | 0 | 0.19 / 0.32 / 0.39 / 0.53 / 0.66 / 0.81 / 0.96 |
| half of tending done by machine | | 1986 [1976 .. 1997] (9 to 16: 1982) |
| the linked real wage against 1750, 1970 / 2050 | 14.09 / 15.22 | 15.02 / 17.06 (7 of 8) |
| per cent a year, 1900 to 1970 | 1.25 | 1.35 |
| 2050 over 1970 | 1.07 [0.87 .. 1.28] | 1.13 [1.00 .. 1.32] |
| hands on the land, 2050 less 1900 | -0.112 | -0.088 |
| tending's share of the works' hours, 2050 | 0.44 | 0.03 |
| judging's and hauling's, 2050 | 0.30, 0.07 | 0.54, 0.12 |
| the machines' minders, 2050 | 0.019 | 0.057 |
| work's share of income, 2050 | 0.61 | 0.51 (lower on 8 of 8) |
| hands on the land, 1970 / 2050 | 0.116 / 0.140 | 0.121 / 0.152 |
| carters' share of hands, 2050 | 0.088 | 0.130 (8 of 8) |
| share living in towns, 2050 | 0.54 | 0.47 (lower on every island of sixteen) |
| people, 2050 | 1.168 M | 1.167 M |
| a works' size, median / ninetieth centile, 2050 | 4.9 / 37.3 | 5.1 / 35.4 |
| slumps in the news (verdicts) | 13.4 | 17.4 (7 of 8) |
| a run's processor seconds | 101.6 | 102.7 |

The table of each decade from the probe (`examples/tending.rs`, below) adds what
one hand makes. At 1950's prices a hand in the mills and factories made 97 a
month in 1970 and 121 in 2050 on main; with the rule 125 and 426. Across all
the works, offices among them, 47 and 70 on main, 52 and 122 with the rule.

**What a tender makes** grows as no other hand's does: in the mills and
factories, where tending is six tenths of the hours, three and a half times
from 1970 to 2050 with the rule against a quarter more without. **Where the
wage goes.** The linked wage is 7 per cent higher in 1970 and 12 per cent
higher in 2050, so the century after 1970 gains 6 points (1.07 to 1.13) and the
worst island rises from 0.87 to 1.00; the years before 1970, which have the
line, gain more (1.25 to 1.35 per cent a year).

## Who loses

Pay by class, reckoned at the gauge's dials (a bend of 0.3, ten years to
follow the work: [pay.md](pay.md)), islands 1 to 8:

| | off | on |
|---|---|---|
| a tender's pay over the mean, 2050 | 1.20 | 0.94 (lower on 8 of 8) |
| a tender's real pay, 2050 over 1970 | 1.35 | 1.20 (lower on 7 of 8) |
| a hauler's | 1.70 | 2.08 |
| a clerk's | 0.63 | 0.71 |
| a judge's | 0.72 | 0.87 |
| the schooled over the plain, 1970 / 2050 | 1.32 / 0.62 | 1.45 / 0.77 |

The tender loses, against everyone else: with no machine in the works'
tending, tenders were the class every freed hand went to, and their pay rose
above the mean (1.20 by 2050); with the machines their pay falls below it
(0.94), and their real pay from 1970 grows a fifth where it grew a third,
while every other class's grows faster than without them. The schooled still
fall below the plain by 2050, but less far (0.77 against 0.62), because the
plain now include the tenders whose work the machines take. The owners of the
works gain most: work's share of income is nine points lower in 2050.

**Where the tenders go.** Not into other works. The works' hands with the rule
are 304 thousand in 2050 on the mean island against 350 without; the land's,
with its minders, are 155 thousand against 130, and the carters 76 thousand
against 51 (the probe's counts, islands 1 to 8). No machine reaches the
carters, and the land's machines are the tractor, the combine and the cutter
of 1915 to 1935, so the hands the works no longer want go where hands are
still wanted by hand: the carts and the fields. That is what empties the towns.

## Why it stands off

It was to be switched on under the game's rules if, with it on, 2050 over 1970
on the mean of islands 1 to 8 came to 1.20 with the other growth asks met. It
comes to 1.13. The three asks of the century after 1900 are met with it on
(1.35 per cent a year; 1.13; a fall of 0.088 in the land's share), and the
worst island's 2050 over 1970 rises from 0.87 to 1.00, so it costs no ask.
It stands off for what it does to the island besides: the towns' share of
people falls from 0.54 to 0.47 on every island of sixteen, the carters rise
from nine to thirteen hundredths of all hands, the land's share stops falling
after 1970, and the news has a third more slumps. Those are the island
lacking a machine for the carts (the design's motor lorry, 1910) and trades
in the towns that take the freed hands (the design's schools, infirmaries and
travel), not faults of this rule; and with them it would be a different
island. So it waits for them, behind its switch, and the Rules tab offers it.

**Nor does it lean on the methods' learning.** The step back of 20 November
found the ceilings after 1970 complements. Tending is not one of them: lifting
the cap on the methods' learning from 1970 (`isostate-core lab --rule
learn_cap=16@1970`, islands 1 to 8) adds 1.49 to the linked wage of 2050
with the rule off and 0.32 with it on, and the rule adds 1.84 with the cap as
it is and 0.66 with it lifted. The two are substitutes: both save hands in the
works, and once machines hold the tending there are fewer to save.

## Where the hands go

Traced on 26 November 2026, on main with the lorry, pace and who works on, islands 1 to 8 to 2050 under
the game's rules with six families, tending off and on. Every number is the mean of the eight islands
over the five years about the mark, from the books (`examples/tending_trace.rs`, below).

**Where a freed tender is found.** Of every hundred hands the works no longer want in 2050, 63 are on the
land (38 of them at the pits: coal 19, clay 12, stone 8; 26 on the farms and pastures) and 32 carting;
5 are for hire and not hired. The works' share of all hands is 0.678 off and 0.570 on; the land's 0.230
and 0.298; the carters' 0.065 and 0.100. The count of townsfolk is a count of the works' hands
(`towns.rs`: a town's people times 0.04 plus 0.8 of the share of its hands in works), so it falls with
them, from 0.571 to 0.488. A hand goes to the land for want of a place in the works, never for its pay:
one wage clears each town's market for its land and its works together, and pay a hand on the land over
pay in the works is 0.98 to 1.00 in every decade of both arms.

**Why: hands follow pay bills.** With one wage, a sector's share of the hands is its share of the pay
bill, and its pay bill over income is what it sells over income times its pay over what it sells. In
2050, tending off against on:

| | sells, share of income | pay over what it sells | pay bill, share of income |
|---|---|---|---|
| mills and factories | 0.322 / 0.332 | 0.169 / 0.049 | 0.055 / 0.016 |
| furnaces | 0.093 / 0.101 | 0.387 / 0.178 | 0.036 / 0.018 |
| builders | 0.166 / 0.178 | 0.218 / 0.100 | 0.036 / 0.018 |
| power stations, electrical, oil and nitrogen works | 0.101 / 0.150 | 0.469 / 0.222 | 0.047 / 0.033 |
| offices | 0.352 / 0.334 | 0.678 / 0.587 | 0.239 / 0.196 |
| the land (seven sectors) | 0.248 / 0.249 | 0.57 / 0.58 | 0.141 / 0.146 |
| the carts | | | 0.046 / 0.057 |

What each sector sells hardly moves: the households' needs are shares of what is left above
subsistence, the same in both arms (comfort 0.38, food 0.105 in 2050), so a good made cheaper is bought
more and the money spent on it stays. What moves is the works' pay over what they sell, which the
machines cut by more than half. The land's is fixed by its Cobb-Douglas share once its own machines
(the tractor, the combine, the cutter) reach their caps, as they have by 1970. So the land's and the
carts' share of the island's pay bill rises from 0.31 to 0.41, and the hands with it. The machines'
own needs feed it: they run on electricity made from coal (coal into the stations nearly doubles, 8.0 to
15.4 million a year), and the island's building, where their plant stands and the owners' larger incomes
go, grows by half, of stone, clay and lime dug at the pits and carted.

The same runs with tending off, smaller: from 1990 to 2050 office computing and models cut the offices'
pay over what they sell from 0.78 to 0.68 and the mills' from 0.22 to 0.17, while the land's pay bill
holds, so the townsfolk fall from 0.603 to 0.571 there too. Machines in the works are the cause in both
arms; tending is the largest of them.

**The candidates**, each turned off and on (the data altered for an experiment, swept on islands 1 to 8
with tending off and on):

| | towns 2050, off / on | the gap |
|---|---|---|
| main | 0.570 / 0.488 | 0.082 |
| the richer shilling leans further to comfort (0.58, from shelter and household goods) | 0.591 / 0.512 | 0.079 |
| the owners spend as households do | 0.605 / 0.516 | 0.089 |
| the four machines that tend reach the farms and pastures too | 0.577 / 0.496 | 0.081 |
| ... every land sector | 0.570 / 0.516 | 0.054 |
| ... every land sector and the carts | 0.566 / 0.520 | 0.046 |
| the robot and the line that runs itself alone reach every land sector and the carts (`tending_wide`) | 0.570 / 0.532 | 0.038 |

- *The budget is closed at the top*: true, a level and not the gap. The needs' shares are flat after
  1970 in both arms, since the lean toward comfort ends in 1970 and real income a head falls after it
  (17.0 subsistence bundles in 1970, 13.1 in 2050: fewer hands a head, and a rising rate on pay). More
  comfort lifts the towns in both arms alike.
- *The gains go to owners whose spending does not come back as work in towns*: the gains do go to owners
  (dividends +0.059 of income, the rent of works' sites +0.047, the merchants +0.031), and owners spend
  twice a household's share on shelter, which is building, dug and carted. But owners who spend as
  households do lift the towns in both arms alike. Not the gap.
- *The machines' savings do not reach prices*: with shares of spending fixed, a price that fell further
  would leave the money spent, and so the hands, where they are. (Part of the saving stays with the works:
  the mills' profit over what they sell is 0.20 off and 0.26 on.)
- *The land takes any hand at some pay*: true, and it is the way the hands go; but not idleness in
  disguise: for hire and not hired rises only from 0.027 to 0.032 of hands, and the land's extra hands
  make grain that is eaten and brewed (43.9 to 49.9 a head a year) and dig coal and stone that is used.
- *Services are made that nobody buys*: every sixth unit of services is lost in store (0.18 to 0.19 of
  their value), in both arms and every decade alike: a standing waste, not the drift.
- *The land has no machine of its own*: true, and the one candidate that narrows the gap, most where the
  pits and the carts are reached as well.
- *The towns are dear*: house rent is 0.12 of households' income from 1970 and falls with tending;
  townsfolk a cell hold at 1,500. Not it.

**The cure tried: `tasks.tending_wide`.** The robot and the line that runs itself reach the tending of
every land sector and of the carts as well as the works' (`wide` in `data/tasks.toml`): the milking robot
and the tractor that steers itself, the shearer at the coal face, the depot's robots, and from the 2030s
the field, the pit and the lorry that run themselves, as the design's robots that walk. The moving line
and the automatic machine stay in the works (the automatic machine on the land too narrowed the gap by
another 0.004). The switch is off under both sets of rules and does nothing while `tasks.tending` is off:
on islands 1 to 8 and 17 to 32 the run with it on and tending off is the run without it, none of 112,000
facts different.

| | islands 1 to 8: tending off | on | on with the cure | islands 17 to 32: off | on | on with the cure |
|---|---|---|---|---|---|---|
| townsfolk 2050 | 0.570 | 0.488 | 0.532 | 0.575 | 0.497 | 0.541 |
| hands on the land 2050 | 0.140 | 0.164 | 0.146 | 0.130 | 0.162 | 0.146 |
| 2050 over 1970 (the linked wage) | 1.18 | 1.16 | 1.28 | 1.20 | 1.22 | 1.36 |
| per cent a year 1900 to 1970; the land's share 2050 less 1900 | 1.26; -0.107 | 1.35; -0.082 | 1.34; -0.100 | 1.30; -0.122 | 1.40; -0.091 | 1.40; -0.107 |
| what is made (linked index, millions), 2050 | 107 | 148 | 175 | 112 | 159 | 184 |
| work's share of income, 2050 | 0.62 | 0.50 | 0.50 | 0.63 | 0.50 | 0.50 |
| a tender's pay over the mean, 2050 | 1.21 | 1.00 | 0.60 | 1.20 | 1.00 | 0.62 |
| slumps in the news (verdicts) | 8.6 | 16.6 | 18.3 | 10.6 | 18.4 | 18.3 |
| years of hunger in the news | 17.6 | 21.5 | 3.1 | 19.1 | 17.9 | 3.7 |

The standing check is 12 of 12 inside in every arm. **It stands off, with tending.** The pass was: with
tending on, the townsfolk no fewer than with it off, what is made a head higher, the three growth asks
met, the twelve inside and the slumps no more. With the cure the towns still fall four points, and the
slumps in the news double (as they do with tending alone), so the pass is missed on two counts.

**Who gains and who loses** with the machines tending and the cure: the haulers' real pay in 2050 is 31.8
times the wage of 1750 against 19.9, the judges' 20.4 against 13.1, the clerks' 14.0 against 11.0, and the
tenders' 12.3 against 20.8, less than they had in 1970 (14.0); work's share of income falls from 0.62 to
0.50, and the land's (rents) rises from 0.175 to 0.220 and the works' (profits) from 0.163 to 0.214. The
land keeps more of the hands (0.258 against 0.230) and the towns fewer.

**What it waits on.** The towns can hold their share under machines that tend only if the hands they free
go to work in towns that richer households buy more of and no machine takes: what the design calls care,
schools, travel and the other trades of presence. Today the island's one comfort is services, made by
offices whose clerking and judging the machines take too, with no presence in them; and a new good cannot
yet be added after 1900 without moving the played century (lorry.md). Until then the cure halves the fall
and leaves it.

**The hunger after 1990.** Under main as it plays (the lorry on), the news reports hunger in a town
nearly every year from about 1994 (17.6 years of it a run on islands 1 to 8; 0.5 with the lorry off, on
the lorry's own sweeps). It is not bread. On island 7 in 2029 to 2047 (`examples/hunger.rs`) every town
that went hungry but one is a small town of 60 to 700 people with fifteen to seventy months' grain a head
in store and every order filled; in its worst month its households bought no grain at all, because their
purse had gone below nothing. The lorries' oil and plant are charged to a town's households after their
month's spending, with no bound (`econ.rs`, the carting's goods paid out of the households' cash): in
Brackhurst (92 people) in August 2046, the month the harvest is carted, the bill came to 113 a head
against 17 in hand, the purse ended at -68 a head, and the next two months' budgets were nothing. On
island 7 from the lorry's coming to 2046 the lorries' bill passed the households' purse in 887
town-months, 154 of them by more than 5 a head. (The one other was the first city, Melness, in 2043, at
80 per cent of its food: a true shortage, its merchants' grain at 1.6 months a head.) Tending does not change it
(21.5 years); the cure cuts it to 3.1, by the robots holding the carts' tending, which was not traced
further. The mend belongs to the lorry's accounting, not here.

## Tried again with care

On 26 November 2026, after care ([care.md](care.md)) was switched on to hold the freed hands in the towns:
islands 1 to 8 and 17 to 32 to 2050 under the game's rules with six families, care on, machines that tend
off, on, and on with the wider reach:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set tasks.tending=false,true \
    --set tasks.tending_wide=false,true --standing --out DIR
```

Every run passed with its money checked every month, and the standing check is 12 of 12 inside in every arm
on both sets of islands, so nothing before 1908 moves. The hunger the morning saw is gone in every arm (under
one year in a run, since the hauliers' till, [lorry.md](lorry.md)).

### The slumps

With machines that tend the news told twice the slumps (16.1 and 16.9 verdicts a run on islands 1 to 8,
against 7.9 without them), and four times the depressions. The news's slump clock reads what is made, linked
([measures.md](measures.md)), and replayed year by year with each fall split by good at the link's prices,
every extra slump is after 1970, and the good that falls most is services, the offices' making, in four
alarms of five.

**What falls first is current.** Month by month on island 7 (`examples/slump_trace.rs`, below), the
island's making swings up and down over two or three years, and with it the market's fill of electricity
(what the stations made over what was asked): 0.99 at the top, 0.88 and 0.77 at the bottom. Coal, oil and
paper are never short. A works' machines stand idle in the measure their current is short ("A machine short
of its run good", [tasks.md](tasks.md)), so where machines hold the tending every works makes less, the
offices most. The stations had a tenth of their capacity to spare all along, and their coal; what they
lacked was their own current. Their machines (the line, the robot, office computing in their offices) ran on
electricity bought from the market, which a station makes and sells and cannot keep: so in a month of short
current the stations' machines stood idle too, the stations made less, and the next month was shorter
still. Their hands were hired for last month's fill, so a short station also took more hands a unit, its
margin fell and it ran less of its plant. The shortage fed itself for a year and then turned. Without
tending the stations' machines held little of their work and the loop was too weak to swing the island.

That is a fault of the rule's timing, not the economy's truth: a power station runs its machines on what it
makes. **The mend, `tasks.own_run`** (on under the game's rules; nothing under the port's, where the tasks
rule is off): a works whose machines run on the good it makes runs them on its own making, so the market's
fill of that good does not idle them; of that good it buys from the market no more than the market's fill,
and its machines take the rest from what it makes. Today only the stations make a run good. (What the
stations' machines take from their own making is not taken from what they sell: their machines' current
times the market's shortfall, not measured.) It acts from 1901, when the first machine runs on anything, and
moves the game's golden hashes of 1901 to 2050.

On island 7 with tending the swing is gone: from 2000 to 2023 what is made moves by 2.4 per cent a year at
most, where it moved by 3 to 7 up and down, and the stations run 0.98 of their plant, where they ran 0.90.
A shortage of current is left, steady from year to year though it flickers from one month to the next: the
market's fill stays near 0.92 from 2000 to 2050 (0.82 at the lowest), with the stations at nearly all their
plant, so their building does not keep up with what the machines ask. That was not traced.

| islands 1 to 8 | tending off | on | on, wider reach |
|---|---|---|---|
| slumps (news verdicts), before the mend | 7.9 | 16.1 | 16.9 |
| with it | 8.8 | 10.5 | 11.2 |
| depressions (a fall of 8 per cent or more) after 1900, a run, before / with | 0.0 / 0.1 | 2.5 / 0.4 | 4.4 / 0.9 |
| islands 17 to 32, slumps with the mend | 10.1 | 12.3 | 13.9 |

With the mend tending adds two slumps a run on islands 1 to 8 and on 17 to 32 (12.3 against 10.1), the wider
reach three to four. What is left is the offices' own: their tending, clerking and judging all held by
machines, they make each month what the market will take with few hands to steady them, and services still
lead the falls. That was not chased further.

### What the mend showed

With the stations' machines running, tending is a different island:

| 24 islands (1 to 8 and 17 to 32), the mend on | main (care off, tending off) | care, tending off | care, tending on | care, on with the wider reach |
|---|---|---|---|---|
| townsfolk 2050 | 0.556 | 0.589 | 0.498 | 0.521 |
| what a wage buys, 2050 over 1970 (linked) | 1.17 | 1.17 | 1.00 | 1.02 |
| what is made a head, 2050 over 1970 | 1.15 | 1.10 | 1.23 | 1.36 |
| slumps (verdicts) | 10.6 | 9.7 | 11.7 | 13.0 |
| the growth asks | met | met | the wage's missed on islands 1 to 8 (0.94) | the wage's missed on islands 1 to 8 (0.96) |

Before the mend the arms with tending had a better wage than without (1.23 and 1.28 on these islands) and the
wider reach kept the towns above main's (0.567). Every year of short current idled the machines and hired
the hands back. With the stations' machines running, the wage holds to 2030 and falls after it, when the line
that runs itself reaches the whole of tending: on islands 1 to 8 with the wider reach the linked wage is 16.6
in 2010, 16.5 in 2030 and 13.8 in 2050, work's share of income 0.46 and profits 0.29. A trial with the line's
reach at nine tenths, the robot's (islands 1 to 8, not landed), lifts the wage of 2050 over 1970 to 0.99 and
1.01 and the towns to 0.490 and 0.528: not enough.

**Who gains and who loses** (the mend on, islands 1 to 8 and 17 to 32, means; tending off / on / on with the
wider reach):

| | tending off | on | wider reach |
|---|---|---|---|
| a tender's real pay, 2050 over 1970 | 1.29 / 1.31 | 0.97 / 1.11 | 0.85 / 1.00 |
| a hauler's | 2.12 / 2.07 | 2.17 / 2.28 | 2.44 / 2.55 |
| a clerk's; a judge's | 0.66; 0.76 / 0.66; 0.78 | 0.51; 0.62 / 0.54; 0.68 | 0.51; 0.66 / 0.57; 0.75 |
| the mean, which care's hands are paid (one wage clears each town) | 1.16 / 1.18 | 0.94 / 1.03 | 0.96 / 1.05 |
| the schooled over the plain, 2050 | 0.60 / 0.61 | 0.60 / 0.61 | 0.63 / 0.66 |
| work's share of income, 2050 | 0.64 / 0.62 | 0.45 / 0.49 | 0.46 / 0.45 |
| rents (the land's share), 2050 | 0.18 / 0.17 | 0.19 / 0.20 | 0.20 / 0.20 |
| profits (the works' share), 2050 | 0.16 / 0.16 | 0.29 / 0.26 | 0.29 / 0.26 |
| hands in care, 2050 | 0.165 / 0.169 | 0.177 / 0.180 | 0.188 / 0.196 |

(Islands 1 to 8 / 17 to 32.) Everyone who works loses against the arm without the machines but the haulers,
whose lorries the wider reach drives; the clerks and judges lose most, since office computing and models took
their tasks already and now the tenders crowd in. What work loses, profits gain: the works' share of income
nearly doubles. Care's hands grow (0.17 to 0.19 of all hands), at the one wage.

**So it stays off.** The pass for switching it on was: the towns of 2050 no lower than main's less 0.02
(0.536); what a wage buys in 2050 over 1970 no lower than with tending off (1.17); what is made a head
clearly higher; the three growth asks and the twelve; slumps no more than two above tending off. With the
mend, made a head (1.23, 1.36), the twelve and, for tending alone, the slumps (+2.0) are met; the towns (0.498,
0.521), the wage (1.00, 1.02) and the wage's growth ask on islands 1 to 8 are missed.

**What is still missing.** The gains of machines that tend go to profits (0.26 to 0.29 of income), and
nothing on the island turns profits back into work in the towns: the owners' spending has no share on care
and twice a household's on building, and the works invest what the machines need, which is plant and current.
And the line that runs itself, at a reach of the whole class, leaves no tending to hands at all. The next
tries, in order: owners who buy care as households do (care.md, "What worries me"); a reach for the line below
the robot's, or a cap; then tending again.

## The probe

`examples/slump_trace.rs` plays one island to 2050 under the game's rules with six families and any setting
by name, and writes a row a month from a given year for each good named: its works' capacity, the share of it
run, their fill and hands, the merchants' stock and sales, the price, the month's making and the market's
fill; and for the island the wage and what is made, linked (the slump clock's series):

```bash
cargo run --profile work -p isostate-core --example slump_trace -- 7 2000 OUT.csv services,electricity,coal tasks.tending=true tasks.own_run=false
```

`examples/tending_trace.rs` plays one island to 2050 under the game's rules with six families and any
setting by name, and writes a long table of its books from 1900, a year at a time: the island's income
account, each good, land sector and method summed over the towns, the towns' table summed, and the
townsfolk, the hands and the tasks' year at each turn. `examples/hunger.rs` writes a row a month for every
town whose households went short in the year, with its grain, its purse and its income:

```bash
cargo run --profile work -p isostate-core --example tending_trace -- 7 OUT.csv tasks.tending=true tasks.tending_wide=true
cargo run --profile work -p isostate-core --example hunger -- 7 2029 2047 OUT.csv
```

`examples/tending.rs` plays one island to 2050 under the game's rules with six
families and any setting by name, and writes a row a year from 1900 for each
kind of works (its hands, what they made at the year's prices and at 1950's, the
number of works) and for the whole (the hands by class, what machines did, the
land's hands, the carters, the linked wage and each tending machine's find):

```bash
cargo run --release -p isostate-core --example tending -- 7 OUT.csv tasks.tending=true
```

## What worries me

- **The kinds of works are not told apart.** Every works' tending is open to
  every machine, the shops' and the offices' as the mills'. The line suited
  the works that make things in many alike; here it takes the easiest fifth of
  a shop's tending as soon as a mill's. The file has no key to say which kinds
  a use reaches; the offices, which hold six tenths of the works' hands by
  2050, have a tenth of their hours tending, so the shops and offices take
  little of the machines' gain.
- **No larger works.** The moving line made works larger, and here a works'
  size is its trade's, not its machines': the median works is a little larger
  with the rule and the largest tenth a little smaller.
- **What runs it.** The robot and the line that runs itself have no computing
  good to run on; like office computing and models they run on electricity and
  are made of electrical goods.
- **Almost all of tending by 2050** (0.96), where the design's robots reach
  nine tenths: the line that runs itself, found from 2030, reaches the whole
  class, and the pace of eight per cent a year carries it there in fifteen
  years. A cap or a reach below one would hold some back.
- **The thresholds are set by hand**, not by the tree's fitter, on one sweep's
  making; when the island changes they should be fitted with the rest.
