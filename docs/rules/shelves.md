# The shelves go with the district

Not in the Python. One condition in `peopling::move_with_cells`
(`crates/isostate-core/src/peopling.rs`), switched by `people.shelves_go`:
off under the port's rules, on under the game's. It is stage 3 of the pass
over all the rules (`docs/designs/rules-pass.md`, section 4.2), and it mends
the fault that `docs/probes/tr-rent.md` (section 4) and
`docs/probes/tr-carriage.md` (d) traced. It has no numbers.

## What it does

Every year, once the market areas are worked out again, the households of
every cell that now trades through another town go to that town, with their
cash, their landlords' cash, last month's income, their carters, their wage
and their prices ([people.md](people.md), "People go with their cells").
A town founded that year also gets the same share of its donor's
merchants' stock of every good, of the merchants' cash above nothing, and of
their measure of sales, the smoothed record of a month's sales and shipments
by which they mark their prices and order their trade.

With `shelves_go` on, a town that already stood gets those three as well
whenever it takes a district. The merchants who served the district go on
serving it: the town that gives a fifth of its people gives a fifth of its
stock, a fifth of its merchants' cash and a fifth of their measure of sales,
to whichever town takes them, new or old.

Two things are not moved, as they are not moved to a new town. A debt
stays where it is: only cash above nothing goes, so merchants who owe keep
what they owe and give none of it (below, "The giving town"). And the
merchants' other books (what they think a good is worth, last month's
orders) stay, because the prices already go with the trade.

Every one of these is a transfer from one town's shelf or purse to
another's. Nothing is made or lost, and no random number is drawn.

## Why

An old town that took a district kept the shelves and the order book of the
town it had been. Uldale on island 1 went from 3,257 people to 13,637 at the
turn of 1848, and its new people came with their cash and their wages and
bought January's bread from a store laid in for three thousand: they got 0.69
of their food. On sixteen islands of today's game, nine town-years were
short of food (under 0.82 of need in the worst month, the line
`people::growth` draws), and all nine were this: a town that had at least
doubled at the year's end, short in January, February or March. Most of the
game's emigrants were the same January, people taking ship from a town whose
shelves held its old people's grain. And it is why the famine years fell when
the canals came in: canals redraw the districts earlier and by less, so the
railway's redrawing is gentler (`tr-carriage.md`, (d)).

Both halves are needed (`tr-rent.md`, section 5). The stock carries the town
through January. The measure of sales makes the merchants order for its new
people after that: with only the stock moving, four hungry town-years in
spring remained on sixteen islands, in towns whose merchants still ordered for
the old trade.

## What the sweeps showed

Sixteen islands under the game's rules with six families, as section 3 of
the design asks: today's game (the switch off), two null histories of today
(`works.walk` 0.0200001 and `carting.adjust` 0.2500001, each moved by a
millionth, which sends every island down another history), and the rule.

```bash
isostate-core sweep --seeds 1-16 --rules game --families 6 --set people.shelves_go=false --out off --quiet
isostate-core sweep --seeds 1-16 --rules game --families 6 --set people.shelves_go=false --set works.walk=0.0200001 --out null-walk --compare off --quiet
isostate-core sweep --seeds 1-16 --rules game --families 6 --set people.shelves_go=false --set carting.adjust=0.2500001 --out null-adjust --compare off --quiet
isostate-core sweep --seeds 1-16 --rules game --families 6 --out on --compare off --quiet
```

### The stage's own measure: no hungry January after a redraw

`examples/spring_hunger.rs` sorts every town-year short of food by what came
before it; `examples/trace_rent.rs --part d` prints each with its months.

| | today | the rule |
|---|---|---|
| islands 1 to 16: town-years short of food | 9 | 1 |
| of them in January to March after the town took a district | 9 | 0 |
| islands 17 to 24 (`tr-rent.md` for today) | 5 | 2 |
| of them after the town took a district | 2 | 0 |
| islands 17 to 48: town-years short of food | 26 | 7 |
| of them in January to March after the town took a district | 16 | 0 |
| of them later in a year after the town took a district | 3 | 0 |
| in a town founded at one of the last two years' ends | 3 | 2 |
| in a town that had just given a district | 1 | 2 |
| in a district whose harvest was under 0.95 | 3 | 3 |

The one left on islands 1 to 16 is Brindale, island 12, May 1775, 0.80 of its
need: a town founded at the end of 1774, in its first spring. The two on
islands 17 to 24 are of the same kind, Brightheath (island 20, June 1759,
0.73) and Barrigg (island 24, June 1819, 0.81), each in the year after its
founding, when its district's stock had come with it and been sold. A town
in its first two years is not counted as hungry by `people::growth`, so none
of the three is a famine year. With canals and without
(`examples/trace_carriage.rs --seeds 1-16 --set canals.enabled=true,false`,
and `spring_hunger` with `{"canals":{"enabled":false}}` to sort them):

| sixteen islands | canals, today (`tr-carriage.md`) | canals, the rule | no canals, today | no canals, the rule |
|---|---|---|---|---|
| hungry town-years | 6 | 0 | 24 (`tr-carriage.md`); 26 by `spring_hunger` | 5 |
| of them after the town at least doubled | 6 | 0 | 17; 16 in January to March | 0 |
| of them in a town that had just given a district | 0 | 0 | 1 | 4 |

So no town goes hungry after taking a district, with canals or without. On
the island without canals a new kind shows: four of the five hungry
town-years left are in towns that had just given their farmland away
(below, "What worries me").

### The card

The aim of the pass this stage answers is aim 12: famine years 0.2 a run or
fewer, and none in the two months after a town takes a district.

| sixteen islands | today | null, walk | null, adjust | the rule | islands, the rule against today |
|---|---|---|---|---|---|
| famine years, in sixteen runs | 6 | 4 | 14 | 0 | 0 up, 6 down |
| emigrants, a run | 113.5 | 28.3 | 177.8 | 6.8 | 0 up, 12 down |
| years with a sailing, in sixteen runs | 1 | 0 | 3 | 0 | |
| news of dearth, a run | 8.4 | 7.9 | 8.4 | 8.4 | 6 up, 3 down |
| what a wage buys, 1831 to 1850 (1750 = 1) | 1.85 | 1.91 | 1.79 | 1.83 | 8 up, 8 down |
| what a wage buys, 1881 to 1900 | 2.68 | 2.71 | 2.68 | 2.67 | 8 up, 8 down |
| output a head, 1881 to 1900 (1750 = 1) | 2.72 | 2.74 | 2.73 | 2.72 | 8 up, 8 down |
| share of people in towns, 1881 to 1900 | 0.421 | 0.420 | 0.421 | 0.421 | 8 up, 8 down |
| people in 1900 | 725,684 | 726,790 | 722,218 | 724,176 | 5 up, 11 down |
| towns in 1900 | 17.2 | 16.3 | 16.6 | 16.1 | 4 up, 9 down |
| the wage's worst year | -0.153 | -0.153 | -0.155 | -0.153 | 8 up, 8 down |
| the first railway | 1827 | 1827 | 1830 | 1828 | |
| railway in 1850, km | 1,348 | 1,327 | 1,238 | 1,298 | 6 up, 10 down |

So aim 12 is met: no famine year on any of the sixteen islands, and no town
short of food after it took a district. Emigration falls about as the traces
said it would (113.5 a run to 6.8, where `tr-rent.md` had 8.3 and the critic
6.8): what is left is 19, 19, 40 and 32 people on islands 1, 2, 4 and 12. Read
the emigrants beside the null pair, though: in today's game one January after
a redraw sends hundreds to sea, and moving a number by a millionth takes the
mean from 113.5 to 28 or to 178. The rule's 6.8, lower on twelve islands and
higher on none, is below every history of today.

Everything else is inside what the null pair moves. 496 of the 561 facts
differ from today's, because every island's history parts at its first
redraw, and each null history differs in as many. Of some 560 facts, seven
lean one way on twelve islands of sixteen or more; one fact in thirteen does
that by chance, so about forty would. Two of the seven are the rule's own
(the emigrants, and the books' measure below); the others (the wild land of
1800, iron ore on the canals in 1840, royalties in 1800, land's share of
income in 1900, industry's share of work in 1751 to 1770) each move by one to
four times what the nulls move them, in no pattern. The scorecard's loss
falls from 151 to 54, and that is not the rule: the towns' share has the
same mean, but one island's 0.384 widens its spread across islands, and the
card takes its tolerance from that spread (`scorecard.md`, "What worries
me"). Of the design's other aims (section 2), those that facts on main
already read (people in 1900, the towns' share, what a wage buys, the first
railway and its length) are in the table and do not move beyond the null
pair. Those that wait on stage 1's facts (the wild town-months of aim 13,
the five worst years of aim 7, the dear years of aim 11) were not read.

### The books

`books_goods_worst`, the books' worst goods balance of the run as a share of
what passed through a town, goes from 4e-15 to 4e-12, higher on every island.
That is rounding, and the books do hear of the stock. The residue in the worst
town and good of each of islands 1 to 4 is between 2e-19 and 9e-16 of a unit
(Ryeport's rails in 1866, Olddale's charcoal in 1805, Stonefield's and
Holmouth's rails in 1881). It is a larger share only because the share is
taken of what the town's own books passed (its opening stock, what it made
and what it received), and a town that held a few thousandths of a unit of
rails can now be sent a share of another town's: the books count that stock
in their `moved` column, as they count any stock that changes between months
(`accounts::sync_stock`), but not in the denominator. The design and
`tr-rent.md` read the same rise as the books not hearing of the stock; they
do. The books are left as they are, because counting moved stock in the
denominator would move the measure with the switch off too, and their own
test (every year under 1e-9, `tests/accounts.rs`) holds under the game's
rules with the rule on.

```bash
cargo run --release -p isostate-core --example accounts -- --seeds 1-4 --rules game
```

## The giving town

The critic of the design asked for one more reading (section 10.3): the
merchants' cash goes only "above nothing", and late in the century many
merchants owe, so the stock and the trade go and the debt stays. Does the
town that gave the district feel it? `examples/shelves_givers.rs` takes every
town that stood before a year and has a tenth fewer people after its end,
and reads it either side of the year's end and through the year after:

```bash
cargo run --release -p isostate-core --example shelves_givers -- --seeds 1-16
cargo run --release -p isostate-core --example shelves_givers -- --seeds 1-16 --set '{"people":{"shelves_go":false}}'
```

| sixteen islands | today | the rule |
|---|---|---|
| towns that gave a district | 369 | 314 |
| share of their people that went, mean | 0.164 | 0.166 |
| their stock kept | 0.95 | 0.84 |
| their measure of sales kept | 0.94 | 0.83 |
| their stock in months of sales, before and after (median) | 1.74 and 1.76 | 1.76 and 1.77 |
| the year after: the worst month's food got (mean); towns under 0.82 | 1.68; none | 1.65; none |
| the year after: grain's price over the year before's | 0.99 | 0.99 |
| the merchants' dividends a head, the year after over the year before (median) | 1.28 | 0.84 |
| givers whose merchants owed | 87 | 59 |
| their merchants' cash a head before and after (median) | -38.9 and -46.4 | -27.5 and -38.7 |
| months they owed in the year after | 11.6 | 11.9 |

(Today a giver's stock still falls a little, because some gave to a town
founded that year.) The giving town feels nothing in its food or its prices.
Stock and sales leave in the same share, so the stock in months of sales, by
which the merchants mark their prices, stays where it was. The debt stays,
but it stays without the rule too, since nothing ever moved a debt: merchants
who owed owe all through the year after either way, and the debt a head rises
by the share of people that left either way. The one difference is the
merchants' dividends to the town's households, which today rise a head after
a giving (the merchants keep the cash and the trade of a larger town and pay
it out to fewer people) and with the rule fall a little (the cash above
nothing went with the trade). Nothing here asks for the debt to go with the
district, so the design's rule stands as it was written. Where the debts come
from is traced in `docs/probes/tr-loose-a.md` (section 1): they are the
losses of a few towns that bring their services in and let a glut of them
spoil, which is a fault of its own and not a district's to carry away.

## Tests

`tests/shelves.rs`:

- the switch is off under the port, on under the game, and absent from the
  port's `params.json`;
- a redraw made by hand on island 3 in 1760, where Highford gives Dunsea the
  cells along their border (0.092 of its people): with the rule the same
  share of every good's stock and of every measure of sales goes, to a part
  in a million million, the island's total of each is unchanged, the
  merchants' cash above nothing goes, and money, the merchants' cash and the
  households' cash are each the same to the penny, with the rule and without;
  without it the old town's shelves are exactly as they were;
- merchants who owe keep the debt, and the stock goes all the same;
- island 14 to 1771, played both ways: without the rule Wyrside, two and a
  half times its people at the end of 1769, gets 0.79 of its food in February
  1770; with it no town that took a district goes short in January to March.

With the condition put back as it was, the last three fail.
`tests/people.rs` keeps two of its histories on the shelves as they were
(the port's island with people, and island 2's sailing of 1827, which came
from Carford's hungry January and which the rule ends).

## What worries me

- **The dearth's own rule now has almost nothing to act on.** With no famine
  year on sixteen islands, the dearth's three numbers, `people.emigration` and
  the emigration facts are idle, as the design foresaw ("What could go
  wrong"). They are the rule for a hungry year and are wanted the day
  something makes the island hungry; stage 9's faster hands on the land are
  the next thing that might.
- **The town that gives its farmland away.** The share that goes is the
  share of the giver's people, good by good alike. When the cells that leave
  are the town's farms, it loses two fifths of its people and nine tenths of
  its own grain, and keeps three fifths of a grain store that its own fields
  used to refill. Without canals four of the five hungry town-years left on
  sixteen islands are such towns, short in March or April: Wyrside (island
  14, 1826; its grain made fell from 67,807 to 4,643 and its households got
  0.18 of their need in April), Kirkgate (island 3, 1827, 0.25), Dunford and
  Redfell (island 14, 1833, 0.59 and 0.56). Today's game without canals has
  one. Under the game as played (with canals) there are none on islands 1 to
  16 and two on islands 17 to 48 (Melhope, island 33, July 1826, 0.33;
  Ivelside, island 34, March 1838, 0.33), against one today.
  It is the stock cycle of a town newly turned importer (`tr-loose-b.md`,
  section 4) with a thinner cushion. A share good by good (the district's own
  part of what the town made and sold of each) would fit the trade better
  than a share of people; it was not tried, because the design asks for the
  rule as it is written, and the January it mends is far the larger thing.
- **A new town's first spring.** The three town-years short of food that are
  left, on twenty-four islands, are towns in the year after their founding,
  whose district's stock came with them in December and was sold by June.
  The people rules give a new town two years' grace, so none counts as a
  famine. It is the same shape as the spring shortages of
  `tr-loose-b.md` (section 4): a month's buyers draw only on the store as the
  month began.
- **Only the share goes, not more.** A district takes the stock its old
  market held for it, and that market may have held little: a town that
  takes most of its new people from a market whose own shelves were thin
  still opens thin. The largest jump that went short in today's game was
  Ryesea's, 4.6 times its people; with the rule no jump went short, but a
  redraw much larger than any seen here has not been tried.
