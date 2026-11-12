# The scorecard's baseline, and the floor of chance under it

The pass over the rules will be judged by the scorecard
([scorecard.md](../rules/scorecard.md)), so it needs a baseline it can trust
and a floor: how far a sweep's numbers move when nothing but chance has
changed. This page gives three cards on sixteen islands with six families
(the game's rules on today's `main`, the port's rules, and the game's rules
with the deeper production tree), the floor measured from five histories of
`main` and five of the tree, which of the scorecard's aims are met, and what
the tree moves beyond the floor. Nothing in the simulation was changed; the
one new piece of code is an example that measures the floor
(`examples/chance_floor.rs`). Measured on 12 November 2026 at `aa9f158`
(`main`), on branch `night/o-scorecard-baseline`.

## In short

1. **Today's card stands as the scorecard page printed it.** Sixteen islands
   of `main` under the game's rules give the same card to the last digit:
   loss 151.4, the towns' share of 1881-1900 at 0.421 against an aim of a
   half, 94 per cent of the loss.
2. **Chance moves one island about as far as the islands differ from each
   other, and the mean of sixteen by a fifth to a half as far.** Five
   histories of `main` that differ only in their draws move one island's
   towns' share of 1881-1900 by 0.008 (median over islands), against 0.0066
   between islands; the mean of sixteen moves by at most 0.0032. That gap
   between the five means is the floor, and a change to the rules must move
   a mean further than it before it is read as the change's doing.
3. **Never compare losses scored on their own tolerances.** By chance alone
   `main`'s loss runs from 49 to 165, because one island's towns falling
   from 0.404 to 0.373 doubles the towns' tolerance. Holding the tolerances at
   one sweep's (`scorecard --spread-from`) the same five histories give 148 to
   158. The tree's loss is 23 on its own tolerances and 36 on `main`'s; the
   second is the one to read.
4. **Of the four aims in the loss, today's rules meet one clearly (coke iron
   passes half within ten years) and miss three by 3.6 floors or more**, the iron
   price's swing among them, which the card calls met (0.174 against 0.16,
   within one tolerance) though only two islands of sixteen meet it.
5. **The tree moves almost everything beyond the floor, and the right way
   for the aims**: the towns' share of 1881-1900 from 0.421 to 0.464 (16
   islands of 16), the iron price's swing from 0.174 to 0.150 (14 of 16; met
   in four of the tree's five histories), the wage of 1881-1900 from 2.68 to
   3.10 times 1750's, output per head from 2.72 to 3.29, the year output per
   head peaks from 1878 to 1896, the farm zigzag down a little, and the tools
   famine nearly gone. Against it: 44,000 fewer people in 1900 (15 of 16),
   work's share of income down from 0.696 to 0.669 (16 of 16), no steel rail
   at all, and the whole of the gain in the towns' share and more is there
   in the first years, not grown during the run.

## How it was run

Three sweeps of `main` at `aa9f158`, with its release build:

```bash
cargo build --release -p isostate-core
isostate-core sweep --seeds 1-16 --rules game --families 6 --out main-game --compare merged-game-f6-l --quiet --jobs 16
isostate-core sweep --seeds 1-16 --rules port --families 6 --out main-port --quiet --jobs 16
isostate-core sweep --seeds 1-16 --rules game --families 6 \
    --set harvest.island_sd=0.0499998,0.0499999,0.0500001,0.0500002 --out main-chance --quiet --jobs 16
isostate-core scorecard main-port --spread-from main-game
```

Against the sweep of the day the scorecard landed on (`merged-game-f6-l`,
seeds 1 to 8) one fact of 446 differs, `news_famine`, which that landing
moved on purpose; so `main-game` is the scorecard page's own sweep, sixteen
islands wide.

**The tree.** The deeper production tree ([tree.md](../rules/tree.md) on
its branch) is on `night/int-tree`, a day behind `main`. The two branches
that carry it forward, `night/o-finish-tree-ui` and its `-check`, contain the
tree but not today's `main` (they lack the scorecard itself, among some 150
commits), so I merged `main` into a scratch branch of my own:

```bash
git checkout -b night/o-scorecard-baseline-tree night/int-tree
git merge main          # params.rs and docs/rules/README.md conflict; both sides kept
```

That built, but every run failed: the books (`accounts.rs`, from `main`) read
each good's decay from the port's table of sixteen goods, and the tree has
more. Five small mends make it run, four of them taken word for word from
`night/o-finish-tree-ui-check`, which had met the same things: the books
take each good's decay as the economy does (`tree::decay`); the `save_state`
example carries the tree's two new fields of the observer; two tests in
`tests/accounts.rs` and `tests/mines.rs` try their case as that branch does.
The fifth is new: `tests/people.rs`'s test that a sailing is news of its own
kind, which landed on `main` after that branch last took it, finds nobody
taking ship with the tree (no island of sixteen has a sailing), and now
tries its island without the tree, as the mines' test does. Then every core
test passed, the golden test among them. The scratch branch is at `be02ef8`
and is not for merging; the tree's own integration will meet the fifth mend
when it next takes `main`.

```bash
isostate-core sweep --seeds 1-16 --rules game --families 6 --out tree-game --compare main-game --quiet --jobs 16
isostate-core sweep --seeds 1-16 --rules game --families 6 \
    --set harvest.island_sd=0.0499998,0.0499999,0.0500001,0.0500002 --out tree-chance --quiet --jobs 16
isostate-core scorecard tree-game --spread-from main-game
```

**The floor.** A history is a sweep of the same rules with
`harvest.island_sd` (0.05 under the game's rules) moved by two or four
millionths of itself either way, which changes nothing but the draws, as
[sens-mines.md](sens-mines.md) and [sens-machines.md](sens-machines.md)
made their floors. With the unmoved sweep that is five histories of each:

```bash
cargo run --release -p isostate-core --example chance_floor -- main-game main-chance --against tree-game --against main-port
cargo run --release -p isostate-core --example chance_floor -- tree-game tree-chance --against main-game
```

For every fact the example prints the mean of sixteen islands in each
history; the **floor**, the largest gap between two of those means; one
island's move (the gap between its highest and lowest value over the
histories, as a median over islands); and the spread between islands, which
is the scorecard's tolerance. With five histories the largest gap is about
2.3 standard deviations of a mean, a little more than two sweeps differ by
chance nine times in ten. It is a rough yardstick: the gap of five draws is
itself uncertain by about a third.

The 176 runs took 29 minutes, sixteen at a time, on a machine shared with
other work, which is why the times on the cards are long and wander.

## The three cards

Mean over sixteen islands [least .. greatest], and the islands that meet
the aim on their own. "Misses by" is in tolerances, first on the card's own
and then on `main`'s (`--spread-from main-game`).

| Aim | Game rules (main) | Port's rules | Game rules with the tree |
|---|---|---|---|
| `urban_share_1881_1900`, at least 0.5 | 0.421 [0.404 .. 0.434], 0/16 | 0.259 [0.250 .. 0.264], 0/16 | 0.464 [0.439 .. 0.484], 0/16 |
| misses by | 11.95 | 48.25; 36.53 | 2.90; 5.46 |
| beside it, `urban_share_1900` | 0.420 | 0.256 | 0.462 |
| `farm_zigzag`, at most 1.2 | 2.03 [1.56 .. 2.72], 0/16 | 4.84 [3.61 .. 5.75], 0/16 | 1.93 [1.56 .. 2.21], 0/16 |
| misses by | 2.88 | 6.37; 12.64 | 3.87; 2.52 |
| beside it, `farm_zigzag_beyond_weather` | 1.27 [0.91 .. 1.81], 6/16 under 1.2 | 4.84, 0/16 | 1.19 [0.98 .. 1.41], 10/16 |
| `price_swing_iron`, at most 0.16 | 0.174 [0.110 .. 0.198], 2/16 | 0.204 [0.156 .. 0.244], 1/16 | 0.150 [0.130 .. 0.193], 13/16 |
| misses by | 0.64 (met) | 1.84; 2.03 | 0 (met) |
| `years_to_half_coke_furnace`, at most 10 | 7.8 [6 .. 11], 15/16 | 7.8 [6 .. 11], 15/16 | 8.1 [6 .. 11], 15/16 |
| `cpu_seconds`, at most 15 (not in the loss) | 39.3 [31.3 .. 46.8] | 39.1 [37.5 .. 40.6] | 38.4 [33.9 .. 41.8] |
| **loss**, own tolerances | **151.4** | **2372.3** | **23.4** |
| **loss**, `main`'s tolerances | 151.4 | 1498.4 | 36.2 |

The port's farm zigzag has no weather in it to take out, so the two
zigzags are the same number. The port keeps no books, so it has no share of
income.

And the facts the card reads without an aim:

| Fact | Game rules (main) | Port's rules | With the tree |
|---|---|---|---|
| `pop_1900` | 725,684 | 684,367 | 682,043 |
| `real_wage_1751_1770_vs_1750` | 1.06 | 0.97 | 1.06 |
| `real_wage_1791_1810_vs_1750` | 1.60 | 1.33 | 1.66 |
| `real_wage_1831_1850_vs_1750` | 1.85 | 1.39 | 1.93 |
| `real_wage_1871_1890_vs_1750` | 2.72 | 1.40 | 2.93 |
| `real_wage_1881_1900_vs_1750` | 2.68 [2.51 .. 2.91] | 1.37 [1.29 .. 1.44] | 3.10 [2.71 .. 3.41] |
| `output_per_head_1881_1900_vs_1750` | 2.72 [2.57 .. 2.90] | 1.53 [1.45 .. 1.58] | 3.29 [3.02 .. 3.64] |
| `output_per_head_peak_year` | 1878 [1864 .. 1898] | 1859 [1833 .. 1880] | 1896 [1888 .. 1900] |
| `farm_share_1881_1900` | 0.239 | 0.429 | 0.249 |
| `industry_share_1881_1900` | 0.489 | 0.276 | 0.543 |
| `mine_share_1881_1900` | 0.272 | 0.295 | 0.208 |
| `income_share_work_1881_1900` | 0.696 | - | 0.669 |
| `share_coke_furnace_1800` | 0.999 | 0.996 | 0.955 |
| `share_textile_mill_1830` | 0.963 | 0.990 | 1.000 |
| `share_steel_rolling_1890` | 0.036 | 0 | 0.000 |
| `rail_first_year` | 1827 | 1828 | 1830 |
| `rail_km_1870` | 1,829 | 1,058 | 1,776 |
| `rail_last_growth_year` | 1900 | 1853 | 1900 |
| `price_swing_rails` | 0.179 | 0.189 | 0.161 |
| `price_tools_dearest_vs_median` | 4.90 | 2.02 | 2.23 |
| `price_tools_years_over_twice_median` | 15.3 | 1.1 | 3.3 |
| `real_wage_worst_year_median_town` | -0.161 | -0.157 | -0.138 |
| `famine_years` | 0.4 | 1.4 | 0.1 |
| `emigration_years` | 0.1 | 0 | 0 |

## The floor

From the five histories of `main` (and, last two columns, of the tree).
"One island" is the median over islands of how far one island moves across
the histories; "between" is the spread between islands in the first
history, the card's tolerance.

| Fact | Floor | One island | Between | Floor, tree | One island, tree |
|---|---|---|---|---|---|
| `urban_share_1881_1900` | 0.0032 | 0.0079 | 0.0066 | 0.0048 | 0.022 |
| `urban_share_1900` | 0.0025 | 0.011 | 0.0067 | 0.0063 | 0.025 |
| `farm_zigzag` | 0.077 | 0.29 | 0.29 | 0.021 | 0.32 |
| `farm_zigzag_beyond_weather` | 0.068 | 0.24 | 0.22 | 0.055 | 0.25 |
| `price_swing_iron` | 0.0038 | 0.016 | 0.022 | 0.011 | 0.029 |
| `years_to_half_coke_furnace` | 0.44 | 1 | 1.2 | 0.69 | 1 |
| `discovered_coke_furnace` | 0.5 | 0 | 6.5 | 1.4 | 0 |
| `cpu_seconds` | 6.6 | 9.7 | 4.3 | 6.1 | 9.4 |
| `pop_1900` | 2,100 | 18,000 | 24,600 | 8,600 | 17,200 |
| `real_wage_1751_1770_vs_1750` | 0.0009 | 0.006 | 0.030 | 0.0023 | 0.005 |
| `real_wage_1791_1810_vs_1750` | 0.015 | 0.044 | 0.11 | 0.040 | 0.059 |
| `real_wage_1831_1850_vs_1750` | 0.054 | 0.17 | 0.13 | 0.077 | 0.25 |
| `real_wage_1871_1890_vs_1750` | 0.040 | 0.19 | 0.15 | 0.079 | 0.31 |
| `real_wage_1881_1900_vs_1750` | 0.025 | 0.17 | 0.12 | 0.081 | 0.29 |
| `output_per_head_1881_1900_vs_1750` | 0.016 | 0.13 | 0.11 | 0.109 | 0.30 |
| `output_per_head_peak_year` | 3.2 | 18 | 10.4 | 2.3 | 6 |
| `farm_share_1881_1900` | 0.0024 | 0.0053 | 0.0062 | 0.0018 | 0.0089 |
| `industry_share_1881_1900` | 0.0040 | 0.010 | 0.0078 | 0.0056 | 0.027 |
| `mine_share_1881_1900` | 0.0039 | 0.012 | 0.0078 | 0.0044 | 0.025 |
| `income_share_work_1881_1900` | 0.0028 | 0.014 | 0.0051 | 0.0067 | 0.024 |
| `share_coke_furnace_1800` | 0.0073 | 0 | 0.0023 | 0.038 | 0 |
| `share_textile_mill_1830` | 0.0082 | 0 | 0.15 | 0 | 0 |
| `share_steel_rolling_1890` | 0.14 | 0.11 | 0.068 | 0.0001 | 0 |
| `rail_first_year` | 3.4 | 5.5 | 7.1 | 4.6 | 10.5 |
| `rail_km_1870` | 45 | 249 | 134 | 118 | 250 |
| `rail_last_growth_year` | 0.3 | 1 | 0.3 | 0.2 | 1 |
| `price_swing_rails` | 0.011 | 0.032 | 0.019 | 0.013 | 0.030 |
| `price_tools_dearest_vs_median` | 2.3 | 3.9 | 2.3 | 0.28 | 0.42 |
| `price_tools_years_over_twice_median` | 4.9 | 17 | 8.3 | 2.3 | 3.5 |
| `real_wage_worst_year_median_town` | 0.0093 | 0.061 | 0.036 | 0.010 | 0.049 |
| `famine_years` | 0.25 | 1 | 0.5 | 0.19 | 1 |
| `emigration_years` | 0.06 | 0 | 0.25 | 0.06 | 0 |

The twenty-year means through the run, floor on `main` (and under the tree):

| Span | Wage | Output per head | Towns' share | Work's share of income |
|---|---|---|---|---|
| 1751-1770 | 0.0009 (0.0023) | 0.0005 (0.0012) | 0.0002 (0.0004) | 0.0004 (0.0009) |
| 1771-1790 | 0.020 (0.013) | 0.015 (0.014) | 0.0017 (0.0014) | 0.0029 (0.0030) |
| 1791-1810 | 0.015 (0.040) | 0.015 (0.053) | 0.0024 (0.0053) | 0.0047 (0.0047) |
| 1811-1830 | 0.046 (0.015) | 0.053 (0.010) | 0.0009 (0.0021) | 0.0036 (0.0026) |
| 1831-1850 | 0.054 (0.077) | 0.047 (0.081) | 0.0067 (0.012) | 0.0053 (0.0042) |
| 1851-1870 | 0.074 (0.129) | 0.070 (0.128) | 0.0031 (0.013) | 0.0044 (0.0047) |
| 1871-1890 | 0.040 (0.079) | 0.032 (0.127) | 0.0028 (0.0055) | 0.0026 (0.0085) |
| 1881-1900 | 0.025 (0.081) | 0.016 (0.109) | 0.0032 (0.0048) | 0.0028 (0.0067) |

What it says:

- **One island moves by chance as far as the islands differ.** For most
  facts of the late century one island's move is 1.0 to 1.6 times the spread
  between islands on `main`, and 1.6 to 1.9 times under the tree. One island
  can tell nothing about a rule; the mean of sixteen can.
- **The mean of sixteen moves by much less than the card's tolerance**:
  about half of it for the towns' share, a fifth for the wage of 1881-1900, a
  sixth for the iron price's swing, a quarter for the zigzag. A change that moves a mean by
  one tolerance is far beyond the floor; one that moves it by a tenth of a
  tolerance is not.
- **The floor grows through the century, and is wider under the tree.** The
  wage and output per head are steady to a thousandth in the first twenty
  years and wander by 0.05 to 0.13 in the middle of the century, where the
  works' building waves and the machines' cap come at different times on
  different histories. The tree's late floors are three to seven times
  `main`'s for the wage and output per head, because its islands are still
  growing in 1900 (output per head peaks in 1896) and a history that reaches
  a method a few years later ends lower.
- **Some facts are all chance.** Steel rails' share of 1890 runs from 0.04 to
  0.18 across five histories of `main`, as the machines' table found: the
  steel rail mill is on the edge of taking hold. `famine_years` and
  `emigration_years` are a few events in sixteen islands and move by a
  quarter and a sixteenth of a famine a run. `cpu_seconds` moves by 6.6
  seconds with the machine's load and `seconds` by three minutes.
- **The loss scored on its own tolerances is no use for comparing sweeps.**
  On `main` it is 151, 96, 127, 165 and 49 in the five histories. In the last,
  island 3's towns' share of 1881-1900 fell from 0.404 to 0.373, the spread
  of the towns' share between islands doubled, and the towns' miss fell from
  12 tolerances to 6. Scored on `main`'s tolerances the five give 151, 152,
  148, 153 and 159: a floor of 10.5 on a loss of 151. The tree's are 23, 25,
  19, 29 and 20 on their own (floor 10.5) and 21.4 to 23.4 on the first
  history's (floor 1.9).

A rule of thumb for the pass over the rules: read a change as the change's
doing when the mean moves by more than the floors of both the rules it left
and the rules it made, and most islands move the same way (twelve of sixteen
or more). A change between one and one and a half floors with the islands
split is chance until more histories say otherwise. The surest test is the
one used for the tree below: five histories on each side, and the five means
of one not overlapping the five of the other. It costs five sweeps where one
would do, about half an hour on this machine for both sides.

## The aims: met, missed, or cannot be told

On `main` under the game's rules, against the floor of its five histories:

| Aim | Today | Verdict |
|---|---|---|
| Towns' share of 1881-1900 at least 0.5 | 0.421 | **missed** by 0.079, 25 floors; no island of sixteen meets it |
| Farm zigzag at most 1.2 | 2.03 | **missed** by 0.83, 11 floors. The fact beside it, the zigzag beyond the weather, is 1.275 against 1.2: a miss of 0.075 on a floor of 0.068, with 6 islands of 16 under 1.2; but all five histories are above it (1.26 to 1.33), so **missed, narrowly** |
| Iron price's swing at most 0.16 | 0.174 | **missed** by 0.014, 3.6 floors, on 14 islands of 16, and in all five histories (0.170 to 0.174). The card says "met" (within one tolerance between islands); the floor says otherwise |
| Coke iron past half within 10 years | 7.8 | **met** by 2.2 years, 5 floors; 15 of 16 islands |
| 150 years in under 15 seconds (not in the loss) | 39 s of processor | **missed**, but read on a crowded machine: the floor of `cpu_seconds` is 6.6 s from the load alone. The aim was set for the port, which also takes 39 s here |
| The railway goes on growing after the first rush (no number) | the last year it grew is 1900 on 16 of 16 | **met** as far as a fact reads it; the port's railway stops growing in 1853 |
| Iron and rails no longer cycling every few years (no number) | swings 0.174 and 0.179 | **cannot be told**: no number is set |
| Iron output's swing under 0.09 in every half-century; iron stock near two months with under 4 per cent of orders unfilled | no fact | **cannot be told** by the card |
| PLAN 1.3's table (population, the wage's path, coke iron by 1800, mill cloth by 1830, rail by 1870) | on the card | **cannot be told**: no number is stated |

With the tree, against the wider of the two floors and the tree's five
histories:

- **Towns' share**: 0.464 (0.463 to 0.468 in five histories), still
  **missed** by 0.036, 7.5 of the tree's floors; no island reaches a half,
  the largest 0.484.
- **Farm zigzag**: 1.93 (1.91 to 1.93), **missed**. Beyond the weather it is
  1.194 in the first history, a hair under 1.2 with 10 islands of 16 under
  it, but 1.19 to 1.25 over five and under 1.2 in only one: **on the edge,
  more likely just missed**.
- **Iron price's swing**: 0.150 in the first history, 0.150 to 0.162 over
  five and under 0.16 in four, on 13 islands of 16: **met, narrowly**; the
  margin, 0.010, is less than the tree's own floor of 0.011.
- **Coke iron**: 8.1 years (7.8 to 8.5), **met**, 15 of 16.

Under the port's rules every aim is missed but coke iron's, and by more than
any floor here; the floor was measured on the game's rules and the port's
own was not, so those verdicts lean on the size of the misses (the towns'
share 0.24 short, the zigzag four times its aim).

## What the tree moves beyond the floor

Against `main`, on the same sixteen islands, with each change beside both
floors (`main`'s, the tree's) and the islands up and down:

| Fact | Main | Tree | Change | Floors (main, tree) | Islands up/down |
|---|---|---|---|---|---|
| `urban_share_1881_1900` | 0.421 | 0.464 | +0.043 | 0.003, 0.005 | 16/0 |
| `urban_share_1751_1770` | 0.215 | 0.295 | +0.080 | 0.0002, 0.0004 | 16/0 |
| `price_swing_iron` | 0.174 | 0.150 | -0.023 | 0.004, 0.011 | 2/14 |
| `farm_zigzag` | 2.03 | 1.93 | -0.10 | 0.08, 0.02 | 7/9 |
| `farm_zigzag_beyond_weather` | 1.27 | 1.19 | -0.08 | 0.07, 0.06 | 6/10 |
| `real_wage_1881_1900_vs_1750` | 2.68 | 3.10 | +0.42 | 0.03, 0.08 | 15/1 |
| `real_wage_1811_1830_vs_1750` | 1.67 | 1.78 | +0.11 | 0.05, 0.02 | 16/0 |
| `output_per_head_1881_1900_vs_1750` | 2.72 | 3.29 | +0.56 | 0.02, 0.11 | 16/0 |
| `output_per_head_peak_year` | 1878 | 1896 | +17 | 3.2, 2.3 | 14/0 |
| `pop_1900` | 725,684 | 682,043 | -43,642 | 2,100, 8,600 | 1/15 |
| `industry_share_1881_1900` | 0.489 | 0.543 | +0.054 | 0.004, 0.006 | 16/0 |
| `mine_share_1881_1900` | 0.272 | 0.208 | -0.065 | 0.004, 0.004 | 0/16 |
| `farm_share_1881_1900` | 0.239 | 0.249 | +0.010 | 0.002, 0.002 | 16/0 |
| `income_share_work_1881_1900` | 0.696 | 0.669 | -0.027 | 0.003, 0.007 | 0/16 |
| `income_share_work_1751_1770` | 0.694 | 0.701 | +0.007 | 0.0004, 0.0009 | 15/1 |
| `price_tools_years_over_twice_median` | 15.3 | 3.3 | -12.0 | 4.9, 2.3 | 0/16 |
| `price_tools_dearest_vs_median` | 4.90 | 2.23 | -2.7 | 2.3, 0.3 | 0/16 |
| `price_swing_rails` | 0.179 | 0.161 | -0.018 | 0.011, 0.013 | 2/14 |
| `real_wage_worst_year_median_town` | -0.161 | -0.138 | +0.023 | 0.009, 0.010 | 11/5 |
| `discovered_coke_furnace` | 1774.5 | 1777.2 | +2.8 | 0.5, 1.4 | 9/6 |
| `share_coke_furnace_1800` | 0.999 | 0.955 | -0.044 | 0.007, 0.038 | 1/3 |

Inside one floor or both on the first histories: years for coke iron to
pass half (+0.25), the first railway's year (+2.8), the railway's length in
1870 (-53 km, inside the tree's floor of 118), the wage of 1771-1790 and
1851-1870, output per head of 1771-1790 and 1851-1870, work's share of
1831-1850, steel rails' share (from 0.036 to nothing, inside `main`'s own
floor of 0.14), famine and emigration years, and the times.

Five histories on each side settle some of these. Each line is the mean of
the five histories' means, with the least and greatest of the five (the
columns `h1` to `h5` the example prints for each):

| Fact | Main, five histories | Tree, five histories | Do they overlap? |
|---|---|---|---|
| `urban_share_1881_1900` | 0.421 [0.419 .. 0.423] | 0.465 [0.463 .. 0.468] | no |
| `price_swing_iron` | 0.173 [0.170 .. 0.174] | 0.155 [0.150 .. 0.162] | no |
| `farm_zigzag` | 2.05 [2.03 .. 2.10] | 1.92 [1.91 .. 1.93] | no |
| `farm_zigzag_beyond_weather` | 1.29 [1.26 .. 1.33] | 1.22 [1.19 .. 1.25] | no |
| `years_to_half_coke_furnace` | 7.50 [7.38 .. 7.81] | 8.20 [7.81 .. 8.50] | they touch |
| `real_wage_1881_1900_vs_1750` | 2.69 [2.68 .. 2.71] | 3.15 [3.10 .. 3.18] | no |
| `output_per_head_1881_1900_vs_1750` | 2.73 [2.72 .. 2.74] | 3.36 [3.29 .. 3.40] | no |
| `output_per_head_peak_year` | 1879 [1878 .. 1881] | 1895 [1894 .. 1896] | no |
| `pop_1900` | 726,584 [725,684 .. 727,767] | 679,700 [674,435 .. 683,020] | no |
| `income_share_work_1881_1900` | 0.698 [0.696 .. 0.699] | 0.665 [0.662 .. 0.669] | no |
| `rail_km_1870` | 1,814 [1,784 .. 1,829] | 1,759 [1,718 .. 1,837] | yes |
| `share_steel_rolling_1890` | 0.100 [0.036 .. 0.179] | 0.0001 [0 .. 0.0001] | no |
| `famine_years` | 0.25 [0.13 .. 0.38] | 0.20 [0.13 .. 0.31] | yes |
| `price_tools_years_over_twice_median` | 16.7 [14.4 .. 19.3] | 3.2 [2.3 .. 4.6] | no |

So the first history of `main` was a low one for steel rails: over five, a
tenth of `main`'s rails are steel in 1890, and under the tree none are, in
every history. The railway's length and the famines do not move.

Reading it:

- **The tree's gain in the towns' share is mostly a higher start.** In 1750
  the towns hold 0.293 of the people under the tree against 0.217 on `main`,
  because the trades it seeds in 1750 are town trades. Over the run the
  towns' share rises by 0.171 under the tree and 0.204 on `main`. PLAN.md's
  test is a rise "from about 0.25 to 0.5"; the tree starts nearer the half
  and climbs less. A pass that judges the towns on the twenty-year mean of
  1881-1900 alone will credit the tree with what it did in 1750.
- **The wage and output per head gain late, and keep growing.** The gain is
  small before 1830, a tenth at 1811-1830, inside the floor in 1851-1870,
  and large only after 1870: output per head peaks in 1896 against 1878, on
  every island that moved. The tree pushes back the late-century stall, but
  its late floors are wide, so its exact late numbers are the least certain
  thing on this page.
- **Fewer people, a smaller share for work.** 44,000 fewer people in 1900,
  lower on 15 islands of 16, and work's share of income 2.7 points lower on
  every island from 1870 on. With more stages of plant between the raw
  material and the household, more of the income is the owners'.
- **The tools famine is nearly gone.** The years in which tools cost over
  twice their middling price fall from 15 to 3, on every island.
- **The farm zigzag falls a little, and on the whole island rather than
  island by island.** It falls by 0.10 to 0.13, beyond both floors, and the
  five histories of each do not overlap, yet it rises on 7 islands of 16 in
  the first pair: a small shift of the mean under a wide scatter, which only
  the five against five could show.
- **No steel rail.** Under the tree the steel rolling mill makes nothing to
  speak of by 1890 in any history, where `main` lays from a twenty-eighth to
  nearly a fifth of its rails in steel. The machines' table found the steel rail
  mill on the edge of taking hold; the tree tips it off.

## What worries me

- **The floor is from five histories.** The gap of five draws is a rough
  measure of their spread, uncertain by about a third, and some facts (the
  zigzag under the tree, 0.021 against `main`'s 0.077) show it. Ten
  histories would halve that; they were not run.
- **One way of making chance.** Every history here moves `island_sd`, so its
  draws differ from the first harvest on. That is the way the earlier tables
  made their floors and it changes every draw that follows, but it is one
  setting; a floor from moving some other setting by millionths should be the
  same size, and was not measured.
- **The port has no floor of its own here.** Its verdicts above rest on
  misses far larger than any floor of the game's rules.
- **The tree's numbers are from a scratch merge**, not from a branch that
  will land. Its integration may settle things differently (the fifth mend
  above, or whatever else changes before it lands), and its card should be
  run again then. The tree also changes the 1750 island, so every comparison
  with it has the start in it as well as the run.
- **The times on the cards are the machine's.** The 176 runs shared the
  machine with other work, so `seconds` here is three to eight times what a
  quiet machine gives, and even `cpu_seconds` wanders by 6 seconds.

## Files

The sweeps, the cards and the example's output are kept outside the
repository with the night's reports, under `scorecard-baseline/`:
`main-game`, `main-port`, `main-chance`, `tree-game`, `tree-chance`, their
`scorecard.txt`, the two cards scored on `main`'s tolerances, and
`chance-floor-main.txt`, `chance-floor-tree.txt`,
`chance-floor-tree-against-main.txt`, `tree-vs-main-both-floors.txt` and
`five-history-means.txt`.
