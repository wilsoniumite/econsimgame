# Settlers from abroad

People who come to the island by ship. Not in the Python. The switch is
`immigration.enabled`, off under the port's rules and under the game's, and
on with the landfall start ([landfall.md](landfall.md)); the code is
`immigration.rs`, one call at the end of `peopling::growth`, and the
settlers' place in the ages' reckoning (`ages::came`, `ages::year`). The
designer asked for it on 26 November 2026 as a setting to turn on and off,
"just so we can test other growth rates and population pyramid changes
beyond births".

## What it does

Until this rule nobody came to the island. Its people grew by their own
births less deaths, and left it only by ship in a hungry year
([people.md](people.md)). With the switch on, once a year, after each town's
births and deaths are set:

- **The pull** of a town on the coast is how far the living there stands
  above the living abroad: `clamp(y / abroad_now - 1, 0, 1)`, where `y` is
  the town's baskets a head (its income over the cost of a bare living, the
  number the people's rule reads). It is nothing where the living is no
  better than abroad and full where it is twice as good.
- **The living abroad** is `abroad`, 1.5 baskets a head, until `abroad_from`
  (1820), and grows `abroad_growth`, 1 per cent a year, after it.
- **The settlers** a port takes in the year are its pull times `ships` (400
  a year, shared among the ports by their people: the ships that call,
  whatever the colony's size) plus `rate` (0.015) of its own people (those
  who come because kin wrote home), and never more than `max_share` (0.05)
  of its people (what it can house).
- **None to a hungry port** (`shun_hungry`, on with landfall). A port went
  hungry in a year when its worst month's food fell under the famine line,
  0.82 of what its households need (`facts::FAMINE_LINE`): the line the
  news's "Hunger in ..." and the people's rule's famine use. Partly short,
  between 0.82 and 1, is not hungry: such a port still loses lives to the
  dearth but makes no news abroad. The ships stay away that year and the
  four after (`shun_years`, 5, the hungry year counted): word of a hungry
  colony went home by the next ships and was believed for some years, and
  the letters of kin that drew most settlers said where not to go (from
  memory). Only the port's own hunger counts; a hungry town inland is the
  island's own trouble. `State::immigration` keeps the last year each town
  went hungry and how many ports were passed by each year.
- **Their ages** are mostly young adults (`immigration::AGES`): two in three
  between 15 and 34, a fifth children who came with them, few past fifty.
  They enter the reckoning of ages at those ages as the year begins, after
  those who took ship and before the year's deaths, so that births stay what
  is left over, as before ([ages.md](ages.md)). Those past fifteen bring the
  schooling a living of `abroad_now` buys.
- **People only.** They bring no money, and the island trades with nobody:
  every purse is the island's, and the monthly check of the money is
  untouched.
- **The news** says when the first ship comes, and any year whose settlers
  are one in a hundred of the island.

It needs the game's people (`people.enabled`): the port's capped rule never
calls it. Off, nobody comes, nothing is reckoned, and `State::immigration`
stays empty.

## Its numbers, and where they come from

All from memory, not from the repository:

| number | value | why |
|---|---|---|
| `abroad` | 1.5 baskets a head | what the settled island lives on in 1750: the country the settlers leave is the island's own old country |
| `abroad_from` | 1820 | western Europe's income a head grew about 0.15 per cent a year from 1700 to 1820 in Maddison's tables, taken as standing still |
| `abroad_growth` | 0.01 a year | the same tables after 1820: about a doubling every seventy years |
| `ships` | 400 a year | a ship or two a year of some two hundred souls, as a young colony had |
| `rate` | 0.015 of a port's people a year | the United States' immigration in its heaviest decades, the 1850s and the 1900s, about one in seventy-five a year |
| `max_share` | 0.05 of a port's people a year | what a town can house; it binds only in a colony's first years |
| `shun_years` | 5 years, the hungry year counted | how long word of a hungry port keeps the ships away: a guess, from the time letters took and the memory of a bad year |
| `AGES` | 0.63 between 15 and 34, 0.20 under 15 | the shape of those who landed in America in the nineteenth century |

## What the sweep showed

On islands 1 to 8 from landfall (the arms in [landfall.md](landfall.md)):
see there. On the settled island the switch is off and nothing moves; turned
on there with `--set immigration.enabled=true`, its settlers are the pull of
a living that stands at about 1.5 baskets a head in 1750, so they come only
as its towns grow richer than abroad, and stop as abroad grows richer after
1820. Not measured on the settled island on this branch.

**None to a hungry port**, 26 November 2026: landed with land that stays
taken and measured with it, so the two are told together in
[frontier.md](frontier.md), "What the runs showed". On islands 1 to 8 from
landfall with carriage on prices from 1750, settlers to 2050 fall from
730,000 to 624,000 on the mean, and years with hunger after 1900 from 1.0 to
0.6 an island. With settlers off altogether the colony has 168,000 people
in 1900 against 493,000, and 3.0 years of hunger after 1900.

## What worries me

- The pull reads each port's own living, so a rich port draws settlers who
  then spread by the island's own migration. A colony's settlers went
  straight on to the land; here they reach it only by the moves between
  towns and by the districts the towns take.
- The living abroad is one number for the whole world. The island's own
  living after 1900 is not comparable to it in any careful sense: `y` is
  baskets of the island's own goods, and the content after 1900 is not the
  content of 1750.
- There is no going home and no emigration beyond the hungry year's: when
  abroad is richer, nobody leaves for it.
