# tr-money: the issue of money, the settings that fail the money check, and the Python's oddities

Branch `night/o-trace-money`, cut from `main` at 295a379 on 12 November 2026. One of four traces with the books of account (piece 12 of `docs/backlog/found-along-the-way.md`; findings K39, K40, R76, C18, R53 and C63 of `docs/backlog/sweep-2026-11-11/`). Nothing under `crates/isostate-core/src` was changed on the branch. The measuring code is `crates/isostate-core/examples/money_trace.rs`:

```bash
cargo run --release -p isostate-core --example money_trace -- [--seeds 1-8] [--rules game|port]
    [--families 6] [--end-year 1900] [--set JSON]... [--no-check] [--years] [--months FROM-TO]
```

It steps each island a month at a time and, around every month, reads the money before it (`econ::money_total`), the issue (from `State::issued`), and from the issue the total the month counted (the issue over `money_growth / 12`). Money only moves inside a month, so what the count left out is the money before the month less what was counted. The books (on in every run here; they change nothing, `docs/rules/accounts.md`) say what the households were paid in the year, by kind. With `--no-check` the monthly money check is off, and the example says in which month it would have stopped the island and why.

Everything is on seeds 1 to 8 under the game's rules with six families unless a line says otherwise; sixteen islands where eight were noisy. Raw output is under `.claude/night/reports/opus/trace-money/` (`raw/` for the example's tables, `sweep-*` for the sweeps). Two trial mends were tried in the worktree behind environment variables and not committed; section 1 and section 3 say what they were.

## In short

1. **Money grows 1.74 to 1.78 per cent a year under the game's rules (1.73 to 1.77 under the port) where `money_growth` says 2.** At the moment of counting, the part missing is the households' whole income for the month, not only the dividends: a local vector (`hh_inc` in `econ::step_month`) holds the month's wages, carting, navvies' and managers' pay and both kinds of dividend until after the count. It is 11.5 to 14.7 per cent of the money; pay is three quarters of it and dividends a tenth to a seventh.
2. **Issuing on the true total moves the price level and little else that is real.** Money in 1900 is 56 million where it was 36 to 38 (+46 per cent), prices and money wages stand a third higher, and what a wage buys does not move beyond the spread between islands (twenty-year means 0.98 to 1.05 of today's on sixteen islands). The one real road is the freight rate, which is a number of coins and does not rise with prices: carting is about a quarter cheaper in real terms by 1900, fewer carters are needed, and fewer towns are founded on the carting they would save (a tenth less canal too, which the freight explains only in part). The merchants' debts fall from 0.28 to 0.22 of a year's wage bill.
3. **None of the settings that fail the money check leaks money.** Each that still fails today takes the money below nothing first, through the same counting rule (a month whose households' income is larger than the money issues less than nothing), and then the check's allowance of a millionth of one coin meets the rounding of purses of thousands of millions. Two of the settings named on the rules' pages no longer fail at all, and with the true total issued the two that still fail pass every month to 1900.
4. **None of the eight oddities of the Python not yet gone through matters under the game's rules.** One is gone in the Rust under both sets of rules, five do not run under the game's (its own mechanisms bypass or mend them), the dead state is still dead apart from three arrays the chronicle, the towns and the ways' books now read, and the sister works' funding runs once in eight islands and moves 1,826 coins.
5. **Large harvest shocks slow the issue by the same rule**, and what makes more money miss the count there is that more of it is owed: the merchants' purses below nothing reach the whole of the money by 1900. With the true total issued, the money of 1900 is the same 56 million at any size of shock.
6. **Found on the way: under the game's rules the merchants owe half the money by 1900**, and it is the price rule that lets them (with `prices.enabled` off they owe 2 per cent of it).

## 1. The issue of money (K39)

### What was measured

```bash
cargo run --release -p isostate-core --example money_trace -- --seeds 1-8 --rules game --families 6 --years
cargo run --release -p isostate-core --example money_trace -- --seeds 1-8 --rules port --families 0
```

| | game's rules, six families | port, no families |
|---|---|---|
| money in January 1750 | 2.67 to 2.68 million | 2.66 to 2.67 million |
| money in December 1900 | 36.4 to 38.5 million | 35.6 to 37.7 million |
| growth a year, 1750 to 1900 | 1.744 to 1.781 per cent | 1.733 to 1.770 per cent (seed 7: 1.766) |
| left out of the count, as a share of the money | 0.115 (1750s) to 0.147 (1900), pooled by decade | 0.114 to 0.147 |

So PORTING.md's "about 1.76 per cent" for seed 7 stands, under the port and under the game's rules alike. The shortfall is the share left out: the growth a year is 2 per cent times one less that share, compounded monthly.

### Where the missing part is

`econ::step_month` builds the households' income for the month in a local vector, `hh_inc`: works' wages and their carting (line 742), the hired managers' cut (735), the land's wages and carting (759), the navvies (836), the carting of trade between towns (953). Households' spending has already been taken out of their cash (781). In "dividends, common fund, issuance" the works' dividends are taken out of the works' cash (families take theirs straight into their own purses, which are counted), the merchants' out of theirs, the fund's out of the fund; then `money_total` is taken and the issue reckoned on it; only then are the dividends and the issue added to `hh_inc`, and `hh_inc` to the households' purses. `money_total` counts `State::hh_income`, but that is always nothing at the moment of counting: it is written only after the count and set back to nothing at the month's end.

So everything households earn in the month is in flight at the count. The books confirm it to the last digit: in every year of every island, what the example finds left out equals what the books say households received less the issue (`IslandYear::to_households - issue`) to within 2e-13 of it. What it is made of, pooled over the eight islands:

| | 1750s | 1800s | 1850s | 1890s |
|---|---|---|---|---|
| left out, as a share of the money | 0.115 | 0.117 | 0.117 | 0.144 |
| of it: pay in works and on the land | 0.76 | 0.76 | 0.79 | 0.84 |
| carting | 0.11 | 0.10 | 0.07 | 0.06 |
| works' dividends to households | 0.08 | 0.07 | 0.08 | 0.05 |
| merchants' dividends | 0.05 | 0.07 | 0.07 | 0.06 |
| navvies and hired managers | 0.00 | 0.00 | 0.00 | 0.00 |

PORTING.md's account ("dividends in flight") names a tenth to a seventh of it.

Why the share rises after 1860 is a second thing. The count is of the money net: a purse below nothing counts against the rest. Under the game's rules purses go below nothing, by 1 per cent of the money in the 1750s and 47 per cent in 1900, and by then 98 per cent of what is owed is the merchants' (the works owe the rest). The households' month income keeps step with the money held above nothing (it is 0.114 of it in the 1750s and 0.100 in 1900), not with the net, so the more is owed the larger the share left out. With the price rule off (`--set '{"prices":{"enabled":false}}'`) the money owed is 2 per cent of the money in 1900, the share left out falls to 0.097 and the money grows 1.80 per cent a year. With the works' rules off instead, 11 per cent is owed and it grows 1.78 per cent a year. So the price rule is what lets the merchants run into debt, and the works' rules deepen it. Why the merchants spend more than they take in under the price rule I did not trace (`prices.rs` was another helper's ground tonight); the books' `merchants_kept` against `stock_value` would show it.

### What issuing on the true total would move

The trial mend was one line in `econ::step_month`, before the issue: add back to `m_tot` what is in flight (`pairwise_sum(&hh_inc)`, the works' and merchants' dividends and the fund's), behind an environment variable so the same build ran both ways. Taking `money_total` at the top of the month would do the same. Then:

```bash
ISOSTATE_TRY_TRUE_ISSUE=1 isostate-core sweep --seeds 1-8 --rules game --families 6 --out .../sweep-true-issue --compare .../merged-game-f6-l
isostate-core sweep --seeds 9-16 --rules game --families 6 --out .../sweep-base-9-16
ISOSTATE_TRY_TRUE_ISSUE=1 isostate-core sweep --seeds 9-16 ... --out .../sweep-true-issue-9-16 --compare .../sweep-base-9-16
```

The money grows exactly 2.018 per cent a year (2 per cent a year, a twelfth of it each month) on every island, and since nothing else moves it, every island has the same money every month: 2.74 million in January 1750 (the warm-up issues too) and 56.1 million in December 1900, against 36.4 to 38.5 million today.

Twenty-year means, the mended run over today's, the mean of the islands' ratios and [the least .. the greatest island]; the first line is seeds 1 to 8, the second 9 to 16:

| | 1750-69 | 1790-1809 | 1830-49 | 1870-89 | 1890-1900 |
|---|---|---|---|---|---|
| money | 1.05 / 1.05 | 1.15 / 1.15 | 1.27 / 1.27 | 1.40 / 1.40 | 1.46 / 1.46 |
| price index (`cpi`) | 1.01 / 1.02 | 1.13 / 1.05 | 1.22 / 1.19 | 1.33 [0.95 .. 1.76] / 1.27 | 1.29 / 1.34 [0.98 .. 1.61] |
| money wage | 1.04 / 1.06 | 1.16 / 1.14 | 1.30 / 1.24 | 1.34 / 1.35 | 1.31 / 1.40 |
| real wage | 1.00 / 1.01 | 0.99 / 1.05 | 1.03 / 1.02 | 0.98 [0.77 .. 1.11] / 1.04 | 0.99 [0.69 .. 1.12] / 1.02 [0.96 .. 1.09] |
| real wage, chained index | 1.00 / 1.01 | 0.98 / 1.07 | 1.04 / 1.04 | 1.00 / 1.04 | 0.98 / 1.02 |
| real output | 1.04 / 1.04 | 1.03 / 1.11 | 1.07 / 1.06 | 1.01 / 1.08 | 1.02 / 1.06 |
| people | 1.00 / 1.00 | 1.00 / 1.02 | 1.00 / 1.02 | 1.00 / 1.02 | 1.00 / 1.02 |

(`python .../tmp/opus/trace-money/annual_cmp.py NEW BASE` over each sweep's `annual.csv`; the script is not in the repository.) Prices rise by less than the money: nominal output is 1.31 to 1.42 times today's where the money is 1.46, so the money held per unit of spending is up a tenth.

What a wage buys does not move beyond the spread between islands, on either set of eight; the sweep's single-year facts agree (`real_wage_1900_vs_1750` 2.64 against 2.60 and 2.74 against 2.63, up on six and five islands of eight).

The debts: the money owed (the merchants', as above) is 0.22 of a year's wage bill in the 1890s with the mend against 0.28 today, and 0.30 of the money in 1900 against 0.47. In coins it is about the same (16.7 million against 17.5); a third more money and a third higher wages make it lighter.

Almost every fact of the sweep moves ("of the 446 facts both have, the seconds aside, 434 differ and 0 of those are starred" on seeds 1 to 8; 432 and 2 starred on 9 to 16), as any change does to these histories. The ones that move the same way on fifteen or sixteen islands of sixteen are the carters' share of hands and of output in every year, and small things that follow from the same cause (below); the town, canal and works counts move the same way on most islands:

| | seeds 1-8, today | mended | seeds 9-16, today | mended | 1-8 mended, freight x1.3 |
|---|---|---|---|---|---|
| carters' share of hands, 1800 | 0.118 | 0.107 | 0.119 | 0.114 | 0.125 |
| carters' share of hands, 1900 | 0.064 | 0.054 | 0.067 | 0.054 (every island lower) | 0.060 |
| carting's share of output, 1900 | 0.044 | 0.038 | 0.046 | 0.037 (every island lower) | 0.041 |
| towns in 1900 | 17.8 | 15.4 | 16.6 | 15.0 | 16.6 |
| towns founded late | 5.1 | 2.6 | 4.0 | 2.8 | 3.4 |
| canals in 1900, km | 892 | 802 | 915 | 846 | 871 |
| works in 1900 | 738 | 687 | 719 | 696 | 705 |

The cause is the freight rate (`Params::freight`, 0.015), a number of coins per unit of weight per cell, which no rule raises with prices. With prices and wages a third higher, a cartload costs about a quarter less in real terms: fewer carters, and less carting saved by a new town (`peopling.rs`, `found_saving` against the district's wages) or by a canal. The test was to raise the freight by the rise in prices, by hand: with the mend and `--set freight=0.0195` (1.3 times) each of these goes part of the way back in 1900 (of the mend's move, three quarters for the canals, three fifths for the carters' share of hands, half for carting's share of output and the towns, a third for late towns and works), and overshoots in 1750 (carters 0.143 against 0.125) where prices had not yet risen. A freight that followed the price level would be the clean test.

The mirror of it, run when this page was checked, does without the mend altogether: today's rules with the freight cut by the same 1.3 (`sweep --seeds 1-8 --rules game --families 6 --set freight=0.01154`, against today's facts). If the mend works through freight in real terms, this should land where the mend does, and for carriage and towns it does: carters' share of hands 0.104 in 1800, 0.069 in 1850 and 0.054 in 1900 (the mend 0.107, 0.070, 0.054), carting's share of output in 1900 0.038 (0.038), towns in 1900 15.9 (15.4), towns founded late 3.1 (2.6). It does not for the canals (879 km against the mend's 802 and today's 892) nor fully for the works (708 against 687 and 738), so for those two the freight is at most part of the road, and on eight islands they may be noise (2 and 3 islands of 8 move the other way under the mend). Being a cut from 1750 rather than a slow slide, it also lowers the carters of 1750 (0.107) and raises what a wage buys in 1900 against 1750 (2.78 against 2.60, up on seven islands of eight), where the mend leaves both.

The rest of what moves the same way on fifteen or sixteen islands is small and goes the same road. The warm-up issues more too, so prices in 1750 are a little higher and carting a little cheaper from the start: real income and goods a head in 1750 are up 0.5 to 0.8 per cent on every island, and the price of coal in wages down. And there are 3 to 5 per cent more quarry workings from 1800 (stone is the heaviest thing carted); with the freight raised 1.3 times they are 13 per cent fewer than today in 1800 instead.

### How sure

Sure of the size and the place: the books and the example agree to rounding on every year of sixteen islands, and the mend gives exactly 2 per cent. When the page was checked, one month was also added up by hand from a trial print at the moment of counting (seed 3, January 1799, not committed): the purses as counted come to 5,627,413.837844, what is in flight to 738,091.036636 (households' pay 526,098.18, carting 72,937.98, a hired manager 0.67, works' dividends 88,183.21, merchants' 50,871.00, no navvies and no fund), and the two together to the money at the top of the month, 6,365,504.874480, to the last printed digit. Seeds 23 and 41, which the page had not used, give 1.753 and 1.766 per cent a year and the same account (the left-out share rising from 0.121 to 0.161 of the money, and steady at a tenth of the purses above nothing). Fairly sure that the real economy is indifferent to the level of prices but for the freight: two sets of eight islands, twenty-year means, ranges that straddle 1. What would change my mind: a fact that moves the same way on all sixteen islands and has no constant in coins behind it. The other constants in coins I found are wide limits (the wage's ceiling of 500 in the wage search, the price floor and ceiling at 0.05 and 40 times base prices) that a third more on prices does not reach.

### For the pass over all the rules: the designer's decision

It is the port's rule, and the golden test holds the port, so any mend is a switch under the game's rules alone (`docs/rules/README.md`). What is to be chosen:

- **Leave it.** The game's money grows 1.72 to 1.78 per cent a year and the shortfall wanders with the money owed (to 1.58 under large harvest shocks). Nothing real depends on that except through the freight rate.
- **Issue on the true total, behind a switch.** One line. Prices and money wages a third higher by 1900, the price index of the game's charts with them; carting cheaper, fewer carters, two or three fewer towns and a tenth less canal by 1900 unless the freight moves with it. Every number a scorecard was read against moves by noise or more, so it should come before the pass over the rules or as its first step, not after.
- **Raise `money_growth` to taste instead.** It would not hold, since the share left out is not a constant (0.115 to 0.147 under the game's rules, to 0.21 under large shocks).

The larger question under it is the freight rate fixed in coins, which `docs/rules/carting.md` already leaves for a decision (checked-rules row 14): with the issue mended or not, a freight that follows the wage or the price level would make the island's carriage indifferent to how much money there is. If the issue is mended, I would mend that in the same step.

## 2. The settings that fail the money check (R76, C18)

How the example classifies a failure: in the month the check would stop the island it prints the money, the drift since the start against what the check allows (a millionth of the money, or of one coin when the money is below one), the month's own drift (the purses after less before less the issue), and that drift as a multiple of the rounding of one sum over the purses without their signs (2.2e-16 times their gross). A leak would show as an own drift many times that rounding.

| setting | its page | today | what happens |
|---|---|---|---|
| `machines`: `quality_b` 2, `quality_cap` 50, `quality_Q0` 0 (C18) | `machines.md` | fails on seeds 3 and 7 | below nothing, then rounding |
| `people`: `growth_top` 0.05, `y_still` 0.5, arrivals unlimited (R76) | `people.md` | holds on sixteen islands | |
| `towns`: `d_new` 0 or 1 | `towns.md` | holds on four and eight islands | (traced there: rounding on purses of 1e18) |
| `prices`: `markup` -0.5 | `prices.md` | fails on seed 1 of 4 without families, on seeds 1, 2 and 4 with six | below nothing, then rounding |
| `mines` under the commons, seed 3 | `mines.md` | mended by `mines::fund_to_share` | (traced there: below nothing, then rounding) |

**The machines (C18).** `--seeds 3,7 --no-check --set '{"machines":{"quality_b":2,"quality_cap":50,"quality_Q0":0}}'`, and month by month on seed 7 (`--seeds 7 --end-year 1767 --months 1764-1766`). Until February 1766 the island is ordinary: 3.56 million in money, 3.16 million counted, an issue of 5,300 a month, no month drifting by more than 1e-9. In March 1766 every machine becomes fifty times better (the first steel), the mines' wages run up, and the households' income for the month is some 56 million, sixteen times the money. The count is -53 million and the issue -89,000. The issue stays below nothing every month after (-140,000, -215,000, ... -1.2 million in September), the money falls with it, 3.48 million, 3.33, 3.12, 2.78, 2.22, 1.30, 0.07, and in October it is -0.98 million. By November the purses below nothing, most of them the landlords', owe 3.1 thousand million, and the purses without their signs add up to 6.2 thousand million. That month's own drift is -1.45e-6, 1.1 times one rounding of the gross; the allowance, with the money below one coin, is 1e-6; the check stops the island. Seed 3 the same in 1785 (money below nothing in August, stopped in September; own drift 0.6 roundings of 1.1e10). Not a leak: rounding, after the money went below nothing by the port's issue.

**The people (R76).** `--seeds 1-8 --no-check --set '{"people":{"growth_top":0.05,"y_still":0.5,"arrivals_max":100}}'`, and seeds 9 to 16: the check holds every month on all sixteen. A sweep of the same (`--params` with those three numbers) gives 9.9 million people in 1850 and 13.1 million in 1900 (12.1 to 15.3), more than the six to eight million on the page, and none of the eight fails. The share left out of the count peaks at 0.19 of the money in the 1850s, far from the 1 at which the issue would turn below nothing. So it cannot be traced on today's rules. The page's guess (the port's issue going below nothing when a month's incomes outgrow the money) is the road the machines and the prices take, and I would expect it was the same; I did not check the rules of that night out to see.

**The towns.** `--set '{"towns":{"d_new":0}}'` on seeds 1 to 4 with six families, and `d_new` 1 on seeds 1 to 8 without: the check holds every month on all twelve. The page's tracing (rounding on purses of 1e18, one month purse by purse) stands as the account of what happened then; I did not look at whether towns still take the whole island.

**The negative markup.** `--seeds 1-4 --families 0 --no-check --set '{"prices":{"markup":-0.5}}'`: seed 1 stops in October 1879; its money has been below nothing since August 1837, the issue below nothing in 1,349 of its 1,558 months, and the month's own drift is 0.2 roundings of purses of 1.0 thousand million. The drift since the start crept past the allowance after 42 years below nothing. As `prices.md` says: below nothing, then rounding. Seeds 2 to 4 hold. With six families (`--families 6`, a check run after the page was first written) three of the four fail, seed 1 in July 1838, seed 2 in September 1854 and seed 4 in January 1824, each after its money went below nothing (in 1836, 1854 and 1820) and with a month's own drift of 0.2 to 0.7 roundings of purses of 650 to 760 million: the same road.

**What it means.** One road, two halves. The first half is the counting rule of section 1: a month whose households' income is larger than the money issues less than nothing, and enough such months take the money below nothing, where it stays (the issue is then a share of a negative number). The second is the check, whose allowance is a millionth of the money, or of one coin once the money is below one, while the rounding of the purses grows with their gross. With the mend of section 1 the money cannot go below nothing (it only grows), so the first half closes. Tried: with the true total issued, the machines' setting on seeds 3 and 7 and the negative markup on seeds 1 to 4, with six families and without, pass the check every month to 1900 (the machines' island is still wrecked, its landlords owing twenty thousand times the money, but the money is there and the purses add up to it). The check would still cry wolf on purses of 1e18 against a money of millions, as the towns' edge once did. An allowance of a few roundings of the purses' gross would make the check say what it means (C12; the check is another helper's ground tonight).

## 3. The Python's oddities under the game's rules (K40)

PORTING.md lists ten; items 2 (section 1) and 3 (the spurs, which a game never builds) were gone through before. The other eight, by reading the code and, where it still runs, by counting:

| | under the game's rules | does it matter there |
|---|---|---|
| 1. ties sorted in a machine's own order | gone in the Rust under both sets of rules: `util::argsort` sorts (key, index) pairs, so ties fall to the lower index on every machine | no |
| 4. entry capital doubled, a limit that cannot matter | does not run: `works::sized_entry` (on in the game) keeps the limit at one and skips the doubling (`firms.rs`, entry) | no |
| 5. a footprint reserved only until the year's end | mended: `works::land_held` counts capacity on order as well as built, so the rebuild of `cell_K_share` keeps the ground of a works being built; `works::ordered` takes the ground of growth at once | no |
| 6. a sister works' money sent to the households when it cannot be raised | still runs for works no family owns (a family's goes back to the family) | once in eight islands: see below |
| 7. a spur topped up twice from one town | does not run: the game builds no port spurs (`rail::plan_spurs` returns under `railways.enabled`), and the game's own call (`ways.rs`) skips the second town when both ends are one | no |
| 8. a new town's share of outflow copied, the donor's kept | does not run: under `people` households go with their cells (`peopling::move_with_cells`), which moves the outflow from the old town to the new | no |
| 9. a donor of -1 | does not run: the game founds towns by `peopling::found_towns`, whose candidates are cells with a market (`catch >= 0`) | no |
| 10. state written and never read | still written; `migration_flow` is now read by the chronicle, `bids` by the towns' rent floor, and `traffic` by the railways' and canals' books (`ways::note_year`: the year's fares, which the sweep reports as fares over capital, read by nothing that decides); the rest (`f_out_acc`, `month_stats.share`, `ag_labor`, `cell_firms`, `m_merch_margin`, `m_trade_value`, `cumprod_1750`, `State::subcost`, `firm_at`) is unread | memory only |

**The sister works' funding (6), counted.** A trial line in `firms::spawn_sister` printed every sister works and every failure (not committed). On seeds 1 to 8 with six families, a sister works was funded 2,174 times and the failure path ran once: a family works between 1800 and 1849, whose 1,826 coins went to its family's purse along with the 1,882 the family had put in. A works no family owns never failed. Under the port without families, 25,286 sister works and no failure. It does not matter under either set of rules; mending it (the parent's part back to the parent) would move nothing measurable, so I did not sweep it.

## 4. Large harvest shocks (R53, C63)

```bash
cargo run --release -p isostate-core --example money_trace -- --seeds 1-8 --set '{"harvest":{"island_sd":0.4}}'
cargo run --release -p isostate-core --example money_trace -- --seeds 1-8 --set '{"harvest":{"island_sd":0.3,"foreseen":1}}'
```

| | game's size (0.05) | 0.4 | 0.3, all foreseen | 0.4, true total issued |
|---|---|---|---|---|
| money in 1900, millions | 36.4 to 38.5 | 26.5 to 35.3 | 32.0 to 36.2 | 56.1 on every island |
| growth a year | 1.74 to 1.78 per cent | 1.53 to 1.72 | 1.66 to 1.74 | 2.018 |
| left out, share of the money, 1750s / 1890s | 0.115 / 0.144 | 0.120 / 0.211 | 0.117 / 0.194 | 0 |
| left out, share of the money above nothing | 0.114 / 0.101 | 0.116 / 0.106 | 0.114 / 0.106 | |
| money owed, share of the money, 1900 | 0.47 | 0.99 | 0.90 | 0.75 |

(On seeds 23 and 41, run when the page was checked: shocks of 0.4 give 1.70 and 1.65 per cent a year, money owed 1.03 times the money in 1900, and with the true total issued 56.1 million on both.)

It is the same thing as section 1. The issue is smaller because more of the money is outside the count when it is counted, and with the true total issued the money is 56.1 million in 1900 at any size of shock: nothing else slows it. What puts more of it outside the count is the money owed. The households' income is the same tenth or so of the money held above nothing at every size of shock; what grows with the shocks is the merchants' debt (99 per cent of all that is owed), which the net count subtracts. Why bigger shocks put the merchants deeper in debt I did not trace; a good harvest bought into store with cash the merchants do not have is the likely road. (The landlords who paid wages no crop covered, which `harvest.md` describes at `foreseen` 2, are no longer in it: `foreseen` is held between 0 and 1.)

## What it means for the pass over all the rules

- **`money_growth` and the issue (`econ::step_month`, "dividends, common fund, issuance").** A decision for the designer, set out at the end of section 1. Either way the scorecard should not read the price level or money wages against a target of 2 per cent.
- **`freight` (`Params::freight`).** The one number found through which the level of prices reaches the real island: carters, carting's share and town foundings, shown both ways (freight raised with the mend, cut without it); the canals and the works count only in part, and another road there is not ruled out. It belongs with the decision on the issue, and with checked-rules row 14.
- **The merchants' debts under the price rule.** Half the money by 1900, all of it at large shocks. Nothing in the economy minds, but it bends the count of section 1 and it is an odd thing for merchants to do for a century. Worth a trace of its own with the books' `merchants_kept` and `stock_value` (it is not in piece 12's list).
- **The money check's allowance** (C12): a few roundings of the purses' gross, not a millionth of the net, would stop the false alarms.
- **Nothing in K40 needs doing under the game's rules.** The PORTING.md list could say, item by item, which run under the port only, with the one-line reasons above.

## Not done

- R76 could not be reproduced on today's rules; the rules of the night it failed were not checked out to see.
- Why the merchants run into debt under the price rule, and deeper under large shocks: measured, not traced.
- The freight test used a constant 1.3 times; a freight that follows the price level was not built.
- The towns' extreme settings were run only to see whether the check holds; whether towns still take the island there was not looked at.
