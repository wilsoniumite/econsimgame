# Devlog

Working notes, newest first. The plan lives in PLAN.md; this is what happened and what is in flight.

## 2026-11-09 (later): a playable first game

The viewer now plays a live game: `cargo run --release -p isostate-viewer` with no arguments makes island 7 with six families, the player's first, and opens over the player's home town with a card saying who you are and how to begin. Naming a `run.bin` still plays a replay.

How it fits together. The core gained families (branch `families`, merged): ownership of works and land, family cash in the money ledger, dividends and rents to owners, members who manage industry groups for a small efficiency bonus, AI families that invest every year, and a live API (`live_static` once, `live_year` after each simulated year). The viewer runs the simulation in its own thread and writes each finished year into the same arrays a replay reads, so the map, panels and timeline did not have to change. The thread only works when asked: the clock asks for the next year when it reaches the last one there is, and the player's orders go in between, so an order given while looking at 1786 lands before 1787 is worked out. With `n_families = 0` the core is still byte-for-byte the port of the Python.

The interface, all in the paper-and-ink style: three floating cards along the top (the family with its worth, cash, works and land; the date with Pause, 1x to 8x and Next year; tabs with little drawn icons), a scrollable side panel (Family, Holdings, Markets, News), a timeline with the major news ticked along it, a help card on H, and an inspector that splits a title from its notes. Click a square in the present and the inspector says who holds it, offers to buy it at 25 years' rent, and lists what could be built there with its cost, its expected return at today's prices, and how deep the market is (the share of the town's purchases it would make, and how far above its usual price the good sells, tagged "thin market" when either is high). The family panel has the family's worth over the years, last year's accounts, the members (click one to put them in charge of a trade) and the great families' table; holdings lists the family's works and flies the camera to them. The end of 1900 brings a card with the final standings.

On the map: family works fly a pennant in the family colour and family land is fenced in it; an ordered works gets surveyors' pegs until it goes up; the square under the pointer is outlined faintly and the clicked one boldly. Unrelated to the game but part of the same push on the look: houses now line the streets in rows before the blocks behind fill in, works stand in scaffolding the year they open or grow a size, and harbour towns have small sailing boats tacking offshore.

Is there a game? A helper wrote scripted players (`examples/bots.rs`) and ran them over four seeds. Passive play comes last every time. Greedy play (always build the best promised return) does badly: the promised returns are snapshots that ignore what the new works and its imitators do to the price, and in thin markets they are wildly high (rail forges promise 1,500% and every one dies). A prudent player who only builds where it would make at most a quarter of the town's purchases and the price is at most three times usual, and puts spare cash in land, finished first in three seeds out of four, but only climbs in the last fifty years. Works die young for everyone (median life 11 years for works founded after 1750, even with no families), because small works are dear per unit and thin markets collapse. Changes made: site options report market depth, AI families keep out of thin markets, and rivals' news is cut to their first works of each kind.

Open design points from that pass:
- Merchants pay the gate price for all output whether or not anyone buys it (from the Python), so cheap works in thin markets can be farmed for a few years. Capping what merchants pay beyond the town's demand would close it.
- `buy_land` takes any family's land at 25 years' purchase without consent. Probably wants a premium or a refusal.
- Land is priced on last year's rent, so buying ahead of a railway is cheap. That may be the intended reward for foresight.
- Family spending is fixed (60% of income plus 2% of cash a month) and is the biggest drag on the player; it could become a lever.
- Pace: the player sits low for a long time. Starting the player with a small estate rather than cash would open the game mid-table.

The ground is now painted per pixel by a shader (helper, branch `terrain-shader`, merged). It repeats the CPU's patchwork rule exactly (the hash runs in wrapping u32 maths, and "lower class if `sub_uniform*255 < t`" turns out to be "`sub_raw <= quant[t]`", one table lookup), so trees and houses still stand on matching ground: a check that casts rays against the mesh and compares classes under every pixel agrees on 99.99% of land pixels, the rest on class edges. Fields have their own crop calendars and rows, hedges are filtered to the pixel with a painted shadow, heath gets heather and gorse, the seasons run from young crops to stubble to frost, and GPU memory for the ground fell from about 31 MB to 50 KB. Known limits: class edges between neighbouring warped cells are hard steps smoothed only by SMAA, and the painted hedge shadows assume the sun stays in the south-west. Carts on the main roads went in after the docs above were written.

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
