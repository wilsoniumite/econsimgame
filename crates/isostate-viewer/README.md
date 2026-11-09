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
| Esc | close the note about what was clicked, or put off a decision |
| T | tilt-shift (miniature focus) on and off |

## Playing

You are a family among six. You start with some cash, land around your seat town and a handful of relations; the other families already hold land and works across the island. The simulation runs a year at a time: Next year plays one and stops, Space keeps going. Every choice is from a short list, never an amount to set.

- **Decisions** come as cards and stop the clock: the family's first trade, what each child learns at fourteen (an apprenticeship in a trade, which the family then knows, or university), who heads the family when a head dies. Each option says what it would mean, the default is marked, and Decide later leaves it for the badge beside Next year. Unanswered, the default is taken at the end of the next year.
- **The seat** is where the head lives. Only there do you build works yourself (click a square in the present to see what could be built, with its cost, its expected return at today's prices and how deep its market is) and buy land. The head runs every trade the family knows at the seat. Click another town to move the seat there at the year end; its square flies your flag.
- **Branches.** Post a grown member to run one of your trades in another town (from their card in the Family panel, which suggests where they would count, or from the town's note). A branch makes its works cheaper to run, the abler the runner the more, and grows on its own: a works a year, or land for estates. In Holdings, set each branch to Grow, Hold or Wind down. Works with nobody of the family over them pay a fifth of their profit to a hired manager.
- **The Family panel** has your worth over the years, last year's accounts, anything waiting for you (decisions, members with nothing to run), the family tree, and the great families, each of which can be opened to see its head, trades and branches. Holdings lists the branches and works; click a works to fly to it. Your works fly a pennant in the family colour and your land is fenced in it.

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

## How the picture is made

The economy only knows shares and densities per 2 km cell, so the viewer invents the detail, deterministically.

- **Ground.** One heightfield mesh (four vertices per cell, relief exaggerated about three times) whose colour a shader works out for every pixel (`src/terrain.wesl`, an extension of Bevy's standard material, so light, shadows, ambient occlusion and haze work as usual). Each cell's two largest land classes are drawn as a patchwork using a smooth noise, so a cell that is 60% farm and 40% wood looks like fields with woods in them; the shader reads the year's mix from a texture with one texel per cell and uses the same warp, hash, noise and quantile table as `world.rs`, so trees and houses stand on matching ground. Fields come from a warped grid, each with its own crop, calendar and rows, with hedges along their edges; heath gets heather and gorse, towns a street grid between the house plots, works yards of cinder and gravel; steep ground turns to rock and high ground to snow. Edges and fine patterns are filtered to the size of a pixel, so they stay crisp up close and calm far away. The seasons come from the clock: young green crops in spring, gold in high summer, stubble and ploughing in autumn, heather in flower late in summer, and frost and lower snow in winter.
- **Things on the ground.** Every possible tree, house, farmstead and mine has a fixed slot with a fixed threshold, and it appears once its cell's share or density passes that threshold. Growth is therefore smooth and stable: nothing reshuffles from one year to the next, towns thicken in place and woods thin tree by tree. Town houses go up first in rows facing the roads, ridges along the street, then fill the blocks behind; the earliest houses keep their ground as a town grows.
- **Works.** Each firm has a stable plot in its cell, first come first served (a live game places new firms as they appear), and its building depends on its method (mills, furnaces, kilns, sheds) and grows in tiers as its capacity doubles. A works stands in scaffolding the year it opens and the year it grows a size. Chimneys of working firms smoke.
- **Life.** Trains run the railway lines, covered carts plod the main roads, and small sailing boats tack off every harbour town.
- **The game's marks.** Family works fly a pennant and family land is fenced, both in the family colour; every family's seat flies a tall flag on its market square (a shorter, paler one where it is moving at the year end); an ordered works is staked out until it goes up; the square under the pointer and the one clicked are outlined on the ground.
- **Towns.** Laid out along the roads that run through them (`src/scatter/towns.rs`): each road through town ground is a high street, with back lanes, outer lanes and cross streets behind it that fill in as the town gets denser. Houses stand on plots fronting a street, set back to a line. The high streets and the market place keep rows of separate houses of different heights and colours; from about 1815 the newer streets are built as brick terraces with slate roofs. Every house has windows and a door. Each town has a paved market square with a market hall (or a market cross in a small town), a church facing it, a station road once the railway comes, and a lighthouse for harbours.
- **Lines.** Rivers, roads and railways are ribbons draped over the terrain through jittered cell centres; lines being built are dashed.
- **Light.** A low warm sun with shadows, soft sky light, screen-space ambient occlusion, distance haze, and an optional tilt-shift depth of field for the miniature look.

## The interface

All of it is Bevy UI in one paper-and-ink style (`src/ui.rs`, `src/family.rs`, `src/game.rs`): cards with rounded corners and soft shadows, the Fredoka font embedded in the binary, and small icons drawn at startup from distance shapes (`src/icons.rs`), so there are no image files. Bevy 0.20 deprecates the `Interaction` component these buttons use; moving to its picking and widget crates is a job for a later upgrade.

## What is next for the look

- Construction with a puff of dust; people on the streets; weather and cloud shadows.
- A title screen, and saving a game (the replay format can carry one).
- Building the year's meshes off the main thread, so a new year never stutters.
