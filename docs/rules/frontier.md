# Births that answer to free land, and hamlets

Not in the Python. Two switches, both off under the port's rules and under
the game's, and both on with the landfall start ([landfall.md](landfall.md)):
`frontier.enabled`, births that answer to free land, and `frontier.hamlets`,
towns founded at a hamlet's size where the land is free. The code is
`frontier.rs`, with one call in `peopling::growth` and the founding's tests
in `peopling::found_towns` and `peopling::candidates`.

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
- `full` is one number for all land and all centuries. As farming improves a
  cell holds more people, and a later century's free land is less free than
  this says.
