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
only the money was short. Nothing is barred for good.

**Lines building count as laid** when the next is reckoned. So two lines of
one year never pay twice for cells they share, the second is not credited
with what the first already saves, and a branch can be promoted off a trunk
that is still building. A line lays only its own cells when it opens.

**Money.** As before, the landlords of a line's two towns put up to 35 per
cent of their cash and the households 8 per cent, and the line goes ahead if
half the expected cost is raised. For a branch the two towns are the market
town of the district and the town at the junction. Three things are new:

- A line orders each month no more than its cash will pay for at the day's
  prices, so it never owes anything: dear rails slow it down.
- A line short of cash for the month calls on its two towns' landlords, a
  tenth of their cash at most.
- What is left when it opens goes back to the subscribers in proportion to
  what each put in.

Every one of these is a transfer between purses `econ::money_total` already
counts (a line's cash is `Project::cash`, as in the port).

**Priced properly.** A cell of line takes `cost_mult` (five) times the port's
rails, timber and stone, and a line takes `build_months` (24) instead of ten
if the market fills every order. And the promoters read the order book: the
price of rails they reckon with is the year's price times one plus the rails
all lines still have on order, over a year's make of rails in the whole
country. A crowd of schemes prices the next one out until the mills catch up.
That, and the money the towns can raise, is what paces the rush; the cap on
lines building at once (`max_projects`, 40) is never reached (the most on any
seed is 35).

**News.** The first railway is the big item it was. Every trunk and every
town's first line is an item of its own. The district branches of a year are
folded into one, lighter, which says how many opened, how many km, and what
they serve (seed 3: "6 branch lines open. 63 km of branch line opened in
1841, to the quarries near Penheath, Highford and Sandside, the collieries
near Penhurst, the works near Carwater and the iron mines near Greybourne.").
Nothing is
called a mineral line for pits and works any more; what a branch serves is
whatever fills most carts in its district, land sector by land sector, with
the works counted by what they move beyond the land's own produce.

## The numbers

`RailwayParams`, in `railways.rs`. Off by default; `game()` switches it on
with these.

| | | |
|---|---|---|
| `enabled` | false | the game's promoters replace the port's |
| `cost_mult` | 5 | rails, timber and stone per cell, times the port's 40, 30 and 30 |
| `build_months` | 24 | months a line takes if every order is filled (the port: 10) |
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
the port; then the same with six families. Mean, and in brackets the least and
greatest seed. Sweep folders `sweep-railways*` under `.claude/night/reports/`.

Kilometres of line:

| | 1830 | 1840 | 1850 | 1860 | 1870 | 1880 | 1890 | 1900 |
|---|---|---|---|---|---|---|---|---|
| port | 260 | 743 | 990 | 1,014 | 1,016 | 1,020 | 1,023 | 1,024 (920 to 1,258) |
| railways | 67 | 458 | 1,160 | 1,614 | 1,825 | 2,001 | 2,125 | 2,236 (2,102 to 2,383) |
| port, six families | 358 | 825 | 1,057 | 1,070 | 1,071 | 1,072 | 1,072 | 1,072 (880 to 1,186) |
| railways, six families | 219 | 611 | 1,345 | 1,724 | 1,966 | 2,137 | 2,278 | 2,404 (2,306 to 2,506) |
| railways and carting | 100 | 336 | 832 | 1,268 | 1,599 | 1,792 | 1,929 | 2,053 (1,939 to 2,158) |
| railways and carting, six families | 197 | 495 | 931 | 1,419 | 1,777 | 1,957 | 2,107 | 2,222 (2,050 to 2,366) |

So: a slower start than the port's (dearer lines), a rush of 45 to 70 km a
year through the 1840s and 1850s, and then branches at 20 km a year falling
to 11 by the end of the century. The network grows in every decade from 1840 on every
seed (in one seed of eight with carting and families, five decades of six).

| | port | railways | railways, six families |
|---|---|---|---|
| first line | 1828 (1815 to 1838) | 1831 (1820 to 1840) | 1829 (1811 to 1842) |
| towns with track in 1860 | 0.98 | 1.00 | 1.00 |
| cell traffic within a cell of track, 1900 | 0.39 | 0.77 (0.75 to 0.80) | 0.81 (0.78 to 0.82) |
| dearest year for rails over the mean of 1800 to 1825 | 2.95 (1.18 to 5.30) | 3.15 (1.72 to 4.86) | 2.63 (1.09 to 5.92) |
| what the network cost, in years of output | 0.02 to 0.04 | 0.42 (0.36 to 0.50) | 0.38 (0.33 to 0.43) |
| fares on track over capital laid out, 1870 | | 0.075 (0.053 to 0.090) | 0.088 (0.064 to 0.113) |
| the same, 1900 | | 0.071 (0.045 to 0.094) | 0.084 (0.067 to 0.107) |
| most lines building at once | 6 | 25.5 (22 to 27) | 27.5 (24 to 35) |
| railway news items (openings; schemes) | 38; 36 | 81; 24 | 89; 26 |

(The port's cost is dx-rail's figure. "Fares" are the year's traffic over
each cell of track at the running cost of track; the capital is each line's
outlay brought to that year's prices by the cost of living. Nobody collects
these fares yet: see below.)

What the lines serve, of about 220 lines a run (no families): 24 trunks; one
town's first line; and of the branches, 92 to works, 51 to quarries and clay
pits, 24 to woods, 21 to farms, 4 to collieries and iron mines, 4 to pasture.
Branches are 68 per cent of the kilometres.

**Output per head and the real wage do not move.** Means of 1860 to 1900,
output per head in 1750 money and the real wage with 1750 as 1:

| | no railways at all | port's railways | railways |
|---|---|---|---|
| output per head | 6.23 | 6.29 | 6.21 |
| real wage | 1.31 | 1.33 | 1.32 |
| output per head, six families | | 6.55 | 6.46 |
| real wage, six families | | 1.45 | 1.43 |

A seed's own two runs differ by five per cent either way, so none of these
differences is established. This is dx-rail's finding again: under the port's
rules carting uses nobody's time, so cheaper carriage frees nothing. What
railways do to living standards once carting takes labour is in
`carting.md`.

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
order book is one good.

## What worries me

- **The branches do not visibly pay the country back.** With everything else
  the same, output per head in 1860 to 1900 is 0 to 2 per cent lower with the
  branches than with the port's trunks (four pairs of sweeps, with and
  without carting and families: minus 1.3, minus 0.1, minus 1.3, minus 1.6
  per cent; each inside the spread between seeds, all four the same way). The
  network costs 0.4 of a year's output where the port's cost 0.03, and by the
  promoters' own sum a line needs to save only 2.4 per cent of its cost a
  year to go ahead (the 6 per cent hurdle after the 2.5 for induced traffic;
  both are the port's numbers, tuned to lines a fifth as dear). I tried a
  stricter promoter, `--set rail_induced=1,1.5` with carting on: 1,680 and
  1,850 km in 1900 instead of 2,050, still rising in every decade but one in
  one seed, and
  output per head of 5.88 and 5.88 against 5.92. So it is not the marginal
  lines; the difference is noise, or it is the trunks' own cost. I left the
  port's 2.5.
- **Subscribers get nothing back.** They pay for the line and the saving goes
  to whoever carts. Fares over track come to 7 or 8 per cent of the capital a
  year, so a company that took them could pay a believable dividend, and a
  family could hold its shares. That is dx-rail's recommendation 5 and is not
  built.
- **A line that runs out of money waits.** It calls on its towns each month
  and is never abandoned. Under the game's rules the slowest line of a run
  took 8 to 19 years from its promotion to its opening (11 on average over
  the seeds), all of them in the rush, when rails are dear. None was still
  waiting in 1900 after ten years or more (`rail_lines_stuck_at_end` is 0 on
  every seed), but nothing prevents it, and while it waits other lines are
  reckoned as if it were there.
- **Every cell of track is a station**, as in the port.
- **With carting on, the start is uneven**: the first line comes between 1816
  and 1850 across eight seeds, and in the latest seed only half the towns
  have track in 1860. That is the smaller economy discovering railways later
  in that seed, not the promoters.
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
`rail_fares_over_capital_1870` and `_1900`, `rail_lines_trunk`,
`rail_lines_to_towns`, `_farms`, `_pastures`, `_quarries`, `_mines`,
`_works`, `_woods`, and `rail_km_share_branches`.
