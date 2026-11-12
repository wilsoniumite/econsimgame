# tr-loose-b: kept town ground and the wage, crucible steel found late, what railways do to people and merchants, and shortages later in the year

One of the traces with the books of account (piece 12 of `docs/backlog/found-along-the-way.md`): four things the traces and tables of 12 November measured and left untraced, each of which bears on the pass over all the rules now being designed. Branch `night/o-trace-loose-b`, cut from `main` at 8bf4505 on 12 November 2026. Nothing under `crates/isostate-core/src` is changed on the branch. Four examples were added to measure with, each with a line at its head saying what it prints: `crucible_late.rs`, `spring_hunger.rs`, `kept_ground.rs` and `merchants_take.rs`. Two mends were tried in the worktree and taken out again (section 1). Everything is under the game's rules with six families unless it says otherwise. The raw output (every sweep, every table, the trial's diff and a file of notes with each command in order) is under `.claude/night/reports/opus/trace-loose-b/`, which is not in the repository.

The four questions:

- **(a)** Kept town ground costs the wage 2 to 5 per cent over some twenty-year spans and one works in thirty, and its two small switches are not the cause (`tr-works.md`, R102). What carries it: the farmland lost under town cells, rent on built ground, works kept out of full cells, or the people the ground lets a town hold?
- **(b)** Crucible steel stalls on most islands when it is found after about 1778, and the late find itself is the cause (`tr-steel.md`). Why?
- **(c)** Railways raise the real wage about 8 per cent yet leave fewer people, and under the port the merchants take 40 per cent of the gain (`tr-carriage.md`). Why fewer people, and why do the merchants' shares differ between the two sets of rules?
- **(d)** On islands 17 to 24 three spring shortages before the harvest were not followed (`tr-rent.md`), and four hungry town-years on islands 17 to 32 were not doublings at a redraw (`tr-carriage.md`). What are they?

## In short

1. **Kept ground's sure costs are output per head and works; its cost to the real wage is the 1750 basket's, and which suspect carries it is not settled.** On 64 islands with six families, output per head from 1840 to 1859 is 4.3 per cent lower with the ground kept (lower on 46 of 64, a standard error of 1.2 points) and 2.1 per cent over the century (0.6); the works standing are 5 to 6 per cent fewer in every span, with families and without (7 per cent over the century without, fewer on 52 of 64). The real wage on the 1750 basket is 4.1 per cent lower in 1840 to 1859 (1.4), and 1.4 over the century; with no families it is no lower over the century (0.1 per cent), and the "1860 to 1900" of the earlier pages does not survive the larger sample. But on the chained cost of living, which follows what households buy, the 1840s' cost to the real wage is 1.6 per cent (1.5) and the century's nothing: much of the wage cost is grain, 20 per cent dearer in the 1840s, weighed at 1750's share of the budget. A trial that lets a town cell keep its unbuilt ground under its old uses gives back every cell's worth of open land and more grain grown (3 per cent more than kept ground in the 1840s, on 47 islands of 64), but set beside kept ground on the same islands it makes grain no cheaper and the cost of living no lower, and its 2.2 points on the 1840s' real wage are 1.3 standard errors; the two small switches off do about the same (2.8 points, 1.7 standard errors). Neither touches the works. Rent on built ground is not a cost to households at all: kept ground's towns charge less house rent, 6.4 per cent of the wage bill against 7.3. (A check of this page overturned its first reading, that half the cost is the farmland under town cells: that came from setting two noisy means against "ground not kept" instead of against each other.)
2. **Crucible steel found late stalls because cheap iron has made tools cheap, and tools are what steel must beat.** Before rails, steel's only buyers are households and landlords wanting implements, where a ton of steel does three times a ton of tools' work and the cheaper per unit of work takes nearly all of the trade. Tools are made of iron. Once the coke furnace has made iron cheap (5 to 7 ten years after a late find, against 15 to 28 after an early one on all but one of the fourteen islands that grow), tools cost 8 to 10 and steel can fetch only 75 to 95; a crucible works at its first skill (0.5 to 0.8) needs 75 to 115 of hands a ton, so new works lose, they make next to nothing, and nothing is learned. Found while tools cost 15 to 20, steel fetches 110 to 150, the first works lose only a little, they go on making a few tons a year, and in about ten years their skill has climbed far enough to pay. Shown both ways: with crucible steel found late everywhere, five of five late islands stall; keep iron dear by delaying the coke furnace, or keep tools dear (twice the smithy's hands) with iron as cheap as it was, and the late islands grow; bring the coke furnace early, and early finds stall. A new works' size and a market already filled are not what separates them: every new crucible works is built at the floor size on early and late islands alike.
3. **Railways leave fewer people because they put more of them in towns, where they die faster.** With railways the share in towns is 0.421 in 1880 to 1899 against 0.407 without, and the people rules charge extra deaths for town life (`people.town_deaths`). Set that to nothing and the sign turns: railways then leave 0.6 per cent more people, on fifteen islands of sixteen, where today they leave 0.6 per cent fewer on fifteen. **The port's merchants take 40 per cent of the gain because the railway widens the margin on what they carry between towns, and under the game it does not.** Under the port with railways that margin is 7.6 per cent of income against 5.7 without, 34 of the 41 points of the merchants' part; under the game it is 4.5 against 4.8, nothing. No one of the game's mechanisms added to the port takes the merchants' part below a quarter (ten tried, 25 to 68 per cent), and the game with its works' life cycle and its people rules both taken away gives them a fifth: it is the two together that compete the margin away.
4. **The shortages later in the year are of three kinds, and all of them are the bottom of the merchants' stock cycle.** On 32 islands, 25 town-years short of food: 18 are the shortage of January to March after a district is redrawn (`tr-carriage.md`, `tr-rent.md`). Of the seven later in the year, three follow a district's harvest 8 to 13 per cent short after a good year (Langside, Marhope, Sunheath), one is a town that gave its farming district to a neighbour at the year's end and kept its townspeople (Ryefell, whose own grain fell from 119,635 to 6,302), two are towns in their second year, which the people rules do not count (Brightheath, Brackwater), and one follows a doubling but comes in July (Rosdale). tr-carriage's four are Langside, Marhope, Sunheath and Ryefell; tr-rent's three are Ryefell, Langside and Brightheath. In every one the store stood at three to seven months of need early in the year, the price fell, little grain was carried in, the store drained for three or four months while the price climbed at most 12 per cent a month, and the month after it ended under a month's need households got only what was on the shelf. Switch off the district weather and the harvest kind goes (none in 11,136 town-years of a poor island-wide harvest); halve the merchants' stock markup and the later shortages fall from seven to three; double it and they rise to 45.

## 1. Kept town ground and the wage (a)

### The question

`docs/rules/towns.md` found the wage and output per head 1 to 3 per cent lower with the town ground kept, and `tr-works.md` found 2 to 5 per cent over some twenty-year span in three settings of four, on sixteen islands, and showed that neither of the two small switches (`towns.donor_wage`, `towns.rent_floor`) nor the room a works leaves for houses (`towns.works_room`) was the cause of the fewer works. Four suspects were left: the farmland the town cells take whole, rent on built ground, works kept out of full cells (a town cell costs a works what the whole cell would pay under houses), and the people the ground lets a town hold.

### What was measured, and how

Kept ground off against on, twenty-year means of the observer's series as an index over each island's 1750s, as `tr-works.md` reads them, first on 32 islands and then on 64, since 16 proved too few (the island-to-island scatter of the difference over the century is about 5 per cent, so a standard error of about 0.9 points on 32 islands):

```bash
isostate-core sweep --seeds 1-16 --rules game --families 6 --set towns.enabled=false,true --out a-f6 --quiet   # and 17-32, 33-64
isostate-core sweep --seeds 1-16 --rules game --families 0 --set towns.enabled=false,true --out a-f0 --quiet   # and 17-32, 33-64
```

Then one suspect at a time, each with kept ground on:

| suspect | how it was taken away |
|---|---|
| the farmland under town cells | a trial (below): a town cell keeps its unbuilt ground under its old uses |
| works kept out of full cells | a trial: a works pays for town ground what the residents pay, as under the port, not what the whole cell would pay under houses |
| the people the ground lets a town hold | `--set towns.smooth=0 --set towns.mean_hands=false`: the town's people counted as the port counts them |
| rent on built ground, and the new town's first year | `--set towns.rent_floor=false --set towns.donor_wage=false` |
| all of them at once | both trials and all four settings, with `towns.works_room=1.0` |

The trials were two switches read from the environment, built into a copy of the program and taken out of the source again (the diff is `trial-a.diff` in the reports folder). The farmland trial is `towns.md`'s own mend in rough: in `land::update_land_use` a town cell is no longer taken whole; its built share is its residents over the port's density for the year (`urban_density0 + urban_density_growth` years, 500 in 1750 and 2,600 in 1900, so that the island's built ground is about what the port's towns take), and the rest of the cell is shared among the uses of the land as open land is. The floor on the cell's rent was not changed to charge the built share only.

`kept_ground.rs` reads, for one setting at a time, what each suspect does to the economy: the farmland and all open land in cells' worth of the land rule's shares, the cells of town ground and the people on them, the house rent over the wage bill, the land's rents over income, the grain made and its price, the hands in works and the works standing.

```bash
cargo run --release -p isostate-core --example kept_ground -- --seeds 1-16 --set '{"towns":{"enabled":false}}'
cargo run --release -p isostate-core --example kept_ground -- --seeds 1-16
```

### What it shows

Sixteen islands, six families, the means over 1800 to 1900 (`kept_ground`):

| | ground not kept | kept (today) | kept, with the farmland trial |
|---|---|---|---|
| farmland, cells' worth | 1,580.5 | 1,553.9 | 1,578.0 |
| all open land, cells' worth | 4,970 | 4,896 | 4,982 |
| cells of town ground | 89 | 203 | 200 |
| people on town ground, thousands | 162.0 | 157.7 | 156.6 |
| house rent over the wage bill | 0.073 | 0.064 | 0.064 |
| rents of farms, woods and pits over income | 0.113 | 0.115 | 0.116 |
| grain made a head a year | 29.1 | 28.8 | 28.8 |
| hands in works, of all people | 0.203 | 0.200 | 0.198 |
| works standing | 530 | 509 | 489 |

Kept ground takes 27 cells' worth of farmland and 75 of all open land (1.5 per cent), and the trial gives all of it back. It does not raise the house rent: kept ground's towns spread their people over more cells further from the centre, where the port's rule charges less a head, so households pay 6.4 per cent of the wage bill in house rent where they paid 7.3.

The real wage and output per head, kept ground against ground not kept, and each remedy against ground not kept (the change in the mean, and on how many islands it is lower):

| six families, 64 islands | 1840-59 | 1860-79 | 1880-1900 | 1800-1900 | output per head, 1800-1900 |
|---|---|---|---|---|---|
| kept (today) | **-4.1%, 44** | -0.9%, 36 | -0.6%, 42 | -1.4%, 38 | -2.1%, 46 |
| kept, farmland trial | -2.8%, 36 | -1.4%, 39 | -0.1%, 34 | -1.3%, 41 | -1.7%, 43 |
| kept, both small switches off | -2.4%, 39 | -1.1%, 38 | -0.6%, 37 | -1.1%, 42 | -1.4%, 45 |

| no families, 64 islands | 1840-59 | 1860-79 | 1880-1900 | 1800-1900 | output per head, 1800-1900 |
|---|---|---|---|---|---|
| kept (today) | +1.7%, 31 | -0.9%, 34 | -0.9%, 38 | -0.1%, 34 | -1.0%, 39 |
| kept, farmland trial | +1.8%, 27 | -0.9%, 27 | 0.0%, 26 | +0.4%, 27 | -0.3%, 29 |
| kept, both small switches off | -0.5%, 36 | -1.5%, 40 | -0.4%, 37 | -0.9%, 35 | -1.0%, 39 |

And on 32 islands, the other suspects (the mean change of each island's real wage over 1800 to 1900, raw and not indexed, against ground not kept; the scatter makes a point either way noise):

| 32 islands | six families | no families |
|---|---|---|
| kept (today) | -2.7% | -1.6% |
| works pay for town ground as under the port | -2.0% | |
| the town's people counted as under the port | -1.9% | |
| the farmland trial | -2.0% | +0.5% |
| both small switches off | -1.1% | -1.5% |
| everything at once | +0.5% | -1.8% |

What the 1840s are made of, six families, 64 islands, kept against not kept, and what each remedy leaves:

| 1840-59 | kept | farmland trial | both switches off |
|---|---|---|---|
| money wage | -1.9% | -2.0% | -1.8% |
| cost of living (the 1750 basket) | +3.3% | +1.5% | +1.4% |
| price of grain (mean; islands dearer) | +20%, 43 | +13%, 37 | +9%, 37 |
| grain made | -4.5% | -2.5% | -1.7% |
| works | -5.2% | -5.7% | -3.7% |
| share of people in towns | -5.8% (lower on 54) | -5.0% | -4.1% |
| the land's share of income | -2.5% | -2.4% | -2.9% |

(This table is of raw span means and the one above of indices over each island's 1750s, as the scripts were run; the real wage's 4.1 above is 4.7 raw.) The parts in this table do not add: 1.9 and 3.3 make 5.2, not 4.1, because a mean of ratios is not the ratio of means when the islands scatter as widely as these do (the cost of living's ratio has a spread of 15 per cent from island to island). In mean logarithms, which do add, the 1840s' real wage is 4.8 lower, the money wage 2.6 and the cost of living 2.4 higher: half and half.

**Added in the check.** The same 64 islands read again with the standard error of each mean (the island-to-island spread over the square root of 64), with the chained cost of living beside the 1750 basket's, and with each remedy set directly against kept ground on the same island (the indices over each island's 1750s, by `decomp.py` in the check's notes):

| six families, 64 islands, 1840-59 | kept against not kept | farmland trial against kept | switches off against kept |
|---|---|---|---|
| real wage, 1750 basket | -4.1% (1.4) | +2.2% (1.6) | +2.8% (1.6) |
| real wage, chained cost of living | -1.6% (1.5) | +0.1% (1.5) | +0.6% (1.5) |
| money wage | -1.9% (1.4) | +0.6% (1.3) | +1.0% (1.4) |
| cost of living, 1750 basket | +3.5% (1.9) | -0.2% (1.9) | -0.4% (1.9) |
| output per head | -4.3% (1.2) | +2.1% (1.4) | +2.6% (1.4) |
| grain's price | +20% (6) | -0.1% (4) | -2% (3) |
| grain made | -4.0% (1.6), less on 40 | +3.0% (1.6), more on 47 | +3.7% (1.7) |
| works | -5.2% (1.3) | 0.0% (1.4) | +2.2% (1.4) |

Over the century (1800 to 1900) kept ground against not kept: the real wage -1.4 (0.7) on the 1750 basket and +0.1 (0.9) chained, output per head -2.1 (0.6), works -5.7 (0.9), fewer on 50 of 64; the farmland trial leaves the works at -5.8. With no families: real wage -0.1 (0.8), output per head -1.0 (0.7), works -7.0 (1.0), fewer on 52 of 64; with the two switches off, -3.9 (0.9).

### The answer

**Where the cost is.** On 64 islands it is a cost of the 1840s and 1850s with families: the real wage 4.1 per cent and output per head 4.3 per cent lower from 1840 to 1859, on 44 and 46 islands of 64, and 1.4 and 2.1 per cent over the century. Without families it is not a cost to the wage at all over the century (0.1 per cent, lower on 34 islands of 64) and a 1 per cent cost to output. The "2 to 4 per cent from 1860 to 1900 with no families" of `towns.md` and `tr-works.md` was the noise of sixteen islands: on 64 the two spans are 0.9 per cent down each.

**What it is made of.** In 1840 to 1859, in mean logarithms, about half of the cost to the 1750 basket's real wage is a dearer cost of living (2.4 points), most of it grain (20 per cent dearer in the mean, dearer on 43 islands of 64, with 4.5 per cent less of it grown), and half a lower money wage (2.6), though neither half alone is two standard errors from nothing. With them go 5 per cent fewer works and a share of people in towns 6 per cent lower on 54 islands of 64: fewer hands in works, and the town's people are counted from its hands in works. The chained cost of living barely moves (+0.9 per cent, 1.8), so on it the real wage is 1.6 per cent lower and within noise: the 1750 basket gives grain the weight it had in 1750's budget.

**Which suspect carries it.** Not settled; set directly against kept ground on the same islands, no remedy moves the real wage by two standard errors.

- *The farmland under town cells.* The trial gives back every cell's worth of open land and more grain grown (3.0 per cent more than kept ground in the 1840s, on 47 islands of 64, most of what kept ground took), and output per head 2.1 per cent (1.4). It does not make grain cheaper than kept ground does (-0.1 per cent, 4) or the cost of living lower (-0.2, 1.9). Its 2.2 points on the real wage are 1.3 standard errors. Set against "ground not kept" the trial seems to halve the dearer grain and living (20 to 13 per cent, 3.3 to 1.5); that is the noise of two means each with its own error of 4 to 6 points on grain, and it does not survive the direct comparison. Without families the trial takes away the little cost there is to output (-1.0 to -0.3 per cent over the century).
- *Rent on built ground* is not a cost to households: they pay less house rent with kept ground, not more. The floor under it (`rent_floor`) and the new town's year at its donor's wage (`donor_wage`), switched off together, move the real wage by about as much as the farmland trial (2.8 points, 1.7 standard errors) and grain's price by as little. With no families they halve the shortfall of works over the century (-7.0 to -3.9 per cent), which bears on `tr-works.md`'s finding that they are not its cause; with families they move it by 1.7 points, within noise.
- *Works kept out of full cells.* Pricing town ground to a works as the port does moves the wage within the noise (-2.0 against -2.7 per cent on 32 islands) and leaves the works as few.
- *The people the ground lets a town hold.* Counting a town's people as the port does moves the wage within the noise (-1.9) and the works not at all.
- None of them, nor any setting tried, gives back the fewer works with families. With everything at once (32 islands) the cost goes with six families (+0.5 per cent) and stays without (-1.8), within the noise of 32 islands either way.

**How sure, and what would change my mind.** Sure that kept ground costs output per head in the 1840s and 1850s with families (4.3 per cent, 3.6 standard errors) and over the century (2.1, 3.6), and that it leaves 5 to 7 per cent fewer works in every span with and without families. Sure that the real wage's cost is mostly on the 1750 basket: chained, it is within noise. Sure that the farmland trial gives back the land and part of the grain grown. Not sure that it, or anything tried, gives back any of the wage: every remedy's effect on it is one to two standard errors on 64 islands. What would change my mind: the farmland mend built properly (with the floor charged on the built share only) and read on 128 islands, against kept ground directly, giving back more than three points of the 1840s' real wage or any of the works.

**For the pass over all the rules.** R102 can be narrowed: kept ground costs the island 2 per cent of output per head over the century and 4 in the decades the towns grow fastest, and 5 to 7 per cent of its works; what it costs the real wage depends on the price index, and on the chained one it is nothing. The fewer works are the effect to trace next; none of the four suspects removes them. `towns.md`'s mend (let a town cell keep the share of its ground its houses do not need under its old uses, and charge the floor on the built share only) would give back all of the open land and most of the grain kept ground takes in the 1840s; the trial does not show that it would give back any of the wage or the works, and it should be built for the land's own sake (the map, and a town that does not sterilise its edge), not as the cure of R102. The density at which a cell counts as built is the mend's one new number; the port's 500 rising 14 a year was the trial's. Anyone who reads kept ground's cost again should read it on 64 islands or more, with families, set the remedies against kept ground directly, and quote standard errors: on 16 its timing is noise, and on 64 a remedy's effect on the wage is still within two of them.

## 2. Crucible steel found late (b)

### The question

`tr-steel.md` showed that a crucible steel found after about 1778 stalls: the first works sell next to nothing, the trade never learns, and the converter comes too late for 1880. It ruled out the wage at the find and named cheap iron by the 1780s as the likeliest reading, without tracing it. Why does a late find stall: cheap iron, a market already filled, the rule for a new works' size, or something else?

### What was measured, and how

`crucible_late.rs` steps each island a year at a time with the books kept. For each island it prints the year crucible steel and the coke furnace were found, the prices of iron, tools and steel and the wage at the find and ten years on, what a unit of implements cost from steel against the cheapest good, the crucible steel made a year in 1800 to 1830 (under 10,000 is a stall), and the first crucible works' first ten years. For the islands asked for with `--detail` it prints, year by year, the trade's works, capacity, output and skill; the island's prices; a unit of implements from iron, tools and steel in the finding town; what households and landlords bought of steel, tools and iron; the first works' own books (capacity, output, use, what it got a ton, its wages and materials a ton, its profit); and the sums entry makes for a new crucible works in the finding town at the size entry would give it (`firms::evaluate_sites`: the size, the small works' penalty, the price it would get, its hands, materials and fuel a ton, its margin and return).

```bash
cargo run --release -p isostate-core --example crucible_late -- --seeds 1-16 --detail 3,9,7 --years 30
cargo run --release -p isostate-core --example crucible_late -- --seeds 1-16 --disc crucible_steel.coal=x30 --disc crucible_steel.iron=x36 --detail 4,12,6 --years 25
cargo run --release -p isostate-core --example crucible_late -- --seeds 1-16 --disc crucible_steel.coal=x30 --disc crucible_steel.iron=x36 --disc coke_furnace.coal=x70 --disc coke_furnace.iron=x60
cargo run --release -p isostate-core --example crucible_late -- --seeds 1-16 --disc crucible_steel.coal=x30 --disc crucible_steel.iron=x36 --method smithy.labor=2.0
cargo run --release -p isostate-core --example crucible_late -- --seeds 1-16 --disc coke_furnace.coal=x12 --disc coke_furnace.iron=x10 --detail 7
```

The second line is `tr-steel.md`'s check: crucible steel's two thresholds doubled, so that the draw finds it late on many islands. The third keeps iron dear on those same islands by doubling the coke furnace's thresholds. The fourth keeps tools dear without touching iron, by giving the smithy twice its hands a ton. The fifth makes iron cheap early, with crucible steel's thresholds as they are.

### What the books say

What steel is for. Before the rolling mills, nothing in the content uses steel as a material: households and landlords buy it as implements, where a ton of steel does three units of work, a ton of tools one and a ton of iron 0.35 (`data/content.toml`, `[services.props]`). They buy the cheapest per unit of work, all but a sliver, by a logit with a narrow spread (`logit_theta` 0.25: a good dearer by half per unit than the cheapest gets about one part in seven of what the cheapest gets). Tools are made from iron by the smithy, 0.55 of iron and a hand a ton.

An island that grows (seed 7, found 1765, no coke furnace yet) and one that stalls (seed 3, found 1784, the coke furnace in 1777). Each year: the island's crucible works, their output and skill; the price of tools and steel; a unit of implements from tools and from steel in the finding town; and what a new crucible works there would see, a ton:

| seed 7 | works | made | skill | tools | steel | a unit of work: tools, steel | new works: price, hands, materials, margin |
|---|---|---|---|---|---|---|---|
| 1766 | 1 | 6 | 0.52 | 19.4 | 740 | 19.8, 225 | 634, 120, 59, +455 |
| 1768 | 7 | 21 | 0.58 | 17.6 | 134 | 18.2, 43 | 121, 102, 45, -26 |
| 1770 | 7 | 8 | 0.61 | 15.4 | 129 | 15.6, 39 | 110, 87, 33, -10 |
| 1772 | 2 | 4 | 0.62 | 17.1 | 154 | 17.4, 43 | 122, 83, 37, +2 |
| 1774 | 5 | 36 | 0.72 | 16.7 | 121 | 17.0, 38 | 106, 72, 34, 0 |
| 1776 | 13 | 142 | 0.91 | 18.3 | 108 | 18.8, 36 | 102, 56, 35, +12 |
| 1780 | 21 | 586 | 1.52 | 14.2 | 71 | 14.3, 24 | 68, 34, 19, +15 |
| 1790 | 39 | 4,382 | 3.19 | 9.4 | 36 | 9.4, 12 | 33, 13, 6, +15 |

| seed 3 | works | made | skill | tools | steel | a unit of work: tools, steel | new works: price, hands, materials, margin |
|---|---|---|---|---|---|---|---|
| 1785 | 1 | 5 | 0.51 | 14.2 | 423 | 14.2, 136 | 384, 143, 36, +205 |
| 1787 | 2 | 1 | 0.55 | 12.6 | 117 | 12.5, 38 | 108, 115, 27, -34 |
| 1790 | 1 | 0 | 0.55 | 10.0 | 127 | 9.8, 38 | 108, 108, 16, -16 |
| 1795 | 1 | 0 | 0.56 | 8.3 | 116 | 7.8, 32 | 91, 93, 11, -12 |
| 1801 | 11 | 28 | 0.64 | 8.3 | 80 | 7.8, 26 | 74, 85, 10, -22 |
| 1806 | 9 | 22 | 0.70 | 8.5 | 77 | 7.9, 27 | 77, 85, 11, -19 |
| 1811 | 10 | 24 | 0.76 | 8.7 | 75 | 8.2, 26 | 73, 77, 11, -15 |
| 1814 | 11 | 8 | 0.80 | 8.8 | 82 | 8.3, 29 | 81, 74, 11, -4 |

On both islands the first works is built in the year of the find, at the least size a works may have (1.6 a month, with the small works' penalty of 1.58 on its hands), sells its first few tons at three to five hundred while there is no stock and the price stands near its ceiling, and is idle from its third year and closed in its fifth (seed 3's in 1789, seed 7's in 1770). That is the same early and late, and so is every new works' size: entry sizes a works at a share of the town's trade in steel, which is next to nothing, so a new crucible works is built at the floor for the first twenty years on both islands. What differs is what the trade does after the first works.

Steel can fetch a price at which a unit of its work costs about two and a half times a unit of tools' (seed 7 in the 1770s: 36 to 43 against 15 to 19), at which households and landlords buy a sliver of it: tens of tons a year where they buy twenty thousand tons of tools. On seed 7 tools cost 15 to 18 for the first fifteen years, so steel fetched 110 to 150, and a crucible works at its first skill came within a little of its costs: its hands cost 72 to 102 a ton and its iron and clay 33 to 45. The few works there were went on making a few tons a year at a small loss, and each ton made taught the trade: its skill rose from 0.58 in 1768 to 0.72 in 1774 and 0.91 in 1776, and from 1776 a new works paid. By 1790 its skill was 3.2 and steel half the price of tools a unit of work.

On seed 3 the coke furnace had come seven years before the find, iron fell from 15.5 to 5.2 in ten years and tools from 15.2 to 8.4. Steel could then fetch 75 to 95 at most, and at a skill of 0.55 to 0.80 its hands alone cost 74 to 115 a ton. In 25 of the 29 years from 1786 to 1814 a new works would have lost money, up to 46 a ton; in the other four (1791, 1800, 1805, 1810) the trade had all but stopped and the price had drifted up, and the works founded on the strength of it lost again the next year. Works were founded (the year's room is never less than one smallest works) and closed again, the island made 0 to 28 tons a year, and in thirty years the skill crept from 0.55 to 0.80. Cheap iron helps crucible steel's costs a little (its iron and clay fall from 36 to 11 a ton) and helps its rival much more, since tools are mostly iron and steel is mostly hands.

### Tried both ways

Crucible steel made a year in 1800 to 1830; "stall" is under 10,000 (the healthy islands make 40,000 to 67,000):

| setting | islands that stall | of those that find it after 1778 |
|---|---|---|
| the game as it is | 2 of 16 (seeds 3 and 9) | 1 of 1 |
| crucible steel's thresholds doubled | 6 of 16 (3, 4, 5, 9, 12, 14) | 5 of 5 |
| doubled, and the coke furnace's doubled too (iron stays dear) | 1 of 16 (seed 4, found 1800 with the coke furnace in 1784) | 1 of 5 |
| doubled, and the smithy needs twice the hands (tools dear, iron cheap) | 1 of 16 (seed 5, found 1794) | 1 of 3 |
| the game's crucible thresholds, the coke furnace's cut to a third (iron cheap early) | 4 of 16 (3, 10, 13, 15), one more weakened (4) | 2 of 2 |

With iron kept dear, the late finds of seeds 3, 5, 12 and 14 (1780 to 1786) make 15,000 to 27,000 a year by 1800 to 1830, where with cheap iron they made under 10,000. With tools kept dear and iron as cheap as before, seeds 3 and 12, found in 1784 and 1785 with iron at 13.7 and 9.6 and falling to 6.8 and 5.8, make 24,000 a year. With iron made cheap early, seed 13 finds crucible steel in 1774 and seed 3 in 1776, years that are early on today's islands, seven and five years after the coke furnace, and both stall; seed 4 finds it in 1770, six years after the coke furnace, and makes 15,000 a year where today it makes 55,000. So the date is not the cause. The cause is the order: crucible steel that has climbed its curve before the coke furnace cheapens tools survives, and crucible steel found after it does not.

### How sure, and what would change my mind

Sure: the mechanism is in the books year by year on the two islands, and the three trials move the stall in the direction it predicts on every island they touch (iron dear: four of five late finds recover; tools dear with cheap iron: two of three recover; iron cheap early: early finds stall). The two that do not fit are a crucible steel found in 1794 and one found in 1800, so late that even dear tools leave it a decade behind. What would change my mind: an island that finds crucible steel after the coke furnace, with tools under 10 in the decade after, and grows.

### For the pass over all the rules

The trap is a learned method whose only market is held by a cheaper substitute: it starts at half its skill, a little dearer than the substitute can tolerate, makes nothing and so never learns. It is the same trap as the steel rail mill's (`tr-steel.md`, section 1), and the same thing makes it worse there: cheap iron cheapens the rival (iron rails, tools) more than it cheapens the steel. The numbers that decide it are crucible steel's start on its curve (`A0` 0.5, `Q0` 50; `tr-steel.md` found `A0` 0.7 ends the stall everywhere), the steel's worth as an implement against tools' (3.0 against 1.0) and the logit's narrowness (`logit_theta` 0.25), which together set how much steel the first works can sell. A designer who wants crucible steel to survive a late find should give its first works some buyer that is not a household choosing implements by price (cutlery, springs, edge tools for the works themselves, all historical), or let a new method learn from the trade it serves and not only from its own output. The deeper production tree (`docs/designs/`) is the place for the first. A late draw also matters more than its date suggests: an invention found after the one that cheapens its rival can be lost for half a century.

## 3. What railways do to people and merchants (c)

### Fewer people

**The question.** `tr-carriage.md` found that with railways the island's people are 0.6 per cent fewer in 1880 to 1899, on fifteen islands of sixteen, though the real wage is 8 per cent higher. Why?

**Measured.**

```bash
isostate-core sweep --seeds 1-16 --rules game --families 6 --set rail_know=0.25,99 --set people.town_deaths=0.012,0 --out c-deaths --quiet
```

`rail_know` 99 means no line is ever built. `people.town_deaths` is the people rules' extra deaths a year for town life, 0.012 a head in a great town where all live on town ground (`docs/rules/people.md`).

| 1880-1899, sixteen islands | `town_deaths` 0.012 (the game) | `town_deaths` 0 |
|---|---|---|
| people with railways, against none | **-0.6%, fewer on 15** | **+0.6%, more on 15** |
| share of people in towns, with and without | 0.421 and 0.407 | 0.409 and 0.394 |
| real wage, with against without | +8.3% | +8.6% |
| output per head | +8.3% | +8.8% |

**The answer.** Railways put more people in towns: the share in towns is 1.4 points higher with them, on every island of sixteen, in every span from 1840. The people rules charge town life extra deaths that rise with the share of a district's people on town ground and with the town's size. That outweighs what the higher income adds in births: take the town deaths away and the same railways leave 0.6 per cent more people instead of 0.6 per cent fewer, on fifteen islands of sixteen each way, with the real wage's gain unchanged.

**How sure.** Sure: one number turns the sign on fifteen islands of sixteen and leaves the rest as it was. What would change my mind: a run in which railways leave fewer people with town deaths at nothing.

### The merchants' share

**The question.** Under the port with carting (no families) the merchants take 40 per cent of the railway's gain and the works' owners 38; under the game they take nothing. `tr-carriage.md` took the game's mechanisms away one at a time and none brought the port's shape back.

**Measured.** First, where the merchants' income comes from. `merchants_take.rs` splits it, year by year, into the spread they take on everything the land and the works sell them (`merchant_spread`, 6 per cent of its worth), the margin on what they carry between towns (what it fetches where it arrives, at that town's mean price of the year, less its worth where it left and the carting), and the rest (sales out of stock at other prices than the year's mean, goods sold that were not in store, rounding). The parts of the gain are each kind's income a head in 1750 money, its share of income times output per head from the sweep of the same runs.

```bash
cargo run --release -p isostate-core --example merchants_take -- --rules port --families 0 --set carting.enabled=true --set rail_know=0.25 > port-rail.csv   # and rail_know=99
cargo run --release -p isostate-core --example merchants_take -- --rules game --families 6 --set rail_know=0.25 > game-rail.csv                         # and rail_know=99
isostate-core sweep --seeds 1-8 --rules port --families 0 --set carting.enabled=true --set accounts.enabled=true --set rail_know=0.25,99 --out c-port-base --quiet
```

Then each of the game's mechanisms added alone to the port with carting (`--set prices.enabled=true`, and so for `works`, `people`, `railways`, `engel`, `machines`, `mines`, `harvest`, `canals`, and `--families 6`), and two and three taken from the game together (`--set people.enabled=false --set works.enabled=false`, and so on), eight islands each, railways against none.

**What it shows.** 1880 to 1899, eight islands, as shares of income:

| | port, with railways | port, without | game, with | game, without |
|---|---|---|---|---|
| merchants' income | 0.096 | 0.073 | 0.044 | 0.047 |
| their spread | 0.067 | 0.066 | 0.071 | 0.071 |
| their margin on what they carry between towns | **0.076** | **0.057** | 0.045 | 0.048 |
| the rest | -0.047 | -0.050 | -0.072 | -0.072 |

and as parts of the railway's gain: under the port, merchants 41 per cent, of which the spread 7, the margin on carriage 34 and the rest -1; under the game, merchants 1 per cent, of which the spread 7, the margin 0 and the rest -6.

The merchants' part of the gain, with one of the game's mechanisms added to the port, or some taken from the game:

| | merchants | works' owners | work | land and houses |
|---|---|---|---|---|
| the port with carting | 40% | 38% | 8% | 14% |
| and the merchants' prices | 40% | 20% | 19% | 21% |
| and the works' life cycle | 43% | 7% | 27% | 23% |
| and the people rules | 45% | 5% | 30% | 20% |
| and the game's railways | 55% | 18% | 4% | 23% |
| and the Engel curve | 44% | 43% | -7% | 20% |
| and the machines | 37% | 21% | 22% | 20% |
| and the mines | 51% | 18% | 9% | 23% |
| and the harvests | 36% | 21% | 22% | 21% |
| and the canals | 25% | 19% | 19% | 37% |
| and six families | 68% | 20% | -7% | 19% |
| the game less prices and works | -1% | 31% | 45% | 26% |
| the game less people and works (margin 12 of the 19) | 19% | 11% | 32% | 38% |
| the game less prices, people and works | 21% | 23% | 26% | 30% |
| the game with no families | 7% | 1% | 61% | 32% |
| the game | -1% | 0% | 69% | 33% |

**The answer.** The merchants' part is the margin on carriage between towns. Under the port the railway makes carting cheaper and the towns' prices do not close the gap it leaves, so the merchants who carry goods keep the difference: their margin goes from 5.7 to 7.6 per cent of income, 34 of their 41 points of the gain. Under the game the same railway leaves their margin where it was, because the towns' prices close up: the wage and the land get the saving through cheaper goods. The spread is a share of what is made and moves with output under both, 7 points of the gain either way. What closes the gap in the game is not one mechanism: none of ten added to the port brings the merchants' part under a quarter, and it takes the works' life cycle and the people rules both taken from the game to bring it back to a fifth (with the margin 12 points of it). The likeliest reading, not traced further, is that under the port neither new works nor new people come to where cheaper carriage has made a market dearer than its neighbours, so the gap stays for the carrier, while under the game works grow where they pay and people go where the wage is, and both close it.

**How sure.** Sure of where the merchants' gain comes from under each set of rules: the split is arithmetic on the books, and the margin moves by 34 points of the gain under one and none under the other. Sure that no single mechanism makes the difference (ten tried, each on eight islands; the parts move by tens of points between settings, so any one figure is rough). Not sure that works and people together are the whole of it: taking them both away brings the merchants back to 19 per cent, not 40.

### For the pass over all the rules

- `people.town_deaths` is what decides whether the railway's towns cost people. If the game is to show railways growing the island, as they did, the town's extra deaths should fall with what the railway brings to a town (cheaper food, coal, a water supply), or the extra deaths should be smaller; today they are the whole of the difference.
- The merchants' takings are not a sign of anything a player should see, under either set of rules: under the game the margin on carriage stays flat whatever the railway does, so no merchant or carrier has a reason to want a line. That bears on railway companies and fares (C86): whoever designs them should know that under the game a line's value goes to the wage and the land (69 and 33 per cent) and none of it to whoever carries or pays.

## 4. Shortages later in the year (d)

### The question

`tr-rent.md` left three shortages in spring on islands 17 to 24 (Ryefell, Langside, Brightheath) that were not the January shortage after a redraw; `tr-carriage.md` left four hungry town-years on islands 17 to 32 with canals that were not doublings. What are they?

### What was measured, and how

`spring_hunger.rs` plays each island a month at a time, as `trace_rent` does, and for every town-year short of food (under 0.82 of its need in its worst month, the line `people::growth` uses) sorts it into one of five kinds, tried in order: *jump*, the town's people grew by half or more at the year's end and it ran short in January to March; *new*, a town founded at one of the last two year's ends, which `people::growth` does not count; *lost*, the town's people fell by a fifth or more at the year's end, because it gave a district to another town; *harvest*, its district's harvest that year was under 0.95; *other*. For every one but the first it prints the district's harvest that year and the year before, the grain the town made, sent and received and its farm hands, this year against last, and its store, price and wage month by month.

```bash
cargo run --release -p isostate-core --example spring_hunger -- --seeds 1-32
cargo run --release -p isostate-core --example spring_hunger -- --seeds 1-32 --set '{"harvest":{"enabled":false}}'
cargo run --release -p isostate-core --example spring_hunger -- --seeds 1-32 --set '{"harvest":{"district_sd":0.0}}'
cargo run --release -p isostate-core --example spring_hunger -- --seeds 1-32 --set '{"prices":{"markup":0.25}}'
cargo run --release -p isostate-core --example spring_hunger -- --seeds 1-32 --set '{"prices":{"markup":1.0}}'
```

The town-years it finds on seeds 17 to 32 are the sixteen `trace_rent --seeds 17-32 --part d` finds, to the month.

### What they are

Thirty-two islands, the game as it is: 25 town-years short of food. Eighteen are the shortage of January to March after a redraw (`tr-carriage.md`, section d). The seven later in the year:

| town, island, year | kind | worst month, food got | what the town's grain did that year |
|---|---|---|---|
| Langside, 18, 1844 | harvest | May, 0.58 | district harvest 0.87 after 0.96; made 35,422 for 44,531; sent 15,761 away |
| Marhope, 27, 1797 | harvest | June, 0.73 | 0.90 after 1.07; made 49,216 for 62,778 |
| Sunheath, 32, 1831 | harvest | July, 0.75 | 0.92 after 1.06; made 33,245 for 45,269; sent 12,308 away |
| Ryefell, 23, 1832 | lost | May, 0.23 | its people 3,431 to 2,162 at the year's end; farm hands 421 to 30; made 6,302 for 119,635 |
| Brightheath, 20, 1759 | new | June, 0.66 | a year old; its district's stock came with it |
| Brackwater, 29, 1830 | new | June, 0.68 | a year old |
| Rosdale, 30, 1807 | other | July, 0.74 | its people 2.2 times at the year's end; harvest 1.03 after 1.13 |

`tr-carriage.md`'s four with canals that were not doublings are Langside, Marhope, Sunheath and Ryefell: three districts' harvests and one town that gave its farmland away. `tr-rent.md`'s three spring shortages on islands 17 to 24 are Ryefell, Langside and Brightheath.

They share one shape. Langside, 1844, at each month's end:

| | Jan | Feb | Mar | Apr | May | Jun | Jul |
|---|---|---|---|---|---|---|---|
| grain in store, months of need | 4.07 | 2.01 | 0.69 | 0.59 | 1.05 | 3.71 | 2.40 |
| price | 0.56 | 0.63 | 0.71 | 0.80 | 0.91 | 0.80 | 0.91 |
| food got, of need | 1.82 | 1.82 | 1.73 | 0.67 | 0.58 | 1.03 | 1.57 |

Early in the year the town has three to seven months of grain in store, often after a good year, and its merchants, who mark their price by their stock, ask little for it: the price is at its lowest in January to March. At that price little grain is worth carrying in, and some of the town's own goes out (Langside sent 15,761 away in the year it ran short, Sunheath 12,308). Then the store drains at one to two and a half months of need a month, the town's own crop being short or gone, while the price climbs by at most 12 per cent a month (`prices::MAX_STEP`). A month's sales can only be from what was in store when the month began (`econ::step_month` fills each buyer's request from the store, and what is made and carried in that month goes on the shelf after the buying), so the month after the store ends under a month's need, households get what is on the shelf: Ryefell's store was 0.23 months at the end of April and its households got 0.23 of their need in May. By then the price has risen past the cost of carrying grain in, and within a month or two the store is three to fourteen months of need.

### Tried

| 32 islands | short of food | jump | new | lost | harvest | other | later than March |
|---|---|---|---|---|---|---|---|
| the game as it is | 25 | 18 | 2 | 1 | 3 | 1 | 7 |
| no harvests | 16 | 14 | 0 | 1 | 0 | 1 | 2 |
| the island's weather but none of the districts' | 17 | 14 | 1 | 0 | 0 | 2 | 3 |
| the stock markup halved (`prices.markup` 0.25) | 26 | 21 | 1 | 0 | 2 | 2 | 3 |
| the stock markup doubled (1.0) | 78 | 11 | 16 | 1 | 19 | 31 | 45 |

Without the district's own weather the harvest kind goes: no town of 11,136 town-years with a poor island-wide harvest went short. The likeliest reading, not traced further: a poor harvest everywhere raises every town's price together, while a poor harvest in one district leaves its town to drain its store at a low price beside neighbours who have no reason to send it grain. The lost kind stays without harvests (Glendale, island 30, 1835: its people 8,510 to 4,424 at the year's end and its grain 316,014 to 50,674). And the merchants' markup scales the whole later kind: halved, seven become three; doubled, 45, as `docs/rules/prices.md` warned (from 1 up the markup over-corrects).

### How sure, and what would change my mind

Sure of the shape: all seven later shortages on 32 islands show it month by month, and the markup moves their number in the direction the shape predicts, by a factor of six one way. Sure that the harvest kind is the districts' weather (three of three with it, none without), though three is a small count. The lost and new kinds are one and two town-years in 32 islands, read from their months and not tried apart. What would change my mind: a later shortage in a town whose store did not fall from several months to under one over the spring.

### For the pass over all the rules

These are not famines a player should see as such: they are a month or two in a town of a few thousand, on one island in five, and every one is over within two months. But they are the visible end of a stock cycle in the towns that import their food. Two rules make it: the merchants mark down a full store so far that carriage stops for a season (`prices.markup` 0.5, with the price's step of 12 per cent a month), and a month's sales can come only from the store as the month began, so a town that runs its store under a month's need goes short that month whatever is made or on its way. A designer who wants grain stores to be what carry a town through a poor year should look at the second: letting a month's buyers draw on that month's own crop and arrivals as well as on the store is the change to try (it was not tried here, and it is the port's rule, so it would need a switch). Two smaller things: a town that gives its farmland to another keeps its merchants' record of a trade that was mostly its own grain (the same condition in `peopling::move_with_cells` as the January kind, the other way round), and the people rules' two years of grace for a new town means two of the seven do not count as hunger at all.

## 5. What it means for the pass over all the rules

- **Kept ground (R102).** Narrow it: 2 per cent of output per head over the century (4 in the 1840s and 1850s, with families) and 5 to 7 per cent of the works in every span; its cost to the real wage is on the 1750 basket only, and no suspect tried gives any of it back by two standard errors. `towns.md`'s mend would give back the land and most of the grain kept ground takes, not the works; build it for the land's sake. The fewer works are what to trace next. Read it on 64 islands or more, remedies against kept ground directly.
- **Learned methods with one market.** Crucible steel and the steel rail mill fall into the same trap for the same reason: a cheaper rival made of iron. `A0`, the steel's worth as an implement and the logit's narrowness set the trap; a buyer that is not a household choosing implements by price, or learning from the trade served, would spring it.
- **`people.town_deaths` decides whether railways cost people.** Today it is the whole of the difference.
- **The merchants' margin on carriage** is where the port's railway gain went; the game competes it away through its works and its people together. Nobody under the game gains from a line but the wage and the land.
- **The merchants' markup and buying from the store alone** make a seasonal stock cycle in importing towns whose bottom is a month's shortage. `prices.markup` is the stronger lever (a factor of six in later shortages between 0.25 and 1.0).

## 6. How to run it again

From the repository's root, after `cargo build --release -p isostate-core --examples` and `cargo build --release -p isostate-core`. `DIR` is a folder of one's own. A sweep of 32 islands takes five to ten minutes on a busy machine, each example on 16 or 32 islands three to eight.

```bash
# (a)
./target/release/isostate-core sweep --seeds 1-64 --rules game --families 6 --set towns.enabled=false,true --out DIR/a-f6 --quiet
./target/release/isostate-core sweep --seeds 1-64 --rules game --families 0 --set towns.enabled=false,true --out DIR/a-f0 --quiet
./target/release/isostate-core sweep --seeds 1-64 --rules game --families 6 --set towns.rent_floor=false --set towns.donor_wage=false --out DIR/a-rent-f6 --quiet
./target/release/isostate-core sweep --seeds 1-32 --rules game --families 6 --set towns.smooth=0 --set towns.mean_hands=false --out DIR/a-count-f6 --quiet
./target/release/examples/kept_ground --seeds 1-16 --set '{"towns":{"enabled":false}}'
./target/release/examples/kept_ground --seeds 1-16
# (b)
./target/release/examples/crucible_late --seeds 1-16 --detail 3,9,7 --years 30
./target/release/examples/crucible_late --seeds 1-16 --disc crucible_steel.coal=x30 --disc crucible_steel.iron=x36
./target/release/examples/crucible_late --seeds 1-16 --disc crucible_steel.coal=x30 --disc crucible_steel.iron=x36 --disc coke_furnace.coal=x70 --disc coke_furnace.iron=x60
./target/release/examples/crucible_late --seeds 1-16 --disc crucible_steel.coal=x30 --disc crucible_steel.iron=x36 --method smithy.labor=2.0
./target/release/examples/crucible_late --seeds 1-16 --disc coke_furnace.coal=x12 --disc coke_furnace.iron=x10
# (c)
./target/release/isostate-core sweep --seeds 1-16 --rules game --families 6 --set rail_know=0.25,99 --set people.town_deaths=0.012,0 --out DIR/c-deaths --quiet
./target/release/isostate-core sweep --seeds 1-8 --rules port --families 0 --set carting.enabled=true --set accounts.enabled=true --set rail_know=0.25,99 --out DIR/c-port --quiet
./target/release/isostate-core sweep --seeds 1-8 --rules port --families 0 --set carting.enabled=true --set accounts.enabled=true --set works.enabled=true --set rail_know=0.25,99 --out DIR/c-port-works --quiet   # and each other mechanism
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --set people.enabled=false --set works.enabled=false --set rail_know=0.25,99 --out DIR/c-game-less-people-works --quiet
./target/release/examples/merchants_take --rules port --families 0 --set carting.enabled=true --set rail_know=0.25 > DIR/port-rail.csv
./target/release/examples/merchants_take --rules port --families 0 --set carting.enabled=true --set rail_know=99 > DIR/port-none.csv
./target/release/examples/merchants_take --rules game --families 6 --set rail_know=0.25 > DIR/game-rail.csv
./target/release/examples/merchants_take --rules game --families 6 --set rail_know=99 > DIR/game-none.csv
# (d)
./target/release/examples/spring_hunger --seeds 1-32
./target/release/examples/spring_hunger --seeds 1-32 --set '{"harvest":{"district_sd":0.0}}'
./target/release/examples/spring_hunger --seeds 1-32 --set '{"prices":{"markup":0.25}}'
```

The span means and the parts of the gain were read from the sweeps' `annual.csv` (each island's span mean, as an index over its own 1750s where the page says so; a kind's income a head is its `acct_share_*` times `gdp_pc`) by short scripts kept with the notes. The two trials of section 1 need their diff, `trial-a.diff` in the reports folder, applied and the program built; the environment's `ISOSTATE_TRY_TOWN_FARM` and `ISOSTATE_TRY_PORT_GROUND` switch them on. They are not on the branch.

## 7. Not done

- **(a)** The farmland mend was a trial: it did not charge the floor on the built share only, and it was not built behind a switch with its own page. Why kept ground has 5 per cent fewer works and a lower money wage in the 1840s was not traced; no remedy tried touches it. Why the two small switches act through the price of grain was reasoned, not traced.
- **(b)** No mend was tried; `tr-steel.md`'s `A0` 0.7 is the one known to work. Whether other learned methods (the engine works, the textile mill) can fall into the same trap was not looked at.
- **(c)** What in the works' life cycle and the people rules together closes the merchants' margin was not traced; taking both away brings back half of the port's figure, not all. The extra deaths with railways were reasoned from the rule and switched, not counted town by town.
- **(d)** The new and lost kinds are three town-years in 32 islands and were read, not tried apart. Buying from the month's own arrivals was proposed, not tried.
- The viewer was neither built nor shot: nothing it uses has changed.

## 8. What was checked

- `cargo test -p isostate-core --release --no-fail-fast`: 214 tests in 23 groups pass, the golden test among them. A first run lost `live_api_reports_every_year_once`, a test of how long a live year takes (109 ms), on a machine running a dozen other helpers' sweeps; it passed when run again.
- `isostate-core sweep --seeds 1-8 --rules game --families 6 --compare .claude/night/reports/merged-game-f6-l`: "of the 446 facts both have, the seconds aside, 0 differ". The branch adds four examples and this page and nothing else.
- The machine was shared throughout, so no timing on this page means more than "a few minutes".
