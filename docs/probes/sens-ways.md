# sens-ways: how much the railways' and the canals' numbers matter

A table of sensitivity for the pass over all the rules (piece 18 of
`docs/backlog/found-along-the-way.md`): most of the mechanisms' numbers were set
on a few islands and never searched. This page takes every number that
[railways.md](../rules/railways.md) and [canals.md](../rules/canals.md) list as a
setting, moves it to about three quarters and to about one and a third of its
value, and asks what moves on the island. Nothing in the simulation was changed.
Measured on 12 November 2026 at a6ff0ec, the game's rules with six families.

## In short

1. **No run broke.** 912 runs, at every value tried: no panic, no failure of the
   monthly money check, no town lost, no year in which the island's people fell
   by as much as 3 per cent. The most famine years in any run was four.
2. **Within the range tried, no setting of either mechanism moves output per
   head or the real wage by much.** As means of twenty years, no change between
   a setting's two values is more than 3 per cent, none is as large as a fifth
   of the spread between islands, and only one runs the same way on 14 islands
   of 16 or more: dearer railway lines (`cost_mult` 6.67 against 3.75) leave
   output a head in 1831 to 1850 0.9 per cent lower and the real wage 1.5 per
   cent lower. The economy feels the railways and the canals through the two
   switches, not through their dials. Switching the game's railways off costs 3.0 per cent of output a head in
   1881 to 1900 (14 islands of 16) and 3.7 per cent of the real wage (13 of 16);
   switching the canals off puts 1.4 more points of the island's hands to carting
   in 1800 (all 16) and gives a run 1.4 famine years where it had 0.4 (9 islands
   more, 3 fewer).
3. **The railway's size is set by the branch planner's dials** more than by
   the money its towns can raise: `districts`, `branch_gap`, `cool_years`, `cost_mult` and the hurdle
   each move the railway of 1900 by 250 to 470 km between their two values, on
   15 or 16 islands of 16. `branch_gap` is not monotone: 3, the value in use,
   gives more line than either 2 or 4.
4. **The hurdle and the induced traffic are one number** for both ways. The
   promoters' test is `induced * saving / cost >= hurdle` and nothing else reads
   either, so `rail_hurdle` 0.08 and `rail_induced` 1.875 give every fact alike.
5. **The canal age keeps its shape whatever is moved.** Every setting tried
   leaves between 0.73 and 0.79 of the network of 1900 open by 1800. The dials
   move the first canal by two to five years and the network by a tenth; none
   stretches the rush, which is the canals page's first worry.
6. **Eight islands and one baseline mislead.** Making a canal dearer and making
   it cheaper both cost the island a town in 1900 on most of seeds 1 to 8. That
   is the baseline's luck, not the canals (below, "How to read the tables").

## What was run

Every sweep is the game's rules with six families to 1900, one setting moved at
a time, both values side by side on the same islands:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set railways.districts=6,11 --out DIR --compare BASELINE --quiet
```

The baseline is the same command with no `--set`. On seeds 1 to 8 it gives every
one of 446 facts as `merged-game-f6-l` (today's main) gives it. Thirty-six
sweeps on seeds 1 to 8 (544 runs); the eighteen settings that moved anything, or
whose result I would stake something on, again on seeds 9 to 16 (256 runs, and
the baseline's 8); and three pairs of settings together on seeds 1 to 8 (96
runs). No run failed. The folders, the scripts that read them and the readings
they printed are under `.claude/night/reports/opus/sens-ways/`.

**The settings.** The railways' are `RailwayParams` (`railways.rs`) and the
three numbers of the port's that `railways::way` reads: `rail_hurdle`,
`rail_induced` and `rail_know`. The port's materials per cell
(`rail_rails_per_cell` and its two neighbours) reach the game's railway only as
`cost_mult` times them, so `cost_mult` stands for all four. `rail_max_projects`
is the port's cap and is not read once `railways.enabled` hands the year to
`ways::plan`; `transport::RAIL_COST` is a constant and cannot be swept. The
canals' are all 23 numbers and switches of `CanalParams` (`canals.rs`).

**The values.** About 0.75 and 1.33 times each number, rounded where the
setting wants it: whole cells for `branch_gap`, whole years for `cool_years`,
whole counts for `districts` and `max_projects`. The canals' six districts went
to 4 and 8, since four and a half has no meaning. The switches were flipped.

## How to read the tables

For each setting the tables give what moved most between its low value and its
high value, as means over the islands, and on how many islands it moved that
way. Reading the two values against each other, island for island, matters
here, for a reason the first readings showed.

**The baseline is one draw.** The island's history is sensitive: nudge almost
anything about the canals and each island takes a somewhat different path. On
seeds 1 to 8 the baseline happens to found more towns than most of the paths a
nudge gives: the canals' stone, bricks, timber, clay and locks, each at three
quarters and at four thirds, all give between 1.0 and 1.75 fewer towns in 1900
than the baseline, on five to seven islands of eight. Making a canal dearer and
making it cheaper cannot both cost a town. Against the eight runs of those
four materials the baseline stands above their median on 7 islands of 8, by
1.4 towns, where a nudge moves a run by 1.0 towns either way. So any comparison
with the one baseline carries the baseline's own luck, and "seven islands of
eight agree" can come from the baseline alone. Reading low against high does
not go through it.

**Path noise.** The same eight nudges give a scale for how far a fact wanders
on one island when nothing much has changed (the median over islands of each
island's standard deviation across the nudges; `readings/noise_materials.txt`):

| fact | baseline, seeds 1 to 8 | spread between islands | path noise on one island |
|---|---|---|---|
| first railway | 1827 | 9.2 years | 1.5 years |
| km of railway, 1850 | 1,328 | 379 | 67 |
| km of railway, 1900 | 2,248 | 111 | 87 |
| km of canal, 1900 | 892 | 162 | 52 |
| towns, 1900 | 17.8 | 2.0 | 1.0 |
| famine years in a run | 0.38 | 0.48 | 0.60 |
| output a head, mean of 1881 to 1900 | 8.27 | 0.40 | 0.13 |
| real wage, mean of 1881 to 1900 | 2.68 | 0.10 | 0.06 |

The railway of 1900 wanders on one island nearly as much as it differs between
islands, so a movement of a hundred kilometres there is nothing on eight
islands. And there is a natural check of the same thing: `canals.hurdle` 0.133
and `canals.induced` 0.75 are the same canal promoter to a quarter of a per cent,
and they give nearly the same canals (35 and 32 fewer km in 1800), while the
railway of 1850 differs between the two by 62 km and the first line by a quarter
of a year.

By chance alone a fact that moves at random agrees on seven or eight islands of
eight about one time in fourteen, and a sweep has about 270 facts that move at
all. So nothing below rests on eight islands where it matters: the settings
marked "16" were run again on seeds 9 to 16, and the counts are of sixteen
islands. One year misleads too, so the economy is read as means of twenty
years (1781 to 1800, 1831 to 1850, 1881 to 1900) from `annual.csv`.

## Railways

In the table, "a to b" is the mean at the low value to the mean at the high
value; the count is the islands on which it moved that way. Baseline means on
sixteen islands: the first line in 1827, 1,348 km in 1850 and 2,254 in 1900, the
network costing 0.21 of a year's output and the lines 0.78 of what was expected.

| setting | value | tried | what moved most, low value to high | islands |
|---|---|---|---|---|
| `enabled` | on | off (the port's promoters) | railway of 1900: 2,254 to 1,031 km; of 1850: 1,348 to 877; towns in 1900: 17.2 to 19.8; canal of 1900: 903 to 1,009 km; famine years 0.38 to 1.06; output a head 1881 to 1900 down 3.0 per cent, real wage down 3.7 per cent | 16 of 16; 15; 14; 15; 9 up, 2 down; 14; 13 (16) |
| `cost_mult` | 5 | 3.75, 6.67 | railway of 1850: 1,450 to 1,148 km, of 1900: 2,462 to 2,085; network's cost 0.18 to 0.25 years of output; fares over capital in 1900 0.40 to 0.27; first line 1.8 years later; canal of 1900 60 km longer | 16; 16; 16; 16; 10 of 11 that moved; 13 (16) |
| `build_months` | 24 | 18, 32 | lines' cost over expected 0.85 to 0.71; most lines building at once 21 to 26; the railway's length does not move | 16; 15 (16) |
| `order_book` | on | off | lines' cost over expected 0.78 to 1.23; network's cost 0.21 to 0.27 years of output; dearest rails 1.39 to 1.88 times 1800 to 1825; railway of 1840 820 to 984 km; slowest line 6.0 to 7.3 years | 16; 16; 14; 14; 14 (16) |
| `max_projects` | 40 | 30, 53 | nothing: every fact as the baseline's | 8 of 8 alike |
| `districts` | 8 | 6, 11 | railway of 1900: 1,948 to 2,418 km, of 1850: +283 km; lines to farms 20 to 46 a run; branches' share of the km 0.70 to 0.77; most lines building 20 to 28 | 16; 16; 16; 16; 16 (16) |
| `branch_gap` | 3 | 2, 4 | railway of 1900: 1,719 at 2 and 2,066 at 4, against 2,254 at 3; lines to works 130 to 75; of 1850: +261 km | 16 and 15 against 3; 16; 16 (16) |
| `district_spacing` | 6 | 4.5, 8 | nothing clear | (8) |
| `cool_years` | 5 | 4, 7 | railway of 1900: 2,079 to 2,335 km; lines to farms 28 to 40 | 16; 14 (16) |
| `rail_hurdle` | 0.06 | 0.045, 0.08 | railway of 1900: 2,400 to 2,113 km; branches' share of the km 0.76 to 0.73; lines to farms 42 to 31; the railway of 1850 hardly moves | 15; 15; 15 (16) |
| `rail_induced` | 2.5 | 1.875, 3.33 | the hurdle's own results the other way about: 1.875 is fact for fact `rail_hurdle` 0.08 | (8) |
| `rail_know` | 0.25 | 0.19, 0.33 | nothing clear; the first line moves on one island of eight | (8) |

What the railway's dials do, then, is size the network, and mostly its
branches: the first line's date does not move by more than two years for any of
them (it belongs to the discovery, as the railways page says), and the railway
of 1850 moves much less than that of 1900 for all but `cost_mult`, `districts`
and `branch_gap`. The money does not pace the lines either: `max_projects` never
binds above 30, and dearer lines (`cost_mult`) mean a smaller network more than
a slower one. The order book is what keeps the promoters' estimates honest:
without it the lines cost 1.23 times what was expected, where with it they come
in under (0.78), on every island.

## Canals

Baseline means on sixteen islands: the first canal in 1774, 695 km in 1800 and
903 in 1900, the network costing 0.14 of a year's output, carters 0.119 of all
hands in 1800.

| setting | value | tried | what moved most, low value to high | islands |
|---|---|---|---|---|
| `enabled` | on | off | carters in 1800: 0.119 to 0.133 of all hands; carting 0.080 to 0.092 of output in 1800; famine years 0.38 to 1.38; output a head 1781 to 1800 down 1.2 per cent; the merchants' share of income in 1800 down a quarter; nothing in 1850 or 1900 | 16; 16; 9 up, 3 down; 15; 16 (16) |
| `cost` | 0.13 | 0.0975, 0.173 | canal of 1800: 728 to 600 km, of 1900: 941 to 793; first canal 4.4 years later; boats' weight-km in 1800 362 to 225 million; carters in 1800 up 0.012; towns in 1900 up 1.7; railway of 1850 up 145 km | 16; 15; 9 of 9 that moved; 16; 16; 12; 12 (16) |
| `wharf` | 0.4 | 0.3, 0.533 | boats' haul in 1820: 41 to 53 km; canal cells carrying anything in 1900: 0.39 to 0.23; carters in 1800 up 0.006; canal of 1900 41 km shorter; first canal 3 years later | 16; 16; 16; 13; 5 of 5 that moved (16) |
| `stone` | 150 | 112.5, 200 | nothing clear (output a head in 1831 to 1850 was 3 per cent higher on all of seeds 1 to 8; on sixteen it is 1 per cent, on 10 of 16) | (16) |
| `bricks` | 80 | 60, 107 | fares over capital in 1800 0.53 to 0.42; canal of 1900 70 km shorter; network's cost 0.020 more years of output; first canal 2 years later | 16; 12; 13; 6 of 6 that moved (16) |
| `timber` | 60 | 45, 80 | nothing clear | (16) |
| `clay` | 200 | 150, 267 | nothing clear | (8) |
| `lock_cells` | 80 | 60, 107 | network's cost 0.125 to 0.156 years of output; boats' haul in 1900 17 to 12 km | 8; 7 (8) |
| `wages` | 2 | 1.5, 2.67 | canal of 1800: 718 to 627 km, of 1900: 933 to 821; network's cost 0.127 to 0.155 years of output; first canal 3 years later | 15; 14; 15; 6 of 6 that moved (16) |
| `navvies` | on | off | only the count of navvies; nothing in carting, output, the wage or the towns | (16) |
| `navvy_max_share` | 0.2 | 0.15, 0.267 | nothing clear in the canals | (8) |
| `navvy_adjust` | 0.25 | 0.1875, 0.333 | nothing clear (baskets a head in 1900 down 3 per cent on 7 of seeds 1 to 8; not on 16) | (16) |
| `build_months` | 36 | 27, 48 | slowest canal 4.1 to 5.8 years; first canal 1.3 years later; most navvies at once 1,410 to 1,070; canal of 1790 60 km shorter | 8; 8; 7; 8 (8) |
| `hurdle` | 0.1 | 0.075, 0.133 | canal of 1790: 652 to 513 km, of 1800: 744 to 644; first canal 4.8 years later; network's cost 0.158 to 0.131 years of output; railway of 1880 up 5 per cent | 16; 15; 9 of 9 that moved; 15; 14 (16) |
| `induced` | 1 | 0.75, 1.33 | the hurdle's own results the other way about | (8) |
| `max_projects` | 12 | 9, 16 | moves 4 and 2 islands of 8 at all; nothing clear | (8) |
| `districts` | 6 | 4, 8 | nothing clear in the canals | (8) |
| `branch_gap` | 3 | 2, 4 | nothing clear in the canals | (8) |
| `district_spacing` | 6 | 4.5, 8 | nothing clear | (8) |
| `cool_years` | 5 | 4, 7 | nothing clear | (8) |
| `discover_coal` | 3,000,000 | 2,250,000, 4,000,000 | moves 3 and 5 islands of 8 at all; where the first canal moves, it is 2.3 years later at the higher value | 3 of 4 that moved (8) |
| `discover_stone` | 3,000,000 | 2,250,000, 4,000,000 | moves 1 island of 8 at all | (8) |
| `know` | 0.25 | 0.19, 0.33 | at 0.19 the first canal does not move (the surveying town starts at 0.25); at 0.33 it is 2.3 years later | 4 of 4 that moved (8) |
| `hearsay` | 0.2 | 0.15, 0.267 | canal of 1780: 190 to 371 km, and nothing after 1790 | 8 (8) |

Two things about the canals' dials. The survey waits on the lesser of coal and
stone dug against their thresholds, and on seven islands of eight that is coal,
so `discover_stone` does almost nothing and `discover_coal` is the one that
counts. And whatever was moved, the shape of the canal age held:

| | first canal | 1780 | 1790 | 1800 | 1830 | 1900 | share of 1900's open by 1800 |
|---|---|---|---|---|---|---|---|
| baseline (16) | 1774 | 261 | 592 | 695 | 839 | 903 | 0.77 |
| `cost` 0.0975 / 0.173 (16) | 1771 / 1775 | 421 / 142 | 634 / 484 | 728 / 600 | 867 / 743 | 941 / 793 | 0.77 / 0.76 |
| `hurdle` 0.075 / 0.133 (16) | 1771 / 1776 | 393 / 157 | 652 / 513 | 744 / 644 | 918 / 814 | 954 / 870 | 0.78 / 0.74 |
| `wages` 1.5 / 2.67 (16) | 1772 / 1775 | 364 / 173 | 642 / 532 | 718 / 627 | 888 / 773 | 933 / 821 | 0.77 / 0.76 |
| `build_months` 27 / 48 (8) | 1774 / 1775 | 310 / 204 | 623 / 563 | 676 / 652 | 822 / 802 | 866 / 863 | 0.78 / 0.76 |
| `hearsay` 0.15 / 0.267 (8) | 1776 / 1773 | 190 / 371 | 552 / 586 | 644 / 656 | 819 / 819 | 866 / 857 | 0.74 / 0.77 |

The dials make the network bigger or smaller and the start a few years sooner
or later, and the rush is over by 1800 in every row.

## The settings that matter most

- **For the railway's size:** `railways.districts` (+470 km in 1900 from 6 to
  11), `railways.branch_gap` (534 km less at 2, 188 less at 4), `cost_mult`
  (377 km less from 3.75 to 6.67, and the only one that also moves 1850 by
  300 km), the hurdle (287 km) and `cool_years` (256 km). All on 15 or 16
  islands of 16.
- **For what the lines cost against their estimates:** `order_book`, then
  `build_months`.
- **For the canals' size and date:** `canals.cost`, the canal `hurdle` (with
  `induced`), `wages` and `wharf`, in that order: each moves the network of 1800
  by 30 to 130 km and the first canal by 2 to 5 years, on 15 or 16 islands of
  16 for the network.
- **For the island:** mostly the two switches. Canals on take a point and a
  half of the hands out of carting in 1800 on every island; canal `cost` moves
  that by six tenths of a point either way and the wharf by two to four tenths,
  on 13 to 16 islands of 16. A cheaper canal also leaves the island with fewer
  towns in 1900 (1.7 fewer at a `cost` of 0.0975 than at 0.173, on 12 of 16),
  as the canals page found of the canals themselves. Output a head and the real
  wage hardly move for any dial (In short, 2).

## Settings that do nothing at these sizes

`railways.max_projects` (never binds above 30: every fact is the baseline's),
`canals.max_projects` (binds for a year on a few islands), `canals.discover_stone`,
`rail_know`, `canals.know` below its value, `canals.navvies` and its two numbers
`navvy_max_share` and `navvy_adjust` (the navvies are at most some 1,300 people
on an island of 200,000), `railways.district_spacing`, and the canals' own
`districts`, `branch_gap`, `district_spacing` and `cool_years` (the canals are
nearly all trunks, so the branch planner's dials have little to work on). The
canals' four materials matter only as the canal's cost: stone, timber and clay
alone move too little to tell apart from path noise, and bricks take some 70 km
off the canal of 1900 on twelve islands of sixteen.

## Where a run breaks

Nowhere in the range tried. All 912 runs finished, the money check held every
month, no town was lost from one year to the next, the island's people never
fell by 3 per cent in a year, and the most famine years in any run was four
(canals off on seeds 1 and 3; `canals.bricks` 107 on seed 6). The extremes the
canals page tried (nought and absurd) also ran. The famine count is the one
number that moved with a switch: canals off gives 1.38 famine years a run on
sixteen islands, railways off 1.06, against 0.38.

## Pairs tried, and pairs worth trying

Three pairs were run together on seeds 1 to 8, both values of each against both
of the other (`readings/pairs.txt`).

- **`railways.districts` with `branch_gap`: they run out together.** At a gap of
  2 more districts add 382 km to the railway of 1900 (1,522 to 1,904); at a gap
  of 4 they add nothing (2,026 to 2,027), and the difference runs the same way
  on all eight islands. Far enough from
  track there are not eleven districts worth a look, so the gap is the binding
  dial and `districts` matters only when the gap is small. All four corners
  give less line than the baseline's 8 and 3.
- **`cost_mult` with `rail_hurdle`: they add.** Each takes some 280 km off the
  railway of 1900 and together they take 560, with no interaction to speak of.
  They differ in when: dearer lines cost the rush (308 km less in 1850), a
  higher hurdle the late branches (55 km in 1850).
- **`canals.cost` with `wharf`: they add too.** The cheapest pair gives 770 km of
  canal in 1800 and the dearest 555; the cheap canal's cost to the island in
  towns (2.25 fewer in 1900 at the lower cost) does not depend on the wharf.

Worth trying next, none run:

- the canal `hurdle` with `railways.cost_mult`, since each way's promoters see
  the other's lines building, to see whether dearer railways leave more to the
  canals after 1830;
- `canals.cost` with `railways.enabled` off, to see whether the canals do more
  for the island when the game's branches do not take their traffic;
- `hearsay` with `discover_coal`, the two numbers that set when the canal age
  begins, for the calendar between islands.

## What I would look at first in a pass over the rules

1. **Fold `hurdle` and `induced` into one number**, for both ways, or say on the
   pages that they are one. Two dials that are the same dial invite a search
   that finds nothing.
2. **`branch_gap` sits at a peak.** The railway is 534 km shorter at 2 and 188
   km shorter at 4, so the size of the network is a by-product of how the
   districts are searched, not of what lines are worth. If the network reads as
   too much on the map (the railways page's worry), `districts` and the hurdle
   are the smooth dials and `branch_gap` is not.
3. **The canal age is short because of the rule, not the numbers.** No setting
   in range stretches it. Tolls, or a promoter who must see a canal pay before
   the next is dug, are where to look, as the canals page says.
4. **The famines.** Canals off gives a run 1.4 famine years where it had 0.4,
   and the railways off 1.1, on sixteen islands. That is the strongest thing
   either mechanism does to the island beyond carting, and it is not traced:
   grain that can go by boat or rail in a bad year is the obvious guess.
5. **The dials can be tuned for the map and the calendar without fear for the
   economy.** Within a third either way none of them moves output a head or the
   wage by more than a few per cent, a fifth of the spread between islands, so
   the scorecard of piece 18 can set them by how the network looks and when it
   comes. That holds for the island as these rules have it, where nobody owns a
   line or a canal; fares that somebody collected would change it.
6. **Read against both values, and on sixteen islands.** On seeds 1 to 8 the
   baseline's own luck reached the towns of 1900, and three "all eight islands"
   results did not survive seeds 9 to 16: `build_months` 32 took 70 km off the
   railway of 1850 on all eight, and on sixteen it is 43 km on 12; canal `cost`
   took 3.6 per cent off output a head in 1831 to 1850 on all eight, and on
   sixteen 1.3 per cent on 12; canal `stone` added 3 per cent to the same, and
   on sixteen 1 per cent on 10. A scorecard built on eight islands would have
   kept all three.

## Files

Under `.claude/night/reports/opus/sens-ways/`: `game` and `game-s9` (the
baselines), `s8/NAME` and `s9/NAME` (one sweep a setting), `pairs/`,
`readings/` (what the scripts printed: `table8.txt` and `table16.txt` against
the baseline, `hilo_r8.txt`, `hilo_c8.txt`, `hilo16.txt` and `hilo16b.txt` low value against
high, `analyse8.txt` every key fact, `noise_materials.txt`, `pairs.txt`), and
`scripts/` (the Python that reads them, and the shell that ran them). `build.md`
there has every command and decision.
