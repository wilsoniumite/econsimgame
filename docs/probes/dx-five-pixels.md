# dx-five-pixels: five pixels of island-1900 that no change of the code moved

Branch `night/o-five-pixels`, from `main` at 973396e, 13 November 2026. Island 7 with six
families, the old island, the twelve standard pictures (`--shots`). Pictures and logs are on the
machine only, under `.claude/night/shots/opus/five-pixels/` and
`.claude/night/reports/opus/five-pixels/`. The viewer is the dev profile throughout. This page
changes no code: everything it measured with was put in for the measuring and taken out again.

## What was seen

When the new island's railways landed (the merge a1e1efe, stage 3 of the new land: railways
find their way by the ground), eleven of the old island's twelve standard pictures came out
the same to the pixel and the twelfth, `island-1900`, did not: five pixels at x 762 to 764,
y 870 to 878 of 1600 by 1000, by one to nine levels of 255, in farmland south of the middle of
the island. Two takes of the merged code agreed with each other. The merge was meant to change
nothing on the old island.

## In short

1. **Nothing drawn on the old island changed.** Every point of every railway line of every
   year (66 years, position and ground to the last bit, and what each point stands on) is the
   same before and after the merge, and so is the fingerprint of every mesh of every year built
   (`ISOSTATE_MESH_SUMS=1`: trees, houses, works, smoke, lanes, roads, rails). Read line by
   line, every change the merge made to `rails.rs`, `network.rs`, `trains.rs` and `main.rs` is
   behind `g.laid` (the new land) or an environment variable, and gives the old numbers on the
   old island.
2. **The five pixels are a tie.** At the centre of pixel (762, 870) two trees by a farm stand
   so close that their crowns cross, 22 km south of the middle of the island at about 57.14,
   77.79 in map cells. One is in leaf group 1 and the other in leaf group 0, and the four leaf
   groups are four separate meshes (they come into leaf and drop their leaves at different
   times). Through the island camera the two crowns' surfaces meet exactly at that pixel's
   centre: their depths, 117.196430 and 117.196433 from the camera, agree to 3 parts in a
   thousand million, which a 32-bit depth cannot tell apart. Where two things lie at one depth
   the card shows whichever of them it is given in the order that wins, and the order the
   engine hands the four leaf groups over in is not fixed from one run to the next. Before the
   merge the picture showed group 1's lighter leaf (140,192,101), after it group 0's darker one
   (134,183,96). The other four pixels move by one level, the lens's bloom and the edge
   smoothing carrying a little of the one pixel to its neighbours.
3. **Both versions give both pictures.** The code from before the merge gives the "after"
   picture whenever its timing is disturbed a little (a few lines of logging, `ISOSTATE_TIMING=1`,
   `ISOSTATE_MESH_SUMS=1`, a pause added to each year's build), and the merged code, with a pause
   of 20 ms added to each year's build, gives all twelve pictures of the set from before the
   merge (`trial-night-rules-4-and-3`) to the pixel. So the merge did not move the picture. It
   changed which way a coin usually falls on this machine.
4. **The new land's two named sets** (`newland-stage3/standard` and `/land`) are unchanged on
   this branch: all twenty-four pictures at 0 pixels.

## How the order comes to change

The engine (Bevy 0.20) draws everything that shares a material and a shape of vertex as one
batch, and keeps the meshes of a batch in an order of its own from frame to frame. The leaf
groups, the evergreens, the trains and the carts are all in the plain vertex-coloured
material, and on island 7 all of them sit in one block of the card's memory (logged from the
renderer at the shot: vertex block 2, index block 1 for every one of the year's meshes), so
they are one batch. Reading the engine's code (`bevy_render` 0.20.1, `render_phase/mod.rs` and
`camera.rs`): it takes a mesh out of a batch's order when the last thing drawn with it goes,
moving the last of the order into its place, and puts it at the end when one comes again; and
it puts the things whose mesh has changed back into the order in the order it finds them in a
hashed set, which can depend on what else changed in the same frame. A year turned on screen
replaces the trains, the carts and the year's meshes, in frames that depend on how long the
year took to build. What was measured:

- island-1900 taken three times running in one run: the same each time, so the order is
  settled when the year goes on screen and does not drift after.
- The carts' and trains' places at the shot: the same in a run that gave the old picture and
  one that gave the new; in one pair two trains had different entity numbers, in another pair
  nothing on the game's side differed at all.
- Leaving out the trains, or the carts, or both: the old pixels in every run tried (seven runs,
  with 0, 40 and 80 ms added to each build).
- Giving the trains, carts and boats a mark of their own in every corner, so that they are a
  batch of their own and the trees keep a batch without them: the old picture with 0 and 20 ms
  added, with `ISOSTATE_TIMING=1` and with `ISOSTATE_MESH_SUMS=1`; the new one with 40, 60 and
  80 ms. Not enough on its own.
- Drawing the twelve tree meshes outside the batches (Bevy's `NoAutomaticBatching` on them,
  which the engine draws one by one in a sorted order after the batches): the same pictures
  to the pixel with 0, 40, 60 and 80 ms added, so the coin is gone; but the trees are then
  drawn after everything batched, and a tie between a tree and something else can turn the
  other way. Against the set from before the merge it moves island-1750 by 37 pixels,
  island-1800 by 70, island-1850 by 19 and island-1900 by 37 (the farm's pixel among them, at
  the "after" value), largest 106 levels at one pixel where a tree now stands in front of
  something light; the other eight pictures not at all.

The step inside the engine that varies with time was not pinned down further than this.

## What it means for checking pictures

- A run of the standard pictures can differ from another run of the same code at pixels where
  two of the year's meshes tie. On the old island as it stands, island-1900's five pixels at
  x 762 to 764, y 870 to 878 are one such place: a difference confined to them, of at most
  nine levels, is this coin and not a change. Under `ISOSTATE_TIMING=1` one pixel of
  town-1850-winter and one of works-1900-close move by one level in the same way, in the code
  from before the merge and after it alike.
- The pictures from before the merge are no truer than those after it. Either set is a fair
  reference for the old island; the set a helper compares against should be one taken on the
  same main, and a five-pixel difference at that farm in island-1900 should be read as above.
- The cure is to fix the order the trees are drawn in, which moves a few dozen pixels of the
  island views once (where trees tie with other things) and then holds them. With the trees
  drawn outside the batches it takes one component on twelve entities in `main.rs`; done more
  carefully, the trees would keep a batch of their own whose order nothing else can disturb, so
  that ties between trees and the ground or the houses go as they go today. Either way the
  standard pictures must be taken again when it lands.

## The commands

```bash
# the standard pictures, against the set from before the merge
ISOSTATE_SAVES=<scratch> timeout 1700 ./target/debug/isostate-viewer.exe --shots <dir> \
    --shots-compare .claude/night/shots/trial-night-rules-4-and-3
# the fingerprint of every year's meshes, printed as each is built
ISOSTATE_MESH_SUMS=1 ISOSTATE_SAVES=<scratch> ./target/debug/isostate-viewer.exe --shots <dir>
# the two trees by the farm, close up
ISOSTATE_SAVES=<scratch> ./target/debug/isostate-viewer.exe --advance 200 --size 1600x1000 \
    --view <dir>/the-two-trees.png,1900,57.15,77.78,0,52,1.2,0.5,none
```

The whole island is seen in island-1900 from (59.5, 1.572, 49.24) in map cells, turned 0
degrees, tilted 52 degrees and 133.78 cells back, through a lens of 30 degrees; that is what a
pixel of it was traced back to the trees through.
