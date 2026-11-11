# dx-growth: why the island's history stops, and what would keep it going

Branch `night/dx-growth` in worktree `.claude/worktrees/wf_b38216ce-d6f-7` (this file is committed there too). Nothing on it is for merging.

## Answer in brief

History stops about 1800, not 1850. On 16 seeds the real wage rises 42% from 1750 to 1800, 5% from 1800 to 1850 and 8% from 1850 to the 1890s (chained index; on the model's own index 32%, -2%, +1%). Causes, ranked by how much of the stall each explains:

1. **Nothing raises productivity on the land after about 1800, and three quarters of all labour works the land.** Husbandry has spread by 1789 and tools are 90% saturated by 1839. Farm machinery (1850s to 1870s) lifts farms only. Quarries, pits, pasture and woods (43% of labour in 1900) get nothing, and their output per worker falls 21 to 37% between 1800 and 1900. Raising all land sectors' productivity 50% by 1900 raises the 1900 real wage 31%.
2. **Five times the people on the same land** costs the farm wage 0.12 to 0.14 log points every fifty years.
3. **Farm machinery arrives as a machinery famine.** Machinery's price rises up to twentyfold for 10 to 20 years, the pumps in the mines go short, and the real wage falls up to 25% (the 0.98 of 1875 in seed 7).
4. **The index.** `real_wage` uses the 1750 basket, heavy in firewood and homespun wool. A chained index reads 1.71 in 1900 where the model reports 1.36.
5. Not causes: the cap on industrial learning (raising `learn_cap` to 16: +4%, within noise), savings (1 to 2% of viable proposals fail for want of money), demand (works run at 82 to 85% of capacity, everyone is employed), food (grain's real price falls a third; opening the ports to foreign grain: +0.4%).
6. **Converters and steel rail mills are stories, not growth.** When hacked to take over all steel and rails, the real wage does not move, because steel has no use that is not already saturated.

Two hacks keep history going: farm machines bought only where they pay (no slump), and machines that keep improving with the converter steel made and spread to quarries and clay pits (1850 to 1900 growth 58% instead of 9%, positive in all 16 seeds; urban share 0.40 with an Engel curve).

## How measured

`crates/isostate-core/examples/probe_growth.rs` steps the simulation, one thread per seed (16 seeds in 40 s), and writes a row a year, every evaluation `firms::entry` made (instrumented behind `st.dbg_on`), what `people::found_towns` saw, and the towns. `examples/probe_tools/*.py` run the variants and make the tables. Hacks are parameters that are 0 by default; with all at 0, seed 7 reproduces the stored baseline (1.03, 1.38, 1.39, 0.98, 1.40; 19 towns; 986 km). Seeds: 3, 7, 11, 21, 5, 13, 17, 29, 1, 2, 4, 6, 8, 9, 10, 12 ("16 seeds"); the first 8 or 4 where stated. Real wages are on a chained Fisher index (`chain.py`) unless stated, decade means in the summary tables, ± one standard error over seeds.

## 1. Sources of growth and when each runs dry

Baseline, 16 seeds, 5-year means:

| | 1750 | 1775 | 1800 | 1825 | 1850 | 1875 | 1900 |
|---|---|---|---|---|---|---|---|
| real wage, model (1750 basket) | 1.03 | 1.07 | 1.36 | 1.36 | 1.34 | 1.26 | 1.36 |
| real wage, chained | 1.04 | 1.11 | 1.48 | 1.52 | 1.55 | 1.50 | 1.71 |
| income / subsistence cost (`y_ratio`) | 1.91 | 2.02 | 3.04 | 3.29 | 3.36 | 3.57 | 3.72 |

| source | exhausted (mean year, range) |
|---|---|
| improved husbandry spread to all towns | 1789 (1774 to 1802) |
| learning at `learn_cap`: crucible steel, coke iron, mills | 1785, 1789, 1796 |
| engine works at cap | 1809 (1794 to 1853) |
| last town founded | 1803 (1765 to 1852) |
| tools multiplier 90% of the way to its ceiling of 1.5 | 1839 (1820 to 1877) |
| railways: 90% of final km, town-to-town cost at its final level (22 to 4.4) | 1843 (1829 to 1870) |
| farm machinery spread | 1862 (1825 to 1881) |
| rolling mills at cap | 1868 (1853 to 1884) |
| converter (discovered 1858, 1833 to 1882), steel rails (1869; never in 1 of 16) | never produce: 0.3% and 0.1% of output |

The farm wage identity (wage = marginal product x farm-gate price), labour-weighted logs:

| change in log | 1750 to 1800 | 1800 to 1850 | 1850 to 1900 |
|---|---|---|---|
| farm productivity (tools, husbandry, machines) | +0.61 | +0.21 | +0.40 |
| land per worker (x 0.28) | -0.12 | -0.14 | -0.14 |
| farm-gate grain price / CPI | -0.16 | -0.10 | -0.23 |
| farm wage / CPI | +0.29 | -0.03 | +0.03 |

After 1850 farm machinery adds 0.40 but grain's price takes 0.23 back: the gain is confined to farms and the labour it frees goes into sectors with falling returns.

| output per worker-month | 1800 | 1900 | change | share of 1900 labour |
|---|---|---|---|---|
| farms | 13.1 | 18.2 | +39% | 31% |
| quarries | 7.7 | 6.1 | -21% | 17% |
| pasture | 3.3 | 2.5 | -25% | 10% |
| collieries | 19.7 | 13.8 | -30% | 9% |
| clay pits | 9.2 | 6.4 | -30% | 4% |
| woods | 4.4 | 2.8 | -37% | 3% |
| services | 0.76 | 0.96 | +26% | 17% |
| all other works | | | | 8% |

Carting falls from 8% of farm output's value (1800) to 4% (1850) and stays. Agglomeration adds 1 to 8% to works' productivity after 1850.

What binds, by counterfactual (1890s real wage; baseline 1.67 ± 0.02):

| change | 1890s real wage |
|---|---|
| `learn_cap` 16 | 1.73 ± 0.03 |
| `hurdle` 0.06 | 1.87 ± 0.03 (a level shift from 1800 on; 1900/1850 unchanged) |
| `growth_cap` 0 (133,000; inventions come later) | 2.61 ± 0.07 |
| all land sectors +50% by 1900 | 2.20 ± 0.02 (against 1.68) |
| one sector +50% (8 seeds) | farms +9%, quarries +8%, collieries +7%, pasture +4%, services +3%, the rest under 2% |
| ports open to the world from 1846 | 1.69 (trade 1.2% of output) |

**The slump.** `econ::sector_tfp` (econ.rs:153-160) charges `farm_mach_input` in full as soon as know-how passes 0.05, while the boost scales with know-how x fill. In seed 7 (invented 1860) machinery's real price goes from 63 to 1,450 and stays above 350 until 1884, `fill[machinery]` falls to 0.41, the collieries' multiplier falls from 5.2 to 3.6, the real wage from 1.33 to 0.99. Over 16 seeds the lowest 5-year mean in 1855 to 1885 is 1.31 ± 0.05 against 1.54 in 1850. Entry's sums promise engine works 270% a year after 1850 (4 seeds), yet machinery output takes 28 years to rise sixfold in seed 7; I did not trace why capacity is so slow.

## 2. Why the late methods are never built

Every town-year after discovery, 16 seeds. The condition that fails is the return, by a wide margin.

**Converter.** The best town's return has a median of -0.05 against the hurdle of 0.12 and clears it in 4% of years. Median sums in the best town: steel sells at 32; iron and other inputs 24, heat 0.5, labour 15; margin -5.7 a unit. The plant costs 1,017 per unit of monthly capacity (150 bricks at about 5, 2 machinery), so the hurdle needs a margin of 12. The best town would clear the hurdle in 7% of years at K0 instead of 0.4 K0, 5% with full know-how, 26% judged five years down its learning curve, 46% with all three. It starts at A0 = 0.5 times (0.5 + 0.5 x 0.31 know-how) against crucible steel at the cap of 4. Even at A = 4 it would make steel for about 11 plus 12 of capital charge against crucible's 25 plus 2 (my arithmetic from content.toml at median prices): the plant is too dear for the method ever to matter. Know-how reaches the towns within about 8 years. The market is 4,200 to 7,200 units a month against an entry plant of 60. The best town's savers can afford the plant in a third of years.

**Steel rail mill.** Median return -0.55. It uses 1.05 steel (32) where a rolling mill uses 1.1 iron (15), and a steel rail sells as an iron rail: inputs cost 44 against a rail price near 30. Nothing in the model makes a steel rail better.

Remedies tried (8 seeds, on top of the machinery fix), share of 1890s output:

| hack | converter steel | steel rails |
|---|---|---|
| entry looks 5 (10) years down the curve, builds at K0 (`entry_lookahead`) | 5% (8%); plants run at 2 to 13% of capacity for decades | 0 |
| successor inherits half the earlier method's A (`learn_spill` 0.5) | 18% | 0 |
| plant at 0.3 of its cost (`late_cap_scale`) | 51 ± 15% | 0 |
| lookahead 5 + cheap plant | 97% | 6% |
| spill + cheap plant | 89% | 8% |
| ... + discovery thresholds halved (`late_disc_scale`; converter 1832, rail mill 1849) | 99.5% | 25% |
| ... + a steel rail counts as 5 iron rails (`steel_rail_life`) | 99.5% | 99.6%; steel's price -34%, rails' -55% |

The real wage is 1.67 to 1.72 in all of them (1.68 without) and the railways stay at about 1,000 km. Entry sized to the market (`entry_boost`) lowered the 1800 wage from 1.49 to 1.36: rejected.

## 3. Population

Pinned. The cap binds when income is above 1.65 times subsistence cost; it runs from 1.9 to 3.7. Share of people in towns growing at exactly `growth_cap`: 79% (1750s), 93% (1760 to 1774), 99.2% (1775 to 1799), 99.9% (1800 to 1849), 99.7% (1850 to 1900); the rest are famine years. Hence 677,000 to 691,000 in every seed.

Rule tried in `people::growth` (people.rs:19): where a town is at the cap, g = `growth_cap` + 0.012 x ln(y / 3.0), at most `pop_cap`. With `pop_cap` 0.018 on the baseline: 0.5% a year to 1775, 0.9% to 1800, 1.2% to 1850, 1.3% after, 673,000 ± 12,000 in 1900 (5.0 times). Costs: inventions 5 to 10 years later (railway 1831 for 1825), since discovery runs on cumulative output; 1890s wage 1.60 against 1.68. With the growth package income reaches 7 and the population 795,000; `pop_cap` 0.013 gives 647,000 to 670,000. Any rule that still gives fivefold leaves the 1900 wage where it was: this is realism, not growth.

## 4. Towns

`T_max` is not binding (17 to 21 of 48). Founding needs a cell with 600 workers within 7x7 cells that is more than 8 cells from every town **and** more than `found_distance` = 10 in path cost from its market (people.rs:134). The path-cost test is what stops it: cells passing fall from 124 to 425 in the 1750s to under 50 by 1790 and to zero between 1790 and 1850 (seed 11 keeps 2 to 4, best score about 400). By the distance test alone 2,000 to 2,500 cells always qualify and the best has 2,000 workers (1750s) rising to 8,500 to 14,000 (1890s).

Hack: if the usual rule finds nothing, found where non-farm work (mines, pits, works) within 7x7 exceeds `found_industry` 1,500 x (population / 135,000), more than `found_cells` 6 cells from every town. 16 seeds: 24.1 towns in 1900 for 18.8; per seed 2.2 founded 1800 to 1824, 0.9 in 1825 to 1849, 0.75 in 1850 to 1874, 0.56 in 1875 to 1900; 54 to 78% on coalfields; 14,000 to 34,000 people each by 1900; 56% of the post-1850 ones on a railway. Threshold 2,500: one more town a seed; 4,000: none. Costs: the 1890s wage is 6% lower (1.58 ± 0.02 against 1.68 ± 0.02). With an unscaled threshold of 3,000, seed 10 collapsed in 1893: a town founded 6 cells from a rival on a railway got a catchment of 4 cells and 363 people, migrants poured in, its wage went from 0.02 to 500 and the money check failed (`examples/probe_town.rs`).

## 5. Prototypes (16 seeds)

| variant | wage 1850 | lowest 5 years 1855-85 | wage 1890s | 1890s / 1850 | urban 1900 | towns 1900 |
|---|---|---|---|---|---|---|
| baseline | 1.54 | 1.31 | 1.67 ± 0.02 | 1.09 | 0.24 | 18.8 |
| A: farm machines bought where they pay (`mach_adopt`) | 1.59 | 1.48 | 1.68 ± 0.02 | 1.06 | 0.24 | 18.8 |
| A + steel package (last row of section 2) | 1.62 | 1.52 | 1.72 ± 0.02 | 1.06 | 0.23 | 18.7 |
| A + machines improve with machinery built, and reach quarries | 1.61 | 1.54 | 2.39 ± 0.10 | 1.49 | 0.27 | 18.8 |
| A + steel + machines improve with converter steel, and reach quarries ("steelq") | 1.73 | 1.80 | 2.70 ± 0.06 | 1.58 | 0.27 | 18.8 |
| steelq + Engel | 1.75 | 1.81 | 2.74 ± 0.08 | 1.58 | 0.40 | 19.7 |
| steelq + Engel + population rule (cap 0.013) + late towns | 1.66 | 1.68 | 2.66 ± 0.05 | 1.61 | 0.40 | 25.1 |
| six families, baseline | 1.63 | 1.55 | 1.81 ± 0.02 | 1.12 | 0.26 | 19.6 |
| six families, full package | 1.81 | 1.80 | 2.86 ± 0.07 | 1.60 | 0.41 | 25.7 |

Steelq per seed: 1890s/1850 between 1.28 and 1.84. On the model's own index the full package goes from 1.41 (1850) to 2.11 (1900). With A, seeds 7 and 3 read 1.23 and 1.27 in 1875 instead of 0.98 and 0.93, and machinery's peak price falls from 1,153 to 444. Quality exponent 0.2 / 0.3 / 0.4 gives 1890s/1850 of 1.41 / 1.58 / 1.79; without the quarries 1.29.

## 6. Recommendations, ranked

All are new `Params` fields, 0 = off, skipped in params.json when 0 (as the family fields are, params.rs:140-189), switched on in one `game_rules()` used by `crates/isostate-viewer/src/live.rs:100-103`. Content changes are scales applied to the loaded `Content` in `sim::start_from` (sim.rs:514), never edits to data/content.toml.

1. **Machines bought where they pay** (`mach_adopt`). `econ::sector_tfp`: for farm machinery, share used x = clamp((gain - cost) / cost, 0, 1) with gain = grain price x boost / (1 + all boosts) and cost = `farm_mach_input` x machinery price; boost and input scale with x, input also with know-how. Farm machinery only: the same rule on pumps and wagonways cut the 1800 wage from 1.48 to 1.20 (4 seeds), because wagonways bought at a loss are what accumulates the rails that trigger rolling mills and railways. Worked if: machinery's price never passes about five times its 1850 level; the 1870s wage is not below 1850's.
2. **Machines keep improving and spread** (the growth fix). `sector_tfp`: quality q = max(1, Q / Q0)^b multiplies `pump_boost`, `wagon_boost` and `farm_mach_boost`; quarries and clay pits get `mech_quarry` x (q - 1) x fill, buying machinery at `pump_input`. Q = cumulative converter steel (`st.method_Q[bessemer]`; `mach_steel_b` 0.3, `mach_steel_Q0` 100,000; needs 3) or cumulative machinery (`st.cumprod`; b 0.6, Q0 150,000; no dependence on 3 but twice the spread between seeds, 1.81 to 3.24 in the 1890s). Risks: a second feedback loop through machinery demand (peak price 600 to 800); calibrate b on many seeds. Worked if: output per worker in quarries and collieries rises after 1860; 1890s/1850 above 1.3 in every seed.
3. **Converters and steel rails that pay** (the story fix; no growth alone). `learn_spill` 0.5 in `firms::update_productivity` (firms.rs:49), for learned methods after an earlier learned method of the same good, so only these two change; `late_cap_scale` 0.3 on their `m_cap`; `steel_rail_life` 5 dividing the steel per rail; `late_disc_scale` 0.5 to 0.75 on their `tech_disc` rows (0.75: converter 1846). Risks: the urban share falls 0.01 (converters need fewer hands). Worked if: "Converter steel overtakes crucible steel" and "Most new rails are steel" appear in every seed.
4. **Engel** (`engel` 1.0; `econ::les`, call at econ.rs:296): above y = 2 the marginal food share is divided by 1 + engel x ln(y / 2) and the difference goes to comfort. Urban share 0.24 to 0.29 / 0.32 / 0.35 for 0.5 / 1 / 2 alone, 0.40 with 2; no wage cost.
5. **Chained index** in `observer::record_month` (observer.rs:186-196), as new series beside the old (yearly Fisher links on `hh_got` + `lord_got`).
6. **Population rule** of section 3 (`pop_slope`, `pop_y_ref`, `pop_cap` 0.013 with 2).
7. **Late towns** of section 4, only with safeguards: refuse a site whose catchment by path cost would be under about 40 cells or 3,000 people, keep 8 cells' spacing, and move people with their cells whenever catchments change (`people::resettle_new_towns` does it only at founding, with a floor of 2%, people.rs:239). Ports and junctions were not tried.

Not worth building for growth: foreign grain, a higher `learn_cap`, entry sized to the market, promoters pooling savings across towns.

## Not checked

The viewer was not built or run. `cargo test --release -p isostate-core` passes on the hacked branch (33 tests; the 15 parity tests finish in 0.00 s, so they probably skip without the Python dumps). Only the landlords regime. Families: only the runs above with six default families; no scripted players. The foreign-trade hack is crude (2.75 ports a seed, fixed world prices). Why the railways stop at about 1,000 km was read from `rail::plan` (rail.rs:95-98: the saving is measured against a route that already goes by rail), not measured. Why engine-works capacity responds so slowly was not traced. The 6% wage cost of late towns is measured, not explained.
