# The atlas

Plates that draw the island whole, for the eye: what the designer asked for
on 11 November ("flowcharts of industries within a region or market with
prices and volumes and transport costs") and again on 19 November ("Can I see
a production flowchart? See recipe usage"). Four plates are ranked in the
backlog under "The atlas": the making, the moving, the ideas and the ground.
The first and the third are built, each as a page that a command writes; the
panels in the game and the hand on them come later. The third came with a
pin for the lab: an idea found in a year of one's choosing ("The pin",
below, and `docs/lab.md`).

The atlas is a tool, not a rule. It changes nothing a run does, so it has no
switch and no page under `docs/rules/`. The code is
`crates/isostate-core/src/atlas.rs`, whose opening comment is its manual, with
the page's own drawing code in `crates/isostate-core/src/atlas_page.html`;
`tests/atlas.rs` holds it to what is said here.

## The first plate: the making

```text
isostate-core atlas making --out DIR [--seeds 1-8] [--rules port|game] [--families N] [--end-year Y]
                           [--set NAME=VALUE]... [--step 10] [--no-towns] [--jobs N] [--quiet]
isostate-core atlas making --from DIR [--out DIR2]
```

It plays the islands (by default as the lab does: the game's rules, six
families, islands 1 to 8, to 2050), reads each one's books of account once it
is over, and writes one page, `DIR/making.html`. The page is a single file
with its figures and its drawing code inside it: it opens in any browser with
no network, and can be mailed or kept. Beside it are `making.json` (the
figures alone; `--from DIR` draws the page again from them without playing,
which takes a second), `making.txt` (what the page shows, in words) and
`README.txt` (the command and the code it was run on).

`--set NAME=VALUE` changes one of the rules from the start of the run, by the
dotted name `sweep --set` takes (one value each: the atlas draws one
setting). `--step` is the length of a period in years. `--no-towns` leaves
out the towns' figures, which are most of the page's size.

What it costs: nothing new is recorded during a run, since the books already
keep what the page needs, so the atlas costs what a sweep of the same islands
does and a few seconds more to read the books. On 19 November eight islands to
2050 took 115 s on the clock, eight at a time, against 109 s for
`sweep --seeds 1-8 --rules game --families 6 --end-year 2050` run straight
after. On 20 November, with a heap for each thread (`src/heaps.rs`, so that
the islands no longer queue on the process's one heap), it took 73 s, eight
at a time. The page is 14 MB with every town of the eight islands, of which the
islands' own figures are 1.1 MB; one island with its towns is 1.8 MB.

## How to read it

The drawing is the production tree as the islands lived it in one period (a
decade unless `--step` says otherwise). From left to right: the uses of the
land and the pits (farmland, pasture, the woods, quarries, clay pits, iron
mines, collieries), the goods they make, the kinds of works that use those
goods, the goods those make, and so on, to the household needs (food, fuel
and light, shelter, clothing, implements, comfort, drink and groceries,
household goods) and the railways and canals on the right.

- **A line is as wide as the value that flows along it in a year**, averaged
  over the period, in hands' years: the money over a year of one hand's wage.
  Counting in wages takes out the century's inflation, so a line that keeps
  its width from one decade to the next carries the same labour's worth. The
  widest line of the period is drawn 26 pixels wide (or of the whole run, by
  the picker), and its value is written in the bar at the top.
- **Three kinds of line**, told apart without colour: solid for what is made
  and what a works uses up (as material or as fuel, a machine's coal, oil or
  power among it); dashed for what a works, a use of the land or a railway
  buys to hold as plant; dotted for what households buy, ending at the need
  it serves.
- **Each good is coloured by its price against the wage, set against the
  period it was first made in**: 1750 for the port's sixteen goods, and for a
  good that comes later the first period anyone made it (before that its
  price is a ceiling nobody paid). Blue is cheaper against the wage, red
  dearer, at three steps each way for eight times; the factor is written on
  the good. A line takes the colour of the good it carries, so a good gone
  dear stains every works that uses or holds it.
- **A kind of works nobody runs is grey; one the island has not yet found is
  faint.** A good nobody makes in the period has a dashed outline and no
  colour.
- **Under the pointer**, a good gives its price, in money and in months of a
  wage, its factor, what was made, and who made, used, held and bought it; a
  kind of works gives what it makes, the year it was found, the works
  standing, hands and output, its recipe (materials, heat and labour a unit)
  and its plant (goods a unit of a month's capacity). A click holds the
  choice and fades the lines that do not touch it.
- **What moved most** names the five goods whose price against the wage moved
  most since the period before, and for each the kinds of works that hold it
  as plant or use it up, and the needs it reaches in the end, followed
  through the works that use it and what they make. Above them, the three
  dearest goods against their first period.
- **The island picker** has each island and "the middle of the eight": every
  figure the median across the islands, period by period. **The town picker**
  has every town an island ever had: a town's figures are its own, its
  colours set against the island's first period, so a town out of line with
  its island shows.
- **Growth after 1900** gives the observer's chained wage and the share of
  hands working the land, year by year, with the period shaded, and the three
  numbers the standing check reads: the wage's growth from 1900 to 1970, the
  wage of 2050 over 1970's, and the hands on the land in 1900 and 2050. For
  the middle they are the mean of the islands' own, as `sweep --standing`
  reads them, so the page can be checked against a sweep.
- **Recipe usage**, under the drawing: every kind of works in the data, the
  first and last year any works of it worked on each island, and its hands at
  its peak period. Those that never ran on any island are named: the
  content's idle part, as `ablate` names the rules'.
- The page's address keeps the choice (`making.html#island=7&town=6&decade=1940`),
  so a view can be sent as a link; `pin=timber` holds a node.

## Where the figures come from

The books of account (`docs/rules/accounts.md`, on under the game's rules)
keep a year's figures for every town and good, every town and kind of works,
every town and use of the land, every works, and every town, kind of works
and good a works used. The atlas adds them up by period and by island:

- a place's wage is its towns' mean wage weighted by their people;
- a good's price is its towns' mean price weighted by what was bought in each
  (by people where nothing was). The second fall traced on 19 November
  (one town of island 7 whose prices rose two hundredfold in the late 1940s)
  coloured the whole island under the observer's people-weighted price,
  where hardly anyone paid it; weighted by what was bought it does not;
- made, used up and bought are the books' own, at the town's price.

Three things the books keep only in part are shared out by rule, and the
page says how much could not be:

1. **Plant.** The books keep what was bought for building by town and good,
   and what each works paid for plant in money. A good's plant purchase in a
   town is shared among the town's kinds of works by what each paid times the
   share of that good in its recipe's plant at base prices; where no recipe
   holds the good (a machine's plant for tasks), by what each paid.
2. **The land's purchases.** Kept by town and good, and by use of the land in
   money. Shared among the uses that may have bought the good (machinery
   anywhere, rails in the pits, and what `data/tasks.toml` says a use's
   machines run on or hold) by what each paid.
3. **The need a purchase served.** The books keep households' spending by
   need, not by good and need. A good's purchase is shared among the needs it
   serves by how much of each it gives and what the town spent on each.

The drawing is laid out by the code from the content the runs were played on
(the game's overlay over `data/content.toml`, with `data/tasks.toml`) and from
what the books saw. Each good and kind of works has a depth: a good the land
makes is 0, a kind of works is one deeper than its deepest material, and a
good is as deep as its shallowest maker; fuel and plant are left out of the
reckoning, since coal and machinery go into nearly everything. Within a
column, nodes are put in the order of the mean place of their neighbours,
eight passes back and forth. Every line the content names (each recipe's
materials, fuels and plant, the machines' run goods and plant, the goods that
serve each need) is drawn, faint where nothing flows, and so is every line any
island used. A good or a kind of works added to the data is drawn the next
time the command runs, with no place set by hand.

## What the first page showed

Made on `main` at 5bfd60d with the atlas on top, on 19 November 2026, eight
islands under the game's rules with six families, to 2050 (`isostate-core
atlas making --out DIR`). The drawing has 92 nodes (28 goods, 48 kinds of
works, 7 uses of the land, 8 needs, the railways and canals) in 9 columns,
and 640 lines.

- **The wood famine, seen and not seen.** Island 7 played with the woods left
  unmanaged (`--seeds 7 --set woods.enabled=false`) has timber as its
  dearest good from the 1920s on: against the wage it stands at 0.99 of its
  1750 price in the 1920s, 2.75 in the 1950s, 4.1 in the 1960s, 6.3 in the
  1970s and 19 by 2050, and it is the one red node on the page, with red
  dashed lines into the powered mills, the steam sawmills, the bricklayers,
  the masons and carpenters and the timber-frame builders, who hold it as
  plant. Sawn timber, second dearest (2.5 in the 1970s, 15 by 2050), is held
  as plant by the common breweries, the gasworks and the contract builders,
  and the page names them under timber, "through sawn timber". With the woods
  managed, as main plays them, island 7's timber in the 1970s is 0.71 of its
  1750 price against the wage and the 1970s are blue from end to end; timber
  becomes the dearest good again only from the 2010s, at 1.9 by 2050. The
  paper mill of the verdict's list holds timber, but it closes in the 1840s
  on every island; the machine paper mill that follows it holds none.
- **The second fall, in one town.** Kelside, town 6 of island 7, in the 1940s:
  electricity costs 1.74 months of a wage against 0.25 in the 1930s, 7.0
  times dearer, and building work 0.48 against 0.09, 5.3 times: the two goods
  that moved most, while the island as a whole hardly moves (its electricity
  0.26 of a wage in both decades, weighted by what was bought). In the 1970s,
  the second episode, Kelside's electricity costs 28 months of a wage on the
  decade's mean and its building work 17. The colour alone shows it less well
  than the list beside it: electricity opened at its ceiling in the 1890s, so
  against that first period even the town's 1940s read 0.35.
- **Growth after 1900, read off the page**, as the mean of the eight islands:
  the chained wage grows 0.31 per cent a year from 1900 to 1970, the wage of
  2050 is 0.921 of 1970's, and the hands on the land are 0.245 in 1900 and
  0.247 in 2050. The sweep of the same islands gives the same to the last
  digit shown, island by island.
- **Recipe usage.** Of the 48 kinds of works, one never runs on any island:
  the nitrogen works. The brewhouse, the country pottery and the old paper
  mill run from 1750 to the early nineteenth century and stop; the powered
  mill runs from the 1760s to 1800 on and peaks at 14,000 to 17,000 hands.
- **What could not be shared out** is nothing to the third decimal of a per
  cent on these runs: every plant purchase found a works that paid for
  plant, every purchase by the land a sector, every household's good a need.

## The third plate: the ideas

```text
isostate-core atlas ideas --out DIR [--seeds 1-16] [--rules port|game] [--families N] [--end-year Y]
                          [--set NAME=VALUE]... [--pin NAME@YEAR]... [--hold NAME@YEAR]... [--edits FILE.json]
                          [--against DIR0] [--jobs N] [--quiet]
isostate-core atlas ideas --from DIR [--out DIR2] [--against DIR0]
```

What the designer asked on 19 November: "Can I see a tech timeline? Move
things around in it?" The command plays the islands (by default the game's
rules, six families, islands 1 to 16, to 2050) and writes `DIR/ideas.html`, a
page that opens with no network, beside `ideas.json` (its figures; `--from`
draws the page again from them), `ideas.txt` (its tables in words) and
`README.txt`. The code is `crates/isostate-core/src/atlas_ideas.rs`, with the
page's drawing code in `atlas_ideas.html`; `tests/atlas_ideas.rs` holds it to
what is said here.

Each island is played as the lab plays an arm (`edits::play_noting`): with no
`--pin`, `--hold` or `--edits` it is the sweep's own island to the bit, and
with them every island is played with the same list (the pin, below). At each
year's end the command notes, for every idea, the year each town's knowledge
of it first stood above 0.02 (from which a works of it may be built there),
the year each of its thresholds was met and the year all were, and the year
the first works of each method stood. It costs what a sweep of the same
islands does: sixteen islands to 2050 took 178 s on 20 November, eight at a time, on a machine
shared with other builds, and 77 s later that day, all sixteen at once (the
default, half the machine's processors) with a heap for each thread.

**Which ideas.** Every idea in the data the runs were played on, read when
the command runs: the tree's methods and techniques (`data/content.toml` with
`data/game.toml` laid over it) that are not known in 1750, and every use of
the tasks rule (`data/tasks.toml`). Those known in 1750 are named under the
tables, not drawn. An idea added to the data tomorrow is drawn tomorrow.

**History's year** is the data's own `target`. The overlay's ideas have one;
of the port's own, railways (1825) and farm machinery (1850) have theirs from
the overlay since 20 November 2026 (`data/game.toml` says why each), and the
rest (the coke furnace, the powered mill, the engine works, crucible steel,
the rolling mill, improved husbandry) have none in the data, and their rows
say "no year in the data". The uses carried their year only in comments; they
now have a `target` key too, which nothing in a run reads: the electric drive
1901, the tractor 1915, the cutter 1920, the combine 1935, office computing
1960 and models 2022, each the year its comment gives as the year it is to be
had. The hoist, a stand-in with "poor history", has 1850 since 20 November:
steam winding and haulage were common by then, and the stand-in is found
from 1901 by design, so its row stands half a century late.

## How to read the third plate

- **A row for each idea**, in the order of history's year (an idea with none
  stands at its median find). Uses of the tasks rule are in italic.
- **On each row**: a dark red tick at history's year; a dot for the year each
  island found it, filled for islands 1 to 8 and open for 9 to 16 (dots of
  one year are stacked); a pale band from the first island to the last; and
  at the right the band's length in years and "×n" for the islands that never
  found it by the end of the run. A pinned idea carries ⌖ and a diamond at
  its year.
- **Under the pointer**, a row gives each island's year, the year every
  threshold was met (the draw then begins in earnest: at its full height the
  chance is 0.09 a year, a mean wait of eleven years), the year each good was
  met, and for a method the year its first works stood.
- **One island, town by town**: for the island and the idea chosen (a click
  on a row chooses it), a map of the island with each town shaded by the year
  it came to know the idea (dark early, pale late, an open ring never; the
  double ring is the town that found it), and beside it the towns in the
  order they came to know it. The uses of the tasks rule are found by the
  island whole and have no towns.
- **The tables**: the ten ideas found furthest apart, the ten furthest from
  history on the mean, and for every idea one line on what it is found on
  (each good's threshold, written as the data writes it, "xN" being N years
  of the island's own 1750 making; the draw; the year it is sought from; for
  a use, whether it is read a head, and the switch it stands behind).
- **`--against DIR0`** lays another folder's spread under each row as a faint
  grey line with its median, and adds a table of each idea of 1900 and after:
  its spread there and here. That is how the pinned page below is read.
- The page's address keeps the choice (`ideas.html#island=14&idea=electrical_works&show=1900`).

## What the third page showed

Made on `main` at c03d7f3 with the plate on top, on 20 November 2026: sixteen
islands under the game's rules with six families, to 2050 (`isostate-core
atlas ideas --out I1`, 178 s, eight at a time). It draws 41 rows: 34 of the
tree's ideas and 7 uses; 21 ideas are known in 1750.

**The islands find each idea decades apart, before 1900 as after it.** The
years from the first island to the last: the powered mill 71 (1757 to 1828),
shops, offices and warehouses 73, the engineering works 69, the hot-blast
furnace 58, sheet glass 55, farm machinery 53; the electrical works 36 (1867
to 1903) and the coal power station 56 (1872 to 1928). Of the 34 ideas of the
tree, 26 are found more than 30 years apart. The uses that wait for a date
(`found_after`) are close where every island has met their threshold by the
date: office computing 7 years, models none, the electric drive 9; the
tractor (on oil made) 29 and the combine 20.

**Within an island the spread is short.** On island 14, the latest to the
power station (1928, in Cragness), the 23 towns standing then all knew it by
1931; the electrical works, found there in 1903, was known in every town
standing by 1906. A town founded later learns an idea from the town it was
founded from or its neighbours.

**Two causes, both general.** Under the pointer each row gives the year all
of an idea's thresholds were met (made since 1750) against the year it was
found (`target/notes/why.py` in the report reads them for every row):

1. *The thresholds.* They are amounts made since 1750, and the islands reach
   them decades apart. The electrical works asks 9.8 million steel and 1.9
   million machinery: the islands made that much steel between 1876 and 1910
   and that much machinery between 1869 and 1905, so the works was ready
   from 1876 (islands 9, 16) to 1910 (island 1), 34 years. The late islands
   are late at the steel and the machine trades: island 1 made its steel by
   1910, island 10 its machinery by 1905, island 14 by 1897.
2. *The draw.* Each year a number is drawn at 0.18 p&#8308; / (1 + p&#8308;), p
   the least of made over threshold: a chance of 0.09 a year when the
   thresholds are just met, at most 0.18. It finds an idea early on some
   islands (island 5 found the electrical works in 1867, 19 years before its
   thresholds were met) and late on others (island 14's
   power station was ready in 1906 and found in 1928). The draw alone spreads
   sixteen islands that are all ready over some fifteen to twenty years:
   managed woodland had its timber on every island by 1863, is sought only
   from 1895, and was found from 1895 to 1917.

## The pin: electricity in history's year on every island

```text
isostate-core atlas ideas --out I2 --pin electrical_works@1885 --pin coal_power_station@1888 --against I1
```

(186 s.) The two ideas that make electricity and electrical goods pinned to
their own history's years on all sixteen islands. Up to 1885 the islands are
the same as unpinned, idea for idea (a pinned idea still takes its yearly
number, so the island's draws stay its own until the pin first changes a
find). The years between the first island and the last, unpinned and pinned:

| idea | history | unpinned | pinned |
|---|---|---|---|
| Electrical works | 1885 | 1867..1903, 36 | 1885, 0 |
| Coal power station | 1888 | 1872..1928, 56 | 1888, 0 |
| Electric drive (use) | 1901 | 1901..1910, 9 | 1901, 0 |
| Managed woodland | 1902 | 1895..1917, 22 | 1895..1914, 19 |
| Hoist (use) | none | 1901..1927, 26 | 1901..1916, 15 |
| Nitrogen works | 1915 | 1904..1936, 32 | 1903..1957, 54 |
| Tractor (use) | 1915 | 1915..1944, 29 | 1915..1939, 24 |
| Cutter (use) | 1920 | 1920..1937, 17 | 1920..1921, 1 |
| Oil from coal | 1925 | 1901..1942, 41 | 1902..1937, 35 |
| Combine (use) | 1935 | 1935..1955, 20 | 1935..1950, 15 |
| Breeding and sprays | 1950 | 1919..1961, 42 | 1917..1965, 48 |
| Office computing (use) | 1960 | 1960..1967, 7 | 1960, 0 |
| Models (use) | 2022 | 2022, 0 | 2022, 0 |

The lab plays the same experiment and gives the same spreads to the year,
from its facts (`discovered_NAME`, `use_found_year_NAME`):

```text
isostate-core lab --out L5 --seeds 1-16 --pin electrical_works@1885 --pin coal_power_station@1888   (474 s)
```

Its arms part on each island in the year the pin first changes a find: 1867
on island 5 (where the island found the electrical works that year, and the
pin holds it back), 1868 on island 7, and so on to 1885 on seven islands. The
chained wage hardly moves: 11.08 against 10.97 of 1750's in 1950, 16.08
against 15.86 in 2050, up on twelve islands and down on four, well inside
the spread from island to island.

What is found on electrical goods closes: the drive, the cutter and office
computing come within a year. What is found on coal and chemicals does not:
oil from coal stays 35 years apart, the nitrogen works 54, breeding 48, the
tractor and the combine (on oil made) 24 and 15. So the 35 years of
electrical goods are not one fault at the root. They are what every idea of
the tree does, before 1900 as after: a threshold on what the island has made
since 1750, which islands of different ground and trades reach decades
apart, and a draw that adds or takes ten to twenty years more.

**The smallest rule that would bring every idea after 1900 within about ten
years** is a deadline beside `found_after`: an idea not yet found by its
history's year plus a few is found then, as if learnt from abroad, in the
town the pin would choose. Thresholds and the draw would still decide an
early find, and before 1900 nothing would change. Its cost in honesty: today
the first works of the electrical works and of the power station stands in
the year it is found on fifteen islands of sixteen (on island 2 the station
three years after), so lateness lives almost wholly in knowing, and a deadline would
take away the honest lateness with the rest: an island with little coal
would learn the power station in 1893 like the others. The lateness worth
keeping would have to move from knowing to building, by a works that waits
until its inputs are to be had at a price it can bear; the deadline alone
does not do that.

## Not yet built

- The panel in the game, and the hand on it: a kind of works forbidden from
  a date, then a recipe's numbers (the backlog's item).
- The ideas in the game, and the hand on the timeline: an idea dragged to a
  year is a pin (`docs/lab.md`), which the core already carries out; no
  button gives one yet.
- The uses of the tasks rule town by town: they are found by the island
  whole, so the third plate draws them for the island only.
- Prices and volumes of carting by way: that is the second plate, the moving.
- Reading a sweep's folder. A sweep keeps by island only each good's price
  and output and each kind's output (`annual.csv`), which leaves out what the
  drawing needs, so the atlas plays its islands itself at a sweep's cost.
