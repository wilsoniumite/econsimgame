# tr-cycles: the building cycle nobody chose, the famine of tools, and what swings

Branch `night/o-trace-cycles`, a trace with the books of account (piece 12 of `docs/backlog/found-along-the-way.md`, rows R111, R112 and C27 of the sweep's tables). Three questions: the building cycle that `README.md` ("Where it stands") and `docs/rules/works.md` describe, as the rules stand now; the famine of tools; and which goods swing most from year to year, and why.

Everything is under the game's rules with six families, at `main` a6ff0ec (12 November 2026), on islands 1 to 8 and, where eight were noisy or the answer mattered, 1 to 16. The measuring program is `crates/isostate-core/examples/trace_cycles.rs`. It plays each island a month at a time; after December it looks at every works of four trades just as the yearly review is about to judge them, and after the review it reads what each trade in each town ordered against its year's room (`State::works_added` against `works::year_budget`), which the books do not keep. Everything else comes from the books (`docs/rules/accounts.md`) and the observer's yearly means. No rule and no fact was changed: the rules under `crates/isostate-core/src` are as on `main`, and the sweep of islands 1 to 8 against today's main says "of the 446 facts both have, the seconds aside, 0 differ".

The cures were tried in the worktree behind switches read from the environment, never committed; section 5 describes each in a few lines. With every switch off that build gave all 332 of the program's numbers exactly as the clean build did.

A check before merging ran the program again (all 376 numbers of the sixteen islands came out the same), ran the smaller room on eight islands the page had not used (17 to 24), wrote the builders' queue again from section 5's words (it gives the same numbers), and tried three things the page had not: the woods' hands and the clay pits' hands made faster each alone, and households' tools in calm years of the same decades as the famines. Where those moved an answer, the text below says so.

## In short

1. **The building cycle is there, a little smaller than the README says, and it is two loops, not one.** Timber's price swings 0.20 a year after 1800 (the standard deviation of the yearly change in its log), with a period of about four years; in annual means it never doubles, but its dearest month of the century is 2.6 times the median month. The mills take their whole year's room in a quarter of town-years and next to nothing in two fifths, every 3.3 years, and up to nine towns in ten do it in the same year. What a wage buys swings 0.063 a year and falls 15 per cent in its worst year (sixteen islands).
2. **The orders carry the timber, and the timber carries the wage.** Plant built without timber (a counterfactual, section 2.4) takes three quarters off timber's swing (0.203 to 0.049 on eight islands, 0.202 to 0.051 on eight more, every island), a third off the real wage's swing and two fifths off its worst year, and raises the wage of the 1880s and 1890s by 5 to 7 per cent. The mills order exactly as lumpily as before. So the mills' rhythm does not come from timber's price; it comes from the review, where returns sit on the hurdle and every works over it orders at least a fifth more.
3. **The size of the lumps matters most; the land's slow hands matter too.** A year's room of 0.10 for 0.15, or a builders' queue, takes 27 to 31 per cent off timber's swing. Letting every land sector's hands follow the wage three times as fast takes 28 per cent off, but the woods' hands alone only 17 per cent, and the woods' and the clay pits' together 22 per cent. The woods' output follows the price of the year before (correlation 0.83) and not this year's (0.18), and with faster hands it follows this year's; that lag is about a sixth of timber's swing.
4. **The plain cures each take a little off and none ends it.** Orders spread evenly over the year: 15 per cent off timber's swing. Room in proportion to what is short: 3 per cent off, and the wage of the 1880s 15 per cent lower. A builders' queue: 31 per cent off, and the wage of the 1830s and 1840s 4 per cent lower. A growth step in proportion to how far the return clears the hurdle: 12 per cent off. The cheapest that helps is a smaller room: on twenty-four islands 0.10 lowers timber's swing and the wage's swing on every island and raises the worst year on 19 of 24; the wage and output of the 1880s and 1890s are as often lower as higher (the wage lower on 12 of 24, its mean 1 per cent higher).
5. **The famine of tools is a famine of the engine works' plant, held long by the smithies' room.** Households had left tools for iron and steel long before (from 1860 they buy next to none in calm years either), so in a famine year the engine works take nearly half the tools as material, works being built (engine works above all, which are built of 20 tools a unit) take nearly half again, a quarter of what is asked for is not got, and the smithies take 88 per cent of their room, run at 0.88 and earn eight times the hurdle on their plant. They grow 9 per cent a year: the room's 15 less 4 of wear. Three times the smithies' room cuts the years above three medians from 8.1 to 2.7 and raises the late wage on 15 islands of 16; half the engine works' room does as much for the famine and costs the wage of the 1830s and 1840s on 15 of 16.
6. **The famine no longer comes "when farm machinery arrives".** Its dearest year is more than thirty years after farm machinery on 11 islands of 16, 13 to 19 years after on three and before it on two, most often in the 1880s and 1890s; the dearest year over the median year is 5.2 on sixteen islands, between 1.8 and 10.6.
7. **What swings most falls in three families and two strays.** The iron family (tools, machinery, iron, rails, steel, coal, ore) swings with a period of eight to ten years: a cycle of orders for plant and for railways. The building family (timber, bricks, stone, and clay in part) swings with a period of about four years: the building cycle. Grain and wool swing with the weather. Clay's sawtooth is a ring between slow hands on the pits and the merchants' markup, which faster hands on the pits alone remove (0.178 to 0.054 on eight islands; 0.044 with every land sector's hands faster). Charcoal is a dead trade with a price nobody pays.

## 1. What was measured, and how

```bash
cargo build --release -p isostate-core
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --out OUT/game --compare MERGED-GAME-F6 --quiet
cargo run --release -p isostate-core --example trace_cycles -- --seeds 1-8 --detail 2 --json OUT/base.json > OUT/base.txt
cargo run --release -p isostate-core --example trace_cycles -- --seeds 1-16 --jobs 16 --json OUT/base16.json > OUT/base16.txt
cargo run --release -p isostate-core --example trace_cycles -- --seeds 1-16 --jobs 16 --set works.year_room=0.10 --json OUT/room10-16.json
cargo run --release -p isostate-core --example trace_cycles -- --seeds 1-8 --set KEY=VALUE --compare OUT/base.json   # each setting in section 5
```

`--detail SEED` adds that island year by year; `--compare FILE` sets every number beside another run's, with a star where the two ranges across islands do not overlap. The outputs are in `.claude/night/reports/opus/trace-cycles/`, and the trial switches as a patch in `.claude/night/tmp/opus/trace-cycles/trial-switches.diff`.

The numbers, all over 1800 to 1900 unless they say otherwise:

- **swing**: the standard deviation of the yearly change in the log of a yearly mean (the sweep's `price_swing_*`). **amp**: the standard deviation of the log about its own centred nine-year mean. **period**: the period between 2 and 12 years at which that deviation has most power. **ac1**: the correlation of one year's change with the next's; near -0.5 is independent shocks to a level, well below is a sawtooth, above zero is a slow wave.
- **dearest**: the dearest year over the median year, from 1780 (works.md's measure of the famine of tools). It mixes the trend in with the swing: steel and cloth, whose prices fall all century, read high on it for that reason alone.
- **room taken**: what a trade in a town ordered in the year over its room. **Whole** years are 0.9 or more, **bare** years 0.1 or less, counted only where the trade has capacity or has just ordered some.
- **at the review**: the share of a trade's built capacity that is free to grow (a return on its plant over the hurdle of 0.12, running at over 0.65, not building), still building, under the hurdle, or slack, with the return reckoned as `firms::review` reckons it: the year's profit over the capacity at this year's prices of the plant.
- **ordered timber**: capacity ordered times the timber in a unit of the method's plant (`m_cap`): what the order will ask the market for over the following months.
- **real wage** is the observer's (`real_wage`, 1750 = 1); levels are twenty-year means, since a single year's wage misleads.

## 2. The building cycle

### 2.1 What it is now

Sixteen islands (`base16.txt`); mean, then the least and greatest island.

| | |
|---|---|
| timber's swing | 0.203 [0.150 .. 0.265] |
| its period | 4.1 years [3.3 .. 6.5] |
| ac1 of its changes | -0.32 |
| dearest year over the median of the nine about it | 1.47 at most [1.20 .. 1.69] |
| dearest month of the century over the median month | 2.57 [2.01 .. 3.68] |
| months over twice the median | 22 in a century [2 .. 66] |
| a year's dearest month over its cheapest | 1.40 on average, 2.18 at worst |
| timber asked for and not got | 0.10 on average, 0.64 in the worst year |
| what a wage buys: swing | 0.063 [0.051 .. 0.084] |
| its worst year | -15.3 per cent [-10.3 .. -19.7] |
| the cost of living: swing | 0.064 |
| the wage's yearly change against timber's | correlation -0.69 |

"Timber trebles for a year every three or four" (`works.md`) was read from December prices on one island. It is true of a dear December against a cheap one; a year's mean doubles over its neighbours on no island, and the dearest month in a century is two and a half times the usual.

The mills (`textile_mill`), town by town:

| | |
|---|---|
| town-years they take their whole room | 0.26 [0.20 .. 0.42] |
| town-years they take next to nothing | 0.39 [0.26 .. 0.46] |
| years between whole years in a town | 3.3 [2.3 .. 4.2] |
| period of the island's mills' room taken | 3.7 years |
| how far two towns' mills move together | correlation 0.40 [0.24 .. 0.64] |
| the most towns with mills taking it whole in one year | 0.88 of them [0.69 .. 1.00] |
| mills' room taken against their return on plant that year | correlation 0.75 |
| against timber's price that year (about its trend) | -0.33 |
| against timber's change the year after | 0.49 |

The brickworks do the same (whole 0.23, bare 0.44), and so do the engine works (0.45 and 0.36) and the smithies (0.31 and 0.40). Lumpy ordering is how every trade under the life cycle orders; the mills and the brickworks are the ones whose plant is timber.

### 2.2 What it is made of

- **Who orders the timber.** Of the timber ordered for plant, brickworks 54 per cent and mills 46 per cent; nobody else orders any to speak of. A brickworks is plant of 10 timber and 10 stone a unit; a mill 10 timber, 25 bricks and 0.3 machinery. The brickworks follow the mills a year behind (they are asked for bricks), so each wave of mills calls up a second wave of timber: the share of their room the island's brickworks take correlates 0.65 [0.44 .. 0.84] with what the mills took the year before and 0.08 with the same year (`brick_room_vs_mill_*`, sixteen islands).
- **How big a wave is.** A year's orders are 4.6 months of the island's timber sales on average and 18 at most. Works being built buy 37 per cent of all timber over the century and 94 per cent in the worst year.
- **Who gives way.** Households (with landlords and families) buy 65 per cent of the timber, for warmth and shelter, and what they buy swings 0.78 a year: they are the buyer who gives way, by turning to coal and stone at a price. That is how a building wave reaches the cost of living.
- **Where it comes from.** The woods' output follows the price of the year before (correlation 0.83 about trend) and hardly this year's (0.18). Woodland cleared at the year's end is 5 per cent of timber made and does not matter here. When the woods' hands alone follow the wage three times as fast (a trial switch of the check, `labor_adjust_ex` 0.5 for the woods' cells only), their output follows this year's price (0.66) more than last year's (0.35), and timber's swing falls from 0.203 to 0.169 on eight islands of eight: the lag is real and is about a sixth of the swing.

Seed 2, the mills and timber at each year's end (from `--detail 2`; "free" is the share of the mills' capacity free to grow at the review, "plant" the price of a unit of a mill's plant over its price at the content's base prices):

| year | timber | real wage | mills' room taken | towns whole | free | return | plant |
|---|---|---|---|---|---|---|---|
| 1869 | 2.26 | 2.78 | 0.28 | 3/17 | 0.31 | 0.11 | 1.16 |
| 1870 | 1.68 | 2.99 | 0.92 | 11/17 | 0.78 | 0.15 | 1.06 |
| 1871 | 2.52 | 2.65 | 0.03 | 2/17 | 0.04 | 0.09 | 1.42 |
| 1872 | 2.08 | 2.74 | 0.06 | 2/17 | 0.07 | 0.08 | 1.25 |
| 1873 | 1.67 | 2.85 | 0.23 | 2/17 | 0.31 | 0.11 | 1.16 |
| 1874 | 1.79 | 2.81 | 0.15 | 2/17 | 0.26 | 0.10 | 1.19 |
| 1875 | 2.07 | 2.85 | 0.85 | 9/17 | 0.74 | 0.14 | 1.25 |
| 1876 | 2.85 | 2.69 | 0.28 | 4/17 | 0.47 | 0.12 | 1.48 |
| 1877 | 2.91 | 2.74 | 0.07 | 3/17 | 0.08 | 0.08 | 1.35 |
| 1878 | 1.73 | 3.15 | 0.82 | 9/17 | 0.69 | 0.15 | 1.12 |

The mills' return on their plant moves between 0.08 and 0.15 about a hurdle of 0.12, so a small move in their margin or in the price of their plant decides whether every mill on the island orders or none does.

### 2.3 Why the mills order in lumps

`firms::review`, once a year, lets a works grow when its return on plant is over the hurdle, it runs at over 0.65 and it is not still building. It then asks for `K * min(0.5, 0.2 * roi / hurdle)`: at least a fifth of its capacity, and more the better it did. The year's room is 15 per cent of the town's monthly trade, so a trade whose works clear the hurdle takes its room whole, and one whose works do not takes nothing. The mills run at 0.94 and are never still building at the review (0.3 per cent of their capacity), so the only thing that decides is the return, and the return sits on the hurdle. Cloth has one price across the island (it is traded between towns), so the towns cross the hurdle together. The capacity arrives within months, the margin narrows, every mill falls under the hurdle at once, and the market grows back into the capacity in two or three years. The brickworks are asked for bricks by the wave and follow it.

What is proven and what is not: that the orders, not timber's price, set the rhythm is proven by the counterfactual below (the mills order as before with timber's price flat). That the knife edge at the hurdle makes the lumps is shown by the growth step in proportion (section 5), which halves the bare years. That the narrowing of cloth's margin is what turns the mills down, rather than the wage or the plant's price, is my reading of the correlations and of seed 2; I did not switch it off.

### 2.4 The test: plant built without timber

A trial switch (`TRC_NOTIMBER`) strikes timber off every order for plant as it is placed; the plant is built of the rest. This is not a cure (plant needs timber) but it says what the orders do to timber and to the wage.

| | as it stands | plant without timber | islands |
|---|---|---|---|
| timber's swing, islands 1-8 | 0.203 | 0.049 | 8 of 8 down |
| the same, islands 9-16 | 0.202 | 0.051 | 8 of 8 down |
| timber's period | 3.9 | 7.4 | |
| dearest month over the median month | 2.62 | 1.51 | |
| real wage's swing, islands 1-8 / 9-16 | 0.064 / 0.062 | 0.042 / 0.044 | 16 of 16 down |
| its worst year, 1-8 / 9-16 | -14.3 / -16.2 per cent | -8.6 / -10.4 per cent | 16 of 16 better |
| real wage 1880-99, 1-8 / 9-16 | 2.68 / 2.77 | 2.82 / 2.95 | 9-16: 8 of 8 up |
| mills: whole years / bare years / years between | 0.25 / 0.40 / 3.3 | 0.26 / 0.40 / 3.3 | |

Three quarters of timber's swing and a third of the wage's are the building cycle. The higher wage is partly the cycle's cost and partly free timber for the plant; the two cannot be told apart this way.

## 3. The famine of tools

### 3.1 What it is now

Sixteen islands. The dearest year for tools since 1780 is 5.2 times the median year [1.8 .. 10.6], with 8.1 years in a run above three medians [0 .. 18]; machinery's dearest year is 5.1 times its median. Farm machinery is found between 1827 and 1850 (1838 on average); the dearest year for tools comes between 1782 and 1899, 1871 on average. On 11 islands of 16 it is more than thirty years after farm machinery, on three 13 to 19 years after, and on two (3 and 8) before it:

| island | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| dearest over median | 6.0 | 4.4 | 1.8 | 6.9 | 10.4 | 5.2 | 3.1 | 6.4 | 3.8 | 4.9 | 5.8 | 3.8 | 10.6 | 3.8 | 2.8 | 3.6 |
| dearest year | 1897 | 1898 | 1782 | 1858 | 1853 | 1867 | 1872 | 1825 | 1880 | 1893 | 1887 | 1899 | 1857 | 1897 | 1891 | 1880 |
| farm machinery | 1850 | 1841 | 1827 | 1839 | 1839 | 1832 | 1831 | 1840 | 1845 | 1844 | 1835 | 1827 | 1844 | 1833 | 1841 | 1832 |
| years over three medians | 5 | 7 | 0 | 18 | 13 | 13 | 2 | 16 | 3 | 9 | 16 | 7 | 15 | 4 | 0 | 2 |

### 3.2 Who goes short

Famine years (tools over three medians) against calm years (under one and a half), means over the islands that have them:

| | famine | calm |
|---|---|---|
| tools asked for and not got | 0.24 | 0.05 |
| bought by households, landlords and families, share | 0.02 | 0.48 |
| the same, a head a year | 0.000 | 0.056 |
| bought by households, landlords and families, share, from 1860 | 0.000 | 0.064 (0.021 without island 3) |
| the same, a head a year, from 1860 | 0.000 | 0.005 (0.000 without island 3) |
| used by engine works as material, share | 0.45 | 0.32 |
| bought by works being built, share | 0.46 | 0.14 |
| machinery asked for and not got | 0.09 | 0.07 |

The households' column is not an answer to the famine's price. Implements can be met with iron, steel or tools, and households had moved to iron and steel long before: seed 2's households bought 33,600 tools in 1791 and none by the 1880s, and from 1860 on, in calm years as in famine years, households buy next to no tools on every island that has a famine: 0.005 tools a head a year in calm years from 1860 against 0.056 in calm years over the whole run, and 0.0004 leaving out island 3, which has no famine and where they still buy (the table's two rows from 1860; its calm years over the whole run are mostly early in the century, when households still bought tools). So nobody's household goes short; the trade has shrunk to what the engine works want. Then a wave of engine works asks for its plant, 20 tools a unit of capacity (`data/content.toml`), on a trade sized to the little that was being sold: in seed 2 tools bought for plant go from a few hundred a year to 5,000 to 12,000 in 1872-75, 1881-86 and 1893-98. The engine works take their whole room in 45 per cent of town-years. The engine works themselves go a quarter short, and machinery a little.

### 3.3 Why the makers do not answer, and what would let them

In famine years the smithies take 88 per cent of their year's room [0.68 .. 1.03], run at 0.88, earn 8.2 times the hurdle on their plant [4.0 .. 17], and their capacity grows 9 per cent a year. In calm years they take 28 per cent, run at 0.63, earn 0.28 and shrink 2 per cent a year. So they answer as fast as the room lets them: 15 per cent of their trade a year, less 4 per cent of wear, while their return says they should grow many times faster. The room is a share of what the town sold, and what a short trade sells is what it makes.

Two trial switches, sixteen islands each, against the same islands as they stand:

| | as it stands | smithies' room x3 | engine works' room x0.5 |
|---|---|---|---|
| tools: years over three medians | 8.1 | 2.7 (13 down, 2 up) | 2.6 (14 down, 1 up) |
| tools: dearest over median | 5.2 | 5.2 (8 down, 8 up) | 3.6 (13 down) |
| tools' swing | 0.240 | 0.185 (13 down) | 0.143 (15 down) |
| real wage 1830-49 | 1.86 | 1.92 (11 up) | 1.77 (15 down) |
| real wage 1880-99 | 2.73 | 2.86 (15 up) | 2.68 (9 down) |
| output per head 1880-99 | 8.38 | 8.81 (15 up) | 8.34 (9 down) |
| works in 1900 | 729 | 692 | 677 |

A wider room for the smithies does not stop the first spike (the dearest year is as dear), because the wave of engine works arrives in one year; it shortens the famine to a third and raises the late wage and output by 5 per cent on fifteen islands of sixteen. A narrower room for the engine works makes the spikes smaller and shorter as well, by starving the machinery trade, and the wage of the 1830s and 1840s pays for it. The room counted on what buyers asked for rather than what they bought (`TRC_ASKED`) did nothing (5.52 for 5.54): when a trade is short, what is asked is only a quarter more than what is sold.

A year's room of 0.10 for every trade (section 5) also cuts the famine years from 8.1 to 2.2 on sixteen islands (14 down, 1 up), through the demand side: the engine works order smaller lumps. On eight more islands (17 to 24) it does less: 9.5 to 5.8, 5 down and 3 up, so over twenty-four 8.6 to 3.4 (19 down, 4 up).

## 4. What swings most

Sixteen islands, 1800-1900, in order of the swing. "Capital" is the share of what is bought that goes into plant, railways, canals, farms and mines; "with timber" and "with iron" the correlation of the yearly change in price with timber's and iron's; "towns" how many towns make at least a twentieth of it.

| good | swing | amp | period | ac1 | with harvest | capital | with timber | with iron | towns | what it is |
|---|---|---|---|---|---|---|---|---|---|---|
| tools | 0.240 | 0.270 | 9.6 | 0.44 | 0.02 | 0.27 | -0.05 | 0.41 | 6.4 | orders: the engine works' plant (section 3) |
| machinery | 0.213 | 0.229 | 10.2 | 0.34 | 0.01 | 0.98 | 0.23 | 0.36 | 7.4 | orders: farms' and works' plant |
| timber | 0.203 | 0.122 | 4.1 | -0.32 | 0.03 | 0.35 | 1 | -0.04 | 8.9 | orders: the building cycle (section 2) |
| iron ore | 0.196 | 0.148 | 7.5 | -0.37 | -0.01 | 0 | 0.07 | 0.27 | 5.9 | a thin market under the iron cycle |
| iron | 0.194 | 0.265 | 8.6 | 0.65 | -0.03 | 0 | -0.04 | 1 | 6.3 | orders: the furnaces' capacity cycle |
| rails | 0.193 | 0.240 | 8.7 | 0.55 | -0.02 | 1.00 | -0.03 | 0.54 | 4.8 | orders: the railways |
| clay | 0.186 | 0.099 | 3.1 | -0.86 | 0.04 | 0.00 | 0.33 | -0.05 | 7.7 | a ring on the pits (below) |
| charcoal | 0.142 | 0.128 | 8.0 | 0.00 | 0.01 | 0 | -0.11 | 0.17 | 0.2 | a dead trade |
| grain | 0.120 | 0.070 | 4.0 | -0.62 | -0.47 | 0 | -0.05 | 0.05 | 8.4 | weather |
| coal | 0.120 | 0.101 | 7.9 | -0.08 | 0.02 | 0 | -0.03 | 0.27 | 5.4 | the iron cycle, at one remove |
| bricks | 0.103 | 0.065 | 4.5 | -0.41 | 0.07 | 0.27 | 0.68 | -0.02 | 7.6 | orders: the building cycle |
| wool | 0.076 | 0.053 | 6.4 | -0.24 | -0.29 | 0 | 0.25 | -0.12 | 8.9 | weather |
| steel | 0.065 | 0.072 | 9.0 | 0.39 | 0.07 | 0 | -0.04 | 0.54 | 7.5 | the iron cycle, damped |
| stone | 0.064 | 0.040 | 5.4 | -0.50 | 0.11 | 0.03 | 0.61 | 0.01 | 7.8 | the building cycle, damped |
| cloth | 0.036 | 0.025 | 8.7 | 0.32 | 0.05 | 0 | -0.07 | -0.20 | 8.1 | steady |
| services | 0.022 | 0.020 | 9.4 | 0.01 | 0.32 | 0 | 0.03 | -0.32 | 7.7 | steady |

How each was told:

- **Weather**: grain and wool, the only goods whose yearly change follows the harvest (-0.47 and -0.29), with the sawtooth independent shocks give.
- **A cycle of orders**: a long period and an ac1 above zero for the iron family, whose price waves last eight to ten years and move together (with iron 0.3 to 0.5); and a four-year period, a large share bought for plant and a strong correlation with what is bought for plant (timber 0.79, bricks 0.84) for the building family. The building trial removes most of timber's (section 2.4). The iron family's cycle is the capacity cycle `dx-cycles.md` found, damped by the merchants' markup and still there; it was not traced further here.
- **A thin market**: charcoal, made in a quarter of a town on average after 1800 (most years nowhere), 41 per cent of what is asked not got, a stock of hundreds of months of sales: a trade that has died and still has a price. Iron ore is the other: three parts in a thousand of the island's output, six towns, and a price that saws year to year (ac1 -0.37) under the iron cycle's wave. I did not trace ore further.
- **Clay is none of the three.** Its yearly change has an ac1 of -0.86, a two-year sawtooth with no weather in it. With the land's hands following the wage three times as fast (`labor_adjust_ex` 0.5 for 0.15) its swing falls from 0.178 to 0.044 and the sawtooth goes (ac1 0.04), on every island of eight; with half the merchants' markup (`prices.markup` 0.25) it falls to 0.122. With the clay pits' hands alone three times as fast and every other sector's as now, it falls to 0.054 and ac1 to -0.23, on every island of eight: it is the pits' own hands. It is a ring between hands that come to the clay pits months late and a markup that over-corrects in a market where supply is elastic, the loop `dx-cycles.md` described for the old farm zigzag, now on the pits.

## 5. The cures, and what each moves

Eight islands each (sixteen where it says), against the same islands as they stand. None is in the code. Each trial switch is a few lines; the patch is kept outside the repository.

| | timber's swing | its period | mills: whole / bare | real wage swing | worst year | wage 1880-99 | works 1900 | tools: years over 3 medians |
|---|---|---|---|---|---|---|---|---|
| as it stands | 0.203 | 3.9 | 0.25 / 0.40 | 0.064 | -14.3 | 2.68 | 738 | 9.3 |
| orders spread evenly to December | 0.173 | 4.1 | 0.27 / 0.37 | 0.060 | -13.0 | 2.65 | 774 | 11.5 |
| room in proportion to what is short | 0.196 | 3.3 | 0.26 / 0.35 | 0.061 | -15.7 | 2.27 | 792 | 1.3 |
| a builders' queue (a month's trade a month) | 0.141 | 4.5 | 0.27 / 0.40 | 0.053 | -12.5 | 2.74 | 729 | 7.3 |
| growth step in proportion to the excess return | 0.178 | 3.5 | 0.16 / 0.22 | 0.056 | -13.4 | 2.63 | 798 | 6.9 |
| plant valued at fixed relative prices | 0.161 | 3.8 | 0.22 / 0.35 | 0.055 | -14.3 | 2.75 | 602 | 5.4 |
| room counted on what was asked | 0.208 | 3.9 | 0.25 / 0.40 | 0.065 | -14.9 | 2.60 | 712 | 9.3 |
| `works.year_room` 0.10 | 0.149 | 3.5 | 0.42 / 0.27 | 0.049 | -11.4 | 2.74 | 764 | 2.4 |
| the same, sixteen islands | 0.148 (16 down) | 3.7 | 0.41 / 0.27 | 0.049 (16 down) | -12.0 (13 better) | 2.74 (7 up) | 747 | 2.2 (14 down) |
| islands 17-24 as they stand | 0.206 | 3.6 | 0.24 / 0.42 | 0.064 | -16.3 | 2.62 | 718 | 9.5 |
| `works.year_room` 0.10 on them | 0.155 (8 down) | 3.2 | 0.42 / 0.27 | 0.051 (8 down) | -12.3 (6 better) | 2.65 (5 up) | 738 | 5.8 (5 down, 3 up) |
| `works.year_room` 0.30 | 0.230 | 5.0 | 0.04 / 0.56 | 0.072 | -16.3 | 2.55 | 675 | 14.1 |
| `labor_adjust_ex` 0.5 | 0.146 | 4.3 | 0.24 / 0.39 | 0.049 | -12.7 | 2.76 | 719 | 3.8 |
| the same for the woods alone | 0.169 | 3.3 | 0.25 / 0.39 | 0.058 | -13.1 | 2.67 | 731 | 9.3 |
| the same for the clay pits alone | 0.177 | 3.8 | 0.23 / 0.38 | 0.059 | -12.5 | 2.69 | 722 | 7.6 |
| the same for the woods and the clay pits | 0.159 | 4.0 | 0.27 / 0.38 | 0.058 | -13.7 | 2.76 | 702 | 8.9 |
| `prices.markup` 0.25 | 0.168 | 4.6 | 0.32 / 0.40 | 0.059 | -14.5 | 2.56 | 739 | 13.0 |
| `prices.enabled` false | 0.150 | 6.9 | 0.36 / 0.39 | 0.057 | -17.0 | 2.41 | 888 | 15.0 |
| plant without timber (not a cure) | 0.049 | 7.4 | 0.26 / 0.40 | 0.042 | -8.6 | 2.82 | 724 | 3.1 |

What each is:

- **Orders spread over the months of a year** (`TRC_SPREAD`). An order asks each month for `1 / (12 - month)` of what is still outstanding, so it is delivered evenly by December, where the port asks a quarter each month and most of it comes in the first quarter. Takes 15 per cent off; the wave is the year's orders, not their first months.
- **Room in proportion to what is short** (`TRC_PROP`). The year's room is the lesser of 15 per cent of the trade and what the town is short of: the greater of last month's sales and asks, grown 7 per cent and run at nine tenths, less every works' capacity making the good there, built and on order. Barely moves timber, ends the famine of tools (1.3 years), and costs 15 per cent of the late wage: as built it holds back trades that should grow. A better-judged version might keep the gain on tools; it would need its own search.
- **A builders' queue** (`TRC_QUEUE`). Works being built in a town may ask in a month for no more of a good than the town's monthly trade in it; the rest waits. Takes 31 per cent off, the most of the plain cures. It is not free: the wage of 1830-49 is 4 per cent lower (1.78 for 1.85, lower on 7 islands of 8), while that of 1880-99 is 2 per cent higher. The check wrote this queue again from these words alone and got every number the same.
- **A growth step in proportion to the excess return** (`TRC_STEP`). A works over the hurdle asks for `0.2 K (roi - hurdle) / hurdle`, at most half, in place of at least a fifth. Halves the mills' bare years and stretches the years between a town's whole years to 4.3, but timber's swing falls only 12 per cent: the towns still turn together.
- **Plant valued at fixed relative prices** (`TRC_PLANT_BASE`). The review reckons the return on the plant at its base prices relative to the output's, so dear timber no longer makes every mill's return look poor. Takes a fifth off and leaves 136 fewer works in 1900.
- **A smaller room** is the cheapest that helps, and the only one tried on sixteen islands and then on eight more. It is a number already there (`works.year_room`), which `works.md` chose at 0.15 against 0.3 before the merchants' markup and the families were in. On islands 17 to 24 it lowers the swings on every island as on the first sixteen, and shortens the famine of tools by less.
- **Faster hands for one land sector** (the check's trial switches `CHK_KX_FORESTRY` and `CHK_KX_CLAYPIT`). The wage's search uses `labor_adjust_ex` cell by cell, with 0.5 for the woods' or the clay pits' cells and 0.15 for the rest. The woods alone take 17 per cent off timber's swing, the pits alone 13, both 22, against 28 for every land sector; the other land sectors carry the rest, by a path not traced here.

## 6. For the pass over all the rules

- **`works.year_room`** wants searching again, with the swings in the score. At 0.10 every swing measured here is smaller on twenty-four islands of twenty-four, the famine of tools is less than half as long (a quarter as long on the first sixteen), and the late wage and output are lower on about half the islands and no lower on the mean; at 0.30 everything is worse. Try 0.05 to 0.15 on sixteen islands or more, scored on twenty-year means of the wage and output and on timber's and the wage's swing.
- **`firms::review`'s growth step** (at least a fifth whenever the return clears the hurdle) is what turns a smooth signal into all or nothing. A step in proportion to the excess return halves the bare years. Worth trying with a smaller room, where the two may add.
- **The land's slow hands** (`labor_adjust_ex`, one number for every land sector) are a quarter of timber's swing and all of clay's sawtooth. The clay pits' hands alone made faster end the sawtooth (0.178 to 0.054 on eight islands of eight); the woods' alone take a sixth off timber's swing. A faster hand for the pits, or the markup lower where supply is elastic, would end the one; the other wants the size of the lumps looked at first. The farms' zigzag that the number was chosen for should be watched if it moves for every sector.
- **The smithies' room.** The room is a share of what a trade sells, and a short trade sells what it makes; a room that widens with a trade's return (a smithy earning eight times the hurdle may grow faster than one earning the hurdle) would end most of the famine and raise the late wage. Three times the smithies' room alone is a crude stand-in for it. Families' works are not rationed by the room, which is why counting a dear trade larger did harm with families in `works.md`; any version of this has to be measured with six families.
- **The engine works' plant of 20 tools a unit** (`data/content.toml`, the Python's content, so not for this pass to change) makes each wave of engine works a famine of tools. It is a content number to look at when the deeper production tree comes.
- **Timber in the households' budget.** Households buy two thirds of the timber and are the buyer who gives way; the wave reaches what a wage buys through warmth and shelter. A cycle in building is honest; one that a household feels as 15 per cent off its wage every few years is larger than history's.

## 7. How sure, and what would change my mind

- **Sure**: that timber's swing is mostly the orders for plant (sixteen islands of sixteen in the counterfactual); that a smaller room lowers the swings on every island (twenty-four of twenty-four); that clay's sawtooth is the pits' own slow hands (the pits alone made faster end it on eight of eight); that households had left tools before the famines came; that the smithies are held by their room in a famine (they take nine tenths of it at eight times the hurdle, and a wider room shortens the famine on 13 of 16); that the famine is no longer tied to the arrival of farm machinery.
- **Less sure**: why the mills cross the hurdle together. The counterfactual shows it is not timber's price; that it is cloth's margin narrowing after a wave is my reading and was not switched off. A trial with cloth's price held for the review would settle it.
- **Noisy**: the famine of tools. Two islands in sixteen have no year over three medians and two have a year over ten times the median; eight islands differ from the next eight by more than some of the cures move them, which is why sections 3.3 and the room of 0.10 were run on sixteen. Eight more islands (17 to 24) bore that out: the room of 0.10 shortened the famine on 14 of the first sixteen and on 5 of the next eight. The cures on eight islands in section 5 that move the famine by less than half should not be believed.
- **What would change my mind**: a run where the room is smaller and the wage lower on the mean and on most islands (the wage of 1880-99 was lower on 9 islands of 16 and 3 of 8 more, the mean a little higher on both, so the room's cost in wage is unproven either way); a December-to-December measure of timber that showed trebling every three or four years on most islands (on these, a year's dearest month is 1.4 times its cheapest on average).
- **Not checked**: the commons scenario; years after 1900; the port; the viewer (nothing here changes it); the scripted players; the cures together.

## 8. How to run it again

```bash
cargo run --release -p isostate-core --example trace_cycles -- --seeds 1-8 --detail 2 --json base8.json
cargo run --release -p isostate-core --example trace_cycles -- --seeds 1-16 --jobs 16 --json base16.json
cargo run --release -p isostate-core --example trace_cycles -- --seeds 1-16 --jobs 16 --set works.year_room=0.10 --compare base16.json
cargo run --release -p isostate-core --example trace_cycles -- --seeds 1-8 --set labor_adjust_ex=0.5 --compare base8.json
cargo run --release -p isostate-core --example trace_cycles -- --seeds 17-24 --set works.year_room=0.10
```

The trial switches need the patch applied (`git apply trial-switches.diff`) and an environment variable each: `TRC_SPREAD=1`, `TRC_PROP=0.07`, `TRC_QUEUE=1`, `TRC_STEP=1`, `TRC_PLANT_BASE=1`, `TRC_ASKED=1`, `TRC_NOTIMBER=1`, `TRC_SMITHY_ROOM=3`, `TRC_ENGINE_ROOM=0.5`. With none set the patched build gives every number as the clean one. The check's own switches are a second patch (`check-trial-switches.diff`, kept beside the first): `CHK_QUEUE=1`, `CHK_KX_FORESTRY=0.5`, `CHK_KX_CLAYPIT=0.5`; with none set it too gives every number as the clean build.
