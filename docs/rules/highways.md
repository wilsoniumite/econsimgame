# The state's ways: trunk roads between the large towns, and a bypass where the through traffic pays

Added on 26 November 2026. Not in the Python. `crates/isostate-core/src/highways.rs`, switched by
`highways.trunk` and `highways.bypass`: off under the port's rules, on under the game's, and acting only after
1908. The fourth stage of the ways' design ([designs/ways.md](../designs/ways.md), part 8): after 1900 the
builder of main roads is the state, and the state counts everybody's time.

## What it does

**The treasury buys road-making.** The island's state had a purse with a rate on pay, schools, pensions and a
university, and bought nothing on the ground. Its works are now road works, as a parish's bridge is
([crossings.md](crossings.md)): the stone is bought at the market of the town whose district holds the squares
(one works a town along the way), and two shillings of wages on every shilling of stone are paid to that town's
men, hands taken from its farms and works as the parishes' menders are. So road-making is work in the towns, and
the books count it as they count every project's outlay. After each month the treasury pays each works its
coming month's bill out of what it holds above one month of its programmes' spending: the teachers and pensions
are paid first, and there is no borrowing. At each year's end the purse adds next year's bill for the works in
hand to next year's spending, so the rate on pay rises to pay for them, and the year's works are counted in the
months the treasury holds ([state.md](state.md)). What is put in hand at once is at most a hundredth of last
year's pay (`WORKS_SHARE`); a way that costs more than that alone is put in hand when nothing else is, and takes
as many years.

**Trunk roads** (`highways.trunk`, from the end of 1909, the year of Britain's Road Board). Each year the state
takes the cheapest way over the ground (no trains, no boats) in the year's cost graph between every two large
towns (those holding a twenty-fifth of all townsfolk or more, the ten largest at most), and prices bettering the
part of it that is not a trunk road yet:

- *what it costs*: 60 men's years at the town's wage a square of lowland (hills and mountains their carting
  cost's multiple), a third of it stone and two thirds wages (`WORK_YEARS`), and a bridge, at the parishes'
  stone a metre, on every link of it that fords or ferries;
- *what it saves a year*: each square's weight by road that year (what wore the roads, `sim::update_roads`) times
  what a trunk road takes off the square's cost, and the fords' or the ferry's charge on the links it bridges.

A route is reckoned whole, and built where it saves a twentieth of its cost a year or more (`HURDLE`), the best
first, as far as the year's budget goes. Reckoned square by square the trunk roads would be stubs round each large
town, where its own district's carts are; reckoned whole they run between the towns.

A trunk road is bettered in place, on the old road's squares, and its fords are bridged with it, so **no trunk
road fords a river**. It runs into the two towns' own middles, as an arterial road, but **not through a third
town's ground**: there the old road is that town's high street, with houses along it, and it is left as it is.

A trunk road's square costs a turnpike's (0.3 on lowland, cheapened by the lorry as every made surface is,
[lorry.md](lorry.md)), less a third of that on the share of the hauling the lorries do (`trunk_mult`; a square at
0.2 where every load goes by motor). A horse goes no faster on a surfaced road; a motor does. So before the lorry
a trunk road pays only where it betters a track or a made road to a turnpike's standard, and after it on every
square.

**Bypasses** (`highways.bypass`, from the end of 1920). For each town that a trunk road, open all its length,
passes through, the state prices two cures for the town's stretch of it:

- *a way round*, over the cheapest squares outside every town's ground in the year's cost graph, from the trunk
  road where it meets the town's ground to where it leaves it, at its men's years and the land it takes (a
  fiftieth of each square's value, `LAND_TAKEN`). It saves the through traffic (the trade between two other
  towns, by road, over the town's squares of the trunk road) what crossing the town costs over going round on a
  trunk road. Being the cheapest way round, it lies on the side the traffic passes, as one arc.
- *widening the high street* into a trunk road where it runs, at three twentieths of the value of the houses
  along it (`FRONTAGE`) and half the men's years. It saves every load over those squares what a trunk road takes
  off a square.

It builds the better of the two where it saves a twentieth of its cost a year. No town gets one without the
traffic, and a ring comes only where two arcs happen to meet. Both are news when they open ("Tarnbridge is
bypassed", "Oakbury's high street is widened").

**What is watched**, whatever the switches, while the traffic record is kept: the through traffic of each square
and each town (what passes a town's ground by road on its way between two other towns), and the treasury's year.

**The record** marks a trunk road's square with 64 and a bypass's with 128 in the road byte (`trans`), whose two
top bits were free. The viewer draws a link between two trunk squares in tarmac and a bypass's in paler tarmac,
and the Traffic view lays a dark line down a trunk road's ribbon and a red one down a bypass's. Their look is a
later piece's.

## Its numbers

| name | game | what it is |
|---|---|---|
| `highways.trunk` | on | trunk roads between the large towns |
| `highways.bypass` | on | a bypass, or a high street widened, where the through traffic pays |
| `TRUNK_FROM`, `BYPASS_FROM` | 1909, 1920 | the first year at whose end each is put in hand |
| `TRUNK_MULT` | 0.2 | a trunk road's square where every load goes by motor, against a turnpike's 0.3 |
| `WORK_YEARS` | 60 | men's years a square of lowland, a third stone and two thirds wages |
| `WAGES` | 2 | shillings of wages on every shilling of stone |
| `HURDLE` | 0.05 | what a way must save a year over its cost |
| `WORKS_SHARE` | 0.01 | the most of last year's pay put in hand at once |
| `LARGE`, `LARGE_MOST` | 0.04, 10 | a large town's share of all townsfolk, and how many at most |
| `FRONTAGE` | 0.15 | the share of the houses' value along a high street widening pulls down |
| `LAND_TAKEN` | 0.02 | the share of a square's land a road takes |

The constants are in `highways.rs`; the two switches are the Rules tab's chips, in plain words.

## What eight islands show

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set highways.trunk=false,true --set highways.bypass=false,true --standing --out arms
cargo run --profile work -p isostate-core --example highways -- 7 2050
```

32 runs, none failed; the standing check 12 of 12 inside in every arm; the three growth asks met in every arm;
no year cut and no month short in the treasury, which holds 1.66 months of its spending at its least. The mean of
islands 1 to 8, both switches on (off in brackets where it differs):

| | 1920 | 1930 | 1950 | 1970 | 2000 | 2050 |
|---|---|---|---|---|---|---|
| trunk road, km | 18 | 49 | 223 | 303 | 346 | 365 |
| the works over the island's income, ten years to the mark, parts in 10,000 | 1.0 | 2.0 | 4.1 | 1.6 | 0.0 | 0.1 |
| hands at the works over all hands, ten years to the mark, parts in 10,000 | | 1.9 | 5.0 | 1.9 | 0.1 | |
| the works over the treasury's spending, ten years to the mark | | 0.005 | 0.008 | 0.002 | 0.000 | |
| tax over income | | 0.046 | 0.053 (0.052) | 0.067 (0.066) | 0.088 (0.087) | 0.100 (0.099) |
| through traffic by road, millions of weight-km | | 0 | 0.02 (0.15) | 5.4 (0.58) | 30.5 (1.0) | 30.7 (0.5) |
| road's share of freight | | | 0.34 (0.32) | 0.52 (0.47) | 0.68 (0.60) | 0.70 (0.60) |
| towns' share of the people | | 0.502 | 0.547 (0.547) | 0.578 (0.579) | 0.587 (0.589) | 0.548 (0.552) |

- **Trunk roads**: 303 km an island by 1970 (200 to 396 by island), the first put in hand in 1930 on the mean
  (1912 on island 7, where a made road was bettered to a turnpike's standard before the lorry), most between 1935
  and 1960 as the lorry takes the hauling. After 1980 few more pay: the ways between the large towns are done.
  No link of a trunk road between two of its squares fords or ferries, on any island.
- **Bypasses and high streets**: two bypasses on one island (island 5, round Tarnbridge, in 1980, on two trunk
  roads), and three high streets widened (islands 1, 6 and 7; island 7's is Oakbury's in 1957, the town with most
  through traffic every decade from 1960). The towns cured stood second by through traffic on the mean when the
  sum was made, none lower than fourth. Through traffic is small beside all freight: the trade between towns by
  road passes few third towns' ground, and the three towns with most carry 0.96 to 0.97 of it. With the trunk
  roads it grows tenfold, because a trunk road draws the trade between towns onto the road.
- **Freight**: the road's share rises by 0.05 in 1970 and 0.08 to 0.10 from 2000, the railway's falls as much: the
  lorry's saving is fullest on a trunk road.
- **The treasury's works** are small: at their most four parts in ten thousand of the island's income, in the
  1940s, and under a hundredth of the treasury's spending. Britain's roads took about a hundredth of its output a
  year between the wars, though nearly all of it was the upkeep of all its roads, which here costs nobody
  anything (part of what worries me, below).
- **The towns' share of the people** moves by less than a hundredth either way: the road-making is work in towns,
  but at two to five hands in ten thousand it is too little to hold a town's hands.
- **The played century**: nothing of either rule acts before the end of 1909, and `tests/highways.rs` holds island
  7 the same island to 1909 with the switches on and off, series by series to the bit. The gate's sweep to 1900
  against the merged reference: of 1,210 facts, 3 differ (`state_seat_*_1900`, +0.0, as before this rule).
- **What it takes**: on island 7 to 2050 with both on, a month 16.5 to 19.0 ms and a year's end 191 to 219 ms of
  processor time (the example above, two runs on a busy machine), against 15.8 ms and 177 ms with both off
  (`-- 7 2050 --off`, one run); the year's end pays for ten searches of the ground a year and the walk of the
  through traffic, and a month only the treasury's payments to the works.

## What is not built

- **Lakes and tideways** (the design's part 9): the state's bridge at a tideway's head, or a ferry's charge, in
  the crossings' table. Not built; the causeways stay a named fault.
- **Upkeep**: a trunk road, like every road here, costs nothing to keep.
- **The land and the houses** are reckoned in the bypass's sum and not bought: no money passes for them.
- **The motorway** and **the car**: the design's later builders. Through traffic here is goods alone.
- **The look** of a trunk road and a bypass: they are drawn as roads in tarmac.

## What worries me

- **Roads are cheap here.** The played century's roads and bridges cost a hundredth of what they cost in
  history (the crossings' page: a parish bridges nearly every ford for 0.23 parts in 10,000 of output). The
  state's are priced in men's years to make its share of the purse honest, so a trunk road is far dearer than a
  parish's made road over the same square, and even so the works come to a few parts in ten thousand of income.
- **Through traffic is the trade between towns only.** A district's own carts never pass a third town, so the
  sum for a bypass reads less traffic than a town's high street carried; the car's journeys, when they come,
  will add to it.
- **The towns that get cured are small.** The largest towns are the trunk roads' ends and are never bypassed;
  a market town on the way between two of them is. That is history's pattern, but the first city's own through
  traffic (four to seven millions of weight-km on island 7) is never priced.
