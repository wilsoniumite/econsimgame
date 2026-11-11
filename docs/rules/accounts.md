# Accounts

Books that say who makes, moves, buys and earns what. Not in the Python. The
switch is `accounts.enabled`; the code is `accounts.rs`, five marked calls in
`econ::step_month` (and one line there reshaped, so that the hired managers'
cuts live long enough to be read), and a few marked lines each in `sim.rs`,
`observer.rs`, `land.rs`, `facts.rs`, `record.rs` and `live.rs`.

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
the sweeps below hold sixteen more). Under the port they are off, nothing is
kept, and `params.json` does not mention them.

## What is kept

Money and quantities are the year's totals (the twelve months added up) unless
a column says otherwise. Hands, capacity and the count of works are means over
the twelve months. A year's books close at the end of the yearly step, after
that year's openings, closures and purchases, so "at the year's end" means as
the next January begins.

Everything is kept for every complete year in single precision
(`Books::hist`, flat tables with the years one after another, in the run
file's layout; `Books::year(k)` gives one year's as a `YearRec` of slices).
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
| `projects` | bought by railways under construction (and by anything else that builds through `State::projects`) |
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
economy, and in a bad month it is below nothing.

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

### A town's own figures: `TOWN_FIELDS`, `[town][32]`

| columns | |
|---|---|
| `pop` | people at the year's end |
| `wage`, `supply`, `carters`, `hands_land`, `hands_works` | means: the wage, hands for hire (carters taken out), hands carting (0 unless `carting` is on), hands on the land and in works |
| `pay_works`, `pay_land`, `pay_projects`, `cart_works`, `cart_land`, `cart_trade`, `managers`, `div_works`, `div_merchants`, `fund`, `issue` | what the town's households received, by kind; the eleven add up to all they received |
| `div_families` | dividends of family works here |
| `rent_land`, `rent_sites`, `rent_town` | rents paid in the town's district: farms, woods, pits and mines; works' sites; town houses |
| `rent_families` | of those three, what went to families |
| `spend_households`, `spend_landlords`, `spend_families` | spending on goods |
| `invest`, `projects` | works' purchases of capital goods; railways' outlay (materials and navvies) |
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
| `pay_works`, `pay_land`, `pay_projects` | wages in works, on the land, on the railways |
| `carting` | carters' pay |
| `managers` | hired managers |
| `div_households`, `div_families`, `works_kept` | works' dividends to their town's households (works no family owns) and to families; profit the works kept |
| `merchants_div`, `merchants_kept` | the merchants' dividend (to their town's households), and what they kept |
| `rent_farm`, `rent_pits`, `rent_sites`, `rent_town`, `rent_by_sector` | rents of farms, pastures and woods; of quarries, pits and mines; of works' sites; of town houses |
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

**In a recorded run.** `run.bin` gets these arrays, only when the books were
kept. `n` is the number of years; every float is float32.

| array | shape | |
|---|---|---|
| `acct_goods` | `[n, T, G, 22]` | `GOODS_FIELDS` |
| `acct_land` | `[n, T, NS, 7]` | `LAND_FIELDS` |
| `acct_methods` | `[n, T, M, 14]` | `METHOD_FIELDS` |
| `acct_towns` | `[n, T, 32]` | `TOWN_FIELDS` |
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

and the header gets an `accounts` object: `start_year`, the column names of
every table (`goods_fields`, `land_fields`, `method_fields`, `town_fields`,
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
(output of every good and method, `gdp_nom`) and against the families' own
record of what they received; and holds runs with the books on to runs with
them off.

On seeds 1 to 8 to 1900, with six families, the worst of every year of every
run:

| | port | game |
|---|---|---|
| goods, as a share of what passed through | 6.5e-15 | 4.1e-15 |
| goods, in units of the good | 1.2e-9 | 4.9e-10 |
| incomes paid against received | 5.9e-13 | 6.9e-15 |
| income against spending | 7.1e-16 | 7.2e-16 |

That is rounding. With `oversold` (below) given its own column, the island's
goods and money add up.

**The books change nothing.** `tests/accounts.rs` compares four runs with the
books on and off bit for bit (every series, every news item, cash, prices,
stocks, capacities, land shares, the recorded map layers). The four sweeps
(seeds 1 to 8, port rules, with and without six families, the books off and
on) say the same of sixteen more: "of the 165 facts both have, the seconds
aside, 0 differ", both times.

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
9.3e-7 and timber 9.0e-8 under the port with six families (5.9e-7, 1.8e-6
and 2.3e-8 under the game's rules); every other good 1e-18 or less, which is
rounding at the floor. By value, in the worst year of a run it is between
7.6e-6 and 3.8e-4 of everything bought on the island under the port, and
between 2.6e-7 and 1.1e-4 under the game's rules. A town can feel it more
than the island does: on seed 7 in 1756, Mallowwick's works burnt 24 units of
coal that were never dug, six parts in ten thousand of the coal that passed
through the town that year. Rationing each fuel on its own availability would
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
0.135 in any year or the landlords' cash go below nothing, which is what
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

Under the game's rules:

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| work's share of income | 0.703 [0.696 .. 0.715] | 0.671 [0.642 .. 0.712] | 0.647 [0.599 .. 0.687] | 0.698 [0.666 .. 0.727] |
| works' owners' share | 0.069 [0.057 .. 0.084] | 0.107 [0.046 .. 0.143] | 0.115 [0.072 .. 0.180] | 0.088 [0.068 .. 0.117] |
| merchants' share | 0.037 [0.026 .. 0.043] | 0.050 [0.023 .. 0.084] | 0.066 [0.011 .. 0.099] | 0.040 [0.015 .. 0.064] |
| land's owners' share | 0.191 [0.185 .. 0.201] | 0.172 [0.160 .. 0.201] | 0.172 [0.150 .. 0.193] | 0.174 [0.169 .. 0.178] |
| household income that is pay | 0.875 [0.858 .. 0.904] | 0.854 [0.818 .. 0.910] | 0.839 [0.786 .. 0.898] | 0.891 [0.863 .. 0.931] |
| works, in years of the wage bill | 0.170 [0.161 .. 0.184] | 0.586 [0.312 .. 1.158] | 1.204 [0.881 .. 1.490] | 1.018 [0.850 .. 1.262] |
| land, in years of the wage bill | 6.79 [6.56 .. 7.17] | 6.42 [6.04 .. 7.04] | 6.65 [5.69 .. 7.76] | 6.23 [5.85 .. 6.57] |
| works and land together | 6.96 [6.73 .. 7.35] | 7.01 [6.58 .. 7.35] | 7.86 [6.76 .. 9.25] | 7.25 [6.72 .. 7.83] |
| families' share of all works' value | 0.071 [0.057 .. 0.094] | 0.416 [0.282 .. 0.493] | 0.472 [0.267 .. 0.638] | 0.558 [0.396 .. 0.734] |

Without families, under the port, the sweep's facts give 0.687, 0.675, 0.604
and 0.579 for work's share and 0.859, 0.820, 0.723 and 0.700 for the share
of household income that is pay.

What I would say from them, and no more:

- **Seven tenths of the island's income is pay in 1750, and a fifth is
  rent.** Both rule sets agree to a point.
- **Work's share falls over the century under the port, by nine or ten
  points, and what it loses goes to the owners of works and to merchants,**
  not to land, whose share drifts down from 0.19 to 0.16. Under the game's
  rules it falls six points by 1850 and is back at 0.70 in 1900; the two
  rule sets' ranges for 1900 do not overlap (0.58 to 0.63 against 0.67 to
  0.73). The single year 1850 is noisy: one seed's works' owners take 0.30
  of income that year under the port.
- **Households live mostly on pay, and less so as the century goes on under
  the port** (0.86 to 0.74 or 0.78): works' and merchants' dividends are
  paid to the households of the works' town, so the island's small capital
  is owned by the people who work, and its growing profits reach them as
  dividends. Under the game's rules the share that is pay stays near 0.85
  to 0.89.
- **All the island's works and land cost about seven years of all its pay,
  eight by 1850.** Nearly all of that is land, and land's part is not a
  finding: land is priced at 25 years' rent by rule, so `land_years` is 25
  times land's share over work's and tells you the ratio of rent to wages
  and nothing about a market in land. The works' part is real, and it grows
  from two months of the wage bill to a year or a little more.
- **The six families own 7 per cent of the works' capital in 1750 and more
  than half of it in 1900**, under both rule sets.

Nothing here falls the way the labour work expects the twentieth century to:
there is no machine yet that takes work's share for good. But the gauges
exist now, and the run to 2050 can be read with them.

## A worked example

Iron in Ryeport in 1850, seed 1, the game's rules, six families
(`--rules game --year 1850 --works`). Ryeport is a town of 85,875 people; the
wage averaged 11.2; 26,849 hands were on the land, 16,491 in works and 2,722
carting.

```text
  in store as the year began          7368.6
  made                               38029.1
    by Coke blast furnace          38029.1   6.0 works, 295 hands, wages 39708, materials 51426, fuel 0,
                                             carting in 0 and out 639, rent 1358, profit 124382, dividends 96343
  received from other towns           7128.7
    from Ulford                     2042.5   worth 10859 there, carting 451 (0.221 a unit)
    from Wilmere                     293.9   worth 1680 there, carting 66 (0.225 a unit)
    from Oldford                    1994.4   worth 10829 there, carting 391 (0.196 a unit)
    from Ormstrand                   128.5   worth 748 there, carting 36 (0.280 a unit)
    from Elmness                     452.5   worth 2323 there, carting 109 (0.240 a unit)
    from Sandford                   1136.0   worth 5716 there, carting 258 (0.227 a unit)
    from Oakdelf                      60.8   worth 303 there, carting 13 (0.208 a unit)
    from Coldhope                    176.8   worth 899 there, carting 34 (0.191 a unit)
    from Stansea                     441.5   worth 2243 there, carting 95 (0.216 a unit)
    from Ashwick                     355.5   worth 1868 there, carting 89 (0.251 a unit)
    from Uldale                       46.3   worth 162 there, carting 6 (0.129 a unit)
  came with districts                    0.0
  sold though not in store               0.0
  bought                             40569.6
    by households                      6.8
    by landlords                       1.6
    by families                        1.0
    by works, as material          40560.2
  sent to other towns                  126.5
    to Ashwick                       126.5   worth 825 here, carting 37 (0.289 a unit)
  spoiled in store                     155.4
  in store as the year ended         11674.9
  left unplaced by the books          0.0016
  asked for 41527.4, of which unmet 957.8
  price 5.908 (from 5.300 to 6.904), the merchants' anchor 6.997
```

38,029 made and 7,129 brought in are 45,158; 40,570 bought, 127 sent and 155
spoiled are 40,852; the difference, 4,306, is what the stock grew by (7,369
to 11,675). The 0.0016 left over is single precision. Ryeport is the
island's forge: six furnaces make the iron, eleven other towns send theirs
as well at about 0.2 a unit in carting (the iron itself is worth 5 to 6), and nearly all of
it goes into the town's own works. Buyers asked for 958 units more than they
got.

The largest of the six furnaces, year by year (`works_books`):

```text
Quennell & Sons Ironworks No. 3 (Coke blast furnace)
  year  owner  capacity      out  revenue   inputs  cart out    hands    wages     rent   profit  dividends  invest  util
  1848  Quenn    1073.5  12595.4   101180    12332       213     93.6    11381      217    77250      58899   35241  0.98
  1849  Quenn    1204.1  13566.7   107138    15095       229     98.7    12248      251    79544      64500   45900  0.94
  1850  Quenn    1319.0  14157.0    80763    18028       239    103.0    13861      286    48588      37828   45459  0.89
  1851  Quenn    1436.5  16403.8    70601    19182       277    117.2    15910      322    35187      29340   44126  0.95
  1852  Quenn    1384.4  17267.8    62460    17488       291    126.3    16047      322    28603      26824    1513  1.00
```

The Quennells' furnace makes a third of the town's iron with a hundred
hands, put 45,000 a year into new capacity while iron was dear, and saw its
profit fall by two thirds in four years as the price came down. Its carting
in is nothing because it stands nearer the pits than the town does: it buys
ore and coal below the town's price (the `inputs` column is what the
merchants were paid, which is the town's price or less).

## What they cost

Per simulated year, measured inside the hooks (`Books::seconds`), eight
islands running at once on a machine that other work was also using:

| | the books | of the run | kept in memory |
|---|---|---|---|
| game's rules, six families | 3.8 ms | 2.3 to 2.6 per cent | 26 to 34 MB |
| port, six families | 13.0 ms | 6.5 to 7.7 per cent | 66 to 76 MB |
| port, six families, `works = false` | | 4.9 to 5.5 per cent | 25 to 28 MB |

(The third row was taken on a busier machine than the other two, so its
share is the fairer figure and its milliseconds are left out.) The port with
six families has seven thousand works by 1900 and the game's rules have a
few hundred, which is most of the difference. Half of the rest is one pass a
month over the land's units (tens of thousands of them), reading five of the
month's arrays. These are wall-clock figures on a shared machine and moved
by a third from one hour to the next; the shares are steadier than the
milliseconds.

The sweep's own clock agrees for the port without families (23.1 s a run with
the books against 24.9 s without, inside the noise of that night) and shows
more with six families and their seven thousand works (29.4 s against 25.8).

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

In the run file the books add what they keep in memory, 26 to 34 MB for a
game with six families. Two thirds of the dense tables are zeros (48 town
slots, a dozen to twenty towns).

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
- **Nothing draws any of this yet.** The viewer's flow chart is another
  helper's work; this page is what it reads.
