# The first city: people follow jobs, work gathers where work is, and room that grows dear

Not in the Python. Three rules of the run to 2050, each with a switch of its
own, each acting only after 1900: `city.follow_jobs`, `city.gather` and
`city.dear_room`, off under the port's rules and all three on under the game's
(`game_params()` in `params.rs`, the line `P.city = CityParams::game()`). The
code is `city.rs`, with hooks marked "city" in `people.rs` (migration),
`firms.rs` (a works' productivity and the reckoning of a site), `ground.rs`
(the month's rent) and `sim.rs`.

**The pass is missed on size and met on share.** With the three rules on, on
islands 1 to 8 in 2050 the largest town has 129,000 townsfolk [84,000 ..
176,000] and 0.219 of all townsfolk, where the island as it was had 69,000 and
0.118 (7 islands of 8 larger); the ask was 150,000 and 0.2. On islands 9 to 16
it has 154,000 and 0.255 (8 of 8). Nothing of 1900 moves (the standing check
12 of 12 inside, each of the twelve the same number on and off), the three
asks of the growth after 1900 are met on both sets of islands, no run fails,
and a run takes no longer (70.0 seconds against 69.4).

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --params OFF.json --out OFF
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --standing --growth --compare OFF --out ON
```

(`OFF.json` is `{"city": {"follow_jobs": false, "gather": false, "dear_room": false}}`.)

## What was wrong

The island of 2050 had some twenty-four towns that were all alike. The largest
held 70,000 townsfolk, 0.115 of all townsfolk, where an island country of a
million people has a first city of 150,000 to 270,000, a fifth to a half of
them; the second town was 0.82 of the first where it should be a third. Two
things held the towns level (the step back of 20 November 2026 found them, on
seventeen settings of the rules as they were):

- People moved between districts only toward higher income a head
  (`people::migration`, `y_ratio`), and local prices keep income a head within
  three per cent across towns. Work made more productive in a large town
  raised its wage and was paid back in its own dearer services and rent, so
  nobody had a reason to move.
- The rent burden could not rise. Under the ground's rule
  ([ground.md](ground.md)) a household spends a fixed share of what is left of
  its budget on room above its least, so the bill is the same share of the
  budget however dear the room.

## The three rules

**People follow jobs** (`city.follow_jobs`). Migration weighs each town by the
wage a newcomer would earn there, the year's mean wage, raised where the
town's works took on hands over the year:

    pull = wage x (1 + jobs x new works' hands / the town's people of working age)

with the new hands held between nothing and half, and `jobs` 2 (a town whose
works took on a tenth more hands than it has people of working age reads a
wage a fifth higher). Everything else in the port's migration stands: only a
gain counts, its rate (0.05 of a town's people a year at most) and its reach
(`migration_dist`), a town takes in at most a tenth of its own people in a
year (`people.arrivals_max`), and the movers carry their cash and their
town's measure of living. The wage is what a job pays, not what living there
leaves: that is the point, since what living there leaves is level.

**Work gathers where work is** (`city.gather`). Every works in a town of U
townsfolk is

    (max(U, 1000) / size)^power

as productive, with `size` 20,000 and `power` 0.07, ramped in over ten years
from 1901 so that a small town's works do not lose a tenth overnight. A
doubling of a town is worth 5.0 per cent on every works in it (the studies of
real cities put it at 2 to 8). The factor is worked out once a year
(`State::city.scale`) and read both by the works' productivity and by the
reckoning of a site, so a works planned in a large town is planned at the
productivity it will have.

**Room is wanted less than in proportion as it dears** (`city.dear_room`). In
the ground's rent, the room a head takes above its least was `beta x spare /
rent`; now it is

    beta x spare / rent x (rent / ref)^(1 - e)

where `ref` is the town's own rent of a unit of room in 1900 carried in
today's money (over the long run's anchor) and `e`, `room_elasticity`, is 0.5.
So the share of the spare budget spent on room is `beta x (rent / ref)^(1 -
e)`: the ground's own in 1900, and rising as the town's room dears, so that a
city's households give up more of their budget for less room. The rent that
clears the town's room is then `(n x beta x spare x ref^(e - 1) / (room - the
least room of all))^(1 / e)`, a closed form as before; where the least room
fills the town the ground's own fallback stands. With `e` at 1 it is the
ground's rule.

Nothing acts in the yearly step of 1900 or before: migration and the works'
productivity from the yearly step of 1901 (the step of 1900 only puts by each
town's works' hands, which the rule owns), the rent from the months of 1901.

## What the island does with them

Each island played once to the end of 1899 under the game's rules, then once
for each arm from that same state (the step back's probe, with arms for these
rules). 2050, mean of the islands [least .. greatest]; burden is house rent
over the townsfolk's budget from the books, the largest town against the
others' mean weighted by townsfolk; `dens` in the largest town, its cells'
mean and top (the other towns' mean of their cells); up/down is the largest
town against the island without the rules, island by island.

| islands 1 to 8 | towns' share | largest town | its share | second / first | land's hands | burden largest / others | `dens` largest mean / top (others) | people | up/down |
|---|---|---|---|---|---|---|---|---|---|
| without them | 0.552 | 69,000 [49k..109k] | 0.118 | 0.82 | 0.173 | 0.109 / 0.108 | 1.32 / 1.53 (1.35) | 1.075 M | |
| jobs | 0.533 | 111,000 [91k..134k] | 0.187 | 0.86 | 0.178 | 0.112 / 0.111 | 1.28 / 1.47 (1.31) | 1.119 M | 8/0 |
| gather | 0.541 | 78,000 [60k..102k] | 0.135 | 0.83 | 0.174 | 0.112 / 0.109 | 1.32 / 1.54 (1.36) | 1.085 M | 7/1 |
| dear room | 0.548 | 73,000 [57k..107k] | 0.124 | 0.79 | 0.173 | 0.170 / 0.177 | 1.63 / 1.88 (1.54) | 1.082 M | 6/2 |
| jobs, gather | 0.514 | 139,000 [111k..170k] | 0.233 | 0.83 | 0.182 | 0.116 / 0.110 | 1.35 / 1.58 (1.30) | 1.169 M | 8/0 |
| jobs, dear room | 0.532 | 123,000 [101k..141k] | 0.206 | 0.75 | 0.183 | 0.186 / 0.197 | 1.69 / 1.93 (1.59) | 1.134 M | 8/0 |
| gather, dear room | 0.545 | 85,000 [59k..129k] | 0.145 | 0.78 | 0.173 | 0.178 / 0.186 | 1.65 / 1.91 (1.62) | 1.090 M | 7/1 |
| **all three** | 0.498 | 129,000 [84k..176k] | 0.219 | 0.84 | 0.197 | 0.220 / 0.191 | 1.78 / 2.08 (1.53) | 1.190 M | 7/1 |
| all three, power 0.12 | 0.486 | 147,000 [115k..181k] | 0.247 | 0.86 | 0.214 | 0.178 / 0.195 | 1.64 / 1.92 (1.49) | 1.241 M | 8/0 |
| all three, migration's reach 300 | 0.499 | 138,000 [122k..156k] | 0.234 | 0.82 | 0.183 | 0.172 / 0.175 | 1.55 / 1.85 (1.46) | 1.203 M | 8/0 |
| all three, with the Engel lean | 0.565 | 156,000 [117k..201k] | 0.260 | 0.82 | 0.151 | 0.190 / 0.192 | 1.68 / 2.00 (1.58) | 1.062 M | 8/0 |

| islands 9 to 16 | towns' share | largest town | its share | second / first | land's hands | burden largest / others | `dens` largest mean / top (others) | people |
|---|---|---|---|---|---|---|---|---|
| without them | 0.554 | 73,000 [59k..88k] | 0.123 | 0.84 | 0.167 | 0.112 / 0.108 | 1.52 / 1.75 (1.32) | 1.076 M |
| jobs, gather | 0.491 | 145,000 [117k..165k] | 0.251 | 0.82 | 0.195 | 0.117 / 0.111 | 1.33 / 1.55 (1.26) | 1.179 M |
| **all three** | 0.503 | 154,000 [128k..189k] | 0.255 | 0.83 | 0.174 | 0.180 / 0.189 | 1.65 / 1.92 (1.56) | 1.206 M |

The growth after 1900, by the linked wage (the mean of the islands' figures:
per cent a year 1900 to 1970, 2050 over 1970, and the land's share of hands
in 2050 less 1900; the asks are 1, 1 and below nothing):

| | islands 1 to 8 | islands 9 to 16 |
|---|---|---|
| without them | 1.245, 1.201, -0.073 | 1.303, 1.212, -0.082 |
| jobs, gather | 1.154, 1.062, -0.064 | 1.230, **0.990**, -0.054: missed |
| **all three** | 1.150, 1.034, -0.049 | 1.200, 1.021, -0.075 |
| all three, power 0.12 | 1.173, **0.973**, -0.032: missed | |
| all three, reach 300 | 1.152, **0.999**, -0.064: missed | |
| all three, with the Engel lean | 1.229, 1.116, -0.091 | |

Read:

- **People following jobs is the rule that makes the city**: alone it takes
  the largest town from 69,000 to 111,000, on every island, by 1950, and the
  wage's spread across towns closes from 0.127 to 0.054 (its coefficient of
  variation, by people) as it does. Then the pull is spent, and the largest
  town stands still.
- **Work gathering is the loop it feeds**: alone it adds 9,000; with people
  following jobs, 28,000 more, and the wage's spread stays open (0.097).
- **Dear room alone does nothing for the city** and lifts every town's burden
  alike, from 0.11 to 0.17. With the other two it is the only arm whose
  largest town pays more of its budget for room than the others (0.220
  against 0.191) and whose ground grows dear enough for `dens` to pass 2 in
  the city: the dear ground a tower needs. It costs the city 10,000 on
  islands 1 to 8 and gives it 9,000 on 9 to 16, which is chance between two
  arms, and without it the wage's ask of 2050 over 1970 is missed on 9 to 16.
  So all three are on.
- **The power is 0.07.** At 0.12 the city is larger (147,000) and the wage
  of 2050 is under that of 1970; a migration that reaches the whole island
  (300 for 60) misses it too. Both stay settings.
- **What holds the city now is the towns' share of people.** A town's
  townsfolk are its works' hands and their households (`towns.rs`), and the
  towns' share of people falls with the rules on (0.552 to 0.498): the people
  who follow the jobs fill the city's district faster than its works take on
  hands, and the rest work its farms. At a fifth of townsfolk the island of
  1.2 million has 600,000 townsfolk to share out; a city of 150,000 is a
  quarter of them. With the top of the Engel rule leaning from shelter and
  clothing to comforts (the step back's arm: comfort's rich share 0.28 to
  0.48 from 1900, which is content and not landed here), services grow, the
  towns' share rises to 0.565, the land's hands fall to 0.151 and the largest
  town reaches 156,000 and 0.260, with the growth asks met. So the city of
  the ask waits on services that grow with income.
- **The first city is seldom the largest town of 1900.** On islands 1 to 8
  with the rules on it was the third to sixth town of 1900 on every island
  (third on two, fourth on two, fifth on three, sixth on one), with 20,000 to
  31,000 townsfolk then; all inland but two ports (Ulmouth, island 1;
  Eastwick, island 8). On islands 9 to 16 it was the largest of 1900 on
  three, the second on one and the third to fifth on four. The towns that grow are those whose works grow
  first after 1900, not those that were large.
- **The towns below the first five empty.** In 2050 the island has 12 to 18
  towns with the rules on against 25 to 30 without them, and the sixth town
  is a tenth of the first: not the rank and size of a real country, whose
  towns fall away steadily. The cluster of four or five large towns is also a
  departure: the second town is still 0.84 of the first.

## What the picture gets wrong for a town of that size

Island 7's Dunheath, 174,000 townsfolk in 2050 with the rules on and 49,000
without (pictures in the night's shots, `first-city`):

- 1950 (128,000 with the rules, 69,000 without): a wide sprawl of the same low terraces and
  cottages as a market town, from the centre to the edge; nothing denser at
  the centre.
- 2000 (167,000): the same sprawl a little wider; no flats, no offices, no tower, no
  road wider than the rest, though `dens` in the city stands at 2.
- 2050: the city fills the whole view as an even carpet of two-storey houses
  and works' yards; at its centre a church, a gasworks and detached cottages;
  the rent map is one colour over all of it (its scale tops at 25,000 pounds a
  square, which every city square passes).

## Facts

From the record, so a run with the rules off has them too (every run under
the towns' rule), in 1900, 1950, 2000 and the last year:
`townsfolk_largest_Y`, the largest town's townsfolk; `townsfolk_largest_share_Y`,
its share of all townsfolk; `townsfolk_second_over_first_Y`.

## What worries me

- The size of the pass is missed on islands 1 to 8, and met on 9 to 16: the
  mean of 1 to 8 is 129,000 with islands from 84,000 to 176,000. The rules
  make a first city of the right share on most islands and of the right size
  on half of them.
- The wage's ask of 2050 over 1970 is met with little to spare (1.034 and
  1.021) and missed by some islands (0.95 and 0.98 on two of 1 to 8). People
  following jobs raises the island's people by a tenth in 2050 (1.19 million
  against 1.08); I did not trace why.
- On a few islands the first city loses half its townsfolk in a decade (one
  from 240,000 in 2009 to 85,000 in 2016), as the district it is the market
  of changes or another town takes the lead. Not traced.
- The pull reads the nominal wage. A town whose wage is high because its
  prices are draws people as readily as one whose wage is high because its
  works want hands.
