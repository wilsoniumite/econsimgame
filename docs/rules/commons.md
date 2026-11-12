# The common fund that carries its debt

Not in the Python. `crates/isostate-core/src/commons.rs`, switched by
`commons.carry_debt`: off under the port's rules, on under the game's. The
whole story, with the island that showed the fault and every sweep behind
the rule, is in [mines.md](mines.md), "Under the commons"; this page is the
short account the convention asks for.

## What it does

Under the commons scenario the rent of land nobody owns goes into a common
fund, and each month the fund shares out nine tenths of what it holds, by
head, to every household on the island. The port shares out nine tenths
whatever the sign, so a fund in debt takes money from every household by
head, whether the household has it or not.

With `carry_debt` on, what the fund has to share out is what it holds and
never less than nothing (`commons::fund_to_share`, read in one line of
`econ::step_month`). A fund in debt pays no dividend until its rents have
made the debt good, and asks nobody for anything, as a town's landlords do
under the other regime. Under the landlords the fund never holds a penny,
so the switch does nothing there. It has no numbers.

## Why

A farm's bad month is small beside the island's rents, and the port's fund
is hardly ever in debt. A working's can be very large: on island 3 a town
founded on a coalfield at the end of 1750 hires more hands than it has at a
wage that doubles every month, the fund owes millions, and the levy empties
every purse on the island until the money itself goes below nothing.

The rule came in with the mines and first hung on `mines.enabled`. It has
had its own switch since, because the levy is a hazard of the commons and
not of the mines: the port's own mines with the machines switched on run
into it too.

## What the sweeps showed

Under the commons, sixteen islands with six families
(mines.md, "On a switch of its own"):

- the machines alone over the port: with the levy island 3 stops with "the
  island's money went below nothing in 1791-2"; with `carry_debt` all
  sixteen run to 1900;
- the game's rules with the mines off, the levy against the fund that
  carries its debt: none failed, and nothing moved clear of the spread
  between islands. The two part in the warm-up, when the fund is in debt
  for some months on most islands.

The game as played, under the landlords, did not move when the rule was
given its switch: eight islands with six families give the same value for
every one of 446 facts as before, and under the commons the same for every
one of 447, since the game's mines were on and the rule with them.

## What worries me

The rule treats the symptom. The fault behind it, a town that hires more
hands than it has, is the labour market's, and happens under both regimes
(mines.md, "When a working hires more hands than its town has").
