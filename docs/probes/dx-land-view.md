# dx-land-view: what it takes for the viewer to draw the new island

Branch `night/dx-land-view` (from `night/core` at 859b680, 12 November 2026), gone over by a critic on `night/o-land-view-critic` the same day: the section "What the critic found" says what was run again, what was corrected (each correction is marked where it stands) and how the stages should change; and the last section, "What was built: stages 0 and 1", says what became of the first two stages on `night/map-01`. It adds two pieces of measuring code and four small hooks, none of which changes a picture or a run unless it is asked for:

- `crates/isostate-core/examples/land_view_probe.rs`: makes islands with the land mechanism and prints what a picture would have to be made from (section 2).
- `crates/isostate-viewer/src/landview.rs`: a probe that draws the core's land with today's ground. With `ISOSTATE_LAND_VIEW` set and a recorded run that carries the land, the heightfield is laid through the core's fine grid, the rivers are the core's lines at the core's widths, the valley floors are the core's wet nodes, and each lake has a sheet of water at its level. Everything else reads the ground as before. `ISOSTATE_LAND_REPORT=1` prints the heights and slopes as drawn, the ground's mesh, and how the year's roads cross the rivers, with or without the probe.
- The hooks: `replay::load` (keeps the land aside), `world::build_ground_inner` (takes the ground from the probe when there is one), `network::rivers` (adds the lakes), `main` (the report).

The probe is for looking and measuring. It is not the stage-one ground: it keeps the land in a global, bends the shore to fit today's thresholds, and draws lakes as squares. The design is in section 6.

Results are on the machine only: measurements under `.claude/night/reports/land-view/`, pictures under `.claude/night/shots/land-view/`, the recorded runs under `.claude/night/tmp/land-view/`. Machine: the workstation the other probes used (48 threads, RTX 4090); other helpers were building beside this one, so the timings are good to a few tens of per cent and no finer. The viewer is the dev profile throughout. The viewer's 119 tests pass with the probe compiled in. With neither variable set, six views of the same run taken before and after the probe was compiled in differ as two takes by the same program differ (0.01 to 0.09% of pixels by more than 8 of 255 against 0.001 to 0.09%: carts, trains and the sea move in a `--view`).

Words: a **node** is a point of the core's fine grid (500 m apart); a **vertex** is a point of the viewer's ground mesh (250 m apart today); a **cell** is the economy's 2 km square; heights **as drawn** are in the viewer's units, where 1 is a cell's side.

## In short

1. **The core hands over more than the viewer needs to start, in a form it can take as it is.** A grid of 480 by 336 nodes of 500 m with height in metres, rock, lake, flow and wet ground; 215 to 242 river lines an island as polylines with a width, a water level and a flow at every point (66 to 72 thousand points, 40 m apart, some 2,760 to 2,930 km; *corrected by the critic: it said 2,800 to 2,900, and the sums of the five classes of section 2 run from 2,759 on island 6 to 2,933 on island 8*); lakes with level, depth and outlet; and seventeen fields by cell. It is 4.2 to 4.3 MB in `run.bin` and the whole `Landform` in a live game. The viewer reads none of it today.
2. **Today's viewer on the new land is wrong in the ways `land.md` says, and worse at the towns.** The mountain belt of 900 m is low rolling ground, a tableland with 300 m scarps is a swell, an upland lake at 95 m is two sandy pits down to the sea, the lowland lake is an arm of the sea with causeways across its corners, and the first town, which the core put at the head of an estuary, stands in the viewer's own invented estuary with its church in the water (pictures `today/`).
3. **The biggest doubt is settled: one function does most of it.** Everything in the viewer reads one heightfield, one list of rivers and one field of valley floors (`world::Ground`). Fill those three from the core and the island is drawn: mountains, valleys with their own rivers in them, tablelands with wooded scarps, lakes with natural shores, tideways as arms of the sea, towns on their water (pictures `probe-e3/`, `probe-seed2-e4/`). Laying 646,753 vertices through the nodes takes 6 to 8 ms, bedding the rivers 11 to 19 ms; the ground's mesh is the size it is today (577 thousand vertices, 18.5 MB, against 575 thousand).
4. **A finer mesh buys little, and relief added below the fine grid's scale is not seen.** The mesh is already twice as fine as the data. At sixteen vertices to a cell (2.27 million vertices, 73 MB) the picture is a little crisper and no more. Relief of an honest size below 500 m is ten or twenty metres, which is less than one storey of a house as the diorama draws houses (about thirteen times life size); it shows at neither resolution. What keeps a close view from being smooth lumps has to be breaks of slope the model does know (the edge of a valley floor, the wall of a cap rock, a river's bank) and the ground shader's paint.
5. **The real widths are honest and thin; the core's width constant is the thing to change, not the viewer's.** The core's rivers are 6 to 83 m wide on island 7 (today's drawn rivers are 52 to 202 m, a main road 68 m). At play distance they read as fine blue lines, which is pleasant, and their bends (some 800 m from loop to loop) read as a crinkle. Drawing the water wider than the core's lets the loops run into each other. With the core's own constant raised from 11 to 24 (one line of `landwater.rs`, tried and put back), the same model gives fewer, larger, less regular loops on the scale of the towns and fields, and the viewer draws them at the core's width (pictures `probe-e3-corewide/`). That constant should be one of the land's numbers.
6. **Bridges: 643 where 126 would join the same roads.** On the new rivers the roads of 1900 cross a river of 1 cubic metre a second or more 643 times (972 with every stream), and 82% of those crossings lie within 6 km along the river of the crossing before. The same roads stay joined with 126 links across a river (95 if a brook under 15 m is forded; *corrected by the critic: it said "crossings"*). It is the same fault on today's island (370 crossings where 42 would do). A road's links and every bridge are the viewer's own, so the viewer can choose them better without a word from the simulation; for a bridge to be a fact the simulation needs roads as links, a cost for crossing, and someone to pay (section 6, stage 4).
7. **Canyons, plateaus and scarps are made, and they survive.** Every island of eight has 170 to 930 square km of tableland whose edge falls 225 to 380 m within a kilometre, and 8 to 53 km of river in a canyon 190 to 360 m deep. On a 500 m grid a canyon cannot be narrower than a kilometre, so at three times relief it is a steep combe; at four or five times, with the steep ground painted as rock, it is a gorge with walls (pictures `probe-seed2-e4/tableland-se-1800.png`, `probe-e5/canyon-1850.png`).
8. **What breaks when the ground is real:** the thresholds written in the viewer's units for the old shore (nothing is built below 0.05 as drawn, which is 33 m at three times relief, and about half the island is lower than that); the railways, which run over ridges on viaducts because their course keeps to the middle of each cell; the sea's colours, tuned to a sea floor the core does not have; trees and houses that stand in lakes; and the summer snow, which no hill of 1,000 m carries.
9. **The stages** (section 6): (0) the land reaches the viewer; (1) the ground from the core's heights; (2) water: rivers, lakes, tideways, falls, the far thread; (3) the land's classes on real hydrology, and the heights in metres; (4) roads, bridges, fords and carts; (5) railways, canals, harbours and boats; (6) scarps and canyons made to read. The old island is drawn by the old code at every stage, and its standard shots must not change by a pixel; the new land stays behind the core's switch until stage 5 is in.

## 1. What the core knows

From `landform.rs`, `landwater.rs`, `landcells.rs`, `record.rs` and `live.rs` on `night/core`, and from running them.

**The fine grid.** `fine = 4` nodes to a cell's side: 480 by 336 nodes of 500 m on the map of 120 by 84 cells, 161,280 nodes. Node (i, j) has its centre at ((i + 0.5) / 4, (j + 0.5) / 4) in cell units. At each node:

| | type | what |
|---|---|---|
| `height` | f32 | metres above the sea; below 0 under it; a lake's bed under a lake |
| `rock` | u8 | basement, cap, soft, band, vale, coal measures, alluvium |
| `lake_id` | u16 (u8 in the record) | 0, or the lake plus 1 |
| `flow` | f32 | mean flow through the node, cubic metres a second |
| `wet` | u8 | dry, flood plain, marsh |

Kept in memory that is 12 bytes a node, 1.9 MB. The viewer's vertex grid today is eight to a cell, 961 by 673: twice as fine as the nodes each way.

**The sea.** There is no mask of the sea in what is kept. The core's rule (`landwater::open_sea`) is every node at or below 0 that the map's corner reaches. On eight islands no dry node stands at or below 0, so `height <= 0` is the sea. The sea rose 5.0 m at the end on island 7 (`drowned`; 4.6 to 5.7 m on the eight). *Corrected by the critic: this said "so a tideway is some 5 m deep at most". It is shallower than that for the most part and deeper in places: of the sea's nodes inside land cells, half lie under 1.2 to 1.4 m of water, nine tenths under 3.1 to 3.8 m, and the deepest under 16 to 18 m (`land_view_critic`, seeds 1 to 8).* The open sea floor goes down to 165 m.

**River lines.** Each river is a polyline from its source to the sea, a lake or the river it joins (`River`: `x`, `y` in cell units, and at every point `width` in metres, `level` in metres above the sea, `flow`, `braided`; and `end`). A line begins where the stream first carries 0.3 cubic metres a second. Points are 40 m apart (the 90th percentile 40 to 46 m). Meanders are in the points: the line has been moved by the migration model and smoothed, and the ground under it cut down to its water at the node it passes through. Cut-off loops that are still water are `oxbows` (12 to 45 an island), each a line with one width and one level; loops cut off in the first three fifths of the migration are taken to have silted up and are not kept.

**Lakes.** `Lake`: level, area, greatest depth, middle, outlet, outflow. Two to six an island. The upland ones are 1.5 to 16 square km (6 to 63 nodes); a lowland one, on three islands of eight here, is 86 to 201 square km at 5 or 6 m above the sea.

**Tideways.** A tideway is not a line and not a field of its own: it is the sea's nodes inside what the cells call land. By cell there is `estuary` (a river's tideway runs through this land cell) and, in the world's own arrays, `coast` and `navigable` for the cells ships and boats reach. The river lines stop where they meet the tide.

**Rain and runoff.** By cell: `rain` (mm a year), `runoff` (mm in each of twelve months), `discharge` (the cell's largest stream), `power` (kW), `basin` (which outlet to the sea), `stream` (which river line), `floodplain`, `marsh`, `wetness`, `slope`, `relief`, `lake`, `shore`, `lake_share`, `site_kind`, `site`. `Landform::discharge_with` and `discharge_month` give the flows of a wet year, a dry district or a month.

**What is reduced to cells** and filled as ever, so that today's viewer still starts: `elev` (the cell's mean height over 1,500 m, as a byte: 5.9 m a step), `terrain`, `river`, `nav`, `down`.

**What the record carries** (`landcells::static_arrays`, `static_header`): all of the above. By the example's count, an island adds 4.17 to 4.29 MB to `run.bin`: the five fine arrays 1.77 MB, the lines 1.39 to 1.51 MB, the cell fields 1.01 MB (half of that the twelve months of runoff); the header's `land` object has the grid's size, `h_ref`, the wind, `drowned`, the lakes and the outlets. A lite pack keeps the header only.

**What the live view carries.** `LiveStatic::land` is a clone of the whole `Landform`, in the same process, so a live game has what a recorded run has.

**What the viewer has of it today.** Nothing. `replay::Replay` reads `elev`, `terrain`, `river`, `down` and never looks for a land array; `live::shell` copies the same four from `LiveStatic` and leaves `land` where it is.

**No way to start a game on it.** A new game is `Game::new`, which plays under `Rules::Game`, and the land is off there; the viewer has no `--set`. A recorded run made with a params file (`land = { enabled = true }`) plays back, which is how every picture here was made. Until the switch is thrown, a live game on the new land would need a constructor in the core that takes the rules with the land on, and its save could not be trusted: a save names its rules and is played again under them, which would be the old island.

**What is not kept and would be wanted** (collected with the rest in section 9): the tideways as lines; the loops that have silted up, for meander scars; the plan's tablelands (where each lies, how thick its cap is, which way it dips); height above the nearest water, which the core works out (`hand`) and drops.

## 2. What was measured

### The core's islands (`land_view_probe`, seeds 1 to 8)

`reports/land-view/core-probe-s1-8.txt`. Least and most over the eight islands unless said.

| | |
|---|---|
| height of the dry land, m | 10% of it below 4 to 6; half below 29 to 43; 90% below 334 to 361; top 897 to 1,074 |
| slope of the dry land (steepest fall to a neighbour) | half below 0.013 to 0.020; 90% below 0.13 to 0.14; 99% below 0.25 to 0.31 |
| share steeper than 0.1, 0.2, 0.35, 0.5 | 14 to 17%, 2.6 to 3.9%, 0.4 to 0.9%, 0.15 to 0.55% |
| flood plain, marsh (share of the dry nodes) | 6.6 to 8.1%, 2.0 to 2.4% |
| river lines | 215 to 242, 65,907 to 71,690 points |
| braided reaches | 0 km on every island |
| tideways | 1,712 to 2,058 nodes of sea inside 468 to 543 land cells, 30 to 57 of them estuary cells; 44 to 52% of those nodes in water one node wide |

Half the island is below 40 m and nearly level. That is real lowland, and it is much lower than the old island as the viewer drew it (below).

The lines by size, island 7:

| mean flow, cubic m a second | km | width of the water, m |
|---|---|---|
| under 1 | 1,074 | 6 to 11 |
| 1 to 5 | 880 | 11 to 25 |
| 5 to 12 (a river cell begins at 5) | 486 | 25 to 38 |
| 12 to 50 (boats from 12) | 416 | 38 to 78 |
| 50 and over | 44 | 79 to 83 |

The widest water on eight islands is 118 m (island 2). Today's viewer draws its rivers 52 to 202 m wide, a lane 44 m and a main road 68 m, and a storey of a house is 41 m.

**Long profiles**, reaches of 1 cubic metre a second or more (1,733 to 1,899 km an island): 154 to 216 km fall 1 in 100 or steeper, 6 to 15 km fall 3 in 100 or steeper, 0 to 2.8 km fall 1 in 10. A break (5 m lost within 250 m of line after a slack reach) happens 0 to 8 times an island. The most lost between two points 40 m apart is 3 to 21 m. So there are rapids on every island and a handful of falls at most, none high.

**The line on the grid.** At the points of those reaches that lie on dry nodes (about 42 thousand an island), the ground a mesh would lay between the nodes stands above the river's water by more than half a metre at 32 to 41% of them, more than 2 m at 17 to 24%, more than 5 m at 9 to 13%, more than 15 m at 2.2 to 3.5%; it lies more than half a metre under the water at 4.9 to 6.3% (*corrected by the critic: it said 5 to 6%*). The core cuts the ground to the water only at the node the line passes through, so a ribbon laid at the river's level on a mesh through the nodes is buried along a fifth of its length. The viewer must still give each river a bed, as it does today.

**Tablelands and canyons.**

| | |
|---|---|
| cap rock at the surface | 173 to 934 square km an island |
| the fall within 1 km of the cap's edge | at the middle 225 to 383 m; 90% under 376 to 473 m; the most 431 to 564 m |
| river (1 cubic m a second or more) within 1 km of the cap | 8 to 53 km |
| ground within 1.5 km above that river's water | at the middle 193 to 362 m; the most 322 to 444 m |

### The ground as drawn (`ISOSTATE_LAND_REPORT`, island 7)

`reports/land-view/viewer-*.txt`. The standard game with six families recorded twice, on the old island and on the new land.

| | old island, today | new land, today's viewer | new land, the probe, relief 3 times | 5 times |
|---|---|---|---|---|
| height of the land as drawn: half below, 90% below, top | 0.35, 1.13, 2.73 | 0.08, 0.42, 1.08 | 0.11, 0.59, 1.41 | 0.14, 0.95, 2.32 |
| slope as drawn: half below, 99% below | 0.08, 0.54 | 0.07, 0.37 | 0.06, 0.75 | 0.10, 1.29 |
| land steeper than 1.15, where the shader paints rock | none | none | 0.31% | 1.5% |
| rivers drawn | 24, 887 km | 27, 888 km | 64, 1,825 km | |
| the ground's mesh | 140 chunks, 574,860 vertices, 18.4 MB | 148, 604,500, 19.3 MB | 140, 576,940, 18.5 MB | |
| `build_ground` | 204 to 225 ms | 207 to 255 ms | 129 to 153 ms | |

The new land drawn by today's viewer has under two fifths of the old island's relief. Drawn from the core's heights at three times it has half, with all of it in the belt and the tablelands and a lowland as level as it really is. At five times the top of the island stands nearly as high as the old one's (2.32 against 2.73).

The probe's own steps: 646,753 vertices laid through the nodes (a Catmull-Rom surface, on 48 threads) in 6 to 8 ms; the 64 rivers of 1 cubic metre a second or more (42,747 points) bedded in 11 to 12 ms, all 236 lines (69,522 points) in 19 to 22 ms. The rest of `build_ground` is the lie of the land (46 to 83 ms), which does not care where the heights came from. `make_slots` is 0.7 to 1.1 s either way (it wanders that much from run to run), the churchyards 0.8 s, the far rivers' texture 26 ms (today's rivers; not measured on the core's).

At sixteen vertices to a cell (`VRES` changed, built, measured and put back): 1,921 by 1,345 vertices laid in 16 ms, rivers bedded in 29 ms, `build_ground` 231 ms, the mesh 2,271,948 vertices and 72.7 MB built in 71 ms, `make_slots` 1.2 s.

The rivers' ribbons are two vertices a point for the water and two for its banks: 171 thousand vertices (6.8 MB) for the rivers of 1 cubic metre a second or more, 278 thousand (11 MB) for every stream, against 111 thousand today. The points are finer than a picture needs.

### Roads against rivers

A road link is drawn between two neighbouring cells that both have a road, as a straight line between their jittered middles (`network::road_segments`); a bridge is built wherever that line crosses a drawn river (`scatter.rs`, with `rivers::crossings`). Island 7:

| year | road links | crossings | of them within 6 km along the river of the one before | links across a river that would keep the same roads joined (*corrected by the critic: the header said "crossings"; the report counts links, and a link can cross more than one river*) |
|---|---|---|---|---|
| **old island, today's rivers** | | | | |
| 1800 | 1,430 | 226 | 178 | 102 |
| 1850 | 2,880 | 319 | 274 | 65 |
| 1900 | 4,802 | 370 | 331 | 42 |
| **new land, today's invented rivers** | | | | |
| 1900 | 4,803 | 272 | 235 | 33 |
| **new land, the core's rivers of 1 cubic m a second or more** | | | | |
| 1800 | 1,486 | 404 | 312 | 175 (153 if a brook under 15 m is forded) |
| 1850 | 2,661 | 492 | 369 | 164 (130) |
| 1900 | 4,803 | 643 | 527 | 126 (95) |
| **new land, every stream of the core** | | | | |
| 1900 | 4,803 | 972 | 732 | 206 (95) |

By the water's width in 1900, rivers of 1 cubic metre a second or more: 85 crossings under 15 m, 215 of 15 to 30 m, 299 of 30 to 60 m, 44 wider. The last column is a floor, not a design: it counts the links across a river needed for every road cell to reach the cells it reaches today, however far round, and it falls as the roads thicken because there is always a way round a river's head.

## 3. The pictures

All of island 7 in 1850 unless said, 1600 by 1000, tilt-shift off. Under `.claude/night/shots/land-view/`, on the machine only. Every picture described here was opened and looked at; of the core's eight, islands 2 and 7 and the zoom were.

**The core's own pictures** (`core/land-1.png` to `land-8.png`, `core/land-7-zoom.png`). Island 2 has a belt down its middle, two tablelands (one by the north coast, one by the sea in the south-east) and three small upland lakes. Island 7 has a belt from north-east to south, a tableland north-west of the middle with canyons eaten into its south-west edge, a lowland lake of 86 square km near the west coast and five upland lakes in the belt's valleys, and drowned valleys all round the coast. The zoom (24 cells across, about the lowland lake) shows what the meanders are at this width: long trains of small, even loops, a zip more than a river, with the lake's shore as squares of 500 m. The squares are the picture's and not the data's; the evenness is the model's.

**Today's viewer on the new land** (`today/`):

- `island-1850.png`: the island from far off. Rivers and woods, and no relief to speak of.
- `melness-1850.png`: Melness, the first town, which the core put at the head of an estuary. The viewer has drawn its own estuary, a funnel a cell and a half wide, through the town: the church stands in the water, the market place is under it, and four roads cross on causeways.
- `lake-1850.png`: the lowland lake drawn as sea, with sand, a boat, surf and roads on causeways across its corners. Its level is 4.5 m, so sea level is nearly right here and nowhere else.
- `tarn-1850.png`: an upland lake of 12 square km at 95 m, drawn as two sandy pits to the sea's level with bare beige parcels round them and a stub of invented river beside.
- `tableland-1850.png`: the tableland as a low green swell with woods on it. No edge, no top.
- `belt-1850.png`: the belt of 900 m as rolling downland, the railway on viaducts over hollows.

**The probe, relief three times** (`probe-e3/`):

- `island-1850.png`: the same view. The belt with its valleys, the tableland, four lakes and the rivers as threads down real valleys. It reads as a landscape.
- `melness-1850.png`: the town on a branching tideway, houses on both banks, the church by the water, roads and the railway crossing the inlet. This is what the core meant by a port at an estuary's head. The inlet's edge wobbles where it is one node wide.
- `glenwick-1850.png`: the largest town on a shoaling bay with headlands, boats off it, and a river coming through the town in loops so tight they tangle.
- `lake-1850.png`: the lowland lake with a shore that follows the ground (not squares: the shore is where the ground comes up through the sheet of water), the town at its outlet, the river running in as a frill of small loops. A strip of the west shore is sand-coloured: those parcels belong to cells that are water in the record.
- `tarn-1850.png`: a ribbon lake lying in a wooded valley. Trees stand in the water at its edge and a farmstead on the shore: nothing knows the lake is there. The beige parcels are still there on the north shore (the cell is water to the record, so its parcels are sea).
- `tableland-1850.png`: a plateau with a steep wooded scarp along its east side and grey rock where it is steepest. The woods took the scarp by themselves: the lie of the land ranks steep ground as wild.
- `canyon-1850.png`: from the tableland's top looking over its south-west edge: combes eaten back into the rim, wooded, with patches of rock. Steep valleys, not yet gorges.
- `belt-1850.png`: ridges and valleys, woods on the valley sides, and the railway climbing over spurs on viaducts.
- `hills-close-1850.png`: a hill valley from play distance: a stream in the valley's bottom, fields on the slopes, woods where it steepens. The hillsides are broad and plain. This is the "smooth lumps" view, and it is better than feared: the valleys are real ones.
- `meanders-1850.png`: a river of some 40 m from close by: a clean line of even loops no wider than the lane beside it. A road runs straight across the train of loops, over bridges too small to make out. The fields take no notice of the river.

**Relief and resolution:**

- `probe-e3-detail/`: the same with relief added below the fine grid's scale in proportion to the slope (up to 45 m). At eight vertices to a cell it cannot be told from `probe-e3/`.
- `probe-e3-detail-vres16/`: sixteen vertices to a cell with the same relief. The valley's bed is narrower, rock shows a little more on the canyon's walls, a stream's bed can be made out on the tableland's top. A modest gain for four times the mesh.
- `probe-e5/`: relief five times. `island-1850.png`: the belt stands out with snow along its crest. `belt-1850.png`: mountains, with the summer snow on the highest ridge (the shader's line at 1.75 is reached) and the railway absurd on its viaducts. `tableland-1850.png`: the plateau with grey crags round its combes and a dark wooded scarp, a railway climbing onto it by a viaduct; of all the pictures this is the one that says "plateau". `canyon-1850.png`: the combes now have rock walls and read as a scarp with crags.
- `probe-seed2-e4/` (island 2 in 1800, relief four times): `tableland-se-1800.png` is the picture to look at for canyons: a plateau by the sea with two rock-walled gorges cut into it and a river in each. `tableland-se-back-1800.png` shows a fault: streams run down the canyon's wall as tilted ribbons where they should fall. `tableland-nw-1800.png`: the northern tableland from above, a level farmed top ringed with dark woods on its scarp, a stream crossing it to the rim.

**Rivers:**

- `probe-e3-streams/play-1850.png`: Mallowbrook from the game's opening distance, every stream drawn at the core's width. Thin, clear, and right for a diorama.
- `probe-e3-wide/`: the water drawn two and a half times the core's width. From close by it is a proper river, and where the loops are tight they run into each other; from far off the lake's river is a ruffle, and at Glenwick two rivers come through the town as tangles.
- `core-wide/land-7-zoom.png` and `probe-e3-corewide/` (the core's width constant at 24 in place of 11): the same river in a few long, uneven loops. `probe-e3-corewide/play-1850.png` is Mallowbrook at a confluence with rivers sweeping round the town, and `lake-1850.png` Sandmere on its lake with the river winding through. These are the best pictures of water here. The smaller streams still run in even trains. `probe-e3-corewide/meanders-1850.png` is her twentieth note in one picture: a river wide enough for its bridges to be seen, and a road beside it that crosses it three times in 3 km, twice within a few hundred metres.

## 4. What the viewer invents today, and what becomes of it

| what | where | on the new land | keep, feed or replace |
|---|---|---|---|
| Heights from the cell's `elev`, blurred | `world::build_ground_inner`, `land_height` | untrue: the core has the height every 500 m | **replace** with the core's heights (kept for the old island) |
| Relief below a cell: swells, spurs, crests | `world::add_relief` | unnecessary: its wavelengths (0.5 to 2.5 km) are the ones the core now makes for real | **drop** on the new land |
| The coast's raggedness, beaches and cliffs | `world::shape_coast` | untrue: the coast is drowned valleys, and it is in the heights | **drop**; a beach from the real slope at the shore |
| The sea's depth | the sea floor in `build_ground_inner`; `sea.wesl`, `terrain.wesl` read it | the core has depths, on another scale (a tideway is mostly under 1.4 m deep, *corrected by the critic from "5 m"*; the old shelf was hundreds as drawn) | **feed**, through a stretch the sea's colours can read |
| River courses: sine curves beside the cell middles | `rivers::build` | untrue | **replace** with the core's lines |
| River widths from the `river` byte | `rivers::half_width` | untrue | **feed** from the core's width |
| The water's level along a river | `rivers::build` (smoothed ground, forced to fall) | untrue | **feed** from the core's level |
| Valleys cut to the river, flood plains about the meander belt | `rivers::carve` | mostly unnecessary: the valleys and their floors are in the heights. A bed for the water is still needed | **replace** with a narrow bed; the valley floor **fed** from `wet` |
| Estuaries: a funnel at every mouth | `rivers::build` (`coast`), `rivers::carve` | untrue, and it drowns the towns | **drop**; the tideway is the sea's own |
| Lakes | none: a lake cell is sea | | **new**: a sheet of water at each lake's level |
| Bridges wherever a road's line crosses a river | `scatter.rs` (the loop over `road_segments`), `rivers::crossings` | honest to the river, not to the road: 643 of them | **replace** the rule (stage 4) |
| Causeways where a road lies on water | `scatter::causeway` | they carry roads across tideways and lake corners | **replace**: no road over tidewater or a lake |
| Road links between neighbouring road cells, straight | `network::road_segments`, `network::year_nets` | the links were always the viewer's; straight lines now cross real water | **replace** the course, keep the rule that a road cell is joined |
| Carts along straight lines at the ground's height | `trains::road_routes`, `trains::along` | the cart that sails over the river | **replace**: the cart follows the road as drawn, deck and all |
| Wood, heath, pasture and arable by height, slope, flood plain and distance to water | `world::build_land` (the three ranks), `world::year_land` | the rule is right and now reads real ground: woods went to the scarps unasked | **keep**; **feed** it marsh and the lakes |
| Water meadows on the valley floor | `terrain.wesl` (`sp.flood`, the mesh's first texture coordinate) | fed already in the probe | **feed** from `wet` |
| The snow line, the tree line | `world::summer_snow_line`, `world::above_tree_line`, `terrain.wesl` (1.75, twice) | in the viewer's units: no summer snow at three times relief, a cap of it at five | **keep** the rule, in metres |
| Beach, sea floor, "dry enough to build" | `terrain.wesl` (0.022, -0.004); `scatter.rs` (0.05, 0.07, 0.004); `scatter/towns.rs` (0.05, 0.03); `world.rs` (0.02); `boats.rs` (-0.03) | written for a land that began at 0.06: at three times relief 0.05 is 33 m, and about half the island is lower | **keep** behind one conversion first, then in metres |
| Rock on steep ground, contour lines | `terrain.wesl` (slope over 1.15; lines above 0.45) | work, and depend on the relief chosen | **keep**; later paint by the core's rock |
| The far view's river thread | `terrain::river_far` from `WaterIndex` | works unchanged on the core's lines | **keep** |
| The railway's course and level | `course.rs`, `profile.rs`, `rails.rs` | the level rules hold; the course keeps to the cell's middle and so runs over spurs | **keep** the level, **replace** how the course is chosen within its cells |
| Canals (to come) | not drawn | the record has their links | **new**, on the railway's profile with pounds for gradients |
| Churchyards on dry ground | `scatter/towns.rs` (`wet`: height under 0.05 or water within 0.03) | reads the ground; needs the lakes and the shore in metres | **keep** |
| Harbours: a lighthouse where a town's cell touches a sea cell | `scatter/towns.rs` (the neighbours' `terrain == 0`) | untrue: a port on a tideway has no sea cell beside it, and a lake cell counts as sea | **feed** from the core's coast and `land_estuary` |
| Boats: loops of open water off a town on a sea cell | `boats.rs` | no loop fits in a tideway; lake cells pass for sea | **replace**: boats along tideways, on lakes, off open coasts |
| Waterwheels and ponds "where a river runs by" | the works' generator, by the cell's `river` | the cell's `power` is known | **feed** later (water industry) |
| Pits on the land mix's diggings | `scatter.rs` | not the land's concern | keep |

## 5. What the probe settles

- **The route.** `Ground` has three things the rest of the viewer reads: `hgt`, `rivers` (with `water`, the index built from them) and `floodplain`. The probe fills the three from the core and changes nothing else, and the island is drawn. So stage 1 is a second maker of `Ground`, not a new viewer.
- **The mesh.** Eight vertices to a cell is enough. The surface through the nodes is smooth and takes 6 to 8 ms.
- **The shore.** The probe's first conversion raised the land by 0.03 at the shore and then by its height. That left the building line (0.05 as drawn) at 13 m, and the low towns lost nearly all their houses; and the tideways, mostly under 1.4 m deep (*corrected by the critic: it said "5 m deep at most"; see section 1*), were the colour of sand. The conversion as it now stands stretches the first metre or two above the tide to clear 0.05 and stretches the depths the way the sea's shader reads them, and both come right (Melness and Glenwick were taken both ways; only the second set is kept). That stretch is the cheap first answer for stage 1, and the honest one is stage 3.
- **The river's bed.** Vertices within the water's half width and 1.42 vertex steps of the line are brought down to the water. That is a flat bed 700 m wide for the smallest stream at eight vertices to a cell, unseen on a valley floor and too wide in a mountain valley, where the V becomes a U. Sixteen vertices halve it. A real bank needs geometry of the river's own (stage 2).
- **Lakes.** A sheet at the lake's level over the lake's nodes and the ring round them makes a natural shore for nothing. It has no depth, no colour of its own and no edge, and where a neighbour node is lower than the water it must be left out or the sheet hangs over the outlet.
- **What it does not settle:** anything about a live game (it reads recorded runs only), the railways, and how any of this looks in winter.

## 6. The design in stages

Two rules hold for every stage.

- **The old island is drawn by the old code.** Recorded runs without the land, and games under the port's rules, go through today's `build_ground_inner`, `rivers::build` and the rest, unchanged. The gate is `--shots` on the standard game before and after with `--shots-compare`: every shot the same.
- **The new land stays behind the core's switch** until stage 5 is in, so the game a player starts is today's until then. Each stage is judged on recorded runs with the land on: its pictures are no worse than the stage before, and the `--land-shots` set and the views of section 3 are taken at each.

### Stage 0: the land reaches the viewer (a day)

- `replay.rs`: `Replay` gains `land: Option<Arc<Land>>`, read in `parse` from the arrays of `landcells::static_arrays` and the header's `land` object, all or nothing, each array checked for its length as the workings are. `Land` is the viewer's own plain copy (the fine arrays, the lines, the oxbows, the lakes, the cell fields it uses).
- `live.rs`: `shell` fills it from `LiveStatic::land`.
- The three places that build a `Replay` by hand in tests (`overlay.rs`, `world.rs`, `live.rs`) say `None`.
- **From the core:** a way to start a game with the land on before the switch is thrown, for trying the live path: a constructor beside `Game::under` that takes the params. A save from such a game cannot be continued (it would be replayed under the game's rules, on the old island), so it is started with pictures only, which do not save.
- **Cost:** 4.2 MB read once. **Tests:** a run with the land loads and one without has `None`; an array cut short is refused by name; the lines' starts and ends agree. **Risk:** none to the picture.
- The probe's global and its hooks go when this lands.

### Stage 1: the ground from the core's heights (two to three days)

- `world.rs`: `build_ground_inner` chooses between today's maker and a new `ground_from_land`, which is the probe's `landview::ground` made proper: heights laid through the nodes at `VRES` (stay at 8), the rivers of stage 2 not yet (see below), `floodplain` from `wet`. `add_relief`, `shape_coast` and the blurs are not called. `Ground` gains what later stages need and nothing reads yet: the metres at each vertex, the marsh, the lakes.
- **The relief.** One number, how many times the relief is raised, in `assets/look.toml` beside the other choices of the look. Three is the old "about three times" and gives a quiet island; four or five gives mountains and makes the canyons read. I would start at four. It is Stella's to choose by eye and costs nothing to change.
- **The shore.** The conversion from metres to the viewer's height carries the probe's stretch, so that every threshold written for the old shore (the table in section 4) goes on meaning what it meant. One function, used for the ground, the rivers' levels and the lakes.
- **Rivers at this stage.** Stage 1 alone would leave the old sine rivers running over real valleys, which is worse than today. So stage 1 takes the core's lines in their plainest form at once: the probe's conversion to `rivers::River` and its bed, drawn by today's `rivers::ribbons`. Stage 2 is then about drawing water well, not about having it.
- **Lakes at this stage:** the probe's sheet, since a lake as a dry hollow is also worse than today.
- **The sea.** `terrain::spawn_sea` hands the sea's shader the same heights, so tideways are water as soon as the ground is right. Deep-sea chunks are still left out by today's rule.
- **Close views.** No relief is added below the nodes. What a close view gets instead comes in stages 2 (banks), 3 (paint) and 6 (walls).
- **From the core:** nothing more.
- **Cost (measured):** laying the heights 6 to 8 ms; bedding 11 to 22 ms; the mesh as today, 577 thousand vertices and 18.5 MB (chunks of open sea are left out by depth, so the count follows how the sea floor is converted: with the sea floor left shallow it was 666 thousand); `build_ground` 130 to 150 ms against 205 to 255. No new texture. A year's rebuild is untouched.
- **Tests:** on three seeds, every land cell's mean height as drawn agrees with the cell's `elev` through the conversion to within a step or two of the byte, so the inspector and the picture agree; no vertex in a dry node is under the sea and none in a sea node above it by more than the surface's overshoot; every river point's ground is at or under its water; `ISOSTATE_AREAS=1` keeps the shares of arable, pasture, wood and heath as closely as on the old island.
- **Pictures:** the ten views of `probe-e3/`, and `--land-shots`.
- **Risk:** the thresholds (met by the stretch); the railways look worse on real relief until stage 5 (`probe-e5/belt-1850.png`), which is the main reason the switch stays off.

### Stage 2: water (four to five days; 2a and 2b can be two helpers)

**2a, rivers** (`rivers.rs`, `network.rs`, `terrain.rs`):

- `River` gains the flow and, when the core sends them, the tidal and braided marks. The lines are thinned to what the bend needs (a point where the line has turned or the width changed enough), which should halve 42,747 points or better.
- **Width.** Drawn at the core's width, with a floor of about 12 m so that a brook is still a line, and nothing wider. Drawing wider than the core's makes the loops collide (`probe-e3-wide/`), because the loops are scaled to the core's width. If the rivers are to be bolder, the core's constant is the place (section 9); the viewer follows.
- **Banks.** The river carries its own strip of ground: from the water's edge a bank of a metre or two and a berm out to where the ground mesh takes over, a vertex step away. The strip is drawn with the ground's own material, which paints by position, so the fields carry on across it; the ground under it is bedded as in the probe and hidden. This is what gives a river an edge at 250 m vertices, and it is the one new mesh trick here. Try it on one river in the lab's light before building it for all.
- **Bars and scars.** A pale bar on the inside of a bend where the line's curvature against its width is high (the viewer has the curvature). Oxbows from the core's `oxbows` as crescents of still water with reeds. Scars of older loops need the core to keep the loops it now drops.
- **Rapids and falls.** A reach falling 3 in 100 or more is white water (6 to 15 km an island). A fall is drawn where the ribbon as laid would be steeper than about one in three as drawn, or where the profile breaks: a step in the ribbon with a pool under it. There are few (0 to 8 breaks an island), and the streams that come over a tableland's lip are small; the picture of them as tilted ribbons (`probe-seed2-e4/tableland-se-back-1800.png`) is what this mends.
- **The far thread.** `terrain::river_far` unchanged, from the new lines. Its texture stays 1,920 by 1,344 in two bytes, 5.2 MB. At play distance the smallest streams are under a pixel wide, so the thread should begin a little nearer than it does today.
- **Honest to discharge** means the width is the core's, the white water is the core's fall times nothing invented, and a river is never drawn where the core has less than its smallest stream. The viewer chooses the least flow it draws (1 cubic metre a second in the probe; every stream at play distance looked right too).

**2b, lakes and tideways** (`terrain.rs`, `world.rs`, a little of `sea.wesl`):

- **Lakes.** A mesh for each lake cut along the line where the ground meets its level (marching squares over the vertices in and round the lake), drawn with the sea's material told that it is a lake: depth from the bed for its colour, no surf, no tide line. Its shore joins `world::water_dist`, so trees, houses and yards keep out of it. The river out of a lake starts at the outlet the core names.
- **Tideways.** The sea's surface already fills them. What is missing is that half their nodes lie in water one node wide, which a surface through the nodes pinches and sometimes breaks. They need bedding as the rivers are bedded, and for that they need a line: the core should carry each river's line on down its drowned valley to open water (section 9). Until it does, the viewer can trace one through the sea's nodes inside the land from each river's end. The margins between a metre under and a metre over the tide are mud and saltmarsh in the ground's paint.
- **The sea's colours.** The depth the sea's shader reads is stretched from metres once, in the conversion, as in the probe. The look's sea colours may want another pass against it.
- **From the core:** the width constant as a number of the land; the tideways as lines; the silted loops; `braided` either made to happen or dropped (it is false at every point of eight islands).
- **Cost:** ribbons 171 thousand vertices before thinning (6.8 MB), with banks perhaps twice that; lakes a few thousand vertices; all built once. Nothing a year.
- **Tests:** no river point's water is under its bed; a river's water never rises downstream as drawn; every lake's mesh lies inside its nodes and their ring; no tree, house or works slot inside a lake or a tideway; the far texture has a thread within a texel of every line.
- **Pictures:** `meanders`, `lake`, `tarn`, `melness`, `glenwick`, the two tablelands of island 2, each close and at play distance, in summer and in midwinter (the rivers go to steel).
- **Risk:** the bank strip is new; the regular meander trains are the model's and no drawing hides them.

### Stage 3: the land's classes on real hydrology, and heights in metres (three days)

- `world::build_land`: the ranks read what they read now, from the real ground. Add to `Fields` the marsh (from `wet`) and the distance to a lake's shore. Marsh is the last ground to be ploughed and the first to be rough pasture or carr: it goes into `wild` and into `lea`. The flood plain is `wet` and already makes meadow.
- `terrain.wesl`, `terrain.rs`: the mesh's second texture coordinate carries marsh (the first carries the flood plain), so the shader paints fen and marsh (reed, dark pools, peat) over pasture and heath, and alder and willow stand where a marsh parcel is wood. No new class in the record: marsh is a tint the fine grid knows, on land whose use the record gives.
- **Heights in metres.** The shore's stretch of stage 1 comes out. Each threshold of section 4's table becomes a height in metres or a distance from water, read through one conversion: built ground at least a metre above the tide or the river's water beside it, the beach by the shore's real slope, the snow by metres. The shader gets the relief's factor as a uniform so that `1.75` becomes a height.
- **Snow and trees.** In metres: no summer snow under about 1,200 m, which is none on these islands; winter snow down to a few hundred metres; the tree line where the core's woodland potential stops (760 m).
- **Woods on valley sides** need nothing: it is what the ranks do with real slopes (`probe-e3/tableland-1850.png`).
- **A map view of the watersheds** falls out here for almost nothing: `overlay.rs` tints by `land_basin` and rules ink at the divides, as the markets view does. Rain and water power are two more.
- **From the core:** nothing. Later, the four open shares a cell that the viewer's README already asks for.
- **Cost:** nothing measurable; the ranks take 46 to 83 ms as now.
- **Tests:** `ISOSTATE_AREAS=1` as before; of the parcels on flood plain, the share under grass is higher than off it; no arable parcel is mostly marsh while a drier one in its district is grass; the class-map check against the shader still agrees.
- **Risk:** small. The conversion touches many lines in `scatter.rs` and `towns.rs`, each a one-line change with a picture to check it.

### Stage 4: roads, bridges, fords and carts (four to five days)

**What is a fact today.** A cell has a road of a level (track, road, turnpike), raised by the traffic through it and lowered after six quiet years (`sim.rs`, `update_roads`). The carting graph joins every cell to its eight neighbours; a river never costs a cart anything. So the links are the viewer's and every bridge is the viewer's. That is also the room there is: the viewer may choose links that need few bridges, and draw the road where a road would go.

**What the viewer may honestly draw until the simulation knows more:**

- **Each cell's place**, where its roads meet, is today's jittered middle moved onto dry ground a little back from any water. Where a river runs through the cell, the place is on one bank, the same bank for the cells along a reach (the town's bank where a town stands on it), so a road down a valley keeps to one side.
- **Bridge sites are found once, from the ground.** Along every river of 15 m or more: at each town on it; at the head of each tideway; and between those, at the firmest narrow places (the flood plain narrow, no marsh), no closer together than the river's size allows (some 4 km on a river of 15 to 30 m, 8 km on one of 30 to 60 m, 12 km on a wider one). A site belongs to the island, not to a year, so a bridge never moves.
- **A link that crosses** a river of 15 m or more is drawn only by way of a site: its course bends to the bridge. If no site is within reach it is not drawn, and its two cells are joined round by the links that do not cross. The measurement says this loses no cell: 95 to 153 links across a river keep every road cell joined (*corrected by the critic: it said "crossings"; and see "What the critic found" on why the measurement does not say this*).
- **A brook** (under 15 m) is forded by a lane and has a single small arch under a main road.
- **Tidewater below the head, and lakes,** are not crossed.
- **A bridge stands** from the first year a road uses its site. Its size is the river's width there and the road's level (`isostate_gen::bridge::RoadBridge` takes both already).
- **The road as drawn** is a polyline with its heights: the ground's, and the deck's on a bridge. The ribbon, the carts (`trains::road_routes`, `carts.rs`), the farm lanes (`scatter::lane`, `RoadIndex`) and the towns' streets (`scatter/towns.rs`) all read that one polyline. The cart then crosses on its bridge because it has nothing else to follow.

- **Files:** `network.rs` (a `RoadNet` for the year: places, links, courses, crossings), `scatter.rs` (bridges and fords from it; `causeway` goes on the new land), `trains.rs` and `carts.rs`, `scatter/towns.rs` (streets), `course.rs` (the railway keeps clear of the place as now).
- **For a bridge to be a fact,** the simulation would have to know:
  1. **Roads as links.** The traffic along each link of the carting graph is known when the flows are assigned; a road's level belongs to the link. The record already carries canals as links with their years, which is the pattern.
  2. **Which links cross water, and how much.** A static array made with the land: for each cell and each of its eight neighbours, the largest flow the way between them crosses (none along a bank).
  3. **What it costs to cross without a bridge:** nothing at a ford under some flow, a ferry's delay above it. Then a bridge is worth something.
  4. **A bridge as a work someone pays for,** dearer by the width, in stone and timber and later iron, through `ways.rs` as the canals and railways go. This is the backlog's "roads and bridges that cost" (Ready next 12), and it waits for the public purse.
  5. **The record:** the bridges as links with the year each opened. The viewer then draws exactly those and fords the rest.
- **Cost:** the year's networks take 12 to 16 ms today off the main thread; the sites are found once at start.
- **Tests:** `ISOSTATE_LAND_REPORT`'s line becomes a test: in 1900 no more bridges over rivers of 15 m or more than sites, none within its river's spacing of another, and every road cell still reaches every cell it reached; along every cart's path, the height over water is a deck's; no road ribbon lies on a lake or on tidewater.
- **Pictures:** `meanders-1850.png` (the road through three bends), Melness and Mallowbrook at play distance, a valley road in the belt.
- **Risk:** the middle one. Roads that bend to bridges change the look of every district and the towns' streets hang on them. Doing it on today's rivers first would be wasted, as the backlog says.

### Stage 5: what else has to follow (four to six days, in separable pieces)

- **Railways** (`course.rs`, `profile.rs`, `rails.rs`). The simulation chooses a line's cells by the cells' heights; the viewer's course passes each cell near its middle. On real relief that is over the spur as often as round it (`probe-e5/belt-1850.png`). The level rules stand. The course needs to choose where in each cell it passes by the ground: shift the passing point across the cell to where the ground lies nearest the level the line will have, and keep to a valley's side above its floor. There are still no cuttings, since the ground is built once; a cutting would mean a chunk of ground rebuilt in the year a line opens (4,225 vertices, 135 kB), which the smooth hand-over could carry, and is worth a look after this.
- **Canals.** The record has their links. A canal is a line on the railway's machinery with a ruling gradient of nothing and its climbs gathered into locks, keeping to the contour and the valley's side, joining a river where its link runs along navigable water. A third of a run's canal cells are river cells, so "a river made navigable" and "a canal beside it" must be told apart by the drawing: where the link follows a navigable river, draw the river with a towpath and a lock at each weir, not a second channel.
- **Towns and works on slopes.** Houses stand on the ground as it lies already. The market place and the churchyard want level ground within their quarter and should look for it; a works' yard is levelled by its own wall. Quarry towns on the scarps are where this will show.
- **Harbours and boats** (`scatter/towns.rs`, `boats.rs`). A port is a town on the core's coast, which on the new land includes a tideway's head (`land_estuary`) and a lake's shore (`land_shore`). A quay along the bank nearest the market; a lighthouse only on the open sea. Boats move along the tideway's line and on the lake's mesh.
- **The inspector** can say what the ground is: the river's flow, the lake, marsh, the basin.
- **Then the switch.** With this in, `game_params()` takes the land (the core's page says how, and which tests to re-anchor), the standard shots are retaken as the new baseline, and the old island lives on for the port's rules and for recordings.

### Stage 6: scarps, canyons and plateaus made to read (two to three days)

Section 7 has the reasoning. The work:

- `terrain.wesl`, `terrain.rs`: a texture of the core's `rock` (480 by 336, one byte, 161 kB). Where cap rock is at the surface and the ground steep, a continuous crag in the cap's own colour with scree below, instead of grey wherever the slope passes a number.
- `world.rs` (`ground_from_land`): on the cap's edge the heights are sharpened: the top held level to the rim, then a wall the thickness of the cap, then the softer slope. The model says this is what is there (hard rock 90 to 150 m thick on soft, cleared from below); the 500 m grid smears it over a node.
- **From the core:** the plan's tablelands in the header (where, how thick the cap, the dip), so the wall's height is the cap's and not a guess.
- **Cost:** nothing measurable. **Pictures:** the two tablelands of island 2 and island 7's, from the foot and from the top. **Risk:** taste; easy to overdo.

## 7. Canyons, plateaus and scarps

**Does the generator make them?** Yes, on every island of eight: one or two tablelands of 173 to 934 square km, flat on top, with an edge that falls 225 to 383 m within a kilometre at the middle and up to 564 m, and rivers that leave them in canyons 190 to 360 m deep (8 to 53 km of such river an island). Elsewhere in the belt there are thousands of river points more than 150 m below the ground within 1.5 km: deep valleys are the rule in the uplands.

**Do they survive to the picture?** The plateau and the scarp do, at any relief (`probe-e3/tableland-1850.png`). The canyon does in part. A canyon in a 500 m grid is at least two nodes from rim to rim, so 300 m deep and 1 km wide: a slope of about 0.6, which at three times relief is steep ground with patches of rock, and reads as a combe. At four or five times the walls pass the shader's rock line along their length and it reads as a gorge (`probe-seed2-e4/tableland-se-1800.png`, where the tableland stands by the sea and the canyons reach the tide).

**What would make them read:** the relief at four or more; rock painted by the core's rock, not only by slope; the cap's wall sharpened as in stage 6; a river in the bottom with white water; a fall where a stream comes over the lip; and the woods that already hang on the scarps. The lowland's banded ridges and vales do not show and will not: the core's page says they come out as a few metres.

## 8. What should wait for a water cycle, and what the picture can carry now

**Wait for the simulation:**

- **Floods.** A flood is a year or a month when a river carries more than its banks hold. `discharge_with` will say so once something drives it (the harvest's wet and dry years are the obvious driver), and the record would carry a flow for each cell and year. The picture then lays water over the flood plain's nodes for that season. Drawing floods before then would be weather the simulation never had.
- **Mill ponds, weirs and leats.** The works' generator draws a wheel and a pond where a river runs by. For that to be a fact a works must draw on its cell's `power`.
- **Dams and reservoirs.** A work at a lake's outlet or across a valley, through `ways.rs`. The picture is ready for it in kind: a reservoir is a lake's mesh at a new level, cut along a contour by the same function as stage 2b, and lakes are not part of the ground.
- **Drainage and irrigation.** Marsh won for the plough changes `wet` by year; the record would carry the drained cells and the shader rule straight dykes across them.
- **Navigations.** A river made navigable with locks, as distinct from a canal.

**The picture can carry already:**

- The whole average water: every stream by its flow, lakes, tideways, marsh and flood plain, rapids and falls.
- **The regime.** Runoff by month is in the record, and on island 7 the largest river runs 1.3 times higher in January than in July. That can show as the season's own look on what is always there: meadows on the flood plain greener and wet in winter, marsh pools fuller, rivers already steel. It is the average year, said to be so.
- **Watersheds**, as a map view (stage 3), and rain and water power beside it. This answers the first word of her note without waiting for any industry.
- **Where the water industry will stand:** the lake outlets, the falls and the steep reaches are the sites, and they will be visible.

## 9. What the core should add

None of it is needed for stages 0 and 1.

1. **The river width's constant as one of the land's numbers** (it is `11.0` in `landwater::river_width`). At 24 the rivers and their bends are on the map's scale (`probe-e3-corewide/`). It is not free: the ground under a river is cut to its course, so the island's cells and the run's history change a little (the recorded run with 24 has 1,403 road links in 1800 where the other has 1,486), and the land's sweeps would be run again.
2. **Meanders less even.** The long trains of equal loops on rivers of 1 to 12 cubic metres a second are the first thing a geographer would point at. A bank that is not equally soft everywhere would break them up. This is the core's to judge; the viewer cannot hide it.
3. **Tideways as lines:** each river's line carried on down its drowned valley to open water, with the inlet's width and a tidal mark.
4. **The silted loops** kept beside the oxbows, for meander scars.
5. **The plan's tablelands** in the header (place, extent, cap thickness, dip).
6. **`braided`** is false at every point of eight islands: make it happen or drop it.
7. **For the live path before the switch:** a `Game` constructor that takes the rules with the land on.
8. **For bridges as facts** (stage 4's list): roads as links, what each link crosses, a cost to cross, a bridge as a work, the bridges in the record.

## 10. Unsure

- **How it looks in winter and in every year.** Every picture here is the summer of 1850 or 1800. Snow in metres, frost on marsh and the bare woods on scarps were not looked at.
- **The live game.** Nothing was run live on the new land, because nothing can be. Stage 0's reading of `LiveStatic::land` is from the code.
- **One island and a half.** The viewer's pictures are island 7, with four of island 2. The core's numbers are eight islands and agree closely, but an island with the 200 square km lake, or one whose first town is on a lake's outlet, was not drawn.
- **The relief's factor.** Three, four and five were looked at on two islands. Four is a guess between a quiet picture and a relief model; it interacts with the rock line, the snow and how steep a street may be.
- **The bank strip** of stage 2 is reasoned, not tried. If it fights the ground at its outer edge, the fallback is today's flat valley floor, which the probe shows is acceptable in the lowland and too wide in the hills.
- **The bridge sites' spacing** (4, 8, 12 km) is a first guess. The measured floor (95 to 153 links across a river to keep the roads joined; *corrected by the critic: it said "crossings"*) says only that far fewer than 643 will do.
- **The count of crossings needed** treats two cells as joined if any chain of links joins them. A real carter would not go 30 km round; a rule that weighed the detour would keep more bridges.
- **Timings** were taken with other helpers building. The ground's steps are tens of milliseconds and the conclusion (it costs nothing that matters) does not hang on them; `make_slots` at sixteen vertices (1.2 s against 0.7 to 1.1) is within what the noise could be.
- **The railways** I looked at in two pictures and did not measure. "The course should choose where in its cell it passes" is where I would begin, not a design.
- **The width experiment** changed a constant in a temporary edit, recorded one run and was put back. It shows the look. It does not show what the wider rivers do to the land's sweeps.

## What the critic found

Branch `night/o-land-view-critic`, cut from `night/dx-land-view` and merged with `main` at 295a379 (12 November 2026; the merge had no conflicts). The critic changed no code but one measuring example, `crates/isostate-core/examples/land_view_critic.rs`, which makes the islands and prints what the design leans on and the probe did not count. Results are on the machine only, under `.claude/night/reports/opus/land-view-critic/` (with a file of notes, `build.md`, that has every command), the recorded runs under `.claude/night/tmp/opus/land-view-critic/`. The machine was shared with a dozen other helpers, so no timing here is better than a factor of one and a half; none of the conclusions hangs on one.

### The numbers, run again

- **The core's.** `cargo run --release -p isostate-core --example land_view_probe -- 1-8` on the merged branch prints, byte for byte, the file the probe kept. So every figure of sections 1, 2 and 7 that comes from it stands, and nothing merged into `main` since the probe was cut has moved the land. Read against that file, four statements of the page were wrong and are corrected in place, each marked: the length of river an island (2,760 to 2,930 km, not 2,800 to 2,900); the share of a river's line that a mesh lays above its water by more than half a metre the wrong way (4.9 to 6.3%, not 5 to 6%); the road table's last column and the sentences that quote it, which count links across a river and not crossings; and the depth of a tideway, which is not "5 m at most": half the tideway's nodes are under 1.2 to 1.4 m of water, nine tenths under 3.1 to 3.8 m, and the deepest under 16 to 18 m.
- **The viewer's.** Island 7 recorded afresh on the merged build, on the new land and on the old (game rules, six families), and read with `ISOSTATE_LAND_REPORT=1 ISOSTATE_TIMING=1 isostate-viewer RUN --describe 1,1`, with and without `ISOSTATE_LAND_VIEW=1`: the heights and slopes as drawn, the rock, the ground's mesh, the rivers drawn, and every road link and crossing of 1800, 1850 and 1900 are the probe's to the digit. The probe's timings could not be checked: the standard shots were running beside these, and the ground took seconds where the probe saw a fifth of one.
- **The old island's picture.** With the probe compiled in and nothing set, `isostate-viewer --shots ... --shots-compare .../merged-15-town-ground` gives 0 pixels of difference in all twelve standard pictures. The design's first rule holds for the probe's way of hooking in, which is the way stage 1 would hook in.

### What the probe did not measure

`cargo run --release -p isostate-core --example land_view_critic -- 1-8`, least and most over the eight islands; the railways, the works and the year's rebuild on island 7 in the dev viewer.

| | |
|---|---|
| the shore as the probe's conversion draws it, at 3x: slope as drawn from a sea node to the dry node beside it | half 0.36 to 0.41; over 0.5, where the ground shader turns a beach to shingle, 16 to 24% of tideway banks and 20 to 29% of open coast; over 1.15, rock, 1 to 3% and 4 to 8%. The same shores without the stretch: half 0.015 to 0.018 |
| what lies under the viewer's cell middles (`network::centres`: where roads meet, the market square stands, the railway passes) | tidewater 1.4 to 1.9% of land cells, a lake up to 0.1%, marsh 1.5 to 2.2%, flood plain 6.2 to 8.0%; ground steeper than 1 in 10 at 18.5 to 22.3%, than 1 in 5 at 4.6 to 6.7% |
| the 48 first towns of eight islands: what their market square stands on | tidewater 9, a lake 2 (island 3's two lake-outlet towns), marsh 8, flood plain 19, dry ground 10. Island 7: Melness, Glenwick and Granness on tidewater |
| the 19 towns of island 7's run (`towns.py` in the notes) | the six of 1749 in cells of 1 to 15 m of relief, on the water. Of the thirteen founded later, nine stand in cells of 137 to 235 m of relief with their market square on ground falling 1 in 8 to 1 in 2.6 (0.38 to 1.17 as drawn at 3x; Carford's is past the shader's rock line) |
| the relief's factor against the viewer's fixed heights, 3x / 4x / 5x | the field system's "hill" (0.30 as drawn) begins at 200 / 150 / 120 m, which is 17 to 20% / 20 to 24% / 22 to 27% of the dry land (on the old island more than half the land is drawn above 0.30); summer snow covers none / 0.03 to 0.37% / 0.65 to 1.46%; the railways' ruling gradient of 0.085 is 1 in 35 / 47 / 59; a line goes into tunnel 67 / 50 / 40 m under the ground |
| the railways of 1900, island 7 (`ISOSTATE_RAIL_CHECK=1`) | old island: 92 viaducts over 45.5 cells, 30 tunnels, 29 inclines. New land, today's viewer: 51, 9, 19. Through the probe at 3x: 204 viaducts over 129.7 cells, 47 tunnels, 61 inclines, a line standing 0.391 (260 m) over the ground. At 5x: 291 viaducts over 199 cells, 83 tunnels, 134 inclines, and a step of 0.252 along a line, 100 m, the fault of her note 12 |
| works drawn smaller than they are for want of room, island 7 (`ISOSTATE_TIMING`) | old island 521 of 2,320 (22%); new land under today's viewer 933 of 2,122 (44%), 82 with no ground of their own; through the probe 816 of 2,122 (38%), none without ground |
| bridge sites at the probe's spacing (4, 8, 12 km on rivers of 15 to 30, 30 to 60, 60 m and more) | at most 250 to 278 an island, against 95 to 153 links that must cross and 558 crossings of such rivers by island 7's roads of 1900 |
| lakes | 2 to 52 cells are water to the record and hold 1 to 18 square km of dry shore, which the viewer draws as sea; 14 to 68% of all lake nodes lie in cells the record calls land (47 to 68% on islands with no lowland lake) |
| a year's rebuild (`ISOSTATE_BENCH=1`) | the same time on the old island, on the new land under today's viewer and through the probe, within the noise (1.2 to 2.2 s a year, dev build, machine loaded); every stream drawn perhaps a quarter to a half more. The year's scenery is 9% more vertices than the old island's in 1760 (11.1 against 10.1 million) and 2% fewer in 1900 |

### Stage by stage

**Stage 0** is sound, small and safe, and can land alone. Three things to add. The land should hang off the replay in an `Arc` of its own (the design says so) and stay out of what `game::pump` copies whole when a year arrives while a scenery build holds the run, which is 17 MB by 1900 already. A lite pack of a run made on the new land carries the header and not the arrays, and would be drawn as today's invented island without a word: it should say so, as the probe's own reader does. And the core's constructor for a game with the land on is the only way any stage can be seen in a live game, and the only way `--shots` can take the new land: ask for it here, not later.

**Stage 1** is the right first step, can land alone, and on the evidence of the probe changes nothing in the old island's twelve pictures. What it underestimates:

- **The middles.** The design leaves the cells' meeting places as they are until stage 4. But the market square, the church's quarter, every high street, the roads, the farm lanes and the railway's passing point all hang on `network::centres`, and on the new land a fifth of the first towns have their market square on tidewater or in a lake and another sixth on marsh. A town that stands at the head of its estuary is drawn with its square in the estuary. Moving each middle to the nearest dry, level spot of its cell (off tidewater, lakes and marsh, back from a river) is a few lines in one function, used by everything, and belongs in stage 1. Which bank of a river, and keeping to one bank down a reach, can wait for stage 4.
- **The lakes.** A sheet of water with nothing keeping out of it puts trees, farmsteads and fields in every upland lake (`probe-e3/tarn-1850.png`), and its nodes count as dry ground when `world::build_land` shares out a district's arable and wood (`Land::area` counts what stands above 0.02). Stage 1 ships the sheet, so it must ship the mask with it: the lake's nodes are water to `Land::area`, to `water_dist` and to every "dry enough" test. And the cells the record calls water hold 1 to 18 square km of shore that the viewer paints as the sea's beige: their parcels want the land's classes from their neighbours.
- **The shore's stretch.** It works, it is one function, and it keeps every threshold of section 4 meaning what it meant. Its price is that every shore is drawn as a bank some twenty times steeper than the core's heights make it: a fifth of tideway banks pass the shader's shingle line. The tideways are where the first towns stand, so this is their waterfront. Keep the stretch in stage 1 (the alternative, the thresholds in metres, is a change to files three other branches are rewriting) but have the shader leave out the shingle on the new land's shore below a metre or two, so that the banks read as mud and sand.
- **The relief's factor is not free.** The design calls it "Stella's to choose by eye" and "costs nothing to change". It is in the viewer's units everywhere: the field system's hill and margin (`build_land`, 0.22 to 0.55), the conifers (0.34 to 0.8), the summer snow (1.75, in the shader twice and in `world.rs` twice), the rock (slope 1.15), the contour lines (0.45), the frost (0.12 times the height), and the railways' every rule (`profile.rs`: ruling gradient, tunnel, how high a line may stand). At 3x the railway's ruling gradient is 1 in 35, at 5x 1 in 59, and at 5x the profile breaks. Stage 1 should fix the factor at 3, the "about three times" all those numbers were tuned at, and leave it there until the design's stage 3 has put them into metres. Canyons that need 4x or 5x to read are stage 6's to make read by their walls and their rock, not by raising the whole island.
- **The cost.** Laying and bedding are cheap, as measured. A year's rebuild is not touched in time; it is in what it builds, up to a tenth more vertices in the early years, which is up to a tenth more of what dx-smooth's hand-over has to carry. The `WaterIndex` holds twice today's length of river (1,825 km against 887) and three times if every stream is drawn, and every house plot, lane and road link asks it a question every year; at one cubic metre a second that is lost in the noise, every stream may not be.
- **No gate on the new land.** The twelve standard pictures are of the old island, so they guard stage 1 against harming it and say nothing about the new land through five stages. Stage 0's constructor makes `--shots` possible on the new land; each stage should then keep a named set of the probe's views and compare against the stage before.

**Stage 2** (water) is well judged in what it draws; it underestimates where the water meets the rest. The bank strip is a second surface the CPU does not know: `world::height_at` reads the heightfield bedded under it, so whatever stands within a vertex step of a river (a bridge's ramp, a house at the 60 m it keeps from the water, a road draped down to the bed) stands on hidden ground, under the strip, and the strip's height has to be answered on the CPU as well, which is the field system's "written twice" (K26) in a new place. The far thread is not "unchanged": its width is read from the river's half width over 0.06 of a cell, which the old rivers filled from a fifth to five sixths of and the core's fill from 2 to 49 per cent, so every thread is drawn thinner than today's and a river of 80 m only twice as wide as a brook; the width wants the flow's logarithm. A lake drawn with the sea's material does not go to steel in midwinter with the rivers that feed it, the seam the sea already has at every river mouth (L95). And the sea's surface is one sheet at nought over the whole map: the lowland lakes stand 5 or 6 m above the sea and are 16 to 18 m deep, so their beds lie 10 to 13 m below it, and a lake drawn in the sea's blended material would show the sea's own sheet, and its line of foam where the bed crosses nought, through its water.

**Stage 3** is right in kind and much larger than written. The section 4 table names about a dozen heights to put into metres; the code has some thirty in fourteen files: besides the table's, `fences.rs` (0.03), `rails.rs` (0.04), `profile.rs` (every rule), `scatter/towns/kept.rs` (0.05 twice), `world::build_land` (the hill, the margin, the tops), the trees' conifers, the snow line and the tree line in `world.rs` and twice in `terrain.wesl`, the contours, the frost, the far shore's line (0.014), the causeway (0.004, 0.016), the deep chunks (-0.75), the boats (-0.03), the roads' drape (`max(0.0)`), and the plan. Most are in `scatter.rs`, `scatter/towns.rs` and `kept.rs`, which the works' site planner and the towns' second pass are rewriting: stage 3 has to come after both land, and is not "small". The watershed map is not "for almost nothing": the map views wash one texel per cell, so the divides would be ruled along cell edges in squares beside ridges drawn every 250 m; the core keeps the basin only by cell, and the viewer would have to trace the drainage on the fine grid itself to rule them on the ridges (cheap, but not nothing).

**Stage 4** (roads and bridges) has the right analysis and one wrong inference. "95 to 153 links keep every road cell joined" says that a well-chosen set of so many crossings would do; it does not say that sites chosen from the ground alone, at a spacing, will be where those links are. With sites placed first and links that find no site dropped, a road cell can be cut off. The bridges should be chosen from the roads: for the year's network, the fewest crossings that keep it joined with no detour longer than some multiple of the straight way, each then moved to the best site within reach; and the set may only grow, so that a bridge once built stays. In a live game the later roads are not known, as with the works' ground, so the set grows year by year and a road's course can change when a nearer bridge opens; the design must say what may move then (the streets and the churches hang on the roads, and a church moves at most once). The road network for the year is wanted by the churchyards, which `game::pump` works out on the main thread when a year arrives, as well as by the scenery: it should be made once a year in one place, and dx-smooth's stage 2 (the year's arrival off the main thread) is the place. Last: the cart that sails over the river (her note 19) is a straight line at the ground's height on the old island as on the new, and "the cart follows the road as drawn, deck and all" needs none of this. It can be mended now, on today's roads, and should not wait three stages.

**Stage 5** puts first what should come second. The railways are measured above: on the new land at 3x they have more than twice the old island's viaducts and nearly three times their length, and at 5x their profile breaks. That is the aqueduct of her note 12 a hundred times over, it is what `probe-e5/belt-1850.png` shows, and it does not depend on water, classes or roads. It should follow stage 1 at once, with `ISOSTATE_RAIL_CHECK` on the new land as its gate: no more viaduct than on the old island, no step. Towns on slopes are not an edge case: nine of island 7's nineteen towns have their market square on a slope of 1 in 8 or steeper. They are towns founded after 1750, in the hills, where `land.md` finds the new land founds its extra quarry towns. They need a stage of their own, after the towns' second pass, not a bullet. And works: on the new land 38% of works are drawn smaller than they are for want of room, against 22% on the old island; the site planner on `night/works-sites` is replacing that code, and its review should be given the new land's wet and steep ground to place on.

**Stage 6** (scarps and canyons) will paint crags by the core's rock in the shader alone, so the trees, farmsteads and houses placed on the CPU will stand on painted cliffs; whatever decides "rock here" has to be decided on the CPU and handed to the shader, as the land's classes are. A wall sharpened into a heightfield stretches everything the shader paints by position down its face: the probe's own `probe-seed2-e4/tableland-se-back-1800.png` shows the vertical streaks on a scarp at 4x. Walls want the rock painted by the face's own direction, or a strip of their own like the river's banks.

**The order, and landing alone.** Every stage can land alone and keep the old island's pictures, because each is behind the run's land. The weakness is the other way: nothing of the new land is seen by Stella until stage 5 is in, eighteen to twenty-three days of work by the design's own estimates, beside three other branches rewriting the same files. She should be able to look at it before then.

### What it leaves out of her notes

- **Note 2, mines at the foot of the mountain.** Not in the design. The core knows the rock at every node (cap, basement, coal measures) and where the scarps are; a working's pit could stand where its rock comes to the surface at the foot of the slope, not on the first free spot of its cell. It belongs with stage 6's rock, or with F7's drawing of the workings.
- **Note 4, "the meandering feels a little coarse".** The design draws the core's meanders honestly and says rightly that their regularity is the core's. But the long trains of equal loops are the first thing she will see on the new land, and they will be judged there, so the core's item 2 (a bank not equally soft everywhere) and the width constant (item 1) should be decided before the switch, not after. Both move the economy a little and need the land's sweeps again.
- **Note 5, land that looks ruled from far off.** The new relief helps: woods take the scarps and valley sides by themselves. But at 3x only a fifth of the island reaches the field system's "hill", so four fifths gets the lowland's pattern of fields, and the town and works ground stays in squares of 2 km (`probe-e3/glenwick-1850.png` shows them). Not addressed, and not this design's to address; it should not be expected to.
- **Note 6, lakes, large rivers, canyons, plateaus.** Answered, with the reservations above. "Large rivers" at the core's width are 80 to 118 m at their widest, fine threads at play distance; the core's width constant is the lever and is the core's decision.
- **Note 7, watersheds and water industry.** The watershed map (stage 3, at the cost above) and the season's regime on the meadows are offered; floods, dams, reservoirs and irrigation are rightly left until the simulation has them. A works before 1790 has a waterwheel and its pond wherever its yard lies within 180 m or so of a drawn river (`scatter.rs`, `firm_water`), whatever that river carries; with twice today's length of river drawn, more works will turn wheels on brooks of a cubic metre a second. The cell's `power` (kW) is the honest measure, and asking it is a small change worth making with stage 2.
- **Notes 19 and 20.** Stage 4 answers both, but the cart of note 19 need not wait (above), and note 20's "roads often run alongside them" (a road keeping to one bank of a valley) is the part of stage 4 that is least specified.

### What it promises that the core cannot yet tell it

- **Tideways as lines** (section 9, item 3), for bedding the half of the tideway that is one node wide. The viewer's own tracing is the stopgap and should be planned as the answer until the core has them.
- **The silted loops** for meander scars (item 4) and **the tablelands' plan** for the height of a cap's wall (item 5): neither is kept. Stage 6 without item 5 is a guess at the wall.
- **Braided reaches**: none on eight islands; the design should not draw for them.
- **A game on the new land before the switch** (item 7): without it there is no live path to test, and no `--shots` of the new land.
- **Bridges as facts**: the design is clear that roads as links, a cost to cross and a bridge as a work are the simulation's, and until then every bridge is the viewer's choice. That is honest; the choice should still be made from the roads (above).
- **The season's regime** (monthly runoff), the cell's water power, its marsh and flood plain, the lake's outlet: all kept, as the design says.

### Verdict: build it changed

The probe is good work: its measurements hold to the digit, its first stage is the right one and is invisible on the old island, and its picture of the new land is far better than today's. Build it, with these stages in place of section 6's:

1. **Stage 0, the land reaches the viewer**, as designed; the land in its own `Arc`; a lite pack of a land run says it cannot be drawn; the core's constructor for a game with the land on, and `--shots` taken on it, so that every later stage has a gate of 0 pixels on the new land as on the old.
2. **Stage 1, the ground from the core's heights**, as designed, and: the relief fixed at three times until the heights are in metres (7 below); the cells' middles moved onto dry, level ground in `network::centres`; the lakes as a mask in `Land::area`, `water_dist` and the dry tests, and the record's lake cells given their neighbours' classes on their dry ground; the stretch kept, with no shingle on the new land's shore. A day or two more than written.
3. **The railways on real relief** (from stage 5): the course chooses its passing point by the ground; the profile's rules stated in metres; gate `ISOSTATE_RAIL_CHECK=1` on the new land no worse than on the old island and no step. Then let Stella look, with a preview switch.
4. **Carts on the road as drawn**, on the old island too, whenever there is a free hand: it needs nothing above.
5. **Water** (stage 2) as designed, and: the bank strip's height answered on the CPU; the far thread's width by the flow's logarithm; lakes on a material of their own with their own level that goes to steel with the rivers; the waterwheels fed by the cell's `power`. Before it, the core decides its width constant and the evenness of its meanders, since the water is tuned on them.
6. **Roads and bridges** (stage 4), after dx-smooth's stage 2: bridges chosen from the year's roads, fewest crossings with a bounded detour, moved to sites, never removed; the road network made once a year where the churches and the scenery both read it; what may move when a bridge opens said in advance.
7. **Heights in metres and the land's classes on real hydrology** (stage 3), after the works' site planner and the towns' second pass have landed, with the whole list of thresholds above and the snow line's four copies changed together. Only then is the relief's factor free to be chosen by eye, with the rail check run again at the factor chosen.
8. **Towns, works and harbours on the real ground**: market places on the flattest dry ground, streets along the contour, the quarry towns on the scarps; quays and boats on tideways and lakes. With whoever carries on from the towns' second pass.
9. **The switch**, then the standard shots retaken.
10. **Scarps and canyons** (stage 6), with the rock decided on the CPU and the walls painted by their own direction; pits at the foot of the slope where their rock comes out (her note 2).

## What was built: stages 0 and 1

Branch `night/map-01` (12 November 2026): items 1 and 2 of the verdict above. The code is `crates/isostate-viewer/src/newland.rs` and the three files under `src/newland/`, with small hooks in `replay.rs`, `live.rs`, `world.rs`, `network.rs`, `terrain.rs`, `terrain.wesl`, `scatter.rs` and `main.rs`, and in the core `save.rs`. The probe's `landview.rs`, its global and its environment switch did not come with it. The viewer's README ("The new land") says how to look. Measurements are on the machine only, with every command, in `.claude/night/reports/opus/map-01/build.md`; pictures under `.claude/night/shots/newland-stage0/`, `.claude/night/shots/newland-stage1/` and `.claude/night/shots/opus/map-01/`. The machine was shared with other helpers throughout, so timings are good to a tenth or so.

### Built as written

- **The land in an `Arc` of its own** (`Replay::land: Option<Arc<NewLand>>`), read in `replay.rs` all or nothing, each array held to the grid's size or the map's and refused by name when short, and filled in `live.rs` from `LiveStatic::land`. A copy of the run copies a pointer (tested in both).
- **A lite pack** of a run made on the new land is refused, as every lite pack is, and now says that the land is not in it either. A full pack that speaks of the land and has none of its arrays is drawn as the old kind of island and says so in a sentence on the terminal.
- **The core's constructor**, `save::Game::on_new_land`, beside `Game::under`, and the viewer's `--new-land`, with which `--shots`, `--land-shots`, `--view`, `--shot`, `--script` and `--describe` all work.
- **The ground from the core's heights**, eight vertices to a cell, with no relief of the viewer's own added and none of its coast; the valley floors from the core's wet ground; the core's lines as `rivers::River`, bedded, drawn by today's ribbons; a sheet of water on each lake; the sea reading the same heights.
- **The relief at three times**, a named constant (`newland::RELIEF`) with the critic's reasons beside it, not a setting of the look.
- **The shore's stretch**, the probe's, in one function (`newland::units`) that the ground, the rivers' levels and the lakes all go through; and no shingle on the new land's shore.
- **The cells' middles** moved in `network::centres` onto dry, level ground of their own cell.
- **The lakes as water** to `Land::area`, to `water_dist` and to the dry tests, and the dry shore of the record's cells of water drawn as land.
- **A gate on the new land**: two named sets of pictures a stage (the standard twelve and the far views, of island 7 with six families), each taken twice and found the same to the pixel.

### Built differently, and why

- **The save carries the land** where the design said such a game would be started with pictures only. `Save::land` (left out of every other save, which is written byte for byte as it was) is a few lines and a test, and with it no save can be continued on the wrong island. The viewer still keeps a game on the new land out of the island's autosave and trials, so that a look at the preview never sets the player's own game aside: it is kept where `--save FILE` names a file, and otherwise says that it is not kept.
- **The river's bed is the squares of the mesh the water lies in** (250 to 500 m), not the probe's flat of 1.42 vertex steps a side with a slope of 0.6 beyond it. The probe's bed took a node off each wall of a canyon (the vertex 500 m from the river was cut from 300 m above the water to 26 m). This one lowers 5,598 of island 7's 646,753 vertices, by 14 m on average, and leaves every river point's ground at or under its water.
- **The surface does not overshoot downwards.** Laid through the nodes it dug pits in valley floors between two nodes of the floor and could put a low shore under the sea; a spot is now never lower than the lowest of the nodes it lies between, and under the sea where all of them are. The test on the shore therefore asks for no overshoot at all.
- **A lake is a height under nought to whoever asks**, not a mask at each test. The "dry enough" tests are some thirty comparisons of `world::height_at` with a threshold, in files the works' planner and the towns' second pass are rewriting. `height_at` tells the depth under a lake's sheet as a height under nought, as it tells the sea's bed, so every one of them keeps out of a lake unchanged, and so will those not yet written. The mesh and the sea's shader read the heightfield itself.
- **The lake's sheet** is cut at the mesh's squares and reaches only to the middle of the dry nodes beside the lake, where the probe drew whole nodes; and it is lifted 0.0006 where the probe lifted 0.004, which drew the lowland lake (4.5 m) over ground up to 7 m.
- **The dry shore takes the open land of the cell beside it**, not all its classes: a first cut painted a headland as the next cell's town. The check found the same wanted for a parcel all under a lake's sheet where it was looked at, whose shore comes up through the sheet between the spots: it too takes the land cell's open land, where it had been painted as the sea's sand (below).
- **The middles weigh more than the critic listed**: ground under the building line (the first 1.4 m, and beside a river near the tide, whose bed brings the ground down to it) counts with the water, since a market place there has no houses.

### What was measured

Island 7 unless said; the tests on islands 2, 3 and 7 (`cargo test -p isostate-viewer newland -- --nocapture`).

| | |
|---|---|
| the land as drawn (`ISOSTATE_LAND_REPORT=1`): heights, half below, 90% below, top | 0.108, 0.593, 1.413 (the probe's 0.11, 0.59, 1.41) |
| slope as drawn: half below, 99% below; steeper than 1.15 | 0.059, 0.769; 0.30% (the probe's 0.06, 0.75; 0.31%) |
| rivers drawn | 64, 42,747 points, 1,825 km, 6 to 83 m wide (islands 2 and 3: 77 and 75 rivers, 1,755 and 1,771 km) |
| the ground: laying, bedding, the lakes, all of `build_ground` | 16 to 35 ms, 2.4 to 3.6 ms, 30 ms, 166 to 286 ms (the old island's 323 ms on the same loaded machine) |
| a cell's mean height as drawn against the core's byte (5.9 m a step), three islands | at the middles of its dry nodes within one step in 98.9 to 99.0% of land cells and within two in 99.6 to 99.8%, 1.6 to 1.7 m out on average; at every vertex in those nodes within one step in 82 to 84% and within two in 92 to 93%, 3.6 to 3.8 m out on average |
| the cells' middles | 2,108 of 5,824 moved, by 384 m on average and 1,413 m at the furthest: 124 off water, 143 off ground too low to build on, 6 back from the water's edge, 133 off marsh, 170 back from a river, 1,532 off steep ground. 17 are left on marsh or low ground, their cells having no better |
| the first towns of three islands | all 18 market places on dry ground but Glenwick's, whose cell has none above 1.8 m (as drawn it stands on its bay's shore with its houses round it) |
| island 7's towns founded later, the nine the critic found on 1 in 8 to 1 in 2.6 | now on 1 in 9 to 1 in 68 (Carford 1 in 10, Brackside 1 in 9, the rest 1 in 21 or gentler) |
| lakes | 469 nodes of lake on island 7 (77 and 110 on islands 2 and 3); 261 parcels lie wholly in a lake and have no dry land; of 270 thousand tree slots, 22 thousand farm slots and 26 thousand hedge slots none is in a lake, and 1,518 stand within 500 m of one |
| cells of water that hold dry shore | 117; 762 parcels of them drawn as land |
| works drawn smaller for want of room (`ISOSTATE_TIMING=1`) | 836 of 2,122 (39%); the old island 521 of 2,320 (22%); none without ground of its own |
| the railways of 1900 (`ISOSTATE_RAIL_CHECK=1`), the next stage's starting point | 205 viaducts over 129.4 cells, 43 tunnels, 51 inclines and banks, no step; a line stands 0.520 over "the ground" at its highest, which is the viaduct across a corner of Carford's lake (the critic's figures for the probe: 204 over 129.7, 47, 61, and 0.391; for the old island 92 over 45.5, 30, 29) |
| a year's rebuild (`ISOSTATE_BENCH=1`, three takes each, turn about) | old island 1,071, 1,007 and 961 ms a year; new land 1,024, 1,104 and 1,082 ms: 6% more on average, with 10% more vertices in 1760 (11.1 against 10.1 million) and 2% fewer in 1900. The same time a vertex (88 against 86 ns in 1760). The index of water holds 1,825 km of river against 887 and does not show |
| the shares of open land as drawn (`ISOSTATE_AREAS=1`, the game of island 7 recorded on each land, 1750, 1800, 1850 and 1900) | over the whole island each class is within 1.2 points of the record's on the new land (the worst is wood, 30.8 for 29.6 in 1850 and 13.1 for 12.0 in 1900) and within 0.8 on the old island; over districts of 12 km a class is out by 1.6, 1.8, 2.4 and 2.2 points on average on the new land and by 1.8, 1.9, 2.2 and 2.2 on the old. On the tests' made-up mixes (three islands) each class is within 1.8 points |
| the two named sets, two takes each | all 24 pictures the same to the pixel at stage 0 and again at stage 1. The stage 1 sets were taken again by the check, with main's towns' second pass merged in and the lakes' shores mended, and are the gate for the next stage; the builder's are kept beside them as `newland-stage1-before-towns-2` |
| the old island | its twelve standard pictures at 0 pixels against `merged-15-town-ground`; the sweep of eight islands at 0 facts differing; a run with no land has the jittered middles to the last bit (tested) |

### The faults the critic foretold, each with its picture

Under `.claude/night/shots/opus/map-01/views/` (island 7, the summer of 1850, each view at play distance and close).

- **A market square in an estuary:** mended. Melness stands on the banks of its tideway, its church and market place on the west bank (`melness-close.png`).
- **Trees in a tarn:** mended. The woods come down to the water and stop (`tarn-close.png`).
- **Shingle banks on a tideway:** mended. The banks are wet and dry sand (`melness-close.png`).
- **A lake's shore painted as beach:** mended. Fields, woods and a farmstead reach the water (`tarn-close.png`, `lake-close.png`). The check found patches of the sea's sand left on the shores of two upland lakes (`check/lake5-close.png`, `check/lake2-close.png`), where a parcel lay all under the sheet at the spots it was looked at, and mended them (`check/lake5-close-mended.png`).
- **The old sine rivers on real valleys:** mended. The rivers are the core's, in their own valleys (`meanders-close.png`, `hills-play.png`).

### What the pictures show that belongs to a later stage

- **Railways on viaducts over real relief** (`belt-play.png`, `hills-close.png`, `canyon-play.png`), and at Carford across a corner of its lake (`carford-close.png`: a lake is water to the railway's profile as the sea is, and its course does not go round): the verdict's item 3, and the first thing anyone will see in the hills.
- **Roads that cross rivers anywhere, and tideways and lakes on causeways** (`meanders-close.png`, `melness-close.png`): item 6. A road across a lake goes under the sheet and is not seen.
- **Banks without edges, and no bars, oxbows or falls** (`meanders-close.png`): item 5.
- **A tideway one node wide breaks into pools** (`melness-play.png`): item 5, the tideways' own lines.
- **A lake's flat shore follows the mesh in steps of 250 m**, and its sheet is one flat colour (`lake-south.png`, `lake-far.png`): item 5, the lake's own mesh and material.
- **The sea's lighthouse on a lake's shore** (`lake-close.png`, Sandmere): item 8.
- **Towns on slopes** (`carford-close.png`, `brackside-close.png`): the market places are on the gentlest ground their cells have, 1 in 10 and 1 in 9, with the church beside them on a shelf, and the streets still take no notice of the contour: item 8.
- **No summer snow, and winter snow only as frost**: the island's top is 1.41 as drawn and the snow line 1.75: item 7.

### Checked

Gone over on `night/map-01-check` the same night, with main merged in twice (the towns' second pass, then the news's mends). The builder's numbers came out again to the digit (the tests on islands 2, 3 and 7, both stage 1 sets at their head at 0 pixels), the old island's twelve are at 0 pixels against main's retaken reference, and the sweep differs from main in nothing. The tests that guard the lakes and the middles fail when what they guard is taken out. A game on the new land played by script and saved in a named file goes on again on the new land; nothing of it reaches the island's autosave, and an ordinary game started against the same folder is on the old island. Counted by the node under them, as the critic counted, the land cells' middles on tidewater, in a lake and on marsh went from 108, 3 and 103 to 2, 0 and 8 on island 2, from 80, 4 and 106 to 0, 0 and 3 on island 3, and from 101, 5 and 85 to 0, 0 and 7 on island 7 (`ISOSTATE_LAND_REPORT=1`); of the first towns of the three islands only Glenwick's market place is still on marsh, its cell having nothing higher. Two things were mended: the lakes' shores above (`world.rs`, one condition, on the new land only), and a recorded run whose nodes name a lake its header does not list is refused by name, where the ground would have stopped at an index out of range. The notes, with every command, are `.claude/night/reports/opus/map-01/check.md`.
