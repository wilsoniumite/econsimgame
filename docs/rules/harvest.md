# Harvest: good years and bad

Under the port every year on the land is an average year. The grain price
moves all the same, but only because the price rule has a swing of its own
(`docs/probes/dx-cycles.md`), and once that is cured (`prices.md`) grain goes
flat: its price changes by under three per cent a year and a downturn comes
twice in a century and a half. With `harvest.enabled` the land gives a little
more or less each year than anyone planned for, the way it does. None of it is
in the Python.

Everything is off by default. With `harvest.enabled = false` nothing is drawn,
kept, told or written, and a run is byte for byte the port's
(`tests/golden.rs`). It is on under the game's rules; `--set
harvest.enabled=true` measures it alone. The code is `harvest.rs`, with two
one-line hooks in `econ::step_month`, one in `sim::yearly`, one in
`sim::run_year` and one in `families::land_price`, each marked `harvest:`.

## What it does

- **The draw.** At the end of each year the coming year's weather is drawn:
  one number for the whole island and a smooth field for its districts, alike
  over about `reach` cells (30 cells is 60 km, a quarter of the island's
  width), so that one valley can fail while the next does well. Together they
  make a multiplier on each cell's yield, log-normal with mean one:
  `exp(island_sd x z + district_sd x field - half the variance)`. The mean is
  one so that an average year stays what the land bids expect. The draws come
  from a stream of their own (`Rngs::harvest`, stream 7) and do not depend on
  the sizes, so a sweep over `island_sd` compares the same weather at
  different strengths. The warm-up years have no weather: 1750 starts from the
  same settled island with the mechanism on or off.
- **Who feels it.** Farms feel all of it. Pastures feel two fifths
  (`pasture`): grass fails in a drought too, but flocks carry over and are
  driven to better ground. Woods feel none of it (`woods`), since a tree does
  not notice one summer, and nor do quarries, pits or mines.
- **When.** The multiplier is applied in the monthly step to what each farm
  and pasture brings in, after its hands are hired and its machines asked for.
  The crop is not what the wage bill was laid out for, and the landlord, who
  takes what is left after wages and cartage, carries the difference: in a bad
  year the rent of poor land can go below nothing, and he pays the hands out
  of his own purse. No money is made or lost; the same sales, wages and rents
  move between the same purses, in other amounts.
- **Half of it is seen.** About half a farm's year is ploughing and sowing,
  done before anyone knows what the summer will bring, and half is reaping,
  threshing and carting, done when the crop can be seen standing. So half the
  weather (`foreseen`) is known when the hands are hired: a thin crop wants
  fewer of them, and the wage feels a bad year as well as the rent. This is
  the one place I left the brief, which asked for a pure surprise; the reason
  is measured below.
- **Nobody plans for it.** Land bids, works and investors go on expecting an
  average year: they read the year's prices, as they always did, and the
  weather reaches them only through those. With families, a cell's price is
  twenty-five years of the rent it *would* have paid in an average year
  (`harvest::average_rent`), so a failed crop does not put a district's land
  on sale at half price.
- **The rest is the markets that were already there.** Less grain comes to
  market, the merchants' stocks run down, the price rises, the real wage
  falls, and where the shelves empty `people::growth` finds a town short of
  food, as it did before.

## The news

Told at the year's end, in the observer's manner (what was seen, with its
numbers), kind `harvest` unless said:

| item | when | weight |
|---|---|---|
| A poor harvest | the country's crop is `news_z` (1.8) island deviations short: 9 per cent at the game's size | 0.4 to 0.75, by the shortfall; 0.6 from ten per cent |
| A bumper harvest | the same above | 0.3 to 0.5 |
| Three lean years | the third year running at least half a deviation short; it takes the place of that year's poor harvest | 0.45 to 0.75, by what the three years lost together |
| The harvest fails around TOWN | one town's farms are `news_local_z` (2.3) district deviations below the country's and below an ordinary year, in a year when the country's harvest is not itself news | 0.35 to 0.55 |
| A dearth of grain (kind `dearth`) | the year's grain price stands `dearth_rise` (15 per cent) above its trend: the mean of the five years before, carried forward at the drift of the last twenty. At most once in three years, and only with the merchants' prices on (`prices.enabled`) | 0.55 to 0.8 |

The viewer's "major" feed shows weights from 0.6, so it gets the dearths, the
worst harvests and the worst runs of lean years; bumper years and one
district's failure stay in the full feed. With prices on that is 17 harvest
items and 6 dearths in 150 years, three every two decades; under the port's
own price rule, whose false swing keeps tripping the dearth line, 16 and 15.
That is why a dearth is now told only beside the merchants' prices: the
dearth reads the price of grain and nothing else, and on the port's rule
that price trips the line with no weather behind it (five times in sixty
years on seed 3 with both sizes at 0; `tests/harvest.rs`,
`no_weather_and_the_ports_prices_tell_no_dearth`). Every count of dearths
under the port's price rule on this page was taken before that.
(Those counts, and every count of dearths in the tables below, were taken
with the line at 12 per cent, where it stood until harvests were set beside
the game's other rules; "In company" says why it moved and what the count is
now.)

Since the chronicle joined the game's rules ([chronicle.md](chronicle.md))
the major feed is no longer "weights from 0.6" but the few items a decade a
history would keep. The harvest's news is ranked there with everything else,
and a harvest and a dearth count as one kind, so a decade keeps at most one
of them, the heavier: 1.4 harvests and 2.2 dearths in a run of 150 years
(seeds 1 to 12, six families), of 16 and 5.4 told. Each item also says why
it is news (`why`: the crop or the price, and the line it passed) while the
chronicle is on.

## What the viewer is given

Only when the mechanism is on:

- `LiveYear::harvest_town`, each town slot's harvest in the year just
  finished against an average year (1.0 is average; the mean over the town's
  farmland, weighted by what the land would bear), and `LiveYear::harvest`,
  the country's.
- In the run file, two more arrays: `town_harvest`, `[years, T]`, and
  `harvest`, `[years]`. A port run's file has neither.
- In the state, for anyone drawing the year as it happens:
  `harvest_town_now` and `harvest_now` are the running year's, and
  `harvest_mult`, `(NS, N)`, is every cell's multiplier for each land use.

## Its numbers

| `harvest.` | game | |
|---|---|---|
| `enabled` | true | off under the port |
| `island_sd` | 0.05 | standard deviation of the island-wide part of a year's yield (in logs) |
| `district_sd` | 0.05 | of the district part, across the island's land in one year |
| `reach` | 30 | cells over which a district's weather is alike |
| `pasture` | 0.4 | share of the shock pastures feel |
| `woods` | 0 | share woods feel |
| `foreseen` | 0.5 | share of the weather known when the hands are hired; 0 is a pure surprise; held between 0 and 1 |
| `news_z` | 1.8 | island deviations that make a poor or a bumper harvest news |
| `news_local_z` | 2.3 | district deviations that make one town's failure news |
| `dearth_rise` | 0.15 | how far above its trend the grain price is a dearth (0.12 until the merge; "In company") |

With both parts at 0.05 the country's harvest has a standard deviation of
0.051 (the district part nearly averages out over the island), and a town's is
about 0.07. The cycles probe asked for about five per cent, and I kept it.

## What the sweeps showed

Seeds 1 to 8 to 1900, three sizes of the island's shock, each set against a
baseline of its own made with the same build. The swing of a thing is the
standard deviation of the yearly change in its log; `ac1` is the correlation
of one year's change with the next, which is -0.5 for independent shocks to a
level, near zero for a level that wanders and well below -0.5 for one that
rings. Every number is the mean over the eight seeds, with the least and the
greatest where that matters.

**With prices on** (the two merged on a scratch branch; this is the island the
game will be played on), no families:

| | prices alone | 0.03 | 0.05 | 0.08 |
|---|---|---|---|---|
| grain price swing | 0.027 | 0.053 | 0.078 | 0.117 |
| grain output swing | 0.023 | 0.059 | 0.090 | 0.141 |
| price `ac1` | +0.20 | -0.36 | -0.52 | -0.55 |
| output `ac1` | -0.01 | -0.56 | -0.65 | -0.67 |
| farm zigzag | 0.80 | 1.37 | 1.95 | 2.92 |
| years with a town in famine | 2.5 [0..4] | 2.1 [0..5] | 3.0 [1..6] | 2.8 [2..4] |
| dearths told | - | 1.9 | 5.9 [3..9] | 13.0 |
| harvest items told | - | 17.2 | 17.2 [9..20] | 15.8 |
| downturns the observer calls (`alarm`) | 2.4 | 3.2 | 6.5 | 15.4 |
| worst year's fall in the real wage | -0.131 [-0.311..-0.070] | -0.107 | -0.104 [-0.171..-0.079] | -0.161 [-0.251..-0.079] |
| people in 1900 | 682,066 | 682,884 | 682,321 [673,970..689,077] | 681,966 |
| grain output on the harvest | - | 0.81 | 0.79 [0.62..0.92] | 0.78 |
| grain price on the harvest | - | -0.88 | -0.86 [-0.97..-0.75] | -0.84 |
| real wage on the harvest | - | 0.23 | 0.21 [0.16..0.29] | 0.19 |

With six families the same table reads the same: at 0.05 the price swing is
0.079 (0.026 with prices alone), the output swing 0.090 (0.023), the two
`ac1` -0.52 and -0.64, famine years 1.6 [0..3] (1.8), dearths 6.2 [4..9],
harvest items 16.4, the worst year's fall -0.102 [-0.168..-0.063] (-0.078),
people in 1900 684,484 (684,341), the real wage on the harvest 0.19
[0.14..0.29].

Read: grain moves again, three times as much as with prices alone, and it
moves as independent years should. A harvest ten per cent short brings in
eight per cent less grain that year (the dearer price calls a few more hands
to the land), sells it nine per cent dearer, and takes two per cent off what a
wage buys, in every seed. The price does not ring: the year after, it is back
within a point or two of where it was. Output has a one-year echo (`ac1`
-0.65, not -0.5): the hands the dear year called in are still there for the
next crop, so a short year is followed by a rather full one. Downturns the
observer notices go from one in sixty years to one in twenty-three. The long
run does not move: people, towns, works, railways and the real wage of 1900
are all inside the spread between seeds.

What does **not** happen: famines are no more common (3.0 years a run against
2.5 is noise), the worst single year for wage earners is no worse, and nobody
dies of it, since the population grows at its cap whatever the harvest
(the rules' `README.md`, "What the port's baseline says"). With merchants who mark up
thin shelves, a harvest ten or fifteen per cent short is rationed by price
before a town runs out. A bad year is dear bread and a lean rent, not a
catastrophe. Whether that is too mild is for whoever tunes the people's side;
at 0.08 dearths come once in twelve years and the observer calls a downturn
every ten.

**Under the port's own price rule**, no families:

| | port | 0.03 | 0.05 | 0.08 |
|---|---|---|---|---|
| grain price swing | 0.086 | 0.091 | 0.104 | 0.126 |
| grain output swing | 0.136 | 0.161 | 0.207 | 0.268 |
| price `ac1` | -0.19 | -0.20 | -0.24 | -0.21 |
| output `ac1` | -0.45 | -0.48 | -0.52 | -0.51 |
| farm zigzag | 4.82 | 5.58 | 6.75 | 8.21 |
| years with a town in famine | 2.0 [0..7] | 3.5 [0..8] | 5.0 [1..8] | 7.0 [1..11] |
| dearths told | - | 12.9 | 15.4 [12..21] | 22.5 |
| harvest items told | - | 17.9 | 16.1 [9..20] | 16.0 |
| worst year's fall in the real wage | -0.180 [-0.229..-0.125] | -0.167 | -0.207 [-0.280..-0.143] | -0.230 [-0.612..-0.117] |
| people in 1900 | 682,305 | 678,157 | 679,719 [671,133..686,813] | 674,099 |
| grain output on the harvest | - | 0.84 | 1.04 [0.55..1.38] | 1.05 |
| grain price on the harvest | - | -0.50 | -0.49 [-0.68..-0.36] | -0.51 |
| real wage on the harvest | - | 0.12 | 0.18 [0.09..0.26] | 0.17 |

With six families at 0.05: price swing 0.107 (0.090 without harvests), output
swing 0.200 (0.132), famine years 3.8 [1..9] (2.1), dearths 17.9, harvest
items 16.1, people in 1900 682,027 (682,983), the real wage on the harvest
0.19 [0.08..0.27].

Read: here the probe's warning holds. The weather itself would add about a
seventh to the swing of grain output; it adds a half, because each shock sets
the old spring going again (the zigzag goes from 4.8 to 6.7). Famine years
double, though one seed in eight still has only one. The dearth line is
tripped fifteen times a run, twelve of them by the false swing, which would
trip it with no weather at all. Harvests belong with `prices`; alone on the
port's price rule they mostly feed its fault.

**Checked again, and out of sample.** A review reran the sweeps above on a
build of its own (`sweep-harvest-review*`, beside the others) and got the
numbers in these tables again, digit for digit, on seeds 1 to 8, with and
without families, under both price rules. It then ran seeds 9 to 16, which
nothing here was tuned on, and a larger shock, 0.12. With prices on, no
families:

| seeds 9 to 16 | prices alone | 0.03 | 0.05 | 0.08 | 0.12 |
|---|---|---|---|---|---|
| grain price swing | 0.027 | 0.055 | 0.083 | 0.124 | 0.175 |
| grain output swing | 0.022 | 0.060 | 0.093 | 0.145 | 0.226 |
| price `ac1` | +0.19 | -0.43 | -0.52 | -0.63 | -0.63 |
| output `ac1` | +0.01 | -0.60 | -0.66 | -0.72 | -0.74 |
| farm zigzag | 0.79 | 1.35 | 1.83 | 2.85 | 4.70 |
| years with a town in famine | 2.8 [0..6] | 1.5 [0..5] | 2.4 [0..6] | 2.9 [1..6] | 4.6 [3..7] |
| dearths told | - | 2.9 | 7.6 [4..10] | 15.1 | 22.4 |
| harvest items told | - | 17.9 | 17.2 [12..20] | 16.5 | 15.6 |
| downturns the observer calls (`alarm`) | 2.4 | 2.6 | 5.8 | 16.4 | 22.9 |
| worst year's fall in the real wage | -0.140 [-0.309..-0.056] | -0.097 | -0.154 [-0.612..-0.056] | -0.098 | -0.137 |
| people in 1900 | 683,771 | 684,739 | 684,687 [671,453..688,693] | 683,239 | 681,486 |
| grain output on the harvest | - | 0.89 | 0.88 [0.66..1.09] | 0.89 | 0.92 |
| grain price on the harvest | - | -0.95 | -0.93 [-1.10..-0.79] | -0.93 | -0.91 |
| real wage on the harvest | - | 0.23 | 0.21 [0.11..0.31] | 0.22 | 0.23 |

With six families on the same seeds at 0.05: price swing 0.083 (0.029 with
prices alone), output swing 0.093 (0.023), the two `ac1` -0.60 and -0.69,
famine years 1.25 [0..3] (1.5), dearths 6.9 [5..9], harvest items 17.1, people
in 1900 685,074 (686,018), the real wage on the harvest 0.22 [0.17..0.27].
Under the port's own price rule, seeds 9 to 16 at 0.05: price swing 0.109
(0.083 without harvests), output swing 0.208 (0.138), zigzag 6.79 (5.01),
famine years 4.1 [2..7] (1.5 [0..4]), dearths 18.4, the real wage on the
harvest 0.17 [0.11..0.29].

Everything above holds as it stood. The three slopes on the harvest have the
same sign in all sixteen seeds at the game's size, with families and without,
under either price rule. The echo in output is a little stronger on the new
seeds (`ac1` -0.66, and -0.69 with families). The shock of 0.12 answers the
question the first tables left: with prices on, famines do not become more
common until the island's harvest varies by twelve per cent a year, and there
they about double (5.0 and 4.6 years a run on the two sets of seeds, against
2.5 and 2.8; with six families 3.9 and 3.1 against 1.75 and 1.5) while the
people of 1900 are a third of a per cent fewer and dearths are told every
seven years. None of the 224 runs of the review's four sweeps with harvests
on (sixteen seeds, three or four sizes, with and without families, under both
price rules) failed the monthly money check.

**Pushed to its edges** (seeds 3 and 7 to 1900, `sweep-harvest-review-x-*`).
Both sizes at 0 with the switch on is the port's run byte for byte (seed 3
with six families: the same monthly series and the same news, with fifteen
dearths added, since the dearth line reads only the price; those dearths are
no longer told, see "The news"). Shocks of 0.2 and
0.4 for the island, or 0.3 for the districts, break nothing: famine comes in
21 to 71 of the 150 years, the people of 1900 number between 640 and 360
thousand, and money holds. `reach`
of 1 and of 200 cells, woods that feel all of the weather and pastures that
feel one and a half times as much or none of it, with prices and six
families: nothing breaks, and the facts stay near the game's. `foreseen`
below 0 or above 1 is now read as 0 or 1. Before that, `foreseen` 2 with
shocks of 0.3 was the one setting that failed: the farms hired for crops no
year brings, the landlords paid wages no crop covered, and the island's stock
of money ran down to nothing by 1830 in both seeds. Run again with the share
held, it is the same run as `foreseen` 1 and passes.

## Pure surprise, or half seen

The brief asked for a pure surprise: the hands hired for an average year, the
crop coming in as it may. I built that first and measured it, and one thing it
was meant to do did not happen. At the game's size, eight seeds:

| `foreseen` | 0 | 0.5 | 1 |
|---|---|---|---|
| with prices: real wage on the harvest | 0.00 [-0.07..0.10] | 0.21 [0.16..0.29] | 0.37 [0.26..0.48] |
| with prices: grain price on the harvest | -0.65 | -0.86 | -1.03 |
| with prices: grain output on the harvest | 0.50 | 0.79 | 1.01 |
| with prices: grain price swing | 0.063 | 0.078 | 0.091 |
| with prices: famine years | 1.9 | 3.0 | 2.0 |
| port's price rule: real wage on the harvest | 0.00 [-0.08..0.10] | 0.18 [0.09..0.26] | 0.36 [0.20..0.44] |
| port's price rule: famine years | 3.6 | 5.0 | 7.0 |

As a pure surprise the real wage does not move with the harvest at all, that
year or the next. The reason is in how the island hires. A farm takes hands
until the last one is worth his wage at the price of grain, and most of the
island works the land, so the wage follows the grain price. In a bad year the
price rises, farms that do not know their crop is thin bid for hands as if it
were full, and the wage rises with the bread. All of the loss falls on the
rent. That is not how a bad year went: a thin crop wanted fewer reapers and
threshers, and the labourer met dear bread with less work.

With `foreseen` the farm's worth of a hand is scaled by what it can see of
its own crop. At 1 the whole year is known in January, which is too much and
is not a surprise to anyone but the land market. Half has a reason one can
say in a sentence (sowing is done blind, reaping is not) and gives a real
wage that falls two per cent for a harvest ten per cent short, in every seed,
while the rent still takes the larger part. `--set harvest.foreseen=0` is the
brief's version, and the sweeps of it are kept beside the others
(`sweep-harvest-surprise*`, `sweep-harvest-x-prices-surprise*`).

## The facts added to the sweep

Under "Prices and cycles", in every run:

- `output_swing_grain`: as `price_swing_grain`, for the grain brought in.
- `price_ac1_grain`, `output_ac1_grain`: the correlation of one year's change
  in the log with the next year's, 1760 to the end.
- `real_wage_worst_year`: the real wage's largest fall on the year before.
- `real_wage_swing`: the standard deviation of its yearly change.
- `famine_years`: years in which `people::growth` found at least one town
  short of food and said so.

Only in runs with harvests on:

- `harvest_sd`: the standard deviation of the log of the country's harvest.
- `grain_output_on_harvest`, `grain_price_on_harvest`,
  `real_wage_on_harvest`: the slope of the year's change in the log of each on
  the log of that year's harvest. 0.2 for the real wage says a harvest ten per
  cent short takes two per cent off it that year.
- `grain_price_year_after_harvest`, `real_wage_year_after_harvest`: the same
  slope taken over two years, from the year before the harvest to the year
  after it: what is left of it by then.

## How it was checked

- `tests/harvest.rs`: off under the port (nothing drawn, kept, told or
  written, and `params.json` does not mention it) and on in the game; the
  multiplier reaches farms and pastures only, unseen it leaves the hands and
  the wage bit for bit as they were and scales each unit's crop, and seen it
  lowers the wage where the harvest is worst; no money is made or lost in a
  month of weather; a sweep over sizes compares the same draws; a run of 22
  years with it on conserves money every month and repeats byte for byte,
  packed run file included; over 60 years the harvest goes with the year's
  change in grain output and against the change in its price, the record has
  every town's harvest, and the news is at most three items a decade with
  weights between 0.3 and 0.8; with six families money holds, the live view
  carries the harvests, and land is priced on an average year's rent. Added
  by the review: `foreseen` above 1 or below 0 behaves as 1 or 0; and over
  sixty years of hard weather (shocks of 0.12) the real wage falls in the
  short years and rises in the full ones, dearths are told in dear years
  after short crops and never within three years of each other, every poor or
  bumper harvest told was one, and the worst harvest of the sixty is told.
- `tests/golden.rs` passes with its constants untouched. The review also ran
  seed 3 under the port, with and without six families, and found the monthly
  series, the news and `params.json` the same files, byte for byte, as the
  ones kept from before any mechanism was written.
- The sweeps above and the two variants kept beside them: 224 runs with
  harvests on, none failed the monthly money check. The review's four sweeps
  on sixteen seeds are another 224, and none of those failed either.
- The scripted players (`examples/bots.rs`, seeds 1, 2, 3 and 7) under the
  game's rules on this branch, which are the port's plus harvests: all four
  strategies play to 1900 with money conserved. On seeds 3 and 7 they end
  where they did under the port (the landlord, who lives on rent, still comes
  first of six with 27.8 and 24.4 million against 23.9 and 24.5); the landlord
  is first on seeds 1 and 2 as well.
- A run with harvests on takes about a twentieth longer (13.6 and 14.3
  seconds for seed 5, side by side on a busy machine).

## In company

Harvests were merged into the game's rules after `prices`, `engel` and
`machines`, so the island they now run on is not the one the tables above
were made on: its households lean off bread as they grow richer, its farms
buy machines, and by 1900 a wage buys two and a half times what it did. This
is what a sweep of the game's rules showed with harvests off and on, six
families, seeds 1 to 12 (`core-steps/harvest-on-off-s1-12` under
`.claude/night/reports/`; the integration's own before and after on seeds 1
to 6 are `core-steps/harvest-before` and `harvest-after`). The first column
is the review's sweep of harvests with prices alone, six families, sixteen
seeds.

| | alone, with prices | in company, off | in company, on |
|---|---|---|---|
| grain price swing | 0.081 | 0.027 | 0.100 [0.086..0.115] |
| grain output swing | 0.092 | 0.016 | 0.096 [0.081..0.106] |
| price `ac1` | -0.56 | +0.28 | -0.63 [-0.75..-0.49] |
| output `ac1` | -0.66 | -0.19 | -0.73 [-0.80..-0.67] |
| farm zigzag | 1.85 | 0.60 | 1.95 [1.42..2.44] |
| grain output on the harvest | 0.84 | - | 0.74 [0.56..0.98] |
| grain price on the harvest | -0.92 | - | -0.98 [-1.22..-0.78] |
| real wage on the harvest | 0.21 | - | 0.25 [0.17..0.32] |
| grain price, the year after | 0.14 | - | 0.33 [0.13..0.47] |
| real wage, the year after | -0.09 | - | -0.15 [-0.27..-0.07] |
| worst year's fall in the real wage | -0.103 | -0.058 | -0.076 [-0.112..-0.055] |
| years with a town in famine | 1.4 [0..3] | 1.8 [0..4] | 3.7 [2..7] |
| downturns the observer calls (`alarm`) | 6.9 | 0.8 | 8.4 [6..13] |
| harvest items told | 16.8 | - | 16.1 [7..21] |
| dearths told, line at 0.12 | 6.6 [4..9] | - | 10.4 [8..13] |
| dearths told, line at 0.15 (the game's now) | | - | 5.5 [2..8] |
| people in 1900 | 684,779 | 684,938 | 683,218 [674,647..688,684] |

**It does in company what it does alone.** The three slopes on the harvest
have their signs in all twelve seeds, and their sizes are the page's: a
harvest ten per cent short brings in seven per cent less grain, sells it ten
per cent dearer and takes two and a half per cent off what a wage buys. The
long run does not move. Towns, works, railway, the shares of work and the
real wage of 1900 are inside the spread between seeds (the wage of 1900 is
2.44 times 1750's against 2.46). On seeds 1 to 6 the facts of 1850 look like
a setback (grain output of 1850 lower in all six, output per head in five),
but 1850 is half way up the steep climb the machines give this island, and a
climb that comes three years late reads as a large gap in any one year. On
twelve seeds the real wage of 1840 to 1859 is higher with harvests in five
and lower in seven, 1.7 per cent lower on average, and by 1880 to 1899 it is
six and six. No run failed, money holds every month, and a run takes two to
five per cent longer (17.2 and 17.7 seconds on two tries against 16.9, six
at a time).

Four things are different, and one number was changed for them.

- **Grain's price answers harder, and that is the Engel curve.** The swing
  is 0.10 a year where it was 0.08. Switching the others off one at a time
  with harvests on (`core-steps/harvest-who`, seeds 1 to 6) puts it on
  `engel`: 0.099 with the curve and 0.077 to 0.079 without, whatever the
  machines do; the price's slope on the harvest is -0.98 to -1.02 against
  -0.88. Households past the reference income put little of a further
  shilling into food, so what they buy of bread hardly gives when it is dear,
  and the price has further to go before the market clears. This is the
  worry below that "the price answers weakly", a little answered by another
  mechanism. It is still nothing like the eighteenth century's three for one.
- **The echo is longer than one year.** The year after a harvest ten per
  cent short, grain is three per cent cheaper than it was before it (one and
  a half alone). The correlation of one year's change in the price with the
  change one, two, three and four years on is -0.63, +0.36, -0.21, +0.19;
  alone with prices it was -0.56, +0.27, -0.14, +0.12, and independent years
  would give -0.5, 0, 0, 0. So a short year is followed by a full one, then
  a rather short one, each about half the size of the last: a saw's edge that
  dies away, not one echo as the worries below have it. It is the farm hands
  answering a price half a year late, as there; the same two-year beat is
  faintly in the price with harvests off (the even lags stand above the odd
  ones: +0.28, +0.62, +0.15, +0.43). The weather only sets it going, and a
  price that answers harder sets it going harder.
- **Famines double.** A town is short of food in 3.7 years of a run against
  1.8, more in eleven seeds of twelve and never fewer, and there are a
  quarter of a per cent fewer people in 1900 (fewer in ten seeds of twelve).
  Alone with prices that took a shock of 0.12. It needs both of the others:
  2.3 years with the curve and no machines, 1.5 with the machines and no
  curve or with neither. I have not traced it further than that. My guess is
  the same thing as the harder price: where households will not give up
  bread, the merchants' markup rations less, and the shelves empty instead.
  In seed 6 the hunger comes between 1796 and 1814, all but once in towns
  that have stood for twenty years or more, and in three of them "households
  get 0% of the food they need" in the worst month. One such item is there
  with harvests off too (Ulstrand, 1803), so some town's shelves are emptying
  for a reason that is not the weather, and the weather adds to it. This is no longer "a bad year is mild", and it is worth a look by
  whoever does the people's side.
- **The dearth line stood too low for this island, and was raised from 0.12
  to 0.15.** At 0.12 a dearth was told 10.4 times a run where the page meant
  six, and of seed 6's thirteen, eight came after a harvest two to four per
  cent short or not short at all: it was the saw's edge tripping the line,
  the fault this page found in the port's price rule, in small. At 0.12 the
  line stood at one and a half times what grain's price moved in a year
  (0.08); it moves 0.10 now, and one and a half times that is 0.15. That gives 5.5
  dearths a run [2..8] (`core-steps/harvest-dearth-line`, twelve seeds; 2.4
  at 0.18), and seed 6's six come after harvests 8, 11, 7, 9, 4 and 4 per
  cent short. Nothing in the economy reads the line; only the news does.
  With the Engel curve off it now gives two or three a run, so the counts of
  dearths in the tables above are no longer what `--set harvest.enabled=true`
  alone tells.
- **The observer tells some harvests a second time, and gets the reason
  wrong.** Its "steep" items (a good's output far off its trend) go from 17 a
  run to 28, and its alarms from one a run to eight. Most are fair: a bad
  harvest is a downturn. But a bumper year can come out as "Grain glut ...
  new capacity is flooding the market" beside "A bumper harvest", and a
  short one as "Grain shortage ... supply has failed" beside a dearth. The
  observer does not know about weather. That is for the chronicle to sort
  out, not for a threshold here.

The tests pass as they stood, the golden constants untouched, and the
scripted players (`examples/bots.rs`, seeds 3 and 7) play all four strategies
to 1900 under the four mechanisms together.

## What worries me

- **In company, famines double and the price saws.** Both are in the section
  above. Neither breaks anything, and neither was so with prices alone, so
  the reassurances further down this list ("a bad year is mild", "one year of
  echo") describe harvests alone and not the game. If the famines turn out to
  be too many once people answer to how they live, the first thing to try is
  the size of the shock; the second is whatever is emptying a town's shelves
  to nothing with no weather at all.
- **The country's harvest may vary more late in the century.** In two seeds
  of three looked at (6 and 9; not 3) the standard deviation of the country's
  harvest is about 0.044 before 1800 and 0.056 to 0.06 after 1850, and seed 6
  has harvests 20 and 18 per cent above an ordinary year in the 1890s. The
  country's figure is the mean over farmland, and my guess is that as the
  farms draw in to the best land the district part stops averaging out.
  Three seeds, one look; I did not measure it properly.
- **Harvests alone are not ready to play.** They were built for an island
  whose prices have been steadied. Under the port's price rule they work and
  conserve money, but they feed the false swing. If `prices` were ever
  switched off, switch this off with it. Part of this is now a rule: the
  dearth, the one item of the harvest's news that reads the price and not
  the weather, is told only while `prices.enabled`. The harvest itself still
  runs on the port's price rule. I chose that over refusing the harvest
  without the prices because the refusal could only be made where a file is
  read, and the sweep's `--set prices.enabled=false` goes round that reader,
  so it would have held on one road and not the other; because a harvest
  that quietly did nothing would leave a `params.json` saying harvests were
  on; and because the tables above under the port's price rule, which are
  the case for harvests belonging with `prices`, can still be run again.
- **A bad year is mild.** No more famines, no worse a worst year, no one
  fewer in 1900. The first two are the merchants doing their job and the size
  of the shock; the third is the port's population rule, which does not look
  at living standards. When the people's side learns to (another helper's
  mechanism), harvests will start to matter for numbers, and the size of the
  shock should be looked at again with it on.
- **A short year is followed by a full one.** Output's `ac1` is -0.65 where
  -0.5 would be clean. It is one year of echo, not a ring, and it comes from
  farm hands answering a price half a year late, which is the port's
  `labor_adjust_ex` and not this mechanism's. It grows a little with the size
  of the shock.
- **The same weather all twelve months.** The island has no seasons: a bad
  year is twelve short months, known (by the game's half) from the first of
  them. A year with a harvest month would make stocks, and a price that peaks
  before the next crop, mean something; that is a larger change than this one.
- **The price answers weakly.** Ten per cent less grain is nine per cent
  dearer. In the eighteenth century it was more like thirty. Households here
  spend a share of their purse on bread whatever it costs, so the price need
  not rise far to clear; a mechanism that makes food the last thing given up
  would sharpen every bad year without touching this one.
- **`foreseen` and the price of land.** `average_rent` puts back the crop the
  weather took from the hands that were hired; with `foreseen` above 0 the
  hands hired were themselves fewer, which it does not undo. It is exact at 0
  and near enough at a half for pricing land. The families' own choice of
  which land to buy still reads the year's rent as paid, so after a failed
  harvest the poorest cells of a district are passed over for a year.
- **Landlords can go below nothing.** In a bad year the rent of marginal land
  is negative and the gentry of a poor district pay the wage bill from their
  purse, which can itself go negative (it could before: the port has it
  wherever rent falls short of wages). With families the same falls on the
  family that owns the land, which is the point, but I did not look for a
  family ruined by one harvest.
- **A new town's runaway wage is not mine, but harvests move it about.** In
  the port a small new town whose land wants far more hands than it has
  people can see its wage double every month to the ceiling of 500 for most
  of a year (seed 3 in 1753: the island's average wage doubles). It sits on a
  knife's edge, so any change to the island moves which seed has it: with
  harvests on the port's price rule it turned up as well in seed 7 (1754),
  seed 6 (1820) and seed 2 (1827), at some sizes of shock and not at others.
  With prices on it is rarer and still there, with harvests or without. On
  sixteen seeds the island's wage rises by 40 per cent or more in a year in 3
  of the 32 runs with prices alone (seed 11 in 1834 by 66 per cent and, with
  six families, in 1837 to nearly three times; seed 2 in 1841) and in 4 of
  the 128 with harvests as well (seed 11 again, to 2.3 times in 1850 at 0.05
  and in 1837 at 0.08 with families; seeds 14 and 5 by 41 and 42 per cent).
  So harvests do not make it commoner, but they do not cure it and they move
  it. A year like that also trips the dearth line and sets the worst-year
  fact: it is the -0.612 in the table of seeds 9 to 16. Under the port's price
  rule it is in 2 of 32 runs without harvests and in 13 of 96 with them.
- **Very large shocks slow the port's issue of money.** With the island's
  shock at 0.4, or at 0.3 with all of it foreseen, the money stock of 1900 is
  24 to 34 million where runs at the game's sizes have 35 to 39. The port
  issues money each month in proportion to what the purses hold at one moment
  of the monthly step, with that month's wages and dividends still on their
  way, and in these runs that sum is smaller; I did not trace why. At the
  game's sizes the money stock does not move (about 37 million in 1900 with
  and without harvests, seeds 1 to 8). It is the port's rule and far from the
  game's numbers; it is written down because it is what finally broke at
  `foreseen` 2.
- **The landlord bot reads the rent as paid.** `examples/bots.rs` ranks land
  by last year's rent over its price, and the price is now on an average
  year's rent, so the bot is drawn to wherever the harvest was best. It gains
  nothing by it, since next year's rent is an ordinary one. The families' own
  buying (above) has the same habit.
- **Sixteen seeds.** The slopes on the harvest are clear of zero in every one;
  the famine counts and the worst year are not clear of the spread at the
  game's size, and I have said so where it matters.
