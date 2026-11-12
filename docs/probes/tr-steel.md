# tr-steel: the steel rail that hardly arrives, the converter's share, and when inventions come

A trace with the books of account (piece 12 of `docs/backlog/found-along-the-way.md`), on branch `night/o-trace-steel`, cut from `main` at a6ff0ec on 12 November 2026. Three questions, behind findings R58, C17 and C129 (the steel rail does not win), R57 and C15 (the converter's numbers), R61 (a steel rail mill in 1815), R167 (discoveries cluster in the largest industrial town) and R16, R89, C89 and C135 (the railway age wanders):

- **(a)** The steel rail hardly arrives. What would a steel rail mill earn, what does it lack, and what would make it pay?
- **(b)** Converter steel is 0.86 of steel in 1880 on average, but nothing on some islands and all of it on others. What decides which?
- **(c)** The first railway opens anywhere from 1809 to 1842. When does each invention come, what sets the date, and what would narrow the spread without fixing the dates by hand?

Everything is the game's rules with six families, on seeds 1 to 16 unless it says 1 to 8. No rule and no fact was changed: nothing under `crates/isostate-core/src` is different on the branch. The measuring program is `crates/isostate-core/examples/steel_trace.rs`. It steps each island a year at a time and, after each year's end, reads the books (`docs/rules/accounts.md`), each invention's progress towards its threshold, and the sums entry would make (`firms::evaluate_sites`) for a new steel rail mill in every town that knows how, beside an iron rolling mill of the same size in the town where the steel mill would do best. It draws none of the run's random numbers and changes nothing in the state, so its runs are the game's own: its discovery years on seeds 1 to 8 are those of the merged sweep `merged-game-f6-l`. It can change a method's numbers or a threshold in its own copy of the content, which is how the trials of (a) and (b) were made. The trials of (c) needed two lines in `tech.rs` that were tried in the worktree and not committed (section 3). The program's output, the sweeps and a file of notes with every command and number are under `.claude/night/reports/opus/trace-steel/`.

## In short

1. **The steel rail mill lacks skill, not cheap steel.** In the town where it would do best, its steel already costs less a rail than an iron rolling mill's iron (in 1880, 2.02 against 2.51) and its plant a third as much. But its hands cost twice as much a rail (5.97 against 2.81), because its skill is 2.0 where the rolling mill's is 4.0. Two is the floor `machines.inherit` gives it, half the rolling mill's skill, and it never makes enough rails of its own to climb its own curve past the floor: up to 222 thousand rails in a century on an island, where its curve needs 108 thousand to reach 2 and 1.1 million to reach 4. So 48 to 176 steel rail mills are founded on an island; each runs flat out for two or three years when rails are dear, stands idle when they are not, and is closed once its losses mount. With the floor at the whole of the rolling mill's skill (`machines.inherit = 1`) steel rails are 0.93 of rails in 1880; with the steel rail mill alone given that skill and nothing else changed, 0.91 (section 6); with half the hands a rail, 0.88; with a curve that learns ten times faster, 0.31 and rising; with the mill's usual size halved, 0.13; with a steel rail worth six iron rails, 0.20. That is also why cheaper ore made it worse, as the check measured with the mines switched off (section 6): three fifths of the steel mill's costs are wages and a third of the iron mill's, so cheaper iron brings the price of rails down further than it brings down the steel mill's costs, and in the best town in 1880 its margin goes from 1.66 a rail without the mines to -0.44 with them, at the same skill of 2.
2. **The converter's share in 1880 is set by when the converter is found, and that by crucible steel.** Steel holds the converter's threshold back on all sixteen islands. Where crucible steel grows from its find in the 1760s the threshold of 2.8 million is reached between 1835 and 1844. On two islands (seeds 3 and 9) the first crucible works sold at a loss for 55 and 25 years and made almost nothing (15 and 4,748 a year in 1800 to 1830, against 50 to 67 thousand elsewhere), the same trap as the steel rail mill's, and the threshold came in 1884 and 1863. Those two are the islands where the draw found crucible steel late (1784 and 1773, where the rest found it by 1769), and the check showed that a late find is what makes the stall: with crucible steel's thresholds doubled, four of the five islands that find it after 1778 stall and none of the nine that find it by 1772 does (section 6). On seed 3 that is too late for 1880: the converter's share is nought. On seed 2 the threshold was passed in 1841 and the yearly draw missed for 27 years, so the share is 0.85. So the converter's share is the draw's twice over. Give crucible steel a first skill of 0.7 for 0.5, or a steeper and quicker draw (point 3), and the converter makes all the steel by 1880 on every island.
3. **An invention's date is mostly the draw's.** Across sixteen islands the year an invention is found has a standard deviation of 5.5 to 16 years; the year its thresholds are reached, 0.6 to 12. The yearly chance, `0.18 p^4 / (1 + p^4)` at progress p, is shallow on both sides of the threshold: an invention has a fifth to three fifths chance of coming before it (the steel rail mill of 1816 at progress 0.39 on seed 6, R61), and after it the chance is 9 to 18 per cent a year, so waits of ten to thirty years happen. The first railway opens 8.3 years after the locomotive is found (standard deviation 2.4), so its date is the invention's. A steeper and quicker chance, `0.5 p^12 / (1 + p^12)`, keeps the mean dates within a few years of where they are (every one of the ten comes sooner, by one to five years) and cuts the spread to a third: the first line opens in 1825.8 with a standard deviation of 3.4 years (1820 to 1832), where today it is 1827.1 with 6.8 (1809 to 1842).
4. **Discoveries do cluster, by the rule's own square.** The town that finds an invention is drawn in proportion to the square of its share of the ingredients, so the town that handles most is drawn about half the time. On an island four towns (three to six) find the ten inventions, the busiest finds five (two to seven), and 42 per cent of finds are in the town with the most hands in works. Drawn in proportion to the share itself, six towns find them and the busiest finds four.

## 1. The steel rail mill (a)

### What was measured, and how

```bash
cargo build --release -p isostate-core --example steel_trace
./target/release/examples/steel_trace.exe --seeds 1-16 --works --csv OUT/a0-base > OUT/a0-base.txt
```

`OUT` is `.claude/night/reports/opus/trace-steel`. Section 2 of the output is the books' table of methods (`METHOD_FIELDS`) summed over every town of the sixteen islands, year by year; section 3 the site sums; section 4, with `--works`, each island's busiest steel rail mill's own books (`accounts::works_books`). `years.csv` has every island and year.

### What the books say

Steel rails' share of the rails made, all sixteen islands pooled, and what the standing mills did:

| | 1850 | 1860 | 1870 | 1880 | 1890 | 1900 |
|---|---|---|---|---|---|---|
| steel rails, share of rails | 0.006 | 0.064 | 0.072 | 0.047 | 0.036 | 0.036 |
| steel rail mills: output over capacity | 0.33 | 0.31 | 0.29 | 0.17 | 0.18 | 0.19 |
| steel rail mills: profit over the value of the plant | -0.00 | 0.16 | 0.05 | 0.02 | 0.03 | 0.07 |
| iron rolling mills: output over capacity | 0.79 | 0.86 | 0.86 | 0.82 | 0.84 | 0.81 |
| iron rolling mills: profit over the value of the plant | 0.08 | 0.17 | 0.13 | 0.07 | 0.08 | 0.09 |
| converter's share of steel | 0.53 | 0.84 | 0.95 | 0.97 | 0.98 | 1.00 |
| price of steel, of iron (island means) | 13.4, 5.1 | 11.3, 5.4 | 8.5, 4.5 | 7.2, 4.0 | 7.1, 4.2 | 7.4, 4.6 |

No island has a year in which steel rails are half of all rails; the most any island reaches in a year is 0.43. From 1860 to 1900 a standing steel rail mill takes 0.58 worker-months a rail and an iron rolling mill 0.26; wages are 0.59 of the value of a steel mill's rails and 0.33 of an iron mill's, materials 0.19 and 0.30.

A steel rail mill's life is a boom and a bust. On the fifteen islands that find it, 48 to 176 are founded in a run. The busiest of seed 5, a works at Whitwater founded in 1853, ran at 0.80 to 0.96 of capacity in 1855, 1861 to 1863 and 1871, made 1,736 rails in 1863 and earned 0.37 on its plant that year, stood nearly idle in 1857 to 1859, 1865 to 1867 and 1873, was below nothing in 13 of its 21 years, and closed in 1874. Every island's busiest mill tells the same story (section 4 of `a0-base.txt`): the steel rail mill is the marginal maker of rails, working when rails are dear and stopping when they are not, because its margin is thin and a works' output answers to its margin.

### What a new mill would earn, and what it lacks

The sums entry makes for a new steel rail mill in the town where it would do best, at the size entry would give it (a tenth of the town's trade in rails), and for an iron rolling mill of the same size in the same town. Means over the islands that know the steel rail mill; costs are per rail. The program scores every cell of a town and keeps the best, where entry scores a handful drawn at random and the eight nearest the material, so these are the best entry could hope for. The last row counts towns, summed over the islands, in that one year.

| | 1860 | 1870 | 1880 | 1890 |
|---|---|---|---|---|
| size a month | 33 | 32 | 33 | 36 |
| skill: steel mill, iron mill | 2.00, 3.99 | 2.00, 4.00 | 2.00, 4.00 | 2.02, 4.00 |
| small works' penalty, sqrt(usual size / size): steel, iron | 1.56, 1.1 | 1.58, 1.1 | 1.57, 1.1 | 1.54, 1.1 |
| material a rail: steel mill (steel x price) | 0.33 x 9.98 = 3.32 | 0.31 x 7.61 = 2.39 | 0.31 x 6.45 = 2.02 | 0.31 x 6.26 = 1.95 |
| material a rail: iron mill (iron x price) | 0.73 x 4.93 = 3.62 | 0.74 x 3.87 = 2.86 | 0.74 x 3.42 = 2.51 | 0.73 x 3.66 = 2.69 |
| hands a rail: steel, iron | 6.26, 2.49 | 5.34, 2.53 | 5.97, 2.81 | 6.17, 2.94 |
| margin a rail: steel, iron | 3.99, 7.47 | 2.31, 4.67 | -0.44, 2.23 | 0.31, 2.81 |
| plant for a rail a month: steel, iron | 97, 256 | 89, 236 | 89, 236 | 94, 250 |
| return a year: steel, iron (the hurdle is 0.12) | 0.39, 0.29 | 0.22, 0.19 | -0.06, 0.09 | 0.04, 0.12 |
| towns where a steel mill clears the hurdle | 67 of 253 | 66 of 255 | 22 of 257 | 30 of 258 |

So in a good year for rails (1860, 1870) a steel rail mill clears the hurdle by more than an iron one, because its plant is cheap; in an ordinary year it does not, because its hands cost twice as much. The factor of two is, in 1880, the product of three things in `econ::firm_coefficients`: the skill (2.0 against 4.0, a factor of 2.0), the small works' penalty (`1 + 0.8 r`, 2.26 against 1.88, a factor of 1.2) and the content's hands a rail (0.7 against 0.8, a factor of 0.875). In 1860 there is a fourth: the best town's know-how of the steel rail mill is 0.79 against the rolling mill's 1.0, and its hands cost 2.5 times the iron mill's.

The skill is the floor `machines::inherit_skill` gives an heir: `inherit` (0.5) of the rolling mill's, which is at `learn_cap` (4) from the 1840s. The steel rail mill's own curve is `0.6 (1 + Q/2000)^0.3` on the rails it has made; it passes 2 at 108 thousand rails and reaches 4 at 1.1 million. In a whole run the steel rail mills of an island make 0 to 222 thousand rails (the middling island 24 thousand), so on all but three islands the floor is all the skill there is, for ever. It is a trap: a method too clumsy to pay makes little, and making is the only way it learns.

### What would make it pay

Each trial changes one number in the program's copy of the content or the parameters and runs sixteen islands again (`--quiet --csv`). Steel rails' share of rails, pooled:

| trial | command | 1860 | 1870 | 1880 | 1890 | 1900 | islands with a year over half | rails' price 1890 |
|---|---|---|---|---|---|---|---|---|
| the game as it is | | 0.06 | 0.07 | 0.05 | 0.04 | 0.04 | 0 | 7.97 |
| the heir starts at the whole of the rolling mill's skill | `--set '{"machines":{"inherit":1.0}}'` | 0.54 | 0.85 | 0.93 | 0.95 | 0.95 | 15 | 5.31 |
| the steel rail mill alone at that skill (the check's) | `--method steel_rolling.A0=4` | 0.50 | 0.80 | 0.91 | 0.95 | 0.95 | 15 | 5.65 |
| the mines switched off (the check's) | `--set '{"mines":{"enabled":false}}'` | 0.07 | 0.14 | 0.13 | 0.16 | 0.18 | 2 | 10.21 |
| half the hands a rail | `--method steel_rolling.labor=0.35` | 0.42 | 0.69 | 0.88 | 0.95 | 0.95 | 15 | 4.65 |
| its own curve learns ten times faster | `--method steel_rolling.Q0=200` | 0.07 | 0.14 | 0.31 | 0.48 | 0.66 | 11 | 6.73 |
| a steel rail stands for six iron rails, not three | `--worth 6` | 0.14 | 0.20 | 0.20 | 0.27 | 0.35 | 4 | 8.08 |
| the usual size of an iron rolling mill (40 for 80) | `--method steel_rolling.K0=40` | 0.07 | 0.10 | 0.13 | 0.17 | 0.24 | 3 | 7.80 |
| crucible steel learns from a higher start (section 2) | `--method crucible_steel.A0=0.7` | 0.12 | 0.12 | 0.11 | 0.09 | 0.09 | 1 | 8.14 |

(The sixteenth island in the first three rows is seed 3, which finds the mill in 1891.) The skill is the cause: the second row moves nothing but the steel rail mill's own skill and does what `inherit = 1` does. Give the steel rail mill its master's skill and the steel rail takes the trade in ten to twenty years and pulls the price of rails down a third, as the steel rail did. With half the hands, or a curve it can climb, the mill's own skill grows past the floor once it is making (to 3.5 by 1900 in both), which is the trap broken from outside. Cheaper steel a rail (the `MACHINES_STEEL_RAIL_WORTH` that `machines.md` and `mines.md` ask to be tuned) helps a little, since material is a fifth of the mill's costs. The mines switched off help a little too, at the same skill of 2, because they leave iron, and so the price of rails, dearer (last row; section 6).

`machines.inherit = 1` on seeds 1 to 8 against `merged-game-f6-l` (`isostate-core sweep --seeds 1-8 --rules game --families 6 --set machines.inherit=1.0 --compare ...`): of 446 facts 275 differ and none is starred. Steel rails are 0.87 of rails in 1890 (0.002), up on seven islands and the same on seed 3. Output per head in 1850 is 2.20 times 1750's against 2.12 (seven up, none down), the 1900 wage 2.64 against 2.60 (5/3), railway in 1900 2,308 km against 2,248 (7/1), works in 1900 705 against 738 (all eight down). The converter, which inherits from crucible steel under the same number, moves little (0.875 of steel in 1880 against 0.857).

### How sure, and what would change my mind

Sure of the cause: the books, the site sums and the trials agree, on sixteen islands, and the trial that changes only the floor takes the share from 0.05 to 0.93. Less sure of the sizes in the trial table, which are single runs of each setting. What would change my mind: a run in which the steel mill's skill is at the floor and it still takes the trade (none of the 112 runs here), or one in which its skill is at 4 and it does not (none of the check's 16 at `A0 = 4`, nor of its two islands more, seeds 17 and 18, at `inherit = 1`). `docs/probes/sens-machines.md` found the same number from the other side: steel rails a sliver at `inherit` 0.5, 17 and 36 per cent of the rails of 1890 at 0.55 and 0.6, three fifths at 0.667.

### For the pass over all the rules

The number to look at is `machines.inherit`, and the thing to decide is what an heir should start from. Half its master's skill is a floor the steel rail mill can never leave. (Before the works' life cycle the steel rail was nearly all of the rails in 1890, `machines.md`; why the many small works of then got it up its curve and the few large ones of now do not, I did not trace.) Three ways that do not fix a date: inheritance of the whole for a method that makes the same good from a better material (`inherit` 1, or a number of its own for the steel rail mill so that the converter keeps 0.5); an heir's curve that starts from where the floor stands, so that what it makes counts from there (in `inherit_skill`, take `Q` such that the heir's own curve gives the floor, and add its output to that); or simply a quicker curve (`Q0` 200). The first is the least change. The steel rail's worth taken at the mill (R59) is a separate fault and still stands; with the floor mended the steel rail takes the whole trade, so a steel rail worth three iron ones for new track as well as relaying will then matter more.

## 2. The converter's share (b)

### What was measured, and how

The same runs (`a0-base`), read with a short script over `years.csv` and `disc.csv` (in the notes): for each island the year crucible steel was found and the year its threshold was reached, the same for the converter, crucible steel made a year in 1800 to 1830, and the converter's share of steel in 1870, 1880 and 1890. Then one trial:

```bash
./target/release/examples/steel_trace.exe --seeds 1-16 --quiet --method crucible_steel.A0=0.7 --csv OUT/b1-cru07 > OUT/b1-cru07.txt
```

### What decides

| seed | crucible steel found (threshold reached) | its output a year, 1800-30 | converter found (threshold reached; progress then) | converter's share 1870, 1880 |
|---|---|---|---|---|
| 1 | 1762 (1762) | 62,632 | 1844 (1837; 1.24) | 1.00, 1.00 |
| 2 | 1765 (1764) | 54,186 | 1868 (1841; 2.18) | 0.03, 0.85 |
| 3 | 1784 (1764) | 15 | 1886 (1884; 1.15) | 0, 0 |
| 9 | 1773 (1760) | 4,748 | 1859 (1863; 0.85) | 0.75, 1.00 |
| the other twelve | 1759 to 1769 | 50,067 to 66,928 | 1822 to 1851 (1835 to 1844) | 1.00, 1.00 |

The converter's threshold is held back by steel on every island (2.8 million made since 1750). Across the sixteen islands the year it is reached goes with crucible steel made in 1800 to 1830 (correlation -0.96). On fourteen islands crucible steel is found in the 1760s and grows every year; on seed 7, found in 1765, it was making money by 1773 and 4,382 a year by 1790. On seeds 3 and 9 the first crucible works opened, sold their first steel and drove its price from 640 or 1,200 to about 120, and then lost money every year at a capacity of 1 to 17 for 55 years (seed 3, to about 1838) and 25 years (seed 9, to about 1808). That is the steel rail mill's trap again: crucible steel's curve starts at 0.5 and climbs only with its own output, and a works that earns a little less than it costs makes nothing to learn from. What put these two in the trap is the date of the find, 1784 and 1773 where the other fourteen found it by 1769. (The first draft read it as the wage at the find, 5.6 and 5.2 against 4.8 on seed 7; but seed 6 found crucible steel at an island wage of 6.1, the highest of the sixteen, and made the most of any. The check's trial with crucible steel found late everywhere is in section 6.) On seed 3 the threshold came in 1884, the converter in 1886, and none of the steel of 1880 is the converter's. Seed 9's came in 1863, in time to make all the steel by 1880.

Seed 2 is the other way to be late. Its threshold was passed in 1841 and the converter was found in 1868 at progress 2.18: 27 years in which the yearly chance was 9 to 17 per cent and missed, which happens about once in fifty.

With crucible steel's first skill at 0.7 (`b1-cru07`), seeds 3 and 9 make 20,707 and 48,384 a year in 1800 to 1830, the converter's threshold is reached in 1839 (1831 to 1853, standard deviation 4.7 years against 11.7), and the converter makes all the steel in 1880 on all sixteen islands. Seed 2's draw then falls in 1845 rather than 1868, because the same sequence of draws meets a different year of crossing.

### How sure, and what would change my mind

Sure: the islands where the share is short are exactly those where crucible steel stalled or the draw missed, and taking away the stall takes away the shortfall. That a late find makes the stall was shown by the check (section 6); why a late find stalls was not traced, though cheap iron by the 1780s is the likeliest reading. If an island with healthy crucible steel and a timely draw still had little converter steel in 1880, this would be wrong.

### For the pass over all the rules

The converter's own numbers (`MACHINES_PLANT_SCALE`, `inherit`) are not what decides its share: once found it takes the trade in ten to fifteen years everywhere. What decides is the date, and the two things that move it are crucible steel's start on its curve (`A0` 0.5, `Q0` 50 in `data/content.toml`) and the draw (section 3), which acts twice: on the converter's own find, and on crucible steel's, since a crucible steel found after about 1778 mostly stalls. The steeper, quicker draw of section 3 finds crucible steel by 1770 on every island and the converter makes all the steel of 1880 on all sixteen; so does `A0` 0.7. A learned method that can stall for half a century at its first skill is worth a check across all the learned methods; crucible steel is the one that showed it here, and `sens-machines.md` found seed 3's crucible steel stalling in 8 of the 30 settings that changed that island's history at all.

## 3. When inventions come (c)

### What was measured, and how

The same runs. For each invention and island the program keeps its progress each year (the least, over its ingredients, of cumulative output over the threshold, as `tech::discovery` reckons it), the year progress first reached a half and the whole, the year found and the progress then, the ingredient that held it back, the sum of the yearly chances before the threshold (the odds of a fluke), and, in the year found, the drawn town's share of the draw's weight and whether it was the town weighted most, the one with the most hands in works or the most people. The railway's first line is from the sweep:

```bash
./target/release/isostate-core.exe sweep --seeds 1-16 --rules game --families 6 --out OUT/sw-base16 --quiet
```

The trials need the hazard's power, which is 4 in `tech.rs` and not a parameter. For them two lines in `tech::discovery` read it from the environment (`ISOSTATE_DISC_POW`), and the town's weight unsquared (`ISOSTATE_DISC_LINEAR`); with neither set the code did what it does. They were not committed, and the tests and the standard sweep below were run after they were taken out.

```bash
./target/release/isostate-core.exe sweep --seeds 1-16 --rules game --families 6 --set discovery_rate=0.5,1.0 --out OUT/sw-rate --quiet
ISOSTATE_DISC_POW=12 ./target/release/isostate-core.exe sweep --seeds 1-16 --rules game --families 6 --set discovery_rate=0.18,0.5,1.0 --out OUT/sw-pow12 --quiet
ISOSTATE_DISC_POW=12 ./target/release/examples/steel_trace.exe --seeds 1-16 --set '{"discovery_rate":1.0}' --csv OUT/c4-pow12-rate1 > OUT/c4-pow12-rate1.txt
ISOSTATE_DISC_LINEAR=1 ./target/release/examples/steel_trace.exe --seeds 1-16 --quiet --csv OUT/c5-linear > OUT/c5-linear.txt
```

### When each invention comes, and why then

Sixteen islands, the game as it is. "Threshold" is the year progress first reached 1.

| invention | found: mean [least .. greatest], sd | threshold: mean, sd | found less threshold: sd | finds at progress under 0.75 | held back by |
|---|---|---|---|---|---|
| crucible steel | 1767 [1759 .. 1784], 5.5 | 1763, 1.5 | 5.7 | 1 | coal 13, iron 3 |
| coke furnace | 1774 [1762 .. 1788], 6.3 | 1772, 2.0 | 5.9 | 1 | coal 15, iron 1 |
| improved husbandry | 1774 [1761 .. 1805], 10.4 | 1772, 0.6 | 10.3 | 5 | grain |
| engine works | 1783 [1774 .. 1797], 6.5 | 1779, 2.2 | 6.6 | 3 | machinery |
| powered mill | 1790 [1766 .. 1826], 14.7 | 1790, 7.9 | 12.0 | 4 | cloth |
| rolling mill | 1792 [1773 .. 1818], 11.0 | 1791, 2.6 | 11.3 | 4 | rails 14, machinery 2 |
| railway | 1819 [1793 .. 1835], 8.6 | 1820, 5.3 | 6.6 | 3 | machinery 11, rails 5 |
| farm machinery | 1838 [1827 .. 1850], 6.5 | 1837, 5.6 | 5.5 | 3 | machinery |
| converter | 1844 [1822 .. 1886], 16.2 | 1845, 11.7 | 10.7 | 4 | steel |
| steel rail mill | 1846 [1816 .. 1891], 14.9 | 1850, 11.2 | 7.6 | 2 | steel 15, machinery 1 |

For the early inventions the threshold is reached within a year or two of the same date on every island (standard deviations of 0.6 to 2.6), and nearly all the spread in the year found is the draw's. For the late ones the threshold's own date spreads too, partly because it is fed by earlier draws (the converter's waits on crucible steel, section 2). The yearly chance is 0.18 p^4 / (1 + p^4): 0.09 at the threshold, 0.035 at progress 0.7, rising to 0.18 well past it. Compounded over the years before the threshold, the chance of a find before it is, on the mean island, 0.21 (crucible steel) to 0.61 (the powered mill, whose progress creeps up for twenty years) for an invention (the check's figure, section 6; the first draft's 0.20 to 0.47 stopped counting at the find and so left out the islands that had their fluke), so a find before the threshold is common (1 to 5 islands of 16 for each invention find it at progress under 0.75; the steel rail mill of seed 6 in 1816 at 0.39 is R61's fluke), and after it the mean wait is five to eleven years with a long tail (seed 2's converter, 27 years; seed 9's improved husbandry, 33 years at progress 3.0).

The first railway follows the find: it opens 8.3 years later on average (6 to 16, standard deviation 2.4; correlation of the two dates 0.98), the line's two years of building and the promoters' money. So the railway age's date is the locomotive's, and that is set by machinery made (60,000 since 1750, on 11 islands of 16) or rails made (150,000, on 5), then the draw.

### What narrows it

| setting (16 islands each) | found: mean of the ten inventions' sd | railway found | first line | km of line in 1850 | converter's share 1880 | wage 1900 over 1750 |
|---|---|---|---|---|---|---|
| the game: 0.18 p^4 / (1 + p^4) | 10.1 | 1818.8, sd 8.6 | 1827.1, sd 6.8 [1809 .. 1842] | 1,348 [575 .. 1,804] | 0.93 [0 .. 1] | 2.61 |
| `discovery_rate` 0.5 | 6.4 | 1809.3, sd 8.4 | 1818.1, sd 8.7 | 1,600 [1,270 .. 1,827] | 1.00 | 2.70 |
| `discovery_rate` 1.0 | 5.9 | 1802.3, sd 5.8 | 1810.6, sd 6.2 | 1,718 [1,554 .. 1,852] | 1.00 | 2.64 |
| power 12, rate 0.18 | 8.8 | 1826.8, sd 8.7 | 1834.7, sd 9.4 | 1,015 [0 .. 1,628] | 0.88 [0 .. 1] | 2.65 |
| **power 12, rate 0.5** | **3.7** | **1818.0, sd 3.4** | **1825.8, sd 3.4 [1820 .. 1832]** | **1,459 [1,185 .. 1,718]** | **1.00** | **2.72** |
| power 12, rate 1.0 | 3.4 | 1815.1, sd 3.7 | 1823.4, sd 4.0 | 1,478 [1,246 .. 1,629] | 1.00 | 2.72 |

A quicker chance alone shortens the wait after the threshold and makes flukes before it likelier, so everything comes five to sixteen years sooner and the railway is no narrower. A steeper chance alone ends the flukes and keeps the slow wait, so everything comes later and one island has no line in 1850. Together they put the find within a few years of the threshold: at power 12 and rate 0.5 the chance is 0.25 a year at the threshold, 0.03 at progress 0.8 and 0.003 at 0.65, and the mean dates hardly move (the railway found in 1818.0 against 1818.8, the converter 1838.4 against 1843.9, farm machinery 1833.7 against 1837.5). What spread remains is the economy's: at power 12 and rate 1.0 the year found is within one to three years of the year the threshold is reached (standard deviation 1.3 to 3.4), and the thresholds themselves come closer together (the railway's 3.4 years against 5.3, the converter's 2.1 against 11.7) because earlier inventions no longer wander. The steel rail mill comes no sooner than 1830 at power 12 and rate 0.5, so the fluke of R61 is gone. Of the facts read here none moves much except what the dates move: the converter is all the steel by 1880 everywhere, the 1850 network is longer and more even, the 1900 wage a little higher (2.72 against 2.61, inside the spread of 0.20 between islands). The sweep's other facts were not compared one by one.

### Where it is found (R167)

The town is drawn with weight (the town's output of the ingredients last year over the thresholds, summed)^2. On sixteen islands, 160 finds:

| | the rule (squared) | drawn by the score itself (trial) |
|---|---|---|
| finds in the town weighted most | 0.47 | 0.28 |
| finds in the town with the most hands in works | 0.42 | 0.26 |
| finds in the most populous town | 0.38 | 0.26 |
| towns that find something, on an island | 4.3 [3 .. 6] | 5.7 [4 .. 8] |
| finds in the busiest such town, of ten | 5.3 [2 .. 7] | 3.8 [2 .. 7] |

So yes, discoveries cluster: half of an island's inventions are found in one town. That is the square in the rule doing what it says. Whether it is a fault is a question of taste rather than of the economy: the town found in starts with know-how 0.25 and the rest follow by diffusion along trade, so the place matters to the map and the news more than to the dates.

### How sure, and what would change my mind

Sure that the draw is most of the spread and that the steeper, quicker chance narrows it: sixteen islands at each of six settings, and the trace at power 12 shows the find sitting on the threshold. Less sure of the costs: I read a handful of facts, not the whole sweep, and the hazard's power is not a parameter, so no setting of the game as it stands reproduces the narrow row. If the designer wants islands whose railway ages differ by a generation, the shallow chance is the way to have it, and none of this is a fault.

### For the pass over all the rules

The numbers to look at are the hazard's power (4, written into `tech::discovery` and again into `canals.rs`'s discovery of the canal, which this trace did not touch) and `discovery_rate` (0.18). Making the power a parameter is a one-line change under the convention (the port's 4 under `default_params`), and 12 with a rate of 0.5 keeps the dates within one to five years of where the game has them and makes them about three times more alike. The canal's first year (1774, standard deviation 5.9) came up to four years sooner in these trials but kept its spread (standard deviation 5.3 to 6.1), because its own copy of the hazard reads `discovery_rate` but keeps the power of 4 (the check's reading of `canals.rs` line 260 and the sweeps' `canal_first_year`). The thresholds themselves (absolute numbers for the railway, multiples of 1750's output for most others) then set the dates, and the railway's two, 60,000 of machinery and 150,000 of rails, are where to look if the railway age is to come at another time.

## 4. What I am unsure of, and what was not done

- **The trials are single settings on sixteen islands.** No setting was searched, and none of the mends is proposed as it stands: `machines.inherit = 1` also governs the converter, a crucible steel `A0` of 0.7 is a guess, and the power of 12 was the first steep number tried.
- **The site sums are entry's, taken after the year's end** on the year's mean prices. Entry itself works on the same numbers in the same yearly step, so they should agree; I did not check a founded mill's recorded reason against them.
- **Why a late find stalls crucible steel** was not traced: the check showed that it does, and that the wage at the find does not separate the islands that stall (section 6).
- **The steel rail's worth taken at the mill** (R59, C22) was not traced; the trial at six iron rails is a change of that number, not of where it is credited.
- **Nothing was run under the port's rules.**

## 5. How to run it again

```bash
cargo build --release -p isostate-core --example steel_trace
./target/release/examples/steel_trace.exe --seeds 1-16 --works                          # sections 1 to 4: finds, the books, the site sums, the busiest mill
./target/release/examples/steel_trace.exe --seeds 1-16 --quiet --set '{"machines":{"inherit":1.0}}'
./target/release/examples/steel_trace.exe --seeds 1-16 --quiet --method steel_rolling.labor=0.35
./target/release/examples/steel_trace.exe --seeds 1-16 --quiet --method crucible_steel.A0=0.7
./target/release/examples/steel_trace.exe --seeds 1-16 --quiet --worth 6                # or --plant S, --disc TECH.GOOD=X
./target/release/examples/steel_trace.exe --seeds 1-16 --quiet --method steel_rolling.A0=4                            # the check's
./target/release/examples/steel_trace.exe --seeds 1-16 --quiet --set '{"mines":{"enabled":false}}'                    # the check's
./target/release/examples/steel_trace.exe --seeds 1-16 --quiet --disc crucible_steel.coal=x30 --disc crucible_steel.iron=x36   # the check's
cargo run --release -p isostate-core -- sweep --seeds 1-16 --rules game --families 6 --set discovery_rate=0.5,1.0 --out OUT
```

A run of sixteen islands takes three to ten minutes on a busy machine. `--csv DIR` writes `disc.csv` and `years.csv` for reading elsewhere.

## 6. After the check

By a second helper, on `night/o-trace-steel-check`, the same day, with nothing taken on trust: every number read again from the runs' own files, the base run made again, the measuring program read against the code it copies, and the rival explanations tried. Sixteen islands, the game's rules, six families, unless it says otherwise. Corrections are made above, in place; this section says what was done and found.

### What was run again

| | the page | run again | how |
|---|---|---|---|
| the base run's `disc.csv` and `years.csv` | `a0-base` | the same to the byte | run again on today's `main` |
| every figure of the tables in sections 1, 2 and 3 | as written | the same | read again from those files and from the sweeps' `facts.csv` |
| the railway's first line, 8.3 years after the find | sd 2.4, correlation 0.98 | 8.3, sd 2.4, 0.977 | `sw-base16` |
| power 12 and rate 0.5: each invention's mean date | "within a few years" | all ten sooner, by 0.8 (the railway) to 5.4 years (the converter) | `sw-pow12` against `sw-base16` |
| seed 5's busiest mill, 1863, by hand | profit 6,809 | revenue 19,194 less inputs 2,713, fuel 118, wages 9,276 and rent 277: 6,810 | its own row of the books |
| the chance of a find before the threshold | 0.20 to 0.47 | 0.21 to 0.61 | see below |
| seed 5's mill below nothing | 11 of 21 years | 13 | its own rows |

The program draws nothing and changes nothing in the state: discovery runs in the yearly step before anything after it touches cumulative output, and the program reads after the year's end, so the progress it reads is the draw's own, and the town weights it rebuilds use the same year's output, people and works' hands as the draw did.

### What the numbers did not say

1. **The chance of a fluke was counted short.** The program stopped adding up an invention's yearly chances in the year it was found, so on an island where the find came before the threshold, the very islands that had a fluke, the sum was cut off early. Counted over every year short of the threshold, found or not (from `years.csv`'s progress columns, and now in the program), the chance of a find before the threshold on the mean island is 0.21 (crucible steel) to 0.61 (the powered mill), with the converter 0.52, the steel rail mill 0.53 and the railway 0.46.
2. **The site sums are the best cell's.** The program scores every cell of a town where entry scores a handful drawn at random, so they are an upper bound on what entry sees. It does not change the comparison, since both mills are scored the same way.
3. **The three-loss rule** closes a works after three years that are both at a loss and nearly idle (under 0.15 of capacity), or when it cannot pay its debts. Seed 5's mill lost money six years running (1865 to 1870) without closing. The page now says the mills close "once their losses mount".
4. **The hands' factor of two** is the product of three things in 1880; in 1860 the steel rail mill's know-how in the best town is 0.79, a fourth.

### The rival explanations, tried

- **(a) Only the steel rail mill's skill.** `inherit = 1` lifts every heir, the converter among them. With nothing changed but the steel rail mill's own start at the rolling mill's skill (`--method steel_rolling.A0=4`), steel rails are 0.50 of rails in 1860, 0.80 in 1870, 0.91 in 1880 and 0.95 in 1890 and 1900, on fifteen islands of sixteen (seed 3 finds the mill in 1891): what `inherit = 1` gives. The cause holds with only the suspect moved.
- **(a) The mines.** The page said cheaper ore "very likely" made the steel rail's case worse. With the mines switched off (`--set '{"mines":{"enabled":false}}'`) steel rails are 0.13 of rails in 1880 and 0.16 in 1890, against 0.05 and 0.04, at the same skill of 2. In the best town in 1880, the mines take 2.76 off the price a rail fetches (10.38 to 7.62) and 0.45 off the steel rail mill's steel a rail (2.47 to 2.02), its hands a rail unmoved (6.16 and 5.97): its margin goes from 1.66 a rail to -0.44, the iron mill's from 4.04 to 2.23. So the reading is right, and measured: a mill whose costs are mostly hands cannot follow the price of rails down when iron gets cheap.
- **(b) The wage at crucible steel's find.** The page read the stall of seeds 3 and 9 as the wage at the find (5.6 and 5.2 against 4.8 on seed 7). Seed 6 found crucible steel in 1759 at an island wage of 6.1, the highest of the sixteen, and made more of it than any. What seeds 3 and 9 share is a late find (1784 and 1773, the rest by 1769). With crucible steel's two thresholds doubled (`--disc crucible_steel.coal=x30 --disc crucible_steel.iron=x36`), so that the draw comes later everywhere, it is found between 1759 and 1786: none of the nine islands that find it by 1772 stalls; of the five that find it after 1778, four stall (seeds 3, 4, 12 and 14 make 7 to 25 a year in 1800 to 1830 and find the converter in 1886 to 1892, too late for 1880) and the fifth starts weakly and recovers by the 1830s. So a late find makes the stall. Ten years after a late find iron costs 5 to 7, where it cost 15 to 22 ten years after most early ones: by the 1780s iron is cheap, and crucible steel's first works sell next to nothing at a price near 100 and never learn. Why cheap iron should starve crucible steel of buyers I did not trace; it would make the stall the steel rail mill's trap with the same cause as (a)'s mines.
- **The worker's power-12 sweep agrees**, read the same way: at power 12 and the game's rate, seed 9 finds crucible steel in 1768 and does not stall, seed 15 finds it in 1781 and does (the converter's share in 1880 nought), and at rate 0.5 every island finds it by 1770 and none stalls.
- **Two islands nobody had used** (seeds 17 and 18) tell the same story: the steel rail mill's skill 2.00 against 4.00 from 1865, its hands a rail 6.08 against 2.91 in 1880, steel rails under 0.06 of rails in every year sampled, two busiest mills that boom for two to five years and stand idle after; with `inherit = 1`, 0.87 of rails in 1880 and 0.98 in 1890.

### What follows for the rules, and what is opinion

Measured: the steel rail mill's skill floor decides the steel rail (two ways of moving only that, and `sens-machines.md`'s ramp in `inherit`); cheaper iron widens the gap at that floor; a late draw for crucible steel stalls it and with it the converter; the shallow hazard is most of the spread in dates, and a steeper, quicker one narrows it. Opinion, and the first draft's: which of `inherit` 1, a number of the steel rail mill's own, or a quicker curve to choose; whether 12 and 0.5 are the hazard's numbers (one setting, not searched); and whether clustered discoveries are a fault. One more for the pass: a draw that can find crucible steel after about 1778 can also take the machines' whole century away from an island (`sens-machines.md`, seed 3), so the hazard's shape matters to more than the railway's date.

The check's runs and notes are in `.claude/night/reports/opus/trace-steel/check/` and `check.md` beside it.
