# The scorecard

A way to read one sweep against the aims of the pass over the rules, and the
facts it reads. Not in the Python, and not a mechanism: nothing here changes a
run. The code is `scorecard.rs`, some additions to `facts.rs` and `sweep.rs`,
one word in `peopling.rs`, and the `scorecard` command in `cli.rs`. It began as
the first step of "Ready next 10" in the backlog (piece 11 of
`docs/backlog/found-along-the-way.md`), and since stage 1 of the pass over the
rules (`docs/designs/rules-pass.md`, section 5) it holds that pass's twenty
aims with their tolerances: every choice of a number in the pass is read on
this card.

There is no search yet. The card gives a count of the aims met and a single
loss that a search could later make as small as it can, and that is all.

## Reading a sweep against its aims

Every sweep writes `scorecard.txt` beside its other files, one card per
setting, and does not print it, so that the table and the comparison stay the
last thing a sweep says. The same card can be had again, or for an older sweep:

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-16 --rules game --families 6 --out DIR
cargo run --release -p isostate-core -- scorecard DIR
cargo run --release -p isostate-core -- scorecard DIR --spread-from REFERENCE_DIR
```

For each aim the card prints its number in the design's table, the fact that
reads it, its mean across the seeds with the least and the greatest, the aim's
band, its tolerance, the spread between the islands, how far the mean misses
the aim in tolerances, whether the aim is met, and how many islands meet it one
by one. Under each it says what to read beside it and who asks for it. Then how
many of the twenty aims are met and which are missed, the loss with each aim's
share of it, the two facts read beside the card, the aims the documents state
that no number here reads, and the facts a scorecard should be read with that
have no aim.

- **The tolerance** is the aim's own, as the design writes it beside the aim,
  in the fact's units: a twentieth of the towns' share, a tenth and a half of
  what a wage buys, two years for the first railway. It is held still: the
  sweep being scored does not move it. Where the design says "none", the mean
  must be on the aim's side.
- **The spread** is the fact's standard deviation across the seeds (with n - 1
  below the line), printed beside the tolerance for reading: a tolerance much
  narrower than the spread between islands asks more than sixteen islands can
  say. With `--spread-from` it is taken from another sweep's setting of the
  same name, or its first. It moves nothing that is judged.
- **The miss** is how far the mean falls on the wrong side of the aim, and
  nought on the right side or inside the band.
- **The islands.** Some aims ask something of the islands one by one: aim 6
  that twelve islands of sixteen grow by a tenth, aim 14 that fourteen open
  their first line between 1820 and 1832, aim 16 that every island makes nine
  tenths of its steel in converters. On fewer islands the card asks as many in
  proportion, rounded up. The islands column counts the seeds whose own value
  meets the islands' test (the aim's test, where the aim asks nothing of the
  islands, and within the aim's tolerance where the aim is a point, such as
  people five times 1750's, since no island is exactly on a point), out of
  every island that finished: one with no value (coke iron found and never
  reaching half) has not met it.
- **Met** means the mean misses by no more than one tolerance (or not at all,
  where there is none) and, where the aim asks it, enough islands meet it.
  An aim asked only of the islands is met when enough of them meet it. An aim
  in two parts (12, 15, 18 and 19) is met when both are.
- **An aim whose fact the sweep does not have** (an older sweep, a run that
  does not end in 1900, or rules that do not keep what the fact reads: the
  books, the chained cost of living and carting are the game's) says so on
  the card and is not met.
- **The loss** is the sum, over the aims on the mean that have a tolerance,
  of the miss in tolerances, squared. An aim with no value at all counts as
  missed by ten, so that a setting is never better for losing a fact. The
  aims with no tolerance and those asked only of the islands (12b, 13, 15b,
  16, 17, 18a and 20) are met or missed and not counted in it, since the
  design gives them no scale to be counted in.
- **A run that failed** is not read, since it has no facts, so the card says
  so on its setting's first lines, by seed, and a setting whose every run
  failed is named at the top. Read the card of a setting that lost islands
  with care: it is scored on the islands it kept.

The card is for runs from 1750 to 1900, the century the aims were stated
for, and most facts it reads are named by that century's years. A run to
another year has no `urban_share_1881_1900` (its twenty-year spans run
1871-1890, 1891-1910 and so on), so the card prints `-`, says there is no such
fact, counts the aim as missed by ten, and does not count it met. A loss from
such a sweep is not to be set against one from a run to 1900.

The card asks for sixteen islands and says so when it has fewer: a run differs
from its neighbour by more than most changes to the rules move it, and for
wages, swings and hard spells sixteen are barely enough (`works.md`,
`towns.md`). The design reads every level as a twenty-year mean over sixteen
islands, never as one year on eight.

## The aims, and where they are stated

The twenty aims of the pass over the rules (`docs/designs/rules-pass.md`,
section 2), with the tolerances as its critic left them (section 10: aim 6
between 10 and 40 per cent, aim 15 on fourteen islands of sixteen). Each is
tied to a document that already asks for it, or to the plain history of an
island that industrialises; the design gives the reasons, and the card prints
them under each aim. Where an aim is about a level it is judged on a twenty-year
mean, since a single year misleads.

| | Fact | Aim, on the mean | Tolerance | Of the islands |
|---|---|---|---|---|
| 1 | `pop_1900_vs_1750` | about 5.0 | 0.5 | |
| 2 | `urban_share_1751_1770` | about 0.25 | 0.05 | |
| 3 | `urban_share_1881_1900` | at least 0.50 | 0.03 | |
| 4 | `urban_share_1881_1900_less_1861_1880` | at least 0.02 | 0.01 | |
| 5 | `real_wage_1881_1900_vs_1750` | 2.5 to 3.5 | 0.15 | |
| 6 | `spending_per_head_chained_1890s_vs_1870s` | 1.10 to 1.40 | 0.03 | at least 1.10 on 12 of 16 |
| 7 | `real_wage_five_worst_years_mean` | at least -0.10 | 0.015 | |
| 8 | `price_swing_timber_from_1800` | at most 0.15 | 0.02 | |
| 9 | `price_swing_iron` | at most 0.16 | 0.02 | |
| 10 | `farm_zigzag_beyond_weather` | at most 1.2 | 0.25 | |
| 11 | `price_tools_years_over_twice_local_median` | at most 4 | 2 | |
| 12a | `famine_years` | at most 0.2 | 0.2 | |
| 12b | `hungry_months_after_new_cells` | | | none, on every island |
| 13 | `town_months_wage_over_ten_middle` | | | none, on every island |
| 14 | `rail_first_year` | about 1826 | 2 years | 1820 to 1832 on 14 of 16 |
| 15a | `rail_km_1850` | 1,200 to 1,700 km | 100 km | 1,200 to 1,700 on 14 of 16 |
| 15b | `rail_last_growth_year` | | | 1890 or later on every island |
| 16 | `share_bessemer_1880` | | | at least 0.9 on every island |
| 17 | `share_steel_rolling_1890` | | | at least 0.5 on 14 of 16 |
| 18a | `last_town_founded` | | | after 1860 on 12 of 16 |
| 18b | `towns_1900` | 18 to 26 | 2 | |
| 19a | `carters_between_towns_share_of_hands_1750` | about 0.02 | 0.01 | |
| 19b | `carters_between_towns_share_of_hands_1900` | at least 0.05 | 0.01 | |
| 20 | `years_to_half_coke_furnace` | at most 10 | none | |

Three were on the card before the pass and stay as they were: the towns' share
(aim 3, now with its own tolerance), iron's swing (aim 9) and coke iron's ten
years (aim 20). One moved: the farm zigzag's 1.2 was set for an island without
weather, and with the harvests on six tenths of the plain zigzag is the
weather's, so the aim is now judged on `farm_zigzag_beyond_weather` and
`farm_zigzag` is printed beside it. Aim 6 is on spending a head over the chained
cost of living, not the observer's `gdp_pc`, which the basket of 1750 turns from
a stall into a fall (`tr-works.md`, section 1); aim 5 stays on the basket of
1750, which is what histories of the real wage are reckoned on.

The aims are copied by hand into `scorecard::AIMS`, with the words that state
them. When the design or a page changes an aim, the table there has to change
with it; a test holds that there are twenty, numbered in order, that every one
names a page, and that the critic's two changes are in.

**Read beside the card.** Two of `PLAN.md` 1.3's six facts, which the plan
names and gives no number, and the pass reads with every stage without an aim:
`share_textile_mill_1830` (the mills' share has nowhere to go) and
`rail_km_1870` (the railway's length is the branch planner's).

**Stated, and not read by a number here:**

- Iron output swinging under 0.09 a year in every half-century, and iron
  stock near two months with under 4 per cent of orders unfilled
  (`prices.md`, "Measuring it again"). No fact gives any of the three; the
  prices page measured them with its own probe.
- Iron and rails no longer cycling with a period of a few years (PLAN.md 1.2).
  No number is given; read `price_swing_iron` and `price_swing_rails`.
- 150 years in under 15 seconds (PLAN.md 1.1). Set for the port of the
  Python, and a run under the game's rules does more. `cpu_seconds` and
  `seconds` are read without an aim: a search over the economy's numbers
  should not be steered by the machine.
- Two to four items a decade in the major feed (PLAN.md 1.4). The chronicle's
  own rule holds that (`per_decade` and `at_least`), so there is nothing to
  calibrate.

The card's list of facts without an aim is the README's headlines, the
measures of the faults the pass mends, and the worries the rules' pages
carry: what a wage buys over five spans of twenty years and on the chained
cost of living, output per head and its peak, the shares of work and of
income, coke's share of iron in 1800, the swing of rails' price, the worst
year for wages from the median town, the town-months with more at work than
people and those whose wage moved by half, the carters, the works near their
market and closing by age, the emigrants' news, and the speed.

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

## The facts added for the pass over the rules

Stage 1 of the pass (`docs/designs/rules-pass.md`, section 5, "Stage 1's
facts") added the facts its card needs that the sweep did not have. None was
renamed and no rule changed: against main's sweep of the same eight islands
(`merged-game-f6-m`, seeds 1 to 8, six families), of the 563 facts both have,
the seconds aside, 0 differ. The new ones are extra columns.

Some can only be seen month by month, so the sweep now steps each year a month
at a time, as `sim::run_year` does, and notes what it needs after each month
and after the year's end (`facts::run_year_noting`, `gather_month`). Nothing it
notes is written back. A run stepped a year at a time (an example, or a test
that calls `sim::run_year`) has no value for the facts from the months.

Each definition was taken from the page or the example that measured it
first, so that the fact reads what the design quotes, and
`tests/card.rs` holds each one: worked out a second way from the run as it
goes, on island 2 under the game's rules to 1790, and where the fact is a count
to an identity. The figures are from seeds 1 to 8 with six families, beside the
design's own.

| Fact | What it is | Taken from | 8 islands | The design |
|---|---|---|---|---|
| `pop_Y_vs_1750` | people in a mark year over 1750's (aim 1) | | 5.42 in 1900 | 5.42 |
| `urban_share_1881_1900_less_1861_1880` | the towns' share over the last twenty years less the twenty before them (aim 4) | | 0.014 | 0.014 |
| `spending_per_head_chained_Ds_vs_1750s`, `_1890s_vs_1870s`, `_peak_year` | spending a head over the chained cost of living, `gdp_nom / pop / cpi_chained` from the yearly means, as decade means (the last decade takes in the run's last year, so 1890 to 1900) against the 1750s, the 1890s against the 1870s (aim 6), and its highest single year | `tr-works.md`, section 1; `examples/trace_works.rs` | 1.047 | up 4.7 per cent |
| `real_wage_five_worst_years_mean` | the mean of the five largest yearly falls of what a wage buys, from 1760 (aim 7) | `tr-works.md`, section 4 | -0.120 | 12.0 per cent down |
| `price_swing_timber`, `price_swing_timber_from_1800` | timber's swing as iron's, from 1760, and from 1800 as the design reads swings (aim 8) | `tr-cycles.md` | 0.174; 0.203 | 0.204 |
| `price_tools_years_over_twice_local_median` | years from 1760 in which tools cost over twice the median of the twenty-five years centred on them, cut short at the run's ends (aim 11) | the design, aim 11 | 8.6 | 8.5 |
| `hungry_months_after_new_cells` | January and February town-months under the famine line (0.82 of the food needed, the people rule's) in a town that took cells from another town's market at the year's end (aim 12) | `tr-carriage.md`, (d); the definition is this stage's | 0.625 a run | |
| `town_months_wage_over_ten_middle`, `town_wage_greatest_vs_middle` | town-months in which a town's wage was over ten times the middle town's (the median of the towns' wages that month, each town once), and the most it ever was (aim 13) | `examples/wage_watch.rs` | none; 2.50 | |
| `town_months_more_at_work_than_people` | town-months with more at work than the town's people of working age | `examples/overhire.rs` | 0.375 a run | |
| `town_months_wage_moved_by_half` | of the town-months from February to December, the share in which a town's wage moved by half or more from the month before (`carting.adjust`) | `examples/wage_watch.rs` | 0.0002 | |
| `diggings_share_Y`, `diggings_share_A_Z` | the hands in quarries, clay pits and mines (their sinkers among them) over all at work, without the carters, navvies and road menders that `mine_share` counts | the design, section 5 | 0.091 in 1750 (beside `mine_share_1750` 0.216) | 0.090 |
| `carters_between_towns_share_of_hands_Y` | each town's carters times the share of its carting bill that is the works' carting and the trade it sends, from the books, over the hands as `carters_share_of_hands_Y` has them (aim 19) | the design, aim 19 and section 4.6 | 0.013 in 1750; 0.033 in 1900 | 0.020; 0.033 |
| `carters_per_1000_weight_path_Y`, `_Y_vs_1750` | the year's carters over the weight carried that year times the path cost it went over (each cell's produce and works' goods to their market, and the trade between towns), per thousand | the design, section 4.6, "How it is checked" | 0.279 in 1750, 0.40 of it by 1900 | |
| `works_closing_chance_age_A_B` | the chance a works closes in a year at an age from A to B (0-2, 3-5, 6-9, 10-14, 15-24, 25-49 and 50 up), closures over works-years at that age, works standing at any time from 1750 | `tr-works.md`, section 3; `examples/trace_works.rs` | 0.048 at 3-5, 0.017 at 25-49 | |
| `works_hands_within_2_of_market_Y` | of the works' hands at the year's end, the share whose works stand within a path cost of 2 of their market (`works.walk`) | `examples/works_census.rs` | 1.000 in 1900 | |

Three of them come out a little differently from the figures they were taken
from, and why:

- **The carters between towns in 1750** read 1.3 per cent where the design has
  2.0. The design's figure was put together from two pages: `tr-carriage.md`'s
  shares of the bill, island-wide and by decade, and `tr-urban.md`'s carters.
  The fact splits each town's carters by its own bill, in the single year
  1750. The aim is "about 2 per cent, one point either way", so today's
  island is inside it on either reading.
- **The works near their market** are 100 per cent from the 1850s, where
  `works.md` has 98 per cent from four seeds of an earlier game. The measure
  is the census's own. Since 1900 reads 0.999 to 1.000 on every one of
  sixteen islands and can show nothing, the card reads the year 1800 (0.91,
  between 0.73 and 0.98), when `works.walk` still has hands to place.
- **The weight carried times its path cost** is reckoned from the year's
  flows at the year's last path costs. At the freight it comes to 1.03 to
  1.05 of what the carting cost less the handling (`tests/card.rs`), not
  exactly: the land's carting is paid at a rate a unit fixed when the land is
  shared out, on the path costs of that moment, and a works' carting at what
  its inputs cost it over the town's price. A twentieth too much weight makes
  the level a twentieth low and moves the change over the century hardly at
  all, which is what stage 6 reads.

One fact the design asks for was left as it is: `works_closed_median_life`
stays beside the closures by age, though `tr-works.md` shows it is a poor
fact, because a fact is never taken away.

## The first card: four aims, sixteen islands under the game's rules

The card as it landed, before the pass's aims, with each tolerance the spread
between the islands of the sweep being scored:

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

The loss was 151, and 94 per cent of it the share of people in towns, because
that fact's spread between islands is under one point.

## The card before the pass

Today's main (the game's rules as they stood on 12 November 2026, with stage 1
built and no rule changed), sixteen islands, six families:

```bash
cargo build --release -p isostate-core
./target/release/isostate-core sweep --seeds 1-16 --rules game --families 6 --out DIR
#  DIR/scorecard.txt; sixteen runs, none failed, 75 s on the clock
```

| | Fact | Mean [least .. greatest] | Aim | Tolerance | Misses by | Met | Islands |
|---|---|---|---|---|---|---|---|
| 1 | `pop_1900_vs_1750` | 5.38 [5.08 .. 5.76] | about 5.0 | 0.5 | 0.75 | yes | 14/16 |
| 2 | `urban_share_1751_1770` | 0.215 [0.201 .. 0.224] | about 0.25 | 0.05 | 0.70 | yes | 16/16 |
| 3 | `urban_share_1881_1900` | 0.421 [0.404 .. 0.434] | at least 0.50 | 0.03 | 2.63 | no | 0/16 |
| 4 | `urban_share_1881_1900_less_1861_1880` | 0.011 [-0.004 .. 0.041] | at least 0.02 | 0.01 | 0.86 | yes | 3/16 |
| 5 | `real_wage_1881_1900_vs_1750` | 2.68 [2.51 .. 2.91] | 2.5 to 3.5 | 0.15 | 0 | yes | 16/16 |
| 6 | `spending_per_head_chained_1890s_vs_1870s` | 1.036 [0.97 .. 1.26] | 1.10 to 1.40, and 1.10 on 12 of 16 | 0.03 | 2.14 | no | 3/16 |
| 7 | `real_wage_five_worst_years_mean` | -0.120 [-0.146 .. -0.089] | at least -0.10 | 0.015 | 1.35 | no | 2/16 |
| 8 | `price_swing_timber_from_1800` | 0.203 [0.150 .. 0.265] | at most 0.15 | 0.02 | 2.64 | no | 1/16 |
| 9 | `price_swing_iron` | 0.174 [0.110 .. 0.198] | at most 0.16 | 0.02 | 0.69 | yes | 2/16 |
| 10 | `farm_zigzag_beyond_weather` | 1.27 [0.91 .. 1.81] | at most 1.2 | 0.25 | 0.30 | yes | 6/16 |
| 11 | `price_tools_years_over_twice_local_median` | 8.0 [0 .. 16] | at most 4 | 2 | 2.00 | no | 4/16 |
| 12a | `famine_years` | 0.375 [0 .. 1] | at most 0.2 | 0.2 | 0.87 | yes | 10/16 |
| 12b | `hungry_months_after_new_cells` | 0.625 [0 .. 3] | none on every island | | | no | 10/16 |
| 13 | `town_months_wage_over_ten_middle` | 0 | none on every island | | | yes | 16/16 |
| 14 | `rail_first_year` | 1827 [1809 .. 1842] | about 1826, and 1820 to 1832 on 14 of 16 | 2 | 0.56 | no | 11/16 |
| 15a | `rail_km_1850` | 1,348 [575 .. 1,804] | 1,200 to 1,700 km, and so on 14 of 16 | 100 | 0 | no | 11/16 |
| 15b | `rail_last_growth_year` | 1900 [1899 .. 1900] | 1890 or later on every island | | | yes | 16/16 |
| 16 | `share_bessemer_1880` | 0.928 [0.000 .. 1.000] | at least 0.9 on every island | | | no | 14/16 |
| 17 | `share_steel_rolling_1890` | 0.036 [0.000 .. 0.248] | at least 0.5 on 14 of 16 | | | no | 0/16 |
| 18a | `last_town_founded` | 1846 [1814 .. 1886] | after 1860 on 12 of 16 | | | no | 4/16 |
| 18b | `towns_1900` | 17.2 [12 .. 20] | 18 to 26 | 2 | 0.41 | yes | 9/16 |
| 19a | `carters_between_towns_share_of_hands_1750` | 0.014 [0.007 .. 0.023] | about 0.02 | 0.01 | 0.64 | yes | 15/16 |
| 19b | `carters_between_towns_share_of_hands_1900` | 0.034 [0.026 .. 0.040] | at least 0.05 | 0.01 | 1.56 | no | 0/16 |
| 20 | `years_to_half_coke_furnace` | 7.81 [6 .. 11] | at most 10 | none | 0 | yes | 15/16 |

**Eight aims of twenty are met** (1, 2, 4, 5, 9, 10, 13 and 20) and twelve
missed (3, 6, 7, 8, 11, 12, 14, 15, 16, 17, 18 and 19). **The loss is 30.7**:
timber's swing and the towns' share 23 per cent each, growth at the end 15,
the tools 13, the carters of 1900 8, the five worst years 6, and the other
nine 12 between them. No aim makes the loss alone any more. Beside the card,
the mills' share of cloth in 1830 is 0.963 [0.411 .. 1.000] and the railway of
1870 is 1,829 km [1,613 .. 2,034].

Read against the design's "today" column (section 2), every figure comes out
as the design has it, within what eight islands against sixteen move: people
5.4 times, the towns' share 0.21 and 0.42, growth at the end 3.6 per cent
(1.036), the five worst years 12.0 per cent down, timber 0.20, iron 0.174, the
zigzag 1.27, the first railway 1827 between 1809 and 1842, the last town 1846
and 17.2 towns. One reads off it: steel rails are 0.036 of rails in 1890 here
for the design's 0.002, which was seeds 1 to 8 (they read 0.002 on this card
too); three of seeds 9 to 16 roll a tenth to a quarter of their rails in
steel. The carters between towns in 1750 are the other (above). Aims 4 and 9
are met on the mean, inside their tolerance, and missed by most islands one by
one (3 and 2 of sixteen): read the islands column beside them.

**How firm "eight" is.** Sixteen islands give each mean a standard error of
its islands' spread over four. Three aims are met or missed by less than that:
the towns' gain at the end (aim 4) is met by half a standard error, the
famine years (12a) by a fifth of one, and iron's swing (9) by 1.2; the five
worst years (7) are missed by 1.2. So another sixteen histories of the same
island could read six aims met, or nine, and a stage that moves the count by
one or two has not shown it until its null pair moves less (the design,
section 3). The loss is no firmer: those four aims are 12 per cent of it.

## What worries me

- **The tolerances are the design's, and nothing else holds them.** They were
  chosen, aim by aim, for what a reader would believe of an island; the card
  copies them. A tolerance much narrower than the islands' spread (the first
  railway: 2 years against a spread of 7) asks the mean of sixteen islands to
  be steadier than sixteen islands are, and the stage's null pair is what says
  whether a move is real (the design, section 3).
- **"Met" on the mean is still generous where an aim asks nothing of the
  islands.** The towns' gain at the end and iron's swing are met on the mean
  and missed by most islands. The islands column is printed for every aim for
  that reason.
- **Seven aims are not in the loss.** Those with no tolerance, and those asked
  only of the islands, are met or missed. A search that read only the loss
  would not see them: the steel rail (aim 17), every island's converter (16)
  and the hungry Januaries (12b) above all. Read the count of aims met with
  the loss, never the loss alone.
- **Aim 12's second half is defined here.** The design asks for no hunger "in
  the two months after a town takes a district" and gives no fact. A town
  takes a district, for this card, when a cell that was another town's market
  is its own after the year's end, and it is hungry when a January or February
  gives it under 0.82 of the food it needs. That is the people rule's famine
  line, so a hungry month here is one the rule would count. Stage 3's own
  measure is `trace_rent --part d` and should agree with it.
- **A twenty-year mean ending in a year lags a rising series.** For a share
  that is still climbing, the mean of 1881-1900 is lower than 1900 itself by
  about half of the last twenty years' rise. Today the towns' share is flat
  from 1880 and the two agree; if a change sets it climbing again, the card
  will judge aim 3 a little harshly, and aim 4 is there to see it.
- **The aims are copied by hand.** `scorecard::AIMS` holds them with the
  words that state them. A page that changes its aim has to change the table
  too.
- **The months are noted in the sweep's own loop.** The facts from the months
  have a value only in a run stepped by `facts::run_year_noting`; an example
  that steps a year at a time gets none of them, and says so with an empty
  cell rather than a nought.
- **`cpu_seconds` is not free of the machine.** A processor shared between
  two threads, a slower clock when it is hot, and caches shared with other
  work all lengthen it: on the quiet machine main's runs took 17 to 19
  seconds on the clock, and on the crowded evening of the first card 25 to 29
  of processor. It is steadier within a sweep, not a constant across days:
  the same sixteen islands, run again later that night on a busier machine,
  took 48.5 seconds of processor a run where they had taken 31.9, and the
  card before the pass, with a dozen other helpers building, 39.8.
- **Only some of the facts are worked out a second way in Python.**
  `parity/check_facts.py` (plain Python, no packages) works out the tools
  facts, the twenty-year means of the towns' share, the wage, output per head
  and the farm and industry shares, a few names by place, and the median
  town's worst year, from the files `isostate-core run` writes. On island 7
  under the game's rules with six families 164 facts agree and none differ;
  on island 3 under the port to 1835, 127. The zigzag beyond the weather, the
  chained wage's and the books' means, most names by place and the facts of
  the pass are not checked there; `tests/card.rs` holds the pass's in Rust.
