# The look: generators and visible change

Notes from a planning chat on 2026-11-09 about how the island should look and how its detail gets made. Nothing here is built yet except where it says so. PLAN.md has the wider plan, DEVLOG.md what happened.

## Decisions (November 2026)

- **Generators, not asset files.** Everything on the map stays code, as now, and grows into procedural generators in the spirit of Tiny Glade: many small generators, one per element (a row of houses, a roof, a farmstead, a bridge, a works), each with hand-written rules. No image-to-3D tools, no Blender files.
- **The simulation holds the pen.** In Tiny Glade the player draws a rough line and the generators supply the charm. Here the economy draws the lines every year (which streets exist, how dense each cell is, each works' method, size and founding year) and the generators turn them into buildings. The player does not draw: a hand-built family seat or model village was considered and dropped.
- **Honest decoration.** Draw only what the sim knows. Windmills where grain grows and sheep on pasture are fine; canals are not, until the sim has canals. Ideas that need new sim features go to PLAN.md instead of onto the map.
- **Zoom bands.** Island scale: the ground shader carries towns, as now. Play distance, where the information is: footprints, roofs, colour, silhouettes, and state you can read. The closest zoom is flavour, as in Anno: windows, doors, gardens, people, scaffolding.
- **Seeing it change is the point.** Zoom into a town and a building goes into scaffolding; a few seconds later the scaffolding comes down and it looks meaningfully different. Changes turn up here and there all the time, so a growing town is felt rather than read off a chart.
- **Built for the average Anno player**, a mid-range card (around an RTX 3060 or 4060), not the development machine. Generating scenery is CPU work, so watch the rebuild time (`ISOSTATE_TIMING=1`) as well as the frame time.

## Visible change

The sim is continuous: densities, shares and capacities move a little every year. Drawn continuously, a wall would grow a centimetre a week and nobody would notice. Swapped on the yearly tick, the whole island would jump at once. So every element shows a discrete state and changes it in visible steps, spread out in time and space.

- **Steps are visible.** The generator decides what an element's states are (a cottage, a second storey, a rear wing; a works tier), and a measure picks the state. Each step should look different at play distance.
- **Staggered thresholds.** Elements that read the same measure (all houses in a cell read its density) must not step together. Each gets a fixed offset on a log-spaced ladder: element i is at step `floor(log(m + m0) / log(1 + s) + phi_i)`, with `phi_i` a hash of its slot and `s` about 10%; `m0` stops tiny values stepping all the time. Neighbours then sit at different steps at any moment, which is variety for free. This is what the slots already do for whether a house or tree exists at all (a fixed threshold per slot), extended to every visible change. Works tiers are already a log ladder (a step at each doubling of capacity).
- **Spread across the year.** The sim gives one value per year, so every crossing would land on the year boundary. Instead, when year k arrives, each element whose step changed gets a start time inside year k, where its threshold falls between the old and new value, plus a little jitter. The picture then runs up to a year behind the numbers, which reads as building taking time. In a live game the next year is simulated when the clock reaches the end of the current one, so year k's changes are known when it starts.
- **Construction takes sim time; the finish takes real time.** At 1x a year lasts two seconds (8x is four years a second). A step goes: scaffolding up, about a year of work (a few seconds at 1x: dust, people, hammering), then the scaffolding comes down on the new state with a one-second settle. Because the scaffolding is a state in sim time, scrubbing and replays show it correctly; only the settle is real time, and it snaps while scrubbing or at 8x. The scaffolding also hides the swap, so generators never have to morph one shape into another. Trees grow by scaling, fields change in the shader, smoke starts and stops.
- **Hysteresis and decay.** The sim cycles (farm employment zigzags every two or three years; iron swings). Going up takes one step; going back takes falling two. Decline does not unbuild: a works that closes goes cold (no smoke), derelict after some years, then is cleared; a shrinking street empties before it loses houses.
- **State is computed, not accumulated.** An element's step in year k is a deterministic scan of years 0..k (cached), so the same year always looks the same however you got there.
- **Rate.** Tune the step size per element type for a rate of visible change (a starting guess: a few changes a second in a growing town at 1x), not a fixed 10%.
- **Time-based changes are for cosmetic life only** (washing on the line, a garden filling in, repainting). Structural steps come from the sim; a building that changes while its economy does not would be dishonest.

### Where towns are now

`scatter/towns.rs` (the other session, 2026-11-09) lays towns out along their roads: high streets, back lanes and cross streets, plots fronting the street that fill in order of how central they are, separate houses on the high streets and the market place, brick terraces with slate roofs on newer streets, windows and doors on every house. Two things in it will fight visible change and are the first fixes:

- Style follows the year on screen (`year < 1815` in `towns::houses`), so every back street flips from separate houses to terraces in 1815. Style should be fixed when a plot is first built.
- Rows are regrouped every year from whichever plots are standing (runs of up to six, and only above a density of 0.6), so a row can merge, split and change as its neighbours fill in. Row boundaries should be fixed per street, like the plots.

## What generators react to

The relationships the sim already knows:

- A house meets a street: the door faces it, a step and a path lead up, corner houses turn the corner.
- Houses side by side become a terrace: party walls with the chimneys on them, a roofline stepping with the slope.
- A road over a river: a bridge, stone arches early and iron later. A railway over a valley: a viaduct; on a slope: a cutting or an embankment.
- A works by a river: a mill race and waterwheel before steam, a tall chimney after.
- Field edges: hedges in the lowlands, dry-stone walls on high ground, gates where tracks meet them. Open strip fields could give way to hedged enclosures as improved husbandry spreads (ground shader).
- Age: each slot knows the year it first filled and takes that era's style. Old cottages sag, new terraces are crisp; old centres have crooked lanes, later streets are straight. Firms carry `born`.
- Material from the local economy: stone near quarries, brick near clay pits and brickworks, timber by woods, slate spreading along the railways. (Renewal could follow land values, but the replay does not carry rent yet.)
- State: each works type recognisable by silhouette at play distance; working, cold and derelict look different; family colours as now.
- Seasons on things: snow on roofs, bare trees, more smoke in winter.

## Light

Much of Tiny Glade's softness is its lighting, from its own renderer, which the developers have not written up. The cheap part of it: generators bake ambient occlusion into vertex colours (dark where walls meet the ground, under eaves, between houses, under trees). Settle the light and post (sun, sky light, AO, fog, tonemapping, tilt-shift) before tuning many colours under them, and move them into a look file next to `content.toml` so trying a direction is a data change.

## Sound

A soundscape from what is in view and what is changing in it. A bed mixed from what the viewport shows, weighted by area and camera height: birds and wind over fields and woods, a murmur in towns, clank and hiss near works, gulls at the harbours, the season and the weather. Events in view: hammering while scaffolding is up, a small pleasing sound when it comes down, train whistles, bells at the turn of the year. Most likely mixed procedurally from recorded samples rather than synthesised. Audio is not in the build (the viewer turns Bevy's default features off); Bevy's own audio or kira are the options. Undecided.

## Order of work

1. **Tooling.** A lab that renders a generator or a street as a contact sheet at the real camera distances in the real light, with each generator's parameters on live sliders for tuning by eye. Named colour and size tokens in place of the inline hex values (about sixty). Generators as pure functions (inputs to geometry, maths only), in their own crate or module so they compile and test quickly.
2. **Visible change on one street.** Fix style at build time and row boundaries per street, add the step ladder and the scheduling, scaffolding as a state, the settle. Judge it up close and at play distance. Tiny Glade itself began as one wall generator.
3. **Chunks.** `rebuild_year` (`crates/isostate-viewer/src/main.rs`) rebuilds the whole island's scenery on the main thread at every year change. Generate per cell in the background (rayon is already a dependency), cache, and rebuild a chunk only when its inputs change.
4. **The kit.** Footprints as polygons (L and T), roofs fitted to footprints including joins, wall strips with openings placed by rule, bevels and a little wobble, baked AO.
5. **Art study.** Three or four directions on the same shots (whole island, a town at play distance, a works close up; 1750 and 1900; summer and winter). To decide: toy or period (Townscaper and Tiny Glade, or Constable, Wright of Derby and *Coalbrookdale by Night*); free colour or regional coherence; how dark industry gets.
6. **Fan out by generator,** in waves, one element per agent, every change on a contact sheet beside approved examples. Rank by screen time: houses and trees are most of the pixels; the fifteen kinds of works carry the most information.

## Sources

- Interview with the Tiny Glade developers, 80.lv, May 2024: <https://80.lv/articles/exclusive-tiny-glade-developers-discuss-bevy-proceduralism-publishers-cozy-games>. Many small generators, each with a pipeline of "creation, assembly, morphing, or deforming"; every rule hand-written; watching things being built is the most vital element; Rust on a modified Bevy ECS with their own Vulkan renderer.
