# A life cycle for works

`works.enabled`, in `crates/isostate-core/src/works.rs`. Off under the port,
on under the game's rules. None of it is in the Python.

Under the port the island ends the century with four and a half thousand
works, seven thousand with six families, nearly all of them small, standing on
a sixth of the land. A works is founded at two fifths of its trade's usual
size and then hardly grows; what grows is their number. With the life cycle a
works that earns its keep grows where it stands, year after year, until its
cell is full, and then opens a sister works next door. In 1900 there are about
650 works, the median one seven times its trade's usual size, a few of them a
hundred times, on about 125 cells beside the towns.

## Why the port's works stay small

The diagnosis is in `.claude/night/reports/dx-firms.md`, measured on eight
seeds. In brief:

- **A works never finishes building.** An order for capacity is delivered a
  quarter of the remainder each month, so it is "open" for five to seven years,
  and a works with an open order may not grow, is never counted idle and is
  never closed as too small. A works can grow about once in six years and
  loses a fifth to depreciation meanwhile.
- **It may not pass four usual sizes**, and a usual size is a handful of
  hands in several trades (a smithy five, an engine works eighteen).
- **Families found a small works a year in every branch**, half of all works
  standing in 1900.
- **A works of the usual size takes a quarter of a cell whatever its trade**:
  a five-man smithy stands on a square kilometre.

So growth in demand is met by more small works, and the map fills with them.

## What the life cycle does

Six parts, lettered as in the diagnosis. Each has its own number or switch in
`WorksParams`, so `--set works.walk=0` (say) measures the rest without it.

**A. A works has finished building when little is left on order**
(`built_share` 0.05). `works::building`: a works counts as building while more
than a twentieth of its capacity, built and on order together, is still to
come. The yearly review (`firms::review`) can then grow a works every year,
shrink it when it stands idle and close it after three losing years, which
the port's flag had been preventing.

**B. Land by hands, and the cell as the size limit** (`hands_per_cell` 2,000,
`cell_limit` 0.8). `works::land_take` is the one place that says how much land
a works needs: its capacity times its trade's hands per unit, over the hands
that fill a cell. A smithy of five men takes a yard; a mill of a thousand
hands half a cell. The rent it bids, the cell's built share
(`State::cell_K_share`), the size limit and the room a newcomer looks for all
come from that function, and `works::land_held` is what one standing works
holds: its capacity built and on order, because the ground is taken the day
building starts. A works grows where it stands until it would take four
fifths of its cell (`works::max_K`) or the cell is full of its neighbours
(`works::cell_fits`). After that the growth goes to a works of the same
concern in the town that still has room (`works::extend_sister`), and if there
is none, to a new sister works the size of that growth on a cell within two of
it that has room (`firms::spawn_sister`, `works::sister_K`). No cell is ever
asked to hold more works than it has ground for; `tests/works.rs` checks that
every year.

A works that grows needs more land, and pays for it: `State::works_lease` is
the most land each works has taken so far, and an order that takes it past
that adds to its monthly rent at what the cell's land earns that year (the
bid a new works makes there). Under the port a works' rent is fixed for life
at what it bid when it was a tenth of the size.

**C. Entry counts the merchant's spread** (`count_spread`). The merchant takes
6 per cent of the price off everything a works sells (`merchant_spread`), and
`firms::evaluate_sites` left it out, so a site promised tens of points of
return more than a works there would get. `works::entry_spread` takes it off
the price entry reckons with. The families' site sums come from the same
function, so the player's "expected return" is now the return a works gets.
With this part alone switched off (eight seeds) 35 per cent more works are
founded and 41 per cent more close, a year and a half younger, for the same
output: the works the false promise called up.

**D. A year's room for new capacity** (`year_room` 0.15, `room_share` 0.34,
`entry_share` 0.10). Each trade in each town may order in a year at most 15 per
cent of the town's monthly trade in what it makes (`State::outflow`, sales and
shipments smoothed), and never less than one smallest works, so a trade nobody
is in yet can always start (`works::year_budget`). The works already there are
served first, in order of the return they earned on their capacity last year
(`works::new_year` sorts the review), no one of them taking more than a third
of the room (`works::growth_allowed`). Entry takes what is left
(`works::sites_with_room`) and builds one works sized to its market, a tenth of
the town's trade within the room and the cell (`works::entry_K`); the port's
doubling of a works that promises over 60 per cent, and its second works a
year, are dropped. `State::works_added` keeps the tally.

This is the part that decides how hard the island invests. Without it (four
seeds, no families) output per head reads a third higher but the real wage is
no higher, a quarter of all capacity stands idle and twice as many works close.
Most of the extra looks like capital goods for capacity that then stands idle
(I did not take the accounts apart). A room of 0.3 instead of 0.15 (eight
seeds, `sweep-works-room30`) gives output per head 3 to 6 per cent higher in
1800 and 1850 and the same in 1900, no gain in the real wage, a quarter more
closures, a fifth more industry cells, and prices that swing 8 to 17 per cent
harder (grain and cloth in every seed). 0.15 is the steadier place to stand.

**E. A growing family branch extends a works it has** (`family_extends`). In
`families::grow`, where the branch would have founded another works of a
method it already runs in that town, the capacity is ordered at its largest
works there that has room, paid from family cash at the same price
(`works::family_extension`, `works::extend_for_family`); the player hears of
it. A branch founds a new works only where it has none, or none with room.
The size of a family's works, and the site sums the player sees, are
`works::entry_K`. The player's own choices are untouched, with one addition: a
works needs room on its cell, and the site list says so where there is none.

**F. Hands walk to work** (`walk` 0.02). Hands live in the town. Labour per
unit of output rises by 2 per cent for each unit of path cost between the
works and its market (`works::walk`, applied in `firms::evaluate_sites` and to
every works' labour in `econ::step_month`). A light trade drifts no further
out than the walk is worth, so mills, steel, engine works, smithies and shops
stand in and by the town (97 per cent of hands within two cells of it), while
coke furnaces stay towards the coal and brickworks on the clay.

Every payment here is one the port already makes, between purses
`econ::money_total` counts: an order is paid from the works' cash, its backers'
or its family's, and rent goes to the land's owner. Switched off, every
function in `works.rs` returns the port's number and the two state tables stay
empty; `tests/golden.rs` holds the port to the byte.

## What was tried and left out

- **Amalgamating small works, stopping economies of scale at four sizes,
  sharper sampling of sites**: the probe found no effect from any of them.
- **A size limit of their own for craft trades** (shops, smithies, handlooms
  and the like stopping at four usual sizes). Measured on four seeds: 790
  works in 1900 instead of 645, with 126 shops instead of 79 and 89 smithies
  instead of 36, but 2,423 works closed in a run instead of 1,390, and a wave
  of 500 closures in the 1860s. A craft works at its limit throws off sister
  works that die. Not worth it. A "works" of shops is then a street of them:
  about 80 on the island in 1900, some 750 hands each. If shops on every
  town cell matter to the picture, draw them from the services capacity of
  the cell, not from the count of works.
- **Utilisation net of the spread.** A works decides how hard to run on its
  margin over the gate price, before the merchant's 6 per cent
  (`econ::step_month`, `firm_demand`). Taking the spread off there is the same
  as raising `util_target_margin` by six points. On four seeds it left two
  points more capacity idle and closed 13 per cent more works, with output and
  wages inside the spread between seeds. Left as the port has it: the works'
  books do count the spread, and its rule of thumb for how hard to run is a
  number to tune, not a sum to correct.
- **A warm-up under the port's rules.** The warm-up decade runs under the
  rules of the run, so 1750 opens with about 280 works, not 600. Tried the
  other way (four seeds): 1750 then opens with 558 works, the millwrights a
  mean path cost of 27 from their market, and 294 works close in the 1750s
  (82 otherwise) while the trades move to town. The player's first decade
  would be a clearance that no history
  explains. Starting in the life cycle's own equilibrium is right.

## Before and after

Eight seeds (1 to 8), the port against the port with `works.enabled=true`,
without families and with six. Means, with the least and greatest seed in
brackets where it matters. The sweeps are `sweep-works-base`, `sweep-works`,
`sweep-works-base-f6` and `sweep-works-f6` under `.claude/night/reports/`.

| | port | works | port, 6 families | works, 6 families |
|---|---|---|---|---|
| works 1750 / 1800 / 1850 / 1900 | 597 / 1,536 / 2,804 / 4,512 | 283 / 316 / 493 / 643 | 598 / 2,131 / 4,156 / 6,973 | 283 / 330 / 582 / 673 |
| works in 1900, range | 4,076 to 4,933 | 594 to 770 | 6,285 to 7,607 | 619 to 715 |
| per thousand people, same years | 4.4 / 6.7 / 7.1 / 6.6 | 2.1 / 1.4 / 1.3 / 0.9 | 4.4 / 9.3 / 10.5 / 10.2 | 2.1 / 1.4 / 1.5 / 1.0 |
| size in 1900, median (K over the usual) | 0.84 | 6.7 (5.3 to 7.9) | 0.46 | 6.6 (5.7 to 7.4) |
| 90th percentile | 2.9 | 40 (32 to 50) | 2.5 | 45 (37 to 61) |
| largest | 3.69 in every seed | 109 (86 to 123) | 3.69 | 119 (109 to 123) |
| capacity held by the largest 1 per cent | 0.080 | 0.115 | 0.058 | 0.140 |
| works founded in a run | 8,655 | 1,758 | 11,879 | 2,005 |
| works closed | 4,710 | 1,397 | 5,472 | 1,613 |
| median life of the closed, years | 11.6 | 9.9 | 13.1 | 9.5 |
| industry cells 1750 / 1800 / 1850 / 1900 | 204 / 332 / 543 / 886 | 2 / 12 / 63 / 125 | 204 / 359 / 601 / 981 | 2 / 18 / 70 / 135 |
| farm area in 1900, cells | 1,282 | 1,417 | 1,251 | 1,386 |
| output per head over 1750, in 1800 / 1850 / 1900 | 1.40 / 1.48 / 1.46 | 1.55 / 1.60 / 1.60 | 1.39 / 1.55 / 1.50 | 1.64 / 1.64 / 1.57 |
| real wage over 1750, in 1800 / 1850 / 1900 | 1.33 / 1.34 / 1.29 | 1.42 / 1.51 / 1.57 | 1.31 / 1.35 / 1.35 | 1.49 / 1.42 / 1.47 |
| price swing: iron | 0.219 | 0.225 | 0.202 | 0.238 |
| rails | 0.204 | 0.214 | 0.200 | 0.231 |
| cloth | 0.083 | 0.105 | 0.084 | 0.103 |
| grain | 0.086 | 0.092 | 0.090 | 0.098 |
| towns in 1900 | 19.2 | 17.8 | 19.8 | 17.8 |
| last town founded | 1798 | 1832 | 1802 | 1837 |

What is clear of the spread between seeds: everything about the works (count,
size, founding, closing, land), the farm area, and the cloth price swinging a
quarter harder. The real wage in 1900 is higher in all eight seeds without
families (1.57 against 1.29) and in six of eight with them; in 1850 with
families it is higher in three seeds and lower in five, which is noise. Iron
and rails swing 3 to 5 per cent harder without families, inside the spread,
and 16 to 18 per cent harder with them, in seven or eight seeds of eight.

By decade (eight seeds, no families) output per head is 5 to 11 per cent
above the port's from 1800 on, and the real wage 9 per cent above in the 1800s
and 15 to 20 per cent above from the 1870s. One thing behind it shows in the
goods: smithies that can grow make two thirds more tools in the 1770s and
three times as many in the 1790s (2,855 a month against 928), and tools are
what raises output on the land.

In 1900 with six families (four seeds: 3, 7, 11, 21; `examples/works_census.rs`)
the island's 676 works are 164 crucible steel works (median 23 usual sizes,
the 90th percentile 70), 122 mills, 98 engine works, 81 brickworks, 76 streets
of shops, 72 coke furnaces, 36 smithies and 21 rolling mills. 178 are sister
works and 182 belong to families (about 3,500 in the port, by the probe's
count). The works that close are
the small fringe: a median life of ten years, while the large ones last.

### Output per head after 1865

The probe saw output per head peak around 1860 to 1870 and sag by 1900 under
its prototype, where the port was flat. On seeds 1 to 8 this build does not
show a sag of its own: the decade means from the 1820s to the 1890s are 6.8,
6.8, 6.6, 6.6, 6.7, 6.8, 6.8, 6.7 against the port's 6.3, 6.3, 6.3, 6.3, 6.3,
6.3, 6.3, 6.2, and both lose 2 per cent between the 1870s and the 1890s. On
the probe's own four seeds there is a hump (7.6 in the 1870s, 7.1 in the
1890s) that the port lacks (6.7 and 6.7), and it is there with the year's
room off and with the walk off, so neither causes it. What the works rules
change in those decades is the arrival of farm machinery: engine works can
now grow, so machinery's price peaks at 230 in the 1860s instead of 480 in
the 1870s, and the 1870s are a good decade instead of a famine. After that
nothing new raises output on the land while people go on multiplying
(`dx-growth.md`), and output per head drifts down at the port's rate. I did
not run the counterfactual without farm machinery, so take this as the likely
reading, not a proof.

### The scripted players

`examples/bots.rs` on seeds 1, 2, 3 and 7, `--rules port` against `--rules
game` (which on this branch is the port plus the life cycle). Net worth in
1900, the player's rank among the six families in 1775/1800/1850/1900, works
founded and standing:

| seed | strategy | port: net worth | rank | works | works: net worth | rank | works |
|---|---|---|---|---|---|---|---|
| 1 | passive | 24.1m | 4/3/2/2 | 1,101, 837 | 22.1m | 2/3/2/2 | 49, 27 |
| 1 | landlord | 37.9m | 2/2/2/1 | 919, 688 | 42.7m | 1/2/1/1 | 54, 41 |
| 1 | founder | 24.6m | 3/2/2/2 | 1,256, 896 | 27.9m | 2/2/2/2 | 158, 43 |
| 1 | thoughtful | 35.0m | 2/2/1/1 | 987, 645 | 35.6m | 1/2/2/2 | 195, 40 |
| 2 | passive | 22.4m | 1/1/1/1 | 856, 701 | 24.3m | 1/1/1/1 | 40, 22 |
| 2 | landlord | 28.3m | 1/1/1/1 | 941, 746 | 28.9m | 1/1/1/1 | 39, 24 |
| 2 | founder | 23.0m | 1/1/1/1 | 1,022, 811 | 19.5m | 1/1/1/1 | 127, 34 |
| 2 | thoughtful | 21.9m | 1/1/1/1 | 817, 573 | 27.0m | 1/1/1/1 | 151, 43 |
| 3 | passive | 17.4m | 4/1/2/3 | 908, 680 | 24.1m | 2/1/1/1 | 47, 25 |
| 3 | landlord | 23.9m | 2/1/1/1 | 841, 672 | 33.9m | 2/1/1/1 | 44, 31 |
| 3 | founder | 16.7m | 4/1/3/2 | 1,004, 741 | 17.8m | 2/1/2/1 | 141, 41 |
| 3 | thoughtful | 21.7m | 2/1/1/1 | 838, 562 | 31.0m | 2/1/1/1 | 151, 58 |
| 7 | passive | 17.2m | 4/2/4/4 | 922, 734 | 21.8m | 4/4/3/1 | 39, 24 |
| 7 | landlord | 24.5m | 4/2/1/1 | 916, 708 | 29.8m | 4/4/1/1 | 38, 22 |
| 7 | founder | 16.8m | 4/3/4/4 | 980, 739 | 19.9m | 4/4/2/3 | 139, 37 |
| 7 | thoughtful | 23.5m | 4/2/1/1 | 987, 652 | 31.9m | 4/4/1/1 | 164, 48 |

The player ends in the same range or a little richer (the landlord and the
thoughtful player gain most; the land is still the best asset), with ranks
within a place or two. A family now holds a few dozen works, each of them
something, instead of seven hundred. The richest rival in 1900 moves both
ways (seed 1: 31 to 38m in the port, 24 to 49m here).

## Beside the other mechanisms

- **The price rule** (the stock markup and the merchants' glut rule that
  `dx-cycles.md` recommends) works month by month on prices and on how hard a
  works runs. The year's room works once a year on how much capacity is
  ordered. Both read `State::outflow`, neither writes it, and they do not
  touch the same lines. They pull the same way: the room is the damper on the
  investment side of the cobweb. It was tuned here without the price rule, at
  the point where a wider room starts buying idle capacity; with steadier
  prices a wider room (0.2 to 0.3) may give more output for the same
  steadiness. That wants one sweep of `works.year_room` once both are merged.
- **Kept town ground** (`dx-towns.md`) can read `works::land_take` for the
  land any works needs and `works::land_held` for what a standing works
  holds; `cell_K_share` is the sum of the second over a cell's works and
  never passes 1. With the walk, the towns report's "town trades on town
  ground" rule should not be needed: 97 per cent of hands already work within
  two cells of their market. Only 14 per cent are on town ground itself,
  because the town's own cells are few; a town that keeps its ground will
  grow over the works beside it.
- **New towns** are founded where work stands far from any market. With
  hands walking there is less of that, so the island has one or two towns
  fewer in 1900 and founds them later (the last around 1835 instead of 1800).

## What worries me

- **Prices swing harder with families**: iron and rails 16 to 18 per cent,
  cloth a quarter (without families too). Larger works order larger lumps. The
  year's room holds it there; the price rule is the cure.
- **The converter is never built** (0 seeds of 8, with or without families;
  the port builds it in 3 of 8 and 2 of 8, after 1883, and it never makes half
  the steel). It is not the spread: with the spread left out of entry's sums
  it is built in one seed of eight, and with a year's room of 0.3 in one. I
  did not find the cause. Steel rails are made in 3 seeds of 8 with families
  and none without (the port: none and 1). Steel needs its own fix.
- **Coke iron and engine works come 6 to 12 years later, mills 6 to 9 years
  earlier.** Discovery follows cumulative output, and under these rules the
  1750s make a third more cloth, a tenth less iron, two fifths less coal and
  half the machinery. Why the early coal is lower I did not find out.
- **A player who founds a works every year loses most of them.** The founder
  bot founds 127 to 158 and has 34 to 43 standing, and ends below the passive
  player in three seeds of four (in the port the two are level). Founding
  beside your own works is now a poor move, and extending is what the branch
  does for you. The player has no "extend this works" button of their own;
  that is a change to the player's choices and was left alone.
- **A new works costs more.** A tenth of a town's trade is many usual sizes in
  a large town. The bots could always afford their first works in 1750, but I
  did not look at what a first works costs the player in a big town late on.
- **Utilisation is lower** (the mean over works 0.76 against 0.81, 0.72
  against 0.84 with families): the small fringe stands half idle before it
  closes. By capacity about 13 per cent stands idle against 11.
- **Sister works are still between a quarter and a third of all works** (178
  to 203 of 650 to 680),
  most of them crucible steel in towns whose cells are full. A town that
  keeps its ground and grows should thin them.
- **The viewer has not been looked at with these rules.** It builds. A works
  can now be a hundred usual sizes and a cell can hold several large works,
  so the tiers, the plots and the works list want a look.
- **Numbers not swept**: `hands_per_cell` (the probe found 1,000 gives about
  250 industry cells and 1,050 works, 4,000 gives 55 and 510), `cell_limit`,
  `room_share`, `entry_share` and `walk` (0.01 leaves a third of hands more
  than two cells out; 0.05 pulls everything into town) are the probe's values,
  checked here only as a set.

## Measuring it

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --out DIR/base
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --set works.enabled=true --out DIR/works --compare DIR/base
cargo run --release -p isostate-core --example works_census -- --decades --set '{"works":{"enabled":true}}' 3 7 11 21
cargo run --release -p isostate-core --example bots -- --rules game
```

The census prints, for each method, the works standing, their sizes, their
hands and where they stand, and decade by decade the works founded, the
sister works, the closures, output per head and the real wage. Expect 600 to
770 works in 1900, a median size of 5 to 8, 120 to 150 industry cells, a real
wage 1.4 to 1.6 times 1750's from 1850 on and under 2,600 works founded in a
run.
