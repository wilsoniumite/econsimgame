# tr-rent: landlords, rent and empty shelves

A trace with the books of account (piece 12 of `docs/backlog/found-along-the-way.md`), on branch `night/o-trace-rent`, cut from `main` at 295a379 on 12 November 2026, and gone over by a checker on `night/o-trace-rent-check` the same day: section 6 says what the check found and what it changed. Four things that were seen and never explained: landlords millions in debt for decades (C93), whether a family's share of a town's house rent can drive the town's remainder below nothing (L206), how far `harvest::average_rent` is out when half the harvest is foreseen (C66), and the old towns of island 6 whose shelves emptied between 1796 and 1814 with no bad weather (R45).

Everything is measured under the game's rules with six families unless it says otherwise, on seeds 1 to 16 for (a), (b) and (d) and 1 to 8 for (c), and found by switching the suspected mechanism off or on and seeing whether the effect goes. The probe is `crates/isostate-core/examples/trace_rent.rs`, which steps each island a month at a time, reads the books (`docs/rules/accounts.md`) and the state around each month and each year's end, and for (c) plays every year three times. No rule and no fact was changed: nothing under `crates/isostate-core/src` is touched, and a sweep of the branch against today's main says 0 of its facts differ (section 6). Two mends were tried in the worktree to see what they would move, and are described in section 5; neither is committed. Raw output is in `.claude/night/reports/opus/trace-rent/`.

## In short

1. **Landlords in debt for decades are the port's runaway wage, and the game no longer has them.** Under the port, on sixteen islands, a town's landlords end a year below nothing in 61 spells, 284 town-years in all, the longest 43 years and the deepest 4.3 million. Every long spell that is not a new town's inheritance begins in a year when one town's wage runs away (to the ceiling of 500 in the five longest, a mean of 133 to 276 for the year where the island's middle town pays 4 to 6), so every sector's rent is below nothing at once and the landlords pay the wage bill out of a purse they do not have. Railways, canals, works and land sales play no part. Under the game's rules there is one spell in sixteen islands, a single year in a new town that sank a colliery. What took the rest away is `people` (households go with their cells): switched off in the game, 154 spells come back, the longest 146 years; switched on over the port, none are left. Nobody charges interest, but the debt is felt: landlords below nothing buy nothing, back no works and subscribe to no line until their rents have paid it off, and a town founded out of their district takes a share of it.
2. **A family's share of a town's house rent cannot drive the remainder below nothing**, beyond rounding. On sixteen islands every town's shares add up to one to within 9e-16, the cells shared among are always the cells the bill was made from, and in the town-years where families own all of a town's houses the remainder is nothing to the sixteenth figure. Were it ever below nothing, the town's landlords would pay the difference to the families out of their rents; money would still be conserved.
3. **`average_rent` is out by about twice the harvest's shortfall, and the wrong way.** After a harvest more than 6 per cent short it puts the island's land at 13.5 per cent over what an average year really pays; after one more than 6 per cent good, 12 per cent under. The rent as paid is within 2 per cent in both. The error has two parts of about the same size. One is C66's: with half the weather foreseen, fewer hands are hired for a thin crop, and the wages saved keep the rent up (hired as for an average year, the rent paid after the worst harvests is 8 per cent short, not 2); that is 3 to 6 points. The other is the price: dearer grain makes up about half of what the weather took, while the correction adds back all of the lost crop at that dear price; that is 3.5 to 7.5 points. The rent as paid comes out near an average year's because the two happen to cancel at `foreseen` 0.5, not because the price makes up the whole loss. It moves the price families pay for land by 5 to 11 per cent in those years and a family's net worth, which is the game's score, with it. Over a run it nearly cancels.
4. **Island 6's empty shelves were empty purses, and the game's railways mended them.** Run again with the mechanisms merged since switched off, the hunger of 1803 to 1806 is there in old towns of that island, "0% of the food they need" and all. The grain was on the shelves (three to seven months of it, the price falling). Each time, a line of the town's, built under the port's rule, had opened the month before with its purse 117,000 to 249,000 below nothing, and the port hands what is left in a line's purse to its town's households (`rail.rs`, `progress`): their cash went below nothing, and a household with no cash buys no bread. With the game's railways on (a line never owes) those hungers go. What remains in today's game is nine town-years on sixteen islands, eight of them in the first two months of a year (six in January) in a town whose people grew two and a half to nearly five times at the year's end because it took a district: its people came, and the grain in the old town's store did not. Letting the stock go with the people (tried, not committed) leaves one, and takes the island's emigrants from 114 a run to 8. On eight islands the trace had not used (seeds 17 to 24) the same January kind is two of five; the other three are shortages in spring, before the harvest, that this trace did not follow.

## 1. Landlords millions in debt (C93)

### The question

The railways review saw it at Walbourne on seed 12 with families (C93: "anything else that draws on `lord_cash` will meet it"); `docs/rules/accounts.md` found it under the port in four seeds of eight and in none under the game's rules. Which towns' landlords, when, how deep against their rents, what put them there, what brings them back, and is it felt?

### What the code says

A town's landlords hold `State::lord_cash`. It falls when they spend (on goods, through `econ::step_month`), when the town's savers put money into works (`firms::draw_pool`), when lines and canals take subscriptions or make calls (`ways`, `rail`), and when families take from savers. Every one of those takes a share of `lord_cash.max(0)`, so none of them can push the purse below nothing. It rises with their rents, credited each month as `lord_income`, and with land the families buy from them. Rent on the land is the residual of the produce after wages, carting and machines, and is below nothing whenever the wage bill passes the produce; under `mines` a working's royalty pays its sinkers and starts below nothing. So the only road below nothing is a month whose rents are negative. Once there, a landlord's spending budget is `lord_cash.max(0)` (`econ.rs` 327), which is nothing; nothing is charged on the debt; and the rents go into the purse until it is above nothing again.

### What was measured

`trace_rent --part a` records, for every town and year, the landlords' cash at the year's end and its least in any month, their rents month by month as credited, and from the books their rents by kind (each land sector, works' sites, town houses, the families' part), their spending, the town's wage (the year's mean, and the most it stood at in a month), what the lines and canals took or returned within the months, and what the year's end did (subscriptions to new lines and canals, land sold to families, and the rest). It prints each spell of year-ends below nothing.

    cargo run --release -p isostate-core --example trace_rent -- --seeds 1-16 --part a,b
    cargo run --release -p isostate-core --example trace_rent -- --seeds 1-16 --part a --port
    cargo run --release -p isostate-core --example trace_rent -- --seeds 1-16 --part a --set '{"people":{"enabled":false}}'
    cargo run --release -p isostate-core --example trace_rent -- --seeds 1-16 --part a --port --set '{"people":{"enabled":true}}'
    cargo run --release -p isostate-core --example trace_rent -- --seeds 1-16 --part a --set '{"mines":{"enabled":false}}'
    cargo run --release -p isostate-core --example trace_rent -- --seeds 1 --part a --follow 1:Stanside:1770
    cargo run --release -p isostate-core --example trace_rent -- --seeds 2 --part a --port --follow 2:Halhope:1844

| sixteen islands, six families | spells | town-years below nothing at a year's end | the longest | the deepest | begun by rents below nothing | came with a town's founding |
|---|---|---|---|---|---|---|
| the game's rules | 1 | 1 | 1 | 7,005 | 1 | 0 |
| the game's rules, `mines` off | 1 | 1 | 1 | 3,485 | 1 | 0 |
| the game's rules, `people` off | 154 | 951 | 146 | 27.1 M | 133 | 20 |
| the port | 61 | 284 | 43 | 4.32 M | 53 | 8 |
| the port, `people` on | 0 | 0 | | | | |

### The answer

**Which towns, when, how deep.** Under the port, every island but one: Welford (seed 3) from 1753 to 1795, 3.6 M deep at a year's end (7.1 M within 1753: half of it went at the year's end with the people of Ormgate, founded out of its district, whose landlords carried it until 1771); Halhope (seed 2) from 1844 to 1862, 4.3 M; Wilhope (seed 6) from 1833 to 1862, 3.9 M; Greyfell (seed 6) 1817 to 1840, 2.2 M. The deepest four are 90 to 220 years of the rents the same landlords drew in the three years before (Halhope's took 20,363 a year and went 4,319,164 below nothing in one year). Most spells are short: of 61, 43 last a year or two, and 11 last more than five.

**What put them there.** In the year each spell began, every land sector's rent was below nothing at once, in a town whose wage ran far above the island's: Halhope's averaged 276 in 1844 and touched the bisection's ceiling of 500, against 5.7 in the island's middle town; Welford's 241 in 1753 against 4.7. Of the eleven spells longer than five years, nine began this way (five touched 500, with means for the year of 133 to 276; four touched 116 to 274, with means of 29 to 68) and two came with a town's founding (Ormgate, 19 years; Penfell, 24). Followed month by month (`--follow 2:Halhope:1844`), Halhope's wage was 8.9 in January and doubled every month, 17.8, 35.6, 71, 142, 284, to 500 in July; the landlords' rents went from -6,778 in January to -814,098 in July, they spent 1,115 in January and nothing after, and nothing else touched their purse. That is the runaway `accounts.md` described (a town whose farms and pits want more hands than it has left after the year's end; the wage doubles each month until enough are shed, and the land pays). Of the 61 spells under the port, 53 began this way, with the wage at the most 9 times the middle town's in the median spell and 111 times at the worst; the other 8 were towns founded out of a district whose landlords were already below nothing, which took a share of the debt with their people (`people::resettle_new_towns` moves a share of `lord_cash`, debt and all). Railway and canal subscriptions, calls and repayments, works' losses and land sales played no part: in the year each spell began they came to nothing or a few thousand.

**Mines' leases.** The one spell under the game's rules is Stanside (seed 1), founded in 1769: in 1770 a colliery was sunk in its district and its royalty, out of which the sinkers are paid, came to -24,145 for the year against 22,564 of other rents, and the landlords ended the year 7,005 below nothing. Followed month by month (`--follow 1:Stanside:1770`), their purse began the year at 15,184; the rents credited were 10,974 in January and fell every month to -5,065 in August as the sinking went on; they spent 21,117 between January and September, budgeting on the good months behind them, and were below nothing from September, after which they spent nothing. January's purse, less 1,581 of rents and 21,117 of spending, is -7,514 in December, and the year's end added 509. Their spending is the books' `spend_landlords` to the shilling: nothing else drew on the purse. They were above nothing again by the end of 1771. Six more town-years dip below nothing within a year and climb out by December, all in towns two years old or less and none by more than 6,317: four on a working's royalty (Ashbank, seed 1, 1834, coal and stone; Larkmoor, seed 12, 1838, stone; Garside, seed 7, 1764, coal; Brackbourne, seed 7, 1840, clay), one on a new town's farms in its first year (Brambank, seed 3, 1751), and one on a single month's rents that the rest of the year made up (Etherside, seed 3, 1815, 322 below). (The probe's count of town-years that dip, three on seed 1, also counts Stanside's 1770 and 1771, which began below nothing.) With `mines` off Stanside's spell goes, and another new town's coal rent puts one other town a year below nothing (Haskbank, seed 16, 1767: 3,485).

**Why the game has none.** `people` sends households with their cells when a district changes market, so a town never wakes in January with land that wants more hands than it has. Switched off in the game, the runaway comes back and with it 154 spells, worse than the port's (Cranbourne, seed 6, 1755: an iron mine's rent of -21.9 M for the year; Wolfedge, seed 6, took 27 M of a neighbour's debt at its founding in 1755 and still owed it in 1900). Switched on over the port, there is not one. It also stops debt travelling: `peopling::move_with_cells` moves a share of each purse's `max(0)`, so a town's debt stays with its landlords.

**What brings them back.** Their rents and nothing else. While below nothing they spend nothing at all (Halhope's landlords spent 18,887 a year before and 0 in each of the eighteen years after), and every shilling of rent goes to the purse. Their rents are often larger after the runaway than before it (Halhope's 237,844 a year in the later years of the spell against 20,363 before; Welford's 74,901 against 19,853), because the town has shed hands and its land is worked by fewer at a lower wage. Welford's took 43 years; Halhope's 19. A debt that a new town takes with its founding is paid from that town's rents, which can be small: Wolfedge's never was.

**Is it felt?** It is not a number nobody reads, but the economy feels it only through what the landlords then do not do, never through a charge.

- Nobody lends it and nobody charges interest. Money is conserved: the households of the town were paid the runaway's wages, as much as four million more than the land made (Halhope), and the landlords' purse went below nothing by that much. In effect the landlords issued money to their tenants and take it back out of rent over twenty or forty years.
- For those years the landlords buy nothing: no goods, no houses, no services. Their demand in the town is gone.
- The town's savers' pool, which backs new works and enlargements (`firms::draw_pool`), has no landlords' part.
- A line or canal that calls on them gets nothing; under the game's rules its call falls on the town's households instead (`ways.rs` 1098, the mend the railways review made for C93).
- Under the port, a town founded out of the district takes part of the debt and starts life with landlords who will spend nothing for years.

### How sure, and what would change my mind

Sure of the cause under the port and of `people` as what removes it: the switch takes 154 spells to none in both directions on sixteen islands. Less sure that the game is free of it for good. A runaway needs a town whose hands fall short of what its land wants; `people` removes the commonest way that happens (a district redrawn at the year's end), and anything new that takes hands from a town all at once (navvies, sinkers, a new kind of labour) could bring it back. Seeing a town's wage pass a hundred in a sweep of the game's rules, or a spell of more than a year under them, would change my mind. I did not find Walbourne: the railways review saw it on a build of that day, and seed 12 under today's port has one spell of a single year (Wolfside, 1751).

### For the pass over all the rules

Nothing to do for the game. Two things a designer should know. The rent of land has no floor anywhere: a landlord, gentry or family, pays wages out of his own purse whenever the produce does not cover them, and a purse can go below nothing with no limit and no charge. That is the port's rule and the game keeps it (it is also what makes a failed harvest fall on the landlords, which `harvest.md` meant). And the mines' sinkers are paid out of a royalty that a young town's landlords may not be able to carry; if mines are ever sunk faster in new towns this is where it will show first. If the overdraft is ever to cost something, the place is the monthly step where `lord_income` is credited: interest on a negative `lord_cash` paid to whoever holds the money would make the debt a transfer with a price, and that would move every number where it occurs.

## 2. A family's share of a town's house rent (L206)

### The question

`families::take_urban_rents` gives each town cell its share of the town's house rent, `urban_pay[t] * cell_urban_rent[c] / urban_rent[t]`, pays the families' cells their part and takes it off the town's remainder, which goes to its landlords (or the common fund). Can the families' part pass the whole, leaving the remainder below nothing?

### What was measured

At every year's end, after town ground has been laid out for the coming year (the shares do not change within a year: neither the ground, nor the market areas, nor the owners), for every town: the sum of the shares over the cells the month's sharing will count for it (`urban[c]` and `catch[c] == t`), the families' part of it, and whether the town's own ground list (`town_ground[t]`, from whose cells `urban_rent[t]` is added up in `towns::update_urban`) holds exactly those cells. `trace_rent --seeds 1-16 --part a,b`.

### The answer

No, beyond rounding. On sixteen islands (1,583 to 2,421 town-years each) the shares add up to one to within 9e-16 in every town-year, and the ground list and the cells shared among never disagree. Families own some of a town's house rent in a fifth to nearly half of town-years, 6 to 20 per cent of it in the mean, and all of it in some town-years on eleven islands of sixteen (Carford on seed 2 in 1818, Cragside on seed 10 in 1879). Then the remainder is `urban_pay * (1 - 1)`: nothing, give or take one part in 1e16.

It holds by construction. The ground a town keeps is sorted to the town whose market area each cell is in at the start of each year's layout (`towns::update_urban`, the first loop), the bill is added up over that list, and the market areas are worked out before the layout and not again until the next year's end. Under the port, `land::update_urban` picks the cells from the market area itself.

If it ever did go below nothing, `econ::pay_rent` would credit the town's landlords a negative rent: they would pay the families the difference out of their purse, or the common fund would under the commons. Money would be conserved and the books would show it as `rent_town` against `rent_families`.

### How sure, and what would change my mind

Sure. Something that changed a cell's market area or a town's ground in the middle of a year, with the year's bill left as it was, would break it; nothing does today, including an order given in a live game.

### For the pass over all the rules

Nothing. The finding can be closed.

## 3. How far `average_rent` is out (C66)

### The question

`harvest.md` prices land on the rent of an average year: `average_rent` adds to the rent a cell paid the crop the weather took from it, at that year's price less carting. It says this is "exact at 0 and near enough at a half", since with half the weather foreseen the hands hired were themselves fewer, which it does not undo. By how much, what reads the figure, and what does a wrong figure move?

### What was measured

Every year of eight islands is played three times from the same January (`trace_rent --seeds 1-8 --part c`, three times as long as a run):

- as it was, giving the figure the families price land on (`est`: the rent each cell paid plus `harvest_rent_gap`);
- again with the same weather but `harvest.foreseen` at 0, so that the hands are hired as for an average year (`est0`, worked out the same way);
- again with no weather at all, every `harvest_mult` 1 (`avg`: the rent each cell would really have paid in an average year, at an average year's prices).

Island land outside the towns, added up over cells; "cell by cell" is the sum over cells of the gap to `avg`, over the sum of `avg`. 1,208 island-years, pooled by the country's harvest; the probe prints this table after the islands one by one.

| the country's harvest | years | rent as paid, over `avg` | the same, hands hired as for an average year | `average_rent` [least and greatest island] | `average_rent`, hands hired as for an average year | cell by cell: paid | `average_rent` | hired as average | land the families bought those years | its price over the right one |
|---|---|---|---|---|---|---|---|---|---|---|
| worse than 0.94 | 143 | -0.021 | -0.082 | +0.135 [+0.116 .. +0.158] | +0.075 | 0.057 | 0.143 | 0.084 | 11.6 M | +0.094 |
| 0.94 to 0.97 | 213 | -0.011 | -0.043 | +0.069 [+0.059 .. +0.076] | +0.038 | 0.051 | 0.077 | 0.045 | 15.3 M | +0.048 |
| 0.97 to 1.03 | 522 | -0.001 | -0.002 | +0.001 [-0.006 .. +0.010] | +0.001 | 0.046 | 0.040 | 0.021 | 40.8 M | -0.002 |
| 1.03 to 1.06 | 168 | +0.008 | +0.035 | -0.062 [-0.067 .. -0.054] | -0.035 | 0.047 | 0.068 | 0.039 | 12.6 M | -0.052 |
| better than 1.06 | 162 | +0.015 | +0.067 | -0.119 [-0.127 .. -0.109] | -0.066 | 0.052 | 0.125 | 0.073 | 12.3 M | -0.106 |

(The last column is the price the families paid, 25 years of `average_rent` never counted below the floor of `land_rent_floor` town wages, over the price 25 years of `avg` would have set under the same floor. The probe works every purchase's price out again and finds `families::land_price` in all 5,396; 36 thousand of the 92.6 M was bought at the floor. A year whose harvest is within a ten-thousandth of a bin's edge can fall on either side of it, which is why a year moved between the second and third rows after the check.)

### The answer

**It is out by about twice the harvest's shortfall, high after a bad year and low after a good one, and the rent as paid would have been nearer.** After a harvest more than 6 per cent short (one year in eight), `average_rent` puts the island's land 13.5 per cent above what an average year pays, on every island (11.6 to 15.8 per cent). The rent actually paid was 2 per cent under. In ordinary years the two are as good as each other on the island's total, and cell by cell `average_rent` does better (0.040 against 0.046), because a district's own weather still varies when the country's does not.

**The part C66 asked about is a little under half of it, and the price is the rest.** Hiring as for an average year takes the error from +13.5 to +7.5 per cent after the worst years: fewer hands for a thin crop is 3 to 6 points across the rows. The second column says how: with the hands hired as for an average year the rent as paid after the worst harvests is 8.2 per cent short, and with half the weather foreseen only 2.1, because the farms hired fewer and paid fewer wages. The rest of the error is the price. With the hands hired as for an average year, the crop the weather took, at that year's price, comes to 15.7 per cent of an average year's rent (7.5 + 8.2), but the rent lost only 8.2: grain sold dearer (a crop 10 per cent short sells 9 per cent dearer, `harvest.md`) and made up about half of it. `average_rent` adds back the whole lost crop at the dear price, as though the landlord had lost all of it. So it is not exact at `foreseen` 0 either: there it is out by the price part alone, 7.5 points after the worst years. And the rent as paid is near an average year's at `foreseen` 0.5 because the wages saved and the dearer grain together happen to make up nearly all of the loss; at `foreseen` 0 it would be 8 per cent short, about as far out as `average_rent`, on the other side.

**What reads it.** Only `families::land_price`, which is 25 years of it (never less than `land_rent_floor` town wages a month). That prices:

- every purchase of land by a family, from the gentry or from another family, by the player's order or an estates branch (`families::buy_at`); a dearer cell also uses more of the branch's spare cash, so fewer are bought;
- the sale of a winding-down estate's poorest cell to the town's savers;
- a family's net worth (`families::net_worth`), which ranks the families in the viewer and is the game's score at the end, and the land in its report.

The families' choice of which cells to buy reads the rent as paid (`cell_rent_year`), not this.

**What a wrong figure moves.** Families paid 9.4 per cent too much for the land they bought after the worst harvests (11.6 M of land on eight islands, so about 1.1 M too much, to the gentry or other families) and 10.6 per cent too little after the best. Over a run it nearly cancels: the error summed over every purchase is -0.2 per cent of 92.6 M, and its absolute size, added up island-year by island-year, 5.2 per cent. A family whose wealth is mostly land sees its worth rise by a tenth in the year after a bad harvest and fall in the year after a good one, which can move it a place in the ranking for a year. The economy itself hardly feels it (section 5).

### How sure, and what would change my mind

Sure of the sizes and the sign: every island lands in the same narrow range, and the comparison is like for like (the same island from the same January, one thing changed). The "right" figure here is the rent of an average year at an average year's prices, which is what land is worth if the weather is noise. A designer who wanted land priced on this year's prices, as if the scarcity would last, would want `est` and not `avg`; nothing in the game behaves as if it would last.

### For the pass over all the rules

Drop the correction, or keep it for the district's own weather only, as long as `harvest.foreseen` stays at 0.5. The rent as paid is within 2 per cent of an average year's after any harvest, because the wages saved by hiring fewer hands for a thin crop and the dearer grain together make up nearly all of what the weather took; `average_rent` is out by 6 to 14 per cent. The right size of the correction hangs on two numbers a designer may move. Were `foreseen` set to 0, the rent as paid would be 8 per cent short after the worst harvests and a correction of about half the lost crop at its price would be right; at 0.5 the right weight is about an eighth (2.1 of 15.6), which is near enough nothing. If the harvest's price answer is ever made stronger (`harvest.md` says thirty per cent dearer for ten short is the historical figure), the rent as paid will rise in a bad year and a correction will be wanted again, but with the opposite sign. Whoever changes `foreseen` or the grain price's answer should look here. (This paragraph was the worker's, mended by the check: the worker read the near-cancelling as the price alone.)

## 4. The old towns whose shelves emptied (R45)

### The question

`harvest.md`, "In company": "In seed 6 the hunger comes between 1796 and 1814, all but once in towns that have stood for twenty years or more, and in three of them households get 0% of the food they need in the worst month ... some town's shelves are emptying for a reason that is not the weather." What is short, who would have made or carted it, and why did they not?

### What was measured

`trace_rent --part d` names every town-year in which a town got under 0.82 of its food at the worst month (`people::growth`'s line), with the twelve months' share of food got, the months round the worst (people, grain in the merchants' store in months of need, its price, the share of need got, the households' cash a head), the books of its food for the year and the year before (made, received, sent, bought, asked, unmet, stock, and what came or went with districts at the year's end), and any line or canal of the town's that opened in the three months before, with what was in its purse.

    cargo run --release -p isostate-core --example trace_rent -- --seeds 1-16 --part d
    cargo run --release -p isostate-core --example trace_rent -- --seeds 1-8 --part d --set '{"works":{"enabled":false},"railways":{"enabled":false},"carting":{"enabled":false},"people":{"enabled":false},"towns":{"enabled":false},"mines":{"enabled":false},"canals":{"enabled":false}}'
    cargo run --release -p isostate-core --example trace_rent -- --seeds 1-8 --part d --set '{"works":{"enabled":false},"carting":{"enabled":false},"people":{"enabled":false},"towns":{"enabled":false},"mines":{"enabled":false},"canals":{"enabled":false}}'
    cargo run --release -p isostate-core --example trace_rent -- --seeds 1-8 --part d --port

The second line is the game's rules as they were when R45 was written (2448d12: `prices`, `engel`, `machines` and `harvest` over the port), approximately: today's code with every mechanism merged since switched off. The third is the same with the game's railways on.

### The answer

**On today's main island 6 has no hunger at all.** On sixteen islands there are nine town-years short of food, none on island 6 (section 4's last part).

**Under the rules of R45's day it is there, and it is not the shelves.** With the later mechanisms off, island 6 has hunger in old towns in 1803 (Kenwater, founded 1752, and Etherside, 1781), 1804 (Cragford, 1749), 1806 (Linbrook, 1749), 1807 (Greyfell) and 1814 (Mormoor), and in the first three the households get nothing for one to five months: the report's island, near enough. In every one of the three, the grain was there:

| | before | the month a line of the town's opened | the months with no food |
|---|---|---|---|
| Kenwater, 1803 | June: 3.6 months of grain in store, price 0.67, households' cash 15.9 a head | July: cash -24.1 a head at the month's end | August to December: 5.4 to 7.6 months in store, price falling from 0.64 to 0.50, food got 0.00; cash -18.1, -13.1, -8.1 a head by October |
| Etherside, 1803 | March: 3.4 months in store, cash 11.3 a head | April: cash -8.7 a head | May and June: 5.1 and 6.5 months in store, price 0.59 then 0.53, food got 0.00 |
| Cragford, 1804 | December: 3.2 months in store, cash 10.0 a head | January: cash -7.0 a head | February and March: 4.7 months in store, price 0.48 then 0.42, food got 0.00 |

(A household's budget for a month is set from its cash as the month begins, so June in Etherside and March in Cragford go without though the cash is back above nothing by their end.)

Nobody bought the grain. Unmet requests for the year are nothing, because nothing was asked: a household's budget is a share of its cash's `max(0)`, so households below nothing ask for nothing and pay no rent. What drove them below nothing was a line of the port's. In each case a line of the town's had opened in the month the cash fell, with its purse 168,092 (Kenwater), 154,606 (Etherside) and 249,432 (Cragford) below nothing, and Linbrook's in 1806 two lines with 117,046 and 18,264 owing. The port's lines buy their materials whether or not they have the money, call on the landlords for a tenth of their cash at most, and when the line opens hand what is left in the purse to the households of its town (`rail.rs`, `progress`: `st.hh_cash[pr.town] += pr.cash`). What is left is a debt, and the households take it: 37 a head in Kenwater, in one month, against cash of 16.

On eight islands under those rules there are 140 town-years short of food (the probe sorts them at the end of its output). 106 are a new town's first year (a separate thing, mended since by a312563 and `people`). Of the other 34, 9 followed a line of the town's that opened in debt, and those 9 hold all 8 in which households got under a twentieth of their food. The remaining 25 got between a tenth and four fifths of it, mostly in one January, and the worst of them (Brightedge, seed 3, 1846, 0.11; Sandedge, seed 4, 1850, 0.10) are the January shortage described below, in a town whose people grew thirteen to seventeen times over at the year's end.

**The cause, switched.** With the game's railways on (a line orders each month no more than its cash pays for, so it never owes; `railways.md`) and everything else as before, island 6's hungers of 1803 to 1806 go, and on eight islands no town-year short of food has households below nothing. Under the port itself the same thing happens: 132 town-years short of food on eight islands, of which 10 are near-total hungers with households below nothing; 8 of them come within three months of a line that opened in debt, and the other two (Mallowport and Brackedge, seed 1, 1828) had households below nothing since the autumn before, with 7 to 14 months of grain in store and its price falling to 0.22. So R45 was mended when the game's railways were merged, without anyone tracing it; it remains in the port, whose rule is kept as it was.

**Who would have made or carted the grain, and why they did not.** Nobody failed to make or cart it. The towns' farms made their usual crop (Kenwater 88,081 in 1803 against 79,923 the year before; Cragford 449,610 against 513,434), grain came in from other towns, and with no buyers the stock grew and the price fell for as long as the households had no cash. It ended when a month's wages had put their cash back above nothing.

**What remains today.** Nine town-years on sixteen islands, a single month each, the worst at 0.43 of need (Ryesea, seed 11, January 1840). None has households below nothing. Eight of the nine are in the first two months of a year (six in January, Ashbank on seed 1 in 1835 and Wyrside on seed 14 in 1770 in February), in a town whose people grew two and a half to nearly five times at the year's end because it took a district: Uldale (seed 1) from 3,257 to 13,637 people in 1848, Carford (seed 2) from 10,103 to 30,792 in 1827, Ryesea from 3,489 to 15,883, Holbrook (seed 15) from 2,168 to 8,127, Ashbank from 3,665 to 9,249. The households came with their cells, and so did their cash and last month's wages (`peopling::move_with_cells`), but the merchants' stock of grain moves with a district only when it goes to a new town (`if st.town_new[j]`). So the town with three or four times its people opens January with the grain its old people needed, 0.4 (Ryesea) to 2.0 (Ashbank) months of the new need; the shelves run short in the first month or the second, the price rises a little, other towns send grain, and within a month or two it is over, with a saw for a month or two as the merchants catch up. The ninth (Wildale, seed 8, 1845) is a year-old town in March.

**On eight more islands it is not the whole of it.** Run on seeds 17 to 24, which the trace had not used (`--seeds 17-24 --part a,b,d`), the game has five town-years short of food. Two are this kind, in February: Ulhope (seed 22, 1842), whose people went from 2,876 to 18,313 at the year's end, and Eastwater (seed 22, 1820), 2,059 to 4,429. The other three come in spring, before the harvest, with no jump in people: Ryefell (seed 23, May 1832, 0.23 of need), a three-year-old town whose own grain fell from 119,635 to 6,302 that year, so that its store ran down to a week's need in April; Langside (seed 18, April and May 1844, 0.58), a town founded in 1800 that sent 15,761 of grain away in a year it ran short; and Brightheath (seed 20, June 1759, 0.66), in its second year. This trace did not follow them; they are the next thing to look at, and the mend of section 5 is not expected to touch them.

### How sure, and what would change my mind

Sure of the cause of R45's near-total hungers: in every one the purse went below nothing in the month a line of the town's opened in debt, and switching to the game's railways removes them. I cannot run the build of that day, so "the same island" is an approximation: the mechanisms are off, but the code beneath them has moved on (the seeds give the same towns, founded in the same years, and the same dates of hunger). Run without families, as the sweeps of R45's day may have been (`--seeds 6 --families 0` with the same `--set`), island 6 comes nearer the report: households get nothing in Stonerigg in 1802 and 1803, Mormoor in 1804 and 1805, Cragford in 1805 and Ulstrand in 1805 (the report named Ulstrand), all old towns, four of them in the months after a line of the town's opened in debt and the other two with the households' cash below nothing already. Less sure of the January shortages today: the timing and the population jumps say what they say, and the mend in section 5 tests it on the sixteen islands where they were found; on eight others they are two of five.

### For the pass over all the rules

Two things. Under the port, a line that opens in debt charges it to its town's households, who then starve with full shelves; the game does not do this, and nothing needs changing unless the port's railways are ever used again under the game's rules. And in the game, the merchants' stock should go with a district to whichever town takes it, as the households, their cash and their wages do. Section 5 says what that would move: nearly all the hunger and nearly all the emigration. Whoever reads the emigration facts (R195 has emigration filed as famine news) should know that today, on the sixteen islands measured, they mostly count this January, not hard times.

## 5. Mends tried, not committed

Each was made in the worktree, built, swept on seeds 1 to 16 under the game's rules with six families, and taken out again with `git checkout`. The baseline is the same sweep of today's code (`.claude/night/reports/opus/trace-rent/base16`); the mends' sweeps are `mend-c` and `mend-d` beside it. Both mends change the run from the first year they act, so most facts differ by a little on most seeds; what matters is what leans one way on most seeds.

**`average_rent` returns the rent paid** (in `harvest.rs`, the body becomes `paid`). The sweep stars none of its facts. The ones that lean furthest: works founded in a run 1,940 to 1,870 (down on 13 seeds of 16), works closed 1,443 to 1,388 (12 of 16), closures told in the news 192 to 194 (up on 14), the canals' traffic in 1900 up a little (12 of 16). That is a run that has gone a slightly different way, which is what a changed price of land does once it reaches the families' purses and the towns' savers; I would not read more into it on sixteen islands. The sweep has no fact for the families' land, so what the mend does where it acts (the prices of land bought after a bad or good harvest, section 3) is not in it. A mend that kept the correction for a district's own weather and dropped it for the country's would keep the small gain in ordinary years (cell by cell 0.040 against 0.046); I did not try it.

**The merchants' stock goes with a district to any town** (in `peopling::move_with_cells`, the stock and the merchants' measure of their sales move with each share of a town's people that changes market, where today they move only to a new town; the merchants' cash still goes only to a new one). On sixteen islands the town-years short of food fall from 9 to 1 (Elmmoor, seed 7, May 1838, 0.69 of need: not a January). The years with a town short of food fall from 7 in all to 1. And emigrants fall from 113.5 a run to 8.3: emigration is down on 12 islands of 16 and up on none, and is nothing on 13 islands where it was nothing on 4. So most of the game's emigration today is people leaving a town in the January after it took a district, because its shelves held its old people's grain. People in 1900, the real wage and the rest do not move beyond the noise. The check tried the mend again with its two halves apart (`trace_rent --seeds 1-16 --part d` on a build with the halves behind a temporary switch, not committed), and both are needed. With the stock and the measure of sales both moving, the nine town-years short of food are one, as above, and on seeds 17 to 24 the five are none (the runs go differently from the first district that moves, so that does not say the spring shortages of section 4 had the same cause). With only the stock moving, four remain, none in January: Wildale (seed 9, March 1823), Wolfheath (seed 12, April 1777), Stonemoor (seed 15, March 1778) and Etherhope (seed 16, May 1837), three of them towns that took a district and its stock at the year's end (Wildale 11,417 of grain, Wolfheath 20,074, Etherhope 16,931) and ran short in spring once that was sold, because the merchants still ordered for the town's old trade. With only the measure of sales moving, six remain, three of them the January kind (Stratbank, Skeldale and Mardelf, their people three and a half to five times what they were in December). So the stock carries the town through January, and the measure of sales makes the merchants order for its new people after that. One thing to fix before it is made for real: the books' goods balance (`books_goods_worst`) goes from 4e-15 to 6e-11 on every island, still rounding, but it means the books' `moved` column does not yet hear of stock that goes to a town that is not new, and `accounts.md` says such a mechanism has to tell them.

## 6. What was checked

- `cargo test -p isostate-core --release`: 210 tests in 23 groups pass, the golden test among them.
- `isostate-core sweep --seeds 1-8 --rules game --families 6 --compare .claude/night/reports/merged-game-f6-l`: "of the 446 facts both have, the seconds aside, 0 differ". The branch adds an example and this page and nothing else.
- The viewer was neither built nor shot: nothing it uses has changed.
- The machine was shared with a dozen other helpers' builds and sweeps throughout, so no timing on this page means much beyond "about a minute" and "three times as long".

### What the check found and changed

Every command on the page was run again with the worker's own probe, and every count in sections 1, 2 and 4 and every figure of section 3's table came out as written. What did not:

- Section 3's table and section 4's sorting of hungers had been made by scripts kept outside the repository, so the page's commands did not print them. The probe now prints both. The table's last column was said to count only purchases priced on the rent and not the floor; the probe's filter counted all of them. It now works out every price as `families::land_price` does, finds all 5,396 again, and counts the error under the floor; at most a year moves between two rows and a third decimal changes.
- Section 3's account of why: the worker read the rent as paid sitting near an average year's as the price making up the loss. Adding the rent as paid with the hands hired for an average year shows it 8.2 per cent short after the worst harvests, so the price makes up about half and the wages saved by foreseeing the rest. The numbers stand; the explanation and the advice (which now hangs on `foreseen`) were mended.
- Section 1: eleven spells last more than five years, not nine; two of the long ones came with a founding and not a runaway, and of the runaways four never touched 500; Welford went 7.1 M below nothing within 1753, not 3.6; Stanside's other rents were 22,564, not 21,000; six fresh dips, not seven.
- Section 4: two of the nine hungers today are in February, not January, and the jumps in people and the months of grain at the turn of the year were wider than written. On seeds 17 to 24 the January kind is two of five.
- New checks, all agreeing with the page: Stanside's and Halhope's purses followed month by month (section 1); seeds 17 to 24 for (a) and (b), no spell and the shares off one by at most 1.2e-15; island 6 without families under R45's rules, which names Ulstrand as the report did; and the mend of section 5 taken apart.

`cargo test -p isostate-core --release` and the sweep of seeds 1 to 8 were run again after the check's changes and a merge of main; section 6's first two lines hold for the branch as it stands.

## 7. How to run it again

```bash
cargo run --release -p isostate-core --example trace_rent -- --seeds 1-16 --part a,b [--out DIR]
cargo run --release -p isostate-core --example trace_rent -- --seeds 1-16 --part a --port
cargo run --release -p isostate-core --example trace_rent -- --seeds 1-16 --part a --set '{"people":{"enabled":false}}'
cargo run --release -p isostate-core --example trace_rent -- --seeds 1-8 --part c [--out DIR]     # three times as long
cargo run --release -p isostate-core --example trace_rent -- --seeds 1-16 --part d [--from 1790 --to 1820]
cargo run --release -p isostate-core --example trace_rent -- --seeds 1 --part a --follow 1:Stanside:1770
```

`--out DIR` writes every town's year for (a) as `debt-seedN.csv` and every year of (c) as `rent-seedN.csv`. Part (c) ends with the pooled table of section 3 and part (d) with the sorting of section 4's hungers by what came before them. `--follow SEED:TOWN:YEAR` prints that town's landlords' purse month by month through that year. The probe takes about a minute for sixteen islands without (c), and three for eight with it.
