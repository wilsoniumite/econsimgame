# isostate-gen

The generators of Isolated State: the functions that turn what the simulation
knows (a plot built up to some density, a works of a kind and a size, a farm
among arable fields) into buildings and trees.

There is no Bevy in this crate. It depends on glam at the version Bevy uses,
so a `Vec3` here is the viewer's `Vec3`, and on serde and toml to read the
palette. It builds in a couple of seconds and its tests run without a window:

    cargo test -p isostate-gen

The viewer calls these functions to draw the island, and its lab lays them
out as contact sheets (see "Looking at a generator" below).

## What a generator is

A plain function. It is handed the buffers it writes into, the site it stands
on, a small struct of parameters with the slot's seed in it, and the palette.
It appends triangles and does nothing else.

```rust
pub struct Cottage {
    pub plot_width: f32,
    pub plot_depth: f32,
    pub storeys: u8,
    pub roof: Roof,
    pub era: Era,
    pub upland: bool,
    pub neighbours: (bool, bool),
    pub detail: Detail,
    pub seed: Seed,
}

pub fn cottage(solid: &mut MeshBuf, smoke: &mut MeshBuf, green: &mut Greenery, site: &Site, c: &Cottage, pal: &Palette)
```

The viewer decides where things stand and which stand this year
(`isostate-viewer/src/scatter.rs`, `scatter/towns.rs` and
`scatter/towns/kept.rs`); a generator decides what one thing looks like.

## The rules

- **Pure.** The same inputs give the same triangles, every time and on every
  machine. No clock, no random numbers, no globals, nothing read from disk.
- **Deterministic by seed.** Everything a thing chooses for itself (its wall
  colour, how wide it is, whether it has a chimney) comes from its `Seed`: two
  numbers from 0 to 1 that the slot it stands on keeps for good. The same
  slot draws the same house in every year, so a town changes only where the
  simulation changed it. `hash2` is the island's one hash; `Seed::of(n)` makes
  a seed for a sheet or a test.
- **Honest.** Draw only what the parameters say. A parameter is something the
  simulation knows (storeys from density, a works' tier from its capital) or a
  fixed choice of the slot. Do not invent state.
- **No Bevy.** Maths only: `glam`, `MeshBuf`, the palette.
- **Colours by token.** Take every colour from the palette by name
  (`*pal.brick.red`, `pal.walls.town_house(v)`). No colour literals in a
  generator. A new colour is a new token: add it to
  `crates/isostate-viewer/assets/look.toml` and to `palette.rs`, named for
  what it is in the world, not for where the code uses it. `shade(c, k)`
  darkens or lightens a token for a face.
- **Use a token for what it is named for: winter goes by it.** Nothing of
  winter is built into a mesh. The viewer's material for what is built
  (`solid`, and `flat`) lays winter on every face that looks to the sky, and
  what it lays there is said by a mark each token carries in its alpha
  (`colour::settles`; `Palette::mark_for_winter` sets them, `shade` and
  `lerp` carry them along, and a solid mesh has no other use for alpha):
  snow on anything unmarked (a roof, a wall's top, a stack of goods); frost,
  and its green gone dull or to bare earth, on the `[ground]` tokens and what
  grows on them (`foliage.hedge`, `foliage.vegetables`) and on a railway's
  banks and cuttings (the stone pitching of a bank by the sea among them,
  `rail.sea_wall`); ice on `water.river`; and nothing on a fire or what is
  hot over one (`iron.glow`, `iron.plate`, `iron.converter`, `smoke.soot`,
  `smoke.stack_top`, `roofs.coke_ovens`, the charcoal clamp), nor on what is
  trodden clear every day (`ground.road_gravel`, `ground.lane_dust`, the
  railway's ballast and rails), so a bridge's roadway keeps its colour between
  parapets that carry snow. So a furnace's
  throat drawn in soot keeps clear of snow and one drawn in a roof's slate
  would not. A new `[ground]` token must be named in `mark_for_winter` or the
  crate does not build; a new token for something hot or wet must be added
  there by hand. Never write an alpha of your own into `solid` or `flat`,
  and mix two tokens with `lerp`, not by hand: the sheets' tests fail a cell
  with a corner there whose alpha is none of the four marks. (How white the
  snow is, how much of a roof it covers and how strong the frost is are the
  viewer's, in the look file's `[winter]` table.)
- **Append only.** A generator adds to the buffers it is given and never
  clears or reads them: the game hands it the one buffer all the island's
  houses share.

## Units and conventions

- **One unit is one cell of the map, 2 km.** A house plot is 0.05 by 0.036; a
  storey of any building is 0.0205 (`scale::STOREY`: see "The scale"); a
  full-grown tree is about 0.11 tall.
  Nothing is to scale with the map (a house 100 m wide), but things are to
  scale with each other. Up is +y, with heights exaggerated as on a model.
- **The site's origin is the middle of the thing's footprint, on the ground.**
  Build upward from it. Walls should reach a little below the origin (`block`
  takes a `sink`) so that sloping ground shows no gap.
- **+x runs along the front, +z is the front** (where the door is). `site.yaw`
  turns the thing about the upright: at a yaw of nothing the front faces
  south, toward the camera as a game opens.
- **Write corners where they stand on the map**, not round nought to be moved
  afterwards. `site.at(dx, dz)` gives the point at an offset in the thing's
  own frame, on the ground there: use it for parts that stand apart (a barn
  across the yard, a chimney beside a mill).
- **Begin a quad at a corner where no two of its points meet.** `quad` takes
  the face's facing from its first corner and the corners either side of it.
  A face that tapers to nothing at one end (a parapet at the foot of a ramp)
  has two points in one place there; begun at that end it has no facing and
  is drawn unlit, a dark wedge on a pale thing. Begin it at the other end.
  The sheets' tests look for such faces in every cell of every sheet.
- **Four kinds of buffer**, because the viewer draws each with its own
  material: `solid` (walls, roofs, everything built), `smoke` (lets light
  through, casts no shadow, and is blended: a corner's alpha is how solid the
  smoke is there), `green` (trees and bushes) and `flat` (what lies on the
  ground: lanes, paving, a works' yard, water). A generator takes only the
  ones it writes. `Model` holds one of each for the lab and for tests.
- **`green` is a `Greenery`: three buffers**, as the seasons need. `evergreen`
  holds trunks, conifers and bushes, which stay as they are. `leaves` holds
  the crowns of the broadleaf trees, which the viewer tints through the year
  (gold in October) and does not draw while they are down. `bare` holds
  their winter limbs (boughs and the lesser limbs that fork from them, each
  one flat blade seen from both sides), drawn only then. A
  broadleaf tree writes all three every time, so that the viewer changes the
  season by showing and hiding and never builds scenery again for it:
  `trees::in_leaf(phase, group)` says which is to be seen, and the island's
  trees are in `LEAF_GROUPS` groups that turn one after another.
- **A thing that knows its age takes it as a parameter**, in the steps it
  shows. A house is handed its `Roof` (what it was covered with the year it
  was built or last roofed, and how weathered that is now: `houses::Roof::of`),
  not the year: the viewer knows when a plot was first built and what could
  be had there then.
- **Stay on the plot.** A house keeps inside its plot (the eaves may overhang
  by a hair), the ground behind it that the towns leave for its yard
  (`houses::BACK_ROOM`), and the strip before it that its path crosses to the
  street (`houses::SET_BACK`). A works keeps inside its yard, which is as wide and as deep as
  `works::footprint(kind, size)` says: the viewer sets ground aside by that
  and nothing else, so anything outside it may stand in a neighbour's yard.
  The tests check both. (Smoke is free to drift.)

## The scale

Every building answers to one scale of detail. It was read off the houses
(`houses/`), which are the reference: bring a new thing to them, never them
to it. The numbers live in `scale.rs`, and a test builds every generator at
every level and measures it against them (see "Holding the line" below).

**Units.**

- **One storey is 0.0205** (`scale::STOREY`), floor to floor, for every
  building: a house, a mill, a station, a granary. Rows of windows stand a
  storey apart. Two things have a reason not to be a whole number of storeys
  high. A *hall* is one tall space (a forge, a rolling mill, an engine
  house, a casting house, a threshing barn, a nave): it is as high as its
  work needs, and its openings are still sized and spaced by the storey and
  the bay. A thatched cottage's eaves stand at 0.92 of a storey, under the
  deep thatch.
- **One bay is 0.014** (`scale::BAY`), the module openings repeat at along a
  wall. Bays run from one module to one and a half: a cottage is two or
  three bays, a townhouse three, a mill as many as its length holds. A
  building grows by bays and storeys, never by drawing its windows bigger.
- **Openings.** A window is about 0.0065 wide and 0.007 tall, its sill 0.0075
  above its floor; a door 0.0052 by 0.0126; a cart or wagon door 0.012 to
  0.02 wide. The floor stands 0.0026 above the ground (`kit::FOOT`), which
  is also the height of the plinth.
- **A roof oversails** its walls by 0.0028 at the eaves and 0.002 at a
  gable's verge under slate (0.003 and 0.0022 under tile, 0.0042 and 0.003
  under thatch: `houses::roof::Cover::lie`), and its edge is as thick as the
  covering (0.0009 slate to 0.0034 thatch).

**The three levels** (`Detail`, chosen for each building by the viewer's
`scatter::Lod` from where the camera is). The numbers are what the houses
measure, with a margin (`scale::LEVELS`): the smallest feature is the least
width of any triangle of the thing; wall is the triangles the kit built to
one bay of one storey of wall, roof the triangles to one bay by one bay of
roof.

| level | camera | what is drawn | smallest feature | wall | roof |
| --- | --- | --- | --- | --- | --- |
| far | beyond 40 | walls and roofs: the shell, its wings and lean-tos, tall chimneys as plain shafts, the great heaps | none finer than 0.0012 | 0.02 to 1.5 | 0.03 to 2.8 |
| play | 4 to 40 | openings as flat panels 0.0006 proud of the wall, stacks and chimneys, the yard's wall as one run, what stands in the yard as plain blocks, the colour of a garden | 0.0012 to 0.006 | 0.6 to 4 | 0.04 to 3.2 |
| near | 4 and nearer | openings let 0.0013 into the wall with jambs, sills and glazing bars; eaves with an edge; stacks with oversailing heads and pots; copings, piers, gates; what stands in the yard in full | 0.00035 to 0.001 | 4 to 24 | 0.1 to 8 |

(The houses measure: far 0.1 to 0.9 and 0.1 to 1.9; play 0.9 to 2.5 and
0.4 to 2.1; near 6.4 to 15.7 and 1.0 to 5.1. A cottage is 14, 70 and 200
to 450 triangles. The wall bands are the houses' with a margin either side.
The roof bands reach lower than the houses' because a roof's detail is at
its edges, its ridge and what stands on it, and a works' roof is many
times the size of a cottage's: a wide plain roof is right.) At the play
and near levels no wall of most of a storey's height may run more than four
modules (`scale::BLANK`) with nothing on it or before it: an opening, a
pier, a lean-to, a stack against it. A thing that is not a building (a
bridge, a pit's heap, a tree) answers only for the first of these: nothing
in it finer than its level allows.

Only what goes into `solid` and `smoke` may follow the level. What goes
into `flat` (a yard's ground, water) and `green` (a garden's tree) is the
same at all three, because when the camera alone has moved the viewer puts
the buildings and the smoke on screen again and leaves the rest.

**Treatments that must match across kinds.** All of them are in `kit.rs`,
which every building is put together from, so a building gets them by
using the kit and not by copying numbers.

- *How an opening sits in a wall.* Far: not drawn. Play: a flat panel on a
  whole wall (`Kit::walls` with a `Side` of `Opening`s). Near: the wall is
  built in strips round a hole, a storey at a time, and the window stands
  0.0013 back with painted jambs, a sill and one bar.
- *Eaves and verges.* `Kit::roof` oversails by what the covering asks and,
  close to, has an edge of the covering's thickness. No roof stops flush
  with its wall unless it stops against a neighbour (`End::Party`).
- *Feet on slopes.* Walls reach below the origin to the lowest corner of
  their range and a little more (`Kit::drop`); close to they have a plinth
  band of `FOOT`. Yard walls and what stands about follow the ground.
- *Wobble and crisp lines.* A building from before 1720 leans and its ridge
  dips (`Kit::crooked`), less so to 1815, and not at all after: new work is
  square and straight.
- *Baked shade.* A wall is darker at its foot (0.78 of its colour; 0.7
  rising to 0.96 over the plinth close to), under its eaves (a band of
  0.003 falling to 0.72), toward an inside corner (0.74 over 0.012) and
  where a neighbour stands close (0.84); the four sides of anything take
  1, 0.93, 0.86 and 0.95 of its colour, front, right, back, left; a roof's
  far slope takes 0.82.
- *Weathering by age.* Roofs weather in four steps toward moss
  (`houses::roof`); a derelict works is dulled and greened (`works`'
  `tone`). Never a colour of its own.
- *Palette families.* Walls, roofs, brick, stone, slate, timber and iron
  come from the palette's families by token; a kind of building picks from
  a family, it does not mix a colour.

## What is here

| file | what |
| --- | --- |
| `mesh.rs` | `MeshBuf` and its shapes: `tri`, `quad`, `quad_toward` and `tri_toward` (a colour at each corner, turned to face a way: what the houses are built of), `block`, `gable`, `frustum` (columns, cones, spires, hipped roofs), `ball`, `beam`, `ribbon`; `append`, `merge`, `transform`, `bounds` |
| `colour.rs` | `Colour` (a token), `Rgba`, `shade`, `lerp`, `pick`, `hex` |
| `palette.rs` | `Palette`: the named colours, read from the look file |
| `hash.rs` | `hash2`, `Seed` |
| `site.rs` | `Site` (where a thing stands), `Model` (the four buffers) |
| `kit.rs` | the parts every building is put together from: ranges with walls and openings, roofs, lean-tos, dormers, bays, stacks, and what stands in a yard (see "Houses"); `Detail` and `Era` |
| `scale.rs` | the scale: `STOREY`, `BAY`, what each level may and must draw (`LEVELS`), `measure` and `faults`, and the `Tally` the kit keeps of the walls and roofs it builds |
| `registry.rs` | every generator, with specimens of each, and the tests that hold them to the scale |
| `houses/` | `cottage`, `townhouse`, `terrace`, `villa`, `farmhouse`, each at three levels of detail; the density at which each adds a storey, `shop_at` and `villa_plot`; and `roof.rs`: what a house is roofed with by when and where it was built, when it is roofed again, how it weathers, and how each covering lies (see "Houses") |
| `works/` | `works` (seventeen kinds, at any size, of any founding year, in five states, at three levels of detail), `footprint`, `rung` and `rung_size`, `flag_point`, `plume`; `pit`; `kit.rs`, the works' own parts, built on the kit, and `plant.rs`, the furnaces, lifts and kilns |
| `farm.rs` | `farmstead`: a `houses::farmhouse` looking into its yard, the threshing barn with its porch and great doors, a cart shed or a byre, a granary on staddle stones, a rick |
| `town/` | each at three levels: `church` (in its walled churchyard; `Church` also says where it stands off a market place and what ground it keeps clear), `station`, `market_cross`, `market_hall`, `lighthouse` |
| `rail.rs` | each at three levels, what stands by a railway where the ground was too steep for an ordinary line: `winding_house` (the stationary engine at the head of an inclined plane, before 1850), `banker_shed` (the banking engine's shed at the foot of a steep bank, after), and `buffer_stop`; `WINDING_HOUSE` and `BANKER_SHED` are the ground each keeps, and `TRACK_CLEAR` how far before its front the track runs |
| `trees.rs` | `tree` (broadleaf, conifer), `hedge_tree`, `bush`; `Greenery`, `in_leaf`, `leaf_group` |
| `bridge.rs` | `road_bridge`, at three levels: stone, with open arches, parapets, ramps and cutwaters; as long as the river is wide where its road crosses, which the viewer works out for a road that crosses aslant |
| `fence.rs` | `fence`: a family's post-and-rail fence along a line of ground points |
| `scenes.rs` | small scenes for the lab: a street, a market place, a patch of wood |
| `sheets.rs` | the lab's contact sheets |

The scenes follow the viewer's layout rules by hand (plot spacing, set-backs,
the densities at which a plot is built). They are not what the game runs: a
change to the layout in `scatter/towns.rs` is not seen in the lab until
`scenes.rs` is changed to match. And they follow the older rule, by each
cell's density, which only a run under the port's rules still uses: since
12 November a game's towns are laid out from the ground they keep
(`scatter/towns/kept.rs`), in closes and turns the scenes know nothing of.
The same goes for the patch of wood: it is
an even stand of trees, while a wood in the game is filled in by `scatter.rs`
from the year's land (`isostate-viewer/src/world.rs`, "the lie of the land"),
thick and tall in its depths, thinner toward its edge, with glades and rides,
stands of taller and lower trees, conifers gathered on high and poor ground,
clumps and lone trees in the fields beside it, and a tree line below the
mountain's summer snow. Where a wood's edge runs is the land's own rule too
(two smooth fields of the year, `YearLand::field`), not a generator's. The
`woods` sheet shows the trees; the game's shots show the woods, and the far
views of the land have a set of their own, which is what to look at after a
change to the land's rules or to the ground shader:

```bash
./target/debug/isostate-viewer.exe --land-shots out/land --shots-compare out/land-before
```

Five cameras chosen from the run (the whole island, wooded lowland from 8 km
up, a hill margin, the country round the home town, the snow line), each in
the summers of 1800 and 1900, in October 1800 and in midwinter 1900: twenty
pictures at most, in about 40 seconds (island 7 as it stands on 12 November
has no hill margin for the third camera to find, so it takes sixteen), on
the standard shots' machinery, so
`--shots-compare` and `--compare-shots` work on them. It prints where each
camera looks, for pointing a `--view` at the same country. The pictures say
whether the land looks right; `ISOSTATE_AREAS=1` says whether it is still the
simulation's (each use as painted against the record, over the island and
over districts of 12 km, and how much of it changes from one year to the
next).

## Houses

A house is put together from the parts in `kit.rs`, as every building is
(a works' own parts, in `works/kit.rs`, are built on them and add the
works' state). A kind of house is a few lines that
place parts on its plot:

- **A `Range`** is a rectangle of the plan with a way round (`a` along its
  ridge, `c` across; turned by quarter turns). `walls` builds its four
  sides, each a `Side`: not drawn (a party wall), blank, or with openings;
  dimmed where a neighbour stands close; darker toward a corner it runs
  into (the angle of a wing). A wall is darker at its foot and under its
  eaves: that is the ambient occlusion, baked into the corners' colours.
- **Openings are placed by rule** (`Opening`: its middle along the wall, its
  foot, its size, and what it is: `Sash`, `Casement`, `Door` with a hood or a
  fanlight, `Shop`, `Arch`; and for the buildings that are not houses
  `Paned`, a window of small panes, round-headed or not; `Leaves`, a pair
  of boarded doors wide enough for a cart; `Mouth`, an opening with nothing
  in it, dark or lit by a fire; `Louvre`). Close to, the wall is built in
  strips round a hole for each, a storey at a time, and the window or door
  is let into the hole; from the play distance the same list is flat panels
  on a whole wall. A `Side::free` wall is cut round openings of any height
  wherever they stand (a hall's tall doors and windows), in level bands.
- **`roof`** fits a pitched roof to a range, each end a gable, a hip, a
  half-hip, stopped against a party wall, or run on into another roof with
  valleys (`End::Valley(run)`: a wing whose eaves are level with its
  house's). `gablet`, `dormer`, `lean_to` (an outshut, a catslide, a porch),
  `bay`, `stack` (with its pots; it gives back where its smoke rises).
- **Behind and about:** `patch` (a garden, a flagged yard, a path),
  `low_wall` (a wall, a hedge, a paling), `shed` (a privy), `butt`,
  `woodpile`. All follow the ground the site hands in.
- **`crooked`**: every corner goes through `Kit::pt`, where an old house's
  walls lean and its ridge dips, so the whole house goes crooked together.

What a house is handed, besides its plot, storeys, roof and seed:

- **`era`** (`Era::of(year)`): the step its style takes from the year its
  plot was first built (a townhouse: the year its ground was first dense
  enough for one). Before 1720 timber frames, jetties, casements, gables to
  the street; to 1815 wash or stone, sashes, hoods and fanlights, parapets;
  to 1860 brick and slate; after, bay windows and L-shaped villas.
- **`upland`**: the hills build in their own stone, to the ground, under
  stone slates darker than the walls; four in ten of the houses that stand
  apart are limewashed over the stone, and a row is bare stone throughout.
- **`neighbours`**: whether a house stands on the next plot to its left and
  right as it faces the street (verges, hips and dim side walls follow).
- **`shop`** (a townhouse): `houses::shop_at(density, seed)` says where.
- **`detail`** (`Detail::Far`, `Play`, `Near`): how much of it to draw. Far:
  walls and roof (a cottage is 14 triangles). Play: openings as flat panels,
  stacks, wings, outshuts, the colour of the garden (about 70). Near:
  everything (200 to 450). The viewer chooses by where the camera is
  (`scatter::Lod`); the three must be the same house, and each kind's tests
  build all three. Only what goes into `solid` and `smoke` may follow the
  level: a garden's tree is the same tree at all three, because when the
  camera alone has moved the viewer puts the houses and their smoke on
  screen again and leaves the island's trees as they are.

A terrace is handed the ground (`Site::on`) and steps with it: houses whose
ground is near enough level share a floor and are built as one run; where
the floor changes the higher run shows its gable. The other houses take up
a slope with their walls (`Kit::drop`).

To add a kind of house: a struct with the inputs above and a `new` that
fills in the usual ones, a function that places parts (`cottage.rs` is the
plainest to copy), the tests (it keeps to its plot at every level, era and
seed; each level draws more than the last; far is a shell and a roof), a
sheet, and a call from `scatter/towns.rs` and from `scatter/towns/kept.rs`
(the towns of a game). Take every choice the house
makes for itself from `seed.pick(n)`, a different `n` for each.

The sheets: `cottages`, `cottages-eras`, `townhouses`, `townhouses-eras`,
`terraces`, `terraces-slope` (the ground is built too, as a bank), `villas`,
`house-backs` (seen from their yards), `house-levels` (the three levels
side by side, with their triangles), `roofs`, `farmsteads`; and the scenes
`street` and `market` (each picture at the level the game draws from its
distance, as every sheet's is), and `street-play`, `street-far` and
`market-play` (one level in every picture, to see it close up).

## Works

A works is a compound in a yard of its own, built of the same parts as a
house (`kit.rs`) and of its own (`works/kit.rs`), and the one generator
with a life: `works::works(solid, smoke, flat, site, &Works { .. }, pal)`.

- **`kind`**: the method the firm works by (`Kind::of_method`). Shops and
  trades (the simulation's "services") are a kind like any other.
- **`size`**: capacity over the method's standard capacity (K over K0), from
  0.4 (anything smaller is drawn as 0.4) to about 100. A works grows by
  adding parts (bays, storeys, furnaces, chimneys, kilns, rows of sheds), and
  its buildings grow only a little (`size` to the power 0.1), so a great
  works is many buildings and not one large one. The viewer hands in a size
  from a ladder, two rungs to a doubling (`rung`, `rung_size`), each works
  with a phase of its own, so that a works changes in steps that can be seen
  and the works of a town do not all step in the same year.
- **`founded`**: the year sets the style for good. Before 1790 stone, stone
  slates and timber, and a waterwheel where `water` is set (a powered mill
  then has its pond in the yard); then brick and slate; from 1840 iron and
  glass (north lights, lanterns, iron-cased furnaces, a continuous kiln).
- **`state`**: `GoingUp` (walls part of the way up, scaffolding, no roofs,
  no smoke), `Extending` (at work, scaffolding round its newest part),
  `Working`, `Idle` (whole, fires out, gate shut) and `Derelict` (roofs
  falling in, stacks broken, weeds in the yard). The parts in `works/kit.rs`
  know the state, so a kind does not have to: it says what stands where.
- **`water`, `rail`**: what the caller knows of the site. `rail` is taken
  and not yet drawn.
- **`detail`**: the level it is drawn at, chosen by the camera as a house's
  is. Far: walls, roofs, the tall chimneys, the furnaces and the great
  heaps. Play: openings as flat panels, stacks, the yard's wall and gate
  piers, goods and heaps, wheels, scaffolding. Near: everything. It is the
  same works at each: the yard's ground and water (`flat`), the smoke, the
  flag's place and every number the works takes for itself are the same at
  all three (a part takes as many of `c.rnd()` at every level), and the
  tests hold it to that.
- **`seed`**: settles what the works chooses for itself.

`footprint(kind, size)` is the yard's width along the front and its depth.
The yard's ground, wall and gate are part of the works, so it reads as a
works on works ground, in a street or in a field. `flag_point` is the top of
its highest gable, for the owner's flag. `plume` is the smoke: one soft shape
with shared corners whose alpha thins downwind (the viewer's smoke material
blends), leaning with `WIND`.

**The works' parts** (`works/kit.rs`, methods of `Ctx`; each knows the
state and the level). Buildings, each giving back a `Shed` to hang things
on:

- `shed`, `shed_pitched`: four walls under a pitched roof, its ridge along
  the front or across the yard. Built before 1790, its ridge sags a little.
- `lean_to` (against a taller building; `stands_free` gives it a back wall
  of its own), `open_shed` (a roof on posts), `sawtooth` (north lights,
  glazed, with their bars close to), `tower` (square, under a pyramid),
  `bank` (flat-topped: coke ovens).
- On a roof: `ridge_vent` (louvred, or a glazed lantern), `roof_lights`.

Openings, hung on a face of a `Shed` (`Face::Front`, `Back`, `EndA`,
`EndB`) and built into the wall when the works is finished
(`Ctx::finish`): `windows` (a bay and a storey apart, or tall on a hall),
`window_band` and `light` (one window of any size, round-headed or not),
`door` (a man's), `doors` (a cart's, boarded), `mouth` (an open bay or
arcade), `glow` (a fire seen in an opening, lit while the works is at
work), `shop_front`, `hoist` (loading doors one above another under a beam
and its hood), `louvre`, `board`. A stretch of wall a kind hung nothing on
is given what such a wall has (`fill`: `Fill::Windows`, the default;
`Fill::Posts` for a boarded shed, the posts and joints of its frame;
`Fill::Nothing`).

Fires and flues: `chimney` (tall and round on a square plinth, with iron
bands and a corbelled head close to), `stack` (short and square), `slab_stack`
(broad, against a shed), `flue` (a plain round one), `plume`, `throat`.

In the yard: `ground`, `patch`, `race` (these three lie flat and are the
same at every level), `wall` (the yard's wall or fence with its gate:
piers from the play distance; a coping, corner piers and the gate's leaves
close to), `heap`, `pile`, `stacks` (goods in rows), `wheel`, `sheer_legs`,
`scaffold`, `builders_yard`.

The plant of the fire trades (`works/plant.rs`, also methods of `Ctx`):
`furnace_stack` (a blast furnace built of masonry: square and battered,
with its blowing arches, string courses and tie plates), `furnace_shell`
(one cased in brick or iron: round, on the brick drum of its hearth, with
its tuyere arches, blast main and hoops), `charging_gallery` (the platform
round a furnace's throat, railed close to), `lift` (the four-legged tower
barrows go up in, braced close to), `barrow_way` (a gantry, a charging
bridge or an incline: a beam with a rail either side close to), `hoops`
(for a `flue`: a stove's, a cupola's) and `bottle_kiln` (wicket to the
front of the yard, firing holes, iron bands). Each is the plain shape from
far off, shows its openings as dark panels and one band from the play
distance, and has the rest close to. The little a kind still draws for
itself (a converter's vessel, a tunnel head, the bell of a closed furnace
top) it draws straight into `c.solid`, asking `c.far()` and `c.near()` what
the level wants; anything that more than one kind could use belongs in the
kit as a part.

To add a kind: a line in `Kind` (and in `of_method`, `method`, `label`,
`plot` and `ALL`; `registry::slot` will not compile until it has a place
there, and the tests then ask for its entry in `registry::REGISTRY`), a
function in a file under `works/` that places parts
from the kit in the yard (`c.hw`, `c.hd` are half its width and depth; `c.g`
is how much bigger than standard the buildings are drawn; a building of
storeys is `c.storeys(n)` high, a hall as high as its work needs), a line in
`build`, and a sheet (`works_sizes(Kind::..)`). Lay it out at the standard
size, then make every count come from what fits (`c.fit`, `c.spread`), so
that it fills its yard at 100 and still fits at 0.4: the tests build every
kind at every size, founding year and state and check that the footprint
holds it.

The railway's banks, viaducts, girder bridges and tunnel mouths are not
generators: they are made along a line from the difference between its level
and the ground, in the viewer (`isostate-viewer/src/rails.rs`). What stands
beside the line is here (`rail.rs`, and the station in `town/`), and the
`railway` sheet shows each by a length of track. Each stands with its front
(+z) to the track; the viewer hands it a level site at the height of the
rails, on ground that lies within a little of that height and clear of every
line (and builds a stone footing under it where the ground falls away), so a
winding house or a shed must keep to the ground its constant says: the
viewer sets that aside and nothing more. They are buildings like any other:
their walls, roofs and openings are the kit's, at the scale, they are in the
registry, and the viewer hands each the level the camera gives its place
when it builds the year's scenery (`rails::standing`), while the banks and
viaducts of a line are made once and kept.

The sheets: `scale` (see "Looking at a generator"), `works-kinds` (every kind at size 3), `works-small` and
`works-standard` (at 0.4 and 1, built early and late), `works-great` (at
30), `works-states` and `works-states-great`, and one for each kind
(`works-mill`, `works-coke-furnace`, ...) at sizes 0.4, 1, 3, 10, 30 and 100,
built in 1760 and in 1870.

## Looking at a generator

The viewer's lab draws a sheet in the game's own light, through the game's
camera, with no game:

    cargo build -p isostate-viewer
    ./target/debug/isostate-viewer.exe --lab list
    ./target/debug/isostate-viewer.exe --lab cottages --out out/lab
    ./target/debug/isostate-viewer.exe --lab all --out out/lab

It writes `NAME-close.png` (as near as the game's camera goes), `NAME-play.png`
(the distance a new game opens at) and `NAME-far.png` (about half way out to
the whole island), and an `index.html` of everything in the directory. A sheet
too long for one close picture goes over several (`NAME-close-2.png`), and so
does a play or far one where the picture asked for is small (`--size 800x600`). From
play distance a house is seven pixels wide, so the play and far pictures show
the middle of the true picture enlarged a whole number of times, each pixel a
square: what you see is exactly what the game draws, only bigger.

Each picture is built at the level of detail the game uses from its
distance (the picture's heading says which): the close picture at the near
level, the play picture at the play level, the far picture at the far
level. A sheet's function is handed that level and passes it to what it
builds. (`house-levels`, `street-play`, `street-far` and `market-play` fix
their own, to show a level from another distance.) The sheet `scale`
stands one of every generator in the registry at each of the three levels,
with its measures against the scale under it: its triangles, its finest
feature in thousandths of a unit, the triangles to a bay of a storey of
wall and to a bay by a bay of roof, and its longest bare wall in bays.

After a change to a generator the viewer rebuilds in about eight seconds and
one sheet takes about three, so edit, build, render, look is under fifteen.

Options: `--season X` (0 midwinter, 0.5 midsummer, as the game has it: at 0
the broadleaf trees are bare and snow lies on what is built, drawn in the
game's own material; at 0.8 the trees are gold),
`--turn DEG` (each thing is turned 30 degrees to its right unless told
otherwise, to show a gable as well as a front; `--turn 0` is square on),
`--loupe N` (enlarge the close picture N times), `--tilt` (the tilt-shift
lens, off in the lab), `--size WxH`, and `--look FILE` to try a palette.

## Adding a generator

1. Write the function and its parameter struct in a module of its own, with
   the rules above. Give the struct doc comments that say where each number
   comes from, and a `detail: Detail` for the level it is drawn at. Build
   its walls, roofs and openings with `kit::Kit` and size it by the scale
   (`scale::STOREY`, `scale::BAY`): "The scale" says what each level draws.
2. Test what must not break: that it is sound (`Model::is_sound`), that it is
   the same each time, that it stays on its plot, and whatever its parameters
   promise (three storeys are taller than two).
3. List it in `registry::REGISTRY`: a function that builds a few specimens
   of it at a level, and a line naming the generator functions it covers.
   Until it is listed the crate's tests fail (see "Holding the line").
4. Add a sheet for it in `sheets.rs`: a function that returns rows of cells,
   and a line in `SHEETS`.

```rust
fn my_things(pal: &Palette, detail: Detail) -> Vec<Vec<Cell>> {
    grid(2, 6, |row, col| {
        let p = MyThing { storeys: row as u8 + 1, detail, seed: Seed::of(col as u32) };
        Cell::new(format!("{} storeys\nseed {col}", row + 1), |m, site| my_thing(&mut m.solid, site, &p, pal))
    })
}

Sheet { name: "my-things", about: "My thing: storeys down the rows, six seeds across.", land: Land::Town, build: my_things },
```

   A cell is one generator call, or several placed together: a street or a
   yard tells more than a house alone. (`Cell::turned` shows a thing from
   another side; `Cell::on_slope` stands it on sloping ground, with the
   ground built as a bank.) The lab works out the spacing from what
   each cell holds, so a sheet says nothing about layout. A label is one or
   two short lines. `land` is the ground the thing stands on in the game, which
   sets the colour of the lab's ground.
5. Call it from the viewer where the thing is placed (`scatter.rs` or
   `scatter/towns.rs`), handing it the site, the slot's seed, the level the
   camera gives its place (`lod.at(x, z)`) and the look (which stands for
   its palette). Put it in a buffer the viewer swaps when the camera alone
   has moved (the houses' or the works'), or give it no levels.
6. Render the sheet and look at it at all three distances, and at `scale`
   beside its neighbours. Play distance is the one that matters most: it
   is where the game is played.

## Holding the line

Each kind of thing was once built with a kit of its own and its own idea
of how fine detail goes, and the picture swung from one to the other. Three
tests in `registry.rs` stop that happening again:

- **Every generator keeps to the scale at every level.** Each entry's
  specimens are built at far, play and near and measured (`scale::measure`,
  from the mesh and from the tally the kit keeps of the walls and roofs it
  builds). A building fails if its smallest feature is finer than its
  level allows or coarser than the level asks; if its triangles to a bay of
  a storey of wall, or to a bay by a bay of roof, lie outside the level's
  band; if at the play or the near level a wall runs more than four
  modules bare; if windows of one size stand one above another at anything
  but a whole number of storeys; if a level has less to it than the one
  before; or if it is a building and the kit built none of its walls.
- **A generator that is not in the registry fails.** The test reads the
  crate's source for free public functions that are handed a `MeshBuf`, a
  `Greenery` or a `Model` to write into, and each must be covered by an
  entry (or named, with the reason, in `NOT_GENERATORS`).
- **The houses are the reference**, and every kind of works has an entry.

`cargo test -p isostate-gen measures -- --ignored --nocapture` prints what
every entry measures at every level (`EACH=1` for every specimen), which is
how the bands were read off the houses; `timings` prints how long every
entry takes to build at every level and how long that is for each thousand
triangles (a house is 0.06 to 0.07 ms a thousand close to, and nothing
should be far off that); `fingerprints` prints a hash of every sheet, to
prove that a change that should leave some things as they were did.

## Changing a generator the game already uses

The standard shots show what a change does to the game's picture:

    ./target/debug/isostate-viewer.exe --shots out/after --shots-compare out/before

A move or a tidy-up should leave every shot the same to the pixel. A change
to how a thing looks should change only the shots that thing is in.
