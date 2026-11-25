# The state's purse: a treasury that balances, a rate on pay, schools and pensions

Added on 20 November 2026. Not in the Python. `crates/isostate-core/src/purse.rs`, switched by
`state.enabled`: off under the port's rules, on under the game's, on the default book of acts, none of which
can be met before 1900.

**The proof.** With the purse on and nothing in it (`state.book=none`: no act, so every rate and programme
stays at nothing), the run is the run with the purse off:

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set state.enabled=false --out off
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set state.book=none --out none --compare off
```

```text
of the 3929 facts both have, the seconds aside, 0 differ and 0 of those are starred
```

`tests/state.rs` holds the same on island 5 to 1906, series by series to the bit, with the news and the money.

**With the book**, on islands 1 to 16 to 2050: no run fails, no month fails the money check, the treasury
holds between 1.5 and 2.5 months of its spending at the end of every year from the first act, the standing
check is 12 of 12 inside on both sets of eight, every series of the played century is what it was to the bit
(1,208 island-years to 1900 in each set of eight, none differing, and the game's golden test holds its first
era), the three growth asks are met, and the working ages' years of school rise from five to eleven.

## What it does

The island had no state: no tax, no school, no pension. This is the floor of one, and the instrument panel for
the questions a state answers on it: who pays, who is paid, what a programme costs. It holds one purse, one tax
and two programmes, and since its second piece a third with a place, the university, in the island's seat
("The university", below).

**The treasury** is a purse like the households' or the merchants': `econ::money_total` counts it, the books
of account count it among who holds the money (`money.treasury`), and every penny that goes in or out of it is
a transfer with a town's households in the same month, so the monthly money check holds as it always has.

**The rate on pay** is taken where pay is banked. In the month's step (`econ::step_month`) the households'
income of a town holds nothing but pay until the dividends are added to it: the works' wages, the land's wages
with the sinkers of new workings, the carting of every kind, the navvies, the hired managers. The rate is taken
on that and on the teachers' own pay, and nothing else: no dividend, no pension, no new money is taxed. Nothing
on the island reads the wage after tax yet; households simply have less to spend (their budgets follow last
month's income), and a rule of hours that reads it is the design's, not built.

**The budget balances by rule.** At each year's end the rate for the next year is set so that the next year's
revenue pays for the programmes and leaves the treasury holding `reserve_months` (two) of its spending:

    rate = (next year's spending * (1 + 2/12) - what the treasury holds) / next year's pay

What it holds above the two months lowers the rate, and what it lacks raises it. There is no debt: in a month
when the treasury could not pay all it owes it pays the teachers first and the pensions with what is left, and
counts the month (`state_short_months`; none on sixteen islands). If the rate would pass `rate_cap` (a half),
every programme is cut in proportion so that a half covers them and the reserve, and the news says so
(`state_cut_years`; none on sixteen islands).

Next year's spending is reckoned as next year's pupils and pensioners at December's wages, scaled by what this
year's programmes at December's wages really cost over the year, and grown by the mean growth of pay over the
last ten years. A first form took twelve times December's cost grown by this year's growth of pay: on island 7
it held the treasury between 1.9 and 2.6 months to 1980 and then anywhere from nothing to 3.8, because
December's wage is not the year's (the season moves it) and a year's growth runs back the next. The scale
cancels the season and the year's noise, and the treasury then stays between 1.7 and 2.5 months on island 7
in every year.

**Schools.** The treasury hires each town's teachers, `school_hands` (a tenth) a pupil: a teacher to fourteen
and those who keep the rooms. They are hired as the canals hire their navvies: taken out of the town's hands
for hire before the month's wage is sought (`purse::take_teachers`, after the menders), at most a quarter of
them, counted at work, and paid the town's wage out of the treasury. A child is at school from five for as many
years as the programme reaches, so the pupils are counted from the ages' rows (`ages.rows`, as the last year's
end left them). A programme grows toward its law by at most `year_room` (0.15) of the law a year, what a trade
may add in a year (`works.year_room`), so a law of six years is reached in seven years and a rise from six to
nine in three.

*Why teachers and not a good bought from works.* The brief asked for schooling first as a good of its own,
made in every town by a kind of works and bought by the treasury, so that the making page, the "made here"
view and the Works tool would show it with no code of their own. It was tried against the code before anything
was written, and three things stand in its way, each the work of days: a good nobody makes stands at its
ceiling, forty times its base price, and the price rule keeps it there while the asking outruns the making
(`tree::price_gap`), so a buyer of a fixed quantity at any price, which the treasury is, would pay up to forty
times the schooling's cost for the years until works were built, and the rate would pass a half at once; works
are founded once a year, one of a trade in a town, by the town's savers and only where the method is known, and
their plant arrives over months, so "a school in every town" would want an act that makes the method known
everywhere and a market that founds a works in each town within a year; and the schools would be private works
paid the treasury's money as their dividends. So the treasury hires its teachers, as the brief allowed. The
schools are not shown by the making page, the "made here" view or the Works tool; the purse's chart and its
facts show them.

**The years of school a cohort gets** are those the treasury paid for while it was at school: the cohort that
turns fifteen this year had a year at each age from five to fourteen, each counted as far as that year's
programme reached it (`purse::cohort_years`, from the last ten years' reach), and the years after fifteen at
this year's reach. Where its families bought more (`ages::school_years`, the stand-in of two to five years by
the town's income) it keeps that. So a law brought in this year gives this year's fifteen-year-olds nothing
more; it reaches the working ages as its cohorts grow up.

**Pensions.** Each person over the pension's age is paid a share of their town's wage each month, into the
household purse of the town they live in. The pensioners are counted from the ages' rows.

**What they cost in work** (since 25 November 2026, [who-works.md](who-works.md)). From 1901 a town's hands are
counted by age (`who_works.enabled`): a child at school, a student at the university and a person the pension
reaches are out of work. So the three programmes cost the island hands as well as money: on islands 1 to 16 in
2050 school takes 31,400 hands, the university 16,700 and the pension 109,600, of 502,000 in work, and the rate
on pay that balances the budget on the fewer workers is 0.164 where it was 0.142.

**The book the game runs.** The acts the island meets by itself, the design's default book for these two
programmes and no others. Each is checked at the end of every year from 1900, met by the island and not
ordered by anybody, and is news in the January it comes into force:

| act | sets | condition |
|---|---|---|
| schools | 6 years of school | the towns hold more than 0.30 of the people |
| longer schooling | 9 years | a generation (25 years) after the first |
| longer still | 11 years | a generation after the last |
| a pension | at 70, a fifth of the wage | over-64s more than 0.055 of the people |
| the later pension | at 65, a quarter of the wage | households' baskets a head more than 12 |

`state.book = none` meets none of them.

**Her hands on it.** Three numbers, `state.school_years`, `state.pension_age` and `state.pension_share`, stand at
"the book" (below nothing) and are pinned at the head of the Rules tab as chips: The book, None, 6, 9, 11 or 13
years; The book, 60, 65 or 70; The book, None, a fifth, a quarter or two fifths. A chip sets the law from the
year it is set and overrides the book until it is set back to "the book"; set before 1900 it acts before 1900,
since it is not the book. A pension share set by hand where the book has no age yet is paid from 70.

## Its numbers

| name | game | what it is |
|---|---|---|
| `state.enabled` | on | the switch; fixed when the island is made (`edits::FIXED_AT_START`) |
| `state.book` | `default` | the book of acts (`default` or `none`) |
| `state.reserve_months` | 2 | the months of its own spending the treasury means to hold |
| `state.rate_cap` | 0.5 | the highest rate on pay; past it the programmes are cut |
| `state.year_room` | 0.15 | how far a programme may grow toward its law in a year, as a share of the law |
| `state.school_hands` | 0.1 | hands a pupil |
| `state.school_years`, `state.pension_age`, `state.pension_share` | -1 | the chips: below nothing is the book's |
| `state.university` | on | the university ("The university", below) |
| `state.university_share` | -1 | its chip: the share of a cohort taken in; below nothing is the book's |
| `state.university_years` | 3 | years a student is taught |
| `state.university_hands` | 0.15 | its staff a student |
| `state.university_rooms` | 60 | units of building work standing as rooms a student |

The book's conditions and figures are constants in `purse::book`, the design's.

## What sixteen islands show

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --standing --out on --compare off
isostate-core sweep --seeds 9-16 --rules game --families 6 --end-year 2050 --standing --out on-b --compare off-b
cargo run --release -p isostate-core --example purse -- --seed 7 --end-year 2050
```

The mean of islands 1 to 8 under the game's rules (islands 9 to 16 within a few hundredths of every row);
income is twelve months of the island's output, which now counts what the treasury bought:

| | 1925 | 1950 | 1975 | 2000 | 2025 | 2050 |
|---|---|---|---|---|---|---|
| the treasury, in months of its spending | 2.15 | 1.97 | 1.92 | 2.04 | 2.08 | 2.01 |
| the rate on pay | 0.055 | 0.075 | 0.097 | 0.117 | 0.128 | 0.129 |
| tax over income | 0.038 | 0.051 | 0.065 | 0.075 | 0.083 | 0.079 |
| schools over income | 0.015 | 0.019 | 0.020 | 0.019 | 0.019 | 0.018 |
| pensions over income | 0.023 | 0.032 | 0.045 | 0.055 | 0.063 | 0.061 |
| teachers over all hands | 0.022 | 0.029 | 0.031 | 0.030 | 0.030 | 0.030 |
| services' hands over all hands | 0.32 | 0.38 | 0.41 | 0.42 | 0.42 | 0.37 |
| pensioners | 56,300 | 90,300 | 133,700 | 178,200 | 209,400 | 227,200 |
| years of school, 15 to 64 | 5.27 | 7.34 | 9.64 | 10.75 | 10.98 | 11.00 |

Without the purse the years of school stand at 4.98 to 4.99 from 1950.

- **The acts.** The first schools act and the first pension are met at the end of 1900 on every island (the
  towns hold 0.44 to 0.48 of the people by then, and the over-64s pass 0.055 on all but one, which follows in
  1901); nine years in 1925 and eleven in 1950. The later pension comes in 1917 on the mean island (1910 to 1928
  on islands 1 to 8, 1913 to 1920 on 9 to 16): the island's households pass twelve baskets a head a generation
  before Britain's did, since the island of 1917 lives better than the Britain of 1945.
- **The treasury** ends every year from the first between 1.65 and 1.73 months at its least and 2.37 to 2.45 at
  its most on islands 1 to 8 (1.54 to 1.79 and 2.31 to 2.49 on 9 to 16); no year was cut and no month short.
- **Against the design's reckoning.** The design reckons schools at 0.03 of income and a pension at 65 of a
  quarter of the wage at 0.04 of income in 1960 and 0.07 in 2040: together 0.07 and 0.10. Here, about 0.057 in
  1960 (0.051 in 1950, 0.065 in 1975) and about 0.080 in 2040 (0.082 in 2025, 0.079 in 2050): four fifths of
  the design's, inside a third of it at both dates. Pensions come out close to the design's. Schools are about
  0.019 of income against its 0.03, because the design's figure is teachers with their rooms and books, and
  here the treasury buys no rooms and no books.
- **Who pays and who is paid.** The state's flows to a town's households (the teachers' pay and the pensions,
  less the tax) against their income before it, in the towns holding the older half of the people and the
  younger half: in 1925 +0.0037 and -0.0046, in 1975 +0.0003 and -0.0060, in 2050 -0.0026 and -0.0082 (islands
  1 to 8; both halves lean below nothing in a year the treasury grows its reserve). The towns with more pupils
  against fewer: +0.0025 and -0.0037 in 1925, nothing and -0.0059 in 1975. So the old towns and the towns full
  of children gain about a hundredth of their income against the others, every year. It is small because the
  tax is taken in the same towns the teachers and pensions are paid in: a state of one rate on pay moves money
  between the working and the rest within a town far more than between towns.
- **People.** 1.154 million in 2050 against 1.108 without the purse, on every island of eight (+4.2 per cent;
  +2.5 in 2000, +1.4 in 1950); the same on 9 to 16. The births rule reads each town's income, and the purse
  moves income toward the towns with more children and more old, where a shilling raises births the most. A
  household's baskets a head are a little lower (15.2 in 2050 against 16.1).
- **The first city.** In 2050 its townsfolk are 171,600 (0.272 of all townsfolk) against 155,100 (0.246)
  without the purse on islands 1 to 8, and 163,100 (0.261) against 170,700 (0.270) on 9 to 16: no steady move.
- **The growth after 1900** (the standing check's three asks): the linked real wage grows 1.25 per cent a year
  from 1900 to 1970 [0.94 .. 1.54] on 1 to 8 and 1.28 [1.09 .. 1.53] on 9 to 16; 2050 over 1970 is 1.10
  [0.86 .. 1.27] and 1.09 [0.94 .. 1.21]; the land's share of hands falls by 0.115 and 0.124. All met.
- **What moved clear on every island**: the years of school, and `mine_share_Y`, which is what is left of the
  hands when the land's and the works' are taken out, and so now holds the teachers (+0.026 from 1950).

## In the viewer

The island half of the Markets panel ends with "The state's purse" from 1900: the rate on pay, schools and
pensions over income as three lines on one scale, their figures in the year on the clock, and the months the
treasury holds (`crates/isostate-viewer/src/purse_chart.rs`). The Rules tab pins the three chips at its head,
in plain words. The acts are news, under "The state".

## The university

Added on 20 November 2026. `crates/isostate-core/src/university.rs`, switched by `state.university`: off under the
port's rules, on under the game's, and acting only with the purse on. A programme of the treasury with a place.

**The seat.** When the treasury's first programme begins (the first act is met, at the end of 1900 on every
island of sixteen), the island's seat is the town with the most townsfolk that year, and it stays the seat.
The town's card says so ("the island's seat, since 1900").

**Founded by the book.** The first university is founded in the seat five years after the first schools act
(1905 on every island), and a second in the largest other town by townsfolk two generations after it (1955).
Each founding is news, under "The state", in the January after ("The University of Glenwick"), and the town's
card names it with its year and its students. By hand, the sandbox's Works tool may found one in the town whose
district holds a square (`Edit::FoundUniversity`) and close any one (`Edit::CloseUniversity`); both wait for
the year's turn, as building a works does.

**Students.** At each year's end the year's intake goes to the nearest open university (by the cost of the
way): from every town, the book's share of its eighteen-year-olds (a fifth of its fifteen-to-nineteen group).
Those who are not of the university's own town move to it as migrants do, with their cash a head, their ages
and schooling, and their town's measure of how well townsfolk live. A student is taught three years
(`university_years`); while taught they are townsfolk of the university's town whatever its works (the town's
count of townsfolk takes them whole and reckons the rest as the port does), and they are not among its hands
for hire: each month the town's hands are less by its students at the working ages' rate of work, before the
wage is sought. After their years they are people of the town, and follow jobs like anyone.

**The book's share of a cohort** is history's for a country of Britain's institutions, drawn straight between
its years: the share of an age group entering higher education, from the Robbins report for 1900 to 1962 (all
full-time higher education, since the island's one university stands for all of it), the age participation
index for 1970 to 2000, and the participation rate after. The chip `state.university_share` (the Rules tab,
pinned with the purse's three: The book, None, a twentieth, a sixth, a third, half) overrides it.

| year | 1900 | 1938 | 1954 | 1962 | 1970 | 1980 | 1990 | 2000 | 2010 | 2020 on |
|---|---|---|---|---|---|---|---|---|---|---|
| share of a cohort | 0.012 | 0.027 | 0.058 | 0.085 | 0.14 | 0.13 | 0.19 | 0.33 | 0.45 | 0.50 |

**Years of school.** A cohort turning fifteen is given, beside its years of school, the share it is offered
times the years taught, as the mean of what its members will have. The share of those aged 15 to 64 who have
been taken in is a fact of its own (`uni_been_Y`), reckoned on the island's ages by the year each age was
eighteen.

**What it costs** falls on the rate on pay with the schools and the pensions: the budget reckons next year's
staff at December's wages with the teachers, and next year's rooms at December's prices.

- *Its staff*, 0.15 hands a student (`university_hands`: a teacher to ten students, and half as many again
  who keep its rooms, books and accounts), are hired as the schools' teachers are: taken out of the town's
  hands before the wage is sought (with its students, at most a quarter of what the teachers leave), counted
  at work, paid the town's wage. Two thirds of them teach and are counted in pay's reckoning
  ([pay.md](pay.md)) as judging, the rest as clerking.
- *Its rooms*, 60 units of building work a student standing (`university_rooms`), wearing out at a fiftieth a
  year, are built by the builders of its town: each month the treasury orders a twelfth of what it lacks and
  what wears out, as a railway orders its materials (served with the orders for plant, `tree.plant_waits`),
  pays the merchants the town's price for what comes, at most a sixth of what it holds in a month, and the
  books take the goods and the pay as a project's.

**Islands 1 to 16 to 2050, the university on against off.**

```bash
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --set state.university=false --out off
isostate-core sweep --seeds 1-8 --rules game --families 6 --end-year 2050 --standing --out on --compare off
cargo run --release -p isostate-core --example purse -- --seed 7 --end-year 2050 --every 10
```

No run fails, no month fails the money check, no year was cut and no month short; the treasury holds 1.53 to
2.54 months of its spending at the end of every year from the first act (its least 1.66 and its most 2.42 on
the mean island; without the university 1.69 and 2.40); the
standing check is 12 of 12 inside on both sets of eight, and every series of the played century is what it was
to the bit (1,208 island-years to 1900 in each set, no cell differing). The mean of islands 1 to 8, then 9 to 16:

| | 1925 | 1950 | 2000 | 2025 | 2050 |
|---|---|---|---|---|---|
| students | 930 | 2,130 | 12,800 | 21,400 | 22,300 |
| share of a cohort taken in | 0.022 | 0.050 | 0.33 | 0.50 | 0.50 |
| share of those aged 15 to 64 who have been | 0.009 | 0.024 | 0.146 | 0.320 | 0.452 |
| its staff over all hands | 0.0003 | 0.0007 | 0.0037 | 0.0061 | 0.0062 |
| what it costs over income | 0.0003 | 0.0005 | 0.0027 | 0.0040 | 0.0037 |

- **Years of school** of those aged 15 to 64: 7.41 against 7.34 in 1950, 11.2 against 10.8 in 2000, 12.4
  against 11.0 in 2050, on both sets of eight.
- **The rate on pay** is 0.140 against 0.129 in 2050 (1 to 8) and 0.142 against 0.129 (9 to 16); 0.076
  against 0.075 in 1950.
- **The seat.** The seat is the largest town in 1900 by construction. It is the largest in 2050 on 8 islands
  of 16 with the university (1, 2, 4, 8, 13, 14 and 16) against 6 without (4, 5, 10, 13, 15 and 16); its share of all townsfolk in 2050 is 0.218 against 0.180 (1 to 8) and 0.231 against 0.223
  (9 to 16). The largest town's share is 0.277 against 0.272 and 0.294 against 0.261; the second over the
  first 0.81 against 0.79 and 0.71 against 0.80. Island by island the moves are large and go both ways (the
  seat of island 6 holds 0.235 of townsfolk with the university and 0.068 without; that of island 5, 0.151
  against 0.258): some ten to twenty thousand students and their staff in a town of a hundred and fifty
  thousand are a push, and where people go after is the jobs' doing.
- **The pay gauge's schooled over plain** is 0.63 in 2050 on and off (1 to 8; 0.66 against 0.61 on 9 to
  16). The reckoning's hands of a class follow the work, not the schooling: more people who have been to the
  university do not make more schooled hands in it, and its staff are a hundredth of the clerks and judges.
  So the reckoning says nothing yet of what the schooled's growing numbers do to their pay.
- **The three growth asks** are met: the linked real wage grows 1.22 per cent a year from 1900 to 1970 (0.99
  to 1.53) and 1.27 (1.07 to 1.50); 2050 over 1970 is 1.13 and 1.11; the land's share of hands falls by
  0.103 and 0.101.

**On the map** a university stands as a works of the offices' kind (`works::site` plans a kind it has no
drawing for from its process: a building of storeys), on its town's centre square, its size by its students,
four hundred to a standard size. Only in a recorded run: the viewer puts it at the end of the record's works
when it reads the run. A drawing of its own is a later piece of art.

**What worries me.** The staff are paid the town's one wage, so a university costs a third of what one costs
in Britain (about 0.004 of income against 0.01). The intake is reckoned from the ages' rows, a reckoning
beneath the played people. A student counts as having been at the intake, so the share who have been leads
the degrees held by three years. The second university is founded by date, not by need.

## What is not built, and where each will hang

Of the design's state only the purse, the rate on pay, schools and pensions are here.

- **The other three taxes.** A tax on spending, as households, landlords and families settle (beside
  `hh_spend` in `econ::step_month`); a tax on returns with expensing, read by the hurdle in `firms::entry`,
  `firms::review` and the families' tests, and taken beside the works' dividend; a tax on rent's growth where
  rent passes (`econ::pay_cell_rents`, `pay_rent`). Each is a rate the budget would take before it sets the
  rate on pay.
- **The fund and its stake, and the dividend by head**: a second purse beside the treasury, paid by head
  through the line that pays the common fund's dividend (`fund_div` in `econ::step_month`).
- **Relief, health and public works**: programmes beside the schools and pensions in `purse::close_year`
  (their law, their reach, their cost at December's wages) and in `purse::month` (what they pay). Health would
  hire hands as the schools do; public works would order goods as a railway does.
- **Statutes as orders**: a law set by hand is today a chip (an edit of a number, logged as any edit is); an
  order that bites years after it is given, with the lag the design writes, is not built.
- **Rooms and books for the schools**, and a school as a works the map can show.
- **The hours a household works**, which is what the wage after tax would move.

## For the piece after this one

- **A programme** is its law (a number in `PurseState`, set in `purse::close_year` from the book or a chip),
  its reach (grown toward the law by `year_room` of it a year), its count by town (from `ages.rows`, set each
  year's end), its cost at December's wages for the budget (in `close_year`, beside `teach_dec` and `pens_dec`),
  and what it pays each month (in `purse::month`, between the treasury and `hh_inc`). Add its spending to
  `Month`, `YearSums` and `PurseYear`, tell the books (`accounts::note_state`), and give it a line in the cut.
- **How the treasury buys**: today only hands, at the town's wage. To buy goods it would add its requests to
  the month's `R` beside the projects', take what the fill gives, pay `price * got` into the town's merchants
  (`merch`) and tell the books in `accounts::Month` as the projects' goods are (see "When another mechanism
  moves goods or money" in [accounts.md](accounts.md)).
- **Where a town's hands can be held out of work**: `purse::take_teachers`, called in `econ::step_month` after
  `roads::take_menders` and before the wage is sought, takes a number of hands out of `supply` for each town;
  `purse::count_teachers` puts them back among those at work. Students would be taken out the same way and not
  counted at work.

## What worries me

- **Nothing reads the wage after tax.** The design's rule of hours is what would make a rate on pay bite on
  work; without it the tax only moves money between purses, and a rate of 13 per cent costs no work.
- **The schools are a third cheaper than the design's**, for want of rooms and books, and the teachers are
  paid the town's one wage: the design's schooled and unschooled hands are one kind here.
- **The later pension comes early**, in the 1910s, on the design's own condition of twelve baskets a head.
  A condition on the island's own growth would date it later; the design's is kept until its author says.
- **The tax reaches every town and the programmes too**, so the flows between towns are small; a state that
  taxes the city and pays the country would need the other taxes.
- **The ages are a reckoning.** The pupils and pensioners are counted from the ages' rows, which are reckoned
  beneath the played people ([ages.md](ages.md)); the full rule of ages will move them.
