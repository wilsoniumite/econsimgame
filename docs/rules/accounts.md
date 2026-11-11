# Accounts

Books that say who makes, moves, buys and earns what. Not in the Python. The
switch is `accounts.enabled`; the code is `accounts.rs`, five marked places in
`econ::step_month`, and one line each in `sim.rs` (twice), `observer.rs`,
`land.rs`, `facts.rs`, `record.rs` (twice) and `live.rs`.

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

Everything is kept for every complete year in single precision (`Books::hist`,
one `YearRec` a year); the year now running is added up in double precision
and is not visible until it closes.

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
`wages`, `inputs` (materials at the town's prices), `fuel` (at the town's
prices), `cart_in` (carting of both out to the works), `cart_out` (carting of
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

FILL_CHECKS

## What does not balance

Two things in the economy itself, which the books found and do not hide.

**Goods are sold that are not in store.** A works that needs heat is rationed
on the average availability of the fuels it burns (`fill_heat`, weighted by
its fuel mix), not on each fuel's own, so when coal is short and charcoal is
not it takes more coal than the market has. The town's stock would go below
nothing and is floored at nothing, which conjures the difference. The books
count it as `oversold`.

FILL_OVERSOLD

**In a year a district changes hands, the land can pay several times its
produce in wages.** Rent is what is left of the produce after wages and
carting, and nothing stops it going below nothing; the landlords pay the
difference out of cash they do not have.

FILL_WELFORD

## What the books say

FILL_SAY

## A worked example

FILL_EXAMPLE

## What they cost

FILL_COST

## What worries me

FILL_WORRIES
