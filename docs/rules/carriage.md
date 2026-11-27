# Carriage in hands: a load takes hands, and they are paid a wage

Not in the Python. Added on 27 November 2026, the fourth piece of the first century laid out anew
(docs/BACKLOG.md, "The first century laid out anew"; the study relay's verdict, section 6, row 4). Two rules,
each behind a switch of its own, both **off under both sets of rules**, and neither written into a
parameters file while off:

| switch | what it does | code |
|---|---|---|
| `carriage.in_hands` | a load carried over a unit of the path's cost takes `carriage.hands` hands for a month, the same number on both starts, and they are paid a wage, the carting town's own or the island's (`carriage.wage`); the railway's station charge is laid from the first year | `carriage.rs`, `longrun::freight`, `haulage::on`, and the readers in the table below |
| `prices.spoils_unkept` | a good that spoils (more than a tenth of its stock lost a month: services, town gas) is not stocked: its price reads the month's asking against its making, and what is not sold in the month is lost | `prices::unkept`, `tree::month_loss`, `tree::price_gap` |

They are meant to be turned on together with `prices.exports_ask` ([prices.md](prices.md), "What is carried
out is asked"), in the flip of the first century (piece 7 of the verdict), and not before: each of the three
alone leaves the island worse than main in one way or another (the study relay's `pockets.md` and
`carriage.md`). With all three off a run is main's to the last digit: both golden files unmoved, and the
game's sweeps on both starts show only the three seat facts that always read 0/0.

## Why

What carrying a ton costs was a sum of coins, fixed in 1750's money: `freight`, 0.015 coins a unit of weight
over a unit of the path's cost (a cell of open lowland with no road). Under the game's rules the settled
island kept that coin until 1900 while the money wage rose about threefold, and from 1901 let it ride on the
cost of living ([longrun.md](longrun.md)); the landfall start let it ride on the cost of living from 1750, a
stopgap ([landfall.md](landfall.md), "Carriage that follows prices"). Neither is what carriage is: a man, a
horse and a cart for a day, paid what a man is paid. On a coin that stands still while the wage rises, or that
follows prices while the real wage rises, a load takes fewer hands every decade with nothing on the road
changed: at landfall a ton carried a cell took a hundred-and-nineteenth of 1760's hands by 2050, of which
roads, canals, railways and the lorry account for about a sixth (study relay, `carriage.md`, section 3).

The second wrong is in the price: a town's price did not hear what merchants carried out of it, so a small
town beside a city read a glut every month and its prices sank to a fiftieth of the city's ("pockets").
`prices.exports_ask` mends that, but alone it lifts the money wage tenfold by 1890, a coin of carting then
buys a fifth of the hands, and the founding test of towns fails (13 towns in 1900 for 21.6). Carriage in
coins was held fair only by the old price rule's lean. The two are mended together or not at all.

The third is what the century probe found once exports are asked: the merchants' share of income falls below
nothing, and the largest part of the fall is services rotting on shelves that aim at two months of sales of a
good that loses half of itself each month.

## What it does

**The hands.** `carriage.hands` is the hands for a month that carry a unit of weight over a unit of the path's
cost: `carriage::HANDS`, the port's 0.015 coins over the settled island's mean wage in 1750 (4.49 coins a
month on islands 1 to 8), so 0.00334. It is the same on both starts: a colony's load takes the hands a
settled island's does, and costs more coins there because its hands are dearer (9.85 coins a month in 1750
at landfall). Better ways lower the hands as they lowered the bill before: the road, the canal, the train,
the sea and the lorry's cheaper squares are all in the path's cost (`transport.rs`),
and the rule does not count them twice.

**Whose wage** (`carriage.wage`, a word):

- `town` (the default): a reckoning made for one town's carting reads that town's wage of the month
  (`carriage::freight_at`). Since a town's carters are its carting bill over that same wage
  (`carting::take_carters`), they are then exactly the hands its loads took, until the lorry is found.
- `island`: every reckoning reads the island's mean wage, weighted by people, of the last closed year (the sum
  [longrun.md](longrun.md) keeps; 1750's first year reads the warm-up's last month).

Reckonings made over the island and not for one town read the island's wage under either word: the promoters
of roads, canals, railways, bridges and trunk roads, the port's railway planner, and the years' books of fares.

**What is no longer read.** With the switch on, `longrun.freight_from` and the landfall start's
`FREIGHT_FROM` are not read: `longrun::freight` returns the hands at the island's wage on its first line. The
railway's station charge (`haulage.handling`, [haulage.md](haulage.md)) needs no year: it is laid only on a
train's edges, so laying it from 1750 is laying it from the first railway. A road's wear by its own traffic
keeps its year, 1901.

**Who reads carriage.** "Coins" is the old freight (fixed coins on the settled island to 1900, then the cost
of living; the cost of living from 1750 at landfall); T is the hands at the town's wage under `town`, the
island's under `island`; I is the hands at the island's wage under either.

| reckoning | where | before | in hands |
|---|---|---|---|
| a works' gate price: its output carted to market | `econ.rs`, `pout` | coins | T, the works' town |
| the trade between towns: a merchant's carriage | `econ.rs`, `tau` | coins | T, the town carried from, whose bill it is |
| the land's delivered prices of its inputs | `land::delta_fields` | coins | T, the cell's market |
| the land's bids for a cell | `land::sector_bids` | coins | T |
| the diggings' carriage | `land::update_land_use`, `ex_tau` | coins | T, the cell's market |
| a works' choice of site | `firms::evaluate_sites` | coins | T |
| a pit's review | `mines::review` | coins | T |
| **the founding test**: the carting a market here would save, against the district's wages | `peopling::candidates` | coins | T, the town the district leaves |
| the prices a district takes along at the year's turn | `peopling::prices_go_with_trade` | coins | T |
| a new town's expected prices | `prices::found_town` | coins | T, the mother town |
| **roads**: a turnpike's case, and the toll paid | `roads.rs` | coins | I |
| **canals**: the year's fares | `canals.rs` | coins | I |
| **railways**: carting saved, local and by trade; the year's fares | `ways.rs` | coins | I |
| the port's railway planner, lines and spurs | `rail.rs` | coins | I |
| **bridges**: traffic times the crossing's charge | `crossings.rs` | coins | I |
| **trunk roads**: wear and bypasses | `highways.rs` | coins | I |
| the station charge, in the path's cost | `haulage::on` | from the close of 1901 | from 1750 |
| a town's carters | `carting::take_carters` | bill over the town's wage | the same; under `town`, the hands the loads took |

Under `town` the founding test reads the town the district would leave: it asks whether a market here would
free enough of this district's hands from carting, and the coins then cancel to hands. (The verdict had it on
the island's wage; under `island` it is.)

**Services are not kept** (`prices.spoils_unkept`; [prices.md](prices.md), "A good that spoils is not
stocked"). A rule reading the good's decay against `prices::SPOILS`, not a line of the content, so that the
switch can turn it, and a switch of its own, so that the flip can weigh it alone: the probe found it mends
main too.

## What it did

Islands 1 to 8, six families, to 2050, both starts, with the three switches on (`carriage.in_hands`,
`prices.exports_ask`, `prices.spoils_unkept`), the two wages and the founding test's number (`people.found_saving`)
at 0.06, 0.08 and 0.10: 96 runs, **none failed**. Year by year with `examples/carriage_probe.rs`; the written check
with `sweep --standing`. Main is the same islands with all three off.

| settled start | main | in hands, the town's wage, 0.08 | all six arms |
|---|---|---|---|
| small poor towns (under 2,000 people at under a fifth of the island's wage), most in a decade, an island-year | 7.7 (1990s) | **0 in every decade** | 0 in every decade |
| largest rise of a town's wage in a year, to 1900 / after | 3.28 / 27.0 | 1.33 / 1.90 | 1.26-1.33 / 1.89-2.69 |
| years with hunger an island, to 1900 / after | 0 / 0.38 | 0 / 0 | 0 / 0-0.38 |
| a town's prices over its city's, least, to 1900 / after | 0.32 / 0.0075 | 0.46 / 0.20 | 0.33-0.57 / 0.19-0.26 |
| hands a 1,000 ton-cells as the crow flies, 1750s / 1890s / 2040s | 0.122 / 0.019 / 0.0046 | 0.121 / 0.053 / 0.032 | |
| carters' share of hands, 1750s / 1890s / 2040s | 0.118 / 0.091 / 0.040 | 0.126 / 0.203 / 0.148 | |
| merchants' share of income, 1790s / 1890s / 2040s | 0.061 / 0.048 / 0.041 | 0.034 / 0.013 / 0.015 | never below nothing |
| towns in 1900 | 21.6 | 32.8 | 29.9-36.6 |
| the written check, met of 29 | 18 | 19 | 16-19 |

| landfall | main | in hands, the town's wage, 0.08 | all six arms |
|---|---|---|---|
| small poor towns, most in a decade, an island-year | 2.9 (1990s) | **0 in every decade** | 0 in every decade |
| largest rise of a town's wage in a year, to 1900 / after | 1.66 / 12.9 | 1.33 / 1.61 | 1.28-1.42 / 1.61-2.24 |
| years with hunger an island, to 1900 / after | 0.13 / 0 | 0.13 / 0.38 | 0.13-0.63 / 0.13-0.5 |
| hands a 1,000 ton-cells, 1750s / 1890s / 2040s | 0.107 / 0.0073 / 0.0011 | 0.157 / 0.051 / 0.025 | |
| towns in 1900 | 16.1 | 27.5 | 25.9-30.9 |
| the written check, met of 25; the colony's 24 | 12; 14 | 12; 14 | 11-12; 14-15 |

A ton carried a cell now takes the same hands on both starts (0.053 and 0.052 a 1,000 in 1900), and 1800's over
1750's lies in the written band (0.73, 0.70 to 0.90; main 0.57). The written check's tables for every arm, and the
probe's by decade, are in the build notes (`.claude/night/reports/go/carriage-in-hands/`).

**Whose wage.** The town's is the truer, and the default: a town's carters are then the hands its loads took, and
on no line does the island's wage do better. The town's wage met 19, 18 and 18 of the settled written check at the
three numbers, the island's 18, 18 and 16; at landfall 12 against 11; the colony's years of hunger to 1900 were
0.13 to 0.25 against 0.38 to 0.63; and the island's wage let one wage rise after 1900 past 2.0 (2.03, at 0.10).

**Each switch alone** (the town's wage, 0.08; settled, then landfall). Carriage in hands alone keeps the pockets
(5.9 and 8.8 an island-year in the 1990s) and the leaps of a town's wage (10.6 and 15.3 after 1900), and has 2.4
hungry years an island after 1900 on the settled start. With `exports_ask` beside it the pockets are gone, but the
merchants' share of income is below nothing from the 1870s on (-0.031 after 1900, settled). With services not kept
as well it stays above nothing in every decade (0.012 settled, 0.020 landfall, after 1900).

**What the brief's pass asked.** With the switches off no golden file moved, and the game's sweeps on both starts
show only the three seat facts that always read 0/0. With the three on, no run failed on islands 1 to 8 on either
start, and there is no small poor town in any decade on either start, at any of the three founding numbers or
either wage.

**Out of their bands with the three on** (the written check, every arm): towns in 1900 (30 to 37 settled, 26 to 31
landfall; band 10 to 25), the carters' share of hands in 1900 (0.20 on both starts; band 0.05 to 0.12), the
townsfolk's share in 1900 (0.38 to 0.39; band 0.40 to 0.60), people in 2050 over 1900 on the settled start (3.0 to
3.3; band 1.5 to 2.3). Higher founding numbers bring the towns in (0.15: 24.1 and 20.0) and nothing else. **Half the
hands a load takes** (`carriage.hands = 0.00167`, at 0.08) brings five in at once on the settled island (23 of 29
met: towns 24.6, people 2050 over 1900 2.01, townsfolk's share 0.418, the least price 0.72, grain's gap between towns
0.115), and at landfall 16 of 25, the colony's 16 and no hunger in either era; carters are then 0.144 of hands in
1900. The 0.015 coins the hands were taken from was the port's coin, not a number from history: the hands a load
takes is the number the flip of the first century should fit, before the founding number.

**The lorry after 1905.** Where the lorry is had, an hour of carting takes 0.55 to 0.77 hands on the settled island
by the 2040s, and 0.38 at landfall, but the hands a load takes over a unit of path fall only a sixth from the 1910s
to the 2040s on the settled island (0.273 to 0.230) and a fifth at landfall (0.276 to 0.214). The bill stays the
load's hands at the wage; the lorry buys its work at hands times the wage plus plant and oil, and so moves it from
hands to plant and oil without saving hands (`carting.rs`). The next piece, the lorry's hands, mends it.

## What goes when it is on for good

Under the game's rules (the port keeps `freight` and reads it as ever):

- `longrun.freight_from`, `LongrunState::freight_base`, the anchor's branch of `longrun::freight`, and the two places
  `freight_base` is set (`longrun::start`, `longrun::note_month`); "Why the freight waits for 1900" in
  [longrun.md](longrun.md) becomes history. Their cases in `tests/longrun.rs` go with them.
- `landfall::FREIGHT_FROM` and its line in `landfall::landfall`; "Carriage that follows prices" in
  [landfall.md](landfall.md); the case in `tests/landfall.rs` that holds it.
- The station's year: `st.hl.year >= FROM` in `haulage::on`. `haulage::FROM` stays for a road's wear.
- `freight` is no longer the game's carriage. What still reads it: the port, the record's own note of it
  (`record.rs`), the rulebook's pinned number (it should pin `carriage.hands`), the inspector's "carting grain
  there" (`inspect.rs`, which reads the port's coin even today), and two examples (`loose_ends.rs`).
- Whichever of `carriage.wage`'s two words is not chosen, and with it `carriage::Wage` and the fall-back of
  `freight_at` to `longrun::freight`.
- With `exports_ask` and `spoils_unkept` on for good: the old flow term (the other arm of `prices::asked`) and the
  stocked-services arm of `tree::price_gap`.

## What worries me

- **The hands a load takes are the port's coin in disguise.** 0.015 coins at the settled wage of 1750 was a choice
  of the study, not a reading of history, and the carters' share of hands (0.20 in 1900 against a band of 0.05 to
  0.12) says it is about twice too many. Half of it brings five written facts in at once on the settled island. I did
  not change it: the brief names the study's number, and the flip should fit it once, beside the lorry's mend.
- **The carters' share rises from 1750 to 1900 under any number** (0.08 to 0.14 at half the hands), because a richer
  island carries more weight a head, and only a better way lowers a load's hands.
- **The founding number is not monotone on eight islands** (towns in 1900 at 0.12, 0.15 and 0.20: 25.1, 24.1, 19.8,
  with hunger coming and going). Sixteen islands, as the verdict asks of the fitter, would tell.
- **A town's wage of the month** is what its bill reads under `town`, and the reckonings a district makes at the
  year's turn read the wage before the turn sets it again.
- **The station charge from 1750** moves nothing before the first railway, as it must; I did not run the rule with
  the charge kept at 1901 to see how much of the seam at 1901 it was.
- **What stays out with the three on**: the second town's townsfolk over the first's (0.85), grain's gap between
  towns after 1900 (0.35, band 0.15), and a town's least price over its city's after 1900 (0.2 to 0.3, band 0.67).
  What is left of a town's gap to its city is the cart, and the cart is the hands.

