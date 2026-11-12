# The pass over all the rules: what the island should do, and the changes that get it there

12 November 2026. This is the design for the first pass over all the game's rules together (`docs/BACKLOG.md`, "Ready next" 10; piece 18 of `docs/backlog/found-along-the-way.md`). It is a design that decides: the questions that `docs/backlog/waiting-on-stella.md` gathers about the rules were handed on for deciding, and each has an answer here with its reason. One thing is not touched: what the player does, the game's core loop, is set aside for thinking of its own (the README, "Where it stands"), and nothing here designs or changes it. The economics of the island is what this page is about.

Nothing is built. The page changes no code, and the pieces it describes are to be built one stage at a time, each behind a switch that is off under the port and on under the game's rules, each with its page under `docs/rules/` and its sweep (`docs/rules/README.md`, "Adding a mechanism").

Three things landed on main while it was being written, after its branch was cut: the sweep's tools and their scorecard (`docs/rules/scorecard.md`), the roads prototype, switched off (`docs/probes/dx-roads.md`), and a probe of new towns (`docs/probes/dx-new-towns.md`). The page had read the first two on their branches and refers to all three where they bear.

Three kinds of evidence are kept apart throughout, as `production-tree.md` keeps them.

- **Measured on the pages.** The six tables of sensitivity (`docs/probes/sens-prices.md`, `sens-machines.md`, `sens-works.md`, `sens-ways.md`, `sens-people.md`, `sens-mines.md`) and the seven traces with the books of account (`tr-money.md`, `tr-rent.md`, `tr-works.md`, `tr-cycles.md`, `tr-steel.md`, `tr-urban.md`, `tr-carriage.md`), all of 12 November 2026, all under the game's rules with six families. Where a number comes from one of them the page is named.
- **Run for this page.** Eight sweeps of eight islands each, to settle doubts the pages left: the design's settable numbers run together, on today's island and on the island with the production tree in, and the year's room set against the land's hands. One of them changed a decision. Section 8 has every command and what each printed.
- **Reasoned.** Everything that needs new code (a freight rate that follows the wage, hands that are rationed, carriers counted as townsfolk) has not run. For each the page says what it should move, from the nearest thing that was measured, and how it will be checked.

The production tree's first stage (`docs/designs/production-tree.md`; `docs/rules/tree.md` on the branch `night/int-tree`) is reviewed and waiting to land, and it changes the island more than any one thing here: builders stand between the quarry and the house, a quarter more is made a head, and four to ten points more of the people live in towns. So the pass is designed for the island with the tree in, and section 5 says which stages go before it lands and which after.

## 1. In one page

**What is wrong, in five lines.** Growth stops about 1880 because the machines stop getting better. A cart's hire is fixed in coins while wages treble, so carriage grows cheaper for no reason, towns stop being founded, and the island is not indifferent to how much money there is. The steel rail never arrives and the dates of invention wander by a generation, both for want of a start on a learning curve. A works orders in lumps, and the lumps reach the household's wage through timber. And a few things are simply faults: a town can hire hands that do not exist, a town that takes a district is not sent its shelves, and the money is counted while a tenth of it is between purses.

**What is decided.**

| | decision | the number | stage |
|---|---|---|---|
| A town hiring more hands than it has | every employer is cut alike to the hands there are, and carries forward what it got | none | 2, before the tree |
| The merchants' stock and their measure of sales when a district changes town | they go with the district, to any town, as the people do | none | 3, before the tree |
| The issue of money | on the true total, at the rate today's island really has | `money.growth` 0.0175 | 4, before the tree |
| The settings that do nothing | nineteen become constants; two pairs that are one number are folded | section 4.5 | 5, before the tree |
| The freight rate | follows the island's money wage, so that a cartload over the same road takes the same hands in 1900 as in 1750 | the wage's mean over five years | 6 |
| The founding of towns | the test stays as it is and inherits the freight; no number moves | `people.found_saving` stays 0.08 | 6 |
| The spread of the dates of invention | a steeper and quicker chance | power 12 for 4, rate 0.5 for 0.18 | 7 |
| The ceiling on machine quality | lifted out of the century's reach | `machines.quality_cap` 12 for 4 | 8 |
| The steel rail | the steel rail mill starts with the whole of the rolling mill's skill | `machines.inherit_rail` 1.0; the converter keeps `inherit` 0.5 | 8 |
| The works' yearly room and the building cycle | the room stays where it is; the cycle is calmed from the other side, by hands on the land that follow the wage faster | `works.year_room` stays 0.15; `hands.land_adjust` 0.5 for the port's 0.15 | 9 |
| The brake on a glut | begins at the stock merchants mean to hold | `prices.glut_months` 2 for 4 | 9 |
| What "living in towns" counts | the works' households, as now, and the households of those who carry between towns | none: 0.04 and 0.8 stay | 10 |
| The Engel reference | stays where it is; it does not follow the island down | `engel.y_ref` stays 2.0 | none |
| The roads prototype | stays off in this pass; if it is ever switched on, at its own toll | `roads.enabled` false; toll 0.05 a cell | none |

**What it should come to.** Four of the numbers above can be set today without new code (the ceiling, the heir's skill, the land's hands and the brake), and they were run together with the tree in on eight islands (section 8). They give: what a head spends growing by half from the 1870s to the 1890s, where today it grows by 5 per cent; the wage's five worst years 8 per cent down, where today they are 12; timber's price swinging 0.15 a year for 0.20 and iron's 0.12 for 0.17; clay's sawtooth gone; and 0.47 of the people in towns over the last twenty years, 0.49 in 1900 itself and rising. The stages that need code should add a railway age that begins between 1820 and 1832 on every island, towns founded into the 1870s, the carriers counted, which brings the towns to a half, no hungry January, and no town that pays eighty times its neighbour's wage. Section 2 has the aims with their tolerances.

**What it costs.** Carriage that follows the wage keeps about one hand in ten on the road or the railway in 1900, where today one in sixteen is, and takes perhaps a twentieth off what the island makes a head late in the century; the lifted ceiling gives back more (7 per cent of the late wage with the tree in, 13 without). There will be more and smaller towns. The steel rail still does not win by 1890 with the tree in, and section 4.9 says why. And every stage after the tree lands moves the output the tree's thresholds are counted in, so the tree's fitter is run again after stages 6, 7 and 8.

## 2. What the island should do, 1750 to 1900

This is the scorecard the pass is judged by. Each aim is tied to a document that already asks for it, or to the plain history of an island that industrialises, and the table says which. The history is from the standard accounts as I remember them (Britain's census reports, Feinstein and Allen on wages, Deane and Cole on the national income) and none of it was checked for this page; it is there to say which side of a number is the believable one, not to fix a figure.

Every level is read as a twenty-year mean over sixteen islands, never as one year on eight (`docs/rules/towns.md`; `sens-mines.md`: by 1900 two histories of one island differ as much as two islands do). "Today" is main as it stands. "With the tree" is the tree alone, with none of this page's changes: from this page's eight-island run of it (section 8) for the figures that are twenty-year means, and from the tree's own sixteen-island table (`docs/rules/tree.md` on `night/int-tree`, "In company with the canals") for aims 9, 12, 14, 15 and 18.

| | the aim | tolerance | today | with the tree | who asks |
|---|---|---|---|---|---|
| 1. People in 1900 over 1750 | 5.0 times | 0.5 either way | 5.4 | 5.1 | `docs/rules/people.md`: `growth_top` "was set so that the island grows about fivefold". England and Wales grew a little over fivefold. |
| 2. Share of people in towns, 1751 to 1770 | 0.25 | 0.05 either way | 0.21 | 0.29 | `PLAN.md` 1.2: "from about 0.25" |
| 3. The same, 1881 to 1900 | 0.50 or more | 0.03 | 0.42 | 0.465 | `PLAN.md` 1.2: "to 0.5 over the run in most seeds" |
| 4. The towns still gaining at the end: 1881 to 1900 over 1861 to 1880 | 0.02 or more | 0.01 | 0.014 | 0.010 | The README: the share has been flat since about 1880. History: the towns' share never stopped rising. |
| 5. What a wage buys in 1881 to 1900, on the basket of 1750 (1750 = 1) | between 2.5 and 3.5 | 0.15 beyond either end | 2.64 | 3.11 | The README's own headline; `PLAN.md` 1.3 asks for "the real wage's path" with a tolerance and gives none. History: Britain's about 2.3 to 2.6 times. |
| 6. Growth that goes on to 1900: spending a head over the chained cost of living, the 1890s over the 1870s | up 10 per cent or more, on twelve islands of sixteen | 3 points | up 3.6 per cent on sixteen islands (`tr-works.md`) | up 29 per cent | The README: "Growth comes early and then stops." History: Britain's output a head grew by about a fifth in those twenty years, Germany's and America's by more. |
| 7. The wage's five worst years, their mean | no worse than 10 per cent down | 1.5 points | 12.0 | 11.4 | The README: "a building cycle nobody chose". History: a dearth year took a tenth to a seventh off what a wage bought. |
| 8. Timber's price, its yearly swing after 1800 | 0.15 or less | 0.02 | 0.20 | 0.23 | `docs/rules/works.md`, "In company": the building cycle |
| 9. Iron's price, its yearly swing | 0.16 or less | 0.02 | 0.174 | 0.150 | `docs/rules/prices.md`, "Measuring it again" |
| 10. The farm zigzag beyond the weather | 1.2 or less | 0.25 | 1.27 | not measured (the plain zigzag is 1.83 for today's 2.08) | `docs/rules/prices.md`'s aim of 1.2, moved to the fact that still answers its question |
| 11. Years in which tools cost over twice the middling price of the twenty-five years about them | 4 or fewer | 2 | 8.5 | 0.25 | The README: "Tools run short". The window is there because with the tree tools grow cheaper all century, and a count against the century's middling price reads that fall as a famine. |
| 12. Hunger: famine years in a run | 0.2 or fewer, and none in the two months after a town takes a district | 0.2 | 0.38 | 0.1 | The README; `docs/rules/people.md`. History: dearths, and no famine after the 1740s. |
| 13. A town paying over ten times the middle town's wage | no town-month in sixteen runs | none | one history in sixty-four | not measured | `docs/rules/mines.md`, `machines.md` and `prices.md`, each among its worries |
| 14. The first railway opens | 1826, and between 1820 and 1832 on fourteen islands of sixteen | 2 years on the mean | 1827, between 1809 and 1842 | 1830 | The README: "anywhere from 1809 to 1842". History: 1825 and 1830. |
| 15. Railway in 1850; still growing in the 1890s | 1,200 to 1,700 km on every island; on every island | 100 km | 575 to 1,804 km; yes | 1,187 km on average | `PLAN.md` 1.2: the network "keeps growing after the first rush" |
| 16. Converter steel's share of steel in 1880 | 0.9 or more on every island | none | 0.86, and nothing on one island of eight | 0.94, and 0.54 on one island of eight | `docs/rules/machines.md` |
| 17. Steel rails' share of the rails made in 1890 | a half or more on fourteen islands of sixteen | none | 0.002 | nothing | The README: "The steel rail hardly arrives". History: nearly all new rail by 1890. The pass is expected to miss this one with the tree in (section 4.9). |
| 18. The last town founded | after 1860 on twelve islands of sixteen; 18 to 26 towns in 1900 | 2 towns | 1847; 17.8 | 1870; 16.6 | `docs/rules/people.md`: towns "go on being founded where a market would save the most carting" |
| 19. Hands carrying between towns (carters, boatmen and railwaymen on the trade between towns and the works' own carting) | about 2 per cent of all hands in 1750, 5 per cent or more by 1900 | 1 point | 2.0 and 3.3 | not measured | History only: carriage was a growing trade all century, about one worker in eleven by the census of 1901. Section 4.6 says why it is an aim. |
| 20. Coke iron passes half of iron | within ten years of its discovery | none | 7.8 years | not measured | `docs/rules/prices.md`, "Measuring it again" |

Four things about the card itself, each a decision.

- **The tolerances are written beside the aims and held still.** The scorecard as it landed (`docs/rules/scorecard.md`) has four aims in its loss, the four the documents already stated with a number, and takes each tolerance from the spread between the islands of the sweep being scored, and says itself that a search could lower its loss by making the islands differ more (`scorecard.md`, "What worries me"). With the tolerances above in `scorecard::AIMS` that worry goes, and so does the other one: today 94 per cent of the loss is the towns' share, because that fact's spread between islands is under one point.
- **The zigzag's aim moves to `farm_zigzag_beyond_weather`.** The aim of 1.2 was set for an island without weather; with the harvests on, the plain zigzag is 2.0 and six tenths of it is the weather's (`scorecard.md`; `sens-prices.md` found the same by switching the harvests off: 1.12). The fact that still answers the page's question reads 1.27 on sixteen islands, which is inside the tolerance.
- **Growth is judged on the chained index, and the level on the basket of 1750.** The observer's `gdp_pc` is deflated by the basket of 1750, which is heavy in timber, wool and grain, so it turns a stall into a fall and moves the peak about a decade early (`tr-works.md`, section 1). Aim 6 is therefore on spending a head over the chained cost of living. Aim 5 stays on the old basket because that is what histories of the real wage are reckoned on, and because the port has no other. The game should tell the player the chained figure and say so; that is a line for the charts' own piece ("Ready next" 4), not for this pass.
- **Nothing on the card is about the families.** Who is rich, what the player earns and whether land beats works belong to the core loop.

Facts the card needs and the sweep does not yet have are listed in section 5, stage 1.

## 3. How a sweep is read in this pass

The tables of sensitivity agree on how not to be fooled, and the pass follows them.

- **Sixteen islands, twenty-year means.** A single year's wage on eight islands cannot tell a movement the size of the difference between two islands from nothing (`sens-works.md`: two placebos each moved the chained wage of 1900 by 4 per cent on seven islands of eight).
- **A null pair with every stage.** Two runs with a number moved by a millionth (`works.walk=0.0200001`, `carting.adjust=0.2500001`) beside the stage's own, so that the stage is read against three histories of today and not one. The baseline of seeds 1 to 8 happens to sit about one late town above its own cloud (`sens-people.md`, `sens-ways.md`).
- **High against low** wherever a number is moved, island for island, so that the baseline's luck drops out (`sens-mines.md`).
- **A stage that mends a fault is read on the fault's own measure first** (a count of wild town-months, of hungry Januaries, the money's rate to four figures), and on the card second. On the card it should move nothing beyond the null pair.
- **The commons scenario is run with every stage and read only for failures.** Its railway moves by 39 per cent from one history to the next (`sens-mines.md`), so no fact of it is a finding without its own null pair.

## 4. The changes

Each has the fault with the page that measured it; the mend, as a rule in words and in the code's terms; the number chosen and why; what it is expected to move; what could go wrong; and how it will be checked. Names of switches are proposals for whoever builds them; what matters is that each is off under `default_params()` and on under `game_params()`. `game_params()` sets only the mechanisms' own structs and never one of the port's numbers, so every new number here lives in a mechanism's struct.

### 4.1 A town cannot hire hands it does not have (stage 2)

**The fault.** Each month a town's wage may rise or fall twofold at most (`econ::step_month`: the bracket is half to twice last month's wage, and never over 500). The land's hands move only `labor_adjust_ex` (0.15) of the way to what their employers want, so most of this month's demand is last month's hands, whatever the wage. When those carried hands alone are more than the town has, no wage in the bracket clears the market, and the code puts everyone asked for to work and pays them. The wage then doubles every month until the land has shed enough. Halhope's on island 2 under the port went 8.9, 17.8, 35.6, 71, 142, 284 and 500 from January to July 1844, and its landlords ended the year 4.3 million below nothing, which took nineteen years of rents to pay off (`tr-rent.md`, section 1). Under the game's rules the people rule took away the commonest way into it (a district redrawn at the year's end) and not the thing itself: thirteen runs of 528 had such a year once any number of the works or of carting was moved, to 47, 100 and 121 times the middle town's wage (`sens-works.md`, "Where a run breaks"); one null history of sixty-four has a town at 84.5 times, with eight town-months in which half as many hands again were at work as the town has people (`sens-mines.md`). On island 4 the island's money level shifted for good after it. It turned up in a run for this page too: with the year's room at 0.10, one island of eight has a year in which what a wage buys falls 73 per cent (section 8). It is the one result in the tables that is a fault and not a setting, and it colours every other table, because one such run carries the mean of eight.

**The mend.** No employer can hold a hand the town does not have. If at the highest wage the month allows the hands asked for are still more than the town has to spare (its people of working age, less its carters, its sinkers and its navvies, as now), the wage is set at that highest wage and every employer in the town, farm, wood, pit and works alike, is cut by the same share, to the hands there are. What an employer carries into next month is what it got, not what it asked for.

In the code: a small mechanism of its own, `hands.rs` with `HandsParams { ration: bool }`. One hook in `econ::step_month` after `bisect_log_wage`: where demand at `hi[t]` exceeds `supply[t]`, the town's `Lex` and `Lf` are multiplied by `supply[t]` over that demand before `st.ex_Lprev` and `st.f_labor` are written. Nothing else changes: production, wages paid and rents follow from the hands as they do now.

The second half is what cures it and not only hides it. Because the carried hands are cut to what exists, next month's demand at any wage is at most the town's supply plus 0.15 of the gap to what is wanted, and the land wants hands as the third or fourth power of its price over the wage, so a wage a few tenths higher clears the market. The wage rises for a month or two and stops.

**The number.** None.

**What it should move.** The wild town-months go: none over ten times the middle town's wage, and none in which more hands are at work than there are people. The landlords' spells below nothing go with them, and the millions paid to households for work nobody did. `real_wage_worst_year` stops being one town's year on the islands where it was (island 4 at the smaller room: 71 per cent down, the other seven 13). On the card nothing should move beyond the null pair: `wage_watch` counts no overworked town-month on islands 3 and 4 at the game's own values, so on most islands the rule never acts.

**What could go wrong.** It changes the labour market for everyone, the landlords' and the families' land included, which is why `mines.md` left it. A town whose supply is nothing or less after its carters are taken needs its own line (nobody is hired). Under the commons the common fund carried these losses; it should carry none now, and `mines::fund_to_share`, which was written for the debt they left, should find nothing to do.

**How it is checked.** `examples/wage_watch.rs` on the runs that had it: `game 6 on 3 4 works.year_room=0.1125` (Brambank, 121 times; Sunhaven, 47), `game 6 on 11 11 carting.labor=0.5` (Melford, 100), `game 6 on 9 9 mines.sink=40.0002` (Iveldelf, 84.5) and `game 6 on 6 6 mines.reach=1.5` (Cranbourne, 16): each under four times the middle town's wage, and no overworked town-month. `trace_rent --seeds 1-16 --part a --set '{"people":{"enabled":false}}'`, which today brings 154 spells of landlords' debt back: with the rule on there should be a handful at most, which tests the mend apart from the people rule. Then the stage's sweep on sixteen islands with its null pair, and both regimes.

### 4.2 The shelves go with the district (stage 3)

**The fault.** When cells change market, their people go with them, and so do their cash, last month's income, their carters and the going price (`peopling::move_with_cells`). The merchants' stock, their cash and their smoothed measure of sales go only when the receiving market is new (`if st.town_new[j]`). An old town that takes a district keeps the shelves and the order book of the town it was. Uldale on island 1 went from 3,257 people to 13,637 at the turn of 1848 and bought January's bread from a store laid in for three thousand (`tr-carriage.md`, (d)). Of 30 hungry town-years on sixteen islands with and without canals, 23 follow a town at least doubling, and more than half of the hungry months are Januaries. That is also why famine years fell from 2.2 a run to 0.4 when the canals came in: the canals redraw the districts earlier and by less, so the railway's redrawing is gentler. It is not grain reaching the hungry by water (the same trace, switched both ways). And most of the game's emigration is the same January: 114 people a run sail today and 8 with the mend (`tr-rent.md`, section 5).

**The mend.** The merchants who served a district go on serving it. Their stock of every good, their cash above nothing and their measure of sales go with each share of a town's people that changes market, to any town, as they already do to a new one.

In the code: the condition `if st.town_new[j]` in `peopling::move_with_cells` is dropped under `people.shelves_go`. The books of account must be told: with the trial the goods balance (`books_goods_worst`) went from 4e-15 to 6e-11 because the books' `moved` column does not hear of stock that goes to a town that is not new (`tr-rent.md`, section 5; `docs/rules/accounts.md` says a mechanism that moves goods has to tell them).

**The number.** None. Both halves are needed: with only the stock moving four hungry town-years of nine remain, in spring, because the merchants still order for the old trade; with only the measure of sales, six remain, three of them Januaries (`tr-rent.md`, the check). Whether the merchants' cash goes too is the one thing the two traces did differently: `tr-rent.md` left it behind and `tr-carriage.md` sent it. It goes, because one rule for a new market and an old one is simpler to read and to hold in a test, and because the trace that sent it was run on the most islands (thirty-two runs here and thirty-two more in review).

**What it should move.** Hungry town-years from 9 to 1 on sixteen islands, and from 5 to none on seeds 17 to 24 (`tr-rent.md`); famine years from 0.38 a run to 0.06 or none; emigrants from 114 a run to about 8; no hunger after any doubling, with canals or without (`tr-carriage.md`: none after 38 doublings with canals and 34 without, and none after 32 and 55 on seeds 17 to 32). Output a head and the real wage of 1801 to 1850 stay inside the spread (5.63 for 5.66, and 1.752 for 1.761, each higher on about half the islands).

**What could go wrong.** The histories part after the first redraw, so every fact moves a little, as with any change. The dearth's and emigration's numbers, which have nothing to work on today, will have less. And the mend does not reach the hunger that is not this: three of the five hungry town-years on seeds 17 to 24 came in spring with no jump in people (section 7, task 6).

**How it is checked.** `trace_rent --seeds 1-16 --part d` and `--seeds 17-24`: no town short of food in the January or February after it took a district. `trace_carriage --seeds 1-16 --set canals.enabled=true,false`: no hunger after a doubling either way. The books balance to rounding again. The sweep, with `famine_years`, `emigration_years` and `news_emigration` read first.

### 4.3 Money issued on the true total, at the rate the island has (stage 4)

**The fault.** `money_growth` says 2 per cent a year, and the money grows 1.74 to 1.78 per cent (`tr-money.md`, section 1). The month's issue is reckoned on a total taken while the households' whole income for the month (their pay, their carting, both kinds of dividend: 11.5 to 14.7 per cent of the money) is held in a local vector and counted nowhere. The share left out is not a constant: it rises with what the merchants owe, to 0.21 of the money under large harvest shocks, where the money then grows 1.53 to 1.72 per cent a year. And a month whose incomes are larger than the money issues less than nothing; enough such months take the money below nothing, where the monthly check's allowance meets the rounding of very large purses. Every setting that still fails the money check fails by that road and none by a leak (`tr-money.md`, section 2).

**The mend.** The issue is the rate over twelve times the money as it stood when the month began.

In the code: a mechanism `money.rs` with `MoneyParams { true_total: bool, growth: f64 }`. In `econ::step_month`, under "dividends, common fund, issuance", the total the issue is reckoned on is `econ::money_total` taken at the top of the month (the trace added back what is in flight, which is the same number), and the rate is `money.growth`. The port's `money_growth` is not read once the switch is on, and under the port nothing changes.

**The number.** `money.growth` 0.0175, not 0.02. Issued monthly that is 1.764 per cent a year, which is the rate today's islands really have, on every island alike. The money of December 1900 comes to about 37.5 million where today's islands have 36.4 to 38.5.

Why not the 2 per cent the old setting names: with it the money of 1900 is 56 million, prices and money wages stand a third higher, and nothing real is gained (`tr-money.md`: what a wage buys is 0.98 to 1.05 of today's on sixteen islands). The island was tuned, mechanism after mechanism, on the money it really had. On that money the wage in coins rises 2.9-fold over the century and the basket of 1750 ends a tenth dearer than it began (section 8, the first column), which is about what Britain's wages and prices did. So the mend is to make the rate what the setting says, and then to say the rate that was being used.

**What it should move.** The money is the same on every island in every month. The real facts should move by no more than the null pair. Prices by decade should be within about 3 per cent of today's: the old rule issued a little faster early in the century, when a ninth of the money was left out, and a little slower late, when a seventh was. The extreme settings that fail the money check (`machines` with `quality_b` 2, `quality_cap` 50 and `quality_Q0` 0 on islands 3 and 7; `prices.markup` -0.5) pass it, as they did in the trace's trial, because money that only grows cannot go below nothing.

**What could go wrong.** Money is not neutral on this island: a point more of new money a year makes it 7 or 8 per cent richer by the 1850s (`tr-works.md`, section 1). At the same average rate that slope does nothing, and stage 6 takes away the one road found for it; section 7, task 1 is the test that it is then gone. The check's allowance (a millionth of the net money, where the rounding grows with the purses' gross) still cries wolf on purses of 1e18; that is a line in the guards and tests, not in this pass.

**How it is checked.** `examples/money_trace.rs --seeds 1-16 --rules game --families 6`: 1.764 per cent a year to four figures on every island and nothing left out of the count. The same under the port: unchanged, and the golden test passes. `money_trace --seeds 3,7 --no-check` with the machines' extreme setting, and `--seeds 1-4 --set '{"prices":{"markup":-0.5}}'`: no month fails. The sweep with its null pair.

### 4.4 Land is priced on the rent it paid (stage 5)

**The fault.** `harvest::average_rent` adds back to a cell's rent the crop the weather took, at that year's price, so that land is priced on an average year. After a harvest more than 6 per cent short it puts the island's land 13.5 per cent above what an average year pays; after one that much better, 12 per cent under. The rent as paid is within 2 per cent both times, because with half the weather foreseen fewer hands are hired for a thin crop and dearer grain makes up about half of the rest (`tr-rent.md`, section 3). Only `families::land_price` reads it: families paid 9.4 per cent too much for land bought after the worst harvests and 10.6 per cent too little after the best.

**The mend.** `average_rent` returns the rent paid, under `harvest.rent_as_paid`. It was tried on sixteen islands: no fact is starred, and nothing leans that eight islands could tell from a run gone another way.

**Why it is here and not with the core loop.** It changes no choice a player has and no rule of the families. It makes one price truer, the one their land is valued at. A family's worth, which is the game's score, stops rising by a tenth in the year after a bad harvest. If `harvest.foreseen` is ever moved from 0.5, or grain's price is made to answer a short harvest more sharply, this has to be looked at again: `tr-rent.md` says how the right correction hangs on both.

### 4.5 The settings that do nothing, and should go (stage 5)

A setting goes if it is a second name for another number, or if the tables found it moving nothing across the whole range they tried (for most of these, half and double its value) and it is not a guard against a setting far out of range. "Goes" means it becomes a constant in its module with the value it has, so no run changes: the stage's sweep must say that 0 facts differ. The port's own numbers stay whatever they do, because the port's `params.json` is held byte for byte.

| setting | what the tables found | what becomes of it |
|---|---|---|
| `canals.induced` | the same dial as `canals.hurdle`: the test is `induced * saving / cost >= hurdle` and nothing else reads either but the news (`sens-ways.md`) | folded into `hurdle`; the news quotes `saving / cost` |
| `mines.payback` | one number with `mines.sink` for opening: a working opens when its royalty would repay `sink / payback` wages a month, and moving both together moves nothing, on sixteen islands (`sens-mines.md`) | restated as `mines.open_at` (10 wages a month) and `mines.sink` (40), which then mean different things |
| `railways.max_projects` | never binds above 30; at 80 every fact is the baseline's (`sens-ways.md`) | constant, 40 |
| `canals.max_projects` | binds for a year on a few islands; nothing clear | constant, 12 |
| `canals.districts`, `branch_gap`, `district_spacing`, `cool_years` | the canals are nearly all trunks, so the branch planner's dials have nothing to work on | constants |
| `canals.navvy_max_share`, `navvy_adjust` | the navvies are at most 1,300 people on an island of 200,000 | constants |
| `mines.cell_limit` | nothing at 0.375 and 0.667, and nothing at 0.25 and 0.833 (`sens-mines.md`) | constant, 0.5 |
| `carting.max_share` | the cap binds in a few town-months even at 0.2 (`sens-works.md`) | constant, 0.5 |
| `people.found_cells`, `found_spacing` | of the founding rule's five gates these two never decide (`sens-people.md`) | constants |
| `people.growth_floor` | births less deaths never fall to 3 per cent a year in any town | constant: it is a guard |
| `towns.decline_years`, `back_span` | nothing at half and at double | constants |
| `prices.glut_floor` | nothing at 0.1, 0.25 or 0.5 (`sens-prices.md`) | constant, 0 |
| `chronicle.at_least`, `fill_floor`, `base_years` | no decade is thin enough for the first two; the third moves one milestone on one island (`sens-mines.md`) | constants |

Kept, though the tables found them quiet, each for a reason:

- `works.built_share`: nothing between 0.01 and 0.5, but at 0 the port's shelter comes back and with it 1,772 small works (`tr-works.md`, section 3). It is a switch in a number's clothes and stays.
- `works.walk` and `carting.adjust`: what each is for (hands near their market; a town's wage that flips month about) is not among the sweep's facts. They stay until stage 1's facts can see them.
- `mines.year_room`, `mines.memory`, `mines.idle_years`: quiet singly, but `mines.md` warns that hasty closing with unlimited opening rings, and that pair was not run.
- The dearth's three numbers, `people.emigration` and `people.arrivals_max`: idle because the island is fed. They are the rule for hungry years and are wanted the day something makes it hungry.
- `rail_hurdle` with `rail_induced`, and `rail_know`: one number under two names, and one that does nothing, but they are the port's. `docs/rules/railways.md` should say that the first two are one.
- `machines.adopt_wagonways`: not idle at all. Switched on it leaves all sixteen islands with no railway (`sens-machines.md`), because the rails the collieries lay at a loss are what teach the island to roll them. It stays off, and its page should say so in its first line.
- `chronicle.floor` and `own_great`: the player's place in the feed is kept at `floor`, so a floor above `own_great` empties it. A test should hold `floor < own_great`; neither number moves.

### 4.6 The freight rate follows the wage (stage 6)

**The fault.** A cartload costs `freight` (0.015) coins for each unit of weight and of path cost, and no rule ever moves it. Under the game's carting rule the whole carting bill is carters' wages, and the carters are the bill over the wage (`docs/rules/carting.md`). The money wage rises 2.9-fold over the century. So in 1900 the same load over the same road takes about a third of the hands it took in 1750, before any road, canal or railway is counted: carters appear to have grown three times as able by doing nothing. Three things follow.

- *Founding stops.* A town is founded where a market would save `found_saving` of its district's wages in carting. The carting is priced in fixed coins and the wages in coins that grow, so the best site's saving falls from 0.21 of wages in the 1750s to 0.05 in the 1880s; measured against 1750's wage it never falls under 0.13 (`tr-urban.md`, (b)). The last town is founded in 1847.
- *The island is not indifferent to money.* Issue a third more money and carting is a quarter cheaper in real terms, there are fewer carters, two or three fewer towns and a tenth less canal (`tr-money.md`, section 1). The freight is the one constant in coins that the level of prices reaches the real island through.
- *Carriage shrinks as a trade when it should grow.* Hands carrying between towns are 2.0 per cent of all hands in 1750 and 3.3 per cent in 1900 (from `tr-carriage.md`'s shares of the bill and `tr-urban.md`'s carters). In Britain carriage was one of the century's growing trades.

**The mend.** Carriage is priced in hands. A unit of weight over a unit of path cost takes the hands it took in 1750, and costs what those hands are paid now.

In the code: `carting::freight(st, P)` gives `P.freight` under the port, and under `carting.freight_by_wage` gives `P.freight` times the island's wage index. The index is the people-weighted mean of the towns' yearly mean wage (`wbar`, which `sim::yearly` already has) over the last `carting.freight_years` years, over the same mean of the warm-up's last years, so that in January 1750 the rate is the port's 0.015 exactly and the island of 1750 is the one everything was tuned on. It is worked out once at the year's end and held for the year, so that the cost fields, the land's bids and a works' site sums all see one rate. Every reader of `P.freight` calls the function instead: `econ.rs` (the produce's net price and the trade between towns), `firms.rs`, `land.rs`, `mines.rs`, `peopling.rs` (the founding test and the going price), `prices.rs`, `rail.rs`, `ways.rs` and `canals.rs`. The handling charge is 3 per cent of a price and follows prices already.

**The numbers.** The wage, not the price level. Under the game's own rule a cartload is so many months of a carter's time, and what does not change when the road does not is the time. A rate that followed prices would leave the hands a load takes falling by the whole of the real wage's rise, 2.6-fold, which is most of the fault left in. The island's wage and not the paying town's: a town's own wage can still jump when it takes a district, and a rate that jumped with it would move the district's bids in the same year. `freight_years` 5: long enough that a bad year's wage does not reprice every road, short enough to follow a century's drift within a few per cent.

**What it should move.** Nothing in 1750, little before 1800 (the index is about 1.4 by the 1780s) and more each decade after: on today's island 1.8 in the 1840s, 2.7 in the 1880s and 3.0 in the 1890s (`tr-urban.md`'s table of the money wage). With the tree in, prices fall further and the wage in coins rises less, 2.2 to 2.7 times over the century by this page's runs (section 8), so the index ends lower and the rule bites less. No run has a freight that rises, so what follows is read from the nearest things measured, all on today's island: the freight doubled from 1750 (`tr-urban.md`), the founding test alone put in 1750's money (the same page's trial), and the freight raised 1.3 times under the money mend (`tr-money.md`).

| | today | freight doubled from 1750 | the founding test alone at 1750's wage | expected with the freight by the wage, on today's island |
|---|---|---|---|---|
| carters, of all hands, 1900 | 0.064 | 0.093 | | 0.09 to 0.12 |
| of them between towns and for the works | 0.033 | | | 0.05 to 0.06 |
| towns in 1900; the last founded | 17.8; 1847 | 23.1; 1880 | 24.0; 1878 | 22 to 25; the 1870s |
| railway in 1900, km | 2,248 | 2,545 | 2,455 | 2,450 to 2,700 |
| works in 1900 | 738 | 823 | 852 | 800 to 880 |
| share of people in towns, 1900 | 0.419 | 0.405 | 0.413 | 0.40 to 0.41 by today's count (section 4.12 changes the count) |
| baskets a head in 1900 | 8.76 | 8.29 | 8.67 | 5 to 8 per cent fewer |

The last row is the cost, and it is real: about four points of the island's hands stay in carriage that today leave it, and the railway's own gain shows what four points are worth (the 3.6 points it frees make output a head 8.3 per cent higher: `tr-carriage.md`, (a)). The island was that much richer than it should have been. With the tree in the cost should be smaller, since the index ends lower and there is a third of the stone to cart, and stone is the heaviest thing carted. Section 4.9 gives back about as much with the tree in (7 per cent of the late wage) and twice as much without it, by an honest road.

**What could go wrong.**

- *The railways and canals overbuild.* A promoter's saving is `freight` times the carting saved, so it now grows with the wage while a line's materials do not: the freight doubled gave 13 per cent more line. If the railway of 1900 passes 2,700 km on the sixteen-island mean, `rail_hurdle` goes from 0.06 to 0.08, which takes 140 to 290 km off the late branches on fourteen or fifteen islands of sixteen and leaves output a head and the wage where they were (`sens-ways.md`). The hurdle is the port's number, so the game needs its own copy in `RailwayParams`.
- *The early century's targets.* Carting already costs the island an eighth of its households' budget in 1800. The freight by the wage adds to that from the 1780s. `carting.labor` is the dial for the first fifty years and does not move 1900 (`sens-works.md`); it is not moved here, and section 7, task 5 says what would settle whether it should be.
- *A slow feedback.* Dearer carriage raises delivered prices, and through them the wage in coins, which raises the freight. The money is fixed, so it cannot run away, but the first sweep should watch the index against today's money wage.
- *Every number tuned in coins against a wage that rose.* The founding test is the known one. The canals' and railways' hurdles, the mines' `haul` and the going price that goes with trade all multiply by the freight and so follow it; I found none that compares the freight with a fixed number of coins.

**How it is checked.** The carters a unit of weight and path cost takes (the carting bill over the wage, over the year's weight times path cost, which `ways.rs` already adds up) is level across the century on every island to within the smoothing. `tr_urban --seeds 1-16`, part (b): the best site's saving over wages no longer falls with the money wage. The sweep on sixteen islands against stage 5 with the tree in. And task 1 of section 7: the same islands at three rates of new money.

### 4.7 The founding of towns, once freight follows the wage (stage 6)

**Decision: the test and its numbers stay as they are.** `people.found_saving` stays 0.08, and the donors' limits stay. The fault was never in the test. A market is founded where it would save a twelfth of a district's wages in carting, and with carriage priced in hands that sentence means in 1880 what it meant in 1750.

**What it should give.** On today's island, 22 to 25 towns in 1900 where there are 18, and the last founded in the 1870s (the table above). With the tree in there are fewer towns to begin with and later ones (15.6 in 1900 on this page's eight islands and 16.6 on the tree's sixteen, the last founded in 1862 and 1870), and the wage in coins rises less, so the freight should add fewer than it adds today: aim 18's 18 to 26 is where it should land. `tr-urban.md` says plainly what an island that goes on founding is like: more and smaller towns (the middling late town 16,700 people for 20,300), a sixth more works, a tenth more railway, a few per cent more people, households no richer or a per cent or two poorer, and a share living in towns that does not rise.

**A caution that landed while this was written.** `docs/probes/dx-new-towns.md` measured the founding rule on sixteen islands and found that its test counts what a district's producers save and not what the island then pays to carry between the new market and the old: counted for the island, nearly every founding costs carting on the day (a late quarry town's producers save 0.10 of its wages and the island pays 0.30), and the carters' share of hands is least near today's count of towns. So towns founded on into the 1870s are for the map and the news, and the island pays for them: a per cent or two of baskets a head, by `tr-urban.md`'s table. I keep the decision with that in view, and the bound at the end of this section is what holds it. Two things soften it. Half of today's late foundings are quarry towns (2.4 of 4.6 a seed), and the tree leaves a third of the stone to dig. And what that probe says to build in the test's place (a margin before a district changes market, villages at pit heads) is the backlog's piece on new towns and not this pass.

**Why not fewer towns.** 0.08 could be raised to hold the count near 18, and `sens-people.md` has the slope (0.107 gives a fifth fewer). I do not, for three reasons. A town every six or seven years to the 1870s is what the people rule was built to give, and what the news and the map want. The cost is small and known. And the count wants reading after the tree, which founds fewer towns and later ones without any of this (16.6 in 1900, the last in 1870). If the sixteen-island mean passes 26 towns, or the late wage against a run without the late towns is down by more than 3 per cent, `found_saving` goes to 0.10 and the page says so.

### 4.8 The spread of the dates of invention (stage 7)

**The fault.** The yearly chance of finding an invention is `discovery_rate * p^4 / (1 + p^4)` at progress `p` towards its thresholds: 0.09 a year at the threshold, 0.035 at seven tenths of the way, 0.18 well past it. It is shallow on both sides. An invention has a fifth to three fifths chance of coming before its threshold, and after it waits of ten to thirty years happen (`tr-steel.md`, section 3). Across sixteen islands the year a threshold is reached has a spread of 0.6 to 12 years and the year of the find 5.5 to 16. The first railway opens 8.3 years after the locomotive, so its date is the draw's: 1809 on one island and 1842 on another, with 575 km of line in 1850 here and 1,804 there. And a late draw can cost an island its century: crucible steel found after about 1778 mostly stalls, the converter then comes too late, the machines never improve, and island 3 ends with a late wage of 1.8 where the game gives 2.6, in about one history in four (`sens-machines.md`; `tr-steel.md`, section 6).

**The mend.** A steeper and quicker chance, `rate * p^power / (1 + p^power)`, so that an invention comes within a few years of the island being ready for it and hardly ever before.

In the code: a mechanism `InventionParams { power: f64, rate: f64 }`, read in `tech::discovery` in place of the literal 4 and `P.discovery_rate`, and in `canals::knowledge`, which has its own copy of the same hazard and today reads the rate and keeps the 4. Off, it is the port's 4 and 0.18 to the last digit.

**The numbers.** Power 12 and rate 0.5: a chance of 0.25 a year at the threshold, 0.03 at eight tenths of the way and 0.003 at 0.65 of it. `tr-steel.md` ran six settings on sixteen islands. A quicker chance alone brings everything five to sixteen years sooner and leaves the railway no narrower. A steeper one alone ends the flukes and keeps the slow wait, so one island has no line in 1850. The two together keep the mean dates within one to five years of where they are and cut the spread to a third:

| sixteen islands (`tr-steel.md`) | today | power 12, rate 0.5 |
|---|---|---|
| the mean spread of the ten inventions' dates, years | 10.1 | 3.7 |
| the first railway opens | 1827.1, between 1809 and 1842 | 1825.8, between 1820 and 1832 |
| km of line in 1850 | 1,348, between 575 and 1,804 | 1,459, between 1,185 and 1,718 |
| converter steel's share of steel, 1880 | 0.93, nothing on one island | 1.00 on every island |
| crucible steel found | by 1784 | by 1770 on every island, and none stalls |
| what a wage buys in 1900 | 2.61 | 2.72 |

Power 12 with rate 1.0 is a little tighter still (3.4 years) and no better for the railway. It was the first steep number tried and no other was searched; the dates are within the aim, so I take it. What spread is left is the island's own: an island poor in coal reaches a threshold later, and that is honest.

**What it should move, with the tree.** The tree's inventions scatter by forty to fifty years for the later ones (gas light in 1808 on one island and 1863 on another), and on a quarter of islands the engineering works is not built by 1900 (`tree.md`, "Discovery dates scatter"). The draw's part of that goes. The island's part stays, and a threshold counted against an island's own 1750 would narrow it further; that is the tree's to decide and is not done here.

**What could go wrong.** The tree's thresholds were fitted so that each invention's mean date falls on its aim under the old chance, and the fitter (`parity/tree_fit.py`) works the hazard out for itself. Under the new one every date comes one to five years sooner. So the fitter is taught the hazard's two numbers and run again as part of this stage, and the tree's page has its table of dates made again. The town an invention is found in is still drawn by the square of its share, so half an island's inventions are found in one town (`tr-steel.md`): that is left alone, as a matter for the news and the map.

**How it is checked.** `steel_trace --seeds 1-16`: the spread of each invention's date, and the year found against the year its threshold is reached (within one to three years). `rail_first_year` between 1820 and 1832 on fourteen islands of sixteen. Crucible steel a year in 1800 to 1830 over 20,000 units on every island. Then the tree's own table of "found, first, half" on sixteen islands.

### 4.9 The ceiling on machine quality, and the steel rail (stage 8)

**The fault, first half.** The machines of the farms, quarries, pits and mines are worth more as machines get better, and their quality is counted in the steel the island has poured: `(1 + Q / 1,000,000)^0.6`, and no more than `machines.quality_cap`, 4. Quality comes within one per cent of 4 between 1862 and 1885 on fifteen islands of sixteen, and the methods' own learning has been at its cap since the 1840s, so after about 1870 nothing on the island is getting better at anything. Output a head, counted in goods, grew by a quarter from the 1850s to the 1870s and by 3 or 4 per cent from the 1870s to the 1890s (`tr-works.md`, section 1). Two pages found the cause independently. With the ceiling at 3, 4 and 5.33 the late wage is 2.43, 2.68 and 2.92; with none in reach the wage of 1900 is 3.21 for 2.60 and output a head peaks in 1895 for 1879 (`sens-machines.md`). On twenty-four islands, growth over the last two decades is 17 to 20 per cent with the ceiling at 8 and 4 to 6 per cent at 4, and 8 is the same as none, since quality reaches only 7.07 by the 1890s (`tr-works.md`). People, the money rule and the works' willingness to invest each change the level or the timing and none changes the stall.

**The mend.** `machines.quality_cap` 12 under the game's rules. One number.

**Why lift the ceiling and not reshape the curve.** `docs/rules/machines.md` asks for a curve that flattens without a ceiling, and the curve already is one: a power of 0.6 on all the steel ever poured gives less for each further ton. The wall was the fault, not the curve. A flatter curve was tried (`quality_b` 0.45 with no ceiling in reach): it moves the peak to 1893 and lowers the wage all the way, 2.51 late for 2.68 (`sens-machines.md`). It is a different island, poorer in mid-century, and nothing asks for that. A ceiling out of reach keeps the game's own run to the 1860s, year for year, and then lets it go on.

**Why 12.** It has to be out of the century's reach on every island, with the tree and without. Run for this page (section 8): on today's island a ceiling of 8 and one of 16 give the same run to the last figure on all eight islands. With the tree in they differ in the third figure (what a wage buys in 1881 to 1900 is 3.320 at 8 and 3.323 at 16), so on an island or two quality just touches 8 in the century's last years. 12 is clear of both.

**What it is worth with the tree.** Less than without, and for another purpose. Without the tree the lifted ceiling is the whole of late growth: what a wage buys in 1881 to 1900 is 2.99 for 2.64, and a head's spending grows 17 per cent from the 1870s to the 1890s for 5. With the tree it adds 7 per cent to the late wage (3.32 for 3.11, higher on eight islands of eight), because with builders there is a third of the stone to dig and stone was half of what the ceiling gave. And the tree's own last wave of inventions already carries the island on: with the tree and a ceiling of 4 a head's spending grows 29 per cent over the last two decades. Lifted, it grows 46 per cent. So with the tree the ceiling is no longer what keeps the island growing. It is what keeps the land growing with the works, so that food, timber and stone do not grow dearer against everything else for the century's last thirty years, and it is what keeps the towns gaining at the end: their share of 1881 to 1900 stands 0.021 above that of 1861 to 1880 with the ceiling lifted and 0.010 without, on eight islands of eight.

**The fault, second half.** The steel rail mill is found on fifteen islands of sixteen, 48 to 176 of them are founded on an island, and steel rails are never half the rails on any island in any year. The mill's steel costs less a rail than an iron mill's iron and its plant a third as much, but its hands cost twice as much, because its skill is 2.0 where the rolling mill's is 4.0. Two is the floor `machines.inherit` (0.5) gives an heir, and the mill never rolls enough to climb its own curve past it: its curve needs 108,000 rails to reach 2 and the middling island's steel mills roll 24,000 in a century (`tr-steel.md`, section 1). It is a trap, and the cheaper ore of the mines made it deeper.

**The mend.** An heir that makes the same good from a better material starts with the whole of its master's skill. Rolling a steel rail is rolling a rail: the mill, the rolls and the men are the rolling mill's, and what is new is the bloom. The converter is another matter, a new process and not a new material, and keeps its half.

In the code: `machines.inherit_rail` (1.0) beside `machines.inherit` (0.5), read in `machines::inherit_skill` for the steel rail mill alone.

**The numbers.** 1.0. The ramp in the shared number is steep and without a natural stopping place: 17 per cent of the rails of 1890 at 0.55, 36 at 0.6, three fifths at 0.667, nine tenths at 1.0 (`sens-machines.md`, `tr-steel.md`). The trial that moved only the steel rail mill's skill gives steel rails 0.50 of rails in 1860, 0.80 in 1870, 0.91 in 1880 and 0.95 in 1890, on fifteen islands of sixteen, with the price of rails three tenths lower, and moves nothing else beyond chance.

**With the tree it is not enough, and the page should say so plainly.** On today's island the mill is found about 1846, in the middle of the railway's rush, and with its master's skill it takes the trade in twenty years. With the tree the mill is found in the 1860s, as it was in history, and by then the rush is over: the island lays a few branches a year, the iron mills have the capacity for them, and nothing wears a rail out. Run for this page with the shared number at 1.0 as a stand-in (section 8), steel rails are 0.07 of the rails made in 1880, 0.23 in 1890 and 0.40 in 1900 with the tree in, between nothing and 0.70 by island in 1890, where on today's island they are 0.87 in 1890. `tree.md` says the same of the mill as it stands: "it waits for railways that wear out". So aim 17 is expected to be missed with the tree in. Two things would close it and neither is this pass's: stage 7's steeper chance, which finds the mill on time on the islands that today find it in the 1880s; and the tree's fifth stage, where a railway renews its rails every fifteen or twenty years and so buys rails after its rush is over.

**What could go wrong.**

- *The island runs on to 2050.* With a ceiling of 12 the wall is met some decades after 1900. What the machines' quality is to be counted in once steel is cheap is the 2050 design's question (`tr-works.md` says the same), and this number should not be mistaken for its answer.
- *One learning curve still carries the land.* If steel does not start, the machines do not improve. It happened in a run for this page: with the ceiling lifted on today's island, one island of eight pours 186 units of steel a month in 1861 to 1880 where the others pour tens of thousands, opens its first railway in 1860 and ends with a late wage of 1.81 for the others' 2.9 to 3.3. That is the late draw of section 4.8, which is why stage 7 goes before this one. Whether the machines should learn from all metal-working and not from steel alone is left.
- *An island that grows by nearly half in its last twenty years.* That is what the tree and the lifted ceiling give together. It is faster than Britain and about what Germany and the United States did, and the late wage stays inside aim 5's band (3.3, before the freight takes its part). If it reads as too fast when the stages are in, the exponent is the dial and task 8 of section 7 is the sweep.
- *The steel rail's worth is still taken at the mill* (R59, C22: a steel rail's longer life cheapens new track as well as relaying). With the floor mended the steel rail takes the whole trade, so that matters more. It belongs with the tree's fifth stage, where railways wear out and rails are bought every year.
- *A threshold moved.* More steel sooner brings forward every invention counted in steel or machinery. The fitter is run again.

**How it is checked.** `trace_works --only growth` on sixteen islands: output a hand on the farms, in the quarries and in the pits still rising from the 1870s to the 1890s; aim 6 of the card. `steel_trace --seeds 1-16`: the steel rail mill's skill at 4.0 from its founding and its hands a rail within a fifth of the iron mill's; steel rails over half of the rails of 1890 on fourteen islands with the tree's stage set to 0, and with the tree in no less than this page's 0.23. Machinery's dearest year after 1850 no more than three times its usual price (`docs/rules/machines.md`'s own measure; island 6 had a machinery famine when the ceiling was lowered, so it is watched when the ceiling is raised).

### 4.10 The works' yearly room and the building cycle (stage 9)

**The fault.** Once a year a works whose return clears the hurdle asks for at least a fifth more capacity, and a trade's room for new capacity in a town is `works.year_room` (0.15) of its monthly trade. So a trade whose works clear the hurdle takes its room whole and one whose works do not takes nothing: the mills take their whole room in a quarter of town-years and next to nothing in two fifths, every 3.3 years, and up to nine towns in ten do it in the same year, since cloth has one price across the island (`tr-cycles.md`, section 2). A mill and a brickworks are built of timber. In the wave's year the works being built buy as much timber as the island makes, households are priced out of it, and what a wage buys falls: the worst year of the century is 15 per cent down, fourteen times in sixteen between 1874 and 1899, and three fifths of it is timber (`tr-works.md`, section 4). The same lumps, in the engine works' orders for plant of 20 tools a unit, are the famine of tools: 8.1 years in a run above three times the middling price.

A wave of orders is one half of it. The other half is that what the wave asks for comes from the land, and the land answers late: hands on the land move only 0.15 of the way a month to what their employers want (`labor_adjust_ex`, the port's). The woods' output follows last year's price (correlation 0.83) and hardly this year's (0.18). Clay's price saws with a period of two years and no weather in it, a ring between hands that reach the pits months late and a markup that over-corrects (`tr-cycles.md`, sections 2 and 4).

**The decision: the room stays at 0.15, and the land's hands follow the wage at 0.5 a month.** I came to this page meaning to take the smaller room. `tr-cycles.md` found that a room of 0.10 lowers timber's swing and the wage's on twenty-four islands of twenty-four, and called it the cheapest cure that helps. The same page found that faster hands on the land do as much on eight islands, and warned that the old farm zigzag might come back with them. So the two were run side by side, each alone and both together (section 8; seeds 1 to 8, today's island):

| | today | the room at 0.10 | the land's hands at 0.5 | both |
|---|---|---|---|---|
| the real wage's yearly swing | 0.064 | 0.049 (lower on 8) | 0.049 (8) | 0.041 (8) |
| timber's swing | 0.204 | 0.149 (8) | 0.146 (8) | 0.100 (8) |
| grain's price swing | 0.115 | 0.109 | 0.079 (8) | 0.076 (8) |
| clay's swing | 0.178 | 0.186 | 0.044 (8) | 0.038 (8) |
| the farm zigzag | 2.08 | 1.87 | 1.31 (8) | 1.09 (8) |
| the wage's five worst years | 12.0 per cent down | 11.0 | 9.7 (better on 6) | 8.1 (8) |
| years tools cost over three times their middling price | 7.25 | 1.75 | 2.6 | 3.9 |
| what a wage buys, 1781 to 1800 (1750 = 1) | 1.52 | 1.35 (lower on 8) | 1.52 | 1.40 (lower on 8) |
| share of people in towns, 1791 to 1810 | 0.267 | 0.227 (lower on 8) | 0.268 | 0.236 (lower on 8) |
| the first railway opens | 1827 | 1837 (later on 7) | 1830 | 1833 |
| what a wage buys, 1881 to 1900 | 2.64 | 2.69 (4 up, 4 down) | 2.72 (7 up) | 2.64 |

Faster hands buy the same calm as the smaller room, to the third figure in the wage's swing and in timber's. They calm grain by a third and end clay's sawtooth, which the room does not touch. The farm zigzag falls by more than a third where it was feared it would rise: with the merchants marking prices on their stock, hands that follow this month's price are steadier than hands that follow last season's. And they cost nothing at the start. The smaller room costs a tenth of what a wage buys before 1800, four points of the towns' share about 1800 and ten years on the first railway (on one island there is no line at all in 1850), because a trade that may add less each year is slower to make the machinery the railway's discovery waits for. `sens-works.md` saw the dear early iron; it did not read the railway's date. Both together are calmer again and keep most of the room's early cost.

In the code: `hands.land_adjust` (0.5) in the `hands` mechanism of section 4.1, read where `econ::step_month` takes `kx` from `P.labor_adjust_ex`. It is one number for every land sector, farms, pastures, woods, quarries, pits and mines, because that is what was run. `tr-cycles.md`'s check ran the woods alone and the clay pits alone and found each does its own part (the woods' hands a sixth of timber's swing, the pits' all of clay's sawtooth), so the number can be split by sector later if a harvest month wants the farms' hands slower again.

**What was tried on the pages and is not taken.** A builders' queue takes 31 per cent off timber's swing and costs 4 per cent of the wage of the 1830s and 1840s. Room in proportion to what is short ends the famine of tools and costs 15 per cent of the late wage. Orders spread over the year take 15 per cent off. Plant valued at fixed prices leaves 136 fewer works. A growth step in proportion to how far the return clears the hurdle halves the mills' bare years and takes 12 per cent off timber's swing; it is task 3 of section 7. Three times the smithies' room shortens the tools famine to a third; it is left because the tree gives tools two new trades (the edge-tool works and the tool factory), and with them tools are no longer short (section 8).

**With the tree.** The tree brings one rule that bears on this, and it is kept: where a market is short the day's trade is served first and orders for plant wait (`tree.plant_waits`). With it the worst year with the tree is 14.0 per cent down and without it 18.0 (`tree.md`). The faster hands add to it. Run with the tree in, beside the ceiling, the heir's skill and the brake (section 8, eight islands, against the tree alone): the wage's five worst years are 7.8 per cent down for 11.4, better on eight islands of eight; the wage's yearly swing is 0.042 for 0.058; timber's 0.147 for 0.229; grain's 0.087 for 0.117; the farm zigzag 1.21 for 1.83. Clay matters more with the tree, since bricks are what the builders build in and the clay pits are 6 per cent of all hands by 1900: its price swings 0.273 a year with the tree alone, the largest swing on the island, and 0.128 with the faster hands.

**What could go wrong.** The port chose 0.15 "so a price blip does not empty the fields in a month", and at 0.5 half of a blip's worth of hands do leave. The grain made is steadier for it on every island, but famine years read 0.75 a run for 0.375 (three islands up and three down, one island with three). That is inside what a null run moves them by (`sens-prices.md`: 0.25 to 0.625), and most of today's hunger is the empty shelves of section 4.2; it is to be read again on sixteen islands once stage 3 is in, and if hunger that is not the shelves' rises with the faster hands, the farms and pastures go back to 0.15 and the woods, pits and quarries keep 0.5. The mines' workings shed hands by the same number, so a working's wild year (section 4.1) is shorter with it, which is to the good.

The railway's date is the other thing to watch. In the three runs of this page with the faster hands the first line opens three to seven years later on the mean (1830 for 1827 alone; 1831 with the other three numbers; 1836 for 1829 with the tree), and each has an island with no line in 1850. But the islands split, fourteen later and ten earlier of twenty-four, and the railway's date has a spread of 8.6 years from the draw alone (`tr-steel.md`), so eight islands cannot tell this from chance. Stage 7 goes first, so it will be read on the steeper chance; if aim 14 is then late, the railway's own two thresholds (60,000 of machinery and 150,000 of rails made) are what to move.

**How it is checked.** `trace_cycles --seeds 1-16` with the tree in: timber's swing and period, the woods' output against this year's price and last year's, clay's `ac1`; aims 7, 8 and 10 of the card; `farm_zigzag_beyond_weather` no higher than at stage 8; hungry town-years by `trace_rent --part d`; and what a wage buys in 1781 to 1800 and 1831 to 1850 no lower than at stage 8 on more than ten islands of sixteen.

### 4.11 The brake on a glut, and the merchants' markup (stage 9)

**The brake: `prices.glut_months` 2 for 4.** The merchants stop taking a works' output by degrees once their stock passes `glut_months` of sales. At three quarters and four thirds of its value the brake moves nothing; at 2 iron's price swings 18 per cent less on all sixteen islands (0.174 to 0.142) and rails' an eighth less, and nothing else moves beyond the null, people and wages included (`sens-prices.md`). Iron's swing is in a line across the values tried: 0.144 at 2, 0.167 at 3, 0.172 at 4, 0.186 at 7. The aim is 0.16, which today's game misses. With the tree iron already swings 0.150, so the brake is the margin and not the cure; it is taken because it is cheap, measured on sixteen islands, and moves nothing else. Run for this page beside the other three settable numbers (section 8), iron's swing is 0.135 for 0.172 on today's island and 0.122 for 0.155 with the tree, lower on eight islands of eight each time.

`glut_stop_months` stays at 8. Taken to 4.5 as well it steadies iron further (0.119) and churns the works: a fifth more founded and over a quarter more closed.

**What could go wrong.** At 2 the brake begins at the stock the merchants mean to hold, which `docs/rules/prices.md` warns can choke trade. It did not in 16 runs. What it does to unfilled orders and to the iron stock was not measured, and those are two of the prices page's own tests: `examples/prices_probe.rs --shock` is run with this stage, and iron's stock should stand near two months with under 4 per cent of orders unfilled.

**The markup stays at 0.5.** `sens-prices.md` found that a stiffer merchant (0.667) lowers the farm zigzag by a sixth with the harvests on and by nothing with them off: what it buys is farm hands that answer a short harvest less, and from 0.8 it costs famines. So the markup is not a cure for a false cycle any more, and whether farm hands should answer the weather less is a question about the harvests (section 6).

### 4.12 What "living in towns" counts (stage 10)

**The fault.** The share of people living in towns is not a count of where anybody lives. Every year each town's town people are set to its people times `0.04 + 0.8 x` its works' hands over its labour force (`towns::update_urban`, and the port's `land::update_urban`), smoothed with last year's. So the island's share is 0.04 plus eight tenths of the share of its hands that work in works, to a thousandth, and the plan's half is 57.5 per cent of all hands in works (`tr-urban.md`, (a)). Of the things that might hold it at 0.42, the ground a town can take never binds, how fast people move does not move it, more towns do not raise it, and there is no gap between town and country wages to close. What holds it is where the other half of the hands work. The largest group is the quarrymen, one hand in seven, because households buy their shelter as raw stone; the tree, which puts builders between the quarry and the house, takes them to under 3 per cent and the share to 0.46 or 0.47.

**The decision.** The count keeps its two numbers and gains one group of people.

- *The 0.04 and the 0.8 stay.* They are the Python's and were never set for the game, but what they say is fair for the whole century: a twentieth of the people live in a town whatever they do, and a fifth of the works' households do not (the mill on its stream, the furnace at the pit, the country smith). Putting every works' household in town reaches 0.51 at a stroke, and costs 9 per cent of the people through the crowding deaths that read the count (`tr-urban.md`'s trial). That is a larger change to the people than to the towns.
- *The households of those who carry between towns are town households.* Wagoners, boatmen, railwaymen and the porters at wharves and stations live where the loads are made up. The count's "works' hands" becomes the works' hands and the carters of the trade between towns and of the works' own carting. The carters who bring a district's produce in to its market stay in the country: that is farm work. In the code: the carting rule already adds the month's bill up from its three parts; under `towns.count_carriers` it keeps the trade's and the works' share of each town's carters beside the total, and `towns::update_urban` adds them to `st.ind_labor[i]` in the count only. Nothing else reads it but the crowding deaths, which read the count as they do now.

**The numbers.** By the books' shares of the bill, the trade between towns and the works' carting are 16 per cent of the bill in the 1750s and 51 per cent in the 1890s (`tr-carriage.md`, (b)). On today's carters that is 2.0 per cent of all hands in 1750 and 3.3 per cent in 1900, so 1.6 points on the share in 1750 and 2.6 in 1900. With the freight by the wage the carriers are 5 to 6 per cent of hands late, and four points or a little more.

**What it should come to.** Over 1881 to 1900 the share is 0.465 with the tree alone and 0.472 with the tree and this page's four settable numbers, where it stands at 0.49 in 1900 itself and is rising (section 8). The freight by the wage takes about a point off by today's count, since more hands are on the road, and the carriers add three to four. So 0.49 to 0.51 for 1881 to 1900, and still rising at the end, because the carriers' trade grows with the island and the lifted ceiling goes on moving hands off the land.

The start is the cost. With the tree the island opens with 0.29 to 0.30 of its people in towns, which is the top of what aim 2 allows, and the carriers add a point and a half: 0.31, a point over. I take the miss and do not bend the count to hit both ends. What puts the start high is the tree's trades of 1750 (builders, sawyers, brewers, potters and the like are an eighth of all hands that year, and the count puts four fifths of them in town), and if the start is to come down it is that fifth of the works' households living out of town that should be larger in 1750 than in 1900, which is a question for whoever reads the tree's towns once it has landed.

**What could go wrong.** The crowding deaths read the count, so more town people mean fewer people. From the trial (nine points more town, 7.7 per cent fewer people by that road), four points should cost about 3.5 per cent. The island with the tree has 5.05 times its people of 1750 (section 8), so that puts it at about 4.9 times, which is aim 1. So `people.town_deaths` (0.012) is not moved. If the island falls under 4.5 times, it goes to 0.010 (each 0.001 is about 17,000 people of 1900: `sens-people.md`).

**How it is checked.** The identity itself: the island's share equals `0.04 + 0.8 x` (works' hands and carriers) over the labour force to a thousandth at every mark, as `tr_urban` prints it. Aims 2, 3, 4 and 1.

### 4.13 The Engel reference stays where it is

**The question.** `engel.y_ref` is 2 bundles a head, set beside an island that began at 1.84 to 1.89. The island now begins at 1.51 to 1.60, so the curve starts to turn only between 1761 and 1782. `docs/rules/engel.md`'s own reasoning (set it where the island starts) would now say 1.55. Each quarter bundle taken off buys one to two points of town, most in mid-century, and costs about one per cent of the people and five per cent more swing in grain, with famine years rising in step (`sens-prices.md`, sixteen islands).

**Decision: it stays at 2.0, and the page's reasoning is retired.** `y_ref` is a statement about households: at two bundles a head a household begins to spend as a richer one does. It is not a statement about the island. An island that begins poorer should take longer to reach that income, and it does. To move the reference down because the island moved down is to change the households so that the island's start looks the same, which is the wrong way round.

The measurements point the same way once the tree is counted. What the lower reference buys is town before 1850: 2.9 and 4.2 points in 1781 to 1800 and 1831 to 1850 at 1.5, and 1.5 points by the 1890s. With the tree the island already opens with 0.29 of its people in towns, above the plan's "about 0.25", and section 4.12 adds a point and a half. The early century does not need more town. The late century is where the card is short, and there the curve has all but finished turning whatever its reference: real income is 8.8 bundles a head in the 1880s and 1890s, and 10.4 with the tree. Nor has the tree changed where the island starts: 1.62 bundles a head in 1750 on eight islands, for today's 1.56 (section 8).

`engel.strength` stays at 3 for the same reason: between 2.25 and 4 it moves the late towns by under a point. What holds the towns of the 1890s is the richer set of shares, which is content, and the tree's overlay gives its own.

### 4.14 The roads prototype stays off

`docs/probes/dx-roads.md` prototyped roads that somebody pays for behind `roads.enabled`, and the prototype is on main, switched off under both sets of rules: a parish makes roads out of a rate, a turnpike trust borrows, takes tolls of 0.05 a cell and pays its lenders, and a bridge costs stone by the metre of river.

**Decision: it is not switched on in this pass.** Four reasons, three of them the probe's own.

- *It moves who pays and not how the island lives.* Switched on, the carting bill, what a wage buys and the towns do not move clear of chance. Nothing on the card is nearer for it.
- *The toll is taken from the wrong purse.* It is taken from the households of the town at the gate, as if the carters who passed lived there. The honest way is a second sum along every path, which is a change to the searches every mechanism uses.
- *It is not safe in a live game.* The live game's list of lines building counts a road works as a railway.
- *It waits on two things that have not happened.* A bridge's cost should follow the new land's rivers, and the roads after the trusts want the public purse of the 2050 design.

**With what toll, when it is.** The prototype's 0.05 of path cost a cell, a sixth of a lowland turnpike's cost. It gives tolls of 0.49 per cent of output in 1760, which is about England's share in the 1830s, and a trust's lenders 5 per cent a year at the median. A toll is a path cost, so once the freight follows the wage the toll follows it too and needs no number of its own.

**What should come first when the roads' turn comes** is the probe's own first step, and it needs none of the above: railway and canal companies that are paid a margin of their fares. It is the half of "who is paid" with the most money in it, 2.5 to 3 per cent of output from 1850. It is not in this pass because shares that somebody owns are the edge of the core loop.

## 5. The order to build them in

Eleven stages. Each lands alone, behind its switch, with its page and its sweep; each is read as section 3 says. The line between stage 5 and stage 6 is the tree's landing.

**The rule for which side of the tree a stage falls.** Before it, the mends of faults, whose effect does not depend on what the island makes: a town's wage either runs away or it does not, with builders or without. After it, the choices of numbers, which do: what is carted, how much steel is poured, what plant is built of and where the hands work are all different with the tree, and a number chosen on today's island would be chosen for an island that is about to go. There is a second reason. The tree's thresholds are plain numbers of units fitted to the island as it stands (`tree.md`, "The fitter"), so everything that moves the island's output after the tree lands costs a pass of the fitter. Stages 2 to 5 move no output in the mean, so the tree can land on them without one.

| stage | what lands | its own measure | what the card should do |
|---|---|---|---|
| 1 | The card: the aims and tolerances of section 2 in `scorecard::AIMS` (the sweep's tools themselves landed while this page was written), and the facts below | no rule changes: 0 facts differ | it can be read |
| 2 | Hands that exist (4.1) | no wild town-month on the named runs | nothing beyond the null pair; aim 13 met |
| 3 | The shelves go with the district (4.2) | no hungry January after a redraw | aim 12 met |
| 4 | Money on the true total at 0.0175 (4.3) | 1.764 per cent a year on every island | nothing beyond the null pair |
| 5 | Housekeeping (4.4, 4.5) | 0 facts differ for the retired settings; the rent as paid alone moves a run | nothing |
| | *The production tree's first stage lands, by its own review. Its thresholds are fitted on the island as stage 5 leaves it.* | | aims 2, 3 and 5 move as its page says |
| 6 | Freight by the wage, and the founding test with it (4.6, 4.7) | carters a unit of weight and path cost level across the century | aims 18 and 19 met; baskets a head 5 to 8 per cent down |
| 7 | The dates of invention (4.8); the fitter taught the hazard and run | each find within three years of its threshold | aims 14, 15 and 16 met |
| 8 | The machines' ceiling and the steel rail (4.9); the fitter run | farm output a hand still rising in the 1890s; the steel rail mill's skill 4.0 | aims 4 and 6 met; aim 5 inside its band; aim 17 read, and expected short |
| 9 | The land's hands and the brake (4.10, 4.11) | timber's swing; clay's sawtooth gone; iron's stock and unfilled orders | aims 7, 8, 9 and 10 met |
| 10 | Who lives in a town (4.12) | the count's identity to a thousandth | aims 3 and 4 met; aim 1 read |
| 11 | The whole card on twenty-four islands, both regimes, with and without families; the README's and the plan's numbers made again; each rule's page brought up to date | | every aim met or its miss explained |

**Stage 1's facts.** Add, never rename (`scorecard.md`). The sweep's tools already give twenty-year means, the tools' dear years against the century's middling price, emigration as a kind of its own, the zigzag beyond the weather and the worst year from the median town. Beyond those the card needs: the tools' dear years against the twenty-five years about each (aim 11); spending a head over the chained cost of living by decade, and its peak year; the mean of each island's five worst years for the real wage; the town-months in which a town paid over ten times the middle town's wage, and those in which more hands were at work than the town has people; the diggings' share of hands without the carters (`mine_share` is one less the farm and industry shares, so 0.126 of its 0.216 in 1750 is carters: the old fact stays and a true one stands beside it); the carters between towns as a share of hands; the carters a unit of weight and path cost; timber's swing (it is not among `price_swing_*` today); closures by a works' age, to stand beside `works_closed_median_life`, which `tr-works.md` shows is a poor fact; the share of hands within two cells of their market, for `works.walk`; and the share of town-months in which a town's wage moved by half, for `carting.adjust`.

**Which pairs interact.** The tables flag most of these; a few follow from the design.

| pair | what is known | what the design does about it |
|---|---|---|
| the issue of money with the freight | issuing a third more money made carting a quarter cheaper (`tr-money.md`) | stage 4 keeps the rate; stage 6 removes the road; task 1 tests that it is gone |
| the freight with the founding test | the test is in coins against a freight in coins (`tr-urban.md`) | one stage |
| the freight with the promoters' hurdles | a saving that grows with the wage against costs that do not | `rail_hurdle` 0.08 if the railway of 1900 passes 2,700 km |
| the freight with `carting.labor` | the dial for the first fifty years, which does not move 1900 (`sens-works.md`) | left; task 5 |
| the ceiling with `quality_b` and `quality_Q0` | the ceiling decides the end and the exponent the middle (`sens-machines.md`) | the ceiling alone; the exponent is task 8 if aim 5 is passed |
| the ceiling with the dates of invention | steel sooner is quality sooner | stage 7 before stage 8, so the ceiling is read on the dates the game will have |
| the ceiling with `learn_cap` | a higher `learn_cap` brings the stall sooner (`tr-works.md`) | `learn_cap` is left |
| the year's room with the land's hands | each calms the building cycle by about as much, and together by more; the room costs the start of the century and the hands do not (section 4.10) | the hands move and the room stays |
| the year's room with the steel rail and with the railway's date | a smaller room makes iron dearer and steel worth rolling (`sens-works.md`), and makes the first railway ten years later (section 8) | one more reason the room stays |
| the land's hands with the markup | faster hands lower the farm zigzag by a third at the game's markup; what a stiffer markup would add was not run | the markup is left |
| the land's hands with the harvest | hands that follow the price follow a short crop too; famine years leaned up on eight islands, inside the noise | read again on sixteen islands after stage 3 |
| the brake with `glut_stop_months`, `price_target_months` and `trade_cap` | the brake at 2 begins at the stock merchants mean to hold | `prices_probe --shock` with stage 9 |
| the hazard with the tree's thresholds, and with the canals' copy of it | the fitter works the hazard out for itself; `canals.rs` keeps a power of 4 | one mechanism read in both places; the fitter taught it |
| the count of town people with `people.town_deaths` | the crowding deaths read the count: nine points of town cost 7.7 per cent of the people (`tr-urban.md`) | `town_deaths` held unless aim 1 is missed |
| the tree with the people | six per cent fewer, by the same deaths (`tree.md`) | the same |
| the tree with the canals | a fifth less canal in 1800: there is a third of the stone to carry (`tree.md`) | left; the freight by the wage raises what a cut saves; task 9 |
| `engel.y_ref` with the harvest's numbers | the curve's cost is a thinner margin in a bad year (`sens-prices.md`) | neither is moved |
| `growth_top` with `y_still` | not one dial: the second sets the start and the first the end (`sens-people.md`) | neither is moved unless aim 1 is missed; then both together |
| `d_old_growth` with `growth_top` | town ground grows with people | neither is moved |
| hands that exist with the sinkers, carters and navvies | each is taken out of a town's supply before the wage is sought | the rule rations what is left; a test for a town with nothing left |

## 6. What to leave alone, and why

- **The people's four numbers** (`growth_top`, `y_still`, `ease`, `town_deaths`). They are the strongest numbers in the game: a quarter off `growth_top` is 268,000 people and 14 per cent on output a head (`sens-people.md`). The island grows 5.4-fold where they were set for five, and the tree and the new count between them take it to about five. So the pass reads aim 1 at stage 10 and moves nothing unless it is missed.
- **The harvest's size and how much of it is foreseen** (`island_sd` 0.05, `foreseen` 0.5). They move the year-to-year swings and nothing in the long run (`sens-machines.md`). A bad harvest takes 3 per cent off what a wage buys for each tenth it is short, and with the shelves mended almost no hunger is left that is not the weather's, so the harvest can be judged on its own for the first time. A harvest month is being designed ("Ready next" 15) and will set both again; nothing should be tuned under it.
- **The merchants' markup** (0.5): section 4.11.
- **`engel.y_ref` and `engel.strength`:** section 4.13.
- **The ground towns keep** (`d_old_growth`, `d_old`, `tau`). They set how much ground the towns stand on and nothing else that sixteen islands can see. They are for the picture, and the towns' second pass is working on that picture now.
- **What kept ground costs** (R99, R102). `sens-people.md` could not find the dip in the wage on sixteen islands; `tr-works.md` found 2 to 5 per cent in some twenty-year span in three settings of four, and could not say which span. Its presence is likely and its cause is not found. The only test is the towns page's own mend (a town cell keeps under its old uses the share of its ground the houses do not need), which is task 4 and not a stage.
- **The railways' branch planner** (`districts`, `branch_gap`, `cool_years`, `cost_mult`). They size the network and move output a head by under 3 per cent (`sens-ways.md`). `branch_gap` sits at a peak, so the network's size is partly a by-product of how the districts are searched; it should be set by how the map reads, once the railways' piece of the viewer has been looked at ("Ready next" 5).
- **The canal age's shape.** Whatever is moved, three quarters of the network of 1900 is open by 1800 (`sens-ways.md`). That is the rule and not its numbers; tolls, or a promoter who must see a canal pay before the next is dug, belong with the companies of section 4.14.
- **`mines.reach` (2.0) and the quarries' two numbers** (`quarry_reach`, `quarry_input`). Each matters and was never searched. Each is about stone, and with builders there is a third of the stone. They are read again after the tree and not before.
- **`works.year_room`** (0.15): section 4.10. It is the life cycle's one number that the economy answers to, and it sits in a flat bottom for the level of the economy from about 0.11 to 0.2 (`sens-works.md`). Below that it buys calm with the start of the century.
- **Whether a large quarry gets a railway branch.** A run had about seventy branches to quarries before the mines and has a dozen now (`docs/rules/mines.md`). With builders there is a third of the stone to carry, so the question is about to change under whoever answers it.
- **The pacing of the early century** (R22, R13, C92). Carting with the life cycle moved the island's steepest decades about ten years later, and the plan's targets for 1750 to 1800 were set without carting. The card has no aim before 1800 but the towns' share at the start, on purpose: history's real wage hardly rose before 1800, and the island's rises by half. Nothing here makes the start faster and one thing (the freight, from the 1780s) makes it a little slower.
- **`learn_cap`.** Raising it brings the stall sooner, because more steel sooner meets the machines' ceiling sooner (`tr-works.md`).
- **The works' numbers that set only the map** (`hands_per_cell`, `cell_limit`, `entry_share`, `room_share`). Free of the economy across wide ranges (`sens-works.md`); the works' site planner should have them.
- **The glut brake on works alone.** A brake on the woods and pits has next to nothing to brake: the island holds five weeks of coal (`tr-carriage.md`, (c)).
- **A works that closes younger than the port's** (C140). It is the life cycle letting its fringe go, as designed (`tr-works.md`, section 3).
- **The handling charge.** It is a tenth to a seventh of the bill and a hand in a hundred (`tr-carriage.md`, (b)).
- **Where an invention is found.** Half of an island's inventions are found in one town, by the rule's own square. It makes a story.
- **Exports and the price rule** (R78, C11). The rule's flow term cannot see exports, so an exporting town reads as a glut; mending it took the families' share of all money from 10 per cent to 66. That is the balance of the family game and waits with the core loop.
- **Everything in the core loop:** land that beats works, the families' share of capital, what the default trade is, a family's works not rationed by the year's room. The last bears on stage 9 (it is why counting a dear trade larger did harm with families), and the stage is measured with six families for that reason; the rule is not touched.
- **The chronicle's thresholds.** They read the economy and never change it. `closures_min` wants reading after stage 9; the rest belong to the news's own pass.
- **The second island generator's numbers.** It is off until the viewer can draw it, and every number on this page will want reading again when it is on.
- **The commons scenario.** It is run with every stage for failures and tuned for nothing.

## 7. What the measurements cannot settle

Each is one sweep or one trace, written so that it can be handed to a worker as it stands. All are the game's rules with six families unless they say otherwise; "the card" is `isostate-core scorecard DIR`.

1. **Is the island indifferent to money once the freight follows the wage?** After stage 6: `isostate-core sweep --seeds 1-16 --rules game --families 6 --set money.growth=0.01,0.0175,0.03`. Read twenty-year means of what a wage buys, spending a head over the chained cost of living, the carters' share, the towns and the canals, high against low. If nothing real moves by more than the null pair, close K39 and the "money is not neutral" of `tr-works.md`. If something does, find the constant in coins behind it; the candidates are the price floor and ceiling (0.05 and 40 times a good's base price: with the tree machinery ends the century at 0.07 of its base price) and the wage's ceiling of 500.
2. **Why do the merchants owe half the money by 1900 under the price rule?** `money_trace --seeds 1-8 --years` with and without `--set '{"prices":{"enabled":false}}'`, reading the books' `merchants_kept` against `stock_value` by decade (`tr-money.md` measured the debt, 47 per cent of the money in 1900 against 2 per cent with the rule off, and did not trace it). Say whether they buy stock on credit they never repay, and whether anything real depends on it.
3. **Does a growth step in proportion add to the faster hands?** `tr-cycles.md`'s trial switch `TRC_STEP` (a works over the hurdle asks for `0.2 K (roi - hurdle) / hurdle`, at most half, in place of at least a fifth), after stage 9 and with the tree in: `trace_cycles --seeds 1-16` with and without the switch. It is the cure on the orders' side that costs the start nothing, where the smaller room does. It is worth a rule if timber's swing falls a further tenth on fourteen islands, the mills' bare years halve as they did alone, and what a wage buys in 1781 to 1800 and in 1881 to 1900 is lower on no more than ten.
4. **What does kept town ground cost, and is it the farmland?** Build the towns page's mend behind a trial switch (a town cell keeps, under its old uses, the share of its ground its houses do not need; the rent floor is charged on the built share only). `sweep --seeds 1-16` with and without it, with six families and with none, read over 1790 to 1810, 1840 to 1859, 1860 to 1879 and 1880 to 1900. If the 2 to 5 per cent comes back in the span where each setting lost it, the mend is a stage; if not, R102 is closed as chance.
5. **What does carting cost the middle of the century, and should `carting.labor` move?** After stage 6: `sweep --seeds 1-16 --set carting.labor=0.5,0.75,1.0`, reading baskets a head for 1781 to 1800 and 1831 to 1850 high against low, and a trace of one island through the 1830s and 1840s at 0.5 and 1.0 with `trace_carriage`. `sens-works.md` could not tell whether half the carters give back a third of the mid-century's cost or none of it. If the effect is steady from island to island and the start of the century is judged too slow once the freight is in, 0.75 is the value to try.
6. **The spring hungers that are not empty shelves.** After stage 3: `trace_rent --seeds 17-24 --part d`, following Ryefell (island 23, May 1832), Langside (island 18, April 1844) and Brightheath (island 20, June 1759) month by month, if they are still there: whose grain it was, where it went, and whether the town's merchants ordered for the year or for the month.
7. **Does the brake at 2 choke anything?** `examples/prices_probe.rs` with `--shock`, once as the game stands (`--out brake4.json`) and once with `--set prices.glut_months=2 --compare brake4.json`, on four islands: iron's stock in months, the share of orders unfilled, and iron output's swing by half-century, which are the three tests of `docs/rules/prices.md` that no sweep reads. The probe was written for the port with the price rule switched on by `--set`; if it cannot take the game's rules whole, teaching it to is the first half of the task.
8. **If the lifted ceiling passes aim 5, which exponent?** Only if what a wage buys in 1881 to 1900 passes 3.5 with the tree in after stage 8, or the last twenty years' growth reads as too fast: `sweep --seeds 1-16 --set machines.quality_cap=12 --set machines.quality_b=0.5,0.55,0.6`. Take the largest exponent that brings the late wage inside the band and keeps aims 4 and 6.
9. **Is the canal age still what its page wants, with the tree and the freight?** After stage 6: `sweep --seeds 1-16 --set canals.cost=0.0975,0.13,0.173`, reading the first canal's year, the km open in 1800 and 1900, and the towns of 1900. The tree leaves a fifth less canal in 1800 (554 km for 695); the freight by the wage raises what a cut saves from the 1780s on. If the two leave under 500 km in 1800, `canals.cost` 0.0975 is the measured way back (33 km more; `sens-ways.md`).
10. **Does the freight want the town's own wage?** A trial with the index taken from the paying town's own five-year wage in place of the island's, on seeds 1 to 16 after stage 6. It is the truer rule (the carter is hired where the load is paid for) and was not chosen because a town's wage jumps when it takes a district. Read the wild town-months and the wage's five worst years; if neither is worse, the town's wage is the one to keep.

## 8. What was run for this page

Eight sweeps, each of seeds 1 to 8 under the game's rules with six families, to 1900: 144 runs, of which none failed, and every one passed the monthly money check. Eight islands are too few for most of what section 3 asks, so these are read for large movements on which all or nearly all the islands agree, and for whether the numbers fight one another.

**Two builds.** Today's island is this page's branch, which changes no code: its sweep with no setting gives every fact as today's main does.

```bash
cargo build --release -p isostate-core
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --out DIR/game --compare MAIN --quiet
#  of the 446 facts both have, the seconds aside, 0 differ and 0 of those are starred
```

`MAIN` is the sweep of main kept from its last landing (`merged-game-f6-l`). The island with the tree in is `night/int-tree` (4490124), which is not on main. Its sources were copied out with `git archive` into a scratch folder and built there, so no branch was checked out or changed. That build with `tree.stage=0` and the ceiling at 4 gives today's island to every figure of the table below on all eight islands, so the two builds can be set side by side.

**The sweeps.** `CORE` is today's build and `TREE` the tree's.

```bash
CORE sweep ... --set labor_adjust_ex=0.15,0.5 --set works.year_room=0.15,0.10 --out DIR/m-cycle
CORE sweep ... --set machines.quality_cap=8 --set machines.inherit=1.0 --set labor_adjust_ex=0.5 --set prices.glut_months=2 --out DIR/m-design --compare DIR/game
CORE sweep ... --set machines.quality_cap=8 --set machines.inherit=1.0 --set works.year_room=0.10 --set prices.glut_months=2 --out DIR/m-combo --compare MAIN
TREE sweep ... --set tree.stage=0,1 --set machines.quality_cap=4,8,16 --out DIR/t-cap
TREE sweep ... --set machines.quality_cap=4,8,16 --out DIR/t-cap1
TREE sweep ... --set machines.quality_cap=8 --set machines.inherit=1.0 --set labor_adjust_ex=0.5 --set prices.glut_months=2 --out DIR/t-design
TREE sweep ... --set machines.quality_cap=8 --set machines.inherit=1.0 --set works.year_room=0.10 --set prices.glut_months=2 --out DIR/t-combo
```

Three things about them. `labor_adjust_ex` is the port's number, which `--set` can reach; the design gives the game its own copy. `machines.inherit` at 1.0 stands in for the steel rail mill's own number, which does not exist yet: it also gives the converter its master's whole skill, which the design does not, and that moves the converter little (`tr-steel.md`: 0.875 of steel in 1880 for 0.857). And the ceiling was run at 8, before these runs showed that 12 is the safer number. The two with the smaller room (`m-combo`, `t-combo`) were run first, when the room at 0.10 was still the plan.

The readings are twenty-year means from each sweep's `annual.csv`, island for island, by a short script kept with the sweeps and not in the repository; the swings are the standard deviation of the yearly change in the logarithm of a yearly mean from 1800 to 1899, except iron's, which is the sweep's own fact.

**The year's room against the land's hands** is the table in section 4.10 (`m-cycle`).

**The four settable numbers, on today's island and with the tree.** Mean of eight islands; in brackets, the islands on which the figure is higher and lower than in the column it is read against (today's game for the second column, the tree alone for the fourth and fifth).

| | today's game | today, the four numbers | the tree alone | the tree, the ceiling lifted | the tree, the four numbers |
|---|---|---|---|---|---|
| share of people in towns, 1751 to 1770 | 0.215 | 0.220 | 0.294 | 0.294 | 0.300 |
| the same, 1881 to 1900 | 0.420 | 0.423 (7/1) | 0.465 | 0.477 (8/0) | 0.472 (7/1) |
| its gain on 1861 to 1880 | 0.014 | 0.013 | 0.010 | 0.021 (8/0) | 0.016 (5/3) |
| what a wage buys, 1781 to 1800 (1750 = 1) | 1.52 | 1.51 (4/4) | 1.50 | 1.50 | 1.56 (6/2) |
| the same, 1881 to 1900 | 2.64 | 2.97 (7/1) | 3.11 | 3.32 (8/0) | 3.26 (7/1) |
| spending a head over the chained cost of living, the 1890s over the 1870s | up 4.7 per cent | up 13.2 (7/1) | up 29.5 | up 45.8 (8/0) | up 49.1 (7/1) |
| the wage's five worst years | 12.0 per cent down | 9.8 (7 better) | 11.4 | 11.0 (4/4) | 7.8 (8 better) |
| the wage's yearly swing | 0.064 | 0.051 (1/7) | 0.058 | 0.059 | 0.042 (0/8) |
| timber's swing | 0.204 | 0.160 (1/7) | 0.229 | 0.233 | 0.147 (0/8) |
| iron's swing (`price_swing_iron`) | 0.172 | 0.135 (0/8) | 0.155 | 0.154 | 0.122 (0/8) |
| grain's price swing | 0.115 | 0.078 (0/8) | 0.117 | 0.115 | 0.087 (0/8) |
| clay's swing | 0.178 | 0.050 (0/8) | 0.273 | 0.278 | 0.128 (0/8) |
| the farm zigzag | 2.08 | 1.22 (0/8) | 1.83 | 1.82 | 1.21 (0/8) |
| years tools cost over twice the middling price of the 25 years about them | 8.5 | 3.9 (0/7) | 0.25 | 0.25 | 0.1 |
| steel rails' share of the rails made in 1890 | 0.002 | 0.87 (7/0) | nothing | nothing | 0.23 (5/1) |
| converter steel's share of steel, 1880 | 0.86 | 0.88 | 0.94 | 0.94 | 0.95 |
| the first railway opens | 1827 | 1831 (3 later, 5 earlier) | 1829 | 1829 | 1836 (5 later, 3 earlier) |
| people in 1900 over 1750 | 5.42 | 5.37 | 5.05 | 5.04 | 5.04 |
| the money wage of 1881 to 1900 over 1750's | 2.88 | 3.01 | 2.47 | 2.61 | 2.69 |
| the cost of the basket of 1750, 1881 to 1900 over 1750 | 1.10 | 1.04 | 0.80 | 0.79 | 0.84 |
| towns in 1900; the last founded | 17.8; 1847 | 16.9; 1856 | 15.6; 1862 | 16.5; 1871 | 15.5; 1879 |
| works standing in 1900 | 731 | 725 | 985 | 1,068 (8/0) | 1,081 (7/1) |
| famine years in a run | 0.375 | 0.5 (3/2) | 0.125 | 0.125 | 0.125 |
| real income in 1750, bundles a head | 1.56 | 1.57 | 1.62 | 1.62 | 1.62 |

What the runs say.

- **The numbers do not fight.** On today's island the four together give what each gave alone on its page: late growth from the ceiling, calm from the hands, a steadier iron price from the brake and a steel rail from the heir's skill, with the start of the century untouched.
- **With the tree in they do more for the swings and less for the level,** as section 4.9 says of the ceiling. The wage's five worst years, its swing, timber, iron, grain and clay are all lower on every island of eight than with the tree alone.
- **The tree alone already grows to 1900** (up 29 per cent over the last two decades, and still rising in its last years on every island), and already has no famine of tools.
- **The steel rail is the miss.** 0.23 of rails in 1890 with the tree, and the mill never built on one island of eight.
- **The late draw is still there.** In `m-design` one island of eight never starts its steel (section 4.9). In `t-design` the engineering works is never built on three islands of eight and the engine works on two; with the tree alone the tree's own page has four and two of sixteen. Stage 7 is for this.
- **The smaller room costs the start** on the tree's island too: with it in place of the faster hands (`t-combo` against `t-design`) what a wage buys in 1781 to 1800 is 1.42 for 1.56, lower on eight islands of eight, and the towns' share of 1791 to 1810 is 0.337 for 0.361. It does give more steel rail (0.45 of rails in 1890 for 0.23), because it keeps iron dear.
- **A fault in the tree's sweep, found on the way.** A sweep that mixes stages (`--set tree.stage=0,1`) writes `annual.csv` under one header, the first setting's, and the tree's rows have more series than that header has names: every column after `out_services` is out of line for them (the chained index reads 0). The stage 1 half of `t-cap` was thrown away for that reason and run again alone as `t-cap1`. `facts.csv` names its facts and is not affected. It is for the tree's reviewers.

Three pictures were drawn from these sweeps and are kept beside them, outside the repository: what a head spends over the chained cost of living under the four settings of the table; the share of people in towns under the same four; and what a wage buys with the smaller room, the faster hands and both.

## 9. What I am least sure of

- **How much the freight by the wage costs.** It is the one large change here that nothing like it has run. The table in section 4.6 is read from a freight doubled from 1750, which is a different experiment: it makes the early century dearer and the late century less dear than the rule will, and it was run without the tree. The direction is not in doubt and the size is.
- **That the fold comes to a half.** The towns' share with everything in is three numbers added, each from a different run: 0.47 from the tree with the settable numbers, a point off for the freight, three or four on for the carriers. If it comes to 0.47 the aim is missed by its tolerance, and the next lever is the one section 4.12 turned down, the 0.8.
- **The faster hands on the land.** They are the decision this page's own runs changed, on eight islands and one evening. The swings they calm are calmed on every island and by a third or more, which eight islands can show. What eight islands cannot show is whether hunger rises with them or the railway comes later, and both leaned the wrong way. Stage 9 is read on sixteen with a null pair for that reason, and section 4.10 says what is taken back if either holds.
- **An island that grows by nearly half in its last twenty years.** The tree and the lifted ceiling together are faster at the end than anything the documents have looked at. It is inside the card, and it may still read as too much.
- **The steel rail.** With the tree in the pass does not deliver it, and before the tree it delivers it fifteen years early, because the mill is found in 1846. The honest cure is a railway that wears out, which is not here.
- **That mending the wild wage changes nothing else.** Every table on the pages was measured with it in. The mend is small and the reasoning for it is plain, but the labour market is the one place where every mechanism meets.
- **Eight islands for the runs of this page.** Section 3 says sixteen, twenty-year means and a null pair. The runs of section 8 have the means and not the rest, and are there to see whether the numbers fight one another and which way the large things go, not to fix a figure.
