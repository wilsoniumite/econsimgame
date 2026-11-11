# Town ground that is kept

`towns.enabled`, in `crates/isostate-core/src/towns.rs`. Off under the port,
on under the game's rules. None of it is in the Python. It builds on the life
cycle for works ([works.md](works.md)) and is measured here with that on.

Under the port a town has no ground of its own. Every year it is given as
many cells as its people need at that year's density, picked afresh from the
cheapest to reach, and the density is made to rise about as fast as the
people. So the towns of 1900 stand on as many cells as those of 1750, a cell
at the edge is town one year and farm the next, and the player's own town
goes from sixteen cells to eight while its people double. With kept ground a
town keeps what it builds and grows outward from it: 63 cells of town in
1750, 231 in 1900, and a cell that joins a town stays.

## Why the port's towns do not grow

The diagnosis is in `.claude/night/reports/dx-towns.md`, measured on eight
seeds. In brief:

- **The count of cells is flat by arithmetic.** A town's cells are its town
  people over the year's density (`urban_density0` 500, rising 14 a year), and
  the density rises 5.2 times while the people rise 4.6 times.
- **The churn is noise.** The cells are re-picked every year. A town's count
  of town people moves 6 per cent a year up or down, because it follows
  December's count of works' hands, and an average town has four cells, so
  the number steps up and down. Of the cells that are town ground in a year a
  tenth are not ten years on, and some sixty cells a run have more than one
  spell as town.
- **A town cell is dear and the works stay off it.** The whole town's rent is
  paid on a handful of cells, so a town cell costs a works ten times the farm
  beside it. With the life cycle for works, 98 per cent of hands work within
  two cells of their market and an eighth of them on town ground.

## What kept ground does

Each town has a list of its cells in the order they joined, the centre first
(`State::town_ground`), and each cell remembers the year it joined
(`State::town_built`). Once a year, in place of `land::update_urban`
(`towns::update_urban`):

**Room.** A cell has room for 300 people the year it joins (`d_new`) and
fills in with age toward a ceiling: 600 in 1750 (`d_old`), 3 more each year
after (`d_old_growth`), reached as 1 - exp(-age / 30) (`tau`). So ground of
1700 holds about 540 people in 1750 and 1,050 in 1900, ground of 1880 holds
about 660 in 1900, and the centre is the oldest ground and the fullest.
`towns::room` is that sum.

**Growth.** When the town's people outgrow the room of its cells it takes the
best free cell next to its own ground, inside its market area, and goes on
until they fit. Best is the port's score: the cheapest to reach from the
centre, a little for being by the water, a little at random. Since a road
makes a cell cheap to reach, towns grow in ribbons along roads and shores,
not in rings. The sea and iron and coal pits are never taken, as under the
port; a pit is grown round.

**Works on town ground.** A cell with works on it may be taken like any other,
and the works stay. The cell then has less room for people: half the usual on
a cell wholly under works, in proportion between (`works_room` 0.5,
`towns::room_beside_works`). Nothing is forbidden to stand anywhere. The walk
to work (`works.walk`) already draws light trades to the town, and what a
works pays for a yard on town ground is what houses would pay for it
(`State::town_ground_rent`, the rent of the whole cell under houses; the port
prices it at what the cell's residents happen to pay, so a cell half under
works is half price). A cell with works on it that no town has taken counts as
industry on the map when the works hold a quarter of it, as before; that is
also the size at which a works that closes leaves its ground derelict
(`works::leaves_derelict`), so the two stay in step. Town ground does not go
derelict.

**Decline.** A town gives up its newest cell, one a year, only after fifteen
years in a row (`decline_years`) with under three fifths (`empty`) of the most
town people it has had (`State::town_peak`). The first version measured the
decline against the town's room, and a town that merely stood still was found
wanting as its old ground went on filling in: on eight seeds seven cells a run
left and came back. Against the peak it is two.

**Old ground in 1750.** While the first towns settle, before the first
recorded year, they fit their people exactly (ground is let go as well as
taken), and their cells are dated from the centre out: the centre a hundred
years before the start, the newest cell sixty (`back_years`, `back_span`). So
1750 opens on towns about the size the port gives them, on old ground. A town
founded later starts on new ground at 300 a cell.

**A cell that changes market** goes with its new town and keeps its year. One
left with no market at all is given up.

**The town's people** are the port's count (the town's people times a share
that rises with the works' hands), smoothed: half last year's, half this
year's (`smooth`). `State::urban_pop` is the smoothed count, so the share of
people living in towns in the series is too. Residents are shared among the
cells in proportion to room.

**Rent.** The port's rule: each resident pays `urban_rent_rate` of the wage,
less the further the cell is from the centre, falling to nothing one step
past the edge. The households' bill is the sum over the town's cells, paid as
it always was. A family that owns a cell the town grows onto is paid that
cell's share (`families::take_urban_rents`), as before.

Three smaller things ride with it, each with its own switch.

- **The year's mean of works' hands** (`mean_hands`). The count of town
  people follows the mean of the year's twelve months, not December's
  (`towns::month`, `towns::year_hands`).
- **A new town's first year** (`donor_wage`). A town founded at the end of a
  year has no year behind it: its mean wage is nothing and its mean prices
  are the first year's list. `land::sector_bids` then prices the land round
  it for a year as if labour were free (rents of 1e9 and more), and entry
  reckons its works with unpaid hands. It is given its donor's year instead,
  the town it was carved out of: wage and prices both, since the donor's wage
  against the 1750 price list would be wrong the other way
  (`towns::new_town_year`).
- **Ground is never priced under open land** (`rent_floor`). A town cell pays
  at least what its ground would earn shared out among the land uses
  (`towns::open_rent`, the rent of the farm beside it). And outside the towns,
  a cell the works have taken three fifths of is given to no other use, so
  nothing was left there to set a price and the rest of its ground came at a
  hundredth; it now costs what it would earn as open land
  (`towns::price_ground`). [works.md](works.md) asked for this.

Nothing here moves money: no payment is added and none is taken away. Switched
off, every function in `towns.rs` returns at once and the state's tables stay
empty; `tests/golden.rs` holds the port to the byte.

## For the viewer

A run with kept ground records three more layers, one byte per cell and year
like the other nine (`record::TOWN_LAYERS`). A port run's `run.bin` has none
of them and is unchanged.

| layer | byte |
|---|---|
| `built` | the year the cell joined its town less 1645 (`record::BUILT_YEAR0`), held to 1 to 255; 0 where the cell is not town ground |
| `res` | the cell's residents, in units of 8 people (`record::RES_UNIT`), 255 at most |
| `works` | the share of the cell under works, 0 to 255, on every cell, town ground included |

In `run.bin` they follow `mix_s`, shaped `[n_years, N]`. In the live view they
are `LiveLayers::built`, `res` and `works`, `None` unless the game keeps town
ground, and `LiveStatic::layers` then names twelve layers. `use` is still 10
on town ground and `inten` there is still residents over the year's ceiling.
The mix layers still show a town cell as town alone, so the works on it are
only in `works`. The viewer was not changed and does not read them yet.

## Before and after

The port with `works.enabled=true`, against the same with
`towns.enabled=true`. Means over seeds 1 to 8, the least and greatest seed in
brackets where it matters. The sweeps are `sweep-towns-works-base`,
`sweep-towns-works`, and the same two ending `-f6`, under
`.claude/night/reports/`; the rows the sweep does not have are from
`examples/towns_census.rs` on the same seeds.

| | works | works and towns | works, 6 families | works and towns, 6 families |
|---|---|---|---|---|
| town cells 1750 / 1800 / 1850 / 1900 | 72 / 54 / 60 / 73 | 63 / 126 / 158 / 231 | 72 / 58 / 64 / 79 | 63 / 124 / 162 / 231 |
| 1900 over 1750 | 1.01 (0.95 to 1.14) | 3.69 (3.50 to 3.92) | 1.09 (0.99 to 1.17) | 3.69 (3.53 to 3.85) |
| of the cells that are town, not town ten years on | 0.103 | 0.005 | 0.102 | 0.004 |
| cells with more than one spell as town, from 1760 | 64 (59 to 73) | 2 (0 to 5) | 67 (64 to 71) | 2 (0 to 4) |
| cells that were town every year from 1760 | 20 | 74 | 21 | 75 |
| industry cells 1750 / 1800 / 1850 / 1900 | 2 / 11 / 63 / 126 | 4 / 6 / 36 / 83 | 2 / 18 / 69 / 132 | 4 / 12 / 44 / 94 |
| works' hands on town ground, same years | 0.71 / 0.54 / 0.21 / 0.11 | 0.52 / 0.67 / 0.57 / 0.50 | 0.71 / 0.51 / 0.19 / 0.12 | 0.52 / 0.63 / 0.54 / 0.45 |
| works' hands within two of their market, 1900 | 0.98 | 0.97 | 0.98 | 0.97 |
| the rent bill over town people's wages, same years | 0.092 / 0.100 / 0.106 / 0.104 | 0.089 / 0.088 / 0.095 / 0.098 | 0.092 / 0.098 / 0.106 / 0.104 | 0.089 / 0.087 / 0.098 / 0.096 |
| median town cell's rent a month, 1750 / 1900 | 203 / 1,927 | 246 / 685 | 204 / 1,827 | 247 / 654 |
| median rent of the open cells beside, 1750 / 1900 | 33 / 185 | 32 / 171 | 31 / 142 | 30 / 138 |
| town cells paying less than the open ground beside, 1900 | none | 2 per cent | none | 2 per cent |
| residents of a town's fullest and emptiest cell, 1750 | 723, 293 | 524, 400 | 721, 292 | 523, 399 |
| the same in 1900 | 2,712, 1,651 | 870, 414 | 2,699, 1,629 | 889, 420 |
| farm land in 1900, cells | 1,431 | 1,378 | 1,399 | 1,387 |
| real wage over 1750, in 1800 / 1850 / 1900 | 1.42 / 1.48 / 1.48 | 1.41 / 1.44 / 1.51 | 1.50 / 1.35 / 1.57 | 1.45 / 1.47 / 1.59 |
| output per head over 1750, same years | 1.56 / 1.61 / 1.57 | 1.49 / 1.56 / 1.56 | 1.63 / 1.59 / 1.63 | 1.58 / 1.64 / 1.68 |
| share of people living in towns, 1750 / 1900 | 0.249 / 0.248 | 0.234 / 0.244 | 0.249 / 0.259 | 0.234 / 0.252 |
| works in 1900 | 604 | 602 | 668 | 619 |
| towns in 1900 / last town founded | 17.6 / 1822 | 18.6 / 1841 | 17.5 / 1831 | 18.0 / 1828 |

What is clear of the spread between seeds: the town cells from 1800 on, the
churn, and the industry cells in 1850 and 1900 (fewer on every seed, because
the towns grow over the works beside them). Half the works' hands now work on
town ground where an eighth did.

What is not: wages and output. The real wage moves by a few hundredths either
way. Output per head without families reads 3 to 5 per cent lower in 1800 and
1850, on seven seeds of eight in 1800, and the same in 1900; with six families
it is lower in 1800 and higher after. On seeds 9 to 16, which nobody had run
(`sweep-towns-works-base-s9`, `sweep-towns-works-s9`), the town cells are 62 /
119 / 156 / 228 against 72 / 57 / 63 / 76, the churn 0.003 against 0.100, the
industry cells 82 against 127 in 1900, and output per head again 2 to 5 per
cent lower (five seeds of eight in 1800, four in 1850) with the real wage in
1900 at 1.48 against 1.55 (two seeds up, six down). So over sixteen seeds
without families output per head is lower in eleven in 1800. No star, but it
leans one way, and there is a likely reason: town ground is taken whole from
the plough, and it is the best-placed land on the island. Farm land in 1900
is 4 per cent less on seeds 1 to 8 and 7 per cent less on seeds 9 to 16 (see
the worries).

The share of people in towns is a point and a half lower in 1750 on every
seed. That is the count, not the people: the year's mean of hands is under
December's while the works are growing, and the smoothing trails a growing
town by a year.

The player's town (seed 7, six families, Melbourne):

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| works: town cells | 16 | 9 | 9 | 8 |
| works: people (in town) | 26,560 (7,528) | 30,885 (9,795) | 42,678 (15,670) | 59,880 (19,805) |
| works and towns: town cells | 14 | 14 | 14 | 21 |
| works and towns: people (in town) | 26,541 (7,450) | 25,130 (6,544) | 40,714 (10,535) | 67,492 (17,691) |

It no longer shrinks. It does not grow for a century either: its town people
are fewer in 1800 than in 1750 (seven towns are founded round it in the 1750s
and take their share), and until about 1850 the old ground filling in holds
those it gains. The island's 3.7 times is mostly new towns; a first town's
ground grows by a half to three times, and three of the eight home towns
above end on the cells they started with or fewer.

### Alone on the port

`--set towns.enabled=true` without the life cycle for works
(`sweep-towns-base`, `sweep-towns`, and `-f6`): town cells 63 / 132 / 149 /
222 against 69 / 55 / 58 / 72, churn 0.011 against 0.110, wages and output
inside the spread. The port's works take a quarter of a cell each whatever
their trade, so the map is still under works: industry cells 959 against 886
in 1900 (and 227 against 204 in 1750, see the first worry). The picture needs
both mechanisms.

### What each small thing does

All eight combinations of `mean_hands`, `donor_wage` and `rent_floor`, eight
seeds each (`sweep-towns-parts`): town cells in 1900 between 220 and 237, the
real wage in 1900 between 1.34 and 1.51, output per head in 1800 between 1.47
and 1.53, the churn between 0.003 and 0.005. None of the three moves the
scorecard beyond the spread between seeds, with one exception: with a new
town reckoned at its donor's wage the last town is founded later (1831 to
1841 against 1818 to 1820 in the mean). They are kept because each is the
truer sum, not because a sweep rewards them.

The floor, read from the census (eight seeds, an earlier setting of the
densities): without it the households' bill in 1900 is 0.093 of the town
people's wages and with it 0.100, which is the port's level; 6 per cent of
town cells pay less than the open ground beside them without it and 3 per
cent with. Wages and output do not tell the two apart.

### The densities

Three settings were run before the choice (census, eight seeds, works on):

| `d_new`, `d_old_growth`, `tau` | town cells 1750 / 1800 / 1850 / 1900 | 1900 over 1750 | hands on town ground 1900 | Melbourne, six families |
|---|---|---|---|---|
| 300, 2, 40 | 65 / 138 / 177 / 281 | 4.3 (4.0 to 4.8) | 0.60 | |
| 300, 3, 30 (chosen) | 63 / 126 / 158 / 231 | 3.7 (3.5 to 3.9) | 0.50 | 14 / 14 / 14 / 21 |
| 400, 1, 30 (six families) | 62 / 125 / 190 / 297 | 4.8 (4.7 to 5.1) | 0.58 | 13 / 13 / 20 / 29 |

The brief was three to five times. The middle one was chosen with company in
mind: an Engel curve that raises the share of people in towns from a quarter
to two fifths (`dx-towns.md`, and another helper's mechanism) puts more than
half again as many people on town ground by 1900, and this setting leaves
room for that inside five or six times. The third gives the home town the
better century (it grows from 1800) and would be the one to take if the towns
stayed at a quarter of the people.

## Beside the other mechanisms

- **The life cycle for works.** Kept ground reads `State::cell_K_share` (the
  works' share of each cell, orders included) for the room beside works, and
  writes `State::land_rent_only` on town cells and on cells the works have
  taken, which `firms::evaluate_sites` and `works::ordered` bid from. A
  works on town ground pays rent on its yard to the cell's owner like any
  other; the residents' rent is for the rest of the cell.
- **An Engel curve** raises the count of town people, and the towns take
  ground for them. Nothing here needs changing for it, but the densities were
  tuned without it: after the merge, one sweep of `towns.d_old_growth` (2 to
  4) and `towns.d_new` (300 to 400) with it on, reading `cells_urban_1900`
  over `cells_urban_1750` and the home town's cells.
- **New towns.** With `donor_wage` a new town's first year is reckoned at real
  wages. Anything else that reads `State::wbar` or `State::pbar` for a town in
  its founding year (the families' sums do) now sees its donor's numbers
  where it saw a wage of nothing.
- **Railways and roads** change what is cheap to reach, and under the port
  that re-sorted the town's cells every year (a third of the churn). Kept
  ground only reads the path costs when it takes a new cell, so a railway
  bends where the town grows next and moves nothing that is built.
- **Families.** Town ground is not farmland (`families::is_farmland`), so a
  family cannot buy it, and there is three times as much of it. Land a family
  already owns when the town reaches it stays theirs and pays them the town's
  rent for it. The scripted players' results are below.

### The scripted players

`examples/bots.rs` under the game's rules, which on this branch are the port
with the life cycle for works and kept town ground. Net worth in 1900 and the
player's rank among the six families in 1775 / 1800 / 1850 / 1900, against
the same table in [works.md](works.md) (works alone):

| seed | strategy | works: net worth | rank | works and towns: net worth | rank | richest rival |
|---|---|---|---|---|---|---|
| 1 | passive | 27.0m | 2/3/3/2 | 19.0m | 2/2/3/3 | 39.5m |
| 1 | landlord | 50.0m | 2/3/1/1 | 36.2m | 2/2/2/1 | 35.0m |
| 1 | founder | 20.1m | 2/3/3/3 | 20.7m | 2/2/3/3 | 30.2m |
| 1 | thoughtful | 29.3m | 2/2/1/2 | 45.5m | 2/2/1/1 | 38.1m |
| 2 | passive | 22.2m | 1/1/1/1 | 22.5m | 1/1/1/1 | 16.3m |
| 2 | landlord | 35.4m | 1/1/1/1 | 35.5m | 1/1/1/1 | 18.7m |
| 2 | founder | 24.5m | 1/1/1/1 | 26.4m | 1/1/1/1 | 19.3m |
| 2 | thoughtful | 27.2m | 1/1/1/1 | 29.6m | 1/1/1/1 | 17.8m |
| 3 | passive | 19.4m | 3/1/1/1 | 21.2m | 2/2/2/2 | 21.9m |
| 3 | landlord | 30.5m | 2/1/1/1 | 32.7m | 2/1/1/1 | 19.8m |
| 3 | founder | 17.7m | 3/1/2/1 | 17.9m | 2/2/2/2 | 19.7m |
| 3 | thoughtful | 28.2m | 3/1/1/1 | 29.7m | 2/1/1/1 | 21.9m |
| 7 | passive | 24.7m | 4/4/1/1 | 15.0m | 4/4/4/3 | 25.3m |
| 7 | landlord | 28.0m | 4/3/2/1 | 26.2m | 4/4/1/1 | 22.1m |
| 7 | founder | 20.7m | 4/4/4/1 | 16.2m | 4/4/3/4 | 23.5m |
| 7 | thoughtful | 33.2m | 4/4/1/1 | 22.2m | 4/4/3/1 | 21.9m |

The works column is that page's. It is like for like: this build with the
towns switched off gives the same eight numbers for seeds 1 and 7, to the
last digit printed. The mean over the sixteen games is 26.0 million against
27.4, the landlord is still first in every seed and buying land is still the
best the player can do, and nothing is out of range. On seeds 2 and 3 the
player ends a little richer in all eight games. On seed 1 it is poorer in two
and richer in one by a third or more each way.

On seed 7, the island a new game opens on, the player ends poorer in all four
(the passive player 15 million against 25, third and not first). The reason
on that island is its neighbours, not its ground. The player's family lives
on its rents until 1770, and to 1775 it is the richer with kept ground (1.77
million against 1.43). But by 1795 the island has sixteen towns where it had
thirteen, Melbourne has 25,000 people in 1800 where it had 31,000, and the
family's rents and works are worth less in the smaller market (66 thousand a
year in rents against 95, works worth 250 thousand against 490). Across eight
seeds the kept ground founds half a town more by 1800, inside the spread, so
this is one island's path and not a rule. It is still the island that matters
most: see the worries.

## What worries me

- **Town ground is taken whole from the plough.** A cell at the edge with 300
  people on its four square kilometres grows nothing (`land::update_land_use`
  treats every town cell as taken, as the port does for its few). With three
  times the town cells that is 4 to 7 per cent of the farm land by 1900, the
  best-placed of it, and it is the likely reason output per head leans 2 to 5
  per cent lower without families. The mend would be to let a town cell keep
  the share of its ground the houses do not need (residents over the
  ceiling, say) under its old uses. It touches the shared land rule and the
  map's mix layers, so it was left for a session of its own.
- **A first town's ground is still for a century.** See Melbourne above. It is
  what the numbers say (the town has fewer town people in 1800 than in 1750),
  but a player watches that town more than any other. A slower ceiling
  (`d_old_growth` 1) starts it growing fifty years sooner at the cost of a
  larger total.
- **The home island founds three more towns, and the player is poorer for
  it** (the scripted players, above). Where a town is founded depends on
  where work stands far from a market, and anything that moves the works
  moves that. Sixteen seeds say the count of towns is not changed beyond the
  spread; seed 7 says a new game's first fifty years are. If the opening
  wants protecting, the place is `people::found_towns` (seven towns in the
  1750s was already the probe's complaint), not here.
- **Works take the middle of the town.** Rent on town ground is small beside
  the wage bill, as it is everywhere (3 to 5 per cent), and the walk to work
  is cheapest at the centre, so the cells round the centre fill with works:
  in 1750 several towns already have a cell a third to a half under works
  within a step of the market. A cell wholly under works still houses half
  its people (`works_room`), the probe's number. Swept on eight seeds
  (`sweep-towns-works-room`): with no room beside works the towns stand on
  256 cells in 1900, with half 231, with all of it 209; wages and output do
  not tell them apart. If the picture wants the works at the edge of the old
  town and not in its market place, the lever is the price of central ground,
  not a rule.
- **Under the port's works, and in 1750, the floor tells the first works
  where to stand.** On the first day prices are the list and wages a guess,
  open land bids more than houses, and the floor makes every town cell cost a
  works what the farm beside it does. Without the floor town cells were the
  cheaper that day and the first works crowded onto them: 71 per cent of
  hands on town ground in 1750 against 52 with it, and under the port's
  quarter-cell works 227 industry cells against 204. By 1800 it has washed
  out. It is an accident of the first day's prices either way.
- **Numbers not swept**: `smooth`, `empty`, `decline_years`, `back_years` and
  `back_span` are the probe's values or mine, checked only as a set.
- **Residents are recorded to the nearest eight**, and the byte stops at
  2,040. The fullest cell on eight seeds held 889 in the mean; a town hemmed
  in by sea and pits with nowhere to grow could pass it, and its people then
  stand above the room of its cells (the layer does not turn them away).
- **A pit is grown round, the sea is not.** A harbour town on a spit can run
  out of neighbours. I did not see it on sixteen seeds, and did not look for
  it either.
- **The viewer has not been looked at with these rules.** It builds. Towns
  are three times the size on the map, `inten` on town ground is now against
  a ceiling of 600 to 1,050 where it was 500 to 2,600, and the three new
  layers are not read yet.

## Measuring it

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --set works.enabled=true --out DIR/base
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --set works.enabled=true --set towns.enabled=true --out DIR/towns --compare DIR/base
cargo run --release -p isostate-core --example towns_census -- --families 6 --set '{"works":{"enabled":true},"towns":{"enabled":true}}' 1 2 3 4 5 6 7 8
```

The census prints, for 1750, 1800, 1850 and 1900, the town and industry
cells, where the works' hands are, what the ground pays and who lives on it;
then the churn by decade, the cells with more than one spell as town ground,
and one town by half-century (the player's seat, or `--town NAME`). Expect 61
to 65 town cells in 1750 and 215 to 250 in 1900, a ten-year churn under 0.01,
fewer than six cells with a second spell, and about half the works' hands on
town ground.
