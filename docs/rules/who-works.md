# Who works, by age: school, the university and the pension take hands out of work

Added on 25 November 2026. Not in the Python. `crates/isostate-core/src/who_works.rs`, switched by
`who_works.enabled`: off under the port's rules, on under the game's. It acts from 1901 and comes in over
ten years; to 1900 every town's hands are half its people, as they have always been, and nothing of the
played century moves.

**The standing check**, islands 1 to 16 to 2050 with the rule off and on:

```bash
isostate-core sweep --seeds 1-16 --rules game --families 6 --end-year 2050 --set who_works.enabled=false --out off
isostate-core sweep --seeds 1-16 --rules game --families 6 --end-year 2050 --standing --out on --compare off
```

```text
16 runs under the game rules: seeds 1 to 16, 1 setting; 0 failed
standing check: 12 of 12 inside (0 moved, 0 OUTSIDE, 0 without a value)
growth after 1900: per cent a year, 1900 to 1970 1.27 [1.00 .. 1.48] met; 2050 over 1970 1.14 [0.94 .. 1.28] met;
                   the land's share of hands, 2050 less 1900, -0.111 [-0.139 .. -0.086] met
```

To 1900 not one fact differs on islands 1 to 8, seed by seed (the comparison to 1900 counts three that have no
value on either side), and `tests/who_works.rs` holds every series to the bit to December 1900 on island 5.

## Why

Under the port a town's hands for hire are half its people (`participation`, 0.5), whatever their ages. Since
20 November the island keeps its people's ages (`ages.md`), and since then it has schooled its children (five
years in 1900, eleven by 2050), sent nearly half of its young through a university by 2050, and paid pensions
to the old, a fifth of its people by 2050 (`state.md`). While the hands stayed half the people, none of that
cost the island anything in work: a child at school was at work too, a student worked while studying, a
pensioner went on working, and the rate on pay bought things that took nobody away from anything. This rule
makes those three take hands out of work.

## What it does

At each year's end, once the ages have closed and the purse has set next year's laws, every town's people are
counted by age for the year to come (`who_works::close_year`), from its seventeen five-year groups:

| ages | in work, of that age | taken out by |
|---|---|---|
| under 10 | none | |
| 10 to 14 | a quarter (`child`) of those not at school | school |
| 15 to 64 | `working` (0.705) of those not at school and not at the university | school past 15; the university |
| 65 and over | 0.55 (`old`) of those no pension reaches, and 0.08 (`pensioned`) of those it does | the pension |

- **A child is at school** from five for the years its town gives it this year: what its families' income buys
  (`ages::school_years`, two to five years by the town's income, the stand-in from before the state) or what
  the treasury pays for this year (`purse`, its programme's reach as the book or a chip sets it), whichever is
  more. So with five years a child is at school from five to nine and may work from ten; with nine, from five
  to thirteen; with eleven, to fifteen, which takes the fifteen-year-olds out of the working ages too.
- **The university's students** (`state.md`, "The university") are counted out of the town they live in while
  they are taught, out of those aged 15 to 24, the youngest first.
- **A pension reaches** everyone over the pension's age while one is paid, as far as the programmes are paid in
  full (`purse.cut`, 1 unless the rate on pay passed its cap). Those it reaches leave work, all but 0.08 of them;
  those of 65 and over it does not reach (at 65 to 69 while the pension is paid from 70) work at 0.55. A
  pension's age set under 65 by hand reaches the working ages too. The pension's size does not enter: the book's
  pension is a fifth and then a quarter of the wage and never grows, and a rule on its size would only add a
  number nobody has measured.
- **One kind of household, and no separate rate for women.** The working ages work at one rate, the same for
  25 to 64 as for 15 to 24. A later rule of households could raise it as fewer children are born and as work a
  woman was hired for grows; this one does not.
- **The fit.** `working` is fitted once, on islands 1 to 8 under the game's rules, so that the island's share in
  work in 1900, counted this way, is the half it played: at 0.705 it is 0.4998 [0.491 .. 0.505]. (That is also
  the design's 0.70 for 25 to 64.)
- **Coming in.** From 1901 each town's share in work moves a tenth of the way a year from the half to its count:
  `(1 - b) * 0.5 + b * count`, with `b` a tenth in 1901 and whole from 1910. At the island the count is 0.5 in
  1900 by the fit, but towns differ by their ages, and the first pension, at 70, comes in with 1901; without the
  ten years a town's hands would fall by two to four per cent in one year.

The count is kept from the first year the ages are, so that the facts can say what it would have counted
before it acts (`work_share_1800` is 0.474, `work_share_1850` 0.478); nothing played reads it before 1901, and
its series are nought until then.

### Where it is read

In three places, each of which reads what it read before by an early return while the rule does not act:

- **The labour supply** (`econ::step_month`): each town's hands for hire are its share in work times its
  people, before the carters, the sinkers, the navvies, the menders, the teachers and the university are taken
  out of them (`who_works::supply`).
- **The two places that turn the works' hands into townsfolk** (`towns::update_urban` under the game's rules,
  `land::update_urban` under the port's): a town's townsfolk are `0.04 + 0.8 * works' hands / hands` of its
  people, and `hands` is now its people in work (`who_works::hands`). A works' hand brings a household, and
  with fewer of the people at work each hand brings more of it.

**Nobody is counted twice.** The university took its students out of the hands at the working ages' rate
(`university::take_hands`). With the rule acting it holds out only the part of them still reckoned as the
half, `1 - b`, and none from 1910, since the count has them out of work already
(`who_works::students_held`). The teachers and the university's staff are hired out of the people in work,
as before: they are at work.

### What the purse does

Nothing new: it takes its rate on the pay there is, and its budget balances by its rule (`state.md`). With
fewer people in work and as many pensioners and pupils, the rate rises: 0.105 in 1975, 0.135 in 2000 and
0.164 [0.151 .. 0.173] in 2050, against 0.100, 0.122 and 0.142 with the rule off.

## Its numbers

| name | game | what it is |
|---|---|---|
| `who_works.enabled` | on | the switch; needs `ages.watch`, without which the half stands |
| `who_works.child` | 0.25 | of those aged 10 to 14 not at school, the share at work |
| `who_works.working` | 0.705 | of those aged 15 to 64 not at school or the university, the share at work: fitted |
| `who_works.old` | 0.55 | of those 65 and over whom no pension reaches, the share at work |
| `who_works.pensioned` | 0.08 | of those a pension reaches, the share who work on |

`who_works::FROM` (1901) and `who_works::BLEND_YEARS` (10) are constants. The four shares are the design's
table, with one rate for all the working ages where it had a rising one for 25 to 64.

## What sixteen islands show

Islands 1 to 16 to 2050, the commands at the top; the mean, with [least .. greatest] where it says something.

| | 1900 | 1925 | 1950 | 1975 | 2000 | 2025 | 2050 |
|---|---|---|---|---|---|---|---|
| in work, of all people (off: 0.500) | 0.500 | 0.484 | 0.484 | 0.465 | 0.441 | 0.423 | 0.416 [0.411 .. 0.419] |
| of those under 15 | 0.079 | 0.065 | 0.017 | 0 | 0 | 0 | 0 |
| of those 15 to 24 | 0.705 | 0.700 | 0.695 | 0.606 | 0.570 | 0.528 | 0.527 |
| of those 25 to 64 | 0.705 | 0.705 | 0.705 | 0.705 | 0.705 | 0.705 | 0.705 |
| of those 65 and over | 0.550 | 0.106 | 0.080 | 0.080 | 0.080 | 0.080 | 0.080 |
| people in work, thousand, on and off | 336, 336 | 412, 425 | 474, 483 | 486, 512 | 480, 533 | 483, 554 | 502, 581 |
| people, thousand, on and off | 672, 672 | 851, 849 | 978, 967 | 1,044, 1,025 | 1,091, 1,066 | 1,141, 1,109 | 1,206, 1,162 |
| made a head, on and off | 36.8, 36.8 | 53.9, 55.5 | 74.9, 77.0 | 78.8, 82.8 | 78.0, 86.3 | 76.5, 86.8 | 85.7, 97.6 |
| made a person in work, on and off | 73.5, 73.5 | 111, 111 | 155, 154 | 169, 166 | 177, 173 | 181, 174 | 206, 195 |
| linked real wage, 1750 = 1, on and off | 5.93, 5.93 | 9.22, 9.25 | 13.0, 13.1 | 14.6, 14.4 | 15.4, 15.2 | 15.3, 15.1 | 16.4, 15.7 |
| the first city's townsfolk, thousand, on and off | 48, 48 | | 146, 149 | | 185, 172 | | 197, 172 |

(What is made is `made_linked`, in 1750's money a month, over the people and over the people in work, which with
the rule off are half the people. The first city is `townsfolk_largest_Y`.)

- **The share in work stays between 0.41 and 0.50** on every island in every year: its least yearly mean
  before 2000 is 0.466 (1928), and in 2050 0.411 on the least island.
- **Children leave work in two steps**, with the book's school acts: from 0.079 of those under 15 to 0.016 by
  1928, once the nine years of the act of 1925 are reached and only the fourteen-year-olds may work, and to
  nothing from 1952, with eleven years. So by about 1930 four in five of the children who worked in 1900 do
  not.
- **The old leave work as the pensions come**: 0.55 of those over 64 in 1900, 0.31 in 1901 when the pension
  at 70 is paid, 0.13 in 1920 and 0.08 from 1940, the pension being paid from 65 from 1917 [1910 .. 1928].
- **What each takes, in hands.** School takes 15,600 hands in 1950 (3.3 per cent of the people in work),
  28,500 in 2000 (5.9) and 31,400 in 2050 (6.3); the university 1,500 (0.3), 9,300 (1.9) and 16,700 (3.3); the
  pension 42,400 (8.9), 85,100 (17.7) and 109,600 (21.8). The pension costs most because the old are many and
  most of them would otherwise work at more than half time.
- **The island makes less a head and more a worker.** In 2050 it has 14 per cent fewer people in work and
  makes 12 per cent less a head, 5 per cent more a worker. What a wage buys is a little higher (16.4 against
  15.7 times 1750's), and the growth asks are met as they were.
- **People and the first city.** There are 1 to 4 per cent more people from 1950 (1.1 in 1950, 3.8 in 2050;
  not traced, though what a wage buys is a little higher and births answer to income), and the first city holds
  0.304 of the townsfolk in 2050 against 0.275: with fewer hands a works' hands bring more townsfolk, and the
  largest town's works are many. The island's share in towns hardly moves (0.541 against 0.540 in 2050).

## In the viewer

Under the people by age, at the foot of the island half of the Markets panel, a chart **Who works**: the share
in work of all people and of each age (under 15, 15 to 24, 25 to 64, over 64), year by year from 1901, with
their figures in the year on the clock (`crates/isostate-viewer/src/ages_chart.rs`); `print` on that half adds
a `who works:` line. The Rules tab's chips for the years of school, the pension's age, its share and the
university's share say that they take hands out of work, and the rule's own switch has a chip of its own
("Who works follows age"), which may be thrown in a running island: thrown on, it counts from the next year's
end; thrown off, the hands are half the people from the next month.

## Tests

`tests/who_works.rs`: off is off (the rule keeps nothing and writes no series) and to 1900 nothing moves (every
series both runs keep is the same to the bit to December 1900 on island 5, with the news; the count of 1900 is
within 0.015 of the half; 1901 plays a tenth of the way to the count); nobody is counted twice (on island 5 to
1912, no age has more at work than it has people, the count's students are the university's, and once the rule
is in whole the university holds no student out of the hands, while its staff are still hired). In the module,
the count with school, students and a pension, set against the count with none of them: what each takes, added
back, is all that would work.

## What worries me

- **The old leave work at once.** A pension that reaches a person takes them out of work whatever it pays, so
  the work of the old falls from 0.55 to 0.08 within twenty years of 1900, where in history it took most of the
  century. The book's pension at 65 comes in 1917 (the design dates it 1945 to 1948 for Britain), and with it
  nearly all of the old's work goes.
- **One rate for the working ages** leaves out what history's rise in work came from, women's: in 2050 the
  island has 0.42 of its people in work, where history's rich countries had about half. A rule of households
  would raise it.
- **The count is the year's at its start.** A town's share is set at the year before's end and held for the
  year; a town founded in the year takes the island's.
- **The jobs a town opens** are still weighed against half its people where the first city's rule pulls
  movers (`city::pull`), and a few facts (`carters_share_of_hands_Y`, the check of more at work than people)
  still divide by half the people. None of them is played except the pull, and the brief named three places.
