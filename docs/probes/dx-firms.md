# dx-firms: why thousands of tiny works, and a life cycle that fixes it


## Answer in brief

1. **Works never finish building, so they cannot grow.** An order for capacity is delivered a quarter of the remainder each month (`econ.rs:562`) and counts as done only when under 1e-6 units remain (`econ.rs:737`), which takes 5 to 7 years. `firms::review` treats any open order as "building" (`firms.rs:362`), and a building works may not grow (`:385`), is never counted idle (`:370`) and is never "tiny" (`:376`). In 1900, 86% of standing works carry such an order (3,704 of 4,307; only 440 have more than 5% of their size still to come). A works can grow about once in six years, by 20 to 50%, and loses 22% to depreciation meanwhile. Growth in demand is met by new small works instead.
2. **The size limit is tiny next to the market.** A works stops at 4 K0 (`firms.rs:387`), and K0 is a few hands in several trades (crucible steel 16, smithy 5, engine works 18).
3. **Families found a 0.4 K0 works a year per branch** (`families.rs:1984`): 3,495 of the 6,813 works in 1900 are theirs.
4. **Industry covers a sixth of the land** because a works of size K0 takes a quarter of a 2 km cell whatever the trade (`firms.rs:210`, `sim.rs:244`): a five-man smithy takes a square kilometre.

Two leads did not hold. Shops and trades are 6% of works, not the bulk. There are almost no zombies (12 to 20 works a run with utilisation under 0.3 for five years; utilisation is 0.8 to 0.97), so closing the idle sooner does nothing useful.

The recommended life cycle (six switches, prototyped) gives about **575 works in 1900 instead of 4,300 (6,800 with families) and 100 industry cells instead of 850 to 970**. Output per head in 1900 is 6 to 10% higher, the real wage over 1850 to 1900 is 15 to 24% higher, and price volatility is 12 to 18% higher.

## What I ran

`crates/isostate-core/examples/probe_firms.rs` steps a run and writes the demography, a macro row per year, every death and where each method stands. `probe_run.sh FILE` runs the variants in a file (`sweep1.txt` to `sweep10.txt`: label, families, JSON overrides), one thread per seed; the `probe_*.py` scripts print the tables. Raw output is in `probe_out/` (197 MB, not committed).

The hacks are `hx_*` fields in `params.rs`, all off by default. With all off, the yearly macro rows for seeds 7 and 3 are byte-identical before and after the hacks and agree with the baseline records (seed 7: 685,124 people and 4,596 firms in December 1900; 6,873 with families). Money was conserved in every run.

Seeds: 3, 5, 7, 11, 13, 17, 21, 29 for the baseline and the final prototype; 3, 7, 11, 21 for the sweeps. "f0" is no families, "f6" is six. "Volatility" is the standard deviation of yearly log changes in the national price, averaged over iron, tools, cloth, bricks, machinery, services and coal.

## 1. The demography of works (unmodified code, 8 seeds)

| | 1750 | 1800 | 1850 | 1900 (range) | per 1000 people, 1900 | industry cells 1750/1900 | urban cells 1900 |
|---|---|---|---|---|---|---|---|
| f0 | 572 | 1,506 | 2,640 | 4,307 (3,967-4,657) | 6.3 | 206/847 | 74 |
| f6 | 572 | 2,074 | 4,176 | 6,813 (6,633-7,093) | 10.0 | 206/966 | 77 |

Works per head are constant from 1775: works multiply, they do not grow.

1900 by method, f0 (f6 in brackets where it differs much):

| method | works | median size, K/K0 | p90 | utilisation | median age | largest works' share of town capacity |
|---|---|---|---|---|---|---|
| crucible steel | 1,212 (1,447) | 1.65 | 3.4 | 0.86 | 58 | 0.04 |
| brickworks | 832 (1,319) | 0.99 (0.76) | 3.1 | 0.90 | 50 | 0.09 |
| powered mill | 630 (1,893) | 0.72 (0.34) | 1.3 (0.56) | 0.97 | 82 (45) | 0.07 |
| engine works | 537 (743) | 0.79 (0.57) | 1.3 | 0.89 | 37 | 0.08 |
| coke furnace | 528 (783) | 0.40 (0.33) | 0.8 | 0.93 | 65 | 0.09 |
| shops and trades | 253 (272) | 0.92 | 3.0 | 0.85 | 17 | 0.23 |
| smithy | 209 (228) | 1.11 | 3.1 | 0.83 | 28 | 0.25 |

Size classes in 1900, trades other than shops (f0 on 4 seeds; f6 in brackets): works under 0.5 K0 are 23% (54%) of works and make 12% (38%) of output; works of 2 to 4 K0 are 20% (13%) of works with 48% (44%) of capacity.

Flows per decade, trades other than shops: f0 has 200 to 460 new works and 50 to 400 sister works in, and 150 to 400 out; f6 adds 200 to 480 family foundings from the 1770s. Over a run f0 founds 8,133 works and loses 4,377. The median life of the dead is 11 years (12.6 in f6). Cause of death (f0): **insolvency 61%, three loss years 39%, tiny or stale almost never**. The loss rule rarely fires because of the building flag. The standing works are old (median age 50): a long-lived mass of small works plus a churning fringe.

Why the small do not grow: in 1900, 98 to 100% of powered mills earned more than the hurdle with utilisation above 0.65 (median return 36%), yet their median size is 0.72 K0 at a median age of 82 years. That is the building flag.

Entry also promises more than a works gets. `evaluate_sites` (`firms.rs:112`) leaves out the merchant's spread, 6% of the price, which `econ.rs:660` takes from revenue. Capacity is cheap next to turnover, so 6% of the price is tens of points of yearly return.

## 2. Simple parameter changes (4 seeds, f0/f6)

| change | works | output per head | real wage | volatility | industry cells | note |
|---|---|---|---|---|---|---|
| baseline | 4,412/6,832 | 6.93/6.61 | 1.42/1.39 | 0.137/0.132 | 887/979 | |
| `entry_K_frac` 1 | 3,725/5,014 | 7.42/7.35 | 1.32/1.29 | 0.135/0.133 | 1,024/1,162 | |
| `entry_K_frac` 2 | 3,099/3,767 | 7.84/7.61 | 1.20/1.16 | 0.124/0.129 | 1,068/1,274 | wage falls |
| entry hurdle x2 | 2,914/6,127 | 6.13/6.57 | 1.18/1.32 | 0.121/0.131 | 744/942 | cloth output halves (f0) |
| entry hurdle x5 | 2,568/5,838 | 5.16/6.25 | 1.16/1.29 | 0.127/0.125 | 743/912 | new methods starve |
| loss year at u < 0.3, close after one | 4,346/6,580 | 5.53/5.61 | 1.76/1.82 | 0.138/0.140 | 876/942 | output down a fifth |
| `footprint_K_mult` 16 | 3,695/6,104 | 7.14/6.81 | 1.44/1.41 | 0.134/0.134 | 265/288 | 400 works above 4 K0 |
| `footprint_K_mult` 64 | 3,332/5,818 | 6.57/6.76 | 1.37/1.43 | 0.128/0.133 | 87/93 | |
| land only: K0 takes 2.5% of a cell | 6,111/8,592 | 6.93/6.96 | 1.44/1.45 | 0.135/0.133 | 183/199 | cells never fill, more entry |
| land by hands (2,000 a cell) | 6,382/8,805 | 7.15/6.72 | 1.45/1.42 | 0.125/0.147 | 104/119 | |
| building flag fixed alone (8 seeds) | 4,057/4,325 | 6.88/7.43 | 1.20/1.31 | 0.166/0.166 | 1,310/1,256 | 16,219 founded (f0), median life 6.4 |

Closing after three years at utilisation under 0.3 changes nothing (4,572/6,929). No single parameter gets under 2,500 works. A higher entry hurdle hurts the economy. Splitting the two jobs of `footprint_K_mult` (the two "land" rows keep the 4 K0 limit) fixes the land but raises the count. Fixing the building flag alone sends growth into the 4 K0 limit and doubles the churn, so it must come with a larger size limit.

## 3. The recommended life cycle

Each is a parameter, off by default and on in the game's rules. Hack names are in brackets; values are the prototype's.

**A. A works has finished building when under 5% of its capacity is still on order** [`hx_build_done` 0.05]. In `firms::review`, the `building` line (`firms.rs:362`) becomes `f_order_K > share * (K + f_order_K)`. A share of 0.2 gives the same result. Efficient works then grow each year, and three loss years become the main cause of death.

**B. Land by hands, and the cell as the size limit** [`hx_land_mode` 2, `hx_workers_per_cell` 2000, `hx_max_K_mult` -1]. A works takes `K * m_labor[m] / hands_per_cell` of its cell (new `firms::land_foot`, used in `evaluate_sites` `firms.rs:113`, `create_firm` `:210`, `close_firm` `:497` and `sim::after_transport` `sim.rs:244`). It grows in place until it would take 0.8 of a cell (new `firms::max_K`, replacing `footprint_K_mult * K0` at `firms.rs:387`). Beyond that, `spawn_sister` opens a sister the size of the growth step (not K0, `:438`) on a neighbouring cell with room. Entry and sisters skip cells where the works would not fit. With 2,000 hands a cell there are 100 to 120 industry cells in 1900 and about 600 works; with 1,000, 250 cells and 1,030 to 1,090 works (310 to 330 of them shops); with 4,000, 55 cells and 513 works.

**C. Entry counts the merchant's spread** [`hx_entry_spread`]. In `evaluate_sites`, `pout` is less `merchant_spread * pbar` (`firms.rs:108`). Without it the prototype has 28% more works and 52% more foundings for the same output. Families' site sums use the same function, so the player's "expected return" becomes honest too.

**D. A year's room for new capacity, incumbents first** [`hx_gap` 0.15, `hx_gap_split` 0.34, `hx_entry_share` 0.10]. Each method in each town may add at most 15% of the town's monthly trade in the good (`st.outflow`: sales plus exports) in a year, and never less than one works of `entry_K_frac` K0, so a new trade can always start. `review` hands the room out in order of realised return, at most a third to one works. `entry` takes what is left and builds one works of up to 10% of the town's trade, if a minimum works fits (the doubling at a return above 0.6 and the second works a year are dropped). New `firms::year_room` and `firms::entry_K`; new state, a `(T, M)` matrix of capacity ordered this year. This is entry sized to the gap, and it damps the cobweb. It is the knob between output and steadiness. With A, B, C and sized entry (4 seeds, f0): a room of 15% gives output per head 6.82, volatility 0.152 and a median life of 13 years; 30% gives 7.67, 0.164 and 9; no limit gives 8.74, 0.173 and 7. The split of a third brings the largest works' share of town capacity from 0.76 to 0.47. Sized entry is not needed for the count (630 works without it); it makes a new works cost many times more, which may suit a family's works (not measured).

**E. A growing family branch extends its works instead of founding another** [`hx_family_extend`]. In `families::grow` (`families.rs:1980`), if the family has a works of that method in the town with room, the capacity is ordered there and paid from family cash. Family works in 1900: 3,495 in the baseline, 474 with A to D, 164 to 193 with E. `prospect_works`, `evaluate`, `site_options`, `found_at` and `grow` use `firms::entry_K` in place of `entry_K_frac * K0`.

**F. Hands walk to work** [`hx_commute` 0.02]. Labor per unit of output is multiplied by `1 + 0.02 * dcatch[cell]` in `evaluate_sites` and after `firm_coefficients` in `econ::step_month` (`econ.rs:374`). See the map section.

Tried and not recommended: amalgamating small works into a larger neighbour (`hx_merge`: 606 works against 610, nothing is left to merge once growth works); stopping economies of scale at 4 K0 (`hx_scale_cap`: no effect on output); sharper sampling of candidate sites (`hx_cand_decay` 3: no effect).

Optional: keeping craft trades (shops, smithies, handlooms, bloomeries, millwrights, charcoal burners, rail forges) at 4 K0 (`hx_craft_max`). It gives 989 works with 323 shops, but 7,084 foundings a run, because craft works at the limit throw off sisters that die.

## 4. Before and after (8 seeds, means; range over seeds in brackets)

The prototype is A to F (`sweep10.txt`, label `protoC2`).

| 1900 unless said | baseline f0 | prototype f0 | baseline f6 | prototype f6 |
|---|---|---|---|---|
| works | 4,307 (3,967-4,657) | 571 (518-661) | 6,813 (6,633-7,093) | 576 (488-653) |
| works around 1800/1850 | 1,537/2,672 | 295/476 | 2,144/4,242 | 308/492 |
| of which shops | 253 | 67 | 272 | 88 |
| family works | | | 3,495 | 164 |
| median size, K/K0 (p90) | 0.86 (2.8) | 7.7 (46) | 0.46 (2.5) | 7.7 (52) |
| largest works' share of town capacity | 0.31 | 0.45 | 0.21 | 0.47 |
| works founded/closed over the run | 8,133/4,377 | 1,582/1,290 | 11,705/5,403 | 1,871/1,580 |
| median age of standing works | 50 | 65 | 52 | 64 |
| median life of the dead | 11.2 | 7.1 | 12.6 | 7.8 |
| zombies | 20 | 3 | 12 | 4 |
| output per head, 1900 | 6.47 | 6.88 | 6.27 | 6.72 |
| output per head, five-year means at 1850/1900 | 6.33/6.41 | 7.15/6.78 | 6.49/6.17 | 7.29/6.78 |
| real wage, 1850 to 1900 mean (lowest year) | 1.32 (0.91) | 1.64 (0.94) | 1.42 (0.93) | 1.63 (0.92) |
| volatility (iron alone) | 0.139 (0.206) | 0.155 (0.221) | 0.142 (0.209) | 0.167 (0.236) |
| population | 681,024 | 678,799 | 684,701 | 679,015 |
| industry cells around 1800/1850/1900 | 335/517/847 | 10/52/97 | 371/607/966 | 16/55/100 |
| urban cells | 74 | 73 | 77 | 74 |
| cells with any works | 948 | 188 | 1,079 | 199 |

In 1900 the prototype (f6, about 17 towns) has 147 crucible steel works (median 25 K0, p90 80), 84 mills, 78 brickworks, 68 engine works, 63 coke furnaces, 88 shops and 28 smithies: a few large works and a tail in each town and trade. 71% of works are above 4 K0 and hold 98% of capacity. The deaths are the small fringe (median peak size of the dead is 0.4 to 4 K0 by trade); the large works last, though the median life of those that die is shorter than before.

The scripted players (`examples/bots.rs` with the rules in an `HX` environment variable, `bots_run.sh`; seeds 3, 7, 11, 21) end in the same range. Net worth in 1900, baseline against prototype: passive 16.3 to 17.8m against 15.1 to 21.3m; landlord 22.9 to 31.1m against 23.9 to 29.9m; thoughtful 20.7 to 29.0m against 21.6 to 27.1m. Ranks in 1900 move by up to three places either way (the founder goes 2nd to 4th on seed 3 and 4th to 1st on seed 7). The richest rival on seed 21 goes from 27-29m to 32-44m. The player founds 35 to 57 works with 19 to 27 standing in 1900 (passive), or 125 to 164 with 32 to 48 standing (founder, thoughtful); the baseline has 740 to 1,050 founded and 560 to 740 standing.

## 5. The map

Unmodified, 95% of hands work in industry cells, and works stand on average 6.5 cells (13 km) from their market: mills are 12.5 cells out, with 3% of their hands within two cells of the market. Nothing ties a works to where its hands live, so light trades drift to wherever inputs are a shade cheaper.

With F at 0.02 (prototype, f0), steel, mills, engine works, shops and smithies stand 82 to 99% within two cells of the market (mean 1.2 to 1.8 cells); coke furnaces stay out (75% on coal or ore, 5 cells out); brickworks stay on the clay (8 cells out). 13% of hands are in town cells and 81% in industry cells. At 0.05 everything moves into town (93% within two cells) and industry cells fall to 75; at 0.01, 68% are within two cells. Output and wages do not move with F.

Land: by hands, at 2,000 a cell. A cell counts as industry at a quarter taken, as now (`land.rs:325`), which is then 500 hands. With the prototype (f6) that is 2 cells in 1750, 16 around 1800 and 100 in 1900, beside 74 urban cells. Raise the density or the threshold if that is still too much.

## Confidence

High on the diagnosis (direct counts, 8 seeds, tight ranges) and on counts, sizes and land under the prototype (the ranges do not overlap the baseline's). Medium on the macro side: output per head in single years varies from 5.3 to 8.4 across seeds in both, and volatility is up in every comparison I made.

## Risks and what I did not check

- Output per head peaks around 1860 to 1870 (about 7.5) and sags to 6.8 by 1900, where the baseline is flat at 6.2 to 6.9 from 1810. I did not look for the cause.
- One or two towns fewer by 1900 (17 to 18 against 19 to 19.5, counted as towns with works).
- Shops fall to 67 to 88 large "works". If shops everywhere matter, use 1,000 hands a cell, or draw shops from service capacity per cell.
- Family balance was checked only with the scripted players, once per seed and strategy (above); AI families' own choices were not examined.
- The viewer was not built or run. Works reach 90 K0 (tiers go on doubling), so plots and tier art need a look.
- `cargo test -p isostate-core --release` passes on the branch with every switch off; nothing tests the switches. The Python reference was not touched.
- Utilisation also ignores the spread (`econ.rs:420`); not touched.
- The warm-up runs under the same rules, so 1750 starts with about 280 works instead of 572.

## How to tell it worked

Run `probe_firms OUT new 6 3,7,11,21 '{...}'`, then `python probe_firms.py compare 6 base new`. Expect 500 to 700 works in 1900, a median size of 6 to 9 K0, about 100 industry cells, a real wage near 1.6 over 1850 to 1900, volatility under 0.17 and under 2,200 works founded. With every switch off, `series_monthly.csv` must match `.claude/night/baseline`.
