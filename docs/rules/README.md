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
| `tests/golden_game.rs` | game, with six families | |

In code, `Rules::Port` and `Rules::Game` name the two; `rules.params()` and
`rules.content()` give what a run needs. A `--params FILE` (TOML or JSON) is
laid over the chosen rules field by field, so a file that says only
`canals = { cost = 0.2 }` keeps the rest of the game's canals.

**Which island.** Under the port's rules a run is on the port's island.
Under the game's rules it has been on the new land since 19 November 2026
([land.md](land.md), "The switch"). The old island is reached by setting the
land's switch off: `--set land.enabled=false` for `sweep` and `lab`, a
parameters file that says `land = { enabled = false }` for `run`
(`--params FILE`), `--old-land` for a
new game in the viewer, and `save::Game::on_old_land` in code. A saved game
says which land it is on.

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
7. **Check it.** The core's tests must pass with the port's golden constants
   untouched (`python crates/isostate-core/run_tests.py` runs them all at
   once; "Running the tests", below). Then run a sweep under the game's rules
   against the port's baseline (below) and read what moved. Switching the
   mechanism on in `game_params()` moves the game's golden hashes, which are
   anchored again when it lands ("The game's golden test", below).

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

## The game's golden test

The port's golden test cannot see the game: under `default_params()` nearly
every hook of the game's rules returns on its first line. `tests/golden_game.rs`
holds the game in the same way. It runs seed 7 under `game_params()` and
`game_content()` with six families, as the viewer plays it, and hashes four
things apart (64-bit FNV-1a), so that a failure says which moved:

| name | what is hashed |
|---|---|
| `economy` | `series_monthly.csv` as `isostate-core run` writes it, less the books' own columns |
| `books` | the books' own columns (`acct_`), with the year and month |
| `news` | `events.json` |
| `run.bin` | the record the viewer draws, with the date it was written set to `0000-00-00` |

It hashes them in two eras, so that a failure also says which century moved:

| era | from | what |
|---|---|---|
| `1750-1900` | the island run to 1900, as the game has always ended | the economy, the books and the news, blind to content added after 1900 |
| `1901-2050` | the same island run on to 2050 | the months and the news from 1901 on, and the whole `run.bin`, strictly |

A change that acts only after 1900 (electricity, offices, the machines for
tasks, which find nothing before 1901) moves the second era alone, and the
message says that the played century held. The second era's `run.bin` holds
the years before 1901 too, so it moves with either era.

**The first era is blind to content added after 1900** (since 20 November
2026). Adding a method that is found after 1900 used to move the first era
without moving the run: the series gained the new method's columns, which
are nothing in every month to 1900, the news names an invention by its place
in the content's list, which a method put before it moves on, and `run.bin`
carries the content's lists. So the first era is now hashed with every column
that is nothing (nought or empty) in every month to 1900 left out, with each
invention in the news named by its name, and without a `run.bin` of its own
(the second era's holds the years to 1900). A column that is anything else in
any month still counts, and so does every other field of the news. Proved on
the case that cost a landing: the tree before the offices (c8fee93) and the
tree after them (03e0dbb) give the same three first-era hashes. The second
era stays strict: a new column or a moved place there moves it.

The seven hashes are kept in `tests/golden_game.txt`, beside the test, not in
its code. Three runs go side by side on three threads: the run to 1900 and
the run to 2050 twice. Before anything is set against the file, the two runs
to 2050 must agree, and the run to 2050 must pass through the run to 1900. So
a failure is one of three things, and the message says which:

- **The run is not repeatable.** The two runs to 2050 disagree. Something in
  the run reads what is not the rules and the island (a clock, a thread, an
  environment variable, a hash map walked in its own order). That is a fault
  whatever the file says, and no hash may be taken until it is found.
- **The run to 2050 does not pass through the run to 1900.** Its months or its
  news up to 1900 are not those of the run that ends in 1900: something reads
  the year the run will end before that year comes. The game is played on to
  2050, so the played century must not depend on how far the run goes. (The
  news is set side by side without the chronicle's marks of what a history
  would keep, `Event::kept`: those are made with the whole run in view, decade
  by decade, and the year 1900 goes with the 1890s in a run that ends there and
  with the 1900s in one that goes on, so one item of 1900 is kept in one run
  and not the other. That is the history's reading of the news, not the news.)
- **The game moved.** The runs agree with each other and not with the file.
  The message names which of the seven moved.

Unlike the port's, these hashes are meant to move. The game's rules are
changed on purpose often (every stage of the rules pass and of the run to
2050, a rule mended or retuned, a new rule switched on), and the economy, the
news and `run.bin` move with them. That is what the test is for: a change
that says it moves nothing must leave all eight as they were, and a change
that moves the game says so and anchors them again. A change to the books
alone moves `books`, `run.bin` (which carries the books' tables) or both, and
never `economy` or `news`: the economy moving when only the books were touched
means the books touched the economy. (Moving the books' note of the works'
dividends below the families' take of theirs moves `run.bin` alone, since
those tables are not among the monthly `acct_` columns.) Like the port's,
these hashes are this machine's (see "What worries me").

**Anchoring them again.** On the tree that will land: the branch with the
newest main merged into it as its last step, or the merge into main itself.
Two branches that both move the game must meet here, and the hashes of
either alone are nobody's. In practice the builder of a stage takes them on
the branch, so that its commits pass their tests; whoever merges main into it
afterwards (its checker, or whoever lands it if main has moved again) takes
them once more after the merge, and that is the commit that counts. When
two such branches have both changed `golden_game.txt`, the merge conflicts on
its lines: settle the conflict with either side and then take them again on
the merged tree. Taking them again is one command:

```bash
ISOSTATE_GOLDEN_WRITE="what moved them and why" cargo test -p isostate-core --test golden_game -- --nocapture
```

It runs the three runs, checks that they are repeatable and that the run to
2050 passes through the run to 1900, writes the seven hashes into
`tests/golden_game.txt` with the reason added as the newest `# Taken:` lines,
below the earlier ones (the file keeps every reason it was ever taken for, as
its history; until 20 November 2026 the command wiped that history to its
own line, and what it wiped was put back from git), prints which moved, old
and new, and passes. It never writes when either check fails.
(`ISOSTATE_GOLDEN_PRINT=1` instead checks nothing and stops with the seven
lines, for pasting by hand.) Who may do it:
whoever builds, checks or lands a change that moves the game's rules on
purpose, or what `run.bin` records for the viewer (the works' site planner
moved `run.bin` alone by writing each good's price into its header), as
above, and only after the sweep under the game's rules has been read and
what moved is what was meant. A stage behind its own switch can show that
nothing else moved: with the switch turned off in `game_params()` for one run
(and not committed so), the test must pass on the old seven. Who may not: a
change that claims to move nothing. If the test fails on such a change, the
change is wrong, not the file. The commit that anchors them says, in its
message, which of the seven moved and why ("1901-2050 economy, news and
run.bin: electricity, which comes after 1900"), as the `# Taken:` lines do.

To see what moved,

```bash
ISOSTATE_GOLDEN_DUMP=some/dir cargo test -p isostate-core --test golden_game
```

writes `game6/` (the run to 1900) and `game6-2050/` (the run to 2050) under
that directory, each with the three files of its run, before and after, for a
diff.

The test makes three runs of the game at once, two of them to 2050, which
costs about one run to 2050's time on a machine with cores to spare: 83 to 86
seconds in the dev profile on 19 November 2026, with another helper's tests
running beside it (a run to 1900 alone took 52 to 76 seconds before the second
era was added), and 70 to 76 seconds on 20 November with a heap for each of
its threads (`src/heaps.rs`). It is in the default run, like the port's, and not behind
`#[ignore]`: a guard that is skipped by default guards nothing. The dev
profile and the release one give the same seven hashes, since the core is
optimised in the dev profile too (`Cargo.toml`).

What it cannot see: another seed, another number of families, a player's
orders (`tests/families.rs` and the viewer's `live_tests.rs` hold those), and
the old island, which the game's rules left on 19 November 2026 (seed 7 is
on the new land since; `tests/save.rs` holds one saved game on the old
island to the island it reached that day). What it sees and cannot
explain is the order of the month: a hook moved across a read or a write of
another fails it without saying why. That order is written down in the
comment above `econ::step_month`: what is read from last month, the order the
four takers of hands come in, and where chance enters.

## Running the tests

```bash
python crates/isostate-core/run_tests.py
```

runs the core's tests: the same programs `cargo test -p isostate-core` builds
(the library's own tests, the command's, one for each file in `tests/`, and
the doc tests), built by cargo in the same way, but all at the same time, each
in the package's folder as cargo runs it. It prints a line for each program as
it finishes, the whole output of any that failed, and a last line counting the
tests; it ends with a non-zero exit code when the build or any program failed.
Anything after `--` is handed to every program (`-- --nocapture`, or a test's
name to run only the tests whose names contain it).

It builds in the dev profile, which optimises the core as the release one
does (`Cargo.toml`) and adds debug assertions and overflow checks: more
checking, not less, and the same golden hashes. Its build is the one the
viewer's build makes anyway, so after a change to the core it is seconds, not
the release profile's five to seven minutes of one-core link-time
optimisation. `--release` builds as before. On 19 November 2026, on the same
shared machine, `cargo test -p isostate-core --release` took 19 minutes 30
seconds (7 minutes 28 seconds of build, then about twelve of the programs one
after another), and this took 5 minutes 6 seconds (95 seconds of build,
including the core's few dependencies, then 210 seconds, about as long as the
longest program, `tests/edits.rs`), with every test passing in both. One file
alone is `cargo test -p isostate-core --test money`.

## The sweep

```bash
cargo run --release -p isostate-core -- sweep --out DIR [--seeds 1-8 | 1,2,5] [--rules port|game]
    [--families N] [--end-year Y] [--set key=v1[,v2...]]... [--params FILE] [--content FILE]
    [--jobs N] [--compare OTHER_DIR] [--standing] [--quiet]
```

Every seed is run against every combination of the `--set` values, a few runs
at a time on their own threads (half the machine's cores unless `--jobs` says
otherwise). Eight seeds to 1900 take from 16 seconds to half a minute on this
machine, depending on what else it is doing, and the numbers do not depend on
how many run at once (`--jobs 1` and `--jobs 8` write the same files, the
seconds aside). Each thread takes its memory from a heap of its own
(`src/heaps.rs`, since 20 November 2026): with the process's one heap the
threads queued on it, and 24 islands to 2050 at once took 439 seconds a run
with about six processors busy; now 80 seconds a run with 23 busy, against 58
alone (what is left is the cores slowing each other, not a queue). A run that panics or fails the monthly money check is reported
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

A setting that has been retired is refused by what it became, the same way
from `--set`, from a `--params` file and from the readers in code
(`params::RETIRED`; stage 5 of the pass over the rules made sixteen settings
that moved nothing into constants, and folded two pairs that were one number):

```text
--set canals.max_projects: canals.max_projects is no longer a setting since stage 5 of the pass over the rules (docs/designs/rules-pass.md, section 4.5): it is the constant canals::MAX_PROJECTS, 12 canals digging at once
```

The `params.json` of a game's run made before stage 5 names them too (a
mechanism that is on is written out whole), so given back with `--params` it
is refused in the same words. Take out the lines it names (setting
`canals.hurdle` or `mines.open_at` as the refusal says, where `induced` was not
1 or `payback` was not 4) and the run is the one it was.

It writes into `DIR`, replacing a sweep that is already there (so the same
command makes a baseline again):

- `annual.csv`: one row per run and year. The run's number, its seed, the
  swept values, the year, then every observer series averaged over the year's
  twelve months.
- `facts.csv` and `facts.json`: one row or object per run, its stylised facts
  (below). An empty cell or a null is a fact with no value, as when a railway
  was never built.
- `summary.txt`: the table it prints.
- `standing.txt`: with `--standing`, the standing check (below).
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
of the 256 facts both have, the seconds aside, 241 differ and 12 of those are starred
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
  works_founded                        11,879    8,655   +3,224 (+37.2%)      8/0  *
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

### The standing check

Every stage of the run to 2050 changes rules that the played century, 1750
to 1900, runs under too. The standing check asks whether that century still
stands where it stood. It is twelve facts of the island to 1900, each set
against the spread a reference has across eight islands:

| | |
|---|---|
| people | `pop_1800`, `pop_1850`, `pop_1900` |
| what a wage buys | `real_wage_chained_1850_vs_1750`, `real_wage_chained_1900_vs_1750` |
| towns | `towns_1900`, `urban_share_1900` |
| the railway | `rail_km_1900`, `rail_first_year` |
| who is paid | `income_share_work_1850`, `income_share_work_1900`, `income_share_land_1900` |

Every stage runs it, with its own switch off and on, and prints the table at
the top of its page:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set STAGE.enabled=false,true --standing --out DIR
```

(A change that has no switch of its own runs the game once with `--standing
--compare` an earlier sweep made the same way, which is then the reference.)
After the runs and the usual tables it prints, and writes to `standing.txt`
beside `summary.txt`, a first line for each later setting, `standing check: N
of 12 inside`, and a table: each fact's mean under the reference (the first
setting) with its [least .. greatest] across seeds, then each later setting's
mean and one word.

- **inside**: the mean lies within the reference's least and greatest.
- **moved**: every seed of one arm lies on one side of every seed of the
  other, as the comparison's star has it (four runs or more a side). The rule
  moved this fact on every island; the stage's page must say that it meant
  to.
- **OUTSIDE**: the mean has left the spread, though the seeds have not stood
  apart.

Only inside passes, and 12 of 12 is the pass. A stage with a fact moved or
OUTSIDE says so in its page's first paragraph, and waits if the move was not
meant. `last_town_founded` is shown beneath the twelve, as "also", and not
counted: it leans with anything that changes what carriage costs. The check
changes no run. `src/standing.rs`.

**The check is a gate.** When any of the twelve is not inside, the sweep ends
with an error (exit code 2) after it has written and printed everything, and
its last lines name each such fact: the setting, its mean, the band (the
reference's least and greatest across its islands) and each island that lies
outside the band, with its value:

```text
the standing check failed: 3 of the twelve not inside
  pop_1850, transition.z_half=2: mean 336,572 is outside the band [347,461 .. 387,834] of game rules, 6 families across its 8 islands (OUTSIDE: the islands overlap the reference's); outside the band: island 1 at 341,554, island 2 at 325,388, ...
  pop_1900, transition.z_half=2: mean 422,312 is outside the band [637,660 .. 710,550] of game rules, 6 families across its 8 islands (moved: every island apart from the reference's); outside the band: island 1 at 390,776, ...
  urban_share_1900, transition.z_half=2: mean 0.501 is outside the band [0.457 .. 0.489] of game rules, 6 families across its 8 islands (OUTSIDE: the islands overlap the reference's); outside the band: island 1 at 0.509, island 4 at 0.517, ...
```

(That is the gate below with `--set transition.z_half=2`, half the demographic
transition's income, set against main's sweep on 19 November 2026; it ended
with exit code 2.)

A fact with no value on either side fails too, since it cannot be set against
its band. Run on to 2050 against main's sweep, the check guards the played
century at every landing, and the run to 2050 is checked by the same command
(no run may fail, and money is checked every month to 2050):

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --standing --compare MAIN_SWEEP --out DIR --quiet
```

The eight islands to 2050 took 88 seconds on 19 November 2026. Main's sweep
may end in 1900 or in 2050: the twelve are facts of 1900 and before, which a
run to 2050 has under the same names and with the same values (the game's
golden test holds, for seed 7, that the run to 2050 passes through the run to
1900). `last_town_founded`, shown beneath them and not counted, is not like
for like between a run to 2050 and one to 1900, and reads OUTSIDE there.

**Growth after 1900.** Beside the twelve the check reports what the design
asks of the century after 1900, from facts every run to 2050 has, island by
island, and then their mean across the islands:

| what is asked | read as | met when the mean is |
|---|---|---|
| the real wage grows 1 per cent a year or more from 1900 to 1970 | `(real_wage_linked_1970 / real_wage_linked_1900)^(1/70) - 1`, per cent a year | 1.00 or more |
| and is no lower in 2050 than in 1970 | `real_wage_linked_2050 / real_wage_linked_1970` | 1.00 or more |
| the share of hands on the land is lower in 2050 than in 1900 | `farm_share_2050 - farm_share_1900` | below 0 |

The first two rows are one fact, the wage's; the third is the land's. The
wage is the linked real wage, `real_wage_linked` ([measures.md](measures.md)):
the mean wage over a cost of living whose basket is held five years at a time,
each link's basket and prices the means of the five years before it, and towns
weighted by what was bought; its facts `real_wage_linked_Y_vs_1750` are each
the mean of the nine years round the year (the last nine for 2050) against
1750's. Until 20 November 2026 the rows read the chained real wage,
`real_wage_chained` ([engel.md](engel.md)), single years; it is printed beneath
them, not counted, and read in their place by a run that does not keep the
linked one. The test that chose between them, and the three numbers under each
on islands 1 to 8 and 9 to 16, are in [measures.md](measures.md). The share of
hands on the land is `farm_share_Y`, the land's share of employment.

They are known to be missed today, so by default they are printed with their
values and do not fail the sweep, whatever the twelve do. `--growth` makes a
miss fail it, for the stage that mends them; with `--growth` a run that ends
before 2050, and so has no value, fails too. `--growth` needs `--standing`.


### The facts

All of them are worked out after the run from what it already keeps (the
observer's series, the yearly snapshots, the final state); the simulation's
loop is not touched. Two things nobody keeps, the towns' growth against the
cap and the land under each use, are noted between years by the sweep's own
loop, and a few that only a month shows (a town's wage against the middle
town's, the hands at work against the people) between months
(`facts::run_year_noting`). The facts are computed from the content, so another content file works:
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
snapshot and the final state of a run to 1790. The facts the mechanisms
added since (below the table) are in neither check.

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

Those are the facts the sweep began with, which every run has. One more is
there only when it happens: `works_table_full`, the year the works' table
filled (all `F_max` of its places taken; no works can be founded after, and
the summary says so in a line beginning `FULL run`). Each mechanism
of the game's rules has since added facts of its own, under the same headings
and one more, Canals; what each means is on the mechanism's page and at the
top of `facts.rs`. Some have a value under the port too (the workings, the
coal's price spread), so as run on 12 November 2026 a sweep of the port has
some 280 facts and a sweep under the game's rules some 450. In brief:

| Heading | Facts added by the mechanisms |
|---|---|
| People and towns | how fast towns grow (`town_growth_pct_p10`, `_p50`, `_p90`, `town_growth_near_old_cap`), the towns founded late and on what (`late_towns`, `late_towns_coal` and the other kinds, `late_town_people_median`, `_largest`), `emigrants` ([people.md](people.md)) |
| Work and wages | `mine_share_Y`, `grain_output_Y_vs_1750`, `real_wage_chained_Y_vs_1750`, `real_income_Y` and the budget's shares `budget_NEED_Y` ([engel.md](engel.md)); `baskets_per_head_Y`, `goods_per_head_Y`; `carting_share_of_output_Y` and `carters_share_of_hands_Y` ([carting.md](carting.md)); the island's income account, `income_share_work_Y`, `_works_Y`, `_merchants_Y`, `_land_Y`, `income_pay_share_Y`, `income_ownership_years_Y`, and the books' own checks `books_goods_worst` and `books_income_worst` ([accounts.md](accounts.md)) |
| Railways | the length every ten years from 1830, how many decades it grew, the towns and traffic it reaches, the price of rails, what the lines cost and earned, how late they ran, and the lines by what they serve (`rail_lines_trunk`, `rail_lines_to_KIND`, `rail_km_share_branches`) ([railways.md](railways.md)) |
| Land | the workings, by kind, and how much of a cell they hold (`workings_Y`, `workings_KIND_Y`, `workings_share_of_cell_Y`, `workings_most_of_cell_Y`), their hands (`hands_KIND_Y`), output a head and price in wages of stone, clay, ore and coal, and `royalties_share_of_output_Y` ([mines.md](mines.md)) |
| Prices and cycles | `output_swing_grain`, `price_ac1_grain`, `output_ac1_grain`, `real_wage_worst_year`, `real_wage_swing`, `famine_years`, and how grain, its price and the wage answer to the harvest that year and the next (`harvest_sd`, `grain_output_on_harvest` and the rest) ([harvest.md](harvest.md), [prices.md](prices.md)) |
| News | the news of the newer kinds: `news_canal`, `news_canal_plan`, `news_dearth`, `news_harvest`, `news_region` |
| Canals | the coal's price spread between towns and its share of warmth and heat; when the first canal was begun and opened and how long before the railway; length, traffic and haul; what the canals carry; the lines by what they serve; what they cost and earned, and the most digging and navvies at once ([canals.md](canals.md)) |
| (the pass over the rules) | the facts its card reads: `pop_Y_vs_1750`, the towns' gain at the end, spending a head over the chained cost of living by decade, the five worst years, timber's swing, the tools' dear years against the years about them, hungry months after a district, wild wages and overworked towns by the town-month, the diggings without the carters, the carters between towns and a unit of weight and path cost, closing by age, the works near their market ([scorecard.md](scorecard.md), "The facts added for the pass over the rules") |

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
  hundreds of times (some 250 facts in a comparison of two port sweeps, some
  450 under the game's rules): `8/0` comes up by chance once in 128, so in any
  comparison two to four such rows may mean nothing, and `7/1` means little. For
  anything close, run more seeds.
- **The golden hashes are this machine's.** They were taken on Windows with
  the MSVC toolchain. Logarithms, exponentials and powers come from the
  platform's maths library, so another system may round a last digit the
  other way, and over 150 years that grows into a different run. Not tried:
  if the golden test fails on a fresh machine with no change to the code,
  suspect this first.
- **The golden test watches the economy, not the map.** It hashes the monthly
  series and the news. A change that only moves what `run.bin` records for
  the viewer (a map layer, the works' table) passes it. The game's golden
  test hashes `run.bin` as well, but only under the game's rules.
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
- **The search of PLAN.md 1.3 is not here.** The scorecard
  ([scorecard.md](scorecard.md)) gives the pass over the rules' twenty aims
  with their tolerances and a loss over them; nothing searches it yet.

## Mechanisms under the game's rules

One line each as they arrive: the name, its page, its switch.

**Switched off after the core's table.** Every so often the core's table
(`isostate-core ablate`, `docs/core.md`) turns each rule off in turn, and a
rule it finds idle is switched off under the game's rules and kept in the
code, with a line in its page saying since when and why, unless its page
states a behaviour a player sees, or it guards what no fact of the island can
show (the books, a run that would stop, one town's year). The table of 20
November 2026 found thirteen idle. Off since: `towns.mean_hands`, and
`commons.carry_debt`, which the hands' rule now covers. Kept on, each with
its reason in its page: the watchers (`chronicle.enabled`,
`accounts.enabled`, `accounts.works`, which the inspector reads,
`gauges.enabled`, `woods.record`); what is drawn (`crossings.enabled`,
`ground.reach`); and the mends a player meets or that guard what no fact
shows (`harvest.rent_as_paid`, `works.spread_first`, `works.keep_order`,
`towns.donor_wage`).

- **prices**: merchants mark prices with the stock they hold and stop buying
  into a glut, which ends the false cycles in iron and on the farms.
  [prices.md](prices.md), `prices.enabled`.
- **engel**: past a reference income each further shilling of a household
  leans from the content's shares toward a richer set (less food and warmth,
  more comfort), and the observer keeps a chained cost of living and real
  wage beside the fixed-basket ones. From 1901 the richer set itself leans,
  a seventieth a year to 1970, from shelter and clothing to comfort (a stage
  of the run to 2050). [engel.md](engel.md), `engel.enabled`, `engel.lean`.
- **machines**: farm and quarry machinery is bought where it pays, machines
  stand for years and get better with the steel poured, and a converter and
  a steel rail that pay. [machines.md](machines.md), `machines.enabled`.
- **the ceiling's ramp** (a stand-in): from 1901 the ceiling on the land
  machines' quality rises a tenth a year from 4 to 9, standing for the
  generations of farm and pit machinery the content does not have, so that
  the wage grows 1 per cent a year from 1900 to 1970. [machines.md](machines.md),
  "The ceiling's ramp", `machines.ramp` and `machines.ramp_top`.
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
- **shelves**: when a district changes town, its merchants' stock, their cash
  and their measure of sales go with it to whichever town takes it, as its
  people do, so that a town that took a district does not open January with
  the shelves of the town it was. [shelves.md](shelves.md),
  `people.shelves_go`.
- **towns**: town ground that is kept. A town keeps the cells it builds on and
  grows outward from them along its roads, taking in the works beside it, so
  the island's town ground is some five times as wide in 1900 as in 1750 and
  a cell that joins a town stays. [towns.md](towns.md), `towns.enabled`. Its
  lesser switch `towns.mean_hands` (the year's mean of works' hands, not
  December's) is off under the game's rules since 20 November 2026, when the
  core's table found it idle, and kept in the code.
- **mines**: a quarry, pit or mine is a working: a pit head with the lease
  of the deposit within its reach, sunk at a cost, paying a royalty to the
  cells it works under and taking land by its hands, so the diggings hold a
  few cells of the island instead of a fifth of it. [mines.md](mines.md),
  `mines.enabled`.
- **canals**: a canal age before the railways. Somebody surveys a canal once
  enough coal and stone are dug, towns learn from the canals that work, and
  promoters dig where the carting saved pays: a layer of water over the map
  with wharves, locks that cost stone and navvies who are hands, until the
  railway takes the traffic. [canals.md](canals.md), `canals.enabled`.
- **land** (on since 19 November 2026; `land.enabled=false` for the old
  island): an island made by uplift
  and erosion in place of the port's noise, with rain, lakes, the flow of
  every reach, river courses that meander and cut off, estuaries that carry
  boats to the sea, and resource fields that come from the land.
  [land.md](land.md), `land.enabled`.
- **tree**: a deeper production tree, laid over the port's content from
  `data/game.toml` up to a stage. Stage 1 is the building trades, sawn timber,
  lime, beer, pottery, town gas, soap, glass and paper, with the later
  generations of iron, tools, machinery and bricks; where a market is short,
  plant on order waits behind the day's trade. Stage 6, the game's, adds the
  first content after 1900: electricity, which is made in each town that
  wants it and does not keep, and electrical goods, with their two works.
  [tree.md](tree.md), `tree.stage` (0 is none of it; 1 is the island before
  the content after 1900).
- **accounts**: books that say who makes, moves, buys and earns what: a
  year's books for every town and every works, and the island's income
  account (work's share, the owners' shares, ownership in years of wages).
  They watch and never touch the economy. [accounts.md](accounts.md),
  `accounts.enabled`.
- **gauges**: read off the books each year: of each good's price, what
  comes to rest as wages, rent and the owners' part through its whole chain;
  work's share of income net of the plant worn out, and of spending; a floor
  under it; the labour inside a machine; what a wage buys of made goods and
  of the land's; who owns the works and the land. They watch the books and
  never touch the economy, and need the books on. [gauges.md](gauges.md),
  `gauges.enabled`.
- **commons**: under the commons the common fund in debt pays no dividend
  until its rents repay it, and charges nobody, where the port shares out its
  debt as a levy on every household. Under the landlords it does nothing.
  Off under the game's rules since 20 November 2026: the hands' rule spares
  the fund the debts it was written for. [commons.md](commons.md),
  `commons.carry_debt`.
- **roads** (off under both sets of rules: a probe's prototype): roads,
  turnpikes and bridges that somebody pays for, a parish out of its rate and
  a turnpike trust out of its tolls. [dx-roads.md](../probes/dx-roads.md),
  `roads.enabled`.
- **hands**: a town cannot hire hands it does not have. When even the
  highest wage the month allows leaves more hands wanted than the town has
  to spare, the wage stays there and every employer in the town is cut by
  the same share to the hands there are, and carries forward what it got.
  [hands.md](hands.md), `hands.ration`.
- **money**: the month's new money is reckoned on all the money there is
  when the month begins, at 0.0175 a year, so that it grows 1.764 per cent a
  year on every island, the rate today's islands had, where the port's count
  left the month's incomes out and the rate wandered. [money.md](money.md),
  `money.true_total`.
- **goods that spoil**: a good of which more than a tenth of a stock rots in
  a month (services) carries no stock markup and is priced at its anchor, so
  the towns that bring services in no longer lose what rots on their shelves
  and the merchants no longer owe two fifths of the money by 1900.
  [prices.md](prices.md), "Goods that spoil", `prices.no_spoil_markup`.
- **a crumb is no glut**: a stock under a hundredth of a month of the works'
  make at full time does not turn the works away, so a town whose sales
  stopped, its stock decaying for ever and never reaching nothing, is no
  longer frozen as the worst glut. [prices.md](prices.md), "A crumb is no
  glut", `prices.crumb`.
- **the price of land**: land is priced on the rent it paid, not on an
  average year's rent worked out by adding back the crop the weather took,
  which put it a tenth too high after a bad harvest and a tenth too low
  after a good one. [harvest.md](harvest.md), "The price of land",
  `harvest.rent_as_paid`.
- **the works of 1749 spread out**: the works a game starts with stand on
  cells of their own, where the port put up to six of one trade on a town's
  one best cell as twins with one history. [works.md](works.md), part G,
  `works.spread_first`.
- **orders paid for first**: a works keeps back what its order for plant
  still costs before it pays a dividend, so the money for an extension is not
  paid out to its owners while the plant is still to come and the works then
  closed for the debt. [works.md](works.md), part H, `works.keep_order`.
- **longrun**: the bounds on every price and on the wage ride on a chained
  cost of living the simulation keeps for itself (the ceilings on the higher
  of it and the wage), and from 1900 the freight does too, so that the
  island can be run on to 2050 with no price held at a bound fixed in 1750's
  money. It does not make the island live well after 1900.
  [longrun.md](longrun.md), `longrun.enabled`.
- **fewer children as towns live better**: each town's income term is cut
  as its townsfolk come to live well (a generation behind), and crowding
  costs fewer lives as drains come with income. It slows the people on an
  island that stops growing richer, and does not level them off.
  [transition.md](transition.md), `transition.enabled`.
- **owners**: whoever put money into a works no family owns holds a share of
  it and is paid by it, so the landlords, who put up part of every works the
  town's pool pays for, are paid their part of its dividends and of its cash
  when it closes, where under the port every penny went to the households.
  [owners.md](owners.md), `owners.enabled`.
- **tasks**: every hour of a works' labour is of one of five classes of task,
  and a machine for a class takes the tasks at which a hand is worth fewer
  machine units than the town's wage buys; works are built with the town's
  machines and move toward them as their orders for plant are filled. The
  works' machine is the electric drive, made of machinery and electrical
  goods and run on electricity, laid with stage 6 of the tree and found from
  1901, so nothing in the played century moves; it holds nothing in a town
  without a station. Under `tasks.land` the land's units pay their share for
  work to hands and machine units alike, and a stand-in, a coal-fired hoist,
  hauls at the pits and quarries (in the works too with the tree at stage
  1); with the content after 1900 the tractor hauls on the fields, the
  combine tends them and the cutter hauls at the pits. A ceiling on yield
  (`tasks.yield_cap`) and a control that holds the machine still
  (`tasks.freeze_from`) are for sweeps. [tasks.md](tasks.md),
  `tasks.enabled`, `tasks.land`.
- **computing** (a thin slice of the run to 2050): office computing, machines
  that reckon, take clerks' tasks in the works from the easiest, found from
  1960 on an island that has made 5.4 electrical goods a head, made of
  electrical goods and run on electricity, a twentieth better each year. No
  computing good stands behind it yet. [computing.md](computing.md), `tasks.computing`.
- **woods**: managed woodland, an invention of the tree's stage 6 found from
  1895 and spreading as every invention does, grows the woods back three times
  as fast where it is known, so that the island no longer runs out of timber
  after 1950; and the woods' cover and timber's price in months of the wage
  recorded (`woods.record`). [woods.md](woods.md), `woods.enabled`.
- **crossings** (acts on the new land only, and does nothing on the old
  island): crossing water costs a ford's charge by its width, or a ferry's
  over 25 m, and a parish builds a bridge out of its rate, with its own men,
  where the carting it saves pays for it; the cells of a valley meet on one
  bank, a way goes round the meanders its straight line cuts, and the
  picture draws a bridge, a ford or a ferry only where the simulation has
  one. [crossings.md](crossings.md), `crossings.enabled`.
- **ground** (a stage of the run to 2050; acts only from 1901): a town's
  room is counted by what is built, each cell built up as its rent rises,
  and its house rent clears that room, blended in from 1901 to 1920; a cell
  joins a town only within five cells of its centre, eight once the tramway
  is found and the town has 30,000 townsfolk. One kind of household, no car
  and no belt unless one is set. [ground.md](ground.md), `ground.enabled`,
  `ground.room`, `ground.reach`.
- **linked** (watches only): a cost of living, the real wage on it and what is
  made, each linked every five years on the means of the five years before, so
  that they do not drift with timber's price; the news's slump clock reads what
  is made, the viewer's charts and the growth after 1900 read the linked wage.
  [measures.md](measures.md), `linked.enabled`, `linked.link`, `linked.window`.
- **pay** (watches only; acts from 1900): each class of task's pay, reckoned
  from the hands the island plays and not played: demand at one wage bends to
  pay, hands follow the work, and pay is scaled so that the hands' mean is the
  one wage; with real pay on the linked wage and the schooled over the plain,
  and the two dials in the Rules tab and the Work panel. [pay.md](pay.md),
  `pay.watch`, `pay.bend`, `pay.follow_years`.
- **city** (a stage of the run to 2050; acts only from 1901): people follow
  jobs, the wage and the works' new hands rather than income a head; every
  works is more productive in a larger town, a doubling worth 5 per cent; and
  room is wanted less than in proportion as it dears, so a city's burden can
  rise. Three switches, all on. [city.md](city.md), `city.follow_jobs`,
  `city.gather`, `city.dear_room`.
