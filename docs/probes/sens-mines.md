# sens-mines: how much the mines' numbers and the chronicle's thresholds matter

One of six tables of sensitivity for the pass over all the rules (piece 18 of
`docs/backlog/found-along-the-way.md`: most of the mechanisms' numbers were set on a few islands and never
searched). This one is the mines (`docs/rules/mines.md`), under the landlords and under the commons, and
the chronicle's own thresholds (`docs/rules/chronicle.md`). Branch `night/o-sens-mines`, cut from `main` at
a6ff0ec (12 November 2026). No rule and no fact was changed: this page is the only thing on the branch.

Every run is the game's rules with six families, from 1740 to 1900. Each of the mines' ten numbers was
run at about three quarters and four thirds of its value on islands 1 to 8, under both regimes, and three
of them again on islands 9 to 16. Each of the chronicle's twenty-two thresholds was run the
same way on islands 1 to 8, under the landlords only (the chronicle's example takes the game's rules as
they are). The sweeps and the scripts that read them are under
`.claude/night/reports/opus/sens-mines/` (the scripts in `scripts/`); none of it is tracked.

## In brief

- **Chance is as large as the islands by 1900.** A change too small to matter (the mines' `reserve` from
  20,000,000 to 20,000,001) sends every island down another history, and by 1900 the histories of one island
  differ as much as the islands differ from each other: baskets a head by 3 per cent, the real wage against
  1750 by 4, the railway by 5 (39 under the commons), the price of coal by 17. In 1750 they differ by a
  tenth of a per cent to one per cent. So nothing in the late century can be read from one baseline on eight islands, and
  the usual "7/1" of a sweep's `--compare` is a weak test there. This changed how the rest was read (next
  section), and it is the first thing I would carry into the pass over the rules.
- **`reach` is the mines' strongest number.** From 1.5 cells to 2.7 there are a third fewer workings in
  1750 and two fifths fewer by 1900 (0/16 under both regimes at every date), each taking 45 per cent more of
  its cell; stone raised a head in
  1750 is a tenth higher and 5 per cent cheaper (16/0, 0/16). Under the landlords the island of 1900 has 9.5
  per cent less railway (1/15) and, on most islands though not all, 5 per cent fewer baskets a head (3/13)
  and 7 per cent less coal a head (5/11).
- **`sink` and `payback` are one number.** A site opens when its royalty would repay `sink / payback` wages
  a month. Moving either moves the count of workings by a ninth and the stone of 1750 by 5 or 6 per cent,
  in mirror image; moving both so that the ratio stays put moves nothing the noise does not.
- **`cell_limit` does nothing** at these sizes: on 27 of the 32 runs every fact is today's to the last digit.
  `hands_per_cell` moves only the land under the diggings, as 1/x. `memory` and `year_room` are at the
  noise floor. `reserve` does what its page says, a little more coal and fewer colliers late. `idle_years`
  shapes the workings the warm-up gives up and nothing after.
- **Nothing breaks.** Every one of the 608 runs of the mines' sweeps finished and passed the monthly money check.
  The worst year for the real wage, -0.45, is a null history (island 9 under the landlords, where one town
  pays 84.5 times the middle town's wage for a month in 1782): the bad year that `mines.md` already owns up
  to is there at today's values, about one history in sixty-four, and no setting here clearly changes how
  often.
- **The chronicle's feed is two numbers**, `per_decade` and `floor`; the player's share of it is the `own_*`
  weights against `floor`; the regional thresholds change what is told and hardly what is kept, and only
  `closures_min` moves even that by more than chance moves it from one history to the next. Three of its
  twenty-two numbers (`at_least`, `fill_floor`, `base_years`) do nothing at a third either way, and the
  two of the overtakings next to nothing.

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
| `reach` | 2.0 | 1.5, 2.7 | (sixteen) workings -34% in 1750, -35% in 1800, -38% in 1850, -43% in 1900 (0/16 each), each +47% of its cell (16/0); stone a head 1750 +11% (16/0), 4.8% cheaper (0/16). Late: railway 1900 -9.5% (1/15); baskets 1900 -4.8% (3/13), real wage 1900/1750 -5.9% (4/12), coal a head 1900 -7.4% (5/11) | 52; 36 on sixteen |
| `haul` | 0.05 | 0.0375, 0.0667 | (sixteen) stone a head 1750 -3.7% (1/15), 1.6% dearer (14/2); colliers' share of hands 1800 +8% (15/1). Coal over the wage 1900 -9% (8/8): nothing late | 21; 7 on sixteen |
| `sink` | 40 | 30, 53 | workings 1900 -11% (0/8), 1750 -5% (0/8); stone a head 1750 -5.2% (1/7), 2.7% dearer (8/0); carters' share of hands 1750 -1.7% (0/8). Nothing late | 37 |
| `payback` | 4 | 3, 5.33 | workings 1900 +11% (8/0), 1750 +4% (8/0); stone a head 1750 +5.9% (8/0), 3.1% cheaper (0/8); goods a head 1750 +0.4% (7/1). Nothing late | 44 |
| `idle_years` | 10 | 8, 13 | clay a head 1750 +12.5% (8/0); workings 1800 +6.3% (8/0); stone a head 1750 -1.2% (0/8). Nothing late | 19 |
| `memory` | 0.3 | 0.225, 0.4 | woodland 1750 -0.2% (0/8); people 1850 +1.0% (8/0, z 3.4) | 6 |
| `year_room` | 0.15 | 0.1125, 0.2 | quarries 1750 +1.2% (7/0); clay over the wage 1881-1900 +17% (7/1). Real income 1881-1900 -3.6% but 3/5 | 12 |
| `reserve` | 20,000,000 | 15,000,000, 26,700,000 | coal a head 1881-1900 +7.2% (8/0); colliers' share 1900 -11% (0/8); ore a head 1841-60 -15% (0/8) | 9 |
| `hands_per_cell` | 8,000 | 6,000, 10,667 | (sixteen) land under the diggings -58% of today's in every year (0/16): 1/x, as built. Baskets a head 1900 +4.4% (10/6): chance, see below | 15; 12 on sixteen |
| `cell_limit` | 0.5 | 0.375, 0.667 | nothing: on 13 of 16 runs every fact is today's | 0 |
| (null) | | `sink` 40.0001, 40.0002 | | 3 |

## The mines under the commons

| setting | today | tried | what moved most, low to high | facts beating the noise |
|---|---|---|---|---|
| `reach` | 2.0 | 1.5, 2.7 | (sixteen) workings -37% in 1750, -41% in 1900 (0/16 each), each +45% of its cell (16/0); stone a head 1750 +10% (16/0), 5.1% cheaper (0/16); goods a head 1750 +0.7% (13/3, z 12). Late: coal a head 1900 -6.6% (4/12); nothing else | 45; 34 on sixteen |
| `haul` | 0.05 | 0.0375, 0.0667 | (sixteen) stone a head 1750 -5.7% (0/16), 2.4% dearer (16/0). Coal over the wage 1900 +21% (10/6): opposite to the landlords and agreed under neither | 26; 7 on sixteen |
| `sink` | 40 | 30, 53 | workings 1900 -11% (0/8); stone a head 1750 -6.4% (0/8), 2.8% dearer (8/0); carters' share 1750 -2.4% (0/8). Nothing late | 38 |
| `payback` | 4 | 3, 5.33 | workings 1900 +12% (8/0); stone a head 1750 +6.3% (8/0), 2.9% cheaper (0/8); carters' share 1750 +2.2% (8/0). Nothing late | 42 |
| `idle_years` | 10 | 8, 13 | clay a head 1750 +13% (8/0); stone a head 1750 -1.3% (0/8). Nothing late | 16 |
| `memory` | 0.3 | 0.225, 0.4 | iron over the wage 1751-70 -4.2% (1/7) | 3 |
| `year_room` | 0.15 | 0.1125, 0.2 | collieries 1900 -11% (1/7); works 1850 -14% (0/8, z -3.4) | 8 |
| `reserve` | 20,000,000 | 15,000,000, 26,700,000 | clay a head 1900 -13% (1/7) | 1 |
| `hands_per_cell` | 8,000 | 6,000, 10,667 | (sixteen) land under the diggings -55 to -59% of today's in every year (0/16). Nothing else | 12; 11 on sixteen |
| `cell_limit` | 0.5 | 0.375, 0.667 | nothing: on 14 of 16 runs every fact is today's | 0 |
| (null) | | `sink` 40.0001, 40.0002 | | 3 |

The two regimes answer the mines' numbers alike in 1750, where the two are one island until the first
rents are paid. After that, the one difference I can see is that the late effects under the landlords
(`reach` on the railway, the baskets and the coal) do not appear under the commons. The commons island
has half the railway and a railway that chance moves by 39 per cent, so it may be there and drowned.

## What the numbers do, in a sentence each

- **`reach`** sets the number and size of the workings, as the page said it would with `sink`. A longer
  lease gathers more deposit behind fewer pit heads, and 1750's stone is cheaper because each pit head
  works more rock. The century's railway is shorter, I take it, because the pit heads are fewer and already
  stand where the carting is short: `mines.md` saw the branches to quarries go from 72 to 13 when the
  mechanism came in, and this is the same lever turned further. I did not trace it.
- **`sink / payback`** is the price of opening: a lower threshold, more and smaller workings and cheaper
  stone in 1750. The wage bill of the sinkers, which is what `sink` alone would seem to be, does next to
  nothing: (`sink` 53, `payback` 5.33) against (30, 3), the same threshold with a third more sinking, beats
  the noise in 5 and 4 facts, none of them a headline.
- **`haul`** acts in 1750 through stone, a few per cent; after that it is lost in the noise.
- **`idle_years`** decides how many of the warm-up's surplus workings are still standing to be given up
  in 1749; clay is 13 per cent more a head in 1750 with 13 years than with 8. Nothing after 1800 moves.
- **`reserve`** at a third either side of 20 million moves the late coal by 7 per cent, as the page found
  between 6, 20 and 40 million.
- **`hands_per_cell`** is the diggings' surface and nothing else, which is what it was built to be. On
  sixteen islands under the landlords the late standard of living leans up with it (baskets a head in 1900
  +4.4 per cent, 10/16; the real wage against 1750 +4.3, 12/16), but five cells' worth of land in five
  thousand cannot carry four per cent of anything, and no island-by-island agreement says otherwise. I read
  it as chance.
- **`cell_limit`** binds only where one cell's workings would take more than half of it, which hardly
  happens: the largest share any working takes in 1900 is about a fifth on the typical island.
- **`memory`** and **`year_room`** were chosen to steady the opening of pits from year to year; at a third
  either side they do nothing a null change does not.

## The chronicle's thresholds

The chronicle reads the economy and never changes it, so an island's history is the same at every
threshold and each difference below is exact: there is no noise to beat, and "8/0" means every island told
more. Today's values on islands 1 to 8: 1,002 items in a run, 59.0 in the major feed (58 to 60), 44 regional
stories told of which 7.0 are kept, 12.2 of the player's own items kept. For scale, chance in the economy
moves an island's regional stories by about 5 and its items by 23 to 36 from one history to the next (the
null histories of the mines).

"Told" is every item of the kind; "kept" is what a history of its decade keeps, the major feed. Low value
first, then high.

| setting | today | tried | what moved most, low; high | islands |
|---|---|---|---|---|
| `per_decade` | 4 | 3, 5 | the feed -14.0 of 59; +11.6. Regional stories kept -3.5; +4.5, markets -3.3; +2.0, milestones -2.1; +1.8 | 8 of 8 each way |
| `at_least` | 2 | 1, 3 | nothing: no decade is thinner than four places allow | 0 |
| `per_kind` | 1 | 2 | dated slumps kept +2.3, turning points +1.4, the player's +1.0; declines -1.0, markets -0.9; the feed +1.0 | 8/0, 7/0, 6/0; 0/6, 0/6; 5/0 |
| `inventions` | 2 | 1, 3 | inventions kept -3.1 of 10, their places to regional stories (+1.3) and markets (+0.6); +0.8, from markets (-0.6) | 0/8, 7/0; 5/0, 0/5 |
| `floor` | 0.6 | 0.45, 0.8 | the player's kept +2.8 (everyday news at 0.5 gets in); the feed -22.4, the player's -10.6 of 12.2, regional stories -5.8, and 13 decades of 15 short of four (none under two) | 7/0; 0/8 |
| `fill_floor` | 0.45 | 0.34, 0.6 | nothing | 0 |
| `own_great` | 0.7 | 0.525, 0.93 | the player's kept -4.5 of 12.2, regional stories +1.3; nothing | 0/8, 7/0; 0 |
| `own_small` | 0.5 | 0.375, 0.667 | nothing; the player's kept +2.8 (births and sales pass the floor) | 0; 7/0 |
| `own_summary` | 0.45 | 0.34, 0.6 | nothing; the player's kept +2.0 (the branches' yearly summary passes it) | 0; 7/0 |
| `base_years` | 5 | 4, 7 | one milestone on one island | 1 |
| `boom_years` | 5 | 4, 7 | coal booms told -0.9 of 5.5; +0.5 | 2/5; 4/2 |
| `boom_growth` | 1.5 | 1.375, 1.667 | coal booms told +1.8, kept +0.9 of 1.0; told -1.1, kept -0.4 | 7/1, 5/0; 0/5, 0/3 |
| `boom_share` | 0.15 | 0.1125, 0.2 | coal booms told +1.5, kept +0.4; -1.4 | 7/0, 3/0; 0/7 |
| `decline_years` | 8 | 6, 11 | declines told +2.0 of 11.5; -1.1 | 6/0; 1/5 |
| `decline_fall` | 0.1 | 0.075, 0.133 | declines told +1.1; -2.5, kept -0.6 of 2.6 | 7/0; 0/8, 0/3 |
| `overtake_years` | 3 | 2, 4 | overtakings +0.3 of 3.2; -0.4 (told and kept) | 2/0; 0/3 |
| `overtake_margin` | 0.05 | 0.0375, 0.0667 | overtakings +0.1; -0.5 | 1/0; 0/3 |
| `fastest_margin` | 0.1 | 0.075, 0.133 | fastest-growing towns told +0.4 of 11.9; -1.0 | 2/0; 0/5 |
| `closures_min` | 5 | 4, 7 | rounds of closures told +8.5 of 11.9; -7.8. Kept +0.1; -0.4 | 8/0; 0/8 |
| `closures_share` | 0.1 | 0.075, 0.133 | rounds of closures told +0.5; -1.5 | 4/1; 0/7 |
| `new_towns_min` | 3 | 2, 4 | items on new towns +2.3 of 12.2, kept +0.3; -0.5 | 8/0, 2/0; 0/4 |
| `min_people` | 5,000 | 3,750, 6,667 | declines told +2.1, kept +0.4; -2.4 | 8/0, 3/0; 0/8 |
| `quiet_years` | 15 | 11, 20 | regional stories told +1.1 (coal booms +0.9); -0.8 | 6/0; 0/5 |

What it comes to:

- **The feed's length is two numbers, `per_decade` and `floor`.** Nothing else moves it by more than one
  item in 59. `per_decade` 3 gives the sparer history the chronicle's page wondered about: two or three
  items in every decade, and it is regional stories, markets and milestones that go, not inventions or the
  player. `floor` 0.8 is too high, and shows a dependence the page leaves unsaid: the player's place is
  kept at `floor` (the page writes its value, 0.6), so a floor above `own_great` empties it.
- **The player's own news is the `own_*` weights against `floor`.** Each moves the player's kept items by 2
  to 4.5 when it crosses the floor and by nothing when it does not; lowering `own_great` below the floor
  takes four and a half of the player's items a run out of the feed; the rest of the news fills most of
  the room (regional stories +1.3; milestones, dearths, new towns and declines about +0.5 each), and the
  feed is one item shorter.
- **The regional thresholds move what is told, hardly what is kept.** One of a kind a decade caps the kept
  stories, so a third either way moves the kept count by under one item a run for every threshold. What is
  told moves by 0.1 to 2.5 stories a run, less than chance moves an island's regional stories from one
  history to the next (about 5), with one exception: `closures_min`. At 4 the rounds of closures told
  nearly double (11.9 to 20.4), at 7 they are a third (4.2): the line of five works sits on a steep part of
  the distribution, as its page found when it was moved from ten.
- **Nothing at all**: `at_least`, `fill_floor`, `base_years`, the `own_*` weights on the side away from the
  floor, and the overtakings' two numbers (under half a story a run).

## The settings that matter most

1. **`mines.reach`**, which sets how many workings there are and how large, from the first year to the
   last, and the only number of the mines whose effect after 1800 holds on sixteen islands (the railway
   under the landlords).
2. **`mines.sink / mines.payback`**, as one number: the count of workings by a ninth either way and 1750's
   stone by 5 or 6 per cent. Nothing late.
3. **`chronicle.per_decade` and `chronicle.floor`**, which between them are the length of the major feed
   (±12 to 22 items of 59), and `floor` also decides whether the player has a place at all.
4. **`chronicle.closures_min`**, the one regional threshold whose story count is steep: 20 rounds of
   closures told at 4, 12 at 5, 4 at 7.

## The settings that do nothing at these sizes

- **`mines.cell_limit`**: at 0.375 and 0.667 every fact is today's on 27 runs of 32; on the other five the
  limit bound once and the island went down another history, with nothing that beats the noise.
- **`mines.memory`** and **`mines.year_room`**: 3 to 12 facts beat the noise, against a floor of 3, and none
  of them is a headline after 1750.
- **`mines.hands_per_cell`** does what it says, the land under the diggings, and nothing else.
- **`mines.haul`**, **`mines.idle_years`** and **`mines.reserve`** act on 1750 or on one good, by a few per
  cent; nothing they do reaches the standard of living.
- **`chronicle.at_least`, `fill_floor` and `base_years`**: no decade is thin enough for the first two to
  act, and the doubling's baseline moves one milestone on one island. The `own_*` weights do nothing on the
  side of the floor they already sit, and `overtake_years` and `overtake_margin` move under half a story a
  run.

## Values at which a run breaks

None of these. All 608 runs of the mines' sweeps (the baselines among them) finished and passed the monthly money check, under both
regimes; so did every run of the chronicle's. No town emptied: the most famine years any run had was four
(`haul` 0.0375, landlords, island 6), against three in the null histories, and famine years are chance (the
spread between histories of one island is as large as the mean or larger).

The worst year for the real wage is the one to watch, and it is chance too. Two runs went below -0.4. One
is `reach` 1.5, landlords, island 6, at -0.40: Cranbourne, a town founded in 1749, pays sixteen times the
middle town's wage in April 1765, and the island's wage goes from 4.35 in 1764 to 9.94 in 1765 and back to
7.67 (`examples/wage_watch.rs game 6 on 6 6 mines.reach=1.5`). The other, and the worse, is a null
history: `sink` 40.0002, landlords, island 9, at -0.45, where Iveldelf, founded in 1776, pays 84.5 times
the middle town's wage in September 1782, with twelve town-months over four times it and eight in which a
town has half as many hands again at work as it has people (`... game 6 on 9 9 mines.sink=40.0002`). So the
bad year that `mines.md` names in its worries ("a working can ask for more hands than its town has") is
there at today's values, in about one history in sixty-four under the landlords (`badyears.py`, below
-0.3: 1 of 64 null histories, 1 of 240 runs at other values), and no setting here makes it clearly likelier.
I did not trace whether Iveldelf's hiring is a mine's or a works'. Under the commons six runs of 240 went
below -0.3, five of them on island 6, which sits at -0.25 to -0.30 in its null histories as well: that
island has a bad year in every history, and the settings only move it a little.

## Pairs worth trying together

- **`sink` with `payback`**, done here: they are one number for opening and almost nothing else. If the
  pass wants fewer knobs, the rule could be stated as a threshold in wages a month (today 10) and a
  sinking cost; the second could be left where it is.
- **`reach` with `haul`**: a longer reach brings in cells further from the pit head, which `haul` then
  discounts; the effective size of a lease is the two together. And `reach` with `reserve`, since a larger
  lease has more deposit to tire.
- **`reach` by sector**: `mines.md` suggests a longer reach for quarries alone, to keep them at the foot of
  the hill in 1900. That is a rule change, not a setting, but it is where this table points.
- **`idle_years` with `year_room`**: the one setting `mines.md` warns against is `idle_years` 1 with no
  `year_room`, hasty closing and unlimited opening together. Each alone does nothing late at a third either
  side; it is the pair that can ring.
- **`hands_per_cell` with `cell_limit`**: the limit can only bind when the hands fill cells quickly, so a
  small `hands_per_cell` is the only place it could be tested.
- **`chronicle.floor` with `own_great`**: the player's place is kept at `floor`, so the two have to move
  together or the player vanishes from the feed (`floor` 0.8 keeps 1.6 of the player's items a run for 12.2).
- **`chronicle.per_decade` with `per_kind`**: `per_decade` 3 takes out regional stories and markets, and
  `per_kind` 2 lets slumps and turning points take two places; together they say what kind of history the
  feed is.
- **`chronicle.boom_growth` with `boom_share`**: each moves the coal booms told by about a third either
  way, and both are tied to how the mines open, which `reach` and `sink / payback` move.

## What I would look at first in the pass over the rules

1. **Measure chance before reading any fact after 1800.** For every table of this pass: a pair of null
   changes per regime, each setting read high against low, sixteen islands, twenty-year means. On one
   baseline and eight islands a sweep reads the commons' luck as an effect of whatever was moved; this
   page's first two settings would have said that both directions of `reach` and `haul` cost the commons
   4 to 9 per cent of its late real wage.
2. **The regimes' own comparison** in the README (the commons with more people, a lower wage, half the
   railway and two thirds of the canal) is one history per island; the railway alone moves by 39 per cent
   by chance under the commons. It wants the same treatment before it is called a finding.
3. **`reach` and the railway.** The one late effect of the mines that holds on sixteen islands is that
   fewer, larger workings mean less railway; whether a large quarry now deserves a branch it does not get
   is the question `mines.md` already hands to `railways.md`.
4. **The bad year**, a town that hires past its people and pays eighty times the middle wage for a month.
   It turns up at today's values in about one history of sixty-four under the landlords, so any sweep of
   any setting will now and then meet it and blame the setting. The mend at the root that `mines.md`
   describes (a working that hires no more than its town has to spare) would take that noise out of every
   table of this pass.
5. **`cell_limit` could go**, or stay as a guard that never fires.
6. **The chronicle's `floor` and the player's place.** The code keeps the player's place at `floor`, so
   the two weights move together; `chronicle.md` writes the floor's value, 0.6, where it describes the
   place, and would be clearer naming `floor` and saying that it must stay below `own_great`.
7. **`closures_min`** was lowered from ten to five when the life cycle for works came in, on counts from
   four islands and then twelve; it is the chronicle's one steep threshold, so it wants another look
   whenever the works' sizes move.

## How it was run

The sweeps (each under `.claude/night/reports/opus/sens-mines/`, with its command in its `README.txt`):

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --out base-l --compare merged-game-f6-l --quiet
isostate-core sweep --seeds 1-8 --rules game --families 6 --set rent_regime=commons --out base-c --quiet
isostate-core sweep --seeds 1-8 --rules game --families 6 --set rent_regime=landlords,commons \
    --set mines.reach=1.5,2.7 --out m-reach --quiet --jobs 16          # and so for each setting
isostate-core sweep ... --set mines.reserve=20000001 --out m-null      # the null changes
isostate-core sweep ... --set mines.sink=40.0001,40.0002 --out m-null2
isostate-core sweep ... --set mines.sink=30,53 --set mines.payback=3,5.33 --out p-sink-payback
isostate-core sweep --seeds 9-16 ...  --out base-16, m-null-16, m-null2-16, m-reach-16, m-haul-16, m-hands_per_cell-16
```

The first said "of the 446 facts both have, the seconds aside, 0 differ" against today's main. The
chronicle's, one island a process so that each island's counts can be set against its own:

```bash
cargo run --release -p isostate-core --example chronicle -- --seed N --counts [--set chronicle.NAME=VALUE]
```

The scripts that read them (Python, not tracked; in `scripts/` beside the sweeps): `slope.py` (high
against low against the null histories), `panel.py` (the headline facts), `noise.py` (chance against the
islands), `same.py` (which runs are today's to the last digit), `extremes.py` and `badyears.py` (failures, worst
years, famine), `worstyear.py` (the years round a run's worst), `chron_queue.py` and `chron_read.py` (the chronicle). The values tried were three quarters and
four thirds of today's, rounded where a setting is a whole number (`idle_years` 8 and 13) and, for
`reach`, chosen so that the lease changes: 1.5 cells is a lease of 9 cells, 2.0 of 13, 2.7 of 21.
