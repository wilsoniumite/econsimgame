# The scorecard

A way to read one sweep against the aims the project's documents state, and
the facts it reads. Not in the Python, and not a mechanism: nothing here
changes a run. The code is `scorecard.rs`, some additions to `facts.rs` and
`sweep.rs`, one word in `peopling.rs`, and the `scorecard` command in
`cli.rs`. It is the first step of "Ready next 5" in the backlog (piece 11 of
`docs/backlog/found-along-the-way.md`): every choice of a number in the coming
pass over the rules waits on being able to read a sweep.

There is no search yet. The card gives a single loss that a search could later
make as small as it can, and that is all.

## Reading a sweep against its aims

Every sweep now writes `scorecard.txt` beside its other files, one card per
setting, and does not print it, so that the table and the comparison stay the
last thing a sweep says. The same card can be had again, or for an older sweep:

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-16 --rules game --families 6 --out DIR
cargo run --release -p isostate-core -- scorecard DIR
cargo run --release -p isostate-core -- scorecard DIR --spread-from REFERENCE_DIR
```

For each aim the card prints the fact that reads it, its mean across the seeds
with the least and the greatest, the aim, a tolerance, how far the mean misses
the aim in tolerances, whether the aim is met, and on how many islands it is
met one by one. Under each it says what to read beside it and where the aim is
stated. Then the loss, with each aim's share of it; the aims the documents
state that no number here reads; and the facts a scorecard should be read with
that have no aim.

- **The tolerance** is the fact's standard deviation across the seeds (with
  n - 1 below the line), so it is the spread between one island and the next.
  With `--spread-from` it is taken from another sweep's setting of the same
  name, or its first: the way to hold the yardstick still while the rules
  move (below, "What worries me"). It is never less than a hundredth of the
  aim, so that a fact every island gives alike does not make any miss
  infinitely large.
- **The miss** is how far the mean falls on the wrong side of the aim, and
  nought on the right side. Every aim stated so far is "at least" or "at most".
- **Met** means the mean misses by no more than one tolerance.
- **Islands** counts the seeds whose own value is on the right side of the
  aim, out of every island that finished: one with no value (coke iron
  found and never reaching half) has not met it. PLAN.md asks some things
  of "most seeds", and this is that.
- **A run that failed** is not read, since it has no facts, so the card says
  so on its setting's first lines, by seed, and a setting whose every run
  failed is named at the top. Read the card of a setting that lost islands
  with care: it is scored on the islands it kept.
- **One island** has no spread, so every tolerance is the floor and the loss
  is many times larger than a sweep of many would give; the card says so.
  Give it `--spread-from` a sweep of many islands.
- **The loss** is the sum, over the aims marked in the loss, of the miss in
  tolerances, squared. An aim with no value at all (a run too short to have
  its fact) counts as missed by ten tolerances, so that a setting is never
  better for losing a fact.

The card is for runs from 1750 to 1900, the century the aims were stated
for, and some facts it reads are named by that century's years. A run to
another year has no `urban_share_1881_1900` (its twenty-year spans run
1871-1890, 1891-1910 and so on), so the card prints `-`, says there is no
such fact, and counts the aim as missed by ten; its swings and its zigzag
are over a longer run. A loss from such a sweep is not to be set against
one from a run to 1900. (Eight islands to 1960: the loss reads 120, of which
100 is the missing towns' share.)

The card asks for sixteen islands and says so when it has fewer: a run differs
from its neighbour by more than most changes to the rules move it, and for
wages, swings and hard spells sixteen are barely enough (`works.md`,
`towns.md`).

## The aims, and where they are stated

Only aims the documents already state with a number are here, each judged on
the fact that reads it. Where an aim is about a level by a year it is judged
on the twenty-year mean ending in that year, since a single year misleads.

| Fact | Aim | In the loss | Where it is stated |
|---|---|---|---|
| `urban_share_1881_1900` (beside it `urban_share_1900`) | at least 0.5 | yes | PLAN.md 1.2, the test of structural change: the urban share "rises from about 0.25 to 0.5 over the run in most seeds"; the README: "The plan asks for a half." |
| `farm_zigzag` (beside it `farm_zigzag_beyond_weather`) | at most 1.2 | yes | `prices.md`, "Measuring it again": "It has worked if the farm zigzag is under 1.2". Set before the harvests came in. |
| `price_swing_iron` | at most 0.16 | yes | `prices.md`, "Measuring it again": the iron price swings under 0.16 a year. |
| `years_to_half_coke_furnace` | at most 10 | yes | `prices.md`, "Measuring it again": coke iron passes half of output within 10 years of its discovery. |
| `cpu_seconds` (beside it `seconds`) | at most 15 | no | PLAN.md 1.1: 150 years in under 15 seconds. Set for the port of the Python; a run under the game's rules does more. The speed of the code is shown and not counted: a search over the economy's numbers should not be steered by the machine. |

Stated, and not read by a number here:

- Iron output swinging under 0.09 a year in every half-century, and iron
  stock near two months with under 4 per cent of orders unfilled
  (`prices.md`, "Measuring it again"). No fact gives any of the three; the
  prices page measured them with its own probe.
- Iron and rails no longer cycling with a period of a few years (PLAN.md 1.2).
  No number is given; read `price_swing_iron` and `price_swing_rails`.
- The railway going on growing after the first rush instead of stopping
  (PLAN.md 1.2). No number; read `rail_last_growth_year` and `rail_km_1870`.
- PLAN.md 1.3 asks for "a small table of stylised facts with tolerances" for
  population growth, the real wage's path, coke iron's share by 1800, mill
  cloth's share by 1830, rail km by 1870 and the urban share by 1900. Only the
  last has a number anywhere. The others are on the card without an aim.
- Two to four items a decade in the major feed (PLAN.md 1.4). The chronicle's
  own rule holds that (`per_decade` and `at_least`), so there is nothing to
  calibrate.

The card's list of facts without an aim is PLAN.md 1.3's list, the README's
headlines and the worries the rules' pages carry: people in 1900, what a wage
buys over five spans of twenty years, output per head, the shares of work and
of income, the methods' shares, the railway's dates and length, the swing of
rails' price, the tools famine, the worst year for wages from the median town,
and the famine and emigration years.

The aims are copied by hand into `scorecard::AIMS`. When a page states a new
aim or changes one, the table there has to change with it, and a test holds
that every aim names a page.

## Which fact to read when

Facts were named by year, so sweeps to different end years had few names in
common. There are now three ways to name a fact, and they answer different
questions.

- **By year**, `urban_share_1900`, `real_wage_1850_vs_1750`: a particular
  year. Read these to ask about that year, or between sweeps that ran the same
  years. The marks are the first year, every fiftieth year after it and the
  last.
- **Over twenty years**, `urban_share_1881_1900`,
  `real_wage_1881_1900_vs_1750`: the mean of the twenty yearly means ending in
  the year named second. Read these for anything that moves from year to
  year (wages, output per head, the shares of work and of people in towns,
  work's share of income) and for any claim that one setting is better than
  another. The spans end twenty, forty, sixty years after the first year and
  so on, and the last always ends with the run: for a whole run 1751-1770,
  1771-1790, ..., 1871-1890 and 1881-1900, the last two overlapping. A run of
  under twenty years after its first has none. Ratios are to the single first
  year, as the single-year facts have them, since every island starts from
  the same 1750.
- **By place in the run**, `pop_last`, `urban_share_last20`,
  `real_wage_last20_vs_first`, `rail_km_mid`, `works_size_median_last`: the
  first year, the middle year (`y0 + (y1 - y0) / 2`: 1825 for a run to 1900,
  1900 for a run to 2050), the last year, or the last twenty. Read these when
  two sweeps end in different years and the question is how each island
  stands at the end of its own run: a run to 2050 against one to 1900, or a
  short run used to test something quickly. They are not the same year on the
  two sides, so a difference in them is partly the difference in years. Each
  `_last` is the last year's fact under a second name, number for number, and
  the tests hold them equal.

The names by place are given for the people (`pop_last`, `pop_mid`,
`urban_share_first`, `_mid`, `_last`, `_last20`, `towns_last`), the shares of
work (`farm_share_`, `industry_share_`, `mine_share_` with `_first`, `_last`,
`_last20`), what a wage buys and output per head (`real_wage_mid_vs_first`,
`_last_vs_first`, `_last20_vs_first`, the same for `output_per_head` and the
chained wage), `income_share_work_last20`, the railway (`rail_km_mid`,
`rail_km_last`), the works (`works_last`, `works_ex_services_last`,
`works_per_1000_people_last`, `works_size_median_last`, `_p90_last`,
`_largest_last`, `works_top_1pct_capacity_share_last`,
`works_utilisation_last`) and the land (`cells_urban_last`,
`cells_industry_last`, `cells_derelict_last`). `news_first` is not one of
them: it counts the news of the kind `first`, a method's first output.

**Add, never rename.** The comparison between two sweeps counts only the facts
both have, so a fact renamed would drop out of it without a word.
`tests/sweep.rs` holds the 243 names a run under the game's rules gave on 12
November 2026, and a run must still give every one.

## The facts added on 12 November 2026

Against the game's sweep of that day (`merged-game-f6-l`, seeds 1 to 8, six
families), one fact of the 446 both have moved, `news_famine`, as it was meant
to (below). The figures here are from seeds 1 to 8 with six families unless
sixteen are named.

- **Twenty-year means** of what a wage buys, output per head, the chained wage,
  the share of people in towns, the shares of work on the land, in the works
  and in the diggings, and work's share of the island's income where the books
  are kept. What a wage buys over 1881-1900 is 2.64 times 1750's [2.51 ..
  2.76] where 1900 alone reads 2.60 [2.36 .. 2.78]: the spread between
  islands is half as wide.
- **The price of tools**: `price_tools_dearest_vs_median`, the dearest year
  from 1760 against the middling year; `price_tools_dearest_year`; and
  `price_tools_years_over_twice_median`. The dearest year is 5.2 times the
  middling one [2.1 .. 9.6], in 1855 on average [1773 .. 1898], with 14.5
  years above twice the middling price [2 .. 27]. On two islands of eight the
  dearest year is in the 1890s, not the famine of the farm machinery that
  `works.md` describes, so the count of dear years is the better reading of a
  famine across seeds. The price is in money; it hardly drifts, the cost of
  living ending between 0.9 and 1.6 of 1750's. Machinery is left out: its
  dearest year is the first years of the run on six islands of eight.
- **Emigration has a kind of its own.** Under the game's rules the sailings
  after a hungry year were filed as famine news, so `news_famine` and
  `famine_years` counted them as hunger. They are now the kind `emigration`,
  counted by `news_emigration` and `emigration_years`. The port files no such
  item, so its news and the golden test are untouched. On sixteen islands
  there is one sailing, on island 2 in 1827, and it came in a year that also
  had hunger: `news_famine` falls from 0.44 to 0.38 a run, `famine_years`
  does not move (0.38), and `news_emigration` is 0.06. The chronicle keeps
  the same items on island 2 as before. The viewer's news panel does not know
  the new kind yet, and shows it under "News" in grey.
- **The farm zigzag beyond the weather.** Since the harvests came in, most of
  `farm_zigzag` is the weather's, and it no longer tells the false cycle the
  prices page cured from good years and bad. `farm_zigzag_beyond_weather`
  sets each year's second difference of the farm share against the log of
  the country's harvest that year and the four before it (least squares, with
  a constant) and measures what the harvests do not account for, as the
  zigzag is measured. `farm_zigzag_weather_share` is the share they account
  for. With no harvests it is the zigzag itself. `farm_zigzag` is unchanged.

  | | `farm_zigzag` | beyond the weather | the weather's share |
  |---|---|---|---|
  | the port, without harvests (the false cycle) | 4.82 | 4.82 | |
  | the port, with harvests | 6.75 | 5.42 | 0.31 |
  | the game's rules, without harvests | 1.12 | 1.12 | |
  | the game's rules, with harvests (as played) | 2.08 | 1.35 | 0.59 |

  So the false cycle still reads about five with the weather on, and the
  game's rules read 1.35, a little above their 1.12 without harvests: the
  rest of the weather's echo, or the old spring set going a little.
- **The worst year for wages from the median town.** `real_wage_worst_year`
  reads the island's wage, every town's averaged by its people, so one town's
  wild wage coming back down reads as a fall for everyone.
  `real_wage_worst_year_median_town` takes each year the wage of the town at
  the middle of the island's people (half of them live in towns that paid
  less), over the year's cost of living, and gives its largest fall on the
  year before. On island 11 with the new land, six families and the commons,
  the island's fact reads -0.565 and the median town's -0.175. On ordinary
  islands the new fact is a little worse than the old (-0.151 against
  -0.143), because one town's wage swings harder than the island's mean.
- **The processor's time.** `seconds` is the time on the clock, with the other
  runs and whatever else the machine was doing beside it. `cpu_seconds` is the
  time the processor spent on the run's own thread, read from Windows (it is
  empty on other systems for now). On a crowded evening eight islands took
  26.5 to 49.6 seconds on the clock and 25.4 to 29.1 of processor.
- **`years_to_half_NAME`**: for each method found during the run, the years
  from its discovery to the year it first makes half its good's output. The
  scorecard reads it for coke.

## The first card: sixteen islands under the game's rules

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-16 --rules game --families 6 --out DIR
```

as run on 12 November 2026:

| Fact | Mean [least .. greatest] | Aim | Misses by | Met | Islands |
|---|---|---|---|---|---|
| `urban_share_1881_1900` | 0.421 [0.404 .. 0.434] | at least 0.5 | 11.95 | no | 0/16 |
| `farm_zigzag` | 2.03 [1.56 .. 2.72] | at most 1.2 | 2.88 | no | 0/16 |
| `price_swing_iron` | 0.174 [0.110 .. 0.198] | at most 0.16 | 0.64 | yes | 2/16 |
| `years_to_half_coke_furnace` | 7.8 [6 .. 11] | at most 10 | 0 | yes | 15/16 |
| `cpu_seconds` (not in the loss) | 31.9 [28.0 .. 35.8] | at most 15 | 8.84 | no | 0/16 |

The loss is 151, and 94 per cent of it is the share of people in towns. Beside
the zigzag, `farm_zigzag_beyond_weather` reads 1.27 [0.91 .. 1.81], and beside
the urban share's twenty-year mean, 1900 alone reads 0.420: the towns' share
has been flat since about 1880.

## What worries me

- **The tolerance comes from the sweep being scored.** A search that read
  this loss could lower it by making the islands differ more from one
  another, which widens every tolerance. Until there is a search, score with
  `--spread-from` a sweep of the rules as they stand, so that the yardstick
  stays where it is while the numbers move.
- **"Met" is generous.** Within one standard deviation between islands, the
  iron price's swing of 0.174 meets an aim of under 0.16, though only two
  islands of sixteen do. Read the islands column beside it.
- **One aim makes the loss.** The towns' share misses by twelve tolerances
  and the others by three at most, so a search on this loss would chase the
  towns alone. The documents give no weights, and I have invented none.
- **The zigzag's aim is older than the weather.** It was set when the farms
  had no harvests, and the weather alone now gives about 1.95. The aim is
  judged on the fact it was stated for, as the documents have it; the fact
  beside it is the one that still answers its question, and on that the
  game's rules stand at 1.27 against 1.2. Whether the aim should move to the
  new fact is a decision for the rules' pass, not for the scorecard.
- **A twenty-year mean ending in a year lags a rising series.** For a share
  that is still climbing, the mean of 1881-1900 is lower than 1900 itself by
  about half of the last twenty years' rise. Today the towns' share is flat
  from 1880 and the two agree; if a change sets it climbing again, the card
  will judge it a little harshly.
- **The aims are copied by hand.** `scorecard::AIMS` holds them with the
  words that state them. A page that changes its aim has to change the table
  too.
- **`cpu_seconds` is not free of the machine either.** A processor shared
  between two threads, a slower clock when it is hot, and caches shared with
  other work all lengthen it: on the quiet machine main's runs took 17 to 19
  seconds on the clock, and on the crowded evening above 25 to 29 of
  processor. It is steadier within a sweep, not a constant across days: the
  same sixteen islands, run again later that night on a busier machine, took
  48.5 seconds of processor a run where they had taken 31.9, and 172 on the
  clock where they had taken 39.
- **Only some of the new facts are worked out a second way.**
  `parity/check_facts.py` (plain Python, no packages) now also works out the
  tools facts, the twenty-year means of the towns' share, the wage, output
  per head and the farm and industry shares, a few names by place, and the
  median town's worst year, from the files `isostate-core run` writes. On
  island 7 under the game's rules with six families 164 facts agree and none
  differ; on island 3 under the port to 1835, 127. The zigzag beyond the
  weather, the chained wage's and the books' means and most names by place
  are not checked there.
