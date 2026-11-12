# dx-harvest-month: a harvest that comes in a month

*Where the code is: this page's verdict is to build a harvest month, but not as this prototype, so only the page is on `main`. The prototype behind `season.enabled`, its test and its measuring example are kept on a branch of their own (`night/o-dx-harvest-month-check`), where every command below runs.*

Branch `night/o-dx-harvest-month` (a prototype and its measuring code, not for merging as they stand). Piece 20 of `docs/backlog/found-along-the-way.md`, from the rows behind it: the island has no seasons, so a bad year is twelve short months with no harvest month and no stock that means anything (R48, C69); grain's price saws for more than a year after a shock (R46) and answers a short harvest too weakly (R49). Since 12 November the clock tells the month, but the farms' year is flat: a year's weather is drawn once, in December, and the farms bring in their grain evenly through the twelve months (`docs/rules/harvest.md`). The question is what it would take for the crop to come in at harvest, be stored and be sold through the year, and what that would do to the island.

Everything is on seeds 1 to 8 under the game's rules with six families, at `main` 11dbe7d. The prototype is `crates/isostate-core/src/season.rs`, behind `season.enabled`, which is off under the port and off under the game; it is hooked into `econ::step_month` in four places marked `season:`. The measuring code is `crates/isostate-core/examples/harvest_months.rs` (below, "How to run it again"). Raw output is under `.claude/night/reports/opus/dx-harvest-month/`, with the notes of every run (`build.md`). With the switch off a sweep gives every one of the 446 facts exactly as `main` does (`sweep-off5.txt`: "0 differ"), and `cargo test -p isostate-core --release` passes, the golden test among it (213 tests, two of them the probe's own in `tests/season.rs`). The viewer builds on it with no new warning, its tests and the generators' pass, and its twelve standard pictures are the same as the last landing's to the pixel.

## In short

1. **Today the farms' year is flat.** In an ordinary year the merchants hold 1.9 months of grain in every month, the island grows within one per cent of the same amount each month, and the price stands within half a point of the year's mean from January to December. A bad harvest is dear from January, at its dearest in July and August (11 per cent above the same month a year before, for a harvest 9 per cent short), and by the next summer grain is 9 per cent cheaper than before it: the saw of R46.
2. **The smallest rule** lets the crop stand in the fields and brings it into store in August and September, with the year's weather on it; the store then has to last the town until the next harvest, and the merchants mark grain's price by how far their store is from what an ordinary year would hold that month. A month in store costs 0.4 per cent of the grain held, so the island loses what it loses today, 2.6 per cent of its crop. The money is not moved in time: merchants pay for the crop as it grows, as now.
3. **It took five forms to get there**, and the four that failed are the finding. The price rule was written for grain that arrives every month, and both of its halves read a store that fills twice a year wrongly: the stock half as a glut followed by a dearth, the flow half as a famine in every town that buys its year in the autumn and a glut in every town that grows for others. Left alone, they starved a quarter of the island every July; patched badly, they starved it five years in six.
4. **The last form works and costs something.** An ordinary year holds 12.9 months of eating at the end of September and 2.4 at the end of July, and hardly anyone is hungry (0.2 per cent of people, in July). A bad harvest's dearth now comes after the harvest and builds to the next July, 18 per cent above the year before at its peak; the saw is gone (the price's lag-one correlation 0.00 against -0.58). But famine years go from 0.4 a run to 4.2, nearly all in July, and there are 3 per cent fewer people in 1900; and the farms' output swings twice as hard from year to year.
5. **Build it, but not as this.** A harvest month gives the clock a year to show, makes stocks and a hungry July mean something, and puts the dearth in the months it belongs in. It should be built with a price rule written for stores (a merchant who prices the store to last to the next harvest), not on three patches to the rule for flows, and with the farm hands' year beside it. Several days, after the scorecard, as the backlog says.

## 1. Today's months

`harvest_months game 6 1 8`, the years 1760 to 1899 of the eight runs together. Each month is set against its own year's mean, so that the century's trends drop out; "held" is the island's stock at the month's end in months of what the island ate that year.

| month | grown | held | price | real wage |
|---|---|---|---|---|
| January | 1.007 | 1.89 | +0.00% | -1.6% |
| March | 1.010 | 1.91 | -0.40% | -1.5% |
| May | 1.002 | 1.92 | -0.47% | -1.9% |
| July | 0.993 | 1.91 | -0.08% | -0.3% |
| September | 0.991 | 1.90 | +0.29% | +1.4% |
| November | 0.996 | 1.89 | +0.32% | +2.6% |

Nothing moves with the season. The dearest month of a year is January in 414 years of 1,120 and December in 170, and the cheapest January in 391 and December in 205: what changes a price within a year is the year's turn, when the land, the towns and the weather are drawn again, not the season. The dearest month stands 0.103 above the cheapest on average (in logs; median 0.093), and between years the year's mean price moves by 0.109 (the standard deviation of its yearly change, the sweep's `price_swing_grain`). The real wage rises through the year by about four points and falls back at its turn. Grain spoiling in store (three per cent a month on the stock after the month's sales) costs 2.6 per cent of all that is grown. Hardly anyone goes hungry: seven town-months in the 1,120 years, all from January to March, about a hundredth of a per cent of people.

**A bad year and a good one.** The tenth worst and the tenth best of each run's harvests, 104 years each, the price against the same month of the year before:

| | January | April | July | October | next January | next April | next July |
|---|---|---|---|---|---|---|---|
| bad year (0.914 on average), price | +2.5% | +8.4% | +11.0% | +10.2% | +4.4% | -5.1% | -9.1% |
| bad year, held (months) | 1.82 | 1.64 | 1.57 | 1.61 | 1.88 | 2.32 | 2.45 |
| bad year, real wage | -3.5% | -5.2% | -3.4% | -0.6% | +1.0% | +0.9% | +2.6% |
| good year (1.096), price | -3.7% | -9.5% | -12.2% | -12.2% | -8.4% | +1.4% | +6.3% |

The weather is known in January and the dearth begins then: half of it through the farms, which hire fewer hands for a crop they can see will be thin (`harvest.foreseen`), and half through the crop itself, which comes to market thin each month. The price rises for six months as the merchants' stock runs down from 1.8 to 1.6 months, is at its dearest from July to September, and falls back in the winter, which is when the calendar year ends. The year after, it overshoots: grain is 9 per cent cheaper by the next summer than before the bad year, because the hands the dear year drew to the land are still there. That is the saw. Households eat 3 per cent less at the worst and nobody is hungry.

## 2. The design

### The smallest rule

- **The crop stands until the harvest.** What a town's farms grow each month joins the town's standing crop and does not reach the market. In August half of what stands is brought into store, and in September the rest; both before that month's market, so that the new grain is sold in the month it is cut. (`season.month` and `season.months`. The picture already ripens the corn in high summer and cuts it in late summer, the oats a little after the wheat: `terrain.wesl`, `crop_look`, where the cut falls at 0.61 to 0.69 of the year's phase, which runs from midwinter.)
- **The year's weather goes on what is brought in.** The weather is drawn in December for the coming calendar year, as now, and each town's harvest for the year (`harvest_town_now`) is put on the whole of what it brings in. The standing crop is kept before the weather, so October to December's growing, which belongs to next year's harvest, takes next year's weather.
- **The store has to last.** Each town's merchants aim to hold, at the end of each month, the two months of sales they always meant to hold, plus what an ordinary year would still hold that month of a year's sales in the town: 4.5 months' sales at the end of August, 10 at the end of September, one fewer each month after, none at the end of July (`season::ordinary_year`). The price is marked by how far the store is from that line, so a store full in October and low in July is an ordinary year, and a short harvest is a store below the line from the day it is in.
- **The money is not moved in time.** Merchants pay for the crop as it grows, at the month's price, as they do today, and the landlords take the weather in their rents month by month. Only the grain is moved. This keeps every purse on the path it has today, so the money check, the landlords' purses and the families' land prices are what they were.

### Who holds the stock, and what it costs

The model has one stock of grain per town, the merchants', and the price is marked on it. In England in the eighteenth century the farmer held most of the year's grain, in stacks in the yard, and threshed it through the winter as the price and his rent day asked; the miller, the factor and the corn merchant held weeks of it, not months. The prototype lumps the stacks and the granaries into the one stock, which is the smallest change and keeps the price where it is marked today. A built version should say who holds what: a farmer who holds his crop until he must pay his rent sells into the market on a schedule of his own, and that is where the season's price comes from historically.

What it costs to hold is, here, only what is lost. Grain in a stack or a granary keeps far better than the content's three per cent a month, which is a rate for stock that turns over in two months. The prototype charges 0.4 per cent of what is held each month in place of it (about five per cent a year to rats, damp and weevils), chosen so that the island loses what it loses today: 2.6 per cent of its crop, measured both ways. At the content's three per cent a seasonal store would lose a quarter of the crop; at 0.8 per cent it lost 5.0 per cent of it (the first form).

Interest is not charged, because the model has none: nobody pays to have money tied up in a stack. Historically that, with the losses, is the cost of carry, and it is why grain was cheapest at harvest and dearest before the next, by some ten or fifteen per cent in an ordinary year. The prototype has no such rise (below). A built version needs a carrying cost the merchant is paid for: the price marked up month by month through the crop year, by the interest and the losses, and down again at the harvest.

### What a bad harvest should do, month by month

The eighteenth century's pattern after a short crop was a rise at the harvest, when the crop could be seen and counted, and a further rise into the following spring and summer, to a peak in June or July before the next harvest; "King's law" is that a crop a tenth short sold some three tenths dearer (R49). Today the island gives a rise from January to August of 11 per cent and a fall after. The prototype gives a rise that begins at the harvest and builds to the next July: +3.5 per cent in August, +8.1 in December, +17.7 the next July, for a harvest 9 per cent short (section 4). The shape is right; it comes too slowly. The price is marked on the store's shortfall as a share of the line, and the line is long after the harvest and short before the next, so the same shortfall reads as small in September and large in June. Households eat as usual into the spring and are rationed late. Marking grain's store twice as hard (`season.markup` 1 and 2, in place of the merchants' 0.5) to make the rise come at the harvest broke the island's money in 2 of 8 runs at 1 and in all 8 at 2 (the money check fails when the island's purses together have gone to nothing or below it, as `harvest.md` recorded at `foreseen` 2). That was measured on the third form of section 3; run again on the last form in the probe's check, it is the same: seeds 1 and 8 fail at 1, in 1842 and 1882, and all eight at 2, between 1797 and 1838. So the price rule as it stands cannot be asked to do it.

### The news of dearth

The dearth item reads the calendar year's mean price of grain against its trend, and names "a harvest N short" from the same calendar year. With a harvest month the dearth falls mostly in the calendar year after the short harvest, so the item would name the wrong harvest, and it would be told in December about a price that peaked in July. With the season on, dearths are told 14.8 times a run against 8.2. A built version should tell a dearth in the month the price crosses its line, from the spring to the harvest, naming the harvest that is being eaten; and a famine in July after a short crop is the hungry gap before the harvest, which is news of its own.

### What else must then follow the seasons

- **The farm hands.** The farms hire the same each month today, on the expected price. A real farm's year was ploughing and sowing in the autumn and spring, little in the winter, and the harvest in August and September, which wanted every hand there was and drew labourers and their families from the towns: the harvest wage was about double the winter wage. A harvest month without a harvest wage is half a season. It is also where the labour economics is (the harvest as the year's labour bottleneck, and the reapers who came from other districts). The prototype leaves hiring flat, and the farms' hands still follow the price, which now swings harder: the output swing between years doubles and the farm zigzag goes from 2.1 to 3.6. A built version should give the farms a monthly profile of hands with its mean at one, and settle the zigzag with it.
- **Carting.** Grain moves after the harvest: the towns that grow for others send their surplus in the autumn, by cart and by water. In the prototype the canals carry grain for a third of their traffic in 1800, against a fifth (`canal_carries_grain_1800` 0.336 against 0.223, every island). The carters (`docs/rules/carting.md`) are counted by the month's carting bill, so they follow the grain already; whether the roads can take it in the autumn and not in the winter is not modelled.
- **Winter.** Roads were bad and canals froze from December to February, works on water power stood in a drought and a frost, and the need for warmth was a winter need. None of it is in the model, and none of it is needed for the grain. It would be the second step of a seasonal island, after the harvest and the hands.
- **The rent day.** Rents were paid at Michaelmas and Lady Day, and a farmer threshed and sold to meet them. Moving the money in time would put the weather on the landlords at the harvest and not month by month, and is how a family could be ruined by one harvest (R51); it is a larger change than this probe, and the families' land prices (R50, R197, C67) would follow it.

## 3. Five forms, and what each taught

| form | what changed | people in a hungry town, July of an ordinary year | famine years a run | people in 1900 | grain's price between years |
|---|---|---|---|---|---|
| main | | none | 0.4 | 731,872 | 0.109 |
| 1 | the crop goes into store at the month's end; the store aimed at the town's own crop; loss 0.008 | 12% (and 25% in August) | not swept | | 0.184 |
| 2 | the crop goes in before the month's market; the store aimed at what the town sells | 23% | not swept | | 0.081 |
| 3 | the flow half of grain's price left out; loss 0.004 | 3.2% | 10.4 | 574,944 | 0.168 |
| 4 | the flow half back, with arrivals spread over a year | 25% | 130.8 | 129,595 | 0.112 |
| 5 | what is wanted includes what the town sends, spread over a year | 0.2% | 4.2 | 709,107 | 0.129 |

The people in a hungry town are those in towns whose households got under 82 per cent of the food they need that month, the line at which `people` counts a famine year. The price between years is the sweep's `price_swing_grain` where there was a sweep, and the same figure from the monthly run otherwise.

- **First form.** August's sales came from the store meant to last to July, and a town that eats bought grain aimed at two months all year while the towns that grow for others aimed at their whole crop, including what they would send away. The buying towns ran dry each summer.
- **Second form.** Every town of island 1 in 1850, month by month (`HARVEST_TOWNS=1:1850`): the towns that grow more than they eat held nothing in July (food 0.00 in five of fifteen towns) while a town that grows 3 per cent of what it eats held 35 months; and in July the empty towns' price was lower than the full town's. The flow half of the price rule sets the month's arrivals against the month's wants. A town that buys its year in the autumn and nothing after reads every month after as a shortage, its anchor rising eight per cent a month however full its store, and a town that has sent its crop away reads the reverse. The anchors part, and trade follows them.
- **Third form.** With the flow half left out for grain, every town holds about its line, and hunger falls to 3 per cent of people in July. But whole islands starved in some Julys: island 6 in 1877 and 1891, every town, after good harvests. The farms grew 0.9 to 1.1 million units a month in 1876 against 1.24 to 1.40 eaten, and 2.7 million in the autumn of 1877. Without the flow half the price hears of a shortfall only through the store, which a full store hides until the spring, and the farms hire on a price a year late: a cobweb of two and three years.
- **Fourth form.** The flow half back, with what a town buys spread over a year. A town that grows twice what it eats then reads a glut of a third every month (its anchor falling 2.7 per cent a month). Today that is balanced by its holding almost nothing, since it sends its grain away as it grows; with a harvest month it holds the crop for months, both halves push its price down together, the crop is sold cheap and fast in the autumn and the island eats its year by the spring. Famine in 131 years of 150.
- **Fifth form.** What a town is asked for includes what it sends, and the two months the merchants always meant to hold are counted on what the town sells and sends over a year. In an ordinary year both halves of the price stand at about nothing and the store against its line does the season's work.

## 4. What the last form moved

### Its months

`harvest_months game 6 1 8 season.enabled=true`, as in section 1, beside today's.

| month | held, today | held | eaten | price | price, today | people in a hungry town |
|---|---|---|---|---|---|---|
| January | 1.89 | 8.51 | 0.990 | -0.86% | +0.00% | none |
| March | 1.91 | 6.45 | 1.001 | -1.16% | -0.40% | none |
| May | 1.92 | 4.41 | 0.998 | -0.65% | -0.47% | 0.01% |
| July | 1.91 | 2.40 | 0.995 | +1.33% | -0.08% | 0.20% |
| August | 1.91 | 7.13 | 0.994 | +1.05% | +0.10% | none |
| September | 1.90 | 12.89 | 0.999 | +0.75% | +0.29% | none |
| November | 1.89 | 10.78 | 1.011 | -0.02% | +0.32% | none |

An ordinary year now has a store: 12.9 months of eating at the end of September, falling by a month a month to 2.4 at the end of July. The island eats within about a point of the same amount every month, and loses 2.6 per cent of its crop in store, as today. The price is dearest in July in 449 years of 1,120 (the year's turn still makes January and December the dearest or cheapest in many), but only by two and a half points between March and July: there is no carrying cost to pay for (section 2). The dearest month stands 0.158 above the cheapest (0.103 today). Hungry towns: 54 town-months in July, 11 in June and a handful in other months, over the 1,120 years of eight islands; 44 years have any (62 in July and 52 years over the whole of the runs, 1750 to 1900).

**A bad year and a good one**, as in section 1:

| | January | April | July | August | October | next January | next April | next July | next October |
|---|---|---|---|---|---|---|---|---|---|
| bad year, price | +0.9% | +1.6% | +2.2% | +3.5% | +5.8% | +9.3% | +11.2% | +17.7% | +4.7% |
| today | +2.5% | +8.4% | +11.0% | +11.0% | +10.2% | +4.4% | -5.1% | -9.1% | -8.0% |
| bad year, held (months) | 8.66 | 5.55 | 2.50 | 6.75 | 10.90 | 7.83 | 4.77 | 1.78 | 12.74 |
| bad year, eaten | 1.005 | 1.008 | 1.000 | 1.000 | 0.998 | 0.985 | 0.994 | 0.970 | 1.028 |
| good year, price | -0.2% | -1.0% | -1.4% | -2.7% | -5.9% | -9.3% | -11.9% | -18.6% | -5.4% |

The bad year is now eaten from the harvest to the next harvest. The price begins to move a little in January (half the weather is foreseen by the farms, and the flow half sees a thin crop growing), rises from the harvest, and is at its dearest the next July, 17.7 per cent above the July before; once the next harvest is in it falls back within a few months, and it does not overshoot. Eating holds up until the next July, when it is 3 per cent short and 1.3 per cent of people are in a town short of food. The real wage falls 3 per cent from January to May of the bad year, when fewer hands are hired, and is 7 per cent up in the autumn after the next harvest.

### Its sweep

`isostate-core sweep --seeds 1-8 --rules game --families 6 --set season.enabled=true --compare merged-game-f6-l` (`sweep-on5.txt`): no run failed the money check; 433 of the 446 facts differ, 9 of them starred.

| | main | harvest month |
|---|---|---|
| years with a town in famine | 0.4 [0..1] | 4.2 (7 islands of 8 up) |
| famine news | 0.5 | 5.8 |
| people in 1850 | 393,274 | 383,957 (-2.4%) |
| people in 1900 | 731,872 | 709,107 (-3.1%, every island) |
| what a wage buys, 1850 and 1900 (1750 = 1) | 1.95 and 2.60 | 2.21 and 2.63 |
| output per head 1900 (1750 = 1) | 2.65 | 2.66 |
| grain's price swing between years | 0.109 | 0.129 |
| its lag-one correlation | -0.58 | 0.00 (every island) |
| grain output's swing | 0.096 | 0.213 |
| its lag-one correlation | -0.69 | -0.29 |
| farm zigzag | 2.08 | 3.60 |
| grain's price on the year's harvest | -1.00 | -0.44 |
| the same over two years, to the year after | +0.40 | -1.15 (every island) |
| the real wage on the year's harvest | 0.29 | 0.26 |
| the same over two years | -0.23 | -0.02 |
| the worst year's fall in the real wage | -0.143 | -0.148 |
| dearths told | 8.2 | 14.8 |
| grain's share of canal traffic, 1800 | 0.223 | 0.336 (starred) |
| cloth's price swing between years | 0.039 | 0.067 (starred) |
| railway in 1900 | 2,248 km | 2,126 km |
| works standing in 1900 | 738 | 708 |

What a wage buys is no worse, and in 1850 better on six islands of eight; the long run does not move outside the spread between seeds. The saw is gone: a short harvest's dearth no longer turns into a glut the year after, because the dearth comes after the harvest and the hands it draws to the land arrive in time for the next one. A harvest 10 per cent short now makes grain 4 per cent dearer in its own calendar year and 11.5 per cent dearer by the year after, where today it makes it 10 per cent dearer and then 4 per cent cheaper than before (R46, R49: better, and still short of King's law). The facts by calendar year read a crop year poorly; they should be taken from August to July. The cost is the hungry July: a famine year on 4 per cent of years, the dearth deaths of `people` in them, and 3 per cent fewer people by 1900. Cloth's price swings harder, on every island; I did not trace it (households' budgets lean on grain's season, and wool is the other farm good). The books of account do not balance on grain with the season on (`books_goods_worst` 0.234), since nobody taught them the standing crop or the store's loss; that is an observer and was left.

## 5. What a player would see

The picture already has seasons: the corn ripens in high summer and is cut in late summer, stubble stands into the autumn, snow lies from late November. With a harvest month the economy would keep time with it.

- **In the Markets panel by the month,** grain's price creeping up from the harvest to the next July after a short crop, and falling back when the new grain is in; in an ordinary year a price that barely moves (this is the carrying cost's absence showing). Today a short crop's price is dearest in July and August and the year after is cheap.
- **A store to watch**: each town's grain in months of its eating, full after the harvest and low in July, which would be worth a line in the town's note and a map view ("Grain in store"). A town with a month in hand in June is a town about to go hungry, and the player can see it coming.
- **News in its month**: a dearth told in the spring or the summer, naming the harvest that is being eaten; a famine in July, in the hungry gap before the harvest.
- **On the ground, if the generators take it up** (not this probe's ground): ricks in the farmyards filling in August and thinning through the winter, the granaries of the towns that buy grain full in the autumn, carts and boats with grain in September and October, and the fields of a short harvest thin and pale before they are cut. The record would need each town's store by the month, which `harvest_months` already reads.
- **Decisions at the right time**: a player who holds grain, or buys a district's harvest, has a reason to do it in August and sell in June. That is the first speculative trade the game would have, and it suits the monthly ticks that the player layer is moving towards.

## 6. What it leaves out

- Only grain has a season. Wool (pasture) still comes evenly, and so do timber and everything dug.
- The farm hands' year (section 2): hiring is flat, and the farm hands still swing with a price that now swings more.
- The cost of carry: no interest and no merchant paid for holding, so no seasonal price in an ordinary year.
- A price rule for stores. The prototype is three patches to a rule for flows (a stock target that follows the season, arrivals and wants spread over a year, the carry on what is sold and sent), and its bad harvest comes too slowly. Grain's own markup above the merchants' 0.5 broke the island's money.
- The money in time: merchants pay for the crop as it grows, landlords take the weather in the rent month by month, and October to December's rent is paid on last year's weather though their grain takes this year's. No rent day.
- The books of account, the record and the live view: none of them knows the standing crop or the store's line, and the books' grain rows do not balance with it on.
- A new town starts with no store and no memory of its sales, and a district that changes market in January leaves its standing crop with the old town. Neither was looked at.
- The news: dearths are still read from the calendar year and name the wrong harvest; a July famine is told as any famine.
- The facts by calendar year (`grain_price_on_harvest` and the like) misread a crop year.
- Eight seeds, and seeds 9 to 16 were not run. The famine count (4.2 a run against 0.4) is clear on seven islands of eight; whether it is a fair price for a hungry gap, or a fault of the store's line that a better rule would remove, is for whoever builds it.
- R47 (the country's harvest varying more late in the century), R50, R51, R197 and C67 (land priced on rent as paid, a family ruined by one harvest) were not touched.

## 7. Whether to build it

Yes, as a mechanism of its own with a price rule written for it, not as this prototype.

The probe shows that the island can have a harvest month without coming apart: on the fifth form the store fills and empties as it should, the money holds on every island, the long run is where it was, and the saw of R46 is gone. It also shows that the price rule cannot simply be handed a seasonal store. Each of its halves had to be bent (the stock half to a line that follows the season, the flow half to arrivals and wants spread over a year), the first four forms of bending starved the island in different ways, and the fifth still rations a short crop too late and still has the farm hands swinging. Three patches to a rule for flows are not a rule for stores.

What to build, in order:

1. **A merchant who prices the store to last.** For a good with a harvest, the price at which, at the present rate of selling, the store reaches the next harvest with its carry: marked up as the store falls behind that, and rising through the crop year by the cost of carrying it. This replaces the three patches, and is where a bad harvest would be rationed from the day it is in (King's law, R49).
2. **The farm hands' year**: a monthly profile of the hands a farm wants, with the harvest wage it brings, and the farm zigzag settled with it.
3. **The news in its month**: dearth and the hungry gap told when they happen, naming the harvest being eaten; facts by crop year in the sweep.
4. **The store in the record and the live view**, a map view of it, and the books taught the standing crop.
5. Later, if wanted: who holds the stock (farmers' stacks against merchants' granaries), the rent day, winter on the roads and the water.

Several days for the first three, as the backlog says, and better after the scorecard exists, since the first question about it will be whether a hungry July on one year in twenty-five is a fair price. It goes well with the people's side: the famines it adds are the ones `people` was built to count.

## 8. How to run it again

```bash
cargo build --release -p isostate-core --bins --example harvest_months
# today's months, and the harvest month's (each takes a few minutes for eight islands)
./target/release/examples/harvest_months game 6 1 8 --csv DIR/months-today
./target/release/examples/harvest_months game 6 1 8 season.enabled=true --csv DIR/months-season
# every town's months of one year in one run
HARVEST_TOWNS=6:1877 ./target/release/examples/harvest_months game 6 1 8 season.enabled=true
# the sweeps
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --set season.enabled=true --out DIR/game-season --compare BASELINE
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --set season.enabled=true --set season.markup=1,2 --out DIR/game-season-markup
```

`harvest_months` prints the ordinary year month by month, the swing of grain's price within and between years, what is lost in store, and the bad and good years month by month for two years; `--csv` writes each run's months. `season.month`, `season.months` and `season.loss` move the harvest, its length and the cost of a month in store. `season.thresh=true` (with `season.enabled=true`, and `season.loss=0` to keep the island's losses where they are today) runs the smaller design of section 9.

## 9. The check: sixteen islands and a smaller design

The probe was checked before it went to `main`. Every number in sections 1 and 4 and in the sweep table came out again to the last digit on the same eight islands, and the sweep with the switch off still gives "0 differ" of 446 facts. Two sentences were wrong and are mended above: section 1 said nobody goes hungry today (there are seven hungry town-months in 1,120 years, at the year's turn), and section 4 counted July's hungry towns over 1750 to 1900 where every other figure is over 1760 to 1899 (54, not 62).

Since then `main` has taken the core's guards (in `harvest.rs` and `prices.rs`), and on the branch merged with it the season-on sweep gives the same island to the person (709,107 people in 1900) but counts its famines a little differently: famine years 3.9 a run, not 4.2, and famine news 4.0, not 5.8. With the switch off the merged branch is today's `main` to every one of its 563 facts.

**Islands 9 to 16**, swept with the season on against the same islands with it off: no run fails the money check; famine years 2.9 a run against 0.4, up on all eight; the saw gone on all eight (grain's price lag-one correlation 0.02 against -0.65); the farm zigzag 3.57 against 1.98 and cloth's price swing 0.065 against 0.039, as on the first eight; but people in 1900 only 0.9 per cent fewer (4 islands up, 4 down), so "3 per cent fewer, every island" is the first eight's figure, about 2 per cent on sixteen. On islands 9 and 10 the months come out in the same shape as section 4: a store of 13.0 months at the end of September and 2.55 at the end of July, and a bad year's dearth rising from the harvest to a peak the next July (+11.3 per cent) with no overshoot after.

**Looking for faults on sixteen islands** (each run with the season off and on, every town every month): no town's price ever reaches its ceiling and no stock goes below nothing; the standing crop is all in by the end of every September; 0.8 per cent of town-years hold under three months of sales after the harvest, and 3.7 per cent still hold more than six months at the end of July (the growing towns, holding grain beyond their own year until it is sent). The famines are a new kind: with the season off, the island's rare famine years (9 over sixteen islands) fall in January to March, at the year's turn; with it on there are 81, and their hungry town-months fall in July (93 of 127), June (20) and May (7). One thing to know: the first month fills every town's store with about seven months of its January crop "as an ordinary year would have left it", so the island opens with four times today's stock of grain, from nowhere.

**A smaller design, tried and found worse.** Could the crop be moved in time without touching the price rule? The check added `season.thresh`: the harvest goes into the farmers' stacks and is threshed into the merchants' stock by an equal share of what is left each month until the next harvest, so the merchants' stock is a flow as it is today, only the weather on it changes in August and not January. Two forms, on the first eight islands, with the stacks losing nothing:

| | main | the probe (form 5) | threshed, flow half reading what was threshed | threshed, price rule as today |
|---|---|---|---|---|
| famine years a run | 0.4 | 4.2 | 22.4 | 3.6 |
| people in 1900 | 731,872 | 709,107 | 620,607 | 720,448 |
| grain's price swing between years | 0.109 | 0.129 | 0.398 | 0.241 |
| its lag-one correlation | -0.58 | 0.00 | -0.07 | -0.27 |
| grain output's swing | 0.096 | 0.213 | 0.550 | 0.353 |
| farm zigzag | 2.08 | 3.60 | 10.7 | 6.71 |
| dearths told | 8.2 | 14.8 | 35.5 | 31.1 |

In the first, the price heard of a harvest only as it was threshed, a year late, and the farms hired on it: the cobweb of the probe's third form, worse. In the second, the price rule is today's to the letter, so its flow half reads the crop growing in the fields while its stock half reads last year's crop coming out of the stacks; the two halves read two harvests and pull against each other, and the farms swing between them. Its bad year is dear from the harvest to the next (+6 per cent from January to July after a short crop), the shape wanted, at a third of the height. Fewer famines than the probe's, but grain's price wanders from year to year twice as far. So the probe's three patches look like the least that keeps the two halves of the rule reading the same season, given the rule; and the verdict of section 7, a price rule written for stores, is the stronger for it. The commands are in section 8; the raw output and the notes of the check are with the probe's, in `check.md`.
