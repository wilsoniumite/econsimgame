# Goods that begin later: a new good that leaves the played century alone

Not in the Python, and not a mechanism with a switch: a rule of the content, and of how a sum over the
goods is taken, that holds under both sets of rules. Since 26 November 2026 a good may say when it
begins, a year or the idea it comes with, and a good that says so moves nothing of a run while
nothing of it is made, bought or held. It is what the car and care wait on: the next goods are goods
after 1900, and until now any new good rewrote the century every golden hash, standing fact and
calibration stands on. The code is `content.rs` (`Begins`, `G_base`, `sum_goods`), `util.rs`
(`pairwise_sum_from`) and the sums named below; the proof is `tests/new_good.rs`.

With today's content nothing moves: both golden files hold untouched.

## What was wrong

The lorry's builder found that a good nothing makes and nobody wants (the motor vehicle, before it
became an idea) moved island 2 from 1764, and found it by elimination. Traced here, on islands 1 to 8
under the game's rules with six families, to 1900 on main (78f01c7), with a scratch copy of
`data/game.toml` given by `tree.file`:

| what was added | facts of 1,231 that moved | where it first parted |
|---|---|---|
| one idle good at its anchor price | 565 | the books in 1847 to 1890 (island 2 in 1849) |
| the same good opening at its ceiling | 898 | the books at the end of 1763 (islands 2 and 8), the economy a year later |
| three idle goods | 1,014 | every island in 1750, 172 columns of the series |

The months are told apart bit for bit by `probe` in `tests/new_good.rs`: two islands stepped
together and their state compared each month, goods matched by name. It said "parted at the end of
1763: f_order_K[605], f_cash[605], f_order[605, stone], family_cash[3]": a family had founded another
works.

## The leaks

1. **The families' typical good** (`families::market_depth`). A family judges how scarce a good is by
   its price over its base price against the same ratio for the town's typical good, the median over
   the goods. A new good is in that list, at its anchor (a ratio of 1) or at its ceiling if it opens
   dear, and moves the median's place; from the first year a family judges a market otherwise and
   founds another works, and by 1765 the island is another island. Mended: the median is taken over
   the goods every run has from the start.
2. **Every sum over the goods** (32 places: in `econ.rs` the works' inputs, the households' and
   landlords' spending, the plant on order, the carting between towns and its weight; in `firms.rs`,
   `families.rs`, `works.rs`, `rail.rs` and `ways.rs` the cost of a works' or a line's plant; the
   observer's cost of living, the linked measures, and the Fisher index under the chained cost of
   living, which in `longrun.rs` carries the bounds on every price). They are taken in numpy's
   pairwise order (`pairwise_sum_by`), kept so that the port holds byte for byte. That order keeps
   eight partial sums while there are whole rows of eight terms and adds the rest one by one, so the
   place of a term decides which others it is rounded with. The game has 29 goods, three rows and five
   more; with 32 there are four rows, and the same terms are rounded in other groups, though the three
   added are nothing. One or two goods more leave the rows as they are and move nothing by this road;
   the third moves 1750. Mended: `Content::sum_goods` takes the goods of the start in numpy's order,
   as before, and each later good's term after them, one at a time. With no later good it is the old
   sum to the last bit.
3. **The works' share of hands** (`observer.rs`, `ind_share`), a sum over the methods in the same
   order: six methods more (50 to 56) moved the industry and mine shares in their last digit, though
   not the run. Mended in the same way (`Content::sum_methods_of`): the methods of a later good go
   after the others, as their good does.

Looked for and not found: a draw for each good (`tech::discovery` draws only for an idea with
thresholds, and not before its `found_after`), a share spread evenly over the goods, an order that
sorts the goods. A sum taken one term at a time (`.iter().sum()`, the books, the gauges) is moved by
nothing added after it.

## How to add a good after 1900

A good is listed in `data/game.toml` with `begins`, a year or the name of the method or invention it
comes with, and a method that makes it:

```toml
  { name = "motor_vehicles", label = "Motor vehicles", weight = 1.0, price = 40.0, color = "#7a3b2e", decay = 0.003, stock0 = 0.0, opens_dear = true, begins = "motor_works", stage = 6 },

[[method]]
name = "motor_works"
output = "motor_vehicles"
found_after = 1905
# ... inputs, labor, capital, K0, discover, as for any method of the tree
```

That is all. A good with `begins` goes after every good without it, whatever the file's order, and its
methods after the other methods; it is counted out of `G_base`, so nothing sums it among the goods of
the start. Care, which no method of the start makes, is the same with `begins = 1901` or the idea that
brings it. What the good does once it has begun is the builder's to show, as for any rule: its
households' props, its uses, its sweep and its page.

`begins` says when the good is there to be made and listed, not when it may be made: that is the
method's `found_after` and thresholds, as before. A good with no `begins` is a good of the start, and
a run with it is another run from 1750: leave it off only to rewrite the century on purpose.

`Content::good_begun(g, year, tech_year)` says whether a good has begun; the record writes `begins`
into its header for a later good (nothing for the others, so `run.bin` of today's content is as it
was).

## The game's golden hashes

The first era (`1750-1900`) holds when a good of a later year or idea is added: its `out_` column is
nought to 1900 and left out already, and its `price_` column, which is its anchor or its ceiling and a
price nothing is sold at, is left out while the good has not begun by the run's last year
(`blind_series` in `tests/golden_game.rs`, since this page). A good that begins in a year or with an
idea found before 1900 keeps its columns, as it should. So a builder who adds a real good after 1900
takes the hashes again for the second era alone, and says so: "1901-2050 economy, news and run.bin:
motor vehicles, a good of 1905". If the first era moves, the good acted before its time, and that is
a fault to find (the probe finds it), not a hash to take.

## The viewer

The Markets panel lists a good from the month it begins, and the map's price and made views step over
a good that has not begun by the year on the clock (`replay::begun`). A good of a year begins in its
January; one that comes with an idea begins, for the viewer, when any of it is first made, since the
record keeps the ideas' names and not the years they were found. The Making panel shows what a town
makes, which a good nobody makes is not part of.

## Measured

All with `--profile work`, on the shared machine.

- Main's content, the mended code against main's: islands 1 to 8 to 1900 under the game's rules,
  0 of 12,208 facts differ, seed for seed; the port's golden test and the game's hold.
- The whole proof (`cargo test --profile work -p isostate-core --test new_good -- --ignored
  --nocapture --test-threads=1 --skip probe`, 493 seconds on the clock), every fact the sweep takes
  and every yearly mean of every series, bit for bit, the test goods' own aside:
  - eight test goods from 1750 (one opening at its ceiling, one like electricity, not carried and not
    kept), nothing making them, islands 1 to 8 to 2050 under the game's rules: 755,256 numbers,
    every one equal; under the port's: 231,080, every one equal;
  - a test good that comes with the idea of its one method, found from 1950, islands 1 to 8 to 1949
    under the game's rules: 496,336 numbers, every one equal.
- Six methods of a later good (50 methods to 56), islands 1 to 8 to 1949: 0 of 21,136 facts differ
  (33 differed in their last digit, the industry and mine shares, before the methods' sum was
  mended).
- A good of 1950 put in `data/game.toml` for one run of the game's golden test (and not kept): "in
  1750-1900, nothing; in 1901-2050, the economy's monthly series and the run file", the books and the
  news of the second era holding too.
- Speed: a sum over the goods of the start is the sum it was; nothing is slower (eight islands to
  2050 took 91 and 89 seconds with and without the three test goods, beside other work).

## What worries me

- **A good without `begins` still moves the century**, by both leaks. That is meant (a good of the
  start is part of 1750), but a builder who forgets the key gets a run that parts in 1764 and a first
  era that moves; the golden test says so.
- **The typical good never counts a later good**, even once it is made in every town. It is a
  measure of what is dear among the goods of the island the families know; a car will not weigh in
  it. Change it when a good after 1900 is meant to be among them, and take the hashes then.
- **A new method of a good of the start** is a method of the start: added after today's, it can move
  the works' share in its last digit as the methods pass a row of eight (a watched series, not the
  run). Give such a method's good `begins`, or leave it.
- **A sum that is negative nought.** A later term of nothing added to a sum of exactly -0.0 gives
  +0.0. No sum over the goods is ever that; noted for completeness.
