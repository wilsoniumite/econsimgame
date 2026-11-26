# Traffic: what each way carries

What each way between two squares carried each year, by cart, boat, rail
and sea, kept in the record and seen on the map; and which towns trade most
with which. Not in the Python. It only watches: nothing the simulation does
reads it, and with it on every month and every year is played as with it
off. The switch is `traffic.enabled`, on under the game's rules and off
under the port's; the code is `traffic.rs` in the core, and `traffic.rs`,
`overlay.rs` (the Traffic view) and `carts.rs` in the viewer. It is the first
half of stage 2 of the ways' design ([designs/ways.md](../designs/ways.md),
part 3); the second half, that a made road no longer falls back once paid
for, is not built.

Stella's note of 19 November asked, among other things: "Can I see town
markets, largest trading partners or flows or things like that?" And the
ways' principle is that traffic makes ways, yet until now nothing in the
record said what a way carries: the picture could not draw a busy road busy,
the later rules (a bypass, a motorway) had nothing to read, and she could
not see what moves on her island.

## What is counted

Each year `sim::update_roads` lays the year's carting on the cheapest paths
the towns' searches found: each district's goods to its market along its
town's tree of paths (`transport::tree_traffic`), and the trade between
towns along the path from the seller to the buyer (`transport::pair_traffic`).
It adds the weight up square by square to wear the roads, and then throws it
away. This rule walks the same trees with the same weights once more, right
after it (`traffic::year`, called from `sim::yearly`), and adds each step of
a path to the *link* it crosses: a square's way to one of its four
neighbours east, south, south-east and south-west (`traffic::SLOTS`; the other
four are its neighbours' links), the same slots the viewer's roads and the
crossings' table use. A square cannot say which way through it a load went
(a crossroads carries four), and the picture draws links, so the count is by
link, as the ways' design argued.

Each step is put down to the way the cost graph the paths were found on
sends it:

| mode | where |
|---|---|
| rail | the edge is railed: both squares are land with track (`Graph::railed`) |
| sea | either square is sea: along the coast, or from a harbour onto the water |
| boat | both squares are navigable river and the edge costs the river's rate; or the step is by canal, which the tree records as one step from where the boat was loaded to where it was left, and which is followed square by square along the canal's own links (`canals::Water::route`) |
| cart | everything else: over the ground, on whatever road the square has |

Where a railway runs, every load over that link goes by rail: the graph
prices the railed edge at the rail's cost for everybody, so a road beside a
railway carries nothing by cart in the simulation. That is why the busiest
ways after 1850 are all railways, and why a link has one mode in practice;
the record keeps the one that carried most of its weight.

What is kept of a year (`State::tf`): the weight over each link both ways
together, the way most of it went, the weight that left each square (the
same as `update_roads`' own count, kept for the test that the two walks are
one walk), and for the facts the weight times the squares it was carried by
each mode, the weight sent (each district's goods to its market, and the
trade between towns apart), and the squares of each road level, railway and
canal.

**People's journeys are not counted.** The design page asked for each square
of town ground to send its working townsfolk to the market place and back
along its town's tree, from 1901, into a second layer. It is cheap to reckon
(one more walk of a tree the simulation already makes, about the cost of the
goods' walk), but nothing reads it yet: the tram, the car and the car park
that would are stages 5 and later of the ways' design. Left for the stage
that has a use for it; what it would take is one more `[f64; 4]` a link in
`traffic::year` filled from `st.urban_res` and the towns' share at work, and
one more layer of four bytes a square in the record (6.1 MB raw to 1900, 12.1
MB to 2050, before gzip).

**The partners are the books'.** A town's largest trading partners need no
new bytes: the books of account (`accounts.enabled`, on under the game's
rules) keep every flow between two towns, good by good, with its quantity,
its worth at the seller's price and the carting paid on it (`accounts::Flow`,
`acct_trade_*` in `run.bin`). `traffic::pairs` sums one year's flows a pair
of towns at a time: the worth, the weight (each good's quantity times its
weight) and the three goods worth most. A live game hands the year's pairs
over summed (`LiveYear::pairs`, `traffic::year_pairs`); the viewer sums a
recording's from its books' tables with the same function.

## In the record

Two layers, written only in a run that keeps the traffic, so the port's
record is untouched:

| array | type, shape | what |
|---|---|---|
| `way_goods` | uint8, `[year, square, 4]` | the weight over each link in the year, on a scale by ratio: 0 below one unit of weight, else 1 to 255 for 1 to a thousand million in 254 steps of 8.5 per cent (`traffic::goods_byte`, `byte_goods`) |
| `way_mode` | uint8, `[year, square]` | the way most of each link's weight went, two bits a link from the lowest (`traffic::mode_of`): 0 cart, 1 boat, 2 rail, 3 sea |

The most any link carried on island 7 to 2050 is 227 million (Dunheath's
railway in 2050), well inside the byte's top. A live game hands over the same
bytes each year (`LiveYear::traffic`, 50 kB a year).

What it adds, island 7 under the game's rules with six families (`run.bin`
written by `isostate-core run --seed 7 --rules game --families 6 --end-year
2050`, the layers cut out of it and gzipped at level 6):

| | raw | gzipped |
|---|---|---|
| `run.bin` to 2050, with the two layers | 247.7 MB | 94.1 MB |
| `way_goods` to 2050 / to 1900 | 12.14 MB / 6.09 MB | 2.96 MB / 1.46 MB |
| `way_mode` to 2050 / to 1900 | 3.03 MB / 1.52 MB | 0.06 MB / 0.03 MB |
| the two, as a share of the run | 6.1 per cent | 3.2 per cent |

## What it costs

`examples/traffic_probe.rs` plays island 7 under the game's rules with six
families to 2050 twice, side by side on two threads, with the rule on and
with it off, and on the run with it off times the rule's walk by itself each
year:

    cargo run --release -p isostate-core --example traffic_probe -- --end-year 2050 --years 1800,1850,1900,1950,2000,2050 --top 10

- Every monthly series the same to the bit, the news the same, the roads and
  their square traffic the same: the rule changes nothing played.
- The weight leaving each land square as this rule counts it equals
  `update_roads`' own, to the bit (worst relative difference 0): the two
  walks are one walk.
- The walk by itself, timed each year: 6.2 ms on the mean (9.7 at the
  longest) on one night's machine, 8.5 ms (13.1) on a busier hour, which is
  2.5 per cent of a year of the simulation to 2050 in both. It walks the same
  trees as `update_roads` and costs about what that does.
- The whole run with it and without, side by side on two threads, to 1900,
  three times (`TRAFFIC_PROBE_UNTIMED=1`, which leaves the walk out of the run
  that has the rule off): 34.7 s against 34.0, 32.7 against 32.1, 28.1
  against 30.4. The difference is inside the machine's noise; the walk's own
  2.5 per cent is the honest figure.
- The viewer: the ribbons are laid in 2 to 10 ms (dev profile) when the year,
  the scale or the zoom's step changes. With the view open at Fast on island
  7 from 1840 (`ISOSTATE_FRAMES_PLAN`, 773 frames), the frames' work had a
  middle of 13.6 ms, a 99th of 31.7 and a longest of 39.5, against 13.4, 24.2
  and 29.5 on the plain map over the same years; three frames over 34 ms one
  after another, 0.1 s in all. The view does not wait.

## Measured

Islands 1 to 8 under the game's rules with six families; the mean of the
eight and the least and greatest. Freight is the weight carried times the
kilometres it went (a square is 2 km), in millions; the mean haul is all the
weight-kilometres over all the weight sent, and the trade's the same for the
trade between towns alone; the roads are kilometres of land squares at each
level as the year's wear left them (`isostate-core sweep --seeds 1-8 --rules
game --families 6`, facts `freight_*`, `haul_km_*`, `trade_haul_km_*`,
`track_km_*`, `made_road_km_*`, `turnpike_km_*`).

| | cart | boat | rail | sea | mean haul, km | the trade's, km | track km | made road km | turnpike km |
|---|---|---|---|---|---|---|---|---|---|
| 1750 | 88 [80 .. 100] | 47 [34 .. 69] | 0 | 18 [9 .. 25] | 20.4 | 99 | 3,904 | 1,238 | 835 |
| 1800 | 129 [94 .. 158] | 277 [203 .. 350] | 0 | 6.6 | 20.4 | 102 | 4,404 | 1,352 | 1,144 |
| 1850 | 126 [85 .. 215] | 82 [13 .. 479] | 1,163 [261 .. 1,451] | 0.4 | 22.7 | 98 | 4,545 | 1,719 | 2,160 |
| 1900 | 260 [238 .. 290] | 19 [9 .. 45] | 4,183 [3,771 .. 4,559] | 0.3 | 17.8 [15.1 .. 22.3] | 86 | 3,546 | 2,886 | 3,368 |
| 1950 | 1,099 [919 .. 1,370] | 63 | 12,659 [11,447 .. 15,057] | 0.6 | 15.7 | 88 | 2,688 | 2,735 | 4,711 |
| 2000 | 1,574 [959 .. 2,108] | 66 | 15,974 [12,771 .. 19,258] | 0.2 | 15.9 | 88 | 2,062 | 1,680 | 4,526 |
| 2050 | 1,970 [1,315 .. 2,791] | 83 [53 .. 153] | 19,693 [16,339 .. 24,266] | 0.1 | 16.4 [14.9 .. 18.3] | 90 | 1,911 | 1,325 [900 .. 2,282] | 4,420 [3,910 .. 5,206] |

(`isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050`;
the kilometres of road are those the ways' page measured with a probe of its
own, to the kilometre, which says the two count the same squares.)

Read:

- **The canal's half-century, then the railway's.** In 1800 boats carry two
  thirds of the weight-kilometres (277 million on the mean, against 129 by
  cart); by 1850 the railway carries five sixths, and from 1900 nine tenths
  and more, to 2050. Nothing changes after 1900 but the amount: freight grows
  four and a half times from 1900 to 2050, and the cart's share stays near a
  tenth, because there is no lorry.
- **The haul is short and gets shorter.** Most of what is carted is a
  district's produce going a few squares to its market, so the mean haul is
  about 20 km to 1850 and 16 km after, as towns fill their districts. The
  trade between towns goes 86 to 102 km on the mean, and hardly changes in
  three centuries: the island is no bigger, and the railway made the far
  towns cheaper without making them nearer.
- **The sea is a coaster's trade of the first decades only**: 18 million
  weight-kilometres in 1750, under one by 1850, as canals and railways open.

## Where the busiest ways are

Island 7 (`traffic_probe`, 1900 and 2050, `probe-7-2050*.txt` in the night's
notes):

- **The ten busiest links in 1900 and in 2050 are all railways**, every one
  with a turnpike on both squares too: in 1900 the line through Dunheath (43,
  65 to 44, 65), 29.9 million a year, then Glenwick's four (21.0 million),
  Dunheath's westward pair, Melness's two and Granness's; in 2050 Dunheath's
  six links (216 million at the most), Mallowbrook's, Garrigg's two and
  Granness's. Nothing but a railway, and in Dunheath a canal beside it.
- **The busiest by cart** are the turnpikes that run from a town's market
  square to its station: in 1900 2.5 million a year at the most (Dunheath,
  Glenwick, Melness, Garrigg); in 2050 15.8 million (Granness, Dunheath,
  Melness, Mallowbrook). Every one of the fifty busiest links by cart, in both
  years, runs between two turnpike squares.
- **Agreement with the roads' wear.** Of the fifty squares `update_roads`
  counts busiest, 49 (1900) and 50 (2050) are an end of one of the hundred
  busiest links. Across all 5,800 land links that carry anything, a link's
  weight ranks with the lesser road level of its two squares at 0.93
  (Spearman), and with its squares' own traffic at 0.65 (1900) and 0.78
  (2050): lower, since a square's count is every link through it.
- **Where they do not agree: the railway wears the roads.** `update_roads`
  wears a square with all the weight that passes it, by any mode, so a square
  a railway runs through is worn by the train's freight and made a turnpike.
  Of island 7's 1,743 turnpike squares in 1900, 1,322 carry less by cart than
  the turnpike's threshold of 40,000 a year, 1,020 of them squares with a
  railway; in 2050, 1,120 of 2,092 (814 with a railway). (A turnpike falls
  back only after six years under a fifth of its threshold, so some of the
  rest are kept by carting between 8,000 and 40,000.) The thousand with a
  railway are turnpikes the trains' weight made or keeps: much of the "main
  road everywhere and therefore nowhere" the ways' page found. The rule is the
  port's and is not changed here; the traffic record is what a mend would
  read (wear a road with its links' weight by cart alone). Mended from 1901
  under `haulage.wear` ([haulage.md](haulage.md)).

## The viewer

The Traffic map view draws each link as a ribbon as wide as what it carried,
in the ink of the way it went, on the three ranges every view has; a click
on a square says what its two busiest ways carried. A town's card ends with
"Trades most with": its five largest partners, what goes each way, the
weight and the goods that lead. Carters' rounds have one, two or three carts
by the carting along them. The viewer's README, "Map views", says how.

## What worries me

- **Rail takes everything it touches.** A road beside a railway carries
  nothing by cart, because the cost graph sends every load over a railed edge
  by rail. A real railway left the short hauls and the parcels to the carts;
  the simulation has no lorry and no short-haul penalty for rail. So the
  picture of 1900 to 2050 is black lines with brown feeders, and the ways'
  later builders will read no road traffic where the railway runs. The lorry
  (the ways' design, stage 3) is what changes that, with the station's charge
  from 1901 ([haulage.md](haulage.md)): with it the railway is a layer of its
  own, and a road beside it carries what goes by road.
- **The canal's steps are put on the canal's links, its wharves not.** A
  load put on a boat pays the wharf in the cost graph and goes on the canal's
  own links; the record has it on those links by boat. The ground step to the
  wharf is cart, as it should be.
- **The partners are at the seller's prices.** A town's card adds what it
  sold at its own prices to what it bought at the other town's. Over a long
  way the two sides of a pair are in different money, which is right for the
  books but makes "worth both ways" a sum of two prices.
