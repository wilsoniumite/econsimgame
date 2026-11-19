# An island that can last three centuries

Not in the Python. `crates/isostate-core/src/longrun.rs`, switched by
`longrun.enabled`: off under the port's rules, on under the game's. It is the
first stage of the run to 2050, and it promises one thing only: that the
island can be run on to 2050 with no price or wage standing at a bound that
was fixed in 1750's money. **It does not make the island live well after
1900.** Run on, the island still goes hungry: people near four million by
2050 and what a wage buys falling back below 1750's, because there is no fall
in births and no technology after steel. Those are later stages. A sweep is
still run to 1900 unless it is given `--end-year 2050`; a new game in the
viewer is played to 2050 unless it is told otherwise ("A game past 1900",
below), and so is the hungry island after 1900 until the later stages land.

## What was wrong

A handful of the port's constants are sums of money fixed in 1750's prices:

- every good's price is held between `price_floor` (0.05) and
  `price_ceiling` (40) times its base price;
- each town's wage is sought between 0.02 and 500 a month;
- `freight` (0.015) is what it costs to cart a unit of weight one unit of
  path, in the delivered prices, the works' and the land's net prices, the
  trade between towns, and every promoter's sum of the carting a railway, a
  canal or a new town would save.

Over the played century they do no harm. Run on to 2050 the money grows
1.76 per cent a year for three hundred years and prices follow it: timber
stands at its ceiling in nearly every month from about 2025, tools come within
a few per cent of theirs, and an island that grows richer than this one (the
rich island, below) has its shops' services at the ceiling in a tenth of all
town-months. A price at its ceiling is no longer a price: the market cannot
say the good is dear, and what is bought of it is set by the bound.

## What it does

The simulation keeps two numbers of its own, each 1 at the end of the
warm-up and set once a year, when December's sales are in:

- **the anchor**, a chained cost of living. Through the year it adds up, good
  by good, what households and landlords bought and the prices they paid (the
  towns' prices weighted by their people). At the year's end it makes one
  Fisher link from last year's quantities and mean prices to this year's and
  multiplies it onto the chain. The anchor is the mean of the chain's last
  five yearly levels. It is the same sum the Engel curve's observer makes for
  the sweep (`engel::record_month`, `docs/rules/engel.md`, "The chained
  index"), made again inside the simulation, because the observer's is never
  read by the economy and must stay so. The two agree to the last digit at
  every year's end (`tests/longrun.rs`).
- **the wage scale**, the island's mean wage weighted by people, the mean of
  the last five years, over its value at the end of the warm-up.

Then:

    ceiling of good g  = price_ceiling * base price of g * max(anchor, wage scale)
    floor of good g    = price_floor   * base price of g * anchor
    the wage's bounds  = 0.02 * anchor  and  500 * max(anchor, wage scale)
    freight            = freight * anchor / (the anchor as 1900 began), from 1900 on

The ceilings ride on the higher of the two because a shop's price is, to
within a few per cent, the wage, so a ceiling riding on the cost of living
alone would stop the shops once the real wage passed some thirty times
1750's. The floors and the freight ride on the cost of living alone.

Each of these is one small function in `longrun.rs` (`ceiling`, `floor`,
`wage_bounds`, `freight`), read wherever the constant was: the month's price
rule and the wage's bracket in `econ::step_month`, the prices a district
brings when it changes market (`peopling.rs`), a new town's prices
(`prices::found_town`, `prices::anchor_follows`), and every place the
freight is read (nineteen, in ten files). With the switch off each returns the constant
itself, by an early return and not by multiplying by one, and the port's
golden test holds.

## Why the freight waits for 1900

The design this stage was built from had the freight ride on the anchor from
1750, expecting the anchor to stand near 0.85 in 1900. It does not. The
island's chained cost of living **halves** over the played century: 0.70 in
1800, 0.50 in 1850 and 0.45 in 1900 (means of islands 1 to 8, the least and
greatest 0.39 and 0.51 in 1900), because the machines cheapen what
households buy and the chain follows what they buy. Carriage riding on that
from 1750 is half as dear by 1900, and the played century moves:

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules game --families 6 --end-year 2050 \
    --set longrun.freight_from=1750,1900,3000 --out runs/longrun-freight
```

| in 1900, islands 1 to 8 | switch off | freight from 1750 | freight from 1900 (the game's) |
|---|---|---|---|
| people | 735,871 [716,484 .. 761,260] | 728,163 | 740,453 |
| real wage, chained, 1750 = 1 | 4.09 [3.80 .. 4.46] | 4.36 | 4.17 |
| towns | 17.6 [14 .. 20] | 14.5 [12 .. 17] | 18.4 [16 .. 21] |
| railway | 2,312 km [2,156 .. 2,569] | **2,121 km** [2,062 .. 2,201] | 2,409 km |
| the last town founded | 1853 [1788 .. 1895] | 1818 | 1861 |

With freight from 1750 the railway of 1900 is below the least island with
the switch off (cheaper carting saves less, so fewer lines pay), and the
founding of towns ends 35 years sooner. So `longrun.freight_from` says the
first year whose freight rides on the anchor, counted from the anchor as
that year began, and the game's is 1900: the played century keeps the
freight it was tuned on, and after 1900 carriage keeps its cost in the
island's own money, as the fixed constant kept it in 1750's. Set to 1750 it
is the design's rule exactly. The year is the end of the played game, and a
later stage that makes the lorry a machine to be bought will take the
freight from there.

## Its numbers

`longrun.enabled` (true in the game); `longrun.anchor_years` 5, the years the
anchor and the wage scale are averaged over, so that a year of dearth does
not lift every ceiling the next; `longrun.freight_from` 1900;
`longrun.standin_growth` false. The constants themselves (40, 0.05, 0.02, 500
and 0.015) are the port's and are not changed.

## A stand-in, to be deleted

`longrun.standin_growth` lifts the two learning caps, the methods'
(`learn_cap`, 4) and the machines' quality (`machines.quality_cap`, 4), to 64.
It exists only so that the anchors can be tried on an island that goes on
growing before anything that would make it grow is built. It is for sweeps:
a live game never runs it (`save::params` turns it off), it does not need
`longrun.enabled`, and it is to be deleted when the first content after 1900
lands. The two caps stay at 4 everywhere else.

## The runs that show it

All under the game's rules with six families, on islands 1 to 8.

**1. Nothing moves with the switch off.** The port's golden test passes. With
`longrun.enabled=false` a game's run is the run before this stage: on islands
1 to 4 to 1900, none of the 624 facts the two sweeps share differs, and the
game's golden test passes on its old four hashes with the switch off in
`game_params()`.

**2. The standing check: the played century stays inside the spread of
seeds.**

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules game --families 6 \
    --set longrun.enabled=false,true --out runs/longrun-1900
```

| in 1900, islands 1 to 8 | switch off | switch on |
|---|---|---|
| people | 735,871 [716,484 .. 761,260] | 740,453 [715,166 .. 778,768] |
| real wage, chained, 1750 = 1 | 4.09 [3.80 .. 4.46] | 4.17 [3.93 .. 4.34] |
| towns | 17.6 [14 .. 20] | 18.4 [16 .. 21] |
| railway | 2,312 km [2,156 .. 2,569] | 2,409 km [2,197 .. 2,584] |
| the towns' share of people | 0.412 [0.401 .. 0.434] | 0.408 [0.394 .. 0.429] |
| work's share of income | 0.691 [0.647 .. 0.715] | 0.698 [0.679 .. 0.719] |
| land's share of income | 0.173 [0.165 .. 0.180] | 0.172 [0.168 .. 0.182] |
| the last town founded | 1853 [1788 .. 1895] | 1847 [1810 .. 1877] |
| the anchor; the wage scale | | 0.47 [0.42 .. 0.54]; 2.01 [1.78 .. 2.20] |

Every mean with the switch on is inside the spread of the islands with it
off. The founding of towns ends six years sooner, as the design expected.

**3. The long run: no price at a bound.**

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules game --families 6 --end-year 2050 \
    --set longrun.enabled=false,true --out runs/longrun-2050
```

| islands 1 to 8, to 2050 | switch off | switch on |
|---|---|---|
| timber: town-months at a bound after 1800 | 0.193 [0.163 .. 0.216] | 0.094 [0.077 .. 0.114] |
| tools | 0.011 [0.000 .. 0.030] | 0.001 [0.000 .. 0.006] |
| charcoal | 0.001 [0.000 .. 0.005] | 0.000 |
| every other good, and the wage | 0.000 | 0.000 |
| people in 2050 | 4,284,966 | 4,367,598 |
| real wage, chained, 2050 | 2.04 [1.87 .. 2.24] | 1.86 [1.47 .. 2.10] |
| real wage on 1750's basket, 2050 | 0.61 [0.51 .. 0.67] | 0.26 [0.22 .. 0.30] |
| the anchor; the wage scale, 2050 | | 1.79 [1.61 .. 2.01]; 3.54 [3.05 .. 4.05] |
| railway in 2050 | 2,688 km | 2,714 km |
| places taken in the works' table | 6,382 [6,139 .. 6,852] | 6,077 [5,747 .. 6,410] |

No good but timber stands at a bound in more than one town-month in a
hundred, and the wage never does. The real wage on 1750's basket falls much
further with the switch on, and that is the switch working: the basket
holds timber at 1750's weight, and timber's price, free of the old ceiling,
climbs from 32 to 112 by 2050 (means of the eight islands), so the
basket's cost goes from 4.9 to 13.1 times 1750's. What households actually
buy, the chained wage, moves far less.

**4. The rich island.** With the stand-in on and fewer births
(`--set longrun.standin_growth=true --set people.growth_top=0.006`), the
island's money wage passes thirteen times 1750's by 1950 and fifty by 2050.

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules game --families 6 --end-year 2050     --set longrun.standin_growth=true --set people.growth_top=0.006 --set longrun.enabled=false,true --out runs/longrun-rich
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules game --families 6 --end-year 2050     --set longrun.standin_growth=true --set people.growth_top=0.006 --set longrun.enabled=false --set price_ceiling=400     --out runs/longrun-ceiling400
```

| islands 1 to 8, to 2050, the stand-in on | switch off | switch on | switch off, ceiling 400 |
|---|---|---|---|
| services made a month, mean of 2001 to 2050 | 84,958 | 92,122 | 95,065 |
| services: town-months at a bound | 0.110 | 0.000 | 0.000 |
| tools | 0.256 | 0.004 | |
| the wage scale, 1950; 2050 | | 13.2; 57.4 | |

With the switch on, services made after 2000 are 3 per cent below the run
with the ceiling lifted by hand; with it off, 11 per cent below.

## The record past 1900

A run's record (`run.bin`, and the live view's years) keeps one byte a cell
and year for when the cell joined its town, for its residents and for its
rent. Those bytes were laid out for a game that ends in 1900: the year in
single steps from 1645 stops at 1900, residents in eights stop at 2,040 to a
cell, and the rent byte runs from a thousandth to a thousand a month in
1750's money. A run on to 2050 dated every cell built after 1900 as built in
1900, and the rent of a town's middle stood at the top of the scale.

So a record has two layouts, chosen by the year its run ends
(`record::TownBytes`):

| | a run that ends by 1900 (every game) | a run past 1900 |
|---|---|---|
| the year a cell joined its town | single years from 1645, to 1900 | steps of two years from 1645, to 2155 |
| residents | in eights, to 2,040 a cell | in sixteens, to 4,080 a cell |
| rent | in 1750's money | over the year's anchor |

A record of the second layout says so in its header (`meta.town_bytes` = 2)
and keeps the anchor each year's rent was taken over (`rent_scale`), so that
the viewer gives the rent in that year's money. A record of the first says
nothing and is the very file it was before this stage, so the game's golden
test does not move, and a record made before there were two layouts reads
as the first. A cell that joined in an odd year of the second layout is
dated to the even year before it.

    cargo run --release -p isostate-core -- run --seed 7 --rules game --families 6 --end-year 2050 --out runs/to2050-7
    cargo run -p isostate-viewer -- runs/to2050-7/run.bin --year 2050

## A game past 1900

A game's last year is chosen when the game is begun, and kept in its save.
Nothing in the rules' parameters says it: `game_params()` is the same with
or without it, so neither golden test moves.

- A new game the viewer begins is played to 2050, or to the year `--to`
  gives (1900 to 2050). The standard pictures (`--shots`) always play a game
  to 1900, so that they and their time are as they were.
- In the core, `save::Game::under_to(rules, seed, families, started, to)` and
  `Game::on_new_land_to` set the parameters' `end_year` before the island is
  made; `Game::under` and `Game::on_new_land` keep the content's last year,
  1900, as before.
- The save says the year (`"to": 2050`) only when it is not 1900. A save
  that does not say it is a game to 1900, so every save made before is
  played as it was. `Save::sound` checks the year the save had reached
  against its own last year: a save of 1950 that does not say it is played
  past 1900 is refused as doctored, as it always was.
- Only rules with the long run on may be played past 1900. Under the port's
  rules, where `longrun.enabled` is off, a last year past 1900 is refused in a
  sentence, whether a game is begun with it or a save says it; so is any
  year outside 1900 to 2050 (`save::last_year_allowed`).
- A game past 1900 keeps its record in the second layout (above), as a run
  past 1900 does: the live view packs its towns by the game's own last year.

Past 1900 the island is the hungry island of "The runs that show it": people
go on multiplying and what a wage buys falls, since there is no content after
steel yet. That is expected until the later stages of the run to 2050 land.

    cargo run -p isostate-viewer -- --new --seed 7 --to 1950
    cargo run -p isostate-viewer -- --sandbox --seed 7 --advance 150

## The works' table

Every works the island has ever had keeps its place in the table (its slot)
for good: the record, the books, the chronicle and the families all name
works by their slot, so a slot is never given to a second works. The table
only fills. It had room for 20,000, which the port keeps; the game has room
for 60,000 (`F_max`). The months now run over the places taken
(`F_count`), not over the whole room, so the room costs some memory and no
time. An island run on to 2050 takes about 6,000 places (5,747 to 6,410
on islands 1 to 8; the fact `works_slots_used`); a table that fills founds nothing more and says so
(`works_table_full`).

## Facts by the decade after 1900

The sweep's facts at marks (`pop_Y`, `urban_share_Y`, `towns_Y`, the
railway, wages, the books, the carting, the mines and this stage's own) are
taken every fifty years to 1900 and every ten years after it, so a run on to
2050 has them for 1910, 1920 and so on (`facts::marks`). A game to 1900 has
the same marks as before.

## Facts added to the sweep

Under "Prices and cycles": `months_at_bound_GOOD` for every good and
`months_at_bound_wage`, the share of town-months from 1801 on in which the
good's price, or the town's wage, stood at its floor or its ceiling (within
a millionth). They are counted whatever the switch, against the bounds the
run is under, so the two arms of a sweep can be set side by side. With the
switch on, `anchor_Y` and `wage_scale_Y` at the sweep's marks, as the year
ended. Under "Works": `works_slots_used`, the places in the works' table
taken by the run's end. The last town founded was already a fact
(`last_town_founded`).

## What the tests hold

`tests/longrun.rs`, seven tests: the switch is off under the port and on in
the game, a port run's `params.json` does not mention it, and a live game
never runs the stand-in; with the switch off every bound, the freight and
both caps are the port's numbers to the last bit, whatever the state holds;
with it on they ride on the anchor and the wage scale as above; the chain is
the observer's to a part in a million million at every year's end, and the
anchor the mean of its last five levels; the freight is the constant until
its year and the anchor's ratio after; the facts are there under either
setting; and a short run with the stand-in conserves its money.

The plumbing has its own: `tests/towns.rs`,
`a_record_past_1900_packs_its_towns_in_two_year_steps` (the second layout's
arithmetic, and the first's is the old arithmetic to the byte) and a run to
1900 writes no `rent_scale`; the viewer's
`a_run_past_1900_reads_its_towns_in_two_year_steps_and_its_rent_in_its_own_money`;
and `facts::tests::the_marks_are_fifty_years_apart_to_1900_and_ten_after`.
A game past 1900 has `tests/save.rs`,
`a_game_to_2050_saved_after_1900_loads_again_to_the_same_island` (island 7
begun to 2050, saved six months into 1906, played again to the same island
to the last digit) and `a_games_last_year_is_kept_in_its_save_and_checked`;
and the viewer's
`a_game_begun_to_1910_plays_past_1900_through_the_thread_and_its_save_says_so`.
The port's and the game's golden tests hold the rest: the works' passes that
now stop at the places taken give the same island to the byte.

## What worries me

- **The rent byte still reaches its top.** Taken over the anchor, a cell's
  rent past 1900 is in the year's own money, but rent grows with the people
  on the land far more than prices do: on island 7 in 2050, 2,226 cells
  have their rent at the top of the byte's scale (3,672 before this stage,
  and 328 in 1900). The record's scale of a thousandth to a thousand a
  month is too narrow for a long run, and a later stage should widen it.

- **The anchor halves over the played century**, and the floors with it. A
  good whose price used to stand at its floor (iron ore, in a few
  town-months on most islands) may now fall further, and that alone gives
  the island another history within the spread of seeds. The ceilings rise
  with the wage, two times 1750's by 1900, so none binds before 1900.
- **Timber still stands at its ceiling** in about one town-month in ten
  after 1800, mostly after 1950, with the switch on: the island has cut its
  woods, and timber's price climbs faster than the wage. The design leaves
  timber aside until the production tree lands it at the ports.
- **The design's harsher test does not match this build.** It moved each
  constant whole from 1750 by a fixed factor, and reckoned the anchor near
  0.85 in 1900; on this build it is 0.45. I have not traced where the two
  part. The game's freight waits for 1900 because of it (above).
- **The rich island does not break as the design's did.** With the ceiling
  at 40 and no anchors, services stand at their ceiling in a tenth of the
  town-months after 1800 and services made after 2000 are about a tenth
  lower than with the ceiling lifted by hand, where the designers' run had
  them fall from over forty thousand units a month to under three thousand.
  Their run was on an older build (before goods that spoil lost their stock
  markup), and I have not rerun it.
- **Eight islands is a small sample**, and by 1900 any change at all moves an
  island as far as islands differ from one another (`towns.md`).
