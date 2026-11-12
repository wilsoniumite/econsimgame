# Mines

Mines and pits that take little of the surface. Not in the Python. The switch
is `mines.enabled`; the code is `mines.rs`, hooked in by marked lines (each
says "mines") in `land::update_land_use`, `land::update_urban`,
`towns::update_urban` and `econ::step_month`, and a few in `record.rs`,
`live.rs`, `sim.rs` and `facts.rs`.

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
`idle_years` (10); its cells are then free, and a neighbour that can reach
them takes them over.

Nothing else ends a working that has hands. **A town goes round a pit head
at work**: under the port a town kept off a cell that was mostly iron mine
or colliery and built freely over quarries and clay pits, which were a slice
of every cell and lost a slice; a working is one place with the lease of a
dozen cells behind it, so the port's rule for mines is kept for all four
(`mines::blocks_town`). An idle working is no obstacle: the town takes the
cell and the working is given up, as is one on the cell a new town is
founded on. **Works may fill the cell beside it**: the pit head was there
first, the works came for the coal, and its surface is what room they leave
it. (The first build gave a working up when works filled six tenths of its
cell or a closed works left it derelict. See "What the review found".)

A working raises from its first month. The sinkers are the price of opening
it, not a wait: the year of sinking and the first year of raising are the
same year.

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
on the rest. What the warm-up gives up leaves no row in the table of
workings below (some five hundred a run would otherwise be listed in 1750 as
abandoned pits, more than there are at work).

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
be drawn: a hundred or so by 1900, each left in a recorded year with
something on its heap. A working with `hands` under one is standing idle. Which cells a
lease covers is in the state (`State::mine_head`) but not in the record; say
if the picture wants it.

**The viewer** was mended only as far as not drawing nonsense: `Replay` has
`workings` (read from `run.bin` or filled from the live year), and where a
year has them `scatter.rs` puts one of the existing pits on the first free
spot of each working's cell, for workings with a hand or more. Without them
(a port run, an old recording) pits are read off the mix as before. Size,
pit heads, spoil heaps and idle or abandoned workings are not drawn.

## What the review found

The mechanism was reviewed before it was merged, by another hand (branch
`night/mines-review`). What was checked, in short: with the switch off a
run writes the files it wrote before any of this (seed 3, and seed 7 with
six families, byte for byte, `params.json` too); no hook does anything or
draws a random number while it is off, and the mechanism draws none when it
is on; all 64 runs of seeds 1 to 16, under the port and under the game's
rules, with and without six families, pass the monthly money check; the
first build's tables for the port were made again from a baseline of the
reviewer's own and came out as printed, and its "before" figures under the
game's rules to the last digit. Three things were mended.

- **The largest collieries were being given up at their height.** A working
  was abandoned whenever its cell was one "no land use may have": town
  ground, but also a cell the works had filled to six tenths, or one a
  closed works had left derelict. Works come to stand beside a colliery for
  its coal, so it was the best pits that went: on seed 3 under the game's
  rules 87 workings with 19,700 hands between them were lost to works
  filling their cells (one colliery of 2,175 men in 1895), and 50 quarries
  and clay pits with 5,600 hands to towns growing over them. Each loss freed
  the lease, somebody sank again beside it, and the town's wage jumped: 29
  town-months in that run with the wage moving by half or more in a month,
  against 7 with the mines off. Now only idleness ends a working that has
  hands, a town goes round every pit head at work, and works may fill the
  cell beside one (D, above). The same run loses one working, of under one
  hand, to a town and has 11 such months. This is most of the "bad years" worry
  of the first build.
- **Five hundred abandoned pits in 1750.** The surplus workings a run starts
  with were given up during the warm-up and then listed for the viewer as
  abandoned, with whatever little they raised in 1740 as their heap: 507
  rows on seed 3 beside 344 workings at work. What the warm-up gives up is
  no longer listed (21 rows in 1750 on that seed, each given up in 1750
  itself).
- **Two notes in the code that did not match it** (what a working's
  believed promise is, and when a town is kept off).

Extreme settings, two seeds each to 1900 under the game's rules with six
families (`mines-review/x-*`): `sink` 0 and 400, `reach` 0, 2 and 4,
`payback` 0, 0.5 and 20, `memory` 0 and 1, `idle_years` 0, 1 and 300,
`year_room` 0 and 5, `reserve` 0 and 100,000, `haul` 0 and 1,
`hands_per_cell` 0 and 200, `cell_limit` 0 and 1, in the combinations that
made 68 runs. None failed the money check and none ran away. `sink` 0 with
`reach` 0 is the port's pattern again (4,000 workings in 1900), as "what I
tried first" says it should be. A `reserve` two hundred times smaller makes
coal ten times as dear in 1900 and halves the baskets a head, so depth can
be made to bite; nothing breaks. The one setting to keep away from
is `idle_years` 1 with no `year_room`: one seed of two then has a year in
which the real wage falls by more than half. (Hasty closing and unlimited
opening chasing each other, I take it; I did not look.)

**The final build on sixteen seeds**, mines on against off, every figure
`before > after` with the seeds up and down (a star: every seed now clear of
every seed then). Seeds 9 to 16 were not looked at while anything was being
decided, and say the same as 1 to 8 unless noted.

Land in cells' worth, the game's rules with six families:

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| farm | 1,368 > 1,562 | 1,308 > 1,551 | 1,325 > 1,633 | 1,159 > 1,453 |
| pasture | 1,547 > 1,856 | 1,546 > 2,003 | 1,519 > 1,994 | 1,667 > 2,237 |
| forestry | 1,355 > 1,578 | 1,197 > 1,438 | 1,004 > 1,283 | 813 > 1,065 |
| quarries, pits and mines | 890 > 0.75 | 1,142 > 2.0 | 1,295 > 4.7 | 1,372 > 9.1 |
| wild | 621 > 782 | 545 > 735 | 486 > 686 | 393 > 589 |

All 16/0 or 0/16, and all starred but the farms of 1850. Under the port
alone: farm 1,410 > 1,626 and 1,288 > 1,607 (1750, 1900), pasture 1,515 >
1,824 and 1,303 > 1,649, forestry 1,210 > 1,420 and 853 > 1,109, mines 897
> 0.84 and 1,197 > 9.4, wild 558 > 711 and 288 > 407.

The workings (cells with a hand or more), the game's rules, six families:

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| quarries | 946 > 132 | 1,372 > 170 | 1,731 > 236 | 2,140 > 294 |
| clay pits | 123 > 86 | 692 > 88 | 1,040 > 108 | 1,116 > 117 |
| iron mines | 12 > 8 | 43 > 9 | 93 > 11 | 173 > 19 |
| collieries | 130 > 35 | 254 > 45 | 292 > 54 | 313 > 62 |
| mean share of its cell | 0.744 > 0.003 | 0.486 > 0.007 | 0.413 > 0.012 | 0.368 > 0.018 |
| the largest share | 0.48 > 0.04 | 0.53 > 0.11 | 0.57 > 0.18 | 0.60 > 0.22 |

Hands as a share of everyone at work, output a head a month, price over the
wage, and the royalty over everything made; the game's rules, six families:

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| quarrymen | 0.062 > 0.073 (16/0) | 0.079 > 0.077 (7/9) | 0.113 > 0.116 (9/7) | 0.133 > 0.138 (9/7) |
| clay diggers | 0.008 > 0.007 | 0.034 > 0.029 | 0.039 > 0.035 | 0.024 > 0.020 |
| iron miners | 0.001 > 0.001 | 0.002 > 0.001 | 0.003 > 0.001 | 0.004 > 0.003 |
| colliers | 0.009 > 0.009 | 0.047 > 0.043 | 0.049 > 0.047 | 0.050 > 0.048 |
| stone raised | 0.236 > 0.280 (16/0) | 0.347 > 0.377 (14/2) | 0.780 > 0.916 (11/5) | 1.55 > 1.77 (14/2) |
| stone's price | 0.264 > 0.229 (0/16) | 0.220 > 0.187 (0/16) | 0.144 > 0.122 (5/11) | 0.080 > 0.070 (1/15) |
| clay's price | 0.151 > 0.098 (0/16*) | 0.156 > 0.117 (0/16*) | 0.105 > 0.081 (2/14) | 0.059 > 0.043 (1/15) |
| ore's price | 0.644 > 0.511 (2/14) | 0.132 > 0.089 (1/15) | 0.082 > 0.047 (1/15) | 0.056 > 0.037 (1/15) |
| coal raised | 0.093 > 0.095 (8/8) | 0.566 > 0.690 (16/0) | 0.887 > 1.15 (14/2) | 1.21 > 1.35 (13/3) |
| coal's price | 0.834 > 0.762 (2/14) | 0.188 > 0.141 (1/15) | 0.129 > 0.100 (4/12) | 0.071 > 0.060 (4/12) |
| royalties | 0.013 > 0.024 (16/0) | 0.023 > 0.025 (8/8) | 0.024 > 0.034 (14/2) | 0.032 > 0.040 (15/1) |

Under the port alone the quarrymen are 0.063 > 0.083 of all hands in 1750
(15/1) and 0.160 > 0.174 in 1900 (9/7), the colliers 0.030 > 0.018 and 0.095
> 0.090; stone is 12 to 18 per cent cheaper in every year (0/16 in all
four), coal 19 to 31 per cent (1/15, 0/16, 2/14, 4/12), and the royalties
are 0.030 > 0.030, 0.034 > 0.035 and 0.044 > 0.044 of output from 1800.

Farm output and the standard of living:

| | | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|---|
| baskets a head | port | 1.74 > 1.88 (16/0) | 3.04 > 3.38 (15/1) | 3.33 > 3.96 (14/2) | 3.64 > 4.23 (14/2) |
| | port, six families | 1.75 > 1.89 (16/0) | 2.98 > 3.31 (14/2) | 3.45 > 4.13 (14/2) | 3.66 > 4.07 (12/4) |
| | game | 1.42 > 1.51 (15/1) | 2.89 > 3.00 (13/3) | 4.09 > 4.24 (11/5) | 8.33 > 9.11 (15/1) |
| | game, six families | 1.43 > 1.52 (16/0) | 3.01 > 3.14 (14/2) | 4.78 > 5.32 (12/4) | 8.03 > 8.86 (15/1) |
| goods a head | game, six families | 2.43 > 2.58 (16/0) | 4.44 > 4.61 (13/3) | 7.04 > 7.76 (13/3) | 10.9 > 11.7 (15/1) |
| real wage against 1750 | port | | 1.34 > 1.38 (13/3) | 1.30 > 1.40 (12/4) | 1.31 > 1.38 (12/4) |
| | game, six families | | 1.62 > 1.61 (10/6) | 2.00 > 2.03 (9/7) | 2.59 > 2.65 (11/5) |
| grain against 1750 | port | | 2.40 > 2.34 (7/9) | 4.29 > 4.70 (11/5) | 9.11 > 8.91 (7/9) |
| | game, six families | | 2.13 > 2.15 (11/5) | 4.40 > 4.47 (10/6) | 11.4 > 11.6 (11/5) |

So the gain is the first build's: five to ten per cent more baskets and
goods a head under the game's rules, clear in 1750 and 1900 and less so
between. The middle of the century is where the two halves of the seeds
part: with six families the baskets of 1800 are up 8 per cent on seeds 1 to
8 (8/0) and 1 per cent on 9 to 16 (6/2), and those of 1850 up 2 per cent
(5/3) and 22 (7/1). Take the sixteen together and do not read much into one
mark year. Grain and the real wage against a run's own 1750 are inside the
spread everywhere.

In company, the game's rules with six families: people in 1900 688 thousand
> 719 thousand (16/0); track in 1900 2,614 > 2,260 km (0/16, starred), lines
to quarries 72 > 13 (0/16, starred), and what the lines earn on their
capital 0.22 > 0.29 in 1900 (14/2); town cells 303 > 320 (13/3); the worst
year for the real wage -0.160 > -0.147 (10/6), and under the port alone
-0.174 > -0.161 (8/8), its worst seed -0.29 where the first build's was
-0.38 and the port's own -0.23.

A run alone on a quiet machine takes what it took: 17.7 seconds with the
mines off and 17.1 with them on, under the game's rules with six families,
and 16.4 and 16.9 under the port, where there are a sixth more works to run
(seeds 3 and 7, one run at a time). The "two seconds" of the first build's
sweeps was other work on the machine.

## What the first build's sweeps showed

These are the tables of the build before the review, on seeds 1 to 8, kept
because the page's argument was made on them; the review's baseline gives
the same "before" figures to the last digit, and the "after" of the final
build is in the section above. Every figure is the mean over the eight; "8/0" is how
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
whole deposit. The game's rules, six families, seeds 1 to 8, the final
build (the port's rule is the first build's baseline):

| | 1750 | 1800 | 1850 | 1900 | all the deposit |
|---|---|---|---|---|---|
| quarrymen's height, port's rule | 0.44 | 0.47 | 0.44 | 0.51 | 0.42 |
| quarrymen's height | 0.37 | 0.40 | 0.40 | 0.46 | 0.42 |
| height of the rock they work | 0.40 | 0.42 | 0.42 | 0.47 | |
| their path cost to market | 8.1 > 5.9 | 6.9 > 5.0 | 3.2 > 2.1 | 2.0 > 1.7 | |
| colliers' height | 0.22 | 0.18 | 0.18 | 0.22 | 0.24 |
| iron miners' height | 0.30 | 0.23 | 0.24 | 0.28 | 0.33 |

The typical quarry has 23, 34, 65 and 124 hands in the four years and the
largest colliery 124, 850, 1,413 and 1,968 (on 0.22 of its cell).

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
  (0.18 to 0.22 against 0.24), and so do the iron mines. So did the port's:
  that is the net price at work, not the leases.
- **They are spaced**, two to four cells apart in a worked field, not edge to
  edge, and on the map of seed 7 many of the large ones are on a railway or
  beside a town.

The review looked at it from above as well. The viewer's top-down plan
(`ISOSTATE_PLAN`, seed 7 with six families in 1900, from a `run.bin` of the
final build) shows towns, fields and woods and not one patch of diggings:
the blobs are gone, and since the plan draws only the ground, so are the
workings until somebody draws them. A picture of the workings themselves
(a square for each by its hands, on the relief, the coal measures, the
towns and the railways; a scratch program, not kept) for 1750, 1800, 1850
and 1900 on the same seed says what the heights say. In 1750 the quarries
are small and stand in loose groups on the flanks of the uplands, on the
sides nearest the six towns, and the few collieries on the corner of each
coalfield nearest a town. By 1850 the quarries make a ring round the foot and shoulders of the
central upland, nearly all of them on a railway, and the collieries have
spread along the edges of their fields beside the towns that have grown
there. By 1900 the railways cross the upland and the largest quarries are
on it, summit and all. So: at the foot of the hill until the railway climbs
it, and on the near edge of the ore field throughout.

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
  is capped at what the works have left, and a pit head stays when works
  fill the cell round it (four workings on seed 3 in 1900 have less land
  than their hands would take). Nobody sinks a new pit head on a cell the
  works have filled. The land rule runs before the year's works are founded,
  so for the rest of that year a cell can be a little over-full, as it could
  before. A works does not look at the workings' surface when it chooses a
  site, nor pay for it.
- **`towns`**: a town on kept ground does not take a cell with a pit head at
  work (`towns::update_urban` asks `mines::blocks_town`), and the port's
  yearly re-pick of town cells asks the same. Town cells in 1900 are 303 >
  320 with six families (13/3): more people, not less room.
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
  (`cell_rent_year`). A family with a branch of the mining or building trade
  in a town still gets its bonus on the units that stand on its land
  (`families::land_boost`); that is now the whole working if the family owns
  the pit head's cell, and nothing if it owns only cells of the lease. The
  families' own decisions are untouched. The scripted players
  (`examples/bots.rs`, seeds 1, 2, 3 and 7) play all sixteen games through
  on the final build. The review played them again with only the mines
  switched off, to see whether royalties had made land the thing to buy.
  They have not: the landlord finishes first on all four seeds either way,
  worth 61, 96, 50 and 41 million in 1900 with the mines and 98, 63, 63 and
  52 without; the thoughtful player is first on all four either way too
  (92, 71, 52 and 42 against 47, 69, 60 and 52). The passive player is
  where the histories differ most (fourth on three seeds with the mines and
  first on seed 2; first on seed 1 without and fourth on the rest), which
  is four islands' luck more than a rule.

**Beside kept town ground.** The first build's tables were measured on the
build this mechanism was written on, before `towns.md` was merged. The
branch has since taken it in (one clause was added to
`towns::update_urban`: a town on kept ground does not take the pit head of
an iron mine or colliery at work, as the port's did not; the review made
that any pit head at work), every test passes, and the sweep was run again under
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
1777; 1793, 1795). That left the digest's wording and its place among a
decade's four read by no test, so when the mines were merged into the
integration branch the test was given a second island: the first fifty years
of seed 8, which founds three towns in the 1750s and three in the 1790s, and
whose two digests are held to the same rules. `tests/machines.rs` checked
every active town's machines at the end of 1790; seed 5 now founds a town in
1790 itself, which has not had a month yet, and the test skips it.

**On the integration branch** (`night/core`, the merge of 11 November; seeds
1 to 6 with six families, the mines against the branch as it stood:
`.claude/night/reports/core-steps/mines-before` and `mines-after`). The
branch had gone on since the review, but not in the economy (saved games as
a log of orders, and the viewer). So the merged build is the reviewed one:
its six runs give every one of the 325 facts the review's sweep gave for
those seeds, to the last digit, and the branch without the mines gives the
review's "off". The sixteen-seed tables under "What the review found" are
therefore this build's, and are the ones to read. No run failed.

On six seeds alone the picture is the same at the ends and noisier in the
middle. People in 1900 694 thousand > 714 thousand (6/0); baskets a head
1.43 > 1.50, 2.94 > 3.17 and 7.83 > 8.52 in 1750, 1800 and 1900 (6/0, 6/0,
5/1); land under the diggings 1,372 cells' worth > 9.1; 491 workings in 1900;
track 2,615 > 2,230 km (0/6, starred), lines to quarries 70 > 12; town cells
303 > 323 (6/0) and works' cells 164 > 204 (6/0); works 699 > 746; the worst
year for the real wage -0.153 > -0.135 (5/1); the price swings of grain,
iron, rails and cloth unchanged (0.109 > 0.108, 0.172 > 0.166, 0.172 >
0.170, 0.037 > 0.037). The mark year 1850 goes the other way on these six
(baskets 5.06 > 4.47, the real wage against 1750 1.98 > 1.83, both 3/3):
seeds 1, 4 and 5 had their fastest years just before 1850 without the mines
and have them just after with, and seed 6 the reverse. On sixteen seeds
1850 is up (baskets 4.78 > 5.32, 12/4), as the page says above: do not read
one mark year on six islands.

Two things the sixteen seeds show that the page had not said, both
`machines.md`'s to look at:

- **Steel rails lose more of their share.** They were 0.19 of the rails
  made in 1890 without the mines and are 0.05 with them (2/13), and no seed
  of sixteen now has a year in which steel rails are half of all rails
  (three had). More rails are made, not fewer (1,627 > 1,872 a month in
  1890, 11/5), and more steel (15/1); what has moved is the iron rail,
  whose iron is a sixth cheaper against the wage in 1890 (0.47 > 0.39 wages
  a unit, 5/11) now that ore is a third cheaper. This is the weakness
  `machines.md` already owns up to ("steel rails are a quarter of rails in
  1890 where they were all of them, iron being half the price it was"), made
  worse by cheaper ore. The converter is untouched (0.92 of steel in 1880
  either way). The tuning that page asks for, of
  `MACHINES_STEEL_RAIL_WORTH` and its two companions with everything on, is
  now wanted a little more.
- **The powered mill is found later**: 1781 > 1790 in the mean, later on
  eight islands of sixteen and the same on the other eight; its first cloth
  1787 > 1794 (9/4). The engine works makes its first machinery four years
  later too (1810 > 1815, 10/5). I did not trace it. The mill's discovery
  waits on machinery made and cloth woven, so the guess is that an island
  with a fifth more pasture and farmland in 1750 keeps its hands on the land
  a few years longer. The worst is seed 13, which finds the mill in 1826
  where it found it in 1784 and still weaves four fifths of its cloth by
  hand in 1830; on the other fifteen the mill makes all the cloth by 1830,
  as it did on all sixteen.

A run took 20.6 seconds against 17.6 in these sweeps (6/0, six at a time on
a machine busy with other builds), and 28.7 against 24.5 one at a time on
seeds 3 and 7 the same evening: a sixth more, both times. The review timed
none on a quiet machine. It is nowhere near a worry, but "costs nothing"
may be too kind.

The viewer needed two things. Its loader had learnt to refuse a `run.bin`
that is cut short, so the table of workings is now read by the same rule (a
column that is missing its end, or of another length than the rest, is an
error and not an island without pits). And the demo that takes the
screenshots (`demo_orders`) was passing its test by accident: on the branch
as it stood the squares round its works cost 28 to 47 thousand each and the
family had 18, so it bought none, and the save held two orders only because
seed 7's best works was a millwright's, in a trade the family had first to
take up. With the mines the best works there is a brickworks, in the trade
the family starts with, and the save held one.
The demo now buys the nearest squares it can afford, up to four and with at
most half its cash, which is what its note always said it did.

## What worries me

- **Bad years.** Earlier builds of this mechanism had years in which a
  town's wage ran away: the port lets a town's labour demand exceed its
  supply for a month and doubles the wage, and a large working whose price
  has just jumped can ask for that, where the port's thousands of small ones
  could not all at once. The review found most of it in large workings
  being given up (above) and mended that. What is left: under the game's
  rules with six families a run still has a few more months in which some
  town's wage moves by half than it has with the mines off (11 against 7 on
  seed 3, 8 against 7 on seed 7), and the worst year for the real wage is a
  little better in the mean (-0.147 against -0.160, 10/6); without families
  it is level (-0.159 against -0.155, 8/8); under the port alone the worst
  seed of sixteen is -0.29 against the port's -0.23. A working is a big
  thing in a small town, and its hands still come and go with its price.
- **A town can be boxed in.** A town goes round a pit head at work, so
  workings on every side of a small town would stop it growing. Town cells
  are up with the mines on, not down, and I saw no town held; but nothing
  prevents it, and nothing lets a town buy a quarry out for building ground.
  A new town is still founded where the carting says, pit head or no, and
  the working on that cell is lost (three on seed 7 with six families:
  quarries of 4 and 32 hands and a clay pit of 54, in 1772, 1795 and 1834).
- **Land over a new lease is cheap for a year.** `families::land_price` is
  so many years of the rent a cell paid last year, with a floor. In the
  year a working is sunk the cells of its lease are charged their share of
  the sinkers, which can take a cell's rent below the floor just before its
  royalty begins. A player who watches for new pit heads can buy coal
  royalties at the price of heath. It is the railway's "buy ahead of the
  line" again (DEVLOG, open design points), only sharper, and it wants the
  same answer when that one is given. Cells with a deposit under them now
  count as farmland to the families (their main use is farm, pasture or
  wood), so the rivals' estates buy into royalties too; before, they
  passed over any cell whose main use was a mine.
- **A working raises while it is being sunk.** The sinkers are a cost and
  not a delay: there is no year between the decision and the first ton, as
  there was for a real shaft. A year's wait would be more honest and would
  give a slow market for new pits a hog cycle to contend with; I did not
  try it.
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
- **Speed** is not a worry after all: timed one run at a time, the mines
  cost nothing that shows (the review, above).
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
abandoned and one at work is not; works that fill a pit head's cell, or
leave it derelict, do not end the working, its surface is what they leave,
the cell is not over-full, and a town is kept off a pit head at work and
off no empty cell; and the viewer's table is recorded, packed, absent from
a port run, and lists nothing the warm-up gave up.

`cargo run -p isostate-core --release --example mines_census -- --rules game
--families 6 [--map coalmine] [--trace] 3 7` prints the tables above for any
rules and seeds, the map, and with `--trace` a sector year by year.
