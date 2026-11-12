# Tree

A deeper production tree, first stage. Not in the Python. The switch is
`tree.stage`: 0 is the content as it was, 1 lays the first stage of the
game's overlay, `data/game.toml`, over it. The code is `tree.rs`, with hooks
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
| `stage` | 0 | 1 | how much of the overlay is laid: every entry with a stage no greater |
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
stage 1 to the last digit. `tree.seed_trades = false` runs: 1750 opens with
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

**The thresholds stand.** The fitter, read on this sweep without writing,
reckons every invention within three years of its aim but the two whose
thresholds it is told to keep: the shops, six years late, and the edge-tool
works, five. In the sweep itself the latest are the engineering works
(found in 1877, aimed at 1870), the gasworks (1822 for 1815), contract
builders (1868 for 1862) and the pottery manufactory (1776 for 1770). The
canals move the tree's clocks very little, and nothing was refitted. Never
built by 1900, in seeds of sixteen: the engineering works in four, the
steel rail mill in four, the new shops in three, the engine works in two.

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
