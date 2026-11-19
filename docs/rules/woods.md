# Woods

Managed woodland, and the woods' cover recorded. Not in the Python. The
switch is `woods.enabled`; `woods.record` keeps the two new series. Both are
off under the port and on under the game's rules. The code is `woods.rs`,
with hooks marked "woods" in `land.rs` (where the woods grow back),
`tech.rs` (where inventions are sought), `observer.rs`, `facts.rs` and
`inspect.rs`; the invention is `managed_woodland` in `data/game.toml`, stage
6 of the tree.

**The standing check passes: twelve of twelve inside.** Islands 1 to 8 with
six families, to 2050, the woods on against off (`isostate-core sweep
--seeds 1-8 --rules game --families 6 --end-year 2050 --set
woods.enabled=false,true --standing --growth`):

| | off [least .. greatest] | on | |
|---|---|---|---|
| people in 1800, 1850, 1900 | 199,119, 360,208, 663,610 [637,563 .. 684,897] | 199,119, 360,208, 663,609 | inside |
| chained real wage, 1850 and 1900 (1750 = 1) | 3.03 [2.55 .. 3.60], 6.27 [5.32 .. 7.17] | 3.03, 6.27 | inside |
| towns in 1900; living in towns | 18.2 [15 .. 23]; 0.465 [0.452 .. 0.474] | 18.2; 0.465 | inside |
| railway in 1900; first line | 2,505 km; 1829 | 2,505 km; 1829 | inside |
| work's share of income, 1850 and 1900; land's in 1900 | 0.651, 0.670; 0.161 | 0.651, 0.669; 0.161 | inside |

Nothing can be found before 1895, so the played century is the same island
to the last digit until then; the few years from 1895 to 1900 on the islands
that find it early move the facts of 1900 in their last digit.

## What was wrong

Run on past 1900 the island ran out of wood. From about 1950 the woods were
cut faster than they grew back, and timber went from a fifth of a month's
wage a unit in 1900 to six months' in 2050. Timber is built into the plant of
the powered mills, the breweries, the paper mills and the gasworks and into
the builders' work, so cloth, beer and shelter grew dear; households spun
their own wool and brewed their own grain, and the farms and pastures took
the hands back. Three diagnosticians and a judge found this on 19 November
2026 (the decline's verdict, in the night's reports): with the woods growing
back three times as fast from 1970, and nothing else touched, the wage rose
from 1970 to 2050 on eight islands of eight. The port's woods grow back at
`forest_regrow` 0.12 a year on what stands, with nothing a forester could do
about it. The island had no way past running out: no imports, no
plantations, and timber in the plant of works that last to 2050.

The woods are a stock on every square: `State::cover`, what stands, up to
what the ground could carry (`World::forest_pot`). Each year it grows by
`forest_regrow * cover * (1 - cover / pot) + 0.01 * (pot - cover)` where the
square has woods or wild land, the woodmen's cut comes off it, and clearing
for farms or towns yields its timber once. Nobody recorded it: the
diagnosis reckoned it back from timber cut and timber's price.

## What it does

**An invention, managed woodland.** The forester's craft: woods planted,
thinned and felled in rotation. It is an invention of the tree, not a works,
as breeding is: `[[tech]] managed_woodland` in `data/game.toml`, stage 6,
headline "Woods planted in rotation at {town}". It is found as the tree's
inventions are, on what the island has made since 1750 (`tech::discovery`),
in the town that handles most of it, and spreads to other towns along the
roads as every invention spreads (`tech::diffusion`).

**Where it is known the woods grow back faster.** In the land's yearly step
the regrowth term of each square is multiplied by `1 + (woods.regrow - 1) *
know`, where `know` is the share of the square's market town that knows the
craft (0.25 in the town that finds it, rising toward one as it spreads). At
`regrow` 3 and full knowledge the port's 0.12 a year is 0.36, the rate the
diagnosis tried. The small drift term `0.01 * (pot - cover)` is left alone,
and so is everything else about how the woods are cut and what uses timber.
Planted and managed woods do yield several times what woods left to
themselves yield, and the date has a precedent: Britain founded its state
forestry, the Forestry Commission, in 1919 after a wartime shortage of
timber.

**Dated, not spurred by dear timber.** The natural spur to plant woods is
dear timber. The tree cannot see a price: an invention is found on a clock of
goods made, with a hazard that rises as the clock runs out, and nothing makes
a finding likelier when a good is dear. A spur on price would be a new rule
for every invention, so this one is dated as the tree dates the others, and
said so. Its clock is timber cut since 1750, 40 million units (an island that
has cut little wood learns the craft late), and it has a floor: a new key of
the overlay, `found_after = 1895`, before which no number is drawn for it
(`Content::tech_after`; the land's machines in `data/tasks.toml` have the
same key). Every island had cut 53 to 62 million units by 1890, so the clock
has run out everywhere by 1895 and the floor is what dates it: found in 1900
on the mean island, between 1895 and 1910.

**Off is off.** With `woods.enabled` false the invention is never sought (no
number is drawn for it from the knowledge stream, so every other invention is
found in the year it was) and the woods grow back at the port's rate. The
game's golden test with the rule thrown off in `game_params()` holds the
economy, the books and the news of both eras to the hashes before it; only
`run.bin` moves, because the record lists the content's inventions and every
town's knowledge of each, and managed woodland is in stage 6's content
whether its rule is on or off. `tests/woods.rs` plays island 3 with the
invention made findable from 1760, the rule off, against the same island
with no such invention in its content, and the two are the same history.

**The woods recorded.** With `woods.record` the observer keeps two series
every month:

- `woods_cover`, the woods' standing cover: what stands on the woodland and
  wild land of the island against what that land could carry, each square
  weighted by the share of it under woods and wild. 1 is every wood as thick
  as its ground allows.
- `timber_in_wages`, a unit of timber at the island's price over the month's
  average wage: timber's price in months of the wage.

The facts read both at the marks, every fifty years to 1900 and every ten
after: `woods_cover_Y` and `timber_in_wages_Y`. When managed woodland is found
is `discovered_managed_woodland`, as for every invention. The inspector's
card for a square shows its woods' cover, as a share of what its ground could
carry, and the share of its market's foresters who manage them.

## Its numbers

`WoodsParams`, in `woods.rs`. Off by default; `game()` gives the second
column.

| | default | game | |
|---|---|---|---|
| `enabled` | false | true | managed woodland may be found, and grows the woods back faster where it is known |
| `regrow` | 3 | 3 | how many times as fast the woods grow back where the whole town knows the craft |
| `record` | false | true | the observer keeps the woods' cover and timber in months of the wage, and the facts read them |

In `data/game.toml`: `discover = { timber = 40000000.0 }`, `target = 1902`,
`found_after = 1895`. `woods.enabled` may be thrown in a running island
(`edits::SWITCHABLE`; tried as every switch is, `tests/edits.rs`).

## What the sweeps showed

Islands 1 to 8, six families, the game's rules, to 2050, in the sweep
above; the means of the facts, with the least and greatest island where it
matters. The lab gives the same numbers island for island (`isostate-core
lab --rule woods.enabled=false@1895 --seeds 1-8 --years
1900,1950,1970,2000,2050`, the game as it is against the woods thrown off
from 1895): the arms part in the year each island finds the craft, 1896 to
1902.

| | 1900 | 1950 | 1970 | 2000 | 2050 |
|---|---|---|---|---|---|
| timber in months of the wage, off | 0.18 | 0.49 | 1.04 | 5.27 | 6.26 [5.53 .. 6.76] |
| on | 0.18 | 0.16 | 0.20 | 0.24 | 0.45 [0.34 .. 0.59] |
| the woods' cover, off | 0.69 | 0.46 | 0.26 | 0.08 | 0.08 |
| on | 0.69 | 0.83 | 0.78 | 0.72 | 0.58 [0.49 .. 0.67] |
| chained real wage (1750 = 1), off | 6.27 | 7.46 | 6.52 | 4.19 | 3.05 |
| on | 6.27 | 7.59 | 7.88 | 7.71 | 7.21 [4.17 .. 10.06] |
| the land's share of hands, off | 0.245 | 0.254 | 0.246 | 0.283 | 0.356 |
| on | 0.245 | 0.241 | 0.229 | 0.216 | 0.247 [0.232 .. 0.259] |
| people, off | 663,610 | 1,041,858 | 1,216,447 | 1,516,710 | 2,392,813 |
| on | 663,609 | 1,042,631 | 1,207,367 | 1,447,244 | 1,941,929 |

Managed woodland is found in 1900 on the mean island (1895 to 1910). With
it the woods thicken for twenty years as the craft spreads (the cover goes
from 0.69 to 0.87 by 1930) and then are cut down slowly, to 0.58 by 2050;
without it they are a tenth of what they were by 2000. Timber stays under
half a month's wage on every island to 2050, where without it it costs six.
The wage in 2050 is higher on all eight islands, by 4.16 on the mean; the
land's share of hands lower on all eight, by 0.108; there are a fifth fewer
people (the transition's doing: a richer island has fewer children); work's
share of income is 0.618 against 0.563.

Against the design's asks for the century after 1900 (the standing check's
own lines):

| | off | on | asked |
|---|---|---|---|
| growth of the chained wage, 1900 to 1970, per cent a year | -0.00 | 0.31 | 1.00 |
| the wage in 2050 over 1970's | 0.50 | 0.92 [0.54 .. 1.25] | 1.00 or more |
| the land's share of hands, 2050 less 1900 | +0.111 | +0.003 [-0.017 .. +0.016] | below 0 |

So the woods no longer run out and the fall after 1970 is nine tenths
mended, but the wage still ends 8 per cent below 1970's on the mean island
and the land's share in 2050 is level with 1900's, not below it. On islands
2, 6 and 7 the wage is as high or higher in 2050 than in 1970; on island 1 it
is level; islands 3 and 5 lose a tenth; islands 4 and 8 lose two fifths.

**On the new land** (`--set land.enabled=true`, which is to become the
game's this week), the same: the standing check twelve of twelve inside;
found in 1900 (1895 to 1907); timber 0.88 of a month's wage in 2050 against
6.92; the cover 0.38 against 0.05; the wage 8.01 in 1970 and 7.11 in 2050
(off 6.42 and 3.18), 0.91 of 1970's; the land's share 0.248 in 2050 against
0.250 in 1900 (off 0.371); 0.53 per cent a year from 1900 to 1970 (off
0.15). The new land's woods are cut faster: by 2050 they are going again,
and three times the regrowth buys about a century there, not for good.

**More regrowth does not mend the rest.** With `woods.regrow` 10 the cover
stays above 0.93 and timber at 0.19 of a month's wage in 2050, and the wage
is 8.70 in 1970 and 6.24 in 2050, 0.71 of 1970's: no better. Nor do pits
that never thin from 1970 (`lab --rule mines.reserve=0@1970` with the woods
on): the wage in 2050 is 4.98 against 7.21, higher on two islands of eight.
What is left of the fall is not the woods' and not the pits'
("What it does not mend").

## What it does not mend

- **A second fall, not the woods'.** With the woods managed, single islands
  still lose a third to a half of their wage in a decade: island 7 from 8.74
  in 1935 to 3.99 in 1950, island 4 from 7.50 in 1970 to 3.66 in 2000, island
  8 from 7.88 in 1970 to 4.28 in 2050, with timber under half a month's wage
  throughout. What grows dear against the wage then is electricity (5.6
  times on island 7 from 1935 to 1950, 3.1 times on island 4), building
  work (3 to 5 times), iron and coal; the budget's shares hardly move. It is
  on the island without managed woodland too (island 7 falls by 1 per cent a
  year from 1900 to 1970 with the woods off), and main has had it since the
  electric drive took the hoist's place in the works: without the woods the
  wage is level from 1900 to 1970 there (6.35 to 6.62), where the diagnosis,
  played before that, had it rising to 8.79. It is for the next stage to
  find; the trace is in the night's notes for this piece.
- **Timber in the plant.** The powered mill's plant still holds timber 10 a
  unit of capacity, the brewery's sawn timber 2, the paper mill's timber 10,
  and the builders use sawn timber. With the woods managed none of these
  starves (timber's price in wages rises 2.3 times from 1970 to 2050, from a
  fifth of a month's wage), but the cut still grows faster than the woods
  and the cover falls after 1930. Taking timber out of the twentieth
  century's plant is the truer cure and is left for later (docs/BACKLOG.md,
  under the note asking for a logging industry).
- **It is a craft, not an industry.** Nobody plants a wood because timber
  pays: the woods are where the land's bidding left them, and the craft only
  makes them grow back faster. The woodmen still cut as the price bids, not
  what grows back.

## What worries me

- **Dated, not spurred.** Every island has run its timber clock out well
  before 1895, so the floor dates the finding and the clock does nothing on
  these islands. A smaller or poorer island would find it later. A spur on
  dear timber would be truer, and would be a new rule for the whole tree.
- **Knowledge, not woods.** The craft acts on every wood in a town's market
  area as soon as the town knows it, as if the woods were replanted in a
  year. The cover rises from 0.69 to 0.87 in twenty years, which is fast for
  oak and about right for conifers.
- **The woodmen are on the land.** The land's share of hands counts the
  woods with the farms and pastures, and managed woods employ more woodmen
  (timber cut is 98,700 a month in 2050 against 16,000), so the craft holds
  the land's share up a little even as it takes the hands off the fields.
- **The wage after 1970 is noisy.** Island by island the 2050 wage over
  1970's runs from 0.54 to 1.25 with the woods on. Eight islands tell a fall
  of 8 per cent from no fall only at about one standard error.
- **`run.bin` moves with the rule off.** The record lists every invention in
  the content, and managed woodland is in stage 6's content whatever its
  switch says. Only the record moves; the economy does not.
