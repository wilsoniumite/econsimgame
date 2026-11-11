# The port and the game's rules

The simulation can be run under two sets of rules, and everything added to it
from here on has to say which of the two it belongs to.

**The port** is `default_params()` and `default_content()`: a line-by-line
port of the Python package, which stays the reference. Under it a run is the
Python's run, checked step by step (`PORTING.md`), and it must stay exactly
that. Nothing new is ever switched on here.

**The game's rules** are `game_params()` and `game_content()`: the port, plus
every mechanism the Python never had (foreign trade, canals, harvests, a life
cycle for works, whatever comes), each switched on with the numbers it was
tuned to. This is what a live game is played under. The list at the end of
this page says which mechanisms are in, and grows as they arrive.

Families are the one thing older than this convention. They are off under
both sets of rules until `n_families` says how many there are, which is the
caller's choice: the viewer asks for six, `--families 6` does it on the
command line (`crates/isostate-core/FAMILIES.md`).

## Which rules run where

| | rules | to get the other |
|---|---|---|
| `isostate-core run` | port | `--rules game` |
| `isostate-core sweep` | port | `--rules game` |
| the viewer, a new game | game | (a replay shows whatever was recorded) |
| `examples/bots.rs`, `examples/live.rs` | game | `--rules port` |
| `tests/parity.rs`, `tests/golden.rs`, `tests/run.rs`, `tests/families.rs` | port | |

In code, `Rules::Port` and `Rules::Game` name the two; `rules.params()` and
`rules.content()` give what a run needs. A `--params FILE` (TOML or JSON) is
laid over the chosen rules field by field, so a file that says only
`canals = { cost = 0.2 }` keeps the rest of the game's canals.

## Adding a mechanism

The pattern is the one the families set: off by default, on under the game's
rules, invisible in a port run's files.

1. **Its own module and its own small struct of numbers.** The struct derives
   `Clone, Debug, PartialEq, Serialize, Deserialize` and carries
   `#[serde(default, deny_unknown_fields)]`. Its `Default` is the mechanism
   switched off. A `game()` constructor gives the values a game is played
   with.
2. **One field in `Params`**, at the end, marked
   `#[serde(default, skip_serializing_if = "is_default")]`. `default_params()`
   leaves it at its default; `game_params()` sets it to `game()`. Because the
   field is left out while it is at its default, the `params.json` of a port
   run does not change.
3. **Off means nothing happens.** Every hook returns at once; the state is
   not touched; no random number is drawn from a stream the port uses. A
   mechanism that needs draws gets its own stream in `rng.rs` (`stream(7)`
   and up: each stream is made from the seed and its own number, so a new one
   disturbs none of the others), not `misc`, which the families draw from.
4. **New state is empty while off**, as the family tables are; new goods or
   methods are added in `game_content()`, never in `data/content.toml`, which
   is the Python's file.
5. **It acts through the same prices and bids as everyone else.** Every flow
   of money is a transfer between purses that `econ::money_total` counts, so
   the monthly conservation check goes on holding.
6. **A page beside this one**, `docs/rules/NAME.md`, written by whoever adds
   it: what it does and why, its numbers, what a sweep showed before and
   after, and what worries you. Add a line to the list at the end of this
   page.
7. **Check it.** `cargo test -p isostate-core --release` must pass with the
   golden constants untouched. Then run a sweep under the game's rules against
   the port's baseline (below) and read what moved.

In code, for canals:

```rust
// canals.rs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CanalParams {
    /// Promoters may dig canals.
    pub enabled: bool,
    /// Cost of carrying a ton a cell by canal, relative to a turnpike.
    pub cost: f64,
}

impl Default for CanalParams {
    /// Off: what the port runs with.
    fn default() -> CanalParams {
        CanalParams { enabled: false, cost: 0.3 }
    }
}

impl CanalParams {
    /// The game's canals.
    pub fn game() -> CanalParams {
        CanalParams { enabled: true, ..CanalParams::default() }
    }
}

// params.rs, at the end of Params
    /// Canals (not in the Python; see docs/rules/canals.md).
    #[serde(default, skip_serializing_if = "is_default")]
    pub canals: CanalParams,

// in default_params():   canals: CanalParams::default(),
// in game_params():      P.canals = CanalParams::game();
```

Nothing else needs wiring: `--set canals.enabled=true` and
`--set canals.cost=0.2,0.3,0.4` work in the sweep as soon as the field exists.

## The golden test

`tests/golden.rs` runs seed 7 to 1900 under `default_params()`, once without
families and once with six, and hashes the text of `series_monthly.csv` and of
`events.json` exactly as `isostate-core run` writes them (64-bit FNV-1a). The
four hashes are constants in the test file, taken at commit a5ea2ea, where the
hashed strings were compared byte for byte with that commit's output files. A
third check, instant, holds `params.json` to the Python's file. The two runs
are separate tests, so they run side by side: about 15 seconds together on a
quiet machine.

When it fails, one of two things has happened. Some new mechanism is leaking
into the default rules: it does something, or draws a random number, while it
is supposed to be off. Or the port itself was changed on purpose. For the
second case only:

```bash
ISOSTATE_GOLDEN_PRINT=1 cargo test -p isostate-core --release --test golden
```

checks nothing: each of the three tests stops with the line to paste over its
constant (as a failure, so that it shows and so that such a run is never taken
for a pass). Say in the commit message why the port moved. To see what
changed, not only that something did,

```bash
ISOSTATE_GOLDEN_DUMP=some/dir cargo test -p isostate-core --release --test golden
```

writes the files of both runs under that directory; do it before and after and
diff the two.

The golden test watches one seed for any change at all. Whether a change is
for the better is the sweep's question.

## The sweep

```bash
cargo run --release -p isostate-core -- sweep --out DIR [--seeds 1-8 | 1,2,5] [--rules port|game]
    [--families N] [--end-year Y] [--set key=v1[,v2...]]... [--params FILE] [--content FILE]
    [--jobs N] [--compare OTHER_DIR] [--quiet]
```

Every seed is run against every combination of the `--set` values, a few runs
at a time on their own threads (half the machine's cores unless `--jobs` says
otherwise). Eight seeds to 1900 take from 16 seconds to half a minute on this
machine, depending on what else it is doing, and the numbers do not depend on
how many run at once (`--jobs 1` and `--jobs 8` write the same files, the
seconds aside). A run that panics or fails the monthly money check is reported
as failed and the rest carry on; the command then ends with an error so a
script notices.

`--set` takes any field of `Params` by name, and a dotted path into a
mechanism's struct. The value goes in through the parameters' JSON form, so
there is no table of names to keep up to date; a number, `true`, a list in
brackets or a bare word (`rent_regime=commons`) all work. Whatever is wrong
with the command line is refused before anything runs or is written: a name
that does not exist, a value of the wrong kind or listed twice, an end year
before the start, more than 10,000 seeds, a `DIR` that cannot be made.

```text
--set land_thetta: there is no parameter "land_thetta". Did you mean land_theta?
```

It writes into `DIR`, replacing a sweep that is already there (so the same
command makes a baseline again):

- `annual.csv`: one row per run and year. The run's number, its seed, the
  swept values, the year, then every observer series averaged over the year's
  twelve months.
- `facts.csv` and `facts.json`: one row or object per run, its stylised facts
  (below). An empty cell or a null is a fact with no value, as when a railway
  was never built.
- `summary.txt`: the table it prints.
- `README.txt`: the command line, the rules, `git describe`, how long it took.

The table gives every fact's mean across the seeds in a column of its own,
one per setting, with the least and greatest value beside it. The range is
left out where its two ends print the same (every seed starts with 135,000
people), and `(3/8)` says only three of eight runs have a value at all. This
is the port on seeds 1 to 8, shortened:

```text
8 runs under the port rules: seeds 1 2 3 4 5 6 7 8, 1 setting; 0 failed; 16 s, 8 at a time
each fact: its mean across seeds, then [least .. greatest] unless the two print the same;
(n/N) when only n of N runs have a value, - when none has

                                      port rules
People and towns
  pop_1750                               135,000
  pop_1800                               229,223  [227,127 .. 231,075]
  pop_1850                               395,529  [390,493 .. 398,936]
  pop_1900                               682,305  [674,793 .. 689,383]
  growth_capped_years                      0.995  [0.974 .. 1.000]
  growth_capped_people                     0.976  [0.951 .. 0.991]
  ...
Work and wages
  ...
  real_wage_1800_vs_1750                    1.33  [1.27 .. 1.45]
  real_wage_1850_vs_1750                    1.34  [1.10 .. 1.58]
  real_wage_1900_vs_1750                    1.29  [0.93 .. 1.43]
  ...
Railways
  rail_first_year                           1828  [1815 .. 1838]
  rail_km_1850                             990.2  [920 .. 1,124]
  rail_km_1870                             1,016  [920 .. 1,258]
  rail_km_1900                             1,024  [920 .. 1,258]
  rail_last_growth_year                     1866  [1833 .. 1899]
  ...
Methods and inventions
  discovered_coke_furnace                   1772  [1764 .. 1780]
  first_output_coke_furnace                 1773  [1765 .. 1781]
  half_of_output_coke_furnace               1780  [1770 .. 1787]
  ...
  discovered_bessemer                       1858  [1833 .. 1882]
  first_output_bessemer                     1888  [1883 .. 1896] (3/8)
  half_of_output_bessemer                      -
  ...

never built, port rules: bessemer (in 5 of 8 runs), steel_rolling (in 7 of 8 runs)
```

With `--compare OTHER_DIR` it then reads `OTHER_DIR/facts.json` and sets each
fact's mean now beside its mean then, with the change. Like is compared with
like: where the two sweeps have seeds in common, only those seeds' runs are
used on either side, so four seeds are never set against eight and the
difference between islands taken for a difference between rules. (With no seed
in common every run of each is used, and the table says so.)

- A line at the top counts what moved.
- `=` is a fact that every seed gives exactly as it gave it before.
- `up/down` counts, seed by seed, where the fact is greater now and where it
  is less. `8/0` is every island moving the same way, however little; by
  chance alone that happens for one fact in 128.
- A star marks a fact whose ranges across seeds do not overlap: every seed now
  is above, or below, every seed then. That is a much harder test (a shift
  larger than the whole spread between islands). It is only given with four
  runs or more a side; with fewer, two sets of seeds under the very same rules
  stand clear of each other too often (one time in three with two a side).

Six families against none, the headings and the starred rows only:

```text
now: port rules, 6 families (8 runs); then: port rules (8 runs) in .claude/night/reports/sweep-port-baseline
seed for seed, on seeds 1 2 3 4 5 6 7 8
of the 128 facts both have, the seconds aside, 114 differ and 11 of those are starred
= every seed gives the number it gave
up/down: the seeds in which the fact is greater now, and those in which it is less
* the ranges across seeds do not overlap: every seed now is above, or below, every seed then
  (marked only with 4 runs or more a side)

                                          now     then            change  up/down
People and towns
Work and wages
Railways
Works
  works_1800                            2,131    1,536     +594 (+38.7%)      8/0  *
  works_1850                            4,156    2,804   +1,352 (+48.2%)      8/0  *
  works_1900                            6,973    4,512   +2,461 (+54.5%)      8/0  *
  works_ex_services_1800                2,046    1,456     +590 (+40.6%)      8/0  *
  works_ex_services_1850                4,010    2,660   +1,351 (+50.8%)      8/0  *
  works_ex_services_1900                6,700    4,260   +2,440 (+57.3%)      8/0  *
  works_per_1000_people_1900             10.2     6.61     +3.6 (+54.4%)      8/0  *
  works_size_median_1900                 0.46     0.84             -0.38      0/8  *
  works_size_p90_1900                    2.52     2.86             -0.34      0/8  *
  works_closed                          5,472    4,710     +763 (+16.2%)      8/0  *
Land
Methods and inventions
Prices and cycles
News
  news_total                            1,485    245.8  +1,240 (+504.4%)      8/0  *
Speed
```

So with families there are half as many works again and the typical one is
half the size. Nothing else moves clear of the spread between seeds, the news
aside, where the families' own items are counted. The tally shows what the
star cannot: these rows have no star, and every island moves the same way.

```text
  urban_share_1850                      0.257    0.231            +0.027      8/0
  urban_share_1900                      0.255    0.237            +0.018      8/0
  industry_share_1850                   0.274    0.239            +0.035      8/0
  industry_share_1900                   0.272    0.248            +0.024      8/0
  works_top_1pct_capacity_share_1900    0.058    0.080            -0.022      0/8
  cells_industry_1900                   980.8    886.2    +94.5 (+10.7%)      8/0
```

Six families building works every year raise the share of people in towns and
of work in industry by two or three points, on every island.

Each setting of the new sweep is set against the old sweep's setting of the
same name, or its first if none has that name. `OTHER_DIR` may be `DIR`
itself: the sweep that was there is read before the first run and replaced
after the last, which is the quick way to ask whether a change to the code
moved anything. A sweep set beside its own earlier self shows `=` in every
row but the seconds, since a run is the same run every time.

### The facts

All of them are worked out after the run from what it already keeps (the
observer's series, the yearly snapshots, the final state); the simulation's
loop is not touched. Two things nobody keeps, the towns' growth against the
cap and the land under each use, are noted between years by the sweep's own
loop. The facts are computed from the content, so another content file works:
a method or good it lacks is skipped, and one it adds gets its rows.

A fact taken from a series in a year is the mean of that year's twelve months,
the number in `annual.csv`: `towns_1800` is the towns there were during 1800.
A fact counted from the yearly snapshot (the works and the land) is taken as
the year ends, after that year's openings and closures: `works_1800` includes
the works founded at the end of 1800. The years are the first, every fiftieth
after it, and the last: 1750, 1800, 1850 and 1900 for a whole run, and the
names change with the run (`pop_1760` for a run to 1760).

The facts can be worked out a second time from the files `isostate-core run`
writes (the monthly CSV, the events and `run.bin`):
`crates/isostate-core/parity/check_facts.py` does it for one seed, in plain
Python written from this page and sharing nothing with `facts.rs`. For seeds
3 and 7, with and without families, and for a run to 1835, every fact that
can be had from those files agrees (107 of them for a whole run). The rest
(`growth_capped_*`, `works_top_1pct_capacity_share`, `works_utilisation` and
the `area_*`) are worked out a second way in `tests/sweep.rs`, from the yearly
snapshot and the final state of a run to 1790.

| Heading | Facts |
|---|---|
| People and towns | `pop_Y`; `growth_capped_years`, the share of years in which most people lived in towns whose growth was held at `growth_cap`, and `growth_capped_people`, the average share of people in such towns; `urban_share_Y`; `towns_Y`; `last_town_founded` |
| Work and wages | `farm_share_Y` and `industry_share_Y` of employment; `real_wage_Y_vs_1750` and `output_per_head_Y_vs_1750`; `output_per_head_peak_year` |
| Railways | `rail_first_year`; `rail_km_Y` for 1850, 1870 and 1900; `rail_last_growth_year`, the last year that ended with more line than the year before |
| Works | `works_Y` standing at the year's end and `works_ex_services_Y`; `works_per_1000_people`; size as built capacity K over the method's K0, for works with something built: `works_size_median`, `_p90`, `_largest`; `works_top_1pct_capacity_share`, the share of all capacity (built and on order, at the year's capital-goods prices) held by the largest hundredth of works; `works_founded`, every works opened from the first year on, sister works included; `works_closed` and their `works_closed_median_life` in years; `works_utilisation`, the mean over works |
| Land | `cells_urban_Y`, `cells_industry_Y` and `cells_derelict_Y`: the number of whole cells a town, the works or dereliction has taken; `area_USE_Y`: the land under farm, pasture, forestry, mines (quarries, pits and mines together) and left wild, in cells, as every cell's share summed (a cell is 2 km square); `urban_churn_10y`, of the cells urban in a year the share not urban ten years on, averaged over the run |
| Methods and inventions | for each method or tech not known at the start: `discovered_NAME`, and for methods `first_output_NAME` and `half_of_output_NAME`, the first year it makes half of all that its good's methods make; the plan's five: `share_coke_furnace_1800`, `share_textile_mill_1830`, `share_bessemer_1880`, `share_rolling_mill_1850`, `share_steel_rolling_1890`; and the list of methods never built |
| Prices and cycles | `price_swing_GOOD` for grain, iron, rails and cloth: the standard deviation of the yearly change in the log of the price, 1760 to the end; `farm_zigzag`, the mean absolute second difference of the yearly farm share of employment, times 100 |
| News | `news_KIND`, the number of news items of each kind, and `news_total` |
| Speed | `seconds` of wall time for the run, with the others running beside it |

### The baselines

Two sweeps of the port on seeds 1 to 8 are what a change is measured against:

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --out .claude/night/reports/sweep-port-baseline
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --families 6 --out .claude/night/reports/sweep-port-families6
```

They are not in the repository (sweep output never is); the two commands make
them again in a minute, and since the port is held in place by the golden test
they come out the same, the seconds aside.

## What the port's baseline says

Read against PLAN.md's list of what is off, on eight seeds instead of one:

- **Population grows at the cap almost every year** (`growth_capped_years`
  0.995). 682 thousand in 1900 is 135 thousand growing at 1.1 per cent for 150
  years, less a little. Living standards do not show in the population at all.
- **Towns do not pull people in.** The urban share is 0.26 in 1750 and 0.24 in
  1900; industry's share of work stays at a quarter; the farm share falls from
  0.65 to 0.46.
- **The real wage rises by a third before 1800 and then stops**: 1.33, 1.34
  and 1.29 times its 1750 level in 1800, 1850 and 1900, and in the worst seed
  below where it started. Output per head peaks around 1867.
- **The railway is built in a rush and stops**: first line about 1828, 990 km
  by 1850, 1,024 km in 1900. In three seeds of eight the last line is added
  by 1843.
- **Coke, mills and rolled rails win completely and early** (shares of 0.997,
  1.000 and 1.000 in 1800, 1830 and 1850). **Steel never arrives**: the
  converter is discovered around 1858 but built in only three seeds of eight,
  none before 1883, and never makes half the steel; steel rails are made in
  one seed.
- **Works stay small and die young**: the median works is 0.84 of its method's
  usual size, the largest 3.69 in every seed, the top hundredth hold 8 per
  cent of capacity, and the median life of a works that closed is under
  twelve years. The 3.69 is a ceiling, not a distribution: a works that would
  grow past `footprint_K_mult` (4) times its method's K0 founds a sister works
  instead (`firms::review`), and the yearly depreciation of 4 per cent keeps
  the largest a little under that.
- **Iron and rail prices swing two and a half times as hard as grain and
  cloth** (0.22 and 0.20 against 0.086 and 0.083).
- **No town is founded after about 1800** (the latest in any seed is 1824).
- **The woods retreat by a third and the wild land halves** (1,211 cells'
  worth of forestry in 1750 and 853 in 1900; 565 of wild land and 288), while
  the farms' and pastures' area falls a little (1,404 to 1,282 and 1,525 to
  1,301). What grows is the land under quarries, pits and mines (below) and
  the works' yards, from 204 cells to 886.

## What worries me

- **Eight seeds is a small sample.** The star in the comparison is a blunt
  test: with eight runs a side, two ranges fail to overlap by chance about
  once in 6,400 tries for one fact (once in 35 with four a side, the fewest it
  is given for), and it says nothing about a shift that is real and smaller
  than the spread. The `up/down` tally sees those, but it is a weak test taken
  a hundred and thirty times: `8/0` comes up by chance once in 128, so in any
  comparison one such row may mean nothing, and `7/1` means little. For
  anything close, run more seeds.
- **The golden hashes are this machine's.** They were taken on Windows with
  the MSVC toolchain. Logarithms, exponentials and powers come from the
  platform's maths library, so another system may round a last digit the
  other way, and over 150 years that grows into a different run. Not tried:
  if the golden test fails on a fresh machine with no change to the code,
  suspect this first.
- **The golden test watches the economy, not the map.** It hashes the monthly
  series and the news. A change that only moves what `run.bin` records for
  the viewer (a map layer, the works' table) passes it.
- **Without the Python dumps the golden test stands alone.** `tests/parity.rs`
  compares each step with the Python, but only against fixtures made by
  `parity/dump_state.py`, which are not in the repository; with none there
  its fifteen tests pass in no time and check nothing.
- **`growth_capped_*` does not see famines.** It reads the income rule in
  `people::growth` before the clamp; a town at the cap in a famine year grew
  two points slower and is still counted as capped.
- **Quarries, pits and mines take a fifth of the land.** The land is counted
  by area (it used to be counted by each cell's main use, by which the woods
  came to nothing: seven cells in 1750 and none after, against 1,211 cells'
  worth by area). By area the mines together go from 880 cells' worth in 1750
  to 1,190 in 1900, nearly as much land as the farms (1,282): about 4,800
  square kilometres of holes in the ground. I have not traced why. The land rule shares each
  cell out in proportion to bid^(1/theta), so every use with any bid at all
  gets a slice, and that may be all it is; it looks like one of the port's
  oddities and is worth a look of its own.
- **`seconds` is wall time under load**: with seven others beside it a run
  took 15 to 17 seconds when the machine was otherwise quiet and 24 to 35 when
  other work was building on it, so it says more about the machine that night
  than about the code.
- **Facts are named by year**, so sweeps to different end years have few
  names in common and compare poorly.
- **The loss function and the search of PLAN.md 1.3 are not here.** The facts
  have no tolerances yet; `facts.json` is the table a loss would be computed
  from.

## Mechanisms under the game's rules

One line each as they arrive: the name, its page, its switch.

- **prices**: merchants mark prices with the stock they hold and stop buying
  into a glut, which ends the false cycles in iron and on the farms.
  [prices.md](prices.md), `prices.enabled`.
- **engel**: past a reference income each further shilling of a household
  leans from the content's shares toward a richer set (less food and warmth,
  more comfort), and the observer keeps a chained cost of living and real
  wage beside the fixed-basket ones. [engel.md](engel.md), `engel.enabled`.
- **machines**: farm and quarry machinery is bought where it pays, machines
  stand for years and get better with the steel poured, and a converter and
  a steel rail that pay. [machines.md](machines.md), `machines.enabled`.
- **harvest**: good years and bad on the farms and pastures, island-wide and
  by district, half of each seen when the hands are hired; with the news of
  poor and bumper harvests, lean years and dearths. It belongs with `prices`.
  [harvest.md](harvest.md), `harvest.enabled`.
- **chronicle**: news that reads like a history: why each item is news, the
  two to four a decade a history would keep, stories of towns and districts,
  and an end to the flood of the player's own works. It never touches the
  economy. [chronicle.md](chronicle.md), `chronicle.enabled`.
- **works**: a life cycle for works. They grow where they stand, take land
  by their hands and stand by their town, so the island has hundreds of works
  of every size instead of thousands of small ones. [works.md](works.md),
  `works.enabled`.
- **railways**: branch lines to towns and districts, every line priced on the
  carting it saves, at five times the port's cost. [railways.md](railways.md),
  `railways.enabled`.
- **carting**: the carting bill takes hands out of each town's labour supply,
  so cheaper carriage frees people for other work. [carting.md](carting.md),
  `carting.enabled`.
- **people**: births and deaths that answer to income, crowding and dearth;
  towns founded all century where a market would save the most carting; and
  households that go with their cells when a market's area changes, taking
  their wage and their prices with them. [people.md](people.md),
  `people.enabled`.
- **land** (off: the viewer cannot draw it yet): an island made by uplift
  and erosion in place of the port's noise, with rain, lakes, the flow of
  every reach, river courses that meander and cut off, and resource fields
  that come from the land. [land.md](land.md), `land.enabled`.
