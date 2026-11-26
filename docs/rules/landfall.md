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
guard on a short town's wage (`hands.short_rise` 1.06). Each can be set off
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

### Land that stays taken, and no settlers to a hungry port

Two of the mends the study of the start ordered on 26 November come with
`--start landfall` (they sit in `FrontierParams::game` and
`ImmigrationParams::game`): land once taken stays taken
(`frontier.claimed`, [frontier.md](frontier.md)) and no ship brings settlers
to a port that went hungry that year or the four before
(`immigration.shun_hungry`, [immigration.md](immigration.md)). Measured with
carriage on prices from 1750 (`--set longrun.freight_from=1750`), islands 1
to 8 to 2050, means: with the two mends the free share is 0.03 in 1950 and
0.04 in 2050 (without them 0.32 and 0.65); people are 493k in 1900 and
1,731k in 2050 (558k and 2,338k), growing 0.28 per cent a year from 2000 to
2050 (0.36); hunger falls to 0.6 years an island after 1900 (1.0) and rises
to 1.6 before it (1.0); the wage in grain is 14.6 / 30.2 / 101.1 in 1800 /
1900 / 2050 (14.3 / 32.3 / 92.5). With settlers off as well the colony has
168k people in 1900 and 684k in 2050. No run failed.

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
it is the pass over the rules' stage 6.

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
