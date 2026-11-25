# The ways: roads, streets and highways, from who builds them and why

25 November 2026. A design, and part 5 is written to be built from tonight. Nothing on this page is built yet; the measuring behind part 1 was done with a probe kept out of the repository, and every command is in the night's notes.

Stella asked on 19 November: "I think roads will need some work? Like, not every city has ringroads, and they aren't perfect rings. A while ago, we talked about how roads crossing meandering rivers 4 times in a row doesn't make much sense, nobody would build that many bridges. I feel like the roads system will need some work. What governs roads in the countryside, a city, a highway, grids, terrain, year they were built, policy, that sort of thing? How can we make the most honest, natural, emergent road system?" The first stage was built that day ([rules/crossings.md](../rules/crossings.md)). On 23 November, shown the first city of 2050, she found it "a bit too chaotic" and asked to leave it until the road works lined up had landed. This page is those works.

Her questions, and the part of this page that answers each:

| her question | answered in |
|---|---|
| the countryside | 2 (traffic wears, land shapes), 4 (the parish, the trust, the county's bridge) |
| a city | 5 (streets inside towns), 4 (the town's board, the owner, the tramway, the car park) |
| a highway | 4 (the state's trunk road, the bypass, the motorway), 6 (the motor age) |
| grids | 2 (what the principle rules out), 5 (one owner at one time; dear ground) |
| terrain | 2 (land decides shape: slope, water, and what stands on it) |
| the year they were built | 4 (each builder has its era), 5 (a street's line outlives its buildings; width and block by era) |
| policy | 4 (who pays, what a hand can turn), 7 (the hand) |
| ring roads | 2 and 4 (a ring is bypass arcs that happen to join; an inner ring only by statute) |
| bridges | 1 (what stands), 2 (one bank), 4 (the county's bridge; the state bridges the fords its trunk roads still wade) |

## 1. What is so today

**The kinds of way the simulation has.** A square (a cell, 2 km of the map) has a road level of 0 to 3: none, a track, a made road, a turnpike. Traffic alone makes them: each year `sim::update_roads` lays the year's carting on the cheapest paths (each district's goods to its market, the trade between towns) and a square's level is the number of thresholds its weight passes (1,500, 9,000 and 40,000 a year); a level falls by one after six quiet years. Nobody pays and nothing is used up. A railway and a canal are works: promoters, subscribers, a cost bought month by month and a year opened ([rules/railways.md](../rules/railways.md), [rules/canals.md](../rules/canals.md)). Crossing water is a work since 19 November: a ford or a ferry costs carting time, and a parish builds a bridge where the fords' charge on last year's traffic pays 5 per cent on its stone and wages; a bridge stands for good ([rules/crossings.md](../rules/crossings.md)). The prototype of parish roads and turnpike trusts with tolls is in the code and off under both sets of rules (`roads.enabled`, [probes/dx-roads.md](../probes/dx-roads.md)). The cost graph prices a lowland square at 1, 0.7, 0.45 and 0.3 by road level, a navigable river at 0.3 and a railway at 0.07.

**What the picture draws.** A made road or a turnpike between two neighbouring squares is a ribbon between their meeting places (0.022 and 0.034 of a square wide), round the meanders its line cuts and round a lake by the shore; a track is drawn only as a town's lane or a farm's. A bridge, a ford or a ferry stands where the simulation has one. Inside a town every street is laid from the ways between squares: the way itself, three lanes either side of it at 0.13, 0.26 and 0.39 of a square, and cross streets every 0.17, each lane giving way where another way is nearer (the viewer's `scatter/towns/kept.rs`). A square taken after 1900 and filled whole is drawn built up but for its park, and its closes that no street reaches are built over one by one (`lots::infill`).

**What is recorded.** The `trans` byte a square a year: the road level, rail, rail building, canal, canal digging. On the new land the crossings' table and each bridge's year, in the run's header. No traffic: the weight `update_roads` lays on the squares is in no layer, and nothing that moves on the map follows it.

**Measured.** Islands 1 to 8 under the game's rules with six families, played to 2050 on the new land; the mean of the eight and, where given, the least and greatest. A square with a railway is counted under railway for freight but under its road level for road; a kilometre is half a square.

| | track km | made road km | turnpike km | railway km | canal km | road links drawn | bridges standing | drawn roads over water: bridge / ford / ferry |
|---|---|---|---|---|---|---|---|---|
| 1800 | 4,404 | 1,352 | 1,144 [998 .. 1,414] | 0 | 526 | 1,354 | 250 [231 .. 285] | 144 / 113 / 16 |
| 1900 | 3,546 | 2,886 | 3,368 [3,216 .. 3,570] | 2,694 | 659 | 4,743 | 672 [600 .. 739] | 401 / 444 / 31 |
| 1950 | 2,688 | 2,735 | 4,711 [4,522 .. 5,144] | 2,815 | 661 | 6,119 | 700 | 468 / 652 / 33 |
| 2000 | 2,062 | 1,680 | 4,526 | 2,834 | 681 | 4,982 | 716 | 380 / 541 / 28 |
| 2050 | 1,911 | 1,325 | 4,420 [3,910 .. 5,206] | 2,839 | 698 | 4,587 | 727 [680 .. 804] | 352 / 483 / 31 |

Freight, as the weight passing each land square in a year, by the way the square has (shares of the whole), and the whole in weight-squares:

| | cart, no road | track | made road | turnpike | boat, canal or navigable river | rail | whole, millions | a head |
|---|---|---|---|---|---|---|---|---|
| 1800 | 0.015 | 0.059 | 0.091 | 0.419 | 0.415 | 0 | 88 | 433 |
| 1900 | 0.000 | 0.003 | 0.007 | 0.038 | 0.011 | 0.941 | 2,077 | 3,053 |
| 1950 | 0 | 0 | 0.002 | 0.065 | 0.013 | 0.920 | 6,552 | 6,765 |
| 2000 | 0 | 0 | 0.001 | 0.075 | 0.012 | 0.912 | 8,332 | 7,825 |
| 2050 | 0 | 0 | 0 | 0.076 | 0.012 | 0.911 | 10,240 | 8,798 |

The first city (the town with most townsfolk that year):

| | townsfolk | squares of town ground | filled whole | filled, with no road drawn through them | all its squares with no road drawn through them |
|---|---|---|---|---|---|
| 1800 | 14,010 | 22.8 | 0 | 0 | 6.9 |
| 1900 | 51,180 | 57.0 | 0 | 0 | 17.0 |
| 1950 | 147,485 | 102.9 | 56.6 | 13.9 | 19.9 |
| 2000 | 160,568 | 99.5 | 61.0 | 7.0 | 10.5 |
| 2050 | 164,758 | 103.8 | 50.8 | 3.5 | 10.5 |

So few filled squares lack a road through them; what none of them has is streets of its own. Its streets are the lanes of whichever ways pass, at their angles, and on island 7 in 2050 the viewer builds over 2,999 closes of 376 filled squares one by one because no street reaches them (`ISOSTATE_TOWNS=towns`; about one close in ten). In Dunheath every square taken after 1900 was filled whole (88 of 88 in 2050), so the first city has no ground grown close by close after 1900 at all.

Read:

- **The railway carries nine tenths of all freight from 1900 to 2050, and nothing changes after it.** There is no lorry: the carters' sector has its tasks in the tasks table and no machine reaches it.
- **The roads lose their order.** Turnpikes, the top of a ladder that only traffic climbs, grow to 4,400 km, three squares of land in four; and by 1950 nearly every square round the first city has one, so a main road is everywhere and therefore nowhere.
- **Roads vanish.** Made roads that the railway left quiet fall back after six years: 2,886 km in 1900, 1,325 in 2050, and the links drawn fall from 6,119 in 1950 to 4,587. The picture of 2050 has fewer roads than 1950's, which no country had.
- **A made road of 2050 fords a river more often than it bridges one** (483 fords and 352 bridges on drawn roads): the parish's rate bridges where the fords' charge pays, and with the freight on the railway few road fords pay.

## 2. The principle

**Traffic makes ways; every costly thing is a work with a builder, a payer, a cost and a year, built when it pays the body that builds it under the rules of its era, and never moved after; land decides shape.** The managing session's words, kept, with four things sharpened.

1. **Who builds decides what counts.** Each builder counts only the traffic that is its own gain: a parish its own district's carting to its own market, a turnpike trust whatever passes its gates, the county the fords' cost to all, an owner the price his building land fetches, the state everybody's time. That one line is why England's main roads were bad before the trusts, why the trusts' roads are today's main roads, and why a bypass is the state's and not the town's. So the traffic the simulation counts must say whose it is: a district's own, and the trade between towns (part 3).
2. **What was paid for is never unmade by quiet.** Only a worn track fades. A made road that loses its traffic stays a made road, bettered no further; a bridge stands; a closed way is closed by a decision (a railway company's, a hand's), never by the passing years. Today's rule, which lets made roads and turnpikes fall back, goes for every level above the track (stage 2).
3. **Bettered in place before moved.** A way is widened, surfaced and eased where it runs; a new line is built only where bettering the old one costs more, and what makes it cost more is what stands beside it: the frontage of a town's high street at its land value. That is the whole of a bypass, and a ring road is only bypass arcs that happen to join. An inner ring cut through a town's middle is a statute's work (the 1960s), so it comes only by the hand.
4. **Land decides shape in three ways, not one:** the slope (a gradient costs, a mountain is gone round), the water (one bank, one bridge, a ford where a bridge does not pay), and what stands on the ground (land is bought at its value, so a new way goes round the dear middle and over the cheap fields).

What follows: the countryside's ways are the oldest lines on the island, kept to one bank and bettered for two centuries; a town's streets are older than its buildings; a grid appears only where one owner or a statute laid ground out at one time, or where the fields themselves were straight (a late enclosure's fields are, and streets laid along them come out nearly square); a motorway runs between the largest towns over cheap ground. What it rules out: perfect rings; a grid laid over an old town; a bridge at every crossing; a road that moves; a road that vanishes because a railway opened; a highway the simulation has no traffic or payer for; and any street drawn where the record does not say ground was built.

## 3. What a way carries

**On what: the link, not the square.** A square's weight cannot say which way through it carries the load (a crossroads carries four), the picture draws links, and the crossings already add up their traffic by link (`CrossingState::traffic`). So the count is by link: each square's four links east, south, south-east and south-west (the other four are its neighbours'), the same slots the crossings' table and the viewer's `road_segments` use.

**What is counted.**

- *Goods, by weight, a year.* The weight carted along each link both ways, from the same trees and flows `update_roads` walks today (one walk, done onto links instead of squares; `tree_traffic` and `pair_traffic` gain a twin). The mode is the link's own in that year's cost graph: rail where it is a railway's, boat on a canal or navigable river, cart otherwise; the record keeps the cart's (what a road carries), since a train and a boat follow their own lines. The core keeps two parts for its rules and never records them apart: a district's own carting to its market, and the trade between towns.
- *People's journeys, a working day, from 1901.* Each square of town ground sends its working townsfolk (its residents times the share at work) to its town's market place and back along its town's own tree of paths: one more walk of a tree the simulation already makes. They are cheap to reckon, so they are counted. Before the tram and the car they only widen streets toward the middle; in the motor age they are split by how they go (on foot, by tram, by car), and the car's part is what fills a car park and a bypass.

**How often:** once a year, at `update_roads`. The simulation lays its carting once a year and has no harvest month; a month's laying would cost the simulation's thread 2.6 ms a month ([probes/dx-record.md](../probes/dx-record.md), stage 4).

**What it adds to a run.** Two layers, `way_goods` and `way_people`, four bytes a square a year each (one a link slot), on the rent byte's log scale of ten decades; `way_people` is nothing before 1901. Island 7 has 10,080 squares, so each layer is 6.1 MB raw to 1900 and 12.1 MB to 2050, beside a record of 233 MB raw to 2050; the sea and quiet links are naughts, and the builder measures them gzipped (the one-byte traffic layer of the record's probe came to 0.64 MB gzipped a run to 1900). A live game sends both, 40 kB a year each. Written only in a run that keeps them (the switch of stage 2), so the port's record is untouched. If gzipped they pass a tenth of the record, the people's layer is kept only for squares of town ground.

**The three uses.** *The picture*: a road's width, its wear (grass down the middle of a quiet lane, a worn crown on a busy one) and how many carts, lorries and cars ride it. *The rules*: what pays to build or to better (part 4), each builder reading its own part. *Seeing*: ribbons along the ways as wide as what they carry, and a town's card naming its largest partners; for the partners the books already keep the trade between towns (`acct_*`, flows between towns), so her note of 19 November, "town markets, largest trading partners or flows", needs this record only for the ribbons.

## 4. Who builds a way, and when

Each builder is a body that raises money once, holds its ways, may charge for passing them, and keeps them. It builds where its gain on last year's traffic pays its hurdle on the cost: stone, hands at the town's wage, and from 1901 the land taken at its value.

| the way | era | who pays | what it reads to decide it pays | what it leaves for good | what a hand can turn |
|---|---|---|---|---|---|
| the worn track | always | nobody | weight passing (today's thresholds) | a line, which fades after quiet years | close a way |
| the parish's made road | 1750 to 1888 | the district's rate on its landlords, and its men's days | its own district's carting saved | the made road on the track's line | the rate; forbid |
| the turnpike trust | 1750 to the 1880s | lenders, paid out of tolls | tolls on all that passes, less upkeep, against 5 per cent | the main road, its toll houses, a few cut-off bends | the toll; forbid trusts |
| the county's bridge | always | the parish rate for the county | fords' and ferries' charge on all traffic (built) | the bridge, widened in place later | bridge cost; forbid |
| the town's board that paves | 1800 to 1974 | a rate on the town's house rent | the town's streets' traffic and its rent | paving, kerbs and lamps; a street's width | a by-law: least width, back lanes |
| the owner who lays out an estate | 1760 on | the landowner or builder (from 1919 the council too) | the town's bid for the square against the plough (`ground.fill`) | the estate's streets, in its era's pattern | a by-law: grid, crescents, culs-de-sac |
| the tramway | 1880s to the 1950s | a company or the town | journeys a day along its main streets | rails in existing streets, a depot; the reach of eight squares | build or forbid |
| the state's trunk road | 1909 on | the treasury | everybody's time on the trade between towns, by motor | the old main road widened, surfaced and bridged | budget; forbid |
| the bypass | 1920s on | the treasury | through traffic's delay in a town's streets, against widening the high street at its frontage's value | an arc round the town's edge of its year | build or forbid, by town |
| the motorway | 1958 on | the treasury | through traffic between the largest towns | a new line over cheap ground, few junctions, cuttings and banks | build between two towns, or forbid |
| the car park | 1950s on | the town, or the works or shop | cars arriving at a square, against its land value | a surface lot on cheap ground, a deck on dear | a by-law: places a building |

Read across the eras: the island's countryside is made by the first four, the towns by the next three, the motor age by the last four. The railway and canal companies are already this pattern, and the crossings rule is the county's bridge. Three more things the table settles:

- **The parish counts its own; the trust counts all.** A parish has no reason to better a road for others' traffic, which is why the trusts came; with the traffic record's two parts the prototype's parish reads its district's share and its trust the whole (the prototype's guess, that a toll is paid by the town at the gate, goes: the toll follows the load).
- **The state's roads are the only ones that read through traffic by motor.** So a main road is main again from the 1920s: a few lines between the large towns get the trunk road's width, its bridges and its later bypasses, and the rest stay the turnpikes and made roads they were. That restores the order part 1 found lost.
- **A bypass is a sum, not a shape.** For each town on a trunk road with through traffic, the state prices widening its high street (the frontage's land value over the way's length through town) against a way round, found over the squares at the town's edge by the cost graph with land at its value; it builds the cheaper when the time saved pays. The way round keeps outside the town's ground of that year, so the town later grows over and past it, and a second bypass of a later year lies further out. Where two towns' bypasses, or one town's on two trunk roads, meet, a ring is there by accident.

## 5. Streets inside towns

This part is built first, tonight, in the viewer alone. It reads only the record, so the same record always draws the same town.

**Where a street comes from.** Four sources, and every street drawn belongs to one of them:

1. **The roads through, older than the town:** the ways between squares, as today. They are the high streets.
2. **The lanes and field boundaries of the closes the town was built over:** on ground built close by close, a street follows the hedge line it replaced. On ground built by 1900 today's lanes stand for these, and are kept.
3. **Streets laid out by one owner at one time:** a square filled whole (`ground.fill`) is one owner's estate in the year it was filled, laid in that era's pattern.
4. **Streets cut through later at great cost:** a later stage's (a bypass, an inner road by statute). Nothing cuts tonight.

**A street's line outlives every building on it.** Rebuilding a lot (`lots.rs`) never moves a street; blocks rise on the plots the streets made.

**The rule, in steps.** For a year k of a record that keeps town ground:

1. **Squares built by 1900 are left alone.** A square whose ground joined its town by 1900 (the record's `built`) keeps today's ways, lanes, cross streets, plots and turns, whatever happens round it.
2. **Every spot of a later square belongs to its nearest way.** The ways are those today's `ways()` makes as its own kind 0 (the roads, and the lane to the square a square grew from where no road runs), not their lanes; the spot's frame is that way's own, `u` along it from its first meeting place and `v` out from it. This is today's rule that a lane gives way to the nearer way, carried to the whole square.
3. **A square filled whole is laid out as an estate of the year it filled** (a square with a park in the record is one filled whole; its year is the year it joined, which the record keeps in two-year steps past 1900). In the frame of step 2, from the way's frontage (`v` above 0.06) outward:

   | joined | pattern | streets | blocks | width |
   |---|---|---|---|---|
   | 1901 to 1918 | by-law terraces | parallel to the way every 0.12, cross streets every 0.42 | 0.12 deep, two rows of terraces back to back | 0.016 |
   | 1919 to 1944 | garden suburb | parallel every 0.16, bowed by 0.05 in a wave of 1.1 along the way; cross streets every 0.55 | 0.16 deep, semis with gardens | 0.018, verges |
   | 1945 to 1969 | estate with a loop | two distributors at 0.2 and 0.46; culs-de-sac off the first every 0.18, 0.13 long, ending in a turning circle | open ground between slabs and pairs | 0.022 and 0.016 |
   | 1970 to 2050 | culs-de-sac | from the way, every 0.22, 0.24 long, a turning circle and a side stub; no way through | detached houses on wide plots | 0.014 |
   | any, on ground worth three times its town's of 1900 (`BLOCK_VALUE`) | blocks | parallel every 0.13, cross streets every 0.2 | perimeter blocks, the street wall all round | 0.018 |

   The land value read is the square's in the year it filled (the record keeps it every year past 1900; the sketch below read it in 1950 or 2050). The patterns' phase is the way's, not the square's, so a street runs on unbroken into the next square beside the same way, and two estates of different years meet in a visible seam, as they do.
4. **Where the ground of two ways that do not meet comes together** (the second-nearest way within a street's width of the nearest, the two sharing no meeting place), a back street runs along the line between them and closes the blocks. Between two ways that meet at a junction nothing is drawn on that line: it would be a spoke from every crossroads.
5. **A square taken after 1900 and not filled** keeps today's lanes until the field boundaries are taken up (step 9, later): it is the town on cheap ground spreading in ribbons, which the designer saw and did not fault.
6. **No street crosses** a park close, a works' yard, a churchyard, the railway or a river's line; it stops at their edge (today's `Laying` checks, kept). A street end that stops short of another street by less than 0.03 is drawn to meet it.
7. **Plots front every new street** as they front lanes today: on both sides, the frontage first, the plot's turn its close's place in the square's order. A filled square's lots are all taken the year it filled (as `lots::infill` takes them now), so the estate goes up at once, in the forms of its year (`forms::choose`), and is rebuilt afterwards only by `lots.rs`'s rule. On ground worth more than its town's of 1900 the frontage plots are thrown together (a street wall), with yards and gardens behind; on blocks (the last row of the table) the block is one `Lot::Block`.
8. **The closes no plot reaches** are built over by themselves as today; with the estates there should be few, and they are counted.
9. *Later, not tonight:* on ground taken after 1900 and not filled, streets along the field boundaries of the closes it was built over (the field system's parcel edges inside the square, which the viewer already makes), so that a town grown on old enclosures has crooked streets and one grown on a late enclosure's straight fields has nearly square ones.

**A works on dear ground.** A works' yard keeps its size today, and streets stop at its wall. On ground worth three times its town's of 1900 a yard is the dearest use of a square; the stage of sites and the price of land should make such a works shrink its yard or move out to cheap ground, and the estate then takes the yard's ground in the year it is given up (today's `Yards` already gives the ground back in order).

**What must not change.** Nothing on a square built by 1900, in any year (its streets, plots, houses and their years); every picture of 1900 or before. A lot that is not rebuilt keeps its building. A street once laid is never moved. Ways between squares are as today. A record made before the ground's layers is drawn as today.

**What the record must newly carry.** Nothing: the year a square joined (two-year steps past 1900), whether it was filled (its `park`), its land value and `dens` are there since 21 November. The streets of step 4 of part 4 (a bypass) will need the record to say where a built way runs, which is stage 4's.

**Its tests.** (a) On island 7, every plot of a square built by 1900 is the same, with its year, in 1900, 1950 and 2050, rule on and off (`ISOSTATE_STREETS=old` draws as today, for pictures side by side, as `ISOSTATE_LOOK=plain` does). (b) The same record drawn at 2050 cold and after drawing 1950 gives the same streets and plots. (c) On made-up ground, a filled square of each era between two parallel ways gets streets of its pattern, joined to both, and none over a park close or a river's line. (d) A plot standing in year k stands in k+1 unless `lots.rs` rebuilt it.

**Its pass.** Island 7 in 2050 (`ISOSTATE_TOWNS=towns`): the closes of filled squares built over by themselves fall from 2,999 to under 600; Dunheath's plots standing from 1900 (1,524 by `ISOSTATE_LOTS=Dunheath`) unchanged; the houses wanted and drawn of 1900 (28,776 and 25,770 on the island) unchanged; a year's arrival no slower than the noise of two runs.

**The pictures to judge it by.** Island 7, 2050, Dunheath from the north at distance 11 (46, 63, pitch 60) and its north-east close at distance 4 (50.5, 59.5, pitch 62), before and after; Mallowbrook at 2050; Iveling and Grancombe (cheap ground: as before); and the standard twelve, whose years to 1900 must look as they did. Today's close view of Dunheath's north-east shows three blocks and a works standing on wide plain town ground: squares filled between 1963 and 1984, each with 1,000 to 1,400 residents in the record, on which hardly a building is drawn. That ground, and the clusters elsewhere, are what the estates fill.

**The sketch.** A script of the standard library (kept with the night's reports, out of the repository) drew Dunheath in 1900, 1950 and 2050 from what island 7's simulation holds, streets coloured by where they come from, under steps 1 to 4 (it imitates today's lanes and leaves out plots, yards and the field boundaries). What it showed, and what changed the rule:

- Laid square by square in each square's own frame, the estates read as a stamp a square, cut off at the square's edge and lying across the ways. Laid in the frame of the nearest way (step 2), the same patterns front the old roads, run on across square edges and close into blocks; that is the rule above.
- Spokes appeared from every junction where the ground of two meeting ways comes together; hence step 4's exception.
- Of Dunheath's 88 filled squares, 6 are by-law ground, 24 garden suburb, 21 post-war, 15 culs-de-sac and 22 blocks on dear ground. The city took 17 squares in the 1910s and 15 in the 1920s, 4 in the 1950s and 24 again in the 1960s and 1970s, none after the 1980s; the blocks stand nearest the old town (4.9 squares from the market place on the mean, against 6.6 for the culs-de-sac).
- By 2050 nearly every square round the city has a turnpike through it, so the brown net of old roads is close and even; the estates make the blocks between, but the town's skeleton of high streets will read only once the state's trunk roads pick out a few (part 4).

## 6. The motor age

**The lorry** is a machine that takes the carter's hauling, as the tractor took the ploughman's: a use on the carting sector in the tasks table (where "no machine reaches the carters yet"), found on oil and on motor vehicles, a good of a motor works the content does not have yet. Bought as plant by the towns' carting, it lowers the hands a load takes, so the freight rate falls where it is found; and it runs well only on a made surface, so in the cost graph a lorry's square costs more by the road under it than a cart's did (a track nearly as dear as open ground, a surfaced trunk road far cheaper than a turnpike). That is what makes roads matter again after 1920 and the railway lose its short hauls.

**The car** is a household's good (travel), bought as incomes rise. Cars a head lengthen a town's reach beyond the tram's eight squares (`ground.reach`), and ground taken by car is built at a lower density: that is the suburb on cheap ground, a square filled after 1945 in the loop or culs-de-sac pattern with wide plots. Its journeys, counted in part 3, fill the streets toward the middle, and the car's share of them asks for parking.

**Parking** is a car park, a work of the town or of a works or shop: cars arriving at a square times the hours they stay, against the square's land value. Cheap ground gets a surface lot; ground worth three times its town's of 1900 a deck (the forms already have a car park).

**The bypass and the motorway** are the state's works of part 4, reading through traffic by motor from the traffic record and paid from the treasury's public works, which today buys only teachers' hands ([rules/state.md](../rules/state.md), "How the treasury buys").

**What is drawn moving.** Carts in proportion to a road's goods by cart until the lorry, then lorries in proportion to the lorry's share; cars in proportion to its journeys by car; trams on the tramway's streets while it runs; trains as today. Parked cars on the car parks and at the kerb of streets where journeys end.

**What the simulation must come to know.** Motor vehicles as a good and a motor works; the lorry as a use on carting; a road's surface as a betterment of its level (a fourth level, the motor road, for the state's trunk roads); cars a head; the people's journeys by mode; and the land a way takes, at its value.

## 7. The hand

The sandbox's stage 7c, never built, laid a road or a railway between two towns by two clicks. With builders it becomes a set of handles on who builds, each an edit at the year's turn, logged like any other:

- **Who may build:** each builder of part 4 on or off for the island, or for a town (no trust in Kelside; no motorway anywhere).
- **Tolls:** a trust's toll a square, from nothing to twice the prototype's.
- **A by-law for streets:** for a town, the pattern its next estates are laid in (by-law grid, crescents, culs-de-sac, blocks), and the least width of a new street.
- **A bypass built or forbidden:** click a town, then the side; the state's sum is shown, and the hand builds it even where it does not pay, at the treasury's cost.
- **A way built or closed:** two clicks for a road between two towns, as 7c designed; a click on a link to close it (its traffic goes round).
- **An inner ring:** only by hand, as a statute, at the land value of what it cuts through.

## 8. Stages

Each about one builder's night.

1. **Town streets** (tonight). Part 5 in the viewer: `scatter/towns/kept.rs` (`ways()` and `lay_out`) and `lots.rs`. It reads the record as it is and needs nothing else. Switch: `ISOSTATE_STREETS=old` for pictures side by side; no rule of the core moves, so neither golden test can. Pass and pictures: part 5.
2. **The traffic record, and nothing unmade by quiet.** In the core: goods by link with their two parts, people's journeys by link from 1901, in the State and in two layers of the record and the live year (part 3), behind `ways.record` (off under the port's rules, on under the game's); and, behind its own switch `ways.keep`, from 1901 a made road or turnpike no longer falls back. In the viewer: a road's width and wear and its carts by its goods. Needs nothing. Pass: the links' weight into each square adds up to today's square traffic to a part in a million; the layers under a tenth of the record gzipped; made roads of 2050 no fewer than 1950's. The game's golden test moves (the record, and `ways.keep` in its second era) and is anchored again.
3. **The lorry.** Content: motor vehicles and a motor works; the lorry as a use on carting; a lorry's cost by road surface. Needs the content line of "new things after 1950". Pass: the carters' hands a load halved within forty years of the lorry's find; the road's share of freight rising from it; the standing check.
4. **The state's ways.** The treasury buying goods (public works); trunk roads (the motor road level) between the large towns, bettered in place with their fords bridged; bypasses by the sum of part 4. Needs 2 (through traffic by link) and 3 (motor traffic). Pass: a few hundred km of trunk road an island by 1970; bypasses on the towns with most through traffic and none on the rest; no made road of a trunk road fording a river.
5. **The car.** Cars a head; reach by car; suburbs at low density; car parks; cars drawn moving and parked. Needs 2 (journeys) and 3 (vehicles).
6. **The builders of the played century.** The prototype's parish and trust, its toll paid by the load from 2's two parts, switched on. It moves 1750 to 1900, so it comes after the motor age and is weighed against the standing check.
7. **The hand on ways** (part 7), on top of 4 and 6.

What I would change in the order the brief gave (streets, record, motor age): split the motor age into the lorry, the state's ways and the car, because a bypass will not pay while rail carries nine tenths of the freight, and the lorry is what changes that; and put "nothing unmade by quiet" in stage 2, since it is one line, the picture of 2050 loses a third of its made roads without it, and it acts only after 1900.

## 9. Open questions, with this page's answers

- **Should the squares built by 1900 also get streets from their field boundaries?** No. Their pictures are the ones the designer has looked at most and likes; part 5 changes only ground taken after 1900. Revisit if she asks.
- **Should bridges cost more?** Yes, with stage 6: a span beyond 20 m dearer than in proportion. Today a parish bridges nearly every ford that has traffic, for 0.23 parts in 10,000 of the period's output, and a dearer bridge is more reason to keep to one bank. Tonight it is left.
- **Lakes and tideways**, which the simulation neither prices nor bridges and the picture crosses on causeways: in stage 4 the state's bridge at a tideway's head, or a ferry's charge across, enters the crossings' table; until then the causeways stay a named fault.
- **Do inner ring roads ever come by themselves?** No: they were planners' works under statute, and the island has no planner. The hand can build one.
- **Is a square too coarse for estates?** A square is 2 km of the map at five times life size, about 120 houses; an estate a square is a large one, and that is the scale the record gives. Smaller estates would need the record to say which part of a square filled when, and it does not.
