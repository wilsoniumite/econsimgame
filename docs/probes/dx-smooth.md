# dx-smooth: where the frames stall, why, and what keeps the view moving

Branch `night/smooth-probe` (from `rust-3d` at cd9b41c, 11 November 2026) in worktree `.claude/worktrees/wf_14ad06b7-ea5-1`, gone over by a critic on 12 November on `night/smooth-probe-review` (worktree `wf_14ad06b7-ea5-2`): section 11 says what was checked and what was changed, the stages of section 6 are the critic's, and the brief for the first stage is at the end. It adds one file to the viewer, `crates/isostate-viewer/src/frames.rs` (a frame recorder, a plan that drives a run with nobody at the window, a frame-time strip behind F3, and three cures that can be tried from the outside), and sixteen one-line hooks: `mod frames;` and `frames::add(&mut app);` in `main.rs`, and a `frames::span(..)` guard as the first line of fourteen systems. The recorder is for merging; the cures in it (`ISOSTATE_FRAMES_TRY`) are measuring code. Results: `.claude/night/reports/smooth/` (a `.csv` with a line a frame and a `.txt` summary for every run named below).

Machine: Stella's workstation (48 threads, RTX 4090, a 120 Hz screen, Vulkan). Island 7, six families, the game's rules, 1600 by 1000. **Other helpers were building and running viewers on the same machine all evening.** Every run carries its own check (the frames of a paused, still game cost 8 to 9 ms of work when the machine is quiet); where that check failed the run is marked "disturbed" and its numbers are not used for more than they can bear. The stalls reported here are ten to forty times a quiet frame, so they stand out of the noise; the fine figures (a millisecond here or there) are from the quiet runs only.

Words: a **frame's work** is what the main thread did for it (its schedules, then the extract) and then what the render thread did for it, with the wait for the screen left out. A **freeze** is one or more frames over 34 ms one after another. **W** is a frame in which a whole year's scenery went on screen, **C** one in which the houses were swapped for the camera alone, **Y** one in which a year arrived from the simulation.

## In short

1. The picture stops for about 0.37 s every time a year's scenery goes on screen. In the release build, in a window at 120 Hz with vsync, the frame of the swap takes 92 ms and the next one 280 ms (median of 22 swaps, 1750 to 1773; the two together 371 ms, the longest 403). At 1x that is 15% of the time, at 2x 33%, at 4x 63%, at 8x 85%; and 4x is the speed a game opens at today. The dev build is the same to within a tenth: the time goes in the engine and in moving memory, both built alike in the two.
2. It is not the simulation and it is not the building of scenery. Both are already off the main thread (a year takes the simulation 105 to 240 ms, a year's scenery 430 to 530 ms to build) and neither holds a frame. It is the hand-over. A year's scenery is 320 to 450 MB of mesh in eighteen pieces, all replaced in one frame. The main thread frees the old ones (11 to 17 ms), measures the new ones' bounds (6 ms), and then copies every byte for the renderer during the extract, with both worlds held still (44 to 65 ms). The render thread then packs and sends them to the card (183 to 276 ms, 0.6 ms a megabyte) and can draw nothing until it is done.
3. The second stall is a year's arrival, on the main thread: 15 to 25 ms in 1750, 70 to 80 ms after 1835, and now and then 150 to 490 ms. `game::pump` lays out works again and works out every churchyard of every year so far (40 ms a year by the 1880s), and `owner_marks` builds the flags and fences (6 ms early, 13 ms late).
4. Third: a camera move across a level of detail. The whole year is built again (half a second, off the main thread), all but the houses is thrown away on the main thread (12 to 15 ms), and 35 to 150 MB of houses is sent at once: two frames of 20 to 25 ms each early, a frame of 65 to 130 ms of work late.
5. Small ones: every step of the seasons (eight a second while the clock runs) makes the engine measure every tree again (6 ms, a frame dropped at 120 Hz); a panel opened costs 13 to 21 ms; a map view 9 to 15 ms; a year arriving rebuilds the open panel whether or not its content changed.
6. Sending "only what changed" will not work on the scenery as it is generated. Cut into squares of 8 cells, 94 to 100% of the trees' megabytes sit in squares that differ from the year before and 77 to 87% of the houses'; in squares of 2 cells, still 70 to 97% and 56 to 68%. And the trees do change: 22 to 30% of all tree corners are new every year from 1750 to 1900 (5 to 7% of the houses'). That is also a fault of the look (woods that shift every year), and the land rules can be held to it with the same tool.
7. A rough prototype of the cure, tried from outside the scenery code: the year's meshes are cut into squares of 16 cells, kept by the renderer alone, and handed over a few squares a frame under a budget of 6 MB. In the same window: a year's scenery now costs two frames of 11 ms (was 92 and 280); at 1x and 2x there is no freeze at all (longest frame 23 and 28 ms; was 285); at 4x five freezes of 35 ms in eight seconds, 2% of the time (was 63%).
8. The critic's re-run (section 11) agrees with all of the above to within a tenth, and found a cheaper first step: the year's fifteen meshes handed over as they are, **one a frame**, kept by the renderer alone. No cutting, no new entities, some twenty-five lines in `rebuild_year`. In a window at 120 Hz the longest wait for a picture falls from 317 ms to 25 at 1x and from 396 to 47 at 8x. Squares do better still (19 and 33) and are the step after.
9. The design, in stages (section 6, rewritten after the critique): (0) the recorder, and the scenery never measured for its bounds; (1) one mesh a frame under a budget, half a day's work, which removes the worst stall; (2) the year's arrival off the main thread, which is the worst stall of the late game once (1) is in; (3) the scenery in pieces of bounded size, each sent only if it changed; (4) send less (the trees above all); (5) what moves keeps to the frame's clock; (6) a sign when the island is catching up.

## 1. How a frame is made today

What the first look had right, and what it had wrong or left open, checked against the code and the engine's source (`~/.cargo/registry/src/*/bevy_*-0.20.1`).

- **Rendering is pipelined.** `DefaultPlugins` adds `PipelinedRenderingPlugin` when Bevy is built with `multi_threaded` (`bevy_internal/src/default_plugins.rs:56`), with and without a window. There is a render thread; frame N is drawn while the main thread runs frame N+1. The two meet once a frame, in the extract: the main thread waits for the render thread to finish the frame before, then copies what the renderer needs out of the main world, and both are held while it does (`bevy_render/src/pipelined_rendering.rs`, `renderer_extract`).
- **Present mode** is `Fifo` (the default of `Window`, `bevy_window/src/window.rs:1243`; `main.rs` does not set it): vsync, and nothing else limits the frame rate. The frames come at the screen's rate, 8.3 ms here. `WinitSettings` is the default `game()`: continuous with the focus, sixty updates a second without it.
- **With no window** (`--view`, `--shots`, `--script`) winit is switched off and `ScheduleRunnerPlugin` runs the app thirty times a second; the camera draws into a picture, nothing is presented and there is no vsync. So a scripted run cannot say what the screen showed. It can say what each frame cost on each thread, which is what the recorder writes.
- **Threads.** Bevy's pools on this machine: 40 compute, 4 async compute, 4 io (`bevy_app/src/task_pool_plugin.rs`: a quarter of the cores each for io and async compute, at most four, the rest compute). The main schedules and the render schedule both run their systems on the compute pool. The simulation has a thread of its own (`live.rs`, `run_thread`). A year's scenery is one task on the async compute pool, and single-threaded (`scatter::build_year`). Rayon's own pool (48 threads) is used at start-up only (the ground, the slots, the far rivers) and by `--class-map` and the plan.
- **The simulation's thread** works one year for each `Cmd::Step`, answers with the year (`Msg::Year`), and saves after it has sent the answer. A save is 1.2 kB written with a sync to the disk: about 6 ms here (measured by writing the same file the same way thirty times). Only one step is ever out (`Live::stepping`), so steps never queue, and the clock stops short of the year it has not got (`advance_time`'s cap): back-pressure and an honest date are there already.
- **A year arrives** in `game::pump`, on the main thread. It does more than write the year into the run: if a scenery build still holds the run or the slots, each is copied whole first (`Arc::make_mut`: the run is about 17 MB by 1900); the news is sorted again; and `scatter::place_new_firms` works out the churchyards of every year so far and lays the works out again in every cell where one is new or has outgrown its ground (26 to 53 cells a year in the 1890s). Then, in the same frame: the families' panel data, `owner_marks` (flags, pennants and 7 to 13 MB of fences, built on the main thread and replaced whole), the side panel rebuilt (`ui.dirty` is bumped whether or not the panel shows anything new), the timeline's ticks despawned and spawned again, toasts, perhaps a decision card.
- **When the clock's year changes** (a frame of its own, later): `rebuild_year` starts the build; `carts::place` and `boats::place` despawn every cart and boat and spawn them again at their fixed places; the side panel is rebuilt again (its key holds the year); the ground material is told the year; the map view's texture is rewritten if one is on.
- **When the build is ready** (`rebuild_year`, the main thread): the ground's parcel texture is replaced; every train is despawned and spawned again at a fixed place on its line; and eighteen meshes are replaced by assignment (twelve of trees, houses, works, smoke, roads, rails, lanes). The old ones are freed there and then. `meshkit::into_mesh` moves the buffers, so making the meshes costs nothing; what costs is in the engine, after:
  1. `PostUpdate`, main thread: `calculate_bounds` (`bevy_camera/src/visibility/mod.rs:557`) reads every position of every changed mesh to find its box: 6 ms.
  2. The extract, main thread, both worlds held: `extract_render_asset` (`bevy_render/src/render_asset.rs:364`) clones every changed mesh, because the meshes are made with `RenderAssetUsages::default()` (kept by both worlds): 44 to 65 ms for 320 to 450 MB.
  3. The render thread, `RenderSystems::PrepareAssets`: `allocate_and_free_meshes` (`bevy_render/src/mesh/allocator.rs`) frees each changed mesh's place on the card, finds it a new one, and packs its corners into the buffer (`write_packed_vertex_buffer_data`): 183 to 276 ms. The source says of itself "TODO: Consider explicitly reusing allocations for changed meshes of the same size".
  4. The next frame on the main thread has nothing to do and waits for the render thread (280 ms in a window). If it starts at the same moment as the render thread's work, its own `PreUpdate` is held for as long (190 to 320 ms seen; 565 and 861 ms in the critic's run on a loaded machine). After the critique this is narrowed but still not established. It was seen only with no window (six frames in one of the critic's runs, none in another, none in a window, where the same time shows as the wait before the extract), which is where the main thread begins its next frame in the very instant the render thread begins its own. Both worlds' systems are tasks on the one compute pool, and the source offers two ways for one to be stuck behind the other there. A system that runs a parallel loop ticks the pool while it waits for its own batches and can be handed any task in it (`bevy_tasks/src/task_pool.rs`: "Pulling tasks from global executor can run tasks unrelated to the scope"), so a main-world system of that kind can find itself packing the render world's meshes. And a worker that takes a task from the pool's queue takes half of what is left in the queue into a queue of its own (`async-executor-1.14.1/src/lib.rs`, `Runner::runnable`), where a main-world system can wait behind the packing, though the executor wakes another worker to prevent exactly that. Which it is was not traced. Either way the lesson is the same: a long system in either world can hold a system of the other, and the cure is the cure for the stall itself, no system that runs for long.
- **A camera move across a level** sets `tl.built = None`; the whole year is built again off the main thread; when it is ready only the houses and the smoke are assigned, and the rest of what was built (300 to 400 MB) is dropped on the main thread.
- **A change of season** does not rebuild anything, as the README says. But `seasons` writes to three materials eight times a second while the clock runs, and the engine answers a changed material by marking every mesh that uses it as changed (`bevy_pbr/src/material.rs:735`, `mark_meshes_as_changed_if_their_materials_changed`), which sends `calculate_bounds` over every tree again: 6 ms in `PostUpdate`, one frame later.
- **A panel** is despawned and spawned whole when its tab, the year or `ui.dirty` changes (`ui::panel_rebuild`): 65 to 190 entities, and 4 to 9 ms in `PostUpdate` for the layout and the text. The Markets charts are painted on the main thread when the panel is rebuilt (0.1 ms in `panel_rebuild` itself for the goods' half).
- **A replay** has no pump. Dragging the timeline or playing asks for a build for each year the clock lands on; each finished build is a whole swap.
- **On the calendar branch** (`night/calendar`) a month arrives as a small message (its series, wages, prices, cash and news); the island is still redrawn once a year, by the same path. So the stalls are the same and come as often as years do: every twelve seconds at "Steady", twice a second at "Fastest". Its speeds are `clock::SPEEDS`: a month in two seconds, a month a second (where a game starts), a season a second, a year a second, two years a second. Today's 1x is half a year a second and today's 8x four; the calendar's fastest is today's 4x and its usual pace a sixth of today's 1x. So once it lands the freeze of a third of a second comes once in twelve seconds at the usual pace (3% of the time, and all the plainer for the months going by smoothly between), and the rows for 8x in this report are of a speed that will no longer exist. Its pump already steps round one of them: a month's series is held back while a scenery build holds the run, so as not to copy the run (`game.held`), which means the month's figures reach the charts up to half a second late. Its `FrameWork` (in `script.rs`) times `First` to `Last` on the main thread: for a year's swap it would say 26 ms, and miss the 65 ms extract and the 280 ms that follow.

## 2. The recorder

`ISOSTATE_FRAMES=PATH` turns it on. Unset, what is left in the app is the strip's card (three hidden nodes of the interface and a texture of 46 kB), one system in `Last` that looks for F3, and one load of a flag in each hooked system. Set, it does not change what it measures: in a window without vsync a paused game made 1,605 frames in eight seconds with the recorder off and 1,637 with it on (4.99 and 4.89 ms a frame; with the recorder off a plan now counts its frames, which is how this was measured). It writes `PATH.csv` and `PATH.txt` when the app leaves (and prints the summary).

- **What the screen got** (added after the critique): `present_ms`, the time on the render thread from the picture before being handed to the screen to this one (the stamp after the `Render` set, in which the engine presents: `bevy_render/src/renderer/mod.rs`). With a window this is what stalls, freezes and limits are now read from. The main thread's `frame_ms` shows the same stall a frame late and split in two (92 and 280 ms for one gap of 320 to 400 between pictures); added up as a freeze the two agree.
- **Main thread**, by stamps in small schedules put between the engine's own (`MainScheduleOrder`): `first_ms`, `pre_ms`, `update_ms`, `post_ms`, `last_ms`, their sum `main_ms`, and `frame_ms` from one frame's start to the next.
- **The extract**: the render app's extract function is wrapped (`SubApp::take_extract`), so `extract_ms` is the copy itself and `wait_ms` the wait for the render thread before it.
- **The heavy systems**, by a guard on their first line: `pump`, `scenery` (`rebuild_year`), `flags` (`owner_marks`), `panel`, `orders`, `mapview`, `seasons`, `traffic` (trains, carts, boats), `hud` (labels, toasts, ticks).
- **The render thread**, by stamps between the sets of its schedule, matched to the main frame they drew: `r_cmds`, `r_assets` (changed meshes and textures to the card), `r_meshes`, `r_views`, `r_acquire` (the wait for the screen), `r_queue`, `r_draw`, `r_cleanup`.
- **What happened**: megabytes of mesh changed, meshes and textures changed, entities made and gone, the clock's year and the newest, whether a year or a build was in hand, how long the build and the year took off the main thread, and marks (`Y W C B K S L P R O D`, listed at the top of `frames.rs`).
- **The summary**, for each named segment: the middle, 95th, 99th and longest frame and how many were over 16.7, 33, 50 and 100 ms, for the frame, the work, the main schedules, the extract and the render thread; the same for the frames each mark fell in; the freezes (how many, how long, what share of the time); and the three longest stalls with everything in them over half a millisecond.
- `ISOSTATE_FRAMES_PLAN="STEPS"` drives the run, in a window or without one: `seg`, `wait`, `settle`, `key`, `play`, `pause`, `speed`, `look`, `pan` (a smooth move and zoom), `year`, `scrub`, `tab`, `markets`, `overlay`, `quit`. A decision that opens is given its default, as `run:` does in a script. `ISOSTATE_FRAMES_PRESENT=novsync` takes the screen's wait away.
- `ISOSTATE_FRAMES_FAIL="freeze<150,frozen<10"` makes the run exit 3 if a segment breaks a limit, or 4 if it does on a machine its "calm" segments show to be busy (section 7).
- `ISOSTATE_FRAMES_CHUNKS=1750-1760` builds those years with no window and says how much of each differs from the year before (section 4).
- `ISOSTATE_FRAMES_TRY=nobounds,renderonly,slices` tries the cures of section 5. `ISOSTATE_FRAMES_SQUARE=100000` with `slices` makes every mesh one piece: the year's meshes handed over one a frame, uncut. `ISOSTATE_FRAMES_TRY=instances` (with `ISOSTATE_FRAMES_INSTANCES=N`) puts N trees on the island as things of their own that share eight meshes, to price that way of drawing them (section 11).
- With `ISOSTATE_TIMING` set the prototype says how long each cut took.

Changed after the critique, besides `present_ms`: a whole swap (W) is told by the roads' mesh, not the works', since `night/works-detail` sends the works again for the camera; a `settle` waits until the game has been quiet for four frames, since the frame scenery goes on screen in is followed by the renderer's work on it and the first run in a window charged a 242 ms wait to the "calm" segment after it and called the machine busy.

Three timers of frames are now in flight and should become one. `night/calendar` has `FrameWork` in `script.rs` (the main schedules only). `night/winter-fixes` has `Shown::longest_frame` and "the longest of the three frames after" in `rebuild_year` (the engine's frame delta). The recorder measures both and the render thread besides: when the three are merged, `sample` and `print` should read the recorder, and `Shown::swaps` (which `winter-fixes` makes a resource) should give the recorder its W and C marks instead of the guess from which mesh changed.

Example (the run behind the first table):

```bash
ISOSTATE_SAVES=out/saves ISOSTATE_FRAMES=out/win ISOSTATE_FRAMES_PLAN="key:Space;settle;wait:3;seg:paused-still;wait:5;\
seg:run-1x;speed:1x;play;wait:10;seg:run-2x;speed:2x;wait:8;seg:run-4x;speed:4x;wait:6;pause;settle;\
seg:paused-zoom;pan:23,28,3,4;wait:1;pan:23,28,60,6;wait:1;pan:23,28,14,4;settle;quit" \
  cargo run -p isostate-viewer --release -- --new
```

Add `--script "wait:9999"` for the same with no window.

## 3. What was measured

### In a window (release, 120 Hz, vsync): what the player sees

Frame to frame, in milliseconds. `m2-win-none`; the rows marked * are from `win-rel-1750`, an earlier run of the same kind.

| 1750 to 1779 | frames | middle | 95th | 99th | longest | over 16.7 | over 33 | over 50 | over 100 | freezes | of the time |
|---|---|---|---|---|---|---|---|---|---|---|---|
| paused, camera still | 594 | 8.3 | 9.1 | 10.1 | 49.6 | 3 | 1 | 0 | 0 | 1 of 50 ms | 1% |
| paused, panning 17 cells and back * | 1200 | 8.3 | 9.5 | 10.3 | 26.7 | 4 | 0 | 0 | 0 | none | 0 |
| paused, zooming 14 to 3 to 60 to 14 * | 1881 | 8.3 | 10.0 | 20.5 | 49.6 | 25 | 2 | 0 | 0 | 2 of 45 ms | 1% |
| 1x, camera still | 1032 | 8.3 | 10.5 | 30.1 | 284.7 | 13 | 8 | 8 | 4 | 4 of 369 ms | 15% |
| 2x | 662 | 8.2 | 13.6 | 93.0 | 282.7 | 29 | 17 | 14 | 7 | 8 of 366 ms | 33% |
| 4x | 320 | 8.1 | 98.1 | 288.1 | 300.8 | 42 | 32 | 22 | 15 | 12 of 416 ms | 63% |
| 8x * | 109 | 11.6 | 293.5 | 388.8 | 424.3 | 41 | 33 | 25 | 16 | 11 of 425 ms | 85% |

A year's scenery going on screen, 22 times: the frame of the swap 92 ms, the frame after 280 ms, the two together 371 ms (95th 399, longest 403). A camera swap, 7 times: 22 ms and 21 ms, together 42 (longest 57). A year arriving (no swap in the same frame), 23 times: 23 ms.

### What each event costs, by thread (release, no window)

Medians, in milliseconds; `rel-1750`, `rel-1835`, `rel-1882` (and `dev-1750` for the dev build). "main" is the main schedules; "of it" names the hooked systems; "extract" is the copy for the renderer; "render" the render thread's whole frame and "to the card" its asset step.

| event | era | n | mesh MB | main | of it | extract | render | to the card | work in all |
|---|---|---|---|---|---|---|---|---|---|
| a whole year's scenery (W) | 1750 to 1800 | 61 | 428 | 25 | swap 14, bounds 6 | 64 | 270 | 265 | 359 |
| | 1835 to 1890 | 54 | 321 | 24 | swap 13, bounds 6 | 53 | 209 | 202 | 290 |
| | 1882 to 1897 | 10 | 321 | 20 | swap 11, bounds 5 | 44 | 188 | 183 | 251 |
| | 1750, dev build | 119 | 420 | 21 | swap 11, bounds 6 | 48 | 255 | 251 | 323 |
| houses for the camera (C) | 1750 | 10 | 47 | 20 | swap 14 | 7 | 26 | 22 | 52 |
| | 1835 | 9 | 58 | 23 | swap 13 | 10 | 46 | 28 | 82 |
| | 1882 | 7 | 73 (to 154) | 18 | swap 11 | 11 | 38 | 33 | 66 (to 131) |
| a year arrives (Y) | 1750 to 1800 | 47 | 9 | 26 | pump 11, flags 7 | 2 | 9 | 4 | 38 |
| | 1835 to 1890 | 46 | 13 | 71 | pump 38, flags 14 | 4 | 12 | 6 | 92 (95th 125, longest 299; 742 with a swap in the same frame) |
| | 1882 to 1897 | 11 | 13 | 79 | pump 50, flags 13 | 4 | 11 | 6 | 99 (longest 290) |
| | 1750, dev build | 97 | 11 | 29 | pump 12, flags 8 | 3 | 10 | 5 | 42 |
| a step of the seasons | any | 155 | 0 | 3.6 | (+6 in the next frame's `PostUpdate`) | 0.6 | 5.2 | 0.3 | 9.4 |
| a panel opened | 1800 to 1897 | 15 | 0 | 8 to 10 | layout 6 to 7 | 2 | 5.5 | 1 | 13 to 19 (longest 21) |
| a map view switched | 1800 to 1897 | 9 | 0 | 4 to 7 | | 1.4 | 4 to 7 | 0.7 | 9 to 15 |
| nothing (paused, still) | 1750 / 1882 | 360 | 0 | 3.4 / 4.0 | `PostUpdate` 2.1 / 2.7 | 0.6 | 4.6 / 4.5 | 0.1 | 8.3 / 8.9 |

- The cost of sending follows the bytes: 265 ms for 428 MB, 183 for 321, 22 for 47, 4 for 9: 0.6 ms a megabyte to the card, 0.15 ms a megabyte for the extract's copy.
- The pump in the late years: `ISOSTATE_TIMING=1` on `rel-1882` puts the churchyards at 38 ms a year (fifteen years; the first call, at start-up, 586 ms), the fences at 12 to 16 ms, and the works laid out again in 26 to 53 cells every year. The longest pumps were 103, 123, 128, 193 and 459 ms (1865 to 1892).
- Off the main thread, release: a year from the simulation 104 ms (1750s), 143 to 240 (1840s to 60s), 226 to 236 (1880s to 90s); a year's scenery 426 to 529 ms in every era (the dev build 518 ms: `ISOSTATE_BENCH`). At 4x and 8x (two and four years a second) the scenery cannot keep up and shows every other year or fewer; the date runs on ahead of the island with no sign of it.
- A year is 9.96 million corners in 1760 and 6.72 million in 1900 (`ISOSTATE_BENCH`): trees 8.80 million falling to 3.04 (a fifth of them the bare limbs of winter, sent every year and drawn for a third of it), houses 0.86 rising to 1.81, works 0.20 rising to 1.15.

### With no window, by speed (release): the share of the time in freezes

Work per frame; a frame over 34 ms counts as frozen for as long as its work took.

| | 1750 to 1776 (`rel-1750`) | 1835 to 1865 (`rel-1835`) | 1882 to 1897 (`rel-1882`) |
|---|---|---|---|
| paused, still: middle / longest | 8.3 / 9.5 | disturbed | 9.0 / 10.9 |
| paused, panning: longest (camera swaps) | 53 (2) | disturbed | not run |
| paused, zooming: longest (camera swaps) | 86 (7) | 99 (7) | 131 (7) |
| 1x: longest / frozen | 374 / 19% | 342 / 17% | 263 / 15% |
| 2x | 378 / 33% | 347 / 52% | 251 / 36% |
| 4x | 372 / 67% | 374 / 75% | 395 / 67% |
| 8x | 378 / 96% | 414 / 87% | 356 / 77% |
| 4x while panning and zooming | 381 / 74% | 742 / 70% | not run |
| a map view on and off: the longest frame of a switch | 12.8 | 14.6 | 13.3 |
| panels opened in turn: longest | 18.9 | 21.2 | 21.2 |
| the timeline dragged over 40 to 80 years in 8 s | 389 / 82% | 396 / 80% | not run |

The dev build, 1750 to 1837 (`dev-1750`, an early run of the plan in which the "paused" parts ran on, so only its events are used): W 21 + 48 + 255 ms, Y 29 ms, as in the table above.

A replay (`m3-replay`, `m3-replay-b`): played at 4x from 1840 on a quiet machine it swapped seventeen whole years in nine seconds at 242 ms of work each and was frozen 48% of the time. Dragging the timeline across the 150 years in 15 s was measured twice, both times on a loaded machine: 57 and 69 whole swaps, 320 to 440 ms each, the picture frozen for most of the drag. There is no pump in a replay; the swap is the game's swap.

### Towards 2050

`isostate-core run --seed 7 --rules game --families 6 --end-year 2050` runs on without trouble and the viewer opens the run (`y2040.png`). The core takes 18 s to 1900 and 37 s to 2050: a year of simulation costs no more after 1900 than before (about 125 ms), with today's content.

The scenery does cost more. `ISOSTATE_FRAMES_CHUNKS`, with the houses built for the play camera: 306 MB a year in 1897, 361 MB about 1997, 437 MB about 2047 (trees 99, houses 185, works 97, roads, rails and yards 56). In the viewer, the same run played at 2x in the 1890s and in the 2040s one after the other (`m4-2050b`, the camera far off, the machine lightly loaded):

| | 1895 to 1900 | 2040 to 2048 |
|---|---|---|
| a year's scenery, MB | 275 to 289 | 354 to 364 |
| the swap: main / extract / to the card / work in all | 20 / 43 / 160 / 230 | 23 to 59 / 64 to 141 / 208 to 262 / 313 to 474 |
| a year's scenery built in | 0.57 to 0.64 s | 1.4 s alone, 2.0 to 2.4 s while the years run |
| a quiet frame's work | 10.4 | 11.9 |

So by 2050, as the island is drawn now: the freeze is as long as in 1750 (0.3 to 0.45 s), and the build takes two to four times as long as in 1900, longer than a year lasts at one year a second, so the island on screen falls behind the date at any speed above the slowest. Houses and works cost more to build for their bytes than trees do. In a live game the pump would be worse again: the churchyards and the laying out of works grow with the years played (section 3). With three or four times the people and works that the design for 2050 expects, where this run has about twice the houses and works of 1900, scale the houses' and works' share by that: 600 to 800 MB a year and a build of three to five seconds. Stage 1 keeps the picture moving through it; stages 2 and 4 are what make it keep up.

## 4. What a year sends, and how much of it is new

`ISOSTATE_FRAMES_CHUNKS` on a recorded run of island 7 (the game's rules): megabytes a year, how many of them lie in squares that differ at all from the year before, and how many corners are not where one was the year before.

| | MB a year | in changed squares of 8 cells | of 4 | of 2 | corners that are new |
|---|---|---|---|---|---|
| trees, 1751 to 1753 | 396 | 396 | 395 | 386 | 28% |
| trees, 1849 to 1851 | 223 | 217 | 206 | 186 | 30% |
| trees, 1897 to 1899 | 141 | 132 | 116 | 98 | 22% |
| trees, 1997 to 1999 | 94 | 75 | 53 | 32 | 3% |
| houses, 1751 to 1753 | 43 | 33 | 27 | 24 | 7% |
| houses, 1897 to 1899 | 82 | 71 | 62 | 56 | 5% |
| houses, 2047 to 2049 | 186 | 175 | 169 | 163 | 5% |
| works, 1897 to 1899 | 52 | 40 | 31 | 25 | 13% |
| roads, rails, yards, smoke, 1897 to 1899 | 37 | 28 | 17 | 11 | 4% |

- Change is small and everywhere. A scheme that sends only the squares a year changed would send nearly all of them, down to squares of two cells (1,470 of them).
- The trees are not stable. The README says "nothing reshuffles from one year to the next" and "woods thin tree by tree". By the count of corners, more than a quarter of the trees are new or altered every year for a century and a half, on an island whose woodland falls by a few per cent a year. Either parcels are changing class back and forth as the open land is shared out again each year (`world::year_land`), or the trees that stay are being drawn at a new size (a wood's depth sets its trees' height, `scatter.rs`), or both. After 1900, when land use has settled, it is 3%. (After the critique: the second is certainly at work, by the code. A wood tree's size is `(0.62 + 0.3 * deep) * (0.78 + 0.44 * stand) + 0.3 * v` and whether it stands at all is `(1 - u) < (0.7 + 0.26 * deep) * (1 - open)`, where `deep` comes from `YearLand::wood` for the tree's parcel: 128 at a wood's edge to 255 deep inside, worked out anew each year from the cell's share of wood. A cell whose share moves by a hundredth moves every one of its trees by a hair. That is not a shimmer the eye would catch, which may be why nobody has; it is a cost, since every corner of those trees is new to the last bit. How much of the 28% is this and how much is parcels changing class was not separated: stage 4's first cure would show it.)
- So the bytes must come down another way, or be spread: sections 5 and 6.

## 5. The stalls, ranked, and three cures tried

| | the stall | thread | why it holds a frame | of the worst frame |
|---|---|---|---|---|
| 1 | a year's scenery to the card | render | 320 to 450 MB of mesh freed, placed again and packed in one step; nothing is drawn until it is done | 183 to 276 ms of 251 to 359 (three quarters) |
| 2 | a year's scenery copied for the renderer | main (extract) | every changed mesh is cloned because both worlds keep it; both worlds are held | 44 to 65 ms (a sixth) |
| 3 | a year's arrival | main | churchyards of every year so far, works laid out again, the run copied if a build holds it, flags and fences built, panel rebuilt | 15 to 25 ms early; 70 to 80 late; to 490 |
| 4 | the old scenery freed and the new measured | main | 18 meshes dropped by assignment; bounds read from every corner | 19 to 25 ms |
| 5 | a camera move across a level | main and render | a whole year built for the houses; the rest freed on the main thread; all the houses sent at once | 42 to 57 ms in a window early; to 131 of work late |
| 6 | the seasons' step | main | a changed material marks its meshes changed, and their bounds are read again | 6 ms, eight times a second |
| 7 | a panel | main | despawned and spawned whole; layout and text | 13 to 21 ms |
| 8 | the frame after a swap, with no window | main | `PreUpdate` held while the render thread packs: one of its systems stuck behind the packing on the shared compute pool (section 1; which way is not established) | hidden behind 1; not seen in a window |
| 9 | the first second of a run | main and render | all the flags and fences at once in `owner_marks` (51 to 144 ms) in a first frame of 120 to 280 ms, then the whole year's scenery as stall 1 | a first frame of a quarter of a second and a freeze of a third of a second just after the window's first picture, once |

Not stalls: the simulation's year, the scenery's build, the save (6 ms on the simulation's thread, after the year has been sent), a map view (a texture of 10,080 texels), the pump's messages (never more than one year out), a group of trees shedding or budding (the 18 ms of work in those frames is stall 6 again, from the step of the seasons before).

In the late game the order changes once stall 1 is cured. With the prototype on, 1882 to 1900 at 8x still froze 28% of the time, the freezes 76 ms in the middle and 184 at the longest, and all of it was stall 3: `game::pump` took 51 to 126 ms in the frame a year arrived in (section 11).

`ISOSTATE_FRAMES_TRY` tries three cures without touching the scenery's code. Dev build, no window, 2x, 1750 to 1760, medians over ten swaps:

| | main in the swap's frame | of it `PostUpdate` | extract | to the card | work in all |
|---|---|---|---|---|---|
| as it is | 21 | 8 | 48 to 60 | 255 to 265 | 323 to 340 |
| `nobounds` (the scenery is never culled, so never measured) | 16 | 2 | 56 | 261 | 338 |
| `nobounds,renderonly` (the renderer alone keeps the year's meshes) | 3 | 2 | 6 | 265 | 277 |

`nobounds` also takes the 6 ms out of the frame after each step of the seasons (`PostUpdate` 4.3 ms a frame on average while the years run, 2.1 with it).

`renderonly` takes away the copy and the freeing (the extract takes the mesh, so there is nothing left to free), and, since a mesh the renderer keeps carries its box with it, the bounds too. The picture is the same (`try-both.png`). That is a sixth of the stall for two lines of code, and it leaves the other five sixths: the bytes still go to the card in one step.

`slices` is the rough prototype. Whatever `rebuild_year` puts on screen of the trees, houses, works and lanes is taken off again in the same frame, cut off the main thread into squares (each triangle to the square its first corner stands in), and handed over a few squares a frame, nearest the camera first, 6 MB a frame; each square is a mesh the renderer alone keeps, with the material, the season and the shadow of the mesh it was cut from. The old year's squares stay on screen until their turn comes.

Release, in a window (120 Hz, vsync), one run after the other (`m2-win-none`, `m2-win-slices16`):

| | as it is | squares of 16 cells, 6 MB a frame |
|---|---|---|
| paused, still: middle / 99th / longest | 8.3 / 10.1 / 49.6 | 8.3 / 9.7 / 10.3 |
| a year's scenery: the frame and the next, together | 371 (longest 403) | 21 (longest 33) |
| 1x: 99th / longest / frozen | 30 / 285 / 15% | 13 / 23 / none |
| 2x | 93 / 283 / 33% | 16 / 28 / none |
| 4x | 288 / 301 / 63% | 31 / 46 / 2% (5 freezes of 35 ms) |
| zooming across the levels: longest | 280 (a year's swap fell in it) | 24 |
| a camera swap, two frames | 42 | 33 |

Release, no window, 2x for twelve seconds (`m1-none`, `m1-slices8`, `m1-slices16`):

| | as it is | squares of 8 cells | squares of 16 cells |
|---|---|---|---|
| quiet frame: work, middle | 8.3 | 8.6 to 9.3 | 8.4 to 8.8 |
| while the years run: middle / 95th / 99th / longest | 9.5 / 35 / 377 / 388 | 13.7 / 30 / 39 / 56 | 12.4 / 26 / 30 / 34 |
| freezes | 15, of 360 ms, 33% of the time | 13, of 38 ms, 3% | 1, of 34 ms |
| the render thread, middle | 5.0 | 9.0 | 7.6 |

What the prototype costs: while a year streams in, each frame carries 5 to 6 MB to the card (2.8 to 3.6 ms on the render thread), so a frame's work is 12 to 14 ms where it was 9.5; a year takes about a second to come in whole, on top of the half second to build it; and squares of 8 cells make 1,644 more things for the engine to track, which shows as 0.3 to 1 ms more work in a quiet frame (squares of 16 cells: 535 things, 0.1 to 0.5 ms). What is left in its longest frames is the year's arrival (23 ms in a window, 28 ms of work) and the camera's swap (the 300 MB freed on the main thread).

What the prototype is not: it cuts the meshes after they are built (a pass over ten million corners, off the main thread), where the real thing would have the generators write into the squares; it shows two years at once for a second (a tree in a square not yet come); its squares are not culled as they could be; and a year that arrives while the last is still streaming throws the rest of the last away.

Measured after the critique (release, a window at 120 Hz with vsync, 1750 to 1820; `review/win-1750`, `review/win-1750-whole-meshes`, `review/win-1750-slices16`). "Picture to picture" is `present_ms`.

| | as it is | one mesh a frame, uncut | squares of 16 cells, 6 MB a frame |
|---|---|---|---|
| 1x: picture to picture, 99th / longest | 16 / 317 | 24 / 25 | 9 / 19 |
| 1x: frames over 16.7 ms, of about 1,000 | 9 | 26 | 4 |
| 1x: freezes | 4 of 314 ms, 14% of the time | none | none |
| 8x: picture to picture, 95th / 99th / longest | 328 / 351 / 396 | 25 / 31 / 47 | 16 / 28 / 33 |
| 8x: frames over 16.7 ms | 31 of 243 | 130 of 562 | 41 of 863 |
| 8x: freezes | 15 of 306 ms, 67% | 5 of 40 ms, 3% | none |
| 8x while panning and zooming: freezes | 26 of 293 ms, 75% | not run | 12 of 36 ms, 4% |
| megabytes that reached the card for each year shown, at 8x | 461 | 464 | 281 |

(The freezes of the first column are added up from `present_ms`, as the other two are. Read from the main thread's frames, as the tables of section 3 were, the same run gives 5 of 321 ms and 17% at 1x and 14 of 352 ms and 72% at 8x: a picture's gap is a little shorter than the two main-thread frames it is split over.)

- **One mesh a frame** is the prototype with one square as large as the island: the fifteen meshes of a year that are not empty (1 to 48 MB each) go to the renderer one a frame, each kept by the renderer alone. It needs no cutting, no new entities and nothing from the generators, and it takes the longest wait for a picture from 317 ms to 25 at 1x and from 396 to 47 at 8x. What is left is a dozen frames of 20 to 25 ms in a row once a year, and at 8x a quarter of all frames over 16.7 ms.
- **Squares** take the rest: the longest wait 19 ms at 1x and 33 at 8x, and a twentieth of the frames over 16.7 ms.
- **The last row is a fault of the prototype** that the real thing must not copy. At 8x a year's squares take longer to hand over than the next year takes to arrive, the next year's arrival throws the rest away, and the nearest go first: so a third of each year (the far squares, the same ones every time) never reached the card while the years ran. With no window, where a frame is 33 ms and the budget is by the frame, more than half was dropped (149 MB a year shown, of 330). The cure is in section 6: no new build while the queue has anything in it, and a budget by the second.
- Cutting costs 39 ms for a million corners (`ISOSTATE_TIMING`): 375 ms of work for a year of 9.6 million, about as much as building it (360 to 390 ms), spread over the four threads of the async pool.
- A piece can be far over the budget: with the camera close to a town in 1900 one square of houses was 84 MB and cost a frame of 46 ms. Pieces need a bound on their size, not only on their ground.

## 6. The design: the view never waits

What Bevy 0.20.1 gives, and what it makes hard:

- **Gives.** Pipelined rendering, so the main thread's ordinary work does not hold the picture. `RenderAssetUsages::RENDER_WORLD`, so a mesh is taken, not copied, and carries its bounds. Changing one mesh asset is atomic for that mesh: the old is on screen until the new is, in the same frame. `NoFrustumCulling` and `NoAutoAabb`. Mesh compression (`Mesh::compressed_mesh`, `bevy_mesh/src/mesh.rs:271`): normals in 4 bytes, colours in 4, positions in 8 relative to the mesh's box, which would take a corner from 40 bytes to 16 or 20. The async compute pool and `check_ready`.
- **Gives, besides** (added after the critique). `Aabb` with `NoAutoAabb` (`bevy_camera/src/visibility/mod.rs:551`): a piece spawned with its own box is culled when it is out of view and never measured by the engine, whatever its material does. `MeshAllocatorSettings` (`bevy_render/src/mesh/allocator.rs`): the size the card's shared buffers start at and grow by. Meshes under 256 MB of one layout share buffers that start at 1 MB and grow by half as much again, each growth a new buffer and a copy of the old on the card (`slab_allocator.rs`, `reallocate_slab`), so a year's 450 MB is reached through some fifteen growths in the first second of a run; raise `min_slab_size` if those show.
- **Makes hard.** There is no partial write to a mesh: a changed mesh is freed and sent whole. The engine's own budget, `RenderAssetBytesPerFrame`, is worse than useless for meshes. The bytes are packed and sent by `allocate_and_free_meshes`, which runs before `prepare_assets` and never asks the budget (`mesh/allocator.rs`: it takes every extracted mesh; the limiter is used in `render_asset.rs` and `erased_render_asset.rs` alone). `prepare_assets` then takes the old mesh off the screen before it asks whether there is budget for the new one (`render_asset.rs:449`, "we remove previous here"). So with a budget set the stall is as long as before and the scenery also vanishes for the frames it waits. The budget has to be kept on the main thread, by choosing how many meshes to change in a frame, which is what the prototype does.
- **Trees as things of their own** (corrected after the critique). The first report held that a tree cannot be an entity: "a hundred thousand would be 30 ms a frame", reckoned from what 1,644 squares cost. That reckoning was wrong: each square is a mesh of its own and a draw of its own, where trees that share a mesh are drawn together. Measured (`ISOSTATE_FRAMES_TRY=instances`, release, no window, a paused game, beside the scenery as it is): 100,000 trees of 59 corners in eight meshes and one material cost 4 to 6 ms more work a frame (the main thread's schedules go from 2.8 ms to 5.6, nearly all of it in `PostUpdate`; the render thread from 4.0 to 5.4 or 6.1); 250,000 cost 5 to 8 ms more (the main thread 6.3 to 8.3 ms, `PostUpdate` from 1.8 to 5.4 or 7.5); replacing 3,000 of them in one frame costs a frame of 23 to 33 ms. So it is a fifth of what was feared and still too dear as a standing cost: a broadleaf tree is two or three things (trunk, crown, bare limbs, each under a material of its own), the island had about a hundred thousand trees in 1750 by the count of its corners, and the work is shared out over this machine's forty threads, which a laptop has not got. The way to stop sending trees is in stage 4.

The stages, as rewritten after the critique (section 11 says what changed and why), in the order that causes the least merging.

Who else changes the hand-over, read from the branches on 12 November:
- `night/winter-fixes` changes `main.rs` most (396 lines), and `rebuild_year` with it. It already makes the year's meshes the renderer's alone (`KEEP_YEAR_MESHES`: a sixth off the stall, as `renderonly` measured), except in a scripted run, which keeps them to check what is on screen against the year built afresh (`check_scenery`). It already lets the trains run on when a year leaves the lines as they were (`same_lines`). It makes `Shown` a resource and puts a frame timer of its own in `rebuild_year`. And it gives the houses, works and yards materials of their own (`built.rs`).
- `night/works-detail` changes two lines of `rebuild_year`: the works go on screen again with the houses when the camera crosses a level.
- `night/calendar` changes `main.rs` by some 300 lines but not `rebuild_year`: the clock, the seasons' clock, the toasts.
- `night/town-ground` and `night/wood-edges` change what is built (`scatter.rs`, `world.rs`), not how it is handed over.

So: `winter-fixes`, `works-detail` and `calendar` first; then stage 0; then stage 1, which rewrites only the dozen lines of `rebuild_year` in which the meshes are assigned.

**Stage 0 (an hour or two). The recorder, and the scenery never measured.**
- Merge `frames.rs` and its hooks. It conflicts with nothing but the first line of a few systems. Let it replace the two other frame timers (section 2).
- Settle what a run with no window keeps. The test of section 7 runs under `--script`, and after `winter-fixes` a scripted run keeps the year's meshes in both worlds, so it would time a copy of 40 to 100 ms that the game in a window no longer makes. Keep the meshes only where `check_scenery` is asked for, or have `ISOSTATE_FRAMES` say not to.
- `NoFrustumCulling` on the eighteen entities of the year's scenery (they span the island and are never out of view). This takes away the 6 ms that follows every step of the seasons (stall 6) and the measuring at a swap; `nobounds` measured it. Once `winter-fixes` is in, the first of these is already gone in a window: a mesh whose data the renderer has taken answers for its bounds from a stored box (`bevy_camera/src/primitives.rs:21`, `final_aabb`), so the seasons' step costs nothing after the frame of the swap. It remains under `--script`, where the meshes are kept, and the 6 ms in the swap's own frame remains everywhere; the one line removes both.

**Stage 1 (half a day). One mesh a frame, under a budget.** Removes stalls 1, 2 and 4: the longest wait for a picture goes from 317 to 396 ms down to 25 to 47 (section 5).
- `rebuild_year` no longer assigns a finished year's meshes all at once. It puts them in a queue, each with the handle it is for, and each frame hands over what fits a budget, the meshes nearest in kind to what the eye is on first (houses and works, then the trees of the group in leaf, then the rest). The old year's mesh of each kind stays on screen until its turn comes.
- The budget is in bytes a second, not bytes a frame: an allowance of `rate` times the frame's length (held between 1/240 and 1/20 s) is added each frame and a mesh goes when the allowance covers it, or at once if nothing has gone for a tenth of a second. Begin at 600 MB a second: a mesh of 47 MB then goes every ninth frame at 120 Hz, each costing one frame of 24 ms, and a year is on screen in three quarters of a second. A budget by the frame makes a run with no window (30 frames a second) take four times as long to show a year as a window does.
- No build is begun while the queue has anything in it. This is the back-pressure the prototype lacks: a year is shown whole or not at all, and when the years come faster than they can be shown, years are skipped and none is left half sent.
- `tl.built` is set when the queue is empty, so `settle`, the scripted runs and the stills wait as they do now. Where the run is there to take stills (`--view`, `--shots`: `ShotPlanArgs` is in the world) the budget is lifted and the whole year goes in one frame as it does today: nobody is watching, the standard shots stay as fast as they are, and their pictures cannot differ. A scripted run and the recorder's plan keep the budget, since they are how it is tested.
- The ground's parcel texture, the trains and the railway's paths change with the first mesh, as now.
- For a second the picture holds two years at once (the houses of the new year among the trees of the old). At a month a second nobody will see it; a person should look at it at the fastest speed.
- What it leaves: a dozen frames of 20 to 25 ms once a year; the camera's swap, which is one mesh of 47 to 176 MB of houses and still costs one frame of 20 to 75 ms; and every stall that is not the scenery's.
- The brief a builder can start from is in section 11.

**Stage 2 (two or three days, the first part an afternoon). The year's arrival off the main thread.** Removes stall 3, which is the worst stall of the late game once stage 1 is in: 51 to 126 ms in the frame a year arrives in from 1882 to 1900, and a quarter of the time frozen at the fastest speed with the prototype on.
- First, by itself: the churchyards of past years are kept, not worked out again every year (27 to 40 ms a year by 1890, and growing with the years: by 2050 it would be twice that).
- `place_new_firms` and the flags and fences move into the scenery's build (they are scenery), on the async pool; the main thread only swaps in the result.
- The run stops being one block that is copied when a build holds it: each year's layers behind an `Arc` of their own, so a new year is an append. The calendar's `game.held` then goes, and a month's figures are never late.
- The pump takes messages under a budget of time a frame; what is not yet applied waits; the date on the bar is the date of what has been applied.
- The panel is rebuilt when what it shows has changed, not whenever a year arrives (stall 7 with it).
- Expected: a year's arrival costs the main thread 2 to 4 ms.
- Long work goes to the async pool or to a thread of its own, never into a system of either world, however far from the main thread that system runs: a long system on the compute pool can hold a system of the other world (section 1, the frame after a swap). The async pool has four threads and the engine compiles its shaders on it too, so keep the scenery's work to two of them.

**Stage 3 (a day or two). The scenery in pieces of bounded size, each sent only if it changed.** Takes the frames of 20 to 25 ms of stage 1 down to a frame's length, cures the camera's swap (stall 5), and lets the engine cull what is out of view. It should wait for `town-ground` and `works-detail` to land, since they change the houses and the works it cuts.
- At the end of `build_scenery`, off the main thread, each buffer is cut into pieces by the square of 16 cells its triangles stand in, and any piece over 4 MB is cut again into quarters (squares of 8 cells, then 4), so that no piece is far over a frame's allowance. The prototype's `cut` is this without the second cut: 39 ms for a million corners, 375 ms for a year, on the async pool. Smoke stays whole: it is small and see-through, and the order of its triangles shows.
- Each piece carries a fingerprint of its corners, worked out in the same pass, and its box. The build is given the fingerprints on screen and drops what has not changed before it ever reaches the main thread. For the trees this buys little while they are built as they are (section 4). For the camera's swap it buys nearly everything: only the pieces of houses and works whose level changed are sent, and the 300 MB of the rest is freed in the task and not on the main thread. `for_camera` then has nothing left to do for the meshes.
- Each piece is an entity with the material and marks of its kind, its own `Aabb`, and `NoAutoAabb`. Stage 0's `NoFrustumCulling` goes. A piece that is empty this year is despawned.
- The queue, the budget by the second, the back-pressure and `tl.built` are stage 1's, unchanged. Pieces go in the order: in view first, then by distance from where the camera looks.
- The scripted check of what is on screen (`check_scenery` in `winter-fixes`) compares piece by piece. `ISOSTATE_MESH_SUMS` is worked out before the cut and does not change.
- Expected, from the prototype: the longest wait for a picture 19 ms at 1x and 33 at 8x, with the year's arrival (stage 2) the whole of what is left.
- Later, not now: have the generators write into the squares' buffers, which saves the cut and lets the squares be built on several threads. It is the first report's stage 1. It touches every generator call in `scatter.rs` and the long meshes of roads and rails, on three branches' ground, and is worth it only once a year's build is too slow (2050).

**Stage 4 (a week, in parts). Send less.** Makes the stream short enough not to be noticed, and is what 2050 needs.
- **The trees, which are 88% of the bytes in 1750 and 46% in 1900.** A tree is one of a few shapes, turned, scaled and set down on a slot that never moves (`isostate_gen::trees::tree`, `scatter::build_year`); all that a year changes is which slots carry a tree and how large it is. Why a quarter of the corners are new each year can be read from `scatter.rs`: a wood tree's size is `(0.62 + 0.3 * deep) * ...`, and `deep` is the wood's depth at the tree's parcel this year (`YearLand::wood`), which moves a little every year in every cell whose share of wood moved. The trees do not shimmer; they all grow or shrink by a hair, and every corner is new to the last bit. Two cures, the first small:
  - A tree's size from its slot alone (or from the wood's depth held to a few steps), so that a year changes only which trees stand. Then count again with `ISOSTATE_FRAMES_CHUNKS`. Even at 3% of corners new a third to four fifths of the bytes lie in squares that changed (section 4, the rows for 1997), so this helps stage 3's fingerprints only in part.
  - The trees built once and never sent again: every slot's tree in buffers that do not change, by square, and a small picture with a byte for each slot (273,418 of them on island 7: 0.3 MB a year where there are now 140 to 400) that says whether the tree stands this year and how large, read where the corners are placed. The ground is drawn this way already (`terrain::Patchwork` and its `year_cells`), and `winter-fixes` gives the buildings a material of their own in the same manner, so the means are in the code. It costs a material for the trees with its own placing of corners in the colour, depth and shadow passes; the card's memory for every slot's tree (the 1750 island is two fifths of the slots at 400 MB; compress them, below); and a rule for trees on slots that had none when the game began. This is the one change that makes the cost of a year independent of how many trees there are, and it takes the trees out of the year's build as well, which is most of its 0.4 s.
- Levels of detail for trees by square, as the houses have: full within reach of the camera, a low tree of a dozen triangles beyond.
- The bare limbs (a fifth of the tree corners) built and sent only when winter is near, not every year.
- Compressed corners (`Mesh::compressed_mesh`: positions in 8 bytes relative to the piece's box, normals in 4, colours quantized to 4; 40 bytes a corner to 16). Check the colours first: eight bits may band in the dark tones. Positions are the piece's box cut into 65,535 steps: fine for trees in a square of 16 cells, too coarse for a house seen close unless its piece is small.

**Stage 5 (a day). What moves stays on the frame's clock.** Trains, carts and boats are despawned and spawned at fixed places at every year (the trains with the scenery, the carts and boats with the clock), so each jumps back once a year. `winter-fixes` already lets the trains run on while the lines are as they were; the carts and boats remain, and the trains in a year that adds a line. Keep each on its line or road by the line's own name, and carry its place along as a share of the length; spawn and despawn only where a line or a harbour is new or gone. The seasons already keep their own clock; the sea and the rivers are shaders on the engine's time. Smoke does not move yet.

**Stage 6 (with the calendar). Months as the unit, and a sign.** Once months land, the hand-over is asked for by what changed, not by the year: a month changes figures (small), a year's end changes the land (the stream of stage 1). The clock already stops at the edge of what the simulation has given. What is missing is the other half: when the island on screen is more than a year behind the date, say so quietly beside the date ("the island is catching up") and let "Fastest" be honest about it. With stage 1's back-pressure the island is never more than one build and one stream behind (about a second and a quarter), which at two years a second is two or three years: the sign will be lit at "Fastest" and nowhere else. The scenery could also be built for the year the clock will reach, not the one it has left, since the build takes half a year at "Fast".

Weighed and left for later or not wanted:
- **The save on a thread of its own.** 6 ms after the year is sent; it delays only the next step, by 3 to 6% of a year's work. Worth doing only if a slow disk shows up, and then as a writer that keeps the latest save only.
- **The task pools.** On this machine nothing is starved by the game itself: the simulation and the build are one thread each of 48. But the disturbed runs show how the view takes a loaded machine: the frame's work is spread over the compute pool (`PostUpdate` and the render queue went from 2 to 20 ms when other programs took the cores). On a four-core laptop Bevy would give the compute pool two threads, and the simulation and a build would be two more busy threads beside the main and render threads. Give the simulation's thread and the build a lower priority than the rest; do not make the pools smaller.
- **Fewer entities.** The year's scenery is eighteen entities now and would be a few hundred with squares, at 0.2 ms a frame. The interface is where entities churn (a panel is 65 to 190, the ticks, the toasts), and it costs 5 to 9 ms when it does, not every frame.
- **Uploads spread by the engine.** See "makes hard".
- **Trees as entities that share a mesh.** Measured and too dear as a standing cost: see "trees as things of their own" above, and stage 4 for the other way.
- **Culling.** Nothing of the year's scenery is ever culled today: seven to ten million corners are drawn in the colour pass and again in each of the sun's shadow passes every frame, wherever the camera looks. This machine's card does not notice (the draw step is 2 to 3 ms). A laptop's would, and stage 3's pieces with their own boxes are what cures it; it is a reason to do stage 3 even if stage 1 looks good enough.
- **A frame budget kept in one place.** Stages 1 to 3 each spend a share of the frame: bytes to the card, the pump's messages, the interface's rebuilds. One small resource should hold the frame's allowance and adapt it: when the time from picture to picture goes over one and a half times its usual length the rate is cut by a third, and it creeps back up while frames are on time. The recorder already has the number (`present_ms`); in the game it can be the engine's real frame delta. Without it the 600 MB a second of stage 1 is a guess that suits this machine and a 120 Hz screen.

## 7. A test that keeps it smooth, and a strip to see it by

The strip: **F3** (or `ISOSTATE_FRAMES_STRIP=1`) shows the last 240 frames as bars at the bottom left, green to 16.7 ms, amber to 33, red beyond, a dotted line at 16.7, a tick under each frame a year arrived in, and under it the middle frame, the longest and how many were over 33 (`strip.png`, taken with no window, where every frame is 33 ms). It is by the clock on the wall, so it shows what the screen showed. It costs a 46 kB texture a frame while it is up and nothing when it is not.

The test, to run before merging anything that touches the scenery's hand-over or the pump. It needs a card but no window, and the dev build will do (the stalls are the same in it):

```bash
ISOSTATE_SAVES=out/smooth-saves ISOSTATE_FRAMES=out/smooth-early ISOSTATE_FRAMES_FAIL="freeze<150,frozen<10" \
ISOSTATE_FRAMES_PLAN="key:Space;settle;wait:2;seg:calm;wait:4;seg:warm;speed:2x;play;wait:3;seg:years;wait:12;pause;settle;\
seg:zoom;pan:23,28,3,4;wait:1;pan:23,28,60,6;wait:1;pan:23,28,14,4;settle;seg:calm-after;wait:4;quit" \
  cargo run -p isostate-viewer -- --new --script "wait:9999"
```

and the same with `--advance 132` (`out/smooth-late`) for the late years. It exits 3 and names the limit if, in a segment, the longest freeze is 150 ms or more or a tenth of the time is spent frozen. The segments named "calm" ask nothing of the game: if one of them has a freeze, or frames of 25 ms, the machine was busy with something else, and a run that breaks a limit then exits 4 ("cannot be judged") instead of 3.

- Today the early run fails (`test-early-asis`: exit 3; on a quiet machine the years' freezes are 360 ms and a third of the time) and so does the late one.
- With the prototype's squares (stage 3 as the stages now stand) the early run passes with room (`test-early-slices`, the prototype in the dev build: exit 0, the years' longest freeze 42 ms and 2% of the time, none while zooming), so a machine three times slower still passes and today's code fails on one twice as fast. With one mesh a frame (stage 1) the freezes at 8x in a window were 40 to 47 ms and 3% of the time. Run as this test in the release build (`review/test-early-whole-meshes`), its "years" segment had four freezes, 44 ms in the middle and 63 at the longest, 1% of the time, well inside the limits; another helper's build then took the machine, the zooming and the last calm segment were ruined, and the run ended with exit 4 and not 3, which is the test doing what it is for. (With no window the share of the time frozen can come out over 100% on a loaded machine: it adds up work on both threads against the clock on the wall. It is a fault of the summary and was left.) The late run passes only with stage 2 (a year's arrival is 99 ms of work there now, to 290).
- `freeze` and `frozen` are the limits to use. `max` and `p99` move when another program takes the cores for a moment. A machine as busy as this one was does worse than that: with a dozen compilers running, paused frames cost 30 to 130 ms and a run "failed" with freezes of seconds, which is what the calm segments are for. They catch a machine that is busy at the start or the end, not one that becomes busy in the middle; a failure is worth one more run before it is believed.
- When the calendar lands, use its speed names in the plan (`speed:` takes years a second as a number too), and add a segment at its fastest.

Added after the critique:
- **Which limits, when.** `freeze<150` is loose: it lets through a pump of 126 ms and a camera's swap of 137, both of which are in the game today. Use it for the early run after stage 1 (one mesh a frame leaves freezes of 40 to 47 ms at the fastest speed, so the margin is three times). After stage 2 hold both runs to `freeze<100,frozen<5`, and after stage 3 add `over50<=2` on the early run. Keep the margin at about three times what the quiet machine shows: the copy and the packing are bound by the speed of memory, which differs less between machines than their cores do.
- **Under `--script` the test will time the wrong path** once `winter-fixes` is merged (stage 0, second point).
- **With no window a frame is 33 ms long**, so a budget by the frame hands a year over four times as slowly as a window does and the test would pass or fail on the budget's unit. Stage 1's budget is by the second for this reason, and is lifted only where the run is there to take stills, never under a script or the recorder's plan.
- **The test cannot see the screen.** It reads work, not pictures. A run in a window (the plan drives one as well as it drives none) is the only check of what vsync and the card make of it, and takes a minute on a desk nobody is using: run the windowed pair of section 5 before a stage is called done.
- **A `settle` now waits four quiet frames** (section 2), so a "calm" segment after a swap is no longer charged with the renderer's work on it; the first run in a window after the critique was called busy for that reason alone.

## 8. What can and cannot be measured with nobody at the window

- **Can, with no window:** each frame's cost on each thread, to a tenth of a millisecond; which system and which step of the renderer it went in; megabytes and entities. This is where the diagnosis comes from, and it agrees with the window: the swap's work of 359 ms against the window's two frames of 371.
- **Can, in a window driven by the plan:** what the screen got, with vsync, on this screen. The window opens on Stella's desktop for a minute; it need not have the focus (the plan sets winit to keep drawing). Whether it was covered by another window was not checked; the frame times (8.3 ms, the screen's) say it was being presented.
- **Cannot:** what the eye makes of it. A person should check, with F3 up: (1) that the freeze at each year is what the strip says (a red bar of two frames, about a third of a second) and that it is what "too fast" and "hitch" have meant; (2) with the prototype (`ISOSTATE_FRAMES_TRY=slices ISOSTATE_FRAMES_SQUARE=16`), whether the second in which two years show at once can be seen, and whether squares arriving nearest-first read as a ripple; (3) whether the woods shimmer from year to year at 1x (section 4 says they should); (4) the trains, carts and boats jumping back at each year; (5) a pan with the mouse, which the plan's smooth move is not: the pointer's ray is cast against the ground every frame (`cursor::hover`) and was not measured.
- **Not measured at all:** a slower machine (every figure is from 48 threads and a 4090; the copy and the packing are memory-bound and should scale with memory speed, not cores); a 60 Hz screen (the freezes are the same, the dropped single frames of the seasons' step disappear); the card's own time (the renderer's draw step is 2 to 3 ms throughout).

## 9. For the READMEs and the devlog

Not edited here, as asked. What should go in:

- `crates/isostate-viewer/README.md`, under "Checking keys and clicks without a window": a section "Checking that the view stays smooth" with the variables of section 2, the plan's steps, the test of section 7 and F3; and in the key table, "F3: the frame-time strip". Under "What is next for the look", replace the last line (per-chunk scenery) with stages 1 to 4 as they stand after the critique.
- The README's "Things on the ground" says nothing reshuffles from one year to the next. For the trees that is not so yet (section 4).
- `docs/DEVLOG.md`: one entry: the stall is the hand-over, not the simulation or the build; the figures of "In short"; the three columns of section 5's table (as it is, one mesh a frame, squares); the order of the stages and that stage 1 waits for `winter-fixes`, `works-detail` and `calendar`; that a game opens today at 4x, where the picture stands still for two thirds of the time.
- `docs/BACKLOG.md`: the stages of section 6 as items, stage 1 first; "a tree's size from its slot alone" as a note for whoever next works on the woods.
- `docs/probes/README.md` has its line.

## 10. Unsure

- **The disturbed runs.** The machine was shared all evening. The windowed pair of section 5, the three no-window runs beside it, and `rel-1750` and `rel-1882` have clean quiet frames and are what the claims rest on. `rel-1835`'s paused parts, `m3-replay`, the first half of `m3-replay-b`, `m4-2050` and `win-rel-1890-panels` are disturbed, and `m4-2050b` lightly (its quiet frames cost 10.4 ms where 8.9 is usual). The scrubbed replay and the run to 2050 want running again on a quiet machine; nothing in them contradicts the rest.
- **Why `PreUpdate` is sometimes held for the length of the render thread's packing** (stall 8). It happens when the main thread's next frame starts while the render thread is in its asset step. The critique narrowed it (only with no window; two ways the shared compute pool allows it, in section 1) and did not settle it: finding the system needs the engine's `trace` feature and a profiler, which this probe did not build. It hides behind the first stall and should go with it; if single frames of 20 to 30 ms with a long `PreUpdate` and nothing in it remain after stage 1 in a run with no window, this is what they are, and they are not what a player sees.
- **Why a quarter of the tree corners are new each year.** The count is exact (a corner is the same if its place, normal and colour are bit for bit the same). That trees change size with the wood's depth is in the code (section 4); how much of the count is that and how much is parcels changing class is not separated.
- **Stage 4's trees built once** are a design on paper. The count of slots is measured; the card's memory, the cost of a material that places corners in three passes, and what to do about a tree on a slot that had none are not.
- **The picture with two years in it** (stage 1: the new year's houses among the old year's trees for up to a second) was not looked at by anyone.
- **The budget's numbers** (600 MB a second, pieces of 4 MB) are from this machine and a 120 Hz screen. The frame budget of section 6 is there so that they need not be right.
- **The critic's runs were on the same shared machine.** Each has quiet "calm" segments at both ends except `review/rel-1750`, whose middle (8x while panning) was disturbed and is not used.
- **The prototype's cost in a window at 4x and 8x.** Five freezes of 35 ms at 4x: those are the year's arrival and frames that carried a large square. The budget is a soft one (a square larger than the budget goes whole); deep woods in squares of 16 cells reach 20 MB. Smaller squares where the trees are thick, or a budget by time, would trim it.
- **Mesh compression** is read from the engine's source, not tried. Positions relative to a square of 16 cells would be 0.5 m apart, too coarse for a house seen close; normals and colours are the safe half.
- **A year's build in 2050** (1.4 s alone, 2.0 to 2.4 s while the years run, on a lightly loaded machine). Why it is slower while the years run I did not find: the build and the render thread's packing may be fighting over memory. Either way the scenery lags the date at one year a second, and building only what changed matters for the build as much as for the bytes. And the run to 2050 has no content after 1900: what the 2050 design adds will change these figures.
- **The first frame of a game** has a stall of its own (`owner_marks` 56 to 150 ms, all the fences at once; shaders) that I did not look into, since it happens once.

## 11. After the critique

A second helper went over this report on 12 November as a graphics engineer would before the design is built: built the release viewer, ran the measurements the design rests on again, read every statement about the engine against Bevy 0.20.1's source, and attacked the list of stalls and the stages. Branch `night/smooth-probe-review` (from `night/smooth-probe` at 8bd5588). Its runs are in `.claude/night/reports/smooth/review/`. What it changed in this report is marked where it stands ("after the critique", "corrected", "added") and listed here.

### What was run again, and whether it agrees

Release build, island 7, six families, the game's rules; the same machine, shared again, each run with quiet "calm" segments at both ends unless said.

| | the first report | the critic's run |
|---|---|---|
| a quiet frame's work, no window, 1750 / 1882 | 8.3 / 8.9 ms | 7.2 / 8.8 |
| a whole year's scenery, quiet machine, 1750: mesh, extract, to the card, work in all | 428 MB, 64, 265, 359 ms | 452 MB, 53, 263, 329 |
| the same while the years run at 8x, 1750 to 1774 | longest 378 | 430 to 457 MB: extract 84 to 108, to the card 284 to 301, work 400 to 459 |
| a whole year's scenery, 1882 to 1900 | 321 MB, 44, 183, 251 | 319 to 340 MB, 40 to 55, 176 to 186; 308 in the middle |
| a year's arrival, 1882 to 1900: the pump | 50 in the middle, to 459 | 51 to 128; the churchyards 26 to 28 of it |
| houses for the camera, 1900 | 73 MB (to 154), work 66 (to 131) | 114 to 176 MB, work 95 to 137 |
| a panel opened, 1900 | 13 to 19 (longest 21) | 14.5 (longest 16) |
| in a window, 1x: the freeze at each year | 369 ms, 15% of the time | 314 to 321 ms, 14 to 17% |
| in a window, 8x | 425 ms, 85% | 306 to 352 ms, 67 to 72% |
| the prototype (squares of 16 cells) in a window: longest frame at 1x | 23 | 19 |
| at 8x | not run | 33, no freeze; 12 freezes of 36 ms while panning and zooming |

They agree. Two things the first report's medians did not separate: the copy in the extract costs up to twice as much, and the packing a tenth more, while the simulation and a build are running beside them (they share the memory's speed); and the freeze in a window is a little shorter here (the first report's 1x was in 1750 to 1773, the critic's in 1767 to 1771; they are the same two frames).

A game opens today at 4x (`Timeline::speed` begins at 2.0, two years a second), which by this report's own table is the speed at which the picture stands still for two thirds of the time. The first report did not say so. It may be much of what "too fast" has meant.

### The recorder

- **Is it honest about what the screen gets?** It measured the main thread's frame, which shows a stall a frame late and split in two; the sum was right. It now also times picture to picture on the render thread (`present_ms`) and reads a window's stalls from that. The two agree to within a tenth (section 5's table and its note).
- **Does turning it on change what it measures?** No: 4.99 ms a frame off, 4.89 on, without vsync.
- **Does it cost nothing when off?** Nearly: section 2 now says what is left.
- **Faults found and mended:** a "calm" segment after a `settle` was charged with the renderer's work on the swap before it and called the machine busy; the mark for a whole swap would have been wrong once `works-detail` lands; with the recorder off there was no way to count frames at all.
- **Not mended:** it guesses W and C from which mesh changed, where `winter-fixes`' `Shown::swaps` would say; the prototype and the other tries live in the same file as the recorder and should be taken out when stage 1 is built; the strip is by the main thread's clock, which is right in sum and a frame late.

### The engine, statement by statement

Right, with the place in the source: rendering is pipelined with and without a window (`bevy_internal/src/default_plugins.rs:57`; the recorder prints it); the present mode is `Fifo` (`bevy_window/src/window.rs:1242`) with a frame latency of two, and nothing else limits frames; the pools are 40, 4 and 4 here (`bevy_app/src/task_pool_plugin.rs`); a mesh kept by both worlds is cloned in the extract (`bevy_render/src/render_asset.rs:364`); a changed mesh is freed and placed again whole (`mesh/allocator.rs`, `free_meshes` and its TODO); a changed material marks its meshes changed (`bevy_pbr/src/material.rs:735`) and a changed mesh has its bounds read again (`bevy_camera/src/visibility/mod.rs:557`); compression takes a corner from 40 bytes to 16 (`bevy_mesh/src/mesh.rs:271` to 330).

Wrong or incomplete, and corrected in place:
- **The engine's byte budget** (section 6). It was said to take a waiting mesh off the screen. It does, and it also limits nothing: the mesh allocator sends every byte in the frame the mesh is extracted, whatever the budget says.
- **A mesh the renderer alone keeps "carries its bounds with it".** The bounds are still read, once, when the renderer takes the mesh (`Mesh::take_gpu_data` calls `compute_aabb`): that is the 6 ms the extract still cost with `renderonly`. Small pieces make it nothing.
- **"A tree cannot be an entity: a hundred thousand would be 30 ms a frame."** Measured: 4 to 6 ms (section 6).
- **The `PreUpdate` held after a swap** (section 1): seen only with no window; narrowed, not settled.
- **"All of them touch `rebuild_year`."** `winter-fixes` rewrites it, `works-detail` changes two lines of it, `calendar` does not touch it (section 6).
- **"Nothing in the viewer reads a mesh back."** True today; `winter-fixes` adds a scripted check that does (`check_scenery`), and keeps the meshes in both worlds under `--script` for it.

Added: how the card's shared buffers grow (section 6); where the engine presents, and so where to time a picture (section 2); that the engine's shaders are compiled on the async pool (`bevy_render/src/render_resource/pipeline_cache.rs:1068`), which the scenery's work must not fill (stage 2).

Asked and answered: what replacing a mesh costs against changing it in place. The same bytes either way, since nothing less than a whole mesh can be sent. Assigned under the handle it had, as `rebuild_year` does, the old mesh's place on the card is freed before the new one's is found (`allocate_and_free_meshes` frees first), so the card's buffers do not grow from year to year. Put under a new handle, the old mesh lives until its handle's `Unused` event, so for a frame the card holds both and the buffers grow to fit. Changing a mesh's corners in place through `attribute_mut` is no cheaper: it is the same `Modified` event and the same whole mesh. So the game is right to assign in place, and the pieces of stage 3 should be too.

### The list of stalls

Nothing on it is innocent. What it missed or ranked wrongly:
- **The late game's worst stall after stage 1 is the pump**, not anything of the scenery's: with the prototype on, 1882 to 1900 at 8x still froze 28% of the time. So the pump is the second stage, ahead of cutting the scenery finer.
- **The first second of a run** (stall 9): a first frame of 120 to 280 ms and then a freeze of a third of a second. It is once, and it is the first thing a player sees. Stage 1 takes the second half of it away with the rest.
- **A piece larger than the budget**: one square of 84 MB of houses, a frame of 46 ms. The prototype's "always one square" rule lets it through.
- **The prototype starves the far squares at speed** (section 5's last row): a third of each year never reached the card at 8x in a window, more than half with no window.
- **Culling**: there is none (section 6).
- **Looked for and not found to be stalls:** trees shedding or budding (the cost in those frames is the seasons' step before); a decision's card opening (nothing of its own above the year it arrives with); the save.
- **Still not measured:** a save on a slow disk (it is after the year is sent, so it can only delay the next year, never a frame); the pointer's ray every frame; a town's first appearance by itself; a machine with few cores; a 60 Hz screen.

### The design

- **Would stage 1 as first written do what it claims?** Yes, by the prototype, and the critic's run at 8x in a window bears it out. But as written it was not a day's work: it had the generators write into squares, which touches every call in `scatter::build_year`, the long meshes of roads and rails, and the same lines three branches are changing. The prototype does not do that either; it cuts after the build.
- **What simpler change buys most of it?** One mesh a frame (section 5). It is now stage 1. It leaves the rest of the stall to a later stage that can wait for the branches in flight.
- **What would the stages break?** Not the standard shots, if the budget is lifted where the run is there to take stills. Not saved games (nothing of the hand-over is saved). The scripted runs wait on `tl.built` as before and so see no half-sent year. The scripted check of `winter-fixes` must follow the pieces at stage 3. The picture with two years in it is new and unseen.
- **What was missing that a game of this kind does:** a frame budget kept in one place and adapted to the machine; a queue with back-pressure, so that what cannot be shown is skipped whole; pieces bounded by size and not only by ground; culling; and the plain observation that most of what is sent is trees that have not moved. All are in section 6 now.

### The stages as they now stand

| | what | how long | what it removes |
|---|---|---|---|
| 0 | the recorder merged; the scenery never measured for bounds; one frame timer where there are three | an hour or two | the 6 ms after each step of the seasons, where `winter-fixes` has not already |
| 1 | one mesh a frame under a budget by the second, with back-pressure | half a day | the freeze of a third of a second: 317 to 396 ms becomes 25 to 47 |
| 2 | the year's arrival off the main thread, the churchyards first | two or three days | 51 to 126 ms a year in the late game, growing with the years |
| 3 | pieces of bounded size, each sent only if it changed, each with its own box | a day or two | the dozen frames of 20 to 25 ms a year; the camera's swap; drawing what is out of view |
| 4 | send less: a tree's size from its slot, then the trees built once; levels for trees; compressed corners | a week, in parts | most of the bytes, and most of a year's build |
| 5 | carts, boats and trains keep their place | a day | the jump back at each year |
| 6 | a sign when the island is behind the date | with the calendar | |

### The brief for stage 1

For whoever builds it, to be read with section 6. It goes in after `night/winter-fixes`, `night/works-detail` and `night/calendar` are merged, and after stage 0.

What to build:
1. A resource `HandOver` in a new file, `handover.rs`, so that `main.rs` gains a few lines and no more: a queue of `(Handle<Mesh>, Mesh)` in the order they are to go, an allowance in bytes, a rate in bytes a second (begin at 600 million), and the year the queue is of.
2. In `rebuild_year`, where a finished build's buffers are assigned to the handles (`*m = non_empty(b)`, three loops): push each `(handle, non_empty(b))` onto the queue instead. Order: houses, works, smoke, then for each group of trees what is green all year and whichever of its leaves and bare limbs `in_leaf` says is showing, then the hidden ones, then lanes, roads, rails. The ground's parcel texture, the trains and `rails.0` change at once, as now. For a camera's build only the houses, the works and the smoke are pushed, as now.
3. Set `tl.built = Some(k)` when the queue has emptied, not when the build lands. Do not begin a build while the queue is not empty (the test `builds.task.is_none()` gains `&& hand_over.queue.is_empty()`).
4. A system `handover::feed`, after `rebuild_year` in the chain: add `rate * dt` to the allowance, with `dt` the real frame delta held between 1/240 and 1/20 s; while the front mesh's bytes are within the allowance, or nothing has gone for a tenth of a second, pop it, assign it (`*meshes.get_mut(&handle)? = mesh`) and take its bytes off; never let the allowance grow past the largest mesh in the queue. Where `ShotPlanArgs` is in the world, hand the whole queue over at once.
5. Adapt the rate: if the real frame delta is over one and a half times its running middle value, multiply the rate by 0.7 (not below 150 million); otherwise by 1.02 (not above 1,200 million).
6. The meshes are the renderer's alone (`winter-fixes`' `non_empty` already makes them so); see stage 0 for what a run with no window keeps.
7. In `frames.rs`: take W and C from `Shown::swaps`; add a mark for "a mesh was handed over" and a column for the queue's length; leave `try_slices` and `cut` behind their variable for stage 3 to take.

What done looks like:
- In a window, release, 120 Hz, from 1750, with the plan of section 2's example at each of the calendar's speeds: no freeze at the usual pace or at a year a second; at the fastest, freezes of no more than 50 ms and under 5% of the time; `present_ms` 99th under 34 ms.
- The test of section 7, early run, exits 0 with `freeze<150,frozen<10`. The late run still fails, on the pump, until stage 2.
- `--shots` gives the same pictures as before (`--compare-shots`), in the same time.
- At the fastest speed, the megabytes handed over for each year shown are a whole year's (about 450 in 1750): nothing is dropped.
- `cargo test -p isostate-viewer` passes; a new test feeds a queue of known sizes through the budget and checks that every mesh goes, in order, and that none goes while the allowance does not cover it.
- A person looks, with F3 up, at the fastest speed: the dozen amber bars a year, and whether the second in which two years show at once can be seen.

What to leave alone: the generators, `scatter.rs`, `build_scenery`'s result, the entities of `setup_world`, the camera's `Lod`. Stage 3 changes those, later.
