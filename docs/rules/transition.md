# Fewer children as towns live better

Not in the Python. `crates/isostate-core/src/transition.rs`, switched by
`transition.enabled`: off under the port's rules, on under the game's. It is
the second stage of the run to 2050, and the first step of a fall in births.
**On an island that has stopped growing richer it slows the people and does
not level them off.** The rule answers to how well each town's townsfolk
live, so an island whose living standards sink after 1900, as this one's do
until content after 1900 is built, sees its births come back as it grows
poorer, and keeps growing: 2.7 million in 2050 where it had 3.9 without the
rule. Only an island that goes on growing richer levels off, and with the
stand-in for growth (below) it levels off far sooner and lower than was
reckoned.

## The standing check

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set transition.enabled=false,true --standing --out DIR
```

```text
standing check: 12 of 12 inside (0 moved, 0 OUTSIDE, 0 without a value): transition.enabled=true against transition.enabled=false

                                  transition.enabled=false                        transition.enabled=true
the twelve
  pop_1800                                         208,621  [194,046 .. 220,543]                  208,731  inside
  pop_1850                                         378,430  [346,723 .. 400,476]                  380,573  inside
  pop_1900                                         690,306  [621,604 .. 732,236]                  692,474  inside
  real_wage_chained_1850_vs_1750                      3.04  [2.41 .. 3.58]                           3.16  inside
  real_wage_chained_1900_vs_1750                      5.49  [4.65 .. 6.30]                           5.91  inside
  towns_1900                                            20  [17 .. 22]                               19.4  inside
  urban_share_1900                                   0.449  [0.437 .. 0.459]                        0.458  inside
  rail_km_1900                                       2,553  [2,312 .. 2,778]                        2,584  inside
  rail_first_year                                     1825  [1818 .. 1846]                           1827  inside
  income_share_work_1850                             0.660  [0.614 .. 0.721]                        0.662  inside
  income_share_work_1900                             0.660  [0.634 .. 0.676]                        0.669  inside
  income_share_land_1900                             0.163  [0.158 .. 0.170]                        0.164  inside
also
  last_town_founded                                   1888  [1877 .. 1897]                           1888  inside
```

All twelve are inside. The towns' share of people in 1900 is the one that
leans: it stands near the top of the spread at every setting tried (below),
because fewer births empty the country before the town ground.

## What it does

Under the game's people (`people.md`) a town's people grow each year by what
its income gives (`growth_top` times how far its baskets a head stand above
`y_still`), less what crowding on town ground costs, less dearth. That rule
has one resting point, where food runs short: an island that grows richer
only grows larger.

With the switch on, once a year and town by town, with `y` the town's
baskets a head:

    z     = the share of its people on town ground (at most 1)  *  y
    zbar  = zbar + (z - zbar) / lag_years            (a town's first year: zbar = z)
    F     = floor + (1 - floor) / (1 + (zbar / z_half) ^ z_power)
    S     = (y - clean_from) / clean_span, held between 0 and 1
    rate  = income term * F  -  crowding * (1 - S)  -  dearth

`z` is how well a town's townsfolk live: on a farm a child works, in a town
a child costs, and both go with income. `zbar` follows it over a
generation, so births answer to how people lived when they were raised. `F`
is the share of the income term kept: 1 on a poor island, a half where
`zbar` is `z_half`, and never less than `floor`. `S` is how far drains and
clean water have come: they come with income, and once complete a crowded
town costs no more lives than the country. Dearth and those who take ship
after a hungry year are as they were.

`zbar` goes where people go, as their cash does:

- with the year's migrants (`people::migration`): a town's `zbar` becomes
  the mean, weighted by people, of what it kept and what came in;
- with the districts a town takes from another when the markets' areas are
  worked out again (`peopling::move_with_cells`), in the same way;
- to a new town: its donor's at first (`peopling::open_town`), then the mean
  of every town that gave it people, weighted by the people each gave.

It lives inside the game's people rule, so it needs `people.enabled`; the
port's capped rule never reads it. With the switch off `peopling::growth`
takes its old line untouched, and a run is the run before it to the byte.

## Its numbers

`transition.enabled` (true in the game); `lag_years` 25, a generation;
`z_half` **4.5** in the game (4 by default); `z_power` 4; `floor` 0.04, so
that however well people live at least a twenty-fifth of the income term is
kept; `clean_from` 4 and `clean_span` 6 baskets a head.

At `z_half` 4 the standing check gave 11 of 12: the towns' share of people
in 1900 was 0.463 against the islands' 0.437 to 0.459 with the rule off.
`z_half` was raised a quarter at a time, the one number allowed to move:

| z_half | inside | towns' share, 1900 | people, 1900 |
|---|---|---|---|
| 4 | 11 | 0.463 OUTSIDE | 657,261 |
| 4.25 | 11 | 0.464 OUTSIDE | 674,987 |
| **4.5** | **12** | 0.458 | 692,474 |
| 4.75 | 11 | 0.460 OUTSIDE | 707,840 |
| 5 | 12 | 0.455 | 722,812 |

(rule off: 0.449 [0.437 .. 0.459]; 690,306 people.) 4.5 is the first step
with all twelve inside, and the game takes it. It passes by a thousandth:
the towns' share sits at the top edge of the spread whatever `z_half` is,
and does not fall steadily as it rises. Turning the drains off
(`clean_from` 1000) lifts it further (0.475), so it is the fall in births
that moves it, not the drains. A later stage that moves the towns' share
may push it out again.

The island of 1900 is richer than the design for this rule reckoned: `z`
stands near 4.9 in 1900 (4.3 to 5.6 across islands), so the rule has
already taken a third of the income term by then (`F` 0.63).

## The runs that show it

All under the game's rules with six families, on islands 1 to 8.

**1. Nothing moves with the switch off.** The port's golden test passes,
and the game's passes on its four hashes from before the rule with the
switch off in `game_params()`. A sweep of islands 1 to 4 to 1900 with `--set
transition.enabled=false`, set beside the same sweep made before this stage
(`--compare`): of the 739 facts the two share, the seconds aside, 0 differ.

**2. The standing check**, above: 12 of 12.

**3. The hungry island**: the same to 2050 (`--end-year 2050`). Without
content after 1900 the island grows poorer after 1950, `z` falls back,
births come back, and the people go on growing.

| islands 1 to 8 | rule off | rule on |
|---|---|---|
| people, 1900 | 690,306 [621,604 .. 732,236] | 692,474 [635,954 .. 744,110] |
| 1950 | 1,250,424 [1,119,376 .. 1,332,863] | 1,064,158 [958,191 .. 1,169,016] |
| 2000 | 2,241,376 [1,994,674 .. 2,405,999] | 1,644,552 [1,461,575 .. 1,852,547] |
| 2050 | 3,896,056 [3,453,735 .. 4,135,278] | 2,736,260 [2,406,322 .. 3,105,814] |
| the year of the most people | 2050 | 2050 |
| `z`, 1900; 1950; 2000; 2050 | | 4.87; 4.26; 3.10; 1.01 |
| `F`, 1900; 1950; 2000; 2050 | | 0.63; 0.54; 0.71; 0.97 |
| real wage, chained, 2050 (1750 = 1) | 2.62 | 3.32 |
| the towns' share, 2050 | 0.226 | 0.286 |

The design reckoned 3.6 to 3.8 million in 2050 with the rule on; the
island lands at 2.7 million, below it, because it is richer in 1900 than
was reckoned and the rule bites for half a century before poverty undoes
it. The standing check of this run is 12 of 12 too.

**4. With growth**: the same with `--set longrun.standin_growth=true`
(`longrun.md`, "A stand-in, to be deleted"), which lifts the two learning
caps from 1750 on.

| islands 1 to 8, the stand-in on | rule off | rule on |
|---|---|---|
| people, 1900 | 709,842 [651,558 .. 754,657] | 587,076 [542,480 .. 628,640] |
| 1950 | 1,288,171 | 617,540 [565,109 .. 670,994] |
| 2000 | 2,348,712 | 638,854 [584,279 .. 694,587] |
| 2050 | 4,234,004 [3,977,917 .. 4,459,799] | 660,373 [603,784 .. 718,259] |
| the year of the most people | 2050 | 2050 |
| `z`, 1900; 1950; 2000; 2050 | | 13.9; 23.3; 25.6; 25.0 |
| `F`, 1900 to 2050 | | 0.13; 0.044; 0.042; 0.041 |
| real wage, chained, 2050 (1750 = 1) | 6.21 | 19.00 |

The people level off, as they were meant to, but in the 1880s and at
about 0.6 million, where the design looked for a level between 1.0 and 1.5
million after 2000. The stand-in lifts the caps from 1750, so this island
is rich long before 1900: `z` is 14 in 1900, three times the island
without it, and `F` is at its floor by 1950. The people still grow a
little (the floor keeps a twenty-fifth of the income term), so their
largest number is in 2050. This is a finding, not a matter for tuning: the
level depends on when the island grows rich, and the stand-in makes it rich
too soon. Its standing check reads 11 of 12, people in 1900 moved, which is
the stand-in's doing, not the played game's.

## Facts

- `people_peak` and `people_peak_year`: the most people the island had in a
  year (the yearly mean), and the year; the last year if they never fell.
  Whatever the switch.
- `transition_F_Y` and `transition_z_Y`: the island's means of `F` and `z`,
  weighted by people, at the sweep's marks. Only with the rule on.

## What worries me

- It is one line where the design has seventeen ages to come
  (`people.ages`, a later stage), and it cannot be both inside the played
  century and level on an island that does not grow: no `z_half` does both.
- Births follow the townsfolk alone. A farming district under a rich town
  has its births cut as the town's are.
- The towns' share of 1900 is at the top edge of its spread.
