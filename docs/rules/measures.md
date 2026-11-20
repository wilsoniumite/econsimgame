# Measures of what a wage buys and of what is made

Added on 20 November 2026. Not in the Python. The rule `linked` (`src/linked.rs`) only watches: nothing in the
economy reads it. With it off under the game's rules, eight islands to 2050 give every one of their 3,700
facts as main does, the news among them.

The island has three ways of saying what a wage buys and how much it makes, and they disagree after 1900. This
page says what each is, which to trust for what, and the test that chose. In short:

- **The basket of 1750** (the port's `cpi`, `real_wage`, `gdp_real`, `gdp_pc`) is right for the first decades
  and wrong after them. By the 1970s it is mostly a price of timber.
- **The chained index** (`cpi_chained`, `real_wage_chained`, [engel.md](engel.md)) links every year. Its wage
  passes the test below on the mean of the islands, but not on every island, and an index of output chained the
  same way drifts badly.
- **The linked measure** (`cpi_linked`, `real_wage_linked`, `made_linked`, this page) holds a basket for five
  years at a time. It passes on every span and on every island. It is what to read after 1900, and since this
  page the news, the viewer's charts and the gate's growth after 1900 read it.

## The three measures

| series | what it is |
|---|---|
| `cpi`, `real_wage` | the basket of the warm-up's last month (what households and landlords bought then) at each month's island prices; the wage over it |
| `gdp_real`, `gdp_pc` | spending (households', landlords', urban rent and investment) over `cpi`: what the news read until now |
| `cpi_chained`, `real_wage_chained` | one Fisher link a year between two years' means, on what households and landlords bought, at the island prices (towns weighted by people) |
| `gauge_fork_all` | the gauges' own chained Fisher index of what households bought, a link a year at prices weighted by what was bought ([gauges.md](gauges.md)) |
| `cpi_linked` | a basket held for five years: the cost of the link's basket at the month's prices over its cost at the link's prices, times the level the link began at; 1 at the warm-up's end |
| `real_wage_linked` | the mean wage over `cpi_linked`, 1 at the warm-up's end, as `real_wage_chained` is |
| `made_linked` | every good made in the month (gross, as `out_*`), valued at the link's prices, in money of the warm-up's last month a month |

**How a link is made.** Links begin in 1750 and every `linked.link` years after (5: 1755, 1760 and on). A link's
basket is the mean of what households and landlords bought a month over the `linked.window` years before it
begins (5), and its prices are the means of the same years. The years are the ones before the link, not round
it, because the series is known month by month (the view never waits, and the news reads each year as it ends).
Five years is one turn of timber's cycle, so no link begins on a dear-timber year alone. The first link is the
warm-up's last month, where the basket of 1750 and the chained index begin too.

**Prices weighted by what was bought.** Each good's price in the cost of living is its price in each town
weighted by what households and landlords bought there that month (a unit value), where the observer's island
price weights the towns by their people. A good nobody bought in the month takes the island price. What is
made is valued at the island prices, since nothing records where each unit was made.

**Joining the links.** A new link is joined to the old over its window: the old link's mean over the window is
the new link's level. For the cost of living that is the old basket at the window's mean prices over the old
basket at its own prices; for what is made, the window's mean output at the old link's prices over the same at
the new. So the series has no step where a link begins, at the window's prices.

**What it costs.** Two sums over the towns and goods a month, and a few more each year. Eight islands to 2050
took 67.0 s of a processor each with it on and 69.4 s with it off, in one sweep side by side; the difference is
the machine's noise, and the measure costs less than it can show.

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set linked.enabled=false,true --jobs 8 --compare MAIN_SWEEP_TO_2050 --out DIR
```

## Why links of a year drift here

A chained index compares each year with the one before on the baskets of both. That is right when prices move
smoothly. Here timber's price swings fivefold against the wage every four or five years from the 1940s, bricks
with it, and what builders buy swings with them. A link from a cheap-timber year to a dear one weighs timber by
the cheap year's large purchases, and the link back by the dear year's small ones, so the two halves of a swing
do not cancel and the index wanders. The check of 20 November found it in an index of what is made chained a
year at a time: 0.78 per cent a year a head from 1985 to 2050 on islands 1 to 16 (0.44 to 0.90), against 0.26
at 1985's prices and 0.21 at 2046's (`drift.py` on the traces below). The linked index of what is made gives
0.23.

The chained wage drifts much less, since households buy little timber and few bricks themselves. The test
below finds it inside the band on the mean of the islands on every span, but outside it on six islands of
sixteen from 1970 to 2050, and it gains about a fifth of a point a year more than the band allows from 1910 to
1940 on the mean (decade by decade, below).

## The test

On islands 1 to 16, the game's rules, six families, to 2050. For each span, the wage over the cost of a fixed
basket: the basket of the span's first five years, and again of its last five, each priced every year at the
prices weighted by what was bought. The true growth lies between the two. Each end of a span is the mean of
the nine years round it (1750 is 1750-1754, 2050 is 2042-2050). Per cent a year, the mean of the islands:

| span | islands | basket of the first years | basket of the last years | chained | gauges | linked, 5 years | 10 years | 25 years |
|---|---|---|---|---|---|---|---|---|
| 1750-1900 | 1-8 | 1.040 | 2.616 | 1.193 | 1.281 | 1.189 | 1.144 | 1.066 |
| | 9-16 | 1.001 | 2.607 | 1.173 | 1.253 | 1.161 | 1.118 | 1.044 |
| 1900-1970 | 1-8 | 1.106 | 1.725 | 1.369 | 1.247 | 1.245 | 1.222 | 1.165 |
| | 9-16 | 1.174 | 1.815 | 1.447 | 1.309 | 1.303 | 1.275 | 1.212 |
| 1970-2050 | 1-8 | 0.211 | 0.271 | 0.253 | 0.219 | 0.228 | 0.231 | 0.225 |
| | 9-16 | 0.223 | 0.283 | 0.225 | 0.228 | 0.240 | 0.242 | 0.237 |
| 1985-2050 | 1-8 | 0.182 | 0.245 | 0.201 | 0.188 | 0.198 | 0.201 | 0.194 |
| | 9-16 | 0.189 | 0.253 | 0.191 | 0.194 | 0.206 | 0.208 | 0.202 |

On the mean every measure lies inside its band on every span. Island by island, of sixteen, inside their own
band on 1750-1900, 1900-1970, 1970-2050 and 1985-2050:

| measure | islands inside |
|---|---|
| the chained wage | 16, 14, 10, 12 |
| the gauges' index | 16, 16, 14, 13 |
| linked, 5 years | 16, 16, 16, 16 |
| linked, 10 years | 16, 16, 16, 16 |
| linked, 25 years | 11, 15, 16, 16 |

**The link's length.** Five years and ten both stay inside on every span and every island; twenty-five does not
(1750-1900 on five islands, where a basket held so long misses how fast households leave homespun and firewood).
The rule was to keep the shortest that stays inside, so `linked.link` is 5. Longer links read lower, as a
basket held longer misses more of the buyers' turning away from what grew dear.

**Decade by decade** (the same test on the decades, the mean of sixteen islands; nine-year ends make adjacent
decades share years, so read it as a guide), the chained wage stands above its decade's band from 1910 to 1940
(by 0.12, 0.29 and 0.12 a year) and in the 1980s (0.09), and below it in the 1960s (0.14). The linked wage at
five years is never more than 0.02 a year from its decade's band.

**Before 1900 the band is wide** (1.0 to 2.6 per cent a year), because the basket of the 1890s holds goods that
did not exist in the 1750s (steel, machinery), priced then at the price of a good nobody made. So the test is
weak before 1900, and says only that no measure is far wrong there. The chained wage's two standing facts to
1900 are left as they are: the test shows no drift before 1900.

**What is made** passes the same test: every good's output at the island prices of the span's first five years
and of its last five, a head, against `made_linked` a head. On islands 1 to 16 the band is 0.92 to 2.63 per cent
a year for 1750-1900, 1.07 to 1.84 for 1900-1970, 0.229 to 0.278 for 1970-2050 and 0.198 to 0.250 for 1985-2050,
and `made_linked` at five years gives 1.44, 1.37, 0.261 and 0.227, inside on every span (at 10 and 25 years too).

How it was run: `examples/linked_trace.rs` plays one island to 2050 and writes a row a year with every observer
series and, for each good, what households and landlords bought (`bought_`) and its price weighted by it
(`uv_`). The scripts that read the traces (`band.py`, `slumps.py`, `gate.py`) were kept with the notes of the
piece, not in the repository.

```bash
cargo run --release -p isostate-core --example linked_trace -- SEED OUT.csv     # islands 1 to 16, 110-134 s each, 16 at once
isostate-core sweep --seeds 1-16 --rules game --families 6 --end-year 2050 --set linked.link=5,10,25 --out DIR    # 48 runs, 259 s
```

## What reads it

- **The news's slump clock** (`observer::yearly`, "downturns, on two clocks") reads the yearly mean of
  `made_linked` in place of `gdp_real` under the game's rules: "What the island makes fell 3.2% this year", and
  the dated slump that follows. The doublings of real output and of the real wage still read the basket of
  1750. Under the port's rules the clock reads `gdp_real`, byte for byte.
- **The viewer**: what a wage buys, in the island's charts and in the Work half, is the linked wage where a run
  keeps it, and the island's charts show what is made a head beside it.
- **The facts**: `real_wage_linked_Y_vs_1750` and `made_linked_pc_Y_vs_1750` for 1850, 1900, 1950, 1970, 2000
  and 2050 (those inside the run) and the run's last year, each the mean of the nine years round the year (the
  last nine for the run's last year) over the year 1750's figure (`linked::nine_years`). So in a run to 1900 the
  1900 fact is 1892-1900, and in a run to 2050 it is 1896-1904. The lab prints both, beside the chained wage.
- **The growth after 1900** of the standing check (`standing::GROWTH`) and the atlas's page of it read the
  linked wage's facts where the run keeps them, and print the chained wage beneath, not counted.

## The news, then and now

The clock replayed on the yearly series of the sweep above (`slumps.py`), islands 1 to 16, by the year of each
slump's peak; a depression is a fall of more than 8 per cent from peak to trough. The replay gives the news's
own count to the item: 41.4 dated slumps an island to 2050 then, 15.8 now, as `news_verdict` says.

| era | slumps dated, then | now | depressions, then | now | deepest, then | now |
|---|---|---|---|---|---|---|
| before 1900 | 319 | 65 | 49 | 5 | 19.0% | 12.9% |
| 1900-1990 | 309 | 53 | 180 | 0 | 28.5% | 5.4% |
| after 1990 | 35 | 134 | 35 | 3 | 36.8% | 10.7% |

After 1990 the news reads 3 depressions where it read 35, the deepest 11 per cent where it read 37, and many
more small slumps: what is made does fall two to four per cent in the dear-timber years, when building falls
15 to 35 per cent, and the old index hid them inside its long false depressions. On island 7 the news read 17
depressions from 1900 and reads one now, 2036 to 2042, 11 per cent.

**Before 1900 the news changes a great deal**, from twenty dated slumps an island to four. Half of the old ones
were the basket of 1750 (spending over the linked cost of living has 161 before 1900, against 319 on the old
basket, and 9 depressions against 49); the rest were dips in spending power, mostly dear harvests, in years
when what was made did not fall 2 per cent. Island 7 dates three slumps by 1785 and none from then to 1900. No
standing fact and no aim of the scorecard reads the slumps, so by the rule of the brief the clock reads the new
series from 1750 and not from 1901 only. If the played century should keep its harvest slumps, the change is
small: read `gdp_real` before 1901 and start the clock afresh in January 1901. I would not: a slump that is a
dear harvest is a dearth, and the harvest's own news tells it ([harvest.md](harvest.md)).

## The gate's three numbers

On the facts of the sweep above, at five-year links (means of the islands):

| | islands 1-8 | islands 9-16 |
|---|---|---|
| chained wage, 1900 to 1970, per cent a year (single years) | 1.375 | 1.461 |
| chained wage, 2050 over 1970 | 1.267 | 1.207 |
| linked wage, 1900 to 1970, per cent a year (nine-year means) | 1.245 | 1.303 |
| linked wage, 2050 over 1970 | 1.201 | 1.212 |
| hands on the land, 2050 less 1900 | -0.073 | -0.082 |

The island meets all three under the linked wage on both sets of islands, so the gate's asks read it (as the
brief's rule says). The linked wage grows 0.13 to 0.16 points a year less than the chained from 1900 to 1970,
which is the chained wage's gain of 1910 to 1940 above its band, and a little the nine-year means.

## Its numbers

| setting | game | what it is |
|---|---|---|
| `linked.enabled` | true | keep the three series, and the slump clock reads what is made |
| `linked.link` | 5 | years a link keeps its basket |
| `linked.window` | 5 | years before a link whose means make its basket and prices |

The rule is fixed when the island is made (`edits::FIXED_AT_START`): its series are made with the record, and
its first link is the warm-up's end. The game's golden hashes moved for it on 20 November 2026 (1750-1900
economy and news, 1901-2050 economy, news and `run.bin`; the books held).

## What worries me

- **A fixed basket within a link misses the buyers' turning away from what grew dear**, so the linked wage
  reads a little low against a perfect index; at five years it is inside the band everywhere, but at 25 years
  it is not. Its joins are an overlap of five years' means, not a Fisher link, so there is a small step at a
  join when this year's prices are far from the window's.
- **What is made is gross output**, every good once for each time it is made, iron in the rails and again in
  the rails. It moves with the quantity of making, not with value added; that is what the news wants, and not
  what an economist would call output.
- **New goods enter a link only at the next one**, valued at the window's prices: the gain of a good's
  arrival is not counted, only its getting cheaper afterwards. The same is true of the chained index.
- **The band is weak before 1900**, for the reason above. A better test there would price a good not yet made
  at the price a buyer would first have paid, which nothing records.
