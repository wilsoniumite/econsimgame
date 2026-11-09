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
  The sums also say how deep the market is: the works' output a month, what
  the town's buyers asked for last month (`town_demand`), what the town made,
  the works' `market_share` of that demand and the good's `price_ratio` to its
  base price. The expected return is a snapshot at last year's prices and 85
  per cent utilisation; where `market_share` is more than a few tenths or the
  price several times its base, it will not survive the works' own output
  (see "Is there a game?").
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
  and the nearest other town, with their bonus applied, found up to three
  works with the best returns above the hurdle rate that they can afford, and
  put what is left into up to two cells of land.
- Landed families first buy up to four cells of farmland in their home town's
  reach, next to their estates where they can, highest rent first; then they
  found up to two works where the return is at least 1.5 times the hurdle rate.
- Both build only where the works' output is at most `family_max_share` (0.25)
  of the town's demand and the price at most `family_max_price_ratio` (3) times
  its base: thin markets are traps (below).

That is all: a few dozen site evaluations per family per year.

## Reports and events

`family_report` gives cash, net worth (cash, works at depreciated capital cost
counting capacity on order, land at `land_price`), the last complete year's
income by source (dividends by group, other dividends, rents, land sales) and
its spending and investment, the firms and cells held, the members, and the
bonus each group gets now. `ownership_shares` gives the families' combined
share of works' capital, of rent paid and of the money stock.

News items of kind "family" (with `Event::family` set) cover every works the
player opens (weight 0.6), a rival's first works of each kind (0.35), deaths of
members (0.5 for the player's, 0.2 for rivals') and a rival's estates passing
another hundred holdings (0.15). Closures of family works say whose they were.
A run with six families has about 340 news items, against 240 without
families. `State::f_paid` books what each family works has paid its family
(dividends, and its cash when it closed).

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
| 1750 | 6.1% | 10.0% | 3.9% |
| 1800 | 2.2% | 11.9% | 3.3% |
| 1850 | 2.5% | 18.2% | 4.3% |
| 1900 | 6.4% | 20.2% | 7.6% |

This looks sane in kind: the families matter without owning the island. The
landed families slowly gather land (about 290 to 440 cells each by 1900, out of
roughly 5,000 that pay rent) and end with net worths of 14 to 18 million,
almost all of it land. Two things are less satisfying:

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

## Is there a game?

`examples/bots.rs` plays the player's family with five scripted strategies,
through the public API only, over seeds 1, 2, 3 and 7 with the defaults and six
families:

- **passive**: does nothing.
- **greedy**: every year founds up to four works with the best expected
  returns above the hurdle rate that it can afford, keeping a quarter of its
  cash, searching the cells within two squares of every town. That is about 450
  cells and 7,000 site evaluations a year, 0.5 to 1 second of a 20 to 30 second
  run.
- **landlord**: every year buys the gentry's land with the best rent per price,
  up to 40 cells, keeping a quarter of its cash.
- **manager**: greedy, and puts its adults in charge of the groups it has most
  works in.
- **prudent**: manager, but builds only where `market_share` is at most 0.25
  and `price_ratio` at most 3, and puts what is left over into land.

Net worth in thousands (k) or millions (m) of money; rank among the six
families (1 is richest); works founded and still standing in 1900; the mean
yearly return `site_options` promised for them and the mean they realised
(dividends and closing cash, plus what is left, less what they cost, per year
of life):

| Seed | Strategy | 1775 | 1800 | 1850 | 1900 | Rank | Works | Promised | Realised |
|---|---|---|---|---|---|---|---|---|---|
| 1 | passive | 0.07k | 0 | 0 | 0 | 6/6/6/6 | | | |
| 1 | greedy | 12.7k | 34k | 215k | 714k | 6/6/6/6 | 591, 167 | 1378% | 185% |
| 1 | landlord | 38.5k | 108k | 764k | 4.14m | 5/5/5/4 | 293 cells | | |
| 1 | manager | 18.5k | 47k | 259k | 1.01m | 6/6/6/6 | 600, 285 | 1367% | 198% |
| 1 | prudent | 33.4k | 211k | 4.20m | 26.8m | 6/4/4/1 | 562, 372 | 104% | 53% |
| 2 | greedy | 5.4k | 36k | 70k | 523k | 6/6/6/6 | 581, 163 | 1357% | 221% |
| 2 | landlord | 59.0k | 141k | 1.12m | 6.00m | 4/5/4/4 | 353 cells | | |
| 2 | manager | 8.0k | 34k | 269k | 1.31m | 6/6/6/6 | 602, 276 | 1451% | 215% |
| 2 | prudent | 42.2k | 246k | 1.45m | 16.8m | 6/4/4/1 | 582, 304 | 94% | 41% |
| 3 | greedy | 4.0k | 0 | 0 | 0 | 6/6/6/6 | 126, 0 | 4150% | 639% |
| 3 | landlord | 37.3k | 112k | 808k | 5.08m | 6/4/4/4 | 311 cells | | |
| 3 | manager | 3.7k | 32k | 212k | 692k | 6/6/5/5 | 595, 252 | 1502% | 306% |
| 3 | prudent | 33.9k | 144k | 1.34m | 9.36m | 5/4/4/4 | 578, 283 | 109% | 31% |
| 7 | greedy | 10.2k | 34k | 112k | 467k | 6/5/5/5 | 592, 150 | 1786% | 240% |
| 7 | landlord | 109k | 206k | 1.20m | 7.29m | 4/4/4/4 | 378 cells | | |
| 7 | manager | 11.6k | 37k | 236k | 864k | 6/5/4/5 | 601, 195 | 1575% | 212% |
| 7 | prudent | 116k | 280k | 2.21m | 20.8m | 4/4/4/1 | 557, 323 | 88% | 46% |

The passive family is last in every seed and year (it has spent its 33k by
1775). The richest rival in 1900 had 26.2m, 14.1m, 11.9m and 17.7m in the
prudent runs of seeds 1, 2, 3 and 7. So the prudent player wins three seeds
of four, one of them by 2 per cent, and comes fourth in the other; the
landlord comes fourth; the builders who trust the promised returns come fifth
or sixth, and in seed 3 the greedy one is bankrupt by 1800. The mean realised
return is pulled up by a few works in thin markets that paid out hugely
before they died; the medians by method, over all the builders' works, tell
the story better:

| Method | Works | Promised | Realised | Standing in 1900 | Price over base when founded |
|---|---|---|---|---|---|
| textile mill | 263 | 41% | 36% | 100% | 1.8 |
| coke furnace | 155 | 48% | 18% | 99% | 3.1 |
| brickworks | 231 | 44% | 17% | 71% | 2.5 |
| engine works | 664 | 92% | 17% | 73% | 2.3 |
| crucible steel | 2,219 | 87% | 29% | 63% | 1.0 |
| rolling mill | 125 | 59% | 6% | 74% | 2.4 |
| smithy | 659 | 96% | 5% | 29% | 2.6 |
| shops and trades | 726 | 69% | -6% | 3% | 3.0 |
| handloom weaving | 6 | 103% | -10% | 0% | 2.6 |
| charcoal burning | 11 | 35% | -8% | 0% | 3.3 |
| millwright | 775 | 1725% | 258% | 0% | 6.1 |
| forged rails | 733 | 1533% | 71% | 0% | 6.4 |

**Are the high expected returns real?** For deep markets and capital-heavy
works, roughly: textile mills realise most of what they promise. For the rest,
no. The expected return is a snapshot: last year's price, 85 per cent
utilisation, today's size. It does not see what the works' own output, and the
competitors its profits attract, will do to the price, and it does not see the
works shrinking. In 1753 near the player's home in seed 7 the handloom offered
104 per cent and the smithy 109 per cent, with small market shares (0.13 and
0.02) but prices 2.7 and 2.9 times their base; those two methods realised -10
and 5 per cent. Millwrights and rail forges promise more than 1,000 per cent
because machinery and rails are asked for (by mines with pumps and wagonways)
long before anyone makes them, so their price sits at many times its base
(it can climb to 40 times); one small works supplies the whole demand, and
then the price collapses.

**Why do player works die young?** Not because they are the player's: in a
run without families, anonymous works die just as young (the median life of
works founded after 1750 that closed is 11 years; only 27 per cent of the
smithies and shops founded after 1750 still stand in 1900, and no millwright
or rail forge does, while coke furnaces, textile mills, brickworks and engine
works live long). The bots' works close at a median age of 10. The causes:

1. Small works are dear per unit. Labor per unit grows with the square root of
   full size over actual size, so a works built at 40 per cent of full size
   that does not grow loses its margin as wages rise or prices fall. Its
   utilisation then drops; idle works shrink 15 per cent a year on top of 4
   per cent depreciation, and close after three loss years, when their cash
   runs out, or when they have shrunk away.
2. Thin markets, as above: the first supplier of a scarce good is paid the
   scarcity price until its output piles up, then the price falls by up to 12
   per cent a month.
3. A family backs its own works (losses in their first three years, and
   growth) only from its own cash. A passive family spends 2 per cent of its
   cash a month and runs dry within a few decades, so its works can be neither
   rescued nor grown. The viewer demo's smithy had all three against it.

**What changed in the tuning.** AI families now build only where the works'
output is at most a quarter of the town's demand and the price at most three
times its base (`family_max_share`, `family_max_price_ratio`), and industrial
families put what their works do not take into a little land. The rivals' news
is cut to their big moves. The player's start, the land price and the spending
rules are unchanged: with them a sensible player can climb to the top by 1900,
the passive one falls to the bottom, and the naive ones do badly, so neither
the starting money nor the land price is what stands in the way.

**Things the design should know.**

- **Thin-market harvesting.** Merchants pay the gate price for everything a
  works makes whether or not anyone buys it (as in the Python: merchants
  absorb the unsold output and the price falls only gradually). Rail forges
  realised 71 per cent a year in the median although every one of them died.
  A player who builds cheap works in thin markets and lets them die harvests
  the merchants' cash; the AI families no longer do. Capping what merchants
  pay for output beyond what the town asked for would close it.
- **Hostile buyouts.** `buy_land` takes any family's land at `land_price`, and
  the owner cannot refuse. A player can strip a rival of its estates; the rival
  gets cash, which it then spends at 2 per cent a month. Consent or a premium
  is probably wanted.
- **Land ahead of the railway.** Land is priced on last year's rent, with no
  expectation in it. Buying where a line will run is the think-ahead play the
  design wants, but it is cheap.
- **Spending is not the player's choice.** Every family spends 60 per cent of
  its income and 2 per cent of its cash a month. It is the biggest drag on the
  player and could become a lever (a frugal or a lavish house).
- **Pace.** The prudent player is fourth to sixth until 1850 and climbs only in
  the last fifty years. If a game should open mid-table, start the player like
  a landed family (an estate worth 1 to 2 per cent of all rent) instead of with
  33k in cash; that is a proposal, not done here.

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
- The AI families' rules are fixed; they do not react to the player.
