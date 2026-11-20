# The core of the island

Written by `isostate-core ablate`, played on 2026-11-19 (UTC) from the code at `7e09f65`. Nothing on this page is written by hand: to make it again, run

```text
isostate-core ablate --out DIR --page docs/core.md
```

This one was made by `isostate-core ablate --out DIR --jobs 24 --page docs/core.md --quiet`.

Every rule the game switches on is turned off from 1750, one at a time, and the island is played again: islands 1, 2, 3, 4, 5, 6, 7, 8 under the game's rules with 6 families, 1750 to 2050. Each such arm is set against the control, the islands as they are, island by island. 344 runs were played, 0 failed.

Of the 41 rules asked, 9 hold up the century, 18 shape the island, 14 are idle, 0 are leaned on.

Read the small counts with care. With 8 islands a side, a fact the rule does not move, only stirs (a rule that changes anything sends each island down another path), stands apart by chance once in 6,435 facts; a rule that stirs the island stirs some 2,700 facts, so 0.43 of them would stand apart by chance alone, and more than that where facts move together. A rule with no more facts apart than chance gives one time in twenty is marked *faint*, unless one of its facts is the same on every island of one arm, which chance cannot give.

![Facts that stand apart with each rule off](core.svg)

## Every rule, by kind

| rule | kind | facts apart | by chance alone | differ, of those both keep | not kept | the twelve out of their band | time of a run |
|---|---|---:|---:|---:|---:|---|---:|
| `prices.enabled` | holds up the century | 78 | 0.45 | 2889 of 2993 | 1 | `pop_1850` | 110% |
| `engel.enabled` | holds up the century | 600 | 0.41 | 2641 of 2756 | 238 | `pop_1900`, `urban_share_1900`, `income_share_land_1900` (not kept without it: `real_wage_chained_1850_vs_1750`, `real_wage_chained_1900_vs_1750`) | 114% |
| `machines.enabled` | holds up the century | 1388 | 0.44 | 2861 of 2968 | 26 | `real_wage_chained_1900_vs_1750`, `towns_1900`, `urban_share_1900` | 101% |
| `works.enabled` | holds up the century | 1056 | 0.45 | 2881 of 2988 | 6 | `real_wage_chained_1850_vs_1750`, `real_wage_chained_1900_vs_1750`, `income_share_work_1900`, `income_share_land_1900` | 180% |
| `railways.enabled` | holds up the century | 49 | 0.40 | 2580 of 2977 | 17 | `towns_1900`, `rail_km_1900` | 114% |
| `carting.enabled` | holds up the century | 208 | 0.43 | 2784 of 2900 | 94 | `urban_share_1900` | 113% |
| `people.enabled` | holds up the century | 579 | 0.44 | 2834 of 2945 | 49 | `pop_1800`, `pop_1850`, `towns_1900`, `income_share_land_1900` | 99% |
| `mines.enabled` | holds up the century | 292 | 0.45 | 2879 of 2994 | 0 | `pop_1800`, `pop_1850`, `pop_1900` | 114% |
| `land.enabled` | holds up the century | 46 | 0.45 | 2878 of 2994 | 0 | `income_share_land_1900` | 97% |
| `prices.no_spoil_markup` | shapes the island (faint) | 1 | 0.45 | 2877 of 2993 | 1 | none | 97% |
| `harvest.enabled` | shapes the island | 5 | 0.44 | 2860 of 2986 | 8 | none | 108% |
| `works.count_spread` | shapes the island | 31 | 0.45 | 2876 of 2991 | 3 | none | 115% |
| `works.family_extends` | shapes the island | 9 | 0.43 | 2782 of 2992 | 2 | none | 116% |
| `railways.order_book` | shapes the island (faint) | 1 | 0.40 | 2584 of 2994 | 0 | none | 110% |
| `people.shelves_go` | shapes the island (faint) | 1 | 0.43 | 2787 of 2994 | 0 | none | 108% |
| `towns.enabled` | shapes the island | 38 | 0.45 | 2875 of 2994 | 0 | none | 109% |
| `towns.rent_floor` | shapes the island (faint) | 1 | 0.45 | 2873 of 2992 | 2 | none | 111% |
| `canals.enabled` | shapes the island (faint) | 2 | 0.42 | 2703 of 2911 | 83 | none | 94% |
| `canals.navvies` | shapes the island | 2 | 0.43 | 2780 of 2991 | 3 | none | 107% |
| `tree.seed_trades` | shapes the island | 14 | 0.45 | 2874 of 2993 | 1 | none | 112% |
| `tree.plant_waits` | shapes the island | 34 | 0.45 | 2873 of 2993 | 1 | none | 118% |
| `longrun.enabled` | shapes the island | 29 | 0.43 | 2751 of 2955 | 39 | none | 125% |
| `transition.enabled` | shapes the island | 593 | 0.43 | 2754 of 2956 | 38 | none | 129% |
| `owners.enabled` | shapes the island (faint) | 2 | 0.43 | 2781 of 2896 | 98 | none | 117% |
| `tasks.enabled` | shapes the island | 246 | 0.27 | 1747 of 2461 | 533 | none | 101% |
| `tasks.land` | shapes the island | 227 | 0.29 | 1879 of 2831 | 163 | none | 108% |
| `woods.enabled` | shapes the island | 376 | 0.36 | 2309 of 2991 | 3 | none | 108% |
| `harvest.rent_as_paid` | idle | 0 | 0.43 | 2785 of 2994 | 0 | none | 103% |
| `chronicle.enabled` | idle | 0 | 0.00 | 0 of 2994 | 0 | none | 103% |
| `works.spread_first` | idle | 0 | 0.45 | 2876 of 2994 | 0 | none | 114% |
| `works.keep_order` | idle | 0 | 0.45 | 2875 of 2990 | 4 | none | 105% |
| `towns.mean_hands` | idle | 0 | 0.45 | 2877 of 2994 | 0 | none | 110% |
| `towns.donor_wage` | idle | 0 | 0.43 | 2796 of 2990 | 4 | none | 112% |
| `accounts.enabled` | idle | 0 | 0.00 | 0 of 2310 | 684 | none (not kept without it: `income_share_work_1850`, `income_share_work_1900`, `income_share_land_1900`) | 107% |
| `accounts.works` | idle | 0 | 0.00 | 0 of 2994 | 0 | none | 114% |
| `gauges.enabled` | idle | 0 | 0.00 | 0 of 2669 | 325 | none | 123% |
| `commons.carry_debt` | idle | 0 | 0.00 | 0 of 2994 | 0 | none | 125% |
| `hands.ration` | idle | 0 | 0.45 | 2876 of 2993 | 1 | none | 130% |
| `money.true_total` | idle | 0 | 0.45 | 2876 of 2992 | 2 | none | 124% |
| `woods.record` | idle | 0 | 0.00 | 0 of 2956 | 38 | none | 110% |
| `crossings.enabled` | idle | 0 | 0.45 | 2877 of 2994 | 0 | none | 117% |

The time of a run is the processor's time on a run with the rule off, against the control's (474 s on the clock for a run of the control): under a hundred per cent, the rule costs that much to run. It is taken on a machine shared with other work, whose load changed through the table, so only a time far from a hundred per cent says much.

## What each rule that holds up the century holds up

- **`prices.enabled`**: `pop_1850` (OUTSIDE).
  Largest apart: `emigrants` 3.0 to 2,002 (85.2 spreads); `price_swing_cloth` 0.044 to 0.103 (6.8 spreads); `grain_price_year_after_harvest` 0.412 to -0.640 (6.8 spreads); `town_months_wage_moved_by_half` 0.000 to 0.002 (6.4 spreads); `farm_zigzag_beyond_weather` 1.26 to 3.66 (5.8 spreads).
- **`engel.enabled`**: `pop_1900` (OUTSIDE), `urban_share_1900` (moved), `income_share_land_1900` (OUTSIDE).
  Largest apart: `transition_F_2050` 0.063 to 0.467 (37.4 spreads); `timber_in_wages_2040` 0.224 to 5.15 (35.9 spreads); `transition_F_2040` 0.064 to 0.383 (26.7 spreads); `industry_share_2040` 0.659 to 0.425 (23.5 spreads); `transition_F_2030` 0.065 to 0.324 (20.2 spreads).
- **`machines.enabled`**: `real_wage_chained_1900_vs_1750` (moved), `towns_1900` (OUTSIDE), `urban_share_1900` (moved).
  Largest apart: `budget_food_2040` 0.086 to 0.302 (99.3 spreads); `price_clay_in_wages_2040` 0.019 to 0.283 (88.9 spreads); `transition_F_2050` 0.063 to 0.994 (86.3 spreads); `budget_food_2020` 0.085 to 0.288 (85.0 spreads); `transition_F_2040` 0.064 to 0.993 (77.8 spreads).
- **`works.enabled`**: `real_wage_chained_1850_vs_1750` (OUTSIDE), `real_wage_chained_1900_vs_1750` (OUTSIDE), `income_share_work_1900` (OUTSIDE), `income_share_land_1900` (moved).
  Largest apart: `cells_industry_1750` 1.8 to 236.2 (78.2 spreads); `works_hands_within_2_of_market_1950` 1.000 to 0.965 (41.3 spreads); `works_per_1000_people_2050` 1.60 to 7.85 (41.1 spreads); `works_per_1000_people_last` 1.60 to 7.85 (41.1 spreads); `works_founded` 7,967 to 38,075 (39.0 spreads).
- **`railways.enabled`**: `towns_1900` (OUTSIDE), `rail_km_1900` (moved).
  Largest apart: `canal_traffic_end_vs_peak` 0.01 to 0.41 (20.0 spreads); `canal_mton_km_2050` 4.73 to 196 (17.9 spreads); `rail_km_1980` 2,811 to 1,092 (10.9 spreads); `rail_km_1950` 2,789 to 1,073 (10.9 spreads); `rail_km_1970` 2,805 to 1,088 (10.6 spreads).
- **`carting.enabled`**: `urban_share_1900` (moved).
  Largest apart: `industry_share_2040` 0.659 to 0.723 (6.5 spreads); `urban_share_2030` 0.563 to 0.613 (4.8 spreads); `mine_share_1751_1770` 0.164 to 0.057 (4.7 spreads); `mine_share_2030` 0.167 to 0.098 (4.5 spreads); `urban_share_2050` 0.564 to 0.614 (4.2 spreads).
- **`people.enabled`**: `pop_1800` (moved), `pop_1850` (moved), `towns_1900` (OUTSIDE), `income_share_land_1900` (OUTSIDE).
  Largest apart: `timber_in_wages_2040` 0.224 to 6.98 (49.3 spreads); `town_growth_pct_p10` 0.10 to 1.10 (46.2 spreads); `budget_clothing_2050` 0.182 to 0.248 (36.2 spreads); `timber_in_wages_2020` 0.173 to 4.65 (33.9 spreads); `timber_in_wages_2050` 0.266 to 6.89 (22.6 spreads).
- **`mines.enabled`**: `pop_1800` (OUTSIDE), `pop_1850` (OUTSIDE), `pop_1900` (OUTSIDE).
  Largest apart: `area_mines_1750` 0.327 to 745 (5533.7 spreads); `area_mines_1800` 1.14 to 929 (2548.2 spreads); `workings_share_of_cell_1750` 0.002 to 1.639 (1736.6 spreads); `area_mines_1850` 2.49 to 1,016 (1468.7 spreads); `area_mines_2010` 5.71 to 978 (942.9 spreads).
- **`land.enabled`**: `income_share_land_1900` (OUTSIDE).
  Largest apart: `workings_claypit_1990` 244.1 to 160.6 (3.0 spreads); `area_pasture_1940` 2,085 to 2,222 (2.9 spreads); `workings_claypit_2000` 246.8 to 165.8 (2.8 spreads); `workings_claypit_1980` 240.6 to 155.5 (2.6 spreads); `workings_claypit_2040` 260.4 to 182.5 (2.5 spreads).

## The rules that shape the island

- **`prices.no_spoil_markup`**, 1 apart: `electricity_lost_1900` 0.075 to 0.169 (1.8 spreads).
- **`harvest.enabled`**, 5 apart: `output_ac1_grain` -0.714 to -0.281 (4.6 spreads); `price_ac1_grain` -0.596 to -0.030 (3.8 spreads); `price_swing_grain` 0.102 to 0.036 (2.1 spreads); `output_swing_grain` 0.100 to 0.024 (1.8 spreads); `farm_zigzag` 1.73 to 1.04 (0.8 spreads).
- **`works.count_spread`**, 31 apart: `works_founded` 7,967 to 13,601 (7.3 spreads); `works_slots_used` 8,425 to 14,106 (7.3 spreads); `works_closed` 6,452 to 11,727 (7.0 spreads); `works_closing_chance_age_10_14` 0.034 to 0.063 (4.7 spreads); `machines_years_2030` 0.072 to 0.093 (4.6 spreads).
- **`works.family_extends`**, 9 apart: `electricity_lost_1900` 0.075 to 0.331 (4.9 spreads); `works_founded` 7,967 to 10,437 (3.2 spreads); `works_slots_used` 8,425 to 10,895 (3.2 spreads); `works_closed` 6,452 to 8,790 (3.1 spreads); `works_closing_chance_age_25_49` 0.017 to 0.022 (2.6 spreads).
- **`railways.order_book`**, 1 apart: `rail_spent_over_expected` 0.78 to 1.41 (5.9 spreads).
- **`people.shelves_go`**, 1 apart: `books_goods_worst` 1.07e-10 to 9.62e-15 (0.3 spreads).
- **`towns.enabled`**, 38 apart: `urban_churn_10y` 0.001 to 0.074 (50.0 spreads); `books_income_worst` 4.83e-15 to 5.98e-14 (23.2 spreads); `cells_urban_1920` 436.6 to 151.1 (3.1 spreads); `cells_urban_1930` 464.6 to 161.1 (2.9 spreads); `cells_urban_1900` 359 to 128.4 (2.7 spreads).
- **`towns.rent_floor`**, 1 apart: `books_income_worst` 4.83e-15 to 9.71e-15 (2.1 spreads).
- **`canals.enabled`**, 2 apart: `electricity_lost_1900` 0.075 to 0.298 (4.2 spreads); `carting_share_of_output_1800` 0.072 to 0.081 (1.4 spreads).
- **`canals.navvies`**, 2 apart: `electricity_lost_1900` 0.075 to 0.364 (5.5 spreads); `canal_most_navvies` 1,166 to 0 (0.8 spreads).
- **`tree.seed_trades`**, 14 apart: `owners_works_hh_1750` 0.204 to 0.237 (4.2 spreads); `owners_works_lords_1750` 0.728 to 0.695 (3.9 spreads); `gauge_hand_hours_1750` 1.383 to 1.446 (2.9 spreads); `hours_haul_1750` 4,592 to 3,501 (2.6 spreads); `hours_tend_1750` 6,807 to 5,532 (2.0 spreads).
- **`tree.plant_waits`**, 34 apart: `machines_years_2030` 0.072 to 0.090 (4.0 spreads); `works_per_1000_people_2050` 1.60 to 2.09 (3.2 spreads); `works_per_1000_people_last` 1.60 to 2.09 (3.2 spreads); `works_utilisation_2050` 0.799 to 0.682 (3.1 spreads); `works_utilisation_last` 0.799 to 0.682 (3.1 spreads).
- **`longrun.enabled`**, 29 apart: `months_at_bound_oil` 0.000 to 0.407 (111.4 spreads); `months_at_bound_electricity` 0.038 to 0.325 (3.8 spreads); `months_at_bound_fertiliser` 0.000 to 0.379 (2.9 spreads); `gauge_labour_in_machinery_1940` 0.633 to 0.531 (1.9 spreads); `carters_between_towns_share_of_hands_2050` 0.061 to 0.038 (1.8 spreads).
- **`transition.enabled`**, 593 apart: `timber_in_wages_2040` 0.224 to 6.94 (49.0 spreads); `town_growth_pct_p10` 0.10 to 0.99 (41.1 spreads); `timber_in_wages_2020` 0.173 to 5.27 (38.6 spreads); `budget_clothing_2050` 0.182 to 0.252 (38.2 spreads); `timber_in_wages_2050` 0.266 to 6.91 (22.6 spreads).
- **`owners.enabled`**, 2 apart: `income_pay_share_1750` 0.937 to 0.885 (1.5 spreads); `income_pay_share_1900` 0.878 to 0.819 (1.2 spreads).
- **`tasks.enabled`**, 246 apart: `budget_food_2040` 0.086 to 0.097 (4.9 spreads); `budget_food_2020` 0.085 to 0.096 (4.4 spreads); `transition_F_2050` 0.063 to 0.107 (4.0 spreads); `transition_F_2040` 0.064 to 0.108 (3.7 spreads); `gauge_hand_hours_2020` 0.049 to 0.068 (3.6 spreads).
- **`tasks.land`**, 227 apart: `oil_to_machines_1950` 0.996 to 0.000 (31.2 spreads); `oil_to_machines_1960` 0.980 to 0.000 (29.2 spreads); `oil_to_machines_1970` 0.996 to 0.000 (20.9 spreads); `oil_to_machines_2020` 0.990 to 0.000 (18.0 spreads); `oil_to_machines_2030` 0.993 to 0.000 (16.8 spreads).
- **`woods.enabled`**, 376 apart: `timber_in_wages_2020` 0.173 to 4.53 (33.0 spreads); `timber_in_wages_2040` 0.224 to 4.68 (32.5 spreads); `timber_in_wages_1990` 0.196 to 3.30 (23.4 spreads); `timber_in_wages_2000` 0.217 to 3.77 (19.5 spreads); `timber_in_wages_2050` 0.266 to 5.76 (18.7 spreads).

## The idle rules, and the second pass

- **`harvest.rent_as_paid`**: it stirs the island (2785 of the 2994 facts both keep differ) and moves no fact clear of the spread.
- **`chronicle.enabled`**: no fact differs on any island.
- **`works.spread_first`**: it stirs the island (2876 of the 2994 facts both keep differ) and moves no fact clear of the spread.
- **`works.keep_order`**: it stirs the island (2875 of the 2990 facts both keep differ) and moves no fact clear of the spread.
- **`towns.mean_hands`**: it stirs the island (2877 of the 2994 facts both keep differ) and moves no fact clear of the spread.
- **`towns.donor_wage`**: it stirs the island (2796 of the 2990 facts both keep differ) and moves no fact clear of the spread.
- **`accounts.enabled`**: no fact both keep differs on any island, and it keeps 684 facts the island does not keep without it: it watches.
- **`accounts.works`**: no fact differs on any island.
- **`gauges.enabled`**: no fact both keep differs on any island, and it keeps 325 facts the island does not keep without it: it watches.
- **`commons.carry_debt`**: no fact differs on any island.
- **`hands.ration`**: it stirs the island (2876 of the 2993 facts both keep differ) and moves no fact clear of the spread.
- **`money.true_total`**: it stirs the island (2876 of the 2992 facts both keep differ) and moves no fact clear of the spread.
- **`woods.record`**: no fact both keep differs on any island, and it keeps 38 facts the island does not keep without it: it watches.
- **`crossings.enabled`**: it stirs the island (2877 of the 2994 facts both keep differ) and moves no fact clear of the spread.

**The second pass**: all 14 idle rules turned off together are the same island. That island is the same island with fewer rules.

The arms the second pass played, in order:

- 14 rules off: the same island
  (harvest.rent_as_paid, chronicle.enabled, works.spread_first, works.keep_order, towns.mean_hands, towns.donor_wage, accounts.enabled, accounts.works, gauges.enabled, commons.carry_debt, hands.ration, money.true_total, woods.record, crossings.enabled)

## The century after 1900

What the design asks of the years after 1900, as the standing check reports it (`docs/rules/README.md`, "The standing check"); reported, not counted in the kinds. Only the arms that differ from the control are listed.

| | per cent a year, 1900 to 1970 | 2050 over 1970 | 2050 less 1900 |
|---|---:|---:|---:|
| the control | 1.14% (met) | 1.13 (met) | -0.076 (met) |
| `prices.enabled` off | 1.18% (met) | 1.22 (met) | -0.075 (met) |
| `prices.no_spoil_markup` off | 1.09% (met) | 1.12 (met) | -0.070 (met) |
| `engel.enabled` off | - (-) | - (-) | -0.012 (met) |
| `machines.enabled` off | 0.26% (missed) | 0.86 (missed) | +0.080 (missed) |
| `harvest.enabled` off | 1.14% (met) | 1.10 (met) | -0.076 (met) |
| `harvest.rent_as_paid` off | 1.14% (met) | 1.11 (met) | -0.078 (met) |
| `works.enabled` off | 1.05% (met) | 1.02 (met) | -0.022 (met) |
| `works.count_spread` off | 1.18% (met) | 1.07 (met) | -0.080 (met) |
| `works.family_extends` off | 1.14% (met) | 1.09 (met) | -0.078 (met) |
| `works.spread_first` off | 1.17% (met) | 1.10 (met) | -0.075 (met) |
| `works.keep_order` off | 1.16% (met) | 1.13 (met) | -0.074 (met) |
| `railways.enabled` off | 1.13% (met) | 1.10 (met) | -0.080 (met) |
| `railways.order_book` off | 1.17% (met) | 1.09 (met) | -0.075 (met) |
| `carting.enabled` off | 1.14% (met) | 1.13 (met) | -0.083 (met) |
| `people.enabled` off | 0.96% (missed) | 0.53 (missed) | +0.004 (missed) |
| `people.shelves_go` off | 1.14% (met) | 1.10 (met) | -0.075 (met) |
| `towns.enabled` off | 1.17% (met) | 1.14 (met) | -0.073 (met) |
| `towns.mean_hands` off | 1.11% (met) | 1.10 (met) | -0.075 (met) |
| `towns.donor_wage` off | 1.10% (met) | 1.13 (met) | -0.075 (met) |
| `towns.rent_floor` off | 1.08% (met) | 1.10 (met) | -0.077 (met) |
| `mines.enabled` off | 1.27% (met) | 1.11 (met) | -0.075 (met) |
| `canals.enabled` off | 1.14% (met) | 1.09 (met) | -0.078 (met) |
| `canals.navvies` off | 1.11% (met) | 1.16 (met) | -0.075 (met) |
| `land.enabled` off | 1.07% (met) | 1.13 (met) | -0.083 (met) |
| `tree.seed_trades` off | 1.10% (met) | 1.12 (met) | -0.073 (met) |
| `tree.plant_waits` off | 1.07% (met) | 1.15 (met) | -0.070 (met) |
| `hands.ration` off | 1.17% (met) | 1.09 (met) | -0.073 (met) |
| `money.true_total` off | 1.19% (met) | 1.11 (met) | -0.080 (met) |
| `longrun.enabled` off | 1.11% (met) | 1.11 (met) | -0.071 (met) |
| `transition.enabled` off | 0.92% (missed) | 0.51 (missed) | +0.022 (missed) |
| `owners.enabled` off | 1.19% (met) | 1.11 (met) | -0.080 (met) |
| `tasks.enabled` off | 0.93% (missed) | 1.09 (met) | -0.041 (met) |
| `tasks.land` off | 1.00% (missed) | 1.08 (met) | -0.033 (met) |
| `woods.enabled` off | 0.96% (missed) | 0.81 (missed) | +0.016 (missed) |
| `crossings.enabled` off | 1.12% (met) | 1.14 (met) | -0.083 (met) |

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
