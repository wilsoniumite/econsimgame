# Births that answer to free land, and hamlets

Not in the Python. Three switches, all off under the port's rules and under
the game's, and all on with the landfall start ([landfall.md](landfall.md)):
`frontier.enabled`, births that answer to free land, `frontier.hamlets`,
towns founded at a hamlet's size where the land is free, and
`frontier.claimed`, land once taken stays taken. The code is `frontier.rs`,
with one call in `peopling::growth` and the founding's tests in
`peopling::found_towns` and `peopling::candidates`.

## Why

In history a colony with land to spare grew by its own births at 2.5 to 3
per cent a year, where an old country with much the same incomes grew at
well under one, and the rate fell as the land filled. People married younger
and had more children where a son could clear a farm of his own. From
memory, not from the repository:

- Benjamin Franklin, *Observations concerning the Increase of Mankind*
  (1751): the colonies double by their own births in about twenty to
  twenty-five years, because land is cheap and people marry young.
- French Canada grew from about 10,000 people in the 1680s to about 70,000
  by 1760 almost without immigration: some 2.5 per cent a year
  (Charbonneau and others).
- England grew about 0.5 to 1 per cent a year in the eighteenth century
  (Wrigley and Schofield, 1981).
- Yasuba (1962) and Easterlin (1976): in the United States before 1860 the
  ratio of children to women was highest where farm land was cheapest and
  fell as the land filled, in the east first.
- Colonial crude birth rates were about 45 to 55 a thousand, and deaths 15
  to 25.

The game's people ([people.md](people.md)) have one ceiling, the same on a
full island and an empty one: a town's births less deaths from income alone
is `growth_top * (1 - (y_still / y)^ease)`, with `growth_top` 0.016. On the
trial of 26 November an island begun from one town of 5,000 grew 1.2 to 1.4
per cent a year whatever the land, and reached 37,000 people by 1900.

## What it does

**Births.** Once a year, town by town, the ceiling is raised by
`births` (0.018) times the town's lifted share of free land:

- *Free land* is the fertile land within `reach` (20 cells, 40 km) of the
  town's market place in its own market area, below the mountains, each
  cell weighed by its fertility and by how far it is from holding `full`
  (10) hands at work on its land: `clamp(1 - hands / full, 0, 1)`. Ten hands
  a cell (4 km2) is about what the settled island has on its land in 1750,
  so the settled island is what full looks like at 1750's farming.
- *The lift* is that share times how far the town lives off its land: its
  hands on the land over all its hands at work, over `farmers` (0.5), at most
  1. The extra births are the farm families'. Half is about what the first
  town has on its land in its first years; history's colonies were nine in
  ten farmers, and the game counts their crafts, shops and services among
  its works.

The income term is multiplied by `1 + births * lift / growth_top`, so the
ceiling is 0.016 on a full island and 0.034 on an empty one: about a
colony's 50 births a thousand less 15 deaths. At the incomes where the
island's rate is about four fifths of its ceiling, that is 2.7 per cent a
year. At a bare living (`y_still`) births still just balance deaths, and the
transition ([transition.md](transition.md)), which multiplies the income
term, takes the extra births down with the rest.

**Hamlets.** Where a district that might become a town has half its land
free or more (the same measure, over the district's cells), it may be made a
town at `hamlet_people` (500) people, giving up to `hamlet_donor` (0.5) of
any one town's people, where the game's founding rule asks 3,000 and a fifth
(`found_people`, `donor_max`). Every other test of a founding stands: 40
cells, a saving in carting of 8 per cent of the district's wages, the donors'
eight years' rest, one town a year. A New England township was granted to
some fifty or sixty families, and a market town of the colonies was a few
hundred to a thousand or two (from memory). With one town and the game's own
numbers the second town waits until the first has 15,000 people: 1838 on
island 7 in the trial.

**Land that stays taken** (`frontier.claimed`). Without it a cell's free
share reads the hands at work on it this year, so land counts as free again
when its hands leave. They do leave, after 1950, when the machines empty the
farms: on islands 1 to 8 from landfall the colony's free share falls to 0.16
in 1900, then climbs to 0.32 in 1950 and 0.65 in 2050, and the frontier's
extra births come back a century after the frontier closed. That is a fault
of the measure, not a history: a farm worked by one man and a machine is not
land a son can clear.

With the switch on, each cell remembers the most hands its land has had at
any year's reckoning (`FrontierState::claimed`, one number a cell, made
before the year's shares are reckoned), and its free share reads that and
not this year's hands, for births and for a hamlet's district alike. Built
over by a street or a works, worked by machines, or grazed by a few men, the
land stays taken.

What becomes of land truly left: while a cell's land has gone back to the
wild (the map's use is `wild`: no farm, pasture, wood, pit, works or street
on it), the hands it is remembered to have had halve every
`left_half_life` years, 30, a generation. So land left wild for three
generations remembers an eighth of its hands: a cell that was full (ten
hands) is seven eighths free again. That is the old field gone back to
scrub, whose owners' heirs have moved away, as New England's hill farms did
after 1850 (from memory). I read "left" from the land's use and not from
its hands because the hands mislead: on island 7, of the 5,121 cells that
ever had five hands or more (the probe's maps every 25 years), 2,055 have
under half a hand in 2050, and of these 1,140 are streets and works, 792 are
farms and pastures worked by machines and only 123 have gone wild.

The save holds no state of its own (a saved game is its seed, its rules and
its orders, played again: `save.rs`), so the remembered hands are made again
when a game is loaded, year by year as they were made the first time.

Off, nothing here runs: `frontier::founding_needs` hands back the people's
own numbers and the income term is as it was, so the settled island is the
same to the bit. `State::frontier` keeps each town's free share and lift for
the probes.

## What it does to the settled island

Nothing: neither switch is on under the game's rules. Switched on there it
would move the island before 1901, since about half its land has fewer than
five hands a cell in 1750 (0.53 to 0.71 of cells have five or more on
islands 1 to 8 in 1800). So it is kept to the landfall start, as the brief
for it allowed. Whether the two starts should share it is the next piece's
question (the two starts as equals in the tests).

## What the runs showed

See [landfall.md](landfall.md), "What the runs showed": the arms "births as
today" and "births that answer to land" differ only by `frontier.enabled`.

**Land that stays taken**, 26 November 2026: `examples/landfall.rs` on
islands 1 to 8 to 2050, `--start landfall --set longrun.freight_from=1750`
(carriage on prices, the study's first mend, not yet the start's own),
before the switch and with it (with it, no settlers to a hungry port as
well). Means of the eight islands; births and deaths a thousand from the
ages' reckoning, means of the 25 years to the date:

| | before | land stays taken |
|---|---|---|
| free share 1900 / 1950 / 2050 | 0.16 / 0.32 / 0.65 | 0.06 / 0.03 / 0.04 |
| people 1900 / 1950 / 2050 | 558k / 1,280k / 2,338k | 493k / 1,051k / 1,731k |
| growth 2000 to 2050, a year | 0.36 per cent | 0.28 per cent |
| births / deaths a thousand, to 1950 | 19.8 / 10.6 | 18.8 / 10.7 |
| births / deaths a thousand, to 2050 | 14.7 / 11.6 | 13.8 / 11.8 |
| years with hunger, to 1900 / after | 1.0 / 1.0 | 1.6 / 0.6 |
| wage in grain 1800 / 1900 / 2050 | 14.3 / 32.3 / 92.5 | 14.6 / 30.2 / 101.1 |
| runs failed | 0 of 8 | 0 of 8 |

The free share no longer climbs after 1950: in 2050 it is at most 0.02
above 1950's on every island. It also falls faster in the nineteenth
century (0.06 in 1900, not 0.16), because land counts as taken from the
year its hands were most, and a cell's hands rise and fall from year to
year as its market draws on it; that takes 12 per cent off the colony's
people in 1900. The settled island cannot feel any of it: the frontier is
off there.

**Births that read the land filling**, a study arm, not built: the
transition's `z` read as everybody's living where the land is taken,
`y * (u + (1 - u) * (1 - free))` with `u` the townsfolk's share (the
finder's `transition.land_fills`). On the same eight islands, with land
that stays taken: 390k people in 1900 and 888k in 2050, growing 0.20 per
cent a year from 2000, but 3.6 years with hunger an island after 1900
against 0.6 without it. On the settled start (islands 7, 2 and 5) it
halves the island, 523k people in 2050 against 1,208k, its births falling
from 1850, since its land is full from 1750. Without it the colony already
grows under half a per cent a year from 2000 (0.28), so it is not needed
for that, and it is not built.

## What worries me

- Free land is measured by the hands on it, and on this island hands spread
  thin over the whole market area from the first year (every cell with a
  path joins a market). So land counts as partly free long after a town's
  country is farmed thinly all over, and the measure says more about the
  reach of a market than about settlement. A charted frontier (the next
  piece) would give it a better meaning.
- Before the lift was weighed by the hands on the land, births stayed lifted
  into the twenty-first century on an island whose people lived in towns:
  2.1 million people by 2050 against the settled island's 1.3 million. The
  weight is a judgement, `farmers` 0.5, not a measurement.
- Land that stays taken reads a cell's best year, and a cell's hands
  wander from year to year with its market: the most a cell ever had is
  more than it held for long. A mean over some years would be truer and
  was not tried.
- `full` is one number for all land and all centuries. As farming improves a
  cell holds more people, and a later century's free land is less free than
  this says.
