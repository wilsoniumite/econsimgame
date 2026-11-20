# The core of the island

Written by `isostate-core ablate`, played on 2026-11-20 (UTC) from the code at `253d3c7`. Nothing on this page is written by hand: to make it again, run

```text
isostate-core ablate --out DIR --page docs/core.md
```

This one was made by `isostate-core ablate --out DIR --page docs/core.md --quiet`.

Every rule the game switches on is turned off from 1750, one at a time, and the island is played again: islands 1, 2, 3, 4, 5, 6, 7, 8 under the game's rules with 6 families, 1750 to 2050. Each such arm is set against the control, the islands as they are, island by island. 792 runs were played, 0 failed.

Of the 45 rules asked, 9 hold up the century, 23 shape the island, 13 are idle, 0 are leaned on.

Read the small counts with care. With 8 islands a side, a fact the rule does not move, only stirs (a rule that changes anything sends each island down another path), stands apart by chance once in 6,435 facts; a rule that stirs the island stirs some 3,100 facts, so 0.48 of them would stand apart by chance alone, and more than that where facts move together. A rule with no more facts apart than chance gives one time in twenty is marked *faint*, unless one of its facts is the same on every island of one arm, which chance cannot give.

![Facts that stand apart with each rule off](core.svg)

## Every rule, by kind

| rule | kind | facts apart | by chance alone | differ, of those both keep | not kept | the twelve out of their band | time of a run |
|---|---|---:|---:|---:|---:|---|---:|
| `prices.enabled` | holds up the century | 74 | 0.53 | 3398 of 3678 | 3 | `income_share_work_1850` | 107% |
| `engel.enabled` | holds up the century | 596 | 0.49 | 3153 of 3439 | 242 | `pop_1900`, `urban_share_1900`, `income_share_land_1900` (not kept without it: `real_wage_chained_1850_vs_1750`, `real_wage_chained_1900_vs_1750`) | 99% |
| `machines.enabled` | holds up the century | 1599 | 0.53 | 3380 of 3647 | 34 | `real_wage_chained_1850_vs_1750`, `real_wage_chained_1900_vs_1750`, `towns_1900`, `urban_share_1900`, `rail_km_1900` | 81% |
| `works.enabled` | holds up the century | 1293 | 0.53 | 3397 of 3671 | 10 | `real_wage_chained_1850_vs_1750`, `real_wage_chained_1900_vs_1750`, `income_share_work_1850`, `income_share_work_1900`, `income_share_land_1900` | 228% |
| `works.spread_first` | holds up the century | 0 | 0.53 | 3391 of 3678 | 3 | `income_share_work_1900` | 77% |
| `railways.enabled` | holds up the century | 32 | 0.48 | 3082 of 3660 | 21 | `towns_1900`, `rail_km_1900` | 87% |
| `carting.enabled` | holds up the century | 144 | 0.51 | 3297 of 3583 | 98 | `urban_share_1900` | 83% |
| `people.enabled` | holds up the century | 451 | 0.52 | 3344 of 3632 | 49 | `pop_1800`, `pop_1850`, `towns_1900`, `income_share_work_1900` | 73% |
| `mines.enabled` | holds up the century | 271 | 0.53 | 3393 of 3677 | 4 | `pop_1850`, `income_share_work_1850` | 79% |
| `harvest.enabled` | shapes the island | 5 | 0.52 | 3374 of 3670 | 11 | none | 92% |
| `harvest.rent_as_paid` | shapes the island (faint) | 1 | 0.51 | 3296 of 3677 | 4 | none | 104% |
| `works.count_spread` | shapes the island | 21 | 0.53 | 3385 of 3675 | 6 | none | 90% |
| `works.family_extends` | shapes the island | 11 | 0.51 | 3297 of 3678 | 3 | none | 86% |
| `railways.order_book` | shapes the island (faint) | 1 | 0.48 | 3090 of 3681 | 0 | none | 80% |
| `people.shelves_go` | shapes the island (faint) | 1 | 0.51 | 3305 of 3681 | 0 | none | 75% |
| `towns.enabled` | shapes the island | 38 | 0.50 | 3248 of 3539 | 142 | none | 75% |
| `towns.rent_floor` | shapes the island (faint) | 1 | 0.53 | 3389 of 3677 | 4 | none | 75% |
| `canals.navvies` | shapes the island | 1 | 0.51 | 3290 of 3681 | 0 | none | 79% |
| `land.enabled` | shapes the island | 28 | 0.53 | 3396 of 3677 | 4 | none | 68% |
| `tree.seed_trades` | shapes the island | 22 | 0.53 | 3390 of 3677 | 4 | none | 78% |
| `money.true_total` | shapes the island (faint) | 2 | 0.53 | 3387 of 3677 | 4 | none | 80% |
| `longrun.enabled` | shapes the island | 48 | 0.51 | 3263 of 3641 | 40 | none | 80% |
| `transition.enabled` | shapes the island | 449 | 0.51 | 3272 of 3642 | 39 | none | 93% |
| `owners.enabled` | shapes the island (faint) | 1 | 0.51 | 3299 of 3583 | 98 | none | 77% |
| `tasks.enabled` | shapes the island | 206 | 0.30 | 1902 of 2641 | 1040 | none | 59% |
| `tasks.land` | shapes the island | 157 | 0.35 | 2276 of 3472 | 209 | none | 82% |
| `tasks.computing` | shapes the island | 98 | 0.27 | 1762 of 3671 | 10 | none | 89% |
| `tasks.models` | shapes the island | 27 | 0.09 | 608 of 3667 | 14 | none | 92% |
| `tasks.office_machines` | shapes the island | 48 | 0.41 | 2621 of 3663 | 18 | none | 94% |
| `woods.enabled` | shapes the island | 23 | 0.44 | 2803 of 3680 | 1 | none | 99% |
| `ground.enabled` | shapes the island | 55 | 0.40 | 2593 of 3634 | 47 | none | 96% |
| `ground.room` | shapes the island | 93 | 0.41 | 2637 of 3681 | 0 | none | 96% |
| `prices.no_spoil_markup` | idle | 0 | 0.53 | 3389 of 3677 | 4 | none | 99% |
| `chronicle.enabled` | idle | 0 | 0.00 | 0 of 3681 | 0 | none | 104% |
| `works.keep_order` | idle | 0 | 0.53 | 3392 of 3679 | 2 | none | 75% |
| `towns.donor_wage` | idle | 0 | 0.51 | 3311 of 3679 | 2 | none | 76% |
| `canals.enabled` | idle | 0 | 0.50 | 3207 of 3598 | 83 | none | 71% |
| `tree.plant_waits` | idle | 0 | 0.53 | 3392 of 3679 | 2 | none | 78% |
| `accounts.enabled` | idle | 0 | 0.00 | 0 of 2906 | 775 | none (not kept without it: `income_share_work_1850`, `income_share_work_1900`, `income_share_land_1900`) | 74% |
| `accounts.works` | idle | 0 | 0.00 | 0 of 3681 | 0 | none | 77% |
| `gauges.enabled` | idle | 0 | 0.00 | 0 of 3320 | 361 | none | 77% |
| `hands.ration` | idle | 0 | 0.53 | 3390 of 3678 | 3 | none | 76% |
| `woods.record` | idle | 0 | 0.00 | 0 of 3643 | 38 | none | 99% |
| `crossings.enabled` | idle | 0 | 0.53 | 3395 of 3680 | 1 | none | 93% |
| `ground.reach` | idle | 0 | 0.41 | 2630 of 3681 | 0 | none | 92% |

The time of a run is the processor's time on a run with the rule off, against the control's (104 s on the clock for a run of the control): under a hundred per cent, the rule costs that much to run. It is taken on a machine shared with other work, whose load changed through the table, so only a time far from a hundred per cent says much.

## What each rule that holds up the century holds up

- **`prices.enabled`**: `income_share_work_1850` (OUTSIDE).
  Largest apart: `emigrants` 3.3 to 1,800 (67.6 spreads); `farm_zigzag_beyond_weather` 1.32 to 4.13 (7.3 spreads); `farm_zigzag` 1.82 to 4.74 (6.6 spreads); `works_closing_chance_age_6_9` 0.036 to 0.055 (4.3 spreads); `output_ac1_grain` -0.709 to -0.359 (4.0 spreads).
- **`engel.enabled`**: `pop_1900` (OUTSIDE), `urban_share_1900` (moved), `income_share_land_1900` (OUTSIDE).
  Largest apart: `transition_F_2000` 0.059 to 0.188 (22.8 spreads); `transition_F_1990` 0.062 to 0.226 (21.0 spreads); `transition_F_2010` 0.057 to 0.167 (19.4 spreads); `transition_F_2020` 0.056 to 0.157 (19.3 spreads); `transition_F_2040` 0.055 to 0.156 (18.5 spreads).
- **`machines.enabled`**: `real_wage_chained_1850_vs_1750` (OUTSIDE), `real_wage_chained_1900_vs_1750` (moved), `towns_1900` (OUTSIDE), `urban_share_1900` (moved), `rail_km_1900` (OUTSIDE).
  Largest apart: `target_clerk_works_1950` 0.250 to 0.133 (12019.4 spreads); `transition_F_2020` 0.056 to 0.988 (177.7 spreads); `transition_F_2040` 0.055 to 0.991 (171.6 spreads); `transition_F_2030` 0.056 to 0.990 (169.7 spreads); `transition_F_2010` 0.057 to 0.986 (164.1 spreads).
- **`works.enabled`**: `real_wage_chained_1850_vs_1750` (OUTSIDE), `real_wage_chained_1900_vs_1750` (moved), `income_share_work_1850` (OUTSIDE), `income_share_work_1900` (OUTSIDE), `income_share_land_1900` (moved).
  Largest apart: `offices_share_services_2050` 1.000 to 0.066 (716.1 spreads); `hours_presence_2050` 8.0 to 35,136 (670.0 spreads); `hours_share_presence_2050` 0.000 to 0.077 (441.4 spreads); `gauge_floor_2050` 0.000 to 0.036 (219.5 spreads); `cells_industry_1750` 1.5 to 233.8 (77.4 spreads).
- **`works.spread_first`**: `income_share_work_1900` (OUTSIDE).
- **`railways.enabled`**: `towns_1900` (OUTSIDE), `rail_km_1900` (moved).
  Largest apart: `rail_km_1970` 2,836 to 1,119 (12.8 spreads); `rail_km_1960` 2,832 to 1,118 (12.8 spreads); `rail_km_1990` 2,842 to 1,132 (12.8 spreads); `rail_km_2000` 2,842 to 1,135 (12.7 spreads); `rail_km_2010` 2,842 to 1,147 (12.6 spreads).
- **`carting.enabled`**: `urban_share_1900` (OUTSIDE).
  Largest apart: `mine_share_1971_1990` 0.173 to 0.093 (8.5 spreads); `industry_share_2011_2030` 0.656 to 0.722 (6.9 spreads); `urban_share_1951_1970` 0.558 to 0.608 (5.5 spreads); `mine_share_2031_2050` 0.181 to 0.102 (5.5 spreads); `mine_share_last20` 0.181 to 0.102 (5.5 spreads).
- **`people.enabled`**: `pop_1800` (moved), `pop_1850` (moved), `towns_1900` (OUTSIDE), `income_share_work_1900` (OUTSIDE).
  Largest apart: `town_growth_pct_p10` 0.09 to 1.10 (110.7 spreads); `timber_in_wages_2050` 0.134 to 3.49 (19.0 spreads); `town_growth_near_old_cap` 0.125 to 0.980 (12.7 spreads); `people_peak` 1,073,645 to 3,474,394 (11.0 spreads); `pop_2050` 1,073,645 to 3,474,394 (11.0 spreads).
- **`mines.enabled`**: `pop_1850` (OUTSIDE), `income_share_work_1850` (OUTSIDE).
  Largest apart: `area_mines_1750` 0.328 to 746 (5556.3 spreads); `area_mines_1800` 1.11 to 933 (2962.0 spreads); `workings_share_of_cell_1750` 0.001 to 1.649 (1855.9 spreads); `area_mines_1850` 2.60 to 1,025 (1385.6 spreads); `area_mines_1980` 4.76 to 961 (827.9 spreads).

## The rules that shape the island

- **`harvest.enabled`**, 5 apart: `output_ac1_grain` -0.709 to -0.501 (2.4 spreads); `price_ac1_grain` -0.561 to -0.132 (2.3 spreads); `price_swing_grain` 0.107 to 0.053 (1.8 spreads); `output_swing_grain` 0.108 to 0.046 (1.7 spreads); `farm_zigzag` 1.82 to 1.30 (1.2 spreads).
- **`harvest.rent_as_paid`**, 1 apart: `urban_share_1951_1970` 0.558 to 0.545 (1.3 spreads).
- **`works.count_spread`**, 21 apart: `works_closing_chance_age_6_9` 0.036 to 0.062 (6.1 spreads); `works_closing_chance_age_10_14` 0.032 to 0.059 (4.6 spreads); `works_closed` 7,443 to 13,400 (3.5 spreads); `works_founded` 9,775 to 16,245 (2.6 spreads); `works_slots_used` 10,233 to 16,750 (2.6 spreads).
- **`works.family_extends`**, 11 apart: `works_closing_chance_age_25_49` 0.017 to 0.021 (1.9 spreads); `works_closed` 7,443 to 10,587 (1.8 spreads); `works_closing_chance_age_15_24` 0.025 to 0.031 (1.6 spreads); `works_founded` 9,775 to 13,408 (1.5 spreads); `works_slots_used` 10,233 to 13,866 (1.5 spreads).
- **`railways.order_book`**, 1 apart: `rail_spent_over_expected` 0.80 to 1.33 (1.9 spreads).
- **`people.shelves_go`**, 1 apart: `books_goods_worst` 3.87e-10 to 5.75e-15 (0.1 spreads).
- **`towns.enabled`**, 38 apart: `urban_churn_10y` 0.001 to 0.078 (33.6 spreads); `cells_urban_1850` 207.2 to 87 (4.0 spreads); `cells_urban_2050` 533.5 to 140.5 (3.1 spreads); `cells_urban_last` 533.5 to 140.5 (3.1 spreads); `cells_urban_2040` 533.4 to 143.9 (3.0 spreads).
- **`towns.rent_floor`**, 1 apart: `works_closing_chance_age_10_14` 0.032 to 0.028 (0.6 spreads).
- **`canals.navvies`**, 1 apart: `canal_most_navvies` 1,211 to 0 (1.4 spreads).
- **`land.enabled`**, 28 apart: `workings_claypit_1960` 213 to 141.4 (2.4 spreads); `workings_claypit_1930` 209 to 140.6 (2.4 spreads); `workings_claypit_1950` 210.8 to 141 (2.3 spreads); `workings_claypit_1940` 209.6 to 140.8 (2.3 spreads); `workings_claypit_1970` 214.5 to 142.2 (2.3 spreads).
- **`tree.seed_trades`**, 22 apart: `hours_share_haul_1750` 0.258 to 0.236 (5.5 spreads); `hours_share_clerk_1750` 0.101 to 0.120 (4.7 spreads); `hours_share_presence_1750` 0.036 to 0.044 (4.6 spreads); `gauge_hand_hours_1750` 1.383 to 1.446 (3.9 spreads); `owners_works_hh_1750` 0.204 to 0.236 (3.0 spreads).
- **`money.true_total`**, 2 apart: `anchor_2040` 0.69 to 0.87 (1.3 spreads); `anchor_2050` 0.79 to 0.99 (1.2 spreads).
- **`longrun.enabled`**, 48 apart: `months_at_bound_electricity` 0.042 to 0.330 (4.3 spreads); `months_at_bound_fertiliser` 0.000 to 0.394 (3.8 spreads); `months_at_bound_oil` 0.000 to 0.415 (1.7 spreads); `months_at_bound_machinery` 0.000 to 0.250 (1.6 spreads); `carters_between_towns_share_of_hands_2050` 0.069 to 0.047 (1.6 spreads).
- **`transition.enabled`**, 449 apart: `town_growth_pct_p10` 0.09 to 0.98 (98.0 spreads); `timber_in_wages_2050` 0.134 to 2.32 (12.4 spreads); `late_town_people_median` 24,643 to 91,454 (12.1 spreads); `people_peak` 1,073,645 to 3,474,218 (11.0 spreads); `pop_2050` 1,073,645 to 3,474,218 (11.0 spreads).
- **`owners.enabled`**, 1 apart: `income_pay_share_1750` 0.937 to 0.885 (1.6 spreads).
- **`tasks.enabled`**, 206 apart: `offices_share_services_2050` 1.000 to 0.884 (88.9 spreads); `gauge_floor_2050` 0.000 to 0.012 (71.1 spreads); `gauge_floor_2040` 0.000 to 0.014 (11.7 spreads); `offices_share_services_2040` 0.996 to 0.881 (8.7 spreads); `transition_F_2020` 0.056 to 0.083 (5.2 spreads).
- **`tasks.land`**, 157 apart: `oil_to_machines_1940` 0.991 to 0.000 (12.0 spreads); `oil_to_machines_1950` 0.988 to 0.000 (11.4 spreads); `machine_share_tend_1990` 0.145 to 0.000 (9.4 spreads); `oil_to_machines_1980` 0.989 to 0.000 (9.0 spreads); `oil_to_machines_1970` 0.993 to 0.000 (8.8 spreads).
- **`tasks.computing`**, 98 apart: `offices_share_services_2050` 1.000 to 0.962 (28.8 spreads); `gauge_floor_2050` 0.000 to 0.004 (24.3 spreads); `hours_presence_2050` 8.0 to 1,021 (19.3 spreads); `hours_share_presence_2050` 0.000 to 0.003 (17.3 spreads); `target_clerk_works_2050` 0.810 to 0.250 (9.8 spreads).
- **`tasks.models`**, 27 apart: `target_judge_works_2050` 0.479 to 0.000 (18.8 spreads); `auto_judge_land_2050` 0.387 to 0.000 (15.9 spreads); `machine_share_judge_2050` 0.394 to 0.000 (13.6 spreads); `aim_judge_works_2050` 0.396 to 0.000 (13.2 spreads); `auto_judge_works_2050` 0.396 to 0.000 (13.2 spreads).
- **`tasks.office_machines`**, 48 apart: `target_clerk_works_1950` 0.250 to 0.000 (25731.6 spreads); `aim_clerk_works_1960` 0.249 to 0.010 (27.9 spreads); `auto_clerk_works_1960` 0.247 to 0.005 (27.5 spreads); `machine_share_clerk_1960` 0.247 to 0.005 (27.5 spreads); `margin_gap_clerk_1960` 1.81 to 7.17 (11.9 spreads).
- **`woods.enabled`**, 23 apart: `woods_cover_1930` 0.878 to 0.594 (11.6 spreads); `woods_cover_1940` 0.876 to 0.560 (4.0 spreads); `woods_cover_2040` 0.812 to 0.339 (3.5 spreads); `woods_cover_1980` 0.847 to 0.466 (3.4 spreads); `works_top_1pct_capacity_share_2050` 0.113 to 0.169 (3.2 spreads).
- **`ground.enabled`**, 55 apart: `rent_burden_1920` 0.109 to 0.166 (16.7 spreads); `rent_burden_2000` 0.109 to 0.168 (13.8 spreads); `rent_burden_1930` 0.110 to 0.167 (12.5 spreads); `rent_burden_1960` 0.110 to 0.164 (11.5 spreads); `rent_burden_1940` 0.111 to 0.164 (10.6 spreads).
- **`ground.room`**, 93 apart: `rent_burden_1920` 0.109 to 0.165 (16.5 spreads); `rent_burden_1930` 0.110 to 0.168 (12.7 spreads); `rent_burden_2000` 0.109 to 0.162 (12.4 spreads); `rent_burden_1960` 0.110 to 0.167 (12.3 spreads); `rent_burden_1940` 0.111 to 0.166 (10.9 spreads).

## The idle rules, and the second pass

- **`prices.no_spoil_markup`**: it stirs the island (3389 of the 3677 facts both keep differ) and moves no fact clear of the spread.
- **`chronicle.enabled`**: no fact differs on any island.
- **`works.keep_order`**: it stirs the island (3392 of the 3679 facts both keep differ) and moves no fact clear of the spread.
- **`towns.donor_wage`**: it stirs the island (3311 of the 3679 facts both keep differ) and moves no fact clear of the spread.
- **`canals.enabled`**: it stirs the island (3207 of the 3598 facts both keep differ) and moves no fact clear of the spread.
- **`tree.plant_waits`**: it stirs the island (3392 of the 3679 facts both keep differ) and moves no fact clear of the spread.
- **`accounts.enabled`**: no fact both keep differs on any island, and it keeps 775 facts the island does not keep without it: it watches.
- **`accounts.works`**: no fact differs on any island.
- **`gauges.enabled`**: no fact both keep differs on any island, and it keeps 361 facts the island does not keep without it: it watches.
- **`hands.ration`**: it stirs the island (3390 of the 3678 facts both keep differ) and moves no fact clear of the spread.
- **`woods.record`**: no fact both keep differs on any island, and it keeps 38 facts the island does not keep without it: it watches.
- **`crossings.enabled`**: it stirs the island (3395 of the 3680 facts both keep differ) and moves no fact clear of the spread.
- **`ground.reach`**: it stirs the island (2630 of the 3681 facts both keep differ) and moves no fact clear of the spread.

**The second pass**: the 13 idle rules turned off together are not the same island. With all of them off: 9 apart, the largest `anchor_2040` 0.69 to 1.09 (2.9 spreads); `anchor_2050` 0.79 to 1.25 (2.7 spreads); `anchor_2030` 0.62 to 0.94 (2.5 spreads); `anchor_2010` 0.46 to 0.67 (2.2 spreads); `anchor_2000` 0.39 to 0.58 (2.2 spreads). The smallest groups that must not go off together, each idle alone:

- `prices.no_spoil_markup` with `works.keep_order`: with both off, the century does not stand (`income_share_work_1850` OUTSIDE, `income_share_work_1900` OUTSIDE); 1 apart, the largest `carters_per_1000_weight_path_1900` 0.154 to 0.128 (1.7 spreads).
- `canals.enabled` with `tree.plant_waits`: with both off, 1 apart, the largest `carting_share_of_output_1800` 0.072 to 0.082 (1.3 spreads).
- `prices.no_spoil_markup` with `hands.ration` with `crossings.enabled`: with all of them off, the century does not stand (`income_share_work_1900` OUTSIDE); 3 apart, the largest `anchor_2040` 0.69 to 0.91 (1.5 spreads); `anchor_2030` 0.62 to 0.78 (1.3 spreads); `town_months_more_at_work_than_people` 0 to 7.2 (0.5 spreads).
- `hands.ration` with `ground.reach`: with both off, 1 apart, the largest `town_months_more_at_work_than_people` 0 to 6.9 (0.6 spreads).
- `towns.donor_wage` with `hands.ration`: with both off, 2 apart, the largest `oil_over_coal_heat_1920` 7.80 to 10.36 (1.3 spreads); `town_months_more_at_work_than_people` 0 to 10.6 (0.6 spreads).

With `works.keep_order` and `tree.plant_waits` and `hands.ration` and `crossings.enabled` and `ground.reach` kept on, the other 8 go off together and the island is the same: `prices.no_spoil_markup`, `chronicle.enabled`, `towns.donor_wage`, `canals.enabled`, `accounts.enabled`, `accounts.works`, `gauges.enabled`, `woods.record`.

The arms the second pass played, in order:

- 13 rules off: not the same island
  (prices.no_spoil_markup, chronicle.enabled, works.keep_order, towns.donor_wage, canals.enabled, tree.plant_waits, accounts.enabled, accounts.works, gauges.enabled, hands.ration, woods.record, crossings.enabled, ground.reach)
- 6 rules off: not the same island
  (prices.no_spoil_markup, chronicle.enabled, works.keep_order, towns.donor_wage, canals.enabled, tree.plant_waits)
- 7 rules off: the same island
  (accounts.enabled, accounts.works, gauges.enabled, hands.ration, woods.record, crossings.enabled, ground.reach)
- 3 rules off: not the same island
  (prices.no_spoil_markup, chronicle.enabled, works.keep_order)
- 3 rules off: the same island
  (towns.donor_wage, canals.enabled, tree.plant_waits)
- 2 rules off: the same island
  (chronicle.enabled, works.keep_order)
- 2 rules off: the same island
  (prices.no_spoil_markup, chronicle.enabled)
- 2 rules off: not the same island
  (prices.no_spoil_markup, works.keep_order)
- 12 rules off: not the same island
  (prices.no_spoil_markup, chronicle.enabled, towns.donor_wage, canals.enabled, tree.plant_waits, accounts.enabled, accounts.works, gauges.enabled, hands.ration, woods.record, crossings.enabled, ground.reach)
- 6 rules off: not the same island
  (prices.no_spoil_markup, chronicle.enabled, towns.donor_wage, canals.enabled, tree.plant_waits, accounts.enabled)
- 6 rules off: the same island
  (accounts.works, gauges.enabled, hands.ration, woods.record, crossings.enabled, ground.reach)
- 3 rules off: the same island
  (prices.no_spoil_markup, chronicle.enabled, towns.donor_wage)
- 3 rules off: not the same island
  (canals.enabled, tree.plant_waits, accounts.enabled)
- 2 rules off: the same island
  (tree.plant_waits, accounts.enabled)
- 2 rules off: not the same island
  (canals.enabled, tree.plant_waits)
- 2 rules off: the same island
  (canals.enabled, accounts.enabled)
- 11 rules off: not the same island
  (prices.no_spoil_markup, chronicle.enabled, towns.donor_wage, canals.enabled, accounts.enabled, accounts.works, gauges.enabled, hands.ration, woods.record, crossings.enabled, ground.reach)
- 5 rules off: the same island
  (prices.no_spoil_markup, chronicle.enabled, towns.donor_wage, canals.enabled, accounts.enabled)
- 8 rules off: not the same island
  (prices.no_spoil_markup, chronicle.enabled, accounts.works, gauges.enabled, hands.ration, woods.record, crossings.enabled, ground.reach)
- 9 rules off: not the same island
  (towns.donor_wage, canals.enabled, accounts.enabled, accounts.works, gauges.enabled, hands.ration, woods.record, crossings.enabled, ground.reach)
- 7 rules off: not the same island
  (prices.no_spoil_markup, accounts.works, gauges.enabled, hands.ration, woods.record, crossings.enabled, ground.reach)
- 7 rules off: the same island
  (chronicle.enabled, accounts.works, gauges.enabled, hands.ration, woods.record, crossings.enabled, ground.reach)
- 4 rules off: the same island
  (prices.no_spoil_markup, accounts.works, gauges.enabled, hands.ration)
- 4 rules off: the same island
  (prices.no_spoil_markup, woods.record, crossings.enabled, ground.reach)
- 5 rules off: the same island
  (prices.no_spoil_markup, accounts.works, woods.record, crossings.enabled, ground.reach)
- 6 rules off: not the same island
  (prices.no_spoil_markup, gauges.enabled, hands.ration, woods.record, crossings.enabled, ground.reach)
- 5 rules off: the same island
  (prices.no_spoil_markup, gauges.enabled, woods.record, crossings.enabled, ground.reach)
- 5 rules off: not the same island
  (prices.no_spoil_markup, hands.ration, woods.record, crossings.enabled, ground.reach)
- 3 rules off: the same island
  (prices.no_spoil_markup, hands.ration, woods.record)
- 4 rules off: not the same island
  (prices.no_spoil_markup, hands.ration, crossings.enabled, ground.reach)
- 3 rules off: not the same island
  (prices.no_spoil_markup, hands.ration, crossings.enabled)
- 3 rules off: not the same island
  (prices.no_spoil_markup, hands.ration, ground.reach)
- 10 rules off: not the same island
  (prices.no_spoil_markup, chronicle.enabled, towns.donor_wage, canals.enabled, accounts.enabled, accounts.works, gauges.enabled, hands.ration, woods.record, ground.reach)
- 5 rules off: not the same island
  (accounts.works, gauges.enabled, hands.ration, woods.record, ground.reach)
- 2 rules off: the same island
  (accounts.works, gauges.enabled)
- 3 rules off: not the same island
  (hands.ration, woods.record, ground.reach)
- 2 rules off: the same island
  (woods.record, ground.reach)
- 2 rules off: the same island
  (hands.ration, woods.record)
- 2 rules off: not the same island
  (hands.ration, ground.reach)
- 9 rules off: not the same island
  (prices.no_spoil_markup, chronicle.enabled, towns.donor_wage, canals.enabled, accounts.enabled, accounts.works, gauges.enabled, hands.ration, woods.record)
- 4 rules off: the same island
  (prices.no_spoil_markup, chronicle.enabled, towns.donor_wage, canals.enabled)
- 5 rules off: the same island
  (accounts.enabled, accounts.works, gauges.enabled, hands.ration, woods.record)
- 7 rules off: the same island
  (prices.no_spoil_markup, chronicle.enabled, accounts.enabled, accounts.works, gauges.enabled, hands.ration, woods.record)
- 7 rules off: not the same island
  (towns.donor_wage, canals.enabled, accounts.enabled, accounts.works, gauges.enabled, hands.ration, woods.record)
- 6 rules off: not the same island
  (towns.donor_wage, accounts.enabled, accounts.works, gauges.enabled, hands.ration, woods.record)
- 6 rules off: the same island
  (canals.enabled, accounts.enabled, accounts.works, gauges.enabled, hands.ration, woods.record)
- 3 rules off: the same island
  (towns.donor_wage, accounts.enabled, accounts.works)
- 4 rules off: not the same island
  (towns.donor_wage, gauges.enabled, hands.ration, woods.record)
- 2 rules off: the same island
  (towns.donor_wage, gauges.enabled)
- 3 rules off: not the same island
  (towns.donor_wage, hands.ration, woods.record)
- 2 rules off: not the same island
  (towns.donor_wage, hands.ration)
- 2 rules off: the same island
  (towns.donor_wage, woods.record)
- 8 rules off: the same island
  (prices.no_spoil_markup, chronicle.enabled, towns.donor_wage, canals.enabled, accounts.enabled, accounts.works, gauges.enabled, woods.record)

## The century after 1900

What the design asks of the years after 1900, as the standing check reports it (`docs/rules/README.md`, "The standing check"); reported, not counted in the kinds. Only the arms that differ from the control are listed.

| | per cent a year, 1900 to 1970 | 2050 over 1970 | 2050 less 1900 |
|---|---:|---:|---:|
| the control | 1.37% (met) | 1.27 (met) | -0.073 (met) |
| `prices.enabled` off | 1.53% (met) | 1.31 (met) | -0.087 (met) |
| `prices.no_spoil_markup` off | 1.44% (met) | 1.20 (met) | -0.087 (met) |
| `engel.enabled` off | - (-) | - (-) | -0.068 (met) |
| `machines.enabled` off | 0.32% (missed) | 0.87 (missed) | +0.102 (missed) |
| `harvest.enabled` off | 1.50% (met) | 1.16 (met) | -0.073 (met) |
| `harvest.rent_as_paid` off | 1.36% (met) | 1.28 (met) | -0.070 (met) |
| `works.enabled` off | 1.02% (met) | 1.17 (met) | -0.030 (met) |
| `works.count_spread` off | 1.48% (met) | 1.24 (met) | -0.074 (met) |
| `works.family_extends` off | 1.44% (met) | 1.17 (met) | -0.076 (met) |
| `works.spread_first` off | 1.41% (met) | 1.19 (met) | -0.077 (met) |
| `works.keep_order` off | 1.35% (met) | 1.24 (met) | -0.071 (met) |
| `railways.enabled` off | 1.39% (met) | 1.16 (met) | -0.061 (met) |
| `railways.order_book` off | 1.41% (met) | 1.18 (met) | -0.076 (met) |
| `carting.enabled` off | 1.40% (met) | 1.25 (met) | -0.084 (met) |
| `people.enabled` off | 1.17% (met) | 0.84 (missed) | -0.034 (met) |
| `people.shelves_go` off | 1.42% (met) | 1.26 (met) | -0.088 (met) |
| `towns.enabled` off | 1.51% (met) | 1.22 (met) | -0.082 (met) |
| `towns.donor_wage` off | 1.45% (met) | 1.22 (met) | -0.078 (met) |
| `towns.rent_floor` off | 1.52% (met) | 1.20 (met) | -0.081 (met) |
| `mines.enabled` off | 1.44% (met) | 1.37 (met) | -0.084 (met) |
| `canals.enabled` off | 1.47% (met) | 1.20 (met) | -0.082 (met) |
| `canals.navvies` off | 1.41% (met) | 1.24 (met) | -0.075 (met) |
| `land.enabled` off | 1.32% (met) | 1.26 (met) | -0.078 (met) |
| `tree.seed_trades` off | 1.43% (met) | 1.23 (met) | -0.077 (met) |
| `tree.plant_waits` off | 1.52% (met) | 1.22 (met) | -0.089 (met) |
| `hands.ration` off | 1.47% (met) | 1.14 (met) | -0.076 (met) |
| `money.true_total` off | 1.40% (met) | 1.19 (met) | -0.076 (met) |
| `longrun.enabled` off | 1.41% (met) | 1.26 (met) | -0.062 (met) |
| `transition.enabled` off | 1.33% (met) | 0.80 (missed) | -0.028 (met) |
| `owners.enabled` off | 1.48% (met) | 1.22 (met) | -0.082 (met) |
| `tasks.enabled` off | 1.17% (met) | 1.01 (met) | -0.048 (met) |
| `tasks.land` off | 1.27% (met) | 1.18 (met) | -0.029 (met) |
| `tasks.computing` off | 1.39% (met) | 1.11 (met) | -0.078 (met) |
| `tasks.models` off | 1.37% (met) | 1.18 (met) | -0.072 (met) |
| `tasks.office_machines` off | 1.41% (met) | 1.24 (met) | -0.079 (met) |
| `woods.enabled` off | 1.41% (met) | 1.21 (met) | -0.075 (met) |
| `crossings.enabled` off | 1.41% (met) | 1.21 (met) | -0.078 (met) |
| `ground.enabled` off | 1.44% (met) | 1.21 (met) | -0.079 (met) |
| `ground.room` off | 1.44% (met) | 1.13 (met) | -0.069 (met) |
| `ground.reach` off | 1.41% (met) | 1.22 (met) | -0.077 (met) |

## What was left out, and why

- `F_max`: 60000 under the game's rules and 20000 under the port's; fixed when the island is made, and not a switch: its off is not one value.
- `towns.mean_hands`: switched off under the game's rules on 20 November 2026, when the table found it idle, and kept in the code: `--set towns.mean_hands=true` plays it.
- `tree.stage`: 6 under the game's rules and 0 under the port's; fixed when the island is made, and not a switch: its off is not one value.
- `tree.canal_stone`: 0.35 under the game's rules and 1.0 under the port's: a number or a choice, not a switch, so its off is not one value.
- `roads.enabled`: off under the game's rules already: there is nothing to turn off.
- `commons.carry_debt`: switched off under the game's rules on 20 November 2026, when the table found it idle, and kept in the code: `--set commons.carry_debt=true` plays it.

## How to read it

The rules asked are found in the tree's own lists, not in one kept for this page: every true-or-false name the rulebook's walk (`rulebook.rs`) finds on under the game's rules and off under the port's, and every lesser switch of a rule the game switches on. A rule added tomorrow is asked too. An arm turns one of them off from 1750 with everything else as it is, exactly what `sweep --set NAME=false` plays.

A fact **stands apart** when every island of the arm lies above, or below, every island of the control, as `sweep --compare` stars a fact (4 islands or more a side). Only the facts both arms keep are counted, and not those of Speed or News: how long a run took is no fact about the island, and the news is what the chronicle writes about it. A fact's **size** is the change of the means over the control's spread from island to island; "from nothing" is a fact that is the same on every island of the control.

The kinds, by those numbers alone: **leaned on**, a run fails with the rule off (other rules lean on it); **holds up the century**, one of the standing check's twelve facts of 1750 to 1900 is not inside the control's band (moved, OUTSIDE, or without a value); **shapes the island**, facts stand apart and the twelve stay inside; **idle**, nothing stands apart. An idle rule that keeps facts the island does not keep without it watches the island (the books, the gauges) and is idle by design.

Rules cover for each other: two that mend one behaviour each look idle alone. So **the second pass** turns every idle rule off together. If that arm is idle too, the island is the same island with fewer rules. If not, the set is halved until the smallest groups that must not go off together are found; one of each group is kept on, and the rest are tried again.

Nothing here switches a rule off under the game's rules. What is switched off is decided after the table is read, and the rule's page then says since when and why.
