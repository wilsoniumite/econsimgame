# sens-prices: how far the merchants' prices and Engel's law answer their numbers

One of six tables of sensitivity for the pass over all the rules (piece 18 of
`docs/backlog/found-along-the-way.md`). It covers two mechanisms: the merchants who mark
prices with their stock (`docs/rules/prices.md`) and the Engel curve in what households
buy (`docs/rules/engel.md`). Each of their eight settings was moved to about three
quarters and four thirds of the game's value, or flipped, on eight islands, and the
ones that mattered were taken further and run on sixteen. No rule and no fact was
changed: this page is measurement only, on the game's rules as they stood at a6ff0ec
(12 November 2026), with six families.

The findings behind it are C3 (the markup not swept since carting, people and towns),
C49, R38 and C130 (the curve about doubles the famines), C50 (strength, `y_ref` and the
ceiling rest on seeds 1 to 8 of the port) and C54 and R39 (`y_ref` was set beside the
island's income of 1750, which has since moved).

## In brief

- **`engel.y_ref` is the setting that matters most**, and it is a trade. Each quarter of
  a bundle taken off it buys one to two points of town (most in mid-century) and costs
  about one per cent of the people of the 1880s and 1890s and some five per cent more
  swing in grain's price, with famine years rising in step. Sixteen islands agree. At 1.0
  the curve turns at subsistence and the island starves: 2.5 famine years a run where the
  game has 0.4.
- **The island's income of 1750 is now 1.51 to 1.60 bundles a head** (seeds 1 to 8), not the 1.84 to 1.89
  that `y_ref` = 2 was set beside. The engel page's own reasoning (set `y_ref` where the
  island starts) would now put it near 1.55. That would be a choice, not a repair; the
  numbers for it are below.
- **With the harvests, the markup's best is nearer 0.67 than 0.5, and its top is lower.**
  The farm zigzag is 2.0 on today's game, not the 0.8 it was when the markup landed, and
  it is least near a markup of 0.67 (1.69, down on 13 islands of 16). Below 0.5 it grows
  fast (2.98 at 0.375, up on all 16). From 0.8 the famines rise: 1.1 famine years a run at
  0.8 and 1.8 at 1.0 (sixteen islands), with alarms of downturn up a fifth. At 1.5 and 2 the
  island breaks, with 21 and 60 famine years a run. **Most of this is the weather.** With
  the harvests switched off (seeds 1 to 8) the zigzag is 1.12 at the game's markup, under
  the prices page's bar of 1.2; it is no smaller at 0.67 (1.12), and the famines do not
  rise at 0.8. What a stiffer merchant buys on today's island is farm hands that answer a
  short harvest less, and what it costs is famine. The top of the window has come down
  without the weather too: at 0.8 the weather-free zigzag is a quarter larger (7 islands
  of 8), where on the port 0.8 measured like 0.5.
- **`engel.strength` matters less than it did.** Incomes reach 8.9 bundles a head in the
  1880s and 1890s (the engel page measured 3.6), so the curve is all but fully turned by
  then whatever its strength. Between 2.25 and 4 the towns of the late century move by
  under a point.
- **The merchants' brake does nothing at these sizes.** `glut_months`, `glut_stop_months`
  and `glut_floor` at three quarters and four thirds (and a floor of 0.1 or 0.25) move
  nothing beyond what a null run moves.
- **Both switches still do what their pages say, and more.** With the curve off the towns of
  the 1890s hold 0.27 of the people, not 0.42: the curve now carries fourteen points of
  town where it carried six when it landed. With the merchants' prices off the farm zigzag
  is 6.0 and famine years 2.4 a run.
- **No run failed** the monthly money check or panicked: 304 runs to 1900.

## How it was measured

Every sweep is eight islands (seeds 1 to 8), six families, the game's rules, to 1900:

```bash
cargo build --release -p isostate-core
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --out game
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --set prices.markup=0.375,0.667 --out markup --compare game
```

and the same with `--seeds 9-16` for the sixteen-island results. The baseline sweep
gives every fact exactly as main's last landing did (`--compare` against
`merged-game-f6-l`: "of the 446 facts both have, the seconds aside, 0 differ").

Each setting's sweep is read against the baseline island for island, by a short script
(kept with the notes, not in the repository). It takes every fact in `facts.csv` and
every yearly series in `annual.csv` averaged over twenty years (1781-1800, 1831-1850,
1881-1900), about 730 quantities, and for each gives the change in the mean, the change
over the standard deviation across the baseline's islands (written z below), and on how
many islands it went up and on how many down.

**What counts as moving.** With 730 quantities and eight islands, chance alone puts 7 or
8 of 8 on one side about seven times in a hundred. So I ran a null control: the markup at
0.4999 and 0.5001, which nudges every island onto a different path and changes nothing
economic. It shows how jumpy each fact is by itself:

| | 7 or 8 of 8 agree | all 8 agree | 7 or 8 agree and z of 1 or more |
|---|---|---|---|
| null (0.4999; 0.5001) | 46; 27 | 5; 2 | 3; 1 |
| a setting that does nothing (`glut_months` 3; 5.33) | 47; 45 | 4; 4 | 11; 2 |
| a setting that matters (`y_ref` 1.5; 2.667) | 184; 123 | 62; 68 | 91; 62 |
| a switch flipped (prices off; curve off) | 214; 199 | 105; 125 | 144; 138 |

In the null, famine years swing from 0.375 a run to 0.25 and 0.625, `news_steep` by a
seventh, the number of towns by a twentieth, and the canal facts by up to two standard
deviations. Those facts need sixteen islands, or a z well past 2, before I believe them,
and anything under a z of 1 on eight islands is noise. Famine years are small counts (0.375
a run on the game's sixteen islands, six islands of sixteen with any), so a single setting
rarely moves them on its own tally; I read them across a row of settings instead.

`gdp_pc` and `gdp_real` are left out of every table: they are deflated by each run's own
prices of 1750, which the markup changes (`prices.md`, "Money buys more in 1750").

## The merchants' prices

The baseline (sixteen islands): `farm_zigzag` 2.03, `price_swing_grain` 0.113, iron 0.174,
rails 0.179, cloth 0.039; `output_swing_grain` 0.096; famine years 0.375 a run; 16.8
alarms of downturn a run.

| setting | game | tried | what moved most (change, z, islands up/down) | agree |
|---|---|---|---|---|
| `prices.enabled` | on | off (8) | `farm_zigzag` 2.08 to 6.03 (z +26, 8/0); grain output swing x2.5 (8/0); price swings: grain +26%, iron +31%, cloth x2.3 (all 8/0); famine years 0.4 to 2.4 (5/0); dearth notices 8 to 19 (8/0); emigrants x10 (8/0); works in 1900 +20% (8/0); real wage 1881-1900 -11% (0/8, z -2.8); towns 1881-1900 -0.014 (0/8); the steel-rolling mill makes a quarter of the rails in 1881-1900 (8/0) | 8 of 8 |
| `prices.markup` | 0.5 | 0.375 (16) | `farm_zigzag` +47% (16/0, z +3.4); grain output swing +25% (16/0, z +3.0); grain price swing -8% (1/15, z -0.9) | 16 of 16 |
| | | 0.667 (16) | `farm_zigzag` -17% (3/13, z -1.2); grain output swing -18% (0/16, z -2.2); nothing else | 13 to 16 of 16 |
| | | 0.8 (16) | zigzag -11% (4/12); grain output swing -21% (0/16); alarms +20% (13/2); famine years 1.1 (7/3, z +1.6) | 12 to 16 of 16 |
| | | 1.0 (16) | zigzag -3% (6/10); grain output swing -29% (0/16); cloth price swing +14% (14/2), iron +12% (13/3); alarms +22% (12/3); famine years 1.8 (6/2, z +3.0); emigrants x4 (12/4) | 12 to 16 of 16 |
| | | 1.5, 2 (8) | broken: famine years 21 and 60 a run, emigrants 6,100 and 23,000, people 1881-1900 -4% and -12% (0/8), zigzag 3.2 and 4.9, every price swing up by half to threefold (8/0) | 8 of 8 |
| `prices.glut_months` | 4 | 3, 5.33 (8) | nothing beyond the null; the largest, works standing 1881-1900 -5% at 3 (1/7, z -1.6) | chance |
| `prices.glut_stop_months` | 8 | 6, 10.67 (8) | nothing beyond the null (real wage 1881-1900 +3% at 6, 7/1, z +0.8) | chance |
| `prices.glut_floor` | 0 | 0.1, 0.25 (8) | at 0.25 only: works standing 1881-1900 -6% (1/7, z -2.0), median works 6% larger (7/1, z +2.2), works founded -7% (3/5) | chance |

The markup, step by step on sixteen islands (eight for 1.5 and 2):

| markup | zigzag | grain output swing | price swing grain / iron / cloth | famine years a run | alarms a run |
|---|---|---|---|---|---|
| 0.375 | 2.98 | 0.121 | 0.104 / 0.167 / 0.040 | 0.56 | 15.5 |
| **0.5** | **2.03** | **0.096** | **0.113 / 0.174 / 0.039** | **0.38** | **16.8** |
| 0.667 | 1.69 | 0.079 | 0.115 / 0.177 / 0.039 | 0.69 | 18.8 |
| 0.8 | 1.81 | 0.076 | 0.119 / 0.181 / 0.041 | 1.13 | 20.2 |
| 1.0 | 1.97 | 0.069 | 0.122 / 0.195 / 0.044 | 1.81 | 20.4 |
| 1.5 (8) | 3.16 | 0.095 | 0.188 / 0.250 / 0.076 | 21.2 | 23.1 |
| 2.0 (8) | 4.92 | 0.179 | 0.278 / 0.287 / 0.129 | 59.8 | 27.0 |

The prices page found the window at 0.3 to 0.8 on the port, with the zigzag at 0.76 and the
best at 0.5. On today's game it starts higher and ends sooner: the zigzag is three times
the size, least between 0.67 and 0.8, and the famines that a markup over 1 brought on the
port already begin at 0.8. C3 said to try a lower markup first if the zigzag came back; on
today's rules it is the other way about.

**What the harvests do to it** (a check of this page; seeds 1 to 8, six families, the
game's rules with `--set harvest.enabled=false --set prices.markup=0.375,0.5,0.667,0.8`,
32 runs, none failed; beside them the same eight islands with the harvests):

| markup | zigzag, no harvests | zigzag, harvests | grain output swing, no harvests / harvests | grain price swing, no harvests | famine years, no harvests / harvests |
|---|---|---|---|---|---|
| 0.375 | 1.21 (7 up, 1 down) | 2.87 | 0.024 / 0.115 | 0.046 (7/1) | 0.25 / 0.62 |
| **0.5** | **1.12** [0.77..1.56] | **2.08** | **0.022 / 0.096** | **0.039** | **0.25 / 0.38** |
| 0.667 | 1.12 (5/3) | 1.75 | 0.021 / 0.079 | 0.038 (5/3) | 0.50 / 0.62 |
| 0.8 | 1.42 (7/1) | 1.78 | 0.026 / 0.076 | 0.048 (8/0) | 0.38 / 1.25 |

The weather is about half of today's zigzag: the false cycle the markup was built against
is 1.12 at the game's markup, still under the prices page's bar of 1.2, which was set for an
island without weather (the backlog's findings 126 and C57 say the same). Without the
harvests 0.5 and 0.67 are as good as each other, so the gain at 0.67 is farm hands that
answer a short harvest less, not a false cycle removed: the grain output swing falls by a
sixth with the harvests and by a twentieth without. The famines that rise from 0.8 rise only
with the harvests. What does not need the weather is the top of the window: without it, 0.8
already swings harder than 0.5 (zigzag +27 per cent, grain's price swing +25 per cent, on
seven and eight islands of eight). That fits a longer lag between a price and the hands than
the port had (carting, people and towns each add a step); it is a reading, not measured.

## Engel's law

The baseline (sixteen islands): the towns hold 0.244, 0.321 and 0.421 of the people in
1781-1800, 1831-1850 and 1881-1900; land 0.468, 0.361 and 0.239 of the hands; real income
2.9, 4.3 and 8.9 bundles a head; real income in 1750 1.51 to 1.60 by island (seeds 1 to 8, `real_income_1750`).

| setting | game | tried | what moved most (change, z, islands up/down) | agree |
|---|---|---|---|---|
| `engel.enabled` | on | off (8) | towns 1881-1900 0.420 to 0.275 (z -17, 0/8); works' share of hands 0.49 to 0.30; land 0.24 to 0.36; grain output 1881-1900 +87%; people 1881-1900 +7% (8/0, z +3.6); real wage 1881-1900 -10% (0/8, z -2.7); grain price swing -26% (0/8, z -3.3); famine years 0.4 to 0 (0/3); dearth notices halved (0/7); railway of 1900 +11% (8/0) | 8 of 8 |
| `engel.y_ref` | 2.0 | 1.5 (16) | towns +0.029 / +0.042 / +0.015 (16/0, 16/0, 15/1; z +2.6, +2.3, +2.3); land 1881-1900 -0.014 (1/15); comfort's budget share 1781-1800 +43% (16/0); grain price swing +9% (14/2, z +1.1); people 1881-1900 -1.9% (3/13); dearth notices +22% (12/2) | 15 to 16 of 16 |
| | | 2.667 (16) | towns -0.017 / -0.032 / -0.022 (2/14, 2/14, 0/16; z -1.5, -1.7, -3.5); land 1881-1900 +0.019 (16/0, z +3.1); grain output 1881-1900 +16% (8/0 on seeds 1-8, z +5.0); grain price swing -10% (0/16, z -1.2); people 1881-1900 +2.5% (14/2); famine years 0.125 (2/6) | 14 to 16 of 16 |
| `engel.strength` | 3.0 | 2.25 (16) | comfort's budget share 1881-1900 -0.012 (0/16, z -2.3); grain output 1881-1900 +6% (16/0, z +1.9); works' share 1881-1900 -0.011 (1/15, z -1.4); towns 1881-1900 -0.008 (1/15, z -1.3); land +0.008 (15/1) | 15 to 16 of 16 |
| | | 4 (16) | comfort 1881-1900 +0.008 (15/1, z +1.5); grain output 1881-1900 -5% (1/15, z -1.6); towns +0.006 / +0.013 / +0.005 (12/4, 12/4, 13/3) | 12 to 15 of 16 |

`y_ref` step by step (sixteen islands but for 1.0 and 2.25, which are seeds 1 to 8; changes
against the game's):

| `y_ref` | towns 1781-1800 / 1831-50 / 1881-1900 | people 1881-1900 | grain price swing | famine years a run | dearth notices |
|---|---|---|---|---|---|
| 1.0 (8) | +0.072 / +0.071 / +0.028 | -5.6% (0/8) | +20% (8/0) | 2.5 (6/1) | +32% (7/0) |
| 1.25 | +0.047 / +0.058 / +0.022 | -3.7% (0/16) | +15% (16/0) | 0.81 (6/4) | +22% (13/2) |
| 1.5 | +0.029 / +0.042 / +0.015 | -1.9% (3/13) | +9% (14/2) | 0.69 (6/5) | +22% (12/2) |
| 1.75 | +0.010 / +0.021 / +0.007 | -1.1% (5/11) | +6% (12/4) | 0.69 (6/3) | +13% (10/4) |
| **2.0** | **0.244 / 0.321 / 0.421** | **648,000** | **0.113** | **0.38** | **8.4** |
| 2.25 (8) | -0.009 / -0.017 / -0.013 | +0.5% | -3% | 0.50 | -11% |
| 2.667 | -0.017 / -0.032 / -0.022 | +2.5% (14/2) | -10% (0/16) | 0.13 (2/6) | -16% (4/12) |

The real wage of 1881-1900 is within two per cent of the game's at every value, on both
indices. Each famine count on its own row is too small to tally, but read down the column
they rise as `y_ref` falls, as the engel page explains: households that give less of each
further shilling to bread stand nearer the floor in a bad year.

Why `y_ref` matters so much and strength so little: real income is 1.5 to 1.6 bundles in
1750, about 3 by 1800, 4.3 in mid-century and 8.9 in the 1890s. `y_ref` decides when the
curve begins to turn, which is the whole of the early century; strength decides how fast it
turns once begun, and by the 1880s it has turned whatever the strength. The curve's
ceiling (the richer set of shares in content, which this table does not touch) is what
holds the towns of the 1890s now.

## The settings that matter most

1. **`engel.y_ref`**: two to four points of town per half bundle, against people, grain's
   calm and famines. The largest lever of the eight after the switches.
2. **`prices.markup`**: the farm zigzag and grain's swings, and past 0.8 the famines.
   Asymmetric: 0.375 costs half as much zigzag again, 0.667 saves a sixth, 1.0 saves nothing
   and brings famines. With the harvests on, most of what it moves is how the farm hands
   answer the weather.
3. **`engel.strength`**: about a point of town, mostly in mid-century; grain output by
   five per cent.

## The settings that do nothing at these sizes

`prices.glut_months`, `prices.glut_stop_months` and `prices.glut_floor`, at three quarters,
four thirds, 0.1 and 0.25. Nothing they move stands clear of the null on eight islands. This
agrees with the prices page (the brake turns away a fraction of one per cent of the iron
works' capacity), and it means the brake's numbers are not worth a search: it is a guard
against a hole, not a tuning knob. The one hint, fewer and larger works with a floor of
0.25, is a single quantity on eight islands.

## Where the run breaks

- `prices.markup` of 1.5 and 2: 21 and 60 famine years a run, tens of thousands of
  emigrants, every price swinging harder than the port's. Famine years already 1.8 a run
  at 1.0. The island's people never fall by more than 1.1 per cent from one year's mean to
  the next on any of the eight islands at 2 (and not at all at 1.5), and no run failed the
  money check; it is a starving island, not a broken program. Whether a single town is
  emptied the sweeps cannot say: they keep no count of people town by town, and the count
  of towns tells nothing, since no rule ever removes a town.
- `engel.y_ref` of 1.0: 2.5 famine years a run, a twentieth fewer people. The engel page
  saw this on two islands with a steep curve; it holds at the game's strength.
- Nothing else tried breaks. Every one of the 304 runs passed the monthly money check.

## Pairs worth trying together

- **`prices.markup` 0.667 with `engel.y_ref` 1.5** (run, sixteen islands): the two add and
  do not cancel. Towns +0.025 / +0.042 / +0.014 (16/0, 16/0, 15/1) as with `y_ref` alone; the
  zigzag -13% (2/14) and grain output swing -16% (0/16) as with the markup alone; grain's
  price swing +12% (15/1), famine years 0.88 (6/5), dearth notices +27% (12/2), alarms +19%
  (13/2). A steadier merchant does not buy back what an earlier curve costs in grain.
- **`engel.y_ref` with the harvest's numbers** (not run): the curve's cost is a thinner margin
  over subsistence in a bad year, and the harvest decides how bad the years are. Any move
  of `y_ref` downward should be read beside `harvest` at its own settings.
- **`prices.markup` with the harvest's numbers** (run in part, by switching the harvests
  off; see "What the harvests do to it"). The zigzag's floor at about 0.67 and the famines
  from 0.8 are both the harvests at work; the markup and the size of a bad year want
  searching as one.
- **`prices.markup` with whatever adds lag to farm hands** (`carting.adjust`, the people
  rules' timing; not run). Without the weather the top of the window has come down from
  above 0.8 to under it, which a longer lag would explain.
- **`prices.glut_months` with `price_target_months` and `trade_cap`** (not run): the brake is
  set by those two, and only matters if either moves.

## What I would look at first in a pass over the rules

1. **Decide `y_ref` as a choice about towns against famines**, not as a calibration. The
   page's rule (set it where the island starts) now says about 1.55, which gives two and a
   half to four points more town before 1850, one and a half by the 1890s, two per cent fewer people
   and a tenth more swing in grain. Settle it with the harvest in view, on sixteen islands.
2. **Give the markup a test that tells weather from a false cycle** before moving it. The
   prices page asks for a zigzag under 1.2; today's game is at 2.0, but 1.12 of that is left
   with the harvests off, so the bar is still met where it was meant to be. Read with the
   harvests on, the zigzag mostly measures how hard the farm hands answer the weather. A
   markup of 0.6 to 0.7 steadies that answer by a sixth and costs famine years (0.69 a run
   on sixteen islands against 0.38, which is near the noise); whether that is wanted is a
   choice about the harvests, so settle it with them, on sixteen islands, with the shock
   test of `examples/prices_probe.rs`.
3. **Leave strength and the brake alone.** Strength's work is now mostly done by income; the
   brake's numbers do nothing measurable.
4. **Re-measure the curve's ceiling.** The engel page's "just under four in ten" was measured
   on two richer islands of the port alone, at 5.3 and about 7 bundles (0.37 and 0.38); today's
   game, at 8.9 bundles and with the other mechanisms, has the towns at 0.42 and still rising. The richer set of shares
   (content, not a setting) is now what decides how far they go.

## Not done

The richer set of shares (`shares_rich`) was not varied: it is content, not a `--set`
setting. The prices probe's shock and the scripted players were not run. The sweeps without
families were not run. Famine counts are too small for any single setting's tally; the rows
of settings carry them.
