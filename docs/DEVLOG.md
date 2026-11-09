# Devlog

Working notes, newest first. The plan lives in PLAN.md; this is what happened and what is in flight.

## 2026-11-09: the family game, first pass

Stella's design (agreed in chat): the player is a family running a conglomerate. Rival families do the same. The government is always run by one family, so losing power does not end the game, and holding power drains the company (family members serving in government are not running it), so a good player can stay in office but most will cycle in and out. Companies mostly run themselves; the player chooses industries, places, tricks and policies, maybe shady ones with risk. Historical politics (limited franchise, rotten boroughs, reform acts, Corn Laws). The player may meddle directly (public works).

Decisions for this pass:
- Build the company half first, no politics yet. Test whether steering a self-running company is fun before adding parliament.
- The family is named people with ages and skills, because the cost of office will be people.
- The design principle that gives the company half its teeth: the sim's firms are short-sighted (they build where today's prices pay), and the player is the one agent who thinks ahead (early entry down learning curves, vertical integration, land along future lines).

In flight:
- Core (helper agent, branch `families`): families with members, firm and land ownership, actions (site options, found works, buy land, assign members), AI families, reports, and a live API (`live.rs`) so the viewer can run the sim in a thread.
- Viewer, terrain shader (helper agent, branch `terrain-shader`): move the patchwork from the CPU-painted texture into a material so fields are crisp at any zoom, with seasons.
- Viewer, interface (me, branch `rust-3d`): a proper game interface in the same paper-and-ink style, then the live game loop with the player's actions on the map and family banners on buildings.

## 2026-11-08: Rust core and 3D viewer

- `crates/isostate-core`: full port, checked step by step against Python (deterministic steps to about 1e-14, stochastic steps draw for draw), 16 seeds agree within chance, 21 s per 150-year run.
- `crates/isostate-viewer`: Bevy 0.20 diorama replay viewer with fields, woods, towns, works with tiers and smoke, churches, stations, lighthouses, trains, seasons, toasts, labels and click-to-inspect.
