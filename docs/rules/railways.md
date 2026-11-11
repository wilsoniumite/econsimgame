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
four per cent of one year's output, where Britain's cost about forty.

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

**Every line is valued by one sum**, trunks too: the carting it would save in
a year, over the whole map, at today's traffic. The promoters copy the cost
graph, lay the line on the copy, and measure again every cell's cost to its
market and the served towns' cost to every other town; the fall, times the
traffic and the trade, times `freight`, is the saving. It is multiplied by
`rail_induced` for the traffic a line draws and set against the expected cost;
the line goes ahead if that return clears `rail_hurdle`. Nothing says where a
line ought to pay. (The sum is quick because only what the new line can reach
more cheaply is searched again; a test holds it to a search from scratch.)

**A refused line cools off.** A pair of towns is left alone for four years, a
town or district for `cool_years` (five), or for two if the sum was good and
only the money was short. Nothing is barred for good. (A refused district
also keeps the promoters away from anything within four cells of it for
those years, a town included.)

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
town of the district and the town at the junction. Three things are new:

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

**News.** The first railway is the big item it was. Every trunk and every
town's first line is an item of its own. The district branches of a year are
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

## What a canal age needs

`ways.rs` is written so that canals are a second `Way` and not a second
planner. A `Way` carries: its id (`Project::way`), its words for the news, the
running cost of a cell, the materials of a cell, the months a line takes, what
the ground charges for building it (`BuildCost`: lowland, hills, mountains,
a term for a river cell, a term per unit of height, a laid cell), the
technology and how well a town must know it, the hurdle and the induced
traffic, the good whose order book is read, the caps and spacings, and three
functions: the cell layer it reads, the layer it writes, and the book
(`WayBook`) its promoters keep. `railways::way` is the pattern.

A canal needs, beyond its `Way`:

- a cell layer in `State` and a `WayBook` of its own;
- one more branch in `transport::build_graph`, so that a canal cell is cheap
  to cross in the real cost graph (`ways::lay` already prices any set of
  cheaper edges for the sums, so the promoters need nothing more);
- its numbers in `BuildCost`: a negative `river` (a valley is where a canal
  wants to be) and a `slope` several times the railway's 40 (locks);
- a call to `ways::plan` each year and `ways::progress` each month beside the
  railway's (in `rail::plan` and `rail::progress`, or in `sim.rs`);
- a bit in the record's `trans` layer if the viewer is to draw it.

Two things are still the railway's alone and are marked in the code: opened
lines are also written to `State::rail_opened` (the record's list), and the
order book is one good. Nothing has yet run two ways at once.

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
  trunks take most of what there is to save.
- **Subscribers get nothing back.** They pay for the line and the saving goes
  to whoever carts. Fares over track come to 7 to 9 per cent of the capital
  a year, so a company that took them could pay a believable dividend, and a
  family could hold its shares. That is dx-rail's recommendation 5 and is not
  built.
- **Calls have no limit.** A line goes ahead on an estimate, and its
  subscribers then answer calls for whatever the materials turn out to cost.
  Taken together the lines cost what was expected (0.92 to 1.01), but the
  worst line of a run costs 2.5 to 9 times its estimate, in the rush, and
  nothing asks whether it is still worth building at that price. A company
  with shares would be the place to settle this too.
- **Nothing abandons a line.** A line short of money or of rails waits. The
  slowest of a run takes 7 to 13 years from its promotion to its opening,
  and the lines that lean on it wait with it. None was still waiting after
  ten years at the end of any run (`rail_lines_stuck_at_end` is 0 on all 56).
- **The date of the railway age is not the promoters'.** It follows the
  discovery, which moves by decades between islands, and comes later with
  carting. Under the game's rules the first line opens in 1850 or later on
  7 islands of 16 (4 of 16 with six families), so anything that needs
  railways by mid-century will often not find them.
- **Every cell of track is a station**, as in the port.
- **What a branch serves is an estimate.** The works' share of a district's
  carts is the cells' traffic less twelve times last month's land produce.
  Nearly half the branches are counted as serving works, more than dx-rail's
  count by land use.
- The viewer was built but not looked at with the new network on screen. It
  draws track from the same cell layer as before, so branches should simply
  appear; the list of opened lines it is given marks a branch as the port
  marked a spur (`b` is -1).

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
