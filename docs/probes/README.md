# Probes

Measurements of the simulation and of the viewer, each made on eight islands or more by a
helper on a branch of its own, and kept as written. None of them changes a rule: where a
page tried a cure, it was a prototype on its branch, and the mechanisms built from them have
their own pages under `docs/rules/`. They are of four kinds:

- **A probe** (`dx-...`) diagnoses something and designs a cure, with a rough prototype or a
  measuring example. The probe branches named in each (`night/dx-...` and `night/o-dx-...`)
  hold the measuring code and are not for merging; where a probe's example came onto `main`
  with its page, it is named below.
- **A trace with the books** (`tr-...`) follows something the sweeps had measured and nobody
  had explained through the books of account (`docs/rules/accounts.md`), by switching the
  suspected cause off and on. These are piece 12 of `docs/backlog/found-along-the-way.md`.
  Each has its measuring example under `crates/isostate-core/examples/`, which reads the
  island and changes nothing in it.
- **A table of sensitivity** (`sens-...`) moves every number a mechanism has as a setting to
  about three quarters and four thirds of its value, one at a time, and says what moves. They
  are for the pass over all the rules, piece 18 of the same page.
- **A reading**: the news read item by item as a player meets it.

And one page is the baseline of the scorecard, with the floor of chance that the pass over
the rules is read against.

Every page from 12 November was done by one helper and checked by a second, who ran its
numbers again and mended what did not hold in place.

## The first probes, of the port (10 November 2026)

Diagnoses of the simulation as it stood at commit a5ea2ea, on 8 to 16 seeds, each with a
rough prototype of a cure. The game's first mechanisms were built from them. Line numbers in
them are those of a5ea2ea.

- `dx-firms.md` (a probe): why the island ends with thousands of tiny works on a sixth of the
  land.
- `dx-towns.md` (a probe): why towns do not grow on the map, the churn of town ground, the flat
  urban share.
- `dx-growth.md` (a probe): why history stops about 1800, the machinery famine, the late
  methods never built.
- `dx-cycles.md` (a probe): where the price swings and the farm zigzag come from.
- `dx-rail.md` (a probe): why the railway network stops, and what railways do to the economy.

## Later probes

- `dx-combines.md` (a probe, of the game's rules at 63f7b46, 11 November 2026, seeds 1 to 8
  with six families): what a works holds (nothing but plant and cash), how the chains lie on
  the map, a design and a small prototype of combined works behind `combines.enabled`, and what
  a railway or navigable water does for a works. A second helper then went through it as its
  critic (branch `night/dx-combines-review`): the measurements stood, some readings of them were
  corrected in place, and its last section has the verdict (build it changed: the record's row
  now; combined works as a concern with books for each stage, not as the prototype has them).
- `dx-smooth.md` (a probe, of the viewer at cd9b41c, 11 November 2026): where the game's frames
  stall (a year's scenery handed to the renderer whole, a third of a second each year), and what
  keeps the view moving whatever the simulation is doing. Its branch, `night/smooth-probe`, adds
  a frame recorder to the viewer (`src/frames.rs`) that is meant to be merged. A critic ran it
  again and rewrote its stages on `night/smooth-probe-review` (section 11): the first stage is
  the year's meshes handed over one a frame, half a day's work, with a brief to build from. That
  stage, with the frame recorder, was finished on `night/o-finish-smooth-1` and landed on
  12 November (`src/handover.rs`, `src/frames.rs`); section 12 says what was built and measured.
- `dx-land-view.md` (a probe, of the viewer against the core's second island, as both stood on
  `night/core` at 859b680, 12 November 2026): what it takes for the viewer to draw the land
  made by water (`docs/rules/land.md`). What the core hands over (a grid of heights every
  500 m, river lines with width, level and flow, lakes, wet ground), how wrong today's picture
  is on it, what the viewer invents that becomes untrue, and a design in stages from "the land
  reaches the viewer" to "scarps and canyons made to read", with roads and bridges on real
  rivers between. Its branch, `night/dx-land-view`, adds a measuring example to the core
  (`examples/land_view_probe.rs`) and a probe to the viewer (`src/landview.rs`, behind
  `ISOSTATE_LAND_VIEW`) that draws the core's ground, rivers and lakes with today's code. A
  critic went over it on `night/o-land-view-critic`: the measurements hold to the digit on the
  merged build, four statements were corrected in place, the twelve standard pictures are
  untouched with the probe compiled in, and the verdict, in its last section, is to build it
  changed: the cells' middles moved off tidewater, the lakes masked and the relief held at three
  times in the first stage; the railways second; bridges chosen from the roads; the heights put
  into metres only after the works' site planner and the towns' second pass land. The critic's
  example, `land_view_critic`, stays on its branch.
- `dx-save-state.md` (a probe, of saved games at a6ff0ec, 12 November 2026, islands 7, 3 and 5
  with six families): what the island's state is and weighs (56 MB in 1750 and 96 MB in 1900 on
  island 7, nearly half of it the books of account), what changes every month and what at the
  year's end, and a rough picture of the whole run written to bytes and read back that goes on
  as the same game to the last digit. A design in stages for keeping such pictures beside the
  log of orders, so that a century loads in tenths of a second and a game can outlive a change
  of the rules. Its example is `examples/save_state.rs`.
- `dx-record.md` (a probe, of the record and the live hand-over at 11dbe7d, 12 November 2026,
  islands 7 and 3 with six families, and 7 again to 2050): every array of `run.bin` and every
  field of the live hand-over, which of them the viewer reads (34 of 63 arrays) and what is lost
  on the way to the picture (a fifth of the land's ground outside the two classes kept, rent
  clipped in town centres, no traffic, no owners in a recording, a works' capacity and nothing
  else); what each missing thing would weigh, by year, raw and gzipped, and in time to write and
  to read; and a next layout in five stages, the first a test that holds the record and the
  works' rows the site planner waits for. Its example is `examples/record_probe.rs`.
- `dx-inspector.md` (a probe, of the books of account at 11dbe7d, 12 November 2026, island 7
  with six families): a market's trades as a flow. Three towns (a farm town, the iron and coal
  town, the largest) read from the books in 1800 and 1880: what each makes, buys in and sends
  out, from and to which towns, at what price and carting, who earns what, and which figures
  balance (all the books promise) and which do not (purses that move at the year's end,
  dividends out of cash, a flow's own price against the year's mean). Text mock-ups of the
  inspector with those figures (a town's card and sheet, a good in a town, one good across the
  island, one works), what the viewer must be handed (nothing from the record for a live game),
  and the order to build it. Its example, `examples/flows.rs`, prints every table.
- `dx-new-towns.md` (a probe, of the game's rules at 11dbe7d, 12 November 2026, sixteen islands
  with six families): where and when towns are founded (11.2 a seed, 4.6 of them from 1800,
  half of those quarry towns), why sites at wharves and junctions fail the founding test (carriage
  there is already cheap), and what a district's founding saves the island rather than its own
  producers (every town founded costs the island more carting than it saves). A prototype that
  founds by the island's saving behind `newtowns.enabled` founds nothing at the model's one
  freight rate; its verdict is not to build it as it stands, and to build first a margin before
  a district changes market, villages at pit heads, and wharves and stations as places with a
  charge. Only the page is on `main`; the prototype, its test and its example `new_towns.rs` are
  on `night/o-dx-new-towns-check`.
- `dx-harvest-month.md` (a probe, of the game's rules at 11dbe7d, 12 November 2026, eight
  islands with six families): a harvest that comes in a month. Today the farms' year is flat
  and grain saws for a year after a bad harvest; a prototype behind `season.enabled`, off under
  both sets of rules, brings the crop into store in August and September, and of its five forms
  the four that failed show that the price rule, written for grain that arrives every month,
  reads a store wrongly. The last form ends the saw and puts the dearth in the months it belongs
  in, but takes famine years from 0.4 a run to 4.2, nearly all in July. Its verdict is to build a
  harvest month with a price rule written for stores. Only the page is on `main`; the prototype,
  its test and its example are on `night/o-dx-harvest-month-check`.
- `dx-roads.md` (a probe, of the game's rules at a6ff0ec, 12 November 2026, eight islands with
  six families and sixteen for the sweep): who pays for the roads and bridges and who is paid a
  railway's fares. Today nobody: a road is worn into being by traffic alone, a river that is not
  navigable costs nothing to cross, and fares are carters' wages. A design from the period's
  own institutions (the parish, the turnpike trust, the bridge by its river's width, the company)
  and a prototype of the first three behind `roads.enabled`, off under both sets of rules,
  which came onto `main` with its example `examples/probe_roads.rs`: switched on, turnpikes are
  half as many, lenders get 5 per cent at the median and 113 bridges an island are built, with
  the carting bill and the wage inside chance. Its order of work starts with railway and canal
  companies that are paid their fares.

## The traces with the books (12 November 2026)

Each under the game's rules with six families, on islands 1 to 8 and on sixteen or more where
an answer was leaned on.

- `tr-rent.md` (at 295a379): landlords in debt for decades (the port's runaway wage, which the
  game's people rule took away), a family's share of town rent (it cannot go below nothing), how
  far `harvest::average_rent` is out (about twice the harvest's shortfall, the wrong way: the
  wages saved by foreseeing and the dearer grain together keep the rent as paid near an average
  year's at `foreseen` 0.5), and island 6's empty shelves (empty purses after a port line opened
  in debt; today mostly a January after a town takes a district, which moving the merchants'
  stock and their measure of sales with it would mend). Example `examples/trace_rent.rs`.
- `tr-carriage.md` (at 295a379): carriage, stocks and famine. Under the game's rules the
  railway's gain reaches the wage (output a head and the real wage both 8 per cent higher, through
  prices; about seven parts in ten to work, its usual share, and to the owners of land and
  houses twice theirs, with next to nothing to works' owners or merchants); the handling
  charge is a tenth to a seventh of the carting bill and a hand in a hundred; coal is never
  held long and timber hardly; and the famines the canals took away were
  the railway's: small towns handed districts larger than themselves, whose merchants' shelves
  stayed behind (`peopling::move_with_cells`). Example `examples/trace_carriage.rs`.
- `tr-money.md` (at 295a379): the issue of money (it grows about 1.77 per cent a year where the
  setting says 2, because the households' whole month's income is outside the count; issuing on
  the true total moves the price level, and through the freight rate fixed in coins, carriage
  and the founding of towns), the extreme settings that fail the money check (none leaks: the
  money goes below nothing, then rounding trips the check), the Python's ten oddities under the
  game's rules (none matters there), and large harvest shocks (the same rule). Example
  `examples/money_trace.rs`.
- `tr-works.md` (at 295a379): growth stops in the 1870s because the machines' quality reaches
  its ceiling (`machines.quality_cap`, about 1870): lifted, growth goes on to 1900 on every
  island of twenty-four; lowered, it stops sooner. The observer's fall after 1879 is its 1750
  price basket. Kept town ground still costs the wage 2 to 5 per cent over some twenty years and
  one works in thirty, and neither of its two small switches is the cause. Works close younger
  than the port's because they stop counting as building in their first year. The worst year
  is a building wave buying the year's timber, with the harvest second. Example
  `examples/trace_works.rs`.
- `tr-steel.md` (at a6ff0ec, sixteen islands): the steel rail mill is held at half the rolling
  mill's skill by `machines.inherit` and never makes enough to climb past it (given that skill
  alone it takes the trade); cheaper ore since the mines widens its gap. The converter's share
  in 1880 is set by the draw twice over, once for the converter and once for crucible steel,
  which stalls when found late. Inventions' dates wander mostly by the shallow yearly chance of
  discovery; a steeper, quicker one cuts the spread to a third. Example
  `examples/steel_trace.rs`.
- `tr-cycles.md` (at a6ff0ec, islands 1 to 16, and 17 to 24 for the smaller room): the
  building cycle (three quarters of timber's swing is orders for plant, in lumps the yearly
  review makes; the land's slow hands are a quarter more), the famine of tools (the engine
  works' plant on a trade households had left, held long by the smithies' room), and which goods
  swing most and why; more than a dozen cures tried, none kept. Example
  `examples/trace_cycles.rs`.
- `tr-urban.md` (at a6ff0ec): why the towns stop at four people in ten (the share is 0.04 plus
  0.8 of the works' share of the hands, the port's count; the quarrymen and the food bill hold
  it), why founding stops in the 1840s (carting saved at a freight fixed in money, weighed
  against money wages that treble), and what limits a town in its fastest decade (its market
  area and a weak pull, not housing). Example `examples/tr_urban.rs`.
- `tr-loose-a.md` (at 8bf4505): four loose ends the traces and tables left. The merchants'
  debts are the price rule's stock markup acting on services, which spoil: a town that brings
  its services in gets months of them at once and loses what rots, and with no markup on
  services the debt is 1.4 per cent of the money and not 45. The runaway wages are a mine
  hiring past its town's hands after a price jumps at a year's turn, which rationing hires to
  the hands there are ends (the pass's stage 2 since). Two in five twin works are the works of
  1749 put on one cell; and a family's mill was closed by the cash of its own extension, sent
  back as dividends before the plant was paid for. Example `examples/loose_ends.rs`.
- `tr-loose-b.md` (at 8bf4505, and on up to 64 islands): four more. Kept town ground's sure
  costs are output per head and works, and its cost to the real wage is mostly the 1750
  basket's; crucible steel found late stalls because cheap iron has already made tools cheap;
  railways leave fewer people because they put more of them in towns, where they die faster,
  and the port's merchants take their share of the gain through a wider margin the game's
  rules compete away; and the shortages later in the year are the bottom of the merchants'
  stock cycle, a month's buyers drawing only on the store as the month began. Examples
  `crucible_late.rs`, `spring_hunger.rs`, `kept_ground.rs` and `merchants_take.rs`.

## The tables of sensitivity (12 November 2026)

The six tables for the pass over all the rules. Each on the game's rules at a6ff0ec, with six families,
on eight islands and sixteen where a result is leaned on. A run is chaotic: by 1900 a change too
small to matter moves an island as far as the islands differ from one another
(`sens-mines.md`), so each table reads a setting's two values against each other, island for
island, and not against one baseline.

- `sens-prices.md`: the merchants' prices and the Engel curve. `engel.y_ref` matters most
  (towns against people, grain's calm and famines); the island now starts at 1.5 to 1.6
  bundles, not 2. With the harvests on, the markup mostly sets how hard the farm hands answer
  the weather (the zigzag is 1.12 with them off, under the prices page's bar), and famines
  rise from 0.8. Strength matters less now that incomes reach 8.9 bundles. The merchants' brake
  does nothing near its values, but `glut_months` 2 steadies iron's price on every island.
- `sens-machines.md`: the machines and the harvests, on 8 and 16 islands. The cap on machine
  quality is most of why output per head peaks about 1879; `inherit` is a steep ramp for the
  steel rail between 0.5 and 0.667; wagonways bought where they pay leave no railway;
  `quality_Q0` is `quality_b`'s twin at half and double; the harvest's lesser numbers,
  `adopt_span` and `life_months` do nothing at these sizes.
- `sens-ways.md`: the railways and the canals (912 runs, none failed). The dials size the
  railway network (`districts`, `branch_gap`, `cool_years`, `cost_mult` and the hurdle each move
  1900's line by 250 to 470 km on 15 or 16 islands of 16) and the canals' size and date, but
  none moves output a head or the wage by more than about 3 per cent in a way the islands agree
  on; the island feels the two mechanisms through their switches. The hurdle and the induced
  traffic are one dial; `branch_gap` sits at a peak; no setting stretches the canal rush; and
  the first line's date has a long tail of its own.
- `sens-mines.md`: the mines, under both regimes, and the chronicle's twenty-two thresholds.
  `reach` is the mines' strong number, `sink / payback` one threshold, `cell_limit` nothing; the
  chronicle's feed is `per_decade` and `floor`.
- `sens-people.md`: people and kept town ground, thirty-five settings in all. Births less
  deaths (`growth_top`, `y_still`, `ease`) decide the island's people, and the people decide its
  town ground, its works and output per head late in the century; `found_saving` and the
  donors' limits decide how many towns; `d_old_growth` how much ground they keep. Kept ground no
  longer costs the real wage of 1900 on sixteen islands. Nothing breaks in 864 runs.
- `sens-works.md`: the works' life cycle and carting, every setting on sixteen islands and
  taken to its far ends on eight, with two placebos to say what chance gives (584 runs, none
  failed). Only the year's room (`works.year_room`) moves the economy: how rough the ride is,
  the early iron and the works' share of late income, not the level of the real wage; carting's
  `labor` moves the first fifty years about in proportion and not 1900; the rest set the map or
  the churn. Thirteen runs had one town's wage run away for a few months, early in the century
  and on the same few islands.

## The scorecard's baseline (12 November 2026)

- `scorecard-baseline.md` (at aa9f158, sixteen islands with six families): three cards of the
  scorecard (`docs/rules/scorecard.md`), for the game's rules on `main`, for the port's, and
  for the game's rules with the deeper production tree, and the floor under them: how far a
  sweep's facts move when only chance has changed, from five histories of each side. A fact
  that a single island can tip can move several floors by chance alone, which the page warns
  of. Its example, `examples/chance_floor.rs`, measures the floor. The cards are those of the
  first scorecard, before the pass's twenty aims; the card of the pass is in
  `docs/rules/scorecard.md`, "The card before the pass".

## A reading (12 November 2026)

- `news-read.md`: a century of the island's news read item by item as a player meets it in the
  News panel, on the game's rules at a6ff0ec with six families: islands 7, 1 and 4 whole and the
  rest sampled. What it does well, 22 faults sorted by how much they spoil a reader's trust (an
  overtaking that names a town in fourth place, a railway dated by a later line, inventions that
  "open" a works decades before the first one, twin works closing twice, a death told twice in
  the player's own family), each with the line that writes it and a suggested wording, and 45
  items checked against the record (41 right).
