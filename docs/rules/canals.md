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

Four or five junctions are cut in a run (two to eleven). The news tells one
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
are counted as employed. In the busiest year of a run that is 800 to 2,800
people on an island of 200,000. Railway navvies are still paid and cost
nobody's time, as in the port: that is the railways' to change.

**It takes years.** Thirty-six months if every order is filled, and a canal
orders no more each month than its cash will pay for, wages included. From
promotion to opening the slowest of a run takes five to eight years. Every
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

Seeds 1 to 8 to 1900 unless a line says otherwise. Mean, and in brackets the
least and greatest seed. Two sets of sweeps stand behind this page, both
under `.claude/night/reports/`. The first build's are `sweep-canals*`:
`-base` and nothing for the port without and with canals, `-c` for carting,
`-rc` for the game's railways and carting, `-game` for the game's rules
(canals off in `-game-base`), `-f6` with six families, `-s9` for seeds 9 to
16. The review's are `canals-review/sw-*`, each on seeds 1 to 16 against a
baseline made by the same build: `sw-port`, `sw-port-f6`, `sw-game` and
`sw-game-f6`, with `-base` for the same rules without canals. No run with
canals failed, 72 in the first set and 64 in the second, and the review's
port rows repeat the first build's to the last digit. The rows for carting
alone and for railways with carting are the first build's and were not run
again; since then the railway's promoters see canals being dug, which moves
the rows with the game's railways by a few kilometres.

**Kilometres of canal, and when the first opens.**

| | first opens | 1770 | 1780 | 1790 | 1800 | 1810 | 1820 | 1830 | 1850 | 1900 |
|---|---|---|---|---|---|---|---|---|---|---|
| canals alone | 1770 (1759 to 1786) | 48 | 265 | 494 | 670 | 719 | 746 | 800 | 820 | 830 (674 to 974) |
| six families | 1776 (1762 to 1786) | 26 | 185 | 468 | 700 | 801 | 820 | 837 | 849 | 856 (710 to 966) |
| with railways and carting | 1774 (1765 to 1789) | 24 | 136 | 353 | 604 | 705 | 719 | 743 | 783 | 786 (616 to 928) |
| the same, six families | 1776 (1765 to 1788) | 15 | 143 | 349 | 624 | 724 | 760 | 800 | 842 | 844 (734 to 988) |
| the game's rules | 1778 (1765 to 1794) | 20 | 155 | 379 | 505 | 596 | 645 | 673 | 737 | 752 (500 to 894) |
| the game's rules, six families | 1777 (1767 to 1782) | 16 | 143 | 408 | 553 | 620 | 667 | 703 | 801 | 807 (684 to 962) |
| seeds 9 to 16: canals alone | 1775 (1761 to 1787) | 16 | 229 | 508 | 687 | 729 | 765 | 783 | 799 | 802 (718 to 972) |
| seeds 9 to 16: the game's rules | 1778 (1770 to 1795) | 7 | 139 | 437 | 537 | 647 | 694 | 727 | 810 | 881 (650 to 1,000) |
| seeds 9 to 16: the game's rules, six families | 1780 (1770 to 1795) | 5 | 88 | 440 | 556 | 659 | 721 | 785 | 865 | 876 (716 to 1,228) |

So: a survey in the 1760s, the first canal open in the 1770s, most of the
network dug between 1780 and 1800 at 20 to 26 km a year, and a tail to the
1830s. Britain's went on longer, with its mania in the 1790s; here the
1780s and 1790s are both. Seeds 9 to 16, which nothing was chosen on, give
the same picture.

A run digs 24 canals between towns alone (21 to 29) and 22 in company with
six families (16 to 38 over sixteen seeds), cuts four or five junctions, and
runs four or five branches: to works (two or three), quarries, woods,
collieries, iron mines and farms (about one of each, or none). Many towns of
these islands stand on their own coal, so what a canal does for a coalfield
it mostly does as a trunk from the pit town to its customers. The network
costs 0.16 of a year's output (0.13 to 0.20), each canal counted at the
output of the year it opened, and the canals come in at 1.06 times what
their promoters expected. None was still unfinished at the end of the runs I
looked at (seeds 1, 4, 12 and 15 in company).

**What they carry.** In the ten years to 1800, of the weight times distance
the boats can be given a name for:

| | coal | stone and clay | grain | timber | ore and iron | works' other goods |
|---|---|---|---|---|---|---|
| canals alone | 0.40 | 0.18 | 0.28 | 0.03 | 0.01 | 0.03 |
| the game's rules, six families | 0.39 | 0.18 | 0.22 | 0.05 | 0.01 | 0.04 |

The rest is wool, cloth, bricks and tools in the towns' trade. The boats
carry 44 million weight-km a year by 1780, 306 by 1800 and 446 by 1820, on
hauls of 44 km, and every cell of canal is in use. Fares on that traffic, at
the canal's own cost and its wharves, come to 0.41 of the capital laid out
in 1800: as with the railways nobody collects them, and a canal would be a
very good thing to own.

**Before the railways, in every run.** The first canal opens before the
first railway in all 72 runs of the first set and all 64 of the review's, by
60 years alone (39 to 86 over sixteen seeds) and by 57 in company with six
families (25 to 77).

**And after.** Nothing is closed and nothing needs to be:

| | 1820 | 1840 | 1860 | 1880 | 1900 |
|---|---|---|---|---|---|
| boats, million weight-km a year, alone (the port's railways) | 446 | 291 | 56 | 75 | 95 |
| cells of canal carrying anything | 1.00 | 0.59 | 0.49 | 0.48 | 0.48 |
| mean haul, km | 43 | 32 | 17 | 17 | 18 |
| boats, in company with six families | 356 | 367 | 137 | 15 | 14 |
| cells of canal carrying anything | 0.98 | 0.72 | 0.34 | 0.27 | 0.22 |

The boats' best year is 1829 alone and 1836 in company. By 1900 they carry
0.17 of it beside the port's thousand kilometres of railway and 0.02 of it
beside the game's 2,500, on hauls of 14 to 18 km, and what is left is stone
and grain (0.33 and 0.36 alone; coal is down to 0.10). Sixty-one per cent of the
canal has track on it or beside it in 1900 alone, 87 per cent in company.
The railway takes the trunks and leaves the canal the short runs to places
the track has not reached.

**Coal's price between towns does not close up.** The spread of coal's
price across towns (the standard deviation over the mean) with canals and
without:

| | 1800 | 1820 | 1850 |
|---|---|---|---|
| alone | 0.38 for 0.35 | 0.38 for 0.34 | 0.50 for 0.45 |
| six families | 0.40 for 0.40 | 0.43 for 0.39 | 0.41 for 0.45 |
| the game's rules, six families | 0.49 for 0.45 | 0.53 for 0.53 | 0.55 for 0.59 |

Alone, five or six seeds of eight move up; with families and in company as
many move down as up. Over sixteen seeds the spread of 1800 is a little
wider with canals in three of the four comparisons (0.38 for 0.35 alone, on
eleven seeds of sixteen; 0.42 for 0.39 with six families, ten; 0.47 for 0.42
under the game's rules, twelve) and not in the fourth (0.47 for 0.45 with
six families in company, six of sixteen), and by 1850 nothing can be told.
Nothing here is established, and certainly no narrowing. The reason is not
the canals. On seed 7 coal is 0.39 to 0.47 in
the five towns that dig most of it and 1.3 to 1.8 everywhere else in 1800,
with a canal to every town by 1793 (`examples/canals.rs --coal`). Carrying a
unit of coal sixty kilometres by canal, wharves and handling included, costs
about 0.12; the gap is more than 1. The rest is the port's trade rule
(`econ::step_month`, "trade between towns"). The share of its stock a town's
merchants send to another grows with the price gap over the price there,
but only up to a gap of half that price, and all a town sends in a month is
capped at `trade_cap` of its stock. With coal at three times the pit price
the rule is at both stops already, and making the carriage cheaper moves no
more of it. dx-rail found the same of the railways. Coal's share of the
households' warmth and of the works' heat does not move either (0.90 for
0.90 and 0.99 for 0.99 in 1800).

**Works do not move to the canal.** Of all works, each counted by its size,
0.25 stand within a cell and a half of where the canals come to be, in 1760
before there is one and in 1800, 1830 and 1900 alike; 0.26 of the land is
that near. Under the game's rules, where works stand by their town, it is
0.76 before and 0.84, 0.80 and 0.73 after. Canals join the towns, and the
towns are where the works already were.

**Output per head and the real wage.** Against the same rules without
canals, each relative to its own 1750, with the seeds that are higher and
lower:

| | output per head 1800 | 1850 | 1900 | real wage 1800 | 1850 | 1900 |
|---|---|---|---|---|---|---|
| canals alone | 1.43 for 1.40 (6/2) | 1.46 for 1.48 (4/4) | 1.50 for 1.46 (5/3) | 1.35 for 1.33 (4/4) | 1.32 for 1.34 (4/4) | 1.34 for 1.29 (6/2) |
| six families | 1.46 for 1.39 (7/1) | 1.57 for 1.55 (5/3) | 1.50 for 1.50 (3/5) | 1.37 for 1.31 (7/1) | 1.36 for 1.35 (4/4) | 1.34 for 1.35 (3/5) |
| with carting | 1.42 for 1.37 (7/1) | 1.56 for 1.46 (7/1) | 1.52 for 1.54 (3/5) | 1.36 for 1.32 (6/2) | 1.34 for 1.23 (7/1) | 1.22 for 1.22 (4/4) |
| with railways and carting | 1.42 for 1.37 (7/1) | 1.53 for 1.51 (6/2) | 1.48 for 1.52 (2/6) | 1.36 for 1.32 (6/2) | 1.31 for 1.29 (4/4) | 1.19 for 1.22 (4/4) |
| the same, six families | 1.44 for 1.40 (8/0) | 1.49 for 1.44 (4/4) | 1.54 for 1.49 (4/4) | 1.38 for 1.35 (7/1) | 1.26 for 1.29 (4/4) | 1.27 for 1.22 (6/2) |

Alone, under the port's rules, canals do little that can be told from the
spread between islands, for the reason railways did little: carting there
uses nobody's time. Once carting takes labour the canals give hands back.
In 1800 the carters are 12.3 per cent of all hands where they were 13.3
(seven seeds of eight; 12.4 for 13.8 with six families, all eight), the
households' budgets buy 5.3 per cent more baskets (all eight) and 3.4 per
cent more goods are made a head. With carting and the port's railways the
gain lasts to 1850: 6.8 per cent more output a head and a real wage of
1.34 for 1.23, seven seeds of eight. With the game's railways it is inside
the spread between seeds by 1850, because the branches then do the same
work. By 1900 nothing can be told in any of them.

## In company

Under the game's rules (prices, the Engel curve, machines, harvests, the
chronicle, works, railways, carting and people), canals on against canals
off. These are the review's sweeps, on the build that is merged, seeds 1 to
8 and seeds 9 to 16 side by side:

| | no families, seeds 1 to 8 | seeds 9 to 16 | six families, seeds 1 to 8 | seeds 9 to 16 |
|---|---|---|---|---|
| output per head, 1800 | 1.63 for 1.61 (8/0) | 1.68 for 1.65 (7/1) | 1.71 for 1.66 (8/0) | 1.73 for 1.69 (7/1) |
| carters among all hands, 1800 | 0.112 for 0.122 (0/8) | 0.116 for 0.128 (0/8) | 0.116 for 0.128 (0/8) | 0.118 for 0.132 (0/8) |
| goods made a head, 1800 | 4.13 for 4.09 (6/2) | 4.17 for 4.13 (6/2) | 4.47 for 4.38 (7/1) | 4.44 for 4.41 (5/3) |
| real wage, 1800 | 1.61 for 1.63 (3/5) | 1.63 for 1.64 (4/4) | 1.65 for 1.65 (3/5) | 1.67 for 1.69 (2/6) |
| real wage, 1850 | 1.76 for 1.83 (1/7) | 1.86 for 1.69 (8/0) | 1.96 for 2.05 (4/4) | 2.13 for 2.13 (3/5) |
| real wage, 1900 | 2.76 for 2.85 (4/4) | 2.79 for 2.80 (4/4) | 2.86 for 2.72 (6/2) | 2.65 for 2.75 (4/4) |
| first railway | 1843 for 1839 (5/1) | 1844 for 1846 (3/4) | 1834 for 1832 (5/2) | 1838 for 1836 (3/0) |
| km of railway, 1850 | 427 for 644 (0/6) | 478 for 385 (4/1) | 968 for 997 (4/3) | 780 for 881 (0/8) |
| km of railway, 1870 | 1,464 for 1,629 (0/8) | 1,522 for 1,381 (5/3) | 1,902 for 1,896 (5/3) | 1,826 for 1,877 (2/6) |
| km of railway, 1900 | 2,281 for 2,372 (2/6) | 2,328 for 2,302 (4/4) | 2,532 for 2,522 (3/5) | 2,508 for 2,496 (3/5) |
| famine years in a run | 1.0 for 2.1 (1/5) | 0.6 for 1.1 (2/4) | 0.9 for 1.75 (2/4) | 0.75 for 0.75 (2/2) |

Two things hold on every set of seeds. The carters are one hand in a hundred
fewer in 1800, on all thirty-two islands, and output a head is two or three
per cent higher, on thirty of them. The real wage of 1800 does not show it
(twelve islands up, twenty down, by a hundredth).

The rest does not hold. On seeds 1 to 8 without families the canals seemed
to hold the railway back and to leave the island behind in the middle of the
century: a third less railway in 1850, a tenth less in 1870, and a real wage
of 1850 lower on seven seeds of eight. The first build's page said so. On seeds 9 to
16 it is the other way about: more railway in 1850 and 1870, and a real wage
of 1850 higher on all eight. With six families there is a little less
railway in 1850 (874 km for 939 over the sixteen seeds, four up and eleven
down) and nothing in the wage. So a canal does take some of what an early
line would have saved, and the first line is a year or two later on more
islands than not, but what that does to the island in 1850 is inside the
difference between one island and the next. By 1900 nothing can be told.

A run in company with six families takes about a tenth longer with canals,
and a run alone on the port a fifth to a third longer, because there the
canal promoters are the only ones doing the sums. The first build timed 18.9
seconds for 16.8 and 25 for 18; the review, one run after another on seed 3
with other work on the machine, 17.2 and 15.8 for 14.6 and 16.8 in company
and 15.4 and 20.5 for 12.4 and 17.9 alone. The sweeps' own seconds were
taken under heavier work still and say less.

**The scripted players** (`examples/bots.rs`, the game's rules, seeds 1, 2,
3 and 7) play all sixteen games through with canals on, on the first build
(`.claude/night/reports/canals-bots-game.txt`) and again on the merged one
(`canals-review/bots-game.txt`). The landlord and the thoughtful player
finish richest of the six families on every seed, as they did before
(`core-steps/people-probe/bots-game.txt`); the passive player and the
founder finish first to fourth, as they did. Seed by seed the fortunes of
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
  the searches do nothing to a game without canals.
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
- **Three things were changed.** A junction is told and counted as a
  junction: before, the news promoted "62 km of canal" for a cut of one
  cell, and one canal in seven in the facts was such a cut. The railway's
  promoters see canals being dug. And the page no longer says that canals
  leave the island behind in 1850, which seeds 9 to 16 do not bear out.

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
