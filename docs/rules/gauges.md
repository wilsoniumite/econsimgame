# Gauges

Gauges read off the books: what share of the island's income and of its
spending goes to work, what the plant worn out does to that share, a floor
under it, the labour inside a machine, what a wage buys of made goods and of
the land's, and who owns the works and the land. Not in the Python. The
switch is `gauges.enabled`, which needs `accounts.enabled`; the code is
`gauges.rs`, one marked call each in `sim.rs` (to make ready and at the
year's end), `observer.rs`, `facts.rs`, `record.rs` and `live.rs`, and one
word in `accounts::own_series`.

## What they are for

The books (`accounts.md`) say who was paid what in a year. They cannot say
what share of the price of a loaf, a plough or a yard of cloth ends up as
somebody's wages, because each of those was made from other goods, which
were made from others again. Nor do they say what the owners of works keep
once the plant they wore out is paid for. The run to 2050 is to be judged by
a handful of such numbers, and every later stage of it (who is paid for
what they put up, machines that take tasks, the ground under the towns, a
state) is read on them. So they come first, on the island as it is, before
anything they are meant to watch has been built.

They are an observer of an observer. Nothing in the economy reads
`State::gauges`, no random number is drawn, and a run with the gauges on is
the same run, number for number, as one with them off (`tests/gauges.rs`,
`runs_identical_with_the_gauges_on_and_off`, and the sweeps below). Under the
port they are off, nothing is kept, and `params.json` does not mention them.
With the books off they do nothing either, whatever their own switch says.

## How they are reckoned

Once a year, as the books close (`sim::end_year`, right after
`accounts::close_year`), from that year's books alone.

### The cost system

For each good, island-wide, what a year's worth of it was worth and what
went into it:

- **On the land**, from the books' land table: the merchants pay `value *
  (1 - merchant_spread)` for the produce, and that is wages, carting,
  machinery and rails, and rent. So of the value: wages and carting are
  work; machinery and rails are goods (split between the two by what the
  town's land bought of each); rent is rent; and the merchants' spread is
  "the rest".
- **In works**, from the methods table: the value is the merchants' spread,
  carting out, materials and fuel, carting in, wages, site rent, hired
  managers and profit. Carting, wages and managers are work. Materials and
  fuel are split good by good in proportion to what each cost the method in
  that town (the books' `uses`). And the plant worn out is taken out of the
  profit and counted as goods used up: the method's mean capacity that year,
  times `depreciation` (0.04 a year, the rate at which the economy itself
  wears capacity down), times what a unit of capacity costs in its town at
  the year's prices (the method's `capital` bundle, as `families::works_value`
  prices it). What is left of the profit, with the spread, is "the rest":
  the owners' part after wear, and the merchants'.
- **Carting between towns** is counted as work in the good carried, and adds
  to its value.

Every good's shares (wages, rent, the rest and each input good) then add up
to one, and the books' single precision is the only thing that stops them
doing so exactly (a test holds them to a hundred-thousandth).

### The whole chain

Following each good back through the goods it was made from is one small
solve a year: with `a` the matrix of input shares, the wages inside a unit
of each good's price are `(I - a)^-1` times the vector of direct wage shares,
and the same for rent and for the rest. Of each good's price, those three
add up to one; that is the solve's own check, kept as `chain_error` and as
the fact `gauges_chain_worst`.

A good worth less than a hundred-thousandth of everything made that year
(`MADE_AT_LEAST`) is taken as not made and is not followed back. Charcoal in
its last years is such a good: a few works stand at work making almost
nothing, their whole wear falls on a value of almost nothing, and on island 7
in 1897 to 1900 that one row put the solve's check out by up to 5.7 before
the rule was there.

### The gauges

| gauge | series | how it is reckoned |
|---|---|---|
| Work's share of income, gross | `acct_share_work` (the books') | the wage bill over income |
| Work's share of income, net of wear | `gauge_work_net` | the wage bill over income less the plant worn out |
| Investment's share of spending | `gauge_invest_share` | works' purchases of plant and the outlay of railways and canals, over spending |
| Spending paid out as wages | `gauge_spend_work` | everything bought for use (by households, landlords and families, by works being built, by railways and canals, and house rent), each good weighted by the wages inside it through its whole chain; the navvies' pay counts whole, house rent as rent |
| The same, households' basket | `gauge_basket_work` | what households bought, with their house rent |
| Spending paid for out of pay | `gauge_spend_by_pay` | households' spending (goods and house rent) times the share of their income that was pay, over all spending; landlords and families are paid no wages |
| The floor under work's share | `gauge_floor` | below |
| Works in years of income | `gauge_plant_years` | every works at its book value over the year's income |
| Labour inside machinery | `gauge_labour_in_machinery` | the wages inside a unit of machinery's price; rent's part is kept beside it (`land_in_machinery`) |
| Labour inside new plant | `gauge_labour_in_plant` | the same for everything works being built bought that year, as one bundle |
| What a wage buys of made goods, of the land's goods, of both | `gauge_fork_made`, `_raw`, `_all` | the island's mean wage (weighted by people) over chained Fisher price indexes of what households bought, each link over the goods bought in both years; the first year is one. A good is "of the land" when a land sector makes it |
| Hours worked over hours by hand | `gauge_hand_hours` | worker-months in works over what the same output would have taken by each good's hand method (below), at its usual size and before learning |
| New work | `gauge_new_work` | hands in methods found in the last forty years over all hands (on the land, in works, carting, digging canals) |
| Services made and lost | `gauge_lost_services` | services spoiled in store over services made |
| The families' share of the works and of the land | `gauge_families_works`, `_land` | the works they own at book value over all works; their rents times `years_purchase` over all land, as the books value it |

Each `gauge_` series holds the year's figure in all twelve of its months,
written when the year's books close and NaN until then, as the books' own
do. `accounts::own_series` counts them among the books', so a saved game's
fingerprint leaves them out (a game saved before the gauges is the same
game) and the game's golden test hashes them with the books.

Each year's `GaugeYear` keeps more than the series: the plant worn out in
money, rent's and the rest's shares of spending, `presence_today` (below),
the price indexes and the wage index themselves, each family's share of the
works and of the land, a `GoodCost` for every good (its value, its direct
shares, its inputs good by good, its wear, its chain's three shares, what
of it was lost, its hours over its hand method's), and for every town two
gauges of its own: work's share of the income earned in the town (pay, over
pay and its works' profit, its merchants' takings and its rents) and the
share of what was bought there that comes to rest as wages through the
island's chain. A town's share can stand outside nothing and one in a year
its land's rent or a works' profit is below nothing.

### The floor under work's share

What would be left to hands if every task a machine could do cost nothing.
The classes of task (hauling, tending, clerking, judging, and work only a
person may do) are not built yet; until they are, the floor needs only the
last of them, and takes it from one small table (`PRESENCE`): the shops and
trades (`services`) hold a tenth of their hands' work in it, and nothing
else on today's island holds any. The land's hands, the carters, the
navvies and the managers hold none.

The reckoning: every other hand's work is made to cost nothing. Each good's
price falls by all of that work inside it, through its whole chain; what it
pays in rent and to its owners a unit is held in money, and so is its
recipe. Spending keeps its money shares good by good (the Engel rule's way:
a household spends a share of its budget on each need, not a quantity), so
it buys more of what grew cheaper. The floor is the share of that spending
that is still the wages of the work that stayed. `presence_today` is the
same work's share of spending at today's prices, before anything falls; the
floor is always above it.

How far it can be trusted. It is a reckoning on one year's recipes and
prices, not a run, and it holds three things still that would move: rent
and owners' income a unit (in the engine the land pays its hands a fixed
share of what it nets, so a land good's rent would not stay put), what
households buy, and the wage. It says how much of today's island is work
that only a person may do, through every chain, once the rest is free. On
today's island, with only the shops' tenth closed to machines, that is a
small number, and it grows as the shops grow. The care and teaching that
would hold most of such work do not exist yet.

### The hand method

The method of the first year that makes a good with the most work a unit:
the bloomery for iron, the hand loom for cloth, the millwright for
machinery, the smithy for tools, charcoal burning, the brickworks, the
forge for rails, the services of 1750. Steel has none (no method of 1750
makes it), nor have the land's goods. A small works takes more hands a unit
than its recipe at its usual size (`econ::firm_coefficients`), so in the
first years, when every works is small, hours worked stand a third above
hours by hand.

## Where to read them

- **In a sweep**, under "Work and wages": every gauge at the fifty-year marks
  (`gauge_floor_1900`), labour inside machinery averaged over 1775 to 1900
  (`gauge_labour_in_machinery_1775_1900`), and `gauges_chain_worst`.
  `annual.csv` carries every gauge in every year.
- **For one island**, year by year, and one year's cost system good by good:

  ```bash
  cargo run --release -p isostate-core --example gauges -- --seed 7 --year 1900 --every 10
  ```

  `--end-year 2050` runs it on, `--every 1` prints every year, `--csv FILE`
  writes every year to a file.
- **In a live game**, `LiveYear::gauges` is the year's `GaugeYear` (None
  unless they are kept), and the series' figures come with the books' in
  `LiveYear::year_series`. `gauges::gauge_year(&st)` gives the last complete
  year's.
- **In the run file**, the header's `gauges` holds every year's, with the
  table of work only a person may do it was reckoned with.

Nothing draws them yet. The three dials and the ledger that would show them
to a player are a later piece of work; what they need (the share of
spending paid out as wages, ownership in years of wages, which the books
keep, and each family's share of the works and the land, each year and for
any town) is in `GaugeYear`, the books and the series.

## The numbers

`GaugesParams`, in `gauges.rs`: one switch, `enabled`, false by default and
true under `game()`. The constants beside it: `PRESENCE` (services 0.10),
`MADE_AT_LEAST` (1e-5), `NEW_WORK_YEARS` (40), `MACHINERY_SPAN` (1775 to
1900).

## What a sweep showed

**They change nothing.** Seeds 1 to 8 under the game's rules with six
families, set beside the same sweep of `main` before the gauges: "of the 624
facts both have, the seconds aside, 0 differ". The game's golden test moves
`books` and `run.bin` (the gauges' series and their years in the header)
and holds `economy` and `news`; with the gauges switched off in
`game_params()` it holds all four.

**On the island as it is**, seeds 1 to 8, six families, as run on 19
November 2026 (the mean across seeds, then the least and greatest):

```bash
cargo run --release -p isostate-core -- sweep --seeds 1-8 --rules game --families 6 --out runs/gauges
```

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| work's share of income, gross (the books') | 0.710 | 0.690 | 0.652 | 0.691 [0.647 .. 0.715] |
| net of wear | 0.713 | 0.699 | 0.670 | 0.712 [0.674 .. 0.734] |
| investment's share of spending | 0.005 | 0.036 | 0.064 | 0.043 [0.014 .. 0.073] |
| spending paid out as wages | 0.678 | 0.689 | 0.671 | 0.686 [0.653 .. 0.712] |
| households' basket | 0.661 | 0.681 | 0.664 | 0.677 [0.652 .. 0.700] |
| spending paid for out of pay | 0.712 | 0.684 | 0.642 | 0.686 [0.642 .. 0.711] |
| the floor | 0.033 | 0.037 | 0.053 | 0.097 [0.094 .. 0.100] |
| works in years of income | 0.12 | 0.36 | 0.74 | 0.75 [0.53 .. 1.02] |
| labour inside new plant | 0.77 | 0.75 | 0.67 | 0.69 [0.52 .. 0.80] |
| what a wage buys of made goods (1750 = 1) | 1 | 1.88 | 3.10 | 4.03 [3.58 .. 4.32] |
| of the land's goods | 1 | 2.08 | 3.08 | 5.51 [4.87 .. 6.22] |
| of both | 1 | 2.08 | 3.05 | 4.60 [4.31 .. 4.93] |
| hours worked over hours by hand | 1.35 | 0.77 | 0.35 | 0.35 [0.34 .. 0.36] |
| services made and lost | 0.21 | 0.17 | 0.16 | 0.17 [0.14 .. 0.19] |
| the families' share of the works | 0.07 | 0.44 | 0.43 | 0.52 [0.43 .. 0.61] |
| and of the land | 0.13 | 0.19 | 0.24 | 0.24 [0.16 .. 0.29] |

What I would say from them:

- **Wear moves work's share by two points at most** before 1900, since the
  works are less than a year of income. Net and gross part as the works
  grow: by under half a point in 1750, two points in 1900.
- **Spending and income tell the same story.** The share of spending paid
  out as wages, reckoned through every chain, stands within about three
  points of the books' share of income at the marks (a test holds six in
  every year of a run to 1800),
  and the share of spending paid for out of pay within one.
- **Labour inside machinery** averages 0.635 over 1775 to 1900 [0.560 ..
  0.673] (`gauge_labour_in_machinery_1775_1900`), inside the band of 0.4 to
  0.75 the design for the run to 2050 reckoned for a century of strong and
  stupid machines, on all eight islands. A single year's figure swings from
  0.2 to 0.9 with the engine works' profit; in 1750 it is 0.06, because the
  first millwrights' price stands far above their costs and most of it is
  profit.
- **The floor is small**: a twentieth of income in 1850 and a tenth in
  1900, all of it the shops' tenth, through the bread and cloth and iron
  in everything a shop sells. On today's island almost no work is closed to
  machines.
- **A wage buys the land's goods better than made ones** after 1850: on
  the island the machines' gains reach the farms, quarries and pits (the
  machines mechanism) more than the works.
- **Hours worked are a third of hours by hand** from 1850 and stay there:
  the methods found in the first century do their work with a third of the
  hands, and nothing after the converter does better.
- **Services lose a sixth of what is made**, a fifth in the first years,
  as the merchants' rule for goods that spoil was expected to show.

**Run on to 2050**, island 7 alone (`--seeds 7 --end-year 2050`; the island
is still the hungry one, with neither stage 1 nor 2 of the run to 2050 in):

| | 1900 | 1950 | 2000 | 2050 |
|---|---|---|---|---|
| work's share of income, gross | 0.715 | 0.715 | 0.655 | 0.554 |
| net of wear | 0.734 | 0.738 | 0.689 | 0.598 |
| investment's share of spending | 0.031 | 0.022 | 0.065 | 0.082 |
| spending paid out as wages | 0.712 | 0.706 | 0.641 | 0.593 |
| the floor | 0.100 | 0.103 | 0.069 | 0.062 |
| works in years of income | 0.64 | 0.76 | 1.35 | 2.22 |
| labour inside machinery | 0.90 | 0.92 | 0.66 | 0.56 |
| what a wage buys of everything (1750 = 1) | 4.93 | 4.42 | 3.49 | 2.49 |
| the families' share of the works | 0.48 | 0.59 | 0.62 | 0.62 |

Work's share falls sixteen points after 1950, and the works' owners (0.07
to 0.19 of income) and the merchants take it, with no machine taking any
task: the island invests more and grows poorer, and its works stand at two
years of income by 2050. Measured work's share moves with investment, which
is why the net share and investment's share stand beside it.

## What worries me

- **The cost system is the island's, not each town's.** A town's own chain
  would need the chains of what it brought in from other towns.
- **Machinery's chain swings.** In a year the engine works earn well, most
  of machinery's price is profit, and the labour inside it falls; in a year
  they do not, it rises toward nine tenths. One year's figure says little; the
  fact over 1775 to 1900 is the one to read.
- **The floor stands on one table of one number.** Until the classes of task
  are in the content it is a reckoning of the shops' tenth.
- **Wear is reckoned, not paid.** The economy wears capacity down by
  `depreciation` without any money passing; the gauges price that wear at
  the year's prices of the plant's goods. Works' book value is reckoned the
  same way, so the two agree.
- **Spending and income differ by a point or two.** Spending paid out as
  wages and work's share of income are two views of one year. They differ
  by what went into the merchants' stock, what spoiled, and the merchants'
  trade margins, which the chain counts with "the rest". A test holds them
  within six points every year of a run to 1800.
