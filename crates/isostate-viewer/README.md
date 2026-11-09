# isostate-viewer

A 3D replay viewer for Isolated State runs, in Bevy 0.20. It reads a `run.bin` written by the Python package (`python -m isostate run ...`) or by the Rust core, and draws the island as a small diorama you can scrub through from 1750 to 1900.

![Kirkford, 1900](../../docs/shots/kirkford-1900.jpg)

## Running

```bash
cargo run -p isostate-core --release -- run --seed 7 --regime landlords --out runs/rust7   # or the Python package
cargo run -p isostate-viewer --release -- runs/rust7/run.bin
```

The first build compiles Bevy and takes a while. On Linux you need `pkg-config`, `libwayland-dev` and `libxkbcommon-dev` to build; Windows and macOS need nothing extra.

| Key or mouse | Does |
|---|---|
| Space | play and pause (from 1750 if at the end) |
| Left / Right | step a year (Page Up / Page Down: ten) |
| Up / Down | faster or slower playback |
| Left drag, WASD | pan |
| Right drag, Q / E | turn and tilt |
| Wheel, R / F | zoom |
| T | tilt-shift (miniature focus) on and off |

Options: `--year 1846` opens on a year; `--size 1600x1000` sets the window.

### Screenshots without a window

Useful for checking the look on a machine with no GPU (it renders in software, so expect a few minutes):

```bash
WGPU_BACKEND=vulkan cargo run -p isostate-viewer -- runs/landlords/run.bin --size 1600x1000 \
    --view out/kirkford.png,1900,77.5,47.5,25,38,10 \
    --view out/island.png,1900
```

Each `--view` is `path,year,x,z,yaw,pitch,distance,season` in map cells and degrees; missing values fall back to the whole-island view, and `season` (0 midwinter, 0.25 spring, 0.5 summer, 0.75 autumn) overrides `--season` for that shot. `--wait 100` sets how many seconds to wait for shaders to compile before the first shot.

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
- **Things on the ground.** Every possible tree, house, farmstead and mine has a fixed slot with a fixed threshold, and it appears once its cell's share or density passes that threshold. Growth is therefore smooth and stable: nothing reshuffles from one year to the next, towns thicken in place and woods thin tree by tree.
- **Works.** Each firm in the replay has a stable plot in its cell, first come first served, and its building depends on its method (mills, furnaces, kilns, sheds) and grows in tiers as its capacity doubles. Chimneys of working firms smoke.
- **Towns.** A church on the market square, a station once the railway reaches the town, a lighthouse for harbours.
- **Lines.** Rivers, roads and railways are ribbons draped over the terrain through jittered cell centres; lines being built are dashed.
- **Light.** A low warm sun with shadows, soft sky light, screen-space ambient occlusion, distance haze, and an optional tilt-shift depth of field for the miniature look.

## What is next for the look

- Houses that line up along the painted streets instead of a jittered grid.
- Moving trains and boats, construction that pops up with a puff of dust, and click-to-inspect with each works' "why here" sums.
