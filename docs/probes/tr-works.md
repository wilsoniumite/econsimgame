# tr-works: growth that stops, kept town ground, the works' lives and the wage's worst year

One of four traces with the books of account (piece 12 of `docs/backlog/found-along-the-way.md`): things the sweeps had measured and nobody had explained. Branch `night/o-trace-works`, cut from `main` at 295a379, on 12 November 2026. Nothing under `crates/isostate-core/src` was changed; one example was added to measure with, `crates/isostate-core/examples/trace_works.rs`. Everything is under the game's rules with six families. The raw output (every sweep, every table the example printed, and a file of notes with each command in order) is under `.claude/night/reports/opus/trace-works/`, which is not in the repository.

The example steps each island a year at a time with the books kept and prints three sections, each the mean over the islands it is given unless it says otherwise:

```bash
cargo run --release -p isostate-core --example trace_works -- [--rules port|game] [--families N]
    [--set JSON] [--only growth|life|worst] [SEED ...]
```

`growth` gives output per head three ways by decade, each good's output per head at fixed prices and its part in the change, the hands and output per hand of every sector and trade, what each method has learned and how good the machines are, and the towns by when they were founded. `life` takes apart the works that closed: why the review closed them, by trade and decade of founding, and the chance a works closes in a year at each age. `worst` takes apart each island's worst year for the real wage: the money wage against the cost of living, which goods raised the cost of living, the harvest, the town that moved the wage most, and who bought the good behind it. Switching a mechanism is done with the sweep (`--set key=v1,v2`), and the decade means quoted below are each island's mean over the decade, as an index over its own 1750s, averaged over islands, read from the sweep's `annual.csv`.

## In short

1. **Growth stops because the machines stop getting better.** Output per head does not fall after 1879 in any measure of quantities; it stops rising. What fall there is in the observer's figure is its price index: it deflates by the cost of the basket of 1750, which is heavy in timber, wool and grain, and those grow dearer once nothing more is gained on the land. What stops is the farms, quarries, clay pits and mines, all of which take their gains from one curve, the machines' quality, which reaches its ceiling of 4 (`machines.quality_cap`) in about 1871 on the median island. Lift the ceiling to 8 and growth in the last two decades goes from 4 per cent to 17, and the year output per head peaks moves from 1879 to 1895. Two and a half times the people, the money rule, and a readier willingness to invest each change the level or the timing, not the stall.
2. **Kept town ground now costs less than one per cent of the wage, and one works in thirty.** Measured again on sixteen islands, the wage and output per head over twenty-year spans are within one per cent with the town ground kept and without. The two suspected causes of the fewer works, the new town's first year reckoned at its donor's wage and the floor under town ground's rent, are not the cause: switching either or both off leaves the works as few or fewer. What is left is the farmland the towns take (3 per cent of it, on every island) and where the works may stand, and neither moves the wage now.
3. **A works that closes dies younger than under the port because it stops being "building" within a year.** Under the port a works with anything on order cannot be counted idle, and an order is never quite delivered, so a works is sheltered from the three-losing-years rule for five to seven years. Under the life cycle it is out of that shelter in its first year. At ages 3 to 5 the game's works close at 4.8 per cent a year where the port's close at 0.9. Put the shelter back (`works.built_share=0`) and the median life goes from 11.7 years to 21.5.
4. **The worst year is a building wave buying the whole year's timber, with an ordinary harvest after a good one.** On fourteen islands of sixteen it falls between 1874 and 1899, the money wage hardly moves, and the cost of living rises 14 to 24 per cent, three fifths of it timber. In that year works being built buy as much timber as the island makes, households are priced out of it, and the price jumps by two thirds. Grain and wool, from a harvest that is middling after a good one, add about a quarter. With harvests off the worst year is 12 per cent down instead of 15; with woods that regrow twice as fast, 13. The "worse at a merge" of C134 was this same year landing on a deeper wave on six islands of eight; it was back where it had been by the next merge.

## 1. Growth that stops

**The question.** The README says output per head under the game's rules peaks in 1879 on average and falls after, and that nobody knows why. Is it technology run out after steel, people growing faster than the works, the land, the money rule, or works that stop investing?

### What falls and what only stops

Sixteen islands, the decades' means as an index over each island's 1750s (`trace_works 1 2 ... 16`):

| | 1850s | 1860s | 1870s | 1880s | 1890s |
|---|---|---|---|---|---|
| output per head, the observer's (`gdp_pc`) | 2.22 | 2.56 | 2.67 | 2.66 | 2.59 |
| spending per head over the chained cost of living | 3.19 | 3.75 | 4.06 | 4.16 | 4.18 |
| all that is made, per head, at fixed prices | 2.47 | 2.83 | 3.03 | 3.10 | 3.13 |
| made for use (not worked up again), per head, at fixed prices | 2.21 | 2.55 | 2.75 | 2.83 | 2.86 |
| the 1750 basket's cost of living | 0.95 | 0.89 | 0.94 | 1.03 | 1.17 |
| the chained cost of living | 0.66 | 0.61 | 0.62 | 0.66 | 0.72 |
| real wage, chained | 3.06 | 3.71 | 4.07 | 4.18 | 4.20 |
| money wage | 1.97 | 2.23 | 2.49 | 2.73 | 3.03 |

"Fixed prices" are each good's price on the island from 1860 to 1879, weighted by what each town made; "all that is made" counts a ton of iron and the tools made from it both, "made for use" counts only what households, landlords, families, farms and pits, building and the railways and canals bought.

The observer's `gdp_pc` is not a count of things. It is the island's spending (households, landlords, house rent and the works' investment) divided by the cost of the basket households bought in 1750 (`observer::record_month`). Every measure that counts goods says the same: output per head grew by a quarter from the 1850s to the 1870s and by 3 or 4 per cent from the 1870s to the 1890s. It does not fall. The observer's figure falls because the 1750 basket grows 25 per cent dearer over those twenty years while the chained one grows 17 per cent dearer: the old basket is heavy in timber, wool and grain, and from the 1870s to the 1890s timber's price rises by 31 per cent, wool's by 35 and grain's by 19, where steel's rises by 2 and services' by 13. By fixed prices output per head peaks in the 1880s or 1890s on thirteen islands of sixteen; by `gdp_pc` it peaks in the 1870s on eight and the 1880s on five.

Prices rise in general after the 1870s, a fifth or so, where for a century they had fallen. That is the money rule meeting an economy that no longer grows: money is issued at a fixed rate whatever is made.

### Which trades stop

Each good's part in the change of all that is made per head, as a share of the earlier decade's total (sixteen islands):

| good | share of all made, 1870s | part, 1850s to 1870s | part, 1870s to 1890s |
|---|---|---|---|
| services | 0.239 | +0.069 | +0.020 |
| stone | 0.125 | +0.059 | +0.012 |
| steel | 0.054 | +0.034 | +0.015 |
| grain | 0.105 | +0.028 | +0.002 |
| cloth | 0.160 | +0.015 | -0.007 |
| iron | 0.028 | +0.014 | +0.004 |
| coal | 0.048 | +0.009 | -0.001 |
| wool | 0.107 | +0.002 | -0.006 |
| timber | 0.012 | -0.007 | -0.002 |
| all goods | | +0.237 | +0.035 |

And where the hands are and what each makes (output per hand, 1870s = 1):

| | share of hands 1850s / 1870s / 1890s | output per hand 1850s / 1870s / 1890s |
|---|---|---|
| farms | 0.171 / 0.108 / 0.106 | 0.55 / 1.00 / 1.03 |
| pasture | 0.104 / 0.116 / 0.121 | 1.11 / 1.00 / 0.90 |
| quarries | 0.117 / 0.133 / 0.135 | 0.71 / 1.00 / 1.06 |
| clay pits | 0.031 / 0.020 / 0.018 | 0.63 / 1.00 / 1.08 |
| collieries | 0.047 / 0.045 / 0.047 | 0.82 / 1.00 / 0.94 |
| services | 0.293 / 0.353 / 0.363 | 0.93 / 1.00 / 1.05 |
| textile mills | 0.021 / 0.022 / 0.021 | 0.97 / 1.00 / 1.02 |
| brickworks | 0.058 / 0.056 / 0.053 | 0.92 / 1.00 / 1.07 |

The growth of the middle of the century was the farms (farm machinery let a farm hand grow nearly twice as much between the 1850s and the 1870s, and freed a third of all farm hands, who went into services and the quarries), the quarries and clay pits, and steel. After the 1870s the farms' output per hand stops, the quarries' and pits' nearly stops, the hands stay where they are, and nothing else is rising fast enough to carry the island: no works but the converter gains more than 7 per cent a hand in twenty years (the converter 13). The towns stop together: those standing in 1750, those founded before 1800 and those founded since make the same per head relative to the island in every decade from the 1840s (1.04, 0.97 and 0.84 to 0.88), and in seed 1 the five largest towns all make 1 to 11 per cent less per head (at their own prices, deflated) in the 1890s than in the 1870s.

### What stops them

Two curves give this island its productivity, and the example prints both.

- **The methods' learning.** Each method's skill rises with what has been made of it up to `learn_cap` (4). The coke furnace, the textile mill, the engine works, crucible steel and the rolling mill are at the cap by the 1840s or 1850s, the converter by the 1890s. Services learn hardly at all (a third of the cap all century) and brickworks reach half. So the works have grown by hands, not by skill, since the middle of the century.
- **The machines' quality.** Farm machinery, the pumps and wagonways of the mines, and the machines of the quarries and clay pits are all worth more as the machines get better, and quality is counted in steel poured, `(1 + Q / 1,000,000)^0.6` up to `machines.quality_cap`, 4. Its mean by the last year of each decade is 2.43 (1840s), 2.95, 3.57, 3.81, 3.91 and 3.97 (1890s). It reaches the cap between 1863 and 1886 on fifteen islands of sixteen (median 1871), and is still short of it in 1900 on the last.

The second curve reaching its cap in about 1871 and output per head peaking in 1879 looked like the answer, so each suspect was switched in turn (eight islands, seeds 1 to 8, against the same eight under today's rules; "late growth" is the change in spending per head over the chained cost of living from the 1870s to the 1890s):

| switched | late growth | the year output per head peaks | real wage 1900 | the command's `--set` |
|---|---|---|---|---|
| nothing (today's game) | +3.8% | 1879 | 2.60 | |
| machines' quality may reach 8 | **+17.2%** | **1895** | 3.21 | `machines.quality_cap=8` |
| methods may learn to 8 | -1.8% | 1875 | 2.23 | `learn_cap=8` |
| both | +8.5% | 1892 | 2.93 | both |
| people grow at most 1.2% a year (459,000 in 1900) | +10% | 1889 | 2.97 | `people.growth_top=0.012` |
| people grow at most 0.8% a year (287,000 in 1900) | +11% | 1894 | 3.40 | `people.growth_top=0.008` |
| at 0.8%, with machines' quality to 8 | +16.2% | 1897 | 3.54 | both |
| no new money | +12.7% | 1888 | 2.35 | `money_growth=0` |
| money 1% a year | +5.4% | 1880 | 2.50 | `money_growth=0.01` |
| money 3% a year | +0.7% | 1874 | 2.73 | `money_growth=0.03` |
| a hurdle of 8% for growing and founding works | +0.9% | 1872 | 2.61 | `hurdle=0.08` |
| a year's room of 25% | -0.4% | 1872 | 2.54 | `works.year_room=0.25` |

What each says:

- **The machines' ceiling is the cause.** With it at 8 nothing changes until the 1860s (the runs are the same to that point), then output per head goes on rising: all that is made per head at fixed prices grows by 16 per cent over the last two decades, on every island of eight (+10 to +25 per cent), where it grew by 3. With the ceiling lifted, output per hand from the 1870s to the 1890s rises by 44 per cent on the farms where it rose by 3, by 47 in the quarries for 6, 65 in the clay pits, 36 in the collieries; grain gets cheaper, not dearer (0.91 of its 1870s price), and the 1750 basket's cost rises by 15 per cent where it rose by 27 (`trace_works --only growth --set '{"machines":{"quality_cap":8}}'`). The works are untouched: their output per hand moves by the same 2 to 7 per cent with the ceiling up or down.
- **Letting the methods learn further makes it come sooner.** A higher `learn_cap` makes more steel sooner, and steel poured is what the machines' quality is counted in, so the ceiling is reached a decade earlier, and then the island falls. Both raised: it stalls a decade later.
- **People are not the cause.** Fewer people do keep growth going longer, but a small island pours less steel and so reaches the machines' ceiling later (quality 3.25 in the 1870s at 0.8 per cent, 3.81 at today's 1.6). With the ceiling lifted, the island of 287,000 and the island of 732,000 grow at the same pace in the last two decades (16 and 17 per cent), and the small one is only 2 per cent richer in the 1890s. The land does press: pasture and the woods make a tenth less a hand from the 1870s to the 1890s. But that is worth a few per cent of output, not the stall.
- **The money rule is not the cause, though it is not neutral.** Every rate of new money stalls; the faster the money, the higher and sooner the island gets to the stall. With none, the island is nearly a quarter poorer in the 1850s and still growing in the 1890s, which is the same ceiling approached more slowly. That money changes real output at all is worth knowing for the pass (below).
- **Works that stop investing are a symptom.** The works' investment is 4.4 per cent of income in the 1870s and 4.0 in the 1890s, down from 5.0 to 5.2 in the middle of the century. Making growth easier (a lower hurdle, a wider room) does not lift the late decades at all and a wider room makes them worse.

**How sure.** Sure of the mechanism: one number moves the peak sixteen years and the late growth fourfold, on every island of eight, and the sectors that respond are exactly those that use machines. Less sure that a ceiling of 8 is the right repair; it only shows where the wall is. What would change my mind: a run with the ceiling lifted that still stalls on sixteen islands, or a stall that arrives before quality reaches 3.9.

**For the pass over all the rules.** `machines.quality_cap` is the number that ends the century's growth, and nothing after steel takes over from it. Run past 1900 (the README's Malthusian 2050) this is the whole story: there is no technology after the machines' cap. Three things a designer should look at: what the machines' quality should go on being counted in once steel is cheap (it is the only curve that reaches the land); whether `gdp_pc` should be deflated by the chained index, since the 1750 basket turns a stall into a fall and moves the peak year by about a decade; and that money is not neutral here (more new money, faster growth until the ceiling), which matters for K39, the money issued at 1.76 per cent a year where the setting says 2.

## 2. Kept town ground: one works in twenty fewer and 1 to 3 per cent off the wage (R102)

**The question.** `docs/rules/towns.md` found, with the town ground kept, about one works in twenty fewer and 1 to 3 per cent off the real wage and output per head, and suspected two causes it did not separate: the farmland the towns take whole from the plough, and ground that is no longer nearly free (the floor under town ground's rent, `towns.rent_floor`) together with a new town's first year reckoned at its donor's wage (`towns.donor_wage`).

**Measured.** Sixteen islands, today's game, kept ground off (`--set towns.enabled=false`) against on, and then all four combinations of the two small switches:

| | ground not kept | kept, both switches off | kept, donor's wage off | kept, floor off | kept (today) |
|---|---|---|---|---|---|
| works 1900 | 752 | 714 | 718 | 723 | 729 |
| works founded in a run | 1,990 | 1,914 | 1,926 | 1,927 | 1,940 |
| farmland 1900, cells' worth | 1,492 | 1,451 | 1,447 | 1,448 | 1,447 |
| real wage, 1870s / 1880s / 1890s | 2.69 / 2.68 / 2.60 | 2.70 / 2.72 / 2.64 | 2.62 / 2.64 / 2.58 | 2.70 / 2.69 / 2.64 | 2.68 / 2.67 / 2.61 |
| output per head, same decades | 2.69 / 2.68 / 2.60 | 2.70 / 2.70 / 2.63 | 2.62 / 2.63 / 2.56 | 2.68 / 2.67 / 2.61 | 2.67 / 2.66 / 2.59 |

(Decade means of the observer's series over each island's 1750s.)

**The answer.** The cost to the wage is gone: with kept ground the wage of the last three decades is within one per cent of the wage without it, a third of a per cent higher in the 1890s, and output per head is 0.3 to 0.7 per cent lower. It was measured at 1 to 3 per cent before the mines and the canals came in; the mines gave the farms a quarter more land, and that may be why the towns' share of it no longer shows; I did not test it. There are still fewer works, 23 fewer in 1900 (3 per cent; fewer on eleven islands of sixteen). Neither suspect is the cause of that: switching off the donor's wage, the floor, or both, leaves fewer works still (714 to 723), all within the spread between islands. What remains of kept ground is the farmland, 45 cells' worth less on every island of sixteen, and the room it leaves the works: a town that keeps its ground grows over the works beside it and asks a works on its ground to share the cell with houses (`towns.works_room`). Of those two only the room bears on how many works there are, and I did not sweep it; `towns.md` records that its builder did, with wages that did not tell the settings apart.

**How sure.** Sure that the two small switches do not cause the fewer works (four settings, sixteen islands, no ordering). Fairly sure that the wage cost is now under a per cent: three decades agree. The 3 per cent fewer works is small enough to be a few towns' arrangement.

**For the pass.** R102 can close: the wage cost it worried about is no longer there to separate. If the 3 per cent fewer works matters, `towns.works_room` is the number to sweep.

## 3. Why a works that closes lives 10.8 years against the port's 13.1 (C140)

**The question.** The median life of a works that closed was 13.1 years under the port with six families and 10.8 at the merge of kept town ground (11.7 on sixteen islands today), and `works.md` named no cause.

**Measured.** `trace_works --only life` on seeds 1 to 8, under the port and under the game. The chance a works closes in a year, by its age, counting every works that stood at any time from 1750:

| age | port | game | game, `works.built_share=0` |
|---|---|---|---|
| 0 to 2 | 0 | 0 | 0 |
| 3 to 5 | 0.009 | 0.048 | 0.001 |
| 6 to 9 | 0.037 | 0.050 | 0.024 |
| 10 to 14 | 0.023 | 0.036 | 0.028 |
| 15 to 24 | 0.014 | 0.022 | 0.028 |
| 25 to 49 | 0.008 | 0.017 | 0.027 |
| 50 and over | 0.003 | 0.009 | 0.015 |

And why the review closed them (`firms::review`, read from what the state keeps of each):

| | port | game |
|---|---|---|
| closed a run | 5,473 | 1,475 |
| three idle losing years: how many, median life | 2,694, 17 years | 928, 10 years |
| in debt past its limit: how many, median life | 2,766, 11 years | 547, 13 years |
| founded in the warm-up, of those closed | 9 per cent (median life 21) | 14 per cent (28) |
| founded 1750 or later: median life | 13 | 10 |

**The answer.** A works under the port is counted as building while anything at all is on order, and an order is delivered a quarter of what is left each month, so it is never quite done; for five to seven years the works cannot be counted idle and cannot shrink to nothing. The life cycle's first part (`works.built_share`, 0.05) ends that: a works is out of building within its first year, so the three-losing-years rule can close it at three. The game's works close five times as often as the port's at ages 3 to 5. The rule is the cause: give the game the port's shelter back (`works.built_share=0`) and the chance of closing at 3 to 5 falls to 0.001, the median life of the closed rises to 21.5 years, and the losing closures' median to 24 (eight islands; the swarm of small works returns with it, 1,772 in 1900, which is why the part exists). In the other direction a shorter shelter gives shorter lives, 11.7, 11.1 and 10.9 years at 0.05, 0.25 and 0.5. The rest of the gap is that the game's works close more at every age (a works that cannot grow because its trade's room is taken, and the harder price swings), and both rule sets are cut short in the same way at the end of the run: works founded in the 1880s that closed can have lived at most twenty years (median 7 in the game, 8 under the port).

**How sure.** Sure: the shelter moves the median from 11 to 21 years in a monotone line, and the ages at which the port and the game differ most are exactly the ages it covers. The median of closed works is a poor fact all the same: it mixes the warm-up's old works with the young, and its answer depends on when in the run the works were founded. The chance of closing by age is the fact to keep.

**For the pass.** Nothing to mend: the shorter life is the fringe of small works being let go, which is what the life cycle was built to do. The sweep should report closures by age (the table above) and not `works_closed_median_life`.

## 4. The wage's worst year (C134), and what it is made of

**The question.** `real_wage_worst_year`, the largest one-year fall of the real wage from 1760, went from 15 to 16 per cent down at the merge of kept town ground, and nobody looked at why. And what is the worst year made of: the building cycle, a harvest, a mine's wild year, or one town's noise?

**Measured.** `trace_works --only worst` on seeds 1 to 16: for each island its worst year, the money wage and the cost of living, each good's part in the cost of living's rise (its weight in the 1750 basket times its change in price), the harvest that year and the year before, each town's part in the wage's change, and who bought the good that did most.

**The answer.** Fourteen of the sixteen worst years fall between 1874 and 1899. In all fourteen the money wage moves by less than 3 per cent and the cost of living rises by 14 to 24 per cent, and timber is 0.45 to 0.83 of that rise (0.57 on average over all sixteen), wool 0.15, grain 0.14 and tools 0.11. No town is behind it: the wage barely moves, and no town's share of its small change says anything. The harvest that year is ordinary or a little poor (0.87 to 1.01) and the year before it was good (1.00 to 1.19).

Who bought the timber, seed 4 for example:

| | 1893 | 1894 | 1895 (the worst year) |
|---|---|---|---|
| timber made on the island | 308,358 | 305,450 | 304,131 |
| bought by works being built | 261,894 | 78,153 | 317,769 |
| bought by households | 53,220 | 230,288 | 33,544 |
| merchants' stock at the year's end | 115,314 | 106,759 | 58,200 |
| price | 2.60 | 1.95 | 3.23 |

In every timber year of the fourteen, works being built buy 260,000 to 370,000 units, about all the timber the island makes that year, where the year before they bought 44,000 to 250,000; households buy 12,000 to 49,000 of it, where the year before they bought 124,000 to 311,000. This is the building cycle of `docs/rules/works.md`: the mills of a town take a year's room for new capacity at once, and a mill is built of timber. Why it is the worst year only after 1870 is the woods: timber made per head in the 1890s is half what it was in the 1840s, while the works being built have grown with the island, so by the 1870s one wave asks for the whole year's timber.

Switched (sixteen islands, the mean of each island's worst year):

| | worst year | when |
|---|---|---|
| today's game | 15.3% down | 14 of 16 after 1870, timber +39 to +94% that year |
| no harvests (`harvest.enabled=false`) | 11.8% down | 15 of 16 still timber, grain within 7 per cent |
| woods that regrow twice as fast (`forest_regrow=0.24`) | 13.0% down | earlier (8 of 16 before 1880); grain and tools lead where timber does not |
| a works grows by half as much at a time (a trial, not committed) | 14.4% down | as today |

The trial was one line in `firms::review`: a works that earns the hurdle asks for at most a quarter of its capacity, a tenth for each hurdle it earns, where today it asks for half and a fifth. It was built, swept on sixteen islands and taken out again. It took a tenth off the yearly swing of the real wage (0.052 for 0.056) and 0.9 points off the worst year; the room is still taken whole, now by several works at once. So the worst year is the building wave first and the harvest second, and neither alone makes it.

**C134.** At merge `e` (before kept ground) and merge `f` (after) every island's worst year fell between 1872 and 1899, with timber up 43 to 105 per cent in it. The mean went from 14.6 to 16.3 per cent down because six islands of eight landed on a different and deeper wave; at merge `l` (today) the same eight read 14.3. Kept ground on against off today, on sixteen islands: 15.3 and 15.0, deeper on seven islands and shallower on nine. The worst of 140 yearly changes is a noisy fact, and C134 was its noise.

**How sure.** Sure of what the worst year is made of (sixteen islands, the same story in fourteen, and two switches that each take off the part they should). Not sure of the split between building and harvest: the two coincide, the harvest being middling after a good one is when the year's room is full, and I did not trace why.

**For the pass.** The building cycle is still the largest swing in the game; its size is set by how a works asks to grow (`firms::review`, the port's half and fifth) and the year's room taken whole (`works.year_room`, `works.room_share`), and its target is timber, whose supply shrinks all century (`forest_regrow`, `forest_capacity`). Halving the step alone is not enough. A fact for the worst year that is not a single extreme (the mean of each island's five worst, say) would stop the next merge from reporting noise.

## What it means for the pass over all the rules

- **`machines.quality_cap` ends the century's growth.** It is the only source of growth left on the land after about 1870, and the land is where the island's productivity came from in the middle of the century. Whatever is to grow after 1880, or after 1900 on the way to 2050, needs a curve that goes on.
- **`gdp_pc` deflated by the 1750 basket turns a stall into a fall**, and moves the peak year about a decade early. Deflate output per head by the chained index, or report output at fixed prices, before reading `output_per_head_peak_year` again.
- **Money is not neutral** here: a point more of new money a year makes the island 7 or 8 per cent richer by the 1850s (a fifth, from none to one) and gets it to the ceiling sooner. Mending K39 will move real numbers, not only prices.
- **R102 can close**; if the 3 per cent fewer works matters, sweep `towns.works_room`.
- **C140 is the life cycle working as designed**; swap `works_closed_median_life` for closures by age.
- **The worst year is the building cycle meeting a shrinking timber supply.** Look at how a works asks to grow and at the woods together; replace the single worst year with something steadier before ranking rules by it.

## Commands

All from the repository's root, `cargo build --release -p isostate-core --examples` and `cargo build --release -p isostate-core` first. `DIR` is a folder of one's own; the sweeps here took 1 to 7 minutes each on a busy machine.

```bash
# the baseline, sixteen islands
./target/release/isostate-core sweep --seeds 1-16 --rules game --families 6 --out DIR/base16 --quiet
# (a)
./target/release/examples/trace_works 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
./target/release/examples/trace_works --only growth --set '{"machines":{"quality_cap":8}}' 1 2 3 4 5 6 7 8
./target/release/examples/trace_works --only growth --set '{"people":{"growth_top":0.008}}' 1 2 3 4 5 6 7 8
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --set learn_cap=4,8 --set machines.quality_cap=4,8 --out DIR/a-tech --quiet
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --set people.growth_top=0.008,0.012,0.016 --out DIR/a-people --quiet
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --set machines.quality_cap=8 --set people.growth_top=0.008,0.016 --out DIR/a-people-q8 --quiet
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --set money_growth=0,0.01,0.02,0.03 --out DIR/a-money --quiet
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --set hurdle=0.08,0.12 --set works.year_room=0.15,0.25 --out DIR/a-invest --quiet
# (b)
./target/release/isostate-core sweep --seeds 1-16 --rules game --families 6 --set towns.enabled=false --out DIR/b-off --quiet
./target/release/isostate-core sweep --seeds 1-16 --rules game --families 6 --set towns.donor_wage=false,true --set towns.rent_floor=false,true --out DIR/b-parts --quiet
# (c)
./target/release/examples/trace_works --rules port --families 6 --only life 1 2 3 4 5 6 7 8
./target/release/examples/trace_works --only life --set '{"works":{"built_share":0.0}}' 1 2 3 4 5 6 7 8
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --set works.built_share=0,0.01 --out DIR/c-built-low --quiet
./target/release/isostate-core sweep --seeds 1-8 --rules game --families 6 --set works.built_share=0.05,0.25,0.5 --out DIR/c-built --quiet
# (d)
./target/release/examples/trace_works --only worst 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
./target/release/isostate-core sweep --seeds 1-16 --rules game --families 6 --set harvest.enabled=false --out DIR/d-noharvest --quiet
./target/release/isostate-core sweep --seeds 1-16 --rules game --families 6 --set forest_regrow=0.24 --out DIR/d-forest --quiet
```

The decade means of the switches are each island's decade mean of a series in `annual.csv` over its own 1750s, averaged over islands; "spending per head over the chained cost of living" is `gdp_nom / pop / cpi_chained` from the same file.

## Not done

- The land was not switched on its own: there is no single number for the land's productivity under the game's rules. The people test, with the machines' ceiling lifted, bounds what the land can be doing to a few per cent.
- `towns.works_room` was not swept for the 3 per cent fewer works.
- The trial of smaller growth steps is the only mend tried for the worst year. Asking for orders more slowly was tried by `works.md` and not again here.
- Why an ordinary harvest after a good one comes with a building wave was not traced.
- The fixed-price measure weights the goods by their own prices of 1860 to 1879; another span of years would give another index, though every one I looked at (the chained index too) says the same: a stall, not a fall.
