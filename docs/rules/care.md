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
