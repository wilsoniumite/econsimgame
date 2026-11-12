# Canals

A canal age before the railways. Not in the Python. The switch is
`canals.enabled`; the code is `canals.rs` (the numbers, the canal as one kind
of way, the know-how, what the boats carried, the facts), the promoters of
`ways.rs` in a canal mode, and a second layer in the cost graph
(`transport.rs`). It builds on the railways' planner
([railways.md](railways.md), "What a canal age needs").

## What it does

A canal is dug where the carting it would save pays for it: between two towns
the sea does not join, or from a district that fills many carts to its town
or to a navigable river. Nothing says when. Boats then carry what goes far,
mostly coal, grain and stone, until a railway runs the same way at half the
cost, and after that the boats carry little. Nothing closes a canal.

**The way itself is a layer of water over the cells.** A cell with a canal is
two places, the ground and the water, and a load is on one or the other.
Getting on to a boat costs `wharf` (0.4 in the cost graph's units) and getting
off costs it again; along the water a cell costs `cost` (0.13), against 0.3
for a navigable river or a turnpike on lowland, 0.45 for a made road and 0.07
for a railway. So a load that takes the canal pays 0.8 for its two wharves
and then well under half the road's rate, which is worth it from about five
cells of turnpike or three of made road. A cart that only crosses a canal
pays nothing for it, and a hop of a cell or two stays on the road. The mean
haul by boat comes out at 44 km.

**A canal is joined link by link.** The water runs from cell to cell only
along the lines that were dug. Two canals in neighbouring cells are not one
canal unless somebody cut a junction, which is a project like any other, with
no new cell and a cell's worth of materials. The links are kept and are what
the record gives the viewer, so it has no junctions to guess at.

Four to six junctions are cut in a run (one to eleven). The news tells one
as a junction ("A junction joins the canals between Sandmere and
Brambridge"), at the weight of a year's branches and with no kilometres, and
the facts count it under `canal_junctions_cut`, not among the canals. Not
every such cut is between two canals: where a later canal's route runs along
an earlier one and takes one of its corners short, the new link closes a
triangle of three cells. There are 2 to 11 such triangles in the seven runs
I counted, about one link in a hundred, and the viewer may want to draw one
as a bend made straighter.

**It keeps to low ground.** The route search finds a river's valley cheap
(a river cell costs 0.7 of a lowland cell), hills twice the lowland, and
every unit of height `lock_cells` (80) cells' worth: the map's heights run
from 0 to 1 and the lowland ends at 0.24, so climbing out of the lowland
costs as much as nineteen cells of level cut. Mountains are priced like the
sea. No canal crosses them, and a pair of towns with only mountains between
them is left alone for ten years at a time.

**Locks are paid for, in stone, bricks and timber.** A cell of canal (2 km)
takes 150 stone, 80 bricks, 60 timber and 200 clay to line it. Each unit of
height on the line takes eighty cells' worth of the first three on top. On
seeds 3 and 7 the locks are half of all the stone, bricks and timber.

**It is mostly labour, and the labour is somebody's.** For every shilling of
materials delivered a canal pays two in wages (a railway pays three tenths),
and the navvies are hands: last month's wage bill of each canal, at the wage
of the town that supplies it, is that many people who are not there to be
hired by farms and works this month, up to a fifth of the town's hands. They
are counted as employed. They are taken on and let go by degrees, a quarter
of the way each month (`navvy_adjust`), as the carters are: a canal's orders
are filled in lumps, its wage bill halves or doubles from one month to the
next in one town-month in twenty, and when the hands followed the bill at
once a town's wage rose and fell by half with it (see the review, below).
In the busiest year of a run the navvies are 1,100 to 2,300 people on an
island of 200,000, and in the town that supplies a large canal as many as a
sixth of its hands. Railway navvies are still paid and cost nobody's time,
as in the port: that is the railways' to change.

**It takes years.** Thirty-six months if every order is filled, and a canal
orders no more each month than its cash will pay for, wages included. From
promotion to opening the slowest of a run takes five or six years. Every
canal has the same thirty-six months whatever its length, a junction of one
cell too: that is the planner's rule for every line of every way, and it is
wrong for the short ones.

**The promoters are the railways'.** `ways::plan` and `ways::progress` run a
second time each year and month with the canal as the `Way`. Every canal is
valued by the same sum as every railway: the cost graph is copied, the canal
is laid on the copy (as links of water), every cell's cost to market and the
served towns' cost to every other town are measured again, and the fall
times the traffic times `freight` is the saving. Canal promoters reckon on
today's traffic alone (`induced` is 1, where the railway's is 2.5) and want
ten per cent on it (`hurdle`). Three things differ from the railway's
promoters, and each is a field of `Way`:

- A branch may start from any town or any cell of navigable river, as well
  as from a canal. So there is no "town's first line": towns are joined by
  trunks, and branches run to districts three cells or more from any town,
  canal or navigable river.
- The districts looked at are the six dearest among those whose own market
  town knows how to dig, not the six dearest of the island.
- They see the railways that are being laid (the lines under way are put on
  their copy of the graph), so no canal is promoted on carting that a line
  already building will take. The railway's promoters see the canals that
  are being dug in the same way (`Way::rivals`, both ways since the review).

Money is the railways' too: the landlords of the two towns put up to 35 per
cent of their cash and the households 8 per cent, half the expected cost
must be raised, a canal short of cash calls on them, and what is left when
it opens goes back to them. Every payment is a transfer between purses
`econ::money_total` counts.

## What lets canals begin

Know-how that has to be found out, and then seen working.

I tried need alone first. The pound lock was old in 1750 and nothing about a
canal had to be invented, so the first version let any town's promoters dig
as soon as the sum paid. It paid at once: eight canals were promoted in
1750, 670 km were open by 1760, and the island of 1750 turned out to have
been sitting on a fortune it had never noticed. The warm-up settles an
economy with no canals in it, and if nothing but need holds them back then
that economy is not one that could have existed. Something has to arrive.

So somebody surveys a canal, once, by the rule every invention in the
content is found by: the chance in a year is `discovery_rate * p^4 /
(1 + p^4)`, where p is how much coal and stone the island has dug since
1750 against 3 million units of each, and the town is drawn by what it digs
of them. An island that moves more coal gets there sooner. That town knows
how (0.25, as with any invention) and the know-how spreads between towns
along the trade routes by `tech::diffusion`'s sum.

One thing is the canal's own. A town teaches by its canal: what a town knows
counts in full, for itself and for its neighbours, once a canal it
subscribed to is open, and for a fifth of that (`hearsay`) until then. So the
first canal is the surveying town's, usually a branch to whatever fills its
carts, the neighbours learn from it over the years it takes to dig and to
prove itself, and the canal age crosses the island at the pace canals are
dug: 50 km in the first ten years, then most of the network in the twenty
after. With plain diffusion (tried on seed 7) sixteen canals were promoted
all over the island within five years of the first, and 580 km were open
ten years on.

How much the rule does was measured in the review, on seeds 3 and 7. With
`hearsay` at 1 (plain diffusion) 548 km are open in 1780; at 0.2, 422; at 0,
where a town with no canal teaches nobody, 209, and then 692 in 1790. So the
rule moves the rush five or ten years and does not make it any longer. What
makes it short is that nearly every canal that will ever pay pays already in
1750: once a town knows how, there is nothing to wait for but the digging.

The know-how is kept in the canals' own state (`CanalState::know`), not in
the content's list of inventions, and the survey is drawn from a stream of
its own (stream 9). I would have preferred a line in `game_content()`. But
the sweep measures a mechanism alone under the port's rules, with the port's
content, where such an invention would not exist and the canals would fall
back to need; and an eleventh invention draws from the stream the other ten
are drawn from, so switching canals off would have moved the date of the
railway. As it is the canals are the same canals under both sets of rules,
and with the switch off no other invention moves.

## The numbers

`CanalParams`, in `canals.rs`. Off by default; `game()` switches it on with
these.

| | | |
|---|---|---|
| `enabled` | false | promoters may dig canals |
| `cost` | 0.13 | path cost of a cell of canal |
| `wharf` | 0.4 | getting a load on to a boat, and again off |
| `stone`, `bricks`, `timber`, `clay` | 150, 80, 60, 200 | a cell of canal |
| `lock_cells` | 80 | cells' worth of stone, bricks and timber per unit of height; the route search charges the same |
| `wages` | 2 | wages on every shilling of materials |
| `navvies` | true | the navvies are hands taken from the supplying town |
| `navvy_max_share` | 0.2 | of a town's hands, at most |
| `navvy_adjust` | 0.25 | of the way each month to the navvies the wage bill pays for |
| `build_months` | 36 | if every order is filled |
| `hurdle` | 0.1 | the yearly return a canal must promise |
| `induced` | 1 | on today's traffic and no more |
| `max_projects` | 12 | canals digging at once: meant as a backstop, but the most at once is 6 to 12, so in some runs it binds for a year |
| `districts` | 6 | districts looked at each year |
| `branch_gap` | 3 | a district is this many cells from town, canal and navigable river |
| `district_spacing` | 6 | and this far from the year's other districts |
| `cool_years` | 5 | years a refused branch is left alone |
| `discover_coal`, `discover_stone` | 3,000,000 each | dug since 1750 before a survey is likely |
| `know` | 0.25 | how well a town must know canals to dig one |
| `hearsay` | 0.2 | what a town with no canal counts for when the know-how spreads |

None of these was searched for. The cost of a cell and the wharf are the
middle of what dx-rail suggested. The materials and wages make a level cell
cost about a fifth of a cell of the game's railway at the content's prices,
and a cell with its share of locks about two fifths; that came out at 0.16
of a year's output for the whole network, against the railways' 0.25 to
0.37, and I left it. The hurdle began at the railway's 0.06 and went to 0.1
after the first run, which is the one number here that was moved to make
fewer canals.

## What the sweeps showed

To 1900. Mean, and in brackets the least and greatest seed. Three sets of
sweeps stand behind this page, all under `.claude/night/reports/`:

- **The merged build's**, `canals-review/m-*`, on seeds 1 to 16: `m-port`
  and `m-port-f6` for canals alone on the port without and with six
  families, `m-game` and `m-game-f6` for the game's rules, each against the
  same rules without canals (`sw-port-base`, `sw-port-base-f6`,
  `m-game-base`, `m-game-base-f6`). This is the code that is merged, beside
  the trunk's kept town ground and its carters taken on by degrees, with the
  navvies taken on by degrees too. Where a table or a sentence does not say
  otherwise, the numbers are these.
- **The review's before the merge**, `canals-review/sw-*`, the same four
  pairs on the branch as it was cut: they repeat the first build's port rows
  to the last digit, and are where "does it hold on seeds 9 to 16" was first
  asked.
- **The first build's**, `sweep-canals*`, on seeds 1 to 8 (`-s9` for 9 to
  16): `-c` for carting alone and `-rc` for the game's railways with
  carting, which were not run again.

No run with canals failed in any of them: 72, 64 and 64 runs, and 64 more at
the extremes.

**Kilometres of canal, and when the first opens** (sixteen seeds; the two
rows marked are the first build's eight):

| | first opens | 1770 | 1780 | 1790 | 1800 | 1810 | 1820 | 1830 | 1850 | 1900 |
|---|---|---|---|---|---|---|---|---|---|---|
| canals alone | 1773 (1759 to 1787) | 36 | 224 | 485 | 665 | 716 | 757 | 789 | 813 | 820 (650 to 1,018) |
| six families | 1776 (1762 to 1786) | 28 | 196 | 508 | 706 | 762 | 804 | 834 | 854 | 861 (712 to 1,060) |
| with railways and carting (first build) | 1774 (1765 to 1789) | 24 | 136 | 353 | 604 | 705 | 719 | 743 | 783 | 786 (616 to 928) |
| the same, six families (first build) | 1776 (1765 to 1788) | 15 | 143 | 349 | 624 | 724 | 760 | 800 | 842 | 844 (734 to 988) |
| the game's rules | 1777 (1767 to 1796) | 10 | 139 | 451 | 592 | 657 | 709 | 759 | 839 | 863 (684 to 1,158) |
| the game's rules, six families | 1776 (1764 to 1796) | 19 | 193 | 494 | 615 | 685 | 730 | 783 | 847 | 850 (568 to 1,204) |

So: a survey in the 1760s, the first canal open in the 1770s, most of the
network dug between 1780 and 1800 at 20 to 26 km a year, and a tail to the
1830s. Britain's went on longer, with its mania in the 1790s; here the
1780s and 1790s are both. Seeds 9 to 16, which nothing was chosen on, give
the same picture as seeds 1 to 8 in every row.

A run digs 24 canals between towns alone (20 to 30) and 23 in company with
six families (11 to 42), cuts four to six junctions, and runs four to six
branches: to works (three), quarries, woods, collieries, iron mines and
farms (about one of each, or none). Many towns of these islands stand on
their own coal, so what a canal does for a coalfield it mostly does as a
trunk from the pit town to its customers. The network costs 0.16 of a
year's output (0.10 to 0.20), each canal counted at the output of the year
it opened, and the canals come in at 1.05 to 1.08 times what their promoters
expected. The slowest canal of a run takes five years, six at the most. None
was still unfinished at the end of the runs I looked at (seeds 1, 4, 12 and
15 in company, before the merge).

**What they carry.** In the ten years to 1800, of the weight times distance
the boats can be given a name for:

| | coal | stone and clay | grain | timber | ore and iron | works' other goods |
|---|---|---|---|---|---|---|
| canals alone | 0.39 | 0.19 | 0.28 | 0.03 | 0.01 | 0.03 |
| the game's rules, six families | 0.38 | 0.17 | 0.24 | 0.05 | 0.01 | 0.04 |

The rest is wool, cloth, bricks and tools in the towns' trade. Alone the
boats carry 38 million weight-km a year by 1780, 297 by 1800 and 454 by
1820, on hauls of 44 km (50 in company), and every cell of canal is in use.
Fares on that traffic, at the canal's own cost and its wharves, come to 0.40
of the capital laid out in 1800 (0.35 in company): as with the railways
nobody collects them, and a canal would be a very good thing to own.

**Before the railways, in every run.** The first canal opens before the
first railway in all 200 runs of the three sets: by 61 years alone (41 to
85) and by 55 in company with six families (14 to 89).

**And after.** Nothing is closed and nothing needs to be:

| | 1820 | 1840 | 1860 | 1880 | 1900 |
|---|---|---|---|---|---|
| boats, million weight-km a year, alone (the port's railways) | 454 | 351 | 91 | 71 | 90 |
| cells of canal carrying anything | 0.99 | 0.65 | 0.49 | 0.47 | 0.47 |
| boats, in company with six families | 417 | 353 | 75 | 12 | 13 |
| cells of canal carrying anything | 0.98 | 0.70 | 0.37 | 0.28 | 0.24 |

The boats' best year is 1832, alone and in company. By 1900 they carry 0.15
of it beside the port's thousand kilometres of railway and 0.02 of it beside
the game's 2,600, on hauls of 14 to 18 km, and what is left is stone and
grain (on the first build's eight seeds 0.33 and 0.36 alone, with coal down
to 0.10). Sixty-one per cent of the canal has track on it or beside it in
1900 alone, 84 per cent in company. The railway takes the trunks and leaves
the canal the short runs to places the track has not reached.

**Coal's price between towns does not close up.** The spread of coal's
price across towns (the standard deviation over the mean) with canals and
without, and the seeds of sixteen in which it is wider and narrower:

| | 1800 | 1820 | 1850 |
|---|---|---|---|
| alone | 0.38 for 0.35 (9/7) | 0.41 for 0.36 (13/3) | 0.42 for 0.42 (7/9) |
| six families | 0.43 for 0.39 (8/8) | 0.41 for 0.38 (11/5) | 0.45 for 0.43 (8/8) |
| the game's rules | 0.47 for 0.41 (14/2) | 0.50 for 0.47 (10/6) | 0.44 for 0.47 (4/12) |
| the game's rules, six families | 0.50 for 0.47 (10/6) | 0.53 for 0.51 (8/8) | 0.57 for 0.56 (9/7) |

If anything the spread is a little wider with canals in the canal years
(fourteen seeds of sixteen under the game's rules in 1800, thirteen alone in
1820). By 1850 it is no different, or under the game's rules without
families narrower on twelve seeds, when the railway has come. In the canal
years there is certainly no narrowing. The reason
is not the canals. On seed 7 coal is 0.39 to 0.47 in
the five towns that dig most of it and 1.3 to 1.8 everywhere else in 1800,
with a canal to every town by 1793 (`examples/canals.rs --coal`, first
build). Carrying a unit of coal sixty kilometres by canal, wharves and
handling included, costs about 0.12; the gap is more than 1. The rest is
the port's trade rule (`econ::step_month`, "trade between towns"). The share
of its stock a town's merchants send to another grows with the price gap
over the price there, but only up to a gap of half that price, and all a
town sends in a month is capped at `trade_cap` of its stock. With coal at
three times the pit price the rule is at both stops already, and making the
carriage cheaper moves no more of it. dx-rail found the same of the
railways. Coal's share of the households' warmth and of the works' heat does
not move either (0.90 for 0.90 and 0.99 for 0.99 in 1800).

**Works do not move to the canal.** Of all works, each counted by its size,
0.24 stand within a cell and a half of where the canals come to be, in 1760
before there is one and in 1800 and 1900 alike; 0.26 of the land is that
near. Under the game's rules with six families, where works stand by their
town, it is 0.71 before and 0.74, 0.69 and 0.58 in 1800, 1830 and 1900.
Canals join the towns, and the towns are where the works already were.

**Output per head and the real wage, alone.** Against the same rules
without canals, each relative to its own 1750, with the seeds that are
higher and lower. The first two rows are the merged build on sixteen seeds,
the rest the first build on eight:

| | output per head 1800 | 1850 | 1900 | real wage 1800 | 1850 | 1900 |
|---|---|---|---|---|---|---|
| canals alone | 1.42 for 1.41 (9/7) | 1.49 for 1.48 (7/9) | 1.49 for 1.47 (9/7) | 1.34 for 1.34 (7/9) | 1.28 for 1.30 (7/9) | 1.31 for 1.31 (8/8) |
| six families | 1.44 for 1.40 (10/6) | 1.53 for 1.55 (9/7) | 1.47 for 1.51 (5/11) | 1.34 for 1.31 (10/6) | 1.31 for 1.36 (8/8) | 1.30 for 1.36 (6/10) |
| with carting (first build) | 1.42 for 1.37 (7/1) | 1.56 for 1.46 (7/1) | 1.52 for 1.54 (3/5) | 1.36 for 1.32 (6/2) | 1.34 for 1.23 (7/1) | 1.22 for 1.22 (4/4) |
| with railways and carting (first build) | 1.42 for 1.37 (7/1) | 1.53 for 1.51 (6/2) | 1.48 for 1.52 (2/6) | 1.36 for 1.32 (6/2) | 1.31 for 1.29 (4/4) | 1.19 for 1.22 (4/4) |
| the same, six families (first build) | 1.44 for 1.40 (8/0) | 1.49 for 1.44 (4/4) | 1.54 for 1.49 (4/4) | 1.38 for 1.35 (7/1) | 1.26 for 1.29 (4/4) | 1.27 for 1.22 (6/2) |

Alone, under the port's rules, canals do nothing that can be told from the
spread between islands, for the reason railways did little: carting there
uses nobody's time, so a canal saves money that nobody was short of and
costs the navvies' work. With six families the port's own railway is later
for it (832 km in 1850 where there were 1,048, thirteen seeds of sixteen
lower), which is the port's rush of lines finding some of its traffic gone,
and the island of 1900 is if anything a little behind. Once carting takes
labour the canals give hands back. On the first build's eight seeds with
carting alone, the carters of 1800 are 12.3 per cent of all hands where they
were 13.3 (seven seeds of eight; 12.4 for 13.8 with six families, all
eight), the households' budgets buy 5.3 per cent more baskets (all eight)
and 3.4 per cent more goods are made a head; with carting and the port's
railways the gain lasts to 1850, and with the game's railways it is inside
the spread between seeds by then, because the branches do the same work.

## In company

Under the game's rules as they stand on the trunk (prices, the Engel curve,
machines, harvests, the chronicle, works, railways, carting, people and
kept town ground), canals on against canals off, on the merged build,
sixteen seeds:

| | no families | six families |
|---|---|---|
| output per head, 1800 | 1.63 for 1.60 (14/2) | 1.72 for 1.69 (16/0) |
| carters among all hands, 1800 | 0.117 for 0.131 (0/16) | 0.123 for 0.138 (0/16) |
| goods made a head, 1800 | 4.08 for 4.05 (12/4) | 4.47 for 4.44 (9/7) |
| real wage, 1800 | 1.58 for 1.58 (8/8) | 1.62 for 1.62 (5/11) |
| real wage, 1850 | 1.79 for 1.71 (8/8) | 1.99 for 2.00 (6/10) |
| real wage, 1900 | 2.77 for 2.74 (9/7) | 2.63 for 2.59 (8/8) |
| output per head, 1850 | 1.93 for 1.84 (10/6) | 2.17 for 2.17 (10/6) |
| first railway | 1837 for 1837 (6/8) | 1831 for 1830 (7/5) |
| km of railway, 1850 | 774 for 770 (7/6) | 1,219 for 1,214 (6/8) |
| km of railway, 1870 | 1,798 for 1,743 (9/7) | 2,052 for 2,054 (9/7) |
| km of railway, 1900 | 2,498 for 2,460 (8/8) | 2,597 for 2,614 (7/9) |
| famine years in a run | 1.1 for 1.75 (3/10) | 1.2 for 1.75 (4/8) |

Two things hold. The carters are one hand in seventy fewer in 1800, on all
thirty-two islands, and output a head is two per cent higher, on thirty of
them. The real wage of 1800 does not show it (thirteen islands up, nineteen
down, by nothing). Famine years are fewer on eighteen islands and more on
seven, which is worth watching and is not established.

Nothing else can be told, and in particular the canals do not hold the
game's railway back. The first build's page said they did, and that the
island with canals was behind in 1850: on seeds 1 to 8 without families
there was a third less railway in 1850 and a real wage lower on seven seeds
of eight. The review's sweeps before the merge found the same on those
seeds and the opposite on seeds 9 to 16 (more railway, and the wage higher
on all eight). On the merged build neither half shows it: the first line
comes in the same year, and the railway of 1850, 1870 and 1900 is the same
length to within a few per cent, as many islands up as down.

A run in company with six families takes about a seventh longer with canals
(25.5 seconds for 22.4 in the merged sweeps, run one after the other, and
23.3 for 19.2 without families). A run alone on the port takes a fifth to a
half longer, because there the canal promoters are the only ones doing the
sums: 28.6 seconds for 17.8 in the sweeps, whose baseline was made hours
earlier on a quieter machine, and a fifth when timed one run after another
on seed 3 before the merge.

**The scripted players** (`examples/bots.rs`, the game's rules, seeds 1, 2,
3 and 7) play all sixteen games through with canals on: on the first build
(`.claude/night/reports/canals-bots-game.txt`), on the review's before the
merge (`canals-review/bots-game.txt`) and on the merged build
(`canals-review/bots-game-merged.txt`). In all three the landlord and the
thoughtful player finish richest of the six families on every seed, as they
did before (`core-steps/people-probe/bots-game.txt`); the passive player and
the founder finish first to fourth, as they did. Seed by seed the fortunes of
1900 differ by up to half in either direction, and that earlier run was made
with slightly older code, so this says the game still plays and no more.

## For the viewer

Only when the switch is on:

- **The run's header** has `"canals": {"cost", "wharf", "links"}`. Each link
  is `{"a", "b", "begun", "opened"}`: two neighbouring cells, the year the
  canal it belongs to was promoted, and the moment it opened (a year with a
  fraction), or null if it was still being dug when the run ended. Links are
  in the order dug, each once. A link whose two ends both carried a canal
  before it is a junction that was cut, or a corner taken short (above).
- **The live API**: `LiveYear::canals` is the same list as it stands at the
  end of each year, every link so far and those being dug.
- **The `trans` layer** marks a cell with a canal open with 16 and one being
  dug with 32. A canal being dug no longer sets 8, which stays "a railway
  building", and canals are left out of `projects`.
- **News**: kinds `canal` (openings) and `canal_plan` (schemes). The first
  canal has weight 1, every later opening and scheme 0.3, a year's branches
  together and a junction 0.25. That is about sixty items a run, thirty of
  each kind. The survey is a `discovery` with no `tech`. The viewer has
  labels and a colour for the two kinds (`ui.rs`); it draws nothing yet. It
  reads the `trans` layer by its bits (1 and 2 for the road, 4 for track, 8
  for track building), so the two new bits do not disturb what it draws now.

A canal's level is not recorded. The viewer would have to work out pounds
and locks from the heights along the links, as map-viewer-ground suggests.

## What worries me

- **It all happens in thirty years.** Britain's canals took seventy. Here
  four fifths of the network is open by 1800. The know-how sets when the
  rush begins, not how long it lasts: `hearsay` at nought puts it off ten
  years and no more (above). It is short because the canals pay too well
  (below), so every town digs all it will ever dig as soon as it knows how.
  Dearer canals, or promoters who must first be paid a toll, would stretch
  it; nothing here was tuned against anything.
- **Canals are nearly all trunks.** The brief had coalfields and quarries
  far from navigable water in mind, and there are four or five such branches
  a run. Trunks are looked at first and the money goes to them; and a
  district must be three cells from a town, a canal and a navigable river to
  be looked at, which on these islands leaves few. Coal still fills two
  fifths of the boats.
- **They pay too well.** Fares of 0.3 to 0.4 of the capital a year say the
  canals are cheap for what they do, or that nobody collects a toll. A
  promoter needs 10 per cent on today's traffic and the traffic then
  doubles. A company that took the tolls, and shares a family could hold, is
  the same gap the railways page describes.
- **Nothing closes a canal.** After 1860 half the cells, or three quarters
  in company, carry nothing. They stay on the map and in the cost graph. The
  viewer could draw a canal with no boats as disused: `CanalState::traffic`
  has the weight through every cell in the last year, but it is not in the
  record.
- **A junction takes three years**, like every canal, and so does a branch
  of two cells. The months should go with the work.
- **Coal's price is no nearer level between towns**, and in 1800 a little
  further from it. If canals are to make coal cheap inland, the trade rule's
  two stops are where to look (above), not the canals.
- **Railway navvies are still free.** A canal's cost the island their work
  elsewhere and a railway's do not, which flatters the railway a little.
- **What the boats carry is partly an estimate.** The towns' trade is known
  good by good. The land's produce is taken at twelve times its last month,
  and a works' goods are whatever its cell moved beyond that. The parts add
  up to within a factor of two of the traffic counted along the paths (the
  test holds them to that and no closer).
- **Every search is a little slower once a canal is open**, since a cell of
  canal is two nodes. The searches that keep the trees use a heap and those
  that only want distances a bucket queue, as before.
- **One river, one canal.** A canal along a navigable river is allowed and
  is cheap to route (it is a river's valley). Nothing says a river made
  navigable is a different thing from a canal beside it.
- **Canals found no towns, and cost the island half of one.** A new town is
  founded where a market would save the most carting, so a canal that saves
  it first leaves less to found a town for ("On the integration branch",
  below). Britain's canals made towns, at their junctions and where they met
  a river. A wharf does not count for anything in `people.md`'s rule, and it
  might.
- **The know-how is not on the viewer's knowledge map**, since it is not one
  of the content's inventions.
- The viewer was built but not run. It does not draw canals.

## What the review checked

A second helper went over the mechanism before it was merged.

- **The port did not move.** Seed 3, and seed 7 with six families, write the
  same `series_monthly.csv`, `events.json` and `params.json` as the baseline
  taken at a5ea2ea, and the golden constants are untouched.
- **Off is off under the game's rules too.** With `canals.enabled=false` a
  run under the game's rules (seed 3, and seed 7 with six families) writes
  the same four files, `run.bin` included, byte for byte, as the build this
  branch was cut from. So the changes to `ways.rs`, `rail.rs`, `econ.rs` and
  the searches do nothing to a game without canals. After the merge with the
  trunk the same was done against the trunk's own build (`rust-3d` at
  cd9b41c), on the same two runs, and again every file is the same.
- **Extreme settings.** Each number in turn was set to nought and to
  something absurd on seeds 3 and 7 (`canals-review/x-*`): 64 runs, none
  failed, money conserved every month. A hurdle of 5, a wharf of 5, a cell
  of water dearer than road, wages of twenty times the materials or locks at
  2,000 cells a unit of height give no canal at all. A hurdle of nought
  gives 1,080 km where there were 785. `know` at nought is need alone and
  the first canal opens in 1754. Fifty years' worth of months
  (`build_months` 600) makes the slowest canal take 56 years and nothing
  else go wrong. The cap of a fifth of a town's hands as navvies never bound
  on these two seeds, and nor did twelve canals at once (on seeds 2, 4 and 5
  twelve are digging in the busiest year).
- **Beside the trunk.** The branch was merged with `rust-3d` (kept town
  ground, carters taken on by degrees, the anchors that go with a district)
  before it was handed on: three conflicts, all of them blocks appended at
  the same place. One of the trunk's tests then failed with canals on, and
  that is the fourth change below.
- **Four things were changed.** The navvies are taken on by degrees. As
  first built they left a town's labour market all at once on last month's
  wage bill, up to a sixth of its hands, and came back the month after if
  the stone had not arrived. Under the game's rules on seed 4 a town's wage
  moved by half or more in 35 of the 663 town-months with navvies in the
  town, and in one of the 4,500 without; `tests/people.rs` holds the game to
  one town-month in two hundred and failed at seven in a thousand. With a
  quarter of the way a month it is none in 937, and none on seeds 7 and 12.
  A junction is told and counted as a junction: before, the news promoted
  "62 km of canal" for a cut of one cell, and one canal in seven in the
  facts was such a cut. The railway's promoters see canals being dug. And
  the page no longer says that canals hold the game's railway back and
  leave the island behind in 1850, which neither seeds 9 to 16 nor the
  merged build bear out.

## On the integration branch

`night/core`, the merge of 11 November, where the canals met the mines for
the first time: everything above was measured on an island whose quarries
and pits were still the port's. The sweeps are under
`.claude/night/reports/core-steps/`, all with six families: `canals-before`
and `canals-after` on seeds 1 to 6, `canals-s7-16-before` and `canals-s7-16`
on seeds 7 to 16, `canals-x-on-off` and `canals-x-timing`. They were made a
second time on the committed build (30e7ddf), which on seeds 1 to 6 gives
every one of the 420 facts the merge gave before it was committed.

**The merge.** Five blocks that the mines and the canals had each added at
the same place (the end of `Params` and of its two constructors, of `State`
and of `LiveYear`, the list in the README, and the labour supply in
`econ::step_month`); both kept, the mines' first. The one with an order to
it is the labour supply. The hands who are not for hire leave it as carters,
then sinkers, then navvies. Each takes its share of what is still there, so
whoever comes last is squeezed first, and that should be the canal: carting
and sinking are work the town already lives by, and a canal can wait a
month.

**Off is the branch as it stood.** With `canals.enabled=false` seeds 1 to 6
give all 323 facts of `canals-before`, to the last digit. The golden
constants are untouched and no run failed (6, 6, 10, 10, 12 and 4 runs).

**The canals are the page's canals, a little longer.** Sixteen seeds with
the mines, and in brackets what "the game's rules, six families" gave above
without them: the first opens in 1774 (1776), 261 km in 1780 (193), 592 in
1790 (494), 695 in 1800 (615), 839 in 1830 (783), 900 in 1850 (847) and 903
in 1900 (850), from 688 to 1,184. A run digs 24 canals between towns (13 to
32), cuts five or six junctions (2 to 9) and runs branches to works (six),
woods (one or two), quarries and collieries (one of each) and, on every
other island, a farming district. It costs 0.14 of a year's output and comes
in at 1.04 times what was expected.
The boats carry 299 million weight-km a year by 1800 and 432 by 1820 on
hauls of 46 km, coal 0.41 of it, grain 0.23, stone and clay 0.16; their best
year is 1828 and by 1900 they carry 25. The first canal is 53 years ahead of
the first railway (31 to 66). At most 1,300 navvies are at work (780 to
2,100), nine or ten canals are digging at once (twelve, the cap, on some
islands) and the slowest takes five years. So beside the mines there is a
little more canal and it comes a little sooner. I did not trace why.

**What it does in company is what the page says.** On for off, sixteen
seeds, with the islands higher and lower:

| | on for off | |
|---|---|---|
| carters among all hands, 1800 | 0.119 for 0.134 | 0/16 |
| farm share of work, 1800 | 0.440 for 0.430 | 14/2 |
| output per head, 1800 | 1.66 for 1.65 | 15/1 |
| baskets a head, 1800 | 3.17 for 3.14 | 15/1 |
| real wage, 1800, 1850, 1900 | 1.59 for 1.61, 1.99 for 2.03, 2.61 for 2.65 | 8/8, 6/10, 8/8 |
| output per head, 1850, 1900 | 2.16 for 2.19, 2.67 for 2.70 | 7/9, 7/9 |
| people in 1900, thousands | 726 for 719 | 13/3 |
| towns, 1830, 1850, 1900 | 15.3 for 15.7, 16.6 for 17.3, 17.2 for 17.8 | 3/10, 4/11, 3/11 |
| first railway | 1827 for 1828 | 5/9 |
| km of railway, 1850, 1870, 1900 | 1,348 for 1,315, 1,829 for 1,863, 2,254 for 2,260 | 7/9, 6/10, 10/6 |
| works, 1900 | 729 for 726 | 7/9 |
| town cells and works' cells, 1900 | 324 for 320, 198 for 205 | 9/6, 6/10 |
| workings, 1900 | 491 for 492 | 7/8 |
| price swing: grain, iron, rails, cloth | 0.113 for 0.111, 0.174 for 0.173, 0.179 for 0.172, 0.039 for 0.038 | 9/7, 9/7, 11/5, 11/5 |
| famine years in a run | 0.4 for 1.4 | 3/9 |

The carters are one hand in seventy fewer in 1800 on every island, the hands
go to the farms, and a head makes and buys one per cent more. After that
nothing can be told from the spread between islands, the railway included.
Famine years are fewer again (on nine islands of sixteen, more on three).
With the two columns of "In company" that makes 27 runs against 10 on the
same sixteen islands, and it is beginning to look like something: grain that
can go by boat in a bad year. It is still not traced.

**One thing is new: half a town fewer.** The islands have the same towns to
1780. From 1790 the island with canals has founded fewer, and by 1850 it has
16.6 where it had 17.3, fewer on eleven islands and more on four. That is
`people.md`'s rule doing what it says: a town is founded where a market
would save the most carting, and a canal has saved some of it already. The
people are not lost (there are one per cent more of them in 1900) and the
share of them in towns is 0.42 either way, a third of a point lower with
canals. Seed 7 founds no town from 1772 to 1790 with canals where it had
founded two, and that is why `tests/chronicle.rs` now watches fifty years
of it for its five foundings and not forty: the same question asked of ten
more years.

**The canal's orders do not strain its materials.** Stone, bricks, timber
and clay cost what they cost against the wage, decade by decade from the
1770s to the 1820s, to within one or two per cent (in the 1780s stone is
0.194 of the wage for 0.195 and bricks 0.454 for 0.456), and two per cent
more stone is dug a head. No price sits at a floor or a ceiling.

**Six seeds are not sixteen.** On seeds 1 to 6 alone 1850 looks like a
gain (baskets a head 4.91 for 4.47, the real wage 1.89 for 1.83, both 4/2)
and on seeds 7 to 16 like a loss (5.41 for 5.83, 3/7; 2.05 for 2.14, 2/8).
It is neither, as the mines page found of the same year: these islands have
their fastest years around 1850, and a few years either way moves the mark.

**The time a run takes.** One at a time on seeds 3 and 5, 17.1 seconds for
15.6; twelve at a time on seeds 1 to 6, 30.6 for 25.1. A tenth to a fifth
more, as before.

**The scripted players** play all sixteen games through beside the mines
(`core-steps/canals-bots-game.txt`, seeds 1, 2, 3 and 7). The landlord and
the thoughtful player finish richest of the six families on every seed, as
they did; the passive player and the founder finish first to fourth.

**The viewer** builds and its tests pass. It reads the `trans` layer by its
bits (1 and 2 the road, 4 track, 8 track building), in the railways' new
course and profiles too, so 16 and 32 disturb nothing, and it still draws no
canal.

## Facts added to the sweep

Under a heading of its own, "Canals", at the end of the table.

In every run, canals or not: `coal_price_spread_Y` for 1780, 1800, 1820 and
1850, the standard deviation over the mean of coal's yearly price across the
active towns; `coal_share_of_warmth_Y` and `coal_share_of_heat_Y`, the
observer's shares in those years.

With canals on: `canal_first_year` (the first opening) and
`canal_first_begun`; `canal_km_Y` for every decade from 1760 and the last
year; `canal_last_begun`; `canal_before_railway` (1 if the first canal opened
before the first railway) and `canal_years_before_railway`; for 1780 and
every twenty years after, `canal_mton_km_Y` (million weight-km a year by
boat, the mean of the ten years ending then), `canal_haul_km_Y` and
`canal_cells_in_use_Y`; for 1800, 1840 and the last year
`canal_carries_coal_Y`, `_stone_clay_`, `_grain_`, `_timber_`, `_iron_ore_`
and `_works_goods_`, shares of what the boats carried that can be named;
`canal_traffic_peak_year` and `canal_traffic_end_vs_peak`;
`canal_works_near_Y` for 1760, 1800, 1830 and the last year, the share of
works by size within a cell and a half of where the canals are at the end,
and `canal_land_near`; `canal_km_beside_track_Y`; `canal_lines_trunk` (the
canals between towns, junctions left out), `canal_junctions_cut` and
`canal_lines_to_collieries`, `_quarries`, `_iron_mines`, `_farms`, `_woods`,
`_works`; `canal_cost_in_years_of_output`, `canal_spent_over_expected`,
`canal_slowest_years`, `canal_most_digging`, `canal_most_navvies`; and
`canal_fares_over_capital_Y` for 1800, 1830 and the last year.

`examples/canals.rs` prints one run: every canal with what it runs to and
what it cost, the network and its traffic decade by decade beside the
railway, and with `--coal` the price of coal town by town.
