# The written check: the island held against history

The standing check's twelve facts (docs/rules/README.md, "The standing check")
have no bands of their own. A fact is "inside" when it lies between the least
and the greatest that a reference sweep gave, and the gate takes a new reference
at every landing. So the twelve can say that the island moved, never whether it
moved towards the world or away from it. The written check is a ruler that does
not move with the island: for each fact a band taken from history, with its
source and how sure the source is, the same for both starts where history
allows. It was written on 27 November 2026 from the study of how 1750 to 1900 is
to be laid out anew (the study's verdict, section 5, and its reader of the bands;
both are kept with the night's reports, not in the repository), before any rule
of that work moves the island.

It changes no run. It is printed by every sweep with `--standing`, first, and
written to `standing.txt`; it does not yet fail the sweep or the gate. A later
piece makes it the gate, as a ratchet: a landing may not lose a fact that main
meets, and a fact main misses may not end further from its band, unless the
landing names it with its reason.

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --start settled,landfall --end-year 2050 --standing --out DIR --quiet
```

The code is `standing::WRITTEN` (the table) and `standing::written` (the
print), in `crates/isostate-core/src/standing.rs`.

## How it reads

Each fact is read island by island, and is **met** when the mean of the islands
lies in its band, as the colony's check reads its facts. The table prints the
mean, the least and greatest of the islands, the band, how many islands lie in
it, one word, how sure the band is, and its source. Where the old twelve have a
reference from an earlier sweep (`--compare`), its mean and word stand beside
each fact.

The lines come in three kinds, printed apart:

- **facts with a band, counted**: met or MISSED, and the first line counts
  them: `the written check: N of M met, K missed, J without a value`. A sweep that
  ends in 1900 has no value for the facts after 1900.
- **unlike England's**: printed with their band and one word, and not counted
  against a landing. The study knew main misses these and that the first moves
  will not mend them (the settled island's wage, above all); they stay printed
  so that nobody forgets them.
- **printed, no band yet**: a number and its reason for having no band.

A landfall setting also gets a line with the count of the colony's check
([scorecard.md](scorecard.md), "The colony's check"), which is printed in full
further down; the study asks that a landing keep at least main's 14 of 24, or
name each loss.

## How sure, and what "from memory" means

- **read**: the reader of the bands read it that day in the source named.
- **memory, high / medium / low**: from memory. The reader (or the judge) knew
  the number and did not check it against its source; high is a census figure
  any account gives, low a guess within the right order. A band from memory is
  the first to be checked when it decides a landing.
- **web**: a search engine's summary of the source, the source itself not read.
- **reasoned**: no source; worked out from what the island is (a closed island
  of 135,000 people with no trade abroad).
- **design**: what the design asks of the century after 1900 (the growth that
  `--growth` holds), not history.
- **a fault**: any amount is a fault.

Two joined, as "read; memory", say which part of the band each is (for the
carriage, Bogart's rates were read and the wages that turn them into days are
from memory).

## The table

"Start" is the start the band holds for. "Read as" is the fact of the sweep's
`facts.csv`, or how two are put together.

### Facts with a band

| fact | read as | band | start | sure | source |
|---|---|---|---|---|---|
| runs that failed | `runs_failed`, 1 for a failed run | none | both | a fault | |
| years with hunger, 1770-1900 | `famine_years_1770_1900` | none | both | memory, high | no dearth in England after the 1740s; colonies after their first twenty years (landfall's year 20 is 1770) |
| years with hunger, 1901-2050 | `famine_years_1901_2050` | none | both | memory, high | the same |
| a town's wage over the year before, at most, to 1900 | `town_wage_rise_max_1752_1900` | 1.33 or less | both | memory, medium | yearly wages outside a gold rush or war paper (the colony's row 20) |
| the same, 1901-2050 | `town_wage_rise_max_1901_2050` | 2.0 or less | both | reasoned | what carriage in hands with exports asked reached (1.99); no history |
| the cost of living over three years, at most, to 1900 | `price_level_3y_max_1750_1900` | 1.5 or less | both | memory, medium | Phelps Brown and Hopkins: 1.35 to 1.45 at most (1798 to 1801) |
| the same, at least | `price_level_3y_min_1750_1900` | 0.7 or more | both | memory, medium | the same: about 0.7 (1813 to 1816, 1818 to 1822) |
| grain's price between towns, 1760-1900 | `grain_price_gap_1760_1900` | 0.15 or less | both | read | NBER w16236, Table 1: 40 English counties 1771-1815, wheat 0.09, barley 0.13, oats 0.14 |
| the same, 1901-2050 | `grain_price_gap_1901_2050` | 0.15 or less | both | read | the same; markets drew closer after, not apart |
| a town's prices over its city's, least, 1750-1900 | `town_price_index_vs_city_least_1750_1900` | 0.67 or more | both | read; reasoned | NBER w16236, Table 2: the furthest pair of counties in a year, 0.378 in logs (a ratio of 1.46); for a basket, reasoned |
| the same, greatest | `town_price_index_vs_city_greatest_1750_1900` | 1.5 or less | both | read; reasoned | the same |
| the same two, 1901-2050 | `town_price_index_vs_city_least_1901_2050`, `_greatest_` | 0.67 or more, 1.5 or less | both | read; reasoned | the same |
| people over 1750, 1800 | `pop_1800_vs_1750` | 1.35 to 1.60 | settled | memory, high | censuses: England and Wales 1.43 (1801 over 1751), Great Britain 1.42 |
| people over 1750, 1850 | `pop_1850_vs_1750` | 2.5 to 3.1 | settled | memory, high | England and Wales 2.89, Great Britain 2.81 |
| people over 1750, 1900 | `pop_1900_vs_1750` | 4.5 to 5.5 | settled | memory, high | England and Wales 5.25, Great Britain 5.0 |
| people in 1900 | `pop_1900` | 100,000 to 500,000 | landfall | memory | colonies of settlement at year 150 (the colony's row 3) |
| people in 2050 over 1900 | `pop_2050` over `pop_1900` | 1.5 to 2.3 | settled | memory, medium | Britain and its like over the twentieth century |
| towns in 1900 | `towns_1900` | 10 to 25 | both | reasoned, low | a town is a market with its district; counted three ways for 680,000 people |
| the second town's townsfolk over the first's, 1900 | `townsfolk_second_over_first_1900` | 0.25 to 0.65 | both | memory, medium | small countries in 1900: Norway 0.32, Scotland 0.42, Sweden 0.43, the Netherlands 0.62 |
| townsfolk's share, 1900 | `urban_share_1900` | 0.40 to 0.60 | both | memory, medium | closed countries about 1900: the United States 0.40, France 0.41, Germany 0.54 |
| first railway | `rail_first_year` | 1820 to 1835 | settled | memory, high | Stockton and Darlington 1825, Liverpool and Manchester 1830 |
| railway in 1900, km a 1,000 people | 1,000 `rail_km_1900` over `pop_1900` | 0.8 to 4.0 | both | memory, medium | Great Britain 0.81, Denmark 1.2, Sweden 2.2, the United States 4.1 |
| share of hands on the land, 1850 | `farm_share_1850` | 0.22 to 0.35 | both | memory, high | Deane and Cole: Great Britain 0.22 in 1851 |
| share of hands on the land, 1900 | `farm_share_1900` | 0.15 to 0.30 | both | reasoned | an island that buys no food stands between Britain's 0.09 and France's 0.41 |
| hands to carry a ton a cell as the crow flies, 1800 over 1750 | `carters_per_1000_weight_cells_1800_vs_1750` | 0.7 to 0.9 | settled | read; memory | Bogart 2013, Table 2: road rates a ton-mile, in a labourer's days (about 1.0 in 1750, 0.8 in 1800) |
| the same, 1800 over 1760 | `carters_per_1000_weight_cells_1800_vs_1760` | 0.7 to 0.9 | landfall | read; memory | the same; landfall's first year is one town, so 1760 is its base |
| carters' share of hands, 1900 | `carters_share_of_hands_1900` | 0.05 to 0.12 | both | memory, medium | about one worker in 11 to 13 in transport in 1901 |
| the linked wage grows, per cent a year, 1900-1970 | from `real_wage_linked_1900_vs_1750` and `_1970_` | 1.0 or more | both | design | the growth after 1900 the design asks (`--growth`) |
| the linked wage, 2050 over 1970 | `real_wage_linked_2050_vs_1750` over `_1970_` | 1.0 or more | both | design | the same |
| share of hands on the land, 2050 less 1900 | `farm_share_2050` less `farm_share_1900` | 0 or less | both | design | the same: fewer hands on the land in 2050 |

### Unlike England's: printed with their band, not counted

| fact | read as | band | start | sure | source |
|---|---|---|---|---|---|
| the linked wage over 1750, 1850 | `real_wage_linked_1850_vs_1750` | 1.15 to 1.7 | both | read | Allen 2009, Table 1 (1.39); Feinstein and Clark's series about it |
| the linked wage over 1750, 1900 | `real_wage_linked_1900_vs_1750` | 2.3 to 3.2 | both | read | Allen 2009, Table 1 (2.87 from 1760) |
| wage in grain over 1760's, decade round 1800 | `wage_in_grain_decade_1800_vs_1760` | 0.6 to 1.2 | both | memory, medium | a southern labourer's week in wheat: about 0.21 quarters in the 1750s, 0.14 about 1800 |
| wage in grain over 1760's, decade round 1900 | `wage_in_grain_decade_1900_vs_1760` | 2.0 to 3.5 | both | memory, medium | the same: about 0.60 quarters about 1900 |
| output a head, 1900 over 1750 | `output_per_head_1900_vs_1750` | 2.3 to 3.2 | settled | read | Allen 2009, Table 1: output a worker 2.82 times |
| land's share of income, 1850 | `income_share_land_1850` | 0.08 to 0.14 | both | read, from a chart | Allen 2009, Fig. 2: about 0.11 |

### Printed, no band yet

| fact | read as | why no band |
|---|---|---|
| work's share of income, 1850 and 1900 | `income_share_work_1850`, `_1900` | what the game's measure counts is not checked against Allen's, which counts the self-employed's work at the wage; the probe of the merchants' share decides |
| people in 2050 over 1900, landfall | `pop_2050` over `pop_1900` | settler colonies grew so (about 2.9); no band for a colony yet |
| the cost of living over three years, 1901-2050 | `price_level_3y_max_1901_2050`, `_min_` | history's wars and their paper lie outside the island |
| hands a ton-cell as the crow flies, 1900 | `carters_per_1000_weight_cells_1900` | printed for the canal and the railway, which Bogart's rates also give |

Struck from the twelve, and why: `income_share_land_1900` (Britain's land share
in 1900 is that of a country that bought half its food abroad; 1850 is read
instead), and the twelve's way of setting a band.

## The new facts

All are worked out after the run from what it already keeps, and change
nothing of it (the gate's two sweeps write `annual.csv` byte for byte as main
did, and the golden files are as they were).

- **Hands a ton-cell as the crow flies**, `carters_per_1000_weight_cells_Y`: the
  year's carters (the mean of its months) over the weight carried times the
  cells it went over as the crow flies, a thousand. The weight is what
  `carters_per_1000_weight_path_Y` counts (each cell's produce and works' goods
  to their market, and the trade between towns); the cells are each cell's
  straight distance to its market's town (half a cell at least) and each town's
  to the other. Since a road, a canal or a train does not shorten a straight
  line, every better way counts as fewer hands, as Bogart's rates a ton-mile
  count them. (The older fact over path cost divides the ways out: what is left
  is the price of carting in hands.) Given at the marks and in 1760, and 1800
  over 1750 and over 1760. The sweep notes the cells in December, beside the
  weight (`facts::gather_month`).
- **Grain's price between towns**, `grain_price_gap_A_Z`: for each year, every
  town's mean price of grain that year (the yearly snapshot's), the mean over
  every pair of towns with people of |ln(P_i / P_j)|; then the mean of the
  years. From 1760, as the price swings are, since prices are still settling
  before it. History's number is the same measure on 40 English counties.
- **A town's prices over its city's**, `town_price_index_vs_city_least_A_Z` and
  `_greatest_`: each town's cost of the cost of living's basket (the households'
  and the lords' purchases of the first year, `cpi_basket`) at its own year's
  prices, over the same at the prices of the year's largest town by people; the
  least and the greatest of any town in any year of the span. The study asked
  for the largest town "within reach"; on an island with one road net every town
  is within reach of the largest, so the largest is read.
- **The wage in grain by decade**, `wage_in_grain_decade_Y_vs_1760`: the
  island's mean wage over grain's national price, the mean of the ten years
  round Y (Y-5 to Y+4, cut to the run), over the same round 1760. A run that
  ends in 1900 has six years round 1900.
- **From 1901**: `town_wage_rise_max_1901_2050` and
  `price_level_3y_max_1901_2050` and `_min_`, as their twins to 1900
  (landfall.rs).
- **People in 2050 over 1900** and **railway a 1,000 people** are read from two
  facts the sweep already had.

## What moved, and how far from version 0.1.0

Beneath the written check the old twelve are printed as before, as **what
moved**: each against the reference's spread across its islands. They are
still the gate's: a fact not inside fails the sweep (docs/rules/README.md).

Then the same twelve are set against **version 0.1.0**, main at 190e5ad on 27
November 2026, from the gate's two reference sweeps of that day (islands 1 to 8,
to 1900, the settled island and landfall), kept island by island in
`data/standing-0.1.0.csv` and read by `standing::v010`. That reference never
changes, so every later sweep can say how far the island has gone from that
version. It is printed, not counted. (Its `last_town_founded` is of runs to
1900, and reads OUTSIDE against a run to 2050, as the twelve's always has.)

## Main's score

Main as version 0.1.0 left it (190e5ad), both starts to 2050, read on 27
November 2026 with

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --start settled,landfall --end-year 2050 --standing --out DIR --quiet
isostate-core sweep --seeds 17-32 --rules game --families 6 --start settled,landfall --end-year 2050 --standing --out DIR --quiet
```

| | islands 1 to 8 | islands 17 to 32 |
|---|---|---|
| settled island, facts with a band met | 18 of 29 | 17 of 29 |
| landfall, facts with a band met | 12 of 25 | 12 of 25 |
| landfall, the colony's 24 | 14 | 13 |

What main misses on both starts and both sets of islands: a town's wage over
the year before (1.5 to 1.7 to 1900; about 10 after, against 2.0); grain's
price between towns (0.17 to 0.21 to 1900 against 0.15; 0.74 to 0.79 after); a
town's prices over its city's (the least 0.26 to 0.42 to 1900, and 0.02 to 0.03
after: towns where the basket costs a fiftieth of the city's); the second
town's townsfolk over the first's (0.73 to 0.85 against 0.25 to 0.65); and the
hands a ton-cell as the crow flies (0.57 over 1750 on the settled island, 0.45
over 1760 at landfall, against 0.7 to 0.9: carriage in fixed coins makes a
load too cheap in hands as the wage rises). The settled island also misses
hunger after 1900 on some islands (0.4 and 0.8 years an island) and stands at
the railway's upper edge (4.00 and 3.92 km a 1,000 people against 4.0).
Landfall also misses its people in 1900 (527,000 to 535,000 against 500,000 at
most), the cost of living's least over three years (0.64 to 0.65 against 0.7),
its carters (0.045 against 0.05) and a hungry year before 1900 on some islands.

Of the facts known wrong, the settled island's wage is twice history's (the
linked wage 5.7 to 5.9 times 1750's by 1900, against 2.3 to 3.2) and the wage in
grain about 6.7 times 1760's (against 2.0 to 3.5); landfall's linked wage is
inside its band in 1850 (1.28 and 1.43) and at its top in 1900 (3.07 and 3.28,
against 3.2), its wage in grain not (1.55 round 1800, 4.6 to 4.7 round 1900). Both starts' land takes 0.15 of income in 1850 against about 0.11.

On the gate's own sweeps, which end in 1900, the facts after 1900 have no
value: the settled island meets 14 of 29 there and landfall 7 of 25.

## Bands to doubt

- **Hands a ton-cell, 0.7 to 0.9.** Bogart's fall is the road carrier's alone.
  The measure counts every better way as fewer hands, and the settled island
  digs its first canal about 1776; a Britain with a quarter of its ton-miles on
  water by 1800 would read lower than 0.7 on this measure too. The band may be
  too strict at its foot.
- **A town's prices over its city's, 0.67 to 1.5.** History's 0.378 in logs is
  for one good, wheat, between counties; a basket should stand closer, so the
  band is loose for most goods and tight for coal, which stood several times
  dearer far from the pit.
- **Railway a 1,000 people, up to 4.0.** The United States' 4.1 is a continent
  being settled; the island is a small closed country, nearer Sweden's 2.2.
- **A town's wage after 1900, at most 2.0**, is no history: it is what the study
  saw the new carriage rules reach.
- **The townsfolk's floor, 0.40 or 0.45**, both from memory; 0.40 is written,
  as the verdict judged.
