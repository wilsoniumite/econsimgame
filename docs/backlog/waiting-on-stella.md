# Waiting on Stella

Part of the [backlog](../BACKLOG.md). Gathered on the evening of 11 November 2026 from what helpers and reviewers left for her, added to on 12 November from the day's landings, and shrunk that evening to what is truly hers. She has handed the project's decisions to the managing session (the director, on the pages of those days): she will seldom weigh in, and a thing she says must be done one way is done that way.

On 19 November it was shrunk again, to what truly needs a person: something to be tried at a real window, and her own taste where she has stated it (the houses, which are the reference and change only on her word, and the look). Her notes of that day put the family game in the back seat, with all its rules and decisions up for grabs, and lifted the wait on the 2050 design. So what waited here only on a decision about the family game, the month-to-month loop or the reading of the 2050 design went to the backlog, under "Released on 19 November" (the core loop and its four hangers-on under "The family game, parked"); and what was left here for the director, with the record of what it decided on 12 November, went there too, under "For the managing session". None of what is left holds other work up. A letter and number in brackets is a row of one of the checked tables in [sweep-2026-11-11/](sweep-2026-11-11/).

## Needs a person at the window

No helper can judge these: each is how the game feels when it is played or watched at a real screen.

- *The sandbox, played at a window.* Added 19 November. Since that morning `--sandbox` opens a game nobody plays a family in, with a tray of tools under the clock (Look, Money, People, Harvest) and Run on for 1, 10 or 25 years (the viewer's README; [designs/sandbox.md](../designs/sandbox.md), section 6.1). Every picture of it so far was taken with no window, by a script; nobody has picked up a tool, clicked a town or used Run on at a real screen. She asked for it to make testing easier for her and because it may be fun in itself, and only she can say whether it is either.
- *The new island, played at a window.* Since the night of 12 to 13 November a game can be started on the land made by water with `--new-land` (the viewer's README, "The new land"): the ground, rivers, lakes and sea from the core's heights, each cell's meeting place on dry ground, and railways that find their way by the ground. It is a preview, never kept in the island's autosave (`--save FILE` keeps it in a file of its own). Every picture of it so far was taken with no window; nobody has played it, panned over it or watched a year turn on it at a real screen, and her notes on the game wait until she has seen it (her note 24).
- *The smooth hand-over.* Its first stage landed; nobody has watched it at a screen. Whether the half second in which a new year's works stand among last year's trees is acceptable until a later stage cures it; and whether, at the fastest speed, a year shown smoothly about one time in two and a half is better than nearly every year with a third of a second's stop. F3 shows the frame strip. (`docs/probes/dx-smooth.md`, section 12)
- *The clock as it plays.* A pause late in a year moves the clock on to the next year (L151); at the two fastest speeds pausing lands on the next January, since at those speeds the year's end is asked for half a year ahead, and at the three slower speeds it happens only in the last weeks of December. The speeds' names (Slow, Steady, Brisk, Fast, Fastest), a game starting at Steady, and a game opening on "January 1751" are the builder's choices.
- *Winter at every turn.* A clock stopped at a year's turn stands at January, so the opening card, every decision card and every moment an order is carried out sit over a winter island. The alternative is to ease to high summer wherever the bar shows a plain year, as the stills do.

## To show her

Matters of taste, her own ground since she has said what she likes (the cosy diorama, and the houses as they are drawn): the managing session decides each and shows her the picture. The pictures are under `.claude/night/shots/` on her machine and are not in the repository, except the last. Where one would change how a house is drawn, it is changed only on her word, since the houses are the reference.

- *October,* the far canopy a bronze and the crowns bright orange: `wood-edges-review/land/lowland-1800-october.png`.
- *Autumn is one strong orange on every broadleaf* (R144, L106): `art-integrate/views/home-1900-autumn.png`, `wood-round-2-autumn.png` (`tint_autumn` in `look.toml`).
- *Winter woods are thin brown sticks at play distance* (R144; the mend is in piece 26): `gen-polish-review/lab-final-winter/woods-play.png`, `art-integrate/views/wood-round-3-winter.png`.
- *A wood's edge cuts arable fields,* where it might have followed the hedge; with hedge-following the stair-stepped blocks stay: `wood-edges/land-2/lowland-1800.png` shows that rule.
- *Hedgerow trees away from woods are down to about 60 per cent* (L37): `merged-7-towns/town-1750.png` against `merged-8-houses-far/town-1750.png`.
- *The hill towns' darker slates and limewash* (L56): `gen-houses-review/views/hill-1800-play.png`. To undo, `slate.stone` back to `#a59c8a` in `look.toml` and `UPLAND_BARE` to 1.0 in the houses generator.
- *Shop fronts by how dense a street is* (L20, R122), drawn from each cell's services capacity rather than the simulation's eighty large shops: `lab-for-stella/street-play.png`.
- *Garden trees as big as a cottage,* drawn island-wide at every level (L65): `lab-for-stella/street-close.png`.
- *Villas follow the year built, not the rents; garden trees, privies, woodpiles and beds stand for nothing in the simulation* (R147, L21): `lab-for-stella/villas-play.png`.
- *A lowland town before 1815 is one warm orange-red field, and dense towns show no thatch* (R145, L96): `merged-8-houses-far/town-1800.png`, and as the town ground draws it today, `merged-15-town-ground/town-1800.png`.
- *The town of 1900 at play distance is ribbons of roofs among closes, not a mass,* which is what the record says (`world::BUILT_GROUND`, or twelve people to a drawn house, are the levers): `merged-15-town-ground/town-1900.png`.
- *A town's closes in the hills are the lowland's bright green* with hedges, in hard-edged blocks against the grey works ground: `town-ground-review/views-before-fixes/garside5-1900.png`.
- *How strong the far wash of a town's roofs is* from across the island: `merged-15-town-ground/island-1900.png`.
- *A hamlet's church is as big as a market town's* (R146, L108): `lab-for-stella/landmarks-play.png` shows the church; no picture of it beside a hamlet was kept.
- *Granness's church stands by the shore with a corner of its yard on wet ground* (L105): `art-integrate/views/coast-town3-1900.png`.
- *The pennant's size, and whether an early water mill wants a pond that large* (R141, R142, L85): `merged-8-houses-far/works-1900-close.png` and today's `merged-15-town-ground/works-1900-close.png`; `lab-for-stella/works-mill-close.png` for the pond.
- *Early stone works read weakly beside brick ones* (L70): `lab-for-stella/works-kinds-play.png`.
- *A works at size 0.4 is smaller than a town house* (L77): `merged-8-houses-far/street-1850.png`.
- *Coke-furnace chimneys are tall and many* in a game of thousands of small furnaces (L78): `merged-8-houses-far/works-1900.png`.
- *Handloom weavers and shops are alike in outline at great sizes,* both a U round a court (L84): the works-great sheet, `gen-works-review/lab/works-great-close*.png`.
- *Tall viaducts over open country* where a line is losing height; a lower limit trades them for more inclines: `rails-review/after/tallest.png`.
- *How much a line wanders on low ground* to follow the contour, and how tight a winding may be: `rails-review/after/coast.png`.
- *How white the snow is,* a colder white drawn brighter than its paint (`winter.snow`, `winter.snow_bright`): `winter-fixes-review/a/street-jan.png` against `final/street-jan.png`.
- *Whether a thaw has edges* or is a soft wash: `winter-fixes-review/b/street-thaw.png` against `d/street-thaw.png`.
- *White roofs among fields that only frost, and conifers that carry no snow:* `winter-fixes-review/final/works-jan.png`.
- *A lower sun shows the relief far better than the standard light* (L163): `look-demo/island-1900.png`, shot with `look-demo/dusk.toml`, against `merged-8-houses-far/island-1900.png` (`[sun]` in `look.toml`).
- *One outlying town flattens the price map's scale* (L119): `overlays-charts-review/price-steel-1750.png`; the true range or quantiles, as the rent view has.
- *The README's front picture* was replaced on the evening of 12 November by Melbourne in 1860 as the game draws it today: `docs/shots/melbourne-1860.jpg`, in the repository.
