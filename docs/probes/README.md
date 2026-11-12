# Probes

Diagnoses of the simulation as it stood at commit a5ea2ea (10 November 2026), each
measured on 8 to 16 seeds by a helper working on a branch of its own, with a rough
prototype of a cure. They are evidence, kept as written. The mechanisms built from
them have their own pages under `docs/rules/`; the probe branches named in each
report (`night/dx-...`) hold the measuring code and are not for merging.

- `dx-firms.md`: why the island ends with thousands of tiny works on a sixth of the land.
- `dx-towns.md`: why towns do not grow on the map, the churn of town ground, the flat urban share.
- `dx-growth.md`: why history stops about 1800, the machinery famine, the late methods never built.
- `dx-cycles.md`: where the price swings and the farm zigzag come from.
- `dx-rail.md`: why the railway network stops, and what railways do to the economy.

Line numbers in them are those of a5ea2ea.

One later probe, of the game's rules as they stood at 63f7b46 (11 November 2026), on
seeds 1 to 8 with six families:

- `dx-combines.md`: what a works holds (nothing but plant and cash), how the chains lie
  on the map, a design and a small prototype of combined works behind `combines.enabled`,
  and what a railway or navigable water does for a works. A second helper then went
  through it as its critic (branch `night/dx-combines-review`): the measurements stood,
  some readings of them were corrected in place, and its last section has the verdict
  (build it changed: the record's row now; combined works as a concern with books for
  each stage, not as the prototype has them).

One probe is of the viewer, not the simulation, as it stood at cd9b41c (11 November 2026):

- `dx-smooth.md`: where the game's frames stall (a year's scenery handed to the renderer
  whole, a third of a second each year), and what keeps the view moving whatever the
  simulation is doing. Its branch, `night/smooth-probe`, adds a frame recorder to the
  viewer (`src/frames.rs`) that is meant to be merged. A critic ran it again and rewrote
  its stages on `night/smooth-probe-review` (section 11): the first stage is now the
  year's meshes handed over one a frame, half a day's work, with a brief to build from.
  As of 12 November that stage, with the frame recorder, was begun on `night/smooth-1`
  and paused part-way with its work committed, so neither is on `main` yet
  (`docs/BACKLOG.md`).

One probe is of the viewer against the core's second island, as both stood on `night/core`
at 859b680 (12 November 2026):

- `dx-land-view.md`: what it takes for the viewer to draw the land made by water
  (`docs/rules/land.md`). What the core hands over (a grid of heights every 500 m, river
  lines with width, level and flow, lakes, wet ground), how wrong today's picture is on it,
  what the viewer invents that becomes untrue, and a design in seven stages from "the land
  reaches the viewer" to "scarps and canyons made to read", with roads and bridges on real
  rivers between. Its branch, `night/dx-land-view`, adds a measuring example to the core
  (`examples/land_view_probe.rs`) and a probe to the viewer (`src/landview.rs`, behind
  `ISOSTATE_LAND_VIEW`) that draws the core's ground, rivers and lakes with today's code.
