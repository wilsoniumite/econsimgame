# tr-loose-a: the merchants' debts, runaway wages, twin works, and a mill extended the year before it closed

Branch `night/o-trace-loose-a`, cut from `main` at 8bf4505 on 12 November 2026. One of the traces with the books of account (piece 12 of `docs/backlog/found-along-the-way.md`); it takes four things the traces and tables of 12 November measured and left untraced (`tr-money.md`, `dx-inspector.md`, `sens-mines.md`, `sens-works.md`, `news-read.md`). Nothing under `crates/isostate-core/src` was changed on the branch. The measuring code is `crates/isostate-core/examples/loose_ends.rs`:

```bash
cargo run --release -p isostate-core --example loose_ends -- MODE [--seeds 1-8] [--rules game|port]
    [--families 6] [--end-year Y] [--set JSON]...
    merchants [--decades]
    wages --seeds S --town NAME --from YEAR --to YEAR [--works GOOD]
    twins
    extend [--works NAME]
    market --seeds S --town NAME --from YEAR --to YEAR [--works GOOD]
```

It steps each island as `sim::run_year` does (twelve months, then the year's end) and reads the state between the months and on either side of the year's end; the books (`docs/rules/accounts.md`) say the rest. Everything is under the game's rules with six families on seeds 1 to 8 unless a line says otherwise. Raw output is under `.claude/night/reports/opus/trace-loose-a/` (`raw/` for the example, `trial-*` for the sweeps), with notes in `build.md` there. Five trial mends were tried in the worktree, each behind an environment variable so that one build ran both ways, and none was committed; each section says what it was and what it moved.

## In short

1. **The merchants' debts are the price rule's markup acting on services, which spoil.** Nobody lends: a purse may go below nothing and the money check counts it as negative money. On every island the merchants as a whole take in more than they pay out (147 to 244 million over the century) and pay nearly all of it away as dividends; the debt is the losses of three to seven towns that make no services of their own and bring them in. In such a town the price of services runs a three-month saw: thin shelves mark the price up, seven months of services arrive at once from a dozen towns, the price is marked down below what they cost to bring, and half of what is left spoils each month. With the markup at 0 the debt is 0.7 per cent of the money in the 1890s, not 45; with no markup on services alone it is 1.4, and nothing else on the eight islands moves beyond the spread between them. It is a fault, not only a number: the merchants of those towns buy dear and sell cheap for a century, a fifth of the services made spoil, and the debt bends the issue of money (`tr-money.md`).
2. **The runaway wages are a mine's hiring, let through by the labour market's monthly limit.** In all four runs followed, a mine (coal or iron) hires two and a half to six times its town's hands in the January after a year's turn, because the price it hires on has just jumped: a district's dearer prices carried in by people who moved (island 9, traced to the town they came from, and very likely island 4), or a new town on a coalfield whose carting to market falls away (island 3; island 11's first year was not traced). The wage may only double in a month and every hand asked for at that wage is hired, whether or not the town has him; the land lets hands go at 15 per cent a month, so for seven to ten months no wage can clear and the wage doubles to its ceiling of 500. Works are its victims, never its cause. The smallest rule that stops it is that no town hires more hands than it has for hire: tried, it ends all four runaways (the worst town is then 1.5 to 1.8 times the middle wage) and moves nothing on today's eight islands beyond the spread between them.
3. **The twins are mostly the works of 1749, put on one cell.** The port's rule for the island's first artisans splits a town's need for a trade into up to six works and finds each the best site in turn, without seeing the ones already placed, so they land on one cell with one size and one history: 48 to 74 of the 162 to 164 old works of 1749 are exact twins, island by island. The rest are sister and family works that pick the same best cell in the same year. It moves nothing in the economy that eight islands can see, but a player sees two to six works of a kind side by side in her first towns and reads each closure in the news. Placing the works of 1749 on separate cells removes those twins and moves nothing beyond the spread.
4. **Ashby Mill No. 5 was closed by the cash of its own extension, not by its trade.** It was sound (a return of 0.156, full time) when the family's branch, which never looks at the works it extends, more than quadrupled it and paid 230,970 into its cash. The works' dividend rule, which knows nothing of an order still to pay for, sent 157,767 of it back to the family within the year while the plant was delivered and paid for, the mill's cash went to -118,688, and the review closed a profitable mill as insolvent. One line in the dividend (keep back what the order will cost) takes the large cases away on eight islands.

## 1. The merchants' debts

### The question

Under the price rule the merchants owe 47 to 60 per cent of all the money by 1900 (`tr-money.md`), and on island 7 in 1880 six towns' merchants hold less than nothing, from bringing in services, which spoil (`dx-inspector.md`, end of section 1.5). Who lends it, which goods and towns it comes from, when it starts, what it does to prices and the issue, and whether it is a fault.

### What was measured

```bash
loose_ends merchants --seeds 1-8 --decades
loose_ends merchants --seeds 1-8 --decades --set '{"prices":{"enabled":false}}'
loose_ends merchants --seeds 1-8 --decades --set '{"prices":{"markup":0.0}}'
loose_ends merchants --seeds 1-8 --decades --set '{"prices":{"glut_months":0.0}}'
loose_ends merchants --seeds 1-8 --decades --set '{"decay":[0.03,0.01,0.005,0.002,0.002,0.002,0.003,0.01,0.002,0.002,0.003,0.005,0.002,0.003,0.002,0.1]}'
loose_ends market --seeds 7 --town Cardale --from 1879 --to 1880
```

(`loose_ends` is `cargo run --release -p isostate-core --example loose_ends --`.) For every island and year the example adds up the merchants' cash, what those below nothing owe, the stock they hold at its value, their takings less what they paid out (`merchants_net` in the town books) and their dividends, and it measures what the year's end moves into or out of their purses (the debts of works that close, which `firms::close_firm` hands to the town's merchants: "creditors take the loss"). It then takes each town's takings apart good by good with the books' balance of goods, at the year's mean price:

    takings on a good = the spread on what the town made (0.06 of its value)
                      + what came in, valued here, less what it cost delivered (the trade table's value and carting)
                      - what spoiled, at the price
                      - what was added to the stock, at the price
                      + what went out, at what it fetched, less its value here
                      + the months' prices against the year's mean

The six terms add up to the books' `merchants_net` to within about a tenth (20 to 28 million of 147 to 244 million on each island, all of it on the same side): the rest is the works that buy their materials below the town's price because they stand nearer the source (`econ::step_month`, `pin`), which this sum prices at the town's.

### Who lends it

Nobody. A purse in this model can go below nothing: the merchants pay for what they buy whether or not they have the cash, and the monthly money check counts their purse as negative money, so the money stays conserved and nobody is owed in particular. The counterpart is every positive purse on the island. No interest is charged, nothing limits it, and no rule reads the merchants' cash except their dividend and the count on which money is issued (`econ::money_total`, `econ::step_month`). It is an overdraft at a bank that does not exist.

### Which towns and which goods

On every island the merchants together take in more than they pay out, and pay nearly all of it away:

| island | debt in 1900, share of the money | towns in debt | net takings 1750-1900 | dividends | closed works' debts |
|---|---|---|---|---|---|
| 1 | 0.88 | 5 | 179.8 M | 189.0 M | 3.1 M |
| 2 | 0.64 | 7 | 244.3 M | 247.3 M | 2.5 M |
| 3 | 0.37 | 4 | 208.3 M | 205.4 M | 2.1 M |
| 4 | 0.40 | 7 | 228.1 M | 226.4 M | 2.8 M |
| 5 | 0.26 | 6 | 175.5 M | 170.8 M | 2.3 M |
| 6 | 0.31 | 3 | 146.7 M | 141.9 M | 2.5 M |
| 7 | 0.39 | 5 | 191.1 M | 188.3 M | 2.3 M |
| 8 | 0.40 | 7 | 167.6 M | 168.1 M | 2.4 M |

The dividend is 0.15 of whatever cash stands above two months of sales, every month, and nothing ever comes back the other way. So the towns whose merchants gain pay it out to their households, and the towns whose merchants lose keep their losses as debt: the island's debt is the sum of the losers' losses. The debts of works that close come to 2 to 3 million a run, spread over every town's merchants, most of whom carry them out of their gains; they are not what puts a town in debt. In 1900 three to seven towns hold all of it, and in every one of the 41 debtor towns listed (the six deepest on each island) the good that lost most over the town's life is services: 0.6 to 14.8 million a town (imports and spoilage together), against at most a few hundred thousand on its next worst good, grain, bricks or cloth.

Good by good, the eight islands and the century added up, in thousands:

| good | spread on what was made | imports | spoiled | stock built up | in the debtor towns' years, all six terms |
|---|---|---|---|---|---|
| services | +458,798 | +91,860 | -1,360,353 | -16,810 | -288,107 |
| grain | +320,353 | +68,446 | -142,501 | -8,468 | +15,500 |
| cloth | +316,936 | +56,080 | -22,143 | -15,195 | +21,891 |
| coal | +115,817 | +722,989 | -6,429 | -9,631 | +32,276 |
| every other good | | | | | +87,390 together (iron ore -1,773) |

In the debtor towns every good but services and iron ore pays, and services lose 288 million. A fifth of the services made on the island spoil (1.36 thousand million of 7.65), at the merchants' cost.

### How a town gets there

Cardale on island 7 has made no services since 1835 and brings them from up to eighteen towns. Month by month in 1879 and 1880 (`loose_ends market`, raw `market-s7-cardale-services.txt`):

| month | price | anchor | stock in | asked for | spoiled | came in | stock out | merchants' cash, change |
|---|---|---|---|---|---|---|---|---|
| 1880-06 | 13.23 | 13.52 | 1,227 | 1,043 | 92 | 1,359 | 1,450 | +10,667 |
| 1880-07 | 11.73 | 12.43 | 1,450 | 895 | 278 | 7,538 | 7,816 | -93,543 |
| 1880-08 | 10.41 | 12.08 | 7,816 | 1,002 | 3,407 | 1,383 | 4,790 | +5,013 |
| 1880-09 | 11.73 | 13.28 | 4,790 | 1,187 | 1,802 | -539 | 1,263 | +20,530 |
| 1880-10 | 13.23 | 13.32 | 1,263 | 1,090 | 86 | 1,360 | 1,446 | +9,591 |
| 1880-11 | 11.73 | 12.25 | 1,446 | 958 | 244 | 7,743 | 7,987 | -62,359 |

The same three months repeat all through both years. When the shelves are thin the markup sets the price above the anchor (13.23). The trade rule then sends to Cardale a share of each sender's stock, in proportion to the gap between Cardale's price and theirs with the carting, without asking what Cardale needs: seven months of requests arrive in one month, landed at 10.3 to 10.5 a unit. The shelves are full, so the markup takes the price down 12 per cent a month, to 10.41, and half of what is not sold spoils each month (3,407, then 1,802). By the third month the shelves are empty and the price goes back up. The merchants buy at the top of their own price and sell at its foot, and lose about 160,000 a year.

### What causes it, shown

| setting | debt as a share of the money, 1890s (mean of 8) | in 1900 | towns in debt in 1900 |
|---|---|---|---|
| the game's | 0.452 | 0.26 to 0.88 | 3 to 7 |
| no glut brake (`prices.glut_months` 0) | 0.492 | 0.16 to 0.83 | 2 to 7 |
| services spoil at 0.1 a month, not 0.5 | 0.119 | 0.07 to 0.19 | 2 to 7 |
| no stock markup (`prices.markup` 0) | 0.007 | 0.00 to 0.02 | 0 to 2 |
| the price rule off | 0.014 | 0.00 to 0.06 | 0 to 2 |

So it is the markup, and the good it bites on is the one that spoils. Under the markup the services do not spoil much more overall (a fifth of what is made, against 16 per cent with it off), but the spoiling moves: from the towns that make services, where the spread pays for it, to towns that only bring them in, where nothing does. In the debtor towns' years services spoiled to 330 million under the game's rules, 9 million with no markup and 16 million with the rule off. The markup does its work on goods that keep (`prices.md`); on a good that halves every month the price it marks follows a stock that swings from empty to glut every three months, and the trade rule, which ships on the price of the moment, ships at the peak.

**Two mends, tried.** Both behind environment variables in the worktree and not committed:

```bash
ISOSTATE_TRY_NO_SPOIL_MARKUP=1 loose_ends merchants --seeds 1-8 --decades
ISOSTATE_TRY_NO_SPOIL_MARKUP=1 isostate-core sweep --seeds 1-8 --rules game --families 6 --out .../trial-no-spoil-markup --compare .../merged-game-f6-l
ISOSTATE_TRY_IMPORT_CAP=1 ...   (the same two)
```

- *No markup on a good that spoils fast* (`prices::mark` takes the markup as 0 for a good whose `decay` is above a tenth a month, which today is services alone; two lines). The debt is 0.014 of the money in the 1890s and 0.00 to 0.03 in 1900, in one town at most. The sweep: 434 facts of 446 differ, none starred; the real wage of 1900 is 2.64 for 2.60 (six islands up, two down), people 0.8 per cent more (six and two), the towns' share of people 0.410 for 0.419 (all eight lower), carters a little more of the hands in 1900 (0.076 for 0.064, all eight). This is the mend I would choose.
- *Ship no more than fills the buyer's stock to its target* (the trade block, ten lines). It ends the debt too (none in the 1890s on any island), but it throttles every good's trade, coal and stone with the rest: 16 facts starred, the real wage of 1900 2.15 for 2.60, people 4.8 per cent fewer and goods a head a quarter fewer on all eight islands, works founded and closed twice as often, and coal's price between towns twice as far apart. The trade rule's habit of shipping a share of the sender's stock is what moves the island's bulk goods, and it cannot be capped by the buyer's own stock target without starving the towns that live on them. Not the mend.

### When it starts

The debt passes a tenth of the money in 1820 to 1870 on seven islands (island 6 is the exception: its debt passes a quarter of the money at once in 1758 and is worked off after; I did not follow that year). Decade by decade, means of the eight islands: 0.003 of the money in the 1760s, 0.017 in the 1800s, 0.104 in the 1830s, 0.229 in the 1860s, 0.452 in the 1890s. It grows as towns are founded or lose their shops and come to depend on services brought in; 27 of those 41 debtor towns were founded after 1790.

### What it does to prices and the issue

Nothing reads the merchants' cash but two rules, so the debt itself moves no price directly. It works through two roads:

- **The issue of money.** Money is issued each month as 2 per cent a year of the money counted, net (`tr-money.md`, section 1), and a purse below nothing counts against the rest. In 1900 the money is 36.4 to 38.5 million under the game's rules and 39.1 to 39.9 million with the markup at 0 or the rule off, where almost nothing is owed: about 5 per cent more. Against the 56.1 million that issuing on the true total would give (`tr-money.md`), the debt is about a tenth of the shortfall; the rest is the households' month of income in flight. `tr-money.md` found prices rising a little less than the money (money 1.46 times, prices 1.29 to 1.33), so the debt holds the price level of 1900 a few per cent lower; I did not sweep it apart from the markup's own effects on prices.
- **What households receive.** The towns in debt pay no merchants' dividend, and the towns that sell them services are paid for services that rot. Real hands in the exporting towns' shops make services that nobody uses: about a fifth of all services made.

### How sure

Sure of the road. The markup alone takes the debt from 45 per cent of the money to 2 per cent at most on every one of the eight islands, and the markup on services alone does the same; the month-by-month account of one town shows the saw and its cost to the coin; the books' balance gives services as the worst good of every debtor town, by ten times or more. Less sure of the size of the second-order effects on prices, which I took from the money in 1900 and `tr-money.md` and did not sweep separately. What would change my mind: a debtor town whose losses are not services, or debt that stays with the markup at 0 on another eight islands.

### Fault or number

A fault, though a quiet one. Merchants who lose money every year for seventy years and go on buying are not merchants; the debt has no lender and no limit; a fifth of the services made rot; and the money count is bent by it. It is not dangerous to the run: nothing else reads the purse.

### For the pass over all the rules

- **`prices.markup` on a good that spoils.** The repair is in the price rule: no markup on a good whose `decay` is above about a tenth a month, which today is services alone (trial above: the debt goes and nothing else moves beyond the spread). The anchor alone prices services without the saw. The production tree's new perishable goods, if any, would fall under the same line.
- **The trade rule asks nothing of the buyer** (`econ::step_month`, the trade block): it ships a share of the sender's stock on the price gap of the moment, so seven months of a good can arrive in one. That is how the bulk trade works and capping it by the buyer's stock starves the island (trial above), so leave it, but know that any good whose price swings fast will be shipped at its peaks.
- **Services carted between towns at all.** Services weigh 4 a unit, more than any good, and spoil by half a month. Whether they should travel is a question for the production tree, which adds shops and offices.
- **The merchants' dividend** pays out gains and never takes in losses, so any loss in a town becomes a debt for ever. A merchant below nothing could at least draw on his town's households, as a works draws on its owners (`firms::back_firm`); or the pass could keep the asymmetry and watch it.

## 2. Runaway town wages

### The question

One town's wage at 84.5, 100 and 120 times the middle town's in 13 of 528 runs of the tables of sensitivity (`sens-mines.md`, `sens-works.md`); `mines.md` says a working can hire more hands than its town has, and `harvest.md` describes a knife's edge. Is it a mine's hiring, a works', the labour market's monthly limit, or all three, and what is the smallest rule that stops it?

### What was measured

The four worst runs the pages name, each reproduced first with `wage_watch` (the 84.5 is `wage_watch game 6 on 9 9 mines.sink=40.0002`, as `sens-mines.md` has it) and then followed month by month:

```bash
loose_ends wages --seeds 9  --set '{"mines":{"sink":40.0002}}'      --town Iveldelf --from 1780 --to 1783 --works iron_ore
loose_ends wages --seeds 11 --set '{"carting":{"labor":0.5}}'       --town Melford  --from 1750 --to 1750
loose_ends wages --seeds 3  --set '{"works":{"year_room":0.1125}}'  --town Brambank --from 1750 --to 1751
loose_ends wages --seeds 4  --set '{"works":{"year_room":0.1125}}'  --town Sunhaven --from 1784 --to 1785
```

Each month: the town's wage and its multiple of the middle town's, its people, the hands for hire as the month counts them (half the people, less the carters, the sinkers of new workings and the navvies, taken off by the same functions the month uses), the hands at work on the land by sector and in works by trade, and the *floor*: what last month's employers must still employ this month whatever the wage, since land lets hands go at 15 per cent a month (`labor_adjust_ex`) and works at 35 (`labor_adjust_firm`).

### What happens

Iveldelf, island 9, a town of 2,627 founded in 1776. All through 1780 and 1781 its hands at work equal its hands for hire every month and its wage is 0.7 to 0.9 of the middle town's. Its price of iron ore is 0.06 to 0.13; the middle town's is about 1.0. At the turn of 1781, 748 people come to it from five towns as districts change market, one of them Falhaven, where iron ore costs 5.78 and buyers ask for 362 a month. Then:

| month | wage | times the middle | for hire | iron miners | other hands | at work / for hire | floor / for hire |
|---|---|---|---|---|---|---|---|
| 1781-12 | 3.88 | 0.69 | 973 | 0 | 970 | 1.00 | 0.78 |
| 1782-01 | 8.66 | 1.58 | 1,320 | 6,450 | 1,129 | 5.74 | 0.83 |
| 1782-02 | 17.33 | 3.07 | 1,315 | 6,092 | 911 | 5.33 | 4.87 |
| 1782-04 | 69.31 | 12.11 | 1,421 | 4,451 | 631 | 3.58 | 3.57 |
| 1782-07 | 500.00 | 84.44 | 1,561 | 2,734 | 374 | 1.99 | 1.99 |
| 1782-10 | 500.00 | 84.04 | 1,628 | 1,679 | 226 | 1.17 | 1.17 |
| 1782-12 | 125.00 | 20.10 | 1,646 | 1,213 | 162 | 0.84 | 0.84 |
| 1783-04 | 9.62 | 1.49 | 1,806 | 707 | 1,098 | 1.00 | 0.48 |

At the turn the town's price of iron ore became 4.58, and the price it expects, on which the land hires, 4.72. That is `peopling::prices_go_with_trade`: a market's price becomes the average of its own (weighted by its own trade, here nothing) and the newcomers', and for what the newcomers' old town bought, the newcomers bring their share of all its requests at its price with the carting. So Falhaven's want of ore (its works are the only buyers of ore) came to Iveldelf with people who never bought any. The iron deposit in Iveldelf's district, idle at a price of 0.1, was offered 4.7, and in January asked for some 43,000 hands: the land's hiring rule moves 0.15 of the way from last month's hands (here none) to `lam * (net price / wage)^(1/alpha)`, which with alpha at 0.3 answers the price to the power of 3.3. 0.15 of the way to 43,000 is 6,450, five times the town's hands for hire.

The labour market then does what it does. The wage is searched each month between half and twice last month's (`econ::step_month`, the labour market), and if demand at twice the wage still exceeds supply, the wage is twice, and every hand demanded is hired and paid, whether the town has him or not. The miners can only be let go at 15 per cent a month, so the floor stays above the hands for hire from February to October: no wage could clear it, and it doubles each month to the ceiling of 500. The town's works and farms are wiped out by the wage meanwhile (works' hands 172 to 1); the ore floods out and its price falls from 4.58 to 1.22 by December, which is what finally lets the wage down.

The other three runs have the same shape, each in the January after a year's turn:

| run | town | in January | hands for hire | the mine's hands | what jumped at the turn | worst |
|---|---|---|---|---|---|---|
| island 11, `carting.labor` 0.5 | Melford, founded 1749 | 1750 | 10,265 | 45,661 colliers | not traced: the first recorded year, the town made in the warm-up | 500, 100 times, July to November |
| island 3, `works.year_room` 0.1125 | Brambank, founded 1750 | 1751 | 2,452 | 11,507 colliers | a town founded on the coalfield, the carting to market falling away (the town `mines.md` names at the game's values) | 500, 121 times, July to October |
| island 4, `works.year_room` 0.1125 | Sunhaven, founded 1749 | 1785 | 19,153 | 30,753 iron miners | iron ore 10.93 at Sunhaven after a turn at which 800 people came (the middle town's 1.65); not traced further | 284, 47 times, June |

### The answer

All three, in a chain, and never a works. A mine is what hires: in every case the hands asked for in the first month are those of one mining sector, two and a half to six times the town's hands for hire. What sets it off is a jump at the year's turn in the net price the mine hires on, by one of two roads: prices carried into a town by people who moved (Iveldelf, traced; Sunhaven very likely, the same jump at a turn when people came), or a town founded on a coalfield, where the carting to a distant market falls away overnight (Brambank, which `mines.md` describes; Melford likely, in the first recorded year). The labour market's monthly limit is what lets it run: the wage can only double, and demand is met in full at the doubled wage. And the land's slow release of hands (15 per cent a month) is what makes it last seven to ten months. Works are victims: their hands go to nothing while it lasts. The `sens-works.md` settings that showed it (the year's room, the carters' share) do not cause it; they move which island crosses the edge, as `sens-works.md` thought.

### The smallest rule, tried

After the month's wage is found, if a town's employers (land and works together) want more hands than it has for hire, each is given its share of the hands there are: every unit's hiring is scaled by supply over demand, and so is what it starts the next month from (`ISOSTATE_TRY_RATION`, about fifteen lines in `econ::step_month`, not committed). The wage still doubles in the first month, since demand at the top of the month's range still exceeds the supply, but the floor of the next month is then under the hands there are, and the wage can clear.

```bash
ISOSTATE_TRY_RATION=1 wage_watch game 6 on 9 9 mines.sink=40.0002     (and 11 with carting.labor=0.5, 3 and 4 with works.year_room=0.1125)
ISOSTATE_TRY_RATION=1 isostate-core sweep --seeds 1-8 --rules game --families 6 --out .../trial-ration --compare .../merged-game-f6-l
```

On the four runs it ends the runaway: no town-month over four times the middle wage and no town working more hands than it has, where island 9 alone had 12 town-months over four times and 8 overworked, and the greatest wages were 84.5, 100, 121 and 47 times; the greatest is now 1.5 to 1.8 times the middle town's, in a January or February. On today's eight islands, where nothing runs away, 434 facts of 446 differ and none is starred: people in 1900 0.8 per cent fewer (seven islands of eight), works in 1900 3 per cent fewer and railway 3 per cent shorter (both mixed), the real wage of 1900 2.60 as before, famine years 0.5 for 0.4. The four facts that move the same way on all eight are small (wild land in 1850 +1.7 per cent, workings in 1800 +3 per cent, the spread of coal prices in 1820 +16 per cent, the swing of grain's output -3 per cent). So it costs the ordinary island next to nothing that eight islands can see, and it removes the one fault that `sens-works.md` said would colour every other table of sensitivity.

### How sure

Sure for these four: each was followed month by month and the arithmetic of January checks (0.15 of the way to a target of 43,000). I followed one of the two roads to its source (Falhaven's price carried to Iveldelf) and took the other from `mines.md`. I did not follow the other nine of the thirteen runs; `sens-works.md`'s table puts them on the same few islands early in the century, and its own reading (a town on a knife's edge, tipped by any change) is what these four show. What would change my mind: a runaway whose first month is a works' hiring, or one in a month that is not a January.

### For the pass over all the rules

- **The rule to add: no town hires more hands than it has for hire** (R54, "employers are not rationed"), scaling every employer's hiring down in proportion when the demand at the month's wage is more than the supply. It is the smallest rule that cuts the chain where it does harm, and it touches nothing in a month that clears (trial above).
- **`peopling::prices_go_with_trade`**, the requests term: the newcomers carry a share of everything their old town asked for, works' materials included, though the works stay behind. Weighting it by the households' own purchases (or leaving out what only works buy) would have spared Iveldelf, and very likely Sunhaven.
- **A new working's first hiring** reads `pexp` raw. A cap on how far a land unit's target may stand above the town's people (the mines page's "a mine that could hire no more than its town has to spare") is a second, narrower mend.
- **The ceiling of 500 and the doubling** are the port's and are where the damage shows, not where it starts.

## 3. Twin works

### The question

`news-read.md` counted 156 works on island 1 and 190 on island 7 that are exact twins of another (same cell, method, year opened and year closed), most of them works the island starts with, and left them for the works: where they come from, whether it is a fault, and whether a player would see it.

### What was measured

```bash
loose_ends twins --seeds 1-8
```

After each run it groups every works that ever stood by cell, method, year opened and year closed, and sorts the twins by where each works came from (the reason kept with it: `old` for the works of 1749, `new` for the investors', `expansion` for a sister works, `family` for a family's).

| island | works in the run | exact twins | in sets | largest set | twins among the 162-164 works of 1749 | works of 1749 sharing their cell with one of their kind |
|---|---|---|---|---|---|---|
| 1 | 2,223 | 156 | 68 | 6 | 51 | 109 of 271 |
| 2 | 2,252 | 148 | 65 | 4 | 59 | 88 of 197 |
| 3 | 1,886 | 139 | 60 | 4 | 53 | 90 of 196 |
| 4 | 2,183 | 180 | 78 | 6 | 74 | 121 of 213 |
| 5 | 2,307 | 125 | 54 | 6 | 71 | 141 of 258 |
| 6 | 2,133 | 177 | 78 | 4 | 48 | 91 of 204 |
| 7 | 2,320 | 190 | 80 | 6 | 68 | 138 of 258 |
| 8 | 2,468 | 156 | 70 | 4 | 74 | 137 of 249 |

(The works of 1749 are 162 to 164 `old` works and 32 to 109 `new` ones the warm-up's investors open; the last column counts both.) The rest of the twins are sister works (17 to 82 a run, of 201 to 309), families' works (8 to 23 of about 245) and investors' (16 to 46 of about 1,400). Only 11 to 19 sets on an island are still alike in capacity when the run ends, and one to eight share a name: the names are drawn at random (island 4: three works called Elmsea Smithy on cell 676, 1749 to 1764; island 7: Greaves's Forge, Granness Smithy and Jowett Edge Tools on cell 1612, 1749 to 1762).

### Where they come from

`sim::seed_firms`, the artisans of 1750, which the port has from the Python. For each town and each of six trades it works out what the town needs, splits it into up to six works of the usual size, and finds a site for each in turn with `firms::candidate_cells` and `firms::evaluate_sites`. The candidates are 32 cells drawn at random plus the eight where the method's bulkiest input is cheapest, which are the same every time; and nothing in the evaluation sees the works already put down, so the best of them is very often the same cell each time. The works so made have the same cell, size, cash and start, so they have the same history until something parts them, and many close in the same year. Later twins arise the same way at a smaller rate: two works of a trade (often twins themselves) put sister works on the same best neighbouring cell in one year, or two families found on the same best cell.

**The trial.** `seed_firms` not putting a second works of a trade on a cell it has already used for that trade in that town (`ISOSTATE_TRY_SEED_SPREAD`, three lines, not committed):

```bash
ISOSTATE_TRY_SEED_SPREAD=1 loose_ends twins --seeds 1-8
ISOSTATE_TRY_SEED_SPREAD=1 isostate-core sweep --seeds 1-8 --rules game --families 6 --out .../trial-seed-spread --compare .../merged-game-f6-l
```

No works of 1749 is a twin any more on any island (0 of 161 to 164), and the twins fall from 125 to 190 an island to 66 to 120, the rest being the later kind. The sweep: 430 facts of 446 differ, none starred, and none of the headline facts moves the same way on more than five islands of eight (works in 1800 4 per cent more, five up and three down; works in 1900 2.6 per cent fewer, two and six; the real wage of 1900 2.62 for 2.60). So the twins of 1749 cost the economy nothing that eight islands can see, and taking them away costs nothing either.

### Is it a fault, and would a player see it

A small fault. Economically a set of twins is one works of two to six times the size, split, and since they decide alike they act as one; the life cycle (which grows a works where it stands) would treat one such works differently, but only in degree. What a player meets is the picture and the news: the viewer gives every works its own lot on its cell (`scatter.rs`), so a town of 1750 shows two to six smithies, mills or brickfields of one kind side by side, and the news tells each closure ("Ulford Brickworks" closing three times, two "Wilmere Bloomery closes" in one month, `news-read.md` fault 6). The twins of 1749 in the examples above close between 1756 and 1785, so a player meets them chiefly in the first decades of a game, when the home town is what she looks at most.

### How sure

Sure of where the works of 1749 come from: the counts and the code agree, and the trial below takes those twins away. Less sure that the later twins (about half) are all the "same best cell in the same year" road: I read that from the code and the examples, not from a count by cause.

### For the pass over all the rules

Under the game's rules `seed_firms` could put a town's works of one trade on different cells, or make them one works of the summed size (`works::sized_entry` already sizes new works that way). The port must keep it as it is.

## 4. A mill extended the year before it closed

### The question

On island 4 the player's family (the Ashbys, under the autopilot) extended Ashby Mill No. 5 at Fairbank "for 230,970" in 1888, and the mill closed in 1889 (`news-read.md`'s notes). What did the family's rule see, what happened, and is the rule that extends blind to what closes?

### What was measured

```bash
loose_ends extend --seeds 1-8 --works "Ashby Mill No. 5"
```

For every island it notes each order of more capacity for a works already standing (from the capacity on order on either side of the year's end) and whether that works closed within two years. For the works named it prints, before each year's review, the figures the review's closing test reads, and after it the works' and the family's cash; then the works' own books and what a unit of its capacity cost at its town's prices. It happens on today's `main` as it did for `news-read.md`.

### What happened

| | before the review of 1888 | after the turn of 1888 | before the review of 1889 |
|---|---|---|---|
| capacity (on order) | 437 (0) | 420 (+1,878) | 1,958 (+340) |
| the trade's usual size | 150 | | |
| return on capacity in the year | 0.156 (the hurdle is 0.12) | | 0.038 |
| utilisation | 0.99 | | 0.97 |
| a month's costs | 1,919 | | 8,437 |
| the works' cash | 8,208 | 242,243 | -118,688 (it closes below -96,583) |
| the family's cash | 666,313 | 262,049 | 486,758 |

And from the works' books for 1889: output 12,102 (5,244 in 1888), profit 9,154, dividends 157,767, paid for capital goods 212,318, utilisation 0.83 over the year. The year's cash adds up to the coin: 242,243 + 9,154 - 157,767 - 212,318 = -118,688.

The mill was doing well. In 1888 a growing branch of the family at Fairbank (`families::grow`) looked for the best new works of its trade: it priced a new powered mill of the size a new works would have in that town (`works::entry_K`: 1,878, twelve and a half times the usual size) at the best candidate site, found it would return more than the hurdle, and, because the family already had a mill of that method in the town with room on its cell (`works::family_extension`), ordered that capacity for Ashby Mill No. 5 instead (`works::extend_for_family`), paying the cost of a new works, 230,970, into the mill's cash at once. The order more than quadrupled the mill.

What then closed it was its cash, not its trade. A works' capital goods are delivered and paid for a quarter of what is still on order each month, at the month's prices. But the works' dividend is paid every month as 0.4 of all the cash above four months of costs (`econ::step_month`, "dividends"), and a family takes its works' dividends straight into its purse. So through 1889 the mill paid 157,767 of the family's 230,970 back to the family as dividends while the deliveries came in, and ran its purse to -118,688 paying for them. The review's closing test (`firms::review`) calls a works insolvent when its cash is below six months of its costs or 0.4 of its plant's value, whichever is greater; -118,688 is below -96,583, so a mill running at 0.97 of its new capacity with a profit was closed. Its debt went to Fairbank's merchants (`firms::close_firm`); the 1,879 of capacity, worth about 260,000 at Fairbank's prices, vanished with it. The family's purse rose by 225,000 in the year, so it was not ruined; it lost a mill four times the size it had.

Prices made it worse but did not make it: a unit of the mill's capacity cost 112.4 at Fairbank's mean prices in 1888 and 139.4 in 1889 (timber went from 1.99 to 3.40, bricks from 2.61 to 3.13, with the mill's own orders among the buyers).

Across the eight islands it is rare. Of 16,000 to 21,000 orders a run for works already standing, 1 to 9 went to works that closed within two years: 38 in all, 24 of them a family's, the largest 100,000 to 190,000 each (Wentworth Mill No. 8, Pemberton's Brickfield No. 9, Calloway's Brickfield No. 11, Vavasour's Brickfield No. 6, Egerton Engine Works No. 5). The large ones are all family works extended by a family's branch late in the century.

**The trial.** One line in the works' dividend: the cash above four months of costs less what the works' order still to come will cost at the town's prices (`ISOSTATE_TRY_KEEP_ORDER`, not committed). With it, orders for works that closed within two years fall from 38 worth 1.66 million on the eight islands to 8 worth 10,256, none larger than 4,324; the large family extensions that closed are all gone. The history of island 4 moves with it, so "Ashby Mill No. 5" is then another mill, which is extended ten times between 1846 and 1884 and stands. The sweep (`trial-keep-order`): 432 facts of 446 differ, none starred; works standing in 1800 are 9 per cent more on all eight islands (258.5 for 237.1; most likely new works, which are founded the same way with their money in their cash and their plant on order, no longer pay their founding money out: Ashby Mill No. 5 paid 27,455 of its 47,840 out in its first year, 1874), works in 1900 1.3 per cent fewer, people in 1900 0.6 per cent fewer and the real wage of 1900 2.55 for 2.60, both four islands up and four down.

### Is the rule that extends blind to what closes?

Yes, in two senses, and the second is the one that bit. The family's rule never reads the works it extends: not its return, its utilisation, its cash or its years at a loss; it reads the return of an imagined new works at the best site in town and gives that capacity to the family's largest works of the method that has room. Here the mill happened to be sound, so that blindness did no harm. And it is blind to the closing test: it puts the money in as cash, where the dividend rule, which knows nothing of an order still to pay for, sends it straight back to the family, and the review then closes the works for the debt the deliveries leave. A works extended by the review itself (`firms::review`, which does read the works' own return and utilisation) can meet the same end, but it orders a step of half its size at most and pays as it goes.

### How sure

Sure of what happened to this mill: each month's cash is accounted for by the dividend and the deliveries, and the trial below, which only stops the dividend paying out money owed on an order, keeps the mill. Fairly sure it is the same road for the other large cases, which have the same shape (a family works, a large sum, closed the next year), though I followed only this one.

### For the pass over all the rules

- **A works keeps back what its order will cost before it pays a dividend** (`econ::step_month`, the works' dividend): the smallest mend, one line, and it is the port's rule only in a game with families and the life cycle that makes such orders large. Or the family pays for its extension as it is delivered, as the railways' promoters call on their subscribers.
- **The closing test** could spare a works whose debt is its own plant still arriving, for as long as it is building (`works::building` already knows when that is).
- **`works::family_extension`** might ask the works it extends for the review's own test (a return above the hurdle and utilisation above 0.65), and its size might be a step on the works' own size rather than a whole new works' capacity; that is a choice for the design, not a fault.

## What it means for the pass over all the rules

Three of the four are rules missing a question, and each has a small mend that the trials say moves little else:

| | the fault | where | the mend tried | what it moved on eight islands |
|---|---|---|---|---|
| merchants' debts | prices marked by stock on a good that halves every month; imports shipped whatever the buyer needs | `prices::mark` | no markup on a good that spoils fast (services) | nothing beyond the spread; the debt goes (a cap on imports also ends it but starves the island's bulk trade) |
| runaway wages | hands hired that the town does not have | `econ::step_month` (labour market) | ration hiring to the hands for hire | nothing beyond the spread; the four runaways end |
| twin works | the works of 1749 put on one cell | `sim::seed_firms` | one cell per works of a trade | nothing beyond the spread; the twins of 1749 go |
| the mill that closed | dividends paid out of money owed on an order | `econ::step_month` (dividends) | keep back what the order will cost | 9 per cent more works in 1800, nothing else beyond the spread |

Each would be a switch under the game's rules, with the port left as it is (`docs/rules/README.md`). Two of them bear on numbers the pass will read: the merchants' debt bends the issue of money (and so every money price; `tr-money.md`), and a runaway wage on one island in forty moves an eight-island mean (`sens-works.md`), so the rationing rule is worth having before the scorecard is read. Two things found on the way belong to the pass as well: `peopling::prices_go_with_trade` carries the works' requests of a town with the people who leave it, which is what set off two of the four runaways; and the merchants' dividend takes gains out and never puts losses back.

## Not done

- The nine other runaway runs of `sens-works.md` were not followed one by one; the four worst were.
- The mends were tried on eight islands only, not sixteen, and only as trials; none was searched for its edges (a town with no hands for hire, a good that spoils at just over a tenth).
- The later twins (sister works and families' works on the same cell in the same year) were read from the code, not counted by cause.
- The price level's answer to the merchants' debt was read from the money of 1900 and `tr-money.md`, not swept apart from the markup's other effects.
- Only Ashby Mill No. 5 was followed month by month; the other large extensions that closed were counted.
