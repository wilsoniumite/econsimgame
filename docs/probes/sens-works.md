# How sensitive the island is to the works' life cycle and to carting

One of six tables of sensitivity for the pass over all the rules (piece 18 of
`docs/backlog/found-along-the-way.md`). The findings behind this one are R119 and C34 (the life
cycle's numbers were checked only as a set) and R13 and C92 (the plan's targets for 1750 to
1800 were set without carting). It changes no rule and no fact. Every number of the two
mechanisms was moved on its own, to about three quarters and four thirds of the game's value,
and the island run again; this page says what moved, by how much against the spread between
islands, and on how many islands.

The game's rules at a6ff0ec (12 November 2026), six families, 1750 to 1900, seeds 1 to 16.

In short:

- **Of the life cycle's nine numbers only the year's room (`year_room`) moves the economy**, and
  what it moves is the century's steadiness and its start, not the level of the wage. A smaller
  room makes iron and tools scarce and dear before 1800 and the swings after it a seventh
  smaller; a larger room the other way round.
- **Carting's `labor` moves the level of the first fifty years** (half the carters gives back
  about half of what carting costs in 1800) and hardly anything after; the delay that carting
  puts into the middle of the century does not follow its size.
- **The rest of the life cycle shapes the map and the churn, not the economy**: the hands to a
  cell and the cell's limit set how many works there are, how large and on how many cells; the
  merchant's spread at entry and the families' extensions set how many are founded and closed.
  The walk, the share for entry, `built_share`, carting's cap and its damper do nothing that
  sixteen islands can see.
- **No run failed** in 600 runs, but in seven of them one town's wage ran away for a few months
  (to 47 and 121 times the middle town's), always on the same few islands near a town's first
  years. That is the labour market's monthly limit, not these mechanisms; the seeds are below.

## How it was run and read

Every setting is a field of `WorksParams` (`crates/isostate-core/src/works.rs`) or
`CartingParams` (`crates/isostate-core/src/carting.rs`), and `sweep --set` reaches each of
them:

```bash
isostate-core sweep --seeds 1-16 --rules game --families 6 --out DIR/base16
isostate-core sweep --seeds 1-8  --rules game --families 6 --set works.year_room=0.1125,0.2 --out DIR/s8-works.year_room
isostate-core sweep --seeds 9-16 --rules game --families 6 --set works.year_room=0.1125,0.2 --out DIR/s9-works.year_room
```

and so on for each key, 600 runs in all. Each run is set beside the same island at the game's
own values. One year misleads, so the yearly series (`annual.csv`) are read as twenty-year means,
1780-99, 1830-49 and 1880-99, beside the sweep's facts; a swing is the standard deviation of the
year-on-year change of the series' logarithm over 1800 to 1899.

How big a movement is, is said against the spread between islands: *d* is the change in the
mean over the standard deviation of the fact across islands, so a *d* of 1 is a movement as large
as the difference between one island and the next. Beside it, the islands on which the fact rose
and fell.

**What chance gives.** The model is chaotic in its details: move any number by a millionth and
each island goes down another path. Two such placebos were run on all sixteen islands
(`works.walk=0.0200001`, `carting.adjust=0.2500001`). On seeds 1 to 8 they moved 22 and 29 of
some 560 facts and series with seven islands of eight agreeing, with *d* up to 1.4: the chained
real wage of 1900 rose 4 per cent on seven islands of eight, and baskets per head in 1850 by 7
per cent. On sixteen islands they moved 2 and 0 facts with fourteen of sixteen agreeing. So
eight islands cannot tell a *d* of 1 from nothing, and every claim here is held to fourteen
islands of sixteen.

**The reference.** Since every setting is read against the same run at the game's values, that
run's own luck shows up as the same movement under every setting. On seeds 1 to 8 it has more
late towns (5.1 a run) than almost any run beside it (3 to 4.5), the placebos included, so at
first nearly every setting seemed to found fewer. The readings below are therefore against the
mean, island by island, of the three runs at the game's own values (the baseline and the two
placebos), with the spread between islands read from the baseline alone. That alone took away one
apparent gain (a cell limit of 1.0 seemed to raise goods per head in 1900 by 3.7 per cent on seven
islands of eight; against the three runs it is 1.5 per cent on five).

In the tables, "of 8" is how many of seeds 1 to 8 moved the way the mean did and "of 16" the
same over all sixteen; where a row has two values tried the counts are for each in turn. The
numbers are sixteen-island means against the game's own values, which are given first. "Facts
at 14/16" counts the facts and series (of about 560) on which fourteen or more islands agree;
the placebos have 0 and 2.

## The works' life cycle

`docs/rules/works.md`. Nine numbers and the switch. The page's own sweeps tried `year_room` from
0.10 to 0.30 and `built_share` from 0.01 to 0.2 in company, and every part switched off alone;
`hands_per_cell`, `cell_limit`, `room_share`, `entry_share` and `walk` were set by the probe
and checked only as a set (R119, C34).

| setting | game | tried | what moved, sixteen islands | of 8 | of 16 | facts at 14/16 |
|---|---|---|---|---|---|---|
| `enabled` | on | off | 6,782 works in 1900 for 720, the median one 0.52 usual sizes for 6.0, on 1,087 cells for 200; goods per head 1900 9.3 for 11.7 (-21%), baskets 7.9 for 8.9 (-11%), the real wage of 1880-99 -14%; the real wage's yearly swing 0.030 for 0.063, timber's 0.043 for 0.204; urban share 1900 0.379 for 0.419; people +4% | 8 | 16 | 227 |
| `year_room` | 0.15 | 0.1125, 0.2 | iron made in 1780-99 -25% / +23%, its price +61% / -31%, tools +34% / -16%; the yearly swing of the real wage 1800-99 -14% / +15%, of the cost of living -20% / +17%, of timber -20% / +13%; works closed -13% / +10%, closures in the news 175 / 216 for 196; steel rails 0.13 / 0.01 of rails in 1890 for 0.05. At 0.1125 alone (at 0.2 the other way, on 11 and 12 islands): the money wage and the cost of living of 1880-99 both 17% lower, the real wage unmoved | 7-8, 6-8 | 13-16, 13-16 | 80, 50 |
| `room_share` | 0.34 | 0.255, 0.4533 | works 1900 770 / 647 for 720, in 1780-99 +17% / -9%; the ninth tenth of size 32 / 44 for 37 usual sizes | 7-8, 7-8 | 13-15, 13-16 | 28, 9 |
| `entry_share` | 0.10 | 0.075, 0.1333 | industry cells 1900 177 / 216 for 200; at 0.075 the works' share of income 1880-99 9.2% for 8.7% | 8, 7 | 15-16, 14 | 9, 3 |
| `hands_per_cell` | 2,000 | 1,500, 2,667 | industry cells 1900 283 / 140 for 200; works 802 / 621 for 720; median size 4.9 / 7.7 for 6.0, the largest 91 / 159 for 121 | 7-8, 8 | 15-16, 16 | 19, 18 |
| `cell_limit` | 0.8 | 0.6, 1.0 | median size 4.75 / 7.2 for 6.0, the largest 92 / 145; works 789 / 659; industry cells 210 / 176; at 0.6 the works' share of income 1880-99 8.2% for 8.7% | 7-8, 7-8 | 13-16, 14-16 | 8, 12 |
| `count_spread` | on | off | works founded 2,703 for 1,916 (+41%), closed 2,145 for 1,428 (+50%), their median life 9.2 years for 11.7; closures in the news +17%; utilisation 1900 0.714 for 0.768; industry cells +11%; the works' share of income 1880-99 -12% | 8 | 14-16 | 27 |
| `family_extends` | on | off | works standing in 1800 544 for 246, in 1850 727 for 516, in 1900 798 for 720; founded +48%, closed +59%, the closed living 13.6 years for 11.7 | 8 | 16 | 18 |
| `built_share` | 0.05 | 0.0375, 0.0667 | nothing (largest *d* 0.2) | | | 2, 1 |
| `walk` | 0.02 | 0.015, 0.0267 | nothing past chance: industry cells 1900 +5% at 0.015 (14 of 16), the timber and cost-of-living swings -7% at 0.0267 (14 and 15 of 16), all with *d* under 1 | | | 5, 12 |

The values tried are three quarters and four thirds of the game's, except `cell_limit`, where
four thirds would be more than the whole cell and 1.0 was tried instead. Wages, goods per head,
baskets, people and the towns' share do not move under any of the life cycle's numbers but the
switch itself and the year's room (the largest *d* among them elsewhere is 0.5, with the islands
split). In particular the real wage of 1880-99 is within 2 per cent of the game's under every
value tried.

## Carting

`docs/rules/carting.md`. Three numbers and the switch. Its page measured `labor` at 0.5 on six
islands and `adjust` from 0.15 to 1 on two; `max_share` was never moved.

| setting | game | tried | what moved, sixteen islands | of 8 | of 16 | facts at 14/16 |
|---|---|---|---|---|---|---|
| `enabled` | on | off | the money wage of 1780-99 -17%; goods per head 1750 +13%; the industry share of 1780-99 0.321 for 0.262, the urban share 0.286 for 0.242; baskets per head 1800 +12.5%, 1850 +21%; goods per head 1850 +13%, 1900 +3%; people 1900 +5.5%; urban share 1900 0.445 for 0.419; railway 1900 +9.5%; famine years 0 for 0.5 a run. Baskets per head in 1900 +1.3% (13 of 16, 6 of 8) | 7-8 | 14-16 | 150 |
| `labor` | 1.0 | 0.5, 0.75 | carters' share of hands in 1800 0.068 / 0.096 for 0.119, in 1900 0.037 / 0.052 for 0.066; the industry share of 1780-99 +11% / +4%, the urban share +9% / +3%; baskets per head 1800 +5.9% / +2.5%; the money wage of 1780-99 -6% / -4%. At 0.5 also people 1900 +3.0% and urban share 1900 0.429. Not on most islands: baskets per head 1850 +2.7% / +7.9% (9 and 11 of 16); in 1900 -0.9% / +0.8% | 7-8, 7-8 | 14-16, 14-16 | 95, 57 |
| `max_share` | 0.5 | 0.375, 0.667 | nothing; the carters' share of hands moves by 1 to 2 per cent either way | | | 2, 0 |
| `adjust` | 0.25 | 0.1875, 0.333 | nothing in the yearly facts | | | 3, 1 |

`labor` can only come down (it is the share of the carting bill that is wages, and a share stays
a share), so its values are three quarters and a half. The early effects are close to straight
lines in it: switching carting off moves the urban share of 1780-99 by 18 per cent, a `labor`
of 0.5 by 9 and 0.75 by 3. Not so in the middle of the century, where carting off is worth a
fifth of the baskets of 1850 and half the carters a twentieth (and that not on most islands).
carting.md found 0.5 closing half the gap of 1850 on seeds 1 to 6 in the game of then; on these
sixteen islands it closes about an eighth.

What `adjust` is for, the month-about flip of a town's wage, is not a yearly fact, and was not
measured again here; `tests/carting.rs` holds it. Nor is the walk's own measure (the share of
hands within two cells of their market), so the walk could be doing to the map what works.md
says without a fact here seeing it.

## The settings that matter most

1. **`works.year_room`**, the one number of the life cycle that the economy answers to. It sets
   how hard the island invests, and the trade between the two ends is plain on every island: a
   smaller room starves the early iron trade (iron twice as dear in 1780-99 at 0.1125 on seeds 1
   to 8) and calms the century after (a seventh off the real wage's yearly swing, a fifth off
   timber's and the cost of living's, an eighth fewer closures); a larger room the reverse. The
   level of the real wage, goods and baskets per head does not move at either end, so works.md's
   reason for standing at 0.15 ("the steadier place") holds, and the choice is one of pacing and
   of how rough the ride is, not of how rich the island ends. Two things it moves that nobody had
   tied to it: the steel rail (a tenth of rails in 1890 at 0.1125, nearly none at 0.2: dearer iron
   makes steel worth rolling, which bears on R58), and the money level of the late century (the
   money wage and the cost of living both a sixth lower at 0.1125; not traced).
2. **`carting.labor`**, the dial for the first fifty years. Each quarter off the share takes about
   a fifth of the carters off the road and gives back roughly a quarter of what carting costs the
   island before 1800 (towns' share, industry's share, baskets in 1800, the money wage), almost
   in a straight line. It is the number to move if the plan's targets for 1750 to 1800 (R13, C92)
   are to be met with carting in. It does not move 1900, and it moves the middle of the century
   much less than carting as a whole does.
3. **`carting.enabled`**, for the size of what it costs: a fifth of the households' budget in
   1850, an eighth in 1800, a twentieth of the people in 1900, two points of urban share, and
   the railway a tenth shorter. By 1900 it costs about 1 per cent of baskets and 3 of goods.
4. **`works.hands_per_cell` and `works.cell_limit`** set the map: how many works stand, how large
   and on how many cells (industry on 140 to 283 cells in 1900 across the values tried). They are
   free of the economy at these values, so they can be set for the picture.
5. **`works.count_spread` and `works.family_extends`** set the churn: switched off, each brings
   back about 800 more foundings and 700 more closures a run, and the second the families' small
   works of the first half century (twice the works in 1800). Neither moves output or the wage.

## The settings that do nothing at these sizes

`works.built_share` (also nothing from 0.01 to 0.2 in works.md), `works.walk` in the yearly facts,
`works.entry_share` beyond the number of industry cells, `works.room_share` beyond the number of
works, `carting.max_share` (the cap binds in only a few town-months) and `carting.adjust` in the
yearly facts. Any of these could be left where it stands in a pass over the rules; `walk` and
`adjust` because what they are for is not among the facts, the rest because nothing moves.

## Where a run breaks

No run failed: 440 runs in the sweeps (and 48 in the pairs below), each with the monthly money
check on, no panic, no failed check. No town emptied: the fewest towns in 1900 in any run is 12,
which the game's own values give on seed 13. Famine years are at most 4 in a run, and one of the
runs at the game's own values has 4 too (a placebo, seed 6).

What did happen, in seven runs of 440 and in none of the 48 runs at the game's values, is a
runaway town wage. One town's wage doubles month after month (the most the labour market lets it
move) while its works go on hiring, for four or five months, and comes down as fast:

| setting | seed | when | the island's mean wage, that year and the next | the worst town |
|---|---|---|---|---|
| `works.year_room=0.1125` | 3 | 1751 | 14.1, then 6.3 (4.2 the year before) | Brambank, founded 1750: 121 times the middle town's wage in July 1751 |
| `works.year_room=0.1125` | 4 | 1785 | 22.6, then 8.7 (5.1 the year before) | Sunhaven, founded 1749: 47 times, June 1785 |
| `carting.labor=0.5` | 3 | 1751 | 17.0, then 6.8 | |
| `carting.labor=0.5` | 11 | 1750 | 49.5, then 11.7 | Melford, founded 1749: 100 times, July 1750 |
| `works.hands_per_cell=2667` | 6 | 1759 | 15.2, then 8.1 | |
| `carting.adjust=0.1875` | 6 | 1759 | 10.7 | |
| `carting.max_share=0.375` | 6 | 1760 | 6.9 (4.3 the year before), milder | |

The worst towns are from `examples/wage_watch.rs` (for example `wage_watch game 6 on 3 4
works.year_room=0.1125`); at the game's values the same seeds give 8.6 times at most (Brambank,
March 1751) and 1.6 on seed 11. On seed 4 the month's mean wage went 6.8, 9.1, 13.5, 22.4, 40.4,
76.2 and back down through 1785, the cost of living rose from 0.95 to 2.0 by 1787 and stayed
there, output fell 14 per cent and the chronicle has the people of a neighbouring town leaving
for Sunhaven, "where a household's income goes more than eight times as far". The real wage of
1785 reads 4.0 times 1750's and that of 1786 is 71 per cent below it, which is the largest
`real_wage_worst_year` of every run here.

It is the same few islands (seeds 3 and 6 five times between them) and nearly always a town's
first year or two, under settings of both mechanisms and in both directions, so it is not these
mechanisms' doing: these islands sit near an edge in the labour market and any small push can tip
them over. The nearest relative is carting's "wage that flipped", which the damper `adjust` ended;
this one is not a flip but a run in one direction. It is for whoever looks after the wage's
monthly limit and the hiring that runs ahead of it (`wage_watch`'s "overworked town-months": 7
and 3 on seeds 3 and 4 with the small room, none at the game's values).

It also means that a mean over eight islands can be carried by one run: the fact
`real_wage_worst_year` of the small room is -0.20 on seeds 1 to 8 for -0.15, and all of that is
seed 4 (-0.71; the other seven average -0.13).

## Pairs

Three pairs were run on seeds 1 to 8, each read against the single setting it adds to (so
against one run, with that run's luck in it; a *d* of 1 is chance here):

- **The year's room without carting** (`carting.enabled=false` with `works.year_room` 0.1125 and
  0.2). The room does what it does with carting: at 0.1125 iron 27 per cent scarcer and two thirds
  dearer before 1800 and the swings of the cost of living, the real wage and timber 13 to 18 per
  cent smaller (seven or eight islands of eight each); at 0.2 iron a fifth more plentiful and a
  tenth more closures. carting.md traced the slow middle of the century to the room rationing a
  smaller trade; if that were the whole of it, a wider room should help more with carting than
  without, and it does not: the gain in the middle of the century from a room of 0.2 is if
  anything larger without carting (goods per head in 1850 +23 per cent, eight islands of eight)
  than with it (+10 per cent, eleven of sixteen). The two add; neither undoes the other.
- **The year's room with half the carters** (`carting.labor=0.5` with the room): the same again,
  adding. The small room's swings and its dear early iron are there at the same size; carters in
  1900 are a fifth more with the small room (seven of eight), where with full carting they are an
  eighth more.
- **The cell's limit at 1.0 with the hands to a cell** at 1,500 and 2,667. The two numbers pull on
  the same thing and can be traded against each other: 1.0 with 1,500 gives the game's number and
  size of works (741, the median 6.1 usual sizes) spread over a quarter more cells (249 for 201);
  1.0 with 2,667 gives the fewest, largest works of any run here (615, the median 8.5, the largest
  193 usual sizes, on 127 cells) for the same goods, baskets and real wage as the game's. (1.0
  with 1,500 has baskets per head in 1900 6 per cent lower on seven islands of eight, *d* 1.0,
  which eight islands cannot tell from chance.)

Worth trying together in a pass over the rules, and not run here:

- `works.year_room` with the merchants' markup (`prices.md`), which works.md names as the other
  damper on the building cycle; the two were last swept side by side before carting, people
  and towns came in, and C3 asks for the markup to be swept again.
- `carting.labor` with the plan's targets for 1750 to 1800, read as baskets and goods per head
  (the ratios to a run's own 1750 hide carting): this is what R13 and C92 ask.
- `works.hands_per_cell` and `works.cell_limit` with kept town ground (`towns.works_room`), which
  decides how much of a town's ground the works may take; together the three set how much of the
  map is industry.
- `works.count_spread` off with `works.family_extends` off: each alone brings back 800 to 900
  foundings a run, and works.md's diagnosis names both among the causes of the port's thousands
  of small works; whether they add or overlap says how much of the churn each is answerable for.

## What I would look at first in a pass over the rules

1. **The runaway town wages.** Seven runs of 440, all near a town's first years on the same few
   islands, a town's wage doubling for five months and the island's money level shifting for good
   after it. It is the one result here that looks like a fault rather than a setting, and it will
   colour every other table of sensitivity, because any setting that tips one island over moves the
   means of eight. The seeds and settings are in the table above.
2. **`works.year_room` as the dial for the building cycle.** It is the only number of the life
   cycle that moves how rough the ride is, and it does so without moving the level of the wage. If
   the building cycle (works.md, "in company") is too large for the game, the room is where to
   turn, and the cost is a slower and dearer start in iron and tools. Its effect on the money level
   of the late century (both wages and prices a sixth lower at 0.1125, the real wage unchanged) is
   unexplained and should be understood before anything is read off a money price.
3. **`carting.labor` for the early targets.** The early economy answers to it nearly in a straight
   line, and nothing after 1850 does, so it can be set to meet the plan's early targets without
   moving the end of the century.
4. **That the slow middle of the century is carting as a whole, not its size.** Half the carters
   gives back half the early cost but only an eighth of the cost in 1850. That suggests a threshold
   (a trade that must reach some size before the machine-makers start) more than a smooth cost, and
   is worth a trace of one island through the 1830s and 1840s with and without carting.
5. **The numbers that set only the map** (`hands_per_cell`, `cell_limit`, `entry_share`,
   `room_share`) can be chosen for the picture; nothing in the economy holds them where they are.
6. **Facts the pass will need and does not have:** the share of hands within two cells of their
   market (for the walk), the share of town-months in which a town's wage moved by half (for the
   damper), and `mine_share_Y` without the carters in it (it is one less the farm and industry
   shares, so under carting it counts the carters: 0.216 in 1750 with carting and 0.109 without,
   so half of "quarries, pits and mines" in 1750 is carters).

## Where the runs are

Under `.claude/night/reports/opus/sens-works/` on the machine they were run on (not in the
repository): `base16` and the placebos, `s8-KEY` (seeds 1 to 8) and `s9-KEY` (seeds 9 to 16) for
each setting, `pair-*` for the pairs, and `readings-panel.txt` and `readings-top.txt` with every
reading quoted here.
