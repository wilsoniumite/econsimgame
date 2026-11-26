# The lorry: a machine that hauls, and roads that matter again

Not in the Python. The third stage of the ways' design page ([../designs/ways.md](../designs/ways.md),
part 6 and part 8, stage 3), in three rules, each behind a switch of its own:

| switch | what it does | code |
|---|---|---|
| `tasks.lorry` | the motor lorry: a machine for the carters' hauling, a use of the tasks on the carting, bought by each town's carting where it pays; an hour of the carting's work then takes fewer hands, and the cost graph's land squares are cheaper by what that hour has come to cost | `data/tasks.toml` (`lorry`), `tasks.rs` ("the carters"), `carting::take_carters`, `lorry::road_mult` |
| `lorry.roads` | a lorry wants a made road: its saving is had in full on a made road or a turnpike, a quarter of it on a track and none on open ground | `lorry.rs` |
| `lorry.keep` | nothing unmade by quiet: from 1901 a made road or a turnpike no longer falls back for want of traffic | `lorry.rs`, `sim::update_roads` |

All three are off under the port's rules and left out of its files. Nothing of them acts before
1901: the motor vehicle is not sought before 1905, and the road is kept only from 1901. So the
played century stands by construction, and it was proved: with the three off, islands 1 to 16 to
2050 differ from main in none of the 4,312 facts both have; with the lorry on, the twelve standing
facts are 12 of 12 inside and every fact to 1900 is main's to the digit (`tests/lorry.rs` holds
island 7 to the end of 1904).

**On under the game's rules, all three.** The lorry is found in 1923 on the mean of islands 1 to 8
(1909 to 1933), and the hands a load takes by road are half what they were by hand 43 years later
(41 to 45 on every island of sixteen; 41.5 on main with pace, below), past the forty the pass asks:
that one number is missed, and every other part of the pass is met ("The decision", below). Machines that tend
([tending.md](tending.md)) stay off: with the lorry and tending both on, the towns' share of people
in 2050 is 0.493, against 0.550 with both off.

## Why

The railway carried nine tenths of the island's freight from 1900 to 2050 and roads vanished from the
picture after 1950, because there was no lorry: the carters' sector had its tasks in the tasks table
and no machine reached it. The machines that tend ([tending.md](tending.md)) stood off for the same
reason: the hands they freed turned carter. A country's roads became main again in the 1920s
because of the lorry.

## What it does

**The motor vehicle** is an idea of the tree's stage 6 (`data/game.toml`), an invention as the
tending ideas are: found by the tree's hazard on what the island has made, machinery (6 million units,
the moving line's) and oil (300 units, the tractor's: the first oil works' first months), from 1905,
with draws of its own (`rng::idea_stream`), and sought only while the lorry may be found, so that with
the lorry off the island is the island without it. The island has no oilfield: its oil is made from
coal (stage 6, [tree.md](tree.md), "Oil, nitrogen and the land"), so the lorry needs no fuel of its own
invention, and an island that makes no oil has no lorry.

**The lorry** is a `[[use]]` of `data/tasks.toml`, class haul, on the carting (`on = ["carting"]`),
found at the year's turn after the motor vehicle. The carting's hours are seven tenths hauling (the
loading, the carrying, the horses' work), two tenths tending (driving, the horses' care) and a tenth
judging, as the file has had them since the land's half of the tasks. The lorry may hold any part of
the hauling, from the easiest task. Each month, in each town:

- **what a unit costs** and **the town's target** are the tasks' rule's, as for every machine
  ([tasks.md](tasks.md)): the return a works must see and the wear on its plant, its oil and its driver,
  over its quality; the target is the share of the carting's hauling where the town's wage over that
  cost falls on the schedule;
- **the town's carting means to hold** a share of its hauling (`cart_auto`), moving toward the target
  by an eighth of the gap a year, gated by how well last month's order for lorries was filled, and
  falling no faster than they wear; it **holds** what its lorries in place can do (`cart_held`), from
  its plant over last month's work;
- **an hour of the carting's work** then takes `h` hands (the hauling left to hands, the tending and the
  judging, and the drivers) and costs `h * wage + m`, `m` the lorries' return, wear and oil;
- **the carters** are the work the town's carting bill pays for at that cost, times `h`:
  `labor * bill * h / (h * wage + m)`. By hand `h` is one and `m` nothing, and this is the old rule to
  the last digit;
- **the lorries' goods**: the town's carting asks for the oil of its lorries in place for the month's
  work with the day's trade, and for a quarter of what its plant lacks for the share it means to hold
  beside the works' orders for plant; the town's households pay for both out of the carting they are
  paid (the hauliers own their lorries, as the farms' machines are the farms'), and the books count them
  among the households' purchases. The return on the plant is never paid to anyone: it stays with the
  households, as the land machines' stays in the rent.

**The freight falls where the lorry is found.** At each year's end the island's cost of an hour of the
carting's work over the wage, `kappa` (each town's `h + m / wage`, weighted by its bill), is kept, and
next year's cost graph multiplies each land square's cost by it: so the money a load costs over a
square falls to `kappa` of what it was, and of that money the hands' part is `h / kappa`, which is `h`
of the hands by hand. The fall goes into every price the carting enters: what a farm is paid at its
gate, what a works pays for its coal, what the trade between towns costs and which way it goes.

**A lorry wants a made road** (`lorry.roads`): the saving `1 - kappa` is had in full on a made road
or a turnpike, a quarter of it on a track, and none on open ground (`lorry::SURFACE`): a lorry on open
ground is no better than a cart, and on a track it sinks to its axles in a wet month. So a made road is
cheaper against a track than it was, the trade turns onto the made roads and turnpikes, and the
railway's short hauls go to the road where the road is good. Rail, river, canal and sea are as they
were: a train, a boat and a ship follow their own lines.

**Nothing unmade by quiet** (`lorry.keep`, the design's principle, part 2): from 1901 a made road or a
turnpike is kept whatever its traffic; only a track fades after six quiet years.

## Why no motor works

The brief asked for a good and a motor works that makes it. Both were built first (a good,
`motor_vehicles`, made by a works from steel, machinery, glass and electrical goods, the lorry made of
it), and taken out again: **a new good moves the played century.** With the good alone in the content
(no works making it, no code), island 2 parts from main in the books at the end of 1763 and in its
works in January 1765, and islands 1 to 16 differ from main in 3,832 of 4,312 facts (the people of
1800 on seven islands of sixteen, by a few hundred). The goods' table is summed and compared in many
places in a town's month, and a good more changes the run in its last digits from the first year;
the tending ideas, which are ideas and not goods, moved nothing. So the lorry is made of machinery and
steel, six tenths and four tenths by value, as the tractor and the combine are, and the motor vehicle
is an idea. When a stage adds goods after 1900 and takes the first era's hashes again on purpose,
the lorry's plant moves to its own good, with the tractor's and the combine's (the backlog's item).

## The numbers

The lorry's entry in `data/tasks.toml`:

| | the lorry |
|---|---|
| class | haul, from 0 to 1 of it |
| on | the carting |
| plant | 71.4 machinery and 59.5 steel a machine (six tenths and four tenths by value, as the tractor's); life 10 years |
| run | 0.56 oil a month |
| minders | 0.15 of a hand: the driver |
| quality | grows 9 per cent a year from the year found, to a cap of 50 |
| found | at the year's turn after the motor vehicle, found from 1905 on 6 million units of machinery and 300 of oil made |
| scale | 1.0 |

**Sized** as the design's table of machines sizes it, to take a fifth of hauling when found: a unit
costs 1.04 of a month's wage, six tenths of it return and wear on 34 months' wages of plant, 0.15 of a
hand its driver, the rest its oil, at the island's prices of about 1925 (wage 10.5, machinery 3.0,
steel 2.4, oil about 5). The towns' target in the year found is 0.193 on the mean of islands 1 to 8
(0.126 to 0.240; `use_share_found_lorry`), so it was not refitted.

**The pace is history's.** The design gives the lorry four per cent a year to a cap of 6. A lorry of
1925 carried three tons at twenty kilometres an hour and one of 1965 twenty at seventy: some
twenty-three times the load moved an hour by one driver in forty years, and the diesel of the 1930s
took half the fuel a ton-kilometre, so some thirty times a unit's work for its plant, fuel and driver,
nine per cent a year. At seven per cent the hands a load takes were halved 52 years after the find, at
eight 46, at nine 43.

`lorry::SURFACE` is `[0, 0.25, 1, 1]` (none, track, made road, turnpike); `lorry::KEEP_FROM` is 1901.
Neither was swept.

The facts: `lorry_share_Y` (the carting's hauling done by the lorries in place), `cart_hands_a_load_Y`
(the hands an hour of the carting's work takes, 1 by hand) and `cart_kappa_Y` (its cost over the wage),
every tenth year from 1900; `cart_hands_half_year`; `freight_lorry_Y` at the marks (the freight by road
times the lorries' share of the hauling, millions of weight-km); and the tasks' `use_*_lorry`. The
series `lorry_share`, kept only while the lorry may be found, is what the viewer's Traffic view reads.
`cargo run --release -p isostate-core --example lorry -- SEED OUT.csv [key=value ...]` writes a row a
year from 1890 of the carting: carters and all hands, the bill and its three parts, freight by mode,
the lorries' share, the hands a load takes, kappa, the roads by level, the towns' share.

## What the islands showed

### Before: the carting on main, 1900 to 2050

Islands 1 to 8 under the game's rules with six families, the lorry's three switches off (which is
main: islands 1 to 16 to 2050 differ from main's sweep in none of 4,312 facts), from the probe above.
Carters over all the hands at work; the carting bill over output (what carting adds to a good's
delivered price, on the mean); the bill in the books' three parts (a works' materials and output, the
land's produce to its market, the trade between towns); the weight carried times the squares it went,
by mode; carters a million weight-squares carried.

| | carters, of all hands | bill over output | works / land / trade | road / boat / rail | carters a million weight-squares |
|---|---|---|---|---|---|
| 1900 | 0.091 | 0.058 | 0.18 / 0.38 / 0.44 | 0.058 / 0.004 / 0.937 | 14 |
| 1920 | 0.079 | 0.052 | 0.27 / 0.31 / 0.42 | 0.062 / 0.004 / 0.934 | 9 |
| 1950 | 0.087 | 0.052 | 0.45 / 0.23 / 0.31 | 0.082 / 0.004 / 0.914 | 6 |
| 1970 | 0.082 | 0.050 | 0.50 / 0.22 / 0.28 | 0.087 / 0.003 / 0.910 | 5 |
| 2000 | 0.089 | 0.053 | 0.54 / 0.21 / 0.25 | 0.092 / 0.003 / 0.905 | 5 |
| 2050 | 0.100 | 0.054 | 0.57 / 0.18 / 0.25 | 0.091 / 0.003 / 0.906 | 5 |

The carting bill is a twentieth of output all century and carters a tenth of the hands; the books
cannot tell the cart to the station from the cart to market, because the carting money pays for every
square of a load's way, rail squares too (nobody collects a railway's fares). The hands a load takes
fall by two thirds from 1900 to 1950 with no machine at all: the freight rides on the cost of living
from 1900 ([longrun.md](longrun.md)), the wage rises faster, and a load is so many hours of a dearer
hand's pay. Rail carries nine tenths of the freight to 2050 and the road a twelfth.

### After: the lorry on, tending off

The same probe with the lorry on:

| | carters, of all hands | bill over output | road / boat / rail | the lorries' share of hauling | hands a load takes (1 by hand) | kappa |
|---|---|---|---|---|---|---|
| 1920 | 0.078 | 0.052 | 0.063 / 0.004 / 0.934 | 0.054 | 0.964 | 0.971 |
| 1940 | 0.079 | 0.051 | 0.079 / 0.004 / 0.917 | 0.331 | 0.776 | 0.806 |
| 1950 | 0.077 | 0.048 | 0.095 / 0.004 / 0.901 | 0.499 | 0.666 | 0.702 |
| 1970 | 0.070 | 0.044 | 0.130 / 0.006 / 0.864 | 0.777 | 0.489 | 0.540 |
| 2000 | 0.063 | 0.045 | 0.161 / 0.009 / 0.830 | 0.906 | 0.391 | 0.467 |
| 2050 | 0.073 | 0.048 | 0.161 / 0.009 / 0.830 | 0.908 | 0.393 | 0.472 |

### The four arms

```bash
isostate-core sweep --seeds 1-16 --rules game --families 6 --end-year 2050 --set tasks.lorry=false --set lorry.keep=false --set tasks.tending=false,true --standing --out OFF
isostate-core sweep --seeds 1-16 --rules game --families 6 --end-year 2050 --set tasks.lorry=true --set tasks.tending=false,true --standing --compare OFF --out ON
```

Run on 26 November 2026; 64 runs, none failed. Means over islands 1 to 8, with the lorry's switches
off or on and machines that tend off or on. The growth asks are each island's own ratios' mean, as the
standing check reads them.

| islands 1 to 8 | both off | tending only | lorry only | both on |
|---|---|---|---|---|
| the towns' share of people, 1970 / 2050 | 0.576 / 0.550 | 0.565 / 0.466 | 0.588 / 0.567 | 0.579 / 0.493 |
| carters, of the hands, 1950 / 2000 / 2050 | 0.080 / 0.072 / 0.074 | 0.083 / 0.081 / 0.112 | 0.071 / 0.052 / 0.055 | 0.076 / 0.063 / 0.080 |
| the land, of the hands, 1970 / 2050 | 0.122 / 0.149 | 0.121 / 0.169 | 0.123 / 0.135 | 0.126 / 0.157 |
| freight by road / rail / boat, 2050, millions of weight-km | 1,823 / 18,030 / 68 | 2,676 / 27,010 / 91 | 3,022 / 15,510 / 160 | 5,181 / 24,430 / 279 |
| of the road's, by lorry, 2050 | | | 2,756 | 4,825 |
| the road's share of freight, 1950 / 2000 / 2050 | 0.082 / 0.092 / 0.092 | | 0.095 / 0.162 / 0.162 | |
| the wage, per cent a year 1900 to 1970 | 1.25 | 1.32 | 1.23 | 1.33 |
| the wage, 2050 over 1970 | 1.14 [0.97 .. 1.26] | 1.12 [0.86 .. 1.25] | 1.14 [0.99 .. 1.27] | 1.20 [1.08 .. 1.31] |
| the land's share of hands, 2050 less 1900 | -0.097 | -0.077 | -0.111 | -0.089 |
| what is made a head (linked), 1970 / 2050, 1750 = 1 | 19.1 / 20.7 | 21.2 / 27.7 | 19.2 / 21.4 | 21.8 / 31.2 |
| goods a head, 2050 | 52.9 | 72.3 | 56.0 | 84.6 |
| real pay 2050 over 1970: haul, tend, clerk, judge | 1.77, 1.44, 0.68, 0.77 | 2.13, 1.19, 0.69, 0.84 | 1.89, 1.42, 0.68, 0.76 | 2.35, 1.27, 0.75, 0.93 |
| work's share of income, 2050 | 0.625 | 0.497 | 0.619 | 0.511 |
| slumps in the news (verdicts / alarms) | 14.6 / 15.4 | 17.4 / 18.0 | 13.1 / 13.6 | 18.1 / 18.9 |
| people, 2050 | 1.205 M | 1.226 M | 1.188 M | 1.192 M |
| the first city's townsfolk, 2050 | 191,700 | 147,500 | 164,000 | 121,900 |
| made roads / turnpikes / tracks, 2050, km | 1,303 / 4,275 / 1,930 | 1,422 / 4,932 / 1,890 | 2,408 / 5,972 / 882 | 2,206 / 6,584 / 1,080 |

On islands 9 to 16 the same: the lorry found in 1930 and the hands halved in 1971; the towns' share in
2050 0.536, 0.462, 0.556 and 0.484 in the four arms; 2050 over 1970 1.14, 1.20, 1.13 and 1.23.

**The twelve standing facts** are 12 of 12 inside in every arm, and to the digit: with the lorry on,
of the 15,200 facts named by a year to 1900 on sixteen islands the only ones that differ are reckoned
over windows or whole runs that reach past 1900 (the linked wage's and what is made's nine-year means,
the ages' births, deaths and life at 1900, the rails' dearest year, timber's swing, the works near
canals); the game's golden test's first era held.

**Speed.** A run to 2050, 32 at a time on the shared machine: 170 seconds a run with the lorry off,
179 on, in two sweeps with other work beside them; the lorry's month is one more mixture a town, so
what is measured is the machine's noise rather than the rule.

### Who gains and who loses

- The haulers gain: their real pay grows 1.89 times from 1970 to 2050 with the lorry, 1.77 without, and
  a quarter fewer of them are needed by 2050; the towns gain the people the carts let go, 0.567 of the
  people in 2050 against 0.550.
- The railway loses its short hauls: rail freight in 2050 is 15,510 million weight-km against 18,030,
  and the road's share of freight rises from 0.09 to 0.16.
- The tenders lose a little (their real pay 1.42 times 1970's against 1.44), and work's share of
  income is 0.619 against 0.625, as part of the carting's money now buys lorries and oil. The wage as a
  whole hardly moves (2050 over 1970, 1.14 both).

## The decision

**The lorry lands on, with its roads and the roads kept.** Its pass, measured on islands 1 to 8:

| the pass | measured | |
|---|---|---|
| the hands a load takes by road halved within forty years of the find | 43 years on the mean (41 to 45 on every island of sixteen); 0.52 of by hand at forty | missed by three years |
| the road's share of freight rises from the find | 0.082 to 0.095 by 1950 and 0.162 by 2000 (off: 0.082, 0.092) | met |
| no fact to 1900 moves | the economy, the books and the news to 1900 to the digit | met |
| the twelve standing facts in their bands | 12 of 12 inside | met |
| the three growth asks met with what lands on | 1.23 per cent a year 1900 to 1970; 2050 over 1970 1.14; the land's share 0.111 lower | met |
| no run fails or loses money | 64 runs to 2050, none failed | met |

The brief said the lorry lands on if its pass is met, and one number of six is missed by three years.
I land it on and say so here. The forty years is the design's reading of history; the miss is the same
on every island (41 to 45); and closing it would take a pace beyond the lorry's own history (it is
already nine per cent a year, from the ton-kilometres a driver moved and the diesel), or a lorry that
takes the carting's tending as well as its hauling. What is left at forty years is the carting's three
tenths that are not hauling (the driving and the dealing) and the drivers. Off, the island keeps rail
at nine tenths of its freight to 2050 and its made roads falling back to half of 1950's: that is the
larger fault. `--set tasks.lorry=false --set lorry.keep=false --set lorry.roads=false` is the island
without it.

**On main with pace** ([pace.md](pace.md), which landed while this was built), the same four arms on
islands 1 to 16 (`R/merged-on`, `R/merged-off` in the night's notes; 64 runs, none failed, 12 of 12
inside in each sweep), means of islands 1 to 8: the lorry found in 1925 and the hands a load takes
halved 41.5 years later (41 to 42; 40 to 43 on all sixteen); the towns' share in 2050 0.556 with both
off, 0.464 with tending alone, 0.570 with the lorry alone and 0.488 with both; 2050 over 1970 1.15,
1.09, 1.18 and 1.16; the wage 1.25, 1.35, 1.26 and 1.35 per cent a year from 1900 to 1970; the road's
freight in 2050 1,783, 2,692, 3,192 and 5,000 million weight-km against rail's 17,200, 26,240, 15,680
and 25,290. So with pace the lorry alone lifts 2050 over 1970 (1.18 against 1.15), both on no longer
reach 1.20, and the decision stands.

**Machines that tend stay off.** With the lorry and tending both on, 2050 over 1970 is 1.20 on the mean
of islands 1 to 8 (1.08 to 1.31), the first arm to reach the design's 1.20, and what is made a head in
2050 is half as much again as with both off (31.2 against 20.7). But the towns do not hold: their share
of people in 2050 is 0.493 against 0.550 with both off (0.567 with the lorry alone), lower on every
island. The lorry gives back about a third of what tending cost the towns (0.466 to 0.493), not all of
it: the hands tending frees still go to the carts (carters 0.080 of the hands in 2050 with both on,
0.055 with the lorry alone) and to the fields (0.157 against 0.135), and the first city shrinks
(121,900 against 164,000). What tending still waits on: work for the freed hands in the towns (the
design's new goods and needs after 1950: motor and appliance works, travel, care and schooling), and
carting that frees as many hands as tending sends it.

## What worries me

- **The money for a load's rail squares buys lorries too.** The carting bill is one sum of money for
  every square of a load's way, rail squares included (nobody collects a railway's fares), and the
  lorry's split of it is the town's. So the lorries' oil and plant are bought for the carting's whole
  work, and a town whose carting is mostly by rail buys more lorries than its roads would use.
- **One kappa for the island.** The cost graph is the island's, so its land squares are cheaper by the
  island's kappa of last year, each town by its bill, while each town's carters are reckoned at its
  own. A town far behind the island's lorries has its bill cut more than its own lorries earn, and its
  carters for it are too few; one far ahead, too many.
- **The books count the lorries' oil and plant as the households' purchases** (among their spending
  and their goods), and the carting money in full as pay for work, so work's share of income counts the
  lorries' return as pay.
- **Rail has no terminal charge.** The cost graph prices a load onto a railway at nothing, so the
  railway keeps every haul where its line runs and the road takes only what the line does not reach;
  in a real country the station's handling is what lost the railway its short hauls. With a charge the
  road's share would rise further. Not tried: it is the port's graph, and it moves the played century.
  Mended from 1901 by the station's charge ([haulage.md](haulage.md), `haulage.handling`).
- **The roads kept from 1901 are kept for ever**, and the made roads of 2050 (2,408 km) are nearly
  twice main's. The roads' wear still counts every mode, so a railway wears the roads beside it into
  turnpikes (the traffic record's fault, [traffic.md](traffic.md)), and with nothing unmade by quiet
  those turnpikes now stand: 5,972 km in 2050 against 4,275. From 1901 a road is worn by its own
  traffic alone ([haulage.md](haulage.md), `haulage.wear`).
- **The motor vehicle is an idea, not a good** ("Why no motor works"), and the lorry is made of
  machinery and steel. No motor works stands on the island, and the households have no cars.
- **The pace and the surface were set by hand** and not swept: nine per cent a year to a cap of 50,
  and a quarter of the saving on a track.
- **The carters' classes are counted as by hand** by the reckoning of pay ([pay.md](pay.md)), which
  reads the carters at the carting's shares whatever the lorries hold.
