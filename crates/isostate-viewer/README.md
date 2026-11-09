# isostate-viewer

The game, and a 3D viewer for Isolated State runs, in Bevy 0.20. It draws the island as a small diorama that changes year by year from 1750 to 1900. It either runs the simulation live, with you playing one of the island's great families, or plays back a `run.bin` written by the Python package or by the Rust core.

![Kirkford, 1900](../../docs/shots/kirkford-1900.jpg)

## Running

```bash
cargo run -p isostate-viewer --release                                    # a new game on island 7 with six families
cargo run -p isostate-viewer --release -- --seed 11 --families 4          # another island, fewer rivals
cargo run -p isostate-core --release -- run --seed 7 --out runs/rust7     # record a run (or use the Python package)
cargo run -p isostate-viewer --release -- runs/rust7/run.bin              # and watch it
```

The first build compiles Bevy and takes a while. On Linux you need `pkg-config`, `libwayland-dev` and `libxkbcommon-dev` to build; Windows and macOS need nothing extra. Other options: `--year 1846` opens a replay on a year, `--advance 20` starts a game twenty years in, `--size 1600x1000` sets the window.

| Key or mouse | Does |
|---|---|
| Space | play and pause (a replay starts again from 1750 at the end) |
| Left / Right | step a year (Page Up / Page Down: ten); in a game, Right at the present plays the next year |
| Up / Down | faster or slower; or the Pause, 1x to 8x buttons |
| Left drag, WASD | pan |
| Right drag, Q / E | turn and tilt |
| Wheel, R / F | zoom |
| Click | ask about a works, a town or a square |
| 1 2 3 4 | the Family, Holdings, Markets and News panels |
| H | every key |
| Esc | close the note about what was clicked |
| T | tilt-shift (miniature focus) on and off |

## Playing

You are a family among six, with some money and no works. The others already hold land and works; landed families live off rents, industrial ones build near home. The simulation runs a year at a time: Next year plays one and stops, Space keeps going.

- **Click a square** (in the present year) and the note says who holds it. Buy it at 25 years' rent: its rent, and the ground rent of anything built on it, comes to the family. Below that is what could be built there: each works with its cost, its expected return at today's prices, and how deep its market is. "Thin market" means the works would make a large share of what its town buys, or the good sells far above its usual price; the promised return will not last there.
- **The Family panel** has your worth over the years, last year's accounts, the great families' table, and the family's members. Click a member to put them in charge of a trade: the family's works in it get a little cheaper to run, more so the abler the manager. People age, die, and heirs come of age.
- **Holdings** lists your works; click one to fly to it. Your works fly a pennant in the family colour and your land is fenced in it.

How families work inside the economy, and what the scripted test players found, is in `crates/isostate-core/FAMILIES.md`.

### Screenshots without a window

Useful for checking the look on a machine with no GPU (it renders in software, so expect a few minutes):

```bash
WGPU_BACKEND=vulkan cargo run -p isostate-viewer -- runs/landlords/run.bin --size 1600x1000 \
    --view out/kirkford.png,1900,77.5,47.5,25,38,10 \
    --view out/island.png,1900
```

Each `--view` is `path,year,x,z,yaw,pitch,distance[,season][,panel]` in map cells and degrees; missing values fall back to the whole-island view (or the home town in a game). A number after the distance is the season (0 midwinter, 0.25 spring, 0.5 summer, 0.75 autumn), overriding `--season` for that shot; a word is the side panel to show (`family`, `holdings`, `markets`, `news`, or `none`). `--wait 100` sets how many seconds to wait for shaders to compile before the first shot.

For a game, `--advance N` plays N years first, `--demo` has the family build a works and buy some land in the first year, and `--pick x,z` starts with that spot clicked, so the order panel shows:

```bash
WGPU_BACKEND=vulkan cargo run -p isostate-viewer -- --demo --advance 3 --pick 31.5,29.5 --size 1600x1000 --wait 150 \
    --view out/game.png,,31,29.6,25,46,4.2,family
```

The welcome card of a new game is left out of screenshots unless `ISOSTATE_WELCOME=1` is set.

### Checking the ground shader against the CPU

The ground shader repeats the patchwork rule of `world.rs` that places the trees and houses, so the two must agree. To check, write the CPU's class under every pixel of some views, shoot the same views with the ground in flat class colours, and compare:

```bash
V=out/check.png,1870,77.6,47.6,0,80,6
cargo run -p isostate-viewer -- runs/landlords/run.bin --size 1600x1000 --class-map --view $V
WGPU_BACKEND=vulkan cargo run -p isostate-viewer -- runs/landlords/run.bin --size 1600x1000 --wait 100 --no-tilt --ground-debug --view $V
python3 crates/isostate-viewer/tools/compare_classes.py out/check.png
```

Only pixels on a class edge (and under the rounded corners of the interface) should differ.

Two quicker checks need no GPU. `ISOSTATE_PLAN=out/plan.ppm` writes a top-down plan of the ground from the CPU's rules (relief, land classes, field edges, rivers, hedgerow trees) and stops; `ISOSTATE_PLAN_VIEW=x0,z0,x1,z1,pixels-per-cell` and `ISOSTATE_PLAN_YEAR` pick the area and the year, and it also lists the railway's embankments, cuttings, viaducts and tunnels. `ISOSTATE_BENCH=1` times the rebuild of a handful of years in CPU time and stops, and `ISOSTATE_TIMING=1` prints how long the ground, the slots and each year's rebuild take while the viewer runs.

## How the picture is made

The economy only knows shares and densities per 2 km cell, so the viewer invents the detail, deterministically.

- **Landform.** One heightfield (eight vertices per cell, relief exaggerated about three times), in chunks of 8 by 8 cells so that what is out of view is not drawn, and open sea far from any shore is left to the sea's surface. Below the scale of a cell the land takes the shape of its terrain: soft swells in the lowland, rounded spurs between V-shaped valleys in the hills, sharp crests in the mountains, added so that each cell keeps its height on average. The coast is ragged, with bays and headlands; low land meets the sea on a narrow beach and high land breaks off in a cliff (`world.rs`).
- **Rivers.** The simulation knows which cells a river runs through and how big it is there. The river runs beside the cell centres (where the roads meet and the market squares are) on a smooth course, and meanders about it as a sine-generated curve, the shape real meanders take: wide, irregular loops in flat lowland, a nearly straight course in steep upland, its loops longer as it widens downstream. Its level falls steadily from source to mouth and it cuts its valley down to that level: a flat floodplain about the meander belt in the lowland, painted as water meadows, a narrow V in the hills, and a drowned funnel, an estuary, at the coast (`rivers.rs`). Roads cross on hump-backed stone bridges, and on a causeway where they cross drowned ground.
- **Fields.** The land is divided into fields by `fields.rs`: blocks are the quads of a jittered grid, turned against the cells and gently warped (more in some districts than others), cut again and again across their longer side, so there are long hedge lines with T-junctions and blocks of near-parallel fields, some in long strips. Fields are small round the towns, middling in the lowland, large on the hills. The first three cuts make a parcel, and a whole parcel takes one land class by the patchwork rule of `world.rs` (the cell's two largest classes, the lower class taking the low values of a smooth noise) looked up at the parcel's own point, so farm, pasture, wood and town change only at hedge lines. The ground shader (`src/terrain.wesl`, an extension of Bevy's standard material, so light, shadows, ambient occlusion and haze work as usual) repeats the field system and the class rule exactly, from a texture of the year's mix with one texel per cell, so trees and houses stand on matching ground. Each field has its own crop, calendar and rows along one side; hedges (stone walls on the hills) run along every field edge with a painted shadow on the side away from the sun; heath gets heather and gorse, towns plain ground of yards, cobbles and gardens, works yards of cinder and gravel; steep ground turns to rock and high ground to snow, and faint contour lines run round the open hillsides as on an engraved map. Edges and fine patterns are filtered to the size of a pixel, so they stay crisp up close and calm far away. The seasons come from the clock: young green crops in spring, gold in high summer, stubble and ploughing in autumn, heather in flower late in summer, and frost and lower snow in winter.
- **Open fields.** Round each town the lowland lies in open fields until its parish is enclosed, in a year of its own between 1755 and 1835: furlongs of long strips, each strip its own crop with ridge and furrow, and no hedges. In that year the hedges are planted, hedgerow trees grow up along them, and farmsteads go up out among the new fields.
- **Sea.** A sheet of water with its own shader (`src/sea.wesl`) that reads the depth under every pixel from the ground's heightfield: clear over the sand, green-blue over the shelf, dark blue far out, a line of foam at the water's edge and bands of surf rolling in, and ripples that catch the sun.
- **Railways.** Each line between junctions is a smooth curve through the cell centres, graded so that it never climbs faster than a ruling gradient and comes back to the ground at the towns and junctions. Where the ground falls away it runs on an embankment (faced with stone by the sea), on a viaduct of brick or stone arches where the valley is deep, and on an iron girder bridge over a river; where the ground rises it runs in a cutting, shown by raised banks either side, and through a tunnel, with a portal at each end, where the hill is too high (`rails.rs`). The trains run on the graded track and vanish into the tunnels.
- **Things on the ground.** Every possible tree, house, farmstead and mine has a fixed slot with a fixed threshold, and it appears once its cell's share or density passes that threshold. Growth is therefore smooth and stable: nothing reshuffles from one year to the next, towns thicken in place and woods thin tree by tree. Hedgerow trees have slots on the field edges, and farmsteads (a house with its back to a hedge, a yard, a barn and a byre, ricks by an arable farm) stand among the fields with a lane to the nearest road. Nothing stands in a river. Town houses go up first in rows facing the roads, ridges along the street, then fill the blocks behind; the earliest houses keep their ground as a town grows.
- **Works.** Each firm has a stable plot in its cell, first come first served (a live game places new firms as they appear), and its building depends on its method (mills, furnaces, kilns, sheds) and grows in tiers as its capacity doubles. A works stands in scaffolding the year it opens and the year it grows a size. Chimneys of working firms smoke.
- **Life.** Trains run the railway lines, covered carts plod the main roads, and small sailing boats tack off every harbour town.
- **The game's marks.** Family works fly a pennant and family land is fenced, both in the family colour; an ordered works is staked out until it goes up; the square under the pointer and the one clicked are outlined on the ground.
- **Towns.** A church on the market square, a station once the railway reaches the town, a lighthouse for harbours.
- **Lines.** Roads are ribbons draped over the terrain between jittered cell centres; railways being built are dashed.
- **Light.** A low warm sun with shadows, soft sky light, screen-space ambient occlusion, distance haze, and an optional tilt-shift depth of field for the miniature look.

## The interface

All of it is Bevy UI in one paper-and-ink style (`src/ui.rs`, `src/family.rs`, `src/game.rs`): cards with rounded corners and soft shadows, the Fredoka font embedded in the binary, and small icons drawn at startup from distance shapes (`src/icons.rs`), so there are no image files. Bevy 0.20 deprecates the `Interaction` component these buttons use; moving to its picking and widget crates is a job for a later upgrade.

## What is next for the look

- Construction with a puff of dust; people on the streets; weather and cloud shadows.
- A title screen, and saving a game (the replay format can carry one).
- Building the year's meshes off the main thread, so a new year never stutters.
