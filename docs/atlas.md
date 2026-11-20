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
after. The page is 14 MB with every town of the eight islands, of which the
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
islands does: sixteen islands to 2050 took RUNTIME s, eight at a time.

**Which ideas.** Every idea in the data the runs were played on, read when
the command runs: the tree's methods and techniques (`data/content.toml` with
`data/game.toml` laid over it) that are not known in 1750, and every use of
the tasks rule (`data/tasks.toml`). Those known in 1750 are named under the
tables, not drawn. An idea added to the data tomorrow is drawn tomorrow.

**History's year** is the data's own `target`. The overlay's ideas have one;
the port's own (the coke furnace, the powered mill, the engine works,
crucible steel, the rolling mill, railways, improved husbandry, farm
machinery) have none in the data, and their rows say "no year in the data".
The uses carried their year only in comments; they now have a `target` key
too, which nothing in a run reads: the electric drive 1901, the tractor 1915,
the cutter 1920, the combine 1935, office computing 1960 and models 2022,
each the year its comment gives as the year it is to be had. The hoist, a
stand-in with "poor history", has none.

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

## Not yet built

- The panel in the game, and the hand on it: a kind of works forbidden from
  a date, then a recipe's numbers (the backlog's item).
- Prices and volumes of carting by way: that is the second plate, the moving.
- Reading a sweep's folder. A sweep keeps by island only each good's price
  and output and each kind's output (`annual.csv`), which leaves out what the
  drawing needs, so the atlas plays its islands itself at a sweep's cost.
