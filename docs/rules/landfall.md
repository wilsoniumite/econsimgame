# Landfall: one town in a corner

A second start for the island. Not in the Python. The code is `landfall.rs`
(the start), with `frontier.rs` ([frontier.md](frontier.md)) and
`immigration.rs` ([immigration.md](immigration.md)) for the rules that come
with it, and a lesser number of the hands' rule, `hands.short_rise` (below,
"The trap"). Built on 26 November 2026 from the trial of that morning.

## The two starts

- **The settled island** is the start every run has had: six towns where the
  land's makers put them, 135,000 people shared among them by the fertile
  land round each, the woods thinned round every one. It is the start under
  both sets of rules, and nothing on this page changes it.
- **Landfall** is the same island, made from the same streams, with one
  town of 5,000 settlers at a harbour near one of its corners, and nobody and
  nothing else: the woods are thinned round that town alone. A ship's
  landfall is its first sight of land and its coming ashore.

How to ask for it:

| where | settled | landfall |
|---|---|---|
| `isostate-core run` and `sweep` | nothing, or `--start settled` | `--start landfall` |
| the sweep, the start alone | | `--set start.landfall=true` |
| the viewer, a new game | `--new` | `--landfall`, or **New game from landfall** on the card of a game gone on with |
| code | `game_params()` | `landfall::set_start(&mut P, "landfall")` or `landfall::landfall(&mut P)` |

`--start landfall` is the start with the three rules that make a colony a
colony: births that answer to free land and hamlets (`frontier.enabled`,
`frontier.hamlets`), settlers from abroad (`immigration.enabled`), and the
guard on a short town's wage (`hands.short_rise` 1.06), and with carriage
that follows prices from 1750 (`longrun.freight_from` 1750, below,
"Carriage that follows prices"), and with pits and quarries that work at
the size of their market (`mines.glut`, below, "Pits at the size of their
market"). Each can be set off
again after it: `--start landfall --set immigration.enabled=false` is the
colony with nobody coming. `--set start.landfall=true` alone is the one town
under today's rules of people. A saved game says its start
(`Save::landfall`, left out of the file for the settled island) and is played
again from it. The start is fixed when the island is made, so the Rules tab
offers it as nothing to change (`edits::FIXED_AT_START`).

### Where the town stands

`landfall::site`: the land's least box has four corners; the islands are
round, so a true corner is sea. A coastal cell is near a corner when it is
within 12 cells (24 km) of the nearest coastal cell to that corner. Among
those, on land below the mountains and three cells from the map's edge, the
town stands on the one the makers' own rule for a town scores highest,
without its noise: the fertile land within five cells, navigable water
beside it, a river at the coast, and on the new land the places towns grew
(the head of an estuary, a confluence). Nothing is drawn, so the island is
the same island to the bit; the makers' own sites are replaced after it is
made. On islands 1 to 8 the town stands at (25, 24), (19, 62), (103, 36),
(16, 60), (25, 23), (99, 59), (89, 58) and (33, 18).

### Why 5,000

A colony some years after its first ships, as Pennsylvania was about 1685
and South Australia about 1838 (from memory), big enough to keep a market of
its own. The trial of 26 November ran one town of 5,000 to 2050 on three
islands with no run failing; fewer would leave the founding rule's districts
and the works' table too thin to say anything.

## What the runs showed

Islands 1 to 8, the game's rules with six families, to 2050, with
`examples/landfall.rs` (a line of JSON a year, maps every 25 years). Three
arms of landfall, which differ only by their switches, and the settled
island beside them:

- **A**, births as today: `--start landfall --set frontier.enabled=false --set immigration.enabled=false` (hamlets and the guard on);
- **B**, births that answer to land: `--start landfall --set immigration.enabled=false`;
- **C**, births and settlers: `--start landfall`;
- **S**, the settled island: `--start settled`.

Means of the eight islands, least and greatest in brackets:

| | A | B | C | S |
|---|---|---|---|---|
| people 1800 | 9.7k | 18k (17k to 18k) | 44k (36k to 59k) | 202k |
| people 1850 | 19k | 57k (54k to 60k) | 140k (119k to 185k) | 364k |
| people 1900 | 38k (38k to 40k) | 147k (133k to 160k) | 311k (253k to 413k) | 680k |
| people 2050 | 230k | 1,619k (1,210k to 1,854k) | 3,974k (2,825k to 6,892k) | 1,301k |
| births less deaths a year, 1751 to 1800 | 0.013 | 0.025 | 0.041 with settlers | 0.008 |
| towns 1800 / 1900 / 2050 | 4.9 / 13.5 / 25.6 | 6.4 / 25.0 / 42.9 | 8.9 / 33.0 / 46.5 | 12.6 / 21.6 / 29.6 |
| second town founded | 1750 on every island | 1750 | 1750 | (six from the start) |
| land with 5+ hands a cell, 1800 / 1900 | 0.00 / 0.01 | 0.01 / 0.25 | 0.07 / 0.38 | 0.62 / 0.89 |
| wage in grain 1800 / 1900 / 2050 | 16.7 / 15.8 / 20.4 | 12.9 / 9.1 / 25.8 | 9.0 / 9.4 / 26.8 | 10.2 / 37.7 / 78.3 |
| first railway | 1982 (1965 to 2000) | 1913 (1900 to 1939) | 1890 (1869 to 1907) | 1828 (1815 to 1848) |
| works 1900 | 367 | 674 | 1,010 | 1,220 |
| under 15, share of people, 1800 | 0.354 | 0.42 to 0.43 | 0.38 to 0.41 | 0.35 |
| settlers to 1850, share of the growth | | | 0.20 to 0.28 | |
| years with hunger, to 1900 / after | 1.0 / 0.5 | 1.9 / 5.5 | 1.5 / 15.6 | 0 / 0.4 |
| largest rise of a town's wage in a year, to 1900 | 1.97 | 2.06 (over 2 on 2 islands) | 2.20 (over 2 on 3) | 3.28 (over 2 on 1) |
| runs failed or losing money | 0 of 8 | 0 of 8 | 0 of 8 | 0 of 8 |

"Wage in grain" is the island's mean wage over the national price of grain,
comparable between arms; "land with 5+ hands" is the share of land cells
with five hands or more at work on them (about 2.5 people a km2, with half
the people at work), which the trial found the best map of settlement.

What this says:

1. The landfall start runs: 32 runs to 2050 and none failed its money
   check.
2. Births that answer to free land give the colony history's pace, 2.5 per
   cent a year from 1751 to 1800 on every island (Franklin's doubling in
   about 25 years), against 1.3 with births as today. They do not give
   300,000 people by 1900: 133,000 to 160,000. From 5,000 that would take
   2.7 per cent a year for 150 years without a pause, and the colony's
   births fall as its towns' country fills (to 1.7 per cent by 1900, B on
   island 1). Births and settlers together give 253,000 to 413,000, 311,000
   on the mean.
3. The second town is founded in the first year on every island (hamlets,
   [frontier.md](frontier.md)), at 21 to 112 km from the first, and by 1850
   the colony has 14 to 20 towns. The maps show what the designer asked
   for: worked land spreading from the corner and from a second focus,
   towns founded behind it.
4. The colony is richer than the settled island in its first fifty years
   with births as today (16.7 against 10.2 in grain in 1800) and poorer by
   1900 in every arm (9 to 16 against 38). Its first railway comes 60 to 150
   years after the settled island's: too few people, too far apart.
5. Past 1900 the colony with free-land births passes the settled island's
   people (1.6 million against 1.3 in 2050), and with settlers it has four
   million, with hunger in five to fifteen years of the century after 1900.
   The free-land lift is weighed by the share of hands on the land, which
   halved it from 2.1 million (the first runs); what is left is a poor,
   crowded island whose births the transition does not slow, since it reads
   how well townsfolk live.
6. Settlers bring 20 to 28 per cent of the colony's growth to 1850, and stop
   about 1820 to 1850 as the living abroad overtakes the island's.
7. The ages carry the settlers: the colony's children are 0.42 of its
   people in 1800 with births that answer to land (history: about half in
   the United States of 1800, from memory), against 0.35 on the settled
   island; settlers, who are young adults, take it to 0.38 to 0.41.

## The trap

The trial held the land beyond 20 km of the town empty through the warm-up
and let it go in 1751: the wage went up twenty to thirty times in a year.
It suspected the freight, a fixed sum of money before 1900. Traced on island
7 at landfall with births as today and no settlers
(`examples/landfall_wage.rs`, months of 1750 to 1755 of the first town), the
cause is three rules meeting, and the freight is one of them:

1. **Carters are the carting bill over the wage** ([carting.md](carting.md)):
   last month's bill, in money at a freight fixed in money, divided by the
   town's wage, is the hands on the road. A market that reaches the whole
   island carts its produce 100 km and more: the first town's carters were
   476 of its 2,500 hands in January 1750, and 690 by May 1751. When the
   wage falls (6.8 by December 1753, from 10) the same bill takes more
   hands.
2. **The land's hands move 15 per cent of the way a month**
   (`labor_adjust_ex`): this month's demand is at least 85 per cent of last
   month's hands, whatever the wage.
3. **The hands' rule holds a short town at the month's highest wage**
   ([hands.md](hands.md)): twice last month's. In January 1754 the town had
   2,009 hands, 556 of them carting, 40 sinking a pit, and 1,590 carried on
   the land and in the works from December: short at any wage the month
   allowed, so the wage was set at twice December's, 13.7, and in February
   at twice that, 27.4, and in March 36.7. Its prices followed, its works'
   hands fell from 771 to 108 as they priced themselves out, and the
   merchants, who pay the carting and buy the produce whatever their cash,
   went from 14,000 below nothing to 560,000 below nothing (the island's
   money is 120,000).

Letting held land go at once makes all three bite in the same month: the
whole island joins the market, the bill jumps, the carters take the town's
hands, the town is short, and its wage doubles each month it stays short,
which in four or five months is twenty or thirty times.

**The guard.** `hands.short_rise`: the most a short town's wage may rise in
a month. At 2, the month's own bound, nothing is read and the rule is as it
was, so the settled island is the same to the bit. The landfall start sets
1.06, twice in a year, and the rationing alone shares out the hands. With
it, the first town of island 7 never doubles in a month; the colony's
largest yearly rise of a town's wage to 1900 is 1.97 with births as today
and 2.06 to 2.20 on five of sixteen runs with the other two rules, which are
clearing wages and not the guard's (on the settled island itself one island
of eight has 3.28, in 1786). **The mend** is for the piece that opens land
year by year: a market whose reach grows with its charting does not cart
from 100 km in its first year. Freight that follows the wage
(`longrun.freight_from`) would stop the carters swelling as the wage falls;
it is the pass over the rules' stage 6. Since 26 November the landfall start
carts at prices from 1750 (next section), which is a step toward it.

## Carriage that follows prices

**A stopgap.** The landfall start sets `longrun.freight_from` to 1750
(`landfall::FREIGHT_FROM`); the settled island keeps 1900. That makes the two
starts differ in a rule, which the house does not want, and it stays only
until the truer rule below is built for both. `--start landfall --set
longrun.freight_from=1900` puts the colony back as it was.

**Why.** Until `freight_from` a cart-load costs a fixed number of coins
([longrun.md](longrun.md)), and carters are the carting bill over the wage
([carting.md](carting.md)). On the settled island that does little harm,
because its money and its people grow at about the same pace. The colony's
people grow 3 to 4 per cent a year and its money 1.75 per cent
([money.md](money.md)), so its prices and its wage in coins fall: the mean
wage from 9.5 coins in 1750 to 1.0 in 1800 and 0.5 in 1900 on islands 1 to 8.
At a freight fixed in coins a load then costs the colony six to nine times
the hours it costs the settled island, and a quarter of its hands cart, for
a century and a half. Found by the study of the landfall start (26 November
2026), whose finder ran this very setting on islands 1 to 8 (its numbers
came out the same to the last digit on this branch).

**What it did.** Islands 1 to 8, the game's rules with six families, to 2050,
`examples/landfall.rs`; "before" is `--start landfall --set
longrun.freight_from=1900` on the same tree. Means, least and greatest in
brackets. The wage in grain compares runs; the linked wage is each run's own
path, 1750 = 1.

| | landfall before (freight from 1900) | landfall now | now, no settlers | the settled island |
|---|---|---|---|---|
| people 1800 / 1900 / 2050 | 44k / 311k / 3,917k | 55k (44k to 64k) / 558k (448k to 679k) / 2,338k | 19k / 176k / 877k | 202k / 680k / 1,266k |
| wage in grain 1800 / 1850 / 1900 / 2050 | 9.0 / 7.3 / 9.3 / 26.9 | 14.3 (12.7 to 15.4) / 10.7 / 32.3 (27.8 to 38.4) / 92.5 | 18.3 / 14.8 / 12.4 / 100.1 | 10.2 / 15.9 / 37.7 / 78.1 |
| linked wage 1900 / 2050 | 1.11 / 2.56 | 2.33 / 8.99 | 1.45 / 9.23 | 6.05 / 15.89 |
| carters over all hands, 1800 / 1900 | 0.27 / 0.21 | 0.12 (0.11 to 0.13) / 0.05 | 0.12 / 0.07 | 0.11 / 0.09 |
| money a head, coins, 1800 / 1900 | 6.3 / 5.2 | 5.1 / 2.9 | 14.7 / 9.0 | 36.7 / 62.7 |
| price level 1800 / 1900 | 0.18 / 0.093 | 0.084 / 0.022 | 0.23 / 0.10 | 0.76 / 0.32 |
| first railway | 1890 (1869 to 1907) | 1858 (1846 to 1877) | 1892 (1869 to 1906) | 1828 (1815 to 1848) |
| towns 1900 | 33.0 | 17.9 | 20.0 | 21.6 |
| years with hunger, to 1900 / after | 1.5 / 17.4 | 1.0 / 1.0 | 1.5 / 3.2 | 0 / 0.4 |
| merchants' debt over the money, worst December to 1800 | 3.40 (0.40 to 6.84) | 3.54 (0.40 to 6.73) | 3.16 (0.40 to 6.18) | 0.01 (0 to 0.03) |
| largest yearly rise of a town's wage to 1900, `short_rise` 1.06 | 1.91, over 2 on 3 | 1.78, over 2 on 1 | 1.76, over 2 on 1 | 1.74 at 2, over 2 on 1 |
| the same at `short_rise` 2, and its hunger to 1900 / after | | 2.03, over 2 on 3 (island 7 3.64); 0.5 / 0.1 | 1.99, over 2 on 2; 0.5 / 0.4 | |
| runs failed or losing money | 0 of 8 | 0 of 8 (and 0 of 8 at 2) | 0 of 8 (and 0 of 8 at 2) | 0 of 8 |

"Carters over all hands" counts the works' hands, the land's and the
carters in December; "the price level" is the chained cost of living,
the end of the warm-up = 1; "the merchants' debt" is the merchants' purses
below nothing, summed over the towns, in December, over the island's money.
The largest rise of a town's wage reads each town's mean wage over its last
year's, for towns past their first two years.

What it says:

1. Carriage on prices takes the colony's carters from about a quarter of its
   hands to 0.12 in 1800 and 0.05 in 1900, below the settled island's 0.09.
   With the carters back at work the colony is richer than the settled
   island in its first fifty years (14.3 in grain in 1800 against 10.2),
   finds and builds its railway in the 1850s, and is as rich as the settled
   island by 1900 (32.3 against 37.7) and richer by 2050.
2. Its prices fall all the same: the price level (the chained cost of
   living, end of the warm-up = 1) is 0.08 in 1800 and 0.02 in 1900, so the
   freight that rides on it falls with it. A ton-kilometre takes fewer hands
   every decade whatever the roads are; that is the stopgap's flaw (below).
3. Hunger after 1900 goes from 17 years in 150 to 1, and the colony of 2050
   has 2.3 million people, not 3.9.
4. The colony founds its towns early and stops: 16 by 1850 and 18 in 1900,
   where the colony carting in coins went on to 33. The founding rule
   wants a district whose carting saved is 8 per cent of its wages
   ([people.md](people.md), `found_saving`), and cheap carriage saves
   little. The settled island with the same setting stops founding in 1823
   (below).
5. Without settlers (`--set immigration.enabled=false`) the colony is
   richer longer and smaller: 18.3 in grain in 1800, then 14.8 and 12.4 in
   1850 and 1900 as its first railway waits for 1892 on the mean; 176,000
   people in 1900 and 877,000 in 2050.
6. The merchants still owe several times the island's money in the first
   years: the worst December to 1800 is 0.4 to 6.7 of the money (island 7
   6.73), in more than ten Decembers on five islands of eight. That is the
   pits' fault, the next piece's.
7. The largest rise of a town's wage in a year to 1900 is 1.78 on the mean
   with the guard (`hands.short_rise` 1.06), over 2 on one island of eight
   (island 2, 2.19). At the game's own 2 it is 2.03, over 2 on three (island 7,
   3.64), and the colony goes hungry less (half a year to 1900 and 0.1 after,
   against 1.0 and 1.0): the guard's question, the piece after next.

**The settled island with the same setting**, once, islands 1 to 8 to 1900
(`sweep ... --set longrun.freight_from=1750 --standing --compare` today's):
ten of the twelve standing facts are inside, and two are not. Towns in 1900
fall from 21.6 to 13.6 on every island (the last founded in 1823, not 1890),
and the railway in 1900 from 2,692 km to 2,086, with half the trunk lines.
The people, the share in towns, the first railway (1829 against 1828) and
the shares of income stay; the chained wage of 1900 over 1750 is 6.32
against 5.96, and carters over hands 0.046 against 0.091. It is
[longrun.md](longrun.md)'s table again, on today's island. Run on to 2050
with the probe, the settled island with the setting is richer and smaller:
the wage in grain 128 against 78 and the linked wage 20.5 against 15.9 in
2050, a million people against 1.27 million, and 15 towns against 30. So the setting
cannot simply go to both starts: it would take half the settled island's
towns.

**What the colony is still short of**, set against the settled island in
1850 and 1900 (the means of the table's runs; a ratio is the colony's over
the settled island's):

1. **Money.** 3.0 coins a head against 48.8 in 1850, 2.9 against 62.7 in
   1900 (0.06 and 0.05), and so prices at a fifteenth of the settled
   island's. Counted in grain the gap is a third (67 against 104 grain a
   head, 189 against 274). New money is issued on the money, not on the
   people, so a colony that grows faster deflates; carriage on prices takes
   the worst of it out of carting, and nothing else.
2. **The railway.** 25 km against 1,152 in 1850, 1,664 against 2,692 in
   1900, the first line thirty years later. The idea is counted in rails and
   machinery made on the island since 1750, which a small colony makes
   slowly; ideas by sea are the piece for it.
3. **Output and the wage.** Output a head (the series `gdp_pc`) is 0.97 of
   the settled island's in 1850 and 0.71 in 1900; the wage in grain 0.67 and
   0.86. The linked wage, each run against its own 1750, is 1.32 against
   2.89 and 2.33 against 6.05: the colony starts richer in grain (11.7
   against 5.7 in 1750) and gains less on it.
4. **People, works and ways.** People 0.60 and 0.82 of the settled
   island's, works 0.69 and 0.86, made roads 0.58 and 0.83; the colony
   grows faster (its towns' rate of increase, 2.1 per cent a year in 1850
   against 1.2) and is closing on it.
5. **Towns and settled land.** As many towns in 1850 (15.9 against 16.2)
   but almost none founded after about 1870, so 17.9 against 21.6 in 1900;
   the land with five hands a cell 0.53 against 0.79 in 1850, and alike by
   1900 (0.81 against 0.89).

Beside these the merchants' debt, three and a half times the money in the
worst December before 1800 on the mean, is the colony's own and the next
piece's (the pits).

**The truer rule: carriage costs the carters' wage.** Carting takes hands by
the load and the distance, and costs their wage, from 1750 on both starts.

- *What it is.* A unit of weight carried over a unit of path takes `h`
  hands for a month; the carting bill is those hands times the town's wage,
  and the carters are those hands. Then the carters follow the loads and
  the ways, not the wage: they neither swell when the wage falls (the trap)
  nor melt as prices fall (the stopgap), and roads, canals and rails, which
  shorten the path, are what give hands back. `h` would be set so that the
  settled island of 1750 carts with the hands it has now: the freight over
  its mean wage of 1750, about 0.015 over 4.5 coins.
- *What it would need.* The rule already exists from the lorry on
  (`tasks::cart_hiring`, [lorry.md](lorry.md)): a unit of the carting's work
  takes `cart_hands` hands and costs `cart_hands` times the wage plus
  `cart_cost`, and the carters are the work times the hands. Carriage that
  costs the carters' wage is that rule for the horse and cart from 1750: the
  freight in the trade between towns, in a market's carting of its produce
  and in a works' inputs (`econ::step_month`, where `tau` is reckoned) would
  be `h` times the paying town's wage, where it is now `longrun::freight`;
  `carting::take_carters` would count the work and not divide the bill by
  the wage. The horses and their fodder, which a cart-load takes too, would
  be the lorry's `cart_cost`: a part of the bill paid to the land for oats
  and hay, if the content had them.
- *Where the hands would come from.* From the town whose market the load
  belongs to, as today (`carting::add_bill` charges the payer's town): a
  farm's produce to its market town, a works' inputs and output to the
  works' town, the trade between two towns to the one that sends. A truer
  form would share a long haul's hands among the towns along its path, so
  that the carting of a far district fed the villages it passes; that needs
  the path's towns, which the cost graph's search has and does not keep.
- *What it would move.* On the settled island the coin wage rises from
  about 4.5 to 7.5 by 1850, so carriage would grow dearer in coins and the
  hands a ton-kilometre takes would stay at 1750's until the ways change;
  today they fall a quarter by 1900 (the sweep's
  `carters_per_1000_weight_path`, 0.206 in 1800 to 0.154 in 1900). More
  carters, dearer carting, and so more saved by a town or a line: the
  settled island's first century would move, in the direction opposite to
  the stopgap's. It waits for the stage that lays that century anew.

## Proposed standing facts for landfall

For the next piece, which makes the two starts equals in the tests. Bands
from history (from memory), not from the runs; set beside them, what the
arm C runs gave. Not tuned to.

| fact | band | why | arm C, islands 1 to 8 |
|---|---|---|---|
| births less deaths a year, 1751 to 1800 (B) | 0.020 to 0.030 | Franklin; French Canada 2.5 per cent | 0.025 (B) |
| growth a year with settlers, 1751 to 1800 | 0.025 to 0.045 | British North America about 3 per cent, Pennsylvania more | 0.039 to 0.049 |
| settlers' share of the growth to 1850 | 0.10 to 0.35 | colonial America: net migration a fifth to a quarter of growth | 0.20 to 0.28 |
| second town founded by | 1780 | Massachusetts Bay founded its first towns within a year of landing | 1750 |
| children under 15, share in 1800 (B) | 0.40 to 0.52 | about half in the United States of 1800 | 0.42 to 0.43 (B) |
| wage in grain, landfall over settled, 1800 (A) | 1.1 to 2.0 | colonial incomes above Britain's (Lindert and Williamson) | 1.6 (A) |
| years with hunger after 1760, to 1900 | 0 to 1 | colonies starved in their first years, seldom after | 1 to 2 |
| largest rise of a town's wage in a year, to 1900 | under 2 | the brief | 1.62 to 2.20 |
| first railway by | 1880 | United States 1830, Canada 1836, Australia 1854, New Zealand 1863 | 1869 to 1907 |
| people per town, 1850 | 3,000 to 15,000 | a market town and its country | 5,600 to 8,400 |
| runs failing or losing money | none | the convention | none |

## What charting will need from this start

- **The first town's market reaches the whole island in 1750.** The land
  hands of 5,000 people spread far (on island 7 nine in ten work within
  98 km of the town in 1750 and 110 km by 1800), and a quarter of the
  town's hands cart. A market area
  that grows with its charting is what makes the colony's country a country
  and not a thin film over the island.
- **The trap above.** Opening land a year at a time keeps the carting bill
  from jumping; `hands.short_rise` stays as a guard.
- **Free land measured by charting.** The frontier's free share counts
  thinly worked land as free; charted and unclaimed land would be the honest
  measure ([frontier.md](frontier.md), "What worries me").
- **The sites.** `landfall::site` and the hamlets' founding are what places
  the colony's towns; a charted frontier would hand the founding rule only
  charted districts (its candidates already need a market).
- **What a square would hold**, as the trial listed: charted or not and
  when, and from which town; per deposit, known or not; who holds it.

## What worries me

- The colony's merchants owe several times the island's money within three
  years of landfall: they pay the carting and buy the produce whatever their
  cash ([prices.md](prices.md) and `docs/probes/tr-loose-a.md` have the
  same on the settled island at a smaller scale). The money check holds,
  since every penny is counted, but the colony's prices treble by 1754.
- After 1900 a landfall island grows past the settled one in people and
  stays poorer, with more hunger. That is not history's frontier, which
  grew rich; it is the island's transition reading how well townsfolk live.
- One town is a single market for the whole island in 1750; every early
  number above is that market's.
