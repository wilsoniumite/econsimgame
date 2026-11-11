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
  0.45, which keeps it in the family's news and out of the major feed;
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

The numbers are rounded to six figures; they are for reading. Inventions,
railways, new towns and the families' items have no `why`: they are things
that happened, not tests that fired.

**Ranking.** Each event carries `kept`: whether a history of its decade would
keep it. Within a decade the heaviest go first, with at most four in all
(`per_decade`), one of a kind (`per_kind`; slumps called and slumps dated are
one kind, as are railway schemes and openings) and two inventions
(`inventions`), and nothing lighter than 0.6 (`floor`). One place is the
player's: the heaviest of the family's own items at 0.6 or more is kept
whatever else happened. A decade that is over and still has fewer than two
(`at_least`) takes the best of what is left above 0.45. A dated slump is
ranked by its weight and its depth, since the observer weighs a slump of 5 per
cent and a depression of 20 the same. An alarm ("Output falls: is this a
slump?") is kept while it is the latest word and gives way to the verdict
that later dates its slump: called live, dated later.

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
| Coal boom around X (0.72) | the pits of X's district raise `boom_growth` (2) times the coal of `boom_years` (5) years before, and `boom_share` (15%) of the island's | a railway or mineral line opened since, and what the local price of coal did: dearer (buyers want more than the pits raise), cheaper (the pits outran their buyers) or steady |
| X in decline (0.68) | X and its district are `decline_fall` (10%) below a peak at least `decline_years` (8) old | where this year's leavers went and how much further an income goes there (what `people::migration` weighs), or that incomes no longer keep a family |
| X overtakes Y as the island's first (0.8) or second (0.7) town | X has been `overtake_margin` (5%) ahead of Y for `overtake_years` (3) running | each town's growth over ten years and what X supplies most of |
| X is the fastest-growing town of the decade (0.62) | at the decade's end, if X outgrew the island by `fastest_margin` (10 points) | how much of the growth moved in from other towns, and what X supplies |
| N works close around X (0.66) | at least `closures_min` (10) works of a district shut in a year and they are `closures_share` (10%) of those that stood, shops and trades aside | what most of them made, and its price a year or three years back, or the price of what they work up, or wages; the largest is named |
| N new towns in the decade (kind `town`, 0.65) | `new_towns_min` (3) or more founded in a decade | which, when, and the largest |

Towns are ranked with their districts (`st.pop`), not by the people inside
the town (`st.urban_pop`), which is a quarter of that and too even between
towns to rank by; hence "first town" and not "largest", and every such story
says "X and its district". A new town takes people and pits from its
neighbours' districts, so nothing is compared across a founding: peaks and
places start again that year. A town in its first five years has no boom or
closures story; a district that never held `min_people` (5,000) is not said
to decline or to be the fastest-growing; a boom or a round of closures is not
told of the same town again within `quiet_years` (15), and a decline only
after a new peak. One story of a kind a year: if several towns qualify the
one hit hardest leads and the others are named in its last sentence.

**Three faults of the observer**, mended only under the switch:

- City milestones fired for the first town past each size and never again.
  Now every town has them; a later one is told more quietly (0.45 and up,
  against 0.55 and up) with the towns that got there first.
- Doublings were measured against 1750 alone, whatever kind of year that was.
  Now against the mean of the first `base_years` (5), and the title says so
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
| `per_decade`, `at_least` | 4, 2 | most and fewest kept in a decade |
| `per_kind`, `inventions` | 1, 2 | most of one kind; of inventions |
| `floor`, `fill_floor` | 0.6, 0.45 | lightest item kept; lightest kept to fill a thin decade |
| `own_great`, `own_small`, `own_summary` | 0.7, 0.5, 0.45 | weights of the player's own news |
| `base_years` | 5 | years a doubling's baseline is the mean of |
| `boom_years`, `boom_growth`, `boom_share` | 5, 2, 0.15 | coalfield boom |
| `decline_years`, `decline_fall` | 8, 0.1 | town in decline |
| `overtake_years`, `overtake_margin` | 3, 0.05 | one town overtaking another |
| `fastest_margin` | 0.1 | fastest-growing town of a decade |
| `closures_min`, `closures_share` | 10, 0.1 | a district's works closing together |
| `new_towns_min` | 3 | a decade of new towns |
| `min_people`, `quiet_years` | 5,000, 15 | no stories of smaller towns; years before a story repeats |

No random number is drawn anywhere in it.

## What the sweeps showed

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
| `news_region` | | 40.9 | | 38.6 |
| `news_total` | 245.8 | 294.2 | 1,485 | 726 |

And what a reader of the major feed meets, six families, the port's rules
against the game's (`--example chronicle -- --seeds 1-8`):

| per run, mean [least .. greatest] of 8 seeds | port | game |
|---|---|---|
| all items | 1,485 [1,324 .. 1,683] | 726 [698 .. 758] |
| the major feed | 1,127 [981 .. 1,328] | 55.2 [53 .. 59] |
| the major feed in a decade | 14 in the 1750s rising to 115 in the 1890s | 3.0 to 4.0, never under 2 or over 4 |
| the player's own items | 1,051 [910 .. 1,254] | 244 [225 .. 260] |
| of them major | all | 11.8 [10 .. 14] |

Major items in a run by kind, port then game: the player's family 1,051 and
12; markets (booms, gluts, slumps) 22 and 9; inventions 10
and 9; slumps dated 11 and 9; regional stories none and 8; turning points 5
and 4; new towns 14 and 2 (the decade's digest); milestones 6 and 2; the
first railway 1 and 1; alarms 8 and none at the end (each has been dated by
then).

Of the 39 regional stories in a run, 8 are kept: about 2 each of coal booms,
overtakings and closures, 1.5 declines, 1 fastest-growing town.

Seed 7 with six families, three decades of the major feed as it now reads:

```text
The 1790s
  1791  Town and country Skelford in decline
        Skelford and its district had 9,652 people at their peak in 1783. There are 8,496 now, 12% fewer.
        Its people leave for Melbourne, where a household's income goes 10% further.
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

## What worries me

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
- **The succession is found by the port's wording.** `chronicle::succession`
  looks for "head of the family, is buried at" in the year's family news. If
  that sentence is reworded the two items simply stay two, as under the port;
  the test would say so.
- **Other mechanisms' news.** Whatever a new mechanism files (a harvest, a
  canal, a strike) is ranked by its weight and kind like anything else: a new
  kind gets one place a decade. Nothing needs registering, but a mechanism
  that files many items at 0.6 or more will crowd the "All" list, not the
  major feed. It can add provenance with `chronicle::why(P, ...)`, which
  returns nothing while the chronicle is off.
- **Events have no stable id.** `rekept` names an event by its place in the
  run's list, which works because the list only grows. Saving and loading a
  game would need the same order kept.
