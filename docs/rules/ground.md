# Ground: room that clears, and how far a town may reach

Not in the Python. A stage of the run to 2050: from the end of 1900 a town's
house rent clears the room it has built, and a town takes no ground farther
than a walk, or later a tram ride, from its centre. The switch is
`ground.enabled`, with `ground.room` and `ground.reach` for its two halves:
off under the port's rules, on under the game's, with no belt. The code is
`ground.rs`, with hooks marked "ground" in `towns.rs` (where a town grows and
its room is counted), `econ.rs` (where house rent is paid), `gauges.rs` (the
fork's room) and `facts.rs`. It acts only from 1901, on one kind of household
(the town's townsfolk), with no car and no belt unless one is set.

**The standing check: 12 of 12 inside, and nothing of 1900 moves.** Nothing
of the rule acts before 1901, so the played century is the same island to the
last digit with it on and off:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set ground.enabled=false,true --standing --out DIR
```

```text
standing check: 12 of 12 inside (0 moved, 0 OUTSIDE, 0 without a value): ground.enabled=true against ground.enabled=false
```

Of the 1,283 facts of the sixteen runs to 1900, the only one that differs is
the rule's own `ground_burden_worst`. Switched off, it is the island without
it: islands 1 to 4 to 2050 with `--set ground.enabled=false`, set against the
build before it, "of the 3555 facts both have, the seconds aside, 0 differ".
The game's golden test moved in its second era only (1901 to 2050).

The gate's three asks of the century after 1900 are met with it on (islands
1 to 8: the chained wage grows 1.50 per cent a year from 1900 to 1970, 2050
is 1.22 times 1970, the land's share of hands falls 0.076). Of the design's
passes for the rule itself, none is met but the standing check (below, "The
design's passes").

## What was wrong

Under the towns' rule ([towns.md](towns.md)) a town's house rent is a fixed
share of the wage a head (`urban_rent_rate`, 0.12 of the wage for each of its
townsfolk, less toward the edge), shared among its cells by `1 - distance /
fringe`. A town takes its best free neighbour whenever its people outgrow its
room, and every cell's room rises with the calendar (6 people a cell a year,
from 530 in 1750 to 1,430 in 1900 and 2,330 in 2050). So room is never
scarce, rent cannot rise as a share of what people earn, and nothing makes the
ground under a large town dear, which is what the later stages of the run to
2050 (the price of land, a dividend the ground takes a share of, towers in
the picture) need.

## What it does

**The day it takes over.** At the end of 1900 (`from`) each town's rent a
unit of room is taken as it stands: the towns' bill over the room the town
has. A unit of room is the room of one person at the ceiling of 1900. The
months of 1901 are the first to pay the new rent.

**Room is counted by what is built** (`ground.room`). From 1901 the calendar
no longer raises a cell's room: it is held at its ceiling of 1900 (a young
cell still fills in toward it as before), times the cell's `dens`. `dens`
starts at 1 and moves toward the square root of the cell's rent a unit of
room over its town's rent of 1900, both over the long run's anchor, by at
most 2 per cent of itself a year (`dens_rate`), kept between 1 and `floors`
(1.5), or up to `floors_frame` (3) where the town knows the steel-frame
builders: the top is `1.5 + 1.5 * know`, `know` being the share of the town
that knows them, as the woods' faster regrowth is read
([woods.md](woods.md)). A cell's rent a unit of room is the town's, shared out
by `1 - distance / fringe` as the rent is collected, so the centre is dearer
than the town's mean and is built up first. A cell no town holds loses what
was built on it.

**Rent that clears.** The townsfolk want at least `room_min` (0.3) of a unit
of room a head, and spend `beta` (0.09) of what is left of their budget after
the subsistence bundle and that least room on more. With `n` townsfolk, `b`
the households' budget a head, `C` the cost of the subsistence bundle a head
at the town's prices and `S` the town's room:

```text
rent a unit of room = n * beta * (b - C) / (S - room_min * n * (1 - beta))
```

where the denominator is not positive, the rent at which the townsfolk would
pay half their budget. Nothing iterates. The month's rent moves a third of
the way to this. Each head takes `room_min * (1 - beta) + beta * (b - C) /
rent` and pays the rent of it, never more than half its budget, so the
town's bill is `n * (rent * room_min * (1 - beta) + beta * (b - C))`. From
1901 to 1920 (`blend_years`) what is charged moves evenly from the old rule's
bill to this one: a twentieth of the way in 1901, all of it from 1920. What
is collected is shared among the town's cells by `1 - distance / fringe` and
paid to their owners, the town's landlords or the families, as before.

**Reach** (`ground.reach`). From 1901 a cell may join a town only within
`reach_walk` (5) cells of the town's centre, its meeting place, in a straight
line; 8 (`reach_tram`) where the tramway is found and the town has 30,000
townsfolk (`tram_people`). Distance is counted as the designers counted it
when they chose 5 and 8: in a straight line across whatever lies between, so
a cell across a lake, a tideway or a river from the centre is as near as one
on the same bank at the same distance. Ground a town held beyond its reach in
1900 stays its own. `belt`, when set, is the most cells a town may add in a
year; unset (the game's), there is none. When no cell can join, the town
stops growing outward and its rent and its `dens` rise instead.

The core's table of 20 November 2026 (`docs/core.md`, islands 1 to 8 to
2050) found reach idle: with it off 2,620 of 3,678 facts differ and none
stands apart, and the growth after 1900 is 1.51 per cent a year, 1.21 and
-0.085, all met. It stays on: it is a day old, and the picture of 2050 is to
read where a town may grow.

Room without reach changes the level of rent and nothing else: under the
towns' rule a town takes a cell whenever its people outgrow its room, so its
room follows its townsfolk, and with room near the townsfolk the rent's
share of their budget is fixed by `beta` and `room_min` (below, "Why the
burden is flat"). Reach without room hardly binds at all. The two are
measured together and apart below.

**The tramway** is found on the island as a whole, as office computing is
([computing.md](computing.md)): not before 1901 (`tram_after`), and then in the
first year the island has made 0.02 electrical goods a head since 1750
(`tram_floor`, the office machines' floor: the electric drive's threshold of
15,000 made over island 7's people in 1893). Every town knows it from then.
It is not an invention of the tree: an invention added to the content would
move the record of the played century (its list of inventions) and draw a
number on the inventions' stream every year it is sought.

**The fork's room.** The gauges' fork ([gauges.md](gauges.md)) gains the wage
over the rent of a unit of room, 1750 = 1: the year's house rent over the
room the towns have, beside the wage over made goods and raw goods. It is
reckoned from 1750 and given in the gauges' year only after 1900: before it
rent is a fixed share of the wage a head, so the figure says little, and the
run file of the played century is left as it was.

**What the record keeps for the picture** (since 20 November 2026). A record
past 1900 of a run that counts room by what is built carries two more layers,
a byte a square a year (`record::GROUND_LAYERS`): `dens`, as hundredths above
1, and the square's land value, its rent a unit of room over its town's rent
of 1900 (both over the anchor), on a scale of twentieths of a doubling from a
sixteenth (`ground::land_value`, which reads the state and changes nothing;
it is the square of what `dens` moves toward). The viewer builds each lot of
a town after 1900 by them (the viewer's `scatter/towns/kept/lots.rs`). A live
game sends the same bytes each year (20 kB). On island 7 to 2050 they are
3.03 MB each raw and 0.10 MB each gzipped, 2.8 per cent of the record's 213 MB
raw; a record to 1900 and the port's carry neither.

## The numbers

| | | |
|---|---|---|
| `from` | 1900 | the last year of the old rule |
| `blend_years` | 20 | 1901 to 1920 |
| `room_min` | 0.3 | units of room a head at least |
| `beta` | 0.09 | of the budget left after subsistence and the least room |
| `dens_rate` | 0.02 | of itself a year |
| `floors`, `floors_frame` | 1.5, 3 | the most `dens` may reach, without and with steel frames |
| `reach_walk`, `reach_tram` | 5, 8 | cells from the centre |
| `tram_people` | 30,000 | townsfolk for trams |
| `belt` | unset | cells a town may add in a year |
| `tram_after`, `tram_floor` | 1901, 0.02 | the tramway's year and electrical goods made a head |

All but the last two are the design's, untuned.

## What the islands showed

Islands 1 to 8 and 9 to 16 under the game's rules with six families, to 2050,
the rule off and on, as run on 20 November 2026:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set ground.enabled=false,true --out DIR
```

| islands 1 to 8 | off | on |
|---|---|---|
| house rent over the townsfolk's budget, 1900 | 0.163 | 0.163 |
| the same in 1920, 1950, 2020, 2050 | 0.165, 0.164, 0.161, 0.165 | 0.109, 0.110, 0.109, 0.108 |
| in the largest town, 1950 and 2020 | 0.159, 0.160 | 0.113, 0.110 |
| town cells, 1950 and 2050 | 545, 583 | 498, 522 |
| `dens` in the largest town, 1950 and 2050: mean (highest) | | 1.30 (1.52), 1.37 (1.58) |
| its rent a unit of room over its own of 1900, real, 1950 and 2050 | | 1.77, 1.92 |
| land's share of income, 1950 and 2050 | 0.167, 0.163 | 0.149, 0.141 |
| what a wage buys of all goods, 1970 and 2050 (1750 = 1) | 16.4, 19.9 | 16.8, 20.5 |
| what a wage buys of room, 1970 and 2050 (1750 = 1) | 1.27, 1.75 | 1.88, 2.08 |
| the chained real wage, 2050 (1750 = 1) | 18.8 | 19.4 |
| people, 2050 | 1.139 million | 1.120 million |
| living in towns, 1950 and 2050 | 0.546, 0.550 | 0.545, 0.545 |
| the tramway found | | 1903 [1901 .. 1909] |

Islands 9 to 16 give the same: the burden from 0.166 to 0.110 by 1920 and
0.108 in 2050, against 0.162 to 0.167 off; town cells in 2050 527 against
559; the tramway in 1901 on ten islands, 1908 and 1909 on two, and 1923 on
island 11, whose electrical works this history finds only in 1916. The worst
month of any town is about a quarter of its budget (0.23 and 0.26 on the
mean island of each eight, 0.29 at most); with a belt one town on one island reaches the half.

**Today's rule takes more than the design reckoned.** The design put the
old rule's burden at 0.10 to 0.11 of a budget, about what the new one takes.
On the island, read from the books, it is 0.163 in 1900 (0.157 in the
largest town), and it stays there to 2050 with the rule off. So the twenty years of blending take the burden down by a
third, from 0.163 to 0.109, and house rent with it: land's share of income
is about two points lower from 1920 on, and what was rent is spent on
goods (baskets a head are 1 per cent higher in 1910, 3 in 1920 and 3.5 in
1930).

**Why the burden is flat.** Under the towns' rule a town takes a cell
whenever its people outgrow its room, so its room follows its townsfolk, and
`dens` adds room wherever rent rises. With room equal to the townsfolk, the
bill over their budget is `beta * (b - C) / b * S / (S - room_min * (1 - beta)
* n)`, which is `0.124 * (b - C) / b`: about 0.11 for as long as the
subsistence bundle is a tenth of a budget, whatever the wage. The burden
moves only where a town presses against its ground, and after 1950 none
does: the island's people level off near 1.1 million and the towns' share
near 0.55, so the largest towns stop growing (Mallowbrook on island 7 has
73,000 townsfolk in 1950 and 58,000 to 60,000 after), and a cell a year is
more than they want.

**Room and reach apart.** Room without reach changes the level of rent and
nothing else; reach without room changes nothing beyond chance:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set ground.room=false,true --set ground.reach=false,true --out DIR
```

| islands 1 to 8, room / reach | off / off | off / on | on / off | on / on |
|---|---|---|---|---|
| the largest town's burden, 1950 and 2020 | 0.159, 0.160 | 0.165, 0.162 | 0.112, 0.110 | 0.113, 0.110 |
| town cells, 2050 | 583 | 588 | 526 | 522 |
| what a wage buys of room, 2050 | 1.75 | 1.75 | 2.06 | 2.08 |

What the room does is the rent's level and the 60 cells the towns no longer
take once what they have is built up. Reach hardly binds: the walk of five
cells holds nine tenths of 1900's town ground already (below), and the tram
reaches eight cells from about 1903, more than any town takes after 1900.

**Reach on the new land.** Counted as the designers counted it, in a
straight line from each town's meeting place, the share of 1900's town ground
beyond five cells is 0.103 [0.064 .. 0.135] on islands 1 to 8 and 0.102
[0.059 .. 0.190] on 9 to 16 (37 cells of 359), and beyond eight 1.5 and 1.3
per cent; on the old island (`--set land.enabled=false`) 0.127 and 0.095,
where the designers had 0.12 to 0.15 on four islands of an older build. So 5
and 8 still fit and are kept: on the day it takes over, the walk holds the
largest towns where they stand until the tram, and no ground is unbuilt.

**A belt.** A belt of one or two cells a year from 1901, or one drawn in
1950 (`isostate-core lab --rule ground.belt=1@1950`), hardly binds: on two
islands of sixteen no town wants a second cell in a year until 1998 and 2028.
Drawn in 1950 it raises the largest town's burden by a tenth of a point in
2020 and four tenths in 2050 (6 islands of 8 higher), the island's by three
tenths in 2050 (8 of 8), and its rent a unit of room by 12 per cent in 2020;
the chained wage of 2050 is 3 per cent lower (6 of 8).

**The caution.** The room rule changes baskets a head in every town, and
births, the Engel rule and migration read baskets. Over the blend baskets a
head rise a little faster with the rule (11.0 against 10.8 in 1910, 12.3
against 11.9 in 1920, 13.7 against 13.2 in 1930), and the people's growth
falls a little faster (9.06 a thousand a year off and 8.92 on in 1920, 7.04
and 6.80 in 1930). No year jumps: the gap between the arms widens by under a
tenth of a thousand a year. The twenty years were enough for that; what does
not go away is the level, a third less house rent.

## The design's passes

| pass | asked | measured, islands 1 to 8 (9 to 16) | |
|---|---|---|---|
| the standing check | 12 of 12 inside | 12 of 12 | met |
| the burden in 1920 within two points of 1900's | 0.143 to 0.183 | 0.109 (0.110) | missed |
| lower in 1950 than in 1920 | below 0.109 | 0.110 (0.110), flat | missed |
| the largest town's burden in 2020 half as high again as in 1950 with a belt, and not without | 1.5 times | belt 1: 0.110 against 0.113; without: 0.110 against 0.113 | missed |
| an hour buys more goods and less room after about 1970 | room falls | goods 16.8 to 20.5, room 1.88 to 2.08 (1.93 to 2.10) | missed |

The first fails on the old rule's level, not the new one's; the other three
fail because, with the people levelling off after 1950 and towns that take
ground as they need it, room is never scarce. Nothing was tuned to meet them.
The rule is on under the game's rules all the same: it does what it says,
the gate's asks are met, and the later stages (sites priced off the ground,
the price of land, a car's reach and a belt by statute) are what can make
room scarce.

## What is not done, and what worries me

- The works' sites on town ground are still priced at the old rule's rent
  (`towns::price_ground` reads the towns' bill): the next stage (sites
  reviewed, the price of land) is where they meet the new rent.
- `dens` stays well under either ceiling: 1.6 at most in the largest towns
  of sixteen islands, most years. Towers will not come from this rule while
  the towns do not press on their ground.
- The record's people a cell are packed in sixteens to 4,080 (2050's layout):
  a cell at the ceiling of 1900 built up threefold would hold 4,300 and its
  byte would saturate. Not reached today (1,741 at most in island 7's largest
  town).
- One kind of household. The middling sort and the landlords, who would bid
  for the same ground at 0.12 and 0.18, wait for `owners.kinds`.
- The tramway makes no news.

## Facts

- `rent_burden_Y` and `rent_burden_largest_Y`: the year's house rent over the
  townsfolk's budget (the households' spending and house rent, times the
  townsfolk's share of the town's people), the island's and the largest
  town's (most townsfolk), read from the books, so a run with the rule off has
  them too; in 1880, 1890 and at the marks from 1900.
- `town_cells_Y`: town ground as the record keeps it.
- `cells_beyond_reach_1900`, `cells_beyond_reach_share_1900`,
  `cells_beyond_tram_1900`: 1900's town ground beyond the walk and the tram,
  counted in a straight line from the town's centre.
- `wage_in_goods_Y` (the gauges' fork over all goods) and `wage_in_room_Y`
  (over a unit of room), 1750 = 1.
- The rule's own: `dens_largest_Y` and `dens_largest_max_Y` (the mean and the
  highest `dens` of the largest town's cells), `rent_room_largest_Y` (its rent
  a unit of room over its own of 1900, both over the anchor),
  `found_year_tramway`, and `ground_burden_worst` (the highest share of a
  budget any town paid in any month).

`examples/ground_profile.rs` writes, for the largest town in the years asked,
each cell's distance from the centre, its `dens`, room and people:

```bash
cargo run --release -p isostate-core --example ground_profile -- --seed 7 --years 1950,2000,2050 --out profile.csv
```

## Tests

`tests/ground.rs`: with the rule on, island 7 to the end of 1900 is the run
with it off (`save::check`, every month of every series to the bit, the
towns' ground and bills); after 1900, with a belt of one and no tramway, no
cell joins a town farther than five cells from its centre and no town takes
two cells in a year, money is conserved every month (the run's own check),
and the burden never passes a half in any town in any month. The port's
golden test is untouched; the game's moved in its second era only.

## After 1900: room that grows dear

Under the game's rules the room a head takes above its least falls with the
rent a unit of room as `(rent / its rent of 1900)^(-0.5)` times this page's,
so that the share of the spare budget spent on room is `beta` in 1900 and
rises as the town's room dears, where by this page's rule it is `beta`
whatever the rent. The rent that clears is the closed form of that demand.
The switch is `city.dear_room`, on under the game's rules, with
`city.room_elasticity` (1 is this page's rule); the rule and what it does to
the burden and to `dens` are in [city.md](city.md).

## A town fills its ground (since 21 November 2026)

The designer, on the pictures of the first city: dense towns should have no
patches of farmland in them, parks yes, and the cities should be properly
dense. The switch is `ground.fill`, with `ground.park` and `ground.outbid`:
off by default, on under the game's rules. It acts only in the yearly steps
after 1900 and only on squares a town takes from then on, so the played
century is the same island to the last digit and nothing jumps in 1901.
The code is in `ground.rs` ("filling") with hooks marked "ground" in
`towns::update_urban`, and the record's two new layers in `record.rs`.

**Why the squares were partly built.** Measured on islands 1 to 8 with the
rule off (`examples/ground_fill.rs`, below):

- A square joins its town with room for 300 people and fills in toward the
  ceiling of 1,430 over some thirty years (`towns::room_at`), so a town that
  grows fast stands on many squares a fifth to two thirds built: in 1925 the
  first city's mean square was 0.72 built, and 0.15 of the ground inside its
  outline was town ground with nothing on it.
- A town's people are shared over all its squares by room (`towns.rs`,
  `per_room`), so wherever room outruns the people (`dens` rising with the
  rent, or a town that has lost people and keeps its ground) every square
  holds fewer people, not the edge alone.
- The picture draws a square's houses from its residents over its `dens`,
  where each head takes the room the rent clears, more as incomes rise. So
  in 2050 the first city's squares, which the core has built whole (0.97 of
  its outline built, 0.003 open), are drawn with paddocks between the houses.
  No part of a town's square is ever farmed in the core (`land.rs` takes town
  ground from every use of the land); what is farmed inside a town's outline
  is the squares no town holds, 0.6 per cent of the first city's in 2050.

**What it does.** From 1901, when a town takes a square, it sets what it would
pay for the whole square (its rent a unit of room as the last month ended,
times the room of a square built whole, less the park and the works' part)
against what the plough would pay for it (`towns::plough_rent`: farm, pasture
and wood bidding for it as open land). Where the town pays at least
`outbid` (1) times as much, the square is built whole at once: its room is the
ceiling of 1900 less its park, times its `dens`, and not 300 filling in. Where
the plough pays more, the square joins as before and fills in with its age,
so a town on cheap ground still spreads in ribbons along its roads. Which
square a town takes is as before: the best free neighbour by the way to
market, the road and the water. The town takes a square only when its people
outgrow the room it has, so its squares are full before it spreads, and once
it can take no more (its reach, or no ground left) its rent and its `dens`
rise, as this page's rule already has it. The town's mean rent is used, not
the cell's share of it by `1 - distance / fringe`: that share is the old
rule's way of dividing the bill and prices the edge at nothing by
construction, so no edge square would ever outbid the plough under it.

**Parks.** A square built whole keeps `park` (a tenth) of its ground open: the
last of its ground to fill, which the picture can take as the end of the
square's order of building, the ground farthest from the market by way. The
park takes room from the square, so its rent falls on the rest. Who pays for
a park is a later piece, with the state's public works; the wet ground by a
river is where the fuller stage of sites and the price of land may put more.

**What the record says.** A record past 1900 that keeps the ground's layers
(`record::GROUND_LAYERS`) now has four: `dens`, `value`, `fill` (the share of
the square its buildings stand on, by room against a square built whole, in
250ths) and `park` (the share kept open, in 250ths). A square with a park is
one built whole; the works' part is in `works`, as before. A live game sends
the same bytes (`LiveLayers::fill`, `park`). On the record the picture can
draw a filled square as filled; it does not yet (it still draws houses from
residents, `world::town_houses`).

| | | |
|---|---|---|
| `fill` | true under the game's rules | a square taken after 1900 is built whole where the town outbids the plough |
| `park` | 0.1 | of a square built whole, kept open |
| `outbid` | 1 | the town fills where its bid is at least this many times the plough's |

None was tuned.

**What the islands showed.** Islands 1 to 8 under the game's rules with six
families, to 2050, the rule off and on:

```bash
cargo run --release -p isostate-core --example ground_fill -- --seeds 1-8 --years 1900,1925,1950,2000,2050 --set ground.fill=false --out off.csv
cargo run --release -p isostate-core --example ground_fill -- --seeds 1-8 --years 1900,1925,1950,2000,2050 --out on.csv
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set ground.fill=false,true --standing --out DIR
```

The outline of a town is its squares and the land no town holds that they
close round (a hole in the town, or a square with five of its eight
neighbours in it). Shares are of the outline, on the mean of the islands:

| the first city, off / on | 1925 | 1950 | 2000 | 2050 |
|---|---|---|---|---|
| squares | 106 / 86 | 115 / 103 | 109 / 100 | 101 / 104 |
| built | 0.768 / 0.887 | 0.885 / 0.936 | 0.947 / 0.940 | 0.966 / 0.934 |
| park | 0 / 0.016 | 0 / 0.024 | 0 / 0.023 | 0 / 0.017 |
| town ground left open | 0.148 / 0.025 | 0.070 / 0.009 | 0.018 / 0.003 | 0.003 / 0.009 |
| farmed (squares no town holds) | 0.062 / 0.052 | 0.028 / 0.013 | 0.014 / 0.010 | 0.006 / 0.019 |
| its squares under half built | 0.112 / 0 | 0 / 0 | 0.004 / 0 | 0 / 0.006 |
| `dens` at the middle and the edge | 1.25, 1.12 / 1.30, 1.14 | 1.69, 1.39 / 1.83, 1.44 | 1.92, 1.51 / 2.07, 1.63 | 2.21, 1.83 / 2.14, 1.69 |

Every town, the town ground left open: 0.136 and 0.046 in 1925, 0.084 and
0.018 in 1950, 0.018 and 0.011 in 2050. From the sweep: town squares on the
island 526 and 434 in 1950, 490 and 414 in 2000, 467 and 463 in 2050; the
burden 0.171 and 0.185 in 1950, 0.216 and 0.199 in 2050 (the largest town's
0.173 and 0.191, 0.225 and 0.208); hands on the farms 0.135 and 0.140 in
1950, 0.143 and 0.140 in 2050; people in 2050 1.152 and 1.168 million. The
standing check: 12 of 12 inside. The three growth asks are met on both arms:
1.22 and 1.22 per cent a year from 1900 to 1970, 2050 over 1970 1.13 and
1.07, the land's share of hands falling 0.103 and 0.106. No run failed. A run
to 2050 takes as long as before (69 to 79 s on both arms, sixteen at once).

So the rule does its work while towns grow: the first city stands on a fifth
fewer squares in 1925 and a tenth fewer in 1950, and its open town ground falls from 0.15 and
0.07 of its outline to under 0.03. By 2050 the island's towns had filled in
by age with the rule off too, and the two arms are alike; the first city's
farmed holes are 0.019 of its outline with the rule on, against 0.006, on a
city that took other ground.

**What is not done.** The picture still draws a square's houses from its
residents, so it shows paddocks inside squares the core has built whole
(the next piece). The ground of a young square on cheap ground is still
wholly the town's in the core while the picture draws fields on its open
part; letting the plough work it until it is built on would make that
honest. Who pays for a park waits for the state's public works. The fuller
stage (sites priced off the ground, the price of land) will set a square's
bid by its own place and not the town's mean, and may put parks on the wet
ground by the rivers.
