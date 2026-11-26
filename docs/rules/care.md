# Care: work of presence in the towns

Added on 26 November 2026. Not in the Python. One switch, `care.enabled`, off under the port's rules and on
under the game's. `src/care.rs`, the need, the good and the method in `data/game.toml`, their kind in
`data/tasks.toml`, and `tests/care.rs`.

Care is nursing, doctoring and minding the old and the sick: work that is done by being there. Its output a
head hardly rose in a century, so its price rose with the wage, and richer and older countries bought more
of it all the same: its share of the hands grew from two or three in a hundred before the First World War
to an eighth and more today. Teaching is not in it: the treasury's schools already hire the island's
teachers out of the rate on pay ([state.md](state.md)).

## Why

Machines that tend ([tending.md](tending.md)) stand off because the hands they free go to the land, the
pits and the carts and the towns empty: hands follow pay bills, the works' pay bill falls with every machine,
and the land's does not ("Where the hands go"). The island had no work in its towns that grows as households
grow richer or older and that no machine takes. Looked at first (below, "What the island had"): after 1970
every need's share of the budget is flat, the one service the island has is the offices', and every hour of
an office is open to some machine. The lean toward services was being tended away. Care is the work that
is not.

## The design, in five sentences

1. **A ninth need, `care`, and one good that serves it**, made by one method, `care_homes` (hospitals and
   homes of care), found by the tree's hazard on services and chemicals made, from 1908; history's year 1911,
   when insurance against sickness paid the panel doctor and the district nurse.
2. **Presence, no learning, no economies of size**: every hour of it is of the class the tasks' file closes
   to machines, so no machine of that rule takes any of it, today's or a later one's; it learns nothing by
   doing; and its hands a unit are the recipe's at any size of works and in any town.
3. **A service**: not carried between towns and not kept (`traded = false`, `stock_months = 0`): what
   December makes waits for January's buyers and is lost at the month's end if they do not buy it.
4. **A want that grows with income and with age**: a household's share of the Engel rule's richer set on
   care is `0.65 * (its town's people over 64 + 3 * those of 80 and over) / its people`, times what the town
   knows of care, and the other needs give it room in proportion; below the Engel rule's reference income
   none is bought.
5. **Households pay**; the treasury's part is written down below and not built, since households carry the
   pass.

## Why each choice

**A need and not a good under comfort.** Within a need the goods are split by a logit on their price per
unit of the need. A care that grows dear as wages rise would lose its share of comfort to the offices'
services: the wrong way round. As a need of its own it holds its share of the budget whatever its price,
and the money spent on it, and so the hands, follow the share.

**Nothing moves before it is found.** The need has a share of nought in every set the content gives
(subsistence, the shares at subsistence, the richer set, the leaned set, the landlords'), so every sum over
the needs gains a term of exactly nought, and the logit within it has one good. The good begins with its
method (`begins`, [goods.md](goods.md)). The method is sought only with the rule on, and its yearly draw
comes from a stream of its own (`rng::idea_stream`, as the tending ideas'), so every other idea is found
when it was. A town buys care only once the idea has reached it (`st.know`), so no money is offered for it
where no works can sell it. With the rule off, a sweep of islands 1 to 8 to 2050 differs from main's in none
of its 37,582 facts; with it on, in none of the played century's ([tests/care.rs](../../crates/isostate-core/tests/care.rs)
holds island 7 bit for bit to the end of the year before care is found).

**The idea.** No idea of the tree is about sickness or age, and a method of the tree is itself an idea,
found and spread as an invention is; so the method is the idea, as the offices are. On services (41 million
made since 1750, the offices' own threshold) and chemicals (6.2 million: medicines, soap, antiseptics) every
island has run out the clock by 1908, the floor, so the hazard is near its most and care is found about five
years later.

**The want.** The Engel rule's real income a head stops rising in 1950 and falls after 1970 (15 bundles to
12 by 2050: fewer hands a head and a rising rate on pay), and at those incomes the richer set governs the
whole budget above subsistence; so a want tied to income alone would stop in 1950. What keeps rising to
2050 is age: the share of the island over 64 goes from 0.06 in 1910 to 0.09 in 1950 and 0.19 in 2050, and the
share of 80 and over from 0.006 to 0.012 and 0.066. The old, and the very old above all, are who care is for.
Income still decides through the Engel rule's weight: a town at its reference income buys none, a richer
town spends its richer share. The weights were chosen on islands 1 to 8 against history's line (below): with
the old alone the want rose too fast before 1950 and too little after; counting one of 80 and over as four
of 65 to 79 (about what the age-cost curves of health and long-term care say) follows the line.

**Flat hands a unit.** The port's rule makes a works smaller than its usual size take more hands a unit
(`1 + 0.8 * sqrt(K0 / K)`), and the city's rule makes a works in a large town more productive. Both stand for
economies of size that a ward does not have: forty beds need forty times the nursing of one. Under them care's
hands a unit fell by two fifths from 1930 to 2050 as its works grew, and its price fell against goods'. So
care's hands a unit are the recipe's whatever the works' size or town (`econ::firm_coefficients`, one line
that is the port's arithmetic for every other method).

**Who works in it.** Hands, as every works' are: the island's one wage clears each town, and who-works
([who-works.md](who-works.md)) counts hands of working age, so care takes the same hands as everyone else.
The pay gauge ([pay.md](pay.md)) counts its hours as presence.

**Who pays: households first.** Landlords and families keep their own shares, which have none on care (the
gentry's nurses are in their comfort).

## The treasury's part, not built

Care for all out of the rate would have the treasury buy each town's care for its old, as it now buys
road-making ([highways.md](highways.md)) and hires teachers: a quantity a head of the old, ordered from the
town's care works at the town's price, and paid out of the rate on pay, which would rise by what it costs.
It would change who pays and who is cared for (a poor town's old would have the care a rich town's have)
more than how many hands care holds: hands follow the money spent on care, whoever spends it, and a
household that pays more on the rate has that much less to spend on care itself. Households alone carry the
pass, so it is not built. It is the natural first act of a politics of care, and the place for it is the
book of acts in `purse.rs`.

## Its numbers

| | game | what it is |
|---|---|---|
| `care.enabled` | on | care is sought, wanted, made and bought |
| `care.base` | 0 | a household's share of the richer set on care where nobody is old |
| `care.per_old` | 0.65 | what each share of the town's old adds to it |
| `care.over80` | 3 | how much more care one of 80 and over wants than one of 65 to 79, beyond the one share each counts for |
| `care.cap` | 0.5 | the most of the richer set care may take |
| the method `care_homes` | labour 0.8 a unit, chemicals 0.04, heat 0.3; plant 5 of building work, 0.1 of steel, 0.04 of machinery a unit of monthly capacity; usual size 400 a month | at base prices seven eighths of its cost is hands |
| the good `care` | base price 3, weight 0.5, not traded, not kept, opens dear | |

## What the island had

Seen first on main (the lorry and its till on, tending off), islands 1 to 8 to 2050 under the game's rules
with six families, tending off, on, and on with the wide reach (`tasks.tending_wide`), each figure the mean
of the islands over the five years about the mark:

| | 1950 | 1970 | 1990 | 2010 | 2050 |
|---|---|---|---|---|---|
| comfort (services), of households' budget | 0.35 | 0.40 | 0.39 | 0.39 / 0.38 / 0.39 | 0.37 / 0.37 / 0.39 |
| food; shelter; clothing | 0.11; 0.15; 0.11 | 0.10; 0.14; 0.08 | 0.10; 0.14; 0.08 | 0.11; 0.14; 0.08 | 0.11; 0.14; 0.09 |
| real income, bundles a head | 14.1 | 15.1 | 14.6 | 13.6 / 13.1 / 14.5 | 11.9 / 11.3 / 14.5 |
| hands in the offices | 0.36 | 0.42 | 0.43 | 0.42 | 0.37 / 0.38 / 0.40 |
| hands in the mills and factories | 0.11 / 0.10 / 0.10 | 0.10 / 0.09 / 0.08 | 0.10 / 0.07 / 0.07 | 0.09 / 0.06 / 0.06 | 0.08 / 0.03 / 0.04 |
| hands on the land; carting | 0.22; 0.10 | 0.20; 0.08 | 0.21; 0.07 | 0.22; 0.07 | 0.24 / 0.31 / 0.28; 0.07 / 0.12 / 0.09 |
| townsfolk | 0.55 / 0.53 / 0.54 | 0.58 / 0.57 / 0.57 | 0.59 / 0.56 / 0.57 | 0.59 / 0.55 / 0.56 | 0.56 / 0.47 / 0.51 |

(One figure where the three arms agree to the second place.) After the lean is complete in 1970 households
spend their gain on nothing new: every need's share is flat, and real income a head falls. The island's one
service is the offices' (the shops are gone by 1960), 0.42 of all hands in 1970; their hours are clerking,
judging, tending and hauling, none presence, so every hour is open to some machine, and their pay over what
they sell falls from 0.66 to 0.53 by 2050 with tending and the wide reach. Hands follow pay bills, so the
hands the machines free go where the pay bill does not fall: the land and the carts. The lean's services
were being tended away.

## What the islands showed

Islands 1 to 8 and 17 to 32 to 2050 under the game's rules with six families, care off and on, crossed with
machines that tend off, on, and on with the wide reach (`tasks.tending_wide`):

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set care.enabled=false,true \
    --set tasks.tending=false,true --set tasks.tending_wide=false,true --standing --out DIR
```

64 runs and 128 runs, none failed, every run's money checked every month; the standing check 12 of 12 inside
in every arm against main, and the growth asks met in every arm. The four arms the brief asks for are main,
care, care with tending, and care with tending and the wide reach; tending without care is beside them.
Means of the islands, the least and greatest in brackets.

| | main | care | care, tending | care, tending wide | tending, no care | wide, no care |
|---|---|---|---|---|---|---|
| townsfolk 2050, islands 1 to 8 | 0.559 [0.518..0.589] | 0.587 [0.560..0.616] | 0.525 | 0.569 | 0.469 | 0.515 |
| islands 17 to 32 | 0.555 [0.495..0.601] | 0.594 [0.543..0.636] | 0.542 | 0.566 | 0.477 | 0.521 |
| hands in care 2050, 1 to 8 (17 to 32) | | 0.163 [0.130..0.180] (0.167) | 0.176 (0.186) | 0.196 (0.193) | | |
| care, of households' spending on goods, 2050 | | 0.187 (0.193) | 0.184 (0.196) | 0.202 (0.204) | | |
| care's price over goods', 1950 / 2050 | | 1.34 / 1.27 (1.32 / 1.39) | 1.38 / 1.32 (1.32 / 1.51) | 1.33 / 1.67 (1.32 / 1.70) | | |
| what a wage buys, goods, 2050 | 19.3 (18.7) | 18.2 (18.9) | 19.8 (21.5) | 22.7 (22.8) | (20.3) | (21.5) |
| made a head, 2050 over 1750 | 21.1 (20.4) | 21.0 (20.4) | 28.2 (28.5) | 32.4 (31.6) | (27.1) | (32.0) |
| work's share of income, 2050 | 0.62 (0.61) | 0.62 (0.64) | 0.52 (0.54) | 0.50 (0.50) | (0.50) | (0.45) |
| rents (the land's share), 2050 | 0.18 (0.17) | 0.17 (0.18) | 0.20 (0.21) | 0.22 (0.21) | (0.21) | (0.22) |
| the schooled over the plain, 2050 | 0.61 (0.64) | 0.62 (0.62) | 0.71 (0.74) | 0.74 (0.76) | (0.73) | (0.84) |
| slumps (news verdicts) | 8.9 (11.4) | 7.9 (10.3) | 16.1 (18.7) | 16.9 (18.4) | (18.1) | (19.2) |
| years of hunger | 0.4 (1.1) | 0.4 (0.7) | 0.8 (0.6) | 0.3 (0.4) | (1.2) | (0.9) |
| linked wage, %/yr 1900-1970; 2050 over 1970; the land's share 2050 less 1900 | 1.18; 1.16; -0.106 | 1.20; 1.14; -0.115 | 1.27; 1.15; -0.105 | 1.28; 1.24; -0.112 | 1.26; 1.03; -0.068 | 1.27; 1.15; -0.063 |
| islands 17 to 32 | 1.20; 1.18; -0.110 | 1.23; 1.16; -0.129 | 1.31; 1.26; -0.115 | 1.30; 1.29; -0.114 | 1.28; 1.13; -0.086 | 1.29; 1.23; -0.094 |
| people 2050 | 1.298 M (1.323) | 1.254 M (1.273) | 1.277 M (1.282) | 1.242 M (1.264) | (1.350) | (1.312) |

**Hands in care, by decade, against history.** Islands 1 to 8 (17 to 32 within a point): 0.028 in 1930,
0.047 in 1950, 0.073 in 1970, 0.103 in 1990, 0.134 in 2010, 0.163 in 2050. History's line, health and social
work over all in work in Britain and the Scandinavian countries, written from memory and so to be read to the
first figure: two or three in a hundred in 1911, three or four in 1931, four or five in 1951, six or seven in
1971, eight to eleven in 1991, eleven to fifteen in 2011, and fifteen to twenty-two in 2050 with the ageing
to come (Norway's today, the highest, is a fifth). The island follows it from 1930 and sits at the lower half
of its band in 2050. The brief's band for 2050 was 0.10 to 0.30; history's is 0.15 to 0.22, and the island
holds to it on twenty-two islands of twenty-four (the two below are at 0.130 and 0.148).

**Care's price rises against goods'.** Its hands a unit are flat, so its price follows the wage, while goods'
fall against the wage as their makers gain: against what households bought in its tenth year, care's price is
a third higher by 1950 and stays so, and higher still where machines tend (1.67 and 1.70 by 2050 with the wide
reach). Its price over the wage settles at 0.9 from 1970. One island of twenty-four ends at its base year's
level (0.99).

**The towns.** Care lifts the townsfolk of 2050 by three to four points (on every island of the twenty-four);
the land's and the pits' hands give them up. People are three to four per cent fewer: more townsfolk, fewer
births ([transition.md](transition.md)), as with the lean.

**A month's cost.** 159.3 processor seconds a run against 156.6 on islands 1 to 8 (1.7 per cent more), and
155.3 against 162.4 on 17 to 32: within what the machine's other work moves.

## With machines that tend

**Does care hold the freed hands in the towns? Half of them, and with the wide reach all of main's.**
Without care, tending takes the townsfolk of 2050 from 0.559 to 0.469 (islands 1 to 8) and 0.555 to 0.477
(17 to 32); with care, from 0.587 to 0.525 and 0.594 to 0.542. The fall is 5 or 6 points where it was 8 or
9, and the island with care and tending has three points fewer townsfolk than main on islands 1 to 8 and one
fewer on 17 to 32 (0.525 and 0.542 against 0.559 and 0.555). With the wide
reach the towns stand at 0.569 and 0.566, above main's on both sets, two to three points below care alone.
Care's hands rise where machines tend (0.163 to 0.176 and 0.196 in 2050), but not by all the hands the
machines free. From the books (the trace's probe, `examples/tending_trace.rs`, islands 1 to 8, 2050), care
against care with tending: the works but care give up 0.096 of all hands, of which care takes 0.015, the land
0.051 and the carts 0.035. Hands follow pay bills, and with work's share of income falling from 0.62 to 0.50
more of the island's income is rent and profit: households' share of all spending falls from 0.59 to 0.54
(0.51 with the wide reach) and the landlords' rises from 0.14 to 0.17 (0.18), and the landlords' shares have
nothing on care. The
slumps in the news double with tending, with care or without (16 to 19 verdicts against 8 to 11). The next
piece decides whether to switch tending on; the four arms are in the sweeps named in this page's notes.

## The tests

`tests/care.rs`: the rule is off under the port's rules and out of its file, on under the game's; the game's
content has the ninth need with no share in any set, one good that serves it, not carried and not kept,
beginning with its method, every hour of which is presence and which learns nothing (`care_stands_behind_its_switch`);
the want makes room in proportion and adds up (`the_want_makes_room_in_proportion`); and island 7 to 1935 with
care on and off, side by side: the same island bit for bit, every series, to the end of the year before care is
found; then care is bought by households alone, never more than a month's making is in stock, what is made is
what was bought and what was lost at a month's end, the works' value is what they paid out and kept (to the
merchants' share, which the books count in the town's price), and the hands are paid half or more of what it
sells for (`nothing_moves_before_care_and_its_books_balance`, in the slow set). The tests of the content's
counts (`tree.rs`, `power.rs`, `tasks.rs`, `new_good.rs`) count care's good, need and method; `tasks.rs`'s case
of office computing is played with care off, since a works of care has no clerking and, founded after the
office machines, means to hold its town's target as the case's saw pit does.

## In the viewer

Care's works stand in the forms of the offices (`lots::office`, the core's method `care_homes` beside
`offices`): a large civic house of its year, its storeys by its hands and its ground's value. The Markets panel
lists care with every good, from the run's record, and the map's "made here" view shows where it is made.
Wanted: an asset of its own, a hospital and a home for the old (a long low block of wards with a garden, a
porch for the ambulance, a chapel), to the houses' detail.

## What worries me

- **The want leans on the ages.** Care's share grows with the town's old and very old, and the ages are a
  reckoning beneath the played people ([ages.md](ages.md)): if they drift, care drifts with them. Care does
  not yet lower the old's deaths, as it should.
- **Real income does not decide much.** At the island's incomes after 1950 the Engel rule's richer set
  governs the whole budget above subsistence, so care's want is set by age and the idea's reach, not by
  income. If a later piece makes income rise after 1970, a second mark of income is the better place for it.
- **The owners buy none.** Landlords and families keep their shares, which have none on care; as more of
  income goes to rent and profit (with machines that tend), care's share of all spending falls behind its
  share of households'. Whether to give the owners a share of care is a question for the next piece.
- **Care's works and the tasks' rule.** A works of care has no class but presence, yet, founded after a
  machine is found, it means to hold its town's target in every class, as a hand trade does: it holds none and
  orders none, but the number is there (`tasks::opened`).
- **Lost early.** In its first decade care is made in towns before their households know it well and half of
  it is lost at the month's end (island 7, 1915 to 1917); from the 1920s a fiftieth.
- **Teaching is the treasury's**, and counted with the teachers, not with care: the teachers are 0.037 of the
  hands in 2050 (`state_teachers_share_2050`), so care and schooling together are a fifth, where the richest
  countries have a fifth to a quarter.
- **History's line is from memory.** It is the right shape and the right size to the first figure, and no more.
