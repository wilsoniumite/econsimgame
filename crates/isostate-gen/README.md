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
    pub seed: Seed,
}

pub fn cottage(solid: &mut MeshBuf, smoke: &mut MeshBuf, site: &Site, c: &Cottage, pal: &Palette)
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
- **Append only.** A generator adds to the buffers it is given and never
  clears or reads them: the game hands it the one buffer all the island's
  houses share.

## Units and conventions

- **One unit is one cell of the map, 2 km.** A house plot is 0.05 by 0.036; a
  storey is 0.024 (`parts::STOREY`); a full-grown tree is about 0.11 tall.
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
- **Four buffers**, because the viewer draws each with its own material:
  `solid` (walls, roofs, everything built), `smoke` (lets light through, casts
  no shadow, and is blended: a corner's alpha is how solid the smoke is
  there), `green` (leaves, which turn with the seasons) and `flat` (what
  lies on the ground: lanes, paving, a works' yard, water). A generator takes only the ones
  it writes. `Model` holds one of each for the lab and for tests.
- **Stay on the plot.** A house keeps inside its plot (the eaves may overhang
  by a hair). A works keeps inside its yard, which is as wide and as deep as
  `works::footprint(kind, size)` says: the viewer sets ground aside by that
  and nothing else, so anything outside it may stand in a neighbour's yard.
  The tests check both. (Smoke is free to drift.)

## What is here

| file | what |
| --- | --- |
| `mesh.rs` | `MeshBuf` and its shapes: `tri`, `quad`, `block`, `gable`, `frustum` (columns, cones, spires, hipped roofs), `ball`, `beam`, `ribbon`; `append`, `merge`, `transform`, `bounds` |
| `colour.rs` | `Colour` (a token), `Rgba`, `shade`, `lerp`, `pick`, `hex` |
| `palette.rs` | `Palette`: the named colours, read from the look file |
| `hash.rs` | `hash2`, `Seed` |
| `site.rs` | `Site` (where a thing stands), `Model` (the four buffers) |
| `parts.rs` | what houses share: `gabled`, `windows`, `chimney`, `smoking_chimney` |
| `houses/` | `cottage`, `townhouse`, `terrace`, and the density at which each adds a storey |
| `works/` | `works` (seventeen kinds, at any size, of any founding year, in five states), `footprint`, `rung` and `rung_size`, `flag_point`, `plume`; `pit` |
| `farm.rs` | `farmstead` |
| `town/` | `church`, `station`, `market_cross`, `market_hall`, `lighthouse` |
| `trees.rs` | `tree` (broadleaf, conifer), `hedge_tree`, `bush` |
| `bridge.rs` | `road_bridge` |
| `scenes.rs` | small scenes for the lab: a street, a market place, a patch of wood |
| `sheets.rs` | the lab's contact sheets |

The scenes follow the viewer's layout rules by hand (plot spacing, set-backs,
the densities at which a plot is built). They are not what the game runs: a
change to the layout in `scatter/towns.rs` is not seen in the lab until
`scenes.rs` is changed to match.

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

Options: `--season X` (0 midwinter, 0.5 midsummer, as the game has it),
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
   yard tells more than a house alone. The lab works out the spacing from what
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
