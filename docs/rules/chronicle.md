# The chronicle

News that reads like a history. The observer and the families between them
file about 1,500 items in a game with six families, and the viewer called
everything of weight 0.6 or more "major": about 1,100 of them, of which over
a thousand were the player's own works opening one by one. The chronicle is
the editor that was missing. It decides nothing in the economy and the
simulation never reads it: every series of a run with it on is the same to the
last bit as with it off (`tests/chronicle.rs`). None of it is in the Python.

It is off by default (`chronicle.enabled`), and off it leaves no trace: no
event has a new field, `State::chron` stays empty, and a port run's
`events.json` and `params.json` are byte for byte what they were (the golden
test). The game's rules switch it on. To see it alone:

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --families 6 --set chronicle.enabled=true --out DIR
cargo run --release -p isostate-core --example chronicle            # seed 7's major feed, decade by decade
cargo run --release -p isostate-core --example chronicle -- --why   # the same, with each item's provenance
cargo run --release -p isostate-core --example chronicle -- --seeds 1-8   # the counts below
cargo run --release -p isostate-core --example chronicle -- --seeds 1-8 --set prices.enabled=false   # the same with one parameter changed
```

The code is `crates/isostate-core/src/chronicle.rs`. The hooks in other files
are one or two lines each and marked `chronicle:`.

## What it does

**The flood.** Under the port every works a branch of the player's family
opens on its own is an item of weight 0.6 (`families::found_at`): 1,051 of the
player's items in a mean game, all of them "major". With the chronicle:

- a works the player orders by hand (`found_works`) is told as it was, each
  one, at 0.6;
- a works a branch opens on its own is told once if it takes the trade to a
  town where the family never had it ("The Fairfax family sets up in textiles
  at Brambridge", 0.6, with the works, its cost and what it expects to
  return);
- everything else the branches opened in the year is one item at the year's
  end ("The Fairfax branches open five works", with the towns and trades), at
  0.45, which keeps it in the family's news and out of the major feed. With
  the life cycle for works (`works.md`) a branch extends a works it has
  before it opens another, and what the branches extended is in the same
  item ("The Fairfax branches open two works and extend three", or "The
  Fairfax branch at Melbourne extends a works"), with what it cost;
- the family's everyday news (births, a child turning fourteen, trainings,
  sales, burials) weighs 0.5; its great moments (a new trade, a new head)
  0.7. A head's death and the succession used to be two items of the same
  weight; now the succession is the great one and says whom the new head
  follows, and the burial is everyday news;
- rivals are told as before: the first works of each kind, at 0.35.

`tests/families.rs` still asserts that every player item weighs at least 0.6:
that is the port, and holds. The game's rules have their own test
(`the_players_works_no_longer_flood_the_news`).

**Provenance.** Every event the observer detects carries `why`: the test in a
few words, the series, the value, the threshold it had to pass, the baseline
it was compared with where there is one, and the same in a sentence. So do the
regional stories below, large closures, and hunger.

```json
"why": { "test": "people against the town's own peak",
         "series": "people in Skelford and its district",
         "value": 8496.1, "threshold": 8686.7, "baseline": 9651.88,
         "note": "12% below the peak of 1783, which is 8 years old (it needs 10% below a peak at least 8 years old)" }
```

(Skelford's district was cut for Golddale in 1783, so its count starts
again there; the story says "had 9,652 people in 1783", not "at their
peak".)

The numbers are rounded to six figures; they are for reading. Inventions,
railways, new towns and the families' items have no `why`: they are things
that happened, not tests that fired. The harvest's news (`harvest.rs`) is of
the testing sort and says why too: how short the crop was and the line it
had to pass, and for a dearth the price, where the earlier years pointed and
the line above that.

**Ranking.** Each event carries `kept`: whether a history of its decade would
keep it. Within a decade the heaviest go first, with at most four in all
(`per_decade`), one of a kind (`per_kind`; railway schemes and openings are
one kind, and so are a harvest and the dearth that is its price), two
inventions (`inventions`) and two turning points (`turning_points`), and
nothing lighter than 0.6 (`floor`). One place is the
player's: the heaviest of the family's own items at 0.6 or more is kept
whatever else happened. A decade that is over and still has fewer than two
(`chronicle::AT_LEAST`) takes the best of what is left above 0.45
(`chronicle::FILL_FLOOR`). A dated slump is
ranked by its weight and its depth, since the observer weighs a slump of 5 per
cent and a depression of 20 the same. An alarm ("Output falls: is this a
slump?") is kept while it is the latest word and gives way to the verdict
that later dates its slump: called live, dated later. (Alarms and verdicts
were one kind until the news was read whole; that only ever dropped a fall
still running beside the verdict on an earlier slump. See "The news read as a
reader would", below.)

The whole list is ranked afresh at each year's end (`chronicle::yearly`) and
once more on the sorted list in `sim::finish`, so a live game ends with the
marks a recorded run has. While a decade is running an item may be kept and
then lose its place to something heavier; `LiveYear::rekept` tells the viewer
which marks changed.

The viewer's major feed, timeline ticks and toasts ask `Event::major()`: the
mark where an event has one, weight 0.6 or more where it has none (a replay
recorded without the chronicle looks as it did).

**Regional stories** (kind `region`, "Town and country"), from what the state
already keeps about towns. Each has a town, a cell, a year and a reason taken
from the same prices and incomes that drove it:

| story | when | the reason it gives |
|---|---|---|
| Coal boom around X (0.72) | the pits of X's district raise `boom_growth` (1.5) times the coal of `boom_years` (5) years before, and `boom_share` (15%) of the island's | a railway or mineral line opened since (under the game's railways: a line to the town, or a branch to its collieries; "the railway that reached" the town only of its first line, and a later one by where it comes from), and what the local price of coal did: dearer (buyers want more than the pits raise), cheaper (the pits outran their buyers) or steady |
| X in decline (0.68) | X and its district are `decline_fall` (10%) below a peak at least `decline_years` (8) old | where this year's leavers went and how much further an income goes there (what `people::migration` weighs), or that incomes no longer keep a family |
| X overtakes Y as the island's first (0.8) or second (0.7) town, or X is now the island's first or second town | for `overtake_years` (3) running the town truly in the place is not Y, the town last told of there, and is `overtake_margin` (5%) clear of it (or of the town below, if none is told), whichever town that is from year to year; X is the one in the place when it is told, and the story says "is now" when X did not grow past Y (a founding or a road redrew either's district since X took the place) or when no town is left to name | each town's growth over ten years, where Y now stands if it is not just below, and what X supplies most of |
| X is the fastest-growing town of the decade (0.62) | at the decade's end, if X outgrew the island by `fastest_margin` (10 points) | how much of the growth moved in from other towns, and what X supplies |
| N works close around X (0.66) | at least `closures_min` (5) works of a district shut in a year and they are `closures_share` (10%) of those that stood, shops and trades aside | what most of them made, and its price a year or three years back, or the price of what they work up, or wages; the largest is named |
| N new towns in the decade (kind `town`, 0.65) | `new_towns_min` (3) or more founded in a decade | which, when, and the largest |

Towns are ranked with their districts (`st.pop`), not by the people inside
the town (`st.urban_pop`), which is a quarter of that and too even between
towns to rank by; hence "first town" and not "largest", and every such story
says "X and its district". A new town takes people and pits from the
districts it is cut out of (`people::resettle_new_towns`), so nothing about
it, or about the towns that held its ground the year before, is compared
across its founding: their peaks start again that year, their coal and their
decade's growth are not set against what they were, and a town that comes
first or second because a founding cut its rival's district is told as being
first or second, not as having overtaken. (Until the news was read whole the
places were seated afresh without a story, and a town could lead the island
for forty years untold.) The towns a founding did not touch
keep their stories, so an island where towns go on being founded all century
still has them (with `found_activity` raised until the last town is founded
around 1850 or later, a run still tells 20 to 33 regional stories). A town
in its first five years has no boom or closures story, and a new town that
never grew past the people it was founded with is not said to decline: it
did not take. A district that never held `min_people` (5,000) is not said to
decline or to be the fastest-growing; a boom or a round of closures is not
told of the same town again within `quiet_years` (15), and a decline only
after a new peak. One story of a kind a year: if several towns qualify the
one hit hardest leads and the others are named in its last sentence.

**Three faults of the observer**, mended only under the switch:

- City milestones fired for the first town past each size and never again.
  Now every town has them; a later one is told more quietly (0.45 and up,
  against 0.55 and up) with the towns that got there first.
- Doublings were measured against 1750 alone, whatever kind of year that was.
  Now against the mean of the first five (`chronicle::BASE_YEARS`), and the title says so
  ("Real output doubles since 1750–54"). Nothing doubles while that mean is
  forming.
- A rival's "first works of its kind" was counted again after it sold its
  only one, because the port asks what the family owns today. The chronicle
  remembers what each family has ever held, from the works they start with.

## Its numbers

`ChronicleParams`, all under `chronicle.` for `--set`:

| | default | |
|---|---|---|
| `enabled` | false (true in a game) | |
| `per_decade` | 4 | most kept in a decade |
| `per_kind`, `inventions`, `turning_points` | 1, 2, 2 | most of one kind; of inventions; of turning points |
| `floor` | 0.6 | lightest item kept, but to fill a thin decade; the player's place is kept at it and above, so it must stay under `own_great` |
| `own_great`, `own_small`, `own_summary` | 0.7, 0.5, 0.45 | weights of the player's own news |
| `boom_years`, `boom_growth`, `boom_share` | 5, 1.5, 0.15 | coalfield boom (`boom_growth` was 2 until the chronicle met the game's other rules: "In company") |
| `decline_years`, `decline_fall` | 8, 0.1 | town in decline |
| `overtake_years`, `overtake_margin` | 3, 0.05 | one town overtaking another |
| `fastest_margin` | 0.1 | fastest-growing town of a decade |
| `closures_min`, `closures_share` | 5, 0.1 | a district's works closing together (ten before the life cycle for works; "Beside the life cycle for works", below) |
| `new_towns_min` | 3 | a decade of new towns |
| `min_people`, `quiet_years` | 5,000, 15 | no stories of smaller towns; years before a story repeats |

Three more were settings until stage 5 of the pass over the rules
(`docs/designs/rules-pass.md`, section 4.5), and are constants in
`chronicle.rs` now, with the values they had: `AT_LEAST` (2), the fewest kept
in a finished decade; `FILL_FLOOR` (0.45), the lightest kept to fill a thin
one; and `BASE_YEARS` (5), the years a doubling's baseline is the mean of.
`sens-mines.md` tried them at 1 and 3, 0.34 and 0.6, and 4 and 7: no decade
was thin enough for the first two to act, and the third moved one milestone
on one island.
A settings file or a `--set` that names one is refused with a sentence that
says so. `floor` and `own_great` stay settings, on one condition: the player's
place in the feed is kept at `floor` and above, so a `floor` at or above
`own_great` empties it, and `tests/chronicle.rs` holds `floor` under
`own_great`.

No random number is drawn anywhere in it.

## What the sweeps showed

This section is the chronicle alone, on the port, as it was built and
reviewed: its coal booms are counted at `boom_growth` 2, which was the line
then. What it does beside the game's other rules is under "In company".

Seeds 1 to 8 under the port's rules with `--set chronicle.enabled=true`
against the same build without it, seed for seed. Of the 129 facts both
sweeps have, 126 are `=` without families and 125 with six, and the ones that
differ are all counts of news: the chronicle does not touch the economy. No
run failed. What moves:

| per run, mean of 8 seeds | no families: off | on | six families: off | on |
|---|---|---|---|---|
| `news_family` | | | 1,243 | 436 |
| `news_milestone` | 11.1 | 16.5 | 11.8 | 18.4 |
| `news_town` | 13.2 | 15.5 | 13.8 | 15.8 |
| `news_region` | | 47.8 | | 47.5 |
| `news_total` | 245.8 | 301.1 | 1,485 | 734.6 |

Seeds 9 to 16, which nothing here was chosen on, say the same: again only
the news counts differ and no run fails. Without families `news_region` is
46.2 [39 .. 51] and `news_total` goes from 237.8 to 291; with six,
`news_family` goes from 1,177 to 433, `news_region` is 42.5 [34 .. 48] and
`news_total` goes from 1,416 to 723.

And what a reader of the major feed meets, six families, the port's rules
against the game's (`--example chronicle -- --seeds 1-8`):

| per run, mean [least .. greatest] of 8 seeds | port | game |
|---|---|---|
| all items | 1,485 [1,324 .. 1,683] | 735 [707 .. 767] |
| the major feed | 1,127 [981 .. 1,328] | 55.4 [53 .. 59] |
| the major feed in a decade | 14 in the 1750s rising to 115 in the 1890s | 3.0 to 4.0, never under 2 or over 4 |
| the player's own items | 1,051 [910 .. 1,254] | 244 [225 .. 260] |
| of them major | all | 11.8 [10 .. 14] |

Major items in a run by kind, port then game: the player's family 1,051 and
12; markets (booms, gluts, slumps) 22 and 8; inventions 10
and 9; slumps dated 11 and 9; regional stories none and 9; turning points 5
and 4; new towns 14 and 2 (the decade's digest); milestones 6 and 2; the
first railway 1 and 1; alarms 8 and none at the end (each has been dated by
then).

Of the 48 regional stories in a run, 9 are kept: 2.5 overtakings, 2.4 coal
booms, 2.2 declines, 1 fastest-growing town and 0.8 rounds of closures (of
4.1, 6.0, 16.5, 10.6 and 10.2 told).

Seed 7 with six families, three decades of the major feed as it now reads:

```text
The 1790s
  1791  Town and country Granness overtakes Etherside as the island's second town
        Granness and its district have 22,001 people to Etherside's 20,427. In ten years Granness has grown
        by 8% and Etherside has shrunk by 1%. Granness supplies 23% of the island's rails.
  1791  Your family      Henry Fairfax heads the family
        Henry follows Anne Fairfax, who died at 77 and is buried at Melbourne, the family's seat.
  1795  Milestone        Real output doubles since 1750–54
        Real output is now 2.0 times its 1750–54 level.
  1799  Turning point    Mills now make most of the cloth
        Powered mills turn out 63% of all cloth. Handloom weavers are paid less each year for the same work.

The 1800s
  1800  Invention        Rails rolled at Ashdelf
        Caleb Calvert passes hot iron between grooved rollers and gets rails by the yard.
  1801  Turning point    Engine works outbuild the millwrights
        Engine works now build 60% of all machinery. Millwrights go back to mending.
  1809  Town and country Etherside overtakes Granness as the island's second town
        Etherside and its district have 27,554 people to Granness's 25,357. In ten years Etherside has grown
        by 18% and Granness has grown by 1%. Etherside supplies 29% of the island's coal.

The 1810s
  1810  Your family      Edward Fairfax heads the family
        Edward follows Henry Fairfax, who died at 79 and is buried at Melbourne, the family's seat.
  1813  Markets          Grain slump
        Grain output fell 30% this year against a trend of +1% a year, while prices fell 1%: buyers have stopped buying.
  1814  Turning point    Bought cloth overtakes homespun
        Households now get 56% of their clothing from bought cloth rather than wool spun at home.
  1814  In hindsight     The depression of 1810–1812
        Output fell 17.7% from its 1810 peak to the 1812 trough and took until 1814 to recover.
        Historians will call this a depression.
```

## In company

The chronicle was merged into the game's rules after `prices`, `engel`,
`machines` and `harvest`, so the island it now tells of is not the port's:
merchants hold stock and prices no longer chase their own tail, a wage buys
two and a half times as much by 1900, and the weather files news of its own.
The integration's sweeps are under `.claude/night/reports/core-steps/`:
`chronicle-before` and `chronicle-after` (the game's rules with six families
on seeds 1 to 6, without and with the chronicle) and `chronicle-s7-12`.

**It still touches nothing.** Of the 184 facts both sweeps have, 180 are `=`
on every seed and the four that differ are counts of news; no run failed on
seeds 1 to 12, and a run takes the 17 seconds it took. In `sim::run_year` it
runs after the yearly step and the observer, so it ranks everything the year
filed, the harvest's news with the rest, and before the snapshots.

| per run, six families | alone, on the port (seeds 1 to 8) | in company (seeds 1 to 12) |
|---|---|---|
| all items, chronicle off | 1,485 | 1,614 (seeds 1 to 6) |
| all items | 735 [707 .. 767] | 763 [680 .. 805] |
| the major feed | 55.4 [53 .. 59] | 55.8 [52 .. 58] |
| the major feed in a decade | 3.0 to 4.0, never under 2 or over 4 | 3.0 to 4.0, never under 2 or over 4 |
| the player's own items | 244 [225 .. 260] | 246 [227 .. 262] |
| of them major | 11.8 [10 .. 14] | 11.6 [9 .. 13] |
| milestones told | 18.4 | 30.4 |
| regional stories told | 47.5 | 45.2 |

So the flood ends as it did and the feed is the length it was. There are
more milestones because every town has them and this island's towns grow:
0.43 of its people live in towns in 1900 against the port's quarter.

What is kept in a run, by kind (seeds 1 to 12): the player's family 11.6,
inventions 9.3, regional stories 8.6, markets 6.9, turning points 6.3,
milestones 4.0, slumps dated 2.6, dearths 2.2, harvests 1.4, new towns 1.6,
the first railway 1.1. Of the regional stories 3.2 are declines, 1.8
overtakings, 1.7 fastest-growing towns, 1.0 rounds of closures and 0.9 coal
booms (of 16.5, 3.2, 11.6, 10.0 and 3.8 told).

Three things were different in company, and were mended in a few lines each.

- **The coal booms had gone, and the line was lowered from 2 to 1.5.** Alone
  on the port a district's pits doubled their coal in five years six times a
  run. In company it was 0.7 times a run on seeds 1 to 6, and none was kept.
  Taking the others away one at a time (`--example chronicle -- --seeds 1-6
  --set NAME.enabled=false`) puts it on `prices`: 4.7 a run without the
  merchants' prices, 1.5 without the machines, 1.8 without the curve. The
  port's coal swung with its iron trade, which overshot and fell back every
  few years, and most of those doublings were the upswing of that false
  cycle; with stock held and gluts not bought into, a coalfield grows more
  steadily. A district that raises half as much coal again in five years,
  and 15 per cent of the island's, is still a boom on an island whose people
  grow by a twentieth in that time, so the line is now 1.5: 3.8 booms told a
  run on seeds 1 to 12 and about one kept. They come where they should, in
  the 1760s to 1790s and again when the railway arrives ("The pits around
  Wilmere raise 2.0 times the coal they raised in 1828, and 33% of all the
  island's. The railway that reached Wilmere in 1833 carries it away."), and
  eighteen of the twenty-one on seeds 1 to 6 say the price fell: the pits
  outran their buyers. The number is in the struct's defaults, so the port
  with `--set chronicle.enabled=true` now reads 1.5 too, and there, with its
  false cycles, it tells 13 coal booms a run (seeds 1 to 8) where the tables
  above say six. Nobody plays that island; for the old count add `--set
  chronicle.boom_growth=2`.
- **A short crop was told twice in one decade's history.** The harvest files
  "A poor harvest" or "Three lean years" (kind `harvest`, up to 0.75) and "A
  dearth of grain" (kind `dearth`, up to 0.8), and as two kinds they could
  take two of a decade's four places for one bad run of years: in five
  decades of the 180 on seeds 1 to 12, and in seed 5's 1890s both for the
  same year. They are now one kind when places are shared out, as slumps
  called and slumps dated are, and the heavier is kept. Nothing else about
  either changes: both are still told, and both are in the "All" list.
- **The harvest's news had no provenance.** It is news because a number
  passed a line, like the observer's, so it now says which
  (`chronicle::why` in `harvest::news`; nothing while the chronicle is off).
  `tests/chronicle.rs` holds it to that.

The viewer and the example label the two kinds "Harvest" and "Dearth" (they
were "News", in grey), and the example takes `--set`, which is how the
numbers above were had.

### Beside the life cycle for works

The life cycle for works (`works.md`) was merged after the chronicle, and it
changes what there is to tell: a tenth as many works, each ten times the
size, and a family that holds a few dozen of them. Seeds 1 to 12, six
families, `--example chronicle -- --seeds 1-12` with it and with `--set
works.enabled=false`:

| per run | the game before | with the life cycle |
|---|---|---|
| all items | 763 [680 .. 805] | 888 [827 .. 926] |
| the major feed | 55.8 [52 .. 58] | 57.5 [54 .. 59] |
| the major feed in a decade | 3.0 to 4.0, never under 2 or over 4 | 3.5 to 4.0, never under 2 or over 4 |
| the player's own items | 246 [227 .. 262] | 226 [200 .. 248] |
| of them major | 11.6 | 12.1 |
| closures told (weight 0.4, none kept) | 56 | 204 |
| alarms; slumps and depressions dated, and kept | 8; 8, 3 | 17; 16, 7 |
| regional stories told | 45.2 | 42.4 |
| of them rounds of closures, told and kept | 10.0, 1.0 | 13.2, 1.8 |
| declines of towns, told and kept | 16.5, 3.2 | 10.8, 1.8 |

(The first column's rounds of closures are at the line of ten works it was
measured with; at five the game without the life cycle tells 18.)

The feed a history keeps is the length it was, with seven dated slumps in
it where there were three: the life cycle brings a building cycle
(`works.md`, "In company with the rest of the game"). Two things were
mended, and one is left.

- **A branch's extensions were the flood over again.** With the life cycle
  a growing branch extends a works it has before it founds another, and
  `works::extend_for_family` told the player of each: 240 items in seed 7's
  game, in 102 of its years, up to eight in a year. They are the branch's own
  doing, as a works it opens is, so they now wait for the year's summary
  (`chronicle::works_extended`, `branch_summary`), which says what was opened
  and what was extended, where, and for how much. With the chronicle off the
  works' own item is made as it was.
- **Rounds of closures had gone, and the line was lowered from ten works to
  five.** A district had some four hundred works and has forty, so ten of
  them shutting in one year hardly ever happens: the story was told once in
  seed 7's game and never in seed 3's, where it had been told 8 and 13
  times. At five (and a tenth of those that stood, as before) it is told 12
  to 15 times a run on seeds 1, 3, 5 and 7, and 13 times on seeds 1 to 12.
  Like the coal boom's line the number is in the struct's defaults, so the
  game with `--set works.enabled=false` reads five too.
- **Left: single closures.** `sim::notable_closures` tells of a works that
  shuts at over one and a half times its trade's usual size, two a year at
  most. That was a large works; the median works is now five or six usual
  sizes, so nearly every year has its two, 204 in a run for 55. They weigh
  0.4 and none is kept, so the major feed does not see them, but the "All"
  list does. What makes a closure worth telling under the life cycle (its
  hands, its place among the works of its kind) is a choice about the news
  and was not made here.

`tests/chronicle.rs` said three things of seed 7's game that are no longer
so, and says them of the game as it is. The family's branches opened about
a thousand works; they now open some fifty and extend them 240 times, so
their doings are counted in the same game with the chronicle off (294 items)
and the news of them must still be a fraction of that. The decade of new
towns was the 1750s and was kept; with hands walking to work the towns come
later, three of them in the 1760s, and the digest is told once with its
years and is kept unless the decade's four places go to heavier things (they
do: two inventions, a coal boom and the family's new trade). And "less news
with the chronicle than without" is held over the whole game (849 items for
975 on seed 7), and over the first forty years for the player's own news,
since there is no longer a flood in every stretch of years to outweigh the
stories the chronicle adds.

### Beside the game's railways and carting

The game's railways and carting (`railways.md`, `carting.md`) were merged
last. Seeds 1 to 12, six families, `--example chronicle -- --seeds 1-12`
with them and with `--set railways.enabled=false --set
carting.enabled=false`:

| per run | the game before | with railways and carting |
|---|---|---|
| all items | 888 [827 .. 926] | 902 [834 .. 960] |
| the major feed | 57.5 [54 .. 59] | 58.2 [55 .. 60] |
| the major feed in a decade | 3.5 to 4.0, never under 2 or over 4 | 3.6 to 4.0, never under 2 or over 4 |
| the player's own items | 226 [200 .. 248] | 220 [160 .. 246] |
| of them major | 12.1 | 12.3 |
| railway openings and schemes told, and kept | 36 and 34, 1 | 83 and 20, 1 |
| regional stories told | 42.4 | 47.0 |
| of them coal booms, told and kept | 4.2, 1.8 | 4.6, 1.5 |
| declines of towns, told and kept | 10.8, 1.8 | 13.0, 2.6 |

The feed a history keeps is the length it was and keeps the same one
railway item, the first line. The branches are told once a year, lightly
(0.35), and stay in the "All" list: 13 or 14 railway items a decade there
from the 1830s to the 1850s.

One thing was mended. **A coal boom said a mineral line had opened whatever
the branch was for.** The story's reason reads the record's list of opened
lines, where the port marks a mineral spur by its having no second town.
The game's promoters build branches to farms, woods, quarries and works as
well, and a town's own first line, and all of them have no second town. So
with `railways.enabled` the story asks the promoters' book what the line was
for (`railways::line_for_coal`): the railway that reached the town carries
the coal away, a branch to the district's collieries brings it down from
the pits, and any other branch is left out of the story. With the switch
off the reason is found as it was. `tests/chronicle.rs` holds the game to
it, and two of its notes on seed 7's game were brought up to date: the last
town is now founded in 1831, and the decade of new towns is the 1770s.

### Beside the game's rules for people

People (`people.md`) came last. Seeds 1 to 12, six families, `--example
chronicle -- --seeds 1-12` with them and with `--set people.enabled=false
--set carting.adjust=1`, which is the game before
(`core-steps/people-probe/chronicle-game.txt` and `chronicle-before.txt`):

| per run | the game before | with people |
|---|---|---|
| all items | 902 [834 .. 960] | 915 [847 .. 982] |
| the major feed | 58.2 [55 .. 60] | 58.9 [57 .. 60] |
| the player's own items, and of them major | 220, 12.3 | 222, 12.1 |
| town items | 11 | 13 |
| regional stories told, and kept | 47.0, 8 | 40.8, 8 |
| of them coal booms | 4.6 | 4.1 |
| one town overtaking another | 3.6 | 3.1 |
| declines of towns | 13.0 | 11.2 |
| the decade's fastest-growing town | 11.6 | 10.2 |
| a district's works closing together | 14.2 | 12.2 |

Two things were fitted.

**A town is redrawn by its cells as well as by a founding.** Under people
households go with their cells every year, so a road or a railway that
carries a district from one market to another moves its people on the day,
with no founding. A town that gained or lost a tenth of its people that way
in a year (`st.pp_moved`, against `REDRAWN_SHARE`) is treated as a founding
treats it: its peak starts again and nothing sets it against what it was.
A tenth is the least fall told as a decline and the least lead told as
outgrowing the island, so a smaller redrawing cannot make a story alone.
That is why a seventh fewer regional stories are told; as many are kept.
`tests/chronicle.rs` holds a town to being redrawn in exactly those years.

**People taking ship is not the hunger.** The item has the kind `famine`,
and `famine_provenance` would have given it the hunger's reason, which
speaks of the food in the town named. The town named is the port they
sailed from. `peopling::growth` now gives the item its own reason (how many
left, against one in a thousand of the island), and the hunger's is only
written where there is none.

### The news read as a reader would

In November a century of the game's news on eight islands was read item by
item, the way a player meets it (`docs/probes/news-read.md`), and checked
against the record. Its plain faults, the ones that tell a reader something
false or that looks like a mistake, are mended here, all under the switch: the
port's news is the same to the letter (the golden test). Seeds 1 to 8 with six
families, `isostate-core run --seed N --rules game --families 6`, before and
after, read side by side:

- **The coal booms' railway.** "The railway that reached Garside in 1829"
  named the latest line to the town since the boom's base year; Garside's
  first line came in 1827. `railways::line_for_coal` now also says which line
  to the town it was and when the first came, and "reached" is said only of
  the first: "The railway reached Garside in 1827, and a second line, from
  Sandmere, in 1829; they carry it away", or, where the first came before the
  base year, "The line from Glenbrook, opened in 1835, carries it away". All
  eleven coal booms that name a line now agree with the railway items told at
  the time (eight of nine named a wrong year before).
- **The island's first and second towns.** The chronicle compared the town
  last announced in a place with the largest town not in it, and kept that
  town seated while it sank; when a founding cut a leading district it seated
  the places afresh without a word. So "Kelside overtakes Melbourne as the
  island's first town" (island 7, 1858) read as if Melbourne were second when
  it was fourth, and Glenbrook led island 7 for forty years untold. Now
  (`chronicle::places`) a place is lost once, for three years running, the
  town truly in it is 5 per cent clear of the town last told of there, whoever
  that is from year to year; the story names the town told of, says where it
  now stands if it is not just below ("Melbourne is now the island's fourth
  town"), and says "Glenbrook is now the island's first town" where a founding
  or a road redrew either district since. A story of first place that names
  the old first just below has told the reader it is second, and it is seated
  there. Stories of first and second place
  went from 26 to 49 on the eight islands, and every one names a town truly in
  the place it names. Of the spells of five years or more at the top that a
  reader never heard of, eleven before, three are left, each a lead that never
  stood 5 per cent clear for three years running.
- **Returns of a hundred per cent and more.** "Built for 31 and expects to
  return 1288 per cent a year" is the investor's own sum for a tiny works and
  true to the model, but reads as a mistake. A return of 100 per cent or more
  is now left out of the item (`chronicle::return_told`): 131 items on the
  eight islands. The page proposed "should pay for itself within a month"; its
  checker would rather leave the figure out, and the checker's way is taken.
- **Grain that moved with the weather.** The observer's market items said
  "supply has failed while buyers still want it" of grain the year after a
  bumper harvest, and "new capacity is flooding the market" of grain coming
  back after a short one. For grain under the harvest's rules the reason is
  now the weather's when the harvest moved the same way as output, by a spread
  or more (`chronicle::grain_reason`): "last year's harvest was a bumper one,
  and this year's is not" (all six such shortages), "the harvest has come back
  after a short one" (fourteen gluts of 24), "the harvest is a bumper one"
  (three) and "the harvest has come in short" (one); and where the harvest
  moved that much but was neither bumper nor short, "the harvest is better
  than last year's" (four gluts) or "the harvest is poorer than last year's"
  (seven shortages), as on island 6 in 1778, when it went from 1.08 of an
  ordinary year to 0.92. Three gluts, in years the harvest hardly moved, keep
  the detector's own reason.
- **Works named after another town.** Works are named for the town whose
  district they were built in and keep the name when a road or a new town
  moves their ground: "Mallowwick Brickworks closes. The brickworks near
  Sandmere ..." Such a works now says it: "open since 1749 and named when its
  ground was Mallowwick's" (`chronicle::named_when`; 45 items).
- **Charcoal is not coal.** "The Ashby family sets up in coal and mining at
  Larkbourne. Ashby's Charcoal is the first works ...": the families count
  charcoal burning in the coal and mining trade, which a reader of the 1770s
  will not expect. The chronicle's own sentences now call it charcoal burning
  (`chronicle::trade_of`); the trade keeps its name everywhere else.
- **The cap of one a kind.** Turning points now have two places a decade
  (`turning_points`), so the 1840s that saw more hands in workshops than on
  the land can also keep "Rolling mills overtake the rail forges"; and alarms
  and verdicts are no longer one kind, which only ever dropped a fall still
  running ("Output falls 10.0%", island 4, 1895) beside the verdict on an
  earlier slump. Turning points kept go from 49 to 60 on the eight islands,
  alarms from 1 to 5; markets items kept from 47 to 42, milestones from 31 to
  27.
- **Counts of one.** "In one years" and "one of them makers of" could be
  written; they are now "In a year" and "one of them a maker of".

Of the sweep's facts only `news_region` and `news_total` move, by the
places stories: 352 to 375 regional stories on the eight islands (from one
fewer to six more a run), 8,016 to 8,039 items in all.

## What worries me

- **Two turning points a decade keep the island's starting state twice.**
  "Coal fires most of industry" in 1751 is the share wandering about the line
  it starts on (the news reading's fault 4), and with two places a decade
  island 7 now keeps "Coal now heats more homes than wood" (1753) beside it.
  The mend for that is in the observer's crossing (no turning point for a
  share that was in the band in the first year), not here.
- **"Is now the island's first town" is told years late.** A lead must stand
  5 per cent clear for three years, so Glenbrook, first from 1763 when
  Garside's founding cut Brambridge's district, is told as first in 1766, and
  Brambridge, first from 1751, in 1754. The margin keeps two towns of a size
  from trading the title every year; a reader may still wonder why the news
  is late.

- **Grain is seven of the 56 items a history keeps.** In company 7.4 of a
  run's kept items are about the crop (seeds 1 to 12): 2.2 dearths, 1.4
  harvests, and 3.8 of the 6.9 markets items, which are "Grain glut" or
  "Grain shortage". The last are the observer's steepness detector meeting
  the weather: a bumper year is a rise in output far from trend with a fall
  in price, and it says so at 0.8, above anything the harvest files, in the
  year the harvest has already told. One decade in eighteen keeps two items
  on grain (ten of 180), none three. I left it: markets have their one place a
  decade and grain is the good a history would pick, but "Grain glut" is a
  duller telling of a bumper harvest than the harvest's own. The mend would
  be in the observer (no steep item on a good in a year its harvest was
  news), not here.
- **The coal line was moved on twelve seeds.** 1.5 gives three or four booms
  a run on seeds 1 to 12 with the rules as they stand tonight. A mechanism
  that changes how mines open or how coal is carried will move that again,
  and the place to look is the last table of `--example chronicle -- --seeds
  1-12`.

- **Four a decade is the top of "two to four".** Most decades fill their
  four places (the mean is 3.7), one of them the player's. `per_decade=3`
  gives a sparer history; I left four because the quiet decades late in the
  century then still show two or three.
- **The player's place is nearly always a succession.** A family left to its
  defaults has a new head every fifteen or twenty years and a new trade three
  times in the run, so in seed 7 nine of its thirteen kept items are "X heads
  the family". A player who orders works by hand gets no more: each is an item at
  0.6, below a succession. When the player's loop gains decisions worth
  remembering they should weigh more than 0.7.
- **Markets items are a fifth of what is kept**, and they are the port's:
  "Rails slump" in 1762, eighty years before a railway, is a real fall in the
  output of the rail forges and reads oddly. The steepness detector weighs
  every good alike (0.8); a history would care more about grain and coal than
  charcoal.
- **A live game takes marks back.** An item is kept when it is the best of
  its decade so far, and the toast is shown then; a heavier item later in the
  decade can take its place in the feed. In a run to 1812 marks were taken
  back (the test counts them); I have not watched how that feels in the
  viewer, and the viewer side (`rekept`, the feed, ticks and toasts) is built
  and compiles but was not looked at on screen.
- **Stories lean on the port's oddities.** "Its people leave for Brightedge,
  where a household's income goes more than three times as far" (seed 3) is
  what `y_ratio` says, and the fastest-growing town nearly always grew "more
  than half" by migration. If the towns mechanism changes how people move,
  these sentences change with it, which is as it should be, but the
  thresholds (a 10 per cent fall, a 10-point margin) were chosen on today's
  towns.
- **The succession was found by the port's wording.** `chronicle::succession`
  looked for "head of the family, is buried at" in the year's family news,
  and read the dead head's name, age and resting place back out of the
  burial's title and sentence. Now `families::people` hands it the facts as
  it writes the burial (`chronicle::Burial`: which item is the burial, who,
  his age, where, and whether that is the seat), and the new head's first
  name, so the burial can be reworded and the succession still told. The
  news of seeds 1 to 8 with six families is the same to the letter either
  way. (`tests/chronicle.rs`,
  `a_succession_holds_to_the_facts_not_to_the_burials_wording`.)
- **The decades are the calendar's.** They were counted from the start year
  in the ranking and at the decade's end, and named by the calendar, which
  agree only for a start year that is a multiple of ten: a `--content` file
  starting in 1755 would have closed "the 1760s" in 1764. Now a decade is
  the calendar's everywhere, and a run from 1755 has a short first one, 1755
  to 1759, whose new towns are counted from 1755 (`chronicle::decade_of`,
  `decade_ends`; two tests in `tests/chronicle.rs` starting in 1755). For a
  start in 1750 nothing moves.
- **Other mechanisms' news.** Whatever a new mechanism files (a harvest, a
  canal, a strike) is ranked by its weight and kind like anything else: a new
  kind gets one place a decade. Nothing needs registering, but a mechanism
  that files many items at 0.6 or more will crowd the "All" list, not the
  major feed. It can add provenance with `chronicle::why(P, ...)`, which
  returns nothing while the chronicle is off. The harvest was the first to
  arrive, and showed the one thing that does need a line here: a mechanism
  that files two kinds about the same event wants them named as one in
  `chronicle::class` ("In company").
- **Events have no stable id.** `rekept` names an event by its place in the
  run's list, which works because the list only grows. Saving and loading a
  game would need the same order kept.
- **A works ordered by hand may get no toast.** The event is what it was
  (0.6), but the toasts now ask for the mark, and the family has one place a
  decade: if a succession or a new trade already holds it, the works the
  player has just ordered is in the family's list and nowhere else. If that
  feels like silence, the toasts could also show the player's own items at
  0.6 or more (about one every three years); it is one condition in
  `main.rs`.
- **What fills a thin decade.** A finished decade with fewer than two items
  at 0.6 takes the best of what is left above 0.45, and with families that
  can be the player's everyday news (a birth, at 0.5). It has not happened
  on seeds 1 to 16, where no decade is that thin.
- **Districts move without foundings too.** Roads and railways shift cells
  between districts every year. People do not move with them, so the stories
  about people are safe; a district's coal does, so a "coal boom" can be
  partly a railway redrawing whose pits are whose. The story says when a
  line has just reached the town.
- **Settings far from the defaults give nonsense, not a crash.** With every
  threshold at zero, or huge, or negative, runs finish (three seeds each):
  the stories are five hundred a run, or none. `closures_min = 0` used to
  stop the run and no longer does.
