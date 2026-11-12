# sens-machines: how much the machines' and the harvests' numbers matter

One of six tables of sensitivity for the pass over all the rules (piece 18 of
`docs/backlog/found-along-the-way.md`: most mechanisms' numbers were set on a
few islands and never searched). This one is for the two mechanisms whose
pages are [machines.md](../rules/machines.md) and
[harvest.md](../rules/harvest.md). Nothing in the code was changed. Measured
on 12 November 2026 at a6ff0ec (main), on branch `night/o-sens-machines`.

## In short

1. **The ceiling on machine quality is the strongest number in either
   mechanism, and it is most of why growth stops.** `quality_cap` at 3, 4 and
   5.33 gives a late wage (the mean real wage of 1881 to 1900, 1750 = 1) of
   2.43, 2.68 and 2.92 on islands 1 to 8, and 2.52, 2.76 and 2.95 on islands
   9 to 16, with every island that reaches the cap agreeing. Quality reaches
   4 between 1863 and 1886 (1870 is typical). With no cap within reach
   the year output per head peaks moves from 1879 to 1895, the wage of 1900
   from 2.60 to 3.21, and four islands of sixteen are still growing in 1900.
   This is the counterfactual the backlog asked for (R121): the cap is most of
   the "growth comes early and stops", though four islands still peak in the
   1880s without it.
2. **`inherit` is the steel rail's switch.** At 0.375 no island lays steel
   rail; at the game's 0.5 a few lay a little (0.2 per cent of rails in 1890
   on islands 1 to 8, 7 per cent on 9 to 16); at 0.667 fifteen islands of
   sixteen do (the sixteenth has no converter until 1886), and steel is a
   fifth to nine tenths of all rail by 1890, two thirds on average. The
   converter does not care. Nothing else moves.
3. **Wagonways bought where they pay (`adopt_wagonways`) leave the island
   with no railway at all**, on all sixteen islands, and cost a tenth of the
   late wage. The page says the switch is off for this reason; it is worse
   now than when the page measured it.
4. **The quarries' two numbers matter and nobody has tuned them.**
   `quarry_reach` moves stone by a fifth either way and the late wage by 0.08
   to 0.15, on fifteen islands of sixteen each way. `quarry_input` moves the
   engine trade by an eighth and nothing beyond it.
5. **Most of the rest do nothing at these sizes**, beyond the noise every
   change makes (below): `adopt_span`, `life_months`, `quality_Q0`, and of the
   harvest `district_sd`, `reach`, `pasture` and `woods`. The harvest's size
   (`island_sd`) and `foreseen` move the year-to-year swings as the page says
   and nothing in the long run. Its three news numbers change the news and
   not one other fact.
6. **Nothing broke.** 416 runs, none failed, the books of every one balance
   to a millionth of a millionth, no town starved (the most famine years in
   any run is four). One thing is fragile instead: on one island of sixteen
   (seed 3) crucible steel never takes off in about one history in five,
   and with no steel the machines never improve and the island has the
   port's flat century, a late wage of 1.8 where the game gives 2.6.

## How it was done

Each setting was run at about three quarters and one and a third of its
value (a switch flipped; `woods`, which is 0, at 0.25), on islands 1 to 8,
six families, the game's rules, against a sweep of the game's rules as they
stand. Results I would stake a decision on were run again on islands 9 to 16
against a baseline of their own. For example:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --out runs/game
isostate-core sweep --seeds 1-8 --rules game --families 6 --set machines.inherit=0.375,0.667 --out runs/m-inherit
isostate-core sweep --seeds 9-16 --rules game --families 6 --set machines.quality_cap=3,5.33 --out runs/m-quality_cap-s9-16
```

The baseline on islands 1 to 8 is the same as main's on every one of its 446
facts ("0 differ" against the night's merged sweep). Each setting was read
against it island for island with a small script: for every fact, and for
twenty-year means of the yearly series (1780-99, 1820-39, 1840-59, 1860-79,
1881-1900), the mean change, the spread between islands, and how many islands
went up and how many down. The scripts, every comparison and every number are
in the notes of the session, beside the sweeps under
`.claude/night/reports/opus/sens-machines/`.

**How to read a movement here.** Almost any change to a number that acts
before 1800 sends every island down a different history: 500 to 590 of the
611 facts and series move, most of them by chance. So I believe a movement
only when seven or eight islands of eight agree, when the two directions move
opposite ways, and when it stands clear of what the "neutral" settings move
it by. Those are nine runs whose settings should not shift the long run
(`life_months` both ways, and the harvest's `district_sd`, `reach`, `pasture`
both ways and `woods`). Their median and largest movements are the floor a
real answer must clear:

| fact (islands 1 to 8) | game | median movement | largest |
|---|---|---|---|
| late wage, 1881-1900 | 2.68 | 0.05 | 0.12 |
| wage, 1840-59 | 2.05 | 0.05 | 0.12 |
| chained wage, 1900 | 4.24 | 0.09 | 0.20 |
| output per head, 1900 | 2.65 | 0.10 | 0.12 |
| year output per head peaks | 1879 | 4 years | 6 years |
| towns' share of people, 1900 | 0.419 | 0.001 | 0.008 |
| people, 1900 | 731,900 | 3,900 | 7,100 |
| machinery made a month, 1881-1900 | 2,890 | 22 | 124 |
| machinery's price, 1881-1900 | 53.9 | 2.7 | 6.5 |
| steel rails' share of rails, 1881-1900 | 0.015 | 0.03 | 0.23 |
| farm zigzag | 2.08 | 0.05 | 0.25 |
| alarms told | 18.0 | 1.4 | 2.0 |

Two traps showed up at once. Machinery's price in 1860-79 rose under
`life_months` 72 and 128 alike and under a smaller harvest: the baseline's
41.5 is a lucky low, and one island's price trebles under almost any change.
And steel rails' share jumps from nothing to a third on one or two islands
under almost any change, because the steel rail mill sits on the edge of
taking hold (point 2 above says which number decides it).

## Machines

Islands 1 to 8 unless said; "16" means islands 9 to 16 were run as well and
the count is over all sixteen. A late wage is the mean of 1881 to 1900.

| setting | game | tried | what moved most, and by how much | islands agreeing |
|---|---|---|---|---|
| `enabled` | true | false | late wage 2.68 to 1.73; chained wage of 1900 4.24 to 2.43; output per head of 1900 2.65 to 1.81; towns' share 0.419 to 0.339; machinery's price 1881-1900 54 to 107; the ninth tenth of works 37 to 54 (the machines still make the top works smaller) | 8 of 8 in each |
| `adopt_span` | 6 | 4.5, 8 | nothing clear of the floor. At 8, machinery's price 1881-1900 54 to 47 and the chained wage up 0.17; at 4.5, machinery made 2 per cent more | 7 of 8 at most, inside the floor |
| `adopt_pumps` | false | true | the island of 1750 itself, since the warm-up runs with it: coal workings 34 to 50, coal's price in wages down 15 per cent; wage of 1780-99 1.52 to 1.42; people of 1900 12,300 fewer (1.7 per cent). The late wage does not move clear of the floor | 8 of 8 in each |
| `adopt_wagonways` | false | true | **no railway on any island**: the rolling mill is never found on 14 of 16 (1872 and 1879 on the other two), the railway never; late wage 2.68 to 2.47 and 2.76 to 2.44; output per head of 1900 2.65 to 2.43 and 2.69 to 2.42; chained wage 4.24 to 3.57; the canals carry on to the end | 16 of 16 without a railway; late wage lower on 15 of 16 |
| `life_months` | 96 | 72, 128 | nothing; one of the settings the floor is measured on | |
| `quality_Q0` | 1,000,000 | 750,000; 1,333,333 | nothing clear on the wage (0.07 either way). At 1,333,333 output per head peaks ten years later (1879 to 1889) | 7 of 8 for the peak |
| `quality_b` | 0.6 | 0.45, 0.8 | 0.45: wage of 1860-79 2.65 to 2.34, machinery made in 1860-79 down 16 per cent, grain down 10 per cent, late wage 2.51, peak 1890. 0.8: wage of 1840-59 2.05 to 2.18, late wage unchanged (the cap is met sooner) | 8 of 8 in 1860-79 at 0.45; 7 of 8 in 1840-59 at 0.8 |
| `quality_cap` | 4 | 3, 5.33 | a clean experiment: each run is the game's own until quality reaches the cap. 3: late wage 2.68 to 2.43 (2.76 to 2.52 on 9 to 16), chained wage of 1900 down 0.45 and 0.50, stone made down a quarter, grain down 8 per cent, towns' share down 0.011. 5.33: late wage 2.92 and 2.95, output per head of 1900 2.92 and 2.96, peak 7 and 10 years later | 16: lower on 16 of 16 at 3, higher on 15 of 16 at 5.33 (seed 3 never reaches 4) |
| `quarry_reach` | 1 | 0.75, 1.33 | stone made 1881-1900 down 19 and up 24 per cent (up 28 on 9 to 16); stone's price in wages up 15 and down 13 per cent; machinery made down 6 and up 9 per cent; late wage down 0.08 and up 0.09 (down 0.14 and up 0.15 on 9 to 16) | 16: stone 16 of 16 each way; late wage 15 of 16 each way |
| `quarry_input` | 0.001 | 0.00075, 0.00133 | machinery made 1881-1900 down 13 and up 17 per cent (up 13 on 9 to 16); at 0.00075 its price 54 to 64 (53 to 61). On islands 1 to 8 the wage of 1900 rose 0.20 at 0.00133, on 9 to 16 it did not move (4 up, 4 down): chance | 16: machinery 16 of 16 each way |
| `inherit` | 0.5 | 0.375, 0.667 | steel rails' share of rails 1881-1900: 0.015 to 0.001 and 0.56 (0.071 to 0.002 and 0.69 on 9 to 16); in 1890 0.002 to 0 and 0.53, and 0.07 to 0 and 0.68; rails' price 1881-1900 down 14 per cent at 0.667. The converter's year and share untouched; the wage untouched | 16: 15 of 16 at 0.375 (one island had none to lose), 16 of 16 at 0.667 |

The content's numbers (`MACHINES_PLANT_SCALE`, `MACHINES_STEEL_RAIL_WORTH`
and `MACHINES_THRESHOLDS` in `content.rs`) are constants, not settings, and
`--set` cannot reach them; they were not swept.

**No cap within reach** (`quality_cap` 100, sixteen islands, beyond the
three-quarters-and-a-third brief because it answers R121):

| | islands 1 to 8 | with no cap | islands 9 to 16 | with no cap |
|---|---|---|---|---|
| year output per head peaks | 1879 [1865 .. 1898] | 1895 | 1878 | 1893 |
| real wage of 1900 | 2.60 | 3.21 | 2.63 | 3.14 |
| chained wage of 1900 | 4.24 | 5.33 | 4.33 | 5.43 |
| output per head of 1900 | 2.65 | 3.16 | 2.69 | 3.11 |
| towns' share of people, 1900 | 0.419 | 0.432 | 0.420 | 0.437 |
| machinery made a month, 1881-1900 | 2,890 | 3,870 | 2,920 | 3,800 |

Every island that reaches quality 4 agrees (fifteen of fifteen; seed 3 never
gets there). Four islands peak in 1900 itself, so they are still growing when
the run ends; four (7, 11, 15 and 16) still peak between 1882 and 1889, so the
cap is not the whole story. Famines, the swing of grain's price and the money
are as they were. And a flatter curve does not do the same: `quality_b` 0.45
with the cap at 5.33 or 100 (the two runs are identical, since quality never
reaches 5.33) moves the peak to 1893 but lowers the wage all the way, 2.51
late for 2.68.

## Harvests

Islands 1 to 8. Today's game, for reference: the country's harvest varies by
0.051 a year; a harvest ten per cent short brings in seven per cent less
grain (slope 0.71), sells it ten per cent dearer (-1.00) and takes three per
cent off what a wage buys (0.285 [0.15 .. 0.40]); famine comes in 0.4 years
of a run [0 .. 1]; a run tells 8.2 dearths, 15.2 harvest items and 18 alarms.

| setting | game | tried | what moved most, and by how much | islands agreeing |
|---|---|---|---|---|
| `enabled` | true | false | grain output's swing 0.096 to 0.022; grain price's swing 0.109 to 0.039, its `ac1` -0.58 to +0.11; farm zigzag 2.08 to 1.12; cloth's price swing 0.039 to 0.029; alarms 18 to 10; worst year for the wage -0.143 to -0.118. Nothing in the long run; famine years 0.375 to 0.25 | 8 of 8; 7 of 8 for the worst year |
| `island_sd` | 0.05 | 0.0375, 0.0667 | grain output's swing 0.073 and 0.126; grain price's swing 0.088 and 0.133; farm zigzag 1.81 and 2.53; dearths told 5.1 and 11.1; alarms 13.4 and 20.2. Nothing in the long run | 8 of 8 for the swings and dearths; 7 and 8 of 8 for the zigzag |
| `district_sd` | 0.05 | 0.0375, 0.0667 | nothing clear of the floor | |
| `reach` | 30 | 22, 40 | nothing clear of the floor | |
| `pasture` | 0.4 | 0.3, 0.533 | farm zigzag 2.33 at 0.533; nothing else | 7 of 8 |
| `woods` | 0 | 0.25 | nothing clear of the floor | |
| `foreseen` | 0.5 | 0.375, 0.667 | grain output's swing 0.088 and 0.107; farm zigzag 2.01 and 2.36; price swing 0.104 and 0.116; grain output on the harvest 0.71 to 0.63 and 0.78; the real wage on the harvest 0.285 to 0.24 and 0.33. Nothing in the long run | 7 and 8 of 8 for the swings; 6 to 8 of 8 for the slopes |
| `news_z` | 1.8 | 1.35, 2.4 | harvest items told 15.2 to 32.9 and 7.1; not one other fact or yearly series moves | 8 of 8 |
| `news_local_z` | 2.3 | 1.725, 3.07 | harvest items told 15.2 to 33.6 and 14.0; nothing else | 8 of 8; 5 of 8 (three unchanged) |
| `dearth_rise` | 0.15 | 0.1125, 0.2 | dearths told 8.25 to 14.6 and 3.25; nothing else | 8 of 8 |

Two things here differ from the page. Dearths are told 8.2 times a run at
the line of 0.15, where the page says 5.5: the island has moved since the
harvests were merged (grain's price now swings by 0.109 a year, not 0.100).
And the harvest no longer doubles the famines, because there are hardly any
to double: 0.375 years a run with it, 0.25 without.

## The settings that matter most

1. **`quality_cap`**, by a distance: a quarter of the late wage between 3 and
   5.33, on every island, and the date when growth stops.
2. **`adopt_wagonways`**, because it takes the railway away. It is a switch
   that is off for that reason, and should stay off until the collieries'
   first wagonways are paid for some other way.
3. **`inherit`**, for the steel rail and nothing else, sharply: the game's
   0.5 sits just below the line where steel rail takes hold.
4. **`quarry_reach`**, for stone and clay and through them the late wage
   (0.1 either way, every island but one).
5. **`quality_b`**, mostly in the middle of the century (the wage of 1860-79
   moves by 0.3), since the cap decides the end.
6. **`island_sd`** and **`foreseen`**, for the year-to-year swing of grain,
   the farm zigzag and the dearths and alarms the news tells.
7. **`adopt_pumps`**, for the island of 1750: more coal mines and a lower
   wage before 1800, and slightly fewer people in 1900.

## The settings that do nothing at these sizes

`adopt_span` (4.5 to 8), `life_months` (72 to 128) and `quality_Q0` (750,000
to 1,333,333) among the machines; `district_sd`, `reach`, `pasture` and
`woods` among the harvests. Each sends the islands down other histories,
and no fact moves clear of the floor with the islands agreeing, except the
year of the peak at the larger `quality_Q0` and the zigzag at the larger
`pasture`. The three news numbers (`news_z`, `news_local_z`, `dearth_rise`)
change the counts of their own news and nothing else: the run is the same
to the last digit. `adopt_span` was set to calm the machinery market; on
today's island the merchants' prices already do that, and at 4.5 or 8 the
market is as calm as at 6.

## Values at which a run breaks

None of 416 runs failed. The books of every run balance to within a
millionth of a millionth (the largest `books_income_worst` of any run is
1.0e-12, the largest `books_goods_worst` 3.7e-14). No run had
more than four years of famine, and no town was emptied. Three things go
wrong short of breaking:

- **`adopt_wagonways` on**: no railway on any island (above).
- **A lower cap brings back a machinery famine on one island.** Seed 6 at
  `quality_cap` 3 leaves the game's own run in 1855, when its quality reaches
  3, and follows it closely until 1878; then machinery's price goes from
  about 50 to 167 and 207 (1880-82), the cost of living from 0.95 to 1.65,
  and the wage of 1882 is 1.33 where the game has 2.22. Machinery is dear
  again in 1896-98 and the wage of 1900 is 1.91 for the game's 2.82. I did
  not trace why a lower ceiling should do this.
- **Seed 3 stalls.** In 8 of the 38 settings that change something,
  crucible steel on seed 3 never takes off: one to a hundred units a month in
  1840-59, where the game's own run of that island makes 724 and the other
  islands about ten thousand. Quality counts all the steel the island has
  ever poured, so it stays near 1, and the late wage is 1.8 to 2.2 for the
  game's 2.58. It happens under harvest settings as readily as machine ones
  (`foreseen` 0.667, `island_sd` 0.0375, `reach` 22 and 40, `woods` 0.25,
  `pasture` 0.533, `quality_b` 0.45, `quality_Q0` 750,000), so it is not any
  one number: that island sits on a knife's edge, and the machines' whole
  century hangs on one learning curve's start. No island of 9 to 16 stalls in
  any setting.

## Pairs worth trying together

- **`quality_cap` with `quality_b` and `quality_Q0`**, on many islands, with
  growth after 1880 as the target. The cap decides the end and the exponent
  the middle; tried here only as `quality_b` 0.45 with a cap out of reach,
  which delays the peak but lowers the whole path. A curve that flattens
  without a ceiling, as the machines page asks, is the same question.
- **`inherit` with the content's `MACHINES_STEEL_RAIL_WORTH`** (which needs
  a content file or a code change): the first decides whether steel rail
  takes hold and the second what it is worth once it has.
- **`quarry_reach` with `quarry_input`**: what a quarry's machine adds and
  what it costs. Only the first moved the wage; together they decide whether
  the quarries are a second engine of growth or a customer of the first.
- **`adopt_wagonways` with the rolling mill's discovery threshold**, if the
  switch is ever to come on: the railway is lost because the rails the
  collieries lay at a loss are what teach the island to roll them.
- **`island_sd` with `foreseen`**: the size of a bad year and how much of it
  the wage feels. Both move the swings in the same direction.

## What I would look at first in a pass over the rules

1. **The cap on machine quality.** It is set at 4 by analogy with the
   methods' `learn_cap`, and it is reached between 1863 and 1886 on fifteen
   islands of sixteen; output per head peaks about a decade later. Lifting it is the single largest change in this table and answers
   most of R121. Something has to stop machines improving for ever, but a
   wall reached in 1880 makes the last twenty years of every game a slow
   decline in output per head.
2. **Why seed 3's crucible steel can fail to start**, and whether the
   machines should learn from all metal-working and not from steel alone. One
   island in sixteen spending a century as the port is not a fair game.
3. **`inherit` and the steel rail.** As it stands steel rail is a sliver of
   the trade on most islands; a value near 0.6 would give it the place the
   machines page meant it to have, and should be measured with the works'
   life cycle on, as that page asks.
4. **Seed 6's machinery famine at a cap of 3.** Small, but it says the
   machinery market can still run away late in the century, which is what
   the machines were built to stop.
5. **The quarries' numbers**, which matter as much as `quality_b` and were
   never searched.
