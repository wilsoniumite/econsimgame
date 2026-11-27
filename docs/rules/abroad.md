# Ideas by sea

Every port has one more neighbour in the spreading of ideas: abroad, across a
sea. Not in the Python. The code is `abroad.rs`, with a few lines in
`tech::diffusion_with` and the tasks rule's year's turn (`tasks.rs`); the
table of years is `data/abroad.toml`. The switch is `abroad.ideas`: off under
both sets of rules, on with the landfall start ([landfall.md](landfall.md)).
A game may be begun without it (the viewer's `--ideas-by-sea off`, Abroad: Technology
unlit on the start page), and it is thrown in a running game from the
Rules tab (landfall.md, "The start's switches").
Built on 26 November 2026 from the study of the landfall start.

**Off is the isolated state.** The settled island is von Thünen's isolated
state, and it stays alone: nothing on this page touches it, and the game's
golden test holds it. With settlers off (`immigration.enabled=false`) and
ideas off (`abroad.ideas=false`), landfall is a colony alone, by two
switches: what an isolated people gets is measured below.

## Why

Before this rule nothing but people crossed the sea to the colony. An idea is
found when the island's own make of certain goods passes a threshold, mostly
fixed in tons (`tech::discovery`, the thresholds in `data/content.toml` and
`data/game.toml`), so a colony of thousands found 65 to 112 years late what
the settled island finds, and its railway came in 1890. History's colonies did
not find their techniques: settlers, books, machines and craftsmen carried
them, and the lag shrank across the period (a Newcomen engine in New Jersey
about forty years after the first, Slater's mill twenty years after
Arkwright's, the Baltimore and Ohio three years after the Stockton and
Darlington, Bessemer steel in America eight years after the patent; from
memory). The study gave the colony the settled island's ideas by hand and its
railway came in the 1830s and 40 per cent of the gap in the wage of 1900
closed.

The form matters more than the numbers. The design wants, later, two islands
on one map with a sea between whose cost can be set. Ideas already spread from
town to town by contact over the cost of the way. So abroad is one more
neighbour of every port, across a sea, and its know of each idea comes from a
table. When a real second island stands there it takes the table's place, and
no rule changes.

## What it does

Each year, after the island's own finding (`tech::discovery`, unchanged):

- **Abroad knows an idea** from the year `data/abroad.toml` gives, and its
  know grows as a town's does that has found an idea and has nobody to learn
  it from: a quarter in that year, then `diffusion_rate` (0.3) times what it
  knows times what it does not, each year (0.60 after five years, 0.87 after
  ten). `abroad::know`.
- **Each port has abroad for a neighbour.** A port is an active town on a
  coast cell, as the settlers' rule reads one. In the spreading
  (`tech::diffusion_with`) abroad weighs `exp(-sea / diffusion_dist)` beside
  the port's other neighbours, before the weights are shared out, so a port
  alone takes 0.05 of its contact from abroad and a port with close
  neighbours less: abroad is diluted as a town across a bay would be.
- **An idea abroad knows counts as found** for the spreading, so the port
  learns it from abroad and the island's other towns from the port.
- **It counts as found on the island** (`State::tech_year`, which the goods
  that begin with an idea, the machines and the tasks' uses read) in the year
  a town first knows it past 0.02 (`abroad::USE`), the know at which a works
  may first take a method. That is news: "Railway comes by sea to Port
  Hedley".
- **The uses of the tasks rule** (the hoist, the tractor, office machines and
  the rest, `data/tasks.toml`) have no know to spread: a use is found at the
  year's turn `abroad.lag` years after abroad has it, if the island has not
  found it first, its switch is on, its `found_after` has passed and the idea
  it waits on is found.
- **What the sea never brings**: an idea known in 1750, an idea that a rule
  keeps back while its switch is off (the tending machines' ideas, care,
  managed woodland), an idea before its `found_after`, and an idea that a pin
  by hand holds back (`edits::Pin`; what a pin does is unchanged).
- **The island's own finding stays.** The colony may find a thing before
  abroad, as history's colonies found labour-saving machines of their own: the
  coke furnace, crucible steel and improved husbandry are found on the island
  a few years before abroad's year on most islands, since their thresholds
  scale with the island's own early making.

Off, `abroad::sea` is None, the old `tech::diffusion` runs to the bit (a test
holds it), and nothing is read.

## The table

`data/abroad.toml` is made by `examples/abroad_table.rs` from a sweep:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --out DIR --quiet
cargo run --profile work -p isostate-core --example abroad_table -- DIR/facts.json > data/abroad.toml
```

Each idea of the tree and each use takes history's year, its `target`
(`data/game.toml` for the tree, `data/tasks.toml` for the uses), where the
content gives one, and never before its `found_after` (the hoist: target 1850,
found after 1901, so 1901). Seven have no target and take the median of the
years the settled island found them on islands 1 to 8 under the game's rules
with six families, an island that never found it counted last: the coke
furnace 1778, the textile mill 1785, the engine works 1782, crucible steel
1766, the rolling mill 1788, improved husbandry 1777, office machines 1901. An
idea whose median is "never" is a comment saying abroad does not know it;
today there is none. `tests/abroad.rs` fails when the table has gone stale
against the content: an idea or use the content has and the table neither
names nor marks, a line for one the content has not, or a line that is not its
target.

The table is read by name, once, so the tree's overlay and the order of the
content do not matter, and nothing new is kept in a saved game.

## Its numbers

| setting | value | why |
|---|---|---|
| `abroad.ideas` | on at landfall | the colony is a piece of the world its settlers come from |
| `abroad.sea` | 105 | in the units of the cost of the way between towns (a cell of lowland with no road is 1, of open sea 0.22): three times the distance over which contact falls by e, so a port alone weighs abroad at 0.05. The real Atlantic is some 550 of these units and would carry nothing; the number is set by the lag it gives, not the miles |
| `abroad.lag` | 5 years | the uses all come after 1900, with steamships and the cable: the tractor and the lorry reached the colonies of settlement within a few years of the old countries (from memory) |

What the sea's cost gives, measured on islands 1 to 8 (below): the colony
first uses an idea a median 8 years after abroad has it, and knows it as well
as a finder does (a quarter of a town) 16 years after. Early in the run, when
the port is alone, the first use comes 5 to 6 years after abroad and a quarter
11 to 14 years after; by the 1860s, with the port among many towns, 10 and 18.
History's lag ran the other way, shrinking from 10 to 40 years in the
eighteenth century to 3 to 10 by the middle of the nineteenth; see "What
worries me". The model of a lone port (`know` from nothing, contact 0.05 times
abroad's know) gives 4 years to first use and 12 to a quarter at 105, 3 and 9
at 70, 7 and 16 at 140: the lag is not very sensitive to the sea's cost,
since once a port knows a little its own know grows by itself.

## What the runs showed

Islands 1 to 8, landfall, the game's rules with six families, to 2050, with
carriage on prices from 1750 (`--set longrun.freight_from=1750`, not yet the
landfall start's own) and land that stays taken, with `examples/landfall.rs`.
"Before" is the start as it was with ideas off (`--set abroad.ideas=false`),
which gives the piece before this one's runs to the digit. Means of the eight,
least and greatest in brackets.

| | before (ideas off) | ideas by sea | the colony alone (no settlers, no ideas) | the settled island |
|---|---|---|---|---|
| first railway | 1863 (1842 to 1879) | 1848 (1846 to 1855) | 1889 (1866 to 1901) | 1828 (1815 to 1848) |
| wage in grain 1800 / 1850 / 1900 | 14.6 / 10.9 / 30.2 | 14.7 / 11.6 / 43.4 | 18.7 / 14.9 / 12.6 | 10.2 / 15.9 / 37.7 |
| wage in grain 1950 / 2000 / 2050 | 66.8 / 100.0 / 101.1 | 89.5 / 114.7 / 106.8 | 31.9 / 75.0 / 101.1 | 68.3 / 83.4 / 78.1 |
| linked wage 1800 / 1850 / 1900 | 1.18 / 1.31 / 2.24 | 1.21 / 1.43 / 3.45 | 1.41 / 1.38 / 1.51 | |
| linked wage 1950 / 2000 / 2050 | 6.30 / 8.94 / 10.03 | 8.08 / 10.12 / 11.12 | 2.69 / 6.83 / 10.03 | |
| the farm's multiplier 1850 / 1900 / 1950 | 2.38 / 7.01 / 19.91 | 2.51 / 10.14 / 22.54 | 2.37 / 2.38 / 7.35 | 10.3 in 1900 (the study, islands 7, 2, 5) |
| people 1900 / 2050 | 493k / 1,731k | 542k / 1,630k | 168k / 684k | 680k / 1,266k |
| years with hunger, to 1900 / after | 1.6 / 0.6 | 1.1 / 3.8 | 1.4 / 3.0 | 0.0 / 0.4 |
| works 1900 / 2050 | 944 / 2,974 | 1,042 / 2,955 | 641 / 1,197 | |
| runs failed | 0 of 8 | 0 of 8 | 0 of 8 | 0 of 8 |

The wage in grain is the island's mean wage over the national price of grain
(to set arms side by side); the linked wage is `real_wage_linked`, 1 in 1750
(for one run's own path); the farm's multiplier is the farm's part of output
that is technique, `econ::sector_tfp`, weighted by each town's farm hands; a
year with hunger is a year with a famine in the news.

What this says:

1. **The railway in the 1840s, on every island.** 1846 to 1855 against 1842 to
   1879. The idea comes by sea in 1833 or 1834 and a town knows it at a
   quarter, enough to promote a line, by 1841 or 1842; the first line follows
   when it pays. Twenty years after the settled island's and abroad's 1825, as
   the colonies of settlement had theirs (Canada 1836, Australia 1854, New
   Zealand 1863; from memory), and no island waits past 1880.
2. **Richer by 1900, and past the settled island after it.** The wage in grain
   in 1900 is 43.4 against 30.2, above the settled island's 37.7; the linked
   wage of 1900 is 3.45 against 2.24. The colony keeps its free land and gets
   the old country's techniques, so from about 1900 it lives better than the
   settled island (on island 7, the chart in the builder's pictures, from
   about 1940).
3. **The farm's multiplier is the settled island's by 1900**: 10.1 against
   7.0 before and 10.3 on the settled island. Farm machinery is known abroad
   from 1850 and comes to the colony in 1859 on the median island; on three
   islands of eight the colony found it first, 1 to 13 years before abroad.
4. **No more people**: 1.63 million in 2050 against 1.73. Land that stays
   taken and no settlers to a hungry port, the piece before this one, hold the
   colony; the study's colony with ideas and no such brake grew to five or six
   million.
5. **Hunger after 1900 rises**, 3.8 years an island against 0.6, and on three
   islands of eight (6, 9 and 10 years on islands 1, 5 and 7; 0 to 3 on the
   others). It is not the tasks' uses: with no use by sea (`abroad.lag=500`)
   islands 5 and 7 have 7 and 6 such years. Traced on island 7: every hungry
   town is a small one (90 to 8,600 people) in the emptied country of the
   east, hungry in March, with grain cheap (a month's bread is a hundredth of
   the wage) and one to two months of it a head in store. In January its
   households buy food for five to seven times their need; the town's own
   grain making falls fourfold at the year's turn and climbs back only by
   December; by March the merchants fill a tenth of what is asked, and the
   households get half their bread. So the hunger is the market of a rich small
   town at the year's turn, which the richer colony meets more often; the
   rule of the year's turn is its own piece (see "What worries me"). The
   colony alone has nearly as much of it: 3.0 years an island after 1900.
6. **The colony alone** is what an isolated people gets: richer than any arm
   in its first fifty years (18.7 in grain in 1800, land for everyone and few
   hands to share it), then held at 1800's techniques for a century (12.6 in
   grain and a farm's multiplier of 2.4 in 1900), its railway in 1889 and on
   one island not before 1901, and the ideas it finds for itself a median 75
   years after abroad. It catches up only after 1950, with 684k people in
   2050.

**The colony's lag behind abroad**, in years, the median of the eight islands
(found: the year the island has it, by the sea or by itself; a quarter: the
year a town first knows it as well as a finder does). The whole table, idea by
idea and island by island, is in the builder's notes of 26 November.

| idea (abroad's year) | with the sea, found / a quarter | before, found | alone, found |
|---|---|---|---|
| improved husbandry (1777) | -9 / -9 | -9 | -3.5 |
| crucible steel (1766) | -3 / -3 | -3 | -3 |
| the engine works (1782) | 6 / 13.5 | 27 | 57.5 |
| the rolling mill (1788) | 6 / 14 | 37.5 | 58 |
| the railway (1825) | 8 / 16 | 27 | 57.5 |
| the hot blast (1830) | 8.5 / 17 | 36 | 82 |
| the machine shop (1838) | 9 / 17 | 36.5 | 75 |
| farm machinery (1850) | 9 / 9.5 | 21.5 | 61.5 |
| the converter (1856) | 10 / 18 | 38 | 86.5 |
| the electrical works (1885) | 10 / 18 | 40 | 89.5 |
| offices (1906) | 2 / 2 | 11 | 61.5 |
| the motor vehicle (1905) | 10 / 18 | 53 | never on 5 of 8 |
| the uses: the hoist, the tractor, the combine (1901, 1915, 1935) | 5 | 24.5, 38, 29.5 | 92.5, never, never |
| median over the ideas found on most islands | 8 / 16 | 36 | 75 |

Ideas whose thresholds count the island's own early making ("xN": improved
husbandry, crucible steel, the coke furnace) the colony finds before abroad
in every arm. Everything priced in absolute tons it found a generation late
with settlers and two or three generations late alone.

## What worries me

- **The lag grows as the colony does**, from 5 years to first use in the
  1770s to 10 by the 1860s, because a port among many neighbours takes less of
  its contact from abroad. History's ran the other way (sail, then steam, then
  the cable). A sea whose cost falls with the century, or abroad weighed as a
  town of abroad's size rather than one more town, would turn it; neither is
  built.
- **Hunger after 1900** (above): the pass asked for at most 2 years an island
  and no more than the piece before (0.6); it is 3.8. The cause traced is not
  in this rule.
- **One number stands for the whole world abroad.** The sea is as wide for a
  port in the island's far corner as for the first town, and abroad knows an
  idea in one year for every island.
- **The table's seven medians are the settled island's**, and move when its
  rules do; the test catches a table gone stale against the content, not
  against the settled island's years.
- **Learning by doing and the machines' quality are not carried**: the colony
  learns to make a method well from its own making alone. The study found
  carrying it worth little.
- **Canals' own know-how** (`st.cn.know`, a separate rule) does not come by
  sea.
