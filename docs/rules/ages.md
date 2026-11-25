# The people's ages, reckoned beneath the played totals

Added on 20 November 2026. Not in the Python. The rule `ages` (`src/ages.rs`) only watches: nothing in the
economy reads it. On islands 1 to 8 run to 2050, every one of the 3,890 facts the island had before gives
what it gave before, and the game's golden test holds the economy and the news in both eras.

A town's people are one number in the game. A pension, a school, a university or a count of who works all
need to know how many of them are children, of working age and old. So this keeps, beneath each town's
number, seventeen five-year groups (0 to 4, 5 to 9, and so on to 75 to 79, and 80 and over) whose sum is the
town's people as the island plays them, in every town at every year's end. It is a reckoning, not a run: the
island's people go on being made by the game's own rules of births, crowding, dearth, sailings, migrants and
districts ([people.md](people.md), [transition.md](transition.md)), and the ages are fitted underneath them.

## What is played and what is reckoned

**Played**, as before: each town's people, year by year. **Reckoned** beneath them, town by town, once a year
as the played rate meets the town's people (`people::growth`):

1. **Those who took ship** leave first, as many as the played rule says left the town that year, drawn three
   times as heavily from ages 15 to 34 as from the other ages (`ages.movers`).
2. **Deaths by age.** Each group dies at a yearly hazard. A broad group's hazard (children under 15, the
   working ages, the old) is its rate at a bare living, `ages.hazard` (0.033, 0.0165 and 0.105 a year),
   lowered by food and by medicine; each five-year group takes it times a fixed profile (`PROFILE`: the first
   five years and the last are the dangerous ones, middle age more than youth). Food is how far the town's
   baskets a head stand above the people's resting income, from nothing there to full at six times it.
   Crowding adds the played rule's deaths in crowded towns (less what drains and clean water save under the
   transition), shared among the broad groups by `ages.toll`; and a dearth adds the played rule's dearth
   deaths, twice as heavy under five and over 64 (`ages.dearth`).
3. **A fifth of each group moves up** to the next; the last group is open.
4. **Births are the remainder**: what the played total needs once the leavers, the deaths and the moves are
   counted. They go into the youngest group.
5. **Where the remainder would be less than nothing** (more died or left than the played total allows), the
   shortfall is taken as more people leaving, drawn as movers are, if the played rule says people took ship
   from the town that year; otherwise as more deaths, twice as heavy on the youngest and the oldest. It is
   counted (`ages_short_town_years`, `ages_short_people`, and the part taken as leavers,
   `ages_short_left_share`). On islands 1 to 8 to 2050 it never happened: in no town in no year did the
   deaths by age and the sailings come to more than the played rate took away.

**People who move take their ages and their schooling.** Migrants between towns (`people::migration`) are
drawn three times as heavily from 15 to 34. A district that changes market (`peopling::move_with_cells`)
goes whole, as it is: its households did not move, the market's edge did. A new town's first people, who come
with their districts in the year it is founded, are drawn as movers are. At each year's end every row is
held to its town's people (`close_year`), and the largest mismatch found there before anything is put right
is kept as a check: on islands 1 to 8 to 2050 it is under one part in a thousand million million
(`ages_drift`, 8.9e-16 at most), which says that every way the game moves people has been followed. Under
the port's rules, thrown on by hand, a town founded there takes its people some way the watcher does not
follow (`people::resettle_new_towns`), and its row is given the island's shape at the year's end.

**At the start**, after the warm-up, each town's row is the stable shape of its first year's hazards for a
people growing at the played rule's income term, scaled to its people.

**Medicine is a stand-in.** The content has no inventions of medicine yet. Until it has, three waves are
known by the year, as history found them: sanitary reform and vaccines (half known in 1900, spreading over
some fifty years), antisepsis and antibiotics (1945, twenty years), modern medicine (1990, sixty years). A
town reaches a wave as far as its food is full, so a poor town gains less of it (`MEDICINE`). When the content
gains the inventions, the town's knowledge of each takes the place of the year.

**Schooling, reckoned.** Each cohort's years of school come from its town's income when it was of school
age: those who turn fifteen take `2 + 3 * ramp(y, 3, 8)` years, with `y` the town's baskets a head averaged
over about the last ten years. Two years at a bare living, five where eight baskets a head can buy three
more; until a state pays for schools nothing else buys them. They keep their years as they age and move. The
figure kept is the mean years of school of the people aged 15 to 64. Since the state's purse
([state.md](state.md)) a cohort takes the years the treasury paid for while it was at school where that is
more; the ages are then read by the purse (its pupils and pensioners) and are no longer only a watcher after
1900.

## What a later piece reads

- `ages::town(&st, t)`: town `t`'s people under 15, aged 15 to 64 and over 64, and the working ages' mean
  years of school, as the last year's end left them (None while the watcher is off or the town is empty).
- `st.ages.rows[t]` and `st.ages.school[t]`: the seventeen groups and each group's years of school.
- `st.ages.years`: the island's year by year (`AgesYear`: people, the groups, births and deaths a thousand,
  life at birth, the three shares, the years of school, the shortfalls).
- The series, the books' own, a year's figure in its twelve months: `ages_g00` to `ages_g16`,
  `ages_births_k`, `ages_deaths_k`, `ages_life`, `ages_under15`, `ages_working`, `ages_over64`,
  `ages_school`.

## Its numbers

| name | game | what it is |
|---|---|---|
| `ages.watch` | on | the switch; fixed when the island is made (`edits::FIXED_AT_START`) |
| `ages.hazard` | 0.033, 0.0165, 0.105 | the yearly hazard of children, the working ages and the old at a bare living with no medicine |
| `ages.food` | 0.75, 0.40, 0.05 | how far full food lowers each broad group's hazard, as `exp(-food * this)` |
| `ages.food_full` | 6 | food is full at this many times the people's resting income |
| `ages.toll` | 1.6, 0.7, 1.0 | each broad group's part of the played crowding deaths |
| `ages.movers` | 3 | how many times as heavily movers are drawn from 15 to 34 |
| `ages.dearth` | 2 | how many times as heavily a dearth's deaths fall under five and over 64 |

The hazards and their answer to food are history's table for a country before medicine, fitted when the
design for the run to 2050 was written to the game's own runs to 1900. Nothing here was tuned to the island:
the remainder makes the totals, whatever the hazards.

## What eight islands show

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --jobs 8 --out runs/ages
```

The mean of islands 1 to 8 under the game's rules on 20 November 2026 (births, deaths and life each the mean
of the nine years round the year; the shares and school that year's), beside history's figures for England
and Wales (Sweden a little behind it in the nineteenth century, much the same by 1950). History's are quoted
from memory and rounded, so read them as bands, not targets:

| | 1800 | 1850 | 1900 | 1950 | 2000 | 2050 |
|---|---|---|---|---|---|---|
| births a thousand | 34.6 | 33.0 | 24.4 [23.4 .. 26.4] | 14.4 [13.8 .. 15.0] | 13.5 | 13.6 |
| history | 37 to 38 | 34 | 29 | 16 | 11 to 12 | |
| deaths a thousand | 23.0 | 20.6 | 13.4 | 11.6 | 12.2 | 12.1 |
| history | 25 to 26 | 22 | 18 | 11.6 | 10 | |
| life at birth, years | 40.9 | 43.5 | 56.8 | 68.1 | 75.5 | 79.0 |
| history | 36 to 40 | 40 | 46 to 48 | 69 | 78 | |
| under 15 | 0.353 | 0.347 | 0.312 | 0.224 | 0.201 | 0.198 |
| history | 0.36 | 0.36 | 0.32 | 0.22 | 0.19 | |
| over 64 | 0.052 | 0.050 | 0.058 [0.055 .. 0.062] | 0.096 | 0.170 | 0.200 |
| history | 0.05 | 0.046 | 0.047 | 0.108 | 0.158 | |
| years of school, 15 to 64 | 2.0 | 2.4 | 4.4 | 5.0 | 5.0 | 5.0 |

The design for the run to 2050 asks for births of 23 to 30 a thousand in 1900 and 15 to 20 in 1950, and about
6 per cent over 64 in 1900.

**Where it sits with history, and where not.**

- **1800 and 1850 sit close**: births and deaths a point or three under England's, life a little longer, the
  shares by age the same. The island of the first century is a country like England's, a little better fed.
- **1900: deaths are five a thousand under England's and life ten years longer.** The island of 1900 is
  richer than the Britain of 1900 (its real wage stands at four times 1750's where Britain's stood at under
  two and a half), so its food is full by then, and the stand-in for medicine gives a fully fed town half of
  sanitary reform and vaccines by 1900. And under the transition its drains are complete from ten baskets a
  head, so its crowded towns cost no more lives than the country; England's towns of 1900 still did. So the
  births, the remainder, are 24.4, inside the design's band at its low edge and five under England's 29: the
  island grows as fast as England did with fewer children born and fewer dying. The share over 64, 5.8 per
  cent, is the design's 6 and a point over England's.
- **1950: births are 14.4 a thousand, under the design's 15 and England's 16.** Deaths are where history has
  them (11.6), so the gap is the played rate: the island's people grow 0.27 per cent a year in 1950 where
  England's grew about 0.4, since the transition's line and the Engel rule's lean after 1900 bring the
  island's growth down early ([engel.md](engel.md), the people's fall). The reckoning only reads the played
  totals, so the remedy is not here: the full rule (below) puts births in the town's own hands and will move
  them.
- **After 1950 the island ages as Europe did**: a fifth of its people over 64 by 2050, a fifth under 15, and
  life at birth 79 years.
- **Life at birth is the same on every island from 1950** (68.1 in 1950, to the tenth): every town's food is
  full by then and the drains complete, so the hazards depend on the year alone, through the stand-in for
  medicine.
- **School stops at five years**: the fallback for schooling before a state lets families buy at most three
  years over the two, and every town's income passes eight baskets a head soon after 1900. A state that pays
  for schools is what will move it.

**A run's cost.** The year's work is seventeen numbers a town; the eight runs to 2050 took 67.7 seconds each
with the watcher off and 67.7 with it on (`seconds`, the same sweeps).

**The first city** (Dunheath on island 7, the largest town in 2050), from the example below: 75,374 people in
1900 with 31 per cent under 15 and 6 over 64, much as the island; 246,583 in 1950 with 70 per cent of working
age, since the migrants who made it were drawn from the young; 322,533 in 2050, a fifth under 15 and a fifth
over 64, as the island.

```bash
cargo run --release -p isostate-core --example ages -- --seed 7 --end-year 2050 --years 1900,1950,2050 --csv FILE
```

## In the viewer

The island half of the Markets panel (key 3) ends with the people by age in the year on the clock, a pyramid
of the seventeen groups in three colours (under 15, 15 to 64, over 64, with their shares above it), and the
shares under 15 and over 64 year by year, both marked as reckoned (`crates/isostate-viewer/src/ages_chart.rs`).
A town's card does not yet say its ages: the record keeps the island's groups, not each town's.

Since 25 November 2026 the ages are read by a rule that plays: from 1901 a town's hands are counted from its
rows, by age, school and pension (`who_works.enabled`, [who-works.md](who-works.md)). The rows themselves are
reckoned as before; what is played is who of them works.

## What the full rule will change

The full rule for ages, a later piece, makes births a town's own: a number of children a woman that falls as
schooling, children who live, town life and the cost of a home take families to limiting births, with
women of 15 to 44 to bear them. Then the town's people are the row's sum, and the played line of births that
falls with income goes. What it changes against this reckoning:

- **Births** come from the town's own reasons, not from the remainder. So the island's people will no longer
  be what today's rules make them; the played century has to be fitted again, which is the risk this watcher
  was built to put off.
- **Deaths** stay much as here: the same hazards, with the content's inventions of medicine in place of the
  year.
- **The remainder** is the measure of what the full rule has to explain: where the reckoned births stand
  far from what the town's own reasons would give, the full rule moves the island's totals.

## What worries me

- **Births are the remainder of everything.** A played rule that moves people for some other reason (a new
  rule of migrants, a town given people by hand in a sandbox) moves the reckoned births with it. The births
  a thousand are as good as the played rates.
- **Deaths in 1900 are a richer island's.** The hazards answer to food on the island's own scale, which
  reaches full by 1900; the stand-in for medicine is history's dates, not the island's knowledge.
- **A fifth up a year blurs the cohorts**: in a five-year group a fifth move up each year whatever their age
  within it, so a baby boom spreads over the groups faster than a real one would. The full rule could keep
  single years where the blur matters.
- **Movers are drawn from 15 to 34, children with them not.** A family that moves takes its children; here
  a migrant is a young adult. It makes the first city's working ages heavier than a city's would be.
