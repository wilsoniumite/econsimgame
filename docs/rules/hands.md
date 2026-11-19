# Hands that exist

A town cannot hire hands it does not have. Not in the Python. The switch is
`hands.ration`, off under the port's rules and on under the game's; the code
is `hands.rs` and two marked calls in `econ::step_month`. It is stage 2 of
the pass over all the rules (`docs/designs/rules-pass.md`, section 4.1), and
it mends a fault rather than choosing a number: it has no numbers.

## What was wrong

Each month the labour market looks for a town's wage between half and twice
last month's, and never above 500. The land's hands move only
`labor_adjust_ex` (0.15) of the way a month to what their employers want, so
most of this month's demand for hands is last month's hands, whatever the
wage. When those carried hands alone are more than the town has, no wage in
the bracket clears the market, and the port puts everyone asked for to work
and pays them. The town then has more people at work than it has people, its
wage has doubled, and next month it can double again until the land has shed
enough.

A working is where this bites, because a pit wants hands as the third or
fourth power of its net price over the wage, and a town founded at its pit
head makes that price jump overnight. Under the game's rules with six
families, island 3's Brambank, founded on the coalfield at the end of 1750,
had 4,724 at work in January 1751 of 3,211 people of working age
(`examples/overhire.rs`; `mines.md`, "When a working hires more hands than
its town has"). Under the port on island 2, Halhope's wage went 8.9, 17.8,
35.6, 71, 142, 284 and 500 from January to July 1844, and its landlords
ended the year 4.3 million below nothing (`docs/probes/tr-rent.md`, section
1). With a setting of the works or of carting moved, thirteen runs of 528
had a town paying 47 to 121 times the middle town's wage
(`docs/probes/sens-works.md`, "Where a run breaks"), and one null history of
sixty-four had a town at 84.5 times (`docs/probes/sens-mines.md`). One such
run carries the mean of eight islands, so the fault coloured every table of
sensitivity.

## What it does

The hands a town has to spare are its people of working age less its
carters, its sinkers and its navvies, as before; each of those three is taken
out of the town's supply first, and this rule shares out what is left.

- If at the highest wage the month allows (twice last month's, or 500) the
  hands asked for are still more than the town has to spare, the town is
  short. Its wage is set at that highest wage.
- Every employer in a short town, farm, wood, quarry, pit, mine and works
  alike, landlords' land and families' land alike, is cut by the same share
  to the hands there are: the share is the hands to spare over the hands
  asked for at that wage.
- What an employer carries into next month is what it got, not what it asked
  for (`ex_Lprev` for the land, `f_labor` for the works).
- A town with nothing left after the carters, sinkers and navvies are taken
  out hires nobody.

In the code, after the wages are found, `hands::short_towns` asks each active
town's demand at the top of its bracket, the same sum the search asks, and
holds a short town at that wage; `hands::ration` then multiplies the town's
land units' hands and its works' hands by the share before anything reads
them. Production, the materials the works order, the wages paid, the rents
and next month's carried hands all follow from the hands got, as in the port
they follow from the hands asked for. Nothing is asked while the switch is
off, so a port run is the port's to the byte (the golden test).

The second half is what cures it and not only hides it. Because the carried
hands are cut to what exists, next month's demand at any wage is at most the
town's hands plus 0.15 of the gap to what is wanted, and the land wants hands
as the third or fourth power of its price over the wage, so a wage a few
tenths higher clears the market. The wage rises for a month or two and stops.
The wage itself still doubles in a month when a town is short; only the
hands that do not exist are gone.

## The numbers

None. `HandsParams` has the one switch, `ration`. Stage 9 of the pass is to
add the land's speed of hiring to the same struct (`hands.land_adjust`,
`hands.farm_adjust`), which is why it is named for the hands and not for the
ration.

## The tests

`tests/hands.rs`. The three in the middle also show the fault with the rule
off, so that they fail if the rule is taken away; the first guards the
switch and the last is a property every run must have:

- the switch is off under the port and on in the game, and a port run's
  `params.json` does not mention it;
- the largest town of island 3 loses four fifths of its people overnight:
  without the rule it has more than half as many again at work as it has
  people; with it no more than it has, its land and works hold exactly the
  hands left after the carters and the rest, every land unit is cut by one
  share, its wage stands at the top of its range, and no other town moves;
- the same town's carters are given every hand it has: without the rule its
  farms and works hire men who do not exist; with it nobody is hired, and the
  next month goes on;
- island 3 with six families in the first three months of 1751: without the
  rule Brambank has more at work than it has people; with it no town does;
- a rationed run to 1755 conserves money every month and is the same run
  twice.

Three older tests were written on histories the rule changes, and were moved
to find their case again: the commons' fund in debt (`tests/mines.rs`) is
held with this rule off, since the rule spares island 3 the bad year the
fund's rule was written for; the sailing that is news of its own kind
(`tests/people.rs`) is island 26's of 1821, since island 2's of 1827 is gone;
and the decade's news from a run begun in 1755 (`tests/chronicle.rs`) is
read to 1800, since seed 7 now has none to tell before the 1790s.

## What the sweeps showed

Measured on 12 November 2026, under the game's rules with six families,
against main as it stood (00aa886). The output is kept outside the
repository, with the notes of how it was read.

**Switched off it is today's game.** `sweep --seeds 1-16 --rules game
--families 6 --set hands.ration=false`, compared with main's last landing on
seeds 1 to 8: of the 563 facts both have, 0 differ.

**Its own measure: no wild town-month.** The five runs the design names
(`docs/designs/rules-pass.md`, 4.1), with `examples/wage_watch.rs` (`game 6
on SEEDS SETTING`, and the same with `hands.ration=false`): the greatest a
town's wage stood over the middle town's, and where.

| the run | without the rule | with it |
|---|---|---|
| island 3, `works.year_room=0.1125` | 120.8 times (Brambank, July 1751); 13 town-months over four times; 7 overworked | 1.6 times; none; none |
| island 4, the same | 47.2 (Sunhaven, June 1785); 7; 3 | 2.1; none; none |
| island 11, `carting.labor=0.5` | 100.0 (Melford, July 1750); 13; 8 | 1.4; none; none |
| island 9, `mines.sink=40.0002` | 84.5 (Iveldelf, September 1782); 12; 8 | 1.5; none; none |
| island 6, `mines.reach=1.5` | 16.0 (Cranbourne, April 1765); 3; 1 | 1.3; none; none |

("Overworked" is the watch's count of town-months with half as many hands
again at work as the town has people.) The column without the rule gives the
design's five numbers to the last figure. At the game's own numbers on
sixteen islands (`wage_watch game 6 on 1 16`), without the rule the worst is
Brambank at 8.6 times in March 1751, with three town-months over four times,
all on island 3; with it no town on any island passes 2.2 times (Kirkside,
island 12, February 1870), and on islands 17 to 24 none passes 1.9
(`wage_watch game 6 on 1 24`). So aim 13 of the pass, no town paying over
ten times the middle town's wage in sixteen runs, is met with room to spare.

`examples/overhire.rs` on the same sixteen islands with the rule on: no
town-month with more at work than the town has people, under the landlords
(of 406,716) or under the commons (of 445,788), where without it there were
6 and 19 (above, `mines.md`). The rule acts, holding a town at the top of
its month's range and sharing out its hands, in 5 town-months from 1750 to
1900 under the landlords and in 15 under the commons. None of the commons
runs failed the money check. Islands 17 to 24 add none over their people
either, of 194,256 town-months under the landlords and 217,620 under the
commons.

Under the commons the common fund carried the losses of a town paying hands
it did not have, and `commons::fund_to_share` was written for the debt they
left. With the rule the fund is never in debt on islands 1 to 16, where
without it island 3's was in debt for 110 months from 1751, at the deepest
4.6 times all the money the island began with, and island 13's for 15
months from 1750 (a short probe, kept outside the repository with the
check's notes, that counts each month the fund ends below nothing, six
families, to 1900). The design expected `fund_to_share` to find no debt
left to deal with, and it finds none.

The landlords' debt, with the people rule off so that redrawn districts
bring the fault back (`examples/trace_rent.rs --seeds 1-16 --part a --set
'{"people":{"enabled":false}}'`): without the rule there are 154 spells of a
town's landlords ending a year below nothing, 951 town-years, the deepest 26
million and one still owed in 1900; with it there is one, Leaside on island
8, 298 below nothing at the end of 1771 and paid off the next year.

**On the scorecard nothing moves beyond chance.** The rule changes every
island's history, because it acts in the ten years of warm-up, when the
first prices are guesses, on every island: against main, 551 of 564 facts
differ on seeds 1 to 8. So it was read as the pass reads a stage
(`rules-pass.md`, section 3): sixteen islands, the rule on and off, each with
a null pair (`works.walk=0.0200001` and `carting.adjust=0.2500001`, numbers
moved by a millionth), so three histories with the rule against three
without. Seed for seed, the mean of the three with less the mean of the three
without, set against how far two histories of the same rules differ:

| fact | without, the mean of three | with, less without | islands up/down |
|---|---|---|---|
| share of people in towns, 1881 to 1900 | 0.421 | -0.001 | 6/10 |
| what a wage buys, 1881 to 1900 (1750 = 1) | 2.69 | -0.02 | 6/10 |
| the wage's worst year | 15.4 per cent down | 0.5 points worse | 7/9 |
| famine years in a run | 0.50 | -0.08 | 4/5 |
| the farm zigzag | 2.02 | -0.02 | 6/10 |
| iron's price swing | 0.175 | -0.002 | 5/11 |
| years from coke iron's discovery to half of iron | 7.6 | -0.3 | 5/9 |
| people in 1900 | 724,900 | -2,300 | 8/8 |
| the first railway | 1828.1 | 2.2 years later | 9/7 |
| canal in 1800 | 695 km | -27 km | 8/8 |
| railway in 1850 | 1,304 km | -131 km | 4/12 |
| railway in 1870 | 1,821 km | -33 km | 9/7 |

Of 550 facts with a value on every island, two pass the test of three
standard errors with twelve islands of sixteen moving the same way, and both
are of 1750: the land under the diggings, 0.018 of a cell less, and the price
of clay in wages, a sixth of a per cent less. Those are the rule's own doing
in the warm-up. A null pair hardly moves the island of 1750, so there a
small change the rule makes on a few islands stands out: on the mean of
sixteen, works in 1750 are 2.5 fewer of 228, the land under farms 1.8 cells
less of 1,562, coal dearer in wages by 1.2 per cent. None of it is on the
card, and none of it goes the same way on twelve islands.

The check of this page read it again on seven histories each way: the rule
on, under the six small moves of `harvest.island_sd` the scorecard's
baseline uses for its floor (`docs/probes/scorecard-baseline.md`), against
that page's seven histories of main. Nothing on the card moves: the railway
of 1850 is 56 km shorter on eight islands and longer on eight, the canals
of 1800 11 km shorter, eight and eight; what a wage buys in 1881 to 1900 is
0.003 less, the towns' share 0.0007 less, people in 1900 3,100 fewer (the
floor of chance is 5,100), the farm zigzag beyond the weather 0.06 lower,
inside its floor of 0.07. Pooling those histories with the three each way
above (nine each way), five facts of 551 pass the test, the land under the
diggings of 1750 and four small ones not on the card: the diggings' share of
the land in 1800, the railway's share in branches, grain's swing and the
royalties' share of output in 1800, each by a few per cent of itself and on
twelve or thirteen islands of sixteen, which is about what chance gives
among 551 facts.

That harvest's null has a weakness worth knowing: moving `harvest.island_sd`
by a ten-millionth leaves 1750 exactly where it was and barely moves the
dates of the first inventions, where `works.walk` and `carting.adjust`
moved by a millionth act from the warm-up. Island 6's coke iron is found in
1776 in six of the harvest's seven histories of main and in 1790 in both of
the other two nulls. Against the
harvest's null alone, the rule seems to put coke iron a year later on the
mean; against nulls that act in the warm-up it does not. A rule that acts in
the warm-up wants a null that acts there too.

**The bar for the coal inventions.** One island's dates do move with the
rule, in every history: island 11 finds coke iron in 1772 for 1778 and the
engine works in 1780 for 1797. The port sets each invention's bar at a
multiple of what the island made in a year of the warm-up (coke iron at 35
years of its coal and 30 of its iron; `sim::start_from`). Without the rule
island 11's warm-up dug 250,000 of coal a year, some of it by hands its
coal town did not have; with it 214,000, so coke iron's bar falls from 8.76
million of coal to 7.48 million and the engine works' from 7.00 to 5.98
million. On the islands whose warm-up had no such town the bars move by
under half a per cent.

The critic asked for two facts to be read against the null pair, because a
single comparison had them lower on eleven islands of sixteen
(`rules-pass.md`, 10.3). The canals of 1800 do not move. The railway of 1850
leans a tenth shorter, on twelve islands, at the edge of chance (2.95
standard errors). On sixteen more islands (seeds 17 to 32, one history each
way) it goes the other way, 76 km longer on nine of sixteen, and the first
line opens four years sooner, not two later. It is the date of the first
line, which is the draw's (`docs/probes/tr-steel.md`), and the rule does not
move it.

**One history to know about.** In the stage's own sweep island 3 stalls from
1840: what a wage buys in 1881 to 1900 is 1.75 times 1750's where without
the rule it is 2.51, and the towns' share 0.34 for 0.40. Its machinery stays
dear for thirty years, farm machinery is found in 1864 and the converter
never. In the rule's two other histories of the same island the figure is
2.78 and 2.45, so it is the draw and not the rule; the late draw is stage 7's
business. It moves the sixteen islands' means (that sweep alone, against
today's, reads the late century a little poorer on five facts), which is why
the stage is read on three histories and not one.

## What worries me

- **The wage still doubles.** A short town is held at twice last month's
  wage, as the port holds it, and only the hands that do not exist are gone.
  The wage rises for a month or two and stops, because the carried hands are
  now real; but a town short of hands could more fairly pay whatever wage
  clears, beyond the month's bracket. That would move more and was not asked.
- **It changes everybody's history.** Every island goes down another road
  from the warm-up on. Read on one history, an island's draw can look like
  the rule (island 3 above). Anything measured on the game's rules before
  this landed is a different history from anything measured after.
- **The works are cut before their materials.** A works' hands are cut by
  the town's share and then, as before, laid off further if it cannot get its
  materials, so a short town's works can end with fewer hands than its share.
  The town's land then has hands to spare that nobody hires that month.
- **It is a mend, not a model of a labour market.** Hands do not move
  between towns within the month, and the land's slow hiring
  (`labor_adjust_ex` 0.15) is what makes a town short at all; stage 9 of the
  pass speeds the woods', pits' and quarries' hiring, and should be read with
  this rule in.
