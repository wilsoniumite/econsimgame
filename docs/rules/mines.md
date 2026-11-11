# Mines

Mines and pits that take little of the surface. Not in the Python. The switch
is `mines.enabled`; the code is `mines.rs`, hooked in by marked lines (each
says "mines") in `land::update_land_use`, `land::update_urban` and
`econ::step_month`, and a few in `record.rs`, `live.rs`, `sim.rs` and
`facts.rs`.

Stella looked at the map and said: "mines still look like big blobs. perhaps
the deposit is big but the surface mine area might just be at the base of the
mountain." She was right about the simulation as well as the picture.

## What was wrong

Under the port a quarry, a clay pit, an iron mine and a colliery are land
uses like a farm. Each bids for a share of its cell's surface, the land rule
shares the cell out in proportion to bid^(1/3), and a mine's output is
Cobb-Douglas in the share it gets. Three things follow.

- **A fifth of the island is holes.** Every cell with any rock, clay, ore or
  coal under it and any price at all gives the diggings a slice. In 1750 they
  hold 880 cells' worth of land (a cell is 2 km square), in 1900 about 1,200:
  nearly as much as the farms. The map then draws 45 per cent of each slice
  as diggings and the rest as heath, a grey-brown blob round every ore field
  and over the central upland.
- **The workings are everywhere and tiny.** In 1750 there is a quarry with a
  hand or more in a thousand cells, a sixth of the island, with two men at the
  typical one and forty at the largest. By 1900 there are 3,700 workings in
  all, and the typical quarry has twelve hands.
- **Farms and woods pay for it.** The land a quarry takes is land a farm, a
  flock or a wood does not have, though a real quarry is a face cut into a
  hillside with the sheep still on top.

A real working is a pit head, an engine house, sidings and a spoil heap over
seams that run for miles underground. What is scarce is the deposit and the
right to work it, for which the landowner is paid a royalty. The acres on top
are neither here nor there.

## What it does

**A. The deposit is the fixed factor.** A working stands on one cell, its pit
head, and has the lease of the deposit within `reach` of it (two cells, so 4
km: thirteen cells at most). The lease covers every cell of deposit that no
working had before it, for as long as the working stands, and nobody else
may sink a pit head on those cells. That last rule is what real leases were
for, and it is also what keeps the workings apart: without it every rich cell
ends up with a pit head of its own, since nobody hauls underground who can
sink a shaft instead.

Each cell of deposit is worked exactly as the port works it,
`A * f * L^(1 - alpha)`, with the whole cell's deposit standing where a share
of its surface stood, and a little richness lost to the haul underground (a
cell d cells from the pit head counts for `1 / (1 + haul * d)` of what is in
it). The hands spread themselves over the lease so that the last man raises
the same everywhere, and that sums, exactly, to one working with

    Y = A * F * L^(1 - alpha),    F = (sum over its cells of (f * haul)^(1/alpha))^alpha

So the production function is the port's, cell by cell; only the fixed factor
has changed, from acres to deposit. It is one unit in the monthly loop, at
the pit head: it hires there, its output is carted from there, and a railway
to the pit head serves the whole lease.

**B. The royalty.** What is left of the sale after the merchant's spread, the
carting, the machines and the wages is the royalty, as it was the rent. It
goes through `econ::pay_cell_rents` like every other rent, to the cells
whose deposit is worked, each by what its cell gives (`mines::royalty_cells`
splits the working's line): to a family that owns the cell, or to the
landlords of the pit head's town, or to the common fund. The landowner here
is also the adventurer: in a bad month the royalty is less than nothing, and
it is the landowners who are out of pocket, as they were for a farm.

**C. Sinking.** A working has to be sunk: `sink` hands (40) for a year at the
town's wage, who raise nothing. They come out of the town's labour supply as
the carters do, and their wages go on the working's wage bill, so in its
first year a working pays its landowners a negative royalty. That is the
landowners' outlay, and it is why nobody opens a pit head for a few tons.

**D. Opening and abandoning.** Each year every site where a pit head might
stand is reckoned up: the royalty a working there would pay, in the town's
wages, from the deposit within its reach that nobody has, at the hands that
suit the year's prices. A site is a prospect when that promise, believed
over the last few years (`memory`) and held by this year's prices too, would
repay the sinking in `payback` years (4, so ten wages a month). The best
prospects are opened first; none within two reaches of another opened the
same year; and in each town only until the new workings would add
`year_room` (0.15) of what its market takes of the good in a month, though one
may always open. That is the same thought as the year's room in `works.md`.

Once sunk, a working costs nothing to keep. A slump lays its hands off month
by month, as the port's labour rule does, and leaves it standing. It is
abandoned when it has stood idle (under one hand on a year's average) for
`idle_years` (10), or when a town grows over its pit head; its cells are then
free, and a neighbour that can reach them takes them over.

**E. Depth.** Everything raised from a cell is counted against it, and the
richness the hands meet falls as they go deeper: `f / (1 + dug / (reserve *
f))`. A rich deposit has more in it and tires more slowly. With `reserve` at
twenty million a cell of middling richness halves after about ten million
units, which only the best cells of the largest collieries come near by
1900. It is set gently on purpose (see the sweeps): the near deposits are
worked first and the workings move on as they tire, or stay and get dearer,
without the century's growth being spent on it.

**F. The surface.** A working takes land by its hands, as a works does
(`works.md`, B): `hands_per_cell` (8,000) fill a cell with pit heads, yards
and spoil, and no cell gives its workings more than `cell_limit` (half). A
quarry of sixty men takes three hectares; a colliery of a thousand, fifty.
That surface is taken out of the pit head's cell before the land rule shares
the rest between farm, pasture, wood and wild, and written back into the land
shares afterwards, so a cell's shares sum to what the works have left of it
exactly as before, and everything that reads the shares (the areas in the
sweep, the map's mix, the tests) goes on reading the truth.

## What I tried first, and why not

Three designs went before this one. They are worth a paragraph because each
looks right.

- **One working a cell, the whole cell's deposit its factor.** Simple, and
  wrong in the sums: Cobb-Douglas with alpha 0.25 has sharply falling returns
  to hands on one cell, so gathering twenty-eight small iron pits into one
  takes four times the hands for the same ore. The port's thousands of tiny
  workings are efficient under its own production function. A working has to
  have a lease of many cells to be large without being wasteful, which is
  what A does.
- **A standing crew.** A fixed crew at every pit (the banksman, the pumps),
  paid monthly, with a working opened when the royalty covered it and closed
  when it did not. Honest, and unstable: the royalty goes as the third or
  fourth power of the net price, so a price a quarter lower closed every pit in a district
  in the same year, the price then went to its ceiling, and the wage
  followed. The warm-up never settled (a wage of 70 in 1750). A cost that is
  sunk once and then gone gives the same least size without the cliff.
- **Deposit to whoever pays most.** With a haul underground, a pit head on
  the cell always pays more for it than a pit head next door, so a rich
  coalfield filled with pit heads edge to edge: the blob again, in pit heads.
  The lease stops that.

## The numbers

`MineParams`, in `mines.rs`. Off by default; `game()` switches it on with
these.

| | | |
|---|---|---|
| `enabled` | false | quarries, pits and mines work a deposit and take little surface |
| `reach` | 2.0 | cells from the pit head a lease reaches (a cell is 2 km) |
| `haul` | 0.05 | richness lost per cell of haul underground: `1 / (1 + haul * d)` |
| `sink` | 40 | hands for a year to sink a working; 0 and a pit head costs nothing |
| `payback` | 4 | years in which the promised royalty must repay the sinking |
| `idle_years` | 10 | years idle before a working is abandoned |
| `memory` | 0.3 | how much of this year's promise goes into what a site is believed to pay |
| `year_room` | 0.15 | new workings a year, as a share of the town's monthly trade in the good; 0 for no limit |
| `reserve` | 20,000,000 | output, per unit of richness, at which a cell's richness has halved; 0 and a deposit never tires |
| `hands_per_cell` | 8,000 | hands that fill a cell with pit heads, yards and spoil |
| `cell_limit` | 0.5 | the most of a cell its workings' surface may take |

`--set mines.reserve=0` measures everything but depth. `reach` and `sink`
between them set how many workings there are; neither has been swept.

A run starts with the best sites of every district opened whatever they
would pay (three rounds, each taking the best sites left that stand two
reaches apart). The first prices are guesses, and the workings nobody then
needs stand idle at no cost until the ten warm-up years have found the
market's level; those still idle in 1749 are abandoned, and the record opens
on the rest.

## What the viewer is given

Only with the mechanism on. Another hand is drawing the workings properly;
this is exactly what is recorded for them.

**The display mix** (`mix_a`, `mix_b`, `mix_s`) shows the surface as it is.
A working's share of its cell is all diggings (the 45 per cent rule is the
port's), and it is small, so it is hardly ever one of a cell's two largest
classes: with the mechanism on the mix is farm, pasture, wood and heath where
the blobs were, and the workings have to be drawn from the table below. The
`use` layer likewise never says quarry or mine.

**The workings**, every year: `Record::workings[k]` and
`LiveYear::workings`, a list of `mines::Working`, and in `run.bin` the same
as flat arrays, year k's rows being `mine_first[k] .. mine_first[k + 1]`:

| field | array | |
|---|---|---|
| `cell` | `mine_cell` (int32) | the cell its pit head stands on |
| `sector` | `mine_sector` (uint8) | index into the header's `sectors`: 3 quarry, 4 clay pit, 5 iron mine, 6 colliery |
| `opened` | `mine_opened` (int32) | the year it was opened; 0 for one that lies abandoned. A run's first workings say 1739 |
| `left` | `mine_left` (int32) | the year it was abandoned, or 0 (for a working at work: the year the one before it there was given up) |
| `hands` | `mine_hands` (float32) | mean hands over the year, the sinkers among them in its first; 0 if abandoned |
| `out` | `mine_out` (float32) | what it raised over the year |
| `heap` | `mine_heap` (float32) | everything raised at this pit head so far: the size of the spoil heap |
| `surface` | `mine_surface` (float32) | the share of its cell its pit head, yards and spoil take |
| `royalty` | `mine_royalty` (float32) | what it paid in royalty over the year |
| `cells` | `mine_cells` (int32) | the cells of its lease, its own among them |

Abandoned workings stay in the list (with `opened` 0) so that an old heap can
be drawn. A working with `hands` under one is standing idle. Which cells a
lease covers is in the state (`State::mine_head`) but not in the record; say
if the picture wants it.

**The viewer** was mended only as far as not drawing nonsense: `Replay` has
`workings` (read from `run.bin` or filled from the live year), and where a
year has them `scatter.rs` puts one of the existing pits on the first free
spot of each working's cell, for workings with a hand or more. Without them
(a port run, an old recording) pits are read off the mix as before. Size,
pit heads, spoil heaps and idle or abandoned workings are not drawn.

## What the sweeps showed

Seeds 1 to 8 to 1900. Every figure is the mean over the eight; "8/0" is how
many seeds moved up and how many down, and a star means every seed now is
clear of every seed then. Sweep folders `sweep-mines*` under
`.claude/night/reports/`. No run failed in any of them.

Two comparisons, each with and without six families: the port with only this
switched on against the port (what the mechanism does alone), and the game's
rules against the game's rules without it (what it does in company). Before
and after are written `before > after`.

**Land, in cells' worth** (the port, no families; with six families the same
to within sixty cells):

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| farm | 1,404 > 1,619 | 1,327 > 1,589 | 1,335 > 1,652 | 1,282 > 1,598 |
| pasture | 1,525 > 1,829 | 1,513 > 1,905 | 1,371 > 1,745 | 1,301 > 1,644 |
| forestry | 1,211 > 1,417 | 1,107 > 1,345 | 999 > 1,285 | 853 > 1,114 |
| quarries, pits and mines | 882 > 0.9 | 1,090 > 2.9 | 1,197 > 6.2 | 1,193 > 13.0 |
| wild | 565 > 716 | 453 > 614 | 373 > 528 | 288 > 406 |

Under the game's rules, six families: farm 1,382 > 1,597 and 1,198 > 1,494
(1750 and 1900), pasture 1,540 > 1,831 and 1,711 > 2,303, forestry 1,344 >
1,562 and 817 > 1,076, mines 880 > 0.8 and 1,399 > 9.2, wild 635 > 788 and
421 > 627. Every one of these is starred. The land the mines gave up went to
farms, flocks, woods and the wild about as the rest of the island is shared.

**The workings** (cells where a sector has a hand or more), the game's rules,
six families:

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| quarries | 985 > 136 | 1,385 > 172 | 1,676 > 239 | 2,135 > 292 |
| clay pits | 117 > 83 | 584 > 83 | 980 > 99 | 1,037 > 100 |
| iron mines | 12 > 5 | 38 > 10 | 84 > 14 | 149 > 20 |
| collieries | 128 > 32 | 257 > 45 | 293 > 53 | 330 > 54 |
| mean share of its cell | 0.715 > 0.003 | 0.491 > 0.006 | 0.423 > 0.012 | 0.384 > 0.020 |
| the largest share | 0.54 > 0.03 | 0.52 > 0.08 | 0.57 > 0.16 | 0.61 > 0.27 |
| hands at the median quarry | 2 > 23 | 3 > 35 | 7 > 69 | 12 > 128 |
| hands at the largest colliery | 30 > 80 | 174 > 667 | 574 > 1,279 | 632 > 2,156 |

So a fifth of the island's land becomes nine cells' worth, the typical
working takes a hundredth of its cell, and there are an eighth as many of
them, each ten times the size. (The last two rows are from
`examples/mines_census.rs`, the rest from the sweep. Under the port alone the
counts are much the same: 147, 198, 258 and 282 quarries.) The largest
colliery of 1900 takes a quarter of its cell, a square kilometre of pit
head, sidings and spoil for two thousand men, which is on the generous side.

**Hands, as a share of everyone at work** (December):

| | | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|---|
| quarries | port | 0.066 > 0.092 | 0.119 > 0.131 | 0.157 > 0.158 | 0.159 > 0.169 |
| | game, six families | 0.065 > 0.077 | 0.082 > 0.086 | 0.112 > 0.125 | 0.132 > 0.144 |
| clay pits | port | 0.006 > 0.004 | 0.016 > 0.012 | 0.028 > 0.022 | 0.031 > 0.038 |
| | game, six families | 0.007 > 0.006 | 0.027 > 0.028 | 0.038 > 0.032 | 0.022 > 0.018 |
| iron mines | port | 0.001 > 0.001 | 0.002 > 0.001 | 0.002 > 0.001 | 0.003 > 0.002 |
| collieries | port | 0.020 > 0.008 | 0.063 > 0.056 | 0.074 > 0.084 | 0.097 > 0.093 |
| | game, six families | 0.009 > 0.006 | 0.047 > 0.043 | 0.052 > 0.049 | 0.051 > 0.053 |

The quarries' share is up in 1750 in every seed (two and a half points under
the port, one under the game's rules) and mostly inside the spread after,
though under the game's rules with families it is up a point in 1850 and
1900 (7/1). The hands hardly move, and cannot fall: see "Why quarrying takes
so many hands" below.

**Output a head and price.** Output is units a month per head; the price is
over the wage, so that runs whose money is worth different amounts can be
set side by side. The port, no families:

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| stone raised | 0.264 > 0.345 | 0.451 > 0.553 | 0.544 > 0.619 | 0.486 > 0.608 |
| stone's price | 0.240 > 0.213 | 0.249 > 0.210 | 0.246 > 0.207 | 0.268 > 0.229 |
| clay raised | 0.045 > 0.047 | 0.071 > 0.079 | 0.108 > 0.110 | 0.102 > 0.155 |
| clay's price | 0.120 > 0.083 | 0.167 > 0.125 | 0.185 > 0.134 | 0.210 > 0.171 |
| ore raised | 0.008 > 0.010 | 0.015 > 0.015 | 0.015 > 0.017 | 0.016 > 0.020 |
| ore's price | 0.393 > 0.343 | 0.185 > 0.126 | 0.150 > 0.086 | 0.187 > 0.112 |
| coal raised | 0.178 > 0.207 | 0.605 > 0.720 | 0.614 > 0.787 | 0.620 > 0.682 |
| coal's price | 0.492 > 0.401 | 0.213 > 0.173 | 0.287 > 0.209 | 0.329 > 0.231 |

The game's rules, six families:

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| stone raised | 0.243 > 0.295 | 0.363 > 0.408 | 0.770 > 1.018 | 1.550 > 1.839 |
| stone's price | 0.259 > 0.224 | 0.204 > 0.175 | 0.140 > 0.115 | 0.078 > 0.067 |
| clay's price | 0.147 > 0.097 | 0.142 > 0.111 | 0.103 > 0.077 | 0.055 > 0.042 |
| ore's price | 0.594 > 0.445 | 0.124 > 0.082 | 0.088 > 0.054 | 0.059 > 0.037 |
| coal raised | 0.093 > 0.089 | 0.575 > 0.715 | 0.874 > 1.122 | 1.215 > 1.337 |
| coal's price | 0.832 > 0.786 | 0.175 > 0.126 | 0.145 > 0.095 | 0.071 > 0.056 |

Stone is a tenth to a sixth cheaper: in every seed and every year of the
port's runs (0/8, with and without families), and in the game's in every
seed to 1800 and with families throughout, but inside the spread in 1900
without them (3/5). Clay is a sixth to a third cheaper (0/8 in most years,
starred in 1750 everywhere). Ore is a third to two fifths cheaper from 1800
on (0/8 or 1/7); in 1750 it is an eighth to a quarter cheaper in the mean
but not clear of the spread (3/5, 2/6). Coal is a fifth cheaper
in 1750 under the port (0/8) and hardly changed under the game's rules (1/7,
6 per cent); a fifth to two fifths cheaper in 1800 and 1850 (0/8 to 3/5); and
in 1900 cheaper in the mean in three comparisons of four but inside the
spread in all of them (2/6, 4/4, 2/6, 2/6).

Why cheaper: a lease gives a working all the deposit within reach, where the
port gave each cell's mine a third of the cell and made it bid against the
farm for that; and the lease is carted from the pit head, which stands where
carting is short.

**Depth** costs little at this `reserve`, and was chosen so. Under the
game's rules without families (`sweep-mines-game-nodepth`, `-depth`), against
no depth at all: at the first value I tried, six million, coal in 1900 was
half as dear again (0/8), a head raised 30 per cent less of it and had 7 per
cent fewer goods (0/8), which is a late-century drag the game does not need.
At twenty million, the game's number, a head raises 12 per cent less coal in
1900 (0/8), its price is inside the spread (3/5), and goods a head are 3 per
cent lower (1/7): felt, and small. At forty million nothing is clear of the
spread but the coal raised.

**Royalties**, as a share of everything made. With the mechanism off nobody
keeps the mines' rent, and the figure is what the units would pay at
December's hands.

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| port | 0.006 > 0.006 | 0.030 > 0.029 | 0.036 > 0.041 | 0.044 > 0.045 |
| port, six families | 0.006 > 0.006 | 0.030 > 0.031 | 0.033 > 0.039 | 0.045 > 0.042 |
| game | 0.009 > 0.024 | 0.015 > 0.020 | 0.024 > 0.030 | 0.036 > 0.038 |
| game, six families | 0.009 > 0.024 | 0.022 > 0.020 | 0.023 > 0.035 | 0.034 > 0.040 |

About what the mines' rent was: three or four per cent of output by 1900.
Under the port in 1750 it is next to nothing either way, because the port
makes every mine pay for pumps and wagonway rails on every ton whatever they
cost, and in 1750 they cost most of the sale (`machines.md` mends that, and
under the game's rules the 1750 royalty is two per cent, up from one: 8/0,
and 7/1 with families).

**Farm output and the standard of living.**

| | | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|---|
| grain against 1750 | port | | 2.39 > 2.25 | 4.52 > 4.74 | 8.93 > 8.67 |
| | game, six families | | 2.01 > 2.04 | 4.15 > 4.44 | 10.80 > 11.47 |
| real wage against 1750 | port | | 1.33 > 1.33 | 1.34 > 1.42 | 1.29 > 1.37 |
| | port, six families | | 1.31 > 1.38 | 1.35 > 1.50 | 1.35 > 1.28 |
| | game | | 1.63 > 1.64 | 1.83 > 1.74 | 2.85 > 2.79 |
| | game, six families | | 1.65 > 1.70 | 2.05 > 2.07 | 2.72 > 2.79 |
| baskets a head | port | 1.76 > 1.89 | 3.04 > 3.23 | 3.44 > 3.85 | 3.64 > 4.23 |
| | port, six families | 1.77 > 1.90 | 2.99 > 3.31 | 3.49 > 4.29 | 3.59 > 3.74 |
| | game | 1.45 > 1.53 | 2.98 > 3.26 | 4.01 > 4.28 | 8.44 > 9.12 |
| | game, six families | 1.46 > 1.54 | 3.08 > 3.36 | 5.00 > 5.78 | 8.23 > 9.02 |
| goods a head | port | 2.85 > 3.04 | 4.35 > 4.54 | 4.98 > 5.72 | 5.42 > 5.95 |
| | game, six families | 2.45 > 2.60 | 4.38 > 4.78 | 7.17 > 8.22 | 10.97 > 11.74 |

The real wage against its own 1750 is starred nowhere and goes both ways
(1900: 6/2, 3/5, 4/4, 6/2), because 1750 itself is better: the mechanism is
there from the first month of the warm-up. The measures that do not depend on
a run's own 1750 (`carting.md`) say what happened. The households' budget
over the cost of subsistence is higher in every year of every comparison:
by 5 to 7 per cent in 1750 (8/0 in all four), 6 to 11 in 1800 (8/0 in three,
5/3 under the port without families), 7 to 23 in 1850 (6/2 to 8/0) and 4 to
16 in 1900 (8/0, 7/1, 7/1, and 4/4 under the port with families). Goods a
head are up 6 per cent in 1750 (8/0 everywhere, starred under the game's
rules), and 2 to 20 per cent after. Under the game's rules with six
families, which is the game as it is played: baskets up 5, 9, 16 and 10 per
cent, goods up 6, 9, 15 and 7 (8/0 in every year but 7/1 for baskets in 1850
and 1900).

Grain is inside the spread under the port and up 6 per cent by 1900 under
the game's rules (7/1 with and without families), from a quarter more
farmland; the farms were not short of land so much as of hands.

## Where the workings stand

`examples/mines_census.rs --map SECTOR` prints the island with a sector's
workings on it, and its table gives the hands' mean height (0 the shore, 1
the highest summit) beside the height of the deposit they work and of the
whole deposit. The game's rules, six families, eight seeds:

| | 1750 | 1800 | 1850 | 1900 | all the deposit |
|---|---|---|---|---|---|
| quarrymen's height, port's rule | 0.44 | 0.47 | 0.44 | 0.51 | 0.42 |
| quarrymen's height | 0.37 | 0.41 | 0.42 | 0.48 | 0.42 |
| height of the rock they work | 0.40 | 0.43 | 0.43 | 0.49 | |
| their path cost to market | 8.1 > 5.8 | 6.9 > 5.2 | 3.2 > 2.0 | 2.0 > 1.9 | |
| colliers' height | 0.21 | 0.18 | 0.18 | 0.21 | 0.24 |
| iron miners' height | 0.44 | 0.24 | 0.21 | 0.25 | 0.33 |

So the pattern is the one asked for, in part.

- **The quarries of 1750 stand on the lower side of the rock**, below the
  average of all the rock there is and well below where the port put them,
  and nearer their markets by a third. Their pit heads stand a little below
  the rock they work (0.37 against 0.40): the foot of the hill, by the two
  cells a lease reaches.
- **They move inward and uphill over the century**, to above the average by
  1900, as the railways arrive and the near rock tires. On the map of seed 7
  in 1900 the largest quarries sit on railways in the middle of the upland.
  That is the model's rock, which is richest on the high ground and is
  everywhere (every land cell has some), so "round the foot of the upland"
  holds for 1750 and not for 1900, when the upland has been opened up.
- **The collieries stand on the low side of their coalfields throughout**
  (0.21 against 0.24), and the iron mines from 1800 on. So did the port's:
  that is the net price at work, not the leases.
- **They are spaced**, two to four cells apart in a worked field, not edge to
  edge, and on the map of seed 7 many of the large ones are on a railway or
  beside a town.

What I did not do is look at it in the viewer's top-down plan
(`ISOSTATE_PLAN`): the text map and the heights are what this rests on. The
maps I read were from a build with a smaller `reserve`; the heights above are
from the final one.

## Why quarrying takes so many hands

Quarrymen are a sixth of all hands by 1900 under the port and 13 or 14 per
cent under the game's rules, and this mechanism does not change that. It
cannot, and the reason is worth having down.

Households buy needs, and shelter is one: above subsistence 22 per cent of
each further shilling goes on it (`shares` in `data/content.toml`), and the
households' budget share for shelter under the game's rules is 13.5 per
cent in 1750 and 22.5 per cent in 1900 (`budget_shelter_Y`). Shelter is
bought as timber, stone or bricks, whichever is cheapest for the shelter in
it, and that is stone: 0.6 a unit for half a unit of shelter, against 1.8
for bricks. So a fifth of what households spend arrives at the quarry face
as the price of raw stone. Three quarters of a quarry's sale, less the
carting, is wages (alpha is 0.25). A fifth of spending times three quarters
is the sixth of all hands.

Nothing on the supply side moves it. The spending share is fixed by the
demand system and the wage share by the Cobb-Douglas, so a quarry that is
twice as good halves the price of stone, doubles the tons and employs the
same men. That is what the sweeps show: a tenth to a third more stone a
head, a tenth to a sixth cheaper, the same hands or a few more. By 1900 a
head of the game's island takes 1.8 units of stone a month. If a unit is
about a ton, as a quarryman's 25 units a month suggests, that is over twenty
tons of stone a head a year, far more than I believe any country has used.

No number in the game's rules is plainly wrong on its own. Shelter at a
fifth of the budget is fair; what is missing is everyone between the quarry
and the house: the masons, bricklayers, carpenters and slaters, and the
house itself as something that stands for a century. That is the building
trades, which are being designed, and I did not build them. If one number
had to change in the meantime it would be shelter's marginal share (0.22 to
about 0.10, the rest to comfort), which would halve the quarrymen; but it
would be a patch on the wrong side of the trade.

## What it does in company

- **`works`**: a works and a working can share a cell; the workings' surface
  is capped at what the works have left. The land rule runs before the
  year's works are founded, so for the rest of that year a cell can be a
  little over-full, as it could before.
- **`people`**: the game's population answers to income, so the better
  standard of living becomes 8 or 9 per cent more people by 1900 (641
  thousand > 696 thousand with six families, 8/0; starred without them).
- **`railways`**: 13 or 14 per cent less track in 1900 under the game's
  rules (2,522 > 2,200 km with six families, 1/7; 0/8 without), and the
  branches to quarries go from 72 to 10 (0/8, starred). A lease is carted
  from one pit head, and the pit heads stand where the carting is already
  short, so few of them are worth a line of their own. Whoever owns
  `railways.md` should look at whether that is the right answer or whether a
  large quarry now deserves a branch it does not get. Under the port's
  railways the first line comes six to nine years sooner (2/6, 2/5: not
  clear of the spread).
- **`machines`**: pumps, wagonways and quarry machinery reach a working
  through `sector_tfp` as they reached the port's mines; nothing changes.
- **`carting`**: the hands' path cost to market is lower, so there are
  slightly fewer carters in 1750 (0.139 > 0.130 of all hands, 0/8).
- **`chronicle`**: a coal boom still names the pit with most hands, which is
  now a pit head. Openings and abandonments are not news; they could be.
- **families**: a family that owns a cell is paid its share of the royalty of
  whatever working has the lease of it, and the price of land counts it
  (`cell_rent_year`). The families' own decisions are untouched. The scripted
  players (`examples/bots.rs`, seeds 1, 2, 3 and 7) play all sixteen games
  through; the landlord is the richest family in 1900 on all four seeds, as
  it was in the tables of `works.md`. I have no table of the same build
  without the mines to set beside it (the example takes no overrides), so
  whether royalties have made land a better buy than it was is not known.

**Beside kept town ground.** Everything above was measured on the build
this mechanism was written on, before `towns.md` was merged. The branch has
since taken it in (one clause was added to `towns::update_urban`: a town on
kept ground does not take the pit head of an iron mine or colliery at work,
as the port's did not), every test passes, and the sweep was run again under
the merged game's rules (`sweep-mines-merged*`, mines on against off, no run
failed). With six families: baskets a head 1.44 > 1.52 in 1750 and 2.97 >
3.20 in 1800 (8/0 both), 4.94 > 4.70 in 1850 (5/3, inside the spread) and
7.82 > 8.84 in 1900 (8/0); goods a head 2.44 > 2.59 (8/0, starred), 4.43 >
4.67 (8/0), 7.06 > 7.22 (5/3), 10.7 > 11.5 (7/1); people in 1900 690
thousand > 721 thousand (8/0); the land as before (mines 874 > 0.8 and
1,373 > 9.2 cells, farm 1,159 > 1,453 in 1900); 3,767 workings > 475 in
1900; stone, clay and ore cheaper in every year (0/8 in most); coal cheaper
from 1800 (0/8 in 1800, 1/7 in 1900); track in 1900 2,594 > 2,240 km (0/8,
starred) and lines to quarries 70 > 10; town cells 305 > 337 (7/1), with
more people to house. A run takes 17.9 seconds against 15.6 (8/0, starred):
two seconds, which is the yearly review.

Two tests of other mechanisms had to be told about the new history, and
neither was changed lightly. `tests/chronicle.rs` held that seed 7 has a
decade with three new towns for the digest to describe; with the mines as
workings seed 7 founds two a decade at most, so the test now holds that a
digest is filed for exactly the decades that have the towns for one, and on
seed 7 that is none (with kept town ground merged as well: 1761, 1769; 1770,
1777; 1793, 1795). The digest's wording and its place among a decade's four
are therefore not exercised by any test at the moment; another seed, or a
scenario that founds three towns, would bring that back. `tests/machines.rs` checked every active town's machines
at the end of 1790; seed 5 now founds a town in 1790 itself, which has not
had a month yet, and the test skips it.

## What worries me

- **Bad years.** Earlier builds of this mechanism had years in which a
  town's wage ran away: the port lets a town's labour demand exceed its
  supply for a month and doubles the wage, and a large working whose price
  has just jumped can ask for that, where the port's thousands of small ones
  could not all at once. The final build's sweeps show no freak in the mark
  years, but the worst year for the real wage is worse in one seed under the
  port (-0.38 against a worst of -0.23) and a little worse in seven seeds of
  eight under the game's rules without families (-0.153 against -0.134 in the
  mean). With families it is better in the mean and 4/4. I did not look year
  by year.
- **Depth is a guess**, set to be felt and small (above). Nothing calibrates
  it.
- **The lease is first come, first served, for ever.** A working keeps its
  cells while it has a hand at work, however badly placed it has become, and
  a railway built past the far side of its lease cannot be used by anyone
  else. Nothing lets a better-placed adventurer buy the old one out.
- **The landowners carry the sinking and the bad months.** There is no
  mining company: the royalty is the residual, as the port's rent was. A
  family that owns coal cells is therefore in the coal trade, losses
  included, without having chosen it.
- **Workings on rich rock under high ground still open there** when a
  railway reaches them. If "at the foot of the hill" is wanted for 1900 too,
  the thing to change is `reach` for quarries (so that a quarry at the foot
  can work a hillside four cells up), not the price rule.
- **The rock is everywhere.** Every land cell has some, so a quarry can open
  anywhere stone is dear enough, and there are 300 of them by 1900. That is
  the world generator's field, not this mechanism's, but it is why quarries
  do not look like a district the way collieries do.
- **A run starts with more workings than it needs** and abandons the ones
  that never found a hand in 1749. The record never sees it, but a mechanism
  that reads the workings during the warm-up would.
- **`reach`, `sink`, `haul`, `payback`, `year_room` and `hands_per_cell`
  were set by judgment and a few runs on two seeds**, not swept. The sweeps
  here are of the mechanism on and off with the numbers in the table, and of
  `reserve`.
- **Speed.** A run under the game's rules is about two seconds slower: 17.9
  against 15.6 with six families on the merged build, 17.1 against 14.9
  without (8/0 both), and 24.0 against 22.7 under the port. I did not
  profile it; the yearly review reckons every site of every sector and
  searches each cell's reach several times, and could be made cheaper.
- **Sweeps on eight seeds**, as everywhere. The areas, the counts and the
  prices of stone and clay are far outside the spread. The standard of
  living is 8/0 in 1750 everywhere and in most later years under the game's
  rules with families.

## Checking it

`cargo test -p isostate-core --release --test mines`: off is the port and
fills nothing; money is conserved for forty years and for twenty with six
families, and a run is deterministic; the mines hold under 3 per cent of the
land they held and the rest is farmed, grazed, wooded or wild, with no less
raised; every year the land shares of every cell sum as they should, each
working's surface is its row of the shares, and the monthly loop has exactly
one unit for every working; every lease is whole, within reach, and nobody
has sunk inside one; the royalty is split to the last penny among the cells
of the lease; what is raised adds up cell by cell to what stands on the
heaps, and a deposit with a small reserve is dearer; a working left idle is
abandoned and one at work is not; and the viewer's table is recorded,
packed, and absent from a port run.

`cargo run -p isostate-core --release --example mines_census -- --rules game
--families 6 [--map coalmine] [--trace] 3 7` prints the tables above for any
rules and seeds, the map, and with `--trace` a sector year by year.
