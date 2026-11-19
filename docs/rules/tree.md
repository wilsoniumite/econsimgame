# Tree

A deeper production tree, first stage. Not in the Python. The switch is
`tree.stage`: 0 is the content as it was, 1 lays the first stage of the
game's overlay, `data/game.toml`, over it, and 6 (the game's) lays the first
content after 1900 on top, electricity and electrical goods ("After 1900:
electricity", below). The code is `tree.rs`, with hooks
marked "tree" in `content.rs`, `econ::step_month`, `sim.rs`, `land.rs`,
`observer.rs`, `families.rs`, `names.rs`, `save.rs`, `util.rs` and
`canals.rs`.

Stella's words on where the simulation should go: it "should be rich, fluid,
real economics, with deep tech and production trees", and she asked by name
for a logging industry. A panel designed the tree the game should have
(`docs/designs/production-tree.md`, with its draft overlay and its critique).
That design was a reckoning at fixed prices: nothing in it with a new good had
ever run. This page is what happened when it did.

## What was wrong

The port has sixteen goods and sixteen methods, and three things follow from
how few they are.

- **Shelter is bought as raw stone.** A household's shelter is a seventh of
  its spending and goes straight to the land: stone is the cheapest shelter
  per unit, so the money of a growing island is spent on a fixed field of rock
  with falling returns. With the mines as workings the quarries still hold
  7.6 per cent of all hands in 1750 and 13.6 per cent in 1900.
- **The century stops unfolding.** Every invention the port has is found by
  about 1860. Iron costs in 1900 what it cost in 1800, machinery ends the
  century at 62 a unit, and the real wage is 2.6 times 1750's.
- **Nothing a household buys is made in a town but cloth, tools and what the
  shops sell.** No brewer, potter, glassmaker or builder, so a town has
  little to be a town for.

## What it does

### The overlay

`data/content.toml` is the Python's file and stays as it is. The game's tree
is a second file laid over it by name:

- an entry (a good, a method, a sector, an invention, a row of the needs'
  table) whose name the content already has changes only the keys the overlay
  writes; a table written (`inputs`, `capital`, `discover`) replaces the
  content's table whole;
- a new name is added after the content's own, so the port's goods and
  methods keep their places and their numbers;
- every entry carries `stage`, and `tree.stage` is a ceiling: the entries
  with a stage no greater are kept. A good of a later stage does not exist,
  and an input, a line of plant or a threshold that names it is dropped. The
  stages are numbered in the order they are to be built.

`tree::lay(base, overlay_text, stage)` does it, and says what is wrong with an
overlay that does not hold together (a new good with no price, a method of a
family trade that is not one of the six, needs' lists of unequal length).
Stage 0 lays nothing and returns the content as given.

**Where it is laid.** `sim::start` lays it, on whatever content it is handed,
as far as `P.tree.stage` says. So `game_content()` is still the port's file
with the machines' changes, the stage is a parameter like any other, and
`--set tree.stage=0,1` works in a sweep. The price is that a caller who keeps
a content of its own beside the run's must ask for the laid one
(`tree::content(&P, rules.content())`, or `sim.content`): the sweep, the
record, the live API, the viewer and the families all read the run's own.

**The machines' content** (`content::machines_content`: the converter's and
the steel rail mill's plant, the steel in a rail, five thresholds) is left
where it is. It has to stay for stage 0 to be the game as it was, and its
tests name its constants. The overlay writes the same plant out again and
sets its own thresholds over the machines' five.

### What the engine had to learn for a seventeenth good

All of it reads as the port when the content does not say otherwise.

- **Decay and opening stock from the content** (`decay`, `stock0` on a good).
  The port's sixteen keep theirs from `P.decay` and the table in `sim.rs`.
- **A width of the logit for each need** (`theta` in `[services]`;
  `econ::service_mix_widths`). Household goods are unlike each other: at the
  port's 0.25 the first of pottery, glass, paper and soap to get a factory
  takes nine tenths of the need; at 2.0 each keeps about its share of the
  money.
- **Goods that are not carried** (`traded = false`): building work and town
  gas are made where they are used. They are left out of the monthly trade
  between towns, which also stops the engine making carters out of houses
  moved from the yard to the market.
- **The 1750 trades** (`seed = true` on a method; `tree::seed_plan`): the
  island opens with its masons, sawyers, lime burners, brewers, potters, soap
  boilers, glassmakers and paper makers in place, as the port opens with its
  smiths and weavers. `tree.seed_trades = false` founds them in the warm-up
  instead.
- **A family trade for each method** (`group`): `Group::of_method` was a
  match on the port's sixteen names. The builders, sawyers, potters,
  glassmakers and paper makers are building; brewers are estates; gasworks and
  the soap and alkali trades are coal; the tool and machine makers are
  engineering; hot blast is iron. Shops belong to no family, as before.
- **The observer's shares from the content** (`tree::share_groups`): a
  group for every need and every good with more than one way of making it,
  beside the port's own, and a line of news when a new way of making a good
  passes half of it.
- **Names for the new works** (`names::firm_patterns`): "Hartley & Co.
  Brewery" and "Ashgate Gas Light Co." instead of "Hartley & Co." for all of
  them.
- **Sums over goods**: `util::pairwise_sum_by` kept its terms on the stack
  for up to sixteen; it is 64 now, and the sum is the same sum.
- **The yearly delivered-price pass** is run only for goods some works buys
  (an input or a fuel); nothing reads the others' fields.
- **A save's fingerprint** is taken on the content with the overlay laid, so
  a change to `data/game.toml` shows.

Nothing else assumed sixteen goods, six needs or the port's method names in a
way that breaks: the record and the live API size their tables from the
content, the viewer's panels list the content's goods, and a works of a
method the art does not know is drawn as "any other works"
(`isostate-gen`, `Kind::Other`) until its own drawing exists.

### Plant waits

This is the one rule the tree adds to the monthly step, and it was not in the
design. `tree.plant_waits`; `econ::step_month`, marked.

Each month a town's market shares out what it has. Under the port everyone
who asked is cut alike: if the town has seven tenths of the bricks asked for,
the household, the works at work and the works being built each get seven
tenths. That is harmless while nothing that plant is built of is also a
works' daily input. With the tree bricks, sawn timber, lime and glass are
both, and it showed at once:

- The year's new plant is ordered as the year turns, and an order asks for a
  quarter of what it still lacks every month. So each spring the market was
  asked for several times the bricks it had (it had 70 per cent of what was
  asked from January to March in 1800 to 1849, 77 per cent in the rest of the
  year), and the bricklayers, who lay them every day, were cut with the rest.
- A works makes what its scarcest input allows. Contract builders use two
  hundredths of a unit of sawn timber in a unit of building; when the market
  had six tenths of the sawn timber asked for, they built six tenths of the
  houses. After 1850 the builders lost 9 per cent of their output this way on
  average, and in a year when timber was short they lost four tenths of it
  for a season. In every seed the worst year of the century for the real
  wage was such a year: timber and sawn timber up by three quarters to a
  double, building work up by a third to two thirds, the real wage down by a
  fifth.

So where a market is short, **the day's trade is served first** (households,
the works' own inputs and fuel, the land's machines), **and the orders for
new plant and for the materials of railways and canals share what is left.**
A works half built can wait a season for its bricks; a bricklayer with a
house half up cannot. Everything is still bought at the market's price and
paid for by the buyer; only the order of the queue changes. The price itself
still answers to everything asked for, the orders that wait included, so a
market that keeps plant waiting is a market whose price is rising: the
day's trade gives way to the price, and the orders are served out of what
it leaves. The sum is `tree::served_first`, with a test of its edges (an
empty market, a market nobody asks anything of, orders alone).

With it the builders' works have 99 per cent of what they ask for in every
half-century (98, 96 and 91 without), and the worst year of the real wage is
what it is without the tree (below). It acts only with a stage laid: without
the tree it measured as no gain (the real wage's yearly swing 0.062 against
0.053, up in all eight seeds, nothing else clear of the spread), so at stage 0
a run is the run without the tree.

### Stage 1: what is in the file

Nine goods, making 25; two needs, making 8; 28 methods, making 44; and five
of the port's methods touched. `data/game.toml` has every number with its
unit sums; in brief:

| good | made by (the year aimed at) | bought by |
|---|---|---|
| building work | masons and carpenters, timber-frame builders, bricklayers; contract builders (1862) | households and landlords, as nine tenths of their shelter |
| sawn timber | saw pit; steam sawmill (1825) | builders, new plant, households for furniture |
| lime and cement | lime kiln; cement works (1850) | builders, gasworks, glass and soda works |
| beer | brewhouse; common brewery (1785) | households: the whole of "drink and groceries" |
| pottery | country pottery; pottery manufactory (1770) | households |
| town gas | gasworks (1815) | the households of its own town, as fuel and light; shops and offices |
| soap and chemicals | soap boilers and ash burners; alkali works (1815); ammonia-soda works (1872) | households; glass and paper works |
| glass | glass cone; sheet glass works (1835); tank glass furnace (1870) | households; contract builders |
| paper | paper mill; machine paper mill (1815) | households; shops and offices |

On the port's goods: the edge-tool works (1778) and the tool factory (1864)
for tools, the hot-blast furnace (1830) for iron, the machine-tool works
(1838) and the engineering works (1870) for machinery, the continuous kiln
(1856) for bricks, and shops, offices and warehouses (1860) for services.

The needs: "drink and groceries" and "household goods" are added, warmth is
shown as "Fuel and light" and takes gas, and raw stone, brick and timber are
re-rated as shelter for the game only (stone 0.25 where the port has 0.5), so
that a household can still heap up its own wall where builders are short, at
a real loss.

Everything the design marks for later stages is out: sites (water, wind,
harbours), flour, fish and produce, cotton and foreign trade, improvements as
data, ships. `docs/designs/production-tree-draft.toml` has them written out;
each waits for an engine change and joins `data/game.toml` when that is
built.

## Its numbers

`TreeParams`, in `tree.rs`. Off by default; `game()` gives the second column.

| | default | game | |
|---|---|---|---|
| `stage` | 0 | 6 | how much of the overlay is laid: every entry with a stage no greater (1 was the game's until the content after 1900, and is the island as it was then) |
| `file` | "" | "" | an overlay file to read in place of the one built in, for trying numbers without building again |
| `seed_trades` | true | true | open 1750 with the new goods' trades in place |
| `plant_waits` | false | true | households and the works at work are served before orders for plant; acts only with a stage laid |
| `canal_stone` | 1 | 0.35 | the share of `canals.discover_stone` the canals' first survey waits for; acts only with a stage laid ("In company with the canals", below) |

`--set tree.stage=1 --set tree.plant_waits=true` measures the tree alone under
the port's rules. Under the game's rules, `--set tree.stage=0` is the game
without it, to the byte: the other numbers do nothing without a stage.

## What the sweeps showed

Eight seeds, the game's rules, six families, to 1900. "Without" is `--set
tree.stage=0`; both were run on the same build, after the mines were merged
and before the canals. (Sixteen seeds, and the island with the canals in,
are under "What the review found" and "In company with the canals".)

| | without the tree | with stage 1 | seeds up/down |
|---|---|---|---|
| real wage in 1800, 1850, 1900 (1750 = 1) | 1.62, 1.93, 2.61 | 1.60, 2.15, 3.33 | 5/3, 7/1, 7/1 |
| the same on the chained index | 1.89, 2.73, 4.18 | 1.80, 3.00, 6.36 | 2/6, 7/1, 8/0 |
| output per head in 1900 | 2.66 | 3.48 | 8/0, clear of the spread |
| people in 1900 | 720,000 | 678,000 | 0/8 |
| living in towns, 1750, 1800, 1850, 1900 | 0.22, 0.26, 0.34, 0.42 | 0.29, 0.34, 0.43, 0.47 | 8/0 each |
| hands in works (industry's share), 1750 and 1900 | 0.22, 0.49 | 0.32, 0.56 | 8/0, clear of the spread |
| hands in quarries, 1750, 1800, 1850, 1900 | 0.076, 0.081, 0.118, 0.136 | 0.022, 0.031, 0.044, 0.019 | 0/8, clear of the spread |
| works standing in 1900 | 747 | 1,069 | 8/0, clear |
| first railway; km in 1850, 1870, 1900 | 1828; 1,239, 1,853, 2,261 | 1826; 1,205, 1,741, 2,317 | inside the spread |
| price swings: grain, iron, rails, cloth | 0.107, 0.168, 0.176, 0.037 | 0.110, 0.155, 0.168, 0.043 | iron down 1/7, cloth up 7/1 |
| the real wage's worst year | -0.146 | -0.142 | 3/5 |
| the real wage's yearly swing | 0.053 | 0.052 | 4/4 |
| dearths; famines (news items a run) | 7.0; 2.6 | 8.5; 0.6 | 5/1; 0/6 |
| towns in 1900; last town founded | 18.8; 1844 | 17.0; 1862 | 2/5; 5/3 |
| woodland in 1900 (cells) | 1,064 | 1,082 | 4/4 |

So the island is about as steady as it was, a quarter richer by the fixed
basket in 1900 and half as rich again by the chained one, with five to nine
points more of its people in towns all century. There are six per cent fewer
people in 1900, in every seed: more of them live in towns, where the people
rule has them die younger. No run failed in any sweep.

Without families the same comparison gives 3.12 against 2.70 for the 1900
wage and 0.038 against 0.136 for the quarries; under the port's rules with
the tree alone, 1.47 against 1.29 and 0.044 against 0.159 (1.56 against 1.35
with six families).

**When each invention is found and built** (mean year over the eight seeds;
`first` is the first works, `half` the year it makes half of its good):

| invention | aim | found | first | half | in 1900 it makes |
|---|---|---|---|---|---|
| pottery manufactory | 1770 | 1774 | 1776 | 1782 | all pottery |
| common brewery | 1785 | 1780 | 1782 | 1792 | all beer |
| edge-tool works | 1778 | 1784 | 1803 | 1816 | none (the tool factory has it) |
| machine paper mill | 1815 | 1812 | 1813 | 1819 | all paper |
| alkali works | 1815 | 1815 | 1817 | 1823 | 39% of soap and chemicals |
| gasworks | 1815 | 1820 | 1821 | | all gas |
| steam sawmill | 1825 | 1822 | 1824 | 1832 | all sawn timber |
| hot-blast furnace | 1830 | 1832 | 1834 | 1850 | all iron |
| machine-tool works | 1838 | 1834 | 1836 | 1844 | 44% of machinery |
| sheet glass works | 1835 | 1836 | 1839 | 1846 | 37% of glass |
| cement works | 1850 | 1853 | 1857 | 1867 | all lime |
| converter (the port's) | 1856 | 1862 | 1864 | 1871 | all steel |
| continuous kiln | 1856 | 1863 | 1864 | 1874 | all bricks |
| contract builders | 1862 | 1860 | 1862 | 1875 | all building work |
| shops, offices and warehouses | 1860 | 1864 | 1875 | 1876 | 70% of services |
| tool factory | 1864 | 1863 | 1868 | 1875 | all tools |
| engineering works | 1870 | 1866 | 1877 (6 seeds of 8) | 1879 | 56% of machinery |
| ammonia-soda works | 1872 | 1871 | 1875 (7 of 8) | 1884 | 61% of soap and chemicals |
| tank glass furnace | 1870 | 1871 | 1872 (7 of 8) | 1876 | 63% of glass |
| steel rail mill (the port's) | 1866 | 1864 | 1875 (7 of 8) | never | nothing to speak of: it waits for railways that wear out |

Every one is found within seven years of its aim on average. The hazard
alone spreads a discovery over about twelve years either side of its mean,
so one seed in eight far from the aim is the dice. (On sixteen seeds the
islands differ by more than the dice: "Discovery dates scatter", in the
review.)

**Hands**, as shares of all hands at work in December:

| | 1750 | 1800 | 1850 | 1900 | without the tree, 1750 and 1900 |
|---|---|---|---|---|---|
| farms | 0.314 | 0.262 | 0.197 | 0.114 | 0.311, 0.106 |
| pasture | 0.115 | 0.102 | 0.082 | 0.116 | 0.122, 0.121 |
| woods | 0.108 | 0.043 | 0.037 | 0.011 | 0.123, 0.014 |
| quarries | 0.022 | 0.031 | 0.044 | 0.019 | 0.076, 0.136 |
| clay pits | 0.006 | 0.015 | 0.015 | 0.059 | 0.006, 0.021 |
| iron mines and collieries | 0.007 | 0.042 | 0.050 | 0.058 | 0.008, 0.053 |
| builders | 0.075 | 0.101 | 0.126 | 0.093 | |
| sawyers | 0.021 | 0.032 | 0.008 | 0.002 | |
| brewers | 0.010 | 0.012 | 0.011 | 0.018 | |
| potters, glass, paper, soap, gas, lime | 0.016 | 0.026 | 0.037 | 0.053 | |
| brickmakers | 0.010 | 0.016 | 0.019 | 0.025 | 0.015, 0.051 |
| iron, steel, tools, machinery, rails | 0.027 | 0.057 | 0.060 | 0.049 | 0.026, 0.055 |
| cloth | 0.046 | 0.008 | 0.019 | 0.020 | 0.049, 0.020 |
| shops and trades | 0.112 | 0.147 | 0.231 | 0.297 | 0.127, 0.362 |
| charcoal burners | 0.011 | 0.000 | 0.000 | 0.000 | 0.010, 0.000 |
| carters and the rest | 0.101 | 0.107 | 0.064 | 0.067 | 0.127, 0.060 |

Builders are a tenth of all hands, as the design reckoned (7.5, 12 and 10
per cent); quarries, pits and mines together are 13.6 per cent in 1900 where
they were 21. All the new trades together are 12 to 18 per cent of hands.

**Needs.** Builders make 98, 96, 89 and 89 per cent of shelter in 1750,
1800, 1850 and 1900. Gas is 9 per cent of fuel and light in 1850 and 23 in
1900. Of household goods, pottery has 18 to 30 per cent, soap and chemicals
20 to 28, glass 14 to 27 and paper 13 to 27 at the four dates. Beer takes
about a fifth of the grain the island grows in 1900. Food is 41 per cent of
household spending in 1750 and 13 in 1900.

**Prices.** No good's price stands at its floor or its ceiling (steel is at
its ceiling in 3 per cent of town-years, before anyone makes it, as in the
port). In 1900 iron is 1.3 a unit, machinery 5.6, bricks 1.0, building work
2.5 and timber 2.7, where without the tree they are 5.9, 62, 2.8, none and
3.4.

**Time.** A simulated year, one run at a time on a machine other work was
using, twice each on three seeds: 0.083 and 0.103 s without the tree, 0.123
and 0.111 s with it. So between a tenth and a half more, about a quarter on
the means, which is what was measured before the mines too (0.120 and
0.150). The goods not carried and the shorter delivered-price pass are
already in that. In the sweep, with eight runs side by side, the tree's
runs were the quicker (17 s against 22).

`cargo run --release -p isostate-core --example tree_market` prints all of
this for any seeds and rules: methods, thresholds, goods, needs, hands and
each good's fill.

## What was changed from the draft, and why

The design's own list of what would need tuning was right about most of it.

| number | draft | now | why |
|---|---|---|---|
| raw stone and brick as shelter | 0.3, 0.6 | 0.25, 0.5 | with the draft's the builders held nine tenths of shelter to 1850 and three quarters in 1900 |
| masons' wood | timber 0.35 | sawn timber 0.2 | masons are nine tenths of building until 1860 and bought no sawn timber, so the sawmills had a tenth of their market and the steam sawmill came forty years late |
| bricklayers' sawn timber | 0.2 | 0.05 | timber: see the next |
| contract builders' sawn timber | 0.18, and 2 in the plant | 0.02, and 0.5 | the island cuts some 30,000 units of timber a month and less each decade; at 0.18 timber ran to eight times its base price and building work never fell |
| ammonia-soda works' usual size | 2000 | 600 | a first works is a tenth of its town's trade, and at a fifth of its usual size it pays a penalty its recipe cannot carry: found and not built |
| tank furnace's usual size | 1500 | 600 | the same: its first works came ten years after it was found |
| engine works' usual size (the port's 3) | 8 | 4 | the same: never built in three seeds of eight at 8, in one at 4 |
| engineering works' aim | 1876 | 1870 | its first works comes ten years after it is found; aimed at 1876 it was never built in four seeds of eight |
| tank furnace's aim | 1874 | 1870 | never built in two seeds of eight, now one |
| every threshold | fitted to the reckoning | fitted to the sweep | below |

The thresholds, draft and now (units made since 1750; "xN" is N years of
1750's output):

| invention | draft | now |
|---|---|---|
| converter | steel 2,800,000; machinery 250,000; iron x220 | steel 3,300,000; machinery 410,000; iron x220 |
| steel rail mill | steel 5,000,000; rails 560,000; machinery 450,000 | steel 4,500,000; rails 580,000; machinery 880,000 |
| contract builders | building 41,000,000; machinery 410,000 | building 55,000,000; machinery 660,000 |
| steam sawmill | sawn timber 5,000,000; machinery 55,000 | sawn timber 7,400,000; machinery 69,000 |
| cement works | lime 2,800,000; machinery 200,000 | lime 3,800,000; machinery 270,000 |
| common brewery | beer 2,100,000; coal 13,000,000 | beer 2,700,000; coal 16,000,000 |
| pottery manufactory | pottery 39,000; coal x35 | the same |
| gasworks | iron 1,200,000; coal 47,000,000 | iron 1,600,000; coal 64,000,000 |
| edge-tool works | steel 400; tools x20 | the same |
| tool factory | steel 4,000,000; machinery 350,000 | steel 4,200,000; machinery 760,000 |
| hot-blast furnace | iron 1,720,000; coal 65,000,000; machinery 70,000 | iron 2,500,000; coal 95,000,000; machinery 94,000 |
| machine-tool works | machinery 110,000; steel 1,300,000 | machinery 150,000; steel 1,700,000 |
| engineering works | steel 7,500,000; machinery 1,000,000 | steel 5,800,000; machinery 1,300,000 |
| continuous kiln | bricks x300; machinery 300,000 | bricks x300; machinery 460,000 |
| shops, offices and warehouses | gas 1,000,000; services 32,000,000 | gas 1,000,000; services 20,000,000 |
| alkali works | soap and chemicals 340,000; iron 1,000,000 | 450,000; iron 1,400,000 |
| ammonia-soda works | soap and chemicals 4,200,000; steel 4,400,000 | 5,600,000; steel 5,300,000 |
| sheet glass works | glass 250,000; chemicals 630,000; iron 2,000,000 | glass 330,000; chemicals 890,000; iron 2,700,000 |
| tank glass furnace | glass 1,400,000; steel 4,400,000 | glass 1,500,000; steel 4,900,000 |
| machine paper mill | paper 230,000; machinery 30,000 | paper 260,000; machinery 42,000 |

Machinery is the number that moved most: with the machine-tool works the
island makes two to three times the machinery the reckoning stood on. The
shops' services went the other way, because with two more needs to spend on
the island buys a fifth to a quarter less from the old shops.

Left as the draft has them, and looked at: gas at 4.5 (9 per cent of fuel and
light in 1850 with gas found five years late, 23 in 1900: inside the
design's band); beer's shares (a fifth of the grain); comfort's richer share
(shops and trades are 30 per cent of hands in 1900 against 36 without the
tree, which the builders and the new trades more than make up); the
machine-tool works' `A0` (machinery's price does not halve within five years
of it).

## The fitter

`crates/isostate-core/parity/tree_fit.py SWEEP_DIR [--write]`.

The thresholds are plain numbers of units, so they are right for one island
and one set of rules. The proof came at once: the mines made ore a third
cheaper and coal a sixth, and with the thresholds as they stood every
invention on iron, steel or machinery came early, the alkali works by fifteen
years, the ammonia-soda works by sixteen, the tank furnace by twelve, the
machine-tool works by ten.

The fitter reads a sweep's `annual.csv`, takes each seed's cumulative output
of every good year by year, and works out for each invention the year it
would be found on average under the engine's own hazard. Then it solves for
the one year Y at which "the clock good at what the island had made by Y, the
guards at 0.85 of what it had made of them" brings that average to the aim.
Its `SPEC` says which good is each invention's clock. Moving one discovery
moves the output the others are counted in, so it moves each threshold seven
tenths of the way and is run again on a new sweep: three passes settled
these to within four years of every aim in the reckoning, and within seven
in the sweep itself.

Run it again whenever the island or the rules change. The canals turned out
to move these clocks very little ("In company with the canals"): the fitter
was read on the merged build and nothing was written. The second island
generator is in the code and switched off; when it is switched on every
number in that table will want fitting again.

## Tried and not kept

- **A wood that is its standing timber** (forestry's `alpha` 0.5 for 0.3,
  `A` 22 for 14, as a sector entry in the overlay). Timber's price and
  output swing by a third from year to year after 1850, with the tree and
  without it (the price by 0.27 without and 0.31 with, the cut by 0.29 and
  0.38: the hands that follow a price arrive half a year late, and once coal
  has taken the fires nothing that buys timber gives way when it is dear). A
  less elastic wood, tried before plant waited, halved the swing in what is
  cut (0.38 to 0.17), left the price's swing where it was (0.30), kept a
  fifth more woodland, and did nothing for the builders' fill. It re-bases
  the woods the harvest, people and land pages were tuned on, for a gain
  that plant waiting gives without it: with plant waiting building work's
  price swings by 0.045 a year after 1850 and its output by 0.053, where
  they swung by 0.12 and 0.24. The numbers are in
  `.claude/night/reports/sweep-tree2-exp-a`.
- **Plant waiting without the tree.** Above: no gain, and a wider yearly
  swing in the real wage in all eight seeds.

## What the review found

A second helper read the code against the design and the brief, tried to
break it, and ran everything again on a build of its own, with seeds 9 to 16
beside 1 to 8. The sweeps are under `.claude/night/reports/`, named
`sweep-treerev-*`.

**Off is off.** The golden test passes with its constants untouched. Seed 3
under the port's rules, with and without six families, writes the same
`series_monthly.csv`, `events.json` and `params.json` as the baseline kept
from before any mechanism (`.claude/night/baseline`). Under the game's rules
`tree.stage = 0` was set beside a build of the commit the tree was cut from:
seed 3 with six families writes the same two files to the byte. The tree
draws no random number, switched on or off, and no hook does anything at
stage 0: one test runs the game without a stage twice, with plant waiting and
the canals' clock as the game has them and put back, and compares every
series bit for bit.

**Money.** Every run of every sweep checks the island's money each month.
None failed: the game with six families at stage 0, at stage 1 and at stage 1
without plant waiting (sixteen seeds each), the game without families (eight
seeds, both stages), the port with the tree alone, with and without families
(eight seeds), and the odd settings below.

**The numbers hold on eight more islands.** Seeds 9 to 16 give what seeds 1
to 8 gave. On all sixteen, before the canals were merged: the real wage in
1900 3.26 against 2.65 (up in fifteen), output per head 3.47 against 2.70
(sixteen, clear of the spread), living in towns 0.465 against 0.423 in 1900
and 0.293 against 0.217 in 1750 (sixteen), quarry hands 0.023 against 0.138
in 1900, 5.7 per cent fewer people (fifteen), railways inside the spread
(1,248 km against 1,315 in 1850, 2,346 against 2,260 in 1900), famine news
0.5 a run against 1.6, dearths 9.1 against 7.7. Every invention is found
within six years of its aim on the sixteen-seed mean. On seeds 9 to 16 alone
no good's price stands at its floor or ceiling, builders are 7.6, 10.3, 12.0
and 7.4 per cent of hands at the four dates, and the last wave is a little
later than on the first eight (the engineering works found in 1879, built in
seven of eight by 1900).

**Plant waiting, measured where it acts.** Stage 1 with and without it, on
sixteen seeds: the real wage's worst year is -0.140 with it and -0.180
without (better in fourteen), its yearly swing 0.052 and 0.060 (fourteen).
It does not slow the railways: 1,248 km against 1,171 in 1850 and 1,773
against 1,788 in 1870. What it costs is plant: 1,040 works stand in 1900
against 1,150 (fewer in thirteen), and the engineering works is never built
in three seeds of sixteen where it was one. No fact stands clear of the
spread either way.

**Discovery dates scatter.** The means sit on their aims; the islands do
not. Over sixteen seeds, with the canals in, the gasworks is found between
1808 and 1863 (the median island in 1818), the steam sawmill between 1800
and 1862, cement between 1832 and 1874, the converter between 1842 and 1888,
contract builders between 1851 and 1884, the new shops between 1840 and
1893, the engineering works between 1856 and 1897. The early inventions are
tighter (the pottery manufactory 1767 to 1792, machine paper 1804 to 1827).
The hazard would give a dozen years either side; the rest is the island. A
threshold is a plain number of units, an island poor in coal or iron takes
decades longer to make it, and each late invention holds back the ones that
wait on its good. The design named this risk. It is honest (an island that
makes little iron learns little about iron) and it is why the last wave is
missing on some islands: the engineering works is built by 1900 in twelve
seeds of sixteen and the new shops in thirteen. A player's island may have
gas light in 1810 or in 1860. Whether that is variety or a fault is a
design question; a threshold counted per head, or against the island's own
1750, would narrow it.

**Why there are fewer people.** It is the people rule's deaths in great towns
and nothing else. With `people.town_deaths = 0` on eight seeds the island
with the tree has 976,000 people in 1900 and the island without it 955,000:
two per cent more, where with the towns' deaths it has six per cent fewer.

**Odd settings.** A stage above the file's highest (`tree.stage = 9`) is
stage 1 to the last digit (as the file then was; it is stage 6 now). `tree.seed_trades = false` runs: 1750 opens with
0.251 of its people in towns where the seeded trades give 0.292, and by 1900
the two islands are the same inside the spread. An overlay file that is
missing or wrong stops the run with a sentence that says what is wrong.

**What was changed.** The sum that shares a short market was taken out of
`econ::step_month` into `tree::served_first` (five lines are left there),
with a test of its edges; no fact of a sweep moved. Then the canals, below.
Nothing was found wrong with the loader, the content's new keys, the
families' trades, the observer's groups or the two short-cuts in the trade
and delivered-price passes (nothing reads a delivered price but a works
buying an input or a fuel; plant is reckoned at the town's price).

## In company with the canals

The review merged the integration branch as it stood (the canals, the land
generator, which is off, and the monthly clock). With `tree.stage = 0` the
merged build gives the integration branch's own sweep fact for fact (seeds 1
to 8, six families, 0 of 420 facts differ).

**The canals' stone clock.** The first canal is surveyed once three million
units of coal and three million of stone have been dug
(`canals.discover_coal`, `discover_stone`). On the island the canals were
tuned on, stone passes three million in 1756 and coal in 1762, so coal is
the clock and stone a guard behind it. With builders the same houses take a
third of the stone (0.34 of it dug by 1760, 0.35 by 1770): stone passes
three million in 1767, becomes the clock, and the first canal is begun in
1780 where it was 1771, with a third less canal in 1800 (467 km against 695,
less in all sixteen seeds). So with a stage laid the stone asked for is
`tree.canal_stone` of the canals' number, 0.35: the guard is met in 1756
again and the first canal is begun in 1773 (1771 without the tree; later in
eight seeds, earlier in six). The hook is one marked line in
`canals::knowledge`; at stage 0 the clock is the canals' own.

What is left is not the clock. With the stone clock eased the island still
has a fifth less canal in 1800 (554 km against 695, less in all sixteen) and
an eighth less in 1900 (789 against 903). Stone and clay were 16 per cent of
what the canals carried in 1800 and are 6 per cent with builders: there is
a third of the stone to move, so fewer cuts pay. A canal also waits behind
the day's trade for its stone and bricks, as a railway does: the slowest
takes 5.6 years to dig against 5.0 (longer in fourteen seeds).

**The island, sixteen seeds, six families, the canals in** (the final build;
"without" is `--set tree.stage=0`):

| | without the tree | with stage 1 | seeds up/down |
|---|---|---|---|
| real wage in 1800, 1850, 1900 (1750 = 1) | 1.59, 1.99, 2.61 | 1.64, 2.10, 3.20 | 9/7, 11/5, 16/0 |
| the same on the chained index | 1.83, 2.84, 4.29 | 1.84, 2.93, 5.80 | 7/9, 8/8, 15/1 |
| output per head in 1900 | 2.67 | 3.43 | 16/0, clear of the spread |
| people in 1900 | 726,000 | 682,000 | 1/15 |
| living in towns, 1750, 1800, 1850, 1900 | 0.22, 0.26, 0.35, 0.42 | 0.29, 0.36, 0.42, 0.46 | 16/0 each |
| hands in works, 1750 and 1900 | 0.22, 0.49 | 0.32, 0.54 | 16/0, clear |
| hands in quarries, 1750, 1800, 1850, 1900 | 0.073, 0.079, 0.113, 0.141 | 0.019, 0.028, 0.044, 0.027 | 0/16, clear |
| works standing in 1900 | 729 | 1,014 | 16/0, clear |
| first railway; km in 1850, 1870, 1900 | 1827; 1,348, 1,829, 2,254 | 1830; 1,187, 1,776, 2,389 | 10/6; 5/11, 6/10, 12/4 |
| first canal begun; km in 1800 and 1900 | 1771; 695, 903 | 1773; 554, 789 | 8/6; 0/16, 5/11 |
| price swings: grain, iron, rails, cloth | 0.113, 0.174, 0.179, 0.039 | 0.112, 0.150, 0.161, 0.043 | iron and rails down 2/14, cloth up 15/1 |
| the real wage's worst year; yearly swing | -0.153; 0.056 | -0.131; 0.051 | 11/5; 6/10 |
| dearths; famines (news items a run) | 8.4; 0.4 | 9.8; 0.1 | 9/5; 2/6 |
| towns in 1850 and 1900; last town founded | 16.6, 17.2; 1846 | 14.6, 16.6; 1870 | 2/12, 6/10; 14/2 |

It is the island of the table further up, with canals: a little over a fifth
richer by the fixed basket in 1900, a third by the chained one, four to ten points more of
its people in towns, six per cent fewer people. Builders are 7.5, 10.1, 12.2
and 8.2 per cent of hands at the four dates and quarrymen 2.2, 3.1, 4.7 and
2.6 (seeds 1 to 8); no good's price stands at its floor or ceiling; every
stage 1 works is built in all eight seeds but the new shops (six). The
tables are in `.claude/night/reports/treerev-m-market-stage1.txt` and
`-stage0.txt`. The railways of mid-century
are a tenth shorter on average and level by 1870, in eleven seeds of
sixteen: inside the spread, and worth watching.

**Time.** On the final build a simulated year, one run at a time on seeds 3,
5 and 7, twice each: 0.102 and 0.104 s without the tree, 0.108 and 0.112 s
with it. Seven per cent more: the twelve months 0.033 s for 0.030, the
yearly step 0.075 s for 0.071. (The quarter measured before was two
readings on a busy machine that disagreed with each other.)

**On the new land.** With `land.enabled = true` (the island made by uplift
and erosion, off in the game) four seeds run at both stages with no run
failed, and the tree does there what it does here: quarry hands 0.039
against 0.145 in 1900, 0.454 against 0.409 living in towns, seven per cent
fewer people. Its thresholds were not fitted to that island.

**The thresholds stand.** The fitter, read on this sweep without writing,
reckons every invention within three years of its aim but the two whose
thresholds it is told to keep: the shops, six years late, and the edge-tool
works, five. In the sweep itself the latest are the engineering works
(found in 1877, aimed at 1870), the gasworks (1822 for 1815), contract
builders (1868 for 1862) and the pottery manufactory (1776 for 1770). The
canals move the tree's clocks very little, and nothing was refitted. Never
built by 1900, in seeds of sixteen: the engineering works in four, the
steel rail mill in four, the new shops in three, the engine works in two.

## On the island stage 5 leaves (19 November 2026)

The tree was held on its branch until stages 2 to 5 of the pass over the
rules had landed (`docs/designs/rules-pass.md`, section 5), and then brought
up to main with them, the books and the new land beside it. Merged and not
yet fitted, the fitter reckoned every invention within four years of its aim,
as the pass expected (its stages 2 to 5 move no output in the mean); in the
sweep itself the furthest were the machine-tool works, found in 1831 for
1838, and the engineering works, 1864 for 1870. Two passes of the fitter
(`tree_fit.py --write` on a sweep of islands 1 to 8 with six families, a new
sweep, and again) moved the thresholds by a tenth or less: steel for the
converter 3.3 to 3.8 million, machinery for the machine-tool works 150,000
to 170,000 and steel 1.7 to 2.0 million, steel for the engineering works 5.8
to 5.4 million and machinery 1.3 to 1.1 million, services for the new shops
20 to 18 million, soap and chemicals for the ammonia-soda works 5.6 to 6.2
million; the rest by a few per cent (`data/game.toml` has them all). On the
second sweep the fitter reckons every invention it fits within two years of
its aim; the two it is told to keep are the shops, four years late, and the
edge-tool works, four. In the sweep itself every invention is found within
eight years of its aim: the converter in 1864 (aimed at 1856), the
engineering works in 1865 (1870), the paper machine in 1811 (1815), the
common brewery in 1780 (1785).

The island with the tree, against main as it stood on 13 November (islands 1
to 8, six families; `isostate-core sweep ... --compare` against main's
reference sweep):

| | main, without the tree | with the tree, fitted | islands up/down |
|---|---|---|---|
| runs failed | 0 | 0 | |
| people in 1900 | 735,871 | 691,385 | 0/8 |
| towns in 1850; in 1900 | 16.9; 17.6 | 14.6; 19.2 | 0/6; 6/1 |
| living in towns, 1900; 1881 to 1900 | 0.412; 0.410 | 0.457; 0.459 | 8/0 each |
| real wage in 1900 (1750 = 1); chained | 2.57; 4.09 | 3.23; 5.92 | 8/0 each |
| output per head in 1900 | 2.63 | 3.49 | 8/0 |
| works standing in 1900 | 735 | 1,128 | 8/0 |
| diggings' share of hands in 1900 | 0.200 | 0.127 | 0/8 |
| railway in 1900; canal in 1800 and 1900 | 2,312 km; 712, 895 km | 2,550 km; 523, 736 km | 8/0; 0/8, 2/6 |
| timber's yearly price swing | 0.164 | 0.186 | 7/1 |
| goods made on every island in 1900 | 15 of 16 | 24 of 25 | |
| the card: aims met, loss | 10 of 20, 27.5 | 13 of 20, 21.6 | |

The good not made is charcoal on both sides. Never built by 1900: the steel
rail mill in four islands of eight, the engineering works and the
ammonia-soda works in two each. On the card the tree meets aims 6, 11, 18
and 19 that main misses and misses aim 4, which main meets; the towns' share
(aim 3) is closer.

The tests that hold a case found on main's island (`tests/hands.rs`,
`money.rs`, `shelves.rs`, `card.rs`, `mines.rs`, `people.rs`) play it with the
tree off and say so, as the pass's own tests do with the rules that came
after them.

## After 1900: electricity

**The standing check passes: twelve of twelve inside.** This is the first
content after 1900, and the first time the overlay goes past its stage 1.
Electricity and electrical goods, with a coal power station and an electrical
works, are found in the 1880s, so the played century could have moved; it did
not. Islands 1 to 8 with six families, to 1900, `--set tree.stage=1,6
--standing` (on the final build, after the refit below):

| | stage 1 [least .. greatest] | stage 6 | |
|---|---|---|---|
| people in 1800, 1850, 1900 | 202,320, 364,624, 673,962 | 200,790, 361,951, 664,089 | inside |
| chained real wage, 1850 and 1900 (1750 = 1) | 3.00 [2.51 .. 3.46], 6.03 [5.10 .. 6.94] | 3.13, 5.87 | inside |
| towns in 1900; living in towns | 19.4 [13 .. 23]; 0.465 [0.457 .. 0.489] | 17.9; 0.467 | inside |
| railway in 1900; first line | 2,482 km; 1832 | 2,581 km; 1827 | inside |
| work's share of income, 1850 and 1900; land's in 1900 | 0.660, 0.670; 0.165 | 0.657, 0.662; 0.162 | inside |

There is no hydro station: it wants the tree's water sites (its stage 2),
which are not built. And there is no grid: electricity is not carried between
towns, so every town that wants it builds its own station.

**The switch is the overlay's stage.** The content after 1900 is numbered on
from the tree's own five stages, so its entries in `data/game.toml` carry
`stage = 6`; the tree's stages 2 to 5 have no entries yet, so stage 6 lays
stage 1 and these and nothing else. `tree::GAME_STAGE` is 6 and the game's
rules lay it. `--set tree.stage=1` is the island as it was, to the byte: a
sweep of islands 1 to 4 against the build this stage was cut from gives "of
the 888 facts both have, the seconds aside, 0 differ", and `tests/power.rs`
holds island 7's fingerprint at stage 1 to 1900 to the one that build gives.

### What is in the file

| | |
|---|---|
| `electricity`, Electricity | weight 20, base price 1.2, not carried (`traded = false`), no opening stock, does not keep (`stock_months = 0`). Fuel and light at 9 for the design's first value and 11 as refitted (light is worth more than heat, as gas's 4.5 is); a fuel a works may burn, at 5 a unit of heat where coal gives 2.4 |
| `electricals`, Electrical goods | weight 0.5, base price 7, decay 0.003 a month, no opening stock; household goods at 5 a unit |
| `electrical_works`, Electrical works | steel 0.25, chemicals 0.1, glass 0.05; heat 1.0, labour 0.45; plant 1.2 years of its output; usual size 40; the engineering trade |
| `coal_power_station`, Coal power station | coal 1.1, labour 0.1, no heat of its own; plant 5 years of its output; usual size 1,500; the coal trade, as the gasworks are |

Plant follows the design's rule for the content after 1900: so many years of
a works' own output at base prices, of kind M (three tenths building work, a
tenth steel, six tenths machinery, by value), written out as the `capital`
line: the station's 72 a unit of monthly capacity at base prices is building
work 9.8, steel 0.24 and machinery 0.62. The electrical works cannot leave the
tree's fifth at base prices, because its steel alone (0.25 at 30, the price of
1750's crucible steel) is worth more than the unit; its heat and labour leave
a fifth at the island's prices of 1885 instead. The file's comments give both
sums. Each starts as good as its recipe and learns nothing (`A0` 1, no curve).
The station's usual size is the electricity a middling town of 1900 (some
25,000 people, three units of fuel and light a head a month) would buy at a
fifth of its fuel and light; the works' is the engine works' worth of output.
Each method has its own row of classes of task in `data/tasks.toml` (the
station .30 .50 .05 .15 0, the works .15 .55 .05 .25 0), and names of its own
("Glenbrook Electric Light Co.", "Jowett Electrical Co.").

The works are found by thresholds on what the island has made, fitted with
`parity/tree_fit.py` in two passes as the rest of the tree's were, with a new
`--only` so that stage 1's thresholds are left as written: the electrical
works on steel (the clock) and machinery, aimed at 1885; the station on
electrical goods and coal, aimed at 1888.

### Goods that do not keep

A good with `stock_months = 0` (electricity alone) is made to be used in the
month. What is left in store at a month's end, once the month's sales are
taken out, is lost, and the month's make is what the next month sells
(`tree::month_loss`; the books count the loss as spoiled, and the gauges'
column for what is made and lost with them). Its price cannot move on the
merchants' stock, which never holds two months: by that rule it would settle
only where a quarter of what is made is lost. So it moves on the month's gap
between what was asked and what was made, `(asked - made) / max(asked, made)`,
with the same step, markup and bounds as every other good's (`tree::price_gap`).
`stock_months` read as any other number is the months of sales merchants mean
to hold, so a later good may keep a month; absent, it is the island's
`price_target_months`, 2, and no other good's price moves.

Both new goods open at their ceiling price in every town (`opens_dear`, a key
on a good). Nobody has either for more than a century, and at their base
prices electricity would be the cheapest fuel on the island by a factor of
three and electrical goods the cheapest household good: every works would put
most of its heat on a fuel it could not get, and a works makes what its
scarcest input allows. The price rule would carry them to the ceiling anyway,
but only after some forty months of hunger for them. Gas is not given the key,
which would move stage 1.

### The eight islands to 2050

Islands 1 to 8, six families, `isostate-core sweep --seeds 1-8 --rules game
--families 6 --end-year 2050`, the build this stage was cut from against this
one (which is stage 1 against stage 6, since stage 1 is that build to the
byte). The design's pass is electricity a fifth of fuel and light by 1930.
With the design's first values (fuel and light 9, heat 5) it was 0.090 in 1930
(0.034 to 0.191 by island), under a tenth, with a unit of warmth from it about
half as dear again as one from coal. Fuel and light was refitted once, from 9
to 11, a fifth more, which at the logit's width of 0.25 brings it near the
aim. Heat was left at 5: electricity gave at most two per cent of the works'
heat before 1930, far from the half by 1910 that would call for a refit.

| | 1900 | 1910 | 1920 | 1930 | 1940 | 1950 | 1970 | 2000 | 2050 |
|---|---|---|---|---|---|---|---|---|---|
| share of fuel and light (units of warmth) | 0.023 | 0.070 | 0.128 | **0.172** | 0.220 | 0.257 | 0.296 | 0.352 | 0.556 |
| least and greatest island | 0 .. 0.113 | 0 .. 0.159 | 0 .. 0.190 | 0.004 .. 0.252 | 0.142 .. 0.295 | 0.201 .. 0.315 | 0.242 .. 0.351 | 0.283 .. 0.444 | 0.427 .. 0.649 |
| share of the works' heat | 0.000 | 0.001 | 0.005 | 0.012 | 0.024 | 0.037 | 0.043 | 0.032 | 0.063 |
| towns with a station (of the towns there are) | 6.1 (17.9) | 14.8 (18.4) | 15.4 (18.6) | 18.1 (18.6) | 17.6 (18.8) | 18.5 (18.8) | 18.1 (18.9) | 18.8 (19.0) | 18.9 (19.1) |
| electricity made and lost at a month's end | 0.33 (4 islands) | 0.23 | 0.11 | 0.11 | 0.09 | 0.06 | 0.07 | 0.08 | 0.06 |
| its price over coal's for a unit of heat | 21.7 | 8.1 | 7.6 | 2.0 | 1.8 | 1.7 | 1.6 | 1.7 | 1.4 |

The electrical works is found in 1888 on the mean island (1872 to 1914) and
the station in 1900 (1876 to 1927): the station waits on the works'
electrical goods, and an island poor in steel finds the works late (the
tree's "Discovery dates scatter", above). The fitter reads sweeps to 1900 and
counts an island that has not found an invention by then as finding it in
1905, so it thought the station's mean was 1889. The price over coal's is
high until about 1925 because it is weighted over the towns that bought any,
some of them with a first station and a price still coming down from the
ceiling.

Of electrical goods, households (with landlords and families) bought all:
nothing yet has them in its plant (the electric drive is the next part).
Household goods, in units of the need, in 1930: soap and chemicals 0.284,
glass 0.258, pottery 0.179, paper 0.175, electrical goods 0.103; in 1950
0.284, 0.261, 0.174, 0.153 and 0.128. Fuel and light in 1930: coal 0.697, gas
0.131, electricity 0.172; in 1950: 0.598, 0.145, 0.257.

Against stage 1:

| | 1900 | 1930 | 1950 | 2000 | 2050 |
|---|---|---|---|---|---|
| chained real wage (1750 = 1), stage 1 | 6.03 | 7.10 | 7.43 | 5.77 | 3.45 |
| stage 6 | 5.87 | 7.56 | 8.00 | 6.67 | 3.97 |
| work's share of income, gross, stage 1 / 6 | 0.670 / 0.662 | 0.662 / 0.665 | 0.654 / 0.668 | 0.586 / 0.597 | 0.578 / 0.578 |
| the same net of wear, stage 1 / 6 | 0.688 / 0.679 | 0.684 / 0.686 | 0.677 / 0.691 | 0.638 / 0.649 | 0.602 / 0.605 |
| people, stage 1 | 673,962 | 889,022 | 1,054,043 | 1,659,405 | 2,781,755 |
| stage 6 | 664,089 | 867,373 | 1,017,810 | 1,536,340 | 2,489,623 |

So the island with electricity lives better after 1930 (the chained wage a
sixth higher in 2000 and 2050) and has fewer people (a tenth fewer in 2050).
Clear of the spread between islands, on every island: more of the people in
towns from 1940 (0.468 against 0.448 in 1940, 0.321 against 0.271 in 2050),
more of the hands in works and fewer in the diggings after 2000, and a smaller
share of the budget on fuel and light from 1950 (0.075 against 0.081). Work's
share of income does not move beyond chance.

**What a run costs.** The same sweep, eight runs at a time, one after the
other on the same machine: a run to 2050 took 82 seconds on the mean island
with 25 goods (72 to 88) and 87 with 27 (82 to 91), six per cent more, against
the design's reckoning of 55 to 105 seconds at thirty goods.

**In the viewer**, a recorded run of island 7 to 2050 shows electricity in the
Markets panel at its ceiling until the first station and coming down to
about three in the 1930s. The stations and electrical works are drawn as any
other works until they have drawings of their own.

`tests/power.rs` holds the overlay at stage 6, the island at stage 1, a good
that keeps nothing from one month to the next with money conserved every
month, and a town with no station that has none to buy.

### What worries me here

- **Works burn their fuel mix whatever each fuel's fill.** The engine shares
  a works' heat over its fuels by price and charges it for each, and where a
  fuel is short the works makes less but still takes its full share of it,
  the excess counted in the books as oversold. With electricity, which has no
  stock to fall back on, that is a few hundred-thousandths of a unit in a town
  whose station has just closed, and more where a station is short.
- **A district that changes town takes its electricity with it**, as its
  merchants' stock of every good: January may sell December's make in a town
  that has no station.
- **The fitter stops at 1900.** It counts a late island as finding the
  invention in 1905, so it reckons the station seven years earlier than the
  sweep to 2050 finds it. Fitting the content after 1900 wants the fitter to
  read to the sweep's last year.
- **Electricity's share keeps rising to 2050**, past half of fuel and light,
  with coal's falling, because nothing else after 1900 competes; oil and gas,
  and the grid, are later parts.

## Oil, nitrogen and the land

The second part of stage 6 is the land's twentieth century: oil, two methods
and an invention in `data/game.toml`, all with `stage = 6`; the tractor and
the cutter that run on oil and electricity, in `data/tasks.toml`
([tasks.md](tasks.md), "The tractor and the cutter"); and nitrogen and
breeding in the land machines' table ([machines.md](machines.md), "Nitrogen
and breeding"). The switch is the overlay's stage, as for electricity:
`--set tree.stage=1` is the island before this part to the byte (islands 1 to
4 against the build it was cut from: "of the 888 facts both have, the
seconds aside, 0 differ").

| | |
|---|---|
| `oil`, Oil | weight 1, base price 1, decay 0.002 a month, no opening stock, keeps; fuel and light at 3.5 a unit, and a fuel a works may burn at 3.5 a unit of heat. Opens at its ceiling price (`opens_dear`), as electricity does: at its base price it would be the island's cheapest warmth in 1750, and nobody has any until it is made |
| `oil_from_coal`, Oil from coal | coal 4 and electricity 0.5 a unit, labour 0.05, no heat of its own; plant 3 years of its output, kind M; usual size 400; the coal trade; aimed at 1925 |
| `nitrogen_works`, Nitrogen works | makes soap and chemicals from coal 1.5 and electricity 1.2, labour 0.1; plant 2.5 years of its output, kind M; usual size 150; the coal trade, as the alkali works; aimed at 1915 |
| `breeding`, Breeding and sprays | an invention, not a works; aimed at 1950 |

Each method has its row of classes of task in `data/tasks.toml` (oil from
coal .25 .50 .05 .20 0, the nitrogen works .10 .50 .05 .35 0) and names
of its own ("Melbourne Coal Oil Co.", "Glenbrook Nitrogen Co."). **There is no
oilfield**: the design's oilfield wants the second land generator, which the
game's rules do not use. So the island's oil is made from coal, and it is
dear: on the eight islands a unit of heat from oil costs six to eight times
one from coal (`oil_over_coal_heat_Y`: 7.5 in 1930, 8.6 in 1950, 6.0 in
1970), where the design reckoned three. Four of coal for 3.5 of heat is 2.7
times coal's heat on its own; the half unit of electricity, at about three a
unit where it is sold, costs as much again as the coal. So oil warms nobody
and fires no works: it is made for the tractors.

**The thresholds** were fitted with `parity/tree_fit.py`, which now reads the
sweep's own last year (it stopped at 1900, which counted every island that
had not found an invention by then as finding it in 1905): oil from coal on
coal made, with electricity kept as a guard; the nitrogen works and breeding
on chemicals made, the nitrogen works with electricity kept as a guard.
Electricity is a poor clock: island 4 makes none until the 1920s. On a sweep
to 2050 (`isostate-core sweep --seeds 1-8 --rules game --families 6
--end-year 2050`, then `python crates/isostate-core/parity/tree_fit.py DIR
--only oil_from_coal,nitrogen_works,breeding`):

| | aimed at | found, mean of 8 | least .. greatest |
|---|---|---|---|
| nitrogen works | 1915 | 1916 | 1901 .. 1943 |
| oil from coal | 1925 | 1926 | 1899 .. 1949 |
| breeding and sprays | 1950 | 1945 | 1928 .. 1974 |

Island 8 finds oil from coal in 1899, by the hazard's long tail; the standing
check holds all the same (twelve of twelve inside, `--set tree.stage=1,6
--standing`). Nothing else of this part is found before 1901.

**A tractor waits on oil, and oil waits on the tractor.** Nobody buys oil at
its ceiling price, so an oil works is built where it is found, sells its
first months' make into a market with no buyer, sees the price fall below
its costs and closes (island 4, on a first build whose tractor waited for
2,000 units: two works, 935 units made by 1923, and then none for a decade;
on six islands of eight the tractor was not found by 1960). The tractor is found on oil made, not on a date: at the
first year's turn from 1915 at which the island has made 300 units of oil,
about what its first works make in their first months. Once it is found the
farms ask for oil and the works stay: oil made is 2,700 a year on the mean
island in 1920, 8,000 in 1930, 22,000 in 1950 and 49,000 in 1970. On island 1,
the last to light a town (its first station in 1933), oil from coal is found
in 1949 and the tractor in 1951. An island with no oil of its own mechanises
its fields late.

**What a run costs.** A run to 2050 took 110 seconds on the mean island (102
to 120) with 28 goods, against 93 (89 to 97) for the base's 27, eight at a
time on a machine shared with other builds.

## What worries me

- **Timber still rings.** The cycle above is the base game's and is not
  cured; plant waiting only keeps it from stopping the builders. Timber's
  market after 1850 is thin (23,000 units a month for 680,000 people) and
  will stay so until foreign trade lands timber at the ports. Contract
  builders are written with a tenth of the sawn timber the design gave them
  for that reason; when timber lands, raise it (the comment in the file says
  so), and the logging trade Stella asked for will be the size the design
  meant. Today sawyers are 2 to 3 per cent of hands until 1830 and a fifth
  of one per cent in 1900.
- **The last wave has no years to spare.** The engineering works is not
  built in two seeds of eight (those that found it after 1880), the
  ammonia-soda works and the tank furnace in one each. Without families,
  which found works larger, it is worse: the engineering works in four seeds
  of eight, the new shops in two, contract builders in one. The game is
  played with six families, and that is what was tuned.
- **Fewer people.** Six per cent fewer in 1900, in fifteen seeds of sixteen.
  It is the people rule doing what it says with a more urban island (traced
  in the review: without the towns' deaths the tree's island has the more
  people), but nobody chose it.
- **A fifth less canal.** With builders there is a third of the stone to
  move, and the canals lose their second cargo: 554 km in 1800 against 695,
  789 in 1900 against 903, after the stone clock is put right. The canals'
  own numbers were tuned on the island that heaped stone; nobody has looked
  at whether the canal age is still what its page wants.
- **Machinery is near its floor in 1900.** It sells at 0.07 of its base
  price (about 5 a unit) where the floor is 0.05, and steel at 0.11: not
  pinned in any town-year to 1900, but a run on to 2050 will pin both unless
  the floor moves or the learning slows.
- **Builders lose a fifth of shelter by 1900.** On the final build they make
  98, 95, 87 and 79 per cent of shelter at the four dates (78 on seeds 9 to
  16), not the 89 of the first eight-seed table: once the continuous kiln
  makes bricks at half their base price, raw brick is 11 per cent of shelter
  and raw stone 7, even rated at a half and a quarter. Builders are still
  8 per cent of hands in 1900 and the quarries under 3.
- **Under the port's rules the tree alone has more famine news** (4.1 items
  a run against 2.2 without families, 4.4 against 2.5 with). Under the
  game's rules, with the harvest and people rules, famines fall (0.6
  against 2.6) and dearths rise a little (8.5 against 7.0). I did not trace
  the port's.
- **Gas is asked for before anyone can make it.** From 1750 households ask
  for a little gas at its opening price (0.6 per cent of fuel and light) and
  get none until the first gasworks. It is small, and it is the port's own
  way with a good nobody makes yet, but a careful reader of the fill table
  will see gas at 3 per cent filled before 1800.
- **The engine works is still late.** Found 1782, first built 1815, and not
  at all in one seed of eight; the millwrights carry machinery until then.
  Without the tree it is first built in 1814, so this is not the tree's
  doing, but the design leaned on it.
- **The steel rail mill** is found on time and hardly built, with the tree
  as without. It waits for railways that wear out (stage 5).
- **Examples that hold a content of their own.** `works_census`,
  `mines_census`, `towns_census` and `towns_edges` now ask for the laid
  content. `machines` and `prices_probe` read the run's own content and are
  right as they are; their tables list the port's goods and methods by name
  and so say nothing of the new ones.
- **Copies of the year's end.** `examples/tree_market.rs` and one test in
  `tests/tree.rs` step the months themselves to read each month's market,
  and so carry a copy of what `sim::run_year` does when the year turns, as a
  dozen other examples and tests do. A mechanism that adds a line there
  must add it to them, or the example's run is not quite the game's.
- **The chronicle's test of new towns** needs an island with a decade of
  three foundings in its first fifty years, and has had to change seed each
  time the island's economy moved (7, 8, 11, now 3).
- **The game has not been seen with 25 goods.** The viewer builds; nobody
  has looked at its panels with nine more goods in their lists, and the 28
  new works are drawn as "any other works".
- **Sixteen seeds, one island generator.** Everything above is means over
  seeds 1 to 8 or 1 to 16 of the port's island.

## For whoever draws the works

The methods exist now; each is drawn as "any other works" until it has a
drawing. From the design (section 8.4), what each looks like from above:

| method | works | from above |
|---|---|---|
| `masons` | Masons and carpenters | a yard of dressed stone and baulks, a lime pit, a hoist; scaffolding on whatever the town is building |
| `carpenters` | Timber-frame builders | a framing yard: oak frames laid out on the ground, a saw pit, bundles of laths |
| `bricklayers` | Bricklayers | brick stacks under thatch hurdles, a mortar mill turned by a horse |
| `contract_builders` | Contract builders | a builder's yard with a steam saw and a joiners' shop, and beyond it whole streets in scaffolding at once |
| `saw_pit` | Saw pit | a trestle over a pit with a log on it, planks stacked to season |
| `steam_sawmill` | Steam sawmill | a long open-sided shed, a boiler chimney, log stacks on one side and sawn deals in ranks on the other, a sawdust heap |
| `lime_kiln` | Lime kiln | a squat stone drum built into a bank with an arched draw hole, white spoil below, smoke from the top |
| `cement_works` | Cement works | a row of bottle kilns and a long grinding shed, grey dust over everything |
| `brewhouse` | Brewhouse | a malt kiln with a cowl beside an inn yard, casks under a lean-to |
| `steam_brewery` | Common brewery | a tall brewing tower with a louvred roof, a vat house, maltings with pyramid kiln roofs, drays in the yard |
| `potbank` | Country pottery | one bottle oven beside a drying shed, a flooded clay hole |
| `pottery_works` | Pottery manufactory | a courtyard works with four to eight bottle ovens of unequal height, shard heaps |
| `gasworks` | Gasworks | a retort house with a row of stub chimneys and round gasholders, one more holder for each tier of capacity |
| `edge_tool_works` | Edge-tool works | a low range of grinding hulls along a dam, small wheels, a tilt-hammer shed |
| `tool_factory` | Tool factory | saw-toothed sheds and one hammer shop with a row of stubby stacks |
| `hot_blast` | Hot-blast furnace | taller iron-banded stacks with domed stoves beside them, an incline to the furnace top, a cast house, slag banks |
| `machine_shop` | Machine-tool works | long saw-tooth north-light roofs, a yard crane, castings stacked outside |
| `engineering_works` | Engineering works | parallel erecting bays, a foundry cupola, engines on wagons in a siding |
| `continuous_kiln` | Continuous kiln | an oval ring of arches round one very tall chimney, a flooded clay pit, a tramway, brick stacks |
| `emporium` | Shops, offices and warehouses | a four-storey block with a glazed ground floor, or a street of shops roofed in iron and glass; part of the town centre |
| `soap_boilers` | Soap boilers and ash burners | iron pots under an open shed, ash pits and cordwood stacks; in town a boiling house with a wide low chimney |
| `alkali_works` | Alkali works | long lead-chamber sheds on legs, the tallest chimney on the map, grey waste heaps and dead hedges |
| `ammonia_soda` | Ammonia-soda works | a cluster of iron towers and pipe bridges, a row of lime kilns, white settling lagoons |
| `glasshouse` | Glass cone | one brick cone eighty feet high, three times the height of its sheds |
| `sheet_glass` | Sheet glass works | long sheds with louvred roofs and a row of short chimneys, crates of sheet in the yard |
| `tank_furnace` | Tank glass furnace | two great gabled halls and a pair of tall regenerator chimneys |
| `paper_mill` | Paper mill | a stone mill with a wheel and slatted drying lofts |
| `machine_paper` | Machine paper mill | a long single-storey machine house, a boiler chimney, a rag store |

Honest decoration follows from numbers the simulation keeps: a chimney on a
works that burns heat, a gasholder for each tier of a gasworks, scaffolding
in proportion to the builders' output, houses in the material of the town's
builders (`share_building_masons`, `_carpenters`, `_bricklayers` and
`_contract_builders` are in the observer's series for the island; a town's
own mix is its works).
