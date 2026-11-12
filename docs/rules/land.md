# Land: an island made by water

A second way of making the island. Not in the Python. The switch is
`land.enabled`; the code is `landform.rs` (the plan of an island and the
erosion), `landwater.rs` (rain, lakes, flow, river courses) and
`landcells.rs` (the `World` the economy runs on, and what is kept for the
viewer). It draws from a random stream of its own (`rng::land_stream`,
stream 11); the port's `generate_world` is not touched.

It is **not yet switched on in the game's rules**. The economy stands on the
new land (the sweeps below, and "On the integration branch" for how it
stands beside the mines and the canals), but the viewer cannot draw it yet.
"Why it is off" says what is needed.

## What was wrong

The port's island is a sum of smooth noise with a ridge laid on it. Rivers
are found afterwards by letting water run down the 2 km cells, so they do
run downhill, but nothing about the land was made by them: no valley was cut
by its river, no basin drained by a tree of streams, no lake, no gorge, no
tableland. The resource fields are noise by terrain class. And everything
below a cell (relief, valleys, the bends of a river) is invented by the
viewer, which is why the meanders look like what they are, sine curves
beside the cell centres.

## What it does

The land is made on a grid finer than the cells (`fine` nodes to a cell's
side: 4, so a node is 500 m), in metres, in five steps.

**1. A plan.** The seed decides where things are before any water runs
(`landform::draw_design`): the line of a mountain belt (the port's curved
spine, which may lie either way round and lean), broken by a pass or two;
foothills beside it; one or two tablelands, each a block of hard rock 90 to
150 m thick lying on 220 m of soft, its beds tilted so that one edge stands
higher than the other; one or two basins of soft rock; a coastal plain, wide
in the lee of the island and narrow where the wind comes ashore; four
coalfields on the belt's flanks and seven ore bodies in it; and the lowland's
strata running in bands, hard ridge and soft vale by turns. The wind is
westerly, from within forty degrees of due west: south-west round to
north-west, give or take.

**2. Uplift and erosion** (`landform::Eroder`). Each step lifts the land (fast
under the belt, slowly elsewhere, and hardly at all in the middle of a basin
or at the shore of the coastal plain, which get a tenth and a twentieth of
what the same ground would get elsewhere),
routes the water over it, and lets every river cut down:

    dz/dt = U - K sqrt(A) S

the stream-power law: a river cuts in proportion to its slope S and the
square root of the area A it drains, at a rate K set by the rock under it.
It is solved from the sea upwards along the drainage in one pass (Braun and
Willett's implicit scheme), so a step can be long and 146 steps are enough.
Hollows are found by a priority flood and fill with what the water brings. A
river drops part of the load it carries from upstream wherever it runs
slacker than it could carry that load on (a fan at the mountain front, a
flood plain below); ground steeper than `max_slope` slides; soil creeps on
the hillsides, hard rock hardly at all, which is what keeps a scarp sharp.
The first 110 steps are run on a grid of half the fineness, where the trunk
valleys and watersheds are settled, and the last 36 on the fine grid.

Hardness does the rest. Where a river crosses a tableland's cap it cuts a
canyon and the soft rock under the cap is cleared from below, so the
tableland keeps a flat top and a sharp edge. In soft rock valleys open into
vales. The model keeps count of how much rock each node has lost, so it
knows which layer is at the surface.

**3. Water** (`landwater::make_water`).

- *Lakes* are scooped where ice would have left them: two to five stretches
  of upland valley deepened along the stream (in valleys draining 12 to 150
  square km, 100 to 520 m up and falling gently, no two within 28 km), the
  floor below the lip at the lower end so that the water stands the whole
  length, and on some islands a broad hollow in a lowland basin, its shore
  drawn in lobes and bays (86 to 223 square km on seeds 1 to 40). A trough
  is drawn 4 to 11 km long, but stops short where the valley comes down to
  60 m or has fallen 60 m, and is not dug at all if that leaves less than
  2 km. The upland lakes come out mostly 4 to 10 square km, with a few of 1
  to 4 and of 10 to 18 (seeds 1 to 40; some of the smallest are hollows
  the erosion left, not troughs). The chance drawn for the lowland lake is
  `lowland_lake`, 0.35, but a hollow scooped beside the coast is taken by
  the sea and is a bay: ten of forty seeds have the lake. These are the
  first things done to the finished land; the sea then rises and the big
  rivers plane their floors (below), and only after that is every hollow
  filled to its spill point. One of 0.75 square km or more is kept as a
  lake with its level, depth, outlet and outflow (the flow of the lake's
  nodes that drain out of it), and smaller ones silt up.
- *The sea rises* next, until `land_fraction` (0.58, the port's) of the
  cells are land, every hollow (the new lakes among them) counting as dry
  (it rises 60 m at most). The lower valleys drown into estuaries, which is how real
  ones were made.
- *The big rivers plane their valley floors*: flat, wider downstream, wide in
  soft rock and hardly at all in hard, where the valley stays a gorge.
- *Rain* comes in with the wind off the sea, falls more as the air is pushed
  up over high ground, and less beyond it: `rain` (900 mm) on the windward
  coast, most on the windward slopes of the belt, least in the rain shadow
  (the wettest land cell has four to six times the rain of the driest, 3.6
  to 6.9 times on seeds 1 to 40, the driest 300 to 550 mm a year and the
  wettest 1,900 to 2,300; the test asks for more than twice).
- *Runoff* is a monthly water balance in each cell: winter is wetter, summer
  takes more back, the soil holds 100 mm, and what is over runs off at once
  or seeps out slowly. It is kept by cell and month.
- *Flow*: the year's runoff gathered down the drainage gives every node a
  mean flow in cubic metres a second.
- *Marsh and flood plain*: every node knows its height above the water it
  drains to (a lake, the sea, or the first stream of 1.5 cubic metres a
  second it meets). Level ground, falling no more than 2 in 1,000, within
  0.9 m of that water is marsh or fen where the water is a lake or the sea,
  where the ground is less than 6 m above the sea, or where it is planed
  valley floor that itself falls less than 4 in 10,000.
  The rest of a planed floor, and any ground within 3 m of a stream, is
  flood plain.

**4. River courses** (`landwater::meander`). Every stream of `stream_flow`
(0.3 cubic metres a second) or more is traced as a line from its source to
the sea, a lake or the bigger river it joins. Then the lowland reaches are
moved by Howard and Knutson's model of meander migration: a bank erodes at a
rate set by the curvature of the bend there and, more strongly, by the bends
just upstream, so bends grow, travel down the valley, and at last meet and
cut themselves off, leaving an oxbow. A reach steeper than 4 in 1,000, or a
stream of less than 1 cubic metre a second, stays where the erosion put it,
and a reach moves the slower the nearer its fall is to that;
a bend cannot climb the valley side (it moves freely where the ground is
within 3 m of the water and not at all where it is 8 m above); a reach on
gravel too steep to hold one channel is marked braided. The biggest rivers
are moved first, and a tributary is led to where its river now runs. Each
point of a line has a width, a flow and a water level that never rises on
the way down. A river's width is 11 times the square root of its flow in
metres, about two and a half times the textbook's, so that a river and its
bends can be seen on a map of 2 km cells; a braided reach is given 2.2
times that.

**5. Cells** (`landcells::make_world`). The economy's `World` is filled from
the fine grid:

- A cell is *land* when at least half its nodes are dry. One island is kept.
  A cell of water that is not a lake cell (below) and that the open sea
  cannot reach from cell to cell is land too: it is an inlet narrower than
  a cell, and the tideways below are made of such cells.
- *Height* `h` is the mean height of the cell's dry nodes over `h_ref`
  (1,500 m), which makes the mean step between neighbouring cells what it is
  on the port's island (0.020 against 0.020), since the cost of carting
  uphill reads it. *Terrain* is from height and from the relief inside the
  cell: mountains above 380 m or with more than 260 m of relief, hills above
  90 m or with more than 50 m, else lowland.
- *Where water goes*: each cell's largest stream is followed on the fine
  grid until it leaves the cell, and that is `down`. A cell's stream is
  never larger than that of the cell it runs into, so there are no rings.
  `acc` is the area upstream in cells; a cell is a `river` from 5 cubic
  metres a second and `navigable` from 12 where it falls less than 1.5 in
  1,000 (its stream's fall over the fine steps that take it out of the
  cell, four at most), and it is both where a quarter of the cell is under
  a lake.
- *Tideways*. The sea's rise drowned the lower valleys into inlets narrower
  than a cell, so a river usually meets the tide a few cells inside what the
  cells call land (100 to 200 land cells an island, give or take, have
  tidewater in them and no cell of sea beside them: 82 to 234 on seeds 1
  to 40, and 99 to 207 on 1 to 12). Left at that, no navigable river
  reached the coast on twelve islands of sixteen, and the first towns had
  no port. So the cells follow
  the tide: every cell with tidewater knows the neighbour its inlet runs on
  into (always one nearer the open sea, counted in fine steps over the
  water), and the flow that has come into the inlet from there up. Every
  such cell takes the larger of its own largest stream and that tideway
  for its `discharge` and `acc`, and its `down` goes with the tide unless
  its own stream is the bigger and leaves the cell by another way. Where the
  tideway is a river's flow or more the cell is an `estuary` cell, and so a
  `river`; each island has 20 to 60 of them (21 to 59 on seeds 1 to 40).
  Where it is a navigable river's flow the cell is `navigable` (tidewater is
  level) and it is `coast`: ships come up it. Fewer cells are that: on
  seeds 1 to 40, 14 to 39 estuary cells an island are both navigable and
  coast (a few of them may be so by their own river or by the open sea
  beside them), and 10 to 34 are so with no cell of sea beside them. With
  them nine tenths of an island's navigable water is joined to the open
  sea by navigable water: 0.90 of it on seeds 1 to 12
  taken together and 0.93 on 1 to 40, though island by island it runs from
  0.74 (seed 11; seed 7 has 0.75) to 0.97.
- *The resource fields come from the land*: corn land from height, slope,
  rain and the rock's soil, best on the flood plain's silt where it is not
  marsh; grass where it is wet, cool or steep and on the water meadows;
  woodland from rain and height, thinning between 380 and 760 m to a
  third or less of full woodland above that (0.28 to 0.33 at most, on 1 to
  34 such cells an island, seeds 1 to 40; a tree line, but not a bare
  one); stone where hard rock is laid bare (scarps, gorges, the belt);
  clay in the vales and on the alluvium; coal in the coalfields, richer
  where the land has been worn down nearly to the seam; ore in the belt,
  and ironstone with the coal.
- *The first towns* are chosen by the port's rule (a fertile country round
  about, navigable water, the coast, 17 cells apart) with the places towns
  really grew counted in: the head of an estuary (the innermost cell of a
  river's tideway, which is the lowest place it can be bridged and as far
  as a ship can come, or the last cell of a river that runs straight out
  into the open sea), a confluence on navigable water, the outlet of a lake
  of 3 square km or more, a gap through the hills. Each adds 0.4 times its
  strength to the port's score: 0.1 to 0.3 for an estuary's head, by the
  size of its river, 0.5 for a confluence or a gap, 0.8 for a lake's
  outlet. Of the first six towns on twelve islands, 30 stand at an
  estuary's head, 27 at a confluence, 7 at a lake's outlet and 8 on plain
  ground (by the kind each site is given: a confluence on a tideway counts
  as a confluence even where it is also the tideway's head); none took a
  gap, though each island has 5 to 24 cells that count as one (4 to 24 on
  forty seeds). The first town of all stands on a tideway with 34 to 114
  cubic metres a second coming down it on all twelve islands, and at the
  tideway's head on eight of them; on half the twelve it is counted a
  confluence.

### A lake cell is water

A cell that is not land (less than half of it dry) and has more of its
water in a lake than in the sea is a lake cell, and water to the economy:
`land` is false and `terrain` is `SEA`, so nothing is grown or built there,
no town stands on it and no railway crosses it, and boats cross it at the
sea's rate. (So by the rule a lake cell may have well under half its ground
under the lake: five nodes of sixteen will do, if the rest is dry ground and
a little sea. None does on seeds 1 to 40, where every lake cell has at least
nine of its sixteen nodes under the lake. Its `h` is the lake's level.) Its
shore is not `coast`: that is tidewater, the sea's shore and the tideways,
which emigrants and (when it comes) foreign trade leave from.
`Landform::lake` says which water cells are lake, `Landform::shore` which
land cells are beside one, and `Landform::estuary` which land cells have a
river's tideway through them. Three small hooks, each marked `land:` where
it stands:

- `land::update_transport`: a town on a lake's shore has a harbour on it.
- `transport::build_graph`, three things. A boat on a navigable river goes
  on into the lake the river runs through and out again, at the river's
  rate. The mouth of a navigable tideway is a harbour whether or not a town
  stands there: a load passes between it and the open sea at a port's cost
  (0.6) and not a beach's (4). And a lake is not an arm of the sea: where a
  lake cell and a sea cell lie side by side (none does on forty seeds, but
  nothing forbids it) the load is carried over the bar at a beach's cost.
- `sim::init_before_firms`: a lake's shore counts as being by the water, as
  the coast and a river cell do, which gives a town's cell there two and a
  half times the weight in the draw of cells looked at as the site of a new
  works (`firms::candidate_cells`).

A ribbon lake a kilometre wide never fills half a cell, so most upland lakes
are not water cells: they are a share of their cells (`lake_share`), and
navigable where they take a quarter of one.

## Its numbers

`LandParams`, with the game's values (`LandParams::game()` is the default
with the switch on):

| | | |
|---|---|---|
| `fine` | 4 | fine nodes along a cell's side (even, 2 to 8) |
| `land_fraction` | 0.58 | share of the map that is land, the port's |
| `relief` | 1000 | height the belt is lifted towards, metres |
| `coarse_steps`, `fine_steps` | 110, 36 | erosion steps at half fineness, and at full |
| `erodibility` | 0.25 | K for rock of hardness 1, 1 square km, a fall over 500 m, per step |
| `deposition` | 0.6 | share of the load from upstream a slack river drops |
| `creep` | 0.06 | how far a hillside node moves to its neighbours' mean each step |
| `max_slope` | 0.55 | steepest slope rock holds |
| `upland_lakes` | 5 | the most upland lakes an island looks for: each seed looks for 2 or more, up to this many (0 for none) |
| `lowland_lake` | 0.35 | chance of a large lowland lake |
| `rain` | 900 | mm a year at sea level on the windward coast |
| `river_flow` | 5 | cubic metres a second from which a cell has a river |
| `nav_flow`, `nav_slope` | 12, 0.0015 | flow from which boats use it, and the steepest fall they bear |
| `stream_flow` | 0.3 | smallest stream kept as a line |
| `meander_steps` | 230 | steps of meander migration |
| `h_ref` | 1500 | metres that are 1 in the world's `h` |

The terrain thresholds, the fields' formulas and the rock hardnesses are in
the code beside what they do. They were tuned at `fine = 4`: at 6 or 8 the
same numbers give more mountain cells (842 and 1,028 on seed 7 against 635),
because a finer grid finds more relief inside a cell.

## What is kept

`World::landform` is `Some(Box<Landform>)` on an island the mechanism made
and `None` on the port's. Fine arrays are row-major, `fh` rows of `fw` nodes;
node (i, j) has its centre at ((i + 0.5) / fine, (j + 0.5) / fine) in cell
units, so cell (x, y) covers x to x + 1 by y to y + 1.

- Fine: `height` (metres above the sea, below 0 under it, a lake's bed under
  a lake), `rock` (basement, cap, soft, band, vale, coal measures, alluvium),
  `lake_id` (0, or the lake plus 1), `flow` (cubic metres a second), `wet`
  (dry, flood plain, marsh).
- `rivers`: each a line of points in cell units from the source down, with
  `width` (m), `level` (m above the sea), `flow` and `braided` at each point,
  and `end` (the sea, a lake, or the river it joins). `oxbows`: lines with a
  width and a level. `lakes`: level, area, depth, middle, outlet, outflow.
  `outlets`: where each river system meets the sea, the largest first, with
  the land it drains.
- By cell: `lake`, `shore`, `lake_share`; `estuary` (land with a river's
  tideway through it); `rain` (mm a year); `runoff` (mm in each month,
  `runoff[m * N + c]`); `discharge` (mean flow of the cell's largest stream,
  or of its tideway); `basin` (which sea outlet) and `stream` (which river line
  the cell's water joins first); `power` (kW: flow times fall, summed over
  the cell's streams); `floodplain` and `marsh` (shares of the cell);
  `wetness` (0 to 1); `slope`; `relief`; `site_kind` and `site` (why a town
  would grow here, and how strongly).

The water cycle is left in a form a later mechanism can drive.
`Landform::discharge_with(&world, &mult)` gives every cell's flow in a year
or month whose runoff is `mult[c]` times the average in each cell (the
harvest's wet and dry years, by district): flow is runoff gathered
downstream, so a dry district upstream lowers the river all the way to the
sea. `discharge_month(&world, m)` gives the average year's flow in month m.
The largest navigable river runs six to eight times as high in January as
in July on seeds 1 to 8 (5.2 to 9.9 times on 1 to 40), since summer's
runoff is little more than what seeps out of the ground; the test asks only
for 1.3 times, on seeds 1 and 6.

**In `run.bin`**, only when the mechanism is on (`landcells::static_arrays`
is the list, with shapes): `land_height`, `land_flow` (float32) and
`land_rock`, `land_lake`, `land_wet` (uint8), all `[fh, fw]`; the river lines
flattened as `river_start` (int32, river r's points are `river_start[r] ..
river_start[r + 1]`), `river_xy` (float32, `[points, 2]`), `river_width`,
`river_level`, `river_flow` (float32), `river_braided` (uint8), `river_end`
(int32: -1 the sea, -2 - L lake L, else the river joined); the oxbows as
`oxbow_start`, `oxbow_xy`, `oxbow_width`, `oxbow_level`; the cell fields as
`land_lakecell`, `land_shore`, `land_estuary`, `land_site_kind` (uint8), `land_basin`,
`land_stream` (int32), `land_lake_share`, `land_rain`, `land_discharge`,
`land_power`, `land_floodplain`, `land_marsh`, `land_wetness`, `land_slope`,
`land_relief`, `land_site` (float32, `[H, W]`) and `land_runoff` (float32,
`[12, H, W]`). The header has a `land` object: `fine`, `fw`, `fh`, `h_ref`,
`wind`, `drowned` (how far the sea rose, metres), `lakes`, `outlets`, and the
names of the rock, wet and site codes. A lite pack keeps the header and
drops the arrays.

**In the live API**, `LiveStatic::land` is the same `Landform`, or `None`.

The old arrays (`elev`, `terrain`, `river`, `nav`, `down`) are filled as ever,
so the viewer as it stands draws the island from them.

## Seeing it

    cargo run --release -p isostate-core --example land_picture -- --seeds 1-12 --out DIR
        [--field rain|flow|fertility|pasture|forest|rock|clay|coal|ore|wetness|power|terrain|geology|basin]
        [--zoom X,Y,WIDTH] [--px N] [--set NAME=VALUE]

writes a hill-shaded picture of each island with the rivers by their width,
lakes, oxbows, marsh, the coast and the first towns, and if asked a field
laid over it. It writes PNG itself (stored blocks), so nothing new is
depended on. `--example land_stats` prints the tables below, and
`--example land_check -- 1-40` makes as many islands as asked and holds each
to what the economy needs of it (six first towns, no ring in `down`, flow
that never falls downstream, river levels that never rise, every field in 0
to 1, water out of every lake), with `--set NAME=VALUE` for odd settings.

Looked at as a geographer would, twelve seeds: the rivers branch like rivers
and their trunk valleys widen downstream; the watersheds can be traced; the
tablelands have edges, a high scarp on one side and a back slope on the
other, with canyons where rivers leave them; the lakes lie in valleys behind
a lip, or in the basin, each with a river out; the coasts are drowned
valleys; every island is different. What is weaker is under "What worries
me".

## How long it takes

2.2 seconds an island in a release build on this machine when it is quiet
(0.9 for the relief, 1.3 for the water, most of that the meanders, and 0.02
for the cells), and up to 3 with other work running beside it; the viewer's
debug build compiles the core optimised, so it is the same there. By
fineness, seed 7: 1.5 s at `fine = 2`, 2.2 s at 4, 3.6 s at 6, 6.5 s at 8.
Four was chosen: a new game is ready in a few seconds, and the viewer adds
its own detail below 500 m. The simulation's yearly cost is unchanged:
nothing of the land runs after the island is made. A sweep's runs take 3.5
to 6 seconds longer on the new island than on the old (20 to 22 against
17), which is eight islands being made at once.

These times were taken when the page was written and the machine was quiet
or nearly so; they cannot be had again on a busy night. On 12 November,
with some thirty other builds and sweeps running, two runs of `land_stats`
on seeds 1 to 12 took 3.2 to 11 seconds an island, and seed 7 took 10 to
21 s at `fine = 6` and 14 to 15 s at 8, which says more about the machine
that night than about the land.

## Twelve islands in numbers

`land_stats`, seeds 1 to 12 (`cargo run --release -p isostate-core
--example land_stats -- 1-12`; run again on 12 November against the code
of that day: every figure in both tables came back but two last digits,
which are put right):

| | least | mean | most |
|---|---|---|---|
| lakes | 2 | 3.9 | 6 |
| largest lake, square km | 6.8 | 86 | 223 |
| islands with a lake over 100 square km | | 5 of 12 | |
| navigable river, tideway and lake, km | 202 | 290 | 388 |
| river lines, km | 2,759 | 2,873 | 3,041 |
| largest basin's share of the island | 10.5% | 18.5% | 31.1% |
| highest point, m | 897 | 978 | 1,074 |
| mean height of the land, m | 92 | 108 | 126 |

Against the port's generator, the same seeds (an island on average; each
field summed over its land cells):

| | port | land | ratio |
|---|---|---|---|
| land cells | 5,845 | 5,817 | 1.00 |
| hill cells | 1,825 | 1,501 | 0.82 |
| mountain cells | 680 | 620 | 0.91 |
| fertility | 3,185 | 3,182 | 1.00 |
| cells with fertility over 0.5 | 3,393 | 4,179 | 1.23 |
| pasture | 3,055 | 3,023 | 0.99 |
| woodland potential | 3,940 | 3,909 | 0.99 |
| stone | 1,600 | 1,634 | 1.02 |
| clay | 1,486 | 1,491 | 1.00 |
| coal | 210 | 197 | 0.94 |
| cells with coal | 475 | 412 | 0.87 |
| ore | 157 | 144 | 0.91 |
| river cells | 282 | 331 | 1.18 |
| navigable cells | 120 | 145 | 1.21 |
| coast cells | 430 | 533 | 1.24 |
| mean step in `h` between neighbours | 0.0196 | 0.0200 | 1.02 |
| mean `h` | 0.247 | 0.072 | 0.29 |
| of the six first towns, on the coast | 2.3 | 3.8 | |
| of the six first towns, on navigable water | 2.2 | 5.6 | |

Where it differs: a quarter more coast and a fifth more river and navigable
water (the estuaries: `river_flow` and `nav_flow` were set so that the
rivers above the tide match the port's, and the tideways come on top);
fewer hill cells; more cells of middling-to-good corn land for the same
total; coal in fewer, richer cells; and a much lower `h`, because `h` is now
height in metres over 1,500 and the port's was stretched to fill 0 to 1 on
every island. The lake cells (1 to 61 an island) come out of the land the
sea would have taken, so the land count holds. And the first towns stand on
the water: nearly all on a navigable river or tideway and most of them
ports, where the port's island has two or three of each.

## What the sweeps showed

*Measured under older rules.* Everything in this section and the next was
measured on the game's rules of the day it was written, and none of it can
be read from today's sweeps: with the switch off under both sets of rules,
their eight islands are the port's. The rules round the land have moved
since. For scale only, today's sweep of seeds 1 to 8 with six families
(`merged-game-f6-l`, 12 November, the land off) has 732 thousand people in
1900, 17.8 towns, a real wage 2.60 times 1750's and 2,248 km of railway,
where the "1-8, six" column below began from 641 thousand, 17.0 towns, 2.72
and 2,522 km on the port's island. What moved together then is a guide to
what the land does; the levels are not today's.

Sixteen seeds under the game's rules, the new land against the old, without
families and with six: seeds 1 to 8, which the land was tuned beside, and 9
to 16, which it was not. No run failed (every month's money is checked). A
different island is a different history, so nearly every fact differs seed
by seed; what matters is whether they moved together. Old to new, with the
seeds that rose and fell:

| | 1-8, none | 1-8, six | 9-16, none | 9-16, six |
|---|---|---|---|---|
| people in 1900, thousands | 621 to 606, 3/5 | 641 to 625, 3/5 | 617 to 621, 5/3 | 639 to 642, 4/4 |
| urban share 1900 | 0.409 to 0.406, 4/4 | 0.414 to 0.410, 3/5 | 0.424 to 0.415, 2/6 | 0.424 to 0.420, 5/3 |
| towns in 1900 | 16.1 to 16.6, 3/5 | 17.0 to 17.5, 5/3 | 14.5 to 16.2, 5/2 | 16.2 to 16.5, 3/5 |
| last town founded | 1846 to 1847, 4/4 | 1837 to 1845, 5/3 | 1852 to 1849, 3/5 | 1846 to 1842, 3/5 |
| real wage 1900 against 1750 | 2.85 to 2.89, 4/4 | 2.72 to 2.89, 5/3 | 2.80 to 2.85, 4/4 | 2.75 to 2.78, 4/4 |
| output a head 1900 against 1750 | 2.83 to 2.92, 5/3 | 2.76 to 2.95, 5/3 | 2.87 to 2.92, 3/5 | 2.81 to 2.92, 5/3 |
| farm share of work 1900 | 0.260 to 0.247, 2/6 | 0.256 to 0.246, 2/6 | 0.239 to 0.237, 4/4 | 0.240 to 0.234, 2/6 |
| first railway | 1839 to 1834, 3/4 | 1832 to 1836, 7/1 | 1846 to 1834, 1/7 | 1836 to 1828, 1/7 |
| railway km 1900 | 2,372 to 2,105, 0/8 | 2,522 to 2,222, 0/8 | 2,302 to 2,184, 2/6 | 2,496 to 2,426, 3/5 |
| towns with track 1900 | all, = | all, = | all, = | all, = |
| branches to works | 100 to 77, 0/8 | 102 to 78, 1/7 | 96 to 73, 1/7 | 95 to 83, 2/6 |
| branches to farms | 22 to 31, 7/1 | 27 to 34, 6/1 | 26 to 37, 7/1 | 33 to 44, 7/1 |
| works in 1750 | 224 to 212, 0/8 | 224 to 212, 0/8 | 216 to 227, 3/5 | 216 to 227, 3/5 |
| works in 1900 | 637 to 654, 3/5 | 678 to 668, 4/4 | 619 to 649, 5/3 | 679 to 689, 3/5 |
| famine years | 2.1 to 2.5, 4/4 | 1.8 to 2.2, 4/3 | 1.1 to 2.1, 5/3 | 0.8 to 1.4, 4/3 |
| emigrants | 436 to 555, 4/4 | 278 to 408, 6/2 | 335 to 347, 3/5 | 182 to 279, 6/2 |
| farm area 1750, cells | 1,387 to 1,487, 8/0 * | 1,382 to 1,482, 8/0 * | 1,395 to 1,492, 8/0 | 1,389 to 1,487, 8/0 |
| pasture area 1750 | 1,538 to 1,446, 0/8 * | 1,540 to 1,448, 0/8 * | 1,521 to 1,463, 0/8 * | 1,523 to 1,464, 0/8 * |
| land under mines 1900 | 1,410 to 1,321, 0/8 * | 1,399 to 1,310, 0/8 | 1,394 to 1,310, 0/8 | 1,400 to 1,311, 1/7 |

Read plainly: towns found and grow as they did; the railways find their
routes and reach every town; people, wages, output and the shape of the
century are inside the spread between seeds, in both halves. What moves on
every island or nearly, and holds on the second eight:

- **Less railway, by a twentieth to an eighth.** On the first eight seeds 11
  and 12 per cent less on every island; on the second eight 5 and 3 per
  cent, on five or six islands of eight. What holds firmly is the kind of
  line: a fifth to a quarter fewer branches to works, and a third to a half
  more to farms and pastures. The first towns stand on navigable water and
  tideways, so a works beside its town already has cheap carriage, and it is
  the farms inland that a branch pays for.
- **More land under the plough and less under grass** (7 per cent and 4 to
  6), from the first year: the new corn land is more even over the lowland
  and the grass is on the hills.
- **Less land under quarries, pits and mines** (6 per cent by 1900): coal
  and stone lie in fewer, richer cells.

Two things to watch that are not clear of the noise:

- **About half a famine year more a run** (2.1 against 1.4 over the 32
  pairs of runs; seed 7 without families has 8). Seventeen runs have more
  and thirteen fewer, so it is not every island. An earlier cut of the fields, with thinner soil
  on the hills, gave three times the famine years: one or two upland towns,
  founded early where a market saved carting, went hungry for years because
  little corn grew within reach. Raising the hills' corn land to nearer the
  port's brought it down to this. It is worth knowing that the people rule
  will found a town where its food has to be carted in.
- **More emigrants with six families** (6/2 in both halves), as would follow
  from more ports for them to leave by: four of the first six towns are on
  tidewater against two or three.

"Fewer works at the start" was true of seeds 1 to 8 (5 per cent fewer, on
every one) and is not of 9 to 16 (5 per cent more): it was those eight
islands.

Under the port's rules, seeds 1 to 8 without families, no run failed either:
682 thousand people in 1900 on both; the real wage in 1900 at 1.42 times
1750's against 1.29 (5/3); 990 km of railway against 1,024 (5/3); the last
town founded in 1781 against 1798 (2/6); pasture in 1750 down 5 per cent on
every seed.

The sweeps are in `.claude/night/reports/`: `sweep-landrev-base`,
`sweep-landrev`, and the same with `-f6`, `-s9` and `-f6-s9`, and
`sweep-landrev-port-base`, `sweep-landrev-port`.

## Why it is off

`game_params()` does not set it. What is missing:

1. **The viewer cannot draw it.** It reads the old arrays, so a game does
   start and play (`--shots` of a run made with `land = { enabled = true }`
   are in `.claude/night/shots/land/game`): the coast, the big lowland lake
   and the navigable rivers are there. But it draws its relief from `elev`,
   which on the port's island fills 0 to 1 and here is metres over 1,500, so
   the mountains hardly show; its meanders are still its own; a lake cell
   is drawn as sea, which for an upland lake is a pit down to sea level; and
   it knows nothing of a tideway, so a town at an estuary's head is a port
   with no sea in sight. Teaching it `land_height`, the river lines, the
   lakes and the tideways is the next helper's work; the viewer still reads
   none of it on 12 November, and `docs/probes/dx-land-view.md` measures
   what it would take and lays the work out in stages.
2. **Tests of other mechanisms are written to the port's island under the
   game's rules.** With the switch thrown for a trial on the island as it
   now is, one fails and the rest pass: `tests/chronicle.rs`
   `towns_and_districts_have_stories`, which asserts particulars of seed
   7's history. On the island as it first was, two more failed, and a
   change to the land may bring them back, since each depends on what
   happens to happen: `tests/chronicle.rs`
   `a_founding_redraws_only_the_districts_it_was_cut_from`, and
   `tests/machines.rs`
   `money_is_conserved_and_runs_are_deterministic_with_machines_on`, which
   found an active town whose machine table was still -1 (a town founded in
   the run's last year: an edge worth its owner's look whatever island is
   used). On the integration branch, beside the towns' ground, the mines and
   the canals, the same trial passes every test (183 of them): the
   chronicle's test had been given fifty years to find its five foundings
   in place of forty when the canals came, and on the new land it finds
   them. That is luck as much as soundness, and the next change to the land
   may undo it. The trial has not been made again since the saved games,
   the mines under the commons and the books of account came in, so it is
   not known whether every test passes with the switch thrown today (the
   comment above the switch in `params.rs` says they do, from that trial).
3. **It had not been run beside the mines and the canals**, which were tuned
   on the old island. It has now ("On the integration branch"): it stands,
   and what it does to each is written there. The water industry and the
   logging are still to come.

The scripted players are not in the way. `examples/bots.rs` plays the game's
rules, so it plays the old island until the switch is thrown; with it thrown
for a trial, all sixteen games (four strategies on seeds 1, 2, 3 and 7) play
to 1900 on the new land, and the player's fortunes are of the same order as
on the old (the thoughtful player ends first of six on all four seeds on
either island, with 57 to 127 million against 40 to 105).

To switch it on: `P.land = crate::landform::LandParams::game();` in
`game_params()`, run the tests (re-anchor any of the three above that fails
again), and run the sweeps again.

## How it was checked

`tests/land.rs`: the switch is off by default and a port run's `params.json`
and `run.bin` do not mention the land; the same seed makes the same island
to the last bit and another seed another; on three seeds the world is whole
(0.58 of the map land within a point and a half, no resource under water,
every field in 0 to 1, six towns 17 cells apart and off the mountains, corn,
coal, ore and navigable water enough); on two seeds every cell's water
reaches the sea or a lake without going round in a ring, a cell's stream is
no larger than the one it runs into, no river's level rises on its way down
and each ends where it says, water runs out of every lake, the wet side has
more than twice the rain of the dry, an average year gives the average flow
back, a year of 0.6 times the runoff 0.6 times the flow, and January runs
higher than July; on three seeds every land cell's water has somewhere to
go, a tideway is land and river and gets out to the open sea, four fifths
of the navigable water is joined to the sea by navigable water, some
tideway is coast and one of the first towns is a port; on one, the mouth of
a navigable tideway is crossed at a port's cost with no town there, every
other shore at a beach's, and a lake cell set beside a sea cell is not one
water with it; and thirty years under the game's rules with six families
on the new land conserve money every month, come out the same twice, and
write the land into `run.bin`. The golden test passes untouched. When the
land was added, a port run of seed 3, with and without families, was seen
to write the files it wrote before (`series_monthly.csv`, `events.json`,
`params.json`); that was a check made once by hand, not a test.

`examples/land_check.rs` on seeds 1 to 40: every island has its six towns
and breaks none of the rules above (run again on 12 November: so it
still is, with no lake cell beside the sea on any of the forty). With one
number at a time set to
something silly on seeds 1 and 2 (`fine` 1, 3, 8 and 100; `land_fraction`
0.2, 0.9 and 1; `relief` 0 and 3,000; no erosion steps; `erodibility` 0 and
5; `deposition` 0 and 3; `creep` 0 and 1; `max_slope` 0; `upland_lakes` 0
and 30; `lowland_lake` 1; `rain` 0 and 5,000; the three flow thresholds and
`nav_slope` at 0; no streams; `meander_steps` 0 and 1,500; `h_ref` 0) an
island is still made and still whole. Only `land_fraction = 0` leaves too
little land for six towns (two on one seed, one on the other).

The review found and mended: rivers that stopped short of the sea (the
tideways above); estuary heads that were looked for only where a river cell
touched a sea cell, which is where an estuary ends, not where it begins; a
lake's outflow given as nothing when its outlet ran straight to the sea; a
line of code meant to make a lake beside the sea a lagoon that could never
run; `upland_lakes` saying one more than it did, and unable to say none;
and a `max_slope` or `h_ref` of 0 sinking or spoiling the island.

## In company

The sweeps above have every mechanism on that was merged when they were
made; the towns' kept ground, the mines and the canals came after, and "On
the integration branch" has those. What to expect of each, written before
any of that was measured:

- **Harvest** has wet and dry years by district; `discharge_with` takes
  exactly that and gives the rivers of such a year.
- **Water power, floods, dams, irrigation**: `power` is flow times fall by
  cell, `floodplain` and `marsh` are where a flood goes and what drainage
  would win, `runoff` by month is the regime a reservoir evens out, and
  `Lake::outlet` is where a dam would stand.
- **Canals** will find lakes and estuaries in their way and navigable water
  to join; a canal's build graph should treat a lake cell as water to use,
  not land to dig, and a tideway cell is navigable already. With the rivers
  joined to the sea there should be less for a canal to do than on the
  port's island. Measured since, in "On the integration branch": there is a
  seventh less canal, and a lake is still neither used nor joined.
- **Logging** reads `forest_pot`, which now follows rain and height, and
  rivers that could float timber (`discharge`, `stream`).
- **People** founds towns where a market saves carting, with a liking for
  rivers and coasts; `site_kind` and `site` are there if it wants the real
  reasons. Its emigrants sail from the nearest town on the `coast`, which
  now includes a town on a tideway; before that, two islands of twelve
  began with no port at all.
- **Anything that reads `coast` as "beside a cell of sea"** (a harbour
  drawn at the water's edge, foreign trade's quays) should know that a
  tideway cell is coast with no sea cell next to it: `Landform::estuary`
  says which.

## On the integration branch

`night/core`, the merge of 12 November, where the land met the towns' kept
ground, the mines and the canals for the first time: everything above was
measured before them, on islands whose quarries and pits were still the
port's and which had no canal. The sweeps are under
`.claude/night/reports/core-steps/`, all under the game's rules with six
families: `land-before` and `land-after` (seeds 1 to 6, the switch off),
`land-on` (the same seeds, on), `land-x-s7-16` (seeds 7 to 16, off and on),
`land-x-canals-mines` (seeds 1 to 6 with the land, the canals and the mines
each on and off, 48 runs), `land-x-timing` and `land-x-timing6`.
`examples/land_canals.rs` says where the canals of a run lie with respect to
the island's water.

The saved games, the mines under the commons and the books of account came
in after this, and the new land has not been run again since. The port's
side of the table below is still close to today's: the sweep of 12 November
on seeds 1 to 8 (`merged-game-f6-l`, the land off) has 732 thousand people
in 1900 for the table's 726, 17.8 towns for 17.2, a real wage of 2.60 for
2.61, 2,248 km of railway for 2,254, 892 km of canal for 903, 0.125 of all
hands carting in 1750 for 0.127 and 0.4 famine years for 0.4; those are
eight seeds against sixteen. The new land's side cannot be read from it.

**The merge.** Four places where the land and the mechanisms merged since
had each added lines at the same spot: the end of `Params` and of its two
constructors, the arrays and the header in `record::pack`, the streams in
`rng.rs` and the list in the README. All kept, the land's last. No stream
number is shared (the people have 8, the canals 9, the land 11). One place
has an order to it, `land::update_transport`: the harbours are marked (a
town on the coast, and on the new land a town on a lake's shore),
`transport::build_graph` makes the ground's graph with the land's three
hooks in it, and then `canals::float` lays the canals over it. The canals
come last because they are a layer over a finished graph; nothing of the
land's reads them.

**Off is the branch as it stood.** With the switch off, seeds 1 to 6 give
all 420 facts of `land-before` to the last digit. The golden constants are
untouched, the viewer builds and its tests pass, and no run failed in any
sweep.

**On, it stands.** Sixteen seeds with everything on, and seeds 1 to 6 again
with the canals, the mines or both off: 34 runs on the new land, none
failed, every month's money counted. A run takes three seconds longer when
six are made side by side (22.6 for 19.5) and under one when it runs alone
(19.3 for 18.6), which is the island being made. On for off, sixteen seeds,
with the islands higher and lower:

| | the new land for the port's | |
|---|---|---|
| people in 1900, thousands | 729 for 726 | 10/6 |
| towns, 1800, 1850, 1900 | 13.8 for 12.6, 18.1 for 16.6, 19.2 for 17.2 | 13/2, 9/3, 12/2 |
| later foundings that are quarry towns, colliery towns | 3.6 for 2.4, 1.1 for 0.5 | 10/5, 7/2 |
| share of people in towns, 1750, 1900 | 0.209 for 0.217, 0.417 for 0.420 | 2/14, 8/8 |
| carters among all hands, 1750, 1800, 1900 | 0.140 for 0.127, 0.134 for 0.119, 0.072 for 0.066 | 15/1, 15/1, 13/3 |
| real wage, 1800, 1850, 1900 | 1.61 for 1.59, 2.04 for 1.99, 2.73 for 2.61 | 8/8, 10/6, 12/4 |
| output per head, 1800, 1850, 1900 | 1.66 for 1.66, 2.21 for 2.16, 2.78 for 2.67 | 9/7, 10/6, 13/3 |
| first canal | 1774 for 1774 | 9/6 |
| km of canal, 1780, 1800, 1850, 1900 | 345 for 261, 595 for 695, 763 for 900, 768 for 903 | 10/5, 2/14, 3/13, 3/13 |
| what the canals cost, in years of output | 0.108 for 0.142 | 0/16 |
| the boats' work in 1820, million weight-km | 474 for 432 | 13/3 |
| of their load in 1800: coal; stone and clay | 0.35 for 0.41; 0.24 for 0.16 | 2/14; 16/0 |
| first railway | 1832 for 1827 | 12/4 |
| km of railway, 1840, 1850, 1870, 1900 | 566 for 820, 1,163 for 1,348, 1,847 for 1,829, 2,426 for 2,254 | 4/11, 4/12, 8/8, 13/3 |
| branches to farms, to works | 57 for 33, 106 for 120 | 14/2, 3/13 |
| works, 1900 | 753 for 729 | 7/9 |
| town cells and works' cells, 1900 | 332 for 324, 194 for 198 | 9/7, 6/9 |
| workings, 1750, 1900 | 302 for 261, 506 for 491 | 14/1, 11/5 |
| clay pits, 1900 | 158 for 116 | 16/0 |
| dug a head in 1800: stone; clay | 0.55 for 0.39; 0.13 for 0.17 | 16/0; 1/15 |
| land under workings in 1900, cells | 9.2 for 9.1 | 8/8 |
| farm and pasture in 1750, cells | 1,727 for 1,562, 1,746 for 1,856 | 16/0, 0/16 |
| price swing: grain, iron, rails, cloth | 0.111 for 0.113, 0.170 for 0.174, 0.178 for 0.179, 0.040 for 0.039 | 8/8, 8/8, 9/7, 12/4 |
| famine years in a run | 1.2 for 0.4 | 7/3 |

People, the works, the land they stand on and the swings of prices are
inside the spread between islands. No price sits at a floor or a ceiling.
What moves:

**The canals have less to do, as expected, and do it cheaper.** A seventh
less canal by 1900 on thirteen islands of sixteen, for three quarters of
the money on all sixteen, carrying a tenth more. They start as early and
are a third longer in 1780, then stop sooner: the first towns are on
navigable water already and a short cut joins a good deal. The boats carry
more stone and clay and less coal.

**They keep to the rivers more than they did.** On seeds 1 to 6, 27 to 35
per cent of a run's canal cells are river cells, against 13 to 21 on the
port's island, and 13 to 19 per cent are on water boats already use
(a navigable river or a tideway), against 5 to 13. A run has 35 to 78 links
of canal that go from one navigable cell to the next, 49 on average for
31, on less canal. That is `canals.md`'s "one river, one canal" grown: a
boat on a canal pays 0.13 a cell and no wharf to stay aboard, a boat on the
river 0.3, a river cell is cheap to dig, and the new land has a fifth more
navigable cells with the towns standing on them. It is a river navigation
more often than a canal beside a river, and nothing in the rules knows the
difference. One to five per cent of the canal lies on a tideway.

**The lakes are out of it.** A canal cannot be dug through a lake cell
(nothing can be built on water) and does not use one: a boat that comes to
a lake's shore by canal lands its load at the wharf and carries it over a
beach to the lake's boats, unless a town stands there. It hardly shows,
because an island has two to four lake cells (33 on seed 6, which has the
lowland lake, and 2 per cent of its canal is on the shore), but the
expectation above, a lake as water for a canal to use, is not met.

**The railway comes later and ends longer, which is not what the page
found.** Before the mines and the canals the new land had a twentieth to an
eighth less railway in 1900. Beside them it has a third less in 1840, as
much in 1870 and 8 per cent more in 1900, on thirteen islands of sixteen.
Seeds 1 to 6 with the mechanisms taken out one at a time say why the end
differs. On the port's island the mines cut the railway of 1900 by a
seventh (2,615 km with neither, 2,230 with the mines, 2,214 with both). On
the new land they hardly do (2,418, 2,357, 2,427). I take the reason to be
that the port's diggings had been spread over a fifth of the land and are
now a few hundred pit heads, while the new land's stone and coal were in
few cells to begin with; that is a reading of six seeds, not a tracing. So
the "less railway" above was measured against an island whose scattered
diggings wanted branches, and beside the mines the new land is the one
with more to carry. The late start is canals and mines together (593 km in
1840 with both on seeds 1 to 6, 840 to 890 with either or neither), and
that I did not trace at all. Lines run over a little more: the worst in a
run costs 3.1 times what was expected for 2.7, on twelve islands. The kind of line holds as the page has it: three quarters more
branches to farms, a ninth fewer to works.

**It is not a cheaper island to carry goods about.** "What worries me"
supposed it was, from the harbours at the tideways. The carters say
otherwise: one hand in seven carts in 1750 against one in eight, on fifteen
islands of sixteen, and still a tenth more in 1900. The first towns stand
on the water, so the corn, the stone and the coal inland are further from
a market.

**Two more towns, and they are quarry towns.** 19.2 towns in 1900 for 17.2,
on twelve islands of sixteen, where the page found none more. Seeds 1 to 6
again: on the new land 16.8 towns without the mines and 19.2 with them, on
the port's island 18.7 and 18.3. The stone lies on the scarps and in the
belt, the mines make each quarry a working with its hands at one place, the
first towns are down on the water, and `people.md`'s rule founds a town
where a market saves the most carting. On seed 16 twelve of the seventeen
foundings are quarry towns. The canals cost this island no towns (19.3 with
them, 19.2 without, seeds 1 to 6).

**More stone is dug and less clay**: 0.55 a head in 1800 for 0.39 on all
sixteen islands, at an eighth less against the wage, and a quarter less
clay, from pits that are a third more numerous by 1900. Rich stone in few
cells, and clay spread thin over the vales. Not traced further (what is
built of each).

**Famine years: 1.2 a run for 0.4.** More on seven islands, fewer on three.
Most of the difference is seed 16, where Ormside, a quarry town founded in
1776, goes short of bread in six of its first eleven years and again in
1849, when its households get a fifth of the food they need. Without that
island it is 0.7 for 0.4. This is the page's warning come true in company
(a town founded where its food has to be carted in), and the quarry towns
are where it happens.

**A small town's branch waited twenty-three years.** Seed 1, Kirkrigg, a
town of 3,000 people and falling: a branch of three cells to a works,
promoted in 1868 and opened in 1891. Nothing was wrong with the line or
its materials. Its landlords had about a hundred in cash, a call takes a
tenth of that a month, and because they were not in debt the households
(33,000 in cash) were never asked. That is `ways::progress`, not the land,
and it wants its owner's look: landlords with a few shillings shield the
households who subscribed with them. It was left alone here, since mending
it would move every run under the game's rules, the land off or on. The
new land shows it because it has more small towns. No line had been
building ten years when a run ended, and on seeds 7 to 16 the slowest line
takes six years on either island.

**The scripted players** play all sixteen games through on the new land
beside the mines and the canals (`core-steps/land-bots-game.txt`, seeds 1,
2, 3 and 7, the switch thrown for the trial). The thoughtful player ends
first of the six families on all four seeds, with 34 to 105 million against
41 to 57 on the port's island; the landlord first on three and second on
one; the passive player and the founder first to fourth, as on the port's.

## What worries me

- **The tablelands are a little too neat**: compact, flat-topped, the same
  brown. They have edges and canyons, but a real one is more ragged and
  carries outliers and a rolling top.
- **The lowland's banded strata hardly show.** They are in the rock (hard
  ridge, soft vale) and the erosion respects them, but the lowland is lifted
  so little that the ridges come out as a few metres.
- **The lowland lake is still a scooped hollow.** Its shore has lobes and
  bays now and is no longer an oval, but it does not follow the valleys it
  drowned as a real lake's does. One island in four has it (ten of forty
  seeds), and where the hollow fell beside the coast there is a wide bay.
- **Coal follows its field's shape** more than its seams: the richness is
  raised where the land is worn down to the seam, but at 2 km a cell
  averages valley side and hilltop together, and the picture is still four
  patches.
- **Width is exaggerated** two and a half times, and with it the meanders'
  wavelength (about 11 widths): honest in form, not in scale.
- **Deltas are not built.** Mouths are estuaries, which suits an island that
  has just had its sea level rise, but a river with a heavy load into a
  sheltered bay would build one.
- **Erosion ignores the rain.** A river's cutting power uses the area it
  drains, not its flow, so the wet side is not cut deeper than the dry.
- **The first towns all stand on the water.** Of six, 5.6 are on navigable
  water and 3.8 are ports, against 2.2 and 2.3 on the port's island, whose
  rule looks mostly at the corn land round about. Here the corn land is
  more even, so the water decides. It is where towns grew, and the economy
  stands on it, but it is a different start: carriage between the first
  towns is cheaper, and the railways' branches go to farms more than to
  works. None of the first towns took a gap. The weights are a guess.
- **An estuary's mouth is a harbour for nothing.** On the port's island
  only a town makes a harbour, and a river's boats reach the sea over a
  beach unless a town stands at the mouth. Here every navigable tideway
  lets them out at a port's cost. That is what an estuary is, but it makes
  this a cheaper island to carry goods about, and some of the missing
  railway is surely that. (So I thought. Beside the mines and the canals
  no railway is missing by 1900 and more hands cart, not fewer: "On the
  integration branch".)
- **Quarry towns are founded where their bread has to be carted in.** The
  stone is on the scarps and in the belt, the corn is not, and beside the
  mines the new land founds two more towns a run, most of them at the
  quarries. One island of sixteen has such a town short of bread for seven
  years. The founding rule weighs the carting a market would save; it does
  not ask what the town will eat.
- **A third of the canal follows a river.** A third of a run's canal cells
  are river cells and a sixth are on water boats use already. A river
  made navigable and a canal dug beside it are one thing to the rules, at
  one price.
- **A lake is not water to a canal.** It cannot be dug through and it is not
  joined: a canal that comes to the shore ends at a beach.
- **A tideway is judged by the river that enters it**, not by its own width
  or depth: 12 cubic metres a second of river and boats use it all the way
  down, however narrow the inlet.
- **Sixteen nodes are few** for a cell's slope, relief and shares. The
  thresholds are tuned to `fine = 4` and move with it.
- **Platform maths.** The plan and the water use `exp`, `sin`, `cos`,
  `atan2`, `powf` and `ln`, so another
  machine may round a last digit the other way and grow a different island
  from the same seed, as the golden test's page already warns of the port.
- **The old viewer's `river` byte** is 1 plus 20 times log2 of `acc`, and
  `acc` is still area in cells, so a river on the wet side is drawn no wider
  than one of the same area on the dry.
