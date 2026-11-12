# Sensitivity: people and the ground towns keep

A table of sensitivity for two of the game's mechanisms, people
([rules/people.md](../rules/people.md)) and kept town ground
([rules/towns.md](../rules/towns.md)): every number each has as a setting,
moved to about three quarters and four thirds of its value one at a time, on
eight islands, and read against today's game on the same islands. It is one of
six such tables for the pass over all the rules (piece 18 of
`docs/backlog/found-along-the-way.md`; the rows behind it are R68, R69, R77,
R99 and R103). Nothing in the simulation was changed to make it.

Measured on 12 November 2026 at a6ff0ec (main after the canals, the mines, the
books of account and saved games), under the game's rules with six families.
The short answer: three numbers decide the island's people (`growth_top`,
`y_still` and `ease`, with `town_deaths` a little behind), and through the
people they decide how much town ground, how many works and what a head earns
late in the century; four decide how many towns there are (`found_saving`
loosened, or `donor_max`, `donor_rest` and `found_people` tightened); three
decide how much ground the towns stand on (`d_old_growth`, `d_old` and `tau`);
and about half of the rest do nothing that can be told from the islands' own
differences at these sizes, two of them nothing at all. No run of 864 failed.

## How it was measured

The baseline is today's game on islands 1 to 8, which the sweep confirmed is
main to the last digit:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --out .../base --compare .../merged-game-f6-l
#  of the 446 facts both have, the seconds aside, 0 differ
```

Then one sweep for each setting, with its values side by side on the same
eight islands:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set people.growth_top=0.012,0.0213 --out .../s8/people.growth_top
```

A value that is a count was rounded (`found_cells` 30 and 53, `donor_rest` 6
and 11, `decline_years` 11 and 20). A share of need stayed a share that means
something: `dearth_from` went to 0.8 and 1.1, since four thirds of 1.0 would
call a month with a third to spare a dearth. Each switch was flipped. Three of
the port's own numbers that these two rules read directly under the game's
rules were swept too, because a pass over the rules will ask about them:
`migration_rate` and `migration_dist` (the migration that the people rule keeps
and limits) and `urban_rent_rate` (the rent a town's residents pay, which kept
ground shares among its cells). `y_still` was also run at 1.5, the value it was
reviewed at.

Where a result looked worth staking a decision on, the same sweep was run on
islands 9 to 16 against a baseline of its own, and the two halves pooled:
sixteen settings in all. Then three pairs, and six settings far outside the
range, to look for where a run breaks: 864 runs in all. The machine was shared
with two dozen other workers, so a sweep took from one to eighteen minutes.

### How to read the tables

A run here is chaotic: change any number at all, even one that does nothing to
speak of, and 380 to 460 of the 446 facts move a little somewhere. So the
count of facts that differ means nothing, and every movement is read against
how much the islands differ among themselves.

- **z** is the mean change, island for island, over the spread of that fact
  across the eight (or sixteen) islands of the baseline (their standard
  deviation). A z of 1 is a change as large as the usual difference between
  two islands.
- **8/0** says the fact rose on eight islands and fell on none. A star means
  every island now stands clear of every island before, as in the sweep's own
  table.
- Wages and output are read as **twenty-year means** from `annual.csv`, as the
  towns page asks: the chained real wage and output per head (the observer's
  `real_wage_chained` and `gdp_pc`) over 1840-59 and 1880-99. The single year
  1900 is given only where it says something the means do not.

What nothing looks like: `people.dearth` at 0.04 moves no fact on seven
islands of eight by a z of a half or more. Setting a number that does nothing
against the baseline still gives three or four facts at 7/1 and a z of about
0.5, by chance. So below, a movement is **real** at 8/0 (or 0/8) with a z of 1
or more, or with a star; a **lean** at 7/1 with a z between a half and one;
and anything less is **nothing**.

The baseline is also one draw among many. Set against a cloud of 128 runs of
the same eight islands nudged by sixteen settings that do nothing material
(the dearth's, emigration's, `arrivals_max`, `found_spacing`, `empty`,
`decline_years`), the spread *within one island* from nudges alone is: towns
of 1900, 1.3; towns founded from 1800, 1.2; the real wage of 1900, 0.19 times
1750's (7 per cent); output per head of 1900, 0.15; railway of 1900, 93 km;
people of 1900, 5,700; town cells of 1900, 14; works of 1900, 36. And today's
baseline on islands 1 to 8 happens to sit high on towns: 5.1 founded from
1800 where the cloud has 4.3, and 17.8 towns in 1900 for 17.1. So almost any
setting read against it shows "fewer late towns", and I give `late_towns`
no weight unless sixteen islands agree. For the numeric settings the tables
therefore read the higher value against the lower, island for island, so that
the baseline's own luck drops out, and give each value against today where it
matters.

## People

Eight islands unless "16" says sixteen. "High against low" is the four-thirds
value against the three-quarters value. People are thousands.

| setting | value | tried | what moved most, high against low | islands agreeing | verdict |
|---|---|---|---|---|---|
| `enabled` | on | off | Against today: famine years 0.4 a run to 7.6; people of 1900 732 to 669 (-8.6%, z -4.2, starred); last town founded 1807 for 1847; the share of town-years at the port's 1.1 per cent 0.22 to 0.96. 16: famine 0.4 to 6.4, people -54, the 1880s real wage +4% and output per head +2% | 7/0, 0/8, 0/8, 8/0; 16: 13/0, 0/16, 12/4, 12/4 | the rule holds hunger off; it costs about 4 per cent of the late wage, through more people |
| `growth_top` | 0.016 | 0.012, 0.0213 | People of 1900 +875 (more than today's whole island), town cells +171%, works +111%, railway +684 km (eight islands); output per head 1880s -33%, chained real wage 1880s -20%; farm land -4% | 15/1 or better on all of them (16), the first three starred | the strongest number here |
| `y_still` | 1.3 | 0.975, 1.5, 1.733 | People of 1800 -68 (z -10), of 1900 -309 (-12.6); town cells -35%, works -26%; output per head 1880s +15%, real wage +7% | 0/16*, 0/16*, 0/16*, 0/16, 16/0, 13/3 (16) | strong; it sets the start |
| `ease` | 2.1 | 1.575, 2.8 | People of 1800 +28, of 1900 +192; town cells +32%, works +20%; output per head 1880s -9%, real wage -6% | 16/0*, 16/0*, 16/0*, 15/1, 0/16, 1/15 (16) | strong |
| `town_deaths` | 0.012 | 0.009, 0.016 | People of 1900 -116 (z -4.7); town cells -14%, works -11%; output per head 1880s +4%, urban share 1880s +2% | 0/16*, 0/16, 1/15, 14/2, 15/1 (16) | real, about three fifths of what `ease` does to people |
| `town_size` | 10,000 | 7,500, 13,333 | People of 1900 +47 (z 3.1); output per head 1880s -4% | 8/0*, 1/7 | real for people, a lean for the rest |
| `dearth_from` | 1.0 | 0.8, 1.1 | Works of 1900 -7% (z -1.2); urban share 1880s -2% | 0/8, 2/6 | a lean; 1.1 calls ordinary years lean |
| `dearth_full` | 0.6 | 0.45, 0.8 | nothing | | nothing |
| `dearth` | 0.03 | 0.0225, 0.04 | nothing: no fact at 7/1 or better with z 0.5 | | nothing |
| `emigration` | 0.02 | 0.015, 0.0267 | nothing; some 130 people a run sail | | nothing |
| `growth_floor` | -0.06 | -0.08, -0.045 | every fact of every island exactly as today | | never binds |
| `arrivals_max` | 0.1 | 0.075, 0.133 | Towns of 1900 +7% (z 0.5) | 7/1 | nothing to speak of |
| `found_saving` | 0.08 | 0.06, 0.107 | Towns of 1900 -31% (z -2.7), works -12%, people of 1900 -19; real wage 1880s +4%. Against today, 16: 0.06 gives towns +17%, 0.107 gives -19% | 0/16, 2/14, 1/15, 14/2; 13/0, 1/14 | the dial for the number of towns |
| `found_cells` | 40 | 30, 53 | 53: every fact of every island exactly as today. 30: one island of eight moves at all | | never binds upward |
| `found_people` | 3,000 | 2,250, 4,000 | Towns of 1900 -16%, towns of 1800 -18%, works in the 1780s -12% (starred). Against today, 16: 2,250 nothing, 4,000 towns -13% | 1/15 (16), 0/8, 0/8; 0/11 | binds only when raised |
| `found_spacing` | 8 | 6, 10.7 | Works of 1900 +6% (z 1.0) | 7/1 | nothing to speak of |
| `donor_max` | 0.2 | 0.15, 0.267 | Towns of 1900 +33%, works +12%, people of 1900 +22. Against today, 16: 0.15 towns -19%, 0.267 +8% | 15/1, 15/1, 15/1 (16); 0/14, 8/4 | binds when lowered, a little when raised |
| `donor_rest` | 8 | 6, 11 | Towns of 1900 -16%, works -9%. Against today, 16: 6 nothing, 11 towns -15% | 2/12, 3/13 (16); 3/11 | binds only when raised |

What the people numbers do, in a sentence: the four that set births less deaths
(`growth_top`, `y_still`, `ease`, `town_deaths`) set how many people there are,
and the number of people sets nearly everything else that moves. More people
means more town ground, more works and more railway, and less output and a
lower wage per head. Across all four, output per head in the 1880s moves
about a third of a per cent the other way for each per cent of people in 1900
(`growth_top` 0.0213: people +83 per cent, output per head -24 per cent;
0.012: -37 and +14; sixteen islands each). The towns' share of the people
hardly moves with any of them (a point or two). The famine years do not move:
the island of today is fed.

The founding rule has five gates, and they do not bind alike. Loosened one at
a time, only `found_saving` lets more towns through; the others hold back what
it would let through, so loosening any of them alone does little, and
tightening any of them cuts the towns by 13 to 20 per cent. Two never decide at
these sizes: `found_cells` (a district of 53 cells is always there when the rest
pass) and `found_spacing`.

## Kept town ground

| setting | value | tried | what moved most, high against low | islands agreeing | verdict |
|---|---|---|---|---|---|
| `enabled` | on | off | Against today, 16: town cells of 1900 324 to 129 (-60%, starred); industry cells +25%; farm land +3.1%; first engine works 22 years earlier; urban share of the 1880s +0.5 of a point. The chained real wage of 1880-99 4.35 with kept ground and 4.32 without; output per head 8.38 and 8.40 | 0/16*, 15/1, 16/0, 0/16, 15/1; 10/6, 11/5 | the ground and the farm land, not the wage |
| `d_new` | 300 | 225, 400 | Town cells of 1900 -10%, of 1750 -4% | 0/7, 0/8 | a little ground |
| `d_old` | 530 | 400, 707 | Town cells of 1750 -40% (z -27), of 1800 -32%, of 1900 -15% | 0/8* each | the first towns' size, fading by 1900 |
| `d_old_growth` | 6 | 4.5, 8 | Town cells of 1900 -27% (z -4.2, starred), farm land +2%, works +6%; wage and output nothing | 0/16*, 15/1, 12/4 (16) | the dial for town ground, and only that |
| `tau` | 30 | 22.5, 40 | Town cells of 1900 +18% (starred), of 1750 +5% | 8/0*, 8/0 | ground, as `d_old_growth` but weaker |
| `works_room` | 0.5 | 0.375, 0.667 | Town cells of 1850 -9% (z -0.9) | 1/7 | nothing to speak of |
| `smooth` | 0.5 | 0.375, 0.667 | Urban share 1880s -2% (z -1.1), works of 1900 +7%. Against today, 16: 0.375 works -6% | 0/16, 14/2 (16); 1/15 | the count of town people, and a lean on works |
| `empty` | 0.6 | 0.45, 0.8 | The ten-year churn of town ground, which is near nothing either way (0.0006 to 0.007) | 8/0* | nothing else |
| `decline_years` | 15 | 11, 20 | nothing | | nothing |
| `back_years` | 60 | 45, 80 | Town cells of 1750 -4%; works of the 1780s +9%. The single year 1900's real wage +10% (7/1), but its twenty-year mean +3% (6/2) | 0/7, 7/1 | a little in 1750; see below on the single year |
| `back_span` | 40 | 30, 53 | nothing | | nothing |
| `mean_hands` | on | off | Against today: works in the 1880s -8% (z -2.4), towns of 1900 -13%. 16: works of 1900 -4%, towns -7% | 0/8, 0/7; 6/10, 3/11 | a lean that fades on sixteen |
| `donor_wage` | on | off | Against today, 16: the first engine works 20 years earlier (on eight islands the first rolling mill 21 years earlier too); nothing else | 2/14 | as the towns page says |
| `rent_floor` | on | off | Against today: chained real wage of 1900 +3.4%, industry cells of 1750 -68%. 16: the wage +1% | 7/1, 0/7; 11/5 | the wage fades on sixteen; the 1750 works remain |

And the port's numbers these two rules read:

| setting | value | tried | what moved most | islands agreeing | verdict |
|---|---|---|---|---|---|
| `migration_rate` | 0.05 | 0.0375, 0.0667 | nothing past noise | | nothing |
| `migration_dist` | 60 | 45, 80 | Against today: 80 gives the single year 1900's real wage +6% (7/1), twenty-year mean +1.5%. 16: +1% (12/4) | | nothing |
| `urban_rent_rate` | 0.12 | 0.09, 0.16 | Land's share of the island's income in 1900 +18% (z 8.6, starred); town cells of 1900 -9%; people of 1800 -2% | 8/0*, 0/8, 0/8 | real, but in the books more than on the map |

Kept ground's numbers mostly do what they say and stop there. `d_old_growth`,
`d_old` and `tau` set how much ground the towns stand on, and nothing else
moves past the spread: not the wage, not output, not the number of towns or of
people. The town cells of 1900 are 324 today on sixteen islands; `d_old_growth`
at 4.5 or 8 makes them a sixth more or fewer, at a point or two of farm land
and no visible cost elsewhere.

## The settings that matter most

1. **`growth_top`**, then **`y_still`**, then **`ease`**. Births less deaths
   decide the island's people, and the people decide the rest. A third off
   `growth_top` takes 268,000 people from 1900 and adds 14 per cent to output
   per head in the 1880s; a third on adds 608,000 and takes 24 per cent (16
   islands: the people on every island, starred; output per head on 15 and 16
   of them). These are the two numbers the people page
   says were set again on six seeds (R69), and everything that has moved 1750
   or 1900 since (mines, canals) moves where they should stand.
2. **`found_saving`**, with **`donor_max`**: the number of towns. 0.06 gives 17
   per cent more towns in 1900 and 0.107 gives 19 per cent fewer, on sixteen
   islands, and works go with them (+3 and -9 per cent). The real wage leans
   the other way: more towns, a little less wage (0.107 against 0.06: +4 per
   cent in the 1880s, 14 islands of 16).
3. **`town_deaths`**: 116,000 people between 0.009 and 0.016, all sixteen
   islands, and with them a seventh of the town ground.
4. **`d_old_growth`**: a quarter of the town ground between 4.5 and 8, all
   sixteen islands, and nothing else.
5. **`urban_rent_rate`** (the port's): land's share of income. It matters to
   the books of account and to who is rich, not to the shape of the island.

## The settings that do nothing at these sizes

- **Never bind**: `growth_floor` (births less deaths never fall to -4.5 per
  cent a year in any town) and `found_cells` raised to 53. Both give every fact
  of every island exactly as today.
- **Nothing that can be told from the islands' own differences**: `dearth`,
  `dearth_full`, `emigration` and `arrivals_max` among people; `works_room`,
  `decline_years`, `back_span` and `empty` among the towns (`empty` moves the
  churn of town ground, which is near nothing either way); `migration_rate`
  and `migration_dist`. The dearth's numbers have nothing to work on: the game's
  island has 0.4 famine years a run, so a rule for hungry years is idle in
  almost every year. That changes if the people rule is switched off (7.6) or
  if a later rule makes the island hungrier.
- **Bind only one way**: `found_people`, `donor_rest`, and to a lesser degree
  `donor_max`, cut the towns when tightened and do little when loosened.
- **Leans that faded on sixteen islands**: `rent_floor` off (the real wage of
  1900 +3.4 per cent on eight islands, +1 per cent on sixteen), `mean_hands`
  off (works -8 per cent in the 1880s on eight, -4 per cent on sixteen), and
  `migration_dist` (+6 per cent on 1900's wage on eight, +1 per cent on
  sixteen).

## Where a run breaks

Nowhere in this range. 864 runs, no panic, no failed money check. Six
settings far outside it, on eight islands each:

| setting | what happened | failed |
|---|---|---|
| `growth_top` 0.032 (twice) | 4.3 million people in 1900; output per head in the 1880s -81 per cent, real wage -48 per cent; town ground four times; towns +42 per cent | 0 of 8 |
| `y_still` 0.65 | 977,000 people; output per head -10 per cent | 0 |
| `town_deaths` 0.03 | 496,000 people; output per head +11 per cent | 0 |
| `found_saving` 0.03 | towns +49 per cent (starred), works +20 per cent, last founding 36 years later; the 1880s wage -4 per cent (0/8) | 0 |
| `d_new` 100 | town cells +10 per cent | 0 |
| `d_old_growth` 0 | town ground more than doubles (+123 per cent, starred), farm land -6 per cent (starred), the wage unmoved | 0 |

Doubling `growth_top` is Malthus and not a crash, as the people page found on
the port's island. The facts cannot say whether a famine empties a town (no
fact counts a town's least people), but no setting here raised the famine
years above 1.25 a run except switching the people rule off (7.6).

## Pairs worth trying together

Run on eight islands.

- **`growth_top` and `y_still`** are not the same dial. `y_still` sets the
  start, and `growth_top` the end. At 0.014 and 1.15 the island has as many
  people in the 1780s and in 1800 as today (+1,300 and -1,200) and 113,000
  fewer in 1900, with output per head in the 1880s 3 per cent higher (7/1).
  At 0.018 and 1.45 it has 4,700 fewer in the 1780s and 97,000 more in 1900.
  At 0.018 and 1.15 everything is more: 267,000 more people, output per head
  -13 per cent. So if the island of 1900 should be smaller without its first
  years changing, lower both together.
- **The founding gates.** `found_saving` 0.06 alone gives 16 per cent more
  towns on these eight islands. With `donor_max` 0.267 and `donor_rest` 6 as
  well, 32 per cent more (8/0) and 13 per cent more works (8/0), the 1880s wage
  -4 per cent (1/7). With `found_saving` 0.05, `donor_max` 0.3, `donor_rest` 5
  and `found_people` 2,250, 80 per cent more towns (starred), a third more
  works, 29,000 more people, and the wage -6 per cent (2/6); famine years 0.4
  to 1.25. So about half of what `found_saving` alone would allow is held back
  by the donors' limits, and more towns cost the late wage, as the people page
  found.
- Not run, and worth it: **`d_old_growth` with `growth_top`**, since town ground
  grows with people (`growth_top` 0.0213 gives 74 per cent more of it) and a
  change to one will want the other set again to keep the island's town ground
  at five times 1750's.

## What I would look at first in a pass over the rules

1. **Set `growth_top` and `y_still` again, both together**, for the island as
   it now stands. They are the largest numbers in either mechanism and were
   last set on six seeds before the mines and the canals moved 1750 and 1900.
   The pair above says how they trade. People decide the late wage here more
   than any other number in these two pages.
2. **Close R99 (the wage that dipped with kept ground) as not reproduced.** On
   sixteen islands today, switching kept ground off leaves the chained real
   wage of 1880-99 at 4.32 against 4.35 and output per head at 8.40 against
   8.38, the wage higher without on ten islands and lower on six. Over
   1840-59 the wage is 4 per cent higher without kept ground (11 islands of
   16), inside the spread between islands (z 0.3), and by the 1880s that is
   gone. Town ground still costs 3 per cent of the farm
   land on every island (1,447 cells' worth against 1,492). The mend the towns
   page left (let a town cell keep the share of its ground the houses do not
   need) would give that land back, but there is no wage to recover by it.
3. **Treat the founding gates as one rule.** Five numbers, of which one
   decides when loosened and three when tightened, and two never. If more
   towns are wanted for the map, `found_saving` with `donor_max` is the
   pair; the cost is a few per cent of the late wage.
4. **Leave alone**, or drop as settings: `growth_floor`, `found_cells`,
   `found_spacing`, `decline_years`, `back_span`. None decides anything at
   these sizes. The dearth's three numbers are idle on today's island and
   should be read again when anything makes the island hungrier.
5. **Measure against a cloud, not one baseline.** One island's single year
   moves by 7 per cent of its wage from a nudge that does nothing, and today's
   baseline on islands 1 to 8 happens to sit about one late town above its
   cloud. A pass that reads a whole scorecard of small changes should read each
   against nudged runs or against the other end of its own range, as here.

## Files

The sweeps are under `.claude/night/reports/opus/sens-people/`: `base` and
`base916` (today, islands 1 to 8 and 9 to 16), `s8/<setting>` (every setting,
islands 1 to 8), `s916/<setting>` (the sixteen run on 9 to 16), `pairs/` and
`edges/`. The readings, island for island and setting for setting, are
`movers8.txt` and `movers16.txt` (each value against today), `contrast8.txt`
and `contrast16.txt` (each setting's high value against its low), and
`movers-edges.txt` and `moverspairs.txt`; the commands and every decision are
in `build.md` beside them.
