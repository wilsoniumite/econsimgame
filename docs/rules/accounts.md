# Accounts

Books that say who makes, moves, buys and earns what. Not in the Python. The
switch is `accounts.enabled`; the code is `accounts.rs`, five marked calls in
`econ::step_month` (and two lines there reshaped, so that the hired managers'
cuts and the mines' list of who is paid each royalty live long enough to be
read), a few marked lines each in `sim.rs`, `observer.rs`, `land.rs`,
`facts.rs`, `record.rs` and `live.rs`, and one small function in `mines.rs`
that says how many sinkers a working has.

## What they are for

Every month `econ::step_month` works out what each farm, pit and works made,
what every buyer got, what went from town to town and who was paid for all of
it, and then keeps a few totals for the observer and throws the rest away. So
the island could not answer the questions a person watching it asks first:
where does this town's iron go, who buys it, what does it cost to cart, does
this works make money and for whom.

With the switch on the rest is kept: a year's books for every town and good,
for every works, and an income account for the island. Two things want them.
The viewer can draw a town's trades from them, as a chart of what flows where
at what price. And the labour work this island is meant to carry (DEVLOG, "on
to 2050") watches three gauges that the island could not report until now:
work's share of income, what ownership costs in years of wages, and who owns
what.

The books are an observer. Nothing in the economy reads `State::acct`, no
random number is drawn, and a run with the books on is the same run, number
for number, as one with them off (a test holds four runs to that, bit for bit;
the sweeps below hold forty-eight more to it by their facts). Under the port
they are off, nothing is kept, and `params.json` does not mention them.

## What is kept

Money and quantities are the year's totals (the twelve months added up) unless
a column says otherwise. Hands, capacity and the count of works are means over
the twelve months. A year's books close at the end of the yearly step, after
that year's openings, closures and purchases, so "at the year's end" means as
the next January begins.

Everything is kept for every complete year in single precision
(`Books::hist`, flat tables with the years one after another, as the run
file has them but with a row for every one of the `T_max` town slots;
`Books::year(k)` gives one year's as a `YearRec` of slices).
The year now running is added up in double precision and is not visible
until it closes.

### A town's goods: `GOODS_FIELDS`, `[town][good][22]`

| column | |
|---|---|
| `made` | by the town's land and works, timber from clearing included |
| `cleared` | of that, timber from woodland cleared at the year's end |
| `households` | bought by households |
| `landlords` | bought by the town's landlords for themselves |
| `families` | bought by families seated here (they buy through the landlords; their part is their part of the budget) |
| `works_inputs` | used up by works as materials |
| `works_fuel` | burnt by works |
| `land` | bought by farms, pits and mines: machinery and rails |
| `building` | bought by works being built or enlarged |
| `projects` | bought by railways and canals under construction (anything that builds through `State::projects`) |
| `sent` | carried to other towns |
| `received` | arrived from other towns |
| `spoiled` | lost in store (`decay`) |
| `moved` | stock that came (+) or went (-) with a district that changed market at the year's end |
| `oversold` | sold though it was not in store (see "What does not balance") |
| `stock_open` | in the merchants' hands as the year began |
| `stock` | and as it ended |
| `asked` | everything buyers asked for; `asked` less the eight "bought" columns is the unmet requests |
| `price` | mean of the twelve month-end prices |
| `price_low`, `price_high` | the least and greatest of them |
| `anchor` | mean of the merchants' anchor (`prices`); 0 when that mechanism is off |

For every town and good:

    made + received + moved + oversold
        = households + landlords + families + works_inputs + works_fuel
          + land + building + projects + sent + spoiled + (stock - stock_open)

### A town's land: `LAND_FIELDS`, `[town][sector][7]`

`out`, `value` (at the town's price), `hands`, `wages`, `carting` (the
produce's way to market), `inputs` (machinery and rails bought), `rent`. The
merchants pay `value * (1 - merchant_spread)` for the produce, and that is
`wages + carting + inputs + rent`: rent is whatever is left, as it is in the
economy, and in a bad month it is below nothing. Under `mines` a quarry's, a
pit's or a mine's rent is its workings' royalty, and the sinkers of a working
being opened are among its `hands` and their pay among its `wages` (which is
why a new working's royalty starts below nothing).

### A town's methods: `METHOD_FIELDS`, `[town][method][14]`

`out`, `value`, `works` (works at work), `capacity` (a month), `hands`,
`wages`, `inputs` (materials, at what the merchants were paid for them:
the town's price, or less for a works that stands nearer the source than the
town does), `fuel` (the same), `cart_in` (carting of both out to the works), `cart_out` (carting of
the output in to the market), `rent` (sites), `managers` (hired managers of
family works with no branch over them), `profit` (after the managers),
`dividends`. The merchants pay `value * (1 - merchant_spread)`; `cart_out` of
it goes to the carters and the rest is the works' revenue, which is `inputs +
fuel + cart_in + wages + rent + managers + profit`.

Beside it, sparse, what each method in each town used of each good:
`(town, method, good)` with `used` (as material), `burnt` (as fuel) and `cost`
(both, delivered to the works). These are the edges of a flow chart: ore and
coal into the furnaces, iron out of them and into the forges.

### A town's own figures: `TOWN_FIELDS`, `[town][33]`

| columns | |
|---|---|
| `pop` | people at the year's end |
| `wage`, `supply`, `carters`, `navvies`, `hands_land`, `hands_works` | means: the wage; hands for hire, with the carters, the canals' navvies and the sinkers of new workings already taken out; hands carting (0 unless `carting` is on); hands digging canals (0 unless `canals` is on); hands on the land, the sinkers among them; hands in works |
| `pay_works`, `pay_land`, `pay_projects`, `cart_works`, `cart_land`, `cart_trade`, `managers`, `div_works`, `div_merchants`, `fund`, `issue` | what the town's households received, by kind; the eleven add up to all they received. `pay_projects` is the navvies' pay, on railways and canals |
| `div_families` | dividends of family works here |
| `rent_land`, `rent_sites`, `rent_town` | rents paid in the town's district: farms, woods, pits and mines; works' sites; town houses |
| `rent_families` | of those three, what went to families |
| `spend_households`, `spend_landlords`, `spend_families` | spending on goods |
| `invest`, `projects` | works' purchases of capital goods; railways' and canals' outlay (materials and navvies) |
| `merchants_net` | the merchants' takings less what they paid out, before their dividend |
| `stock_value` | their stock at the year's end, at that month's prices |
| `cash_households`, `cash_landlords`, `cash_merchants` | at the year's end |

And `[town][need][2]`: what households, and what landlords and families,
spent on each need (the goods bought for it, at the month's prices).

### Trade: `(from, to, good)` with `qty`, `value`, `carting`

Every flow between two towns in the year. `value` is at the sending town's
price; `carting` is what was paid on it, which the receiving town's merchants
pay to the sending town's carters (carriage by the road, plus the handling
charge). `carting / qty` is the transport cost of a unit on that road that
year.

### A works: `WORKS_FIELDS`, `[row][21]`

One row for every works that worked, built or stood in the year, by firm id:
`owner` (the family, -1 for the town's small capital), `method`, `town`,
`capacity` (at the year's end), `out`, `value`, `revenue`, `inputs`, `fuel`,
`cart_in`, `cart_out`, `hands`, `wages`, `rent`, `managers`, `profit`
(`revenue - inputs - fuel - cart_in - wages - rent - managers`), `dividends`,
`invest` (paid for capital goods), `util` (output over capacity, mean of the
months it worked), `cash` (at the year's end), `months` (it worked).

For the last complete year only, each works' materials, fuel and their
delivered cost by good are kept as well, and each town's twelve months of
price, make, sales and closing stock for every good (`MONTH_FIELDS`).

### The island: `IslandYear`

One for every complete year (`Books::years`). Its fields, in four groups.

**What was paid, by kind.**

| | |
|---|---|
| `pay_works`, `pay_land`, `pay_projects` | wages in works, on the land (sinkers included), on the railways and canals |
| `carting` | carters' pay |
| `managers` | hired managers |
| `div_households`, `div_families`, `works_kept` | works' dividends to their town's households (works no family owns) and to families; profit the works kept |
| `merchants_div`, `merchants_kept` | the merchants' dividend (to their town's households), and what they kept |
| `rent_farm`, `rent_pits`, `rent_sites`, `rent_town`, `rent_by_sector` | rents of farms, pastures and woods; of quarries, clay pits, iron mines and collieries (their royalties, under `mines`); of works' sites; of town houses; and the land's seven sectors one by one |
| `issue`, `fund_div` | new money and the common fund's dividend, both paid to households by head |

**Who received it.** `to_households`, `to_landlords`, `to_families` (each
family's `dividends` and `rents`), `to_fund`.

**What was spent.** `spend_households`, `spend_landlords`, `spend_families`
(goods), `rent_town` again (house rent), `invest`, `projects`;
`need_households` and `need_landlords` by need.

**The headlines.**

    wage_bill = pay_works + pay_land + pay_projects + carting + managers
    income    = wage_bill + works' profit + merchants' net + rents
    spending  = spend_households + spend_landlords + spend_families
                + rent_town + invest + projects

`income` and `spending` are the same number (the books check it), and it is
twelve times the observer's `gdp_nom` for the year. The four shares of it add
up to one:

- `share_work`: the wage bill.
- `share_works`: works' profit after the managers (`div_households +
  div_families + works_kept`).
- `share_merchants`: `merchants_div + merchants_kept`.
- `share_land`: the four rents.

`pay_share` is the wage bill over everything households received (pay,
dividends, the fund's dividend and new money): the share of household income
that is pay. Landlords and families receive no pay at all: a landlord's income
is rent, a family's is dividends and rent.

**Who owns what**, at the year's end:

- `works_value`, `works_value_families`: every works at depreciated capital
  cost, which is how a family's net worth counts its works
  (`families::works_value`: capacity built and on order, at the year's mean
  prices of the capital goods).
- `land_value`, `land_value_families`: `years_purchase` (25) times the rent
  the land paid in the year, all four kinds. A family's net worth prices its
  land the same way cell by cell, with two differences the books leave out:
  a cell's rent is never counted below `land_rent_floor` town wages a month,
  and under `harvest` it is the rent of an average year.
- `money`: who holds the money (households, landlords, merchants, works,
  families, projects, the fund).
- `works_years`, `land_years`, `ownership_years`: `works_value`, `land_value`
  and their sum over the wage bill. All the island's works and land, in
  years of all its pay.

Carters' pay is counted as pay for work. Under the port it is paid to
households for work nobody does (`carting.md`); under the game's rules the
carters exist.

## Where to read them

**In a live game.** `LiveYear::accounts` is the year's `IslandYear` (None
unless the books are kept). The eight headline series are in
`LiveYear::series` like any other: `acct_share_work`, `acct_share_works`,
`acct_share_merchants`, `acct_share_land`, `acct_pay_share`,
`acct_works_years`, `acct_land_years`, `acct_ownership_years` (group
"distribution"). They are monthly series like the rest, but each holds the
year's figure in all twelve of its months, written when the year's books
close; the months of the year now running are NaN. The yearly message stays
small; for the rest the viewer asks:

```rust
// one town's books for a year (None: the last complete year)
accounts::town_books(&sim.st, &sim.content, town, Some(1850)) -> Option<TownBooks>
// one works' books: every complete year of its life
accounts::works_books(&sim.st, &sim.content, firm) -> Option<WorksBooks>
// the island's account of the last complete year
accounts::island_year(&sim.st) -> Option<IslandYear>
```

All three are re-exported from the crate's root and all their types derive
`Serialize`.

`TownBooks` has `year`, `town`, `name`; `goods` (one `GoodBooks` per good, in
the content's order: the columns above by name, plus `bought`, the eight
buyers together, `unmet`, and `unplaced`, what the balance leaves over);
`land` (one `LandBooks` per sector); `methods` (a `MethodBooks` for each
method that worked there, with its `uses` good by good); `figures` (the
town's own figures as `(name, value)` pairs in `TOWN_FIELDS` order); `needs`;
`sent` and `received` (`TownFlow`: the other town, the good, `qty`, `value`,
`carting`); and `months`, for the last complete year only: `[month][good]`
of `[price, made, bought, stock]`.

`WorksBooks` has `firm`, `name`, `method`, `years` (a `WorksYear` for every
complete year it stood: the columns above by name, with `owner` as an
`Option`) and `inputs` (for the last complete year: each good it used, as
material and as fuel, and what both cost delivered).

`town_books` is None for a year not yet complete, a town slot that does not
exist, or with the books off. A works founded this year has no `years` yet.
`works_books` is None with the books off or for a works that does not exist;
with `accounts.works` off it comes back with no `years`.

A game that is loaded from a save (`save.rs`) has its books again: a save is
the orders, loading plays the years over, and the books are kept as they
were the first time.

**In a recorded run.** `run.bin` gets these arrays, only when the books were
kept. `n` is the number of years; every float is float32. `T` here is the
number of towns there are as the run ends (the header's `towns` list, and
`accounts.towns`), not the `T_max` slots of the file's other town arrays:
slots are filled in order, the rest would be nothing but zeros, and they were
a fifth of a game's whole file. A town founded late has rows of zeros for
the years before it.

| array | shape | |
|---|---|---|
| `acct_goods` | `[n, T, G, 22]` | `GOODS_FIELDS` |
| `acct_land` | `[n, T, NS, 7]` | `LAND_FIELDS` |
| `acct_methods` | `[n, T, M, 14]` | `METHOD_FIELDS` |
| `acct_towns` | `[n, T, 33]` | `TOWN_FIELDS` |
| `acct_needs` | `[n, T, S, 2]` | households; landlords and families |
| `acct_trade_off` | `[n + 1]` int32 | year `k`'s flows are rows `off[k]..off[k + 1]` of the next two |
| `acct_trade_key` | `[rows, 3]` int32 | from, to, good |
| `acct_trade_val` | `[rows, 3]` | qty, value, carting |
| `acct_use_off` | `[n + 1]` int32 | the same for what methods used |
| `acct_use_key` | `[rows, 3]` int32 | town, method, good |
| `acct_use_val` | `[rows, 3]` | used, burnt, cost |
| `acct_works_off` | `[n + 1]` int32 | year `k`'s works are rows `off[k]..off[k + 1]` of the next two |
| `acct_works_id` | `[rows]` int32 | firm id, ascending within a year |
| `acct_works` | `[rows, 21]` | `WORKS_FIELDS` |

and the header gets an `accounts` object: `start_year`, `towns` (the `T`
above), the column names of every table (`goods_fields`, `land_fields`, `method_fields`, `town_fields`,
`need_fields`, `trade_key`, `trade_val`, `use_key`, `use_val`,
`works_fields`), and `island`, the list of every year's `IslandYear`. A
`lite` pack leaves out the three works arrays. The last year's months and
the works' inputs by good are not in the file.

**From the command line.**

```bash
cargo run --release -p isostate-core --example accounts -- [--seeds 1-8] [--rules port|game]
    [--families N] [--end-year Y] [--good iron] [--year 1850] [--town NAME_OR_INDEX]
    [--works] [--island] [--years] [--no-works-books]
```

prints the table of shares below, the books' checks and cost, and a town's
books for one good. The sweep gets `income_share_work_Y`,
`income_share_works_Y`, `income_share_merchants_Y`, `income_share_land_Y`,
`income_pay_share_Y`, `income_ownership_years_Y` at the marks, and
`books_goods_worst` and `books_income_worst`, under "Work and wages", only
for runs that kept books.

## The numbers

`AccountsParams`, in `accounts.rs`. Off by default; `game()` switches it on.

| | | |
|---|---|---|
| `enabled` | false | keep the books |
| `works` | true | keep every works' books too; off, only the towns' and the island's |

## The checks

The books check themselves every month and every year, and keep what is left
over in the year's account, so a leak shows as a number.

- `balance_goods`: the identity above, for every town and good, added up in
  double precision; the worst town and good as a share of what passed through
  (opening stock, made, received), with where it was and how many units.
- `balance_income`: every month, what was paid as income against what
  households, landlords, families and the fund were credited with; the
  worst month.
- `balance_spending`: `income` against `spending`.

`tests/accounts.rs` holds all three under 1e-9 for every year of a run under
the port with six families, under the game's rules with six families, and
under the commons regime; checks the books against the observer's own series
(output of every good and method, `gdp_nom`), against the families' own
record of what they received, against the mines' own count of each digging
sector's hands, output and royalty, and against the canals' own count of
their navvies; and holds runs with the books on to runs with them off.

On seeds 1 to 8 to 1900, with six families, the worst of every year of every
run (the game's rules as they stand with the mines, the canals and kept town
ground beside the rest):

| | port | game |
|---|---|---|
| goods, as a share of what passed through | 6.5e-15 | 4.3e-15 |
| goods, in units of the good | 1.2e-9 | 7.1e-10 |
| incomes paid against received | 5.8e-13 | 2.7e-13 |
| income against spending | 7.0e-16 | 5.9e-16 |

That is rounding. With `oversold` (below) given its own column, the island's
goods and money add up. Seeds 9 to 16 under the game's rules say the same
(5.6e-15 and 2.7e-13 at worst over all sixteen).

**The books change nothing.** `tests/accounts.rs` compares four runs with the
books on and off bit for bit (every series, every news item, cash, prices,
stocks, capacities, land shares, the recorded map layers). Six sweeps on
seeds 1 to 16 say the same of forty-eight more, by every fact the sweep
keeps: under the port without families and with six, the books off and then
on, "of the 257 facts both have, the seconds aside, 0 differ" (259 with
families); under the game's rules with six families, 0 of 420. And seed 3
under the port, with and without families, writes the `series_monthly.csv`,
`events.json` and `params.json` it wrote before any mechanism was added.

## What does not balance

Two things in the economy itself, which the books found and do not hide.

**Goods are sold that are not in store.** A works that needs heat is rationed
on the average availability of the fuels it burns (`fill_heat` in
`econ::step_month`, weighted by its fuel mix), not on each fuel's own. So
when coal is short and charcoal is not, a works that burns both gets more
coal than its share of what is there. The town's stock would go below nothing
and is floored at nothing, which makes up the difference out of thin air. The
books count it as `oversold`, so that the rest balances.

It is small, and it is only the fuels. Over all eight seeds and all 151
years, as a share of everything bought of the good: coal 1.1e-5, charcoal
9.3e-7 and timber 9.0e-8 under the port with six families (1.3e-7, 2.4e-7
and 7.6e-9 under the game's rules); every other good 1e-17 or less, which is
rounding at the floor. By value, in the worst year of a run it is between
7.6e-6 and 3.8e-4 of everything bought on the island under the port, and
between 5.7e-8 and 5.1e-5 under the game's rules. A town can feel it more
than the island does: when the books were first tried (seed 7, the game's
rules of that day), Mallowwick's works burnt 24 units of coal in 1756 that
were never dug, six parts in ten thousand of the coal that passed through
the town that year. Rationing each fuel on its own availability would
end it; that is a change to the port, so it is not made here.

**Under the port, a town's wage can run to the ceiling for months, and the
land pays.** Rent on the land is what is left of the produce after wages,
carting and machines, and nothing keeps it above nothing. In four of the
eight seeds under the port with six families there is one year in which one
town's wage runs up to the bisection's ceiling (it may double each month, to
at most 500) and stays there for some months:

| seed | year | town | its mean wage that year | its land's produce | wages paid on it | land's share of the island's income | landlords in debt |
|---|---|---|---|---|---|---|---|
| 1 | 1835 | Redmoor | 215 (5 to 6 elsewhere) | 0.31 M | 4.83 M | -0.06 | 5 years |
| 2 | 1844 | | | | | -0.08 | 8 years |
| 3 | 1753 | Welford | 241 (4 to 5 elsewhere) | 0.72 M | 7.74 M | -0.72 | 28 years |
| 6 | 1833 | | | | | -0.02 | 26 years |

(Seeds 2 and 6 were not looked at town by town.) In seed 3 the island's wage
bill goes from 5.0 M to 12.5 M for one year and back, work's share of income
reads 1.53, and the landlords end the year 6.4 M in debt, four fifths of a
year of the whole island's income, which they pay off over the next 28
years. Money is conserved throughout: the landlords' purse simply goes below
nothing, and nothing in the economy minds. Every year in which this happens,
land's share and `land_years` are below nothing too, and the island's
headline shares for that year say more about one town's bad months than
about the island.

I did not trace the cause to the line. What the books show fits a town whose
farms' and pits' hands for the month (which move only 15 per cent of the way
to what the wage warrants, `labor_adjust_ex`) add up to more than the people
it has left after its district or its people were redrawn at the year's end:
then no wage clears the market, the wage doubles each month until enough
hands have been shed, and the land pays the bill. It does not happen under
the game's rules: in none of the eight seeds does land's share fall under
0.11 in any year or the landlords' cash go below nothing, which is what
`people` (households go with their cells, taking their wage with them) was
made to do.

## What the books say

Seeds 1 to 8, six families; the mean across seeds, then the least and
greatest. `cargo run --release -p isostate-core --example accounts -- --rules
port` (or `game`) prints these.

Under the port:

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| work's share of income | 0.692 [0.688 .. 0.697] | 0.678 [0.656 .. 0.693] | 0.590 [0.512 .. 0.638] | 0.599 [0.582 .. 0.629] |
| works' owners' share | 0.098 [0.091 .. 0.114] | 0.094 [0.039 .. 0.132] | 0.167 [0.108 .. 0.299] | 0.149 [0.113 .. 0.179] |
| merchants' share | 0.019 [0.011 .. 0.026] | 0.049 [0.022 .. 0.074] | 0.082 [0.036 .. 0.110] | 0.089 [0.058 .. 0.107] |
| land's owners' share | 0.190 [0.170 .. 0.204] | 0.179 [0.166 .. 0.199] | 0.160 [0.138 .. 0.174] | 0.163 [0.157 .. 0.174] |
| household income that is pay | 0.860 [0.839 .. 0.871] | 0.842 [0.818 .. 0.866] | 0.743 [0.634 .. 0.818] | 0.777 [0.748 .. 0.822] |
| works, in years of the wage bill | 0.157 [0.148 .. 0.166] | 0.433 [0.276 .. 0.623] | 1.136 [0.675 .. 1.886] | 1.110 [0.865 .. 1.411] |
| land, in years of the wage bill | 6.88 [6.16 .. 7.41] | 6.59 [6.18 .. 7.24] | 6.79 [6.24 .. 7.27] | 6.83 [6.25 .. 7.41] |
| works and land together | 7.04 [6.33 .. 7.56] | 7.03 [6.58 .. 7.52] | 7.93 [7.50 .. 8.71] | 7.94 [7.38 .. 8.82] |
| families' share of all works' value | 0.069 [0.055 .. 0.077] | 0.232 [0.163 .. 0.297] | 0.453 [0.320 .. 0.522] | 0.563 [0.461 .. 0.620] |

Under the game's rules (as they stand with the mines, the canals and kept
town ground in them):

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| work's share of income | 0.711 [0.704 .. 0.718] | 0.681 [0.655 .. 0.732] | 0.654 [0.591 .. 0.711] | 0.697 [0.677 .. 0.719] |
| works' owners' share | 0.060 [0.052 .. 0.082] | 0.095 [0.042 .. 0.142] | 0.117 [0.056 .. 0.182] | 0.087 [0.067 .. 0.108] |
| merchants' share | 0.028 [0.017 .. 0.036] | 0.052 [0.031 .. 0.076] | 0.063 [0.039 .. 0.082] | 0.042 [0.019 .. 0.059] |
| land's owners' share | 0.201 [0.188 .. 0.214] | 0.172 [0.157 .. 0.195] | 0.165 [0.148 .. 0.175] | 0.174 [0.169 .. 0.180] |
| household income that is pay | 0.900 [0.877 .. 0.922] | 0.859 [0.822 .. 0.925] | 0.842 [0.761 .. 0.901] | 0.886 [0.859 .. 0.905] |
| works, in years of the wage bill | 0.162 [0.154 .. 0.170] | 0.537 [0.243 .. 1.106] | 1.173 [0.811 .. 1.533] | 1.043 [0.877 .. 1.156] |
| land, in years of the wage bill | 7.06 [6.67 .. 7.54] | 6.30 [5.87 .. 6.69] | 6.31 [6.03 .. 6.66] | 6.23 [6.03 .. 6.41] |
| works and land together | 7.22 [6.83 .. 7.71] | 6.84 [6.57 .. 6.99] | 7.48 [6.97 .. 7.86] | 7.27 [6.95 .. 7.57] |
| families' share of all works' value | 0.068 [0.054 .. 0.081] | 0.415 [0.314 .. 0.465] | 0.429 [0.324 .. 0.501] | 0.507 [0.406 .. 0.578] |

Seeds 9 to 16 tell the same story: over all sixteen the sweep's facts give
work's share as 0.709, 0.679, 0.654 and 0.693 (in 1900 between 0.670 and
0.735), the share of household income that is pay as 0.894, 0.858, 0.835
and 0.887, and works and land together as 7.14, 6.90, 7.39 and 7.33 years.

Without families, under the port, the sweep's facts give 0.687, 0.675, 0.604
and 0.579 for work's share and 0.859, 0.820, 0.723 and 0.700 for the share
of household income that is pay.

What I would say from them, and no more:

- **Seven tenths of the island's income is pay in 1750, and a fifth is
  rent.** The two rule sets agree to within two points.
- **Work's share falls over the century under the port, by nine or ten
  points, and what it loses goes to the owners of works and to merchants,**
  not to land, whose share drifts down from 0.19 to 0.16. Under the game's
  rules it falls six points by 1850 and is back at 0.70 in 1900; the two
  rule sets' ranges for 1900 do not overlap (0.58 to 0.63 against 0.68 to
  0.72, or 0.67 to 0.74 on sixteen seeds). The single year 1850 is noisy:
  one seed's works' owners take 0.30 of income that year under the port.
- **Households live mostly on pay, and less so as the century goes on under
  the port** (0.86 to 0.74 or 0.78): works' and merchants' dividends are
  paid to the households of the works' town, so the island's small capital
  is owned by the people who work, and its growing profits reach them as
  dividends. Under the game's rules the share that is pay stays between
  0.84 and 0.90.
- **All the island's works and land cost about seven years of all its pay,
  eight by 1850 under the port.** Nearly all of that is land, and land's part is not a
  finding: land is priced at 25 years' rent by rule, so `land_years` is 25
  times land's share over work's and tells you the ratio of rent to wages
  and nothing about a market in land. The works' part is real, and it grows
  from two months of the wage bill to a year or a little more.
- **The six families own 7 per cent of the works' capital in 1750 and half
  of it or more in 1900**, under both rule sets.

Nothing here falls the way the labour work expects the twentieth century to:
there is no machine yet that takes work's share for good. But the gauges
exist now, and the run to 2050 can be read with them. Two islands run that
far under the game's rules (`--seeds 1,2 --end-year 2050`; the books balance
to the last year) give work's share as 0.70 in 1900, 0.65 in 1950, 0.66 in
2000 and 0.58 in 2050, the share of household income that is pay as 0.89,
0.85, 0.86 and 0.78, the works as 1.1, 1.6, 1.6 and 2.1 years of the wage
bill, and the families' part of them as 0.51 rising to 0.65. Two islands
are a hint and no more.

## A worked example

Iron in Ulford in 1850, seed 1, the game's rules, six families
(`--rules game --year 1850 --works`). Ulford is a town of 54,394 people; the
wage averaged 11.5; 14,977 hands were on the land, 10,412 in works and 1,229
carting.

```text
  in store as the year began         3803.9
  made                              29295.8
    by Coke blast furnace        29295.8   6.0 works, 218 hands, wages 30141, materials 34139, fuel 0,
                                           carting in 39 and out 267, rent 1032, profit 90326, dividends 85769
  received from other towns          6111.7
    from Wilmere                   658.1   worth 3163 there, carting 151 (0.229 a unit)
    from Oldford                  1373.1   worth 6624 there, carting 278 (0.203 a unit)
    from Ryeport                   512.8   worth 2405 there, carting 107 (0.210 a unit)
    from Ormstrand                 144.9   worth 703 there, carting 31 (0.216 a unit)
    from Elmness                   785.7   worth 3805 there, carting 133 (0.170 a unit)
    from Sandford                  165.1   worth 812 there, carting 40 (0.240 a unit)
    from Oakbridge                 194.6   worth 846 there, carting 29 (0.149 a unit)
    from Coldbank                 1264.4   worth 5647 there, carting 240 (0.190 a unit)
    from Stanside                  212.2   worth 1045 there, carting 52 (0.243 a unit)
    from Ashside                    78.5   worth 356 there, carting 12 (0.154 a unit)
    from Uldale                     87.3   worth 397 there, carting 16 (0.187 a unit)
    from Carford                   149.4   worth 800 there, carting 37 (0.244 a unit)
    from Crandelf                   37.5   worth 164 there, carting 6 (0.160 a unit)
    from Longwater                 106.0   worth 500 there, carting 24 (0.230 a unit)
    from Ashbank                   172.3   worth 855 there, carting 41 (0.238 a unit)
    from Barbrook                  169.9   worth 798 there, carting 40 (0.235 a unit)
  came with districts                   0.0
  sold though not in store              0.0
  bought                            31685.7
    by households                   29.5
    by landlords                     5.9
    by families                      3.4
    by works, as material        31646.9
  sent to other towns                 147.2
    to Ormstrand                    12.2   worth 66 here, carting 3 (0.235 a unit)
    to Carford                      13.6   worth 81 here, carting 4 (0.262 a unit)
    to Barbrook                    121.5   worth 699 here, carting 32 (0.267 a unit)
  spoiled in store                     72.2
  in store as the year ended         7306.3
  left unplaced by the books        -0.0005
  asked for 31835.8, of which unmet 150.1
  price 5.484 (from 4.833 to 6.671), the merchants' anchor 5.758
```

29,296 made and 6,112 brought in are 35,408; 31,686 bought, 147 sent and 72
spoiled are 31,905; the difference, 3,502, is what the stock grew by (3,804
to 7,306). The 0.0005 left over is single precision. Ulford is a forge
town: six furnaces make the iron, sixteen other towns send theirs as well at
about 0.2 a unit in carting (the iron itself is worth a little under 5 where
it comes from), and nearly all of it goes into the town's own works. Buyers asked for
150 units more than they got.

The largest of the six furnaces, year by year (`works_books`):

```text
Quennell & Sons Ironworks (Coke blast furnace), year by year
  year  owner  capacity      out  revenue   inputs     fuel  cart in  cart out    hands    wages     rent   profit  dividends  invest  util
  1847  Quenn     681.8   8273.0    37961     8062        0       15        98     60.6     7849      235    21801      21592       0  0.97
  1848  Quenn     761.4   8565.6    51652     9239        0       15       102     60.5     7858      235    34304      25499   27057  0.94
  1849  Quenn     857.2  10039.3    65463     9159        0       18       119     72.1     9386      253    46648      37543   33196  0.98
  1850  Quenn     964.0  10086.8    53720    11500        0       18       120     71.8     9934      308    31962      29257   35376  0.88
  1851  Quenn    1059.3  11936.1    48020    14805        0       21       142     81.5    11000      344    21850      16299   31596  0.94
  1852  Quenn    1021.1  12745.2    42715    12446        0       22       151     90.1    11513      344    18391      17802    1040  1.00
  1853  Quenn     980.4  12254.9    39957    13563        0       21       145     87.3    11521      344    14508      13712      32  1.00
```

The Quennells' furnace makes a third of the town's iron with seventy hands,
put some 30,000 a year into new capacity while iron was dear, and saw its
profit fall by two thirds in four years as the price came down. Its fuel is
nothing because a coke furnace takes its coal as a material, and its carting
in is next to nothing: its ore and coal reach it at about the town's price
(the `inputs` column is what the merchants were paid, which is the town's
price, or less for a works that stands nearer the source than the town does).

## What they cost

Per simulated year, measured inside the hooks (`Books::seconds`), eight
islands running at once on a machine that other work was also using:

| | the books | of the run | kept in memory |
|---|---|---|---|
| game's rules, six families | 2.9 ms | 1.9 to 2.0 per cent | 29 to 33 MB |
| port, six families | 9.5 ms | 6.0 to 6.9 per cent | 66 to 76 MB |
| port, six families, `works = false` | | 4.9 to 5.5 per cent | 25 to 28 MB |

(The first two rows are the review's, on the game's rules as they stand with
the mines and the canals; when the books were first written, on a busier
night, they read 3.8 and 13.0 ms and half a point more of the run. The third
row is from that night, so its share is the fairer figure and its
milliseconds are left out.) The port with
six families has seven thousand works by 1900 and the game's rules have a
few hundred, which is most of the difference. Half of the rest is one pass a
month over the land's units (tens of thousands of them), reading five of the
month's arrays. These are wall-clock figures on a shared machine and moved
by a third from one hour to the next; the shares are steadier than the
milliseconds.

The sweep's own clock, on sixteen seeds, eight islands at a time: under the
game's rules with six families a run takes 27.6 s with the books and 25.8 s
without, eight islands slower and eight faster, which is the noise of the
machine; under the port without families 21.3 s against 23.6 s, noise again.
Under the port with six families and their seven thousand works it is 25.1 s
against 21.9 s, and all sixteen islands are slower: three seconds a run,
twice what the hooks' own clock counts. The rest is presumably the works'
tables crowding the economy's own data out of the processor's cache; I did
not measure that. The port keeps no books unless it is asked to, so this is
the price of asking.

One thing cost far more than all of that until it was found, and whoever
adds the next observer should know of it. The first version copied the
month's dividends into a fresh vector every month and built each year's
tables as new vectors. In a single run that cost nothing measurable. In a
sweep, where eight islands run in one process and share its allocator, runs
took half as long again (33 s against 22 s) though the hooks themselves
accounted for one second: a block held from one month to the next, among
the hundred the economy allocates and frees each month, was enough to keep
the allocator from handing the same memory back, and every thread paid for
it. Eight separate processes showed no cost at all, which is how it was
found. The books now ask the allocator for nothing month by month: the
dense tables of every year are reserved when the books open, the sparse
ones grow by doubling, and the month's scratch is reused.

The run file of a game with six families grows from 24 MB to 44 MB with the
books in it (seed 1; it would be 55 MB if every town slot were written, not
only the towns there are). Most of what is left is the towns' goods and
methods, a row for every town, good or method and year whether or not the
town makes the thing; keeping only the rows with something in them would
be the next saving. A `lite` pack leaves the works out.

## When another mechanism moves goods or money

The books watch `econ::step_month` and nothing else, so a mechanism that
moves goods or money by another road has to tell them, or the balance test
will say so for it (`tests/accounts.rs`, `the_books_balance_under_the_game`).

- Goods that enter or leave a town's stock **inside the month** (exports
  abroad, a canal's own purchases) must go through the month's `sold`,
  `prod_TG`, `exports` or `imports`, with their own entry in
  `accounts::Month` and their own column, or the goods will not balance.
- Goods that appear in a stock **between months** are booked as `moved`
  unless the mechanism calls `accounts::note_cleared` (or something like it)
  to say they were made.
- Income paid to households, landlords or families inside the month must be
  one of the kinds in `TOWN_FIELDS`, or the month's income check fails.
- Rent on the land is worked out in the books from each sector's sums (value
  less carting, wages and inputs), because reading the per-unit arrays is
  what the land's pass costs. A mechanism that changes how a unit's rent is
  made (a royalty, a tithe) must change `accounts::note_month` with it; the
  income check will fail until it does.

It has happened once, and went as it should. When the books met the mines
and the canals three things had to be told to them, and the tests said so
(with the first alone left out, the month's income check fails in 1756 by
2.5 per cent, in four tests):

- `mines` pays the sinkers of a new working at the town's wage out of its
  royalty, hands the labour market never sees. The books now count them
  among the sector's hands (`mines::crew`), so wages and rent come out as
  they were paid.
- `mines` pays a working's royalty to the owners of every cell of its lease,
  not to the pit head's cell alone. The books read the land's rents as
  `econ::pay_cell_rents` was handed them (`Month::land_rents`), so a
  family's rents are the family's own figure again.
- `canals` pays its navvies two shillings on every shilling of materials
  where a railway pays three tenths, and takes them out of the town's hands.
  The books read each project's wages as they were paid
  (`Project::navvy_pay`), and a town's navvies have a column.

`tests/accounts.rs` (`the_books_keep_up_with_the_mines_and_the_canals`)
holds all three in place against the mines' and the canals' own counts.
Kept town ground needed nothing: a works' rent is still its `f_rent`.

## What worries me

- **Only complete years can be read.** The year now running is being added
  up and has no books yet, so a player who founds a works in March sees its
  books the next January. The last complete year's months are kept, for
  prices, make, sales and stock only.
- **The merchants' share is cash, not profit.** It is what the merchants
  took in less what they paid out. In a year they build stock it is low, in
  a year they run it down it is high; `stock_value` is kept beside it so the
  two can be read together, but the books do not revalue the stock.
- **Land has no market price.** `land_value` is 25 years' rent by rule, and
  it goes below nothing in the years rent does.
- **Capital movements are not in the income account.** Money put into new
  works and railways by a town's savers, a closed works' cash returned (or
  its debts, which its town's merchants take), land bought and sold, cash
  that travels with migrants, emigrants' fares: all happen at the year's end
  outside the month, and the books show only their result, in `money`. An
  account of who put what into which works would be the next table.
- **The shares for a founding year can be nonsense under the port** (above).
- **The works' books are dear where works are counted in thousands.** Under
  the port with six families they are more than half the books' cost and
  two thirds of what is kept. `accounts.works = false` drops them.
- **The land's rent in the books is worked out, not read** (see the section
  above). The monthly check holds it to what the landlords received, to
  rounding, on every run so far.
- **A works' inputs by good are kept for the last year only**; for earlier
  years there is its method's use of each good in its town.
- **Railways and canals share their columns** (`projects` among a good's
  buyers, `pay_projects` and `projects` among a town's figures). A chart that
  wants to tell a canal's bricks from a railway's rails can do it by the
  good, not by the buyer.
- **A town's hands do not quite add up to its supply.** `supply` is the
  hands for hire once carters, navvies and sinkers are taken out, and
  `hands_land` counts the sinkers, as the mines do; so in a year a working
  is sunk, land and works together can stand above `supply` by its forty
  sinkers.
- **A recorded game's file nearly doubles** (above).
- **Nothing draws any of this yet.** The viewer's flow chart is another
  helper's work; this page is what it reads.
