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

(numbers below)

## What worries me

(below)
