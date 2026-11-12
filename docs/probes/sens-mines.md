# sens-mines: how much the mines' numbers and the chronicle's thresholds matter

One of six tables of sensitivity for the pass over all the rules (piece 18 of
`docs/backlog/found-along-the-way.md`: most of the mechanisms' numbers were set on a few islands and never
searched). This one is the mines (`docs/rules/mines.md`), under the landlords and under the commons, and
the chronicle's own thresholds (`docs/rules/chronicle.md`). Branch `night/o-sens-mines`, cut from `main` at
a6ff0ec (12 November 2026). No rule and no fact was changed: this page is the only thing on the branch.

Every run is the game's rules with six families, from 1740 to 1900. Each of the mines' ten numbers was
run at about three quarters and four thirds of its value on islands 1 to 8, under both regimes, and the
two that matter most again on islands 9 to 16. Each of the chronicle's twenty-two thresholds was run the
same way on islands 1 to 8. The sweeps and the scripts that read them are under
`.claude/night/reports/opus/sens-mines/` (the scripts in `scripts/`); none of it is tracked.

## In brief

- **Chance is as large as the islands by 1900.** A change too small to matter (the mines' `reserve` from
  20,000,000 to 20,000,001) sends every island down another history, and by 1900 the histories of one island
  differ as much as the islands differ from each other: baskets a head by 3 per cent, the real wage against
  1750 by 4, the railway by 5 (39 under the commons), the price of coal by 17. In 1750 they differ by a
  tenth of a per cent. So nothing in the late century can be read from one baseline on eight islands, and
  the usual "7/1" of a sweep's `--compare` is a weak test there. This changed how the rest was read (next
  section), and it is the first thing I would carry into the pass over the rules.
- **`reach` is the mines' strongest number.** From 1.5 cells to 2.7 there are 41 per cent fewer workings at
  every date (0/16 under both regimes), each taking 45 per cent more of its cell; stone raised a head in
  1750 is a tenth higher and 5 per cent cheaper (16/0, 0/16). Under the landlords the island of 1900 has 9.5
  per cent less railway (1/15) and, on most islands though not all, 5 per cent fewer baskets a head (3/13)
  and 7 per cent less coal a head (5/11).
- **`sink` and `payback` are one number.** A site opens when its royalty would repay `sink / payback` wages
  a month. Moving either moves the count of workings by a ninth and the stone of 1750 by 5 or 6 per cent,
  in mirror image; moving both so that the ratio stays put moves nothing the noise does not.
- **`cell_limit` does nothing** at these sizes: on 27 of the 32 runs the record is byte for byte today's.
  `hands_per_cell` moves only the land under the diggings, as 1/x. `memory` and `year_room` are at the
  noise floor. `reserve` does what its page says, a little more coal and fewer colliers late. `idle_years`
  shapes the workings the warm-up gives up and nothing after.
- **Nothing breaks.** Every one of 528 runs finished and passed the monthly money check. The worst year for
  the real wage, -0.40, is `reach` 1.5 on island 6 under the landlords: a town whose wage runs to sixteen
  times the middle town's in April 1765, the bad year that `mines.md` already owns up to.
- The chronicle: (filled in below).

## How the runs were read, and why not against one baseline

The plan was the usual one: a sweep at today's values (`base-l`, `base-c`), and each setting read against
it island for island with `--compare`'s up/down count. The first two settings looked alike in a way that
made me doubt it. Under the commons every setting, whatever its direction, lowered the late real wage on
six or seven islands of eight. So I ran a null change, `reserve` 20,000,001: the late real wage under the
commons fell by 4.0 per cent against the baseline on seven islands of eight, baskets a head in 1900 by 4.0
per cent (2/6), real income in 1881-1900 by 2.6 (2/6). Nothing had changed. The commons baseline happens to
sit high on most of its islands among the histories it might have had, and every comparison with it reads
that luck as an effect.

So each setting is read as its high value against its low, island for island: both are perturbed, and the
baseline's luck drops out. And the noise is measured, not guessed. Each island has four histories that
differ only by chance under each regime: today's values, `reserve` 20,000,001, and `sink` 40.0001 and
40.0002 (`m-null`, `m-null2`). The spread between those four, pooled over islands, is the chance in one
history; twice its variance over the number of islands is the chance in a mean difference of two. For each
fact, z is the mean of high minus low over that. A fact *beats the noise* when |z| is 3 or more and every
island but one moves the same way. Null against null (`sink` 40.0001 against 40.0002, with the noise taken
from the other two histories) gives three such facts of about 500 under each regime, so three is the floor.

What chance does, on sixteen islands, four histories each (`noise.py`; "chance" is the spread between
histories of one island, "islands" the spread between islands):

| | today, landlords | chance | islands | today, commons | chance | islands |
|---|---|---|---|---|---|---|
| workings in 1750 | 262 | 1.0% | 11% | 254 | 1.0% | 11% |
| stone raised a head, 1750 | 0.280 | 0.4% | 11% | 0.291 | 0.5% | 11% |
| goods a head, 1750 | 2.58 | 0.1% | 3.2% | 2.81 | 0.2% | 3.5% |
| workings in 1900 | 492 | 1.9% | 7.0% | 474 | 2.8% | 7.3% |
| people in 1900 | 725,000 | 1.1% | 2.8% | 827,000 | 1.1% | 2.0% |
| baskets a head, 1900 | 8.84 | 3.2% | 2.7% | 9.10 | 4.2% | 5.0% |
| real wage 1900 against 1750 | 4.33 | 3.6% | 4.2% | 3.82 | 4.5% | 4.3% |
| real income, mean of 1881-1900 | 8.93 | 2.2% | 5.4% | 9.22 | 3.2% | 5.1% |
| railway in 1900, km | 2,256 | 4.8% | 4.3% | 1,142 | 39% | 29% |
| coal raised a head, 1900 | 1.36 | 5.7% | 7.5% | 1.20 | 5.3% | 9.2% |
| coal over the wage, 1900 | 0.057 | 17% | 16% | 0.071 | 16% | 22% |
| worst year for the real wage | -0.160 | 0.042 | 0.027 | -0.176 | 0.028 | 0.031 |
| famine years in a run | 0.39 | 0.64 | 0.35 | 0.58 | 0.67 | 0.42 |

The warm-up's ten years have not yet let the histories part in 1750, so anything a setting does to 1750
shows clearly even at a per cent or two. By 1900 they have parted as far as the islands themselves. A
mean over twenty years helps (real income of 1881-1900 against the single year 1900), and sixteen islands
help; one year on eight islands against one baseline does not.

## The mines under the landlords

Low to high, as a share of today's mean; the islands that agree in brackets; eight islands unless it says
sixteen. "1750" facts are where a setting shows cleanly; "late" is what beats the noise after 1800.

| setting | today | tried | what moved most, low to high | facts beating the noise |
|---|---|---|---|---|
| `reach` | 2.0 | 1.5, 2.7 | (sixteen) workings -43% at every date (0/16), each +47% of its cell (16/0); stone a head 1750 +11% (16/0), 4.8% cheaper (0/16). Late: railway 1900 -9.5% (1/15); baskets 1900 -4.8% (3/13), real wage 1900/1750 -5.9% (4/12), coal a head 1900 -7.4% (5/11) | 52; 36 on sixteen |
| `haul` | 0.05 | 0.0375, 0.0667 | (sixteen) stone a head 1750 -3.7% (1/15), 1.6% dearer (14/2); colliers' share of hands 1800 +8% (15/1). Coal over the wage 1900 -9% (8/8): nothing late | 22; 7 on sixteen |
| `sink` | 40 | 30, 53 | workings 1900 -11% (0/8), 1750 -5% (0/8); stone a head 1750 -5.2% (1/7), 2.7% dearer (8/0); carters' share of hands 1750 -1.7% (0/8). Nothing late | 37 |
| `payback` | 4 | 3, 5.33 | workings 1900 +11% (8/0), 1750 +4% (8/0); stone a head 1750 +5.9% (8/0), 3.1% cheaper (0/8); goods a head 1750 +0.4% (7/1). Nothing late | 44 |
| `idle_years` | 10 | 8, 13 | clay a head 1750 +12.5% (8/0); workings 1800 +6.3% (8/0); stone a head 1750 -1.2% (0/8). Nothing late | 19 |
| `memory` | 0.3 | 0.225, 0.4 | woodland 1750 -0.2% (0/8); people 1850 +1.0% (8/0, z 3.4) | 6 |
| `year_room` | 0.15 | 0.1125, 0.2 | quarries 1750 +1.2% (7/0); clay over the wage 1881-1900 +17% (7/1). Real income 1881-1900 -3.6% but 3/5 | 12 |
| `reserve` | 20,000,000 | 15,000,000, 26,700,000 | coal a head 1881-1900 +7.2% (8/0); colliers' share 1900 -11% (0/8); ore a head 1841-60 -15% (0/8) | 9 |
| `hands_per_cell` | 8,000 | 6,000, 10,667 | land under the diggings -57% of today's in every year (0/8): 1/x, as built. Nothing else | 15 |
| `cell_limit` | 0.5 | 0.375, 0.667 | nothing: 13 of 16 runs byte for byte today's | 0 |
| (null) | | `sink` 40.0001, 40.0002 | | 3 |

## The mines under the commons

| setting | today | tried | what moved most, low to high | facts beating the noise |
|---|---|---|---|---|
| `reach` | 2.0 | 1.5, 2.7 | (sixteen) workings -41% (0/16), each +45% of its cell (16/0); stone a head 1750 +10% (16/0), 5.1% cheaper (0/16); goods a head 1750 +0.7% (13/3, z 12). Late: coal a head 1900 -6.6% (4/12); nothing else | 45; 34 on sixteen |
| `haul` | 0.05 | 0.0375, 0.0667 | (sixteen) stone a head 1750 -5.7% (0/16), 2.4% dearer (16/0). Coal over the wage 1900 +21% (10/6): opposite to the landlords and agreed under neither | 26; 7 on sixteen |
| `sink` | 40 | 30, 53 | workings 1900 -11% (0/8); stone a head 1750 -6.4% (0/8), 2.8% dearer (8/0); carters' share 1750 -2.4% (0/8). Nothing late | 38 |
| `payback` | 4 | 3, 5.33 | workings 1900 +12% (8/0); stone a head 1750 +6.3% (8/0), 2.9% cheaper (0/8); carters' share 1750 +2.2% (8/0). Nothing late | 42 |
| `idle_years` | 10 | 8, 13 | clay a head 1750 +13% (8/0); stone a head 1750 -1.3% (0/8). Nothing late | 17 |
| `memory` | 0.3 | 0.225, 0.4 | iron over the wage 1751-70 -4.2% (1/7) | 3 |
| `year_room` | 0.15 | 0.1125, 0.2 | collieries 1900 -11% (1/7); works 1850 -14% (0/8, z -3.4) | 8 |
| `reserve` | 20,000,000 | 15,000,000, 26,700,000 | clay a head 1900 -13% (1/7) | 1 |
| `hands_per_cell` | 8,000 | 6,000, 10,667 | land under the diggings -55% of today's in every year (0/8). Nothing else | 12 |
| `cell_limit` | 0.5 | 0.375, 0.667 | nothing: 14 of 16 runs byte for byte today's | 0 |
| (null) | | `sink` 40.0001, 40.0002 | | 3 |

The two regimes answer the mines' numbers alike in 1750, where the two are one island until the first
rents are paid. After that, the one difference I can see is that the late effects under the landlords
(`reach` on the railway, the baskets and the coal) do not appear under the commons. The commons island
has half the railway and a railway that chance moves by 39 per cent, so it may be there and drowned.

## What the numbers do, in a sentence each

- **`reach`** sets the number and size of the workings, as the page said it would with `sink`. A longer
  lease gathers more deposit behind fewer pit heads; 1750's stone is cheaper because each quarry works more
  rock from the foot of the hill, and the century's railway is shorter because the pit heads are fewer and
  already stand where the carting is short (`mines.md` saw the branches to quarries go from 72 to 13 when
  the mechanism came in; this is the same lever turned further).
- **`sink / payback`** is the price of opening: a lower threshold, more and smaller workings and cheaper
  stone in 1750. The wage bill of the sinkers, which is what `sink` alone would seem to be, does next to
  nothing: (`sink` 53, `payback` 5.33) against (30, 3), the same threshold with a third more sinking, beats
  the noise in 5 and 4 facts, none of them a headline.
- **`haul`** acts in 1750 through stone, a few per cent; after that it is lost in the noise.
- **`idle_years`** decides how many of the warm-up's surplus workings are still standing to be given up
  in 1749; clay is 13 per cent more a head in 1750 with 13 years than with 8. Nothing after 1800 moves.
- **`reserve`** at a third either side of 20 million moves the late coal by 7 per cent, as the page found
  between 6, 20 and 40 million.
- **`hands_per_cell`** is the diggings' surface and nothing else, which is what it was built to be.
- **`cell_limit`** binds only where one cell's workings would take more than half of it, which hardly
  happens: the largest share any working takes in 1900 is about a fifth on the typical island.
- **`memory`** and **`year_room`** were chosen to steady the opening of pits from year to year; at a third
  either side they do nothing a null change does not.

## The chronicle's thresholds

(to come)

## The settings that matter most

## The settings that do nothing at these sizes

## Values at which a run breaks

## Pairs worth trying together

## What I would look at first in the pass over the rules

## How it was run
