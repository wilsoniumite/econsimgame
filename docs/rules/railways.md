# Railways

Branch lines priced on what they save, at a believable cost. Not in the
Python. The switch is `railways.enabled`; the code is `ways.rs` (the promoters,
who know nothing about railways in particular) and `railways.rs` (the numbers,
the railway as one kind of way, and the facts for the sweep). The diagnosis
this answers is `docs/probes/dx-rail.md`.

## What was wrong

Under the port the network is built in one rush and stops: 990 km by 1850 and
1,024 km in 1900, on eight seeds. It does not stop because lines fail their
sums. It stops because the promoters have nothing left to look at. `rail::plan`
only considers pairs of towns, and by the 1850s every pair is joined;
`rail::plan_spurs` only considers the 25 busiest cells of the map, which by
then are all on the line. Meanwhile three fifths of what the country carts
starts more than a cell from any track, and the whole network has cost two to
four per cent of one year's output, where Britain's cost about forty. (These
are the port's numbers, dx-rail's and the first sweeps'. The port's rules are
held byte for byte by `tests/golden.rs`, so they still stand, but they were
not measured again for this page.)

## What the game's promoters do

With the switch on, `rail::plan` hands the year over to `ways::plan`, and
`rail::plan_spurs` does nothing. Every year, once some town knows how
(`rail_know`, as before), the promoters look at three kinds of line:

- **Trunks** between two towns, the busiest pairs first, as before.
- **A town's first line.** Every town without track is routed to the nearest
  point of the network, wherever that is. A line may leave from any junction,
  not only from a town.
- **District branches.** The carting bill of every 5 by 5 block of cells is
  added up (each cell's traffic times its cost to market), and the eight
  dearest blocks that are three cells or more from track, and six from each
  other, are each routed to the nearest point of the network. These are the
  quarries, the groups of works, the collieries and the farming districts
  that the port never reaches.

"Nearest" is by what the ground charges to build over, reckoned as the
port's promoters reckon a route (`transport::rail_build_graph`): hills cost
2.2 times the lowland, mountains 6, a river 1.5 more for the bridge, every
unit of height 40, and a cell already carrying track 0.15. So a branch may
leave the network some way from the point nearest in kilometres.

**Every line is valued by one sum**, trunks too: the carting it would save in
a year, over the whole map, at today's traffic. The promoters copy the cost
graph, lay the line on the copy, and measure again every cell's cost to its
market and the served towns' cost to every other town; the fall, times the
traffic and the trade, times `freight`, is the saving. It is multiplied by
`rail_induced` for the traffic a line draws and set against the expected cost
(the materials, and the navvies' wages on top at three tenths of every
shilling, as in the port); the line goes ahead if that return clears
`rail_hurdle`. Nothing says where a line ought to pay. A district branch is
valued on the cells' carting alone; a trunk counts its two towns' trade with
every other town as well, and a town's first line its one town's. (The sum
is quick because only what the new line can reach more cheaply is searched
again; a test holds it to a search from scratch.)

**A refused line cools off.** A pair of towns is left alone for four years, a
town or district for `cool_years` (five), or for two if the sum was good and
only the money was short. A pair with no way between them over land is left
for ten, since the ground will not change. Nothing is barred for good. (A
refused district also keeps the promoters away from anything within four
cells of it for those years, a town included.)

**Lines building count as laid** when the next is reckoned. So two lines of
one year never pay twice for cells they share, the second is not credited
with what the first already saves, and a branch can be promoted off a trunk
that is still building. A line lays only its own cells when it opens, and it
does not open before the lines its route runs over: a branch never opens
onto a junction that is not there yet. (Before the review a four-cell branch
to the woods could open months ahead of the first railway it hung from.)

**The cost is reckoned at prices somebody pays.** The promoters used to take
the year's price of rails, timber and stone in the buying town. In a town
that neither makes nor buys rails that price sits near the floor, because
nobody has ever paid it, and a line promoted on it then paid many times the
estimate once its own orders arrived. Now what the town's own makers can
supply in the time is reckoned at the town's price, and the rest at no less
than what the country's makers got for it (each town's price weighted by
what it made). The buying town is whichever of the line's two towns is
cheaper by that reckoning.

**Money.** As before, the landlords of a line's two towns put up to 35 per
cent of their cash and the households 8 per cent, and the line goes ahead if
half the expected cost is raised. For a branch the two towns are the market
town of the district (for a town's first line, the town itself) and the town
at the junction: the junction's own market town if that has track, and
otherwise the town with track that is cheapest to reach from the junction.
Three things are new:

- A line orders each month no more than its cash will pay for at the day's
  prices, so it never owes anything: dear rails slow it down.
- A line short of cash for the month calls on its two towns' landlords, a
  tenth of their cash at most. Where a town's landlords are in debt, its
  households, who subscribed with them, answer in their place with a
  fortieth of their cash. (Landlords can be in debt for decades. Before the
  review a line whose landlords were could call on nobody, and two branches
  on seed 12 waited forty years.)
- What is left when it opens goes back to the subscribers in proportion to
  what each put in.

Every one of these is a transfer between purses `econ::money_total` already
counts (a line's cash is `Project::cash`, as in the port).

**Priced properly.** A cell of line takes `cost_mult` (five) times the port's
rails, timber and stone, and a line takes `build_months` (24) instead of ten
if the market fills every order. And the promoters read the order book: the
price of rails they reckon with is marked up by one plus the rails all lines
still have on order, over a year's make of rails in the whole country. A
crowd of schemes prices the next one out until the mills catch up. That, and
the money the towns can raise, is what paces the rush; the cap on lines
building at once (`max_projects`, 40) is a backstop (the most at once is 19
to 37 across the sweeps below, and 40 in one run of 56).

**News.** The first railway is the big item it was (the first trunk to
open, weight 1). Every trunk and every town's first line is an item of its
own when it opens (weight 0.45). A scheme is news only for a trunk, as in the
port: branches and first lines are told when they open, not when they are
promoted. The district branches of a year are
folded into one, lighter, which says how many opened, how many km, and what
they serve ("6 branch lines open. 63 km of branch line opened in 1841, to
the quarries near Penheath, Highford and Sandside, the collieries near
Penhurst, the works near Carwater and the iron mines near Greybourne."). It
is dated by the last of them to open. Nothing is called a mineral line for
pits and works any more; what a branch serves is whatever fills most carts
in its district, land sector by land sector, with the works counted by what
they move beyond the land's own produce.

## The numbers

`RailwayParams`, in `railways.rs`. Off by default; `game()` switches it on
with these.

| | | |
|---|---|---|
| `enabled` | false | the game's promoters replace the port's |
| `cost_mult` | 5 | rails, timber and stone per cell, times the port's 40, 30 and 30 |
| `build_months` | 24 | months a line takes if every order is filled (the port: 10); under 1 is taken as 1 |
| `order_book` | true | promoters mark rails up by what is on order |
| `max_projects` | 40 | lines building at once: a backstop, not the pace |
| `districts` | 8 | districts looked at each year |
| `branch_gap` | 3 | a district is looked at only this many cells from track |
| `district_spacing` | 6 | and this far from the year's other districts |
| `cool_years` | 5 | years a refused branch is left alone |

The hurdle (`rail_hurdle`, 0.06), the induced traffic (`rail_induced`, 2.5),
the know-how (`rail_know`) and the running cost of a cell of track
(`transport::RAIL_COST`) are the port's and are shared with it.

## What the sweeps showed

Seeds 1 to 8 to 1900, the port's rules with only this switched on, against
the port; then the same with six families, and with carting. Mean, and in
brackets the least and greatest seed. These are the numbers after the
review's repairs (sweep folders `sweep-railways-review-*` under
`.claude/night/reports/`); the network is about a tenth smaller than the
first version's, because lines are now priced at what they will really cost.
Everything in this section was measured on the port's island, before the
railways joined the game's other rules, and none of it can be read from
today's sweep (the game's rules with every switch on); what that sweep shows
is under "In company", below.

Kilometres of line:

| | 1830 | 1840 | 1850 | 1860 | 1870 | 1880 | 1890 | 1900 |
|---|---|---|---|---|---|---|---|---|
| port | 260 | 743 | 990 | 1,014 | 1,016 | 1,020 | 1,023 | 1,024 (920 to 1,258) |
| railways | 78 | 373 | 1,006 | 1,519 | 1,738 | 1,849 | 1,932 | 2,022 (1,931 to 2,104) |
| port, six families | 358 | 825 | 1,057 | 1,070 | 1,071 | 1,072 | 1,072 | 1,072 (880 to 1,186) |
| railways, six families | 149 | 521 | 1,192 | 1,610 | 1,855 | 1,973 | 2,083 | 2,167 (2,072 to 2,320) |
| railways and carting | 96 | 289 | 722 | 1,141 | 1,455 | 1,657 | 1,805 | 1,885 (1,548 to 1,990) |
| railways and carting, six families | 138 | 393 | 852 | 1,244 | 1,606 | 1,802 | 1,908 | 2,021 (1,865 to 2,140) |

So: a slower start than the port's (dearer lines), a rush of nearly 60 km a
year through the 1840s and 1850s, 22 a year in the 1860s, and then branches
at about 10 km a year to the end of the century. Without carting the network
grows in every decade from 1840 on every seed. With carting it does so from
its first line on: one seed of eight discovers the railway only in 1843 and
has nothing to show for the 1840s.

| | port | railways | railways, six families | railways and carting | the same, six families |
|---|---|---|---|---|---|
| first line | 1828 (1815 to 1838) | 1833 (1821 to 1845) | 1831 (1815 to 1843) | 1836 (1816 to 1854) | 1833 (1806 to 1855) |
| towns with track in 1860 | 0.98 | 1.00 | 1.00 | 0.89 (0.13 to 1) | 0.91 (0.50 to 1) |
| cell traffic within a cell of track, 1900 | 0.39 | 0.75 (0.73 to 0.78) | 0.77 (0.73 to 0.81) | 0.71 (0.63 to 0.74) | 0.74 (0.70 to 0.77) |
| dearest year for rails over the mean of 1800 to 1825 | 2.95 (1.18 to 5.30) | 3.05 (1.71 to 5.68) | 3.61 (1.25 to 7.89) | 2.69 (1.91 to 3.37) | 3.13 (2.11 to 5.07) |
| what the network cost, in years of output | 0.02 to 0.04 | 0.37 (0.30 to 0.42) | 0.36 (0.27 to 0.55) | 0.36 (0.32 to 0.39) | 0.38 (0.30 to 0.49) |
| what the lines cost over what was expected | | 0.92 (0.78 to 1.08) | 0.98 (0.78 to 1.22) | 0.93 (0.85 to 1.16) | 1.01 (0.82 to 1.20) |
| the same for the worst line of a run | | 5.4 (2.6 to 9.2) | 4.8 (2.6 to 6.5) | 3.4 (2.5 to 4.3) | 5.4 (3.0 to 7.6) |
| fares on track over capital laid out, 1870 | | 0.075 (0.056 to 0.096) | 0.090 (0.059 to 0.126) | 0.066 (0.043 to 0.093) | 0.067 (0.036 to 0.094) |
| the same, 1900 | | 0.083 (0.063 to 0.117) | 0.093 (0.077 to 0.120) | 0.074 (0.060 to 0.088) | 0.078 (0.049 to 0.106) |
| most lines building at once | 6 | 25 (20 to 29) | 29 (21 to 40) | 23 (19 to 29) | 27 (21 to 33) |
| the slowest line, years from promotion to opening | | 9.1 (7.0 to 11.5) | 10.2 (6.7 to 13.5) | 8.7 (6.8 to 12.2) | 10.4 (6.6 to 12.8) |
| railway news items (openings; schemes) | 38; 36 | 81; 27 | 85; 28 | 77; 27 | 83; 27 |

(The port's cost is dx-rail's figure. "Fares" are the year's traffic over
each cell of track at the running cost of track; the capital is each line's
outlay brought to that year's prices by the cost of living. Nobody collects
these fares yet: see below. Before the review's repairs the lines cost 1.2
to 1.4 times what was expected and the worst of a run 8 to 18 times.)

What the lines serve, of about 190 lines a run (no families): 27 trunks; one
town's first line; and of the branches, 87 to works, 36 to quarries and clay
pits, 20 to woods, 14 to farms, 4 to pasture, 3 to collieries and iron
mines. Branches are 66 per cent of the kilometres.

**Output per head and the real wage do not move.** Means of 1860 to 1900,
output per head in 1750 money and the real wage with 1750 as 1:

| | no railways at all | port's railways | railways |
|---|---|---|---|
| output per head | 6.23 | 6.29 | 6.26 |
| real wage | 1.31 | 1.33 | 1.34 |
| output per head, six families | | 6.54 | 6.46 |
| real wage, six families | | 1.45 | 1.42 |

A seed's own two runs differ by five per cent either way, so none of these
differences is established. This is dx-rail's finding again: under the port's
rules carting uses nobody's time, so cheaper carriage frees nothing. What
railways do to living standards once carting takes labour is in
`carting.md`.

**Out of sample.** Seeds 9 to 16, which the numbers above were never tuned
on:

| | 1830 | 1840 | 1850 | 1860 | 1870 | 1880 | 1890 | 1900 |
|---|---|---|---|---|---|---|---|---|
| port | 149 | 445 | 603 | 923 | 998 | 1,008 | 1,011 | 1,012 (812 to 1,264) |
| railways | 24 | 202 | 513 | 972 | 1,435 | 1,710 | 1,838 | 1,937 (1,742 to 2,160) |
| railways and carting | 4 | 65 | 193 | 490 | 962 | 1,487 | 1,719 | 1,837 (1,646 to 1,976) |
| railways and carting, six families | 36 | 169 | 522 | 901 | 1,358 | 1,628 | 1,823 | 1,952 (1,760 to 2,210) |

The shape holds: the network is about 1,900 km in 1900, 0.71 to 0.74 of cell
traffic is within a cell of track, the lines cost 0.92 to 1.04 of what was
expected, the slowest takes 7 to 13 years, no run fails. The calendar does
not. These islands discover the railway ten years later than seeds 1 to 8
even under the port (first line 1838 against 1828), the game's lines take
another five to open, and carting another seven: the first line is in 1843
(1827 to 1858) with railways alone and in 1850 (1830 to 1863) with carting,
and in 1860 fewer than half the towns have track (three quarters with
families). The rails price peak is
also less tame here with carting: 4.4 and 5.2 times on average, and 10 and
11 times in the worst seed of each sweep.

## In company

Everything above was measured on the port with the railways alone, or with
carting. They were merged into the game's rules together, after the
merchants' prices, the Engel curve, the machines, the harvests, the
chronicle and the life cycle for works, so the island they are built on is
not the port's: a wage buys nearly three times as much by 1900, half the
hands are in a few hundred large works, and some of the mills roll steel
rails. The
integration's sweeps are under `.claude/night/reports/core-steps/`:
`railways-before` and `railways-after` (the game's rules with six families
on seeds 1 to 6, before the merge and after), `railways-x-2x2` and
`railways-x-2x2-s7-12` (seeds 1 to 6 and 7 to 12 with `railways.enabled` and
`carting.enabled` each off and on, so each can be read alone in company),
and `railways-probe/bots-game.txt`. No run failed, and a run takes the
twelve or thirteen seconds it took.

**Switched off, nothing moves.** With both switches off under the game's
rules the merged code gives every one of the 186 facts of every seed as the
code before the merge gave it.

Kilometres of line, six families, carting on in every row but the third. The
first three rows are the merge's sweeps on twelve seeds; the last is today's
(`merged-game-f6-l`: the game's rules as they stand on 12 November, with the
canals, the mines, people, kept town ground and the books all on, seeds 1 to
8; see "Today", below):

| | 1830 | 1840 | 1850 | 1860 | 1870 | 1880 | 1890 | 1900 |
|---|---|---|---|---|---|---|---|---|
| the port's promoters, in company | 436 | 739 | 882 | 945 | 961 | 962 | 962 | 972 (808 to 1,156) |
| the game's, in company | 314 | 809 | 1,282 | 1,754 | 2,100 | 2,369 | 2,539 | 2,630 (2,458 to 2,714) |
| the game's, alone on the port (seeds 1 to 8, above) | 138 | 393 | 852 | 1,244 | 1,606 | 1,802 | 1,908 | 2,021 (1,865 to 2,140) |
| the game's, today (seeds 1 to 8) | 336 | 830 | 1,328 | 1,621 | 1,835 | 1,993 | 2,129 | 2,248 (2,008 to 2,400) |

| the game's railways and carting, six families | alone on the port (seeds 1 to 8) | in company (seeds 1 to 12) | today (seeds 1 to 8) |
|---|---|---|---|
| first line | 1833 (1806 to 1855) | 1828 (1804 to 1848) | 1827 (1809 to 1842) |
| decades from 1840 in which the network grew | all six | all six, on every seed | all six, on every seed |
| towns with track in 1860 | 0.91 (0.50 to 1) | 1.00 | 0.99 (0.95 to 1) |
| cell traffic within a cell of track, 1900 | 0.74 (0.70 to 0.77) | 0.87 (0.85 to 0.89) | 0.82 (0.78 to 0.86) |
| dearest year for rails over the mean of 1800 to 1825 | 3.13 (2.11 to 5.07) | 1.58 (0.76 to 4.11) | 1.30 (0.64 to 2.89) |
| what the network cost, in years of output | 0.38 (0.30 to 0.49) | 0.25 (0.17 to 0.32) | 0.21 (0.17 to 0.27) |
| what the lines cost over what was expected | 1.01 (0.82 to 1.20) | 0.81 (0.71 to 0.92) | 0.77 (0.71 to 0.91) |
| the same for the worst line of a run | 5.4 (3.0 to 7.6) | 3.0 (1.4 to 7.3) | 3.2 (2.2 to 5.6) |
| fares on track over capital laid out, 1870 and 1900 | 0.067, 0.078 | 0.23, 0.26 (0.20 to 0.37) | 0.29, 0.32 (0.22 to 0.38) |
| most lines building at once | 27 (21 to 33) | 26 (22 to 33) | 24 (20 to 28) |
| the slowest line, years from promotion to opening | 10.4 (6.6 to 12.8) | 7.0 (5.0 to 9.6) | 6.2 (5.4 to 8.5) |
| lines still building after ten years, at the end | 0 | 0 | 0 |
| railway news items (openings; schemes) | 83; 27 | 83; 20 | 95; 27 |

(Today's column is read from the sweep's `summary.txt`: the facts
`rail_first_year`, `rail_decades_grown_of_6`, `rail_towns_with_track_1860`,
`rail_traffic_near_track_1900`, `rail_price_peak_vs_1800_1825`,
`rail_cost_in_years_of_output`, `rail_spent_over_expected`,
`rail_worst_overrun`, `rail_fares_over_capital_1870` and `_1900`,
`rail_most_lines_building`, `rail_slowest_line_years`,
`rail_lines_stuck_at_end`, `news_railway` and `news_rail_plan`. A news item
"of openings" counts each trunk and first line and each year's folded
branches once.)

So the promoters do in company what they do alone: a slower start than the
port's (three years later to the first line, on every seed, and less line
than the port's in 1830), then growth in every decade to the end of the
century, every town joined by 1860, no line stuck. What is different is the
size. The network is 2,630 km where alone it was 2,020, and the rush is
sooner: 1,280 km by 1850 for 850. There is more to carry (at the content's
base prices the island makes 7.2 of goods a head in 1850 and 11.3 in 1900,
where the port with the same two switches made 4.3 and 5.1), and rails are
cheaper, because the machines' steel rail stands for three iron ones and the
mills that roll it can grow. So more lines clear the same hurdle, they come
in under their estimates instead of on them, and the traffic over them would
pay a quarter of their cost a year in fares where alone it paid a
fourteenth. Of some 300 lines a run 20 are trunks and one or two a town's
first line; of the branches 107 run to works, 76 to quarries and clay pits,
47 to woods, 31 to farms, 8 to pasture and 7 to collieries and iron mines.
Branches are 78 per cent of the kilometres (66 alone).

**Today.** On today's sweep the same promoters, beside everything merged
since (the canals, the mines, people, kept town ground and the books), build
less line than at the merge, though not less than alone: 2,250 km in 1900
where the merge's sweeps had 2,630, and 1,330 by 1850 for 1,280, so the rush
is as soon and the tail is shorter (the 1870s add 160 km where they added
270). The shape is the same: a first line in 1827 (1809 to 1842), growth in
every decade on every seed, every town joined by 1860 but for one town on
one island, no line stuck, and the lines in at 0.77 of their estimates. The
island makes 7.9 of goods a head in 1850 and 11.4 in 1900
(`goods_per_head_1850` and `_1900`). Fares over capital are higher than at
the merge, nearly a third of it a year. Of some 250 lines a run 27 are
trunks and one or two a town's first line; of the branches 118 run to works,
43 to woods, 31 to farms, 14 to quarries and clay pits, 11 to pasture and 4
to collieries and iron mines (`rail_lines_*`), and branches are 74 per cent
of the kilometres (`rail_km_share_branches`). The branches to quarries are a
fifth of what they were at the merge, 14 for 76. The mines (`mines.md`)
came in between, and they decide where stone and clay are dug; I did not
trace whether that is why. (The integration that brought the mines and the
canals together already had 2,254 km in 1900 with canals and 2,260
without: `canals.md`, "On the integration branch".)

**In company the branches do pay the country back.** With carting on, the
game's railways against the port's, seed for seed on twelve seeds: in 1900
households' budgets buy 3.5 per cent more baskets (8.56 for 8.27, nine
seeds of twelve higher), 3.6 per cent more goods are made a head (eleven of
twelve), carters are 6.6 per cent of all hands where they were 8.2 (eleven
of twelve lower), and a run has 2.0 famine years for 3.3. In 1850 there is
nothing to see (4.88 baskets for 4.87): the two networks of 1850 differ by
400 km and the trunks do most of the saving. Alone on the port the same
comparison showed nothing or a little less (the first worry below). With
carting off the branches are still worth 2 per cent of goods a head in 1900
(eleven seeds of twelve), which the port's rules never showed; I did not
trace through which of the other mechanisms. (All of this is the merge's
2x2 sweeps. Today's sweep has only the game's railways, so the comparison
cannot be read from it, and it was not made again.)

One thing was different in company, and was mended in a few lines.

- **The chronicle called every branch a mineral line.** Its story of a coal
  boom says what carries the coal away, and it read the record's list of
  opened lines as the port wrote it: a line with no second town was a mineral
  spur. Under the game's promoters such a line may run to farms, woods or
  works, or be a town's own first line, so a boom was told with "A mineral
  line opened in 1847 brings it down from the pits" whatever that year's
  branch ran to. With `railways.enabled` the story now asks
  the promoters' book what the line was for (`railways::line_for_coal`): the
  railway that reached the town, as a trunk or as its first line, "carries
  it away"; a branch to the district's collieries "brings it down from the
  pits"; any other branch is not mentioned. In seed 7's game three booms
  named a mineral line: one now names the railway that reached Ashdelf in
  1827 and two say nothing of a line, since no line to the pits had opened.
  With the switch off the story is the port's, word for word.
  `tests/chronicle.rs` holds the game to it.

The news: a run's railway items are 83 openings and 20 schemes where the
port's promoters gave 35 and 34, and of all of them the major feed keeps
one, the first railway, as before. The major feed is the length it was
(58 items a run for 57.5). (Those were the merge's sweeps. Today's gives 95
openings and 27 schemes a run, one scheme for each trunk; the major feed is
not among the sweep's facts and was not counted again.)

## Other ways: the canals

`ways.rs` was written so that canals would be a second `Way` and not a second
planner, and they now are (`canals.md`): two ways run at once under the
game's rules, the railway's promoters and then the canal's, each year and
each month. A `Way` carries: its id (`Project::way`), its words for the news,
the running cost of a cell, the materials of a cell and, for a way with
locks, the materials for every unit of height it climbs (`rise`), the wages
paid on every shilling of materials (`wages`: three tenths for the railway),
the months a line takes, what the ground charges for building it
(`BuildCost`: lowland, hills, mountains, a term for a river cell, a term per
unit of height, a laid cell), the technology and how well a town must know
it, the hurdle and the induced traffic, the good whose order book is read,
the caps and spacings, and three functions: the cell layer it reads, the
layer it writes, and the book (`WayBook`) its promoters keep. Then the
things the canal needed and the railway leaves empty: `wharf` (a way that
floats, joined link by link, with a charge to get on and off), `heads` (cells
a branch may start from besides the way itself), `opened` (what the way
keeps beyond its layer when a line opens), `first_any` (the first line of
any kind is the great news, not the first trunk), `local` (districts are
looked at only where their market town knows how) and `rivals` (another
way's lines under way, laid on the promoters' copy of the cost graph). The
railway's `rivals` puts the canals being dug on its graph, so no line is
promoted on carting a canal will have taken. `railways::way` and
`canals::way` are the two patterns.

The canal did not go into `transport::build_graph`: an open canal is a
second layer over the cells (`transport::Graph::boat`), put on the cost
graph each year by `canals::float`. Its `BuildCost` has a negative `river`
(-0.3: a valley is where a canal wants to be) and a `slope` of 80 for locks,
twice the railway's 40. The record marks a canal open with 16 in `trans` and
one being dug with 32.

Two things are still the railway's alone and are marked in the code: opened
lines are also written to `State::rail_opened` (the record's list), and the
order book is one good (rails; the canal reads none).

## What worries me

- **The branches do not visibly pay the country back.** With everything else
  the same, output per head in 1860 to 1900 is a little lower with the
  branches than with the port's trunks in all four pairs of sweeps: minus
  0.5 per cent, minus 1.2 with families, minus 0.6 with carting, and minus
  2.5 with carting and families. The first three are inside the spread
  between seeds. The last is seven seeds of eight, with the real wage at
  1.25 against 1.32, and that is the pair the game is played under. The
  network costs 0.36 of a year's output where the port's cost 0.03, and by
  the promoters' own sum a line needs to save only 2.4 per cent of its cost
  a year to go ahead (the 6 per cent hurdle after the 2.5 for induced
  traffic; both are the port's numbers, tuned to lines a fifth as dear).
  Before the review's repairs a stricter promoter was tried
  (`--set rail_induced=1,1.5` with carting on): about a fifth less line and
  no change in output per head, so it is not the marginal lines. The
  trunks take most of what there is to save. In company with the game's
  other rules the same comparison comes out the other way by 1900 (3.5 per
  cent more baskets a head, nine seeds of twelve; "In company", above), and
  shows nothing in 1850.
- **Subscribers get nothing back.** They pay for the line and the saving goes
  to whoever carts. Fares over track come to 7 to 9 per cent of the capital
  a year, so a company that took them could pay a believable dividend, and a
  family could hold its shares. That is dx-rail's recommendation 5 and is not
  built. In company the fares came to 23 to 26 per cent of the capital a
  year at the merge, and on today's sweep to 29 in 1870 and 32 in 1900: on
  the game's island a railway would be the best thing anyone could own, and
  nobody owns it.
- **Calls have no limit.** A line goes ahead on an estimate, and its
  subscribers then answer calls for whatever the materials turn out to cost.
  Taken together the lines cost what was expected (0.92 to 1.01), but the
  worst line of a run costs 2.5 to 9 times its estimate, in the rush, and
  nothing asks whether it is still worth building at that price. (Those are
  the port's island. On today's sweep the lines together cost 0.77 of the
  estimate and the worst of a run 2.2 to 5.6 times it.) A company with shares
  would be the place to settle this too.
- **Nothing abandons a line.** A line short of money or of rails waits. The
  slowest of a run takes 7 to 13 years from its promotion to its opening
  (5.4 to 8.5 on today's sweep), and the lines that lean on it wait with it.
  None was still waiting after ten years at the end of any run
  (`rail_lines_stuck_at_end` is 0 on all 56, and on today's eight).
- **The date of the railway age is not the promoters'.** It follows the
  discovery, which moves by decades between islands, and comes later with
  carting. Under the game's rules the first line opens in 1850 or later on
  7 islands of 16 (4 of 16 with six families), so anything that needs
  railways by mid-century will often not find them. (Those were the port
  with these two switches. In company the first line was open by 1848 on all
  of seeds 1 to 12, but the length of 1850 ran from 130 km to 1,956; on
  today's sweep the first line is open by 1842 on all of seeds 1 to 8 and
  the length of 1850 runs from 575 km to 1,804.)
- **Every cell of track is a station**, as in the port.
- **In company it is a great deal of railway.** 2,630 km is some 1,300 cells
  of track on an island of about 5,900 land cells: more than one cell in
  five has a line through it, 87 per cent of all cell traffic starts within
  a cell of one, and at the height of the rush 22 to 33 lines are building
  at once. (Those were the merge's sweeps, and the count of land cells was
  taken then. On today's sweep it is 2,250 km, some 1,120 cells or about
  one land cell in five, 82 per cent of the traffic within a cell of track,
  and 20 to 28 lines building at once.) By area that is near Britain's
  density of line in 1900, and the promoters' sums say every kilometre
  pays, but nobody has looked at it on
  the map, where a line is drawn across its whole cell and every cell of it
  has a station. If it reads as too much, `branch_gap` and `districts` are
  the dials (how far from track a district must be, and how many are looked
  at a year), and a higher `rail_hurdle` would thin it too. None was tried
  in company.
- **What a branch serves is an estimate.** The works' share of a district's
  carts is the cells' traffic less twelve times last month's land produce.
  Nearly half the branches were counted as serving works, more than dx-rail's
  count by land use; on today's sweep it is more than half (118 of some 220
  branches a run).
- The viewer draws track from the same cell layer as for the port's lines
  (bit 4 of `trans`, and 8 for track building), so the branches are drawn
  as any other line. The list of opened lines it is given marks a branch,
  and a town's first line too, as the port marked a spur (`b` is -1).

## Facts added to the sweep

Under "Railways": `rail_km_Y` for every decade from 1830;
`rail_decades_grown_of_6`, the decades from 1840 that ended with more line
than they began with; `rail_towns_with_track_1860` and `_1900`;
`rail_traffic_near_track_1900`, the share of the last year's cell traffic
that starts within a cell of track; `rail_price_peak_vs_1800_1825`. And from
the promoters' book, with no value while the switch is off:
`rail_cost_in_years_of_output` (each line's outlay over the output of the
year it opened, summed), `rail_most_lines_building`,
`rail_slowest_line_years`, `rail_lines_stuck_at_end`,
`rail_fares_over_capital_1870` and `_1900`, `rail_spent_over_expected` (what
all the lines cost over what their promoters expected) and
`rail_worst_overrun` (the same for the line that overran most),
`rail_lines_trunk`, `rail_lines_to_towns`, `_farms`, `_pastures`,
`_quarries`, `_mines`, `_works`, `_woods`, and `rail_km_share_branches`.
