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
town keeps what it builds and grows outward from it: 69 cells of town in
1750 and 185 in 1900 with the life cycle for works alone, 62 and 303 beside
the rest of the game's rules, and a cell that joins a town stays.

It is merged into the game's rules; what it does there, and the one number
that moved at the merge, are under "In company, after the merge".

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
fills in with age toward a ceiling: 530 in 1750 (`d_old`), 6 more each year
after (`d_old_growth`; 5 until the merge), reached as 1 - exp(-age / 30)
(`tau`). So ground of 1700 holds about 490 people in 1750 and 1,430 in 1900,
ground of 1880 holds about 850 in 1900, and the centre is the oldest ground
and the fullest.
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
works pays for a yard on town ground is what houses would pay for it, or the
plough if that is more (`State::town_ground_rent`, the rent of the whole cell;
the port prices it at what the cell's residents happen to pay, so a cell half
under works is half price). A cell with works on it that no town has taken counts as
industry on the map when the works hold a quarter of it, as before; that is
also the size at which a works that closes leaves its ground derelict
(`works::leaves_derelict`), so the two stay in step. Town ground does not go
derelict.

**Decline.** A town gives up its newest cell, one a year, only after fifteen
years in a row (`decline_years`) with under three fifths (`empty`) of the most
town people it has had (`State::town_peak`). The first version measured the
decline against the town's room, and a town that merely stood still was found
wanting as its old ground went on filling in: on eight seeds seven cells a run
left and came back. Against the peak it is one or two.

**Old ground in 1750.** While the first towns settle, before the first
recorded year, they fit their people exactly (ground is let go as well as
taken), and their cells are dated from the centre out: the centre a hundred
years before the start, the newest cell sixty (`back_years`, `back_span`). So
1750 opens on towns the size the port gives them (69 cells against 72 on
eight seeds; 61 against 61 beside the game's other rules), on old ground. A
town founded later starts on new ground at 300 a cell.

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
  (`towns::new_town_year`). The hook stands just before the market areas are
  worked out again, so it serves whichever rule founded the town.
- **Ground is never priced under what it would earn without the town**
  (`rent_floor`). A town cell pays at least what its ground would earn under
  the plough: shared among farm, pasture and woodland as the land rule shares
  it (`towns::plough_rent`). The pits and quarries are left out on purpose.
  The coal under a town is not what its houses displaced, and a pit's bid
  swings with the price of coal; held to the rent of open land under every
  use, as this was first built, a town on a coalfield paid half its wages in
  rent in a dear year and a hamlet founded on one twice its wages (see the
  review, below). The residents of a full cell pay the whole floor; in a town
  that has emptied it falls with the people (by people over room), so the few
  who stay do not pay the farm rent of ground nobody lives on. A works' yard
  on town ground bids against the whole of it, houses or plough, whichever
  pays more. And outside the towns, a cell the works have taken three fifths
  of is given to no other use, so nothing was left there to set a price and
  the rest of its ground came at a hundredth; it now costs what it would earn
  as open land, pits included, since there a pit is what a works displaces
  (`towns::open_rent`, `towns::price_ground`). [works.md](works.md) asked
  for this.

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

Everything in this section and the next two was measured before the merge,
with the ceiling rising 5 a year. At the 6 it rises now the same sweep gives
69 / 123 / 139 / 185 town cells where the first table has 69 / 125 / 143 /
194 (2.7 times for 2.8), and 69 / 121 / 143 / 179 with six families; the
churn is 0.005 as before and the rest was not measured again
(`core-steps/towns-alone-at-6`, `-f6`).

The port with `works.enabled=true`, against the same with
`towns.enabled=true`. Means over seeds 1 to 8, the least and greatest seed in
brackets where it matters. The sweeps are `sweep-towns-review-base` and
`sweep-towns-review`, and the same two ending `-f6`, under
`.claude/night/reports/`; the rows the sweep does not have are from
`examples/towns_census.rs` on the same seeds. (For the works columns those
rows are the first builder's; the review's own baseline sweep gives the same
numbers as his, seed for seed.)

| | works | works and towns | works, 6 families | works and towns, 6 families |
|---|---|---|---|---|
| town cells 1750 / 1800 / 1850 / 1900 | 72 / 54 / 60 / 73 | 69 / 125 / 143 / 194 | 72 / 58 / 64 / 79 | 69 / 124 / 146 / 195 |
| 1900 over 1750 | 1.01 (0.95 to 1.14) | 2.82 (2.59 to 3.03) | 1.09 (0.99 to 1.17) | 2.82 (2.63 to 3.00) |
| of the cells that are town, not town ten years on | 0.103 | 0.005 | 0.102 | 0.005 |
| cells with more than one spell as town, from 1760 | 64 (59 to 73) | 1 (0 to 3) | 67 (64 to 71) | 1 (0 to 3) |
| cells that were town every year from 1760 | 20 | 80 | 21 | 80 |
| industry cells 1750 / 1800 / 1850 / 1900 | 2 / 11 / 63 / 126 | 4 / 7 / 41 / 94 | 2 / 18 / 69 / 132 | 4 / 12 / 49 / 104 |
| works' hands on town ground, same years | 0.71 / 0.54 / 0.21 / 0.11 | 0.47 / 0.67 / 0.51 / 0.42 | 0.71 / 0.51 / 0.19 / 0.12 | 0.47 / 0.59 / 0.48 / 0.42 |
| works' hands within two of their market, 1900 | 0.98 | 0.98 | 0.98 | 0.98 |
| the rent bill over town people's wages, same years | 0.092 / 0.100 / 0.106 / 0.104 | 0.088 / 0.088 / 0.093 / 0.095 | 0.092 / 0.098 / 0.106 / 0.104 | 0.088 / 0.087 / 0.093 / 0.095 |
| median town cell's rent a month, 1750 / 1900 | 203 / 1,927 | 219 / 771 | 204 / 1,827 | 220 / 693 |
| median rent of the open cells beside, 1750 / 1900 | 33 / 185 | 32 / 177 | 31 / 142 | 30 / 151 |
| town cells paying less than the open ground beside, 1900 | none | 5 per cent | none | 3 per cent |
| residents of a town's fullest and emptiest cell, 1750 | 723, 293 | 471, 373 | 721, 292 | 470, 372 |
| the same in 1900 | 2,712, 1,651 | 999, 534 | 2,699, 1,629 | 1,004, 495 |
| farm land in 1900, cells | 1,431 | 1,410 | 1,399 | 1,376 |
| real wage over 1750, in 1800 / 1850 / 1900 | 1.42 / 1.48 / 1.48 | 1.40 / 1.42 / 1.51 | 1.50 / 1.35 / 1.57 | 1.47 / 1.50 / 1.38 |
| output per head over 1750, same years | 1.56 / 1.61 / 1.57 | 1.49 / 1.54 / 1.56 | 1.63 / 1.59 / 1.63 | 1.60 / 1.65 / 1.51 |
| share of people living in towns, 1750 / 1900 | 0.249 / 0.248 | 0.234 / 0.240 | 0.249 / 0.259 | 0.234 / 0.252 |
| works in 1900 | 604 | 614 | 668 | 624 |
| towns in 1900 / last town founded | 17.6 / 1822 | 17.8 / 1834 | 17.5 / 1831 | 17.8 / 1825 |

What is clear of the spread between seeds: the town cells from 1800 on, the
churn, and the industry cells (fewer on every seed in 1850 and 1900, because
the towns grow over the works beside them). Two fifths to a half of the
works' hands now work on town ground where an eighth did. The few town cells
that pay less than the open ground beside them are, I believe, the ones
beside pits, whose rent the floor leaves out on purpose; I did not check them
cell by cell.

Seeds 9 to 16 say the same (`sweep-towns-review-base-s9`,
`sweep-towns-review-s9`, and `-f6-s9`): town cells 70 / 126 / 151 / 189
against 72 / 57 / 63 / 76, the churn 0.003 against 0.100, industry cells 93
against 127 in 1900; with six families 70 / 125 / 157 / 194 against 72 / 55 /
63 / 74 and 107 against 135.

Wages and output want more care than one year can give. With six families on
seeds 1 to 8 the real wage of 1900 reads 1.38 against 1.57, lower on seven
islands and level on the eighth; on seeds 9 to 16 it reads 1.54 against 1.49.
A year's real wage can move by a fifth from one year to the next on any
island, so a scorecard's single year is a throw of the dice. Over twenty
years and sixteen seeds:

| mean over the years, over 1750 | 1790-1810 | 1840-60 | 1860-80 | 1880-1900 |
|---|---|---|---|---|
| real wage, works / works and towns | 1.44 / 1.41 | 1.53 / 1.49 | 1.59 / 1.55 | 1.55 / 1.54 |
| seeds of 16 in which towns read higher | 3 | 5 | 3 | 7 |
| the same with six families | 1.47 / 1.46 | 1.52 / 1.54 | 1.65 / 1.59 | 1.56 / 1.55 |
| seeds of 16 higher | 8 | 9 | 6 | 4 |
| output per head, works / works and towns | 1.54 / 1.50 | 1.66 / 1.62 | 1.70 / 1.67 | 1.64 / 1.63 |
| seeds of 16 higher | 5 | 4 | 4 | 5 |
| the same with six families | 1.60 / 1.59 | 1.68 / 1.69 | 1.75 / 1.70 | 1.66 / 1.64 |
| seeds of 16 higher | 9 | 8 | 5 | 5 |

So kept ground costs a little: 1 to 3 per cent of the real wage and of output
per head without families, on about three islands in four, and less than that
or nothing with six families. Two things could do it and I have not separated
them. Farm land is 1 to 2 per cent less in 1900 (town ground is taken whole
from the plough, and it is well-placed land). And there are about one works
in twenty fewer in three sweeps of four (624 against 668 with six families,
577 against 626 and 640 against 676 on seeds 9 to 16; on the port's own works
6,425 against 6,973, fewer on every island): ground on town cells and on
cells the works have taken is no longer nearly free, and a new town's first
year no longer founds works with unpaid hands.

The share of people in towns is a point and a half lower in 1750 on every
seed. That is the count, not the people: the year's mean of hands is under
December's while the works are growing, and the smoothing trails a growing
town by a year.

The player's town (seed 7, six families, Melbourne):

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| works: town cells | 16 | 9 | 9 | 8 |
| works: people (in town) | 26,560 (7,528) | 30,885 (9,795) | 42,678 (15,670) | 59,880 (19,805) |
| works and towns: town cells | 15 | 15 | 15 | 21 |
| works and towns: people (in town) | 26,542 (7,451) | 27,022 (6,573) | 41,674 (11,091) | 69,796 (20,657) |
| beside the game's other rules: town cells | 13 | 10 | 25 | 33 |
| beside the game's other rules: people (in town) | 23,323 (6,315) | 18,400 (3,684) | 34,384 (16,967) | 61,875 (32,541) |

With the works alone it no longer shrinks, and it does not grow for a century
either: the share of people in towns stays at a quarter under these rules,
its town people are fewer in 1800 than in 1750 (seven towns are founded round
it in the 1750s and take their share), and until about 1850 the old ground
filling in holds those it gains. The island's growth is mostly new towns, and
four of the eight home towns end on the cells they started with or fewer. The
last two rows are the game as it will be played (see the densities, below):
there the town does give up three cells by 1800, because it loses two fifths
of its town people for a generation, and then more than doubles.

### Alone on the port

`--set towns.enabled=true` without the life cycle for works
(`sweep-towns-base`, `sweep-towns-review-alone`, and `-f6`): town cells 70 /
139 / 149 / 186 against 69 / 55 / 58 / 72, churn 0.010 against 0.110, wages
and output inside the spread. The port's works take a quarter of a cell each
whatever their trade, so the map is still under works: industry cells 960
against 886 in 1900 (and 225 against 204 in 1750, see the worries). The
picture needs both mechanisms.

### What each small thing does

All eight combinations of `mean_hands`, `donor_wage` and `rent_floor`, eight
seeds each (`sweep-towns-parts`, run by the first builder before the review,
at his numbers): none of the three moved the scorecard beyond the spread
between seeds, with one exception: with a new town reckoned at its donor's
wage the last town was founded later (1831 to 1841 against 1818 to 1820 in
the mean). They are kept because each is the truer sum, not because a sweep
rewards them.

The floor as it now stands, on sixteen seeds without families
(`examples/towns_edges.rs`): it raises the residents' rent on 1 to 3 per cent
of a run's town cells and years and adds a tenth of one per cent to 1.2 per
cent to the households' bill. The dearest town-year on each island costs its households
0.12 to 0.23 of their wages (0.12 is the port's rate), and no town-year in
sixteen runs costs a quarter. With six families on eight seeds, floor on
against off: 195 town cells against 193 in 1900 and a real wage of 1.54
either way over the last twenty years (`sweep-towns-review-f6-xfloor`).

### The densities

The first builder ran three settings before choosing (census, eight seeds,
works on, a first ceiling of 600):

| `d_new`, `d_old_growth`, `tau` | town cells 1750 / 1800 / 1850 / 1900 | 1900 over 1750 | Melbourne, six families |
|---|---|---|---|
| 300, 2, 40 | 65 / 138 / 177 / 281 | 4.3 (4.0 to 4.8) | |
| 300, 3, 30 (his choice) | 63 / 126 / 158 / 231 | 3.7 (3.5 to 3.9) | 14 / 14 / 14 / 21 |
| 400, 1, 30 (six families) | 62 / 125 / 190 / 297 | 4.8 (4.7 to 5.1) | 13 / 13 / 20 / 29 |

He chose the denser middle with company in mind, and he was right to. The
game is not played with the works alone. Beside the merchants' prices, the
Engel curve, the machines and the rest (branch `night/core` as it stood,
merged into a scratch copy of this branch and thrown away after), the share
of people in towns goes from 0.21 to 0.41 and the island's town people grow
ninefold, not fivefold. Eight seeds, six families, `--rules game`, town
ground off and then on at eight settings (`sweep-towns-review-core-base-f6`,
`-core-f6`, `-core-f6-denser`, `-core-f6-530`, `-core-f6-530-5`):

| `d_new`, `d_old`, `d_old_growth` | town cells 1750 / 1800 / 1850 / 1900 | 1900 over 1750 | farm land 1900 | real wage 1900 |
|---|---|---|---|---|
| town ground off | 61 / 47 / 71 / 115 | 1.9 | 1,198 | 2.72 |
| 300, 600, 1 | 54 / 102 / 240 / 503 | 9.3 | 1,123 (starred) | 2.67 |
| 300, 600, 3 (his choice) | 54 / 95 / 190 / 377 | 7.0 | 1,160 | 2.85 |
| 400, 600, 1 | 54 / 94 / 217 / 467 | 8.7 | 1,128 (starred) | 2.53 |
| 400, 600, 3 | 54 / 88 / 174 / 353 | 6.6 | 1,155 | 2.52 |
| 300, 600, 5 | 54 / 87 / 161 / 304 | 5.6 | 1,165 | 2.61 |
| 300, 600, 7 | 54 / 85 / 139 / 254 | 4.7 | 1,170 | 2.69 |
| 300, 530, 5.5 | 61 / 94 / 167 / 305 | 5.0 | 1,169 | 2.72 |
| 300, 530, 5 (the review's choice) | 61 / 96 / 175 / 303 | 5.0 (4.5 to 5.5) | 1,163 | 2.61 |

Farm land is less with town ground on nearly every seed in every row; in the
two starred rows every seed is below every seed without. The real wage
differs from the first row by less than the islands differ among themselves
(2.2 to 3.1 without town ground), so it does not choose. The ground does. The
brief asked that 1750 look as it does now and that town ground grow three to
five times. A first ceiling of 530 puts the first towns on the 61 cells they
have in company without kept ground (and on 69 against 72 with the works
alone), where 600 gave 54 and 63. A ceiling rising 5 a year keeps the island
to five times in company, at 3 per cent of the farm land. With the works
alone that is 2.8 times, a little under the brief; the game is played in
company. Home towns in company go from 11 to 15 cells to 20 to 64 (one and a
half to four and a half times).

No run failed in company (72 runs). One test of another mechanism does, and
wants a line from whoever merges: see the next section.

(At the merge the company had moved again, and 5 a year gave five and a half
times. It is 6 now, for the same five times: "In company, after the merge".)

## Beside the other mechanisms

- **The life cycle for works.** Kept ground reads `State::cell_K_share` (the
  works' share of each cell, orders included) for the room beside works, and
  writes `State::land_rent_only` on town cells and on cells the works have
  taken, which `firms::evaluate_sites` and `works::ordered` bid from. A
  works on town ground pays rent on its yard to the cell's owner like any
  other; the residents' rent is for the rest of the cell.
- **The Engel curve, the machines and the rest of the game** put two fifths
  of the people in towns by 1900 where the port has a quarter, and the towns
  take ground for them. The densities are now set for that (the table above):
  61 cells in 1750 and 303 in 1900 on eight seeds with six families, no run
  failed, wages and output inside the spread, farm land 3 per cent less. If
  the island should carry less town, `towns.d_old_growth` is the dial: 7 a
  year gave 4.7 times at a first ceiling of 600.
- **New towns.** With `donor_wage` a new town's first year is reckoned at real
  wages. Anything else that reads `State::wbar` or `State::pbar` for a town in
  its founding year (the families' sums do) now sees its donor's numbers
  where it saw a wage of nothing. The people rules found towns by a rule of
  their own, later in the year's step; the hook in `sim::yearly` stands after
  both, just before the market areas are worked out again, and takes the
  donor from the old market area as `people::found_towns` does.
- **The people rules** read `State::urban_pop` for how crowded a town is.
  Under kept ground that is the smoothed count from the year's mean of hands,
  a little under the port's while a town grows.
- **Railways and roads** change what is cheap to reach, and under the port
  that re-sorted the town's cells every year (a third of the churn). Kept
  ground only reads the path costs when it takes a new cell, so a railway
  bends where the town grows next and moves nothing that is built.
- **Families.** Town ground is not farmland (`families::is_farmland`), so a
  family cannot buy it, and there is three times as much of it (five times in
  company). Land a family already owns when the town reaches it stays theirs
  and pays them the town's rent for it, which on a full cell is never less
  than the plough paid. The scripted players' results are below.

### Merging it

Tried against `night/core` at 3ad1e81. Four files want both sides kept and
nothing more: the end of `Params` and the two lists in `params.rs`, the end
of `State`, the fields of `Record`, and the list at the end of
`docs/rules/README.md`. In `sim::yearly` keep the people rules' block where
the port's `found_towns` is called, and keep the one line
`towns::new_town_year(...)` just before `st.prev_catch = st.catch.clone()`,
which is after both founding rules
(`.claude/night/reports/towns-review-merge-into-core-sim.diff` is that file's
difference as it built and ran).

So merged, every test passes but one that is not this mechanism's:
`money_is_conserved_and_runs_are_deterministic_with_machines_on` in
`tests/machines.rs`. It runs seed 5 to 1790 under the game's rules and holds
that every town standing at the end has machines counted. With kept ground
that island founds a town (Ruthrigg) at the close of 1790, the run's last
step; no month has passed for it, so it is still marked as having none. The
test should let off a town founded in the run's last year.

### The scripted players

`examples/bots.rs` under the game's rules, which on this branch are the port
with the life cycle for works and kept town ground. Net worth in 1900 and the
player's rank among the six families in 1775 / 1800 / 1850 / 1900, against
the same table in [works.md](works.md) (works alone):

| seed | strategy | works: net worth | rank | works and towns: net worth | rank | richest rival |
|---|---|---|---|---|---|---|
| 1 | passive | 27.0m | 2/3/3/2 | 22.3m | 2/2/2/2 | 45.9m |
| 1 | landlord | 50.0m | 2/3/1/1 | 42.5m | 2/2/2/2 | 49.1m |
| 1 | founder | 20.1m | 2/3/3/3 | 22.6m | 2/2/2/3 | 31.0m |
| 1 | thoughtful | 29.3m | 2/2/1/2 | 38.2m | 2/2/2/1 | 37.2m |
| 2 | passive | 22.2m | 1/1/1/1 | 23.1m | 1/1/1/1 | 16.8m |
| 2 | landlord | 35.4m | 1/1/1/1 | 33.7m | 1/1/1/1 | 15.5m |
| 2 | founder | 24.5m | 1/1/1/1 | 21.9m | 1/1/1/1 | 15.6m |
| 2 | thoughtful | 27.2m | 1/1/1/1 | 32.2m | 1/1/1/1 | 19.1m |
| 3 | passive | 19.4m | 3/1/1/1 | 15.9m | 4/2/4/4 | 17.7m |
| 3 | landlord | 30.5m | 2/1/1/1 | 29.7m | 3/1/1/1 | 19.7m |
| 3 | founder | 17.7m | 3/1/2/1 | 15.4m | 4/2/4/4 | 18.3m |
| 3 | thoughtful | 28.2m | 3/1/1/1 | 32.4m | 4/1/1/1 | 21.2m |
| 7 | passive | 24.7m | 4/4/1/1 | 19.1m | 4/4/3/3 | 25.9m |
| 7 | landlord | 28.0m | 4/3/2/1 | 29.2m | 3/3/1/1 | 24.3m |
| 7 | founder | 20.7m | 4/4/4/1 | 15.0m | 4/4/4/3 | 28.2m |
| 7 | thoughtful | 33.2m | 4/4/1/1 | 23.8m | 4/4/2/1 | 20.2m |

The works column is that page's; the first builder checked it is like for
like (his build with the towns switched off gave the same numbers for seeds
1 and 7). The mean over the sixteen games is 26.1 million against 27.4. The
landlord ends with the most on three seeds and the thoughtful player on the
fourth, so buying land is still the best the player can do, and nothing is
out of range. What moves is the player who does little: the passive player
and the founder end 2 to 6 million poorer on seeds 3 and 7 and fall from
first to third or fourth among the six. One island's game differs from the
next by more than that (seed 1's thoughtful player gains 9 million), so
sixteen games do not say kept ground makes the idle poorer; they say look
again after the merge, when the people rules found the towns and the
neighbours of the first fifty years are other neighbours anyway.

## The review

A second helper went through the mechanism before it was merged. What was
checked, and what changed.

- **The port is untouched.** The golden constants are as they were. With the
  switch off, seed 3 writes the same `series_monthly.csv`, `events.json` and
  `params.json` as the baseline, with and without six families, and the same
  `run.bin` byte for byte as the build the port was taken from (a5ea2ea).
- **Off means nothing happens.** Each hook returns at once or is not reached;
  the tables stay empty; the record and the live view carry nothing new. No
  random stream is drawn from, on or off.
- **Money.** Nothing here pays anybody. No run failed the monthly check in
  any sweep: 72 runs of the works and towns (seeds 1 to 16 with and without
  families, and the settings tried on the way), 16 of the towns alone, 64
  beside the game's other rules.
- **The first builder's numbers reproduce**: the baseline sweep seed for
  seed, and his tables from his own sweeps.
- **Extreme settings.** Room beside works of none and all, no smoothing and
  total smoothing, a decline rule that lets go at once, no back-dating, a
  falling ceiling and a ceiling of a hundred thousand: nothing breaks and
  nothing runs away. Towns thinner than a village (`d_new` 1, or 0) take the
  whole island; with no farm left wages fall to nothing, the land rule's
  bids and every purse with them run to 1e18, and the money check then trips
  on rounding. That is not a leak (one month traced purse by purse), and no
  setting near the game's comes near it.
- **Changed: the floor.** It was the rent of open land under every use, pits
  and quarries too. On sixteen seeds the dearest town-year cost its
  households from 0.13 to 2.2 times their wages (the households pay at most
  half their purse, so nothing broke, but they went without): towns standing
  on coal in a year of dear coal, hamlets newly founded on a coalfield, and,
  once the pits were left out, a town at a tenth of its room paying the farm
  rent of all its empty ground. Now the floor is the plough's rent and falls
  with the people in an emptied town: 0.12 to 0.23.
- **Changed: the densities**, for the game as it is played (above).
- **Changed: where the new town's year is mended**, so that the people rules'
  foundings are mended too.
- **Tests added**: the floor (a pit's bid does nothing to a town's rent, a
  farm's bid does, an emptied town pays in proportion, a works' yard pays the
  whole), a town with nowhere to grow, and the test of new ground now holds
  every cell that joins without touching its town to the reason, a pit on
  last year's map.

## In company, after the merge

Kept ground was merged into the game's rules tenth, after the people rules.
The integration's sweeps are under `.claude/night/reports/core-steps/`, all
under the game's rules with six families unless said: `towns-before` and
`towns-after` (seeds 1 to 6, the game before the merge and after),
`towns-s7-18-before` and `towns-s7-18` (twelve seeds nothing was chosen on),
`towns-off` (the merged code with `towns.enabled=false`: every fact of every
seed is what it was before), `towns-after-at-5` and `towns-s7-18-at-5` (the
merge before the one number below was changed), `towns-x-ceiling` (that
number at 5, 6, 7 and 8 on eight seeds), `towns-x-no-donor_wage`,
`-no-rent_floor` and `-no-mean_hands` (each small switch off, at 5),
`towns-alone-at-6` and `-f6` (the port with the works, as this page measures
it alone), and in `towns-probe/` the census, the edges and the scripted
players in company. 126 runs in the sweeps, none failed.

**The merge.** The four lists wanted both sides and nothing more, as "Merging
it" says. In `sim::yearly` the order is: the year's roads, then the people
rules' founding (which wants the roads), then `towns::new_town_year`, then
the market areas. The new town's year has to be mended after the founding,
since there is no town before it, and before the market areas are worked out
again, since the donor is read from last year's. Nothing else touches.

**What it does in company**, against the game before, seed for seed:

| | before, seeds 1 to 6 | after | before, seeds 7 to 18 | after |
|---|---|---|---|---|
| town cells 1750 / 1800 / 1850 / 1900 | 62 / 50 / 78 / 121 | 62 / 96 / 182 / 303 | 62 / 50 / 77 / 122 | 62 / 97 / 168 / 299 |
| of the cells that are town, not town ten years on | 0.083 | 0.004 | 0.087 | 0.003 |
| industry cells 1800 / 1850 / 1900 | 16 / 81 / 225 | 12 / 50 / 164 | 17 / 77 / 220 | 14 / 56 / 169 |
| people, thousands, 1800 / 1850 / 1900 | 203 / 372 / 686 | 203 / 373 / 694 | 201 / 370 / 685 | 201 / 371 / 689 |
| share of people living in towns, 1750 / 1850 / 1900 | 0.216 / 0.352 / 0.414 | 0.211 / 0.335 / 0.408 | 0.216 / 0.340 / 0.422 | 0.212 / 0.325 / 0.407 |
| towns 1800 / 1850 / 1900 | 12.7 / 16.3 / 16.5 | 13.3 / 18.3 / 18.7 | 12.8 / 17.2 / 17.6 | 12.6 / 16.0 / 16.7 |
| real wage over 1750, in 1800 / 1850 / 1900 | 1.60 / 2.04 / 2.77 | 1.61 / 1.98 / 2.54 | 1.63 / 1.98 / 2.67 | 1.63 / 1.99 / 2.58 |
| output per head over 1750, same years | 1.68 / 2.22 / 2.79 | 1.67 / 2.20 / 2.59 | 1.70 / 2.18 / 2.75 | 1.70 / 2.14 / 2.65 |
| works 1800 / 1850 / 1900 | 242 / 521 / 687 | 236 / 522 / 699 | 257 / 514 / 711 | 246 / 454 / 652 |
| median works' size, 1900 | 5.62 | 5.71 | 5.49 | 5.89 |
| first engine works / first rolling mill | 1802 / 1799 | 1811 / 1811 | 1803 / 1812 | 1810 / 1818 |
| railway km 1850 / 1870 / 1900 | 1,520 / 2,192 / 2,644 | 1,444 / 2,160 / 2,615 | 1,119 / 2,038 / 2,613 | 1,175 / 2,029 / 2,606 |
| farm land in 1900, cells | 1,182 | 1,159 | 1,199 | 1,163 |
| price swings: grain / iron / rails / cloth | 0.108 / 0.171 / 0.185 / 0.036 | 0.109 / 0.172 / 0.172 / 0.037 | 0.114 / 0.172 / 0.171 / 0.038 | 0.115 / 0.177 / 0.175 / 0.039 |
| famine years a run | 1.3 | 1.5 | 1.2 | 1.6 |
| seconds a run (the machine was busy) | 18.6 | 19.6 | 27.1 | 18.2 |

The town ground does what this page says it does: five times the cells by
1900 (4.0 to 5.5 by seed on the eighteen), the churn gone, a quarter fewer
industry cells because the towns grow over the works beside them. The census in
company (`towns-probe/census-game.txt`, eight seeds): 62 / 97 / 177 / 305
cells, 4.96 times (4.5 to 5.2 by seed), two cells a run with a second spell,
0.46 / 0.55 / 0.50 / 0.28 of the works' hands on town ground where the game
before had 0.75 / 0.48 / 0.17 / 0.05, and the rent bill at 0.09 of the town
people's wages in every half-century. Melbourne, the player's town on seed 7,
goes from 13 cells to 10 in 1800, 21 in 1850 and 29 in 1900; before the merge
it went 13, 4, 7, 9.

Wages and output, read the way this page asks, as twenty-year means over the
eighteen seeds (real wage over 1750, after against before, and the seeds in
which after is the higher):

| | 1790-1810 | 1840-60 | 1860-80 | 1880-1900 |
|---|---|---|---|---|
| real wage | 1.62 / 1.62 (9) | 2.03 / 2.05 (8) | 2.51 / 2.59 (6) | 2.63 / 2.69 (5) |
| output per head | 1.67 / 1.68 (9) | 2.17 / 2.20 (7) | 2.62 / 2.69 (6) | 2.69 / 2.75 (5) |

So 1 per cent or less to 1860 and 2 to 3 per cent after, which is what kept
ground cost with the works alone. The single years of the scorecard say more
than that on seeds 1 to 6 (2.54 for 2.77 in 1900) and less on the other
twelve; the means are the ones to believe. Population, railways, prices and
the number of famine years are inside the spread between islands. The count
of towns goes up on seeds 1 to 6 (one island that had ten towns now has
twenty-one, like most) and down on seeds 7 to 18; it is not a drift. On
seeds 7 to 18 there are a twelfth fewer works in 1900, on eleven islands of
twelve, and the median works is larger on all twelve: the one works in
twenty of "What worries me", a little more of it.

**The ceiling rises 6 a year, not 5.** The review set 5 so that the island's
town ground grew five times in company, the top of what was asked for. The
people rules' numbers were set for the game after that (their page, "In
company"), and at the merge 5 gave 342 cells in 1900 for 62, five and a half
times, and up to six on one island. `towns-x-ceiling`, eight seeds:

| `d_old_growth` | town cells 1800 / 1850 / 1900 | 1900 over 1750 | farm land 1900 | real wage 1900 |
|---|---|---|---|---|
| 5 | 102 / 189 / 342 | 5.5 | 1,163 | 2.67 |
| 6 (now) | 97 / 177 / 305 | 5.0 | 1,159 | 2.59 |
| 7 | 96 / 172 / 282 | 4.6 | 1,175 | 2.50 |
| 8 | 92 / 153 / 260 | 4.2 | 1,173 | 2.64 |

Wages and farm land do not choose, as before; the ground does, and 6 is the
review's five times again. With the works alone it is 2.7 times where 5 gave
2.8.

**The first engine works waited for a new town.** This is what the merge
found, and it is the reason the works' new trades come later in the table.
Before kept ground, a town founded under the people rules had a year with a
mean wage of nothing, as under the port, and `firms::entry` reckoned works
there with unpaid hands. On seeds 7 to 12 the island's first engine works was
founded in a town's founding year on five islands of six (Saldale 1784,
Goldstrand 1791, Cragwater 1797, Ryefield 1808, Westfell 1802), and its
first rolling mill on five; a hundred and more works a run were founded in a
town's first two years. With the donor's wage (`donor_wage`) the first engine
works stands where it pays, in a town that already stood, in 1788, 1789,
1817, 1826, 1823 and 1840 (these six runs were made with the ceiling at 5 a
year), and twenty to sixty works a run are founded
in a town's first two years. On eighteen seeds the first engine works makes
machinery seven to nine years later and the first rolling mill five to
twelve, and with `donor_wage` switched off they come as early as they did
(`towns-x-no-donor_wage`). The discoveries are no later. So the game's
industry was being started by an accident, and now it is not; that an engine
works does not pay for thirty years after it is known is a question for the
machines and the works, not for the towns.

**The chronicle's test** of a decade of new towns (`tests/chronicle.rs`) held
that the digest is kept unless the decade's four are all heavier. On seed 7
the digest of the 1750s is now outweighed by three items, and the fourth
place is the player's, which `chronicle::rank` keeps for the family's own
heaviest item whatever else happened; here that item is the lighter. The
test now leaves the player's place out of "all heavier", which is the rule
as the chronicle's page gives it. No other test moved, and the machines test
that "Merging it" warns of passes: seed 5 founds no town in 1790 any more.
It will fail again on a history that does.

**The edges in company** (`towns-probe/edges-game.txt`, eighteen seeds; the
example takes `--rules game` now): no town above its room, the fullest cell
1,426 people, the floor binding on 1 per cent of cell-years or fewer. The
dearest town-year is 0.12 to 0.29 of the town people's wages on seventeen
islands and 0.98 on one: Fenmere on seed 4, in 1838, the year it was
founded. See the first worry below.

**The scripted players** (`towns-probe/bots-game.txt`, against
`people-probe/bots-game.txt`, the game before the merge; seeds 1, 2, 3 and 7,
four strategies each). The mean net worth in 1900 over the sixteen games is
45.8 million for 46.7. The landlord and the thoughtful player end first of
the six families on all four islands, as they did. The two who do little
fare worse: the passive player ends fourth on three islands of four where it
was two, and the founder fourth on three where it was one (9.0 million for
28.3 on seed 2, 19.7 for 25.2 on seed 3, 14.7 for 25.2 on seed 7). That is
the direction the first sixteen games pointed, on other neighbours; it is
still sixteen games, and still not shown to be the towns' doing.

## What worries me

- **A new town's first year is reckoned at its donor's prices whole.** The
  donor's wage is right for it; the donor's prices are what things fetch at
  the donor's market, and the new town was founded because the carts there
  cost its district a good part of what it earns. So for one year the land
  round a new town is priced as if it lay beside the old market. Fenmere
  (seed 4, 1838) shows it: the plough's rent on its two cells is 2,200 a
  month in its founding year and 360 the year after, and through the floor
  its 445 town people are billed 0.98 of their wages for that year (they pay
  no more than half their purse). The merchants' prices already mend the same
  thing for what a new town expects, by taking the carriage from the site to
  the donor off (`prices::found_town`); `towns::new_town_year` should
  probably do the same with the year's mean prices. It was left because two
  town-years in 38,000 cost over a quarter of the wages and this is the only
  one over three tenths, and because for a works' inputs the truer price is
  the donor's plus the carriage, not less: it wants a sweep of its own.
- **Town ground is taken whole from the plough.** A cell at the edge with 300
  people on its four square kilometres grows nothing (`land::update_land_use`
  treats every town cell as taken, as the port does for its few). That is 1
  to 2 per cent of the farm land by 1900 with the works alone and 3 per cent
  in company, well-placed land, and it is one of the two likely reasons the
  real wage and output per head read 1 to 3 per cent lower (above). The mend
  would be to let a town cell keep the share of its ground the houses do not
  need under its old uses, and to charge the floor on the built share only.
  It touches the shared land rule and the map's mix layers, so it was left
  for a session of its own.
- **A town that loses its people keeps its ground for fifteen years, and a
  first town can stand still for a century.** Both are the rule doing what it
  was asked. In company Melbourne gives up three cells by 1800 and then
  grows to 33; with the works alone it stands on fifteen until 1850. A player
  watches that town more than any other.
- **The scripted players who do little end poorer on two islands of four**
  (above). Not shown to be the towns' doing. Run again after the merge, they
  end fourth of six on three islands of four ("In company, after the
  merge"). Town ground cannot be bought and there is five times as much of
  it round a seat; whether that is what does it has not been traced.
- **A single year's real wage misleads.** `real_wage_1900_vs_1750` read 0.2
  lower on every island of eight with six families and 0.05 higher on the
  next eight. Read twenty-year means from `annual.csv` before believing a
  scorecard's row about wages, here and for any other mechanism.
- **Works take the middle of the town.** Rent on town ground is small beside
  the wage bill, as it is everywhere (3 to 5 per cent), and the walk to work
  is cheapest at the centre, so the cells round the centre fill with works.
  On sixteen seeds about a quarter of the town cells of 1900 are a quarter
  under works and 3 to 11 of some 190 are nine tenths under. A cell wholly
  under works still houses half its people (`works_room`), the probe's
  number. Swept on eight seeds by the first builder
  (`sweep-towns-works-room`, his densities): with no room beside works the
  towns stood on 256 cells in 1900, with half 231, with all of it 209; wages
  and output did not tell them apart. If the picture wants the works at the
  edge of the old town and not in its market place, the lever is the price
  of central ground, not a rule.
- **In 1750 the floor tells the first works where to stand.** On the first
  day prices are the list and wages a guess, open land bids more than
  houses, and the floor makes every town cell cost a works what the farm
  beside it does. Without the floor town cells were the cheaper that day and
  the first works crowded onto them: 71 per cent of hands on town ground in
  1750 against 47 with it, and under the port's quarter-cell works 225
  industry cells against 204. By 1800 it has washed out. It is an accident
  of the first day's prices either way.
- **One works in twenty fewer** in most sweeps (above), every island with
  the port's works and six families. Ground that was nearly free now has a
  price, and a new town's first year has a wage. I take both for the truer
  sum; nobody has measured which of the two it is.
- **Numbers not swept one at a time**: `smooth`, `empty`, `decline_years`,
  `back_years` and `back_span` are the probe's values or the first
  builder's. The review tried their extremes for breakage only.
- **Residents are recorded to the nearest eight**, and the byte stops at
  2,040. The fullest cell on sixteen seeds held 1,275. A town with nowhere
  to grow crowds the ground it has, its people then stand above the room of
  its cells and the byte stops at its top (there is a test).
- **A pit is grown round, the sea is not.** A harbour town on a spit can run
  out of neighbours. On sixteen seeds no town's people ever stood above its
  room, so it has not happened yet.
- **The viewer has not been looked at with these rules.** It builds. Towns
  are three to five times the size on the map, `inten` on town ground is now
  against a ceiling of 530 to 1,430 where it was 500 to 2,600, and the three
  new layers are not read yet.

## Measuring it

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --set works.enabled=true --out DIR/base
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --set works.enabled=true --set towns.enabled=true --out DIR/towns --compare DIR/base
cargo run --release -p isostate-core --example towns_census -- --families 6 --set '{"works":{"enabled":true},"towns":{"enabled":true}}' 1 2 3 4 5 6 7 8
cargo run --release -p isostate-core --example towns_edges -- 1 2 3 4 5 6 7 8
```

The census prints, for 1750, 1800, 1850 and 1900, the town and industry
cells, where the works' hands are, what the ground pays and who lives on it;
then the churn by decade, the cells with more than one spell as town ground,
and one town by half-century (the player's seat, or `--town NAME`). With the
works alone expect 67 to 72 town cells in 1750 and 170 to 200 in 1900, a
ten-year churn under 0.01, fewer than four cells with a second spell, and
two fifths of the works' hands on town ground. Under `--rules game`, about
62 and 300.

`towns_edges` prints one line a seed of what the means hide: the dearest
town-year (the rent bill over the town people's wages; expect 0.12 to 0.25
and no more than a handful of town-years over a quarter), how often the floor
binds, the most crowded town against its room (1.00: no town has yet run out
of ground), the fullest cell, and the ground given up. Run it with `--set`
after changing any of the numbers, and with `--rules game --families 6` for
the game as it is played.
