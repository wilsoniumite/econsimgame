# dx-inspector: a market's trades as a flow, from the books of account

Branch `night/o-dx-inspector`, from `main` at 11dbe7d (12 November 2026). Stella's note 13 asks
for ways to see what is going on, so that she can say whether it makes sense: "Flowcharts of
industries within a region or market with prices and volumes and transport costs". The books of
account (`docs/rules/accounts.md`) landed the same day and hold what such a chart is drawn from.
This probe reads them for three towns, holds their figures against one another, and designs the
inspector, as text, with the real figures in it.

The only code is an example, which reads the books and changes nothing:

```bash
cargo run --release -p isostate-core --example flows -- [--seed 7] [--rules game] [--families 6]
    [--end-year 1880] [--years 1800,1880] [--survey] [--towns NAME,NAME,...] [--good iron]
    [--works] [--history] [--weigh]
```

Every figure below is from island 7 under the game's rules with six families, the island and
the rules a new game is played on, run to 1880 so that the works' materials good by good (kept
for the last complete year only) are 1880's. A run to 1900 gives the same figures for 1800 and
1880, line for line. Money is in the island's units, as the books keep it; quantities are in
each good's units; a price is "a unit". Outputs are on the machine, under
`.claude/night/reports/opus/dx-inspector/`.

## In short

1. **The books answer the question as it was asked.** For any town and year they say what it
   made and with how many hands, what came in and went out, from and to which towns, at what
   price and with what carting, and who was paid. Every identity they promise holds in the
   eight town-years read here to single precision, the worst being a tenth of a unit of coal in
   3.6 million (section 1.5).
2. **What does not balance is outside the month, and the books say so.** Households' and
   merchants' cash also move at the year's end (savings put into works, the cash of works that
   close, people who move); dividends are paid out of cash, not out of the year's profit; and a
   flow's own price is not the year's mean price. None is an error. Each changes how a figure
   must be shown.
3. **The flows are worth looking at.** Read this way the island shows at once things no chart
   in the viewer shows: Garside sells iron at 3.45 a unit in 1800 to towns where it fetches 4.4
   to 4.8; Cardale sells cloth at 3.08 in 1880 to Garside and Stratbank, where it fetches 5.67
   and 5.49, and buys coal from five towns while it sends coal to ten; Glenbrook in 1800 sends
   98 per cent of the clay it digs away and buys 28,065 bricks, with 19,038 more asked for and
   not had.
4. **And one thing that is wrong.** In 1880 the merchants of six towns hold less than nothing,
   Elmwick's 2.48 million and Cardale's 2.19 million. Cardale has had no shops of its own since
   1835 and brings its services in from eighteen towns; half of what is in store spoils every
   month, and its merchants paid 337,310 in 1880 for services of which they sold 12,783 units at
   11.78 (end of section 1.5). The trade rule never asks whether the buyer can pay or the good
   keeps.
5. **The inspector** (section 2) starts from the card the viewer already shows when a town or
   a works is clicked: three lines of the year's trade, and a sheet behind them with what came
   in, what was made and what went out as bands of ink, a good's own ledger on a click, one
   good across the island as lines on the map, and a works' account. It shows the year the map
   shows, says which figures are means, and gives no advice.
6. **The viewer needs nothing from the record for a live game** (section 3). One town's books
   take 0.02 ms to make on the simulation's thread and are some 60 kB as JSON (less as the
   struct itself); a works' about a tenth of a millisecond and 20 to 24 kB. They can be asked for on a click, as a
   square's prospects are today. The smallest useful piece is the card's three lines, then the
   sheet. Only a recorded run, and the map's lines of traffic, need more.

## 1. Three towns, from the books

The towns: **Cardale**, a farm town (founded 1791; the land makes 81 per cent of the value it
makes in 1800 and 60 in 1880, wool, grain and stone, and cloth from its mills by 1880);
**Garside**, the iron and coal town (the cheapest iron on the island in 1800 and the most iron
in 1880, and 2.44 million units of coal that year); and **the largest**: Glenbrook
in 1800 (28,402 people) and Stratbank in 1880 (72,934), which was itself a farm-and-coal town
of 11,465 in 1800. `flows --survey` lists every town in both years; `flows --towns
Cardale,Garside,Glenbrook,Stratbank --works` prints all that follows, and more (3,408 lines).

How to read the tables. *Made* is valued at the town's price in the month it was made, added
up over the year; the merchants pay the maker 94 per cent of that. *Came in* is valued where
it left, at the sending town's price in the months it went, which is what the town's merchants
paid for it; they paid the carting on top, to the sending town's carters. *A unit there* is the
flows' value over their quantity; *carting* is carting over quantity; *the price here* is the
receiving town's mean. *Went out* is the same from the other end; where *the price there* is
a range, it is of the towns that took the eight largest flows, which `flows` lists for each
good.

The tables do not show every good. What was made shows the larger goods, mostly the largest
by value, and puts the rest in one line, so that the column adds up to "all of it". What came
in and what went out show five goods each, the five largest by value except in three tables,
where a smaller good stands in for the fifth: iron for tools in what came into Cardale in 1800,
steel for wool in what went out of Glenbrook in 1800, iron for wool in what went out of
Garside in 1880. `flows` lists them all.

### 1.1 Cardale, the farm town

**1800.** 5,301 people, the wage 4.17 a month, 1,802 hands on the land, 357 in works, 582
carting.

| made | units | value | by |
|---|---|---|---|
| wool | 35,420 | 44,786 | pasture, 532 hands |
| grain | 71,161 | 41,544 | farms, 561 hands |
| stone | 73,202 | 38,826 | quarries, 460 hands |
| services | 1,822 | 17,885 | 2 shops, 291 hands |
| timber | 16,766 | 13,642 | woods, 170 hands (543 of it from clearing, which the value leaves out) |
| cloth | 1,389 | 7,536 | 1 mill, 8 hands |
| 5 more goods | | 16,016 | coal, steel, machinery, iron, iron ore |
| all of it | | 180,235 | |

| came in | units | from | chiefly | a unit there | carting | the price here |
|---|---|---|---|---|---|---|
| grain | 45,424 | 6 towns | Melbourne 23,575 | 0.464 | 0.120 | 0.581 |
| services | 1,261 | 8 | Granness 427, Mallowwick 419 | 9.063 | 0.630 | 9.919 |
| coal | 27,986 | 4 | Greyside 10,078 | 0.291 | 0.139 | 0.592 |
| bricks | 2,140 | 7 | Skelford 1,249 | 2.244 | 0.228 | 2.677 |
| iron | 961 | 12 | Garside 236 | 4.084 | 0.207 | 4.395 |

| went out | units | to | chiefly | a unit here | carting | the price there |
|---|---|---|---|---|---|---|
| wool | 14,388 | 12 towns | 1,170 to 1,372 to each of eight | 1.265 | 0.096 | 2.29 to 2.47 |
| stone | 27,439 | 12 | Granness 4,009 | 0.530 | 0.212 | 0.93 to 1.29 |
| timber | 7,590 | 12 | Granness 901 | 0.836 | 0.172 | 1.45 to 1.76 |
| coal | 9,933 | 7 | Glenbrook 2,137, Melbourne 2,123 | 0.591 | 0.178 | 0.81 to 1.66 |
| steel | 258 | 12 | Stratbank 39 | 18.72 | 0.664 | 20.2 to 20.9 |

Cardale's wool leaves at half what it fetches where it goes; its coal comes in at 0.29 and
goes out at 0.59, to dearer towns again.

**1880.** 8,960 people, the wage 7.78, 3,063 on the land, 717 in works, 842 carting.

| made | units | value | by |
|---|---|---|---|
| stone | 584,314 | 209,239 | quarries, 1,278 hands |
| cloth | 58,909 | 181,262 | 3 mills, 208 hands |
| wool | 60,801 | 158,458 | pasture, 1,016 hands |
| grain | 240,857 | 71,670 | farms, 435 hands |
| machinery | 952 | 36,174 | 3 engine works, 125 hands |
| coal | 94,424 | 34,283 | pits, 243 hands |
| iron | 8,543 | 29,394 | 3 coke furnaces, 75 hands |
| 6 more goods | | 84,868 | bricks, steel, timber, tools, iron ore, rails |
| all of it | | 805,348 | |

| came in | units | from | chiefly | a unit there | carting | the price here |
|---|---|---|---|---|---|---|
| services | 30,744 | 18 towns | | 10.463 | 0.509 | 11.776 |
| steel | 5,744 | 18 | | 5.396 | 0.205 | 5.374 |
| bricks | 16,132 | 15 | | 1.667 | 0.112 | 1.686 |
| coal | 115,222 | 5 | Kelside 35,960, Greyside 34,245 | 0.210 | 0.071 | 0.361 |
| grain | 63,200 | 6 | Skelford 26,940 | 0.231 | 0.033 | 0.299 |

| went out | units | to | chiefly | a unit here | carting | the price there |
|---|---|---|---|---|---|---|
| stone | 211,862 | 16 towns | Melbourne 20,658 | 0.358 | 0.108 | 0.70 to 0.78 for the first two |
| cloth | 19,647 | 18 | Garside 1,275 | 3.076 | 0.106 | 5.67 at Garside, 5.49 at Stratbank |
| wool | 21,563 | 17 | Melbourne 1,379 | 2.603 | 0.106 | 6.41 at Melbourne |
| steel | 4,485 | 18 | Salford 501 | 5.134 | 0.202 | 5.84 at Salford |
| coal | 58,773 | 10 | Glenbrook 13,784 | 0.362 | 0.081 | 0.77 at Glenbrook |

Steel, bricks and coal both come in and go out. Cloth and wool leave at half the price they
fetch where they go, as wool did eighty years before.

### 1.2 Garside, the iron and coal town

**1800.** 13,059 people, the wage 6.12, 3,821 on the land, 1,462 in works, 1,031 carting.

| made | units | value | by |
|---|---|---|---|
| grain | 251,150 | 142,276 | farms, 1,281 hands |
| stone | 125,745 | 114,705 | quarries, 982 hands |
| coal | 279,369 | 90,624 | pits, 524 hands |
| wool | 32,241 | 73,918 | pasture, 581 hands |
| services | 5,590 | 61,365 | shops, 644 hands |
| bricks | 17,292 | 45,001 | 3 brickworks, 226 hands |
| iron | 7,548 | 26,030 | 4 coke furnaces, 71 hands, at full capacity |
| 8 more goods | | 157,886 | timber, cloth, machinery, rails, steel, tools, clay, iron ore |
| all of it | | 711,805 | |

| came in | units | from | chiefly | a unit there | carting | the price here |
|---|---|---|---|---|---|---|
| services | 5,573 | 12 towns | Brambridge 1,468 | 9.396 | 0.698 | 11.072 |
| cloth | 7,052 | 12 | Sandmere 1,432 (at 4.94) | 6.060 | 0.213 | 6.861 |
| grain | 49,814 | 4 | Sandmere 21,278 | 0.442 | 0.102 | 0.565 |
| wool | 6,216 | 9 | Dunworth 1,234, Cardale 1,204 (at 1.26) | 1.950 | 0.131 | 2.296 |
| steel | 470 | 12 | | 19.065 | 0.682 | 20.363 |

| went out | units | to | chiefly | a unit here | carting | the price there |
|---|---|---|---|---|---|---|
| stone | 35,482 | 8 towns | | 0.895 | 0.239 | |
| coal | 96,875 | 9 | 14,209 to each of five | 0.324 | 0.178 | 1.16 to 1.66 for those five |
| wool | 5,545 | 9 | | 2.189 | 0.117 | |
| bricks | 4,521 | 10 | | 2.603 | 0.293 | |
| iron | 3,050 | 12 | Melbourne 324 | 3.449 | 0.215 | 4.40 to 4.77 |

The furnaces' ore cost them 0.304 a unit delivered and their coal 0.292, both a little under
the town's price, since the furnaces stand nearer the pits than the market does. A unit of coal
sent to Melbourne was bought at 0.32 in Garside and cost 0.17 to carry; Melbourne's price that
year was 1.63 on the mean.

**1880.** 58,220 people, the wage 11.36, 11,831 on the land, 14,373 in works, 2,702 carting.

| made | units | value | by |
|---|---|---|---|
| services | 146,921 | 1,653,363 | shops, 10,788 hands |
| stone | 2,397,620 | 1,396,258 | quarries, 6,572 hands |
| coal | 2,440,098 | 625,808 | pits, 2,632 hands |
| cloth | 94,923 | 537,868 | 5 mills, 315 hands |
| steel | 79,189 | 429,754 | 10 converters, 926 hands |
| iron | 83,882 | 305,752 | 5 coke furnaces, 614 hands |
| 9 more goods | | 1,039,777 | wool, grain, machinery, bricks and five more |
| all of it | | 5,988,580 | |

| came in | units | from | chiefly | a unit there | carting | the price here |
|---|---|---|---|---|---|---|
| cloth | 93,715 | 18 towns | Glenbrook 15,547 | 5.037 | 0.166 | 5.673 |
| grain | 1,259,717 | 17 | Melbourne 254,265, Sandmere 204,859 | 0.263 | 0.046 | 0.361 |
| wool | 26,952 | 18 | Salford 3,643 | 5.388 | 0.189 | 6.215 |
| services | 12,794 | 14 | Melbourne 2,903 | 10.506 | 0.470 | 11.227 |
| bricks | 75,311 | 18 | Melbourne 18,989 | 1.712 | 0.117 | 1.924 |

| went out | units | to | chiefly | a unit here | carting | the price there |
|---|---|---|---|---|---|---|
| stone | 675,201 | 6 towns | Melbourne 194,413 | 0.578 | 0.089 | 0.78 at Melbourne |
| services | 26,466 | 9 | Ormside 4,951 | 10.609 | 0.512 | 12.18 at Ormside |
| coal | 872,356 | 14 | Brambridge 102,864 | 0.256 | 0.075 | 0.61 at Brambridge |
| steel | 23,870 | 16 | Stratbank 3,255 | 5.258 | 0.203 | 5.99 at Stratbank |
| iron | 9,664 | 14 | Marwater 1,787 | 3.646 | 0.148 | 3.22 to 3.67 |

Of the 83,882 units of iron, 70,898 are used in the town: 54,480 by the converters, 9,279 by
the engine works, 5,205 by the rolling mills, 1,934 by the smiths. Garside feeds itself from
seventeen towns' grain.

### 1.3 The largest: Glenbrook in 1800, Stratbank in 1880

**Glenbrook, 1800.** 28,402 people, the wage 6.13, 8,436 on the land, 4,195 in works, 1,564
carting.

| made | units | value | by |
|---|---|---|---|
| grain | 802,181 | 430,553 | farms, 3,835 hands |
| services | 36,522 | 364,519 | shops, 3,366 hands |
| cloth | 32,624 | 212,872 | 5 mills, 123 hands |
| wool | 88,437 | 207,408 | pasture, 1,637 hands |
| stone | 145,319 | 170,782 | quarries, 1,518 hands |
| clay | 76,211 | 34,997 | pits, 478 hands, at a rent of -5,709 |
| 6 more goods | | 268,857 | timber, steel, tools, iron, machinery, iron ore |
| all of it | | 1,689,988 | |

| came in | units | from | chiefly | a unit there | carting | the price here |
|---|---|---|---|---|---|---|
| bricks | 28,065 | 12 towns | Melbourne 6,558 | 2.634 | 0.300 | 3.219 |
| coal | 124,609 | 11 | Kelside 26,893 (at 0.345), Granness 22,543 (at 0.807) | 0.534 | 0.188 | 1.662 |
| wool | 8,494 | 9 | Kelside 2,839 | 1.982 | 0.116 | 2.329 |
| cloth | 2,906 | 10 | Sandmere 1,158 | 5.703 | 0.221 | 6.521 |
| stone | 14,462 | 6 | Kelside 6,614 | 0.849 | 0.190 | 1.175 |

| went out | units | to | chiefly | a unit here | carting | the price there |
|---|---|---|---|---|---|---|
| cloth | 9,444 | 10 towns | Dunworth 1,442 | 6.252 | 0.225 | 6.94 at Dunworth |
| services | 4,376 | 8 | Stratbank 1,354 | 9.869 | 0.794 | 11.23 at Stratbank |
| grain | 59,867 | 4 | Kelside 49,667 | 0.528 | 0.100 | 0.67 at Kelside |
| clay | 74,746 | 11 | Skelford 25,018 | 0.411 | 0.164 | 0.73 at Skelford |
| steel | 917 | 11 | Kelside 168 | 19.352 | 0.713 | 20.90 at Kelside |

Glenbrook has no brickworks. It digs clay at a loss to the land (the pits' wages and carting
come to more than the clay sells for), sends 98 per cent of it away, and brings bricks in:
19,038 more were asked for than could be had.

**Stratbank, 1880.** 72,934 people, the wage 11.44, 15,205 on the land, 17,807 in works, 2,842
carting.

| made | units | value | by |
|---|---|---|---|
| services | 205,323 | 2,246,710 | shops, 14,621 hands |
| stone | 1,775,347 | 1,116,225 | quarries, 5,168 hands |
| cloth | 199,479 | 1,094,200 | 8 mills, 607 hands |
| grain | 2,605,633 | 848,890 | farms, 3,746 hands |
| coal | 3,380,631 | 773,270 | pits, 3,240 hands |
| wool | 88,071 | 562,612 | pasture, 2,444 hands |
| 9 more goods | | 1,056,863 | steel, iron, bricks, machinery and five more |
| all of it | | 7,698,770 | |

| came in | units | from | chiefly | a unit there | carting | the price here |
|---|---|---|---|---|---|---|
| wool | 41,457 | 17 towns | Kelside 5,969 | 5.532 | 0.197 | 6.379 |
| cloth | 46,861 | 18 | Glenbrook 7,111 | 4.891 | 0.166 | 5.493 |
| bricks | 70,566 | 18 | Melbourne 13,989 | 1.726 | 0.156 | 1.952 |
| steel | 18,026 | 18 | Garside 3,255 | 5.367 | 0.218 | 5.992 |
| stone | 150,033 | 5 | Kelside 119,430 | 0.510 | 0.072 | 0.629 |

| went out | units | to | chiefly | a unit here | carting | the price there |
|---|---|---|---|---|---|---|
| services | 36,663 | 10 towns | Ormside 5,882 | 10.413 | 0.510 | 12.18 at Ormside |
| coal | 1,240,242 | 14 | Glenbrook 156,251 | 0.229 | 0.112 | 0.77 at Glenbrook |
| iron | 23,848 | 18 | Sandmere 2,943 | 3.308 | 0.162 | 3.62 at Sandmere |
| grain | 217,636 | 2 | Kelside 217,525 | 0.326 | 0.028 | 0.39 at Kelside |
| cloth | 6,593 | 6 | Garside 5,029 | 5.471 | 0.180 | 5.67 at Garside |

The island's largest town makes nearly a third of its value in its shops, digs more coal than
any other town, and brings 150,033 units of stone in, most of it from Kelside, while it sends
8,987 out.

### 1.4 Who earns what

From the town's own figures (`TOWN_FIELDS`) and its methods' books, in money of the year.

| | Cardale 1800 | Cardale 1880 | Garside 1800 | Garside 1880 | Glenbrook 1800 | Stratbank 1880 |
|---|---|---|---|---|---|---|
| **the households received** | 149,363 | 484,580 | 535,696 | 4,429,894 | 1,324,657 | 5,380,535 |
| pay in works | 17,815 | 67,030 | 107,418 | 1,959,606 | 308,714 | 2,444,193 |
| pay on the land | 90,199 | 285,887 | 280,641 | 1,611,350 | 620,835 | 2,085,653 |
| carting (for works, of the land's produce, of what the town sent) | 28,753 | 82,034 | 75,369 | 363,740 | 118,421 | 397,848 |
| dividends of works no family owns | 7,377 | 42,120 | 55,838 | 320,122 | 135,079 | 259,631 |
| the merchants' dividend | 2,172 | 0 | 9,408 | 128,116 | 125,825 | 135,006 |
| new money | 3,048 | 7,508 | 7,022 | 46,959 | 15,784 | 58,204 |
| **the families**: dividends of their works here | 0 | 0 | 0 | 15,907 | 93,992 | 79,461 |
| **rent**: of farms, woods, pits | 30,484 | 97,556 | 94,997 | 444,828 | 191,344 | 673,951 |
| of works' ground | 128 | 1,897 | 1,130 | 39,042 | 11,080 | 53,326 |
| of town houses | 2,421 | 16,370 | 15,265 | 289,549 | 59,137 | 405,526 |
| of the three rents, to families | 2,555 | 9,444 | 9,765 | 78,552 | 83,804 | 0 |
| **the works' profit**, after managers | 10,328 | 84,504 | 70,356 | 433,085 | 267,003 | 445,008 |
| **the merchants**: took in less paid out | 14,979 | -134,458 | 3,502 | 117,778 | 107,184 | 141,754 |
| their cash at the year's end | 34,658 | -2,191,248 | 101,065 | 994,632 | 331,897 | 1,275,741 |

Where a unit of value made went (`flows` prints it for every town-year; the ranges are of
these six): on the land, about 0.06 to the merchants, 0.59 to 0.65 to wages, 0.06 to 0.11 to carting, up to 0.07 to machines
and rails, and 0.17 to 0.21 to rent; in the works, 0.06 to the merchants, 0.21 to 0.58 to
wages, 0.10 to 0.45 to materials, and 0.10 to 0.37 to profit, profit falling as the century
goes on (Garside 0.30 in 1800 and 0.13 in 1880; Glenbrook 0.37 in 1800).

### 1.5 What balances and what does not

`flows` holds every figure it prints against the others. In all eight town-years (the four towns
in both years) these hold to single precision, less than a part in ten million (the worst is 7.4
parts in a hundred million, Cardale's pay in works in 1880; a method that made nothing, as
Glenbrook's steel rail mill in 1880, prints a share of 2e-3 of a difference that is itself
nought):

- for every good, made, received, moved and oversold against bought, sent, spoiled and the
  change in stock (the worst left over is 0.127 units of coal of 3.59 million through Stratbank
  in 1880);
- made against the land's and the works' output and the timber from clearing;
- sent and received against the flows, good by good;
- each land sector's value less the merchants' spread against its wages, carting, machines and
  rent; each method's against its materials, fuel, carting in, wages, ground rent, managers and
  profit, and its materials, fuel and carting in against what its goods cost delivered;
- the town's figures against the sums of its sectors and methods (pay, carting, rents, hands,
  managers, dividends), and the carting the town's households earned on trade against the
  carting on what it sent;
- the works' own rows, added up, against the methods (output, wages, profit, dividends, and the
  town's investment);
- the merchants' net less their dividend against the change in their cash, to a tenth of a
  unit, in six of the eight.

What does not, and why:

1. **The households' purse.** What they received less what they spent on goods and house rent
   is not the change in their cash: it is out by 171 in Garside in 1800 and by 88,677 in
   Garside in 1880. Their cash also moves at the year's end, outside the month the books
   watch: savings drawn into new works and railways, the cash of works that close, cash that
   goes with people who move, emigrants' fares. The books' page says so ("Capital movements are
   not in the income account"), and names the table that would follow them.
2. **The merchants' purse, twice.** Out by 326 in Garside in 1800 and by 11,461 in Glenbrook in
   1880. Merchants take on a closed works' debts, and their cash moves with districts that
   change market and with emigrants' fares, all at the year's end.
3. **Dividends above the year's profit.** Ten method-years pay out more than they made:
   Cardale's coke furnaces in 1800 paid 1,386 on a profit of 91; Glenbrook's shops in 1880
   paid 67,929 on 58,463. A works pays its dividend out of its cash. Both figures are true, and
   the inspector should show both, not a share.
4. **A flow's price is not the year's price.** A flow is valued at the sending town's price in
   the month it went; the town's price in the books is the mean of twelve month-end prices.
   Garside's iron swung from 2.90 to 4.36 in 1880, and what went to Glenbrook went at 3.92 a
   unit against Garside's mean of 3.54, delivered at 4.12 where Glenbrook's mean was 3.60. Read
   against the year's means that trade lost money; in its months it did not, since a flow goes
   only when the gap between the two towns' prices that month beats the carting (`econ.rs`,
   the trade block). The inspector has to show a flow at its own price, and call the means
   means.
5. **Flows of nothing.** The books keep every flow above zero, so a town-year has 2 to 18 flows
   of less than a twentieth of a unit (charcoal to eight towns from Cardale in 1800, at 0.000
   units; Glenbrook brought in fifteen such in 1800). They are true and say nothing; the inspector should leave them out and count them.
6. **Nothing here came with districts or was sold out of thin air.** `moved` is 0 and
   `oversold` 1e-13 or less of what was bought, in all eight.

**Found along the way: merchants in debt for services that spoil.** In 1880 the merchants of
Elmwick (-2,484,120), Cardale (-2,191,248), Skelford (-1,052,297), Coldmoor (-232,248), Ormside
(-47,407) and Brackbourne (-4,101) hold less than nothing (`flows --survey`). Cardale's held
34,658 in 1800 and went below nothing in 1825, the years its own shops closed (services made:
1,822 in 1800, 211 in 1830, none from 1835; `flows --history --towns Cardale`). Since then
the town brings its services in: 30,744 units in 1880 from eighteen towns, costing 337,310
delivered, of which 15,826 spoiled in store (services lose half of what is in store each month)
and 12,783 were sold, at 11.78. The merchants lost 134,458 that year, twice what the town's
works paid in wages. The trade rule (`econ.rs`, the trade block) sends a share of the sender's
stock wherever the price gap beats the carting; it asks neither whether the buyer's merchants
can pay nor whether the good keeps. Money is conserved and nothing in the economy minds. It is
the port's rule, so this probe changes nothing; it is a finding for the backlog.

## 2. The inspector

### What it is for, and what it is not

It shows what is: what a town made, brought in and sent out in a year, at what prices and with
what carting, and who was paid. It gives no advice, offers no action and draws no conclusion.
Every figure in it is one the books hold, or a sum or quotient of them that is named as such
("a unit", "the year's mean"). Its manner is the game's: paper cards, ink lines, the Fredoka
hand, muted figures beside dark ones, no colour that means good or bad.

It grows out of the inspector card the viewer has today, bottom right, which says what a
clicked town or works is (`main.rs`, `inspect_panel` and `describe`). It always shows the year
the map shows: in a game stopped in June 1881 that is 1880, the last whole year, and it says
so; looking back it is the year looked at. It asks for that year by name, never for "the last",
because for a moment at the turn of a year the simulation has closed a year the player has not
yet been shown.

### At a glance: a town's card

A click on Garside in a game whose map shows 1880. The card says what it says today and three
lines more.

```text
  Garside
  founded 1763
  58,220 people in its market area, … in the town, in 1880.          (today's lines)

  The year's trade, 1880
    made here       5.99 M     services, stone, coal and 12 more goods
    came in         1.28 M     from 18 towns; 99 k paid to carry it
    went out        1.26 M     to 18 towns; Garside's carters paid 157 k
                                                                  the books ›
```

(The first three lines are today's card, from the record, and stay as they are; the people in
the town itself are not in the books, so the mock-up leaves that figure out.) The three money
figures are the year's value of what was made at Garside's prices, of what came in at the
prices where it left, and of what went out at Garside's. Nothing is compared and nothing is
ranked.

### On a click: the town's books

"the books" opens a sheet over the map, as the Markets panel does. Three columns joined by ink
bands, each band as wide as its value; the goods are rows; hovering a band names it.

```text
  Garside · the books of 1880                          58,220 people · the wage 11.36 a month

   CAME IN, worth where it left       MADE HERE, at Garside's prices      WENT OUT, at Garside's prices

   cloth       93,715   472 k ━━━━━┓   services   1.65 M ━━━━━━━━━━━━   ┏━━━━ stone      675,201   390 k
   grain    1,259,717   332 k ━━━━┓┃   stone      1.40 M ━━━━━━━━━━     ┃┏━━━ services    26,466   281 k
   wool        26,952   145 k ━━┓ ┃┃   coal         626 k ━━━━━          ┃┃┏━━ coal       872,356   224 k
   services    12,794   134 k ━━┫ ┃┃   cloth        538 k ━━━━           ┃┃┃┏━ steel       23,870   126 k
   bricks      75,311   129 k ━━┫ ┃┃   steel        430 k ━━━            ┃┃┃┃  wool         9,053    53 k
   10 more goods         67 k ━━┛ ┃┃   iron         306 k ━━             ┃┃┃┃  9 more goods         185 k
                                  ┗┻━━▶   9 more goods 1.04 M        ━━━┻┻┻┛
   carried in for                 99 k                                     carried out for        157 k

   Who was paid      wages 3.57 M · carting 0.36 M · rent of land and works' ground 0.48 M
                     the works' profit 0.43 M · the merchants' takings less what they paid 0.12 M
```

The bands are a sketch of the idea, not of the drawing: what came in and what was made meet in
the town's store, and from it go what was used, bought and sent. What it holds is section
1.2's tables, drawn. The "who was paid" line names five payments that do not overlap: the
households' dividends come out of the works' profit and the merchants' takings, and the rent of
town houses out of what the households received, so the line leaves those out rather than count
them twice (the
households' 4.43 M of section 1.4 holds 0.45 M of dividends). It opens to section 1.4's column
for the town, as a list with the same words. Under it, small and muted, a line says how
many flows were too small to draw ("2 flows of less than a twentieth of a unit are not drawn").

### On a click: a good in the town

A click on "iron" anywhere in the sheet.

```text
  Iron in Garside, 1880          the price 3.54 a unit, the mean of the twelve month-ends
                                 2.90 in the cheapest month, 4.36 in the dearest

   in store at New Year           7,505
   made                          83,882    5 coke furnaces, 614 hands, 98 in a hundred of their capacity
   came in                        6,784    from 18 towns, at 3.52 a unit where it left, 0.15 to carry
   used here                     70,898    converters 54,480 · engine works 9,279 · rolling mills 5,205 · smiths 1,934
   went out                       9,664    to 14 towns, at 3.65 a unit here, 0.15 to carry
   spoiled in store                 149
   in store at the year's end    17,459
   asked for and not had          1,157

   went out to       units   a unit here   to carry   delivered   that town's price, the year's mean
   Marwater          1,787      3.52         0.13        3.65        3.54
   Mallowwick        1,614      3.59         0.15        3.74        3.67
   Brambridge        1,295      3.85         0.13        3.98        3.62
   Melbourne         1,279      3.77         0.16        3.93        3.66
   10 more towns     3,690
```

The table shows the flow's own price beside the other town's mean and does not subtract one
from the other: the second is a mean of months the iron may not have gone in (section 1.5,
point 4). The twelve months of the last complete year's price, make, sales and stock are in
the books too (`TownBooks::months`), and a small line of them under the heading would show
when the dear months were.

### One good across the island

From the Markets panel's goods list, or from the good's ledger ("iron across the island"). On
the map, as a view like the price map: every town that makes iron as a ring the size of what it
made, and every flow above a share of the year's traffic as an ink line from town to town, as
thick as its quantity, with a small mark at the receiving end. The legend card:

```text
  Iron in 1880
  made 536,352 in 19 towns; 142,692 carried between them, at 0.16 a unit to carry
  the year's mean price from 3.22 at Coldmoor to 3.67 at Mallowwick
  lines for the 93 flows of 500 units or more, two thirds of what was carried;
  221 smaller flows are not drawn
```

(`flows --good iron` counts them: of the year's 314 flows of iron, 274 are of 100 units or more
and carry 98 per cent, 93 of 500 or more carry 65 per cent, 30 of 1,000 or more carry 34 per
cent. Where to draw the line is for whoever builds it; the card says where it is.) A click on a
line:

```text
  Stratbank to Sandmere, iron, 1880
  2,943 units, at 3.35 a unit in Stratbank, 0.15 to carry, delivered at 3.50
  Sandmere's price that year 3.62, from 2.95 to 4.62
```

And a click on a town gives the good's ledger above. In 1800 the same view shows the iron of
twelve towns, with Garside's at 3.45 a unit going out to twelve towns where it fetched 4.4 to
4.8; in 1880 the lines run mostly out of Stratbank, Granness, Kelside and Garside.

### One works

A click on a works adds its account to today's card. Denton & Sons Ironworks, the largest of
Garside's coke furnaces in 1880 (`flows --towns Garside --good iron --works --years 1880`):

```text
  Denton & Sons Ironworks
  Coke blast furnace, since 1775; owned by Garside's savers, no family
  1880: made 28,711 iron with 207 hands, 98 in a hundred of its capacity of 2,444 a month

   the iron, at Garside's price              104,657
     to the merchants (6 in a hundred)         6,279
     carried in to the market                    292
   paid to the works                          98,085
     iron ore, 34,315 delivered at 0.20        6,942
     coal, 39,462 delivered at 0.24            9,475
     wages, 207 hands                         28,259
     rent of its ground                          548
     profit                                   52,863
   out of its cash: paid out 42,342; spent on new plant 52,362; cash at the year's end 57,853

   profit, 1840 to 1880   ▁▁▁▁▂▂▂▂▂▂▃▃▃▃▃▂▂▂▃▄▅▆█▇▅▄▃▃▅▆█▇▅▂▁▃▃▅▆█▆   8,102 to 66,614
```

Every line is a column of its books for the year (`WorksYear`), and the two materials are its
inputs by good (`WorksBooks::inputs`, kept for the last complete year only; for an earlier year
the card would give its method's use in the town, or leave the two lines out). The profit line
in the sketch is of 1840 to 1880, drawn from the books' figures; the card would draw every
year the works stood (from 1775 here) and name the least and the greatest, with their years on
a hover. Paid out and spent come to more than the profit, and the card says both without a share.

### What the inspector keeps to

- The year the map shows, asked for by name. Of the year in progress it says nothing until it
  closes, and says that ("the books of 1881 close in January 1882").
- A flow at its own price; a town's price as the mean it is, with the cheapest and dearest
  month beside it.
- Totals that add up: where it shows parts it shows the whole, and the parts are the books'.
- Units always: "units", "a unit", "hands", "a month".
- Flows too small to matter left out and counted.
- No colour, order or word that judges. Lists go by size, largest first; nothing is "best".
- Nothing that waits on a decision: no button but the ones that open and close.

## 3. What the viewer must be handed, what it weighs, and in what order to build it

### What it has today

The viewer keeps eight yearly series from the books (`acct_share_work` and the rest, group
"distribution"), written into the months by `apply_year` from `LiveYear::year_series`. Each
year's `LiveYear` also carries the island's whole income account (`LiveYear::accounts`, an
`IslandYear`), which the viewer does not read. Everything else, the towns' and works' books,
stays on the simulation's thread inside `save::Game`.

### What it must be handed

| for | what | how | weight (`flows --weigh`, the run to 1880, then to 1900) |
|---|---|---|---|
| a town's card and sheet, a good in the town | `accounts::town_books(st, C, town, Some(year))` | a request and a reply on the game thread's channel, as a square's prospects are asked for (`Cmd::Site` and `Msg::Site`) | 0.02 ms to make; 59 to 65 kB as JSON with names repeated in every row (the struct is less); 282 to 340 flows |
| a works' card | `accounts::works_books(st, C, firm)` | the same | 0.07 to 0.14 ms (three runs on a shared machine); 20 to 24 kB as JSON |
| one good across the island | every town's row for that good and the year's flows of it | `town_books` for each of the 19 towns, or a small new function in the core that returns one good's rows and flows | 19 x 0.02 ms; the year's flows are 2,946 to 2,999, of which 187 to 314 are iron's, a pass of 0.01 ms |
| the same in a recorded run | the books' arrays in `run.bin` (`acct_goods`, `acct_trade_*`, `acct_works*` and the rest) | read when the run is loaded; the header names every column | 47,322 to 48,204 numbers a year in the dense tables (0.19 MB, more than half of them nothing), 70 kB of flows and uses; 27 to 32 MB for the whole run; a `lite` pack has no works |

Nothing needs to go into `LiveYear` or `LiveMonth`, and a game played by the month pays nothing
for the books it does not open. A request costs the game thread a fiftieth of a millisecond,
behind a month's step of 2.5 ms or a year's end of 70 ms.

### In what order

1. **The year's trade on a town's card.** The viewer asks for a town's books of the map's year
   when a town is clicked, and adds the three lines (section 2, "at a glance").
   A request and reply on the channel, a dozen lines of text, a test in `live_tests.rs` that the
   books asked of a played game are those of a run to the same year. Half a day. It is the
   smallest piece that tells Stella something she cannot see now.
2. **The town's sheet and a good's ledger.** The sheet with its three columns of bands, the
   goods clickable, the good's ledger with its partners. A file of its own (`books.rs`), so as
   to stay out of `ui.rs` and `chart.rs`, which the production tree's panels are changing
   tonight. Two days.
3. **A works' account on its card.** `works_books` on a works click; the takings and where they
   went, the profit line. One day. Its little chart wants `chart.rs`'s painter; until that file
   is free, a line of glyphs as above will do.
4. **Each flow's value where it arrived.** In the core, keep beside each flow its value at the
   receiving town's price in the month it arrived: one more number per flow, some 12 kB a year,
   a fourth column of `acct_trade_val`. Then the sheet can say what a flow was worth on arrival
   in its own months, and point 4 of section 1.5 goes away. It changes `accounts.rs`, `record.rs`
   and the run file, so it is checked both ways (the golden test, the sweep, the viewer's tests).
   Half a day.
5. **One good across the island, on the map.** Ink lines between towns, the legend card and a
   click on a line. It works in the map views, which belong to `overlay.rs`, so after the
   production tree's panels and legends land (Ready next 4 in the backlog works there too). Two
   to three days.
6. **Recorded runs.** Read the books' arrays from `run.bin`, so that a replay has the same
   cards and sheet; without works in a `lite` pack. A day. This, and traffic on the map, are
   what wait on the pass over the record (Ready next 6); a live game does not.
7. **The open year.** Later: the months of the year in progress are in the books and are not
   handed out (`accounts.md`, "What worries me"); a sheet for "1881 so far" would want them, and
   would have to say the same however the game reached June.

### What worries me

- The word: the viewer already calls its click card "the inspector", and this design keeps the
  name for the card and calls the rest "the books". Stella may want the whole to have one name.
- Three of its pieces want files that other workers hold tonight (`chart.rs`, `overlay.rs`, the
  legends). The first two pieces do not.
- A town's sheet in 1880 has 280 to 340 flows; the sheet draws goods, not flows, and a good's
  ledger lists its partners. Whether a town-to-town picture of all goods together would say
  anything is not tried here.
- Services travel between towns with a carting cost, as goods do, and spoil by half a month.
  That is the model's, and the inspector will show it as it is; section 1.5's finding is what
  it looks like.
- The figures of a year are complete only at its end, so in a game played by the month the
  sheet is behind the clock, as the map is: in June 1881 both are of 1880. It says so; it
  cannot do better until the open year is handed out.
- The finding of section 1.5 is the port's trade rule meeting a good that will not keep. The
  inspector shows it; whether and how to mend it is a question about the rules, not about the
  inspector, and is left for the backlog.
