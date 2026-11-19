# Crossings: one bank, one bridge

Crossing water is a thing that costs, that is built once by somebody, and
that everybody then uses. Not in the Python. The switch is
`crossings.enabled`, on under the game's rules and off under the port's; the
code is `crossings.rs` in the core, `network.rs` (`route`, the fords) in the
viewer and `bridge.rs` (`road_ford`) in the generators. It acts on the new
land only ([land.md](land.md), `land.enabled`), whose fine rivers say where
the water is. On the old island it does nothing at all. It is the first stage
of "Ways" in the backlog, and step 3 of the four in
[the roads probe](../probes/dx-roads.md), section 7.

Stella's note of 19 November: "A while ago, we talked about how roads
crossing meandering rivers 4 times in a row doesn't make much sense, nobody
would build that many bridges."

## Why roads crossed rivers again and again

Under the port a river that is not navigable costs a road nothing to cross,
so the cheapest way wanders over it as often as it likes; and the picture
puts a bridge wherever its drawn road meets its drawn river. Measured on the
new land, two things made the picture's roads cross the same river again
and again, and only one of them was the price:

- **A way is drawn straight between two cells' meeting places**, the spot
  where each cell's roads meet (2 km apart). Along a lowland valley the
  river's meander belt lies across that line, and the straight line cuts
  the belt two, three or four times. On island 7, 96 of the 2,002 ways
  between neighbouring cells that cross water cut the same river two or
  more times, 81 of them three times or more. That is the road crossing a
  meandering river four times in a row, inside a single link, and no price
  in the cost graph can make it go round: the cost graph knows the link,
  not its line.
- **The meeting places of a valley's cells took either bank** as it
  happened: each cell's spot was the dry, level place nearest the cell's
  middle, and down a valley that falls on one bank, then the other. A road
  from one to the next crossed and crossed back for want of a place to meet.

So the rule changes the geometry as well as the price.

## What it does

**A table of crossings.** Every two neighbouring land cells are joined by a
way between their meeting places. The meeting places are worked out in the
core (`crossings::meeting_places`, moved there from the viewer's
`newland/middles.rs`, and the viewer now calls the core's), so the
simulation and the picture agree on every way's line to the last bit; a
viewer test holds them to it on islands 2, 3 and 7. Wherever a way meets
the line of a river carrying a cubic metre a second or more (every river the
viewer draws), it crosses water there, as wide as the river is at that
point. Widths are reckoned as the textbook's 4.4 times the root of the flow
(`roads::WIDTH_PER_ROOT_FLOW`), not the land page's drawn widths, which are
two and a half times that.

**One bank.** A cell that a river runs through has its meeting place on the
side of the river where 60 per cent or more of the cell lies; where the
river runs through its middle, on the side the cell up the same river took.
So down a reach the cells' meeting places keep to one bank.

**Round the meanders.** Where a way's straight line cuts the same river an
even number of times, the way keeps to its bank and goes round the meander,
along the water's edge, and does not cross at all; it costs the extra length
at 1 a cell (a cell of open lowland), bridge or none. Where it cuts it an
odd number of times, it crosses once, at the middle of those points,
straight over, and goes round the rest. (Where two rivers' points run among
each other along a way, it is drawn straight and every point is a crossing;
this is rare.)

**A ford or a ferry costs.** A way over water with no bridge costs `ford`
(0.03) more to pass in the cost graph for each metre of water forded, plus
`ford_base` (0); a river wider than `ford_width` (25 m) cannot be forded and
is crossed by a ferry at `ferry` (1.5). A lowland cell with no road costs 1
and with a made road 0.45, so a 13 m stream costs 0.4 to ford and a 22 m one
0.66. The charge is carting time, paid to the carters as every other unit of
the graph is. It is added to both directions of the link, except where both
cells have a railway (a railway bridges what it crosses as part of its own
cost) or both are navigable (a boat going down the river does not cross it).

**A bridge is built once by the parish.** Each year, after the roads have
measured the year's traffic (`sim::update_roads`, unchanged), the weight
carted over each way over water is added up from the same trees and flows
the roads were worn with. A way with no bridge is priced at `bridge_stone`
(8 stone) a metre of water, bought at the market of the town whose district
holds it, with `wages` (2) on every shilling of stone paid to that town's
men. Where its fords' or ferry's charge on last year's traffic would pay
`hurdle` (5 per cent) a year on that, the parish puts it on its list, best
first, as far as a highway rate of `rate` (2 per cent) of its landlords'
cash goes in a year (a quarter of that of its households' where the
landlords are in debt). One bridge works a parish at a time.

The work is the prototype's road works (`ways::ROAD`, serving
`crossings::BRIDGE`): the stone is ordered at the market over `months` (12),
the works pays its wages to the town's households, whose men are taken from
its farms and works while they build (`roads::take_menders`), and a works
short of cash calls on its parish for more. When its stone is all bought the
bridge stands, for good, and the way costs nothing more to cross. The year
it was built is kept (`CrossingState::built`, `bridges`). A bridge where
there was a ferry is news in its town.

**The roads of 1750 are inherited with some bridges.** Where a made road or
a turnpike of the first recorded year crosses water and its traffic passes
the parishes' own test, its bridge was built by earlier centuries and
stands; the rest ford. (Giving every made road's crossing a bridge baked the
warm-up's zigzags into stone.)

Nothing else of the prototype in [the roads probe](../probes/dx-roads.md)
is switched on: no turnpike trusts, no tolls, no parish roads; traffic wears
the roads as before.

**What the picture draws.** On a run that keeps its crossings, a road goes
round the meanders its straight line cuts and crosses a river once where
the simulation says; a bridge stands only where the simulation built one,
and from the year it was built; a ford is a bar of gravel just over the
water at the road's width with stepping stones beside it, and carts wade it;
a ferry is a timber landing on each bank and the ferry's flat boat. A run
that keeps no crossings (the old island, a recorded run made before this
rule) is drawn as before.

## What it does to the island

Islands 1 to 8 on the new land under the game's rules with six families, the
rule off and on, side by side
(`cargo run --release -p isostate-core --example probe_crossings -- --seeds 1-8`;
with the rule off the probe reads the meeting places and the straight ways
the picture drew before it). Mean over the islands, and the least and the
greatest.

**The measure Stella would recognise.** A road here is a chain of links from
one junction to the next; a crossing counts when the same road crosses the
same river again within 10 km along it, and a place is a road and a river
where that happens.

| | every way, crossings crossed again | places | the ways drawn (made roads and turnpikes), crossings crossed again | places |
|---|---|---|---|---|
| 1750, off | 116 [95 .. 130] | 41.9 | 97.9 [81 .. 117] | 31.5 |
| 1750, on | 21.4 [9 .. 41] | 10.1 | 30.5 [21 .. 49] | 12.6 |
| 1850, off | 107 | 39.1 | 86.4 | 30.8 |
| 1850, on | 16.0 | 7.8 | 29.4 | 13.1 |
| 1900, off | 90.9 [76 .. 119] | 32.8 | 86.6 [60 .. 104] | 31.8 |
| 1900, on | 12.1 [6 .. 26] | 6.0 | 20.8 [12 .. 29] | 9.5 |

So by 1900 an eighth of the repeated crossings are left on all the ways, and
a quarter on the ways the viewer draws; and every crossing left is a bridge,
a ford or a ferry. Most of what is left is a made road down a valley whose
cells' meeting places could not all be put on one bank (a river through the
very middle of a cell, or a cell whose only dry ground is on the far bank)
crossing a stream under 14 m twice, which at 0.4 a ford is cheaper than two
diagonal steps round (0.83 of a cell). A fixed charge of 0.3 a ford on top
moved nothing (island 7: 22 against 23 in 1900), so `ford_base` is 0.

**Crossings of water, and how they are crossed.** On the ways drawn:

| | crossings | bridges | fords | ferries |
|---|---|---|---|---|
| 1750, off | 225 [196 .. 244] | (every one drawn as a bridge, none paid for) | | |
| 1750, on | 142 [133 .. 157] | 71 | 56 | 15 |
| 1800, on | 166 | 102 | 52 | 12 |
| 1850, on | 252 | 148 | 92 | 12 |
| 1900, off | 473 [425 .. 505] | | | |
| 1900, on | 415 [375 .. 472] | 229 | 172 | 14 |

Every way, tracks too: 535 crossings in 1750 off and 413 on (53 bridges, 336
fords, 24 ferries), 799 and 716 in 1900 (325 bridges, 374 fords, 17 ferries).

**Bridges built, what they cost and who paid.** 100 an island stand in 1750
[85 .. 111]; the parishes build 560 more by 1900 [494 .. 638]: 70 in the
1750s, as the first rate meets the busiest fords, 15 to 18 a decade until
the 1810s, then 33 to 80 a decade from the 1820s to the 1860s as traffic
grows and the railways' feeder roads come, and 25 to 45 a decade after. They
span some 4,900 m of river an island in all. Every one was paid out of the
landlords' rate (the households' never had to: no parish's landlords were in
debt when it built). What they cost is small: 2.1 parts in 10,000 of the
1750s' output, 0.2 to 0.5 parts in the decades after, 0.23 in 10,000 of the
whole period's output.

**The economy** (`sweep --seeds 1-8 --rules game --families 6 --set
land.enabled=true --set crossings.enabled=false,true --standing`; 16 runs,
none failed). The standing check stays twelve of twelve inside. The carting
bill over output: 0.080 off and 0.082 on in 1750, 0.072 and 0.074 in 1800,
0.048 and 0.059 in 1850, 0.057 and 0.057 in 1900; carters 7.7 against 9.4
per cent of hands in 1850. What a wage buys, against 1750: 1.63 and 1.64 in
1800, 1.96 and 2.13 in 1850, 3.07 and 3.14 in 1900. The 1850 carting bill
moves most: the railway age comes later and thinner with the rule on
(stations in 6.4 towns in 1900 against 10.2), and that is the islands'
history moving, not a cost the rule adds, since in 1900 the bill is the same.
Under the game's rules on the old island, where the rule does nothing, a
sweep against main's gives 0 of 942 facts different.

## What worries me

- **The rule changes where the roads meet, not only what crossing costs.**
  The banks and the going round are geometry, and they do most of the work
  (island 7, 1900, ways drawn, crossings crossed again: 104 with the rule
  off; the price alone 105, the price with the banks 80, all three 22). Both are on whenever the rule is,
  and the viewer always uses the banks on the new land.
- **Small streams are still forded twice where going round is far.** That is
  honest at 0.4 a ford, but it is most of what is left of Stella's complaint.
- **A bridge is cheap.** 8 stone a metre, the prototype's number, makes the
  whole period's bridges 0.23 in 10,000 of output, and a parish bridges
  nearly every ford with a road's traffic. Dearer bridges would leave more
  fords, and more reason to keep to one bank.
- **The parish is the town of the link's first cell**, and one bridge works
  a parish at a time. A bridge between two districts is paid by one.
- **A ferry is drawn but not run**: its landings and its boat stand there
  from 1750 to the year a bridge replaces it, and no boat crosses.
- **The picture's other users read straight lines.** The ribbon, bridges,
  fords, and the carts follow the way round a meander; the houses and fields
  that keep off roads, the lanes, the carts' check (`carts::check`) and the
  railway's level crossings still read the straight line between the two
  middles. Some 5 per cent of the ways over water go round (74 to 99 an
  island of 22,600 ways between neighbouring cells, on islands 2, 3 and 7).
- **Saves.** A saved game replays its orders from the start, so the bridges
  are rebuilt on load; but a game saved before this rule replays on new
  meeting places and a new history.
