# The news read as a reader would

A century of the island's news, read item by item the way a player meets it in
the News panel: the label and the month, the title, the sentence under it.
Nobody had read a whole run of it since the canals, the mines and the clock
that tells the month came in. This page says what it is like to read, decade
by decade; what it does well and should keep; every fault found, with the
item as it stands, the island, the year and the line of code that writes it,
sorted by how much it would spoil a reader's trust; and for each a suggested
wording or rule. No code was changed. The chronicle itself
(`docs/rules/chronicle.md`) is described on its own page; this is a reading of
what it and the other files write.

## How it was read

Eight islands under the game's rules with six families, the player's family
taking every default, on main as it stood on 12 November 2026 (a6ff0ec):

```bash
cargo run -p isostate-core --release -- run --seed N --rules game --families 6 --out DIR   # N = 1 to 8
cargo run -p isostate-core --release --example chronicle -- --seeds 1-8                   # the counts
```

Each run's news is in its `events.json` and in the header of its `run.bin`.
A short script printed it as the panel shows it (`ui.rs`, `news_panel`): the
label, the month as `clock::news_month` gives it, the title and the text, with
the weight, the "kept" mark and the town added for checking. Island 7 (the
Fairfax family of Melbourne, the island a new game starts on) and island 1
(the Wentworths of Oldford) were read whole, every item from 1750 to 1900:
1,046 and 1,035 items. Of the other six the major feed was read whole and the
rest sampled, and every kind of sentence the eight runs write (155 shapes of
title) was looked at at least once. Faults found by reading were then counted
over all eight runs with a second script, and 29 items of many kinds were
checked against the simulation's own record in `run.bin` (the towns' people
and prices, the harvest, the monthly series, the works' records, the railway
and canal books).

The eight runs write 8,016 items, 947 to 1,046 a run. The major feed, the
items the chronicle keeps, is 58 to 60 a run, three or four in every decade.

## What a century of news is like to read

Most of what follows is island 7, the island a game starts on. The labels and
months are as the panel shows them.

**The 1750s.** The first thing a player sees, before the first month is
played, is an old bloomery closing and a child born to the family. The first
kept item comes in the second year:

> Turning point · October 1751 — **Coal fires most of industry.** 58% of industrial heat now comes from coal.

It reads as history, but coal already gave 55 per cent of industry's heat in
January 1750: the island starts at the line, and this is the share crossing it
again. Then a strong story for anyone playing the Fairfaxes:

> Town and country · November 1758 — **Melbourne in decline.** Melbourne and its district had 26,362 people in 1750. There are 18,947 now, 28% fewer. Its people leave for Granness, where a household's income goes 24% further. Mallowwick has lost people too.

Every figure in it is right (the record has 26,362 and 18,947, and incomes at
Granness 1.24 times Melbourne's). Coal booms come one after another (Granness,
Brambridge, Kelside), the old bloomeries and smithies close two a year, and
the rival families bury their heads and name new ones in pairs.

**The 1760s and 1770s.** Inventions, then the canal age. The inventions are
the best-written items in the run:

> Invention · July 1775 — **Engine works opens at Melbourne.** Lydia Clegg builds steam engines to order. They are enormous and wasteful, and they run.

The first canal opens in January 1771, and from 1772 the canal schemes come
four at a time in December and open three years later, each with its length,
its cost of carting saved and the year it was begun. The family news is at its
thickest here: 54 family items in a mean 1770s, more than one a month, most of
them the rivals opening "their first" works of some kind, at returns no reader
will believe (below).

**The 1780s and 1790s.** Coke iron, powered mills and rolled rails; charcoal
dies, and is told as four market items over twenty years ending in "Charcoal
shortage" (output down 99 per cent against a trend of minus 49 per cent a
year). The family's branches extend the same five or six works every year in
the same sentence. The kept items read well together:

> Turning point · October 1790 — **Mills now make most of the cloth.** Powered mills turn out 64% of all cloth. Handloom weavers are paid less each year for the same work.
>
> Your family · December 1791 — **Henry Fairfax heads the family.** Henry follows Anne Fairfax, who died at 77 and is buried at Melbourne, the family's seat.

**The 1800s and 1810s.** Quieter: a dearth, a town passing ten thousand, canal
junctions, the locomotive in 1816, and the turning point a historian would
underline:

> Turning point · October 1818 — **More hands in workshops and mills than on the land.** Industry employs 35% of workers; farms, pastures and woods 34%.

**The 1820s to 1840s.** The railway decades. In the "All" list there are 13 to
17 railway items a decade: schemes in December, openings in their true month,
and once a year "5 branch lines open". The major feed keeps only the first:

> Railway · July 1823 — **The first railway opens: Kelside to Stratbank.** 35 km of track, begun in 1820. Freight between the towns now moves at a fraction of the carting cost.

After it, a reader of the major feed hears nothing more of railways in that
decade or the next two, except a coal boom in 1842 that mentions a branch to
the collieries.
Harvest and dearth agree with each other and with the record every time I
checked:

> Harvest · July 1830 — **Three lean years.** For the third year running the harvest falls short of an ordinary one: by 3%, by 4% and now by 14%.
>
> Dearth · September 1830 — **A dearth of grain.** Grain sells 21% above where the prices of the last years pointed, after a harvest 14% short. A wage buys 8% less than it did last year.

From the 1840s crucible steel closes works by works, two a year, until the
converter wins:

> Turning point · October 1854 — **Converter steel overtakes crucible steel.** Air-blown converters make 56% of the steel. What cost a week per pot now takes twenty minutes per charge.

**The 1850s to 1890s.** The inventions are over, and the century's second half
reads as weather and funerals. Every year has two closures (from the late
1860s nearly always "& Co.", the shops and trades of some town), a branch-line
item, the family's extensions and a rival's burial and successor. Every three
or four years output falls and is told twice, live and in hindsight:

> Hard times · November 1878 — **Output falls 8.8%.** Real output fell 8.8% this year. Nobody yet knows how long it will last.
>
> In hindsight · December 1880 — **The depression of 1877–1878.** Output fell 8.8% from its 1877 peak to the 1878 trough and took until 1880 to recover. Historians will call this a depression.

Island 7 has ten such falls between 1865 and 1898, island 1 has one of 19
per cent in 1894–96, island 8 "The depression of 1892–1898"; and nothing says
that growth has stopped, which is the story of these decades (output per head
peaks in 1879 on average). The towns' overtakings are the one thread a reader
can follow, and they go wrong here (below). Island 7 ends on an alarm in 1898
whose verdict never comes, as five runs of eight do.

## What it does well, and should keep

- **Its figures are right.** Of 29 items checked against the record, 25 are
  right in every figure, to the rounding they state: harvests and their best
  and worst districts, dearths and what a wage then buys, a town's people at
  its peak and now, how much further an income goes where its people went,
  the price of coal before and after a boom, the first railway's month and
  length, the depth and dates of a depression, the share of cloth from mills,
  when industry passes the land, a works' years and town, which works closed
  in a district and what they made. Three are wrong and one cannot be
  reproduced; they are faults 1 and 2 below.
- **Every item has a reason a reader can follow.** "Its people leave for
  Granness, where a household's income goes 24% further." "The clay they work
  up costs 28% more there than a year ago, and the price of bricks has not
  followed." "Coal is 18% cheaper there than in 1754: the new pits have
  outrun their buyers." This is what makes it read as history and not as a
  log, and an economist can check each one.
- **Stories are finished.** All 449 schemes promoted on the eight islands
  (canals, railways and junctions) open, and every opening says when it was
  begun. Every alarm but the last is dated in hindsight. Every harvest that
  causes a dearth is the one the dearth names.
- **The major feed is always three or four items a decade**, never empty and
  never a flood, and one place is the family's.
- **The writing.** The inventions, the turning points and the regional
  stories are written by someone who knows the period: "One horse on a
  towpath draws what sixty draw on the road. Coal owners begin to count what
  their carting costs them." "Millwrights go back to mending." Keep the
  voice.
- **The succession.** "Henry follows Anne Fairfax, who died at 77 and is
  buried at Melbourne, the family's seat" is one item where there were two.
- **Provenance.** Every item that is news because a number passed a line says
  which number and which line, so checking an item takes a minute.

## The faults, worst first

The order is by how much each would spoil a reader's trust: first what is
false, then what looks impossible, then what is weighed wrongly or left out,
then stories without a start, then sentences that read oddly, then floods.
Line numbers are those of a6ff0ec. Three cautions about where a mend can go:
the news written by `families.rs`, `observer.rs`, `rail.rs`, `sim.rs` and
`data/content.toml` is the port's and is held byte for byte by the golden
test, so a change of wording there has to be made under the game's rules only
(as the chronicle already rewrites the succession); `chronicle.rs` and
`ways.rs` have other workers in them tonight; and the counts below are over
the eight runs unless an island is named.

| | fault | how often | where |
|---|---|---|---|
| 1 | an overtaking names a town that was not in the place it lost, and the real changes of leader are never told | 4 of 26 wrong; 3 of 4 overtakings on island 7 | `chronicle.rs` 851–889 |
| 2 | a coal boom names the year of the latest line to a town, as "the railway that reached" it | 6 of 9 | `chronicle.rs` 709–711, `railways.rs` 171–177 |
| 3 | an invention's headline says a works opens, years before the first works of its kind | 24 of 55 five or more years later, up to 45 | `data/content.toml` (headlines), `observer.rs` 615–616 |
| 4 | turning points in the first years that are the island's starting state | 7 of 8 islands, 1751 to 1755 | `observer.rs` 53 (the crossing) |
| 5 | a grain glut after lean years blamed on "new capacity" | 11 of 24 gluts | `observer.rs` 495 |
| 6 | the same works closes twice; one name for many works | 19 told twice in a year; 121 names told more than once | `sim.rs` 490–510 |
| 7 | a new works "expects to return 1288 per cent a year" | 131 at 100 per cent or more, up to 1,920 | `families.rs` 1164, `chronicle.rs` 302 |
| 8 | sums of money with no £, in two styles | every founding and extension | `families.rs` 1164, `chronicle.rs` 302, 393, 402 |
| 9 | the family's own works closing is told lighter than a birth | 43 told, none kept | `sim.rs` 496 (weight 0.40) |
| 10 | hunger never kept; dearths with no cause; the late-century stall never told | 4 hungers, none kept; 13 of 66 dearths | `people.rs` 38–54, `harvest.rs` |
| 11 | the mines tell nothing | no item in eight runs | `mines.rs` |
| 12 | the railway age vanishes from the major feed after the first line | every island | `chronicle.rs` ranking (`per_kind`) |
| 13 | openings "begun in" a year in which nothing was told | 16 of 465; the first canal on 5 of 8 | `ways.rs` 1202, 1217 |
| 14 | "William Fairfax, is buried at Melbourne." | 6 | `families.rs` 1676 |
| 15 | "brings the trade home from" the family's own seat | 26 of 220 | `families.rs` 1713 |
| 16 | a method's name used as a noun: "its first forged rails", "The shops and trades near X ... its costs" | every rival founding and closure | `families.rs` 1162, `sim.rs` 505 |
| 17 | "The Calloway family's Calloway's Brickfield No. 2 closes" | 224 | `sim.rs` 496 |
| 18 | titles that start with a figure: "3 branch lines open" | 423 | `ways.rs` 784, `peopling.rs` 220, `observer.rs` 538 |
| 19 | "the country" and "the island" for the same place | 125 and 277 | `observer.rs` 38, 422, 455, 616; `chronicle.rs` 481 |
| 20 | "The slump of 1764–1765 ... Historians will call this a recession." | 112 of 139 verdicts | `observer.rs` 585–588 |
| 21 | small things: "Extended, for 3,307 in all:", "11th" after "tenth", months that mean nothing, a works told twice on its first day, rivals' deaths told twice, "Coal, Machinery, Rails and Charcoal" | see below | various |
| 22 | floods in the "All" list: family items 41%, closures 19% | every run | `families.rs`, `sim.rs`, `chronicle.rs` |

### 1. Overtakings that name the wrong town

> Town and country · November 1858 — **Kelside overtakes Melbourne as the island's first town.** Kelside and its district have 47,471 people to Melbourne's 43,542. In ten years Kelside has grown by 21% and Melbourne has shrunk by 11%. Kelside supplies 27% of the island's rails.

Island 7. Kelside is first, but Melbourne is fourth: Stratbank has 46,046 and
Glenbrook 45,196, and Melbourne had been third or lower since 1856. Three
years later:

> Town and country · November 1861 — **Stratbank overtakes Melbourne as the island's second town.**

Melbourne is again fourth, behind Glenbrook (45,799 to 41,216). And the first
of island 7's overtakings, in 1754, has Brambridge overtake Melbourne as first
when Melbourne had already fallen to fourth behind Sandmere and Glenbrook. On
island 5, "Brackbourne overtakes Ivelwater as the island's second town" (1851)
names a town that was third. Leaving the newcomer aside, the town said to lose
first or second place was not in it, in the year or three years before, in 4
of 26 overtakings; in five more it had slipped to third by the time the story
was told. And the changes that mattered most on island 7 are not told at all:
Glenbrook led the island from about 1780 to 1800, and Melbourne, the player's
own seat, from about 1804 to 1852. A player reads "Brambridge first" in 1754
and next hears of the first place in 1858, as Melbourne loses it.

Why: the chronicle keeps the two places "as last announced" and compares each
holder only with the largest challenger (`chronicle.rs` 866–870). A holder
that sinks below other towns keeps its seat until someone passes it by five
per cent for three years; when the first place changes hands the old first is
seated second whatever its rank (884); and when a founding redraws a leading
district the places are re-seated silently (858–863), which is how Glenbrook
and Melbourne came to lead unannounced.

*Suggested rule.* Rank by the true order each year: a story when the town
first in the order has been first, by the margin, for three years, naming the
town it displaced from that place in the true order, not the seat. A town that
takes first place after a silent re-seating is a story too: "Melbourne is now
the island's first town" (with the same reasons the overtaking gives).

### 2. "The railway that reached X" in the wrong year

> Town and country · November 1832 — **Coal boom around Garside.** The pits around Garside raise 1.7 times the coal they raised in 1827, and 20% of all the island's. The railway that reached Garside in 1829 carries it away. Coal is 9% cheaper there than in 1827: the new pits have outrun their buyers.

Island 7. The railway reached Garside in August 1827, when the line from
Brambridge opened, and the News panel said so at the time ("Railway opens:
Brambridge to Garside"). 1829 is the second line, from Sandmere. On island 1
the boom around Coldbank (1836) dates the railway to 1836, though the island's
first railway of all, Oldford to Coldbank, opened there in March 1835. Six of
the nine coal booms that name a railway give a later year than the first line
since the boom's base year. `railways::line_for_coal` (`railways.rs` 172) walks
the promoters' book from the end and returns the latest line that touches the
town; `chronicle.rs` 711 calls it "the railway that reached".

*Suggested wording.* Take the earliest line since the base year, or keep the
latest and say what it is: "The railway reached Garside in 1827, and a second
line, from Sandmere, in 1829; they carry it away."

### 3. An invention's headline says a works opens, decades before one does

> Invention · July 1785 — **Engine works opens at Coldbank.** William Lister builds steam engines to order. They are enormous and wasteful, and they run.
>
> First output · November 1830 — **Oates Engine Works founded.** The first engine works business in the country sets up near Ashside.

Island 1: forty-five years and another town apart. On the same island "Rails
rolled at Ulford" (1789) is followed by the first rolling mill near Oakbridge
in 1830. Of the 55 inventions on the eight islands whose kind was ever built,
24 have their first works five or more years later and 20 in another town;
island 3's steel rails (1891) are never built at all. The two items cannot
both be true as a reader takes them: the headline says a works opened, the
later item says this is the first. (The invention is a discovery of how; the
first works is the first business built to do it at a profit. The model is
right; the words are not.)

The headlines are in `data/content.toml` (the engine works' at line 269), the
port's file, so a change belongs in the game's own content; the first works
is written at `observer.rs` 615–616.

*Suggested wording.* Headlines that say what was shown, not what opened:
"Steam engines built to order at Coldbank" rather than "Engine works opens";
and a first works that says how long the idea waited: "Oates Engine Works
founded. The first engine works built to make money sets up near Ashside,
45 years after William Lister showed it could be done at Coldbank."

### 4. Turning points that are where the island starts

> Turning point · October 1751 — **Coal fires most of industry.** 58% of industrial heat now comes from coal.

Island 7, and kept. Coal gave 55 per cent of industry's heat in January 1750
and 46 per cent in December, so the share was wandering about the line the
island starts on, and the item marks it crossing back. Seven islands of eight
tell it between 1751 and 1755 and six of them keep it: it is the first kept
item a new player reads. (Its companion, "Coal now heats more homes than
wood", is a truer change: the share of household warmth starts at 0.45 and
crosses 0.55 a year or two later. Two islands keep that one instead.)

*Suggested rule.* A share that was within the band (above 0.45) in the first
year has no turning point: it began there. Or say so: "Coal already fires
most of industry, and its share grows."

### 5. A good harvest after lean years told as new capacity

> Markets · November 1831 — **Grain glut.** Grain output rose 28% this year against a trend of -1% a year, while prices fell 23%: new capacity is flooding the market.

Island 7, kept, the year after "Three lean years" and a dearth. The fields did
not grow; the weather came back. Eleven of 24 grain gluts follow a short
harvest or a dearth within two years. This is R24 in the backlog, seen from
the reader's side. `observer.rs` 495.

*Suggested rule.* As the chronicle page proposes: no steep item on a good in a
year its harvest was news, and for grain after a short crop, "The harvest
comes back: grain output is up 28% on the lean year before."

### 6. The same works closes twice

> Closure · November 1764 — **Crowther's Forge closes.** The smithy near Mallowwick, open since 1749, can no longer cover its costs.
>
> Closure · November 1764 — **Crowther's Forge closes.** The smithy near Mallowwick, open since 1749, can no longer cover its costs.

Island 7, two items, word for word. The record has two works called Crowther's
Forge on the same cell, opened in 1749 and closed in 1764 at the same size:
twins. Island 1 opens its news with two "Wilmere Bloomery closes" in November
1750, and closes "Ulford Brickworks" in 1763, 1771 and 1852, each time "near
Oakbridge". Over the eight runs 19 closures repeat a title already told that
year and 121 names are told closing more than once. In the record more than
half of all works share their name with another (1,188 of 2,223 on island 1),
and 156 on island 1 and 190 on island 7 are exact twins (same cell, method,
year opened and year closed), most of them works the island starts with. One round of closures names five works it counts among six
brickworks: they are all "Mallowwick Brickworks", standing near Sandmere.
`sim.rs` 490–510 writes the item; the names come from the works' naming.

*Suggested rule.* In the news, a works whose name another standing works
shares takes its place or its number: "Crowther's Forge, the larger of two of
that name near Mallowwick, closes"; two that close the same year and share a
name are one item: "Both of the Crowther's Forges near Mallowwick close". The
twins themselves are a question for the works, not the news.

### 7. Returns no reader will believe

> Your family · December 1767 — **The Fairfax family sets up in iron and steel at Brambridge.** Fairfax's Rail Forge is the first works the family has had in that trade there. It is built for 31 and expects to return 1288 per cent a year.

Island 7. Rivals the same year: 1,383 and 1,108 per cent. Millwrights in 1788:
1,141 and 1,195. Crucible steel in the 1770s: 200 to 300. Over the eight runs
131 founding items expect 100 per cent or more, the highest 1,920. They are
the investor's own sums (`f_reason.roi`) for a tiny works, and true to the
model, but no reader can take a forge that pays for itself thirteen times a
year as anything but a mistake. `families.rs` 1164 (rivals), `chronicle.rs`
302 and 385 (the family's own).

*Suggested wording.* Above 100 per cent, say it in words: "It is built for
£31 and should pay for itself within a month." Or leave the return out and say
what the money buys.

### 8. Sums of money with no £, in two styles

The family's own items say "built for 1,766" and "More capacity is ordered for
499,612 in all" (island 1, 1890); the rivals' say "built for 3475". There is
no £ anywhere in the news, though the panels write £18k and £1.00m
(`ui.rs`, `money`). A reader cannot set "98,230" for a brickfield (island 1,
1894) against anything. `families.rs` 1164 writes `{:.0}`; `chronicle.rs` 302,
393 and 402 write thousands with commas.

*Suggested wording.* The viewer's `money` everywhere: "built for £1,766",
"£500k in all".

### 9. The family's own losses weigh less than a birth

> Your family · November 1856 — **The Fairfax family's Fairfax Steel No. 11 closes.** The crucible steel near Melbourne, open since 1841, can no longer cover its costs.

Island 7: a works 53 times its trade's usual size. It weighs 0.40, as every notable closure does (`sim.rs` 496);
a child born to the family weighs 0.50 and a child turning fourteen the same.
Forty-three of the family's own works close in the eight runs and none is
kept; "Fairfax Mill No. 7 closes" (1870), a mill built in 1859 for 28,376,
goes the same way. A player will remember losing a mill longer than the
births.

*Suggested rule.* Under the chronicle, the family's own closure of a works at
several usual sizes weighs as its founding did (0.60), with what it cost and
what it made: "Fairfax Mill No. 7 closes. Built at Kelside in 1859 for £28k,
it can no longer cover its costs."

### 10. Hunger never kept, dearths without a cause, and the stall never told

> Hunger · August 1848 — **Hunger in Uldale.** Bread runs short in Uldale: households get 69% of the food they need at the worst point of the year.

Island 1, weight 0.50, not kept; seven years later "Uldale in decline" (37 per
cent fewer people) does not mention it. All four hungers on the eight islands
go unkept, and island 2's "427 people leave the island", after a hungry year,
too (`people.rs` 38–54, `peopling.rs` 220). A history keeps a hunger.

> Dearth · September 1874 — **A dearth of grain.** Grain sells 27% above where the prices of the last years pointed. A wage buys 17% less than it did last year.

Island 1, kept. The harvest that year was an ordinary one (0.999 of an
ordinary year in the record); output fell 13.7 per cent the same year. The item
names no cause, and 13 of the 66 dearths say nothing of a harvest. A reader
assumes the weather.

And the largest story of the century's second half is told nowhere. The
README's own summary is that output per head peaks in 1879 and falls after;
the news tells eight separate slumps on island 7 between 1865 and 1898 and
each "took until" some year "to recover".

*Suggested rules.* Hunger at 0.7 at least, and a decline story that names a
hunger in its town's last ten years. A dearth without a short harvest says
what else moved: "... in a year output fell 13.7%". And a decade digest when
output per head ends a decade lower than it began: "A lost decade: output per
head is 6% lower than in 1880, after three slumps."

### 11. The mines tell nothing

`mines.rs` writes no news. In eight runs the only mention of a mine is a branch
line "to the iron mines near Cranbourne" (island 6, 1830). A pit head opening,
a royalty paid to a family, a mine hiring more hands than its town has (the
README's town that paid 8.6 times its neighbours' wage): none reaches the
reader. The coal booms are told from the districts' coal, not the workings.

*Suggested items.* The first pit on a coalfield or orefield ("Pits sunk at
Garside"), a working that becomes the island's largest, and the family's
royalties when they become a large part of its income.

### 12. The railway age vanishes from the major feed

The ranking allows one item of a kind a decade, and railway schemes and
openings are one kind, so the major feed keeps the first line and nothing
after it: on island 3 the major feed of the 1830s and 1840s, when the network
grew from 577 to 1,638 km, holds no railway item at all. The milestones
("The railway network passes 1,000 km") weigh 0.65 and compete with every
other milestone. A history of those decades would lead with the railways.

*Suggested rule.* A decade digest for the railways as for new towns, kept like
one: "The railway decade: 11 lines open, from 329 to 1,061 km. Every town but
Coldmoor is on the network."

### 13. Openings with no beginning

> Canal · January 1771 — **The first canal opens: the collieries near Brambridge.** 16 km of canal, begun in 1766. What the collieries send to Brambridge now goes by water at a fraction of the carting cost.

Island 7, kept, weight 1.0. Nothing was told in 1766. Branch canals and branch
lines are built without a scheme item, and on five islands of eight the first
canal is a branch ("the works near Melbrook", "the woods near Mallowbourne").
A town's own first line ("The railway reaches Elmwick ... The line was begun
in 1825") is the same. Sixteen of 465 openings name a year in which nothing
was said. `ways.rs` 1202 and 1217.

*Suggested rule.* The first canal and a town's first line are schemes worth
telling: "Coal owners near Brambridge cut a canal to the town", in the year
begun. Branches can stay as they are, without "begun in".

### 14. A stray comma

> Your family · December 1783 — **William Fairfax dies at 65.** William Fairfax, is buried at Melbourne.

A member who was neither head nor running a trade (`families.rs` 1676: the
comma is in the format, before the empty description). Six in the eight runs,
all of them the player's own family, since only its members' deaths are told
when they held no post. This is the port's sentence, held by the golden
test, so the mend is the chronicle's: rewrite the item under the game's rules
as `chronicle::succession` rewrites the head's. "William Fairfax is buried at
Melbourne."

### 15. Bringing the trade home from home

> Your family · December 1769 — **The Fairfax family takes up coal and mining.** Eleanor Fairfax brings the trade home from Melbourne.

Melbourne is the family's seat; Eleanor trained there. 26 of 220 such items
say it of the seat. `families.rs` 1713. *Suggested wording:* when the town is
the seat, "Eleanor Fairfax learnt it at Melbourne".

### 16. A method's name used as a noun

> Families · December 1774 — **The Neville family opens its first millwright at Glenbrook.**
>
> Closure · November 1755 — **Fenwick & Co. closes.** The shops and trades near Melbourne, open since 1749, can no longer cover its costs.

Also "its first forged rails", "its first charcoal burning", "its first
crucible steel", and "The forged rails near Sandmere ... its costs".
`families.rs` 1162 and `sim.rs` 505 put the method's label where a noun for
the works belongs. *Suggested wording:* a works noun for each method ("rail
forge", "millwright's shop", "charcoal ground", "crucible works", "shop"):
"opens its first rail forge", "The shop near Melbourne".

### 17. "The Calloway family's Calloway's Brickfield No. 2 closes"

224 closures of families' works say the surname twice (`sim.rs` 496). *Suggested
wording:* "Calloway's Brickfield No. 2 closes" with the family in the text, or
"The Calloways close their brickfield near Skelford".

### 18. Titles that start with a figure

"3 branch lines open" (`ways.rs` 784), "427 people leave the island"
(`peopling.rs` 220), "Shortages in 4 industries" (`observer.rs` 538): 423
titles. *Suggested wording:* the number in words below a hundred, as the
regional stories already do ("Seven works close around Sandmere"); "Three
branch lines open".

### 19. The country and the island

"Melbourne is the first town in the country with more than 10,000
inhabitants" (1808) and "Glenbrook is the second town on the island" (1815);
"Coke furnaces now make 64% of the country's iron"; "There are now 329 km of
railway in the country". 125 items say "the country", 277 "the island", of the
same place. `observer.rs` 38, 422, 455, 616 and `chronicle.rs` 481. *Suggested
wording:* "the island" throughout, as the game and the README call it.

### 20. A slump that historians call a recession

> In hindsight · December 1767 — **The slump of 1764–1765.** Output fell 3.2% from its 1764 peak to the 1765 trough and took until 1767 to recover. Historians will call this a recession.

112 of 139 verdicts (`observer.rs` 585–588). *Suggested wording:* one word for
each depth: "The recession of 1764–1765", or keep "slump" in both places.

### 21. Small things

- **"Extended, for 3,307 in all:"** (`chronicle.rs` 402): "Extended, for £3,307
  in all, at Melbourne one in iron and steel; ...". And the same item every
  year: island 1 tells "The Wentworth branch at Oldford extends a works.
  Wentworth Steel, in iron and steel: more capacity is ordered for 157" (or
  146 to 190) thirteen times between 1778 and 1794 (`chronicle.rs` 393). A run
  of the same extension could be told once, when it begins, and again only if
  it stops.
- **Ordinals.** "Marwater is the tenth town on the island with more than
  10,000 inhabitants" and then "Greyside is the 11th" (`chronicle.rs`
  1283–1291 spells only to ten). "Eleventh" and "twelfth" would do.
- **Months that mean nothing.** Every invention is dated July, every harvest
  July, every dearth September, every turning point October, every closure,
  market item and regional story November, and all family news December
  (`year + 0.5`, `+ 0.9`, `+ 0.95` across the files). Only railway and canal
  openings carry a true month. Since the clock tells the month, a reader takes
  "July 1785" as when the engine ran. Either a real month or the year alone
  for the kinds that have none.
- **One works told twice on its first day.** "First output · Beaumont Mill
  founded. The first powered mill business in the country sets up near
  Granness" and, a month later, "The Beaumont family opens its first powered
  mill at Granness. Beaumont Mill is built for 9098" (island 7, 1784): four in
  the eight runs. One item, the family's, saying it is the first on the island.
- **Rivals' deaths told twice.** "Thomas Neville dies at 72 / Thomas Neville,
  head of the family, is buried at Brambridge" and then "Georgiana Neville
  heads the family / The Nevilles are led from Brambridge": 469 such pairs.
  The chronicle joins the player's; the rivals' could be joined the same way.
- **Capitals mid-sentence.** "Coal, Machinery, Rails and Charcoal all moved far
  outside their trend this year" (`observer.rs` 525–538 joins the goods'
  labels).
- **A year said twice.** "8 km of branch canal opened in 1818" in an item of
  December 1818 (`ways.rs` 794).
- **"Rails slump" in 1765**, sixty years before a railway: the forged rails
  are for wagonways, and the reader does not know it (R27). "Wagonway rails"
  until the locomotive would tell them.
- **The run ends mid-sentence.** Five runs of eight end on "Nobody yet knows
  how long it will last." A last item at the run's end could close it: "The
  century ends in a slump: output is 7.8% below 1897."

### 22. Floods in the "All" list

The major feed is well kept; the "All" list, which is what a player scrolls in
a quiet year, is mostly two kinds. Per run, of about 1,000 items: family news
414 (41 per cent; 54 in a mean 1770s, more than one a month), closures 191 (19
per cent; 16 to 20 a decade from the 1870s, most of them "& Co." shops and
trades from the late 1860s), railway openings 95 and schemes 27, canal items
67; and among the family news, the rivals' burials and successors, about 120. Of the family news, the rivals' "first works
of a kind" and the branches' yearly extensions are the bulk. None of this is
wrong, but a reader looking for what happened on the island in 1876 finds
"Bramall & Co. No. 17 closes" and "Dent & Co. No. 10 closes" first. A
"Families" filter already exists; closures of shops and trades below a few
usual sizes could stay out of "All" as the branches' doings do.

## What was checked against the simulation

Twenty-nine items, island 7 unless named, each against `run.bin` (the towns'
people, people in the town itself, incomes and prices; the harvest by town;
the monthly series; the works' records; the railway and canal books).

| item | said | the record | |
|---|---|---|---|
| Melbourne in decline, 1758 | 26,362 in 1750, 18,947 now; incomes 24% further at Granness | 26,362 and 18,947; 1.914 / 1.544 = 1.24 | right |
| Brambridge overtakes Melbourne, 1754 | 27,415 to 22,070; Melbourne shrunk 16% | as said; but Melbourne fourth | figures right, place wrong (fault 1) |
| Coal boom around Granness, 1759 | coal 18% cheaper than in 1754 | 2.071 to 1.700, -17.9% | right |
| A bumper harvest, 1792 | 9% above; best around Cardale, 14%; grain 14% cheaper | 1.094; Cardale 1.138, the best; 0.675 to 0.581 | right |
| A dearth of grain, 1806 | after a harvest 4% short | 0.964 | right |
| Melbourne passes 10,000, 1808 | the first town to | 10,111 in the town, the largest | right |
| The first railway, July 1823 | Kelside to Stratbank, 35 km | 1823.54, Kelside and Stratbank, 34.97 km | right |
| Railway passes 250 km, 1831 | 329 km | 248 in 1830, 329 in 1831 | right |
| The depression of 1833–1834 | 10.8%; recovered 1837 | 1,953,022 to 1,742,519; above again in 1837 | right |
| Melbourne passes 25,000, 1847 | | 25,523 | right |
| Kelside overtakes Melbourne as first, 1858 | | Melbourne fourth | wrong (fault 1) |
| Stratbank overtakes Melbourne as second, 1861 | | Melbourne fourth | wrong (fault 1) |
| Coal boom around Garside, 1832 | the railway reached Garside in 1829 | first line August 1827 | wrong (fault 2) |
| Fairfax Mill No. 7 closes, 1870 | open since 1859, a powered mill near Kelside | born 1859, closed 1870, Kelside, powered mill | right |
| The first canal, January 1771 | begun in 1766 | links begun 1766, opened 1771.04 | right |
| Seven works close around Sandmere, 1757 | six brickworks, largest Sowerby Clothiers; "of the 36" | seven, six brickworks and Sowerby Clothiers; 30 standing by the works' own town | right; the 36 counts the district and is not reproduced |
| Granness fastest-growing, 1750s | 17,586 to 21,888; the island 6% | as said; 5.2% by the monthly series, 5.6% by the chronicle's sum of districts | right |
| Mills make most of the cloth, 1790 | 64% | 0.636 in the last three months | right |
| Real output doubles, 1791 | 2.0 times 1750–54 | 2.03 | right |
| More hands in workshops than on the land, 1818 | 35% and 34% | 0.355 and 0.336 (1817: 0.350 and 0.357) | right |
| Kelside fastest-growing, 1770s | 11,831 to 23,135 | 23,135 in 1779 | right |
| Crowther's Forge closes twice, 1764 | | two works of that name, one cell, same years | right (fault 6) |
| Seven works close around Garside, 1899 | all rails; largest Neville Steel Rail Co. No. 7, 1894 | seven rail works, as named | right |
| island 1: Oldford overtakes Ulford, 1772 | 33,957 to 30,253; +17% and +2% in six years | as said | right |
| island 1: dearth, 1874 | a wage buys 17% less | real wage 2.998 to 2.485 | right (no cause given: fault 10) |
| island 1: Oldford passes 10,000, 1778 | | 10,076 | right |
| island 1: Uldale in decline, 1855 | 13,637 in 1847, 37% fewer | 13,637 to 8,626 | right |
| island 1: Wilmere Bloomery closes twice, 1750 | | two works of that name, cells 2587 and 2467; a third told in 1773 | right (fault 6) |
| island 1: Ulford Brickworks closes, 1852 | the third time | five works of that name, all near Oakbridge | right (fault 6) |

## What I did not do

- No code was changed, and no wording was tried: each suggestion above is a
  proposal for whoever next works in the file named, under the game's rules
  where the sentence is the port's.
- The viewer was not run; the panel's form was taken from `ui.rs` and the
  months from `clock.rs`.
- The twin works (fault 6) were counted but not traced: why the island starts
  with some 150 works that are exact copies of another is a question for the
  works, not the news.
- Six of the eight islands were sampled, not read whole; the counts cover all
  eight.
