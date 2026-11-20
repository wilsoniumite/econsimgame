# The core of the island

Written by `isostate-core ablate`, played on 2026-11-20 (UTC) from the code at `70d9d70-dirty`. Nothing on this page is written by hand: to make it again, run

```text
isostate-core ablate --out DIR --page docs/core.md
```

This one was made by `isostate-core ablate --out DIR --page docs/core.md --quiet`.

Every rule the game switches on is turned off from 1750, one at a time, and the island is played again: islands 1, 2, 3, 4, 5, 6, 7, 8 under the game's rules with 6 families, 1750 to 2050. Each such arm is set against the control, the islands as they are, island by island. 392 runs were played, 0 failed.

Of the 47 rules asked, 9 hold up the century, 25 shape the island, 13 are idle, 0 are leaned on.

Read the small counts with care. With 8 islands a side, a fact the rule does not move, only stirs (a rule that changes anything sends each island down another path), stands apart by chance once in 6,435 facts; a rule that stirs the island stirs some 3,100 facts, so 0.48 of them would stand apart by chance alone, and more than that where facts move together. A rule with no more facts apart than chance gives one time in twenty is marked *faint*, unless one of its facts is the same on every island of one arm, which chance cannot give.

![Facts that stand apart with each rule off](core.svg)

## Every rule, by kind

| rule | kind | facts apart | by chance alone | differ, of those both keep | not kept | the twelve out of their band | time of a run |
|---|---|---:|---:|---:|---:|---|---:|
| `prices.enabled` | holds up the century | 53 | 0.53 | 3393 of 3678 | 0 | `pop_1850` | 107% |
| `engel.enabled` | holds up the century | 569 | 0.49 | 3150 of 3440 | 238 | `pop_1900`, `urban_share_1900`, `income_share_land_1900` (not kept without it: `real_wage_chained_1850_vs_1750`, `real_wage_chained_1900_vs_1750`) | 97% |
| `machines.enabled` | holds up the century | 1619 | 0.52 | 3349 of 3623 | 55 | `real_wage_chained_1900_vs_1750`, `towns_1900`, `urban_share_1900` | 80% |
| `works.enabled` | holds up the century | 1298 | 0.53 | 3392 of 3671 | 7 | `real_wage_chained_1850_vs_1750`, `real_wage_chained_1900_vs_1750`, `income_share_work_1900`, `income_share_land_1900` | 249% |
| `railways.enabled` | holds up the century | 43 | 0.48 | 3086 of 3661 | 17 | `towns_1900`, `rail_km_1900` | 101% |
| `carting.enabled` | holds up the century | 181 | 0.51 | 3291 of 3584 | 94 | `urban_share_1900` | 95% |
| `people.enabled` | holds up the century | 517 | 0.52 | 3337 of 3628 | 50 | `pop_1800`, `pop_1850`, `towns_1900`, `income_share_land_1900` | 92% |
| `mines.enabled` | holds up the century | 256 | 0.53 | 3396 of 3677 | 1 | `pop_1800`, `pop_1850`, `pop_1900` | 94% |
| `land.enabled` | holds up the century | 44 | 0.53 | 3387 of 3677 | 1 | `income_share_land_1900` | 83% |
| `prices.no_spoil_markup` | shapes the island (faint) | 1 | 0.53 | 3390 of 3678 | 0 | none | 97% |
| `harvest.enabled` | shapes the island | 5 | 0.52 | 3369 of 3671 | 7 | none | 92% |
| `works.count_spread` | shapes the island | 15 | 0.53 | 3386 of 3676 | 2 | none | 95% |
| `works.family_extends` | shapes the island | 12 | 0.51 | 3288 of 3677 | 1 | none | 99% |
| `railways.order_book` | shapes the island (faint) | 1 | 0.48 | 3082 of 3678 | 0 | none | 92% |
| `people.shelves_go` | shapes the island (faint) | 1 | 0.51 | 3292 of 3678 | 0 | none | 93% |
| `towns.enabled` | shapes the island | 36 | 0.50 | 3243 of 3538 | 140 | none | 93% |
| `towns.rent_floor` | shapes the island (faint) | 1 | 0.53 | 3384 of 3677 | 1 | none | 93% |
| `canals.enabled` | shapes the island (faint) | 2 | 0.50 | 3204 of 3595 | 83 | none | 85% |
| `canals.navvies` | shapes the island | 4 | 0.51 | 3283 of 3676 | 2 | none | 94% |
| `tree.seed_trades` | shapes the island | 19 | 0.53 | 3383 of 3678 | 0 | none | 98% |
| `tree.plant_waits` | shapes the island | 3 | 0.52 | 3377 of 3678 | 0 | none | 96% |
| `hands.ration` | shapes the island | 1 | 0.53 | 3386 of 3678 | 0 | none | 96% |
| `money.true_total` | shapes the island (faint) | 1 | 0.53 | 3386 of 3677 | 1 | none | 92% |
| `longrun.enabled` | shapes the island | 26 | 0.51 | 3253 of 3640 | 38 | none | 95% |
| `transition.enabled` | shapes the island | 569 | 0.51 | 3262 of 3640 | 38 | none | 110% |
| `owners.enabled` | shapes the island | 4 | 0.51 | 3290 of 3580 | 98 | none | 93% |
| `tasks.enabled` | shapes the island | 254 | 0.29 | 1898 of 2641 | 1037 | none | 68% |
| `tasks.land` | shapes the island | 176 | 0.35 | 2273 of 3469 | 209 | none | 90% |
| `tasks.computing` | shapes the island | 104 | 0.27 | 1764 of 3668 | 10 | none | 92% |
| `tasks.models` | shapes the island | 29 | 0.10 | 615 of 3664 | 14 | none | 91% |
| `tasks.office_machines` | shapes the island | 57 | 0.41 | 2619 of 3660 | 18 | none | 91% |
| `woods.enabled` | shapes the island | 42 | 0.43 | 2797 of 3676 | 2 | none | 94% |
| `ground.enabled` | shapes the island | 58 | 0.40 | 2577 of 3631 | 47 | none | 92% |
| `ground.room` | shapes the island | 92 | 0.41 | 2624 of 3678 | 0 | none | 91% |
| `harvest.rent_as_paid` | idle | 0 | 0.51 | 3285 of 3678 | 0 | none | 96% |
| `chronicle.enabled` | idle | 0 | 0.00 | 0 of 3678 | 0 | none | 95% |
| `works.spread_first` | idle | 0 | 0.53 | 3382 of 3677 | 1 | none | 95% |
| `works.keep_order` | idle | 0 | 0.53 | 3385 of 3676 | 2 | none | 94% |
| `towns.mean_hands` | idle | 0 | 0.53 | 3388 of 3678 | 0 | none | 93% |
| `towns.donor_wage` | idle | 0 | 0.51 | 3300 of 3675 | 3 | none | 94% |
| `accounts.enabled` | idle | 0 | 0.00 | 0 of 2906 | 772 | none (not kept without it: `income_share_work_1850`, `income_share_work_1900`, `income_share_land_1900`) | 87% |
| `accounts.works` | idle | 0 | 0.00 | 0 of 3678 | 0 | none | 90% |
| `gauges.enabled` | idle | 0 | 0.00 | 0 of 3317 | 361 | none | 92% |
| `commons.carry_debt` | idle | 0 | 0.00 | 0 of 3678 | 0 | none | 92% |
| `woods.record` | idle | 0 | 0.00 | 0 of 3640 | 38 | none | 92% |
| `crossings.enabled` | idle | 0 | 0.53 | 3388 of 3678 | 0 | none | 92% |
| `ground.reach` | idle | 0 | 0.41 | 2620 of 3678 | 0 | none | 84% |

The time of a run is the processor's time on a run with the rule off, against the control's (81 s on the clock for a run of the control): under a hundred per cent, the rule costs that much to run. It is taken on a machine shared with other work, whose load changed through the table, so only a time far from a hundred per cent says much.

## What each rule that holds up the century holds up

- **`prices.enabled`**: `pop_1850` (OUTSIDE).
  Largest apart: `emigrants` 3.0 to 1,983 (84.4 spreads); `farm_zigzag_beyond_weather` 1.43 to 3.90 (6.0 spreads); `price_swing_cloth` 0.052 to 0.102 (4.6 spreads); `output_ac1_grain` -0.716 to -0.330 (4.3 spreads); `grain_price_year_after_harvest` 0.388 to -0.606 (3.5 spreads).
- **`engel.enabled`**: `pop_1900` (OUTSIDE), `urban_share_1900` (moved), `income_share_land_1900` (OUTSIDE).
  Largest apart: `transition_F_2030` 0.056 to 0.162 (23.5 spreads); `transition_F_2050` 0.056 to 0.163 (21.4 spreads); `transition_F_2020` 0.057 to 0.164 (21.2 spreads); `transition_F_2040` 0.056 to 0.162 (21.2 spreads); `transition_F_2010` 0.058 to 0.174 (18.8 spreads).
- **`machines.enabled`**: `real_wage_chained_1900_vs_1750` (moved), `towns_1900` (OUTSIDE), `urban_share_1900` (moved).
  Largest apart: `transition_F_2030` 0.056 to 0.990 (208.2 spreads); `transition_F_2040` 0.056 to 0.991 (187.5 spreads); `transition_F_2050` 0.056 to 0.993 (186.0 spreads); `transition_F_2020` 0.057 to 0.988 (184.4 spreads); `transition_F_2010` 0.058 to 0.986 (150.5 spreads).
- **`works.enabled`**: `real_wage_chained_1850_vs_1750` (OUTSIDE), `real_wage_chained_1900_vs_1750` (OUTSIDE), `income_share_work_1900` (OUTSIDE), `income_share_land_1900` (moved).
  Largest apart: `offices_share_services_2050` 0.999 to 0.039 (305.9 spreads); `hours_presence_2050` 28.4 to 35,912 (259.8 spreads); `hours_share_presence_2050` 0.000 to 0.078 (187.9 spreads); `cells_industry_1750` 1.8 to 236.2 (78.2 spreads); `gauge_floor_2050` 0.000 to 0.036 (76.6 spreads).
- **`railways.enabled`**: `towns_1900` (OUTSIDE), `rail_km_1900` (moved).
  Largest apart: `canal_traffic_end_vs_peak` 0.01 to 0.55 (14.4 spreads); `canal_mton_km_2050` 6.95 to 308 (14.2 spreads); `rail_km_1980` 2,795 to 1,108 (12.4 spreads); `rail_km_1970` 2,791 to 1,104 (12.2 spreads); `rail_km_2010` 2,800 to 1,121 (12.2 spreads).
- **`carting.enabled`**: `urban_share_1900` (moved).
  Largest apart: `urban_share_2000` 0.561 to 0.615 (4.8 spreads); `mine_share_1751_1770` 0.164 to 0.057 (4.7 spreads); `industry_share_1991_2010` 0.662 to 0.725 (4.5 spreads); `industry_share_1971_1990` 0.659 to 0.724 (4.4 spreads); `urban_share_1991_2010` 0.561 to 0.611 (4.4 spreads).
- **`people.enabled`**: `pop_1800` (moved), `pop_1850` (moved), `towns_1900` (OUTSIDE), `income_share_land_1900` (OUTSIDE).
  Largest apart: `town_growth_pct_p10` 0.09 to 1.10 (115.2 spreads); `timber_in_wages_2050` 0.150 to 3.59 (22.8 spreads); `hours_clerk_2050` 47,597 to 138,229 (11.7 spreads); `works_hands_within_2_of_market_2010` 1.000 to 0.984 (11.7 spreads); `cells_urban_2050` 522.4 to 1,508 (11.6 spreads).
- **`mines.enabled`**: `pop_1800` (OUTSIDE), `pop_1850` (OUTSIDE), `pop_1900` (OUTSIDE).
  Largest apart: `area_mines_1750` 0.327 to 745 (5533.7 spreads); `area_mines_1800` 1.14 to 929 (2548.2 spreads); `workings_share_of_cell_1750` 0.002 to 1.639 (1736.6 spreads); `area_mines_1850` 2.49 to 1,016 (1468.7 spreads); `workings_share_of_cell_1800` 0.005 to 0.946 (916.6 spreads).
- **`land.enabled`**: `income_share_land_1900` (OUTSIDE).
  Largest apart: `area_wild_1900` 555 to 606 (2.2 spreads); `works_hands_within_2_of_market_1750` 0.887 to 0.807 (2.1 spreads); `area_farm_1920` 1,526 to 1,383 (2.1 spreads); `workings_claypit_1970` 212.6 to 139.2 (2.0 spreads); `workings_claypit_2040` 224.9 to 148.1 (2.0 spreads).

## The rules that shape the island

- **`prices.no_spoil_markup`**, 1 apart: `electricity_lost_1900` 0.075 to 0.169 (1.8 spreads).
- **`harvest.enabled`**, 5 apart: `price_ac1_grain` -0.566 to -0.129 (2.9 spreads); `output_ac1_grain` -0.716 to -0.508 (2.3 spreads); `price_swing_grain` 0.108 to 0.051 (1.7 spreads); `output_swing_grain` 0.110 to 0.045 (1.5 spreads); `farm_zigzag` 1.91 to 1.24 (0.9 spreads).
- **`works.count_spread`**, 15 apart: `works_closed` 7,539 to 12,370 (6.2 spreads); `works_closing_chance_age_10_14` 0.031 to 0.056 (4.8 spreads); `works_founded` 9,994 to 14,945 (3.6 spreads); `works_slots_used` 10,451 to 15,450 (3.5 spreads); `works_closing_chance_age_6_9` 0.037 to 0.061 (3.4 spreads).
- **`works.family_extends`**, 12 apart: `waiting_haul_pits_1920` 0.006 to 0.000 (5.9 spreads); `aim_haul_pits_1920` 0.012 to 0.000 (5.5 spreads); `electricity_lost_1900` 0.075 to 0.331 (4.9 spreads); `auto_haul_pits_1920` 0.006 to 0.000 (4.4 spreads); `works_closed` 7,539 to 10,689 (4.1 spreads).
- **`railways.order_book`**, 1 apart: `rail_spent_over_expected` 0.78 to 1.41 (6.1 spreads).
- **`people.shelves_go`**, 1 apart: `books_goods_worst` 1.21e-10 to 7.70e-15 (0.1 spreads).
- **`towns.enabled`**, 36 apart: `books_income_worst` 5.21e-15 to 6.58e-14 (32.9 spreads); `urban_churn_10y` 0.001 to 0.078 (24.7 spreads); `cells_urban_2040` 521.5 to 143.9 (4.9 spreads); `cells_urban_2030` 519.8 to 148.9 (4.7 spreads); `cells_urban_2000` 519.1 to 156.6 (4.5 spreads).
- **`towns.rent_floor`**, 1 apart: `books_income_worst` 5.21e-15 to 1.52e-14 (5.4 spreads).
- **`canals.enabled`**, 2 apart: `electricity_lost_1900` 0.075 to 0.298 (4.2 spreads); `carting_share_of_output_1800` 0.072 to 0.081 (1.4 spreads).
- **`canals.navvies`**, 4 apart: `electricity_lost_1900` 0.075 to 0.364 (5.5 spreads); `waiting_haul_pits_1920` 0.006 to 0.002 (3.8 spreads); `aim_haul_pits_1920` 0.012 to 0.004 (3.5 spreads); `canal_most_navvies` 1,166 to 0 (0.8 spreads).
- **`tree.seed_trades`**, 19 apart: `hours_share_haul_1750` 0.258 to 0.236 (5.9 spreads); `hours_share_clerk_1750` 0.101 to 0.119 (4.8 spreads); `hours_share_presence_1750` 0.036 to 0.044 (4.7 spreads); `owners_works_hh_1750` 0.204 to 0.237 (4.2 spreads); `owners_works_lords_1750` 0.728 to 0.695 (3.9 spreads).
- **`tree.plant_waits`**, 3 apart: `budget_wares_1980` 0.090 to 0.089 (1.9 spreads); `price_swing_timber` 0.427 to 0.356 (1.0 spreads); `price_swing_timber_from_1800` 0.459 to 0.383 (1.0 spreads).
- **`hands.ration`**, 1 apart: `town_months_more_at_work_than_people` 0 to 6.4 (0.6 spreads).
- **`money.true_total`**, 1 apart: `anchor_2050` 0.77 to 1.00 (1.5 spreads).
- **`longrun.enabled`**, 26 apart: `months_at_bound_oil` 0.000 to 0.402 (107.0 spreads); `months_at_bound_electricity` 0.037 to 0.320 (3.8 spreads); `months_at_bound_fertiliser` 0.000 to 0.367 (2.2 spreads); `wage_in_room_2040` 1.99 to 2.67 (1.9 spreads); `wage_in_room_2050` 2.08 to 2.77 (1.7 spreads).
- **`transition.enabled`**, 569 apart: `town_growth_pct_p10` 0.09 to 1.00 (103.3 spreads); `timber_in_wages_2050` 0.150 to 2.91 (18.3 spreads); `hours_clerk_2050` 47,597 to 161,816 (14.8 spreads); `cells_urban_2050` 522.4 to 1,706 (13.9 spreads); `cells_urban_last` 522.4 to 1,706 (13.9 spreads).
- **`owners.enabled`**, 4 apart: `income_pay_share_1750` 0.937 to 0.885 (1.5 spreads); `rent_burden_1900` 0.163 to 0.155 (1.4 spreads); `income_pay_share_1900` 0.878 to 0.819 (1.2 spreads); `income_pay_share_2050` 0.899 to 0.846 (0.8 spreads).
- **`tasks.enabled`**, 254 apart: `offices_share_services_2050` 0.999 to 0.790 (66.6 spreads); `gauge_floor_2050` 0.000 to 0.022 (46.5 spreads); `gauge_floor_2040` 0.000 to 0.022 (16.4 spreads); `offices_share_services_2040` 0.996 to 0.798 (12.5 spreads); `budget_wares_1980` 0.090 to 0.087 (8.4 spreads).
- **`tasks.land`**, 176 apart: `machine_share_tend_2050` 0.138 to 0.000 (11.2 spreads); `machine_share_tend_2000` 0.143 to 0.000 (10.4 spreads); `oil_to_machines_1960` 0.995 to 0.000 (8.0 spreads); `machine_share_tend_2020` 0.148 to 0.000 (7.6 spreads); `oil_to_machines_1950` 1.001 to 0.000 (7.6 spreads).
- **`tasks.computing`**, 104 apart: `target_clerk_works_2040` 0.746 to 0.250 (15.3 spreads); `auto_clerk_works_2050` 0.764 to 0.248 (14.7 spreads); `machine_share_clerk_2050` 0.764 to 0.248 (14.7 spreads); `aim_clerk_works_2050` 0.767 to 0.250 (14.4 spreads); `aim_clerk_works_2040` 0.700 to 0.250 (13.7 spreads).
- **`tasks.models`**, 29 apart: `auto_judge_land_2050` 0.390 to 0.000 (19.5 spreads); `machine_share_judge_2050` 0.395 to 0.000 (18.5 spreads); `aim_judge_works_2050` 0.397 to 0.000 (18.3 spreads); `auto_judge_works_2050` 0.397 to 0.000 (18.3 spreads); `target_judge_works_2050` 0.484 to 0.000 (15.8 spreads).
- **`tasks.office_machines`**, 57 apart: `target_clerk_works_1950` 0.250 to 0.000 (from nothing); `auto_clerk_works_1960` 0.247 to 0.006 (71.5 spreads); `machine_share_clerk_1960` 0.247 to 0.006 (71.5 spreads); `aim_clerk_works_1960` 0.250 to 0.012 (63.9 spreads); `margin_gap_clerk_1960` 1.81 to 6.99 (17.9 spreads).
- **`woods.enabled`**, 42 apart: `woods_cover_1920` 0.871 to 0.619 (7.1 spreads); `woods_cover_2020` 0.841 to 0.355 (6.2 spreads); `woods_cover_1990` 0.846 to 0.409 (5.8 spreads); `woods_cover_1930` 0.889 to 0.608 (5.6 spreads); `woods_cover_1940` 0.890 to 0.604 (5.5 spreads).
- **`ground.enabled`**, 58 apart: `rent_burden_1940` 0.109 to 0.168 (26.0 spreads); `rent_burden_2030` 0.108 to 0.167 (20.1 spreads); `rent_burden_1920` 0.109 to 0.164 (19.3 spreads); `rent_burden_1980` 0.109 to 0.162 (11.8 spreads); `rent_burden_2010` 0.109 to 0.169 (11.6 spreads).
- **`ground.room`**, 92 apart: `rent_burden_1940` 0.109 to 0.165 (25.0 spreads); `rent_burden_1920` 0.109 to 0.165 (19.5 spreads); `rent_burden_2030` 0.108 to 0.165 (19.5 spreads); `rent_burden_1980` 0.109 to 0.166 (12.8 spreads); `rent_burden_2010` 0.109 to 0.166 (11.0 spreads).

## The idle rules, and the second pass

- **`harvest.rent_as_paid`**: it stirs the island (3285 of the 3678 facts both keep differ) and moves no fact clear of the spread.
- **`chronicle.enabled`**: no fact differs on any island.
- **`works.spread_first`**: it stirs the island (3382 of the 3677 facts both keep differ) and moves no fact clear of the spread.
- **`works.keep_order`**: it stirs the island (3385 of the 3676 facts both keep differ) and moves no fact clear of the spread.
- **`towns.mean_hands`**: it stirs the island (3388 of the 3678 facts both keep differ) and moves no fact clear of the spread.
- **`towns.donor_wage`**: it stirs the island (3300 of the 3675 facts both keep differ) and moves no fact clear of the spread.
- **`accounts.enabled`**: no fact both keep differs on any island, and it keeps 772 facts the island does not keep without it: it watches.
- **`accounts.works`**: no fact differs on any island.
- **`gauges.enabled`**: no fact both keep differs on any island, and it keeps 361 facts the island does not keep without it: it watches.
- **`commons.carry_debt`**: no fact differs on any island.
- **`woods.record`**: no fact both keep differs on any island, and it keeps 38 facts the island does not keep without it: it watches.
- **`crossings.enabled`**: it stirs the island (3388 of the 3678 facts both keep differ) and moves no fact clear of the spread.
- **`ground.reach`**: it stirs the island (2620 of the 3678 facts both keep differ) and moves no fact clear of the spread.

**The second pass**: all 13 idle rules turned off together are the same island. That island is the same island with fewer rules.

The arms the second pass played, in order:

- 13 rules off: the same island
  (harvest.rent_as_paid, chronicle.enabled, works.spread_first, works.keep_order, towns.mean_hands, towns.donor_wage, accounts.enabled, accounts.works, gauges.enabled, commons.carry_debt, woods.record, crossings.enabled, ground.reach)

## The century after 1900

What the design asks of the years after 1900, as the standing check reports it (`docs/rules/README.md`, "The standing check"); reported, not counted in the kinds. Only the arms that differ from the control are listed.

| | per cent a year, 1900 to 1970 | 2050 over 1970 | 2050 less 1900 |
|---|---:|---:|---:|
| the control | 1.50% (met) | 1.22 (met) | -0.076 (met) |
| `prices.enabled` off | 1.54% (met) | 1.31 (met) | -0.100 (met) |
| `prices.no_spoil_markup` off | 1.38% (met) | 1.25 (met) | -0.080 (met) |
| `engel.enabled` off | - (-) | - (-) | -0.066 (met) |
| `machines.enabled` off | 0.38% (missed) | 0.85 (missed) | +0.082 (missed) |
| `harvest.enabled` off | 1.47% (met) | 1.17 (met) | -0.077 (met) |
| `harvest.rent_as_paid` off | 1.48% (met) | 1.22 (met) | -0.082 (met) |
| `works.enabled` off | 1.07% (met) | 1.12 (met) | -0.029 (met) |
| `works.count_spread` off | 1.45% (met) | 1.20 (met) | -0.079 (met) |
| `works.family_extends` off | 1.44% (met) | 1.26 (met) | -0.084 (met) |
| `works.spread_first` off | 1.46% (met) | 1.25 (met) | -0.082 (met) |
| `works.keep_order` off | 1.53% (met) | 1.16 (met) | -0.071 (met) |
| `railways.enabled` off | 1.43% (met) | 1.20 (met) | -0.078 (met) |
| `railways.order_book` off | 1.46% (met) | 1.22 (met) | -0.078 (met) |
| `carting.enabled` off | 1.47% (met) | 1.20 (met) | -0.088 (met) |
| `people.enabled` off | 1.25% (met) | 0.87 (missed) | -0.033 (met) |
| `people.shelves_go` off | 1.48% (met) | 1.22 (met) | -0.083 (met) |
| `towns.enabled` off | 1.51% (met) | 1.22 (met) | -0.082 (met) |
| `towns.mean_hands` off | 1.37% (met) | 1.27 (met) | -0.073 (met) |
| `towns.donor_wage` off | 1.45% (met) | 1.20 (met) | -0.082 (met) |
| `towns.rent_floor` off | 1.42% (met) | 1.18 (met) | -0.081 (met) |
| `mines.enabled` off | 1.56% (met) | 1.34 (met) | -0.087 (met) |
| `canals.enabled` off | 1.47% (met) | 1.22 (met) | -0.086 (met) |
| `canals.navvies` off | 1.50% (met) | 1.18 (met) | -0.073 (met) |
| `land.enabled` off | 1.38% (met) | 1.23 (met) | -0.080 (met) |
| `tree.seed_trades` off | 1.45% (met) | 1.17 (met) | -0.072 (met) |
| `tree.plant_waits` off | 1.54% (met) | 1.21 (met) | -0.079 (met) |
| `hands.ration` off | 1.48% (met) | 1.19 (met) | -0.077 (met) |
| `money.true_total` off | 1.45% (met) | 1.23 (met) | -0.078 (met) |
| `longrun.enabled` off | 1.39% (met) | 1.24 (met) | -0.066 (met) |
| `transition.enabled` off | 1.27% (met) | 0.79 (missed) | -0.040 (met) |
| `owners.enabled` off | 1.52% (met) | 1.21 (met) | -0.082 (met) |
| `tasks.enabled` off | 1.21% (met) | 1.01 (met) | -0.045 (met) |
| `tasks.land` off | 1.35% (met) | 1.14 (met) | -0.031 (met) |
| `tasks.computing` off | 1.50% (met) | 1.08 (met) | -0.078 (met) |
| `tasks.models` off | 1.50% (met) | 1.13 (met) | -0.075 (met) |
| `tasks.office_machines` off | 1.49% (met) | 1.19 (met) | -0.078 (met) |
| `woods.enabled` off | 1.49% (met) | 1.17 (met) | -0.067 (met) |
| `crossings.enabled` off | 1.48% (met) | 1.17 (met) | -0.078 (met) |
| `ground.enabled` off | 1.47% (met) | 1.22 (met) | -0.075 (met) |
| `ground.room` off | 1.50% (met) | 1.19 (met) | -0.073 (met) |
| `ground.reach` off | 1.51% (met) | 1.21 (met) | -0.085 (met) |

## What was left out, and why

- `F_max`: 60000 under the game's rules and 20000 under the port's; fixed when the island is made, and not a switch: its off is not one value.
- `tree.stage`: 6 under the game's rules and 0 under the port's; fixed when the island is made, and not a switch: its off is not one value.
- `tree.canal_stone`: 0.35 under the game's rules and 1.0 under the port's: a number or a choice, not a switch, so its off is not one value.
- `roads.enabled`: off under the game's rules already: there is nothing to turn off.

## How to read it

The rules asked are found in the tree's own lists, not in one kept for this page: every true-or-false name the rulebook's walk (`rulebook.rs`) finds on under the game's rules and off under the port's, and every lesser switch of a rule the game switches on. A rule added tomorrow is asked too. An arm turns one of them off from 1750 with everything else as it is, exactly what `sweep --set NAME=false` plays.

A fact **stands apart** when every island of the arm lies above, or below, every island of the control, as `sweep --compare` stars a fact (4 islands or more a side). Only the facts both arms keep are counted, and not those of Speed or News: how long a run took is no fact about the island, and the news is what the chronicle writes about it. A fact's **size** is the change of the means over the control's spread from island to island; "from nothing" is a fact that is the same on every island of the control.

The kinds, by those numbers alone: **leaned on**, a run fails with the rule off (other rules lean on it); **holds up the century**, one of the standing check's twelve facts of 1750 to 1900 is not inside the control's band (moved, OUTSIDE, or without a value); **shapes the island**, facts stand apart and the twelve stay inside; **idle**, nothing stands apart. An idle rule that keeps facts the island does not keep without it watches the island (the books, the gauges) and is idle by design.

Rules cover for each other: two that mend one behaviour each look idle alone. So **the second pass** turns every idle rule off together. If that arm is idle too, the island is the same island with fewer rules. If not, the set is halved until the smallest groups that must not go off together are found; one of each group is kept on, and the rest are tried again.

Nothing here switches a rule off under the game's rules. What is switched off is decided after the table is read, and the rule's page then says since when and why.
