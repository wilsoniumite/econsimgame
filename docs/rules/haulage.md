# Haulage: the railway pays for its handling, and a road is worn by what runs on it

Not in the Python. A mend between the ways' third and fourth stages ([../designs/ways.md](../designs/ways.md),
parts 3, 6 and 8), so that the lorry takes the short hauls. Two rules, each behind a switch of its own,
both acting only from 1901:

| switch | what it does | code |
|---|---|---|
| `haulage.handling` | from the cost graph made at the close of 1901 the railway is a layer of its own over the squares, as a canal is: a load is on the ground or on a train, and pays the wharf's charge (0.4) to be put on a train and again to be taken off | `haulage.rs`, `transport::build_graph_trains`, `transport::dijkstra_layers`, `distances_layers` |
| `haulage.wear` | from 1901 a road is worn by the weight that goes over it by road, not by the trains, boats and ships that pass the same squares | `haulage::wear`, `sim::update_roads` |

Both are off under the port's rules and left out of its file, and both are **on under the game's rules**.
Nothing of either acts before 1901, so the played century stands by construction, and it was proved:
with both on, island 7 is the same island to the end of 1900 to the bit (`tests/haulage.rs`), the game's
golden test held its first era, and of the 7,824 facts named by a year to 1900 on islands 1 to 8 the only
ones that differ are reckoned over windows or whole runs that reach past 1900 (below).

## Why

The lorry landed on 26 November and lifted the road's share of the island's freight from 0.08 to 0.16 by
2000, while the railway still carried 0.83 in 2050. In the countries the island is drawn from, the lorry
had most inland freight within forty years of its coming. Its builder named two causes
([lorry.md](lorry.md), "What worries me"):

- **The cost graph charged a railway haul nothing at its ends.** A railed edge cost 0.07 a square for
  everybody, so the railway kept every haul along its line however short, and a load paid nothing to be
  put on a wagon and taken off. In a real country the station's handling is what lost the railway its
  short hauls: the goods yard's cranes and porters, the shunting of a wagon into a train and out again,
  and the cart to and from the yard at each end.
- **A road's wear counted the railway's freight beside it.** `update_roads` wore a square with all the
  weight that passed it by any way, so the trains' weight made the roads beside them turnpikes
  ([traffic.md](traffic.md), "Where they do not agree"), and from 1901 those are kept (`lorry.keep`).

## What it does

**The station** (`haulage.handling`). A square with track is two places from the graph made at the close
of 1901: the ground and the train (and a third, the water, where it has a canal). Between the ground and
the train is the station, charged each way; a train goes from a square with track to any of its eight
neighbours with track at the railway's 0.07 a step, as the port's railed edge did; the ground between two
squares with track costs what the ground costs, by its road, its slope and its fords. So:

- a load that rides the railway pays the station twice and the rail's rate between, and a cart that only
  crosses the line, or runs on the road beside it, pays nothing for the line;
- a load changing between a train and a boat pays for each, since it goes by the ground between;
- the charge is in the cost graph's units, so it is carting as every other square of a load's way is:
  money in each load's carting bill and in every delivered price, and carters' hands
  ([carting.md](carting.md)). Nobody collects it as a fare.

**The charge is the canal's wharf, 0.4, by the same rule** (`haulage::station`, which reads
`canals.wharf`): lifting a load from a cart to a wagon is the same work as lifting it to a boat. The
harbour's 0.6 (`transport::PORT_COST`), a ship's longer loading, was tried first: with it the station
was dearer than the wharf, and on island 7 the canals took back the short hauls the railway gave up (boat
freight in 1910 ten times main's). With one charge for both, a canal wins only where it is the cheaper way
a square, as before 1901. The charge was not fitted to any share; nothing was swept.

**What a load then costs.** In the cost graph's units, a load whose two ends are on the line, with a
lowland turnpike beside it, at the lorry's cost of an hour of carting over the wage of the year (kappa on
the mean of islands 1 to 8 with both rules on: 0.870 in 1930, 0.556 in 1970, 0.460 in 2010), each cell
turnpike / made road / rail with the station (rail without it, as before):

| | 10 km | 20 km | 40 km | 60 km | 100 km |
|---|---|---|---|---|---|
| 1930 | 1.31 / 1.96 / 1.15 (0.35) | 2.61 / 3.92 / 1.50 (0.70) | 5.22 / 7.83 / 2.20 (1.40) | 7.83 / 11.75 / 2.90 (2.10) | 13.05 / 19.57 / 4.30 (3.50) |
| 1970 | 0.83 / 1.25 / 1.15 (0.35) | 1.67 / 2.50 / 1.50 (0.70) | 3.34 / 5.00 / 2.20 (1.40) | 5.00 / 7.51 / 2.90 (2.10) | 8.34 / 12.51 / 4.30 (3.50) |
| 2010 | 0.69 / 1.04 / 1.15 (0.35) | 1.38 / 2.07 / 1.50 (0.70) | 2.76 / 4.14 / 2.20 (1.40) | 4.14 / 6.21 / 2.90 (2.10) | 6.90 / 10.35 / 4.30 (3.50) |

So by rail is cheaper than a turnpike from 8 km in 1930, 17 km in 1970 and 24 km in 2010 (than a made
road from 5, 9 and 12 km), for loads that start and end on the line. Most do not: a load that must be
carted to the station and from it pays those squares too, and that is why the road's share of the hauls
of 20 to 60 km rises as well (below). A boat on a canal costs 0.8 and 0.13 a square.

**Wear** (`haulage.wear`). A square is worn by what leaves it toward its market by road (a step the
traffic record calls cart, whatever its road), and a town's own square also by what arrives there by
road and by its own; trains, boats on rivers and canals, and ships wear no road. The trade between towns
the same. The weight passing each square (`State::traffic`), which the railway's fares are reckoned on,
still counts every way.

**The tree is still a tree over squares.** Where the cheapest way to a square ends on a train, its
predecessor in the town's tree is the square where the train was boarded, and the search says so
(`transport::Paths::by`, kept per town in `State::hl`). The squares a train passes are not on the tree,
so `haulage::wear` and the traffic record lay the train's weight on them along the cheapest way by
track (`haulage::Rails`), and both count every square as before: the two walks are still one walk
(`tests/haulage.rs`, `tests/traffic.rs`).

**The promoters** of new lines and canals reckon on the railway put back on the ground, edge for edge the
port's graph (`transport::Graph::flatten_train`, `ways::Net`): their sums are as they were. They do not
see the station (below).

## What the islands showed

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set haulage.handling=false,true --set haulage.wear=false,true --standing --compare MAIN --out ARMS
```

Run on 26 November 2026 on main d368b39 with the rule built in and its switches set by the sweep; 32
runs, none failed, money conserved in every month of every run. Both switches off is main to the digit
(of 4,381 facts, 0 differ). Means over islands 1 to 8.

### Freight by mode

The road's share of freight (weight carried times how far, the traffic record's), and the railway's:

| | 1900 | 1910 | 1930 | 1950 | 1970 | 1980 | 2000 | 2050 |
|---|---|---|---|---|---|---|---|---|
| road, both off (main) | 0.058 | 0.059 | 0.070 | 0.098 | 0.130 | 0.160 | 0.165 | 0.167 |
| road, both on | 0.058 | 0.149 | 0.198 | 0.322 | 0.470 | 0.539 | 0.600 | 0.596 |
| rail, both off | 0.937 | 0.937 | 0.926 | 0.898 | 0.864 | 0.831 | 0.825 | 0.823 |
| rail, both on | 0.937 | 0.825 | 0.768 | 0.645 | 0.498 | 0.427 | 0.367 | 0.372 |

Boats carry 0.033 from 1910 with both on (0.01 off): rivers have no wharf, so a short haul by river
competes with a station. Freight in millions of weight-km, both off / handling alone / wear alone / both
on: by road in 2050 3,192 / 8,385 / 3,110 / 9,174 (of it by lorry 2,946 / 7,608 / 2,847 / 8,367); by rail
15,680 / 5,730 / 15,550 / 5,733; by boat 193 / 835 / 131 / 482. The mean haul falls from 14.9 km to 11.8
in 2050, the trade between towns' stays at 86 to 88.

**By length of haul.** Of the freight of loads whose whole haul is under 20 km, 20 to 60 and over 60, the
share that went by road (`road_share_hauls_*`):

| | under 20 km, off / on | 20 to 60 km, off / on | over 60 km, off / on |
|---|---|---|---|
| 1900 | 0.143 / 0.143 | 0.048 / 0.048 | 0.001 / 0.001 |
| 1930 | 0.145 / 0.555 | 0.047 / 0.039 | 0.001 / 0.000 |
| 1970 | 0.220 / 0.886 | 0.068 / 0.182 | 0.005 / 0.003 |
| 2010 | 0.265 / 0.941 | 0.083 / 0.377 | 0.012 / 0.006 |

And the railway's own freight by the length of the stretch a load rode it without leaving the train
(`rail_freight_*`): under 20 km 0.29 in 1900, then 0.38, 0.48 and 0.50 in 1930, 1970 and 2010 with the
rules off, against 0.17, 0.05 and 0.02 with them on; over 60 km 0.39 in 1900, 0.27 in 1970 off and 0.69
on. **Where loads board**: with the rules off a load joins the line wherever its cart first meets the
track (a twentieth at a town's own square in 1970); with them on, at the station that serves it best,
which is half the time a town's own (0.47 in 1970, 0.50 in 2050).

### The four arms

| islands 1 to 8 | both off (main) | wear alone | handling alone | both on |
|---|---|---|---|---|
| the road's share of freight, 1950 / 1970 / 2000 | 0.098 / 0.130 / 0.165 | 0.093 / 0.131 / 0.164 | 0.311 / 0.448 / 0.563 | 0.322 / 0.470 / 0.600 |
| hauls under 20 km by road, 1970 | 0.220 | 0.226 | 0.858 | 0.886 |
| the towns' share of people, 1970 / 2050 | 0.596 / 0.570 | 0.593 / 0.563 | 0.573 / 0.546 | 0.579 / 0.552 |
| carters, of the hands, 1910 / 1950 / 2000 / 2050 | 0.079 / 0.069 / 0.051 / 0.051 | 0.079 / 0.073 / 0.051 / 0.054 | 0.119 / 0.093 / 0.060 / 0.063 | 0.118 / 0.094 / 0.059 / 0.064 |
| the hands a load takes halved (the lorry) | 1966 | 1966 | 1970 | 1969 |
| the railway's fares over its capital, 2050 | 0.568 | 0.511 | 0.309 | 0.405 |
| railway km, 2050; the last year it grew | 2,841; 2028 | 2,847; 2012 | 2,861; 2021 | 2,850; 2025 |
| made roads / turnpikes / tracks, 2050, km | 2,376 / 5,964 / 814 | 2,526 / 5,658 / 697 | 2,369 / 6,004 / 753 | 2,366 / 5,797 / 793 |
| the wage, per cent a year 1900 to 1970 | 1.26 | 1.25 | 1.14 | 1.16 |
| the wage, 2050 over 1970 | 1.18 [0.99 .. 1.27] | 1.14 | 1.11 | 1.16 [1.07 .. 1.26] |
| the land's share of hands, 2050 less 1900 | -0.107 | -0.100 | -0.102 | -0.107 |
| the linked real wage, 2050 over 1750 | 16.96 | 16.31 | 14.66 | 15.53 |
| goods a head, 2050 | 57.2 | 55.8 | 52.4 | 54.3 |
| people, 2050 | 1.178 M | 1.191 M | 1.320 M | 1.295 M |

**The twelve standing facts** are 12 of 12 inside in every arm, and to the digit: of the 7,824 facts
named by a year to 1900 on the eight islands, with both on against both off, 71 differ, all reckoned
over windows or whole runs that reach past 1900 (the linked wage's, what is made's and spending's
nine-year and decade means, the ages' births, deaths and life at 1900, the rails' dearest year, timber's
swing, the works near canals and the towns with a power station, which read the run's last state).

**The railways' books.** A line still pays: fares over capital in 2050 are 0.405 with both on against
0.568, the fares being the railway's rate on the weight over its squares; no line is closed, since no
rule closes one (the ways' principle: a closed way is closed by a decision), and the network is 2,850 km
in 2050 against 2,841. The station's charge is not in the fares: it is the carters' and porters' work,
paid to hands, not a return on the line's capital.

**The turnpikes.** From 1901 the made roads and turnpikes are kept whatever their traffic (`lorry.keep`),
so wear cannot unmake the turnpikes the trains made before 1900; what it stops is the trains making more.
With wear alone the island has 5,658 km of turnpike in 2050 against 5,964; with both on 5,797, since the
station puts more on the road, and that is the road's own traffic.

**What a run takes.** The month is untouched: the station is in the yearly cost graph, and nothing
monthly searches it. `cargo run --profile work -p isostate-core --example haulage` plays island 7 to 2050
with the rules on and off side by side on two threads and times them by processor time from 1901: a
month 16.16 ms on and 16.26 off (16.06 and 16.58 ms a million people), a year's end 178 ms on and 156
off, of which the year's cost graph and its searches (`land::update_transport`, `delta_fields`) are 50.1
ms and 45.9; the rest is the walks that lay the trains' weight on the squares they pass, and an island
with more people: some 3 seconds more for a run to 2050. In the sweeps the whole runs'
processor time was 119 and 136 seconds off and on in one, 93 and 81 in the other, on a machine shared
with other builds: noise larger than the rule.

### Who gains and who loses

- **The roads and the carters.** The road carries 0.60 of the freight from 2000 against 0.17, and passes
  the railway about 1972 on the mean (by 1970 on four islands of eight, by 2000 on all). Carters are 0.118
  of the hands in 1910 against 0.079, and 0.064 in 2050 against 0.051: the short hauls go by cart before
  the lorry comes.
- **The railway** loses its short hauls (its stretches under 20 km from half its freight to a fiftieth)
  and keeps the long: two thirds of what it carries rides over 60 km. Its fares over capital fall by
  three tenths.
- **Everyone pays a little for carting**: a load that used to ride the line for nothing at its ends now
  pays the station or goes by road. The wage grows 1.16 per cent a year from 1900 to 1970 against 1.26,
  the linked real wage in 2050 is 15.5 times 1750's against 17.0, goods a head 54 against 57, and the
  towns hold 0.552 of the people in 2050 against 0.570. The island has more people (1.30 M against 1.18
  M): the dearer carting holds them in the countryside's market towns.

## The decision

**Both land on.** The pass, measured on islands 1 to 8:

| the pass | measured | |
|---|---|---|
| with handling on, hauls under 20 km mostly by road by 1970 | 0.886 of their freight by road in 1970 (0.858 with handling alone; 0.220 off) | met |
| the road's share of freight rises through the century | 0.058, 0.149, 0.198, 0.322, 0.470, 0.600 in 1900, 1910, 1930, 1950, 1970, 2000; level from 2000 (0.596 in 2050) | met |
| whether it passes the railway's, and when | yes, about 1972 on the mean (0.470 against 0.498 in 1970, 0.539 against 0.427 in 1980) | said |
| no fact to 1900 moves | the economy, the books and the news to 1900 to the digit | met |
| the twelve standing facts in their bands | 12 of 12 inside | met |
| the three growth asks met with what lands on | 1.16 per cent a year 1900 to 1970; 2050 over 1970 1.16; the land's share 0.107 lower | met |
| no run fails or loses money | 32 runs to 2050, none failed | met |
| the month is no slower | 16.16 ms a month on, 16.26 off (island 7, 1901 to 2050, processor time); a year's end 178 ms against 156 | met |

Wear alone moves little (300 km fewer turnpikes in 2050) and lands with the station because it is the
honest count: a road is worn by its own traffic. `--set haulage.handling=false --set haulage.wear=false`
is the island without them.

## What worries me

- **The turn of 1901 is a step.** The charge comes in whole at the close of 1901, so the carters jump from
  0.079 of the hands to 0.118 between 1900 and 1910, and the road's share from 0.06 to 0.15. The station
  always cost something; only the switch's year makes it new.
- **The promoters do not see the station.** New lines and canals are reckoned on the port's graph, so a
  branch promoted after 1901 is valued on short hauls it will not carry. Few lines are built after 1900
  (2,692 km in 1900, 2,850 in 2050).
- **Rivers have no wharf.** A navigable river is still in the ground layer at 0.3 a square with nothing
  to get on or off, so after 1901 a short haul by river beats a station and boats carry 0.033 of the
  freight against 0.01. By the same rule a river boat should pay the wharf; it needs a layer for the
  rivers.
- **One station everywhere.** Every square with track is a station, a goods yard on every 2 km of line.
  Real goods yards were fewer and at towns; with fewer stations the road would take still more.
- **The station is not cheapened by machines.** Cranes, the fork lift and the container cut the handling
  after 1950; here it is the wharf's 0.4 to 2050, while the lorry's cost falls.
- **The traffic record's crossings** (`crossings::link_traffic`) no longer count a train's weight over a
  river among the fords' traffic, since the train's squares are not on the tree: a parish bridges for its
  carts alone, which is the honest count, but it came with the station rather than with wear.
