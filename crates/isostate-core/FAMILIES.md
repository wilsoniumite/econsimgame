# Families and the live API

This adds the company half of the family game to the core: families that own
works and land, a player family that acts through an API, rival AI families
that act once a year, and a live API so the viewer can run the simulation in a
background thread and draw each year as it is simulated. None of it is in the
Python.

Everything is off by default. With `n_families = 0` the state has no families
and no owner tables, every hook returns at once, and a run is byte for byte the
same as before (checked: `series_monthly.csv`, `events.json`, `params.json` and
`run.bin` are identical apart from the date in the header, and all parity tests
pass unchanged). Turn families on with `n_families` in the parameters (TOML or
JSON), or `isostate-core run --families 6`.

## The pieces

- `src/families.rs`: families, members, ownership, the money flows, the
  actions, the AI and the reports.
- `src/live.rs`: `live_static` and `live_year`.
- Small hooks elsewhere: `econ::step_month` (dividends, rents, spending, the
  management bonus, `money_total`), `firms.rs` (a family backs its own works and
  gets their cash when they close; sister works inherit the owner),
  `sim.rs` (families start after the warm-up and act at the end of each year;
  closures of family works name the family), `record.rs` (the yearly layers
  and static encodings are shared with the live API), `names.rs` (family and
  member names), `state.rs` and `params.rs`.
- `tests/families.rs` and `examples/live.rs`.

## Families and members

Family 0 is the player; families 1 and up are AI. AI families alternate between
two strategies: odd indices are landed, even ones industrial. Home towns go to
the families in order of town size (the player gets the largest town, family 1
the next, and so on, wrapping round if there are more families than towns).

Each family starts with three to six members: a head aged 40 to 60 and others
aged 5 to 40, each with a first name, an age and a skill drawn uniformly
between 0.2 and 0.9. Each member also gets a lifespan between 55 and 80, drawn
at birth and not shown. Members age a year at each year end and die at their
lifespan. When fewer than three are left alive, an heir of 18 joins. The
draws come from the `misc` random stream, which the Python never uses, so
turning families on does not change any other stream.

Adults (18 and over) can manage one industry group each, and a group has one
manager:

| Group | Works | Land units on family land |
|---|---|---|
| Iron and steel | bloomery, charcoal and coke furnaces, crucible steel, converter, forged rails, rolling and steel rail mills | |
| Textiles | handloom weaving, powered mill | |
| Coal and mining | charcoal burning | iron mines, collieries |
| Engineering | smithy, millwright, engine works | |
| Building materials | brickworks | quarries, clay pits |
| Estates | | farms, pastures, woodland |

Shops and trades belong to no group. A family's works in a managed group need
`1 - family_bonus * skill` of the inputs, heat and labor per unit that other
works need (up to 8 per cent less at skill 1). The land units on a family's
cells in a managed group get the same bonus as extra productivity. The player
assigns managers with `assign_member`; AI families put their most skilled
adults in charge of the groups they hold most in, every year.

## Ownership and money

`State::f_owner` holds each firm's owner and `State::cell_owner` each cell's,
-1 meaning nobody in particular: the anonymous small capital that funds firms
through the town pools, and the anonymous gentry who receive rent. Every flow
is a transfer inside the money ledger, and `econ::money_total` includes the
families' cash, so the monthly conservation check holds with families on.

- **Dividends** of a family's works go to the family instead of the households
  of the works' town.
- **Rent** paid on a family cell (by the land units on it, by the works on it,
  and the cell's share of its town's urban rent) goes to the family instead of
  the landlords (or, under the commons regime, the common fund).
- **Spending**: each month a family spends `family_cons_income` of last month's
  dividends and rents plus `family_cons_wealth` of its cash, never more than its
  cash, like the landlords' budget. The budget is added to the home town's
  landlord budget, so it buys the landlords' basket in the home town's market;
  the family then repays the landlords its share of what was bought. Goods,
  prices and merchants see one more rich household.
- **Backing**: a family's young works that run short, and its growing works,
  are topped up from the family's cash instead of the town pool. When a family
  works closes, its cash goes to the family.
- **The start**: after the warm-up, every family gets `family_start_cash` of the
  money stock, taken from the households and landlords of all towns in
  proportion to their cash. Each landed family gets the farmland nearest its
  home town until it holds `family_start_share` of all rent; each industrial
  family gets the largest works of its home town until it holds that share of
  all works' capital. The player gets that share of works' capital in cash, to
  build with as it likes.

## What a family can do

All return `Result` with a readable error.

- `site_options(state, content, params, family, cell)`: for every method, the
  sums `firms::evaluate_sites` gives entry for that cell (gate price, inputs at
  delivered prices, heat, labor, margin, rent bid, the cost of founding and the
  expected return), with the family's management bonus included, or why the
  method cannot be built there (not known in the town yet). Errors if nothing
  can be built on the cell (sea, out of every town's reach, derelict, full).
- `found_works(..., family, method, cell, year, events)`: builds the works as
  entry would (`firms::create_firm`, capacity `entry_K_frac` of the method's
  full size), paid from the family's cash, owned by the family, with a reason
  record of kind "family" and a news item ("The Ashby family opens a coke
  blast furnace at Kirkford"). The works is named after the family.
- `buy_land(..., family, cell)`: at `land_price`, `years_purchase` years of the
  rent the cell paid over the last year (never counted below `land_rent_floor`
  town wages a month, so unused land is cheap but not free). The money goes to
  the seller: another family, or for the gentry's land, wherever that cell's
  rent goes now (the landlords of its town, or the common fund).
- `assign_member(state, family, member, group)`: `Some(group)` or `None` to
  stand down; the group's previous manager stands down.

## AI families

At the end of each year, after entry and railway planning, each AI family keeps
a reserve of `family_reserve_months` months of spending and invests the rest:

- Industrial families evaluate a few candidate cells per method
  (`firms::candidate_cells`, the same sampling entry uses) in their home town
  and the nearest other town, with their bonus applied, and found up to three
  works with the best returns above the hurdle rate that they can afford.
- Landed families first buy up to four cells of farmland in their home town's
  reach, next to their estates where they can, highest rent first; then they
  found up to two works where the return is at least 1.5 times the hurdle rate.

That is all: a few dozen site evaluations per family per year.

## Reports and events

`family_report` gives cash, net worth (cash, works at depreciated capital cost
counting capacity on order, land at `land_price`), the last complete year's
income by source (dividends by group, other dividends, rents, land sales) and
its spending and investment, the firms and cells held, the members, and the
bonus each group gets now. `ownership_shares` gives the families' combined
share of works' capital, of rent paid and of the money stock.

News items of kind "family" (with `Event::family` set) cover family works
opening (weight 0.6 for the player, 0.35 for AI families), deaths of members
(0.5 and 0.2) and AI land purchases (0.15). Closures of family works say whose
they were. AI families produce many items (about 1,500 over a run with six
families), so the viewer will want to filter by weight.

## The live API

`live_static(&State, &Content) -> LiveStatic` after `sim::start`, then a
`LiveCursor::new(&Content)` and, after each `sim::run_year`,
`live_year(&mut cursor, &State, &Content, &Params, &sim.events) -> LiveYear`.
`LiveStatic` has everything `record::pack` writes once (size, height, terrain,
rivers, flow directions, the resource fields as bytes, goods, methods with
their output good, full size and group, techs, sectors, use and display class
names, the layer names, the series names and labels, the groups). `LiveYear`
has the year's nine map layers (the bytes `record::snapshot` writes, checked
equal in the tests), the town arrays, every live firm's capacity, full
information on firms and towns new since the last call (with owner and reason
sums), firms closed since the last call, railways under construction, news
since the last call, the year's mean of every series, every family's report
and every cell's owner.

`live_year` takes about 1 ms (worst 9 ms in a run on a loaded machine), far
under the 30 ms budget; most of it is the display mix of the map layers. A run
with six families took 25.1 s against 24.3 s without, back to back on the same
loaded machine: the families cost about 3 per cent.

## How much families own in a default run

Seed 7, landlords regime, six families (the player, who does nothing by itself
in a default run, three landed and two industrial AI families), from
`cargo run -p isostate-core --release --example live`:

| Year | Works' capital | Rent paid | Money stock |
|---|---|---|---|
| 1750 | 6.2% | 9.9% | 4.3% |
| 1800 | 2.5% | 11.8% | 3.6% |
| 1850 | 2.5% | 16.6% | 5.1% |
| 1900 | 6.8% | 18.4% | 7.9% |

This looks sane in kind: the families matter without owning the island. The
landed families slowly gather land (about 260 to 370 cells each by 1900, out of
roughly 5,000 that pay rent) and end with net worths around 15 million, almost
all of it land. Two things are less satisfying:

- The industrial families lose ground for a century. They start with about 3
  per cent of works' capital each, but they can found at most three small
  works a year near home while the anonymous savers of 18 towns found dozens,
  so their share halves by 1800 and only recovers when the landed families
  start building works too. If AI industrialists should keep pace, they need
  bigger works or more of them; `family_cons_income`, `family_cons_wealth` and
  `family_reserve_months` are the levers.
- Land dominates wealth. At 25 years' purchase all land is worth about 14
  times the money stock in 1750, so a landed family with 3 per cent of the rent
  is worth some 50 times an industrial family with 3 per cent of the works.
  That is historically fair for 1750 but will matter for balance.

The player's family, left alone, spends its starting cash at 2 per cent a month
and ends with nothing; in a game the player acts.

## Surprises

- **A new town's land is priced as if labor were free for a year.** In the
  Python (and so in the port), a town founded at the end of a year has a mean
  wage of zero for that year (`wage_acc` only starts accumulating the next
  month), and `land.update_land_use` clamps it to 1e-6. The Cobb-Douglas land
  bids around the new town then come out near 1e19, so the rent map (`cell_rent`)
  is astronomical there for that year. Entry steers clear (the rent bids make
  every return negative), so the economy barely notices, but a land market
  priced on `cell_rent` would. That is why `land_price` uses the rent each
  cell actually paid over the last year, booked month by month
  (`State::cell_rent_paid`, `State::cell_rent_year`). It is worth fixing in the
  Python (give a new town its donor's wage in `wbar`).
- **Values must count works under construction.** A new works has no capacity
  until its capital goods arrive, so valuing works at built capacity made every
  founding look like a loss; works are valued on capacity built and on order.
- **Spending through the landlords' basket** keeps the demand system
  unchanged, but it means family luxury spending shows up in the landlords'
  spending series.
- **Python sends a failed sister works' money to the households** (Python bug
  6 in PORTING.md). For a family works the money goes back to the family; for
  everyone else the port still does what the Python does.

## Not done

- Families never sell works or land of their own accord, and there is no market
  for works between families. The player can buy any cell, including another
  family's, at `land_price`.
- `run.bin` does not record owners or families; the live API carries them.
- Members do not marry or have children beyond the heir rule; the family keeps
  at least three living members.
- No politics yet: nothing here touches the government.
