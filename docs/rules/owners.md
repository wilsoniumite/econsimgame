# Who is paid for what they put up

Not in the Python. `crates/isostate-core/src/owners.rs`, switched by
`owners.enabled`: off under the port's rules, on under the game's. It is the
fourth stage of the run to 2050, and the first step of a rule for who owns
the works. **This step pays the landlords for the money they put into works;
it does nothing else.** There is still one kind of household, there is no
savers' fund, and the public holds no share of anything: those two holders
are kept in each works' shares at nothing, for later steps. The households
are a little poorer for it from the first month (3 per cent fewer people in
1900), and every fact of the standing check stays inside the spread between
islands.

## The standing check

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --set owners.enabled=false,true --standing --out DIR
```

```text
standing check: 12 of 12 inside (0 moved, 0 OUTSIDE, 0 without a value): owners.enabled=true against owners.enabled=false

                                  owners.enabled=false                        owners.enabled=true
the twelve
  pop_1800                                     208,731  [194,136 .. 220,504]              202,320  inside
  pop_1850                                     380,573  [349,408 .. 402,862]              364,624  inside
  pop_1900                                     692,474  [635,954 .. 744,110]              673,962  inside
  real_wage_chained_1850_vs_1750                  3.16  [2.62 .. 3.73]                       3.00  inside
  real_wage_chained_1900_vs_1750                  5.91  [4.89 .. 6.51]                       6.03  inside
  towns_1900                                      19.4  [15 .. 24]                           19.4  inside
  urban_share_1900                               0.458  [0.427 .. 0.469]                    0.465  inside
  rail_km_1900                                   2,584  [2,386 .. 2,738]                    2,482  inside
  rail_first_year                                 1827  [1816 .. 1842]                       1832  inside
  income_share_work_1850                         0.662  [0.619 .. 0.715]                    0.660  inside
  income_share_work_1900                         0.669  [0.653 .. 0.697]                    0.670  inside
  income_share_land_1900                         0.164  [0.154 .. 0.172]                    0.165  inside
also
  last_town_founded                               1888  [1871 .. 1896]                       1892  inside
```

All twelve are inside. People are the ones that lean: lower in every year
by 3 to 4 per cent, because the households, whose income the births rule
reads, are paid less of the works' dividends.

## What it does

Under the port a works no family owns is paid for out of its town's pool:
the households offer `invest_frac_hh` of their cash and the landlords
`invest_frac_lord` of theirs, and each pays in proportion to what it
offered (`firms::draw_pool`). Yet every penny of the works' dividends, and
its cash when it closes, went to the town's households. The landlords put
up money and were never paid for it.

With the switch on each such works keeps its **shares**: the parts of it
held by its town's households, its landlords, the savers' fund and the
public. The last two are nothing in this step.

**Paying in.** Wherever money is drawn from the pool for a works, the
shares move. With `B` the works' book before the money comes in (its
capacity, built and on order, at the year's prices, `families::works_value`,
plus its cash above nothing) and `x_h` what holder `h` paid,

    share_h = (share_h * B + x_h) / (B + sum of x)

This is done where a new works is founded (`firms::entry`: its shares are
what each paid over the whole); where a standing works is backed by its
town's savers (`firms::back_firm`, when it is short of cash in its first
years or grows); and where a works grows by a sister works next door
(`firms::spawn_sister`, `works::extend_sister`), whose money is part the
parent's cash and part the pool: the part from the parent's cash is held as
the parent is, and the parent's own shares do not move.

**Being paid.** A works' dividend is split by its shares: the households'
part goes to the households as before, and the landlords' part into the
landlords' income with the month's rents. A works that closes pays out its
cash the same way, and so does the money a sister works hands back when it
cannot be founded. A works changes town only with its district: its shares
stay as they are and are paid to the purses of the town it is in.

**Shares never set.** A works whose shares are all nothing (every works of
the warm-up, which nobody paid for) takes its town's pool ratio, what the
households and the landlords would offer, the first time its shares are
read: at the first month's dividends. A town whose pool is empty gives all
to the households, as the port does.

**Families.** A family's works pay the family, as they always have, and
their shares are never read. When a family winds a works down and sells it
to the town's savers (`families::wind_down`), the works is then held as the
households and the landlords paid for it. Families are still given their
first works free at the start.

**Railways.** A railway's subscription earns no share: a railway pays no
fare, and the landlords should not be paid for it out of the mills.

**The books** count the landlords' part as dividends, not as rent
(`div_landlords`, [accounts.md](accounts.md)), and the month's check of the
rents leaves it out. Every flow is a transfer between purses
`econ::money_total` counts, and the monthly money check holds.

With the switch off every one of these places takes its old branch, so the
port and the game without it are the same runs to the byte.

## Its numbers

One switch, `owners.enabled` (true in the game). Nothing to tune: what
each holder is paid follows from what each put in.

## The runs that show it

All under the game's rules with six families.

**1. Nothing moves with the switch off.** The port's golden test passes,
and the game's passes on its four hashes from before the rule with the
switch off in `game_params()`. A sweep of islands 1 to 4 to 1900 with `--set
owners.enabled=false`, set beside the same sweep made before this stage
(`--compare`): of the 819 facts the two share, the seconds aside, 0 differ.

**2. The standing check**, above: 12 of 12.

**3. What it did.** Islands 1 to 8, the means:

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| works' book held by families | 0.068 | 0.370 | 0.452 | 0.470 |
| by landlords | 0.729 | 0.391 | 0.316 | 0.298 |
| by households | 0.203 | 0.239 | 0.233 | 0.232 |
| landlords' share of all works' dividends | 0.663 | 0.463 | 0.369 | 0.396 |
| works' dividends over the landlords' whole income | 0.186 | 0.219 | 0.237 | 0.242 |
| households' baskets a head, rule off / on | 1.56 / 1.46 | 3.38 / 3.28 | 5.94 / 5.30 | 10.6 / 10.2 |
| works standing, rule off / on | | 437 / 438 | 843 / 859 | 1,107 / 1,160 |

Works are founded at the pace they were: 3,733 over the run with the rule
on, 3,660 without it. The landlords hold three quarters of the works of
1750 because they hold most of the money then, and so would have paid for
most of them; as works are founded out of the households' savings too their
share falls, and the families' rises as their branches build. A fifth to a
quarter of the landlords' income now comes from the works. The largest gap
from one of any works' shares, over every island's run, is 1e-15
(`owners_stake_worst`).

**4. Island 7 to 2050** (`--seeds 7 --end-year 2050`, both ways):

| | 1950 | 2000 | 2050 |
|---|---|---|---|
| works' book: families / landlords / households | 0.460 / 0.304 / 0.237 | 0.410 / 0.327 / 0.262 | 0.266 / 0.445 / 0.289 |
| landlords' share of works' dividends | 0.341 | 0.379 | 0.386 |
| works' dividends over landlords' income | 0.205 | 0.355 | 0.352 |
| baskets a head, rule off / on | 9.33 / 9.02 | 7.16 / 7.24 | 3.20 / 2.99 |
| work's share of income, gross, off / on | 0.646 / 0.645 | 0.598 / 0.611 | 0.585 / 0.581 |
| work's share, net of wear, off / on | 0.669 / 0.667 | 0.657 / 0.657 | 0.606 / 0.603 |

After 2000 the families' share of the works falls and the landlords' rises
to nearly half, as the works founded in the last half-century are paid for
out of the pool and not by families (why the families build less then has
not been traced). Work's share hardly moves either way: this step moves who is paid
the owners' part, not how large it is.

```bash
cargo run --release -p isostate-core --example owners -- --seed 7 --end-year 2050
```

prints the same for one island, year by year.

## What worries me

- The landlords' three quarters of 1750 is the pool ratio of the warm-up's
  first month. It is what the rule says, but it is an accident of how much
  cash each purse was given at the start, not a history.
- The households' baskets a head fall by a tenth in 1750 at once, and the
  people follow. The standing check holds, but a later stage that also
  moves the people may find less room.
- A works' book is its capacity at today's prices, not what it earns, so a
  holder who pays into a works that earns well is given the same share as
  one who pays into one that earns badly. That is the brief's rule for this
  step; a price for shares belongs with the savers' fund.
