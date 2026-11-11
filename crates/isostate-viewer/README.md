# isostate-viewer

The game, and a 3D viewer for Isolated State runs, in Bevy 0.20. It draws the island as a small diorama that changes year by year from 1750 to 1900. It either runs the simulation live, with you playing one of the island's great families, or plays back a `run.bin` written by the Python package or by the Rust core.

![Kirkford, 1900](../../docs/shots/kirkford-1900.jpg)

## Running

```bash
cargo run -p isostate-viewer --release                                    # the game on island 7 with six families: a new one, or the one you left
cargo run -p isostate-viewer --release -- --new                           # a new game, whatever is saved
cargo run -p isostate-viewer --release -- --seed 11 --families 4          # another island, fewer rivals
cargo run -p isostate-core --release -- run --seed 7 --out runs/rust7     # record a run (or use the Python package)
cargo run -p isostate-viewer --release -- runs/rust7/run.bin              # and watch it
```

The first build compiles Bevy and takes a while. On Linux you need `pkg-config`, `libwayland-dev` and `libxkbcommon-dev` to build; Windows and macOS need nothing extra. Other options: `--year 1846` opens a replay on a year, `--advance 20` starts a game twenty years in, `--size 1600x1000` sets the window, `--at 31.5,29.5` points the camera at a spot in map cells, `--overlay rent` opens on a map view, and `--load FILE` and `--save FILE` name a saved game (see Saved games). `--help` lists every option, those of the standard shots (`--shots`) and of the generators' lab (`--lab`) among them. An option that is not one, an option with no value after it, or a value that cannot be read (`--advance 3O`, `--season summer`, `--overlay rents`) stops the viewer with the reason and the list (exit code 2), rather than running as if it had not been given. A `run.bin` that is cut short or damaged is refused with a message naming the file and the array; the "make a run first" hint follows only a file that is not there.

| Key or mouse | Does |
|---|---|
| Space | play and pause (a replay starts again from 1750 at the end) |
| Left / Right | step a year (Page Up / Page Down: ten); in a game, Right at the present plays the next year |
| Up / Down | one speed button faster or slower (1x is a year every two seconds, 8x four years a second); or the Pause, 1x to 8x buttons |
| Left drag, WASD | pan |
| Right drag, Q / E | turn and tilt |
| Wheel, R / F | zoom |
| Click | ask about a works, a town or a square |
| 1 2 3 4 | the Family, Holdings, Markets and News panels |
| M | map views: rent, markets, reach, prices, wages, then the plain map (Shift+M back) |
| G | the next good, when prices are mapped (Shift+G back) |
| H | every key |
| Esc | close the note about what was clicked, put off a decision, or close the card at the century's end |
| T | tilt-shift (miniature focus) on and off |

While the opening card, a decision or the card at the century's end is up, the clock stands still and the other keys wait. Any key or a click closes the opening card without starting the clock (Alt, Shift, Ctrl and Print Screen do not).

## Playing

You are a family among six. You start with some cash, land around your seat town and a handful of relations; the other families already hold land and works across the island. The simulation runs a year at a time: Next year plays one and stops, Space keeps going. Every choice is from a short list, never an amount to set.

- **Decisions** come as cards and stop the clock: the family's first trade, what each child learns at fourteen (an apprenticeship in a trade, which the family then knows, or university), who heads the family when a head dies. Each option says what it would mean, the default is marked, and Decide later leaves it for the badge beside Next year. Unanswered, the default is taken at the end of the next year.
- **The seat** is where the head lives. Only there do you build works yourself (click a square in the present to see what could be built, with its cost, its expected return at today's prices and how deep its market is) and buy land. The head runs every trade the family knows at the seat. Click another town to move the seat there at the year end; its square flies your flag.
- **Branches.** Post a grown member to run one of your trades in another town (from their card in the Family panel, which suggests where they would count, or from the town's note). A branch makes its works cheaper to run, the abler the runner the more, and grows on its own: a works a year, or land for estates. In Holdings, set each branch to Grow, Hold or Wind down. Works with nobody of the family over them pay a fifth of their profit to a hired manager.
- **The present.** Orders (Build, Buy, Move the seat, answering a decision) are given in the newest year, and that is the year on screen for most of each year while the clock runs: the next year is asked for as late as will still have it ready. For a moment near each year's end it is being worked out: the buttons go unlit, and a decision card opened then says "The year is turning…" until it arrives. Pausing in that moment moves the clock on to the new year. Step back with Left or the timeline and the panels show the past; the order panel offers "Back to" the present, and the Family and Holdings panels and the family's card in the top bar say which year their figures are of ("The Fairfax family in 1780"), since the family is kept only as it stands today.
- **The Family panel** has your worth over the years, last year's accounts, anything waiting for you (decisions, members with nothing to run), the family tree, and the great families, each of which can be opened to see its head, trades and branches. Holdings lists the branches and works; click a works to fly to it. Your works fly a pennant in the family colour and your land is fenced in it.

- **News.** The panel lists the newest 120 items and says so at its foot ("The newest 120 of 457 are listed."); Show more adds 120.
- **Seasons.** The seasons keep their own clock: one round takes 24 seconds however fast the years go, holds when the game is paused and never jumps. The top bar shows the year alone; it names the season only when one is held with `--season` or a view's season.

How families work inside the economy, and what the scripted test players found, is in `crates/isostate-core/FAMILIES.md`.

### Map views

Press **M** to step through five views washed over the ground (Shift+M steps back), or click a chip on the legend card at the bottom of the screen. On the plain map the card shrinks to a "Map views" button, which brings back the view last used.

| View | What it shows | Scale |
|---|---|---|
| Rent | what the ground of each square earns its owners in a year | by ratio, leaving out the poorest tenth and the dearest twentieth of the land, so the ends read "or less" and "or more" |
| Markets | the country that trades in each town's market, with ink lines where one town's pull gives way to the next | none: six soft tints, no two neighbours alike |
| Reach | how far each square is from the market it trades in, as kilometres of flat open country | from "at the market" to a round distance nine tenths of the land is within |
| Prices | the year's average price of one good in each square's market town | from the cheapest town to the dearest, both named |
| Wages | what a month's work pays in each town, across its market area | from the lowest-paying town to the highest |

The price map starts with grain. **G** shows the next good and Shift+G the one before, as do Back and Next on the card; opening a good in the Markets panel sets the map to that good too. The legend card gives the year, one sentence on what is shown, and the real figures at the two ends. Click a square with a view on and the inspector adds the view's value there. Trees are hidden while a view is on, and the views that go by market towns name every town. Nothing is shown that the record does not hold: rent is stored on a scale that stops at 1000 a month, so the middle of a town reads "or more".

### Charts

The Markets panel (key **3**) has two halves, chosen by chips at the top:

- **Goods**: the prices against 1750. Click a good's row for its history underneath: price in ink (left scale) and how much is made a month in the family's colour (right scale). Click again to close it.
- **The island**: people, what a wage buys (1750 = 1), the share living in towns, railway length, and the number of works and shops.

Each chart labels its first and last year and its low and high value and marks the year on screen; when you look back at an earlier year the years after it are drawn fainter.

### Saved games

A game saves itself. After every year and every order the simulation thread writes the game to one file for the island and its number of families, `island-7-families-6.json`, in `%APPDATA%\Isolated State\saves` on Windows (`~/Library/Application Support/Isolated State/saves` on macOS, `$XDG_DATA_HOME/isolated-state/saves` or `~/.local/share/isolated-state/saves` on Linux; `ISOSTATE_SAVES=DIR` names another folder; with none of these to go by, `saves` beside the working directory). The terminal says where when the game starts.

Start the viewer with nothing but the game asked for and, if that island has a save, its years are played again before the window opens and the opening card says "Welcome back" and offers **Continue from 1786** and **New game**. A click anywhere else, or any key, also continues. New game starts the program again with `--new` and leaves the old game saved beside the new one.

- `--new` or `--seed N`: a new game, whatever is saved. `--advance N` also starts a new game, and it is autosaved.
- `--load FILE`: go on with that save, of any island; with it, `--advance N` plays N more years after the saved ones. The game then continues in its island's autosave, not in FILE.
- `--save FILE`: keep the game in FILE instead of the autosave. FILE must be free or already hold a saved game; anything else there is left alone and the game does not start.
- Runs that only take pictures (`--view`, `--shot`, `--shots`, `--describe`, `--class-map`), `--demo` games and scripted runs (`--script`) neither continue the autosave nor write it. `--save FILE` gives any of them a place; a scripted run also keeps an autosave when `ISOSTATE_SAVES` names a folder for it.

Beside the autosave: `.1.json` to `.3.json` are the last three years, used by themselves if the latest cannot be read (the card says so). `.previous.json`, `.previous.2.json` and `.previous.3.json` are the last three games that a new game on the same island set aside; open one with `--load`. `.unread.json` is a file found in the autosave's place that this version could not read. If the game cannot be saved (a full disk, a folder that cannot be written), the note under the date says so and the terminal says why.

A save is a log of orders, not the state of the island: the seed, the number of families, the rules' name with a fingerprint of their numbers, the year reached, a fingerprint of the island as it stood, and one line for each order with the year it was given in.

```json
{"year":1750,"kind":"decide","id":0,"choice":{"kind":"trade","group":"iron"}},
{"year":1750,"kind":"found","method":"charcoal_furnace","cell":3511}
```

Loading plays the years again and gives each order where it was first given (`isostate_core::save`). The simulation is deterministic, so the island comes back the same to the last digit; `tests/save.rs` in the core and `src/live_tests.rs` here hold it to that. It takes about a second for every seven years in the dev build, before the window opens, and the terminal counts the decades. The other side of that: a save is only as good as the rules it was made under. When the simulation has changed since, the years play out differently; an order the simulation now refuses is skipped and counted, and the opening card says how many, or that the island stands differently. The terminal lists each refused order and why.

### Screenshots without a window

Useful for checking the look on a machine with no GPU (it renders in software, so expect a few minutes):

```bash
WGPU_BACKEND=vulkan cargo run -p isostate-viewer -- runs/landlords/run.bin --size 1600x1000 \
    --view out/kirkford.png,1900,77.5,47.5,25,38,10 \
    --view out/island.png,1900
```

Each `--view` is `path,year,x,z,yaw,pitch,distance[,season][,words]` in map cells and degrees; missing values fall back to the whole-island view (or the home town in a game), and a value that is not a number is refused. A number after the distance is the season (0 midwinter, 0.25 spring, 0.5 summer, 0.75 autumn), overriding `--season` for that shot. Words after the distance may come in any order: the side panel to show (`family`, `holdings`, `markets`, `news`, or `none`), a map view (`rent`, `areas`, `reach`, `price`, `price:coal` or any other good's name, `wages`, `plain`; `--overlay NAME` sets it for the whole run), and `island` or `open:GOOD` for what the Markets panel shows. `--wait 100` sets how many seconds to wait for shaders to compile before the first shot. A `--view` of a year a game has not reached shows the newest year.

```bash
cargo run -p isostate-viewer -- --advance 100 --view out/coal.png,1850,60,46,0,52,130,markets,price:coal,open:coal
```

`--describe X,Z` prints what a click at that spot would show and stops; with `--overlay NAME` it adds the line the inspector would add for that view.

For a game, `--advance N` plays N years first, `--demo` has the family build a works and buy some land in the first year, and `--pick x,z` starts with that spot clicked, so the order panel shows:

```bash
WGPU_BACKEND=vulkan cargo run -p isostate-viewer -- --demo --advance 3 --pick 31.5,29.5 --size 1600x1000 --wait 150 \
    --view out/game.png,,31,29.6,25,46,4.2,family
```

The opening card of a game and the card at the century's end are left out of screenshots unless `ISOSTATE_WELCOME=1` is set; `ISOSTATE_WELCOME=continue` or `=new` presses that button of the opening card once, to see what it does with nobody at the pointer. `ISOSTATE_HELP=1` opens the help card from the start.

### Checking keys and clicks without a window

`--script "STEPS"` runs the game with no window, plays the steps in order as a player's presses and clicks, prints what happened on lines starting "script:", and stops. It is for what a still cannot show: keys, clicks and timing.

```bash
cargo run -p isostate-viewer -- --advance 30 --script "key:Space;key:Escape;settle;click:1x;run:10;print;key:M;map;shot:out/after.png"
```

Steps are separated by semicolons:

| Step | Does |
|---|---|
| `key:NAME` | press a key for one frame: Space, Left, Right, Up, Down, PageUp, PageDown, Escape, a letter or a digit; `key:Shift+M` holds Shift |
| `click:LABEL` | click the button with that text (a card open over it stops the click) |
| `decide:N` | click option N of the open decision, counting from 0 |
| `wait:SECONDS`, `frames:N` | let the game run |
| `settle` | wait until no year is being worked out and the scenery is current |
| `sample:SECONDS` | let the game run and count, frame by frame, where the clock stood and how far the season moved |
| `run:SECONDS` | the same, answering decisions with their default to keep the years going |
| `print[:NOTE]` | print the clock, the cards, the order panel and the side panel |
| `tab:NAME`, `pick:X,Z`, `look:X,Z,DIST`, `scroll:PIXELS` | open a side panel, click the map, point the camera, scroll the side panel |
| `chip:NAME` | click a chip of the legend card: `rent`, `areas`, `reach`, `price`, `wages`, `plain`, or `pill` for the Map views button |
| `good:next`, `good:back` | the legend's steps through the goods |
| `row:GOOD` | click a good's row in the Markets panel |
| `half:island`, `half:goods` | click a chip of the Markets panel |
| `map[:NOTE]` | let the panels catch up, then print in one line what the map and the Markets panel show |
| `shot:PATH` | save a picture |

A scripted run takes its pictures with `shot:PATH`: `--view`, `--shot` and `--shots` are passed over beside it. It does not touch the saved game unless `ISOSTATE_SAVES` or `--save` gives it a place of its own, and New game in a scripted run says what it would start instead of starting the program again. The list with what each step does is also at the top of `src/script.rs`.

### Checking the ground shader against the CPU

The ground shader repeats the patchwork rule of `world.rs` that places the trees and houses, so the two must agree. To check, write the CPU's class under every pixel of some views, shoot the same views with the ground in flat class colours, and compare:

```bash
V=out/check.png,1870,77.6,47.6,0,80,6
cargo run -p isostate-viewer -- runs/landlords/run.bin --size 1600x1000 --class-map --view $V
WGPU_BACKEND=vulkan cargo run -p isostate-viewer -- runs/landlords/run.bin --size 1600x1000 --wait 100 --no-tilt --ground-debug --view $V
python3 crates/isostate-viewer/tools/compare_classes.py out/check.png
```

Only pixels on a class edge (and under the rounded corners of the interface) should differ.

Two quicker checks need no GPU. `ISOSTATE_PLAN=out/plan.ppm` writes a top-down plan of the ground from the CPU's rules (relief, land classes, field edges, rivers, hedgerow trees) and stops; `ISOSTATE_PLAN_VIEW=x0,z0,x1,z1,pixels-per-cell` and `ISOSTATE_PLAN_YEAR` pick the area and the year, and it also lists the railway's embankments, cuttings, viaducts and tunnels. `ISOSTATE_BENCH=1` times the rebuild of a handful of years by the clock on the wall and stops (so bench on a quiet machine), and `ISOSTATE_TIMING=1` prints how long the ground, the slots and each year's rebuild take while the viewer runs.

## How the picture is made

The economy only knows shares and densities per 2 km cell, so the viewer invents the detail, deterministically.

- **Landform.** One heightfield (eight vertices per cell, relief exaggerated about three times), in chunks of 8 by 8 cells so that what is out of view is not drawn, and open sea far from any shore is left to the sea's surface. Below the scale of a cell the land takes the shape of its terrain: soft swells in the lowland, rounded spurs between V-shaped valleys in the hills, sharp crests in the mountains, added so that each cell keeps its height on average. The coast is ragged, with bays and headlands; low land meets the sea on a narrow beach and high land breaks off in a cliff (`world.rs`).
- **Rivers.** The simulation knows which cells a river runs through and how big it is there. The river runs beside the cell centres (where the roads meet and the market squares are) on a smooth course, and meanders about it as a sine-generated curve, the shape real meanders take: wide, irregular loops in flat lowland, a nearly straight course in steep upland, its loops longer as it widens downstream. Its level falls steadily from source to mouth and it cuts its valley down to that level: a flat floodplain about the meander belt in the lowland, painted as water meadows, a narrow V in the hills, and a drowned funnel, an estuary, at the coast (`rivers.rs`). Roads cross on hump-backed stone bridges, and on a causeway where they cross drowned ground.
- **Fields.** The land is divided into fields by `fields.rs`: blocks are the quads of a jittered grid, turned against the cells and gently warped (more in some districts than others), cut again and again across their longer side, so there are long hedge lines with T-junctions and blocks of near-parallel fields, some in long strips. Fields are small round the towns, middling in the lowland, large on the hills. The first three cuts make a parcel, and a whole parcel takes one land class by the patchwork rule of `world.rs` (the cell's two largest classes, the lower class taking the low values of a smooth noise) looked up at the parcel's own point, so farm, pasture, wood and town change only at hedge lines. The ground shader (`src/terrain.wesl`, an extension of Bevy's standard material, so light, shadows, ambient occlusion and haze work as usual) repeats the field system and the class rule exactly, from a texture of the year's mix with one texel per cell, so trees and houses stand on matching ground. Each field has its own crop, calendar and rows along one side; hedges (stone walls on the hills) run along every field edge with a painted shadow on the side away from the sun; heath gets heather and gorse, towns plain ground of yards, cobbles and gardens, works yards of cinder and gravel; steep ground turns to rock and high ground to snow, and faint contour lines run round the open hillsides as on an engraved map. Edges and fine patterns are filtered to the size of a pixel, so they stay crisp up close and calm far away. The seasons come from a clock of their own (`SeasonClock`, `SEASON_ROUND_SECS = 24` in `main.rs`), which the ground, the trees, the sun and the date all read: while the game plays it turns in real time, one round in 24 seconds whatever the speed of the years, and paused it holds where it is. So there are young green crops in spring, gold in high summer, stubble and ploughing in autumn, heather in flower late in summer, and frost and lower snow in winter.
- **Open fields.** Round each town the lowland lies in open fields until its parish is enclosed, in a year of its own between 1755 and 1835: furlongs of long strips, each strip its own crop with ridge and furrow, and no hedges. In that year the hedges are planted, hedgerow trees grow up along them, and farmsteads go up out among the new fields.
- **Sea.** A sheet of water with its own shader (`src/sea.wesl`) that reads the depth under every pixel from the ground's heightfield: clear over the sand, green-blue over the shelf, dark blue far out, a line of foam at the water's edge and bands of surf rolling in, and ripples that catch the sun.
- **Railways.** Each line between junctions is a smooth curve through the cell centres, graded so that it never climbs faster than a ruling gradient and comes back to the ground at the towns and junctions. Where the ground falls away it runs on an embankment (faced with stone by the sea), on a viaduct of brick or stone arches where the valley is deep, and on an iron girder bridge over a river; where the ground rises it runs in a cutting, shown by raised banks either side, and through a tunnel, with a portal at each end, where the hill is too high (`rails.rs`). The trains run on the graded track and vanish into the tunnels.
- **Things on the ground.** Every possible tree, house, farmstead and mine has a fixed slot with a fixed threshold, and it appears once its cell's share or density passes that threshold. Growth is therefore smooth and stable: nothing reshuffles from one year to the next, towns thicken in place and woods thin tree by tree. Hedgerow trees have slots on the field edges, and farmsteads (a house with its back to a hedge, a yard, a barn and a byre, ricks by an arable farm) stand among the fields with a lane to the nearest road. Nothing stands in a river. Town houses go up first in rows facing the roads, ridges along the street, then fill the blocks behind; the earliest houses keep their ground as a town grows.
- **Works.** Each firm has a stable plot in its cell, first come first served (a live game places new firms as they appear), and its building depends on its method (mills, furnaces, kilns, sheds) and grows in tiers as its capacity doubles. A works stands in scaffolding the year it opens and the year it grows a size. Chimneys of working firms smoke.
- **Life.** Trains run the railway lines, covered carts plod the main roads, and small sailing boats tack off every harbour town.
- **The game's marks.** Family works fly a pennant and family land is fenced, both in the family colour; every family's seat flies a tall flag on its market square (a shorter, paler one where it is moving at the year end); an ordered works is staked out until it goes up; the square under the pointer and the one clicked are outlined on the ground.
- **Towns.** Laid out along the roads that run through them (`src/scatter/towns.rs`): each road through town ground is a high street, with back lanes, outer lanes and cross streets behind it that fill in as the town gets denser. Houses stand on plots at fixed places along a street, set back to a line. The high streets and the market place keep separate houses of different heights and colours, each rebuilt as a fuller townhouse at a density of its own. The back streets go up a row at a time, in rows fixed per street: a row built from 1815 on is a brick terrace with a slate roof, an earlier one a row of cottages, and each keeps the style it was built in. Every house has windows and a door. Each town has a paved market square with a market hall (or a market cross in a small town), a church facing it, a station road once the railway comes, and a lighthouse for harbours.
- **Lines.** Roads are ribbons draped over the terrain between jittered cell centres; railways being built are dashed.
- **Map views.** `overlay.rs` writes one tint per square (and, in alpha, the square's market) into a per-cell texture, binding 104 of the ground material, and sets three numbers in the ground's uniforms (`overlay`, `overlay_soft`, `overlay_lines`). `terrain.wesl` (`overlay_wash`) mixes it over the land and rules ink lines where the market changes. With no view chosen the strength is 0, the branch does not run, and the ground is drawn exactly as before. The figures come from the record's `rent`, `dist`, `town_wage` and `town_price` (replays) or the same values from each year of a live game, and the texture is rewritten only when the view, the good, the year or the number of years played changes.
- **Light.** A low warm sun with shadows, soft sky light, screen-space ambient occlusion, distance haze, and an optional tilt-shift depth of field for the miniature look.

## The interface

All of it is Bevy UI in one paper-and-ink style (`src/ui.rs`, `src/family.rs`, `src/game.rs`, `src/overlay.rs`): cards with rounded corners and soft shadows, the Fredoka font embedded in the binary, and small icons drawn at startup from distance shapes (`src/icons.rs`), so there are no image files. The charts of the Markets panel are painted into images on the CPU (`src/chart.rs`, 1 to 2 ms each) when the panel is rebuilt, which is when the tab, the year or the selection changes. In a live game `game::Now` says where the clock stands: Present (orders land here), Turning (the next year is worked out but not yet shown; nothing is sent) or Past; the next year is asked for as late as will still have it ready (`lead_years` in `game.rs`, from how long the last year took). Bevy 0.20 deprecates the `Interaction` component these buttons use; moving to its picking and widget crates is a job for a later upgrade.

## What is next for the look

- Construction with a puff of dust; people on the streets; weather and cloud shadows.
- A title screen with a list of saved games, and a saved game that does not have to play its years again (a snapshot beside the log of orders).
- Building the scenery per chunk and caching it, so a new year rebuilds only what changed (it is already built off the main thread, about 0.6 to 0.8 s a year in a debug build).
