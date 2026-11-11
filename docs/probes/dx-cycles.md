# dx-cycles: where the price swings and the farm zigzag come from, and what damps them

Branch `night/dx-cycles` in the worktree `.claude/worktrees/wf_b38216ce-d6f-10` (measuring code only, not for merging). Probe: `crates/isostate-core/examples/probe_cycles.rs`. Analysis scripts: `tools/dx-cycles/*.py`. Every switch lives in a bag `Params.hx` that is empty by default; with it empty, seed 7 reproduces `.claude/night/baseline/s7` (worst relative difference 5e-6, the CSV rounding) and `cargo test -p isostate-core --release` passes (33 tests).

## Answer in brief

Both problems are one mechanism, and it is in the price rule, not in the capacity or land decisions. `econ::step_month` (econ.rs 889-911) moves a price by `price_speed` times the inventory gap, so the price is the running sum of past stock gaps, while the stock is the running sum of past surpluses. Two sums in a loop make an oscillator, and the only friction is the small `price_flow` term. The period comes out near 8 years for goods with inelastic supply and demand (iron, rails, machinery, steel, coal, and real output with them) and 3.2 years for grain, where farm labour is elastic but follows the price half a year late. Land shares have nothing to do with the zigzag: freezing them changes nothing.

The cure is to let merchants mark the price up and down with the stock they hold (price = slow anchor x exp(k x gap), k = 0.5). It removes the farm zigzag, cuts the swings of the iron price by about 40 per cent and of iron output by two thirds (rails prices hardly improve), removes the phantom slumps, raises output and real wages by 5 to 10 per cent, and leaves the response to discoveries, railways and a capacity shock as fast as before. Seeing the capacity pipeline, investing on averaged prices, investing more slowly and changing the land clocks do not cure it.

## 1. What was measured

Runs of 150 years without families on seeds 3, 7, 11, 21, 5, 13 (1, 2, 9, 17 as a check on the final set); diagnostics and single cures on 3, 7, 11, 21. Per series and half-century: `sd_y`, the sd of yearly log changes of annual means; `amp`, the sd of the log deviation from a centred 10-year moving average; the dominant period of that deviation (periodogram, 0.5 to 12 years); `ac1`, the lag-1 autocorrelation of yearly changes. Levels are decade means.

A warning for anyone comparing variants: `gdp_real` and `gdp_pc` are in each run's own 1750 money, and anything that changes the warm-up moves the 1750 price level (seed 7: 1750 wage 4.9 unmodified, 3.0 with `price_flow` 0.3, 9.1 with `price_target_months` 4). Compare real wages, physical outputs or growth ratios.

## 2. Size and period, unmodified rules

Mean of 6 seeds; 1750-99 / 1800-49 / 1850-1900.

| series | sd_y | amp | period, years | ac1 | monthly sd |
|---|---|---|---|---|---|
| price iron | .187 / .200 / .214 | .27 / .29 / .31 | 8.6 / 11.1 / 8.6 | .64 / .71 / .75 | .017-.019 |
| price rails | .162 / .172 / .204 | .20 / .24 / .29 | 8.6 / 8.8 / 9.9 | .55 / .68 / .76 | .017-.019 |
| price machinery | .182 / .209 / .259 | .25 / .27 / .33 | 7.8 / 7.5 / 8.4 | .51 / .58 / .74 | .021-.026 |
| price steel | .220 / .121 / .137 | .26 / .15 / .17 | 7.9 / 7.5 / 8.6 | .55 / .28 / .25 | .013-.022 |
| price cloth | .096 / .070 / .083 | .08 / .06 / .08 | 3.7 / 4.9 / 6.5 | .02 / .07 / .19 | .008-.010 |
| price grain | .063 / .070 / .101 | .06 / .06 / .09 | 4.8 / 3.2 / 3.2 | .06 / -.30 / -.26 | .007-.012 |
| output grain | .081 / .102 / .178 | .06 / .08 / .14 | 3.2 / 3.2 / 3.2 | -.25 / -.36 / -.50 | .013-.033 |
| farm labour share | .040 / .063 / .109 | .03 / .05 / .08 | 3.2 / 3.2 / 3.2 | -.35 / -.39 / -.44 | .006-.015 |
| real output | .030 / .035 / .052 | .04 / .04 / .07 | 7.8 / 7.2 / 8.1 | .09 / .10 / .15 | |

Worst: iron, rails and machinery, all century, with a period of 8 to 10 years (not "a few") and about 3x from trough to peak (iron 5 to 17 against a base price of 6). The farm zigzag has the same 3.2-year period in every seed and nearly triples in size by 1850-1900. The iron cycle is national: in seed 7, 1800-49, the 19 towns have sd_y .15 to .23 and 14 of them follow the national price at a correlation above .9. Rails prices are wilder (.27 to .39) in towns that neither make nor use rails.

## 3. Mechanism

**The loop.** Let x be the log price above its balance and I the stock, in months of sales, above target. The rule is dx/dt = -(speed/target) I - (flow/2) dI/dt, and the market gives dI/dt = e x, with e the sum of the supply and demand elasticities. That is a spring with period 2 pi / sqrt(speed e / target): 6.8 years for e = 0.4 (iron), 2.6 years for e = 2.8 (grain), with damping ratio flow sqrt(e target / speed) / 4 = 0.10 for iron. If supply answers the price L months late, the loop gains energy once L exceeds about flow x target / (2 x speed) = 2.7 months. Farm labour lags 6 to 9 months (`pexp` at `expect_rate` 0.3, then `labor_adjust_ex` 0.15); capacity lags one to two years.

**One iron cycle, seed 7** (yearly means; stock in months of sales; "ordered" is the capacity ordered that year by entrants and growing works):

| year | price | output | requests | stock | capacity | ordered | utilisation |
|---|---|---|---|---|---|---|---|
| 1812 | 16.7 | 3031 | 2891 | 1.6 | 3307 | 529 | .95 |
| 1814 | 12.0 | 3662 | 3283 | 4.7 | 3892 | 62 | .94 |
| 1816 | 6.5 | 3775 | 3528 | 6.7 | 3803 | 17 | .99 |
| 1817 | 5.0 | 3581 | 4227 | 5.7 | 3695 | 0 | .98 |
| 1818 | 4.9 | 3317 | 4585 | 2.3 | 3552 | 0 | .95 |
| 1820 | 8.4 | 2426 | 3756 | 1.0 | 3315 | 205 | .89 |
| 1821 | 12.2 | 2427 | 3742 | 1.0 | 3334 | 475 | .81 |
| 1822 | 15.6 | 3232 | 3779 | 1.1 | 3589 | 233 | .88 |

At 16 the trade over-orders (14 entries and 305 of expansion in 1812). Output passes requests in mid-1812, yet the price needs five and a half years to fall, at the rule's limit of about 2.5 per cent a month, while works run at 93 to 99 per cent and merchants pay for all of it (econ.rs 660): the stock reaches 7 months. At 5, buyers ask for 4,600 against 3,400 made, the stock is burnt in 18 months, and then only two thirds of requests are filled (1820-21) while the price climbs back at 2 to 3 per cent a month and ordering restarts. No works closes in 1812-22. Capacity decisions feed the swing but do not make it (section 4).

**The farm zigzag, seed 7, 1803-09.** Grain price .596 to .660 (peaks 1805-01 and 1808-01); farm labour 36,100 -> 42,200 -> 38,400 -> 44,600, peaking six months after the price; stock 0.72 to 1.28 million (1.4 to 2.2 months), peaking a year after the price. Farm land stays between 1,336 and 1,352 cells although farm bids swing 15 per cent from year to year.

**Diagnostics** (4 seeds; farm labour share sd_y by half-century; iron price sd_y 1800-49):

| change | farm share sd_y | iron |
|---|---|---|
| none | .042 / .059 / .109 | .205 |
| land shares frozen from 1750 | .042 / .059 / .102 | .188 |
| `land_adjust` 0.5 | .062 / .073 / .141 | .216 |
| land bids on a 3-year average of prices and wages | .046 / .051 / .102 | .186 |
| `price_speed` 0.06 | .103 / .144 / .201 | .376 |
| `price_speed` 0.015 | .014 / .019 / .022 | .175 |
| `price_flow` 0.3 | .015 / .011 / .031 | .143 |
| `price_target_months` 4 | .025 / .037 / .067 | .188 |
| `labor_adjust_ex` 0.5 | .014 / .019 / .033 | .180 |
| `expect_rate` 1 | .015 / .020 / .042 | .193 |
| `labor_adjust_ex` 0.05 | .052 / .056 / .095 | .318 |

Every price and labour row moves as the loop predicts; the land rows do nothing. The growth of the zigzag late in the century has a second cause, the machinery famine (section 7).

## 4. Cures tested

sd_y for 1800-49; real wage of the 1840s as a percentage of the unmodified run on the same seeds. First block 4 seeds, second block 6.

| cure | iron price | iron output | farm share | real wage | verdict |
|---|---|---|---|---|---|
| none | .205 | .158 | .059 | 100 | |
| pipeline: entrants and growing works expect the price times (market x 1.05 / (market + on order))^eta, eta 1; 2 | .198; .176 | .164; .116 | .059; .062 | 103; 100 | no cure alone |
| investment on a moving average of prices (rate .5; .33) | .192; .175 | .157; .127 | .062; .062 | 102; 105 | no cure |
| slower investment (growth capped at 15 per cent a year, no double-size entry) | .153 | .104 | .061 | 89, and 62 in the 1890s | kills growth |
| `price_flow` 0.15; 0.3 | .191; .143 | .098; .073 | .022; .011 | 93; 93 | zigzag gone, but outputs -6 to -24 per cent, 11 to 16 per cent of iron requests unmet, locomotive 11 to 12 years late |
| merchants' stock cap alone (3 months) | .111 | .146 | .067 | 104 | partial, industry only |
| **stock markup**, k = .3; .5; .6; .8 | .155; .108; .133; .157 | .067; .047; .054; .061 | .017; .010; .009; .009 | 101; 106; 104; 106 | the cure |
| k .5 + stock cap 4 | .144 | .072 | .011 | 111 | |
| k .5 + cap 4 + harvests .05 | .129 | .050 | .023 | 109 | recommended |
| k .5 + cap 4 + pipeline eta .5; 1 | .123; .121 | .048; .043 | | 106; 102 | small gain |

Raising `price_flow` fails because the flow term compares local requests with arrivals and ignores shipments to other towns, so an exporting town reads as a permanent glut (section 7).

**Response to real change** (6 seeds; unmodified -> recommended). Rails output after the locomotive x2.1 -> x2.0, peaking after 11.2 -> 11.0 years; railway km 10 and 25 years on, 376 -> 385 and 982 -> 1,033; coke iron reaches half of output in 8.3 -> 7.7 years, mills in 12.2 -> 10.2, rolling mills in 18.0 -> 20.7; iron output doubles after coke in 15.8 -> 18.2 years. The median discovery years over 10 seeds do not move (locomotive 1823 -> 1823; single seeds move up to 20 years either way). The pipeline at eta 2 slows rolling mills to 35 years.

**Shock test**: 30 per cent of iron capacity destroyed at the end of 1815, against the same seed unshocked (6 seeds, mean log ratio).

- Unmodified: price +.11, +.37, +.77, +1.11, +1.13 in years 1 to 5; capacity overshoots by .23; the price then undershoots to -.70 in year 12 and is back near zero in year 18. Real output -.13 in year 4, recovered in year 8.
- Cure (k .5, cap 4): price +.21, +.56, +.90, +.99, +.81; capacity overshoots by .16; undershoot -.47 in year 10, within .1 of zero by year 15. Real output -.10 in year 3, recovered in year 6. Iron output is back to par in year 5 or 6 in both.
- Adding the pipeline cuts the capacity overshoot only to .12 (eta 1) and .10 (eta 2).
- With 30 per cent of capacity added instead, the cure's price falls sooner (-.69 in year 3, against -.67 in year 5) and returns without overshooting.

## 5. Recommendation, in order

**1. Stock markup in the price rule.** New parameter `price_stock_k` (0 = off; game 0.5; 0.5 to 0.8 measure alike, 0.3 is weaker). New state `price_anchor` (T x G). In `econ::step_month`, where the price is set: move the anchor by the old rule (`exp(clamp(price_speed x gap + price_flow x imbalance, +-0.12))`, clamped to floor and ceiling), then move the price toward `anchor x exp(k x gap)` by at most 12 per cent a month. Set the anchor to the price in `sim::init_before_firms`, and copy the donor's anchor along with its price in `people::found_towns` (people.rs 176); without that copy a new town starts from a stale anchor. With k = 0 take the old branch so the bytes stay the same. Hack: `stockP` in econ.rs.

Measured on 6 seeds, k .5 alone: iron price sd_y .187/.200/.214 -> .118/.108/.154; iron output .174/.154/.176 -> .071/.047/.064; farm share .040/.063/.109 -> .009/.010/.021; real output .030/.035/.052 -> .013/.007/.020. Unmet iron requests 10 -> 2 per cent, the worst year's fill .53 -> .90, utilisation .88 -> .95, stock 3.0 -> 1.9 months. Real wage +6, iron +2 to +8, cloth +9 to +19 per cent.

**2. Merchants stop buying into a glut.** `merchant_stock_months` (0 = off; game 4): a works' utilisation is multiplied by a factor that falls from 1 at 4 months of the town's stock of its good (over the smoothed outflow `st.outflow`) to 0.2 at 8 months. It belongs inside `firm_demand` (econ.rs 420) so the wage sees it, and in the Newton slope. Hack: `stock_cap`. On top of k .5 it trims late iron and rails prices (sd_y 1850-1900 .154 -> .132 and .207 -> .167) and removes what is left of output bought with no sale in sight: the share of output made where the town's stock is over 6 months of requests and shipments is 6.2 / 9.7 / 13 per cent for iron / rails / machinery unmodified, 0.3 / 4.2 / 4.0 with k alone, 0.3 / 0.6 / 1.0 with the cap. This is the "cap what merchants pay" point in the DEVLOG. Not tested: the `stock_thin` variant for towns with no outflow at all.

**3. Harvests.** With the false cycle gone grain is too flat (price sd_y .02) and downturns are rare. `harvest_sd` (0 = off; game 0.05): one island-wide log-normal factor a year on farm output, applied after labour is hired, from a stream of its own (the hack hashes seed and year in `sim::yearly`, so no other stream moves). It gives grain price sd_y .066 and grain output .06, with ac1 near -.5 as independent shocks should; the same shocks in the unmodified rules ring (grain output sd_y .135 / .146 / .241).

**4. Optional: the pipeline at eta 0.5 to 1** (`pipe` in firms.rs: `Pipeline`, used in `entry`, `review` and `spawn_sister`). Iron output sd_y .072 -> .048 and little else. Do not use 2.

**Not worth doing**: land clocks or smoothed land bids, investment on averaged prices, slower investment, a higher `price_flow`.

**How to tell it worked** (means of 6 seeds, `probe_cycles`, then `tools/dx-cycles/agg.py`): farm share sd_y under .035 in every half-century with harvests (under .025 without); iron price sd_y under .18 and iron output under .08; iron stock near 2 months and unmet iron requests under 4 per cent; coke iron past half of output within 10 years; median locomotive within 5 years of 1823.

**Risks.** Pacing and balance shift a little: levels +5 to +10 per cent, 7 per cent fewer firms founded. With six AI families and nobody answering decisions (6 seeds) the player's rank is 3 3 3 4 1 1 before and 1 4 4 4 1 1 after; the families' shares of works, rent and money are 57/25/10 per cent before and 61/25/11 after; family works that close live 21 years instead of 16. A price can now stand up to 65 per cent above its anchor when a town's stock runs out. Not tried: the scripted players in `bots.rs`, the commons regime, the viewer's Markets panel.

## 6. What volatility remains, and the observer

Recommended set (k .5, cap 4, harvests .05) against the unmodified rules, 6 seeds. The 4 fresh seeds agree: iron .107/.127/.155, farm share .019/.021/.030, alarms 16.8 -> 6.2.

| | unmodified | recommended |
|---|---|---|
| price iron sd_y | .187 / .200 / .214 | .113 / .129 / .162 |
| price rails | .162 / .172 / .204 | .139 / .164 / .184 |
| price machinery | .182 / .209 / .259 | .166 / .130 / .196 |
| price steel | .220 / .121 / .137 | .197 / .049 / .050 |
| price cloth | .096 / .070 / .083 | .048 / .029 / .033 |
| price grain | .063 / .070 / .101 | .067 / .066 / .065 |
| output grain | .081 / .102 / .178 | .057 / .068 / .068 |
| farm labour share | .040 / .063 / .109 | .018 / .023 / .031 |
| real output | .030 / .035 / .052 | .021 / .019 / .020 |
| steep events per run (boom, glut, shortage, slump) | 44.8 (10.7, 6.5, 13.3, 14.3) | 29.5 (5.0, 9.5, 7.8, 7.2) |
| alarms; verdicts; of them depressions | 15.2; 15.0; 5.2 | 5.7; 5.5; 0.5 |
| years real output falls over 2 / 4 per cent | 21.0 / 11.3 | 6.2 / 1.3 |
| years the iron price moves over 20 per cent | 51.8 | 24.3 |

What remains is a cycle of 7 to 10 years in iron, rails and machinery with amp .15 to .25 (1.5 to 2x from trough to peak): the capacity cycle proper, plus lumpy railway orders. Rails prices hardly improve. The observer still calls two booms, gluts, shortages or slumps a decade, and a downturn every 26 years instead of every 10. Without harvests it would call 1.8 downturns a run.

## 7. Other findings

**The flow term is blind to exports** (econ.rs 899-901). Counting shipments to other towns as wanted (`flow_exports`), with k .5 and cap 4, raises the real wage by 11 / 21 / 21 per cent and coal by 28 to 48 per cent, drops the farm share of the 1840s from .51 to .47, and removes nearly every downturn (0.3 alarms a run). But stocks pile up in thin markets (rails 18 months at the 95th centile), and in the one family test (4 seeds, a set that also had harvests and technique choice) the families' share of money went from 10 to 66 per cent, against 10 for the same set without the flow fix. A real distortion worth fixing, but a rebalancing job; not in the set.

**The machinery famine.** Once farm machinery is found (1834 to 1866 in the 6 seeds), every farm buys it per unit of output whatever it costs (`sector_tfp`, econ.rs 155-161). Machinery requests rise 4 to 5x, its price goes from about 70 to between 870 and 1,940 within 11 to 15 years, and the real wage falls 20 to 37 per cent and stays below its old level for 17 to 23 years, in every seed. With the cure it is 8 to 28 per cent and 7 to 21 years. This is what makes the zigzag grow late in the century, and it is probably the late-century stall. A quick test of adoption that follows what the machine pays (`tech_choice`, a logit .03 of the output price wide, on top of the recommended set) holds the peak price to 260-580 and the wage fall to 2 to 14 per cent. It is untuned (in seed 5 farm machinery then arrives only in 1890) and wants its own pass, including why machinery capacity grows only 9 per cent a year (seed 3) at twenty times its usual price.

## 8. Not checked

The Python reference; the viewer; years after 1900; more than 10 seeds; why the flow fix raises output so much; how `price_stock_k` interacts with `price_target_months`. No significance tests were run: by eye the changes in the farm share and in iron output are several times the spread between seeds, those in iron and machinery prices about twice, and those in rails prices within it.
