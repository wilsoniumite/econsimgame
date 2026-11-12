# Backlog

What Stella notices while playing, in her words, and what becomes of each note. It is also where the order of work is decided. The diary of what was done is [DEVLOG.md](DEVLOG.md); this file is what is waiting, what is being done, and why in that order.

Last gone through: the morning of 12 November 2026, when the mines, the wood edges and the railways' profiles landed.

## How it is kept

- **A note goes in here first**, dated and word for word, in "The notes, as written" at the foot. It is not started because it was said.
- **Notes on one thing are gathered** and taken up together, so that a second thought can change the first before any work is done.
- **The backlog is gone through at each landing** (whenever a piece is merged into `main`) and when a round of notes arrives. Notes the landed work has answered are closed, with where. Notes it has made moot are pruned, with a line saying why. The rest are ranked again.
- **A note is taken up when its time has come:** nothing in flight is rewriting the same code or the same part of the picture; what it depends on has landed; and it ranks above whatever else is ready. Two exceptions: something that stops her playing, and a fix of a line or two in code nobody is working in.
- **The ranking**, highest first: what breaks play or makes the picture say something untrue; what many other things wait on; what she has raised more than once; what makes the island's economics truer; polish.
- **What helpers and reviewers find** along the way is kept on a page of its own, [backlog/found-along-the-way.md](backlog/found-along-the-way.md), ranked in the same list as her notes, and never dressed as her note. Below, a number alone is one of her notes and a number after F is one of the pieces found.

Until the evening of 11 November her notes were taken up as they arrived, usually within minutes. One piece had to be stopped a quarter of an hour in and started again when her next note changed its direction. This file is the answer to that (note 18).

## In flight

Started before this file existed, and left to finish. The usage limit cut every one of them off on the evening of the 11th; each was carried on the next morning from what it had committed. What each branch is expected to answer of the helpers' findings is listed at the foot of the page of findings, to be checked when it lands.

| Piece | Notes | Branch | State |
|---|---|---|---|
| Canals (core) | 9 | `night/canals-review` | reviewed; being merged on the integration branch |
| A second island generator: erosion, real rivers, lakes (core) | 4, 6 | `night/land-review` | reviewed; merged on the integration branch after the canals. It stays switched off until the viewer can draw it |
| Books of account for towns, works and the island (core) | 13 | `night/accounts` | built; in review |
| The production tree's first stage (core) | 8, in part | `night/tree` | being built; a second helper carries on from the first's saved work |
| Months that step, and slower speeds (viewer) | 10 | `night/calendar` | built; in review |
| Towns drawn from the ground they keep (viewer) | 14 | `night/town-ground` | being built; cut off twice, and a third helper carries on from the saved work |
| Industry brought up to the houses; one kit, one scale, a test that holds it (viewer) | 14, 16 | `night/works-detail` | built; in review |
| Winter, and loose ends reviewers found (viewer) | none | `night/winter-fixes` | built; in review |
| Where frames stall: a recorder, measurements and a design, no fix yet (viewer) | 17 | `night/smooth-probe` | measured and designed; with its critic |
| The documents a newcomer reads first: README, the plan's account of where we are (F23) | none | `night/docs-front` | written; being checked command by command |

Queued behind one of these and already briefed: works laid out by what they do and hold (note 16), which starts on `night/works-detail` once that has been reviewed.

## Ready next, in order

Nothing here is started until its trigger is met and the backlog has been gone through again. Her notes and what helpers found are in one order.

1. **A real game is not lost to a test start** (F1). A game started for a test can set the island's saved game aside, and four such starts can lose it; two copies of the game write the same autosave. *Trigger:* `night/calendar` is merged, since it changes the save. *Why first:* it is the one thing here that can cost her a game she is playing, and it is hours of work.
2. **Draw what the core pieces add, each as it lands** (notes 2, 4, 6, 9; F7). Mines as small workings at the foot of the hill; canals; the new land's relief, rivers and lakes. The mines are merged (12 November) and are drawn only as a stand-in, one of the old pits at each working: the grey blocks are gone and the mines are nearly invisible. *Trigger:* for the mines, `night/works-detail` and `night/town-ground` are merged, since they are rewriting the generators and the placement code the drawing would work in; for the canals and the new land, each is merged. *Why here:* these are her first notes.
3. **Roads, bridges and carts laid again on the new rivers** (notes 19, 20; F3). The cart that sails over a river and clips through a bridge was already in a helper's notes: a cart's path is a straight line between cell centres at the height of the ground. The roads' links and every bridge are the viewer's own invention, which is why a road down a valley crosses its river again and again. Neither is mended on its own, because the rivers are about to be replaced. *Trigger:* the viewer draws the new land. *If the new land does not land,* mend both on today's rivers.
4. **Map views and charts that say what is so** (F2; bears on note 13). One outlying town flattens the price map's scale; a price is shown for goods nobody makes yet; a wage chart that should read 1 in 1750 reads 1.04. *Trigger:* `night/calendar` and `night/town-ground` are merged, since both work in the map views.
5. **A look at each viewer piece once it has landed:** towns (F5), railways (F6), works (F8). What reviewers could not judge until the piece was in. The railways have landed, so F6 is ready: whether the whole network reads as too much on the map, the churches that now have a line against their yard, eighteen curves too tight for a train, roads that drape over the banks, sidings that are plain ribbons. *Trigger:* each landing; the railways' part now ranks below items 1 to 4.
6. **What the record carries for the viewer, and a test that holds it** (F9; note 16). The record keeps two land classes a cell where the picture uses four, clips rent and distance, and carries no traffic; the golden test would not notice a change to any of it. With it goes a row for each works and year (capacity, what is on order, hands, how hard it runs, its state: twelve bytes), which the probe of combined works found to be true today and which the viewer's site planner needs in place of its stand-ins. *Trigger:* mines, canals, the new land and the books of account are merged, since each adds to the record. *Why here:* the inspector, ownership on the map and anything that follows traffic wait on it.
7. **The inspector** (note 13): a market's industries as a flow, with prices, volumes and carting. *Trigger:* the books of account are merged, and item 6.
8. **A hand-over that keeps the view smooth** (note 17; F14). *Trigger:* the probe's design is written and checked, and the monthly clock and the buildings' levels of detail are merged, since all three change the same code.
9. **A save that outlives a change to the simulation** (F10). A saved game is its orders played again from 1750, so every core piece merged makes older saves drift, and loading a century takes 23 seconds. *Trigger:* `night/calendar` and the core pieces in review are merged; before the run to 2050. Large.
10. **A pass over all the game's rules together, and a scorecard.** First the sweep's own tools (F11: twenty-year means over sixteen seeds, tolerances, facts not named by year), then the things measured and never explained, which the books make cheap to trace (F12), then the pass itself (F18: growth that comes early and stops in the 1880s, the wage's dip with kept town ground, swings, famines, the steel rail, numbers never searched). *Trigger:* mines, canals, the new land and the books of account are merged.
11. **Combined works as a mechanism** (note 16; F4). The probe is in ([probes/dx-combines.md](probes/dx-combines.md)) and its critic says build it changed: a concern with books for each stage, stages kept in proportion, formed by neighbours merging or by a user building its own maker, families' works included. As prototyped it moves nothing on the island, so it is for the picture and for the tree's longer chains, and ranks no higher than this. Stock yards at the works are not to be built as first specified: a works holds no goods today, and the yards as designed would hoard in a shortage. *Trigger:* the production tree's first stage is merged.
12. **Roads and bridges that cost, and a railway company: someone who pays and someone who is paid** (notes 20, 21; F15, F16). Today nobody pays for a road and nobody is paid a railway's fares: see the answer under note 21. *Trigger:* the new land is merged (a bridge's cost should follow the river's width), canals are merged (their promoters are the pattern), and the public purse in the 2050 design has been read.
13. **A lab that shows what the game runs** (note 14; F13). The lab's streets, market places and woods are a second copy kept by hand, and will stop matching the game when the new towns and wood edges land. *Trigger:* `night/town-ground`, `night/wood-edges` and `night/works-detail` are merged.
14. **What the simulation knows and the picture does not show** (notes 13, 16; F17): machines, harvests, who owns the land, where a news item happened. *Trigger:* items 6 and 7.
15. **New towns and districts that change market** (F19), and **a harvest month** (F20; bears on note 10). *Trigger:* canals and the calendar are merged and the scorecard exists.
16. **A water industry** (note 7), **logging beyond sawn timber** (note 8), and what the tree's later stages inherit (F21). *Trigger:* the new land and the tree's first stage are merged.
17. **The first stage of the run to 2050** (note 1; F22). *Trigger:* Stella has read the design and answered its questions.
18. **Polish, in this order:** the clock, the cards and the present (F24); the news watched in the running game (F25); winter and the light (F26); a title screen and a start that shows something (F27); life and weather (F28). *Trigger:* each names the landing it follows on its own page.

One piece could go at once and was taken up the same evening, since the repository was about to be read by someone new: **documents that still describe the Python port** (F23).

## Waiting on Stella

In full, with where to look for each, on [backlog/waiting-on-stella.md](backlog/waiting-on-stella.md). None of it holds other work up: each has an answer assumed meanwhile, which is how the game stands today.

- **The core loop** (what a family does from month to month, and why it is fun) is hers. Nothing here touches it. Helpers found nineteen things that belong to it and they wait with it: land beats works on every island, a family that takes every default ends fourth of six, a player who founds a works every year loses most of them.
- **The 2050 design, to read.** Written on 12 November by a panel (two digests, three designs, one made from them, two critics, a revision): what the island would live through in five acts, the mechanisms in an order of work whose first stage is a day, what it would show and what it would not. It is on her machine at `.claude/night/reports/design-2050/to-2050.md` and is not committed here until she says it may be, since it draws on work of hers that is not published. Its first page is written to be read in five minutes and ends with four questions that are hers; sixteen more are in its section 15, each with the answer assumed meanwhile. Nothing of it is built until she has read it (Ready next 17).
- **Nine decisions about the rules** that reviewers left for her, among them: exports and the price rule (mending it rebalances the whole game); which price index the player is told (the wage of 1900 is 2.82 times 1750's on one and 2.08 on the other); how much of a harvest is foreseen and how hard a bad year should bite; a fixed freight rate while wages rise.
- **About twenty matters of taste,** each with a picture: the hill towns' darker slates and limewash; the October gold of the far woods; shop fronts that follow how dense a street is and not the simulation's shops; garden trees as big as a cottage; one strong orange for every autumn tree; a hamlet's church as big as a market town's; whether a lower sun should be the standard light.

## Found along the way

Not her notes: what helpers and reviewers reported and left open. On the evening of 11 November four pairs of helpers went through the reviewers' reports, the rules pages, the READMEs, the probes and the code's own comments. They gathered 609 rows and checked each against the code as it stands and the branches in flight: 105 were fixed, moot or no fault and are pruned, each with its reason; 83 are already with a branch in flight; the other 421 are 328 faults once the same fault told twice is counted once. Those are gathered into 28 pieces of work and a tail of smaller things on [backlog/found-along-the-way.md](backlog/found-along-the-way.md), and placed in "Ready next" above as F1 to F28. The pruned rows, and where every row went, are in [backlog/sweep-2026-11-11/](backlog/sweep-2026-11-11/README.md) with the four checked tables.

## Closed

- **3, houses with too little detail:** answered by the house kit, merged 11 November. Her later notes 14 and 16 take it further.
- **2, mines as big blobs:** answered in the simulation by the mines, merged 12 November: diggings take about 9 cells' worth of land in 1900 where they took 1,373. The picture still shows a working only as a stand-in, so the drawing stays open at Ready next 2.
- **5, land that looks ruled:** answered from the whole-island view by the far view, merged 11 November, and from mid distance by the wood, heath and hill edges, merged 12 November.
- **12, a railway like an aqueduct that then drops:** answered by the railways' profiles, merged 12 November: every cell of track has one level, a line winds or takes an incline where the ground is too steep, and the check finds no step anywhere. What is still wrong with them is at Ready next 5.
- **11, "I really like a lot of the new looks":** nothing to do but keep going that way.
- **15, examples of the generators:** five contact sheets sent; twelve are under `.claude/night/shots/lab-for-stella/` on her machine, and `isostate-viewer --lab list` names the rest.
- **16, in part: what the simulation knows of a works.** Measured on 11 November: a works holds plant and cash and no goods; being on a railway is worth about 0.3 per cent of its output. So stock yards and sidings are drawn from stand-ins and from where the track runs, and the picture should not claim more. The rest of note 16 is in flight or at Ready next 6 and 11.
- **18, the backlog itself:** this file.
- **22, pushing, and `main`:** `rust-3d` was moved onto `main` and pushed on the evening of 11 November; what is committed on `main` is pushed in the same step, and there are no pull requests.

## Pruned

- **The first reading of note 14** (bring the houses down wherever they are finer than the works): overtaken within the hour by note 16, "I love the houses". The houses are the reference and are not to change.
- **105 rows of the helpers' findings,** fixed since they were reported, made moot by later work, or no fault: named with their reasons in the sweep's appendix.

## The notes, as written

**10 November, evening, handing over for the night.** "There's definitely some issues with the core game loop not quite being fun, but I think for tonight you should work on everything else".

**11 November, about noon.**

1. "I want to also run the game further, into 2050 or so, so we can do computing and AI, so it can capture the full course of dynamics that we have in my labor repo."

**11 November, after playing the morning's build.** "Just some ideas and thoughts":

2. "mines still look like big blobs. perhaps the deposit is big but the surface mine area might just be at the base of the mountain."
3. "houses seem to have less detail than industries now, too much less."
4. "Do rivers go downhill? The meandering feels a little coarse, can we simulate real river dynamics?"
5. "forests and farms look too regular when zooming out. I think there was a good fff on the topic" (Factorio's Friday Facts 390).
6. "lakes and large rivers and canyons and plateaus and that sort of thing would be really nice."
7. "Watersheds and water industry. Probably need a water cycle for that. floods, and dams and reservoirs and irrigation."
8. "logging industry"
9. "canals, prior to railroads."

**11 November, afternoon, after playing again.** "don't read too much into it right now":

10. "the game still feels like it goes too fast, I want to at least know what month it is, maybe week. victoria does 100 years as days and weeks for their ticks, maybe we can approach that even though we're doing a few hundred years more."
11. "I really like a lot of the new looks, still more to be done but definitely in the right direction"
12. "there was a rail line that looks like an aqueduct in one place and then suddenly drops down. steeper gauges maybe need special trains or more winding paths up hills, or other solutions"
13. "we'll need more ways to inspect what's going on in the UI for me to be able to give input on if things make sense. Flowcharts of industries within a region or market with prices and volumes and transport costs, im not sure, I leave that up to you".

**11 November, late afternoon, on seeing the new houses.**

14. "I am noticing now the detail level of the houses is now higher than the industry, whilst their variation still feels low (multi stories outside the city?). I think a consolidation pass is needed, or some way of ensuring all art assets in future get the same detail scale."
15. "btw, can i get some examples of the tiny glade like asset generation? just to get a feel for it?"
16. "OK, I love the houses, so i think industry just needs to be brought up to the same detail level. And I think industrial works in general will need more interesting procedural algorithms to really make them look natural and fill out the space they are given (preferably in a way that reflects the underlying data as much as possible, idk, inventory sizes, different stages of production using different technologies, dilapidation, train stations, truck loading stations, industries combined not needing transport between them, i.e vertical integration, but they're less flexible or whatnot, again these are all just ideas, I'm leaving decisions up to you)".
17. "I have a feeling we'll need to separate the threads doing the simulation, the graphical updates, and the drawing itself, (or something like that, i'm not a graphical dev !!!) so that even if the game needs to slow down the computation the view is kept smooth."

**11 November, evening.**

18. "I like to be able to write things i notice, but i feel like you pick them up too soon, rather than putting them into a backlog and taking them only when it's the opportune time. So i want you to make sure you have a proper backlog and prune it for things that are no longer relevant if another pass already changed something so much that my input is no longer relevant, and decide what to prioritise."
19. "I saw a horse and cart sail over a river and then clip through a bridge".
20. "bridges are expensive, roads often run alongside them." Read as: because a bridge is dear, a road should keep to one side of a river and cross where it must, where the game lets roads cross freely. If she meant roads drawn beside their own bridges, that is the drawing fault of note 19.
21. "And who's paying for bridges and roads?" As the simulation stands, nobody. A cell's road becomes a track, a road and then a turnpike when the traffic over it passes a threshold, and falls back after six quiet years (`sim.rs`, `update_roads`); no money, material or labour goes into it. A river never costs a road anything (it only makes carriage cheaper where it is navigable), so a road bridge exists in the picture and nowhere in the economy. Railways are paid for under the game's rules: the landlords and households of a line's two towns subscribe, the line buys its rails, timber and stone month by month, and a river crossing makes it dearer to lay ([rules/railways.md](rules/railways.md)); nobody collects fares yet.
22. "there should be a standing rule that if something is committed, it's pushed. We won't really use PRs in this project, and tbh, you should probably move the rust 3d branch onto main since, well, i like it".
