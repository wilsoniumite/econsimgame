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

## The probe

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
