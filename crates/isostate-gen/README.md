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
(`isostate-viewer/src/scatter.rs` and `scatter/towns.rs`); a generator decides
what one thing looks like.

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
  banks and cuttings; ice on `water.river`; and nothing on a fire or what is
  hot over one (`iron.glow`, `iron.plate`, `iron.converter`, `smoke.soot`,
  `smoke.stack_top`, `roofs.coke_ovens`, the charcoal clamp), nor on what is
  trodden clear every day (`ground.road_gravel`, `ground.lane_dust`, the
  railway's ballast and rails), so a bridge's roadway keeps its colour between
  parapets that carry snow. So a furnace's
  throat drawn in soot keeps clear of snow and one drawn in a roof's slate
  would not. A new `[ground]` token must be named in `mark_for_winter` or the
  crate does not build; a new token for something hot or wet must be added
  there by hand. Never write an alpha of your own into `solid` or `flat`.
- **Append only.** A generator adds to the buffers it is given and never
  clears or reads them: the game hands it the one buffer all the island's
  houses share.

## Units and conventions

- **One unit is one cell of the map, 2 km.** A house plot is 0.05 by 0.036; a
  storey of a house is 0.0205 (of a works 0.022, of the towns' other buildings
  0.024, `parts::STOREY`); a full-grown tree is about 0.11 tall.
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

## What is here

| file | what |
| --- | --- |
| `mesh.rs` | `MeshBuf` and its shapes: `tri`, `quad`, `quad_toward` and `tri_toward` (a colour at each corner, turned to face a way: what the houses are built of), `block`, `gable`, `frustum` (columns, cones, spires, hipped roofs), `ball`, `beam`, `ribbon`; `append`, `merge`, `transform`, `bounds` |
| `colour.rs` | `Colour` (a token), `Rgba`, `shade`, `lerp`, `pick`, `hex` |
| `palette.rs` | `Palette`: the named colours, read from the look file |
| `hash.rs` | `hash2`, `Seed` |
| `site.rs` | `Site` (where a thing stands), `Model` (the four buffers) |
| `parts.rs` | `gabled`, a plain building (the barns and sheds of a farmstead), and `STOREY`, the storey of what is neither a house nor a works |
| `houses/` | `cottage`, `townhouse`, `terrace`, `villa`, `farmhouse`, each at three levels of detail; `kit.rs`, the parts they are put together from; the density at which each adds a storey, `shop_at` and `villa_plot`; and `roof.rs`: what a house is roofed with by when and where it was built, when it is roofed again, how it weathers, and how each covering lies (see "Houses") |
| `works/` | `works` (seventeen kinds, at any size, of any founding year, in five states), `footprint`, `rung` and `rung_size`, `flag_point`, `plume`; `pit` |
| `farm.rs` | `farmstead`: a `houses::farmhouse` looking into its yard, the barn, a cart shed, a rick |
| `town/` | `church` (in its walled churchyard; `Church` also says where it stands off a market place and what ground it keeps clear), `station`, `market_cross`, `market_hall`, `lighthouse` |
| `trees.rs` | `tree` (broadleaf, conifer), `hedge_tree`, `bush`; `Greenery`, `in_leaf`, `leaf_group` |
| `bridge.rs` | `road_bridge`: stone, with open arches, parapets, ramps and cutwaters; as long as the river is wide where its road crosses, which the viewer works out for a road that crosses aslant |
| `fence.rs` | `fence`: a family's post-and-rail fence along a line of ground points |
| `scenes.rs` | small scenes for the lab: a street, a market place, a patch of wood |
| `sheets.rs` | the lab's contact sheets |

The scenes follow the viewer's layout rules by hand (plot spacing, set-backs,
the densities at which a plot is built). They are not what the game runs: a
change to the layout in `scatter/towns.rs` is not seen in the lab until
`scenes.rs` is changed to match. The same goes for the patch of wood: it is
an even stand of trees, while a wood in the game is filled in by `scatter.rs`
from the year's land (`isostate-viewer/src/world.rs`, "the lie of the land"),
thick and tall in its depths, thinner toward its edge, with glades and rides,
stands of taller and lower trees, conifers gathered on high and poor ground,
clumps and lone trees in the fields beside it, and a tree line below the
mountain's summer snow. The `woods` sheet shows the trees; the game's standard
shots show the woods.

## Houses

A house is put together from the parts in `houses/kit.rs`, as a works is
from `works/kit.rs`. (The two kits are separate: a works' parts know its
state, a house's its level of detail and its age. They share `MeshBuf`, the
palette, `Site` and the works' `plume`.) A kind of house is a few lines that
place parts on its plot:

- **A `Range`** is a rectangle of the plan with a way round (`a` along its
  ridge, `c` across; turned by quarter turns). `walls` builds its four
  sides, each a `Side`: not drawn (a party wall), blank, or with openings;
  dimmed where a neighbour stands close; darker toward a corner it runs
  into (the angle of a wing). A wall is darker at its foot and under its
  eaves: that is the ambient occlusion, baked into the corners' colours.
- **Openings are placed by rule** (`Opening`: its middle along the wall, its
  foot, its size, and what it is: `Sash`, `Casement`, `Door` with a hood or a
  fanlight, `Shop`, `Arch`). Close to, the wall is built in strips round a
  hole for each, a storey at a time, and the window or door is let into the
  hole; from the play distance the same list is flat panels on a whole wall.
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
  everything (about 350). The viewer chooses by where the camera is
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
sheet, and a call from `scatter/towns.rs`. Take every choice the house
makes for itself from `seed.pick(n)`, a different `n` for each.

The sheets: `cottages`, `cottages-eras`, `townhouses`, `townhouses-eras`,
`terraces`, `terraces-slope` (the ground is built too, as a bank), `villas`,
`house-backs` (seen from their yards), `house-levels` (the three levels
side by side, with their triangles), `roofs`, `farmsteads`; and the scenes
`street` and `market` (every house in full), `street-play`, `street-far`
and `market-play` (at the level the game draws from that distance: the play
picture of `street-play` is what the game shows).

## Works

A works is a compound in a yard of its own, and the one generator with a
life: `works::works(solid, smoke, flat, site, &Works { .. }, pal)`.

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
- **`seed`**: settles what the works chooses for itself.

`footprint(kind, size)` is the yard's width along the front and its depth.
The yard's ground, wall and gate are part of the works, so it reads as a
works on works ground, in a street or in a field. `flag_point` is the top of
its highest gable, for the owner's flag. `plume` is the smoke: one soft shape
with shared corners whose alpha thins downwind (the viewer's smoke material
blends), leaning with `WIND`.

To add a kind: a line in `Kind` (and in `of_method`, `method`, `label`,
`plot` and `ALL`), a function in a file under `works/` that places parts
from the kit in the yard (`c.hw`, `c.hd` are half its width and depth; `c.g`
is how much bigger than standard the buildings are drawn), a line in
`build`, and a sheet (`works_sizes(Kind::..)`). Lay it out at the standard
size, then make every count come from what fits (`c.fit`, `c.spread`), so
that it fills its yard at 100 and still fits at 0.4: the tests build every
kind at every size, founding year and state and check that the footprint
holds it.

The sheets: `works-kinds` (every kind at size 3), `works-small` and
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
   comes from.
2. Test what must not break: that it is sound (`Model::is_sound`), that it is
   the same each time, that it stays on its plot, and whatever its parameters
   promise (three storeys are taller than two).
3. Add a sheet for it in `sheets.rs`: a function that returns rows of cells,
   and a line in `SHEETS`.

```rust
fn my_things(pal: &Palette) -> Vec<Vec<Cell>> {
    grid(2, 6, |row, col| {
        let p = MyThing { storeys: row as u8 + 1, seed: Seed::of(col as u32) };
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
4. Call it from the viewer where the thing is placed (`scatter.rs` or
   `scatter/towns.rs`), handing it the site, the slot's seed and the look
   (which stands for its palette).
5. Render the sheet and look at it at all three distances. Play distance is
   the one that matters most: it is where the game is played.

## Changing a generator the game already uses

The standard shots show what a change does to the game's picture:

    ./target/debug/isostate-viewer.exe --shots out/after --shots-compare out/before

A move or a tidy-up should leave every shot the same to the pixel. A change
to how a thing looks should change only the shots that thing is in.
