# A life cycle for works

`works.enabled`, in `crates/isostate-core/src/works.rs`. Off under the port,
on under the game's rules. None of it is in the Python.

Under the port the island ends the century with four and a half thousand
works, seven thousand with six families, nearly all of them small, standing on
a sixth of the land. A works is founded at two fifths of its trade's usual
size and then hardly grows; what grows is their number. With the life cycle a
works that earns its keep grows where it stands, year after year, until its
cell is full, and then opens a sister works next door. In 1900 there are
some 650 works, the median one six or seven times its trade's usual size, a
few of them a hundred times, on about 130 cells beside the towns. Those are
the port's rules with the life cycle alone, as it was built. In the game as
it is played now, with every other rule on and six families, there are 738
works in 1900, the median one 6.0 usual sizes and the largest 121, on 199
industry cells (the mean of eight seeds, `isostate-core sweep --seeds 1-8
--rules game --families 6`, 12 November 2026).

## Why the port's works stay small

The diagnosis is in `.claude/night/reports/dx-firms.md`, measured on eight
seeds. In brief:

- **A works never finishes building.** An order for capacity is delivered a
  quarter of the remainder each month, so it is "open" for five to seven years,
  and a works with an open order may not grow, is never counted idle and is
  never closed as too small. A works can grow about once in six years and
  loses a fifth to depreciation meanwhile.
- **It may not pass four usual sizes**, and a usual size is a handful of
  hands in several trades (a smithy five, an engine works eighteen).
- **Families found a small works a year in every branch**, half of all works
  standing in 1900.
- **A works of the usual size takes a quarter of a cell whatever its trade**:
  a five-man smithy stands on a square kilometre.

So growth in demand is met by more small works, and the map fills with them.

## What the life cycle does

Six parts, lettered as in the diagnosis. Each has its own number or switch in
`WorksParams`, so `--set works.walk=0` (say) measures the rest without it.

**A. A works has finished building when little is left on order**
(`built_share` 0.05). `works::building`: a works counts as building while more
than a twentieth of its capacity, built and on order together, is still to
come (and always while nothing is built yet). The yearly review
(`firms::review`) can then grow a works every year, shrink it when it stands
idle and close it after three years in a row of losing money while it stood
nearly idle (a return below nothing and under 15 per cent of its capacity
used, the port's test), which the port's flag had been preventing.

**B. Land by hands, and the cell as the size limit** (`hands_per_cell` 2,000,
`cell_limit` 0.8). `works::land_take` is the one place that says how much land
a works needs: its capacity times its trade's hands per unit, over the hands
that fill a cell. A smithy of five men takes a yard; a mill of a thousand
hands half a cell. The rent it bids, the cell's built share
(`State::cell_K_share`), the size limit and the room a newcomer looks for all
come from that function, and `works::land_held` is what one standing works
holds: its capacity built and on order, because the ground is taken the day
building starts. A works grows where it stands until it would take four
fifths of its cell (`works::max_K`) or the cell is full of its neighbours
(`works::cell_fits`). After that the growth goes to a works of the same
concern in the town that still has room (`works::extend_sister`), and if there
is none, to a new sister works the size of that growth on a cell within two of
it that has room (`firms::spawn_sister`, `works::sister_K`). No cell is ever
asked to hold more works than it has ground for; `tests/works.rs` checks that
every year of a run (seed 7 to 1829, the port with the life cycle, no
families).

A sister works, or a family's second works of a kind in a town, takes the
first one's name with a number: "Ashby Mill No. 2". The port numbers it by
counting every works whose name begins with the first one's, so a family's
"Ashby Mill" was numbered on from the town of Ashby's "Ashby Mills", and
"Denton Steel" from "Denton Steelworks". Under the life cycle only the works
of that very name count (`names::works_of_name`). Works that have closed
still count, so no name is given twice in a run: when "Ashby Mill" has shut,
the next is "Ashby Mill No. 2", as a colliery's third pit kept its number
after the second was shut. Names move no number in a sweep.

A works that grows needs more land, and pays for it: `State::works_lease` is
the most land each works has taken so far, and an order that takes it past
that adds to its monthly rent (`works::ordered`). The new ground costs what
the cell's land earns that year (the bid a new works makes there), and never
less than the works pays for the ground it has. The second half matters more
than it looks: once the works have taken three fifths of a cell the port's
land rule gives the rest to nobody, no other use is left to set a price, and
by 1900 three works in four stand on such cells (seed 7). Without the floor
their growth there came free. That is the port's land rule with the life
cycle alone. Under the game's rules kept town ground puts a price on such a
cell (`rent_floor`, `towns::price_ground`; `towns.md`): it costs what it
would earn as open land, and a cell of the town itself costs what the
housing a works displaces would pay. `works::ordered` reads the same price,
so the ground a works grows onto there has a price of its own; how often the
floor still binds has not been measured. Under the port a works' rent is fixed
for life at what it bid when it was a tenth of the size. Rent is a small cost
either way: 3 to 5 per cent of the works' wage bill (seed 7). The seed 7
numbers in this paragraph were measured with the life cycle alone and have
not been measured again under the game's rules.

**C. Entry counts the merchant's spread** (`count_spread`). The merchant takes
6 per cent of the price off everything a works sells (`merchant_spread`), and
`firms::evaluate_sites` left it out, so a site promised tens of points of
return more than a works there would get. `works::entry_spread` takes it off
the price entry reckons with. The families' site sums come from the same
function, so the player's "expected return" is now the return a works gets.
With this part alone switched off (eight seeds, measured before the review's
repairs) 35 per cent more works are founded and 41 per cent more close, a year
and a half younger, for the same output: the works the false promise called up.

**D. A year's room for new capacity** (`year_room` 0.15, `room_share` 0.34,
`entry_share` 0.10). Each trade in each town may order in a year at most 15 per
cent of the town's monthly trade in what it makes (`State::outflow`, sales and
shipments smoothed), and never less than one smallest works, so a trade nobody
is in yet can always start (`works::year_budget`). The works already there are
served first, in order of the return they earned on their capacity last year
(`works::new_year` sorts the review), no one of them taking more than a third
of the room (`works::growth_allowed`). Entry takes what is left
(`works::sites_with_room`) and builds one works sized to its market, a tenth of
the town's trade within the room and the cell (`works::entry_K`); the port's
doubling of a works that promises over 60 per cent, and its second works a
year, are dropped. `State::works_added` keeps the tally.

This is the part that decides how hard the island invests. Without it (four
seeds, no families) output per head reads a third higher but the real wage is
no higher, a quarter of all capacity stands idle and twice as many works close.
Most of the extra looks like capital goods for capacity that then stands idle
(nobody took the accounts apart). A room of 0.3 instead of 0.15 (eight seeds,
before the review's repairs) gives output per head 3 to 6 per cent higher in
1800 and 1850 and the same in 1900, no gain in the real wage, a quarter more
closures, a fifth more industry cells, and prices that swing 8 to 17 per cent
harder (grain and cloth in every seed). 0.15 is the steadier place to stand.
What it costs is under "What worries me": a trade far too small for its
market can only grow at the room's pace.

**E. A growing family branch extends a works it has** (`family_extends`). In
`families::grow`, where the branch would have founded another works of a
method it already runs in that town, the capacity is ordered at its largest
works there that has room, paid from family cash at the same price
(`works::family_extension`, `works::extend_for_family`); the player hears of
it, at once or, with the chronicle, in the year's summary of what the
branches did. A branch founds a new works only where it has none, or none
with room.

The size of a works the player founds, and of the site sums the player sees,
is `works::entry_K` or, where the family's cash does not reach that far, the
works it can pay for, never under the smallest (`families::founding_K`,
`works::within_purse`). A tenth of a big town's trade is many usual sizes, and
without this a family too poor for it could found nothing there at all, which
the port never refused. The player's own choices are otherwise untouched, with
one addition: a works needs room on its cell, and the site list says so where
there is none.

**F. Hands walk to work** (`walk` 0.02). Hands live in the town. Labour per
unit of output rises by 2 per cent for each unit of path cost between the
works and its market (`works::walk`, applied in `firms::evaluate_sites` and to
every works' labour in `econ::step_month`). A light trade drifts no further
out than the walk is worth, so mills, steel, engine works, smithies and shops
stand in and by the town (98 per cent of hands within two cells of it), while
coke furnaces stay towards the coal and brickworks on the clay.

Every payment here is one the port already makes, between purses
`econ::money_total` counts: an order is paid from the works' cash, its backers'
or its family's, and rent goes to the land's owner. A concern that cannot pay
for an extension is found out before anybody's money is drawn. Switched off,
every function in `works.rs` that the port reaches returns the port's number
(`year_budget` and `town_trade` are not switched, but are only asked for by
functions that are) and the two state tables stay empty; `tests/golden.rs`
holds the port to the byte.

## Two mends from the pass over the rules

Two faults that `docs/probes/tr-loose-a.md` traced (sections 3 and 4) are
mended here, each behind a switch of its own in `WorksParams`, off under the
port and on under the game's rules. Unlike parts A to F they act whether or
not the life cycle is on, and `--set works.enabled=true` alone leaves them
off. Their measure is `examples/works_mends.rs` (below, under "Measuring it"),
on sixteen islands under the game's rules with six families, 13 November 2026.

The core's table of 20 November 2026 (`docs/core.md`) found both idle: no
fact of the island stands apart with either off. Both stay on, for what a
player sees and no fact counts. Without G the works of 1749 stand as twins on
one cell, drawn as one yard and told closing twice in the news, and the trade
a new game offers first can change. Without H a family that pays for an
extension can see its money paid back out as dividends and the works closed
for the plant's debt; the sweep has no fact of the families' orders.

**G. The works of 1749 stand on cells of their own** (`spread_first`). The
artisans a game starts with are the port's (`sim::seed_firms`): each town's
need for a trade is split into up to six works, and each is sited in turn at
the best of its candidate cells. Nothing in that choice sees the works already
put down, and eight of the candidates are the same every time (where the
trade's bulkiest input is cheapest), so the six often land on one cell, with
one size and one history: two to six smithies, brickworks or charcoal burnings
side by side in a town of 1750, closing in the same year and told in the news
one by one ("Crowther's Forge closes" twice in one month). The rule: each
works of 1749 takes the best of its candidates that no works of its trade in
its town already stands on; if every candidate has one, the best of the
town's other cells where a works could stand (`firms::can_stand`, the
candidates' own test); and only if the town has no such cell left, the
candidates as they were (`works::first_sites`). It draws no chance, so it is
the same in every run of an island, and it sizes nothing: each town has the
same capacity of each trade as before (`tests/works.rs`).

| sixteen islands | today | the rule |
|---|---|---|
| works of 1749 sharing a cell with one of their trade in their town | 1,254 of 2,599 | none |
| exact twins in a run (the same cell, method, year opened and year closed) | 2,450 | 1,543 |
| of them works of 1749 | 953 | none |
| sister works | 834 of 4,276 | 873 of 4,236 |
| investors' works | 403 of 23,841 | 451 of 23,542 |
| families' works | 260 of 3,832 | 219 of 3,637 |

The other three in five twins are not the works of 1749 and the rule leaves
them alone. They are sister works (`firms::spawn_sister`), investors' works
and families' works that a later year's choice puts on the same best cell as
another of their kind in the same year: on sixteen islands 10 in the 1760s,
rising to 200 to 220 a decade in the 1880s and 1890s, and about as many with
the rule as without (`works_mends`'s count by decade).
The same words would not cover them: they are sited by three other rules, in
years when the cell they share already has works of other kinds on it, and
the rule for 1749 knows its town's whole list of works because it makes them
all at once. In the picture each works of 1749 now has a yard on its own
cell, where before the site planner fitted two to six yards of one trade into
one cell (`.claude/night/shots/opus/rules-5c-works/`, not in the repository:
Brambridge and Granness on island 7 in 1750, before and after).

**H. A works pays for its order before its owners** (`keep_order`). A works'
plant is delivered a quarter of what is still on order each month, and paid
for as it comes, while its dividend is 0.4 a month of all its cash above four
months of costs (`econ::step_month`). So money put in for an order (a
family's extension, paid into the works' cash at once by
`works::extend_for_family`; or a new works' founding money) went straight back
out to the owners while the plant was still to come, and the works then paid
for the deliveries out of nothing. `tr-loose-a.md` followed Ashby Mill No. 5
on island 4: extended for 230,970 in 1888, it paid 157,767 of it back to the
family as dividends within the year, ran its cash to -118,688 paying for the
plant, and the review closed it, sound, as insolvent. The rule: the cash a
dividend is reckoned on is less what the works' order still costs, the goods
to come at its town's prices as the month leaves them (`works::kept_for_order`).
It is a rule for every works, the player's among them; what a family is
offered and how it decides are as they were.

| sixteen islands | today | the rule |
|---|---|---|
| orders of more capacity for standing works | 302,515, worth 1,971 million | 297,761, worth 1,886 million |
| of them for works that closed within three years | 212, worth 5.53 million | 100, worth 0.14 million |
| of those a family's | 118, worth 3.91 million | 62, worth 0.12 million |
| the largest such order | 254,087 | 8,026 |

What still closes within three years of an order is the ordinary failure of
small works, a few thousand each (steel rail works that a family's branch
grew in a town that soon had several, millwrights and rail forges in the
1810s to 1830s). With both mends on, 111 orders worth 0.15 million.

**What else they move.** Read as the pass reads a stage (its design,
section 3): sixteen islands under the game's rules with six families, today's
game beside a null pair (`works.walk=0.0200001`, `carting.adjust=0.2500001`),
each mend alone and both (`.claude/night/reports/opus/rules-5c-works/s16-*`,
not in the repository). With both switches off the game is today's: 0 facts
of 626 differ.

| sixteen islands | today | null pair | G alone | H alone | both |
|---|---|---|---|---|---|
| aims of the card met; its loss | 8; 41.4 | 9, 9; 31.1, 38.7 | 9; 30.1 | 9; 32.0 | 9; 29.7 |
| facts moved island by island by a paired t over 3 | | 4, 6 | 11 | 4 | 13 |
| the works' share of income, 1800 / 1900 | 0.075 / 0.087 | 0.086, 0.092 / 0.093, 0.090 | 0.081 / 0.088 | 0.090 / 0.087 | 0.097 / 0.089 |
| works standing in 1800 / 1900 | 233 / 739 | 245, 248 / 720, 721 | 235 / 717 | 246 / 725 | 260 / 718 |
| the families' worth in 1900, island by island against today (the player's) | | +19, +9 per cent (+22, +19) | +11 (+32) | +1 (+11) | +5 (+8) |

The card moves no more than the null pair does, nor do the works' share of
income and the families' worth. H moves nothing beyond the null pair at all.
`tr-loose-a.md`'s trial of it found 9 per cent more works in 1800 on eight
islands of eight; on sixteen that is today's history lying low, since both
null histories have 5 and 6 per cent more too, and H alone 5. G moves one
thing clear of the null pair, which nobody foresaw and which follows from
what it does: the building trades of 1750. With a town's brickworks of 1749 no longer all on the one cell where
clay is cheapest, fewer clay pits are sunk in the warm-up (79 for 87 in 1750,
fewer on fifteen islands of sixteen), a sixth fewer hands dig clay in 1750
and a tenth less clay a head is dug, and stone takes its place (2.7 per cent
more a head, on all sixteen). It is gone by 1800. (The road from where the
brickworks went to the clay pits is read from the facts, not traced.) With both on, a few late
facts stand at the edge of the null pair: the clay pits' hands in 1900 (0.017
of all hands for 0.020), the farms' share of hands in 1881 to 1900 (a fifth of
a point lower) and the cells wholly the works' in 1800 (15 for 18.5).

**The check.** A second reader made the measures again on the tree with
stage 5 of the pass merged in. `works_mends` gave the builder's sixteen
islands to the digit; with both switches off 0 facts of 627 differ from
main's on islands 1 to 8, and the game's four golden hashes are main's. On
islands 17 to 24, which the build did not use, the faults answer the same
way: works of 1749 sharing a cell 550 of 1,303 before and none after, twins
1,196 and 782 (737 with both), orders for works that closed within three
years 120 worth 1.38 million and 49 worth 0.18 million (77 worth 0.16
million with both). Read again with the null pair: the building trades of
1750 move as above, and one thing more is G's. The works of 1749 that the
rule moves off their trade's best cell stand on a second-best one, and more
of them close in the first decade: 429 of 1,899 by 1760 for 348 of 1,944,
so that works three to five years old close at 0.050 a year for 0.042
(the null pair 0.046 and 0.046); by 1800 fewer of them have closed than
before (1,569 for 1,629). The commons scenario runs to 1900 on all sixteen
islands with both on. Two things a player meets. The trade a new game offers
the player first is the best prospect at the seat, which reads where the
works of 1749 stand, so G changes it on 7 islands of 24 (island 7's from
building materials to engineering). And a game saved before these mends is
played again from the new start: it opens, says the island has drifted, and
refuses the orders the new start does not allow (on island 7, three
brickworks of a save's four orders, the family no longer knowing building
materials), as `save.rs` says a save made under other rules does.

## I. The ground a works takes: by its method and its hands at work

`works.by_method`, with `works.keep` and `squares.keep`: off under the port,
on under the game's rules, and like G and H a switch of its own. Built on 26
November 2026; its measure is `examples/works_ground.rs` (below).

**Why.** Part B gives every works 2,000 square metres of ground for each hand
of its plant, whatever it makes: a cell of two kilometres holds 2,000 hands.
That is a refinery's ground, not a mill's or an office's. Measured on islands
1 to 8 under the game's rules (six families, the mean of the eight), the
works held 0.55 of the houses' ground in 1850 and 0.67 in 1900, but 1.45 in
1950, 2.32 in 2000 and 3.04 in 2050 (177,827 squares of 125 metres against
58,591): offices 1,033 km², steel-frame builders 444, converters 417, care
254. Two things made it so. The offices and their kind took two hundred times
an office's ground. And the books counted the plant's hands (its capacity at
the method's labour a unit) where the hands at work had fallen far below them:
the plant's hands were 1.8 times the hands at work in 1850 and 1900 and 4.0
times in 2050, so that a converter works held 18,000 square metres for each
man in it.

**The rule.** Each method carries its square metres a hand in the content
(`ground` in `data/game.toml`, read into `Content::m_ground`; the table and
its sources are at the end of that file). A works' ground is its hands at work
times that number:

- A method that cannot be found before 1901 (`found_after`: offices,
  steel-frame builders, hospitals and homes of care) takes history's number:
  10, 30 and 60 square metres a hand. Its hands are its plant's hands at full
  work: the plant's at the method's labour a unit, fewer as the works grows
  more productive (`f_prod`) and as machines take their tasks (`tasks`'
  factor), never more than the plant's (`works::ground_need`).
- Every other method can stand before 1900 and keeps today's number, 2,000
  square metres a hand of its plant, so that nothing of 1750 to 1900 moves.
  History's number is written beside it (`ground_history`, from 10 for the
  handloom to 1,500 for a power station) and waits for the stage that lays the
  old towns out anew. Such a works holds ground for the share of its plant's
  hands the machines leave it (`State::works_hands`). No machine for a task is
  found before 1901, so to 1900 that share is 1 by cause, not by date, and the
  first era of the golden test holds.

**When hands leave.** Buildings do not vanish in a month. Once a year, at the
head of the review, each works settles its ground with what its hands need
(`works::settle_ground`): it keeps what it holds, and pays rent on it, until
its hands need under `works.keep` (0.8) of it, and then gives back down to
what they need, its rent falling in the same measure. Hands that come back
take ground at once, at the cell's rent, as an order does (`works::ordered`).
On the squares a works keeps its squares while its ground in the books is at
`squares.keep` (0.7) of them or more (`squares::keep_squares`; on a full cell
the works keeping most over its ground gives way first). The squares' log
flickered: on island 7 to 2050, 303,033 of its 1.88 million changes gave a
square back to the tenant it had within three years, as a works' ground in
the books rose and fell by a square. With the rule, 25,942 of 790,582 do
(islands 1 to 8: 22,384 to 27,508 each, against 289,141 to 351,150).

**Rent** is paid on the ground held. For a works of the new methods the rent
follows its ground both ways; for the old, it falls with the hands' share as
machines take hands, and is otherwise as part B has it (the most ground it
has taken, priced when it took it).

**Works smaller than a square.** The books hold any share of a cell. The
squares pool a cell's shares into whole squares by the largest remainders
(`squares::share_out`), so a works under half a square may hold none of its
own: it stands in a square a neighbour holds, or in a house block. Works under
one square of ground in the books, islands 1 to 8 (with no square of their
own): 165 (73) in 1850 and 95 (32) in 1900 either way; in 2050 411 (267) for
133 (64), nearly all of them offices, builders and care homes, which are now
yards and buildings in a town and not slabs. The piece after this one puts
shops and offices inside the house blocks.

**`cell_limit`** (0.8 of a cell) is left as it is. It still binds every old
method at 1,600 hands of plant; for an office at 10 square metres a hand it is
320,000, so offices are limited by the year's room alone.

### What it does (islands 1 to 8, to 2050)

`works_ground --seeds 1-8` with the rule off (`--set
'{"works":{"by_method":false}}'`, main's books to the bit) and on; the mean of
eight islands:

| | 1900 | 1950 | 2000 | 2050 |
|---|---|---|---|---|
| works' squares, off / on | 42,161 / 45,827 | 110,098 / 57,830 | 143,269 / 64,540 | 177,827 / 61,981 |
| houses' squares, off / on | 63,293 / 62,441 | 75,844 / 87,708 | 61,685 / 70,267 | 58,591 / 65,401 |
| works over houses, off / on | 0.67 / 0.73 | 1.45 / 0.66 | 2.32 / 0.92 | 3.04 / 0.95 |
| works' ground in the books, km², off / on | 659 / 659 | 1,720 / 826 | 2,239 / 901 | 2,779 / 861 |
| works standing, off / on | 1,216 / 1,216 | 2,410 / 2,117 | 2,252 / 1,706 | 2,304 / 1,470 |
| m² a hand at work, off / on | 3,663 / 3,663 | 5,407 / 2,526 | 6,394 / 2,502 | 7,935 / 2,457 |
| rent over sales, off / on | 1.5% / 1.5% | 4.5% / 1.5% | 4.0% / 1.4% | 4.2% / 1.5% |
| rent over profit before rent, off / on | 8.2% / 8.2% | 25.3% / 10.8% | 23.4% / 11.1% | 20.1% / 9.1% |
| what landowners take from works a year, millions | 0.69 / 0.69 | 7.2 / 2.2 | 25 / 11 | 87 / 48 |

In 1900 the books are the same to the bit; the squares hold 9 per cent more
for the works, which keep squares while their plant ebbs (`squares.keep`).

The squares, island by island in 2050 (works over houses): 0.86, 0.85,
0.90, 1.04, 0.82, 0.99, 1.05 and 1.15, for 2.88, 2.66, 2.85, 3.11, 2.65,
2.63, 3.38 and 4.79 with the rule off. Where the works still hold more than
the houses it is the old methods' 2,000 square metres a hand of plant: the
converters alone hold 23,000 squares on the mean island. A town cell of 2050 (island 1's cell 3318, the cell of its
largest town with the most works) was two converter works of 182 and 65
squares with three squares of houses; it is now two converter works of 124 and
87 squares and a brewery of 44.

**The rest of the island** (islands 1 to 8 and 17 to 32, to 2050, six
families, the rule off and on side by side: `isostate-core sweep --seeds
1-8,17-32 --rules game --families 6 --end-year 2050 --set
works.by_method=false,true`; 0 of 48 runs failed):

- **What a wage buys** (the linked wage, `real_wage_linked`, over 1750's):
  1966 to 1975 13.6 for 16.2 on islands 1 to 8, higher on all eight (by 1.2
  to 3.9), and 13.5 for 16.0 on islands 17 to 32, higher on all sixteen;
  2041 to 2050 15.8 for 19.4, higher on all 24. Less rent, and works sized by
  what pays rather than by room: the study's five times the room (15.4 for
  13.7) did the same for the same reason.
- **Towns' share of people** in 2050: 0.582 and 0.590 on islands 1 to 8,
  0.592 and 0.587 on 17 to 32; island by island up to 0.03 either way on 1 to
  8 and up to 0.06 on 17 to 32 (island 28, 0.598 for 0.536). In 1970 it is
  higher (0.591 for 0.610). In 1900 it is the same to the digit.
- **People**: 1.26 million in 2050 for 1.23 (lower on 23 islands of 24).
- **Hard spells**: the real wage's worst year after 1800 is a little deeper
  (37 per cent down for 39 on 1 to 8, 31 for 36 on 17 to 32; half the islands
  each way); famine years 0.4 for 0.6 and 0.8 for 0.8 a run.
- **The works**: 2,371 standing in 2050 for 1,500; the works' share of
  income 0.158 for 0.141; closures in the news 494 for 474 a run.
- **Processor time** of a run to 2050, side by side in one sweep: 160 s for
  119 on islands 1 to 8, 153 for 113 on 17 to 32: a quarter less, from fewer
  works.

**The largest works** are now offices. In 2050 the five largest on islands 1
to 8 have 35,000 to 45,000 clerks at work, 0.17 to 0.22 of their town's
works' hands and 0.26 to 0.38 of their island's office work, on 47 to 61
squares; before, the largest were hospitals of 1,200 to 1,300 hands on 170 to
200 squares, each 0.01 of its town's hands and 0.02 of its good, stopped by
their cells. Two offices of one town often stand side by side at the same
size (on island 4, 45,249 and 45,248 clerks): the year's room is shared out a
third to each, so the two largest grow in step.

**What `cell_limit` still binds.** Every old method: in 2050 the works on a
cell built 0.95 or more are 14 an island for 223, and none of the five largest
is at its own limit. For offices, builders and care it no longer binds at all.
`examples/works_ground.rs` prints both.

### What worries me in part I

- **The old methods still hold too much.** Their 2,000 square metres a hand
  of plant is the books' number for 1750 to 1900, not history's (10 for a
  handloom to 1,500 for a power station), and a converter works of 2050 still
  holds 13,000 square metres for each man in it. They lose ground only as
  machines take their hands' tasks. History's numbers wait for the stage that
  lays the old towns out anew.
- **Offices of 45,000 clerks**, a third of an island's office work in one
  concern. The room no longer stops them; nothing else does yet.
- **The squares hold more than the books**: 12 per cent in 2050, 9 per cent
  in 1900, as buildings stand while their plant ebbs. Before 1900 that is the
  only thing part I changes, and only on the squares.
- **People** are 3 per cent fewer in 2050, on nearly every island, and the
  towns' share moves by up to 0.06 on one island. Not traced.

## J. The works of a street live in the blocks of houses

`works.in_blocks`, with `works.block_m2` (65) and `works.block_weight` (4):
off under the port, on under the game's rules, and a switch of its own that
acts only with part I. Built on 26 November 2026; its measure is
`examples/works_blocks.rs` (below).

**Why.** A shop, an office and a nurse's round do not stand on ground of
their own beside the town: they are in it, on the street floor and above.
Under parts B and I they were works like a foundry, each with its yard, and
a town's middle came out on the squares as a slab of offices.

**Which works.** By cause, from the content (`works::in_blocks`): a method
whose site is floor, history's ground a hand at most `block_m2` (65 square
metres, the content's "shops, offices and warehouses together"), and whose
good is sold where it is made, to the households round it: a good they buy
for a need that is not carted between towns or is carted at `block_weight`
or more (services, at 4). By today's content that is the shops and trades,
the emporium, the offices, care, and the builders (the masons, carpenters,
bricklayers, contractors and steel-frame builders, whose work is on others'
ground and whose own is an office and a yard in the street). Not the
handloom (10 square metres a hand, but cloth is light to cart), the smithy,
the brewhouse, the potbank, any mill or the gasworks. Neither half of the
cause alone sorts them: by the ground alone a storeyed mill (30) and an
engineering shop (50) would be in the blocks, by the good alone the gasworks.

**In the books** only the methods by history's number (part I) let floor:
offices, care and the steel-frame builders, none of which can be found before
1901 (`works::floor_in_books`). The shops and trades, the emporium and the old
builders stand from 1750 and keep their yards in the books at today's 2,000
square metres a hand, with the rest of the methods of 1750 to 1900, so that
nothing of the first era moves; the squares put them in the blocks already.
On a town's cell such a works' ground is floor let in the cell's house blocks
(`State::cell_floor`, by cell, as `cell_K_share` is for the yards; which one a
works counts in is `State::works_floor`, settled once a year at the head of
the review as its cell joins a town or leaves it). Off a town's cells it has
a yard as any works.

- **What it pays, and to whom.** As for a yard: the rent it bid when it was
  founded and, for ground it takes later, what the cell's houses would pay
  for it (1.25 times the cell's rent that year, never less than it pays now:
  `ordered`), monthly, to the owner of the cell it stands on, the town's
  landlords or a family. Nothing new is paid.
- **What it does to a house's rent.** The floor let takes room from the
  dwellings as a yard did: a cell's room is its clear room times its `dens`
  times `towns::room_beside_works` of its yards and its floor together, and
  the house rent that clears a town's room ([ground.md](ground.md)) rises as
  that room falls. So the rent of a house is what it was under part I, to the
  same ground; what changes is that the town's map no longer counts the
  offices as a works district: `can_stand`, the mines' room, the site of a
  new town and the record's `works` layer read the yards alone, and
  `cell_fits` reads both.

**No `cell_limit` but the ground.** A works of a method by history's number
has no size limit on its cell (`max_K` is unbounded): it grows where it stands
while its cell holds its ground and it pays for it (`cell_fits`, `ordered`).
The old methods keep their 0.8 of a cell (1,600 hands of plant), since lifting
it moves the first era: it binds before 1900. **Sister works** stay as they
are, and are now opened only when a works' cell is full: its growth goes to a
works of its concern in the town with room, or to a new one on a cell within
two.

**On the squares** a works of the blocks on a town's cell holds no square
(`squares::floored`); each house block says how much of its floor is let and
to what (`squares::Table::floor`), which the record's log carries
([squares.md](squares.md), "Floor in the blocks"). Since the squares are read
by no rule, they put the shops, the emporium and the old builders in the
blocks from 1750, at the books' own ground for them (2,000 square metres a
hand of plant), so that in 1900 a builders' or a shops' quarter is let whole.

### What it does (islands 1 to 8, to 2050)

`works_blocks --seeds 1-8` (the census of the squares and of the largest
works) and a sweep of the three sets of rules side by side: main (part I
off), part I alone, and part I with J (`isostate-core sweep --seeds 1-8
--rules game --families 6 --end-year 2050 --set works.by_method=false,true
--set works.in_blocks=false,true`; 32 runs, none failed; with part I off the
two arms of J give every fact the same).

The squares, the mean of islands 1 to 8, with J (and part I alone):

| | 1900 | 1950 | 2000 | 2050 |
|---|---|---|---|---|
| works' squares | 37,667 (45,827) | 56,138 (57,830) | 64,194 (64,539) | 60,574 (61,981) |
| of them on a town's cells, shops, offices, care and builders | 0 (8,179) | 0 (2,182) | 0 (576) | 0 (667) |
| house blocks | 69,931 (62,441) | 89,213 (87,708) | 72,948 (70,267) | 68,285 (65,401) |
| of them with floor let | 8,159 | 2,214 | 656 | 738 |
| eighths let to shops / offices / care / builders | 28,620 / 0 / 0 / 35,658 | 299 / 1,243 / 858 / 14,760 | 89 / 1,702 / 2,131 / 994 | 0 / 1,724 / 2,713 / 1,137 |
| works over houses | 0.54 (0.73) | 0.63 (0.66) | 0.88 (0.92) | 0.89 (0.95) |

Island by island in 2050 the works over the houses are 0.80, 0.94, 0.77,
0.92, 0.73, 0.90, 0.97 and 1.14 (part I alone: 0.86, 0.85, 0.90, 1.04,
0.82, 0.99, 1.05, 1.15). By 2050 the shops and trades are gone (six works on
eight islands: the offices took the services' trade), so what is let is
offices', care's and the builders' floor, 697 blocks' worth on the mean
island (5,574 eighths), a twelfth of the floor let in 1900, when the books'
2,000 square metres a hand of the shops and the old builders let 8,000.

**The standing check** (`--set works.in_blocks=false,true --standing`): 12 of
12 inside, every one the same number either way (nothing of the first era
moves), and the growth asks met: the linked wage 1.41 per cent a year from
1900 to 1970 (1.42 with part I alone), 2050 over 1970 1.17 (1.20), the hands
on the land 0.121 less in 2050 than in 1900 (0.126).

**From landfall** (island 7, `works_blocks --seeds 7 --start landfall`): no
works' square on a town's cells is a shop's, an office's, care's or a
builder's in any year; in 2050 101,805 works' squares, 120,553 house blocks
(1,049 of them with floor let: 3,799 eighths to offices, 1,937 to care, 2,240
to builders), 4,396 park, 1,512 pits, 475 derelict, 28,080 open; the largest
works two offices of Garside of 23,282 and 23,273 clerks, founded in 1973
and 1975, 0.23 of the town's works' hands.

The rest of the island, the mean of islands 1 to 8, main / part I / part I
and J:

- **What a wage buys** (the linked wage over 1750's): 1966 to 1975 13.6 /
  16.2 / 16.0, higher than main's on all eight; 2041 to 2050 15.8 / 19.4 /
  18.7, higher on all eight.
- **Towns' share of people** in 2050: 0.582 / 0.590 / 0.573; island by island
  +0.04 to -0.04 against main.
- **People** in 2050: 1.263 / 1.225 / 1.226 million.
- **Hands in care** in 2050: 0.165 / 0.157 / 0.158 of all at work, in
  history's band (0.15 to 0.22) on the mean; four islands of eight are a
  little under it (0.139 to 0.146), as with part I alone.
- **Hard spells**: the real wage's worst year after 1800 is -0.37 / -0.39 /
  -0.44 (every arm's worst years fall after 2000); famine years 0.4 / 0.6 /
  0.4 a run.
- **The works**: 2,304 / 1,471 / 1,559 standing in 2050; the works' share of
  income 0.158 / 0.141 / 0.158.
- **Processor time** of a run to 2050, side by side: 186 / 183 / 201 seconds.

### The largest works, and what stops an office

In 2050, islands 1 to 8 (hands at work in December; `works_blocks`):

| | works | an island | median | the five largest |
|---|---|---|---|---|
| offices | 266 | 33 | 793 | 33,083, 32,595, 32,108, 31,762, 29,682 |
| care | 2,735 | 342 | 83 | 3,150, 3,147, 3,037, 1,915, 1,757 |
| builders | 718 | 90 | 5 | 9,282, 7,457, 5,364, 5,198, 5,143 |
| shops and trades | 6 | 1 | 80 | 188, 123, 116, 80 |

With part I alone the five largest offices had 37,732 to 45,249 hands. Every
large town's largest employer is an office, of 0.13 to 0.27 of its works'
hands and 0.06 to 0.19 of its people in work; the largest share of any town
of 2,000 works' hands or more is 0.29 (a builder of 611 in Carstrand, island
8). No town has one employer with a third of its hands.

**Why the largest come in pairs.** The near-equal offices are two works of
one town on one cell, founded by investors a year or two apart, not sisters
(island 2's Sunford: 4,973.6 and 4,973.4 clerks, the same 9,056.1 of capacity
on order). Each year a works loses 4 per cent of its plant (`depreciation`)
and, while it earns over the hurdle and works its plant at over 0.65, asks
for up to half its plant again; the year's room (part D) is shared out best
earner first, no one works taking more than `room_share` of it. The first two
in the queue both ask for more than their third and both get exactly a
third: K' = 0.96 K + a, with the same a, which comes to one size, a / 0.04,
whatever each began at (the gap shrinks by 4 per cent a year; island 4's
pair had 1,784 of capacity between them in 1920 and 9 in 2050). The third in
line gets what is left, a little less. Care's works do it too. It is an
artefact of an equal ration against a proportional wear, not of anything an
office does.

**What stops an office**, in the rules as they are: the year's room, a share
of its town's trade in services of which it takes at most a third, against
its wear; and growth only while its return clears the hurdle and it works
its plant at over 0.65 (the largest offices' plant runs six or seven times
their hands at work, by productivity and office computing). The cost of its
floor does not bind: rent is 1.5 per cent of the works' sales. Neither did
`cell_limit` (320,000 hands of plant for an office). So an office is held to
a fixed share of its town's custom, and a town that buys more services grows
its two first offices in step without end.

**What a one-employer town would need**, and what I would do: nothing yet.
No town has a third of its hands in one works, and the cap that holds the
offices is a share of the town's own trade. What I would change is the
ration, not add a limit: share the year's room in proportion to each works'
asking (or to its plant) instead of a third to each in the queue, so that
two works of one town stop converging to one size and the order of the queue
stops deciding the island's largest works. That is a change to part D, which
acts before 1900, and so a piece of its own with the first era anchored
again.

### What worries me in part J

- **The old methods' floor is the books' 2,000 square metres a hand.** On the
  squares the shops, the emporium and the old builders are in the blocks from
  1750, but at the books' ground for them, so a builders' quarter of 1900 or
  1950 is a run of blocks let whole (36,000 eighths, 4,500 blocks, on the
  mean island in 1900; 110,000 on island 2 in 1950). History's 15 and 30
  square metres wait, as in part I, for the stage that lays the old towns out
  anew.
- **Processor time**: a run to 2050 takes 8 per cent more than main's and 10
  per cent more than part I's, side by side. Not traced: there are 6 per cent
  more works than with part I alone, and more sister works, since a full town
  cell now sends a growing office to a sister.
- **The hard spells after 2000 are a little deeper** (the worst year -0.44
  for -0.39 with part I alone and -0.37 on main), as are the towns' share on
  islands 3, 4 and 8 (0.04 under main's). Not traced.
- **Care's hands** stand under history's band on four islands of eight.
- **A record of layout 1** (before 26 November) keeps its squares in a form
  the viewer no longer reads; it opens without them and says so.

## What was tried and left out

- **Amalgamating small works, stopping economies of scale at four sizes,
  sharper sampling of sites**: the probe found no effect from any of them.
- **A size limit of their own for craft trades** (shops, smithies, handlooms
  and the like stopping at four usual sizes). Measured on four seeds: 790
  works in 1900 instead of 645, with 126 shops instead of 79 and 89 smithies
  instead of 36, but 2,423 works closed in a run instead of 1,390, and a wave
  of 500 closures in the 1860s. A craft works at its limit throws off sister
  works that die. Not worth it. A "works" of shops is then a street of them:
  about 80 on the island in 1900, some 750 hands each. If shops on every
  town cell matter to the picture, draw them from the services capacity of
  the cell, not from the count of works.
- **Utilisation net of the spread.** A works decides how hard to run on its
  margin over the gate price, before the merchant's 6 per cent
  (`econ::step_month`, `firm_demand`). Taking the spread off there is the same
  as raising `util_target_margin` by six points. On four seeds it left two
  points more capacity idle and closed 13 per cent more works, with output and
  wages inside the spread between seeds. Left as the port has it: the works'
  books do count the spread, and its rule of thumb for how hard to run is a
  number to tune, not a sum to correct.
- **A warm-up under the port's rules.** The warm-up decade runs under the
  rules of the run, so 1750 opens with about 280 works, not 600 (with the
  life cycle alone; in the game as it is played now, 219, between 190 and 267
  on eight seeds). Tried the
  other way (four seeds): 1750 then opens with 558 works, the millwrights a
  mean path cost of 27 from their market, and 294 works close in the 1750s
  (82 otherwise) while the trades move to town. The player's first decade
  would be a clearance that no history explains. Starting in the life cycle's
  own equilibrium is right.
- **A wider room for a good that is dear** (the review, sixteen seeds each).
  The year's room is a share of what a town sold, so a trade far too small
  for its market is let grow only at the pace of the little it sells (below,
  under the worries). Three ways of letting a shortage widen it were built and
  measured. Counting the trade at the good's usual price, a ten-year average,
  for the room and for the size of a new works: without families the dearest
  year for tools falls from 7.6 to 4.4 times the median, machinery in the
  1750s from 1,440 to 890, and coke and engine works are found seven and ten
  years sooner, for the same real wage; but with six families the real wage in
  1900 is 1.36 times 1750's instead of 1.53 and tools are no steadier, because
  a family's works is sized to the same inflated trade and nothing rations it.
  For the room alone: the real wage without families falls from 1.52 to 1.44,
  closures rise 6 to 9 per cent and tools are hardly steadier (6.2 against
  7.6). Only above twice or one and a half times the usual price: no better
  than nothing. None of them is in the code. What they share is that a run
  differs from its neighbour by more than the cure moves it, so anything
  short of a large effect is lost; the price rule is the better place to
  look first. (The sweeps: `sweep-works-review-v3` for the thresholds,
  `-v5-s16` and `-v5-f6-s16` for the trade at usual prices, `-v6-s16` and
  `-v6-f6-s16` for the room alone, against `-v4-s16` and `-v4-f6-s16`.)

## Before and after

The port against the port with `works.enabled=true`, without families and
with six. These are the port's rules with the life cycle alone, as the review
measured them; a sweep of today's game cannot give them and they were not
measured again (the game as it stands is under "In company"). Means over
eight seeds, with the least and greatest seed in brackets where it matters.
The first table is seeds 1 to 8, the seeds the mechanism was built on; the
second is seeds 9 to 16, which nobody had run before the review.
The sweeps are `sweep-works-review-base`, `-base-f6`, `-base-s9`,
`-base-f6-s9`, `-v4-s16` and `-v4-f6-s16` under `.claude/night/reports/`.

Seeds 1 to 8:

| | port | works | port, 6 families | works, 6 families |
|---|---|---|---|---|
| works 1750 / 1800 / 1850 / 1900 | 597 / 1,536 / 2,804 / 4,512 | 282 / 302 / 499 / 604 | 598 / 2,131 / 4,156 / 6,973 | 282 / 318 / 624 / 668 |
| works in 1900, range | 4,076 to 4,933 | 549 to 686 | 6,285 to 7,607 | 621 to 722 |
| per thousand people, same years | 4.4 / 6.7 / 7.1 / 6.6 | 2.1 / 1.3 / 1.3 / 0.9 | 4.4 / 9.3 / 10.5 / 10.2 | 2.1 / 1.4 / 1.6 / 1.0 |
| size in 1900, median (K over the usual) | 0.84 | 7.4 (6.4 to 8.9) | 0.46 | 6.6 (6.0 to 7.1) |
| 90th percentile | 2.9 | 43 (35 to 53) | 2.5 | 42 (38 to 49) |
| largest | 3.69 in every seed | 104 (90 to 121) | 3.69 | 116 (107 to 127) |
| capacity held by the largest 1 per cent | 0.080 | 0.115 | 0.058 | 0.134 |
| works founded in a run | 8,655 | 1,744 (1,480 to 2,171) | 11,879 | 1,963 (1,699 to 2,216) |
| works closed | 4,710 | 1,422 (1,191 to 1,795) | 5,472 | 1,576 (1,352 to 1,821) |
| median life of the closed, years | 11.6 | 10.0 | 13.1 | 10.0 |
| utilisation in 1900, mean over works | 0.81 | 0.76 | 0.84 | 0.76 |
| industry cells 1750 / 1800 / 1850 / 1900 | 204 / 332 / 542 / 886 | 2 / 11 / 63 / 126 | 204 / 359 / 601 / 981 | 2 / 18 / 69 / 132 |
| farm area in 1900, cells | 1,282 | 1,431 | 1,251 | 1,399 |
| quarries, pits and mines in 1900, cells | 1,193 | 1,426 | 1,188 | 1,438 |
| output per head over 1750, in 1800 / 1850 / 1900 | 1.40 / 1.48 / 1.46 | 1.56 / 1.61 / 1.57 | 1.39 / 1.55 / 1.50 | 1.63 / 1.59 / 1.63 |
| real wage over 1750, in 1800 / 1850 / 1900 | 1.33 / 1.34 / 1.29 | 1.42 / 1.48 / 1.48 | 1.31 / 1.35 / 1.35 | 1.50 / 1.35 / 1.57 |
| price swing: iron | 0.219 | 0.225 | 0.202 | 0.238 |
| rails | 0.204 | 0.215 | 0.200 | 0.232 |
| cloth | 0.083 | 0.102 | 0.084 | 0.106 |
| grain | 0.086 | 0.092 | 0.090 | 0.096 |
| towns in 1900 | 19.2 | 17.6 | 19.8 | 17.5 |
| last town founded | 1798 | 1822 | 1802 | 1831 |
| closures in the news | 46 | 131 | 45 | 151 |
| the converter is built in | 3 seeds of 8 | none | 2 of 8 | none |

Seeds 9 to 16:

| | port | works | port, 6 families | works, 6 families |
|---|---|---|---|---|
| works 1750 / 1800 / 1850 / 1900 | 613 / 1,578 / 2,622 / 4,423 | 275 / 272 / 510 / 626 | 613 / 2,275 / 4,222 / 6,832 | 275 / 339 / 595 / 676 |
| works in 1900, range | 4,000 to 4,654 | 542 to 724 | 6,528 to 7,145 | 632 to 763 |
| size in 1900, median / 90th percentile / largest | 0.86 / 2.9 / 3.69 | 6.7 / 43 / 102 | 0.45 / 2.6 / 3.69 | 6.3 / 43 / 117 |
| works founded / closed | 8,127 / 4,282 | 1,646 / 1,296 | 11,551 / 5,298 | 2,068 / 1,668 |
| median life of the closed, years | 11.9 | 9.9 | 13.0 | 9.1 |
| utilisation in 1900 | 0.86 | 0.78 | 0.87 | 0.71 |
| industry cells in 1900 | 868 | 127 | 969 | 135 |
| output per head over 1750, in 1800 / 1850 / 1900 | 1.42 / 1.48 / 1.48 | 1.55 / 1.69 / 1.62 | 1.40 / 1.54 / 1.52 | 1.62 / 1.74 / 1.59 |
| real wage over 1750, in 1800 / 1850 / 1900 | 1.34 / 1.27 / 1.32 | 1.46 / 1.56 / 1.55 | 1.31 / 1.36 / 1.38 | 1.49 / 1.55 / 1.49 |
| price swing: iron / rails | 0.211 / 0.189 | 0.223 / 0.217 | 0.206 / 0.178 | 0.258 / 0.247 |
| cloth / grain | 0.085 / 0.083 | 0.106 / 0.095 | 0.084 / 0.085 | 0.109 / 0.099 |
| towns in 1900 / last town founded | 18.2 / 1798 | 17.1 / 1834 | 18.9 / 1802 | 17.1 / 1832 |
| the converter is built in | 3 seeds of 8 | 1 of 8 | 2 of 8 | none |

So the effect holds on seeds it was not built on. What is clear of the spread
between seeds, in both sets: everything about the works (count, size,
founding, closing, land) and the cloth price swinging a quarter harder. The
land the works give back goes to the farms and pastures (a tenth more of each)
and, a fifth more, to quarries, pits and mines. Output per head is higher in
every seed in 1800 and in 5 to 7 seeds of 8 after. The real wage is higher in
1900 in 6 to 7 seeds of 8 everywhere; in 1850 it is higher in 6 to 8 of 8
except with families on seeds 1 to 8, where it is level (4 up, 4 down: one of
those seeds is in a tools famine that year).

Prices swing harder, and more than the first eight seeds said. Without
families iron and rails swing 3 to 15 per cent harder, inside the spread. With
six families it is 16 to 18 per cent on seeds 1 to 8 and 26 per cent for iron
and 39 for rails on seeds 9 to 16, where every seed with the life cycle swings
harder than every seed without, grain included (17 per cent).

By decade (sixteen seeds, no families) output per head is 8 to 14 per cent
above the port's from the 1760s on, and the real wage 8 or 9 per cent above in
the 1780s to 1800s, 12 to 14 in the 1810s to 1840s and 16 to 25 from the
1850s. One thing behind it shows in the goods: smithies that can grow make
twice the tools in the 1780s and three to four times as many from the 1790s to
the 1810s, and tools are what raises output on the land. Cloth, iron and steel
cost half to two thirds of what they do under the port from the 1830s, and by
the end nearly twice as much steel is made.

In 1900 with six families (four seeds: 3, 7, 11, 21; `examples/works_census.rs`)
the island's 699 works are 154 crucible steel works (median 22 usual sizes,
the 90th percentile 65), 134 mills, 94 engine works, 82 brickworks, 77 streets
of shops, 71 coke furnaces, 48 smithies, 25 rolling mills and a dozen
millwrights. 186 are sister works and 182 belong to families (about 3,500 in
the port, by the probe's count). 98 per cent of hands work within two cells of
their market and 14 per cent on town ground. The works that close are the
small fringe, with a median life of ten or eleven years, while the large ones
last.

### Hard spells

Each run's worst five years for the real wage after 1800, and its deepest
fall from an earlier five-year peak (sixteen seeds; the mean over seeds, then
the worst seed):

| | port | works | port, 6 families | works, 6 families |
|---|---|---|---|---|
| worst five years, real wage over 1750 | 1.07, 0.91 | 1.31, 0.95 | 1.22, 1.10 | 1.32, 0.82 |
| deepest fall | 27%, 37% | 22%, 43% | 20%, 25% | 25%, 52% |
| tools: dearest year over the median year since 1780 | 2.2 | 7.6 | 1.9 | 6.6 |
| tools: years above three medians | 0 | 5 | 0 | 4 |
| machinery: dearest year over the median | 12 | 7.3 | 9.8 | 10.7 |
| machinery: years above three medians | 17 | 10 | 16 | 14 |

The floor is higher than the port's on average. The falls are no shallower,
and with families the worst of them is twice as deep. Under the port the hard
spell is a machinery famine when farm machinery arrives; here it is machinery
and tools together (see the worries).

### Output per head after 1865

The probe saw output per head peak around 1860 to 1870 and sag by 1900 under
its prototype, where the port was flat. On sixteen seeds both do it. Without
families the decade means from the 1840s to the 1890s are 6.8, 7.0, 7.1, 7.1,
6.9, 6.8 with the life cycle and 6.2, 6.2, 6.2, 6.4, 6.4, 6.2 under the port:
4 per cent off the peak against 3. With six families it is 6.9, 7.1, 7.4,
7.2, 7.0, 6.8 against 6.4, 6.6, 6.7, 6.7, 6.5, 6.3: 8 per cent against 6.
What the works rules change in those decades is the arrival of farm
machinery: engine works can now grow, so without families machinery peaks at
190 in the 1850s instead of 510 in the 1870s. After that nothing new raises
output on the land while people go on multiplying (`dx-growth.md`), and output
per head drifts down at about the port's rate. Nobody ran the counterfactual
without farm machinery, so take this as the likely reading, not a proof.

### The scripted players

`examples/bots.rs` on seeds 1, 2, 3 and 7, `--rules port` against `--rules
game`, which when this was measured was the port plus the life cycle and
nothing else (it is now the whole game: see "In company"). Net worth in
1900, the player's rank among the six families in 1775/1800/1850/1900, works
founded and standing:

| seed | strategy | port: net worth | rank | works | works: net worth | rank | works |
|---|---|---|---|---|---|---|---|
| 1 | passive | 24.1m | 4/3/2/2 | 1,101, 837 | 27.0m | 2/3/3/2 | 50, 30 |
| 1 | landlord | 37.9m | 2/2/2/1 | 919, 688 | 50.0m | 2/3/1/1 | 53, 30 |
| 1 | founder | 24.6m | 3/2/2/2 | 1,256, 896 | 20.1m | 2/3/3/3 | 162, 39 |
| 1 | thoughtful | 35.0m | 2/2/1/1 | 987, 645 | 29.3m | 2/2/1/2 | 185, 46 |
| 2 | passive | 22.4m | 1/1/1/1 | 856, 701 | 22.2m | 1/1/1/1 | 44, 20 |
| 2 | landlord | 28.3m | 1/1/1/1 | 941, 746 | 35.4m | 1/1/1/1 | 51, 31 |
| 2 | founder | 23.0m | 1/1/1/1 | 1,022, 811 | 24.5m | 1/1/1/1 | 141, 38 |
| 2 | thoughtful | 21.9m | 1/1/1/1 | 817, 573 | 27.2m | 1/1/1/1 | 168, 41 |
| 3 | passive | 17.4m | 4/1/2/3 | 908, 680 | 19.4m | 3/1/1/1 | 50, 34 |
| 3 | landlord | 23.9m | 2/1/1/1 | 841, 672 | 30.5m | 2/1/1/1 | 55, 29 |
| 3 | founder | 16.7m | 4/1/3/2 | 1,004, 741 | 17.7m | 3/1/2/1 | 126, 37 |
| 3 | thoughtful | 21.7m | 2/1/1/1 | 838, 562 | 28.2m | 3/1/1/1 | 153, 49 |
| 7 | passive | 17.2m | 4/2/4/4 | 922, 734 | 24.7m | 4/4/1/1 | 36, 27 |
| 7 | landlord | 24.5m | 4/2/1/1 | 916, 708 | 28.0m | 4/3/2/1 | 47, 25 |
| 7 | founder | 16.8m | 4/3/4/4 | 980, 739 | 20.7m | 4/4/4/1 | 129, 26 |
| 7 | thoughtful | 23.5m | 4/2/1/1 | 987, 652 | 33.2m | 4/4/1/1 | 170, 46 |

The player ends in the same range or a little richer (the landlord gains in
every seed and the thoughtful player in three of four; the land is still the
best asset). Its rank in 1900 is the same or a place either way, except that
on seeds 3 and 7 the passive player and the founder rise from third or fourth
to first. A family now holds a few dozen works, each of them something,
instead of seven hundred. The richest rival in 1900 stays in the same range
(18 to 38 million under the port, 17 to 34 here).

## Beside the other mechanisms

- **The price rule** (the stock markup and the merchants' glut rule that
  `dx-cycles.md` recommends) works month by month on prices and on how hard a
  works runs. The year's room works once a year on how much capacity is
  ordered. Both read `State::outflow`, neither writes it, and they do not
  touch the same lines. They pull the same way: the room is the damper on the
  investment side of the cobweb. It was tuned here without the price rule, at
  the point where a wider room starts buying idle capacity; with steadier
  prices a wider room (0.2 to 0.3) may give more output for the same
  steadiness. That wanted one sweep of `works.year_room` once both were
  merged, read with six families and with the hard spells beside the means,
  because the tools famines below are half made of slow prices (in the worst
  one seen, tools stayed above ten times their usual price for four years
  after the stock had begun to pile up). The sweep has been run (the next
  section): 0.15 stands.
- **Kept town ground** (`dx-towns.md`) can read `works::land_take` for the
  land any works needs and `works::land_held` for what a standing works
  holds; `cell_K_share` is the sum of the second over a cell's works and
  never passes 1. With the walk, the towns report's "town trades on town
  ground" rule should not be needed: 98 per cent of hands already work within
  two cells of their market. Only 14 per cent are on town ground itself,
  because the town's own cells are few; a town that keeps its ground will
  grow over the works beside it. A newcomer's bid for ground on a cell the
  works have mostly taken was the port's floor of a hundredth (the cell has
  no other rent to outbid), and this page asked whoever priced town ground to
  price that too. Kept town ground has done it: under the game's rules such a
  cell costs what it would earn as open land (`rent_floor`,
  `towns::price_ground`), and `works::ordered`, which reads the same
  `State::land_rent_only`, charges a growing works the same.
- **New towns** are founded where work stands far from any market. With
  hands walking there is less of that, so the island has one or two towns
  fewer in 1900 and founds them later (the last around 1830 instead of 1800).
  That is the port's founding rule. Under the game's rules the people rules
  found the towns (`people.md`), and in the game as it is played now there
  are 17.8 towns in 1900, the last founded in 1847 (1814 to 1873 by seed).

## In company with the rest of the game

Everything above was measured on the port with the life cycle alone. When it
was merged into the game's rules the merchants' prices (`prices.md`), the
Engel curve (`engel.md`), the machines (`machines.md`), the harvests
(`harvest.md`) and the chronicle (`chronicle.md`) were already there, so the
whole was run together: seeds 1 to 6 and then 7 to 12, six families, to
1900, the game's rules before the life cycle and with it
(`core-steps/works-before`, `works-after`, `works-s7-12-before` and
`works-s7-12` under `.claude/night/reports/`). No run failed. Seeds 1 to 6,
the tally seed for seed (a star where every island with the life cycle
stands clear of every island without), and beside it what the life cycle did
alone, with six families on seeds 1 to 8:

| six families | the game before | with the life cycle | tally | alone (port, then works) |
|---|---|---|---|---|
| works standing 1800 / 1850 / 1900 | 1,983 / 4,313 / 6,858 | 282 / 565 / 711 | 0/6* | 2,131 / 4,156 / 6,973, then 318 / 624 / 668 |
| size in 1900: median, ninth tenth, largest | 0.53, 1.9, 3.7 | 5.5, 36, 121 | 6/0* | 0.46, 2.5, 3.7, then 6.6, 42, 116 |
| works founded, closed in a run | 12,288, 5,990 | 2,141, 1,668 | 0/6* | 11,879, 5,472, then 1,963, 1,576 |
| median life of a works that closed | 17.2 | 10.8 | 0/6* | 13.1, then 10.0 |
| utilisation in 1900, mean over works | 0.89 | 0.78 | 0/6* | 0.84, then 0.76 |
| industry cells 1800 / 1850 / 1900 | 350 / 656 / 992 | 22 / 106 / 236 | 0/6* | 359 / 601 / 981, then 18 / 69 / 132 |
| town cells 1900 | 124 | 130 | 6/0 | |
| works' share of labour, towns' share of people, 1900 | 0.493, 0.432 | 0.524, 0.453 | 6/0*, 6/0* | |
| towns in 1900, the last founded | 19.2, 1809 | 16.8, 1839 | 1/4, 5/1 | 19.8, 1802, then 17.5, 1831 |
| real wage 1800 / 1850 / 1900 (1750 = 1) | 1.42 / 1.84 / 2.45 | 1.56 / 2.51 / 2.80 | 5/1, 6/0, 6/0 | 1.31 / 1.35 / 1.35, then 1.50 / 1.35 / 1.57 |
| the same, chained | 1.56 / 2.18 / 3.29 | 1.82 / 3.49 / 4.35 | 6/0, 6/0, 6/0* | |
| output per head 1800 / 1850 / 1900 | 1.43 / 1.97 / 2.57 | 1.63 / 2.49 / 2.65 | 6/0, 6/0, 5/1 | 1.39 / 1.55 / 1.50, then 1.63 / 1.59 / 1.63 |
| the year output per head peaks | 1892 | 1877 | 0/6 | |
| railway km 1850, 1900 | 1,033, 1,039 | 925, 971 | 2/4, 2/4 | |
| converter's share of steel, 1880 | 0.98 | 1.00 | 3/0 | never built, either way |
| steel rails' share of rails, 1890 | 0.99 | 0.26 [0 .. 1] | 0/5 | none, either way |
| price swings: grain, iron, rails, cloth | 0.099, 0.126, 0.137, 0.027 | 0.106, 0.176, 0.172, 0.037 | 6/0, 6/0, 5/1, 6/0* | 0.090, 0.202, 0.200, 0.084, then 0.096, 0.238, 0.232, 0.106 |
| the real wage: its yearly swing, its worst year | 0.030, 7% down | 0.062, 17% down | 6/0*, 0/6* | |
| famine years in a run | 3.7 | 2.2 | 1/5 | |
| alarms, closures in the news | 8.2, 53 | 16.3, 201 | 6/0, 6/0* | closures 45, then 151 |
| seconds a run, six at a time | 18 | 13 | 0/6* | |

Seeds 7 to 12 say the same: the wage 1.56, 2.10 and 2.92 for 1.45, 1.78 and
2.43; output per head 1.59, 2.12 and 2.67 for 1.48, 1.93 and 2.55; 725 works
for 6,907, the median one 5.5 usual sizes; 1,538 closures for 5,906; industry
on 233 cells for 1,006; 16.8 towns for 20.2; 1,017 km of railway for 1,121;
the real wage's yearly swing 0.065 for 0.033 and its worst year 16 per cent
down for 8; steel rails 0.10 of rails in 1890 for 0.98.

All of this is the game as it stood at that merge. The game's railways,
carting, the people rules, kept town ground, the mines, the canals and the
books of account have come in since, so none of it can be read from a sweep
of today's game, and it was not measured again. Where the same fact exists
today it reads as follows: the merge's column from the table above, beside
the game as it is played now (seeds 1 to 8, six families,
`isostate-core sweep --seeds 1-8 --rules game --families 6`, 12 November
2026; the mean over seeds):

| six families | with the life cycle, at the merge | the game now |
|---|---|---|
| works standing 1800 / 1850 / 1900 | 282 / 565 / 711 | 237 / 532 / 738 |
| size in 1900: median, ninth tenth, largest | 5.5, 36, 121 | 6.0, 37, 121 |
| works founded, closed in a run | 2,141, 1,668 | 1,991, 1,475 |
| median life of a works that closed | 10.8 | 11.7 |
| utilisation in 1900, mean over works | 0.78 | 0.75 |
| industry cells 1800 / 1850 / 1900 | 22 / 106 / 236 | 18 / 68 / 199 |
| towns in 1900, the last founded | 16.8, 1839 | 17.8, 1847 |
| real wage 1800 / 1850 / 1900 (1750 = 1) | 1.56 / 2.51 / 2.80 | 1.63 / 1.95 / 2.60 |
| the same, chained | 1.82 / 3.49 / 4.35 | 1.90 / 2.88 / 4.24 |
| output per head 1800 / 1850 / 1900 | 1.63 / 2.49 / 2.65 | 1.70 / 2.12 / 2.65 |
| the year output per head peaks | 1877 | 1879 |
| railway km 1850, 1900 | 925, 971 | 1,328, 2,248 |
| converter's share of steel, 1880 | 1.00 | 0.86 |
| steel rails' share of rails, 1890 | 0.26 | 0.002 |
| price swings: grain, iron, rails, cloth | 0.106, 0.176, 0.172, 0.037 | 0.109, 0.172, 0.183, 0.039 |
| the real wage: its yearly swing, its worst year | 0.062, 17% down | 0.056, 14% down |
| famine years in a run | 2.2 | 0.4 |
| alarms, closures in the news | 16.3, 201 | 18, 191 |

The works themselves are where the merge left them: much the same count and
size, a little fewer founded and closed. The middle of the century has moved
most. The wage of 1850 reads 1.95 where it read 2.51, most likely because
carting puts off the machines' rise by about ten years (`carting.md`, "In
company"; nobody has taken today's numbers apart). The wages of 1800 and
1900 are nearer the merge's (1.63 for 1.56, 2.60 for 2.80), so what is said
below of the middle decades
should be read as of the merge, not of today.

What they say (at the merge):

- **In company the life cycle does what it does alone.** A tenth of the
  works, ten times the size, three quarters of the ground given back, three
  closures where there were ten. The island now has half its hands in works
  where the port has a quarter, so the works take 236 cells where alone they
  took 132: the same rule on twice the hands.
- **The wage is higher at every mark and on every island**, most of all in
  the middle of the century (2.51 for 1.84 in 1850). Works that can grow
  make more of what the machines are made of, sooner: in the 1840s 1,370
  units of machinery a month at 52 for 810 at 70, and 13,200 of iron at 4.8
  for 6,800 at 8.7. By decade the real wage stands an eighth above the game
  before from the 1780s to the 1820s, 28 per cent above in the 1840s and 39
  in the 1850s, and then the gap closes: 30, 22, 16 and 11 per cent in the
  four decades after. From the 1850s the wage with the life cycle hardly
  moves (2.68, 2.73, 2.82, 2.83 and 2.75 by decade, where the game before
  goes on from 1.93 to 2.47), output per head peaks fifteen years sooner,
  and in the 1890s it is a little under the game before (7.9 for 8.1 on the
  model's measure). The growth has moved into the first hundred years. The
  machines' page feared the pair would cost a fifth of the wage of 1900
  (2.05 for 2.50 in its trial); with the merchants' prices and the reviewed
  life cycle it does not.
- **Its numbers hold.** The year's room was tuned without the price rule,
  and this page asked for it to be swept again once both were in. On seeds 1
  to 8 with six families (`core-steps/works-x-room`) a room of 0.10, 0.15,
  0.20 and 0.30 gives a real wage in 1850 of 2.15, 2.48, 2.43 and 2.41 and
  in 1900 of 2.83, 2.88, 2.59 and 2.54, with closures rising all the way
  (1,330, 1,640, 1,716, 1,935): 0.15 is still the place to stand. With the
  room and the sized entry both off (`works-x-noD`) the wage of 1900 is 2.04
  and its deepest fall from an earlier peak 49 per cent on average: the room
  is what keeps the rest in order. `built_share` from 0.01 to 0.2 changes
  nothing that shows (`works-x-built`); the merchants' markup of 0.5 is still
  the best of 0.3, 0.5 and 0.8 (`works-x-markup`); and each part switched off
  alone does in company what it did alone (`works-x-walk0`, `-nospread`,
  `-entry0`, `-noextend`, `-built0`, `-hands0`): without A or B the thousands
  of small works come back, 2,100 and 3,000 of them standing in 1900.
- **With the life cycle switched off** (`--set works.enabled=false` under
  the game's rules) the merged code gives every fact of every seed as the
  code before the merge gave it (`core-steps/works-off`), two counts of news
  aside: a district's closures are now a story at five works, not ten
  (`chronicle.md`, "Beside the life cycle for works").
- **The scripted players** (`examples/bots.rs`, seeds 1, 2, 3 and 7, the
  whole game; `core-steps/works-probe/bots-game.txt`) end with 19 to 48
  million. The landlord is first in every seed, and the founder founds 120
  to 148 works and has 25 to 32 standing, as alone.

What is new in company, and not good:

- **A building cycle, which the merchants' prices now show.** The textile
  mills of a town order the whole of a year's room in one year (14 to 16 per
  cent of their capacity), are then overbuilt, order next to nothing for two
  or three years, and start again; the brickworks follow a year behind. The
  need on trend is about 7 per cent a year (people, incomes and wear), and a
  works that earns the hurdle asks for a fifth more (the port's rule in
  `firms::review`), so the room is taken whole or hardly at all. A mill is
  built of timber, and an order asks each month for a quarter of what is
  still outstanding: a wave year's orders are ten to twenty months of the
  island's ordinary timber sales, most of it wanted in the first quarter.
  Alone this was there and hardly showed (seed 2: the mills ordering 15 per
  cent in 1876-77, 1883 and 1891, timber between 2.4 and 4.3 and never more
  than a quarter dearer than the year before).
  With merchants who mark the price on their stock, timber trebles for a
  year every three or four (seed 2 at each year's end from 1884: 1.5, 3.0,
  2.0, 1.5, 1.6, 4.1, 1.7, 1.6, 1.6, 4.7), and timber is in every
  household's warmth and shelter. Over seeds 1 to 6 after 1800 the yearly
  swing of timber's price is 0.24 for 0.05, of the cost of living 0.07 for
  0.02 and of the real wage 0.07 for 0.03; the real wage's worst year is 17
  per cent down for 7; output falls by more than 4 per cent in 14 years of a
  run for 2 (`core-steps/works-probe`); and the news has twice the alarms
  and seven slumps a history keeps where it had three. The level is higher
  all the while: the lowest five years after 1800 stand at 1.61 times 1750's
  wage for 1.44.
  It is an honest cycle (lumpy building, a wood that answers a year late),
  and a game may want one, but it is now the largest swing on the island
  and nobody chose its size. Two cures were tried and neither is in the
  code. A smaller room (0.10) takes a tenth off the swing and a seventh off
  the wage of 1850. Asking for an order more slowly, a tenth or a twentieth
  of what is outstanding each month where the port asks a quarter (a trial
  build, `works-x-hack-rate10` and `-rate05`), brings the real wage's swing
  to 0.047 and 0.038 and its worst year to 12 and 10 per cent down; but a
  works then counts as building for longer, so there are 930 and 1,470 works
  in 1900 for 710, the median one 4.2 and 2.4 usual sizes for 5.5, and the
  wage of 1850 is 2.17 and 2.06 for 2.51. The cure is more likely in how
  much a works asks for when it grows (what its trade is short of, not a
  fifth at a time) than in how fast the builders are supplied. That is a
  change to the review and wants its own sweep on sixteen seeds.
- **The tools famines are milder and still there.** Tools' dearest year is
  5.8 times their median year (6.6 alone; 1.6 in the game before), with nine
  years of a run above three medians, and the real wage's deepest fall from
  an earlier five-year peak is 16 per cent on average and 22 in the worst
  seed (25 and 52 alone; 4 and 5 in the game before). The merchants' prices
  take the worst off, as this page hoped, and do not cure it.
- **After a loss of capacity the rebuilding overshoots and rings.** The
  price rule's shock (30 per cent of iron capacity lost at the end of 1815;
  `examples/prices_probe.rs --shock`, in `core-steps/works-probe`): capacity
  is back a year sooner than without the life cycle and real output loses a
  quarter as much, but capacity then stands 18 per cent over where it would
  have been (9 without), 13 to 18 per cent under eleven to thirteen years on
  and 12 over again at seventeen, with the price of iron two fifths lower
  and then two thirds higher at the turns. The price rule's worry, that the
  room would
  ration the rebuilding, did not come true. The echo is the lumpiness of the
  building cycle again.
- **The steel rail does not carry over, and the converter comes too soon.**
  Steel rails are a quarter of rails in 1890 (all of them in one seed, under
  three tenths in the rest) where they were all of them, and a tenth on
  seeds 7 to 12. Iron is half
  the price it was, the furnaces being large, so an iron rail is cheap and
  the steel mill has little to offer. The converter still makes all the
  steel by 1880, but in three seeds of six it is found between 1816 and 1830
  (1840 to 1861 before): crucible works that can grow pour steel sooner,
  and the converter's place among the inventions is counted in steel poured.
  The machines' page says the converter's numbers and the quality curve want
  tuning again under the life cycle. They do, for the dates more than for
  the wage; `machines.inherit = 0`, its safe setting, gives a wage of 2.98
  in 1900 for 2.80 on these seeds and no converter at all
  (`core-steps/works-x-inherit0`).
- **Closures in the news**: 201 a run for 53, none of them kept by the
  chronicle. The threshold in `sim::notable_closures` was the worry alone
  and is the same worry here (`chronicle.md` has the count).
- **The railway is a tenth shorter** on both sets of seeds (971 km for 1,039
  and 1,017 for 1,121 in 1900, four islands of six down each time), and
  there are two or three towns fewer, founded later, as alone.

Two things were mended at the merge, both where the life cycle meets the
chronicle, and `chronicle.md` has the account: a branch's extensions (part
E) were an item each for the player, 240 in a game, and now go into the
chronicle's one summary a year; and a district's round of closures, which
ten works shutting in a year had made a story, is one at five now that a
district has forty works and not four hundred.

## What worries me

- **The building cycle in company** (the section above): mills that order
  a year's room at once and then nothing for three years, timber trebling
  when they do, the real wage's worst year 17 per cent down where it was 7.
  It is the largest swing in the game now, and nobody chose its size.
- **A trade far too small for its market grows only at the room's pace.**
  The room is 15 per cent a year of what the town sold, so however dear the
  good, its works and any newcomer can together add about a sixth a year. It
  shows in three places. The 1750s: millwrights make half the port's
  machinery at two and a half times its price (1,440 against 570), two fifths
  less coal is dug, and coke and engine works are found three to twelve years
  later than under the port, mills three to nine years sooner. (That it is
  the room: with the trade counted at usual prices, above, coal and both
  dates come back to the port's and machinery most of the way.) New goods: steel takes until the
  1810s or 1820s to pass the port's output (150 a month in the 1780s against
  590), then nearly doubles it. And tools: households turn to steel for their
  implements, the smithies shrink from 1810 to 1850, and when farm machinery
  arrives the engine works want tools the remaining smithies cannot make. In
  a quarter to a half of the seeds tools then cost ten to twenty times the
  usual for some years (five on average, fifteen in the worst), machinery
  with them, and in the worst of them the real wage falls by two fifths to a
  half (seed 8 before the review's repairs: tools from 14 in 1857 to 306 in
  1873 while the smithies took every year's room in nearly every town;
  output per head down a quarter).
  The port has no tools famine, because any number of small smithies can
  open at once; it has a longer machinery famine instead. Three cures were
  tried and none kept (above).
- **Prices swing harder, most of all with families**: cloth a quarter, iron
  and rails 16 to 39 per cent with six families. Larger works order larger
  lumps. The year's room holds it there; the price rule is the cure.
- **The converter is not built**, with the life cycle alone on the port's
  content. (The game's content makes the converter cheaper, `machines.md`,
  and under the game's rules it is built: in the game as it is played now it
  is built on every seed of eight, between 1825 and 1888, and makes 0.86 of
  the steel in 1880, the mean of the eight.) Under the port with the life
  cycle it is built on one seed of thirty-two (the port alone builds it in 10
  of 32, late in the century in all but one, and it never makes half the
  steel). The cause is the price of steel. Crucible works that can grow to
  twenty usual sizes make steel at 17 to 22 from the 1830s where the port's
  small ones need 28 to 39, and at that price the converter loses money at
  any size: on seed 7 its best
  site promises between minus 5 and minus 20 per cent a year from 1850 to
  1900 against a hurdle of 12, where under the port steel at 52 gives it 12.5
  in 1895. The converter's numbers (half productivity to begin with, 150
  bricks and 2 of machinery a unit) were only ever just good enough against
  dear steel. That is a matter for the game's content, not for these rules.
- **A player who founds a works every year loses most of them.** The founder
  bot founds 126 to 162 and has 26 to 39 standing, and ends below the passive
  player in three seeds of four (in the port the two are level). Founding
  beside your own works is now a poor move, and extending is what the branch
  does for you. The player has no "extend this works" button of their own;
  that is a change to the player's choices and was left alone.
- **A family's works are not rationed by the year's room.** The room is taken
  by the review and by entry; a family's founding and a branch's extension
  count against it afterwards but are never refused, so a town's trade can
  order more than its room in a year by what its families add. It is why
  counting a dear good's trade larger did harm with families.
- **Closures are in the news three times as often** (131 to 153 items a run
  against 46). `sim::notable_closures` calls a works of over one and a half
  usual sizes notable, which was a large works under the port and is an
  ordinary one here. The threshold wants setting by whoever shapes the news
  (in company: 201 a run for 53, none of them kept by the chronicle).
- **Utilisation is lower** (the mean over works 0.76 to 0.78 against 0.81 to
  0.86 without families, 0.71 to 0.76 against 0.84 to 0.87 with): the small
  fringe stands half idle before it closes.
- **Sister works are still about a quarter of all works** (186 of 699), most
  of them crucible steel in towns whose cells are full. A town that keeps its
  ground and grows should thin them.
- **The viewer has been looked at with these rules for the works themselves,
  not yet for the family's list.** A works can now be a hundred usual sizes
  and a cell can hold several large works. The standard pictures play a game
  under these rules and take the cell with the most works standing in 1900
  from play distance and close up; the works' generators draw a works at any
  size from 0.4 to a hundred usual sizes (the lab's works sheets show 0.4, 1,
  3, 10, 30 and 100), and since the works' detail landed every works is made
  from the houses' kit at every level of detail, so the tiers and plots this
  worry once named are gone. What nobody has looked at on screen is the
  family's Build list under the life cycle: it offers the first five works
  the family may build there, deep markets and the best returns first, and a
  works refused because "the cell has no
  room left for ..." (`works::room_on_cell`) is left out of it without a word
  (the viewer's `game.rs`).
- **Numbers not swept**: `hands_per_cell` (the probe found 1,000 gives about
  250 industry cells and 1,050 works, 4,000 gives 55 and 510), `cell_limit`,
  `room_share`, `entry_share` and `walk` (0.01 leaves a third of hands more
  than two cells out; 0.05 pulls everything into town) are the probe's values,
  checked only as a set. The review ran each to an extreme, sixteen
  combinations at a time with six families (`sweep-works-review-x1`, `-x2`):
  no run failed or lost money, and the island degrades the way one would
  expect (200 hands to a cell and a tenth of a cell as the limit gives the
  port's thousands of small works and a real wage below 1750's).
- **A run differs from its neighbour by a lot.** Two builds that differ in
  one small rule give, seed for seed, real wages 0.2 apart in a given year.
  Eight seeds settle the works' count and size; for wages, swings and hard
  spells sixteen are barely enough.

## Measuring it

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --out DIR/base
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules port --set works.enabled=true --out DIR/works --compare DIR/base
cargo run --release -p isostate-core --example works_census -- --decades --families 6 --set '{"works":{"enabled":true}}' 3 7 11 21
cargo run --release -p isostate-core --example bots -- --rules game
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules game --families 6 --out DIR/game
cargo run --release -p isostate-core --example works_mends -- --seeds 1-16 --list 3
cargo run --release -p isostate-core --example works_mends -- --seeds 1-16 --set '{"works":{"spread_first":false,"keep_order":false}}'
cargo run --profile work -p isostate-core --example works_ground -- --seeds 1-8
cargo run --profile work -p isostate-core --example works_ground -- --seeds 1-8 --set '{"works":{"by_method":false}}'
```

`works_ground` is the measure of part I: for each year asked (`--years`,
1850 to 2050 by fifty), summed over the islands, the ground works hold by kind
and by method in the books and on the squares, their hands at work and their
plant's hands, square metres a hand of each, the houses' squares, rent over
sales and over profit before rent, what landowners take, what a town cell's
land earns, works under one square, the largest works and what they share of
their town's hands and of their good; then the squares' log and its flicker
(a change that gives a square back to a tenant it had in the three years
before), the first year each method stood, and one town cell printed square by
square (`--cell C`).

`works_mends` is the measure of parts G and H: the exact twins by where they
came from and by the decade they opened, the works of 1749 that share a cell
with one of their trade in their town, the orders of more capacity for
standing works and those whose works closed within three years, and the
families' worth at the end. Under the game's rules on sixteen islands
(13 November 2026) expect no works of 1749 sharing a cell, about 1,350
twins, and about 110 orders worth about 0.15 million closing within three
years (one history only); with both switches off, about 1,250 works of 1749 sharing, 2,400 to
2,450 twins and 200 to 215 such orders worth 4.5 to 6.4 million (the second
line, and the null pair).

The census prints, for each method, the works standing, their sizes, their
hands and where they stand, and decade by decade the works founded, the
sister works, the closures, output per head and the real wage. It runs the
port with no families unless told otherwise, so `--families 6` is what gives
the census of six families above. For the port with the life cycle (the
second line) expect 540 to 770 works in 1900, a median size of 5 to 9, 110 to
145 industry cells, a real wage 1.3 to 1.8 times 1750's from 1850 on (a seed
in a tools famine falls below that for a few years) and under 2,300 works
founded in a run. For the game as it is played (the last line; 12 November
2026) expect 680 to 800 works in 1900, a median size of 5.6 to 6.3, 180 to
240 industry cells and 1,650 to 2,250 works founded in a run (the least and
greatest of the eight seeds, rounded outward).

## After 1900: work gathers where work is

Under the game's rules, from the yearly step of 1901, every works in a town
of U townsfolk is `(U / 20,000)^0.07` as productive, ramped in over ten
years: a doubling of a town is worth 5 per cent on every works in it, and the
reckoning of a site reads the same factor. The switch is `city.gather`, on
under the game's rules, with `city.power`, `city.size` and `city.ramp_years`;
the rule and what it does to the island are in [city.md](city.md).
